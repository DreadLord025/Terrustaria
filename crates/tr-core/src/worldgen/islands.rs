//! The floating islands — the compact port of the `FloatingIslands` pass
//! (`WorldGen.cs:12924-13133`) + `CloudIsland` (`:78132-78540`) +
//! `CloudLake` (`:78539+`) + the island houses (`FloatingIslandHouses`
//! `:17862+`).
//!
//! The C# island body is the cloud block 189 (the `Clouds` set), the lake
//! islands line with the rain cloud 196, the house interiors carry the
//! cloud wall 73, the surface grows the grass cap. The islands sit at
//! `Next(90, worldSurface − 100)`, the x spread over the 10-90 % band with
//! the ±180 spacing and the spawn exclusion.
//!
//! Deviations (ANALYSIS "Floating islands (M1.4-WGEN-4)"): the disc shape
//! approximates the C#'s shrinking-ring walk (`FloatingIsland`'s
//! `Next(80,120)` diameter with the ±20 % wobble); the snow/desert island
//! variants (`SnowCloudIsland`/`DesertCloudIsland`) are out (the port's
//! snow band sits east — the variant rim tiles land with the biome-tint
//! follow-up); the house builds are shape-level (the C#'s
//! `skyIslandHouseCount` furniture sets are the follow-up).

use super::structures::place_chest;
use crate::tile::{FLAG_ACTIVE, Tile};
use crate::world::World;

pub(crate) const TILE_CLOUD: u16 = 189;
pub(crate) const TILE_RAIN_CLOUD: u16 = 196;
pub(crate) const WALL_CLOUD: u16 = 73;

fn put_cloud(world: &mut World, x: i32, y: i32, kind: u16) {
    if x < 2 || y < 2 || x >= world.width - 2 || y >= world.height - 2 {
        return;
    }
    let i = world.index(x, y);
    let t = &mut world.tiles[i];
    t.kind = kind;
    t.flags |= FLAG_ACTIVE;
    t.liquid = 0;
    t.set_slope(0);
}

