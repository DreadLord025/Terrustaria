# 87 — IslandHouses 4-way reconcile (W4-g44 addendum, 2026-11-21)

Four branches independently ported `FloatingIslandHouses`
(`WorldGen.cs:17859-17874`) → `IslandHouse` (`:79203-79484`). This is the
arm-by-arm sunplate-fidelity table against the C# source, with the KEEP
recommendation for the merge gate. Code-only mode: every verdict below
is a direct source read (the C# line ranges are the ledger), no builds.

**The four ports:**

| Lane | Commit | File | Shape |
| --- | --- | --- | --- |
| **g44** (this branch) | `ae4472e6` + `cb5985c9` (variant A) | `worldgen/islands.rs` (`island_house` + `furniture` mod) | both passes in one module; verbatim pass mechanics; compact disc bodies; ALL house arms after variant A |
| **d103** | `eb018138` | `worldgen/island_houses.rs` (572 lines) | separate module; compact islands pass keeps its stand-ins; full `AddBuriedChest` core (`buried_chest::add_buried_chest`) |
| **d104** | `93b1938e` | `worldgen/island_houses.rs` (596 lines) | separate module; `IslandHouseSlot` ledger; `add_sky_chest` slice (larva + altar vetoes); temple-door/evil-3x2 reuse |
| **g37 B5** | `3d74564e` | `worldgen/passes_b.rs` §B5 (286 lines) | sites-tuple ledger; classic-only furniture (remix/drunk/getGood arms deferred); own `add_buried_chest` |

## Arm-by-arm: `IslandHouse` (`WorldGen.cs:79203-79484`)

✓ = verbatim arm present; ~ = present with a recorded deviation; ✗ = absent/deferred.

| # | C# arm | lines | g44 | d103 | d104 | g37 |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | `flag = WorldGen.crimson` read | :79205 | ✓ `opts.crimson` param | ✓ `flags.crimson` | ✓ `crimson` param | ✗ (no crimson input) |
| 2 | ±30 dungeon/87/3/83 wall abort | :79207-79216 | ✓ `is_dungeon_wall` | ✓ 9-entry `WALL_DUNGEON` | ✓ 9-entry const | ✓ `scene::is_dungeon_wall` |
| 3 | drunk crimson half-split | :79217-79220 | ✓ (variant A) | ✓ | ✓ | ✗ |
| 4 | sunplate 202 / disc wall 82 | :79221-79222 | ✓ | ✓ | ✓ | ✓ |
| 5 | `Next(2)` door flip + `Next(7,12)`×`Next(5,7)` rolls, draw order | :79224-79230 | ✓ | ✓ | ✓ | ✓ |
| 6 | ground scan `j-15..j+30`, wall>0 + SolidTile, kinds 192/191/384 excluded; `j>worldSurface → j-5` | :79231-79247 | ✓ (scan-miss → skip: sub-vanilla valve, C# builds at Y=0) | ~ (scan-miss → `floor_y = j` fallback) | ~ (scan-miss → `vy = j` fallback) | ~ (scan-miss → `vec_y = j`, documented) |
| 7 | shell/interior bounds clamped to world BEFORE the corner-skip test | :79249-79268, :79288-79303 | ~ (per-cell bounds guard, corner test on unclamped bounds — identical for anchors in 0.1..0.9×width; differs only at world-edge anchors the pass never produces) | ✓ clamps | ✓ clamps | ✓ clamps |
| 8 | sunplate shell, top corners skipped, liquid/wall/half/slope reset | :79269-79283 | ✓ | ✓ | ✓ | ✓ |
| 9 | interior carve, wall-0 cells only, top corners skipped | :79304-79314 | ✓ | ✓ | ✓ | ✓ |
| 10 | door carve 5 wide × 3 tall | :79315-79322 | ✓ | ✓ | ✓ | ✓ |
| 11 | door style: remix∧(¬anniv∨getGood) → crimson 5 / corrupt 38, else 9 | :79323-79337 | ✓ (variant A) | ✓ | ✓ (via `place_temple_door`) | ✗ (classic 9 only) |
| 12 | opposite wall column `num7..=num8+1` | :79338-79347 | ✓ | ✓ | ✓ | ✓ |
| 13 | chest cycle `skyIslandHouseCount`, `>3 → Next(4)`, 159/65/158/2219 | :79348-79368 | ✓ | ✓ | ✓ | ✓ |
| 14 | chest style getGood 2 / else 13 | :79369-79376 | ✓ | ✓ | ✓ | ✗ (13 only) |
| 15 | `AddBuriedChest` core (floor scan, larva 231 veto, ±2 altar 26/237 veto, style resolution, stocking) | :79371/:79375 → :35889-35963 | ✓ FULL core after the islands-final graft (`e172bc76` — `buried_chest::add_buried_chest`, the d103 salvage; was ~ compact stand-in) | ✓ full `buried_chest::add_buried_chest` | ✓ `add_sky_chest` slice (larva + altar vetoes, sky-band loot tree deferred to the chest lane) | ✓ own `add_buried_chest` |
| 16 | fountain scatter (style>0): 100000 attempts, first 50000 refuse 202, `Place2xX(207, style)` + `SwitchFountain` | :79377-79392 | ✓ (variant A; `Place2xX` fountain arm + `SwitchFountain` ±72) | ✓ | ✓ | ✓ |
| 17 | `skyIslandHouseCount++` after the place | :79394 | ✓ | ✓ | ✓ | ✓ |
| 18 | window strips wall 21, gate ¬remix∨(anniv∧¬getGood) | :79395-79419 | ✓ (variant A gate) | ✓ | ✓ | ✗ (always on) |
| 19 | table/chairs: remix arm crimson 14/5 + 15/8, corrupt 469/2 + 15/38; classic 14/7 + 15/10 | :79420-79441 | ✓ (variant A; `place_table` takes the kind — 469 rides `PlaceTile` case 469 → `Place3x2`, :60000-60003) | ✓ (`place_3x2` kind param) | ✓ (`evil::place_3x2` + `place_1x2`) | ✗ (classic only) |
| 20 | left chair frameX += 18 on both rows, UNCONDITIONAL | :79442-79443 | ✓ | ✓ | ✓ | ✓ |
| 21 | three banners 91 `Next(7,10)`, same gate as windows | :79444-79461 | ✓ (variant A gate) | ✓ | ✓ | ✗ (always on) |
| 22 | style-1 palms: `Next(3,6)` budget, 100000 scatter, `TryGrowingTreeByType(323)` = `GrowPalmTree(x, y+1, 0, false)` | :79462-79483 | ✓ (variant A; shared `ocean::grow_palm_tree`) | ✓ (same seam) | ✓ (same seam) | ✓ (same seam) |
| 23 | `SquareTileFrame` after each write | pass-wide | ~ chain-framing deviation (all four ports; the temple precedent) | ~ | ~ | ~ |

## Pass-level arms: `FloatingIslands` (`:12927-13122`) + the houses pass

| C# arm | lines | g44 | d103 | d104 | g37 |
| --- | --- | --- | --- | --- | --- |
| count `(int)(width * 0.0008)` | :12934 | ✓ | ✗ compact `3 + width/4200` | ✗ same | ✗ same |
| skyLakes budget + `num2 >= num` lake tail | :11182-11190, :12955, :13051 | ✓ | ✗ (count stand-in) | ✗ | ✗ |
| errorWorld triple before `num3` | :12936-12939 | ✓ | ✗ | ✗ | ✗ |
| extra multipliers 2x/10x + skyLakes carry | :12942-12954 | ✓ | ~ (cap only) | ~ | ~ |
| spawn re-draw 150→100, errorWorld skip | :12969-12979 | ✓ | ✗ compact 340 band | ✗ | ✗ |
| anchor spacing strict bounds 180→100 | :12981-12993 | ✓ | ~ | ~ | ~ |
| column-scan y-band + surfaceLow−50 clamp + noSurface/errorWorld arms | :12996-13048 | ✓ | ✗ compact band | ✗ | ✗ |
| skyblock `/42` budget, fixed y, ±150/±75 window | :12960-13040 | ✓ (variant B witness) | ✗ | ✗ | ✗ |
| drunk snow/desert variant bodies | :13075-13087 | ✓ (variant A — verbatim `SnowCloudIsland`/`DesertCloudIsland`) | ✗ | ✗ | ✗ |
| num10 style table (remix/getGood/anniversary/notTheBees) | :13090-13103 | ✓ (variant A) | ✗ style 0 | ✗ | ✗ |
| GenVars ledger + post-loop `num3*2` cap | :13108-13120 | ✓ | ✓ | ✓ (`IslandHouseSlot`) | ✓ (sites tuples) |
| houses-pass deny gate + lake skip | :17861, :17868-17871 | ✓ | ✓ (step adapter) | ✓ | ✓ |
| RNG world-pass slots (`pass_rng("Floating Islands")` / `("Floating Island Houses")`) | WorldGenerator.cs:497 model | ✓ | ✓ | ✓ | ✓ |
| disc bodies (`CloudIsland`/`CloudLake` ring walks) | :78132-79202 | ~ compact M1.4-WGEN-4 shape (all four; the variant bodies ARE verbatim in g44) | ~ | ~ | ~ |

## KEEP recommendation

**KEEP g44's `worldgen/islands.rs` (both passes) as the merge survivor.**
After variant A it is the only port with the verbatim pass mechanics
(count/skyLakes/errorWorld/y-band/spacing/budget) AND the complete
`IslandHouse` style switch AND the verbatim drunk variant bodies. The
other three are strict subsets on the house arms and keep the compact
islands-pass stand-ins:

1. **Drop `d103:island_houses.rs` and `d104:island_houses.rs` wholesale**
   at the merge (duplicate modules; every arm they carry is in g44's
   `island_house` except #15 below). Their pipeline rows/progress keys
   fold into g44's `floating island houses` step (same vanilla slot
   family — d103 after cave walls, d104 between living trees and trees,
   g44 after the islands step in the grouped chain; the merge gate picks
   ONE slot, g44's adjacency witness `w4_g44_the_island_houses_step_follows_the_islands`
   pins it).
2. **Drop `g37:passes_b.rs` §B5** (`island_house`, `switch_fountain`,
   `pass_floating_island_houses`) — classic-only, strictly dominated.
   The rest of passes_b.rs (B1-B4, B6-B9) is untouched by this call.
3. **Salvage candidate — CLOSED (islands-final 1/5, `e172bc76`): #15,
   the `AddBuriedChest` core.** d103's full core is grafted into g44 as
   `worldgen/buried_chest.rs` and the house chest call re-pointed at it
   (the larva-231 + ±2-altar-26/237 vetoes d104's slice carried are
   inside it, plus the flag12 remix retype / errorWorld shuffle /
   shimmer abort / noTraps swap neither sky slice had). See the FINAL
   adjudication addendum below — no salvage candidates remain.
4. **Ledger-y convention:** g44 stores `num11 - radius/2` for the
   compact discs (the disc-top composition fix, `a6412835`) and `num11`
   verbatim for the variant bodies; the peers store their own compact
   anchors. The g44 convention is the one the ground-scan witnesses
   pin — keep it.
5. **anchors.txt:** keep the two g44 rows (`fn pass_floating_islands`,
   `fn pass_floating_island_houses`); the d103/d104/g37 island-houses
   rows must be repointed at the g44 markers when their branches merge
   (never deleted to silence the canary — repointed, per the keep-one
   precedent).

**Skyblock claim boundary (read from the pushed branches):** f24 owns
`GenerateSkyBlockWorld` (`9cbbf9de`, `worldgen/skyblock.rs` wiring) +
`SkyblockIsland` (`a9eb3fc3`) + the deny-gate census (`372a4ec7`,
`skyblock_gates.rs::deny_floating_islands`). g44's island-side arms
(the `/42` budget, the fixed y line, the clear window, the pass gates)
are position/gate logic INSIDE `FloatingIslands` — they stay g44's and
compose with f24's terrain (f24's gate helper and g44's
`deny_floating_islands` are the same two-flag predicate; the merge gate
should keep ONE copy, f24's shared helper, and re-point g44's two call
sites at it).

## Sources read

- `decompiled/Terraria/WorldGen.cs` :12927-13122, :17859-17874,
  :77346-78130, :79203-79484, :39663-39722, :51150-51197,
  :30343-30386, :27116-27203, :78530-78537, :52066-52195,
  :31773-31803, :59843-59883, :60000-60003, :46258-46300,
  :70111-70151.
- `git show eb018138:crates/tr-core/src/worldgen/island_houses.rs` (d103).
- `git show 93b1938e:crates/tr-core/src/worldgen/island_houses.rs` (d104).
- `git show 3d74564e:crates/tr-core/src/worldgen/passes_b.rs` §B5 (g37).
- `origin/task/W4-f24-worldgen-passes-7` @ `9f2276d5` (the claim ledger
  `docs/research/12k-f24-worldgen-gated.md` + `skyblock_gates.rs`).
- This branch: `cb5985c9` (variant A), `2b041eaf` (variant B).

---

## FINAL adjudication addendum (2026-11-21, islands-final phase)

The dispatch flagged a CONFLICT between this note's KEEP-g44 and d104's
`docs/research/86-wg-passes-4-reconcile.md` (tip `0c7a3483`), which keeps
d104 for pass 28-IslandHouses. Re-read of d104's latest confirms the two
notes DO NOT actually conflict — they adjudicate different comparisons:

1. **86 §1 B5 compares d104 vs g37 ONLY.** Its arm table has exactly two
   port columns (`d104 (island_houses.rs)` / `g37
   (passes_b.rs::island_house)`); g44 appears NOWHERE in 86 (verified by
   a full-text scan of the tip's file). Its verdict — "d104 more complete
   + better-seamed", the drunk/remix/getGood arms beating g37's
   classic-only house — is correct FOR THAT PAIR and stands.
2. **87 compares all FOUR ports including g44 AFTER variant A.** g44
   carries every arm d104 has (the same drunk half-split, remix door
   5/38, getGood chest style 2, remix furniture, anniversary gates,
   fountain, palms — rows 3/11/14/16/18/19/21/22 above) PLUS the verbatim
   islands-pass mechanics d104 keeps as compact stand-ins (count
   formula, skyLakes tail, errorWorld triple, spawn re-draw, spacing,
   y-band, skyblock arms, the variant bodies, the num10 style table —
   the pass-level table above). On the house arms d104 ⊆ g44; on the
   pass arms d104 ⊊ g44. "Peers strict-subsets" is a statement about
   g44-vs-peers, not a denial of 86's d104-vs-g37 ordering.
3. **d104's house module is UNCHANGED since 87's read**: `git diff
   93b1938e 0c7a3483 -- crates/tr-core/src/worldgen/island_houses.rs`
   is EMPTY (the reconcile commit touched docs only). The arm table
   above is still current against d104's tip.
4. **The one arm where a peer was MORE faithful (#15, the AddBuriedChest
   core) is now closed IN g44** (islands-final 1/5, `e172bc76`): d103's
   full `buried_chest` core (unchanged `eb018138` → `339d9f6d`, verified
   by an empty diff) is grafted as `worldgen/buried_chest.rs` and the
   `island_house` chest call re-pointed at it — the larva-231 +
   ±2-altar-26/237 vetoes d104's `add_sky_chest` slice carried are
   inside the core, plus the arms NEITHER peer's sky slice had: the
   flag12 remix evil retype (`:36176-36196` — the plain remix world's
   sky chest becomes crimson-43 wood / corrupt-3 mahogany 467; d104's
   slice always placed 21/13), the errorWorld style shuffle, the
   shimmer abort, the noTraps swap, `NearOtherChests`. After the graft
   g44 is a superset of ALL THREE peers on EVERY arm — no salvage
   candidates remain.

**FINAL verdict (supersedes nothing, completes both notes): KEEP g44's
`worldgen/islands.rs` + the grafted `worldgen/buried_chest.rs` for the
Floating Islands + Floating Island Houses family.** At the merge:

- Drop `d103:island_houses.rs`, `d104:island_houses.rs`, `g37:passes_b.rs`
  §B5 (86 §2.3's "keeps ONE copy (d104's)" resolves to g44's once g44 is
  in the comparison — the merge gate reads BOTH notes).
- d103's `buried_chest.rs` ALSO carries the `Surface Chests` pass
  (`:17130-17210`) — that half is d103's own claim (86 §2.3 keeps d103's
  B1-B4 twins); at the merge the two `add_buried_chest` cores dedupe onto
  ONE (this graft is the same code, re-pointed seams: `edit::solid_tile`
  vs the local copy, `place_chest_kind` vs `place_chest`-21-only — the
  kind-aware twin is the survivor, the remix 467 arm needs it).
