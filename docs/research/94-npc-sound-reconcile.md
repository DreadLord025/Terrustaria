# 94 — NPC-sound dup-reconcile: W4-e9 (`3793fef8`) vs W4-f2 (`9de95697`)

**Phase:** reconcile (code-only, no cargo/tests/graphify; witnesses WRITTEN
NOT RUN). Both branches sit on the same base `a725acb0` (main). e9 =
`origin/task/W4-e9-npc-sounds-2` tip `3793fef8` (7 commits, 938 insertions,
6 files); f2 = `origin/task/W4-f2-npc-sounds-3` tip `9de95697` (5 commits,
1179 insertions, 9 files). The two branches independently ported the SAME
40 sites — `NPC.IdleSounds`' 32-arm ladder (`NPC.cs:90079-90206`) +
`Collision_WaterCollision`'s 8 case-19 splash sites (`:93121`/`:93138`/
`:93154`/`:93171` entry + `:93219`/`:93236`/`:93252`/`:93269` exit) — plus
e9's two extras (VileSpit `:21480`, Ghost `:24693`). This note is the
arm-by-arm fidelity compare against the C# windows (each re-read ≤100
lines), the KEEP/DROP union decision, and the doc-sweep ports.

## 1. IdleSounds — arm-by-arm (all 32 arms, both branches vs C#)

C# shape (`NPC.cs:90075-90208`): the `!(shimmerTransparency > 0f)` head
gate `:90077` wraps ONE if/else-if ladder; each arm is
`type-test && Main.rand.Next(N) == 0`, playing
`PlaySound(29, (int)position.X, (int)position.Y, style)` — the int overload
(`SoundEngine.cs:102-108`, NO NextFloat), the style argument
(`Next(lo, hi)`) evaluated INSIDE the winning branch AFTER the gate roll.
A matching type consumes exactly ONE gate roll even on failure; no later
arm rolls for that type; an unlisted type draws NOTHING.

| # | C# lines | type gate | gate N | style | e9 `idle_sound_cue` | f2 `idle_sounds` |
|---|----------|-----------|--------|-------|--------------------|------------------|
| 1 | :90079-90082 | 239/240 | 900 | Next(24,26) | MATCH | MATCH |
| 2 | :90083-90086 | 62/156 | 700 | Next(26,30) | MATCH | MATCH |
| 3 | :90087-90090 | 177 | 600 | Next(30,32) | MATCH | MATCH |
| 4 | :90091-90094 | 226 | 800 | 32 pinned | MATCH | MATCH |
| 5 | :90095-90098 | 153 | 1000 | 33 pinned | MATCH | MATCH |
| 6 | :90099-90102 | wet && 63/64/103 | 1000 | Next(34,36) | MATCH (wet term precedes roll) | MATCH (wet term inside condition) |
| 7 | :90103-90106 | 198/199 | 1000 | Next(36,38) | MATCH | MATCH |
| 8 | :90107-90110 | 170/180/171 | 600 | Next(38,41) | MATCH | MATCH |
| 9 | :90111-90114 | 250 | 700 | Next(41,44) | MATCH | MATCH |
| 10 | :90115-90118 | 580 | 800 | 44 pinned | MATCH | MATCH |
| 11 | :90119-90122 | 582 | 800 | 44 pinned | MATCH | MATCH |
| 12 | :90123-90126 | 581 | 800 | Next(44,47) | MATCH | MATCH |
| 13 | :90127-90130 | 508 | 800 | 44 pinned | MATCH | MATCH |
| 14 | :90131-90134 | 509 | 800 | Next(44,47) | MATCH | MATCH |
| 15 | :90135-90138 | 494 | 600 | 47 pinned | MATCH | MATCH |
| 16 | :90139-90142 | 467 | 700 | Next(48,50) | MATCH | MATCH |
| 17 | :90143-90146 | 468 | 700 | Next(50,53) | MATCH | MATCH |
| 18 | :90147-90150 | 288 | 400 | Next(53,55) | MATCH | MATCH |
| 19 | :90151-90154 | 524-527 | 700 | Next(55,57) | MATCH (`(524..=527).contains`) | MATCH (four-way or) |
| 20 | :90155-90158 | 496/497 | 600 | Next(57,59) | MATCH | MATCH |
| 21 | :90159-90162 | 389 | 800 | Next(59,61) | MATCH | MATCH |
| 22 | :90163-90166 | 471 | 600 | Next(61,63) | MATCH | MATCH |
| 23 | :90167-90170 | 482 | 600 | Next(63,66) | MATCH | MATCH |
| 24 | :90171-90174 | 388 | 700 | Next(66,69) | MATCH | MATCH |
| 25 | :90175-90178 | 520 | 800 | Next(69,73) | MATCH | MATCH |
| 26 | :90179-90182 | 477 | 600 | 73 pinned | MATCH | MATCH |
| 27 | :90183-90186 | 258 | 700 | Next(74,78) | MATCH | MATCH |
| 28 | :90187-90190 | 252 | 700 | 78 pinned | MATCH | MATCH |
| 29 | :90191-90194 | 469 | 700 | Next(79,81) | MATCH | MATCH |
| 30 | :90195-90198 | 253/82 | 700 | Next(81,84) | MATCH | MATCH |
| 31 | :90199-90202 | 498-506 | 600 | Next(84,86) | MATCH | MATCH |
| 32 | :90203-90206 | 391 | 700 | Next(86,87) | MATCH (one-wide window still draws) | MATCH (one-slot draw) |

**Verdict: both ladders are verbatim-faithful on all 32 arms** — gate
windows, style ranges, chain order, the shimmer head gate, the arm-6 wet
term (a dry 63/64/103 draws NOTHING and falls out — no later arm names
those types), and the Scutlix one-wide `Next(86,87)` draw.

Structural differences (behaviour-equivalent, not fidelity gaps):

