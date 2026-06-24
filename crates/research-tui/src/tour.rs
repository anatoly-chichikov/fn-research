//! Screenshot tour — render every screen state to a PNG. Stub for now;
//! step 9 of the plan implements the rasterizer.

pub mod fixtures;
pub mod rasterize;

use std::path::{Path, PathBuf};

use ratatui::backend::TestBackend;
use ratatui::Terminal;

use crate::app;
use crate::TourSummary;

/// Default rendered size — 156 columns × 44 rows.
pub const DEFAULT_COLS: u16 = 156;
/// Default rendered rows.
pub const DEFAULT_ROWS: u16 = 44;

/// Run the full tour into `out_dir`, returning paths of every PNG written.
pub fn run(out_dir: PathBuf) -> std::io::Result<TourSummary> {
    std::fs::create_dir_all(&out_dir)?;
    let mut shots = Vec::new();
    for shot in fixtures::all() {
        let path = out_dir.join(format!("{}.png", shot.name));
        let backend = TestBackend::new(DEFAULT_COLS, DEFAULT_ROWS);
        let mut terminal = Terminal::new(backend).expect("test backend");
        terminal
            .draw(|f| app::render(&shot.state, f))
            .expect("draw");
        let buffer = terminal.backend().buffer().clone();
        rasterize::render_buffer_to_png(&buffer, &path)?;
        shots.push(path);
    }
    Ok(TourSummary { out_dir, shots })
}

/// Capture a single state's buffer as text — used by snapshot tests.
pub fn capture_text(state: &crate::state::State, cols: u16, rows: u16) -> String {
    let backend = TestBackend::new(cols, rows);
    let mut terminal = Terminal::new(backend).expect("test backend");
    terminal.draw(|f| app::render(state, f)).expect("draw");
    let buffer = terminal.backend().buffer().clone();
    rasterize::buffer_to_text(&buffer)
}

/// Convenience for capturing into a chosen path (used by tests).
pub fn capture_png(state: &crate::state::State, path: &Path) -> std::io::Result<()> {
    let backend = TestBackend::new(DEFAULT_COLS, DEFAULT_ROWS);
    let mut terminal = Terminal::new(backend).expect("test backend");
    terminal.draw(|f| app::render(state, f)).expect("draw");
    let buffer = terminal.backend().buffer().clone();
    rasterize::render_buffer_to_png(&buffer, path)
}