- 86's other verdicts are UNTOUCHED by this note: Quick Cleanup / Hellforge
  ordering / Piles / Spawn Point stay d104's, Spreading Grass takes the
  one-argument repeat:true fix, g39's helpers are the dedupe target.
- The ledger-y convention (KEEP-4) and the anchors repointing (KEEP-5)
  stand as written.

### Islands-final cross-notes (the four sibling mechanics)

- **Sky-lake fishing (2/5, `4c170c02`)**: the sky-band catch rows are
  LANDED (`fishing.rs::vanilla_drop_rules`, M7-FISH-2) — re-census against
  `GameContentFishDropPopulator.cs:73-81` (the six sky quest fish +
  Damselfish 2304) and `:214` (the sky crate 3206 / Azure 3985 pair,
  `Crate && Height0`) found ZERO gaps; three witnesses pin the rows and
  the end-to-end sky-lake bowl (pond gate → height_level 0 → quest
  catch). Claim read: f35 (`cd6d53a2f`) and f36 (`fac5f2224`) claim
  expert bags / OOA medals only — no fishing claims on either tip.
- **Skyware chest loot (3/5, `0898f352`)**: the flag12 tail (SkyMill
  2197 1/3, the six-piece pottery, Cloud 751 ×50-100) verified landed in
  `chest_gen.rs`; two gaps fixed — the sky main's `Prefix(-1)`
  (`:36232`/`:36513`) and the CreativeWings 4978 1/40 rider
  (`:36526-36531`); the 467-block census fixes (the Cloud stack rides
  both blocks, the style-13 5234 row gates on 467) landed with 1/5.
