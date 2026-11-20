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

## Status

- [ ] 28 Floating Island Houses
- [ ] 29 Quick Cleanup
- [ ] 30 Hellforge
- [ ] 31 Spreading Grass
- [ ] 32 Surface Ore and Stone
- [ ] 33 Place Fallen Log
- [ ] 34 Traps
- [ ] 35 Piles
- [ ] 36 Spawn Point