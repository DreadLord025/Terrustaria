//! The modern-crafting-UI panel primitives: `UIPanel`'s 9-slice,
//! `UIHorizontalSeparator` and `UIScrollbar`, ported from
//! `decompiled/Terraria/GameContent/UI/Elements/`.
//!
//! Every draw takes the sheet as an `Option`; a `None` sheet draws nothing rather
//! than panicking, so `Lookups::default()` renders a blank frame.

use tr_render::{SheetRect, SpriteRenderer};

use super::color;
use crate::assets;

/// `UIPanel._cornerSize` (`UIPanel.cs:10`).
pub const CORNER: i32 = 12;
/// `UIPanel._barSize` (`UIPanel.cs:12`).
pub const BAR: i32 = 4;

/// Pushes one sprite from a sub-rectangle `src` (in the sheet's own pixels) at the
/// destination `dst` (`[x, y, w, h]`). Draws nothing for a zero-sized piece.
fn push_sheet(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    sheet: &SheetRect,
    dst: [f32; 4],
    src: [f32; 4],
    color: [f32; 4],
) {
    if dst[2] <= 0.0 || dst[3] <= 0.0 {
        return;
    }
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let x0 = sheet.x as f32 + src[0];
    let y0 = sheet.y as f32 + src[1];
    let uv = [x0 / aw, y0 / ah, (x0 + src[2]) / aw, (y0 + src[3]) / ah];
    r.push_ui_sprite([dst[0], dst[1]], [dst[2], dst[3]], uv, color);
}

/// `UIPanel.DrawPanel` (`UIPanel.cs:50-66`) destination rects: 12 px corners, 4 px
/// edge strips read from the sheet, and a stretched centre. `rect` is the panel's
/// outer rect; the original truncates its origin/size to ints
/// (`point = ((int)X, (int)Y)`, `point2 = point + (int)size - corner`).
///
/// Order matches the draw calls: TL, TR, BL, BR, top, bottom, left, right, centre.
pub fn panel_rects(rect: [f32; 4]) -> [[f32; 4]; 9] {
    // `UIPanel.cs:53-56`.
    let x = rect[0] as i32;
    let y = rect[1] as i32;
    let x2 = x + rect[2] as i32 - CORNER;
    let y2 = y + rect[3] as i32 - CORNER;
    let w = (x2 - x - CORNER) as f32;
    let h = (y2 - y - CORNER) as f32;
    let (x, y, x2, y2) = (x as f32, y as f32, x2 as f32, y2 as f32);
    let c = CORNER as f32;
    [
        [x, y, c, c],              // 57: TL   src (0, 0)
        [x2, y, c, c],             // 58: TR   src (corner+bar, 0)
        [x, y2, c, c],             // 59: BL   src (0, corner+bar)
        [x2, y2, c, c],            // 60: BR   src (corner+bar, corner+bar)
        [x + c, y, w, c],          // 61: top edge, src (corner, 0, bar, corner)
        [x + c, y2, w, c],         // 62: bottom edge
        [x, y + c, c, h],          // 63: left edge
        [x2, y + c, c, h],         // 64: right edge
        [x + c, y + c, w, h],      // 65: centre (stretched `bar x bar` block)
    ]
}

/// The matching source rects on a `sheet_w x sheet_h` texture. The original
/// hard-codes the offsets (`UIPanel.cs:57-65`): `0`, `corner`, `corner + bar`; a
/// 28x28 sheet (`2 * corner + bar`) is assumed. The sizes are carried for parity
/// with the pure-helper signature.
pub fn panel_srcs(sheet_w: f32, sheet_h: f32) -> [[f32; 4]; 9] {
    let _ = (sheet_w, sheet_h);
    let (c, b) = (CORNER as f32, BAR as f32);
    [
        [0.0, 0.0, c, c],
        [c + b, 0.0, c, c],
        [0.0, c + b, c, c],
        [c + b, c + b, c, c],
        [c, 0.0, b, c],
        [c, c + b, b, c],
        [0.0, c, c, b],
        [c + b, c, c, b],
        [c, c, b, b],
    ]
}

/// `UIPanel.DrawPanel` (`UIPanel.cs:50-66`): draws the 9-slice of `sheet` over
/// `rect`. `sheet == None` draws nothing.
pub fn draw_panel(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    sheet: Option<SheetRect>,
    rect: [f32; 4],
    color: [f32; 4],
) {
    let Some(sheet) = sheet else {
        return;
    };
    let dst = panel_rects(rect);
    let src = panel_srcs(sheet.width as f32, sheet.height as f32);
    for i in 0..9 {
        push_sheet(r, lookups, &sheet, dst[i], src[i], color);
    }
}

