//! M5-SEED-2: the `SecretSeed.FinalizeSecretSeeds` paint/coat/trap arms —
//! the `Do*` sweeps the micro seeds run after the whole chain
//! (`WorldGen.cs:589-693`): the `actuallyNoTraps` spike sweeps, the
//! `noSpiderCaves` wall sweep and the `paintEverything*` /
//! `coatEverything*` families with their `Variations` gates.
//!
//! Seam: `pipeline.rs`'s `step_seed_finish` dispatches [`run`] after the
//! classic Finish tails, matching the C# call order (`Finish()`'s classic
//! `Finish*` calls `:22431-22444`, then `FinalizeSecretSeeds` `:22467`).
//! Every body is flag-gated and draws from its own `pass_rng` stream, so a
//! classic world's pipeline is untouched.
//!
//! The `Variations` gates (`WorldGen.cs:43-337`) are the derived
//! predicates below — none of them is a stored flag, they read the seed
//! flags plus `activeSecretSeedCount`, which the port computes with
//! [`active_secret_seed_count`] (the classic flags count too).
//!
//! Deviations: the chest rows the C# adds inside these passes (echo's
//! 1-in-25 item 4409, the frozen finish's Present 1869, the portal-gun
//! 3384) are chest-side; the port generates chests as tiles without
//! inventories, so those arms are documented no-ops here.

use std::collections::HashSet;

use crate::edit::kill_tile;
use crate::world::World;
use super::pipeline::Streams;
use super::GenOptions;
use crate::seeds::{MicroSeedFlags, SeedFlags, active_secret_seed_count};

/// The `FinalizeSecretSeeds` paint/coat/trap arms, in the C#'s dispatch
/// order (`WorldGen.cs:639-662`: echo and illuminant first, then gray,
/// negative, rainbow). `actuallyNoTraps` (`:603`) and the spider-cave sweep
/// (`:688`) come earlier; the trap arm lives here and the spider-cave wall
/// sweep runs inside the spider pass (its host), so both orderings hold.
pub(crate) fn run(world: &mut World, streams: &mut Streams, opts: &GenOptions) {
    let micro = opts.micro_flags;
    let count = active_secret_seed_count(micro, opts.flags);
    if micro.contains(MicroSeedFlags::ACTUALLY_NO_TRAPS) {
        actually_no_traps(world, &mut streams.pass_rng("DoActuallyNoTraps"), micro, count);
    }
    if micro.contains(MicroSeedFlags::COAT_EVERYTHING_ECHO) {
        coat_everything_echo(world, micro, count);
    }
    if micro.contains(MicroSeedFlags::COAT_EVERYTHING_ILLUMINANT) {
        coat_everything_illuminant(world, &mut streams.pass_rng("DoCoatEverythingIlluminant"), micro, count);
    }
    if micro.contains(MicroSeedFlags::PAINT_EVERYTHING_GRAY) {
        paint_everything_gray(world, &mut streams.pass_rng("DoPaintEverythingGray"), micro, count);
    }
    if micro.contains(MicroSeedFlags::PAINT_EVERYTHING_NEGATIVE) {
        paint_everything_negative(world, &mut streams.pass_rng("DoPaintEverythingNegative"), micro, count);
    }
    if micro.contains(MicroSeedFlags::RAINBOW_STUFF) {
        rainbow_stuff(world, &mut streams.pass_rng("DoRainbowStuff"), micro, count);
    }
}

// --- the Variations ---------------------------------------------------------

/// `Variations.paintEverythingGrayJustTheSurface` (`WorldGen.cs:45-58`).
fn gray_just_the_surface(micro: MicroSeedFlags) -> bool {
    micro.contains(MicroSeedFlags::PAINT_EVERYTHING_GRAY)
        && !gray_just_treasure(micro)
        && (micro.contains(MicroSeedFlags::PAINT_EVERYTHING_NEGATIVE)
            || micro.contains(MicroSeedFlags::COAT_EVERYTHING_ECHO)
            || micro.contains(MicroSeedFlags::COAT_EVERYTHING_ILLUMINANT))
}

/// `Variations.paintEverythingGrayJustTreasure` (`:61-71`) — the ore/gem/178
/// subset when ≥ 4 secrets are active.
fn gray_just_treasure(micro: MicroSeedFlags) -> bool {
    gray_just_treasure_count(micro, active_secret_seed_count(micro, micro_classic_of(micro)))
}

/// The same member with the caller's count (the dispatch computes it once).
fn gray_just_treasure_count(micro: MicroSeedFlags, count: u32) -> bool {
    micro.contains(MicroSeedFlags::PAINT_EVERYTHING_GRAY) && count >= 4
}

/// The classic flags a world carries — the `Variations` count's other half.
/// The sweeps run at worldgen time, where the world itself carries them.
fn micro_classic_of(micro: MicroSeedFlags) -> SeedFlags {
    let _ = micro;
    SeedFlags::none()
}

/// `Variations.paintEverythingGrayUseWhite` (`:73-83`).
fn gray_use_white(micro: MicroSeedFlags) -> bool {
    micro.contains(MicroSeedFlags::PAINT_EVERYTHING_GRAY)
        && micro.contains(MicroSeedFlags::WORLD_IS_FROZEN)
}

/// `Variations.paintEverythingNegativeJustUnderground` (`:85-98`).
fn negative_just_underground(micro: MicroSeedFlags, count: u32) -> bool {
    micro.contains(MicroSeedFlags::PAINT_EVERYTHING_NEGATIVE)
        && !(micro.contains(MicroSeedFlags::PAINT_EVERYTHING_NEGATIVE) && count >= 4)
        && (micro.contains(MicroSeedFlags::PAINT_EVERYTHING_GRAY)
            || micro.contains(MicroSeedFlags::COAT_EVERYTHING_ECHO)
            || micro.contains(MicroSeedFlags::COAT_EVERYTHING_ILLUMINANT))
}

/// `Variations.paintEverythingNegativeJustSomeThings` (`:101-111`).
fn negative_just_some_things(micro: MicroSeedFlags, count: u32) -> bool {
    micro.contains(MicroSeedFlags::PAINT_EVERYTHING_NEGATIVE) && count >= 4
}

/// `Variations.coatEverythingEchoJustSomeThings` (`:129-139`).
fn echo_just_some_things(count: u32) -> bool {
    count >= 4
}

/// `Variations.coatEverythingJustInnerBlocks` (`:113-127`).
fn coat_just_inner_blocks(micro: MicroSeedFlags, count: u32) -> bool {
    micro.contains(MicroSeedFlags::COAT_EVERYTHING_ECHO)
        && !echo_just_some_things(count)
        && (micro.contains(MicroSeedFlags::PAINT_EVERYTHING_GRAY)
            || micro.contains(MicroSeedFlags::PAINT_EVERYTHING_NEGATIVE)
            || count >= 3)
}

