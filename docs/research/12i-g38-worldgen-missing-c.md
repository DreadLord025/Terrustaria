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
| range claim (this doc) | (this commit) | claimed |
| C1 Place Fallen Log | — | pending |
| C2 Traps | — | pending |
| C3 Piles | — | pending |
| C4 Spawn Point | — | pending |
| C5 Guide | — | pending |
| C6 Sunflowers | — | pending |
| C7 Herbs | — | pending |
| C8 Dye Plants | — | pending |
