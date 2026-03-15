use std::path::Path;

use research_domain::session;
use research_domain::task;
use research_storage::repository::{self, Mutable};

/// Create session and return short identifier.
pub fn seed(
    data: &Path,
    topic: &str,
    query: &str,
    processor: &str,
    language: &str,
    provider: &str,
) -> String {
    let repo = repository::repo(data);
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Local::now().naive_local();
    let stamp = task::format(&now);
    let value = serde_json::json!({
        "id": id,
        "topic": topic,
        "tasks": [],
        "created": stamp,
        "query": query,
        "processor": processor,
        "language": language,
        "provider": provider,
    });
    let item = session::session(&value);
    repo.append(item);
    println!("Created session: {}", &id[..8]);
    println!("Topic: {}", topic);
    id[..8].to_string()
}
