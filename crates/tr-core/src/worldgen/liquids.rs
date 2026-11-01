//! Liquid placement: the forest lake, the ocean shores and lava puddles.

use super::TILE_DIRT;
use super::noise::Rng;
use super::terrain::{Layers, SurfaceMap};
use crate::tile::{FLAG_ACTIVE, FLAG_LIQUID_HONEY, FLAG_LIQUID_LAVA, Tile};
use crate::tiledata as td;
use crate::world::World;

/// Depth of the lake floor below the water line: 1 tile at the shore, 6 at the
/// centre. Cosine profile, mirroring the shallow bowls `WorldGen.SonOfLakinater` carves.
pub(crate) fn lake_floor_depth(dx: i32, half: i32) -> i32 {
    let t = dx as f32 / half as f32;
    let c = (t * std::f32::consts::FRAC_PI_2).cos();
    1 + (5.0 * c * c).round() as i32
}

/// Carves the lake bowl into the finished terrain, floods it and the ocean shores,
/// then drips lava onto cave floors. Lowers the lake columns of the surface map to
/// the carved floor so the later surface passes dress the bowl, not the old bank.
pub(crate) fn place_liquids(
    world: &mut World,
    rng: &mut Rng,
    map: &mut SurfaceMap,
    layers: &Layers,
    width: i32,
) {
    // --- liquids ------------------------------------------------------------
    let lake_center = map.lake_center;
    let lake_half = map.lake_half;
    let lake_level = map.lake_level;
    let sea_level = map.sea_level;
    let beach = map.beach;
    let avg_surface = layers.avg_surface;
    let hell_top = layers.hell_top;
    let surface = &mut map.heights;
    // Lake: carve a proper bowl into the finished terrain, then fill it to the water
    // line. `WorldGen.SonOfLakinater` clears the tiles below its start row and only
    // places liquid in rows at/below it; the floor here follows `lake_floor_depth`.
    for dx in -lake_half..=lake_half {
        let x = lake_center + dx;
        if x < 1 || x >= width - 1 {
            continue;
        }
        let floor = lake_level + lake_floor_depth(dx, lake_half);
        let top = surface[x as usize];
        if top < floor {
            // Lower the ground: clear every tile from the old surface to the floor.
            // Above the water line the cut is open air, so the old dirt wall goes too
            // (surface lakes in the original have no wall behind the exposed bank).
            for y in top..floor {
                let i = world.index(x, y);
                let wall = if y < lake_level {
                    0
                } else {
                    world.tiles[i].wall
                };
                world.tiles[i] = Tile {
                    wall,
                    ..Default::default()
                };
            }
        } else if top > floor {
            // Natural ground sits below the water line (beach ramp overlap); raise a
            // dirt floor so the bowl still holds water.
            for y in floor..top {
                let i = world.index(x, y);
                let wall = world.tiles[i].wall;
                world.tiles[i] = Tile {
                    kind: TILE_DIRT,
                    wall,
                    flags: FLAG_ACTIVE,
                    ..Default::default()
                };
            }
        }
        // Bowl floor: dirt, solid, level (slopes are applied later from `surface`).
        let i = world.index(x, floor);
        let wall = world.tiles[i].wall;
        world.tiles[i] = Tile {
            kind: TILE_DIRT,
            wall,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
        surface[x as usize] = floor;
        // Water only in air cells at/below the water line; never above `lake_level`.
        for y in lake_level..floor {
            let i = world.index(x, y);
            if !world.tiles[i].active() {
                world.tiles[i].liquid = 255;
                world.tiles[i].flags &= !(FLAG_LIQUID_LAVA | FLAG_LIQUID_HONEY);
            }
        }
    }
    // Ocean water at both shores.
    for shore_x in (0..beach).chain(width - beach..width) {
        if shore_x < 0 || shore_x >= width {
            continue;
        }
        let top = surface[shore_x as usize];
        for y in sea_level..top {
            let i = world.index(shore_x, y);
            if !world.tiles[i].active() {
                world.tiles[i].liquid = 255;
                world.tiles[i].flags &= !(FLAG_LIQUID_LAVA | FLAG_LIQUID_HONEY);
            }
        }
    }
    // Lava puddles on cave floors.
    for _ in 0..700 {
        let x = rng.range(10, width - 10);
        let y = rng.range((avg_surface as i32) + 110, hell_top - 4);
        let floor_solid = world
            .tile(x, y + 1)
            .is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind));
        let air_here = world
            .tile(x, y)
            .is_some_and(|t| !t.active() && t.liquid == 0);
        if !floor_solid || !air_here {
            continue;
        }
        let depth = rng.range(1, 4);
        for d in 0..depth {
            let yy = y - d;
            if yy < 1 {
                break;
            }
            let i = world.index(x, yy);
            if world.tiles[i].active() || world.tiles[i].liquid != 0 {
                break;
            }
            world.tiles[i].liquid = 255;
            world.tiles[i].flags |= FLAG_LIQUID_LAVA;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::lake_floor_depth;
    use crate::tiledata as td;
    use crate::world::World;
    use crate::worldgen::{GenOptions, TILE_DIRT, TILE_GRASS, generate};

    /// `lake_center`, `lake_half` and the water line (first row with liquid).
    fn lake_params(w: &World) -> (i32, i32, i32) {
        let center = (w.width / 2 + 150).min(w.width - 80);
        let level = (0..w.height)
            .find(|&y| w.tile(center, y).unwrap().liquid > 0)
            .expect("lake has no water at its centre");
        (center, 32, level)
    }

    #[test]
    fn lake_water_stays_at_or_below_the_water_line() {
        let mut w = generate(&GenOptions::default());
        w.compute_column_surface();
        let (center, half, level) = lake_params(&w);
        // The rim must stand above the water line so the lake is not a perched pool.
        let rim_low = [center - half - 1, center + half + 1]
            .iter()
            .map(|&x| w.surface_of(x))
            .min()
            .unwrap();
        assert!(rim_low <= level, "lake rim is below the water line");

        for dx in -(half + 3)..=(half + 3) {
            let x = center + dx;
            // Nothing above the water line, including the dry banks.
            for y in 0..level {
                assert_eq!(
                    w.tile(x, y).unwrap().liquid,
                    0,
                    "liquid above the lake surface at {x},{y}"
                );
            }
            // Inside the bowl: water only in air over the solid floor.
            for y in level..level + 8 {
                let t = w.tile(x, y).unwrap();
                if t.liquid > 0 {
                    assert!(!t.active(), "liquid inside a solid tile at {x},{y}");
                    assert!(
                        y < w.surface_of(x),
                        "liquid floats above the ground surface at {x},{y}"
                    );
                }
            }
        }
    }

    #[test]
    fn lake_keeps_grass_on_the_dry_banks() {
        let mut w = generate(&GenOptions::default());
        w.compute_column_surface();
        let (center, half, level) = lake_params(&w);
        let mut grass_banks = 0;
        for dx in [-(half + 1), half + 1] {
            let x = center + dx;
            let top = w.surface_of(x);
            let t = *w.tile(x, top).unwrap();
            assert!(t.active(), "bank column x={x} has no ground");
            if t.kind == TILE_GRASS {
                assert!(
                    top <= level,
                    "grass bank at x={x} sits below the water line (top {top}, level {level})"
                );
                assert!(
                    (0..w.height).all(|y| w.tile(x, y).unwrap().liquid == 0),
                    "dry bank at x={x} contains liquid"
                );
                grass_banks += 1;
            }
        }
        assert!(grass_banks >= 1, "no grass bank left in the old lake span");
    }

    #[test]
    fn lake_bowl_floor_is_solid_below_the_surface() {
        let mut w = generate(&GenOptions::default());
        w.compute_column_surface();
        let (center, half, level) = lake_params(&w);
        let mut deepest = 0;
        for dx in -half..=half {
            let x = center + dx;
            let top = w.surface_of(x);
            let t = *w.tile(x, top).unwrap();
            assert!(
                t.active() && td::TILE_SOLID.has(t.kind),
                "bowl floor is not solid at {x},{top}"
            );
            assert_eq!(t.kind, TILE_DIRT, "bowl floor is not dirt at {x},{top}");
            let depth = top - level;
            assert_eq!(
                depth,
                lake_floor_depth(dx, half),
                "unexpected bowl depth at x={x}"
            );
            assert!(
                depth >= 1,
                "bowl floor is not below the water line at x={x}"
            );
            deepest = deepest.max(depth);
            assert!(
                w.tile(x, top + 1).unwrap().active(),
                "bowl floor is only one tile thin at x={x}"
            );
            assert!(
                w.tile(x, level).unwrap().liquid > 0,
                "shore is dry at the water line at x={x}"
            );
        }
        assert_eq!(deepest, 6, "bowl should be 6 deep at its centre");
        assert_eq!(lake_floor_depth(0, half), 6);
        assert_eq!(lake_floor_depth(half, half), 1);
        assert_eq!(lake_floor_depth(-half, half), 1);
    }
}
