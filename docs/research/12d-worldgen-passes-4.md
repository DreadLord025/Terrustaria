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
| 30 Hellforge | 2ac7c298 | landed (code-only; hash re-pin deferred to the gate) |
| 31 Spreading Grass | 3a101031 | landed (code-only; classic arm, remix arm deferred to d107; hash re-pin deferred) |
| 32 Surface Ore and Stone | (this commit) | landed (code-only; 512-pin unchanged by the width guard; default re-pin deferred) |

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
  (`j < maxTilesY - 20`), `j--`, `PlaceTile(num3, j, 77)` (the `Place3x2`
  route `:60079-60083`), the type-77 success check, the 10000-failure
  valve. The wall-miss arm redraws WITHOUT bumping the failure count
  (only the place-failure + catch arms do). The `try/catch`'s
  out-of-bounds reads map to the port's bounds-checked `tile` reads.
* **Rust**: `structures.rs::pass_hellforge` (+ the `TILE_HELLFORGE`
  const + the a==b witness), `pipeline.rs` (`step_hellforge` after
  `quick cleanup`, the skyblock gate), `progress.rs` (`Lang.gen[36]`
  `:18191`).
* **RNG**: the retry draws ride the fresh `pass_rng("Hellforge")`
  stream — no other ledger shifts.
* **Hash doc**: both pins gain the pass-30 stale-by-design notes.
  Re-pin DEFERRED to the gate; **a == b untouched** + the pass-level
  witness `w4_d104_30_the_hellforge_places_and_replays`.

### Pass 31 — Spreading Grass (landed, classic arm)

* **C# read in full**: the pass body `WorldGen.cs:18237-18446` — the
  gate (`!denyAllGeneration && (!notTheBees || remix || anniversary)`),
  sweep 1 (`:18242-18313`: the ±50 columns down to `worldSurface`, the
  jungle-grass 60 → dirt spread, the stone/snow/ore ±3 flavour scan with
  the sand-53 latch + the open-air `flag` gate + the evil cover veto +
  the jungle-band 59/60 split), sweep 2 (`:18314-18351`: the ±10
  columns, the `flag2` open-sky latch, the first-surface
  `SpreadGrass(num3, num4, 0, 2, repeat: false)`, the
  `worldSurfaceHigh` break). The remix arm (`:18352-18443`) is d107's.
  `SpreadGrass` `:75146-75268` re-read for the `repeat` parameter.
* **Rust**: `surface.rs::pass_spreading_grass` (+ 2 witnesses),
  `evil.rs::spread_evil_grass` gains the `repeat` parameter (the evil
  calls pass `true` — behaviour unchanged; the recursion gate is
  `repeat && grassSpread < 1000`), `pipeline.rs`
  (`step_spreading_grass` after `hellforge`, the gate),
  `progress.rs` (`Lang.gen[37]` `:18241`).
* **RNG**: the classic arm draws NOTHING — the step's stream slot is
  the remix arm's future home. No ledger shifts.
* **Hash doc**: both pins gain the pass-31 stale-by-design notes (the
  surface cells move; no draws). Re-pin DEFERRED; **a == b untouched** +
  the pass-level witness `w4_d104_31_the_surface_dirt_takes_grass`.

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

## Status

- [x] 28 Floating Island Houses
- [x] 29 Quick Cleanup
- [x] 30 Hellforge
- [x] 31 Spreading Grass (classic arm; remix arm → d107)
- [x] 32 Surface Ore and Stone
- [ ] 33 Place Fallen Log
- [ ] 34 Traps
- [ ] 35 Piles
- [ ] 36 Spawn Point