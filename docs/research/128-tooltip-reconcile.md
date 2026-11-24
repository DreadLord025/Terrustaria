# 128 — Tooltip 2-way reconcile (W4-e16 × W4-f28, code-only) — STATUS: FINAL

**Branch:** `task/W4-e16-item-tooltip-lines` (this doc) · **Probed tips:**
e16 `380f05a1` (30 witnesses) vs f28 `85165b8b`
(`origin/task/W4-f28-tooltip-lines-2`; full diff
read: `604f208d` GenItemData provision → `d771dcac` port → `d5f7dd82` records →
`85165b8b` compile fixes). Common base `a725acb0` (main). **Mode:** CODE-ONLY —
git + file reads, no compiler; NO code changes to f28.

**CLOSEOUT (final):** the Lead's salvage-exec **f28c is ACTIVE on the f28
branch** — f28 becomes canonical and executes the §5/§8 salvages there.
This branch is now banner-marked (commit `1c15c74c`, comments only):
every duplicated half carries a `DEPRECATED-BY-UNION (128 §…)` marker,
every unique survivor a `UNIQUE-SALVAGE (128 §…)` marker — nothing
deleted, the witnesses + anchors still read the live code until f28c
folds the salvages. The exact hunk list f28c reads is §8.2; the final
witness-dedupe map is §6.

**Deferred-lane re-verification (closeout probe):**
- `f28b` still NOT FOUND (fetch + branch scan at closeout) — f28's tip is
  still `85165b8b`; the §5-f28 list is the whole f28 salvage surface.
- **c24 journey LANDED on its branch** (`origin/task/W4-c24-journey-ui`
  tip `e2e50a42`, NOT in main): see §7 — the sacrifice rows' constants-off
  rationale is REVISED (the gate inputs now exist; the rows still unported
  on every lane).
- **e23 bestiary LANDED on its branch** (`origin/task/W4-e23-bestiary-ui-grid`
  tip `6daf6d3d`, NOT in main): the `bestiary_notes` writer EXISTS
  (`ui/bestiary.rs:534`, witness `the_bestiary_notes_join_the_non_empty_conditions`
  `:2562`) — §7's bestiary entry updated; e16's DROP note is superseded.

## 1. The two shapes

| | e16 (`380f05a1`) | f28 (`85165b8b`) |
| --- | --- | --- |
| Line builder | one `ui/tooltip.rs::item_tooltip_lines` (~550 ln, all 36 kinds inline) | `ui/tooltip.rs` first half + `ui/tooltip_lines2.rs` (1407 ln: `append_second_half` + shared gate helpers) |
| Flag source | `ui/tooltip_sets.rs` census tables (286 ln, hand-copied `ItemID.Sets`/write-censuses) | **GenItemData provision**: 5 new `ItemDef` columns (`expert`/`has_vanity_effects`/`tile_wand`/`not_ammo`/`extractinator_consumable`, `gen.py` stock + `DefaultTo*`/`MakeUsableWithChlorophyteExtractinator` handlers), regen + FNV re-pin `0x1dd2_84fc_4b36_2f46` + `registry.rs` correction rows; ID-sets hard-coded inline in `tooltip_lines2.rs` |
| Prefixed stats | `prefix::prefixed_stats_of(def, prefix)` — applies over the CALLER's row (witness-overridable) | `tooltip_lines2::hover_stats(kind, prefix)` → `prefixed_stats` re-reads the registry; unregistered kinds fall back to the bare def |
| Context | flat `TooltipContext` fields: `context: i32` (full HoverRef→ctx map), `favorited`, `has_coin_gun` (58-slot grid+coin+ammo scan), `shop_open`, `open_bank: Option<usize>`, `show_loadout_share_hint`, `master_color` (static 1.0) | `HoverFlags {favorited, social, worn_armor, bestiary_notes}` + `ctx.world: TooltipWorld` + `inventory: Option<&[ItemSlot]>` (`.take(58)` over a **50-slot** slice) + `open_chest: i32` (captured pre-`chest::draw`) |
| World gates | `ui::tooltip::TooltipWorld` (7 fields) draw_hud param, flattened into ctx | `ui::tooltip_lines2::TooltipWorld` (9 fields: +`npc_shop`, `master_color`, `tenth_anniversary`; `STOCK` const) draw_hud param + `ctx.world` |
| Loadout head | flag/flag2 latches + `canLoadoutShareAt` 8-12 + `UI.ItemLoadoutShared` + `!flag` equipable suppression + share-hint row + settings key | DEFERRED (plain tip[56]/[57] favourite branch only) |
| `{0:+0;-#}` | port-side `signed_fmt` (research/33 D5 workaround) | real .NET custom-section grammar in `tr-core format.rs::apply_format_spec` + standard-specifier guard (`85165b8b`) |
| Master colour | static `ctx.master_color = 1.0` (no animator) | `App.master_color/master_color_dir` + `advance_master_color` (±0.05, `Main.cs:18225-18235`) wired in `frame/draw.rs` + `flow.rs` fixture |
| Rapid attack | read-only `Player::apply_rapid_attack_bonus(&self, …)` (no increase arm, no decay — cited) | full `(&mut self, …, increase_bonus)` (+0.12 cap 0.62) + `ResetEffects` −0.005 decay (`Player.cs:18278-18285`) |
| DD2 gate | `downed_dd2_any = dd2_downed_t1‖t2‖t3` (matches `DD2Event.DownedInvasionAnyDifficulty` getter, `DD2Event.cs:89-99`) | `world.dd2_won` stand-in (the win sentinel — wrong granularity) |
| Fishing key | **`PrecentFishingPower`** (the vanilla typo, `Main.cs:20806`) | `RecentFishingPower` (base-era wrong key carried forward — renders nothing) |
| Witnesses | 30 (26 `w4_e16` tooltip + 3 sets + 1 settings) | 18 (15 `w4_f28` tooltip_lines2 + 3 tr-core format; the commit/ANALYSIS prose says "13" — under-counts, actual `fn w4_f28_*` = 15) |

