//! The generic progress bar element — W4-d83 (research/18b row 46):
//! `UIProgressBar` (`GameContent/UI/Elements/UIProgressBar.cs:7-46`) with its
//! private `UIInnerProgressBar` (`:9-16`). Not `UIGenProgressBar` (row 31,
//! the world-gen bar, landed S1-E-17 in `ui/world_load.rs`) — this is the
//! plain blue eased bar the resource-pack/workshop screens use.

/// `UIProgressBar` (`UIProgressBar.cs:7-46`), research/18b row 46 (W4-d83):
/// a `MagicPixel` blue fill whose width eases toward the target every draw.
/// The inner bar is a child at `Height.Precent = 1f` (`:26`); its draw is
/// `MagicPixel` scaled `(Width, Height / 1000f)` (`:14`) — the 1000×1000
/// magic pixel shrunk to the child rect, `Color.Blue` untinted.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ProgressBar {
    /// `_visualProgress` (`:20`) — the eased width fraction.
    visual_progress: f32,
    /// `_targetProgress` (`:22`).
    target_progress: f32,
}

impl ProgressBar {
    /// `SetProgress` (`:31-38`): the target is set unconditionally; the
    /// visual **snaps down** when the new value is below it (a reset never
    /// eases backwards) but a rise eases through [`Self::draw_tick`].
    pub fn set_progress(&mut self, value: f32) {
        self.target_progress = value;
        if value < self.visual_progress {
            self.visual_progress = value; // `:34-37`
        }
    }

    /// `DrawSelf`'s ease (`:40-45`): `visual = visual * 0.95 + 0.05 * target`,
    /// then the inner bar's `Width.Precent = visual`. Returns the new width
    /// fraction (the caller sizes the child).
    pub fn draw_tick(&mut self) -> f32 {
        self.visual_progress = self.visual_progress * 0.95 + 0.05 * self.target_progress;
        self.visual_progress
    }

    /// The current width fraction (`_visualProgress`), for hit tests/reads.
    pub fn visual_progress(&self) -> f32 {
        self.visual_progress
    }

    /// `UIInnerProgressBar.DrawSelf` (`:11-15`): the child rect `[x, y, w, h]`
    /// draws `MagicPixel` at `(x, y)` scaled `(w, h / 1000f)` in `Color.Blue`
    /// — the port returns the fill rect + tint (the 1000 px sheet is the
    /// renderer's magic pixel).
    pub fn fill_draw(child_rect: [f32; 4]) -> ([f32; 4], [u8; 4]) {
        (
            [child_rect[0], child_rect[1], child_rect[2], child_rect[3]],
            [0, 0, 255, 255], // `Color.Blue`
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The ease + snap-down (`UIProgressBar.cs:31-45`): a rise eases at
    /// 0.95/0.05 per draw, a drop snaps the visual immediately, and the
    /// fill is untinted Blue over the whole child rect.
    #[test]
    fn the_bar_eases_up_and_snaps_down() {
        let mut bar = ProgressBar::default();
        bar.set_progress(1.0);
        assert_eq!(bar.visual_progress(), 0.0, "the rise does not snap");
        assert_eq!(bar.draw_tick(), 0.05); // 0*0.95 + 0.05*1
        let second = bar.draw_tick();
        assert!((second - 0.0975).abs() < 1e-6); // 0.05*0.95 + 0.05
        bar.set_progress(0.02);
        assert_eq!(bar.visual_progress(), 0.02, "the drop snaps (`:34-37`)");
        // The eased value keeps climbing toward the (lower) target from 0.02.
        let third = bar.draw_tick();
        assert!((third - (0.02 * 0.95 + 0.05 * 0.02)).abs() < 1e-6);
        let (rect, tint) = ProgressBar::fill_draw([10.0, 20.0, 300.0, 8.0]);
        assert_eq!(rect, [10.0, 20.0, 300.0, 8.0]);
        assert_eq!(tint, [0, 0, 255, 255]);
    }
}
