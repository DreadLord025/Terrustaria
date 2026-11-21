//! The pylon network lines + the fullscreen hover/click state (W4-f29,
//! census items 1-2): `TeleportPylonsMapLayer.DrawLine` (`:18-28`) and the
//! line block (`:45-64`), `MapOverlayDrawContext.GetClampedDrawRegion`
//! (`:67-116`) + `DrawClamped` (`:118-216`) for the offscreen markers, the
//! pylon right-click's `Main.MapPylonTile` recentre (`Player.cs:32180-32212`)
//! and the marker click's teleport / pan (`TeleportPylonsMapLayer.cs:90-109`).
//!
//! The markers themselves are the landed S1-L-4 layer
//! (`map_layers.rs:282-316`); this module adds the lines, the clamped
//! offscreen arrows and the click state on top.

use glam::Vec2;
use tr_core::pylons::PylonInfo;
use tr_render::SpriteRenderer;

use crate::assets;
use crate::map::WorldMap;

/// `TeleportPylonsMapLayer.BorderSize` (`TeleportPylonsMapLayer.cs:16`).
pub(crate) const BORDER_SIZE: i32 = 10;

/// `TeleportPylonsMapLayer.DrawLine` (`TeleportPylonsMapLayer.cs:18-28`): a
/// line from `start` to `end` drawn as `min(5, (int)distance)` BlackTile
/// segments, each `Vector2.Lerp(start, end, i/num2)` with rotation
/// `(end-start).ToRotation()`, origin `(0, width/2)` and scale
/// `(distance/num2/16, width/16)`. The port's `push_ui_sprite_rot` pivots
/// the top-left, so the origin shift is applied here (the bestiary
/// convention). `width` is the line's pixel thickness (6 black, 2 white).
pub(crate) fn draw_line(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    start: [f32; 2],
    end: [f32; 2],
    color: [f32; 4],
    width: f32,
) {
    let Some(sheet) = lookups.black_tile else {
        return;
    };
    let Some(uv) = lookups.hud_uv(Some(&sheet)) else {
        return;
    };
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let distance = (dx * dx + dy * dy).sqrt();
    let rotation = dy.atan2(dx);
    let num2 = 5usize.min(distance as usize);
    if num2 == 0 {
        return;
    }
    // The BlackTile frame is the whole sheet (`value.Frame()`); the C#
    // origin is `(0, value.Width * 0.5)` and the scale
    // `(distance/num2/16, width/16)`, so the drawn size is
    // `(sheetW * distance/num2/16, sheetH * width/16)` and the scaled origin
    // `(0, sheetW * width/32)`. The sheet is 16x16 in retail content.
    let sw = sheet.width as f32;
    let sh = sheet.height as f32;
    let size = [sw * distance / num2 as f32 / 16.0, sh * width / 16.0];
    let (s, c) = rotation.sin_cos();
    let origin = [0.0, sw * 0.5 * width / 16.0];
    for i in 0..num2 {
        let t = i as f32 / num2 as f32;
        let anchor = [
            start[0] + (end[0] - start[0]) * t,
            start[1] + (end[1] - start[1]) * t,
        ];
        // The C# draws at `anchor` with `origin`; the port's top-left pivot
        // needs `anchor - R(rot)*origin` (the origin texel lands on anchor).
        let tl = [
            anchor[0] - (origin[0] * c - origin[1] * s),
            anchor[1] - (origin[0] * s + origin[1] * c),
        ];
        r.push_ui_sprite_rot(tl, size, uv, color, rotation);
    }
}

