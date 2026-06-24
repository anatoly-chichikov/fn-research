//! Named text styles built on the palette.
//!
//! The TUI never reaches into the palette directly — it pulls a named style
//! that corresponds to a semantic role. This keeps the visual language
//! consistent across screens. Color is reinforcement only: every state also
//! carries a word and a number, so the design survives monochrome.

use ratatui::style::{Modifier, Style};

use crate::palette;

/// Primary body text — titles, active question, slider value.
pub fn body() -> Style {
    Style::default().fg(palette::TEXT)
}

/// Secondary body text — prose, sub-angles, summaries.
pub fn body_soft() -> Style {
    Style::default().fg(palette::TEXT_SOFT)
}

/// Tertiary body text — labels, hints, anchor words, timestamps.
pub fn body_dim() -> Style {
    Style::default().fg(palette::TEXT_DIM)
}

/// A CAPS label / kicker eyebrow — same color as dim text, meaning comes from case.
pub fn label() -> Style {
    Style::default().fg(palette::TEXT_DIM)
}

/// Decorative hairline rule — never carries text.
pub fn rule() -> Style {
    Style::default().fg(palette::BORDER_DIM)
}

/// Crisp structural rule — the header baseline / focused frame.
pub fn rule_strong() -> Style {
    Style::default().fg(palette::BORDER)
}

/// Dim chrome — borders, separators.
pub fn chrome() -> Style {
    Style::default().fg(palette::BORDER_DIM)
}

/// The one warm voice — what is active. Used sparingly.
pub fn accent() -> Style {
    Style::default().fg(palette::ACCENT)
}

/// Bold accent — the brand mark, a focused angle title.
pub fn accent_bold() -> Style {
    Style::default()
        .fg(palette::ACCENT)
        .add_modifier(Modifier::BOLD)
}

/// Bold accent for the brand mark.
pub fn brand() -> Style {
    accent_bold()
}

/// Dark caps printed ON a filled terracotta chip.
pub fn on_accent() -> Style {
    Style::default()
        .fg(palette::CHIP_INK)
        .bg(palette::ACCENT_SOFT)
        .add_modifier(Modifier::BOLD)
}

/// Dark caps printed ON a filled gold chip.
pub fn on_gold() -> Style {
    Style::default()
        .fg(palette::CHIP_INK)
        .bg(palette::GOLD)
        .add_modifier(Modifier::BOLD)
}

/// The cool structural voice — sub-angle markers, clean slider fill, sparkline.
pub fn structure() -> Style {
    Style::default().fg(palette::STRUCTURE)
}

/// Wave foam — the regenerating spinner + IN SYNC. Transient only.
pub fn wave() -> Style {
    Style::default().fg(palette::WAVE)
}

/// Deprecated alias — markers used to be wave-soft; now structure.
pub fn wave_soft() -> Style {
    structure()
}

/// The single "changed / stale" signal.
pub fn gold() -> Style {
    Style::default().fg(palette::GOLD)
}

/// The empty slider remainder — a dim solid block.
pub fn track() -> Style {
    Style::default().fg(palette::TRACK)
}

/// The single brightest element — the keyboard caret (a painted cell).
pub fn caret() -> Style {
    Style::default().fg(palette::GROUND).bg(palette::FOAM)
}

/// Focus selection band — full-width row background with primary cream text.
pub fn band() -> Style {
    Style::default().fg(palette::TEXT).bg(palette::BAND)
}

/// Deprecated alias for [`band`] — the old reverse-video selection style.
pub fn selection() -> Style {
    band()
}

/// Dim text on the focus band (for metadata on a selected row).
pub fn band_dim() -> Style {
    Style::default().fg(palette::TEXT_DIM).bg(palette::BAND)
}

/// Status: in sync / completed (cool, calm).
pub fn ok() -> Style {
    Style::default().fg(palette::WAVE)
}

/// Status: error.
pub fn err() -> Style {
    Style::default().fg(palette::DANGER)
}

/// kicker label — caps + accent (an eyebrow that should read as the one warm note).
pub fn kicker() -> Style {
    Style::default().fg(palette::ACCENT)
}
