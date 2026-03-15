use std::path::{Path, PathBuf};

use super::generator::Generated;

/// Frame detection result.
pub struct DetectResult {
    /// Whether a frame was detected.
    pub frame: bool,
    /// Detection info.
    pub info: serde_json::Value,
}

/// Scan result for folder.
pub struct ScanResult {
    /// Total images scanned.
    pub total: usize,
    /// Images with frames.
    pub hits: usize,
    /// Per-file results.
    pub rows: Vec<ScanRow>,
}

/// Per-file scan result.
pub struct ScanRow {
    /// Image path.
    pub path: String,
    /// Whether frame was detected.
    pub frame: bool,
    /// Detection info.
    pub info: serde_json::Value,
}

/// Retry result.
pub struct RetryResult {
    /// Whether frame remains.
    pub frame: bool,
    /// Number of attempts made.
    pub tries: u32,
    /// Detection info.
    pub info: serde_json::Value,
}

/// Object that detects frames.
pub trait Framed {
    /// Detect frame in image path.
    fn detect(&self, path: &Path) -> DetectResult;
    /// Scan folder for cover images.
    fn scan(&self, root: &Path) -> ScanResult;
}

/// Return attempt path for backup.
fn attempt(path: &Path, step: u32) -> PathBuf {
    let file = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();
    let dot = file.rfind('.').unwrap_or(file.len());
    let stem = &file[..dot];
    let tail = &file[dot..];
    let name = format!("{}.attempt-{}{}", stem, step, tail);
    let root = path.parent().unwrap_or(Path::new("."));
    root.join(name)
}

/// Copy cover to attempt path.
fn backup(path: &Path, step: u32) -> Result<PathBuf, String> {
    let target = attempt(path, step);
    std::fs::copy(path, &target).map_err(|e| format!("Backup copy failed: {}", e))?;
    Ok(target)
}

/// Regenerate image until frame is gone or limit reached.
pub fn retry(
    gen: &dyn Generated,
    det: &dyn Framed,
    topic: &str,
    path: &Path,
    limit: u32,
) -> RetryResult {
    gen.generate(topic, path)
        .unwrap_or_else(|_| path.to_path_buf());
    let result = det.detect(path);
    if result.frame {
        let mut step = 1;
        loop {
            backup(path, step).ok();
            gen.generate(topic, path)
                .unwrap_or_else(|_| path.to_path_buf());
            let value = det.detect(path);
            if !value.frame {
                return RetryResult {
                    frame: false,
                    tries: step,
                    info: value.info,
                };
            }
            if step >= limit {
                return RetryResult {
                    frame: true,
                    tries: limit,
                    info: value.info,
                };
            }
            step += 1;
        }
    } else {
        RetryResult {
            frame: false,
            tries: 0,
            info: result.info,
        }
    }
}

/// Return cover image paths under root.
pub fn files(root: &Path, lead: &str, exts: &[&str]) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let walker = walkdir(root);
    for entry in walker {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        let dot = name.rfind('.').unwrap_or(name.len());
        let ext = &name[dot..];
        let regular = entry.file_type().map(|t| t.is_file()).unwrap_or(false);
        let prefixed = name.starts_with(lead);
        let accepted = exts.contains(&ext);
        if regular && prefixed && accepted {
            result.push(entry.path().to_path_buf());
        }
    }
    result.sort();
    result
}

/// Walk directory entries.
fn walkdir(root: &Path) -> Vec<std::fs::DirEntry> {
    let mut result = Vec::new();
    collect(root, &mut result);
    result
}

/// Collect directory entries recursively.
fn collect(dir: &Path, result: &mut Vec<std::fs::DirEntry>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                collect(&entry.path(), result);
            } else {
                result.push(entry);
            }
        }
    }
}

