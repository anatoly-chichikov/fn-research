mod execute;
mod launch;
mod print;
mod seed;
mod support;

use std::path::{Path, PathBuf};

use research_domain::provider::Provider;

use execute::Config;

/// Parsed CLI arguments.
pub struct Parsed {
    /// Command name.
    pub cmd: String,
    /// Remaining positional arguments.
    pub tail: Vec<String>,
    /// Named options.
    pub opts: Options,
}

/// Named CLI options.
pub struct Options {
    /// Research processor.
    pub processor: String,
    /// Result language.
    pub language: String,
    /// Data provider.
    pub provider: Provider,
    /// Output HTML instead of PDF.
    pub html: bool,
}

/// Parse CLI arguments.
pub fn parse(args: &[&str]) -> Parsed {
    let mut processor = "pro".to_string();
    let mut language = "\u{0440}\u{0443}\u{0441}\u{0441}\u{043a}\u{0438}\u{0439}".to_string();
    let mut provider_text = "parallel".to_string();
    let mut html = false;
    let mut positional = Vec::new();
    let mut idx = 0;
    while idx < args.len() {
        match args[idx] {
            "--processor" if idx + 1 < args.len() => {
                idx += 1;
                processor = args[idx].to_string();
            }
            "--language" if idx + 1 < args.len() => {
                idx += 1;
                language = args[idx].to_string();
            }
            "--provider" if idx + 1 < args.len() => {
                idx += 1;
                provider_text = args[idx].to_string();
            }
            "--html" => {
                html = true;
            }
            other => {
                positional.push(other.to_string());
            }
        }
        idx += 1;
    }
    let provider = provider_text
        .parse::<Provider>()
        .unwrap_or_else(|e| panic!("{}", e));
    let opts = Options {
        processor,
        language,
        provider,
        html,
    };
    let cmd = positional.first().cloned().unwrap_or_default();
    let tail: Vec<String> = positional.into_iter().skip(1).collect();
    Parsed { cmd, tail, opts }
}

/// Application instance.
pub struct App {
    root: PathBuf,
    data: PathBuf,
    out: PathBuf,
    conf: Config,
}

impl App {
    /// Create application from root path.
    pub fn new(root: &Path) -> Self {
        let out = root.join("output");
        Self {
            root: root.to_path_buf(),
            data: out.clone(),
            out,
            conf: Config {
                env: Box::new(support::env),
                emit: Box::new(research_pdf::document::env::emit),
                provider: Box::new(|name| {
                    let env = support::env;
                    match name {
                        "valyu" => {
                            let key = env("VALYU_API_KEY");
                            let net = Box::new(research_api::http::Http::new("valyu"));
                            let log = Box::new(research_api::progress::make());
                            let state = Box::new(research_api::valyu::status::Status::new(
                                "https://api.valyu.network/v1",
                                &key,
                                Box::new(research_api::http::Http::new("valyu")),
                                Box::new(research_api::progress::make()),
                            ));
                            Box::new(research_api::valyu::Valyu::new(
                                &key,
                                "https://api.valyu.network/v1",
                                net,
                                log,
                                state,
                            ))
                        }
                        "xai" => {
                            let root = std::env::current_dir().unwrap();
                            let unit: Box<dyn research_api::xai::bridge::Bound> =
                                Box::new(research_api::xai::bridge::NullBound);
                            let opts = serde_json::json!({});
                            Box::new(research_api::xai::xai(&root, unit, &opts))
                        }
                        _ => {
                            let key = env("PARALLEL_API_KEY");
                            let net = Box::new(research_api::http::Http::new("parallel"));
                            let log = Box::new(research_api::progress::make());
                            Box::new(research_api::parallel::Parallel::new(
                                &key,
                                "https://api.parallel.ai",
                                net,
                                log,
                            ))
                        }
                    }
                }),
                cover: Box::new(|topic, path| {
                    use research_image::generator::Generated;
                    let gen = research_image::generator::generator()
                        .map_err(|e| format!("Generator creation failed: {}", e))?;
                    gen.generate(topic, path)
                        .map_err(|e| format!("Cover generation failed: {}", e))?;
                    Ok(())
                }),
            },
        }
    }

    /// Create application with custom config.
    pub fn with_config(root: &Path, conf: Config) -> Self {
        let out = root.join("output");
        Self {
            root: root.to_path_buf(),
            data: out.clone(),
            out,
            conf,
        }
    }
}

