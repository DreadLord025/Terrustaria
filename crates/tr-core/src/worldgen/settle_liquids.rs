//! The `Settle Liquids` pass (`WorldGen.cs:16132-16210`,
//! `GenPassNameID.SettleLiquids`, weight 9398.525) — W4-f25, the orphan
//! adoption (`docs/research/12g-f25-settle-liquids.md`). The first real
//! liquid-simulation pass of the generator: the ocean retype sweep, the
//! `QuickWater(3)` bottom-up pre-settle, the `WaterCheck` re-registration and
//! the 10-round `quickSettle` budget loop, all riding the port's
//! [`LiquidSystem`] (the faithful `UpdateLiquid`/`AddWater`/buffer port).
//!
//! Vanilla slot: after Jungle Chests (`:15977`), before Remove Water From
//! Sand (`:16211`). The port's chain groups the later carve/decor family
//! between them, so the step sits immediately before `remove water` — the
//! M5-OCEAN phase-4 neighbourhood — with every liquid-producing pass
//! (Lakes, the ocean family, the jungle/shrine water) already applied.
//!
//! No RNG draws in the whole pass (`QuickWater`/`SettleWaterAt`/`WaterCheck`/
//! `UpdateLiquid` draw nothing — the `kill_tile` drops ride the port's
//! deterministic position seeds), so the insertion is ledger-neutral for
//! every other step's stream.
//!
//! Documented deviations (ANALYSIS "The Settle Liquids pass (W4-f25)"):
//! - The per-pass [`LiquidSystem`] instance stands in for the C# statics;
//!   `ClearPendingLiquid` + the `numLiquid == 0` exit invariant
//!   (`WorldGenerator.cs:508`) make the instance boundary equivalent.
//! - `Skyblock.denyAllGeneration` (`:16134`), `SecretSeed.extraLiquid`
//!   (`:16147-16151`), `notTheBees` (`:16152-16155`) and the `noTraps`
//!   boulder arms (`Liquid.cs:109-112`/`:139-142`) are special-seed bodies
//!   the d107/c30 lanes own — flag-gated, classic no-ops here.
//! - `GenVars.shimmerPosition` is d102's Shimmer-pass product (unmerged at
//!   this landing): the `ShimmerRemoveWater` arms read the port's
//!   `GenVars.shimmer_position` seam (`None` until d102's step writes it —
//!   the C# `Vector2D.Zero` reset, `WorldGen.cs:7039`, makes the arm a
//!   corner no-op on a classic world either way).
//! - `Main.tileSolid[484] = false` (`:16208`) is the Gems-pass sand-slide
//!   exemption made permanent; the port's static tile tables do not model
//!   load-order pokes (the `desert.rs` precedent) — recorded, not applied.
//!
//! W4-f25 g46 liquid-union (2026-11-21, research/70 carried on
//! `task/W4-g46-liquid-evaporation` @ 9bc722e5f): at the liquid merge this
//! driver DEDUPES onto the g46-union `LiquidSystem` surfaces — the private
//! `quick_water`/`settle_water_at`/`attempt_to_move`/`shimmer_remove_water`/
//! `in_any_potential_dungeon_bounds` copies DELETE onto
//! `LiquidSystem::quick_water`/`gen_settle_loop` + the
//! `set_gen_settle_context`/`set_gen_shimmer_position`/
//! `set_gen_dungeon_context` seams (the export surface f25 landed is
//! ALREADY carried there, the R-12-5 addendum); the ocean retype sweep +
//! `LiquidInteractionsCleanup` stay driver-local (the union quick_water
//! documents the cleanup as the gen driver's). The ORIGIN-wall dungeon
//! fix (`Liquid.cs:152`/`:188`) is applied HERE pre-merge — both copies
//! now agree. The world-hash pins RE-DERIVE at the union gate (the real
//! DelWater drain/tail + swap-remove buffer + this fix move the gen
//! output); the gen-side `edit_kills` drain choice is DISCARD (the
//! worldgen has no loot side channel — the `square_tile_frame_gen`
//! precedent), recorded in 12g.

use crate::liquid::LiquidSystem;
use crate::tile::{LiquidKind, Tile};
use crate::world::World;

use super::pipeline::GenVars;

