//! The image *element* state machines the immediate draws have no place for.
//! W4-d82 (research/18b row 26): `UICyclingImage`
//! (`GameContent/UI/Elements/UICyclingImage.cs:8-38`) — the frame-cycling
//! counter, independent of the draw.
//! W4-d83 (research/18b rows 36-38): the `UIImage` base (draw/scale/origin
//! geometry, `UIImage.cs:8-100`), `UIImageFramed` (sub-frame pick + grid slice,
//! `UIImageFramed.cs:8-50`) and `UIImageWithBorder` (the border sizing + draw,
//! `UIImageWithBorder.cs:8-68`).
//!
//! The cycling counter's one user is `AWorldListItem.GetIcons`
//! (`AWorldListItem.cs:181`, d81's row 1): the world card's evil/biome icon
//! strip cycles its textures.

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

/// `UIImage` (`GameContent/UI/Elements/UIImage.cs:8-100`), research/18b row 36
/// (W4-d83): the draw-geometry state machine. The port keeps the fields the
/// `DrawSelf` reads plus the `SetImage` resize; the caller binds the sheet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageElement {
    /// `ImageScale = 1f` (`UIImage.cs:12`).
    pub image_scale: f32,
    /// `Rotation` (`:14`), radians, XNA `SpriteBatch.Draw` rotation.
    pub rotation: f32,
    /// `ScaleToFit` (`:16`): the stretch-to-rect draw arm (`:82-86`).
    pub scale_to_fit: bool,
    /// `AllowResizingDimensions = true` (`:18`): `SetImage` resizes the element
    /// to the texture (`:52-56`).
    pub allow_resizing_dimensions: bool,
    /// `Color = Color.White` (`:20`), bytes pre-`color::tint`.
    pub color: [u8; 4],
    /// `NormalizedOrigin = Vector2.Zero` (`:22`), the 0..1 origin fraction.
    pub normalized_origin: [f32; 2],
    /// `Frame` (`:24`): `None` = the whole sheet (XNA's null `Rectangle?`).
    pub frame: Option<[i32; 4]>,
    /// `RemoveFloatingPointsFromDrawPosition` (`:26`): floor the draw pos
    /// (`:94-97`).
    pub remove_floating_points_from_draw_position: bool,
    /// `UseTextureSizeForOrigin = true` (`:28`): the origin/centring vector is
    /// the texture size, not the element size (`:89-92`).
    pub use_texture_size_for_origin: bool,
    /// `Width`/`Height` — the element style the ctor's `SetImage` pins to the
    /// texture (`:54-55`).
    pub size: [f32; 2],
}

impl Default for ImageElement {
    fn default() -> Self {
        // The field initialisers (`UIImage.cs:12-28`); the size starts 0 until
        // `set_image` runs (the ctor always calls it, `:38-46`).
        Self {
            image_scale: 1.0,
            rotation: 0.0,
            scale_to_fit: false,
            allow_resizing_dimensions: true,
            color: [255, 255, 255, 255],
            normalized_origin: [0.0, 0.0],
            frame: None,
            remove_floating_points_from_draw_position: false,
            use_texture_size_for_origin: true,
            size: [0.0, 0.0],
        }
    }
}

impl ImageElement {
    /// `SetImage` (`UIImage.cs:48-68`): with `AllowResizingDimensions` the
    /// element takes the texture's pixel size. Returns the new size when a
    /// resize happened (the caller re-derives its layout), `None` otherwise.
    pub fn set_image(&mut self, texture_size: [f32; 2]) -> Option<[f32; 2]> {
        if self.allow_resizing_dimensions {
            self.size = texture_size; // `Width.Set(w, 0f)` / `Height.Set(h, 0f)`
            Some(self.size)
        } else {
            None
        }
    }

