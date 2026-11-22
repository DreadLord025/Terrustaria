//! The creation-screen button *element* halves the immediate draw ports left
//! out — `UIDifficultyButton` (`GameContent/UI/Elements/UIDifficultyButton.cs`)
//! and `UIHairStyleButton` (`UIHairStyleButton.cs`). W4-d82 (research/18b rows
//! 29/32): the draw/click/title halves are landed in `char_creation.rs`
//! (`draw_hair_button`, the difficulty fills at `:1981-2018`, `Click::Difficulty`
//! / `Click::Hair`); this module carries the two state machines that draw has
//! no place for — the one-shot hover sound and the hair preview's frame skip.
//!
//! Both classes share the *identical* hover-sound latch (`UIDifficultyButton.cs:49-60`,
//! `UIHairStyleButton.cs:43-54`), so it is ported once ([`HoverSoundLatch`]) —
//! the house rule since the 2026-11-01 refactor.

/// The `_hovered` + `_soundedHover` pair both creation buttons keep
/// (`UIDifficultyButton.cs:24-26`, `UIHairStyleButton.cs:19-21`).
///
/// The sound fires from **DrawSelf**, not from `MouseOver`: while hovered, the
/// first draw plays `SoundEngine.PlaySound(12)` (the menu tick) and latches
/// `_soundedHover`; every later hovered draw is silent; a draw with the mouse
/// off the button re-arms the latch (`UIDifficultyButton.cs:49-60`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HoverSoundLatch {
    /// `_hovered` — set by `MouseOver` (`:88-92`), cleared by `MouseOut`
    /// (`:94-98`), *after* the base bubble call in both.
    pub hovered: bool,
    /// `_soundedHover` — the draw-side one-shot.
    sounded: bool,
}

impl HoverSoundLatch {
    /// `MouseOver` (`UIDifficultyButton.cs:88-92`): base first (the ancestor
    /// `IsMouseHovering` walk, `framework::bubble_chain`), then the flag.
    pub fn mouse_over(&mut self) {
        self.hovered = true;
    }

    /// `MouseOut` (`UIDifficultyButton.cs:94-98`).
    pub fn mouse_out(&mut self) {
        self.hovered = false;
    }

    /// The `DrawSelf` head (`UIDifficultyButton.cs:49-60`): returns `true`
    /// exactly when the caller must `PlaySound(12)`. The latch is set even on
    /// the sounding draw; the unhovered draw clears it.
    pub fn draw_tick(&mut self) -> bool {
        if self.hovered {
            let play = !self.sounded;
            self.sounded = true;
            play
        } else {
            self.sounded = false;
            false
        }
    }
}

/// `UIHairStyleButton._framesToSkip` (`UIHairStyleButton.cs:23`, `:36-39`,
/// `:65-69`): after the borders draw, a positive counter decrements and the
/// `DrawPlayerHead` preview is skipped for that many frames — the scroll
/// performance valve (`UICharacterCreation` calls `SkipRenderingContent` while
/// the hair list scrolls). The panel + borders still draw every frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HairPreviewSkip {
    frames_to_skip: i32,
}

impl HairPreviewSkip {
    /// `SkipRenderingContent(timeInFrames)` (`:36-39`) — a set, not an add.
    pub fn skip_rendering_content(&mut self, time_in_frames: i32) {
        self.frames_to_skip = time_in_frames;
    }

    /// The `:65-69` gate, evaluated once per draw *after* the borders: `true`
    /// means draw the head this frame; a positive counter loses one frame.
    pub fn should_draw_head(&mut self) -> bool {
        if self.frames_to_skip > 0 {
            self.frames_to_skip -= 1;
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The one-shot: first hovered draw sounds, the rest are silent, leaving
    /// re-arms (`UIDifficultyButton.cs:49-60`).
    #[test]
    fn the_hover_sound_is_a_one_shot_per_hover() {
        let mut l = HoverSoundLatch::default();
        assert!(!l.draw_tick(), "unhovered draws never sound");
        l.mouse_over();
        assert!(l.draw_tick(), "the first hovered draw plays 12");
        assert!(!l.draw_tick());
        assert!(!l.draw_tick());
        l.mouse_out();
        assert!(!l.draw_tick(), "the unhovered draw re-arms silently");
        l.mouse_over();
        assert!(l.draw_tick(), "a fresh hover sounds again");
    }

    /// `SkipRenderingContent(3)` skips exactly three head draws, then the
    /// preview returns (`UIHairStyleButton.cs:65-69`); a re-set replaces.
    #[test]
    fn the_skip_counter_drains_one_per_draw() {
        let mut s = HairPreviewSkip::default();
        assert!(s.should_draw_head(), "no skip armed: the head draws");
        s.skip_rendering_content(3);
        assert!(!s.should_draw_head());
        assert!(!s.should_draw_head());
        assert!(!s.should_draw_head());
        assert!(s.should_draw_head(), "the fourth draw is back");
        s.skip_rendering_content(2);
        s.skip_rendering_content(1); // a set, not an add (`:38`)
        assert!(!s.should_draw_head());
        assert!(s.should_draw_head());
    }
}
