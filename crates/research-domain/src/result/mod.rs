use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Object with URL source.
pub trait Sourced {
    /// Return source title.
    fn title(&self) -> &str;
    /// Return source URL.
    fn url(&self) -> &str;
    /// Return relevant excerpt.
    fn excerpt(&self) -> &str;
}

/// Object with text summary.
pub trait Summarized {
    /// Return text summary.
    fn summary(&self) -> String;
}

/// Object with sources list.
pub trait Listed {
    /// Return sources list.
    fn sources(&self) -> &[CitationSource];
}

/// Object that can serialize to map.
pub trait Serialized {
    /// Return map representation.
    fn data(&self) -> HashMap<String, serde_json::Value>;
}

/// Object with presence signal.
pub trait Presence {
    /// Return true when value is present.
    fn presence(&self) -> bool;
}

/// Citation source with title, url and excerpt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CitationSource {
    text: String,
    location: String,
    fragment: String,
}

impl CitationSource {
    /// Create citation source from components.
    pub fn new(title: &str, url: &str, excerpt: &str) -> Self {
        Self {
            text: title.to_string(),
            location: url.to_string(),
            fragment: excerpt.to_string(),
        }
    }
}

impl Sourced for CitationSource {
    fn title(&self) -> &str {
        &self.text
    }

    fn url(&self) -> &str {
        &self.location
    }

    fn excerpt(&self) -> &str {
        &self.fragment
    }
}

impl Serialized for CitationSource {
    fn data(&self) -> HashMap<String, serde_json::Value> {
        let mut map = HashMap::new();
        map.insert(
            "title".to_string(),
            serde_json::Value::String(self.text.clone()),
        );
        map.insert(
            "url".to_string(),
            serde_json::Value::String(self.location.clone()),
        );
        map.insert(
            "excerpt".to_string(),
            serde_json::Value::String(self.fragment.clone()),
        );
        map
    }
}

/// Create source from JSON value.
pub fn source(item: &serde_json::Value) -> CitationSource {
    CitationSource::new(
        item.get("title").and_then(|v| v.as_str()).unwrap_or(""),
        item.get("url").and_then(|v| v.as_str()).unwrap_or(""),
        item.get("excerpt").and_then(|v| v.as_str()).unwrap_or(""),
    )
}

/// Remove sources section from summary text.
pub fn purge(text: &str) -> String {
    let heading = Regex::new(r"(?m)^#{1,6}\s*Sources?\s*$").unwrap();
    if let Some(mat) = heading.find(text) {
        let before = &text[..mat.start()];
        let after = &text[mat.end()..];
        let next = Regex::new(r"(?m)^\#{1,6}\s").unwrap();
        let rest = if let Some(next_match) = next.find(after) {
            &after[next_match.start()..]
        } else {
            ""
        };
        format!("{}{}", before.trim_end_matches('\n'), rest)
    } else {
        text.to_string()
    }
}

/// Research report with summary and sources.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResearchReport {
    text: String,
    items: Vec<CitationSource>,
}

impl ResearchReport {
    /// Create report from summary and sources.
    pub fn new(summary: &str, sources: Vec<CitationSource>) -> Self {
        Self {
            text: summary.to_string(),
            items: sources,
        }
    }
}

impl Summarized for ResearchReport {
    fn summary(&self) -> String {
        purge(&self.text)
    }
}

impl Listed for ResearchReport {
    fn sources(&self) -> &[CitationSource] {
        &self.items
    }
}

impl Serialized for ResearchReport {
    fn data(&self) -> HashMap<String, serde_json::Value> {
        let mut map = HashMap::new();
        map.insert(
            "summary".to_string(),
            serde_json::Value::String(self.text.clone()),
        );
        let list: Vec<serde_json::Value> = self
            .items
            .iter()
            .map(|s| serde_json::to_value(s.data()).unwrap())
            .collect();
        map.insert("sources".to_string(), serde_json::Value::Array(list));
        map
    }
}

impl Presence for ResearchReport {
    fn presence(&self) -> bool {
        true
    }
}

impl std::fmt::Display for ResearchReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

/// Empty report placeholder.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EmptyReport {
    text: String,
    items: Vec<CitationSource>,
}

impl EmptyReport {
    /// Create empty report.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Summarized for EmptyReport {
    fn summary(&self) -> String {
        self.text.clone()
    }
}

impl Listed for EmptyReport {
    fn sources(&self) -> &[CitationSource] {
        &self.items
    }
}

impl Serialized for EmptyReport {
    fn data(&self) -> HashMap<String, serde_json::Value> {
        let mut map = HashMap::new();
        map.insert(
            "summary".to_string(),
            serde_json::Value::String(self.text.clone()),
        );
        map.insert("sources".to_string(), serde_json::Value::Array(Vec::new()));
        map
    }
}

impl Presence for EmptyReport {
    fn presence(&self) -> bool {
        false
    }
}

/// Dynamic report type that can be either real or empty.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Report {
    /// Full research report.
    Full(ResearchReport),
    /// Empty placeholder.
    Empty(EmptyReport),
}

impl Report {
    /// Return summary text.
    pub fn summary(&self) -> String {
        match self {
            Report::Full(r) => r.summary(),
            Report::Empty(r) => r.summary(),
        }
    }

    /// Return sources list.
    pub fn sources(&self) -> &[CitationSource] {
        match self {
            Report::Full(r) => r.sources(),
            Report::Empty(r) => r.sources(),
        }
    }

    /// Return map representation.
    pub fn data(&self) -> HashMap<String, serde_json::Value> {
        match self {
            Report::Full(r) => r.data(),
            Report::Empty(r) => r.data(),
        }
    }

    /// Return true when value is present.
    pub fn presence(&self) -> bool {
        match self {
            Report::Full(r) => r.presence(),
            Report::Empty(r) => r.presence(),
        }
    }
}

/// Create result from optional JSON value.
pub fn result(item: Option<&serde_json::Value>) -> Report {
    match item {
        Some(val) => {
            let raw = val.get("summary");
            let text = match raw {
                Some(serde_json::Value::Object(map)) => map
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                Some(serde_json::Value::String(s)) => s.clone(),
                _ => String::new(),
            };
            let list = val
                .get("sources")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().map(source).collect())
                .unwrap_or_default();
            Report::Full(ResearchReport::new(&text, list))
        }
        None => Report::Empty(EmptyReport::new()),
    }
}

#[cfg(test)]
mod tests;
