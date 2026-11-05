//! The biome passes — the compact deterministic ports of
//! `GameContent/Generation/Biomes/*` and the `GenPassNameID` order
//! (`ID/GenPassNameID.cs`): the jungle (`JunglePass`), the evil
//! (`CorruptionAndCrimson`), the glowing mushroom patches
//! (`GlowingMushroomPatches`), the granite and marble caves and the
//! dungeon (`Dungeon`).
//!
//! Tile ids (`TileID.cs`): the jungle grass 60, the mud 59, the mushroom
//! grass 70, the corrupt grass 23, the ebonstone 25, the ebonsand 112, the
//! crimson grass 199, the crimstone 203, the crimsand 234, the ice 161, the
//! granite 368, the marble 367, the dungeon bricks 41/43/44. Wall ids
//! (`WallID.cs`): the jungle 64, the mushroom 74, the granite 180, the
//! marble 178, the ebonstone 3, the crimson 192, the dungeon 7/8/9.
//!
//! Deviations (ANALYSIS "Biome passes (M1.4-WGEN-1)"): the C# passes are
//! shape-driven (`ShapeRunner`/`ShapeFloodFill` chains, `DungeonCrawler`);
//! the port reproduces the vanilla SHAPES with its own deterministic noise
//! and blob machinery — the exact tile-level layout is not 1:1; the jungle
//! shrines, the temple, the pyramids and the chests are the follow-up
//! tasks; the hallow is the hardmode strip (M5-HARDFORCE).

use super::{TILE_DIRT, TILE_GRASS, TILE_SAND, TILE_SNOW, TILE_STONE, WALL_DIRT, WALL_STONE};
use crate::tile::{FLAG_ACTIVE, Tile};
use crate::world::World;

pub(crate) const TILE_MUD: u16 = 59;
pub(crate) const TILE_JUNGLE_GRASS: u16 = 60;
pub(crate) const TILE_MUSHROOM_GRASS: u16 = 70;
pub(crate) const TILE_CORRUPT_GRASS: u16 = 23;
pub(crate) const TILE_EBONSTONE: u16 = 25;
pub(crate) const TILE_EBONSAND: u16 = 112;
pub(crate) const TILE_CRIMSON_GRASS: u16 = 199;
pub(crate) const TILE_CRIMSTONE: u16 = 203;
pub(crate) const TILE_CRIMSAND: u16 = 234;
pub(crate) const TILE_GRANITE: u16 = 368;
pub(crate) const TILE_MARBLE: u16 = 367;
pub(crate) const TILE_DUNGEON_BRICK: u16 = 41;

pub(crate) const WALL_JUNGLE: u16 = 64;
pub(crate) const WALL_MUSHROOM: u16 = 74;
pub(crate) const WALL_GRANITE: u16 = 180;
pub(crate) const WALL_MARBLE: u16 = 178;
pub(crate) const WALL_EBONSTONE: u16 = 3;
pub(crate) const WALL_CRIMSON: u16 = 192;


/// The biome span decisions shared by the passes (the C# `GenVars` slice):
/// the jungle on the west edge, the evil band east of spawn, the dungeon at
/// the snow side's surface.
pub(crate) struct BiomeSpans {
    pub(crate) jungle_start: i32,
    pub(crate) jungle_end: i32,
    pub(crate) evil_start: i32,
    pub(crate) evil_end: i32,
    pub(crate) dungeon_x: i32,
}

pub(crate) fn plan_spans(width: i32, surface: &[i32], beach: i32) -> BiomeSpans {
    // The jungle hugs the west edge after the ocean ramp (`JunglePass`
    // places `jungleOriginX` away from the snow half and the spawn).
    let jungle_start = (beach + 40).max(20);
    let jungle_end = (jungle_start + width / 5).max(jungle_start + 40).min(width / 3).max(jungle_start + 20);
    // The evil band sits between the spawn and the snow half, clear of the
    // spawn plateau (`CorruptionAndCrimson` picks its centre off the
    // spawn/jungle/snow triangle); the tiny worlds collapse it.
    let evil_start = width / 2 + 260;
    let evil_end = (evil_start + width / 12).min(width * 2 / 3 - 120);
    let (evil_start, evil_end) = if evil_end <= evil_start + 40 {
        (0, 0)
    } else {
        (evil_start, evil_end)
    };
    // The dungeon digs at the snow side (`DungeonCrawler`'s entrance spawns
    // at the far edge opposite the jungle).
    let dungeon_x = (width - beach - 60).max(width / 2);
    let _ = surface;
    BiomeSpans {
        jungle_start,
        jungle_end,
        evil_start,
        evil_end,
        dungeon_x,
    }
}

