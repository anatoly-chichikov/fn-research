//! Sessions screen data — projection of the on-disk repository.

use std::path::{Path, PathBuf};

use chrono::NaiveDateTime;
use research_domain::session::Sessioned;
use research_storage::repository::Loadable;

/// One row on the sessions screen.
#[derive(Clone, Debug)]
pub struct SessionRow {
    pub id_short: String,
    pub topic: String,
    pub provider: String,
    pub status: SessionStatus,
    pub created: NaiveDateTime,
    pub pdf: Option<PathBuf>,
}

/// Status of a session — collapsed from per-task states.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionStatus {
    Done,
    Running,
    Error,
}

/// Read sessions for the sessions screen, sorted newest first.
pub fn load(out: &Path) -> Vec<SessionRow> {
    if !out.exists() {
        return Vec::new();
    }
    let repo = research_storage::repository::repo(out);
    let mut rows: Vec<SessionRow> = repo.load().iter().map(|s| project(out, s)).collect();
    rows.sort_by_key(|row| std::cmp::Reverse(row.created));
    rows
}

/// Build a row from a session, without touching the filesystem (no folder
/// creation — we predict paths and only check existence).
fn project(out: &Path, s: &research_domain::session::ResearchSession) -> SessionRow {
    let status = if s.pending().is_some() {
        SessionStatus::Running
    } else {
        s.tasks()
            .iter()
            .find_map(|t| {
                use research_domain::task::Tasked;
                let v = t.status();
                if v == "completed" {
                    Some(SessionStatus::Done)
                } else if v.contains("error") || v.contains("failed") {
                    Some(SessionStatus::Error)
                } else {
                    None
                }
            })
            .unwrap_or(SessionStatus::Done)
    };
    let provider = format!("{} · {}", s.provider(), s.processor());
    let id = Sessioned::id(s);
    let id_short = id.chars().take(8).collect();
    let pdf = predict_pdf(out, s);
    SessionRow {
        id_short,
        topic: s.topic().to_string(),
        provider,
        status,
        created: *s.created(),
        pdf,
    }
}

/// Compute the expected PDF path for a session without creating directories.
fn predict_pdf(out: &Path, s: &research_domain::session::ResearchSession) -> Option<PathBuf> {
    use research_storage::organizer::{organizer, Organized};
    let org = organizer(out);
    let folder = Organized::name(&org, s.created(), s.topic(), Sessioned::id(s));
    let provider = s.provider().to_string();
    let pdf = out
        .join(&folder)
        .join(format!("{}-{}.pdf", folder, provider));
    if pdf.exists() {
        Some(pdf)
    } else {
        None
    }
}
