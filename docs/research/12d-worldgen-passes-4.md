# 12d — WorldGen passes, lane 4 (task/W4-d104-worldgen-passes-4)

> Wave 4 · 2026-11-20 · code-only mode (no cargo/tests on this branch) ·
> source: `docs/research/12-worldgen.md` (the 52-pass gap list) re-derived
> against the current `pipeline.rs` (109 decls, 61 executed steps, 55 distinct
> vanilla names → **53 unexecuted**, of which 2 are special-seed-gated).

## Range claim (documented early, before any port)

The six `worldgen-passes-N` lanes (d101…d106) split the **classic (ungated)
unexecuted passes in C# `AddPasses` order**. The two gated passes —
`Skyblock` (`WorldGen.cs:11511`) and `Dual Dungeons Dither Snake` (`:14755`) —
belong to the special-seed lanes (d107 / c30), not to these six. That leaves
**51 classic passes**; lanes take ~9 each in anchor order.

Scripted derivation (VANILLA_CHAIN names minus EXECUTED_PIPELINE `vanilla`
fields, sorted by anchor): lane 1 = #1-9, lane 2 = #10-18, lane 3 = #19-27,
**lane 4 (this branch) = #28-36**, lane 5 = #37-45, lane 6 = #46-51.

### Lane 4 — the nine passes (C# order)

| # | Pass (`GenPassNameID`) | C# anchor | Rust home (planned) |
| --- | --- | --- | --- |
| 28 | Floating Island Houses | `WorldGen.cs:17859` | `islands.rs` |
| 29 | Quick Cleanup | `WorldGen.cs:17875` | `micro_finish.rs` / new arm |
| 30 | Hellforge | `WorldGen.cs:18187` | `structures.rs` |
| 31 | Spreading Grass | `WorldGen.cs:18237` | `surface.rs` |
| 32 | Surface Ore and Stone | `WorldGen.cs:18447` | `surface.rs` |
| 33 | Place Fallen Log | `WorldGen.cs:18513` | `surface.rs` |
| 34 | Traps | `WorldGen.cs:18657` | `structures.rs` |
| 35 | Piles | `WorldGen.cs:18788` | `structures.rs` |
| 36 | Spawn Point | `WorldGen.cs:19460` | `pipeline.rs` (replaces the compact spawn in `step_world_tail`) |

Neighbours for context (NOT this lane): #27 Cave Walls `:17711` (lane 3),
#37 Grass Wall `:19589` (lane 5). `Water Chests` `:17245` and `Oasis` `:16243`
sit in lane 3's numeric range but are also claimed by
`task/W4-c33-underwater-chests-oasis`; lane 3/merger reconciles.

## Discipline

- Verbatim ports (golden rule): same structure, maths, draw order; citation
  comment per block; readable Rust names.
- Each pass gets its `EXECUTED_PIPELINE` row at the vanilla slot + a
  positional witness; per-pass `pass_rng("<vanilla name>")` stream so the
  insertion shifts no other step's ledger.
- **Hash re-pins are DEFERRED to the merge gate** (code-only mode — no
  compiler on this branch, same discipline as S1-K-2 wave B / S1-P-i33):
  `world_hash.rs` gains a per-pass stale-by-design note; the `a == b`
  determinism halves of both pin tests stay untouched and must stay green
  at the gate.
- Commit + push per pass; docs + `anchors.txt` row per pass.

## Insertion point (derived against the current `pipeline.rs`)

All nine passes sit in vanilla between the `Remove Water From Sand` /
`Shell Piles` / `Smooth World` / `Living Trees` block (`:15476-16419`) and
`Planting Trees` (`:19940`). The executed chain runs that block as
`remove water` → `shell piles` → `slopes` → `living trees`, then `trees`
(Planting Trees). So the nine rows insert **between `living trees` and
`trees`**, in C# order. (`Pots` `:17996` sits inside the range but is
already executed as the earlier `pots` step — a pre-scaffold deviation,
not re-added.) `Spawn Point` (`:19460`) lifts the compact spawn out of
`step_world_tail` into its own row; the tail keeps only the moon/hell
draws that must stay last.

