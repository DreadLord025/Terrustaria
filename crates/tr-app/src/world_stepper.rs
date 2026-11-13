//! The testing step-frame stepper — `Terraria.Testing.WorldUpdateStepper`
//! (`WorldUpdateStepper.cs`, 58 lines) ported 1:1. While `paused` (the
//! `pauseworld` toggle, `ToolkitDebugCommands.cs:742-749` — the port surfaces
//! it as the debug panel's World-tab row), the world steps only on the step
//! keys' pacing: End/NumpadAdd/XButton1 held 12+ frames step every 2nd frame,
//! a fresh release steps once. The `Updates Since Paused` HUD overlay tracks
//! the pin.

use std::collections::HashSet;
use winit::keyboard::KeyCode;

/// `WorldUpdateStepper` (`WorldUpdateStepper.cs:7-58`). The field rides
/// [`App`](crate::app::App) as `stepper`; the step-key input state stays on
/// App (`keys` + `mouse_back_held` + the was-shift pair) and feeds
/// [`WorldUpdateStepper::should_update_world`].
pub(crate) struct WorldUpdateStepper {
    /// `Paused` (`:15`): the `pauseworld` toggle. While true the world steps
    /// only through the pacing below.
    pub(crate) paused: bool,
    /// `StartUpdateCount` (`:11`): the `game_update_count` pin taken by
    /// [`WorldUpdateStepper::toggle_paused`]; the HUD overlay diffs against it.
    pub(crate) start_update_count: u32,
    /// `StepHeldCount` (`:13`): the held-frames counter behind the 12+/÷2
    /// pacing. Mutates only inside [`WorldUpdateStepper::should_update_world`]
    /// — the C#'s early `return` from a paused `Update` (`Main.cs:17197-17199`)
    /// never reaches the `:17278` gate, so an in-game-paused frame freezes it.
    pub(crate) step_held_count: u32,
    /// `DrawnThisUpdate` (`:9`): false at a step's head (`Main.cs:17747`),
    /// true on the draw side (`Main.cs:62478`). Its C# consumer (the dust
    /// pass) is unported, so nothing reads it yet — the flag is still pinned
    /// 1:1 for the behaviour.
    pub(crate) drawn_this_update: bool,
}

impl WorldUpdateStepper {
    pub(crate) fn new() -> Self {
        Self {
            paused: false,
            start_update_count: 0,
            step_held_count: 0,
            drawn_this_update: false,
        }
    }

    /// `WorldUpdateStepper.ShouldUpdateWorld` (`WorldUpdateStepper.cs:17-37`):
    /// the `Main.cs:17278` gate's stepper half. `down`/`was_down` are this
    /// frame's and the previous frame's step-key reading
    /// ([`WorldUpdateStepper::get_step_key_state`] + App's shifted snapshot).
    pub(crate) fn should_update_world(&mut self, down: bool, was_down: bool) -> bool {
        // `if (!down) StepHeldCount = 0;` (`:20-23`) — runs before the
        // `Paused` check, so a released key clears the count even unpaused.
        if !down {
            self.step_held_count = 0;
        }
        // `if (!Paused) return true;` (`:24-27`).
        if !self.paused {
            return true;
        }
        // `if (down && ++StepHeldCount >= 12 && StepHeldCount % 2 == 0)`
        // (`:28-31`): held 12+ frames, every 2nd frame steps — held frames
        // 1..11 (and every odd held frame past 12) step nothing.
        if down && {
            self.step_held_count += 1;
            self.step_held_count >= 12 && self.step_held_count % 2 == 0
        } {
            return true;
        }
        // `if (wasDown && !down) return true;` (`:32-35`): a fresh release
        // steps exactly once.
        if was_down && !down {
            return true;
        }
        false
    }

    /// `WorldUpdateStepper.GetStepKeyState` (`WorldUpdateStepper.cs:39-43`)'s
    /// `down` half: End | NumpadAdd held (`Main.keyState.IsKeyDown`) ∪
    /// XButton1 (`PlayerInput.MouseInfo.XButton1`, the port's
    /// `mouse_back_held`). The `wasDown` half is App's shifted snapshot.
    pub(crate) fn get_step_key_state(keys: &HashSet<KeyCode>, mouse_back_held: bool) -> bool {
        // `Main.keyState.IsKeyDown(Keys.End) || Main.keyState.IsKeyDown(Keys.Add)`
        // (`:41`) — `Keys.Add` is the numpad plus, winit's `NumpadAdd`.
        keys.contains(&KeyCode::End) || keys.contains(&KeyCode::NumpadAdd) || mouse_back_held
    }

