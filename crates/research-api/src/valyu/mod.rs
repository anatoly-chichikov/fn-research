pub mod status;

use std::collections::HashMap;

use crate::http::Requested;
use crate::link::{self, Linkable};
use crate::progress::{self, Progressed};
use crate::traits::{Grounded, Researchable};
use research_domain::result::CitationSource;

/// Return newest message and updated seen count.
pub fn message(
    value: &serde_json::Value,
    seen: &HashMap<String, usize>,
    token: &str,
) -> (String, HashMap<String, usize>) {
    let items = value
        .get("messages")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let size = *seen.get(token).unwrap_or(&0);
    if items.len() <= size {
        return (String::new(), seen.clone());
    }
    let item = items.last().unwrap();
    let text = if let Some(arr) = item.get("message").and_then(|v| v.as_array()) {
        arr.iter()
            .map(|v| v.as_str().unwrap_or("").to_string())
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        item.get("message")
            .or_else(|| item.get("content"))
            .or_else(|| item.get("text"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    let mut next = seen.clone();
    next.insert(token.to_string(), items.len());
    (text, next)
}

/// Valyu research client.
pub struct Valyu {
    key: String,
    base: String,
    net: Box<dyn Requested>,
    log: Box<dyn Progressed>,
    state: Box<dyn status::Statused>,
}

impl Valyu {
    /// Create client from components.
    pub fn new(
        key: &str,
        base: &str,
        net: Box<dyn Requested>,
        log: Box<dyn Progressed>,
        state: Box<dyn status::Statused>,
    ) -> Self {
        Self {
            key: key.to_string(),
            base: base.to_string(),
            net,
            log,
            state,
        }
    }
}

impl Researchable for Valyu {
    fn start(&self, query: &str, processor: &str) -> String {
        let url = format!("{}/deepresearch/tasks", self.base);
        let body = serde_json::json!({
            "input": query,
            "model": processor,
            "output_formats": ["markdown", "pdf"]
        });
        let payload = serde_json::json!({
            "headers": {
                "Content-Type": "application/json",
                "x-api-key": self.key
            },
            "body": body.to_string(),
            "timeout": 60000
        });
        let result = self.net.post(&url, &payload);
        let code = result.status().unwrap_or(0);
        if code >= 300 {
            panic!("Valyu create failed with status {}", code);
        }
        let text = result.body().unwrap_or("");
        let data: serde_json::Value = serde_json::from_str(text).unwrap_or(serde_json::json!({}));
        data.get("deepresearch_id")
            .or_else(|| data.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    }

    fn stream(&self, id: &str) {
        let timeout_ms = research_domain::TASK_TIMEOUT_HOURS * 3600000;
        let start = std::time::Instant::now();
        loop {
            let data = self.state.status(id);
            let status = data.get("status");
            let value = status
                .and_then(|v| {
                    if v.is_object() {
                        v.get("value").and_then(|v| v.as_str())
                    } else {
                        v.as_str()
                    }
                })
                .unwrap_or("");
            let done = matches!(value, "completed" | "failed" | "cancelled" | "canceled");
            emit_progress(self.log.as_ref(), &data);
            if done {
                return;
            }
            if start.elapsed().as_millis() as u64 > timeout_ms {
                panic!("Valyu task timed out for id={}", id);
            }
            self.state.pause(180000);
        }
    }

    fn finish(&self, id: &str) -> serde_json::Value {
        let data = self.state.status(id);
        let output = data.get("output");
        let text = output
            .and_then(|v| {
                if v.is_object() {
                    v.get("markdown")
                        .or_else(|| v.get("content"))
                        .and_then(|v| v.as_str())
                } else {
                    v.as_str()
                }
            })
            .unwrap_or("");
        let sources = data
            .get("sources")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let base = basis_from(&sources);
        let status = data.get("status");
        let state = status
            .and_then(|v| {
                if v.is_object() {
                    v.get("value").and_then(|v| v.as_str())
                } else {
                    v.as_str()
                }
            })
            .unwrap_or("completed");
        let code = data
            .get("deepresearch_id")
            .or_else(|| data.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or(id);
        serde_json::json!({
            "id": code,
            "status": state,
            "output": text,
            "basis": base,
            "raw": data
        })
    }
}

impl Grounded for Valyu {
    fn basis(&self, sources: &[CitationSource]) -> Vec<serde_json::Value> {
        let policy = link::make();
        sources
            .iter()
            .filter_map(|s| {
                let loc = research_domain::result::Sourced::url(s);
                if loc.is_empty() {
                    return None;
                }
                let head = research_domain::result::Sourced::title(s);
                let head = if head.is_empty() {
                    policy.domain(loc)
                } else {
                    head.to_string()
                };
                let note = research_domain::result::Sourced::excerpt(s);
                Some(serde_json::json!({
                    "citations": [{
                        "title": head,
                        "url": loc,
                        "excerpts": [note]
                    }]
                }))
            })
            .collect()
    }
}

/// Build basis from source values.
fn basis_from(sources: &[serde_json::Value]) -> Vec<serde_json::Value> {
    let policy = link::make();
    sources
        .iter()
        .filter_map(|data| {
            let loc = data.get("url").and_then(|v| v.as_str()).unwrap_or("");
            if loc.is_empty() {
                return None;
            }
            let text = data
                .get("content")
                .or_else(|| data.get("snippet"))
                .or_else(|| data.get("description"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let head = data.get("title").and_then(|v| v.as_str()).unwrap_or("");
            let head = if head.is_empty() {
                policy.domain(loc)
            } else {
                head.to_string()
            };
            Some(serde_json::json!({
                "citations": [{
                    "title": head,
                    "url": loc,
                    "excerpts": [text]
                }]
            }))
        })
        .collect()
}

/// Emit progress info for Valyu.
fn emit_progress(log: &dyn Progressed, data: &serde_json::Value) {
    let status = data
        .get("status")
        .map(|v| v.to_string())
        .unwrap_or_default();
    let current = data
        .pointer("/progress/current_step")
        .and_then(|v| v.as_u64());
    let total = data
        .pointer("/progress/total_steps")
        .and_then(|v| v.as_u64());
    let msg = data.get("message").and_then(|v| v.as_str()).unwrap_or("");
    let mut items: Vec<String> = Vec::new();
    if !status.is_empty() && status != "\"\"" {
        items.push(status.trim_matches('"').to_string());
    }
    if let (Some(c), Some(t)) = (current, total) {
        items.push(format!("{}/{}", c, t));
    }
    if !msg.is_empty() {
        items.push(msg.to_string());
    }
    let line = if items.is_empty() {
        data.to_string()
    } else {
        items.join(" | ")
    };
    log.emit(&format!("[PROGRESS] {}", line));
}

/// Create Valyu client from config map.
pub fn valyu(item: &serde_json::Value) -> Valyu {
    let key = item
        .get("key")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| std::env::var("VALYU_API_KEY").unwrap_or_default());
    let mut base = item
        .get("base")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            std::env::var("VALYU_BASE_URL").unwrap_or_else(|_| "https://api.valyu.ai".to_string())
        });
    if base.contains("api.valyu.ai") && !base.ends_with("/v1") {
        base = format!("{}/v1", base.trim_end_matches('/'));
    }
    let mode = item.get("mode").and_then(|v| v.as_str()).unwrap_or("");
    if key.is_empty() && mode != "basis" {
        panic!("VALYU_API_KEY is required");
    }
    let log = progress::make();
    let net = crate::http::make();
    let unit = status::make(
        &base,
        &key,
        Box::new(crate::http::make()),
        Box::new(progress::make()),
    );
    Valyu::new(&key, &base, Box::new(net), Box::new(log), Box::new(unit))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::{HttpResponse, Requested};
    use crate::response;
    use crate::response::Responded;
    use research_domain::ids;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct FakeNet {
        code: u64,
        text: String,
        captured: std::cell::RefCell<String>,
    }

    impl FakeNet {
        fn new(code: u64, text: &str) -> Self {
            Self {
                code,
                text: text.to_string(),
                captured: std::cell::RefCell::new(String::new()),
            }
        }
    }

    impl Requested for FakeNet {
        fn get(&self, _url: &str, _data: &serde_json::Value) -> HttpResponse {
            let mut fields = HashMap::new();
            fields.insert("status".to_string(), serde_json::json!(self.code));
            fields.insert("body".to_string(), serde_json::json!(self.text));
            HttpResponse::new(fields)
        }

        fn post(&self, _url: &str, data: &serde_json::Value) -> HttpResponse {
            if let Some(body) = data.get("body").and_then(|v| v.as_str()) {
                *self.captured.borrow_mut() = body.to_string();
            }
            let mut fields = HashMap::new();
            fields.insert("status".to_string(), serde_json::json!(self.code));
            fields.insert("body".to_string(), serde_json::json!(self.text));
            HttpResponse::new(fields)
        }
    }

    struct FakeStatus {
        data: serde_json::Value,
        paused: std::cell::RefCell<u64>,
        count: AtomicU32,
    }

    impl FakeStatus {
        fn new(data: serde_json::Value) -> Self {
            Self {
                data,
                paused: std::cell::RefCell::new(0),
                count: AtomicU32::new(0),
            }
        }
    }

    impl status::Statused for FakeStatus {
        fn status(&self, _id: &str) -> serde_json::Value {
            self.count.fetch_add(1, Ordering::SeqCst);
            self.data.clone()
        }

        fn pause(&self, span: u64) {
            *self.paused.borrow_mut() = span;
        }
    }

    struct PollingStatus {
        phase: String,
        paused: std::cell::RefCell<u64>,
        count: AtomicU32,
    }

    impl PollingStatus {
        fn new(phase: &str) -> Self {
            Self {
                phase: phase.to_string(),
                paused: std::cell::RefCell::new(0),
                count: AtomicU32::new(0),
            }
        }
    }

    impl status::Statused for PollingStatus {
        fn status(&self, _id: &str) -> serde_json::Value {
            let n = self.count.fetch_add(1, Ordering::SeqCst);
            if n == 0 {
                serde_json::json!({"status": self.phase})
            } else {
                serde_json::json!({"status": "completed"})
            }
        }

        fn pause(&self, span: u64) {
            *self.paused.borrow_mut() = span;
        }
    }

    #[test]
    fn the_valyu_start_returns_run_identifier() {
        let mut rng = ids::ids(17001);
        let run = format!("dr_{}", ids::digit(&mut rng, 100000));
        let _query = ids::greek(&mut rng, 6);
        let body = serde_json::json!({"deepresearch_id": run}).to_string();
        let net = FakeNet::new(200, &body);
        let state = FakeStatus::new(serde_json::json!({}));
        let client = Valyu::new(
            "key",
            "https://example.com",
            Box::new(net),
            Box::new(progress::make()),
            Box::new(state),
        );
        let query = ids::greek(&mut rng, 6);
        let model = format!("standard-{}", ids::digit(&mut rng, 1000));
        let result = client.start(&query, &model);
        assert_eq!(run, result, "start did not return expected identifier");
    }

    #[test]
    fn the_valyu_start_uses_versioned_endpoint() {
        let base = "https://api.valyu.ai";
        let client = valyu(&serde_json::json!({"key": "key", "base": base}));
        assert!(
            client.base.contains("/v1"),
            "Valyu did not use versioned endpoint"
        );
    }

    #[test]
    fn the_valyu_start_preserves_processor() {
        let mut rng = ids::ids(17004);
        let run = format!("dr_{}", ids::digit(&mut rng, 100000));
        let _query = ids::cyrillic(&mut rng, 6);
        let _key = ids::greek(&mut rng, 5);
        let model = ids::cyrillic(&mut rng, 5);
        let body = serde_json::json!({"deepresearch_id": run}).to_string();
        let net = FakeNet::new(200, &body);
        let state = FakeStatus::new(serde_json::json!({}));
        let client = Valyu::new(
            "key",
            "https://example.com",
            Box::new(net),
            Box::new(progress::make()),
            Box::new(state),
        );
        let result = client.start("query", &model);
        assert!(!result.is_empty(), "Valyu changed processor");
    }

    #[test]
    fn the_valyu_reads_progress_messages() {
        let mut rng = ids::ids(17013);
        let token = ids::cyrillic(&mut rng, 6);
        let seen = HashMap::new();
        let value = serde_json::json!({"messages": [{"message": token}]});
        let (result, _) = message(&value, &seen, "trun_x");
        assert_eq!(token, result, "message was not returned");
    }

    #[test]
    fn the_valyu_formats_list_messages() {
        let mut rng = ids::ids(17015);
        let left = ids::cyrillic(&mut rng, 4);
        let right = ids::armenian(&mut rng, 4);
        let seen = HashMap::new();
        let value = serde_json::json!({"messages": [{"message": [left, right]}]});
        let (result, _) = message(&value, &seen, "trun_x");
        assert_eq!(
            format!("{} {}", left, right),
            result,
            "list message was not joined"
        );
    }

    #[test]
    fn the_valyu_polls_every_three_minutes() {
        let mut rng = ids::ids(17023);
        let id = ids::cyrillic(&mut rng, 6);
        let _key = ids::greek(&mut rng, 5);
        let _base = ids::latin(&mut rng, 6);
        let phase = ids::arabic(&mut rng, 6);
        let state = PollingStatus::new(&phase);
        let net = FakeNet::new(200, "");
        let client = Valyu::new(
            "key",
            "https://example.com",
            Box::new(net),
            Box::new(progress::make()),
            Box::new(state),
        );
        client.stream(&id);
    }

    #[test]
    fn the_valyu_uses_raw_status_payload() {
        let mut rng = ids::ids(17017);
        let ident = format!("dr_{}", ids::digit(&mut rng, 100000));
        let output = ids::cyrillic(&mut rng, 6);
        let title = ids::greek(&mut rng, 5);
        let link = format!("http://example.com/{}", ids::digit(&mut rng, 1000));
        let payload = serde_json::json!({
            "success": true,
            "status": "completed",
            "output": {"markdown": output},
            "sources": [{
                "title": title,
                "url": link,
                "content": output
            }],
            "deepresearch_id": ident
        });
        let state = FakeStatus::new(payload);
        let net = FakeNet::new(200, "");
        let client = Valyu::new(
            "key",
            "https://example.com",
            Box::new(net),
            Box::new(progress::make()),
            Box::new(state),
        );
        let data = client.finish(&ident);
        let item = response::response(&data);
        assert_eq!(output, item.text(), "Markdown did not match raw payload");
    }
}
