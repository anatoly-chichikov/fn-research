use std::collections::HashMap;
use std::path::Path;

use research_domain::session::Sessioned;
use research_storage::repository::{self, Loadable, Mutable};

use crate::execute::{self, Config};
use crate::seed;

/// Create session and run research.
#[allow(clippy::too_many_arguments)]
pub fn launch(
    root: &Path,
    data: &Path,
    out: &Path,
    topic: &str,
    query: &str,
    processor: &str,
    language: &str,
    provider: &str,
    conf: &Config,
) -> Result<(), String> {
    let processor = if provider == "xai" && processor == "year" {
        "social".to_string()
    } else {
        processor.to_string()
    };
    if processor == "lite" {
        return Err("Run failed because processor lite is not supported".to_string());
    }
    if provider == "xai" && processor != "social" && processor != "full" {
        return Err("Run failed because processor must be social or full for xai".to_string());
    }
    let modes = ["fast", "standard", "heavy"];
    if provider == "valyu" && !modes.contains(&processor.as_str()) {
        return Err("Run failed because processor is not supported for valyu".to_string());
    }
    let mode = if modes.contains(&processor.as_str()) {
        processor.clone()
    } else {
        "standard".to_string()
    };
    let pairs: Vec<(String, String)> = if provider == "all" {
        vec![
            ("parallel".to_string(), processor.clone()),
            ("valyu".to_string(), mode),
        ]
    } else {
        let proc = if provider == "valyu" {
            mode
        } else {
            processor.clone()
        };
        vec![(provider.to_string(), proc)]
    };
    let first = &pairs[0];
    let id = seed::seed(data, topic, query, &first.1, language, &first.0);
    execute::execute(root, data, out, &id, conf);
    if pairs.len() > 1 {
        let second = &pairs[1];
        let repo = repository::repo(data);
        let list = repo.load();
        let pick = list.iter().find(|s| s.id().starts_with(&id));
        if let Some(item) = pick {
            let mut opts = HashMap::new();
            opts.insert("provider".to_string(), second.0.clone());
            opts.insert("processor".to_string(), second.1.clone());
            let updated = item.reconfigure(&opts);
            repo.update(updated);
            execute::execute(root, data, out, &id, conf);
        }
    }
    Ok(())
}
