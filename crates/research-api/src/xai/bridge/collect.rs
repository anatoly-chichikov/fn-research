use std::path::PathBuf;

use super::fetch::FetchResult;
use crate::xai::brief::BriefItem;

/// Result of multi-prompt collection.
#[derive(Debug)]
pub struct CollectResult {
    /// Markdown parts per section.
    pub parts: Vec<String>,
    /// All citations.
    pub marks: Vec<super::super::citations::Citation>,
    /// All reference links.
    pub links: Vec<String>,
    /// Prompt strings sent.
    pub prompts: Vec<String>,
}

/// Return resources path.
fn root() -> PathBuf {
    if let Ok(dir) = std::env::var("RESOURCES_DIR") {
        return PathBuf::from(dir);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../resources")
}

/// Convert camelCase key to kebab-case.
fn kebab(text: &str) -> String {
    let mut result = String::new();
    for ch in text.chars() {
        if ch.is_uppercase() {
            result.push('-');
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch);
        }
    }
    result
}

/// Walk JSON and convert all object keys to kebab-case.
fn normalize(data: &serde_json::Value) -> serde_json::Value {
    match data {
        serde_json::Value::Object(map) => {
            let mut result = serde_json::Map::new();
            for (k, v) in map {
                result.insert(kebab(k), normalize(v));
            }
            serde_json::Value::Object(result)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(normalize).collect())
        }
        _ => data.clone(),
    }
}

/// Return prompt template as JSON value.
pub fn template() -> serde_json::Value {
    let path = root().join("prompts/xai-section.edn");
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    if text.trim().is_empty() {
        panic!("Prompt template is missing");
    }
    let edn: edn_rs::Edn = text
        .parse()
        .unwrap_or_else(|_| panic!("Prompt template parse failed"));
    let json = edn.to_json();
    let raw: serde_json::Value =
        serde_json::from_str(&json).unwrap_or_else(|_| panic!("Prompt template conversion failed"));
    normalize(&raw)
}

/// Return prompt map with replacements applied.
pub fn fill(
    data: &serde_json::Value,
    slots: &std::collections::HashMap<String, String>,
) -> serde_json::Value {
    match data {
        serde_json::Value::String(s) => {
            if let Some(val) = slots.get(s.as_str()) {
                serde_json::Value::String(val.clone())
            } else {
                data.clone()
            }
        }
        serde_json::Value::Object(map) => {
            let mut result = serde_json::Map::new();
            for (k, v) in map {
                result.insert(k.clone(), fill(v, slots));
            }
            serde_json::Value::Object(result)
        }
        serde_json::Value::Array(arr) => {
            let items: Vec<serde_json::Value> = arr.iter().map(|v| fill(v, slots)).collect();
            serde_json::Value::Array(items)
        }
        _ => data.clone(),
    }
}

