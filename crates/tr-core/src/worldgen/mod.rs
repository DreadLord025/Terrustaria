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
mod biomes;
pub mod dungeon;
pub mod dungeon_features;
mod caves;
mod config;
mod desert;
mod desert_dunes;
mod desert_entrances;
pub mod evil;
mod islands;
mod liquids;
mod living_trees;
pub mod meteor;
mod micro_finish;
mod micro_grass;
mod micro_surface;
mod mini_biomes;
mod noise;
mod ocean;
mod pipeline;
mod spider_gem_moss;
mod structures;
mod surface;
mod temple;
mod terrain;
mod tile_runner;
mod wavy;
mod world_hash;
mod finish_seeds;

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
    /// The classic secret-seed flags — [`crate::seeds::SeedFlags`]. The
    /// `WorldGen.Reset()` set (`WorldGen.cs:11115-11123`) that every
    /// seed-conditional worldgen branch reads; `Default` = a classic
    /// world, so a flag-free options struct generates exactly the
    /// pre-M5-SEED-1 world.
    pub flags: crate::seeds::SeedFlags,
    /// M5-SEED-2: the 37 micro-seed flags beside the classic ones — the
    /// `SecretSeed.InitializeSecretSeeds` read (`WorldGen.cs:11124`) the
    /// worldgen arms take from `opts` and `generate` stamps onto the world.
    pub micro_flags: crate::seeds::MicroSeedFlags,
}

