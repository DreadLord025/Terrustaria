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
- [ ] 41 Flowers
- [ ] 42 Mushrooms
- [ ] 43 Random Gems
- [ ] 44 Moss Grass
- [ ] 45 Larva
