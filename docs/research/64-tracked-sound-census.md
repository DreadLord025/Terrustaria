# 64 — Tracked-sound entity-loop census (W4-d80)

Scope: the per-entity tracked-sound call sites the S1-E-23 layer
(`tr-app/src/tracked_sound.rs`, `SoundPlayer.cs:1-132` / `ActiveSound.cs:1-166`)
left unwired — the drill/chainsaw held-tool loops, the DD2 boss loops
(Betsy / lane portal / kobold / wither beast), the bomb fuse, and the
vampire sun-scorch sizzle. The layer itself, the blizzard loops
(`SceneState.cs:197-247`) and the achievement toast
(`AchievementInitializer.cs:731-734`) already landed (S1-E-23 / S1-E-26a).

Naming note: the task brief's "Main trackedSounds" is
`SoundPlayer._trackedSounds` (`SoundPlayer.cs:9`, the 4096-slot
`SlotVector<ActiveSound>`); `Main.cs` itself holds no tracked-sound fields
in 1.4.5.8 — the pool hangs off `SoundEngine.SoundPlayer`.

## RNG contract (the census' core claim)

**No tracked play draws `Main.rand`.** Every `PlayTrackedSound` /
`PlayTrackedLoopedSound` consumes exactly two AUDIO-ONLY static streams:

1. `LegacySoundStyle.Random.Next(_style, _style + Variations)` — the
   per-play variation pick (`LegacySoundStyle.cs:21-28`), drawn ONLY when
   `Variations != 1`. The port's stand-in: `Audio::style_random`
   (seeded 1, S1-E-23 ANALYSIS).
2. `SoundStyle._random.NextFloat()` — `GetRandomPitch`
   (`SoundStyle.cs:40-43`), drawn ALWAYS (even at variance 0, the draw
   still happens: `NextFloat() * 0 - 0 * 0.5`). The port's stand-in:
   `Audio::sound_style_random` (the second seeded-1 stream).

The `ActiveSound` ctors (`ActiveSound.cs:45-101`) run `GetRandomSound()`
(the variation pick) then `Pitch += GetRandomPitch()` then
`DetermineIntendedVolume()` — no gameplay draws, no position draws. The
`MaxTrackedInstances` cap (`SoundEngine.cs:126-129`) and the 10 000-px
screen-centre guard (`SoundPlayer.cs:17-20`/`:31-34`) refuse BEFORE either
draw. Consequence: wiring any site below cannot move a gameplay RNG
stream; the witnesses pin that (a muted/audible tracked play leaves every
`RngKey` stream pristine).

The drill/chainsaw cues are NOT tracked plays — they are legacy case-2
item plays on a `soundDelay` cadence (see below); their one `Main.rand`
pitch draw rides the `UpdateProjectiles` stream (`Main.cs:17775`) AFTER
the attenuation gate (`LegacySoundPlayer.cs:390-436` precedes `:516`).

## Site census

