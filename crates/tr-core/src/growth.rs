//! The world-growth ticks — `WorldGen.UpdateWorld` (`WorldGen.cs:71495-71632`),
//! `GrowAlch` (`:45777-45890`) and the crystal-shard arm of `hardUpdateWorld`
//! (`:69600-69640`).
//!
//! Source map (line numbers into `decompiled/Terraria/`):
//! - the rate (`GetWorldUpdateRate` `:72020-72030`): `min(dayRate, 24)`, 0
//!   when the time freezes; `dayRate` = the time rate ×5 when every player
//!   sleeps (`Main.cs:6210-6220`)
//! - the loops (`:71540-71632`): the overground
//!   `maxX·maxY · 3e-5 · rate` (×1.5 raining), the underground
//!   `maxX·maxY · 1.5e-5 · rate`; each overground pick rolls the alch-seed
//!   gate `Next(num7·100) == 0 → skip` with
//!   `num7 = Lerp(151, 151·2.8, clamp(maxTilesX/4200 − 1, 0, 1))`
//!   (`:71606-71614`)
//! - `UpdateWorld_OvergroundTile` (`:72030+`): the port's slice is the herb
//!   branch (`TileID.Sets.tileAlch` = {82, 83, 84} → `GrowAlch`); the other
//!   arms (the trees, the cacti, the shells, the bamboo) land with their
//!   follow-ups
//! - `GrowAlch` (`:45777-45890`): the herb lifecycle — the mature 84
//!   blooms into 83 by the day/night/rain/depth cue (the frameX stages
//!   0/18/36/72/90/108), the 83 flashes stage frames, the sprout 82 matures;
//!   the liquid kill keeps only the matching stage
//! - the crystal shards (`hardUpdateWorld` `:69603-69640`): the
//!   `CanGrowCrystalShards` stones {117, 116, 164, 402, 403} below the rock
//!   layer roll `Next(5)`, pick a 4-dir neighbour, need it empty, keep the
//!   density `< 2` per radius-6 box and place tile 129 with the style
//!   `Next(18)` (the big shard `18 + Next(6)` at 1/50)
//!
//! Deviations (ANALYSIS "Growth ticks (M7-GROWTH-1)"): the vines/moss (the
//! grass-growth pass `:74269+`), the tree/cactus growth and the biome
//! spread = M7-GROWTH-2; the notTheBees/remix loop variants are skipped
//! (the port generates no such seeds); the NetMessage sides are MP scope.

use crate::tile::FLAG_ACTIVE;
use crate::tile_light::FastRandom;
use crate::world::World;

/// `TileID.Sets.tileAlch` (`TileID.cs`) — the herb tiles GrowAlch runs on.
pub const TILE_ALCH: [u16; 3] = [82, 83, 84];

/// `TileID.Sets.CanGrowCrystalShards` (`TileID.cs:585`).
pub const CAN_GROW_CRYSTAL_SHARDS: [u16; 5] = [117, 116, 164, 402, 403];

/// `GetWorldUpdateRate` (`WorldGen.cs:72020-72030`): `min(dayRate, 24)`,
/// 0 while the time freezes.
pub fn world_update_rate(day_rate: i32, freeze_time: bool) -> i32 {
    if freeze_time {
        return 0;
    }
    day_rate.min(24)
}

/// The alch-seed gate (`:71606-71614`): `Next(num7·100) == 0` skips the
/// pick; `num7 = Lerp(151, 151·2.8, clamp(maxTilesX/4200 − 1, 0, 1))`.
pub fn alch_seed_skips(world: &World, rng: &mut FastRandom) -> bool {
    let t = ((world.width as f64 / 4200.0 - 1.0).clamp(0.0, 1.0));
    let num7 = 151.0 + (151.0 * 2.8 - 151.0) * t;
    rng.next((num7 * 100.0) as i32) == 0
}