/// `WorldGen.oceanDepths` (`WorldGen.cs:10963-10974`): above `oceanLevel`
/// and within `beachDistance` of either edge.
fn ocean_depths(x: i32, y: i32, width: i32, ocean_level: f64) -> bool {
    if y as f64 > ocean_level {
        return false;
    }
    // `WorldGen.beachDistance` = 380 (`WorldGen.cs:86914`).
    x < 380 || x > width - 380
}

/// The pass body (`WorldGen.cs:16132-16210`). `remix` rides
/// `WorldGen.remixWorldGen` (the `SettleWaterAt` lava-retype arm and the
/// `LavaCheck` desert-strip gate); a classic world passes `false`.
pub(crate) fn settle_liquids(world: &mut World, gv: &GenVars, remix: bool) {
    let (width, height) = (world.width, world.height);
    // `WorldGen.oceanLevel` (`WorldGen.cs:4447`): `(worldSurface +
    // rockLayer) / 2 + 40`.
    let ocean_level = (world.world_surface + world.rock_layer) / 2.0 + 40.0;

    // --- the ocean retype sweep (`:16137-16146`) ---------------------------
    // Every ocean-depths cell holding a non-water liquid becomes water.
    for i in 0..width {
        for j in 0..height {
            if ocean_depths(i, j, width, ocean_level) {
                let idx = world.index(i, j);
                let t = &mut world.tiles[idx];
                if t.liquid > 0 && t.liquid_kind() != LiquidKind::Water {
                    t.clear_liquid_kind();
                }
            }
        }
    }

    // The per-pass instance — the C# statics' stand-in (module doc).
    let mut sys = LiquidSystem::new(world);
    sys.set_generating(remix);

    // `Liquid.worldGenTilesIgnoreWater(ignoreSolids: true)` (`:16156`).
    sys.world_gen_tiles_ignore_water(true);
    // `Liquid.QuickWater(3)` (`:16157`).
    quick_water(world, gv, &mut sys, remix);
    // `WorldGen.WaterCheck()` (`:16158`).
    sys.water_check(world);

    // --- the settle loop (`:16159-16204`) ----------------------------------
    // 10 outer rounds; each round budgets `num3 * 5` `UpdateLiquid` calls
    // (`num3` = the active+buffer count at the round head, re-raised whenever
    // the live count exceeds it) and runs until `numLiquid == 0` or the
    // budget breaks. `WaterCheck` re-registers between the rounds.
    sys.set_quick_settle(true);
    let mut num = 0;
    let num2 = 10;
    while num < num2 {
        let mut num3 = sys.num_liquid_total() as i64;
        num += 1;
        let mut num5 = num3 * 5;
        while sys.active_len() > 0 {
            num5 -= 1;
            if num5 < 0 {
                // The `extraLiquid` round bump (`:16173-16176`) is the
                // special-seed arm — classic breaks straight out.
                break;
            }
            let live = sys.num_liquid_total() as i64;
            if live > num3 {
                num3 = live;
            }
            // The progress fractions (`:16179-16203`) drive the loading bar
            // only — no tile effect, no draw.
            sys.update(world);
        }
        sys.water_check(world);
    }

    // --- the tail (`:16205-16208`) -----------------------------------------
    sys.clear_pending_liquid();
    sys.set_quick_settle(false);
    sys.world_gen_tiles_ignore_water(false);
    // `Main.tileSolid[484] = false` — the load-order poke the port's static
    // tables do not model (module doc / desert.rs precedent).
}

/// `Liquid.QuickWater(3)` (`Liquid.cs:104-148`): the bottom-up drop-and-fan
/// pre-settle. `verbose = 3` drives the progress display only (`:126`);
/// `minY`/`maxY` default to 3 / `maxTilesY - 3`. The generation arms call
/// `ShimmerRemoveWater` before and after the sweep; the `noTraps` boulder
/// arms are the special-seed lane's. The `LiquidInteractionsCleanup` tail
/// (`:144-147`) runs for a generating non-skyblock world.
fn quick_water(world: &mut World, gv: &GenVars, sys: &mut LiquidSystem, remix: bool) {
    shimmer_remove_water(world, gv);
    // `Main.tileSolid[379] = true` (`:114`) + `tilesIgnoreWater(true)` (`:115`).
    sys.set_bubble_solid(true);
    sys.tiles_ignore_water(true);
    let min_y = 3;
    let max_y = world.height - 3;
    for num in (min_y..=max_y).rev() {
        for i in 4..world.width - 4 {
            if world.tile(i, num).is_some_and(|t| t.liquid != 0) {
                settle_water_at(world, gv, sys, i, num, remix);
            }
        }
    }
    sys.tiles_ignore_water(false);
    shimmer_remove_water(world, gv);
    liquid_interactions_cleanup(world);
}