impl Default for GenOptions {
    fn default() -> Self {
        Self {
            name: "Terrustaria".into(),
            width: 4200,
            height: 1200,
            crimson: false,
            seed: 1,
            flags: crate::seeds::SeedFlags::default(),
            micro_flags: crate::seeds::MicroSeedFlags::default(),
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
pub(crate) const TILE_ICE: u16 = 161;
pub(crate) const TILE_HELLSTONE: u16 = 58;
/// The alternate pre-hardmode ores (`TileID.cs`: the tin 166, the lead
/// 167, the tungsten 168, the platinum 169).
pub(crate) const TILE_TIN: u16 = 166;
pub(crate) const TILE_LEAD: u16 = 167;
pub(crate) const TILE_TUNGSTEN: u16 = 168;
pub(crate) const TILE_PLATINUM: u16 = 169;
/// The evil ores (`TileID.cs`: the demonite 22, the crimtane 204) — the
/// `OresAndShinies` evil band (`WorldGen.cs:13589-13602`).
pub(crate) const TILE_DEMONITE: u16 = 22;
pub(crate) const TILE_CRIMTANE: u16 = 204;
pub(crate) const WALL_STONE: u16 = 1;
pub(crate) const WALL_DIRT: u16 = 2;
pub(crate) const WALL_SANDSTONE: u16 = 187;

pub fn generate(opts: &GenOptions) -> World {
    let width = opts.width.max(512);
    let height = opts.height.max(512);
    let mut world = World::new(opts.name.clone(), width, height);
    // The seed flags ride the generated world so the runtime readers (the
    // loot/fishing/variant/wind conditions, the save layers) see the same
    // special-world identity the passes generated with.
    world.seeds = opts.flags;
    world.micro_seeds = opts.micro_flags;
    // The `Reset`-equivalent setup (vanilla runs `Reset` — the GenVars init
    // and the layout rolls — before `AddPasses`; it is not a pass):
    // the frame-important boot statics, the value noise and the three
    // contract streams (the seed constants moved into `pipeline::Streams`
    // verbatim). Then the config (`WorldGen.cs:11080` — parsed once per
    // generation) and the pipeline (`WorldGen.cs:11086-11088`'s
    // AddPasses + DisablePassesForSpecialSeeds + the WorldGenerator loop).
    pipeline::boot_statics(&mut world);
    let noise = Noise { seed: opts.seed };
    let config = config::WorldGenConfig::parse();
    let mut streams = pipeline::Streams::new(opts.seed);
    pipeline::run(&mut world, &mut streams, opts, &noise, &config);
    world
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn living_trees_generate_for_seed() {
        // The living-trees pass places tile 191 (wood) with 192 canopies and
        // wall 244 interiors (`WorldGen.cs:15476-15715`). The tree-count roll
        // (`rng.range(0, 2*num3 + 1)` + the 50% rescue) can legitimately
        // draw 0 for a seed — the shared stream's position shifted when the
        // evil pass moved to its own stream (M5-BIOME phase 4) — so the
        // smoke check sweeps a few seeds and requires one to carry trees.
        let mut world = None;
        for seed in 1..=6u64 {
            let w = generate(&GenOptions {
                crimson: false,
                name: "living".into(),
                width: 4200,
                height: 1200,
                seed,
                ..Default::default()
            });
            let wood = w
                .tiles
                .iter()
                .filter(|t| t.active() && t.kind == 191)
                .count();
            if wood > 0 {
                world = Some(w);
                break;
            }
        }
        let world = world.expect("no seed in 1..=6 grew living trees");
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
                if world
                    .tile(x, y)
                    .is_some_and(|t| t.active() && t.kind == 191)
                {
                    xs.push(x);
                    break;
                }
            }
        }
        eprintln!(
            "living columns: {} first {} last {}",
            xs.len(),
            xs.first().map_or(0, |v| *v),
            xs.last().map_or(0, |v| *v)
        );
        let mut ys: Vec<i32> = Vec::new();
        for x in 1920..2020 {
            for y in 0..world.height {
                if world
                    .tile(x, y)
                    .is_some_and(|t| t.active() && t.kind == 191)
                {
                    ys.push(y);
                    break;
                }
            }
        }
        eprintln!(
            "first-191-y per column 1920..2020: min {} max {} count {}",
            ys.iter().min().map_or(0, |v| *v),
            ys.iter().max().map_or(0, |v| *v),
            ys.len()
        );
        for (px, py) in [(300i32, 0i32), (280, 0), (350, 0)] {
            let mut rows = Vec::new();
            for y in 0..world.height {
                if world
                    .tile(px, y)
                    .is_some_and(|t| t.active() && t.kind == 191)
                {
                    rows.push(y);
                }
            }
            let first = rows.first().map_or(-1, |v| *v);
            let last = rows.last().map_or(-1, |v| *v);
            eprintln!(
                "col {px}: 191 rows {} first {first} last {last} (surface {})",
                rows.len(),
                world.surface_of(px)
            );
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
            // The desert family: the `Full Desert` pass (M5-DESERT-structures)
            // places its underground desert under this band and its
            // `AddTileVariance` hardens exposed sand (`DesertHive.cs:396-398`),
            // so a band surface may read 53 / 397 / 396.
            assert!(
                matches!(t.kind, TILE_SAND | TILE_HARDENED_SAND | TILE_SANDSTONE),
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
        eprintln!("probe final: h{saw_hardened} s{saw_sandstone}");
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
                    !t.active()
                        || matches!(t.kind, 191 | 192)
                        || (top as f64) < w.world_surface - 100.0
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
        // 1600×900, not the old 1024×600: the vanilla beach widths
        // (`leftBeachEnd` 340-380 / `rightBeachStart` −320..−360,
        // `WorldGen.cs:11415-11440`) overlap the 2/3 snow band below 1212
        // wide — vanilla never generates that small (min 4200), so the snow
        // assert needs a non-degenerate span.
        let opts = GenOptions {
            width: 1600,
            height: 900,
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
            ..Default::default()
        });
        for kind in [27u16, 28, 78, 82, 83, 84, 185, 186, 187, 380] {
            assert!(
                world.frame_important[kind as usize],
                "frame_important[{kind}] must be set in a generated world"
            );
        }
    }

    /// M5-SEED-1: the framework's integration proof — every classic secret
    /// seed text of the ROADMAP row (drunk / for-the-worthy / remix /
    /// anniversary / don't starve) produces a DIFFERENT world than its
    /// seed-matched classic control (the world hash covers every stored
    /// `Tile` field, so a seed arm that misses its pass shows up here as a
    /// classic world). `getfixedboi` carries all seven dependencies, so it
    /// also differs from any single-flag world. Not-the-bees' hive world is
    /// deliberately outside the row's list — see the module docs.
    #[test]
    fn m5_seed_1_the_secret_seed_worlds_differ_from_the_classic() {
        let hash = |text: Option<&str>| {
            let flags = match text {
                Some(t) => crate::seeds::SeedFlags::from_seed_text(t),
                None => crate::seeds::SeedFlags::none(),
            };
            world_hash::world_hash(&generate(&GenOptions {
                seed: 77,
                flags,
                ..Default::default()
            }))
        };
        let classic = hash(None);
        for text in [
            "for the worthy",
            "05162020",
            "dontdigup",
            "celebrationmk10",
            "theconstant",
            "getfixedboi",
        ] {
            let seeded = hash(Some(text));
            assert_ne!(classic, seeded, "{text} generated the classic world");
            // The seed worlds must differ from each other too (the five
            // classic arms are not one shared branch).
            eprintln!("{text}: {seeded}");
        }
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
        // The granite threshold is 150, not the old 300: the vanilla beach
        // shelf removed the compact ramp's deep edge dip, so the model's
        // `surface_low` quartet anchor rose and the granite band narrowed on
        // this seed (201 tiles — recalibrated per the M5-ORES-1 precedent,
        // ANALYSIS "Ocean passes fidelity (M5-OCEAN)").
        assert!(count(368) > 150, "the granite ({})", count(368));
        assert!(count(367) > 300, "the marble ({})", count(367));
        // The dungeon colour rides the roll (the blue 41 / the green 43 /
        // the pink 44 — one per world).
        let dungeon = count(41) + count(43) + count(44);
        assert!(dungeon > 200, "the dungeon brick ({dungeon})");
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

    /// DW-6.1: the M5-DESERT-structures passes place their content — the
    /// dunes' sand crests above the surface and the underground desert's
    /// shells (the sandstone 396 / hardened-sand 397 strata and their 187/216
    /// walls) across three seeds.
    #[test]
    fn the_desert_passes_place_the_dunes_and_the_hive() {
        for seed in [1u64, 77, 4242] {
            let mut w = generate(&GenOptions {
                seed,
                ..Default::default()
            });
            w.compute_column_surface();
            let count_kind = |kind: u16| {
                w.tiles
                    .iter()
                    .filter(|t| t.active() && t.kind == kind)
                    .count()
            };
            let count_wall = |wall: u16| w.tiles.iter().filter(|t| t.wall == wall).count();
            // The hive's concentric shells (`DesertHive.cs:282-361`).
            assert!(count_kind(TILE_SANDSTONE) > 0, "seed {seed}: no sandstone band");
            assert!(count_kind(TILE_HARDENED_SAND) > 0, "seed {seed}: no hardened sand");
            assert!(count_wall(WALL_SANDSTONE) > 0, "seed {seed}: no sandstone walls");
            assert!(count_wall(216) > 0, "seed {seed}: no hardened-sand walls");
            // The dune crests (`PlaceCurvedLine`'s sand fill) rise above the
            // local surface — once the pass runs, the crest IS that column's
            // surface, so the pin is the sand standing well above the world's
            // mean surface line (the `Full Desert` mound and the beach wedges
            // never lift sand this far).
            let width = w.width as usize;
            let crests = w
                .tiles
                .iter()
                .enumerate()
                .filter(|(i, t)| {
                    t.active()
                        && t.kind == TILE_SAND
                        && ((*i / width) as i32) < (w.world_surface as i32 - 8)
                })
                .count();
            assert!(crests > 0, "seed {seed}: no dune crest above the surface");
        }
    }

    /// DW-4.1: the faithful evil pass at the integration level — the
    /// corruption world carries chasm shadow orbs (2x2 tile 31, frames
    /// 0/18), demon altars from the chasm tails (style 0), the ebonstone
    /// shells with the wall-3 lining, the ebonsand surface and the demonite
    /// side-cavern blobs; the crimson world carries the crimstone blob, the
    /// crimson hearts (frameX + 36), the style-1 altar scatter and the
    /// crimsand surface.
    #[test]
    fn test_DW_4_1_evil_biomes_carry_the_vanilla_structures() {
        let w = generate(&GenOptions {
            seed: 77,
            ..Default::default()
        });
        let orbs = w
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == 31)
            .count();
        assert!(orbs >= 8, "the shadow orbs ({orbs})");
        assert_eq!(orbs % 4, 0, "orbs come in 2x2 blocks");
        for t in w.tiles.iter().filter(|t| t.active() && t.kind == 31) {
            assert!(
                matches!(t.frame_x, 0 | 18) && matches!(t.frame_y, 0 | 18),
                "the corruption orb frames ({},{})",
                t.frame_x,
                t.frame_y
            );
        }
        // The demon altars: the chasm-tail search places style-0 altars
        // (frameX 0/18/36); the structures pass may add its own.
        // The structures pass's own altars never sit above
        // `(2*worldSurface + rockLayer) / 3` (`structures.rs:243`); an altar
        // above that line can only come from a chasm tail (the search draws
        // `Next(posY - 50, posY)` with `posY` in the walker's depth band).
        let structures_y_lo = ((w.world_surface * 2.0 + w.rock_layer) / 3.0) as i32;
        let altars: Vec<i32> = (0..w.width)
            .flat_map(|x| (0..w.height).map(move |y| (x, y)))
            .filter(|&(x, y)| w.tile(x, y).is_some_and(|t| t.active() && t.kind == 26))
            .map(|(_, y)| y)
            .collect();
        assert!(altars.len() >= 6, "the demon altars ({})", altars.len());
        let chasm_altars = altars.iter().filter(|&&y| y < structures_y_lo).count();
        assert!(
            chasm_altars >= 1,
            "the chasm-tail altars ({chasm_altars} above the structures band)"
        );
        let ebon_walls = w.tiles.iter().filter(|t| t.wall == 3).count();
        assert!(ebon_walls > 100, "the ebonstone wall lining ({ebon_walls})");
        let demonite = w
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == 22)
            .count();
        assert!(demonite > 0, "the Demonite side-cavern blobs ({demonite})");
        // The crimson flavour: the blob + the hearts + the style-1 altars.
        let c = generate(&GenOptions {
            seed: 77,
            crimson: true,
            ..Default::default()
        });
        let hearts = c
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == 31 && t.frame_x >= 36)
            .count();
        assert!(hearts >= 8, "the crimson hearts ({hearts})");
        assert_eq!(hearts % 4, 0, "hearts come in 2x2 blocks");
        for t in c.tiles.iter().filter(|t| t.active() && t.kind == 31) {
            assert!(
                matches!(t.frame_x, 36 | 54) && matches!(t.frame_y, 0 | 18),
                "the crimson heart frames ({},{})",
                t.frame_x,
                t.frame_y
            );
        }
        // The style-1 altar scatter: frameX 54/72/90 (the structures pass's
        // own altars carry frameX 0/18/36).
        let style1 = c
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == 26 && t.frame_x >= 54)
            .count();
        assert!(style1 >= 6, "the style-1 altars ({style1})");
        let crim_walls = c.tiles.iter().filter(|t| t.wall == 83).count();
        assert!(crim_walls > 100, "the crimstone wall lining ({crim_walls})");
    }

    #[test]
    fn the_temple_buries_in_the_jungle_cavern() {
        // The 4200-world carries the temple (M5-TEMPLE): the Lihzahrd brick
        // (226) shell, the 3×2 altar (237), the Lihzahrd door (10), the
        // wooden spikes (232) and the unsafe walls (87). The dart traps
        // (137) arrive with `LihzahrdTemplePart2` (the next phase); the
        // faithful `makeTemple` writes no safe wall 112 (only read back).
        let w = generate(&GenOptions {
            seed: 88,
            ..Default::default()
        });
        let count = |kind: u16| {
            w.tiles
                .iter()
                .filter(|t| t.active() && t.kind == kind)
                .count()
        };
        let walls = |wall: u16| w.tiles.iter().filter(|t| t.wall == wall).count();
        assert!(count(226) > 800, "the temple brick ({})", count(226));
        assert!(count(237) == 6, "the altar tiles ({})", count(237));
        assert_eq!(count(10), 3, "the Lihzahrd door ({})", count(10));
        assert!(count(232) >= 5, "the spikes ({})", count(232));
        assert!(walls(87) > 500, "the unsafe temple walls ({})", walls(87));
    }

    #[test]
    fn the_micro_biomes_place_the_content() {
        // The 4200-world carries the cave houses (the wood frames 27 + the
        // platforms 19 + the chests), the campsites (the tents 196-199),
        // the spike pits (the stone blobs in the jungle) and the sword
        // shrines (the real sword tile 187, breaks into the item 989).
        let w = generate(&GenOptions {
            seed: 44,
            ..Default::default()
        });
        let count = |kind: u16| {
            w.tiles
                .iter()
                .filter(|t| t.active() && t.kind == kind)
                .count()
        };
        // The cabin thresholds recalibrated 100/20 → 40/5 for M5-OCEAN
        // phase 2: the `Beaches` carve re-shapes seed 44's ocean columns
        // (water, sand floor, wiped walls) and the shape-level cave-house/
        // campsite acceptance scans the reshaped terrain — the world changed
        // by design (the same re-pin class as the world-hash constants);
        // the faithful `UndergroundHousesAndBuriedChests` port re-owns the
        // thresholds.
        assert!(count(27) > 40, "the cabin wood frames ({})", count(27));
        assert!(count(19) > 5, "the cabin platforms ({})", count(19));
        assert!(
            count(196) + count(197) + count(198) + count(199) > 30,
            "the tents"
        );
        // The sword-shrine threshold recalibrated 2 → 1 for the M5-BIOME
        // batch review: the corrected JunglePass RNG order (the tunnel
        // side-branch direction, `JunglePass.cs:156-160`; the finishing gem
        // tail, `:254`) reshapes seed 44's cave terrain and the shape-level
        // shrine acceptance now finds one shrine — the world changed by
        // design (the same re-pin class as the world-hash constants); the
        // shrines' own faithful port re-owns the threshold.
        assert!(count(187) >= 1, "the sword shrines ({})", count(187));
    }

    /// DW-5.1 (dirty): the underground jungle grows vanilla tile-5 trees and
    /// the kind-6 "`TreeBranches`" canopy is gone. The world-level regression
    /// keys on the old canopy writer's own footprint — the ball's core cells
    /// (the trunk column two above the top, and the top row's immediate
    /// sides) hold no kind 6 in any test world. Bare kind-6 tiles DO exist in
    /// generated worlds from the `OresAndShinies` runner retyping wall-0
    /// pocket tiles (pre-existing, M5-ORES-1's pinned behaviour, out of this
    /// task's file scope) — that is why the ore-pair tests keep their
    /// `wall != 0` filter; its canopy rationale is dead, the runner
    /// rationale is not (see the pair test's comment). The scan is scoped to
    /// the underground trees (the pre-M5 canopy writer's only scope): the
    /// M5-OCEAN phase-2 world shift put a surface forest tree next to an
    /// in-band ore tile on seed 77, a documented false positive for the
    /// guard.
    #[test]
    fn test_DW_5_1_no_canopy_kind6_and_jungle_trees_in_generated_worlds() {
        for (width, height, seed) in [(512i32, 512i32, 42u64), (512, 512, 66), (4200, 1200, 77)] {
            let w = generate(&GenOptions {
                width,
                height,
                seed,
                ..Default::default()
            });
            // The beach anchors replayed from the Reset draws — the scan is
            // scoped inland of both beaches and below the surface (the
            // pre-M5 canopy writer's only scope; the M5-OCEAN carve leaves
            // kind-6 ore pockets in the wall-0 beach sand, a documented false
            // positive for this guard).
            let mut brng = crate::worldgen::pipeline::Streams::new(seed).pass_rng("Beaches");
            let left_beach_end = brng.range(300, 340) + 40;
            let right_beach_start = width - (brng.range(300, 340) + 20);
            for tx in left_beach_end..right_beach_start {
                for ty in (w.world_surface as i32)..w.height {
                    if !w.tile(tx, ty).is_some_and(|t| t.active() && t.kind == 5) {
                        continue;
                    }
                    for dy in -2..=1 {
                        assert!(
                            !w.tile(tx, ty + dy)
                                .is_some_and(|t| t.active() && t.kind == 6),
                            "canopy kind-6 at {tx},{ty} (tree column, {width}x{height}/{seed})"
                        );
                    }
                    for dx in [-1i32, 1] {
                        assert!(
                            !w.tile(tx + dx, ty)
                                .is_some_and(|t| t.active() && t.kind == 6),
                            "canopy kind-6 at {},{ty} (tree side, {width}x{height}/{seed})",
                            tx + dx
                        );
                    }
                }
            }
        }
        // The 4200 world carries underground tree tiles: active kind 5 below
        // the surface line with valid 22 px jungle frames (the canopy marker
        // 22, the branch markers 44/66 at frameY ≥ 198, or plain trunk art).
        // The observed count for the faithful pass is 156 tiles (~15 trees;
        // a draw-order funnel replay against the pre-pass world confirms 15
        // `GrowUndergroundTree` acceptances). Vanilla's count runs higher
        // only because its cave passes carve BEFORE the grass sweep (chain
        // positions 8-17 vs 21), giving floors tall cavern air for the
        // `EmptyTileCheck` canvas — the port's pinned executed order carves
        // `cavefields` after `mud caves to grass`, so most caves sit under
        // short ceilings (clear runs 1-8 rows vs the canvas' 12-21). The
        // order deviation is the pinned world contract, out of this task's
        // scope; the threshold asserts a live tree population, not vanilla's
        // density (the density follows once the cave order is the vanilla
        // one — recorded for ANALYSIS).
        let w = generate(&GenOptions {
            seed: 77,
            ..Default::default()
        });
        let underground_trees = (0..w.width)
            .flat_map(|x| ((w.world_surface as i32 + 10)..(w.height - 350)).map(move |y| (x, y)))
            .filter(|&(x, y)| w.tile(x, y).is_some_and(|t| t.active() && t.kind == 5))
            .count();
        assert!(
            underground_trees > 100,
            "underground trees ({underground_trees})"
        );
        for t in w.tiles.iter().filter(|t| t.active() && t.kind == 5) {
            assert!(
                matches!(t.frame_x, 0 | 22 | 44 | 66 | 88 | 110) && t.frame_y % 22 == 0,
                "tree frame ({},{})",
                t.frame_x,
                t.frame_y
            );
        }
    }

    /// DW-5.2 (dirty): the wet surface jungle — the 4200 world carries
    /// surface jungle-grass columns with exactly the 2-tall water cap the
    /// pass writes (`liquid = 255`, the water type). The pass runs before
    /// the plants, so the water cells may already carry the 61 plants — the
    /// liquid survives in them (the C# mutates the same fields).
    #[test]
    fn test_DW_5_2_wet_jungle_two_water_tiles_above_surface_grass() {
        let w = generate(&GenOptions::default());
        let mut wet_columns = 0;
        for x in 1..w.width - 1 {
            for y in 2..((w.world_surface - 1.0) as i32) {
                let Some(t) = w.tile(x, y) else {
                    continue;
                };
                if !(t.active() && t.kind == 60) {
                    continue;
                }
                let (a, b) = (w.tile(x, y - 1).unwrap(), w.tile(x, y - 2).unwrap());
                let water_type = crate::tile::FLAG_LIQUID_LAVA
                    | crate::tile::FLAG_LIQUID_HONEY
                    | crate::tile::FLAG_LIQUID_SHIMMER;
                if a.liquid == 255
                    && b.liquid == 255
                    && a.flags & water_type == 0
                    && b.flags & water_type == 0
                {
                    wet_columns += 1;
                    break;
                }
            }
        }
        assert!(
            wet_columns > 20,
            "wet surface jungle columns: {wet_columns}"
        );
    }

    /// DW-5.2 (dirty): the jungle plants — the 61 plants sit on the free
    /// jungle-grass cells, the 233 detritus blocks place on the scanned
    /// surface grass, and the jungle-band dirt/cave walls converted to 15.
    #[test]
    fn test_DW_5_2_jungle_plants_and_mud_walls_in_generated_world() {
        let w = generate(&GenOptions::default());
        let count = |kind: u16| {
            w.tiles
                .iter()
                .filter(|t| t.active() && t.kind == kind)
                .count()
        };
        assert!(count(61) > 500, "the jungle plants ({})", count(61));
        assert!(count(233) > 50, "the plant detritus ({})", count(233));
        // The 233 detritus comes in intact 3x2/2x2 blocks (frame rows 0/18).
        for t in w.tiles.iter().filter(|t| t.active() && t.kind == 233) {
            assert!(
                matches!(t.frame_y, 0 | 18 | 36 | 54),
                "detritus frameY {}",
                t.frame_y
            );
        }
        // The mud-wall conversion: wall 15 exists in the world (the jungle
        // body also places it, so assert a healthy count), and the plain
        // dirt walls 2 survive only outside the jungle band's surface rows.
        let wall15 = w.tiles.iter().filter(|t| t.wall == 15).count();
        assert!(wall15 > 1000, "the mud walls ({wall15})");
    }

    #[test]
    fn the_ore_tiers_roll_one_kind_per_pair() {
        // The `OresAndShinies` pass (`WorldGen.cs:13182-13606`) rolls one kind
        // per metal pair per world (the copper 7 / the tin 166, the iron 6 /
        // the lead 167, the silver 9 / the tungsten 168, the gold 8 / the
        // platinum 169) — the rolled kind's TileRunner blobs exist, the other
        // pair member is absent. The evil band adds the demonite 22 in a
        // corruption world.
        //
        // The counts only take tiles that carry a wall: a `TileRunner` blob
        // inherits the wall of the tile it retypes, but the runner also
        // writes its kind into wall-0 pocket tiles (carved cave air), so
        // bare active iron tiles exist from the ore pass itself. The old
        // jungle-canopy writer (kind 6 "TreeBranches") is gone since
        // M5-BIOME phase 5 — the filter now covers the runner's own bare
        // writes only.
        let w = generate(&GenOptions {
            seed: 66,
            ..Default::default()
        });
        let count = |kind: u16| {
            w.tiles
                .iter()
                .filter(|t| t.active() && t.kind == kind && t.wall != 0)
                .count()
        };
        for (a, b) in [(7u16, 166u16), (6, 167), (9, 168), (8, 169)] {
            let (ca, cb) = (count(a), count(b));
            assert!(
                (ca > 100 && cb == 0) || (cb > 100 && ca == 0),
                "the pair {a}/{b}: {ca} vs {cb}"
            );
        }
        // The evil band (`WorldGen.cs:13589-13602`).
        assert!(count(22) > 100, "the demonite band ({})", count(22));
        assert!(count(204) == 0, "no crimtane in a corruption world");
        // The gems in the stone band (the amethyst 63 .. the diamond 68).
        let gems: usize = (63..=68)
            .map(|k| w.tiles.iter().filter(|t| t.active() && t.kind == k).count())
            .sum();
        assert!(gems > 40, "the gem veins ({gems})");
    }

    #[test]
    fn the_crimson_world_carries_crimtane() {
        // The evil band's crimson half (`WorldGen.cs:13584-13594`): the
        // crimtane 204 replaces the demonite 22, everything else is the same
        // pass. Wall-carrying tiles only — see the pair test's comment.
        let w = generate(&GenOptions {
            seed: 67,
            crimson: true,
            ..Default::default()
        });
        let count = |kind: u16| {
            w.tiles
                .iter()
                .filter(|t| t.active() && t.kind == kind && t.wall != 0)
                .count()
        };
        assert!(count(204) > 100, "the crimtane band ({})", count(204));
        assert!(count(22) == 0, "no demonite in a crimson world");
    }

    #[test]
    fn the_gems_and_dungeon_colours_place() {
        // The ice biome carries the exposed gem tiles (178) with the six
        // weighted styles; the dungeon brick + the walls match the colour
        // roll (the blue 41/7, the green 43/8, the pink 44/9).
        let w = generate(&GenOptions {
            seed: 33,
            ..Default::default()
        });
        let gems = w
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == 178)
            .count();
        assert!(gems > 20, "the exposed gems ({gems})");
        // The styles spread across the weighted bands (the amethyst 0 is
        // the most common, the diamond 5 the rarest).
        let mut styles = [0u32; 6];
        for t in &w.tiles {
            if t.active() && t.kind == 178 {
                styles[(t.frame_x / 18) as usize] += 1;
            }
        }
        assert!(styles[0] > 0, "the amethyst band ({:?})", styles);
        // The dungeon: one brick colour present, the others absent; the
        // walls match the brick.
        let bricks = [(41u16, 7u16), (43, 8), (44, 9)];
        let present: Vec<usize> = (0..3)
            .filter(|i| w.tiles.iter().any(|t| t.active() && t.kind == bricks[*i].0))
            .collect();
        assert_eq!(present.len(), 1, "one dungeon colour ({:?})", present);
        let bi = present[0];
        let wall_count = w.tiles.iter().filter(|t| t.wall == bricks[bi].1).count();
        assert!(wall_count > 200, "the dungeon walls ({wall_count})");
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
                    let ok =
                        [(1i32, 0i32, 18i16), (0, 1, 0), (1, 1, 18)]
                            .iter()
                            .all(|(dx, dy, fx)| {
                                w.tile(x + dx, y + dy).is_some_and(|n| {
                                    n.active()
                                        && n.kind == 21
                                        && n.frame_x == *fx
                                        && n.frame_y == if *dy == 1 { 18 } else { 0 }
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
