//! The `UserInterface` state stack — `_currentState` + `_history`,
//! `SetState`/`GoBack`/`AddToHistory` (`Terraria/UI/UserInterface.cs:276-322`).
//!
//! Screen-model ruling: the immediate screens keep their own per-screen state;
//! this ports the *stack machine* as pure code over screen ids (`u32`, the same
//! identity space as `framework::Element`/`pointer`), so `app/flow.rs`'s
//! `next_flow_state` table can grow a faithful `GoBack` without a retained tree.
//!
//! Verified 2026-11-20: `UserInterface.GoBack` has **zero call sites** in
//! 1.4.5.8 — every `GoBack()` grep hit is a screen's own private helper
//! (`UIWorldCreation.cs:670-674` plays sound 11 and calls
//! `Main.OpenWorldSelectUI()`; same shape in `UICharacterCreation` and
//! `UIWorldCreationAdvanced`), not this method. The machine is ported for
//! completeness (research 18 deviation 9); no screen relies on the 2-deep pop.

/// `UserInterface.SetState`'s observable side effects, in the C#'s order
/// (`:276-303`) — the caller runs them against its screens/pointer machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateChange {
    /// `:288-291` — `_lastElementHover.MouseOut(evt)` when a hover was live.
    MouseOutLastHover,
    /// `:292` — the old state's `Deactivate()`.
    Deactivate(u32),
    /// `:295` — `ResetState()` (the pointer machine's
    /// `reset_on_state_change`, the 200 ms lockout).
    ResetPointer,
    /// `:299` — the new state's `Activate()`.
    Activate(u32),
    /// `:300` — the new state's `Recalculate()` (and `:298` `_isStateDirty`).
    Recalculate(u32),
}

/// `UserInterface`'s state half: `_currentState` (`:99`) and `_history` (`:101`).
/// Ids stand in for `UIState` references; `None` is the C#'s null state.
#[derive(Clone, Debug, Default)]
pub struct StateStack {
    current: Option<u32>,
    history: Vec<u32>,
}

impl StateStack {
    /// `_currentState`.
    pub fn current(&self) -> Option<u32> {
        self.current
    }

    /// `IsVisible = _currentState != null` (`:302`).
    pub fn is_visible(&self) -> bool {
        self.current.is_some()
    }

    /// `IsElementUnderMouse`'s state half (`:349-356`): visible and the hover is
    /// not the root `UIState` itself. `hover_is_state` carries the C#'s
    /// `_lastElementHover is UIState` test; `None` hover is false (`:351`).
    pub fn is_element_under_mouse(&self, hover: Option<u32>, hover_is_state: bool) -> bool {
        self.is_visible() && hover.is_some() && !hover_is_state
    }

    /// `UserInterface.SetState` (`:276-303`). Returns the side effects in order;
    /// a no-op (`state == _currentState`, `:278-281`) returns an empty list and
    /// touches nothing — including the history.
    pub fn set_state(&mut self, state: Option<u32>) -> Vec<StateChange> {
        if state == self.current {
            return Vec::new();
        }
        let mut effects = Vec::new();
        // `:282-285` — a non-null incoming state is appended to the history
        // BEFORE the old state deactivates.
        if let Some(s) = state {
            self.add_to_history(s);
        }
        // `:286-293` — the old state's MouseOut-then-Deactivate.
        if let Some(old) = self.current {
            effects.push(StateChange::MouseOutLastHover);
            effects.push(StateChange::Deactivate(old));
        }
        self.current = state;
        // `:295` — ResetState runs for null transitions too.
        effects.push(StateChange::ResetPointer);
        // `:296-301` — Activate + Recalculate only for a non-null state.
        if let Some(s) = state {
            effects.push(StateChange::Activate(s));
            effects.push(StateChange::Recalculate(s));
        }
        effects
    }

    /// `UserInterface.GoBack` (`:305-313`): with ≥2 history entries, take the
    /// second-from-last, drop the last **two**, then `SetState` it — which
    /// re-appends it (`:284`), so the net effect is "pop one". Returns the
    /// `SetState` effects, or an empty list when the history is too shallow
    /// (the C# does nothing then).
    pub fn go_back(&mut self) -> Vec<StateChange> {
        if self.history.len() >= 2 {
            let state = self.history[self.history.len() - 2];
            let n = self.history.len();
            self.history.truncate(n - 2);
            self.set_state(Some(state))
        } else {
            Vec::new()
        }
    }

    /// `UserInterface.AddToHistory` (`:315-322`): append, and when the count
    /// exceeds 32 drop the **first four** (`RemoveRange(0, 4)`) — the history
    /// sawtooths 33 → 29, it is not a sliding 32-window.
    fn add_to_history(&mut self, state: u32) {
        self.history.push(state);
        if self.history.len() > 32 {
            self.history.drain(0..4);
        }
    }

