//! The surface height map and the terrain fill.
//!
//! [`surface_height_map`] shapes the per-column surface row (hills, the spawn
//! plateau, desert dunes, the flattened lake span and the ocean beach ramps);
//! [`fill_terrain`] paints the dirt/sand/stone/ash columns under it and derives
//! the `world_surface` / `rock_layer` anchors from `TerrainPass.ApplyPass`.

use super::noise::Noise;
use super::{
    TILE_ASH, TILE_DIRT, TILE_GRASS, TILE_HARDENED_SAND, TILE_SAND, TILE_SANDSTONE, TILE_SNOW,
    TILE_STONE, WALL_DIRT, WALL_SANDSTONE, WALL_STONE,
};
use crate::tile::{FLAG_ACTIVE, Tile};
use crate::world::World;

/// Everything the surface-height pass computes that the later passes consume.
pub(crate) struct SurfaceMap {
    /// Per-column surface row; the lake pass lowers its columns to the carved floor.
    pub(crate) heights: Vec<i32>,
    pub(crate) spawn_center: i32,
    pub(crate) desert_enabled: bool,
    pub(crate) desert_start: i32,
    pub(crate) desert_end: i32,
    pub(crate) lake_center: i32,
    pub(crate) lake_half: i32,
    pub(crate) lake_level: i32,
    pub(crate) beach: i32,
    pub(crate) sea_level: i32,
}

/// Layer anchors derived from the finished surface: `world_surface`/`rock_layer`
/// land on the world, the rest scopes the cave, ore and lava passes.
pub(crate) struct Layers {
    pub(crate) avg_surface: f64,
    pub(crate) world_surface: f64,
    pub(crate) rock_layer: f64,
    pub(crate) hell_top: i32,
}

/// Shapes the per-column surface row before any tile is painted.
pub(crate) fn surface_height_map(width: i32, height: i32, noise: &Noise) -> SurfaceMap {
    // --- surface height map -------------------------------------------------
    let avg = height as f32 * 0.30;
    let spawn_center = width / 2;
    // An inland desert west of spawn (small worlds skip it). The band is kept clear
    // of the spawn plateau and the lake.
    let desert_half = 170;
    let desert_center = spawn_center - 380;
    let desert_enabled = width >= 2000;
    let desert_start = if desert_enabled {
        (desert_center - desert_half).max(200)
    } else {
        0
    };
    let desert_end = if desert_enabled {
        (desert_center + desert_half).min(spawn_center - 120)
    } else {
        0
    };
    let mut surface = vec![0i32; width as usize];
    for x in 0..width {
        let fx = x as f32;
        // Large rolling hills plus small roughness.
        let hills = noise.fbm(fx * 0.0028, 11.0, 4, 0.5);
        let detail = noise.fbm(fx * 0.018, 71.0, 3, 0.5);
        let mut h = avg + hills * 42.0 + detail * 5.0;
        // Gentle spawn plateau so the player starts on flat ground.
        let dist = (x - spawn_center).abs() as f32;
        if dist < 45.0 {
            let t = 1.0 - dist / 45.0;
            h = h * (1.0 - t * 0.7) + avg * t * 0.7;
        }
        surface[x as usize] = (h.round() as i32).clamp(80, height - 340);
    }
    // Flatten the desert into smooth dunes so it reads differently from the forest.
    if desert_enabled {
        for x in (desert_start - 30)..(desert_end + 30) {
            if x < 0 || x >= width {
                continue;
            }
            let d = (x - desert_center).abs();
            let blend = if d <= desert_half {
                1.0
            } else {
                1.0 - (d - desert_half) as f32 / 30.0
            };
            let dune =
                avg + (x as f32 * 0.045).sin() * 5.0 + noise.value(x as f32 * 0.02, 9.0) * 4.0;
            surface[x as usize] = (surface[x as usize] as f32 * (1.0 - blend)
                + dune.clamp(80.0, (height - 340) as f32) * blend)
                .round() as i32;
        }
    }
    // Limit adjacent height differences to one tile so the whole surface can be
    // expressed with slope tiles instead of vertical stair-steps.
    for _ in 0..3 {
        for x in 1..width as usize {
            let prev = surface[x - 1];
            surface[x] = surface[x].clamp(prev - 1, prev + 1);
        }
        for x in (0..width as usize - 1).rev() {
            let next = surface[x + 1];
            surface[x] = surface[x].clamp(next - 1, next + 1);
        }
    }

    // A small forest lake near spawn, one tile below the local ground (like
    // `WorldGen.SonOfLakinater`): the span is flattened to the water line here and the
    // bowl itself is carved into the finished terrain by the `--- liquids ---` pass, so
    // caves/ore passes cannot punch through the floor.
    let lake_center = (spawn_center + 150).min(width - 80);
    let lake_half = 32;
    let lake_level = (surface[lake_center as usize] + 1).clamp(90, height - 300);
    // The two rim columns are flattened too, so the shore grass sits exactly at the
    // water line and no backdrop shows through a step.
    for dx in -(lake_half + 2)..=(lake_half + 2) {
        let x = lake_center + dx;
        if x < 2 || x >= width - 2 {
            continue;
        }
        surface[x as usize] = lake_level;
    }

    // Ocean beaches at both edges: a sand ramp down from the sea level into deep water.
    let beach = 140;
    let sea_level = (avg as i32 + 4).clamp(90, height - 300);
    let inner = beach + 30;
    for (x, d) in (0..inner)
        .map(|x| (x, x))
        .chain((width - inner..width).map(|x| (x, width - 1 - x)))
    {
        if x < 0 || x >= width {
            continue;
        }
        let t = 1.0 - d as f32 / inner as f32;
        let ramp = sea_level + (t * t * 30.0) as i32;
        if d < beach {
            surface[x as usize] = ramp;
        } else {
            let k = (inner - d) as f32 / 30.0;
            surface[x as usize] =
                (surface[x as usize] as f32 * (1.0 - k) + ramp as f32 * k).round() as i32;
        }
    }

    SurfaceMap {
        heights: surface,
        spawn_center,
        desert_enabled,
        desert_start,
        desert_end,
        lake_center,
        lake_half,
        lake_level,
        beach,
        sea_level,
    }
}

