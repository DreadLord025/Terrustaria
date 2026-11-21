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

**No `e9b` branch exists on origin** (full `ls-remote` checked twice this
phase), and the e9 pushed tip `3793fef8` contains NO SandShark content
(its 6-file diff is exactly §1-§4's scope; the e9 worktree is clean at
the tip). The claimed e9b work is unpushed/in flight — nothing to
reconcile against yet. What the C# site is, for the record:
`NPC.cs:40628-40637` (the aiStyle-103 Sand Sharks' chase arm, family
542-545): the lunge gate (`!flag123 && Math.Sign(velocity.X) == direction
&& Distance < 400 (remix 700) && (ai[2] >= 30 || ai[2] < 0)`) fires, on
`localAI[0] == 0`, `SoundEngine.PlaySound(14, base.Center, 542)` +
`localAI[0] = -1`, then `ai[2] = -30` + the
`DirectionTo(aim + (0,-80)) * 12` leap. **The site is already claimed:**
`origin/task/W4-d33-npc-ai-styles-95-127` (`ee8b2668`, ancestor of tip
`dcb59b7f`) ports the whole aiStyle-103 arm as the UNWIRED
`npc_ai_style103.rs`, the cry included as a `(14, 542, cx, cy)` entry in
its `sounds: Vec<(i32, i32, f32, f32)>` out-struct (the style75 sounds
seam). If e9b pushes a duplicate, the d33 module is the keeper (it owns
the machine); the reconcile pattern of this note applies.

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

**Remaining documented skips (unchanged):** the splash dust bursts
(the dust lanes), `TryRemovingWaterPerishableEffects` (`:93277-93301`,
the buff seam — its `dripping` tail is already cited in `npc_buffs.rs`),
the shimmer `TryAddingRepeatedBuff(353, 100)` (`:93071-93074`, the buff
seam), the exit velocity.X halving (`:93179`, the movement lane).