    /// The history contents (test/inspection; the C# field is private).
    pub fn history(&self) -> &[u32] {
        &self.history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHAR_SELECT: u32 = 1;
    const CHAR_CREATE: u32 = 2;
    const WORLD_SELECT: u32 = 3;

    #[test]
    fn set_state_orders_history_hover_deactivate_reset_activate_recalculate() {
        // `UserInterface.cs:276-303` order, observed through the effect list.
        let mut stack = StateStack::default();
        assert!(stack.set_state(Some(CHAR_SELECT)).iter().all(|e| matches!(
            e,
            StateChange::ResetPointer | StateChange::Activate(_) | StateChange::Recalculate(_)
        )));
        let effects = stack.set_state(Some(CHAR_CREATE));
        assert_eq!(
            effects,
            vec![
                StateChange::MouseOutLastHover,
                StateChange::Deactivate(CHAR_SELECT),
                StateChange::ResetPointer,
                StateChange::Activate(CHAR_CREATE),
                StateChange::Recalculate(CHAR_CREATE),
            ]
        );
        assert_eq!(stack.history(), &[CHAR_SELECT, CHAR_CREATE]);
        assert_eq!(stack.current(), Some(CHAR_CREATE));
        assert!(stack.is_visible());
    }

    #[test]
    fn set_state_to_the_same_state_is_a_no_op_including_history() {
        // `:278-281` — early return before AddToHistory.
        let mut stack = StateStack::default();
        stack.set_state(Some(CHAR_SELECT));
        assert!(stack.set_state(Some(CHAR_SELECT)).is_empty());
        assert_eq!(stack.history(), &[CHAR_SELECT]);
    }

    #[test]
    fn set_state_null_deactivates_and_resets_but_does_not_activate() {
        // `:282` skips the history append; `:295` still resets; `:302` hides.
        let mut stack = StateStack::default();
        stack.set_state(Some(CHAR_SELECT));
        let effects = stack.set_state(None);
        assert_eq!(
            effects,
            vec![
                StateChange::MouseOutLastHover,
                StateChange::Deactivate(CHAR_SELECT),
                StateChange::ResetPointer,
            ]
        );
        assert!(!stack.is_visible());
        assert_eq!(stack.history(), &[CHAR_SELECT]);
    }

    #[test]
    fn go_back_pops_two_and_reappends_one() {
        // `:305-313` + `:284`: A, B, C -> GoBack lands on B with history A, B.
        let mut stack = StateStack::default();
        stack.set_state(Some(CHAR_SELECT));
        stack.set_state(Some(CHAR_CREATE));
        stack.set_state(Some(WORLD_SELECT));
        let effects = stack.go_back();
        assert_eq!(stack.current(), Some(CHAR_CREATE));
        assert_eq!(stack.history(), &[CHAR_SELECT, CHAR_CREATE]);
        assert!(effects.contains(&StateChange::Deactivate(WORLD_SELECT)));
        assert!(effects.contains(&StateChange::Activate(CHAR_CREATE)));
        // Shallower than 2: nothing happens (`:307`'s false arm).
        let mut shallow = StateStack::default();
        shallow.set_state(Some(CHAR_SELECT));
        assert!(shallow.go_back().is_empty());
        assert_eq!(shallow.current(), Some(CHAR_SELECT));
        assert_eq!(shallow.history(), &[CHAR_SELECT]);
    }

    #[test]
    fn go_back_to_the_current_state_still_pops_but_fires_nothing() {
        // A, B, A: GoBack targets history[len-2] = B (NOT A). `RemoveRange`
        // drops the last two, leaving [A], then SetState(B) re-appends it
        // (`UserInterface.cs:305-313`, `:282-284`) - B activates, so the
        // effects are not empty and the history ends [A, B].
        let mut stack = StateStack::default();
        stack.set_state(Some(CHAR_SELECT));
        stack.set_state(Some(CHAR_CREATE));
        stack.set_state(Some(CHAR_SELECT));
        let effects = stack.go_back();
        assert!(effects.contains(&StateChange::Deactivate(CHAR_SELECT)));
        assert!(effects.contains(&StateChange::Activate(CHAR_CREATE)));
        assert_eq!(stack.current(), Some(CHAR_CREATE));
        assert_eq!(stack.history(), &[CHAR_SELECT, CHAR_CREATE]);
    }

    #[test]
    fn the_history_sawtooths_33_to_29_not_a_sliding_window() {
        // `:318-321`: `if (Count > 32) RemoveRange(0, 4)` — at the 33rd append
        // the FIRST FOUR go, leaving 29, not the oldest one leaving 32.
        let mut stack = StateStack::default();
        for i in 0..32u32 {
            stack.set_state(Some(i + 1));
        }
        assert_eq!(stack.history().len(), 32);
        assert_eq!(stack.history()[0], 1);
        stack.set_state(Some(33));
        assert_eq!(stack.history().len(), 29);
        assert_eq!(stack.history()[0], 5);
        assert_eq!(stack.history()[28], 33);
    }

    #[test]
    fn is_element_under_mouse_needs_visible_non_root_hover() {
        // `:349-356`: hidden -> false; null hover -> false; a UIState hover
        // (the root) -> false; a real element hover -> true.
        let mut stack = StateStack::default();
        assert!(!stack.is_element_under_mouse(Some(7), false));
        stack.set_state(Some(CHAR_SELECT));
        assert!(!stack.is_element_under_mouse(None, false));
        assert!(!stack.is_element_under_mouse(Some(CHAR_SELECT), true));
        assert!(stack.is_element_under_mouse(Some(7), false));
    }
}