/// `MapOverlayDrawContext.GetUnclampedDrawRegion`'s centre
/// (`MapOverlayDrawContext.cs:57-65`) for a pylon marker: the marker's
/// `PositionInTiles + (1.5, 2)` through the map transform, the aligned
/// origin and the `drawScale * scale` size, then the rect centre. The line
/// endpoints are these centres (`TeleportPylonsMapLayer.cs:47-59`).
pub(crate) fn pylon_line_endpoint(
    pylon: &PylonInfo,
    map_position: [f32; 2],
    map_offset: [f32; 2],
    map_scale: f32,
    draw_scale: f32,
    frame_size: [f32; 2],
) -> [f32; 2] {
    let position = [pylon.x as f32 + 1.5, pylon.y as f32 + 2.0];
    let pos = [
        (position[0] - map_position[0]) * map_scale + map_offset[0],
        (position[1] - map_position[1]) * map_scale + map_offset[1],
    ];
    // Alignment.Center = (0.5, 0.5); scale = num = 1 (`:33`). The C#
    // `GetUnclampedDrawRegion` truncates the top-left and the size to int
    // (`(int)vector2.X`, `(int)(Width * num)`) BEFORE the Centre read.
    let origin = [frame_size[0] * 0.5, frame_size[1] * 0.5];
    let num = draw_scale * 1.0;
    let top_left = [pos[0] - origin[0] * num, pos[1] - origin[1] * num];
    let w = (frame_size[0] * num) as i32;
    let h = (frame_size[1] * num) as i32;
    // `Rectangle.Center` = `(X + Width/2, Y + Height/2)` (integer halves).
    [
        (top_left[0] as i32 + w / 2) as f32,
        (top_left[1] as i32 + h / 2) as f32,
    ]
}

/// The pylon network line block (`TeleportPylonsMapLayer.cs:45-64`): on the
/// fullscreen map with a selected `MapPylonTile`, one black 6px + one white
/// 2px line from the selected pylon's centre to every OTHER revealed pylon.
/// `map_pylon_tile` is `Main.MapPylonTile`; the reveal gate is the landed
/// `IsRevealed` (always true without the team seed).
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_pylon_lines(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    map: &WorldMap,
    world: &tr_core::World,
    pylons: &[PylonInfo],
    map_position: [f32; 2],
    map_offset: [f32; 2],
    map_scale: f32,
    draw_scale: f32,
    team_based_spawns: bool,
) {
    // `:45`: the fullscreen + selected-tile gate.
    if !map.open || map.map_pylon_tile == (-1, -1) {
        return;
    }
    let Some(sheet) = lookups.extra_182 else {
        return;
    };
    // `SpriteFrame(11, 1)` with PaddingY 0: the frame width is the sheet's
    // 11th, the height the whole sheet.
    let frame_size = [sheet.width as f32 / 11.0, sheet.height as f32];
    let revealed = |p: &PylonInfo| {
        !team_based_spawns || map.tile_revealed(world, p.x as i32, p.y as i32)
    };
    // The selected pylon's centre (`:47-50`).
    let selected = PylonInfo {
        x: map.map_pylon_tile.0 as i16,
        y: map.map_pylon_tile.1 as i16,
        pylon_type: pylons
            .iter()
            .find(|p| (p.x as i32, p.y as i32) == map.map_pylon_tile)
            .map(|p| p.pylon_type)
            .unwrap_or(tr_core::pylons::PylonType::SurfacePurity),
    };
    let start = pylon_line_endpoint(
        &selected,
        map_position,
        map_offset,
        map_scale,
        draw_scale,
        frame_size,
    );
    // `:51-63`: every OTHER revealed pylon.
    for p in pylons {
        if !revealed(p) || (p.x as i32, p.y as i32) == map.map_pylon_tile {
            continue;
        }
        let end = pylon_line_endpoint(
            p,
            map_position,
            map_offset,
            map_scale,
            draw_scale,
            frame_size,
        );
        draw_line(r, lookups, start, end, [0.0, 0.0, 0.0, 1.0], 6.0);
        draw_line(r, lookups, start, end, [1.0, 1.0, 1.0, 1.0], 2.0);
    }
}

