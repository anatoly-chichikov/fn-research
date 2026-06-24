//! Single-spinner "thinking through your topic..." screen.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::state::{Action, Screen, State};
use crate::theme;

const FRAMES: &[&str] = &["▖", "▘", "▝", "▗"];

/// Render the spinner.
pub fn render(state: &State, area: Rect, frame: &mut Frame<'_>) {
    let layout = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(area);

    let f = FRAMES[(state.spinner_tick as usize) % FRAMES.len()];
    let body = if let Some(err) = state.error.as_ref() {
        Line::from(vec![Span::styled(format!("error: {err}"), theme::err())])
    } else {
        Line::from(vec![
            Span::styled(format!("{}  ", f), theme::accent()),
            Span::styled("thinking through your topic…", theme::body_dim()),
        ])
    };
    let p = Paragraph::new(body).alignment(Alignment::Center);
    frame.render_widget(p, layout[1]);
}

/// Cancel returns to input.
pub fn handle_key(_state: &mut State, key: KeyEvent) -> Action {
    if matches!(key.code, KeyCode::Esc) {
        Action::Go(Screen::Input)
    } else {
        Action::Stay
    }
}