- **Harpy spawn band (4/5, `c55cdc02`)**: the 0.35/0.45 height-band latch
  (`NPC.cs:980-987`) is landed verbatim (M4-EVENT-4b); the dispatch tail
  gained the Wyvern arm's `!AnyNPCs(87) && !noWorms` short-circuits, the
  town-slime `RollLuck(25)` draw and the TERMINAL Harpy default (no
  ground fallthrough). The Harpy kind (48) is d34's claim
  (`4012a7f3a`, npc-kinds-1-70, unmerged); d126 owns the Wyvern spawn
  assembly. Claim read: g61 (`a96289284`) is spawn DUST only — no
  spawn-wiring claims; the dispatch tail was unclaimed.

### Sources read (addendum)

- `decompiled/Terraria/WorldGen.cs` :35755-35798, :35877-36217,
  :36234-36333, :36424-36539, :37390-37539, :37548-37568,
  :57813-57846, :79344-79458, :9688-9695.
- `decompiled/Terraria/Chest.cs` :543-557; `Tile.cs` :428-435
  (`anyShimmer`); `ID/TileID.cs` :504 (`Sets.Boulders`).
- `decompiled/Terraria/GameContent/FishDropRules/GameContentFishDropPopulator.cs`
  :73-87, :204-223; `Projectile.cs` :20680-20754; `Main.cs` :67032
  (the 41-fish quest pool); `ID/ItemID.cs` (the sky ids).
- `decompiled/Terraria/NPC.cs` :969-1010, :1350-1424; `ID/NPCID.cs`
  :10773/:10851/:12049.
- `git show origin/task/W4-d103-worldgen-passes-3:crates/tr-core/src/worldgen/buried_chest.rs`
  @ `339d9f6d` (the full core + the Surface Chests pass).
- `git show origin/task/W4-d104-worldgen-passes-4:crates/tr-core/src/worldgen/island_houses.rs`
  @ `0c7a3483` (`add_sky_chest` :379-431) +
  `docs/research/86-wg-passes-4-reconcile.md` (full).
- `origin/task/W4-d34-npc-kinds-1-70` @ `4012a7f3a` (the Harpy rows),
  `origin/task/W4-d126-wyvern-chain-wire` @ `7ddf25a49` (no spawner.rs
  diff — the assembly is its own pass), `origin/task/W4-g61-npc-spawn-dust`
  @ `a96289284`, `origin/task/W4-f35-expert-drop-trees` @ `cd6d53a2f`,
  `origin/task/W4-f36-ooa-medal-drops` @ `fac5f2224` (claim greps).

---

## BIOME CLOSEOUT — FINAL STATUS + the final-keep instruction (2026-11-21)

The lane's closeout dispatch (desert boundary / sky-lake water /
island-house housing / this note). Code-only: every verdict is a direct
source read; the four witnesses are written, NOT run. Branch tip at
publication: **`8e791809a`** (closeout 3/4; this note is 4/4).

