//! The worldgen world hash — the port of `WorldGenerator.HashWorld`
//! (`WorldBuilding/WorldGenerator.cs:517-545`), the per-pipeline fingerprint
//! vanilla records into `GenPassResult.Hash` and compares across snapshot
//! runs (`CheckLatestPassResultAgainstManifest`).
//!
//! The fold structure is verbatim: per column, XOR the tile hashes and mix
//! with `h = rotl13(h) * 5 + 3864292196` (the FNV-1a finaliser family,
//! unchecked u32 arithmetic), then the same mix across the column hashes.
//! The per-tile field mix is the port's own — C# hashes via
//! `TileSnapshot.TileStruct.GetHashCode()`, whose .NET internals are not
//! worth reproducing; what matters is that every stored `Tile` field is
//! covered (kind, wall, frame_x, frame_y, liquid, flags, meta, wall_frame —
//! the state the `.trwld` save serialises) and the result is stable.
//!
//! Two distinct pins live here:
//! * **the algorithm** — `mix` (`*5 + 3864292196`), the per-tile `*31` field
//!   mix and the `rotl13` fold order — is frozen by
//!   `the_fnv_constants_are_frozen_by_a_fixed_input` over a fixed synthetic
//!   grid (derived independently, not from this code). It only moves if the
//!   hash *itself* changes.
//! * **the generator output** — `SEED42_512` / `DEFAULT_WORLD` — is a
//!   regression tripwire for the port's **own** pipeline: a faithful worldgen
//!   change moves it by design and the constant is re-pinned. It is *not* a
//!   proof of C# parity (the per-tile mix is the port's own, so the number is
//!   not comparable to vanilla's `HashWorld`); that proof is S1-A-7.

use crate::tile::Tile;
use crate::world::World;

const FNV_STEP: u32 = 5;
const FNV_BIAS: u32 = 3864292196;

#[inline]
fn mix(h: u32) -> u32 {
    h.wrapping_mul(FNV_STEP).wrapping_add(FNV_BIAS)
}

/// The per-tile field mix — every stored `Tile` field, folded in a fixed
/// order.
#[inline]
fn tile_hash(t: &Tile) -> u32 {
    let mut h = t.kind as u32;
    h = h.wrapping_mul(31).wrapping_add(t.wall as u32);
    h = h.wrapping_mul(31).wrapping_add(t.frame_x as u32);
    h = h.wrapping_mul(31).wrapping_add(t.frame_y as u32);
    h = h.wrapping_mul(31).wrapping_add(t.liquid as u32);
    h = h.wrapping_mul(31).wrapping_add(t.flags as u32);
    h = h.wrapping_mul(31).wrapping_add(t.meta as u32);
    h.wrapping_mul(31).wrapping_add(t.wall_frame as u32)
}

