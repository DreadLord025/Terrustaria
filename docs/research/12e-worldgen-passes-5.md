# 12e — WorldGen passes, lane 5 (task/W4-d105-worldgen-passes-5)

> Wave 4 · 2026-11-20 · code-only mode (no cargo/tests on this branch) ·
> source: `docs/research/12-worldgen.md` (the 52-pass gap list + the
> `8a856d94` lane split) re-derived against the current `pipeline.rs`
> at base `a725acb0`.

## Range claim (documented early, before any port)

The six `worldgen-passes-N` lanes (d101…d106) split the **classic (ungated)
unexecuted passes in C# `AddPasses` order** (the `VANILLA_CHAIN` anchors).
The published lane-split doc (`8a856d94`, in `12-worldgen.md`) numbers
**lane 5 = #37-45**; d104's own pushed doc (`2d9cfcbb`) ends its range at
Spawn Point `:19460` and hands "Grass Wall `:19589` to lane 5", while this
lane's dispatch starts at Guide — d106's pushed doc (`89a96c19`) records
that orphan and **adopts Grass Wall itself**. This lane's nine passes are
therefore exactly the published #37-45, disjoint from every sibling's
stated range.

### Lane 5 — the nine passes (C# order)

| # | Pass (`VANILLA_CHAIN` name) | `GenPassNameID` | C# anchor | Rust home (planned) |
| --- | --- | --- | --- | --- |
| 37 | Guide | SpawnStarterNPCs | `WorldGen.cs:19685` | new `starter_npcs.rs` (gen-time NPC ledger) |
| 38 | Sunflowers | SunflowersPart2 | `WorldGen.cs:19898` | `surface.rs` |
| 39 | Herbs | AlchemyHerbs | `WorldGen.cs:19964` | `surface.rs` (`PlantAlch` :45901) |
| 40 | Dye Plants | DyePlants | `WorldGen.cs:19981` | `surface.rs` (`plantDye` :71286 + `MatureTheHerbPlants` :45748 + `GrowGlowTulips` :45622 + `PlaceChilletEggs` :45652) |
| 41 | Flowers | Flowers | `WorldGen.cs:20447` | `surface.rs` |
| 42 | Mushrooms | Mushrooms | `WorldGen.cs:20599` | `surface.rs` |
| 43 | Random Gems | ExposedGemsUnderground | `WorldGen.cs:20729` | `spider_gem_moss.rs` |
| 44 | Moss Grass | LongMoss | `WorldGen.cs:20769` | `spider_gem_moss.rs` |
| 45 | Larva | BeeLarvaInBeehives | `WorldGen.cs:20868` | `structures.rs` (the hive ledger) |

Neighbours for context (NOT this lane): #36 Grass Wall `:19589` (adopted
by d106), Planting Trees `:19940` / Webs And Honey `:19995` / Weeds
`:20064` / Glowing Mushrooms and Jungle Plants `:20122` / Jungle Plants
`:20165` / Vines `:20193` / Gems In Ice Biome `:20697` / Muds Walls In
Jungle `:20813` are already executed steps; #46 Settle Liquids Again
`:21246` starts lane 6.

## Discipline

- Verbatim ports (golden rule): same structure, maths, draw order; citation
  comment per block; readable Rust names; C# read in ≤100-line windows.
- Each pass gets its `EXECUTED_PIPELINE` row at the vanilla slot + a
  positional witness; per-pass `pass_rng("<vanilla name>")` stream so the
  insertion shifts no other step's ledger (the genRand current convention).
- **Hash re-pins are DEFERRED to the merge gate** (code-only mode — no
  compiler on this branch, same discipline as S1-K-2 wave B / S1-P-i33):
  `world_hash.rs` gains a per-pass stale-by-design note; the `a == b`
  determinism halves of both pin tests stay untouched and must stay green
  at the gate.
- Commit + push per pass; docs + `anchors.txt` row per pass.

## Status

- [x] Range claim documented (this doc)
- [x] 37 Guide — `starter_npcs.rs::pass_starter_npcs` (the gen-time NPC
      ledger `World.gen_npcs`, the App materialises at
      `enter_loaded_world`); step `guide` before `trees` (the port's tree
      family stands in for the unexecuted Spawn Point / Grass Wall /
      Sunflowers slots); witness
      `w4_d105_the_guide_step_sits_before_the_trees` + 9 unit tests.
      No tile writes → neither hash pin moves on this commit.
