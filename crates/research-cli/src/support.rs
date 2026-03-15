use std::path::Path;

use research_storage::organizer::{self, Organized};

/// Return environment value by key.
pub fn env(key: &str) -> String {
    std::env::var(key).unwrap_or_default()
}

/// Store Valyu images or XAI prompts for output folder.
pub fn store(name: &str, provider: &str, data: &serde_json::Value, root: &Path) {
    let items = data
        .get("images")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let prompts = data
        .get("prompts")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if provider == "valyu" && !items.is_empty() {
        let org = organizer::organizer(root);
        let tag = organizer::slug(provider);
        let tag = if tag.is_empty() {
            "provider".to_string()
        } else {
            tag
        };
        let folder = org.folder(name, provider);
        let base = folder.join(format!("images-{}", tag));
        for item in &items {
            let link = item.get("image_url").and_then(|v| v.as_str()).unwrap_or("");
            let code = item.get("image_id").and_then(|v| v.as_str()).unwrap_or("");
            if link.is_empty() || code.is_empty() {
                continue;
            }
            let ext = link
                .rfind('.')
                .map(|i| &link[i..])
                .filter(|s| s.len() < 6)
                .unwrap_or(".png");
            let target = base.join("images").join(format!("{}{}", code, ext));
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            if !target.exists() {
                let resp = reqwest::blocking::get(link);
                if let Ok(resp) = resp {
                    if resp.status().as_u16() < 300 {
                        if let Ok(bytes) = resp.bytes() {
                            std::fs::write(&target, &bytes).ok();
                        }
                    }
                }
            }
        }
    }
    if provider == "xai" && !prompts.is_empty() {
        let org = organizer::organizer(root);
        let tag = organizer::slug(provider);
        let tag = if tag.is_empty() {
            "provider".to_string()
        } else {
            tag
        };
        let folder = org.folder(name, provider);
        let path = folder.join(format!("prompts-{}.edn", tag));
        let text = serde_json::to_string_pretty(&prompts).unwrap_or_default();
        std::fs::write(&path, text.as_bytes()).ok();
    }
}
