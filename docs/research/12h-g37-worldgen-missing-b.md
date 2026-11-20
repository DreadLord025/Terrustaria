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

## Landing ledger

| pass | commit | status |
| --- | --- | --- |
| range claim (this doc) | 8ef3e111 | claimed |
| census re-probe + claim confirmation (this doc) | 30f605c7 | confirmed — all nine UNLANDED on every pushed branch |
| B1 Statues | a0bf6174 | ported — witnesses written not run |
| B2 Surface Chests | 1cf9f0f1 | ported — witnesses written not run |
| B3 Water Chests | (this commit) | ported — witnesses written not run |
| B4 Cave Walls | — | pending |
| B5 Floating Island Houses | — | pending |
| B6 Quick Cleanup | — | pending |
| B7 Hellforge | — | pending |
| B8 Spreading Grass | — | pending |
| B9 Surface Ore and Stone | — | pending |