### FINAL-KEEP instruction (publish to the active merge lanes g36d / g37c / d104d)

**KEEP g44's `worldgen/islands.rs` + the grafted `worldgen/buried_chest.rs`
for the Floating Islands (#26) + Floating Island Houses (#73) + skyLakes
family. This is FINAL — no salvage candidates remain, no peer arm is
uncovered.** At the merge:

1. **DROP** `d103:island_houses.rs`, `d104:island_houses.rs`,
   `g37:passes_b.rs` §B5 (`island_house` / `switch_fountain` /
   `pass_floating_island_houses`) + their `islands.rs` ledger edits.
2. **Dedupe the two `add_buried_chest` cores onto ONE** — the kind-aware
   survivor (`place_chest_kind`, the remix 467 arm needs it). g44's graft
   IS d103's core re-pointed (`edit::solid_tile` vs the local copy,
   `place_chest_kind` vs `place_chest`-21-only); keep d103's **Surface
   Chests** half (`:17130-17210`, its own claim, 86 §2.3 keeps d103's
   B1-B4 twins).
3. **Keep ONE deny-gate helper** — f24's shared
   `skyblock_gates::deny_floating_islands`; re-point g44's two call sites.
4. **Ledger-y convention** (KEEP-4): g44 stores `num11 - radius/2` for the
   compact discs, `num11` verbatim for the variant bodies — keep it.
5. **anchors.txt** (KEEP-5): keep the five g44 rows (`pass_floating_islands`,
   `pass_floating_island_houses`, `variant_cloud_island`, `switch_fountain`,
   `add_buried_chest`); repoint the d103/d104/g37 island-houses rows at the
   g44 markers (never delete to silence the canary).
6. **NEW — the housing wall-table dedupe:** `housing::is_house_wall` now
   delegates to `edit::wall_house` (the FULL 279-entry `Main.wallHouse`).
   d118 (`91db266ea`) / d119 (`04d7f9fd2`) keep the narrow subset copy at
   their tips — the merge dedupes onto g44's full-table read (see §3 below).

### §1 — The desert boundary (closeout 1/4, `998b7a0c`)

The dispatch's "g44 owns desert" is the lane SLUG (`biome-sky-lakes`), not
a desert-pass claim. g44's `desert.rs` / `desert_entrances.rs` are
UNTOUCHED base code (empty diff vs origin/main). Owners, re-verified:

| C# unit | Owner | Evidence (tip read) |
| --- | --- | --- |
| `Sand Patches` `WorldGen.cs:11689-11717` | **d101** | `d0042f85e`, `passes1.rs::pass_sand_patches` @ `4194acf8e` — count `0.013*width`, spawn-middle rejection re-rolls the CLASSIC band even in remix (`:11711-11712`), strength-before-steps draw order. Verified line-for-line vs the C#. |
| `DesertBiome.cs:25-40` entrance family (the antlion nests / larva holes) | **g40** | `7bb5332e5` + `a8150064b` @ `e367fb56b` — the 17-deviation residual closed verbatim (the y-major Circle walk, Blotches at 5 sites, SetTile≠ResetToType, the chambers OuterOutline/tunnel chain, the pit SmootherStep π), the anniversary short-circuit BEFORE the fraction draw + the JSON 0.5 `ChanceOfEntrance` through `CreateBiome`. |
| `DesertHive` `PostPlacementEffect.Smooth` residual | **g40 next slice** | g40 census (c) — the `tile_smooth_slope_one` seam exists (g40 P1); unclaimed beyond g40. |
| The buried-chest desert dispatch arm `:36023-36048` | **g44** (the graft) | reads WORLD STATE not GenVars: `IsUndergroundDesert` `:35755-35781` = `y>=worldSurface` + 15%..85% x-band + the ±15 wall-187/216 probe. The g40 residual moves the wall geometry, never the probe's shape. |

Witness `w4_g44_desert_arm_retypes_on_the_hive_wall_probe` (buried_chest.rs):
both wall arms (187 sandstone / 216 hardened) retype to 467 style 10, the
wall-free control keeps 21 style 1, the above-surface gate refuses; the
pocket seats mid-band (x=400 on 800 wide) so the x-window admits.

### §2 — Sky-lake WATER physics (closeout 2/4, `582fa094`)

**The lakes SURVIVE settle.** Verified against the C# chain (not assumed):
`WaterCheck` (`WorldGen.cs:80415-80494`) zeroes liquid only INSIDE a solid
tile (`:80426-80432` — the bowl cells are inactive water) and wakes a cell
only when the cell below is open-and-not-full or a side cell is open with a
DIFFERENT level (`:80449-80470`). The compact bowl's rain-cloud 196
floor/sides are `tileSolid` true (`Main.cs:7118`; the generated `TILE_SOLID`
bit verified set) and the rows are uniformly 255 → no wake arm fires.
`SettleWaterAt`'s fall/spread walks (`Liquid.cs:171-278`) have nowhere to
go; the waterLine lava re-type (`:184-187`) needs the fall arm (`flag4`) —
inert on an enclosed lake (the load settle pins `waterLine = maxTilesY`).

**g47 co-lane read (tip `5c09a68b8`):** `gen_settle_loop` (`liquid.rs:1698`)
is `WorldGen.cs:16158-16206` verbatim (WaterCheck re-registration,
quickSettle up, 10 rounds × `num3*5` budget with live re-raise, WaterCheck
between, ClearPendingLiquid + quickSettle down); `quick_water` /
`water_check` / `settle_water_at` are the `Liquid.cs` ports on ONE
`UpdateLiquid` engine shared with the runtime. The gen settle does NOT
drain island lakes. The `WillWaterPlacedHereStayPut`-gated placement
(`islands.rs:192`, `WorldGen.cs:78530-78537`) is exactly the enclosed-bowl
invariant the settle preserves.

Witness `w4_g44_the_sky_lake_water_survives_settle` (islands.rs): builds the
lake disc, pins the 31×3×255 mass + the stay-put predicate on every water
cell, wakes every bowl cell through the landed `LiquidSystem`, drains 600
updates — mass unchanged, zero leak below the floor / past the sides.

### §3 — Island-house NPC housing (closeout 3/4, `8e791809`)

**THE FIX:** `housing::is_house_wall` delegates to `edit::wall_house` (the
FULL 279-entry `Main.wallHouse`, S1-P-i33, verified member-for-member vs
`Main.cs` incl. the `153..167` loop; the existing
`s1_p_i33_wall_house_is_the_full_main_table` witness pins wall 82 + the
count 279, so the delegation inherits that coverage). `CheckRoom`'s ±2 scan
reads `Main.wallHouse` DIRECTLY (`WorldGen.cs:6204`/`:6212`); the old narrow
subset lacked wall **82** (`WallID.DiscWall`, `Main.cs:10384` true), so every
g44 sky-island house failed `TooManyUnsafeWalls`. The full-table read admits
the disc-wall room exactly as vanilla — the same correction the hammer lane
made at the i31→i33 gate (`hammer.rs:157-162`), now on the room-check side.

