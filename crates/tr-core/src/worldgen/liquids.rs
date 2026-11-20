//! Liquid placement: the forest lake, the ocean shores and lava puddles,
//! and the Waterfalls pass's liquid-edge half-brick sweep.

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
    let (left_beach_end, right_beach_start) = (map.left_beach_end, map.right_beach_start);
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
    // Ocean water at both shores — retired in M5-OCEAN phase 2: the
    // faithful `Beaches` pass (`WorldGen.cs:14888`) carves the ocean prism
    // and owns the water; the compact flood would double-place it inside
    // the carved columns.
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

/// `WorldGen.SolidTile` inside the Waterfalls pass's
/// `Main.tileSolid[191] = false` window (`WorldGen.cs:16615`/`:16664`):
/// living wood (191, `TileID.cs:1011`) reads non-solid for the pass's two
/// sweeps only; the toggle is restored at the pass tail, so the port models
/// it as a local predicate over the shared `solid_tile` (the
/// `dungeon_features.rs::solid_for_trap` precedent for the 379 window).
fn solid_waterfalls(world: &World, x: i32, y: i32) -> bool {
    if world.tile(x, y).is_some_and(|t| t.kind == 191) {
        return false;
    }
    crate::edit::solid_tile(world, x, y)
}

/// `Main.tile[i, j].active()` on an in-bounds scan cell (the C# reads the
/// initialized tile directly; the port's `Option` folds the bounds).
fn scan_active(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| t.active())
}

/// `Main.tile[i, j].halfBrick()` on an in-bounds scan cell.
fn scan_half_brick(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| t.half_brick())
}

/// `Main.tile[i, j].liquid > 0` on an in-bounds scan cell.
fn scan_liquid(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| t.liquid > 0)
}

