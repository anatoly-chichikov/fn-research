//! Read-only API key inventory.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::keys::KeyRow;
use crate::state::{Action, State};
use crate::theme;

/// Render the keys screen.
pub fn render(state: &State, area: Rect, frame: &mut Frame<'_>) {
    let layout = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(3),
        Constraint::Min(0),
    ])
    .split(area);

    frame.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            " connections",
            theme::body().add_modifier(ratatui::style::Modifier::BOLD),
        )])),
        layout[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            " keys are read from your shell environment. set or unset them outside the TUI.",
            theme::body_dim(),
        )]))
        .wrap(Wrap { trim: true }),
        layout[1],
    );

    let row_layout = Layout::vertical(
        std::iter::repeat_n(Constraint::Length(1), state.keys.len()).collect::<Vec<_>>(),
    )
    .split(layout[2]);

    for (i, row) in state.keys.iter().enumerate() {
        if let Some(rect) = row_layout.get(i) {
            frame.render_widget(Paragraph::new(format_row(row)), *rect);
        }
    }
}

fn format_row(row: &KeyRow) -> Line<'static> {
    let req_marker = if row.required {
        Span::styled(" *  ", theme::err())
    } else {
        Span::styled(" opt", theme::body_dim())
    };
    let value_span = if row.set {
        Span::styled(format!("  {}", row.preview), theme::body())
    } else {
        Span::styled("  not set", theme::body_dim())
    };
    let stat_style = if row.set {
        theme::ok()
    } else if row.required {
        theme::err()
    } else {
        theme::body_dim()
    };
    let stat = if row.set {
        "✓ active"
    } else if row.required {
        "required"
    } else {
        "— optional"
    };

    Line::from(vec![
        Span::styled(format!(" {:<18}", row.name), theme::body()),
        req_marker,
        value_span,
        Span::styled("    ", theme::body_dim()),
        Span::styled(stat.to_string(), stat_style),
    ])
}

/// Handle a key event on the keys screen.
pub fn handle_key(_state: &mut State, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::Back,
        _ => Action::Stay,
    }
}
