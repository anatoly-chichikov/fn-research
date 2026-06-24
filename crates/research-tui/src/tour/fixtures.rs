//! State fixtures — one per screenshot. Deterministic, so the tour PNGs and
//! the text snapshots are reproducible.

use std::path::PathBuf;

use crate::brief::mock;
use crate::keys::KeyRow;
use crate::sessions::{SessionRow, SessionStatus};
use crate::spawn::SpawnInfo;
use crate::state::{EditKind, EditState, InputFocus, Iteration, Region, Screen, State};

/// One captured shot.
pub struct Shot {
    pub name: &'static str,
    pub state: State,
}

/// All shots, in committed order.
pub fn all() -> Vec<Shot> {
    vec![
        shot("01-input-empty", input_empty()),
        shot("02-input-typed", input_typed()),
        shot("03-generating", generating_mid()),
        shot("04-tune-default", tune_default()),
        shot("05-tune-stale", tune_stale()),
        shot("06-tune-regenerating", tune_regenerating()),
        shot("06-tune-history", tune_history()),
        shot("07-tune-edit", tune_edit()),
        shot("08-approve", approve_default()),
        shot("09-confirmation", confirmation_default()),
        shot("10-sessions-rows", sessions_with_rows()),
        shot("11-sessions-empty", sessions_empty()),
        shot("12-keys", keys_mixed()),
        shot("13-help", help_overlay()),
    ]
}

fn shot(name: &'static str, state: State) -> Shot {
    Shot { name, state }
}

fn base() -> State {
    let mut s = State::new(true);
    s.keys = sample_keys(true);
    s.sessions = sample_sessions();
    s
}

/// Empty topic, focus on the topic field.
pub fn input_empty() -> State {
    let mut s = base();
    s.screen = Screen::Input;
    s.input_focus = InputFocus::Topic;
    s.topic.clear();
    s
}

/// Typed topic.
pub fn input_typed() -> State {
    let mut s = input_empty();
    s.topic = "Quantum computing".to_string();
    s
}

/// Spinner mid-cycle.
pub fn generating_mid() -> State {
    let mut s = input_typed();
    s.screen = Screen::Generating;
    s.spinner_tick = 14;
    s
}

/// Build a take with the given root knob matrix and a label.
fn iteration(knobs: [[u8; 3]; 3], label: &str, parent: Option<usize>) -> Iteration {
    let mut brief = mock::sample("Quantum computing", "English");
    for (a, root) in brief.topics.iter_mut().enumerate() {
        root.depth = knobs[a][0];
        root.novelty = knobs[a][1];
        root.applied = knobs[a][2];
    }
    Iteration::new(brief, parent, label.to_string())
}

/// A state with three takes already in history, BASE = the latest.
fn tuned() -> State {
    let mut s = base();
    s.topic = "Quantum computing".to_string();
    s.iterations = vec![
        iteration([[3, 3, 3], [4, 3, 2], [3, 2, 4]], "first pass", None),
        iteration(
            [[4, 3, 3], [5, 4, 2], [3, 3, 5]],
            "deeper \u{00b7} more applied",
            Some(0),
        ),
        iteration(
            [[4, 4, 3], [5, 4, 3], [4, 3, 5]],
            "newer \u{00b7} applied lens",
            Some(1),
        ),
    ];
    s.base_idx = 2;
    s.pending = Some(s.iterations[2].brief.clone());
    s.screen = Screen::Tune;
    s.region = Region::Tree;
    s.cursor = 7;
    s
}

/// Tune, in sync, the third angle focused.
pub fn tune_default() -> State {
    tuned()
}

/// Tune, stale — two pending knob edits on the focused angle.
pub fn tune_stale() -> State {
    let mut s = tuned();
    if let Some(p) = s.pending.as_mut() {
        p.topics[2].depth = 5;
        p.topics[2].applied = 5;
    }
    s.cursor = 6;
    s
}

/// Tune mid-regenerate — knobs locked, the spinner badge live.
pub fn tune_regenerating() -> State {
    let mut s = tune_stale();
    s.regenerating = true;
    s.spinner_tick = 3;
    s
}

/// Tune with the iteration history focused, cursor on the first take.
pub fn tune_history() -> State {
    let mut s = tuned();
    s.region = Region::History;
    s.hist_cursor = 0;
    s
}

/// Tune with the edit modal open over the focused angle title.
pub fn tune_edit() -> State {
    let mut s = tuned();
    let title = s.base().unwrap().topics[s.focus_angle()].title.clone();
    s.editing = Some(EditState {
        kind: EditKind::Root,
        draft: title.clone(),
        original: title,
    });
    s
}