    /// `DrawSelf`'s geometry (`UIImage.cs:70-99`). `rect` is the calculated
    /// style `[x, y, w, h]`, `texture_size` the bound sheet's pixels.
    ///
    /// - `ScaleToFit` (`:82-86`): `Fit` — the sheet stretched to
    ///   `ToRectangle()` (`CalculatedStyle.cs:15-18`, int truncation), the
    ///   `Frame` sub-rect kept, no rotation/scale/origin.
    /// - otherwise (`:87-98`): `At` — pos = `position + v2*(1-scale)/2 +
    ///   v2*normalizedOrigin` with `v2` the texture size when
    ///   `UseTextureSizeForOrigin` else the element size (`:89-92`), floored
    ///   per-component when `RemoveFloatingPointsFromDrawPosition` (`:94-97`,
    ///   `Vector2.Floor`); the draw origin is `textureSize * normalizedOrigin`
    ///   (`:98`), the scale `ImageScale`, the rotation `Rotation`.
    pub fn draw_geometry(&self, rect: [f32; 4], texture_size: [f32; 2]) -> ImageDraw {
        if self.scale_to_fit {
            return ImageDraw::Fit {
                rect: [
                    rect[0] as i32,
                    rect[1] as i32,
                    rect[2] as i32,
                    rect[3] as i32,
                ],
            };
        }
        let v2 = if self.use_texture_size_for_origin {
            texture_size // `:91`
        } else {
            [rect[2], rect[3]] // `:88`
        };
        let mut pos = [
            rect[0] + v2[0] * (1.0 - self.image_scale) / 2.0
                + v2[0] * self.normalized_origin[0],
            rect[1] + v2[1] * (1.0 - self.image_scale) / 2.0
                + v2[1] * self.normalized_origin[1],
        ]; // `:93`
        if self.remove_floating_points_from_draw_position {
            pos = [pos[0].floor(), pos[1].floor()]; // `:96`
        }
        ImageDraw::At {
            pos,
            origin: [
                texture_size[0] * self.normalized_origin[0],
                texture_size[1] * self.normalized_origin[1],
            ], // `:98` `vector * NormalizedOrigin`
            scale: self.image_scale,
            rotation: self.rotation,
        }
    }
}

/// The two `UIImage.DrawSelf` arms (`UIImage.cs:82-98`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ImageDraw {
    /// The `ScaleToFit` stretch (`:84`): destination `ToRectangle()`.
    Fit { rect: [i32; 4] },
    /// The positioned draw (`:98`): top-left `pos`, XNA origin/scale/rotation.
    At {
        pos: [f32; 2],
        origin: [f32; 2],
        scale: f32,
        rotation: f32,
    },
}

/// `UIImageFramed` (`GameContent/UI/Elements/UIImageFramed.cs:8-50`),
/// research/18b row 37 (W4-d83): a fixed sub-frame of a sheet, sized to the
/// frame, drawn at the element position with no scale/rotation (`:45-49`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FramedImage {
    /// `_frame` (`UIImageFramed.cs:12`) — the source rect on the sheet.
    pub frame: [i32; 4],
    /// `Width`/`Height`, pinned to the frame size by every setter (`:20-21`,
    /// `:29-30`, `:36-37`).
    pub size: [f32; 2],
    /// `Color { get; set; }` (`:14`), `Color.White` from the ctor (`:22`).
    pub color: [u8; 4],
}

impl FramedImage {
    /// The ctor (`UIImageFramed.cs:16-23`): frame + size + White.
    pub fn new(frame: [i32; 4]) -> Self {
        Self {
            frame,
            size: [frame[2] as f32, frame[3] as f32],
            color: [255, 255, 255, 255],
        }
    }

    /// `SetImage(texture, frame)` (`:25-31`) / `SetFrame(frame)` (`:33-38`):
    /// both re-pin the size to the frame; the texture swap is the caller's.
    pub fn set_frame(&mut self, frame: [i32; 4]) {
        self.frame = frame;
        self.size = [frame[2] as f32, frame[3] as f32];
    }

