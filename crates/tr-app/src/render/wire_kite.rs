//! The Wire Kite's placement-ghost DRAW (S1-W4-g81 residuals) —
//! `Main.DrawProj`'s `proj.type == 651` arm (`Main.cs:31620-31907`), the
//! over-wire cached projectile (`drawLayer = 5`, `Projectile.cs:6999` →
//! `Main.CacheProjDraws` `:38177-38178` → `DrawCachedProjs` `:62315`,
//! right after `DrawWires` `:62303`).
//!
//! The ghost previews the mass-wire operation's L-path from the anchor
//! tile `(ai[0], ai[1])` to the kite's cell: every path cell draws the
//! 651 sheet's row-0 mask quad in the channel-mix colour and the row-1
//! mask quad in the grey; with the Actuator channel on, each cell also
//! stamps `WireUi[11]` UNDER the pair. The cutter channel swaps the grey
//! to (50,50,50,255).
//!
//! The colour mix (`:31629-31658`) is XNA byte arithmetic: `num246`
//! starts at 1 and increments per enabled channel (Red, Blue, Green —
//! in THAT order), each `Lerp(color47, channelColor, 1f / num246)`; the
//! Yellow lerp uses `1f / (num246 + 1f)` WITHOUT incrementing (verbatim
//! quirk). `color47.A = 0` at the end (`:31657`) — the C# batch is
//! `BlendState.AlphaBlend` (premultiplied `One, OneMinusSrcAlpha`,
//! `Main.cs:38213`/`:62294`), so an alpha-0 tint with non-zero RGB is an
//! ADDITIVE term (`src + dst`): the ghost glows over the world. The
//! port's pipeline is the same equation (`sprite.wgsl` keeps the rgb
//! un-premultiplied, alpha = `tex.a * color.a`), so the C# alpha values
//! reproduce the additive ghost verbatim (the seal fix, research/283 —
//! the earlier alpha-255 override turned it into a semi-transparent
//! REPLACE).
//!
//! Deviations (ANALYSIS "The Wire Kite ghost (S1-W4-g81)"):
//! * the ghost pushes to the port's UI range (pre-transformed screen
//!   pixels) rather than a world batch — the C# draws it in the world
//!   pass (`DrawCachedProjsOverWiresUI`, `Main.cs:57034`, right after
//!   `DrawWires` `:57030`, before the projectile/player passes). The
//!   port's push order matches (after `draw_wires`, before the generic
//!   projectile walk), but the UI range draws OVER the scene filter
//!   where the C# ghost draws UNDER it — the recorded filter-order
//!   deviation;
//! * the L-path's corner/middle mask picks are ported verbatim including
//!   the `num245 = 1` sign multipliers (the `* num245` terms are
//!   identity — kept for the citation).

use glam::Vec2;
use tr_core::smart_cursor::multi_tool_mode as mode;
use tr_render::SpriteRenderer;

use crate::assets;
use crate::projectile::Projectile;

/// The ghost's channel-mix colour (`Main.cs:31629-31658`): the grey base
/// (127,127,127), the per-channel byte lerps in the C#'s order, and the
/// cutter's grey swap. Returns `(color47, color46)` — the mask tint and
/// the grey/cutter tint. `color47` alpha 0 (the additive ghost), `color46`
/// alpha 0 normally / 255 with the cutter (the C# values, `:31657`).
pub fn ghost_colors(tool_mode: u8) -> ([u8; 4], [u8; 4]) {
    let mut color46 = [255u8, 255, 255, 0];
    let mut color47 = [127u8, 127, 127, 0];
    let mut num246 = 1.0f32;
    if tool_mode & mode::RED != 0 {
        num246 += 1.0;
        color47 = lerp_bytes(color47, [255, 0, 0, 255], 1.0 / num246);
    }
    if tool_mode & mode::BLUE != 0 {
        num246 += 1.0;
        color47 = lerp_bytes(color47, [0, 0, 255, 255], 1.0 / num246);
    }
    if tool_mode & mode::GREEN != 0 {
        num246 += 1.0;
        color47 = lerp_bytes(color47, [0, 255, 0, 255], 1.0 / num246);
    }
    if tool_mode & mode::YELLOW != 0 {
        // `:31652` — the Yellow lerp does NOT increment num246.
        color47 = lerp_bytes(color47, [255, 255, 0, 255], 1.0 / (num246 + 1.0));
    }
    if tool_mode & mode::CUTTER != 0 {
        color46 = [50, 50, 50, 255];
    }
    // `:31657` — `color47.A = 0` ALWAYS (the channel lerps raise the alpha
    // toward the targets' 255, this resets it). `color46` keeps its alpha:
    // 0 normally, 255 with the cutter. The C# batch is `BlendState.AlphaBlend`
    // (`Main.cs:38213`/`:62294`, premultiplied `One, OneMinusSrcAlpha`): an
    // alpha-0 tint with NON-zero RGB is an ADDITIVE term (`src + dst`), the
    // visible ghost glow. The port's pipeline is the same equation
    // (`sprite.wgsl`'s fragment keeps rgb un-premultiplied, alpha =
    // `tex.a * color.a`), so the C# alpha values reproduce the additive
    // ghost verbatim — the earlier alpha-255 override turned it into a
    // semi-transparent REPLACE (the seal fix, research/283).
    color47[3] = 0;
    (color47, color46)
}