/// Default frame detection config.
pub fn config() -> serde_json::Value {
    serde_json::json!({
        "cap": 0.05,
        "min": 1,
        "std": 8.0,
        "diff": 12.0,
        "edge": 0.25,
        "noise": 0.05,
        "sides": 4,
        "tone": 15.0,
        "span": 1024.0,
        "sigma": 0.33,
        "floor": 0.0,
        "ridge": 0.35,
        "peak": 3.0,
        "band": 1,
        "lead": "cover",
        "exts": [".png", ".jpg", ".jpeg", ".webp"]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use research_domain::ids;

    struct FakeGenerator {
        data: Vec<u8>,
    }

    impl Generated for FakeGenerator {
        fn generate(&self, _topic: &str, path: &Path) -> Result<PathBuf, String> {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::write(path, &self.data).map_err(|e| format!("Write failed: {}", e))?;
            Ok(path.to_path_buf())
        }
    }

    struct FakeDetector {
        frames: std::sync::Mutex<Vec<bool>>,
    }

    impl Framed for FakeDetector {
        fn detect(&self, _path: &Path) -> DetectResult {
            let mut list = self.frames.lock().unwrap();
            let frame = if list.is_empty() {
                false
            } else {
                list.remove(0)
            };
            DetectResult {
                frame,
                info: serde_json::json!({}),
            }
        }

        fn scan(&self, _root: &Path) -> ScanResult {
            ScanResult {
                total: 0,
                hits: 0,
                rows: Vec::new(),
            }
        }
    }

    #[test]
    fn the_retry_stores_the_first_failed_attempt() {
        let mut rng = ids::ids(61021);
        let name = format!("attempt-{}", ids::digit(&mut rng, 10000));
        let dir = std::env::temp_dir().join(format!("retry-{}", ids::uuid(&mut rng)));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!("{}.jpg", name));
        let data: Vec<u8> = (0..64).map(|i| (i * 7) as u8).collect();
        let gen = FakeGenerator { data };
        let det = FakeDetector {
            frames: std::sync::Mutex::new(vec![true, false]),
        };
        retry(&gen, &det, "\u{0442}\u{0435}\u{043c}\u{0430}", &path, 4);
        let target = dir.join(format!("{}.attempt-1.jpg", name));
        assert!(target.exists(), "Attempt backup was not created");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn the_retry_stores_the_second_failed_attempt() {
        let mut rng = ids::ids(61022);
        let name = format!("attempt-{}", ids::digit(&mut rng, 10000));
        let dir = std::env::temp_dir().join(format!("retry-{}", ids::uuid(&mut rng)));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!("{}.jpg", name));
        let data: Vec<u8> = (0..64).map(|i| (i * 7) as u8).collect();
        let gen = FakeGenerator { data };
        let det = FakeDetector {
            frames: std::sync::Mutex::new(vec![true, true, false]),
        };
        retry(&gen, &det, "\u{0442}\u{0435}\u{043c}\u{0430}", &path, 4);
        let target = dir.join(format!("{}.attempt-2.jpg", name));
        assert!(target.exists(), "Second attempt backup was not created");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn the_retry_skips_backup_when_no_frame() {
        let mut rng = ids::ids(61023);
        let name = format!("attempt-{}", ids::digit(&mut rng, 10000));
        let dir = std::env::temp_dir().join(format!("retry-{}", ids::uuid(&mut rng)));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!("{}.jpg", name));
        let data: Vec<u8> = (0..64).map(|i| (i * 7) as u8).collect();
        let gen = FakeGenerator { data };
        let det = FakeDetector {
            frames: std::sync::Mutex::new(vec![false]),
        };
        retry(&gen, &det, "\u{0442}\u{0435}\u{043c}\u{0430}", &path, 4);
        let target = dir.join(format!("{}.attempt-1.jpg", name));
        assert!(!target.exists(), "Unexpected backup was created");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn the_retry_returns_tries_count() {
        let mut rng = ids::ids(61025);
        let name = format!("retry-{}", ids::digit(&mut rng, 10000));
        let dir = std::env::temp_dir().join(format!("retry-{}", ids::uuid(&mut rng)));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!("{}.jpg", name));
        let data: Vec<u8> = (0..64).map(|i| (i * 7) as u8).collect();
        let gen = FakeGenerator { data };
        let det = FakeDetector {
            frames: std::sync::Mutex::new(vec![true, true, false]),
        };
        let result = retry(&gen, &det, "\u{0442}\u{0435}\u{043c}\u{0430}", &path, 4);
        assert_eq!(
            2, result.tries,
            "Retry did not report correct attempt count"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn the_retry_stops_at_limit() {
        let mut rng = ids::ids(61027);
        let name = format!("limit-{}", ids::digit(&mut rng, 10000));
        let dir = std::env::temp_dir().join(format!("retry-{}", ids::uuid(&mut rng)));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!("{}.jpg", name));
        let data: Vec<u8> = (0..64).map(|i| (i * 7) as u8).collect();
        let gen = FakeGenerator { data };
        let det = FakeDetector {
            frames: std::sync::Mutex::new(vec![true, true, true, true, true]),
        };
        let result = retry(&gen, &det, "\u{0442}\u{0435}\u{043c}\u{0430}", &path, 3);
        assert!(result.frame, "Retry did not report frame at limit");
        std::fs::remove_dir_all(&dir).ok();
    }
}
