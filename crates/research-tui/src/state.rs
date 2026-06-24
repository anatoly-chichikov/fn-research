//! TUI state — single source of truth for what the user sees.
//!
//! Every screen's render fn reads from `State` and never mutates. Mutation
//! happens in `handle_key` and in the channel-driven update fns. The brief
//! lives as an append-only list of [`Iteration`]s plus a `pending` working
//! copy whose knobs the user tweaks; the diff between them is the "stale" state.

use crate::brief::TuiBrief;
use crate::keys::KeyRow;
use crate::sessions::SessionRow;
use crate::spawn::SpawnInfo;

/// Top-level screen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Screen {
    Input,
    Generating,
    Tune,
    Approve,
    Confirmation,
    Sessions,
    Keys,
}

/// Which field on the input screen has focus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputFocus {
    Topic,
    Provider,
    Processor,
}

impl InputFocus {
    /// Cycle to the next field (Tab — wraps).
    pub fn next(self) -> Self {
        match self {
            Self::Topic => Self::Provider,
            Self::Provider => Self::Processor,
            Self::Processor => Self::Topic,
        }
    }

    /// Move focus down the form, clamped at the last field.
    pub fn down(self) -> Self {
        match self {
            Self::Topic => Self::Provider,
            _ => Self::Processor,
        }
    }

    /// Move focus up the form, clamped at the topic field.
    pub fn up(self) -> Self {
        match self {
            Self::Processor => Self::Provider,
            _ => Self::Topic,
        }
    }
}

/// A depth/novelty/applied knob — a regeneration parameter on a 1..5 scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prop {
    Depth,
    Novelty,
    Applied,
}

impl Prop {
    /// All properties in display order.
    pub const ALL: [Prop; 3] = [Prop::Depth, Prop::Novelty, Prop::Applied];

    /// Lowercase one-word name.
    pub fn short(self) -> &'static str {
        match self {
            Self::Depth => "depth",
            Self::Novelty => "novelty",
            Self::Applied => "applied",
        }
    }

    /// Uppercase label used on the knob row.
    pub fn caps(self) -> &'static str {
        match self {
            Self::Depth => "DEPTH",
            Self::Novelty => "NOVELTY",
            Self::Applied => "APPLIED",
        }
    }

    /// Single-letter tag used in the compact `d4 n3 a5` digest.
    pub fn tag(self) -> char {
        match self {
            Self::Depth => 'd',
            Self::Novelty => 'n',
            Self::Applied => 'a',
        }
    }

    /// Cycle by signed delta within the three knobs.
    pub fn cycle(self, delta: i32) -> Self {
        let idx = Self::ALL.iter().position(|p| *p == self).unwrap_or(0) as i32;
        let next = (idx + delta).rem_euclid(Self::ALL.len() as i32) as usize;
        Self::ALL[next]
    }
}

/// One committed take of the brief, appended on every regenerate.
#[derive(Clone, Debug)]
pub struct Iteration {
    /// The committed questions + knob matrix for this take.
    pub brief: TuiBrief,
    /// Parent take this was regenerated from (`None` for the first pass).
    pub parent: Option<usize>,
    /// Short human summary of what this take changed.
    pub label: String,
}

impl Iteration {
    /// Construct a take.
    pub fn new(brief: TuiBrief, parent: Option<usize>, label: String) -> Self {
        Self {
            brief,
            parent,
            label,
        }
    }
}

/// Which region of the Tune screen owns the keyboard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    Tree,
    History,
}

/// The brief lifecycle, derived from the iteration list + pending edits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BriefState {
    InSync,
    Stale(u8),
    Regenerating,
}

/// What the user is editing in the modal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditKind {
    Root,
    Intent,
    Topic,
}

/// Modal edit state when the user is editing text inline.
#[derive(Clone, Debug)]
pub struct EditState {
    pub kind: EditKind,
    pub draft: String,
    pub original: String,
}

/// What a pending y/n confirmation will do when accepted.
#[derive(Clone, Debug)]
pub enum Guarded {
    /// Discard the whole session and return to Input.
    BackToInput,
    /// Quit the app.
    Quit,
    /// Drop pending knob edits and restore the iteration at this index.
    RestoreOver(usize),
}