| # | Site | C# anchor | Style | Slot storage | Cadence / gate | Draws (gameplay) |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Drill loop (aiStyle 20, `pick > 0` held) | `Projectile.cs:25510-25516` | `SoundID.Item190` (2,190) | `soundDelay = 15` | `soundDelay <= 0`, per sub-step, BEFORE the channel gate | case-2 default `Next(-6,7)*0.01` ×1 |
| 2 | Chainsaw loop (aiStyle 20, else) | `Projectile.cs:25517-25521` | `SoundID.Item189` (2,189) | `soundDelay = 20` | same | same ×1 |
| 3 | Laser Drill 445 (aiStyle 75's 445 arm) | `Projectile.cs:66246-66250` | `SoundID.Item132` (2,132) | `soundDelay = 23` | `soundDelay <= 0` | case-2 132 arm `Next(-20,21)*0.001` ×1 |
| 4 | Betsy wind attack (phase 2 entry) | `NPC.cs:61980-61983` | `DD2_BetsyWindAttack` (42, var 3) | none (one-shot) | `ai[0]==2 && ai[1]==0` | 0 |
| 5 | Betsy flame breath (phase 3 dash) | `NPC.cs:62028` | `DD2_BetsyFlameBreath` (42, var 1) | `localAI[2]` float dance | `ai[1]==num13` (40) | 0 |
| 6 | Betsy breath position-follow | `NPC.cs:61993-61997` | — | `localAI[2]` | every phase-3 tick, `Position = Center` | 0 |
| 7 | Betsy dead-handle drop | `NPC.cs:62302-62305` | — | `localAI[2] = Invalid` | AI tail, `GetActiveSound == null` | 0 |
| 8 | Portal open | `NPC.cs:40979-40983` | `DD2_EtherianPortalOpen` (42, var 1) | `localAI[3] = Invalid` reset | first tick (`localAI[0]==0`, `ai[1]==0`) | 0 |
| 9 | Portal idle loop ensure | `NPC.cs:40984-40987` | `DD2_EtherianPortalIdleLoop` (42, var 1) | `localAI[3]` | `localAI[0] > 150 && GetActiveSound == null` | 0 |
| 10 | Portal fade ensure + volume | `NPC.cs:41029-41037` | same | `localAI[3]` | phase 2: ensure, `Volume = scale` | 0 |
| 11 | Portal death stop | `NPC.cs:41044` | — | — | `ai[0] >= 550`: `activeSound?.Stop()` | 0 |
| 12 | Kobold handle reset | `NPC.cs:63198-63202` | — | `localAI[0]/[1] = Invalid` | `localAI[3]==0` (spawn-in ramp) | 0 |
| 13 | Kobold explode stops | `NPC.cs:63206-63207` | — | — | `ai[1]==2` entry: both `?.Stop()` | 0 |
| 14 | Kobold explosion | `NPC.cs:63256` | `DD2_KoboldExplosion` (42, var 3) | none | `ai[0] >= 3` (the death tick) | 0 |
| 15 | Kobold ignite | `NPC.cs:63263-63268` | `DD2_KoboldIgnite` (42, var 1) | `localAI[0]` | `ai[0]>0 && ai[1]==0 && life>0 && !SlotId.IsValid` | 0 |
| 16 | Kobold ignite+scream loops | `NPC.cs:63269-63274` | `DD2_KoboldIgniteLoop` / `DD2_KoboldScreamChargeLoop` (42, var 1) | `localAI[0]` / `localAI[1]` | `localAI[2] == 30` (the ignite timer) | 0 |
| 17 | Kobold loops ensure-or-follow | `NPC.cs:63288-63307` | same | same | `ai[0]==0 && ai[1]==1`: null → play, else `Position = Center` | 0 |
| 18 | Wither beast handle reset | `NPC.cs:63362-63365` | — | `localAI[1] = Invalid` | `ai[0]==0` | 0 |
| 19 | Wither beast aura pulse | `NPC.cs:63380-63388` | `DD2_WitherBeastAuraPulse` (42, var 2) | `localAI[1]` | the `ai[0]==1` charge tick: null → play, else follow | 0 |
| 20 | Bomb fuse loop | `Projectile.cs:46917-46921` | `BombFuse` (42, var 1, vol 0.2) | `localAI[2]` | `GetActiveSound == null && timeLeft > 1 && IsABombWithFuse[type]` | 0 |
| 21 | Bomb fuse death stop | `Projectile.cs:70673-70677` | — | `localAI[2] = 0` | `Kill()`, aiStyle 16, `netMode != 2` | 0 |
| 22 | Vampire sizzle ensure | `Player.cs:28590-28598` | `VampireSizzle` (42, var 1, vol 1) | `_sizzleAudioHandle` | `GetActiveSound == null && lerp != 0`, `Volume = lerp` override | 0 |
| 23 | Sizzle follow | `Player.cs:28599-28603` | — | — | every tick: `Volume = lerp; Position = Center` | 0 |
| 24 | Sizzle counter | `Player.cs:28582-28589` | — | `sunScorchCounter` | `Clamp(counter + (burning ? 1 : -num), 0, 300)`, `num = 6` (dead: 2 + burning cleared); `lerp = GetLerpValue(0,120,counter,clamped)` | 0 |

Deferred (owning AI unported, the layer is ready):

| Site | C# anchor | Why deferred |
| --- | --- | --- |
| RC car engine loop | `Projectile.cs:38253-38267` | the RC-car AI (volume/pitch remap machine) has no port arm |
| Jim's Drone loop | `Projectile.cs:40983-40997` | the 1020 drone AI has no port arm (id-table row only) |
| DD2 projectile one-shots (towers, phoenix, book staff…) | `Projectile.cs:54717`/`:55776`/`:68465`/… | ride their owning projectile-AI waves (W4-d75..77 census) |
| ParticleOrchestrator tracked plays | `ParticleOrchestrator.cs:1888`/`:2123`/`:2427` | the particle system is unported |

## Conditions (the looped plays' stop gates)

- `ProjectileAudioTracker.IsActiveAndInGame`
  (`ProjectileAudioTracker.cs:15-27`): `!Main.gameMenu &&
  Main.projectile[index].active && type == expectedType` — the bomb fuse
  (site 20) stops the instant its slot dies or is recycled into another
  kind. The port folds the closure to an explicit stop at the slot's
  death/detonation (the pool cannot hand a live projectile reference to a
  `Send` closure); the `gameMenu` term folds with it (the focus/pause gate
  already freezes the pool). Recorded in ANALYSIS.
- `VampireSizzleTracker.IsActiveAndInGame`
  (`VampireSizzleTracker.cs:12-19`): `!Main.gameMenu && Main.vampireSeed &&
  player.sunScorchCounter > 0` — the port mirrors the counter into a
  shared atomic the closure reads; the seed term gates the ensure-play
  (the counter only rises under the seed, `Player.cs:28676`).
- The DD2 "loops" (sites 9/16/17/19) are plain tracked plays of loop-named
  assets in the C# — NOT `PlayTrackedLoopedSound`; they play out once and
  the ensure-arm re-plays them when the slot died. The port keeps that
  exactly (no `IsLooped`, no condition).

## Port folds recorded by this wave

1. The NPC AI machines have no `Audio` borrow — the tracked ops queue as
   `NpcSoundCue` intents (the `wants_sound` convention) and the sim drain
   applies them against the pool right after the machine's draws, writing
   the `SlotId` floats back into `Npc.local_ai` (the C# `localAI` dance).
2. The wither-beast aura ensure-or-follow fires on every aura-active tick
   in the port (the port's `ai[0] != 0` branch) instead of the C#'s single
   `ai[0]==1` charge tick — the ensure gate makes the overlap identical
   while the pulse lives; the re-play cadence after it dies is the fold.
3. The kobold `ai[0]==0 && ai[1]==1` ensure-or-follow state is unreachable
   in the port's folded walker (the `ai[0]==1 → ai[1]=2` latch is
   immediate, S1-D-3) — the block is ported verbatim and stays inert.
4. The bomb-fuse condition fold (above). Site 20's trailing
   `GetActiveSound(...)` (`Projectile.cs:46920`) is a dead read — dropped.
5. The drill cadence reads the owner's held item through the port's
   `channel_item` snapshot (the C# reads
   `Main.player[owner].inventory[selectedItem].pick`,
   `Projectile.cs:25512`); the held tool IS the channel item while the
   drill lives.

## Landing status (W4-d80, 2026-11-21)

All 24 census sites landed across five phases on
`task/W4-d80-tracked-sound-layer` (code-only mode — tests written NOT run):

* **Phase 2** (`fb945c08`): sites 1-3 — the drill/chainsaw cadence cues +
  the Laser Drill 445's Item132/23 (`Projectile::wants_sound`, the
  `soundDelay` re-arms, the cue before the channel gate).
* **Phase 3** (`4d28b70d`): sites 4-19 — the DD2 NPC loops. `NpcSoundCue`
  gained the tracked arms; `world_sim/npcs.rs::apply_npc_tracked_cue`
  applies them and writes the `SlotId` floats back into `Npc.local_ai`.
  `TrackedStyle::by_name` resolves the styles through `soundid.rs`. The
  kobold's `localAI[3]` spawn-in counter now ramps 0 → 60 (`NPC.cs:63769`)
  so the `== 0` handle-reset fires once (fold 3's inert ensure-or-follow
  stays verbatim).
* **Phase 4** (`1f859218`): sites 20-21 — the bomb fuse.
  `ProjTrackedCue`/`wants_tracked`/`fuse_alive` +
  `world_sim/projectiles.rs::apply_proj_tracked_cue`; the
  `ProjectileAudioTracker` condition folds to a shared `Arc<AtomicBool>`
  the death stop flips false (fold 4). `is_bomb_with_fuse` ports the
  verbatim `ProjectileID.cs:424` set.
* **Phase 5** (`84cbb1ff`): sites 22-24 — the vampire sizzle.
  `Player.sun_scorch_counter`/`sizzle_audio_handle`/`sizzle_alive` +
  `tick_sun_scorch`/`sun_scorch_lerp`; the counter ticks on BOTH the live
  (`UpdateSunScorch` `:25634`) and dead (`UpdateDead` `:17079`) paths;
  `world_sim/player.rs::update_sizzle_audio` owns the play/follow. The
  `VampireSizzleTracker` condition folds to a shared atomic mirroring
  `vampireSeed && counter > 0`.

The RNG contract holds: every tracked play draws ONLY the two audio-only
statics (`style_random`/`sound_style_random`); the witnesses pin the
gameplay streams pristine. The deviations (the shared-atomic condition
folds, the wither/kobold cadence folds, the unported burning granter) are
recorded in ANALYSIS "The tracked-sound entity loops (W4-d80)".