## 2. Per-line-kind table (C# order, `Main.cs:20631-21343`)

KEEP = the implementation the merge should keep (both land ⇒ keep-one, named).
"e16" = `ui/tooltip.rs` (+`tooltip_sets.rs`); "f28" = `ui/tooltip.rs` +
`ui/tooltip_lines2.rs` unless noted.

| # | Line-kind | C# anchor | e16 file/symbol | f28 file/symbol | KEEP verdict |
| --- | --- | --- | --- | --- | --- |
| 0 | hover name + rare colour (expert −12 disco / −13 master pulse) | :20633, :20328-20332, :20557-20567 | tooltip.rs `hover_name` + `expert_only` census + static master | tooltip.rs + lines2 `tooltip_name_color` + App animator | **f28** (animated master; expert from `d.expert` column) |
| 1 | loadout-share arm (flag/flag2, SharedFrom/Shared) | :20638-20654 | tooltip.rs latches + `tooltip_sets::can_loadout_share_at` | DEFERRED (module doc) | **e16** (latches + ctx 8-12 live; SharedFrom unreachable both) |
| 2 | favorited tip[56]/[57] + bank-blocked notice | :20655-20663 | tooltip.rs + `blocked_from_transfer_into_chest` (bank idx 0/3) | lines2 `favorite_rows` (raw chest −2/−5) | **f28 helper**, e16 ctx wiring (equivalent mapping; f28 `open_chest` pre-capture fix is real) |
| 3 | social tip[61] (`!vanity && !hasVanityEffects`) | :20664-20668 | tooltip.rs + `has_vanity_effects` 21-write census | lines2 `social_row` + `d.has_vanity_effects` column | **f28** (column; census retires) |
| 4 | damage row + gates (notAmmo/coin-gun/rapid/×mult +5e-6) | :20669-20700 | tooltip.rs inline gate, `has_coin_gun` 58-slot scan, `prefixed_stats_of` | lines2 `damage_row_gate`/`damage_row_value`, `.take(58)` over 50 slots | **f28 helpers + e16 coin-gun scan** (f28 slice misses coin/ammo rows 50-57) |
| 5 | crit row (`GetVisualCritChance`, Revolver 2269 clamp) | :20701-20716 | tooltip.rs inline visual crit over `ps.crit` | lines2 `visual_crit_chance`/`crit_row_value` | **f28** (same maths; helper seam) |
| 6 | use-time tiers (summon-whip gate) | :20717-20752 | tooltip.rs inline `tr_core::whip::is_whip` | lines2 `use_time_row_gate` | **f28 helper** (identical) |
| 7 | knockback word (minionKB, ×1.1 INT trunc, Psycho-Knife, f64 1.5) | :20753-20802 (+ :20333-20350) | tooltip.rs inline chain over `ps.knock_back` | lines2 `display_knockback` | **f28 helper** (identical chain; +no-player stake disjunct) |
| 8 | fishing power / bait rows | :20804-20815 | tooltip.rs **`PrecentFishingPower`** | tooltip.rs `RecentFishingPower` | **e16** (MUST-SALVAGE typo fix; f28 key renders nothing) |
| 9 | equipable (+ `!flag` gate, shop tip[60] 854/3035) | :20816-20827 | tooltip.rs `!loadout_share_flag` + `shop_open` | tooltip.rs + lines2 `equipable_row(kind, npc_shop)` (no `!flag`) | **e16 gate + f28 helper** (flag only matters once loadout head lands — e16's does) |
| 10 | tileWand tip[52] + consumed name | :20828-20832 | tooltip.rs + `tooltip_sets::tile_wand` 6-write table | lines2 `tile_wand_row(d.tile_wand)` | **f28** (column; table retires) |
| 11 | quest / vanity rows | :20833-20841 | tooltip.rs (base) | tooltip.rs (base) | either (untouched both) |
| 12 | defense/pick/axe×5/hammer/tileBoost | :20843-20875 | tooltip.rs (base) | tooltip.rs (base) | either (untouched both) |
| 13 | healLife (+3001 `RestoresLifeRange` healLife..120) / healMana | :20876-20893 | tooltip.rs inline `text_args` | lines2 `restores_life_row` (`format_net` 2-arg) | **f28 helper** (identical output) |
| 14 | buff/healing/mana-healing/mount chain | :20894-20913 | tooltip.rs (base) | tooltip.rs (base) | either |
| 15 | mana cost (spaceGun skip, prefixed mana) | :20914-20918 | tooltip.rs over `ps.mana` | tooltip.rs over `hover_stats` mana | **f28** (with §3 stats seam note) |
| 16 | place/ammo/consumable ladder (alt-use/notAmmo/extractinator) | :20919-20936 | tooltip.rs + `tooltip_sets` tables | lines2 `place_row_active`/`ammo_row_active`/`consumable_row_active` | **f28 helpers**; VERIFY `wall.is_some()` vs C# `createWall > 0` (e16 checks `w > 0`) |
| 17 | material | :20937-20941 | tooltip.rs (base) | tooltip.rs (base) | either |
| 18 | Wireable (trigger∪mechanism, tile-105 statue carve-out) | :20942-20946 | tooltip.rs + `tooltip_sets::wireable` (hand-copied tile sets) | lines2 inline over generated `SET_IS_A_TRIGGER`/`SET_IS_A_MECHANISM` bitsets | **f28** (generated bitsets regen-proof); statue set dedupe to ONE table |
| 19 | Container (tile 21/467) | :20947-20951 | tooltip.rs inline | lines2 inline | either (identical) |
| 20 | WireTrigger (tile 441/468) | :20952-20956 | tooltip.rs inline | lines2 inline | either (identical) |
| 21 | per-item ToolTip lines + cursed-by-Plantera line-0 swap | :20957-20972 | tooltip.rs + `uses_cursed_by_plantera_tooltip` | lines2 `append_second_half` head | **f28** (identical swap; set table dedupe) |
| 22 | anniversary Wizard Hat 238 | :20973-20977 | tooltip.rs `ctx.anniversary` | lines2 `world.tenth_anniversary` | **f28** (field rename at merge, §4) |
| 23 | FTW Crispy Honey 1127 | :20978-20982 | tooltip.rs | lines2 | either (identical) |
| 24 | Mechdusa 556/557/544 last-line replace; 5334 double-walk | :20983-20999 | tooltip.rs `pop` + `last_mut` replace | lines2 `pop;pop;push` | **f28** (literal `numLines--` walk; net output identical — MUST-SALVAGE shape) |
| 25 | DD2 poppers misc[104] | :21000-21004 | tooltip.rs + `dd2_needs_etherian_mana`, **t1‖t2‖t3** gate | lines2 `is_dd2_crystal_gear`, `dd2_won` gate | **f28 row + e16 gate derivation** (`DD2Event.cs:89-99` = T1‖T2‖T3) |
| 26 | WellFed misc[40] (expert) | :21005-21009 | tooltip.rs + `is_well_fed` | lines2 `is_well_fed` | either (identical {26,206,207}) |
| 27 | buff duration (int `/60` first, banker's) | :21010-21015 | tooltip.rs inline | lines2 inline | either (identical) |
| 28 | yoyo " " placeholder + yoyoLogo index | :21016-21021 | tooltip.rs (index dropped — no return) | lines2 (returns `Option<usize>` index) | **f28** (index kept for the OneDropLogo draw arm) |
| 29 | Expert row (`item.expert`) | :21022-21026 | tooltip.rs + `expert_only` 25-write census | lines2 `d.expert` column | **f28** (column; census retires) |
| 30 | Master row (INSTANCE rare == −13) | :21027-21031 | tooltip.rs `ps.rare` | lines2 `hover_stats` rare (Guarding-63 shift witnessed) | **f28** (witness pins the prefix-shift drop) |
| 31 | prefix % rows (dmg/speed−1/crit-raw/mana-INVERT/scale/velocity/oldKB) + pen/tag `{0:+0;-#}` | :21032-21151 | tooltip.rs `append_prefix_lines` + `push_percent_delta` + `signed_fmt`; oldKB = `ps.knock_back` pre-scale | lines2 `append_prefix_rows` + tr-core `apply_format_spec`; oldKB = `stats.knock_back` | **f28** (real section format; e16 `signed_fmt` retires — same output, §3) |
| 32 | accessory fixed rows 62-80 (Warding "+20 " space) | :21152-21265 | tooltip.rs match table | lines2 match table | either (identical incl. the space) |
| 33 | armor-set bonus row (grey 130/190 unworn / LimeGreen complete) | :21267-21311 | tooltip.rs `best_worn_set` + `ctx.context != 8` | lines2 `SETS.find` (array[0]) unworn / `best_worn_set` worn + `flags.worn_armor` | **f28** (unworn reads array[0] literally); VERIFY e16 `best_worn_set` row choice for the unworn text |
| 34 | share hint (flag2 && Settings.ShowLoadoutShareHint, OurFavoriteColor 190) | :21312-21317 | tooltip.rs + settings.rs key (Put :4219/Get :4553, default true Player.cs:412) | DEFERRED | **e16** (MUST-SALVAGE: key + row + round-trip witness) |
| 35 | journey creative-sacrifice rows (difficulty==3, ctx whitelist) | :21318-21333 | cited drop (constants rationale REVISED at closeout — c24 landed the gate inputs, §7) | DEFERRED (no sacrifice rows; ctx whitelist rides loadout port) | **drop-both, cited — pending port** (c24 tip `e2e50a42`: difficulty-3 + `sacrifice_numbers` now exist; e16 ctx map is the seam, §7) |
| 36 | BestiaryNotes split lines | :21334-21342 | DROP cited — SUPERSEDED (e23 writer landed, §7) | lines2 emits over `HoverFlags.bestiary_notes: Option<&'static str>` (inert until the wire) | **f28 seam** (e23 `bestiary_notes` writer EXISTS at tip `6daf6d3d`; `DrawMouseOver` :121-126 feeds `Main.HoverItem` — the remaining work is the hover WIRE, §7) |
| — | line tint byte-truncation | :20406 | tooltip.rs `draw_item_tooltip` | tooltip.rs `draw_item_tooltip` | either (identical `(byte)(c*num2)`) |
| — | shop lines (`AddShopLines`) | :20448-20555 | out of scope (W4-f37) | out of scope (S1-P-i44 chrome follow-up) | drop-both → f37 |
| — | diff 1/2 name colours | :20620-20627 | unported (no difficulty) | deferred (call passes 0) | drop-both, cited |

## 3. Base recommendation — GenItemData provision vs census

**KEEP f28's GenItemData-provisioned stats/flags as the base.** It is the
faithful AND complete approach:

- The five flags are `Item.SetDefaults`/`ResetStats` state
  (`Item.cs:48951/:48955/:48966/:48967/:49008` + the `DefaultTo*` helpers) —
  exactly what the generator already provisions for every other field. The
  census tables (`expert_only` 25 writes, `has_vanity_effects` 21 writes,
  `tile_wand` 6 writes) are hand-maintained duplicates that silently rot when
  a future wave touches `Item.cs`; the columns regenerate + FNV-pin
  (`0x1dd2_84fc_4b36_2f46`) and the two hand-written `ITEM_CORRECTIONS` rows
  carry the stock values. `notAmmo` also retires the `item.rs::not_ammo`
  duplicate (keep-one at merge).
- The e16 census approach stays ONLY for pure ID-set membership with no
  `ItemDef` column: `PlaceTileOnAltUse` (`ItemID.cs:412`), `IsWireableStatue`
  (`:1581`), `UsesCursedByPlanteraTooltip` (`:410`), `IsWellFed`
  (`BuffID.cs:43`), the yoyo/DD2 kind lists, `canLoadoutShareAt`
  (`ItemSlot.cs:395-399`), `IsBlockedFromTransferIntoChest`
  (`ChestUI.cs:1291-1302`). f28 hard-codes the first five INLINE in
  `tooltip_lines2.rs` — at merge dedupe to ONE table (keep `tooltip_sets.rs`
  as the home, or the generated bitsets where they exist: f28's
  `SET_IS_A_TRIGGER`/`SET_IS_A_MECHANISM` beat e16's hand-copied tile slices).
- **Stats seam:** keep e16's `prefix::prefixed_stats_of(def, prefix)` as the
  application primitive and re-point f28's `hover_stats` at it when the
  caller holds the row (the e16 witnesses override `ItemDef` fields — kb 9.5,
  whip projectile — which f28's registry re-read cannot see). f28's
  bare-def fallback for unregistered kinds is worth keeping as the outer
  `Option` shape.
- **Structure:** keep f28's two-module split (`tooltip.rs` first half +
  `tooltip_lines2::append_second_half`, the anchored seam) and port e16's
  loadout head / context map / settings / gates INTO it — not the reverse
  (f28 is 1407 ln + generator + format machinery; e16's unique value is
  ~200 ln of head/gate/settings logic).

## 4. `draw_hud` `TooltipWorld` param collision — resolution

Both branches append a trailing `draw_hud` param at the SAME position (after
`npc_chat_text`) and grow `TooltipContext` at the same spot — a guaranteed
conflict in `ui/mod.rs`, `app/frame/draw.rs`, `player.rs` (rapid-attack field
+ helper), `app/mod.rs`/`flow.rs` (f28-only). Resolution (one struct, one
param):

1. **Keep f28's `tooltip_lines2::TooltipWorld`** (the superset: `npc_shop`,
   `master_color`, `STOCK` const) as THE struct; rename `tenth_anniversary`
   → `anniversary` or keep f28's name and fix e16's call sites — pick f28's
   name (its witnesses + anchors already read it). Delete e16's
   `ui::tooltip::TooltipWorld`.
2. **Draw-site derivation takes e16's values where they are more faithful:**
   `downed_dd2_any = world.dd2_downed_t1 || t2 || t3` (e16; matches
   `DD2Event.DownedInvasionAnyDifficulty`, `DD2Event.cs:89-99`, which
   `Player.cs:24450-24453` latches into the .plr flag) — NOT f28's
   `dd2_won` sentinel. `expert_mode`/seeds/mechdusa/disco identical both.
   `npc_shop: shop.is_some()` (f28, filled at both tooltip call sites).
   `master_color: self.master_color` (f28 animator).
3. **Context:** keep f28's `ctx.world` + `flags` + `inventory` + `held_kind`
   + `open_chest` shape; fold e16's flat fields in: extend `HoverFlags` with
   `context: i32` (e16's full HoverRef→ctx map: grid 0 / coin 1 / ammo 2 /
   armor 8,10,9,11 / dye 12 / misc 16-20 / misc-dye 33 / trash 6 / bank 4 /
   hotbar 13 / craft 22) and keep `show_loadout_share_hint` on ctx. The
   loadout head (e16) then reads `flags.context` through
   `can_loadout_share_at`. `favorited` rides `HoverFlags` (f28) but the
   draw-site favourited lookup takes e16's map (coin/ammo favourites via
   `coin_slot_index`/`ammo_slot_index` — f28 only stamps Grid + hotbar).
4. **Coin-gun scan:** replace f28's `inventory.take(58)` (the passed slice is
   `&inventory[..]`, `INV = 50` — coin/ammo rows 50-57 unreachable) with
   e16's `has_coin_gun` bool computed over grid+`equip.coins`+`equip.ammo`
   (`inv::INV_SLOTS = 58`, `Player.cs:55894-55904`), OR pass the concatenated
   58-slot view. The bool is the smaller edit.