/// Approve the active take.
pub fn approve_default() -> State {
    let mut s = tuned();
    s.screen = Screen::Approve;
    s.region = Region::Tree;
    s
}

/// Confirmation with fake spawn info.
pub fn confirmation_default() -> State {
    let mut s = tuned();
    s.screen = Screen::Confirmation;
    s.spawned = Some(SpawnInfo {
        session_id: "3e4fc072-1234-5678-9abc-def012345678".to_string(),
        topic: "Quantum computing".to_string(),
        provider: "parallel".to_string(),
        processor: "ultra".to_string(),
        language: "English".to_string(),
        output_path: PathBuf::from("./output/2026-06-24_quantum-computing_3e4fc072"),
        log_path: PathBuf::from("./output/run-3e4fc072.log"),
        pid: 87234,
        mocked: true,
    });
    s
}

/// Sessions screen with rows.
pub fn sessions_with_rows() -> State {
    let mut s = base();
    s.screen = Screen::Sessions;
    s.sessions = sample_sessions();
    s.sessions_idx = 1;
    s
}

/// Sessions screen with no rows.
pub fn sessions_empty() -> State {
    let mut s = base();
    s.screen = Screen::Sessions;
    s.sessions = Vec::new();
    s
}

/// Keys screen with mixed-set state.
pub fn keys_mixed() -> State {
    let mut s = base();
    s.screen = Screen::Keys;
    s.keys = sample_keys(false);
    s
}

/// Help overlay displayed on top of tune.
pub fn help_overlay() -> State {
    let mut s = tuned();
    s.help_open = true;
    s
}

fn sample_sessions() -> Vec<SessionRow> {
    use chrono::NaiveDate;
    vec![
        SessionRow {
            id_short: "041abcd1".into(),
            topic: "AI transformation of academic research".into(),
            provider: "parallel · ultra".into(),
            status: SessionStatus::Done,
            created: NaiveDate::from_ymd_opt(2026, 5, 6)
                .unwrap()
                .and_hms_opt(14, 2, 0)
                .unwrap(),
            pdf: None,
        },
        SessionRow {
            id_short: "040c12fa".into(),
            topic: "Rust ownership model for systems people".into(),
            provider: "parallel · ultra".into(),
            status: SessionStatus::Done,
            created: NaiveDate::from_ymd_opt(2026, 5, 4)
                .unwrap()
                .and_hms_opt(9, 31, 0)
                .unwrap(),
            pdf: None,
        },
        SessionRow {
            id_short: "039b2110".into(),
            topic: "Gut-microbiome × longevity protocols".into(),
            provider: "valyu · standard".into(),
            status: SessionStatus::Done,
            created: NaiveDate::from_ymd_opt(2026, 5, 2)
                .unwrap()
                .and_hms_opt(18, 44, 0)
                .unwrap(),
            pdf: None,
        },
        SessionRow {
            id_short: "037e8c30".into(),
            topic: "Why container shipping rates collapsed".into(),
            provider: "xai · full".into(),
            status: SessionStatus::Error,
            created: NaiveDate::from_ymd_opt(2026, 4, 27)
                .unwrap()
                .and_hms_opt(11, 8, 0)
                .unwrap(),
            pdf: None,
        },
    ]
}

fn sample_keys(env_filled: bool) -> Vec<KeyRow> {
    if env_filled {
        vec![
            KeyRow {
                name: "PARALLEL_API_KEY",
                required: true,
                set: true,
                preview: "••••••••a3f2".into(),
            },
            KeyRow {
                name: "VALYU_API_KEY",
                required: true,
                set: true,
                preview: "••••••••7c91".into(),
            },
            KeyRow {
                name: "XAI_API_KEY",
                required: false,
                set: false,
                preview: String::new(),
            },
            KeyRow {
                name: "GEMINI_API_KEY",
                required: false,
                set: true,
                preview: "••••••••dQ".into(),
            },
            KeyRow {
                name: "REPORT_FOR",
                required: false,
                set: true,
                preview: "Anton K.".into(),
            },
        ]
    } else {
        vec![
            KeyRow {
                name: "PARALLEL_API_KEY",
                required: true,
                set: true,
                preview: "••••••••a3f2".into(),
            },
            KeyRow {
                name: "VALYU_API_KEY",
                required: true,
                set: false,
                preview: String::new(),
            },
            KeyRow {
                name: "XAI_API_KEY",
                required: false,
                set: false,
                preview: String::new(),
            },
            KeyRow {
                name: "GEMINI_API_KEY",
                required: false,
                set: true,
                preview: "••••••••dQ".into(),
            },
            KeyRow {
                name: "REPORT_FOR",
                required: false,
                set: false,
                preview: String::new(),
            },
        ]
    }
}
