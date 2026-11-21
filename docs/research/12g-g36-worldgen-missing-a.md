# 12g — W4-g36: worldgen missing-a (mop-up lane, first quarter)

> Wave 4 · lane `task/W4-g36-worldgen-missing-a` @ base a725acb0 · claimed 2026-11-20
> Parent audit: `docs/research/12-worldgen.md` (the unexecuted-pass census).
> Census source: d101's `12b-d101-passes-1.md` at its pushed tip (4194acf8) —
> **42 unexecuted classic passes** after d101's nine rows landed.

## In-flight probe (all pushed branches + census docs read at claim time)

- **d101** (4194acf8, `12b`): all 9 rows landed (#1-8 + the Granite row-split).
- **d102** (28ec5561, `12a`): Silt c746a7da, Webs 96e4b92f, Slush d4fd0af0,
  Mountain Caves 665041f8, Gems 28ec5561 **landed**; Shimmer, Clean Up Dirt,
  Dirt Rock Wall Runner, Wood Tree Walls, Settle Liquids still pending there.
- **d103** (561b3ab2, lane-split section in `12-worldgen.md`): Oasis 90b7eee6,
  Waterfalls 561b3ab2 landed; #21-27 pending.
- **d104** (2d9cfcbb, `12d`): claim only (#28-36).
- **d105** (5bb4116e, `12e`): claim only (#37-45).
- **d106** (ee0fcbef, `12f`): Grass Wall landed (R1); R2-R6 pending.
- **f24**: still nothing pushed (no remote branch).
- **f25** (fd098528, `12g-f25-settle-liquids.md`, committed 22:08:23 —
  **94 s before this lane's claim**): orphan adoption of **Settle Liquids**
  (`WorldGen.cs:16132-16210`), the one pass in no d10x lane's own ledger,
  per the d106 Grass Wall precedent. Its claim predates ours → **this lane
  yields A5 Settle Liquids to f25** (re-probe amendment below).
- **g37** (8ef3e111 claim + 30f605c7 re-probe, `12h`): claim only — fixes the
  deterministic quartered split of the 37-pass remainder (as of its claim
  time) over g36/g37/g38/g39: **a = #1-9, b = #10-18, c = #19-27, d = #28-37**,
  and "nothing here touches a pass a d10x lane has LANDED"; "If a sibling's
  pushed claim differs, the merger reconciles".
- **g38** (d32b13cc, `12i`): claim only — c-quarter minus Grass Wall.
- **g39** (b24483c2 tip): H-helper ports (Place2x2/3x2/3x3/3x4/5x4/6x3/4x2,
  wall-painting wrappers, PlaceLogicTiles) — no classic pass rows; d-quarter
  passes still pending there.

## The gap this lane takes

g37's published a-slice (#1-9 of the 37-remainder): Mountain Caves, Gems,
Shimmer, Clean Up Dirt, Dirt Rock Wall Runner, Wood Tree Walls, Settle
Liquids, Wall Variety, Life Crystals. Since g37's claim, d102 has **landed**
Mountain Caves (665041f8) and Gems (28ec5561) — per g37's own rule this lane
does not touch them. The remaining census is 42 − 2 (d102) − 1 (d106 Grass
Wall) = **34 unexecuted**; this lane takes the a-quarter minus the two
delivered rows, minus the f25 orphan claim of Settle Liquids (fd098528,
22:08:23, earlier than this claim): **6 passes**, all still unlanded on every
pushed branch and in no other lane's own ledger.
The d102/d103 *pending* overlap (Shimmer..Wood Tree Walls in d102's stated
range, Wall Variety/Life Crystals in d103's) is exactly what g37's mop-up
split reassigns; the merger reconciles per `12h`. Settle Liquids was in the
a-slice but f25's earlier pushed orphan claim wins it — this lane drops A5.

### This lane's six passes (C# anchor order)

| # | Pass (`VANILLA_CHAIN` name) | C# anchor | Rust home (planned) |
| --- | --- | --- | --- |
| A1 | Shimmer | `WorldGen.cs:15195` (+ `ShimmerMakeBiome` :34853, `ShimmerMakeBiomeOpening` :35059) | new `worldgen/shimmer.rs` |
| A2 | Clean Up Dirt | `WorldGen.cs:15246` | `micro_finish.rs` |
| A3 | Dirt Rock Wall Runner | `WorldGen.cs:15461` (+ `DirtyRockRunner` :77145) | `micro_finish.rs` |
| A4 | Wood Tree Walls | `WorldGen.cs:15717` | `living_trees.rs` |
| A5 | ~~Settle Liquids~~ | `WorldGen.cs:16132` | **yielded to f25** (fd098528 orphan claim, earlier) |
| A6 | Wall Variety | `WorldGen.cs:16697` | `micro_finish.rs` |
| A7 | Life Crystals | `WorldGen.cs:16755` | `structures.rs` |

Neighbours NOT this lane: Gems :15050 (d102 landed), Oasis :16243 /
Waterfalls :16610 (d103 landed), Statues :16853 (g37's B1).

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
| range claim (this doc) | 0b01c170 | claimed |
| claim amendment (f25 yield + re-probe) | f474adc1 | claimed |
| A1 Shimmer | 702e1e75 | landed (code-only; sub-vanilla width guard — SEED42_512 unchanged; DEFAULT_WORLD re-pin deferred to the gate) |
| A2 Clean Up Dirt | 7260a9aa | landed (code-only; both pins move by design — re-pin deferred to the gate) |
| A3 Dirt Rock Wall Runner | 7005940a | landed (code-only; both pins move by design — re-pin deferred to the gate) |
| A4 Wood Tree Walls | aa3ee000 | landed (code-only; RNG-free; both pins move by design — re-pin deferred to the gate) |
| A5 Settle Liquids | fd098528 (f25) | yielded — f25's earlier orphan claim wins |
| A6 Wall Variety | (this commit) | landed (code-only; both pins move by design — re-pin deferred to the gate) |
| A7 Life Crystals | — | pending |