- [x] 38 Sunflowers — `surface.rs::pass_sunflowers` + `place_sunflower`
      (the band attempts `:19898-19939` + the 2×4 `PlaceSunflower`
      `:53636-53686`); step `sunflowers` between `guide` and `trees`;
      witness `w4_d105_the_sunflowers_step_sits_between_guide_and_trees`
      + 5 unit tests; the `Skyblock.denySomeGeneration` gate helper lands
      in `pipeline.rs`. Hash pins stale-by-design (tile 27 writes).
- [x] 39 Herbs — `surface.rs::pass_herbs` + `plant_alch` +
      `place_suitable_herb_here` (`:19964-19980` + `PlantAlch`
      `:45901-45938` + `PlaceSuitableHerbHere` `:45940-45970`, reusing
      `edit::place_alch`); step `herbs` between `trees` and
      `surface flora`; witness
      `w4_d105_the_herbs_step_sits_after_the_trees` + 3 unit tests.
- [x] 40 Dye Plants — `surface.rs::pass_dye_plants` + `plant_dye` +
      `place_dye` + `mature_the_herb_plants` + `grow_glow_tulips` +
      `place_chillet_eggs` (`:19981-19994` + `plantDye` `:71286-71455` +
      `PlaceDye` `:53988-54064` + `MatureTheHerbPlants` `:45748-45775` +
      `GrowGlowTulips` `:45622-45650` + `PlaceChilletEggs` `:45652-45678`
      + `Place2x2` `:51330-51423`); step `dye plants` between `herbs` and
      `surface flora`; witness
      `w4_d105_the_dye_plants_step_sits_after_the_herbs` + 5 unit tests;
      the 227/656/752 frame-important boot statics land.
