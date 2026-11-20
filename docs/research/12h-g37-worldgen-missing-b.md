# 12h — W4-g37: worldgen missing-b (mop-up lane, second quarter)

> Wave 4 · lane `task/W4-g37-worldgen-missing-b` @ base a725acb0 · claimed 2026-11-20
> Parent audit: `docs/research/12-worldgen.md` (the unexecuted-pass census).
> Sibling lanes read at claim time (pushed docs only): d101 `12b-d101-passes-1.md`
> (9/9 landed), d102 `12a-d102-passes-2.md` (Silt/Webs/Slush landed, rest pending),
> d103 `12-worldgen.md` lane-split section (Oasis/Waterfalls landed, rest pending),
> d104 `12d-worldgen-passes-4.md` (claim only), d105 `12e-worldgen-passes-5.md`
> (claim only), d106 `12f-worldgen-passes-6.md` (claim only).

## The gap this lane takes

The d10x census lists 50 classic unexecuted passes. Landed on the sibling
branches: d101 #1-8 (Sand Patches..Generate Ice Biome) + the Granite row-split,
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

## Landing ledger

| pass | commit | status |
| --- | --- | --- |
| range claim (this doc) | (this commit) | claimed |
| B1 Statues | — | pending |
| B2 Surface Chests | — | pending |
| B3 Water Chests | — | pending |
| B4 Cave Walls | — | pending |
| B5 Floating Island Houses | — | pending |
| B6 Quick Cleanup | — | pending |
| B7 Hellforge | — | pending |
| B8 Spreading Grass | — | pending |
| B9 Surface Ore and Stone | — | pending |
