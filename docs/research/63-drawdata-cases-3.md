# 63 — DrawData case census, slice 3 (W4-d63)

> Wave 4 · researched 2026-11-20 · branch `task/W4-d63-drawdata-cases-3` @ base a725acb0 ·
> code-only mode (no builds/tests run) · sources: `decompiled/Terraria/GameContent/Drawing/TileDrawing.cs`,
> `crates/tr-core/src/drawdata.rs`, `crates/tr-core/src/drawdata_furniture.rs`,
> `crates/tr-app/src/drawdata_cases_a.rs`, `crates/tr-app/src/drawdata_cases_b.rs`,
> `crates/tr-app/src/render/tiles.rs`.

## Range claim (documented early, per wave protocol)

The four W4 drawdata slices partition the **C#-order union of `GetTileDrawData` case ids**
(first switch `TileDrawing.cs:4540-6032`, then the second/glow switch `:6049-6507` ids not
already in the first; 361 + 10 = **371 distinct ids**). The in-flight sibling d64
(`origin/task/W4-d64-drawdata-cases-4`, `docs/research/64-drawdata-cases-4.md` @ `161c6389`)
claimed the LAST quarter and stated the convention; this doc takes the THIRD quarter:

| Slice | Index range | Ids |
| --- | --- | --- |
| d61 | [0, 93) | 93 |
| d62 | [93, 186) | 93 |
| **d63 (this doc)** | **[186, 279)** | **93** |
| d64 | [279, 371) | 92 + the 215 epilogue |

Boundary cross-check (brace-balanced re-extraction, `tools/_d63_census.py`, depth-1
labels only — the nested cage/monolith sub-switches at :5147/:5674/:5783 are bodies):
index 185 = `354@5310` (last d62), index 186 = `355@5311` (first d63), index 278 =
`286@5757` (last d63), index 279 = `298@5758` (first d64 — matches d64's claimed list
verbatim). The 215 campfire epilogue (`:6033-6044`, not a case label) belongs to d64.

**d63 owns these 93 ids** (C# order; `@` = first-switch label line; all 93 are
first-switch ids — none of the 10 second-switch-only ids fall in this quarter):

```
355@5311 499@5312 377@5316 463@5320 464@5321 491@5325 379@5329 349@5332
441@5342 468@5343 390@5357 412@5360 36@5364 406@5367 452@5387      (batch 2)
455@5397 454@5409 453@5412 456@5415 405@5418 12@5433 31@5434 96@5435
639@5436 665@5437 696@5438 238@5441 593@5445 594@5466 592@5488      (batch 3)
228@5491 231@5492 243@5493 247@5494 244@5498 565@5509 235@5520
217@5523 218@5524 564@5525 219@5529 220@5530 642@5531 270@5535
271@5536                                                          (batch 4)
581@5537 660@5564 572@5591 428@5605 442@5612 275@5629 276@5630
277@5631 278@5632 279@5633 280@5634 281@5635 296@5636 297@5637
309@5638                                                          (batch 5)
358@5639 359@5640 413@5641 414@5642 542@5643 550@5644 551@5645
553@5646 554@5647 558@5648 559@5649 599@5650 600@5651 601@5652
602@5653                                                          (batch 6)
603@5654 604@5655 605@5656 606@5657 607@5658 608@5659 609@5660
610@5661 611@5662 612@5663 632@5664 640@5665 643@5666 644@5667
645@5668                                                          (batch 7)
710@5669 285@5756 286@5757                                        (batch 8)
```

Overlap note: ids 441/468 (and 349's dresser group) appear in BOTH switches; their
first-switch index falls in d63's quarter, so the second-switch chest-glow rows
(`:6110-6156` region) are audited here alongside the first-switch body — d64's doc
explicitly defers "the 21/467/441/468 chest glow rows" to the d62/d63 quarters.
Ids 285/286 share the small-animal cage body with d64's 298..629 tail; the shared
group header is cited once per doc, the inner arms audited by whoever owns the id.

## Headline census result — the "missing cases" premise is stale

d64's brace-balanced census (`161c6389`) shows **371/371 ids covered** across
`drawdata.rs` / `drawdata_cases_a.rs` / `drawdata_cases_b.rs` / `drawdata_furniture.rs`
— 0 missing case arms. This doc independently re-extracted the same 371-id union
(`tools/_d63_census.py` → `tools/_d63_union.txt`) and confirms the count. The d63
quarter's per-case audit below verifies each arm against the C# verbatim and records
what is *actually* still missing on the draw side (static-only deviations, unwired
furniture passes, unconsumed glow records — not case gaps). research/09's
"≈113 of ≈376 case ids" (09-tile-drawing.md:31/111/188) is STALE, predating S1-F-9
waves A-E.

d62's in-flight classification (`TR-W4-d62/tools/_d62_census_out.txt`, untracked)
flags these d63-quarter ids as STATIC (deviation-comment arms in `draw_data`) or
DELEG (delegated pass): 349, 441, 468, 405, 275-281, 296, 297, 309, 358, 359, 413,
414, 542, 550, 551, 553, 554, 558, 559, 599-612, 632, 640, 643-645, 710, 285, 286
(STATIC) and 453, 456 (DELEG → `cases_b`). The audit below re-derives each verdict
from the C# verbatim rather than trusting the comment tags.

## Per-case audit

Filled in per batch below (15 ids per commit). Verdict key: **PORTED** (arm faithful,
wired), **PORTED-UNWIRED** (arm faithful, no live caller), **DEVIATION** (documented
approximation — static part only), **MISSING** (no arm anywhere).
