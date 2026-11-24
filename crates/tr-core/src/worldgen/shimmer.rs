//! W4-g36 A1: the Shimmer pass (`GenPassNameID.Shimmer`,
//! `WorldGen.cs:15195-15245`) — the aether biome's placement: the depth/x
//! anchor rolls opposite the dungeon, the `ShimmerMakeBiome` retry loop
//! (`:34853-35057` — the stone shell, the shimmer pool, the num==0 island
//! arm with its ceiling stalactites and the 500 gem-tree speckle) and the
//! two `ShimmerMakeBiomeOpening` tunnels (`:35059-35102`). The pass
//! publishes `GenVars.shimmerPosition` (the Altars/Wall Variety/Moss
//! consumers' anchor) and protects the 200×200 rect.
//!
//! Deviations (code-only lane, the d10x convention):
//! - `PlaceTight` rides the spider pass's port (`spider_gem_moss.rs`); the
//!   `CheckStalactite` follow-up (`:38320-38323`) is ported below but its
//!   `KillTile` failure arm is the port's field-clear (the worldgen kill
//!   cascade's loot side is out of scope — no items exist at gen time).
//! - The gem trees ride a local `GrowTreeWithSettings` port on the pass's
//!   xorshift stream (the `biome_detail::grow_underground_tree` /
//!   `ocean::grow_palm_tree` precedent — `growth/trees.rs`'s port draws on
//!   `UnifiedRandom`, a different stream contract). Two fidelity repairs
//!   the shared growth port parks land here because the shimmer arm needs
//!   them: the `settings.WallTest` delegate (`GemTreeWallTest`,
//!   `:30269-30302`) and the `nactive()` side-ground gate (`:30688-30694`).
//! - The `RangeFrame` closing arm (`:30841`) is the runtime framer's job
//!   (the gem trunks 583-589 are frame-important from boot).
//! - The tenth-anniversary depth arm (`:15217-15228`) is modelled; the
//!   `tenthAnniversaryWorldGen` colour roll inside the tree grow
//!   (`:30414-30417`) draws nothing at gen time (`isGeneratingOrLoadingWorld`).

use super::biome_detail::empty_tile_check;
use super::spider_gem_moss::place_tight;
use super::Rng;
use crate::tile::{Tile, FLAG_ACTIVE, FLAG_HALF_BRICK, FLAG_LIQUID_SHIMMER, LiquidKind};
use crate::tiledata::{TILE_SOLID, TILE_SOLID_TOP};
use crate::world::World;

/// `TileID.Stone` (`TileID.cs:561`).
const TILE_STONE: u16 = 1;
/// `TileID.Crimstone` (`TileID.cs:1035`) / `TileID.Ebonstone` (`:679`) —
/// the biome-rect veto pair (`:34883`).
const TILE_CRIMSTONE: u16 = 203;
const TILE_EBONSTONE: u16 = 25;
/// `TileID.Larva` (`TileID.cs:1093`) — `PlaceTight`'s exemption.
const TILE_LARVA: u16 = 231;
/// `WorldGen.shimmerSafetyDistance` (`WorldGen.cs:86915`, static ctor).
pub(crate) const SHIMMER_SAFETY_DISTANCE: f64 = 150.0;

/// `WorldGen.SolidTile` (`WorldGen.cs:70111-70131`) — out-of-bounds reads
/// as solid; no RNG.
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && TILE_SOLID.has(t.kind)
        && !TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

/// `WorldGen.InWorld(x, y, fluff)` (`WorldGen.cs:9688-9695`).
fn in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// `Tile.anyShimmer` (`Tile.cs:428-435`): liquid > 0 AND the shimmer bit.
fn any_shimmer(t: &Tile) -> bool {
    t.liquid > 0 && t.flags & FLAG_LIQUID_SHIMMER != 0
}

/// The pass body (`WorldGen.cs:15195-15245`). `dungeon_side` is the C#'s
/// `GenVars.CurrentDungeonGenVars.dungeonSide` (-1 Left / 0 None / 1 Right,
/// `DungeonSide.cs:13-15`); `< Right` takes the wide-x arm. Returns the
/// placed centre — the `GenVars.shimmerPosition` publication (`:15241`) —
/// or `None` when the sub-vanilla width guard fires (below). The
/// `Skyblock.denyAllGeneration` gate is the caller's (the step adapter).
pub(crate) fn pass_shimmer(
    world: &mut World,
    world_surface: f64,
    rock_layer: f64,
    dungeon_side: i32,
    anniversary: bool,
    rng: &mut Rng,
) -> Option<(i32, i32)> {
    // Sub-vanilla width guard (the `pass_mining_explosives` /
    // `pass_mahogany_trees` precedent): both x-arms need
    // `(int)(maxTilesX * 0.11) > 200` (`:15210`/`:15228`/`:15233`/`:15238`);
    // below that the C# `Next(200, (int)(0.11 * width))` throws. The port
    // skips the pass before any draw, so the 512-wide pin world is
    // untouched (the `SEED42_512` width-guard proof).
    if (world.width as f64 * 0.11) as i32 <= 200 {
        return None;
    }
    // The depth band (`:15199-15208`): the C# `(int)` casts bind tighter
    // than the division — `(int)(worldSurface + rockLayer) / 2 + 50` and
    // `(int)((double)((maxTilesY - 250) * 2) + rockLayer) / 3`.
    let num = (world_surface + rock_layer) as i32 / 2 + 50;
    let mut num2 = (((world.height - 250) * 2) as f64 + rock_layer) as i32 / 3;
    if num2 > world.height - 330 - 100 - 30 {
        num2 = world.height - 330 - 100 - 30;
    }
    if num2 <= num {
        num2 = num + 50;
    }
    // The first anchor rolls (`:15209-15216`).
    let mut y = rng.range(num, num2);
    let mut x = if dungeon_side < 1 {
        rng.range((world.width as f64 * 0.89) as i32, world.width - 200)
    } else {
        rng.range(200, (world.width as f64 * 0.11) as i32)
    };
    let num5 = world_surface as i32 + 150;
    let mut num6 = (rock_layer + world_surface + 200.0) as i32 / 2;
    if num6 <= num5 {
        num6 = num5 + 50;
    }
    // The tenth-anniversary depth re-roll (`:15217-15220`).
    if anniversary {
        y = rng.range(num5, num6);
    }
    // The placement retry loop (`:15221-15240`). The C# has no iteration
    // cap on the classic arm (it spins until a site passes); the port
    // keeps the same loop — the sub-vanilla worlds' bands still contain
    // valid stone sites (the witness proves termination on 512×512).
    let mut attempts = 0i32;
    while !shimmer_make_biome(world, x, y, rng) {
        attempts += 1;
        if anniversary && attempts < 10000 {
            y = rng.range(num5, num6);
            x = if dungeon_side < 1 {
                rng.range((world.width as f64 * 0.89) as i32, world.width - 200)
            } else {
                rng.range(200, (world.width as f64 * 0.11) as i32)
            };
        } else if attempts > 20000 {
            y = rng.range(world_surface as i32 + 100 + 20, num2);
            x = if dungeon_side < 1 {
                rng.range((world.width as f64 * 0.8) as i32, world.width - 200)
            } else {
                rng.range(200, (world.width as f64 * 0.2) as i32)
            };
        } else {
            y = rng.range((world_surface + rock_layer) as i32 / 2 + 20, num2);
            x = if dungeon_side < 1 {
                rng.range((world.width as f64 * 0.89) as i32, world.width - 200)
            } else {
                rng.range(200, (world.width as f64 * 0.11) as i32)
            };
        }
    }
    Some((x, y))
}

