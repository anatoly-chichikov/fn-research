use std::fs;
use std::path::Path;

use research_domain::session::{ResearchSession, Sessioned};

use crate::organizer::{self, Organized};

/// Persist session list into output folder.
pub fn store(root: &Path, items: &[ResearchSession]) {
    let org = organizer::organizer(root);
    for item in items {
        let name = org.name(item.created(), item.topic(), item.id());
        let base = root.join(&name);
        fs::create_dir_all(&base).ok();
        let path = base.join("session.json");
        let data = item.data();
        let text = serde_json::to_string_pretty(&serde_json::to_value(&data).unwrap_or_default())
            .unwrap_or_default();
        fs::write(&path, text).ok();
    }
}
