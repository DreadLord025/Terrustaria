# 281 - FindFrame census addendum after f26 slice 7h (was key 68; renumbered per registry-150 §3/§6.3/§7.3, KEEPS §9.1: 7c->201 7e->203 7f->205 7g->251 7h->281 7i->282 7j->283; executed at the plan-B3 merge of task/W4-f26-findframe-extra-3)

> RENUMBER # 68 addendum — FindFrame census after f26 slice 7h (code-only)

Audit date: 2026-11-21. Branch `task/W4-f26-findframe-extra-3`, slice
7h, base tip `899896d4` (slice 7g DONE + docs). **Code-only mode: no
cargo/rustc/tests/graphify invoked** — every claim is git/file-read
evidence against the PUSHED remote tips, re-probed at census time.

The master wire-census lives at `docs/research/200-findframe-wire-status.md`
on `task/W4-d44-findframe-low-1` (tip `c863a7a5` — census refresh 3,
UNCHANGED at this census); its §6 carries the mechanically-derived
**130-kind UNCLAIMED worklist**. This addendum rides the f26 branch so
the two lanes never text-conflict. The merger folds the count below
into the master §1.

## The window re-probe (at the CURRENT tips)

The brief's pool: 55 (the 7g addendum post-state). The re-probe at
THIS slice's census time (`git fetch origin --prune` then per-tip
`git rev-parse` + mechanical grep):

* **f27 tip `1a24a133`** UNCHANGED since the 7g census — the
  slice-10 claim band (531..545 + the 549 fix) stays as counted; its
  `npc_findframe_extra10.rs` carries the 538/539 squirrel arms whose
  machine this slice copies VERBATIM for the gem twins (`:76458-76512`
  group). No NEW claim band above 545 (no extra11 module; the
  kind-name grep hits are `creative_sacrifices.rs` data rows only).
* **d42 tip `2d7ac08c`** UNCHANGED — the 38 LANDED arms stay as
  counted. **c4 `0df6964b` / e26 `048cca6c`** UNCHANGED.
* **c8 tip ADVANCED `3705670d` → `708522fc`** — the five new commits
  are the crawler finals (the sight-provider cross-lane contract, the
  d22 compose transplant, the sheets census, the final census + the
  110-tick saturation pin fix). The 606 Maggot crawler arm is
  UNCHANGED — the 8-id inline subtraction stands, no count change.
* **g10 tip `5dee2865`** (NEW this census — "lane 3 batch 2: land +
  wire TownSlimeCopper 684, the last IsTownPet member"): its
  `npc_findframe_pet.rs` carries `town_pet_find_frame` — the
  humanoid-group town-pet arm (`NPC.cs:68622-68633` case list → the
  shared body with the pet overrides). Its `npc.rs::find_frame`
  routes exactly **11 kinds**: TownCat/TownDog/TownBunny +
  TownSlimeBlue/Green/Old/Purple/Rainbow/Red/Yellow/Copper =
  **637/638/656/670/678/679/680/681/682/683/684** — 11 of the
  master's 130, NOW claimed (the 7g next-window note's "637/638 the
  town cat/dog" is superseded). **687 BoundTownSlimeYellow is NOT in
  the arm's match** — it stays unclaimed (its own case `NPC.cs:76620`,
  not the pet group).
* **f26's own pushed 7g claims** (435/436/631/632/635, tip
  `899896d4`) = 5 more off the master's stale rows.
* **d41 tip `5d036721`** UNCHANGED (the salamanders — dedupe inside
  our own 7c claims); mechanical grep over its tree for this slice's
  eight ids + kind names (`HallowBoss`/`GemSquirrel*`) = ZERO arms
  (the `creative_sacrifices.rs` rows only). **d40 `8fe1d6d3`** stops
  at 490; **d43 `a725acb0`** IS the wave-3 base.
* **g6 `bf99e983` / g7 `6ca3e668` / d44-d49 / e13 / e14 / g8 / g9 /
  f17 / f18**: the mechanical `\b(636|639|64[0-5]) =>` scan over
  every pushed `npc.rs` + findframe module + `npc/*.rs` = ZERO; the
  kind-name scan's ONLY hits are the `creative_sacrifices.rs`
  sacrifice-count data rows (present on every branch — not arms).
  The c8 `npc.rs:4435` dodge comment explicitly notes the Empress's
  kind is UNPORTED ("out with its unported kind").
* **This branch's own `npc.rs`**: no kind/arm for any of the eight.

## The census count (the d44 refresh-3 base)

| step | subtraction | remaining |
| --- | --- | --- |
| the d44 refresh-3 §6 UNCLAIMED worklist (`c863a7a5`) | — | 130 |
| the f26 7d/7e/7f pushed claims (the refresh-3 stale rows) | 15 | 115 |
| the f27 slice-10 NEW claims (537-545; the 549 fix changes no count) | 9 | 106 |
| the d42 LANDED arms (the 38-id list, tip `2d7ac08c` unchanged) | 38 | 68 |
| the c4/c8/e26 INLINE arms (581/587/592/607/615/606/604/605 — c8 advanced but the arm unchanged) | 8 | 60 |
| the **g10 LANDED town-pet arm** (`5dee2865`: 637/638/656/670/678-684) | 11 | 49 |
| the f26 slice-7g pushed claims (435/436/631/632/635) | 5 | 44 |
| f26 slice-7h THIS claim (636/639-645) | 8 | **36 unclaimed** |