**Census:** the housing checker carries NO sky-band gate (`CheckRoom` reads
only edge/solid/wall/furniture conditions; the sole surface read is the
per-type Truffle condition `:4974`) — altitude never refuses a room. The
GENERATED house is not move-in-ready exactly as vanilla: the 5-wide door
carve (`:79317-79322`) keeps open flanks (`PlaceTile(10)` `:59843-59883` →
`PlaceDoor` `:31773-31803` fills ONE column) and the house ships NO torch
(`RoomNeeds` incomplete until the player adds a light).

**d118/d119 cross-note:** their tips (`91db266ea` / `04d7f9fd2`) keep the
narrow `is_house_wall` copy. d118 owns the QuickFindHome pass/canSpawn shape
+ occupancy queries; d119 owns the homeless-arrival + relocation drivers.
Neither touches the wall table's sky-band correctness — the merge dedupes
their `is_house_wall` onto g44's full-table read (FINAL-KEEP item 6). The
housing fix is a pure predicate widening on the room-check path (no RNG
draws), so d118/d119's stream witnesses stay valid.

Two witnesses: `w4_g44_the_disc_wall_is_a_house_wall_and_the_sky_room_scores`
(housing.rs — the disc-wall sky room scans + RoomNeeds-completes + scores a
home spot; the wall-stripped control fails `HoleInWallIsTooBig`) and
`w4_g44_the_generated_island_house_ships_open_flanks_and_no_torch`
(islands.rs — pins the open flanks + the missing torch + the
table/chair/door flags).

### §4 — matrix-88 / matrix-86 cross-receipts

**matrix-88 (g36, tip `6d46f6a54`) — RECEIPT:**
- §2.11 KEEP g44 (rows 26 + 73): **CONFIRMED, now FINAL.** The §2.11 merger
  note "verify g44 carries d103's drunk-crimson wall split + case-10 door
  dispatch (if absent, port those two arms)" is **RESOLVED — both present**:
  the drunk crimson half-split at `islands.rs:1199-1202` (`:79217-79220`),
  the case-10 door dispatch at `islands.rs:1011-1027` (`place_tile_door`,
  the up-hang/down-hang `PlaceTile` case-10 arms `:59843-59883`) + the remix
  5/38 style split at `:1293-1297`. No gate-port needed.
- §2.8 "re-point g44's IslandHouse chest call onto d103's buried_chest core":
  **DONE in-lane** (`e172bc76`, the graft) — g44 carries the full core as
  `buried_chest.rs`, the house call re-pointed. The merge dedupes the two cores onto
  the kind-aware survivor (FINAL-KEEP item 2).
- §6.3 checklist "g44 IslandHouse: drunk crimson wall split + case-10 door
  dispatch": **CLOSED** (both verified present, above).
- §6.2 merge order "g44 islands ledger → g44 houses": unchanged; g44's
  closeout adds no pipeline row (the housing fix is a predicate, the desert
  + sky-lake items are witnesses).

**matrix-86 (d104, tip `242692e1c`) — RECEIPT:**
- §1 B5 (d104 vs g37): the d104-wins verdict **stands FOR THAT PAIR** — 86
  compares d104 vs g37 ONLY (g44 appears nowhere in 86, re-verified by a
  full-text scan at the new tip `242692e1c`). 87's four-way table (g44 after
  variant A + the graft) dominates d104 on every house arm AND the pass
  mechanics. The two notes adjudicate different comparisons — no conflict.