* **API shape.** e9 `idle_sound_cue(npc_type, wet, shimmer, roll) ->
  Option<i32>` returns the style; its `roll_range` helper draws
  `lo + roll(hi - lo)` — exactly one `Sample()`, equivalent to
  `Next(lo, hi)` (`UnifiedRandom.cs:120-141`). f2 `idle_sounds(npc_type,
  wet, shimmer, rng, play)` calls `rng.next_range(lo, hi)` directly and
  invokes the `play` callback inside the winning arm. Both consume the
  identical draw sequence gate→style→(play's pitch) on the caller's
  stream; f2's shape matches the module's existing emitters
  (`fighter_ambient_moans`) and needs no adapter closure at the wiring.
* **Positions.** Both wire `(n.x, n.y)` — the C# plays at
  `(int)base.position.X/Y` (NOT Center); `play_fighter_moan` truncates.
  MATCH both.
* **Type key.** Both key `kind.cs_id()` (e9's tip commit `3793fef8` is
  exactly this comment — the MERGER-L7 protocol-space precedent; f2's
  wiring already used `cs_id`). MATCH both.
* **Dead guard.** e9's wiring wraps the call in `if npcs[i].hp > 0` — the
  C#'s `life <= 0` early return (`NPC.cs:90385-90392`) precedes
  `IdleSounds()` (`:90422`). f2's original wiring had NO guard (a
  mid-pass-killed NPC could roll at its own slot turn before the death
  retain). **e9 wins this point; the guard is adopted into f2 this
  phase** (see §6).
* **Bank rows.** e9 registers `Zombie_24-86` (63 rows) in `SFX_CUES` +
  `zombie_asset`; f2 registers `Zombie_24-87` (64 rows). The C# case-29
  volume band is `24..=87` (`LegacySoundPlayer.cs:1009-1011`); slot 87 is
  unreachable from THIS ladder (`Next(86,87)` pins 86) but the band
  includes it and the full switch was ported "for the later waves'
  callers". **f2's superset wins** (e9's rows are a strict subset).

## 2. Collision_WaterCollision — site-by-site + gate compare

C# shape (`NPC.cs:93052-93275`, called from `UpdateCollision` `:92526`
AFTER `AI()`): never-wet head `:93055` (690@ai0==0 / 72 / style21 /
style67 / 376 / 579 / 541 / style7@ai0==25 → flag=false, wetCount=0,
lava=false); WetCollision + honey/shimmer latches `:93063-93075`;
aiStyle-116 wetCount pin `:93077-93080`; entry edge `:93087`
(`flag && !wet && wetCount==0` → wetCount=10 THEN the per-liquid arms);
exit edge `:93177` (`!flag && wet` → velocity.X*=0.5, wet=false, the 620
jump, THEN `wetCount==0` → wetCount=10 + arms reading the OLD latches);
the tail `:92527-92536` (dry → clear the three latches; wetCount>0 →
decrement).

| site | C# | style | e9 `water_collision_splash` | f2 `water_collision_sounds` |
|------|----|-------|------------------------------|------------------------------|
| entry shimmer | :93121 | 4 | MATCH | MATCH |
| entry honey | :93138 | 1 (default) | MATCH | MATCH |
| entry water | :93154 | 0 | MATCH | MATCH |
| entry lava | :93171 | 1 (default) | MATCH | MATCH |
| exit shimmer | :93219 | 5 | MATCH | MATCH |
| exit honey | :93236 | 1 | MATCH | MATCH |
| exit water | :93252 | 0 | MATCH (style) | MATCH (style) |
| exit lava | :93269 | 1 | MATCH (style) | MATCH (style) |

Gate-by-gate (the C# carries EIGHT textually distinct gates; the 300/59
order swap at `:93234` vs `:93136` is the same set):

| gate | C# | set | e9 | f2 (pre-reconcile) |
|------|----|-----|----|--------------------|
| entry shimmer | :93119 | long + 147 + 615 override | MATCH (`water_splash_gate`) | MATCH (`water_gate`) |
| entry honey | :93136 | short + 147 | MATCH (`honey_splash_gate`) | MATCH (`honey_gate`) |
| entry water | :93152 | long + 147 + 615 | MATCH | MATCH |
| entry lava | :93169 | short + 147 | MATCH | MATCH (`lava_entry_gate`) |
| exit shimmer | :93217 | long + 147 + 615 | MATCH | MATCH |
| exit honey | :93234 | short + 147 (300/59 swapped) | MATCH | MATCH |
| exit water | :93250 | long **WITHOUT 147** + 615 | **MISS** (reuses the entry gate) | **MISS** (reused `water_gate`) — **FIXED this phase** (§6) |
| exit lava | :93267 | short **WITHOUT 147** | **MISS** (reuses `honey_splash_gate`, keeps 147) | MATCH (`lava_exit_gate`, the documented quirk) |

Both 147-drops are latent in practice (type 147 Pinky is aiStyle 1, which
every gate already excludes) but the golden rule keeps them verbatim.

Structural coverage:

| feature | e9 | f2 |
|---------|----|----|
| never-wet head `:93055` (wetCount zero + lava force-false) | ABSENT (decision takes `wet_now` precomputed; no caller exists) | PRESENT, verbatim |
| aiStyle-116 wetCount pin `:93077-93080` | ABSENT | PRESENT |
| honey/shimmer latch writes `:93064-93075` | caller-owned (no caller) | PRESENT (`honey_now`/`shimmer_now` inputs) |
| entry wetCount=10 even when the type gate refuses | MATCH (`arm_cooldown`) | MATCH |
| exit wet=false BEFORE the wetCount check `:93180` | caller-owned | MATCH |
| exit arms read OLD latches | MATCH (inputs) | MATCH (state reads before the tail) |
| tail dry-clear + decrement `:92527-92536` | ABSENT | PRESENT (`water_collision_tail`) |
| `Npc` wet fields (`wetCount`/`lavaWet`/`honeyWet`/`shimmerWet`) | ABSENT (e9's own doc cites the gap) | PRESENT (spawn 0/false, the Entity defaults) |
| `noTileCollide` census (the `:90490` UpdateCollision head gate) | ABSENT | PRESENT (87-kind SetDefaults census + pumpkin/frost fold) |
| sim wiring | ABSENT — `water_collision_splash` is `#[allow(dead_code)]`, "the water-collision layer's wiring consumes the decision" | PRESENT — post-machine at the UpdateCollision position, gated `!no_tile_collide()`, reusing `npc_ai_slime::wet_collision` + `player::lava_collision` (the 441 refusal `:93318-93321`), lavaWet latched before the pass (`:93322-93324`) |
| RNG | draw-free decision (correct — the case-19 pitch rides `play_splash_sound`) | draw-free pass (same contract) |

**Verdict: f2's water pass is the faithful superset.** e9's decision fn
matches the 8 styles and the entry gates but misses the never-wet head,
the 116 pin, the tail, BOTH exit-gate 147-drops, and has no state/wiring.
f2 missed only the exit-WATER 147-drop (fixed this phase).

## 3. KEEP recommendation — the union (f2 is the union branch)

**KEEP from f2 (canonical):**
* `ambient_sound.rs::idle_sounds` — the ONE ladder (32 arms, §1).
* `ambient_sound.rs::water_collision_sounds` + `water_collision_tail` +
  `WaterWetState` — the ONE water pass (§2), now with the exit-water gate
  fix.
* `npc.rs` `wet_count`/`lava_wet`/`honey_wet`/`shimmer_wet` fields (+ the
  `wet` doc update) — the authoritative Entity state.
* `npc/data.rs::NpcKind::no_tile_collide` — the 87-kind census.
* `audio.rs` `Zombie_24-87` SFX_CUES rows + `ambient_sound.rs`
  `zombie_asset` 24-87 arms (the superset bank).
* Both sim wirings (pre-machine idle at `:90422` — now with e9's `hp > 0`
  guard; post-machine water at the UpdateCollision position).

**KEEP from e9 (the extras — additive, zero overlap with f2's 40 sites;
ported into f2 verbatim this phase so the merger can drop e9 wholesale):**
* VileSpit `ai[0]==2` tick cry (`NPC.cs:21478-21480`):
  `npc_ai_style9.rs::vile_spit_queue_cry` (queues `NpcDeath(9)` at the
  MOVED position — the double `position += velocity` precedes the play),
  the `NPC_Killed_9` bank row, the `play_npc_death_int` style-9 arm (int
  overload, NO NextFloat). STAGED (`#[allow(dead_code)]`) — the 112/666
  kinds are unported (the port's `to_u16` 112 is AncientCultist's
  protocol slot, NOT VileSpit; `cs_id` 439).
* Ghost flee cry (`NPC.cs:24686-24694`):
  `npc_ai_poltergeist.rs::ghost_flee_cry` (the dead-or->3000px gate, the
  `ai[3]==1` fade state, the `localAI[3]` 0→1 one-shot latch, queues
  `NpcDeath55` at CENTER), the `NpcSoundCue::NpcDeath55` variant + the
  `play_npc_sound_request` dispatch arm, the `NPC_Killed_55` bank row, the
  `play_npc_death_style` style-55 arm (the façade consumes the NextFloat
  on the audio-only stream). STAGED — the 316 kind is unported (the
  aiStyle-22 machine ports only the 330 Poltergeist branches; the Opacity
  fade + active=false drop stay the 316 wiring's).
* e9's wiring-level nimbus stream witness (adapted to f2's `idle_sounds`
  shape, §5).

**Naming union (wetCount vs the wet fields):** both branches already
snake-case `Entity.wetCount` as `wet_count` — no rename needed. The union
keeps f2's names end-to-end: `Npc.wet_count`/`lava_wet`/`honey_wet`/
`shimmer_wet` (the persistent Entity state) ↔ `WaterWetState.wet_count`/
`lava_wet`/`honey_wet`/`shimmer_wet` (the per-tick snapshot) ↔ the pass's
`wet_now`/`honey_now`/`shimmer_now` inputs (this tick's
`Collision.WetCollision` results). e9's `WaterSplashDecision.arm_cooldown`
("wetCount=10 armed") and its `was_wet`/`wet_count` parameters describe
the same state but own no writes — they retire with the decision fn; the
f2 pass writes `state.wet_count = 10` directly at the C# positions
(`:93089`/`:93187`). e9's gate helper names (`water_splash_gate`/
`honey_splash_gate`) are subsumed by f2's inline `water_gate`/`honey_gate`/
`lava_entry_gate`/`lava_exit_gate` (+ the new `water_exit_gate`).

## 4. DROP list (the merger drops these from e9; f2 carries the union)

