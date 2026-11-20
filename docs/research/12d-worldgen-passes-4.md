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
| 28 Floating Island Houses | (this commit) | landed (code-only; hash re-pin deferred to the gate) |

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

## Status

- [x] 28 Floating Island Houses
- [ ] 29 Quick Cleanup
- [ ] 30 Hellforge
- [ ] 31 Spreading Grass
- [ ] 32 Surface Ore and Stone
- [ ] 33 Place Fallen Log
- [ ] 34 Traps
- [ ] 35 Piles
- [ ] 36 Spawn Point