RNG neutrality: `floating islands` is the LAST `streams.drng` consumer,
so pass 28 moving the house draws off `drng` onto the fresh
`pass_rng("Floating Island Houses")` stream shifts no other step. Every
other new pass draws only its own `pass_rng("<vanilla name>")` stream.

## Landing ledger (ONE pass per commit+push)

| pass | commit | status |
| --- | --- | --- |
| range doc | 2d9cfcbb | claimed |
| 28 Floating Island Houses | 93b1938e | landed (code-only; hash re-pin deferred to the gate) |
| 29 Quick Cleanup | df067118 | landed (code-only; hash re-pin deferred to the gate) |
| 30 Hellforge | 2ac7c298 (+574f1d58 PlaceTile-77 graft) | landed (code-only; hash re-pin deferred to the gate) |
| 31 Spreading Grass | 3a101031 (+74103ae4 repeat:true fix) | landed (code-only; classic arm, remix arm deferred to d107; hash re-pin deferred) |
| 32 Surface Ore and Stone | 19bc132b | landed (code-only; 512-pin unchanged by the width guard; default re-pin deferred) |
| 33 Place Fallen Log | 2fdc8b96 | landed (code-only; 512-pin unchanged by the width guard; default re-pin deferred) |
| 34 Traps | 7c768356 (+341f4280 witness fix) | landed (code-only; 512-pin unchanged by the width guard; default re-pin deferred) |
| 35 Piles | dc92d139 | landed (code-only; both pins stale-by-design; re-pin deferred) |
| 36 Spawn Point | (this commit) | landed (code-only; pins unchanged — spawn fields unhashed, own stream) |

### Pass 28 — Floating Island Houses (landed)