/// `GrowAlch` (`WorldGen.cs:45777-45890`): the herb lifecycle. The stages
/// ride `frameX` (0/18/36/72/90/108) and the bloom target 83; the mature 84
/// blooms back; the liquid kill keeps only the matching stage.
pub fn grow_alch(world: &mut World, x: i32, y: i32, day_time: bool, raining: bool, rng: &mut FastRandom) {
    let Some(t) = world.tile(x, y) else {
        return;
    };
    if !t.active() {
        return;
    }
    // `:45784-45796`: the liquid kill — the wrong liquid for the stage
    // destroys the herb (the lava keeps only the stage-5 cactus-stem shape).
    if t.liquid > 0 {
        let stage = t.frame_x / 18;
        let lava = t.flags & crate::tile::FLAG_LIQUID_LAVA != 0;
        let water = t.liquid_kind() == crate::tile::LiquidKind::Water;
        let keep = (lava && stage == 5) || (water && (stage == 1 || stage == 4));
        if !keep {
            crate::edit::kill_tile(world, x, y, false);
            return;
        }
    }
    if t.kind == 82 {
        // The sprout matures: the special cues bloom straight into 83.
        let frame_x = world.tile(x, y).map(|t| t.frame_x).unwrap_or(0);
        let below_surface = y as f64 > world.world_surface;
        let near_underworld = y > world.height - 200;
        let bloom = if rng.next(50) == 0 && (frame_x == 108 || rng.next(2) == 0) {
            true
        } else if day_time && frame_x == 0 && rng.next(50) == 0 {
            true
        } else if !day_time && frame_x == 18 && rng.next(50) == 0 {
            true
        } else if raining && frame_x == 72 && rng.next(50) == 0 {
            true
        } else if below_surface && frame_x == 36 && rng.next(50) == 0 {
            true
        } else if near_underworld && frame_x == 90 && rng.next(50) == 0 {
            true
        } else {
            false
        };
        if bloom {
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = 83;
            }
        }
        return;
    }
    // `:45862-45883`: the stage-3 flash (`frameX == 36`, 2 of 3 rolls skip).
    if world.tile(x, y).is_some_and(|t| t.frame_x == 36) && rng.next(3) != 0 {
        let kind = world.tile(x, y).map(|t| t.kind).unwrap_or(0);
        if kind == 83 {
            if rng.next(2) == 0 {
                if let Some(t) = world.tile_mut(x, y) {
                    t.kind = 84;
                }
            }
        } else if kind == 84 || rng.next(5) == 0 {
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = 83;
            }
        }
    }
}

/// `UpdateWorld_OvergroundTile`'s port slice (`:72030+`): the herb branch.
/// The other arms (the trees, the cacti, the beach shells, the bamboo) land
/// with M7-GROWTH-2.
pub fn update_world_overground_tile(
    world: &mut World,
    x: i32,
    y: i32,
    day_time: bool,
    raining: bool,
    rng: &mut FastRandom,
) {
    let Some(t) = world.tile(x, y) else {
        return;
    };
    if !t.active() {
        return;
    }
    if TILE_ALCH.contains(&t.kind) {
        grow_alch(world, x, y, day_time, raining, rng);
    }
}

/// The crystal-shard arm of `hardUpdateWorld` (`:69603-69640`): the shard
/// stones below the rock layer roll `Next(5)`, the 4-dir neighbour must be
/// empty, the radius-6 density stays under 2, the style `Next(18)` (the big
/// shard `18 + Next(6)` at 1/50).
pub fn grow_crystal_shards(world: &mut World, x: i32, y: i32, rng: &mut FastRandom) {
    let Some(t) = world.tile(x, y) else {
        return;
    };
    if !t.active() || !CAN_GROW_CRYSTAL_SHARDS.contains(&t.kind) {
        return;
    }
    // `((double)j > Main.rockLayer || remixWorld)`.
    if (y as f64) <= world.rock_layer {
        return;
    }
    if rng.next(5) != 0 {
        return;
    }
    // `:69609-69625`: the 4-dir pick — 0 left, 1 right, 2/3 down (the C#'s
    // `default: num3 = ((num != 0) ? 1 : -1)` resolves to +1 for both —
    // the shards never grow upward).
    let (dx, dy) = match rng.next(4) {
        0 => (-1, 0),
        1 => (1, 0),
        _ => (0, 1),
    };
    let (nx, ny) = (x + dx, y + dy);
    if world.tile(nx, ny).is_some_and(|t| t.active()) {
        return;
    }
    // `:69629-69638`: the radius-6 density scan for the existing shards
    // (tile 129).
    let mut count = 0;
    for k in (x - 6)..=(x + 6) {
        for l in (y - 6)..=(y + 6) {
            if world
                .tile(k, l)
                .is_some_and(|t| t.active() && t.kind == 129)
            {
                count += 1;
            }
        }
    }
    if count >= 2 {
        return;
    }
    // `:69639-69643`: the style — the big shards 18..=23 at 1/50.
    let mut style = rng.next(18);
    if rng.next(50) == 0 {
        style = 18 + rng.next(6);
    }
    if let Some(t) = world.tile_mut(nx, ny) {
        t.kind = 129;
        t.frame_x = (style * 18) as i16;
        t.flags |= FLAG_ACTIVE;
    }
}

