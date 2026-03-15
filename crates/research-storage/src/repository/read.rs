use regex::Regex;
use std::fs;
use std::path::Path;

use research_domain::session::{self, ResearchSession, Sessioned};

/// Return sessions from output folder.
pub fn items(root: &Path) -> Vec<ResearchSession> {
    if !root.exists() {
        return Vec::new();
    }
    let mut list: Vec<ResearchSession> = Vec::new();
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };
    let rule = Regex::new(r"^(\d{4}-\d{2}-\d{2})_(.+)_([A-Za-z0-9]{8})$").unwrap();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let edn = path.join("session.edn");
        let json = path.join("session.json");
        if json.exists() {
            if let Some(item) = load_json(&json) {
                list.push(item);
            }
        } else if edn.exists() {
            if let Some(item) = load_edn(&edn) {
                list.push(item);
            }
        } else if let Some(caps) = rule.captures(&name) {
            let date = caps[1].to_string();
            let slug = caps[2].to_string();
            let code = caps[3].to_string();
            let time = format!("{}T00:00:00", date);
            let id = format!("{}-migrated", code);
            let names = response_files(&path);
            let tasks: Vec<serde_json::Value> = names
                .iter()
                .enumerate()
                .map(|(index, file_name)| {
                    let size = file_name.len();
                    let tag = &file_name[9..size - 5];
                    let tid = format!("{}-{}-{}", code, tag, index);
                    let service = if tag == "xai" {
                        "x.ai".to_string()
                    } else {
                        format!("{}.ai", tag)
                    };
                    serde_json::json!({
                        "id": tid,
                        "status": "completed",
                        "service": service,
                        "created": time
                    })
                })
                .collect();
            let data = serde_json::json!({
                "id": id,
                "topic": slug,
                "tasks": tasks,
                "created": time
            });
            let item = session::session(&data);
            let text = serde_json::to_string_pretty(
                &serde_json::to_value(item.data()).unwrap_or_default(),
            )
            .unwrap_or_default();
            fs::write(&edn, &text).ok();
            list.push(item);
        }
    }
    list.sort_by(|a, b| a.created().cmp(b.created()));
    list
}

/// Load session from JSON file.
fn load_json(path: &Path) -> Option<ResearchSession> {
    let text = fs::read_to_string(path).ok()?;
    let data: serde_json::Value = serde_json::from_str(&text).ok()?;
    Some(session::session(&data))
}

/// Load session from EDN file (may contain JSON-compatible format).
fn load_edn(path: &Path) -> Option<ResearchSession> {
    let text = fs::read_to_string(path).ok()?;
    let data: serde_json::Value = serde_json::from_str(&text)
        .ok()
        .or_else(|| parse_edn_to_json(&text))?;
    Some(session::session(&data))
}

/// Parse EDN string into JSON value.
fn parse_edn_to_json(text: &str) -> Option<serde_json::Value> {
    let parsed: edn_rs::Edn = text.parse().ok()?;
    edn_to_json(&parsed)
}

/// Convert EDN value to JSON value.
fn edn_to_json(edn: &edn_rs::Edn) -> Option<serde_json::Value> {
    match edn {
        edn_rs::Edn::Str(s) => Some(serde_json::Value::String(s.clone())),
        edn_rs::Edn::Int(n) => Some(serde_json::json!(*n)),
        edn_rs::Edn::UInt(n) => Some(serde_json::json!(*n)),
        edn_rs::Edn::Double(n) => {
            let f: f64 = n.to_string().parse().unwrap_or(0.0);
            Some(serde_json::json!(f))
        }
        edn_rs::Edn::Bool(b) => Some(serde_json::Value::Bool(*b)),
        edn_rs::Edn::Nil => Some(serde_json::Value::Null),
        edn_rs::Edn::Key(k) => {
            let name = k.strip_prefix(':').unwrap_or(k);
            Some(serde_json::Value::String(name.to_string()))
        }
        edn_rs::Edn::Vector(v) => {
            let vec = v.clone().to_vec();
            let items: Vec<serde_json::Value> = vec.iter().filter_map(edn_to_json).collect();
            Some(serde_json::Value::Array(items))
        }
        edn_rs::Edn::Map(m) => {
            let mut map = serde_json::Map::new();
            let btree = m.clone().to_map();
            for (k, v) in &btree {
                let key = k.strip_prefix(':').unwrap_or(k).to_string();
                if let Some(val) = edn_to_json(v) {
                    map.insert(key, val);
                }
            }
            Some(serde_json::Value::Object(map))
        }
        edn_rs::Edn::List(l) => {
            let vec = l.clone().to_vec();
            let items: Vec<serde_json::Value> = vec.iter().filter_map(edn_to_json).collect();
            Some(serde_json::Value::Array(items))
        }
        edn_rs::Edn::Empty => Some(serde_json::Value::Null),
        _ => None,
    }
}

/// List response-*.json files in directory.
fn response_files(path: &Path) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("response-") && name.ends_with(".json") {
                names.push(name);
            }
        }
    }
    names.sort();
    names
}
