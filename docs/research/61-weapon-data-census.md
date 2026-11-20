# 61 — Weapon data census: `Item.SetDefaults` vs the port's item table

Worker W3-i02 (S1-P, code-only). Scope: **every weapon row** of the port's
generated item-data table vs the C# `Item.cs` `SetDefaults` weapon case bodies.
No code changes; this is a census + fix-slice worklist.

## Verdict

**0 WRONG rows, 0 MISSING rows.** All **624** weapon rows of
`crates/tr-app/src/itemdata.rs` (`ITEMS`, 6196 ids) match an independent
re-execution of the C# case bodies on **all 18 census fields**
(damage, useTime, useAnimation, knockBack, crit, shoot, shootSpeed, useSound,
autoReuse, channel, mana, useStyle, useAmmo, reuseDelay, melee/ranged/magic/summon
flags). Class membership agrees on all 624 (no flips). The 25 seed-variant
weapon overrides (`VARIANT_OVERRIDES`) also match the C# variant branches.
The table is generated (`tools/GenItemData/gen.py`) and the census confirms the
generator's weapon output is faithful; the fix slices below are therefore
**empty** and are recorded as regression guards, not work items.

## Method

- **Port side.** `itemdata.rs:165-6362` — one `ItemDef` row per id, emitted by
  `tools/GenItemData/gen.py` from the decompiled sources (header `:1-11`).
  Weapon row = `melee|ranged|magic|summon` flag set **and** `damage >= 0`
  (this includes melee tools, ammo and coins, which carry a damage class in C#).
  The runtime reads the table directly (`prefix.rs:381,398`, `player.rs:3184`,
  `registry.rs:20-21`) — no parallel hand-written weapon stats exist in `crates/`
  (grep for `damage`/`use_time` rows finds only this table + its tests).
- **C# side.** An independent extractor (written for this census, not the
  generator's parser) re-runs each id through: `ResetStats` stock
  (`Item.cs:48944-49051`), the routed `SetDefaults1-5` / `SetFoodDefaults`
  dispatcher (`Item.cs:48685-48712`; spans `:1387`, `:12931`, `:21380`,
  `:27659`, `:35462`, `:48040`), the `DefaultTo*`/`SetWeaponValues` helpers
  (`:47770-48633`, `:48415-48420`), and the epilogue (projHook zeroing
  `:48893-48898`, drill/chainsaw ×0.6 `:48899-48912`, deprecated→air
  `:48922-48925`). Only constant/set loaders are shared with `gen.py`
  (SoundID ctor order, `ItemID.Sets`, id consts, projHook).
- **Semantics.** Base rows = normal world: all `ItemVariants` conditions are
  seed-gated (`ItemVariants.cs:117-145` — remix/getGoodWorld/mechdusa/skyblock),
  so `SelectVariant` returns null and `== Variant` bodies are skipped;
  `!= DisabledBossSummon` bodies apply. `useSound` values are indices into
  `soundid.rs::SOUND_STYLES` (SoundID ctor order, `SoundID.cs:1254-1768`);
  the tables below print the C# field name (e.g. `Item1` = index 126).
- **Coverage check.** 1709 non-weapon ids could not be resolved by the census
  extractor (placeables/fish/accessories whose helpers it does not model);
  a scan of their C# case bodies found **zero** weapon markers
  (`this.melee/ranged/magic/summon = true`, `this.damage =`, weapon helpers),
  and none is a weapon in the port — so no weapon row escaped comparison.
  49 weapon rows resolve via `default:`/`goto case`/nested `SetDefaultsN`
  paths rather than a direct label (cited as `default` below).

## Family summary (624 rows)

| family | rows | class | notes |
|---|---|---|---|
| sword/other-melee | 54 | Melee | useStyle 1, no projectile |
| shortsword | 12 | Melee | useStyle 13 stab (incl. Ruler, Gladius, Piercing Starlight) |
| boomerang/sword-beam | 68 | Melee | useStyle 1 + shoot (phaseblades, beam swords, boomerangs, Paladin's Hammer) |
| spear | 23 | Melee | useStyle 5 + shoot, no channel |
| yoyo/flail | 40 | Melee | useStyle 5 + channel (yoyos, flails, DD2 monk staves, Laser Drill) |
| tool-melee | 105 | Melee | pick/axe/hammer power rows (drills get epilogue ×0.6) |
| bow/repeater | 40 | Ranged | useAmmo 40 (Arrow) |
| gun/launcher | 36 | Ranged | useAmmo 97/71/169/283/931/949/3108/1783/1785/1836/1261 |
| rocket-launcher | 7 | Ranged | useAmmo 771 (Rocket) |
| star-cannon | 2 | Ranged | useAmmo 75 (FallenStar): 197, 4060 |
| thrown/other-ranged | 28 | Ranged | consumables, flamethrowers, piranha gun, paintball gun |
| ammo | 64 | Ranged | arrows/bullets/rockets/darts/coins/seeds/flares (`ammo != 0`) |
| magic | 81 | Magic | staves, tomes, guns, guitars, channels |
| summon-staff/sentry | 46 | Summon | minion staves + DD2 sentries |
| whip | 18 | Summon | `DefaultToWhip` (`Item.cs:47810-47826`), sound Item152 |

Class totals: Melee 302, Ranged 177, Magic 81, Summon 64.

## Per-family tables (representative rows; C# == port on every cell)

Columns: dmg = damage; ut/ua = useTime/useAnimation; kb = knockBack;
shoot = projectile id (`-` = none); spd = shootSpeed; sound = `Item.UseSound`
(C# name; `null` = none); auto = autoReuse; chan = channel; mana.
`Item.cs` = case-label line of the row's case body (`default` = reached via a
switch default / `goto case` / nested call). Every row was compared on all 18
fields; only representatives are printed — full id rosters in the appendix.

### tool-melee (105 rows total)

| id | name | dmg | ut/ua | kb | crit | shoot | spd | sound | auto | chan | mana | Item.cs |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | Iron Pickaxe | 5 | 13/20 | 2 | 0 | - | - | Item1 | Y | n | - | :1391 |
| 7 | Iron Hammer | 7 | 20/30 | 5.5 | 0 | - | - | Item1 | Y | n | - | :1472 |
| 367 | Pwnhammer | 26 | 14/27 | 7.5 | 0 | - | - | Item1 | Y | n | - | :5662 |
| 660 | Pearlwood Hammer | 10 | 19/29 | 5.5 | 0 | - | - | Item1 | Y | n | - | :9103 |
| 798 | Deathbringer Pickaxe | 12 | 14/22 | 3.5 | 0 | - | - | Item1 | Y | n | - | :10723 |
| 990 | Pickaxe Axe | 35 | 7/25 | 4.75 | 0 | - | - | Item1 | Y | n | - | :12768 |
| 1202 | Titanium Pickaxe | 27 | 7/25 | 5 | 0 | - | - | Item1 | Y | n | - | :14853 |
| 1234 | Chlorophyte Warhammer | 80 | 14/35 | 8 | 0 | - | - | Item1 | Y | n | - | :15270 |
| 1294 | Picksaw | 34 | 6/16 | 5.5 | 0 | - | - | Item1 | Y | n | - | :15987 |
| 2341 | Reaver Shark | 16 | 13/22 | 3 | 0 | - | - | Item1 | Y | n | - | default |
| 2798 | Laser Drill | 35 | 6/25 | 4.75 | 0 | 445 | 36 | null | n | Y | - | default |
| 3488 | Tungsten Axe | 7 | 18/26 | 4.5 | 0 | - | - | Item1 | Y | n | - | :31602 |
| 3493 | Lead Hammer | 8 | 19/29 | 5.5 | 0 | - | - | Item1 | Y | n | - | :31533 |
| 3512 | Silver Axe | 6 | 18/26 | 4.5 | 0 | - | - | Item1 | Y | n | - | :31368 |
| 4317 | Blood Hamaxe | 30 | 11/27 | 7 | 0 | - | - | Item1 | Y | n | - | :36974 |
| 5295 | Acorn Axe | 20 | 12/24 | 5 | 0 | - | - | Item1 | Y | n | - | :42126 |

### sword/other-melee (54 rows total)

| id | name | dmg | ut/ua | kb | crit | shoot | spd | sound | auto | chan | mana | Item.cs |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 4 | Iron Broadsword | 12 | 20/20 | 5.5 | 0 | - | - | Item1 | n | n | - | :1428 |
| 24 | Wooden Sword | 7 | 20/20 | 5 | 0 | - | - | Item1 | n | n | - | :1661 |
| 213 | Staffof Regrowth | 7 | 13/25 | 3 | 0 | - | - | Item1 | Y | n | - | :4021 |
| 653 | Ebonwood Sword | 11 | 19/19 | 6 | 0 | - | - | Item1 | n | n | - | :8998 |
| 672 | Cutlass | 53 | 16/16 | 4 | 0 | - | - | Item1 | Y | n | - | :9251 |
| 881 | Cactus Sword | 10 | 30/30 | 4.5 | 0 | - | - | Item1 | n | n | - | :11627 |
| 1185 | Palladium Sword | 49 | 22/22 | 5.5 | 0 | - | - | Item1 | Y | n | - | :14561 |
| 1827 | Bladed Glove | 14 | 8/8 | 4 | 0 | - | - | Item1 | Y | n | - | :20232 |
| 2273 | Katana | 18 | 20/20 | 3.5 | 15 | - | - | Item1 | Y | n | - | default |
| 2608 | Falcon Blade | 25 | 20/20 | 6 | 0 | - | - | Item1 | Y | n | - | default |
| 3211 | Bladetongue | 55 | 28/28 | 5.75 | 0 | - | - | Item1 | Y | n | - | :28955 |
| 3490 | Tungsten Broadsword | 14 | 19/19 | 6 | 0 | - | - | Item1 | n | n | - | :31571 |
| 3520 | Gold Broadsword | 15 | 18/18 | 6.5 | 0 | - | - | Item1 | n | n | - | :31279 |
| 5096 | Ham Bat | 57 | 20/20 | 6.5 | 0 | - | - | Item1 | Y | n | - | :40835 |
| 5129 | Flymeal | 15 | 17/17 | 5 | 0 | - | - | Item1 | n | n | - | :41803 |
| 5284 | Ash Wood Sword | 13 | 17/17 | 5 | 0 | - | - | Item1 | n | n | - | :41945 |

### shortsword (12 rows total)

| id | name | dmg | ut/ua | kb | crit | shoot | spd | sound | auto | chan | mana | Item.cs |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 6 | Iron Shortsword | 8 | 12/12 | 4 | 0 | 940 | 2.1 | Item1 | n | n | - | :1455 |
| 486 | Ruler | 12 | 5/20 | 0.5 | 0 | 842 | 2.4 | Item1 | Y | n | - | :7166 |
| 1227 | Chlorophyte Saber | 80 | 6/30 | 5 | 0 | 1127 | 2.4 | Item1 | Y | n | - | :15139 |
| 3483 | Platinum Shortsword | 13 | 10/10 | 5 | 0 | 945 | 2.1 | Item1 | n | n | - | :31640 |
| 3489 | Tungsten Shortsword | 10 | 11/11 | 4 | 0 | 943 | 2.1 | Item1 | n | n | - | :31581 |
| 3495 | Lead Shortsword | 9 | 12/12 | 4 | 0 | 941 | 2.1 | Item1 | n | n | - | :31522 |
| 3501 | Tin Shortsword | 7 | 12/12 | 4 | 0 | 939 | 2.1 | Item1 | n | n | - | :31464 |
| 3507 | Copper Shortsword | 5 | 13/13 | 4 | 0 | 938 | 2.1 | Item1 | n | n | - | :31405 |
| 3513 | Silver Shortsword | 9 | 12/12 | 4 | 0 | 942 | 2.1 | Item1 | n | n | - | :31347 |
| 3519 | Gold Shortsword | 12 | 11/11 | 5 | 0 | 944 | 2.1 | Item1 | n | n | - | :31289 |
| 4463 | Gladius | 15 | 6/18 | 3 | 0 | 802 | 2.4 | Item1 | n | n | - | :37698 |
| 4923 | Piercing Starlight | 80 | 6/18 | 4 | 10 | 927 | 15 | null | n | Y | - | :39889 |

### bow/repeater (40 rows total)

| id | name | dmg | ut/ua | kb | crit | shoot | spd | sound | auto | chan | mana | Item.cs |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 39 | Wooden Bow | 4 | 30/30 | 0 | 0 | 1 | 6.1 | Item5 | n | n | - | :1830 |
| 44 | Demon Bow | 14 | 25/25 | 1 | 0 | 1 | 6.7 | Item5 | n | n | - | :1894 |
| 435 | Cobalt Repeater | 35 | 23/23 | 1.5 | 0 | 1 | 9 | Item5 | Y | n | - | :6488 |
| 655 | Ebonwood Bow | 8 | 28/28 | 0 | 0 | 1 | 6.6 | Item5 | n | n | - | :9028 |
| 661 | Pearlwood Bow | 12 | 20/20 | 0 | 0 | 1 | 7 | Item5 | n | n | - | :9119 |
| 725 | Ice Bow | 39 | 14/14 | 4.5 | 0 | 1 | 10 | Item5 | Y | n | - | :9858 |
| 1194 | Orichalcum Repeater | 40 | 19/19 | 2 | 0 | 1 | 9.75 | Item5 | Y | n | - | :14715 |
| 2515 | Palm Wood Bow | 6 | 29/29 | 0 | 0 | 1 | 6.6 | Item5 | n | n | - | default |
| 3019 | Hellwing Bow | 22 | 13/13 | 5.5 | 0 | 485 | 6 | Item5 | Y | n | - | :27945 |
| 3486 | Tungsten Bow | 10 | 26/26 | 0 | 0 | 1 | 6.6 | Item5 | n | n | - | :31612 |
| 3498 | Tin Bow | 7 | 28/28 | 0 | 0 | 1 | 6.6 | Item5 | n | n | - | :31495 |
| 3510 | Silver Bow | 9 | 27/27 | 0 | 0 | 1 | 6.6 | Item5 | n | n | - | :31378 |
| 3859 | DD2 Betsy Bow | 38 | 30/30 | 4.5 | 3 | 710 | 11 | Item102 | Y | n | - | :34124 |
| 4953 | Fairy Queen Ranged Item | 50 | 2/30 | 2 | 0 | 1 | 10 | Item5 | Y | n | - | :39974 |
| 5282 | Ash Wood Bow | 10 | 25/25 | 0 | 0 | 1 | 6.6 | Item5 | n | n | - | :41903 |

### ammo (64 rows total)

| id | name | dmg | ut/ua | kb | crit | shoot | spd | sound | auto | chan | mana | Item.cs |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 40 | Wooden Arrow | 5 | 100/100 | 2 | 0 | 1 | 3 | null | n | n | - | :1845 |
| 41 | Flaming Arrow | 7 | 100/100 | 2 | 0 | 2 | 3.5 | null | n | n | - | :1857 |
| 73 | Gold Coin | 100 | 10/15 | 0 | 0 | 160 | 3 | null | Y | n | - | :2260 |
| 283 | Seed | 4 | 100/100 | 0 | 0 | 51 | - | null | n | n | - | :4735 |
| 546 | Cursed Bullet | 15 | 100/100 | 4 | 0 | 104 | 5 | null | n | n | - | :7896 |
| 772 | Rocket II | 40 | 100/100 | 4 | 0 | 3 | - | null | n | n | - | :10403 |
| 1179 | Chlorophyte Bullet | 9 | 100/100 | 4.5 | 0 | 207 | 5 | null | n | n | - | :14478 |
| 1335 | Ichor Bullet | 13 | 100/100 | 4 | 0 | 279 | 5.25 | null | n | n | - | :16528 |
| 1342 | Venom Bullet | 15 | 100/100 | 4.1 | 0 | 283 | 5.3 | null | n | n | - | :16610 |
| 1352 | Golden Bullet | 10 | 100/100 | 3.6 | 0 | 287 | 4.6 | null | n | n | - | :16708 |
| 3009 | Crystal Dart | 14 | 100/100 | 3.5 | 0 | 477 | 1 | null | n | n | - | :27801 |
| 3567 | Moonlord Bullet | 20 | 100/100 | 3 | 0 | 638 | 2 | null | n | n | - | :32145 |
| 4449 | Honey Rocket | 40 | 100/100 | 4 | 0 | - | - | null | n | n | - | :37617 |
| 5377 | Spelunker Flare | 1 | 100/100 | 1.5 | 0 | 1008 | 6 | null | n | n | - | :42600 |
| 5379 | Rainbow Flare | 1 | 100/100 | 1.5 | 0 | 1010 | 6 | null | n | n | - | :42625 |
| 5380 | Shimmer Flare | 1 | 100/100 | 1.5 | 0 | 1011 | 6 | null | n | n | - | :42637 |

### thrown/other-ranged (28 rows total)

| id | name | dmg | ut/ua | kb | crit | shoot | spd | sound | auto | chan | mana | Item.cs |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 42 | Shuriken | 10 | 15/15 | 0 | 0 | 3 | 9 | Item1 | n | n | - | :1869 |
| 154 | Bone | 20 | 12/12 | 2.3 | 0 | 21 | 8 | Item1 | n | n | - | :3210 |
| 160 | Harpoon | 25 | 30/30 | 6 | 0 | 23 | 15 | Item10 | Y | n | - | :3294 |
| 168 | Grenade | 60 | 45/45 | 8 | 0 | 30 | 5.5 | Item1 | n | n | - | :3436 |
| 287 | Poisoned Knife | 14 | 15/15 | 2.4 | 4 | 54 | 12 | Item1 | Y | n | - | :4783 |
| 506 | Flamethrower | 35 | 6/30 | 0.3 | 0 | 85 | 7 | Item34 | Y | n | - | :7392 |
| 1000 | Confetti Gun | 0 | 15/15 | 0 | 0 | 178 | 10 | Item11 | n | n | - | :12913 |
| 1156 | Piranha Gun | 40 | 30/30 | 1 | 0 | 190 | 14 | Item10 | n | Y | - | :14213 |
| 1910 | Elf Melter | 57 | 6/30 | 0.425 | 0 | 85 | 8.5 | Item34 | Y | n | - | :20722 |
| 2586 | Sticky Grenade | 60 | 45/45 | 8 | 0 | 397 | 5.5 | Item1 | n | n | - | default |
| 3094 | Javelin | 17 | 24/24 | 4.75 | 0 | 507 | 11.5 | Item1 | n | n | - | :28614 |
| 3197 | Frost Daggerfish | 17 | 13/13 | 3.5 | 0 | 520 | 12.5 | Item1 | n | n | - | :29636 |
| 3350 | Painter Paintball Gun | 12 | 9/24 | 1.25 | 7 | 587 | 10 | null | n | n | - | :31950 |
| 3378 | Bone Javelin | 20 | 25/25 | 5 | 0 | 598 | 10 | Item1 | Y | n | - | :30876 |
| 3379 | Bone Dagger | 14 | 14/14 | 1.5 | 0 | 599 | 10 | Item1 | Y | n | - | :30895 |
| 3821 | Ale Throwing Glove | 32 | 32/32 | 4 | 0 | 669 | 11 | Item1 | n | n | - | :33903 |

### boomerang/sword-beam (68 rows total)

| id | name | dmg | ut/ua | kb | crit | shoot | spd | sound | auto | chan | mana | Item.cs |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 46 | Lights Bane | 16 | 20/20 | 5 | 0 | 974 | - | Item1 | n | n | - | :1928 |
| 55 | Enchanted Boomerang | 17 | 20/20 | 8 | 0 | 6 | 10 | Item1 | n | n | - | :2033 |
| 198 | Blue Phaseblade | 26 | 18/18 | 3 | 0 | 1059 | 14 | Item15 | Y | n | - | :3832 |
| 273 | Nights Edge | 40 | 25/25 | 4.5 | 0 | 972 | 5 | Item1 | Y | n | - | :4578 |
| 671 | Keybrand | 105 | 20/20 | 6.5 | 16 | 1074 | 15.9 | Item1 | Y | n | - | :9233 |
| 674 | True Excalibur | 72 | 18/18 | 4.5 | 0 | 983 | 11 | Item1 | Y | n | - | :9280 |
| 989 | Enchanted Sword | 23 | 45/21 | 4.25 | 0 | 173 | 9.5 | Item1 | Y | n | - | :12751 |
| 1513 | Paladins Hammer | 100 | 15/15 | 9 | 0 | 301 | 14 | Item1 | Y | n | - | :18489 |
| 1918 | Fruitcake Chakram | 19 | 15/15 | 8 | 0 | 333 | 11 | Item1 | n | n | - | :20829 |
| 2424 | Anchor | 70 | 20/20 | 8 | 0 | 383 | 20 | Item1 | n | n | - | default |
| 3063 | Meowmere | 200 | 14/14 | 6.5 | 0 | 502 | 12 | Item1 | Y | n | - | :28323 |
| 3766 | Green Phasesaber | 50 | 16/16 | 3 | 0 | 1067 | 14 | Item15 | Y | n | - | :33344 |
| 3769 | Yellow Phasesaber | 50 | 16/16 | 3 | 0 | 1070 | 14 | Item15 | Y | n | - | :33347 |
| 4258 | Orange Phaseblade | 26 | 18/18 | 3 | 0 | 1071 | 14 | Item15 | Y | n | - | :36590 |
| 5298 | Trimarang | 21 | 20/20 | 4.5 | 0 | 1000 | 11.5 | Item1 | Y | n | - | :42037 |
| 5670 | Rainbow Phaseblade | 26 | 18/18 | 3 | 0 | 1101 | 14 | Item15 | Y | n | - | :44567 |

### magic (81 rows total)

| id | name | dmg | ut/ua | kb | crit | shoot | spd | sound | auto | chan | mana | Item.cs |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 64 | Vilethorn | 10 | 28/28 | 1 | 0 | 7 | 32 | null | n | n | 10 | :2134 |
| 112 | Flowerof Fire | 48 | 16/16 | 5.5 | 0 | 15 | 7.5 | Item20 | n | n | 9 | :2700 |
| 494 | Magical Harp | 42 | 12/12 | 1.5 | 0 | 76 | 4.5 | null | Y | n | 5 | :7245 |
| 726 | Frost Staff | 46 | 12/12 | 5 | 0 | 359 | 16 | Item20 | Y | n | 12 | :9885 |
| 742 | Emerald Staff | 19 | 32/32 | 4.25 | 0 | 124 | 8 | Item43 | Y | n | 8 | :10025 |
| 1121 | Bee Gun | 9 | 12/12 | 0.25 | 0 | 181 | 8 | Item11 | Y | n | 5 | :13783 |
| 1295 | Heat Ray | 90 | 10/10 | 3 | 0 | 260 | 15 | Item12 | Y | n | 8 | :16006 |
| 1801 | Bat Scepter | 45 | 12/12 | 3 | 0 | 316 | 10 | Item32 | Y | n | 6 | :19975 |
| 2882 | Charged Blaster Cannon | 100 | 20/20 | 2 | 0 | 460 | 14 | Item75 | n | Y | 14 | default |
| 3269 | Medusa Head | 40 | 20/20 | 2 | 0 | 535 | 1 | null | Y | Y | 15 | :30313 |
| 3542 | Nebula Blaze | 130 | 12/12 | 3 | 0 | 634 | 6 | Item20 | Y | n | 12 | :31851 |
| 3852 | Book Staff | 36 | 3/25 | 9 | 0 | 712 | 11 | DD2_BookStaffCast | Y | n | 20 | :34086 |
| 5065 | Princess Weapon | 70 | 25/25 | 5 | 0 | 950 | 10 | Item43 | Y | n | 18 | :40612 |
| 6155 | Lightning Strike | 150 | 55/55 | 10 | 10 | 1117 | 8 | Item43 | Y | n | 30 | :47446 |
| 6173 | Arc Surge | 180 | 6/18 | 4 | 0 | 1122 | 8 | Item15 | Y | n | 18 | :47555 |

### gun/launcher (36 rows total)

| id | name | dmg | ut/ua | kb | crit | shoot | spd | sound | auto | chan | mana | Item.cs |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 95 | Flintlock Pistol | 13 | 16/16 | 1 | 6 | 14 | 6 | Item184 | n | n | - | :2477 |
| 96 | Musket | 31 | 32/32 | 5.25 | 8 | 10 | 9 | Item183 | n | n | - | :2496 |
| 164 | Handgun | 26 | 15/15 | 3 | 0 | 14 | 10 | Item41 | n | n | - | :3368 |
| 281 | Blowpipe | 9 | 25/25 | 3.5 | 0 | 10 | 11 | Item63 | Y | n | - | :4703 |
| 534 | Shotgun | 24 | 45/45 | 6.5 | 0 | 10 | 7 | Item36 | n | n | - | :7739 |
| 905 | Coin Gun | 0 | 8/8 | 2 | 0 | 158 | 10 | Item11 | Y | n | - | :11824 |
| 1254 | Sniper Rifle | 200 | 36/36 | 8 | 25 | 10 | 16 | Item40 | n | n | - | :15518 |
| 1265 | Uzi | 30 | 9/9 | 3.5 | 0 | 14 | 13 | Item11 | Y | n | - | :15704 |
| 1782 | Candy Corn Rifle | 60 | 9/9 | 2 | 6 | 311 | 10 | Item11 | Y | n | - | :19750 |
| 1870 | Red Ryder | 20 | 38/38 | 3.75 | 0 | 10 | 8 | Item11 | Y | n | - | :20586 |
| 2270 | Gatligator | 21 | 7/7 | 1.5 | 0 | 10 | 8 | Item41 | Y | n | - | default |
| 3008 | Dart Rifle | 52 | 38/38 | 5.5 | 0 | 10 | 14.5 | Item99 | Y | n | - | :27781 |
| 3788 | Onyx Blaster | 24 | 48/48 | 6.5 | 0 | 10 | 7 | Item36 | n | n | - | :33551 |
| 4703 | Quad Barrel Shotgun | 14 | 55/55 | 6.5 | 0 | 14 | 7 | Item36 | n | n | - | :38652 |
| 5117 | Pew Matic Horn | 20 | 15/15 | 1 | 0 | 968 | 14 | Item61 | Y | n | - | :40997 |

### yoyo/flail (40 rows total)

| id | name | dmg | ut/ua | kb | crit | shoot | spd | sound | auto | chan | mana | Item.cs |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 162 | Ball O Hurt | 15 | 45/45 | 5.5 | 0 | 25 | 12 | Item1 | n | Y | - | :3330 |
| 163 | Blue Moon | 27 | 45/45 | 6 | 0 | 26 | 12 | Item1 | n | Y | - | :3349 |
| 801 | The Meatball | 17 | 45/45 | 5.5 | 0 | 154 | 12 | Item1 | n | Y | - | :10774 |
| 3279 | Corrupt Yoyo | 16 | 25/25 | 4.5 | 0 | 542 | 16 | Item1 | n | Y | - | :30044 |
| 3281 | Jungle Yoyo | 18 | 25/25 | 3.75 | 0 | 544 | 16 | Item1 | n | Y | - | :30046 |
| 3283 | Chik | 34 | 25/25 | 3.3 | 0 | 546 | 16 | Item1 | n | Y | - | :30048 |
| 3287 | Reds Yoyo | 70 | 25/25 | 4.5 | 0 | 550 | 16 | Item1 | n | Y | - | :30052 |
| 3291 | Kraken | 120 | 25/25 | 4.3 | 10 | 554 | 16 | Item1 | n | Y | - | :30056 |
| 3317 | Valor | 28 | 25/25 | 3.85 | 0 | 564 | 16 | Item1 | n | Y | - | default |
| 3835 | Monk Staff T1 | 50 | 30/30 | 7 | 0 | 697 | 24 | DD2_MonkStaffSwing | Y | Y | - | :33958 |
| 3858 | Monk Staff T3 | 140 | 30/30 | 5 | 0 | 707 | 24 | DD2_SkyDragonsFurySwing | Y | Y | - | :34104 |
| 4272 | Drippler Flail | 55 | 40/40 | 6.5 | 0 | 757 | 15.9 | Item1 | n | Y | - | :36757 |
| 5011 | Mace | 9 | 45/45 | 4.6 | 0 | 947 | 11 | Item1 | n | Y | - | :40333 |
| 5294 | Hive Five | 24 | 25/25 | 3.75 | 0 | 999 | 10 | Item1 | n | Y | - | :42004 |
| 5526 | Flairon Flail | 66 | 20/20 | 4.5 | 0 | 1058 | 14 | Item1 | Y | Y | - | :43524 |

### star-cannon (2 rows total)

| id | name | dmg | ut/ua | kb | crit | shoot | spd | sound | auto | chan | mana | Item.cs |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 197 | Star Cannon | 55 | 12/12 | 3 | 0 | 955 | 14 | Item9 | Y | n | - | :3809 |
| 4060 | Super Star Cannon | 60 | 18/18 | 5 | 0 | 728 | 20 | null | Y | n | - | :35671 |

### spear (23 rows total)

| id | name | dmg | ut/ua | kb | crit | shoot | spd | sound | auto | chan | mana | Item.cs |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 274 | Dark Lance | 34 | 22/22 | 5 | 0 | 46 | 6 | Item1 | n | n | - | :4597 |
| 277 | Trident | 14 | 31/31 | 6 | 0 | 47 | 4 | Item1 | n | n | - | :4639 |
| 280 | Spear | 8 | 31/31 | 6.5 | 0 | 49 | 3.7 | Item1 | n | n | - | :4686 |
| 406 | Adamantite Glaive | 49 | 25/25 | 6 | 0 | 66 | 5 | Item1 | n | n | - | :6109 |
| 537 | Cobalt Naginata | 44 | 28/28 | 4 | 0 | 97 | 4.3 | Item1 | n | n | - | :7772 |
| 550 | Gungnir | 61 | 22/22 | 6.4 | 0 | 105 | 5.6 | Item1 | n | n | - | :7928 |
| 802 | The Rotted Fork | 17 | 31/31 | 5 | 0 | 153 | 4 | Item1 | n | n | - | :10793 |
| 1193 | Orichalcum Halberd | 46 | 25/25 | 5.5 | 0 | 215 | 4.5 | Item1 | n | n | - | :14697 |
| 1200 | Titanium Trident | 48 | 23/23 | 6.2 | 0 | 218 | 5 | Item1 | n | n | - | :14817 |
| 1228 | Chlorophyte Partisan | 49 | 23/23 | 6.2 | 0 | 222 | 5 | Item1 | n | n | - | :15158 |
| 1314 | KO Cannon | 40 | 28/28 | 6.5 | 0 | 271 | 15 | Item174 | Y | n | - | :16265 |
| 1571 | Scourgeofthe Corruptor | 70 | 20/20 | 5 | 0 | 306 | 14 | Item39 | Y | n | - | :18839 |
| 1947 | North Pole | 80 | 30/30 | 6.7 | 0 | 342 | 4.75 | Item1 | n | n | - | :21104 |
| 2331 | Obsidian Swordfish | 70 | 20/20 | 6.5 | 20 | 367 | 4 | Item1 | n | n | - | default |
| 2611 | Flairon | 66 | 20/20 | 4.5 | 0 | 404 | 14 | Item1 | Y | n | - | default |
| 4061 | Thunder Spear | 14 | 28/28 | 6 | 0 | 730 | 3.5 | Item1 | n | n | - | :35695 |

### rocket-launcher (7 rows total)

| id | name | dmg | ut/ua | kb | crit | shoot | spd | sound | auto | chan | mana | Item.cs |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 758 | Grenade Launcher | 60 | 20/20 | 4 | 0 | 133 | 10 | Item61 | Y | n | - | :10231 |
| 759 | Rocket Launcher | 55 | 30/30 | 4 | 0 | 134 | 5 | Item11 | Y | n | - | :10249 |
| 760 | Proximity Mine Launcher | 80 | 50/50 | 4 | 0 | 135 | 12 | Item11 | Y | n | - | :10267 |
| 1946 | Snowman Cannon | 67 | 15/15 | 4 | 0 | 338 | 15 | Item11 | Y | n | - | :21086 |
| 2796 | Electrosphere Launcher | 40 | 12/12 | 2 | 0 | 442 | 12 | Item92 | n | n | - | default |
| 3546 | Fireworks Launcher | 25 | 30/30 | 4 | 0 | 134 | 15 | Item156 | Y | n | - | :31932 |
| 3930 | Celeb2 | 50 | 6/6 | 10 | 0 | 714 | 17 | null | Y | Y | - | :34741 |

### summon-staff/sentry (46 rows total)

| id | name | dmg | ut/ua | kb | crit | shoot | spd | sound | auto | chan | mana | Item.cs |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1157 | Pygmy Staff | 40 | 28/28 | 3 | 0 | 191 | 10 | Item44 | Y | n | - | :14231 |
| 1309 | Slime Staff | 8 | 28/28 | 2 | 0 | 266 | 10 | Item44 | Y | n | - | :16187 |
| 2364 | Hornet Staff | 12 | 22/22 | 2 | 0 | 373 | 10 | Item76 | Y | n | - | default |
| 2551 | Spider Staff | 26 | 36/36 | 3 | 0 | 390 | 10 | Item83 | Y | n | - | default |
| 2749 | Xeno Staff | 36 | 36/36 | 2 | 0 | 423 | 10 | Item44 | Y | n | - | default |
| 3249 | Deadly Sphere Staff | 55 | 36/36 | 2 | 0 | 533 | 10 | Item113 | Y | n | - | :29921 |
| 3571 | Rainbow Crystal Staff | 130 | 30/30 | 7.5 | 0 | 643 | 14 | Item78 | n | n | - | :32190 |
| 3824 | DD2 Ballistra Tower T1 Popper | 30 | 30/30 | 4.7 | 0 | 677 | 1 | DD2_DefenseTowerSpawn | n | n | - | :33807 |
| 3829 | DD2 Lightning Aura T1 Popper | 4 | 30/30 | 0.25 | 0 | 688 | 1 | DD2_DefenseTowerSpawn | n | n | - | :33810 |
| 3830 | DD2 Lightning Aura T2 Popper | 11 | 30/30 | 0.25 | 0 | 689 | 1 | DD2_DefenseTowerSpawn | n | n | - | :33811 |
| 3834 | DD2 Explosive Trap T3 Popper | 126 | 30/30 | 0.5 | 0 | 693 | 1 | DD2_DefenseTowerSpawn | n | n | - | :33815 |
| 4607 | Storm Tiger Staff | 41 | 36/36 | 2.5 | 0 | 831 | 10 | Item44 | Y | n | - | :38134 |
| 5005 | Empress Blade | 90 | 36/36 | 4 | 0 | 946 | 10 | Item82 | Y | n | - | :40278 |
| 5114 | Abigails Flower | 6 | 36/36 | 2 | 0 | 970 | 10 | AbigailSummon | Y | n | - | :40964 |
| 5663 | Palworld Minion Cattiva | 11 | 15/15 | 2 | 0 | 1093 | 10 | null | Y | n | - | :44471 |
| 6161 | Clay Pot Minion | 5 | 36/36 | 2 | 0 | 1118 | 10 | Item82 | Y | n | - | :47486 |

### whip (18 rows total)

| id | name | dmg | ut/ua | kb | crit | shoot | spd | sound | auto | chan | mana | Item.cs |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 4672 | Bland Whip | 14 | 30/30 | 1 | 0 | 841 | 4 | Item152 | n | n | - | :38510 |
| 4678 | Sword Whip | 55 | 28/28 | 2 | 0 | 847 | 4 | Item152 | n | n | - | :38538 |
| 4679 | Mace Whip | 160 | 35/35 | 11 | 10 | 848 | 4 | Item152 | n | n | - | :38542 |
| 4680 | Scythe Whip | 110 | 27/27 | 3 | 0 | 849 | 4 | Item152 | n | n | - | :38547 |
| 4911 | Cool Whip | 45 | 30/30 | 1.5 | 0 | 912 | 4 | Item152 | n | n | - | :39831 |
| 4912 | Fire Whip | 37 | 30/30 | 2 | 0 | 913 | 4 | Item152 | n | n | - | :39835 |
| 4913 | Thorn Whip | 18 | 30/30 | 1.5 | 0 | 914 | 4 | Item152 | n | n | - | :39839 |
| 4914 | Rainbow Whip | 170 | 30/30 | 4 | 0 | 915 | 4 | Item152 | n | n | - | :39843 |
| 5074 | Bone Whip | 29 | 30/30 | 2 | 0 | 952 | 5 | Item152 | n | n | - | :40679 |
| 5473 | Cob Whip | 9 | 35/35 | 0.7 | 0 | 1028 | 3 | Item152 | n | n | - | :43159 |
| 5474 | Corrupt Whip | 17 | 30/30 | 1.35 | 0 | 1029 | 4 | Item152 | n | n | - | :43163 |
| 5475 | Crimson Whip | 19 | 30/30 | 1.25 | 0 | 1030 | 4 | Item152 | n | n | - | :43167 |
| 5476 | Meteor Whip | 18 | 30/30 | 1.5 | 0 | 1031 | 4 | Item152 | n | n | - | :43171 |
| 5477 | Flower Whip | 50 | 30/30 | 2 | 0 | 1032 | 4 | Item152 | n | n | - | :43175 |
| 5478 | Eel Whip | 150 | 30/30 | 3 | 0 | 1033 | 4 | Item152 | n | n | - | :43179 |
| 5479 | Constellation Whip | 130 | 30/30 | 4 | 0 | 1034 | 4 | Item152 | n | n | - | :43183 |

## Seed-variant weapon rows (all verified)

`ItemVariants.cs:121-145` registers 25 variants; the weapon ones and their C#
branch sites (all deltas match `VARIANT_OVERRIDES`, `itemdata.rs:7258-7334`):

| id | item (ItemID name) | variant (seed) | C# branch | delta vs base |
|---|---|---|---|---|
| 112 | Flowerof Fire | Stronger (remix) | `Item.cs:2716` | dmg 85, ut/ua 12/12, rare 6 |
| 157 | Aqua Scepter | Stronger (remix) | `:3270` | dmg 90, ut 5, ua 10, rare 8 |
| 197 | Star Cannon | Rebalanced (getGoodWorld) | `:3826` | dmg `(int)(55*0.9)`=49, ut `(int)(12*1.1)`=13 |
| 517 | Magic Dagger | Weaker (remix) | `:7559` | dmg 14, ut/ua 15/15, mana 5, autoReuse false |
| 683 | Unholy Trident | Weaker (remix) | `:9440` | dmg 77, mana 9, rare 3 |
| 725 | Ice Bow | Weaker (remix) | `:9876` | dmg 8, ut/ua 17/17 |
| 1314 | KO Cannon | Weaker (remix) | `:16282` | dmg 18, ut/ua 25/25, kb 5, autoReuse false |
| 1319 | Snowball Cannon | Stronger (remix) | `:16344` | dmg 22, ut/ua 6/6 |
| 1325 | Chain Knife | Stronger (remix) | `:16428` | dmg 60, ut/ua 15/15, autoReuse true |
| 2623 | Bubble Gun | Weaker (remix) | `:25693` | dmg 5 |
| 3069 | Wandof Sparking | Stronger (remix) | `:28415` | dmg 40, ut/ua 10/10, spd 12, mana 6, autoReuse true |
| 4060 | Super Star Cannon | Rebalanced (getGoodWorld) | `:35677` | dmg `(int)(60*0.9)`=54, ut `(int)(18*1.1)`=19 |
| 5147 | Wandof Frosting | Stronger (remix) | `:41120` | dmg 42, ut/ua 10/10, spd 12, mana 6, autoReuse true |
| 5279-5281 | Ash Wood armor | Weaker (remix) | `:41878/:41888/:41898` | defense 1 (non-weapon fields) |
| 5282 | Ash Wood Bow | Weaker (remix) | `:41917` | dmg 6, ut/ua 29/29 |
| 5283 | Ash Wood Hammer | Weaker (remix) | `:41939` | dmg 4, hammer 35 |
| 5284 | Ash Wood Sword | Weaker (remix) | `:41958` | dmg 8, ut/ua 23/23 |
| 5687 | Slime Spear | Weaker (skyblock) | `:44686` | dmg 9 |
| 5688 | Slime Whip | Weaker (skyblock) | `:44694` | dmg 12 |

Boss-summon items 544/556/557 (`!= DisabledBossSummon`, `:7873/:8005/:8018`)
and 5334 (`== Enabled`, `:42307`) are consumables, not weapons; their restore
rows were checked too (useStyle 4→0, ut/ua 45→100, consumable→false,
material→true) and match. Value deltas follow the port's buy-price convention
(`value` = `buyPrice` = 5× `sellPrice`, `Item.cs:49417-49425`); the census
compared weapon fields, and the ×5 relation holds on every variant row.

## WRONG / MISSING rows and fix slices

**WRONG: none. MISSING: none.** Every weapon id present in C# `SetDefaults`
has a port row with identical census fields, and no port weapon row lacks a
C# counterpart. Fix slices are recorded empty so a future regression has a
named home:

| slice | families covered | WRONG | MISSING | action |
|---|---|---|---|---|
| swords | sword/other-melee, shortsword, boomerang/sword-beam | 0 | 0 | none — guard only |
| bows | bow/repeater, star-cannon | 0 | 0 | none — guard only |
| guns | gun/launcher, rocket-launcher, thrown/other-ranged | 0 | 0 | none — guard only |
| staves | magic | 0 | 0 | none — guard only |
| summon | summon-staff/sentry, whip | 0 | 0 | none — guard only |
| other | spear, yoyo/flail, tool-melee, ammo | 0 | 0 | none — guard only |

Guard recommendation (not implemented here — code-only census): the existing
`itemdata.rs` spot-check tests (`:7357+`) and `tools/GenItemData/test_gen.py`
already pin representative rows; `python tools/GenItemData/gen.py --check`
remains the drift gate. Any future weapon-stat change must go through the
generator, never a hand-edit (`itemdata.rs:1-2`).

## Caveats / out of scope

- Fields outside the census list (width/height, scale, rare, value, buffType/
  buffTime, noMelee/noUseGraphic, shootsEveryUse, sentry/DD2Summon flags) were
  not diffed; the generator emits them and its `--check` gate covers drift.
- Prefix application (`prefix.rs`), ammo damage stacking and the item-use
  runtime (`Player.ItemCheck`) are behaviour, not data — covered by other
  census slices, not this one.
- The census extractor lives in the worker scratch dir (not committed); its
  results are reproducible from the cited C# spans with the shared loaders.

## Appendix — full weapon id rosters (624)

**sword/other-melee** (54): 4, 24, 121, 155, 186, 213, 426, 482, 483, 484, 653, 656, 659, 672, 795, 881, 921, 946, 1123, 1166, 1185, 1192, 1199, 1304, 1786, 1827, 1909, 2273, 2330, 2517, 2608, 2745, 3013, 3097, 3106, 3211, 3258, 3349, 3351, 3484, 3490, 3496, 3502, 3508, 3514, 3520, 3823, 4707, 4711, 5094, 5096, 5097, 5129, 5284

**shortsword** (12): 6, 486, 1227, 3483, 3489, 3495, 3501, 3507, 3513, 3519, 4463, 4923

**boomerang/sword-beam** (68): 46, 55, 65, 119, 190, 191, 198, 199, 200, 201, 202, 203, 273, 284, 368, 561, 670, 671, 674, 675, 676, 723, 724, 757, 989, 1122, 1226, 1306, 1324, 1327, 1513, 1569, 1825, 1826, 1918, 1928, 2424, 2880, 3012, 3018, 3030, 3054, 3063, 3065, 3352, 3543, 3764, 3765, 3766, 3767, 3768, 3769, 3772, 3827, 4258, 4259, 4760, 4764, 4818, 4956, 5298, 5382, 5462, 5535, 5536, 5669, 5670, 5671

**spear** (23): 274, 277, 280, 390, 406, 537, 550, 756, 802, 1186, 1193, 1200, 1228, 1297, 1314, 1325, 1571, 1947, 2331, 2332, 2611, 4061, 5687

**yoyo/flail** (40): 162, 163, 220, 389, 801, 1259, 3262, 3278, 3279, 3280, 3281, 3282, 3283, 3284, 3285, 3286, 3287, 3288, 3289, 3290, 3291, 3292, 3315, 3316, 3317, 3368, 3389, 3473, 3835, 3836, 3858, 4144, 4272, 4788, 4789, 4790, 5011, 5012, 5294, 5526

**tool-melee** (105): 1, 7, 10, 45, 103, 104, 122, 196, 204, 217, 367, 383, 384, 385, 386, 387, 388, 579, 654, 657, 660, 776, 777, 778, 787, 797, 798, 799, 882, 922, 990, 991, 992, 993, 1188, 1189, 1190, 1195, 1196, 1197, 1202, 1203, 1204, 1222, 1223, 1224, 1230, 1231, 1232, 1233, 1234, 1262, 1294, 1305, 1320, 1506, 1507, 1917, 2176, 2320, 2341, 2342, 2516, 2746, 2774, 2776, 2779, 2781, 2784, 2786, 2798, 3098, 3464, 3466, 3481, 3482, 3485, 3487, 3488, 3491, 3493, 3494, 3497, 3499, 3500, 3503, 3505, 3506, 3509, 3511, 3512, 3515, 3517, 3518, 3521, 3522, 3523, 3524, 3525, 4059, 4317, 5095, 5283, 5295, 5519

**bow/repeater** (40): 39, 44, 99, 120, 435, 436, 481, 578, 655, 658, 661, 682, 725, 796, 923, 1187, 1194, 1201, 1229, 2223, 2515, 2624, 2747, 2888, 3019, 3029, 3052, 3480, 3486, 3492, 3498, 3504, 3510, 3516, 3540, 3854, 3859, 4381, 4953, 5282

**gun/launcher** (36): 95, 96, 98, 164, 219, 266, 281, 434, 533, 534, 679, 800, 905, 964, 986, 1254, 1255, 1258, 1265, 1319, 1553, 1782, 1784, 1835, 1870, 1929, 2269, 2270, 2797, 3007, 3008, 3107, 3475, 3788, 4703, 5117

**rocket-launcher** (7): 758, 759, 760, 1946, 2796, 3546, 3930

**star-cannon** (2): 197, 4060

**thrown/other-ranged** (28): 42, 154, 160, 161, 168, 279, 287, 506, 1000, 1130, 1156, 1809, 1910, 1913, 2586, 2590, 3094, 3116, 3197, 3210, 3350, 3378, 3379, 3548, 3821, 4343, 4344, 5460

**ammo** (64): 40, 41, 47, 51, 71, 72, 73, 74, 97, 234, 265, 278, 283, 515, 516, 545, 546, 771, 772, 773, 774, 931, 949, 988, 1179, 1235, 1261, 1302, 1310, 1334, 1335, 1341, 1342, 1349, 1350, 1351, 1352, 1614, 1783, 1785, 1836, 3003, 3009, 3010, 3011, 3103, 3104, 3108, 3567, 3568, 4445, 4446, 4447, 4448, 4449, 4457, 4458, 4459, 4915, 5348, 5377, 5378, 5379, 5380

**magic** (81): 64, 112, 113, 127, 157, 165, 218, 272, 494, 495, 496, 514, 517, 518, 519, 683, 726, 739, 740, 741, 742, 743, 744, 788, 1121, 1155, 1178, 1244, 1256, 1260, 1264, 1266, 1295, 1296, 1308, 1313, 1336, 1444, 1445, 1446, 1801, 1930, 1931, 2188, 2622, 2623, 2750, 2795, 2882, 3006, 3014, 3051, 3053, 3069, 3105, 3209, 3269, 3377, 3476, 3541, 3542, 3570, 3779, 3787, 3852, 3870, 4062, 4270, 4347, 4348, 4715, 4952, 5065, 5118, 5147, 5461, 6152, 6153, 6154, 6155, 6173

**summon-staff/sentry** (46): 1157, 1309, 1572, 1802, 2364, 2365, 2366, 2535, 2551, 2584, 2621, 2749, 3249, 3474, 3531, 3569, 3571, 3818, 3819, 3820, 3824, 3825, 3826, 3829, 3830, 3831, 3832, 3833, 3834, 4269, 4273, 4281, 4607, 4758, 5005, 5069, 5114, 5119, 5456, 5463, 5663, 5664, 6148, 6149, 6161, 6164

**whip** (18): 4672, 4678, 4679, 4680, 4911, 4912, 4913, 4914, 5074, 5473, 5474, 5475, 5476, 5477, 5478, 5479, 5480, 5688