- [x] 41 Flowers — `surface.rs::pass_flowers` + `flower_cell` +
      `place_flower` + `is_fit_to_place_flower_in` (`:20447-20598` + the
      `PlaceTile`-3 arm `:59507-59580` + `IsFitToPlaceFlowerIn`
      `:58231-58251`); the `GenVars.logX/logY` seam lands (the fallen-log
      pass's product, −1 reset); step `flowers` between `vines` and
      `muds walls in jungle` (DW-5.3's adjacency re-pinned +2); witness
      `w4_d105_the_flowers_step_sits_after_the_vines` + 3 unit tests.
- [x] 42 Mushrooms — `surface.rs::pass_mushrooms` + `mushroom_cell`
      (`:20599-20696`): the patch attempts turn the grass tufts (3) /
      sunflowers (24) into mushrooms (frameX 144) and the glowing
      mushrooms (201) tall (frameX 270); step `mushrooms` between
      `flowers` and `muds walls in jungle` (DW-5.3's adjacency re-pinned
      +3); witness `w4_d105_the_mushrooms_step_sits_after_the_flowers`
      + 2 unit tests.
- [x] 43 Random Gems — `spider_gem_moss.rs::pass_random_gems` +
      `place_exposed_gem` + `check_and_adjust_multi_directional` +
      `valid_anchor` + the three `SolidTileAllow*Slope` helpers
      (`:20729-20768` + the `PlaceTile`-178 arm `:59691-59701` +
      `CheckAndAdjustMultiDirectionalTile` `:60314-60366` +
      `ValidAnchorForMultiDirectionalTile` `:60297-60312` +
      `SolidTileAllowTopSlope/LeftSlope/RightSlope`
      `:70320-70378`); the 178/184 frame-important boot statics land;
      step `random gems` between `mushrooms` and `muds walls in jungle`
      (DW-5.3's adjacency re-pinned +4); witness
      `w4_d105_the_random_gems_step_sits_after_the_mushrooms` + 3 unit
      tests.
- [x] 44 Moss Grass — `spider_gem_moss.rs::pass_moss_grass` +
      `place_long_moss` (`:20769-20812` + the `PlaceTile`-184 arm
      `:59703-59720`, the two independent moss/moss-brick gates each
      drawing a `Next(3)` frame variant); step `moss grass` between
      `random gems` and `muds walls in jungle` (DW-5.3's adjacency
      re-pinned +5); witness
      `w4_d105_the_moss_grass_step_sits_after_the_random_gems` + 3 unit
      tests.
- [x] 45 Larva — `structures.rs::pass_larva` + `place_larva`
      (`:20868-20900` + the `Place3x3`-231 arm `:53534-53633`); the hive
      pass (`pass_beehives`) now publishes the larva anchors into the
      `GenVars.larva` ledger (`HiveBiome.CreateStandForLarva`
      `HiveBiome.cs:357-365`, the 5-border clamp + the 100-entry cap)
      instead of placing them inline — the Larva pass owns the 3×4 clear
      + the 3×3 larva object; step `larva` between `muds walls in jungle`
      and `floating islands` (DW-5.3's adjacency re-pinned); the
      dual-dungeons prune now skips a real step; witness
      `w4_d105_the_larva_step_sits_after_the_muds_walls` + 3 unit tests.

**Lane 5 complete: all nine passes (#37-45) landed, one per commit.**

## Doc-sweep phase (lane 39, 2026-11-21)

The six residuals over the landed passes were re-censused against the C#
source; the full census + verdicts live in
`docs/research/129-doc-sweep-wg-37.md`. Three REAL gaps ported (one commit
each), the rest verified-present or cross-lane:

- [x] Herb growth tick — `growth/herbs.rs::grow_alch`: the generic 1/50
      bloom gate's frame-108 condition was INVERTED
      (`WorldGen.cs:45798-45811`) + the 1/30 fireblossom maturity tail
      (`:45890-45898`) was missing; 2 witnesses (`35ccf38ea`).
- [x] Runtime dye twin — `growth/decorations.rs::plant_dye`: the remix
      bypass halves (`:71298` / `:71378`) + the lava/honey-only exotic
      liquid veto (`:71318-71321`, shimmer passes); 2 witnesses
      (`52978903f`). The gen-time twin already carried all three.
- [x] Sunflower happiness — the `player.sunflower` flag now feeds
      `SpawnFlags.sunflower` in the live spawn context
      (`world_sim/npcs.rs`), arming `NPC.GetSpawnRate`'s `:712-716`
      modifier; 1 witness (`7eb486f07`).
- [x] Guide starter-NPC spawn — VERIFIED PRESENT (full arm census +
      `GetAdjustedFloorPosition`/`Scan3By3` + the name-category table);
      the copper tools are the player start inventory (already ported),
      not the guide pass.
- [x] GlowTulips glow — VERIFIED PRESENT (`TileLightScanner.cs:389-393`
      `[0.2, 0.55, 0.5]` == `scanner.rs:751`); cross-note: outside the
      g18/e8 Main.cs AddLight scope (a scanner seed, already landed).
- [x] ChilletEggs — gen placement + mount data + recipes PRESENT; the
      egg-tile-break drop (Check2x2-752) and the bag-open
      (OpenChilletEgg) are real gaps but CROSS-LANE (tile-framing / UI
      container-opening), recorded not ported. **RETIRE correction**:
      the bag-open half is CLAIMED + LANDED on W4-f35
      (`misc_bags::open_chillet_egg` `0fd244e79`, dispatch `00a5b27db`);
      the tile-break drop stays UNCLAIMED — a task-seed for the g34
      Check\* continuation (752→6142 + the sibling 751→5667), NOT g56's
      void lane. Full census: research/129 RETIRE §R2.

No generation-time tile writes in any of the three ports — neither
`world_hash.rs` pin moves; the lane-5 gen pins stay stale-by-design.

## RETIRE phase — the wg-lane5-family declaration (2026-11-21, SEALED)

Branch `task/W4-d105-worldgen-passes-5`, base `a725acb04`, 20 commits
(19 + this RETIRE commit), code-only throughout (witnesses WRITTEN NOT RUN, no cargo/rustfmt/
graphify on the branch). This is the lane's complete family record for
the merge gate.

### The commit family

| phase | commits | scope |
| --- | --- | --- |
| range claim | `5bb4116ee` | the #37-45 range doc (this file's head) |
| #37 Guide | `c499a7670` | `starter_npcs.rs` + World.gen_npcs + App materialisation |
| #38 Sunflowers | `4350c46bc` | `surface.rs::pass_sunflowers` + the skyblock gate helper |
| #39 Herbs | `f42727bfa` | `surface.rs::pass_herbs` (PlantAlch via edit::place_alch) |
| #40 Dye Plants | `dd43734a6` | `surface.rs` plantDye/PlaceDye/MatureTheHerbPlants/GrowGlowTulips/PlaceChilletEggs |
| #41 Flowers | `3980b799d` | `surface.rs::pass_flowers` + GenVars.log_x/log_y |
| #42 Mushrooms | `03efd7bcc` | `surface.rs::pass_mushrooms` |
| #43 Random Gems | `9f7b1a248` | `spider_gem_moss.rs::pass_random_gems` + 178/184 boot statics |
| #44 Moss Grass | `d590d99a8` | `spider_gem_moss.rs::pass_moss_grass` |
| #45 Larva | `22a0026ce` | `structures.rs::pass_larva` (the hive ledger seam); LANE 9/9 |
| witness re-pins | `93cdd3252` | guide/sunflowers/herbs/dye adjacency pins |
| #40 faithfulness | `96f65cd60` | dye_nearby hanging-tail counts ANY 227 (`:71445`) |
| test robustness | `d089699d5`, `15c58b503`, `241ffa394` | seed sweeps, MicroSeedFlags::from_sum, genvars_defaults |
| doc-sweep ports | `35ccf38ea`, `52978903f`, `7eb486f07` | herb gate+tail / dye remix+veto / sunflower SpawnFlags feed |
| doc-sweep census | `2983d86c8` | research/129 + this file's sweep section + 3 anchors |
| RETIRE | (this commit) | research/129 SEALED + this declaration |

### Owned files (the lane's write-set vs base)

- `crates/tr-core/src/worldgen/`: `starter_npcs.rs` (new), `surface.rs`,
  `spider_gem_moss.rs`, `structures.rs`, `pipeline.rs` (the nine step
  insertions + GenVars fields), `mod.rs`, `progress.rs`, `mini_biomes.rs`,
  `world_hash.rs` (stale-by-design comments only).
- `crates/tr-core/src/`: `world.rs` (gen_npcs + GenVars-facing fields),
  `tileobject.rs` (the 227/656/752 registry rows),
  `growth/herbs.rs` + `growth/decorations.rs` (the sweep's runtime fixes).
- `crates/tr-app/src/`: `app/mod.rs` (starter-NPC materialisation + name
  resolution), `spawner.rs` (the sunflower witness),
  `world_sim/npcs.rs` (the one-line SpawnFlags.sunflower feed).
- `anchors.txt` (+13 rows), docs (this file, research/129, ROADMAP row).

### Co-lane boundaries (retire census, tips read 2026-11-21)

- **g42** (`fdedf9776`): ZERO herbs.rs hunks — d105's grow_alch is the
  keep-one; its dispatcher D6/F1 actuated-wall-arm structure composes
  over the fix. decorations.rs overlap is one `pub(super)` export line
  vs the dye hunks — disjoint. world_hash.rs :238 comment-block conflict
  resolves KEEP-BOTH (research/129 §R1).
- **g58** (`2ff3b7062`): shop-mood latch system only; no spawner/player/
  SpawnFlags-literal overlap — d105's sunflower feed is the keep-one
  (§R3). **f12** (`22ce2a41f`): owns the calmed (106) field + wiring;
  its live SpawnFlags.calmed feed is not yet landed — d105 keeps calmed
  constants-off, the merger keeps both feed lines (§R3).
- **g34** (`cf85a1cf9`): owns the Check\* break-drop family incl.
  check_2x2 — the 752→6142 drop arm is its continuation's task-seed
  (unclaimed, §R2). **g56** (`3ab631923`): void-vault UI only — owns
  NOTHING of the Chillet chain. **f35** (`1abae9ade`): owns the bag-open
  half — OpenChilletEgg landed there (§R2).
- d102/d106 (gen-pass co-lanes): ranges #10-18 / #46+ — disjoint by the
  range claim above; neither claims the runtime GrowAlch/plantDye twins.

### Hash-pin state

Both `world_hash.rs` pins (`SEED42_512`, `DEFAULT_WORLD`) are
stale-by-design from #38 on (tile writes: 27/82/227/656/752/3/178/184/
231 + the herb maturity writes); the re-pin is DEFERRED to the merge
gate. The `a == b` determinism halves are untouched. The three
doc-sweep ports are runtime-only and move no pin. Guide (#37) writes no
tiles (the NPC ledger is not hashed).