/// Object that runs CLI operations.
pub trait Applied {
    /// List sessions.
    fn list(&self);
    /// Show session details.
    fn show(&self, id: &str);
    /// Generate report for session.
    fn generate(&self, id: &str, html: bool);
    /// Create session.
    fn create(&self, topic: &str) -> String;
    /// Create session and run research.
    fn run(&self, topic: &str, query: &str, processor: &str, language: &str, provider: &Provider);
    /// Run research for existing session.
    fn research(&self, id: &str, query: &str, processor: &str, language: &str, provider: &Provider);
}

impl Applied for App {
    fn list(&self) {
        print::enumerate(&self.data);
    }

    fn show(&self, id: &str) {
        print::display(&self.data, id);
    }

    fn generate(&self, id: &str, html: bool) {
        print::render(&self.data, &self.out, id, html);
    }

    fn create(&self, topic: &str) -> String {
        seed::seed(&self.data, topic, "", "", "", "")
    }

    fn run(&self, topic: &str, query: &str, processor: &str, language: &str, provider: &Provider) {
        match launch::launch(
            &self.root, &self.data, &self.out, topic, query, processor, language, provider,
            &self.conf,
        ) {
            Ok(()) => {}
            Err(e) => panic!("{}", e),
        }
    }

    fn research(
        &self,
        id: &str,
        query: &str,
        processor: &str,
        language: &str,
        provider: &Provider,
    ) {
        let repo = research_storage::repository::repo(&self.data);
        let list = research_storage::repository::Loadable::load(&repo);
        let pick = list
            .iter()
            .find(|s| research_domain::session::Sessioned::id(*s).starts_with(id));
        if let Some(item) = pick {
            let mut opts = std::collections::HashMap::new();
            opts.insert("query".to_string(), query.to_string());
            opts.insert("processor".to_string(), processor.to_string());
            opts.insert("language".to_string(), language.to_string());
            opts.insert("provider".to_string(), provider.to_string());
            let updated = research_domain::session::Sessioned::reconfigure(item, &opts);
            research_storage::repository::Mutable::update(&repo, updated);
            execute::execute(&self.root, &self.data, &self.out, id, &self.conf);
        }
    }
}

