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
    // Re-pinned by S1-K-2 wave A (`worldgen/mini_biomes.rs`
    // `pass_mahogany_trees`): the Micro Biomes pass's `MahoganyTreeBiome`
    // block (`WorldGen.cs:21105-21123`) lands — the living mahogany trees
    // (383/384, wall 78) grow in the jungle's mud caverns on the fresh
    // `pass_rng("Micro Biomes")` stream. The 512-wide pin is UNCHANGED:
    // the sub-vanilla guard skips the placement block there (the C#'s
    // `Next(worldSurface + 50, maxTilesY - 500)` band collapses), and the
    // framing's 383/384 merge arms (`WorldGen.cs:82077-82082`) only fire
    // on the new kinds.
    // Re-pinned at the merge gate for fix/vines-destroy-walls: the Vines
    // pass write is now the C#'s field-only form (`WorldGen.cs:20246-20251`
    // — active/type/ClearSlope), so generated vine cells KEEP their
    // wall/wall_frame/liquid/paint instead of zeroing them. The RNG draw
    // order is unchanged (the write draws nothing) — the a == b determinism
    // asserts stayed green on the failing gate run; only the pinned values
    // moved. Old -> new: 2673964944 -> 124297949 (measured on the merged
    // tree, gate log TR-LD-gate-vines.log).
    // S1-K-2 wave B (code-only): `pass_mining_explosives` lands after the
    // sword shrines — the pin stays UNCHANGED for the same reason: the
    // `RandomWorldPoint(rockLayer, 380, 200, 380)` x-band
    // `Next(380, maxTilesX - 380)` is empty on a 512-wide world (the C#
    // throws there; the port's width guard skips the block before any
    // draw). The 141/411 frame-important boot bits are inert without the
    // kinds. The DEFAULT_WORLD re-pin is deferred to the gate (below).
    // S1-P-i33 (code-only): the wall-framing mask fix (see DEFAULT_WORLD
    // below) moves this pin too — glass & co. beside a wall change the
    // generated `wall_frame`. Stale-by-design; the gate re-pins it.
    // W4-f25 (2026-11-20, code-only): the `Settle Liquids` pass
    // (`WorldGen.cs:16132-16210`, `settle_liquids.rs::settle_liquids`) lands
    // immediately before `remove water` — the first real liquid-simulation
    // pass. On a 512-wide world the Lakes/ocean water now drops, fans and
    // levels through `QuickWater`/`SettleWaterAt` + the 10-round quickSettle
    // loop, and the water-line lava retype fires below `GenVars.waterLine`,
    // so the liquid fields move BY DESIGN. The pass draws NO RNG (the
    // kill_tile drops ride deterministic position seeds), so every other
    // step's stream is untouched. The re-pin is DEFERRED to the merge gate
    // (no compiler on this branch); the a == b determinism assert stays.
    // W4-g42 (2026-11-21, code-only): the glowing-mushroom biome lane moves
    // this pin BY DESIGN — the `GlowingMushroomPatches` verbatim deltas
    // (the double cluster bound, the ShroomPatch first-ring activation guard
    // + the py advance + the vy one-off, the underground-desert veto) change
    // the mud/grass tiles, and the plants-pass mushroom arm now grows the
    // tile-5 giant trees (the triple `GrowTree` retry) instead of the
    // shape-level 71 write. The 512-wide world runs 1 cluster (512/700 →
    // ceil 1) whose patches + trees land. The re-pin is DEFERRED to the
    // merge gate (no compiler on this branch); the `a == b` determinism
    // half stays untouched.
    // W4-c31 (2026-11-21, code-only): the faithful hive rewrite
    // (`worldgen/hive.rs` — `HiveBiome.cs:11-383` + the `Beehives`
    // `WorldGen.cs:15927-15976` and `BeeLarvaInBeehives` `:20868-20900`
    // passes) replaces the shape-level dome. The pass keeps the shared
    // `srng` stream but its DRAW SCHEDULE differs from the dome's (the
    // tunnel chains' per-cell dither rolls, the honey-fall walks, the
    // RandomWorldPoint origins vs the dome's jungle-span picks), so every
    // downstream srng consumer re-rolls and the hive/larva tiles change.
    // BOTH pins move BY DESIGN even on the 512 world (the count roll
    // `1 + Next(0, 0)` = 1 hive still runs; the sub-vanilla band guard
    // only skips when `maxY - 300 <= (surface+rock)>>1`, which a 512×512
    // world's shallow layers can clear). Stale-by-design; the gate
    // re-pins. The `a == b` determinism assertion stays green — the pass
    // is a pure function of (world, stream), no ambient state.
    // W4-c31 jungle residuals (2026-11-21, code-only): the faithful
    // `JungleShrines` rewrite (`worldgen/structures.rs`,
    // `WorldGen.cs:15977-16131`) replaces the shape-level hut. The pass
    // moves OFF the shared `srng` onto its own fresh
    // `pass_rng("Jungle Chests")` slot, so every downstream `srng` consumer
    // (the chests, altars, pyramids, pots, cave houses, mini-biomes) re-rolls
    // along the stream, and the shrine tiles themselves change (the
    // `GenVars.jungleHut` material, the proximity veto, the torch/floor/
    // pillars/roof sweeps). The Reset stand-in also draws `jungle_hut`
    // (`Next(5)`) AFTER `crimson_left`, shifting the Reset-stream tail.
    // BOTH pins move BY DESIGN. On the 512 world the shrine y band
    // `[(surface+rock)/2, maxY-400)` collapses (the sub-vanilla valve skips
    // the pass), but the `srng` schedule shift + the Reset `jungle_hut` draw
    // still move the downstream tiles. Stale-by-design; the gate re-pins.
    // The `a == b` determinism assertion stays green — the pass is a pure
    // function of (world, stream), no ambient state.
    // W4-g44 (2026-11-21, code-only): the verbatim FloatingIslands +
    // FloatingIslandHouses passes land (`worldgen/islands.rs` — the count
    // `(int)(width * 0.0008)` + the skyLakes tail-lake cadence, the
    // column-scan y-band, the sunplate-202/disc-wall-82 houses on their
    // own `pass_rng` slots). On a 512-wide world the pass still runs
    // (num = 0, skyLakes = 1 — one lake attempt; the column scan's
    // sub-vanilla valve may skip it), and the shared `drng` loses the old
    // islands' draws — but every remaining `drng` consumer runs EARLIER,
    // so the 512 pin moves only if the lake disc places. Stale-by-design
    // either way; the gate re-pins it. The `a == b` determinism asserts
    // stay green (no new entropy source — the passes are seed-keyed).
    // W4-g43 (2026-11-21, code-only): the temple residuals round moves this
    // pin BY DESIGN — the painting's Place3x3Wall frames (the 3×3 walled
    // write, style 88 → 864-900/108-144), the trap/plate/altar/furniture
    // SquareTileFrame tails (the support-check cascade can pop an
    // unsupported object), the unconditional trap colour draw (the stream
    // shifts wherever a plate cell re-uses a wire) and the plate pre-Clears
    // all change the generated temple tiles. The re-pin is DEFERRED to the
    // merge gate (no compiler on the branch); the a == b determinism
    // asserts and the w4_g43 whole-world witness guard the streams, and the
    // measured value rides that test's log for the gate's re-derivation.
    // W4-g41 (code-only): the egg-sac validator + the new decor arms (see
    // DEFAULT_WORLD below) move this pin too — the 512 world's spider caves
    // restyle/pop sacs and the framer keeps the 165 frames. Stale-by-design;
    // the gate re-pins it.
    // W4-g41 follow-up (code-only): the NOT-THE-BEES host (`not_the_bees.rs`)
    // is flag-gated on NOT_THE_BEES — a classic 512 world never enters it, so
    // this pin is UNCHANGED by the host port (the egg-sac deferral above
    // stands). Census: `docs/research/94-not-the-bees-host-census.md`.
    // W4-g41 finish gaps (2026-11-21, code-only): `finish_no_traps.rs`
    // (`WorldGen.cs:25986-26224`) and `finish_remix.rs` (`:26350-26770`)
    // land at `step_seed_finish`'s C# slots (`:22423-22426` FIRST /
    // `:22444-22447` after FTW). BOTH are flag-gated (NO_TRAPS / REMIX) -
    // a classic 512 world enters neither and draws nothing, so this pin
    // is UNCHANGED by the two hosts (the egg-sac deferral above stands).
    // The hosts draw on fresh `pass_rng("FinishNoTraps")` /
    // `("FinishRemixWorld")` streams; the a == b witnesses are written
    // NOT run. Census: `docs/research/95-finish-no-traps-remix-census.md`.
    // W4-g40 (code-only): the marble/granite residual places nothing on a
    // 512-wide world (both Count draws truncate to 0), so the residual
    // itself leaves this pin unchanged; the P0 hive-rising stalactite arm
    // may move it via the spider-cave decor — the gate re-pins or confirms
    // (see the DEFAULT_WORLD note). The entrance residual (research 222,
    // renumbered from 94)
    // CAN fire on the 512 world (the 36-wide desert band's entrance
    // fraction) — its Blotches ledger may move this pin too; same gate.
    // W4-g40 hive-smooth (research 248, renumbered 246-248 at merge): the PostPlacementEffect walk adds
    // slope/half-brick states inside the hive — RNG-free, but the tile
    // bytes move, so this pin is stale-by-design too when the hive fires.
    // W4-d103 pass 19 (2026-11-20, code-only): the `Oasis` pass
    // (`WorldGen.cs:16243-16265` + `PlaceOasis` `:10707-10932`) lands on a
    // fresh `pass_rng("Oasis")` stream between `remove water` and
    // `shell piles`, and the `Cactus, Palm Trees, & Coral` oasis block
    // (`:21326-21363`) wakes over the new `GenVars.numOasis` ledger.
    // This pin is UNCHANGED by design: the pass's sub-vanilla width guard
    // (`Next(680, maxTilesX - 680)` needs > 1360 wide) skips before any
    // draw on the 512-wide world, and the oasis block draws nothing with
    // `numOasis == 0`. The DEFAULT_WORLD re-pin below is deferred to the
    // merge gate (no compiler on this branch); the a == b determinism
    // asserts stay in place.
    // W4-d103 pass 20 (2026-11-20, code-only): the `Waterfalls` pass
    // (`WorldGen.cs:16610-16666`) lands on a fresh `pass_rng("Waterfalls")`
    // stream after `living trees`. Unlike the Oasis width guard, the two
    // sweeps run at EVERY width (the 20-tile margin only), so this pin is
    // stale-by-design IF the 512 fixture carries a qualifying liquid edge
    // (a solid cell with air on both sides beside liquid, or a half-brick
    // shore) — unmeasured, no compiler on this branch. The gate re-pins
    // whichever pin moved; the a == b determinism asserts stay in place
    // (the pass draws only its own fresh stream).
    // W4-d103 pass 21 (2026-11-20, code-only): the `Wall Variety` pass
    // (`WorldGen.cs:16697-16754`) lands on a fresh `pass_rng("Wall Variety")`
    // stream after `waterfalls`. The count is `(int)(300 * 512*512/5040000)`
    // = 15 placements; the point band `Next(surface, 512-190)` is non-empty
    // on the 512 fixture, so the pass RUNS there — whether a qualifying
    // stone-floor pocket (>50 cells, drained, clean) exists is unmeasured
    // (no compiler on this branch). Stale-by-design if one does; the gate
    // re-pins. a == b stays (the pass draws only its own fresh stream).
    // W4-d103 pass 22 (2026-11-20, code-only): the `Life Crystals` pass
    // (`WorldGen.cs:16755-16852`) lands on a fresh `pass_rng("Life Crystals")`
    // stream after `wall variety`. The classic y band needs height − 300
    // above `(2*surface + rockLayer)/3` — on the 512-tall fixture the band
    // is narrow but non-empty IF the surface sits low enough; the size
    // guard skips before any draw otherwise. Unmeasured (no compiler on
    // this branch); stale-by-design if crystals place. a == b stays.
    // W4-d103 pass 23 (2026-11-20, code-only): the `Statues` pass
    // (`WorldGen.cs:16853-16970`) lands on a fresh `pass_rng("Statues")`
    // stream after `life crystals`. The count is
    // `(int)(73*2 * 512/4200)` = 17 on the 512 fixture; the size guard
    // needs height − 300 above the y-band floor — 512 − 300 = 212, so the
    // pass runs IF the band floor sits below 212 (unmeasured). The new
    // 105/349/135 frame-important boot bits are inert without the kinds.
    // Stale-by-design if statues place; the gate re-pins. a == b stays.
    // W4-d103 pass 24 (2026-11-20, code-only): the `Surface Chests` pass
    // (`WorldGen.cs:17130-17210`) lands on a fresh `pass_rng("Surface
    // Chests")` stream after `statues`, through the faithful AddBuriedChest
    // core (the Reset hell-chest shuffle now draws 5 words on the Reset
    // stream — the crimson roll's stream tail moves, but no executed pass
    // reads the Reset stream after Reset, so the tiles are untouched by
    // the shuffle alone). The size guard needs > 600 wide — the 512
    // fixture skips before any draw, so this pin is UNCHANGED by the pass
    // itself. a == b stays.
    // W4-d103 pass 25 (2026-11-20, code-only): the `Water Chests` pass
    // (`WorldGen.cs:17245-17355`) lands on a fresh `pass_rng("Water
    // Chests")` stream after `surface chests`. The size guard needs
    // UnderworldLayer (512−200=312) > 50 and the surface below it — the
    // 512 fixture PASSES the guard, and the scatter's unbounded water
    // re-draw spins on a waterless world. The fixture carries ocean water
    // (the Beaches carve), so the pass runs; whether a style-17 chest
    // lands is unmeasured (no compiler on this branch). Stale-by-design
    // if one does; the gate re-pins. a == b stays.
    // W4-d103 pass 26 (2026-11-20, code-only): the `Cave Walls` pass
    // (`WorldGen.cs:17711-17839`) lands on a fresh `pass_rng("Cave Walls")`
    // stream after `water chests`. The size guard needs > 400 wide — the
    // 512 fixture PASSES it (512 > 400), and the y band (512−220=292)
    // sits above the (surface+rock)/2 floor, so the pass runs; whether a
    // counted pocket qualifies is unmeasured (no compiler on this branch).
    // Stale-by-design if walls paint; the gate re-pins. a == b stays.
    // W4-d103 pass 27 (2026-11-20, code-only): the `Floating Island Houses`
    // pass (`WorldGen.cs:17859-17874` + IslandHouse `:79203-79484`) lands
    // on a fresh `pass_rng("Floating Island Houses")` stream after `cave
    // walls`, and the islands pass now DEFERS its house build to this slot
    // (the vanilla ledger split — the inline shape-level plank box is
    // retired for the faithful sunplate house). The 512 fixture's islands
    // pass skips (the sky band is too thin), so the ledger is empty and
    // this pin is UNCHANGED by the split itself. a == b stays.
    // W4-d103 tail P2/P3/P4 (2026-11-21, code-only): the hardmode transform
    // rewrite (the initializeHardMode head draws + the dungeon-side bias +
    // the remix band + GERunner verbatim) runs only on the hardmode switch
    // / the start_in_hardmode micro-seed — no classic pipeline step calls
    // it. This pin is UNCHANGED. a == b stays.
    // W4-d103 tail P8/P11/P13 (2026-11-21, code-only): the hardUpdateWorld
    // rewrite (the hardModeWorldUpdates gate, the chlorophyte arms, the
    // verbatim infection loops) is RUNTIME-only — `UpdateWorld` runs on the
    // fixed-step growth tick, never inside the generation pipeline. This
    // pin is UNCHANGED. a == b stays.
    // W4-d103 tail P9 (2026-11-21, code-only): the underground moss arm
    // (`UpdateWorld_UndergroundTile`'s `:73699-73755` branch) is RUNTIME-only
    // — the growth tick, not the pipeline. This pin is UNCHANGED. a == b
    // stays.
    // W4-d106 R1 (2026-11-20, code-only): the `Grass Wall` pass
    // (`WorldGen.cs:19589-19684` + `Spread.Wall2` `:3416-3524` +
    // `countDirtTiles` `:9637-9681`) lands between `living trees` and
    // `trees` on a fresh `pass_rng("Grass Wall")` stream. The pin is
    // EXPECTED UNCHANGED: the compact terrain's grass tops carry wall 0
    // (walls start at `top + 3`, `terrain.rs:343`) and no earlier executed
    // pass writes wall 63, so neither the seed-cell test (`:19604`) nor
    // the flower speckle (`:19658`) fires — the pass draws its `Next(4)`
    // lattice on its own isolated stream and writes nothing. If a
    // grass-with-wall cell does exist on some seed, the pin moves BY
    // DESIGN; the re-pin is DEFERRED to the merge gate (no compiler on
    // this branch). The a == b determinism asserts stay in place.
    // W4-d106 R2 (2026-11-21, code-only): the `Settle Liquids Again` pass
    // (`WorldGen.cs:21246-21309`) lands before `cacti` — the shared
    // `QuickWater`/`WaterCheck`/quick-settle machinery drains the liquids
    // the late passes disturbed. It draws NO RNG (the settle engine is
    // deterministic over the tile state), so no other step's stream moves;
    // the pin moves BY DESIGN wherever a liquid cell settles/re-types/
    // merges (the lake, the ocean shelves, the lava puddles, the honey).
    // Re-pin DEFERRED to the merge gate; a == b stays green.
    // W4-d106 R3 (2026-11-21, code-only): the `Tile Cleanup` pass
    // (`WorldGen.cs:21520-21942`) lands between `cacti` and `lihzahrd altar`
    // on a fresh `pass_rng("Tile Cleanup")` stream — the whole-world slope
    // clear, the half-brick conversions, the water-death kills, the drip
    // placement and the object frame normalisation all move tile bytes BY
    // DESIGN. The pass draws only its own stream, so no other step's ledger
    // shifts; a == b stays green. Re-pin DEFERRED to the merge gate.
    // W4-d106 R4 (2026-11-21, code-only): the `Stalac` pass
    // (`WorldGen.cs:22038-22120`) lands after `water plants` on a fresh
    // name-seeded `UnifiedRandom` (`pass_unified("Stalac")`) — the gem-tree
    // scatter (583-589) and the stalactite/stalagmite placement move tile
    // bytes BY DESIGN. The pass draws only its own stream; a == b stays
    // green. Re-pin DEFERRED to the merge gate.
    // W4-d106 R5 (2026-11-21, code-only): the `Remove Broken Traps` pass
    // (`WorldGen.cs:22121-22140`) lands after `stalac` — the wire-network
    // BFS strips broken trap circuits (the dungeon/temple/mining-explosives
    // wires), moving tile + wire bytes BY DESIGN. Draws no RNG; a == b
    // stays green. Re-pin DEFERRED to the merge gate.
    // W4-d106 R6 (2026-11-21, code-only): the `Final Cleanup` pass
    // (`WorldGen.cs:22141-22490`) lands between `remove broken traps` and
    // `world tail` — the wall-hole fill, the sand-family fall columns, the
    // boulder normalisation, the dungeon-wall/minecart liquid strips, the
    // painting wall inheritance and the poo-668 scatter move tile bytes BY
    // DESIGN, and the final `Liquid.LiquidCheck` merge sweep re-types/merges
    // liquid cells. The classic body draws only the poo scatter's fresh
    // `pass_rng("Final Cleanup")` stream (the sweep is RNG-free), so no other
    // step's ledger shifts; a == b stays green. The seed `Finish*` arms are
    // flag-gated (a classic world skips them). Re-pin DEFERRED to the merge
    // gate. On the 512-wide pin world the poo scatter's `Next(50, w-50)` /
    // `Next(50, h-200)` ranges stay valid (512 > 250), so the scatter runs.
    // W4-g36 A1 (code-only): the Shimmer pass (`WorldGen.cs:15195-15245`,
    // `shimmer.rs::pass_shimmer`) lands between `ocean caves` and
    // `fragile ice`. This pin is UNCHANGED by this pass alone: the
    // sub-vanilla width guard skips the pass before any draw on a
    // 512-wide world (the x-band `Next(200, (int)(0.11*512))` is empty —
    // the C# throws there; the `pass_mining_explosives` width-guard
    // precedent). The DEFAULT_WORLD re-pin is deferred to the gate.
    // W4-g36 A2 (code-only): the Clean Up Dirt pass
    // (`WorldGen.cs:15246-15359`, `micro_finish.rs::pass_clean_up_dirt`)
    // lands after `shimmer` — the two surface sweeps strip the exposed
    // dirt walls (2/40/64/86) above the surface on EVERY world, so this
    // pin moves BY DESIGN; deferred to the gate.
    // W4-g36 A3 (code-only): the Dirt Rock Wall Runner pass
    // (`WorldGen.cs:15461-15475` + `DirtyRockRunner` `:77145-77210`,
    // `micro_finish.rs::pass_dirt_rock_wall_runner`) lands after
    // `clean up dirt` — the per-column walkers convert wall 2 → 59 on
    // every world with surface dirt walls, so this pin moves BY DESIGN;
    // deferred to the gate.
    // W4-g36 A4 (code-only): the Wood Tree Walls pass
    // (`WorldGen.cs:15717-15749`, `living_trees.rs::wood_tree_walls`)
    // lands after `living trees` — the wall-244 fill behind the living
    // wood crosses moves this pin BY DESIGN (RNG-free, no ledger shift);
    // deferred to the gate.
    // W4-g36 A6 (code-only): the Wall Variety pass
    // (`WorldGen.cs:16697-16754`, `micro_finish.rs::pass_wall_variety`)
    // lands after `slopes` — the cave-wall variety fill (196/204/208/212
    // families) moves this pin BY DESIGN on every world with air pockets;
    // deferred to the gate.
    // W4-g36 A7 (code-only): the Life Crystals pass
    // (`WorldGen.cs:16755-16852`, `structures.rs::pass_life_crystals`)
    // lands after `wall variety` — the 2×2 hearts move this pin BY DESIGN
    // (the y-band `(worldSurface*2 + rockLayer)/3 .. height-300` does not
    // collapse on the 512 world, so the pass runs and seats crystals);
    // deferred to the gate. The sub-vanilla guard only fires when the
    // band genuinely collapses (a deeper surface/rock layer).
    // W4-g36 gap-fill (code-only): the ShimmerCleanUp arm
    // (`shimmer.rs::shimmer_clean_up`, the Final Cleanup `:22472` tail) is
    // UNWIRED — no executed step calls it, so this pin stays UNCHANGED. The
    // merger composes it into g37's Final Cleanup; the 512 world's Shimmer
    // pass takes the sub-vanilla width guard (no shimmer placed →
    // `shimmer_position` None → the arm's zero-sentinel early return), so even
    // wired the arm is a no-op on the 512 pin. a == b untouched.
    // W4-g37 B1 (code-only): the Statues pass (`WorldGen.cs:16853-16970`,
    // `passes_b.rs::pass_statues`) lands before `cave houses`. On a 512-wide
    // world the whole x band sits inside the ±380 `oceanDepths` beach margin
    // (`x < 380 || x > 132` covers every roll), so all 17 candidates are
    // rejected and no statue places — this pin is UNCHANGED by B1 alone (the
    // isolated `pass_rng("Statues")` stream shifts no other ledger). The
    // DEFAULT_WORLD pin below moves BY DESIGN; deferred to the gate. The
    // a == b halves stay untouched.
    // W4-g37 B2 (code-only): the Surface Chests pass
    // (`WorldGen.cs:17130-17210`, `passes_b.rs::pass_surface_chests`) lands
    // after `cave houses` — the count `(int)(512*0.005) = 2` surface chests
    // place on their own `pass_rng("Surface Chests")` stream (the y band sits
    // above the oceanLevel line, so oceanDepths never re-rolls them), moving
    // this pin BY DESIGN; deferred to the gate. The a == b halves stay
    // untouched.
    // W4-g37 B3 (code-only): the Water Chests pass
    // (`WorldGen.cs:17245-17355`, `passes_b.rs::pass_water_chests`) lands
    // after `surface chests` — the ocean-cave treasure chests + the 9-wide
    // water scatter place on their own stream, moving this pin BY DESIGN;
    // deferred. (The 512-world's ocean-cave treasure count may be 0, but the
    // scatter still runs.) a == b untouched.
    // W4-g37 B4 (code-only): the Cave Walls pass
    // (`WorldGen.cs:17711-17839`, `passes_b.rs::pass_cave_walls`) lands after
    // the temple family — the enclosed-space + jungle wall fills repaint the
    // cave walls on their own stream, moving this pin BY DESIGN; deferred.
    // a == b untouched.
    // W4-g37 B5 (code-only): the Floating Island Houses pass
    // (`WorldGen.cs:17859-17874`, `passes_b.rs::pass_floating_island_houses`)
    // now builds the recorded island sites (the FloatingIslands pass no longer
    // builds houses inline) — the sunplate shells + skyware interiors + chests
    // move this pin BY DESIGN; deferred. a == b untouched.
    // W4-g37 B6 (code-only): the Quick Cleanup pass
    // (`WorldGen.cs:17875-17995`, `passes_b.rs::pass_quick_cleanup`) lands
    // after the island houses — the ocean/desert/slope sweep retypes + clears
    // tiles on its own stream (the sand-column fill draws Next(4,7)), moving
    // this pin BY DESIGN; deferred. a == b untouched.
    // W4-g37 B7 (code-only): the Hellforge pass
    // (`WorldGen.cs:18187-18236`, `passes_b.rs::pass_hellforge`) lands after
    // `pots` — the width/200 hellforges place on the hell-fort walls on their
    // own stream, moving this pin BY DESIGN; deferred. a == b untouched.
    // W4-g37 B8 (code-only): the Spreading Grass pass
    // (`WorldGen.cs:18237-18446`, `passes_b.rs::pass_spreading_grass`) lands
    // after `hellforge` — the surface mud/ore conversion + grass spread retype
    // tiles (no RNG draw), moving this pin BY DESIGN; deferred. a == b
    // untouched.
    // W4-g37 B9 (code-only): the Surface Ore and Stone pass
    // (`WorldGen.cs:18447-18512`, `passes_b.rs::pass_surface_ore_and_stone`)
    // lands after `spreading grass` — the surface ore veins + stone blobs
    // place on their own stream, moving this pin BY DESIGN; deferred. a == b
    // untouched.
    // W4-g37 gap-fill (code-only): the Final Cleanup pass
    // (`WorldGen.cs:22141-22490`, `final_cleanup.rs::pass_final_cleanup`)
    // lands after `water plants` — the whole-map sweep runs on the 512 world
    // too (the wall-hole fill, the sand hardening, the dungeon liquid resets
    // are tile-state arms), so this pin moves BY DESIGN; deferred to the
    // gate. Only the 668 dirt-throw scatter is width-guarded out (the
    // `Next(50, maxX-50)` band collapses at 512). a == b untouched.
    // W4-g38 C2 (2026-11-21, code-only): the `Traps` pass
    // (`WorldGen.cs:18657-18787` + `placeTrap` `:8922-9513` + `PlaceSandTrap`
    // `:35463-35739`) lands after `fallen log` on a fresh `pass_rng("Traps")`
    // stream. The pin is EXPECTED UNCHANGED: on the 512-wide world the trap
    // x band `[200, width−200)` = `[200, 312)` is live, BUT the classic
    // trap count is `width * 0.05 = 25` and every attempt's floor walk /
    // headroom scan rejects on the compact 512 terrain (the same reason the
    // dungeon trap pass places none there) — and the sand-trap band
    // `[0.15w, 0.85w)` needs wall 187 the 512 world's desert never paints.
    // If a trap does place on some seed, the pin moves BY DESIGN; re-pin
    // DEFERRED to the merge gate. a == b stays green (own fresh stream).
    // W4-g38 C3 (2026-11-21, code-only): the `Piles` pass
    // (`WorldGen.cs:18788-19459` + `PlaceSmallPile` `:46815-46873` + the
    // `Place3x2`/`PlaceTile` 186/187 slices) lands after `traps` on a fresh
    // `pass_rng("Piles")` stream. This pin MOVES BY DESIGN: the pile point
    // band `[25, width−25)` is LIVE on the 512-wide world (unlike C1/C2's
    // collapsed bands) — the seven ladders place 185/186/187 piles on its
    // cave floors and surface. Re-pin DEFERRED to the merge gate (no
    // compiler on this branch); a == b stays green (own fresh stream).
    // W4-g38 C4 (2026-11-21, code-only): the `Spawn Point` pass
    // (`WorldGen.cs:19460-19588`) lands after `piles` on a fresh
    // `pass_rng("Spawn Point")` stream, replacing the compact spawn the
    // world tail computed. This pin is EXPECTED UNCHANGED: on a classic
    // world the pass writes ONLY `Main.spawnTileX/Y` (the port's
    // `world.spawn_x/spawn_y`) — no tiles — and the world hash covers tile
    // fields only. The sole worldgen spawn consumer (`micro_finish`'s
    // actually-no-traps spike rings) is gated behind ≥4 secret seeds, so a
    // classic world's tiles are untouched. The pass draws only its own
    // fresh stream. a == b stays green. (Secret-seed worlds that DO consume
    // the spawn — noSurface force-clear, the ≥4-secret spike rings — move
    // by design; no pin covers them.)
    // W4-g38 C5 (2026-11-21, code-only): the `Guide` pass
    // (`SpawnStarterNPCs`, `WorldGen.cs:19685-19897`) lands after
    // `spawn point`. This pin is EXPECTED UNCHANGED: the pass writes only
    // the starter-NPC roster (`world.starter_npcs`, not hashed) + the owned
    // rescue flags — no tiles, no RNG draws. a == b stays green.
    // W4-g38 C6 (2026-11-21, code-only): the `Sunflowers` pass
    // (`SunflowersPart2`, `WorldGen.cs:19898-19939` + `PlaceSunflower`
    // `:53636-53686`) lands after `guide` on a fresh
    // `pass_rng("Sunflowers")` stream. This pin MOVES BY DESIGN: the span
    // band is live on the 512-wide world (the centre roll spans the full
    // width) — the 2×4 tile-27 sunflowers plant on its grass seats.
    // Re-pin DEFERRED to the merge gate; a == b stays green.
    // W4-g38 C7 (2026-11-21, code-only): the `Herbs` pass (`AlchemyHerbs`,
    // `WorldGen.cs:19964-19980` + `PlantAlch` `:45901-45938` +
    // `PlaceSuitableHerbHere` `:45940-45970` + `PlaceAlch` `:45380-45479`)
    // lands after `sunflowers` on a fresh `pass_rng("Herbs")` stream. This
    // pin MOVES BY DESIGN: the 1.7w ladder is live on the 512-wide world
    // (the x band `[20, 492)` spans the width) — the stage-0 tile-82 herbs
    // plant on their soils. Re-pin DEFERRED to the merge gate; a == b
    // stays green.
    // W4-g38 C8 (2026-11-21, code-only): the `Dye Plants` pass
    // (`DyePlants`, `WorldGen.cs:19981-19994` + `plantDye` `:71286-71455` +
    // `PlaceDye` `:53988-54064` + `MatureTheHerbPlants` `:45748-45775` +
    // `GrowGlowTulips` `:45622-45650` + `PlaceChilletEggs` `:45652-45678`)
    // lands after `herbs` on a fresh `pass_rng("Dye Plants")` stream. This
    // pin MOVES BY DESIGN: on the 512×512 world the w plantDye attempts
    // roll the live [100, 412) × [100, 312) band (the 95-margin entry
    // admits j ≤ 417), the maturity sweep rolls over every cell, and the
    // glow-tulip/chillet budget loops run their y bands ([worldSurface,
    // 312) / [rockLayer, 312)) — the tile-227 dye, the 82-84 stage rolls
    // and the 656/752 objects move the hash. Re-pin DEFERRED to the merge
    // gate; a == b stays green.
    // W4-c33 pass 2 (code-only): the faithful `Pyramids` rewrite moves this
    // pin too — the pass now consumes the published pyr anchors on its own
    // `pass_rng("Pyramids")` stream (was `srng`: the later srng consumers —
    // pots, beehives, cave houses — shift), and the 151 hull replaces the
    // shape-level 480 wherever an anchor passes the sand-column gate.
    // Stale-by-design; the gate re-pins it (a == b stays).
    // W4-c33 pass 3 (code-only; MERGE KEEP-ONE matrix-88 §2.9: the pass
    // body is d103's water_chests.rs — the c33 pass_underwater_chests copy
    // dropped, the d103 slot after `surface chests` stands): its own stream
    // draws nothing from the
    // shared ledgers, but the placed style-17 chests retype water cells
    // (and the trySlope flatten touches the floor slopes), so this pin
    // moves by design wherever the scatter finds water. Stale-by-design;
    // the gate re-pins it (a == b stays).
    // W4-c33 pass 4 (code-only): the Water Plants j-loop (the lily/cattail/
    // bamboo arms) joins ahead of the seaweed walk on the SAME
    // `pass_rng("Water Plants")` stream — the deferred draws now interleave
    // with the seaweed rolls (the documented d103 deferral closes), and the
    // 518/519/571 plants retype surface water cells. Stale-by-design; the
    // gate re-pins it (a == b stays).
    // W4-d101 pass 1 (2026-11-20, code-only): the `Sand Patches` pass
    // (`WorldGen.cs:11689-11717`, `passes1.rs::pass_sand_patches`) lands on
    // its own `pass_rng("Sand Patches")` stream — on a 512-wide world
    // `(512 * 0.013) as i32 = 6` sand-53 blobs scatter through the
    // surface-to-rock band, so this pin moves BY DESIGN. The re-pin is
    // DEFERRED to the merge gate (no compiler on this branch); the a == b
    // determinism assert stays in place.
    // W4-d101 pass 2 (code-only): `Mount Caves` (`WorldGen.cs:11784-11857`,
    // `passes1.rs::pass_mount_caves`) raises `(512 * 0.001) as i32 = 0`
    // mountains on a 512-wide world — the count truncates to zero, so this
    // pass draws NOTHING and the pin does not move from it (the pass-1
    // re-pin above still stands).
    // W4-d101 pass 3 (code-only): `Dirt Wall Backgrounds`
    // (`WorldGen.cs:11858-11895`) re-paints the surface band's walls (the
    // snow-40 arm over the eastern-third snow skin, the per-column stop
    // walk) — this pin moves BY DESIGN; deferred to the gate.
    // W4-d101 pass 4 (code-only): `Clay` (`WorldGen.cs:11959-12007`) —
    // `512*512*2E-05 = 5` + `*5E-05 = 13` + 5 clay blobs plus the tidy
    // sweep move this pin BY DESIGN; deferred to the gate.
    // W4-d101 pass 5 (code-only): `Small Holes` (`:12008-12066`) — 393
    // paired diggers on the 512² world; deferred to the gate.
    // W4-d101 pass 6 (code-only): `Dirt Layer Caves` (`:12067-12106`) — 7
    // long worms on the 512² world; deferred to the gate.
    // W4-d101 pass 7 (code-only): `Rock Layer Caves` (`:12107-12161`) — 34
    // deep worms on the 512² world; deferred to the gate.
    // W4-d101 pass 8 (code-only): `Generate Ice Biome` (`:12312-12422`) —
    // the snow walk retypes the eastern band (the Reset origin draws land
    // in [0.6, 0.75]·width) and its fringe drip reaches below the lava
    // line; deferred to the gate.
    // W4-d101 pass 9 (code-only): the marble/granite step split is
    // RNG-NEUTRAL — same call order, same per-pass streams, no new tiles —
    // so this pin does not move from it (only the progress weight sum
    // gains the Granite slot's own weight, which draws nothing).
    // W4-d102 pass 1 (2026-11-20, code-only): the Silt pass
    // (`WorldGen.cs:13143-13181`, `worldgen/passes2.rs::pass_silt`) lands
    // between `dirt to mud` and `ash blobs` on its own `pass_rng("Silt")`
    // stream — the two kind-123 TileRunner loops run on EVERY world
    // (0.0001·area big blobs + 0.0005·area small ones over
    // rockLayerHigh..maxTilesY), so this pin moves BY DESIGN. The re-pin is
    // DEFERRED to the merge gate (no compiler on this branch); the a == b
    // determinism assert stays in place.
    // W4-d102 pass 2 (code-only): the Webs pass
    // (`WorldGen.cs:13607-13644`, `passes2.rs::pass_webs`) lands between
    // `ores and gems` and `evil` — the 0.0006·area cobweb runners grow on
    // every world with cave pockets (the port's carved caves qualify), so
    // this pin moves BY DESIGN too; deferred to the merge gate.
    // W4-d102 pass 3 (code-only): the Slush sweep
    // (`WorldGen.cs:14697-14752`, `passes2.rs::pass_slush`) lands between
    // `liquids` and `dungeon` — a pure retype over the GenVars snow rect,
    // which is EMPTY (snowTop == snowBottom == 0) until d101's Generate
    // Ice Biome producer lands, so this pin is UNCHANGED by this pass
    // alone (the silt/ice kinds it would retype do not occur in the empty
    // rect; the sweep draws nothing).
    // W4-d102 pass 4 (code-only): the Mountain Caves openings
    // (`WorldGen.cs:14873-14887` + CaveOpenater `:80085-80166` + Cavinator
    // `:79984-80083`, `passes2.rs::pass_mountain_caves`) land between
    // `constant lava layer` and `beaches` — a no-op while
    // `GenVars.numMCaves == 0` (d101's Mount Caves producer not yet on
    // this branch), so this pin is UNCHANGED by this pass alone; the
    // merged tree re-pins once the anchors exist.
    // W4-d102 pass 5 (code-only): the Gems pass
    // (`WorldGen.cs:15050-15137`, `passes2.rs::pass_gems`) lands between
    // `gravitating sand` and `ocean caves` — the six gem-kind runner bands
    // vein stone on every world and the sand-edge slide shuffles the
    // shore columns, so this pin moves BY DESIGN; deferred to the gate.
    // W4-d102 pass 6 (code-only): the Shimmer pass
    // (`WorldGen.cs:15195-15245` + ShimmerMakeBiome `:34853-35057`,
    // `worldgen/shimmer.rs`) lands after `ocean caves` — UNCHANGED by this
    // pass alone on the 512-wide world: the sub-vanilla valve (the x band
    // `Next(200, 0.2·512)` collapses) skips the pass BEFORE any draw, so
    // neither the tiles nor the stream move. The full-size worlds gain the
    // shimmer pool (DEFAULT_WORLD below); the merged tree re-pins.
    // W4-d102 pass 7 (code-only): the Clean Up Dirt pass
    // (`WorldGen.cs:15246-15359`, `passes2.rs::pass_clean_up_dirt`) lands
    // after `shimmer` — the two wall-scrub sweeps clear the dirt-wall
    // band the terrain/Dirt-Wall-Backgrounds model leaves in the surface
    // rows (the ±1 unconditional neighbour arms reach the shallower
    // columns beside every first-active row), so this pin moves BY
    // DESIGN; deferred to the gate. The noSurface arm replaces the scrub
    // with the top-100/101 fill (a classic world reads no flag).
    // W4-d102 pass 8 (code-only): the Dirt Rock Wall Runner pass
    // (`WorldGen.cs:15461-15475` + DirtyRockRunner `:77145-77210`,
    // `passes2.rs::pass_dirt_rock_wall_runner`) lands after `pyramids` —
    // one candidate per column retypes wall-2 blobs to 59 in the surface
    // band on every world ≥20 wide (the 512 world qualifies), so this pin
    // moves BY DESIGN; deferred to the gate.
    // W4-d102 pass 9 (code-only): the Wood Tree Walls sweep
    // (`WorldGen.cs:15717-15749`, `passes2.rs::pass_wood_tree_walls`)
    // lands after `living trees` — draws nothing; wall 244 fills the
    // living-wood 191 interiors wherever the Living Trees step planted
    // them (the 512 world's count roll can rescue one tree). If a tree
    // exists this pin moves BY DESIGN; deferred to the gate.
    // W4-d104 pass 28 (2026-11-20, code-only): the `Floating Island
    // Houses` pass (`WorldGen.cs:17859-17874` + `IslandHouse`
    // `:79203-79484`) lands — the compact islands pass's inline plank box
    // retires for the faithful sunplate house at the vanilla slot, on the
    // fresh `pass_rng("Floating Island Houses")` stream. Any world whose
    // islands placed moves BY DESIGN (the house tiles/walls/frames swap);
    // a 512-wide world's island band collapses (the sky-band guard), so
    // this pin is UNCHANGED by the pass itself — the sibling lanes' pins
    // still re-measure at the gate. The a == b determinism halves stay
    // untouched.
    // W4-d104 pass 29 (2026-11-21, code-only): the `Quick Cleanup` pass
    // (`WorldGen.cs:17875-17993`) lands after Floating Island Houses — the
    // full-world tidy sweep (the ocean lava→water swap, the beach sand
    // drip on the fresh `pass_rng("Quick Cleanup")` stream, the
    // sandstone-wall retypes/liquid arms, the surface-sand wall
    // inheritance, the slope/half-brick clear family under the
    // 137/130-non-solid window). On a 512-wide world the beach band
    // (`x < 380 || x > width - 380`) covers the whole sweep width, so the
    // drip's draws and the slope clears move this pin BY DESIGN too —
    // stale, the gate re-pins. The a == b halves stay untouched.
    // W4-d104 pass 30 (2026-11-21, code-only): the `Hellforge` pass
    // (`WorldGen.cs:18187-18234`) lands — `maxTilesX / 200` forges retry
    // onto the hell-house wall columns on the fresh `pass_rng("Hellforge")`
    // stream. On a 512-wide world the count is 2 and the compact hell
    // forts' wall-14 columns can host a forge, so this pin moves BY
    // DESIGN too — stale, the gate re-pins. The a == b halves stay
    // untouched.
    // W4-d104 pass 30 graft (2026-11-21, code-only, research/86 §6.2):
    // the bare `place_3x2` write becomes the full `PlaceTile(77)` route
    // (the g37 B7 body grafted into this lane's correctly-ordered pass —
    // the entry gate, the inactive-seat Clear arm, the
    // ResetsHalfBrickPlacementAttempt reset, the two SquareTileFrame
    // calls). The retry DRAW SCHEDULE is unchanged (the route draws no
    // RNG); the tile bytes can move where a seat carried residue or a
    // refused placement resets halfBrick/frames. This pin was ALREADY
    // stale-by-design from the pass landing — the graft folds into the
    // same deferred re-pin. The a == b halves stay untouched.
    // W4-d104 pass 31 (2026-11-21, code-only): the `Spreading Grass`
    // classic arm (`WorldGen.cs:18237-18351`) lands — the surface biome
    // retypes + the first-surface grass spread move the surface cells of
    // every world (the 512 included). No RNG draw (the classic arm is
    // draw-free), so only the tiles move. Stale-by-design; the gate
    // re-pins. The a == b halves stay untouched.
    // W4-d104 pass 31 fix (2026-11-21, code-only, research/86 §6.1): the
    // sweep-2 call flips `repeat:false` → `repeat:true` — the LIVE
    // `:18334` call takes the C# default (`:75146`); the ported
    // `repeat:false` was the dead `:18339` catch arm. The 3×3 recursion
    // now fires, so MORE surface cells grass (the whole first-surface
    // dirt run above worldSurface, not one cell per column). Still NO
    // RNG draw — no pinned grass-spread draws exist to re-derive; this
    // pin was ALREADY stale-by-design and folds into the same deferred
    // re-pin. The a == b halves stay untouched.
    // W4-d104 pass 32 (2026-11-21, code-only): the `Surface Ore and
    // Stone` pass (`WorldGen.cs:18447-18511` + `OrePatch` `:10579-10705`
    // + `StonePatch` `:10242-10324`) lands — the ore/stone patch scatters
    // on the fresh `pass_rng("Surface Ore and Stone")` stream. The
    // 512-wide pin is UNCHANGED by this pass: the sub-vanilla width guard
    // skips before any draw (the stone count `Next(1, width*7/4200)`
    // throws there; the C# never generates a ≤600-wide world). The
    // a == b halves stay untouched.
    // W4-d104 pass 33 (2026-11-21, code-only): the `Place Fallen Log`
    // pass (`WorldGen.cs:18513-18655`) lands — the 488 logs on the fresh
    // `pass_rng("Place Fallen Log")` stream. The 512-wide pin is
    // UNCHANGED: the ≤800 width guard skips before any draw (the C#
    // column draw `Next(400, width-400)` throws there). The a == b
    // halves stay untouched.
    // W4-d104 pass 34 (2026-11-21, code-only): the `Traps` pass
    // (`WorldGen.cs:18657-18786` + `placeTrap` `:8922-9513` +
    // `PlaceSandTrap` `:35463-35735`) lands — the dart/boulder/explosive/
    // geyser traps + the desert sand traps on the fresh
    // `pass_rng("Traps")` stream. The 512-wide pin is UNCHANGED: the
    // ≤400 width guard skips before any draw (the C# column draw
    // `Next(200, width-200)` throws there). The a == b halves stay
    // untouched.
    // W4-d104 pass 35 (2026-11-21, code-only): the `Piles` pass
    // (`WorldGen.cs:18788-19458`) lands — the seven decor scatters on the
    // fresh `pass_rng("Piles")` stream. The 512-wide pin MOVES by design
    // (the column band `Next(25, 487)` is live there) — stale, the gate
    // re-pins. The a == b halves stay untouched.
    // W4-d104 pass 36 (2026-11-21, code-only): the `Spawn Point` pass
    // (`WorldGen.cs:19460-19587`) lands — the spawn search replaces the
    // compact tail anchor. The spawn fields are NOT hashed (the tile
    // fold only), and the search draws on its own stream, so this pin is
    // UNCHANGED by the pass. The a == b halves stay untouched.
    // W4-d108 (M5-SEED temple branches, code-only): the drunk/FTW ×3 +
    // remix ×2 room-count chain (`WorldGen.cs:33439-33450`), the drunk
    // spike budget ×1.5 + second rows (`:34065-34207`) and the remix
    // placement-search/fallback arms (`:15853-15922`) are ALL flag-gated —
    // the classic pinned worlds draw and write NOTHING new, so this pin
    // is UNCHANGED by the branch (the `a == b` determinism asserts in both
    // pin tests still guard the streams; the seeded worlds' hashes are
    // unpinned until the gate generates them). `PaintTheTemple` itself
    // already landed with M5-SEED-1 (`finish_seeds.rs`) and only fires
    // under the anniversary flag — likewise inert here.
    const SEED42_512: u32 = 124297949;
    // Re-pinned by WORLDGEN-CAVES (task/worldgen-caves): the `Surface Caves`
    // pass (`WorldGen.cs:12162-12270`, groups 1-4) now digs its
    // surface-opening tunnels before `Shinies`, so the default world's
    // tiles move downstream of the carve. Measured on this branch. (The
    // 512 world's pin survives: its single small runner lands in the beach
    // zone the `Beaches` pass re-shapes anyway, and the final bytes match.)
    // Re-pinned by S1-K-2 wave A: the mahogany trees (above) — the new
    // 383/384 tiles and wall-78 interiors move the hash directly, and the
    // downstream tile-dependent passes (the vines on the mud, the slopes,
    // the tree/flora placement) legitimately re-roll around them.
    // Measured on this branch (the synced a23d87e5 base).
    // S1-K-2 wave B (2026-11-19, code-only): `pass_mining_explosives`
    // (`MiningExplosivesBiome.cs:10-83` + `WorldGen.cs:21061-21101`) lands
    // on a fresh `pass_rng("Micro Biomes")` copy — on a DEFAULT-sized world
    // the wired ore traps (the tier-ore veins, the 141/411 pair, the red
    // wires) move this pin BY DESIGN. The re-pin is DEFERRED: no compiler
    // may run on this branch, so the new value cannot be measured; the
    // gate re-pins it. The `fix/vines-destroy-walls` branch (b95bfc60)
    // also moves the worldgen pins (stale-by-design) — both branches re-pin
    // at the gate, sequentially. `SEED42_512` below is UNCHANGED: the
    // explosives block's `Next(380, maxTilesX - 380)` beach band collapses
    // on a 512-wide world (the C# throws; the port's sub-vanilla width
    // guard skips the block), and the 141/411 boot-static bits only matter
    // when those kinds exist.
    // Re-pinned at the merge gate for fix/vines-destroy-walls (see the
    // SEED42_512 note above): vine cells keep their wall fields, so the
    // default world's hash moves too. Old -> new: 143177146 -> 311316399
    // (measured on the merged tree, a == b determinism stayed green).
    // The S1-K-2 wave B re-pin ran sequentially AFTER the vines re-pin at
    // the merge gate: measured on the merged tree (vines + mining
    // explosives + thin ice), a == b determinism stayed green on the
    // failing run and SEED42_512 stayed UNCHANGED (the width-guard proof
    // held). Old -> new: 311316399 -> 3647283075 (gate log
    // TR-LD-gate-k2b.log).
    // S1-P-i33 (2026-11-20, code-only): the wall-framing mask fix
    // (`Framing.WallFrame`'s neighbour test, `Framing.cs:345-376` — an active
    // `TruncatesWalls` tile 54/328/459/748 now joins the mask, and an
    // echo-coated invisible wall drops out) changes the generated
    // `wall_frame` cells wherever glass & co. sit beside a wall. BOTH pins
    // move BY DESIGN; the re-pin is DEFERRED to the merge gate (no compiler
    // on this branch). The `a == b` determinism assertion stays green — the
    // mask is a pure function of the tile/wall state, no RNG draw added.
    // W4-f25 (code-only): the `Settle Liquids` pass (see SEED42_512 above)
    // settles the default world's lakes/ocean water and retypes the
    // below-water-line falls to lava, so this pin moves BY DESIGN too —
    // deferred to the gate with the SEED42_512 note. No RNG draw added.
    // W4-f25 helper round (2026-11-21, code-only): the missing-WorldGen-helper
    // ports (`spread::tile_is_exposed_to_air` etc., the `12g-f25-worldgen-
    // helpers.md` ledger C1-C8) are UNWIRED — no executed step calls them,
    // so BOTH pins are untouched by that round (the g39 H1-H15 precedent).
    // The CLOSEOUT's ConvertSkyIslands is gen-time but anniversary-only (see
    // the DEFAULT_WORLD note) — this pin is untouched by it too.
    // W4-f25 WIRING phase (code-only): the C3/C4 spray cells land at the
    // Clentaminator/holy-water/SpreadGrass consumers, the C7 tree conversion
    // at `try_kill_invalid_tree_above`, the C8 moss colour pick at
    // `edit/kill.rs::check_long_moss` — all RUNTIME seams (no executed
    // worldgen step calls them), so both pins stay unchanged. The 184 boot
    // static is a `frame_important` flag, not a hashed tile field. C5 stays
    // unwired (d103 P8 owns the hardUpdateWorld chlorophyte arms).
    // W4-f25 CLOSEOUT (2026-11-21, code-only): ConvertSkyIslands lands at the
    // FinishTenthAnniversaryWorld call (`finish_seeds.rs::convert_sky_islands`)
    // — a GEN-TIME seam, but the anniversary pass only runs on the
    // tenthAnniversary seed; neither pinned world (SEED42_512 / DEFAULT_WORLD)
    // sets it, so both pins stay unchanged. The solution-spray rows
    // (146-149/1015-1017) and the Convert 15/16/18/19 dungeon-biome cells are
    // runtime / unwired respectively — no executed classic step calls them.
    // W4-f25 g46 LIQUID-UNION (2026-11-21, code-only): the SettleWaterAt
    // dungeon re-type now reads the ORIGIN cell's wall (Liquid.cs:152/:188 —
    // the pre-fix copy read the landing cell's, a real deviation research/70
    // step 6 flagged). This moves the gen output wherever an origin/landing
    // wall pair differs inside the potential bounds — BOTH pins are already
    // stale-by-design from the Settle Liquids landing; the RE-DERIVE at the
    // g46-union gate covers this fix too (the union's real DelWater drain +
    // swap-remove buffer move them further). The gen-side edit_kills drain
    // choice is DISCARD (no worldgen loot side channel, the C# gen-time
    // noItem gate :63546-63549) — recorded in 12g per 70's witness table.
    // The f25e/g42e seal rounds are comment/witness-only — no pin effect.
    // W4-g42 (2026-11-21, code-only): the glowing-mushroom biome lane moves
    // this pin BY DESIGN (see SEED42_512 above) — the default 4200-wide
    // world runs 6 clusters, the verbatim ShroomPatch + the plants-pass
    // giant trees change every mushroom patch's tiles, and the
    // `DoSurfaceIsMushrooms` seed arm (flag-gated, classic draws nothing)
    // leaves the classic world's stream untouched. The re-pin is DEFERRED
    // to the merge gate; the `a == b` determinism half stays untouched.
    // W4-c31 (2026-11-21, code-only): the faithful hive rewrite (see the
    // SEED42_512 note above) moves this pin too — the tunnel-carved hives,
    // the honey-fall cubes/dents, the larva stands and the 231 Place3x3
    // frames replace the dome's tiles, and the srng schedule shift
    // re-rolls every downstream pass. Stale-by-design; the gate re-pins
    // it sequentially after the S1-P-i33 mask re-pin. `a == b` stays green.
    // W4-c31 jungle residuals (2026-11-21, code-only): the faithful
    // `JungleShrines` rewrite (see the SEED42_512 note above) moves this pin
    // too — on a DEFAULT-sized world the shrine y band is live, so the
    // faithful huts (the `jungleHut` material shells, the carved interiors,
    // the torches, the mud pillars, the roof rows) replace the shape-level
    // boxes, and the `srng` schedule shift re-rolls every downstream pass.
    // Stale-by-design; the gate re-pins. `a == b` stays green.
    // W4-g44 (2026-11-21, code-only): the verbatim island passes move this
    // pin BY DESIGN on every default-sized world — the discs relocate (the
    // count 3+1 vs the old 4, the column-scan y-band + the surfaceLow
    // clamp), the lakes re-cadence to the tail, and the houses retype
    // planks-9 → sunplate-202 with the disc-wall-82 interiors, the door,
    // the windows and the furniture. The re-pin is DEFERRED to the merge
    // gate (no compiler on this branch); `a == b` stays green (the
    // `pass_rng("Floating Islands")` / `("Floating Island Houses")` slots
    // are seed-keyed, no new entropy).
    // W4-g43 (2026-11-21, code-only): the temple residuals round (see the
    // SEED42_512 note above) moves this pin too — the DEFAULT-sized world
    // carries a full temple, so the painting frames, the SquareTileFrame
    // tails and the trap draw-order shift all land here. Re-pin DEFERRED to
    // the merge gate; a == b stays green (the changes are deterministic).
    // W4-g41 (2026-11-21, code-only): the spider-cave egg-sac validator
    // (`spider_gem_moss.rs::check_stalactite`, `WorldGen.cs:38996-39175`)
    // pops unsupported sacs and restyles mismatched ones (the chain's
    // `Next(3)*18` draw, `:38665`), the missing stalactite arms place new
    // tiles, the pile boulder-seat veto removes piles, and the
    // `frame_important[165]` boot static keeps the stored frames through the
    // framer — so classic worlds change BY DESIGN. BOTH pins move; the
    // re-pin is DEFERRED to the merge gate (no compiler on this branch, the
    // S1-K-2 wave B precedent). The `a == b` determinism witnesses are
    // written NOT run. Census: `docs/research/93-spider-cave-census.md`.
    // W4-g41 follow-up (2026-11-21, code-only): the NOT-THE-BEES host
    // (`not_the_bees.rs`, `WorldGen.cs:25107-25360` + `:25388-25797`) lands at
    // `step_seed_finish`'s C# slot (`:22435-22439`) and WIRES the egg-sac
    // task's `spider_wall_remover` at the two FTW+NTB search loops
    // (`:25409-25434`). BOTH functions are flag-gated on NOT_THE_BEES (and
    // skyblock), so a classic world never enters them and draws nothing —
    // the two pins are UNCHANGED by this task (the egg-sac re-pin deferral
    // above stands as-is). The host's draws ride fresh `pass_rng("NotTheBees")`
    // / `pass_rng("FinishNotTheBees")` streams; the `a == b` determinism
    // witnesses (incl. a full-pipeline NTB+FTW world) are written NOT run.
    // Census: `docs/research/94-not-the-bees-host-census.md`.
    // W4-g41 finish gaps (2026-11-21, code-only): the FinishNoTraps +
    // FinishRemixWorld hosts (`finish_no_traps.rs` / `finish_remix.rs`,
    // `WorldGen.cs:25986-26224` / `:26350-26770`) are flag-gated on
    // NO_TRAPS / REMIX - a classic DEFAULT world never enters them and
    // draws nothing, so this pin is UNCHANGED by the two hosts (the
    // egg-sac re-pin deferral above stands as-is). d107 co-lane claim
    // check: the remix host was explicitly deferred by d107's pushed
    // ANALYSIS, the no-traps claims are micro-seed side only - both
    // hosts unclaimed/taken. The a == b witnesses (full-pipeline
    // NO_TRAPS and REMIX worlds) are written NOT run. Census:
    // `docs/research/95-finish-no-traps-remix-census.md`.
    // W4-g40 (2026-11-21, code-only): the marble/granite residual port
    // (`biomes.rs::pass_marble/pass_granite` — the seven SlabStates +
    // SmoothSlope masks, ResetToType, the corrected Sets.Ore literal, the
    // PlaceTight Next(4) arms and the Tile.SmoothSlope cascade on the Marble
    // stream; ShouldUseLava on GenVars.lavaLine, the resistance<0.01 clear
    // arm, the SolidTile cleanup census and the FastRandom PlaceDecorations
    // on the Granite side) moves DEFAULT_WORLD BY DESIGN — every marble and
    // granite chamber changes shape/walls/decor, and the new PlaceTight
    // draws shift the Marble pass's own stream tail (isolated — no other
    // pass's ledger moves). The re-pin is DEFERRED to the merge gate (no
    // compiler on this branch); `a == b` stays green (the witnesses in
    // `biomes.rs` pin the per-function ledgers and the end-to-end replay).
    // SEED42_512: the marble/granite Count draws truncate to 0 on a 512-wide
    // world (WorldArea 512*512/5040000 → (4*x) as int = 0; WorldWidth
    // 512/4200 → 0), so both passes place nothing there — unchanged by the
    // residual itself. The P0 stalactite-table additions (the hive RISING
    // arm, `WorldGen.cs:38543-38552`, previously missing) can move the pin
    // if a spider-cave hive seat exists on the 512 world; unmeasurable
    // code-only, so the gate re-pins or confirms it.
    // W4-g40 entrance residual (2026-11-21, code-only, research 222 —
    // renumbered from 94 per the d54 research/150 registry): the
    // four underground-desert entrances now run the faithful Blotches
    // draws + SetTile/ClearTile/Smooth chains (`desert_entrances.rs`), and
    // the entrance gate reads the JSON 0.5 with the anniversary/extraLiquid
    // halves (`DesertBiome.cs:23`). BOTH pins may move BY DESIGN: the 512
    // world's desert band is 36 tiles wide and its entrance fraction can
    // fire (the Blotches ledger then shifts the Full Desert stream tail —
    // isolated, no other pass's stream moves); the default world's
    // entrances change shape/walls wherever the roll fires. The re-pin is
    // DEFERRED to the merge gate (no compiler on this branch); `a == b`
    // stays green (the `desert_entrances.rs` witnesses pin the Blotches
    // ledger replay + the per-kind end-to-end replay, `desert.rs` pins the
    // anniversary stream separation).
    // W4-g40 hive-smooth slice (2026-11-21, code-only, research 248 - renumbered 246-248 at merge): the
    // DesertHive PostPlacementEffect.Smooth map + the post-carve
    // Tile.SmoothSlope(applyToNeighbors:false) walk (DesertHive.cs:210-221)
    // land on the biomes::tile_smooth_slope_one seam. RNG-FREE (no Rng or
    // FastRandom draw added or moved — the carve's ledgers are untouched),
    // but the hive's shell edges gain slope/half-brick states, so
    // DEFAULT_WORLD moves BY DESIGN wherever the hive fires; SEED42_512
    // moves iff its desert band seats a hive. Re-pin DEFERRED to the merge
    // gate; a==b kept (the three desert.rs hive-smooth witnesses pin the
    // map's arm-exactness, the slope delta and the RNG-neutral replay).
    // W4-d103 pass 19 (code-only): the Oasis basins (the water carve, the
    // sand rim, the bridge speckle) plus the awakened cacti-pass oasis
    // block (palms/sea oats/oasis plants over each basin) move this pin BY
    // DESIGN on the 4200-wide default world. The re-pin is DEFERRED to the
    // merge gate (no compiler on this branch); a == b stays green — the
    // pass draws only its own fresh stream.
    // W4-d103 pass 20 (code-only): the Waterfalls half-brick sweeps (the
    // liquid-edge pounds + the half-brick shore arms, the `tileSolid[191]`
    // window over the living wood) move this pin BY DESIGN on the
    // 4200-wide default world — the lake/ocean/cave liquid edges qualify.
    // The re-pin is DEFERRED to the merge gate (no compiler on this
    // branch); a == b stays green — the pass draws only its own fresh
    // stream, and the half-brick flag rides the existing tile hash mix.
    // W4-d103 pass 21 (code-only): the Wall Variety pocket retexture (the
    // 196-215 unsafe-wall families over the drained >50-cell cave pockets,
    // `(int)(300 * 4200*1200/5040000)` = 300 placements on the default
    // world) moves this pin BY DESIGN — the cave fields qualify. The
    // re-pin is DEFERRED to the merge gate (no compiler on this branch);
    // a == b stays green — the pass draws only its own fresh stream, and
    // the wall field rides the hash mix.
    // W4-d103 pass 22 (code-only): the Life Crystals pass (100 heart
    // crystals on the 4200x1200 default, the 2×2 tile-12 writes with their
    // 18 px frames + the flattened supports) moves this pin BY DESIGN.
    // The re-pin is DEFERRED to the merge gate (no compiler on this
    // branch); a == b stays green — the pass draws only its own fresh
    // stream.
    // W4-d103 pass 23 (code-only): the Statues pass (146 statues on the
    // 4200-wide default, the 3×2 tile-105/349 writes + the four wired
    // pressure-plate traps) moves this pin BY DESIGN. The re-pin is
    // DEFERRED to the merge gate (no compiler on this branch); a == b
    // stays green — the pass draws only its own fresh stream (the wires
    // are a side table, not hashed).
    // W4-d103 pass 24 (code-only): the Surface Chests pass (21 chests on
    // the 4200-wide default, the AddBuriedChest core's 2×2 tile-21 writes
    // + the living-tree style-12 arm) moves this pin BY DESIGN. The re-pin
    // is DEFERRED to the merge gate (no compiler on this branch); a == b
    // stays green — the pass draws only its own fresh stream.
    // W4-d103 pass 25 (code-only): the Water Chests pass (the ocean-cave
    // treasure anchors + 18 scatter chests on the 4200-wide default, the
    // style-17 water chests through the AddBuriedChest core) moves this
    // pin BY DESIGN. The re-pin is DEFERRED to the merge gate (no compiler
    // on this branch); a == b stays green — the pass draws only its own
    // fresh stream.
    // W4-d103 pass 26 (code-only): the Cave Walls pass (168 pocket
    // retextures + 84 jungle-wall conversions on the 4200-wide default,
    // the Spread.Wall/Wall2 floods) moves this pin BY DESIGN. The re-pin
    // is DEFERRED to the merge gate (no compiler on this branch); a == b
    // stays green — the pass draws only its own fresh stream.
    // W4-d103 pass 27 (code-only): the Floating Island Houses pass (the
    // faithful sunplate-202 houses over the island ledger — the door, the
    // skyware chest, the fountain, the table/chairs, the banners) replaces
    // the islands pass's inline shape-level plank box, moving this pin BY
    // DESIGN on the 4200-wide default. The re-pin is DEFERRED to the merge
    // gate (no compiler on this branch); a == b stays green — the pass
    // draws only its own fresh stream.
    // W4-d103 tail P2/P3/P4 (2026-11-21, code-only): the hardmode transform
    // (`initializeHardMode` head + remix band + the verbatim GERunner) is a
    // POST-pipeline runtime function — no executed step calls it on a
    // classic world, so this pin is UNCHANGED by the rewrite. The
    // `start_in_hardmode` micro-seed world changes by design (it has no
    // pin). `World.dungeon_x/y` became owned state (the dungeon step
    // publishes the anchor); neither field is hashed. a == b stays.
    // W4-d103 tail P8/P11/P13 (2026-11-21, code-only): hardUpdateWorld +
    // its gates are RUNTIME-only (the fixed-step growth tick) — no
    // executed pipeline step calls them, classic or seeded. This pin is
    // UNCHANGED. a == b stays.
    // W4-d103 tail P9 (2026-11-21, code-only): the underground moss arm is
    // RUNTIME-only (the growth tick). This pin is UNCHANGED. a == b stays.
    // W4-d106 R1 (code-only): the Grass Wall pass (see SEED42_512 above) —
    // expected UNCHANGED for the same wall-0-grass-top reason on the
    // 4200-wide default world; any grass-with-wall seed cell would move it
    // BY DESIGN. Re-pin DEFERRED to the merge gate; a == b stays green
    // (the pass draws only its own fresh stream).
    // W4-d106 R2 (code-only): `Settle Liquids Again` (see SEED42_512 above)
    // — the settle engine re-types/drops/merges liquid cells across the
    // lake, ocean shelves and lava puddles, so this pin moves BY DESIGN.
    // No RNG draw is added (the engine is deterministic over tile state),
    // so a == b stays green. Re-pin DEFERRED to the merge gate.
    // W4-d106 R3 (code-only): `Tile Cleanup` (see SEED42_512 above) — the
    // slope clear / drip placement / object normalisation move this pin BY
    // DESIGN on the fresh `pass_rng("Tile Cleanup")` stream; a == b stays
    // green. Re-pin DEFERRED to the merge gate.
    // W4-d106 R4 (code-only): `Stalac` (see SEED42_512 above) — the gem
    // trees + speleothems move this pin BY DESIGN on the fresh
    // `pass_unified("Stalac")` stream; a == b stays green. Re-pin DEFERRED
    // to the merge gate.
    // W4-d106 R5 (code-only): `Remove Broken Traps` (see SEED42_512 above)
    // — the broken-circuit strip moves this pin BY DESIGN (no RNG draw);
    // a == b stays green. Re-pin DEFERRED to the merge gate.
    // W4-d106 R6 (code-only): `Final Cleanup` (see SEED42_512 above) — the
    // wall-hole fill, the sand fall columns, the boulder/liquid/painting
    // sweeps, the poo scatter and the final LiquidCheck merge sweep move
    // this pin BY DESIGN on the fresh `pass_rng("Final Cleanup")` stream;
    // a == b stays green. Re-pin DEFERRED to the merge gate.
    // W4-g36 A1 (code-only): the Shimmer pass (see SEED42_512 above) carves
    // the aether biome on every DEFAULT-sized world — the stone shell, the
    // shimmer pool, the gem trees and the two openings move this pin BY
    // DESIGN; the re-pin is deferred to the gate. The pass draws only its
    // own `pass_rng("Shimmer")` stream, so no other step's ledger shifts.
    // W4-g36 A2 (code-only): the Clean Up Dirt pass (see SEED42_512 above)
    // strips the exposed surface dirt walls on every world — this pin
    // moves BY DESIGN too; deferred. Own `pass_rng("Clean Up Dirt")`
    // stream; the `a == b` halves stay untouched.
    // W4-g36 A3 (code-only): the Dirt Rock Wall Runner pass (see
    // SEED42_512 above) — the wall 2 → 59 conversions move this pin BY
    // DESIGN; deferred. Own `pass_rng("Dirt Rock Wall Runner")` stream.
    // W4-g36 A4 (code-only): the Wood Tree Walls pass (see SEED42_512
    // above) — the wall-244 fill moves this pin BY DESIGN; deferred.
    // RNG-free; the `a == b` halves stay untouched.
    // W4-g36 A6 (code-only): the Wall Variety pass (see SEED42_512 above)
    // — the cave-wall fill moves this pin BY DESIGN; deferred. Own
    // `pass_rng("Wall Variety")` stream; the shimmer-safety re-roll reads
    // the A1 centre (a cross-pass GenVars read, no shared-stream draw).
    // W4-g36 A7 (code-only): the Life Crystals pass (see SEED42_512 above)
    // — the 2×2 hearts (and the errorWorld/noSurface mana stars) move this
    // pin BY DESIGN; deferred. Own `pass_rng("Life Crystals")` stream.
    // W4-g36 gap-fill (code-only): the ShimmerCleanUp arm
    // (`WorldGen.cs:34716-34821`, `shimmer.rs::shimmer_clean_up`) — the Final
    // Cleanup pass's `:22472` shimmer tail, deferred back to g36 by g37's
    // Final Cleanup port (`4ced188f9`). UNWIRED by design (the g39/f25
    // convention): no executed step calls it yet, so BOTH pins stay UNCHANGED
    // on this branch. The merger composes it into g37's Final Cleanup at
    // `:22472`; at that point it moves the pins BY DESIGN (the inner-disc
    // cleanse, the outer-ellipse wall clear + Convert-0, the pool refill, the
    // anniversary Shimmerator flood, the tree-under-shimmer kill) — deferred
    // to the gate. Own draws ride the caller's `pass_rng("Final Cleanup")`
    // stream (the two NextDouble jitter rolls per outer-sweep cell); a == b
    // untouched (the W:7 witnesses replay it).
    // W4-g37 B1 (code-only): the Statues pass (see SEED42_512 above) places
    // its 73-style cycle on the default 4200-wide world — the statue bodies,
    // the four trap plates and their red wires move this pin BY DESIGN;
    // deferred to the gate.
    // W4-g37 B2 (code-only): the Surface Chests pass (see SEED42_512 above)
    // places its ~21 surface chests + the AddBuriedChest floor scans on the
    // default world — this pin moves BY DESIGN; deferred to the gate.
    // W4-g37 B3 (code-only): the Water Chests pass (see SEED42_512 above) —
    // the treasure + scatter chests move this pin BY DESIGN; deferred.
    // W4-g37 B4 (code-only): the Cave Walls pass (see SEED42_512 above) — the
    // wall fills move this pin BY DESIGN; deferred.
    // W4-g37 B5 (code-only): the Floating Island Houses pass (see SEED42_512
    // above) — the island houses move this pin BY DESIGN; deferred.
    // W4-g37 B6 (code-only): the Quick Cleanup pass (see SEED42_512 above) —
    // the cleanup sweep moves this pin BY DESIGN; deferred.
    // W4-g37 B7 (code-only): the Hellforge pass (see SEED42_512 above) — the
    // hellforges move this pin BY DESIGN; deferred.
    // W4-g37 B8 (code-only): the Spreading Grass pass (see SEED42_512 above) —
    // the surface conversion moves this pin BY DESIGN; deferred.
    // W4-g37 B9 (code-only): the Surface Ore and Stone pass (see SEED42_512
    // above) — the ore veins + stone blobs move this pin BY DESIGN; deferred.
    // W4-g37 gap-fill (code-only): the Final Cleanup pass
    // (`WorldGen.cs:22141-22490`, `final_cleanup.rs::pass_final_cleanup`)
    // lands after `water plants` — the wall-hole fill, the falling-sand
    // hardening, the boulder re-seats, the dungeon-wall liquid resets and the
    // 668 dirt throws move this pin BY DESIGN; deferred to the gate. The
    // dirt-throw scatter's `Next(50, maxX-50)` band collapses on the 512
    // world (the sub-vanilla guard skips it), but the sweep arms still run
    // there — SEED42_512 moves too, deferred. a == b untouched (the scatter
    // rides its own `pass_rng("Final Cleanup")` stream).
    // W4-g38 C2 (2026-11-21, code-only): the Traps pass (see SEED42_512
    // above) — on the 4200-wide default world the `width * 0.05` = 210
    // trap attempts place dart/boulder/explosive/geyser traps on the cave
    // floors (the 135 plates, 137 darts, 138-family boulders, 141
    // explosives, 443 geysers + the red wires), and the `width * 0.003` =
    // 13 sand-trap attempts carve the underground-desert chambers — this
    // pin moves BY DESIGN. The pass draws only its own fresh stream; the
    // downstream tile-dependent passes legitimately re-roll around the
    // new traps. Re-pin DEFERRED to the merge gate; a == b stays green.
    // W4-g38 C3 (2026-11-21, code-only): the Piles pass (see SEED42_512
    // above) — on the 4200-wide default world the seven ladders (0.06w cave
    // piles, 0.01w hell piles, 0.03w + 0.0035w surface/house 3×2 piles,
    // 0.6w + 0.02w + 0.15w small-pile scatters) place the 185/186/187 piles
    // across the surface and caves — this pin moves BY DESIGN. The pass
    // draws only its own fresh stream. Re-pin DEFERRED to the merge gate;
    // a == b stays green.
    // W4-g38 C4 (2026-11-21, code-only): the Spawn Point pass (see
    // SEED42_512 above) — on the classic default world the pass writes only
    // the spawn coordinates, no tiles, so this pin is EXPECTED UNCHANGED
    // (the world hash covers tile fields only; the spawn is not hashed).
    // The compact height-map anchor the world tail used retired into the
    // faithful pass. a == b stays green (own fresh stream).
    // W4-g38 C5 (2026-11-21, code-only): the Guide pass (see SEED42_512
    // above) — the roster + flags are not hashed and the pass draws
    // nothing, so this pin is EXPECTED UNCHANGED. a == b stays green.
    // W4-g38 C6 (2026-11-21, code-only): the Sunflowers pass (see
    // SEED42_512 above) — on the 4200-wide default world the `width *
    // 0.002` = 9 span attempts plant the 2×4 tile-27 sunflowers on the
    // surface grass — this pin moves BY DESIGN. The pass draws only its
    // own fresh stream. Re-pin DEFERRED to the merge gate; a == b stays
    // green.
    // W4-g38 C7 (2026-11-21, code-only): the Herbs pass (see SEED42_512
    // above) — on the 4200-wide default world the `width * 1.7` = 7140
    // PlantAlch attempts plant the stage-0 tile-82 herbs on the grass /
    // mud / jungle / sand / ash / snow soils (the deep + surface + full
    // y ladders, the ±15 count box) — this pin moves BY DESIGN. The pass
    // draws only its own fresh stream. Re-pin DEFERRED to the merge gate;
    // a == b stays green.
    // W4-g38 C8 (2026-11-21, code-only): the Dye Plants pass (see
    // SEED42_512 above) — on the 4200-wide default world the 4200 plantDye
    // attempts plant the tile-227 dye family (styles 0-7), the maturity
    // sweep advances the herb stages (82 → 83 → 84 + the deep-mud plants),
    // and the budget loops place the 2 glow tulips (656) + the 6 chillet
    // eggs (752) — this pin moves BY DESIGN. The pass draws only its own
    // fresh stream. Re-pin DEFERRED to the merge gate; a == b stays green.
    // W4-c33 pass 2 (2026-11-20, code-only): the faithful `Pyramids`
    // rewrite (`WorldGen.cs:15360-15460` + `Pyramid` `:27669-27971`) —
    // the anchor loop over `GenVars.PyrX/PyrY/numPyr` (the Dunes/Ocean
    // Sand producers), the 151 sandstone-brick hull with the wall-34
    // sweep, the entrance shaft, the zig-zag tunnel with the chamber
    // (the Pharaoh chest via the shared `AddBuriedChest` half, the 185
    // piles, the four 91 banners, the 28 pot row) and the post-chamber
    // surface tunnel — on its own `pass_rng("Pyramids")` stream, moved
    // to the vanilla slot after `ocean caves`. The pyramid tiles/walls
    // move directly, the srng tail shifts, and the downstream
    // tile-dependent passes re-roll around the new hulls. Stale-by-
    // design; the re-pin is DEFERRED to the merge gate (no compiler on
    // this branch). The a == b determinism halves stay untouched.
    // W4-c33 pass 3 (2026-11-21, code-only; MERGE KEEP-ONE matrix-88 §2.9:
    // d103's water_chests.rs owns the pass — the c33 copy dropped, the
    // slot note stands): `UnderwaterChests` / "Water
    // Chests" (`WorldGen.cs:17245-17355`) lands — the ocean-cave treasure
    // loop (the NextFromList main, the widening jitter box, the liquid>250
    // gate) + the 9·width/4200 scatter (the Next(10) 863 override, the
    // 186/4404/277/187 cycle, the two 10000-cap water hunts, the beach-band
    // trySlope) through the shared `AddBuriedChest` half (style 17), on its
    // own `pass_rng("Water Chests")` stream at the vanilla slot after
    // `chests`. The style-17 chests + the flattened floor slopes move the
    // water cells directly; the flag5 loot pair (4425/4460) joins the
    // stocking hook. Stale-by-design; the re-pin is DEFERRED to the merge
    // gate. The a == b halves stay untouched.
    // W4-c33 pass 4 (2026-11-21, code-only): the lily/cattail/bamboo arms
    // of `LilypadsCattailsBambooAndSeaweed` (`WorldGen.cs:21988-22017` +
    // `PlaceLilyPad` `:59086-59200`, `PlaceCatTail` `:58629-58742`,
    // `GrowCatTail` `:58974-59084`, `PlaceBamboo` `:58391-58445`) join
    // `ocean.rs::water_plants` ahead of the seaweed walk — the d103
    // deferral closes, the deferred draws join the same stream in vanilla
    // order, the 518/519/571 plants grow on the surface waters (the 571
    // frame-important boot bit lands). Stale-by-design; the re-pin is
    // DEFERRED to the merge gate. The a == b halves stay untouched.
    // W4-d101 pass 1 (code-only): the Sand Patches blobs (54 on a 4200-wide
    // world) retype the surface-to-rock band, so this pin moves BY DESIGN
    // too — deferred to the gate with the SEED42_512 note above.
    // W4-d101 pass 2 (code-only): Mount Caves raises 4 dirt mountains on a
    // 4200-wide world (`Mountinater`, the climbing circle) — this pin moves
    // BY DESIGN too; deferred to the gate.
    // W4-d101 pass 3 (code-only): the Dirt Wall Backgrounds sweep moves the
    // surface-band wall fields — deferred to the gate.
    // W4-d101 pass 4 (code-only): the Clay bands (100 + 252 + 100 blobs on
    // the default size) + the tidy sweep — deferred to the gate.
    // W4-d101 pass 5 (code-only): Small Holes' 7560 paired diggers carve
    // the underground — deferred to the gate.
    // W4-d101 pass 6 (code-only): Dirt Layer Caves' 151 long worms —
    // deferred to the gate.
    // W4-d101 pass 7 (code-only): Rock Layer Caves' 655 deep worms —
    // deferred to the gate.
    // W4-d101 pass 8 (code-only): the Generate Ice Biome walk retypes the
    // snow band's dirt/stone/clay/sand into 147/161 (+ wall 40) — a large
    // deliberate move; deferred to the gate.
    // W4-d101 pass 9: the marble/granite row split is RNG-neutral (above).
    // W4-d102 pass 1 (code-only): the Silt pass (see SEED42_512 above)
    // retypes stone to silt 123 across the rock layer on every world —
    // this pin moves BY DESIGN too; deferred to the merge gate.
    // W4-d102 pass 2 (code-only): the Webs pass (see SEED42_512 above) —
    // the cobweb 51 tiles move this pin BY DESIGN; deferred to the gate.
    // W4-d102 pass 3 (code-only): the Slush sweep (see SEED42_512 above)
    // is a no-op on the empty snow rect — UNCHANGED by this pass alone.
    // W4-d102 pass 4 (code-only): the Mountain Caves openings (see
    // SEED42_512 above) are a no-op while numMCaves == 0 — UNCHANGED by
    // this pass alone.
    // W4-d102 pass 5 (code-only): the Gems pass (see SEED42_512 above) —
    // the gem veins + sand slide move this pin BY DESIGN; deferred.
    // W4-d102 pass 6 (code-only): the Shimmer pass (see SEED42_512 above)
    // carves the shimmer pool + stone shell on every full-size world (the
    // 4200-wide default qualifies; only the sub-vanilla widths skip) —
    // this pin moves BY DESIGN; deferred to the gate.
    // W4-d102 pass 7 (code-only): the Clean Up Dirt pass (see SEED42_512
    // above) scrubs the surface dirt-wall band on every world — this pin
    // moves BY DESIGN; deferred to the gate.
    // W4-d102 pass 8 (code-only): the Dirt Rock Wall Runner pass (see
    // SEED42_512 above) retypes the wall-2 blobs to 59 — this pin moves
    // BY DESIGN; deferred to the gate.
    // W4-d102 pass 9 (code-only): the Wood Tree Walls sweep (see
    // SEED42_512 above) walls the living-tree interiors on the full-size
    // worlds — this pin moves BY DESIGN; deferred to the gate.
    // W4-d104 pass 28 (2026-11-20, code-only): the faithful `Floating
    // Island Houses` pass (above) moves this pin BY DESIGN — the default
    // 4200-wide world's 3-4 islands swap their plank-box houses for the
    // sunplate shells/interiors/furniture, and the house draws move off
    // the shared `drng` (the islands pass was its last consumer, so no
    // other step's ledger shifts). The re-pin is DEFERRED to the merge
    // gate (no compiler on this branch); the a == b determinism halves
    // stay untouched.
    // W4-d104 pass 29 (2026-11-21, code-only): the `Quick Cleanup` sweep
    // (above) moves this pin BY DESIGN — the slope/half-brick clears and
    // the wall retypes fire across the default world, and the beach sand
    // drip draws on the pass's own stream. Re-pin DEFERRED to the merge
    // gate; the a == b halves stay untouched.
    // W4-d104 pass 30 (2026-11-21, code-only): the `Hellforge` pass
    // (above) moves this pin BY DESIGN — the default world's 21 forge
    // retries place 77 forges in the hell houses. Re-pin DEFERRED to the
    // merge gate; the a == b halves stay untouched.
    // W4-d104 pass 30 graft (2026-11-21, code-only, research/86 §6.2):
    // the full PlaceTile-77 route (above) replaces the bare Place3x2 —
    // draw-schedule unchanged, tile bytes may move on residue seats /
    // refused-placement resets. Folds into the SAME deferred re-pin;
    // the a == b halves stay untouched.
    // W4-d104 pass 31 (2026-11-21, code-only): the `Spreading Grass`
    // classic arm (above) moves this pin BY DESIGN — the surface retypes
    // and the first-surface grass land across the default world. No RNG
    // draw; re-pin DEFERRED to the merge gate; the a == b halves stay
    // untouched.
    // W4-d104 pass 31 fix (2026-11-21, code-only, research/86 §6.1): the
    // repeat:true flip (above) fires the 3×3 recursion — more surface
    // cells grass, still zero draws. Folds into the SAME deferred
    // re-pin; the a == b halves stay untouched.
    // W4-d104 pass 32 (2026-11-21, code-only): the `Surface Ore and
    // Stone` pass (above) moves this pin BY DESIGN — the default world's
    // ore veins + stone blobs land on the surface band. Re-pin DEFERRED
    // to the merge gate; the a == b halves stay untouched.
    // W4-d104 pass 33 (2026-11-21, code-only): the `Place Fallen Log`
    // pass (above) moves this pin BY DESIGN — the default world's 1-3
    // fallen logs (488) land on the grass surface. Re-pin DEFERRED to
    // the merge gate; the a == b halves stay untouched.
    // W4-d104 pass 34 (2026-11-21, code-only): the `Traps` pass (above)
    // moves this pin BY DESIGN — the default world's ~210 traps + ~12
    // sand traps land underground. Re-pin DEFERRED to the merge gate;
    // the a == b halves stay untouched.
    // W4-d104 pass 35 (2026-11-21, code-only): the `Piles` pass (above)
    // moves this pin BY DESIGN — the default world's ~2500 pile decor
    // cells land across the seven scatters. Re-pin DEFERRED to the merge
    // gate; the a == b halves stay untouched.
    // W4-d104 pass 36 (2026-11-21, code-only): the `Spawn Point` pass
    // (above) leaves this pin UNCHANGED — the spawn fields are not
    // hashed and the search draws on its own stream. The a == b halves
    // stay untouched.
    // W4-d108 (M5-SEED temple branches, code-only): UNCHANGED for the same
    // reason as SEED42_512 above — every new arm sits behind a seed flag
    // the default world does not carry (the room-count multipliers, the
    // drunk spike rows, the remix search/fallback; `a == b` stays the
    // guard). The pre-existing S1-P-i33 staleness note still applies at
    // the gate.
    const DEFAULT_WORLD: u32 = 3647283075;
    // W4-d107 (2026-11-20, code-only): the special-seed branches land in
    // `caves.rs` (the drunk ore tier re-rolls + evil split + sky-band gate +
    // remix band table, `WorldGen.cs:13189-13602`; the SurfaceCaves noSurface
    // gate + remix triple, `:12164`/`:12170-12175`) and the dualDungeons
    // pruning name fix (`pipeline.rs` — `IceBiome` is "Generate Ice Biome",
    // `GenPassNameID.cs:239`, NOT "Ice" `:280`). BOTH CLASSIC PINS STAY:
    // every new draw is flag-gated and every pruning change needs the
    // dualDungeons micro-seed — a classic world runs the byte-identical
    // ledger. The AFFECTED SEEDS' hashes move by design and are pinned as
    // a==b witnesses instead of constants (no compiler on this branch):
    // `w4_d107_*` in `caves.rs` (drunk/remix/noSurface/surfaceIsInSpace pass
    // level) and `w4_d107_the_affected_seeds_are_deterministic` in
    // `worldgen/mod.rs` (whole-world a==b per affected seed + dualDungeons).
    // If a later gate measures per-seed constants, pin them there; the
    // classic constants above must NOT move for this task.
    // Re-pinned by S1-A-4 (the gen-time chest stocking): the chest sites draw
    // on their pass's stream now — the shrines' jungle item + `Next(2)`
    // x-jitter (`WorldGen.cs:10170-10196`/`:17220`), the pyramid's main-item
    // double roll (`:27853-27873`) and the hell house's `832`/`4281` roll
    // (`:29292-29297`) — and every chest writes its resolved `num9` style
    // into the frames (`:35978-35988`), so the srng tail and the chest tiles
    // move. Values re-measured on the merged tree (S1-A-3's pins + S1-A-4).
    // >>> cc526b1 (Merge branch 'M5-DUNGEON-lights': DungeonGlobalLights + the restored spike pass)
    // W4-f24 (2026-11-21, code-only): lane B ports the gated Dual Dungeons
    // Dither Snake pass's geometry — `snake_math.rs` (the f64 Vector2D +
    // Utils.Remap/Lerp/GetLerpValue/RotatedBy/SafeNormalize, ReLogic
    // `Vector2D.cs` + `Utils.cs:107/:315/:350/:730/:1686/:1846`) and
    // `dungeon_bounds.rs` (`DungeonBounds.cs:8-337`, the clamped hitbox
    // rectangle). UNWIRED: pure math/geometry, no pipeline step consumes
    // them yet — no RNG draw added, no tile written. BOTH CLASSIC PINS STAY
    // and the a == b determinism halves are untouched. The gated step (B6)
    // will draw only under `MicroSeedFlags::DUAL_DUNGEONS`, so a classic
    // world keeps the byte-identical ledger; the affected seed's hash moves
    // by design and rides an a == b witness (written NOT run).
    // W4-f24 B2: `dungeon_control_line.rs` ports `DungeonControlLine`
    // (`DungeonControlLine.cs:9-285`) — CanPaint's tangent/normal blend,
    // the Bayer style-transition + edge dither (the NextFloat/Next(3)/
    // Next(2)/Next(4) rarity rolls), Paint's brick+wall+unbreakable-tier
    // stamp, IsSelfIntersecting/AdjustTangents, IsInsideBorder and
    // GetPotentialRoomPosition. Still UNWIRED (B6 lands the gated step) —
    // no classic draw, both pins stay, a == b untouched.
    // W4-f24 B3: `dither_snake.rs` ports `DitherSnake`
    // (`DitherSnake.cs:10-199`) — Add/closest/GetLineContaining (depth-3
    // walk), GetPositionAlongSnake, the 12-point circle border tests
    // (ExtraBuffer 1/cos(π/6)), GetRoomPositionInsideBorder (the ×0.8
    // shrink + Top/Center/Bottom classification), SetTangents, the 100-sweep
    // AdjustTangentsToPreventSelfIntersection, and the PlotLine border walk
    // (Utils.cs:2233-2292 early-exit). Still UNWIRED — both pins stay.
    // W4-f24 B4: `dither_snake.rs` gains the `DitherSnakePass` body
    // (`DitherSnakePass.cs:22-133`) — CalculateDungeonDitherSnake (the
    // serpentine grid, the two NextDouble radius walks, the row-end curve
    // back, the dungeon-side end caps + self-intersection sweep),
    // GenerateDungeonDitherSnake (paint each line into the outer hitbox) and
    // ApplyPass's two-dungeon loop. Still UNWIRED (B5 publishes the producer
    // GenVars, B6 lands the gated step) — both pins stay, a == b untouched.
    // W4-f24 B5: `dungeon_genvars.rs` ports the producer chain —
    // SetupDungeonGenVarVariables's colour roll + drunk wall re-roll
    // (DungeonCrawler.cs:56-108), CreatePotentialDungeonBounds
    // (DungeonUtils.cs:2067-2087), the dual style quads (:111-128) with the
    // paint fields of the four statics c32 deferred (Cavern/Corruption/
    // Jungle/Temple) + Snow/Desert/Hallow + the GetCurrentDungeonStyle
    // wrapper (DungeonGenerationStyles.cs:95-135). The setup draws only
    // under DUAL_DUNGEONS (B6 wires it) — both classic pins stay.
    // W4-f24 B6: the gated step lands — `pipeline.rs` gains the
    // `dither snake` row (vanilla "Dual Dungeons Dither Snake",
    // WorldGen.cs:14755) between liquids and dungeon, the Dunes-pass dual
    // setup (:11520-11525, flag-gated) publishing GenVars.dual_dungeons, and
    // step_dither_snake (the NormalizedDistanceSafeFromDither :11465 +
    // both dungeons' calculate/paint). BOTH CLASSIC PINS STAY: the setup
    // and the step body are gated behind MicroSeedFlags::DUAL_DUNGEONS, so a
    // classic world runs the byte-identical ledger (the step is a no-op).
    // The dual seed's hash moves by design and rides the a == b witness
    // `w4_f24_the_dither_snake_is_gated_and_deterministic` (written NOT run).
    // W4-f24 A1: `skyblock.rs` ports the Skyblock statics + gate census
    // (WorldGen.cs:3063-3281) — the 75-name denyAllGeneration table + the
    // 22-name denySomeGeneration table (the AddPasses body census mapped to
    // VANILLA_CHAIN names), denySomeGeneration/denyFloatingIslands/
    // spawnSolidifier/spawnShimmerPool, and the SkyblockState ScanTiles/
    // Calculate ledger. pass_disabled_by_special_seeds gains the classic
    // SeedFlags arm (the skyblock pruning). STILL no classic draw — the
    // tables fire only under SeedFlags::SKYBLOCK; both pins stay.
    // W4-f24 A2: `skyblock.rs` gains SkyblockIsland (WorldGen.cs:22818-23556)
    // — the proximity veto, the 14-style palette, the two drifting ellipsoid
    // blobs + stalactite drip + tidy sweep, the style-1 shimmer pool, the
    // wall backing, the solidifier/monolith/music-box objects (the tileobject
    // registry seam + SwitchMonolith/SwitchMB), the cloud puffs + grass
    // fringe, the LandmassData ledger. STILL UNWIRED (A4 lands the gated
    // step) — both classic pins stay, a == b untouched.
    // W4-f24 A3: `seed_tails.rs` ports the skyblock generator's tail helpers
    // — DoPooEverywhere (WorldGen.cs:2966-2993), DoErrorWorldShuffleBlocks
    // (:1094-1278 + GetRandomBlock :1084 + SwapTiles :1280 +
    // errorWorldAdjustment :329), DoAdditionalChangesAroundSpawnIfNeeded
    // (:24936 + PlaceTorchesAroundSpawn :24989 + RemoveSpecificTilesUnderSpawn
    // :24953). STILL UNWIRED (A4 lands the gated step) — both pins stay.
    // W4-f24 A4: the gated Skyblock step lands — `skyblock.rs` gains
    // GenerateSkyBlockWorld (WorldGen.cs:22526-22816: the tile clear, the
    // spawn-centre seed bands, the spawn/team/shimmer islands, the flag2
    // per-seed island loop, the spawn-surface find, the extraLivingTrees
    // tree, the poo/error-world tails, the spawn-tail torches), the
    // pipeline `skyblock` row (after plan biome spans, before dunes) +
    // step_skyblock, and the GenVars landmass/spawn-randomized/dungeon-anchor
    // fields. BOTH CLASSIC PINS STAY: the generator + the A1 denyAll/denySome
    // pruning fire only under SeedFlags::SKYBLOCK; a classic world runs the
    // byte-identical ledger. The skyblock seed's hash moves by design and
    // rides `w4_f24_the_skyblock_world_is_gated_and_deterministic` (NOT run).
    // LANE A COMPLETE — both gated passes (Skyblock + Dither Snake) landed.

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