/// `WorldGen.ShimmerMakeBiome` (`WorldGen.cs:34853-35057`): one placement
/// attempt — the shape rolls, the rect veto scan, the shell/pool carve and
/// (on the num==0 shape) the island arm, then the two openings and the gem
/// speckle. `false` leaves the world untouched (the veto scan runs before
/// any write).
fn shimmer_make_biome(world: &mut World, x: i32, y: i32, rng: &mut Rng) -> bool {
    // The shape rolls (`:34855-34870`): the num==0 wide/tall variant vs
    // the default, each axis jittered, then the radius and opening size.
    let num = rng.range(0, 2);
    let (mut num2, mut num3) = if num == 0 { (0.55, 2.0) } else { (0.6, 1.3) };
    let mut num4 = 0.3;
    num2 *= 1.05 - rng.next_f64() * 0.1;
    num3 *= 1.05 - rng.next_f64() * 0.1;
    num4 *= 1.0 - rng.next_f64() * 0.1;
    let num5 = rng.range(105, 125);
    let num6 = (num5 as f64 * num4) as i32;
    let num7 = (num5 as f64 * num2) as i32;
    let num8 = rng.range(9, 13);
    let num9 = x - num5;
    let num10 = x + num5;
    let num11 = y - num5;
    let num12 = y + num5;
    // The veto scan (`:34875-34888`): InWorld(50) + no crimstone/ebonstone.
    // The C# reads `.type` WITHOUT an active gate — an inactive tile keeps
    // its kind, so the veto fires on the stored kind either way.
    for i in num11..=num12 {
        for j in num9..=num10 {
            if !in_world(world, j, i, 50) {
                return false;
            }
            let kind = world.tile(j, i).map_or(0, |t| t.kind);
            if kind == TILE_CRIMSTONE || kind == TILE_EBONSTONE {
                return false;
            }
        }
    }
    // The pool-edge anchors (`:34889-34898`).
    let mut num13 = y;
    if rng.range(0, 4) == 0 {
        num13 = y - rng.range(0, 2);
    }
    let mut num14 = y - num8;
    if rng.range(0, 4) == 0 {
        num14 = y - num8 - rng.range(0, 2);
    }
    // The carve loop (`:34899-34959`). The draw order per cell is the
    // C#'s exact sequence: liquid clear, the two 1/4 edge re-rolls, the
    // shell radius (two NextDouble), the active-edge pair (two Next(2)),
    // the 0.9 wall test, the opening radius (two NextDouble), the opening
    // threshold NextDouble, the pool radius (two NextDouble), the pool
    // threshold NextDouble, then the 1/2 rim roll on the last row.
    for k in num11..=num12 {
        for l in num9..=num10 {
            let idx = world.index(l, k);
            world.tiles[idx].liquid = 0;
            if rng.range(0, 4) == 0 {
                num13 = y - rng.range(0, 2);
            }
            if rng.range(0, 4) == 0 {
                num14 = y - num8 + rng.range(0, 2);
            }
            // `:34912` — the shell radius: the 1.4/1.2 y-stretch above /
            // below the centre row, each axis jittered ±2%.
            let y_stretch = if k <= y { 1.4 } else { 1.2 };
            let dx = (l - x).abs() as f64 * (1.0 + rng.next_f64() * 0.02);
            let dy = (k - y).abs() as f64 * y_stretch * (1.0 + rng.next_f64() * 0.02);
            let mut num15 = (dx * dx + dy * dy).sqrt() as i32;
            if num15 < num5 {
                let t = &mut world.tiles[idx];
                t.set_paint(0);
                t.set_slope(0);
                t.flags &= !FLAG_HALF_BRICK;
                t.kind = TILE_STONE;
                world.set_wall_paint(l, k, 0);
                // `:34920` — the active edge: BOTH Next(2) draws happen
                // (the `&&` evaluates its left operand's draw first, and
                // the right operand's draw only when the left passed).
                let left_ok = l > num9 + 5 + rng.range(0, 2);
                if left_ok && l < num10 - 5 - rng.range(0, 2) {
                    world.tiles[idx].flags |= FLAG_ACTIVE;
                }
                if (num15 as f64) < num5 as f64 * 0.9 {
                    world.tiles[idx].wall = 0;
                }
            }
            // `:34929-34937` — the opening carve (the num3 stretch).
            let dx = (l - x).abs() as f64 * (1.0 + rng.next_f64() * 0.02);
            let dy = (k - y).abs() as f64 * num3 * (1.0 + rng.next_f64() * 0.02);
            num15 = (dx * dx + dy * dy).sqrt() as i32;
            if k > num14 && k < num13 {
                world.tiles[idx].flags &= !FLAG_ACTIVE;
            }
            if k < num13 && num15 < (num7 as f64 * (1.0 + rng.next_f64() * 0.02)) as i32 {
                world.tiles[idx].flags &= !FLAG_ACTIVE;
            }
            // `:34938-34957` — the shimmer pool (the ×2 y-stretch, the
            // 0.025 threshold jitter, the 127/255 liquid rows).
            let dx = (l - x).abs() as f64 * (1.0 + rng.next_f64() * 0.02);
            let dy = ((k - y).abs() * 2) as f64 * (1.0 + rng.next_f64() * 0.02);
            num15 = (dx * dx + dy * dy).sqrt() as i32;
            if k < y - 1 || num15 >= (num6 as f64 * (1.0 + rng.next_f64() * 0.025)) as i32 {
                continue;
            }
            // `:34943` — the rim veto: above Y+2 always clears; on the
            // last rows only the exact num6-1 rim at 1/2 survives.
            if k <= y + 2 || num15 != num6 - 1 || rng.range(0, 2) != 0 {
                world.tiles[idx].flags &= !FLAG_ACTIVE;
            }
            if k >= y {
                let t = &mut world.tiles[idx];
                t.liquid = if k == y { 127 } else { u8::MAX };
                t.flags |= FLAG_LIQUID_SHIMMER;
            }
        }
    }
    // The num==0 island arm (`:34961-35017`): the stone islands along the
    // pool's wide axis with the ceiling stalactites.
    if num == 0 {
        let mut m = (x as f64 - num5 as f64 * num4) as i32 - rng.range(-15, 1) - 5;
        let end = (x as f64 + num5 as f64 * num4) as i32 + rng.range(0, 16);
        let mut shrink_count = 0i32;
        while m < end {
            // `:34969-34974` — climb to the ceiling above the pool. The
            // C# walks into null tiles above row 0 (its `.active()` reads
            // false there); the port's bounds guard stands in for the
            // null walk — on a generated world the carve's stone shell
            // always stops the climb first (the witness substrate proves
            // it), so the guard never fires in practice.
            let mut ceil = y - 3;
            while in_world(world, m, ceil, 0) && !world.tile(m, ceil).is_some_and(|t| t.active()) {
                ceil -= 1;
            }
            ceil -= 4;
            let mut width = rng.range(5, 10);
            let mut height_budget = rng.range(15, 21);
            // `:34977-34998` — the island's shrinking rows. The C# `for
            // (n = m - num18; n < m + num18; n++)` re-initialises `n` each
            // row and exits at the first `n >= m + num18` (the boundary
            // moves as `m`/`num18` change mid-row), so `n` is tracked
            // literally rather than recomputed after the loop.
            let mut n = m - width;
            while width > 0 {
                n = m - width;
                while n < m + width {
                    let idx = world.index(n, ceil);
                    world.tiles[idx].flags |= FLAG_ACTIVE;
                    world.tiles[idx].kind = TILE_STONE;
                    n += 1;
                }
                shrink_count += 1;
                if rng.range(0, 3) < shrink_count {
                    shrink_count = 0;
                    width -= 1;
                    m += rng.range(-1, 2);
                }
                if height_budget <= 0 {
                    width -= 1;
                }
                height_budget -= 1;
                ceil += 1;
            }
            // `:34999-35015` — the trailing stone column + the stalactite.
            // `n` here is the for-loop's exit value (`:34999` `n -= Next(1,3)`).
            n -= rng.range(1, 3);
            for dy in [-2i32, -1, 0] {
                let idx = world.index(n, ceil + dy);
                world.tiles[idx].flags |= FLAG_ACTIVE;
                world.tiles[idx].kind = TILE_STONE;
            }
            if rng.range(0, 2) == 0 {
                let idx = world.index(n, ceil + 1);
                world.tiles[idx].flags |= FLAG_ACTIVE;
                world.tiles[idx].kind = TILE_STONE;
                // KEEP-ONE (matrix-88 §3.2 point 4): the canonical place_tight
                // runs CheckStalactite INSIDE its tail (g41's body) — the
                // explicit check_stalactite call dropped (the double-validation
                // trap). The draw schedule is identical (one Next(3) on a
                // mismatch, same order).
                place_tight(world, rng, n, ceil + 2, false);
            } else {
                place_tight(world, rng, n, ceil + 1, false);
            }
            m += rng.range(9, 14);
        }
    }
    // The two side openings (`:35018-35019`).
    shimmer_make_biome_opening(world, -1, x - num5, y, num8, rng);
    shimmer_make_biome_opening(world, 1, x + num5, y, num8, rng);
    // The gem-tree speckle (`:35020-35055`): 500 sites in the ±70 × [-2,3)
    // box, one of the seven gem trunks, both horizontal neighbours solid.
    let num20 = 70;
    for _ in 0..500 {
        let tx = rng.range(x - num20, x + num20);
        let ty = rng.range(y - 2, y + 3);
        let tree = 583 + rng.range(0, 7) as u16;
        if solid_tile(world, tx - 1, ty) && solid_tile(world, tx + 1, ty) {
            grow_gem_tree(world, tree, tx, ty, rng);
        }
    }
    true
}