/// Paints the terrain columns under the surface row and derives the layer anchors.
pub(crate) fn fill_terrain(
    world: &mut World,
    width: i32,
    height: i32,
    noise: &Noise,
    map: &SurfaceMap,
) -> Layers {
    // --- terrain fill -------------------------------------------------------
    let surface = &map.heights;
    let beach = map.beach;
    let desert_enabled = map.desert_enabled;
    let desert_start = map.desert_start;
    let desert_end = map.desert_end;
    let in_desert = |x: i32| desert_enabled && x >= desert_start && x < desert_end;
    // Mean surface height, used as the anchor for cave/ore placement below.
    let avg_surface = surface.iter().sum::<i32>() as f64 / width as f64;
    // `TerrainPass.ApplyPass`: `Main.worldSurface = (int)(max(surfaceY) + 25.0)` — the
    // *deepest* column of the pre-carve surface, 25 tiles below it, not the mean. The
    // underground backdrop (`DrawBackground_DirtBackground`) and the surface parallax
    // (`DrawSurfaceBG`) are both anchored to this line, so it has to track the lowest
    // terrain or the dirt sheet ends up floating above the ground.
    let surface_max = surface.iter().max().copied().unwrap_or(0) as f64;
    let world_surface = surface_max + 25.0;
    // `TerrainPass.ApplyPass`: `Main.rockLayer = worldSurface + (int)((num8 - worldSurface) / 6) * 6`
    // with `num8` the deepest per-column rock start.
    let rock_raw = (height as f64 * 0.43).max(world_surface + 65.0);
    let rock_layer = world_surface + ((rock_raw - world_surface) / 6.0).floor() * 6.0;
    let hell_top = (height - 170) as i32;
    // Eastern third of the world is a snow biome.
    let snow_start = width * 2 / 3;

    for x in 0..width {
        let top = surface[x as usize];
        let dirt_depth = 16 + (noise.value(x as f32 * 0.05, 3.0) * 6.0) as i32;
        for y in 0..height {
            let i = world.index(x, y);
            let mut tile = Tile::default();
            let desert = in_desert(x);
            if y >= top {
                if y == top {
                    tile.kind = if x < beach + 24 || x >= width - beach - 24 {
                        TILE_SAND
                    } else if x >= snow_start {
                        TILE_SNOW
                    } else if desert {
                        TILE_SAND
                    } else {
                        TILE_GRASS
                    };
                } else if y < top + dirt_depth {
                    tile.kind = if desert { TILE_SAND } else { TILE_DIRT };
                } else if desert && y < top + dirt_depth + 14 {
                    tile.kind = TILE_HARDENED_SAND;
                } else if desert && y < top + dirt_depth + 30 {
                    tile.kind = TILE_SANDSTONE;
                } else if y < hell_top {
                    tile.kind = TILE_STONE;
                } else {
                    tile.kind = TILE_ASH;
                }
                tile.flags |= FLAG_ACTIVE;
                if y > top + 2 {
                    tile.wall = if desert {
                        WALL_SANDSTONE
                    } else if (y as f64) < rock_layer {
                        WALL_DIRT
                    } else {
                        WALL_STONE
                    };
                }
            }
            world.tiles[i] = tile;
        }
    }

    Layers {
        avg_surface,
        world_surface,
        rock_layer,
        hell_top,
    }
}
