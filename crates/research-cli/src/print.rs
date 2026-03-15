use std::path::Path;

use research_domain::session::Sessioned;
use research_domain::task::Tasked;
use research_pdf::document::data;
use research_pdf::document::{self, Exported};
use research_pdf::palette;
use research_storage::organizer::{self, Organized};
use research_storage::repository::{self, Loadable};

/// List sessions.
pub fn enumerate(path: &Path) {
    let repo = repository::repo(path);
    let list = repo.load();
    if list.is_empty() {
        println!("No research sessions found");
        return;
    }
    for item in &list {
        let count = item.tasks().len();
        let code = &item.id()[..8.min(item.id().len())];
        let head = format!("[{}] {}", code, item.topic());
        let stamp = item.created().format("%Y-%m-%dT%H:%M:%S").to_string();
        println!("{}", head);
        println!("  Created: {}", stamp);
        println!("  Tasks: {}", count);
        println!();
    }
}

/// Show session details.
pub fn display(path: &Path, id: &str) {
    let repo = repository::repo(path);
    let list = repo.load();
    let pick = list.iter().find(|s| s.id().starts_with(id));
    match pick {
        Some(item) => {
            println!("Topic: {}", item.topic());
            println!("ID: {}", item.id());
            let stamp = item.created().format("%Y-%m-%dT%H:%M:%S").to_string();
            println!("Created: {}", stamp);
            println!("\nTasks ({}):", item.tasks().len());
            for task in item.tasks() {
                println!("\n  [{}] {}", task.status(), task.query());
                let (text, sources) = data::resultmap(path, item, task);
                if !text.is_empty() {
                    let bound = text
                        .char_indices()
                        .map(|(i, _)| i)
                        .take_while(|&i| i <= 100)
                        .last()
                        .unwrap_or(0);
                    let part = &text[..bound];
                    println!("  Summary: {} [truncated]", part);
                }
                if !sources.is_empty() {
                    println!("  Sources: {}", sources.len());
                }
            }
        }
        None => println!("Session not found: {}", id),
    }
}

/// Generate report for session.
pub fn render(path: &Path, out: &Path, id: &str, html: bool) {
    let repo = repository::repo(path);
    let list = repo.load();
    let pick = list.iter().find(|s| s.id().starts_with(id));
    match pick {
        Some(item) => {
            let provider = if !item.tasks().is_empty() {
                let last = item.tasks().last().unwrap();
                last.provider().to_string()
            } else {
                "parallel".to_string()
            };
            let org = organizer::organizer(out);
            let name = org.name(item.created(), item.topic(), item.id());
            let cover = org.existing(&name, &provider);
            let pal = palette::palette();
            let doc = document::Document::new(item, &pal, cover, out);
            let target = if html {
                org.html(&name, &provider)
            } else {
                org.report(&name, &provider)
            };
            if html {
                doc.page(&target);
                println!("HTML saved: {}", target.display());
            } else {
                match doc.save(&target) {
                    Ok(path) => println!("PDF saved: {}", path.display()),
                    Err(e) => println!("PDF generation failed: {}", e),
                }
            }
        }
        None => println!("Session not found: {}", id),
    }
}
