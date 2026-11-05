//! The biome-detail passes — the compact ports of the `GenPassNameID`
//! decorators: the ocean caves (`OceanCaves` `WorldGen.cs:15167-15191`), the
//! fragile ice over the ice-biome water (`FragileIceOverIceBiomeWater`
//! `:16667-16690`), the underground jungle trees (`UndergroundJungleTrees`
//! `:17840-17856`) and the glowing-mushroom/jungle plants
//! (`GlowingMushroomPlantsUndergroundAndJunglePlants` `:20122-20158`).
//!
//! Tiles (`TileID.cs`): the vines 52, the jungle vines 62, the jungle plants
//! 61, the glowing mushroom 71, the mushroom vines 528, the ice 161, the
//! fragile ice 172? — the C# `MakeWateryIceThing` breaks the ice skin over
//! the water into the open pockets (`WorldGen.cs:16667`'s loop calls it on
//! every non-lava water tile in the ice band; the port reproduces the
//! pocket-carve shape on the ice band's water).
//!
//! Deviations (ANALYSIS "Biome detail passes (M1.4-WGEN-3)"): the ocean
//! caves reuse the port's blob carving aimed at the ocean floor (the C#
//! `oceanCave`'s winding shaft shape is approximated); the underground
//! jungle trees place the simple 5-8 tall trunk + the leaf ball (the C#
//! `GrowUndergroundTree`'s canvas size); the mushroom plants ride the port's
//! mushroom-grass domes from WGEN-1; the hallowed/crimson vine arms wait
//! for the hallow (M5-HARDFORCE).

use super::biomes::{
    TILE_JUNGLE_GRASS, TILE_MUD, TILE_MUSHROOM_GRASS,
};
use super::{TILE_SNOW, TILE_STONE, TILE_DIRT, TILE_GRASS};
use crate::tile::{FLAG_ACTIVE, Tile};
use crate::world::World;

pub(crate) const TILE_VINES: u16 = 52;
pub(crate) const TILE_JUNGLE_VINES: u16 = 62;
pub(crate) const TILE_JUNGLE_PLANTS: u16 = 61;
pub(crate) const TILE_GLOWING_MUSHROOM: u16 = 71;
pub(crate) const TILE_ICE: u16 = 161;

fn active_at(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| t.active())
}

/// The ocean caves (`OceanCaves`, `:15167-15191`): one winding shaft per
/// ocean side (a `Next(3)` gate per side) from the surface down past the
/// ocean floor, carved wide enough to walk. The shaft reuses the port's
/// blob carve aimed at the beach ramps.
pub(crate) fn pass_ocean_caves(
    world: &mut World,
    surface: &[i32],
    beach: i32,
    rng: &mut super::Rng,
) {
    for side in 0..2 {
        if rng.range(0, 3) != 0 {
            continue;
        }
        let x = if side == 0 {
            rng.range(55, 95)
        } else {
            rng.range(world.width - 95, world.width - 55)
        };
        // The surface scan (`:15182-15184`): the first active tile down.
        let mut y = 0;
        while y < world.height - 2 && !active_at(world, x, y) {
            y += 1;
        }
        if y >= world.height - 4 {
            continue;
        }
        // The winding shaft down past the sea floor, then the terminal room.
        let bottom = (surface[x as usize] + 60).min(world.height - 60);
        let mut cx = x;
        let mut cy = y;
        while cy < bottom {
            let w = if cy < y + 12 { 1 } else { 2 };
            for dx in -w..=w {
                let px = cx + dx;
                if px < 2 || px >= world.width - 2 {
                    continue;
                }
                let i = world.index(px, cy);
                world.tiles[i] = Tile::default();
            }
            cy += 1;
            if rng.range(0, 3) == 0 {
                cx += rng.range(-1, 2);
            }
        }
        // The terminal bubble room.
        let r = rng.range(5, 9);
        for px in (cx - r)..=(cx + r) {
            for py in (cy - r)..=(cy + r) {
                if px < 2 || py < 2 || px >= world.width - 2 || py >= world.height - 2 {
                    continue;
                }
                let dist = ((px - cx) * (px - cx) + (py - cy) * (py - cy)) as f32;
                if dist.sqrt() <= r as f32 {
                    let i = world.index(px, py);
                    world.tiles[i] = Tile::default();
                }
            }
        }
        let _ = beach;
    }
}

