# 12 — WorldGen / WorldBuilding passes

> Wave 1 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP

## Scope

The C# side is the world-construction half of the game: the god file
`decompiled/Terraria/WorldGen.cs` (87 372 lines; `docs/ANALYSIS.md:37`) plus the
`Terraria.WorldBuilding` package (40 files, 5 298 lines — `WorldGenerator.cs` 551,
`GenVars.cs` 477, `Modifiers.cs` 675, `WorldGenSnapshot.cs` 382, `WorldGenRange.cs`
65, `WorldGenConfiguration.cs` 51, `Shapes.cs` 230, `Actions.cs` 637, `TileFont.cs`
386, `Searches.cs` 127, `ModShapes.cs` 116, `WorldManifest.cs` 64, the nine
`WorldSeedOption_*.cs`), plus the biome/structure helper classes under
`Terraria.GameContent.*` that the passes instantiate (`Biomes/*`, `Generation/*`:
`DunesBiome`, `DesertBiome`, `MarbleBiome`, `GraniteBiome`, `HoneyPatchBiome`,
`CaveHouseBiome`, `CampsiteBiome`, `SpikePitBiome`, `EnchantedSwordBiome`,
`DungeonCrawler`, `TileFont`). `docs/ANALYSIS.md:79` prices the whole generation
subsystem at "~45 000 lines".

Responsibility: turn a seed into a finished tile world. `WorldGen.Reset`
(`WorldGen.cs:11109-11282`) initialises the GenVars statics and rolls the layout
(`jungleOriginX` `:11341-11376`, `crimsonLeft` `:11216-11223`, the beach anchors
`:11415-11440`, the `flipInfections` triple `:11125-11127`), `AddPasses`
(`WorldGen.cs:11502-22143`) builds the ~110-`GenPass` pipeline,
`DisablePassesForSpecialSeeds` (`WorldGen.cs:22493`) prunes it for seeds,
`GenerateWorld` (`WorldGen.cs:11071-11107`) drives it, and
`WorldGenerator.GenerateWorld` (`WorldBuilding/WorldGenerator.cs:419-486`) walks
`_passes`, re-seeding `Main.rand` per pass (`:497`) and hashing each result
(`HashWorld` `:517-545`). After the pipeline, `GenerateWorld`'s tail calls the
spawn-point pass and the seed `Finish*` passes.

The Rust side is `crates/tr-core/src/worldgen/` — 31 files, ~32 800 lines
(`mod.rs` 1 066, `pipeline.rs` 3 066, `evil.rs` 4 280, `temple.rs` 3 350,
`ocean.rs` 2 661, `biomes.rs` 2 709, `tile_runner.rs` 1 625,
`biome_detail.rs` 1 760, `spider_gem_moss.rs` 1 669, `living_trees.rs` 1 113,
`desert.rs` 980, `finish_seeds.rs` 961, `dungeon.rs` 940,
`desert_entrances.rs` 844, `mini_biomes.rs` 855, `micro_surface.rs` 574,
`structures.rs` 558, `terrain.rs` 477, `desert_dunes.rs` 460, `islands.rs` 461,
`caves.rs` 381, `micro_finish.rs` 610, `wavy.rs` 618, `surface.rs` 353,
`meteor.rs` 309, `config.rs` 299, `world_hash.rs` 229, `micro_grass.rs` 188,
`liquids.rs` 265, `noise.rs` 106) plus the shared `crates/tr-core/src/rng.rs`
xorshift core. The module header (`worldgen/mod.rs:1-14`) states the scope
outright: "a compact generator modelled on the shapes of `WorldGen`'s passes …
rather than a port of all 45k lines".

## Key entities

- `decompiled/Terraria/WorldGen.cs::AddPasses` (`:11502-22143`) — the 109
  registration sites; the authoritative pass order and the anchors the Rust chain
  mirrors. `setWorldSize`/`SetWorldSize` (`:6551-6589`) hold the 4200×1200 /
  6400×1800 / 8400×2400 size table.
- `decompiled/Terraria/WorldGen.cs::Reset` (`:11109-11282`) — the pre-pass state
  setup: GenVars zeroing, the `jungleOriginX`/`crimsonLeft`/beach/hell-chest-pool
  rolls, the flag reads (`:11115-11127`).
- `decompiled/Terraria/WorldGen.cs::GenerateWorld` (`:11071-11107`) —
  `DisablePassesForSpecialSeeds` → `_generator.GenerateWorld()` → the Finish tail.
