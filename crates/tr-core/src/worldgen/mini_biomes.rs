//! The micro-biomes — the shape-level ports of the `GameContent/Biomes`
//! micro classes: `CaveHouseBiome` (the underground cabins), `CampsiteBiome`
//! (the desert campsites), `SpikePitBiome` (the jungle spike pits) and
//! `EnchantedSwordBiome` (the sword shrines).
//!
//! Source map (line numbers into `decompiled/Terraria/GameContent/Biomes/`):
//! - `CaveHouseBiome.Place` (`:34-92`): the 25-tile wire/chest exclusion,
//!   the `HouseUtils.CreateBuilder` room — the port carves the wood-framed
//!   room with the platform, the chest and the torches in the underground
//!   dirt/stone band (the C#'s `HouseBuilder` variant walls = the
//!   shape-level approximation)
//! - `CampsiteBiome.Place` (`:9-55`): the disc `Next(6, 10)` radius, the
//!   solid-count gate, the tile 196 + `Next(4)` (the tent) replacing the
//!   sand family {53, 396, 397, 404}
//! - `SpikePitBiome.Place` (`:10-57`): the stone (1) pit — the 4+4+4
//!   shrinking circles offset down, the blob blotches; the port drops the
//!   spike tile placement (the jungle spikes = the follow-up)
//! - `EnchantedSwordBiome.Place` (`:19-113`): the 0/1 tile-scan gate (the
//!   1250 air/dirt cells in the 50×50 box), the 55-tile depth gate, the
//!   no-sand check, the 1-wide shaft down to the shrine chamber; the port
//!   places the shaft + the chamber with the sword item tile 197 (the C#'s
//!   sword sprite tile — the breakable item drop = the follow-up)
//!
//! Deviations (ANALYSIS "Micro-biomes (M1.4-WGEN-8)"): the C#'s
//! `HouseBuilder` room variants (the granite/marble/mushroom/ice/jungle
//! chest chances) are the shape-level approximation; the spike tiles and
//! the sword item drop are the follow-ups; the structure-map exclusions
//! are the per-pass neighbour scans.

use super::structures::place_chest;
use crate::tile::{FLAG_ACTIVE, Tile};
use crate::world::World;

fn active_at(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| t.active())
}

fn clear(world: &mut World, x: i32, y: i32, wall: u16) {
    if x < 2 || y < 2 || x >= world.width - 2 || y >= world.height - 2 {
        return;
    }
    let i = world.index(x, y);
    world.tiles[i] = Tile::default();
    world.tiles[i].wall = wall;
}

