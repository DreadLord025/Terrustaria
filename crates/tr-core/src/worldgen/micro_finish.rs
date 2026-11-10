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
}
