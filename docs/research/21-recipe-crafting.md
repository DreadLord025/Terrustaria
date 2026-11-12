# 21 — Recipes / crafting

> Wave 2 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP

## Scope

C# side is one god file: `decompiled/Terraria/Recipe.cs` (16 784 lines) plus
`decompiled/Terraria/RecipeGroup.cs` (112 lines) and the request/grant half in
`decompiled/Terraria/GameContent/CraftingRequests.cs` (369 lines). The UI peers are
`Terraria/GameContent/UI/NewCraftingUI.cs` (filters, grid, guide mode) and
`Terraria/UI/CraftingUI.cs` (legacy panel + the `RecipeFilterHack` bridge). Rust side is
`crates/tr-app/src/craft.rs` (1 296 lines — the table types, the availability machine, the
consume/grant), the generated `crates/tr-app/src/recipes_gen.rs` (19 513 lines), and
`crates/tr-app/src/ui/crafting.rs` (1 003 lines — pure chrome). Wiring:
`crates/tr-app/src/app/frame.rs:1298-1334` (per-frame availability scan) and
`crates/tr-app/src/app/clicks.rs:356-409` (toggle, hit-test, craft, grant, sound, achievement).

## Key entities

- `Terraria/Recipe.cs::SetupRecipes` (`:604`) — the 15k-line recipe-authoring block; flat
  `currentRecipe` assignments plus `for`/`int[,]` ranges, the 35 `Add*Furniture` helpers
  (`:15260-16630`), `AddCritterStatueRecipe` (`:16630`) and the two reverse-recipe passes
  (`:16648`/`:16672`).
- `Terraria/Recipe.cs::AddRecipe` (`:16723-16736`) — the commit: `alchemy = true` iff
  `requiredTile == 13` (`:16725-16728`), `TileUsedInRecipes[tile] = true`, `currentRecipe =
  new Recipe()`, `numRecipes++`.
- `Terraria/Recipe.cs::Recipe` (`:58-112`) — `createItem`, `requiredItem[15]`,
  `requiredTile = -1`, `acceptedGroups[15]`, `requiredItemQuickLookup[15]`, the environment
  bools (`needHoney/Water/Lava/TorchGodsFavor/alchemy/needSnowBiome/needGraveyardBiome/
  needMechdusa`), `notDecraftable`, `crimson`, `corruption`. Caps: `maxRequirements = 15`,
  `maxRecipes = 3700` (`:16776-16777`).
- `Terraria/Recipe.cs::UpdateRecipeList` (`:230-259`) — the availability filter: guide-item
  mode, else filter-hack × `PlayerMeetsEnvironmentConditions` (`:322-358`) ×
  `CollectedEnoughItemsToCraft` (`:275-311`).
- `Terraria/Recipe.cs::CreateRequiredItemQuickLookups` (`:15162-15191`) — folds each
  `requiredItem[j]` into the group fake id when the authored item is a member.
- `Terraria/RecipeGroup.cs::RecipeGroup` (`:9-27`, `:35-112`) — `ValidItems`/`Items`,
  `RegisteredId` (registration order), `FakeItemIdOffset = 1000000` (`:108`),
  `CountUsableItems` (`:78-89`), `DecraftItemId` = lowest-value member (`:57-60`).
- `crates/tr-app/src/craft.rs::craft` — `Recipe` (`:39-64`), `Requirement` (`:86-93`),
  `RecipeGroups` consts (`:101-136`), `GROUPS` (`:151-241`, 34 rows), `group` (`:243`),
  `Requirement::matches` (`:256-263`), `TILE_COUNTS_AS` (`:269-279`), `satisfies_station`
  (`:287-298`), `Adjacent`/`scan_adjacent` (`:302-358`), `ingredient_discount` (`:380-384`),
  `chests_in_range` (`:401-448`), `collect_owned` (`:451-470`), `meets_environment`
  (`:476-486`), `craftable` (`:491-508`), `available` (`:513-527`), `can_craft` (`:591`),
  `try_craft` (`:607-649`), `RECIPES` (`:656`).
