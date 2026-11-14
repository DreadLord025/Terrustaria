# 21 — Recipes / crafting

> Wave 2 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/ANALYSIS/ATLAS

## Scope

The crafting subsystem in C# is three files: `decompiled/Terraria/Recipe.cs` (the recipe
record, the `SetupRecipes` authoring block, the group registry, the quick-lookup fold and
the available-recipe filter), `decompiled/Terraria/RecipeGroup.cs` (the group type, 112
lines) and the craft/grant side in `decompiled/Terraria/GameContent/CraftingRequests.cs`
plus `decompiled/Terraria/GameContent/UI/NewCraftingUI.cs` (the filter interface). The
station/liquid/biome context comes from `decompiled/Terraria/Player.cs` (`AdjTiles`,
`SetAdjTile`, `tileRangeX/Y`).

On the Rust side the module is `crates/tr-app/src/craft.rs` (the rule engine and the
group table) + `crates/tr-app/src/recipes_gen.rs` (the GENERATED recipe table, 19 513
lines) + `crates/tr-app/src/ui/crafting.rs` (the panel) + `crates/tr-app/src/app/clicks.rs`
(the craft click), with the generator in `tools/gen_recipes.py` (627 lines). The port
owns the rule engine; the C# table is parsed into it by the generator.

Responsibility: decide *which* recipes a player may craft right now (station reachable,
liquid/biome present, ingredients held — in the inventory and in nearby chests), execute
the craft (consume the ingredients in the C#'s order, grant the result, sound, achievement)
and render the list of craftable recipes in the panel.

## Key entities

**C# — `decompiled/Terraria/Recipe.cs`** (16 668 lines measured; ATLAS quotes 16 784)

- `Terraria/Recipe.cs::Recipe` (13-…) — the recipe record: `createItem`,
  `requiredItem[maxRequirements]`, `requiredTile`, `acceptedGroups[]`,
  `requiredItemQuickLookup[]`, `needHoney/needWater/needLava/needTorchGodsFavor/alchemy/
  needSnowBiome/needGraveyardBiome/needMechdusa`, `notDecraftable`, `crimson`, `corruption`,
  `customShimmerResults`.
- `Terraria/Recipe.cs::RequiredItemEntry` (15-53) — `itemIdOrRecipeGroup` + `stack`;
  `IsRecipeGroup` = `id >= RecipeGroup.FakeItemIdOffset`; `Matches` (36-43).
- `Terraria/Recipe.cs::SetupRecipes` (604) — the authored recipe block: **2 475 textual
  `Recipe.AddRecipe();` sites**, expanding to **2 481 executed** rows after the two `for`
  ranges, the `int[,]` table, the 34 `Add*Furniture` helpers and the critter-statue calls.
- `Terraria/Recipe.cs::SetupRecipeGroups` (497-533) — **34** `new RecipeGroup(...).Register()`
  registrations; the registration order is the group id.
- `Terraria/Recipe.cs::AddRecipe` (16723-16736) — commits `currentRecipe` into
  `Main.recipe[numRecipes]`, sets `alchemy` iff `requiredTile == 13`, flags
  `TileUsedInRecipes[requiredTile]`.
- `Terraria/Recipe.cs::SetIngredients` (16696-16716), `SetCraftingStation` (16718-16721),
  `RequireGroup` (143-153) — the three authoring primitives the generator parses.
- `Terraria/Recipe.cs::CreateRequiredItemQuickLookups` (15162-15191) — the group
  expansion rule: a slot's item is replaced by the group's fake id **only if the authored
  item is a member of an accepted group**.
- `Terraria/Recipe.cs::UpdateRecipeList` (230-259) — the available-recipe filter:
  guide-item mode short-circuits; otherwise `(filterHack == null || Accepts) &&
  PlayerMeetsEnvironmentConditions && CollectedEnoughItemsToCraft`.
- `Terraria/Recipe.cs::PlayerMeetsEnvironmentConditions` (322-358) — `requiredTile` vs
  `player.adjTile[requiredTile]`, then `adjWaterSource/adjHoney/adjLava/ZoneSnow/ZoneGraveyard`.
- `Terraria/Recipe.cs::HowManyTimesCanRecipeBeCrafted` (290-311) /
  `CollectedEnoughItemsToCraft` (275-283) — `min(owned / stack)` over the quick-lookup.
- `Terraria/Recipe.cs::GetIngredientsForOneCraft` (196-212) /
  `GetIngredientCraftingDiscount` (214-228) — the alchemy 1/3 discount off the consumed stack.
- `Terraria/Recipe.cs::SetupTileInheritance` (559-571) — the 9 `TileCountsAs` pairs.
- `Terraria/Recipe.cs::CreateReversePlatformRecipes` (16648-16670) /
  `CreateReverseWallRecipes` (16672-16694) — the derived reverse rows, each inserted right
  after its source with `notDecraftable = true`.
- `Terraria/Recipe.cs::AddCritterStatueRecipe` (16630-16646) — 50 Stone + 5 critter at tile 283.

**C# — elsewhere**

- `Terraria/RecipeGroup.cs::RecipeGroup` (9-111) — `FakeItemIdOffset = 1000000` (108),
  `Register` (67-76), `GetGroupFakeItemId` (91-94), `Contains` (96-99),
  `GetPlaceholderItemType` = `Items[0]` (101-104), `CountUsableItems` (78-89).
- `Terraria/Player.cs::Player.AdjTiles` (35661-35699) — the reach scan +
  `adjWaterSource/adjHoney/adjLava/alchemyTable`; `SetAdjTile` (35643-35659) — the
  `TileCountsAs` recursion; `DefaultTileRangeX/Y = 5/3` (57113-57114).
- `Terraria/GameContent/CraftingRequests.cs::CraftItem` (97-120), `Consume` (222-237),
  `ConsumeItemsFrom` (244-291), `CreateResult` (122-130) — the consume/grant path.
- `Terraria/GameContent/UI/NewCraftingUI.cs::RecipeFilter` (78-171) — the filter interface
  (`CraftStationRecipeFilter`, `WaterSourceRecipeFilter`).

**Rust — `crates/tr-app/src/craft.rs`**

- `craft.rs::Recipe` (39-64) + `BASE` (67-78) — the port's recipe record
  (`create`, `required`, `stations`, 5 env flags, `alchemy`, `not_decraftable`).
- `craft.rs::Requirement` (86-93) + `NO_GROUP` (96) + `Requirement::matches` (256-263) —
  one `requiredItem` slot with its accepted group.
- `craft.rs::RecipeGroupDef` (142-147) + `GROUPS` (151-238) + `group` (239-241) — the
  34-group registry (`id`, `name`, `icon` = C# `Items[0]`, `members` = `ValidItems`).
- `craft.rs::TILE_COUNTS_AS` (269-279) + `satisfies_station` (287-298) — the station chain.
- `craft.rs::Adjacent` (302-314) + `scan_adjacent` (321-358) + `Zones` (363-366) — the
  station/liquid/biome context.
- `craft.rs::collect_owned` (451-470), `meets_environment` (476-486), `craftable` (491-508),
  `available` (513-527), `can_craft` (591-601), `try_craft` (607-649) — the rule engine.
- `craft.rs::consume_kind` (537-555) + `consume_chest` (564-587) — the two drain passes.
- `craft.rs::RECIPES` (656) — `include!("recipes_gen.rs")`.
- `recipes_gen.rs` (GENERATED, 19 513 lines) — **2 764 rows**: 2 480 authored + 284 derived reverse.
- `ui/crafting.rs` (layout/draw/`hit_test`/`draw_filter_strip`) + `app/clicks.rs::
  handle_crafting_clicks` (347-412) — the panel and the craft click.
- `tools/gen_recipes.py` — the generator (parses `Recipe.cs` → emits `recipes_gen.rs`).

## Dependencies

- Calls into: `crates/tr-app/src/itemdata.rs` (the item registry — the generator's
  validation source and its stand-in for the C#'s `createTile/createWall`),
  `tr_core::World` (tile reads), `crate::player` (position/width), the scene metrics
  (`zone_snow`/`zone_graveyard`), `crate::item::ItemSlot` / `EMPTY_SLOT`,
  `crate::chest::WorldChest`, `crate::settings` (`craft_from_nearby_chests`),
  `crate::audio` (the `Grab` grant cue), `crate::achievements` (`notify_craft`).
- Called by: `app/clicks.rs::handle_crafting_clicks` (the craft action),
  `app/frame.rs` (1312, the panel's available list), `app/clicks.rs` (the nearby-chests
  toggle), `ui/crafting.rs` (the cell grid).
- Shared data structures: `ItemSlot`, `WorldChest`, `Player`, `World`; the `RECIPES` slice
  is shared by the rule engine, the panel and the click path.
- Generator-only dependency: `decompiled/Terraria/ID/ItemID.cs` (item names,
  `Sets.TextureCopyLoad`), `decompiled/Terraria/ID/TileID.cs` (tile constants,
  `Sets.Platforms`).

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| The authored recipe block (`SetupRecipes`) | PORTED | `recipes_gen.rs` (GENERATED) + `tools/gen_recipes.py` | **Re-scoped 2026-11-14 (S1-L-11d)**: the C# has **2 895** executed authored sites, not 2 481 — the generator's tail branch broke out of the expansion loop **before** copying the text that precedes `Recipe.CreateReverseWallRecipes()` (`Recipe.cs:15148`), so the 414 rows between the last expansion mark and that call were dropped (the audit's 2 481 was the generator's own truncated output). The branch now copies that text; the table is **3 206 rows = 2 895 authored + 311 reverse**, the `needTorchGodsFavor`/`needMechdusa` rows are ported (3 rows), and the pin in `tools/test_gen_recipes.py` re-taken (2 895 authored). → **2 480 ported rows** (1 skipped: `needTorchGodsFavor`, `Recipe.cs` `needTorchGodsFavor = true` row). The rows are in the C# registration order, so the panel's listing order matches. |
| The derived reverse rows (`CreateReversePlatformRecipes`/`CreateReverseWallRecipes`) | PORTED | `recipes_gen.rs` (311 `reverse` rows) | **Re-pinned 2026-11-14 (S1-L-11)**: the rows now derive from the C# item data (`Item.SetDefaults` createTile/createWall — `SetDefaults1..5`, the `if (type >= A && type <= B)` range blocks, the `DefaultTo*` delegation), not from `itemdata.rs`; the table is **2 791** rows (2 480 + 311), the 27 restored flips are strictly additive. Gated by `gen_recipes.py --check`, `tools/test_gen_recipes.py` and `craft.rs::the_reverse_rows_match_the_csharp_derivation`. See D1/D7 (closed). |
| `Recipe` record fields | PORTED | `craft.rs::Recipe` (39-72) | Ported: `createItem`, `requiredItem`, `requiredTile`, `needWater/needHoney/needLava/needSnowBiome/needGraveyardBiome`, `alchemy`, `notDecraftable`. **S1-L-11d (2026-11-14)**: `needTorchGodsFavor` and `needMechdusa` are ported (the generator emits them; the gates are `player.unlockedBiomeTorches` and `SpecialSeedFeatures.Mechdusa`, `Recipe.cs:349-356`) (no port systems), `customShimmerResults`, `crimson`, `corruption` (decraft/shimmer bookkeeping only). |
| The group registry (`SetupRecipeGroups`) | PORTED | `craft.rs::GROUPS` (151-238) | **34/34** groups, id = registration order, `icon` = C# `Items[0]`, `members` = C# `ValidItems` (e.g. Birds 2015/2016/2017 ↔ `Recipe.cs:499`). Verified by `the_group_table_is_the_setuprecipegroups_order`. |
| The group-expansion rule | PORTED | `craft.rs::Requirement::matches` (256-263) + `gen_recipes.py` (317-343) | C# `CreateRequiredItemQuickLookups` (15162-15191) substitutes the fake id only when the authored item is a member; the generator binds a group to a member slot and leaves an unbound group as the exact item (the Lesion rows) — the same outcome. The C#'s fake-id bucket (`FakeItemIdOffset = 1000000`) is folded per-requirement in Rust; the totals agree (the port's own note, `craft.rs` 446-450). |
| The station check (`requiredTile` + reach) | PORTED | `craft.rs::scan_adjacent` (321-358) + `satisfies_station` (287-298) | **S1-L-11d (2026-11-14)**: the reach is now the C#'s formula (`craft.rs::tile_reach`: the Extendo Grip / the Architect's Gizmo Pack +3/+2, the held Toolbox +1/+1, the Journey mode's Far Placement Range power ×2 +8/+8, the artisan loaf +4/+4, the 20-tile clamp — `Player.cs:18815-18826`/`:13040-13044`/`:14955-14958`/`:35671`, `TileReachCheckSettings.cs:20-54`); before it the reach was the bare X5/Y3. The chain the scan is inclusive of `HX`/`HY`, which is also the C#'s `Left..=Right` once the XNA-exclusive `Rectangle.Right == X + Width` is applied (deviation D2, refuted). The extender / Artisan-Bread boosts remain absent (S1-L-11d). |
| `TileCountsAs` inheritance | PORTED | `craft.rs::TILE_COUNTS_AS` (269-279) | The 9 pairs are verbatim (`Recipe.cs:562-570`), and the C#'s `SetAdjTile` recursion is expressed as a chain walk to the same reachability. |
| The environment flags | PARTIAL (5/5 ported) | `craft.rs::meets_environment` (476-486) | water/honey/lava/snow/graveyard all check the same source (`adjWaterSource`, `adjHoney`, `adjLava`, `ZoneSnow`, `ZoneGraveyard`). 2 C# flags have no port system. |
| The available-recipe filter | PARTIAL | `craft.rs::available` (513-527) | env + enough-ingredients is ported, but the C#'s **`RecipeFilterHack`** (`NewCraftingUI.cs:78-171`) and the **guide-item mode** (`CollectGuideRecipes`) are not; the filter strip in `ui/crafting.rs` is draw-only. |
| The ingredient count (`_ownedItems`) | PORTED | `craft.rs::collect_owned` (451-470) | Inventory + in-range chests, summed per item id; the group fold is done per requirement (equivalent totals). |
| The consume order | PARTIAL | `craft.rs::try_craft` (607-649) | Inventory pass first, then the chests in collection order, slot index order, partial-decrement-then-clear — matches `ConsumeItemsFrom` (244-291). But the inventory capacity is equivalent — **corrected 2026-11-13** (S1-L-11f): `Player.inventory = Item[59]` (`Player.cs:1576`) splits in the port as `inventory[50]` + `equip.coins[4]` + `equip.ammo[4]` + `equip.trash` (the 59th slot, `inventory[58]`'s trash stash at `:5130/:5145`); and the C#’s `CreateResult` (`OnCreated` + `Prefix(-1)`) is not applied. |
| The alchemy discount | PORTED | `craft.rs::ingredient_discount` (`:382-396`) | 1/3 roll per unit while brewing at an alchemy table, applied to the consumed stack while availability counts the full stack — same split as the C#. The RNG stream is the C#'s `UnifiedRandom` generator (`App.craft_rng`, seeded per world — S1-L-11e closed the splitmix stand-in; the shared-stream interleave is the S1-A-11 residual). |
| The nearby-chest ingredients | PORTED | `craft.rs::chests_in_range` (401-444) | 600 px radius (`NearbyChests.cs:14`), tiles 21/341, raster order (the C# iterates `Main.chest[]`'s instance order — the port has no instance registry; documented in ANALYSIS). |
| The craft grant (result + sound + achievement) | PARTIAL | `app/clicks.rs::handle_crafting_clicks` | **S1-L-11d (2026-11-14)**: the created item is the C#'s `CreateResult` (`CraftingRequests.cs:122-131`) — the `OnCreated` swap (5437 → 5358) and the natural prefix roll (`Item.Prefix(-1)`, stack ≤ 1) — and the grant keeps the prefix (`Item.CanStack`). The cursor gate (`Main.CursorHasSpaceToCraftRecipe`, `Main.cs:42309-42345`) makes a prefixable recipe craft once per click. The `Grab` cue plays (`CraftingEffects.cs:51`). **Remaining deviation**: the C# hands the result to the cursor (`Main.CraftItem_GrantItem`, `Main.cs:42607-42612`) and the port grants it into the inventory directly (ANALYSIS). |
| Focus / refocused recipe / repositioning | MISSING | — | `Main.focusRecipe`, `TryRefocusingRecipe`, `craftingUI.VisuallyRepositionRecipes` have no counterpart. |

## Tests

`crates/tr-app/src/craft.rs` — **23** `#[test]` functions:
`the_progression_batch_registers_verbatim`, `test_dw_1_1_table_integrity` (the full-table
walk: ids/groups/stacks, count ≥ 2 700), `the_group_table_is_the_setuprecipegroups_order`,
`requirement_matches_any_group_member`, `station_inheritance_walks_the_counts_as_chain`,
`test_dw_1_2_workbench_reference_recipe`, `test_dw_2_1_can_craft_cases`,
`the_environment_gates_the_liquid_and_biome_recipes`,
`scan_adjacent_reads_liquids_and_the_water_tiles`,
`the_chest_range_scan_finds_tile_chests_in_600px`, `the_chest_ingredients_count_and_drain`,
`draining_the_last_stocked_chest_slot_trims_the_implicit_air_tail`,
`a_stale_entry_with_a_dead_tile_is_filtered_from_the_range_scan`,
`the_toggle_off_ignores_the_chests`, `available_is_the_update_recipe_list_composition`,
`addrecipe_sets_alchemy_on_the_tile_13_recipes`,
`the_alchemy_table_discounts_a_third_of_the_units`,
`test_dw_2_2_try_craft_consumes_and_returns`, `test_dw_2_3_scan_adjacent_semantics`,
`test_dw_2_4_station_gate_and_dirty_inputs`, `test_dw_2_5_full_inventory_still_crafts`,
`furniture_recipe_crafts_and_consumes`, `the_utility_batch_chains_the_stations`.

`crates/tr-app/src/ui/crafting.rs` — **10** `#[test]` (layout/geometry/filter-strip only;
no rule test).

**Gap:** ~~no test asserts the *count* against the C# (the integrity test only checks
`count >= 2700`), and no CI step runs `tools/gen_recipes.py --check`~~ — **closed
2026-11-14 (S1-L-11)**: `craft.rs::the_reverse_rows_match_the_csharp_derivation` pins
the 311 reverse flips, `tools/test_gen_recipes.py` re-derives the plan, and
`gen_recipes.py --check` is part of the AGENTS.md verification loop, so the table drift
below can no longer pass the suite.

## Deviations & risks

- **D1 — the recipe table is out of date (behavioural, high).**
  **CLOSED 2026-11-14** (`S1-L-11-recipes-reverse-from-csharp`): the generator derives
  the reverse rows from the **C# item data** — a fresh parse of `Item.SetDefaults`
  (`Item.cs:48663` → `ResetStats` stock + `SetDefaults1..5` case bodies, incl. the
  `if (type >= A && type <= B)` range blocks and the `DefaultTo*` delegation chain) —
  and the regenerated table (**2 791** = 2 480 + **311**) is committed; the 27 restored
  flips are strictly additive (0 removed). The old driver — the port's `itemdata.rs`
  `tile:`/`wall:` fields — was also read with a suffix bug (`projectile: Some(n)`
  contains `tile: Some(n)` as a substring), which is what had actually moved the
  derived set. `python tools/gen_recipes.py --check` + `tools/test_gen_recipes.py` +
  the Rust pin `craft.rs::the_reverse_rows_match_the_csharp_derivation` (311
  `notDecraftable` flips) now gate the drift (AGENTS.md "Verification before merging").
  Historical report: the fresh run emitted **2 791** rows (2 480 authored + **311**
  reverse) against the then-committed **2 764** (2 480 + **284**); the 27 extra rows
  were reverse-wall recipes (`Wood x1`, `Sandstone x1`, `IronBrick x1`, `LunarBrick x1`,
  `LesionBlock x1` ×2, `Cog x1`, `CrystalBlock x1`, `SpiderBlock x1`, `HardenedSand x1`,
  the three `Sandstone` variants, the three `SillyBalloon*`, …).
- **D2 — REFUTED (was "the station reach region is one tile too large").**
  `Player.AdjTiles` (`Player.cs:35671-35674`) scans
  `WorldUtils.ClampToWorld(TileReachCheckSettings.Simple.GetTileRegion(this))`.
  `GetTileRegion` builds `new Rectangle(LX, LY, HX-LX, HY-LY)`
  (`TileReachCheckSettings.cs:56-60`), and the **XNA** `Rectangle.Right` is
  `X + Width` (*exclusive* — the same convention the port's own
  `tr_core::whip::Rect::right` follows), so `Right == HX` and the
  `for (i = Left; i <= Right; i++)` loop covers `LX..=HX`. The port's inclusive
  `lx..=hx` (`craft.rs::scan_adjacent`) is therefore 1:1; the earlier claim
  assumed the `System.Drawing` `Right = X + Width - 1` convention. Verified
  2026-11-13 (`S1-L-11b-craft-reach`) and locked by
  `craft::tests::the_station_reach_region_matches_the_csharp_xna_rect`
  (left/right/top/bottom boundary, one tile outside on each side). The
  remaining real reach gaps are the extender/Artisan-Bread boosts
  (`Player.cs:35671`'s `TB = ateArtisanBread ? 4 : 0`), tracked under S1-L-11d.
- **D3 — the inventory drain is 50 slots wide, the C#'s 58 (behavioural, low-medium).**
  `CraftingRequests.ConsumeItemsFrom` is called with `58`
  (`CraftingRequests.cs:227`); the port's `INV = 50` (`crates/tr-app/src/item.rs:20`).
  Coins/ammo/dye slots live above 50 in the C# inventory, so a recipe that consumes one
  of those cannot be drained in the port. Already admitted in ANALYSIS.md:1036.
- ~~**D4 — the crafted item is not post-processed (behavioural, low).**~~ — **CLOSED
  2026-11-14 (S1-L-11d)**: `CreateResult` (`CraftingRequests.cs:122-131`) is ported
  as `craft.rs::create_result` — the recipe's item cloned, `Item.OnCreated`'s type
  swap (`Item.cs:48936-48942`: the ShellphoneDummy 5437 becomes the Shellphone
  5358) and `Item.Prefix(-1)`'s natural roll for a stack ≤ 1 (`Item.cs:504-610`).
  The grant carries the rolled prefix (`Item.CanStack`, `Item.cs:49865-49872`), and
  the C#'s cursor gate (`Main.CursorHasSpaceToCraftRecipe`, `Main.cs:42309-42345`)
  makes a prefixable recipe craft once per click while the stackable ones keep
  crafting. RNG (S1-A-10): the prefix roll draws from the port's shared
  `Main.rand` stand-in for the natural rolls (`App::prefix_rng`); the C# draws it
  from the same global `Main.rand` as the alchemy discount — the port's two
  stand-ins (`prefix_rng` / `craft_rng`) are the S1-A-11/S1-L-6c residual. Tests
  `s1_l_11d_a_created_prefixable_item_takes_a_random_prefix`,
  `s1_l_11d_the_created_item_takes_the_csharp_on_created_swap`,
  `s1_l_11d_a_prefixable_recipe_crafts_once_per_click` and the app-level
  `s1_l_11d_a_crafted_weapon_keeps_its_rolled_prefix` (mutation-checked).
  **Remaining deviation**: the C# hands the result to the cursor
  (`Main.CraftItem_GrantItem`, `Main.cs:42607-42612`) and the port grants it into
  the inventory directly (ANALYSIS "The crafting pipeline (S1-L-11d)").

- **D5 — the available-recipe filter is incomplete (behavioural, low-medium).**
  `UpdateRecipeList` (`Recipe.cs:244-252`) applies `CraftingUI.RecipeFilterHack.Accepts`
  before the environment test, and the guide-item mode (`Main.guideItem`) replaces the
  whole list. The port's `available` does neither; `ui/crafting.rs` draws the filter strip
  and the search bar but they do not filter. The ATLAS M3-CRAFT-2 row lists the guide
  mode as "out" but does not mention the `RecipeFilterHack`; ROADMAP S1-H-7 (299) calls
  the filter/search bars "draw-only today", and S1-L-11 (359) is the open task.
- **D6 — the alchemy discount uses a private RNG stream (determinism, low).**
  **CLOSED 2026-11-13** (`S1-L-11e-craft-rng-gen-path`): `App.craft_rng` is the exact C#
  `UnifiedRandom` generator (the splitmix `next_rand3` is deleted) — `ingredient_discount`
  rolls `next_max(3) == 0` verbatim (`Recipe.cs:221`), seeded per world `world_seed | 1`
  like the wiring RNG (S1-L-6c). The shared-stream *interleave* with the vanilla global
  `Main.rand` stays the port-wide S1-A-11 residual, unchanged by this fix.
  `the_craft_rolls_come_from_the_unified_random_stream` pins the replay + the draw count
  (mutation-checked).
- **D7 — the reverse-recipe derivation is a proxy (behavioural, low, same root as D1).**
  **CLOSED 2026-11-14** (with D1): the generator reads the C#'s own item registry —
  per-type `createTile`/`createWall` parsed from `Item.cs` (`SetDefaults1..5`; the
  series forms `createWall = 116 + type - 1948` are evaluated per stacked case id, the
  `(ushort)`-casted `DefaultTo*` arguments are resolved through the delegation chain).
  `tools/test_gen_recipes.py` re-derives the whole reverse plan independently of
  `reverse_pass` and diffs it against the emitted rows.
- **D9 — documentation drift in the ATLAS row (cosmetic).**
  **CLOSED 2026-11-14** (with D1): `M3-CRAFT-6` (ATLAS.md:1375) now says **2 480
  authored / 2 791 total** and the helper count is corrected to 34 (`AddRecipe` is not
  a furniture helper). The claimed "the 35 `Add*Furniture` helpers" docstring was not
  found in the generator itself (grep) — the count lives in the ATLAS row only.
- **D8 — the generator hardcodes an absolute path outside the worktree (tooling, low).**
  **CLOSED 2026-11-13** (`S1-L-11e-craft-rng-gen-path`): `gen_recipes.py` reads
  `ROOT/decompiled/Terraria` — the checkout's own junction (a fresh worktree creates it per
  AGENTS.md; a junction-less checkout fails loudly). Verified by `gen_recipes.py --check`
  loading the tree; the committed `recipes_gen.rs` drift (D1) is S1-L-11's open row.
- **D9 — documentation drift in the ATLAS row (cosmetic).** `M3-CRAFT-6` (ATLAS.md:1375)
  says "2479 authored"; the actual split is **2 480 authored** (2 481 executed sites −
  1 skipped `needTorchGodsFavor`). The generator's docstring also says "the 35
  `Add*Furniture` helpers" — there are **34** (the 35th name it matches is `AddRecipe`
  itself). Not admitted.
- **D10 — the artisan-bread reach boost is missing (behavioural, low).**
  `Player.AdjTiles` widens the region by 4 tiles when `ateArtisanBread`
  (`Player.cs:35671`); the port's region is fixed at X5/Y3.
- **D11 — the port's `Recipe` allows multiple stations, the C#'s allows one (cosmetic).**
  `stations: &[u16]` + `meets_environment`'s `all(...)` can express states the C#'s single
  `requiredTile` cannot. Equivalent for every current row (the generator emits ≤ 1).
- **D12 — the pending-craft-request subtraction is missing (behavioural, low).**
  `Recipe.SubtractOwnedItem` (261-267) removes already-requested ingredients from
  `_ownedItems`; the port has no pending requests (no multiplayer craft requests), so the
  available list may show recipes another client is already crafting.

## Task seeds

- **S** — ~~Regenerate `recipes_gen.rs` from the current decompiled tree and commit it; add
  `python tools/gen_recipes.py --check` to the CI/verification loop so D1 cannot recur.~~
  **DONE 2026-11-14** (S1-L-11): regenerated (2 791 rows) and the check is part of the
  verification (AGENTS.md) with `tools/test_gen_recipes.py` alongside. Closes: D1, D9.
- **M** — ~~Derive the reverse rows from the C#'s own item data (`createTile/createWall`)
  instead of `itemdata.rs`, or pin the item registry and regenerate; add a test that the
  reverse-row count matches the C#'s rule.~~ **DONE 2026-11-14** (S1-L-11): the
  generator parses `Item.SetDefaults`; the count test is
  `craft.rs::the_reverse_rows_match_the_csharp_derivation` (311 flips) +
  `tools/test_gen_recipes.py` (independent transcription). Closes: D1, D7.
- **M** — Fix the reach region to the C#'s exclusive right/bottom edge
  (`HX-1`/`HY-1`) and add a boundary test (a station exactly at the vanilla edge must not
  be adjacent; one tile further must not be either). Closes: D2.
- **M** — Implement the available-recipe filter: port the `RecipeFilterHack`
  equivalents (`CraftStationRecipeFilter`, `WaterSourceRecipeFilter`) and the guide-item
  mode (`Recipe.CollectGuideRecipes`), wiring the existing draw-only filter strip in
  `ui/crafting.rs`. Depends on: the panel work of research/20. Closes: D5; overlaps
  ROADMAP S1-H-7 / S1-L-11.
- **S** — Widen the inventory drain to the C#'s 58 slots (coins/ammo/dye) so recipes
  consuming them work. Closes: D3.
- **M** — Port the created-item pipeline: `CreateResult`'s `OnCreated(RecipeItemCreationContext)`
  and the `Prefix(-1)` on a stack ≤ 1. Depends on: the prefix engine (M3-PREFIX).
  Closes: D4.
- **S** — Move the generator's `DECOMP` to a repo-relative path (or read the worktree
  junction) and drop the absolute `Terraria-Rust` path. Closes: D8.
- ~~**S** — Port the artisan-bread reach bonus (+4 tiles) in `scan_adjacent`.~~ —
  **DONE 2026-11-14 (S1-L-11d)**: `craft.rs::tile_reach` ports the C#'s whole
  reach formula — `Player.tileRangeX/Y` (the statics `Player.cs:2496-2498`, the
  defaults `:57113-57114`) with the per-tick writes (the Journey mode's Far
  Placement Range power ×2 +8/+8, `Player.cs:18818-18826`; the Extendo Grip / the
  Architect's Gizmo Pack +3/+2, `:13040-13044`; the held Toolbox +1/+1,
  `:14955-14958`), the `TileReachCheckSettings.Simple` clamp of 20
  (`TileReachCheckSettings.cs:20-34`, `:84`) and the artisan loaf's +4
  (`Player.cs:35671`). Closes: D10. Tests `s1_l_11d_the_tile_reach_follows_the_csharp_formula`
  and `s1_l_11d_the_artisan_bread_reaches_a_far_station`. The loaf's own use
  effect (item 5326, `Player.cs:45218-45221`) is still unported — the flag
  `Player.ateArtisanBread` rides the player and stays false in play.
- **S — the generator's tail branch** (`expand_body`, `tools/gen_recipes.py`): the
  expansion loop broke out on the `Recipe.CreateReverseWallRecipes()` tail
  **before** copying the text that precedes it, so the 414 authored rows
  between the last expansion mark and that call (`Recipe.cs:15148`) were
  dropped — the table's "2 481 executed sites" was the generator's own
  truncated output. **DONE 2026-11-14 (S1-L-11d)**: the branch copies the
  text first; the table is 3 206 rows = 2 895 authored + 311 reverse and the
  pin in `tools/test_gen_recipes.py` re-taken.
- **M** — Port the pending-craft-request subtraction (`SubtractOwnedItem` +
  `CraftingRequests._pendingCrafts`) so the multiplayer path reserves ingredients.
  Depends on: the netcode work in research/17. Closes: D12.
- **S** — Correct the ATLAS `M3-CRAFT-6` row (authored count 2 480, the 34 helpers) and
  add a "recipe table drift" note to `docs/ANALYSIS.md`. Closes: D9.
