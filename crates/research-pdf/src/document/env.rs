use std::path::Path;
use std::process::Command;

use research_domain::task::{ResearchRun, Tasked};

/// Return environment value by key.
pub fn env(key: &str) -> String {
    std::env::var(key).unwrap_or_default()
}

/// Render PDF using WeasyPrint.
pub fn emit(html: &str, path: &Path) -> Result<std::path::PathBuf, String> {
    let tmp = tempfile::Builder::new()
        .prefix("report")
        .suffix(".html")
        .tempfile()
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    std::fs::write(tmp.path(), html.as_bytes())
        .map_err(|e| format!("Failed to write temp file: {}", e))?;
    let mut vars: std::collections::HashMap<String, String> = std::env::vars().collect();
    let home = vars
        .get("DYLD_FALLBACK_LIBRARY_PATH")
        .cloned()
        .unwrap_or_default();
    let list: Vec<&str> = [home.as_str(), "/opt/homebrew/lib", "/usr/local/lib"]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect();
    let link = list.join(":");
    vars.insert("DYLD_FALLBACK_LIBRARY_PATH".to_string(), link);
    let result = Command::new("uv")
        .args([
            "run",
            "--with",
            "weasyprint",
            "python",
            "-m",
            "weasyprint",
            &tmp.path().display().to_string(),
            &path.display().to_string(),
        ])
        .envs(&vars)
        .output()
        .map_err(|e| format!("Failed to run WeasyPrint: {}", e))?;
    if result.status.success() {
        Ok(path.to_path_buf())
    } else {
        Err(format!(
            "WeasyPrint failed with code {}",
            result.status.code().unwrap_or(-1)
        ))
    }
}

/// Return report author from env.
pub fn author() -> String {
    env("REPORT_FOR")
}

/// Return service name from latest task.
pub fn service(tasks: &[ResearchRun]) -> String {
    tasks
        .last()
        .map(|t| t.provider().to_string())
        .unwrap_or_else(|| "parallel.ai".to_string())
}

/// Return provider slug from task service.
pub fn provider(task: &ResearchRun) -> String {
    let name = task.provider();
    if name == "x.ai" {
        "xai".to_string()
    } else if name.ends_with(".ai") {
        name.split('.').next().unwrap_or(name).to_string()
    } else {
        name.to_string()
    }
}
