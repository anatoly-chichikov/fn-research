use chrono::NaiveDateTime;
use std::collections::HashMap;

use crate::brief::{self, Brief, Node};
use crate::result::{self, Report};

/// Object representing a research task.
pub trait Tasked {
    /// Return task identifier.
    fn id(&self) -> &str;
    /// Return brief details.
    fn brief(&self) -> &Brief;
    /// Return research query.
    fn query(&self) -> String;
    /// Return task status.
    fn status(&self) -> &str;
    /// Return task result object.
    fn report(&self) -> &Report;
    /// Return task language.
    fn language(&self) -> &str;
    /// Return task provider.
    fn provider(&self) -> &str;
    /// Return creation time.
    fn created(&self) -> &NaiveDateTime;
    /// Return completion time.
    fn completed(&self) -> Option<&NaiveDateTime>;
    /// Return task marked as completed.
    fn finish(&self, value: Report) -> ResearchRun;
    /// Return map representation.
    fn data(&self) -> HashMap<String, serde_json::Value>;
}

/// Parse ISO datetime string.
pub fn parse(text: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S%.f"))
        .expect("Invalid datetime format")
}

/// Format NaiveDateTime into ISO string.
pub fn format(time: &NaiveDateTime) -> String {
    time.format("%Y-%m-%dT%H:%M:%S").to_string()
}

/// Research run record.
#[derive(Debug, Clone)]
pub struct ResearchRun {
    code: String,
    content: Brief,
    status: String,
    lang: String,
    service: String,
    processor: String,
    stamp: NaiveDateTime,
    done: Option<NaiveDateTime>,
    value: Report,
}

impl Tasked for ResearchRun {
    fn id(&self) -> &str {
        &self.code
    }

    fn brief(&self) -> &Brief {
        &self.content
    }

    fn query(&self) -> String {
        brief::render(&self.content, &self.lang)
    }

    fn status(&self) -> &str {
        &self.status
    }

    fn report(&self) -> &Report {
        &self.value
    }

    fn language(&self) -> &str {
        &self.lang
    }

    fn provider(&self) -> &str {
        &self.service
    }

    fn created(&self) -> &NaiveDateTime {
        &self.stamp
    }

    fn completed(&self) -> Option<&NaiveDateTime> {
        self.done.as_ref()
    }

    fn finish(&self, value: Report) -> ResearchRun {
        ResearchRun {
            code: self.code.clone(),
            content: self.content.clone(),
            status: "completed".to_string(),
            lang: self.lang.clone(),
            service: self.service.clone(),
            processor: self.processor.clone(),
            stamp: self.stamp,
            done: Some(chrono::Local::now().naive_local()),
            value,
        }
    }

    fn data(&self) -> HashMap<String, serde_json::Value> {
        let mut map = HashMap::new();
        map.insert(
            "id".to_string(),
            serde_json::Value::String(self.code.clone()),
        );
        map.insert(
            "status".to_string(),
            serde_json::Value::String(self.status.clone()),
        );
        map.insert(
            "language".to_string(),
            serde_json::Value::String(self.lang.clone()),
        );
        map.insert(
            "service".to_string(),
            serde_json::Value::String(self.service.clone()),
        );
        if !self.processor.is_empty() {
            map.insert(
                "processor".to_string(),
                serde_json::Value::String(self.processor.clone()),
            );
        }
        map.insert("brief".to_string(), brief::data(&self.content));
        map.insert(
            "created".to_string(),
            serde_json::Value::String(format(&self.stamp)),
        );
        if let Some(ref done) = self.done {
            map.insert(
                "completed".to_string(),
                serde_json::Value::String(format(done)),
            );
        }
        map
    }
}

/// Create task from JSON value.
pub fn task(item: &serde_json::Value) -> ResearchRun {
    let text = item
        .get("language")
        .and_then(|v| v.as_str())
        .unwrap_or("\u{0440}\u{0443}\u{0441}\u{0441}\u{043a}\u{0438}\u{0439}");
    let name = item
        .get("service")
        .and_then(|v| v.as_str())
        .unwrap_or("parallel.ai");
    let name = normalize_provider(name);
    let time = parse(
        item.get("created")
            .and_then(|v| v.as_str())
            .expect("Task missing created field"),
    );
    let done = item.get("completed").and_then(|v| v.as_str()).map(parse);
    let entry = item.get("brief");
    let query_text = entry
        .and_then(|e| e.get("text"))
        .and_then(|v| v.as_str())
        .or_else(|| item.get("query").and_then(|v| v.as_str()))
        .unwrap_or("");
    let explicit_topic = entry
        .and_then(|e| e.get("topic"))
        .and_then(|v| v.as_str())
        .or_else(|| item.get("topic").and_then(|v| v.as_str()));
    let explicit_items = entry
        .and_then(|e| e.get("items"))
        .and_then(|v| v.as_array())
        .filter(|a| !a.is_empty())
        .map(|arr| arr.iter().map(json_to_node).collect::<Vec<Node>>());
    let content = brief::parse(query_text, explicit_topic, explicit_items.as_deref());
    let raw = item.get("result");
    let value = result::result(raw);
    let code = item
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let processor = item.get("processor").and_then(|v| v.as_str()).unwrap_or("");
    ResearchRun {
        code,
        content,
        status: item
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        lang: text.to_string(),
        service: name.to_string(),
        processor: processor.to_string(),
        stamp: time,
        done,
        value,
    }
}

/// Normalize provider name (fix xai.ai -> x.ai).
fn normalize_provider(name: &str) -> String {
    let parts: Vec<&str> = name.split('.').collect();
    if name.ends_with(".ai") && parts.first() == Some(&"xai") && name != "x.ai" {
        "x.ai".to_string()
    } else {
        name.to_string()
    }
}

/// Convert JSON value to Node.
fn json_to_node(value: &serde_json::Value) -> Node {
    let text = value
        .get("text")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let items = value
        .get("items")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().map(json_to_node).collect())
        .unwrap_or_default();
    Node { text, items }
}

#[cfg(test)]
mod tests;