/// A y/n guard prompt protecting a destructive action.
#[derive(Clone, Debug)]
pub struct Confirm {
    pub message: String,
    pub action: Guarded,
}

/// The full TUI state.
pub struct State {
    pub screen: Screen,
    pub nav: Vec<Screen>,

    // input
    pub topic: String,
    pub provider_idx: usize,
    pub processor_idx: usize,
    pub input_focus: InputFocus,

    // generating
    pub spinner_tick: u32,
    pub generating_started_at: std::time::Instant,
    pub error: Option<String>,
    /// Monotonic id of the currently-valid brief request. A worker reply whose
    /// epoch differs (cancelled or superseded) is discarded, not accepted.
    pub gen_epoch: u64,
    /// Whether a brief request is in flight (blocks a second concurrent one).
    pub in_flight: bool,

    // brief — append-only history + a pending working copy
    pub iterations: Vec<Iteration>,
    pub base_idx: usize,
    pub pending: Option<TuiBrief>,
    pub regenerating: bool,

    // tune focus
    pub region: Region,
    /// Focused knob, flattened: `angle * 3 + knob_index`, range 0..9.
    pub cursor: u8,
    pub hist_cursor: usize,
    pub peek: Option<usize>,

    // overlays
    pub editing: Option<EditState>,
    pub confirm: Option<Confirm>,
    pub help_open: bool,

    // confirmation
    pub spawned: Option<SpawnInfo>,

    // sessions / keys
    pub sessions: Vec<SessionRow>,
    pub sessions_idx: usize,
    pub keys: Vec<KeyRow>,
    pub keys_idx: usize,

    // lifecycle
    pub should_quit: bool,
    pub mock: bool,
}

impl State {
    /// Default initial state — Input screen, empty topic.
    pub fn new(mock: bool) -> Self {
        Self {
            screen: Screen::Input,
            nav: Vec::new(),
            topic: String::new(),
            provider_idx: 0,
            processor_idx: crate::options::default_processor_idx("parallel"),
            input_focus: InputFocus::Topic,
            spinner_tick: 0,
            generating_started_at: std::time::Instant::now(),
            error: None,
            gen_epoch: 0,
            in_flight: false,
            iterations: Vec::new(),
            base_idx: 0,
            pending: None,
            regenerating: false,
            region: Region::Tree,
            cursor: 0,
            hist_cursor: 0,
            peek: None,
            editing: None,
            confirm: None,
            help_open: false,
            spawned: None,
            sessions: Vec::new(),
            sessions_idx: 0,
            keys: Vec::new(),
            keys_idx: 0,
            should_quit: false,
            mock,
        }
    }

    /// Switch to a screen, pushing the current one onto the back-stack.
    pub fn go(&mut self, target: Screen) {
        if self.screen != target {
            self.nav.push(self.screen.clone());
            self.screen = target;
        }
    }

    /// Pop one frame off the back-stack; returns the screen we land on.
    pub fn back(&mut self) -> Screen {
        let target = self.nav.pop().unwrap_or(Screen::Input);
        self.screen = target.clone();
        target
    }

    /// Begin a brief request: bump the epoch, mark in-flight, return the epoch
    /// the worker must echo back for its reply to be accepted.
    pub fn start_request(&mut self) -> u64 {
        self.gen_epoch = self.gen_epoch.wrapping_add(1);
        self.in_flight = true;
        self.gen_epoch
    }

    /// Cancel any in-flight request: bump the epoch so a late reply is dropped.
    pub fn cancel_request(&mut self) {
        self.gen_epoch = self.gen_epoch.wrapping_add(1);
        self.in_flight = false;
        self.regenerating = false;
    }

    /// Whether a worker reply tagged with `epoch` is still the one we await.
    pub fn accepts(&self, epoch: u64) -> bool {
        epoch == self.gen_epoch
    }

    /// Tear down the whole brief session — drop history, pending, in-flight work.
    pub fn reset_session(&mut self) {
        self.cancel_request();
        self.iterations.clear();
        self.pending = None;
        self.base_idx = 0;
        self.region = Region::Tree;
        self.cursor = 0;
        self.error = None;
    }

