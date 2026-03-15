use std::path::Path;

use research_domain::processor;
use research_domain::provider::Provider;

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
    provider: &Provider,
    conf: &Config,
) -> Result<(), String> {
    let proc =
        processor::resolve(processor, provider).map_err(|e| format!("Run failed because {}", e))?;
    let id = seed::seed(
        data,
        topic,
        query,
        &proc.to_string(),
        language,
        &provider.to_string(),
    );
    execute::execute(root, data, out, &id, conf);
    Ok(())
}
