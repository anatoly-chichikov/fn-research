use std::path::{Path, PathBuf};

/// Object that can store XAI cache.
pub trait Stored {
    /// Return cache file path.
    fn path(&self, id: &str) -> PathBuf;
    /// Load cached payload.
    fn load(&self, id: &str) -> serde_json::Value;
    /// Save cached payload.
    fn save(&self, id: &str, data: &serde_json::Value) -> PathBuf;
}

/// XAI cache store.
pub struct Cache {
    root: PathBuf,
}

impl Cache {
    /// Create cache at root path.
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }
}

impl Stored for Cache {
    fn path(&self, id: &str) -> PathBuf {
        self.root
            .join("tmp_cache")
            .join("xai")
            .join(format!("{}.json", id))
    }

    fn load(&self, id: &str) -> serde_json::Value {
        let path = self.path(id);
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&text).unwrap_or(serde_json::Value::Null)
    }

    fn save(&self, id: &str, data: &serde_json::Value) -> PathBuf {
        let path = self.path(id);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).ok();
        }
        let text = serde_json::to_string_pretty(data).unwrap_or_default();
        std::fs::write(&path, text.as_bytes()).ok();
        path
    }
}

/// Return cache store.
pub fn make(root: &Path) -> Cache {
    Cache::new(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use research_domain::ids;

    #[test]
    fn the_cache_loads_saved_payload() {
        let mut rng = ids::ids(18309);
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let item = make(root);
        let text = ids::cyrillic(&mut rng, 6);
        let id = ids::ascii(&mut rng, 6);
        let data = serde_json::json!({
            "query": text,
            "config": {"mode": ids::greek(&mut rng, 4)}
        });
        item.save(&id, &data);
        let value = item.load(&id);
        assert_eq!(
            text,
            value.get("query").unwrap().as_str().unwrap(),
            "cache did not load payload"
        );
    }
}