/// `WorldGen.ShimmerRemoveWater` (`WorldGen.cs:34823-34851`): inside the
/// 150-radius circle around `GenVars.shimmerPosition`, every non-shimmer
/// liquid is zeroed and every tombstone 162 deactivated. The position is
/// d102's Shimmer-pass product — `None` (the C# `Vector2D.Zero` reset) makes
/// the circle sit off-world, a no-op (module doc).
fn shimmer_remove_water(world: &mut World, gv: &GenVars) {
    let Some((sx, sy)) = gv.shimmer_position else {
        return;
    };
    let num = 150;
    for i in (sy - num)..=(sy + num / 2) {
        for j in (sx - num)..=(sx + num) {
            if j < 0 || i < 0 || j >= world.width || i >= world.height {
                continue;
            }
            let dx = (j - sx).abs() as f64;
            let dy = (i - sy).abs() as f64;
            if (dx * dx + dy * dy).sqrt() < num as f64 {
                let idx = world.index(j, i);
                if world.tiles[idx].liquid_kind() != LiquidKind::Shimmer {
                    world.tiles[idx].liquid = 0;
                }
                if world.tiles[idx].kind == 162 {
                    world.tiles[idx].flags &= !crate::tile::FLAG_ACTIVE;
                }
            }
        }
    }
}

