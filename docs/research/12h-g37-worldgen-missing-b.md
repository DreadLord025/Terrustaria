# 12h — W4-g37: worldgen missing-b (mop-up lane, second quarter)

> Wave 4 · lane `task/W4-g37-worldgen-missing-b` @ base a725acb0 · claimed 2026-11-20
> Parent audit: `docs/research/12-worldgen.md` (the unexecuted-pass census).

## Census re-probe (2026-11-20, continuation of 8ef3e111)

Scripted diff of `pipeline.rs` at this branch's base (a725acb0): **109
`VANILLA_CHAIN` decls (106 classic, `gate == ""`), 66 executed steps citing
55 distinct vanilla names → 51 unexecuted classic passes** — the same list
d101's `12b` census derived.

In-flight probe — every pushed worldgen branch (`git ls-remote` + the
pushed `docs/research/12*` docs and `pipeline.rs` diffs):

- **d101** @ 4194acf8 (`12b-d101-passes-1.md`): **9/9 LANDED** — census
  #1-9 (Sand Patches…Generate Ice Biome + the Granite row-split).
- **d102** @ 28ec5561 (`12a`): **5 LANDED** — Silt c746a7da, Webs 96e4b92f,
  Slush d4fd0af0, Mountain Caves 665041f8, Gems 28ec5561 (census #10-14);
  Shimmer, Clean Up Dirt, Dirt Rock Wall Runner, Wood Tree Walls pending
  there (g36's a-slice reassigns them per this lane's split).
- **d103** @ 561b3ab2 (lane-split section in `12-worldgen.md`): **2 LANDED**
  — Oasis 90b7eee6, Waterfalls 561b3ab2 (census #20-21); Wall Variety…Cave
  Walls + Floating Island Houses claimed, pending.
- **d104** @ 2d9cfcbb (`12d`): claim only — Floating Island Houses…Spawn
  Point (census #28-36), zero passes landed.
- **d105** @ 5bb4116e (`12e`): claim only — Guide…Larva (its own #37-45
  numbering), zero landed.
- **d106** @ ee0fcbef (`12f`): **1 LANDED** — Grass Wall (census #37);
  Settle Liquids Again, Tile Cleanup, Stalac, Remove Broken Traps, Final
  Cleanup pending (g39's d-slice).
- **d107** @ 26672912 / **d108** @ 7ac633e9: special-seed / temple-paint
  lanes — no classic-pass rows.
- **f24**: no pushed branch. **f25** @ fd098528 (`12g-f25-settle-liquids.md`):
  claim only — adopted the Settle Liquids orphan (census #19), pending
  (g36's A5 claims it too; the merger reconciles that pair).
- **g36** @ 0b01c170 (`12g-g36-worldgen-missing-a.md`): claim only — the
  a-slice minus d102's landed Mountain Caves/Gems = Shimmer, Clean Up Dirt,
  Dirt Rock Wall Runner, Wood Tree Walls, Settle Liquids, Wall Variety,
  Life Crystals (7 passes), all pending.
- **g38** @ d32b13cc (`12i-g38-worldgen-missing-c.md`): claim only — the
  c-slice minus d106's landed Grass Wall = Place Fallen Log, Traps, Piles,
  Spawn Point, Guide, Sunflowers, Herbs, Dye Plants (8 passes), pending.
- **g39**: no pushed branch (the d-slice: Flowers…Final Cleanup, 10 passes).
- **c33** (Water Chests/Oasis, referenced at claim time): no pushed branch.

**Landed on siblings: 9 + 5 + 2 + 1 = 17 → 34 unexecuted passes remain,
none landed anywhere.** The mop-up union g36 (7) + g37 (9, this lane) +
g38 (8) + g39 (10) = 34 covers the remainder exactly — no orphans left.
Every one of this lane's B1-B9 is UNLANDED on every pushed branch; the
claim stands unchanged. The d103/d104 pending overlaps are exactly what
this mop-up split reassigns (both siblings' docs defer to the merger).

## The gap this lane takes (claim-time derivation, 8ef3e111)

The d10x census lists 50 classic unexecuted passes. Landed on the sibling
branches at claim time: d101 #1-8 (Sand Patches..Generate Ice Biome) + the
Granite row-split,
d102 Silt/Webs/Slush (#9-11), d103 Oasis/Waterfalls (#19-20). That leaves
**37 unexecuted passes** (d10x numbering 12-18 and 21-50). The four
`worldgen-missing-{a,b,c,d}` mop-up lanes (g36/g37/g38/g39) had pushed no claim
docs at this lane's claim time, so this lane fixes the deterministic split:
**the 37-item remainder in C# anchor order, quartered — a = #1-9, b = #10-18
(this lane), c = #19-27, d = #28-37**. If a sibling's pushed claim differs,
the merger reconciles; nothing here touches a pass a d10x lane has LANDED.

### This lane's nine passes (remainder #10-18, C# order)

| # | Pass (`GenPassNameID`) | C# anchor | Rust home (planned) |
| --- | --- | --- | --- |
| B1 | Statues | `WorldGen.cs:16853` | `structures.rs` |
| B2 | Surface Chests | `WorldGen.cs:17130` | `structures.rs` / `chest_stock.rs` seam |
| B3 | Water Chests | `WorldGen.cs:17245` | `ocean.rs` (c33 claimed but pushed nothing — merger reconciles) |
| B4 | Cave Walls | `WorldGen.cs:17711` | new arm (`caves.rs` / `passes_b.rs`) |
| B5 | Floating Island Houses | `WorldGen.cs:17859` | `islands.rs` (d103/d104 overlap — merger reconciles) |
| B6 | Quick Cleanup | `WorldGen.cs:17875` | `micro_finish.rs` / new arm |
| B7 | Hellforge | `WorldGen.cs:18187` | `structures.rs` |
| B8 | Spreading Grass | `WorldGen.cs:18237` | `surface.rs` |
| B9 | Surface Ore and Stone | `WorldGen.cs:18447` | `surface.rs` |

Neighbours NOT this lane: Life Crystals :16755 (a's #9), Place Fallen Log
:18513 (c's #19). The a-slice is Mountain Caves, Gems, Shimmer, Clean Up Dirt,
Dirt Rock Wall Runner, Wood Tree Walls, Settle Liquids, Wall Variety, Life
Crystals; c = Place Fallen Log, Traps, Piles, Spawn Point, Grass Wall, Guide,
Sunflowers, Herbs, Dye Plants; d = Flowers, Mushrooms, Random Gems, Moss
Grass, Larva, Settle Liquids Again, Tile Cleanup, Stalac, Remove Broken
Traps, Final Cleanup.

## Discipline

- Verbatim ports (golden rule): same structure, maths, draw order; citation
  comment per block; readable Rust names; C# read in ≤100-line windows.
- Each pass gets its `EXECUTED_PIPELINE` row at the vanilla slot + a
  positional witness; per-pass `pass_rng("<vanilla name>")` stream so the
  insertion shifts no other step's ledger (the genRand current convention).
- **Hash re-pins are DEFERRED to the merge gate** (code-only mode — no
  compiler on this branch, same discipline as S1-K-2 wave B / the d10x lanes):
  `world_hash.rs` gains a per-pass stale-by-design note; the `a == b`
  determinism halves of both pin tests stay untouched and must stay green
  at the gate.
- ONE pass per commit+push; docs + `anchors.txt` row per pass.

## B1 Statues — landing notes (this commit)

- **Pass body** `WorldGen.cs:16853-16970` → `passes_b.rs::pass_statues`.
  Classic arm only: the `Skyblock.denySomeGeneration` gate is false; the
  noTraps ×15 (`:16861-16868`), starGame (`:16869-16872`) and skyblock
  (`:16874-16878`) count arms are special seeds (deferred, cited inline).
  `num3 = (73*2) * (maxTilesX/4200)`; the y band
  `((int)(worldSurface*2 + rockLayer))/3 .. maxTilesY-300`; the remix extra
  draw (`:16894-16897`) is deferred. The 100-budget `oceanDepths` re-roll
  (`:16898-16912`, the dual-dungeons bounds disjunct deferred), the
  fall-to-first-active (`:16919-16926`), the `num7--` step up, the
  frame-important-and-not-tileCut + `anyShimmer` veto (`:16934-16943`), the
  forced `PlaceTile` (`:16946`), the success/trap/`num++` arm and the
  10000-attempt give-up (`:16948-16964`) — all verbatim.
- **Statue table** `SetupStatueList` (`:4449-4488`): 44 entries 105 styles
  0-43 with `[34]=(349,0)` / `[43]=(105,50)`, then the 29 appended styles —
  73 entries. `StatuesWithTraps = {4,7,10,18}` (`GenVars.cs:474`).
- **Helpers** (B-lane-local slices, cited): `PlaceTile` (`:59342-60295`,
  the B-kind preamble + dispatch + tail), `Place2xX` (`:39663-39722`, the
  105-style-34→349 remap + the `style/55` sheet wrap + the 207 4-tall
  height), `Place1x1` (`:45188-45235`, the plate-135 arm),
  `PlaceStatueTrap` (`:4490-4513`, the `SolidTile2(Tile)` overload
  `:70134-70151` — null→true, no solidTop exclusion), `WireLine`
  (`WorldBuilding/WorldUtils.cs:109-129` — the pre-swap point captures, the
  horizontal-then-vertical red-wire runs, `PlaceWire` `:58111-58120`).
- **Boot statics**: `frame_important[105/349/135]` (`Main.cs:7707/7727/7721`).
- **Pipeline**: `step!("statues", "Statues", …)` before `cave houses`
  (vanilla `:16853` < `:16971`); count test 66→67; positional witness
  `w4_g37_the_statues_step_precedes_the_cave_houses`; progress `Gen(29)`
  (`:16857`).
- **Hash notes**: `SEED42_512` UNCHANGED by B1 alone — the 512-wide world
  sits entirely inside the ±380 `oceanDepths` beach margin, so every statue
  candidate is rejected (proof in the `world_hash.rs` note); `DEFAULT_WORLD`
  moves BY DESIGN (the 4200-wide world places the cycle). Re-pins deferred
  to the gate; the `a == b` halves untouched.
- **Deviations**: the pass-local `Main.tileSolid` toggles are unmodelled
  (static `td::TILE_SOLID`, the `spider_gem_moss.rs` precedent); the
  `PlaceTile` arms that never fire for the B kinds are documented out of
  slice in the module header; the g39 helper-lane duplicates
  (`Place1x1`/`Place1x2`/`Place3x2`) are merger-reconciled.
- **Verification**: code-only mode — no cargo on this branch. Witnesses
  written (the pipeline count/order tests + the positional witness), not
  run; the gate runs them.

## B2 Surface Chests — landing notes (this commit)

- **Pass body** `WorldGen.cs:17130-17210` → `passes_b.rs::pass_surface_chests`.
  The `noSurface` gate (`:17132`) reads the micro flag; the skyblock budget
  (`:17136-17139`) is deferred. Count `(int)(maxTilesX*0.005)`; the 2000
  give-up budget; the y band `(int)worldSurfaceLow .. (int)worldSurface`; the
  **unbounded** `oceanDepths` re-roll (`:17155-17159`, the remix arm deferred);
  the wall-2/59/244 surface gate (`:17163-17173`); the ±50 `j+=2`/`k+=2`
  wall-244 scan with the `Next(num8)==0` acceptance + `num8++` tightening
  (`:17176-17193`); `AddBuriedChest(num3, num4, 0, notNear:true, flag2?12:-1,
  trySlope:false, 0)` (`:17195`).
- **Shared placement core** (new, B2/B3/B5): `place_chest_at` = `PlaceChest`
  (`:57813-57846`, the boulder-floor veto + the near-chest exclusion + the
  `TileObject.CanPlace`/`Place` route — the temple.rs:1981 precedent);
  `add_buried_chest` = `AddBuriedChest`'s placement half (`:35877-37568`):
  the floor scan, the shimmer/larva-231 aborts, the style-17 ±30 chest box +
  the trySlope top-slope flatten/restore, the ±2 boulder/altar box (fluff
  100), the `SolidTile` floor gate, the `num9` style resolution (`flag15`
  default 1, the caller override), the desert/ice/hell family arms (kept 1:1;
  a surface chest never reaches them in a classic world), the style-12
  living-tree wall gate, then `PlaceChest(i-1, k-1)`. The stocking block
  (`:36218-37551`) rides the `chest_stock` hook (S1-A-4) with the layer-anchor
  save/restore (the mini_biomes.rs:1144 precedent).
- **Pipeline**: `step!("surface chests", …)` after `cave houses`, before
  `chests`; count 67→68; witness
  `w4_g37_the_surface_chests_step_follows_the_cave_houses`; progress `Gen(31)`
  (`:17134`).
- **Hash notes**: BOTH pins move BY DESIGN (the surface band sits above
  oceanLevel, so the 512-world's 2 chests place); re-pins deferred; `a == b`
  untouched.
- **Verification**: code-only — witnesses written, not run.

## B3 Water Chests — landing notes (this commit)

- **Pass body** `WorldGen.cs:17245-17355` (`GenPassNameID.UnderwaterChests`)
  → `passes_b.rs::pass_water_chests`. The treasure loop (`:17250-17270`):
  per `GenVars.numOceanCaveTreasure` anchor the `NextFromList{863,186,277,
  187,4404}` main (extraLiquid 268 deferred), the `num = 2.0; num += 0.1`
  widening search with the `(int)num` / `(int)num/2` bands, the `:17264` edge
  fold, the `InWorld + liquid > 250 + water` gate, `AddBuriedChest(…, 17,
  trySlope:true, 0)`. The scatter (`:17271-17353`): `9 * (maxTilesX/4200)`
  iterations, the 1-in-10 863 / the `num4` 186→4404→277→187 cycle, the two
  liquid-search loops (`Next(50, maxX-50)` × the depth / surface bands,
  `liquid >= 250` water), `trySlope = num8 < beachDistance || num8 >
  maxX - beachDistance`, the 10000 outer budget.
- **GenVars seam**: reads `gv.num_ocean_cave_treasure` / `gv.ocean_cave_treasure`
  (the `ocean caves` step's product, `ocean.rs::record_cave_treasure`) — no
  new GenVars slots.
- **Deviation**: the C# inner liquid-search loops are unbounded (they hang on
  a waterless world); the port caps them at 10000 re-rolls (a degenerate-world
  guard, cited inline). The notTheBees/remix/extraLiquid/tenth arms are
  deferred.
- **Pipeline**: `step!("water chests", …)` after `surface chests` (the port's
  chest-family grouping; vanilla `:17245` sits after Jungle Chests Placement
  `:17211`, before Spider Caves `:17356` — the isolated stream makes the
  grouping RNG-neutral); count 68→69; witness
  `w4_g37_the_water_chests_step_follows_the_surface_chests`; progress `Gen(33)`
  (`:17249`).
- **Hash notes**: both pins move BY DESIGN; deferred; `a == b` untouched.
- **Verification**: code-only — witnesses written, not run.

## B4 Cave Walls — landing notes (this commit)

- **Pass body** `WorldGen.cs:17711-17839`
  (`GenPassNameID.CaveWallsInEnclosedSpaces`) → `passes_b.rs::pass_cave_walls`.
  The `tileSolid[162/226/232]` toggles (`:17713-17715`) are unmodelled (static
  `td::TILE_SOLID`). The 4 %-width enclosed-space loop (`:17720-17786`): the
  `((worldSurface+rockLayer)/2) .. maxTilesY-220` band, the 500-budget
  `countTiles(jungle:false, lavaOk:true)` search (`10 <= tiles < 1500`), the
  wall pick — `shroomCount > rockCount*0.75` → 80, `iceCount > 0` → 40/71,
  `lavaCount > 0` → 79, else the 4-way 59/61/170/171 — then `Spread.Wall`. The
  remix second loop (`:17787-17809`) is deferred. The 2 %-width jungle loop
  (`:17811-17837`): the `worldSurface .. lavaLine` band, the wall-64-gated
  `countTiles(jungle:true)`, the `wallHouse`/244 stale-count gate
  (`:17828-17831`), the 1000 budget, `Spread.Wall2(…, 15)`.
- **Helpers**: `count_tiles` = `countTiles`+`nextCount` (`:9546-9635`, the
  explicit-stack walk, both jungle modes, the 70/1/147-161/53-396-397
  counters); `spread_wall` = `Spread.Wall` (`:3356-3414`); `spread_wall2` =
  `Spread.Wall2` (`:3416-3524`, the 5000 `maxWallOut2` budget, the
  `CannotBeReplacedByWallSpread` {4,40,3,83,87,244,34} + `WallSpreadStopsAtAir`
  {63,62} sets, `WallID.cs:98-99`).
- **BASELINE_WALLS adaptation**: the port pre-lays dirt/stone/sandstone walls
  under the solid rows (`terrain.rs:344-350`); the C# leaves them 0 until this
  pass. The `wall != 0` tests in `count_tiles`/`spread_wall` read
  `BASELINE_WALLS` {1,2,187,216} as the C#'s 0 (the `spider_gem_moss.rs`
  precedent, cited inline).
- **Pipeline**: `step!("cave walls", …)` after `temple part 2`, before
  `statues`; count 69→70; witness
  `w4_g37_the_cave_walls_step_follows_the_temple`; progress `Gen(63)`
  (`:17718`).
- **Hash notes**: both pins move BY DESIGN; deferred; `a == b` untouched.
- **Verification**: code-only — witnesses written, not run.

## B5 Floating Island Houses — landing notes (this commit)

- **The d103/d104 overlap resolved**: the port previously built the island
  houses INLINE in `pass_floating_islands` (a shape-level plank box). Vanilla
  runs `FloatingIslandHouses` (`:17859-17874`) as a SEPARATE later pass over
  the sites `FloatingIslands` records (`GenVars.floatingIslandHouseX/Y/Style`
  + `skyLake`, `:13108-13111`). This commit ports the faithful two-pass split:
  `pass_floating_islands` now records `(x, y, style, skyLake)` sites into the
  new `GenVars.island_house_sites` (the house cap still bounds them), and
  `passes_b.rs::pass_floating_island_houses` builds the non-lake sites. The
  shape-level `build_house` is removed; the islands tests now count recorded
  sites. (d103/d104's pending claims defer to the merger — this lane's B5 is
  the faithful port.)
- **`IslandHouse`** (`WorldGen.cs:79203-79484`) → `passes_b.rs::island_house`:
  the ±30 dungeon/87/3/83-wall veto, the `Next(2)` direction + `Next(7,12)` /
  `Next(5,7)` rolls, the surface-anchor scan (the 192/191/384 exclusions), the
  sunplate-202 shell (the top-corner skip) + the skyware-82 interior (wall-0
  only), the door opening + the style-9 door, the far wall column, the
  `skyIslandHouseCount` cycle chest (style 13, `AddBuriedChest(i, num13-3)`),
  the fountain (islandStyle > 0), the counter advance, the window-21 walls,
  the 14-style-7 table + the 15-style-10 chairs + the left-chair frame shift,
  the 91 banners (`Next(7,10)`), the palm trees (islandStyle == 1). The
  classic islandStyle is 0 → the fountain/palm arms are no-ops; the
  remix/drunk/getGood door/chest variants are deferred.
- **`SwitchFountain`** (`:51150-51185`): the 2×4 frameY ±72 toggle; the
  `Wiring.running` SkipWire tail is a runtime concern (no-op in gen).
- **Pipeline**: `step!("floating island houses", …)` after `floating islands`;
  count 70→71; witness `w4_g37_the_island_houses_step_follows_the_islands`;
  progress `Key("WorldGeneration.FloatingIslandHouses")` (`:17863`).
- **Hash notes**: both pins move BY DESIGN (the house build moved passes +
  became faithful); deferred; `a == b` untouched.
- **Verification**: code-only — witnesses written, not run.

## B6 Quick Cleanup — landing notes (this commit)

- **Pass body** `WorldGen.cs:17875-17995` → `passes_b.rs::pass_quick_cleanup`.
  The 20-margin sweep: the ocean non-water→water retype (`:17893-17896`); the
  ocean surface-sand column fill (`:17897-17910`, the `bottomSlope` clear, the
  `Next(4,7)` budget, the 397/53/495 stop conditions on k/k+1/k+2); the
  desert-wall 187/216 retype (59/123/224/368/367→397) + the `rockLayer` liquid
  clear / lava-255 set (`:17911-17933`, the dualDungeons gate); the
  surface-sand ±3 wall bleed (`:17934-17957`); the slope cleanup
  (`:17958-17989`, the `SaveSlopes`/`CanBeClearedDuringGeneration`/platform
  arms + the 137 dart-trap deactivation + the 225-halfBrick exemption).
- **Deviations**: the notTheBees arm (`:17881-17884`) is a special seed
  (deferred); the `tileSolid[137/130]` toggles (`:17885-17886`, `:17992-17993`)
  are unmodelled (static `td::TILE_SOLID`); the `roundLandmasses` gate reads
  the micro flag.
- **Pipeline**: `step!("quick cleanup", …)` after `floating island houses`,
  before `pots`; count 71→72; witness
  `w4_g37_the_quick_cleanup_step_precedes_the_pots`; progress
  `Key("WorldGeneration.QuickCleanup")` (`:17879`).
- **Hash notes**: both pins move BY DESIGN; deferred; `a == b` untouched.
- **Verification**: code-only — witnesses written, not run.

## B7 Hellforge — landing notes (this commit)

- **Pass body** `WorldGen.cs:18187-18236` → `passes_b.rs::pass_hellforge`.
  The `maxTilesX/200` count; the per-forge search: `Next(1, maxX)` ×
  `Next(maxY-250, maxY-30)`, the hell-fort wall 13/14 gate (`:18204`), the
  fall-to-first-active with the `maxY-20` bound + the `j--` step-up
  (`:18206-18209`), `PlaceTile(num3, j, 77)` (default mute/forced,
  `:18210`), the success / 10000 give-up arm (`:18211-18222`).
- **Deviation**: the C# `catch` arm (`:18225-18232`) increments `num2` only on
  an OOB read; the port's dense array never throws, so a non-wall site retries
  without incrementing (cited inline). The port's hell forts paint wall 14
  (`structures.rs::WALL_HELL_BRICK`), so the gate fires.
- **Pipeline**: `step!("hellforge", …)` after `pots`, before `beehives`;
  count 72→73; witness `w4_g37_the_hellforge_step_follows_the_pots`; progress
  `Gen(36)` (`:18191`).
- **Hash notes**: both pins move BY DESIGN; deferred; `a == b` untouched.
- **Verification**: code-only — witnesses written, not run.

## B8 Spreading Grass — landing notes (this commit)

- **Pass body** `WorldGen.cs:18237-18446`
  (`GenPassNameID.SpreadingGrassOnSurfaceSunflowersEvilsOnSurfaceAndLavaCleanup`)
  → `passes_b.rs::pass_spreading_grass`. The 50-margin surface sweep
  (`:18242-18320`): the jungle-grass-60 3×3 mud/dirt spread (`:18252-18268`,
  the above-active → 59 else 60), the stone/clay/ore 7×7 surface-tile pick
  (`:18270-18317`, the 53-sticky + the 59/60/147/161/199/23 candidates, the
  open-air-above flag, the infection 23/199 + jungle 59/60 overrides). The
  per-column grass spread (`:18321-18353`): the `flag2` open-air gate, the
  `worldSurfaceHigh` break, `SpreadGrass(num3, num4)` (dirt 0, grass 2).
- **`spread_surface_grass`** = the generation-time `SpreadGrass(0, 2)` slice
  (`:75146-75268`): the `InWorld(10)` + dirt-0 guard, the surface rule (grass 2
  refuses at/below worldSurface), the 3×3 enclosure scan + lava exception, the
  `CanBeClearedDuringGeneration` gate, the plain type-2 write, the bounded 3×3
  recursion (`grassSpread < 1000`). The `SquareTileFrame` re-frame is the
  port's documented deviation (the evil.rs slice precedent).
- **Deviations**: the remix arms (`:18354-18443`, the surface grass / sunflower
  / Convert / QuickWater sweeps) are special seeds — deferred; the notTheBees
  gate (`:18239`) reads the classic flags. **No RNG** — the pass is tile-only.
- **Pipeline**: `step!("spreading grass", …)` after `hellforge`, before
  `beehives`; count 73→74; witness
  `w4_g37_the_spreading_grass_step_follows_the_hellforge`; progress `Gen(37)`
  (`:18241`).
- **Hash notes**: both pins move BY DESIGN (the surface retypes); deferred;
  `a == b` untouched.
- **Verification**: code-only — witnesses written, not run.

## B9 Surface Ore and Stone — landing notes (this commit)

- **Pass body** `WorldGen.cs:18447-18512` →
  `passes_b.rs::pass_surface_ore_and_stone`. The ore-patch count
  `Next(maxX*5/4200, maxX*10/4200)` (`:18452`), the `maxX/420` attempt budget,
  the `Next(beachDistance, maxX-beachDistance)` x roll + the 0.48-0.52 centre
  re-roll, the `worldSurfaceLow .. worldSurface` y band, the ±200 `orePatchX`
  spacing ledger (`:18466-18479`, the `maxOrePatch-1` cap), `OrePatch`. The
  stone-patch count `Next(1, maxX*7/4200)` (`:18484`), the ±100 spacing + the
  0.47-0.53 centre exclusion, `StonePatch`.
- **`OrePatch`** (`:10579-10705`): the 1-in-3 iron / else copper pick, the
  floor scan + grass/wall/underground gate, the `Next(2)` depth bump + seed
  cell + `OreHelper`, the `Next(8,13)`-budget walk (the ±1 x / 1-2 y steps, the
  1-in-3 extra-y, the 1-in-4 side cell), the drifting blob (the `Next(5,9)`
  radius / `Next(9,14)` steps, the inner-clear / outer-ore rings, the
  `NextDouble` drift + clamp).
- **`StonePatch`** (`:10242-10330`): the floor scan (with the 199/23
  exclusion), the `Next(13,18)` radius / `Next(13,19)` steps blob, the 1-in-7
  pile/furniture arm (the 186 pile + the 185 small piles), the inner stone
  retype.
- **Helpers**: `ore_helper` (`:10228-10240`), `surface_patch_floor` (the shared
  floor scan + gate), `place_small_pile` (`:46815-46873`, the anyLava veto +
  the size-1 two-cell / size-0 single-cell arms + the boulder veto).
- **GenVars**: `num_ore_patch` / `ore_patch_x[50]` added (`GenVars.cs:240-244`,
  `:461-462`).
- **Pipeline**: `step!("surface ore and stone", …)` after `spreading grass`,
  before `beehives`; count 74→75; witness
  `w4_g37_the_surface_ore_step_follows_the_spreading_grass`; progress `Gen(37)`
  (the pass sets `progress.Set(1.0)` with no message — the slot keeps the prior
  row).
- **Hash notes**: both pins move BY DESIGN; deferred; `a == b` untouched.
- **Verification**: code-only — witnesses written, not run.

## Lane complete

All nine B-passes (B1-B9) are ported, one commit each, witnesses written (not
run — code-only mode). The `a == b` determinism halves of both `world_hash.rs`
pin tests stay untouched; the pinned VALUES are stale-by-design and re-pin at
the merge gate. The g39 helper-lane duplicates (`Place1x1`/`Place1x2`/
`Place3x2`/`PlaceSmallPile`) and the d103/d104 island-house overlap are
merger-reconciled. No other lane's landed pass was touched.

## Landing ledger

| pass | commit | status |
| --- | --- | --- |
| range claim (this doc) | 8ef3e111 | claimed |
| census re-probe + claim confirmation (this doc) | 30f605c7 | confirmed — all nine UNLANDED on every pushed branch |
| B1 Statues | a0bf6174 | ported — witnesses written not run |
| B2 Surface Chests | 1cf9f0f1 | ported — witnesses written not run |
| B3 Water Chests | 4c58c56c | ported — witnesses written not run |
| B4 Cave Walls | ef7c9f76 | ported — witnesses written not run |
| B5 Floating Island Houses | 3d74564e | ported — witnesses written not run |
| B6 Quick Cleanup | b48a702b | ported — witnesses written not run |
| B7 Hellforge | 454e43ba | ported — witnesses written not run |
| B8 Spreading Grass | df468d00 | ported — witnesses written not run |
| B9 Surface Ore and Stone | (this commit) | ported — witnesses written not run |
