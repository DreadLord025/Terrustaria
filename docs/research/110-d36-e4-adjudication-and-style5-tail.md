# 110 — W4-d36 E4 continuation: the band adjudication, the writer race, and the aiStyle-5 tail port

Status: ADJUDICATION EXECUTED + ITEM-4 PORTED, WIRED (the concurrent
writer's batch-2 pass adopted this session's module) + the timed-row FIX
(this session's follow-up commit); the walker batches are the concurrent
writer's (see §2). **BAND COMPLETE (§6): batch 5 landed the last five
kinds with the Lead v640 ID correction (795/796), bijection ALL PINS
HOLD — roster 297, d36 27/27 (Vampire 159 g8-owned@762 excluded).**

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
  MisassembledSkeleton 202 → 790, PantlessSkeleton 203 → ~~791~~ **795**,
  SpikedJungleSlime 204 → ~~792~~ **796**, (Moth 205 → 793 — landed), IcyMerman 206 →
  **794**. Band headroom: R7's 767-816 ceiling holds (794 ≤ 816); i16-safe.
  **ID CORRECTION (Lead v640, see §6): the 791/792 assignments above are
  VOID — d37 reserves 791 → Crab / 792 → DungeonGuardian, both already in
  main via d34. The landed slots are 795 (PantlessSkeleton) / 796
  (SpikedJungleSlime); max id 796 ≤ 816.**
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

Landed on the branch (22): b0's five (141/148/149/153/154 @767-771), the
hover trio (173/176/205 @781/782/793), Wolf 155@772, Frankenstein 162@773,
BlackRecluse 163@774, SwampThing 166@775, UndeadViking 167@776, E4
batch 3's five (CorruptPenguin 168@777, PigronCorruption 170@778,
PigronHallow 171@779, PigronCrimson 180@783, FaceMonster 181@784), and E4
batch 4's four (SnowFlinx 185@785, ArmoredViking 197@786, Lihzahrd 198@787,
LihzahrdCrawler 199@788 — all on `update_skeleton`/the ladder rows (185/198
num83 base 1.5, 197 cap-2, 199 the OWN cap-4 arm `NPC.cs:57486`); the
SnowFlinx air-roll `:58820-58831` + the strike latch `:81364-81367` + the
strong-hit kb ×1.5 `:81449-81452` (`knock_back_npc`'s new type row); the
Lihzahrd 55%-life Transform(199) `:57027-57030` (the `wants_transform`
drain's 199 arm, the style75 one-tick fold); FindFrame humanoid-group
`:68575`/`:68577`/`:68578` + case 199's two-band clock `:75030-75068`;
bestiary 185/197 own entries + the Lihzahrd pair's ONE credit (199
Hide=true `NPCID.cs:2577-2582`, `ContentSamples.cs:1052`
`creditIds[198] = creditIds[199]` — the entry emits at net 198 keyed on
"LihzahrdCrawler", the BlackRecluse 238 precedent; the 199 DRAW sheet
stays its own via `texture_net_id`, the cs_id hand row keeps 199 so the
ladder arm stays live, s1_f_3's fold exemption covers the pair);
immunities 185 {44,324} / 197 {20,44,324} / 198+199 {20}
(`NPCID.cs:7203`/`:7227`/`:7234`/`:7241`); cues NPCHit1/2/26 +
NPCDeath1/2/29 (the bank rows added); sheets 202/D36_SHEETS 22, render_key
568-571, pins 292/292/292, bijection ALL PINS HOLD (scripted, max id 793);
4 witnesses WRITTEN NOT RUN). The Lihzahrd's solar-form remainder is the
Transform ITSELF (landed) — the crawler form IS 199, ported in full; the
`IdleSounds` 1000-roll (`:90103-90106`, style Next(36,38)) + the hit/death
dust/gores (`:86807-86824`/`:88551-88558`/`:89674-89689`) + the 199
flag8-false row (`:57072`) / wide ahead-probe (`:59953`, the zombie-scoped
fighter scan) are the documented remainders.
**Vampire 159: g8-OWNED** (verified §2 — their pushed tip's `Vampire =>
762`), skipped on this branch; the wave ports 27, not the brief's 28.
Remaining unclaimed (**5**): the bone trio 201/202/203, SpikedJungleSlime
204, IcyMerman 206. The concurrent writer's in-flight batch may claim
more — check the tip before taking rows (the AGENTS.md branch-claim
rule). The next slots ride the §1 definitive table (HeadacheSkeleton 201
→ 789, …, IcyMerman 206 → **794**).

## 6. The band-COMPLETE receipt (E4 batch 5, the SEAL — 2026-11-22)

The LAST five kinds are landed on this branch and the d36 band is
**SEALED**: `python tools/_d36_bijection.py` → **ALL PINS HOLD — roster
297, d36 27/28, max id 796** (297/297/297 to/from/enum, ALL_KINDS
297/297/297, s1_f_3 297, all 12 hard exhaustive sites + bestiary
entry/fold coverage PASS). The 28th slot is **Vampire 159, g8-OWNED
@762** (verified §2 against `origin/task/W4-g8-npc-kind-wire-1`'s pushed
`Vampire => 388` cs row / 762 protocol row) — excluded by design, so the
wave's port count is **27/27**, not the brief's 28.

### The ID CORRECTION (Lead v640) — §1's 791/792 rows are VOID

The d37 tables reserve protocol **791 → Crab** and **792 →
DungeonGuardian**, and both kinds are **ALREADY IN MAIN via d34**. The
§1 definitive table's old `PantlessSkeleton 203 → 791` /
`SpikedJungleSlime 204 → 792` assignments are therefore VOID; batch 5
uses the corrected slots (verified free pre-edit: no `to_u16` arm
carried 795/796, and 791/792 stay unclaimed on this branch —
`from_u16(791) == None`, `from_u16(792) == None` pinned in the
witnesses):

| kind | vanilla | protocol | note |
| --- | --- | --- | --- |
| HeadacheSkeleton | 201 | **789** | §1 slot, unchanged |
| MisassembledSkeleton | 202 | **790** | §1 slot, unchanged |
| PantlessSkeleton | 203 | **795** | CORRECTED (791 = d37 Crab) |
| SpikedJungleSlime | 204 | **796** | CORRECTED (792 = d37 DungeonGuardian) |
| IcyMerman | 206 | **794** | §1 slot, unchanged |

`render_key` is a separate hash space: the five take the next free
injective slots **572-576** (571 was batch 4's max; scripted-verified
each appears exactly once).

### The five kinds (SetDefaults VERBATIM + the wiring)

- **The bone trio 201/202/203** (`NPC.cs:11572-11584`/`:11585-11597`/
  `:11598-11610`): 18x40 aiStyle 3; dmg 20/18/22, def 12/8/8, HP
  55/65/60, kb 0.5, value 130/120/110, NPCHit2/NPCDeath2. All ride
  `update_skeleton` on the ladder's EXISTING num83 rows — the OWN caps
  **1.1/0.9/1.2** (`:57625-57636`, already in `npc_ai3_speeds`) + the
  `:57693` `1 + (1 - scale)` factor (1 at the stock scale). FindFrame's
  humanoid-group case list (`:68579-68581`): 201/202 the 15-row strips
  with the air row **0** (`:69647` lists 201/202), 203 the 16-row strip
  with the air row **1** (the list names 201/202, **NOT 203** — the
  Lead's row-0 correction). All three ARE in the `:57340` 1000-roll
  moan list (the ambient gate admits them; the moan arms were already
  live) and in the `:59996` flag24 door-bash list (the zombie-scoped
  fighter scan's remainder keeps the bash unported for them, the
  batch-4 note). Immunities {20} (`NPCID.cs:7249`/`:7256`/`:7263`).
  Death gores 265/267+43/44, 268+43/44, 269+43/266
  (`:88569-88596`) — the gore/dust wave's remainder.
- **SpikedJungleSlime 204** (`NPC.cs:11611-11624`): 24x18 aiStyle 1,
  dmg 28, def 8, HP 65, **scale 1.15**, **alpha 50**, value 300,
  NPCHit1/NPCDeath1, NO kb row (the head's 1f default `:8533`). Rides
  `update_slime` (the wave-A routing) + the wave-A FindFrame bridge —
  the slime-group case list (`:70810`) over the 2-row strip
  (`FindFrameKind::SpikedJungleSlime` added to all four module sites +
  the count pin). The scale rides `Npc::scale` (the ToxicSludge
  precedent) and the draw's `scaled_anchor`; the alpha rides the ctor
  row. The `AI_001` shooter block (`:61386-61439`: flag3-true + the
  80-tick expert five-spike fan / the 65-tick classic lob, the 176
  JungleSpike at dmg 13) is the pure `npc_ai_slime` module's census —
  the flag3 row keeps the machine whole, the shots land with the
  module's wiring swap (the 184 precedent). The **176 JungleSpike**
  `proj_data` row IS landed (`Projectile.cs:2621-2629`: 6x6 aiStyle 1
  penetrate −1 alpha 255) + its `ai_arrow` fade hook
  (`:55109-55128`: alpha −50, the first-tick ai[1] latch + the Item17
  cue on `wants_sound`; the dust 22 skipped). Immunity {20}
  (`NPCID.cs:7270`). Hit/death dust 4 in the (149,208,104) tint
  (`:85010-85026`) — the gore/dust wave's remainder.
- **IcyMerman 206** (`NPC.cs:11640-11653`): 18x40 aiStyle 3, dmg 60,
  def 30, HP 280, kb 0.5, value 500, NPCHit1/NPCDeath1, coldDamage (no
  port slot field — the IceTortoise note). Rides the aiStyle-3 shooter
  block on `update_martian_shooter` (the pirate/martian precedent):
  the **50-tick** stop timer (`:59131-59134`), the **EXACT-half fire**
  (`:59208` `ai[1] == num163`, 25 — kind-gated so the older kinds keep
  their window shape), the **7-speed 177 IcewaterSpit at dmg 37**
  (`:59219-59222`/`:59408-59411`/`:59462-59465`), the **centre −10
  chaser anchor** (`:59284-59287`, both the aim's +10 − |dx|×0.1 lead
  drop and the spawn Y), the ±40 integer jitter with NO rescale
  (`:59392-59396`), the `ai[2] <= 0` + grounded stop-entry gate
  (`:59634`) and the default **1/0.07/0.8** walk (`:59717-59719`; the
  ladder declines 206 via `shooter_excluded`, `:58322`). FindFrame
  case 206 (`:76060-76098`) over the **20-row** strip: the grounded
  facing flip, the ai[2] attack-row hold, the walk band rows 6..19
  (|vx|×3 clock, >6 advance, wrap at the strip's end back to 6), the
  air row 0. The **177 IcewaterSpit** `proj_data` row IS landed
  (`Projectile.cs:2630-2640`: 10x10 aiStyle 28 penetrate −1 alpha 255,
  coldDamage no slot field — the ai_frost_bolt flight). Immunity
  {44,324} (`NPCID.cs:7284`, NO 20); in NEITHER AI_003 moan list (the
  walker gate excludes it, the shooter precedent). The flag8-false row
  (`:57072`) rides the zombie-scoped fighter-scan remainder; the
  hit/death dust 137 + gores 273-275 (`:86828-86847`) the gore/dust
  wave's; the 177's Chilled 1500 / Frozen Next(30,120) rolls
  (`Projectile.cs:12366-12373`) the player-debuff layer's unported
  hit-time draws (the QUEEN_BEE_STINGER precedent).

### The surfaces (all scripted-verified)

Bestiary: five OWN entries (no Hide row — `NPCID.cs:2592-2618` carries
Velocity=1f only for 201/202/203/206, 204 has no draw-modifier row; no
`ContentSamples` fold), the bone trio all display "Skeleton" (en-US
NPCs.json); populator refs Caverns 201/202/203
(`BestiaryDatabaseNPCsPopulator.cs:1469-1471`), UndergroundJungle 204
(`:1132`), UndergroundSnow 206 (`:1529`). ENTRIES 270 → **275** unique
credits. Sheets: NPC_201/202/203/204/206 in STATIC_SHEET_NAMES,
BESTIARY_NPC_SHEET_IDS 202 → **207** == the parallel vec order
(scripted), D36_SHEETS 22 → **27**. Cues: NPCHit2/NPCDeath2 (trio) +
NPCHit1/NPCDeath1 (204/206) — all four banks already ship (no new
audio rows). netglue: the trio + 206 the cave-walker seed, 204 the
wave-A surface seed. ALL_KINDS + s1_f_3 **292 → 297**; `update_kind_ai`
exactly-one-arm per kind (297, no dups, scripted). 5 witnesses WRITTEN
NOT RUN (`w4_d36_batch5_{headache_skeleton,misassembled_skeleton,
pantless_skeleton,spiked_jungle_slime,icy_merman}_*`).

### The 27-kind d36 roster (the sealed band)

| # | kind | vanilla | protocol | batch |
| --- | --- | --- | --- | --- |
| 1 | ToxicSludge | 141 | 767 | b0 |
| 2 | Penguin | 148 | 768 | b0 |
| 3 | PenguinBlack | 149 | 769 | b0 |
| 4 | GiantTortoise | 153 | 770 | b0 |
| 5 | IceTortoise | 154 | 771 | b0 |
| 6 | Wolf | 155 | 772 | E4 |
| 7 | Frankenstein | 162 | 773 | E4b |
| 8 | BlackRecluse | 163 | 774 | E4 batch 2 |
| 9 | SwampThing | 166 | 775 | E4 batch 2 |
| 10 | UndeadViking | 167 | 776 | E4 batch 2 |
| 11 | CorruptPenguin | 168 | 777 | E4 batch 3 |
| 12 | PigronCorruption | 170 | 778 | E4 batch 3 |
| 13 | PigronHallow | 171 | 779 | E4 batch 3 |
| 14 | Crimera | 173 | 781 | hover |
| 15 | MossHornet | 176 | 782 | hover |
| 16 | PigronCrimson | 180 | 783 | E4 batch 3 |
| 17 | FaceMonster | 181 | 784 | E4 batch 3 |
| 18 | SnowFlinx | 185 | 785 | E4 batch 4 |
| 19 | ArmoredViking | 197 | 786 | E4 batch 4 |
| 20 | Lihzahrd | 198 | 787 | E4 batch 4 |
| 21 | LihzahrdCrawler | 199 | 788 | E4 batch 4 |
| 22 | HeadacheSkeleton | 201 | 789 | E4 batch 5 |
| 23 | MisassembledSkeleton | 202 | 790 | E4 batch 5 |
| 24 | PantlessSkeleton | 203 | **795** | E4 batch 5 (corrected) |
| 25 | SpikedJungleSlime | 204 | **796** | E4 batch 5 (corrected) |
| 26 | Moth | 205 | 793 | hover |
| 27 | IcyMerman | 206 | 794 | E4 batch 5 |

Excluded: **Vampire 159 → 762 (g8-owned)**. Free in the band: 780 (the
batch-3 gap), 791/792 (d37-reserved, in main via d34). Max id 796 ≤
R7's 816 ceiling; i16-safe. **The d36 band is COMPLETE — 27/27.**
