# 07 — Item core: defaults, prefixes, tooltips

> Wave 1 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP

## Scope

The module is "what an item *is*" before any use logic: the per-id default table, the
prefix system that mutates those defaults on an instance, and the read-out of both on
screen (name colour, tooltip rows, price/stack).

C# side (all under `decompiled\Terraria\`):

- `Item.cs` (49 703 lines) — the default table itself. `SetDefaults(int Type, ItemVariant)`
  dispatcher at `Item.cs:48663` fans out to `SetDefaults1` `:1387` (ids ≤ 1000),
  `SetDefaults2` `:12931` (≤ 2001), `SetDefaults3` `:21380` (≤ 3000), `SetDefaults4`
  `:27659` (≤ 3989) and `SetDefaults5` `:35462` (the rest); every arm first calls the
  shared stock reset `netDefaults(int type)` `:1211-1367`, and the dispatcher epilogue
  `:48713-48925` fills the shared fields (name, damage class, `width`/`height`, the
  `GetItemSettings` hooks). `ResetStats(int Type)` `:48944-49051` is the stock-value
  reset (incl. `this.maxStack = Item.CommonMaxStack;` `:49017`, with
  `Item.CommonMaxStack = 9999` set once in the static ctor `:49966`).
  Instance/utility surface: `Clone()` `:49837`, `Refresh(bool)` `:49945`,
  `TurnToAir()` `:49884`, `CanStack` `:49865`, `GetStoreValue()` `:49901-49908`,
  `shopCustomPrice` field `:291`, the `buyPrice`/`sellPrice` coin statics `:49417`/
  `:49422-49425`, `OnCreated` `:48936-48942`.
  The prefix engine: `Item.Prefix(int prefixWeWant)` `:504-618` (application order,
  rejection retry, rare ladder, `value²`), `TryGetPrefixStatMultipliersForItem`
  `:709-1082` (the per-id switch — there is no stored stat table),
  `BestPrefixValue` `:1082-1099`, `GetRollablePrefixes` `:1101-1140`,
  `RollAPrefix` `:1139-1148`, `IsAPrefixableAccessory` `:1150`,
  `GetPopupRarityColor` `:652-707`.
- `GameContent\Prefixes\PrefixLegacy.cs` — the eight roll tables `:29-73` and the
  membership/`ItemSets` lists `:98-104`.
- `ID\ItemID.cs` — `ItemID.Count = 6196` `:14861`; the `Sets` flags the defaults and
  the prefix gates read (`IsFood` `:498`, `CanGetPrefixes` `:429`, `IsAMaterial` `:487`,
  `IsLavaImmuneRegardlessOfRarity`); `ID\PrefixID.cs::ReducedNaturalChance` `:14`.
- `GameContent\Items\ItemVariants.cs` — the per-seed variant picker called from
  `SetDefaults` (`ROADMAP.md:514-516`).
- `Main.cs` — `MouseText_DrawItemTooltip_GetItemNameColor(int rare, byte diff)`
  `:20557-20629` and `MouseText_DrawItemTooltip_GetLinesInfo` `:20631-20875`; the
  reforge input slot/cost block in `DrawInventory` `:41584-41601`.
- `Player.cs` — `GrantPrefixBenefits(Item)` `:13953-14047` (the 62-80 accessory-prefix
  table) and `GetItemExpectedPrice(Item, out selling, out buying)` `:35359-35389`.
- `Lang.cs` — the `LegacyMenu`/`LegacyInterface`/`LegacyTooltip` key families `:481-497`.

