# 80 — DOC-SWEEP lane 9: the projectile-sound residuals (S1-E-21h d76)

**Branch** `task/W4-d76-proj-sounds-2` (worktree `TR-W4-d76`), the DOC-SWEEP
phase on top of the d76 second-wave port (`65d82312`) and its classification
doc (`0a1fdae3`). **Code-only mode**: no cargo/rustc/rustfmt/tests/dotnet/
graphify invoked (the Lead's constraint); every witness is WRITTEN NOT RUN.
Verification = whole-block C# re-reads against the cited `Projectile.cs`
windows, the sibling-branch claim cross-checks (`git show` of the pushed
`origin/task/W4-d77-proj-sounds-3`, `…d18…`, `…d19…`, `…d21…` tips), and the
zero-caller greps for the dead `_Old` arms.

The d76 deferred list (ANALYSIS "The d76 port", commit `0a1fdae3`) carried
four residual families. This lane closes them:

1. the Corrupt Whip's ×0.6 `volumeScale` (the Lights Bane post-gate-scale
   precedent — ported),
2. the Kill()-tail non-bomb arms in the d76 line-order range
   (`Projectile.cs:41978-70958`) that d77 did not claim,
3. the unported-kind cues whose rows NOW landed (the d18/d19/d21 row pushes
   — cross-checked against the d76 DEFERRED kind list),
4. the dead `_Old` arms (verified truly dead, zero callers).

Commits: `fe205fc9` (the volume-scale port), `dd90f162` (the Kill()-tail cue
map), plus this doc + the anchors rows.

---

## 1. The volumeScale port (the Corrupt Whip ×0.6 + the Lights Bane slash)

