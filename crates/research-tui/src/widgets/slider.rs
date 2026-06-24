//! The 15-cell block bar — a filled run over a dim track, all `█` so the
//! filled and empty cells share one width class (no `▰▱` tofu, no shear).
//!
//! The fill boundary is the value; the printed `N/5` beside it is the
//! authoritative signal, so color and length are reinforcement only.

use ratatui::style::Style;
use ratatui::text::Span;

use crate::palette;

/// Total bar width in cells — 3 per level so `1→███ … 5→███████████████`.
pub const WIDTH: usize = 15;

/// Build the bar as two spans: a filled run in `fill`, the remainder a dim track.
pub fn spans(value: u8, fill: Style) -> Vec<Span<'static>> {
    let filled = (value.clamp(1, 5) as usize) * 3;
    let filled = filled.min(WIDTH);
    vec![
        Span::styled("\u{2588}".repeat(filled), fill),
        Span::styled(
            "\u{2588}".repeat(WIDTH - filled),
            Style::default().fg(palette::TRACK),
        ),
    ]
}

/// One block glyph whose height encodes a 1..5 knob value — used by the
/// iteration sparkline fingerprint.
pub fn spark(value: u8) -> &'static str {
    match value.clamp(1, 5) {
        1 => "\u{2581}", // ▁
        2 => "\u{2583}", // ▃
        3 => "\u{2584}", // ▄
        4 => "\u{2586}", // ▆
        _ => "\u{2588}", // █
    }
}
