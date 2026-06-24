//! Text snapshot tests — every fixture from the tour gets a committed text
//! capture. To bless updates after intentional changes:
//!
//!     RESEARCH_TUI_BLESS=1 cargo test --package research-tui

use std::path::PathBuf;

use research_tui::tour::{capture_text, fixtures};

const COLS: u16 = 156;
const ROWS: u16 = 44;

fn snapshot_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
}

fn check(name: &str, text: &str) {
    let path = snapshot_dir().join(format!("{}.txt", name));
    let bless = std::env::var("RESEARCH_TUI_BLESS").is_ok();
    if bless || !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("snapshot dir");
        }
        std::fs::write(&path, text).expect("write snapshot");
        return;
    }
    let expected = std::fs::read_to_string(&path).expect("read snapshot");
    if expected.trim_end() != text.trim_end() {
        eprintln!("--- expected ({} bytes) ---\n{}", expected.len(), expected);
        eprintln!("--- actual ({} bytes) ---\n{}", text.len(), text);
        panic!("the {} snapshot doesnt match", name);
    }
}

#[test]
fn the_input_empty_snapshot_matches() {
    let s = fixtures::input_empty();
    check("01-input-empty", &capture_text(&s, COLS, ROWS));
}

#[test]
fn the_input_typed_snapshot_matches() {
    let s = fixtures::input_typed();
    check("02-input-typed", &capture_text(&s, COLS, ROWS));
}

#[test]
fn the_tune_default_snapshot_matches() {
    let s = fixtures::tune_default();
    check("04-tune-default", &capture_text(&s, COLS, ROWS));
}

#[test]
fn the_tune_stale_snapshot_matches() {
    let s = fixtures::tune_stale();
    check("05-tune-stale", &capture_text(&s, COLS, ROWS));
}

#[test]
fn the_tune_history_snapshot_matches() {
    let s = fixtures::tune_history();
    check("06-tune-history", &capture_text(&s, COLS, ROWS));
}

#[test]
fn the_approve_snapshot_matches() {
    let s = fixtures::approve_default();
    check("08-approve", &capture_text(&s, COLS, ROWS));
}

#[test]
fn the_confirmation_snapshot_matches() {
    let s = fixtures::confirmation_default();
    check("09-confirmation", &capture_text(&s, COLS, ROWS));
}

#[test]
fn the_sessions_with_rows_snapshot_matches() {
    let s = fixtures::sessions_with_rows();
    check("10-sessions-rows", &capture_text(&s, COLS, ROWS));
}

#[test]
fn the_sessions_empty_snapshot_matches() {
    let s = fixtures::sessions_empty();
    check("11-sessions-empty", &capture_text(&s, COLS, ROWS));
}

#[test]
fn the_keys_mixed_snapshot_matches() {
    let s = fixtures::keys_mixed();
    check("12-keys", &capture_text(&s, COLS, ROWS));
}

#[test]
fn the_help_overlay_snapshot_matches() {
    let s = fixtures::help_overlay();
    check("13-help", &capture_text(&s, COLS, ROWS));
}
