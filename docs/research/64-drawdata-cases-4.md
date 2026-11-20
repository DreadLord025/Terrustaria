# 64 — DrawData case census, slice 4 (W4-d64)

> Wave 4 · researched 2026-11-20 · branch `task/W4-d64-drawdata-cases-4` @ base a725acb0 ·
> code-only mode (no builds/tests run) · sources: `decompiled/Terraria/GameContent/Drawing/TileDrawing.cs`,
> `crates/tr-core/src/drawdata.rs`, `crates/tr-core/src/drawdata_furniture.rs`,
> `crates/tr-app/src/drawdata_cases_a.rs`, `crates/tr-app/src/drawdata_cases_b.rs`,
> `crates/tr-app/src/render/tiles.rs`.

## Range claim (documented early, per wave protocol)

The four W4 drawdata slices partition the **C#-order union of `GetTileDrawData` case ids**
(first switch `TileDrawing.cs:4540-6032`, then the second/glow switch `:6049-6510` ids not
already in the first; 361 + 10 = **371 distinct ids**, plus the 215 campfire epilogue
`:6033-6044` which is not a `case` label). Consecutive quarters by index:

| Slice | Index range | Ids |
| --- | --- | --- |
| d61 | [0, 93) | 93 |
| d62 | [93, 186) | 93 |
| d63 | [186, 279) | 93 |
| **d64 (this doc)** | **[279, 371)** | **92 + the 215 epilogue** |

**d64 owns these 92 ids** (C# order; `@` = first-switch label line, glow-only ids cite the
second switch):

```
298@5758 299@5759 310@5760 339@5761 361@5762 362@5763 363@5764 364@5765
391@5766 392@5767 393@5768 394@5769 532@5770 533@5771 538@5772 544@5773
555@5774 556@5775 582@5776 619@5777 629@5778          (small-animal cages tail)
282@5833 505@5834 543@5835 598@5843 568@5851 569@5852 570@5853
288@5861 289@5862 290@5863 291@5864 292@5865 293@5866 294@5867 295@5868
360@5869 580@5870 620@5871 521@5884 522@5885 523@5886 524@5887 525@5888
526@5889 527@5890 316@5898 317@5899 318@5900          (water/jellyfish cages)
207@5908 410@5930 480@5945 509@5946 657@5947 720@5948 721@5949 725@5950
658@5962 733@5982                                     (fountain + monoliths)
326@5991 327@5992 328@5993 329@5994 345@5995 351@5996 421@5997 422@5998
458@5999 459@6000 708@6001                            (fall blocks + conveyors)
541@6004 507@6007 508@6008 336@6014 340@6015 341@6016 342@6017 343@6018
344@6019 89@6023 102@6026 753@6029                    (echo/starry/fire/benches)
19@6157 34@6268 10@6312 11@6323 88@6343 42@6363 87@6377 93@6414 101@6446
104@6466                                              (second-switch glow rows)
215@6033 (campfire epilogue, not a case label)
```

Overlap note: ids 568/569/570/580/657/89 appear in BOTH switches; the first-switch body
is audited here (their index falls in d64's quarter) and the second-switch glow rows are
cited alongside. The 21/467/441/468 chest glow rows belong to d62/d63 quarters and are
NOT claimed here.

## Headline census result — the "missing cases" premise is stale

A brace-balanced extraction of every outer `case` label in both switches
(`TileDrawing.cs:4540-6032` + `:6049-6510`, depth-1 labels only — the nested cage/monolith
sub-switches are bodies, not new cases) yields **371 distinct ids**. Diffing against the
union of all four port modules' match arms / guards:

| Port module | Ids covered (∩ C# set) |
| --- | --- |
| `tr-core/src/drawdata.rs::draw_data` (:130-1031) + 215 epilogue (:1036) | 353 + 215 |
| `tr-app/src/drawdata_cases_a.rs` (518/529/530 probes + glow switch) | 41 |
| `tr-app/src/drawdata_cases_b.rs::case_draw_data` (:107) | 16 |
| `tr-core/src/drawdata_furniture.rs::cage_table` (:111) + `is_basic_chest` | 94 |
| **Union** | **371 / 371 — 0 missing** |

The 18 ids absent from `draw_data` alone are all delegated, not missing:
719/518/579/453/456/541/507/508 → `cases_b`/`cases_a` (engine-driven arms, wired at
`render/tiles.rs:700`/`:731`); 19/34/10/11/88/42/87/93/101/104 → `cases_a` glow switch
(wired at `tiles.rs:731`). ROADMAP S1-F-9's "352 of 361, only 9 engine-state ids stay
default (5/453/456/507/508/518/541/579/719)" is now 353 of 361 — case 5 gained its static
20×20 arm in wave A (`c654e020`), leaving exactly the 8 delegated ids above.

Partition convention note: at base a725acb0 no sibling slice (d61/d62/d63) had documented
a range (their branches sit at the same base; d61's worktree holds only untracked census
scripts). This doc claims the LAST quarter in C# order and states the convention so the
other slices can take [0,93)/[93,186)/[186,279) without overlap.

**Therefore research/09's "≈113 of ≈376 case ids" (09-tile-drawing.md:31/111/188) and
90-consolidated-findings.md:50 are STALE** — they predate S1-F-9 waves A-E
(`2eedb54d`/`b38a1cb3`/`4bfffc19`/`16c00bcc`). The per-case audit below verifies the
d64 quarter arm-by-arm against the C# verbatim and records what is *actually* still
missing on the draw side (wiring, not cases).

## Per-case audit

Filled in per batch below. Verdict key: **PORTED** (arm faithful, wired),
**PORTED-UNWIRED** (arm faithful, no live caller), **DEVIATION** (documented approximation).

*(batches land in subsequent commits)*