/// `UIHorizontalSeparator` + `Utils.DrawPanel(tex, edgeWidth=2, edgeShove=0, ...)`
/// (`UIHorizontalSeparator.cs:31-35`, `Utils.cs:2612-2617`): three pieces —
/// `edgeWidth` left, stretched middle, `edgeWidth` right — all the sheet's height.
///
/// The height element of each rect is a placeholder: `UIHorizontalSeparator`'s
/// height *is* the sheet height (`UIHorizontalSeparator.cs:28`), which
/// `draw_horizontal_separator` substitutes.
pub fn separator_rects(x: f32, y: f32, width: f32) -> [[f32; 4]; 3] {
    // `Utils.cs:2614-2616` with `edgeWidth = 2`, `edgeShove = 0`.
    [
        [x, y, 2.0, 0.0],
        [x + 2.0, y, width - 4.0, 0.0],
        [x + width - 2.0, y, 2.0, 0.0],
    ]
}

/// The matching source rects on a `sheet_w x sheet_h` separator texture
/// (`Utils.cs:2614-2616`).
pub fn separator_srcs(sheet_w: f32, sheet_h: f32) -> [[f32; 4]; 3] {
    [
        [0.0, 0.0, 2.0, sheet_h],
        [2.0, 0.0, sheet_w - 4.0, sheet_h],
        [sheet_w - 2.0, 0.0, 2.0, sheet_h],
    ]
}

/// `UIHorizontalSeparator.DrawSelf` (`UIHorizontalSeparator.cs:31-35`): three pieces
/// of `ui_separator` (`Images/UI/CharCreation/Separator1`). Missing sheet draws
/// nothing.
pub fn draw_horizontal_separator(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    x: f32,
    y: f32,
    width: f32,
    color: [f32; 4],
) {
    let Some(sheet) = lookups.ui_separator else {
        return;
    };
    let rects = separator_rects(x, y, width);
    let srcs = separator_srcs(sheet.width as f32, sheet.height as f32);
    let sh = sheet.height as f32;
    for i in 0..3 {
        let dst = [rects[i][0], rects[i][1], rects[i][2], sh];
        push_sheet(r, lookups, &sheet, dst, srcs[i], color);
    }
}

/// `UIScrollbar.GetHandleRectangle` (`UIScrollbar.cs:85-94`): the outer bar rect and
/// `(bar, handle)`. `view`/`max_view` are `_viewSize`/`_maxViewSize`; `_viewPosition`
/// is 0 in P3. Width is 20 (`UIScrollbar.cs:59`); the inner rect drops the 5 px
/// `PaddingTop`/`PaddingBottom` (`:68-69`).
pub fn scrollbar_rects(rect: [f32; 4], view: f32, max_view: f32) -> ([f32; 4], [f32; 4]) {
    // `dimensions.ToRectangle()` (`:121`, `CalculatedStyle.cs:17`) truncates to ints.
    let bar = [
        rect[0].trunc(),
        rect[1].trunc(),
        rect[2].trunc(),
        rect[3].trunc(),
    ];
    let (mut view, mut max_view) = (view, max_view);
    if max_view == 0.0 && view == 0.0 {
        // `UIScrollbar.cs:88-92`.
        view = 1.0;
        max_view = 1.0;
    }
    let ix = bar[0];
    let iy = bar[1] + 5.0;
    let ih = bar[3] - 10.0;
    // `Rectangle((int)inner.X, (int)(inner.Y + inner.Height * (viewPos/maxView)) - 3,
    // 20, (int)(inner.Height * (viewSize/maxView)) + 7)` (`:93`), `viewPos = 0`.
    let hy = (iy + ih * (0.0 / max_view)).trunc() - 3.0;
    let hh = (ih * (view / max_view)).trunc() + 7.0;
    (bar, [ix, hy, 20.0, hh])
}

/// `UIScrollbar.CanScroll` (`UIScrollbar.cs:49`): `_maxViewSize != _viewSize`.
pub fn scrollbar_can_scroll(view: f32, max_view: f32) -> bool {
    max_view != view
}

/// `UIScrollbar.DrawBar` (`UIScrollbar.cs:96-101`): a 6 px cap, a stretched 4 px body
/// and a 6 px base, all the sheet's full width.
fn draw_scrollbar_bar(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    sheet: &SheetRect,
    dim: [f32; 4],
    color: [f32; 4],
) {
    let w = sheet.width as f32;
    let h = sheet.height as f32;
    let pieces = [
        ([dim[0], dim[1] - 6.0, dim[2], 6.0], [0.0, 0.0, w, 6.0]),
        ([dim[0], dim[1], dim[2], dim[3]], [0.0, 6.0, w, 4.0]),
        ([dim[0], dim[1] + dim[3], dim[2], 6.0], [0.0, h - 6.0, w, 6.0]),
    ];
    for (dst, src) in pieces {
        push_sheet(r, lookups, sheet, dst, src, color);
    }
}

