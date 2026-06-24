//! StateBadge — the brief lifecycle in one fixed header cell. Word-first, so
//! the state survives monochrome; color and the edit count are reinforcement.

use ratatui::text::{Line, Span};

use crate::state::{BriefState, State};
use crate::theme;

/// Build the right-aligned badge line for the header.
pub fn line(state: &State) -> Line<'static> {
    match state.brief_state() {
        BriefState::InSync => Line::from(vec![Span::styled("IN SYNC", theme::ok())]),
        BriefState::Stale(n) => Line::from(vec![
            Span::styled("STALE", theme::gold()),
            Span::styled(
                format!("  \u{00b7} {n} EDIT{}", if n == 1 { "" } else { "S" }),
                theme::body_dim(),
            ),
        ]),
        BriefState::Regenerating => {
            let frames = [
                "\u{2583}", "\u{2585}", "\u{2586}", "\u{2588}", "\u{2586}", "\u{2585}",
            ];
            let f = frames[(state.spinner_tick as usize) % frames.len()];
            Line::from(vec![
                Span::styled("REGENERATING ", theme::wave()),
                Span::styled(f.to_string(), theme::wave()),
            ])
        }
    }
}