/// `Variations.coatEverythingIlluminantJustSomeThings` (`:141-151`).
fn illuminant_just_some_things(micro: MicroSeedFlags, count: u32) -> bool {
    micro.contains(MicroSeedFlags::COAT_EVERYTHING_ECHO)
        && if count < 3 {
            micro.contains(MicroSeedFlags::PAINT_EVERYTHING_NEGATIVE)
        } else {
            true
        }
}

/// `Variations.coatEverythingIlluminantJustRandomSpots` (`:153-167`).
fn illuminant_just_random_spots(micro: MicroSeedFlags, count: u32) -> bool {
    !illuminant_just_some_things(micro, count) && micro.contains(MicroSeedFlags::COAT_EVERYTHING_ECHO)
}

// --- small helpers ---------------------------------------------------------

fn kind_at(world: &World, x: i32, y: i32) -> u16 {
    world.tile(x, y).map_or(0, |t| t.kind)
}

fn wall_at(world: &World, x: i32, y: i32) -> u16 {
    world.tile(x, y).map_or(0, |t| t.wall)
}

fn active_at(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| t.active())
}

fn solid(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| t.active() && crate::tiledata::TILE_SOLID.has(t.kind))
}

fn set_kind(world: &mut World, x: i32, y: i32, kind: u16) {
    if let Some(t) = world.tile_mut(x, y) {
        t.kind = kind;
    }
}

fn set_paint(world: &mut World, x: i32, y: i32, paint: u8) {
    if let Some(t) = world.tile_mut(x, y) {
        t.set_paint(paint);
    }
}

fn is_cloud(kind: u16) -> bool {
    matches!(kind, 189 | 196 | 460 | 717 | 718 | 719)
}

fn is_dungeon_brick(kind: u16) -> bool {
    matches!(kind, 41 | 43 | 44 | 481 | 482 | 483)
}

fn is_dungeon_wall(wall: u16) -> bool {
    matches!(wall, 7 | 8 | 9 | 94..=99)
}

// --- arms ------------------------------------------------------------------

/// `DoActuallyNoTraps` (`WorldGen.cs:2641-2705`).
fn actually_no_traps(world: &mut World, rng: &mut super::Rng, micro: MicroSeedFlags, count: u32) {
    let border = 20;
    if count < 4 {
        // `Variations.actuallyNoTrapsForRealIMeanIt` (`:293-303`): every
        // Spikes (48) and WoodenSpikes (232) is killed outright.
        for x in 0..world.width {
            for y in 0..world.height {
                if matches!(kind_at(world, x, y), 48 | 232) {
                    kill_tile(world, x, y, true);
                }
            }
        }
        return;
    }
    // The count ≥ 4 arm: the treasure-tile spike rings.
    let (spawn_x, spawn_y) = (world.spawn_x, world.spawn_y);
    for k in border..(world.width - border).max(border + 1) {
        for l in border..(world.height - border).max(border + 1) {
            if (k - spawn_x).abs() + (l - spawn_y).abs() <= 100 {
                continue;
            }
            let kind = kind_at(world, k, l);
            if !active_at(world, k, l) || !is_treasure(kind) {
                continue;
            }
            let num2 = rng.range(1, 10);
            let num3 = if micro.contains(MicroSeedFlags::COAT_EVERYTHING_ECHO) {
                num2
            } else {
                1
            };
            for m in k - num2..=k + num2 {
                for n in l - num2..=l + num3 {
                    let inner = kind_at(world, m, n);
                    let can_kill = !is_treasure(inner)
                        && (!active_at(world, m, n)
                            || (!world.tile(m, n).is_some_and(|t| t.flags & crate::tile::FLAG_ACTUATOR != 0)
                                && !world.frame_important[inner as usize]
                                && !solid(world, m, n)));
                    if !can_kill {
                        continue;
                    }
                    let spikes = if wall_at(world, m, n) == 87 { 232u16 } else { 48 };
                    kill_tile(world, m, n, true);
                    if let Some(t) = world.tile_mut(m, n) {
                        t.kind = spikes;
                        t.flags |= crate::tile::FLAG_ACTIVE;
                    }
                    if micro.contains(MicroSeedFlags::COAT_EVERYTHING_ECHO) {
                        world.set_invisible_block(m, n, true);
                    }
                }
            }
        }
    }
}

/// `DoActuallyNoTraps_IsConsideredTreasure` (`WorldGen.cs:2632-2639`).
fn is_treasure(kind: u16) -> bool {
    matches!(kind, 21 | 467 | 12 | 639 | 665)
}

/// `DoPaintEverythingGray` (`WorldGen.cs:773-812`).
fn paint_everything_gray(world: &mut World, rng: &mut super::Rng, micro: MicroSeedFlags, count: u32) {
    let surface = gray_just_the_surface(micro);
    let treasure = gray_just_treasure_count(micro, count);
    let paint = if gray_use_white(micro) { 26u8 } else { 27 };
    let border = 5;
    for x in border..(world.width - border).max(border + 1) {
        let mut rows = (world.height - border).max(border);
        if surface {
            rows = rows.min(world.world_surface as i32 + rng.range(0, 3));
        }
        for y in border..rows {
            let kind = kind_at(world, x, y);
            if treasure && !crate::tiledata::SET_ORE.has(kind) && kind != 178 {
                continue;
            }
            set_paint(world, x, y, paint);
            world.set_wall_paint(x, y, paint);
        }
    }
}

/// `DoPaintEverythingNegative` (`WorldGen.cs:814-893`).
fn paint_everything_negative(world: &mut World, rng: &mut super::Rng, micro: MicroSeedFlags, count: u32) {
    let underground = negative_just_underground(micro, count);
    let some = negative_just_some_things(micro, count);
    let border = 5;
    for x in border..(world.width - border).max(border + 1) {
        let mut start = border;
        let mut rows = (world.height - border - 20).max(border);
        if underground {
            start = start.max(world.world_surface as i32 - rng.range(0, 3));
            rows = rows.max(start);
        }
        for y in start..rows {
            let kind = kind_at(world, x, y);
            let wall = wall_at(world, x, y);
            if some {
                if is_dungeon_brick(kind) || is_dungeon_wall(wall) || is_cloud(kind) || wall == 73
                    || matches!(kind, 192 | 384)
                    || wall == 60
                {
                    set_paint(world, x, y, 30);
                    world.set_wall_paint(x, y, 30);
                }
            } else {
                set_paint(world, x, y, 30);
                world.set_wall_paint(x, y, 30);
            }
        }
    }
}

