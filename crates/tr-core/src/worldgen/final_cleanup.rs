//! W4-g37 FINAL CLEANUP gap-fill — the `Final Cleanup` pass
//! (`WorldGen.cs:22141-22490`, `GenPassNameID.FinalCleanup`), the last classic
//! pass of the vanilla chain. Census at the claim probe: g36's master matrix
//! (`docs/research/88` @ 6d46f6a5) flags it "GAP — unlanded" (row 109) and
//! d106's `12f` R6 box is unchecked at its tip `e2c2609c` — no owner claimed
//! the body, so this lane fills the gap (docs/research/12h + 75 cross-notes).
//!
//! Ported arms (the classic world):
//! - the surface wall-hole fill (`FillWallHolesInArea` `:9886-10053`),
//! - the whole-map sweep (`:22150-22378`): the falling-sand column drop +
//!   hardening (53/112/234/224/123), the 187/216 lava top-up, the 137 slope
//!   clear, the boulder re-seat / 26-above collapse, the 323 liquid kill, the
//!   dungeon-wall liquid reset (374→373, 56→water-255), the 314 minecart
//!   liquid strip, the 332 torch floor, the beach-band partial-liquid clear,
//!   the painting wall inheritance,
//! - the 668 dirt-throw scatter (`:22379-22422`, `GetWorldSize` `:6559-6570`).
//!
//! Deferred (cited inline, the lane convention — special seeds + unported
//! machinery): the `tileSolid[484]=false` window (`:22143` — the port's
//! static-table convention, `passes_b.rs` module doc), the
//! `surfaceIsDesertNormalFunction` early arm (`:22146-22149` — the port runs
//! `DoSurfaceIsDesert` inside `micro_surface::run` at the seed-finish step,
//! the M5-SEED-2 grouping), every `Finish*` seed tail (`:22423-22456` — the
//! port's `step_seed_finish` owns the landed ones; FinishNoTraps /
//! NotTheBees / FinishNotTheBees / FinishRemixWorld / the anniversary paint
//! tail / ShimmerifySideOfWorld / PaintTheSandRainbowAndSandWallsBlack are
//! unported), the vampire-seed block (`:22457-22466`), `ShimmerCleanUp`
//! (`:22472` — reads `GenVars.shimmerPosition`, which g36 A1 owns; no slot on
//! this branch), the `Liquid.LiquidCheck` merge sweep (`:22473-22483` — the
//! f25/d106 liquid-core seam), `ExtraSpawnPointManager.PrepareExtraSpawns` /
//! `TownNPCPositionsCleanup` (`:22484-22485` — no town-NPC gen state in the
//! port), the `notTheBees`/`getGoodWorldGen` flag resets (`:22486-22487` —
//! the port's flags are immutable `GenOptions`), and `Main.AnglerQuestSwap`
//! (`:22488` — a `Main.rand` runtime draw, not a worldgen-stream draw; the
//! app's day-reset owns it, `world.rs:547-552`). The per-cell `TileFrame`
//! calls (`:22240`, `:22376`) are the port's documented framing deferral
//! (the later framing pass; `micro_surface.rs` precedent).
//!
//! RNG: the sweep draws none; the dirt-throw scatter draws on the pass's own
//! `pass_rng("Final Cleanup")` stream (the port's per-pass re-seed model).

use super::noise::Rng;
use super::passes_b::{in_world, ocean_depths};
use crate::tile::{LiquidKind, FLAG_ACTIVE, FLAG_HALF_BRICK, FLAG_INACTIVE};
use crate::tiledata as td;
use crate::world::World;

/// `Main.wallDungeon` (`Main.cs:10596-10607`) — the local-copy convention
/// (`evil.rs:111`, `finish_seeds.rs:69`).
const WALL_DUNGEON: [u16; 9] = [7, 8, 9, 94, 95, 96, 97, 98, 99];

