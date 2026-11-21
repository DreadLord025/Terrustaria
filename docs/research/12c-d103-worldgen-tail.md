# 12c — W4-d103 tail: the WorldGen POST-pass census (classic chain exhausted)

> Wave 4 · lane `task/W4-d103-worldgen-passes-3` (continuation after 9/9,
> tip 8be04cb0) · claimed 2026-11-21.
> Parent census: `12b-d101-passes-1.md` (the 42 unexecuted at d101's base)
> and the `12-worldgen.md` lane-split section. All pushed worldgen branches
> re-read after `git fetch --prune` (d101-d108, f24/f25, g36-g39, d1, g42).

## Part 1 — the classic unexecuted-pass list: REMAINING = ZERO (all 51 claimed)

The 51 classic (`gate == ""`) unexecuted `VANILLA_CHAIN` passes at base
a725acb0 (the d101/d102/g37/g39 scripted diffs agree) are covered by the
union of the pushed claims below. No pass is unclaimed; no lane may port a
new classic pass without colliding. Landed-vs-pending per claimant:

| # (C# order) | pass | anchor | claimant(s) | landed on a pushed branch |
| --- | --- | --- | --- | --- |
| 1-9 | Sand Patches … Generate Ice Biome, Granite | 11689-12839 | d101 | **9/9** (4194acf8 + Granite split) |
| 10-14 | Silt, Webs, Slush, Mountain Caves, Gems | 13143-15050 | d102 | **5/5** (28ec5561) |
| 15-18 | Shimmer, Clean Up Dirt, Dirt Rock Wall Runner, Wood Tree Walls | 15195-15717 | d102 + g36 A1-A4 | **4/4** (g36 aa3ee000; d102 29e8859b duplicates — merger reconciles) |
| 19 | Settle Liquids | 16132 | f25 (orphan adopt) | **landed** (0ed1f3cf) |
| 20-27 | Oasis … Cave Walls, Floating Island Houses | 16243-17859 | d103 (this lane) + g37 B1-B5 | **9/9** (d103 eb018138; g37 overlaps — merger reconciles) |
| 28-31 | Quick Cleanup, Hellforge, Spreading Grass, Surface Ore and Stone | 17875-18447 | d104 + g37 B6-B9 | **4/4** (d104 19bc132b; g37 94c365ee duplicates) |
| 32 | Place Fallen Log | 18513 | d104 + g38 C1 | **landed** (d104 2fdc8b96 / g38 8c9b7b60) |
| 33 | Traps | 18657 | d104 + g38 C2 (claim) | **landed** (d104 7c768356 + fix 341f4280) |
| 34-35 | Piles, Spawn Point | 18788, 19460 | d104 + g38 C3/C4 | pending (both claim) |
| 36 | Grass Wall | 19589 | d106 R1 (orphan adopt) | **landed** (ee0fcbef) |
| 37-40 | Guide, Sunflowers, Herbs, Dye Plants | 19685-19981 | d105 + g38 C5-C8 | pending (both claim) |
| 41-45 | Flowers, Mushrooms, Random Gems, Moss Grass, Larva | 20447-20868 | d105 | pending |
| 46 | Settle Liquids Again | 21246 | d106 R2 | **landed** (08aa8289) |
| 47-50 | Tile Cleanup, Stalac, Remove Broken Traps, Final Cleanup | 21520-22141 | d106 R3-R6 | pending |

Gated registrations (not classic): Skyblock `:11511` + the skyblock Jungle
re-registration + Dual Dungeons Dither Snake `:14755` — f24 landed all
three (9cbbf9de / 993864e1). d107 owns the seed-branch arms of executed
passes; d108 the temple seed branches. g39 landed the H1-H15 placement
helpers (bab25558 close). **The dispatch's exhaustion rule fires: this lane
takes the WorldGen POST-passes — the hardmode/growth functions outside
`AddPasses`.**

## Part 2 — the POST-pass census (WorldGen.cs hardmode family vs landed Rust)

| # | C# function | anchor | Rust today | gap |
| --- | --- | --- | --- | --- |
| P0 | `GenerateHellevator` | — | — | **does not exist in this decompilation (1.4.5.8)** — no `Hellevator`/`Elevator` symbol anywhere in `WorldGen.cs`; nothing to port under that name |
| P1 | `StartHardmode` | 31701-31719 | `tr-app world_sim/growth.rs::start_hardmode` | landed (netMode gate, chat, progression 9); `force` param unused (no caller forces) |
| P2 | `initializeHardMode` head | 31570-31661 | `spread.rs::initialize_hard_mode` is the runner half ONLY (fixed ±width/7 starts, `:31659-31660` slice) | **the anchor draws + the dungeon-side bias are missing** (Next(300,400)/Next(200,300)/Next(2), num3/num4/num5, num6) — THIS LANE |
| P3 | `initializeHardMode` remix arm | 31616-31656 | none | **missing** (the width/7 + width/14 hallow band, the per-column Next(-1,2) start) — THIS LANE |
| P4 | `GERunner` | 76169-76441 | `spread.rs::gerunner` (approximation: no shimmer pre-scan, no 20-margin box, wrong wall sets, fixed velocity, no exit band) | **verbatim rewrite** — THIS LANE |
| P5 | `initializeHardMode` moss-wall tail | 31663-31698 | none | **missing** (25·width/4200 placements, RandomWorldPoint, the Crimson/Corrupt/Hallow wall-family roll, ShapeFloodFill(1000) + OnlyWalls pocket + OuterOutline wall ring) — THIS LANE |
| P6 | `SmashAltar` | 48927-49109 | `ores.rs::smash_altar` (classic tiers/bands/blobs landed) | drunk tier-flip arms `:48939-48948`/`:48976-48985`/`:49008-49017`, the remix band `:49056-49067`, the anniversary strength `:49069-49071` — THIS LANE (seed arms); the wraith spawn `:49078-49105` parks (no NPC.SpawnOnPlayer port) |
| P7 | `OreRunner` | 48831-48925 | `ores.rs::ore_runner` (shape walk landed) | the 59-above-sky stop `:48843-48846`, the velocity drift + clamp `:48915-48923`, the runtime paint-reset + SquareTileFrame `:48887-48895`, the wallType/stayInArea/onlyReplace parameters — THIS LANE |
| P8 | `hardUpdateWorld` | 69600-69955 | `spread.rs::hard_update_world_gated` + `growth.rs::grow_crystal_shards` | **LANDED (P8 slice)** — the hardModeWorldUpdates gate `:69602`, the 60-grass chlorophyte seeding arm `:69652-69660`, the SINGLE type-9 purge pick `:69689-69700` (the C# `while(flag)` never re-arms), ChlorophyteDefense `:69456-69462` + Convert types 9/10 `:55671-55763`, the sunflower retry semantics (`continue`, not break), the walls:false Converts |
| P9 | Moss conversion micro-passes | 72371-72430 (overground), 73699-73755 (underground) | `growth/grass.rs::grow_moss_arm` / `grow_moss_arm_underground` | **LANDED (P9 slice)** — the underground arm (no NextDouble gate, the flag9 net gate parks, the shared `moss_arm_body` + the bare-write PlaceTile(184) spray approximation) |
| P10 | `MossConversion` | 71457-71493 | `growth/grass.rs::moss_conversion` | landed verbatim (DW-2.9 witnesses) |
| P11 | `CanChlorophyteGrow` | 69464-69515 | `spread.rs::can_chlorophyte_grow` | **LANDED (P8 slice)** — the remix arm (+5/×15/−4/−8) + the above-rock halving of BOTH caps and ×1.5 of BOTH boxes |
| P12 | `DoStartInHardmode` | 1899-1903 | `micro_surface.rs::start_in_hardmode` | landed (seed-gated); rides the P2-P5 rewrite automatically |
| P13 | `hardModeWorldUpdates` set | 71501 | `growth.rs::hard_mode_world_updates` | **LANDED (P8 slice)** — `hardMode || (remix && getGood && !anniversary)`, read by both dispatchers + the app growth tick |
| P14 | `TransformWorldOnBackgroundThread` | 31721-31742 | inline (documented deviation in `start_hardmode`) | stays (single-threaded port; no behavioural gap) |

Not this lane (runtime/event helpers, own lanes): `TryProtectingSpawnedItems`/
`UndoSpawnedItemProtection`, `Netplay.ResetSections`, the altar wraith spawn
(NPC spawn lane), `ClearUnbreakableWallsWithPaintUpTo` (d108 temple paint).

## This lane's slices (one per commit)

1. **(this doc)** the tail census + claim. → `fe9a565f`
2. **P2+P3+P4** — `initializeHardMode` head verbatim + `GERunner` verbatim
   rewrite (the two are one draw chain; splitting them would leave the head
   drawing anchors nobody consumes). → `04637c12`
3. **P5** — the moss-wall tail (the flood-fill/outline framework inline,
   the `wall_variety.rs` precedent). → `23f55212`
4. **P6+P7** — SmashAltar seed arms + OreRunner's missing halves.
   → `339d9f6d`
5. **P8+P11+P13** — hardUpdateWorld's gates + the chlorophyte arms +
   Convert types 9/10 + ChlorophyteDefense. → landed (this slice):
   - `hard_update_world_gated` (`:69600-69955`) verbatim: the
     `hardModeWorldUpdates` gate (`growth.rs::hard_mode_world_updates`,
     `:71501`), the actuated skip, the shard head gates moved to the call
     site (`grow_crystal_shards` = the placement body only), the 60-grass
     1/300 seeding, the 211/346 growth + the **single** type-9 purge pick
     (the C# `while (flag)` at `:69689` clears `flag` and never re-arms —
     one `Next(-6,7)` pair per source update, NOT a loop-until-air), the
     post-Plantera `Next(2)` halving, the three infection loops with the
     `continue`-retry / defence-break semantics and `walls: false`
     Converts (the old wall-converts-with-the-tile test assertion was the
     deviation — fixed).
   - `nearby_chlorophyte` (`:69517-69558`): the 211+346 ±5 count, the ≥3
     no-draw short-circuit, the border margin, the remix `Next(-1,3)` roll.
   - `ChlorophyteDefense` (`:69456-69462`): the remix-only type-10 purge;
     `convert_purge_cell` = Convert types 9/10 (`:55671-55763`) with the
     `KillTile` default-args thorn kill.
   - `can_chlorophyte_grow` (`:69464-69515`) fixed: the remix
     +5/×15/−4/−8 arm and the above-rock halving of BOTH caps + ×1.5 of
     BOTH boxes.
   - Both `UpdateWorld` dispatchers call the gated form on ACTIVE picks
     only (the C# `nactive()` else-if; the old unconditional call on
     inactive cells drew nothing anyway but sat at the wrong site). The
     overground dispatcher's arm placement keeps the port's existing
     approximation shape (documented at the call site).
   - 6 witnesses written NOT run; pins unchanged (runtime-only), a==b
     stays.
6. **P9** — the underground moss arm. → landed (this slice): the
   `:73699-73755` branch is the overground body minus the `NextDouble`
   gate — `grow_moss_arm_underground` calls the extracted shared
   `moss_arm_body` (the overground `grow_moss_arm` keeps its gate and
   delegates to the same body, behaviour frozen). The dispatcher routes
   the moss/moss-brick kinds into the arm and falls through to the wall
   arms (the C# does not return). The `flag9` net gate parks; the
   `PlaceTile(184)` spray keeps the overground slice's bare-write
   approximation (the `:59703-59721` support gate / `frameY = Next(3)*18`
   / re-frame documented at the body). 2 witnesses written NOT run; pins
   unchanged (runtime-only), a==b stays.

**The POST-pass family is exhausted by this lane's slices** (P0 does not
exist; P1/P10/P12/P14 landed or stay by design; P2-P9 + P11 + P13 landed
above).

## Discipline

- Verbatim ports (golden rule): same structure, maths, draw order; citation
  comment per block; C# read in ≤100-line windows.
- The hardmode transform draws on the caller's stream (the C# `genRand` ==
  `Main.rand` under the SwapRandom, `:4445`); no new stream is invented.
  `World.dungeon_x` becomes owned state (the `.wld` header field already
  exists — the port wrote/skipped it as "unowned"; this lane fills it).
- **Hash pins**: every slice is outside the classic pipeline or flag-gated
  (drunk/remix/anniversary/hardmode), so `SEED42_512` and `DEFAULT_WORLD`
  stay UNCHANGED by these slices (the `start_in_hardmode` micro-seed world
  changes by design — it has no pin). `a == b` determinism asserts stay
  untouched; witnesses written NOT run (code-only lane, no cargo/tests).
- ONE slice per commit+push; `anchors.txt` row per slice.