/// `Color.Lerp` per byte, truncated (the `render/pylons.rs` precedent).
fn lerp_bytes(a: [u8; 4], b: [u8; 4], t: f32) -> [u8; 4] {
    [
        (a[0] as f32 + (b[0] as f32 - a[0] as f32) * t) as u8,
        (a[1] as f32 + (b[1] as f32 - a[1] as f32) * t) as u8,
        (a[2] as f32 + (b[2] as f32 - a[2] as f32) * t) as u8,
        (a[3] as f32 + (b[3] as f32 - a[3] as f32) * t) as u8,
    ]
}

/// One path cell's mask X (`Main.cs:31662-31899`): the source-rect column
/// the C# picks for the cell's role. `role` is (start, middle, end) ×
/// (vertical, horizontal, corner); the corner forms ADD the two sides.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mask {
    /// `(0,0,16,16)` — the same-cell case (`:31662`).
    Same,
    /// `(18,y)` / `(72,y)` — the vertical start/end caps (`:31677`/`:31699`).
    CapUp,
    CapDown,
    /// `(36,y)` / `(144,y)` — the horizontal caps (`:31714`/`:31736`).
    CapRight,
    CapLeft,
    /// `(90,y)` — the vertical middle (`:31689`).
    MidV,
    /// `(180,y)` — the horizontal middle (`:31726`).
    MidH,
    /// The L-corner's combined column (`:31838-31847`): the C# ADDS the
    /// two side columns onto `value56.X` — the sum addresses a combined
    /// mask on the 651 sheet.
    Corner(f32),
}

impl Mask {
    /// The source column (the row is 0 for the colour quad, 18 for the
    /// grey — the C# writes `value.Y = 18` after the first draw).
    pub fn x(self) -> f32 {
        match self {
            Mask::Same => 0.0,
            Mask::CapUp => 18.0,
            Mask::CapDown => 72.0,
            Mask::CapRight => 36.0,
            Mask::CapLeft => 144.0,
            Mask::MidV => 90.0,
            Mask::MidH => 180.0,
            Mask::Corner(x) => x,
        }
    }
}

/// One ghost quad: the tile cell, the mask, and whether it is the grey
/// row (the colour row draws first, then the grey — `:31667-31669`).
#[derive(Clone, Copy, Debug)]
pub struct GhostQuad {
    pub tx: i32,
    pub ty: i32,
    pub mask: Mask,
}

