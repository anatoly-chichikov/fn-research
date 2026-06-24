//! Main loop, render dispatch, and event plumbing.
//!
//! `run` enters raw mode, sets up an alternate screen, and pumps events into
//! the [`State`] until the user quits. `render` is reused by the screenshot
//! tour — never inline rendering logic anywhere else.

use std::io;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossterm::event::{
    DisableBracketedPaste, EnableBracketedPaste, Event, KeyEvent, KeyEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::Frame;
use ratatui::Terminal;

use crate::brief::generator::BriefGenerator;
use crate::brief::{mock::MockBriefGenerator, BriefError, BriefRequest, TuiBrief};
use crate::events as global_events;
use crate::language;
use crate::options;
use crate::palette;
use crate::screens;
use crate::spawn::{spawn, SpawnMode};
use crate::state::{Action, InputFocus, Screen, State};
use crate::widgets;
use crate::RunOptions;

/// Internal events produced by background work + ticker. Brief replies carry
/// the request epoch so a cancelled or superseded reply can be discarded.
enum AppEvent {
    Key(KeyEvent),
    Paste(String),
    Tick,
    BriefReady(u64, Box<TuiBrief>),
    BriefFailed(u64, BriefError),
}

/// Live entry point.
pub fn run(opts: RunOptions) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableBracketedPaste)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, opts);

    disable_raw_mode().ok();
    let mut stdout = io::stdout();
    execute!(stdout, DisableBracketedPaste, LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    result
}

fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    opts: RunOptions,
) -> io::Result<()> {
    let mut state = State::new(opts.mock);
    state.sessions = crate::sessions::load(&opts.root.join("output"));
    state.keys = crate::keys::load();

    let (tx, rx): (Sender<AppEvent>, Receiver<AppEvent>) = mpsc::channel();
    spawn_input_thread(tx.clone());
    spawn_tick_thread(tx.clone());

    let generator: Arc<dyn BriefGenerator> = if opts.mock {
        Arc::new(MockBriefGenerator::default())
    } else {
        match crate::brief::gemini::GeminiBriefGenerator::from_env() {
            Some(g) => Arc::new(g),
            None => {
                eprintln!("GEMINI_API_KEY not set — falling back to sample brief");
                Arc::new(MockBriefGenerator::default())
            }
        }
    };

    loop {
        terminal.draw(|f| render(&state, f))?;

        let evt = match rx.recv_timeout(Duration::from_millis(120)) {
            Ok(e) => e,
            Err(mpsc::RecvTimeoutError::Timeout) => AppEvent::Tick,
            Err(_) => break,
        };

        match evt {
            AppEvent::Tick => {
                state.spinner_tick = state.spinner_tick.wrapping_add(1);
            }
            AppEvent::Paste(text) => {
                if let Some(edit) = state.editing.as_mut() {
                    edit.draft.push_str(&text);
                } else if state.screen == Screen::Input && state.input_focus == InputFocus::Topic {
                    state.topic.push_str(&text);
                }
            }
            AppEvent::Key(key) => {
                if key.kind != KeyEventKind::Press && key.kind != KeyEventKind::Repeat {
                    continue;
                }
                let action = if let Some(a) = global_events::handle_global(&mut state, key) {
                    a
                } else {
                    dispatch(&mut state, key)
                };
                match action {
                    Action::Stay => {}
                    Action::Go(target) => {
                        if state.screen == Screen::Generating && target == Screen::Input {
                            state.cancel_request();
                        }
                        if target == Screen::Sessions {
                            state.sessions = crate::sessions::load(&opts.root.join("output"));
                        }
                        if target == Screen::Keys {
                            state.keys = crate::keys::load();
                        }
                        state.go(target);
                    }
                    Action::Back => {
                        state.back();
                    }
                    Action::Quit => break,
                    Action::RequestBrief => start_fresh(&mut state, &tx, &generator),
                    Action::Regenerate => {
                        if !state.in_flight {
                            if let Some(pending) = state.pending.clone() {
                                state.error = None;
                                state.regenerating = true;
                                let epoch = state.start_request();
                                request_brief(
                                    epoch,
                                    BriefRequest::retune(pending),
                                    &tx,
                                    &generator,
                                );
                            }
                        }
                    }
                    Action::Spawn => {
                        do_spawn(&mut state, &opts);
                    }
                }
                if state.screen == Screen::Generating
                    && state.iterations.is_empty()
                    && !state.in_flight
                {
                    start_fresh(&mut state, &tx, &generator);
                }
            }
            AppEvent::BriefReady(epoch, brief) => {
                if state.accepts(epoch) {
                    let was_generating = state.screen == Screen::Generating;
                    state.accept_brief(*brief);
                    state.error = None;
                    state.in_flight = false;
                    if was_generating {
                        state.go(Screen::Tune);
                    }
                }
            }
            AppEvent::BriefFailed(epoch, err) => {
                if state.accepts(epoch) {
                    state.error = Some(err.to_string());
                    state.regenerating = false;
                    state.in_flight = false;
                    if state.screen == Screen::Generating {
                        state.go(Screen::Input);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Start a fresh brief request from the typed topic, discarding any prior session.
fn start_fresh(state: &mut State, tx: &Sender<AppEvent>, generator: &Arc<dyn BriefGenerator>) {
    if state.in_flight {
        return;
    }
    let topic = state.topic.trim().to_string();
    let language = language::detect(&topic).to_string();
    state.reset_session();
    let epoch = state.start_request();
    request_brief(epoch, BriefRequest::fresh(&topic, &language), tx, generator);
}

fn spawn_input_thread(tx: Sender<AppEvent>) {
    std::thread::spawn(move || loop {
        if let Ok(true) = crossterm::event::poll(Duration::from_millis(80)) {
            let event = match crossterm::event::read() {
                Ok(Event::Key(key)) => Some(AppEvent::Key(key)),
                Ok(Event::Paste(text)) => Some(AppEvent::Paste(text)),
                _ => None,
            };
            if let Some(event) = event {
                if tx.send(event).is_err() {
                    break;
                }
            }
        }
    });
}

fn spawn_tick_thread(tx: Sender<AppEvent>) {
    std::thread::spawn(move || {
        let interval = Duration::from_millis(140);
        let mut last = Instant::now();
        loop {
            std::thread::sleep(Duration::from_millis(20));
            if last.elapsed() >= interval {
                if tx.send(AppEvent::Tick).is_err() {
                    break;
                }
                last = Instant::now();
            }
        }
    });
}

fn request_brief(
    epoch: u64,
    req: BriefRequest,
    tx: &Sender<AppEvent>,
    generator: &Arc<dyn BriefGenerator>,
) {
    let tx = tx.clone();
    let gen_arc = Arc::clone(generator);
    std::thread::spawn(move || match gen_arc.generate(&req) {
        Ok(brief) => {
            let _ = tx.send(AppEvent::BriefReady(epoch, Box::new(brief)));
        }
        Err(e) => {
            let _ = tx.send(AppEvent::BriefFailed(epoch, e));
        }
    });
}

fn do_spawn(state: &mut State, opts: &RunOptions) {
    let Some(brief) = state.base().cloned() else {
        return;
    };
    let provider = options::PROVIDERS[state.provider_idx].id;
    let processor = options::processors_for(provider)
        .get(state.processor_idx)
        .map(|p| p.id)
        .unwrap_or("pro");
    let mode = if opts.mock {
        SpawnMode::Mock { root: &opts.root }
    } else {
        SpawnMode::Real { root: &opts.root }
    };
    match spawn(&brief, provider, processor, mode) {
        Ok(info) => {
            state.spawned = Some(info);
            state.go(Screen::Confirmation);
        }
        Err(e) => {
            state.error = Some(e);
        }
    }
}

fn dispatch(state: &mut State, key: KeyEvent) -> Action {
    match state.screen {
        Screen::Input => {
            let action = screens::input::handle_key(state, key);
            if let Action::Go(Screen::Generating) = action {
                state.error = None;
                state.go(Screen::Generating);
                state.generating_started_at = Instant::now();
                return Action::RequestBrief;
            }
            action
        }
        Screen::Generating => screens::generating::handle_key(state, key),
        Screen::Tune => screens::tune::handle_key(state, key),
        Screen::Approve => screens::approve::handle_key(state, key),
        Screen::Confirmation => {
            state.reset_session();
            state.spawned = None;
            state.topic.clear();
            state.nav.clear();
            screens::confirmation::handle_key(state, key)
        }
        Screen::Sessions => screens::sessions::handle_key(state, key),
        Screen::Keys => screens::keys::handle_key(state, key),
    }
}

/// Top-level draw — used by both the live loop and the tour.
pub fn render(state: &State, frame: &mut Frame<'_>) {
    let area = frame.area();
    fill_background(area, frame);
    let gutter = Rect {
        x: area.x.saturating_add(2),
        y: area.y.saturating_add(1),
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };
    let body = widgets::header::render(state, gutter, frame);
    let body = widgets::footer::render(state, body, frame);

    match state.screen {
        Screen::Input => screens::input::render(state, body, frame),
        Screen::Generating => screens::generating::render(state, body, frame),
        Screen::Tune => screens::tune::render(state, body, frame),
        Screen::Approve => screens::approve::render(state, body, frame),
        Screen::Confirmation => screens::confirmation::render(state, body, frame),
        Screen::Sessions => screens::sessions::render(state, body, frame),
        Screen::Keys => screens::keys::render(state, body, frame),
    }

    if let Some(edit) = state.editing.as_ref() {
        widgets::edit_modal::render(edit, area, frame);
    }
    if let Some(confirm) = state.confirm.as_ref() {
        render_confirm(confirm, area, frame);
    }
    if state.help_open {
        render_help_overlay(area, frame);
    }
}

fn fill_background(area: Rect, frame: &mut Frame<'_>) {
    use ratatui::style::Style;
    use ratatui::widgets::Block;
    let block = Block::default().style(Style::default().bg(palette::GROUND));
    frame.render_widget(block, area);
}

fn render_confirm(confirm: &crate::state::Confirm, area: Rect, frame: &mut Frame<'_>) {
    use ratatui::style::Style;
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{Block, Borders, Clear, Paragraph};
    let card = centered(area, 56, 5);
    frame.render_widget(Clear, card);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(palette::GOLD))
        .style(Style::default().bg(palette::SURFACE));
    let inner = block.inner(card);
    frame.render_widget(block, card);
    let lines = vec![
        Line::from(Span::styled(confirm.message.clone(), crate::theme::body())),
        Line::from(""),
        Line::from(vec![
            Span::styled("y", crate::theme::accent()),
            Span::styled(" confirm    ", crate::theme::body_dim()),
            Span::styled("n", crate::theme::accent()),
            Span::styled(" cancel", crate::theme::body_dim()),
        ]),
    ];
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_help_overlay(area: Rect, frame: &mut Frame<'_>) {
    use ratatui::layout::{Constraint, Layout};
    use ratatui::style::Style;
    use ratatui::text::Span;
    use ratatui::widgets::{Block, Borders, Clear, Paragraph};

    frame.render_widget(Clear, area);
    frame.render_widget(
        Block::default().style(Style::default().bg(palette::INSET)),
        area,
    );
    let card = centered(area, 66, 20);
    frame.render_widget(
        Block::default().style(Style::default().bg(palette::SURFACE)),
        card,
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(palette::ACCENT))
        .style(Style::default().bg(palette::SURFACE))
        .title(Span::styled(" shortcuts ", crate::theme::accent()));
    let inner = block.inner(card);
    frame.render_widget(block, card);

    let lines = vec![
        line("j / k", "move angle (tune) / take (history)"),
        line("tab", "next knob in the angle"),
        line("\u{2190} / \u{2192}", "adjust the focused knob \u{00b1}1"),
        line("d n a", "jump to depth / novelty / applied"),
        line("\u{21b5}", "regenerate (when stale) / restore (history)"),
        line("a", "approve the current take"),
        line("H", "focus the iteration history"),
        line("space", "peek a take's diff (history)"),
        line("u", "revert pending knob edits"),
        line("e", "edit the focused angle title"),
        line("esc", "back / cancel"),
        line("F1 F2", "sessions \u{00b7} keys"),
        line("?", "toggle this help"),
        line("\u{2303}C", "quit"),
    ];
    let body = Layout::vertical(vec![Constraint::Length(1); lines.len()]).split(inner);
    for (i, l) in lines.into_iter().enumerate() {
        if let Some(rect) = body.get(i) {
            frame.render_widget(Paragraph::new(l), *rect);
        }
    }
}

fn line(key: &'static str, label: &'static str) -> ratatui::text::Line<'static> {
    use ratatui::text::{Line, Span};
    Line::from(vec![
        Span::styled(format!("  {key:<10}"), crate::theme::accent()),
        Span::styled(label, crate::theme::body_dim()),
    ])
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    Rect {
        x: area.x + (area.width.saturating_sub(w)) / 2,
        y: area.y + (area.height.saturating_sub(h)) / 2,
        width: w,
        height: h,
    }
}
