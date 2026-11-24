# 64 — Item-tooltip line census (W4-e16, code-only)

**Branch:** `task/W4-e16-item-tooltip-lines` · **Base:** `origin/main` `a725acb0` ·
**Mode:** CODE-ONLY — file reads + git only, no compiler invoked; witnesses are
WRITTEN NOT RUN.

Scope: every line-kind `Main.MouseText_DrawItemTooltip_GetLinesInfo`
(`Main.cs:20631-21343`) can append, censused against the landed port
(`crates/tr-app/src/ui/tooltip.rs::item_tooltip_lines`, S1-E-20 at base —
which stops at the per-item `Item.ToolTip` text, `Main.cs:20958-20972`).
The sibling `MouseText_DrawItemTooltip_AddShopLines` (`:20448-20555`) is the
shop lane's (W4-f37), out of this census beyond the boundary row.
`CreateTooltipLine` does not exist in this decompilation — the line builder
is `GetLinesInfo` itself; the per-line *text* object is `UI/ItemTooltip.cs`
(the `Lines`/`GetLine` split-on-`\n` cache, bounded by W4-c21 `d7f7e9cc`).

## 1. Line-kind census (C# order)

| # | C# lines | Line-kind | Port at base | Disposition |
| --- | --- | --- | --- | --- |
| 0 | :20633 | hover name (`GetHoverName`, affix + stack) | LANDED | colour fix: prefixed rare + expert −12 disco (family 3/2) |
| 1 | :20638-20654 | loadout-share arm (`canLoadoutShareAt` ctx 8-12): `UI.ItemLoadoutSharedFrom` / `UI.ItemLoadoutShared` / flag2; sets `flag` | MISSING | port (no loadouts → SharedFrom unreachable, census note) |
| 2 | :20655-20663 | favorited: tip[56]+tip[57] (+ `UI.ItemCannotBePlacedInsideItself` when `ChestUI.IsBlockedFromTransferIntoChest`, `ChestUI.cs:1291-1302`: 3213→bank, 4131/5325→bank4) | MISSING | port |
| 3 | :20664-20668 | social && !vanity && !hasVanityEffects → tip[61] | MISSING | port (ctx 9/11 social, `ItemSlot.cs:3191-3194`; hasVanityEffects census §2) |
| 4 | :20669-20700 | damage row — gate `damage>0 && (!notAmmo \|\| useStyle!=0) && (type<71 \|\| type>74 \|\| HasItem(905))`; `ToolTipDamageMultiplier` then `RapidAttackBonusDamage` {2608,3349,672} (`ItemID.cs:344`) × `ApplyRapidAttackBonus` read-only (`Player.cs:18251-18273`) then weapon multiplier +5e-6 | PARTIAL — plain `damage>0`, no notAmmo/coin/rapid gates; reads BASE damage (prefix ignored) | fix gates + prefixed stats + rapid read |
| 5 | :20701-20716 | crit row — `GetVisualCritChance` (`Item.cs:637-645`: 2269 revolver clamp) | PARTIAL — base crit, no revolver arm | fix |
| 6 | :20717-20752 | use-time tiers — gate `useStyle!=0 && (!summon \|\| IsAWhip[shoot])` | PARTIAL — summon/whip exclusion missing | fix |
| 7 | :20753-20802 | knockback word — minionKB add, quiver/stake `(int)(kb*1.1f)` TRUNCATION, Psycho-Knife 3106 held+hovered stealth arm (:20762-20765), `(double)kb<=1.5` first tier | PARTIAL — no truncation, no 3106 arm | fix |
| 8 | :20804-20815 | fishing power / bait rows | LANDED | — |
| 9 | :20816-20827 | equipable — `!flag` gate + shop arm (854/3035 && npcShop>0 → tip[60]) | PARTIAL — no !flag, no shop arm | fix |
| 10 | :20828-20832 | tileWand → tip[52] + `Lang.GetItemNameValue(tileWand)` (kinds 832→9, 932→154, 933→9, 1129→1124, 3360/3361→620; `Item.cs:11149/12148/12161/13888/30697/30710`) | MISSING | port (table; ItemDef has no column) |
| 11 | :20833-20841 | quest / vanity rows | LANDED | — |
| 12 | :20843-20875 | defense/pick/axe×5/hammer/tileBoost rows | LANDED | — |
| 13 | :20876-20893 | healLife (+3001 `RestoresLifeRange` arm) / healMana | PARTIAL — 3001 arm missing | fix |
| 14 | :20894-20913 | buff/healing/mana-healing/mount chain | LANDED | — |
| 15 | :20914-20918 | mana cost (spaceGun skip) | LANDED | prefixed mana (family 3) |
| 16 | :20919-20936 | place/ammo/consumable — `createWall>0 \|\| createTile>-1 \|\| type==849`; inner `PlaceTileOnAltUse[type] \|\| consumable` (`ItemID.cs:412`); ammo `!notAmmo`; consumable `!chlorophyteExtractinatorConsumable` (86/1329/2337-2339, `Item.cs:48505-48513`) | PARTIAL — no alt-use set, no notAmmo, no extractinator exclusion | fix |
| 17 | :20937-20941 | material | LANDED | — |
| 18 | :20942-20946 | `CommonItemTooltip.Wireable` — tile ∈ IsATrigger∪IsAMechanism (`TileID.cs:184-185`) & (tile!=105 \|\| IsWireableStatue[type], `ItemID.cs:1581`) | MISSING | port |
| 19 | :20947-20951 | `CommonItemTooltip.Container` — tile 21/467 | MISSING | port |
| 20 | :20952-20956 | `CommonItemTooltip.WireTrigger` — tile 441/468 | MISSING | port |
| 21 | :20957-20972 | per-item ToolTip lines; line 0 → tip[59] when `UsesCursedByPlanteraTooltip` {1533-1537,4714} (`ItemID.cs:410`) && !downedPlantBoss | PARTIAL — cursed override missing | fix |
| 22 | :20973-20977 | anniversary && 238 → `WizardHatDuringAnniversary` | MISSING | port |
| 23 | :20978-20982 | getGoodWorld && 1127 → `BurningBlock` | MISSING | port |
| 24 | :20983-20999 | Mechdusa (`SpecialSeedFeatures.cs:119-128` = remix && ftw): 556/557/544 last line → `MechSummonDuringEverything`; else 5334: drop last line, replace previous with `MechdusaSummonNotDuringEverything` (the double `numLines--` walk) | MISSING | port |
| 25 | :21000-21004 | DD2 poppers {3818-3820,3824-3826,3829-3834} && !downedDD2EventAnyDifficulty → misc[104] | MISSING | port |
| 26 | :21005-21009 | WellFed {26,206,207} (`BuffID.cs:43`) && expertMode → misc[40] | MISSING | port |
| 27 | :21010-21015 | buffTime duration — `<60s` SecondDuration else MinuteDuration, `Math.Round` banker's, minutes divide `(buffTime/60)/60.0` (int first) | MISSING | port |
| 28 | :21016-21021 | yoyo blank line + `yoyoLogo` index {3262,3282-3286,3315-3317,3291,3389} | MISSING | port (OneDropLogo texture unported — blank line keeps geometry, documented) |
| 29 | :21022-21026 | item.expert → `GameUI.Expert` (25 `this.expert=true` writes, §2) | MISSING | port |
| 30 | :21027-21031 | rare==-13 → `GameUI.Master` | MISSING | port |
| 31 | :21032-21151 | prefix comparison: damage/speed(×−1)/crit/mana(colour INVERTED)/scale/velocity/knockback(oldKB) % rows + armorPenetration/bonusTagDamage `{0:+0;-#}` rows; red (190,120,120) / green (120,190,120) | MISSING | port over `prefix::prefixed_stats` |
| 32 | :21152-21265 | accessory prefix fixed rows 62-80 (def 1-4, mana +20, crit 2/4, dmg 1-4, move 1-4, melee 1-4) | MISSING | port |
| 33 | :21267-21311 | armor-set bonus row — single piece grey (130,130,130,190) / worn best-set LimeGreen-or-grey; wornArmor = ctx 8 (`ItemSlot.cs:3188-3189`) | MISSING (machinery landed `armor_sets.rs`) | wire |
| 34 | :21312-21317 | flag2 && `Player.Settings.ShowLoadoutShareHint` → `UI.ItemLoadoutShareHint`, OurFavoriteColor (255,231,69,190) (`Main.cs:66748`) | MISSING | port + settings key (`Main.cs:4219/:4553`) |
| 35 | :21318-21333 | journey creative-sacrifice rows (difficulty==3) | MISSING | CONSTANTS-OFF — the port has no journey difficulty (`player.rs:4494` precedent); cited no-op |
| 36 | :21334-21342 | `Item.BestiaryNotes` split lines | MISSING | DROP — only `UIBestiaryInfoItemLine.cs:155` ever writes the field (bestiary item cache); the port's bestiary drop rows never build an item tooltip. Cited |
| — | :20362 | `AddShopLines` (shop price rows) | out of scope | W4-f37 shop lane |
| — | :20329-20332 | expert name colour rare=−12 (disco) | MISSING | port via `App.disco` |
| — | :20560-20567 | −13 masterColor / −12 disco name colours; diff 1/2 overrides | DROP (masterColor/difficulty unported) | keep documented; −12 disco lands with the expert row |

