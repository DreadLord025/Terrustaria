//! The NPC light-cast sites, THIRD batch (W4-f8, S1-E-5 wave 4).
//!
//! `NPC.cs` carries exactly 80 DIRECT `Lighting.AddLight` call sites. The
//! first 40 (`:21307`–`:48641`) ride `task/W4-c20-addlight-npcs-tiles`
//! (pushed claim `9555e586`, `npc_lights_a.rs`); the second 40
//! (`:49929`–`:92412`) ride W4-e7's in-flight `npc_lights_b.rs` (its header
//! table claims all 40). This module ports the UNCLAIMED REMAINDER: the
//! two INDIRECT light writes — `Utils.PlotTileLine(base.Top, base.Bottom,
//! base.width, DelegateMethods.CastLightOpen)`:
//!
//! | # | site | arm | port |
//! |---|------|-----|------|
//! | R1 | `:63542-63543` | `AI_107_ImprovedWalkers` case 566/567 (DD2SkeletonT1/T3) | [`dd2_body_cast_light`] |
//! | R2 | `:65828-65829` | `AI_109_DarkMage`'s `type == 564 || type == 565` block | [`dd2_body_cast_light`] |
//!
//! Both arms set `DelegateMethods.v3_1 = new Vector3(0.3f, 0.05f, 0.45f)
//! * 1.5f` and then cast the body line — the same colour and the same
//! walk, one shared fn. The cast feeds the SAME
//! [`crate::entity_lights::EntityLights`] pair as every other family (one
//! list pair per engine, `LightingEngine.cs:36-38`, max-merged one frame
//! late `:208-235`); the tile-anchor vocabulary is shared from there.
//!
//! ## Negative census (the remainder is exactly these two)
//!
//! * Case-insensitive `addlight` over the full NPC.cs: the same 80 direct
//!   sites, no other spelling.
//! * `Lighting.` forms outside `AddLight`: only `:92348`'s
//!   `Lighting.GetColor` — a READ (the gold-critter sparkle gate), already
//!   cited in e7's module docs.
//! * `PlotTileLine` forms: `:55401`/`:55680` pass
//!   `DelegateMethods.SearchAvoidedByNPCs` (a collision probe, NO light;
//!   ported in `npc_ai_town.rs::body_line_avoids`), plus the two
//!   `CastLightOpen` casts ported here.
//! * Zero `TileLightScanner`/`LightingEngine`/`lightValue`/`SpreadLight`/
//!   other `CastLight*` forms in NPC.cs. The `CastLightOpen` DELEGATE
//!   itself (`DelegateMethods.cs:726-741`) is W4-e8's `tile_lights.rs`
//!   scope — that module's header explicitly leaves the NPC update-side
//!   call sites to the NPC modules; this module carries the NPC-side
//!   call-site copy (the `npc_ai_town.rs::plot_line` precedent for the
//!   shared `Utils.PlotLine` walk) so the feed does not depend on e8's
//!   uncommitted file. The merger should keep ONE delegate copy.
//!
//! ## RNG census — why the feed is draw-side
//!
//! Both casts are RNG-free: the `v3_1` write and the `PlotTileLine` walk
//! draw nothing. In `AI_107` the cast runs BEFORE the case's
//! `localAI[3]`-gated dust/gore halves (`:63573-63610`, `Main.rand`), and
//! in `AI_109` before the cast-cycle sounds/projectiles — the feed's
//! placement cannot move any stream (the `npc_lights.rs` precedent).
//! The `Main.rand.Next(600)` at `:55401` belongs to the SearchAvoided
//! probe, not a light.
//!
//! ## netOffset conventions (verbatim per site)
//!
//! Both casts read the RAW position: `AI_107`'s only netOffset wrap is
//! the Drakin 620 arm's (`:63655`/`:63690`, AFTER the 566/567 case), and
//! `AI_109`'s wraps sit in the tail blocks (`:66224`/`:66256`, after the
//! per-kind block). Neither 566/567 nor 564/565 wraps its cast.
//!
//! ## The walk (verbatim `Utils.PlotTileLine`, `Utils.cs:2337-2350`)
//!
//! `Top` = `(position.X + width/2, position.Y)`, `Bottom` = `(position.X
//! + width/2, position.Y + height)` (`Entity.cs:86-91`/`:120-125`), so
//! the unit direction is exactly (0, 1) and the perpendicular
//! `(-dir.Y, dir.X) * width/2` is pure `-width/2` in X. NOTE the C#'s
//! offset signs: `point = start - perp` is the RIGHT edge (`cx + w/2`)
//! and `point2 = start + perp` the LEFT edge — so `lineMinOffset` is the
//! right-column offset and the inner span walks RIGHT→LEFT
//! (`Utils.cs:2343-2349`). The four `ToTileCoordinates` points
//! (`Utils.cs:1807-1810`: `(int)v >> 4` — truncate the coordinate, then
//! ARITHMETIC-shift) give the offsets relative to the start tile; the
//! outer `PlotLine` walks the centre column (`jump: true` default), each
//! step plotting the horizontal span with `jump: false`
//! (`Utils.cs:2238-2292`). `PlotLine`'s `i != x1` loop NEVER plots the
//! end point (only the `x0 == x1 && y0 == y1` head plots a lone tile),
//! so the span's LEFTMOST column is dropped (the 18 px skeleton's
//! 2-column span lights only its right column; the 34 px mage's
//! 3-column span lights its right two) and the body's last tile row is
//! unploted — vanilla quirks, kept verbatim. The
//! `npc_ai_town.rs::body_line_avoids` twin swaps the two edges (harmless
//! for its boolean probe's common case); the light list's ORDER is part
//! of the C# per-frame list, so this port keeps the C# right-first order.
//! `CastLightOpen` never aborts an in-bounds walk (it returns true even
//! on a solid tile — the open-test gates only the light,
//! `DelegateMethods.cs:726-741`), so the abort paths are unreachable
//! here but kept verbatim.