    /// `SetFrame(frameCountHorizontal, frameCountVertical, frameX, frameY,
    /// sizeOffsetX, sizeOffsetY)` (`:40-43`): `texture.Frame(...)`
    /// (`Utils.cs:1344-1349` — integer cell division) then `OffsetSize`
    /// (`Utils.cs:1328-1333` — the offsets add to width/height only).
    pub fn set_frame_grid(
        &mut self,
        texture_size: [i32; 2],
        horizontal_frames: i32,
        vertical_frames: i32,
        frame_x: i32,
        frame_y: i32,
        size_offset_x: i32,
        size_offset_y: i32,
    ) {
        let w = texture_size[0] / horizontal_frames; // `Utils.cs:1346`
        let h = texture_size[1] / vertical_frames; // `:1347`
        self.set_frame([
            w * frame_x,
            h * frame_y,
            w + size_offset_x,
            h + size_offset_y,
        ]); // `:1348` + `OffsetSize`
    }
}

/// `UIImageWithBorder` (`GameContent/UI/Elements/UIImageWithBorder.cs:8-68`),
/// research/18b row 38 (W4-d83): a `UIImage` plus a border sheet drawn over it.
/// The inner image's state rides [`ImageElement`]; this carries the border
/// sizing (`SetBorder`) and the border draw arm.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BorderedImage {
    /// The `UIImage` base fields (`: base(texture)` ctor, `:14-24`).
    pub image: ImageElement,
}

impl BorderedImage {
    /// The ctor (`UIImageWithBorder.cs:14-24`): base `SetImage(texture)` first
    /// (sizes to the image), then `SetBorder` **overwrites** the size with the
    /// border's (`:30-31`) — the border sheet is the element's footprint.
    pub fn new(texture_size: [f32; 2], border_size: [f32; 2]) -> Self {
        let mut image = ImageElement::default();
        image.set_image(texture_size);
        image.set_image(border_size); // `SetBorder`'s unconditional resize
        Self { image }
    }

    /// `SetBorder` (`:26-40`): unlike `UIImage.SetImage` there is **no**
    /// `AllowResizingDimensions` gate — the size always takes the border's.
    pub fn set_border(&mut self, border_size: [f32; 2]) -> [f32; 2] {
        self.image.size = border_size;
        self.image.size
    }

    /// The border half of `DrawSelf` (`:42-67`): after `base.DrawSelf`, the
    /// border draws with the base's colour/scale/rotation/origin — but the
    /// centring vector is always the **border texture's** size (`:60-61`),
    /// never the element size, and the border has no `Frame` (`:66` null).
    pub fn border_draw_geometry(&self, rect: [f32; 4], border_size: [f32; 2]) -> ImageDraw {
        if self.image.scale_to_fit {
            return ImageDraw::Fit {
                rect: [
                    rect[0] as i32,
                    rect[1] as i32,
                    rect[2] as i32,
                    rect[3] as i32,
                ],
            }; // `:55-58`
        }
        let mut pos = [
            rect[0] + border_size[0] * (1.0 - self.image.image_scale) / 2.0
                + border_size[0] * self.image.normalized_origin[0],
            rect[1] + border_size[1] * (1.0 - self.image.image_scale) / 2.0
                + border_size[1] * self.image.normalized_origin[1],
        ]; // `:61`
        if self.image.remove_floating_points_from_draw_position {
            pos = [pos[0].floor(), pos[1].floor()]; // `:62-65`
        }
        ImageDraw::At {
            pos,
            origin: [
                border_size[0] * self.image.normalized_origin[0],
                border_size[1] * self.image.normalized_origin[1],
            ], // `:66`
            scale: self.image.image_scale,
            rotation: self.image.rotation,
        }
    }
}

/// `UISlicedImage` (`GameContent/UI/Elements/UISlicedImage.cs:8-67`),
/// research/18b row 51 (W4-d83): a 9-slice image whose draw is one
/// `Utils.DrawSplicedPanel` (`:46-50`, the `uistate::spliced_rects` table).
/// Unlike `UIImage.SetImage`, its `SetImage` (`:41-44`) swaps the texture
/// **without** resizing — only the ctor pins the size (`:37-38`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlicedImage {
    /// `Width`/`Height` — the ctor's texture size (`:37-38`).
    pub size: [f32; 2],
    /// `_color` (`:12`) — **no initializer**: the C# field defaults to
    /// `default(Color)` = transparent black `(0,0,0,0)`, so an un-set
    /// sliced image draws nothing visible. Not White.
    pub color: [u8; 4],
    /// `_leftSliceDepth`/`_rightSliceDepth`/`_topSliceDepth`/
    /// `_bottomSliceDepth` (`:14-20`), all 0 until a `SetSliceDepths`.
    pub slices: [i32; 4], // [left, right, top, bottom]
}