/// Collect response data for multi prompt.
pub fn collect(
    fetcher: &dyn Fn(&str) -> FetchResult,
    head: &[String],
    items: &[BriefItem],
    top: &str,
) -> CollectResult {
    let topic = top.trim().to_string();
    let line = head.iter().find(|item| {
        let lower = item.trim().to_lowercase();
        lower.starts_with("язык ответа:") || lower.starts_with("response language:")
    });
    let text = line.map(|s| s.trim().to_string()).unwrap_or_default();
    let lower = text.to_lowercase();
    let text = if lower.starts_with("язык ответа:") {
        text["Язык ответа:".len()..].to_string()
    } else if lower.starts_with("response language:") {
        text["Response language:".len()..].to_string()
    } else {
        text
    };
    let text = text.trim().to_string();
    let text = if text.ends_with('.') {
        text[..text.len() - 1].to_string()
    } else {
        text
    };
    let language = if text.is_empty() {
        "unspecified".to_string()
    } else {
        text
    };
    let temp = template();
    let list: Vec<BriefItem> = items
        .iter()
        .filter(|item| !item.text.trim().is_empty())
        .map(|item| BriefItem {
            depth: item.depth.clamp(1, 3),
            text: item.text.trim().to_string(),
        })
        .collect();
    let size = list.len();
    let mut path: Vec<String> = Vec::new();
    let mut leaves: Vec<(String, String)> = Vec::new();
    for idx in 0..size {
        let item = &list[idx];
        let depth = if item.depth > path.len() + 1 {
            path.len() + 1
        } else {
            item.depth
        };
        path.truncate(depth - 1);
        path.push(item.text.clone());
        let next = if idx + 1 < size {
            list[idx + 1].depth
        } else {
            0
        };
        let leaf = next <= item.depth;
        if leaf {
            let name = path.last().cloned().unwrap_or_default();
            let head = &path[..path.len().saturating_sub(1)];
            let text = if !head.is_empty() {
                format!("Context: {}\nFocus: {}", head.join(" / "), name)
            } else {
                name.clone()
            };
            leaves.push((name, text));
        }
    }
    let num_re = regex::Regex::new(r"^\d+[.)]\s+").unwrap();
    let mut parts = Vec::new();
    let mut marks = Vec::new();
    let mut all_links: Vec<String> = Vec::new();
    let mut prompts = Vec::new();
    for (name, text) in &leaves {
        let mut slots = std::collections::HashMap::new();
        slots.insert("<<response_language>>".to_string(), language.clone());
        slots.insert("<<topic>>".to_string(), topic.clone());
        slots.insert("<<section_title>>".to_string(), name.clone());
        slots.insert("<<section_details>>".to_string(), text.clone());
        let data = fill(&temp, &slots);
        let prompt = serde_json::to_string(&data).unwrap_or_default();
        let result = fetcher(&prompt);
        let body = result.body;
        let cells = result.cells;
        let refs = result.links;
        let rows: Vec<&str> = body.lines().collect();
        let mut lines: Vec<String> = Vec::new();
        let mut seen = false;
        let mut title = String::new();
        let mut flag = false;
        for row in &rows {
            let trim = row.trim_start();
            let blank = row.trim().is_empty();
            let mark = trim.starts_with('#');
            let lead = if mark {
                trim.chars().take_while(|c| *c == '#').count()
            } else {
                0
            };
            let heading = if mark {
                trim[lead..].trim_start().to_string()
            } else {
                String::new()
            };
            let heading = num_re.replace(&heading, "").trim().to_string();
            let start = mark && !seen;
            if start {
                title = heading.clone();
            }
            let dup = seen && heading.to_lowercase() == title.to_lowercase();
            let mask = flag && blank;
            let line = if mark && !dup {
                if seen {
                    format!("### {}", heading)
                } else {
                    format!("## {}", heading)
                }
            } else {
                String::new()
            };
            if mask {
                // skip
            } else if mark && !dup {
                lines.push(line);
            } else if mark && dup {
                // skip duplicate heading
            } else {
                lines.push(row.to_string());
            }
            if mask {
                flag = false;
            } else {
                flag = mark && dup;
            }
            if mark {
                seen = true;
            }
        }
        let body = lines.join("\n").trim().to_string();
        all_links.extend(refs);
        parts.push(body);
        marks.extend(cells);
        prompts.push(prompt);
    }
    CollectResult {
        parts,
        marks,
        links: all_links,
        prompts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use research_domain::ids;

    #[test]
    fn the_collector_normalizes_heading_levels() {
        let mut rng = ids::ids(18311);
        let topic = ids::greek(&mut rng, 6);
        let alpha = ids::armenian(&mut rng, 5);
        let beta = ids::arabic(&mut rng, 5);
        let gamma = ids::hebrew(&mut rng, 5);
        let delta = ids::hiragana(&mut rng, 5);
        let eta = ids::hebrew(&mut rng, 4);
        let theta = ids::arabic(&mut rng, 4);
        let iota = ids::armenian(&mut rng, 4);
        let lang_word = ids::greek(&mut rng, 4);
        let lang = format!("Response language: {}.", lang_word);
        let head = vec![lang, String::new(), topic.clone()];
        let body = format!("# {}\n\n## {}\n\n### {}", eta, theta, iota);
        let fetcher = |_: &str| FetchResult {
            body: body.clone(),
            cells: Vec::new(),
            links: Vec::new(),
        };
        let left = format!("{}: {} — {}", alpha, beta, gamma);
        let right = format!("{} — {}", delta, gamma);
        let items = vec![
            BriefItem {
                depth: 1,
                text: left,
            },
            BriefItem {
                depth: 1,
                text: right,
            },
        ];
        let data = collect(&fetcher, &head, &items, &topic);
        let target = vec![
            format!("## {}\n\n### {}\n\n### {}", eta, theta, iota),
            format!("## {}\n\n### {}\n\n### {}", eta, theta, iota),
        ];
        assert_eq!(target, data.parts, "output did not match expected headings");
    }

    #[test]
    fn the_collector_drops_duplicate_headings() {
        let mut rng = ids::ids(18313);
        let topic = ids::greek(&mut rng, 6);
        let alpha = ids::armenian(&mut rng, 5);
        let _beta = ids::hebrew(&mut rng, 5);
        let gamma = ids::hiragana(&mut rng, 4);
        let lang_word = ids::greek(&mut rng, 4);
        let lang = format!("Response language: {}.", lang_word);
        let head = vec![lang, String::new(), topic.clone()];
        let body = format!("# {}\n\n## {}\n\n### {}", alpha, alpha, gamma);
        let fetcher = |_: &str| FetchResult {
            body: body.clone(),
            cells: Vec::new(),
            links: Vec::new(),
        };
        let item = format!("{} — {}", _beta, gamma);
        let items = vec![BriefItem {
            depth: 1,
            text: item,
        }];
        let data = collect(&fetcher, &head, &items, &topic);
        let target = vec![format!("## {}\n\n### {}", alpha, gamma)];
        assert_eq!(target, data.parts, "Duplicate heading was not removed");
    }

    #[test]
    fn the_collector_builds_prompt_with_section_context() {
        let mut rng = ids::ids(18317);
        let topic = ids::greek(&mut rng, 6);
        let alpha = ids::arabic(&mut rng, 5);
        let beta = ids::hebrew(&mut rng, 6);
        let lang_word = ids::greek(&mut rng, 4);
        let lang = format!("Response language: {}.", lang_word);
        let head = vec![lang, String::new(), topic.clone()];
        let text = format!("{} — {}", alpha, beta);
        let cell = std::sync::Mutex::new(String::new());
        let fetcher = |note: &str| {
            *cell.lock().unwrap() = note.to_string();
            FetchResult {
                body: String::new(),
                cells: Vec::new(),
                links: Vec::new(),
            }
        };
        let items = vec![BriefItem {
            depth: 1,
            text: text.clone(),
        }];
        collect(&fetcher, &head, &items, &topic);
        let prompt = cell.lock().unwrap().clone();
        let data: serde_json::Value = serde_json::from_str(&prompt).unwrap();
        let check = data.get("topic").and_then(|v| v.as_str()) == Some(&topic)
            && data.get("section-title").and_then(|v| v.as_str()) == Some(&text)
            && data.get("section-details").and_then(|v| v.as_str()) == Some(&text)
            && data.get("response-language").and_then(|v| v.as_str()) == Some(&lang_word)
            && data.get("scope").and_then(|v| v.as_str()).is_some()
            && data
                .get("heading-guidance")
                .and_then(|v| v.as_str())
                .is_some()
            && data.get("task").and_then(|v| v.as_str()).is_some()
            && data
                .get("task")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .contains("deep research");
        assert!(check, "prompt text did not match expected format");
    }

    #[test]
    fn the_collector_expands_nested_items() {
        let mut rng = ids::ids(18318);
        let topic = ids::greek(&mut rng, 6);
        let alpha = ids::arabic(&mut rng, 5);
        let beta = ids::hebrew(&mut rng, 6);
        let gamma = ids::hiragana(&mut rng, 4);
        let delta = ids::armenian(&mut rng, 5);
        let lang_word = ids::greek(&mut rng, 4);
        let lang = format!("Response language: {}.", lang_word);
        let head = vec![lang, String::new(), topic.clone()];
        let items = vec![
            BriefItem {
                depth: 1,
                text: alpha.clone(),
            },
            BriefItem {
                depth: 2,
                text: beta.clone(),
            },
            BriefItem {
                depth: 2,
                text: gamma.clone(),
            },
            BriefItem {
                depth: 1,
                text: delta.clone(),
            },
        ];
        let data = collect(
            &|_| FetchResult {
                body: String::new(),
                cells: Vec::new(),
                links: Vec::new(),
            },
            &head,
            &items,
            &topic,
        );
        let list: Vec<serde_json::Value> = data
            .prompts
            .iter()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect();
        let keep: std::collections::HashSet<String> = list
            .iter()
            .filter_map(|v| {
                v.get("section-title")
                    .and_then(|s| s.as_str())
                    .map(String::from)
            })
            .collect();
        let check = list.len() == 3
            && list[0].get("section-title").and_then(|v| v.as_str()) == Some(&beta)
            && list[0].get("section-details").and_then(|v| v.as_str())
                == Some(&format!("Context: {}\nFocus: {}", alpha, beta))
            && list[1].get("section-title").and_then(|v| v.as_str()) == Some(&gamma)
            && list[1].get("section-details").and_then(|v| v.as_str())
                == Some(&format!("Context: {}\nFocus: {}", alpha, gamma))
            && list[2].get("section-title").and_then(|v| v.as_str()) == Some(&delta)
            && list[2].get("section-details").and_then(|v| v.as_str()) == Some(&delta)
            && !keep.contains(&alpha);
        assert!(check, "Nested items were not expanded into leaf prompts");
    }
}