use glam::Vec2;
use tr_core::world::World;

use crate::npc::{Npc, NpcKind};

/// One `AddLight` source: tile x, tile y, RGB (the `EntityLights::add`
/// triple).
pub(crate) type Light = (i32, i32, [f32; 3]);

/// `DelegateMethods.v3_1` for both DD2 casts (`NPC.cs:63542`/`:65828`):
/// `new Vector3(0.3f, 0.05f, 0.45f) * 1.5f`. The XNA product is one
/// float multiply per component — kept as the expression, not folded
/// literals, so the f32 rounding order is the C#'s.
const DD2_CAST_RGB: [f32; 3] = [0.3 * 1.5, 0.05 * 1.5, 0.45 * 1.5];

/// `Utils.PlotLine` (`Utils.cs:2238-2292`) — the Bresenham walk with the
/// `jump: false` double-plot on the minor-axis step. `plot` returns false
/// to abort (the C# `TileActionAttempt` convention). The
/// `npc_ai_town.rs::plot_line` twin — kept here so the light feed does
/// not reach into the town AI module's private fn.
fn plot_line(
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    jump: bool,
    plot: &mut dyn FnMut(i32, i32) -> bool,
) -> bool {
    if x0 == x1 && y0 == y1 {
        return plot(x0, y0);
    }
    let mut x0 = x0;
    let mut y0 = y0;
    let mut x1 = x1;
    let mut y1 = y1;
    let flag = (y1 - y0).abs() > (x1 - x0).abs();
    if flag {
        std::mem::swap(&mut x0, &mut y0);
        std::mem::swap(&mut x1, &mut y1);
    }
    let num = (x1 - x0).abs();
    let num2 = (y1 - y0).abs();
    let mut num3 = num / 2;
    let mut num4 = y0;
    let num5 = if x0 < x1 { 1 } else { -1 };
    let num6 = if y0 < y1 { 1 } else { -1 };
    let mut i = x0;
    while i != x1 {
        if flag {
            if !plot(num4, i) {
                return false;
            }
        } else if !plot(i, num4) {
            return false;
        }
        num3 -= num2;
        if num3 >= 0 {
            i += num5;
            continue;
        }
        num4 += num6;
        if !jump {
            if flag {
                if !plot(num4, i) {
                    return false;
                }
            } else if !plot(i, num4) {
                return false;
            }
        }
        num3 += num;
        i += num5;
    }
    true
}

