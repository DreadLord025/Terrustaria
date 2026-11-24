//! The particle overlay element — W4-d83 (research/18b row 45):
//! `UIParticleLayer` (`GameContent/UI/Elements/UIParticleLayer.cs:8-49`).
//! A mouse-transparent `UIElement` hosting a `ParticleRenderer`; the port
//! carries the anchor maths (`Recalculate`) — the renderer itself
//! (`Graphics/Renderers/ParticleRenderer.cs`) is unported and stays the
//! caller's seam (`AddParticle`/`ClearParticles`/`Update`/`Draw` forward).

/// `UIParticleLayer` (`UIParticleLayer.cs:8-49`), research/18b row 45
/// (W4-d83): the ctor sets `IgnoresMouseInteraction = true` (`:18`) — the
/// layer never eats clicks. The two public offset vectors are the anchor
/// knobs; `Recalculate` re-derives the renderer's anchor from the element
/// rect every layout pass.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ParticleLayer {
    /// `AnchorPositionOffsetByPercents` (`:12`) — the fraction of the rect
    /// size added to the top-left.
    pub offset_by_percents: [f32; 2],
    /// `AnchorPositionOffsetByPixels` (`:14`) — the pixel nudge on top.
    pub offset_by_pixels: [f32; 2],
    /// `IgnoresMouseInteraction = true` (`:18`) — pinned by the ctor; kept
    /// as a field so the layout pass reads it like any element's.
    pub ignores_mouse_interaction: bool,
}

impl ParticleLayer {
    /// The ctor (`:16-21`): mouse-transparent, zero offsets, a fresh
    /// `ParticleRenderer` (the caller's) wired to `OnUpdate`.
    pub fn new() -> Self {
        Self {
            offset_by_percents: [0.0, 0.0],
            offset_by_pixels: [0.0, 0.0],
            ignores_mouse_interaction: true,
        }
    }

    /// `Recalculate`'s anchor (`:28-33`):
    /// `r = GetDimensions().ToRectangle()` (**int truncation**,
    /// `CalculatedStyle.cs:15-18`), then
    /// `AnchorPosition = r.TopLeft() + OffsetByPercents * r.Size() +
    /// OffsetByPixels` — the percents ride the *truncated* size, and the
    /// result is float (`Vector2` + int-vector promotion).
    pub fn anchor_position(&self, rect: [f32; 4]) -> [f32; 2] {
        let r = [
            rect[0] as i32,
            rect[1] as i32,
            rect[2] as i32,
            rect[3] as i32,
        ]; // `ToRectangle()`
        [
            r[0] as f32 + self.offset_by_percents[0] * r[2] as f32 + self.offset_by_pixels[0],
            r[1] as f32 + self.offset_by_percents[1] * r[3] as f32 + self.offset_by_pixels[1],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The anchor maths (`UIParticleLayer.cs:31-32`): the rect truncates to
    /// ints first, the percents ride the truncated size, the pixels add
    /// last; a centred layer with a nudge lands on the truncated centre.
    #[test]
    fn the_anchor_truncates_the_rect_then_offsets() {
        let mut layer = ParticleLayer::new();
        assert!(layer.ignores_mouse_interaction, "the ctor pins it (`:18`)");
        // Zero offsets: the anchor is the truncated top-left.
        assert_eq!(layer.anchor_position([10.9, 20.8, 100.7, 50.2]), [10.0, 20.0]);
        layer.offset_by_percents = [0.5, 0.5];
        layer.offset_by_pixels = [1.5, -2.0];
        // r = (10, 20, 100, 50): (10 + 50 + 1.5, 20 + 25 - 2) = (61.5, 43).
        assert_eq!(layer.anchor_position([10.9, 20.8, 100.7, 50.2]), [61.5, 43.0]);
        // The percents ride the TRUNCATED size: 101.9 -> 101 (half 50.5),
        // not 101.9 (half 50.95).
        assert_eq!(layer.anchor_position([0.0, 0.0, 101.9, 0.0])[0], 50.5 + 1.5);
    }
}