fn put(world: &mut World, x: i32, y: i32, kind: u16, wall: u16) {
    if x < 0 || y < 0 || x >= world.width || y >= world.height {
        return;
    }
    let i = world.index(x, y);
    let t = &mut world.tiles[i];
    t.kind = kind;
    t.flags |= FLAG_ACTIVE;
    t.set_slope(0);
    if y > 1 {
        t.wall = wall;
    }
}

fn active_at(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| t.active())
}

/// The jungle pass (`JunglePass` + `MudCavesToJungleGrass`): the surface
/// band converts to the mud with the jungle grass on top, the underground
/// band (to the rock layer) fills with the mud carrying the jungle walls,
/// and the blob caves inside it gain the mud walls with the water pockets.
pub(crate) fn pass_jungle(
    world: &mut World,
    surface: &[i32],
    spans: &BiomeSpans,
    rock_layer: f64,
    rng: &mut super::Rng,
) {
    for x in spans.jungle_start..spans.jungle_end {
        let top = surface[x as usize];
        for y in top..world.height.min(top + rock_layer as i32 + 40) {
            if !active_at(world, x, y) {
                continue;
            }
            let t = world.tile(x, y).unwrap();
            let is_solid_ground = matches!(t.kind, TILE_DIRT | TILE_STONE | TILE_GRASS | 147 | 53);
            if !is_solid_ground {
                continue;
            }
            let kind = if y == top {
                TILE_JUNGLE_GRASS
            } else {
                TILE_MUD
            };
            put(world, x, y, kind, WALL_JUNGLE);
        }
        // The surface dips: the shallow jungle pools (`SurfaceWaterInJungle`
        // shape — every ~40 columns a 4-8 wide, 2-4 deep water pocket).
        if rng.range(0, 40) == 0 {
            let half = rng.range(2, 5);
            let depth = rng.range(2, 5);
            let base = surface[x as usize];
            for dx in -half..=half {
                let px = x + dx;
                if px < spans.jungle_start || px >= spans.jungle_end {
                    continue;
                }
                let d = depth - dx.abs() / (half.max(1));
                for dy in 1..=d {
                    let py = base + dy;
                    let i = world.index(px, py);
                    world.tiles[i].kind = 0; // the pocket air
                    world.tiles[i].flags &= !FLAG_ACTIVE;
                    world.tiles[i].wall = WALL_JUNGLE;
                }
                if let Some(p) = world.tile_mut(px, base + d + 1) {
                    if p.active() {
                        p.liquid = 255;
                        p.set_liquid_kind(crate::tile::LiquidKind::Water);
                    }
                }
            }
        }
    }
    // The underground mud blobs leak beyond the band edges
    // (`MudCavesToJungleGrass`'s irregular boundary): the random mud pockets
    // around the band's underground half.
    if spans.jungle_end <= spans.jungle_start {
        return;
    }
    let blobs = ((spans.jungle_end - spans.jungle_start) / 24).max(4);
    let blob_top = surface[spans.jungle_start as usize];
    let blob_bottom = (rock_layer as i32).max(blob_top + 20);
    for _ in 0..blobs {
        let cx = rng.range(spans.jungle_start - 40, spans.jungle_end + 40);
        let cy = rng.range(blob_top, blob_bottom);
        let r = rng.range(6, 16);
        for x in (cx - r)..=(cx + r) {
            for y in (cy - r)..=(cy + r) {
                if x < 2 || y < 2 || x >= world.width - 2 || y >= world.height - 2 {
                    continue;
                }
                let dist = ((x - cx) * (x - cx) + (y - cy) * (y - cy)) as f32;
                if dist.sqrt() > r as f32 {
                    continue;
                }
                if !active_at(world, x, y) {
                    continue;
                }
                let t = world.tile(x, y).unwrap();
                if t.kind != TILE_STONE {
                    continue;
                }
                put(world, x, y, TILE_MUD, WALL_JUNGLE);
            }
        }
    }
}

