pub mod citations;
pub mod data;
pub mod env;
pub mod sources;
pub mod tasks;
pub mod text;

use std::path::{Path, PathBuf};

use research_domain::brief::Brief;
use research_domain::pending::Pendinged;
use research_domain::session::Sessioned;
use research_domain::task::Tasked;

use crate::palette::Colored;
use crate::style::{self, Styled};

/// Object with author signature.
pub trait Signed {
    /// Return HTML signature.
    fn html(&self) -> String;
}

/// Object that can render document.
pub trait Rendered {
    /// Return HTML document.
    fn render(&self) -> String;
}

/// Object that can export to file.
pub trait Exported {
    /// Save PDF to path.
    fn save(&self, path: &Path) -> Result<PathBuf, String>;
    /// Save HTML to path.
    fn page(&self, path: &Path) -> PathBuf;
}

/// Author signature.
pub struct Signature {
    name: String,
}

impl Signature {
    /// Create signature from author name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl Signed for Signature {
    fn html(&self) -> String {
        let repo = "https://github.com/anatoly-chichikov/fn-research";
        let link = format!("<a href=\"{}\"><b>fn research</b></a>", repo);
        let mark = format!("<span class=\"signature-mark\">{}</span>", link);
        let text = if self.name.is_empty() {
            format!("AI generated report with {}", mark)
        } else {
            format!(
                "AI generated report for <span class=\"author\">{}</span> with {}",
                self.name, mark
            )
        };
        format!("{}<br>May contain inaccuracies, please verify", text)
    }
}

/// Document instance for PDF rendering.
pub struct Document<'a> {
    session: &'a dyn Sessioned,
    palette: &'a dyn Colored,
    cover: Option<PathBuf>,
    root: PathBuf,
    author_fn: Box<dyn Fn() -> String>,
}

impl<'a> Document<'a> {
    /// Create document from session, palette, cover and root.
    pub fn new(
        session: &'a dyn Sessioned,
        palette: &'a dyn Colored,
        cover: Option<PathBuf>,
        root: &Path,
    ) -> Self {
        Self {
            session,
            palette,
            cover,
            root: root.to_path_buf(),
            author_fn: Box::new(env::author),
        }
    }

    /// Create document with custom author function.
    pub fn with_author(
        session: &'a dyn Sessioned,
        palette: &'a dyn Colored,
        cover: Option<PathBuf>,
        root: &Path,
        author_fn: Box<dyn Fn() -> String>,
    ) -> Self {
        Self {
            session,
            palette,
            cover,
            root: root.to_path_buf(),
            author_fn,
        }
    }
}

/// Return research title from session topic or brief.
pub fn title(session: &dyn Sessioned) -> String {
    let topic = session.topic();
    if !topic.trim().is_empty() {
        return topic.to_string();
    }
    let tasks = session.tasks();
    let head = tasks.first();
    let pending = session.pending();
    let info: Option<&Brief> = if let Some(task) = head {
        Some(task.brief())
    } else {
        pending.map(|p| p.brief())
    };
    match info {
        Some(brief) => {
            let parsed = &brief.title;
            if parsed.trim().is_empty() {
                String::new()
            } else {
                parsed.clone()
            }
        }
        None => String::new(),
    }
}

/// Render brief section.
pub fn brief(session: &dyn Sessioned) -> String {
    let tasks = session.tasks();
    let head = tasks.first();
    let pending = session.pending();
    let info: Option<&Brief> = if let Some(task) = head {
        Some(task.brief())
    } else {
        pending.map(|p| p.brief())
    };
    let (items, topic) = match info {
        Some(brief) => (brief.questions.clone(), brief.title.clone()),
        None => (Vec::new(), String::new()),
    };
    let html = if !items.is_empty() {
        let topic_html = if topic.trim().is_empty() {
            String::new()
        } else {
            format!("<p>{}</p>", text::escape(&topic))
        };
        format!("{}{}", topic_html, text::outline(&items))
    } else if topic.trim().is_empty() {
        String::new()
    } else {
        let content = text::listify(&topic);
        let content = text::normalize(&content);
        let content = text::rule(&content);
        let content = citations::stars(&content);
        let html = text::markdown(&content);
        let html = citations::tables(&html);
        let html = citations::codeindent(&html);
        citations::backslash(&html)
    };
    if html.trim().is_empty() {
        String::new()
    } else {
        format!(
            "<div class=\"brief\"><div class=\"container\">\
             <h1>Exploration Brief</h1>\
             <div class=\"query\">{}</div></div></div>",
            html
        )
    }
}

