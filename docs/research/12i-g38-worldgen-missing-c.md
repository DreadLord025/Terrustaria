# 12i — W4-g38: worldgen missing-c (mop-up lane, third quarter)

> Wave 4 · lane `task/W4-g38-worldgen-missing-c` @ base a725acb0 · claimed 2026-11-20
> Parent audit: `docs/research/12-worldgen.md` (the unexecuted-pass census).
> Census re-derived on this branch's base (`pipeline.rs`): 109 chain decls,
> 61 executed steps, **51 unexecuted classic passes** — the same 51 the d10x
> lanes split. The mop-up quartering is g37's pushed `12h` doc.

## In-flight probe (all pushed worldgen branches read at claim time)

- **d101** (`12b`, tip 4194acf8): all 9 rows landed (#1-8 + Granite row-split).
- **d102** (`12a`, tip 28ec5561): Silt c746a7da, Webs 96e4b92f, Slush d4fd0af0,
  Mountain Caves 665041f8, Gems 28ec5561 **landed**; Shimmer, Clean Up Dirt,
  Dirt Rock Wall Runner, Wood Tree Walls pending there.
- **d103** (lane-split section in `12-worldgen.md`, tip 561b3ab2): Oasis
  90b7eee6, Waterfalls 561b3ab2 landed; #21-27 pending.
- **d104** (`12d`, 2d9cfcbb): claim only (#28-36, Floating Island Houses..Spawn Point).
- **d105** (`12e`, 5bb4116e): claim only (#37-45, Guide..Larva).
- **d106** (`12f`, tip ee0fcbef): **Grass Wall landed** (R1); R2-R6 pending.
- **d107 / d108**: special-seed / temple-paint lanes (not classic-pass mop-up).
- **f24**: at base, nothing pushed.
- **f25** (fd098528): adopted the **Settle Liquids** orphan (`:16132`).
- **g36** (`12g`, 0b01c170): missing-a = Shimmer, Clean Up Dirt, Dirt Rock Wall
  Runner, Wood Tree Walls, Settle Liquids, Wall Variety, Life Crystals.
- **g37** (`12h`, 8ef3e111): missing-b = Statues..Surface Ore and Stone; fixes
  the deterministic quartered split **a=#1-9, b=#10-18, c=#19-27, d=#28-37**
  over g36/g37/g38/g39, "nothing here touches a pass a d10x lane has LANDED".
- **g39**: at base, nothing pushed (missing-d = #28-37).

## The gap this lane takes

g37's published **c-slice (#19-27 of the 37-remainder)**: Place Fallen Log,
Traps, Piles, Spawn Point, Grass Wall, Guide, Sunflowers, Herbs, Dye Plants.
Since that claim, d106 has **landed Grass Wall** (ee0fcbef) — per g37's own
rule this lane does not touch it. The remaining **8 passes** are all still
unlanded on every pushed branch (d104/d105 hold them only as claim-only range
docs, which is exactly the gap g37's mop-up split reassigns). This lane takes
those 8, disjoint from g36 (a), g37 (b), g39 (d) and f25's Settle Liquids.

### This lane's eight passes (C# anchor order)

| # | Pass (`VANILLA_CHAIN` name) | `GenPassNameID` | C# anchor | Rust home (planned) |
| --- | --- | --- | --- | --- |
| C1 | Place Fallen Log | PlaceFallenLog | `WorldGen.cs:18513` | `surface.rs` |
| C2 | Traps | Traps | `WorldGen.cs:18657` | `structures.rs` |
| C3 | Piles | Piles | `WorldGen.cs:18788` | `structures.rs` |
| C4 | Spawn Point | SpawnPoint | `WorldGen.cs:19460` | `pipeline.rs` (replaces the compact spawn in `step_world_tail`) |
| C5 | Guide | SpawnStarterNPCs | `WorldGen.cs:19685` | new `starter_npcs.rs` (gen-time NPC ledger) |
| C6 | Sunflowers | SunflowersPart2 | `WorldGen.cs:19898` | `surface.rs` |
| C7 | Herbs | AlchemyHerbs | `WorldGen.cs:19964` | `surface.rs` (`PlantAlch` :45901) |
| C8 | Dye Plants | DyePlants | `WorldGen.cs:19981` | `surface.rs` (`plantDye` :71286 + `MatureTheHerbPlants` :45748 + `GrowGlowTulips` :45622 + `PlaceChilletEggs` :45652) |

Neighbours NOT this lane: Surface Ore and Stone `:18447` (g37's B9), Grass
Wall `:19589` (d106 landed), Flowers `:20447` (d105/g39's d-slice). The
d104/d105 *pending* overlap (Place Fallen Log..Spawn Point in d104's stated
range, Guide..Dye Plants in d105's) is exactly what g37's mop-up split
reassigns; the merger reconciles per `12h`.

## Discipline

- Verbatim ports (golden rule): same structure, maths, draw order; citation
  comment per block; readable Rust names; C# read in ≤100-line windows.
- Each pass gets its `EXECUTED_PIPELINE` row at the vanilla slot + a
  positional witness; per-pass `pass_rng("<vanilla name>")` stream so the
  insertion shifts no other step's ledger.
- **Hash re-pins are DEFERRED to the merge gate** (code-only mode — no
  cargo/tests/graphify on this branch, the S1-K-2-wave-B / d10x convention):
  `world_hash.rs` gains a per-pass stale-by-design note; the `a == b`
  determinism halves of both pin tests stay untouched and must stay green
  at the gate.
- ONE pass per commit+push; docs + `anchors.txt` row per pass.

## Landing ledger

| pass | commit | status |
| --- | --- | --- |
| range claim (this doc) | d32b13cc | claimed |
| C1 Place Fallen Log | 8c9b7b60 | landed (code-only, witnesses written NOT run) |
| C2 Traps | (this commit) | landed (code-only, witnesses written NOT run) |
| C3 Piles | — | pending |
| C4 Spawn Point | — | pending |
| C5 Guide | — | pending |
| C6 Sunflowers | — | pending |
| C7 Herbs | — | pending |
| C8 Dye Plants | — | pending |

## C1 landing note (2026-11-21)

Re-probe before C1: d104/d105 still claim-only (2d9cfcbb / 5bb4116e), d106
still Grass-Wall-only (ee0fcbef), g36 claim-only (0b01c170), g37 census
re-probe only (30f605c7 — all nine B-passes unlanded), f25 claim-only
(fd098528), g39 pivoted to the **helper lane** (`12j`, H1-H15 landed —
general `Place3x2` in its new `place_helpers.rs`, unwired). None of the
eight C-passes landed anywhere; the claim stands.

C1 ports `WorldGen.cs:18513-18656` verbatim into `surface.rs`
(`place_fallen_log` + the `PlaceTile(488)` slice + a local type-488
`Place3x2` slice + `SolidTile2(int,int)` `:70492` + `denySomeGeneration`
`:3133`). The local `place_3x2` duplicates g39's H6 general helper by
design at this base (g39's `place_helpers.rs` is not on this branch); the
merger dedupes onto the general helper — the 488 path takes its default
arm, identical behaviour. Slot: after `cave houses`, before `thin ice`
(the nearest executed neighbours of the vanilla `:18513` slot between
Surface Ore and Stone `:18447` and Traps `:18657`). Stream:
`pass_rng("Place Fallen Log")` — no other step's ledger shifts.
`GenVars.log_x/log_y` (Reset −1, `:11210-11211`) published for the Flowers
consumers (`:20468`/`:20532`, d105/g39's slice). Boot static: 488
frame-important (`Main.cs:7644`). Progress message `Lang.gen[85]`
(`:18524`). Deferred (flag-gated, documented in the pass doc comment):
the extraLiquid water-features head (`DoExtraLiquidAddBubbleBlocks`
`:1648-1807` — the settle-liquids lane's QuickWater dependency) and the
remix `GenVars.remixSurfaceLayerLow/High` band (nearest published anchors
`rock_low`..`height-350` drawn in the C#'s spot). Hash pins: SEED42_512
expected UNCHANGED (the `[400,112)` x band collapses on 512 — width guard
after the count roll, the mining-explosives precedent); DEFAULT_WORLD
moves BY DESIGN (logs place on the 4200-wide world). Re-pins deferred to
the gate; `a == b` halves untouched. Witnesses: 3 unit tests in
`surface.rs` (frame write + floor/body/bounds vetoes, the existing-log
veto, the gates + width-guard draw schedule) + the pipeline positional
witness `w4_g38_c1_the_fallen_log_step_sits_after_the_cave_houses`; the
step-count/name-list tests updated 66→67.

## C2 landing note (2026-11-21)

C2 ports `WorldGen.cs:18657-18787` (the Traps pass) + `placeTrap`
(`:8922-9513`, all four variants) + `PlaceSandTrap` (`:35463-35739`) +
`placeTNTBarrel` (`:8886-8907`) verbatim into `structures.rs`
(`pass_traps` + `place_trap`/`place_trap_dart`/`place_trap_boulder`/
`place_trap_explosive`/`place_trap_geyser` + `place_sand_trap` +
`place_tnt_barrel`, with local `trap_*` helpers for `SolidTile`/
`SolidTile2`/`SolidTileAllowBottomSlope`/`Place1x1`/`Place2x2`/`PlaceTile`/
`KillTile`/`IsTileNearby`/`CanGeneratePressurePlateAt`/the three
`closeEnoughTo*` proximity scans). Slot: after `fallen log`, before
`thin ice` (the vanilla `:18657` slot between Place Fallen Log `:18513`
and Piles `:18788`). Stream: `pass_rng("Traps")` — no other step's ledger
shifts. `GenVars.boulder_pets_placed` (Reset 0, `:8323`) published for the
pet-boulder budget. Boot statics: 135/138/664/484/711-716/443/654
frame-important (`Main.cs:7721`/`:7432-7441`/`:9716`/`:7362`; 137/141
already present). Progress message `Lang.gen[34]` (`:18663`). The
pass-local `Main.tileSolid[379]=false` + `SetBoulderSolidity` windows ride
a `boulder_solid` flag threaded into `trap_solid` (the port's `TILE_SOLID`
is static). Deferred (flag-gated / port-model seams, documented in the
section note): `placingTraps` (UI-only), `starGame` (runtime joke seed),
`shimmerPosition` proximity (no shimmer position yet), the dual-dungeon
`placeTrap_CanContinue` arm (no bounds model), `trapDiag` (diagnostics),
the drunk `Thread.Sleep` delay loop (draws/writes nothing). Hash pins:
SEED42_512 expected UNCHANGED (the 512 terrain rejects every trap attempt
+ no wall-187 desert); DEFAULT_WORLD moves BY DESIGN (210 trap attempts +
12 sand-trap attempts place on the 4200-wide world). Re-pins deferred to
the gate; `a == b` halves untouched. Witnesses: 5 unit tests in
`structures.rs` (bubble/boulder solidity windows, Place1x1 plate frames +
floor veto, Place2x2 boulder frames + floor veto, the dart trap
plate+dart+wire, the pass gates' no-draw schedule) + the pipeline
positional witness `w4_g38_c2_the_traps_step_sits_after_the_fallen_log`;
the step-count/name-list tests updated 67→68.