Rust side (all under `crates\tr-app\src\`):

- `itemdata.rs` (7 527 lines, GENERATED) — `ItemDef` `:19-116` (38 fields),
  `ITEMS: [ItemDef; 6196]` `:120`, `NAMES: [&str; 6196]` `:6320`,
  `NEGATIVE_DEFAULTS: [(i16, u16); 48]` `:7100`, `ON_CREATED: &[(u16, u16)]` `:7113`,
  `VariantOverride` `:7206` + `VARIANT_OVERRIDES` `:7213`, tests `:7308-7550`.
- `registry.rs` — the accessor façade over the table + `ITEM_CORRECTIONS`.
- `prefix.rs` (854 lines) — the whole prefix engine (tables, multipliers, roll, reforge).
- `item.rs` — the in-memory slot/drop types (`ItemSlot`, `ItemDrop`), `slot_for_pickup`.
- `world_sim\items.rs` — the fixed-step item world (pickup, void vault, lava, shimmer).
- `ui\tooltip.rs` (646 lines) + `lang.rs` — the on-screen read-out.
- `shop.rs` — the price math (`buy_price`, `sell_price`, `reforge_cost`).

Responsibility: the C# keeps one mutable `Item` whose fields are re-derived on demand by
`SetDefaults`/`Refresh`/`Prefix`; the port splits this into a static per-kind table
(`itemdata.rs`) plus pure recompute functions (`prefix.rs::prefixed_stats`) and a small
instance record (`ItemSlot`/`ItemDrop` = kind, count, prefix only).

## Key entities

C#:

- `Terraria\Item.cs::SetDefaults` `:48663` — the dispatcher; the only place per-id
  defaults are authored.
- `Terraria\Item.cs::ResetStats` `:48944-49051` — the stock reset every `netDefaults`
  starts from.
- `Terraria\Item.cs::Prefix` `:504-618` — prefix application (rounding order, rejection,
  rare ladder, price).
- `Terraria\Item.cs::TryGetPrefixStatMultipliersForItem` `:709-1082` — the per-id
  multiplier switch (62 weapon arms + the accessory arms).
- `Terraria\Item.cs::GetRollablePrefixes` `:1101-1140` / `RollAPrefix` `:1139-1148` —
  the category order and the natural-roll gate.
- `Terraria\GameContent\Prefixes\PrefixLegacy.cs` `:29-104` — the roll tables +
  membership sets (source of truth for `prefix.rs`'s tables).
- `Terraria\Main.cs::MouseText_DrawItemTooltip_GetItemNameColor` `:20557-20629` —
  the rare→colour table (this is the whole rarity-colour logic).
- `Terraria\Main.cs::MouseText_DrawItemTooltip_GetLinesInfo` `:20631-20875` — the
  tooltip row builder (≈245 lines of gates).
- `Terraria\Player.cs::GrantPrefixBenefits` `:13953-14047` — the 62-80 flat stat table.

Rust:

- `crates\tr-app\src\itemdata.rs::ItemDef` `:19-116` — one static row per kind; note
  `width`/`height` `:85-88` are modelled (rare among fields) and the `DamageClass` is
  *derived* `:56-58`, not stored.
- `crates\tr-app\src\itemdata.rs::ITEMS` `:120` — 6 196 rows, ids `0..6195`
  (`ItemID.Count = 6196`, `ItemID.cs:14861`).
- `crates\tr-app\src\itemdata.rs::NEGATIVE_DEFAULTS` `:7100` / `ON_CREATED` `:7113` /
  `VARIANT_OVERRIDES` `:7213` — the `netDefaults` epilogue arms, the `OnCreated`
  list and the `ItemVariants` table.
- `crates\tr-app\src\registry.rs::max_stack` `:114-116`, `::rare` `:201`,
  `::damage` `:196`, `::place_tile` `:175-185` (carries the id-3813 workaround `:181-183`),
  `::tile_drop` `:130-163`, `::by_tile` `:166` / `::by_wall` `:171`, the coin helpers
  `:213-232`, and `::make_npc` `:106-108` (a documented stub — every kind maps to 0).
  `registry.rs` also holds `ITEM_CORRECTIONS` for id 6171 (`PORT_MAP.md:154`).
- `crates\tr-app\src\prefix.rs::PREFIXES_FOR_*` `:22-53` — the eight roll tables;
  `::SWORDS_HAMMERS_AXES_PICKS` `:56`, `::GUNS_BOWS` `:78`, `::MAGIC` `:86`,
  `::SUMMON` `:93`, `::REDUCED_NATURAL_CHANCE` `:107`, `::CAN_GET_PREFIXES_EXCLUDED`
  `:114` — the membership sets (sorted, binary-searched via `::in_set` `:126`).
- `crates\tr-app\src\prefix.rs::rollable_prefixes` `:132` — `GetRollablePrefixes`'s
  order incl. the accessory denial-list arm for `IsAPrefixableAccessory`.
- `crates\tr-app\src\prefix.rs::Multipliers::stat_switch` `:195` / `::value_multiplier`
  `:293` / `::rejected` `:316` — the three `Item.Prefix` helpers.
- `crates\tr-app\src\prefix.rs::prefixed_stats` `:367-402` — the application block,
  recomputed from the kind's defaults; `round_bankers` for the integer fields.
- `crates\tr-app\src\prefix.rs::shift_rare` `:406-428` — the rare ladder + clamp.
- `crates\tr-app\src\prefix.rs::roll_prefix` `:455` / `::best_prefix_value` `:504` /
  `::reforge_roll` `:528` / `::grant_prefix_benefits` `:540-591`.
- `crates\tr-app\src\item.rs::ItemSlot` `:67` / `::MAX_STACK` `:78` / `::EMPTY_SLOT`
  `:80` / `::slot_for_pickup` `:91`; `::ItemDrop` `:132`.
- `crates\tr-app\src\world_sim\items.rs::update_items` `:21-224` / `::spawn_item_drop`
  `:226` / `::spawn_item_drop_px` `:240`.
- `crates\tr-app\src\ui\tooltip.rs::item_name_color` `:107-129`,
  `::tooltip_box_rect` `:136-151`, `::item_tooltip_lines` `:166-296`,
  `::draw_item_tooltip` `:301-361`, `::inv_bg_rects` `:35-54`.
- `crates\tr-app\src\lang.rs::tip` `:148`, `::menu` `:141`, `::inter` `:154`,
  `::item_name` `:161-169` (with the English fallback).
- `crates\tr-app\src\shop.rs::buy_price` `:1257-1260`, `::sell_price` `:1282-1285`,
  `::reforge_cost` `:1272-1277`, `::round_half_to_even` `:1289-1296`.

## Dependencies

- Calls into: `tr_core::localization` (`LanguageManager`, the culture list) from
  `lang.rs`; `tr_core::World` from `world_sim\items.rs` (lava/tile reads) and `shop.rs`;
  `crate::assets` (`lookups`, the `Inventory_Back13` sheet) from `ui\tooltip.rs`;
  `crate::player::Player` from `prefix.rs::grant_prefix_benefits` (called at
  `player.rs:2831`).
- Called by: `registry`/`itemdata` are read by essentially every system — the held item
  (`world_sim\editing.rs`), the inventory UI (`ui\inventory.rs`, `ui\slot.rs`), the
  crafting panel (`ui\crafting.rs`), the shops (`shop.rs` + `ui\shop`*), the tooltip
  draw (`ui\tooltip.rs::draw_item_tooltip`, driven by `hovered_slot` `:407`).
- Shared data structures: `ItemDef` (the static table) is the only per-item source of
  truth; `ItemSlot`/`ItemDrop` are the two instance records (kind + count + prefix).
  `PrefixedStats` (`prefix.rs:344-361`) is the derived-instance view.

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `SetDefaults` id coverage | PASS | `itemdata.rs::ITEMS` `:120` | `[ItemDef; 6196]` == `ItemID.Count = 6196` (`ItemID.cs:14861`); all five bands generated (`ROADMAP.md:521-525`, M3-ITEM-1) |
| `netDefaults`/`ResetStats` stock defaults | PASS | `itemdata.rs::ItemDef` doc comments | each field carries its stock value + `Item.cs` line cite (e.g. `max_stack` `:24-25` → `Item.cs:49017`, `value` `:61-62` → `Item.cs:49030`); fallback path tested by `registry::tests::unregistered_kinds_fall_back_to_the_vanilla_default` `:309` |
| Dispatcher epilogue (`GetItemSettings`) | PARTIAL | `itemdata.rs::ITEMS` | the epilogue's `width`/`height` are modelled (`:85-88`); the C#'s other epilogue fields (shoot-speed class scaling, the `mech.SpawnChance`-style special hooks) are folded into the generated rows or absent — UNVERIFIED row-by-row |
| `ItemVariants` (per-seed picks) | PARTIAL | `itemdata.rs::VARIANT_OVERRIDES` `:7213` | table present; `ROADMAP.md:521-523` records "conditions default false" until the seed system drives them |
| `netDefaults` negative ids + `OnCreated` | PASS | `NEGATIVE_DEFAULTS` `:7100`, `ON_CREATED` `:7113` | both pinned by `itemdata.rs::tests::test_DW_1_2_negative_defaults_and_on_created` `:7498` and `registry.rs::tests::test_dw_2_2_net_defaults_and_on_created` `:385` |
| `Clone`/`Refresh`/`DeepClone` | MISSING | — | the port has no instance snapshot; `prefix.rs::prefixed_stats` `:367` recomputes from the kind's defaults instead. Exact for a fresh `SetDefaults` item and (per `ROADMAP.md:1389`, M3-REFORGE-1) for a reforge, since the C# also resets before re-rolling |
| `favorited`, `shopCustomPrice`, `buyOnce`, `isAShopItem`, `questItem`, `vanity`, `uniqueStack`, `tileWand`, `fishingPole`, `mountType` | MISSING | — | no `ItemDef` fields; the favourites ride the slot-index map (`ROADMAP.md:1392`, M3-STORE-3); the rest are consumers' gaps |
| Prefix roll tables + membership sets | PASS | `prefix.rs:22-124` | counts 40/14/35/36/22/14 (terrarian/accessories also present) match `PrefixLegacy.cs:29-104`; sorted + binary-searched, pinned by `prefix.rs::tests::tables_are_sorted_and_prefix_lists_verbatim` `:605` |
| Prefix application order/rounding | PASS | `prefix.rs::prefixed_stats` `:387-401` | integers through banker's rounding (`round_bankers`), floats multiply, crit/bonus added — same order as `Item.cs:575-598` |
| Rare ladder + `value²` | PASS | `prefix.rs::shift_rare` `:406-428`, `value_multiplier` `:293` | ≥1.2 → +2, ≥1.05 → +1, ≤0.8 → −2, ≤0.95 → −1, clamp `[-1, 11]` skipped at ≤ −11; pinned by `value_and_rare_shift` `:750` |
| Rejection retry (invisible prefix) | PASS | `prefix.rs::Multipliers::rejected` `:316` + `roll_prefix` `:455` | pinned by `invisible_changes_are_rejected_and_rolled_again` `:725` and `rejection_retry_never_lands_the_invisible_prefix` `:734` |
| Roll modes (`Prefix(-1..-4)`) | PASS (mechanism) / PARTIAL (RNG) | `prefix.rs::roll_prefix` `:455`, `reforge_roll` `:528` | modes ported (`exact_and_special_modes` `:842`); the draw source is a splitmix stand-in (`next_u64` `:434`), **not** `UnifiedRandom` |
| `GrantPrefixBenefits` 62-80 | PASS | `prefix.rs::grant_prefix_benefits` `:540-591` | arm-for-arm identical to `Player.cs:13953-14047`; C# `statManaMax2 += 20` `:13973` folds into the port's derived `mana_max` (`player.rs:590`, re-granted per tick at `:2655`/`:2681`) |
| `BestPrefixValue` + reforge top tier | PASS | `prefix.rs::best_prefix_value` `:504`, `reforge_roll` `:528` | pinned by `best_prefix_value_matches_the_visible_table` `:809`, `reforge_roll_never_lands_none_and_flags_the_top_tier` `:823` |
| Reforge cost | PARTIAL | `shop.rs::reforge_cost` `:1272-1277` | `value × stack × PriceAdjustment / 3` matches the inline block `Main.cs:41584-41601`; the Discount-Card ×0.8 arm (`Main.cs:41588-41590`) is not modelled (no discount accessory) |
| `GetItemExpectedPrice` | PARTIAL | `shop.rs::buy_price` `:1257`, `::sell_price` `:1282` | the non-`buyOnce` arms (×adjustment / ÷adjustment) match `Player.cs:35387-35388` incl. banker's rounding; but the port reads the **kind default** `ITEMS[i].value` while the C# reads the instance's `GetStoreValue()` (`Item.cs:49901-49908`, prefix `value²`), and the `buyOnce` + `discountAvailable` arms (`:35369-35383`, `:35390-35396`) are absent |
| Rarity name colour | PARTIAL | `ui\tooltip.rs::item_name_color` `:107-129` | −11, −1, 1..10 and ≥11 are byte-identical to `Main.cs:20568-20619` (verified in this pass); the `diff == 1`/`diff == 2` master/journey overrides (`:20620-20627`) and the dynamic −12/−13 arms (`:20564-20567`) are not modelled |
| Tooltip rows | PARTIAL | `ui\tooltip.rs::item_tooltip_lines` `:166-296` | the stat rows are faithful (damage + class suffix, crit, use-time tiers, knockback words, defense/pick/`axe × 5`/hammer, `tileBoost` sign only when positive — all verified against `Main.cs:20843-20875`); missing: `Item.ToolTip` per-item text, the player-side layers, heal/mana/fishing-pole/mount/pet/quest/vanity/`tileWand` rows and the stack-suffixed `GetHoverName` (below) |
| `CanStack` | PASS | `crate::bank::can_stack` | ported (`ROADMAP.md:1393`, M3-CHEST-INT) against `Item.cs:49865-49871` |
| Lava immunity / `TurnToAir` for drops | PARTIAL | `world_sim\items.rs:100-104` | the `rare ∈ {0,−1}` burn + immune-set test `rare0_drops_burn_in_lava_and_the_immune_set_survives` `:473`; the item-level `TurnToAir` (`Item.cs:49884`) and `IsLavaImmuneRegardlessOfRarity` breadth are not separately verified here — UNVERIFIED against the full set |

## Tests

Grep of `#[test]` over the module's files (line = the test's `fn` line):

- `itemdata.rs` (5): `test_DW_1_2_spot_sample_matches_item_cs` `:7309`,
  `test_DW_1_4_food_and_epilogue_rules_present` `:7460`,
  `test_DW_1_2_negative_defaults_and_on_created` `:7498`, `test_DW_1_2_variant_tables`
  `:7507`, `test_DW_1_2_names` `:7542` (helper `same_except_id` `:7299`).
- `registry.rs` (11): `use_timings_match_item_setdefaults` `:279`,
  `unregistered_kinds_fall_back_to_the_vanilla_default` `:309`,
  `test_DW_2_2_wooden_set_item_data_matches_item_cs` `:316`,
  `test_dw_2_2_generated_rows_match_item_cs_spots` `:351`,
  `test_dw_2_2_net_defaults_and_on_created` `:385`,
  `test_dw_2_3_out_of_range_fallbacks_unchanged` `:397`,
  `test_DW_3_2_tile_drop_wooden_set` `:413`, `test_DW_3_1_tile_drop_map` `:434`,
  `test_DW_2_3_the_corrected_staff_rows_answer_the_item_cs_values` `:457`,
  `test_DW_2_3_a_prefixed_staff_rides_the_corrected_base` `:506`,
  `test_DW_2_3_the_placeholder_band_neighbours_stay_untouched` `:519`.
- `prefix.rs` (13): `tables_are_sorted_and_prefix_lists_verbatim` `:605`,
  `category_resolution_order` `:635`, `natural_roll_is_quarter_no_prefix` `:658`,
  `non_prefixable_kinds_consume_no_draws` `:692`, `stat_switch_spot_checks` `:704`,
  `invisible_changes_are_rejected_and_rolled_again` `:725`,
  `rejection_retry_never_lands_the_invisible_prefix` `:734`,
  `value_and_rare_shift` `:750`, `accessory_prefixes_bump_value_only` `:772`,
  `grant_prefix_benefits_sums` `:785`,
  `best_prefix_value_matches_the_visible_table` `:809`,
  `reforge_roll_never_lands_none_and_flags_the_top_tier` `:823`,
  `exact_and_special_modes` `:842`.
- `item.rs` (5): `pickup_prefers_stacks_then_empty_slots` `:364`,
  `registry_maps_items_to_tiles_walls_and_coins` `:406`,
  `drops_fall_and_rest_on_the_ground` `:426`, `drops_bounce_off_walls` `:440`,
  `magnet_collects_a_drop_next_to_the_player` `:458`.
- `ui\tooltip.rs` (9): `m7_ui_6_the_tooltip_rows_follow_their_gates` `:455`,
  `inv_bg_min_size_is_twenty` `:492`,
  `item_name_color_zero_is_white_neg_one_is_grey` `:500`,
  `item_name_color_covers_high_and_negative_rarities` `:506`,
  `tooltip_clamps_to_the_screen` `:521`,
  `hovered_slot_none_when_not_over_a_cell` `:527`,
  `a_copper_shortsword_shows_damage_crit_use_time_and_a_knockback_word` `:614`,
  `a_tool_shows_its_pick_axe_hammer_rows` `:630`,
  `a_plain_block_has_no_stat_rows` `:643`.
- `lang.rs` (2): `russian_texts_and_item_names_resolve` `:191` (skips itself when no
  localization sources are present), `tip_and_inter_keys_follow_the_c_sharp_prefixes`
  `:223`.
- `world_sim\items.rs` (8): `a_drop_in_shimmer_transforms_on_the_ninetieth_tick` `:329`,
  `the_ramp_decays_out_of_shimmer` `:354`,
  `the_coin_merge_multiplies_and_airs_out` `:378`,
  `a_decraftable_item_spawns_its_ingredients` `:392`,
  `a_non_transformable_item_never_transforms` `:408`,
  `the_shimmered_item_floats_without_gravity` `:420`,
  `rare0_drops_burn_in_lava_and_the_immune_set_survives` `:473`,
  `the_doll_still_triggers_the_sacrifice` `:537`.
- `shop.rs` (11) — stock/resolve tests plus the two price tests
  `the_prices_ride_the_happiness_adjustment` `:1438` and
  `the_buy_price_matches_the_happiness_multiplier` `:1517`.

Coverage is strong for the *table* (spot-samples against `Item.cs`) and the *prefix
engine*; weak for the *tooltip* (rows/colours only, no pixel/text snapshot of a real
vanilla tooltip) and for the *price* math (no money-format or discount-card test).

## Deviations & risks

1. **Pickup double-add — `world_sim\items.rs:191-194`** (behavioural, high, NOT
   admitted anywhere). The pickup block adds the transfer twice:
   `let drop_count = remaining;` (`:191`) then
   `inventory[slot].count += drop_count` (`:193`) **and**
   `inventory[slot].count += drop.count` (`:194`). On the normal path (no void vault)
   `remaining == drop.count`, so a pickup of N grants 2N into the slot. The C#
   `Player.GetItem` transfers once and leaves the leftover in the world item. This is
   the single most consequential finding of this module — every tile drop, mob drop and
   shop grant is doubled. Fix: delete line 194 and use `remaining`/`drop_count`
   consistently (the vault path needs `remaining`, not `drop.count`).
2. **`MAX_STACK` / `CommonMaxStack` mismatch for out-of-range ids** (behavioural, low,
   partially admitted). `item::MAX_STACK = 999` (`item.rs:78`) and
   `registry::max_stack` `:114-116` (falls back to 999) diverge from the C#
   `Item.CommonMaxStack = 9999` (`Item.cs:49017`, set `:49966`), which is what
   `ResetStats` assigns before any id-specific `maxStack`. Generated rows carry the
   correct per-id values, so only ids outside `0..6195` (modded/unregistered) or the
   air id are wrong. `registry.rs:321` acknowledges the 9999 baseline in a comment.
3. **Prefix RNG is a stand-in** (determinism, medium, NOT admitted in this file's
   sources). `prefix.rs::next_u64`/`next` `:434-453` is a splitmix stream shared with
   `rain.rs`, not `UnifiedRandom`; `roll_prefix` consumes draws in the C# order, but
   the concrete stream differs, so a vanilla-replay of prefix rolls cannot match. Only
   matters for exact-parity/replay work.
4. **`GetStoreValue` is bypassed by the price helpers** (behavioural, medium, partially
   admitted). `shop.rs::buy_price` `:1257` / `::sell_price` `:1282` read
   `ITEMS[i].value` (kind default). The C# prices the *instance*:
   `GetStoreValue()` = `shopCustomPrice ?? value` (`Item.cs:49901-49908`) where `value`
   already carries the prefix's ×`value²` (`Item.cs:614-615`). A prefixed item therefore
   prices wrong in the port, and the `buyOnce` / `discountAvailable` arms
   (`Player.cs:35369-35383`, `:35390-35396`) are missing entirely. The reforge cost at
   least takes the caller's current value (`shop.rs:1272`) and documents the prefix
   scaling (`:1267-1271`).
5. **Tooltip is a subset of `GetLinesInfo`** (behavioural/cosmetic, medium, admitted in
   the port's own comments `ui\tooltip.rs:163-165`, `:252-256`). Missing rows: the
   per-item `Item.ToolTip` text, the player-side layers (weapon damage multiplier, crit
   bonuses, `minionKB`, the quiver ×1.1), `healLife`/`healMana`/mana-restore rows
   (`Main.cs:20876+`), the fishing-pole pair (`PrecentFishingPower`/`BaitRequired`,
   `:20805-20810` — the registry has no `fishingPole` field), and the equipable gate is
   narrower than the C#: the port tests only `head/body/leg_slot > 0 || accessory`
   (`:257`) while `Main.cs:20816` also admits `projHook[shoot]`, `mountType != -1` and
   light/vanity-pet buffs; `vanity` (`tip[24]` `:20838-20841`), `questItem`
   (`inter[65]` `:20833-20836`) and `tileWand` (`tip[52]` `:20828-20831`) rows are
   absent. Line 0 also drops the stack suffix — the C# uses
   `Item.GetHoverName(item, ItemSlot.EstimateDisplayStack(item))` (`Main.cs:20633`), the
   port `lang::item_name(kind)` (`lang.rs:161`).
6. **Rarity colour: the journey/master overrides are missing** (cosmetic, low, admitted
   in `ui\tooltip.rs:124-125`). `item_name_color` `:107-129` matches
   `Main.cs:20568-20619` exactly for the static rarities, but `diff == 1` (mcColor) and
   `diff == 2` (hcColor) (`Main.cs:20620-20627`) and the dynamic −12 (DiscoR/G/B) /
   −13 (masterColor·200) arms (`:20564-20567`) have no port; the function has no `diff`
   parameter. No vanilla item currently uses −12/−13, so this is unreachable today.
7. **No instance-level item record** (behavioural, medium, structural). There is no
   `Clone`/`Refresh`/`favorited`/`uniqueStack`/`stack`-per-instance state beyond
   `ItemSlot`/`ItemDrop`; everything is recomputed from `ITEMS[kind]`. Correct for
   fresh drops and reforges (the C# resets too) but any future system that mutates an
   instance in place (enchanted/painted items, the Angler's quest item state) has no
   home yet.
8. **`make_npc` is a stub** (behavioural, low, admitted at `registry.rs:100-108`): the
   shimmer release arm always maps to 0, so NPC-releasing shimmer transforms are
   inert (the mechanism itself is tested).

ROADMAP DONE rows re-verified in this pass: M3-ITEM-1 (table scale-up) **PASS**
(6196/6196 rows + variants table + tests); M3-PREFIX (roll engine + tables) **PASS**
(tables/rounding/rare ladder/benefits all match the cited C#); M3-REFORGE-1 (cost +
best tier) **PARTIAL** (correct except the missing discount arm, which the row itself
admits); `Main.cs` tooltip localization row `PORT_MAP.md:162` **PARTIAL** (stat rows
only). PORT_MAP's `Item.cs` → `item.rs` "partial" row (`:165`) is confirmed — and the
pickup double-add above is the concrete reason it cannot be marked done.

## Task seeds

- **S — Fix the pickup double-add** (`world_sim\items.rs:191-194`): keep one addition
  using `remaining`, add a regression test (drop a 5-stack onto an empty inventory row
  and assert the slot holds 5, plus the void-vault case where the vault takes 3 and the
  inventory receives 2). Depends on nothing; blocks any inventory-consumer milestone.
- **S — Add the tooltip's missing gates** (`ui\tooltip.rs::item_tooltip_lines`: add
  `heal_life`/`heal_mana`, `quest_item`, `vanity`, `tile_wand`, the fishing-pole pair)
  — needs the `fishingPole`/`vanity`/`questItem`/`tileWand` `ItemDef` columns from the
  generator first, so pair with the generator task.
- **M — Price the instance, not the kind**: give `ItemSlot` a `value` (or thread
  `PrefixedStats::value` into `shop.rs`), implement `GetStoreValue` semantics
  (`shopCustomPrice ?? value`) and the `buyOnce`/`discountAvailable` arms of
  `Player.cs:35359-35389`; test with a known prefixed item (e.g. a Demonite row
  prettied with a common prefix) against the C# formula.
- **M — Port `Clone`/`Refresh` as an instance snapshot** if and when a system needs
  in-place item mutation (paint, the Angler quest state, loadouts); today
  `prefixed_stats` recompute-from-defaults is only exact because both C# and port reset
  before mutating.
- **L — Finish the tooltip pass to parity with `GetLinesInfo`** (`Main.cs:20631-20875`):
  per-item `ToolTip` text (a data port), the player-side layers (needs the accessory
  damage/crit/minionKB context), `GetHoverName`'s stack suffix, and a golden-text test
  comparing a handful of real items against vanilla strings.
- **L — Verify the generated table row-by-row against a decompile-wide extract**
  (`SetDefaults1..5`): a script that diffs every `ItemDef` field between
  `itemdata.rs` and a machine-read of `Item.cs` (the existing tests are spot samples).
  This closes the "dispatcher epilogue" PARTIAL above and would also surface the
  `width`/`height`/`fishingPole` generator gaps.
- **S — Decide the prefix RNG contract**: either document the splitmix stand-in in
  `ANALYSIS.md` as an accepted deviation, or add a `UnifiedRandom`-compatible stream
  for prefix rolls if exact replays become a requirement.
