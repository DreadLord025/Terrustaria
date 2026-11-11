//! The `UserInterface` pointer machine (`Terraria/UI/UserInterface.cs`): the
//! per-button caches that turn raw press/release edges into `MouseDown`/`MouseUp`/
//! click/`LeftDoubleClick`, the 500 ms double-click window and the 200 ms
//! state-change click disable.
//!
//! Screen-model ruling (plan §"Chosen Approach"): this is a **pure** port — no
//! `UserInterface` object, no history/`GoBack` (dead code in 1.4.5.8: zero call
//! sites), no gamepad. The machine holds the *timing* half of the C# and is fed
//! element ids by its callers, so the screens keep drawing and hit-testing
//! themselves (the immediate-mode shape of the rest of the `ui` kit).
//!
//! Two call paths share this code:
//! * the in-game path is frame-ticked — `app/frame.rs` advances the clock and the
//!   handlers, which is the C#'s `Update` (`:190-228`);
//! * the front screens (`Main.menuMode == 888`) are event-driven — `app/loop.rs`
//!   hands the machine the press/release as they arrive, like the C# reads
//!   `Main.mouseLeft` inside `Update`'s `HandleClick` (`:214-218`).
//!
//! The clock is `App::anim_time` in milliseconds. `TR_SHOT` pins the frame step to
//! 1/60 (`app/frame.rs:92-104`), so the machine stays deterministic across a
//! screenshot double-run — the C#'s `GameTime.TotalGameTime.TotalMilliseconds`
//! analogue.

/// `UserInterface.cs:41` — `private const double DOUBLE_CLICK_TIME = 500.0;`.
pub const DOUBLE_CLICK_MS: f64 = 500.0;

/// `UserInterface.cs:43` — `private const double STATE_CHANGE_CLICK_DISABLE_TIME = 200.0;`.
pub const STATE_CHANGE_CLICK_DISABLE_MS: f64 = 200.0;

/// One of the two pointer caches (`UserInterface.InputPointerCache`, `:15-39`).
/// The element identity the C# holds as object references is a `u32` id here —
/// the screens assign ids in Phase 2; `None` is the C#'s null `LastDown`/
/// `LastClicked`/`mouseElement`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerCache {
    /// `InputPointerCache.WasDown` (`:17`) — the button's state at the previous
    /// observation, which makes the press/release edges (`HandleClick`'s
    /// `isDown && !cache.WasDown`, `:232`).
    pub(crate) was_down: bool,
    /// `InputPointerCache.LastDown` (`:19`) — the element that took the press.
    pub(crate) last_down: Option<u32>,
    /// `InputPointerCache.LastClicked` (`:21`) — the last *clicked* element, the
    /// double-click comparison (`:236`).
    pub(crate) last_clicked: Option<u32>,
    /// `InputPointerCache.LastTimeDown` (`:15`) — ms of the last press.
    pub(crate) last_time_down_ms: f64,
}

impl Default for PointerCache {
    fn default() -> Self {
        Self {
            was_down: false,
            last_down: None,
            last_clicked: None,
            last_time_down_ms: f64::NEG_INFINITY,
        }
    }
}

impl PointerCache {
    /// `InputPointerCache.Clear()` (`:33-38`) — drops `LastClicked`/`LastDown` and
    /// resets `LastTimeDown`. `WasDown` is *not* touched here: the C#'s
    /// `ImmediatelyUpdateInputPointers` (`:159-163`) re-syncs it from the live
    /// buttons right after, which is what `PointerState::reset_on_state_change`
    /// ports.
    pub(crate) fn clear(&mut self) {
        self.last_clicked = None;
        self.last_down = None;
        self.last_time_down_ms = f64::NEG_INFINITY;
    }
}

/// The mouse button — the C#'s `LeftMouse`/`RightMouse` caches (`:53-91`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerButton {
    Left,
    Right,
}

/// What the machine produced for one observation (`HandleClick`'s four
/// delegates, `UserInterface.cs:55-70`): `MouseDownEvent`, `MouseUpEvent`,
/// `ClickEvent` or `DoubleClickEvent`. Only one is produced per edge, exactly as
/// the C# fires at most one delegate pair per `HandleClick` call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerEvent {
    /// `element.LeftMouseDown/RghtMouseDown(evt)` (`:55-58`, `:75-79`).
    Down,
    /// `element.LeftMouseUp/RghtMouseUp(evt)` (`:59-62`, `:80-83`).
    Up,
    /// `element.LeftClick/RghtClick(evt)` — the release inside the down element
    /// (`:249`).
    Click,
    /// `element.LeftDoubleClick/RghtDoubleClick(evt)` (`:238`).
    DoubleClick,
}

