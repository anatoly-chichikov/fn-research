//! Sessions list screen.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::sessions::{SessionRow, SessionStatus};
use crate::state::{Action, State};
use crate::theme;

/// Render the sessions screen.
pub fn render(state: &State, area: Rect, frame: &mut Frame<'_>) {
    let layout = Layout::vertical([Constraint::Length(2), Constraint::Min(0)]).split(area);

    frame.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            " sessions",
            theme::body().add_modifier(ratatui::style::Modifier::BOLD),
        )])),
        layout[0],
    );

    if state.sessions.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "  no runs yet — start one from the input screen.",
                theme::body_dim(),
            ))),
            layout[1],
        );
        return;
    }

    let row_layout = Layout::vertical(
        std::iter::repeat_n(Constraint::Length(1), state.sessions.len()).collect::<Vec<_>>(),
    )
    .split(layout[1]);

    for (i, row) in state.sessions.iter().enumerate() {
        let active = i == state.sessions_idx;
        let line = format_row(row, active);
        if let Some(rect) = row_layout.get(i) {
            frame.render_widget(Paragraph::new(line), *rect);
        }
    }
}

fn format_row(row: &SessionRow, active: bool) -> Line<'static> {
    let mark = if active { "▶ " } else { "  " };
    let mark_style = if active {
        theme::accent()
    } else {
        theme::body_dim()
    };
    let id_style = theme::accent();
    let title_style = if active {
        theme::accent()
    } else {
        theme::body()
    };
    let stat_style = match row.status {
        SessionStatus::Done => theme::ok(),
        SessionStatus::Running => theme::accent(),
        SessionStatus::Error => theme::err(),
    };
    let stat_label = match row.status {
        SessionStatus::Done => "✓ done",
        SessionStatus::Running => "· running",
        SessionStatus::Error => "✗ failed",
    };

    Line::from(vec![
        Span::styled(mark.to_string(), mark_style),
        Span::styled(format!("#{}  ", row.id_short), id_style),
        Span::styled(crate::screens::truncate(&row.topic, 50), title_style),
        Span::styled(format!("   {}  ", row.provider), theme::body_dim()),
        Span::styled(format!("{}  ", stat_label), stat_style),
        Span::styled(
            row.created.format("%d %b  %H:%M").to_string(),
            theme::body_dim(),
        ),
    ])
}

/// Handle a key on the sessions screen.
pub fn handle_key(state: &mut State, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::Back,
        KeyCode::Down | KeyCode::Char('j') => {
            if !state.sessions.is_empty() {
                state.sessions_idx = (state.sessions_idx + 1).min(state.sessions.len() - 1);
            }
            Action::Stay
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.sessions_idx = state.sessions_idx.saturating_sub(1);
            Action::Stay
        }
        _ => Action::Stay,
    }
}