5. **`Player::apply_rapid_attack_bonus`:** keep f28's full `&mut` form
   (increase arm + `ResetEffects` decay); e16's read-only wrapper retires
   (the tooltip call passes `increase_bonus: false` — f28's
   `damage_row_value` already does, PRE-charge read witnessed).
6. **`format_net`:** keep f28's `apply_format_spec` (+ the `85165b8b`
   custom-vs-standard guard); retire e16's `signed_fmt` (its pen/tag
   witnesses assert the same "+10 armor penetration" strings f28's format
   produces — they survive the swap unchanged).
7. **anchors.txt:** keep all three rows, but e16's
   `tooltip.rs|fn append_prefix_lines` marker dies with the keep-one dedupe
   (f28 `append_prefix_rows` replaces it) — REPOINT the row at the surviving
   symbol at merge; never delete it to silence the canary.

## 5. MUST-SALVAGE lists

**From e16 (lost if f28 wins wholesale):**
1. `PrecentFishingPower` typo-fix (`Main.cs:20806` — f28 still reads the
   base-era wrong `RecentFishingPower`; the row renders nothing).
2. `prefix::prefixed_stats_of` (caller-row seam; §3).
3. oldKB-20333 capture discipline — both capture pre-glove-scale
   (`ps.knock_back` / `stats.knock_back`); keep ONE, witnessed by e16
   `w4_e16_the_knockback_delta_reads_the_prefix_only_oldkb`.
