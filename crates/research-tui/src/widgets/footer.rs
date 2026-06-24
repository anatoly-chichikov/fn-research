//! Bottom footer — `[key] label` hints in the kamishibai bracket convention so
//! a keypress can never be mistaken for a clickable element. One per-screen
//! KeyMap, mode-aware (a moded key always shows its current verb).

use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::state::{BriefState, Region, Screen, State};
use crate::theme;

/// One `[key] label` hint.
struct Hint {
    key: &'static str,
    label: &'static str,
}

impl Hint {
    fn new(key: &'static str, label: &'static str) -> Self {
        Self { key, label }
    }

    fn spans(&self) -> Vec<Span<'static>> {
        vec![
            Span::styled(format!("[{}]", self.key), theme::accent_bold()),
            Span::styled(format!(" {}", self.label), theme::body_dim()),
        ]
    }
}

/// Render the footer into `area`. Returns the area above the footer.
pub fn render(state: &State, area: Rect, frame: &mut Frame<'_>) -> Rect {
    let chunks = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "\u{2500}".repeat(area.width as usize),
            theme::rule(),
        ))),
        chunks[1],
    );

    let hints = hints_for(state);
    let mut spans = Vec::new();
    for (i, hint) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("   "));
        }
        spans.extend(hint.spans());
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), chunks[2]);
    frame.render_widget(
        Paragraph::new(Line::from(Hint::new("?", "keys").spans())).alignment(Alignment::Right),
        chunks[2],
    );

    chunks[0]
}

fn hints_for(state: &State) -> Vec<Hint> {
    if state.confirm.is_some() {
        return vec![Hint::new("y", "confirm"), Hint::new("n", "cancel")];
    }
    if state.editing.is_some() {
        return vec![Hint::new("\u{21b5}", "save"), Hint::new("esc", "cancel")];
    }
    match state.screen {
        Screen::Input => vec![
            Hint::new("\u{2191}\u{2193}", "field"),
            Hint::new("\u{2190}\u{2192}", "choose"),
            Hint::new("\u{21b5}", "newline"),
            Hint::new("Ctrl+G", "generate"),
        ],
        Screen::Generating => vec![Hint::new("esc", "cancel")],
        Screen::Tune => match state.region {
            Region::History => vec![
                Hint::new("\u{2191}\u{2193}", "take"),
                Hint::new("\u{21b5}", "restore"),
                Hint::new("space", "peek"),
                Hint::new("esc", "back"),
            ],
            Region::Tree => {
                let mut v = vec![
                    Hint::new("\u{2191}\u{2193}", "move"),
                    Hint::new("\u{2190}\u{2192}", "tune"),
                ];
                match state.brief_state() {
                    BriefState::Stale(_) => v.push(Hint::new("Ctrl+G", "regenerate")),
                    _ => v.push(Hint::new("a", "approve")),
                }
                v.push(Hint::new("H", "history"));
                v.push(Hint::new("e", "edit"));
                v.push(Hint::new("esc", "back"));
                v
            }
        },
        Screen::Approve => vec![Hint::new("\u{21b5}", "launch"), Hint::new("esc", "back")],
        Screen::Confirmation => vec![Hint::new("\u{21b5}", "start another")],
        Screen::Sessions => vec![
            Hint::new("\u{2191}\u{2193}", "navigate"),
            Hint::new("esc", "back"),
        ],
        Screen::Keys => vec![Hint::new("esc", "back")],
    }
}