* **C# read in full**: the pass body `WorldGen.cs:17859-17874` (the
  `Skyblock.denyFloatingIslands` gate `:3087-3096` — skyblock &&
  !extraFloatingIslands — rides the step adapter), `IslandHouse`
  `:79203-79484` (wall veto, drunk half-split, direction/width/height
  rolls, floor scan, sunplate shell + interior, door-side clearance,
  door, far column, chest main-item cycle + `AddBuriedChest`, fountain,
  windows, table/chairs + left-chair frameX bump, banners, style-1
  palms), `Place1x2` `:46652-46679`, `Place2xX` `:39663-39722` (the
  207→num3=4 arm), `Place3x2` `:52066-52196` (the shared evil.rs copy
  generalised: the campfire-215 liquid veto + the 14-style-25 row arm —
  behaviour-identical for the evil pass's type-26 calls), `SwitchFountain`
  `:51150-51193` (the `Wiring.running` half never fires during worldgen),
  `TryGrowingTreeByType` `:30343-30354` (323 → `GrowPalmTree(x, y, 0,
  false)` — the ocean.rs port), the `AddBuriedChest` sky-band slice
  `:35871-36210` (the 231/altar vetoes, the `SolidTile` floor walk, the
  `(i-1, k-1)` `PlaceChest` + the S1-A-4 stocking hook).
* **Rust**: `worldgen/island_houses.rs` (new — the verbatim port + the
  helpers + 2 witnesses), `islands.rs` (the inline plank `build_house`
  retires; the pass records the `GenVars.floatingIslandHouseX/Y/Style/
  skyLake` ledger `:13108-13111`, lakes included, the post-loop
  `num3 * 2` cap `:13117-13119` truncates the ledger), `pipeline.rs`
  (`GenVars.island_house_slots`, `step_floating_island_houses` between
  `living trees` and `trees`, the skyblock gate), `progress.rs`
  (`WorldGeneration.FloatingIslandHouses` message `:17863`),
  `evil.rs::place_3x2` promoted `pub(crate)` + generalised,
  `temple.rs::place_temple_door` promoted `pub(crate)`.
* **RNG neutrality**: the islands pass was the LAST `streams.drng`
  consumer; the house draws move onto the fresh
  `pass_rng("Floating Island Houses")` stream — no other step's ledger
  shifts. The compact islands pass also loses its inline `Next(4,7)` /
  `Next(3,5)` / main-item draws from `drng` (the accepted world change —
  `drng` has no later consumer).
* **Deviations** (module doc + ANALYSIS pointer): the compact island
  model records its own anchors (`cx`, disc top row) and style 0 for
  classic worlds — the fountain (`islandStyle > 0`) and palm
  (`islandStyle == 1`) arms never fire classically but are ported
  verbatim for the seed lanes; the chest rides the compact
  `AddBuriedChest` sky-band slice (the full loot-branch tree is the chest
  lane's); the `anyShimmer` veto defers with the shimmer lane.
* **Hash doc**: `world_hash.rs` gains the pass-28 stale-by-design notes
  on BOTH pins (`SEED42_512` unchanged by this pass — the 512 island band
  collapses; `DEFAULT_WORLD` moves by design — the sunplate houses
  replace the plank boxes). Re-pin values DEFERRED to the merge gate
  (no compiler on this branch). **The `a == b` determinism halves of
  both pin tests are untouched** — plus the pass-level a==b witness
  `w4_d104_28_the_house_builds_and_replays_byte_equal`.

### Pass 29 — Quick Cleanup (landed)

* **C# read in full**: the pass body `WorldGen.cs:17875-17993` — the
  `Skyblock.denyAllGeneration` gate (`:3131` — plain skyblock), the
  `NotTheBees()` call (the seed lane's), the `tileSolid[137]/[130]`
  false-window (`:17884-17885` → restore `:17991-17992`), the
  `oceanDepths` lava→water swap, the beach sand drip (the per-check
  `Next(4, 7)` condition draw — the C# re-rolls the bound on EVERY
  condition evaluation, the port re-rolls per iteration), the
  sandstone/hardened-sand wall retypes (59/123/224/368/367 → 397) + the
  dualDungeons-gated liquid arms, the surface-sand ±3 wall inheritance
  (the LAST walled column wins — the C# `break` exits only the inner
  loop), the `SaveSlopes`/`Platforms`/`CanBeClearedDuringGeneration`
  clear family (the 225-half-brick exemption, the 137-trap neighbour
  veto). `oceanDepths` `:10963-10973` + `oceanLevel` `:4447`
  (`(worldSurface + rockLayer)/2 + 40`), `Tile.ClearSlope`
  `Tile.cs:130-134`, `SaveSlopes` `TileID.cs:468-471` (the tile_runner
  `save_slopes` copy promoted `pub(crate)`).
* **Rust**: `micro_finish.rs::pass_quick_cleanup` (the sweep + the
  windowed `solid_tile_qc` predicate + 2 witnesses), `pipeline.rs`
  (`step_quick_cleanup` after `floating island houses`, the skyblock
  gate), `progress.rs` (`WorldGeneration.QuickCleanup` `:17879`).
* **RNG**: only the beach drip draws (`Next(4, 7)` per condition check)
  on the fresh `pass_rng("Quick Cleanup")` stream — no other ledger
  shifts.
* **Deviations**: the `NotTheBees()` whole-world conversion stays with
  the seed lane (biomes.rs's documented scope); the static `tileSolid`
  table carries the 137/130 window on the predicate (the temple
  `spike_solid` precedent).
* **Hash doc**: both pins gain the pass-29 stale-by-design notes (the
  512 world's beach band covers the whole sweep width, so its pin moves
  too). Re-pin DEFERRED to the gate; **a == b untouched** + the
  pass-level witness `w4_d104_29_the_cleanup_sweep_replays_and_tidies`.

### Pass 30 — Hellforge (landed)

* **C# read in full**: the pass body `WorldGen.cs:18187-18234` —
  `num = maxTilesX / 200` forges, each a `while (!flag)` retry: the
  `Next(1, maxTilesX)` column + `Next(maxTilesY - 250, maxTilesY - 30)`
  row, the wall 13/14 gate, the drop scan to the first active cell
  (`j < maxTilesY - 20`), `j--`, `PlaceTile(num3, j, 77)` (the FULL
  PlaceTile route — see the graft below), the type-77 success check, the
  10000-failure valve. The wall-miss arm redraws WITHOUT bumping the
  failure count (only the place-failure + catch arms do). The
  `try/catch`'s out-of-bounds reads map to the port's bounds-checked
  `tile` reads.
* **Graft (574f1d58, research/86 §6.2)**: g37 research/75 §2.7's
  recommendation accepted — the bare `evil::place_3x2` write became
  `structures.rs::place_tile_hellforge`, the PlaceTile-77 slice
  (`:59342-60295` re-read): the 488 veto `:59345`, the entry gate
  `:59373` (`tileSolid[77]` is FALSE — the generated `TILE_SOLID` bit 77
  is 0 — so the entry always admits), the inactive-seat
  `Clear(Tile|TilePaint|Slope)` `:59473-59476` (incl. the Glow/Echo
  coatings, the g38 fallen-log idiom), the
  `ResetsHalfBrickPlacementAttempt[77]` reset `:59477-59482` (the set is
  default-true, `TileID.cs:497`), the 77 dispatch → `Place3x2` +
  `SquareTileFrame` `:60079-60083` (`edit::square_tile_frame_gen`, the
  temple precedent), the tail re-frame + result `:60255-60262`. The
  sub-vanilla `height <= 250` guard adopted from g37. **The ORDER stays
  d104's** — hellforge AFTER `hell forts` (g37's slot precedes the
  wall-14 producer and never fires, §1 B7). The retry draw schedule is
  unchanged (the route draws no RNG).
* **Rust**: `structures.rs::pass_hellforge` + `place_tile_hellforge`
  (+ the `TILE_HELLFORGE` const + the a==b witness + 3 graft witnesses),
  `pipeline.rs` (`step_hellforge` after `quick cleanup`, the skyblock
  gate), `progress.rs` (`Lang.gen[36]` `:18191`).
* **RNG**: the retry draws ride the fresh `pass_rng("Hellforge")`
  stream — no other ledger shifts.
* **Hash doc**: both pins gain the pass-30 stale-by-design notes (+ the
  graft notes — the tile bytes may move on residue seats / refused-
  placement resets; same deferred re-pin). Re-pin DEFERRED to the gate;
  **a == b untouched** + the pass-level witness
  `w4_d104_30_the_hellforge_places_and_replays`.

### Pass 31 — Spreading Grass (landed, classic arm)

* **C# read in full**: the pass body `WorldGen.cs:18237-18446` — the
  gate (`!denyAllGeneration && (!notTheBees || remix || anniversary)`),
  sweep 1 (`:18242-18313`: the ±50 columns down to `worldSurface`, the
  jungle-grass 60 → dirt spread, the stone/snow/ore ±3 flavour scan with
  the sand-53 latch + the open-air `flag` gate + the evil cover veto +
  the jungle-band 59/60 split), sweep 2 (`:18314-18351`: the ±10
  columns, the `flag2` open-sky latch, the first-surface LIVE
  `SpreadGrass(num3, num4)` call (`:18334` — the defaults `dirt: 0,
  grass: 2, repeat: true`, `:75146`), the `worldSurfaceHigh` break. The
  remix arm (`:18352-18443`) is d107's. `SpreadGrass` `:75146-75268`
  re-read for the `repeat` parameter.
* **Fix (74103ae4, research/86 §6.1)**: the original port passed
  `repeat: false` — that is the DEAD `:18339` catch arm (SpreadGrass has
  its own try/catch `:75148/:75254` and never throws). The call-site
  flipped to `repeat: true`; the bounded 3×3 recursion
  (`grassSpread < 1000`, a DEPTH cap — the outer column loop re-seeds
  past it) now fires. The old single-cell witness was stale-by-design
  WRONG and re-derived (the recursion grasses the open-flanked column
  down to the `j >= worldSurface && dirt == 0` refusal); + the shelf
  recursion witness + the 1100-cell depth-cap/re-seed witness.
* **Rust**: `surface.rs::pass_spreading_grass` (+ 4 witnesses),
  `evil.rs::spread_evil_grass` gains the `repeat` parameter (the evil
  calls pass `true` — behaviour unchanged; the recursion gate is
  `repeat && grassSpread < 1000`), `pipeline.rs`
  (`step_spreading_grass` after `hellforge`, the gate),
  `progress.rs` (`Lang.gen[37]` `:18241`).
* **RNG**: the classic arm draws NOTHING — the step's stream slot is
  the remix arm's future home. No ledger shifts; no pinned grass-spread
  draws exist to re-derive.
* **Hash doc**: both pins gain the pass-31 stale-by-design notes (+ the
  fix notes — more cells grass, still zero draws; same deferred
  re-pin). Re-pin DEFERRED; **a == b untouched** + the pass-level
  witness `w4_d104_31_the_surface_dirt_takes_grass`.

### Pass 32 — Surface Ore and Stone (landed)

* **C# read in full**: the pass body `WorldGen.cs:18447-18511` (the ore
  scatter `Next(width*5/4200, width*10/4200)` with the `width/420` retry
  budget, the beach band `Next(380, width-380)`, the [0.48, 0.52] centre
  re-roll, the `worldSurfaceLow..worldSurface` row, the ±200 spacing +
  the `numOrePatch < maxOrePatch - 1` record; the stone scatter
  `Next(1, width*7/4200)`, the [0.47, 0.53] band, the ±100 spacing),
  `OrePatch` `:10579-10705` (the copper/iron `Next(3)` tier roll, the drop
  scan, the grass-triplet/wall vetoes, the ±10×+7..+30 body scan, the
  `Next(2)` sink, the seed write + `OreHelper`, the `Next(8, 13)`
  re-roll-per-check vein walk with the side pockets, the drifting blob's
  per-cell radii + the 1-in-8 double + the velocity clamps), `StonePatch`
  `:10242-10324` (the same gates + the 199/23 veto, the blob retype, the
  pile decor: the 186 `Place3x2` `Next(7, 13)` style + the two 185
  `PlaceSmallPile` calls), `OreHelper` `:10228-10240`, `PlaceSmallPile`
  `:46815-46873` (the `type` parameter generalised on the
  spider_gem_moss copy; the 185 boulder seat veto inert during worldgen,
  D20), `GenVars.cs:240-244`/`:461` (maxOrePatch 50), `Reset` `:8323`.
* **Rust**: `surface.rs::pass_surface_ore_and_stone` + `ore_patch` +
  `stone_patch` + `ore_helper` (+ the a==b witness),
  `spider_gem_moss.rs::place_small_pile` gains the `type` parameter
  (the spider calls pass 185 — unchanged) + promoted `pub(crate)`,
  `pipeline.rs` (`GenVars.ore_patch_x`, `step_surface_ore_and_stone`
  after `spreading grass`, the skyblock gate).
* **RNG**: all draws on the fresh `pass_rng("Surface Ore and Stone")`
  stream; the `NextDouble` draws map to `next_f64` (the ocean.rs
  precedent). No other ledger shifts.
* **Deviations**: the sub-vanilla width guard skips the pass before any
  draw on ≤600-wide worlds (the C# `Next(1, width*7/4200)` throws
  there); the `SquareTileFrame` follow-ups defer to the framing pass
  (the port frames at the chain end).
* **Hash doc**: `SEED42_512` UNCHANGED by this pass (the width guard);
  `DEFAULT_WORLD` stale-by-design. Re-pin DEFERRED; **a == b untouched**
  + the pass-level witness `w4_d104_32_the_patches_place_and_replay`.

### Pass 33 — Place Fallen Log (landed)

* **C# read in full**: the pass body `WorldGen.cs:18513-18655` — the
  `denySomeGeneration` gate (`:3133-3147`), the `extraLiquid` bubble arm
  (seed lane), the `noSurface` gate, the `width/2100 + Next(-1, 2)` count
  (the remix `Next(0, 2)` arm deferred), the 30000/5000 budgets (the
  skyblock /10), the ±400 column + `Next(10, worldSurface)` row, the
  centre re-roll above half-budget, the drop scan, the
  `worldSurface - 10` veto, the ±50 cloud/sand/dungeon/Crimson/Corrupt
  scan, the ±10 overhead clear scan, the liquid gate (relaxed below
  budget/5), the grass triplet (the notTheBees 60 / frozen 147 / desert
  53 alternatives), `PlaceTile(num5, j-1, 488)` (the `Place3x2` route
  `:59887-59890`), the 1-in-2 `GenVars.logX/logY` record.
* **Rust**: `surface.rs::pass_place_fallen_log` +
  `deny_some_generation` (+ the a==b witness), `pipeline.rs`
  (`GenVars.log_anchor`, `step_place_fallen_log` after `surface ore and
  stone`), `progress.rs` (`Lang.gen[85]` `:18524`).
* **RNG**: all draws on the fresh `pass_rng("Place Fallen Log")` stream.
* **Deviations**: the ≤800 width guard (the C# column draw throws
  there); the remix count/row arms + the extraLiquid arm defer to the
  seed lanes.
* **Hash doc**: `SEED42_512` UNCHANGED (the width guard);
  `DEFAULT_WORLD` stale-by-design. Re-pin DEFERRED; **a == b untouched**
  + the witness `w4_d104_33_the_fallen_log_places_and_replays`.

### Pass 34 — Traps (landed)

* **C# read in full**: the pass body `WorldGen.cs:18657-18786` (the
  `denySomeGeneration` + `actuallyNoTrapsForRealIMeanIt` +
  `notTheBees && !noTraps && !remix` gate, the `tileSolid[379]` window,
  the `width*0.05` count with the getGood ×1.5 / skyblock /5, the
  1150-attempt retry onto wall-less underground cells outside the ocean
  band, the `width*0.003` sand-trap scatter onto wall-187 columns),
  `placeTrap` `:8922-9513` (the head gates + the classic type roll +
  case 0 dart `:9057-9201`, case 1 boulder `:9202-9399` with the
  boulder-type roll chain + the pet cap, case 2 explosive `:9400-9459`,
  case 3 geyser `:9460-9500`), `PlaceSandTrap` `:35463-35735`,
  `placeTrap_CanContinue` `:8909-8920`, `IsTileNearby` `:8862-8884`,
  `AreAnyTilesInSetNearby` `:8787-8804`, the three boulder-proximity
  helpers `:8806-8860`, `CanGeneratePressurePlateAt` `:11029-11049`,
  `Place1x1` `:45188-45233`, `Place2x2` `:51330-51420`, `GetWorldSize`
  `:6559-6570`, `Tile.ClearTile` `Tile.cs:123-128`.
* **Rust**: `worldgen/traps.rs` (new — the full port + 2 witnesses),
  `pipeline.rs` (`GenVars.shimmer_position` + `boulder_pets_placed`,
  `step_traps` after `place fallen log` with the three-part gate),
  `progress.rs` (`Lang.gen[34]` `:18663`).
* **RNG**: all draws on the fresh `pass_rng("Traps")` stream.
* **Deviations**: the noTraps seed arms (count multipliers, TNT barrels,
  boulder-solidity flips, anniversary type-1) + the drunk progress loop
  + the `starGame` multiplier defer to the seed lanes / are unported;
  the shimmer veto reads `GenVars.shimmer_position` (Zero until d102
  publishes); the ≤400 width guard; the explosive case's `break` →
  shared `return false` quirk kept 1:1.
* **Hash doc**: `SEED42_512` UNCHANGED (the width guard);
  `DEFAULT_WORLD` stale-by-design. Re-pin DEFERRED; **a == b untouched**
  + the witnesses `w4_d104_34_the_geyser_places_and_replays` /
  `w4_d104_34_the_dart_case_wires_the_plate`.

### Pass 35 — Piles (landed)

* **C# read in full**: the pass body `WorldGen.cs:18788-19458` — the
  `denySomeGeneration` gate, the `tileSolid` window (379/229/190/196/
  189/717/718/719/202/460/484 false → restore), the seven scatters:
  large-pile underground `width*0.06` (`:18809-18927`, the biome style
  chain + the 1-in-75 187-17 override + the dungeon 1-in-3 skip + the
  shimmer veto + the cluster), large-pile deep `width*0.01`
  (`:18928-18993`), large-pile surface `width*0.03` (`:18994-19054`, the
  grass-side 187 arm + the -1 veto), large-pile walled-surface
  `width*0.0035` (`:19055-19125`), small-pile underground `width*0.6`
  (`:19126-19287`, the wall-87 re-roll + the style-offset chains),
  small-pile surface `width*0.02` (`:19288-19366`), small-pile
  dirt/snow-wall surface `width*0.15` (`:19367-19440`, the
  full-liquid-sand veto). `GetPileGenerationAttempts` `:23647-23655`
  (`width/2`, skyblock /10), `PlaceSmallPile` `:46815-46873` under the
  window, `PlaceTile` 186/187 → `Place3x2` (`:59993-60002`).
* **Rust**: `structures.rs::pass_piles` + the window helpers
  (`pile_floor`, `place_small_pile_window`, `pile_cluster`,
  `pile_attempts`) + the a==b witness; `traps.rs::ocean_depths_trap`
  promoted `pub(crate)` (shared); `pipeline.rs` (`step_piles` after
  `traps`), `progress.rs` (`Lang.gen[89]` `:18792`).
* **RNG**: all draws on the fresh `pass_rng("Piles")` stream.
* **Deviations**: the noTraps boulder-solidity flip is the seed lane's;
  the ≤50 width guard; the `anyShimmer` vetoes read the liquid-shimmer
  flag (the shimmer lane publishes no tiles before this pass on a
  classic world).
* **Hash doc**: BOTH pins stale-by-design (the 512 column band is live).
  Re-pin DEFERRED; **a == b untouched** + the witness
  `w4_d104_35_the_piles_place_and_replay`.

### Pass 36 — Spawn Point (landed)

* **C# read in full**: the pass body `WorldGen.cs:19460-19587` — the
  skyblock arm (the `SolidTile3` drop + the ±20 acceptance,
  `:19462-19478`), the classic search (the centre ±5 jitter walk, the
  first active non-379 cell, the widening retry on the
  below-surface/liquid/geyser-nearby gates, the ±10 fallback,
  `:19527-19573`), the remix bottom-centre arm (`:19574-19584`). The
  noSurface valid-area search + the SpawnOnBeach arm are the seed
  lanes'. `SolidTile3` `:70465-70483`, `IsTileNearby` (the traps copy
  promoted `pub(crate)`).
* **Rust**: `surface.rs::pass_spawn_point` + `solid_tile3` (+ the a==b
  witness), `pipeline.rs` (`step_spawn_point` after `piles`; the compact
  spawn + the remix arm LIFTED OUT of `step_world_tail` — the tail keeps
  only the moon/hell draws), `progress.rs` (no message — the pass sets
  only `progress.Set(1.0)`).
* **RNG**: the search draws on the fresh `pass_rng("Spawn Point")`
  stream; the tail's `streams.rng` moon/hell draws stay last (the
  `worldgen_rolls_hell_back_style` order pin holds).