/// Render cover image html.
fn coverimage(cover: &Option<PathBuf>) -> String {
    match cover {
        Some(path) if path.exists() => {
            format!(
                "<div class=\"cover-image\"><img src=\"file://{}\" alt=\"Cover\" /></div>",
                path.display()
            )
        }
        _ => String::new(),
    }
}

impl Rendered for Document<'_> {
    fn render(&self) -> String {
        let (content, _urls) = tasks::tasks(&self.root, self.session);
        let catalog = catalog(&self.root, self.session);
        let extra = sources::section(&catalog);
        let author_name = (self.author_fn)();
        let sign = Signature::new(&author_name);
        let note = sign.html();
        let css_root = std::env::var("RESOURCES_DIR")
            .unwrap_or_else(|_| format!("{}/../../resources", env!("CARGO_MANIFEST_DIR")));
        let css_style = style::style(self.palette, &css_root);
        let css = css_style.css();
        let stamp = self.session.created().format("%Y-%m-%d").to_string();
        let brief_html = brief(self.session);
        let body = format!(
            "{}<div class=\"container content\"><div class=\"tasks\">{}</div></div>\
             <div class=\"container\">{}</div>",
            brief_html, content, extra
        );
        let data = text::anchors(&body);
        let toc_html = text::toc(&data.items);
        let body = data.html;
        let escaped_title = text::escape(&text::heading(&title(self.session)));
        format!(
            "<!DOCTYPE html><html lang=\"en\"><head>\
             <meta charset=\"UTF-8\" />\
             <title>{}</title><style>{}</style></head><body>\
             <div class=\"page-footer\">{}</div>\
             <div class=\"intro\">{}\
             <div class=\"intro-content\"><h1>{}</h1>\
             <div class=\"meta\"><p class=\"subtitle\">{}</p>\
             <p class=\"date\">{}</p></div></div></div>\
             {}{}</body></html>",
            escaped_title,
            css,
            note,
            coverimage(&self.cover),
            escaped_title,
            note,
            stamp,
            toc_html,
            body
        )
    }
}

impl Exported for Document<'_> {
    fn save(&self, path: &Path) -> Result<PathBuf, String> {
        let html = self.render();
        env::emit(&html, path)
    }

    fn page(&self, path: &Path) -> PathBuf {
        let html = self.render();
        std::fs::write(path, html.as_bytes()).expect("Failed to write HTML file");
        path.to_path_buf()
    }
}

/// Collect sources from session tasks.
fn catalog(root: &Path, session: &dyn Sessioned) -> Vec<sources::SourceEntry> {
    let list = session.tasks();
    let mut result: Vec<sources::SourceEntry> = Vec::new();
    for task in list {
        let (_text, task_sources) = data::resultmap(root, session, task);
        let name = env::provider(task);
        for source in task_sources {
            let link = text::trim(source.url());
            if !link.is_empty() {
                result.push(sources::SourceEntry {
                    source: Box::new(source),
                    provider: name.clone(),
                });
            }
        }
    }
    result
}

/// Create document instance.
pub fn document<'a>(
    session: &'a dyn Sessioned,
    palette: &'a dyn Colored,
    cover: Option<PathBuf>,
    root: &Path,
) -> Document<'a> {
    Document::new(session, palette, cover, root)
}

use research_domain::result::Sourced;

#[cfg(test)]
mod tests {
    use super::*;
    use research_domain::ids;
    use research_domain::result::{CitationSource, Report, ResearchReport};
    use research_domain::session;
    use research_domain::task;

    fn stamp() -> String {
        let now = chrono::Local::now().naive_local();
        task::format(&now)
    }

    fn session_from(topic: &str, entries: Vec<serde_json::Value>) -> session::ResearchSession {
        let data = serde_json::json!({
            "topic": topic,
            "tasks": entries,
            "created": stamp()
        });
        session::session(&data)
    }

