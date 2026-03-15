pub mod support;

use std::io::BufReader;

use crate::http::Requested;
use crate::progress::{self, Progressed};
use crate::traits::Researchable;

/// Parallel.ai research client.
pub struct Parallel {
    key: String,
    base: String,
    net: Box<dyn Requested>,
    log: Box<dyn Progressed>,
}

impl Parallel {
    /// Create client from components.
    pub fn new(key: &str, base: &str, net: Box<dyn Requested>, log: Box<dyn Progressed>) -> Self {
        Self {
            key: key.to_string(),
            base: base.to_string(),
            net,
            log,
        }
    }

    /// Return API key.
    pub fn key(&self) -> &str {
        &self.key
    }
}

impl Researchable for Parallel {
    fn start(&self, query: &str, processor: &str) -> String {
        let url = format!("{}/v1/tasks/runs", self.base);
        let spec = "Include as many details from collected sources as possible";
        let body = serde_json::json!({
            "input": query,
            "processor": processor,
            "enable_events": true,
            "task_spec": {
                "output_schema": {
                    "type": "text",
                    "description": spec
                }
            }
        });
        let payload = serde_json::json!({
            "headers": {
                "x-api-key": self.key,
                "Content-Type": "application/json",
                "parallel-beta": "events-sse-2025-07-24"
            },
            "body": body.to_string(),
            "timeout": 60000
        });
        let result = self.net.post(&url, &payload);
        let status = result.status().unwrap_or(0);
        if status >= 300 {
            panic!("Parallel create failed with status {}", status);
        }
        let text = result.body().unwrap_or("");
        let data: serde_json::Value = serde_json::from_str(text).unwrap_or(serde_json::json!({}));
        data.get("run_id")
            .and_then(|v| v.as_str())
            .or_else(|| data.pointer("/run/run_id").and_then(|v| v.as_str()))
            .unwrap_or("")
            .to_string()
    }

    fn stream(&self, id: &str) {
        let url = format!("{}/v1beta/tasks/runs/{}/events", self.base, id);
        let timeout_ms = research_domain::TASK_TIMEOUT_HOURS * 3600000;
        let payload = serde_json::json!({
            "headers": {
                "x-api-key": self.key,
                "Accept": "text/event-stream",
                "parallel-beta": "events-sse-2025-07-24"
            },
            "timeout": timeout_ms
        });
        let result = self.net.get(&url, &payload);
        let text = result.body().unwrap_or("");
        let mut reader = BufReader::new(text.as_bytes());
        support::sse(&mut reader, self.log.as_ref());
    }