/// `UpdateWorld`'s growth loops (`:71540-71632`): the overground
/// `3e-5 · rate` picks (×1.5 raining) with the alch-seed gate, then the
/// underground `1.5e-5 · rate` picks (the port's slice runs the same herb
/// pass — the underground arms are GROWTH-2). Returns the picks made (the
/// diagnostics).
pub fn update_world(
    world: &mut World,
    day_rate: i32,
    raining: bool,
    freeze_time: bool,
    day_time: bool,
    rng: &mut FastRandom,
) -> u32 {
    let rate = world_update_rate(day_rate, freeze_time);
    if rate == 0 {
        return 0;
    }
    let mut picks = 0u32;
    let mut overground_count =
        (world.width as f64 * world.height as f64) * 3e-5 * rate as f64;
    if raining {
        overground_count *= 1.5;
    }
    for _ in 0..overground_count as i32 {
        if alch_seed_skips(world, rng) {
            continue;
        }
        let x = rng.next_range(10, world.width - 10);
        let y = rng.next_range(10, world.world_surface as i32 - 1);
        update_world_overground_tile(world, x, y, day_time, raining, rng);
        picks += 1;
    }
    let underground_count = (world.width as f64 * world.height as f64) * 1.5e-5 * rate as f64;
    for _ in 0..underground_count as i32 {
        let x = rng.next_range(10, world.width - 10);
        let y = rng.next_range(world.world_surface as i32 - 1, world.height - 20);
        // The underground arms (the vines, the moss, the spread) are
        // GROWTH-2; the herb branch keeps the parity here.
        update_world_overground_tile(world, x, y, day_time, raining, rng);
        picks += 1;
    }
    picks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::Tile;

    fn world() -> World {
        let mut w = World::new("t".into(), 4200, 400);
        w.rock_layer = 160.0;
        w
    }

    fn put(w: &mut World, x: i32, y: i32, kind: u16, frame_x: i16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            frame_x,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    #[test]
    fn the_rate_caps_and_freezes() {
        // `:72020-72030`: `min(dayRate, 24)`, the freeze → 0.
        assert_eq!(world_update_rate(1, false), 1);
        assert_eq!(world_update_rate(30, false), 24);
        assert_eq!(world_update_rate(30, true), 0);
    }

    #[test]
    fn the_alch_gate_uses_the_world_size_lerp() {
        // `:71606-71614`: `num7 = Lerp(151, 422.8, maxTilesX/4200 − 1)` —
        // the 4200-wide world sits at t = 0 → 151; a 8400-wide world at
        // t = 1 → 422.8. The gate skips on `Next(num7·100) == 0`.
        let w = world();
        let mut rng = FastRandom::new(5);
        let skips = (0..1000).filter(|_| alch_seed_skips(&w, &mut rng)).count();
        // The skip chance = 1/(151·100) ≈ 1/15100 → ~0.07/1000 rolls; the
        // 4200-world roll is nearly never skipped.
        assert!(skips <= 2, "skips {skips}");
    }

    #[test]
    fn the_herb_lifecycle_blooms_by_the_cues() {
        // The sprout 82 (the stage-0 frame) blooms in the daylight.
        let mut w = world();
        put(&mut w, 30, 90, 82, 0);
        let mut rng = FastRandom::new(1);
        let mut bloomed = false;
        for _ in 0..4000 {
            grow_alch(&mut w, 30, 90, true, false, &mut rng);
            if w.tile(30, 90).unwrap().kind == 83 {
                bloomed = true;
                break;
            }
        }
        assert!(bloomed, "the daylight cue blooms the stage-0 sprout");
        // The rain cue drives the stage-72 frame.
        let mut w = world();
        put(&mut w, 30, 90, 82, 72);
        let mut rng = FastRandom::new(2);
        let mut bloomed = false;
        for _ in 0..4000 {
            grow_alch(&mut w, 30, 90, false, true, &mut rng);
            if w.tile(30, 90).unwrap().kind == 83 {
                bloomed = true;
                break;
            }
        }
        assert!(bloomed, "the rain cue blooms the stage-72 frame");
        // The generic arm (`:45785-45800`): every frame except 108 blooms on
        // `Next(50) == 0` regardless of the cue — the cue arms are the
        // chain's fallbacks.
        let mut w = world();
        put(&mut w, 30, 90, 82, 0);
        let mut rng = FastRandom::new(3);
        let mut bloomed = false;
        for _ in 0..400 {
            grow_alch(&mut w, 30, 90, false, false, &mut rng);
            if w.tile(30, 90).unwrap().kind == 83 {
                bloomed = true;
                break;
            }
        }
        assert!(bloomed, "the generic 1/50 arm blooms any frame");
    }

    #[test]
    fn the_wrong_liquid_kills_the_herb() {
        let mut w = world();
        put(&mut w, 30, 90, 82, 0);
        let i = w.index(30, 90);
        w.tiles[i].liquid = 200;
        let mut rng = FastRandom::new(1);
        grow_alch(&mut w, 30, 90, true, false, &mut rng);
        assert!(
            w.tile(30, 90).unwrap().flags & FLAG_ACTIVE == 0,
            "the water on the stage-0 herb kills it"
        );
    }

    #[test]
    fn the_shards_grow_sparse_and_big_rarely() {
        let mut w = world();
        // The stone below the rock layer with an air pocket BELOW (the
        // shards grow down/sideways only).
        for x in 100..120 {
            for y in 200..210 {
                if !(x == 110 && y == 206) {
                    put(&mut w, x, y, 117, 0);
                }
            }
        }
        let mut rng = FastRandom::new(4);
        let mut grew = 0;
        for _ in 0..200 {
            let before = w
                .tiles
                .iter()
                .filter(|t| t.active() && t.kind == 129)
                .count();
            grow_crystal_shards(&mut w, 110, 205, &mut rng);
            let after = w
                .tiles
                .iter()
                .filter(|t| t.active() && t.kind == 129)
                .count();
            grew += (after - before) as i32;
        }
        assert!(grew > 0, "the shards grow on the stones");
        // The density cap: no 6-radius box holds 3 shards.
        let mut max_density = 0;
        for x in 100..120 {
            for y in 200..210 {
                let mut count = 0;
                for k in (x - 6)..=(x + 6) {
                    for l in (y - 6)..=(y + 6) {
                        if w.tile(k, l).is_some_and(|t| t.active() && t.kind == 129) {
                            count += 1;
                        }
                    }
                }
                max_density = max_density.max(count);
            }
        }
        assert!(max_density <= 2, "the density cap holds (max {max_density})");
    }

    #[test]
    fn the_driver_picks_at_the_verbatim_rates() {
        // 4200×400 tiles at rate 1: the overground 3e-5·tiles = 50 picks,
        // ×1.5 raining = 75; the underground 1.5e-5·tiles = 25.
        let mut w = world();
        let mut rng = FastRandom::new(9);
        let picks = update_world(&mut w, 1, false, false, true, &mut rng);
        assert_eq!(picks, 50 + 25);
        let picks = update_world(&mut w, 1, true, false, true, &mut rng);
        assert_eq!(picks, 75 + 25);
        // The frozen time picks nothing.
        let picks = update_world(&mut w, 1, false, true, true, &mut rng);
        assert_eq!(picks, 0);
    }
}
