//! Hokusai palette — a near-monochrome blue field, a three-step cream ladder,
//! one warm terracotta, and a single gold "changed" signal.
//!
//! Every value is machine-verified for contrast (`tmp_design/contrast_check.py`):
//! the reading tiers clear WCAG AA with headroom and the accents clear the 3:1
//! large/UI floor. Colors are `Color::Rgb` so the rasterizer paints them
//! directly. The TUI never reaches in here — it pulls a named [`crate::theme`]
//! style. Requires a truecolor terminal (`COLORTERM=truecolor`).

use ratatui::style::Color;

// ---- Grounds (separate by lightness; one hairline allowed on raised cards) ----

/// Default background — a desaturated deep Prussian / bero-ai. Cream reads 13.9:1.
pub const GROUND: Color = Color::Rgb(0x0F, 0x1D, 0x33);
/// Raised layer — collapsed iteration bars, modal frame, peek card.
pub const SURFACE: Color = Color::Rgb(0x20, 0x32, 0x4F);
/// Deepest well — generating log block, modal scrim.
pub const INSET: Color = Color::Rgb(0x07, 0x0E, 0x1C);
/// Focus selection band — a dark indigo that still carries cream/dim/state text.
pub const BAND: Color = Color::Rgb(0x1C, 0x30, 0x48);

// ---- Cream ladder (the "ink"; three luminance steps carry text rank) ----

/// Primary text — titles, active question, slider value `N/5`. 13.9:1.
pub const TEXT: Color = Color::Rgb(0xF2, 0xE8, 0xD0);
/// Secondary text — intent paragraph, sub-angle prose, summaries. 11.5:1.
pub const TEXT_SOFT: Color = Color::Rgb(0xE0, 0xD4, 0xB6);
/// Tertiary text — CAPS labels, knob names, hints, timestamps, anchor words. 8.9:1.
pub const TEXT_DIM: Color = Color::Rgb(0xC8, 0xBC, 0x9A);
/// Foam crest — the single brightest element, used once per screen for the caret.
pub const FOAM: Color = Color::Rgb(0xFF, 0xFE, 0xF7);

// ---- The one warm voice ----

/// Terracotta — brand, focused angle title, focus rail, BASE, active index. 5.9:1.
pub const ACCENT: Color = Color::Rgb(0xD0, 0x86, 0x65);
/// Terracotta fill — chip background; carries dark-ink caps, never cream. 4.1:1.
pub const ACCENT_SOFT: Color = Color::Rgb(0xB5, 0x69, 0x4F);
/// Dark ink for caps printed ON a filled chip. 4.5:1 on ACCENT_SOFT.
pub const CHIP_INK: Color = Color::Rgb(0x0A, 0x14, 0x26);

// ---- The cool structural voice (belongs to the field; recedes) ----

/// Permanent cool structure — sub-angle markers, clean slider fill, sparkline, borders. 4.7:1.
pub const STRUCTURE: Color = Color::Rgb(0x6E, 0x89, 0xA8);
/// Wave foam — reserved for the transient regenerating spinner + IN SYNC. 7.5:1.
pub const WAVE: Color = Color::Rgb(0x7F, 0xB4, 0xCA);

// ---- Transient signals ----

/// Gold — the single "changed / stale" signal: STALE badge, dirty knob fill + value. 7.9:1.
pub const GOLD: Color = Color::Rgb(0xD9, 0xA8, 0x6A);
/// Muted vermilion / shu — errors only, bold/large. 5.0:1.
pub const DANGER: Color = Color::Rgb(0xDE, 0x6A, 0x48);

// ---- Chrome ----

/// Empty slider remainder — a dim solid block, recessive against the ground.
pub const TRACK: Color = Color::Rgb(0x30, 0x43, 0x5F);
/// Hairline rules / inactive rails. Decorative, never carries text.
pub const BORDER_DIM: Color = Color::Rgb(0x3A, 0x4A, 0x66);
/// Crisp rule — header baseline, focused frame. 4.7:1 (clears the UI floor).
pub const BORDER: Color = Color::Rgb(0x6E, 0x89, 0xA8);

/// Default cell foreground used by the rasterizer.
pub const DEFAULT_FG: Color = TEXT;
/// Default cell background used by the rasterizer.
pub const DEFAULT_BG: Color = GROUND;

// ---- Back-compat aliases (old constant names → nearest new role) ----
// Kept so the rasterizer's ANSI fallbacks and not-yet-rewritten screens keep
// compiling and pick up the new palette automatically.

/// Deprecated alias for [`GROUND`].
pub const INK: Color = GROUND;
/// Deprecated alias for [`SURFACE`].
pub const INK_2: Color = SURFACE;
/// Deprecated alias for [`INSET`].
pub const INK_DEEP: Color = INSET;
/// Deprecated alias for [`TEXT`].
pub const PAPER: Color = TEXT;
/// Deprecated alias for [`TEXT_SOFT`].
pub const PAPER_SOFT: Color = TEXT_SOFT;
/// Deprecated alias for [`TEXT_DIM`].
pub const PAPER_DIM: Color = TEXT_DIM;
/// Deprecated alias for [`ACCENT`].
pub const OCHRE: Color = ACCENT;
/// Deprecated alias for [`WAVE`].
pub const WAVE_SOFT: Color = STRUCTURE;
/// Deprecated alias for [`BAND`].
pub const WAVE_DEEP: Color = BAND;
/// Deprecated alias for [`BORDER`].
pub const MUTED: Color = BORDER;
/// Deprecated alias for [`BORDER_DIM`].
pub const MUTED_2: Color = BORDER_DIM;
/// Deprecated alias for [`WAVE`] (green dropped from the palette).
pub const SUCCESS: Color = WAVE;
