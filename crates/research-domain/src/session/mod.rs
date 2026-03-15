use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::pending::{self, PendingRun, Pendinged};
use crate::processor::{self, Processor};
use crate::provider::Provider;
use crate::task::{self, ResearchRun, Tasked};

/// Object representing research session.
pub trait Sessioned {
    /// Return session identifier.
    fn id(&self) -> &str;
    /// Return session topic.
    fn topic(&self) -> &str;
    /// Return task list.
    fn tasks(&self) -> &[ResearchRun];
    /// Return creation time.
    fn created(&self) -> &NaiveDateTime;
    /// Return pending run.
    fn pending(&self) -> Option<&PendingRun>;
    /// Return research query.
    fn query(&self) -> &str;
    /// Return processor.
    fn processor(&self) -> &Processor;
    /// Return research language.
    fn language(&self) -> &str;
    /// Return provider.
    fn provider(&self) -> &Provider;
    /// Return new session with appended task.
    fn extend(&self, value: ResearchRun) -> ResearchSession;
    /// Return session with pending run.
    fn start(&self, value: PendingRun) -> ResearchSession;
    /// Return session without pending run.
    fn reset(&self) -> ResearchSession;
    /// Return session with updated research parameters.
    fn reconfigure(&self, opts: &HashMap<String, String>) -> ResearchSession;
    /// Return map representation.
    fn data(&self) -> HashMap<String, serde_json::Value>;
}

/// Research session record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResearchSession {
    code: String,
    name: String,
    runs: Vec<ResearchRun>,
    stamp: NaiveDateTime,
    hold: Option<PendingRun>,
    text: String,
    proc: Processor,
    lang: String,
    prov: Provider,
}

impl Sessioned for ResearchSession {
    fn id(&self) -> &str {
        &self.code
    }

    fn topic(&self) -> &str {
        &self.name
    }

    fn tasks(&self) -> &[ResearchRun] {
        &self.runs
    }

    fn created(&self) -> &NaiveDateTime {
        &self.stamp
    }

    fn pending(&self) -> Option<&PendingRun> {
        self.hold.as_ref()
    }

    fn query(&self) -> &str {
        &self.text
    }

    fn processor(&self) -> &Processor {
        &self.proc
    }

    fn language(&self) -> &str {
        &self.lang
    }

    fn provider(&self) -> &Provider {
        &self.prov
    }

    fn extend(&self, value: ResearchRun) -> ResearchSession {
        let mut runs = self.runs.clone();
        runs.push(value);
        ResearchSession {
            code: self.code.clone(),
            name: self.name.clone(),
            runs,
            stamp: self.stamp,
            hold: None,
            text: self.text.clone(),
            proc: self.proc,
            lang: self.lang.clone(),
            prov: self.prov,
        }
    }

    fn start(&self, value: PendingRun) -> ResearchSession {
        ResearchSession {
            code: self.code.clone(),
            name: self.name.clone(),
            runs: self.runs.clone(),
            stamp: self.stamp,
            hold: Some(value),
            text: self.text.clone(),
            proc: self.proc,
            lang: self.lang.clone(),
            prov: self.prov,
        }
    }

    fn reset(&self) -> ResearchSession {
        ResearchSession {
            code: self.code.clone(),
            name: self.name.clone(),
            runs: self.runs.clone(),
            stamp: self.stamp,
            hold: None,
            text: self.text.clone(),
            proc: self.proc,
            lang: self.lang.clone(),
            prov: self.prov,
        }
    }

    fn reconfigure(&self, opts: &HashMap<String, String>) -> ResearchSession {
        let prov = opts
            .get("provider")
            .and_then(|v| v.parse::<Provider>().ok())
            .unwrap_or(self.prov);
        let proc = opts
            .get("processor")
            .and_then(|v| processor::resolve(v, &prov).ok())
            .unwrap_or(self.proc);
        ResearchSession {
            code: self.code.clone(),
            name: self.name.clone(),
            runs: self.runs.clone(),
            stamp: self.stamp,
            hold: self.hold.clone(),
            text: opts
                .get("query")
                .cloned()
                .unwrap_or_else(|| self.text.clone()),
            proc,
            lang: opts
                .get("language")
                .cloned()
                .unwrap_or_else(|| self.lang.clone()),
            prov,
        }
    }

    fn data(&self) -> HashMap<String, serde_json::Value> {
        let mut map = HashMap::new();
        map.insert(
            "id".to_string(),
            serde_json::Value::String(self.code.clone()),
        );
        map.insert(
            "topic".to_string(),
            serde_json::Value::String(self.name.clone()),
        );
        let tasks: Vec<serde_json::Value> = self
            .runs
            .iter()
            .map(|t| serde_json::to_value(t.data()).unwrap())
            .collect();
        map.insert("tasks".to_string(), serde_json::Value::Array(tasks));
        map.insert(
            "created".to_string(),
            serde_json::Value::String(task::format(&self.stamp)),
        );
        if !self.text.is_empty() {
            map.insert(
                "query".to_string(),
                serde_json::Value::String(self.text.clone()),
            );
        }
        map.insert(
            "processor".to_string(),
            serde_json::Value::String(self.proc.to_string()),
        );
        if !self.lang.is_empty() {
            map.insert(
                "language".to_string(),
                serde_json::Value::String(self.lang.clone()),
            );
        }
        map.insert(
            "provider".to_string(),
            serde_json::Value::String(self.prov.to_string()),
        );
        if let Some(ref hold) = self.hold {
            map.insert(
                "pending".to_string(),
                serde_json::to_value(hold.data()).unwrap(),
            );
        }
        map
    }
}

/// Create session from JSON value.
pub fn session(item: &serde_json::Value) -> ResearchSession {
    let list: Vec<ResearchRun> = item
        .get("tasks")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().map(task::task).collect())
        .unwrap_or_default();
    let time = task::parse(
        item.get("created")
            .and_then(|v| v.as_str())
            .expect("Session missing created field"),
    );
    let hold = item
        .get("pending")
        .filter(|v| !v.is_null())
        .map(pending::pending);
    let code = item
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let prov = item
        .get("provider")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<Provider>().ok())
        .unwrap_or(Provider::Parallel);
    let proc_text = item
        .get("processor")
        .and_then(|v| v.as_str())
        .unwrap_or("pro");
    let proc = processor::resolve(proc_text, &prov)
        .unwrap_or(Processor::Parallel(crate::processor::ParallelMode::Pro));
    ResearchSession {
        code,
        name: item
            .get("topic")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        runs: list,
        stamp: time,
        hold,
        text: item
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        proc,
        lang: item
            .get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        prov,
    }
}

#[cfg(test)]
mod tests;