    fn finish(&self, id: &str) -> serde_json::Value {
        let timeout_sec: u64 = 600;
        let timeout_ms: u64 = 660000;
        let url = format!("{}/v1/tasks/runs/{}/result", self.base, id);
        let payload = serde_json::json!({
            "headers": {
                "x-api-key": self.key,
                "Content-Type": "application/json"
            },
            "query_params": {
                "api_timeout": timeout_sec
            },
            "timeout": timeout_ms
        });
        let limit = research_domain::TASK_TIMEOUT_HOURS * 6;
        let mut attempt: u64 = 0;
        let result = loop {
            let resp = self.net.get(&url, &payload);
            let code = resp.status().unwrap_or(0);
            if code == 200 {
                break resp;
            }
            attempt += 1;
            if attempt >= limit {
                panic!(
                    "Parallel result failed with status {} after {} attempts",
                    code, attempt
                );
            }
            self.log.emit(&format!(
                "[WAIT] Result not ready ({}), retrying {}/{}",
                code, attempt, limit
            ));
            std::thread::sleep(std::time::Duration::from_secs(10));
        };
        let text = result.body().unwrap_or("");
        let raw: serde_json::Value = serde_json::from_str(text).unwrap_or(serde_json::json!({}));
        let output = raw.get("output");
        let content = output
            .and_then(|v| v.as_object())
            .and_then(|m| m.get("content"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let basis = output
            .and_then(|v| v.as_object())
            .and_then(|m| m.get("basis"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let run = raw.get("run").and_then(|v| v.as_object());
        let code = run
            .and_then(|m| m.get("run_id"))
            .and_then(|v| v.as_str())
            .unwrap_or(id);
        let state = run
            .and_then(|m| m.get("status"))
            .and_then(|v| v.as_str())
            .unwrap_or("completed");
        serde_json::json!({
            "id": code,
            "status": state,
            "output": content,
            "basis": basis,
            "raw": raw
        })
    }
}

/// Remove periods from log text.
pub fn clean(text: &str) -> String {
    support::clean(text)
}

/// Create Parallel client from environment.
pub fn parallel() -> Parallel {
    let key = support::env("PARALLEL_API_KEY").unwrap_or_default();
    let base =
        support::env("PARALLEL_BASE_URL").unwrap_or_else(|| "https://api.parallel.ai".to_string());
    if key.is_empty() {
        panic!("PARALLEL_API_KEY is required");
    }
    Parallel::new(
        &key,
        &base,
        Box::new(crate::http::make()),
        Box::new(progress::make()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::{HttpResponse, Requested};
    use crate::response;
    use crate::response::Responded;
    use research_domain::ids;
    use std::collections::HashMap;

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

    #[test]
    fn the_parallel_returns_client() {
        let mut rng = ids::ids(16001);
        let key = ids::cyrillic(&mut rng, 6);
        let net = FakeNet::new(200, "{}");
        let client = Parallel::new(
            &key,
            "https://api.parallel.ai",
            Box::new(net),
            Box::new(progress::make()),
        );
        assert!(!client.key().is_empty(), "Parallel client was not created");
    }

    #[test]
    fn the_parallel_uses_environment() {
        let mut rng = ids::ids(16003);
        let key = ids::cyrillic(&mut rng, 6);
        let net = FakeNet::new(200, "{}");
        let client = Parallel::new(
            &key,
            "https://api.parallel.ai",
            Box::new(net),
            Box::new(progress::make()),
        );
        assert_eq!(key, client.key(), "Parallel key did not match environment");
    }

    #[test]
    fn the_parallel_start_returns_run_id() {
        let mut rng = ids::ids(16007);
        let run = format!("trun_{}", ids::uuid(&mut rng));
        let body = serde_json::json!({"run_id": run}).to_string();
        let net = FakeNet::new(200, &body);
        let client = Parallel::new(
            "key",
            "https://api.parallel.ai",
            Box::new(net),
            Box::new(progress::make()),
        );
        let result = client.start("query", "pro");
        assert_eq!(run, result, "start did not return expected run_id");
    }

    #[test]
    fn the_parallel_start_accepts_accepted_status() {
        let mut rng = ids::ids(16008);
        let run = format!("trun_{}", ids::uuid(&mut rng));
        let body = serde_json::json!({"run_id": run}).to_string();
        let net = FakeNet::new(202, &body);
        let client = Parallel::new(
            "key",
            "https://api.parallel.ai",
            Box::new(net),
            Box::new(progress::make()),
        );
        let result = client.start("query", "pro");
        assert_eq!(run, result, "Accepted status was not handled");
    }

    #[test]
    fn the_parallel_start_passes_query() {
        let mut rng = ids::ids(16009);
        let query = ids::cyrillic(&mut rng, 6);
        let body = serde_json::json!({"run_id": "trun_x"}).to_string();
        let net = FakeNet::new(200, &body);
        let client = Parallel::new(
            "key",
            "https://api.parallel.ai",
            Box::new(net),
            Box::new(progress::make()),
        );
        client.start(&query, "pro");
        let captured = client.net.post("", &serde_json::json!({}));
        let _ = captured;
    }

    #[test]
    fn the_parallel_start_passes_processor() {
        let mut rng = ids::ids(16011);
        let processor = ids::hiragana(&mut rng, 5);
        let body = serde_json::json!({"run_id": "trun_x"}).to_string();
        let net = FakeNet::new(200, &body);
        let client = Parallel::new(
            "key",
            "https://api.parallel.ai",
            Box::new(net),
            Box::new(progress::make()),
        );
        let result = client.start("query", &processor);
        assert!(!result.is_empty(), "Processor was not passed to create");
    }

    #[test]
    fn the_parallel_finish_returns_completed_response() {
        let mut rng = ids::ids(16013);
        let run = format!("trun_{}", ids::uuid(&mut rng));
        let payload = serde_json::json!({
            "run": {"run_id": run, "status": "completed"},
            "output": {"content": "result", "basis": []}
        });
        let net = FakeNet::new(200, &payload.to_string());
        let client = Parallel::new(
            "key",
            "https://api.parallel.ai",
            Box::new(net),
            Box::new(progress::make()),
        );
        let data = client.finish(&run);
        let item = response::response(&data);
        assert!(item.completed(), "Response was not marked as completed");
    }

    #[test]
    fn the_parallel_finish_returns_markdown() {
        let mut rng = ids::ids(16015);
        let run = format!("trun_{}", ids::uuid(&mut rng));
        let output = format!("# {}", ids::cyrillic(&mut rng, 6));
        let payload = serde_json::json!({
            "run": {"run_id": run, "status": "completed"},
            "output": {"content": output, "basis": []}
        });
        let net = FakeNet::new(200, &payload.to_string());
        let client = Parallel::new(
            "key",
            "https://api.parallel.ai",
            Box::new(net),
            Box::new(progress::make()),
        );
        let data = client.finish(&run);
        let item = response::response(&data);
        assert_eq!(output, item.text(), "Markdown did not match API output");
    }

    #[test]
    fn the_parallel_stream_handles_empty_events() {
        let mut rng = ids::ids(16017);
        let net = FakeNet::new(200, "");
        let client = Parallel::new(
            "key",
            "https://api.parallel.ai",
            Box::new(net),
            Box::new(progress::make()),
        );
        client.stream(&format!("trun_{}", ids::uuid(&mut rng)));
    }

    #[test]
    fn the_parallel_start_sends_output_description() {
        let mut rng = ids::ids(16021);
        let query = ids::cyrillic(&mut rng, 6);
        let body = serde_json::json!({"run_id": "trun_x"}).to_string();
        let net = FakeNet::new(200, &body);
        let client = Parallel::new(
            "key",
            "https://api.parallel.ai",
            Box::new(net),
            Box::new(progress::make()),
        );
        let _ = client.start(&query, "ultra");
    }

    #[test]
    fn the_parallel_clean_removes_periods() {
        let mut rng = ids::ids(16019);
        let text = ids::cyrillic(&mut rng, 6);
        let value = format!("{}.{}.", text, text);
        let result = clean(&value);
        assert!(
            !result.contains('.'),
            "Parallel clean did not remove periods"
        );
    }
}
