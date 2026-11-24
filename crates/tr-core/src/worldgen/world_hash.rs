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
    // W4-d109 (code-only, the save half of S1-A-3): the `.trwld` v18 tail
    // persists `World::variants` and the framing passes reuse them on a save
    // load. This pin is UNCHANGED by that change: `generate()` runs only the
    // pipeline steps and never frames (framing is the app-side post-pipeline
    // seam), and `variants` is not a hashed `Tile` field — the pin hashes an
    // un-framed world. (The S1-P-i33 deferred re-pin above still stands.)
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
    // W4-c29 (code-only): the dead-man's-chest block lands BEFORE the
    // thin ice — the pin stays UNCHANGED: `generate()` carries no
    // chest-stock hook, so `world.chests` is empty, the candidate scan
    // finds nothing and the block draws only the DeadManChests config
    // roll on its own fresh `pass_rng("Micro Biomes")` copy (no tile
    // writes, no other step's ledger shifts). The 138 frame-important
    // boot bit is inert without the kind.
    // W4-c29 gem-moss fidelity (2026-11-21, code-only): `Spread.Gem`'s
    // duplicate-visit redraw + the 178 anchor gate/frame-row draw, the
    // `neonMossBiome` clamp fixes and the moss arm A/F changes all ride
    // the EXISTING `pass_rng("Gem Caves")` / `pass_rng("Moss")` slots —
    // no new stream, but the draw COUNTS and the painted tiles change, so
    // this pin moves BY DESIGN on any world whose gem/moss passes place
    // content. The 512 world's gem-cave band (`rockLayer+30 .. height-230`)
    // and moss bands collapse on the compact terrain, so the placement may
    // be nil — but the neon-moss arm A runs on every world (`width/2100`
    // rounds to 0 on 512, so arm A is skipped there too). Whether the pin
    // moves depends on the 512 terrain's pockets; the re-pin is DEFERRED
    // to the gate (no compiler on this branch). The a==b halves stay green.
    // W4-d105 lane 5 (2026-11-20, code-only): the classic unexecuted passes
    // #37-45 land one per commit — Guide (the gen-time starter-NPC ledger,
    // `WorldGen.cs:19685-19897`), Sunflowers (`:19898`), Herbs (`:19964`),
    // Dye Plants (`:19981`), Flowers (`:20447`), Mushrooms (`:20599`),
    // Random Gems (`:20729`), Moss Grass (`:20769`) and Larva (`:20868`).
    // Each draws from its own `pass_rng("<vanilla name>")` stream (no other
    // step's ledger shifts), but the tile writes move every world's hash BY
    // DESIGN. The re-pins are DEFERRED to the merge gate (no compiler on
    // this branch); the `a == b` determinism halves stay untouched. The
    // Guide pass writes no tiles (the NPC ledger is not hashed), so its
    // commit alone moves neither pin. #38 Sunflowers writes tile 27 on the
    // grass tops (the band attempts + the PlaceSunflower style draws), so
    // both pins move by design from that commit on. #39 Herbs plants tile
    // 82 (the PlantAlch attempts, `WorldGen.cs:45901-45938`) — both pins
    // move again by design. #40 Dye Plants writes tile 227 (the plantDye
    // attempts), matures the herbs (82 → 83 → 84) and plants the glow
    // tulips 656 + the chillet eggs 752 (`WorldGen.cs:19981-19994` +
    // `:71286-71455` / `:45748-45775` / `:45622-45678`) — both pins move
    // again by design. #41 Flowers recolours the grass tufts to the patch
    // styles (3 → 73) and plants the bare-top flowers
    // (`WorldGen.cs:20447-20598`) — both pins move again by design.
    // #42 Mushrooms turns the grass tufts / sunflowers into mushrooms
    // (frameX 144) and the glowing mushrooms tall (frameX 270)
    // (`WorldGen.cs:20599-20696`) — both pins move again by design.
    // #43 Random Gems scatters the exposed gems 178 (the weighted Next(12)
    // styles + the Next(3) frame_y) and the marble/granite diamond patches
    // (`WorldGen.cs:20729-20768`) — both pins move again by design.
    // #44 Moss Grass grows the long moss 184 on the free neighbours of
    // every moss tile (`WorldGen.cs:20769-20812`) — both pins move again
    // by design. #45 Larva moves the larva placement out of the hive pass
    // into its own slot (the 3×4 clear + the Place3x3-231 writes,
    // `WorldGen.cs:20868-20900`) — the hive-pass larva tiles now land later
    // (after the muds-walls scan), so both pins move again by design.
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
    // W4-a2 / P0-3 (2026-11-20, code-only): the chest tile-kind audit
    // (`chest_gen::CHEST_KINDS` [21,341] -> [21,467], `TileID.cs:570`) moves
    // NEITHER pin: every worldgen chest placement already writes kind 21
    // (`structures::place_chest`, the C#'s `chestTileType 0 -> 21` default,
    // `WorldGen.cs:35880-35882`) — no site ever placed 341, the fixes touch
    // the load-cull set / test plants / the cave-house exclusion probe
    // (21-only -> BasicChest {21,467}, a no-op while no gen pass writes 467)
    // only. No tile byte and no RNG draw changed; ANALYSIS "Chest tile-kind
    // audit (W4-a2 / P0-3)".
    // W4-c32 (2026-11-20, code-only): the dungeon biome style tables land
    // (`worldgen/dungeon_styles.rs` — DungeonGenerationStyles.cs's
    // Snow/Desert/Crimson/Hallow statics + Crystal, GetCurrentDungeonStyle,
    // GetStyleForTile/GetStyleForWall). UNWIRED data tables + pure lookups:
    // no pipeline step consumes them, no RNG draw is added and no tile is
    // written, so BOTH pins are UNCHANGED and the a == b determinism
    // assertions stay green (the S1-K-2 wave B constants-off class). The
    // wiring round (the dual-dungeon list arms + the per-style feature
    // passes) re-pins when it lands.
    // W4-c32-convert (2026-11-21, code-only): the convert-tables family
    // SEAL (research/315, chain 281→315 per registry 150 §9) — tests + docs
    // only (3 table-pin witnesses in
    // dungeon_styles.rs; the BiomeDungeonRoom bodies use
    // DungeonUtils.ChangeTileType NOT WorldGen.Convert, f25's Convert
    // 15/16/18/19 cells stay zero-call-site UNWIRED on f25, g42's
    // Convert(3) closes at four sites). No game-logic change: BOTH pins
    // UNCHANGED, a == b untouched.
    // W4-d109 (code-only): the `.trwld` v18 variants persistence leaves this
    // pin UNCHANGED for the same reason as SEED42_512 above — `generate()`
    // never frames and `variants` is unhashed. The i33 deferred re-pin stands.
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
    // W4-c29 (2026-11-21, code-only): the dead-man's-chest block
    // (`DeadMansChestBiome.cs:10-625` + `WorldGen.cs:20916-20938`) draws
    // only the DeadManChests config roll on the no-hook path (empty
    // `world.chests` → no candidates → no tile writes), on its own fresh
    // `pass_rng("Micro Biomes")` copy — this pin is UNCHANGED by W4-c29.
    // The stocked tr-app path traps gold chests by design (the 467
    // retype, the 137/138/141 traps, the red wires); that moves the
    // APP-side world, not this tr-core pin. The S1-P-i33 deferral above
    // still stands; the gate re-pins sequentially.
    // W4-c29 gem-moss fidelity (2026-11-21, code-only): the gem/moss
    // fidelity fixes (the `Spread.Gem` duplicate-visit redraw, the 178
    // anchor gate + `Next(3)*18` frame-row draw, the `neonMossBiome`
    // clamp/step/loop fixes, the moss arm A reject ledger + dungeon
    // exclusion, the arm F live column-major `Main.tileMoss` sweep) change
    // the gem-cave and moss-pass tiles and draw counts on the DEFAULT
    // 4200x1200 world — arm A places `4200/2100 = 2` neon biomes and the
    // gem/moss cave passes find pockets, so this pin moves BY DESIGN. The
    // re-pin is DEFERRED to the merge gate (no compiler on this branch);
    // the `a == b` determinism halves stay green. The 178/184
    // frame-important boot bits (`Main.cs:7413-7414`) keep the stored gem
    // frames through the load-time framer.
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
