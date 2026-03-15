pub mod bridge;
pub mod brief;
pub mod cache;
pub mod citations;

use crate::traits::Researchable;
use bridge::Bound;
use cache::Stored;

/// Return fixed window days.
pub fn window() -> u64 {
    365
}

/// XAI research provider.
pub struct Xai {
    data: serde_json::Value,
    store: cache::Cache,
    unit: Box<dyn Bound>,
}

impl Xai {
    /// Create XAI provider from components.
    pub fn new(data: serde_json::Value, store: cache::Cache, unit: Box<dyn Bound>) -> Self {
        Self { data, store, unit }
    }
}

impl Researchable for Xai {
    fn start(&self, query: &str, _processor: &str) -> String {
        let code = uuid::Uuid::new_v4().to_string();
        let days = window();
        let mut pack = self.data.clone();
        if let Some(obj) = pack.as_object_mut() {
            obj.insert("window".to_string(), serde_json::json!(days));
        }
        let payload = serde_json::json!({
            "query": query,
            "config": pack
        });
        self.store.save(&code, &payload);
        code
    }

    fn stream(&self, _id: &str) {}

    fn finish(&self, id: &str) -> serde_json::Value {
        let data = self.store.load(id);
        let text = data.get("query").and_then(|v| v.as_str()).unwrap_or("");
        let pack = data.get("config").cloned().unwrap_or(serde_json::json!({}));
        let raw = self.unit.run(text, &pack);
        let out = raw.get("output").cloned().unwrap_or(serde_json::json!({}));
        let body = out
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let basis = out
            .get("basis")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let state = raw
            .get("run")
            .and_then(|r| r.get("status"))
            .and_then(|v| v.as_str())
            .unwrap_or("completed")
            .to_string();
        let code = raw
            .get("run")
            .and_then(|r| r.get("run_id"))
            .and_then(|v| v.as_str())
            .unwrap_or(id)
            .to_string();
        serde_json::json!({
            "id": code,
            "status": state,
            "output": body,
            "basis": basis,
            "raw": raw
        })
    }
}

/// Create XAI provider from config.
pub fn xai(root: &std::path::Path, unit: Box<dyn Bound>, opts: &serde_json::Value) -> Xai {
    let model = opts
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("grok-4-1-fast");
    let mode = opts
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("social_multi");
    let turns = opts.get("turns").and_then(|v| v.as_u64()).unwrap_or(2);
    let tokens = opts.get("tokens").and_then(|v| v.as_u64()).unwrap_or(3500);
    let flag = opts.get("follow").and_then(|v| v.as_bool()).unwrap_or(true);
    let section = opts
        .get("section")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let domains = opts
        .get("domains")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| {
            vec![
                "reddit.com".to_string(),
                "youtube.com".to_string(),
                "tiktok.com".to_string(),
                "instagram.com".to_string(),
                "linkedin.com".to_string(),
            ]
        });
    let data = serde_json::json!({
        "model": model,
        "mode": mode,
        "turns": turns,
        "window": window(),
        "tokens": tokens,
        "follow": flag,
        "section": section,
        "domains": domains,
    });
    let store = cache::make(root);
    Xai::new(data, store, unit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::response;
    use research_domain::ids;

    struct FakeUnit {
        result: serde_json::Value,
    }

    impl Bound for FakeUnit {
        fn run(&self, _text: &str, _pack: &serde_json::Value) -> serde_json::Value {
            self.result.clone()
        }
    }

    #[test]
    fn the_xai_start_stores_query() {
        let mut rng = ids::ids(17001);
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let query = ids::cyrillic(&mut rng, 6);
        let model = ids::latin(&mut rng, 6);
        let mode = ids::greek(&mut rng, 5);
        let unit = FakeUnit {
            result: serde_json::json!({}),
        };
        let opts = serde_json::json!({
            "model": model,
            "mode": mode,
            "turns": 2,
            "window": 3,
            "tokens": 4,
            "follow": false,
            "section": false,
            "domains": [ids::armenian(&mut rng, 4)]
        });
        let item = xai(root, Box::new(unit), &opts);
        item.start(&query, "365");
        let cache = root.join("tmp_cache").join("xai");
        let entries: Vec<_> = std::fs::read_dir(&cache)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        let path = &entries[0].path();
        let text = std::fs::read_to_string(path).unwrap();
        let data: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(
            query,
            data.get("query").unwrap().as_str().unwrap(),
            "Query was not stored"
        );
    }

    #[test]
    fn the_xai_start_uses_window() {
        let mut rng = ids::ids(17002);
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let query = ids::cyrillic(&mut rng, 5);
        let unit = FakeUnit {
            result: serde_json::json!({}),
        };
        let opts = serde_json::json!({
            "model": ids::latin(&mut rng, 6),
            "mode": ids::greek(&mut rng, 4),
            "turns": 2,
            "window": 365,
            "tokens": 4,
            "follow": false,
            "section": false,
            "domains": [ids::armenian(&mut rng, 4)]
        });
        let item = xai(root, Box::new(unit), &opts);
        item.start(&query, "90");
        let cache = root.join("tmp_cache").join("xai");
        let entries: Vec<_> = std::fs::read_dir(&cache)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        let path = &entries[0].path();
        let text = std::fs::read_to_string(path).unwrap();
        let data: serde_json::Value = serde_json::from_str(&text).unwrap();
        let pack = data.get("config").unwrap();
        let days = pack.get("window").unwrap().as_u64().unwrap();
        assert_eq!(365, days, "Window did not use fixed year");
    }

    #[test]
    fn the_xai_start_ignores_processor() {
        let mut rng = ids::ids(17004);
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let query = ids::cyrillic(&mut rng, 5);
        let bad = ids::hiragana(&mut rng, 4);
        let unit = FakeUnit {
            result: serde_json::json!({}),
        };
        let opts = serde_json::json!({
            "model": ids::latin(&mut rng, 6),
            "mode": ids::greek(&mut rng, 4),
            "turns": 2,
            "window": 365,
            "tokens": 4,
            "follow": false,
            "section": false,
            "domains": [ids::armenian(&mut rng, 4)]
        });
        let item = xai(root, Box::new(unit), &opts);
        let raised = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            item.start(&query, &bad);
        }));
        assert!(raised.is_ok(), "Invalid processor was not ignored");
    }

    #[test]
    fn the_xai_finish_returns_markdown() {
        let mut rng = ids::ids(17003);
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let query = ids::cyrillic(&mut rng, 7);
        let model = ids::latin(&mut rng, 7);
        let text = ids::hebrew(&mut rng, 8);
        let code = ids::ascii(&mut rng, 10);
        let unit = FakeUnit {
            result: serde_json::json!({
                "run": {
                    "run_id": code,
                    "status": "completed"
                },
                "output": {
                    "content": text,
                    "basis": []
                }
            }),
        };
        let opts = serde_json::json!({
            "model": model,
            "mode": ids::greek(&mut rng, 4),
            "turns": 5,
            "window": 6,
            "tokens": 7,
            "follow": true,
            "section": true,
            "domains": [ids::arabic(&mut rng, 4)]
        });
        let item = xai(root, Box::new(unit), &opts);
        let run = item.start(&query, "365");
        let result = item.finish(&run);
        let resp = response::response(&result);
        use crate::response::Responded;
        assert_eq!(text, resp.text(), "Markdown did not match output");
    }
}
