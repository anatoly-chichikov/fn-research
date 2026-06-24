//! Centered single-field edit overlay used by `e` on phase1/phase2.

use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::palette;
use crate::state::EditState;
use crate::theme;

/// Render the edit modal centered over `area`.
pub fn render(edit: &EditState, area: Rect, frame: &mut Frame<'_>) {
    frame.render_widget(Clear, area);
    frame.render_widget(
        Block::default().style(Style::default().bg(palette::INK_DEEP)),
        area,
    );
    let modal = centered(area, 78, 14);
    frame.render_widget(
        Block::default().style(Style::default().bg(palette::INK_2)),
        modal,
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(palette::OCHRE))
        .style(Style::default().bg(palette::INK_2))
        .title(Span::styled(
            match edit.kind {
                crate::state::EditKind::Root => " edit angle title ",
                crate::state::EditKind::Intent => " edit intent ",
                crate::state::EditKind::Topic => " edit topic ",
            },
            theme::accent(),
        ));
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    let body = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(edit.draft.clone(), theme::body()),
            Span::styled("█", theme::accent()),
        ]),
    ])
    .wrap(Wrap { trim: false })
    .alignment(Alignment::Left)
    .style(Style::default().bg(palette::INK_2));
    frame.render_widget(body, inner);
}

/// Centered subrect.
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
