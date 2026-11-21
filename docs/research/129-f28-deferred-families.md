# 129 — The f28 deferred families: census + closure (research/128 §7 follow-up)

**Branch:** `task/W4-f28-tooltip-lines-2` · **Mode:** CODE-ONLY (git + file
reads; witnesses WRITTEN NOT RUN). Research/128 (the e16 × f28 two-way
reconcile, `docs/research/128-tooltip-reconcile.md` on
`origin/task/W4-e16-item-tooltip-lines` @ `c5a2db32`) listed five deferred
families with PROVISIONAL verdicts because the f28b tip did not exist at
probe time. This note re-censuses each against the C# and records the
closure state on THIS branch (the f28 lane + the concurrent session's
commits both landed here).

## The census

| Family | C# anchor | 128 §7 verdict (provisional) | State on this branch | Evidence |
| --- | --- | --- | --- | --- |
| crafting-tooltip lines | `NewCraftingUI.AddTooltipLines` `NewCraftingUI.cs:541-549`, call `Main.cs:20363-20366` | drop-both → crafting UI port | **PORTED (seam live, writer dormant)** — `craft::add_crafting_tooltip_lines` appends the ONE pink (255,140,160) row LAST after GetLinesInfo + shop lines; `GetReasonForRecipeNotAvailable` (`:530-539`) = `Lang.inter[22] + missing-objects` or `GameUI.NotEnoughMaterials`; `Recipe.PlayerMeetsEnvironmentConditions`' missing list (`Recipe.cs:322-364`) in `craft::missing_environment`. The `TooltipContext.crafting_missing_reason` seam stays `None` at the draw sites — the port's grid lists craftable rows only (no persistent `_recipeListLookup`), so no unavailable recipe is hoverable until the recipe-list port | commit `2d83ed345`; witnesses `w4_f28_the_missing_environment_lists_the_gates_in_order` / `…the_reason_picks_the_environment_or_the_materials_row` / `…the_crafting_row_appends_pink_and_skips_air` (`craft.rs`) |
| rapid-attack use-site grant | `Player.cs:44291` (`ProcessHitAgainstNPC`), `ApplyRapidAttackBonus` `:18251-18273`, `GetWeaponDamage` `:53421-53429`, decay `:18278-18285` | no writer yet both; f28 machinery waits on the melee lane | **PORTED** — `resolve_melee_strike` truncates `GetWeaponDamage` to INT (+5E-06f epsilon, the `damage <= 0` passthrough kept) then `p.apply_rapid_attack_bonus(dmg, held.kind, !immortal)` BEFORE the cane-1 arm (`:44292` order); {2608, 3349, 672} charge +0.12 capped 0.62; the immortal dummy never charges; the −0.005/tick `ResetEffects` decay was already in (`player.rs`). The tooltip's PRE-charge read (`Main.cs:20675`) now reads a live field | commit `e4558e73c`; witness `w4_f28_the_melee_hit_charges_the_rapid_attack_bonus` (`world_sim/editing.rs`) |
| shop-lines f37 co-lane | `MouseText_DrawItemTooltip_AddShopLines` `Main.cs:20448-20555`, call `:20362` | out of scope both → W4-f37 | **PORTED on this branch** (the concurrent lane closed it ahead of f37) — `shop::add_shop_lines`: the `npcShop <= 0 ‖ value < 0 ‖ coins 71-74` early return (`:20450`), the `isAShopItem‖buyOnce ? buy : sell` pick (`:20455`), the SELL fifth + sellback-memo correction (`:20469-20483`), the coin-tier split, tip[49]/[50]/[51]; `draw_hud` takes `ShopTooltipState` (ShiftInUse + the sellback memo), the shop-grid hover stamps context 15 (`buy`, `ItemSlot.cs:3195-3197`). SKIP cited: the `shopSpecialCurrency` arm (`:20456-20460`) — no `CustomCurrencyManager` port | commit `4c2e47a20`; witnesses `w4_f28_the_shop_line_gates` / `…the_buy_row_splits_the_coin_tiers` / `…the_sell_row_refunds_the_memo` (`shop.rs`). **f37 merge note:** keep-one — this implementation IS the f37 lane's `AddShopLines` scope; f37 should re-point, not re-port |
| diff 1/2 name colours | `GetItemNameColor` `Main.cs:20620-20627`, stock `:66765-66766` | drop-both cited (call passes 0) | **PORTED (arms live, source dormant)** — `tooltip_name_color(rare, expert, diff, world)` applies mcColor (1, 0.6, 0) / hcColor (1, 0.15, 0.1) as the LAST overrides after every rarity arm; `TooltipContext.diff` carries `MouseTextCache.diff` (`:20172`). The item tooltip call passes 0 (`:45419`); the only non-zero source is the stealthed-remote-player hover (`DrawMouseOver` `:42934`, unported) — so the arms are inert today but 1:1 | commit `3d2cec11b`; witness `w4_f28_the_diff_colours_override_every_rarity` (`tooltip_lines2.rs`) |
| yoyo-logo draw arm | `Main.cs:20408-20439` (5-copy pass), row `:21016-21021`, texture `AssetInitializer.cs:310` | render work deferred; f28 returns the index | **PORTED** — `OneDropLogo` packs into the static atlas half (`Lookups.one_drop_logo`); `item_tooltip_rows` exposes the `yoyoLogo` index `append_second_half` returns; `draw_item_tooltip` paints four black copies at ±1 px + the `mouseTextColor`-tinted copy over that row INSTEAD of the " " text; the row height still advances (`:20444` runs for every row) | commit `d3599cc32`; witness `w4_f28_the_one_drop_logo_packs` (`assets.rs`) |

## What stays deferred (the residual, all cited in ANALYSIS)

