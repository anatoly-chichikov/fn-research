//! Tune — THE product screen, single-column angle blocks. Each angle prints
//! once (full-width title, a one-line why, then 3 sub-rows each pairing a
//! sub-question on the left with one dial pinned to the right). `↑/↓` move the
//! cursor across the 9 dials; `←/→` adjust the focused one live (no mode). The
//! left rail carries both focus (thickness) and state (gold when that angle's
//! questions went stale). `Ctrl+G`/`↵` regenerate when stale; `H` browses takes.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::Frame;

use crate::brief::TuiBrief;
use crate::palette;
use crate::state::{
    knob_of, Action, BriefState, Confirm, EditKind, EditState, Guarded, Prop, Region, Screen, State,
};
use crate::theme;
use crate::widgets::knob;

/// Render the tune screen (or the history browser when that region is focused).
pub fn render(state: &State, area: Rect, frame: &mut Frame<'_>) {
    if state.region == Region::History {
        render_history(state, area, frame);
        return;
    }
    let Some(base) = state.base() else {
        return;
    };
    let Some(pending) = state.working() else {
        return;
    };
    let locked = matches!(state.brief_state(), BriefState::Regenerating);
    let knob_x = area.x + area.width.saturating_sub(knob::WIDTH);
    let mut y = area.y;

    put(
        frame,
        area.x,
        y,
        area.width,
        label_value("TOPIC", &base.topic),
    );
    y = y.saturating_add(1);
    let intent_h = wrapped_height(area.width.saturating_sub(9), &base.intent).min(2);
    put_label(frame, area.x, y, "INTENT");
    wrapped(
        frame,
        area.x.saturating_add(9),
        y,
        area.width.saturating_sub(9),
        intent_h,
        &base.intent,
        theme::body_dim(),
    );
    y = y.saturating_add(intent_h);
    y = rule(area.x, y, area.width, area.bottom(), frame);
    y = y.saturating_add(1);

    for a in 0..3usize {
        if y.saturating_add(5) > area.bottom() {
            break;
        }
        y = render_block(state, base, pending, a, locked, area, knob_x, y, frame);
    }

    y = y.saturating_add(1);
    if y < area.bottom() {
        put(
            frame,
            area.x,
            y,
            area.width,
            Line::from(vec![
                Span::styled(
                    format!("v{} / {} takes", state.base_idx + 1, state.iterations.len()),
                    theme::body_dim(),
                ),
                Span::styled("   ", theme::body_dim()),
                Span::styled("[H]", theme::accent_bold()),
                Span::styled(" history", theme::body_dim()),
            ]),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn render_block(
    state: &State,
    base: &TuiBrief,
    pending: &TuiBrief,
    a: usize,
    locked: bool,
    area: Rect,
    knob_x: u16,
    top: u16,
    frame: &mut Frame<'_>,
) -> u16 {
    let focused_angle = !locked && state.focus_angle() == a;
    let stale = Prop::ALL.iter().any(|p| state.knob_changed(a, *p));
    let rail_glyph = if focused_angle {
        "\u{258c}"
    } else {
        "\u{258f}"
    };
    let rail_style = if stale {
        theme::gold()
    } else if focused_angle {
        theme::accent()
    } else {
        theme::rule()
    };
    let rail_lines: Vec<Line> = (0..5)
        .map(|_| Line::from(Span::styled(rail_glyph, rail_style)))
        .collect();
    frame.render_widget(
        Paragraph::new(rail_lines),
        Rect {
            x: area.x,
            y: top,
            width: 1,
            height: 5,
        },
    );

    let title_style = if stale {
        theme::body_dim()
    } else {
        theme::body()
    };
    let stamp_w = if stale { 14 } else { 0 };
    let title_w = area.width.saturating_sub(5 + stamp_w);
    put(
        frame,
        area.x.saturating_add(2),
        top,
        title_w.saturating_add(3),
        Line::from(vec![
            Span::styled(format!("{}  ", a + 1), theme::body_dim()),
            Span::styled(
                truncate(&base.topics[a].title, title_w as usize),
                title_style,
            ),
        ]),
    );
    if stale {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "\u{21b5} regenerate",
                theme::gold(),
            )))
            .alignment(Alignment::Right),
            Rect {
                x: area.x,
                y: top,
                width: area.width,
                height: 1,
            },
        );
    }
    put(
        frame,
        area.x.saturating_add(6),
        top.saturating_add(1),
        area.width.saturating_sub(6),
        Line::from(Span::styled(
            truncate(&base.topics[a].note, area.width.saturating_sub(6) as usize),
            theme::body_dim(),
        )),
    );

    let sub_x = area.x.saturating_add(11);
    let sub_w = knob_x.saturating_sub(sub_x).saturating_sub(1);
    for (k, prop) in Prop::ALL.into_iter().enumerate() {
        let row = top.saturating_add(2 + k as u16);
        let sub = &base.topics[a].subs[k];
        put(
            frame,
            area.x.saturating_add(6),
            row,
            4,
            Line::from(Span::styled(
                format!("{}.{}", a + 1, k + 1),
                theme::structure(),
            )),
        );
        put(
            frame,
            sub_x,
            row,
            sub_w,
            Line::from(Span::styled(
                truncate(&sub.title, sub_w as usize),
                if stale {
                    theme::body_dim()
                } else {
                    theme::body_soft()
                },
            )),
        );
        let focused = focused_angle && state.cursor % 3 == k as u8;
        let knob_rect = Rect {
            x: knob_x,
            y: row,
            width: knob::WIDTH,
            height: 1,
        };
        if focused {
            frame.render_widget(
                Block::default().style(Style::default().bg(palette::BAND)),
                knob_rect,
            );
        }
        let value = knob_of(&pending.topics[a], prop);
        let changed = state.knob_changed(a, prop);
        let bg = if focused { Some(palette::BAND) } else { None };
        frame.render_widget(
            Paragraph::new(knob::line(prop, value, focused, changed, locked, bg)),
            knob_rect,
        );
    }
    top.saturating_add(6)
}