/// `DelegateMethods.CastLightOpen` — the ONE delegate copy is W4-e8's
/// `tile_lights.rs::cast_light_open` (`DelegateMethods.cs:726-741`, the
/// merger keep-one per research/366 §1 f8 row); this adapter only folds
/// the DD2 colour into the `CastLightEnv` (the C# sets `v3_1` before
/// `Utils.PlotTileLine`; `v2_1`/`f_1` are unread by CastLightOpen).
fn cast_light_open(world: &World, x: i32, y: i32, rgb: [f32; 3], out: &mut Vec<Light>) -> bool {
    let env = crate::tile_lights::CastLightEnv {
        rgb,
        origin: Vec2::ZERO,
        range: 0.0,
    };
    crate::tile_lights::cast_light_open(world, x, y, &env, out)
}

/// `Utils.PlotTileLine(Top, Bottom, width, CastLightOpen)`
/// (`Utils.cs:2337-2350`) — the vertical body line with the half-width
/// perpendicular offset. For the NPC's vertical Top→Bottom line the unit
/// direction is (0,1), so the perpendicular is (−1,0)·w/2: the offsets
/// are pure ±w/2 in X (the `npc_ai_town.rs::body_line_avoids` twin, with
/// the light delegate instead of the avoid probe). The four points use
/// `ToTileCoordinates`' truncate-then-ARITHMETIC-shift (`Utils.cs:1807-
/// 1810`) — NOT the quotient form, they differ below zero.
fn body_line_cast(world: &World, x: f32, y: f32, w: f32, h: f32, rgb: [f32; 3], out: &mut Vec<Light>) {
    let half = w as f64 / 2.0;
    let start = (x + w / 2.0, y);
    let end = (x + w / 2.0, y + h);
    // `:2342-2344` — perp = (-dir.Y, dir.X) * w/2 = (-w/2, 0) for the
    // vertical line, so `start - perp` is the RIGHT edge (lineMinOffset)
    // and `start + perp` the LEFT edge (lineMaxOffset): the inner span
    // walks RIGHT→LEFT, verbatim.
    let p_min = (((start.0 as f64 + half) as i32) >> 4, (start.1 as i32) >> 4);
    let p_max = (((start.0 as f64 - half) as i32) >> 4, (start.1 as i32) >> 4);
    let p_start = ((start.0 as i32) >> 4, (start.1 as i32) >> 4);
    let p_end = ((end.0 as i32) >> 4, (end.1 as i32) >> 4);
    let min_off = (p_min.0 - p_start.0, p_min.1 - p_start.1);
    let max_off = (p_max.0 - p_start.0, p_max.1 - p_start.1);
    plot_line(p_start.0, p_start.1, p_end.0, p_end.1, true, &mut |px, py| {
        plot_line(
            px + min_off.0,
            py + min_off.1,
            px + max_off.0,
            py + max_off.1,
            false,
            &mut |tx, ty| cast_light_open(world, tx, ty, rgb, out),
        )
    });
}