- **loadout SharedFrom SOURCE**: the row itself landed (`39babafa7` —
  `HoverFlags.shared_from_loadout`, `Main.cs:20641-20644`); its writer
  `GetSharedLoadoutItem` (`ItemSlot.cs:2839-2846` →
  `GetEffectiveArmor`/`GetEffectiveDye`) needs the g55 loadout system.
- **Journey research SOURCE**: the rows landed (`39babafa7` —
  `research::research_tooltip_rows`, the 17-context whitelist +
  difficulty-3 gate + `CreativeSacrificeNeeded`/teammate-credit arms,
  `Main.cs:21318-21333`); the c24 App fields (`TooltipContext.difficulty`
  /`research`) stay 0/`None` until the creative tracker ports.
- **BestiaryNotes writer** (`:21334-21342`): the seam
  (`HoverFlags.bestiary_notes`) is live; the only writer
  (`UIBestiaryInfoItemLine.cs:137-156`) is unported (the e23 branch's
  `ui::bestiary::bestiary_notes` exists — the hover WIRE is the work).
- **diff-hover source**: the colour arms are live; no hover passes a
  non-zero `diff` until the stealthed-remote-player hover ports.
- **crafting-reason writer**: the row seam is live; the persistent
  recipe-list port owns the unavailable-recipe hover.

Witness count note: `39babafa7` (the co-lane contracts commit) landed
between this census's first draft and its commit — the table's five
families were already closed by then; the SharedFrom/Journey rows moved
from "deferred" to "ported, source dormant" in the same window.

## Retire-phase updates (the e16c-128-FINAL receipts)

Read against `origin/task/W4-e16-item-tooltip-lines` tip `1dc8520f3`
(research/128 STATUS FINAL — the S1-S13 hunk list, §6 witness-dedupe map,
§7 re-verified drops). The per-item receipt table + the FINAL GetLinesInfo
census + the merger instruction live in
`docs/research/130-f28-retire.md` (this branch).

- **Co-lane probe (retire phase):** at the first probe the worktree carried
  live f28b WIP (`research.rs`/`tooltip_lines2.rs`/`anchors.txt`/
  `ANALYSIS.md` dirty, mtimes minutes old — LEFT untouched per the live-lane
  rule). The co-lane then committed and pushed it itself as `e9e37216a`
  (`Colors.JourneyMode` = (255,120,187), the XNA BYTE-Lerp truncation —
  re-verified against `ID/Colors.cs:113`: `Lerp(HotPink, White, 0.1f)`);
  the worktree is clean at the retire HEAD. No commit-rescue was needed.
- **Journey rows — verdict UPDATE (128 §7 pending-port → ROWS PORTED,
  source dormant):** `39babafa7` landed `research::research_tooltip_rows`
  (the 17-context whitelist `research_row_context`, the difficulty-3 gate,
  `CreativeSacrificeNeeded` remaining-count in the corrected JourneyMode
  colour, the ctx-29 teammate-credit arm) + `ResearchTracker::sacrifice_numbers`
  (the c24 accessor, `ItemsSacrificedUnlocksTracker.cs:75-86`), witnessed by
  `w4_f28_the_co_lane_rows_ride_their_contract_fields`. The residual is the
  SOURCE: `TooltipContext.difficulty`/`research` stay 0/`None` at the draw
  sites until c24's `active_player_difficulty` + `App.research` wire lands;
  the teammate credit stays `None` single-player.
- **Bestiary rows — verdict UPDATE (128 §7 e16-DROP superseded → seam
  PORTED, writer on e23):** the `HoverFlags.bestiary_notes` seam emits the
  `\n`-split rows LAST with the `IsNullOrWhiteSpace` guard
  (`tooltip_lines2.rs` `append_second_half` tail); the co-lane witness pins
  the split order + the whitespace skip. The e23 writer
  (`ui::bestiary::bestiary_notes`, tip `6daf6d3d`) exists on its branch —
  the remaining work is the hover WIRE, not the rows.
- **SharedFrom row — PORTED, source dormant:** `39babafa7` (the
  `UI.ItemLoadoutSharedFrom` 1-based arm beats the favourited arm, witnessed);
  the g55 `GetSharedLoadoutItem` writer is the residual (above).

## The e16 MUST-SALVAGE folds (this session, per 128 §4/§5)

1. `PrecentFishingPower` typo key (`Main.cs:20806`) — `57dd02078`.
2. DD2 gate = `World::downed_dd2_any()` T1‖T2‖T3 (`DD2Event.cs:89-99`) —
   `9ad88616f`.
3. Coin-gun 58-slot scan (`ui::has_coin_gun`, `Player.cs:55894-55904`) —
   `868a98faa`.
4. `prefix::prefixed_stats_of` caller seam + oldKB-20333 parameter —
   `4c5482cb9`.
5. `Settings.show_loadout_share_hint` (Put `:4219`/Get `:4553`, default
   true `Player.cs:412`) — `a83122391`; the loadout head (flag/flag2,
   `UI.ItemLoadoutShared`, `!flag` equipable suppression, share-hint row,
   `can_loadout_share_at` 8-12, the full HoverRef→ctx map + coin/ammo
   favourites) — carried by `4c2e47a20`/`3d2cec11b` (the concurrent
   session committed the shared files with these hunks in).
6. The `createWall > 0` place-gate VERIFY (128 §2 row 16) — `f0826fe3d`.

Witness union note: `w4_f28_*` now counts 33 across 9 files (grep
`fn w4_f28_`) + 3 unnamed-prefix format tests in `tr-core format.rs`
(`w4_f28_tests` module). All WRITTEN NOT RUN — the merged branch is the
first place they execute (128 §8).