/// The evil pass (`CorruptionAndCrimson`): the surface band of the
/// evil sand/grass over the evil stone, with the chasms — the vertical
/// shafts widening downward — cut into it. `crimson` picks the crimson
/// tile set.
pub(crate) fn pass_evil(
    world: &mut World,
    surface: &[i32],
    spans: &BiomeSpans,
    rng: &mut super::Rng,
    crimson: bool,
) {
    // The tiny-world collapse (see `plan_spans`).
    if spans.evil_end <= spans.evil_start {
        return;
    }
    let (grass, stone, sand, wall) = if crimson {
        (
            TILE_CRIMSON_GRASS,
            TILE_CRIMSTONE,
            TILE_CRIMSAND,
            WALL_CRIMSON,
        )
    } else {
        (
            TILE_CORRUPT_GRASS,
            TILE_EBONSTONE,
            TILE_EBONSAND,
            WALL_EBONSTONE,
        )
    };
    // The surface band: the evil grass over the evil stone + the sand cap.
    for x in spans.evil_start..spans.evil_end {
        let top = surface[x as usize];
        let depth = 8 + (x as f32 * 0.11).sin().abs() as i32 * 4;
        for y in top..(top + depth).min(world.height - 2) {
            if !active_at(world, x, y) {
                continue;
            }
            let t = world.tile(x, y).unwrap();
            if !matches!(t.kind, TILE_DIRT | TILE_STONE | TILE_GRASS) {
                continue;
            }
            let kind = if y == top {
                grass
            } else if y < top + 3 {
                stone
            } else {
                stone
            };
            put(world, x, y, kind, wall);
        }
    }
    // The chasms: 3-4 shafts, each a widening runner pair cut down past the
    // rock layer (`CorruptionAndCrimson`'s chasm shapes).
    let chasms = 3 + (spans.evil_end - spans.evil_start) / 90;
    for _ in 0..chasms {
        let x0 = rng.range(spans.evil_start + 20, spans.evil_end - 20);
        let top = surface[x0 as usize];
        let bottom = (world.rock_layer as i32 + 60).min(world.height - 60);
        let mut x = x0;
        let mut w = 2;
        let mut y = top - 4;
        while y < bottom {
            let dig = w;
            for dx in -dig..=dig {
                put(world, x + dx, y, 0, wall);
            }
            // The stone sheath around the shaft (the evil stone lining).
            put(world, x - dig - 1, y, stone, wall);
            put(world, x + dig + 1, y, stone, wall);
            y += 1;
            if rng.range(0, 3) == 0 {
                x += rng.range(-1, 2);
            }
            if y % 24 == 0 && w < 5 {
                w += 1;
            }
        }
    }
}

/// The glowing mushroom pass (`GlowingMushroomPatches`): 1-2 underground mud
/// pockets near the jungle's depth carry the mushroom grass (the surface of
/// the blob) with the mushroom walls.
pub(crate) fn pass_mushroom(
    world: &mut World,
    spans: &BiomeSpans,
    rock_layer: f64,
    rng: &mut super::Rng,
) {
    if spans.jungle_end <= spans.jungle_start {
        return;
    }
    let patches = 1 + (world.width / 4200).max(1);
    for _ in 0..patches {
        let cx = rng.range(spans.jungle_start, spans.jungle_end + 400);
        let cy_lo = rock_layer as i32;
        let cy_hi = (rock_layer as i32 + 140).min(world.height - 80);
        if cy_hi <= cy_lo {
            return;
        }
        let cy = rng.range(cy_lo, cy_hi);
        let r = rng.range(14, 26);
        for x in (cx - r)..=(cx + r) {
            for y in (cy - r)..=(cy + r) {
                if x < 2 || y < 2 || x >= world.width - 2 || y >= world.height - 2 {
                    continue;
                }
                let dist = ((x - cx) * (x - cx) + (y - cy) * (y - cy)) as f32;
                if dist.sqrt() > r as f32 {
                    continue;
                }
                if !active_at(world, x, y) {
                    continue;
                }
                let t = world.tile(x, y).unwrap();
                let is_solid = matches!(t.kind, TILE_STONE | TILE_MUD | TILE_DIRT);
                if !is_solid {
                    continue;
                }
                // The shell: the mushroom grass on the blob's boundary, the
                // mud inside (`GlowingMushroomPatches`'s dome shape).
                let near_edge = {
                    let e = 2.5f32;
                    let d = dist.sqrt();
                    (r as f32 - d) < e
                };
                if near_edge {
                    put(world, x, y, TILE_MUSHROOM_GRASS, WALL_MUSHROOM);
                } else {
                    put(world, x, y, TILE_MUD, WALL_MUSHROOM);
                }
            }
        }
    }
}

/// The granite and marble cave passes (`Granite`/`Marble`): 1-2 blob caves
/// each in the cavern layer, the walls and the floor converted.
pub(crate) fn pass_granite_marble(
    world: &mut World,
    rock_layer: f64,
    rng: &mut super::Rng,
) {
    for (kind, wall) in [(TILE_GRANITE, WALL_GRANITE), (TILE_MARBLE, WALL_MARBLE)] {
        let caves = 1 + (world.width / 4200).max(1);
        let cy_lo = rock_layer as i32 + 60;
        let cy_hi = world.height - 260;
        if cy_hi <= cy_lo {
            return;
        }
        for _ in 0..caves {
            let cx = rng.range(200, (world.width - 200).max(201));
            let cy = rng.range(cy_lo, cy_hi);
            let r = rng.range(18, 34);
            for x in (cx - r)..=(cx + r) {
                for y in (cy - r)..=(cy + r) {
                    if x < 2 || y < 2 || x >= world.width - 2 || y >= world.height - 2 {
                        continue;
                    }
                    let dist = ((x - cx) * (x - cx) + (y - cy) * (y - cy)) as f32;
                    if dist.sqrt() > r as f32 {
                        continue;
                    }
                    if !active_at(world, x, y) {
                        continue;
                    }
                    let t = world.tile(x, y).unwrap();
                    if t.kind != TILE_STONE {
                        continue;
                    }
                    put(world, x, y, kind, wall);
                }
            }
        }
    }
}