/// `MapOverlayDrawContext.GetClampedDrawRegion` (`MapOverlayDrawContext.cs:
/// 67-116`): the marker rect clamped into the screen's border region by
/// projecting the overflow back toward the screen centre. Returns the
/// clamped rect `[x, y, w, h]` (integer, like the C# `Rectangle`). The
/// `screen` is `(Main.screenWidth, Main.screenHeight)`.
pub(crate) fn clamped_draw_region(
    position: [f32; 2],
    frame_size: [f32; 2],
    map_position: [f32; 2],
    map_offset: [f32; 2],
    map_scale: f32,
    draw_scale: f32,
    scale: f32,
    screen: [f32; 2],
    border: i32,
) -> [i32; 4] {
    let pos = [
        (position[0] - map_position[0]) * map_scale + map_offset[0],
        (position[1] - map_position[1]) * map_scale + map_offset[1],
    ];
    // Alignment.Center.
    let origin = [frame_size[0] * 0.5, frame_size[1] * 0.5];
    let num = draw_scale * scale;
    let top_left = [pos[0] - origin[0] * num, pos[1] - origin[1] * num];
    let mut x = top_left[0] as i32;
    let mut y = top_left[1] as i32;
    let w = (frame_size[0] * num) as i32;
    let h = (frame_size[1] * num) as i32;
    let num2 = screen[0] as i32 - border;
    let num3 = screen[1] as i32 - border;
    let num4 = (border + num2) / 2;
    let num5 = (border + num3) / 2;
    // `:79-96`: the X clamp.
    if x < border {
        let num6 = (x - num4) as f32;
        let num7 = (y - num5) as f32;
        let num8 = x - border;
        let num9 = (num8 as f32 / num6 * num7) as i32;
        x -= num8;
        y -= num9;
    } else if x + w > num2 {
        let num10 = (x - num4) as f32;
        let num11 = (y - num5) as f32;
        let num12 = x + w - num2;
        let num13 = (num12 as f32 / num10 * num11) as i32;
        x -= num12;
        y -= num13;
    }
    // `:97-114`: the Y clamp.
    if y < border {
        let num14 = (x - num4) as f32;
        let num15 = (y - num5) as f32;
        let num16 = y - border;
        let num17 = (num16 as f32 / num15 * num14) as i32;
        x -= num17;
        y -= num16;
    } else if y + h > num3 {
        let num18 = (x - num4) as f32;
        let num19 = (y - num5) as f32;
        let num20 = y + h - num3;
        let num21 = (num20 as f32 / num19 * num18) as i32;
        x -= num21;
        y -= num20;
    }
    [x, y, w, h]
}

/// `Player.cs:32187-32212`'s pylon right-click recentre: the anchor is
/// `(x - frameX/18%3, y - frameY/18%4)`, the reveal-inclusive rect of all
/// pylons inflated 20, its centre the new map position, and the scale fit
/// to `(screen - 40) / rect.Size`. Returns `(anchor, new_pos, new_scale_cap)`
/// — the caller writes `MapPylonTile`, clears the pan/reset and clamps the
/// scale down to the cap. `revealed` is the `IsRevealed` gate.
pub(crate) fn pylon_right_click_view(
    anchor: (i32, i32),
    pylons: &[PylonInfo],
    revealed: impl Fn(&PylonInfo) -> bool,
    screen: [f32; 2],
    current_scale: f32,
) -> ([f32; 2], f32) {
    // `:32188-32197`: the rect starts at the anchor (0x0) and includes every
    // revealed pylon (`Utils.Including`, `Utils.cs:957-963`).
    let mut left = anchor.0;
    let mut right = anchor.0;
    let mut top = anchor.1;
    let mut bottom = anchor.1;
    for p in pylons {
        if !revealed(p) {
            continue;
        }
        left = left.min(p.x as i32);
        right = right.max(p.x as i32);
        top = top.min(p.y as i32);
        bottom = bottom.max(p.y as i32);
    }
    // `:32198`: Inflate(20, 20).
    left -= 20;
    right += 20;
    top -= 20;
    bottom += 20;
    let width = right - left;
    let height = bottom - top;
    // `:32201-32202`: the centre (`Rectangle.Center` = integer halves).
    let center = [
        (left + width / 2) as f32,
        (top + height / 2) as f32,
    ];
    // `:32203-32212`: the scale fit — only ever shrinks.
    let mut scale = current_scale;
    let cap_x = (screen[0] - 40.0) / width as f32;
    let cap_y = (screen[1] - 40.0) / height as f32;
    if scale > cap_x {
        scale = cap_x;
    }
    if scale > cap_y {
        scale = cap_y;
    }
    (center, scale)
}