impl SlicedImage {
    /// The ctor (`:34-39`): size = texture, colour transparent, slices 0.
    pub fn new(texture_size: [f32; 2]) -> Self {
        Self {
            size: texture_size,
            color: [0, 0, 0, 0],
            slices: [0; 4],
        }
    }

    /// `SetSliceDepths(top, bottom, left, right)` (`:52-58`) — note the
    /// parameter order is **top/bottom first**, the fields left/right first.
    pub fn set_slice_depths(&mut self, top: i32, bottom: i32, left: i32, right: i32) {
        self.slices = [left, right, top, bottom];
    }

    /// `SetSliceDepths(fluff)` (`:60-66`): all four the same.
    pub fn set_slice_fluff(&mut self, fluff: i32) {
        self.slices = [fluff; 4];
    }

    /// `DrawSelf` (`:46-50`): the int-truncated rect through
    /// `uistate::spliced_rects` with the four depths and `_color`. Returns
    /// the truncated rect + depths in the helper's `(l, r, t, b)` order.
    pub fn draw_rect(&self, rect: [f32; 4]) -> ([i32; 4], [i32; 4]) {
        (
            [rect[0] as i32, rect[1] as i32, rect[2] as i32, rect[3] as i32],
            self.slices,
        )
    }
}

/// `UIToggleImage` (`GameContent/UI/Elements/UIToggleImage.cs:8-71`),
/// research/18b row 55 (W4-d83): the two-frame toggle button — one sheet,
/// an on/off source offset each, drawn at the ctor's fixed size in
/// White (hover) or Silver (rest). The achievements filter buttons'
/// immediate draw is landed (`achievements.rs:356-367`, the 34 px stride);
/// this is the element state the other screens (workshop, resource packs)
/// adopt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToggleImage {
    /// `_drawWidth`/`_drawHeight` (`:14-16`) — also the element size
    /// (`:34-35`) and the source rect's size (`:53`).
    pub draw_size: [i32; 2],
    /// `_onTextureOffset` (`:18`).
    pub on_offset: [i32; 2],
    /// `_offTextureOffset` (`:20`).
    pub off_offset: [i32; 2],
    /// `_isOn` (`:22`) — starts false (no ctor assignment).
    pub is_on: bool,
}

impl ToggleImage {
    /// The ctor (`:26-36`): both textures are the *same* sheet; only the
    /// offsets differ. Size = `(width, height)`, state off.
    pub fn new(width: i32, height: i32, on_offset: [i32; 2], off_offset: [i32; 2]) -> Self {
        Self {
            draw_size: [width, height],
            on_offset,
            off_offset,
            is_on: false,
        }
    }

    /// `LeftClick` (`:56-60`): toggle **before** the base bubble.
    pub fn left_click(&mut self) {
        self.toggle();
    }

    /// `SetState` (`:62-65`) / `Toggle` (`:67-70`).
    pub fn set_state(&mut self, value: bool) {
        self.is_on = value;
    }

    pub fn toggle(&mut self) {
        self.is_on = !self.is_on;
    }

    /// `DrawSelf` (`:38-54`): the state picks the offset, the destination
    /// is `(int)X, (int)Y` at the fixed draw size, the source is
    /// `(offset.X, offset.Y, drawW, drawH)`, the tint White when hovering
    /// else Silver. Returns `(dest, src, tint)`.
    pub fn draw_plan(&self, rect: [f32; 4], hovering: bool) -> ([i32; 4], [i32; 4], [u8; 4]) {
        let point = if self.is_on { self.on_offset } else { self.off_offset };
        (
            [rect[0] as i32, rect[1] as i32, self.draw_size[0], self.draw_size[1]],
            [point[0], point[1], self.draw_size[0], self.draw_size[1]],
            if hovering {
                [255, 255, 255, 255]
            } else {
                [192, 192, 192, 255] // `Color.Silver`
            },
        )
    }
}