* **Hash doc**: BOTH pins UNCHANGED by this pass (the spawn fields are
  not hashed; the search draws on its own stream). **a == b untouched**
  + the witness `w4_d104_36_the_spawn_seats_on_the_surface`.

## Lane result

All nine rows executed: `EXECUTED_PIPELINE` grows by 9 steps (the
islands pass's inline house build retired into pass 28, the compact
spawn lifted into pass 36). The unexecuted classic census drops by 9
(the sibling lanes own the remainder). The hash pins are stale-by-design
where the passes move tiles (28, 29, 30, 31, 32-default, 33-default,
34-default, 35); passes 32/33/34 keep the 512 pin via their sub-vanilla
width guards and pass 36 moves neither pin. The merge gate re-measures
sequentially after the sibling lanes (the S1-K-2-wave-B convention).
The `a == b` determinism halves of both pin tests are untouched, and
every pass carries its own positional + a==b witness.

## Status

- [x] 28 Floating Island Houses
- [x] 29 Quick Cleanup
- [x] 30 Hellforge
- [x] 31 Spreading Grass (classic arm; remix arm → d107)
- [x] 32 Surface Ore and Stone
- [x] 33 Place Fallen Log
- [x] 34 Traps
- [x] 35 Piles
- [x] 36 Spawn Point