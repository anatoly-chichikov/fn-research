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
        let ron = path.join("session.ron");
        let json = path.join("session.json");
        if ron.exists() {
            if let Some(item) = load_ron(&ron) {
                list.push(item);
            }
        } else if json.exists() {
            if let Some(item) = load_json(&json) {
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
            let conf = ron::ser::PrettyConfig::default().struct_names(true);
            let text = ron::ser::to_string_pretty(&item, conf).unwrap_or_default();
            fs::write(&ron, &text).ok();
            list.push(item);
        }
    }
    list.sort_by(|a, b| a.created().cmp(b.created()));
    list
}

/// Load session from RON file.
fn load_ron(path: &Path) -> Option<ResearchSession> {
    let text = fs::read_to_string(path).ok()?;
    ron::from_str(&text).ok()
}

/// Load session from JSON file.
fn load_json(path: &Path) -> Option<ResearchSession> {
    let text = fs::read_to_string(path).ok()?;
    let data: serde_json::Value = serde_json::from_str(&text).ok()?;
    Some(session::session(&data))
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
