use std::io::BufRead;

use crate::progress::{self, Progressed};

/// Return current time milliseconds.
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Return environment value by key.
pub fn env(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

/// Remove periods from log text.
pub fn clean(text: &str) -> String {
    progress::make().clean(text)
}

/// Emit progress event to stdout.
pub fn emit(log: &dyn Progressed, data: &serde_json::Value) {
    let kind = data.get("type").and_then(|v| v.as_str()).unwrap_or("");
    if kind == "task_run.state" {
        let state = data
            .pointer("/run/status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let line = format!("[STATUS] {}", state);
        log.emit(&line);
    } else if kind == "task_run.progress_stats" {
        let meter = data
            .get("progress_meter")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let ratio = meter as u64;
        let count = data
            .pointer("/source_stats/num_sources_read")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let line = format!("[PROGRESS] {}% | Sources: {}", ratio, count);
        log.emit(&line);
    } else if kind.starts_with("task_run.progress_msg") {
        let text = data.get("message").and_then(|v| v.as_str()).unwrap_or("");
        let text = if text.len() > 120 {
            format!("{} [cut]", &text[..120])
        } else {
            text.to_string()
        };
        let label = kind.split('.').next_back().unwrap_or("").to_uppercase();
        let line = format!("[{}] {}", label, text);
        log.emit(&line);
    } else if kind == "error" {
        let text = data
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error");
        let line = format!("[ERROR] {}", text);
        log.emit(&line);
    }
}

/// Parse SSE data payload into map.
pub fn parse(text: &str) -> serde_json::Value {
    serde_json::from_str(text).unwrap_or(serde_json::json!({}))
}

/// Stream SSE events from reader.
pub fn sse(reader: &mut dyn BufRead, log: &dyn Progressed) -> bool {
    let mut data: Vec<String> = Vec::new();
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');
                if trimmed.is_empty() {
                    let payload = data.join("\n");
                    if !payload.is_empty() {
                        let body = parse(&payload);
                        if body.is_object() && !body.as_object().unwrap().is_empty() {
                            emit(log, &body);
                        }
                    }
                    data.clear();
                } else if let Some(rest) = trimmed.strip_prefix("event:") {
                    let _ = rest.trim();
                } else if let Some(rest) = trimmed.strip_prefix("data:") {
                    data.push(rest.trim().to_string());
                }
            }
            Err(_) => break,
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use research_domain::ids;

    #[test]
    fn the_support_cleans_periods() {
        let mut rng = ids::ids(16019);
        let text = ids::cyrillic(&mut rng, 6);
        let value = format!("{}.{}.", text, text);
        let result = clean(&value);
        assert!(
            !result.contains('.'),
            "Parallel clean did not remove periods"
        );
    }

    #[test]
    fn the_support_parses_json() {
        let text = r#"{"type":"task_run.state","run":{"status":"running"}}"#;
        let data = parse(text);
        assert_eq!(
            "task_run.state",
            data.get("type").unwrap().as_str().unwrap(),
            "SSE parse did not extract type"
        );
    }

    #[test]
    fn the_support_parses_invalid_json_as_empty() {
        let data = parse("not json");
        assert!(
            data.as_object().unwrap().is_empty(),
            "Invalid JSON did not return empty map"
        );
    }

    #[test]
    fn the_support_streams_empty_input() {
        let log = progress::make();
        let mut cursor = std::io::Cursor::new(b"");
        let result = sse(&mut cursor, &log);
        assert!(result, "Stream did not complete on empty input");
    }
}