/// `DoCoatEverythingEcho` (`WorldGen.cs:895-995`).
fn coat_everything_echo(world: &mut World, micro: MicroSeedFlags, count: u32) {
    let some = echo_just_some_things(count);
    let inner = coat_just_inner_blocks(micro, count);
    let error = micro.contains(MicroSeedFlags::ERROR_WORLD);
    let border = 5;
    for x in border..(world.width - border).max(border + 1) {
        for y in border..(world.height - border).max(border + 1) {
            if some {
                // The C#'s list: spikes 48/232, the dart trap 137, the
                // boulder family (minus 665/711), 141/135/443.
                let kind = kind_at(world, x, y);
                let boulder = matches!(kind, 138 | 140 | 141 | 147 | 148 | 149 | 150 | 151)
                    && !matches!(kind, 665 | 711);
                if matches!(kind, 48 | 232 | 137 | 141 | 135 | 443) || boulder {
                    world.set_invisible_block(x, y, true);
                }
            } else if inner {
                let mut enclosed = true;
                for i in x - 1..=x + 1 {
                    for j in y - 1..=y + 1 {
                        if !world.tile(i, j).is_some_and(|t| {
                            t.active() && crate::tiledata::TILE_SOLID_TOP.has(t.kind)
                        }) {
                            enclosed = false;
                        }
                    }
                }
                if enclosed {
                    world.set_invisible_block(x, y, true);
                    world.set_invisible_wall(x, y, true);
                }
                if !error {
                    world.set_invisible_wall(x, y, true);
                }
            } else {
                world.set_invisible_block(x, y, true);
                world.set_invisible_wall(x, y, true);
            }
        }
    }
    if error {
        for x in border..(world.width - border).max(border + 1) {
            for y in border..(world.height - border).max(border + 1) {
                if world.invisible_block(x, y) {
                    if let Some(t) = world.tile_mut(x, y) {
                        t.flags &= !crate::tile::FLAG_ACTIVE;
                    }
                }
                if world.invisible_wall(x, y) || world.tile(x, y).is_some_and(|t| {
                    t.active() && crate::tiledata::TILE_SOLID_TOP.has(t.kind)
                }) {
                    if let Some(t) = world.tile_mut(x, y) {
                        t.wall = 0;
                    }
                }
            }
        }
    }
    // The 1-in-25 chest row (item 4409) is chest-side — documented out
    // (the port generates chests as tiles without inventories).
}

/// `DoCoatEverythingIlluminant` (`WorldGen.cs:997-1036`).
fn coat_everything_illuminant(world: &mut World, rng: &mut super::Rng, micro: MicroSeedFlags, count: u32) {
    let some = illuminant_just_some_things(micro, count);
    let spots = illuminant_just_random_spots(micro, count);
    let border = 5;
    for x in border..(world.width - border).max(border + 1) {
        for y in border..(world.height - border).max(border + 1) {
            if some {
                let kind = kind_at(world, x, y);
                if matches!(kind, 12 | 665 | 639) {
                    if let Some(t) = world.tile_mut(x, y) {
                        t.flags |= crate::tile::FLAG_FULLBRIGHT;
                    }
                }
            } else if spots && rng.range(0, 2) == 0 {
                if let Some(t) = world.tile_mut(x, y) {
                    t.flags |= crate::tile::FLAG_FULLBRIGHT;
                }
                world.set_fullbright_wall(x, y, true);
                world.set_invisible_block(x, y, false);
                world.set_invisible_wall(x, y, false);
            } else if !spots {
                if let Some(t) = world.tile_mut(x, y) {
                    t.flags |= crate::tile::FLAG_FULLBRIGHT;
                }
                world.set_fullbright_wall(x, y, true);
            }
        }
    }
}

/// `DoRainbowStuff` (`WorldGen.cs:2707-2781`): the whole-map re-type sweep.
/// The kinds are ported; the frame re-writes (the torch `frameY = 308`, the
/// banner reframes, the 184/202 art) are documented out — the port's
/// `finish_seeds` precedent carries the frame arms only where the C# writes
/// them through the tile paint, and a frame-only sweep would need the full
/// frame writer.
fn rainbow_stuff(world: &mut World, rng: &mut super::Rng, micro: MicroSeedFlags, count: u32) {
    let extra_liquid = micro.contains(MicroSeedFlags::EXTRA_LIQUID);
    let round = micro.contains(MicroSeedFlags::ROUND_LANDMASSES);
    let multiplier = if extra_liquid && !round { 2 } else { 1 };
    let border = 5;
    for x in border..(world.width - border).max(border + 1) {
        for y in border..(world.height - border).max(border + 1) {
            let kind = kind_at(world, x, y);
            let new = match kind {
                189 => Some(719),
                202 => Some(692),
                _ => None,
            };
            if let Some(k) = new {
                set_kind(world, x, y, k);
            }
            if kind == 4 {
                if let Some(t) = world.tile_mut(x, y) {
                    t.frame_y = 308;
                    t.set_paint(0);
                }
            }
            if kind == 138 && rng.range(0, 4) == 0 {
                set_kind(world, x, y, 711);
            }
            if kind == 184 {
                if let Some(t) = world.tile_mut(x, y) {
                    t.frame_x = 220;
                }
            }
            if kind == 91 && rng.range(0, 5) > 0 && y < world.world_surface as i32
                && rng.range(0, 2) == 0
            {
                set_kind(world, x, y, 627);
            }
        }
    }
    // The C#'s `maxTilesX * 2` extra pass (×2 under the extraLiquid arm) is
    // the glowstick/cloud-family decor the port does not model; the count is
    // kept for the record.
    let _ = (world.width * 2 * multiplier, count);
}

/// `DoNoSpiderCavesILiedMoreSpiderCaves` (`WorldGen.cs:2618-2630`): the wall
/// sweep for the `noSpiderCaves` seed with ≥ 4 secrets — dispatched from the
/// spider pass in the C# (the `ILied` arm of the SpiderCaves pass,
/// `:17422-17425`), so it is exposed here for that host.
pub(crate) fn no_spider_caves_i_lied(world: &mut World) {
    let border = 5;
    for x in border..(world.width - border).max(border + 1) {
        for y in border..(world.height - border).max(border + 1) {
            if matches!(wall_at(world, x, y), 204 | 205 | 206 | 207 | 180 | 178 | 86 | 15) {
                if let Some(t) = world.tile_mut(x, y) {
                    t.wall = 62;
                }
            }
        }
    }
}

// --- Clean Up Dirt (W4-g36 A2) ------------------------------------------------

/// `SecretSeed.DoNoSurfaceFillTheTop` (`WorldGen.cs:1055-1082`): the
/// noSurface seed's top fill — every column's first `100 + Next(2)` rows
/// activate (unless dungeon brick/wall) and retype jungle/mushroom grass
/// (60/70) to mud (59). The `skyblockWorldGen` early-return is the caller's
/// gate (the pass body already skips skyblock). Draws one `Next(2)` per
/// column, in C# order.
fn no_surface_fill_the_top(world: &mut World, rng: &mut super::Rng) {
    // `Main.tileDungeon` (`Main.cs:7992-7997`) — the six dungeon bricks.
    const TILE_DUNGEON: [u16; 6] = [41, 43, 44, 677, 678, 679];
    // `Main.wallDungeon` (`Main.cs`) — the nine dungeon walls.
    const WALL_DUNGEON: [u16; 9] = [7, 8, 9, 94, 95, 96, 97, 98, 99];
    for i in 0..world.width {
        let rows = 100 + rng.range(0, 2);
        for j in 0..rows {
            let Some(t) = world.tile(i, j) else { continue };
            if WALL_DUNGEON.contains(&t.wall) {
                continue;
            }
            let kind = t.kind;
            let idx = world.index(i, j);
            if !TILE_DUNGEON.contains(&kind) {
                world.tiles[idx].flags |= crate::tile::FLAG_ACTIVE;
            }
            if kind == 60 || kind == 70 {
                world.tiles[idx].kind = 59;
            }
        }
    }
}