    /// `WorldUpdateStepper.TogglePaused` (`WorldUpdateStepper.cs:45-49`):
    /// flips `Paused` and pins `StartUpdateCount = Main.GameUpdateCount` —
    /// even mid-frame, even mid-hold.
    pub(crate) fn toggle_paused(&mut self, game_update_count: u32) {
        self.paused = !self.paused;
        self.start_update_count = game_update_count;
    }

    /// `WorldUpdateStepper.DrawHUD` (`WorldUpdateStepper.cs:51-57`)'s text:
    /// the `Updates Since Paused: N` border string while paused, `None`
    /// otherwise (the caller renders it bottom-right, `Main.cs:62405-62411`'s
    /// UIScale batch). `RecordReplay` is unported, so its `ReplayMode`
    /// guard drops with it. The count is the C#'s
    /// `(int)(GameUpdateCount - StartUpdateCount)` cast.
    pub(crate) fn hud_text(&self, game_update_count: u32) -> Option<String> {
        if !self.paused {
            return None;
        }
        let since = game_update_count.wrapping_sub(self.start_update_count) as i32;
        Some(format!("Updates Since Paused: {since}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bare stepper, unpaused, count pinned at 100 — the unit-test seam
    /// (the pacing needs no App; the frame-level tests drive the gate).
    fn stepper(paused: bool) -> WorldUpdateStepper {
        let mut s = WorldUpdateStepper::new();
        if paused {
            s.toggle_paused(100);
        }
        s
    }

    /// DW-2.1 — unpaused, every (down, wasDown) combination steps: the
    /// `if (!Paused) return true;` arm (`WorldUpdateStepper.cs:24-27`) makes
    /// the pacing dead weight while the world runs.
    #[test]
    fn s1_b_3_stepper_pacing_unpaused_steps_every_frame() {
        let mut s = stepper(false);
        for (down, was_down) in [(false, false), (true, false), (false, true), (true, true)] {
            assert!(
                s.should_update_world(down, was_down),
                "unpaused steps with down={down} wasDown={was_down}"
            );
        }
        // The early return leaves `StepHeldCount` untouched: a held key
        // while unpaused accumulates nothing (the count only moves in the
        // paused arms, `:28-31`).
        for _ in 0..20 {
            s.should_update_world(true, true);
        }
        assert_eq!(s.step_held_count, 0, "unpaused frames never count holds");
    }

    /// DW-2.1 — the watched boundary (`Main.cs` pacing, `:28-31`): while
    /// paused, held frames 1..11 step nothing and the 12th held frame steps.
    #[test]
    fn s1_b_3_stepper_pacing_boundary_11_no_12_yes() {
        let mut s = stepper(true);
        for held in 1..=11 {
            assert!(
                !s.should_update_world(true, true),
                "held frame {held} steps nothing (the 11-frame boundary)"
            );
        }
        assert!(
            s.should_update_world(true, true),
            "the 12th held frame steps (12 >= 12 && 12 % 2 == 0)"
        );
    }

    /// DW-2.1 — past the threshold the pacing steps every 2nd held frame
    /// (`StepHeldCount % 2 == 0`): 13 no, 14 yes, 15 no, 16 yes.
    #[test]
    fn s1_b_3_stepper_pacing_held_steps_every_2nd() {
        let mut s = stepper(true);
        for _ in 0..12 {
            s.should_update_world(true, true);
        }
        for held in 13..=16 {
            let expected = held % 2 == 0;
            assert_eq!(
                s.should_update_world(true, true),
                expected,
                "held frame {held} steps = {expected}"
            );
        }
    }

    /// DW-2.1 — a fresh release steps exactly once (`wasDown && !down`,
    /// `:32-35`): the release frame steps, the next released frame does not.
    #[test]
    fn s1_b_3_stepper_pacing_release_steps_once() {
        // Released from under the threshold too — the release edge steps
        // regardless of the count.
        let mut s = stepper(true);
        for _ in 0..3 {
            assert!(!s.should_update_world(true, true));
        }
        assert!(
            s.should_update_world(false, true),
            "the release frame steps once"
        );
        assert!(
            !s.should_update_world(false, false),
            "the frame after the release steps nothing"
        );

        // And from past the threshold (the usual step-then-hold-again flow).
        let mut s = stepper(true);
        for _ in 0..12 {
            s.should_update_world(true, true);
        }
        assert!(s.should_update_world(false, true), "the release steps");
        assert!(!s.should_update_world(false, false));
    }

    /// DW-2.1 edge — the held count resets on release (`:20-23` runs before
    /// the `Paused` check): after a release, a re-hold needs 12 fresh frames
    /// (the release frame itself still stepped).
    #[test]
    fn s1_b_3_stepper_pacing_released_key_resets_count() {
        let mut s = stepper(true);
        for _ in 0..14 {
            s.should_update_world(true, true);
        }
        assert!(s.should_update_world(false, true), "the release steps");
        assert_eq!(s.step_held_count, 0, "the release cleared the count");
        for held in 1..=11 {
            assert!(
                !s.should_update_world(true, true),
                "the re-hold counts from zero: frame {held} steps nothing"
            );
        }
        assert!(s.should_update_world(true, true), "the 12th re-held frame steps");
    }

    /// DW-2.1 edge — the C# verbatim count semantics across an unpause
    /// mid-hold: the unpaused frames freeze the count (`:24-27` returns
    /// before the increment), so re-pausing mid-hold resumes pacing from the
    /// stale value (15 → no step, 16 → step).
    #[test]
    fn s1_b_3_stepper_pacing_count_freezes_across_unpause() {
        let mut s = stepper(true);
        for _ in 0..14 {
            s.should_update_world(true, true);
        }
        assert_eq!(s.step_held_count, 14);
        // Unpause mid-hold (the key never released): true every frame, no
        // count movement.
        s.paused = false;
        for _ in 0..5 {
            assert!(s.should_update_world(true, true));
        }
        assert_eq!(s.step_held_count, 14, "unpaused frames freeze the count");
        // Re-pause mid-hold: the next held frame resumes at 15 (no step),
        // then 16 (step).
        s.paused = true;
        assert!(
            !s.should_update_world(true, true),
            "held frame 15 resumes without stepping (odd)"
        );
        assert!(
            s.should_update_world(true, true),
            "held frame 16 steps (even)"
        );
    }

    /// DW-2.2 — `TogglePaused` (`WorldUpdateStepper.cs:45-49`): flips
    /// `Paused` and pins `StartUpdateCount` to the passed `GameUpdateCount`
    /// on every toggle — including the resume toggle, which re-pins (the
    /// next pause counts from its own pin).
    #[test]
    fn s1_b_3_stepper_toggle_pins_start_update_count() {
        let mut s = WorldUpdateStepper::new();
        assert!(!s.paused);
        assert_eq!(s.start_update_count, 0);
        s.toggle_paused(65536);
        assert!(s.paused, "the first toggle pauses");
        assert_eq!(s.start_update_count, 65536, "the pin takes the count");
        s.toggle_paused(66000);
        assert!(!s.paused, "the second toggle resumes");
        assert_eq!(
            s.start_update_count, 66000,
            "the resume toggle re-pins too (the C# writes it unconditionally)"
        );
    }

    /// DW-2.2 — `DrawHUD`'s text (`WorldUpdateStepper.cs:51-57`): `None`
    /// while unpaused (never draws), the `Updates Since Paused: N` border
    /// string while paused diffing against the pin — including the C#'s
    /// `(int)(GameUpdateCount - StartUpdateCount)` cast on a wrapped u32.
    #[test]
    fn s1_b_3_stepper_hud_overlay_text_and_guard() {
        let mut s = WorldUpdateStepper::new();
        assert!(
            s.hud_text(65536).is_none(),
            "unpaused never draws the overlay"
        );
        s.toggle_paused(65536);
        assert_eq!(
            s.hud_text(65543).as_deref(),
            Some("Updates Since Paused: 7"),
            "the overlay text and count, 7 steps since the pin"
        );
        assert_eq!(
            s.hud_text(65536).as_deref(),
            Some("Updates Since Paused: 0"),
            "the pin frame itself reads 0"
        );
        // The C# cast: `(int)(0x80000000u)` wraps to i32::MIN.
        assert_eq!(
            s.hud_text(65536u32.wrapping_add(0x80000000)).as_deref(),
            Some("Updates Since Paused: -2147483648"),
            "the (int) cast's wrapped count"
        );
        s.toggle_paused(66000);
        assert!(
            s.hud_text(66000).is_none(),
            "resumed hides the overlay again"
        );
    }
}