fn render_history(state: &State, area: Rect, frame: &mut Frame<'_>) {
    let mut y = area.y;
    put_at(
        frame,
        area.x,
        y,
        area.width,
        Line::from(Span::styled("history", theme::accent_bold())),
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "pick a take to roll back to \u{2014} later takes are kept",
            theme::body_dim(),
        )))
        .alignment(Alignment::Right),
        Rect {
            x: area.x,
            y,
            width: area.width,
            height: 1,
        },
    );
    crate::widgets::header::underline(frame, area.x, y.saturating_add(1), "history");
    y = y.saturating_add(3);
    for (i, it) in state.iterations.iter().enumerate() {
        let row = y.saturating_add(i as u16 * 2);
        if row >= area.bottom() {
            break;
        }
        let is_base = i == state.base_idx;
        let is_cursor = i == state.hist_cursor;
        if is_cursor {
            frame.render_widget(
                Block::default().style(Style::default().bg(palette::BAND)),
                Rect {
                    x: area.x,
                    y: row,
                    width: area.width,
                    height: 1,
                },
            );
        }
        let bg = |s: Style| if is_cursor { s.bg(palette::BAND) } else { s };
        let mark = if is_cursor {
            Span::styled("\u{258c} ", theme::accent())
        } else {
            Span::raw("  ")
        };
        let id_style = if is_cursor {
            bg(theme::body())
        } else if is_base {
            theme::accent()
        } else {
            theme::body_dim()
        };
        let mut spans = vec![
            mark,
            Span::styled(format!("v{:<2} ", i + 1), id_style),
            Span::styled(
                format!("{:<34}", truncate(&it.label, 34)),
                bg(theme::body_soft()),
            ),
            Span::styled(
                format!("{}  ", digest_all(&it.brief)),
                bg(theme::body_dim()),
            ),
        ];
        if is_base {
            spans.push(Span::styled(
                "BASE",
                if is_cursor {
                    bg(theme::body())
                } else {
                    theme::accent_bold()
                },
            ));
        }
        put_at(frame, area.x, row, area.width, Line::from(spans));
    }
}

fn digest_all(brief: &TuiBrief) -> String {
    brief
        .topics
        .iter()
        .map(|r| format!("{}{}{}", r.depth, r.novelty, r.applied))
        .collect::<Vec<_>>()
        .join(" ")
}

fn label_value(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<8} "), theme::label()),
        Span::styled(value.to_string(), theme::body()),
    ])
}

fn put_label(frame: &mut Frame<'_>, x: u16, y: u16, text: &str) {
    put_at(
        frame,
        x,
        y,
        text.len() as u16 + 2,
        Line::from(Span::styled(text.to_string(), theme::label())),
    );
}