**The C#.** `SoundEngine.PlaySound(LegacySoundStyle type, Vector2 position,
float pitchOffset = 0f, float volumeScale = 1f)` (`SoundEngine.cs:88-99`)
folds the call's fourth argument into `type.Volume * volumeScale` and passes
it to the legacy player as the `volumeScale` parameter. The legacy player
applies it as the **post-gate** `soundEffectInstance.Volume *= volumeScale`
(`LegacySoundPlayer.cs:1411`) — AFTER the attenuation gate (`:369-436`) and
AFTER the per-case volume tweak. Two d76-range sites carry a non-1 scale:

- `AI_165_Whip` `:47720` — the Corrupt Whip (1029) tip cue
  `PlaySound(SoundID.NPCDeath1, vector2, 0f, 0.6f)` (id 4, style 1, ×0.6).
- `AI_188_LightsBane` `:42234-42238` — the slash cue
  `PlaySound(SoundID.Item60, base.Center)` then
  `soundEffectInstance.Volume *= 0.15f * this.ai[0]` (id 2, style 60, the
  post-play scale).

The d76 second wave ported both cues' id/style/position but DEFERRED the
scale ("the queue→drain batch plays at the style's own volume — the Lights
Bane post-gate-scale precedent"). This lane ports the scale.

**The port.** The proj-sound queue payload widens from the 5-tuple
`(id, style, x, y, pitchOffset)` to the 6-tuple
`(id, style, x, y, pitchOffset, volumeScale)`:

- `Projectile.wants_sound` (`projectile.rs:716`), the pool's
  `pending_proj_sounds` (`:14560`) and `drain_proj_sounds` (`:16209`), and
  the golf arm's `GolfTick.sounds` / `GolfEffect::CupScored.cue` /
  `emit_golf_ball_explosion_draws` / `put_ball_in_cup_text` /
  `put_ball_in_cup` return types (`projectile_ai_golf.rs`) all carry the 6th
  term. Every existing cue construction passes `1.0` (the no-scale default);
  the two scaled sites pass their scale.
- `Audio::play_use_sound` becomes a thin wrapper over the new
  `Audio::play_use_sound_scaled(style_index, x, y, pitch_offset,
  call_volume_scale, rng)` (`audio.rs:1094`). The attenuation gate still
  reads the UNSCALED `style.volume`; the case-2 volume tweak runs; then the
  emit multiplies `volume_scale * call_volume_scale` — exactly the C# order
  (`:1411` post-tweak). The case-42 trackable arm scales the same way.
- `npc_hit_play` / `npc_death_play` (the case-3/4 tails) gain a
  `call_volume_scale` parameter, applied post-tweak
  (`volume_scale * call_volume_scale`). Their existing callers
  (`play_npc_hit_sound`/`_int`, `play_npc_death_sound`/`_style`/`_int`) pass
  `1.0`; `play_proj_sound`'s id-3/4 whip arm threads the queue's term.
- `Audio::play_proj_sound` gains the `volume_scale` parameter and routes it
  to the id-2 (`play_use_sound_scaled`) and id-3/4 (`npc_hit_play`/
  `npc_death_play`) façades. The drain (`world_sim/projectiles.rs:460`) and
  the golf-cup consumer (`:630`) destructure the 6th term and pass it.
- The whip tip arm (`projectile.rs:5656`) queues
  `CORRUPT_WHIP => (4, 1, tip.x, tip.y, 0.0, 0.6)`; the other three tip cues
  pass `1.0`. The Lights Bane arm (`:10814`) queues
  `(2, 60, cx, cy, 0.0, 0.15 * self.ai[0])`.

**RNG.** The scale is a volume-only term — it draws NOTHING. The case-2/3/4
pitch draws ride the same stream slots as before (the gate precedes them,
the scale multiplies after). The witness
`s1_e_21h_d76_sweep_the_volume_scale_is_draw_free` (`audio.rs`) pins that the
×0.6 / ×0.15 plays draw exactly the same gameplay draws as their volume-1
twins, and that a scaled out-of-range play is still draw-free.

**Deviation closed.** The d76 "post-gate-scale deferred" note is superseded
for these two sites. The 301 Stacked Discs' −0.33 `pitchOffset` arm stays
deferred (a pitch term, not a volume term — the queue's 5th slot already
carries it, but the 301 kind is unported).

---

## 2. The Kill()-tail non-bomb cue map (the d76 range's last four sites)

**The claim cross-check.** d77 (`origin/task/W4-d77-proj-sounds-3`, tip
`ccd30b55`) owns sites 241-437 (lines `:70978`-the `Kill()` tail) and ported
95 Kill()-tail cues via its `kill_tail_cue` death-hook map. Its map's kind
arms were read whole (`git show …:projectile.rs`): the lowest line it claims
is `:71112` (the 932 Fairy Queen shot). The d76 range ends at `:70958`, so
the four `PlaySound` sites at `:70865`/`:70904`/`:70936`/`:70958` are
**in-range and unclaimed by d77** — this lane ports them. (The d76 DEFERRED
list named them "Kill-tail 1020/1105/1085/1080/1084"; d77's pushed map does
not name any of those kinds.)

**The C#** (`Kill()` `:70642`, the arms before d77's `:71112`):

- `:70855-70901` — `type == 1020 || type == 1105` (Jim's Drone / RC Car):
  `:70863` stops the tracked hum (`GetActiveSound(SlotId.FromFloat(localAI[2]))
  ?.Stop()`, the 1020 only), then `:70865`
  `PlaySound(SoundID.Item62, base.position)` (id 2, style 62, at position),
  then the 15+5+3 dust census and the gore ring (`:70866-70900`).
- `:70902-70923` — `type == 1085` (Pretty Mirror): `:70904`
  `PlaySound(SoundID.Item106, base.Center)` (id 2, style 106, at Center),
  then the oldVelocity drag + the 10 dust + the 4 gore (`:70905-70914`),
  then the `brokenMirrorBadLuckTime = 25200` write (`:70921`, the
  player-rect-gated bad-luck state).
- `:70926-70955` — `type == 1080` (Bird Droppings): the stinky-buff
  `ApplyBuffTo(player, 120, 600)` (`:70933`, the player-rect-gated write),
  then `:70936` `PlaySound(SoundID.Item177, (int)Center.X, (int)Center.Y)`
  (id 2, style 177, the int-overload position), then the 30-dust census,
  then `active = false; return` (`:70953-70954`).
- `:70956-70975` — `type == 1084` (Torch God helper): `:70958`
  `PlaySound(SoundID.Item10, base.Center)` (id 2, style 10, at Center), then
  the 40-dust torch census (`:70959-70974`).

Every arm plays its cue BEFORE its own state writes (or has none), so the
death hook's PRE-hook position read is the C#'s play position.

**The port.** A new `ProjectilePool::kill_tail_cue(slot)`
(`projectile.rs:20177`) — the d77 `kill_tail_cue` shape, kinds disjoint from
d77's map:

```
JIMS_DRONE | REMOTE_CONTROL_CAR => (2, 62, pos.x, pos.y, 0.0, 1.0)  // :70865
PRETTY_MIRROR                   => (2, 106, center.x, center.y, …) // :70904
BIRD_DROPPINGS                  => (2, 177, center.x, center.y, …) // :70936
TORCH_GOD_HELPER                => (2, 10, center.x, center.y, …)  // :70958
```

It pushes straight into `pending_proj_sounds` (the death hook runs AFTER the
sub-step loop's `wants_sound` forward, so the intent cannot ride the slot
field). The call sits inside the `!died_out_of_world` gate of the `!active`
death hook (`projectile.rs:20045`), right after the channel-cancel — the
out-of-world guard's direct deactivate never reaches `Kill()`, so it never
cues (the S1-P-i23 precedent). The drain routes id 2 through the wave-A
façade (the volume term rides the §1 plumbing).

**The rows.** The five kinds were unrowed in the d76 base. This lane rows
them (`projectile/data.rs`) so the cue map can spawn them:

| kind | C# SetDefaults | row |
| --- | --- | --- |
| 1020 Jim's Drone | `:9659-9666` | 12×12, aiStyle 195, penetrate −1, stock tileCollide/3600 |
| 1080 Bird Droppings | `:10078-10086` | 10×10, aiStyle 200, alpha 255, penetrate −1, ignoreWater |
| 1084 Torch God helper | `:10115-10124` | 4×4, aiStyle 202, extraUpdates 2, timeLeft 300, penetrate −1, tileCollide false |
| 1085 Pretty Mirror | `:10125-10132` | 18×18, aiStyle 2, friendly, penetrate 1 |
| 1105 RC Car | `:10294-10302` | 26×26, aiStyle 205, friendly, tileCollide true |

The 1084/1105 row values are kept **identical to d21's** (`20fe4b3e`) so the
merge is conflict-free on those two rows. The 1085's aiStyle-2 rotation
ladder (`:23245`, `rot += direction * 0.4`) is already in the port's
`ai_style2`. The four unarmed styles (195/200/202/205) join the
`UNARMED_ROWED` census (`projectile.rs:23989`, the i25 documented-no-op
precedent) — their AI machines are the follow-ups (d21 ports 202/205's
bodies; 195/200 stay census rows). The cue lands on death regardless of the
AI arm.

**Bank.** `Item_62` joins `SFX_CUES` (`audio.rs:284`, the asset verified on
disk); `Item_106`/`Item_177`/`Item_10` were already registered.

**State halves deferred (the cue-first precedent).** The 1080's stinky-buff
write (`:70933`) and the 1085's bad-luck write (`:70921`) are player-state
writes the cue precedes — the cue lands; the buff/luck halves are the
player-state follow-up (the d77 "state-bearing death bursts stay unported,
the cue alone lands where it precedes every write" convention). The
1020/1105 tracked-hum Stop (`:70863`) is the standing D4 instance-pool
deviation. The dust/gore censuses stay skipped (the module's RNG note).

**Witness.** `s1_e_21h_d76_sweep_the_kill_tail_cue_map` (`projectile.rs`):
one queued intent per death at the pre-hook position, the payload's
id/style/volume exact, the 1020/1105 pair shares Item62, the
1085/1080/1084 cues ride Center, and a non-mapped kind (the bomb 28) queues
nothing. WRITTEN NOT RUN.

---

## 3. The now-rowed kinds (the d18/d19/d21 row pushes — cross-check)

The d76 DEFERRED "unported kinds" list named kinds with no row/arm. The
sibling lanes pushed rows since. Cross-checking each against the pushed
`origin/task/W4-d18-proj-ai-171-180` (`d7d631e9`), `…d19…` (`a4185f89`),
`…d21…` (`20fe4b3e`) tips:

| kind | cue site | rowed by | cue armed by | d76 action |
| --- | --- | --- | --- | --- |
| 984 NightsEdge | `:41978` Item60 ×0.65 | d19 | d19 `ai_nights_edge` (`wants_sound`) | none — d19 owns it (the ×0.65 scale is d19's deferred-scale note; the §1 plumbing now supports it, but the arm is d19's) |
| 933/1100 FinalFractal | `:43120` Item169 | d19 | d19 `ai_final_fractal` | none — d19 owns it |
| 949 BadTorch | `:42796` Item8 | d19 | d19 `ai_bad_torch_luck` | none — d19 owns it |
| 918 FireExplosion | `:45271` Item14 | d18 | d18 `ai_fire_explosion` | none — d18 owns it |
| 1084 Torch God helper | `:70958` Item10 | d21 | **unclaimed** (d21 ports the AI, not the death cue) | **ported** (§2) |
| 1105 RC Car | `:70865` Item62 | d21 | **unclaimed** | **ported** (§2) |
| 1020 Jim's Drone | `:70865` Item62 | d76 (§2) | d76 | **ported** (§2) |
| 1080 Bird Droppings | `:70936` Item177 | d76 (§2) | d76 | **ported** (§2) |
| 1085 Pretty Mirror | `:70904` Item106 | d76 (§2) | d76 | **ported** (§2) |

The d18/d19 cue arms were read whole (`git show …:projectile_ai_misc.rs` /
`…:projectile_ai_sword_styles.rs`): each queues its cue on the
`wants_sound` seam (the 5-tuple form on those branches — the §1 widening
will need a merge-time reconciliation, but the cue id/style/position are
already correct there). **No now-rowed kind's cue is left unclaimed except
the five Kill()-tail kinds, which §2 ports.**

**Still deferred (the arm is the prerequisite).** The remaining d76
DEFERRED kinds whose cues sit INSIDE an unported AI machine cannot land
without that machine:

- 864 Smolstar `:45778` (the Dig cue inside `AI_169_Smolstars`, aiStyle 169
  — `UNARMED_ROWED`, the i25 documented no-op). Rowed in base, but the arm
  is unported; the cue is not a death-hook cue, so §2's map cannot carry it.
- 75/102 hostile bombs `:46941`/`:47131` — unrowed, aiStyle 16 hostile; the
  `:46941` cue is `PlaySound(66)` (the zombie-voice, the non-id-2 E21e
  table), the `:47131` cue is Item10 inside the hostile-bomb arm. The port's
  `ai_bomb` handles only the friendly thrown-bomb/rocket kinds, not these.
- The rest of the d76 DEFERRED list (535 Medusa, Rapier, Kite, 653
  CompanionCube, 472, 467/468, 437/435, 174, 605/920/921/926/937, 176, 408,
  302, 593/592, 257, 1073, 195, 576/577, 374/376, 585, 587, 572, 811/814/676,
  299, 498, 439, 600/735/927, 611, 615/630/705, DD2Pet) — none rowed by
  d18/d19/d21 (verified by grep of the three pushed `data.rs` tips); their
  cues stay deferred until their kinds row + arm.

---

## 4. The dead `_Old` arms (verified truly dead)

The d76 DEFERRED list named two dead `_Old` arms. Verified by zero-caller
grep (whole `Projectile.cs` + the top-level `Terraria/*.cs` and
`GameContent/*.cs`, direct paths only — no junction recursion):

- **`AI_009_MagicMissiles_Old`** (`:43427`). Its ONLY caller is `:43199`,
  inside `AI_009_MagicMissiles`'s `if (this.type == 491)` gate. The 491
  (the old Magic Missile variant) is **unrowed** in the port, so the arm
  never runs. Its two cue sites — `:43589` (`type == 79`, Item9) and
  `:43606` (`type == 16`, Item9) — are **dead**: only 491 routes to `_Old`,
  and 79/16 take the NEW `AI_009` path. The 16's live Item9 whoosh is the
  landed `:43361-43367` cue (the port's `ai_magic_missile`,
  `projectile.rs:8211`); the `_Old` `:43606` is a dead duplicate. The 79 is
  unrowed. **No port action** — the port never ported `_Old` (grep empty).
- **`AI_015_HandleMovementCollision_Old`** (`:44171`). **ZERO callers** —
  the only `HandleMovementCollision` call (`:19323`) targets the NEW
  `AI_015_HandleMovementCollision` (`:43655`). Its cue `:44197`
  (`PlaySound(0, (int)position.X, (int)position.Y)`, the Dig thud) is dead.
  **No port action.**
- **`AI_015_Flails_Old`** (`:44205`, bonus find). **ZERO callers** — dead.
  Not in the d76 deferred list, but the same `_Old` family; recorded here
  for completeness. **No port action.**

The port carries none of the `_Old` arms (the grep for
`magic_missiles_old|handle_movement_collision_old|flails_old` over `crates/`
is empty), so there is nothing to remove — the verification confirms the
d76 "dead `_Old` arms" classification was correct, and the cues are not
missing ports.

---

## Verification

Code-only: every ported arm re-read against its C# block (the whip tip's
`getActualCollisionPoints = false` curve + the ×0.6 fourth argument, the
Lights Bane post-play `0.15 * ai[0]`, the four Kill()-tail arms' pre-write
cue positions, the five SetDefaults rows). The sibling-branch claims read
whole from the pushed tips (d77's `kill_tail_cue` kind map, d18/d19's cue
arms, d21's rows). The dead `_Old` arms verified by zero-caller grep. The
`Item_62` asset confirmed on disk. The two new witnesses
(`s1_e_21h_d76_sweep_the_volume_scale_is_draw_free`,
`s1_e_21h_d76_sweep_the_kill_tail_cue_map`) plus the updated whip-tip /
Lights Bane / queue-tuple asserts are WRITTEN, NOT RUN — **no compiler
invoked**. Anchors rows added for `play_use_sound_scaled` and
`kill_tail_cue` (the build canary markers verified present).