fn put(world: &mut World, x: i32, y: i32, kind: u16) {
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

/// `CaveHouseBiome` (`:34-92`): the underground cabin — the wood frame, the
/// carved interior with the background walls, the wood platform, the chest
/// and the torches. The chest-chance variants (the ice/jungle/granite/
/// marble/mushroom/desert) resolve to the plain gold chest (the port's
/// chests hold no inventories yet).
pub(crate) fn pass_cave_houses(
    world: &mut World,
    rock_layer: f64,
    rng: &mut super::Rng,
) -> u32 {
    let count = 8 + rng.range(0, 5) as i32;
    let mut placed = 0u32;
    let y_lo = world.world_surface as i32 + 40;
    let y_hi = (rock_layer as i32 + 260).min(world.height - 120);
    if y_hi <= y_lo {
        return placed;
    }
    for _ in 0..count {
        let x = rng.range(60, world.width - 60);
        // The spawn exclusion.
        if (x - world.width / 2).abs() < 120 {
            continue;
        }
        let y = rng.range(y_lo, y_hi);
        let hw = rng.range(5, 9);
        let hh = rng.range(3, 5);
        // `:38-52`: the 25-tile exclusion — the chests and the wires nearby
        // reject the spot.
        let mut blocked = false;
        for dx in -25..=25i32 {
            for dy in -25..=25i32 {
                if world
                    .tile(x + dx, y + dy)
                    .is_some_and(|t| t.active() && t.kind == 21)
                {
                    blocked = true;
                    break;
                }
            }
            if blocked {
                break;
            }
        }
        if blocked {
            continue;
        }
        // The ground must be the dirt/stone at the spot.
        if !world
            .tile(x, y)
            .is_some_and(|t| t.active() && matches!(t.kind, 0 | 1))
        {
            continue;
        }
        // The frame: the wood box.
        for px in (x - hw)..=(x + hw) {
            for py in (y - hh)..=(y + hh) {
                put(world, px, py, 27); // `TileID.WoodBlock`? — the wood frame tile
            }
        }
        // The interior: carved with the wood walls.
        for px in (x - hw + 1)..=(x + hw - 1) {
            for py in (y - hh + 1)..=(y + hh - 1) {
                clear(world, px, py, 9);
            }
        }
        // The floor: the wood platform row (the tile 19 — the platform).
        for px in (x - hw + 1)..=(x + hw - 1) {
            let py = y + hh / 2;
            put(world, px, py, 19);
        }
        // The torches on the interior walls.
        for tx in [x - hw + 2, x + hw - 2] {
            let ty = y - hh + 2;
            let i = world.index(tx, ty);
            if !world.tiles[i].active() {
                world.tiles[i] = Tile {
                    kind: 4,
                    ..Default::default()
                };
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        // The chest on the floor.
        let _ = place_chest(world, x - 1, y + hh / 2 - 1);
        placed += 1;
    }
    placed
}

/// `CampsiteBiome` (`:9-55`): the desert campsite — the disc of the radius
/// `Next(6, 10)` replacing the sand family {53, 396, 397, 404} with the
/// tent tiles (196 + `Next(4)`), gated by the solid-count check.
pub(crate) fn pass_campsites(world: &mut World, rng: &mut super::Rng) -> u32 {
    let count = 4 + rng.range(0, 4) as i32;
    let mut placed = 0u32;
    for _ in 0..count {
        let x = rng.range(100, world.width - 100);
        let y = rng.range(world.world_surface as i32, (world.world_surface as i32 + 60).min(world.height - 60));
        // The disc solid-count gate (`:11-17`): the solid cells must be
        // within 5 of the total.
        let mut solid = 0;
        let mut total = 0;
        for dx in -10..=10 {
            for dy in -10..=10 {
                total += 1;
                if active_at(world, x + dx, y + dy) {
                    solid += 1;
                }
            }
        }
        if solid < total - 5 {
            continue;
        }
        let radius = rng.range(6, 10);
        let tent = 196 + rng.range(0, 4) as u16;
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                let px = x + dx;
                let py = y + dy;
                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                if dist > radius as f32 {
                    continue;
                }
                if let Some(t) = world.tile(px, py) {
                    if t.active() && matches!(t.kind, 53 | 396 | 397 | 404) {
                        put(world, px, py, tent);
                    }
                }
            }
        }
        placed += 1;
    }
    placed
}

/// `SpikePitBiome` (`:10-57`): the jungle stone pit — the shrinking
/// circle blobs offset down (the C#'s 4+4+4 circle walk); the spike tile
/// placement is the follow-up.
pub(crate) fn pass_spike_pits(
    world: &mut World,
    spans: &super::biomes::BiomeSpans,
    rock_layer: f64,
    rng: &mut super::Rng,
) -> u32 {
    if spans.jungle_end <= spans.jungle_start {
        return 0;
    }
    let count = 3 + rng.range(0, 4) as i32;
    let mut placed = 0u32;
    let y_lo = rock_layer as i32;
    let y_hi = (rock_layer as i32 + 200).min(world.height - 120);
    if y_hi <= y_lo {
        return placed;
    }
    for _ in 0..count {
        let cx_lo = spans.jungle_start + 20;
        let cx_hi = (spans.jungle_end - 20).max(cx_lo + 1);
        let cx = rng.range(cx_lo, cx_hi);
        let cy = rng.range(y_lo, y_hi);
        // The 4 shrinking circles offset down (the pit walls).
        let mut r0 = rng.range(8, 10);
        for step in 0..4 {
            let r = r0 + step;
            let oy = 5 * step + 5;
            for dx in -r..=r {
                for dy in -r..=r {
                    let dist = ((dx * dx + dy * dy) as f32).sqrt();
                    if dist <= r as f32 {
                        put(world, cx + dx, cy + oy + dy, 1);
                    }
                }
            }
            r0 -= 1;
        }
        // The pit hollow: the inner circles carved.
        let mut r1 = rng.range(6, 7);
        for step in 0..4 {
            let r = r1 + step;
            let oy = 2 * step + 12;
            for dx in -r..=r {
                for dy in -r..=r {
                    let dist = ((dx * dx + dy * dy) as f32).sqrt();
                    if dist <= r as f32 {
                        clear(world, cx + dx, cy + oy + dy, super::biomes::WALL_JUNGLE);
                    }
                }
            }
            r1 -= 1;
        }
        // The spikes (48) on the inner rim touching the air (`:53-55`: the
        // IsTouchingAir cells, the checkerboard 2 spacing).
        for dx in -r1..=r1 {
            for dy in -r1..=r1 {
                let px = cx + dx;
                let py = cy + 12 + dy;
                if (dx + dy) % 2 != 0 {
                    continue;
                }
                if active_at(world, px, py) {
                    continue;
                }
                // The cell touches the pit's stone (the wall or the floor).
                let touching = [
                    (px - 1, py),
                    (px + 1, py),
                    (px, py - 1),
                    (px, py + 1),
                ]
                .iter()
                .any(|(nx, ny)| active_at(world, *nx, *ny));
                if touching {
                    put(world, px, py, 48);
                }
            }
        }
        placed += 1;
    }
    placed
}

/// `EnchantedSwordBiome` (`:19-113`): the sword shrine — the 1-wide shaft
/// down from the surface into the buried chamber with the sword tile (197,
/// the C#'s sword sprite tile; the breakable item drop = the follow-up).
pub(crate) fn pass_sword_shrines(
    world: &mut World,
    surface: &[i32],
    rock_layer: f64,
    desert: (i32, i32),
    rng: &mut super::Rng,
) -> u32 {
    let count = 2 + rng.range(0, 3) as i32;
    let mut placed = 0u32;
    for _ in 0..count {
        let x = rng.range(100, world.width - 100);
        // The desert band skips (the C#'s tile-scan gate `:24-30` requires
        // 1250 of the tiles 0/1 in the 50×50 box — the desert's sand 53
        // fails it, so the shrines never generate there).
        if x >= desert.0 && x < desert.1 {
            continue;
        }
        // The depth gate (`:32-38`): the origin 55+ tiles under the
        // surface.
        let ground = surface[x as usize];
        if ground < 55 {
            continue;
        }
        let shaft_top = ground - rng.range(10, 30);
        // The 1-wide shaft down to the chamber.
        let depth = rng.range(8, 14);
        let chamber_y = shaft_top + depth;
        if chamber_y as f64 > rock_layer + 200.0 {
            continue;
        }
        for py in shaft_top..chamber_y {
            clear(world, x, py, 0);
        }
        // The chamber: the 7×5 hollow with the stone frame.
        for px in (x - 3)..=(x + 3) {
            for py in chamber_y..(chamber_y + 5) {
                if px == x - 3 || px == x + 3 || py == chamber_y + 4 {
                    put(world, px, py, 1);
                } else {
                    clear(world, px, py, 0);
                }
            }
        }
        // The sword tile in the chamber's centre (`:100-108`): the
        // `ChanceOfRealSword` config value is 1.0 — the real sword tile 187
        // always (the break drops the Enchanted Sword item 989); the fake
        // tile 186 arm is dead with this config.
        put(world, x, chamber_y + 2, 187);
        placed += 1;
    }
    placed
}