#[cfg(test)]
mod element_tests {
    use super::*;

    /// `UIToggleImage` (`UIToggleImage.cs:26-70`): the achievements filter
    /// geometry — 32×32 frames at (34j, 0)/(34j, 34) — flips on click, the
    /// rest tint is Silver, and the destination truncates.
    #[test]
    fn the_toggle_flips_frames_and_dims_at_rest() {
        let mut t = ToggleImage::new(32, 32, [34, 0], [34, 34]);
        assert!(!t.is_on, "the ctor leaves it off");
        let (dst, src, tint) = t.draw_plan([10.9, 20.8, 99.0, 99.0], false);
        assert_eq!(dst, [10, 20, 32, 32], "the fixed draw size, not the rect");
        assert_eq!(src, [34, 34, 32, 32]);
        assert_eq!(tint, [192, 192, 192, 255]);
        t.left_click();
        assert!(t.is_on);
        let (_, src, tint) = t.draw_plan([0.0, 0.0, 32.0, 32.0], true);
        assert_eq!(src, [34, 0, 32, 32]);
        assert_eq!(tint, [255, 255, 255, 255]);
        t.set_state(false);
        assert!(!t.is_on);
        t.toggle();
        assert!(t.is_on);
    }

    /// `UISlicedImage` (`UISlicedImage.cs:34-66`): the ctor sizes to the
    /// texture and leaves the colour transparent; the 4-arg setter takes
    /// top/bottom first; the draw truncates the rect.
    #[test]
    fn the_sliced_image_defaults_transparent_and_orders_top_first() {
        let mut s = SlicedImage::new([44.0, 44.0]);
        assert_eq!(s.size, [44.0, 44.0]);
        assert_eq!(s.color, [0, 0, 0, 0], "default(Color), not White (`:12`)");
        s.set_slice_depths(1, 2, 3, 4); // top, bottom, left, right
        assert_eq!(s.slices, [3, 4, 1, 2]); // [left, right, top, bottom]
        s.set_slice_fluff(10);
        assert_eq!(s.slices, [10; 4]);
        let (r, d) = s.draw_rect([10.9, 20.8, 100.7, 30.2]);
        assert_eq!(r, [10, 20, 100, 30]);
        assert_eq!(d, [10; 4]);
    }


    /// The default draw (`UIImage.cs:93-98`): scale 1, origin 0 — the position
    /// is the element position untouched, the origin zero; `SetImage` sizes
    /// the element to the sheet (`:52-56`) and honours the resize gate.
    #[test]
    fn the_default_image_draws_at_the_element_position() {
        let mut img = ImageElement::default();
        assert_eq!(img.set_image([30.0, 20.0]), Some([30.0, 20.0]));
        assert_eq!(img.size, [30.0, 20.0]);
        assert_eq!(
            img.draw_geometry([100.5, 50.25, 30.0, 20.0], [30.0, 20.0]),
            ImageDraw::At {
                pos: [100.5, 50.25],
                origin: [0.0, 0.0],
                scale: 1.0,
                rotation: 0.0,
            }
        );
        img.allow_resizing_dimensions = false;
        assert_eq!(img.set_image([64.0, 64.0]), None);
        assert_eq!(img.size, [30.0, 20.0], "the gate keeps the old size");
    }

