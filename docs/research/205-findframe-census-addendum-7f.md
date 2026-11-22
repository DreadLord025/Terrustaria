# 205 - FindFrame census addendum after f26 slice 7f (was key 68; renumbered per registry-150 §3/§6.3/§7.3, KEEPS §9.1: 7c->201 7e->203 7f->205 7g->251 7h->281 7i->282 7j->283; executed at the plan-B3 merge of task/W4-f26-findframe-extra-3)

> RENUMBER # 68 addendum — FindFrame census after f26 slice 7f (code-only)

Audit date: 2026-11-21. Branch `task/W4-f26-findframe-extra-3`, slice
7f, base tip `f1a0eaa9` (slice 7e DONE + docs). **Code-only mode: no
cargo/rustc/tests/graphify invoked** — every claim is git/file-read
evidence against the PUSHED remote tips, re-probed at census time.

The master wire-census lives at `docs/research/200-findframe-wire-status.md`
on `task/W4-d44-findframe-low-1` (tip `7cb3f8d0`); the 7c addendum
(`201-findframe-census-addendum-7c.md`) carries the full derivation
table. This addendum rides the f26 branch so the two lanes never
text-conflict. The merger folds the count below into the master §1.

## The compatible census count (the 7c/7d/7e method re-run)

| step | subtraction | remaining |
| --- | --- | --- |
| the 7e census after this branch's 5 claims (`9c2b0a84`) | — | 117 |
| f26 slice-7f THIS claim (531/532/534/535/536) | 5 | **112 unclaimed** |

## The re-probe exclusions (zero net — all dedupe or no-arm)

* **f27 tip `0e187428`** (slice 9 + docs, UNCHANGED since the 7e
  census): `git show …:docs/ANALYSIS.md | findstr /C:"slice 10"` =
  NO match — there is NO slice-10 claim band; the wave-7/8/9 claim
  sections stop at the 463/467/471/472/492 fix set (inside the
  already-subtracted d40/f27 bands). Mechanical grep over its
  `npc_findframe_extra7.rs`/`extra8.rs`/`extra9.rs` for
  531/532/534/535/536: the ONLY hit is the extra-7 slime
  group-comment mention of 535 (`:70801-70846`'s case-list note) —
  no machine, no kind, no seam row. Zero FindFrame arms.
* **533 DesertDjinn** is port-covered — the S1-E-5 `npc.rs:8086`
  arm (the 7e addendum's tail note re-confirmed on this tree;
  anchors row 311). EXCLUDED from the claim window; the five picked
  ids skip it.
* **d41 tip `4ed22f0b`** is STILL the CLAIM-only commit (the
  SetDefaults census, band 491-560). Mechanical grep over its
  `npc.rs`/`npc/data.rs` for all five ids + the five kind names
  (`DesertScorpionWall`/`DesertBeast`/`DemonTaxCollector`/
  `SlimeSpiked`/`TheBride`) = ZERO.
* **d42 tip `9b22c091`** (ADVANCED past the 7e census's `88df5809` —
  now the 578-630 roster-join docs commit). Same grep = ZERO (the
  band starts at 561; the five ids are below it).
* **d40 tip `8fe1d6d3`** unchanged — the roster stops at 490.
* **d43 tip `a725acb0`** IS the wave-3 base (no work).
* **d44-d49 / e13 / e14**: the d44 master wire-status table rows for
  the wall group's co-members: 165 WallCreeperWall
  `WIRED(d46-self)`, 237 JungleCreeperWall + 238 BlackRecluseWall +
  240 BloodCrawlerWall `WIRED(g7)` (f27) — port-covered, NOT
  unclaimed; only 531 was open in that group. The 508/580/582
  antlion co-members: 508 rides this branch's 7c module (claimed);
  580 WalkingAntlion / 582 LarvaeAntlion stay unclaimed ABOVE this
  slice's 531-540 window (the 7c module's "ride later slices" note).
  The zombie group's low-id
  members ride wave A / the base `npc.rs::walker_frame`; the slime
  group's low-id members ride wave A / the base Slime arm.
* **This branch's own `npc.rs`**: no arm for any of the five (no
  `DesertScorpionWall`/`DesertBeast`/`DemonTaxCollector`/
  `SlimeSpiked`/`TheBride` kind in `NpcKind`).

## This slice's claim (5 kinds, ONE commit)

531 DesertScorpionWall — the 165/237/238/240/531 wall group
(`NPC.cs:73112-73145`, label `:73116`): the `(|vx| + |vy|) * num194`
EVERY-tick accumulator with the 531-ONLY `num194 = 0.4` override
(`:73119-73122`), the <6/<12/<18/<24 bands rows 0/1/2/3, the else
counter-zero WITHOUT a row write (the row HOLDS 3 — the extra-3 Moth
quirk). 532 DesertBeast — the 508/532/580/582 walking-antlion group
(`:70067-70103`, label `:70068`), the VERBATIM 7c machine copy on the
8-row count (the walk strip widens to rows 2..7). 534
DemonTaxCollector — the humanoid-group case list (`:68615`) with the
ONE `num108 = 12` stride override (`:69528-69531`, the f27 Fritz
inline precedent), num57 = 0 (aiStyle 3 `:15817`, no townNPC), idle
row 0, air row 1 (not in the `:69647` pin list); the ai[0] pose zones
the documented skip. 535 SlimeSpiked — the slime strip group
(`:70818` → `:70828-70845`), the VERBATIM extra-3 num2-fed machine
(the wave-A slime keep-one). 536 TheBride — the zombie walker group
(`:76344` → `:76347-76390`), the VERBATIM wave-A `zombie_walk_frame`
copy (the air/turn row-2 pin, the <8/<16/<24/<32 bands, the lap
row-hold quirk). Module
`crates/tr-app/src/npc_findframe_extra7f.rs` + the
`find_frame_kind_extra7f` seam; 11 module witnesses + the Python
replay (`tools/_f26_7f_replay.py`) + the JS sweep twin
(`tools/_f26_7f_sweep.js`) WRITTEN NOT RUN (code-only); zero-RNG +
zero-world-read grep over the five cited ranges = ZERO; frame
counts cross-checked `Main.cs:67082` vs `NPC_FRAME_COUNT` (531=4,
532=8, 534=8, 535=2, 536=3 — `:67137` entries 2/3/5/6/7; all 697
slots re-diffed = ZERO mismatches, a mechanical text-parse diff).

The next ascending unclaimed ids after this slice: 537/538/539/540
(537 SandSlime rides the SAME slime strip group `:70819`; 538/539
the squirrel pair `:76459-76460`; 540 PartyBunny the bunny hop group
`:76857` — d44's 46/47 + f27's 337/443 keep-one; re-check at the next
census).