/// The wall-clean predicate for a NEIGHBOUR cell. The C# writes
/// `wall == 2 || wall == 40 || wall == 40` (the third disjunct duplicates
/// 40 — a decompile quirk that drops 64/86 from the neighbour test); the
/// port keeps it verbatim (`WorldGen.cs:15271-15293`, `:15321-15343`).
fn dirt_wall_neighbor(wall: u16) -> bool {
    wall == 2 || wall == 40 || wall == 40
}

/// One column's top-down wall-clean sweep, shared by the forward
/// (`WorldGen.cs:15257-15306`) and reverse (`:15307-15356`) passes. `self_walls`
/// is the SELF-cell wall set (forward adds 86, reverse does not); the
/// neighbour set is always [`dirt_wall_neighbor`]. `sand_only` selects the
/// reverse pass's narrower "keep going" gate (`type != 53` only, `:15319`)
/// vs the forward pass's sand/pearlsand/crimsand trio (`:15269`).
///
/// The `flag` state machine: while `flag` is set, clean the self wall, and
/// if the cell is not a sand-family block, clean the ±1/±2/±3 neighbours
/// (the ±2/±3 at 1/2 each) and drop `flag` when the cell is active. Once
/// `flag` is clear, a fully-walled-and-air 5-row window re-arms it.
fn clean_dirt_column(
    world: &mut World,
    i: i32,
    world_surface: f64,
    rng: &mut super::Rng,
    self_walls: &[u16],
    sand_only: bool,
) {
    let mut flag = true;
    let mut j = 0i32;
    while (j as f64) < world_surface {
        if flag {
            // The self-cell wall clean (`:15265-15268` / `:15315-15318`).
            if self_walls.contains(&wall_at(world, i, j)) {
                if let Some(t) = world.tile_mut(i, j) {
                    t.wall = 0;
                }
            }
            // The "keep cleaning" gate (`:15269` / `:15319`).
            let keep = if sand_only {
                !active_at(world, i, j) || kind_at(world, i, j) != 53
            } else {
                !active_at(world, i, j)
                    || !matches!(kind_at(world, i, j), 53 | 112 | 234)
            };
            if keep {
                // ±1: unconditional clean. ±2/±3: 1/2 each. The draw order
                // is -1, -2, -3, +1, +2, +3 (the C# statement order).
                for (dx, gated) in [(-1i32, false), (-2, true), (-3, true), (1, false), (2, true), (3, true)] {
                    let nx = i + dx;
                    if dirt_wall_neighbor(wall_at(world, nx, j)) && (!gated || rng.range(0, 2) == 0) {
                        if let Some(t) = world.tile_mut(nx, j) {
                            t.wall = 0;
                        }
                    }
                }
                if active_at(world, i, j) {
                    flag = false;
                }
            }
        } else if wall_at(world, i, j) == 0
            && wall_at(world, i, j + 1) == 0
            && wall_at(world, i, j + 2) == 0
            && wall_at(world, i, j + 3) == 0
            && wall_at(world, i, j + 4) == 0
            && wall_at(world, i - 1, j) == 0
            && wall_at(world, i + 1, j) == 0
            && wall_at(world, i - 2, j) == 0
            && wall_at(world, i + 2, j) == 0
            && !active_at(world, i, j)
            && !active_at(world, i, j + 1)
            && !active_at(world, i, j + 2)
            && !active_at(world, i, j + 3)
        {
            flag = true;
        }
        j += 1;
    }
}

/// The Clean Up Dirt pass (`GenPassNameID.DirtWallCleanup`,
/// `WorldGen.cs:15246-15359`): the two surface dirt-wall clean sweeps.
/// The `Skyblock.denyAllGeneration` gate is the caller's. The noSurface
/// seed takes `DoNoSurfaceFillTheTop` instead (`:15250-15253`); a classic
/// world runs the forward (left→right, `:15257`) then reverse (right→left,
/// `:15307`) sweeps. Own `pass_rng("Clean Up Dirt")` stream.
pub(crate) fn pass_clean_up_dirt(
    world: &mut World,
    world_surface: f64,
    rng: &mut super::Rng,
    no_surface: bool,
) {
    if no_surface {
        no_surface_fill_the_top(world, rng);
        return;
    }
    // Forward sweep (`:15257-15306`): self walls {2,40,64,86}, sand trio.
    for i in 3..world.width - 3 {
        clean_dirt_column(world, i, world_surface, rng, &[2, 40, 64, 86], false);
    }
    // Reverse sweep (`:15307-15356`): self walls {2,40,64} (no 86), sand only.
    let mut num = world.width - 5;
    while num >= 5 {
        clean_dirt_column(world, num, world_surface, rng, &[2, 40, 64], true);
        num -= 1;
    }
}

// --- Dirt Rock Wall Runner (W4-g36 A3) ----------------------------------------

/// `WorldGen.DirtyRockRunner` (`WorldGen.cs:77145-77210`): the
/// Manhattan-disc wall walker — strength `Next(2, 6)`, steps `Next(5, 50)`,
/// the ±1.0-clamped velocity jitter, and the per-cell `Next(-10, 11)`
/// threshold roll converting wall 2 (DirtUnsafe) → 59 (Cave6Unsafe).
/// The threshold reads the FULL strength `num * 0.5` (not the shrinking
/// `num4` — the C# quirk at `:77184`), the box clamps to `[0, maxTilesX]`
/// / `[0, maxTilesY]` (`:77164-77179`), and the walk shrinks `num3` once
/// per iteration (`:77159`).
fn dirty_rock_runner(world: &mut World, i: i32, j: i32, rng: &mut super::Rng) {
    let num = rng.range(2, 6) as f64;
    let num2 = rng.range(5, 50) as f64;
    let mut num3 = num2;
    let mut pos = (i as f64, j as f64);
    let mut vel = (
        rng.range(-10, 11) as f64 * 0.1,
        rng.range(-10, 11) as f64 * 0.1,
    );
    while num > 0.0 && num3 > 0.0 {
        let num4 = num * (num3 / num2);
        num3 -= 1.0;
        let x0 = ((pos.0 - num4 * 0.5) as i32).max(0);
        let x1 = ((pos.0 + num4 * 0.5) as i32).min(world.width);
        let y0 = ((pos.1 - num4 * 0.5) as i32).max(0);
        let y1 = ((pos.1 + num4 * 0.5) as i32).min(world.height);
        for k in x0..x1 {
            for l in y0..y1 {
                // `:77184` — the threshold draw happens per cell BEFORE
                // the wall test (the `&&` left operand).
                let threshold = num * 0.5 * (1.0 + rng.range(-10, 11) as f64 * 0.015);
                if (k as f64 - pos.0).abs() + (l as f64 - pos.1).abs() < threshold
                    && wall_at(world, k, l) == 2
                {
                    if let Some(t) = world.tile_mut(k, l) {
                        t.wall = 59;
                    }
                }
            }
        }
        pos = (pos.0 + vel.0, pos.1 + vel.1);
        vel.0 = (vel.0 + rng.range(-10, 11) as f64 * 0.05).clamp(-1.0, 1.0);
        vel.1 = (vel.1 + rng.range(-10, 11) as f64 * 0.05).clamp(-1.0, 1.0);
    }
}