4. `Settings.ShowLoadoutShareHint` (key Put/Get/default + round-trip
   witness) + the whole loadout head: flag/flag2 latches,
   `UI.ItemLoadoutShared`, `!flag` equipable suppression, share-hint row
   (255,231,69,190), `can_loadout_share_at` 8-12.
5. The full HoverRef→context map + coin/ammo favourite stamps + the
   58-slot `has_coin_gun` scan + the t1‖t2‖t3 DD2 derivation (§4).

**From f28 (lost if e16 wins wholesale):**
1. `master_color`/`master_color_dir` App animator + `advance_master_color`
   + the `frame/draw.rs` tick + `flow.rs` fixture (e16 is static 1.0).
2. The Mechdusa-5334 literal double-`numLines--` walk (`pop;pop;push`) —
   net-identical to e16 but the shape the C# audit trail expects.
3. GenItemData 5-flag provision + regen + FNV re-pin + correction rows (§3).
4. `format_net` custom-section grammar + standard-specifier guard + 3
   tr-core witnesses (zero-section `{0:+0;-#;zero}`, padding, overflow).
5. Full rapid-attack form + `ResetEffects` decay; `GetVisualCritChance`
   helper; the `yoyoLogo` index return; `HoverFlags.bestiary_notes` seam;
   the `open_chest` pre-`chest::draw` capture fix (`85165b8b`).