    /// The committed brief currently shown (the BASE iteration).
    pub fn base(&self) -> Option<&TuiBrief> {
        self.iterations.get(self.base_idx).map(|i| &i.brief)
    }

    /// The working copy whose knobs the user is tuning.
    pub fn working(&self) -> Option<&TuiBrief> {
        self.pending.as_ref()
    }

    /// Number of root knobs whose pending value differs from the committed take.
    pub fn dirty_count(&self) -> u8 {
        let (Some(base), Some(pending)) = (self.base(), self.pending.as_ref()) else {
            return 0;
        };
        let mut n = 0u8;
        for a in 0..3 {
            for prop in Prop::ALL {
                if knob_of(&base.topics[a], prop) != knob_of(&pending.topics[a], prop) {
                    n = n.saturating_add(1);
                }
            }
        }
        n
    }

    /// Whether the given root knob differs from the committed take.
    pub fn knob_changed(&self, angle: usize, prop: Prop) -> bool {
        let (Some(base), Some(pending)) = (self.base(), self.pending.as_ref()) else {
            return false;
        };
        knob_of(&base.topics[angle], prop) != knob_of(&pending.topics[angle], prop)
    }

    /// The derived lifecycle state.
    pub fn brief_state(&self) -> BriefState {
        if self.regenerating {
            return BriefState::Regenerating;
        }
        match self.dirty_count() {
            0 => BriefState::InSync,
            n => BriefState::Stale(n),
        }
    }

    /// Append a freshly generated brief, making it the new BASE + pending.
    pub fn accept_brief(&mut self, brief: TuiBrief) {
        let (parent, label) = if self.iterations.is_empty() {
            (None, "first pass".to_string())
        } else {
            (Some(self.base_idx), self.delta_label(&brief))
        };
        self.iterations
            .push(Iteration::new(brief.clone(), parent, label));
        self.base_idx = self.iterations.len() - 1;
        self.pending = Some(brief);
        self.regenerating = false;
        self.region = Region::Tree;
        self.hist_cursor = self.base_idx;
        self.peek = None;
    }

    /// Restore an earlier iteration as the new BASE (fork — later takes survive).
    pub fn restore(&mut self, idx: usize) {
        if let Some(it) = self.iterations.get(idx) {
            self.base_idx = idx;
            self.pending = Some(it.brief.clone());
            self.region = Region::Tree;
            self.peek = None;
        }
    }

    /// Drop pending knob edits back to the committed values.
    pub fn revert_pending(&mut self) {
        if let Some(base) = self.base().cloned() {
            self.pending = Some(base);
        }
    }

    /// The angle the focused knob belongs to.
    pub fn focus_angle(&self) -> usize {
        (self.cursor / 3).min(2) as usize
    }

    /// The focused knob.
    pub fn focus_knob(&self) -> Prop {
        Prop::ALL[(self.cursor % 3) as usize]
    }

    /// Move the knob cursor one step toward APPLIED of the last angle.
    pub fn cursor_down(&mut self) {
        self.cursor = (self.cursor + 1).min(8);
    }

    /// Move the knob cursor one step toward DEPTH of the first angle.
    pub fn cursor_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    /// Jump the cursor to the DEPTH knob of angle `a` and leave adjust mode.
    pub fn cursor_to_angle(&mut self, a: u8) {
        self.cursor = a.min(2).saturating_mul(3);
    }

    /// Adjust the focused root knob by `delta`, clamped to 1..=5.
    pub fn adjust_knob(&mut self, delta: i32) {
        let angle = self.focus_angle();
        let knob = self.focus_knob();
        if let Some(pending) = self.pending.as_mut() {
            let v = knob_mut(&mut pending.topics[angle], knob);
            let next = (*v as i32 + delta).clamp(1, 5) as u8;
            *v = next;
        }
    }