/// The L-path walk (`Main.cs:31659-31907`): the same-cell case, the
/// straight vertical/horizontal runs, and the two-segment L (the
/// player's facing picks the axis order — facing right walks Y first,
/// facing left walks X first). The middle-cell scans gate on
/// `WorldGen.InWorld(x, y, 1)`; the port's `in_bounds` is the same
/// `[1, max-2]` window through the caller's `in_world` closure.
pub fn ghost_path(
    ps: (i32, i32),
    pe: (i32, i32),
    facing_right: bool,
    in_world: impl Fn(i32, i32) -> bool,
) -> Vec<GhostQuad> {
    let mut out = Vec::new();
    let mut push = |tx, ty, mask| out.push(GhostQuad { tx, ty, mask });
    if ps == pe {
        // `:31659-31670` — the same-cell case.
        push(pe.0, pe.1, Mask::Same);
        return out;
    }
    if ps.0 == pe.0 {
        // `:31672-31707` — the straight vertical run.
        let d = pe.1 - ps.1;
        let step = d.signum();
        push(ps.0, ps.1, if d > 0 { Mask::CapDown } else { Mask::CapUp });
        let mut y = ps.1 + step;
        while y != pe.1 {
            push(ps.0, y, Mask::MidV);
            y += step;
        }
        push(pe.0, pe.1, if d > 0 { Mask::CapUp } else { Mask::CapDown });
        return out;
    }
    if ps.1 == pe.1 {
        // `:31709-31744` — the straight horizontal run.
        let d = pe.0 - ps.0;
        let step = d.signum();
        push(ps.0, ps.1, if d > 0 { Mask::CapRight } else { Mask::CapLeft });
        let mut x = ps.0 + step;
        while x != pe.0 {
            push(x, ps.1, Mask::MidH);
            x += step;
        }
        push(pe.0, pe.1, if d > 0 { Mask::CapLeft } else { Mask::CapRight });
        return out;
    }
    // `:31746-31907` — the L. The first segment walks the facing axis;
    // the corner cell's mask ADDS the two sides (`:31838-31847`).
    let sx = (pe.0 - ps.0).signum();
    let sy = (pe.1 - ps.1).signum();
    if facing_right {
        // `:31756-31762` — Y first from the anchor.
        push(ps.0, ps.1, if sy > 0 { Mask::CapDown } else { Mask::CapUp });
        let mut y = ps.1 + sy;
        while y != pe.1 {
            if in_world(ps.0, y) {
                push(ps.0, y, Mask::MidV);
            }
            y += sy;
        }
        // The corner (`:31836-31847`): X-side + Y-side.
        let corner = corner_mask(sx, sy, true);
        push(ps.0, pe.1, corner);
        let mut x = ps.0 + sx;
        while x != pe.0 {
            if in_world(x, pe.1) {
                push(x, pe.1, Mask::MidH);
            }
            x += sx;
        }
        // The end cap (`:31890-31899`): the X-side only.
        push(pe.0, pe.1, if sx > 0 { Mask::CapLeft } else { Mask::CapRight });
    } else {
        // `:31763-31769` — X first from the anchor.
        push(ps.0, ps.1, if sx > 0 { Mask::CapRight } else { Mask::CapLeft });
        let mut x = ps.0 + sx;
        while x != pe.0 {
            if in_world(x, ps.1) {
                push(x, ps.1, Mask::MidH);
            }
            x += sx;
        }
        let corner = corner_mask(sx, sy, false);
        push(pe.0, ps.1, corner);
        let mut y = ps.1 + sy;
        while y != pe.1 {
            if in_world(pe.0, y) {
                push(pe.0, y, Mask::MidV);
            }
            y += sy;
        }
        // The end cap (`:31893-31895`): the Y-side only.
        push(pe.0, pe.1, if sy > 0 { Mask::CapUp } else { Mask::CapDown });
    }
    out
}

/// The corner cell's combined mask (`Main.cs:31838-31847`): the C# ADDS
/// the two source columns (`value56.X += ...`). The sum lands on a
/// combined-mask column of the 651 sheet — [`Mask::Corner`] carries it.
fn corner_mask(sx: i32, sy: i32, y_first: bool) -> Mask {
    // facing right (`:31843-31847`): X += (sx>0 ? 36 : 144) + (sy>0 ? 18 : 72)
    // facing left  (`:31838-31842`): X += (sx>0 ? 144 : 36) + (sy>0 ? 72 : 18)
    let a = if y_first {
        if sx > 0 { 36.0 } else { 144.0 }
    } else {
        if sx > 0 { 144.0 } else { 36.0 }
    };
    let b = if y_first {
        if sy > 0 { 18.0 } else { 72.0 }
    } else {
        if sy > 0 { 72.0 } else { 18.0 }
    };
    Mask::Corner(a + b)
}