## 6. Witness union — FINAL dedupe map (30 + 18 = 48; the Lead's "16" under-counts f28)

e16: 26 `w4_e16_*` in `tooltip.rs` + 3 in `tooltip_sets.rs` + 1 settings
(`w4_e16_the_loadout_share_hint_round_trips`). f28: **15** `w4_f28_*` in
`tooltip_lines2.rs` (the commit message + ANALYSIS say "13 witnesses" — the
actual test count is 15) + 3 in `tr-core format.rs` = 18. All WRITTEN NOT RUN
(code-only both). Overlap map (keep-both where the seam differs; dedupe
identical assertions at merge, never drop coverage):

| Topic | e16 witness | f28 witness | Merge |
| --- | --- | --- | --- |
| coin-gun damage gate | `…coin_damage_row_needs_the_coin_gun` (tip[3] RANGED, 25 dmg) | `…coin_gun_gate_and_the_range_restore` | keep e16 (58-slot) + f28 range half |
| summon/whip use-time | `…use_time_row_skips_the_summon_staffs…` | `…summon_use_time_row_needs_a_whip` | dedupe (identical) |
| quiver truncation / Psycho Knife | 2 witnesses | `…knockback_chain_truncates_and_scales` | keep-both (e16 row-level, f28 helper-level) |
| rapid attack | `…rapid_attack_bonus_scales_only_the_three_kinds` | `…rapid_attack_scale_caps_and_decays` (PRE-charge, 0.62 cap) | keep f28 + e16 set-guard |
| revolver visual crit | `…revolver_visual_crit_clamps_the_charge` | `…visual_crit_and_the_set_bonus_row` | keep-both |
| shop tip[60] | `…shop_swaps_the_discount_card_equipable_row` | `…discount_card_shopping_row` | dedupe |
| tile wand | `…tile_wand_row_names_the_consumed_material` | `…favourite_social_and_wand_rows` | keep-both (table vs column) |
| Strange Brew range | `…strange_brew_prints_the_life_range` | (in coin_gun_gate_and_range) | dedupe |
| place/ammo/consumable | `…place_ammo_consumable_gates` | `…place_ammo_consumable_ladder_repairs` | dedupe + e16 `wall>0` check |
| wiring rows | `…wiring_rows_follow_the_tile_sets` + sets `…wireable_gate_carves_out_the_statues` | `…wireable_rows_follow_the_statue_exception` | keep-both (bitsets vs table) |
| cursed Plantera | `…biome_keys_stay_cursed_until_plantera` | `…jungle_key_swaps_its_first_line…` | dedupe |
| seed rows | `…seed_rows_gate_on_their_world_flags` | `…seed_and_dd2_rows_gate_on_the_world_flags` | keep-both (e16 DD2 gate derivation) |
| Mechdusa | `…mechdusa_swap_rewrites_the_last_lines` | `…mechdusa_summon_rows_replace_the_tail` | keep f28 shape, e16 assertions |
| DD2 + well-fed | `…dd2_and_well_fed_rows` | (seed_and_dd2 + well_fed_and_duration) | dedupe |
| duration banker's | `…duration_row_rounds_bankers` | `…well_fed_and_duration_rows` | dedupe |
| yoyo/expert/master | `…yoyo_expert_and_master_rows` | `…yoyo_placeholder_and_the_expert_row` + Guarding-63 in prefix rows | keep f28 (master-shift pin) |
| prefix % deltas | `…legendary_prefix…` + `…negative_deltas_are_red…` | `…prefix_comparison_rows` | keep-both (e16 mana-invert 3-prefix matrix is deeper) |
| pen/tag signed | `…whip_prefix_prints_the_signed_pen_and_tag_rows` | format.rs `the_prefix_rows_section_format_matches_dotnet` | keep-both (output identical post-§4.6) |
| accessory fixed | `…accessory_prefixes_print_the_fixed_rows` | (in prefix_comparison) | dedupe |
| oldKB | `…knockback_delta_reads_the_prefix_only_oldkb` | (doc-level) | keep e16 |
| favourite + bank block | `…favorite_rows_and_the_bank_block` | `…favourite_social_and_wand_rows` | keep-both (bank idx vs raw chest) |
| loadout suppression | `…loadout_share_arm_suppresses_the_equipable_row` | — | **e16 only** |
| social | `…social_row_skips_the_vanity_and_info_kinds` | (favourite_social_and_wand) | keep-both |
| armor set | `…armor_set_row_grey_unworn_green_complete` | `…visual_crit_and_the_set_bonus_row` | keep-both |
| sets tables | 3 `tooltip_sets` tests | — | e16 only (retire rows whose tables dedupe away) |
| settings key | `…loadout_share_hint_round_trips` | — | **e16 only** |
| format grammar | — | 3 tr-core tests | **f28 only** |

