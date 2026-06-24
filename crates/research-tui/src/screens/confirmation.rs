//! "Launched" confirmation card displayed after detached spawn.

use crossterm::event::KeyEvent;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::palette;
use crate::spawn::SpawnInfo;
use crate::state::{Action, Screen, State};
use crate::theme;

/// Render the confirmation card.
pub fn render(state: &State, area: Rect, frame: &mut Frame<'_>) {
    let Some(info) = state.spawned.as_ref() else {
        return;
    };

    let card = centered(area, 70, 14);
    frame.render_widget(
        Block::default().style(Style::default().bg(palette::INK_2)),
        card,
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(palette::OCHRE))
        .style(Style::default().bg(palette::INK_2))
        .title(Span::styled(" launched ", theme::accent()));
    let inner = block.inner(card);
    frame.render_widget(block, card);

    let layout = Layout::vertical([Constraint::Length(1); 12]).split(inner);
    let mut row = 0usize;
    let mut put = |line: Line<'_>| {
        if row < layout.len() {
            frame.render_widget(Paragraph::new(line), layout[row]);
            row += 1;
        }
    };

    put(field("session", &short(&info.session_id)));
    put(field("topic", &info.topic));
    put(field(
        "provider",
        &format!("{} · {}", info.provider, info.processor),
    ));
    put(field("language", &info.language));
    put(Line::from(""));
    put(field("output", &display_path(&info.output_path)));
    put(field("log", &display_path(&info.log_path)));
    put(field(
        "pid",
        &if info.mocked {
            "—  (mock — no child spawned)".to_string()
        } else {
            format!("{}  (running in background)", info.pid)
        },
    ));
    put(Line::from(""));
    put(Line::from(Span::styled(
        "  press any key — start another",
        theme::body_dim(),
    )));

    let _ = render_info_consume(state, info);
}

fn field(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {:<10}", label), theme::body_dim()),
        Span::styled(value.to_string(), theme::body()),
    ])
}

fn short(id: &str) -> String {
    id.chars().take(8).collect()
}

fn display_path(p: &std::path::Path) -> String {
    let s = p.display().to_string();
    let max = 56;
    if s.chars().count() <= max {
        s
    } else {
        let chars: Vec<char> = s.chars().collect();
        let n = chars.len();
        let head: String = chars[..18].iter().collect();
        let tail: String = chars[n - 30..].iter().collect();
        format!("{}…{}", head, tail)
    }
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect {
        x,
        y,
        width: w,
        height: h,
    }
}

fn render_info_consume(_state: &State, _info: &SpawnInfo) -> bool {
    true
}

/// Any key returns to the input screen.
pub fn handle_key(_state: &mut State, _key: KeyEvent) -> Action {
    Action::Go(Screen::Input)
}
