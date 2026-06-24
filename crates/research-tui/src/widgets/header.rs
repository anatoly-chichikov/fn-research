//! Top header — the screen name underlined like an active tab (the app's own
//! mark, not an inverted block), a dim subtitle beside it, and the StateBadge
//! on the right. Same cells on every screen so the eye anchors, body breathes.

use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::state::{Screen, State};
use crate::theme;
use crate::widgets::badge;

/// Render header into `area`. Returns the body area below it.
pub fn render(state: &State, area: Rect, frame: &mut Frame<'_>) -> Rect {
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(area);

    let (name, subtitle) = title(state);
    let cols = Layout::horizontal([Constraint::Min(20), Constraint::Length(26)]).split(chunks[0]);
    let mut left = vec![Span::styled(name, theme::accent_bold())];
    if !subtitle.is_empty() {
        left.push(Span::raw("   "));
        left.push(Span::styled(subtitle, theme::body_dim()));
    }
    frame.render_widget(Paragraph::new(Line::from(left)), cols[0]);
    frame.render_widget(
        Paragraph::new(right_line(state)).alignment(Alignment::Right),
        cols[1],
    );
    underline(frame, area.x, chunks[1].y, name);

    chunks[2]
}

/// Paint the short accent underline under a tab word.
pub fn underline(frame: &mut Frame<'_>, x: u16, y: u16, word: &str) {
    let len = word.chars().count().max(1);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "\u{2500}".repeat(len),
            theme::accent(),
        ))),
        Rect {
            x,
            y,
            width: len as u16,
            height: 1,
        },
    );
}

/// The screen name (lowercase tab) + a dim subtitle, per screen.
fn title(state: &State) -> (&'static str, &'static str) {
    match state.screen {
        Screen::Input => ("new research", "what do you want to know?"),
        Screen::Generating => ("thinking", "reading your topic\u{2026}"),
        Screen::Tune => ("tune", "set the depth of each angle, then regenerate"),
        Screen::Approve => ("approve", "final review before launch"),
        Screen::Confirmation => ("launched", "the research is running"),
        Screen::Sessions => ("sessions", "past research runs"),
        Screen::Keys => ("keys", "api keys in the environment"),
    }
}

/// The badge on brief screens, else nothing.
fn right_line(state: &State) -> Line<'static> {
    let on_brief =
        matches!(state.screen, Screen::Tune | Screen::Approve) && !state.iterations.is_empty();
    if on_brief {
        badge::line(state)
    } else {
        Line::from("")
    }
}