/// `WorldGen.ShimmerMakeBiomeOpening` (`WorldGen.cs:35059-35102`): the
/// do-while tunnel walk — clears the opening-size column band, walks the
/// trailing wall strip, and drifts Y at 1/2 until three clear columns or
/// the 100-step / 50-margin budget ends it.
fn shimmer_make_biome_opening(
    world: &mut World,
    direction: i32,
    x: i32,
    mut y: i32,
    cave_opening_size: i32,
    rng: &mut Rng,
) {
    let mut num = x;
    let size = cave_opening_size - 1;
    loop {
        num += direction;
        let mut clear = true;
        // `:35068-35083` — the three-column solid scan + the column clear.
        for i in (y - size + 1)..(y - 1) {
            if solid_tile(world, num, i)
                || solid_tile(world, num + direction, i)
                || solid_tile(world, num + direction * 2, i)
            {
                clear = false;
            }
            if let Some(t) = world.tile_mut(num, i) {
                t.flags &= !FLAG_ACTIVE;
            }
        }
        // `:35084-35087` — the trailing column band.
        for j in (y - size)..y {
            if let Some(t) = world.tile_mut(num - direction, j) {
                t.flags &= !FLAG_ACTIVE;
            }
        }
        // `:35088-35095` — the two wall clears behind solid cells.
        if solid_tile(world, num - direction, y - size - 1) {
            if let Some(t) = world.tile_mut(num - direction, y - size - 1) {
                t.wall = 0;
            }
        }
        if solid_tile(world, num - direction, y) {
            if let Some(t) = world.tile_mut(num - direction, y) {
                t.wall = 0;
            }
        }
        // `:35096-35099` — the 1/2 Y drift.
        if rng.range(0, 2) == 0 {
            y += rng.range(-1, 2);
        }
        // `:35101` — the do-while condition.
        if clear || !(num >= 50 && num <= world.width - 50 && (num - x).abs() <= 100) {
            break;
        }
    }
}

// --- the gem-tree grow (the shimmer speckle's settings arm) -------------------

/// `WorldGen.GemTreeWallTest` (`WorldGen.cs:30269-30302`):
/// `DefaultTreeWallTest` (`WallID.Sets.AllowsPlantsToGrow`, the
/// `growth/trees.rs` transcription) plus the dirt/cave-wall family.
fn gem_tree_wall_test(wall: u16) -> bool {
    const ALLOWS_PLANTS_TO_GROW: [u16; 27] = [
        0, 150, 138, 145, 107, 152, 140, 139, 141, 106, 245, 315, 317, 63, 64, 65, 66, 67, 68,
        69, 81, 70, 264, 268, 265, 74, 80,
    ];
    ALLOWS_PLANTS_TO_GROW.contains(&wall)
        || matches!(
            wall,
            2 | 54 | 55 | 56 | 57 | 58 | 59 | 61 | 185 | 196 | 197 | 198 | 199 | 208 | 209
                | 210 | 211 | 212 | 213 | 214 | 215
        )
}

/// `WorldGen.GemTreeGroundTest` (`WorldGen.cs:30304-30315`): the stone or
/// moss conversion tiles.
fn gem_tree_ground_test(kind: u16) -> bool {
    crate::tiledata::SET_CONVERSION_STONE.has(kind) || crate::tiledata::SET_CONVERSION_MOSS.has(kind)
}

/// One gem-tree cell write — `active(true)` + type + `UseBlockColors`
/// (the ground's paint) + the frame pair (`:30434-30436` et al.).
fn write_tree_cell(world: &mut World, x: i32, y: i32, kind: u16, paint: u8, fx: i32, fy: i32) {
    let idx = world.index(x, y);
    let t = &mut world.tiles[idx];
    t.flags |= FLAG_ACTIVE;
    t.kind = kind;
    t.set_paint(paint);
    t.frame_x = fx as i16;
    t.frame_y = fy as i16;
}