/// `Liquid.SettleWaterAt` (`Liquid.cs:150-289`): lift the origin cell's
/// liquid, drop it through open air, fan it sideways into same-type
/// partially-full cells, and land the remainder. The generation retype
/// (`:178-192`) turns falling water to lava below the water line (the remix
/// band variant) and back to water inside the dungeon walls.
fn settle_water_at(
    world: &mut World,
    gv: &GenVars,
    sys: &mut LiquidSystem,
    ox: i32,
    oy: i32,
    remix: bool,
) {
    sys.tiles_ignore_water(true);
    let origin = world.tile(ox, oy).copied().unwrap_or_default();
    if origin.liquid == 0 || (origin.active() && origin.kind == 379) {
        return;
    }
    let mut num = ox;
    let mut num2 = oy;
    let tile_at_xy_has_lava = origin.liquid_kind() == LiquidKind::Lava;
    let flag = origin.liquid_kind() == LiquidKind::Honey;
    let flag2 = origin.liquid_kind() == LiquidKind::Shimmer;
    let mut num3 = origin.liquid as i32;
    let mut b = kind_code(origin.liquid_kind());
    world.tiles[world.index(ox, oy)].liquid = 0;
    let mut flag3 = true;
    loop {
        // --- the drop (`:169-177`) ----------------------------------------
        let mut tile2 = at(world, num, num2 + 1);
        let mut flag4 = false;
        while num2 < world.height - 5
            && tile2.liquid == 0
            && !sys.tile_is_solid(&tile2)
        {
            num2 += 1;
            flag4 = true;
            flag3 = false;
            tile2 = at(world, num, num2 + 1);
        }
        // --- the generation retype (`:178-192`) ---------------------------
        if flag4 && !flag && !flag2 {
            if remix {
                b = if num2 > gv.lava_line
                    && ((num2 as f64) < world.rock_layer - 80.0
                        || num2 > world.height - 350)
                    && !ocean_depths(num, num2, world.width, ocean_level(world))
                {
                    1
                } else {
                    0
                };
            } else if num2 > gv.water_line {
                b = 1;
            }
            if in_any_potential_dungeon_bounds(world, num, num2)
                && super::dungeon_features::is_dungeon_wall(origin.wall)
            {
                // W4-f25 g46 union fix (research/70 step 6): the C# `tile`
                // reference binds at the ORIGIN (`Liquid.cs:152`) and the
                // `:188` wall read is `tile.wall` — the ORIGIN cell's wall,
                // not the landing cell's (the pre-fix copy read
                // `world.tiles[index(num, num2)].wall`, a real deviation).
                // The bounds test DOES read the landing (num, num2) — both
                // arguments are the fan's current cell in the C#. This
                // moves the gen output where origin/landing walls differ;
                // the world-hash RE-DERIVE obligation covers it.
                b = 0;
            }
        }
        // --- the sideways fan (`:193-272`) --------------------------------
        let mut num4 = -1i32;
        let mut num5 = 0i32;
        let mut num6 = -1i32;
        let mut num7 = 0i32;
        let mut flag5 = false;
        let mut flag6 = false;
        let mut flag7 = false;
        loop {
            let cx = num + num5 * num4;
            if at(world, cx, num2).liquid == 0 {
                num6 = num4;
                num7 = num5;
            }
            if num4 == -1 && cx < 5 {
                flag6 = true;
            } else if num4 == 1 && cx > world.width - 5 {
                flag5 = true;
            }
            tile2 = at(world, cx, num2 + 1);
            if tile2.liquid != 0
                && tile2.liquid != u8::MAX
                && kind_code(tile2.liquid_kind()) == b
            {
                let mut num8 = 255 - tile2.liquid as i32;
                if num8 > num3 {
                    num8 = num3;
                }
                let j = world.index(cx, num2 + 1);
                world.tiles[j].liquid += num8 as u8;
                num3 -= num8;
                if num3 == 0 {
                    break;
                }
            }
            if num2 < world.height - 5 && tile2.liquid == 0 && !sys.tile_is_solid(&tile2) {
                flag7 = true;
                break;
            }
            let tile3 = at(world, num + (num5 + 1) * num4, num2);
            if (tile3.liquid != 0 && (!flag3 || num4 != 1)) || sys.tile_is_solid(&tile3) {
                if num4 == 1 {
                    flag5 = true;
                } else {
                    flag6 = true;
                }
            }
            if flag6 && flag5 {
                break;
            }
            if flag5 {
                num4 = -1;
                num5 += 1;
            } else if flag6 {
                if num4 == 1 {
                    num5 += 1;
                }
                num4 = 1;
            } else {
                if num4 == 1 {
                    num5 += 1;
                }
                num4 = -num4;
            }
        }
        num += num7 * num6;
        if num3 == 0 || !flag7 {
            break;
        }
        num2 += 1;
    }
    // --- the landing (`:280-288`) ------------------------------------------
    // The fan's 5-tile edge flags keep (num, num2) in-world by construction
    // (the C# `Main.tile[,]` indexer never wraps); the guard is the port's
    // bounds-safe stand-in for a degenerate sub-margin canvas.
    if num >= 0 && num2 >= 0 && num < world.width && num2 < world.height {
        let idx = world.index(num, num2);
        world.tiles[idx].liquid = num3 as u8;
        set_kind_code(&mut world.tiles[idx], b);
        if world.tiles[idx].liquid > 0 {
            attempt_to_move(world, sys, num, num2, tile_at_xy_has_lava, 1);
            attempt_to_move(world, sys, num, num2, flag, 2);
            attempt_to_move(world, sys, num, num2, flag2, 3);
        }
    }
    sys.tiles_ignore_water(false);
}

/// The `AttemptToMoveLava`/`Honey`/`Shimmer` family (`Liquid.cs:291-433`):
/// the three bodies are identical bar the kind flag and the check call —
/// the first side/up/down neighbour holding a liquid whose kind-flag differs
/// from the origin's runs the reaction check at the origin (when the origin
/// held the kind) or at that neighbour.
fn attempt_to_move(
    world: &mut World,
    sys: &mut LiquidSystem,
    x: i32,
    y: i32,
    origin_had_kind: bool,
    kind: u8,
) {
    for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
        let t = at(world, nx, ny);
        if t.liquid > 0 && (kind_code(t.liquid_kind()) == kind) != origin_had_kind {
            if origin_had_kind {
                sys.liquid_reaction_check(world, x, y, kind);
            } else {
                sys.liquid_reaction_check(world, nx, ny, kind);
            }
            return;
        }
    }
}

