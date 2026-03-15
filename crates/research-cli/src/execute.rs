use std::path::{Path, PathBuf};

use research_api::response::{self, Responded};
use research_api::traits::Researchable;
use research_domain::brief::{self, Brief};
use research_domain::pending::{self, Pendinged};
use research_domain::processor::Processor;
use research_domain::provider::{Labeled, Provider};
use research_domain::result::Serialized;
use research_domain::session::{ResearchSession, Sessioned};
use research_domain::task;
use research_pdf::document::{self, Rendered};
use research_pdf::palette;
use research_storage::organizer::{self, Organized};
use research_storage::repository::{self, Loadable, Mutable};

use crate::support;

/// Injectable dependencies for execution.
#[allow(clippy::type_complexity)]
pub struct Config {
    /// Environment variable lookup.
    pub env: Box<dyn Fn(&str) -> String>,
    /// PDF generation from HTML.
    pub emit: Box<dyn Fn(&str, &Path) -> Result<PathBuf, String>>,
    /// Provider factory by name.
    pub provider: Box<dyn Fn(&str) -> Box<dyn Researchable>>,
    /// Cover image generation.
    pub cover: Box<dyn Fn(&str, &Path) -> Result<(), String>>,
}

/// Run research for session.
pub fn execute(_root: &Path, data: &Path, out: &Path, id: &str, conf: &Config) {
    let repo = repository::repo(data);
    let list = repo.load();
    let pick = list.iter().find(|s| s.id().starts_with(id));
    let pick = match pick {
        Some(s) => s.clone(),
        None => {
            println!("Session not found: {}", id);
            return;
        }
    };
    println!("Session: {}", pick.topic());
    if let Some(pend) = pick.pending() {
        resume(&repo, &pick, pend, out, conf);
    } else {
        start(&repo, &pick, out, conf);
    }
}

/// Resume pending research run.
fn resume(
    repo: &repository::Repository,
    session: &ResearchSession,
    pend: &research_domain::pending::PendingRun,
    out: &Path,
    conf: &Config,
) {
    let run = pend.id().to_string();
    let query = pend.query();
    let processor = *pend.processor();
    let language = pend.language().to_string();
    let provider = *pend.provider();
    let exec = (conf.provider)(&provider.to_string());
    println!("Resuming run: {}", &run[..16.min(run.len())]);
    println!("Query: {}", query);
    println!("Processor: {}", processor);
    println!("Streaming progress");
    exec.stream(&run);
    println!("Fetching result");
    let raw = exec.finish(&run);
    let resp = response::response(&raw);
    let updated = session.reset();
    repo.update(updated.clone());
    complete(
        repo,
        &updated,
        session,
        pend.brief(),
        &resp,
        &provider,
        &language,
        &processor,
        out,
        conf,
    );
}

/// Start new research run.
fn start(repo: &repository::Repository, session: &ResearchSession, out: &Path, conf: &Config) {
    let query = session.query().to_string();
    let processor = *session.processor();
    let language = session.language().to_string();
    let provider = *session.provider();
    let exec = (conf.provider)(&provider.to_string());
    let run = exec.start(&query, &processor.to_string());
    let pend_data = serde_json::json!({
        "run_id": run,
        "query": query,
        "processor": processor.to_string(),
        "language": language,
        "provider": provider.to_string(),
        "topic": session.topic(),
    });
    let pend = pending::pending(&pend_data);
    let content = pend.brief().clone();
    let state = session.start(pend);
    repo.update(state);
    println!("Query: {}", query);
    println!("Processor: {}", processor);
    println!("Language: {}", language);
    println!("Research started: {}", run);
    println!("Streaming progress");
    exec.stream(&run);
    println!("Fetching result");
    let raw = exec.finish(&run);
    let resp = response::response(&raw);
    let updated = session.reset();
    repo.update(updated.clone());
    complete(
        repo, &updated, session, &content, &resp, &provider, &language, &processor, out, conf,
    );
}

/// Complete research: save response, create task, generate cover and PDF.
#[allow(clippy::too_many_arguments)]
fn complete(
    repo: &repository::Repository,
    updated: &ResearchSession,
    original: &ResearchSession,
    content: &Brief,
    resp: &response::Response,
    provider: &Provider,
    language: &str,
    processor: &Processor,
    out: &Path,
    conf: &Config,
) {
    let tag = provider.to_string();
    let org = organizer::organizer(out);
    let name = org.name(original.created(), original.topic(), original.id());
    org.response(&name, &tag, resp.raw());
    support::store(&name, &tag, resp.raw(), out);
    let text = resp.text();
    let sources = resp.sources();
    let pack: serde_json::Value = serde_json::json!({
        "summary": text,
        "sources": sources.iter()
            .map(|s| serde_json::to_value(s.data()).unwrap())
            .collect::<Vec<_>>()
    });
    let now = chrono::Local::now().naive_local();
    let stamp = task::format(&now);
    let task_data = serde_json::json!({
        "id": uuid::Uuid::new_v4().to_string(),
        "query": resp.id(),
        "status": "completed",
        "language": language,
        "service": provider.label(),
        "processor": processor.to_string(),
        "brief": brief::data(content),
        "created": stamp,
        "result": pack,
    });
    let run = task::task(&task_data);
    let result = updated.extend(run);
    repo.update(result.clone());
    let cover = org.cover(&name, &tag);
    let folder = org.folder(&name, &tag);
    let count = sources.len();
    println!("Response saved: {}", folder.display());
    println!("Results saved: {} sources", count);
    let key = (conf.env)("GEMINI_API_KEY");
    if key.is_empty() {
        println!("Gemini API key not set skipping image generation");
    } else {
        println!("Generating cover image");
        match (conf.cover)(result.topic(), &cover) {
            Ok(()) => println!("Cover generated: {}", cover.display()),
            Err(e) => println!("Cover generation failed: {}", e),
        }
    }
    let path = org.report(&name, &tag);
    let pal = palette::palette();
    let cover_opt = if cover.exists() { Some(cover) } else { None };
    let doc = document::Document::new(&result, &pal, cover_opt, out);
    let html = doc.render();
    match (conf.emit)(&html, &path) {
        Ok(p) => println!("PDF generated: {}", p.display()),
        Err(e) => println!("PDF generation failed: {}", e),
    }
}