/// `WorldGen.GrowTreeWithSettings` (`WorldGen.cs:30388-30847`) on the
/// worldgen stream — the gem profiles only (`:3930-4001`: heights 7..12,
/// padding 4, sapling 590). The `TryGrowingTreeByType` dispatch
/// (`:30343-30387`) maps 583..589 onto these; the tenth-anniversary colour
/// roll draws nothing at gen time and the errorWorld height arm is a
/// secret-seed branch (the caller's flag gates it out — classic only).
fn grow_gem_tree(world: &mut World, tree: u16, checked_x: i32, checked_y: i32, rng: &mut Rng) -> bool {
    // The sapling-stack walk (`:30390-30393`) — the C#'s quirk: the active
    // test reads `checkedY` (fixed), the kind test reads the walking `num`.
    let mut num = checked_y;
    while world.tile(checked_x, checked_y).is_some_and(|t| t.active())
        && world.tile(checked_x, num).is_some_and(|t| t.kind == 590)
    {
        num += 1;
    }
    // The liquid gate (`:30395-30398`).
    for dx in [-1i32, 0, 1] {
        if world.tile(checked_x + dx, num - 1).is_some_and(|t| t.liquid != 0) {
            return false;
        }
    }
    // The ground gate (`:30399-30408`): nactive, no half-brick/slope,
    // GemTreeGroundTest + GemTreeWallTest.
    let Some(ground) = world.tile(checked_x, num) else {
        return false;
    };
    if !ground.active() || ground.inactive() || ground.half_brick() || ground.slope() != 0 {
        return false;
    }
    let paint = ground.paint();
    let wall = world.tile(checked_x, num - 1).map_or(0, |t| t.wall);
    if !gem_tree_ground_test(ground.kind) || !gem_tree_wall_test(wall) {
        return false;
    }
    // The side grounds (`:30409-30412`).
    let side = |dx: i32| {
        world
            .tile(checked_x + dx, num)
            .is_some_and(|t| t.active() && gem_tree_ground_test(t.kind))
    };
    if !side(-1) && !side(1) {
        return false;
    }
    // The height roll + the clear box (`:30418-30427`).
    let height = rng.range(7, 13);
    if !empty_tile_check(world, checked_x - 2, checked_x + 2, num - (height + 4), num - 1, 20) {
        return false;
    }
    // The trunk loop (`:30431-30685`): per row the variant Next(3), the
    // shape Next(10) (top/bottom forced 0, the same-side re-roll), the
    // frame table, then the branch arms with their two draws each.
    let mut left_last = false;
    let mut right_last = false;
    for i in (num - height)..num {
        let mut num4 = rng.range(0, 3);
        let mut num5 = rng.range(0, 10);
        if i == num - 1 || i == num - height {
            num5 = 0;
        }
        while ((num5 == 5 || num5 == 7) && left_last) || ((num5 == 6 || num5 == 7) && right_last) {
            num5 = rng.range(0, 10);
        }
        left_last = num5 == 5 || num5 == 7;
        right_last = num5 == 6 || num5 == 7;
        let dy = num4 * 22;
        let (fx, fy) = match num5 {
            1 => (0, 66 + dy),
            2 => (22, dy),
            3 => (44, 66 + dy),
            4 => (22, 66 + dy),
            5 => (88, dy),
            6 => (66, 66 + dy),
            7 => (110, 66 + dy),
            _ => (0, dy),
        };
        write_tree_cell(world, checked_x, i, tree, paint, fx, fy);
        if left_last {
            num4 = rng.range(0, 3);
            let (fx, fy) = if rng.range(0, 3) < 2 {
                (44, 198 + num4 * 22)
            } else {
                (66, num4 * 22)
            };
            write_tree_cell(world, checked_x - 1, i, tree, paint, fx, fy);
        }
        if right_last {
            num4 = rng.range(0, 3);
            let (fx, fy) = if rng.range(0, 3) < 2 {
                (66, 198 + num4 * 22)
            } else {
                (88, 66 + num4 * 22)
            };
            write_tree_cell(world, checked_x + 1, i, tree, paint, fx, fy);
        }
    }
    // The side-ground caps (`:30686-30747`): `nactive()` — active AND not
    // framed-off (the shared growth port's `!active` reads the decompile's
    // `nactive` name wrong; the shimmer arm needs the faithful gate).
    let side_ground = |dx: i32| {
        world.tile(checked_x + dx, num).is_some_and(|t| {
            t.active() && !t.inactive() && !t.half_brick() && t.slope() == 0
                && crate::growth::is_tile_type_fit_for_tree(t.kind)
        })
    };
    let mut flag4 = side_ground(-1);
    let mut flag5 = side_ground(1);
    if rng.range(0, 3) == 0 {
        flag4 = false;
    }
    if rng.range(0, 3) == 0 {
        flag5 = false;
    }
    if flag5 {
        let num4 = rng.range(0, 3);
        write_tree_cell(world, checked_x + 1, num - 1, tree, paint, 22, 132 + num4 * 22);
    }
    if flag4 {
        let num4 = rng.range(0, 3);
        write_tree_cell(world, checked_x - 1, num - 1, tree, paint, 44, 132 + num4 * 22);
    }
    // The centre cap (`:30748-30802`) — the Next(3) draws even when no
    // side ground passed (the else-if chain then writes nothing).
    let num4 = rng.range(0, 3);
    if flag4 && flag5 {
        write_tree_cell(world, checked_x, num - 1, tree, paint, 88, 132 + num4 * 22);
    } else if flag4 {
        write_tree_cell(world, checked_x, num - 1, tree, paint, 0, 132 + num4 * 22);
    } else if flag5 {
        write_tree_cell(world, checked_x, num - 1, tree, paint, 66, 132 + num4 * 22);
    }
    // The top cap (`:30803-30840`): the Next(13) column pick draws FIRST,
    // then the Next(3) variant inside the chosen arm.
    let wide = rng.range(0, 13) != 0;
    let num4 = rng.range(0, 3);
    let fx = if wide { 22 } else { 0 };
    write_tree_cell(world, checked_x, num - height, tree, paint, fx, 198 + num4 * 22);
    // `RangeFrame` (`:30841`) — the runtime framer's job (583-589 are
    // frame-important from boot, `pipeline::boot_statics`).
    true
}


// --- ShimmerCleanUp (the Final Cleanup tail arm, WorldGen.cs:34716-34821) -----
//
// W4-g36 gap-fill: g37 ported the Final Cleanup pass body (`4ced188f9`,
// `final_cleanup.rs`) but DEFERRED this arm back to g36 — "reads
// GenVars.shimmerPosition, which g36 A1 owns; no slot on this branch". g36 A1
// publishes `GenVars.shimmer_position` (`step_shimmer`), so the arm lands here.
// It stays UNWIRED (the g39/f25 convention): the merger composes it into g37's
// Final Cleanup at `:22472`, after the `Finish*` tails and before the
// `Liquid.LiquidCheck` merge sweep. Every helper below is pass-local and cited;
// the shared `spread::convert_cleanse` (Convert type 0) and `edit::kill_tile`
// are reused, not copied.

