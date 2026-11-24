# 203 - FindFrame census addendum after f26 slice 7e (was key 68; renumbered per registry-150 §3/§6.3/§7.3, KEEPS §9.1: 7c->201 7e->203 7f->205 7g->251 7h->281 7i->282 7j->283; executed at the plan-B3 merge of task/W4-f26-findframe-extra-3)

> RENUMBER # 68 addendum — FindFrame census after f26 slice 7e (code-only)

Audit date: 2026-11-21. Branch `task/W4-f26-findframe-extra-3`, slice
7e, base tip `63417edd` (slice 7d DONE). **Code-only mode: no
cargo/rustc/tests/graphify invoked** — every claim is git/file-read
evidence against the PUSHED remote tips, re-probed at census time.

The master wire-census lives at `docs/research/200-findframe-wire-status.md`
on `task/W4-d44-findframe-low-1` (tip `7cb3f8d0`); the 7c addendum
(`201-findframe-census-addendum-7c.md`) carries the full derivation
table. This addendum rides the f26 branch so the two lanes never
text-conflict. The merger folds the count below into the master §1.

## The compatible census count (the 7c/7d method re-run)

| step | subtraction | remaining |
| --- | --- | --- |
| the 7c census after this branch's 15 claims (`a2e47596`) | — | 127 |
| f26 slice-7d pushed claim (`63417edd`: 516/521/523/524/525) | 5 | 122 |
| f26 slice-7e THIS claim (526/527/528/529/530) | 5 | **117 unclaimed** |

## The re-probe exclusions (zero net — all dedupe or no-arm)

* **d41 tip `4ed22f0b`** is STILL the CLAIM-only commit (the
  SetDefaults census). Mechanical grep over its `npc.rs`/`npc/data.rs`
  for all five ids: the ONLY hit is `npc.rs:24578`'s Mechanic shop
  test list — the shop ITEM id 530 (Pressure Plate), not an NPC arm.
  Zero FindFrame arms.
* **d42 tip `88df5809`** (NEW since the 7d census — it was unpushed
  then) is the WIP 578-587 enum checkpoint ("Does not compile", the
  108-union-plan note). Same grep: the shop-item 530 at `npc.rs:24636`
  ONLY. Zero FindFrame arms, no 526-530 kind.
* **f27 tip `0e187428`** (slice 9 + docs, ADVANCED past the 7d census's
  `72448094`): its `npc_findframe_extra8.rs`/`extra9.rs` carry NO
  526/527/528/529/530 arm (mechanical grep = no match). Slice 9 is the
  463/467/471/472/492 d40-stub FIX set — all inside the already-
  subtracted d40/f27 claim bands. Dedupes.
* **d40 tip `8fe1d6d3`** unchanged — the roster stops at 490, no
  `DesertGhoulCrimson`/`DesertGhoulHallow`/`DesertLamia*`/
  `DesertScorpionWalk` kind (grep = no match).
* **d43 tip `a725acb0`** IS the wave-3 base (no work).
* **d44-d49 / e13 / e14** greps for the five ids over their npc
  rosters/findframe modules = no kind / no arm. The d44 master
  wire-status table rows for the scorpion group's low-id co-members:
  164 WallCreeper `branch-wired(d46)`, 236 JungleCreeper + 239
  BloodCrawler `WIRED(g7)` (f27) — port-covered, NOT unclaimed; only
  530 was open in that group.
* **This branch's own `npc.rs`**: no arm for any of the five (the
  `npc.rs:24578` hit is the shop-item 530 again).

## This slice's claim (5 kinds, ONE commit)

526/527 DesertGhoulCrimson/Hallow — the 7d-deferred pair completing
the 524..527 group (`NPC.cs:69908-69945`, labels `:69910-69911`; the
machine copied VERBATIM from the 7d module, the 7c nine-salamander
ONE-group precedent). 528/529 DesertLamiaLight/Dark (`:69946-69977`
— the ghoul shape MINUS the lift-to-2, the wrap at row >= count − 1
back to 1, the air pins row count − 1 = 8). 530 DesertScorpionWalk
(the 164/236/239/530 group `:73083-73111` — the air counter-zero-FIRST
+ row 4 rising / 0 falling BREAK, the UNCONDITIONAL grounded sprite
copy, the |vx|*1.1 NO-+1 accumulator feeding the WHILE >= 6 multi-row
stepper with the STRICT > 3 wrap, NO idle branch). Module
`crates/tr-app/src/npc_findframe_extra7e.rs` + the
`find_frame_kind_extra7e` seam; 9 module witnesses + the Python
replay (`tools/_f26_7e_replay.py`) + the JS sweep twin
(`tools/_f26_7e_sweep.js`) WRITTEN NOT RUN (code-only); zero-RNG +
zero-world-read grep over both cited ranges = ZERO (the 7d flare
resolved-draw seams NOT needed — `FrameInput` is vx/vy only); frame
counts cross-checked `Main.cs:67082` vs `NPC_FRAME_COUNT` (526/527=8
`:67136` entries 7/8, 528/529=9 `:67136` entries 9/10, 530=5 `:67137`
entry 1; all 697 slots re-diffed = ZERO mismatches). Commit:
`9c2b0a84`.

The next ascending unclaimed ids after this slice: 531/532/534/535/
536 (the 531 DesertScorpionBlack joins the 165/237/238/240/531 group
`NPC.cs:73112-73145` — the 0.4-vs-0.5 accumulator variant; 532 rides
the 7c walking-antlion group `:70067-70103`; 533 DesertDjinn is
port-covered — the S1-E-5 `npc.rs` arm, anchors row 311; re-check at
the next census).