/// The fragile-ice pockets (`FragileIceOverIceBiomeWater`, `:16667-16690`):
/// the ice band's water tiles break their ice skin — the port carves the
/// 1-3 tile holes into the ice sheet above the water pockets, so the frozen
/// lakes read as broken.
pub(crate) fn pass_fragile_ice(world: &mut World, snow_start: i32, rng: &mut super::Rng) {
    let y0 = world.world_surface as i32;
    let y1 = world.height - 100;
    for x in 10..world.width - 10 {
        if x < snow_start {
            continue;
        }
        for y in y0..y1.min(world.height - 2) {
            let Some(t) = world.tile(x, y) else {
                continue;
            };
            if t.liquid == 0 || t.active() {
                continue;
            }
            // The water tile under an ice sheet: the skin opens.
            if !active_at(world, x, y - 1) {
                continue;
            }
            let above = world.tile(x, y - 1).unwrap();
            if above.kind != TILE_ICE || rng.range(0, 4) != 0 {
                continue;
            }
            let width = rng.range(1, 4);
            for dx in 0..width {
                let px = x + dx;
                if px >= world.width - 2 {
                    break;
                }
                if world
                    .tile(px, y - 1)
                    .is_some_and(|t| t.active() && t.kind == TILE_ICE)
                {
                    let i = world.index(px, y - 1);
                    world.tiles[i] = Tile::default();
                }
            }
        }
    }
}