- d104's house module is UNCHANGED through the new tip: `git diff 93b1938e
  242692e1c -- island_houses.rs` is EMPTY (the §6 addendum commits are
  docs-only). The arm table above is still current.
- 86's other verdicts (Quick Cleanup / Hellforge / Piles / Spawn Point / the
  Spreading Grass repeat:true fix) are UNTOUCHED by this note.

**Peer-tip re-verification ledger (the house modules at the closeout read):**

| Lane | Tip | House-module drift since 87's read |
| --- | --- | --- |
| d103 | `12318e505` | `buried_chest.rs` UNCHANGED (`339d9f6d`→`12318e505` empty); `island_houses.rs` a borrow-fix only (`eb018138`→`12318e505`: copy the ledger entry before the mutable `gv` borrow, +9/-3, no arm change). |
| d104 | `242692e1c` | `island_houses.rs` UNCHANGED (`93b1938e`→`242692e1c` empty); the §6 addendum is docs-only. |
| g37 | `81fd1e0d0` | `passes_b.rs` +720 lines but ALL in B6-B9 (the pile/gap-fill arms); §B5 `island_house` still classic-only (door style 9 at `:1751`, the remix 5/38 arms deferred, no crimson input) — strictly dominated by g44. |

### Sources read (closeout)

- `decompiled/Terraria/WorldGen.cs` :11689-11717 (Sand Patches),
  :12500-12543 (Full Desert), :16156-16207 (the gen settle),
  :35755-35798 (IsUndergroundDesert/IsDungeon), :36017-36064 (the desert
  dispatch), :59843-59883 (PlaceTile case 10), :6041-6243
  (StartRoomCheck/CheckRoom), :5858-6029 (ScoreRoom), :79248-79484
  (IslandHouse shell/carve/door/furniture), :80415-80513
  (WaterCheck/ClearPendingLiquid), :31773-31803 (PlaceDoor).
- `decompiled/Terraria/GameContent/Biomes/DesertBiome.cs` :13-71,
  `Desert/DesertDescription.cs` :40-97, `Desert/SurfaceMap.cs` :22-74,
  `Desert/SandMound.cs` (full).
- `decompiled/Terraria/Liquid.cs` :69-289 (tilesIgnoreWater /
  worldGenTilesIgnoreWater / QuickWater / SettleWaterAt).
- `decompiled/Terraria/Main.cs` :7118 (tileSolid[196]), :10384
  (wallHouse[82]), :10427 (wallHouse[73]), the wallHouse init block
  (279 members, the `153..167` loop).
- `git show origin/task/W4-g47-liquid-skip-settle:crates/tr-core/src/liquid.rs`
  @ `5c09a68b8` (gen_settle_loop / quick_water / water_check /
  settle_water_at / settle_loaded_world).
- `git show origin/task/W4-d101-worldgen-passes-1:crates/tr-core/src/worldgen/passes1.rs`
  @ `4194acf8e` (pass_sand_patches).
- `git show origin/task/W4-g40-biome-marble-granite:docs/research/94-w4g40-biome-residual-census.md`
  + `crates/tr-core/src/worldgen/desert.rs` @ `e367fb56b` (the entrance gate
  + the anniversary witness).
- `git show origin/task/W4-g36-worldgen-missing-a:docs/research/88-worldgen-master-matrix.md`
  @ `6d46f6a54` (§2.8/§2.11/§6.2/§6.3).
- `git show origin/task/W4-d104-worldgen-passes-4:docs/research/86-wg-passes-4-reconcile.md`
  @ `242692e1c` (§1 B5 + the g44 full-text scan).
- `origin/task/W4-d118-housing-scoreroom` @ `91db266ea`,
  `origin/task/W4-d119-housing-relocation` @ `04d7f9fd2` (the
  `is_house_wall` subset copies + the claim scopes).
- This branch: `998b7a0c` (desert witness), `582fa094` (sky-lake witness),
  `8e791809` (the housing fix + 2 witnesses).

## RETIRE §5 — the wall_house dedupe: EXACT hunks per branch (2026-11-21)

Re-read at the LATEST origin tips (fetch at retire): **d118 `91db266ea`**,
**d119 `04d7f9fd2`** — unchanged since the closeout read. Blob census (the
merge-safety facts):

| File | d118 blob | d119 blob | g44 base (origin/main) | verdict |
| --- | --- | --- | --- | --- |
| `tr-core/src/housing.rs` | `2ecd7924e` | `2ecd7924e` (IDENTICAL pair) | `f2a7c31af` | d118 rewrote `solid_tiles_in_box` (:110+, the `Collision.cs:3455-3488` port) + the occupancy fns; the `is_house_wall` block (:18-27) is byte-identical to main's. **Never take g44's housing.rs wholesale** — apply ONLY the hunk below. |
| `tr-core/src/edit/wall.rs` | `9f39cbde8` | `9f39cbde8` | = g44's pre-retire blob | `edit::wall_house` (the FULL 279-entry table, `:386`) exists IDENTICALLY at both peer tips — the delegation target compiles with zero extra edits. |
| `tr-core/src/edit/hammer.rs` | `5ef3e5d9c` | `5ef3e5d9c` | = g44's pre-retire blob | the i31→i33 gate note is identical; g44's retire commit refreshes its two stale "narrower subset" comments (comment-only, no marker change — the `pub fn wall_house` anchors row is untouched). |

### THE hunk (apply verbatim to BOTH d118 and d119 `crates/tr-core/src/housing.rs`)

Replace lines **18-27** at both tips (the 2-line doc + the 8-line narrow fn):

```rust
/// `Main.wallHouse` (`Main.cs:10403-10470` + `:9741-9860`): the walls a
/// house may be enclosed by.
pub fn is_house_wall(wall: u16) -> bool {
    matches!(
        wall,
        23..=27 | 29..=39 | 41..=47 | 60 | 66..=68 | 72..=74 | 84 | 85 | 88..=93 | 100..=128
            | 173 | 179 | 181 | 183 | 184 | 186 | 224..=227 | 231..=235 | 237..=241 | 245
            | 312 | 313 | 315..=317 | 323..=346
    )
}
```

with g44's landed block (g44 `housing.rs:18-33`, commit `8e791809a`):

```rust
/// `Main.wallHouse` (`Main.cs:9741-9860` + `:10353-10577`, the FULL
/// 279-entry table): the walls a house may be enclosed by. `CheckRoom`'s
/// ±2 scan reads `Main.wallHouse` directly (`WorldGen.cs:6204`/`:6212`)
/// — the SAME table the wall-edit gates read, so this delegates to
/// [`crate::edit::wall_house`] (the S1-P-i33 faithful copy, verified
/// member-for-member against the C# incl. the `153..167` loop). The old
/// narrower subset (missing 1/4-6/…/75-78/**82**/129-169/…) refused the
/// sky-island houses' disc wall 82 (`WallID.DiscWall`, `Main.cs:10384`
/// sets it true) — the W4-g44 island-house housing fix: a generated
/// IslandHouse interior is a valid NPC room exactly as vanilla. The
/// d118/d119 housing lanes keep their own copies of the subset at their
/// tips — the merge dedupes onto this full-table read (research/87
/// closeout §3).
pub fn is_house_wall(wall: u16) -> bool {
    crate::edit::wall_house(wall)
}
```

(The doc's last sentence describes the pre-dedupe state; the merger may
trim it to "— deduped per research/87 RETIRE §5" — the fn body is the
load-bearing part. The anchors canary marker is the body line
`crate::edit::wall_house(wall)`, g44's `anchors.txt:904` row — keep that
row through the merge.)

### Per-branch notes

- **d118 (`91db266ea`, scorer/QuickFindHome lane):** the hunk is the ONLY
  edit needed. Its `TooManyUnsafeWalls` witness (peer `housing.rs:817-828`)
  uses the unsafe dirt wall 3 — absent from the FULL table too (`wall_house`
  admits 1/4-6/10-12/16-27/29-39/…, never 3), so the witness stays green
  under the widening. No peer test pins the narrow subset (full grep for
  `is_house_wall(82)` / `!is_house_wall` at both tips: zero hits). The
  widening is a pure predicate change — zero RNG draws — so d118's stream
  witnesses are unaffected.
- **d119 (`04d7f9fd2`, relocation lane):** same single hunk, PLUS a free
  correctness gain: `town_spawn.rs:463` (`spawn_town_npc`'s sticky-flag arm)
  calls `tr_core::housing::is_house_wall(wall)` where the C# reads
  `Main.wallHouse[Main.tile[x, y].wall]` DIRECTLY (`WorldGen.cs:5411`,
  re-read at retire). Under the narrow copy a disc-wall-82 anchor tile
  failed to arm `canSpawn`; the delegation makes that call site C#-correct
  with NO d119-side edit. d119's own witnesses (the msg-60 apply legs, the
  60-witness framed-body decode) never read the wall table.
- **Optional additive (both branches):** g44's two housing witnesses —
  `w4_g44_the_disc_wall_is_a_house_wall_and_the_sky_room_scores`
  (g44 `housing.rs:1005-1035`, append inside `mod tests`) and
  `w4_g44_the_generated_island_house_ships_open_flanks_and_no_torch`
  (g44 `islands.rs:2387`, rides the islands KEEP anyway). Written-not-run;
  they pin wall 82/73 admission + the unsafe 2/3 refusal on the merged
  predicate.
- **Merge order:** the hunk is independent of FINAL-KEEP items 1-5 (the
  islands/buried-chest family) — it can land with either housing lane's
  merge; whichever lands SECOND sees the delegation already present and
  resolves the identical-block conflict by keeping ONE copy.

## RETIRE §6 — the harpy spawn band: activation receipt (2026-11-21)

The `c55cdc024` dispatch (terminal-default + the Wyvern short-circuits)
re-verified at the peers' LATEST tips. **Verdict: the kind-gate resolves
CLEANLY at merge — g8's Harpy row activates the band with ONE published
hunk below; the feather shot is NOT active at any tip (cited skip), the
exact port hunk is published for the g8 machine lane.**

### What is landed where (tip reads at retire)

| Piece | Where | Evidence |
| --- | --- | --- |
| The sky-band latch (0.35/0.45 height bands, `NPC.cs:980-987`) + the skyMob dispatch shape | **g44** `spawner.rs:798-837/:938-1011` | verbatim `FindSpawnTile`; the dispatch's drone/probe arms spawn, the Wyvern arm (`:989`) carries `hard_mode && !wyvern_exists && !no_worms` BEFORE the `Next(10)` (the C# `:1407` `&&` short-circuit), the town-slime arm (`:1000`) consumes its `Next(25)`, the Harpy default (`:1010`) is TERMINAL `None` — never falls to the ground ladders. |
| `SpawnContext.wyvern_exists` wiring | **g44** `world_sim/npcs.rs:6752-6755` | `AnyNPCs(87)` = any live `NpcKind::WyvernHead` — d126's head kind is the roster probe; the second site `:7640` pins `false` (the no-roster ctx). |
| The wyvern CHAIN machinery (assembly, head/follower steps, the pool driver) | **d126** `7ddf25a49` (`npc_ai_wyvern.rs:151/:244/:485/:596`) | landed + wired into the sim; **d126 does NOT touch `spawner.rs`** (empty diff vs base `a725acb04`) — no dispatch conflict with g44. The dispatch-hit spawn itself stays the recorded deferral (the arm consumes the roll, returns `None`); activating it means routing the hit into d126's `assemble_wyvern_chain` (`:151`, head ctor `:773` shape) — a d126-lane decision, NOT a g44 gap. |
| The Harpy KIND row | **g8** `82c93cecf` (batch 4) AND **d34** `4012a7f3a` | BOTH land `NpcKind::Harpy` — g8 with protocol id **386** (`npc/data.rs:629/:998`, `ai_style` 14 at `:2647`, `net_id` 48 at `:3374`, size 24×34 `:3226`), d34 with protocol id **275** (`data.rs:601`). **The merge must keep ONE Harpy row** (a duplicate enum variant will not compile — the conflict is loud, not silent). Either survivor activates the band identically: `net_id() == 48` + `ai_style() == 14` on both. |
| The style-14 machine | base + g8 routing | g8 routes `ai_style 14 => update_cave_bat` (`npc.rs:5286-5287`), the kind arm retired to `unreachable!` (`:7123`); `update_cave_bat` (`:12016`) stamps `wet`, builds `Style14Env`, drains `bat_ai`'s shot into `wants_hostile_shot`, integrates `integrate_flyer_collision`. d34's tip routes the same kind through `update_cave_bat` directly (`npc.rs:6439`) — same machine either way. |
| The Harpy's OWN machine branches (wet arm + feather shot) | **NOT landed anywhere** | `npc_ai_style14.rs:121` (`:23333-23345` wet arm) and `:135` (`:23416-23436` feather shot) are CITED SKIPS at g8's tip, at main, and at EVERY origin branch (the full-branch sweep for `feather_shot`/`harpy_wet`/an active `net_id == 48` arm: zero hits; the file's only commits are the S1-D-3 pair `d6bc30ed1`/`364a9f14b`). g8's batch-4 commit message pins them as "cited machine skips (the machine round)". **g8b-feather-shot is a FUTURE slice, not a landed branch — no origin branch carries that slug.** |

### The merge-resolution verdict (the activation question)

1. **Dispatch × kind: CLEAN.** g8/d34 touch `npc.rs`/`npc/data.rs`/render/
   bestiary; g44's dispatch lives in `spawner.rs` + the `SpawnContext`
   field — g8's diff vs base does NOT touch `spawner.rs` (empty diff,
   verified). No textual conflict on the dispatch; the semantic join is
   the single hunk below.
2. **Kind × kind: ONE loud conflict.** g8 (386) vs d34 (275) both add
   `Harpy` — keep ONE row + its protocol-id band (the merger's roster-band
   decision; g8's 385-387 band completes the ≤230 wiring, d34's 271-278
   band completes kinds 1-70 — whichever roster order the merge adopts,
   the OTHER lane's Harpy row + `to_u16`/`from_u16` entries dedupe onto it).
   The g44 witnesses reference NO protocol id (they assert `None` spawns
   today), so either survivor keeps them compiling.
3. **Wyvern gate × d126: CLEAN.** The `wyvern_exists` probe reads
   `NpcKind::WyvernHead` — d126's kind, landed. Both deferrals (the
   dispatch-hit spawn, the 686 town-slime kind) stay recorded, not gaps.

### THE activation hunk (apply at the merge, g44 `spawner.rs`)

Replace the terminal-default block (`spawner.rs:1003-1010` at the retire
tip — the 8-line comment + `return None;`):

```rust
        // The Harpy default (`:1419-1422`): `SpawnNPC(spawnTileX * 16 + 8,
        // spawnTileY * 16, 48)` — the skyMob arm's TERMINAL else. The
        // Harpy kind (48, aiStyle 14) lands with d34's npc-kinds-1-70
        // lane (`4012a7f3a`, unmerged); until the merge the dispatch is
        // terminal-None — a sky candidate NEVER falls through to the
        // ground ladders (the old port spawned surface slimes in the
        // sky on a dispatch miss — the C#'s else-if chain cannot).
        return None;