/// `WorldGen.maxTileCount` (`WorldGen.cs:86921`, static ctor) — the value at
/// the Final Cleanup slot: the last write before it is `SurfaceDirtWallsToGrassWalls`
/// (`:19593`, 3500); the only later write is `AddSpikeCaves` (`:24087`), which
/// runs inside the Shell Piles pass (`:16305`) under a seed gate and also sets
/// 3500. So the Shimmerator flood budget is 3500 on every world.
const SHIMMERATOR_MAX_TILES: usize = 3500;

/// `WorldGen.SetBoulderSolidity`'s tile set (`WorldGen.cs:11009-11019`) — the
/// nine boulder kinds `Liquid.tilesIgnoreWater` strips alongside the grate 546
/// (`liquid.rs::BOULDER_SOLIDS`'s cited local copy — the pass-local convention).
const BOULDER_SOLIDS: [u16; 9] = [138, 484, 664, 711, 712, 713, 714, 715, 716];

/// `WorldGen.SolidTile` under the `Liquid.tilesIgnoreWater(ignoreSolids: true)`
/// window (`:34767`): the grate 546 and the boulder family lose solidity, so
/// the Shimmerator flood walks through them. The stock `solid_tile` predicate
/// plus the nine-kind + 546 exemption (`liquid.rs::tile_is_solid`'s window arm).
fn solid_tile_ignore_water(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    if t.kind == 546 || BOULDER_SOLIDS.contains(&t.kind) {
        return false;
    }
    solid_tile(world, x, y)
}

/// `Tile.shimmer()` (`Tile.cs:379-382`): the liquid TYPE is shimmer — the two
/// header bits `(bTileHeader & 0x60) == 96`, independent of the amount. The
/// port's `liquid_kind()` reads the same two bits (the `any_shimmer` helper
/// above is the amount-gated `anyShimmer`, a different C# method).
fn tile_shimmer(t: &Tile) -> bool {
    t.liquid_kind() == LiquidKind::Shimmer
}

/// `Tile.honey()` (`Tile.cs:362-365`): the liquid TYPE is honey.
fn tile_honey(t: &Tile) -> bool {
    t.liquid_kind() == LiquidKind::Honey
}

/// `WorldGen.ShimmerRemoveWater` (`WorldGen.cs:34823-34851`): the ±150 ×
/// [-150, +75] box around the centre; inside the 150-radius disc, non-shimmer
/// liquid clears and a 162 (BreakableIce) deactivates. The `generatingWorld`
/// gate (`:34825`) is always true at the Final Cleanup slot.
fn shimmer_remove_water(world: &mut World, cx: i32, cy: i32) {
    let num = 150;
    for i in (cy - num)..=(cy + num / 2) {
        for j in (cx - num)..=(cx + num) {
            if !in_world(world, j, i, 0) {
                continue;
            }
            let dx = (j - cx).abs() as f64;
            let dy = (i - cy).abs() as f64;
            if (dx * dx + dy * dy).sqrt() as i32 >= num {
                continue;
            }
            let Some(t) = world.tile(j, i).copied() else {
                continue;
            };
            if !tile_shimmer(&t) {
                if let Some(cell) = world.tile_mut(j, i) {
                    cell.liquid = 0;
                }
            }
            if t.kind == 162 {
                if let Some(cell) = world.tile_mut(j, i) {
                    cell.flags &= !FLAG_ACTIVE;
                }
            }
        }
    }
}

/// `WorldGen.Shimmerator` + `ShimmeratorNext` + `Shimminate`
/// (`WorldGen.cs:35118-35147`, `:35104-35116`): the bounded flood over the
/// connected non-solid liquid pool (plus active obsidian 56), then every
/// counted cell turns to shimmer-255 and the tree/obsidian/cut tiles die.
/// The C# recurses (`ShimmeratorNext` calls itself on the four neighbours);
/// the port walks an explicit LIFO stack so a deep pool never overflows the
/// call stack (the `flood_fill_wall_variety` iterative-fill precedent). The
/// neighbour push order is down/up/right/left, so the pop visits left first —
/// a deterministic DFS pre-order (the reverse-push of the C#'s left/right/up/
/// down recursion). The exact visit order differs from the C# recursion only
/// in traversal sequence, not in the counted SET (the same connected pool);
/// the `Shimminate` write loop replays the port's insertion order, which is
/// deterministic — the a == b contract. `CountedTiles` is a `HashSet`
/// (membership only — the C# `Dictionary<Point, bool>`'s value is always
/// `true`); `order` records the insertion sequence for the write loop.
fn shimmerator(world: &mut World, x: i32, y: i32) {
    let mut counted: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
    let mut order: Vec<(i32, i32)> = Vec::new();
    let mut stack: Vec<(i32, i32)> = vec![(x, y)];
    let mut num_tile_count = 0usize;
    while let Some((px, py)) = stack.pop() {
        if num_tile_count >= SHIMMERATOR_MAX_TILES {
            break;
        }
        // `:35133-35136` — the edge cell saturates the budget (the C# sets
        // numTileCount = maxTileCount, which aborts the whole flood).
        if px <= 1 || px >= world.width - 1 || py <= 1 || py >= world.height - 1 {
            num_tile_count = SHIMMERATOR_MAX_TILES;
            break;
        }
        if counted.contains(&(px, py)) {
            continue;
        }
        let Some(t) = world.tile(px, py).copied() else {
            continue;
        };
        let qualifies = (!solid_tile_ignore_water(world, px, py)
            && !tile_shimmer(&t)
            && t.liquid > 0)
            || (t.active() && t.kind == 56);
        if !qualifies {
            continue;
        }
        counted.insert((px, py));
        order.push((px, py));
        num_tile_count += 1;
        // `:35141-35144` — the C# recurses left, right, up, down. The LIFO
        // stack pops last-pushed-first, so push down/up/right/left to visit
        // left first (the reverse-push DFS pre-order — deterministic, a == b).
        stack.push((px, py + 1));
        stack.push((px, py - 1));
        stack.push((px + 1, py));
        stack.push((px - 1, py));
    }
    // `:35123` — Shimminate only when 0 < numTileCount < maxTileCount.
    if num_tile_count == 0 || num_tile_count >= SHIMMERATOR_MAX_TILES {
        return;
    }
    // `Shimminate` (`:35104-35116`): every counted cell → shimmer-255, then
    // the tree 5 / obsidian 56 / cut (except pot 28) tiles die.
    for &(px, py) in &order {
        if let Some(cell) = world.tile_mut(px, py) {
            cell.set_liquid_kind(LiquidKind::Shimmer);
            cell.liquid = u8::MAX;
        }
        let kind = world.tile(px, py).map_or(0, |t| t.kind);
        if kind == 5 || kind == 56 || (crate::tileobject::tile_cut(kind) && kind != 28) {
            crate::edit::kill_tile(world, px, py, true);
        }
    }
}

