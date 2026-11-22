# 68 addendum — FindFrame census after f26 slice 7c (code-only)

Audit date: 2026-11-21. Branch `task/W4-f26-findframe-extra-3`, slice
7c, base tip `b9b49f66` (slice 7b DONE). **Code-only mode: no
cargo/rustc/tests/graphify invoked** — every claim is git/file-read
evidence against the PUSHED remote tips, re-probed at census time.

The master wire-census lives at `docs/research/68-findframe-wire-status.md`
on `task/W4-d44-findframe-low-1` (tip `7cb3f8d0` — the d44 wire-round
docs); this addendum rides the f26 branch so the two lanes never
text-conflict. The merger folds the count below into the master §1.

## The compatible census count (the 7b method re-run)

| step | subtraction | remaining |
| --- | --- | --- |
| C# top-level `case` ids (`NPC.cs:66705`, two-tab labels) | — | 602 |
| port-covered on main (`npc.rs::find_frame` arms via the `npc/data.rs:2992` cs_id hand rows + bestiary net ids + wave-A/B rosters + the five folded segment families 135/246/249/371/373) | 248 | 354 |
| d44-d49 claims (75 ids, the master §2 tables) | 75 | 279 (the e13 base) |
| deduped e13/e14/f26×3/f27-wave-4 claim ids | 75 | 204 |
| f27 slice-7 pushed claim (`ef07e06b`: 316-318/322-324/333-337 + 437/438/440/442) | 15 | 189 |
| d38 LANDED arms (tip `8176c078` — the 297..304 in-band remainder ALL wired) | 8 | 181 |
| d40 LANDED arms 443-454/460-470/472 (batches 3-8) | 24 | **157** (the 7b census) |
| f26 slice-7b pushed claim (`b9b49f66`: 473-477/479-488) | 15 | **142** |
| f26 slice-7c THIS claim (494-506/508/509) | 15 | **127 unclaimed** |

## The re-probe exclusions (zero net — all dedupe)

* **d40 tip `8fe1d6d3` (batches 9-12)** landed INLINE `npc.rs`
  find_frame arms for 473..488. Only **480 Medusa / 481 GreekSkeleton
  / 482 GraniteGolem are FAITHFUL**; the other twelve are row-0 stubs
  (the aiStyle machines UNWIRED, the style73 precedent). All 15 ids
  are inside the already-subtracted 7b claim set — no new census
  subtraction. The arm-by-arm keep-one verdict (module-vs-inline,
  which faithful) is published in docs/ANALYSIS.md "W4-f26 FindFrame
  extra wave 7c": **480/481/482 keep the INLINE arms at merge** (both
  sides verbatim-identical, row-index vs pixel-row convention only;
  the inline arms are already in `NpcKind` + wired through
  `npc_rows`); **the twelve stub kinds keep the extra7b MODULE arms**
  (the only faithful machines — the wiring round replaces the stubs).
* **f27 tip `72448094` (slice 8)** claims 443-454/460/461/462 — inside
  the d40-landed subtraction; its faithful arms supersede the d40
  row-0 stubs there (the f27 ANALYSIS d40-merger notes). Dedupes.
* **d41 (`491-560`) / d42 (`561-630`) have NO pushed refs** (`git
  ls-remote origin` probed 2026-11-21; the local branches sit at the
  wave-3 land base `a725acb0` with zero FindFrame arms). **d43's
  pushed tip IS `a725acb0`** (no work yet). No kind in the 494+ band
  has a landed faithful arm on ANY pushed tip — the band was fully
  open at claim time.

## This slice's claim (15 kinds, 5 per commit)

494/495 Crawdad pair (`NPC.cs:69657-69733`), 496/497 GiantShelly pair
(`:69994-70066`), 498..506 the nine Salamanders (`:69849-69907`), 508
GiantWalkingAntlion (the 508/532/580/582 group `:70067-70103` — only
508 claimed), 509 GiantFlyingAntlion (the 509/581 group
`:70104-70119` — only 509 claimed). Module
`crates/tr-app/src/npc_findframe_extra7c.rs` + the
`find_frame_kind_extra7c` seam; 10 witnesses WRITTEN NOT RUN
(code-only); zero-RNG grep over all five cited ranges = ZERO; frame
counts cross-checked `Main.cs:67082` vs `NPC_FRAME_COUNT` (494/495=8,
496/497=12, 498-506=11, 508=6, 509=9). Commits: batch 1 `8dd9c170`,
batch 2 `10661c1c`, batch 3 `f4b5f97c`.

The next ascending unclaimed ids after this slice: 516/521/523/524/
525/526/527/528/529/530/531/532/534/535/536 (the 516+ band; 507/510-
515/517/519/522 have no case labels and 520 is port-covered — the
main `npc.rs` MartianWalker arm, cs_id 520 — re-check at the next
census). The five low-id remainders 135/246/249/371/373 are the
folded segment families (port-covered via the folded net ids, NOT
unclaimed — the mechanical raw-diff artefact the 248 figure already
absorbs).
