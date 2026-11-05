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
/// the island's surface with the cloud-wall interior and the chest.
fn build_house(world: &mut World, cx: i32, top: i32, rng: &mut super::Rng) {
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
    // The chest inside on the floor (`skyIslandHouseCount`'s chest arm).
    let _ = place_chest(world, cx + hw - 2, top - 1);
}

/// The `FloatingIslands` pass (`:12924-13133`): 3-4 islands over the 10-90 %
/// band at `Next(90, worldSurface − 100)`, the ±180 spacing, the spawn
/// exclusion, every third island a sky lake. Returns the island count.
pub(crate) fn pass_floating_islands(
    world: &mut World,
    surface: &[i32],
    rng: &mut super::Rng,
) -> u32 {
    // The C# worlds are 4200+ wide; the tiny test worlds (the sky band
    // thinner than `Next(90, worldSurface − 100)` plus the disc) skip.
    let radius_max = 60i32;
    if (world.world_surface as i32) - 100 - radius_max / 2 <= 90 {
        return 0;
    }
    let count = 3 + (world.width / 4200) as i32;
    let mut placed: Vec<(i32, i32)> = Vec::new();
    let mut lakes = 0u32;
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
            // the middle).
            if (x - world.width / 2).abs() < 150 {
                continue;
            }
            // The ±180 spacing from the placed islands (`:12987-12995`).
            if placed
                .iter()
                .any(|(px, _)| (x - px).abs() < 180)
            {
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
            lake = i % 3 == 2;
            break;
        }
        // `FloatingIsland`'s `Next(80, 120)` is the DIAMETER — the
        // half-width 40-60 and the vertical half-extent ≈ half of that.
        let radius = rng.range(40, 61);
        build_island(world, cx, cy, radius, lake, rng);
        build_house(world, cx, cy - radius / 2, rng);
        placed.push((cx, cy));
        if lake {
            lakes += 1;
        }
    }
    lakes
}
