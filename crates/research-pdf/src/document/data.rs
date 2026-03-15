use std::path::{Path, PathBuf};

use research_api::response::{self, Responded};
use research_domain::result::CitationSource;
use research_domain::session::Sessioned;
use research_domain::task::{ResearchRun, Tasked};
use research_storage::organizer::{self, Organized};

use super::env;

/// Load raw response from output folder.
pub fn raw(root: &Path, session: &dyn Sessioned, task: Option<&ResearchRun>) -> serde_json::Value {
    let org = organizer::Organizer::new(root);
    let name = org.name(session.created(), session.topic(), session.id());
    let base = root.join(&name);
    match task {
        Some(run) => {
            let tag = organizer::slug(&env::provider(run));
            let tag = if tag.is_empty() {
                "provider".to_string()
            } else {
                tag
            };
            let path = base.join(format!("response-{}.json", tag));
            if path.exists() {
                let content = std::fs::read_to_string(&path).unwrap_or_default();
                serde_json::from_str(&content)
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()))
            } else {
                serde_json::Value::Object(serde_json::Map::new())
            }
        }
        None => serde_json::Value::Object(serde_json::Map::new()),
    }
}

/// Build response from raw map and task.
pub fn responsemap(
    root: &Path,
    session: &dyn Sessioned,
    raw: &serde_json::Value,
    task: &ResearchRun,
) -> response::Response {
    let name = env::provider(task);
    if name == "valyu" {
        let output = raw.get("output");
        let text = match output {
            Some(serde_json::Value::Object(map)) => map
                .get("markdown")
                .or_else(|| map.get("content"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            Some(serde_json::Value::String(s)) => s.clone(),
            _ => String::new(),
        };
        let text = images(root, session, &text, raw, task);
        let sources = raw
            .get("sources")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let basis: Vec<serde_json::Value> = sources
            .iter()
            .filter_map(|s| {
                let url = s.get("url").and_then(|v| v.as_str()).unwrap_or("");
                if url.is_empty() {
                    return None;
                }
                let title = s.get("title").and_then(|v| v.as_str()).unwrap_or("");
                let excerpt = s
                    .get("snippet")
                    .or_else(|| s.get("excerpt"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                Some(serde_json::json!({
                    "citations": [{
                        "title": title,
                        "url": url,
                        "excerpts": [excerpt]
                    }]
                }))
            })
            .collect();
        let state = raw.get("status");
        let status = match state {
            Some(serde_json::Value::Object(map)) => map
                .get("value")
                .and_then(|v| v.as_str())
                .unwrap_or("completed"),
            Some(serde_json::Value::String(s)) => s.as_str(),
            _ => "completed",
        };
        let code = raw
            .get("deepresearch_id")
            .or_else(|| raw.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let data = serde_json::json!({
            "id": code,
            "status": status,
            "output": text,
            "basis": basis,
            "raw": raw
        });
        response::response(&data)
    } else {
        let output = raw.get("output");
        let text = match output {
            Some(serde_json::Value::Object(map)) => map
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            _ => String::new(),
        };
        let basis = match output {
            Some(serde_json::Value::Object(map)) => map
                .get("basis")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default(),
            _ => Vec::new(),
        };
        let run = raw.get("run");
        let code = match run {
            Some(serde_json::Value::Object(map)) => {
                map.get("run_id").and_then(|v| v.as_str()).unwrap_or("")
            }
            _ => "",
        };
        let status = match run {
            Some(serde_json::Value::Object(map)) => map
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("completed"),
            _ => "completed",
        };
        let data = serde_json::json!({
            "id": code,
            "status": status,
            "output": text,
            "basis": basis,
            "raw": raw
        });
        response::response(&data)
    }
}

/// Return text and sources for task.
pub fn resultmap(
    root: &Path,
    session: &dyn Sessioned,
    task: &ResearchRun,
) -> (String, Vec<CitationSource>) {
    let data = raw(root, session, Some(task));
    if data.is_object() && !data.as_object().unwrap().is_empty() {
        let resp = responsemap(root, session, &data, task);
        (resp.text().to_string(), resp.sources())
    } else {
        let value = task.report();
        (value.summary(), value.sources().to_vec())
    }
}

/// Append images block to markdown.
pub fn images(
    root: &Path,
    session: &dyn Sessioned,
    text: &str,
    raw: &serde_json::Value,
    task: &ResearchRun,
) -> String {
    let items = raw
        .get("images")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let org = organizer::Organizer::new(root);
    let name = org.name(session.created(), session.topic(), session.id());
    let base = root.join(&name);
    let tag = organizer::slug(&env::provider(task));
    let tag = if tag.is_empty() {
        "provider".to_string()
    } else {
        tag
    };
    let folder = base.join(format!("images-{}", tag));
    let ext_re = regex::Regex::new(r"(\.[^./]+)$").unwrap();
    let mut lines: Vec<String> = Vec::new();
    for item in &items {
        let link = item.get("image_url").and_then(|v| v.as_str()).unwrap_or("");
        let title = item
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Chart");
        let code = item.get("image_id").and_then(|v| v.as_str()).unwrap_or("");
        let path = if link.is_empty() {
            String::new()
        } else {
            url::Url::parse(link)
                .ok()
                .map(|u| u.path().to_string())
                .unwrap_or_default()
        };
        let ext = ext_re
            .captures(&path)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| ".png".to_string());
        let file = if code.is_empty() {
            PathBuf::new()
        } else {
            folder.join(format!("{}{}", code, ext))
        };
        let resolved = if !code.is_empty() && file.exists() {
            format!("file://{}", file.display())
        } else {
            link.to_string()
        };
        let add = !resolved.is_empty() && !text.contains(&resolved);
        if add {
            lines.push(format!("![{}]({})", title, resolved));
        }
    }
    let block = if lines.is_empty() {
        String::new()
    } else {
        format!("## Images\n\n{}", lines.join("\n"))
    };
    let rows: Vec<&str> = text.split('\n').collect();
    let heading_re = regex::Regex::new(r"^#+\s*").unwrap();
    let mut idx: i32 = -1;
    for (i, row) in rows.iter().enumerate() {
        let line = row.trim();
        let label = heading_re.replace(line, "").to_lowercase();
        let is_heading = line.starts_with('#');
        let is_source = matches!(
            label.as_str(),
            "source" | "sources" | "reference" | "references"
        );
        if is_heading && is_source {
            idx = i as i32;
        }
    }
    if !block.is_empty() && idx >= 0 {
        let idx = idx as usize;
        let head = rows[..idx].join("\n");
        let tail = rows[idx..].join("\n");
        format!("{}\n\n{}\n\n{}", head.trim_end(), block, tail.trim_start())
    } else if !block.is_empty() {
        format!("{}\n\n{}", text.trim_end(), block)
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use research_api::response::Responded;
    use research_domain::ids;
    use research_domain::result::Sourced;

    fn valyu_task(provider: &str) -> research_domain::task::ResearchRun {
        let data = serde_json::json!({
            "query": "q",
            "status": "completed",
            "service": format!("{}.ai", provider),
            "created": "2026-01-01T00:00:00"
        });
        research_domain::task::task(&data)
    }

    fn valyu_session(topic: &str) -> research_domain::session::ResearchSession {
        let data = serde_json::json!({
            "topic": topic,
            "tasks": [],
            "created": "2026-01-01T00:00:00"
        });
        research_domain::session::session(&data)
    }

    #[test]
    fn the_data_responsemap_valyu_preserves_sources() {
        let mut rng = ids::ids(26001);
        let title = ids::cyrillic(&mut rng, 6);
        let number = ids::digit(&mut rng, 1000);
        let link = format!("https://example.com/{}", number);
        let snippet = ids::greek(&mut rng, 6);
        let output = ids::hiragana(&mut rng, 6);
        let raw = serde_json::json!({
            "status": "completed",
            "output": {"markdown": output},
            "sources": [{"url": link, "title": title, "snippet": snippet}],
            "deepresearch_id": "dr_test"
        });
        let session = valyu_session("topic");
        let task = valyu_task("valyu");
        let resp = responsemap(Path::new("output"), &session, &raw, &task);
        let sources = resp.sources();
        assert_eq!(
            1,
            sources.len(),
            "Valyu sources were not preserved in responsemap"
        );
    }

    #[test]
    fn the_data_responsemap_valyu_maps_snippet_to_excerpt() {
        let mut rng = ids::ids(26003);
        let title = ids::armenian(&mut rng, 6);
        let number = ids::digit(&mut rng, 1000);
        let link = format!("https://example.com/{}", number);
        let snippet = ids::hebrew(&mut rng, 6);
        let raw = serde_json::json!({
            "status": "completed",
            "output": {"markdown": "text"},
            "sources": [{"url": link, "title": title, "snippet": snippet}],
            "deepresearch_id": "dr_test"
        });
        let session = valyu_session("topic");
        let task = valyu_task("valyu");
        let resp = responsemap(Path::new("output"), &session, &raw, &task);
        let source = resp.sources().into_iter().next().unwrap();
        assert_eq!(
            snippet,
            source.excerpt(),
            "Valyu snippet was not mapped to excerpt"
        );
    }
}
