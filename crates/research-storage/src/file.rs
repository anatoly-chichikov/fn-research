use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors from file operations.
#[derive(Error, Debug)]
pub enum FileError {
    /// File not found at path.
    #[error("File not found: {0}")]
    NotFound(String),
    /// IO error during read or write.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON parse error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Object that can read data.
pub trait Reader {
    /// Return file content as JSON value.
    fn read(&self) -> Result<serde_json::Value, FileError>;
}

/// Object that can write data.
pub trait Writer {
    /// Persist JSON value to file.
    fn write(&self, value: &serde_json::Value) -> Result<PathBuf, FileError>;
}

/// Object that can check file existence.
pub trait Existing {
    /// Return true when file exists.
    fn exists(&self) -> bool;
}

/// JSON file wrapper.
pub struct JsonFile {
    location: PathBuf,
}

impl JsonFile {
    /// Create JSON file wrapper from path.
    pub fn new(path: &Path) -> Self {
        Self {
            location: path.to_path_buf(),
        }
    }
}

impl Reader for JsonFile {
    fn read(&self) -> Result<serde_json::Value, FileError> {
        if !self.location.exists() {
            return Err(FileError::NotFound(self.location.display().to_string()));
        }
        let text = fs::read_to_string(&self.location)?;
        let value: serde_json::Value = serde_json::from_str(&text)?;
        Ok(value)
    }
}

impl Writer for JsonFile {
    fn write(&self, value: &serde_json::Value) -> Result<PathBuf, FileError> {
        if let Some(parent) = self.location.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(value)?;
        fs::write(&self.location, text)?;
        Ok(self.location.clone())
    }
}

impl Existing for JsonFile {
    fn exists(&self) -> bool {
        self.location.exists()
    }
}

impl std::fmt::Display for JsonFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.location.display())
    }
}

/// Create JsonFile from path.
pub fn file(path: &Path) -> JsonFile {
    JsonFile::new(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use research_domain::ids;
    use tempfile::TempDir;

    #[test]
    fn the_jsonfile_writes_and_reads_data() {
        let mut rng = ids::ids(22001);
        let dir = TempDir::new().unwrap();
        let name = format!("test-{}.json", ids::uuid(&mut rng));
        let path = dir.path().join(name);
        let item = file(&path);
        let key = ids::cyrillic(&mut rng, 6);
        let val = serde_json::json!({ key.clone(): "value" });
        item.write(&val).unwrap();
        let data = item.read().unwrap();
        assert_eq!(
            "value",
            data.get(&key).unwrap().as_str().unwrap(),
            "Read data did not match written data"
        );
    }

    #[test]
    fn the_jsonfile_raises_on_missing_file() {
        let mut rng = ids::ids(22003);
        let path = PathBuf::from(format!("/nonexistent-{}.json", ids::uuid(&mut rng)));
        let item = file(&path);
        let raised = item.read().is_err();
        assert!(raised, "Reading missing file did not raise exception");
    }

    #[test]
    fn the_jsonfile_exists_returns_false_for_missing() {
        let mut rng = ids::ids(22005);
        let path = PathBuf::from(format!("/nonexistent-{}.json", ids::uuid(&mut rng)));
        let item = file(&path);
        assert!(!item.exists(), "Exists returned true for missing file");
    }

    #[test]
    fn the_jsonfile_exists_returns_true_for_existing() {
        let mut rng = ids::ids(22007);
        let dir = TempDir::new().unwrap();
        let name = format!("test-{}.json", ids::uuid(&mut rng));
        let path = dir.path().join(name);
        let item = file(&path);
        item.write(&serde_json::json!({"test": true})).unwrap();
        assert!(item.exists(), "Exists returned false for existing file");
    }

    #[test]
    fn the_jsonfile_creates_parent_directories() {
        let mut rng = ids::ids(22009);
        let dir = TempDir::new().unwrap();
        let nest = format!("a-{}/b-{}", ids::uuid(&mut rng), ids::uuid(&mut rng));
        let path = dir.path().join(nest).join("test.json");
        let item = file(&path);
        item.write(&serde_json::json!({"nested": true})).unwrap();
        assert!(item.exists(), "File was not created in nested directory");
    }

    #[test]
    fn the_jsonfile_handles_unicode() {
        let mut rng = ids::ids(22011);
        let dir = TempDir::new().unwrap();
        let name = format!("unicode-{}.json", ids::uuid(&mut rng));
        let path = dir.path().join(name);
        let item = file(&path);
        let text = format!(
            "\u{65E5}\u{672C}\u{8A9E}\u{30C6}\u{30B9}\u{30C8}-{}",
            ids::uuid(&mut rng)
        );
        item.write(&serde_json::json!({"text": text})).unwrap();
        let data = item.read().unwrap();
        assert_eq!(
            text,
            data.get("text").unwrap().as_str().unwrap(),
            "Unicode content was corrupted"
        );
    }
}