- `decompiled/Terraria/WorldGen.cs::SpawnPoint` (`:19460-19588`) — the vanilla
  spawn selector (vanilla's last pass before the Grass/Guide tail).
- `decompiled/Terraria/WorldBuilding/WorldGenerator.cs::RunPass` (`:486-515`) —
  `Main.rand = new UnifiedRandom(this._seed)` before every pass, the progress
  start/end, and `Invariant.Assert(Liquid.numLiquid == 0 …)` (`:508`).
- `decompiled/Terraria/WorldBuilding/WorldGenerator.cs::HashWorld` (`:517-545`) —
  per-column `h ^= tile.GetHashCode(); h = rotl13(h)*5 + 3864292196`, then the same
  mix across columns.
- `decompiled/Terraria/GameContent/Passes/PassLegacy.cs` (`_weightMap`
  `:141-243`, default `1f` `:27-34`) — the progress weights.
- `decompiled/Terraria/WorldBuilding/GenPassNameID.cs:221-331` — the display-name
  strings the Rust chain reproduces verbatim.
- `crates/tr-core/src/worldgen/mod.rs::generate` (`:122-144`) — the single entry
  point: `boot_statics` → `Noise` → `WorldGenConfig::parse` → `Streams::new` →
  `pipeline::run`; `GenOptions` (`:52-84`, default 4200×1200/seed 1) and
  `roll_crimson` (`:89-91`, `(seed >> 56) & 1`).
- `crates/tr-core/src/worldgen/pipeline.rs::VANILLA_CHAIN` (`:370-1028`) — 109
  `PassDecl`s (name, weight, `WorldGen.cs` anchor, gate) that model the C# order;
  never walked at runtime (`#[allow(dead_code)]` on `anchor`/`gate`).
- `crates/tr-core/src/worldgen/pipeline.rs::EXECUTED_PIPELINE` (`:1058-1206`) — 62
  `Step`s (display name → vanilla name → adapter fn) in the port's pinned
  execution order; each step body is the verbatim `generate()` call site.
- `crates/tr-core/src/worldgen/pipeline.rs::Streams` (`:83-123`) — `fnv1a64`
  (`:68-77`) plus the three contract streams (`rng`/`drng`/`srng`, seeds
  `seed ^ 0xA0761D6478BD642F` / `^ 0x5DEECE66D1A49B5F` / `^ 0x9E3779B97F4A7C15`)
  and `pass_rng(name) = Rng(seed ^ fnv1a64(name))` (`:120-122`).
- `crates/tr-core/src/worldgen/pipeline.rs::GenVars` (`:152-313`) — the port's
  slice of the C# GenVars statics (jungle bounds/origin, temple bounds, beach and
  shell anchors, pyramid slots, desert locations, ocean-cave treasures,
  `crimson_left`, `flip_infections`), reset to the C# values in `Default`.
- `crates/tr-core/src/worldgen/pipeline.rs::run` (`:2497-2527`) — the
  `RunPass`-shaped loop: the Reset stand-in (`:2505-2512`), then per step the
  config view lookup and the adapter call; nothing reads a gate.
- `crates/tr-core/src/worldgen/pipeline.rs::boot_statics` (`:2410-2491`) — the
  `Main.tileFrameImportant` boot bits (`Main.cs:7663-7687` et al.) the generated
  world must carry.
- `crates/tr-core/src/worldgen/pipeline.rs::step_world_tail` (`:2355-2401`) — the
  non-vanilla tail: `world_surface`/`rock_layer`, the spawn (classic
  `map.spawn_center`, `heights[center]-6`; the remix bottom-centre arm
  `WorldGen.cs:19574-19584`), `time = 27000`, `moon_type = rng % 9`,
  `hell_back_style = rng % 3`.
- `crates/tr-core/src/worldgen/world_hash.rs::world_hash` (`:42-57`,
  `tile_hash` `:29-38`) — the `HashWorld` port with the pinned constants
  `SEED42_512 = 3075125982` / `DEFAULT_WORLD = 366022756` (`:173-174`).
- `crates/tr-core/src/worldgen/config.rs` — the verbatim embedded
  `Terraria.GameContent.WorldBuilding.Configuration.json` plus
  `WorldGenRange::get_random`'s scaling (`WorldArea = maxTilesX*maxTilesY/5040000`,
  `WorldWidth = maxTilesX/4200`).
- `crates/tr-core/src/worldgen/terrain.rs` — `surface_height_map` (`:71-204`,
  `spawn_center = width/2` at `:80`), `fill_terrain` (`:207-372`), `Layers`
  (`:45-69`), the ≥20-recentering exclusion note (`:54-61`).
- `crates/tr-core/src/seeds.rs` — `OPTION_TABLE` (`:112+`, the nine classic
  options in `WorldGenerationOptions.cs:30-41` order) and `MICRO_SEED_ORDER`
  (`:88` `NotTheBees`, … `:162` `dontdigup`).
- `crates/tr-app/src/ui/world_creation.rs::size_dimensions` (`:662`) and
  `create_world_in` (`:746-774`) — the size table and the seed/flags feed into
  `worldgen::generate`; test at `:1397`.

