mod execute;
mod launch;
mod print;
mod seed;
mod support;

use std::path::{Path, PathBuf};

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
    pub provider: String,
    /// Output HTML instead of PDF.
    pub html: bool,
}

/// Parse CLI arguments.
pub fn parse(args: &[&str]) -> Parsed {
    let mut opts = Options {
        processor: "pro".to_string(),
        language: "\u{0440}\u{0443}\u{0441}\u{0441}\u{043a}\u{0438}\u{0439}".to_string(),
        provider: "parallel".to_string(),
        html: false,
    };
    let mut positional = Vec::new();
    let mut idx = 0;
    while idx < args.len() {
        match args[idx] {
            "--processor" if idx + 1 < args.len() => {
                idx += 1;
                opts.processor = args[idx].to_string();
            }
            "--language" if idx + 1 < args.len() => {
                idx += 1;
                opts.language = args[idx].to_string();
            }
            "--provider" if idx + 1 < args.len() => {
                idx += 1;
                opts.provider = args[idx].to_string();
            }
            "--html" => {
                opts.html = true;
            }
            other => {
                positional.push(other.to_string());
            }
        }
        idx += 1;
    }
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
    fn run(&self, topic: &str, query: &str, processor: &str, language: &str, provider: &str);
    /// Run research for existing session.
    fn research(&self, id: &str, query: &str, processor: &str, language: &str, provider: &str);
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

    fn run(&self, topic: &str, query: &str, processor: &str, language: &str, provider: &str) {
        match launch::launch(
            &self.root, &self.data, &self.out, topic, query, processor, language, provider,
            &self.conf,
        ) {
            Ok(()) => {}
            Err(e) => panic!("{}", e),
        }
    }

    fn research(&self, id: &str, query: &str, processor: &str, language: &str, provider: &str) {
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
    use research_api::traits::Researchable;
    use research_domain::ids;
    use research_domain::pending::Pendinged;
    use research_domain::session::{self, Sessioned};
    use research_domain::task;
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
        let processor = ids::cyrillic(&mut rng, 5);
        let language = ids::cyrillic(&mut rng, 4);
        let provider = ids::cyrillic(&mut rng, 6);
        let data = parse(&[
            "run",
            &topic,
            &query,
            "--processor",
            &processor,
            "--language",
            &language,
            "--provider",
            &provider,
        ]);
        let result = data.opts.processor == processor
            && data.opts.language == language
            && data.opts.provider == provider
            && !data.opts.html;
        assert!(result, "Options were not parsed");
    }

    #[test]
    fn the_application_run_forwards_parameters() {
        let mut rng = ids::ids(25003);
        let topic = ids::cyrillic(&mut rng, 6);
        let query = ids::greek(&mut rng, 7);
        let processor = ids::cyrillic(&mut rng, 4);
        let language = ids::greek(&mut rng, 4);
        let text = ids::cyrillic(&mut rng, 6);
        let log = Arc::new(Mutex::new(Vec::new()));
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let app = App::with_config(root, conf(log.clone(), &text, serde_json::json!({})));
        std::fs::create_dir_all(root.join("output")).unwrap();
        app.run(&topic, &query, &processor, &language, "parallel");
        let repo = repository::repo(&root.join("output"));
        let list = repo.load();
        let pick = list.first().unwrap();
        assert_eq!(topic, pick.topic(), "Run did not forward topic");
    }

    #[test]
    fn the_application_run_executes_all_providers() {
        let mut rng = ids::ids(25004);
        let topic = ids::cyrillic(&mut rng, 6);
        let query = ids::greek(&mut rng, 7);
        let processor = ids::cyrillic(&mut rng, 4);
        let language = ids::greek(&mut rng, 4);
        let text = ids::cyrillic(&mut rng, 6);
        let log = Arc::new(Mutex::new(Vec::new()));
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("output")).unwrap();
        let app = App::with_config(root, conf(log.clone(), &text, serde_json::json!({})));
        app.run(&topic, &query, &processor, &language, "all");
        let tracked = log.lock().unwrap();
        let total = tracked.len();
        let uniques: std::collections::HashSet<&String> = tracked.iter().collect();
        let result = total == 2 && uniques.len() == 2;
        assert!(result, "Run did not execute two providers for all");
    }

    #[test]
    #[should_panic(expected = "lite is not supported")]
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
        app.run(&topic, &query, "lite", &language, "valyu");
    }

    #[test]
    fn the_application_skips_cover_when_key_missing() {
        let mut rng = ids::ids(25005);
        let topic = ids::cyrillic(&mut rng, 6);
        let query = ids::greek(&mut rng, 7);
        let processor = ids::armenian(&mut rng, 5);
        let language = ids::hiragana(&mut rng, 4);
        let provider = "parallel";
        let run = ids::arabic(&mut rng, 8);
        let text = ids::cyrillic(&mut rng, 12);
        let stamp = task::format(&chrono::Local::now().naive_local());
        let ident = ids::uuid(&mut rng);
        let entry = serde_json::json!({
            "run_id": run,
            "query": query,
            "processor": processor,
            "language": language,
            "provider": provider,
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
        app.research(token, &query, &processor, &language, provider);
        let org = organizer::organizer(&out);
        let name = org.name(sess.created(), sess.topic(), sess.id());
        let cover = org.cover(&name, provider);
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
        let processor = ids::armenian(&mut rng, 5);
        let language = ids::hiragana(&mut rng, 4);
        let provider = ids::cyrillic(&mut rng, 5);
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
            "processor": processor,
            "language": language,
            "provider": provider,
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
        app.research(token, &query, &processor, &language, &provider);
        let org = organizer::organizer(&out);
        let name = org.name(sess.created(), sess.topic(), sess.id());
        let tag = organizer::slug(&provider);
        let tag = if tag.is_empty() {
            "provider".to_string()
        } else {
            tag
        };
        let folder = org.folder(&name, &provider);
        let path = folder.join(format!("response-{}.json", tag));
        let content = std::fs::read_to_string(&path).unwrap();
        let data: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(raw, data, "Raw response did not match stored response");
    }

    #[test]
    fn the_application_continues_after_cover_failure() {
        let mut rng = ids::ids(25008);
        let topic = ids::cyrillic(&mut rng, 6);
        let query = ids::greek(&mut rng, 7);
        let processor = ids::armenian(&mut rng, 5);
        let language = ids::hiragana(&mut rng, 4);
        let provider = ids::cyrillic(&mut rng, 5);
        let run = ids::arabic(&mut rng, 8);
        let text = ids::cyrillic(&mut rng, 12);
        let stamp = task::format(&chrono::Local::now().naive_local());
        let ident = ids::uuid(&mut rng);
        let entry = serde_json::json!({
            "run_id": run,
            "query": query,
            "processor": processor,
            "language": language,
            "provider": provider,
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
        app.research(token, &query, &processor, &language, &provider);
        let org = organizer::organizer(&out);
        let name = org.name(sess.created(), sess.topic(), sess.id());
        let path = org.report(&name, &provider);
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
        let processor = "pro";
        let language = ids::cyrillic(&mut rng, 4);
        let provider = "parallel";
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
        app.research(token, &query, processor, &language, provider);
        let org = organizer::organizer(&out);
        let name = org.name(sess.created(), sess.topic(), sess.id());
        let folder = org.folder(&name, provider);
        let path = folder.join("session.edn");
        let content = std::fs::read_to_string(&path).unwrap();
        let data: serde_json::Value = serde_json::from_str(&content).unwrap();
        let item = &data["tasks"][0]["brief"];
        let seen = item.get("topic").is_some()
            && item.get("items").is_some()
            && item.get("text").is_none();
        assert!(seen, "Brief was not stored in session");
    }

    #[test]
    fn the_application_preserves_brief_structure() {
        let mut rng = ids::ids(25012);
        let topic = ids::cyrillic(&mut rng, 6);
        let text_val = ids::greek(&mut rng, 6);
        let leaf = ids::hiragana(&mut rng, 6);
        let node = ids::armenian(&mut rng, 6);
        let processor = "pro";
        let language = ids::cyrillic(&mut rng, 4);
        let provider = "parallel";
        let run = ids::arabic(&mut rng, 8);
        let output = ids::cyrillic(&mut rng, 6);
        let stamp = task::format(&chrono::Local::now().naive_local());
        let ident = ids::uuid(&mut rng);
        let brief = serde_json::json!({
            "topic": topic,
            "items": [{
                "text": text_val,
                "items": [
                    {"text": leaf, "items": []},
                    {"text": node, "items": []},
                ]
            }]
        });
        let entry = serde_json::json!({
            "run_id": run,
            "brief": brief,
            "processor": processor,
            "language": language,
            "provider": provider,
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
        app.research(token, &query, processor, &language, provider);
        let org = organizer::organizer(&out);
        let name = org.name(sess.created(), sess.topic(), sess.id());
        let folder = org.folder(&name, provider);
        let path = folder.join("session.edn");
        let content = std::fs::read_to_string(&path).unwrap();
        let data: serde_json::Value = serde_json::from_str(&content).unwrap();
        let item = &data["tasks"][0]["brief"];
        let first = item
            .get("items")
            .and_then(|v| v.as_array())
            .and_then(|a| a.first());
        let nested = first
            .and_then(|n| n.get("items"))
            .and_then(|v| v.as_array())
            .map(|a| !a.is_empty())
            .unwrap_or(false);
        assert!(nested, "Nested brief items were not preserved");
    }
}