    /// Compose a short label describing how `next` differs from the current BASE.
    fn delta_label(&self, next: &TuiBrief) -> String {
        let Some(base) = self.base() else {
            return "re-tuned".to_string();
        };
        let mut up = Vec::new();
        let mut down = Vec::new();
        for a in 0..3 {
            for prop in Prop::ALL {
                let b = knob_of(&base.topics[a], prop);
                let n = knob_of(&next.topics[a], prop);
                if n > b {
                    up.push(prop);
                } else if n < b {
                    down.push(prop);
                }
            }
        }
        let word = |p: Prop, up: bool| match (p, up) {
            (Prop::Depth, true) => "deeper",
            (Prop::Depth, false) => "lighter",
            (Prop::Novelty, true) => "newer",
            (Prop::Novelty, false) => "settled",
            (Prop::Applied, true) => "more applied",
            (Prop::Applied, false) => "more why",
        };
        let mut parts = Vec::new();
        for p in Prop::ALL {
            if up.contains(&p) {
                parts.push(word(p, true));
            } else if down.contains(&p) {
                parts.push(word(p, false));
            }
        }
        if parts.is_empty() {
            "re-tuned".to_string()
        } else {
            parts.join(" · ")
        }
    }
}

/// Read a root's knob value.
pub fn knob_of(root: &crate::brief::TuiRoot, prop: Prop) -> u8 {
    match prop {
        Prop::Depth => root.depth,
        Prop::Novelty => root.novelty,
        Prop::Applied => root.applied,
    }
}

/// Mutable handle to a root's knob value.
pub fn knob_mut(root: &mut crate::brief::TuiRoot, prop: Prop) -> &mut u8 {
    match prop {
        Prop::Depth => &mut root.depth,
        Prop::Novelty => &mut root.novelty,
        Prop::Applied => &mut root.applied,
    }
}

/// Result of handling a key — the main loop turns this into a state change.
#[derive(Clone, Debug)]
pub enum Action {
    Stay,
    Go(Screen),
    Back,
    RequestBrief,
    Regenerate,
    Spawn,
    Quit,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brief::mock;

    fn loaded() -> State {
        let mut s = State::new(true);
        s.accept_brief(mock::sample("Quantum computing", "English"));
        s.cursor = 0;
        s
    }

    #[test]
    fn the_first_brief_appends_exactly_one_take() {
        let s = loaded();
        assert_eq!(1, s.iterations.len(), "first brief did not append one take");
    }

    #[test]
    fn the_first_take_is_in_sync() {
        let s = loaded();
        assert_eq!(
            BriefState::InSync,
            s.brief_state(),
            "a freshly accepted brief was not in sync"
        );
    }

    #[test]
    fn the_adjusting_a_knob_makes_the_brief_stale() {
        let mut s = loaded();
        s.adjust_knob(-1);
        assert_eq!(
            BriefState::Stale(1),
            s.brief_state(),
            "adjusting a knob did not mark the brief stale"
        );
    }

    #[test]
    fn the_reverting_pending_returns_to_in_sync() {
        let mut s = loaded();
        s.adjust_knob(-1);
        s.revert_pending();
        assert_eq!(
            BriefState::InSync,
            s.brief_state(),
            "reverting pending edits did not return to in sync"
        );
    }

    #[test]
    fn the_knob_value_never_leaves_the_one_to_five_range() {
        let mut s = loaded();
        for _ in 0..20 {
            s.adjust_knob(1);
        }
        let v = knob_of(&s.pending.as_ref().unwrap().topics[0], Prop::Depth);
        assert_eq!(5, v, "knob escaped the upper bound");
    }

    #[test]
    fn the_cancelled_request_reply_is_rejected() {
        let mut s = State::new(true);
        let epoch = s.start_request();
        s.cancel_request();
        assert!(
            !s.accepts(epoch),
            "a cancelled request's late reply was still accepted"
        );
    }

    #[test]
    fn the_restore_preserves_later_takes() {
        let mut s = loaded();
        s.adjust_knob(-1);
        s.accept_brief(mock::retune(s.pending.as_ref().unwrap()));
        s.restore(0);
        assert_eq!(
            2,
            s.iterations.len(),
            "restoring an earlier take truncated the history"
        );
    }
}