    /// The centring maths (`UIImage.cs:93`): at scale 0.5 with a 40x20 texture
    /// the pos shifts by `(10, 5)`; `NormalizedOrigin (0.5, 0.5)` adds another
    /// `(20, 10)` and the draw origin becomes `(20, 10)` (`:98`). The floor
    /// gate (`:94-97`) truncates the fraction; `UseTextureSizeForOrigin=false`
    /// switches the vector to the element size (`:88-92`).
    #[test]
    fn scale_origin_and_floor_follow_the_draw_self_maths() {
        let mut img = ImageElement::default();
        img.image_scale = 0.5;
        let g = img.draw_geometry([10.0, 10.0, 100.0, 100.0], [40.0, 20.0]);
        assert_eq!(
            g,
            ImageDraw::At { pos: [20.0, 15.0], origin: [0.0, 0.0], scale: 0.5, rotation: 0.0 }
        );
        img.normalized_origin = [0.5, 0.5];
        let g = img.draw_geometry([10.0, 10.0, 100.0, 100.0], [40.0, 20.0]);
        assert_eq!(
            g,
            ImageDraw::At { pos: [40.0, 25.0], origin: [20.0, 10.0], scale: 0.5, rotation: 0.0 }
        );
        img.remove_floating_points_from_draw_position = true;
        let g = img.draw_geometry([10.7, 10.9, 100.0, 100.0], [40.0, 20.0]);
        assert_eq!(
            g,
            ImageDraw::At { pos: [40.0, 25.0], origin: [20.0, 10.0], scale: 0.5, rotation: 0.0 }
        );
        img.use_texture_size_for_origin = false;
        let g = img.draw_geometry([0.0, 0.0, 100.0, 100.0], [40.0, 20.0]);
        assert_eq!(
            g,
            ImageDraw::At { pos: [75.0, 75.0], origin: [20.0, 10.0], scale: 0.5, rotation: 0.0 }
        );
    }

    /// `ScaleToFit` (`UIImage.cs:82-86`): the destination is `ToRectangle()`
    /// — every component int-truncated (`CalculatedStyle.cs:17`).
    #[test]
    fn scale_to_fit_truncates_the_destination_rectangle() {
        let mut img = ImageElement::default();
        img.scale_to_fit = true;
        assert_eq!(
            img.draw_geometry([10.9, 20.8, 30.7, 40.6], [64.0, 64.0]),
            ImageDraw::Fit { rect: [10, 20, 30, 40] }
        );
    }

    /// `UIImageFramed`'s grid slice (`UIImageFramed.cs:40-43` →
    /// `Utils.cs:1344-1349` + `:1328-1333`): a 100x60 sheet in 3x2 frames,
    /// cell (1, 1) with offsets (+2, -4) — integer division 33x30, pos
    /// (33, 30), size (35, 26); the element size follows the frame.
    #[test]
    fn the_framed_grid_slice_divides_and_offsets_like_utils() {
        let mut f = FramedImage::new([0, 0, 10, 10]);
        assert_eq!(f.size, [10.0, 10.0]);
        assert_eq!(f.color, [255, 255, 255, 255]);
        f.set_frame_grid([100, 60], 3, 2, 1, 1, 2, -4);
        assert_eq!(f.frame, [33, 30, 35, 26]);
        assert_eq!(f.size, [35.0, 26.0]);
        f.set_frame([5, 6, 7, 8]);
        assert_eq!(f.size, [7.0, 8.0], "SetFrame re-pins the size (`:36-37`)");
    }

    /// `UIImageWithBorder` (`UIImageWithBorder.cs:14-40`): the ctor's
    /// `SetBorder` overwrites the image's size with the border's, and the
    /// border draw centres on the **border** size even when the base image
    /// uses the element size for its own origin (`:60-61` vs `UIImage.cs:89`).
    #[test]
    fn the_border_sizes_the_element_and_centres_on_itself() {
        let mut b = BorderedImage::new([20.0, 20.0], [30.0, 30.0]);
        assert_eq!(b.image.size, [30.0, 30.0], "SetBorder wins the ctor size");
        b.image.image_scale = 0.5;
        b.image.use_texture_size_for_origin = false;
        // Base image: element-size vector (100x100) -> pos (25, 25).
        assert_eq!(
            b.image.draw_geometry([0.0, 0.0, 100.0, 100.0], [20.0, 20.0]),
            ImageDraw::At { pos: [25.0, 25.0], origin: [0.0, 0.0], scale: 0.5, rotation: 0.0 }
        );
        // Border: always its own 30x30 -> pos (7.5, 7.5).
        assert_eq!(
            b.border_draw_geometry([0.0, 0.0, 100.0, 100.0], [30.0, 30.0]),
            ImageDraw::At { pos: [7.5, 7.5], origin: [0.0, 0.0], scale: 0.5, rotation: 0.0 }
        );
        assert_eq!(b.set_border([40.0, 40.0]), [40.0, 40.0]);
    }
}