/// The dungeon pass (`Dungeon` + `DungeonCrawler`): the surface entrance box
/// and the descending shaft, then 3 corridor levels of the dungeon brick
/// with the dungeon walls, the corridors carved inward. The brick colour
/// rides the world's dungeon roll (41 blue / 43 green / 44 pink).
pub(crate) fn pass_dungeon(world: &mut World, surface: &[i32], spans: &BiomeSpans, rng: &mut super::Rng) {
    // The brick colour rides the world roll (`Main.cs`'s dungeon roll): the
    // blue 41 / the green 43 / the pink 44 — the walls follow (7/8/9).
    let brick_i = rng.range(0, 3) as usize;
    let brick = [41u16, 43, 44][brick_i];
    let dungeon_wall = [7u16, 8, 9][brick_i];
    let x0 = spans.dungeon_x;
    let top = surface[x0 as usize];
    let bottom = (world.height - 220).min(top + (world.rock_layer as i32) * 2);
    // The entrance tower: a 5-wide brick shell down from the surface.
    let mut y = top - 8;
    while y < top + 30 {
        for dx in -3..=3 {
            put(world, x0 + dx, y, brick, dungeon_wall);
        }
        y += 1;
    }
    // The shaft: carved through the brick column down to the corridors.
    for yy in (top - 7)..(top + 30) {
        for dx in -1..=1 {
            let i = world.index(x0 + dx, yy);
            world.tiles[i] = Tile::default();
            world.tiles[i].wall = dungeon_wall;
        }
    }
    // The corridor levels: a stair-stepping crawl to the bottom, each level
    // a carved tunnel in the brick-filled rock with the dungeon walls.
    let mut cx = x0;
    let mut cy = top + 40;
    let level_gap = ((bottom - cy) / 4).max(30);
    while cy < bottom {
        let end_x = if cx < world.width / 2 {
            cx + rng.range(60, 140)
        } else {
            cx - rng.range(60, 140)
        };
        let step = if end_x > cx { 1 } else { -1 };
        let mut x = cx;
        while x != end_x {
            for dy in 0..4 {
                for dx in 0..2 {
                    let i = world.index(x + dx, cy + dy);
                    if x + dx > 2 && x + dx < world.width - 2 && cy + dy < world.height - 2 {
                        world.tiles[i] = Tile::default();
                        world.tiles[i].wall = dungeon_wall;
                    }
                }
            }
            // The brick lining above and below the tunnel.
            for dy in [-1i32, 4] {
                for dx in 0..2 {
                    put(world, x + dx, cy + dy, brick, dungeon_wall);
                }
            }
            x += step;
        }
        // The vertical drop to the next level.
        let next_cy = (cy + level_gap).min(bottom);
        while cy < next_cy {
            for dx in 0..2 {
                let i = world.index(cx, cy);
                if cx > 2 && cx < world.width - 2 && cy < world.height - 2 {
                    world.tiles[i] = Tile::default();
                    world.tiles[i].wall = dungeon_wall;
                }
                let _ = dx;
                cy += 1;
                break;
            }
        }
        cx = end_x;
    }
}

/// The solid biome-ground tile test for the island clearance scans (the
/// stone/dirt/ice/sand family the islands must not overlap).
pub(crate) fn is_solid_biome_tile(kind: u16) -> bool {
    matches!(
        kind,
        0 | 1 | 53 | 59 | 112 | 147 | 161 | 199 | 203 | 234 | 367 | 368 | 41
    )
}

/// The tree pass extension: the jungle and the evil grass grow trees like
/// the forest grass (`UndergroundJungleTrees` for the surface jungle — the
/// surface jungle uses the regular tree placement on the jungle grass).
pub(crate) fn tree_soil(kind: u16) -> bool {
    matches!(
        kind,
        TILE_GRASS | 147 | TILE_JUNGLE_GRASS | TILE_CORRUPT_GRASS | TILE_CRIMSON_GRASS | TILE_MUSHROOM_GRASS
    )
}
