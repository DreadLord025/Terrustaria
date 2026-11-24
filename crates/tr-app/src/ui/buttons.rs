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
//!
//! W4-d83 (research/18b row 35): `UIIconTextButton`
//! (`GameContent/UI/Elements/UIIconTextButton.cs:12-132`) — the creation-screen
//! category button (44x34 base, spliced panel + right-edge icon). Its hover
//! sound (`:80-91`) is the same latch, reused; this module adds the colour
//! state machine, the `SetText` resize and the draw geometry.

use super::char_creation::lerp_rgb;
use super::color;
use super::uistate::INVENTORY_DEFAULT_COLOR;

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

    /// `UIIconTextButton`'s colour chain (`UIIconTextButton.cs:47`, `:95`,
    /// `:98`, `:111`, `:118`): the rest colour is
    /// `Lerp(Black, InventoryDefaultColor, FadeFromBlack)` at opacity 1; the
    /// hover swaps to `Lerp(InventoryDefaultColor, White, 0.7)` at 0.7; the
    /// panel tints through a second `Lerp(Black, color, FadeFromBlack)` and
    /// the icon through `Lerp(color, White, 0.7)`, both `* opacity`.
    #[test]
    fn the_icon_text_button_colour_chain_matches_the_lerps() {
        let mut b = IconTextButton::new(1.0);
        assert_eq!(b.size, [44.0, 34.0], "the ctor's base 44x34 (`:38-39`)");
        assert_eq!(b.color, INVENTORY_DEFAULT_COLOR, "fade 1 rest colour");
        assert_eq!(b.opacity, 1.0);
        assert_eq!(b.panel_tint(), [63, 65, 151, 255]);
        // Icon at rest: Lerp((63,65,151), White, 0.7) = (197,197,223).
        assert_eq!(b.icon_tint(), [197, 197, 223, 255]);
        b.mouse_over();
        assert_eq!(b.color, [197, 197, 223, 255]);
        assert_eq!(b.opacity, 0.7);
        // Panel: (197,197,223,255) * 0.7 truncates -> (137,137,156,178).
        assert_eq!(b.panel_tint(), [137, 137, 156, 178]);
        // Icon: Lerp((197,197,223), White, 0.7) = (237,237,245) * 0.7 ->
        // (165,165,171,178) — 237*0.7 = 165.9 truncates to 165, not 166.
        assert_eq!(b.icon_tint(), [165, 165, 171, 178]);
        assert!(b.latch.draw_tick(), "the first hovered draw plays 12 (`:82-84`)");
        b.mouse_out();
        assert_eq!(b.color, INVENTORY_DEFAULT_COLOR, "MouseOut restores (`:118`)");
        assert_eq!(b.opacity, 1.0);
    }

    /// The fade gate (`UIIconTextButton.cs:47/:95`): at `FadeFromBlack = 0.5`
    /// the rest colour is `Lerp(Black, (63,65,151), 0.5) = (31,32,75,255)`
    /// (alpha lerps 255 -> 255, it stays opaque) and the panel lerp halves
    /// the RGB again -> `(15,16,37,255)`.
    #[test]
    fn the_fade_from_black_lerps_twice() {
        let b = IconTextButton::new(0.5);
        assert_eq!(b.color, [31, 32, 75, 255]);
        assert_eq!(b.panel_tint(), [15, 16, 37, 255]);
    }

    /// `SetText`'s resize (`UIIconTextButton.cs:71-75`): with an icon the
    /// button grows to `titleW + iconW + 26` x `max(titleH, iconH) + 16`;
    /// without one the 44x34 base stays. The icon draws at
    /// `(X + W - iconW - 5, CenterY - iconH/2)` with **integer** halving
    /// (`:98` `(float)(Height() / 2)`).
    #[test]
    fn set_text_resizes_and_the_icon_rides_the_right_edge() {
        let mut b = IconTextButton::new(1.0);
        b.set_text([50.0, 16.0], Some([24.0, 25.0]));
        assert_eq!(b.size, [100.0, 41.0]);
        // CenterY = 20 + 41*0.5 = 40.5; iconH/2 int = 12; y = 28.5 (the odd
        // icon height keeps the half pixel — the C# halves as int).
        assert_eq!(b.icon_pos([10.0, 20.0, 100.0, 41.0], [24.0, 25.0]), [81.0, 28.5]);
        let mut no_icon = IconTextButton::new(1.0);
        no_icon.set_text([50.0, 16.0], None);
        assert_eq!(no_icon.size, [44.0, 34.0], "no icon: no resize (`:71`)");
    }
}