**FINAL verdicts (closeout, f28c reads these literally):**

- **Dedupe (retire the e16 twin, f28 witness carries the topic — 8 e16 fns):**
  summon/whip use-time, shop tip[60], Strange Brew range (assertions ride
  f28 `…coin_gun_gate_and_the_range_restore`), cursed Plantera, DD2+well-fed
  (`…dd2_and_well_fed_rows` — its DD2-gate assertions MOVE into the salvaged
  t1‖t2‖t3 derivation test before retiring), duration banker's, accessory
  fixed rows (inside f28 `…prefix_comparison_rows`).
  Plus the place/ammo/consumable pair: dedupe onto f28
  `…place_ammo_consumable_ladder_repairs` **after adding e16's `wall > 0`
  assertion** (§2 row 16 VERIFY) — never drop that check with the twin.
- **Keep-both (different seam/depth):** quiver+Psycho-Knife (2 e16 row-level
  vs f28 helper-level), revolver visual crit, tile wand row-level (the
  sets-table twin retires, §3), wiring rows (statue carve-out keeps its
  `tooltip_sets` home), seed rows (e16 pins the DD2 derivation), prefix %
  deltas (e16 mana-invert 3-prefix matrix is deeper), pen/tag signed
  (output identical post-§4.6), favourite+bank block, social, armor set.
