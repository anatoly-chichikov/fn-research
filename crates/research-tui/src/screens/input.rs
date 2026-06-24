//! Topic input — a multi-line, growing field (type or paste a whole question)
//! plus a provider/processor chip. Enter inserts a newline; Ctrl+G generates.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::options::{processors_for, PROVIDERS};
use crate::palette;
use crate::state::{Action, InputFocus, Screen, State};
use crate::theme;

/// Render the input screen into `area`.
pub fn render(state: &State, area: Rect, frame: &mut Frame<'_>) {
    let mut y = area.y.saturating_add(1);
    let topic_focused = state.input_focus == InputFocus::Topic;

    let field_w = area.width;
    let lines: Vec<Line<'static>> = if state.topic.is_empty() {
        let mut first: Vec<Span<'static>> = Vec::new();
        if topic_focused {
            first.push(Span::styled("\u{2588}", theme::accent()));
        }
        first.push(Span::styled(
            " type or paste your question\u{2026}",
            theme::body_dim(),
        ));
        vec![Line::from(first)]
    } else {
        let mut ls: Vec<Line<'static>> = state
            .topic
            .split('\n')
            .map(|seg| Line::from(Span::styled(seg.to_string(), theme::body())))
            .collect();
        if topic_focused {
            if let Some(last) = ls.last_mut() {
                last.spans.push(Span::styled("\u{2588}", theme::accent()));
            }
        }
        ls
    };
    let text_h = if state.topic.is_empty() {
        1
    } else {
        wrapped_height(field_w, &state.topic)
    }
    .max(1)
    .min(area.height.saturating_sub(4));
    frame.render_widget(
        Paragraph::new(Text::from(lines)).wrap(Wrap { trim: false }),
        Rect {
            x: area.x,
            y,
            width: field_w,
            height: text_h,
        },
    );
    y = y.saturating_add(text_h).saturating_add(1);

    let underline_style = if topic_focused {
        Style::default().fg(palette::ACCENT)
    } else {
        theme::rule()
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "\u{2500}".repeat(field_w as usize),
            underline_style,
        ))),
        Rect {
            x: area.x,
            y,
            width: field_w,
            height: 1,
        },
    );
    y = y.saturating_add(2);

    let provider = &PROVIDERS[state.provider_idx];
    let processors = processors_for(provider.id);
    let provider_labels: Vec<&str> = PROVIDERS.iter().map(|p| p.label).collect();
    let processor_labels: Vec<&str> = processors.iter().map(|p| p.label).collect();
    frame.render_widget(
        Paragraph::new(picker_row(
            "PROVIDER",
            &provider_labels,
            state.provider_idx,
            state.input_focus == InputFocus::Provider,
        )),
        Rect {
            x: area.x,
            y,
            width: field_w,
            height: 1,
        },
    );
    y = y.saturating_add(1);
    frame.render_widget(
        Paragraph::new(picker_row(
            "PROCESSOR",
            &processor_labels,
            state
                .processor_idx
                .min(processor_labels.len().saturating_sub(1)),
            state.input_focus == InputFocus::Processor,
        )),
        Rect {
            x: area.x,
            y,
            width: field_w,
            height: 1,
        },
    );
}

/// One inline picker row: a dim label then every option, the selected one in
/// terracotta (wrapped in `«…»` when the field is focused so ←/→ reads as live).
fn picker_row(label: &str, options: &[&str], selected: usize, focused: bool) -> Line<'static> {
    let mut spans = vec![Span::styled(format!("{label:<10}  "), theme::body_dim())];
    for (i, opt) in options.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("   "));
        }
        if i == selected && focused {
            spans.push(Span::styled(
                format!("\u{00ab}{opt}\u{00bb}"),
                theme::accent_bold(),
            ));
        } else if i == selected {
            spans.push(Span::styled(opt.to_string(), theme::accent()));
        } else {
            spans.push(Span::styled(opt.to_string(), theme::body_dim()));
        }
    }
    Line::from(spans)
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

/// Handle a key event on this screen.
pub fn handle_key(state: &mut State, key: KeyEvent) -> Action {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('g') {
        return if state.topic.trim().is_empty() {
            Action::Stay
        } else {
            Action::Go(Screen::Generating)
        };
    }
    match key.code {
        KeyCode::Tab => {
            state.input_focus = state.input_focus.next();
            Action::Stay
        }
        KeyCode::BackTab => {
            state.input_focus = state.input_focus.up();
            Action::Stay
        }
        KeyCode::Down => {
            state.input_focus = state.input_focus.down();
            Action::Stay
        }
        KeyCode::Up => {
            state.input_focus = state.input_focus.up();
            Action::Stay
        }
        KeyCode::Enter if state.input_focus == InputFocus::Topic => {
            state.topic.push('\n');
            Action::Stay
        }
        KeyCode::Enter if !state.topic.trim().is_empty() => Action::Go(Screen::Generating),
        KeyCode::Left | KeyCode::Right => {
            let dir: i32 = if key.code == KeyCode::Right { 1 } else { -1 };
            cycle(state, dir);
            Action::Stay
        }
        KeyCode::Backspace if state.input_focus == InputFocus::Topic => {
            state.topic.pop();
            Action::Stay
        }
        KeyCode::Char(c) if state.input_focus == InputFocus::Topic => {
            state.topic.push(c);
            Action::Stay
        }
        _ => Action::Stay,
    }
}

fn cycle(state: &mut State, dir: i32) {
    match state.input_focus {
        InputFocus::Topic => {}
        InputFocus::Provider => {
            let n = PROVIDERS.len() as i32;
            let next = (state.provider_idx as i32 + dir).rem_euclid(n) as usize;
            state.provider_idx = next;
            state.processor_idx = crate::options::default_processor_idx(PROVIDERS[next].id);
        }
        InputFocus::Processor => {
            let provider = PROVIDERS[state.provider_idx].id;
            let processors = processors_for(provider);
            if processors.is_empty() {
                return;
            }
            let n = processors.len() as i32;
            state.processor_idx = (state.processor_idx as i32 + dir).rem_euclid(n) as usize;
        }
    }
}