/// `WorldGen.LiquidInteractionsCleanup` (`WorldGen.cs:23672-23799`): every
/// obsidian 56 / shimmer-block 659 cell is cleared and refilled to 255 with
/// the single liquid kind its open neighbours hold (a mixed neighbourhood
/// keeps the block; an empty one defaults by block type — 56 → lava,
/// 659 → shimmer). The `:23753-23756` up-neighbour shimmer arm increments
/// the WATER counter (`num5++`) — a C# quirk kept verbatim.
fn liquid_interactions_cleanup(world: &mut World) {
    for i in 1..world.width - 2 {
        for j in 1..world.height - 2 {
            let idx = world.index(i, j);
            let tile = world.tiles[idx];
            if !tile.active() || (tile.kind != 56 && tile.kind != 659) {
                continue;
            }
            world.tiles[idx].liquid = 0;
            world.tiles[idx].clear_liquid_kind();
            // The four neighbours' kinds — only an INACTIVE liquid cell
            // contributes (`:23689-23692`).
            let kind_of = |t: Tile| -> i32 {
                if !t.active() && t.liquid > 0 {
                    kind_code(t.liquid_kind()) as i32
                } else {
                    -1
                }
            };
            let num = kind_of(at(world, i - 1, j));
            let num2 = kind_of(at(world, i + 1, j));
            let num3 = kind_of(at(world, i, j - 1));
            let num4 = kind_of(at(world, i, j + 1));
            // The per-kind neighbour counters (`:23697-23760`), verbatim:
            // water → num5, lava → num6, honey → num7, shimmer → num8 —
            // EXCEPT the UP neighbour's shimmer arm (`:23753-23756`) which
            // the C# writes as `num5++` (a water increment), a quirk kept 1:1.
            let (mut num5, mut num6, mut num7, mut num8) = (0, 0, 0, 0);
            if num == 0 { num5 += 1; }
            if num2 == 0 { num5 += 1; }
            if num3 == 0 { num5 += 1; }
            if num4 == 0 { num5 += 1; }
            if num == 1 { num6 += 1; }
            if num2 == 1 { num6 += 1; }
            if num3 == 1 { num6 += 1; }
            if num4 == 1 { num6 += 1; }
            if num == 2 { num7 += 1; }
            if num2 == 2 { num7 += 1; }
            if num3 == 2 { num7 += 1; }
            if num4 == 2 { num7 += 1; }
            if num == 3 { num8 += 1; }
            if num2 == 3 { num8 += 1; }
            if num3 == 3 { num5 += 1; } // the :23755 quirk (water, not shimmer)
            if num4 == 3 { num8 += 1; }
            let mixed = (num5 > 0 && (num6 > 0 || num7 > 0 || num8 > 0))
                || (num6 > 0 && (num5 > 0 || num7 > 0 || num8 > 0))
                || (num7 > 0 && (num5 > 0 || num6 > 0 || num8 > 0))
                || (num8 > 0 && (num5 > 0 || num6 > 0 || num7 > 0));
            if mixed {
                continue;
            }
            let mut liquid_type: u8 = 0;
            if num5 == 0 && num6 == 0 && num7 == 0 && num8 == 0 {
                liquid_type = match tile.kind {
                    56 => 1,
                    659 => 3,
                    _ => 0,
                };
            }
            if num5 > 0 {
                liquid_type = 0;
            } else if num6 > 0 {
                liquid_type = 1;
            } else if num7 > 0 {
                liquid_type = 2;
            } else if num8 > 0 {
                liquid_type = 3;
            }
            // `tile.ClearEverything()` (`Tile.cs:110-121`) then the refill.
            world.tiles[idx] = Tile::default();
            world.tiles[idx].liquid = u8::MAX;
            set_kind_code(&mut world.tiles[idx], liquid_type);
        }
    }
}

/// `DungeonUtils.InAnyPotentialDungeonBounds(x, y)` (`DungeonUtils.cs:2089-
/// 2108`) over the port's single-dungeon world: the outer potential bounds
/// `CreatePotentialDungeonBounds` computes (`:2067-2087`, the
/// `DungeonCrawler.cs:109` call — 0.1 middle / 0.05 edges, the surface+10 /
/// underworld−10 rows), the dungeon side fixed Right (M5-BIOME). The
/// `DungeonBounds` setters clamp to 10..max−10 and `CalculateHitbox` bumps a
/// degenerate extent (`DungeonBounds.cs:53-98`/`:310-322`).
fn in_any_potential_dungeon_bounds(world: &World, x: i32, y: i32) -> bool {
    let (w, h) = (world.width, world.height);
    let percent_on_top = (world.world_surface + 10.0) / h as f64;
    // `Main.UnderworldLayer` = maxTilesY − 200 (`Main.cs:2790`).
    let percent_on_bottom = (h - 200 - 10) as f64 / h as f64;
    let num = 0.1 / 2.0;
    // leftDungeon = false (the port's dungeon sits Right).
    let left = clamp_bound((w as f64 * (0.5 + num)) as i32, w);
    let right = clamp_bound(w - (w as f64 * 0.05) as i32, w);
    let top = clamp_bound((h as f64 * percent_on_top) as i32, h);
    let bottom = clamp_bound((h as f64 * percent_on_bottom) as i32, h);
    // CalculateHitbox's degenerate-extent bump.
    let right = if right <= left { left + 1 } else { right };
    let bottom = if bottom <= top { top + 1 } else { bottom };
    x >= left && x < right && y >= top && y < bottom
}

