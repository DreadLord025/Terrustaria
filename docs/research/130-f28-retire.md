# 130 — The f28 retire receipts: e16c-128-FINAL S1-S13 + the GetLinesInfo census

**Branch:** `task/W4-f28-tooltip-lines-2` · **Mode:** CODE-ONLY (git + file
reads + a marker-census script; witnesses WRITTEN NOT RUN, no cargo).
**Inputs:** `origin/task/W4-e16-item-tooltip-lines` tip `1dc8520f3`
(research/128 STATUS FINAL, §8.2's S1-S13 hunk list — sources = e16 tip
`1c15c74c`), this branch's HEAD `e9e37216a` (the co-lane's JourneyMode
byte-Lerp fix, pushed; the earlier f28b WIP probe found the lane LIVE and
left it — it committed itself, see research/129's retire-phase note).

## 1. The S1-S13 receipt table (per-item, verified at HEAD `e9e37216a`)

Every row read against the working files at HEAD (line numbers are HEAD
line numbers). "Executed-by" names the commit that landed the hunk on this
branch (f28c = this lane's salvage rounds; co-lane = the concurrent f28b
session's commits on the same branch).

| # | Hunk (128 §8.2) | Verdict | Evidence at HEAD | Executed-by |
| --- | --- | --- | --- | --- |
| S1 | loadout head latches + `UI.ItemLoadoutShared` arm (`:20636-20654`) | **EXECUTED** | `tooltip.rs:363-399`: `loadout_share_flag`/`share_hint_flag` latches, ctx≥0 + `can_loadout_share_at`, SharedFrom → favourited → flag2 if/else-if order | co-lane `4c2e47a20`/`3d2cec11b` (carried), `39babafa7` (SharedFrom arm) |
| S2 | `can_loadout_share_at` ctx 8-12 (`ItemSlot.cs:395-399`) | **EXECUTED (home moved)** | `tooltip_lines2.rs:128` `pub fn can_loadout_share_at` — the ONE table lives in lines2, NOT a resurrected `tooltip_sets.rs` (that file never existed on f28; the merge keeps f28's structure per §8.1). Witnessed inside `…loadout_share_arm_suppresses_the_equipable_row` (:1604-1608: 8-12 in, 7/13/−1 out) | co-lane `4c2e47a20` |
| S3 | `!flag` gate on the equipable row (`:20816`) | **EXECUTED** | `tooltip.rs:516` `if !loadout_share_flag && (…)` wraps the `equipable_row(kind, npc_shop)` call (:528); the helper itself stays f28's (`tooltip_lines2.rs:859`) | co-lane `4c2e47a20` |
| S4 | share-hint row (`:21312-21317`, 255,231,69,190) + settings key | **EXECUTED** | row: `tooltip_lines2.rs:605-610` (`share_hint_flag && show_loadout_share_hint`, OurFavoriteColor α190); key: `settings.rs:251` field, `:331` default true (`Player.cs:412`), `:669` parse arm (Get `:4553`), `:733` to_json row AFTER `SettingsEnabled_OpaqueBoxBehindTooltips` BEFORE `MinimapFrame` (the §8.2 key-order rider); round-trip witness `settings.rs:1520` | `a83122391` (key) + co-lane `4c2e47a20` (row) |
| S5 | `PrecentFishingPower` typo-fix (`Main.cs:20806`) | **EXECUTED** | `tooltip.rs:497` reads `PrecentFishingPower`; `:500` argument-less `BaitRequired` (`:20808`); no `RecentFishingPower` read remains; witness ADDED per the §6 rider: `…the_fishing_rows_read_the_vanilla_typo_key` (`tooltip_lines2.rs:1509`) | `57dd02078` |
| S6 | `prefixed_stats_of` caller seam; re-point `hover_stats` | **EXECUTED** | `prefix.rs:402` `prefixed_stats_of(def, prefix)` applies over the CALLER's row (prefix==0 passthrough :403-418); `tooltip_lines2.rs:372-374` `hover_stats(d: &ItemDef, prefix)` delegates to it — the registry re-read is gone; the registry `Option` wrapper stays as `prefixed_stats`; witnessed `…the_caller_row_seam_lets_an_overridden_knockback_reach_the_word_tier` (:1527, kb 9.5 override reaches the word tier) | `4c5482cb9` |
| S7 | oldKB-20333 capture discipline (keep ONE, e16 witness pins) | **EXECUTED** | `tooltip.rs:356` `let old_kb = stats.knock_back` BEFORE the display chain; rides `append_second_half(old_kb)` (:651) → `append_prefix_rows` (:655) → the `:21124` compare (:729); e16's witness ported verbatim-in-spirit: `…the_knockback_delta_reads_the_prefix_only_oldkb` (:1553, +15% prefix-only, no +130% glove leak, word tier reads 12.65) | `4c5482cb9` |
| S8 | full HoverRef→ctx map + coin/ammo favourite stamps | **EXECUTED** | `HoverFlags.context: i32` (`tooltip_lines2.rs:86`); `ui/mod.rs:1240-1294`: grid 0 / coin 1 / ammo 2 (favourites via `coin_slot_index`/`ammo_slot_index`), armor 8/10/9/11 + worn/social stamps, dye 12, misc 16-20 (MISC_CONTEXT order), misc-dye 33, trash 6, bank 4, hotbar 13; craft 22 at the crafting call site (:1063); shop 15 (:1244) | co-lane `3d2cec11b` (map) + `4c2e47a20` (shop stamp) |
| S9 | 58-slot `has_coin_gun` bool replaces `.take(58)` | **EXECUTED** | `ui/mod.rs:1366` `fn has_coin_gun(&[ItemSlot; INV], &EquipSlots)` over grid+`equip.coins`+`equip.ammo` (`Player.cs:55894-55904`, stack>0 guard); `damage_row_gate` takes the bool (`tooltip_lines2.rs:842-846`); no `.take(58)` remains; witness `ui/mod.rs:1418` (coin cell 50, ammo cell 57, empty stack, grid arm) | `868a98faa` |
| S10 | `downed_dd2_any` = T1‖T2‖T3 derivation replaces `dd2_won` | **EXECUTED** | `tr-core world.rs` `pub fn downed_dd2_any()` (`DD2Event.cs:89-99`); draw-site `app/frame/draw.rs:1812` `downed_dd2_any: world.downed_dd2_any()` into `TooltipWorld` (:1801); gate `tooltip_lines2.rs:519`; witness `world.rs:1476` `…the_dd2_any_getter_is_the_three_tier_disjunction` | `9ad88616f` |
| S11 | place gate `createWall > 0` VERIFY | **EXECUTED** | `tooltip_lines2.rs:886-888` `places()`: `d.wall.map_or(false, \|w\| w > 0) \|\| d.tile.is_some() \|\| kind == 849` — the shared helper feeds all three ladder gates (:890-903); the `Some(0)` admission is refused, matching `:20919` | `f0826fe3d` |
| S12 | the witness-union riders (§6 FINAL map) | **EXECUTED** (two riders closed in THIS retire round, below) | see §2 | salvage rounds + this round |
| S13 | anchors `fn append_prefix_lines` row REPOINT | **EXECUTED (adapted)** | f28's anchors.txt never carried e16's `tooltip.rs\|fn append_prefix_lines` row (e16-only file history); the surviving-symbol coverage exists as `prefix.rs\|pub fn prefixed_stats_of` (line 900) + the `tooltip_lines2.rs\|pub fn append_second_half` row (899) naming the prefix block. No canary row deleted; all 10 tooltip-related markers script-verified present at HEAD (see §4) | `6fc53d5c5`/`1b6d45a3e` (rows), this note records the verdict |

**S12 detail (the §6 FINAL verdicts, per bullet):**

- *Dedupe — retire the e16 twin:* all 8 named e16 fns are absent from f28
  (they never landed here; the f28 witness carries each topic):
  summon/whip use-time → `…summon_use_time_row_needs_a_whip` (:1327);
  shop tip[60] → `…discount_card_shopping_row` (:1458); Strange Brew range
  → inside `…coin_gun_gate_and_the_range_restore` (:1318-1320); cursed
  Plantera → `…jungle_key_swaps_its_first_line…` (:1090); DD2+well-fed →
  `…seed_and_dd2_rows…` (:1154-1163) + `…well_fed_and_duration_rows`
  (:1171) — the DD2-gate assertions MOVED into the salvaged derivation
  test `world.rs:1476` before the twin retired, per the rider; duration
  banker's → `…well_fed_and_duration_rows` (:1184-1192, 1.5→2 and 0.5→0);
  accessory fixed → inside `…prefix_comparison_rows` (:1362-1372).
  Place/ammo/consumable → `…place_ammo_consumable_ladder_repairs` (:1274)
  **with the `wall > 0` assertion ADDED this round** (the §2-row-16 rider —
  `f0826fe3d` landed the gate but no witness pinned the `Some(0)` refusal;
  closed below).
- *Keep-both:* quiver+Psycho-Knife — the e16 row-level pair's assertions
  ride f28's `…caller_row_seam…` (:1527, the 9.5-override quiver truncation
  IS e16's row-level fixture) + `…knockback_chain_truncates_and_scales`
  (:1478, helper level); revolver visual crit (:1421, clamp 200→100 =
  e16's clamp assertion); tile wand (:1261-1264, column not table); wiring
  statue carve-out (:1074, the sets-table twin retires with the table —
  the carve-out lives in `is_wireable_statue`); seed rows (:1134); prefix
  % deltas (:1342 + the mana-invert matrix ADDED this round, below);
  pen/tag signed (:1373-1378 + the 3 tr-core format tests); favourite+bank
  block (:1229-1250, bank −2/−5 vs e16's idx 0/3 — equivalent mapping);
  social (:1251-1260); armor set (:1436-1451 grey-190 unworn / LimeGreen
  complete).
- *Keep f28 shape, port e16 assertions:* coin-gun — e16's tip[3]
  RANGED-25-dmg assertions are IN `…coin_gun_gate_and_the_range_restore`
  (:1313-1317 `"25{damage}"` with `damage = tip(3)`) + the 58-slot scan
  witness (`ui/mod.rs:1418`); rapid attack — f28 PRE-charge/cap/decay
  (:1395-1414) + e16's three-kind set-guard (`apply_rapid_attack_bonus(12,
  4, true) == 12` :1413 + the use-site witness
  `world_sim/editing.rs:8947`); Mechdusa — `pop;pop;push` shape (:510-517)
  with e16's last-line assertions (:1120-1128, both dropped lines asserted
  gone); yoyo/expert/master — the Guarding-63 master-shift pin (:1379-1388)
  + the expert column check (:1213-1216 `d.expert` def 1131 → Expert row +
  disco colour), covering e16's expert-set assertions.
- *e16-only survivors:* `…loadout_share_arm_suppresses_the_equipable_row`
  (:1579, ported name-for-name), `…knockback_delta_reads_the_prefix_only_oldkb`
  (:1553), `…loadout_share_hint_round_trips` (`settings.rs:1520`), the
  fishing-typo witness ADDED (:1509). The `tooltip_sets` statue/alt-use
  table tests retire with their tables into the columns/bitsets — the
  statue carve-out + alt-use gate keep their coverage in
  `…wireable_rows_follow_the_statue_exception` (:1074) and
  `…place_ammo_consumable_ladder_repairs` (:1296-1300).
- *f28-only survivors:* the 3 tr-core format tests
  (`localization/format.rs:349/:361/:378` in `mod w4_f28_tests`).

**This round's two S12 closures (witness-only edits, WRITTEN NOT RUN):**

1. `w4_f28_the_prefix_comparison_rows` gains e16's negative-red +
   mana-invert 3-prefix matrix (Broken 10 on the Iron Broadsword → −17%
   RED; Adept 26 / Mystical 35 on the Space Gun 127 mana 6 → −17% GREEN /
   +17% RED through the inverted test, `Main.cs:21093`) — the §6
   "e16 mana-invert 3-prefix matrix is deeper" keep-both rider.
2. `w4_f28_the_place_ammo_consumable_ladder_repairs` gains the `wall > 0`
   assertion (`wall: Some(0)` refuses the place gate, `Some(1)` admits it;
   `Main.cs:20919`) — the §6 "never drop that check with the twin" rider.

**Witness union after this round:** 33 `fn w4_f28_*` across 9 files + 3
unnamed-prefix format tests = 36 fns; the two closures ADD assertions to
existing fns (no new fn), so the union's assertion-set count only grows —
the 48-set floor (128 §6) holds: every retired e16 twin's assertions are
either already in the surviving f28 witness or moved per the map above.

## 2. GetLinesInfo FINAL census (ALL 36 line-kinds, C# `Main.cs:20631-21343`)

Method: the C# window :20631-21343 read in full (8 slices); a PowerShell
marker script checked every kind's distinctive symbol/key/constant against
the HEAD `tooltip.rs` + `tooltip_lines2.rs` + `research.rs` — **55/55
markers OK, 0 MISSING**; the loose markers then hand-verified in ≤100-line
windows (cited below). Line numbers are HEAD `e9e37216a`.

| # | Kind (C# anchor) | Rust home at HEAD | State |
| --- | --- | --- | --- |
| 0 | hover name + rare colour (:20633, :20328-32, :20557-20627) | `tooltip.rs:290` `hover_name` + `:359-362`; `lines2:386` `tooltip_name_color` (−12 disco / −13 master pulse / mc-hc LAST) + App animator (`draw.rs:343-346`) | landed both halves |
| 1 | loadout-share arm (:20636-20654) | `tooltip.rs:363-399` (S1); SharedFrom arm :376-384 | landed; g55 writer dormant |
| 2 | favourited tip[56]/[57] + bank-block (:20655-20663) | `lines2:916` `favorite_rows` (raw chest −2/−5, `ChestUI.cs:1291-1302`) | landed |
| 3 | social tip[61] (:20664-20668) | `lines2:907` `social_row` over `d.has_vanity_effects` | landed (column) |
| 4 | damage row + gates (:20669-20700) | `lines2:842` gate (notAmmo/useStyle/coin bool) + `:958` value (mult, rapid set {2608,3349,672}, +5e-6 :971); class suffix `tooltip.rs:413-419` (2/3/4/53/55) | landed |
| 5 | crit row (:20701-20716) | `lines2:943` `crit_row_value` + `visual_crit_chance` (Revolver 2269 clamp) | landed |
| 6 | use-time tiers (:20717-20752) | `lines2:850` gate (whip) + `tooltip.rs:443-459` the 8 tiers (≤8/20/25/30/35/45/55/else) | landed |
| 7 | knockback word (:20753-20802 + :20333-20350) | `lines2:787` `display_knockback` (glove/buff/shroomite/minionKB/quiver-stake INT trunc/Psycho 3106/no-player stake disjunct) + `tooltip.rs:469-487` the 9 tiers (f64 1.5) | landed |
| 8 | fishing/bait (:20804-20815) | `tooltip.rs:495-506` `PrecentFishingPower` + no-arg `BaitRequired` + `BaitPower` | landed (S5) |
| 9 | equipable + !flag + shop tip[60] (:20816-20827) | `tooltip.rs:513-531` (hook/mount/pets admitted) + `lines2:859` `equipable_row` (854/3035) | landed (S3) |
| 10 | tileWand tip[52] (:20828-20832) | `lines2:930-939` `tile_wand_row` (consumed name) | landed (column) |
| 11 | quest/vanity (:20833-20841) | `tooltip.rs:537-541` `inter(65)` / `tip(24)` | landed |
| 12 | defense/pick/axe×5/hammer/tileBoost (:20843-20875) | `tooltip.rs:545-577` (axe `d.axe * 5` :558; +N sign arm :571-575) | landed |
| 13 | healLife range/mana (:20876-20893) | `lines2:869` `restores_life_row` (3001 → healLife..120 via `format_net`) + `tooltip.rs:586-590` RestoresMana | landed |
| 14 | buff/healing/mana-healing/mount chain (:20894-20913) | `tooltip.rs:593-617` (Buffs → Healing → ManaHealing → Mount, `is_cart` exclusion, light/vanity-pet gates) | landed |
| 15 | mana cost (:20914-20918) | `tooltip.rs:619-627` (spaceGun skip {127,4347,4348,514}, prefixed mana, manaCost mult INT) | landed |
| 16 | place/ammo/consumable ladder (:20919-20936) | `lines2:886-903` `places()` (wall>0) + the three gates (alt-use set/notAmmo/extractinator) | landed (S11) |
| 17 | material (:20937-20941) | `tooltip.rs` tip(36) arm | landed |
| 18 | Wireable (:20942-20946) | `lines2:460-466` generated `SET_IS_A_TRIGGER`/`SET_IS_A_MECHANISM` + tile-105 `is_wireable_statue` carve-out | landed (bitsets) |
| 19 | Container (:20947-20951) | `lines2:468-470` tile 21/467 | landed |
| 20 | WireTrigger (:20952-20956) | `lines2:472-474` tile 441/468 | landed |
| 21 | ToolTip lines + cursed swap (:20957-20972) | `lines2:479-487` (`uses_cursed_by_plantera_tooltip`, line-0 swap while !downed_plant_boss) | landed |
| 22 | anniversary Wizard Hat (:20973-20977) | `lines2:489-494` `world.tenth_anniversary && kind == 238` | landed |
| 23 | FTW Crispy Honey (:20978-20982) | `lines2:496-498` `get_good_world && 1127` | landed |
| 24 | Mechdusa (:20983-20999) | `lines2:502-517` — 556/557/544 last-line replace; 5334 literal `pop;pop;push` double-walk | landed (f28 shape) |
| 25 | DD2 poppers (:21000-21004) | `lines2:342-347` the 12-kind set + `:519-521` `!world.downed_dd2_any` gate (S10) | landed |
| 26 | WellFed expert (:21005-21009) | `lines2:327-329` {26,206,207} + `:523-525` | landed |
| 27 | buff duration (:21010-21015) | `lines2:529-542` int `/60` first, `round_bankers` both arms | landed |
| 28 | yoyo placeholder + index (:21016-21021) | `lines2:333-338` the 11 kinds + `:545-549` index return; draw arm `draw_item_tooltip` 5-copy pass (`d3599cc32`) | landed incl. draw |
| 29 | Expert row (:21022-21026) | `lines2:551-553` `d.expert` column | landed |
| 30 | Master row (:21027-21031) | `lines2:558-561` INSTANCE rare via `hover_stats` (Guarding-63 shift witnessed) | landed |
| 31 | prefix % rows + pen/tag (:21032-21151) | `lines2:651-748` — all 7 deltas (dmg/speed×−1/crit-raw/mana-INVERT :702/scale/velocity/oldKB :729), pen/tag `{0:+0;-#}` via tr-core `apply_format_spec` | landed |
| 32 | accessory fixed 62-80 (:21152-21265) | `lines2:752-777` the 19-arm match (Warding "+20 " trailing space :758) | landed |
| 33 | armor-set row (:21267-21311) | `lines2:569-600` — unworn `SETS.find` array[0] grey α190; worn `best_worn_set` LimeGreen/grey | landed |
| 34 | share hint (:21312-21317) | `lines2:605-610` (S4) | landed |
| 35 | journey sacrifice rows (:21318-21333) | `research.rs:76-133` `research_row_context` (17 ctx) + `research_tooltip_rows` (difficulty 3, remaining count, JourneyMode (255,120,187), ctx-29 teammate arm); fed at `lines2:616-625` | landed; c24 source dormant |
| 36 | bestiary notes (:21334-21342) | `lines2:630-636` `\n`-split + `IsNullOrWhiteSpace` guard | landed; e23 wire dormant |
| — | line tint byte-trunc (:20406) | `tooltip.rs` `draw_item_tooltip` `(byte)(c*num2)` | landed |
| — | shop lines (:20448-20555) | `shop.rs` `add_shop_lines` (`4c2e47a20`; f37 keep-one) | landed on this branch |
| — | crafting row (:20363-20366) | `craft.rs` `add_crafting_tooltip_lines` (`2d83ed345`) | landed; writer dormant |
| — | diff 1/2 name colours (:20620-20627) | `lines2:386` mc/hc LAST overrides (`3d2cec11b`) | landed; hover source dormant |

**Census verdict: NO line-kind is missing.** All 36 GetLinesInfo kinds +
the 4 out-of-function riders are landed on this branch (rows 1/35/36 with
their cited SOURCE-side dormancies — the rows themselves emit). The five
residual deferrals are all writers/sources, listed in research/129.

## 3. FINAL merger instruction (128 + 129 → the Lead)

1. **`task/W4-f28-tooltip-lines-2` is THE canonical tooltip branch.**
   Merge it whole (its tip at retire: this note's commit on top of
   `e9e37216a`). It carries: the GenItemData 5-flag provision + FNV pin,
   the two-module split, the tr-core format grammar, the master animator,
   the full rapid-attack machinery + use-site grant, every S1-S13 salvage,
   the co-lane's shop/crafting/diff/yoyo-logo closures and the
   SharedFrom/Journey/bestiary contract rows.
2. **DROP `task/W4-e16-item-tooltip-lines` at the merge — do NOT merge
   it.** Every UNIQUE-SALVAGE hunk is executed on f28 (§1's receipts);
   its remaining unique content is the retired census tables
   (`tooltip_sets.rs`), `signed_fmt`, the read-only rapid-attack wrapper
   and its `TooltipWorld` — all superseded per 128 §3/§4. Its witnesses
   are deduped per §6 (the two deeper e16 matrices landed here this
   round). Banner-grep on e16 finds no unported UNIQUE-SALVAGE marker.
   The e16 branch + worktree retire after the f28 merge lands (Lead's
   cleanup pass; never delete from this lane).
3. **Keep-one riders for the eventual main merge:** f37 re-points at
   `shop.rs::add_shop_lines` (already landed here); the g55/c24/e23
   mergers fill the three contract fields (`HoverFlags.shared_from_loadout`
   ← `get_effective_armor`'s `Option<usize>` at ctx 8-12;
   `TooltipContext.difficulty`/`research` ← `active_player_difficulty` +
   `App.research` at the draw_hud sites; `HoverFlags.bestiary_notes` ←
   the e23 hover wire) — the rows need NO further tooltip-side work.
4. **First run of the witnesses happens on the merged branch** (128 §8.4):
   all 33+3 are WRITTEN NOT RUN (code-only lane); the merged branch runs
   `cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids`.

## 4. Verification performed (this round, code-only)

- `git status --porcelain` clean at entry (the co-lane WIP self-committed
  as `e9e37216a` between probes; first probe found it LIVE — left alone).
- The 55-marker PowerShell census over the HEAD blobs: **55 OK / 0
  MISSING** (output in the session log); loose markers hand-read in
  ≤100-line windows (`tooltip.rs:305-630`, `tooltip_lines2.rs:330-975 +
  the witness module`, `research.rs:60-159`, `ui/mod.rs:1055-1294 +
  :1408-1437`, `settings.rs:724-743`, `prefix.rs:402-446`).
- The C# window `Main.cs:20631-21350` read in full (8 slices);
  `ID/Colors.cs:113` re-read (the co-lane's JourneyMode fix verified).
- All 10 tooltip-related anchors.txt markers `git grep -F`-verified
  present at HEAD (10/10 OK).
- The two S12 witness closures: WRITTEN NOT RUN (no cargo per the lane
  rules); arithmetic hand-derived (Broken 10: round(12×0.85)=10 →
  (10−12)/12×100 = −16.67 → −17; Adept 26: round(6×0.85)=5 → −17 GREEN;
  Mystical 35: round(6×1.2)=7 → +17 RED).