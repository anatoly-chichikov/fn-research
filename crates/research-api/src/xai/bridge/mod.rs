pub mod collect;
pub mod fetch;

use std::path::Path;

/// Object that can run XAI python runner.
pub trait Bound {
    /// Run XAI and return response map.
    fn run(&self, text: &str, pack: &serde_json::Value) -> serde_json::Value;
}

/// Placeholder bound that returns empty result.
pub struct NullBound;

impl Bound for NullBound {
    fn run(&self, _text: &str, _pack: &serde_json::Value) -> serde_json::Value {
        serde_json::json!({})
    }
}

/// Return python executable path.
pub fn binary(root: &Path, exec: &str) -> String {
    let venv = root.join(".venv").join("bin").join("python");
    if exec.is_empty() {
        if venv.exists() {
            venv.to_string_lossy().to_string()
        } else {
            "python3".to_string()
        }
    } else {
        exec.to_string()
    }
}
