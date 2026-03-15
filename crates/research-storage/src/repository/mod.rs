pub mod read;
pub mod write;

use std::path::{Path, PathBuf};

use research_domain::session::{ResearchSession, Sessioned};

/// Object that can load sessions.
pub trait Loadable {
    /// Return session list.
    fn load(&self) -> Vec<ResearchSession>;
}

/// Object that can save sessions.
pub trait Savable {
    /// Persist session list.
    fn save(&self, items: &[ResearchSession]);
}

/// Object that can update sessions.
pub trait Mutable {
    /// Append session.
    fn append(&self, value: ResearchSession);
    /// Find session by id.
    fn find(&self, value: &str) -> Option<ResearchSession>;
    /// Update session by id.
    fn update(&self, value: ResearchSession);
}

/// Session repository.
pub struct Repository {
    root: PathBuf,
}

impl Repository {
    /// Create repository from output path.
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }
}

impl Loadable for Repository {
    fn load(&self) -> Vec<ResearchSession> {
        read::items(&self.root)
    }
}

impl Savable for Repository {
    fn save(&self, items: &[ResearchSession]) {
        write::store(&self.root, items);
    }
}

impl Mutable for Repository {
    fn append(&self, value: ResearchSession) {
        let mut items = self.load();
        items.push(value);
        self.save(&items);
    }

    fn find(&self, value: &str) -> Option<ResearchSession> {
        let items = self.load();
        items.into_iter().find(|i| i.id() == value)
    }

    fn update(&self, value: ResearchSession) {
        let items = self.load();
        let store: Vec<ResearchSession> = items
            .into_iter()
            .map(|i| {
                if i.id() == value.id() {
                    value.clone()
                } else {
                    i
                }
            })
            .collect();
        self.save(&store);
    }
}

/// Create repository from output path.
pub fn repo(root: &Path) -> Repository {
    Repository::new(root)
}

#[cfg(test)]
mod tests;
