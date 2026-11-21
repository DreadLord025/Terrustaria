# 110 — W4-d36 E4 continuation: the band adjudication, the writer race, and the aiStyle-5 tail port

Status: ADJUDICATION EXECUTED + ITEM-4 PORTED, WIRED (the concurrent
writer's batch-2 pass adopted this session's module) + the timed-row FIX
(this session's follow-up commit); the walker batches are the concurrent
writer's (see §2).

## 1. The Lead adjudication (E4-KINDS-CONTINUE item 2) — EXECUTED

The foreign stash `d36-inflight-4` (batch-1 WIP: Wolf/Vampire/Frankenstein/
BlackRecluse/SwampThing, npc.rs-only, never compiled) claimed "E4 band
767-771" in its comments. Under the d37-R7 re-pin (research/108 `a341daa12`,
authoritative; doc-109's supersede-note) **767-771 belongs to batch 0's five
landed kinds** (ToxicSludge 767 / Penguin 768 / PenguinBlack 769 /
GiantTortoise 770 / IceTortoise 771, tip `8c5839b03`). Ruling:

- **b0 owns 767-771** (landed, frozen).
- **The Wolf batch rides 772+**, ascending vanilla order, Vampire 159
  SKIPPED (g8-owned @762, verified §2). The definitive post-skip table
  (the landed rows bold-checked against the tips): Wolf 155 → **772**
  (`a11f61850`), Frankenstein 162 → **773** (`85ee24abb`),
  BlackRecluse 163 → **774**, SwampThing 166 → **775**, UndeadViking
  167 → **776** (the three `69bf20113`), CorruptPenguin 168 → 777,
  PigronCorruption 170 → 778, PigronHallow 171 → 779, (Crimera 173 → 781,
  MossHornet 176 → 782 — landed hover slots, 780 stays free),
  PigronCrimson 180 → 783,
  FaceMonster 181 → 784, SnowFlinx 185 → 785, ArmoredViking 197 → 786,
  Lihzahrd 198 → 787, LihzahrdCrawler 199 → 788, HeadacheSkeleton 201 → 789,
  MisassembledSkeleton 202 → 790, PantlessSkeleton 203 → 791,
  SpikedJungleSlime 204 → 792, (Moth 205 → 793 — landed), IcyMerman 206 →
  **794**. Band headroom: R7's 767-816 ceiling holds (794 ≤ 816); i16-safe.
- **The stash is PRESERVED, not landed** (`d36-inflight-4` intact; the
  colliding band claim is void, the stat rows/FindFrame arms are the
  re-port's source material). Wolf landed first at the adjudicated slot:
  tip `a11f61850` (the concurrent writer, §2) — enum + all 12 hard sites +
  the unicorn machine's OWN 155 arms + FindFrame 155 verbatim, pins
  279/279/190, bijection ALL PINS HOLD.

## 2. The writer race (this worktree, 2026-11-21 afternoon)

A concurrent E4 writer is executing the SAME continuation in this worktree
(the doc-109 §0 foreign-writer pattern, third recurrence):

- ~17:15 it landed Wolf@772 (`a11f61850`, pushed) — exactly the adjudicated
  slot/render-key/pins this note's session had derived independently.
- It swept this session's enum-only WIP (the 19 remaining variants, no data
  arms — never compiled) into stash **`d36-inflight-5`**, preserved.
- ~18:50 it landed Frankenstein@**773** (`85ee24abb`, pins 280/280/191) —
  SKIPPING Vampire 159 with the claim "159 Vampire is g8-owned @762,
  doc-250 s6.10". **VERIFIED by this session** against the pushed tip
  `origin/task/W4-g8-npc-kind-wire-1`: its `npc/data.rs` carries
  `NpcKind::Vampire => 762` (to_u16) + `762 => NpcKind::Vampire`
  (from_u16) + `cs_id 159`. So Vampire IS g8-owned (the E3 pushed-lane
  ownership rule) — the brief's 28-kind list predates that claim; the
  skip is CORRECT, not a divergence. The wave's true port count is 27
  (28 − Vampire), and Frankenstein 773 = the first free slot after the
  skip keeps the band ascending.
- It preserved THREE foreign batch-2 WIP snapshots verbatim as
  `tools/_d36_inflight_batch2{,b,c}.patch` (no stashes touched), NOT landed.
  **Patch c carries the concurrent writer's OWN item-4 port**: a
  `STINGER: u16 = 55` const + `proj_data` row in `projectile/data.rs`
  (byte-identical VALUES to this session's landed row, different comment
  placement) + a `wants_item_sounds` post-tick drain in
  `world_sim/npcs.rs` for the same Item17 cue.

Partition taken (the AGENTS.md "never work on the same task as another
agent" rule + the brief's preserve-don't-collide directive): **the walker
kinds and their hot files (`npc.rs`, `npc/data.rs`, the bestiary/assets/
buffs/netglue/portrait/entities sites) were the concurrent writer's**;
this session took the brief's item 4 (below) in the files that stayed
disjoint (`npc_ai_style5.rs` NEW, `projectile.rs`, `projectile/data.rs`,
`main.rs`), with an explicit disjoint `git add` list. The race then
RESOLVED itself: the batch-2 commit `69bf20113` absorbed this session's
item-4 hunks verbatim, adopted the module and wired it (the dedupe
below). The only follow-up this session lands is the timed-row FIX the
wiring commit's header claimed but missed (`world_sim/npcs.rs`, one
hunk — by then the writer's own item-4 hunk there was committed, so the
file is shared-history, not mid-flight).

### The item-4 DEDUPE — RESOLVED (the L-B gate precedent)

Two faithful item-4 ports were in flight: this session's
`npc_ai_style5.rs` + STINGER row/rider, and the concurrent writer's
PRESERVED patch-c (STINGER const/row + `wants_item_sounds` drain + their
npc.rs call sites). Resolution (the batch-2 commit `69bf20113`): the
concurrent writer ADOPTED this session's module (the doc-109 note: "the
more complete port — the Hornet 42/231-235 family + getGoodWorld third
draw + the integer-half anchor + the stealth gate") and wired it; their
patch-c stays preserved-not-landed, so:

1. `projectile/data.rs`: ONE `pub const STINGER` + ONE `STINGER =>` row
   (this session's; the `#[deny(unreachable_patterns)]` guard makes any
   future duplicate a hard compile error — the L-B gate's own shape).
2. The Item17 cue: ONE drain path (`wants_item_sounds` → the sim's
   post-tick drain). The placement deviation vs the C#'s mid-AI call
   position is recorded in the module doc + §3 — the gameplay RNG order
   is untouched (the façade pitch draw rides the audio stream only).
3. The volley call site: ONE `style5_tail` invocation per tick per kind,
   at the END of `AI_005` (the frozen A/B twin shares it).
4. The timed-shot list's STINGER row: MISSING from `69bf20113` despite
   its header claim — landed by this session's follow-up commit (§3).

## 3. Item 4 — the aiStyle-5 tail: PORTED + WIRED + the timed-row fix

The MossHornet stinger volley (`NPC.cs:50754-50810`) and the Moth wet arm
(`:50726-50738`) — the hover batch's documented remainders — are ported
AND live:

- **`npc_ai_style5.rs`** (this session): `style5_tail` — the C#'s two tail
  blocks verbatim (the Moth's own wet buoyancy ×0.95/−0.5/−4-clamp +
  TargetClosest; the 42/176/231-235 group's wet arm + the 101-latch Item17
  cue + the charge rolls `Next(5,20)*0.1*scale` (ONE for the family, a
  SECOND for 176, a THIRD for getGoodWorld — the exact draw order) + the
  still-player suppression + the 130 fire gate (flag2 =
  `AI_GlobalFiringDistanceCheck`, `Collision.CanHit`) + the ±20 aim jitter
  + the rolling-side gate + the ×8 normalize + the `(int)(30f*scale)` 176
  / `(int)(10f*scale)` family damage + the 101 latch/netUpdate). The
  integer-half-height spawn anchor (`(float)(base.height / 2)`, `:50783`)
  is carried. 7 witnesses WRITTEN NOT RUN (the mirrored-oracle
  stream-position method pins every draw count/order).
- **The wiring** (the concurrent writer's batch-2 commit `69bf20113`,
  which ABSORBED this session's module/doc/anchors/projectile hunks
  verbatim and completed the seam): `ai_005_hover_family` carries the
  `rng` thread and calls `style5_tail` after the kind match; the shot
  drains to `wants_hostile_shot`, the Item17 cue to `wants_item_sounds` →
  the sim's post-tick drain. The frozen A/B twin calls the tail too (the
  axis-vs-kind-dispatch equivalence holds — the Hornet 42's charge rolls
  are NEW behaviour both paths share, the doc-109 batch-2 note §3).
- **The dedupe RESOLVED**: the concurrent writer chose THIS module over
  their preserved patch-c approach (the doc-109 note: "the more complete
  port"); patch c stays preserved-not-landed, its duplicate STINGER
  const/row never landed (the `#[deny(unreachable_patterns)]` guard would
  have made it a hard error). ONE drain path exists (`wants_item_sounds`).
- **The follow-up fix (this session's second commit)**: the batch-2
  wiring's header CLAIMED the sim timed-list's STINGER row but the hunk
  was MISSING from `69bf20113` — the wired shot would have lived the
  stock 3600 (a 12× lifetime deviation vs the C#'s `timeLeft = 300`,
  `NPC.cs:50797`). The fix lands the one-line
  `|| kind == crate::projectile::STINGER` row (the QUEEN_BEE_STINGER
  shape) + the module-doc reconciliation (the UNWIRED contract text →
  the LANDED wiring record).
- Recorded deviations (module doc + this note):
  * the port's `hover_step` integrates inside the AI (a one-tick lag on
    the wet buoyancy's velocity writes only);
  * `Collision.CanHit` folds to the centre-only `can_hit_line` probe
    (the `npc.rs:10065` convention);
  * the Item17 cue drains POST-TICK (`wants_item_sounds`, the W-10
    unicorn-tails placement) instead of the C#'s mid-AI call position
    (`:50756`) — the cue's façade pitch draw moves past the tick's later
    drains; the gameplay RNG order is untouched;
  * the poison-on-hit (`Projectile.cs:12261-12271`, the 55/719 `Next(3)`
    Poisoned roll) is the player-debuff layer's unported hit-time draw
    (the QUEEN_BEE_STINGER precedent — the 719 twin ships without it too).

## 4. Pins

This session's slice adds NO kinds: the roster pins are the concurrent
writer's batch-2 landing (**283/283/194** at `69bf20113`, max id 793,
ENTRIES 254 numeric-net rows, D36_SHEETS 13 — the doc-109 batch-2 note).
The bijection script (`tools/_d36_bijection.py`) at this note's commit:
ALL PINS HOLD (283/283, d36 13/28, max id 793, i16-safe). The `proj_data`
duplicate-arm guard (`#[deny(unreachable_patterns)]`) covers the 55 row;
the `s1_d_4c_every_rowed_style_has_exactly_one_live_arm` coverage pin
holds (aiStyle 1 was already armed; the rider gate is kind-scoped, not a
new style arm). Kind 55 is script-verified disjoint from every other
aiStyle-1 gate list (SLICE_A/B/C, ARROW_KINDS, the special bullets, the
frost/moonlord/darts/thrown sets) and from every `proj_data` range arm.

## 5. Kinds remaining (the E4 28-kind wave, at this note's commit)

Landed on the branch (13): b0's five (141/148/149/153/154 @767-771), the
hover trio (173/176/205 @781/782/793), Wolf 155@772, Frankenstein 162@773,
BlackRecluse 163@774, SwampThing 166@775, UndeadViking 167@776.
**Vampire 159: g8-OWNED** (verified §2 — their pushed tip's `Vampire =>
762`), skipped on this branch; the wave ports 27, not the brief's 28.
Remaining unclaimed (**14**): CorruptPenguin 168, the pigron trio
170/171/180, FaceMonster 181, SnowFlinx 185, ArmoredViking 197, Lihzahrd
198, LihzahrdCrawler 199, the bone trio 201/202/203, SpikedJungleSlime
204, IcyMerman 206. The concurrent writer's in-flight batch may claim
more — check the tip before taking rows (the AGENTS.md branch-claim
rule). The next slots ride the §1 definitive table (CorruptPenguin 168
→ 777, PigronCorruption 170 → 778, PigronHallow 171 → 779, 780 free,
the hover 781/782 fixed, PigronCrimson 180 → 783, …, IcyMerman 206 →
**794**).
