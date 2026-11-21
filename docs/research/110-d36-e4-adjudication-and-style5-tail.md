# 110 — W4-d36 E4 continuation: the band adjudication, the writer race, and the aiStyle-5 tail port

Status: ADJUDICATION EXECUTED + ITEM-4 PORT (this note's session); the walker
batches are the concurrent writer's (see §2).

## 1. The Lead adjudication (E4-KINDS-CONTINUE item 2) — EXECUTED

The foreign stash `d36-inflight-4` (batch-1 WIP: Wolf/Vampire/Frankenstein/
BlackRecluse/SwampThing, npc.rs-only, never compiled) claimed "E4 band
767-771" in its comments. Under the d37-R7 re-pin (research/108 `a341daa12`,
authoritative; doc-109's supersede-note) **767-771 belongs to batch 0's five
landed kinds** (ToxicSludge 767 / Penguin 768 / PenguinBlack 769 /
GiantTortoise 770 / IceTortoise 771, tip `8c5839b03`). Ruling:

- **b0 owns 767-771** (landed, frozen).
- **The Wolf batch rides 772+**, ascending vanilla order: Wolf 155 → **772**,
  Vampire 159 → 773, Frankenstein 162 → 774, BlackRecluse 163 → 775,
  SwampThing 166 → 776, UndeadViking 167 → 777, CorruptPenguin 168 → 778,
  PigronCorruption 170 → 779, PigronHallow 171 → 780, (Crimera 173 → 781,
  MossHornet 176 → 782 — landed hover slots), PigronCrimson 180 → 783,
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
  doc-250 s6.10". **Divergence flagged for the Lead**: this session's brief
  (E4-KINDS-CONTINUE) lists Vampire 159 in the 28-kind port set at 773;
  the concurrent writer's doc-250 citation re-assigns it to g8. The two
  authority chains (the brief's R7 ascending table vs their doc-250 s6.10)
  need one ruling; this note does NOT silently rewrite either. Their
  slot arithmetic (Frankenstein 773 = the first free slot after the skip)
  keeps the band ascending either way.
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
buffs/netglue/portrait/entities sites, and — discovered mid-flight —
`world_sim/npcs.rs`, where they carry an uncommitted item-4 hunk) are the
concurrent writer's**; this session took the brief's item 4 (below) in
the files that stayed disjoint (`npc_ai_style5.rs` NEW, `projectile.rs`,
`projectile/data.rs`, `main.rs`). No shared-file hunk of this session is
staged — the `git add` list is explicit and disjoint. The remaining-kind
census at this note's commit: Wolf + Frankenstein landed (10/28 on the
branch), Vampire's ownership contested (above), the concurrent writer's
in-flight batch pending.

### The item-4 DEDUPE contract (GATE DEDUPE L-B precedent)

Two faithful item-4 ports are in flight: this session's LANDED
`npc_ai_style5.rs` + STINGER row/rider, and the concurrent writer's
PRESERVED patch-c (STINGER const/row + `wants_item_sounds` drain + their
npc.rs call sites). Whoever lands second MUST dedupe, not append:

1. `projectile/data.rs`: ONE `pub const STINGER` + ONE `STINGER =>` row
   (the values are identical; the `#[deny(unreachable_patterns)]` guard
   makes a duplicate a hard compile error — the L-B gate's own shape).
2. The Item17 cue: ONE drain path. The C# plays it MID-AI (`:50756`,
   before the charge rolls) — this module's `out.sounds` call-position
   drain is the faithful placement; the patch-c `wants_item_sounds`
   post-tick drain moves the cue's stream position past the same tick's
   later draws (the W-10 unicorn-tails deviation shape). Reconcile to
   the call-position drain, or record the post-tick placement as the
   deliberate deviation in ANALYSIS — do NOT keep both.
3. The volley call site: ONE `style5_tail` (or equivalent) invocation
   per tick per kind, at the END of `AI_005` — the module doc's contract.

## 3. Item 4 — the aiStyle-5 tail: PORTED (module) + live projectile infra

The MossHornet stinger volley (`NPC.cs:50754-50810`) and the Moth wet arm
(`:50726-50738`) — the hover batch's documented remainders — are ported:

- **`npc_ai_style5.rs`** (NEW): `style5_tail` — the C#'s two tail blocks
  verbatim (the Moth's own wet buoyancy ×0.95/−0.5/−4-clamp + TargetClosest;
  the 42/176/231-235 group's wet arm + the 101-latch Item17 cue + the
  charge rolls `Next(5,20)*0.1*scale` (ONE for the family, a SECOND for
  176, a THIRD for getGoodWorld — the exact draw order) + the still-player
  suppression + the 130 fire gate (flag2 = `AI_GlobalFiringDistanceCheck`,
  `Collision.CanHit`) + the ±20 aim jitter + the rolling-side gate + the
  ×8 normalize + the `(int)(30f*scale)` 176 / `(int)(10f*scale)` family
  damage + the 101 latch/netUpdate). The integer-half-height spawn anchor
  (`(float)(base.height / 2)`, `:50783`) is carried. 7 witnesses WRITTEN
  NOT RUN (the mirrored-oracle stream-position method pins every draw
  count/order).
- **Live infrastructure** (the wiring's data half, landed):
  `projectile::STINGER` (55) — the `proj_data` row
  (`Projectile.cs:1375-1383`: 10x10, aiStyle 1, penetrate −1, tileCollide,
  stock 3600 life); `AI_STYLE_1_RIDERS` membership (the `flag3 = false`
  skip list `Projectile.cs:55954` — resting counter, straight flight, the
  BULLET_SNOWMAN precedent). The atlas packs `Projectile_55` through
  `spawnable_kinds()` automatically (the row-derived list);
  `proj_frames(55)` = 1 (no `Main.projFrames[55]` override exists).
  The sim's timed-shot list needs ONE line at the wiring step
  (`|| kind == crate::projectile::STINGER`, the QUEEN_BEE_STINGER row's
  shape) for the C#'s `timeLeft = 300` override (`NPC.cs:50797`) —
  `world_sim/npcs.rs` carried the concurrent writer's UNCOMMITTED item-4
  hunk (a `wants_item_sounds` drain for the same Item17 cue) at this
  commit, so this session did NOT stage that file (the explicit-add
  disjointness rule; without the line the shot lives the stock 3600 —
  the module doc pins it). NOTE for the Lead: the concurrent writer is
  ALSO porting item 4 via the queue seam — the two approaches must be
  reconciled at the wiring step (this module's `out.sounds` vs their
  `wants_item_sounds` post-tick drain; the C# plays the cue mid-AI, so
  this module's call-position drain is the faithful placement).
- **UNWIRED seam** (the `npc_ai_eye.rs`/`npc_findframe_b.rs` convention):
  the one remaining edit is `npc.rs::ai_005_hover_family` — thread `rng`
  (the `ai_009_sphere_family` signature precedent) and call `style5_tail`
  after the Hornet/MossHornet/Moth `update_hover` arms, draining
  `out.shot` → `wants_hostile_shot` and `out.sounds` → the style75
  `out.sounds` drain shape. The FULL contract (env fields, drain code,
  deviations) is the module doc. The seam file is the concurrent writer's
  (§2) — the wiring lands with their batch's completion or the next
  session, NOT colliding mid-batch.
- Recorded deviations (module doc): the port's `hover_step` integrates
  inside the AI (a one-tick lag on the wet buoyancy's velocity writes
  only); `Collision.CanHit` folds to the centre-only `can_hit_line` probe
  (the `npc.rs:10065` convention); the poison-on-hit
  (`Projectile.cs:12261-12271`, the 55/719 `Next(3)` Poisoned roll) is the
  player-debuff layer's unported hit-time draw (the QUEEN_BEE_STINGER
  precedent — the 719 twin ships without it too).

## 4. Pins

This session's slice adds NO kinds: the roster pins stay the concurrent
writer's (279/279/190 at `a11f61850`; their in-flight five will move them).
The bijection script (`tools/_d36_bijection.py`) walks the working tree —
run at this note's commit it reads THEIR mid-batch state; the committed
HEAD pins hold (the script's §1-4 checks are roster-only, untouched by
this slice). The `proj_data` duplicate-arm guard
(`#[deny(unreachable_patterns)]`) covers the new 55 row; the
`s1_d_4c_every_rowed_style_has_exactly_one_live_arm` coverage pin holds
(aiStyle 1 was already armed; the rider gate is kind-scoped, not a new
style arm).