/// The whole machine (`UserInterface`'s pointer half). The `CurrentState`, hover
/// tracking and visibility live with the callers; this is only what turns buttons
/// and element ids into `PointerEvent`s.
#[derive(Clone, Debug)]
pub struct PointerState {
    left: PointerCache,
    right: PointerCache,
    /// `_clickDisabledTimeRemaining` (`:97`) — ms left of the state-change
    /// lockout.
    click_disabled_ms: f64,
}

impl Default for PointerState {
    fn default() -> Self {
        Self {
            left: PointerCache::default(),
            right: PointerCache::default(),
            click_disabled_ms: 0.0,
        }
    }
}

impl PointerState {
    /// `UserInterface.cs:203-204` — the lockout countdown, in ms, before the
    /// handles that consume it.
    pub fn tick(&mut self, dt_ms: f64) {
        self.click_disabled_ms = (self.click_disabled_ms - dt_ms).max(0.0);
    }

    /// `Update`'s `flag2` (`:204`) — the click lockout is live.
    /// The left button's held state (`UserInterface`'s `MouseLeft`) — the
    /// window's bar drag reads it while the lock is armed (`:424`).
    pub fn left_held(&self) -> bool {
        self.left.was_down
    }

    pub fn clicks_disabled(&self) -> bool {
        self.click_disabled_ms > 0.0
    }

    /// `UserInterface.cs:165-179` — `ResetState`, the `SetState` tail. `held` is
    /// `ImmediatelyUpdateInputPointers`'s snapshot of the two buttons (`:159-163`):
    /// a button held across the switch must not look like a fresh press edge on
    /// the next observation, so `WasDown` is re-synced **before** the caches are
    /// cleared and the lockout armed.
    pub fn reset_on_state_change(&mut self, held: [bool; 2]) {
        self.left.was_down = held[0];
        self.right.was_down = held[1];
        self.left.clear();
        self.right.clear();
        self.click_disabled_ms = self.click_disabled_ms.max(STATE_CHANGE_CLICK_DISABLE_MS);
    }

    /// The press edge of `HandleClick` (`UserInterface.cs:232-242`): the
    /// `MouseDown` delegate, the double-click test and the `LastTimeDown` write.
    /// `now_ms` is the clock, `hit` the element under the pointer (`None` = no
    /// element, the C#'s `mouseElement == null` guard at `:232`).
    pub fn press(
        &mut self,
        now_ms: f64,
        button: PointerButton,
        hit: Option<u32>,
    ) -> Option<PointerEvent> {
        let cache = match button {
            PointerButton::Left => &mut self.left,
            PointerButton::Right => &mut self.right,
        };
        // `if (isDown && !cache.WasDown && mouseElement != null)` (`:232`).
        if !hit.is_some() {
            cache.was_down = true;
            return None;
        }
        let element = hit.expect("checked above");
        // `if (cache.LastClicked == mouseElement && time - cache.LastTimeDown < 500)` (`:236`).
        let double = cache.last_clicked == Some(element)
            && now_ms - cache.last_time_down_ms < DOUBLE_CLICK_MS;
        cache.last_down = Some(element);
        if double {
            // `cache.DoubleClickEvent(...); cache.LastClicked = null;` (`:238-239`).
            cache.last_clicked = None;
            cache.was_down = true;
            cache.last_time_down_ms = now_ms;
            return Some(PointerEvent::DoubleClick);
        }
        // `cache.MouseDownEvent(...)` (`:235`) + `cache.LastTimeDown = time` (`:241`).
        cache.was_down = true;
        cache.last_time_down_ms = now_ms;
        Some(PointerEvent::Down)
    }