## Dependencies

- Calls into: `world.rs` (`World::new`, `tiles`, `spawn_*`, `frame_important`,
  `seeds`/`micro_seeds`), `tile.rs` (`Tile`, flags), `tiledata/` (tile ids, the
  generated `CanBeClearedDuringGeneration`), `framing.rs` + `edit.rs`
  (`square_tile_frame_gen`) for the post-pipeline framing the generator relies on,
  `growth.rs` (`spread_grass` stand-in), `rng.rs` (the xorshift core), `seeds.rs`
  (flags), `chest_gen.rs`/`place_chest` (loot rows), `log` for the per-step trace.
- Called by: `crates/tr-app/src/ui/world_creation.rs` (world creation),
  `crates/tr-app/src/app/mod.rs` (`:3769-3789` — the default-world regen with
  `GenOptions::default()`), `TR_REGEN`/`TR_SEED` dev flags in `main.rs`/`app`.
- Shared data structures: `GenVars` ↔ `Layers`/`SurfaceMap`/`BiomeSpans`
  (terrain products consumed by caves/ores/jungle/ocean/temple), `Streams`
  (three shared + per-pass streams), `WorldGenConfig`/`ConfigSection`
  (`config.rs`), `SeedFlags`/`MicroSeedFlags`, `StructureMap`-shaped protection is
  **not** ported — the C#'s `StructureMap` (`WorldBuilding/StructureMap.cs`, 94
  lines) has no Rust counterpart; passes that would register rects just write
  tiles.

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| Pass registration + order (`AddPasses` `WorldGen.cs:11502-22143`) | PARTIAL | `pipeline.rs::VANILLA_CHAIN` `:370-1028` | 109 decls mirror the C# anchors and display names; `pipeline::the_vanilla_chain_declares_106_classic_passes` pins 106 classic + skyblock pair + dither snake. It is a *model*: `anchor`/`gate` are `#[allow(dead_code)]`, nothing walks the chain. |
| Runner + progress weights (`WorldGenerator.cs:419-515`, `PassLegacy.cs:141-243`) | PARTIAL | `pipeline.rs::run` `:2497-2527` | Loop shape ported; weights only feed a `log::debug!` (`:2517-2523`) — no `GenerationProgress`, no `UIWorldLoad`, no `PauseAfterPass`/`Controller` debug pause. |
| Per-pass reseed (`WorldGenerator.cs:497`) | PARTIAL | `pipeline.rs::pass_rng` `:120-122` | xorshift stand-in for `UnifiedRandom`, and `seed ^ fnv1a64(name)` instead of `new UnifiedRandom(seed)` per pass — deliberate and documented at `:109-118`. |
| World hash (`HashWorld` `:517-545`) | PORTED (own tile mix) | `world_hash.rs::world_hash` `:42-57` | Fold verbatim (`rotl13 * 5 + 3864292196`, u32 wrap); the per-tile hash is the port's own because .NET `TileStruct.GetHashCode` is not worth reproducing (`:9-13`). Cannot be compared with vanilla manifest values |
| World size table (`SetWorldSize` `:6572-6589`) | PORTED | `ui/world_creation.rs::size_dimensions` `:662` (+test `:1397`) | 0→4200×1200, 1→6400×1800, 2→8400×2400 |
| Configuration (`WorldGenConfiguration.cs`, `WorldGenRange.cs`, the embedded JSON) | PORTED (data-only) | `config.rs` | JSON embedded verbatim + `get_random` scaling; **every adapter ignores the view** (`run` `:2524` builds `pass_config`, the step fns take `_pass_config`) — constants stay hard-coded per pass |
| GenVars statics (`GenVars.cs`, 477 lines) | PARTIAL | `pipeline.rs::GenVars` `:152-313` | ~25 fields of the C#'s ~100 statics; `Default` re-zeroes them to the C# reset values |
| `Reset` (`WorldGen.cs:11109-11282`) | PARTIAL | `run` `:2505-2512` | `crimson_left` drawn from `pass_rng("Reset")`, `flip_infections` derived; the other ~20 Reset rolls (hell chest pool shuffle `:11274-11297`, `slimeRainTime`, the `crimStone/ebonStone` swap) are scattered into the owning passes or missing |
| Terrain (`TerrainPass.cs`) | PARTIAL | `terrain.rs:71-372` | Height map + column fill + `Layers` quartet are load-bearing; the surface curve is FBM value noise (`noise.rs:16-45`), not the C#'s; the ≥20 recentering (`TerrainPass.cs:217-228`) is skipped (`terrain.rs:54-61`); the `surfaceIsInSpace` band/headroom is mapped onto the compact model with a documented approximation (`terrain.rs:230-260`) |
| Cave family (`Tunnels` `:11817`, `Mount Caves`, `Clay`, `Small Holes`, `Rock/Dirt Layer Caves`, `Surface Caves`) | PARTIAL | `caves.rs::step_cave_fields` (step `vanilla: "Tunnels"`) | One compact cavern+worm field covers five C# passes; `Sand Patches`, `Slush`, `Silt`, `Clay` have no step |
| Ores (`OresAndShinies` `:13182-13606`, `TileRunner` `:76770-77142`) | PORTED | `caves.rs` (`ores_and_shinies`, `roll_tiers`, `band_table`), `tile_runner.rs` | Density bands, tier flips `:11274-11297`, the walker and its draw ledger are ported and ledger-tested |
| Evil (`CorruptionAndCrimson` `:13866-14554`) | PORTED | `evil.rs` | Verbat-ish port incl. `ChasmRunner`/`Crim*`, the shell tables, the anchors; at the vanilla slot between `ores and gems` and `liquids` |
| Jungle family (`JunglePass.cs`, `Mud Caves To Grass` `:12468`, `Dirt To Mud` `:13123`, `Wet Jungle` `:15810`, plants `:20122`…, `MudsWallsInJungle` `:20813`) | PORTED | `biomes.rs`, `biome_detail.rs` | Faithful per-need passes with their own streams |
| Ocean family (`Ocean Sand` `:11580`, `Beaches` `:14888`, `TuneOceanDepth` `:26960`, `Gravitating Sand` `:15138`, `oceanCave` `:76452`, `Remove Water From Sand` `:16211`, `Shell Piles` `:16266`, coral/palms `:21310`) | PORTED | `ocean.rs` (+ `desert_dunes.rs` for `Dunes` `:11517`) | All six M5-OCEAN phases landed; the oasis block is dead in the port (`ocean.rs:1806-1811`) |
| Desert (`DesertBiome` `:12500`, `Dunes` `:11517`, the four entrances) | PORTED | `desert.rs`, `desert_dunes.rs`, `desert_entrances.rs` | Inline `PlotLine`/`Shapes`/`Modifiers` ports; special-seed arms (`surfaceIsDesert`/`remix`/`tenthAnniversary`/`dualDungeons`) deferred |
| Dungeon (`Dungeon` `:14757`, `DungeonCrawler.cs`) | PARTIAL | `dungeon.rs::generate_dungeon` | The faithful crawler (setup rolls, Legacy layout/hall/room walkers) replaced the shape pass 2026-11-09; out: the entrance-hall precalculation (`:520`), `DungeonGlobal*` feature passes, the biome styles (`dungeonGenerationStyles`, incl. dual-dungeon) |
| Temple (`LihzahrdTemple` `:15834`, `makeTemple` `:33435`, `templePart2` `:34464`, `LihzahrdAltar` `:21943`) | PORTED (with follow-ups) | `temple.rs` (3 350 lines) | Placement search, rooms, corridors, sweeps, traps, chests, furniture, altar re-write — cargo row marks 🟡/🔄; loot tables / Golem gate / secret-seed arms remain |
| Structures (shrines `:15977`, chests, altars `:15750`, pyramids `:15367`, pots `:17996`, hives `:15927`, hell forts `:32683`, mini-biomes) | PARTIAL | `structures.rs`, `mini_biomes.rs` | Shape-level for several; `Life Crystals`, `Statues`, `Traps`, `Piles`, `Larva` (bee larvae) passes have **no** step |
| Living trees (`:15476-15715`, `GrowLivingTree` `:27973`) | PORTED | `living_trees.rs` | Faithful, incl. the drunk/`extraLivingTrees` ladders; cargo row says done (ROADMAP's status table still says "parked") |
| Floating islands (`:12924-13133`, houses `:17862`) | PARTIAL | `islands.rs` | Shape-level cloud disc + lake + house; snow/desert variants and furniture sets out |
| Plants & surface dressing (`Planting Trees` `:20179`, `Herbs`, `Dye Plants`, `Weeds`, `Flowers`, `Mushrooms`, `Spreading Grass`, `Surface Ore and Stone`, `Place Fallen Log`, `Cactus…` `:21310`, `Water Plants` `:21973`) | PARTIAL | `surface.rs::plant_trees` (step `"trees"`→`"Planting Trees"`), `biome_detail`/`ocean` steps | Only the tree step, the weeds/flora step and the cacti/coral seaweed arms map to vanilla slots; `Herbs`, `Dye Plants`, `Flowers`, `Mushrooms`, `Moss Grass`, `Spreading Grass`, `Surface Ore and Stone`, `Place Fallen Log` are **MISSING** |
| Liquids (`Settle Liquids` `:19620`, `Settle Liquids Again` `:22137`, `Lakes` `:14556`, `Waterfalls` `:16667`) | PARTIAL | `liquids.rs` (the `Lakes` step) | Static water/lava lakes only; **no liquid simulation pass** and therefore no equivalent of the `Invariant.Assert(Liquid.numLiquid == 0)` (`WorldGenerator.cs:508`); both `Settle Liquids` and `Waterfalls` are missing |
| Shimmer (`Shimmer` `:15195`) | MISSING | — | Not a worldgen liquid in the port (admitted in ANALYSIS M5-STRUCT-5's list) |
| Spawn selection (`SpawnPoint` `:19460-19588`) | PARTIAL | `pipeline.rs::step_world_tail` `:2355-2401` | Classic: `map.spawn_center` and `heights[center]-6`; the vanilla pass scans for a valid cell (and the remix arm). No `randomSpawn`/`teamBasedSpawns` |
| Starter NPC (`Guide` `:19850`) | MISSING | — | No spawn-starter-NPC step (spawning is module 24) |
| Wall polish (`Cave Walls` `:17711`, `Wall Variety` `:16950`) | MISSING | — | The passes that re-texture cave walls are not ported |
| Cleanup (`Quick Cleanup` `:17949`, `Tile Cleanup` `:21200`, `Remove Broken Traps` `:22130`, `Final Cleanup` `:22141`) | MISSING | — | No steps; the port relies on its framing pass |
| Special seeds (9 classic + 37 micro) | PARTIAL | `seeds.rs`, `finish_seeds.rs`, `micro_*.rs` | Framework (option table, BCrypt `to_secret`, flags on `World`/`GenOptions`, TRWLD v7) plus the ported arms; `noSurface`, `roundLandmasses`, `errorWorld`, `extraLiquid`, `dualDungeons`, `teamBasedSpawns`, `randomSpawn`, `addTeleporters`, `graveyardBloodmoonStart` deferred (admitted in ROADMAP M5-SEED-1/2) |
| Skyblock world gen (`WorldGen.skyblockWorldGen` gate, the duplicated `Jungle` registration `WorldGen.cs:11509`) | MISSING | `VANILLA_CHAIN` gate text only | The gate is never read at runtime, so a skyblock seed runs the classic pipeline (behavioural) |
| World snapshot / manifest / progress screen (`WorldGenSnapshot.cs`, `WorldManifest.cs`, `CheckLatestPassResultAgainstManifest`) | MISSING | — | `world_hash.rs` ports only the hashing half |
| Tile/`GenVars` framing seam (`framing`, `square_tile_frame_gen`) | PORTED | `edit.rs::square_tile_frame_gen` + `pipeline.rs::boot_statics` `:2410-2491` | Frame-important kinds are set pre-pipeline so the generated world survives the framer |

Coverage arithmetic (verified by scripted diff of `VANILLA_CHAIN` names against
`EXECUTED_PIPELINE`'s `vanilla` fields): 62 executed steps map onto 54 distinct
vanilla names; **52 of the 106 classic passes have no executed step** —
`Sand Patches`, `Mount Caves`, `Dirt Wall Backgrounds`, `Clay`, `Small Holes`,
`Dirt Layer Caves`, `Rock Layer Caves`, `Surface Caves`, `Generate Ice Biome`,
`Granite` (folded into the `"Marble"` step), `Silt`, `Webs`, `Slush`,
`Mountain Caves`, `Gems`, `Shimmer`, `Clean Up Dirt`, `Dirt Rock Wall Runner`,
`Wood Tree Walls`, `Settle Liquids`, `Oasis`, `Waterfalls`, `Wall Variety`,
`Life Crystals`, `Statues`, `Surface Chests`, `Water Chests`, `Cave Walls`,
`Floating Island Houses`, `Quick Cleanup`, `Hellforge`, `Spreading Grass`,
`Surface Ore and Stone`, `Place Fallen Log`, `Traps`, `Piles`, `Spawn Point`,
`Grass Wall`, `Guide`, `Sunflowers`, `Herbs`, `Dye Plants`, `Flowers`,
`Mushrooms`, `Random Gems`, `Moss Grass`, `Larva`, `Settle Liquids Again`,
`Tile Cleanup`, `Stalac`, `Remove Broken Traps`, `Final Cleanup`.

Special seeds: `notthebees`, `drunk`, `celebrationmk10`, `dontdigup`,
`fortheworthy`, `notraps`, `remix`, `getfixedboi`, `skyblock` are all present as
names/values in `seeds.rs::OPTION_TABLE`, and all 37 micro-seeds are present in
`MICRO_SEED_ORDER` with their BCrypt hashes pinned (the golden vectors
`pumpkinseason`, `arachnophobia`, `noSpiderCaves`, `extraLiquid`, `dualDungeons`
at `seeds.rs:700-733`). Only their *presence* was checked per the audit brief.

## Tests

244 `#[test]` functions live under `crates/tr-core/src/worldgen/` (grep of
`#[test]` + the following `fn`). The load-bearing ones for this module:

- Determinism / byte-identity proof (the "worldgen FNV proof" the AGENTS.md
  determinism rule points at): `world_hash::the_512_seed_42_world_hashes_to_the_pinned_constant`,
  `world_hash::the_default_world_hashes_to_the_pinned_constant` (both assert two
  generations are equal *and* equal to the pinned constant),
  `world_hash::the_hash_covers_every_tile_field`,
  `pipeline::the_runner_reproduces_generate_tile_for_tile`,
  `pipeline::test_DW_1_1_fnv1a64_known_vectors`,
  `pipeline::test_DW_1_1_pass_rng_deterministic_and_name_distinct`,
  `mod::deterministic_for_seed`.
- Model/chain integrity: `pipeline::the_vanilla_chain_declares_106_classic_passes`,
  `pipeline::every_executed_step_maps_into_the_chain`,
  `config::the_embedded_copy_is_unmodified`,
  `config::world_gen_range_matches_the_csharp_scaling_and_bounds`,
  `pipeline::genvars_defaults_carry_the_csharp_reset_values`.
- Vanilla-slot positional tests (the "no silent re-order" contract):
  `test_DW_3_3_new_steps_sit_in_their_vanilla_positions`,
  `test_DW_2_3_ocean_steps_sit_in_their_vanilla_positions`,
  `test_DW_4_4_late_ocean_steps_sit_in_their_vanilla_positions`,
  `test_DW_4_3_evil_step_sits_between_ores_and_liquids`,
  `test_DW_5_3_new_steps_sit_in_their_vanilla_positions`,
  `test_DW_5_6_cave_decor_steps_sit_in_their_vanilla_positions`.
- Content smoke tests: `mod::has_surface_trees_and_caves`,
  `mod::living_trees_generate_for_seed`, `mod::desert_band_has_sand_dunes_and_no_trees`,
  `mod::the_biome_passes_place_the_missing_biomes`,
  `mod::the_temple_buries_in_the_jungle_cavern`,
  `mod::test_DW_1_4_generated_world_carries_frame_important_bits`,
  `mod::m5_seed_1_the_secret_seed_worlds_differ_from_the_classic`.
- Per-pass fidelity: 40+ draw-ledger tests (`tile_runner::draw_ledger_matches_the_csharp_replay`,
  `evil::test_DW_4_3_shell_draw_ledger`, `ocean::ocean_sand_draw_ledger`, …) and
  15 "deterministic for a seed" module tests (`desert::cluster_group_is_deterministic_for_a_seed`,
  `temple::test_DW_2_4_temple_is_deterministic`, …).

No cross-check against a real vanilla `.wld` exists anywhere — every pin is
port-internal (pre-scaffold or post-phase hash constants, screenshot SHA pairs).

## Deviations & risks

Every row below was re-read from code; ROADMAP `DONE` rows are treated as claims.

1. **Scope-level: the generator is not a port.** `mod.rs:1-14` (and
   `ANALYSIS.md:1004-1026`) state it is "a compact generator modelled on the shapes
   of WorldGen's passes … rather than a port of all 45k lines". Severity:
   behavioural (whole-world). Admitted: yes, in the module header, ROADMAP
   M5-GEN-1 and PORT_MAP.md:34 — but the golden rule's "1:1" framing makes this the
   single largest gap in the audit's remit.
2. **RNG divergence, by design.** `Streams::pass_rng` is xorshift64 seeded
   `seed ^ fnv1a64(name)` where vanilla re-seeds `UnifiedRandom(seed)` per pass
   (`WorldGenerator.cs:497`). Draw *order* is faithful; the byte stream cannot be,
   so no generated world can ever match a vanilla world tile-for-tile. Severity:
   pixel/determinism. Admitted: `pipeline.rs:109-118` and per-section ANALYSIS
   notes. Risk: ROADMAP's repeated phrase "classic worlds byte-identical"
   (M5-BIOME, M5-OCEAN, M5-SEED-1/2, …) means *identical to the port's previous
   state*, not to vanilla — a doc reader can easily misread it as vanilla parity.
3. **Config layer is parsed but not consumed.** ROADMAP M5-GEN-1 claims
   "config-driven passes"; `run` (`:2524`) builds `pass_config` per step and every
   adapter takes `_pass_config` unused, so every C# `Configuration.json` value is
   still a hard-coded constant in the pass bodies. Severity: behavioural for any
   value a mod/reference config would change. Admitted: only in the `run` doc
   comment (`:2493-2496`) — not in ROADMAP or ANALYSIS.
4. **Chain weights are `_weightMap` values, not ctor weights, and unused.**
   `Terrain` 507.352 vs `TerrainPass.cs:51-54`'s 449.3721923828125; `Jungle`
   11205.83 vs `JunglePass.cs:14-17`'s 10154.65234375; `Cactus, Palm Trees, & Coral`
   1.0 because the map key is `"Cactus & Coral"`. Nothing consumes them except a
   `log::debug!`. Severity: cosmetic (the weights only ever drove the C# progress
   bar, which is not ported). Admitted: not in the docs, only in the code's own
   notes.
5. **52 classic passes have no executed step** (list above). Behavioural, and
   several have visible consequences: no heart crystals (`Life Crystals`) so a
   generated world's 100 HP cap is unreachable legitimately, no statues/traps/
   piles, no starter NPC (`Guide`), no `Settle Liquids` so water is static, no
   `Surface Ore and Stone`, no herbs/flowers/mushrooms decoration, no `Wall
   Variety`. Admitted: partially — ROADMAP M5-GEN-1 says "the remaining vanilla
   passes = their M5 tasks", PORT_MAP.md:34 repeats it, but no task id covers
   ~20 of the 52 (`Quick Cleanup`, `Tile Cleanup`, `Final Cleanup`, `Wall Variety`,
   `Cave Walls`, `Grass Wall`, `Guide`, `Herbs`, `Dye Plants`, `Flowers`,
   `Mushrooms`, `Moss Grass`, `Surface Ore and Stone` …).
6. **Skyblock and dual-dungeon gates are declarative only.** `VANILLA_CHAIN`'s
   `gate` field is `#[allow(dead_code)]` and `run` never tests it, so
   `SeedOptionId::Skyblock` / `dualDungeons` produce a classic-shaped world.
   Severity: behavioural. Admitted: the `dualDungeons` half is in M5-SEED-1/2's
   deferred list; the skyblock half is not listed anywhere.
7. **`Settle Liquids` has no equivalent, so the C#'s post-pass liquid invariant
   cannot be honoured.** `WorldGenerator.cs:508` asserts `Liquid.numLiquid == 0`
   after every pass; the port has no liquid-flux simulation, so generated water is
   whatever the passes wrote. Severity: behavioural (and a structural blocker for
   `Waterfalls`/`Shimmer`). Admitted: no.
8. **Spawn point is a compact approximation.** `step_world_tail` takes
   `spawn_center` and `heights[center]-6`; the C# `SpawnPoint`
   (`WorldGen.cs:19460-19588`) scans upward from the surface centre with flood
   checks and rejects occupied/liquid cells. Severity: behavioural (occasionally a
   spawn inside terrain). Admitted: the remix arm's doc comment cites the C#, the
   classic deviation is not in the docs.
9. **Terrain formulas are mapped, not ported.** `surface_is_in_space` maps the C#'s
   `num10/num11` fractions (0.17/0.26→0.22) onto the compact model's
   `clamp(80, height-340)` with an explicit `height-340-0.04*height` band, and
   concedes "the ceiling arm does not bind in practice" (`terrain.rs:230-260`); the
   ≥20 recentering is skipped because the compact rock line would trigger it on
   every world (`terrain.rs:54-61`, ANALYSIS "OresAndShinies fidelity"). Severity:
   behavioural/pixel. Admitted: yes, in code comments + the ORES-1 ANALYSIS note.
10. **Hash is not comparable with vanilla.** `world_hash::tile_hash` is the port's
    own field mix because .NET's `TileStruct.GetHashCode` is not reproducible
    (`world_hash.rs:9-13`); the constants therefore only ever prove port-vs-port
    identity. Severity: determinism (self-consistency only). Admitted: yes.
11. **Pinned constants in the docs lag the code.** `world_hash.rs:173-174` pins
    `SEED42_512 = 3075125982` / `DEFAULT_WORLD = 366022756`; ANALYSIS's
    "Cave decorations (M5-STRUCT-5)" section records `3074238937`/`4238218118`
    (pre-M5-DUNGEON + pre-honey-patch), and ROADMAP M5-GEN-1's `3510339375`/
    `3248003836` are the pre-scaffold values. The chain of custody is complete in
    `world_hash.rs`'s comment block (`:64-172`), but three published numbers are
    stale. Severity: cosmetic (documentation). Admitted: implicitly.
12. **A test name lies about its assertion.** `pipeline::the_executed_pipeline_is_59_named_steps`
    asserts `names.len() == 62` (the 62-step growth is documented above it). The
    step count grew from 39→44→50→52→56/61→62 across M5 phases and the name was
    never updated. Severity: cosmetic. Admitted: no.
13. **ROADMAP's status table contradicts its own catalog.** ROADMAP.md:69 marks
    world generation 🟡 with "DUNGEON, TEMPLE, STRUCT-5 … parked LIVING-trees",
    while `living_trees.rs` (1 113 lines, 12 tests) is a faithful port whose
    catalog row (`ROADMAP.md:1571`) says done; M5-DUNGEON/M5-TEMPLE are 🔄/🟡
    partial, not parked. Severity: cosmetic (planning only). Admitted: no.
14. **`StructureMap` protection is absent**, so passes that in vanilla register
    their rects (cave houses, honey patches, dungeon) cannot protect each other;
    the port writes tiles directly and relies on step order. Severity: behavioural
    (rare overlap corruption). Admitted: the honey-patch section names a
    `protected` vec "stand-in" (ANALYSIS M5-BIOME-2). Not stated as a general rule.
15. **`GenVars` coverage is ~25 of ~100 statics** (`pipeline.rs:152-313` vs
    `GenVars.cs`), so any future pass needing e.g. `numOasis`, `shimmerPosition`,
    `DungeonX/Y` or the beach `shellStart*` quartet's full C# set finds no slot.
    Severity: behavioural (future work). Admitted: partially, per-feature.

## Task seeds

- **S — Fix the doc-level lies (30 min).** Rename
  `pipeline::the_executed_pipeline_is_59_named_steps` to `…_is_62_named_steps`;
  refresh ANALYSIS's M5-STRUCT-5 pins and ROADMAP.md:69's world-gen row
  (living trees done, dungeon/temple 🟡 partial); state in ANALYSIS (and one line
  in ROADMAP) that "byte-identical" means *port-internal*. Depends on nothing.
- **S — Make the config view real for one pass (½ day).** Thread
  `pass_config` into `caves.rs::ores_and_shinies` (the
  `OresAndShinies`/`OreRunner` density entries) and delete the corresponding
  hard-coded constants, keeping `the_embedded_copy_is_unmodified` as the guard.
  Proves the plumbing that M5-GEN-1 claims. No dependency.
- **M — Port the cheap missing passes that need no new machinery (2-3 days):**
  `Life Crystals` (`:16805` — the `maxTilesX*0.4e-4` rolls over the cave band),
  `Statues`/`Traps`/`Piles`/`Floating Island Houses` furniture (`Pots…` `:17996`
  already exists as a shape pass, so these extend `mini_biomes.rs`/`structures.rs`),
  `Sunflowers`, `Flowers`, `Mushrooms`, `Herbs`/`Dye Plants` (the `Planting Trees`
  family in `biome_detail.rs`). Each needs its `EXECUTED_PIPELINE` row at the
  `WorldGen.cs` anchor plus a positional test and a hash re-pin. Blocked by the
  re-pin discipline: land them one per branch so `world_hash.rs` constants stay
  reviewable.
- **M — Skyblock gate (1-2 days).** Read `VANILLA_CHAIN`'s `gate` in `run` and
  implement the skyblock shape (small island + the `Jungle` skyblock pass at
  `WorldGen.cs:11509`). Depends on the gate being honoured, which is also the
  precondition for `dualDungeons`.
- **M — Liquid settlement pass (3-4 days).** Port a minimal `Liquid` update that
  satisfies `WorldGenerator.cs:508`'s invariant, then wire the two `Settle Liquids`
  passes, `Waterfalls` and, with them, `Shimmer`. Depends on the existing
  `liquid.rs` module; enables the biggest behavioural cluster of the 52.
- **L — Verifiable world parity harness (week).** Re-implement
  `WorldGenSnapshot`/`WorldManifest` (`WorldGenSnapshot.cs`, `WorldManifest.cs`)
  end-to-end — snapshot tiles after each pass, compare against a recorded manifest,
  and expose them in a `--worldgen-audit` dev flag replacing the `log::debug!`
  trace. This is the only way any future claim of "same world" can be *proved*
  rather than asserted by pinned hashes.
- **L — Remaining faithful terrain (week).** Replace the FBM surface model with
  `TerrainPass`'s actual height/rock lines (including the ≥20 recentering,
  `TerrainPass.cs:217-228`) once the compact bands it replaced (caves/ores) are
  re-anchored — the current skip (`terrain.rs:54-61`) is load-bearing for them.
- **Cross-cutting — assign the 20 orphan passes** (`Quick Cleanup`, `Tile
  Cleanup`, `Final Cleanup`, `Wall Variety`, `Cave Walls`, `Grass Wall`, `Guide`,
  `Herbs`, `Dye Plants`, `Flowers`, `Mushrooms`, `Moss Grass`, `Surface Ore and
  Stone`, `Place Fallen Log`, `Remove Broken Traps`, `Silt`, `Slush`, `Clay`,
  `Webs`, `Mountain Caves`) to real task ids in ROADMAP so the "remaining passes =
  their M5 tasks" claim stops being a 20-row hole.
