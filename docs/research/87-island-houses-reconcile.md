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