/// `UIScrollbar.DrawSelf` (`UIScrollbar.cs:103-124`): the cyan track
/// (`ui_scrollbar`, `Scrollbar2`) and the handle (`ui_scrollbar_inner`,
/// `ScrollbarInner`). Draws nothing when `auto_hide && !can_scroll` (`:105`).
pub fn draw_scrollbar(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    rect: [f32; 4],
    view: f32,
    max_view: f32,
    auto_hide: bool,
) {
    if auto_hide && !scrollbar_can_scroll(view, max_view) {
        return;
    }
    let (bar, handle) = scrollbar_rects(rect, view, max_view);
    if let Some(sheet) = lookups.ui_scrollbar {
        draw_scrollbar_bar(r, lookups, &sheet, bar, [1.0; 4]);
    }
    if let Some(sheet) = lookups.ui_scrollbar_inner {
        // `Color.White * 0.85f` when not dragging/hovering (`UIScrollbar.cs:122`).
        let c = color::tint(color::mul([255, 255, 255, 255], 0.85));
        draw_scrollbar_bar(r, lookups, &sheet, handle, c);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_rects_are_the_ui_panel_9_slice() {
        // `UIPanel.cs:52-56` with the P3 panel outer rect (20, 350, 490, 512):
        // `point2 = (20 + 490 - 12, 350 + 512 - 12) = (498, 850)`, edge `width =
        // 498 - 20 - 12 = 466`, `height = 850 - 350 - 12 = 488`.
        let r = panel_rects([20.0, 350.0, 490.0, 512.0]);
        assert_eq!(r[0], [20.0, 350.0, 12.0, 12.0]); // TL
        assert_eq!(r[1], [498.0, 350.0, 12.0, 12.0]); // TR
        assert_eq!(r[2], [20.0, 850.0, 12.0, 12.0]); // BL
        assert_eq!(r[3], [498.0, 850.0, 12.0, 12.0]); // BR
        assert_eq!(r[4], [32.0, 350.0, 466.0, 12.0]); // top edge
        assert_eq!(r[5], [32.0, 850.0, 466.0, 12.0]); // bottom edge
        assert_eq!(r[6], [20.0, 362.0, 12.0, 488.0]); // left edge
        assert_eq!(r[7], [498.0, 362.0, 12.0, 488.0]); // right edge
        assert_eq!(r[8], [32.0, 362.0, 466.0, 488.0]); // centre
    }

    #[test]
    fn panel_rects_truncate_the_origin_and_size() {
        // A fractional rect is truncated to ints first (`UIPanel.cs:53-54`).
        let r = panel_rects([20.9, 350.9, 490.9, 512.9]);
        assert_eq!(r[0], [20.0, 350.0, 12.0, 12.0]);
        assert_eq!(r[3], [498.0, 850.0, 12.0, 12.0]);
    }

    #[test]
    fn panel_srcs_use_the_corner_and_bar_offsets() {
        // `UIPanel.cs:57-65`: offsets 0, corner (12), corner+bar (16).
        let s = panel_srcs(28.0, 28.0);
        assert_eq!(s[0], [0.0, 0.0, 12.0, 12.0]);
        assert_eq!(s[1], [16.0, 0.0, 12.0, 12.0]);
        assert_eq!(s[2], [0.0, 16.0, 12.0, 12.0]);
        assert_eq!(s[3], [16.0, 16.0, 12.0, 12.0]);
        assert_eq!(s[4], [12.0, 0.0, 4.0, 12.0]);
        assert_eq!(s[5], [12.0, 16.0, 4.0, 12.0]);
        assert_eq!(s[6], [0.0, 12.0, 12.0, 4.0]);
        assert_eq!(s[7], [16.0, 12.0, 12.0, 4.0]);
        assert_eq!(s[8], [12.0, 12.0, 4.0, 4.0]);
    }

    #[test]
    fn separator_rects_are_the_three_draw_panel_pieces() {
        // `NewCraftingUI` separator (36, 439, 462) through `Utils.DrawPanel(tex, 2, 0)`.
        let r = separator_rects(36.0, 439.0, 462.0);
        assert_eq!([r[0][0], r[0][1], r[0][2]], [36.0, 439.0, 2.0]);
        assert_eq!([r[1][0], r[1][1], r[1][2]], [38.0, 439.0, 458.0]);
        assert_eq!([r[2][0], r[2][1], r[2][2]], [496.0, 439.0, 2.0]);
        // `Separator1` is 6x4: the right 2 px start at `sheet_w - 2 = 4`.
        let s = separator_srcs(6.0, 4.0);
        assert_eq!(s[0], [0.0, 0.0, 2.0, 4.0]);
        assert_eq!(s[1], [2.0, 0.0, 2.0, 4.0]);
        assert_eq!(s[2], [4.0, 0.0, 2.0, 4.0]);
    }

    #[test]
    fn scrollbar_can_scroll_matches_the_source() {
        // `UIScrollbar.cs:49`: `_maxViewSize != _viewSize`.
        assert!(!scrollbar_can_scroll(1.0, 1.0));
        assert!(scrollbar_can_scroll(1.0, 2.0));
        assert!(!scrollbar_can_scroll(0.0, 0.0));
    }

    #[test]
    fn scrollbar_handle_uses_the_inner_dimensions() {
        // `UIScrollbar.cs:93` with the P3 scrollbar rect (482, 454, 20, 392),
        // view = max_view = 1: inner y = 454 + 5, inner h = 392 - 10.
        let (bar, handle) = scrollbar_rects([482.0, 454.0, 20.0, 392.0], 1.0, 1.0);
        assert_eq!(bar, [482.0, 454.0, 20.0, 392.0]);
        assert_eq!(handle, [482.0, 459.0 - 3.0, 20.0, 382.0 + 7.0]);
    }
}