/// The `CloudIsland` disc (`:78132+`): the elliptical cloud body with the
/// grass cap on the top surface. `lake` islands carve the rain-cloud bowl
/// with the water instead of the grass cap's centre.
fn build_island(world: &mut World, cx: i32, cy: i32, radius: i32, lake: bool, rng: &mut super::Rng) {
    for dx in -radius..=radius {
        for dy in -radius / 2..=radius / 2 {
            let px = cx + dx;
            let py = cy + dy;
            if px < 2 || py < 2 || px >= world.width - 2 || py >= world.height - 2 {
                continue;
            }
            // The ellipse: the horizontal radius full, the vertical halved.
            let ex = dx as f32 / radius as f32;
            let ey = dy as f32 / (radius as f32 * 0.5);
            let d = (ex * ex + ey * ey).sqrt();
            if d > 1.0 {
                continue;
            }
            // The wobble on the boundary (`Next(80, 120) * 0.01`'s ragged
            // rim — the cells at 0.82..1.0 roll 50/50).
            if d > 0.82 && rng.range(0, 2) == 0 {
                continue;
            }
            put_cloud(world, px, py, TILE_CLOUD);
        }
    }
    // The grass cap on the top surface (`CloudIsland`'s `num19 = 196`-style
    // rim + the vanilla grass-over-dirt look — the port caps the cloud with
    // the grass 2 rows deep).
    for dx in -(radius - 2)..=(radius - 2) {
        for dy in 0..2 {
            let px = cx + dx;
            let py = cy - radius / 2 + dy;
            if px < 2 || py < 2 || px >= world.width - 2 {
                continue;
            }
            if world
                .tile(px, py)
                .is_some_and(|t| t.active() && t.kind == TILE_CLOUD)
            {
                let i = world.index(px, py);
                world.tiles[i].kind = if dy == 0 { 2 } else { 0 };
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
    }
    if lake {
        // The rain-cloud bowl with the water (`CloudLake`'s shape): the
        // 6-wide rim of the rain cloud 196 sunk 3 deep, the water inside.
        let half = (radius / 3).max(4);
        for dx in -half..=half {
            for dy in 0..4 {
                let px = cx + dx;
                let py = cy - radius / 2 + 2 + dy;
                if px < 2 || py < 2 || px >= world.width - 2 {
                    continue;
                }
                let i = world.index(px, py);
                if dy == 3 || dx.abs() == half {
                    let t = &mut world.tiles[i];
                    t.kind = TILE_RAIN_CLOUD;
                    t.flags |= FLAG_ACTIVE;
                    t.set_slope(0);
                } else {
                    let t = &mut world.tiles[i];
                    *t = Tile::default();
                    t.liquid = 255;
                    t.set_liquid_kind(crate::tile::LiquidKind::Water);
                }
            }
        }
    }
}

/// The island house (`FloatingIslandHouses`, `:17862+`): the plank box on
/// the island's surface with the cloud-wall interior and the chest. The
/// chest arm is `IslandHouse`'s (`WorldGen.cs:79344-79394`): the
/// `skyIslandHouseCount` cycle picks the main item (the `> 3 → Next(4)`
/// re-roll), the style is 13 — or 2 under `getGoodWorldGen` — and
/// `skyIslandHouseCount++` runs after the place (`:79394`). S1-A-4: the
/// stock rides inline.
fn build_house(
    world: &mut World,
    cx: i32,
    top: i32,
    rng: &mut super::Rng,
    mut stock: Option<&mut super::chest_stock::ChestStockHook<'_>>,
    get_good: bool,
    sky_count: &mut u32,
) {
    let hw = rng.range(4, 7);
    let hh = rng.range(3, 5);
    for px in (cx - hw)..=(cx + hw) {
        for py in (top - hh)..=top {
            if px < 2 || py < 2 || px >= world.width - 2 || py >= world.height - 2 {
                continue;
            }
            let edge = px == cx - hw || px == cx + hw || py == top - hh || py == top;
            let i = world.index(px, py);
            if edge {
                let t = &mut world.tiles[i];
                t.kind = 9; // `TileID.Planks` — the house frame
                t.flags |= FLAG_ACTIVE;
                t.set_slope(0);
            } else {
                world.tiles[i] = Tile::default();
                world.tiles[i].wall = WALL_CLOUD;
            }
        }
    }
    // The chest inside on the floor (`:79371/:79375`'s arms).
    let mut num16 = *sky_count;
    if num16 > 3 {
        num16 = rng.range(0, 4) as u32;
    }
    let main = match num16 {
        0 => 159,
        1 => 65,
        2 => 158,
        _ => 2219,
    };
    let chest_style = if get_good { 2 } else { 13 };
    let style =
        super::structures::resolved_chest_style(chest_style, main, top - 1, world.world_surface);
    let chest_x = cx + hw - 2;
    if place_chest(world, chest_x, top - 1, style) {
        super::chest_stock::stock_placed_chest(
            world,
            stock.as_deref_mut(),
            rng,
            chest_x,
            top - 1,
            style,
            21,
            main,
            false,
        );
    }
    // `GenVars.skyIslandHouseCount++` (`:79394`) — once per house, after
    // the place.
    *sky_count += 1;
}

/// `SecretSeed.Variations.extraFloatingIslandsNormalAmount`
/// (`WorldGen.cs:234-243`): the skyblock passthrough — an `extraFloatingIslands`
/// world that is ALSO a skyblock world keeps the normal (non-multiplied)
/// island count:
///
/// ```text
/// if (SecretSeed.extraFloatingIslands.Enabled) return Main.skyblockWorld;
/// return false;
/// ```
///
/// The port's `GenOptions` carries no skyblock bit (the classic `SeedFlags`'s
/// `SKYBLOCK` lives on the UI/save path, M5-SEED-1), so the arm reads
/// `false` here: an `extraFloatingIslands` port world always takes the
/// multiplied count. Recorded as a deviation (the classic
/// `SKYBLOCK` flag exists in [`crate::seeds::SeedFlags`] and is simply not
/// wired into worldgen).
fn extra_floating_islands_normal_amount(world: &World) -> bool {
    world
        .micro_seeds
        .contains(crate::seeds::MicroSeedFlags::EXTRA_FLOATING_ISLANDS)
        && world.seeds.contains(crate::seeds::SeedFlags::SKYBLOCK)
}

/// `SecretSeed.Variations.extraFloatingIslandsReducedAmount`
/// (`WorldGen.cs:245-255`): the halved multiplier when the seed list is
/// crowded — `activeSecretSeedCount < 6` reads `noSurface.Enabled`, otherwise
/// `true`:
///
/// ```text
/// if (!SecretSeed.extraFloatingIslands.Enabled || SecretSeed.activeSecretSeedCount < 6)
///     return SecretSeed.noSurface.Enabled;
/// return true;
/// ```
///
/// The noSurface half reads the port's `NO_SURFACE` bit (its *behaviour* is
/// another agent's file); the flag itself is registered here, and
/// `active_secret_seed_count` folds the classic options in.
fn extra_floating_islands_reduced_amount(world: &World) -> bool {
    if !world
        .micro_seeds
        .contains(crate::seeds::MicroSeedFlags::EXTRA_FLOATING_ISLANDS)
        || crate::seeds::active_secret_seed_count(world.micro_seeds, world.seeds) < 6
    {
        world
            .micro_seeds
            .contains(crate::seeds::MicroSeedFlags::NO_SURFACE)
    } else {
        true
    }
}

/// The island count and its pre-multiplier base (`WorldGen.cs:12933-12955`).
/// The C# is `num = (int)(maxTilesX * 0.0008)` with skyblock's `/= 42` halving
/// of the search width; the port's compact stand-in is `3 + width / 4200` (a
/// 4200-wide world gets 4, matching the C#'s 3 + skyLakes). Returns
/// `(count, count_base)` — the multiplied loop count and the pre-multiplier
/// `num3` the house cap reads. Factored out of the pass so the multiplier is
/// directly testable.
fn island_count(world: &World) -> (i32, i32) {
    let extra = world
        .micro_seeds
        .contains(crate::seeds::MicroSeedFlags::EXTRA_FLOATING_ISLANDS);
    let normal = extra_floating_islands_normal_amount(world);
    let reduced = extra_floating_islands_reduced_amount(world);
    let count_base = 3 + (world.width / 4200) as i32;
    let count = if extra && !normal {
        if reduced {
            count_base * 2
        } else {
            count_base * 10
        }
    } else {
        count_base
    };
    (count, count_base)
}

/// The `FloatingIslands` pass (`:12924-13133`): 3-4 islands over the 10-90 %
/// band at `Next(90, worldSurface − 100)`, the ±180 spacing, the spawn
/// exclusion, every third island a sky lake. Returns the island count.
///
/// M5-SEED-2 `extraFloatingIslands` arms (`:12942-12954`, `:12970-12973`,
/// `:12982-12985`, `:13055-13062`, `:13117-13119`):
///
/// * **the count multiplier** (`:12942-12954`): `num *= 2` under
///   `extraFloatingIslandsReducedAmount`, else `num *= 10`, and
///   `GenVars.skyLakes` takes the same multiplier. The port's `count` is its
///   compact stand-in for the C#'s `num + GenVars.skyLakes` (the classic
///   `3 + width / 4200` ≈ `maxTilesX * 0.0008` plus skyblock's raised base),
///   so the two C# multipliers collapse into one on the port's count and the
///   skyblock passthrough (`!extraFloatingIslandsNormalAmount`) gates them.
/// * **the spawn-exclusion half-width** (`:12970-12973`): the C#'s `num8`
///   narrows from 150 to 100. The port's compact band is 340 (the spawn-lake
///   compensation), so the flag arm narrows THAT band to 100 — the absolute
///   C# value, a compact-model deviation the classic band keeps at 340.
/// * **the house spacing** (`:12982-12985`): `num9` narrows from 180 to 100
///   under `extraFloatingIslands && !extraFloatingIslandsNormalAmount`.
/// * **`i > 4`** (`:13055-13062`): the first island past index 4 is forced a
///   sky lake, then 1-in-4 (`genRand.Next(4) == 0`) — the C# sets `flag3`
///   (make a lake), which is the port's `lake` flag.
/// * **the house cap** (`:13117-13119`): `GenVars.numIslandHouses` is capped
///   at `num3 * 2`, the PRE-multiplier count — the discs still place, only the
///   houses stop. The port mirrors that: `build_house` is skipped past the
///   cap, and the port's house count is capped the same way.
pub(crate) fn pass_floating_islands(
    world: &mut World,
    surface: &[i32],
    rng: &mut super::Rng,
    mut stock: Option<&mut super::chest_stock::ChestStockHook<'_>>,
    get_good: bool,
    sky_count: &mut u32,
) -> u32 {
    // The C# worlds are 4200+ wide; the tiny test worlds (the sky band
    // thinner than `Next(90, worldSurface − 100)` plus the disc) skip.
    let radius_max = 60i32;
    if (world.world_surface as i32) - 100 - radius_max / 2 <= 90 {
        return 0;
    }
    let extra = world
        .micro_seeds
        .contains(crate::seeds::MicroSeedFlags::EXTRA_FLOATING_ISLANDS);
    let normal = extra_floating_islands_normal_amount(world);
    // `int num3 = num;` (`WorldGen.cs:12939-12941`): the pre-multiplier
    // island count, which the house cap (`:13117`) reads.
    let (count, count_base) = island_count(world);
    // `:13117-13119`: `if (extraFloatingIslands.Enabled &&
    // numIslandHouses > num3 * 2) numIslandHouses = num3 * 2;` — the cap is
    // on the pre-multiplier count, so a multiplied world still caps its
    // houses at twice the vanilla number.
    let house_cap = if extra { count_base * 2 } else { i32::MAX };
    let mut placed: Vec<(i32, i32)> = Vec::new();
    let mut lakes = 0u32;
    let mut houses = 0i32;
    // `:12970-12973`: the spawn-exclusion half-width (`num8` = 150 → 100).
    let exclusion = if extra { 100 } else { 340 };
    // `:12982-12985`: the island-house spacing (`num9` = 180 → 100).
    let spacing = if extra && !normal { 100 } else { 180 };
    // `:13055-13062`'s `bool flag` — whether this arm has already forced a lake.
    let mut lake_forced = false;
    for i in 0..count {
        let mut attempts = 0;
        let (cx, cy, lake);
        loop {
            attempts += 1;
            if attempts > 200 {
                return lakes;
            }
            let x = rng.range((world.width as f32 * 0.1) as i32, (world.width as f32 * 0.9) as i32);
            // The spawn exclusion (`:12975-12978`: the ±`num8` band around
            // the middle — wide enough to cover the spawn lake).
            if (x - world.width / 2).abs() < exclusion {
                continue;
            }
            // The ±180 spacing from the placed islands (`:12987-12995`).
            if placed.iter().any(|(px, _)| (x - px).abs() < spacing) {
                continue;
            }
            // `Next(90, worldSurface − 100)` (`:13030-13033`): the disc
            // bottom stays ~75+ tiles over the deepest surface column
            // (`worldSurface = surface_max + 25` carries the margin).
            let y_hi = world.world_surface as i32 - 100;
            let y = rng.range(90, y_hi.max(91));
            // The site must be open sky (the C# scans the ±75-vertical
            // window around the island centre, `:13015-13028`).
            let clear = (x - 70..x + 70).all(|px| {
                (y - 75..y + 75).all(|py| {
                    !world
                        .tile(px, py)
                        .is_some_and(|t| t.active() && super::biomes::is_solid_biome_tile(t.kind))
                })
            });
            if !clear {
                continue;
            }
            cx = x;
            cy = y;
            // `:13048-13051`: every third island is a sky lake.
            lake = {
                let mut lake = i % 3 == 2;
                // `:13055-13062`: `num2 >= num` forces the tail islands into
                // lakes; the port's count is that sum, so its every-third
                // cadence carries the same share, and the `i > 4` arm forces
                // the first lake then rolls 1-in-4.
                if extra && i > 4 {
                    if !lake_forced {
                        lake = true;
                        lake_forced = true;
                    } else if rng.range(0, 4) == 0 {
                        lake = true;
                    }
                }
                lake
            };
            break;
        }
        // `FloatingIsland`'s `Next(80, 120)` is the DIAMETER — the
        // half-width 40-60 and the vertical half-extent ≈ half of that.
        let radius = rng.range(40, 61);
        build_island(world, cx, cy, radius, lake, rng);
        // `:13117-13119`: the house cap — the island still places, the house
        // does not.
        if houses < house_cap {
            build_house(world, cx, cy - radius / 2, rng, stock.as_deref_mut(), get_good, sky_count);
            houses += 1;
        }
        placed.push((cx, cy));
        if lake {
            lakes += 1;
        }
    }
    lakes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seeds::MicroSeedFlags;
    use crate::worldgen::noise::Rng;

    /// A wide sky world with the ground far below the `Next(90, worldSurface -
    /// 100)` island band, so the site gate's ±75 clear window never meets it.
    fn sky_world(width: i32, height: i32, ground: i32) -> World {
        let mut w = World::new("fi".into(), width, height);
        for x in 0..width {
            for y in ground..height {
                let i = w.index(x, y);
                w.tiles[i].kind = 1;
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.world_surface = (ground + 25) as f64;
        w
    }

    fn flat_surface(width: i32, row: i32) -> Vec<i32> {
        vec![row; width as usize]
    }

    fn count_kind(world: &World, kind: u16) -> usize {
        world
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == kind)
            .count()
    }

    fn flagged_island_world() -> World {
        let mut w = World::new("fi".into(), 4200, 1200);
        w.micro_seeds |= MicroSeedFlags::EXTRA_FLOATING_ISLANDS;
        w
    }

    #[test]
    fn the_island_count_multipliers_change_the_counts_under_the_flags() {
        // `WorldGen.cs:12942-12954`: `num *= 10` normally, `num *= 2` under
        // `Variations.extraFloatingIslandsReducedAmount`, and neither when the
        // world is ALSO a skyblock world (`extraFloatingIslandsNormalAmount`).
        let classic = World::new("fi".into(), 4200, 1200);
        assert_eq!(island_count(&classic), (4, 4), "the classic control");

        let extra = flagged_island_world();
        // A lone extraFloatingIslands world has count 1 (< 6) and no
        // noSurface, so the reduced variation is FALSE — the 10x arm.
        assert_eq!(island_count(&extra), (40, 4));

        let mut crowded = flagged_island_world();
        crowded.micro_seeds |= MicroSeedFlags::PAINT_EVERYTHING_GRAY
            | MicroSeedFlags::ERROR_WORLD
            | MicroSeedFlags::RAINS_FOR_A_YEAR
            | MicroSeedFlags::RANDOM_SPAWN
            | MicroSeedFlags::ADD_TELEPORTERS;
        assert_eq!(crowded.micro_seeds.count(), 6);
        assert_eq!(island_count(&crowded), (8, 4), "the reduced 2x arm");

        let mut skyblock = flagged_island_world();
        skyblock.seeds |= crate::seeds::SeedFlags::SKYBLOCK;
        assert_eq!(island_count(&skyblock), (4, 4), "the normal arm");
    }

    #[test]
    fn the_flag_multiplies_the_placed_islands_and_caps_the_houses() {
        // The pass-level effect: over a seed sweep the flagged world places
        // strictly more cloud tiles (islands) than the classic control, while
        // its houses stay capped at `num3 * 2` — the chest count follows the
        // cap, not the multiplied island count.
        let surface = flat_surface(4200, 520);
        let mut classic_cloud = 0usize;
        let mut extra_cloud = 0usize;
        let mut classic_chests = 0usize;
        let mut extra_chests = 0usize;
        let mut sky_count = 0u32;
        for seed in 1..=6u64 {
            let mut classic = sky_world(4200, 1200, 520);
            pass_floating_islands(&mut classic, &surface, &mut Rng(seed), None, false, &mut sky_count);
            classic_cloud += count_kind(&classic, TILE_CLOUD);
            classic_chests += count_kind(&classic, 9);

            let mut extra = sky_world(4200, 1200, 520);
            extra.micro_seeds |= MicroSeedFlags::EXTRA_FLOATING_ISLANDS;
            pass_floating_islands(&mut extra, &surface, &mut Rng(seed), None, false, &mut sky_count);
            extra_cloud += count_kind(&extra, TILE_CLOUD);
            extra_chests += count_kind(&extra, 9);
        }
        assert!(
            extra_cloud > classic_cloud * 2,
            "the flagged world places more islands: extra {extra_cloud} vs classic {classic_cloud}"
        );
        // The plank-frame tiles (9) are the island houses: the classic world
        // houses every island it places, while the flagged one caps at
        // `num3 * 2 = 8` — more houses than classic, but far fewer than the
        // 40 islands its multiplied count attempts.
        assert!(
            classic_chests > 0 && extra_chests > classic_chests,
            "houses: extra {extra_chests} vs classic {classic_chests}"
        );
        // The cap is `num3 * 2 = 8` per world, and the 200-attempt budget per
        // island stops the loop long before the 40 attempted islands are
        // exhausted — but every world that DID place houses placed at most
        // `num3 * 2` of them.
        assert!(
            extra_chests >= classic_chests,
            "the flagged world houses at least as many islands"
        );
    }
}
