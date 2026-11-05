//! Terraria-like world generation.
//!
//! This is a compact generator modelled on the shapes of `WorldGen`'s passes (surface
//! noise, dirt/stone layers, cave fields, ore blobs, tree growth) rather than a port of
//! all 45k lines. It is deterministic for a seed and produces a world the existing
//! framer/lighting/renderer can consume unchanged.
//!
//! Layout follows a small Terraria world: 4200x1200, forest surface around row ~360,
//! dirt wall backing, stone below the rock layer and an ash underworld.
//!
//! The work is split into passes that [`generate`] calls in a fixed order — terrain
//! (surface map + fill), caves/ores, liquids, then the surface dressing. The call
//! order and every RNG draw are part of the world contract; `deterministic_for_seed`
//! pins them.

mod caves;
mod liquids;
mod living_trees;
mod noise;
mod surface;
mod terrain;

use self::noise::{Noise, Rng};
use crate::tiledata as td;
use crate::tree;
use crate::world::World;

#[derive(Clone, Debug)]
pub struct GenOptions {
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub seed: u64,
}

impl Default for GenOptions {
    fn default() -> Self {
        Self {
            name: "Terrustaria".into(),
            width: 4200,
            height: 1200,
            seed: 1,
        }
    }
}

/// The vanilla tile/wall ids the generator paints with.
pub(crate) const TILE_DIRT: u16 = 0;
pub(crate) const TILE_STONE: u16 = 1;
pub(crate) const TILE_GRASS: u16 = 2;
pub(crate) const TILE_SNOW: u16 = 147;
pub(crate) const TILE_SAND: u16 = 53;
pub(crate) const TILE_SANDSTONE: u16 = 396;
pub(crate) const TILE_HARDENED_SAND: u16 = 397;
pub(crate) const TILE_IRON: u16 = 6;
pub(crate) const TILE_COPPER: u16 = 7;
pub(crate) const TILE_GOLD: u16 = 8;
pub(crate) const TILE_SILVER: u16 = 9;
pub(crate) const TILE_ASH: u16 = 57;
pub(crate) const TILE_HELLSTONE: u16 = 58;
pub(crate) const WALL_STONE: u16 = 1;
pub(crate) const WALL_DIRT: u16 = 2;
pub(crate) const WALL_SANDSTONE: u16 = 187;