/// The `DungeonBounds.Left/Right/Top/Bottom` setter clamp
/// (`DungeonBounds.cs:53-98`): `MathHelper.Clamp(value, 10, max - 10)`.
fn clamp_bound(v: i32, max: i32) -> i32 {
    v.clamp(10, max - 10)
}

/// `WorldGen.oceanLevel` (`WorldGen.cs:4447`) — the retype arm's read.
fn ocean_level(world: &World) -> f64 {
    (world.world_surface + world.rock_layer) / 2.0 + 40.0
}

/// The C# `Main.tile[x, y]` indexer never returns null in-world; the fan's
/// ±walk stays inside the 5-tile margins, so an out-of-world read degrades
/// to the default (empty, non-solid) tile — the port's bounds-safe stand-in.
#[inline]
fn at(world: &World, x: i32, y: i32) -> Tile {
    world.tile(x, y).copied().unwrap_or_default()
}

#[inline]
fn kind_code(k: LiquidKind) -> u8 {
    match k {
        LiquidKind::Water => 0,
        LiquidKind::Lava => 1,
        LiquidKind::Honey => 2,
        LiquidKind::Shimmer => 3,
    }
}

#[inline]
fn set_kind_code(t: &mut Tile, code: u8) {
    t.set_liquid_kind(match code {
        1 => LiquidKind::Lava,
        2 => LiquidKind::Honey,
        3 => LiquidKind::Shimmer,
        _ => LiquidKind::Water,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    /// A 40×30 basin: stone floor at y=25, walls at x=0/39, open air above.
    fn basin() -> World {
        let mut w = World::new("settle".into(), 40, 30);
        for x in 0..40 {
            for y in 25..30 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        w.world_surface = 10.0;
        w.rock_layer = 15.0;
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    fn gv() -> GenVars {
        let mut g = GenVars::default();
        g.water_line = 20;
        g.lava_line = 24;
        g
    }

    fn total(w: &World) -> i64 {
        w.tiles.iter().map(|t| t.liquid as i64).sum()
    }

    #[test]
    fn the_pass_settles_a_suspended_pool_and_clears_the_queue() {
        // A 255-water column hanging in the air above the basin floor: the
        // QuickWater drop lands it on the floor, the settle loop levels it,
        // and ClearPendingLiquid empties the active list (the
        // WorldGenerator.cs:508 invariant).
        let mut w = basin();
        let i = w.index(20, 10);
        w.tiles[i].liquid = 255;
        w.mark_chunk(20, 10);
        let before = total(&w);
        settle_liquids(&mut w, &gv(), false);
        // The water reached the floor row (y=24, just above the stone).
        let floor: i64 = (1..39).map(|x| w.tiles[w.index(x, 24)].liquid as i64).sum();
        assert!(floor > 0, "the pool settled onto the floor ({floor})");
        assert_eq!(w.tiles[w.index(20, 10)].liquid, 0, "the origin emptied");
        // Mass is conserved up to the settle loop's >250 top-ups (the C#'s
        // own quickSettle bumps) — never lost.
        assert!(total(&w) >= before, "no liquid vanished");
    }

    #[test]
    fn the_ocean_retype_sweep_turns_lava_to_water() {
        // An ocean-depths cell (x < 380, above oceanLevel) holding lava is
        // retyped to water by the :16137-16146 sweep before any settling.
        let mut w = basin();
        // oceanLevel = (10 + 15)/2 + 40 = 52.5 — every row of the 30-tall
        // basin is above it, and x=2 is inside the left beachDistance.
        let i = w.index(2, 24);
        w.tiles[i].liquid = 100;
        w.tiles[i].set_liquid_kind(LiquidKind::Lava);
        settle_liquids(&mut w, &gv(), false);
        // Whatever liquid remains in the left ocean column is water-typed.
        for y in 0..30 {
            let t = w.tiles[w.index(2, y)];
            if t.liquid > 0 {
                assert_eq!(t.liquid_kind(), LiquidKind::Water, "ocean lava retyped at y{y}");
            }
        }
    }

    #[test]
    fn the_pass_is_deterministic() {
        let build = || {
            let mut w = basin();
            for (x, y) in [(10, 8), (20, 10), (30, 6)] {
                let i = w.index(x, y);
                w.tiles[i].liquid = 200;
                w.mark_chunk(x, y);
            }
            w
        };
        let mut a = build();
        let mut b = build();
        settle_liquids(&mut a, &gv(), false);
        settle_liquids(&mut b, &gv(), false);
        assert_eq!(a.tiles, b.tiles, "a == b");
    }

    #[test]
    fn the_cleanup_refills_a_lone_obsidian_block() {
        // LiquidInteractionsCleanup (:23672-23799): an obsidian 56 with only
        // water neighbours is cleared and refilled to 255 water.
        let mut w = basin();
        let i = w.index(20, 24);
        w.tiles[i] = Tile { kind: 56, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let j = w.index(19, 24);
        w.tiles[j].liquid = 100; // water, inactive cell
        liquid_interactions_cleanup(&mut w);
        let t = w.tiles[i];
        assert!(!t.active(), "the block was cleared");
        assert_eq!(t.liquid, 255);
        assert_eq!(t.liquid_kind(), LiquidKind::Water);
    }

    /// W4-f25 g46 union fix (research/70 step 6): the `SettleWaterAt`
    /// dungeon re-type reads the ORIGIN cell's wall — the C# `tile`
    /// reference binds at `:152` and `:188` reads `tile.wall`, NOT the
    /// landing cell's (the bounds test DOES read the landing cell).
    /// Fixture: on the 40x30 basin with `world_surface = 0` the potential
    /// bounds are x 22..38, y 10..11 (top = clamp(10) = 10, bottom =
    /// clamp(-180) = 10 bumped to 11). A FULL-WIDTH stone shelf at y=11
    /// holds the pool on row 10 (a partial shelf would let the fan's
    /// hole-scan walk the mass off the edge — the `flag7` escape); the
    /// origin (25, 9) — OUTSIDE the bounds — carries water 200 behind a
    /// dungeon wall 7 and drops onto (25, 10) INSIDE the bounds, below
    /// `waterLine = 5` (the classic retype fires, b = 1). The ORIGIN
    /// wall 7 re-types it back to water; the pre-fix landing-wall read
    /// (wall 0) left it lava. The fan finds no hole (every below-cell is
    /// the shelf), so the whole 200 lands on (25, 10) and the settle
    /// loop levels it across the row — the landing cell stays wet.
    /// WRITTEN NOT RUN (code-only round).
    #[test]
    fn w4_f25_g46_the_dungeon_retype_reads_the_origin_wall() {
        let shelf = |w: &mut World| {
            for x in 0..40 {
                let s = w.index(x, 11);
                w.tiles[s] = Tile { kind: 1, ..Default::default() };
                w.tiles[s].flags |= FLAG_ACTIVE;
            }
        };
        let mut w = basin();
        w.world_surface = 0.0;
        let mut g = gv();
        g.water_line = 5;
        shelf(&mut w);
        // The origin: water 200 at (25, 9) behind a dungeon wall 7.
        let o = w.index(25, 9);
        w.tiles[o].liquid = 200;
        w.tiles[o].wall = 7;
        w.mark_chunk(25, 9);
        settle_liquids(&mut w, &g, false);
        let t = w.tiles[w.index(25, 10)];
        assert!(t.liquid > 0, "the pool levelled on the shelf row");
        assert_eq!(t.liquid_kind(), LiquidKind::Water, "the ORIGIN wall re-type");
        // The mirror: the same drop with NO origin wall keeps the lava
        // retype (the landing cell's wall 0 cannot re-type it back).
        let mut w2 = basin();
        w2.world_surface = 0.0;
        shelf(&mut w2);
        let o2 = w2.index(25, 9);
        w2.tiles[o2].liquid = 200;
        w2.mark_chunk(25, 9);
        settle_liquids(&mut w2, &g, false);
        let t2 = w2.tiles[w2.index(25, 10)];
        assert!(t2.liquid > 0, "the mirror pool");
        assert_eq!(t2.liquid_kind(), LiquidKind::Lava, "no origin wall — the retype stands");
    }
}
