//! The image *element* state machines the immediate draws have no place for.
//! W4-d82 (research/18b row 26): `UICyclingImage`
//! (`GameContent/UI/Elements/UICyclingImage.cs:8-38`). The `UIImage` base
//! (draw/scale/origin) is d83's lane (row 36); this module carries only the
//! frame-cycling counter, which is independent of the draw.
//!
//! The one user is `AWorldListItem.GetIcons` (`AWorldListItem.cs:181`, d81's
//! row 1): the world card's evil/biome icon strip cycles its textures.

/// `UICyclingImage` (`UICyclingImage.cs:8-38`): a `UIImage` that swaps its
/// texture every `FramesPerCycle` updates. The port keeps the counter + index;
/// the texture list is the caller's (the C# holds `List<Asset<Texture2D>>`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CyclingImage {
    /// `FramesPerCycle` (`:10`), armed to 45 by the ctor (`:20`).
    pub frames_per_cycle: i32,
    /// `_currentTextureIndex` (`:14`), starts 0 — the ctor calls
    /// `SetImage(_textureAssets[0])` (`:22`).
    current_index: usize,
    /// `_framesCounted` (`:16`).
    frames_counted: i32,
    /// `_textureAssets.Count` (`:12`) — the cycle's modulus.
    texture_count: usize,
}

impl CyclingImage {
    /// The ctor (`:18-23`): 45 frames per cycle, index 0 shown first. An empty
    /// list is not constructible in the C# (`_textureAssets[0]` would throw);
    /// the port clamps the count to at least 1 so the counter never divides by
    /// zero, and the caller supplies a real sheet.
    pub fn new(texture_count: usize) -> Self {
        Self {
            frames_per_cycle: 45,
            current_index: 0,
            frames_counted: 0,
            texture_count: texture_count.max(1),
        }
    }

    /// The texture index the draw should show (`_currentTextureIndex`).
    pub fn current_index(&self) -> usize {
        self.current_index
    }

    /// `Update` (`:25-37`): `++_framesCounted >= FramesPerCycle` resets the
    /// counter and advances the index, wrapping at the list count; the advance
    /// calls `SetImage` (the caller re-binds the sheet). Returns the new index
    /// when a swap happened this update, `None` otherwise.
    pub fn update(&mut self) -> Option<usize> {
        self.frames_counted += 1; // `:28` `++this._framesCounted`
        if self.frames_counted >= self.frames_per_cycle {
            self.frames_counted = 0; // `:30`
            self.current_index += 1; // `:31` `++this._currentTextureIndex`
            if self.current_index >= self.texture_count {
                self.current_index = 0; // `:33`
            }
            return Some(self.current_index); // `:35` SetImage
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The 45-frame cadence + wrap (`UICyclingImage.cs:28-35`): with 3 textures
    /// the index advances on updates 45, 90, 135 and wraps 2 -> 0.
    #[test]
    fn the_cycle_advances_every_45_frames_and_wraps() {
        let mut c = CyclingImage::new(3);
        assert_eq!(c.current_index(), 0);
        for _ in 0..44 {
            assert_eq!(c.update(), None, "no swap before the 45th frame");
        }
        assert_eq!(c.update(), Some(1), "the 45th update swaps to 1");
        for _ in 0..44 {
            assert_eq!(c.update(), None);
        }
        assert_eq!(c.update(), Some(2));
        for _ in 0..44 {
            assert_eq!(c.update(), None);
        }
        assert_eq!(c.update(), Some(0), "the index wraps at the count");
    }

    /// A single-texture list still ticks but never leaves index 0 (`:31-34`).
    #[test]
    fn a_single_texture_stays_at_zero() {
        let mut c = CyclingImage::new(1);
        for _ in 0..44 {
            assert_eq!(c.update(), None);
        }
        assert_eq!(c.update(), Some(0));
        assert_eq!(c.current_index(), 0);
    }
}