fn main() {
    let root = std::env::current_dir().unwrap();
    let app = App::new(&root);
    let args: Vec<String> = std::env::args().skip(1).collect();
    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let data = parse(&refs);
    match data.cmd.as_str() {
        "list" => app.list(),
        "show" => app.show(data.tail.first().map(|s| s.as_str()).unwrap_or("")),
        "generate" => app.generate(
            data.tail.first().map(|s| s.as_str()).unwrap_or(""),
            data.opts.html,
        ),
        "create" => {
            app.create(&data.tail.join(" "));
        }
        "run" => app.run(
            data.tail.first().map(|s| s.as_str()).unwrap_or(""),
            data.tail.get(1).map(|s| s.as_str()).unwrap_or(""),
            &data.opts.processor,
            &data.opts.language,
            &data.opts.provider,
        ),
        "research" => app.research(
            data.tail.first().map(|s| s.as_str()).unwrap_or(""),
            data.tail.get(1).map(|s| s.as_str()).unwrap_or(""),
            &data.opts.processor,
            &data.opts.language,
            &data.opts.provider,
        ),
        _ => println!("Unknown command"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::GenericImageView;
    use research_api::traits::Researchable;
    use research_domain::ids;
    use research_domain::pending::Pendinged;
    use research_domain::session::{self, Sessioned};
    use research_domain::task::{self, Tasked};
    use research_storage::organizer::{self, Organized};
    use research_storage::repository::{self, Loadable, Savable};
    use std::sync::{Arc, Mutex};

    struct FakeProvider {
        name: String,
        text: String,
        run: String,
        raw: serde_json::Value,
        log: Arc<Mutex<Vec<String>>>,
    }

    impl Researchable for FakeProvider {
        fn start(&self, _query: &str, _processor: &str) -> String {
            self.log.lock().unwrap().push(self.name.clone());
            self.run.clone()
        }

        fn stream(&self, _id: &str) {}

        fn finish(&self, _id: &str) -> serde_json::Value {
            serde_json::json!({
                "id": &self.run,
                "status": "completed",
                "output": &self.text,
                "basis": [],
                "raw": self.raw.clone()
            })
        }
    }

    fn conf(log: Arc<Mutex<Vec<String>>>, text: &str, raw: serde_json::Value) -> Config {
        let output = text.to_string();
        Config {
            env: Box::new(|_| String::new()),
            emit: Box::new(|_, path| Ok(path.to_path_buf())),
            provider: Box::new(move |name| {
                Box::new(FakeProvider {
                    name: name.to_string(),
                    text: output.clone(),
                    run: format!("{}-run", name),
                    raw: raw.clone(),
                    log: log.clone(),
                })
            }),
            cover: Box::new(|_, _| Ok(())),
        }
    }

    #[test]
    fn the_cli_parses_command() {
        let mut rng = ids::ids(25001);
        let text = ids::cyrillic(&mut rng, 6);
        let data = parse(&["create", &text]);
        assert_eq!("create", data.cmd, "CLI command was not parsed");
    }

    #[test]
    fn the_cli_parses_options() {
        let mut rng = ids::ids(25002);
        let topic = ids::cyrillic(&mut rng, 6);
        let query = ids::cyrillic(&mut rng, 7);
        let language = ids::cyrillic(&mut rng, 4);
        let data = parse(&[
            "run",
            &topic,
            &query,
            "--processor",
            "ultra",
            "--language",
            &language,
            "--provider",
            "valyu",
        ]);
        let result = data.opts.processor == "ultra"
            && data.opts.language == language
            && data.opts.provider == Provider::Valyu
            && !data.opts.html;
        assert!(result, "Options were not parsed");
    }

    #[test]
    fn the_application_run_forwards_parameters() {
        let mut rng = ids::ids(25003);
        let topic = ids::cyrillic(&mut rng, 6);
        let query = ids::greek(&mut rng, 7);
        let language = ids::greek(&mut rng, 4);
        let text = ids::cyrillic(&mut rng, 6);
        let log = Arc::new(Mutex::new(Vec::new()));
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let app = App::with_config(root, conf(log.clone(), &text, serde_json::json!({})));
        std::fs::create_dir_all(root.join("output")).unwrap();
        app.run(&topic, &query, "pro", &language, &Provider::Parallel);
        let repo = repository::repo(&root.join("output"));
        let list = repo.load();
        let pick = list.first().unwrap();
        assert_eq!(topic, pick.topic(), "Run did not forward topic");
    }

    #[test]
    #[should_panic(expected = "is not a valid valyu processor")]
    fn the_application_run_rejects_valyu_lite_processor() {
        let mut rng = ids::ids(25006);
        let topic = ids::cyrillic(&mut rng, 6);
        let query = ids::greek(&mut rng, 7);
        let language = ids::greek(&mut rng, 4);
        let text = ids::cyrillic(&mut rng, 6);
        let log = Arc::new(Mutex::new(Vec::new()));
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("output")).unwrap();
        let app = App::with_config(root, conf(log, &text, serde_json::json!({})));
        app.run(&topic, &query, "lite", &language, &Provider::Valyu);
    }

    #[test]
    #[should_panic(expected = "is not a valid provider")]
    fn the_application_rejects_all_provider() {
        let _ = parse(&["run", "topic", "query", "--provider", "all"]);
    }

    #[test]
    fn the_application_skips_cover_when_key_missing() {
        let mut rng = ids::ids(25005);
        let topic = ids::cyrillic(&mut rng, 6);
        let query = ids::greek(&mut rng, 7);
        let language = ids::hiragana(&mut rng, 4);
        let run = ids::arabic(&mut rng, 8);
        let text = ids::cyrillic(&mut rng, 12);
        let stamp = task::format(&chrono::Local::now().naive_local());
        let ident = ids::uuid(&mut rng);
        let entry = serde_json::json!({
            "run_id": run,
            "query": query,
            "processor": "pro",
            "language": language,
            "provider": "parallel",
        });
        let sess = session::session(&serde_json::json!({
            "id": ident,
            "topic": topic,
            "tasks": [],
            "created": stamp,
            "pending": entry,
        }));
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let out = root.join("output");
        std::fs::create_dir_all(&out).unwrap();
        let store = repository::repo(&out);
        store.save(&[sess.clone()]);
        let log = Arc::new(Mutex::new(Vec::new()));
        let app = App::with_config(root, conf(log, &text, serde_json::json!({})));
        let token = &ident[..8];
        app.research(token, &query, "pro", &language, &Provider::Parallel);
        let org = organizer::organizer(&out);
        let name = org.name(sess.created(), sess.topic(), sess.id());
        let cover = org.cover(&name, "parallel");
        assert!(
            !cover.exists(),
            "Cover image was generated despite missing key"
        );
    }

    #[test]
    fn the_application_writes_raw_response() {
        let mut rng = ids::ids(25007);
        let topic = ids::cyrillic(&mut rng, 6);
        let query = ids::greek(&mut rng, 7);
        let language = ids::hiragana(&mut rng, 4);
        let run = ids::arabic(&mut rng, 8);
        let text = ids::cyrillic(&mut rng, 12);
        let stamp = task::format(&chrono::Local::now().naive_local());
        let ident = ids::uuid(&mut rng);
        let key = ids::cyrillic(&mut rng, 6);
        let value = ids::greek(&mut rng, 6);
        let raw = serde_json::json!({ key.clone(): value });
        let entry = serde_json::json!({
            "run_id": run,
            "query": query,
            "processor": "ultra",
            "language": language,
            "provider": "parallel",
        });
        let sess = session::session(&serde_json::json!({
            "id": ident,
            "topic": topic,
            "tasks": [],
            "created": stamp,
            "pending": entry,
        }));
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let out = root.join("output");
        std::fs::create_dir_all(&out).unwrap();
        let store = repository::repo(&out);
        store.save(&[sess.clone()]);
        let log = Arc::new(Mutex::new(Vec::new()));
        let app = App::with_config(root, conf(log, &text, raw.clone()));
        let token = &ident[..8];
        app.research(token, &query, "ultra", &language, &Provider::Parallel);
        let org = organizer::organizer(&out);
        let name = org.name(sess.created(), sess.topic(), sess.id());
        let folder = org.folder(&name, "parallel");
        let path = folder.join("response-parallel.json");
        let content = std::fs::read_to_string(&path).unwrap();
        let data: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(raw, data, "Raw response did not match stored response");
    }

    #[test]
    fn the_application_continues_after_cover_failure() {
        let mut rng = ids::ids(25008);
        let topic = ids::cyrillic(&mut rng, 6);
        let query = ids::greek(&mut rng, 7);
        let language = ids::hiragana(&mut rng, 4);
        let run = ids::arabic(&mut rng, 8);
        let text = ids::cyrillic(&mut rng, 12);
        let stamp = task::format(&chrono::Local::now().naive_local());
        let ident = ids::uuid(&mut rng);
        let entry = serde_json::json!({
            "run_id": run,
            "query": query,
            "processor": "fast",
            "language": language,
            "provider": "valyu",
        });
        let sess = session::session(&serde_json::json!({
            "id": ident,
            "topic": topic,
            "tasks": [],
            "created": stamp,
            "pending": entry,
        }));
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let out = root.join("output");
        std::fs::create_dir_all(&out).unwrap();
        let store = repository::repo(&out);
        store.save(&[sess.clone()]);
        let log = Arc::new(Mutex::new(Vec::new()));
        let key_text = ids::latin(&mut rng, 6);
        let output = text.clone();
        let app = App::with_config(
            root,
            Config {
                env: Box::new(move |k| {
                    if k == "GEMINI_API_KEY" {
                        key_text.clone()
                    } else {
                        String::new()
                    }
                }),
                emit: Box::new(|_, path| {
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent).ok();
                    }
                    std::fs::write(path, b"pdf").ok();
                    Ok(path.to_path_buf())
                }),
                provider: Box::new(move |name| {
                    Box::new(FakeProvider {
                        name: name.to_string(),
                        text: output.clone(),
                        run: format!("{}-run", name),
                        raw: serde_json::json!({}),
                        log: log.clone(),
                    })
                }),
                cover: Box::new(|_, _| {
                    Err("Cover generation failed model=none status=none".to_string())
                }),
            },
        );
        let token = &ident[..8];
        app.research(token, &query, "fast", &language, &Provider::Valyu);
        let org = organizer::organizer(&out);
        let name = org.name(sess.created(), sess.topic(), sess.id());
        let path = org.report(&name, "valyu");
        assert!(
            path.exists(),
            "Report was not generated after cover failure"
        );
    }

    #[test]
    fn the_application_saves_brief_in_session() {
        let mut rng = ids::ids(25009);
        let topic = ids::cyrillic(&mut rng, 6);
        let query = format!(
            "{}\n\n{}",
            ids::cyrillic(&mut rng, 5),
            ids::greek(&mut rng, 7)
        );
        let language = ids::cyrillic(&mut rng, 4);
        let _run = ids::arabic(&mut rng, 8);
        let text = ids::cyrillic(&mut rng, 12);
        let stamp = task::format(&chrono::Local::now().naive_local());
        let ident = ids::uuid(&mut rng);
        let sess = session::session(&serde_json::json!({
            "id": ident,
            "topic": topic,
            "tasks": [],
            "created": stamp,
        }));
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let out = root.join("output");
        std::fs::create_dir_all(&out).unwrap();
        let store = repository::repo(&out);
        store.save(&[sess.clone()]);
        let log = Arc::new(Mutex::new(Vec::new()));
        let app = App::with_config(root, conf(log, &text, serde_json::json!({})));
        let token = &ident[..8];
        app.research(token, &query, "pro", &language, &Provider::Parallel);
        let loaded = store.load();
        let hit = loaded.iter().find(|s| s.id() == ident).unwrap();
        let run = &hit.tasks()[0];
        let brief = run.brief();
        assert!(!brief.title.is_empty(), "Brief was not stored in session");
    }

    #[test]
    fn the_application_preserves_brief_structure() {
        let mut rng = ids::ids(25012);
        let topic = ids::cyrillic(&mut rng, 6);
        let text_val = ids::greek(&mut rng, 6);
        let leaf = ids::hiragana(&mut rng, 6);
        let node = ids::armenian(&mut rng, 6);
        let language = ids::cyrillic(&mut rng, 4);
        let run = ids::arabic(&mut rng, 8);
        let output = ids::cyrillic(&mut rng, 6);
        let stamp = task::format(&chrono::Local::now().naive_local());
        let ident = ids::uuid(&mut rng);
        let brief = serde_json::json!({
            "title": topic,
            "language": language,
            "questions": [{
                "scope": text_val,
                "details": [
                    {"scope": leaf, "details": []},
                    {"scope": node, "details": []},
                ]
            }]
        });
        let entry = serde_json::json!({
            "run_id": run,
            "brief": brief,
            "processor": "pro",
            "language": language,
            "provider": "parallel",
        });
        let sess = session::session(&serde_json::json!({
            "id": ident,
            "topic": topic,
            "tasks": [],
            "created": stamp,
            "pending": entry,
        }));
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let out = root.join("output");
        std::fs::create_dir_all(&out).unwrap();
        let store = repository::repo(&out);
        store.save(&[sess.clone()]);
        let log = Arc::new(Mutex::new(Vec::new()));
        let pend = research_domain::pending::pending(&entry);
        let query = pend.query();
        let app = App::with_config(root, conf(log, &output, serde_json::json!({})));
        let token = &ident[..8];
        app.research(token, &query, "pro", &language, &Provider::Parallel);
        let loaded = store.load();
        let hit = loaded.iter().find(|s| s.id() == ident).unwrap();
        let run = &hit.tasks()[0];
        let brief = run.brief();
        let first = &brief.questions[0];
        let nested = !first.details.is_empty();
        assert!(nested, "Nested brief questions were not preserved");
    }

    /// Render PDF pages to PNG screenshots using pdftoppm.
    fn screens(
        pdf: &std::path::Path,
        folder: &std::path::Path,
        dpi: u32,
    ) -> Vec<image::DynamicImage> {
        let prefix = folder.join("page");
        let output = std::process::Command::new("pdftoppm")
            .args(["-png", "-r", &dpi.to_string()])
            .arg(pdf)
            .arg(&prefix)
            .output()
            .expect("pdftoppm must be installed");
        assert!(
            output.status.success(),
            "pdftoppm failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let mut files: Vec<_> = std::fs::read_dir(folder)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|e| e == "png").unwrap_or(false))
            .collect();
        files.sort();
        files.iter().map(|p| image::open(p).unwrap()).collect()
    }

    /// Return first mismatching page (1-indexed), or 0 if all match.
    fn mismatch(left: &[image::DynamicImage], right: &[image::DynamicImage]) -> usize {
        if left.len() != right.len() {
            return left.len().min(right.len()) + 1;
        }
        for (idx, (one, two)) in left.iter().zip(right.iter()).enumerate() {
            if one.dimensions() != two.dimensions() || one.as_bytes() != two.as_bytes() {
                return idx + 1;
            }
        }
        0
    }

    #[test]
    #[ignore]
    fn the_application_generates_pdf_screenshots() {
        let base = std::path::PathBuf::from("baseline-research");
        let brief_text = std::fs::read_to_string(base.join("brief-parallel.ron")).unwrap();
        let brief_obj = research_domain::brief::decode(&brief_text).unwrap();
        let raw: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(base.join("response-parallel.json")).unwrap(),
        )
        .unwrap();
        let cover_src = base.join("cover-parallel.jpg");
        let gold = base.join("baseline.pdf");
        let output_obj = raw.get("output").unwrap();
        let content = output_obj
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let basis = output_obj
            .get("basis")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let run_info = raw.get("run").unwrap();
        let run_code = run_info
            .get("run_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let normalized = serde_json::json!({
            "id": run_code,
            "status": "completed",
            "output": content,
            "basis": basis,
            "raw": raw
        });
        let mut rng = ids::ids(25011);
        let lang = ids::cyrillic(&mut rng, 6);
        let head = ids::ascii(&mut rng, 6);
        let ident = ids::uuid(&mut rng);
        let stamp = task::format(&chrono::Local::now().naive_local());
        let entry = serde_json::json!({
            "run_id": run_code,
            "brief": research_domain::brief::data(&brief_obj),
            "processor": "pro",
            "language": lang,
            "provider": "parallel",
        });
        let sess = session::session(&serde_json::json!({
            "id": ident,
            "topic": "Clojure production pain points",
            "tasks": [],
            "created": stamp,
            "pending": entry,
        }));
        let pend = research_domain::pending::pending(&entry);
        let query = pend.query();
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let out = root.join("output");
        std::fs::create_dir_all(&out).unwrap();
        let store = repository::repo(&out);
        store.save(&[sess.clone()]);
        unsafe {
            std::env::set_var("REPORT_FOR", "Anatoly Chichikov");
        }
        let cover_path = cover_src.clone();
        let norm = normalized.clone();
        let app = App::with_config(
            root,
            Config {
                env: Box::new(|key| {
                    if key == "GEMINI_API_KEY" {
                        "key".to_string()
                    } else {
                        String::new()
                    }
                }),
                emit: Box::new(research_pdf::document::env::emit),
                provider: Box::new(move |name| {
                    let data = norm.clone();
                    let tag = name.to_string();
                    Box::new(FakeProvider {
                        name: tag.clone(),
                        text: data["output"].as_str().unwrap_or("").to_string(),
                        run: data["id"].as_str().unwrap_or("").to_string(),
                        raw: data["raw"].clone(),
                        log: Arc::new(Mutex::new(Vec::new())),
                    })
                }),
                cover: Box::new(move |_, target| {
                    std::fs::copy(&cover_path, target)
                        .map(|_| ())
                        .map_err(|e| format!("Copy failed: {}", e))
                }),
            },
        );
        app.research(&ident[..8], &query, "pro", &lang, &Provider::Parallel);
        let org = organizer::organizer(&out);
        let name = org.name(sess.created(), sess.topic(), sess.id());
        let path = org.report(&name, "parallel");
        assert!(path.exists(), "Generated PDF was not created");
        let cache = std::path::PathBuf::from("tmp_cache");
        let folder = cache.join(format!("pdf-screens-{}", head));
        let left = folder.join("baseline");
        let right = folder.join("generated");
        std::fs::create_dir_all(&left).unwrap();
        std::fs::create_dir_all(&right).unwrap();
        let lefts = screens(&gold, &left, 150);
        let rights = screens(&path, &right, 150);
        let miss = mismatch(&lefts, &rights);
        let text = if miss == 0 {
            String::new()
        } else {
            let name = format!("page-{:03}.png", miss);
            format!(
                "Page {} did not match baseline baseline={} generated={}",
                miss,
                left.join(&name).display(),
                right.join(&name).display()
            )
        };
        assert_eq!(0, miss, "{}", text);
    }
}