- **Keep f28 shape, port e16 assertions:** coin-gun damage gate (e16 tip[3]
  RANGED-25-dmg assertions + the 58-slot scan land on f28's gate+range
  witness), rapid attack (f28 PRE-charge/cap/decay + e16's three-kind
  set-guard assertions), Mechdusa (`pop;pop;push` shape, e16 last-line
  assertions), yoyo/expert/master (f28's Guarding-63 master-shift pin wins;
  e16 `…yoyo_expert_and_master_rows` retires only after its expert-set
  assertions are covered by the `d.expert` column checks).
- **e16-only survivors (UNIQUE-SALVAGE, must land on f28):**
  `…loadout_share_arm_suppresses_the_equipable_row`,
  `…knockback_delta_reads_the_prefix_only_oldkb`,
  `…loadout_share_hint_round_trips` (settings), the fishing typo-fix
  (currently assertion-free — ADD a `PrecentFishingPower` witness at f28c),
  `tooltip_sets` statue/alt-use table tests (the tile-wand + extractinator
  halves retire with their tables into the columns).
- **f28-only survivors:** the 3 tr-core format tests
  (`the_prefix_rows_section_format_matches_dotnet`,
  `plain_and_two_arg_holes_still_format`, `zero_pad_and_optional_holes`).

Net after dedupe: ~38 distinct witnesses; the union coverage must not
shrink below the 48 assertion sets — every retired twin's assertions are
either already in the surviving f28 witness or explicitly moved above.

## 7. Drops union (both branches' cited deferrals — one list at merge)

- **Journey sacrifice rows** (:21318-21333): RE-VERIFIED at closeout
  against `origin/task/W4-c24-journey-ui` tip `e2e50a42` (landed on its
  branch, NOT in main). The old "constants-off, difficulty==3 never
  holds" rationale is **REVISED**: `App::active_player_difficulty()`
  (`app/mod.rs:2380`) can now return 3 (the char-creation Journey row,
  `Click::Difficulty(3)` witness `char_creation.rs:2660`), and the sacrifice tracker
  landed (`research.rs` `ResearchTracker::sacrifice_numbers` :94,
  `register_sacrifice` :35; `App.research` field `app/mod.rs:190`). c24's
  `draw_hud` even feeds the journey grid's hovered cell into
  `hover_item` (its `ui/mod.rs` diff) with contexts 29/34
  (`journey.rs::grid_context` :641). **The rows themselves remain
  unported on every lane** (c24 did NOT touch `ui/tooltip.rs`; no
  `CreativeSacrificeNeeded`/`ItemUnlockedByTeammate` string anywhere in
  its crates) — so the drop verdict STANDS, but it is now a *pending
  port*, not a constants-off no-op: once f28c's merged tooltip reads the
  ctx whitelist (0,1,2,3,4,32,15,6,22,35,7,5,29,34,41,42,43 — e16's
  context map is the seam) + difficulty 3 + `sacrifice_numbers`, the rows
  are directly implementable. The teammate-credit arm (:21327-21332)
  stays single-player inert (c24 `research.rs:23-24`).