```

with:

```rust
        // The Harpy default (`:1419-1422`): `SpawnNPC(spawnTileX * 16 + 8,
        // spawnTileY * 16, 48)` — the skyMob arm's TERMINAL else (the
        // W4-g44 retire activation: the kind landed g8/d34, aiStyle 14
        // rides update_cave_bat → bat_ai). The C# SpawnNPC (:5238-5267)
        // hands NewNPC the tile-top Y — the flyer ctor centres X on the
        // tile (+8 − w/2) and seats the box top at the spawn row, the
        // spawn_hover convention; a refused (colliding) placement still
        // returns — the else-if chain never falls through.
        return Npc::spawn_hover(ctx.world, x, y / 16, NpcKind::Harpy);
```

Witness update (same merge): `the_sky_default_never_falls_through_to_the_
ground_ladders` (`spawner.rs:3586`) flips its 400-seed sweep from
`assert!(n.is_none())` to `assert!(n.is_none_or(|n| n.kind ==
NpcKind::Harpy))` — the terminal property (never a ground kind) is the
invariant; the ground control below it is unchanged. The wyvern-gates
witness (`:3527`) needs NO change: its `assert!(n.is_none(), "the tail
arms spawn nothing yet")` at `:3554` holds only while the town-slime
`Next(25)` misses — on a `Next(25)` HIT the activated default spawns a
Harpy, so that assert becomes `n.is_none_or(|n| n.kind ==
NpcKind::Harpy)` too (the STREAM equality asserts — the load-bearing
part — are untouched).

### The feather-shot residual (the g8 machine lane's next slice)

Exact C# re-read (`NPC.cs:23333-23345` + `:23416-23436`, retire): the wet
arm is the WATER_GROUP lift shape verbatim (`vy>0 → ×0.95`, `−0.5`, cap
`−4`, `TargetClosest()`); the shot arm is `ai[0]++` every server tick,
fires at **30/60/90** (NOT the sickle's 20/40/60/80) on `CanHit`, speed
**6f** (NOT 0.2), the SAME X-then-Y `Next(-100,101)` jitter order, type
**38** (`ProjectileID.Feather`) damage **15** kb 0, `timeLeft = 300`; the
reset arm is `flag24 && ai[0] >= 400 + Next(400)` (NOT 300+300). Port
shape: a second request struct (or a kind-tagged shot enum) drained by
`update_cave_bat` beside `DemonSickleShot` — the `wants_hostile_shot`
tuple already carries the type id, so the drain needs no new seam. RNG
delta: on Harpy shot ticks TWO draws (X then Y), on reset-eligible ticks
ONE `Next(400)` — the g8 machine round's stream witnesses must pin both.
Until then a merged Harpy flies + approaches + bounces + wets (the shared
arms) but does not shoot — the recorded skip, exactly as g8's tip.

## RETIRE §7 — sky-lake FISHING end-to-end: the chain compose-note (2026-11-21)

The `4c170c027` census-landed verdict (M7-FISH-2) re-verified at f35's
LATEST tip **`1abae9ade`** (the doc-sweep tip; the crate machinery landed
`00a5b27db`). **The full chain COMPOSES — every leg is landed, no seam is
missing, no lane re-claims another's leg.** The one witness gap (a
crate-caught-in-a-sky-lake end-to-end) is a COMPOSITION of two existing
witnesses, noted below, not a port gap.

### The chain, leg by leg (all reads at the retire tips)

| # | Leg | Owner / tip | Evidence |
| --- | --- | --- | --- |
| 1 | The lake EXISTS + survives settle | **g44** (this branch) | `islands.rs:192` the `WillWaterPlacedHereStayPut` gate (`WorldGen.cs:78530-78537`); closeout §2's witness `w4_g44_the_sky_lake_water_survives_settle` (`islands.rs:2323`) — 600 settled updates, mass unchanged. |
| 2 | The bowl is FISHABLE + rolls the space band | **g44** `4c170c027` | witness `the_sky_lake_bowl_builds_a_space_band_attempt` (`fishing.rs:2566`): the compact CloudLake clears the 75-tile pond gate (`get_fishing_pond_state`), `try_build_fishing_attempt` rolls `height_level 0` (the `Projectile.cs:20712` `y < worldSurface*0.5` arm, re-read at retire — the ladder :20712-20731 verbatim in `fishing.rs:398-432`). |
| 3 | The QUEST catch (the Cloudfish witness) | **g44** `4c170c027` | same witness, second half: `AttemptInputs.quest_item = 2453` + `angler_alive` → the probe carries `quest_fish 2453` into the context (`FishingCheck_ProbeForQuestFish`, `Projectile.cs:20734`) → the `QuestFish(2453) + Height0` row (`fishing.rs:1795`, `GameContentFishDropPopulator.cs:73-81`) answers over a 60-seed sweep. All seven sky rows (2461/2453/2473/2476/2458/2459/2304) pinned by the row-condition witness (`fishing.rs:2449-2479`). |
| 4 | The CRATE catch rows | **g44** `4c170c027` (M7-FISH-2, base-landed) | `fishing.rs:1675-1676`: `[3206]` Rare 1/1 `Crate && Height0 && EarlyMode`, `[3985]` Rare 1/1 `Crate && Height0 && HardMode` (`CONDS_CRATE_H0_*`, `:1589-1590`; the populator `:214` pair). The band+hardmode witness (`fishing.rs:2545-2558`): the azure answers in hardmode, the early crate is GONE in hardmode, BOTH refuse at `height_level 1`. |
| 5 | The bite → reel → catch delivery | base (S1-P-i35, M7-FISH-2 consumers) | `projectile_bobber.rs`: `bobber_step` builds the attempt through leg 2's `try_build_fishing_attempt` (`:379`), `accumulate_bite` (`:306`), the reel state (`:192+`); `kill_hook` (`:431`, `Projectile.cs:71986-71994`) rolls `catch_stack` and queues `(item, stack)`; `world_sim/projectiles.rs:569` drains into `slot_for_pickup` (inventory-first, overflow drops — `GiveItemToPlayer` :53907-53958). A rolled 3206/3985 rides this leg IDENTICALLY to a fish — `ai[1]` carries any item id. |
| 6 | The crate OPEN (the 26-crate dispatch) | **f35** `00a5b27db` @ tip `1abae9ade` | `fishing_crates.rs`: `is_fishing_crate` includes 3206 (`:81`) + 3985 (`:94`), `is_hardmode_crate` includes 3985 (`:109`); the clicks `OpenableBag` dispatch routes both into `open_fishing_crate` (`Player.cs:8028-9250`). The SKY arms: the weapon pick `[158, 65, 159, 2219]` Next(4) (`:421`, C# `:8653-8663` — Starfury/Enchanted Boomerang/Ball O'Hurt/Shiny Red Balloon), the tails (`:610`, C# `:9117-9157`): 1/2 pottery Next(6) over the SAME six pieces as the sky CHEST (5226/5254/5238/5258/5255/5388), 1/2 Cloud 751 ×50-100, 1/40 CreativeWings 4978, the `skyblock_low_tiles`-seam 1/3 SkyMill 2197. |

### The compose verdicts

- **f35 does NOT touch `tr-core/src/fishing.rs`** (empty diff vs
  origin/main at `1abae9ade`, verified) — the catch-side rows are g44's
  census-landed M7-FISH-2 base code; f35 owns ONLY the open side. Zero
  claim overlap; the merge is a clean union.
- **The sky-crate OPEN witnesses exist**: `w4_f35_sky_crate_low_tiles_
  seam_gates_the_sky_mill` (`fishing_crates.rs:1364` — the seam on/off
  pair) + the 3206/3985 rows inside `w4_f35_every_shared_crate_zero_
  script_grants_the_traced_rows` (`:1223`, the pair at `:1257/:1262`) and
  `w4_f35_every_shared_crate_replays_the_csharp_draw_order` (`:1813`, the
  sky pair with `skyblock_low_tiles: true` at `:1837-1838`).
- **The one recorded seam** (f35's own doc): `CrateEnv::skyblock_low_tiles`
  wires FALSE (the gen-time active-tile census `WorldGen.cs:3209-3217` is
  unported — the e25-style constants-off note). On a NORMAL sky-lake world
  that is the C#-correct value (lowTiles is a skyblock-seed flag); the
  SkyMill tail activates only when a skyblock lane ports the census. Not
  a g44 gap, not an f35 gap — a named future seam.
- **The composition witness gap (recorded, not blocking):** no single
  witness drives legs 2→4→5→6 in one run (catch a 3206 IN the bowl, then
  open it). Both halves are pinned separately (leg 4's band witness rolls
  the crate id; leg 6's witnesses open that id); the join is `kill_hook`'s
  item-id-agnostic queue — the same drain the fish witnesses ride. A
  future composed witness would build the §2 bowl, force `crate = true`
  on the context, sweep to a 3206, feed it through `kill_hook` →
  `drain_catches` → `open_fishing_crate`, and assert a sky grant list.
  WRITTEN NOWHERE yet — this note is the receipt that the chain needs no
  port work, only that optional witness.
- **The chest-side sibling** (for the merger's family map): the sky CHEST
  loot (`0898f3521`, the flag12 tails + Prefix(-1) + the CreativeWings
  rider) and the sky CRATE tails share the pottery/cloud/wings rows by
  C# design (`WorldGen.cs:37418-37527` vs `Player.cs:9117-9157`) — two
  landings of the same item family, NOT a dedupe target (different
  streams, different gates: the chest's Next(6) is unconditional inside
  flag12, the crate's rides its own 1/2).
