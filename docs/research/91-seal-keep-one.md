# 91 — SEAL: the corrected keep-one (g4 vs d22 vs f35) + the fighter-family bible

W4-g4 branch `task/W4-g4-npc-style3-fighter`, SEAL phase (DONE base
`69d7737b`, this seal on the SAME branch). Code-only lane: no
cargo/tests/rustfmt/graphify invoked; witnesses stay WRITTEN NOT RUN.
This note RE-ADJUDICATES the d22c mutual-receipt against the g4 delta
that landed AFTER the d22c probe, and publishes the corrected keep-one
for the d22d-seal/merger.

## 0. The stale-probe timeline (why the re-adjudication was needed)

| commit | lane | time (2026-11-21 +0200) | what |
| --- | --- | --- | --- |
| `a9369ec38` | g4 | 08:09 | the d22r RESUME keep-both fold-list (docs) |
| `b72cdfbfc` | f35 | 08:19 | **f35 ports the heal seam** (`npc_death_heals.rs`, the superset) |
| `00a5b27db` | f35 | 08:43 | f35 wires `spawn_heals` on the death path |
| `55840d9a0` | g4 | 09:16 | g4 FINAL code: the DUPLICATE heal seam + the spawn_coins LIVE wiring + the scan arms |
| `78386ab14` | d22 | 12:39 | R3 deprecation: probes g4 at `a9369ec38` — "NO delta" (**STALE**: g4's origin tip was `55840d9a` by then) |
| `a1a21235f` | d22 | 12:40 | R4 FINAL: "merge d22 only, never g4-after" |
| `69d7737ba` | g4 | 12:56 | g4 FINAL docs (91-F): claims the heal seam as g4-unique |

Two probes were stale in opposite directions: d22's R3 missed g4's
`55840d9a`; g4's 91-F §1 read f35's census `:39` disclaimer as current
when it sits in a **"Port status BEFORE this task"** table
(`65-expert-drop-trees-census.md`, section header at `:27`). f35's own
PORT_MAP row at its tip `1abae9ade` marks the heal seam **done**
(`npc_death_heals.rs` + `world_sim/npcs.rs::spawn_heals`, "W4-f35
widened scope"). The Lead's seal premise ("g4-DropHeals-unique-KEEP")
is therefore **INVERTED by the evidence** — see §1.

## 1. The heal-seam re-adjudication: f35 KEEPS, g4 RETIRES (duplicate)

**Both lanes ported `NPCLoot_DropHeals` (`NPC.cs:79098-79195`) +
`NPCLoot_DropCommonLifeAndMana` (`:79531-79557`).** f35's port is:

* **EARLIER** — `b72cdfbfc` (08:19) + the `00a5b27db` wiring (08:43),
  both before g4's `55840d9a` (09:16).
* **A SUPERSET** — `npc_death_heals.rs` also carries
  `DoDeathEvents_DropBossPotionsAndHearts` (`:78945-79016`, the boss
  potion chain + the 5-9 hearts), the EoC/WoF KilledToday **Badger's
  Hat 5004** latch-and-reset (the World fields + the `clock.rs` dusk
  reset, `Main.cs:65221`), and the EoW head's dynamic boss latch
  (`DropEoWLoot`, `:78414-78429`). g4's copy has NONE of these.
* **ANCHORED + MAPPED** — f35's `anchors.txt` row
  (`npc_death_heals.rs|pub fn drop_boss_potions_and_hearts`) + the
  PORT_MAP heals row + 10 witnesses (incl. the 33-kind x 5-seed x
  both-expert same-seed stream-equality replays).
* **THE SAME CORE CENSUS** — read side-by-side against the C#: the num
  latch (16/121/81), the mana-first `Next(2)` + the else-if second
  `Next(2)`, the trailing `RollLuck(2)`, the luck-rolls-before-stat-
  gates order, the whole per-type switch (305-314/329/330 `RollLuck(4)`,
  326 `RollLuck(6)`, 315 always, 341 `Next(5,11)`, 338-340
  `RollLuck(5)`, 342 `Next(3)!=0`, 325/327/344/345/346 `Next(6)+6`,
  267 `Next(2)`+life, 13-15 `Next(4)`+life, 116-119
  `!expert||Next(5)==0`, 139 `Next(2)`), the seasonal remap before the
  velocity triplet — f35's `drop_common_life_and_mana`/`drop_heals`
  match g4's helpers line-for-line in draw order. Both wire AFTER
  `spawn_loot` on the `UpdateNPCs` stream (the recorded coins-before-
  loot fold is shared).

**Verdict: keep f35's `spawn_heals`; RETIRE g4's `spawn_heal_drops`.**
Merging both would DOUBLE-FEED the death stream (every kill draws the
common block twice). This seal commit removes g4's duplicate (the fn +
the two helpers + the death-path call + `HealScriptRng` + the 8 seam
witnesses) and leaves a tombstone comment at the old site
(`world_sim/npcs.rs:7088`, the `91-seal-keep-one.md` marker at `:7097`)
+ a repointed
anchors row. The g4 C# census survives as the audit trail
(`91-final-fighter-census.md` §1, corrected below).

**Consequence for the d22 instruction:** d22's R4 "merge d22 only,
never g4-after" no longer drops any unique heal work — after this
seal, g4's heal code is GONE by g4's own hand. The feared
"g4-DropHeals dropped by the d22-only merge" resolves as
"f35-DropHeals KEPT by the f35 merge".

## 2. The g4 delta `55840d9a` adjudicated hunk-by-hunk (the corrected keep-one)

| hunk | verdict | evidence / merge route |
| --- | --- | --- |
| `spawn_heal_drops` + `drop_common_life_and_mana` + `drop_heals_type_switch` + the death-path call + the 8 seam witnesses | **RETIRED on g4 (this seal)** — f35 owns the seam (§1) | f35 `npc_death_heals.rs` @ `b72cdfbfc` + `spawn_heals` @ `00a5b27db`; tombstone `world_sim/npcs.rs:7088` |
| **`spawn_coins` midas/blood-moon LIVE inputs** (`npc.buffs.has_buff(72)` `:79631` + `app.world.blood_moon` `:79660`) | **g4-UNIQUE — MUST SURVIVE** | d22 tip `world_sim/npcs.rs:7218-7219` still `false, false`; f35 tip `:6954-6955` still `false, false`. Neither lane carries the wiring. **Merge route: a targeted 2-argument hunk + the witness `w4_g4_spawn_coins_midas_and_blood_moon_inputs_are_live` (g4 `npcs.rs:8116`) + the anchors row, cherry-applied AFTER d22+f35** (both `spawn_coins` bodies are otherwise identical to g4's — the hunk applies cleanly to either) |
| the scan type-bonus arms (GoblinThief 27 +1, AngryBones 31/294-296 +6, Butcher 460 always-open) | **DROP g4 side — d22 already carries the SAME code** | d22 `npc_ai_fighter_scan.rs:498-513` (`t == 27` +1 `:498`, `matches!(t, 31\|294\|295\|296)` +6 `:500`, `t == 460` flag28 `:512`) — g4's "restore" was against its own pre-union zombie-only scan; the d22 axis scan never lost them |
| the two scan witnesses (`w4_g4_the_goblin_thief_bash_adds_one` g4 `:652`, `w4_g4_the_skeleton_family_bash_bonus_census` g4 `:678`) | **g4-UNIQUE — cherry-pick as witness-only** | d22 has NO Thief/Skeleton bonus witness (grep at its tip). Both call `fighter_obstacle_scan(npc, world, rng)` — the UNION signature d22 keeps (`npc_ai_fighter_scan.rs:271`), so they compile against the d22 tree unchanged |
| the GoblinPeon 26 smash in the drain (`break_tile` seam) | **DROP g4 side — KEEP d22's** | d22 `world_sim/npcs.rs:4890` uses `tr_core::edit::kill_tile` + `kill_tile_sound` + the drop — closer to the C# `WorldGen.KillTile(num198, num199 - 1)` (`:60049-60056`); g4's `editing::break_tile` rides the PICK path (the `CanKillTile` refusals the C# KillTile does NOT run). d22's form is the faithful one |
| the Peon-smash witness (`w4_g4_the_goblin_peon_smashes_the_door` g4 `:17797`) | **re-derive at the merge gate** | it pins g4's `break_tile` behaviour; against d22's `kill_tile` form the refusal-path assertions must be re-read (written-not-run either way) |
| the 91-F census doc + the FINAL ANALYSIS receipt + the 3 anchors | **audit trail on g4** (d22 R3's docs-only cherry-pick offer stands) | `91-final-fighter-census.md` §1 carries the seal correction; the `fn spawn_heal_drops` anchor row is repointed to the tombstone marker BY THIS LANE (a deliberate retire, not a lossy merge — the canary row still passes) |

**The corrected merger instruction (for d22d-seal / the Lead):**

1. Merge **d22** (`a1a21235f`) — canonical for EVERY fighter AI arm
   (the R3 KEEP table stands unchanged).
2. Merge **f35** (`1abae9ade`) — owns the heal seam + the bag/crate/
   present openers. The death-path seam conflict (`spawn_heals` call
   site vs d22's `npcs.rs`) resolves to f35's call AFTER `spawn_loot`.
3. **Never merge g4 wholesale.** After this seal the g4 branch's ONLY
   unique behaviour is the `spawn_coins` LIVE-inputs hunk (+ its
   witness + anchors row) and the two scan witnesses. Cherry-apply
   those two hunks; optionally cherry-pick the g4-unique docs
   (`91-doc-sweep-fighter.md`, `91-addendum-d22r-fold-list.md`,
   `91-final-fighter-census.md`, this note) as the audit trail.
4. Run the written-not-run witnesses at the merge gate (d22 R4's
   instruction stands); re-derive the Peon-smash witness against
   d22's `kill_tile` form.
5. The Lead retires the g4 branch post-verify (d22 R3/R4's rule — this
   lane does not delete its own branch mid-flight).

## 3. MartianWalker 520: the shooter remainder's lane = d41 (NOT d43, NOT a generic projectile lane)

The 91-F census §3 recorded the ONE genuine fighter remainder: the
MartianWalker 520 rides `update_skeleton` (the walk IS routed) but the
C# 520 is a SHOOTER-BLOCK type (`:59069` list). Re-verified C# params
(read at this seal):

* the shot cooldown `num162 = 15` (`:59109`), the speed `num164 = 8`
  (`:59253`), **`num170 = 592` / `num169 = 35`** (`:59442-59445`) —
  the 592 laser projectile;
* the walk `num180 = 4f / num181 = 1f / num182 = 0.7f`
  (`:59735-59739`);
* the **400-px stop**: `type == 520 && Distance < 400f &&
  CanHitLine(...)` → `flag22 = true; ai[3] = 0f` (`:59763-59766`);
* the **aim cycle**: `localAI[2]++`, at `>= 6` reset +
  `localAI[3] = player.DirectionFrom(Top + (0, 20)).ToRotation()`
  (`:59792-59799`).

**Lane ownership:** 520 sits in the **491-560 band = d41**
(`task/W4-d41-npc-kinds-491-560`, claim `4ed22f0b3`). d43 covers
631-697 — NOT 520. The 91-F "projectile-work lane" phrasing is
CORRECTED: d41's own precedent ports a kind's shooter machine AND its
projectile in-lane — batch 2 (`5d036721d`) ported `update_salamander`
(the AI_003 shooter block's 498-506 terms) **plus** the SalamanderSpit
572 `proj_data` row. The 520 shooter-block (the 592 laser + the aim
cycle + the 400-px stop) is the same shape: **a d41 task seed**, with
the C# params above as its work order. The kind itself is already
landed (wave-3 base: `npc.rs:113` + `bestiary.rs:1353`); only the
offence is missing. The engineer's turret-spawn (`:59004-59038`) +
the zapper's spear (`:59062-59065`) stay with the projectile-work lane
(their kinds 386/389 are outside the d34-d43 bands' unported sets).

## 4. The f35 disclaimer receipt (the Lead's item 3) — CONFIRMED STALE, ownership INVERTED

The Lead asked to confirm "f35-lane reads g4-owns" from the
`f35-e0e5b53d` census disclaimer. Receipt:

* At `e0e5b53d` the disclaimer was TRUE — that commit predates f35's
  heal port (log order: `e0e5b53d` → `0fd244e79` → `b72cdfbfc` → ...).
* f35 then **widened its own scope** (`5bf66b89d` "the widened scope":
  the census tables + the PORT_MAP heals row + the
  `drop_boss_potions_and_hearts` anchor) and ported the seam.
* At f35's tip `1abae9ade` the disclaimer row is a historical BEFORE
  snapshot; the live status is the PORT_MAP **done** row.
* **f35 does NOT read g4-owns — f35 owns it itself.** g4's 91-F §1
  claim ("the seam is g4's to port") was based on the stale row and is
  corrected by this seal (§1). No cross-lane handoff is needed; the
  keep-one is f35.

## 5. 91-F SEALED + the fighter-family bible

**91-F is SEALED**: `91-final-fighter-census.md` stays the fighter
census of record with its §1 ownership claim corrected by this note
(the C# draw census itself was verified line-by-line and is UNCHANGED
— both ports implement it identically). INDEX carries the 91-S row.

**The fighter-family bible (the merge-gate one-liner set):**

| lane | owns | tip |
| --- | --- | --- |
| **d22** | ALL fighter AI arms (the axis + the zombie fold + the shooters + P1-P6/P9 + the Peon smash + the moan head) — CANONICAL | `a1a21235f` |
| **f35** | the death heal seam (`npc_death_heals.rs` + `spawn_heals`, the superset) + the bag/crate/present openers | `1abae9ade` |
| **g4** | the `spawn_coins` LIVE-inputs hunk + the Thief/Skeleton scan witnesses + the audit-trail docs; the duplicate heal port RETIRED by this seal | this seal commit |
| **g8** | the ≤230 kind wires + the style-14 shots — COMPOSES with d22 (disjoint kind sets, `82c93cec` in its ancestry) | `e1d9578a4` |
| **g9** | the ten kind-gated rows on the same machines — COMPOSES (disjoint) | `5d08fdf29` |
| **g10** | the town-pet wires (637/638/656/670/678-684) — COMPOSES (disjoint, no style-3 overlap) | `5dee28650` |
| **d41** | the 491-560 kind band + **the 520 shooter-block task seed** (§3) | `727578f2d` |
| **d34-d43** | the R2 123-roster-gap table + the `fighter_walker_kind` stream obligation (d22 R2/R4) | per-lane |

**Verification.** Code-only: every claim read from `git show` at the
named tips (d22 `a1a21235f`, f35 `1abae9ade`, d41 `727578f2d`, g8/g9/g10
tips) + the C# re-read (`NPC.cs:79098-79195`/`:79531-79557` the heal
seam, `:59109`/`:59253`/`:59442-59445`/`:59735-59739`/`:59763-59766`/
`:59792-59799` the 520 params, `Item.cs:49649-49772` the NewItem remap
+ prefix order). The retirement edits are comment-tombstone + code
removal only; no behaviour change on the g4 branch beyond removing the
duplicate feed. Witnesses WRITTEN NOT RUN.