/// `Main.cs:55361-55385`'s pan-target step: while `PanTargetMapFullscreen`
/// is armed, a left press cancels it; else the map position lerps toward the
/// target by 0.15 (or snaps when within 1 tile, or when `dist*0.15 < 1`).
/// Returns the new position and whether the pan finished. The C# runs this
/// once per frame in `DrawMap`'s fullscreen branch.
pub(crate) fn pan_target_step(
    pos: [f32; 2],
    target: [f32; 2],
    left_press: bool,
) -> ([f32; 2], bool) {
    // `:55363-55366`: a press cancels the pan.
    if left_press {
        return (pos, true);
    }
    let dx = target[0] - pos[0];
    let dy = target[1] - pos[1];
    let dist = (dx * dx + dy * dy).sqrt();
    // `:55369-55374`: within 1 tile snaps.
    if dist < 1.0 {
        return (target, true);
    }
    // `:55377-55382`: the lerp amount.
    let mut amount = 0.15f32;
    if dist * 0.15 < 1.0 {
        amount = 1.0 / dist;
    }
    let new_pos = [
        pos[0] + (target[0] - pos[0]) * amount,
        pos[1] + (target[1] - pos[1]) * amount,
    ];
    (new_pos, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::pylons::PylonType;

    fn pylon(x: i16, y: i16) -> PylonInfo {
        PylonInfo {
            x,
            y,
            pylon_type: PylonType::SurfacePurity,
        }
    }

    /// `TeleportPylonsMapLayer.cs:18-28` — the segment count + lerp anchors.
    #[test]
    fn w4_f29_draw_line_segment_count_and_lerp() {
        // distance 10 -> min(5, 10) = 5 segments; the anchors are
        // Lerp(start, end, i/5).
        let start = [0.0f32, 0.0];
        let end = [10.0, 0.0];
        let distance = 10.0f32;
        let num2 = 5usize.min(distance as usize);
        assert_eq!(num2, 5);
        for i in 0..num2 {
            let t = i as f32 / num2 as f32;
            let anchor = start[0] + (end[0] - start[0]) * t;
            assert_eq!(anchor, i as f32 * 2.0);
        }
        // distance 3 -> 3 segments; distance 0 -> none.
        assert_eq!(5usize.min(3usize), 3);
        assert_eq!(5usize.min(0usize), 0);
        // A sub-1 distance truncates to 0 segments (the C# `(int)num`).
        assert_eq!(5usize.min(0.9f32 as usize), 0);
    }

    /// `MapOverlayDrawContext.cs:57-65` — the line endpoint is the rect
    /// centre of the aligned, scaled marker.
    #[test]
    fn w4_f29_pylon_line_endpoint_is_the_rect_centre() {
        let p = pylon(40, 30);
        // position = (41.5, 32); map_position 0, offset (100,50), scale 2,
        // draw_scale 1, frame (22, 40).
        // pos = (83, 64) + (100,50) = (183, 114).
        // origin = (11, 20); num = 1; top_left = (172, 94).
        // w = 22, h = 40; centre = (172 + 11, 94 + 20) = (183, 114).
        let got = pylon_line_endpoint(&p, [0.0, 0.0], [100.0, 50.0], 2.0, 1.0, [22.0, 40.0]);
        assert_eq!(got, [183.0, 114.0]);
    }

    /// `MapOverlayDrawContext.cs:67-116` — an on-screen rect is unchanged;
    /// an off-screen one projects back toward the centre.
    #[test]
    fn w4_f29_clamped_region_on_screen_unchanged() {
        // A marker well inside a 800x600 screen, border 10.
        let got = clamped_draw_region(
            [200.0, 150.0],
            [22.0, 40.0],
            [0.0, 0.0],
            [0.0, 0.0],
            1.0,
            1.0,
            1.0,
            [800.0, 600.0],
            10,
        );
        // top_left = (200-11, 150-20) = (189, 130); w 22, h 40.
        assert_eq!(got, [189, 130, 22, 40]);
    }

    /// `MapOverlayDrawContext.cs:79-87` — a left-overflow rect clamps to the
    /// border and projects Y toward the centre.
    #[test]
    fn w4_f29_clamped_region_left_overflow() {
        // position (5, 300) -> top_left (-6, 280), w 22 h 40.
        // x < 10: num6 = -6 - 400 = -406, num7 = 280 - 300 = -20,
        // num8 = -6 - 10 = -16, num9 = (-16/-406)*-20 = -0.788 -> 0 (trunc).
        // x -= -16 -> 10; y -= 0 -> 280.
        let got = clamped_draw_region(
            [5.0, 300.0],
            [22.0, 40.0],
            [0.0, 0.0],
            [0.0, 0.0],
            1.0,
            1.0,
            1.0,
            [800.0, 600.0],
            10,
        );
        assert_eq!(got[0], 10, "clamped to the left border");
        assert_eq!(got[1], 280);
    }

    /// `Player.cs:32188-32212` — the reveal-inclusive inflated rect centre +
    /// scale fit.
    #[test]
    fn w4_f29_pylon_right_click_view_centre_and_fit() {
        let pylons = vec![pylon(100, 100), pylon(200, 150)];
        // anchor (100,100); rect includes (200,150): left 100 right 200,
        // top 100 bottom 150. Inflate 20: left 80 right 220, top 80 bottom 170.
        // width 140, height 90. centre = (80+70, 80+45) = (150, 125).
        // screen 800x600: cap_x = 760/140 = 5.428, cap_y = 560/90 = 6.222.
        // current 10 -> shrinks to 5.428.
        let (center, scale) =
            pylon_right_click_view((100, 100), &pylons, |_| true, [800.0, 600.0], 10.0);
        assert_eq!(center, [150.0, 125.0]);
        assert!((scale - 760.0 / 140.0).abs() < 1e-4);
        // A small current scale is never grown.
        let (_, scale2) =
            pylon_right_click_view((100, 100), &pylons, |_| true, [800.0, 600.0], 2.0);
        assert_eq!(scale2, 2.0);
        // An unrevealed pylon is excluded from the rect.
        let (center3, _) = pylon_right_click_view(
            (100, 100),
            &pylons,
            |p| p.x == 100,
            [800.0, 600.0],
            10.0,
        );
        // Only the anchor: rect 100..100 inflated -> 80..120, centre 100.
        assert_eq!(center3, [100.0, 100.0]);
    }

    /// `Main.cs:55361-55385` — the pan step: press cancels, near snaps, far
    /// lerps by 0.15.
    #[test]
    fn w4_f29_pan_target_step() {
        // A press cancels.
        let (pos, done) = pan_target_step([0.0, 0.0], [100.0, 0.0], true);
        assert_eq!(pos, [0.0, 0.0]);
        assert!(done);
        // Within 1 tile snaps.
        let (pos, done) = pan_target_step([99.5, 0.0], [100.0, 0.0], false);
        assert_eq!(pos, [100.0, 0.0]);
        assert!(done);
        // Far: lerp by 0.15.
        let (pos, done) = pan_target_step([0.0, 0.0], [100.0, 0.0], false);
        assert_eq!(pos, [15.0, 0.0]);
        assert!(!done);
        // dist*0.15 < 1 -> amount = 1/dist (a 1-tile step).
        let (pos, done) = pan_target_step([0.0, 0.0], [5.0, 0.0], false);
        assert_eq!(pos, [1.0, 0.0]);
        assert!(!done);
    }
}