/// The underground jungle trees (`UndergroundJungleTrees`, `:17840-17856`):
/// the `Next(10)` per column over the mud band — the 5-8 tall trunks with
/// the leaf ball, rooted in the mud.
pub(crate) fn pass_underground_jungle_trees(
    world: &mut World,
    spans: &super::biomes::BiomeSpans,
    rock_layer: f64,
    rng: &mut super::Rng,
) {
    if spans.jungle_end <= spans.jungle_start {
        return;
    }
    for x in spans.jungle_start..spans.jungle_end {
        for y in (world.world_surface as i32 - 1)..((rock_layer as i32 + 60).min(world.height - 60)) {
            if rng.range(0, 10) != 0 {
                continue;
            }
            // The mud root with the air above.
            if !world
                .tile(x, y)
                .is_some_and(|t| t.active() && t.kind == TILE_MUD)
            {
                continue;
            }
            if active_at(world, x, y - 1) || active_at(world, x, y - 2) {
                continue;
            }
            // The trunk 5-8 tall.
            let height = rng.range(5, 9);
            let mut clear = true;
            for dy in 1..=height + 1 {
                if active_at(world, x, y - dy) {
                    clear = false;
                    break;
                }
            }
            if !clear {
                continue;
            }
            for dy in 1..=height {
                let i = world.index(x, y - dy);
                world.tiles[i] = Tile {
                    kind: 5, // `TileID.Trees` — the wood trunk
                    ..Default::default()
                };
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
            // The leaf ball around the top.
            let top = y - height;
            for dx in -2..=2 {
                for dy in -2..=1 {
                    let px = x + dx;
                    let py = top + dy;
                    if px < 2 || py < 2 || px >= world.width - 2 || py >= world.height - 2 {
                        continue;
                    }
                    if active_at(world, px, py) {
                        continue;
                    }
                    let dist = (dx * dx + dy * dy) as f32;
                    if dist.sqrt() > 2.6 {
                        continue;
                    }
                    let i = world.index(px, py);
                    world.tiles[i] = Tile {
                        kind: 6, // `TileID.TreeBranches` — the canopy
                        ..Default::default()
                    };
                    world.tiles[i].flags |= FLAG_ACTIVE;
                }
            }
        }
    }
}

/// The glowing-mushroom and jungle plants
/// (`GlowingMushroomPlantsUndergroundAndJunglePlants`, `:20122-20158`): the
/// mushroom grass grows the glowing mushrooms (71) with the triple
/// tree-retry, the jungle grass grows the plants (61) on every free cell.
pub(crate) fn pass_mushroom_and_jungle_plants(world: &mut World) {
    for x in 5..world.width - 5 {
        for y in (world.world_surface as i32).min(5)..world.height - 5 {
            let Some(t) = world.tile(x, y) else {
                continue;
            };
            if !t.active() || y < 5 {
                continue;
            }
            if world.tile(x, y - 1).is_some_and(|t| t.active()) {
                continue;
            }
            if y >= world.world_surface as i32 && t.kind == TILE_MUSHROOM_GRASS {
                // `:20132-20142`: the glowing mushroom on the free cell (the
                // triple tree retry collapses to the plant placement).
                let i = world.index(x, y - 1);
                world.tiles[i] = Tile {
                    kind: TILE_GLOWING_MUSHROOM,
                    frame_x: 0,
                    ..Default::default()
                };
                world.tiles[i].flags |= FLAG_ACTIVE;
            } else if t.kind == TILE_JUNGLE_GRASS {
                // `:20147-20150`: the jungle plants on every free grass cell.
                let i = world.index(x, y - 1);
                world.tiles[i] = Tile {
                    kind: TILE_JUNGLE_PLANTS,
                    frame_x: ((x % 3) * 18) as i16,
                    ..Default::default()
                };
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
    }
}

/// The vines (`ActionVines` + the grass-growth vine arms): the vines hang
/// from the grass overhangs — the jungle vines 62 under the jungle grass,
/// the regular vines 52 under the forest grass, both growing down to the
/// length caps (`Next(2..70)` per family — the port caps 12 for the
/// surface vines and 20 for the jungle).
pub(crate) fn pass_vines(world: &mut World, rng: &mut super::Rng) {
    for x in 5..world.width - 5 {
        for y in 5..world.height - 5 {
            let Some(t) = world.tile(x, y) else {
                continue;
            };
            if !t.active() {
                continue;
            }
            if world.tile(x, y + 1).is_some_and(|n| n.active()) {
                continue;
            }
            let (vine, cap) = match t.kind {
                TILE_JUNGLE_GRASS => (TILE_JUNGLE_VINES, 20),
                TILE_GRASS => (TILE_VINES, 12),
                TILE_MUSHROOM_GRASS => (528, 12),
                _ => continue,
            };
            // The overhang only: the grass cell must hang (the air below).
            if rng.range(0, 3) != 0 {
                continue;
            }
            let length = rng.range(2, cap);
            for dy in 1..=length {
                let py = y + dy;
                if py >= world.height - 2 || active_at(world, x, py) {
                    break;
                }
                let i = world.index(x, py);
                world.tiles[i] = Tile {
                    kind: vine,
                    frame_x: 0,
                    ..Default::default()
                };
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
    }
}

/// `ExposedGemsInIceBiome` (`WorldGen.cs:20697-20735`): `maxTilesX · 0.25`
/// attempts — the y between `(worldSurface + rockLayer) / 2` and the lava
/// line, the x in the snow band, the anchor tile from the snow/ice family
/// (147/161/162/224); the gem patch (the tile 178, the styles 0-5 weighted
/// `Next(12)`: <3 → 0, <6 → 1, <8 → 2, <10 → 3, ≥11 → 5, else 4) fills the
/// `Next(1,4)`-half-sized rectangle of the free cells.
pub(crate) fn pass_exposed_gems(
    world: &mut World,
    snow_start: i32,
    rock_layer: f64,
    rng: &mut super::Rng,
) {
    let attempts = (world.width as f64 * 0.25) as i32;
    let y_lo = ((world.world_surface + rock_layer) / 2.0) as i32;
    let y_hi = (world.height - 170).min(world.height - 2);
    if y_hi <= y_lo {
        return;
    }
    for _ in 0..attempts {
        let y = rng.range(y_lo, y_hi);
        // The snow band's span at this row: the port's snow band is the
        // eastern third (the fill's `snow_start`), the gems scan inside it.
        let x = rng.range(snow_start.max(40), world.width - 40);
        let Some(t) = world.tile(x, y) else {
            continue;
        };
        if !t.active() || !matches!(t.kind, 147 | 161 | 162 | 224) {
            continue;
        }
        // The patch rectangle: the `Next(1, 4)` half-sizes.
        let left = rng.range(1, 4);
        let right = rng.range(1, 4);
        let up = rng.range(1, 4);
        let down = rng.range(1, 4);
        // The style weights (`:20718-20724`).
        let roll = rng.range(0, 12);
        let style = if roll < 3 {
            0
        } else if roll < 6 {
            1
        } else if roll < 8 {
            2
        } else if roll < 10 {
            3
        } else if roll >= 11 {
            5
        } else {
            4
        };
        for px in (x - left)..(x + right) {
            for py in (y - up)..(y + down) {
                if px < 40 || py < 40 || px >= world.width - 40 || py >= world.height - 40 {
                    continue;
                }
                if world.tile(px, py).is_some_and(|t| !t.active()) {
                    let i = world.index(px, py);
                    let t = &mut world.tiles[i];
                    t.kind = 178;
                    t.frame_x = (style * 18) as i16;
                    t.frame_y = 0;
                    t.flags |= FLAG_ACTIVE;
                }
            }
        }
    }
}

const _: (u16, u16, u16) = (TILE_SNOW, TILE_STONE, TILE_DIRT);