/// The draw entry: the ghost quads for one live kite, pushed into the ui
/// range's screen pixels (the over-wire slot — the caller draws it right
/// after `draw_wires`, before the generic projectile pass).
pub fn draw_wire_kite(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    p: &Projectile,
    tool_mode: u8,
    facing_right: bool,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    world_w: i32,
    world_h: i32,
) {
    let Some(sheet) = lookups.proj_sheets.get(&crate::projectile::WIRE_KITE).copied() else {
        return;
    };
    let ps = (p.ai[0] as i32, p.ai[1] as i32);
    let cx = p.pos.x + p.w * 0.5;
    let cy = p.pos.y + p.h * 0.5;
    // `Main.cs:31627` — `proj.Center.ToTileCoordinates()` = `(int)X >> 4`
    // (`Utils.cs:1807-1810`): cast FIRST, then arithmetic-shift (floors
    // negatives; the truncating-divide form differed there).
    let pe = ((cx as i32) >> 4, (cy as i32) >> 4);
    let (c47, c46) = ghost_colors(tool_mode);
    let actuator = tool_mode & mode::ACTUATOR != 0;
    let t47 = tint(c47);
    let t46 = tint(c46);
    let aw = lookups.atlas_size.0 as f32;
    let ah = lookups.atlas_size.1 as f32;
    let uv_col = |x: f32, y: f32| {
        let x0 = sheet.x as f32 + x;
        let y0 = sheet.y as f32 + y;
        [x0 / aw, y0 / ah, (x0 + 16.0) / aw, (y0 + 16.0) / ah]
    };
    // `WireUi[11]`'s whole-sheet quad (`:31665` — null source rect).
    let stamp = lookups.wire_ui[11].and_then(|s| lookups.hud_uv(Some(&s)));
    let in_world = |x: i32, y: i32| x >= 1 && y >= 1 && x < world_w - 1 && y < world_h - 1;
    for q in ghost_path(ps, pe, facing_right, in_world) {
        let pos = [
            (q.tx as f32 * 16.0 - camera.x) * zoom + screen.x * 0.5,
            (q.ty as f32 * 16.0 - camera.y) * zoom + screen.y * 0.5,
        ];
        let size = [16.0 * zoom, 16.0 * zoom];
        // The actuator stamp UNDER the pair (`:31663-31666` and every
        // twin site).
        if actuator {
            if let Some(uv) = stamp {
                r.push_ui_sprite(pos, size, uv, t46);
            }
        }
        // The colour row then the grey row (`:31667-31669`).
        r.push_ui_sprite(pos, size, uv_col(q.mask.x(), 0.0), t47);
        r.push_ui_sprite(pos, size, uv_col(q.mask.x(), 18.0), t46);
    }
}