/// `WorldGenerator.HashWorld` over the port's tile storage (row-major
/// `tiles[y * width + x]`; the fold is still column-major like the C#).
pub(crate) fn world_hash(world: &World) -> u32 {
    let (width, height) = (world.width, world.height);
    let mut hash: u32 = 0;
    for x in 0..width {
        let mut line: u32 = 0;
        for y in 0..height {
            line ^= tile_hash(&world.tiles[(y * width + x) as usize]);
            line = (line << 13) | (line >> 19);
            line = mix(line);
        }
        hash ^= line;
        hash = (hash << 13) | (hash >> 19);
        hash = mix(hash);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worldgen::GenOptions;

    /// The constants captured from the PRE-scaffold `generate()` output
    /// (the straight-line body, before it was rewired through
    /// `pipeline::run`) — they are the byte-identity proof: the scaffold
    /// must reproduce them exactly, across two runs each. Re-pinned for
    /// M5-ORES-1: the ore step now runs the faithful `OresAndShinies`
    /// TileRunner pass (ANALYSIS "OresAndShinies fidelity (M5-ORES-1)"),
    /// which changes every world on every seed by design. Re-pinned again
    /// for M5-BIOME phase 3: the faithful `JunglePass` replaces the
    /// shape-level band (and leaves the shared `rng` stream), `Mud Caves To
    /// Grass` and `Dirt To Mud` join as their own steps, and the wall-arm /
    /// slope-fix TileRunner semantics landed. Re-pinned for M5-BIOME
    /// phase 4: the faithful `CorruptionAndCrimson` replaces the shape-level
    /// evil band, moves to the vanilla slot between `ores and gems` and
    /// `liquids` (its own `pass_rng("Corruption")` stream), and the
    /// frame-important 26/31 boot statics landed — every world on every
    /// seed changes by design (the ANALYSIS section lands in Phase 7).
    /// Re-pinned for M5-BIOME-2: the three micro-biome passes land — the
    /// faithful `GlowingMushroomPatches` (`ShroomPatch` walk + the
    /// `SpreadGrass(59, 70)` sweep + the tidy sweep) on its own
    /// `pass_rng("GlowingMushroomPatches")` stream, the marble slab caves
    /// (`MarbleBiome` port) on `pass_rng("Marble")` and the granite pressure
    /// simulation (`GraniteBiome` port) on `pass_rng("Granite")`, and the
    /// granite step moves after the cave carve (the vanilla order — its
    /// CanPlace gate needs the pre-carved caves). Every world on every seed
    /// changes by design (the ANALYSIS section lands with the branch).
    /// Re-pinned for the M5-DUNGEON entrance-hall precalculation
    /// (`MakeDungeon_GenerateNextEntranceHall_Precalculated`): the brick
    /// shell now rides ±(strength+4) around the surface entrance, so every
    /// world with a dungeon changes — `SEED42_512` included (the entrance
    /// tower's shell above the shaft).
    /// `SEED42_512` is UNCHANGED by the phase-4 move: on a 512-wide world
    /// the evil pass runs nothing (the sub-vanilla width gate skips it —
    /// vanilla's centre draw `Next(500, maxTilesX - 500)` throws there) and
    /// the old shape pass drew nothing either (its spans collapsed).
    /// Re-pinned for the batch-review RNG-order fixes: the tunnel
    /// side-branch direction now maps `Next(2) == 0` to +2 and the
    /// finishing gem tail draws `type:` first (`JunglePass.cs:156-160`,
    /// `:254` — named arguments evaluate in source order) — the same draw
    /// counts with different outcomes, so every world shifts again.
    /// Re-pinned for M5-BIOME phase 5: the faithful jungle detail passes
    /// land — `UndergroundJungleTrees` on its own
    /// `pass_rng("Jungle Trees")` stream (tile-5 trees replace the kind-6
    /// canopy; the later `drng` consumers shift along that stream — the
    /// accepted world change), `Wet Jungle` and `Muds Walls In Jungle`
    /// join at their vanilla slots, the plants pass takes its faithful
    /// jungle arms plus the `JunglePlantsPart2` detritus sweep, and
    /// `GenVars.jungle_min_x/max_x` gain their writer — every world on
    /// every seed changes by design (the ANALYSIS section lands in
    /// Phase 7).
    /// Re-pinned for M5-BIOME phase 6: the temple placement becomes the
    /// faithful `LihzahrdTemple` search (`WorldGen.cs:15834-15926`) on its
    /// own `pass_rng("Jungle Temple")` stream — the temple relocates to the
    /// search's bury point (the sub-vanilla worlds take the C# fallback via
    /// the height valve), the shared `srng` stream loses the pass's draws
    /// (the later `srng` consumers shift — the accepted world change), the
    /// shell publishes `GenVars.temple`, and 137/237 join the frame-
    /// important boot statics — every world on every seed changes by
    /// design (the ANALYSIS section lands in Phase 7).
    /// Re-pinned for M5-OCEAN phase 1: the vanilla beach geometry lands —
    /// `GenVars.leftBeachEnd`/`rightBeachStart` drawn at the head of the
    /// terrain step from the `Beaches` stream (`WorldGen.cs:11415-11440`:
    /// `Next(300,340)`, the fixed-Right dungeon side adds 40 left / 20
    /// right), the compact t²·30 edge ramp retires for the vanilla flat
    /// shelf at sea level, and `SurfaceMap.beach` splits into the two
    /// anchors (the living-tree anchors, the jungle/dungeon spans, the
    /// edge flood and the sand-top rule all ride the new values — every
    /// world on every seed changes by design).
    /// Re-pinned for M5-OCEAN phase 2: the ocean sand/water family lands —
    /// the faithful `Ocean Sand` (`WorldGen.cs:11580`) after the spans
    /// planner, `Beaches` (`:14888`) and `Gravitating Sand` (`:15138`)
    /// between `dungeon` and `ocean caves` (the carve itself: air/water/
    /// sand prism, `TuneOceanDepth`'s seafloor curve, the wall wipe, the
    /// shell anchors), the compact edge flood and fill sand-top rule
    /// retire — every world on every seed changes by design.
    /// Re-pinned for M5-OCEAN phase 3: the shape-level ocean-cave
    /// placeholder retires for the faithful `WorldGen.oceanCave` walker
    /// (`WorldGen.cs:76452-76713` — the shrinking ellipse, the sentinel-264
    /// carve marker, the stalactite columns, the water shafts) on its own
    /// `pass_rng("Create Ocean Caves")` stream (was `streams.drng` — the
    /// accepted world change, the later `drng` consumers shift) — every
    /// world on every seed changes by design.
    /// Re-pinned for M5-OCEAN phase 4: the late water/sand passes land —
    /// `Remove Water From Sand` (`WorldGen.cs:16211`) and `Shell Piles`
    /// (`:16266`) before `slopes` (vanilla `:16211`/`:16266` → Smooth World
    /// `:16419`), and the `Water Plants` seaweed arm (`:21973`) after the
    /// vegetation family (its per-pass stream) — the shell blobs retype the
    /// shore sand, the inland water clears, the seaweed grows — every world
    /// on every seed changes by design.
    /// Re-pinned for M5-TEMPLE phase 3: `templePart2` lands
    /// (`WorldGen.cs:34464-34658` — the dart traps with their wire runs, the
    /// buried chests, the statues, the workbench/table/chair furniture and
    /// the painting) plus the `LihzahrdAltar` re-write (`:21943`) — every
    /// world on every seed changes by design. These are the final M5-TEMPLE
    /// pins.
    /// Re-pinned for M5-STRUCT-5 over the M5-TEMPLE pins: the cave decorations
    /// land — the spider caves (`Spread.Spider`'s wall 62 over the cave
    /// pockets, with the pots/piles/stalactite decor), the gem caves (the gem
    /// walls 48-53 and the rim conversion), the moss (the cave walls 54-58,
    /// the neon-moss biomes, the scatters) and the `Webs And Honey` cobweb
    /// pass — every world on every seed changes by design.
    // M5-DUNGEON (2026-11-09): the pipeline's `Dungeon` step moved from the
    // shape-level approximation (`biomes::pass_dungeon`) to the faithful
    // crawler (`worldgen/dungeon.rs::generate_dungeon`), so the generated
    // worlds carry the crawler's halls/rooms instead of the corridor
    // approximation.
    // Re-pinned by M5-BIOME-2's second slice: the honey patches ride the
    // hive pass (`WorldGen.cs:15946-15968`), so every world with a hive
    // gains its `Next(5)` pockets. The determinism half of both tests
    // (a == b) still guards the stream.
    // Re-pinned by M5-DUNGEON's banner pass (`DungeonGlobalBanners`,
    // `DungeonCrawler.cs:503`): `200 * maxTilesX / 4200` banners (tile 91, three
    // rows each) in the colour's `bannerStyles` 10..15, so every dungeon moves.
    // Re-pinned by M5-DUNGEON's trap pass (`DungeonGlobalTraps`,
    // `DungeonCrawler.cs:497`): dart traps with their pressure plates and wire
    // runs, so every dungeon's hash moves once more. (The 512-wide test world
    // sits under its own surface line, so only the stream moved here; the
    // full-size worlds do place traps.)
    // Re-pinned by M5-DUNGEON's wall-variant pass (`DungeonGlobalWallVariants`,
    // `DungeonCrawler.cs:483`): five rounds of disc-seeded flood fills repaint
    // the dungeon's base brick wall with the colour's painted variants
    // (94/95, 98/99, 96/97), so every dungeon's walls move again.
    // Re-pinned by M5-DUNGEON's light pass (`DungeonGlobalLights`, the
    // `:0.94` step): lanterns, chandeliers and the wired switch each light
    // carries, plus the three lantern style rolls, so every dungeon's hash
    // moves again.
    // >>> cc526b1 (Merge branch 'M5-DUNGEON-lights': DungeonGlobalLights + the restored spike pass)
    // Re-pinned by BUG-25 (`worldgen/mini_biomes.rs` `pass_cave_houses`): the
    // cave houses carry the `WoodHouseBuilder.cs:12-13` constants now — the
    // `TileType` 30 (`WoodBlock`) frame and the `WallType` 27 (`Planked`)
    // interior walls — instead of the Sunflower tile 27 frame and the pink
    // dungeon wall 9 interior, so every world's tile/wall fields move.
    // Re-pinned again by S1-A-3 (framing variants): the full-world framing
    // passes now roll a `genRand.Next(0, 3)` variant per framed tile and store
    // it (`WorldGen.cs:81705-81709` / `Framing.cs:376-404`) instead of the
    // position hash, so every merge-variant/large-frame tile's frame_x/frame_y
    // changes — every hash moves. (The `variants` field itself is not hashed;
    // the frame pixels are.) Values measured on the merged tree (BUG-25 +
    // S1-A-3 together).
    const SEED42_512: u32 = 3221031423;
    const DEFAULT_WORLD: u32 = 235359465;
    // >>> cc526b1 (Merge branch 'M5-DUNGEON-lights': DungeonGlobalLights + the restored spike pass)

    #[test]
    fn the_512_seed_42_world_hashes_to_the_pinned_constant() {
        let opts = GenOptions {
            width: 512,
            height: 512,
            seed: 42,
            ..Default::default()
        };
        let a = world_hash(&crate::worldgen::generate(&opts));
        let b = world_hash(&crate::worldgen::generate(&opts));
        assert_eq!(a, b);
        assert_eq!(a, SEED42_512);
    }

    #[test]
    fn the_default_world_hashes_to_the_pinned_constant() {
        let opts = GenOptions::default();
        let a = world_hash(&crate::worldgen::generate(&opts));
        let b = world_hash(&crate::worldgen::generate(&opts));
        assert_eq!(a, b);
        assert_eq!(a, DEFAULT_WORLD);
    }

    /// The **frozen** hash: `mix` (`*5 + 3864292196`), the per-tile `*31` field
    /// mix and the column/whole-world `rotl13` order, pinned against a fixed
    /// synthetic grid. Unlike `SEED42_512`/`DEFAULT_WORLD` (generator-output
    /// tripwires that move on *every* worldgen fidelity change), these constants
    /// change only if the hash algorithm itself changes. The value was derived
    /// independently — a separate implementation of the
    /// `WorldGenerator.HashWorld` fold (`WorldGenerator.cs:517-545`), not copied
    /// from this port — so the pin is not self-referential.
    ///
    /// `grid[i]`, row-major `i = y * 4 + x` (`W = 4`, `H = 3`), is:
    /// `kind = (i*3+1)%7`, `wall = (i*5+2)%4`, `frame_x = (i*7)%9`,
    /// `frame_y = (i*11)%10`, `liquid = (i*13)%200`, `flags = (i*17)%256`,
    /// `meta = (i*19)%256`, `wall_frame = (i*23)%256`.
    fn frozen_grid() -> World {
        let mut w = World::new("golden".into(), 4, 3);
        for i in 0..(4 * 3) {
            let t = &mut w.tiles[i];
            t.kind = ((i * 3 + 1) % 7) as u16;
            t.wall = ((i * 5 + 2) % 4) as u16;
            t.frame_x = ((i * 7) % 9) as i16;
            t.frame_y = ((i * 11) % 10) as i16;
            t.liquid = ((i * 13) % 200) as u8;
            t.flags = ((i * 17) % 256) as u8;
            t.meta = ((i * 19) % 256) as u8;
            t.wall_frame = ((i * 23) % 256) as u8;
        }
        w
    }

    #[test]
    fn the_fnv_constants_are_frozen_by_a_fixed_input() {
        let w = frozen_grid();
        assert_eq!(world_hash(&w), 2634455973, "the world fold");
        assert_eq!(tile_hash(&w.tiles[0]), 3517817697, "tile 0");
        assert_eq!(tile_hash(&w.tiles[1]), 1245549601, "tile 1");
        assert_eq!(tile_hash(&w.tiles[11]), 2902946927, "tile 11");
    }

    #[test]
    fn the_hash_covers_every_tile_field() {
        let mut w = World::new("t".into(), 8, 8);
        let base = world_hash(&w);
        w.tiles[0].kind = 5;
        assert_ne!(world_hash(&w), base, "kind");
        w.tiles[0].kind = 0;
        w.tiles[0].wall = 3;
        assert_ne!(world_hash(&w), base, "wall");
        w.tiles[0].wall = 0;
        w.tiles[0].frame_x = 4;
        assert_ne!(world_hash(&w), base, "frame_x");
        w.tiles[0].frame_x = 0;
        w.tiles[0].frame_y = 2;
        assert_ne!(world_hash(&w), base, "frame_y");
        w.tiles[0].frame_y = 0;
        w.tiles[0].liquid = 9;
        assert_ne!(world_hash(&w), base, "liquid");
        w.tiles[0].liquid = 0;
        w.tiles[0].flags = 1;
        assert_ne!(world_hash(&w), base, "flags");
        w.tiles[0].flags = 0;
        w.tiles[0].meta = 7;
        assert_ne!(world_hash(&w), base, "meta");
        w.tiles[0].meta = 0;
        w.tiles[0].wall_frame = 11;
        assert_ne!(world_hash(&w), base, "wall_frame");
        w.tiles[0].wall_frame = 0;
        assert_eq!(world_hash(&w), base);
    }
}
