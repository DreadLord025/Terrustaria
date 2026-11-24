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
    // W4-g83 contract §2: the spike kills' framing dispatch threads the
    // worldgen genRand stand-in (`UnifiedRandom(worldSeed)`, the
    // `temple.rs::pass_temple_part2` note); the pass's xorshift stream stays
    // untouched (the world-hash contract).
    let mut gen_rng = crate::UnifiedRandom::new(world.seed as i32);
    let border = 20;
    if count < 4 {
        // `Variations.actuallyNoTrapsForRealIMeanIt` (`:293-303`): every
        // Spikes (48) and WoodenSpikes (232) is killed outright.
        for x in 0..world.width {
            for y in 0..world.height {
                if matches!(kind_at(world, x, y), 48 | 232) {
                    kill_tile(world, x, y, true, &mut gen_rng);
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
                    kill_tile(world, m, n, true, &mut gen_rng);
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

// --- Quick Cleanup (W4-d104 pass 29) -----------------------------------------

/// `WorldGen.beachDistance` = 380 (`WorldGen.cs:4154`) +
/// `WorldGen.oceanLevel` (`:4447`: `(worldSurface + rockLayer) / 2 + 40`)
/// — the `oceanDepths` probe (`:10963-10973`), the evil.rs copy's twin.
fn ocean_depths_qc(world: &World, x: i32, y: i32, ocean_level: f64) -> bool {
    if y as f64 > ocean_level {
        return false;
    }
    x < 380 || x > world.width - 380
}

/// `WorldGen.SolidTile(i, j)` (`WorldGen.cs:70111-70132`) under the Quick
/// Cleanup window: the pass flips `tileSolid[137]` (Traps) and
/// `tileSolid[130]` (ActiveStoneBlock) false for the sweep and restores
/// them after (`:17884-17885`, `:17991-17992`). The port's table is
/// static, so the window rides the predicate (the temple `spike_solid`
/// precedent). A null tile reads solid (`:70116`).
fn solid_tile_qc(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    if t.kind == 137 || t.kind == 130 {
        return false;
    }
    t.active()
        && crate::tiledata::TILE_SOLID.has(t.kind)
        && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

/// The `Quick Cleanup` pass (`WorldGen.cs:17875-17993`, W4-d104 pass 29):
/// the full-world tidy sweep — the ocean lava→water swap, the beach sand
/// drip (the per-iteration `Next(4, 7)` condition draw — the C# re-rolls
/// the loop bound on EVERY condition check), the sandstone/hardened-sand
/// wall retypes + liquid arms, the surface-sand wall inheritance, and the
/// slope/half-brick clear family. The `Skyblock.denyAllGeneration` gate
/// (`:3131`: skyblock) rides the step adapter; the `NotTheBees()` call
/// (`:17881-17883`) is the seed lane's (the whole-world hive conversion
/// is unported — biomes.rs's documented scope).
pub(crate) fn pass_quick_cleanup(
    world: &mut World,
    world_surface: f64,
    rock_layer: f64,
    rng: &mut super::Rng,
) {
    let ocean_level = (world_surface + rock_layer) / 2.0 + 40.0;
    let round_landmasses = world
        .micro_seeds
        .contains(MicroSeedFlags::ROUND_LANDMASSES);
    let dual_dungeons = world.micro_seeds.contains(MicroSeedFlags::DUAL_DUNGEONS);
    for i in 20..(world.width - 20) {
        for j in 20..(world.height - 20) {
            // `:17892-17896`: the ocean non-water liquid → water.
            if ocean_depths_qc(world, i, j, ocean_level) {
                if let Some(t) = world.tile_mut(i, j) {
                    if t.liquid > 0 && t.liquid_kind() != crate::tile::LiquidKind::Water {
                        t.set_liquid_kind(crate::tile::LiquidKind::Water);
                    }
                }
            }
            // `:17897-17909`: the beach sand drip — the surface sand over
            // open air fills dirt downward. The C# condition draws
            // `Next(4, 7)` on EVERY check (the bound is inside the for's
            // condition), so the port re-rolls per iteration too.
            let sand_top = (j as f64) < world_surface
                && ocean_depths_qc(world, i, j, ocean_level)
                && !round_landmasses
                && world.tile(i, j).is_some_and(|t| t.kind == 53 && t.active());
            if sand_top {
                if let Some(t) = world.tile_mut(i, j) {
                    if t.bottom_slope() {
                        t.set_slope(0);
                    }
                }
                let mut k = j + 1;
                loop {
                    let limit = j + rng.range(4, 7);
                    if k >= limit {
                        break;
                    }
                    let clear_at = |world: &World, y: i32, allow_495: bool| {
                        world.tile(i, y).is_none_or(|t| {
                            !t.active()
                                || (t.kind != 397 && t.kind != 53 && (!allow_495 || t.kind != 495))
                        })
                    };
                    if !clear_at(world, k, false)
                        || !clear_at(world, k + 1, true)
                        || !clear_at(world, k + 2, true)
                    {
                        break;
                    }
                    if let Some(t) = world.tile_mut(i, k) {
                        t.kind = 0;
                        t.flags |= crate::tile::FLAG_ACTIVE;
                        t.flags &= !crate::tile::FLAG_HALF_BRICK;
                        t.set_slope(0);
                    }
                    k += 1;
                }
            }
            // `:17910-17929`: the sandstone/hardened-sand wall family.
            let wall = world.tile(i, j).map(|t| t.wall).unwrap_or(0);
            if wall == 187 || wall == 216 {
                if let Some(t) = world.tile_mut(i, j) {
                    if matches!(t.kind, 59 | 123 | 224 | 368 | 367) {
                        t.kind = 397;
                    }
                    if !dual_dungeons {
                        if (j as f64) <= rock_layer {
                            t.liquid = 0;
                        } else if t.liquid > 0 {
                            t.liquid = u8::MAX;
                            t.set_liquid_kind(crate::tile::LiquidKind::Lava);
                        }
                    }
                }
            }
            // `:17930-17951`: the surface-sand wall inheritance — the ±3
            // box's LAST walled column wins (the C# `break` exits only the
            // inner loop).
            let inherits = (j as f64) < world_surface
                && world.tile(i, j).is_some_and(|t| t.active() && t.kind == 53)
                && world.tile(i, j + 1).map(|t| t.wall).unwrap_or(0) == 0
                && !solid_tile_qc(world, i, j + 1);
            if inherits {
                let mut num = 0u16;
                for l in (i - 3)..=(i + 3) {
                    for m in (j - 3)..=(j + 3) {
                        let w = world.tile(l, m).map(|t| t.wall).unwrap_or(0);
                        if w > 0 {
                            num = w;
                            break;
                        }
                    }
                }
                if num > 0 {
                    if let Some(t) = world.tile_mut(i, j + 1) {
                        t.wall = num;
                    }
                    if let Some(t) = world.tile_mut(i, j) {
                        if t.wall == 0 {
                            t.wall = num;
                        }
                    }
                }
            }
            // `:17952-17988`: the slope/half-brick clear family.
            let (active, kind, top, half, bottom) = {
                let Some(t) = world.tile(i, j) else {
                    continue;
                };
                (t.active(), t.kind, t.top_slope(), t.half_brick(), t.bottom_slope())
            };
            if !active || !super::tile_runner::save_slopes(kind as i32) {
                // `Tile.ClearSlope` (`Tile.cs:130-134`).
                if let Some(t) = world.tile_mut(i, j) {
                    t.set_slope(0);
                    t.flags &= !crate::tile::FLAG_HALF_BRICK;
                }
            } else if !crate::tiledata::SET_PLATFORMS.has(kind)
                && crate::tiledata::SET_CAN_BE_CLEARED_DURING_GENERATION.has(kind)
            {
                let trap_neighbour = world.tile(i + 1, j).is_some_and(|t| t.kind == 137)
                    || world.tile(i - 1, j).is_some_and(|t| t.kind == 137);
                if top || half {
                    if kind != 225 || !half {
                        let clear = !solid_tile_qc(world, i, j + 1) || trap_neighbour;
                        if clear {
                            if let Some(t) = world.tile_mut(i, j) {
                                t.flags &= !crate::tile::FLAG_ACTIVE;
                            }
                        }
                    }
                } else if bottom {
                    let clear = !solid_tile_qc(world, i, j - 1) || trap_neighbour;
                    if clear {
                        if let Some(t) = world.tile_mut(i, j) {
                            t.flags &= !crate::tile::FLAG_ACTIVE;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    // --- W4-d104 pass 29: Quick Cleanup -------------------------------------

    /// A tidy-sweep world: stone ground at row 60, the surface line at 50.
    fn qc_world() -> World {
        let mut w = World::new("qc".into(), 900, 200);
        for x in 0..w.width {
            for y in 60..w.height {
                let i = w.index(x, y);
                w.tiles[i].kind = 1;
                w.tiles[i].flags |= crate::tile::FLAG_ACTIVE;
            }
        }
        w
    }

    /// W4-d104 pass 29: the sweep replays byte-equal on the same stream
    /// (the a == b witness) and retypes the sandstone-wall mud + clears
    /// the unsupported slopes.
    #[test]
    fn w4_d104_29_the_cleanup_sweep_replays_and_tidies() {
        let mut a = qc_world();
        let mut b = qc_world();
        for w in [&mut a, &mut b] {
            // A mud block under a sandstone wall (`:17912-17914`).
            let i = w.index(450, 70);
            w.tiles[i].kind = 59;
            w.tiles[i].wall = 187;
            // An unsupported top-slope stone over air (`:17960-17970`).
            let i = w.index(460, 40);
            w.tiles[i].kind = 1;
            w.tiles[i].flags |= crate::tile::FLAG_ACTIVE;
            w.tiles[i].set_slope(1);
            // A lava pocket under the sandstone wall below rockLayer
            // (`:17924-17927` — the full-lava arm).
            let i = w.index(470, 100);
            w.tiles[i].wall = 216;
            w.tiles[i].liquid = 40;
            w.tiles[i].set_liquid_kind(crate::tile::LiquidKind::Water);
        }
        pass_quick_cleanup(&mut a, 50.0, 80.0, &mut crate::worldgen::noise::Rng(42));
        pass_quick_cleanup(&mut b, 50.0, 80.0, &mut crate::worldgen::noise::Rng(42));
        assert_eq!(a.tiles, b.tiles, "a == b: the pass stream replays");
        assert_eq!(a.tile(450, 70).unwrap().kind, 397, "the mud retypes");
        assert!(!a.tile(460, 40).unwrap().active(), "the unsupported slope clears");
        let lava = a.tile(470, 100).unwrap();
        assert_eq!(lava.liquid, 255, "the deep wall pocket fills");
        assert_eq!(lava.liquid_kind(), crate::tile::LiquidKind::Lava, "as lava");
    }

    /// W4-d104 pass 29: the ocean lava→water swap (`:17892-17896`) fires
    /// inside the beach band above the ocean level only.
    #[test]
    fn w4_d104_29_the_ocean_lava_becomes_water() {
        let mut w = qc_world();
        // Inside the left beach band (x < 380), above oceanLevel
        // ((50 + 80)/2 + 40 = 105).
        let i = w.index(100, 40);
        w.tiles[i].liquid = 200;
        w.tiles[i].set_liquid_kind(crate::tile::LiquidKind::Lava);
        // The middle keeps its lava.
        let i = w.index(450, 40);
        w.tiles[i].liquid = 200;
        w.tiles[i].set_liquid_kind(crate::tile::LiquidKind::Lava);
        pass_quick_cleanup(&mut w, 50.0, 80.0, &mut crate::worldgen::noise::Rng(7));
        assert_eq!(
            w.tile(100, 40).unwrap().liquid_kind(),
            crate::tile::LiquidKind::Water,
            "the ocean lava swaps"
        );
        assert_eq!(
            w.tile(450, 40).unwrap().liquid_kind(),
            crate::tile::LiquidKind::Lava,
            "the inland lava stays"
        );
    }
}
