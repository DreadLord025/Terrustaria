# 251 - FindFrame census addendum after f26 slice 7g (was key 68; renumbered per registry-150 §3/§6.3/§7.3, KEEPS §9.1: 7c->201 7e->203 7f->205 7g->251 7h->281 7i->282 7j->283; executed at the plan-B3 merge of task/W4-f26-findframe-extra-3)

> RENUMBER # 68 addendum — FindFrame census after f26 slice 7g (code-only)

Audit date: 2026-11-21. Branch `task/W4-f26-findframe-extra-3`, slice
7g, base tip `7f38fd34` (slice 7f DONE + docs). **Code-only mode: no
cargo/rustc/tests/graphify invoked** — every claim is git/file-read
evidence against the PUSHED remote tips, re-probed at census time.

The master wire-census lives at `docs/research/200-findframe-wire-status.md`
on `task/W4-d44-findframe-low-1` (tip `c863a7a5` — census refresh 3);
its §6 carries the mechanically-derived **130-kind UNCLAIMED
worklist**. This addendum rides the f26 branch so the two lanes never
text-conflict. The merger folds the count below into the master §1.

## The window re-probe (the brief's 537-541 window is CLOSED)

The brief named 537/538/539/540 + the next unclaimed (541?). The
re-probe at THIS slice's census time:

* **f27 tip `1a24a133`** (slice 10 + docs, ADVANCED past the 7f
  census's `0e187428`): `git show …:docs/ANALYSIS.md | findstr
  /C:"slice 10"` = **the claim band NOW EXISTS** — "claims the 14
  unclaimed ids ascending from 531: 531/532/534/535/536/537/538/539/
  540/541/542/543/544/545 plus 549 the lane-portal FIX". ALL 15 arms
  are LIVE in its `npc_findframe_extra10.rs` (the batch-2/3
  `_ => unimplemented!` catch-all is GONE — the squirrel/bunny/
  elemental/shark/portal arms all present, mechanical grep). The
  531-536 six dedupe against this branch's own pushed 7f claims;
  **537-545 are f27's 9 NEW claims**; 549 is a FIX of its own kind
  (no claim-count change); 533 DesertDjinn stays port-covered
  EXCLUDED (the S1-E-5 `npc.rs:8086` arm); 546/547/548 have NO case
  label. The brief's whole window is f27-claimed — this slice takes
  the NEXT truly-unclaimed ids instead.
* **d42 tip `2d7ac08c`** (ADVANCED past `9b22c091` — the roster now
  LANDED with find_frame arms): its `npc.rs` find_frame region
  (mechanical kind scan) covers **38 of the master's 130**:
  578/580/582-586/590/591/593-603/608-614/616-620/624-629. Its own
  roster comment (`npc.rs:1094-1095`) defers 581/587 to c4
  (style-44), 592/607/615 to c4b (style-16), 604/605 to e26b (luck),
  606 to c8 (style-66).
* **c4 tip `0df6964b`** carries the INLINE style-44 arm for 581/587
  (`npc.rs:6854-6869`) + the style-16 fish arm for 592/607/615
  (`:8391-8393`); **c8 tip `3705670d`** the crawler-group arm for
  606 (`:7121`); **e26 tip `048cca6c`** the ladybug arm for 604/605
  (`:6756-6766`) — all eight port-covered on their pushed tips.
* **d41 tip `5d036721`** (ADVANCED past the CLAIM-only `4ed22f0b` —
  batch 2, the nine salamanders 498-506): mechanical grep over its
  `npc.rs` for this slice's five ids + kind names
  (`ArmedZombieTwiggy`/`ArmedZombieCenx`/`RockGolem`/`MaggotZombie`/
  `SporeSkeleton`) = ZERO (the salamanders dedupe against this
  branch's own 7c claims).
* **d40 tip `8fe1d6d3`** unchanged — the roster stops at 490.
  **d43 tip `a725acb0`** IS the wave-3 base (no work).
* **g6 `bf99e983` / g7 `6ca3e668` / d44-d49 / e13 / e14**: the
  mechanical `\b<id> =>` scan over every pushed findframe module's
  seam = ZERO for all five ids. The d49 high2 module's own witness
  pins `find_frame_kind_high2(435) == None` ("the next wave owns
  435+"); its ANALYSIS defers "435/436/591 ride the next wave" —
  591 has since landed on d42, 435/436 are THIS slice's.
* **This branch's own `npc.rs`**: no kind/arm for any of the five.

## The census count (the d44 refresh-3 base)

| step | subtraction | remaining |
| --- | --- | --- |
| the d44 refresh-3 §6 UNCLAIMED worklist (`c863a7a5`) | — | 130 |
| the f26 7d/7e/7f pushed claims (the refresh-3 stale rows: 516/521/523-530/531/532/534-536) | 15 | 115 |
| the f27 slice-10 NEW claims (537-545; the 549 fix changes no count) | 9 | 106 |
| the d42 LANDED arms (the 38-id list above) | 38 | 68 |
| the c4/c8/e26 INLINE arms (581/587/592/607/615/606/604/605) | 8 | 60 |
| f26 slice-7g THIS claim (435/436/631/632/635) | 5 | **55 unclaimed** |

The 7f post-state **112** was the 7c-method compatible count (the
602 − 248 − 75 − … ladder); the d44 refresh-2/3 master re-derivation
(273 INLINE-ARM + 199 module-claimed + 130 UNCLAIMED = 602) is the
new base — the two reconcile: the master's 130 already absorbs the
d49-deferred 435/436/591 tail (+3 over the 7c 127) and re-counts the
roster-lane stubs as INLINE-ARM coverage. The brief's "112 − 5 =
107" arithmetic is superseded by the mechanical re-probe: the f27
band + the d42/c4/c8/e26 landings subtract 62 more between the two
censuses.

## This slice's claim (5 kinds, ONE commit `969144ca`)

435 ArmedZombieTwiggy / 436 ArmedZombieCenx — the d49-deferred pair
of the `case 430..436/591` armed-zombie group (`NPC.cs:70879-70951`,
labels `:70884-70885`): the VERBATIM d49 high2 machine copy (the 7e
ghoul ONE-group precedent) — the `ai[2] > 0` attack ladder rows
3/4/5/6 off the <5/<10/<15 bands (counter + sprite UNTOUCHED), then
the wave-A zombie walk (the grounded ±1 flip, the air/turn row-2
pin, the stand row-0 zero, the |vx| NO-+1 accumulator, the
<8/<16/<24/<32 bands rows 0/1/2/1, the lap counter-zero row-HOLD —
the 7f bride quirk). 631 RockGolem — its own case
(`:75259-75313`): the UNCONDITIONAL grounded sprite copy (direction
0 rides verbatim), the `ai[2]` windup pose zone (`0 < ai[2] < 100`,
`amount = (ai[2] - 68) / 32` NOT clamped) — the facing+can-hit
branch `(int)Lerp(15, count, amount)` rows, the overlap branch
`(int)Lerp(10, 15, amount)`, the else row 0, the counter UNTOUCHED;
the walk `0.2 + |vx|` STRICT >8 step over rows 1..7 with the
LITERAL `num31 = 8` cap (NOT the 21-row count — rows 8..20 are the
pose zone's, never walkable) + the row-<1 lift; the air zeroes
BOTH. The pose zone's world reads (`:75270-75273` —
`Main.player[target]`, the hitbox intersect, `Collision.CanHit`)
ride the `FrameInput` seam as three RESOLVED booleans (the extra-7b
slope/frame_x36 precedent — the bridge owns the probe). 632
MaggotZombie / 635 SporeSkeleton — the humanoid-group case list
(`:68619`/`:68621`) → the shared fighter walk with the 632-ONLY
`num108 = 12` stride override (`:69524-69527` — the 7f
tax-collector twin) vs 635's group default >6; `num57 = 0` (both
aiStyle 3, `:17138`/`:17179`, no townNPC) — the FULL-16 wrap;
`num109` = row 2; idle row 0 (not the 140/489 pair); air row 1
(neither in the `:69647` pin list); the |vx|*2 THEN ++ group
accumulator. The ai[0] pose zones (`:68662-69488`, the
`Main.rand` draws at `:68944`/`:68951`/`:68953`/`:68959`/`:69003`/
`:69007`/`:69412`/`:69436`) are the documented skip (the 7f
precedent): AI_003 writes `ai[0]` ONLY 0/1 for these kinds (the
stand-turn `:57385-57396`; the 1000+ confused writes are
379/380/111-gated), so no zone condition can hold at FindFrame
time (`UpdateNPC` order `:90423` AI → `:90511` FindFrame).

Excluded from the window with cite: **634 SporeBat** — its flutter
arm draws `Main.rand.Next(10)` (`NPC.cs:74844`), the zero-RNG rule
excludes it from THIS wave (it stays unclaimed for a seam-fed
wave); **633 BestiaryGirl** — port-covered (`npc.rs:Zoologist`, the
d44 master row). Module `crates/tr-app/src/npc_findframe_extra7g.rs`
+ the `find_frame_kind_extra7g` seam; 11 module witnesses + the
Python replay (`tools/_f26_7g_replay.py`) + the JS sweep twin
(`tools/_f26_7g_sweep.js`) WRITTEN NOT RUN (code-only); zero-RNG +
zero-world-read grep over the five cited ranges (each ≤ 80 lines:
`:70887-70951`, `:75259-75313`, `:68647-68661`, `:69515-69585`,
`:69642-69655`) = ZERO RNG, the ONLY world reads the golem pose
zone's four lines (seam-fed); frame counts cross-checked
`Main.cs:67082` vs `NPC_FRAME_COUNT` (435/436 = 7 `:67127` entries
5/6, 631 = 21 `:67147` entry 1, 632 = 16 entry 2, 635 = 16 entry 5;
all 697 slots re-diffed = ZERO mismatches, a mechanical text-parse
diff).

The next ascending unclaimed ids after this slice: **634** (the
RNG-drawing SporeBat — needs the dust seam), then 636 HallowBoss
(the one-line `AI_120_HallowBoss_IsInPhase2` row pin,
`NPC.cs:67022-67024`), 637/638 the town cat/dog (the humanoid
group's pet rows — the `:69491-69509` idle wiggle + the `:69651`
air row-8 pin), 639-645 the gem squirrels (the f27 squirrel-group
twins), 646-652 the gem bunnies — re-check at the next census.