- `tools/gen_recipes.py` (627 lines) — the generator that emits `recipes_gen.rs`
  (`--check` mode verifies drift).

## Dependencies

- Calls into: `registry` (every create/required id is validated against `itemdata`),
  `player::{Player, WIDTH, HEIGHT}`, `item::{ItemSlot, INV}`, `chest::WorldChest`,
  `tr_core::{World, tile::LiquidKind}`.
- Called by: `app/frame.rs:1309-1334` and `app/clicks.rs:362-409` only (plus
  `world_sim/editing.rs:5336` constructing a `WorldChest`).
- Shared data structures: `Recipe`, `Requirement`, `RecipeGroupDef`, `Adjacent`, `Zones`
  (`:362-366`), the `RECIPES` static spliced with `include!("recipes_gen.rs")`.

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| Recipe row table | PARTIAL | `craft.rs::RECIPES` + `recipes_gen.rs` | COUNT PARITY **FAILS**: Rust `2764` rows (indices `// 0:` … `// 2763:`) vs C# `2763` literal `Recipe.AddRecipe();` sites + `22` `AddCritterStatueRecipe` calls (`:9630-9650`) + `284` reverse rows = ~3069 expected rows. Spot-verified missing create ids: `4473/4474/4475`, `4846/4964/5133`, `5522/5066/5471/5467/5468`, `5286/5287`, `3391/3392/3393`, `4390/4391`, `4992` — all present in `itemdata.rs`, so the generator dropped rows, not unknown ids. |
| Generator + parity check | PARTIAL | `tools/gen_recipes.py` | Handles the documented forms, but `--check` is not enforced in CI; the row-count drift is invisible. |
| `Recipe` struct fields | PORTED | `craft.rs::Recipe` | All env flags + `not_decraftable`; ` Stations` is `&[u16]` (C# has a single `int requiredTile`), `alchemy` pre-resolved. |
| Ingredient entry + group bind | PORTED | `craft.rs::Requirement` | `RequireGroup` → `Requirement::group`, membership-bound (`CreateRequiredItemQuickLookups`, `:15162-15191`). |
| Group expansion when counting | PORTED | `craftable` (`:499-505`) | Sums members directly = `AddFakeCountsForItemGroups` (`:409-416`) + `CountUsableItems`. |
| RecipeGroup registry (34) | PORTED | `craft.rs::GROUPS` (`:151-241`) | Registration order = id, icon = first member; `test_dw_...`/`the_group_table...` assert 34. |
| Station check (tile + inheritance) | PORTED | `satisfies_station` (`:287-298`) | 9 `TileCountsAs` pairs, chain walk = `Player.SetAdjTile` recursion (`Player.cs:35643-35659`). |
| Adjacency scan (reach, liquids) | PORTED | `scan_adjacent` (`:321-358`) | `REACH_X=5`/`REACH_Y=3` = `TileReachCheckSettings.Simple` + `Player.tileRangeX/Y` defaults (`Player.cs:57113-57116`); liquid `>200` by kind; `CountsAsWaterForCrafting {172,207}`. |
| Environment flags | PARTIAL | `meets_environment` (`:476-486`) | Missing `needTorchGodsFavor` + `needMechdusa` (no port systems — documented skip). |
| Availability filter | PARTIAL | `craft.rs::available` (`:513`) | Env × enough-ingredients only; **no `RecipeFilter` tile/water-source tabs** (`NewCraftingUI.cs:78-178`) and **no guide-item mode** (`Recipe.cs:235-241`, `CollectGuideRecipes` `:441`). |
| Craft consume + grant | PARTIAL | `try_craft` (`:607-649`) | Inventory-then-chest order matches `CraftingRequests.Consume` (`:218-290`); **single craft only** — no `qty` loop (`:97-118`), no `quickCraft`/remote-craft queuing, no `CursorHasSpaceToCraftRecipe` (`:102`), no `Prefix(-1)` on `stack<=1` (`:126-129`), no `OnCreated`. |
| Alchemy 1/3 discount | PORTED | `ingredient_discount` (`:380-384`) | Per-unit `rand.Next(3)==0`; rolls the port's `craft_rng` instead of the shared `Main.rand` (documented). |
| Reverse platform/wall recipes | PORTED | `recipes_gen.rs` (284 rows) | Generator re-derives them (`gen_recipes.py:515-537`); depends on the row table, so the row loss above propagates. |

## Tests

`craft.rs` has 23 `#[test]` functions — `the_progression_batch_registers_verbatim` (`:707`),
`test_dw_1_1_table_integrity` (`:753`), `the_group_table_is_the_setuprecipegroups_order`
(`:789`), `requirement_matches_any_group_member` (`:806`),
`station_inheritance_walks_the_counts_as_chain` (`:819`),
`test_dw_1_2_workbench_reference_recipe` (`:833`), `test_dw_2_1_can_craft_cases` (`:846`),
`the_environment_gates_the_liquid_and_biome_recipes` (`:872`),
`scan_adjacent_reads_liquids_and_the_water_tiles` (`:913`),
`the_chest_range_scan_finds_tile_chests_in_600px` (`:945`),
`the_chest_ingredients_count_and_drain` (`:969`),
`draining_the_last_stocked_chest_slot_trims_the_implicit_air_tail` (`:1003`),
`a_stale_entry_with_a_dead_tile_is_filtered_from_the_range_scan` (`:1031`),
`the_toggle_off_ignores_the_chests` (`:1052`),
`available_is_the_update_recipe_list_composition` (`:1071`),
`addrecipe_sets_alchemy_on_the_tile_13_recipes` (`:1095`),
`the_alchemy_table_discounts_a_third_of_the_units` (`:1109`),
`test_dw_2_2_try_craft_consumes_and_returns` (`:1167`),
`test_dw_2_3_scan_adjacent_semantics` (`:1183`),
`test_dw_2_4_station_gate_and_dirty_inputs` (`:1197`),
`test_dw_2_5_full_inventory_still_crafts` (`:1220`),
`furniture_recipe_crafts_and_consumes` (`:1237`),
`the_utility_batch_chains_the_stations` (`:1262`). `ui/crafting.rs` has 10
(`layout`/`hit_test`/filter-strip geometry — chrome only). Note the integrity test only
asserts `seen.len() >= 2700` (`:785`), which is exactly what let the ~300 missing rows pass.

## Deviations & risks

1. **Missing recipe rows (behavioural, HIGH)** — `recipes_gen.rs` holds 2764 rows while the C#
   `SetupRecipes` produces roughly 2763 authored sites + 22 critter statues + 284 reverse rows
   (~3069). Hand-verified absent create ids include 4473/4474/4475, 4846, 4964, 5133, 5066,
   5522, 5286/5287 and the 3391/3392/3393 graveyard rows — all of which exist in the port's
   `itemdata.rs`, so this is generator loss, not unknown ids. Not admitted anywhere; the
   ROADMAP M3-CRAFT-6 row claims "2764 rows in the C# registration order". Every downstream
   claim about crafting completeness inherits this.
2. **Weak parity assertion (behavioural, HIGH)** — `test_dw_1_1_table_integrity` pins only
   `>= 2700` (`:785`); there is no exact-count assertion and no `gen_recipes.py --check` in the
   test gate, so drift is silent.
3. **Station-tab filters missing (behavioural, MEDIUM)** — C# `RecipeFilter` /
   `TileBasedRecipeFilter` / `CraftStationRecipeFilter` / `WaterSourceRecipeFilter`
   (`NewCraftingUI.cs:78-178`) gate `UpdateRecipeList` through
   `CraftingUI.RecipeFilterHack` (`Recipe.cs:244`, `CraftingUI.cs:112`). The Rust panel draws
   the 11-tab strip (`ui/crafting.rs:589-736`) but no filter ever narrows `available()`.
4. **Guide mode missing (behavioural, MEDIUM)** — `Main.guideItem` +
   `CollectGuideRecipes` (`Recipe.cs:235-241`/`:441`) and `InGuideCraftMenu`
   (`NewCraftingUI.cs:439-446`) have no Rust equivalent; `ui/mod.rs:863` hardcodes `false`.
5. **Single-craft only (behavioural, MEDIUM)** — `try_craft` returns one `(kind, count)`
   (`:648`); C# `CraftItem` loops `qty` with a re-check per iteration (`CraftingRequests.cs:102`)
   and supports `quickCraft` / remote-craft queuing (`:142-173`).
6. **Craft result post-processing missing (behavioural, LOW-MED)** — `CreateResult`
   (`:122-131`) rolls `item.Prefix(-1)` when `stack <= 1` and runs `OnCreated`; the Rust grant
   is a plain stack merge (`clicks.rs:400-402`) with `max_stack` clamping.
7. **Range extensions ignored (behavioural, LOW)** — `scan_adjacent` hardcodes reach 5/3. The
   C# widens it via Artisan Bread (`Player.cs:35661`'s `ateArtisanBread ? 4 : 0`), the
   Extendo Grip/Ruler (`:13042-13043`), the Architect Gizmo Pack (`:14956-14957`) and the
   Journey far-placement power (`:18817-18825`).
8. **Chest enumeration order (determinism, LOW)** — C# walks `Main.chest[]` instance order
   (`Recipe.cs:395`); the port scans tile kinds 21/341 in raster order. Documented in ANALYSIS.
9. **Alchemy RNG stream (determinism, LOW)** — the port's `craft_rng` (`next_rand3`, `:370`)
   replaces the shared `Main.rand`; vanilla's draws interleave with every other consumer.
10. **DONE-row re-verification** — `docs/ROADMAP.md:71`, `:527-559`, `:1364-1369` mark
    M3-CRAFT-recipes / 2 / 3 / 4 / 5 / 6 DONE. The static claims are TRUE for the group table
    (34 groups verbatim), the env flags, the station inheritance, the alchemy discount and the
    consume order; the "full table" claim in M3-CRAFT-6 (`:1368`) is **PARTIAL**, not DONE,
    because of findings 1 and 2.

## Task seeds

- **L — Regenerate + reconcile the recipe table.** Re-run `python tools/gen_recipes.py` in the
  primary checkout, diff its `skipped`/`warnings` output against the emitted rows, and fix the
  parsing losses (start with the 4473/4846/5133/3391 anchors). Then replace the `>= 2700`
  assertion with the exact C# count and add a `--check` step to the test gate.
  Depends on: a fresh `decompiled/` sync (the graph and the decompile must match).
- **M — Station-tab + guide filters.** Port `NewCraftingUI.RecipeFilter` (the four concrete
  filters) into `craft::available` as an optional predicate and wire the 11-tab strip +
  `RecipeFilterHack` semantics; then the guide-item mode as a second source of rows.
  Depends on: the recipe-table task (a correct row set first).
- **S — Multi-craft / quickCraft.** Extend `try_craft` to a `qty` loop with the per-iteration
  `CollectedEnoughItemsToCraft` + cursor-space re-check.
- **S — Craft result post-processing.** Apply `Prefix(-1)` for `stack <= 1` results and
  `OnCreated`; gate on M3-PREFIX landing.
- **S — Reach parity.** Make `REACH_X/Y` derive from the player's modifiers (Extendo/Architect/
  Artisan Bread/Journey) instead of the hardcoded 5/3.