fn wrapped(frame: &mut Frame<'_>, x: u16, y: u16, w: u16, h: u16, text: &str, style: Style) {
    if h == 0 {
        return;
    }
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(text.to_string(), style))).wrap(Wrap { trim: true }),
        Rect {
            x,
            y,
            width: w,
            height: h,
        },
    );
}

fn rule(x: u16, y: u16, width: u16, bottom: u16, frame: &mut Frame<'_>) -> u16 {
    if y >= bottom {
        return y;
    }
    put(
        frame,
        x,
        y,
        width,
        Line::from(Span::styled(
            "\u{2500}".repeat(width as usize),
            theme::rule(),
        )),
    );
    y.saturating_add(1)
}

fn truncate(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        let mut out: String = chars[..max.saturating_sub(1)].iter().collect();
        out.push('\u{2026}');
        out
    }
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
    put_at(frame, x, y, width, line);
}

fn put_at(frame: &mut Frame<'_>, x: u16, y: u16, width: u16, line: Line<'static>) {
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

fn ctrl(key: KeyEvent, c: char) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char(c)
}

/// Handle a key event on the tune screen.
pub fn handle_key(state: &mut State, key: KeyEvent) -> Action {
    if matches!(state.brief_state(), BriefState::Regenerating) {
        return Action::Stay;
    }
    match state.region {
        Region::History => handle_history(state, key),
        Region::Tree => handle_tree(state, key),
    }
}

fn handle_tree(state: &mut State, key: KeyEvent) -> Action {
    if ctrl(key, 'g') {
        return regenerate(state);
    }
    match key.code {
        KeyCode::Left | KeyCode::Char('h') => {
            state.adjust_knob(-1);
            Action::Stay
        }
        KeyCode::Right | KeyCode::Char('l') => {
            state.adjust_knob(1);
            Action::Stay
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.cursor_down();
            Action::Stay
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.cursor_up();
            Action::Stay
        }
        KeyCode::Char('1') => {
            state.cursor_to_angle(0);
            Action::Stay
        }
        KeyCode::Char('2') => {
            state.cursor_to_angle(1);
            Action::Stay
        }
        KeyCode::Char('3') => {
            state.cursor_to_angle(2);
            Action::Stay
        }
        KeyCode::Enter => regenerate(state),
        KeyCode::Char('a') if matches!(state.brief_state(), BriefState::InSync) => {
            Action::Go(Screen::Approve)
        }
        KeyCode::Char('u') => {
            state.revert_pending();
            Action::Stay
        }
        KeyCode::Char('H') => {
            state.region = Region::History;
            state.hist_cursor = state.base_idx;
            Action::Stay
        }
        KeyCode::Char('e') => {
            let title = state
                .base()
                .map(|b| b.topics[state.focus_angle()].title.clone());
            if let Some(title) = title {
                state.editing = Some(EditState {
                    kind: EditKind::Root,
                    draft: title.clone(),
                    original: title,
                });
            }
            Action::Stay
        }
        KeyCode::Esc => {
            state.confirm = Some(Confirm {
                message: format!(
                    "discard this session ({} take{})?",
                    state.iterations.len(),
                    if state.iterations.len() == 1 { "" } else { "s" }
                ),
                action: Guarded::BackToInput,
            });
            Action::Stay
        }
        _ => Action::Stay,
    }
}

fn handle_history(state: &mut State, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            let last = state.iterations.len().saturating_sub(1);
            state.hist_cursor = (state.hist_cursor + 1).min(last);
            Action::Stay
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.hist_cursor = state.hist_cursor.saturating_sub(1);
            Action::Stay
        }
        KeyCode::Char(' ') => {
            state.peek = if state.peek == Some(state.hist_cursor) {
                None
            } else {
                Some(state.hist_cursor)
            };
            Action::Stay
        }
        KeyCode::Enter | KeyCode::Char('r') => {
            if state.dirty_count() > 0 {
                state.confirm = Some(Confirm {
                    message: format!("discard {} pending edit(s)?", state.dirty_count()),
                    action: Guarded::RestoreOver(state.hist_cursor),
                });
            } else {
                state.restore(state.hist_cursor);
            }
            Action::Stay
        }
        KeyCode::Esc | KeyCode::Char('H') | KeyCode::Char('h') => {
            state.region = Region::Tree;
            state.peek = None;
            Action::Stay
        }
        _ => Action::Stay,
    }
}

fn regenerate(state: &State) -> Action {
    if matches!(state.brief_state(), BriefState::Stale(_)) {
        Action::Regenerate
    } else {
        Action::Stay
    }
}