/// Sites R1/R2 — the DD2 body casts (`NPC.cs:63542-63543` AI_107's
/// 566/567 case head, `:65828-65829` AI_109's 564/565 per-kind block).
/// Unconditional for the four kinds: the skeleton casts every tick of
/// the case (the `localAI[3] >= 120` materialize gate at `:63545` gates
/// the DAMAGE/velocity tail, not the cast), the mage casts every tick of
/// the per-kind block (before the cast-cycle state machine). RAW
/// position (module docs). No arm draws RNG.
pub(crate) fn dd2_body_cast_light(n: &Npc, world: &World) -> Vec<Light> {
    match n.kind {
        NpcKind::DD2SkeletonT1 | NpcKind::DD2SkeletonT3 | NpcKind::DD2DarkMage | NpcKind::DD2DarkMageT3 => {}
        _ => return Vec::new(),
    }
    let (w, h) = n.size();
    let mut out = Vec::new();
    body_line_cast(world, n.x, n.y, w, h, DD2_CAST_RGB, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::FLAG_ACTIVE;

    fn npc(kind: NpcKind, x: f32, y: f32) -> Npc {
        let mut n = Npc::new(kind, x, y, 0, 0.0);
        n.hp = 100;
        n
    }

    /// The C# colour product, component by component (`NPC.cs:63542`).
    #[test]
    fn the_cast_colour_is_the_v3_1_product() {
        let w = World::new("t".into(), 64, 800);
        let n = npc(NpcKind::DD2SkeletonT1, 160.0, 176.0);
        let lights = dd2_body_cast_light(&n, &w);
        assert!(!lights.is_empty());
        for (_, _, rgb) in lights {
            assert_eq!(rgb, [0.3f32 * 1.5, 0.05 * 1.5, 0.45 * 1.5]);
        }
    }

    /// The 18×40 skeleton (`NPC.cs:16348-16349`) at x=160: centre 169,
    /// span [160, 178] → columns 10..11; the inner span walks right→left
    /// and `PlotLine` never plots its end point, so only column 11
    /// lights. Rows y=176..215 → tiles 11, 12, 13; the outer walk plots
    /// 11 and 12 (the end row 13 is unploted, `Utils.cs:2256`).
    #[test]
    fn the_skeleton_cast_lights_the_right_column_of_each_body_row() {
        let w = World::new("t".into(), 64, 800);
        let n = npc(NpcKind::DD2SkeletonT1, 160.0, 176.0);
        let lights = dd2_body_cast_light(&n, &w);
        assert_eq!(lights, vec![(11, 11, DD2_CAST_RGB), (11, 12, DD2_CAST_RGB)]);
        // The T3 shares the arm (`case 566: case 567:`).
        let t3 = npc(NpcKind::DD2SkeletonT3, 160.0, 176.0);
        assert_eq!(dd2_body_cast_light(&t3, &w), lights);
    }

    /// The 34×62 mage (`NPC.cs:16450-16451`) at x=160: centre 177, span
    /// [160, 194] → columns 10..12; the right→left inner walk plots 12
    /// then 11 and drops the end column 10. Rows y=176..237 → tiles
    /// 11..14, the end row 14 unploted. The list ORDER is right-first,
    /// the C# per-frame list's order.
    #[test]
    fn the_mage_cast_lights_two_tiles_per_body_row_right_first() {
        let w = World::new("t".into(), 64, 800);
        let n = npc(NpcKind::DD2DarkMage, 160.0, 176.0);
        let lights = dd2_body_cast_light(&n, &w);
        assert_eq!(
            lights,
            vec![
                (12, 11, DD2_CAST_RGB),
                (11, 11, DD2_CAST_RGB),
                (12, 12, DD2_CAST_RGB),
                (11, 12, DD2_CAST_RGB),
                (12, 13, DD2_CAST_RGB),
                (11, 13, DD2_CAST_RGB),
            ]
        );
        let t3 = npc(NpcKind::DD2DarkMageT3, 160.0, 176.0);
        assert_eq!(dd2_body_cast_light(&t3, &w), lights);
    }

    /// `CastLightOpen`'s open-test (`DelegateMethods.cs:736`): an active
    /// solid stone block (TileID.Stone 1) stays DARK, but the walk does
    /// not abort — the rows below it still light. An actuated
    /// (`inActive()`) stone lights again.
    #[test]
    fn solid_tiles_stay_dark_but_the_walk_continues() {
        let mut w = World::new("t".into(), 64, 800);
        let i = w.index(11, 11);
        w.tiles[i].kind = 1; // TileID.Stone — tileSolid, not solidTop
        w.tiles[i].flags |= FLAG_ACTIVE;
        let n = npc(NpcKind::DD2SkeletonT1, 160.0, 176.0);
        let lights = dd2_body_cast_light(&n, &w);
        assert_eq!(lights, vec![(11, 12, DD2_CAST_RGB)], "row 11 solid, row 12 open");
        w.tiles[i].flags |= tr_core::tile::FLAG_INACTIVE;
        let lights = dd2_body_cast_light(&n, &w);
        assert_eq!(lights, vec![(11, 11, DD2_CAST_RGB), (11, 12, DD2_CAST_RGB)]);
    }

    /// The anchor is `ToTileCoordinates`' truncate-then-shift, not the
    /// quotient form. At (-20, -20) the skeleton's centre is -11, span
    /// [-20, -2] → columns -2..-1 (shift: -2>>4 = -1, -11>>4 = -1; the
    /// quotient form would give 0 for both); the first plotted tile
    /// (-1, -2) is OUT of world, so CastLightOpen's bounds gate aborts
    /// the whole walk — no lights (`DelegateMethods.cs:728-731`).
    #[test]
    fn an_out_of_world_cast_aborts_the_walk() {
        let w = World::new("t".into(), 64, 800);
        let n = npc(NpcKind::DD2SkeletonT1, -20.0, -20.0);
        assert!(dd2_body_cast_light(&n, &w).is_empty());
    }

    /// At (-10, 0) the centre is -1 (`-1 >> 4` = -1, NOT the quotient
    /// form's 0), the right edge 8 → column 0: min_off = +1, so the
    /// inner span plots column 0 (in world) and drops the end column
    /// -1. Rows y=0..40 → tiles 0, 1, 2; the outer walk plots 0 and 1.
    #[test]
    fn negative_positions_use_the_shift_anchor() {
        let w = World::new("t".into(), 64, 800);
        let n = npc(NpcKind::DD2SkeletonT1, -10.0, 0.0);
        let lights = dd2_body_cast_light(&n, &w);
        assert_eq!(lights, vec![(0, 0, DD2_CAST_RGB), (0, 1, DD2_CAST_RGB)]);
    }

    /// Every other kind is dark here — the two casts are the whole
    /// third-batch remainder (the 80 direct sites belong to c20/e7).
    #[test]
    fn other_kinds_cast_nothing() {
        let w = World::new("t".into(), 64, 800);
        for kind in [NpcKind::Slime, NpcKind::Zombie, NpcKind::Guide, NpcKind::DD2KoboldFlyerT3] {
            assert!(dd2_body_cast_light(&npc(kind, 160.0, 176.0), &w).is_empty());
        }
    }

    /// The third batch's claim census: NPC.cs carries EXACTLY 80 direct
    /// `Lighting.AddLight` sites (c20 owns 1-40, e7 owns 41-80) and
    /// EXACTLY two light-bearing `PlotTileLine` casts — the remainder
    /// this module ports. Reads the gitignored `decompiled/` junction
    /// at test time and FAILS when absent (the S1-L-12d /
    /// `main_addlight_census.rs` precedent). Written not run (code-only).
    #[test]
    fn the_npc_cs_light_census_is_80_direct_plus_two_casts() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../decompiled/Terraria/NPC.cs");
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "{} could not be read ({e}) — create the junction \
                 (`cmd /c mklink /J decompiled ..\\Terraria-Rust\\decompiled`) \
                 before running the suite (the S1-L-12d precedent)",
                path.display()
            )
        });
        let lines: Vec<&str> = text.lines().collect();
        let direct: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, l)| l.contains("Lighting.AddLight("))
            .map(|(i, _)| i + 1)
            .collect();
        assert_eq!(direct.len(), 80, "the c20/e7 split assumes exactly 80");
        assert_eq!(direct[0], 21307, "c20's first site");
        assert_eq!(direct[39], 48641, "c20's last site");
        assert_eq!(direct[40], 49929, "e7's first site");
        assert_eq!(direct[79], 92412, "e7's last site");
        let casts: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, l)| l.contains("PlotTileLine") && l.contains("CastLightOpen"))
            .map(|(i, _)| i + 1)
            .collect();
        assert_eq!(casts, vec![63543, 65829], "the third-batch remainder");
        // The colour write sits one line above each cast, verbatim.
        for &c in &casts {
            assert_eq!(
                lines[c - 2].trim(),
                "DelegateMethods.v3_1 = new Vector3(0.3f, 0.05f, 0.45f) * 1.5f;"
            );
            assert_eq!(
                lines[c - 1].trim(),
                "Utils.PlotTileLine(base.Top, base.Bottom, base.width, DelegateMethods.CastLightOpen);"
            );
        }
        // Negative census: no other light-write form in the file.
        let other_light: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, l)| {
                (l.contains("TileLightScanner")
                    || l.contains("LightingEngine")
                    || l.contains("SpreadLight")
                    || (l.contains("Lighting.") && !l.contains("Lighting.AddLight(")))
            })
            .map(|(i, _)| i + 1)
            .collect();
        assert_eq!(other_light, vec![92348], "the GetColor READ, not a write");
        let probes: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, l)| l.contains("PlotTileLine") && !l.contains("CastLightOpen"))
            .map(|(i, _)| i + 1)
            .collect();
        assert_eq!(probes, vec![55401, 55680], "the SearchAvoidedByNPCs probes, no light");
    }
}