fn tint(c: [u8; 4]) -> [f32; 4] {
    [
        c[0] as f32 / 255.0,
        c[1] as f32 / 255.0,
        c[2] as f32 / 255.0,
        c[3] as f32 / 255.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_single_channel_mix_is_the_half_lerp() {
        // `:31635-31639`: Red only — num246 = 2, Lerp((127..), Red, 1/2):
        // 127 + (255-127)*0.5 = 191, 127 + (0-127)*0.5 = 63.5 → 63.
        // `:31657` resets color47.A to 0 (the additive ghost); color46
        // keeps alpha 0 without the cutter.
        let (c47, c46) = ghost_colors(mode::RED);
        assert_eq!(c47, [191, 63, 63, 0]);
        assert_eq!(c46, [255, 255, 255, 0], "no cutter keeps the white grey, alpha 0");
    }

    #[test]
    fn the_multi_channel_mix_walks_the_running_count() {
        // Red+Blue+Green: num246 2→3→4. After Red (191,63,63); Blue at
        // 1/3: (127.33→127, 42, 127.66→127); Green at 1/4: (95.25→95,
        // 42+(255-42)*0.25 = 95.25→95, 127+(0-127)*0.25 = 95.25→95).
        let (c47, _) = ghost_colors(mode::RED | mode::BLUE | mode::GREEN);
        assert_eq!(c47, [95, 95, 95, 0]);
    }

    #[test]
    fn the_yellow_lerp_does_not_increment_the_count() {
        // `:31650-31653`: Yellow alone — num246 stays 1, the lerp is
        // 1/(1+1) = 0.5: (191, 191, 63).
        let (c47, _) = ghost_colors(mode::YELLOW);
        assert_eq!(c47, [191, 191, 63, 0]);
        // Red+Yellow: num246 = 2 after Red, Yellow at 1/3:
        // 191 + (255-191)/3 = 212.33 → 212; 63 + (255-63)/3 = 127;
        // 63 + (0-63)/3 = 42.
        let (c47, _) = ghost_colors(mode::RED | mode::YELLOW);
        assert_eq!(c47, [212, 127, 42, 0]);
    }

    #[test]
    fn the_cutter_swaps_the_grey() {
        // `:31654-31657`: the cutter's grey is (50,50,50,255) — alpha 255
        // (a REPLACE quad, unlike the additive alpha-0 normal grey).
        let (_, c46) = ghost_colors(mode::CUTTER);
        assert_eq!(c46, [50, 50, 50, 255]);
    }

    #[test]
    fn the_same_cell_path_is_one_quad() {
        // `:31659-31670`.
        let path = ghost_path((10, 20), (10, 20), true, |_, _| true);
        assert_eq!(path.len(), 1);
        assert_eq!(path[0].mask, Mask::Same);
    }

    #[test]
    fn the_vertical_run_caps_and_middles() {
        // `:31672-31707`: down-run (10,20)→(10,23): start CapDown (72),
        // two MidV (90), end CapUp (18).
        let path = ghost_path((10, 20), (10, 23), true, |_, _| true);
        assert_eq!(path.len(), 4);
        assert_eq!(path[0].mask, Mask::CapDown);
        assert_eq!(path[1].mask, Mask::MidV);
        assert_eq!(path[1].ty, 21);
        assert_eq!(path[3].mask, Mask::CapUp);
        assert_eq!(path[3].ty, 23);
        // The up-run flips the caps.
        let path = ghost_path((10, 23), (10, 20), true, |_, _| true);
        assert_eq!(path[0].mask, Mask::CapUp);
        assert_eq!(path[3].mask, Mask::CapDown);
    }

    #[test]
    fn the_horizontal_run_caps_and_middles() {
        // `:31709-31744`: right-run (10,20)→(13,20).
        let path = ghost_path((10, 20), (13, 20), true, |_, _| true);
        assert_eq!(path.len(), 4);
        assert_eq!(path[0].mask, Mask::CapRight);
        assert_eq!(path[1].mask, Mask::MidH);
        assert_eq!(path[3].mask, Mask::CapLeft);
    }

    #[test]
    fn the_l_path_facing_right_walks_y_first() {
        // `:31756-31762` + `:31843-31847`: (10,20)→(13,23), facing right:
        // start CapDown, MidV at 21/22, corner at (10,23) with
        // 36 + 18 = 54, MidH at 11/12, end CapLeft.
        let path = ghost_path((10, 20), (13, 23), true, |_, _| true);
        assert_eq!(path.len(), 7);
        assert_eq!(path[0].mask, Mask::CapDown);
        assert_eq!(path[3].tx, 10);
        assert_eq!(path[3].ty, 23);
        assert_eq!(path[3].mask.x(), 54.0);
        assert_eq!(path[6].mask, Mask::CapLeft);
    }

    #[test]
    fn the_l_path_facing_left_walks_x_first() {
        // `:31763-31769` + `:31838-31842`: (10,20)→(13,23), facing left:
        // start CapRight, MidH at 11/12, corner at (13,20) with
        // 144 + 72 = 216, MidV at 21/22, end CapUp.
        let path = ghost_path((10, 20), (13, 23), false, |_, _| true);
        assert_eq!(path.len(), 7);
        assert_eq!(path[0].mask, Mask::CapRight);
        assert_eq!(path[3].tx, 13);
        assert_eq!(path[3].ty, 20);
        assert_eq!(path[3].mask.x(), 216.0);
        assert_eq!(path[6].mask, Mask::CapUp);
    }

    #[test]
    fn the_middle_scan_skips_out_of_world_cells() {
        // `:31801`/`:31869`: the InWorld(x,y,1) gate — the corner and
        // the caps draw regardless, the middles skip.
        let path = ghost_path((10, 20), (13, 23), true, |x, y| x > 10 || y > 22);
        // start + corner + end always; MidV 21/22 skipped (x=10, y<=22),
        // MidH 11/12 pass (x>10).
        assert_eq!(path.len(), 5);
    }
}
