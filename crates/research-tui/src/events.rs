//! Global hotkey routing — intercepts before per-screen dispatch.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::state::{Action, EditKind, Guarded, InputFocus, Screen, State};

/// Try to handle as a global hotkey. Returns `Some(action)` if consumed.
pub fn handle_global(state: &mut State, key: KeyEvent) -> Option<Action> {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Some(Action::Quit);
    }
    if state.confirm.is_some() {
        return Some(handle_confirm(state, key));
    }
    if state.editing.is_some() {
        return Some(handle_edit(state, key));
    }
    if state.help_open {
        if matches!(key.code, KeyCode::Char('?') | KeyCode::Esc) {
            state.help_open = false;
        }
        return Some(Action::Stay);
    }
    match key.code {
        KeyCode::F(1) => {
            state.sessions_idx = 0;
            Some(Action::Go(Screen::Sessions))
        }
        KeyCode::F(2) => {
            state.keys_idx = 0;
            Some(Action::Go(Screen::Keys))
        }
        KeyCode::Char('?')
            if !(state.screen == Screen::Input && state.input_focus == InputFocus::Topic) =>
        {
            state.help_open = true;
            Some(Action::Stay)
        }
        _ => None,
    }
}

fn handle_confirm(state: &mut State, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
            let action = state.confirm.take().map(|c| c.action);
            match action {
                Some(Guarded::BackToInput) => {
                    state.reset_session();
                    state.nav.clear();
                    state.screen = Screen::Input;
                    Action::Stay
                }
                Some(Guarded::Quit) => Action::Quit,
                Some(Guarded::RestoreOver(idx)) => {
                    state.revert_pending();
                    state.restore(idx);
                    Action::Stay
                }
                None => Action::Stay,
            }
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            state.confirm = None;
            Action::Stay
        }
        _ => Action::Stay,
    }
}

fn handle_edit(state: &mut State, key: KeyEvent) -> Action {
    let Some(edit) = state.editing.as_mut() else {
        return Action::Stay;
    };
    match key.code {
        KeyCode::Esc => {
            state.editing = None;
        }
        KeyCode::Enter => {
            commit_edit(state);
        }
        KeyCode::Backspace => {
            edit.draft.pop();
        }
        KeyCode::Char(c) => {
            edit.draft.push(c);
        }
        _ => {}
    }
    Action::Stay
}

fn commit_edit(state: &mut State) {
    let Some(edit) = state.editing.take() else {
        return;
    };
    let angle = state.focus_angle();
    let base_idx = state.base_idx;
    let draft = edit.draft;
    for brief in [
        state.iterations.get_mut(base_idx).map(|i| &mut i.brief),
        state.pending.as_mut(),
    ]
    .into_iter()
    .flatten()
    {
        match edit.kind {
            EditKind::Root => brief.topics[angle].title = draft.clone(),
            EditKind::Intent => brief.intent = draft.clone(),
            EditKind::Topic => brief.topic = draft.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    #[test]
    fn the_topic_field_keeps_a_question_mark_from_the_help_hotkey() {
        let mut s = State::new(true);
        s.screen = Screen::Input;
        s.input_focus = InputFocus::Topic;
        let consumed = handle_global(
            &mut s,
            KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE),
        );
        assert!(
            consumed.is_none(),
            "the global help hotkey swallowed a topic-field question mark"
        );
    }

    #[test]
    fn the_help_hotkey_opens_help_off_the_input_screen() {
        let mut s = State::new(true);
        s.screen = Screen::Tune;
        let _ = handle_global(
            &mut s,
            KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE),
        );
        assert!(
            s.help_open,
            "the help hotkey did not open help on a non-input screen"
        );
    }
}