From e9 `3793fef8`:
1. `ambient_sound.rs::idle_sound_cue` + `roll_range` (the duplicate
   ladder — f2's `idle_sounds` is the keeper, §1/§3).
2. `ambient_sound.rs` `zombie_asset` 24-86 rows (f2's 24-87 superset).
3. `audio.rs` `Zombie_24-86` SFX_CUES rows (f2's 24-87 superset).
4. `npc.rs::WaterSplashDecision` + `water_splash_gate` +
   `honey_splash_gate` + `water_collision_splash` (subsumed by f2's pass;
   e9's gates miss the never-wet head, the 116 pin and both exit 147
   quirks, §2).
5. e9's idle wiring block in `world_sim/npcs.rs` (f2's replaces it; e9's
   `hp > 0` guard is ADOPTED, not dropped).
6. e9 tests `w4_e9_the_idle_arms_roll_once_then_pick_the_slot` +
   `w4_e9_the_idle_slots_resolve_through_case_29` (they drive the dropped
   `idle_sound_cue`; f2's 32-arm seed-146 draw-order witness + bank
   witnesses cover the same contract, §5).
7. e9 tests `w4_e9_the_water_collision_entry_splash_edges_and_styles` +
   `w4_e9_the_water_collision_exit_splash_edges_and_styles` (they drive
   the dropped decision fn; f2's entry/exit/gates witnesses cover the
   same plus the head/pin/tail/quirks).

KEEP from e9 (already unioned into f2): the VileSpit/Ghost helpers + bank
rows + cue/dispatch arms + their three tests
(`w4_e9_the_spit_and_ghost_death_rows_follow_the_overload_split`,
`w4_e9_the_ghost_flee_cry_latches_once_at_the_centre`,
`w4_e9_the_vile_spit_cry_queues_at_the_moved_position` — names kept for
provenance).

## 5. Witness union

| contract | e9 witness | f2 witness | union keeper |
|----------|-----------|-----------|--------------|
| per-arm gate→style draw order | `..._idle_arms_roll_once...` (Rolls call-count, arms sampled) | `w4_f2_idle_sounds_the_first_ten_arms_draw_in_the_c_order` (ALL 32 arms, seed-146 same-seeded UnifiedRandom mirror incl. the pitch) | f2 (stronger: full-stream peek equality, every arm) |
| draw-free paths (shimmer/failed gate/dry trio/unlisted) | inside the arms test | `w4_f2_idle_sounds_the_gates_and_fall_throughs_draw_nothing_extra` | f2 |
| case-29 bank resolution + x0.5 + guard-before-pitch | `..._idle_slots_resolve_through_case_29` (17 sampled slots) | `w4_f2_the_idle_case29_slots_resolve_the_bank` (all 64) + `w4_f2_the_idle_case29_bank_rows_play_with_one_pitch_draw` (audio-level, all 64) | f2 |
| wiring stream contract (idle roll BEFORE the AI, dead skip) | `w4_e9_the_nimbus_idle_roll_rides_the_stream_before_the_ai` | (none) | e9's, ADAPTED to f2's wiring as `w4_f2_the_nimbus_idle_roll_rides_the_stream_before_the_ai` |
| water entry/exit styles + cooldown + gates | the two decision tests | `w4_f2_water_collision_the_entry_quartet...` / `..._exit_quartet_reads_the_old_latches` / `..._the_gates_block_and_the_lava_quirk` (+ the exit-water 147 pin added this phase) | f2 |
| case-19 six-style stream | (cites E21f) | (cites the landed `s1_e_21f_the_jump_and_splash_draws_ride_the_passed_stream`) | landed E21f |
| spit/ghost overload split + latch + moved position | the three extras tests | (none) | e9's, ported verbatim |

## 6. Fidelity fixes landed in f2 this phase (from the compare)

1. **Exit-water gate** (`ambient_sound.rs`): the C# `:93250` gate DROPS
   `type != 147` (its entry twin `:93152` carries it) — the SECOND
   147-quirk, missed by BOTH branches' original passes. f2 now carries a
   distinct `water_exit_gate` used by the `:93252` arm; the quirk witness
   pins type 147 @ aiStyle 0 exiting water (plays, where the entry gate
   would refuse).
2. **Idle dead guard** (`world_sim/npcs.rs`): e9's `hp > 0` wrap adopted
   — the C#'s `life <= 0` return (`:90385-90392`) precedes `IdleSounds()`
   (`:90422`).

## 7. e9b (SandShark `:40632` + wiring) — pushed-tip read

**SUPERSEDED by §9** — at the original reconcile the e9 pushed tip was
`3793fef8` (no SandShark content, no `e9b` branch); e9 has since pushed
`a41b2b72` (the water wiring + the 27 tail sites + the fix round). The
SandShark site's ownership stands: **d33's `npc_ai_style103.rs` is the
keeper** (it owns the machine; tip `b08acf463` carries the `math_sign`
zero-case helper — e9b's fix-round point already matched there). What
the C# site is, for the record:
`NPC.cs:40628-40637` (the aiStyle-103 Sand Sharks' chase arm, family
542-545): the lunge gate (`!flag123 && Math.Sign(velocity.X) == direction
&& Distance < 400 (remix 700) && (ai[2] >= 30 || ai[2] < 0)`) fires, on
`localAI[0] == 0`, `SoundEngine.PlaySound(14, base.Center, 542)` +
`localAI[0] = -1`, then `ai[2] = -30` + the
`DirectionTo(aim + (0,-80)) * 12` leap. e9b's `sand_shark_cry` decision
helper DROPS (the d33 machine's sounds out-struct owns the site — the
keep-one precedent); f2 carries the queue-side `SandSharkCry` cue +
dispatch (§10) for the wiring's drain.

## 8. DOC-SWEEP — the two documented f2 skips, ported (unclaimed)

Claim check: `git grep` over ALL origin task branches for `93181` /
`93325` / `Collision_LavaCollision` — only f2's own skip comments and
e9's doc cite them; no branch ports either. Both ported this phase:

### 8a. The type-620 exit jump (`NPC.cs:93181-93184`) — movement, census 1

`if (this.type == 620 && this.GetTargetData().Center.Y < base.Center.Y)
base.velocity.Y -= 8f;` — the GoblinShark's water-exit leap, inside the
exit edge AFTER the velocity.X halving + wet clear, BEFORE the wetCount
check. Census: **type 620 only** (`NPCID.GoblinShark`, `NPCID.cs:11917`;
the aiStyle-107 improved-walker family's sole user of this arm;
SetDefaults `NPC.cs:16950-16968` — 34x58, 5000 HP, lavaImmune). Ported as
the pure staged helper `ambient_sound.rs::water_exit_jump(npc_type,
target_center_y, center_y) -> f32` (the −8 delta or 0). STAGED/UNWIRED:
the 620 kind is unported, and 620 is in `NPCID.Sets.UsesNewTargeting`
(`NPCID.cs:10434`) — its `GetTargetData` can return an NPC target
(`NPC.cs:6901-6912`), a targeting axis the port lacks; the caller passes
the resolved target centre when the kind lands. The strict `<` (equal
centres do NOT jump) is pinned by the witness. The velocity.X halving
(`:93179`) stays the movement lane's (it is not 620-specific).

### 8b. The lava strike/buff (`NPC.cs:93325-93337`) — combat seam, census 49

`Collision_LavaCollision` (`:93315-93340`): on a lava overlap (the 441
refusal `:93318-93321` applied — f2's wiring already does both), after
the `lavaWet = true` latch: `if (!lavaImmune && !dontTakeDamage &&
Main.netMode != 1 && immune[255] == 0)` → `immune[255] = 30`; then
remix && !friendly → `AddBuff(24, 180)` ONLY; else `AddBuff(24, 420)` +
`StrikeNPCNoInteraction(50, 0f, 0)`. Ported:

* `npc/data.rs::NpcKind::lava_immune()` — the SetDefaults census: **49
  vanilla ids** carry `lavaImmune = true` (the `:8511` reset is false):
  24 FireImp, 59 LavaSlime, 60 Hellbat, 62 Demon, 66 VoodooDemon, 151
  Lavabat, 156 RedDevil, 379/380 CultistArchers, 438 CultistDevote, 534
  DemonTaxCollector, the DD2 band 548/549 + 551-578 minus 550 (the
  Bartender is town), 620 GoblinShark, 653/654/655 the hell critters, 657
  QueenSlime, 661 EmpressButterfly, 662 PirateGhost, 690 StatueMimic.
  **33 map to ported kinds** (FireImp, LavaSlime, Hellbat, VoodooDemon +
  the 29 DD2 kinds); the 16 unported ids (62/151/156/379/380/438/534/578/
  620/653/654/655/657/661/662/690) land with their kinds. The LIVE writes
  (FairyCritter `:48091`/`:48114`/`:48155`, Plantera's phase-2 `:60715`)
  are not modelled — the SetDefaults row stands (the `no_gravity`/
  `no_tile_collide` granularity precedent).
* `npc.rs::Npc::lava_collision_combat(remix_world) -> bool` — the gate +
  `immune[255] = 30` + the buff arms; returns whether the 50-damage
  strike fires (the caller runs the strike seam). `Main.netMode != 1`
  folds to the port's host/single-player constant (the established
  convention). The buff rides the landed `NpcBuffs::add_buff` (the
  buffImmune[24] rows — e.g. LavaSlime's `[20, 24, 323]` — refuse it
  there, exactly the C#'s `AddBuff` head); the OnFire DoT is the landed
  `npc_buffs` engine's.
* Wiring (`world_sim/npcs.rs`, the f2 water block): the lava latch site
  now runs the combat half BEFORE the splash pass (the C#'s `:92526`
  argument-evaluation order — `Collision_LavaCollision()` runs first),
  the strike through `Npc::strike_projectile(npcs, i,
  strike_npc_damage(50, defense, false), x, 0.0, false, Some(255),
  expert)` — the `StrikeNPCNoInteraction` owner-255 shape
  (`NPC.cs:81184-81187`), knockback 0 (the `knock_back_npc` zero-impulse
  no-op), the defense fold the port's strike callers apply
  (`strike_npc_damage`, `NPC.cs:81239`). The strike's HitSound tail is
  c13's StrikeNPC lane (the port's strike fns are soundless; the owning
  caller plays) — the lava strike inherits that seam.
* Witnesses: the census spot-checks (`npc/data.rs`), the combat gate/buff
  matrix (`npc.rs`), and the wiring tick (a zombie in a lava pool takes
  the 420-tick OnFire + the defense-folded 50 strike + immune[255]=30; a
  LavaSlime beside it takes NOTHING) — WRITTEN NOT RUN.

**Remaining documented skips (updated by §9):** the splash dust bursts
(the dust lanes), `TryRemovingWaterPerishableEffects` (`:93277-93301`,
the buff seam — its `dripping` tail is already cited in `npc_buffs.rs`),
the shimmer `TryAddingRepeatedBuff(353, 100)` (`:93071-93074`, the buff
seam). The exit velocity.X halving (`:93179`) is NO LONGER a skip — §9
adopts it into the pass.

## 9. CLOSEOUT — the e9b pushed tip `a41b2b72` re-adjudication

The original reconcile (§§1-8) read e9 at `3793fef8`. e9 has since
pushed four more commits: `cb53fa39` (the 8 splash arms — already in
§2's compare), `d7a61594` (the water WIRING: `npc_water_collision.rs`,
488 lines), `9b96eff4` (the 27 tail sites, `npc_sounds_tail.rs`),
`255d7ccf` (the Math.Sign zero-case + surface-inset witness fix round),
`a41b2b72` (allow(dead_code) on the unwired module). Full diff re-read
this phase; the arm-by-arm re-adjudication:

### 9a. The water layer — e9b is now superset-SHAPED, still not faithful

e9b's `npc_water_collision.rs` landed the state + wiring §2 listed as
ABSENT: the `wet_count`/`lava_wet`/`honey_wet`/`shimmer_wet` fields
(+ a fifth, `wet_prev`), the never-wet head (`never_wet`, verbatim
`:93055`), the 116 pin, the sticky sets, the entry/exit edges, the
`wetCount=10` arming even on a refused play, the exit velocity.X halving
(`:93179` — f2 had cited it as the movement lane's), the dry-clear +
decrement tail, the centred-box/surface-inset/slope probes
(`wet_collision_probe`/`lava_collision_probe` — faithful re-ports of
`Collision.cs:1572-1651`/`:1653-1684`, duplicating the port's landed
`npc_ai_slime::wet_collision` + `player::lava_collision`), and the
sim wiring at the `:92526` position gated by a hand-listed
`water_collision_kind` census.

**The decision fn it wires is UNCHANGED from `3793fef8`** — the §2
fidelity gaps ALL stand:

| gap | e9b `a41b2b72` | f2 (post-closeout) |
|-----|----------------|--------------------|
| exit-WATER gate `:93250` drops 147 | **MISS** (`water_splash_gate` reused, npc.rs:167) | FIXED (§6, `water_exit_gate`) |
| exit-lava gate `:93267` drops 147 | **MISS** (`honey_splash_gate` reused, npc.rs:171) | MATCH (`lava_exit_gate`) |
| the probes | duplicate re-ports (two new copies) | the landed shared fns (keep-one) |
| the kind gate | hand-listed ~70-kind `water_collision_kind` (drifts from SetDefaults) | the 87-kind `no_tile_collide` census (the `:90490` gate's own table) |
| the lava combat half `:93325-93337` | cited skip ("the damage-layer seam") | PORTED + WIRED (§8b) |
| the 620 exit jump `:93181-93184` | cited skip ("unported-kind movement") | STAGED (§8a, `water_exit_jump`) |
| the shimmer buff `:93071-93074` | cited skip | cited skip (both — the buff seam) |

**Verdict re-confirmed: f2's water pass remains the faithful superset;
the KEEP recommendation (§3) stands unchanged.** e9b's wiring is
real but rides the unfaithful decision — the merger still drops e9
wholesale, now including `npc_water_collision.rs` + its 4 witnesses +
the wiring block.

**Two e9b points ADOPTED into f2 this phase** (the fold-list, e9→f2):

1. **The `wet_prev` edge latch** (e9b npc.rs field + `collision_water_step`
   keying). Re-derived against the port: `update_cave_bat`/
   `update_swimmer` (npc.rs:10492/:18200) overwrite `self.wet` pre-AI
   with the `is_wet` stand-in for the machine reads — and the bat family
   (CaveBat/JungleBat/Hellbat/VoodooDemon, aiStyle 14) is NOT in the
   `no_tile_collide` census (the C# aiStyle-14 block writes only
   `noGravity`, `:22851`), so those kinds DO reach f2's water block with
   a clobbered `wet`. Keying the edge on `wet` would miss every entry
   edge on them (the machine probe already set true). f2 adopts e9b's
   field verbatim: `Npc::wet_prev` (spawn false), the sim snapshots it
   into `WaterWetState.wet` and stamps BOTH `wet`/`wet_prev` at the
   tail. The Goldfish/Piranha/Shark swimmers are noTileCollide (census)
   and never reach the block — their `wet` overwrite is machine-only,
   unchanged. Witness: `w4_f2_the_splash_edge_keys_on_the_wet_prev_latch`
   (the CaveBat in the column: entry arms 10→9 despite the noGravity
   play refusal, the wet→wet tick decays 9→8).
2. **The exit velocity.X halving** (`:93179`). f2's original pass cited
   it as the movement lane's; e9b landed it in the wiring. Adopted INTO
   the pure pass via `WaterWetState.vx` (the sim snapshots `Npc::vx` in,
   copies out; the exit edge halves BEFORE the wet clear + the arms —
   the C# order). The exit-quartet witness pins 4→2. The §8 skip list
   updated.

**e9b's probes/witnesses:** the centred-box/surface-inset/slope maths
match the landed shared fns line-for-line (both re-read against
`Collision.cs:1572-1651` this phase) — the duplicates DROP; e9b's
surface-inset witness contract (the liquid-1 column, the y=940 miss /
full-cell hit) is ADAPTED onto the shared probe as
`npc_ai_slime.rs::w4_f2_the_wet_probe_surface_inset_misses_a_shallow_box`
(+ the y=958 inset-band hit). e9b's wiring stream witness
(`w4_e9_the_water_splash_wiring_spends_one_case19_pitch`) is ADAPTED to
f2's inline-play shape as
`w4_f2_the_water_splash_wiring_spends_one_case19_pitch`. e9b's
`wet_prev`-vs-`wet` rationale comment is adopted into the field docs.

### 9b. The exact fold-list, both ways (e9c active reads)

**f2 → e9c (what e9c must NOT re-port — f2 owns these):**
* `idle_sounds` (the 32-arm ladder) + both wirings' `hp > 0` guard.
* `water_collision_sounds`/`water_collision_tail`/`WaterWetState`
  (now with `vx` + the `wet_prev` snapshot contract) + the sim water
  block (the `no_tile_collide` census gate, the lava combat half, the
  latch stamping).
* `Npc::wet_count`/`lava_wet`/`honey_wet`/`shimmer_wet`/`wet_prev`.
* `NpcKind::no_tile_collide`/`lava_immune` + `Npc::lava_collision_combat`.
* `water_exit_jump` (staged).
* The bank superset: `Zombie_24-87` + `Zombie_115-117` (case 65) +
  `NPC_Killed_9/17/55/63` + `Item_150/167/170/172` + the 7 Pal/
  StatueMimic `NPC_CUSTOM_CUES` rows.
* The cue/dispatch arms: `NpcDeath(9/17)` int, `NpcDeath55/63` façade,
  `NpcHit6`, `ItemCue`, `SandSharkCry`, `QueenSlimeMoan`, `Pixie`,
  `Trackable` + `play_queen_slime_moan`/`play_npc_trackable`.
* `vile_spit_queue_cry` + `ghost_flee_cry` (staged, §3).
* `npc_sounds_tail.rs`'s TWO staged helpers (§10).

**e9 → DROP (the merger drops these; f2 carries the union):** §4's
list stands, PLUS:
8. `npc_water_collision.rs` whole (the probes duplicate the landed
   shared fns; `never_wet`/the 116 pin/the edges/the tail are subsumed
   by f2's pass; `water_collision_kind` is superseded by the
   `no_tile_collide` census; the wiring block is replaced by f2's —
   the `wet_prev` keying and the vx halving ADOPTED, not dropped).
9. e9b's 4 water witnesses (the probe pair, the never-wet/116 test, the
   exit/churn test) — f2's entry/exit/gates/latch/inset/stream witnesses
   cover the same contracts plus the quirks/combat/jump.
10. `npc_sounds_tail.rs`'s 25 machine-owned decision helpers (§10's
    census — the d-band machines own the sites; the two unclaimed
    helpers are re-landed in f2's own `npc_sounds_tail.rs`).
11. e9b's 10 tail witnesses (they drive the dropped helpers; the
    bank/cue contracts are covered by f2's
    `w4_f2_the_tail_bank_rows_follow_the_overload_split` +
    `w4_f2_the_case65_moan_picks_115_118_behind_the_guards` +
    `w4_f2_the_medusa_gaze_and_the_reflect_tink`; the machine-side
    gate contracts live in the d-band modules' own witnesses).
12. e9b's `Npc` field/cue/bank duplicates (the same names f2 carries —
    the union keeps f2's spellings; e9b's `Splash(i32)` cue retires
    with the decision fn, f2's pass plays inline).

**KEEP from e9 (already unioned):** §3's extras + this phase's two
adoptions (`wet_prev`, the vx halving) + the two adapted witnesses.

## 10. The 27-tail-site census union (e9b `9b96eff4` vs the d-bands)

e9b's re-census: 232 NPC.cs PlaySound sites, 31 unclaimed by
citation-range, minus `:40581` (the S1-E-21g roar bank) and the three
sphere HitEffect sites (c13) = 27, "every one on an UNPORTED kind".
**That claim is stale** — the d-band machine waves pushed since. The
full-branch `git grep` sweep (every origin task branch, the site
citations + the cue names) re-adjudicates each:

| # | site(s) | kind(s) | e9b helper | owner (pushed) | verdict |
|---|---------|---------|-----------|----------------|---------|
| 1 | :24449 | Antlion 69 | `antlion_spit_cue` | d24 `npc_ai_style19.rs` (`Ai19Effects::sounds`) + g8 wires the kind (`Item5` cue) | DROP (machine owns; g8's `Item5` folds onto f2's `ItemCue(5)` at the merge) |
| 2 | :37268/:37321 | 472/521 | `apparition_teleport_cue` | d32 `npc_ai_style86.rs` (`sound_item8`) | DROP |
| 3 | :40632 | SandShark 542-545 | `sand_shark_cry` | d33 `npc_ai_style103.rs` (the `(14,542)` sounds entry + `math_sign`) | DROP (keep-one d33, §7) |
| 4 | :42983/:42987/:42997/:43001 | Pals 695/696 | `pal_idle_cry`/`pal_joy_cry` | d33 `npc_ai_style127.rs` (`PAL_SOUNDS` out-struct) | DROP helper; **bank rows ADOPTED** (the 4 trackables + `play_npc_trackable`) |
| 5 | :43562/:43569/:43580/:43608 | StatueMimic 690 | `statue_mimic_scare/jump/laugh` | d33 `npc_ai_style126.rs` (the trackable sounds out-struct) | DROP helper; **bank rows ADOPTED** (scare/jump/laugh_0) |
| 6 | :43726 | ClumsySlimeBalloon 686 | `balloon_pop_cue` | d33 `npc_ai_style125.rs` (the pop DeathSound out-field) | DROP helper; **NPC_Killed_63 + the style-63 arm ADOPTED** |
| 7 | :47951/:48062 | WindyBalloon 594 | `balloon_pop_cue` | d33 `npc_ai_style113.rs` (the pop out-field) + d42's DeathSound row | DROP helper; the row above serves both |
| 8 | :44848 | PirateGhost 662 | `pirate_ghost_spawn_cue` | d33 `npc_ai_style122.rs` (the `(3,6)` sounds entry) | DROP helper; **NpcHit6 cue + dispatch ADOPTED** (the queue-side spelling) |
| 9 | :45558 | QueenSlime 657 | `queen_slime_landing_cue` | d33 `npc_ai_style121.rs` (the `(2,167)` sounds entry) | DROP helper; **Item_167 row ADOPTED** |
| 10 | :45759 | QueenSlime 657 | `queen_slime_idle_moan` | d33 `npc_ai_style121.rs` (the `(65,1)` roar entry — the Next(360) gate in the machine) | DROP helper; **case-65 arm + Zombie_115-117 + QueenSlimeMoan cue + play_queen_slime_moan ADOPTED** (the bank superset — d33's machine has no case-65 player yet) |
| 11 | :47263/:47268/:47293 | BloodNautilus 618 | `blood_nautilus_volley_cue`/`scream_cue` | d33 `npc_ai_style117.rs` (the `(2,170)`/`(2,172)` entries) | DROP helpers; **Item_170/172 rows ADOPTED** |
| 12 | :48327/:48473/:48545 | FairyCritter 653-655 | `fairy_circle_hum`/`fairy_far_hum` | d33 `npc_ai_style112.rs` (the `(27,1)` entries) | DROP helpers; **Pixie cue + dispatch ADOPTED** (`play_pixie` already landed S1-E-21g) |
| 13 | :48682 | FairyEffects (static) | `fairy_effects_cue` | d33 `npc_ai_style112.rs::fairy_effects` (the `(2,4)` entry) | DROP (Item_4 row already landed) |
| 14 | :56403 | Medusa 480 | `medusa_gaze_cue` | **UNCLAIMED** — d22's style-3 core claims the 22 plain walkers only (the 480 gaze block unported); d40's Medusa is data-only (SetDefaults/FindFrame), no gaze machine | **PORTED into f2** (`npc_sounds_tail.rs::medusa_gaze_cue`, staged) + NPC_Killed_17 + the int style-17 arm |
| 15 | :66429 | BigMimic 473-476 | `reflect_projectile_cue` | **UNCLAIMED (the sound body)** — d32's `npc_ai_style87.rs` emits the `reflect_request` out-field (the `ReflectProjectiles(Hitbox)` scan, `:66395-66404`), but NO branch ports the `ReflectProjectile` body (`:66427-66429` — the Item150 play + the velocity/damage rewrite); the sim's projectile pass does not consume the request | **PORTED into f2** (`npc_sounds_tail.rs::reflect_projectile_cue`, staged — the sound seam the projectile pass's reflect wiring consumes) + Item_150 row |

The bank-superset principle (§1's Zombie_24-87 precedent): the machine
modules own the SITES (their out-structs carry the cue tuples), but
their wirings will drain through the shared bank + dispatch — so f2
carries every row/arm/cue the 27 sites name, and the d-band wirings
become one-call consumers. e9b's pure decision helpers duplicate the
machines' gate maths (already witnessed there) and DROP wholesale
except the two unclaimed sites, re-landed in f2's own
`npc_sounds_tail.rs` (2 helpers + 1 witness, staged
`#![allow(dead_code)]` — the style64/67 precedent).

**QueenSlime-case-65 note:** e9b's case-65 arm
(`LegacySoundPlayer.cs:1382-1402` — the 115/116/117 playing guards,
`Next(115,118)`, ×0.5, NO pitch) is verbatim-faithful (re-read this
phase); f2 adopts it + the three `zombie_asset` slots + the three
`SFX_CUES` rows. d33's style-121 machine emits `(65, 1)` — its wiring
maps that onto `play_queen_slime_moan` (the style argument is ignored
by case 65, the C# reads no Style there).

## 11. The staged VileSpit/Ghost/620-jump — the d-band activation reads

The three staged helpers' kinds were re-checked against the CURRENT
pushed tips (the original reconcile read them unported):

* **VileSpit 112/666** — d35 (`e5558b0a9`) ports the 112 DATA row
  (`NpcKind::VileSpit`, to_u16 455, aiStyle 9, NPC_Killed_9) but
  explicitly EXCLUDES it from the style axis
  (`9 if !matches!(self.kind, NpcKind::VileSpit)` — "the 112 data-only
  kinds skip the aiStyle-9 family; they fall through to the
  update_kind_ai no-op until the AI wave"). g8's wire wave does not
  touch it. The 666 kind is unported everywhere. **The cry stays
  STAGED** — no machine runs the `ai[0]==2` tick; activating would
  queue a cry from a kind that never moves. The d35 AI wave (or g8's
  next batch) is the activation point: `vile_spit_counter_tick`'s
  `true` → the double move → `vile_spit_queue_cry`.
* **Ghost 316** — d38 (`6f9dd02db`) ports the kind AND wires
  `npc_ai_ghost::ghost_ai`, which INLINES the flee cry as
  `NpcSoundCue::NpcDeath(55)` — but d38's `play_npc_death_int` has only
  the 6/22 arms, so that queue currently resolves to None (a silent
  bank gap on d38's own branch). f2's `NpcDeath55` façade arm +
  `NPC_Killed_55` row + `ghost_flee_cry` are the faithful spelling
  (the C# `:24693` is the SoundID façade — the NextFloat IS consumed).
  **The merge fold: d38's inline `NpcDeath(55)` push repoints to
  `NpcDeath55`** (or the dispatch's `NpcDeath(55)` arm routes to the
  façade); f2's staged `ghost_flee_cry` then retires as the duplicate
  (d38's machine owns the latch + the fade). Until that merge, f2's
  helper stays STAGED (the 316 kind is not on f2's tree — activation
  is not possible branch-locally).
* **620 GoblinShark** — d42 (`9b22c091a`, the roster-complete tip)
  ports the DATA row (to_u16 394, cs_id 620, aiStyle 107, 34×58, 5000
  HP, lavaImmune) but the kind arm is the idle stand-in
  (`NpcKind::GoblinShark => {}` — "aiStyle 107 is UNWIRED"). d43
  (631-697) has NOT pushed (its tip is the base `a725acb0`). **The
  jump stays STAGED** — no aiStyle-107 machine integrates the shark,
  and the `UsesNewTargeting` resolved-centre axis is still absent.
  The activation point is the 107 machine wave: the exit edge calls
  `water_exit_jump(620, resolved_target_center_y, center_y)` and adds
  the delta to vy.

**FINAL status: research/94 is COMPLETE.** The reconcile (§§1-6), the
doc-sweep (§8), the e9b re-adjudication (§9), the tail-census union
(§10) and the activation reads (§11) are all landed; the union branch
is f2, the DROP list (§4 + §9b) is exact, and the three staged helpers'
activation points are named. No open questions remain for the merger.
(The retire phase extends the note: §12 the exact d38 repoint hunk,
§13 the 232-site FINAL declaration, §14 the e9d cross-receipt +
FINAL-FINAL.)

## 12. The d38 NpcDeath55 gap — the EXACT repoint hunk (for d38d/merger)

Re-probed at d38's CURRENT pushed tip `1ba0395c7` (the FINAL sheet-4
docs commit; the ghost module unchanged since `6f9dd02db`). The gap
§11 named stands, now line-pinned:

* **The push:** `crates/tr-app/src/npc_ai_ghost.rs:67` — inside
  `ghost_ai`'s far/dead gate (`player.dead || dist > 3000.0`, `:60`),
  the `ai[3] == 1.0` fade arm (`:61`), the `local_ai[3]` 0→1 one-shot
  latch (`:64-65`) queues `NpcSoundCue::NpcDeath(55)` at `(cx, cy)` —
  the CENTRE (`npc.center()` at the fn head), the position the C#
  passes (`NPC.cs:24693`, `(int)base.Center.X/Y`). The latch + the
  centre are faithful; only the cue spelling is wrong.
* **The silent resolution:** d38's `audio.rs::play_npc_death_int`
  (`:1366`) has ONLY the `6 => "NPC_Killed_6"` / `22 =>
  "NPC_Killed_22"` arms (`:1377-1378`) — style 55 hits `_ => return
  None` (`:1379`). d38's tree has NO `NpcDeath55` enum variant
  (`npc.rs:1222-1255`: `NpcDeath(i32)`/`NpcDeath61`/`NpcDeath13`
  only), NO `NPC_Killed_55` SFX_CUES row, and its
  `play_npc_death_style` (`:1341`) has only the 13/61 arms. The cry
  is a silent bank gap on d38's own branch.
* **Why the repoint, not a dispatch re-route:** the C# site is
  `SoundEngine.PlaySound(SoundID.NPCDeath55, ...)` — the STYLE façade
  (`SoundEngine.cs:93-99`), which consumes the `GetRandomPitch`
  NextFloat on the audio-only stream. `NpcDeath(i32)` is the port's
  INT-overload spelling (`SoundEngine.cs:102-108`, NO NextFloat — the
  tower/Moon-parts/VileSpit sites). Routing `NpcDeath(55)` to the
  façade inside the dispatch would break the overload split for every
  int caller; the faithful fix is the one-line cue re-key at the
  push site.

**The hunk (apply on the merged tree, or on d38d after f2's union
lands — f2 supplies every symbol it names):**

```diff
--- a/crates/tr-app/src/npc_ai_ghost.rs   (d38 tip 1ba0395c7)
+++ b/crates/tr-app/src/npc_ai_ghost.rs   (post-merge, f2 union)
@@ -64,7 +64,7 @@
             if npc.local_ai[3] == 0.0 {
                 npc.local_ai[3] = 1.0;
                 npc.wants_sound.push(crate::npc::NpcSoundRequest {
-                    cue: crate::npc::NpcSoundCue::NpcDeath(55),
+                    cue: crate::npc::NpcSoundCue::NpcDeath55,
                     x: cx,
                     y: cy,
                 });
```

The symbols f2's union carries (all verified present on this branch):
the `NpcSoundCue::NpcDeath55` variant (`npc.rs:1178`), the dispatch arm
(`world_sim/npcs.rs:1158-1160` → `play_npc_death_style(55, ...)`), the
`NPC_Killed_55` SFX_CUES row (`audio.rs:192`, case 4) and the style-55
arm (`audio.rs:1470`, the façade NextFloat + the shared case-4 tail).
The drain runs on the `UpdateNPCs` stream after `update_rng` (the
E21g seam) — the same stream d38's `ghost_ai` draws on, so the cry's
pitch draw lands in the C# position.

**The retirement fold:** with the repoint applied, f2's staged
`npc_ai_poltergeist.rs::ghost_flee_cry` retires as the duplicate —
d38's `ghost_ai` owns the latch + the Opacity fade (the alpha-fold
convention) + the flee drift. Its witness
`w4_e9_the_ghost_flee_cry_latches_once_at_the_centre` retargets onto
d38's machine (drive `ghost_ai` with a dead/far player, pin the ONE
`NpcDeath55` push at the centre + the `local_ai[3]` latch); the
`NPC_Killed_55` bank/overload contract stays covered by f2's
`w4_f2_the_tail_bank_rows_follow_the_overload_split` + the audio-level
style-55 witness (`audio.rs:4340-4352`).

**Pre-merge local fix (d38d's option, NOT applied here — never touch
other lanes):** if d38d wants the cry audible before the merge, the
faithful local shape is the same façade spelling (add the `NpcDeath55`
variant + dispatch + row + style-55 arm — f2's hunks verbatim); adding
a `55` arm to `play_npc_death_int` would be the WRONG overload (it
would drop the NextFloat the C# consumes).