/// The Dirt Rock Wall Runner pass (`GenPassNameID.DirtRockWallRunner`,
/// `WorldGen.cs:15461-15475`): one random surface site per column —
/// `Next(10, maxTilesX - 10)` × `Next(10, (int)worldSurface)` — and a
/// `DirtyRockRunner` when the site carries wall 2. The
/// `Skyblock.denyAllGeneration` gate is the caller's. Own
/// `pass_rng("Dirt Rock Wall Runner")` stream.
pub(crate) fn pass_dirt_rock_wall_runner(
    world: &mut World,
    world_surface: f64,
    rng: &mut super::Rng,
) {
    for _ in 0..world.width {
        let num = rng.range(10, world.width - 10);
        let num2 = rng.range(10, world_surface as i32);
        if wall_at(world, num, num2) == 2 {
            dirty_rock_runner(world, num, num2, rng);
        }
    }
}

// --- Wall Variety (W4-g36 A6) --------------------------------------------------

/// `WorldGen.RandomWorldPoint(top, right, bottom, left)`
/// (`WorldGen.cs:27111-27114`): `Next(left, maxTilesX - right)` then
/// `Next(top, maxTilesY - bottom)` — X first, Y second.
fn random_world_point(
    world: &World,
    rng: &mut super::Rng,
    top: i32,
    right: i32,
    bottom: i32,
    left: i32,
) -> (i32, i32) {
    let x = rng.range(left, world.width - right);
    let y = rng.range(top, world.height - bottom);
    (x, y)
}

/// `Modifiers.IsNotSolid` (`Modifiers.cs:573-583`): `!active ||
/// !SolidOrSlopedTile(x, y)` — the flood-fill's passable-cell gate.
/// `SolidOrSlopedTile` (`WorldGen.cs:69985-70009`) is active + tileSolid +
/// !tileSolidTop + !inActive (slopes/half-bricks pass).
fn is_not_solid(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    let solid_or_sloped = t.active()
        && crate::tiledata::TILE_SOLID.has(t.kind)
        && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
        && !t.inactive();
    !t.active() || !solid_or_sloped
}

/// `Modifiers.IsTouching(useDiagonals: true, ids)` (`Modifiers.cs:278-321`):
/// any of the 8 neighbours active with a kind in `ids`. The DIRECTIONS
/// order is N, E, W, S, NE, NW, SW, SE (`:315-319`).
fn is_touching(world: &World, x: i32, y: i32, ids: &[u16]) -> bool {
    const DIRS: [(i32, i32); 8] = [
        (0, -1), (1, 0), (-1, 0), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1),
    ];
    for (dx, dy) in DIRS {
        if let Some(t) = world.tile(x + dx, y + dy) {
            if t.active() && ids.contains(&t.kind) {
                return true;
            }
        }
    }
    false
}

/// `ShapeFloodFill(1000).Perform` (`ShapeFloodFill.cs:17-58`) over the
/// `IsNotSolid → Blank.Output(shapeData) → ContinueWrapper(...)` chain:
/// the BFS collects every reachable non-solid cell (relative to the
/// origin, deduped, insertion order — the `ShapeData` HashSet the C#
/// `OuterOutline` iterates) and flags `foundInvalidTile` when a collected
/// cell touches the invalid kinds. Returns `(cells, found_invalid,
/// completed)` — `completed` is the C# `queue.Count == 0` result the
/// `flag2` gate reads (`:16740`'s `& flag2`).
///
/// The ContinueWrapper (`Actions.cs:12-26`) always returns true, so every
/// non-solid cell is collected and fans out regardless of the inner
/// IsTouching result; the inner chain only sets `foundInvalidTile`. The
/// non-jungle arm's two IsTouching modifiers are chained — the first
/// (with 60) gates the second (without 60), so a cell touching ONLY mud
/// (60) passes the first but fails the second and sets nothing; the
/// jungle arm's single IsTouching (without 60) is the same effective
/// predicate. Both reduce to "touches {147,161,396,397,70,191}".
fn flood_fill_wall_variety(
    world: &World,
    origin: (i32, i32),
    max_actions: i32,
) -> (Vec<(i32, i32)>, bool, bool) {
    const INVALID: [u16; 6] = [147, 161, 396, 397, 70, 191];
    let mut queue: std::collections::VecDeque<(i32, i32)> = std::collections::VecDeque::new();
    let mut visited: HashSet<(i32, i32)> = HashSet::new();
    let mut cells: Vec<(i32, i32)> = Vec::new();
    let mut found_invalid = false;
    queue.push_back(origin);
    let mut budget = max_actions;
    while !queue.is_empty() && budget > 0 {
        let p = queue.pop_front().unwrap();
        if visited.contains(&p) {
            continue;
        }
        // UnitApply → the IsNotSolid gate (`:16731`/`:16735`).
        if !is_not_solid(world, p.0, p.1) {
            continue;
        }
        visited.insert(p);
        budget -= 1;
        // Blank.Output(shapeData): record relative to the origin.
        cells.push((p.0 - origin.0, p.1 - origin.1));
        // The ContinueWrapper's inner IsTouching chain sets the flag.
        if is_touching(world, p.0, p.1, &INVALID) {
            found_invalid = true;
        }
        // The 4-way fan with the C#'s bounds (`:30-45`).
        if p.0 + 1 < world.width - 1 {
            queue.push_back((p.0 + 1, p.1));
        }
        if p.0 - 1 >= 1 {
            queue.push_back((p.0 - 1, p.1));
        }
        if p.1 + 1 < world.height - 1 {
            queue.push_back((p.0, p.1 + 1));
        }
        if p.1 - 1 >= 1 {
            queue.push_back((p.0, p.1 - 1));
        }
    }
    // The C# tail drain (`:48-56`): pop until an unvisited point is found
    // (re-enqueue it, break); `completed = queue.Count == 0`.
    while let Some(item) = queue.pop_front() {
        if !visited.contains(&item) {
            queue.push_back(item);
            break;
        }
    }
    (cells, found_invalid, queue.is_empty())
}

