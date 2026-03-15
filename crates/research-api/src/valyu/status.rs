use crate::http::Requested;
use crate::progress::Progressed;

/// Object that can fetch Valyu status.
pub trait Statused {
    /// Return status payload.
    fn status(&self, id: &str) -> serde_json::Value;
    /// Pause before retry.
    fn pause(&self, span: u64);
}

/// Valyu status client.
pub struct Status {
    base: String,
    key: String,
    net: Box<dyn Requested>,
    log: Box<dyn Progressed>,
}

impl Status {
    /// Create status client from components.
    pub fn new(base: &str, key: &str, net: Box<dyn Requested>, log: Box<dyn Progressed>) -> Self {
        Self {
            base: base.to_string(),
            key: key.to_string(),
            net,
            log,
        }
    }
}

impl Statused for Status {
    fn pause(&self, span: u64) {
        std::thread::sleep(std::time::Duration::from_millis(span));
    }

    fn status(&self, id: &str) -> serde_json::Value {
        let url = format!("{}/deepresearch/tasks/{}/status", self.base, id);
        let payload = serde_json::json!({
            "headers": {
                "Content-Type": "application/json",
                "x-api-key": self.key
            },
            "timeout": 60000
        });
        let limit = 4u32;
        let span = 1000u64;
        for step in 0..limit {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.net.get(&url, &payload)
            }));
            let (code, body) = match result {
                Ok(resp) => {
                    let code = resp.status();
                    let text = resp.body().unwrap_or("").to_string();
                    (code, text)
                }
                Err(_) => (None, String::new()),
            };
            if let Some(c) = code {
                if c < 300 {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&body) {
                        return data;
                    }
                }
            }
            let signal = code.is_none() || code.map(|c| c >= 500 || c == 429).unwrap_or(true);
            let time = std::cmp::min(span * (step as u64 + 1), span * 8);
            let note = format!(
                "Valyu status non200 id={} status={} attempt={}{}",
                id,
                code.map(|c| c.to_string())
                    .unwrap_or_else(|| "none".to_string()),
                step + 1,
                if signal {
                    format!(" wait_ms={}", time)
                } else {
                    String::new()
                }
            );
            let _ = self.log.emit(&self.log.clean(&note));
            if signal {
                if step < limit - 1 {
                    self.pause(time);
                } else {
                    panic!(
                        "Valyu status failed id={} status={} attempts={}",
                        id,
                        code.map(|c| c.to_string())
                            .unwrap_or_else(|| "none".to_string()),
                        limit
                    );
                }
            } else {
                panic!(
                    "Valyu status failed id={} status={}",
                    id,
                    code.map(|c| c.to_string())
                        .unwrap_or_else(|| "none".to_string())
                );
            }
        }
        panic!("Valyu status failed id={}", id)
    }
}