    fn task_entry(
        query: &str,
        status: &str,
        service: &str,
        result: Option<Report>,
    ) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "query".to_string(),
            serde_json::Value::String(query.to_string()),
        );
        map.insert(
            "status".to_string(),
            serde_json::Value::String(status.to_string()),
        );
        map.insert(
            "service".to_string(),
            serde_json::Value::String(service.to_string()),
        );
        map.insert("created".to_string(), serde_json::Value::String(stamp()));
        if let Some(report) = result {
            let data = report.data();
            map.insert("result".to_string(), serde_json::to_value(data).unwrap());
        }
        serde_json::Value::Object(map)
    }

    #[test]
    fn the_document_render_contains_doctype() {
        let mut rng = ids::ids(24001);
        let topic = ids::cyrillic(&mut rng, 6);
        let item = session_from(&topic, vec![]);
        let pal = crate::palette::palette();
        let root = std::path::PathBuf::from("output");
        let doc = Document::new(&item, &pal, None, &root);
        let html = doc.render();
        assert!(
            html.contains("<!DOCTYPE html>"),
            "Rendered document did not contain DOCTYPE"
        );
    }

    #[test]
    fn the_document_render_contains_topic() {
        let mut rng = ids::ids(24003);
        let topic = ids::hiragana(&mut rng, 6);
        let head = text::heading(&topic);
        let item = session_from(&topic, vec![]);
        let pal = crate::palette::palette();
        let root = std::path::PathBuf::from("output");
        let doc = Document::new(&item, &pal, None, &root);
        let html = doc.render();
        assert!(
            html.contains(&text::escape(&head)),
            "Rendered document did not contain heading"
        );
    }

    #[test]
    fn the_document_heading_uppercases_initial_letter() {
        let mut rng = ids::ids(24005);
        let value = ids::cyrillic(&mut rng, 6);
        let head: String = value.chars().next().unwrap().to_uppercase().collect();
        let tail: String = value.chars().skip(1).collect();
        let goal = format!("{}{}", head, tail);
        let result = text::heading(&value);
        assert_eq!(goal, result, "Heading did not uppercase initial letter");
    }

    #[test]
    fn the_document_includes_palette_colors() {
        let mut rng = ids::ids(24007);
        let topic = ids::hiragana(&mut rng, 6);
        let item = session_from(&topic, vec![]);
        let pal = crate::palette::palette();
        let root = std::path::PathBuf::from("output");
        let doc = Document::new(&item, &pal, None, &root);
        let html = doc.render();
        let colors = [
            "#F6EFE3", "#1C2430", "#193D5E", "#3A5F88", "#6B645A", "#E3D9C6", "#D04A35", "#1C2833",
            "#DDD5C5", "#BFB5A3",
        ];
        let all = colors.iter().all(|c| html.contains(c));
        assert!(
            all,
            "Rendered document did not include Hokusai palette colors"
        );
    }

    #[test]
    fn the_document_escapes_html() {
        let topic = "<script>alert('xss')</script>";
        let item = session_from(topic, vec![]);
        let pal = crate::palette::palette();
        let root = std::path::PathBuf::from("output");
        let doc = Document::new(&item, &pal, None, &root);
        let html = doc.render();
        assert!(
            html.contains("&lt;script&gt;"),
            "Rendered document did not escape HTML"
        );
    }

    #[test]
    fn the_document_omits_author_when_missing() {
        let mut rng = ids::ids(24011);
        let value = ids::greek(&mut rng, 5);
        let service = ids::hiragana(&mut rng, 4);
        let report = Report::Full(ResearchReport::new(&value, vec![]));
        let entry = task_entry(&value, "completed", &service, Some(report));
        let item = session_from(&value, vec![entry]);
        let pal = crate::palette::palette();
        let root = std::path::PathBuf::from("output");
        let author_fn = Box::new(|| String::new());
        let doc = Document::with_author(&item, &pal, None, &root, author_fn);
        let html = doc.render();
        assert!(
            !html.contains("<span class=\"author\">"),
            "Author span was present"
        );
    }

    #[test]
    fn the_document_renders_author_name() {
        let mut rng = ids::ids(24013);
        let name = ids::cyrillic(&mut rng, 6);
        let value = ids::greek(&mut rng, 5);
        let service = ids::hiragana(&mut rng, 4);
        let report = Report::Full(ResearchReport::new(&value, vec![]));
        let entry = task_entry(&value, "completed", &service, Some(report));
        let item = session_from(&value, vec![entry]);
        let pal = crate::palette::palette();
        let root = std::path::PathBuf::from("output");
        let name_clone = name.clone();
        let author_fn = Box::new(move || name_clone.clone());
        let doc = Document::with_author(&item, &pal, None, &root, author_fn);
        let html = doc.render();
        assert!(html.contains(&name), "Author name was missing");
    }

    #[test]
    fn the_document_title_prefers_session_topic() {
        let mut rng = ids::ids(24015);
        let raw = ids::armenian(&mut rng, 6);
        let item = session_from(&raw, vec![]);
        let head = title(&item);
        assert_eq!(raw, head, "Title did not prefer session topic");
    }

    #[test]
    fn the_document_renders_exploration_brief_title() {
        let mut rng = ids::ids(24017);
        let query = ids::cyrillic(&mut rng, 6);
        let status = ids::cyrillic(&mut rng, 6);
        let _language = ids::cyrillic(&mut rng, 6);
        let service = ids::cyrillic(&mut rng, 6);
        let entry = task_entry(&query, &status, &service, None);
        let item = session_from(&query, vec![entry]);
        let pal = crate::palette::palette();
        let root = std::path::PathBuf::from("output");
        let doc = Document::new(&item, &pal, None, &root);
        let html = doc.render();
        assert!(
            html.contains("Exploration Brief"),
            "Exploration Brief title was missing"
        );
    }

    #[test]
    fn the_document_html_creates_file() {
        let mut rng = ids::ids(24019);
        let dir = std::env::temp_dir().join(format!("doc-{}", ids::uuid(&mut rng)));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!("test-{}.html", ids::uuid(&mut rng)));
        let item = session_from("T", vec![]);
        let pal = crate::palette::palette();
        let root = std::path::PathBuf::from("output");
        let doc = Document::new(&item, &pal, None, &root);
        doc.page(&path);
        assert!(path.exists(), "HTML file was not created");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn the_document_render_contains_synthesis() {
        let mut rng = ids::ids(24021);
        let summary = ids::hiragana(&mut rng, 6);
        let report = Report::Full(ResearchReport::new(&summary, vec![]));
        let entry = task_entry("q", "completed", "parallel.ai", Some(report));
        let item = session_from("T", vec![entry]);
        let pal = crate::palette::palette();
        let root = std::path::PathBuf::from("output");
        let doc = Document::new(&item, &pal, None, &root);
        let html = doc.render();
        assert!(
            html.contains(&summary),
            "Rendered document did not contain synthesis"
        );
    }

    #[test]
    fn the_document_renders_repo_link() {
        let mut rng = ids::ids(24023);
        let name = ids::cyrillic(&mut rng, 6);
        let value = ids::greek(&mut rng, 5);
        let report = Report::Full(ResearchReport::new(&value, vec![]));
        let entry = task_entry(&value, "completed", "parallel.ai", Some(report));
        let item = session_from(&value, vec![entry]);
        let pal = crate::palette::palette();
        let root = std::path::PathBuf::from("output");
        let name_clone = name.clone();
        let author_fn = Box::new(move || name_clone.clone());
        let doc = Document::with_author(&item, &pal, None, &root, author_fn);
        let html = doc.render();
        assert!(html.contains("fn-research"), "Repo link was missing");
    }

    #[test]
    fn the_document_renders_sources_section() {
        let mut rng = ids::ids(24025);
        let head = ids::cyrillic(&mut rng, 6);
        let note = ids::cyrillic(&mut rng, 6);
        let number = ids::digit(&mut rng, 1000);
        let link = format!("https://example.com/{}", number);
        let source = CitationSource::new(&head, &link, &note);
        let report = Report::Full(ResearchReport::new(&head, vec![source]));
        let entry = task_entry(&head, "completed", "valyu.ai", Some(report));
        let item = session_from(&head, vec![entry]);
        let pal = crate::palette::palette();
        let root = std::path::PathBuf::from("output");
        let doc = Document::new(&item, &pal, None, &root);
        let html = doc.render();
        assert!(html.contains("Sources"), "Sources section was missing");
    }
}