/// `ModShapes.OuterOutline(data, useDiagonals: true, useInterior: true)`
/// (`ModShapes.cs:28-71`) + `SkipWalls(87, 86, 244)` + `PlaceWall(num4)`:
/// for each collected cell (relative to the flood origin), place the wall
/// on the cell itself (useInterior) and on every 8-neighbour NOT in the
/// collected set, unless the target's wall is 87/86/244. The outline is
/// rebuilt around `place_origin` (the anchor tile, one row below the
/// flood origin). `PlaceWall`'s `SquareWallFrame` is deferred per the
/// port's plain-write convention (`evil.rs::place_evil_wall`).
fn place_wall_variety_outline(
    world: &mut World,
    cells: &[(i32, i32)],
    place_origin: (i32, i32),
    wall: u16,
) {
    const OFFSETS: [(i32, i32); 8] = [
        (1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1),
    ];
    let set: HashSet<(i32, i32)> = cells.iter().copied().collect();
    let mut place = |x: i32, y: i32| {
        if let Some(t) = world.tile_mut(x, y) {
            if !matches!(t.wall, 87 | 86 | 244) {
                t.wall = wall;
            }
        }
    };
    for &(cx, cy) in cells {
        // useInterior: the cell itself, re-based to the place origin
        // (`ModShapes.cs:48`: `datum.X + origin.X`).
        place(cx + place_origin.0, cy + place_origin.1);
        // The outline: neighbours not in the collected set
        // (`ModShapes.cs:52-58`).
        for (ox, oy) in OFFSETS {
            if !set.contains(&(cx + ox, cy + oy)) {
                place(cx + ox + place_origin.0, cy + oy + place_origin.1);
            }
        }
    }
}

