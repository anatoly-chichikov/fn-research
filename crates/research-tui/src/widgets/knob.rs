//! KnobSlider — one depth/novelty/applied dial: `NAME «bar» N/5 •`.
//!
//! Arrows are always live, so a FOCUSED knob hugs its bar in `«…»` (push me
//! ←/→) and the caller paints a faint band over the dial field only. A CHANGED
//! knob (pending ≠ committed) turns the bar, value and a trailing `•` gold. The
//! `N/5` digit stays the authority; fill and colour reinforce it.

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

use crate::palette;
use crate::state::Prop;
use crate::theme;
use crate::widgets::slider;

/// Fixed dial width so nothing shifts horizontally between states.
pub const WIDTH: u16 = 31;

/// One dial line, right-aligned by the caller into a [`WIDTH`]-wide field.
pub fn line(
    prop: Prop,
    value: u8,
    focused: bool,
    changed: bool,
    locked: bool,
    bg: Option<Color>,
) -> Line<'static> {
    let tint = |s: Style| match bg {
        Some(c) => s.bg(c),
        None => s,
    };
    let name_style = if locked {
        tint(Style::default().fg(palette::TRACK))
    } else if focused {
        tint(theme::accent_bold())
    } else {
        tint(theme::body_dim())
    };
    let fill = if locked {
        Style::default().fg(palette::TRACK)
    } else if changed {
        theme::gold()
    } else if focused {
        theme::body()
    } else {
        theme::structure()
    };
    let fill = tint(fill);
    let chev = focused && !locked;
    let lb = if chev { "\u{00ab}" } else { " " };
    let rb = if chev { "\u{00bb}" } else { " " };
    let value_style = if locked {
        tint(Style::default().fg(palette::TRACK))
    } else if changed {
        tint(theme::gold())
    } else {
        tint(theme::body())
    };
    let dot = if changed && !locked {
        Span::styled(" \u{2022}", tint(theme::gold()))
    } else {
        Span::styled("  ", tint(theme::body_dim()))
    };
    let mut spans = vec![
        Span::styled(format!("{:<8}", prop.caps()), name_style),
        Span::styled(lb.to_string(), tint(theme::accent())),
    ];
    for s in slider::spans(value, fill) {
        let style = match bg {
            Some(c) => s.style.bg(c),
            None => s.style,
        };
        spans.push(Span {
            content: s.content,
            style,
        });
    }
    spans.push(Span::styled(rb.to_string(), tint(theme::accent())));
    spans.push(Span::styled(format!(" {value}/5"), value_style));
    spans.push(dot);
    Line::from(spans)
}
