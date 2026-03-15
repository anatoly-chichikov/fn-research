use std::collections::HashMap;

/// HTTP response wrapper.
pub struct HttpResponse {
    fields: HashMap<String, serde_json::Value>,
}

impl HttpResponse {
    /// Create response from fields.
    pub fn new(fields: HashMap<String, serde_json::Value>) -> Self {
        Self { fields }
    }

    /// Return field value.
    pub fn field(&self, key: &str) -> Option<&serde_json::Value> {
        self.fields.get(key)
    }

    /// Return status code.
    pub fn status(&self) -> Option<u64> {
        self.fields.get("status").and_then(|v| v.as_u64())
    }

    /// Return body string.
    pub fn body(&self) -> Option<&str> {
        self.fields.get("body").and_then(|v| v.as_str())
    }
}

/// Object that can perform HTTP requests.
pub trait Requested {
    /// Return HTTP GET response.
    fn get(&self, url: &str, data: &serde_json::Value) -> HttpResponse;
    /// Return HTTP POST response.
    fn post(&self, url: &str, data: &serde_json::Value) -> HttpResponse;
}

/// HTTP client wrapper.
pub struct Http {
    kind: String,
}

impl Http {
    /// Create HTTP client.
    pub fn new(kind: &str) -> Self {
        Self {
            kind: kind.to_string(),
        }
    }

    /// Return client kind.
    pub fn kind(&self) -> &str {
        &self.kind
    }
}

impl Requested for Http {
    fn get(&self, url: &str, data: &serde_json::Value) -> HttpResponse {
        let timeout_ms = data
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(60000);
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_millis(timeout_ms))
            .build()
            .unwrap();
        let mut req = client.get(url);
        if let Some(hdrs) = data.get("headers").and_then(|v| v.as_object()) {
            for (k, v) in hdrs {
                if let Some(val) = v.as_str() {
                    req = req.header(k.as_str(), val);
                }
            }
        }
        if let Some(params) = data.get("query_params").and_then(|v| v.as_object()) {
            let pairs: Vec<(String, String)> = params
                .iter()
                .map(|(k, v)| {
                    let val = match v {
                        serde_json::Value::String(s) => s.clone(),
                        _ => v.to_string(),
                    };
                    (k.clone(), val)
                })
                .collect();
            req = req.query(&pairs);
        }
        match req.send() {
            Ok(resp) => {
                let code = resp.status().as_u16() as u64;
                let body = resp.text().unwrap_or_default();
                let mut fields = HashMap::new();
                fields.insert("status".to_string(), serde_json::json!(code));
                fields.insert("body".to_string(), serde_json::json!(body));
                HttpResponse::new(fields)
            }
            Err(e) => {
                eprintln!("HTTP GET failed: {}", e);
                HttpResponse::new(HashMap::new())
            }
        }
    }

    fn post(&self, url: &str, data: &serde_json::Value) -> HttpResponse {
        let timeout_ms = data
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(60000);
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_millis(timeout_ms))
            .build()
            .unwrap();
        let mut req = client.post(url);
        if let Some(hdrs) = data.get("headers").and_then(|v| v.as_object()) {
            for (k, v) in hdrs {
                if let Some(val) = v.as_str() {
                    req = req.header(k.as_str(), val);
                }
            }
        }
        if let Some(body) = data.get("body").and_then(|v| v.as_str()) {
            req = req.body(body.to_string());
        }
        match req.send() {
            Ok(resp) => {
                let code = resp.status().as_u16() as u64;
                let body = resp.text().unwrap_or_default();
                let mut fields = HashMap::new();
                fields.insert("status".to_string(), serde_json::json!(code));
                fields.insert("body".to_string(), serde_json::json!(body));
                HttpResponse::new(fields)
            }
            Err(e) => {
                eprintln!("HTTP POST failed: {}", e);
                HttpResponse::new(HashMap::new())
            }
        }
    }
}

/// Return default HTTP client.
pub fn make() -> Http {
    Http::new("reqwest")
}

#[cfg(test)]
mod tests {
    use super::*;
    use research_domain::ids;

    struct FakeHttp {
        code: u64,
        text: String,
    }

    impl FakeHttp {
        fn new(code: u64, text: &str) -> Self {
            Self {
                code,
                text: text.to_string(),
            }
        }
    }

    impl Requested for FakeHttp {
        fn get(&self, _url: &str, _data: &serde_json::Value) -> HttpResponse {
            let mut fields = HashMap::new();
            fields.insert("status".to_string(), serde_json::json!(self.code));
            fields.insert("body".to_string(), serde_json::json!(self.text));
            HttpResponse::new(fields)
        }

        fn post(&self, _url: &str, _data: &serde_json::Value) -> HttpResponse {
            let mut fields = HashMap::new();
            fields.insert("status".to_string(), serde_json::json!(self.code));
            fields.insert("body".to_string(), serde_json::json!(self.text));
            HttpResponse::new(fields)
        }
    }

    #[test]
    fn the_http_get_returns_response() {
        let mut rng = ids::ids(18201);
        let host = ids::ascii(&mut rng, 6);
        let path = ids::cyrillic(&mut rng, 4);
        let code = 200 + ids::digit(&mut rng, 50) as u64;
        let body = ids::greek(&mut rng, 6);
        let raw = format!("https://{}.com/{}", host, path);
        let item = FakeHttp::new(code, &body);
        let result = item.get(&raw, &serde_json::json!({"timeout": 1}));
        let value = result.status().unwrap();
        assert_eq!(code, value, "http get did not return response");
    }

    #[test]
    fn the_http_post_returns_response() {
        let mut rng = ids::ids(18203);
        let host = ids::ascii(&mut rng, 6);
        let path = ids::armenian(&mut rng, 4);
        let code = 200 + ids::digit(&mut rng, 50) as u64;
        let body = ids::hebrew(&mut rng, 6);
        let raw = format!("https://{}.org/{}", host, path);
        let item = FakeHttp::new(code, &body);
        let result = item.post(&raw, &serde_json::json!({"timeout": 1}));
        let value = result.status().unwrap();
        assert_eq!(code, value, "http post did not return response");
    }
}