/// `WorldGen.GetWorldSize` (`WorldGen.cs:6559-6570`): 0 ≤4200 wide, 1 ≤6400,
/// else 2.
fn get_world_size(width: i32) -> i32 {
    if width <= 4200 {
        0
    } else if width <= 6400 {
        1
    } else {
        2
    }
}

/// `WorldGen.SolidTile(i, j)` (`WorldGen.cs:70111-70131`) with the pass's
/// `tileSolid[484] = false` window (`:22143`) riding the predicate — the
/// static-table convention (`spider_gem_moss.rs`'s `tileSolid[379]`
/// precedent). The `noDoors` default is false; an OOB cell is not solid (the
/// C# catch returns false).
fn solid_tile_fc(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && t.kind != 484
        && td::TILE_SOLID.has(t.kind)
        && !td::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

/// `Tile.ClearEverything` (`Tile.cs:110-121`) — every header + the frames.
/// The `desert_entrances.rs` copy's shape, plus the wall-frame byte (the
/// port's packed `bTileHeader3` stand-in).
fn clear_everything(t: &mut crate::tile::Tile) {
    t.kind = 0;
    t.wall = 0;
    t.liquid = 0;
    t.flags = 0;
    t.meta = 0;
    t.frame_x = 0;
    t.frame_y = 0;
    t.wall_frame = 0;
}

/// `WorldGen.KillTile`'s generation-time strip (`WorldGen.cs:63517`, the
/// `isGeneratingOrLoadingWorld → noItem` path) — the `ocean.rs::kill_tile_gen`
/// copy (house style: local cited slices).
fn kill_tile_gen(t: &mut crate::tile::Tile) {
    if !t.active() {
        return;
    }
    t.flags &= !FLAG_ACTIVE;
    t.kind = 0;
    t.frame_x = 0;
    t.frame_y = 0;
}

// --- the surface wall-hole fill (`:22144`) -----------------------------------

/// `WorldGen.FillWallHolesInArea(Rectangle(0, 0, maxTilesX, worldSurface))`
/// (`WorldGen.cs:9886-9898`): the clamped column walk. `Rectangle.Right` is
/// `X + Width` = maxTilesX, so the `i <= num2` loop's last iteration is the
/// guarded no-op — the port walks `num..num2` (the `i >= 0 && i < maxTilesX`
/// guard is exactly that range).
pub(crate) fn fill_wall_holes_in_area(world: &mut World, world_surface: f64) {
    let end_y = world_surface as i32;
    for x in 0..world.width {
        fill_wall_holes_in_column(world, x, 0, end_y);
    }
}

/// `WorldGen.FillWallHolesInColumn` (`WorldGen.cs:9900-9926`): the x clamp
/// `[2, maxX-3]`, the y clamps, the wall→no-wall edge latch, one
/// `FillWallHolesInSpot(x, i, 150)` per edge.
fn fill_wall_holes_in_column(world: &mut World, x: i32, start_y: i32, end_y: i32) {
    let x = x.clamp(2, world.width - 1 - 2);
    let start_y = start_y.max(2);
    let end_y = end_y.min(world.height - 2);
    let mut flag = false;
    for i in start_y..end_y {
        if world.tile(x, i).map(|t| t.wall).unwrap_or(0) == 0 {
            if flag {
                flag = false;
                fill_wall_holes_in_spot(world, x, i, 150);
            }
        } else {
            flag = true;
        }
    }
}

/// `WorldGen.FillWallHolesInSpot` (`WorldGen.cs:9928-10053`): the two-list
/// BFS over the hole — the 150-cell budget, the `InWorld(1)` margin, walled
/// cells vote into the tally, wall-0 cells expand only through an inactive
/// 3-row/column cross, the majority wall fills the hole (no votes → wall 2).
fn fill_wall_holes_in_spot(world: &mut World, origin_x: i32, origin_y: i32, max_walls: usize) -> bool {
    if !in_world(world, origin_x, origin_y, 2) {
        return false;
    }
    // `hashSet` — the visited set; `list2` — the next ring. The C#'s
    // `list.Remove(item)` always removes the head it just read.
    let mut visited: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
    let mut ring: Vec<(i32, i32)> = vec![(origin_x, origin_y)];
    // `dictionary` — the wall tally. The C# `Dictionary` iteration order is
    // insertion order in practice; the port keeps a Vec of (wall, count) in
    // first-seen order so the `>` (not `>=`) majority pick replays the same
    // tie-break (the first wall to reach the max count wins).
    let mut tally: Vec<(u16, i32)> = Vec::new();
    while !ring.is_empty() {
        let mut list = std::mem::take(&mut ring);
        while let Some(item) = list.first().copied() {
            if visited.len() >= max_walls {
                return false;
            }
            list.remove(0);
            if visited.contains(&item) {
                continue;
            }
            if !in_world(world, item.0, item.1, 1) {
                continue;
            }
            visited.insert(item);
            let wall = world.tile(item.0, item.1).map(|t| t.wall).unwrap_or(0);
            if wall != 0 {
                match tally.iter_mut().find(|(w, _)| *w == wall) {
                    Some((_, c)) => *c += 1,
                    None => tally.push((wall, 1)),
                }
                continue;
            }
            // `:9976-10000`: expand only if an inactive cell sits in the
            // horizontal or vertical 3-cross.
            let mut flag = false;
            for i in (item.0 - 1)..=(item.0 + 1) {
                if !world.tile(i, item.1).is_some_and(|t| t.active()) {
                    flag = true;
                    break;
                }
            }
            if !flag {
                for j in (item.1 - 1)..=(item.1 + 1) {
                    if !world.tile(item.0, j).is_some_and(|t| t.active()) {
                        flag = true;
                        break;
                    }
                }
            }
            if flag {
                for next in [
                    (item.0 - 1, item.1),
                    (item.0 + 1, item.1),
                    (item.0, item.1 - 1),
                    (item.0, item.1 + 1),
                ] {
                    if !visited.contains(&next) {
                        ring.push(next);
                    }
                }
            }
        }
    }
    if visited.len() == 1 {
        return false;
    }
    // `:10030-10043`: the strict-`>` majority pick, no votes → wall 2.
    let mut wall: u16 = 0;
    let mut num = -1;
    for (w, c) in &tally {
        if *c > num {
            wall = *w;
            num = *c;
        }
    }
    if num == -1 {
        wall = 2;
    }
    for &(x, y) in &visited {
        if let Some(t) = world.tile_mut(x, y) {
            if t.wall == 0 {
                t.wall = wall;
            }
        }
    }
    true
}

// --- the whole-map sweep arms -------------------------------------------------

/// The falling-sand arm (`WorldGen.cs:22155-22242`): an active
/// 53/112/234/224/123 cell over a non-solid floor drops up to 10 cells
/// through air/tileCut (the 314 minecart veto, the surface+10 / wall-191 /
/// oceanDepths gates), hardens on exhaustion (397/398/399/147/1), de-slopes
/// a sloped solid floor, or hardens in place. The 323-above re-frame
/// (`:22238-22241`) is the framing deferral.
fn falling_sand_arm(world: &mut World, i: i32, j: i32, world_surface: f64, rock_layer: f64) {
    let kind = world.tile(i, j).map(|t| t.kind).unwrap_or(0);
    let below_open = world
        .tile(i, j + 1)
        .is_some_and(|t| !t.active() || crate::tileobject::tile_cut(t.kind));
    let below_wall = world.tile(i, j + 1).map(|t| t.wall).unwrap_or(0);
    if (j as f64) < world_surface + 10.0
        && below_open
        && below_wall != 191
        && !ocean_depths(world, i, j, world_surface, rock_layer)
    {
        let mut num = 10;
        let mut num2 = j + 1;
        // `:22159-22168`: the 314 minecart-track veto in the next 10 rows.
        for k in num2..(num2 + 10) {
            if world.tile(i, k).is_some_and(|t| t.active() && t.kind == 314) {
                num = 0;
                break;
            }
        }
        // `:22169-22179`: the drop — the source cell de-slopes each step.
        while world
            .tile(i, num2)
            .is_some_and(|t| !t.active() || crate::tileobject::tile_cut(t.kind))
            && num > 0
            && num2 < world.height - 50
        {
            if let Some(t) = world.tile_mut(i, num2 - 1) {
                t.set_slope(0);
                t.flags &= !FLAG_HALF_BRICK;
            }
            if let Some(t) = world.tile_mut(i, num2) {
                t.flags |= FLAG_ACTIVE;
                t.kind = kind;
                t.set_slope(0);
                t.flags &= !FLAG_HALF_BRICK;
            }
            num2 += 1;
            num -= 1;
        }
        let below = world.tile(i, num2).copied();
        if num == 0 && !below.is_some_and(|t| t.active()) {
            // `:22180-22205`: exhausted over air — harden.
            let hardened = match kind {
                53 => 397,
                112 => 398,
                234 => 399,
                224 => 147,
                123 => 1,
                _ => return,
            };
            if let Some(t) = world.tile_mut(i, num2) {
                t.kind = hardened;
                t.flags |= FLAG_ACTIVE;
            }
        } else if below.is_some_and(|t| {
            t.active() && td::TILE_SOLID.has(t.kind) && !td::TILE_SOLID_TOP.has(t.kind)
        }) {
            // `:22206-22210`: seated on a solid floor — de-slope it.
            if let Some(t) = world.tile_mut(i, num2) {
                t.set_slope(0);
                t.flags &= !FLAG_HALF_BRICK;
            }
        }
    } else if world.tile(i, j + 1).is_some_and(|t| {
        td::TILE_SOLID.has(t.kind)
            && !td::TILE_SOLID_TOP.has(t.kind)
            && (t.top_slope() || t.half_brick())
    }) {
        // `:22212-22216`: the sloped-solid-floor de-slope.
        if let Some(t) = world.tile_mut(i, j + 1) {
            t.set_slope(0);
            t.flags &= !FLAG_HALF_BRICK;
        }
    } else {
        // `:22217-22237`: harden in place.
        let hardened = match kind {
            53 => Some(397),
            112 => Some(398),
            234 => Some(399),
            224 => Some(147),
            123 => Some(1),
            _ => None,
        };
        if let Some(k) = hardened {
            if let Some(t) = world.tile_mut(i, j) {
                t.kind = k;
            }
        }
    }
    // `:22238-22241`: the 323-above `TileFrame` — the framing deferral.
}

/// The boulder re-seat arm (`WorldGen.cs:22253-22300`): every active
/// `TileID.Sets.Boulders` cell re-writes its 2×2 body from the frame-derived
/// origin; a 26 altar above collapses the body to stone (484 → 397).
fn boulder_arm(world: &mut World, i: i32, j: i32) {
    let Some(t) = world.tile(i, j).copied() else {
        return;
    };
    let num3 = i - i32::from(t.frame_x) / 18;
    let num4 = j - i32::from(t.frame_y) / 18;
    let mut flag = false;
    for l in 0..2 {
        if world
            .tile(num3 + l, num4 - 1)
            .is_some_and(|t| t.active() && t.kind == 26)
        {
            flag = true;
            break;
        }
        for m in 0..2 {
            if let Some(cell) = world.tile_mut(num3 + l, num4 + m) {
                cell.flags |= FLAG_ACTIVE;
                cell.set_slope(0);
                cell.flags &= !FLAG_HALF_BRICK;
                cell.kind = t.kind;
                cell.frame_x = (l * 18) as i16;
                cell.frame_y = (m * 18) as i16;
            }
        }
    }
    if flag {
        let kind: u16 = if t.kind == 484 { 397 } else { 0 };
        for n in 0..2 {
            for num7 in 0..2 {
                if let Some(cell) = world.tile_mut(num3 + n, num4 + num7) {
                    cell.flags |= FLAG_ACTIVE;
                    cell.set_slope(0);
                    cell.flags &= !FLAG_HALF_BRICK;
                    cell.kind = kind;
                    cell.frame_x = 0;
                    cell.frame_y = 0;
                }
            }
        }
    }
}

/// The painting wall-inheritance arm (`WorldGen.cs:22344-22375`): a
/// wall-less painting inside `InWorld(2)` takes the first non-zero wall of
/// its left/right/up/down neighbours, in that order.
fn painting_arm(world: &mut World, i: i32, j: i32) {
    if world.tile(i, j).map(|t| t.wall).unwrap_or(0) > 0 || !in_world(world, i, j, 2) {
        return;
    }
    let mut num13: u16 = 0;
    for (nx, ny) in [(i - 1, j), (i + 1, j), (i, j - 1), (i, j + 1)] {
        if num13 == 0 {
            num13 = world.tile(nx, ny).map(|t| t.wall).unwrap_or(0);
        }
    }
    if num13 != 0 {
        if let Some(t) = world.tile_mut(i, j) {
            t.wall = num13;
        }
    }
}

/// The `Final Cleanup` pass body (`WorldGen.cs:22141-22490`), the classic
/// arm. `world_surface`/`rock_layer` are the pass-time layer anchors (the
/// step adapter reads `GenVars.layers()` — the world's own fields are only
/// stamped at the world tail). Returns the dirt-throw count (`num14`).
pub(crate) fn pass_final_cleanup(
    world: &mut World,
    world_surface: f64,
    rock_layer: f64,
    remix: bool,
    dual_dungeons: bool,
    not_the_bees: bool,
    anniversary: bool,
    extra_liquid: bool,
    rng: &mut Rng,
) -> u32 {
    // `:22143`: `Main.tileSolid[484] = false` — rides `solid_tile_fc` (the
    // static-table convention; the only solidity read in the pass).
    // `:22144`: the surface wall-hole fill.
    fill_wall_holes_in_area(world, world_surface);
    // `:22145`: `progress.Message = Lang.gen[86]` — the progress table.
    // `:22146-22149`: the surfaceIsDesertNormalFunction early arm — deferred
    // (the port's DoSurfaceIsDesert runs at the seed-finish step).
    for i in 0..world.width {
        for j in 0..world.height {
            let t = match world.tile(i, j) {
                Some(t) => *t,
                None => continue,
            };
            // `:22155-22242`: the falling-sand arm.
            if t.active()
                && !solid_tile_fc(world, i, j + 1)
                && matches!(t.kind, 53 | 112 | 234 | 224 | 123)
            {
                falling_sand_arm(world, i, j, world_surface, rock_layer);
            }
            // `:22243-22247`: the 187/216 desert-wall lava top-up.
            if (t.wall == 187 || t.wall == 216) && t.liquid > 0 && !remix && !dual_dungeons {
                if let Some(cell) = world.tile_mut(i, j) {
                    cell.liquid = u8::MAX;
                    cell.set_liquid_kind(LiquidKind::Lava);
                }
            }
            // `:22248-22252`: the 137 dart-trap slope clear (no active gate).
            if t.kind == 137 {
                if let Some(cell) = world.tile_mut(i, j) {
                    cell.set_slope(0);
                    cell.flags &= !FLAG_HALF_BRICK;
                }
            }
            // `:22253-22300`: the boulder re-seat.
            if t.active() && td::SET_BOULDERS.has(t.kind) {
                boulder_arm(world, i, j);
            }
            // `:22301-22304`: the 323 fossil over liquid dies.
            if t.kind == 323 && t.liquid > 0 {
                if let Some(cell) = world.tile_mut(i, j) {
                    kill_tile_gen(cell);
                }
            }
            // `:22305-22318`: the dungeon-wall liquid reset.
            if WALL_DUNGEON.contains(&t.wall) {
                if let Some(cell) = world.tile_mut(i, j) {
                    cell.set_liquid_kind(LiquidKind::Water);
                }
                if world.tile(i, j).is_some_and(|t| t.kind == 374) {
                    if let Some(cell) = world.tile_mut(i, j) {
                        cell.kind = 373;
                    }
                }
                if world.tile(i, j).is_some_and(|t| t.active() && t.kind == 56) {
                    if let Some(cell) = world.tile_mut(i, j) {
                        kill_tile_gen(cell);
                        cell.set_liquid_kind(LiquidKind::Water);
                        cell.liquid = u8::MAX;
                    }
                }
            }
            // `:22319-22333`: the 314 minecart liquid strip — 15 rows up
            // (the C# `while (j - num12 < num10)` includes j), 1 row at j.
            if t.active() && t.kind == 314 && !extra_liquid {
                let mut num12 = j;
                while j - num12 < 15 {
                    if let Some(cell) = world.tile_mut(i, num12) {
                        cell.liquid = 0;
                    }
                    num12 -= 1;
                }
                // `:22329-22332`: num11 = 1 — the single j row (already
                // cleared by the loop above; kept as the C# writes it).
                let mut num12 = j;
                while num12 - j < 1 {
                    if let Some(cell) = world.tile_mut(i, num12) {
                        cell.liquid = 0;
                    }
                    num12 += 1;
                }
            }
            // `:22334-22339`: the 332 torch floor — ClearEverything + seat.
            if t.active() && t.kind == 332 && !world.tile(i, j + 1).is_some_and(|t| t.active()) {
                if let Some(cell) = world.tile_mut(i, j + 1) {
                    clear_everything(cell);
                    cell.flags |= FLAG_ACTIVE;
                    cell.kind = 332;
                }
            }
            // `:22340-22343`: the beach-band partial-liquid clear.
            if i > 380
                && i < world.width - 380
                && (j as f64) < world_surface
                && t.liquid > 0
                && t.liquid < u8::MAX
                && world.tile(i - 1, j).map(|t| t.liquid).unwrap_or(0) < u8::MAX
                && world.tile(i + 1, j).map(|t| t.liquid).unwrap_or(0) < u8::MAX
                && world.tile(i, j + 1).map(|t| t.liquid).unwrap_or(0) < u8::MAX
                && !world.tile(i - 1, j).is_some_and(|t| t.active() && td::SET_CLOUDS.has(t.kind))
                && !world.tile(i + 1, j).is_some_and(|t| t.active() && td::SET_CLOUDS.has(t.kind))
                && !world.tile(i, j + 1).is_some_and(|t| t.active() && td::SET_CLOUDS.has(t.kind))
            {
                if let Some(cell) = world.tile_mut(i, j) {
                    cell.liquid = 0;
                }
            }
            // `:22344-22375`: the painting wall inheritance.
            if t.active() && td::SET_PAINTINGS.has(t.kind) {
                painting_arm(world, i, j);
            }
            // `:22376`: `WorldGen.TileFrame(i, j)` — the framing deferral.
        }
    }
    // `:22379-22422`: the 668 dirt-throw scatter.
    let mut num14 = 0u32;
    let mut num15 = match get_world_size(world.width) {
        1 => 6,
        2 => 9,
        _ => 3,
    };
    if anniversary {
        num15 *= 5;
    }
    let min_value = 50;
    let max_value = world.width - 50;
    let max_value2 = world.height - 200;
    let mut num17 = 3000;
    // The sub-vanilla guard: the C# `Next(50, maxX-50)` / `Next(50, maxY-200)`
    // bands collapse on a tiny world (the 512 pin's world keeps its bytes).
    if max_value <= min_value || max_value2 <= min_value {
        return num14;
    }
    while num14 < num15 as u32 {
        num17 -= 1;
        if num17 <= 0 {
            break;
        }
        let num18 = rng.range(min_value, max_value);
        let num19 = rng.range(min_value, max_value2);
        let Some(t) = world.tile(num18, num19).copied() else {
            continue;
        };
        if t.active() && (t.kind as usize) < td::TILE_COUNT {
            let mut flag2 = td::SET_DIRT.has(t.kind);
            if not_the_bees {
                flag2 = flag2 || td::SET_MUD.has(t.kind);
            }
            if flag2 {
                num14 += 1;
                if let Some(cell) = world.tile_mut(num18, num19) {
                    // `Tile.ClearTile` (`Tile.cs:123-128`): ClearSlope +
                    // active(false) + inActive(false) — the kind survives.
                    cell.set_slope(0);
                    cell.flags &= !FLAG_HALF_BRICK;
                    cell.flags &= !FLAG_ACTIVE;
                    cell.flags &= !FLAG_INACTIVE;
                    cell.flags |= FLAG_ACTIVE;
                    cell.kind = 668;
                }
            }
        }
    }
    // `:22423-22488`: the seed tails / shimmer / liquid-merge / town-NPC /
    // angler arms — deferred (module doc). `:22489`: `Lang.gen[87]` — the
    // world-tail step already carries that message row.
    num14
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::Tile;

    fn flat(w: i32, h: i32) -> World {
        World::new("fc".into(), w, h)
    }

    fn set(world: &mut World, x: i32, y: i32, kind: u16, active: bool) {
        let i = world.index(x, y);
        world.tiles[i].kind = kind;
        if active {
            world.tiles[i].flags |= FLAG_ACTIVE;
        }
    }

    /// The wall-hole fill: a wall-2 column band with a 3-wide air pocket
    /// inside the surface region takes the majority wall; the pocket's
    /// walled rim votes, the wall-0 interior fills.
    #[test]
    fn w4_g37_fc_the_wall_hole_fill_takes_the_majority_wall() {
        let mut w = flat(60, 60);
        // Wall 2 everywhere in rows 5..25, then carve a wall-0 pocket at
        // (20..23, 10..13) surrounded by active stone so the BFS stays small.
        for x in 2..58 {
            for y in 2..28 {
                let i = w.index(x, y);
                w.tiles[i].wall = 2;
            }
        }
        for x in 20..23 {
            for y in 10..13 {
                let i = w.index(x, y);
                w.tiles[i].wall = 0;
            }
        }
        fill_wall_holes_in_area(&mut w, 30.0);
        for x in 20..23 {
            for y in 10..13 {
                assert_eq!(w.tile(x, y).unwrap().wall, 2, "the hole filled ({x},{y})");
            }
        }
    }

    /// The falling-sand arm: a sand cell over 12 air rows drops 10 and
    /// hardens to 397 at the exhaustion row; the source column de-slopes.
    #[test]
    fn w4_g37_fc_the_sand_column_drops_and_hardens() {
        // x = 500 sits outside the ±380 oceanDepths margins on a 1000-wide
        // world (the drop gate's `!oceanDepths` arm).
        let mut w = flat(1000, 600);
        set(&mut w, 500, 100, 53, true);
        let mut rng = Rng(42);
        pass_final_cleanup(&mut w, 200.0, 400.0, false, false, false, false, false, &mut rng);
        // The drop starts at 101 and writes 10 cells (101..110); the seat
        // at 111 hardens to 397 on exhaustion. The column pass re-visits
        // its own writes top-down (the C# does too), but each dropped 53
        // cell now sits over an active 53 — a SOLID floor for the fc
        // predicate — so the outer `!solid_tile_fc(i, j+1)` gate refuses
        // the arm and the drop rows keep their 53. Only the exhaustion
        // seat is 397; the source cell at 100 keeps its 53 (the drop
        // branch never retypes the source — the C# quirk, kept).
        assert_eq!(w.tile(500, 100).unwrap().kind, 53, "the source stays sand");
        for y in 101..111 {
            assert_eq!(w.tile(500, y).unwrap().kind, 53, "the drop row {y}");
            assert!(w.tile(500, y).unwrap().active());
        }
        assert_eq!(w.tile(500, 111).unwrap().kind, 397, "the hardened seat");
        assert!(w.tile(500, 111).unwrap().active());
    }

    /// The boulder arm: a 138 boulder whose top-left frame origin sits under
    /// a 26 altar collapses to stone 0; without the altar it re-seats.
    #[test]
    fn w4_g37_fc_the_boulder_collapses_under_an_altar() {
        let mut w = flat(600, 600);
        // The boulder body at (300..302, 450..452), anchor cell (301, 451)
        // with frameX/frameY 18 → origin (300, 450). Below the dirt-throw
        // scatter's y band ([50, height-200) = [50, 400)) so the collapsed
        // active kind-0 cells (SET_DIRT) cannot be re-typed to 668.
        for x in 300..302 {
            for y in 450..452 {
                set(&mut w, x, y, 138, true);
            }
        }
        let i = w.index(301, 451);
        w.tiles[i].frame_x = 18;
        w.tiles[i].frame_y = 18;
        set(&mut w, 300, 449, 26, true); // the altar above the left column
        let mut rng = Rng(42);
        pass_final_cleanup(&mut w, 200.0, 400.0, false, false, false, false, false, &mut rng);
        for x in 300..302 {
            for y in 450..452 {
                let t = w.tile(x, y).unwrap();
                assert_eq!(t.kind, 0, "the collapsed body ({x},{y})");
                assert!(t.active());
                assert_eq!(t.frame_x, 0);
                assert_eq!(t.frame_y, 0);
            }
        }
    }

    /// The dungeon-wall arm: a 56 obsidian over a dungeon wall dies into
    /// water-255; a 374 drip retypes to 373.
    #[test]
    fn w4_g37_fc_the_dungeon_wall_liquid_reset() {
        let mut w = flat(600, 600);
        set(&mut w, 300, 300, 56, true);
        let i = w.index(300, 300);
        w.tiles[i].wall = 7;
        set(&mut w, 310, 300, 374, true);
        let i = w.index(310, 300);
        w.tiles[i].wall = 94;
        w.tiles[i].set_liquid_kind(LiquidKind::Lava);
        let mut rng = Rng(42);
        pass_final_cleanup(&mut w, 200.0, 400.0, false, false, false, false, false, &mut rng);
        let t = w.tile(300, 300).unwrap();
        assert!(!t.active(), "the 56 died");
        assert_eq!(t.liquid, 255);
        assert_eq!(t.liquid_kind(), LiquidKind::Water);
        let t = w.tile(310, 300).unwrap();
        assert_eq!(t.kind, 373, "the drip retyped");
        assert_eq!(t.liquid_kind(), LiquidKind::Water);
    }

    /// The dirt-throw scatter: two same-stream runs replay byte-equal
    /// (a == b) and the 668 throws land on dirt only.
    #[test]
    fn w4_g37_fc_the_dirt_throw_scatter_replays() {
        let build = || {
            let mut w = flat(1000, 600);
            for x in 0..w.width {
                for y in 250..w.height {
                    let i = w.index(x, y);
                    w.tiles[i] = Tile {
                        kind: 0,
                        flags: FLAG_ACTIVE,
                        ..Default::default()
                    };
                }
            }
            w
        };
        let mut a = build();
        let mut b = build();
        let na = pass_final_cleanup(&mut a, 200.0, 400.0, false, false, false, false, false, &mut Rng(42));
        let nb = pass_final_cleanup(&mut b, 200.0, 400.0, false, false, false, false, false, &mut Rng(42));
        assert_eq!(a.tiles, b.tiles, "a == b: the pass stream replays");
        assert_eq!(na, nb);
        assert_eq!(na, 3, "the small-world count is 3");
        assert!(
            a.tiles.iter().any(|t| t.active() && t.kind == 668),
            "the 668 throws exist"
        );
    }
}