/// The `Waterfalls` pass (`WorldGen.cs:16610-16666`): two full-world sweeps
/// that pound the solid cells beside liquid into half bricks so water pours
/// over the edge. Sweep 1 (`:16616-16645`) finds a solid cell with air on
/// both sides, solid below and liquid on either side, draws the
/// `Next(8, 20)` pair for the vertical half-brick veto window, then the
/// conditional `Next(10)` for the obsidian/hellstone brick (75/76) rarity,
/// and pounds. Sweep 2 (`:16646-16663`) pounds the solid cell whose
/// neighbour is a half-brick shore with liquid two cells out (no draws).
/// `PoundTile` is the shared gated port (`edit/hammer.rs`, the
/// `isGeneratingOrLoadingWorld` arm — the temple/mini-biomes `|_, _| false`
/// chest-closure precedent). The `tileSolid[191]` window rides
/// [`solid_waterfalls`]; the skyblock gate lives in the step adapter.
pub(crate) fn pass_waterfalls(world: &mut World, rng: &mut Rng) {
    // Sweep 1 (`:16616-16645`).
    for i in 20..world.width - 20 {
        for j in 20..world.height - 20 {
            // `:16621` — the liquid-edge gate, C# short-circuit order.
            if solid_waterfalls(world, i, j)
                && !scan_active(world, i - 1, j)
                && solid_waterfalls(world, i, j + 1)
                && !scan_active(world, i + 1, j)
                && (scan_liquid(world, i - 1, j) || scan_liquid(world, i + 1, j))
            {
                let mut flag = true;
                // `:16624-16627` — the two draws, then the window bounds.
                let num = j - rng.range(8, 20);
                let num2 = j + rng.range(8, 20);
                // `:16628-16634` — any half brick in the column window vetoes.
                for k in num..=num2 {
                    if scan_active(world, i, k) && scan_half_brick(world, i, k) {
                        flag = false;
                    }
                }
                // `:16635-16638` — the brick rarity draw (75/76 only).
                let kind = world.tile(i, j).map_or(0, |t| t.kind);
                if (kind == 75 || kind == 76) && !rng.chance(10) {
                    flag = false;
                }
                if flag {
                    // `:16641` — `WorldGen.PoundTile(i, j)`.
                    crate::edit::pound_tile(world, i, j, true, |_, _| false);
                }
            }
        }
    }
    // Sweep 2 (`:16646-16663`) — the half-brick shore arms, no RNG.
    for l in 20..world.width - 20 {
        for m in 20..world.height - 20 {
            // `:16651` — the spike (48) / wooden-spike (232) exclusions read
            // the stored type, then the two-solid column gate.
            let kind = world.tile(l, m).map_or(0, |t| t.kind);
            if kind != 48
                && kind != 232
                && solid_waterfalls(world, l, m)
                && solid_waterfalls(world, l, m + 1)
            {
                // `:16653-16656` — the left half-brick shore.
                if !solid_waterfalls(world, l + 1, m)
                    && scan_active(world, l - 1, m)
                    && scan_half_brick(world, l - 1, m)
                    && scan_liquid(world, l - 2, m)
                {
                    crate::edit::pound_tile(world, l, m, true, |_, _| false);
                }
                // `:16657-16660` — the right half-brick shore.
                if !solid_waterfalls(world, l - 1, m)
                    && scan_active(world, l + 1, m)
                    && scan_half_brick(world, l + 1, m)
                    && scan_liquid(world, l + 2, m)
                {
                    crate::edit::pound_tile(world, l, m, true, |_, _| false);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::lake_floor_depth;
    use super::{pass_waterfalls, Rng};
    use crate::tile::{FLAG_ACTIVE, FLAG_HALF_BRICK};
    use crate::tiledata as td;
    use crate::world::World;
    use crate::worldgen::{GenOptions, TILE_DIRT, TILE_GRASS, generate};

    /// W4-d103 pass 20: a stone column with air on both sides, solid below
    /// and water on the left — sweep 1 pounds the edge cell (the two
    /// `Next(8, 20)` draws find no half brick in the window, kind 1 skips
    /// the brick rarity draw).
    fn waterfall_edge_world(kind: u16) -> World {
        let mut world = World::new("wf".into(), 100, 100);
        for y in 40..60 {
            let i = world.index(50, y);
            world.tiles[i].kind = kind;
            world.tiles[i].flags |= FLAG_ACTIVE;
        }
        // Water beside the edge cell (50, 40), air on both sides.
        world.tiles[world.index(49, 40)].liquid = 255;
        world
    }

    #[test]
    fn waterfalls_sweep1_pounds_the_liquid_edge() {
        let mut world = waterfall_edge_world(1);
        pass_waterfalls(&mut world, &mut Rng(0x5EED));
        assert!(
            world.tile(50, 40).unwrap().half_brick(),
            "the liquid edge is pounded"
        );
        // The cells below keep their full height (no liquid beside them).
        assert!(!world.tile(50, 41).unwrap().half_brick());
    }

    #[test]
    fn waterfalls_sweep1_half_brick_window_vetoes() {
        let mut world = waterfall_edge_world(1);
        // A half brick inside every possible window (j-19..=j+19 for j=40).
        let i = world.index(50, 45);
        world.tiles[i].flags |= FLAG_HALF_BRICK;
        pass_waterfalls(&mut world, &mut Rng(0x5EED));
        assert!(
            !world.tile(50, 40).unwrap().half_brick(),
            "the window veto refuses the pound"
        );
    }

    #[test]
    fn waterfalls_sweep2_pounds_the_half_brick_shore() {
        let mut world = World::new("wf2".into(), 100, 100);
        for y in 40..60 {
            let i = world.index(50, y);
            world.tiles[i].kind = 1;
            world.tiles[i].flags |= FLAG_ACTIVE;
        }
        // The left neighbour is a half-brick shore with liquid two out.
        let shore = world.index(49, 40);
        world.tiles[shore].kind = 1;
        world.tiles[shore].flags |= FLAG_ACTIVE | FLAG_HALF_BRICK;
        world.tiles[world.index(48, 40)].liquid = 255;
        pass_waterfalls(&mut world, &mut Rng(0x5EED));
        assert!(
            world.tile(50, 40).unwrap().half_brick(),
            "the shore arm pounds the column top"
        );
    }

    #[test]
    fn waterfalls_living_wood_window_draws_nothing() {
        // The `tileSolid[191] = false` window: living wood reads non-solid,
        // so the identical edge fixture pounds nothing and draws nothing.
        let mut world = waterfall_edge_world(191);
        let mut rng = Rng(0x5EED);
        pass_waterfalls(&mut world, &mut rng);
        assert_eq!(rng.0, 0x5EED, "no draws on the living-wood column");
        assert!(
            world.tiles.iter().all(|t| !t.half_brick()),
            "the window refuses every pound"
        );
    }

    #[test]
    fn waterfalls_pass_is_deterministic_for_a_seed() {
        let base = waterfall_edge_world(1);
        let mut a = base.clone();
        let mut b = base.clone();
        pass_waterfalls(&mut a, &mut Rng(42));
        pass_waterfalls(&mut b, &mut Rng(42));
        assert_eq!(a.tiles, b.tiles, "a == b");
    }

    /// `lake_center`, `lake_half` and the water line (first row with liquid).
    fn lake_params(w: &World) -> (i32, i32, i32) {
        let center = (w.width / 2 + 150).min(w.width - 80);
        let level = (0..w.height)
            .find(|&y| w.tile(center, y).unwrap().liquid > 0)
            .expect("lake has no water at its centre");
        (center, 32, level)
    }

    /// The ground row below any sky objects (the floating islands' clouds
    /// hang over the terrain — the first solid from the sky is not the
    /// ground anymore; walk down to the first dirt tile).
    fn ground_row(w: &crate::world::World, x: i32) -> i32 {
        let mut y = w.surface_of(x);
        while y < w.height - 2 && w.tile(x, y).unwrap().kind != TILE_DIRT {
            y += 1;
        }
        y
    }

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
            // The bank's top: the first non-cloud tile from the surface (the
            // floating islands hang over the terrain).
            let mut top = w.surface_of(x);
            while top < w.height - 2
                && matches!(w.tile(x, top).unwrap().kind, 189 | 196)
            {
                top += 1;
            }
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
            let top = ground_row(&w, x);
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
