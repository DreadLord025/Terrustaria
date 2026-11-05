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

mod biome_detail;
mod islands;
mod biomes;
mod structures;
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
    /// `WorldGen.crimson`: the evil biome's flavour (the corruption when
    /// false).
    pub crimson: bool,
}

impl Default for GenOptions {
    fn default() -> Self {
        Self {
            name: "Terrustaria".into(),
            width: 4200,
            height: 1200,
            crimson: false,
            seed: 1,
        }
    }
}

/// The world's evil flavour roll for the `_optionEvil = Random` choice —
/// the C# menu rolls `WorldGen.crimson` from the seed; the port derives it
/// deterministically from the seed's high bit.
pub fn roll_crimson(seed: u64) -> bool {
    (seed >> 56) & 1 == 1
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

    let noise = Noise { seed: opts.seed };
    let mut rng = Rng(opts.seed ^ 0xA076_1D64_78BD_642F);

    // --- surface height map + terrain fill ----------------------------------
    let mut map = terrain::surface_height_map(width, height, &noise);
    let layers = terrain::fill_terrain(&mut world, width, height, &noise, &map);

    // --- biomes (the C# order: the jungle/evil/mushroom/granite/marble run
    // before the caves so the caves cut through them) -----------------------
    let spans = biomes::plan_spans(width, &map.heights, map.beach);
    biomes::pass_jungle(&mut world, &map.heights, &spans, layers.rock_layer, &mut rng);
    biomes::pass_evil(&mut world, &map.heights, &spans, &mut rng, opts.crimson);
    biomes::pass_mushroom(&mut world, &spans, layers.rock_layer, &mut rng);
    biomes::pass_granite_marble(&mut world, layers.rock_layer, &mut rng);

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

    // --- dungeon (the C# `Dungeon` pass runs after the caves; carving after
    // the liquids keeps the shafts dry) -------------------------------------
    biomes::pass_dungeon(&mut world, &map.heights, &spans, &mut rng);

    // --- structures (the C# order: the shrines and the altars ride the
    // world's later passes; the chests fill the registered spots; the pots
    // scatter last) ---------------------------------------------------------
    // The structures run on their own stream — the shared  feeds the
    // living-tree/tree passes after this point and must not shift.
    // The detail passes (the ocean caves run with the caves' timing, the
    // plants/vines after the biomes; the fragile ice after the liquids).
    let mut drng = Rng(opts.seed ^ 0x5DEE_CE66_D1A4_9B5F);
    biome_detail::pass_ocean_caves(&mut world, &map.heights, map.beach, &mut drng);
    biome_detail::pass_fragile_ice(&mut world, width * 2 / 3, &mut drng);
    biome_detail::pass_underground_jungle_trees(&mut world, &spans, layers.rock_layer, &mut drng);
    let mut srng = Rng(opts.seed ^ 0x9E37_79B9_7F4A_7C15);
    let shrine_spots = structures::pass_jungle_shrines(&mut world, &spans, layers.rock_layer, &mut srng);
    structures::pass_chests(&mut world, &shrine_spots, &mut srng);
    structures::pass_altars(&mut world, opts.crimson, &mut srng);
    structures::pass_pyramids(
        &mut world,
        map.desert_start,
        map.desert_end,
        &map.heights,
        &mut srng,
    );
    structures::pass_pots(&mut world, &mut srng);
    structures::pass_beehives(&mut world, &spans, layers.rock_layer, &mut srng);
    structures::pass_hell_forts(&mut world, layers.hell_top, &mut srng);
    biome_detail::pass_mushroom_and_jungle_plants(&mut world);
    biome_detail::pass_vines(&mut world, &mut drng);
    // The floating islands (`FloatingIslands` — the C# order places them
    // right after the biome passes; the drng stream keeps the tree stream
    // stable).
    islands::pass_floating_islands(&mut world, &map.heights, &mut drng);

    // --- surface slopes -----------------------------------------------------
    surface::apply_slopes(&mut world, width, &map.heights);

    // --- living trees -------------------------------------------------------
    // Vanilla generates "Living Trees" before "Trees"
    // (`GenPassNameID.cs:267-268`), so the clearance scans see bare terrain.
    living_trees::plant_living_trees(
        &mut world,
        &mut rng,
        &map.heights,
        map.beach,
        (map.desert_start, map.desert_end),
    );

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
            crimson: false,
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
        // The floating islands eat part of the tree canvas (the clearance
        // scan rejects the anchors under them) — the thresholds account for
        // the reduced count.
        assert!(wood > 300, "living wood tiles: {wood}");
        assert!(leaves > 300, "leaf blocks: {leaves}");
        assert!(wall > 100, "living wood walls: {wall}");
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
            // The pyramid entrances open the desert surface (vanilla
            // `Pyramid` carves into the dunes) — the air columns skip. The
            // living trees grow on sand too (`GrowLivingTree` gates only the
            // anchor air, not the soil) — their columns skip. The floating
            // islands hang well above the surface line (the disc bottom
            // stays ≥ 100 tiles over it), so the sky columns skip wholesale.
            if !t.active() || matches!(t.kind, 191 | 192) || (top as f64) < w.world_surface - 100.0
            {
                continue;
            }
            assert!(
                t.kind == TILE_SAND,
                "desert surface at x{x} is kind {}",
                t.kind
            );
            for y in top..top + 60 {
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
        #[cfg(test)]
        eprintln!(
            "probe final: h{saw_hardened} s{saw_sandstone}"
        );
        if !(saw_hardened && saw_sandstone) {
            let mut p2 = String::new();
            for x in [start + 10, start + 60, start + 130, end - 30] {
                let top = w.surface_of(x);
                let t = w.tile(x, top).unwrap();
                let mut ks = String::new();
                for y in top..top + 60 {
                    ks.push_str(&format!("{},", w.tile(x, y).unwrap().kind));
                }
                p2.push_str(&format!(
                    "x{x} top{top} kind{} skip{} [{ks}] ",
                    t.kind,
                    !t.active() || matches!(t.kind, 191 | 192) || (top as f64) < w.world_surface - 100.0
                ));
            }
            panic!("missing desert strata ws{:.0}: {p2}", w.world_surface);
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

    #[test]
    fn the_biome_passes_place_the_missing_biomes() {
        // The 4200-wide world carries the jungle (60/59), the evil band
        // (23/25 corruption — or 199/203 crimson), the mushroom patches
        // (70), the granite/marble caves (368/367) and the dungeon brick.
        let w = generate(&GenOptions {
            seed: 77,
            ..Default::default()
        });
        let count = |kind: u16| {
            w.tiles
                .iter()
                .filter(|t| t.active() && t.kind == kind)
                .count()
        };
        assert!(count(60) > 200, "the jungle grass ({})", count(60));
        assert!(count(59) > 500, "the mud ({})", count(59));
        assert!(count(25) > 200, "the ebonstone ({})", count(25));
        assert!(count(23) > 50, "the corrupt grass ({})", count(23));
        assert!(count(70) > 50, "the mushroom grass ({})", count(70));
        assert!(count(368) > 300, "the granite ({})", count(368));
        assert!(count(367) > 300, "the marble ({})", count(367));
        assert!(count(41) > 200, "the dungeon brick ({})", count(41));
        // The crimson flavour swaps the evil tile set.
        let c = generate(&GenOptions {
            seed: 77,
            crimson: true,
            ..Default::default()
        });
        let crimstone = c
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == 203)
            .count();
        assert!(crimstone > 200, "the crimstone ({crimstone})");
        assert_eq!(
            c.tiles
                .iter()
                .filter(|t| t.active() && t.kind == 25)
                .count(),
            0,
            "no ebonstone in the crimson world"
        );
    }

    #[test]
    fn the_hives_and_forts_place_the_content() {
        // The 4200-world carries the hive domes (225) with the honey (the
        // liquid 2) and the larva (231), the hell forts (the brick 75 with
        // the walls 14, the lava, the chests).
        let w = generate(&GenOptions {
            seed: 63,
            ..Default::default()
        });
        let count = |kind: u16| {
            w.tiles
                .iter()
                .filter(|t| t.active() && t.kind == kind)
                .count()
        };
        let honey = w
            .tiles
            .iter()
            .filter(|t| t.liquid > 0 && t.flags & crate::tile::FLAG_LIQUID_HONEY != 0)
            .count();
        assert!(count(225) > 200, "the hive blocks ({})", count(225));
        assert!(honey > 100, "the honey pockets ({honey})");
        assert!(count(231) >= 1, "the larva ({})", count(231));
        assert!(count(75) > 300, "the hell brick ({})", count(75));
    }

    #[test]
    fn the_floating_islands_hang_in_the_sky() {
        // The 4200-world carries 3-4 cloud islands (189) with the grass
        // caps, the sky lake (the rain cloud 196 + the water) and the
        // houses (the planks 9 + the cloud walls 73 + the chests).
        let w = generate(&GenOptions {
            seed: 55,
            ..Default::default()
        });
        let count = |kind: u16| {
            w.tiles
                .iter()
                .filter(|t| t.active() && t.kind == kind)
                .count()
        };
        assert!(count(189) > 1500, "the cloud blocks ({})", count(189));
        assert!(count(196) > 30, "the rain cloud rim ({})", count(196));
        assert!(count(9) > 40, "the house planks ({})", count(9));
        // The islands sit above the surface line: every cloud tile's row is
        // above `world_surface` (the surface scan finds none below).
        let below = w
            .tiles
            .chunks(w.width as usize)
            .enumerate()
            .filter(|(y, _)| (*y as f64) > w.world_surface)
            .map(|(_, row)| row.iter().filter(|t| t.active() && t.kind == 189).count())
            .sum::<usize>();
        assert_eq!(below, 0, "no cloud tiles below the surface line");
    }

    #[test]
    fn the_structure_passes_place_the_contents() {
        // The 4200-world carries the shrine brick huts (119) with the torches,
        // the chests (21), the demon altars (26), the pyramid brick (480) and
        // the pots (28).
        let w = generate(&GenOptions {
            seed: 91,
            ..Default::default()
        });
        let count = |kind: u16| {
            w.tiles
                .iter()
                .filter(|t| t.active() && t.kind == kind)
                .count()
        };
        // The brick ring only (the carved interior does not count): ~25-32
        // tiles per shrine shell.
        assert!(count(119) > 150, "the shrine brick ({})", count(119));
        assert!(count(4) > 3, "the shrine torches ({})", count(4));
        assert!(count(21) >= 4, "the chests ({})", count(21) / 4);
        assert!(count(26) > 30, "the demon altars ({})", count(26) / 6);
        assert!(count(480) > 300, "the pyramid brick ({})", count(480));
        assert!(count(28) > 50, "the pots ({})", count(28));
        // The chest tiles come in intact 2x2 quads — every chest anchor
        // (frame 0,0) has its three siblings.
        let mut anchors = 0;
        for y in 1..w.height - 2 {
            for x in 1..w.width - 2 {
                let t = w.tile(x, y).unwrap();
                if t.active() && t.kind == 21 && t.frame_x == 0 && t.frame_y == 0 {
                    let ok = [(1i32, 0i32, 18i16), (0, 1, 0), (1, 1, 18)]
                        .iter()
                        .all(|(dx, dy, fx)| {
                            w.tile(x + dx, y + dy).is_some_and(|n| {
                                n.active() && n.kind == 21 && n.frame_x == *fx && n.frame_y == if *dy == 1 { 18 } else { 0 }
                            })
                        });
                    if ok {
                        anchors += 1;
                    }
                }
            }
        }
        assert!(anchors >= 4, "the intact chest quads ({anchors})");
    }
}