/// Return status client.
pub fn make(base: &str, key: &str, net: Box<dyn Requested>, log: Box<dyn Progressed>) -> Status {
    Status::new(base, key, net, log)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::HttpResponse;
    use crate::progress;
    use research_domain::ids;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct RetryNet {
        fault: u64,
        success: u64,
        body: String,
        count: AtomicU32,
    }

    impl RetryNet {
        fn new(fault: u64, success: u64, body: &str) -> Self {
            Self {
                fault,
                success,
                body: body.to_string(),
                count: AtomicU32::new(0),
            }
        }
    }

    impl Requested for RetryNet {
        fn get(&self, _url: &str, _data: &serde_json::Value) -> HttpResponse {
            let n = self.count.fetch_add(1, Ordering::SeqCst);
            let code = if n == 0 { self.fault } else { self.success };
            let text = if n == 0 { "" } else { &self.body };
            let mut fields = HashMap::new();
            fields.insert("status".to_string(), serde_json::json!(code));
            fields.insert("body".to_string(), serde_json::json!(text));
            HttpResponse::new(fields)
        }

        fn post(&self, _url: &str, _data: &serde_json::Value) -> HttpResponse {
            HttpResponse::new(HashMap::new())
        }
    }

    struct NullNet {
        success: u64,
        body: String,
        count: AtomicU32,
    }

    impl NullNet {
        fn new(success: u64, body: &str) -> Self {
            Self {
                success,
                body: body.to_string(),
                count: AtomicU32::new(0),
            }
        }
    }

    impl Requested for NullNet {
        fn get(&self, _url: &str, _data: &serde_json::Value) -> HttpResponse {
            let n = self.count.fetch_add(1, Ordering::SeqCst);
            if n == 0 {
                let mut fields = HashMap::new();
                fields.insert("status".to_string(), serde_json::Value::Null);
                return HttpResponse::new(fields);
            }
            let mut fields = HashMap::new();
            fields.insert("status".to_string(), serde_json::json!(self.success));
            fields.insert("body".to_string(), serde_json::json!(self.body));
            HttpResponse::new(fields)
        }

        fn post(&self, _url: &str, _data: &serde_json::Value) -> HttpResponse {
            HttpResponse::new(HashMap::new())
        }
    }

    #[test]
    fn the_status_retries_transient_errors() {
        let mut rng = ids::ids(18401);
        let id = ids::cyrillic(&mut rng, 6);
        let _key = ids::greek(&mut rng, 5);
        let _base = ids::latin(&mut rng, 6);
        let fault = 500 + ids::digit(&mut rng, 50) as u64;
        let success = 200 + ids::digit(&mut rng, 50) as u64;
        let state = ids::armenian(&mut rng, 6);
        let body = serde_json::json!({"status": state}).to_string();
        let net = RetryNet::new(fault, success, &body);
        let log = progress::make();
        let item = Status {
            base: String::new(),
            key: String::new(),
            net: Box::new(net),
            log: Box::new(log),
        };
        let wrapped = StatusNoPause { inner: item };
        let data = wrapped.status(&id);
        assert_eq!(
            state,
            data.get("status").unwrap().as_str().unwrap(),
            "status did not recover from transient error"
        );
    }

    #[test]
    fn the_status_retries_missing_status() {
        let mut rng = ids::ids(18403);
        let id = ids::cyrillic(&mut rng, 6);
        let _key = ids::hebrew(&mut rng, 5);
        let _base = ids::latin(&mut rng, 6);
        let success = 200 + ids::digit(&mut rng, 50) as u64;
        let state = ids::hiragana(&mut rng, 6);
        let body = serde_json::json!({"status": state}).to_string();
        let net = NullNet::new(success, &body);
        let log = progress::make();
        let item = Status {
            base: String::new(),
            key: String::new(),
            net: Box::new(net),
            log: Box::new(log),
        };
        let wrapped = StatusNoPause { inner: item };
        let data = wrapped.status(&id);
        assert_eq!(
            state,
            data.get("status").unwrap().as_str().unwrap(),
            "status did not recover from missing status"
        );
    }

    /// Status wrapper that skips pause.
    struct StatusNoPause {
        inner: Status,
    }

    impl StatusNoPause {
        fn status(&self, id: &str) -> serde_json::Value {
            let url = format!("{}/deepresearch/tasks/{}/status", self.inner.base, id);
            let payload = serde_json::json!({
                "headers": {
                    "Content-Type": "application/json",
                    "x-api-key": self.inner.key
                },
                "timeout": 60000
            });
            let limit = 4u32;
            for step in 0..limit {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    self.inner.net.get(&url, &payload)
                }));
                let (code, body) = match result {
                    Ok(resp) => {
                        let code = resp.status();
                        let text = resp.body().unwrap_or("").to_string();
                        (code, text)
                    }
                    Err(_) => (None, String::new()),
                };
                if let Some(c) = code {
                    if c < 300 {
                        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&body) {
                            return data;
                        }
                    }
                }
                let signal = code.is_none() || code.map(|c| c >= 500 || c == 429).unwrap_or(true);
                if signal && step < limit - 1 {
                    continue;
                }
                if !signal || step >= limit - 1 {
                    panic!("Valyu status failed");
                }
            }
            panic!("Valyu status failed")
        }
    }
}