/// `UIIconTextButton` (`GameContent/UI/Elements/UIIconTextButton.cs:12-132`),
/// research/18b row 35 (W4-d83): the creation-screen category button — a
/// spliced grayscale panel with a right-edge icon and a left-aligned title.
/// The port keeps the colour/opacity state machine, the `SetText` resize and
/// the draw geometry; the panel pieces ride `uistate::spliced_rects` and the
/// title is a `UIText` child (row 53) the caller lays out at
/// `Left = 10px, VAlign = 0.5` (`:62-66`).
///
/// The ctor's `titleAlignmentX`/`titleWidthReduction` parameters
/// (`:36`) are **dead** in 1.4.5.8 — the body never reads them; not ported.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IconTextButton {
    /// `Width`/`Height` — 44x34 from the ctor (`:38-39`), resized by
    /// [`Self::set_text`] when an icon is bound.
    pub size: [f32; 2],
    /// `FadeFromBlack = 1f` (`:24`), the public fade knob.
    pub fade_from_black: f32,
    /// `_color` (`:20`) — the state colour, bytes.
    pub color: [u8; 4],
    /// `_opacity = 0.7f` (`:28`) — set to 1 by the ctor's `SetColor` (`:47`).
    pub opacity: f32,
    /// `_hoverColor` (`:22`), White from the ctor (`:40`). **Dead** in
    /// 1.4.5.8: `SetHoverColor` (`:128-131`) writes it but `DrawSelf` lerps
    /// to `Color.White` literally (`:98`); kept for the API's sake.
    pub hover_color: [u8; 4],
    /// The `_hovered`/`_soundedHover` latch (`:30-32`, `:80-91`) — the same
    /// one-shot as the difficulty/hair buttons, shared.
    pub latch: HoverSoundLatch,
}

impl IconTextButton {
    /// The ctor's colour half (`UIIconTextButton.cs:38-47`): 44x34, White
    /// hover colour, `SetColor(Lerp(Black, InventoryDefaultColor, fade), 1)`.
    pub fn new(fade_from_black: f32) -> Self {
        let color = lerp_rgb([0, 0, 0, 255], INVENTORY_DEFAULT_COLOR, fade_from_black);
        Self {
            size: [44.0, 34.0],
            fade_from_black,
            color,
            opacity: 1.0,
            hover_color: [255, 255, 255, 255],
            latch: HoverSoundLatch::default(),
        }
    }

    /// `SetText`'s resize (`:71-75`): with an icon,
    /// `Width = titleW + iconW + 26`, `Height = max(titleH, iconH) + 16`.
    /// `title_size` is the measured `UIText.GetDimensions()`; the text swap
    /// itself (the child `UIText`, `:56-70`) is the caller's.
    pub fn set_text(&mut self, title_size: [f32; 2], icon_size: Option<[f32; 2]>) {
        if let Some(icon) = icon_size {
            self.size = [
                title_size[0] + icon[0] + 26.0,
                title_size[1].max(icon[1]) + 16.0,
            ];
        }
    }

    /// `MouseOver` (`:108-113`): `SetColor(Lerp(InventoryDefaultColor, White,
    /// _whiteLerp = 0.7), 0.7)` then the hover flag.
    pub fn mouse_over(&mut self) {
        self.color = lerp_rgb(INVENTORY_DEFAULT_COLOR, [255, 255, 255, 255], 0.7);
        self.opacity = 0.7;
        self.latch.mouse_over();
    }

    /// `MouseOut` (`:115-120`): back to the ctor's rest colour at opacity 1.
    pub fn mouse_out(&mut self) {
        self.color = lerp_rgb([0, 0, 0, 255], INVENTORY_DEFAULT_COLOR, self.fade_from_black);
        self.opacity = 1.0;
        self.latch.mouse_out();
    }

    /// The panel tint (`:95`): `Lerp(Black, _color, FadeFromBlack) * _opacity`
    /// — the fade applies **twice** (once in the state colour, once here).
    pub fn panel_tint(&self) -> [u8; 4] {
        color::mul(
            lerp_rgb([0, 0, 0, 255], self.color, self.fade_from_black),
            self.opacity,
        )
    }

    /// The icon tint (`:98`): `Lerp(_color, White, _whiteLerp = 0.7) *
    /// _opacity` (`_whiteLerp`, `:26`, is private and never changed).
    pub fn icon_tint(&self) -> [u8; 4] {
        color::mul(
            lerp_rgb(self.color, [255, 255, 255, 255], 0.7),
            self.opacity,
        )
    }

    /// The icon position (`:98`): `(X + Width - iconW - 5, CenterY -
    /// (float)(iconH / 2))` — the half-height is **integer** division before
    /// the float cast, so an odd-height icon sits half a pixel high.
    pub fn icon_pos(&self, rect: [f32; 4], icon_size: [f32; 2]) -> [f32; 2] {
        [
            rect[0] + rect[2] - icon_size[0] - 5.0,
            rect[1] + rect[3] * 0.5 - (icon_size[1] as i32 / 2) as f32,
        ]
    }
}
