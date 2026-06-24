//! Approve — read-only final review of the active iteration before handing
//! off to the headless engine. The 3×3 question tree in the reading column,
//! knobs hidden, intent at the top. Quiet and scannable.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::brief::TuiBrief;
use crate::state::{Action, State};
use crate::theme;

/// Render the approve screen.
pub fn render(state: &State, area: Rect, frame: &mut Frame<'_>) {
    let Some(brief) = state.base() else {
        return;
    };
    let mut y = area.y;
    put(
        frame,
        area.x,
        y,
        area.width,
        Line::from(Span::styled("READY TO LAUNCH", theme::kicker())),
    );
    y = y.saturating_add(1);
    put(
        frame,
        area.x,
        y,
        area.width,
        Line::from(Span::styled(
            brief.topic.clone(),
            theme::body().add_modifier(ratatui::style::Modifier::BOLD),
        )),
    );
    y = y.saturating_add(2);

    if let Some(err) = state.error.as_ref() {
        put(
            frame,
            area.x,
            y,
            area.width,
            Line::from(Span::styled(
                format!("launch failed \u{2014} {err}"),
                theme::err(),
            )),
        );
        y = y.saturating_add(2);
    }

    let intent_h = wrapped_height(area.width, &brief.intent).min(area.height.saturating_sub(2));
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            brief.intent.clone(),
            theme::body_dim(),
        )))
        .wrap(Wrap { trim: true }),
        Rect {
            x: area.x,
            y,
            width: area.width,
            height: intent_h,
        },
    );
    y = y.saturating_add(intent_h).saturating_add(1);
    y = rule(area, y, frame);

    for i in 0..brief.topics.len() {
        if y >= area.bottom() {
            break;
        }
        y = render_root(brief, i, area, y, frame);
    }
}

fn render_root(brief: &TuiBrief, i: usize, area: Rect, mut y: u16, frame: &mut Frame<'_>) -> u16 {
    let root = &brief.topics[i];
    let title_w = area.width.saturating_sub(4);
    let title_h = wrapped_height(title_w, &root.title);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!("{}", i + 1),
            theme::accent_bold(),
        ))),
        Rect {
            x: area.x,
            y,
            width: 3,
            height: 1,
        },
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            root.title.clone(),
            theme::body().add_modifier(ratatui::style::Modifier::BOLD),
        )))
        .wrap(Wrap { trim: true }),
        Rect {
            x: area.x.saturating_add(4),
            y,
            width: title_w,
            height: title_h,
        },
    );
    y = y.saturating_add(title_h).saturating_add(1);
    for (j, sub) in root.subs.iter().enumerate() {
        if y >= area.bottom() {
            return y;
        }
        let sub_w = area.width.saturating_sub(8);
        let sub_h = wrapped_height(sub_w, &sub.title);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("{}.{}", i + 1, j + 1),
                theme::structure(),
            ))),
            Rect {
                x: area.x.saturating_add(4),
                y,
                width: 4,
                height: 1,
            },
        );
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                sub.title.clone(),
                theme::body_soft(),
            )))
            .wrap(Wrap { trim: true }),
            Rect {
                x: area.x.saturating_add(8),
                y,
                width: sub_w,
                height: sub_h,
            },
        );
        y = y.saturating_add(sub_h);
    }
    y.saturating_add(1)
}

fn rule(area: Rect, y: u16, frame: &mut Frame<'_>) -> u16 {
    if y >= area.bottom() {
        return y;
    }
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "\u{2500}".repeat(area.width as usize),
            theme::rule(),
        )))
        .alignment(Alignment::Left),
        Rect {
            x: area.x,
            y,
            width: area.width,
            height: 1,
        },
    );
    y.saturating_add(1)
}

fn wrapped_height(width: u16, text: &str) -> u16 {
    let usable = width.max(1) as usize;
    let mut lines = 0u16;
    for paragraph in text.split('\n') {
        let mut count = 0usize;
        for word in paragraph.split_whitespace() {
            let w = word.chars().count();
            if count == 0 {
                count = w;
            } else if count + 1 + w <= usable {
                count += 1 + w;
            } else {
                lines = lines.saturating_add(1);
                count = w;
            }
        }
        lines = lines.saturating_add(1);
    }
    lines.max(1)
}

fn put(frame: &mut Frame<'_>, x: u16, y: u16, width: u16, line: Line<'static>) {
    frame.render_widget(
        Paragraph::new(line),
        Rect {
            x,
            y,
            width,
            height: 1,
        },
    );
}

/// Handle a key event on the approve screen.
pub fn handle_key(_state: &mut State, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc | KeyCode::Char('b') => Action::Back,
        KeyCode::Enter => Action::Spawn,
        _ => Action::Stay,
    }
}
