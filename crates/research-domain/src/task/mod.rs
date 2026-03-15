use chrono::NaiveDateTime;
use std::collections::HashMap;

use crate::brief::{self, Brief, Question};
use crate::processor::{self, Processor};
use crate::provider::{Labeled, Provider};
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
    fn provider(&self) -> &Provider;
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
    service: Provider,
    processor: Processor,
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
        brief::render(&self.content)
    }

    fn status(&self) -> &str {
        &self.status
    }

    fn report(&self) -> &Report {
        &self.value
    }

    fn language(&self) -> &str {
        &self.content.language
    }

    fn provider(&self) -> &Provider {
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
            service: self.service,
            processor: self.processor,
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
            serde_json::Value::String(self.content.language.clone()),
        );
        map.insert(
            "service".to_string(),
            serde_json::Value::String(self.service.label().to_string()),
        );
        map.insert(
            "processor".to_string(),
            serde_json::Value::String(self.processor.to_string()),
        );
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
    let language = item
        .get("language")
        .and_then(|v| v.as_str())
        .unwrap_or("\u{0440}\u{0443}\u{0441}\u{0441}\u{043a}\u{0438}\u{0439}");
    let service = item
        .get("service")
        .and_then(|v| v.as_str())
        .unwrap_or("parallel.ai")
        .parse::<Provider>()
        .unwrap_or(Provider::Parallel);
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
    let brief_language = entry
        .and_then(|e| e.get("language"))
        .and_then(|v| v.as_str())
        .unwrap_or(language);
    let content = brief::parse(
        query_text,
        brief_language,
        explicit_title,
        explicit_questions.as_deref(),
    );
    let raw = item.get("result");
    let value = result::result(raw);
    let code = item
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let proc_text = item
        .get("processor")
        .and_then(|v| v.as_str())
        .unwrap_or("pro");
    let processor = processor::resolve(proc_text, &service)
        .unwrap_or(Processor::Parallel(crate::processor::ParallelMode::Pro));
    ResearchRun {
        code,
        content,
        status: item
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        service,
        processor,
        stamp: time,
        done,
        value,
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