/// `WorldGen.ShimmerCleanUp` (`WorldGen.cs:34716-34821`) — the Final Cleanup
/// pass's shimmer arm (`:22472`). `shimmer_position` is the `GenVars` centre
/// g36 A1 publishes (`None` = the zero-vector sentinel, the C# `:34718` early
/// return). `anniversary` gates the tenth-anniversary drip/Shimmerator tail
/// (`:34789-34809`). Draws on the pass's own stream (the two `NextDouble`
/// jitter rolls per outer-sweep cell, `:34753`); the merger passes
/// `pass_rng("Final Cleanup")` (the scatter's stream, the C# `genRand`).
///
/// UNWIRED by design — the merger composes this into g37's Final Cleanup at
/// `:22472`. The `Liquid.LiquidCheck` merge sweep that follows it in the C#
/// (`:22473-22483`) is the f25/d106 liquid-core seam (§3.3), not this arm.
/// The witnesses below exercise it (the g39 `place_helpers` unwired convention);
/// `allow(dead_code)` keeps the non-test build quiet until the merger wires it.
#[allow(dead_code)]
pub fn shimmer_clean_up(
    world: &mut World,
    shimmer_position: Option<(i32, i32)>,
    anniversary: bool,
    rng: &mut Rng,
) {
    // `:34718-34721` — the zero-vector sentinel: no shimmer placed, no arm.
    let Some((cx, cy)) = shimmer_position else {
        return;
    };
    if cx == 0 && cy == 0 {
        return;
    }
    // `:34722` — ShimmerRemoveWater.
    shimmer_remove_water(world, cx, cy);
    let num = 120;
    let num2 = 90;
    // `:34730-34763` — the ±120 box, two distance gates per cell.
    for i in (cy - num)..=(cy + num) {
        for j in (cx - num)..=(cx + num) {
            if world.tile(j, i).is_none() {
                continue;
            }
            // `:34734` — the inner 120-radius disc (no jitter): the evil-ore
            // cleanse, the merge-block/cobweb kill, the moss cleanse, the
            // long-moss kill. Each `if` re-reads the (possibly just-written)
            // kind — the C# reads Main.tile[j,i].type fresh each time.
            let dx = (j - cx).abs() as f64;
            let dy = (i - cy).abs() as f64;
            if ((dx * dx + dy * dy).sqrt() as i32) < num {
                let kind = world.tile(j, i).map_or(0, |t| t.kind);
                if kind == 22 || kind == 204 {
                    if let Some(cell) = world.tile_mut(j, i) {
                        cell.kind = 1;
                    }
                }
                let kind = world.tile(j, i).map_or(0, |t| t.kind);
                if matches!(kind, 51 | 56 | 229 | 230 | 659) {
                    if let Some(cell) = world.tile_mut(j, i) {
                        cell.flags &= !FLAG_ACTIVE;
                    }
                }
                let kind = world.tile(j, i).map_or(0, |t| t.kind);
                if crate::tiledata::SET_CONVERSION_MOSS.has(kind) {
                    if let Some(cell) = world.tile_mut(j, i) {
                        cell.kind = 1;
                    }
                }
                let kind = world.tile(j, i).map_or(0, |t| t.kind);
                if kind == 184 {
                    if let Some(cell) = world.tile_mut(j, i) {
                        cell.flags &= !FLAG_ACTIVE;
                    }
                }
            }
            // `:34753` — the outer 90-radius ellipse, y-stretch 1.4 above the
            // centre / 1.2 below, each axis jittered ±2% (two NextDouble per
            // cell, drawn unconditionally — the C# ternary picks the stretch
            // but both arms draw the same two rolls).
            let y_stretch = if (i as f64) > cy as f64 { 1.2 } else { 1.4 };
            let jx = (j - cx).abs() as f64 * (1.0 + rng.next_f64() * 0.02);
            let jy = (i - cy).abs() as f64 * y_stretch * (1.0 + rng.next_f64() * 0.02);
            if ((jx * jx + jy * jy).sqrt() as i32) < num2 {
                if world.tile(j, i).is_some_and(|t| t.kind == 162) {
                    if let Some(cell) = world.tile_mut(j, i) {
                        cell.flags &= !FLAG_ACTIVE;
                    }
                }
                if let Some(cell) = world.tile_mut(j, i) {
                    cell.wall = 0;
                }
                // `:34760` — Convert(j, i, 0, 3): the cleanse, size 3.
                crate::spread::convert_cleanse(world, j, i, 3);
            }
        }
    }
    // `:34764-34788` — the pool refill walk. `Liquid.tilesIgnoreWater(true)`
    // (`:34767`) only affects the liquid solidity reads, which this arm does
    // not make (it writes liquid directly); the window is a no-op here and is
    // documented rather than modelled (the `liquid.rs` ignore_solids seam is
    // the runtime pass's, §3.3).
    let mut num7 = cx;
    let mut num8 = cy;
    let mut b: u8 = 127;
    // Sub-vanilla bounds guard (the `pass_shimmer` width-guard precedent): the
    // C# walks Main.tile[num7, num8] unguarded and relies on the carve's stone
    // shell to stop it; on a degenerate fixture the walk could leave the map.
    // The guard breaks before an out-of-bounds read (a generated world's shell
    // always stops the walk first, so it never fires in practice).
    loop {
        let cur = world.tile(num7, num8).copied();
        let Some(cur) = cur else { break };
        if !(cur.liquid <= b || !tile_shimmer(&cur)) {
            break;
        }
        // `:34770-34775` — walk left filling inactive cells.
        while world.tile(num7, num8).is_some_and(|t| !t.active()) {
            if let Some(cell) = world.tile_mut(num7, num8) {
                cell.liquid = b;
                cell.set_liquid_kind(LiquidKind::Shimmer);
            }
            num7 -= 1;
            if !world.in_bounds(num7, num8) {
                break;
            }
        }
        // `:34776-34780` — walk right from the centre filling inactive cells.
        num7 = cx;
        while world.tile(num7, num8).is_some_and(|t| !t.active()) {
            if let Some(cell) = world.tile_mut(num7, num8) {
                cell.liquid = b;
                cell.set_liquid_kind(LiquidKind::Shimmer);
            }
            num7 += 1;
            if !world.in_bounds(num7, num8) {
                break;
            }
        }
        num7 = cx;
        num8 += 1;
        b = u8::MAX;
        if !world.in_bounds(num7, num8) {
            break;
        }
        if world.tile(num7, num8).is_some_and(|t| t.active()) {
            break;
        }
    }
    // `:34789-34809` — the tenth-anniversary tail: the ±170 × [+40, height-430)
    // band retypes the drip blocks to shimmer-drip 709 and Shimmerators every
    // non-shimmer/non-honey liquid cell (wall != 86).
    if anniversary {
        let num9 = 170;
        for k in (cx - num9)..=(cx + num9) {
            for l in (cy + 40)..(world.height - 330 - 100) {
                if !in_world(world, k, l, 0) {
                    continue;
                }
                let Some(t) = world.tile(k, l).copied() else {
                    continue;
                };
                if t.wall == 86 {
                    continue;
                }
                if matches!(t.kind, 375 | 374 | 373) {
                    if let Some(cell) = world.tile_mut(k, l) {
                        cell.kind = 709;
                    }
                }
                if t.liquid > 0 && !tile_shimmer(&t) && !tile_honey(&t) {
                    shimmerator(world, k, l);
                }
            }
        }
    }
    // `:34810` — Liquid.tilesIgnoreWater(false): the window closes (no-op here,
    // documented above).
    // `:34811-34820` — the tree-under-shimmer kill: every active tree 5 holding
    // shimmer liquid in the 10-margin box dies.
    for m in 10..(world.width - 10) {
        for n in 10..(world.height - 10) {
            let Some(t) = world.tile(m, n).copied() else {
                continue;
            };
            if t.liquid > 0 && tile_shimmer(&t) && t.kind == 5 {
                crate::edit::kill_tile(world, m, n, true);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stone box world: solid stone rows [40, height), air above — the
    /// shimmer carve's substrate.
    fn stone_world(width: i32, height: i32) -> World {
        let mut w = World::new("shimmer".into(), width, height);
        for x in 0..width {
            for y in 40..height {
                let i = w.index(x, y);
                w.tiles[i].kind = TILE_STONE;
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w
    }

    /// W4-g36 A1: the veto scan (`WorldGen.cs:34875-34888`) refuses an
    /// out-of-bounds rect and a crimstone/ebonstone cell WITHOUT writing.
    #[test]
    fn a1_the_veto_scan_refuses_bounds_and_evil_stone() {
        let mut w = stone_world(300, 300);
        let before = w.tiles.clone();
        // The rect [x±105..125, y±105..125] at (60, 150) leaves the world.
        assert!(!shimmer_make_biome(&mut w, 60, 150, &mut Rng(7)));
        assert_eq!(w.tiles, before, "the veto leaves the world untouched");
        // A crimstone cell inside the rect vetoes too.
        let i = w.index(150, 150);
        w.tiles[i].kind = TILE_CRIMSTONE;
        let before = w.tiles.clone();
        assert!(!shimmer_make_biome(&mut w, 150, 150, &mut Rng(7)));
        assert_eq!(w.tiles, before);
    }

    /// W4-g36 A1: a successful carve leaves the shimmer pool (liquid 127
    /// on the centre row, 255 below, the shimmer bit set) inside a stone
    /// shell, and the pass is deterministic (a == b).
    #[test]
    fn a1_the_carve_pools_shimmer_and_replays() {
        let mut a = stone_world(400, 400);
        let mut b = stone_world(400, 400);
        assert!(shimmer_make_biome(&mut a, 200, 200, &mut Rng(42)));
        assert!(shimmer_make_biome(&mut b, 200, 200, &mut Rng(42)));
        assert_eq!(a.tiles, b.tiles, "a == b");
        let pool: Vec<&Tile> = a
            .tiles
            .iter()
            .filter(|t| t.liquid > 0 && t.flags & FLAG_LIQUID_SHIMMER != 0)
            .collect();
        assert!(!pool.is_empty(), "the pool must hold shimmer");
        assert!(pool.iter().any(|t| t.liquid == 127), "the centre row is 127");
        assert!(pool.iter().any(|t| t.liquid == 255), "the deep rows are 255");
    }

    /// W4-g36 A1: the full pass terminates on a vanilla-width world (the
    /// retry loop's bands stay non-empty), publishes a centre inside the
    /// world, and replays tile-for-tile. The 512-wide world takes the
    /// sub-vanilla width guard (the x-band `Next(200, (int)(0.11*512))`
    /// is empty — the C# throws; the port skips before any draw).
    #[test]
    fn a1_the_pass_terminates_and_replays() {
        // Layers (100, 160): the depth band is [180, 446] — the stone
        // substrate below row 40 covers it, and the dungeon-side x-band
        // [200, 264) stays inside the InWorld(50) margin on 2400 columns.
        let mut a = stone_world(2400, 1200);
        let mut b = stone_world(2400, 1200);
        let pa = pass_shimmer(&mut a, 100.0, 160.0, 1, false, &mut Rng(42));
        let pb = pass_shimmer(&mut b, 100.0, 160.0, 1, false, &mut Rng(42));
        assert!(pa.is_some(), "a vanilla-width world places the shimmer");
        assert_eq!(pa, pb);
        assert_eq!(a.tiles, b.tiles, "a == b");
        let (x, y) = pa.unwrap();
        assert!(in_world(&a, x, y, 0));
        // The 512-wide guard: no draws, no writes.
        let mut small = stone_world(512, 512);
        let before = small.tiles.clone();
        let mut rng = Rng(42);
        assert_eq!(pass_shimmer(&mut small, 100.0, 220.0, 1, false, &mut rng), None);
        assert_eq!(small.tiles, before, "the width guard writes nothing");
        assert_eq!(rng.0, Rng(42).0, "the width guard draws nothing");
    }

    /// W4-g36 A1: the opening walk clears its column band and stops at the
    /// 100-step budget (`WorldGen.cs:35101`).
    #[test]
    fn a1_the_opening_clears_and_stops_at_the_budget() {
        let mut w = stone_world(400, 400);
        // Solid stone everywhere below row 40: the walk never finds three
        // clear columns, so it runs to the |num - X| <= 100 budget.
        shimmer_make_biome_opening(&mut w, 1, 150, 200, 10, &mut Rng(3));
        // The walked columns' band rows are inactive now.
        assert!(!w.tile(151, 195).unwrap().active());
        // Beyond the budget the stone survives.
        assert!(w.tile(260, 195).unwrap().active());
    }

    // --- ShimmerCleanUp witnesses (the Final Cleanup tail arm) ---------------

    /// A stone-box world with a shimmer centre at (cx, cy): solid stone below
    /// row 40, an air pocket around the centre, and `frame_important` sized so
    /// the `kill_tile` cascade reads it safely. The centre cell holds a
    /// shimmer-127 pool row (the carve's shape) so the refill walk terminates.
    fn cleanup_world(width: i32, height: i32, cx: i32, cy: i32) -> World {
        let mut w = World::new("shimmer-cleanup".into(), width, height);
        w.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        for x in 0..width {
            for y in 40..height {
                let i = w.index(x, y);
                w.tiles[i].kind = TILE_STONE;
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        // An air pocket (cx±30, cy±30) so the sweep's writes are observable.
        for x in (cx - 30)..=(cx + 30) {
            for y in (cy - 30)..=(cy + 30) {
                if w.in_bounds(x, y) {
                    let i = w.index(x, y);
                    w.tiles[i] = Tile::default();
                }
            }
        }
        // The centre pool row: shimmer-127 at the centre, 255 below (the
        // carve's liquid shape) — the refill walk's termination anchor.
        let i = w.index(cx, cy);
        w.tiles[i].liquid = 127;
        w.tiles[i].set_liquid_kind(LiquidKind::Shimmer);
        for dy in 1..=3 {
            if w.in_bounds(cx, cy + dy) {
                let j = w.index(cx, cy + dy);
                w.tiles[j].liquid = 255;
                w.tiles[j].set_liquid_kind(LiquidKind::Shimmer);
            }
        }
        w
    }

    /// The zero-vector sentinel (`:34718`): `None` and `Some((0,0))` both
    /// return before any write or draw.
    #[test]
    fn cleanup_the_zero_sentinel_is_a_noop() {
        let mut w = cleanup_world(400, 400, 200, 200);
        let before = w.tiles.clone();
        let mut rng = Rng(42);
        shimmer_clean_up(&mut w, None, false, &mut rng);
        assert_eq!(w.tiles, before, "None writes nothing");
        assert_eq!(rng.0, Rng(42).0, "None draws nothing");
        shimmer_clean_up(&mut w, Some((0, 0)), false, &mut rng);
        assert_eq!(w.tiles, before, "the zero vector writes nothing");
        assert_eq!(rng.0, Rng(42).0, "the zero vector draws nothing");
    }

    /// The inner-disc cleanse (`:34736-34751`): an evil-ore 22 → stone 1, a
    /// cobweb 51 deactivates, a moss conversion tile → stone 1, a long-moss
    /// 184 deactivates — all inside the 120-radius disc around the centre.
    #[test]
    fn cleanup_the_inner_disc_cleanses_and_kills() {
        let mut w = cleanup_world(400, 400, 200, 200);
        // Place the targets near the centre (well inside radius 120).
        let set = |w: &mut World, x: i32, y: i32, kind: u16| {
            let i = w.index(x, y);
            w.tiles[i].kind = kind;
            w.tiles[i].flags |= FLAG_ACTIVE;
        };
        set(&mut w, 205, 200, 22); // demonite → stone
        set(&mut w, 210, 200, 51); // cobweb → inactive
        set(&mut w, 215, 200, 184); // long moss → inactive
        // A moss conversion tile (the SET_CONVERSION_MOSS family).
        let moss_kind = (0..crate::tiledata::TILE_COUNT as u16)
            .find(|&k| crate::tiledata::SET_CONVERSION_MOSS.has(k))
            .expect("a moss conversion kind exists");
        set(&mut w, 220, 200, moss_kind);
        shimmer_clean_up(&mut w, Some((200, 200)), false, &mut Rng(7));
        assert_eq!(w.tile(205, 200).unwrap().kind, 1, "the demonite cleansed");
        assert!(!w.tile(210, 200).unwrap().active(), "the cobweb died");
        assert!(!w.tile(215, 200).unwrap().active(), "the long moss died");
        assert_eq!(w.tile(220, 200).unwrap().kind, 1, "the moss cleansed");
    }

    /// The outer-ellipse wall clear + cleanse (`:34753-34761`): a walled cell
    /// inside the 90-radius ellipse loses its wall and runs Convert-0. The
    /// two NextDouble jitter rolls are drawn per cell (a == b replays them).
    #[test]
    fn cleanup_the_outer_ellipse_clears_walls_and_replays() {
        let build = || {
            let mut w = cleanup_world(400, 400, 200, 200);
            // A dirt wall (2) on a stone cell near the centre — Convert-0
            // turns the wall to the cleanse target and the wall clears.
            for x in 190..=210 {
                let i = w.index(x, 195);
                w.tiles[i].wall = 2;
                w.tiles[i].kind = TILE_STONE;
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
            w
        };
        let mut a = build();
        let mut b = build();
        shimmer_clean_up(&mut a, Some((200, 200)), false, &mut Rng(42));
        shimmer_clean_up(&mut b, Some((200, 200)), false, &mut Rng(42));
        assert_eq!(a.tiles, b.tiles, "a == b — the jitter stream replays");
        // The centre-adjacent walled cells lost their wall (the ellipse gate).
        assert_eq!(a.tile(200, 195).unwrap().wall, 0, "the wall cleared");
    }

    /// The pool refill walk (`:34764-34788`): an inactive cell above the
    /// centre pool fills with shimmer-127, and the walk terminates at the
    /// active floor below (the carve's stone shell).
    #[test]
    fn cleanup_the_refill_walk_fills_and_terminates() {
        let mut w = cleanup_world(400, 400, 200, 200);
        // An inactive cell just left of the centre on the pool row.
        let i = w.index(199, 200);
        w.tiles[i] = Tile::default(); // inactive, no liquid
        shimmer_clean_up(&mut w, Some((200, 200)), false, &mut Rng(3));
        let t = w.tile(199, 200).unwrap();
        assert!(t.liquid > 0, "the refill filled the inactive cell");
        assert_eq!(t.liquid_kind(), LiquidKind::Shimmer, "filled with shimmer");
    }

    /// The tree-under-shimmer kill (`:34811-34820`): an active tree 5 holding
    /// shimmer liquid in the 10-margin box dies.
    #[test]
    fn cleanup_the_tree_under_shimmer_dies() {
        let mut w = cleanup_world(400, 400, 200, 200);
        let i = w.index(100, 100);
        w.tiles[i].kind = 5; // tree
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.tiles[i].liquid = 200;
        w.tiles[i].set_liquid_kind(LiquidKind::Shimmer);
        shimmer_clean_up(&mut w, Some((200, 200)), false, &mut Rng(5));
        assert!(!w.tile(100, 100).unwrap().active(), "the shimmer-soaked tree died");
    }

    /// The tenth-anniversary tail (`:34789-34809`): a drip block 373 in the
    /// band retypes to shimmer-drip 709; the classic arm leaves it alone.
    #[test]
    fn cleanup_the_anniversary_tail_retypes_drips() {
        let build = || {
            let mut w = cleanup_world(600, 900, 300, 300);
            // A drip 373 below the centre+40 band, wall != 86.
            let i = w.index(300, 360);
            w.tiles[i].kind = 373;
            w.tiles[i].flags |= FLAG_ACTIVE;
            w.tiles[i].wall = 1;
            w
        };
        let mut classic = build();
        shimmer_clean_up(&mut classic, Some((300, 300)), false, &mut Rng(9));
        assert_eq!(classic.tile(300, 360).unwrap().kind, 373, "classic keeps the drip");
        let mut anniversary = build();
        shimmer_clean_up(&mut anniversary, Some((300, 300)), true, &mut Rng(9));
        assert_eq!(anniversary.tile(300, 360).unwrap().kind, 709, "anniversary retypes to shimmer-drip");
    }

    /// The Shimmerator flood (`:35118-35147`): a connected non-shimmer liquid
    /// pool turns to shimmer-255; the flood is bounded and deterministic.
    #[test]
    fn cleanup_the_shimmerator_floods_the_pool() {
        let build = || {
            let mut w = cleanup_world(600, 900, 300, 300);
            // A water pool (non-shimmer) in the anniversary band, below cy+40.
            for x in 290..=310 {
                for y in 350..=360 {
                    let i = w.index(x, y);
                    w.tiles[i] = Tile::default();
                    w.tiles[i].liquid = 200;
                    w.tiles[i].set_liquid_kind(LiquidKind::Water);
                }
            }
            w
        };
        let mut a = build();
        let mut b = build();
        shimmer_clean_up(&mut a, Some((300, 300)), true, &mut Rng(11));
        shimmer_clean_up(&mut b, Some((300, 300)), true, &mut Rng(11));
        assert_eq!(a.tiles, b.tiles, "a == b — the flood replays");
        // The pool cells turned to shimmer-255 (the Shimmerator write).
        let t = a.tile(300, 355).unwrap();
        assert_eq!(t.liquid_kind(), LiquidKind::Shimmer, "the pool shimmerated");
        assert_eq!(t.liquid, 255, "filled to 255");
    }
}
