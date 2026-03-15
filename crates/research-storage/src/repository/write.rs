use std::fs;
use std::path::Path;

use research_domain::session::{ResearchSession, Sessioned};

use crate::organizer::{self, Organized};

/// Persist session list into output folder.
pub fn store(root: &Path, items: &[ResearchSession]) {
    let org = organizer::organizer(root);
    let conf = ron::ser::PrettyConfig::default().struct_names(true);
    for item in items {
        let name = org.name(item.created(), item.topic(), item.id());
        let base = root.join(&name);
        fs::create_dir_all(&base).ok();
        let path = base.join("session.ron");
        let text = ron::ser::to_string_pretty(item, conf.clone()).unwrap_or_default();
        fs::write(&path, text).ok();
    }
}
