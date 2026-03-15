use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::brief::{self, Brief, Question};
use crate::processor::{self, Processor};
use crate::provider::Provider;

/// Object with pending run details.
pub trait Pendinged {
    /// Return run identifier.
    fn id(&self) -> &str;
    /// Return brief details.
    fn brief(&self) -> &Brief;
    /// Return research query.
    fn query(&self) -> String;
    /// Return processor.
    fn processor(&self) -> &Processor;
    /// Return research language.
    fn language(&self) -> &str;
    /// Return provider.
    fn provider(&self) -> &Provider;
    /// Return map representation.
    fn data(&self) -> HashMap<String, serde_json::Value>;
}

/// Pending run record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PendingRun {
    code: String,
    content: Brief,
    proc: Processor,
    prov: Provider,
}

impl Pendinged for PendingRun {
    fn id(&self) -> &str {
        &self.code
    }

    fn brief(&self) -> &Brief {
        &self.content
    }

    fn query(&self) -> String {
        brief::render(&self.content)
    }

    fn processor(&self) -> &Processor {
        &self.proc
    }

    fn language(&self) -> &str {
        &self.content.language
    }

    fn provider(&self) -> &Provider {
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
            serde_json::Value::String(self.proc.to_string()),
        );
        map.insert(
            "language".to_string(),
            serde_json::Value::String(self.content.language.clone()),
        );
        map.insert(
            "provider".to_string(),
            serde_json::Value::String(self.prov.to_string()),
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
    let explicit_title = entry
        .and_then(|e| e.get("title"))
        .and_then(|v| v.as_str())
        .or_else(|| entry.and_then(|e| e.get("topic")).and_then(|v| v.as_str()))
        .or_else(|| item.get("topic").and_then(|v| v.as_str()));
    let explicit_questions = entry
        .and_then(|e| e.get("questions"))
        .and_then(|v| v.as_array())
        .filter(|a| !a.is_empty())
        .map(|arr| arr.iter().map(json_to_question).collect::<Vec<Question>>())
        .or_else(|| {
            entry
                .and_then(|e| e.get("items"))
                .and_then(|v| v.as_array())
                .filter(|a| !a.is_empty())
                .map(|arr| arr.iter().map(json_to_question).collect::<Vec<Question>>())
        });
    let run_id = item.get("run_id").and_then(|v| v.as_str()).unwrap_or("");
    let proc_text = item.get("processor").and_then(|v| v.as_str()).unwrap_or("");
    let language = entry
        .and_then(|e| e.get("language"))
        .and_then(|v| v.as_str())
        .or_else(|| item.get("language").and_then(|v| v.as_str()))
        .unwrap_or("");
    let prov = item
        .get("provider")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<Provider>().ok())
        .unwrap_or(Provider::Parallel);
    let proc = processor::resolve(proc_text, &prov)
        .unwrap_or(Processor::Parallel(crate::processor::ParallelMode::Pro));
    let content = brief::parse(
        query_text,
        language,
        explicit_title,
        explicit_questions.as_deref(),
    );
    PendingRun {
        code: run_id.to_string(),
        content,
        proc,
        prov,
    }
}

/// Convert JSON value to Question.
fn json_to_question(value: &serde_json::Value) -> Question {
    let scope = value
        .get("scope")
        .and_then(|v| v.as_str())
        .or_else(|| value.get("text").and_then(|v| v.as_str()))
        .unwrap_or("")
        .to_string();
    let details = value
        .get("details")
        .and_then(|v| v.as_array())
        .or_else(|| value.get("items").and_then(|v| v.as_array()))
        .map(|arr| arr.iter().map(json_to_question).collect())
        .unwrap_or_default();
    Question { scope, details }
}

#[cfg(test)]
mod tests;