    /// The release edge of `HandleClick` (`UserInterface.cs:243-253`): the click is
    /// produced only when the pointer is still inside the element that took the
    /// press (`lastDown.ContainsPoint(this.MousePosition)`, `:246`), then the
    /// `MouseUp` delegate and `LastDown = null`.
    pub fn release(
        &mut self,
        now_ms: f64,
        button: PointerButton,
        hit: Option<u32>,
    ) -> Option<PointerEvent> {
        let cache = match button {
            PointerButton::Left => &mut self.left,
            PointerButton::Right => &mut self.right,
        };
        // `else if (!isDown && cache.WasDown && cache.LastDown != null)` (`:243`).
        if cache.last_down.is_none() {
            cache.was_down = false;
            return None;
        }
        let last_down = cache.last_down.expect("checked above");
        // `if (lastDown.ContainsPoint(this.MousePosition))` (`:246`) — the port's
        // analogue is the element still under the pointer.
        let clicked = hit == Some(last_down);
        if clicked {
            // `cache.ClickEvent(...); cache.LastClicked = cache.LastDown;` (`:248-249`).
            cache.last_clicked = cache.last_down;
        }
        let _ = now_ms;
        cache.last_down = None;
        cache.was_down = false;
        if clicked {
            Some(PointerEvent::Click)
        } else {
            Some(PointerEvent::Up)
        }
    }

