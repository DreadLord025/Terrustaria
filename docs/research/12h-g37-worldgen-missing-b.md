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

## Landing ledger

| pass | commit | status |
| --- | --- | --- |
| range claim (this doc) | 8ef3e111 | claimed |
| census re-probe + claim confirmation (this doc) | (this commit) | confirmed — all nine UNLANDED on every pushed branch |
| B1 Statues | — | pending |
| B2 Surface Chests | — | pending |
| B3 Water Chests | — | pending |
| B4 Cave Walls | — | pending |
| B5 Floating Island Houses | — | pending |
| B6 Quick Cleanup | — | pending |
| B7 Hellforge | — | pending |
| B8 Spreading Grass | — | pending |
| B9 Surface Ore and Stone | — | pending |