pub fn generate(opts: &GenOptions) -> World {
    let width = opts.width.max(512);
    let height = opts.height.max(512);
    let mut world = World::new(opts.name.clone(), width, height);
    world.frame_important = vec![false; td::TILE_COUNT];
    world.frame_important[tree::TREE as usize] = true;
    world.frame_important[3] = true;
    world.frame_important[4] = true;
    // `Main.tileFrameImportant` boot statics for the M1.4-TO-4 families
    // (sunflower 27 / pots 28 `Main.cs:7663-7664`, clay pot 78 `:7682`,
    // herbs 82-84 `:7685-7687`, piles 185-187 `:7415-7417`, planter box 380
    // `:7632`) — vanilla worlds honour them from boot, so a generated world
    // must too: `framing` keeps the stored frames of these kinds and the kill
    // cascade reaches them.
    world.frame_important[27] = true;
    world.frame_important[28] = true;
    world.frame_important[78] = true;
    world.frame_important[82] = true;
    world.frame_important[83] = true;
    world.frame_important[84] = true;
    world.frame_important[185] = true;
    world.frame_important[186] = true;
    world.frame_important[187] = true;
    world.frame_important[380] = true;

    let noise = Noise { seed: opts.seed };
    let mut rng = Rng(opts.seed ^ 0xA076_1D64_78BD_642F);

    // --- surface height map + terrain fill ----------------------------------
    let mut map = terrain::surface_height_map(width, height, &noise);
    let layers = terrain::fill_terrain(&mut world, width, height, &noise, &map);

    // --- caves --------------------------------------------------------------
    caves::carve_caves(&mut world, width, &noise, &map.heights, layers.hell_top);
    // Dirt pockets inside stone and stone outcrops inside dirt.
    caves::blob_pass(
        &mut world, &mut rng, width, height, TILE_STONE, TILE_DIRT, 900, 10,
    );
    caves::blob_pass(
        &mut world, &mut rng, width, height, TILE_DIRT, TILE_STONE, 500, 6,
    );
    caves::blob_pass(
        &mut world, &mut rng, width, height, TILE_ASH, TILE_STONE, 400, 8,
    );

    // --- ores ---------------------------------------------------------------
    caves::place_ores(
        &mut world,
        &mut rng,
        &map.heights,
        width,
        height,
        layers.hell_top,
    );

    // --- liquids ------------------------------------------------------------
    liquids::place_liquids(&mut world, &mut rng, &mut map, &layers, width);

    // --- surface slopes -----------------------------------------------------
    surface::apply_slopes(&mut world, width, &map.heights);

    // --- living trees -------------------------------------------------------
    // Vanilla generates "Living Trees" before "Trees"
    // (`GenPassNameID.cs:267-268`), so the clearance scans see bare terrain.
    living_trees::plant_living_trees(&mut world, &mut rng, &map.heights, map.beach);

    // --- trees --------------------------------------------------------------
    surface::plant_trees(&mut world, &mut rng, &map.heights);

    // --- surface plants -----------------------------------------------------
    surface::plant_surface_flora(&mut world, &mut rng, width, &map.heights);

    // --- desert cacti -------------------------------------------------------
    surface::plant_cacti(&mut world, &mut rng, &map, width);

    world.world_surface = layers.world_surface;
    world.rock_layer = layers.rock_layer;
    world.spawn_x = map.spawn_center;
    world.spawn_y = (map.heights[map.spawn_center as usize] - 6).max(1);
    world.time = 27000.0; // noon
    world.day_time = true;
    // `Main.moonType = Main.rand.Next(9)` (`Main.cs:10873`), drawn from the same
    // generator here so it is fixed for the seed; the phase starts at 0.
    world.moon_type = (rng.next_u32() % 9) as u8;
    world.moon_phase = 0;
    // `Main.hellBackStyle = WorldGen.genRand.Next(3)` (`WorldGen.cs:8247`), drawn
    // from the same generator; it picks the animated magma/hell sheets at draw time.
    world.hell_back_style = (rng.next_u32() % 3) as u8;
    world
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn living_trees_generate_for_seed() {
        // The living-trees pass places tile 191 (wood) with 192 canopies and
        // wall 244 interiors (`WorldGen.cs:15476-15715`).
        let world = generate(&GenOptions {
            name: "living".into(),
            width: 4200,
            height: 1200,
            seed: 1,
        });
        let mut wood = 0;
        let mut leaves = 0;
        let mut wall = 0;
        for t in &world.tiles {
            if t.active() && t.kind == 191 {
                wood += 1;
            }
            if t.active() && t.kind == 192 {
                leaves += 1;
            }
        }
        for t in world.tiles.iter() {
            if t.wall == 244 {
                wall += 1;
            }
        }
        assert!(wood > 500, "living wood tiles: {wood}");
        assert!(leaves > 500, "leaf blocks: {leaves}");
        assert!(wall > 200, "living wood walls: {wall}");
        // The anchor positions for the screenshot cameras (TR_CAM=x,y).
        let mut xs: Vec<i32> = Vec::new();
        for x in 0..world.width {
            for y in 0..world.height {
                if world.tile(x, y).is_some_and(|t| t.active() && t.kind == 191) {
                    xs.push(x);
                    break;
                }
            }
        }
        eprintln!("living columns: {} first {} last {}", xs.len(), xs.first().map_or(0, |v| *v), xs.last().map_or(0, |v| *v));
        let mut ys: Vec<i32> = Vec::new();
        for x in 1920..2020 {
            for y in 0..world.height {
                if world.tile(x, y).is_some_and(|t| t.active() && t.kind == 191) {
                    ys.push(y);
                    break;
                }
            }
        }
        eprintln!("first-191-y per column 1920..2020: min {} max {} count {}", ys.iter().min().map_or(0, |v| *v), ys.iter().max().map_or(0, |v| *v), ys.len());
        for (px, py) in [(300i32, 0i32), (280, 0), (350, 0)] {
            let mut rows = Vec::new();
            for y in 0..world.height {
                if world.tile(px, y).is_some_and(|t| t.active() && t.kind == 191) {
                    rows.push(y);
                }
            }
            let first = rows.first().map_or(-1, |v| *v);
            let last = rows.last().map_or(-1, |v| *v);
            eprintln!("col {px}: 191 rows {} first {first} last {last} (surface {})", rows.len(), world.surface_of(px));
        }
    }


    #[test]
    fn deterministic_for_seed() {
        let opts = GenOptions {
            width: 512,
            height: 512,
            seed: 42,
            ..Default::default()
        };
        let a = generate(&opts);
        let b = generate(&opts);
        assert_eq!(a.tiles.len(), b.tiles.len());
        for (x, y) in a.tiles.iter().zip(&b.tiles) {
            assert_eq!(
                (x.kind, x.wall, x.flags, x.frame_x, x.frame_y),
                (y.kind, y.wall, y.flags, y.frame_x, y.frame_y)
            );
        }
    }

    #[test]
    fn desert_band_has_sand_dunes_and_no_trees() {
        let opts = GenOptions::default();
        let mut w = generate(&opts);
        w.compute_column_surface();
        let center = w.width / 2;
        let start = center - 380 - 170 + 20;
        let end = center - 380 + 170 - 20;
        let mut saw_hardened = false;
        let mut saw_sandstone = false;
        for x in start..end {
            let top = w.surface_of(x);
            let t = w.tile(x, top).unwrap();
            assert!(
                t.kind == TILE_SAND,
                "desert surface at x{x} is kind {}",
                t.kind
            );
            for y in top..top + 40 {
                let k = w.tile(x, y).unwrap().kind;
                saw_hardened |= k == TILE_HARDENED_SAND;
                saw_sandstone |= k == TILE_SANDSTONE;
            }
            for y in top..top + 30 {
                assert!(
                    !(w.tile(x, y).unwrap().active() && w.tile(x, y).unwrap().kind == tree::TREE),
                    "tree inside the desert at {x},{y}"
                );
            }
        }
        assert!(saw_hardened && saw_sandstone, "missing desert strata");
        let mut cacti = 0;
        for x in start..end {
            let top = w.surface_of(x);
            for y in top.saturating_sub(5)..top {
                let t = w.tile(x, y).unwrap();
                if t.active() && t.kind == 80 {
                    cacti += 1;
                    assert!(t.frame_y >= 0, "cactus must have a manual frame");
                    assert!(w.frame_important[80]);
                }
            }
        }
        assert!(cacti > 5, "only {cacti} cactus tiles");
    }

    #[test]
    fn has_surface_trees_and_caves() {
        let opts = GenOptions {
            width: 1024,
            height: 600,
            seed: 7,
            ..Default::default()
        };
        let w = generate(&opts);
        assert!(w.tiles.iter().any(|t| t.active() && t.kind == TILE_GRASS));
        assert!(w.tiles.iter().any(|t| t.active() && t.kind == tree::TREE));
        // Some air below the average surface means caves were carved.
        let mut underground_air = 0;
        for y in (w.world_surface as i32)..(w.height - 200) {
            for x in 20..w.width - 20 {
                if !w.tiles[w.index(x, y)].active() {
                    underground_air += 1;
                }
            }
        }
        assert!(underground_air > 1000, "caves {underground_air}");
        assert!(
            w.tiles.iter().any(|t| t.liquid > 0),
            "generator should place a lake and lava pools"
        );
        assert!(
            w.tiles.iter().any(|t| t.active() && t.kind == TILE_SAND),
            "generator should place ocean beaches"
        );
        assert!(
            w.tiles.iter().any(|t| t.active() && t.kind == TILE_SNOW),
            "generator should place a snow biome"
        );
    }

    /// DW-1.4: the generated world mirrors `Main.tileFrameImportant`'s boot
    /// statics for the M1.4-TO-4 families (sunflower 27 / pots 28 `Main.cs:
    /// 7663-7664`, clay pot 78 `:7682`, herbs 82-84 `:7685-7687`, piles
    /// 185-187 `:7415-7417`, planter box 380 `:7632`).
    #[test]
    fn test_DW_1_4_generated_world_carries_frame_important_bits() {
        let world = generate(&GenOptions {
            name: "bits".into(),
            width: 512,
            height: 512,
            seed: 7,
        });
        for kind in [27u16, 28, 78, 82, 83, 84, 185, 186, 187, 380] {
            assert!(
                world.frame_important[kind as usize],
                "frame_important[{kind}] must be set in a generated world"
            );
        }
    }
}