- **BestiaryNotes writer** (:21334-21342): RE-VERIFIED at closeout against
  `origin/task/W4-e23-bestiary-ui-grid` tip `6daf6d3d` (landed on its
  branch, NOT in main). **The writer EXISTS**: `ui/bestiary.rs:534`
  `bestiary_notes(conditions)` ports `SetBestiaryNotesOnItemCache`
  (`UIBestiaryInfoItemLine.cs:137-156`, the `"\n"` join at :155), witnessed
  by `the_bestiary_notes_join_the_non_empty_conditions` (:2562); the hover
  gate `info_item_shows_tooltip` (:550) ports `DrawSelf` :112-119 and is
  `#[allow(dead_code)]`-marked "the drop-row hover tooltip lands with the
  tooltip work". C# `DrawMouseOver` (:121-126) sets
  `Main.HoverItem = _infoDisplayItem` — so e16's DROP note is SUPERSEDED:
  keep f28's `HoverFlags.bestiary_notes` seam; the remaining work is the
  WIRE (e23's notes string → the hover item → the tooltip flags), not the
  writer. The seam stays inert until that wire lands.
- **OneDropLogo draw arm** (:20408-20439): both emit the " " placeholder;
  f28 returns the row index — the texture pass (5-copy shadow + tint) is
  render work, deferred.
- **loadout-SharedFrom** (:20640-20643): `GetSharedLoadoutItem` unported —
  unreachable in e16's live arm, deferred in f28; rides the loadout port.
- **shop-f37** (`AddShopLines` :20448-20555 + `NewCraftingUI.AddTooltipLines`
  :20363-20366): out of scope both — W4-f37 shop lane / crafting UI port.
- **diff 1/2 name colours** (:20620-20627): the tooltip call passes 0;
  medium/hardcore colours ride the difficulty port.
- **rapid-attack use-site grant** (`Player.cs:44291`): no writer yet both;
  f28's decay + increase arm are live machinery waiting on the melee lane.

## 8. Merge order — FINAL (f28c executes on the f28 branch)

1. **f28 is the base** (`origin/task/W4-f28-tooltip-lines-2` tip
   `85165b8b`: generator + format + two-module structure). f28c salvages
   INTO it there — e16 does NOT rebase; its branch stays banner-marked
   until the salvages land, then retires.
2. **The exact e16 salvage-hunk list (f28c reads this; source = e16 tip
   `1c15c74c`, banner markers `UNIQUE-SALVAGE`):**

   | # | e16 source (file · symbol · C# anchor) | f28 destination | Note |
   | --- | --- | --- | --- |
   | S1 | `ui/tooltip.rs` · loadout head latches + `UI.ItemLoadoutShared` arm (`:20636-20654`) | `tooltip_lines2::append_second_half` head / `tooltip.rs` first half, reading `HoverFlags.context` | flag suppresses the equipable row (S3); flag2 feeds S4 |
   | S2 | `ui/tooltip_sets.rs` · `can_loadout_share_at` (ctx 8-12, `ItemSlot.cs:395-399`) | `tooltip_sets.rs` stays the ONE table home | pure ID-set, survives §3 |
   | S3 | `ui/tooltip.rs` · `!loadout_share_flag` gate on the equipable row (`:20816`) | wrap f28 `equipable_row(kind, npc_shop)` call (`tooltip_lines2.rs:776`) | f28 helper itself KEEP |
   | S4 | `ui/tooltip.rs` · share-hint row (`:21312-21317`, tint 255,231,69,190) + `settings.rs` `show_loadout_share_hint` (field/Default/parse arm/to_json row) | `append_second_half` tail + `settings.rs` | Put `Main.cs:4219` / Get `:4553` / default true `Player.cs:412`; round-trip witness rides |
   | S5 | `ui/tooltip.rs` · `PrecentFishingPower` typo-fix (`Main.cs:20806`) | f28 `tooltip.rs:401` — replace `RecentFishingPower` | ADD a witness (e16 has none for this row) |
   | S6 | `prefix.rs` · `prefixed_stats_of(def, prefix)` | keep as the application primitive; re-point f28 `hover_stats` (`tooltip_lines2.rs:333`) at it when the caller holds the row | keep f28's bare-def fallback as the outer `Option` |
   | S7 | `ui/tooltip.rs` · `old_kb = ps.knock_back` capture BEFORE the glove/buff scales (`:20333`) | f28 prefix block already captures `stats.knock_back` pre-scale — keep ONE, pin with e16 `…prefix_only_oldkb` witness | discipline, not new code |
   | S8 | `ui/mod.rs` · full HoverRef→ctx map + coin/ammo favourite stamps (`ItemSlot.cs:3184-3198`) | extend f28 `HoverFlags` with `context: i32`; draw-site favourited lookup takes e16's map | grid 0 / coin 1 / ammo 2 / armor 8,10,9,11 / dye 12 / misc 16-20 / misc-dye 33 / trash 6 / bank 4 / hotbar 13 / craft 22 |
   | S9 | `ui/mod.rs` · 58-slot `has_coin_gun` bool (grid + `equip.coins` + `equip.ammo`, `Player.cs:55894-55904`) | replace f28 `inv.iter().take(58)` (`tooltip_lines2.rs:761`, a 50-slot slice) | the bool is the smaller edit (§4.4) |
   | S10 | `app/frame/draw.rs` · `downed_dd2_any = dd2_downed_t1‖t2‖t3` (`DD2Event.cs:89-99`) | f28 `TooltipWorld` draw-site derivation — replace the `dd2_won` sentinel | §4.2 |
   | S11 | `ui/tooltip.rs` · place gate `wall.map_or(false, \|w\| w > 0)` (`createWall > 0`) | VERIFY f28 `place_row_active`'s `wall.is_some()`; take `> 0` if it differs | §2 row 16 |
   | S12 | witnesses: `…loadout_share_arm_suppresses_the_equipable_row`, `…knockback_delta_reads_the_prefix_only_oldkb`, `…loadout_share_hint_round_trips`, the mana-invert matrix assertions, the DD2-gate assertions, the coin-gun tip[3] assertions, the `wall > 0` assertion | port per the §6 FINAL map | never shrink below the 48 assertion sets |
   | S13 | `anchors.txt` · the `fn append_prefix_lines` row | REPOINT at f28 `append_prefix_rows` (`tooltip_lines2.rs:561`) | never delete to silence the canary (§4.7); the `tooltip_sets::wireable` row survives (statue carve-out stays) |

   Dedupe at the same time (§3): f28's inline ID-set copies fold into
   `tooltip_sets.rs`; the four census tables + `signed_fmt` + the
   read-only rapid-attack wrapper + e16 `TooltipWorld` retire (all
   banner-marked on e16). `Settings.ShowLoadoutShareHint` to_json row
   order: keep e16's slot (after `SettingsEnabled_OpaqueBoxBehindTooltips`,
   before `MinimapFrame`) — the key-order witness reads it.
3. **Deferred-lane verdicts are FROZEN** (§7 re-verified): c24 `e2e50a42`
   and e23 `6daf6d3d` are on their branches, not main; f28b never
   appeared. The journey rows stay a pending port; the bestiary seam
   stays inert until the hover wire.
4. Run the witness union (§6) on the merged f28 branch — both lanes are
   WRITTEN NOT RUN; the merged branch is the first place they execute.
   e16's branch + worktree retire only after every S-hunk is verified on
   f28 (banner grep: no `UNIQUE-SALVAGE` marker left unported).
