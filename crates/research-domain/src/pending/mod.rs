use std::collections::HashMap;

use crate::brief::{self, Brief, Node};

/// Object with pending run details.
pub trait Pendinged {
    /// Return run identifier.
    fn id(&self) -> &str;
    /// Return brief details.
    fn brief(&self) -> &Brief;
    /// Return research query.
    fn query(&self) -> String;
    /// Return processor name.
    fn processor(&self) -> &str;
    /// Return research language.
    fn language(&self) -> &str;
    /// Return provider name.
    fn provider(&self) -> &str;
    /// Return map representation.
    fn data(&self) -> HashMap<String, serde_json::Value>;
}

/// Pending run record.
#[derive(Debug, Clone)]
pub struct PendingRun {
    code: String,
    content: Brief,
    proc: String,
    lang: String,
    prov: String,
}

impl Pendinged for PendingRun {
    fn id(&self) -> &str {
        &self.code
    }

    fn brief(&self) -> &Brief {
        &self.content
    }

    fn query(&self) -> String {
        brief::render(&self.content, &self.lang)
    }

    fn processor(&self) -> &str {
        &self.proc
    }

    fn language(&self) -> &str {
        &self.lang
    }

    fn provider(&self) -> &str {
        &self.prov
    }

    fn data(&self) -> HashMap<String, serde_json::Value> {
        let mut map = HashMap::new();
        map.insert(
            "run_id".to_string(),
            serde_json::Value::String(self.code.clone()),
        );
        map.insert(
            "processor".to_string(),
            serde_json::Value::String(self.proc.clone()),
        );
        map.insert(
            "language".to_string(),
            serde_json::Value::String(self.lang.clone()),
        );
        map.insert(
            "provider".to_string(),
            serde_json::Value::String(self.prov.clone()),
        );
        map.insert("brief".to_string(), brief::data(&self.content));
        map
    }
}

/// Create pending run from JSON value.
pub fn pending(item: &serde_json::Value) -> PendingRun {
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
    let run_id = item.get("run_id").and_then(|v| v.as_str()).unwrap_or("");
    let processor = item.get("processor").and_then(|v| v.as_str()).unwrap_or("");
    let language = item.get("language").and_then(|v| v.as_str()).unwrap_or("");
    let provider = item
        .get("provider")
        .and_then(|v| v.as_str())
        .unwrap_or("parallel");
    PendingRun {
        code: run_id.to_string(),
        content,
        proc: processor.to_string(),
        lang: language.to_string(),
        prov: provider.to_string(),
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