    /// `MouseCaptured` (`UserInterface.cs:113-124`): a press is still held down
    /// (either button), which the C#'s input-routing guards read.
    pub fn mouse_captured(&self) -> bool {
        self.left.was_down || self.right.was_down
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Expected values hand-derived from `UserInterface.cs:230-255` / `:165-179` /
    // `:41,43` — never from this code. The clock is ms like the C#'s
    // `GameTime.TotalGameTime.TotalMilliseconds`; `now = 1000` is the "now" the C#
    // reads from its `time` argument.

    /// One full click: press at `t`, release at the same element at `t + 50`.
    fn click(machine: &mut PointerState, button: PointerButton, t: f64, hit: u32) {
        let down = machine.press(t, button, Some(hit));
        assert_eq!(down, Some(PointerEvent::Down));
        let up = machine.release(t + 50.0, button, Some(hit));
        assert_eq!(up, Some(PointerEvent::Click));
    }

    #[test]
    fn dw_1_1_press_fires_mouse_down_and_a_release_inside_fires_click() {
        let mut m = PointerState::default();
        // `HandleClick`'s press arm (`:232-242`): `LastDown = element`,
        // `MouseDownEvent`, `LastTimeDown = now`.
        assert_eq!(
            m.press(1000.0, PointerButton::Left, Some(7)),
            Some(PointerEvent::Down)
        );
        // The release arm (`:243-253`): the pointer still inside element 7 → click.
        assert_eq!(
            m.release(1050.0, PointerButton::Left, Some(7)),
            Some(PointerEvent::Click)
        );
    }

    #[test]
    fn dw_1_1_a_second_press_within_500ms_is_a_double_click_and_clears_last_clicked() {
        let mut m = PointerState::default();
        click(&mut m, PointerButton::Left, 1000.0, 7);
        // The second press at 1000 + 300 ms on the same element: the C# fires
        // DoubleClickEvent and clears `LastClicked` (`:236-240`).
        assert_eq!(
            m.press(1300.0, PointerButton::Left, Some(7)),
            Some(PointerEvent::DoubleClick)
        );
        // `LastClicked = null` → a third press is a plain MouseDown again.
        assert_eq!(
            m.press(1400.0, PointerButton::Left, Some(7)),
            Some(PointerEvent::Down)
        );
    }

    #[test]
    fn dw_1_1_the_same_pair_past_500ms_is_click_then_down() {
        let mut m = PointerState::default();
        click(&mut m, PointerButton::Left, 1000.0, 7);
        // `time - cache.LastTimeDown < 500.0` (`:236`) — 600 ms past the press.
        assert_eq!(
            m.press(1600.0, PointerButton::Left, Some(7)),
            Some(PointerEvent::Down)
        );
    }

    #[test]
    fn dw_1_1_double_click_requires_the_same_element() {
        let mut m = PointerState::default();
        click(&mut m, PointerButton::Left, 1000.0, 7);
        // A press on a different element inside the window: `LastClicked != element`
        // (`:236`) → a plain MouseDown, and no double-click.
        assert_eq!(
            m.press(1200.0, PointerButton::Left, Some(9)),
            Some(PointerEvent::Down)
        );
    }

    #[test]
    fn dw_1_2_release_outside_the_down_element_fires_mouse_up_and_no_click() {
        let mut m = PointerState::default();
        assert_eq!(
            m.press(1000.0, PointerButton::Left, Some(7)),
            Some(PointerEvent::Down)
        );
        // The pointer moved to element 9 before the release: `ContainsPoint` is
        // false (`:246`) → `MouseUpEvent` only, `LastDown = null`.
        assert_eq!(
            m.release(1050.0, PointerButton::Left, Some(9)),
            Some(PointerEvent::Up)
        );
        // And no click is remembered, so the next press on 9 is a MouseDown.
        assert_eq!(
            m.press(1100.0, PointerButton::Left, Some(9)),
            Some(PointerEvent::Down)
        );
    }

    #[test]
    fn dw_1_2_the_state_change_disable_swallows_the_next_press() {
        let mut m = PointerState::default();
        // `ResetState` (`:178`) arms the 200 ms lockout on a state change.
        m.reset_on_state_change([false, false]);
        assert!(m.clicks_disabled());
        // `HandleClick` is not called at all while `_clickDisabledTimeRemaining > 0`
        // (`:214-218`) — the caller must gate on `clicks_disabled()`.
        // After the countdown the press passes again.
        m.tick(200.0);
        assert!(!m.clicks_disabled());
        assert_eq!(
            m.press(1200.0, PointerButton::Left, Some(3)),
            Some(PointerEvent::Down)
        );
    }

    #[test]
    fn dw_1_2_the_right_button_behaves_like_the_left() {
        let mut m = PointerState::default();
        assert_eq!(
            m.press(1000.0, PointerButton::Right, Some(4)),
            Some(PointerEvent::Down)
        );
        assert_eq!(
            m.release(1050.0, PointerButton::Right, Some(4)),
            Some(PointerEvent::Click)
        );
        assert_eq!(
            m.press(1200.0, PointerButton::Right, Some(4)),
            Some(PointerEvent::DoubleClick)
        );
        // The left cache is untouched by the right button's traffic.
        assert_eq!(m.left.last_down, None);
        assert_eq!(m.left.last_clicked, None);
    }

    #[test]
    fn dw_1_3_reset_state_clears_a_stale_was_down() {
        let mut m = PointerState::default();
        // A button held across a `SetState`: the press already happened on the old
        // screen, so `ImmediatelyUpdateInputPointers` (`:159-163`) syncs
        // `WasDown = true` and `ClearPointers` drops `LastDown`.
        m.reset_on_state_change([true, false]);
        // The very next observation is a *release* (`HandleClick`'s else-if,
        // `:243`) whose `LastDown == null` → nothing fires, and `WasDown` goes
        // false, so the following real press is a fresh edge.
        assert_eq!(m.release(1000.0, PointerButton::Left, None), None);
        assert_eq!(
            m.press(1100.0, PointerButton::Left, Some(5)),
            Some(PointerEvent::Down)
        );
    }

    #[test]
    fn dw_1_3_the_disable_countdown_decays_to_zero_and_clamps() {
        let mut m = PointerState::default();
        m.reset_on_state_change([false, false]);
        m.tick(150.0);
        assert!(m.clicks_disabled());
        // `Math.Max(0.0, remaining - elapsed)` (`:203`) — never negative.
        m.tick(150.0);
        assert!(!m.clicks_disabled());
        m.tick(1000.0);
        assert!(!m.clicks_disabled());
    }

    #[test]
    fn a_press_with_no_element_under_the_pointer_fires_nothing() {
        let mut m = PointerState::default();
        // `isDown && !cache.WasDown && mouseElement != null` (`:232`) — the C#'
        // `GetElementAt` miss. `WasDown` is still synced so the release stays an
        // edge-less no-op.
        assert_eq!(m.press(1000.0, PointerButton::Left, None), None);
        assert_eq!(m.release(1050.0, PointerButton::Left, None), None);
        assert!(!m.mouse_captured());
    }

    #[test]
    fn mouse_captured_tracks_any_held_button() {
        let mut m = PointerState::default();
        assert_eq!(
            m.press(1000.0, PointerButton::Left, Some(1)),
            Some(PointerEvent::Down)
        );
        assert!(m.mouse_captured());
        m.release(1050.0, PointerButton::Left, Some(1));
        // `MouseCaptured` reads `WasDown` (`:113-124`) — never latched true.
        assert!(!m.mouse_captured());
    }

    #[test]
    fn the_double_click_window_boundary_is_exclusive() {
        let mut m = PointerState::default();
        click(&mut m, PointerButton::Left, 1000.0, 2);
        // Exactly 500 ms is *not* inside the window (`< 500.0`, `:236`).
        assert_eq!(
            m.press(1500.0, PointerButton::Left, Some(2)),
            Some(PointerEvent::Down)
        );
        // One ms inside is.
        let mut n = PointerState::default();
        click(&mut n, PointerButton::Left, 1000.0, 2);
        assert_eq!(
            n.press(1499.0, PointerButton::Left, Some(2)),
            Some(PointerEvent::DoubleClick)
        );
    }
}