## 2. Set censuses (declaration → value)

- `Item.expert` — 25 writes (`Item.cs:13923,28599,28647,29748,29757,29898,
  30614,30626,30647,30655,30663,30669,30678,30781,32271,34163,37575,39327,
  39951,40025,40183,40201,40881,40952,41995`), all unconditional in their
  case bodies → kinds 1131, 3090, 3097, 3223, 3224, 3245, 3318-3332, 3333,
  3335, 3336, 3337, 3353, 3354-3356, 3367, 3577, 3860-3862, 4444, 4782,
  4954, 4957, 4987, 4989, 5100, 5111, 5289. `ItemDef` carries no column —
  table in `ui/tooltip.rs` (the only consumer).
- `Item.hasVanityEffects` — 21 writes: direct kinds 485, 497, 861, 1987,
  3110, 3305, 3581, 3929, 4563, 4762, 4874, 5000, 5075, 5076, 5113, 5331,
  5452, 5547; `DefaultToGolfBall` (3989, 4242-4255); `DefaultToInfoAccessory`
  (15-18, 393, 395, 707-709, 3036, 3037, 3084, 3095, 3096, 3099, 3102,
  3118-3123); `DefaultToMusicBox` (562-574, 1596-1610, 1963-1965, 2742,
  3044, 3235-3237, 3370, 3371, 3796, 3869, 4077-4082, 4237, 4356-4358,
  4421, 4606, 4979, 4985, 4990-4992, 5006, 5014-5040, 5044, 5112, 5362,
  5538, 5539, 5578-5582, 5637-5639, 6144-6146 — also `vanity=true`, so the
  social row's `!vanity` gate already excludes them).
- `Item.notAmmo` — 10 writes → 27, 71-74, 169, 353, 370, 408, 1246
  (matches the landed `item.rs::not_ammo`).
- `Item.tileWand` — 6 writes (§1 row 10).
- `chlorophyteExtractinatorConsumable` — `MakeUsableWithChlorophyteExtractinator`
  called at `Item.cs:2405` (86), `:16477` (1329), `:23497` (2337-2339).
- `Player.rapidAttackBonus` — no port field; the tooltip reads it without
  the increase arm (`increaseBonus=false` default). Field lands at 0 with
  no writer (the melee `:44291` increase path is the combat lane's);
  `ResetEffects` decay `:18278-18284` cited for that lane.

## 3. Fix families (commit order)

1. **Stat-gate corrections** — rows 4-7, 9, 10, 13, 16 + 18-20 (the missing
   first-half line-kinds), `Player.rapid_attack_bonus` field.
2. **ToolTip tail + world rows** — rows 21-30 (cursed override, anniversary,
   burning, mechdusa, DD2, well-fed, duration, yoyo, expert set + disco name
   colour, master).
3. **Prefix comparison** — rows 31-32 + the prefixed-stats read for rows
   0/4/5/6/7/15 (the C# reads the PREFIXED instance; the port read base).
4. **Favorite/social/loadout + armor-set + share-hint + wiring** — rows
   1-3, 33-34, `TooltipContext` growth, `draw_hud` wiring, settings key.

Witnesses per family: WRITTEN NOT RUN (code-only constraint).