The 7g post-state 55 − 11 (g10) − 8 (this slice) = 36 — the same
ladder from the other end. The 36-id post-slice worklist, enumerated
mechanically (the master's 130 minus every subtraction above):
**634** (1), **646-655** (10 — 656 is g10-claimed, OUT of the run),
**657-662** (6), **664** (1), **667** (1), **669** (1),
**671-677** (7 — 670 is g10-claimed), **687-689** (3),
**691-696** (6) = 1+10+6+1+1+1+7+3+6 = **36**. The 44-id pre-slice
worklist is the same set plus this slice's 636/639-645 (8). (The
master §6's
contiguous-run shorthand "634-662 (29) / 669-684 (16)" splits at the
g10 claims: 656 out of the first run, 670/678-684 out of the second;
this slice took 636/639-645 out of the first run's head.)

## This slice's claim (8 kinds, ONE commit `89cef0b4`)

636 HallowBoss — its own case (`NPC.cs:67022-67024`): the one-line
phase pin `frame.Y = AI_120_HallowBoss_IsInPhase2() ? num : 0` on the
2-row sheet (`Main.cs:67147` entry 6). The phase read
(`NPC.cs:45814-45821`) is PURE `ai[3]`: `== 1f || == 3f` (2 is the
`IsGenuinelyEnraged` slot, NOT phase 2) — it rides
`FrameInput::ai3`, no world probe (the extra-7b resolved-input
precedent; the wiring bridge copies `ai[3]`). The counter AND
sprite UNTOUCHED. 639-645 the seven gem squirrels — the
`case 299/538/539/639-645` group (`NPC.cs:76458-76512`, labels
`:76461-76467`): the VERBATIM f27 slice-10 squirrel machine copy
(the 7g ONE-group precedent — the wiring merge keeps one): the
grounded ±1 flip, the idle row-0 zero, the `|vx| * 0.5` THEN `++`
accumulator with the STRICT >6 step, the `>= count - 1` wrap back to
row **1** (rows 1..4 walkable on the 6-row sheet, row 5 never), NO
lift-to-start-row (a row-0 start walks 0 → 1 — the 7e lamia shape),
the air row 2 rising / row 4 falling both counter-zeroed with the
sprite UNTOUCHED. The aiStyle-3 backpedal branch (`:76478-76483`) is
DEAD: all seven aiStyle 7 (`NPC.cs:17226-17236`, the
`type >= 639 && type <= 645` SetDefaults block) — the documented
dead branch, the f27 538/539 precedent (`:15877`/`:15889`).

Excluded from the window with cite: **634 SporeBat** — its flutter
arm draws `Main.rand.Next(10)` (`NPC.cs:74844`), the zero-RNG rule
keeps it unclaimed for the d44f seam-fed wave. **Its RNG-seam
contract is PUBLISHED in the module header**
(`npc_findframe_extra7h.rs`): the bridge draws `Next(10)` EVERY tick
for type 634 ONLY (the `&&` short-circuits for the floater group's
other members 182/210/211/253/316/662/664 — zero draws; the draw
rides `UpdateNPC`'s AI → FindFrame order, `:90423` → `:90511`), feeds
the RESOLVED `spore_dust_draw_is_zero` boolean (the 7d SolarFlare
resolved-draw precedent), the `Dust.NewDust` 165 spawn + the
`velocity *= 0.1f` + `noGravity` writes (`:74846-74848`) a
bridge-side effect keyed on the same boolean. The post-seam frame
machine is zero-RNG: the vx-SIGN sprite gates (`:74850-74857` — NOT
direction, the 7c antlion shape), `rotation = vx * 0.1f`
(`:74858`), the bare `++frameCounter` (`:74864` — the 210/211 second
++ does NOT apply), the >= 6 step (`:74875-74879` — the type-48
`num291 = 5` override does NOT apply), the wrap at
`num292 = count − 1` (`:74867-74870` — the 49/51/60/634 decrement
DOES apply: rows 0..3 flap on the 5-row sheet, row 4 never;
`Main.cs:67147` entry 4 = 5).

Module `crates/tr-app/src/npc_findframe_extra7h.rs` + the
`find_frame_kind_extra7h` seam; 7 module witnesses + the Python
replay (`tools/_f26_7h_replay.py`) + the JS sweep twin
(`tools/_f26_7h_sweep.js`) WRITTEN NOT RUN (code-only); zero-RNG +
zero-world-read grep over the three cited ranges (each ≤ 80 lines:
`:67022-67024`, `:45814-45821`, `:76468-76512`) = ZERO; frame counts
cross-checked `Main.cs:67082` vs `NPC_FRAME_COUNT` (636 = 2 `:67147`
entry 6, 639-645 = 6 `:67148` entries 1-7; all 697 slots re-diffed =
ZERO mismatches, a mechanical text-parse diff).

The next ascending unclaimed ids after this slice: **634** (the
RNG-drawing SporeBat — the seam contract above), then **646-652 the
gem bunnies** (the `case 46/47/303/337/443/464/540/614/646-652` hop
group, `NPC.cs:76851-76904` — the f27 540 twin: the `|vx| + 1` >6
walk over the FULL 7 rows with the >= count wrap to 0, the air rows
4 rising / 6 falling; all seven aiStyle 7 `:17238-17248`), 653
HellButterfly / 661 EmpressButterfly (the flap group), 654 Lavafly,
655 MagmaSnail, 657 QueenSlimeBoss, 658-660 the Queen Slime minions,
662 PirateGhost / 664 TorchGod (the floater group — the 634 seam
contract's siblings), 667 GoldenSlime / 676 ShimmerSlime, 669
Stinkbug, 671-675 the birds, 677 Shimmerfly, 687-689, 691-696 —
re-check at the next census (the f27/d42/g10 lanes are moving).
