//! Interactive console interface for the research binary.
//!
//! Running `research` with no subcommand opens this TUI. The TUI walks the user
//! from a topic input through brief generation (one Gemini call), two phases of
//! interactive review, an approve step, and finally a detached spawn of the
//! headless `research run` flow that produces the PDF.

pub mod app;
pub mod brief;
pub mod events;
pub mod keys;
pub mod language;
pub mod options;
pub mod palette;
pub mod screens;
pub mod sessions;
pub mod spawn;
pub mod state;
pub mod theme;
pub mod tour;
pub mod widgets;

use std::io;

/// Runtime options for the TUI entry point.
pub struct RunOptions {
    /// When true, skip Gemini and use a sample brief; spawn becomes a no-op.
    pub mock: bool,
    /// Workspace root (used for output dir and detached spawn cwd).
    pub root: std::path::PathBuf,
}

/// Launch the live TUI. Blocks until the user quits.
pub fn run(opts: RunOptions) -> Result<(), String> {
    app::run(opts).map_err(|e| e.to_string())
}

/// Result of the screenshot tour.
pub struct TourSummary {
    pub out_dir: std::path::PathBuf,
    pub shots: Vec<std::path::PathBuf>,
}

/// Render every screen state to a PNG under `out_dir`.
pub fn tour(out_dir: std::path::PathBuf) -> Result<TourSummary, String> {
    tour::run(out_dir).map_err(|e: io::Error| e.to_string())
}