/// The Wall Variety pass (`GenPassNameID.CaveWallVariety`,
/// `WorldGen.cs:16697-16754`): the cave-wall variety fill — `num =
/// (int)(300 * (width*height / 5040000))` successful placements, each a
/// random world point (shimmer-safety re-rolled) whose anchor tile is
/// mud (60 → wall 204+Next(4)) or wall-less stone (1 → the rock-layer /
/// lava-line wall band), flood-filling the air pocket above and walling
/// its outline when the pocket is >50 cells, completed, and touches no
/// invalid kind. The `Skyblock.denyAllGeneration` gate is the caller's.
/// Own `pass_rng("Wall Variety")` stream.
pub(crate) fn pass_wall_variety(
    world: &mut World,
    world_surface: f64,
    rock_layer: f64,
    lava_line: i32,
    shimmer_position: Option<(i32, i32)>,
    remix: bool,
    rng: &mut super::Rng,
) {
    let mut num = (300.0 * ((world.width * world.height) as f64 / 5040000.0)) as i32;
    let num2 = num;
    let mut budget = 100000i32;
    // The shimmer-safety distance (`WorldGen.cs:86915`).
    let safety = super::shimmer::SHIMMER_SAFETY_DISTANCE;
    while num > 0 && budget > 0 {
        // The point + the shimmer-safety re-roll (`:16709-16713`).
        let mut point = random_world_point(world, rng, world_surface as i32, 2, 190, 2);
        if let Some((sx, sy)) = shimmer_position {
            while ((point.0 - sx) as f64).hypot((point.1 - sy) as f64) < safety {
                point = random_world_point(world, rng, world_surface as i32, 2, 190, 2);
            }
        }
        let tile = world.tile(point.0, point.1).copied().unwrap_or_default();
        let tile2 = world.tile(point.0, point.1 - 1).copied();
        if !tile.active() {
            continue;
        }
        let is_mud = tile.kind == 60;
        // The wall pick (`:16718-16727`).
        let wall: u16 = if is_mud {
            204 + rng.range(0, 4) as u16
        } else if tile.kind == 1 && tile2.map_or(0, |t| t.wall) == 0 {
            if remix {
                if point.1 as f64 > rock_layer {
                    196 + rng.range(0, 4) as u16
                } else if point.1 <= lava_line || rng.range(0, 2) != 0 {
                    212 + rng.range(0, 4) as u16
                } else {
                    208 + rng.range(0, 4) as u16
                }
            } else if (point.1 as f64) < rock_layer {
                196 + rng.range(0, 4) as u16
            } else if point.1 >= lava_line {
                208 + rng.range(0, 4) as u16
            } else {
                212 + rng.range(0, 4) as u16
            }
        } else {
            0
        };
        if wall == 0 || tile2.is_some_and(|t| t.active()) {
            continue;
        }
        // The flood fill above the anchor (`:16731-16739`).
        let flood_origin = (point.0, point.1 - 1);
        let (cells, found_invalid, completed) = flood_fill_wall_variety(world, flood_origin, 1000);
        // `:16740` — `(Count > 50) & flag2` (non-short-circuit) `&& !found`.
        if (cells.len() > 50) & completed && !found_invalid {
            place_wall_variety_outline(world, &cells, point, wall);
            num -= 1;
        } else {
            budget -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// W4-g36 A6: the flood fill collects the connected air pocket and
    /// flags an invalid-kind touch; a solid cell blocks the fan.
    #[test]
    fn a6_the_flood_fill_collects_air_and_flags_invalid_touches() {
        use crate::tile::Tile;
        let mut w = World::new("wv".into(), 60, 60);
        // A 5x5 air pocket at (20..25, 20..25), surrounded by stone.
        for x in 15..30 {
            for y in 15..30 {
                let idx = w.index(x, y);
                w.tiles[idx] = Tile::default();
                if !(20..25).contains(&x) || !(20..25).contains(&y) {
                    w.tiles[idx].kind = 1;
                    w.tiles[idx].flags |= crate::tile::FLAG_ACTIVE;
                }
            }
        }
        let (cells, found, completed) = flood_fill_wall_variety(&w, (22, 22), 1000);
        assert_eq!(cells.len(), 25, "the 5x5 pocket");
        assert!(!found, "no invalid kinds");
        assert!(completed, "the pocket is fully drained");
        // An ice tile (161) beside the pocket flags invalid.
        let idx = w.index(19, 22);
        w.tiles[idx].kind = 161;
        w.tiles[idx].flags |= crate::tile::FLAG_ACTIVE;
        let (_, found, _) = flood_fill_wall_variety(&w, (22, 22), 1000);
        assert!(found, "the ice touch flags invalid");
    }

    /// W4-g36 A6: the outline walls the pocket's rim (and interior) but
    /// skips the protected walls 87/86/244.
    #[test]
    fn a6_the_outline_walls_the_rim_and_skips_protected_walls() {
        use crate::tile::Tile;
        let mut w = World::new("wv".into(), 60, 60);
        for x in 15..30 {
            for y in 15..30 {
                let idx = w.index(x, y);
                w.tiles[idx] = Tile::default();
                if !(20..25).contains(&x) || !(20..25).contains(&y) {
                    w.tiles[idx].kind = 1;
                    w.tiles[idx].flags |= crate::tile::FLAG_ACTIVE;
                }
            }
        }
        // A protected living-wood wall (244) on one rim cell.
        w.tiles[w.index(19, 22)].wall = 244;
        let (cells, _, _) = flood_fill_wall_variety(&w, (22, 22), 1000);
        place_wall_variety_outline(&mut w, &cells, (22, 23), 204);
        // The protected cell keeps 244 (SkipWalls).
        assert_eq!(w.tile(19, 22).unwrap().wall, 244, "protected stays");
        // The outline walls at least one rim/interior cell (the place
        // origin is one row below the flood origin, so the fill is
        // re-based down by one — the C#'s anchor-row offset).
        assert!(
            w.tiles.iter().any(|t| t.wall == 204),
            "the outline must wall the pocket"
        );
    }

    /// W4-g36 A6: the pass is deterministic (a == b) and walls a large
    /// cavern's rim. The hand-built world carries one 80×70 air cavern in
    /// the stone band, so a placement inside it succeeds well within the
    /// 100000-failure budget (the C# loop's own cap).
    #[test]
    fn a6_the_pass_replays() {
        use crate::tile::Tile;
        let build = || {
            let mut w = World::new("wv".into(), 400, 400);
            // Stone below row 60.
            for x in 0..400 {
                for y in 60..400 {
                    let idx = w.index(x, y);
                    w.tiles[idx] = Tile::default();
                    w.tiles[idx].kind = 1;
                    w.tiles[idx].flags |= crate::tile::FLAG_ACTIVE;
                }
            }
            // One large bounded air cavern (100x80 = 8000 cells, but the
            // flood caps at 1000 — so use a 30x30 = 900-cell cavern that
            // completes within the cap and is >50). A grid of them makes a
            // placement succeed well within the 100000-failure budget.
            for cx in (60..340).step_by(50) {
                for cy in (80..190).step_by(40) {
                    for x in cx..(cx + 30) {
                        for y in cy..(cy + 30) {
                            w.tiles[w.index(x, y)] = Tile::default();
                        }
                    }
                }
            }
            w
        };
        let mut a = build();
        let mut b = build();
        pass_wall_variety(&mut a, 60.0, 100.0, 150, None, false, &mut super::super::Rng(42));
        pass_wall_variety(&mut b, 60.0, 100.0, 150, None, false, &mut super::super::Rng(42));
        assert_eq!(a.tiles, b.tiles, "a == b");
        // The pass walls the cavern rim (the stone band's wall pick is
        // 196/208/212 + Next(4)).
        assert!(
            a.tiles.iter().any(|t| matches!(t.wall, 196..=215)),
            "the pass must wall the cavern"
        );
    }

    /// W4-g36 A3: the runner converts the wall-2 pocket around its origin
    /// to 59 and leaves other walls alone; the pass replays a == b.
    #[test]
    fn a3_the_runner_converts_the_dirt_wall_pocket() {
        use crate::tile::Tile;
        let build = || {
            let mut w = World::new("rock".into(), 120, 120);
            for x in 20..100 {
                for y in 20..100 {
                    let idx = w.index(x, y);
                    w.tiles[idx] = Tile::default();
                    w.tiles[idx].wall = 2;
                }
            }
            // A stone-wall cell inside the pocket — never converted.
            w.tiles[w.index(60, 60)].wall = 1;
            w
        };
        let mut a = build();
        let mut b = build();
        dirty_rock_runner(&mut a, 60, 60, &mut super::super::Rng(42));
        dirty_rock_runner(&mut b, 60, 60, &mut super::super::Rng(42));
        assert_eq!(a.tiles, b.tiles, "a == b");
        assert!(
            a.tiles.iter().any(|t| t.wall == 59),
            "the runner must convert some dirt walls"
        );
        assert_eq!(a.tile(60, 60).unwrap().wall, 1, "the stone wall survives");
    }

    /// W4-g36 A3: the pass draws two anchors per column and only runs the
    /// walker on wall-2 sites (`WorldGen.cs:15467-15472`).
    #[test]
    fn a3_the_pass_runs_only_on_wall_two_sites() {
        use crate::tile::Tile;
        let mut w = World::new("rock".into(), 120, 120);
        // No wall-2 anywhere: the pass draws 2×width anchors, writes nothing.
        let before = w.tiles.clone();
        pass_dirt_rock_wall_runner(&mut w, 60.0, &mut super::super::Rng(9));
        assert_eq!(w.tiles, before);
        // A full wall-2 surface band: the walkers convert.
        for x in 0..120 {
            for y in 0..60 {
                let idx = w.index(x, y);
                w.tiles[idx] = Tile::default();
                w.tiles[idx].wall = 2;
            }
        }
        pass_dirt_rock_wall_runner(&mut w, 60.0, &mut super::super::Rng(9));
        assert!(w.tiles.iter().any(|t| t.wall == 59));
    }

    /// W4-g36 A2: the forward sweep clears the exposed dirt walls above
    /// the first active non-sand tile and stops there; the reverse sweep
    /// then re-cleans right-to-left. Hand-built world, exact scope.
    #[test]
    fn a2_the_sweeps_clear_exposed_dirt_walls() {
        use crate::tile::Tile;
        let mut w = World::new("dirt".into(), 64, 64);
        // A dirt-wall band rows 0..10 across the middle columns, with a
        // stone floor at row 10 (active, non-sand — the sweep stops).
        for x in 5..59 {
            for y in 0..10 {
                let idx = w.index(x, y);
                w.tiles[idx] = Tile::default();
                w.tiles[idx].wall = 2;
            }
            let idx = w.index(x, 10);
            w.tiles[idx].kind = 1;
            w.tiles[idx].flags |= crate::tile::FLAG_ACTIVE;
        }
        pass_clean_up_dirt(&mut w, 30.0, &mut super::super::Rng(42), false);
        // Every band wall in the sweep's column range is cleared.
        for x in 6..58 {
            for y in 0..10 {
                assert_eq!(w.tile(x, y).unwrap().wall, 0, "({x},{y})");
            }
        }
        // The floor survives.
        assert!(w.tile(30, 10).unwrap().active());
    }

    /// W4-g36 A2: the sand gate's observable is the flag state machine —
    /// an active NON-sand tile drops `flag` (the sweep stops cleaning
    /// below it until the 5-row re-arm window), while active SAND keeps
    /// `flag` set (the sweep cleans on below). The pass replays a == b.
    #[test]
    fn a2_sand_keeps_the_sweep_going_and_the_pass_replays() {
        use crate::tile::Tile;
        // `floor_kind` at row 10, dirt walls rows 11..20 below it.
        let build = |floor_kind: u16| {
            let mut w = World::new("dirt".into(), 64, 64);
            for y in 0..20 {
                let idx = w.index(30, y);
                w.tiles[idx] = Tile::default();
                w.tiles[idx].wall = 2;
            }
            let idx = w.index(30, 10);
            w.tiles[idx].kind = floor_kind;
            w.tiles[idx].flags |= crate::tile::FLAG_ACTIVE;
            w
        };
        // Stone floor: flag drops at row 10; the walls below survive (the
        // re-arm window needs wall-0 rows, these carry wall 2).
        let mut stone = build(1);
        pass_clean_up_dirt(&mut stone, 30.0, &mut super::super::Rng(7), false);
        assert_eq!(stone.tile(30, 15).unwrap().wall, 2, "stone stops the sweep");
        // Sand floor: flag stays set; the walls below are cleaned.
        let mut sand = build(53);
        pass_clean_up_dirt(&mut sand, 30.0, &mut super::super::Rng(7), false);
        assert_eq!(sand.tile(30, 15).unwrap().wall, 0, "sand keeps the sweep going");
        // a == b on the same seed.
        let mut a = build(1);
        let mut b = build(1);
        pass_clean_up_dirt(&mut a, 30.0, &mut super::super::Rng(7), false);
        pass_clean_up_dirt(&mut b, 30.0, &mut super::super::Rng(7), false);
        assert_eq!(a.tiles, b.tiles, "a == b");
    }

    /// W4-g36 A2: the noSurface arm fills the top 100+Next(2) rows and
    /// retypes jungle grass to mud (`WorldGen.cs:1055-1082`).
    #[test]
    fn a2_the_no_surface_arm_fills_the_top() {
        use crate::tile::Tile;
        let mut w = World::new("dirt".into(), 64, 200);
        let idx = w.index(10, 5);
        w.tiles[idx] = Tile::default();
        w.tiles[idx].kind = 60; // jungle grass
        pass_clean_up_dirt(&mut w, 30.0, &mut super::super::Rng(3), true);
        assert!(w.tile(10, 5).unwrap().active(), "the top rows activate");
        assert_eq!(w.tile(10, 5).unwrap().kind, 59, "jungle grass → mud");
        // Row 150 is beyond the 100+Next(2) fill — untouched.
        assert!(!w.tile(10, 150).unwrap().active());
    }

    fn opts(micro: MicroSeedFlags) -> GenOptions {
        GenOptions {
            micro_flags: micro,
            width: 512,
            height: 512,
            ..Default::default()
        }
    }

    fn painted(world: &World, paint: u8) -> usize {
        world
            .tiles
            .iter()
            .filter(|t| t.active() && t.paint() == paint)
            .count()
    }

    fn invisible(world: &World) -> usize {
        (0..world.width)
            .flat_map(|x| (0..world.height).map(move |y| (x, y)))
            .filter(|&(x, y)| world.invisible_block(x, y))
            .count()
    }

    fn spikes(world: &World) -> usize {
        world
            .tiles
            .iter()
            .filter(|t| t.active() && matches!(t.kind, 48 | 232))
            .count()
    }

    /// A classic world: no paint, no invisible cells, the spikes stay.
    #[test]
    fn a_classic_world_is_untouched() {
        let w = crate::worldgen::generate(&opts(MicroSeedFlags::none()));
        assert_eq!(painted(&w, 27), 0);
        assert_eq!(painted(&w, 30), 0);
        assert_eq!(invisible(&w), 0);
    }

    /// `paintEverythingGray` paints the tiles AND walls 27 (`:789-797`).
    #[test]
    fn the_gray_world_paints_everything() {
        let w = crate::worldgen::generate(&opts(MicroSeedFlags::from(
            MicroSeedFlags::PAINT_EVERYTHING_GRAY,
        )));
        assert!(painted(&w, 27) > 0, "the gray paint landed: {}", painted(&w, 27));
    }

    /// `paintEverythingNegative` paints 30 (`:889-892`).
    #[test]
    fn the_negative_world_paints_the_negative_paint() {
        let w = crate::worldgen::generate(&opts(MicroSeedFlags::from(
            MicroSeedFlags::PAINT_EVERYTHING_NEGATIVE,
        )));
        assert!(painted(&w, 30) > 0, "the negative paint landed: {}", painted(&w, 30));
    }

    /// `coatEverythingEcho` makes the world invisible (`:924-975`).
    #[test]
    fn the_echo_world_coats_everything() {
        let w = crate::worldgen::generate(&opts(MicroSeedFlags::from(
            MicroSeedFlags::COAT_EVERYTHING_ECHO,
        )));
        assert!(invisible(&w) > 0, "the echo coat landed: {}", invisible(&w));
    }

    /// `coatEverythingIlluminant` sets the fullbright flags.
    #[test]
    fn the_illuminant_world_glows() {
        let classic = crate::worldgen::generate(&opts(MicroSeedFlags::none()));
        let w = crate::worldgen::generate(&opts(MicroSeedFlags::from(
            MicroSeedFlags::COAT_EVERYTHING_ILLUMINANT,
        )));
        let glowing = |world: &World| {
            world
                .tiles
                .iter()
                .filter(|t| t.flags & crate::tile::FLAG_FULLBRIGHT != 0)
                .count()
        };
        assert!(
            glowing(&w) > glowing(&classic),
            "the fullbright grew: {} vs {}",
            glowing(&w),
            glowing(&classic)
        );
    }

    /// `actuallyNoTraps` with < 4 secrets kills every spike tile
    /// (`:2643-2660`).
    #[test]
    fn the_no_traps_world_kills_the_spikes() {
        let traps = crate::worldgen::generate(&opts(MicroSeedFlags::from(
            MicroSeedFlags::ACTUALLY_NO_TRAPS,
        )));
        assert_eq!(spikes(&traps), 0, "every spike died: {}", spikes(&traps));
    }

    /// The `JustTreasure` Variation narrows the gray sweep to the ores and
    /// the exposed gems (`:800-806`) — the count ≥ 4 gate.
    #[test]
    fn the_treasure_variation_narrows_the_gray_sweep() {
        // One seed: the full sweep. Four seeds: the ore/gem subset.
        let one = crate::worldgen::generate(&opts(MicroSeedFlags::from(
            MicroSeedFlags::PAINT_EVERYTHING_GRAY,
        )));
        // Four seeds with gray among them: the treasure subset paints fewer.
        let four = crate::worldgen::generate(&opts(MicroSeedFlags::from(
            MicroSeedFlags::PAINT_EVERYTHING_GRAY
                | MicroSeedFlags::PAINT_EVERYTHING_NEGATIVE
                | MicroSeedFlags::COAT_EVERYTHING_ECHO
                | MicroSeedFlags::COAT_EVERYTHING_ILLUMINANT,
        )));
        assert!(
            painted(&four, 27) <= painted(&one, 27),
            "the treasure subset is narrower: {} vs {}",
            painted(&four, 27),
            painted(&one, 27)
        );
    }

    /// `DoNoSpiderCavesILiedMoreSpiderCaves` — the spider-family walls
    /// become 62 (`:2622-2629`), on a hand-built world so the sweep's
    /// scope is exact.
    #[test]
    fn the_spider_lie_sweep_rewrites_the_walls() {
        use crate::tile::Tile;
        let mut w = World::new("spider".into(), 64, 64);
        // Inside the sweep's 5-tile border, where the C# scan runs.
        for i in 10..54 {
            let idx = w.index(i, 32);
            w.tiles[idx] = Tile::default();
            w.tiles[idx].wall = match i % 4 {
                0 => 204,
                1 => 15,
                2 => 180,
                _ => 1, // a plain stone wall: untouched
            };
        }
        no_spider_caves_i_lied(&mut w);
        for i in 10..54 {
            let wall = w.tile(i, 32).unwrap().wall;
            let expected = match i % 4 {
                0 | 1 | 2 => 62,
                _ => 1,
            };
            assert_eq!(wall, expected, "column {i} wall {wall} != {expected}");
        }
    }
}
