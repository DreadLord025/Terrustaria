# 111 — DOC-SWEEP screen-fx lane (W4-d70, 2026-11-21)

Code-only DOC-SWEEP continuation of `task/W4-d70-moonlord-shake-screens`
(base `8f9961d8`, the five screen-effect draw rows + the MoonLordShake Apply
half). This lane ports the REAL gaps the base commit recorded as deviations,
censuses the remaining screen shaders, and writes the witnesses. No cargo /
tests / graphify were run (code-only lane); every test below is **written,
not run**.

Scope: the four deviations the Lead named —
1. the PaladinsShield peer loops (PlayerNet team/shield fields),
2. the LensFlare occlusion mask + shadow offset,
3. the other screen shaders census (Shockwave / HeatDistortion / Ripple),
4. the DrawWhite flash (death/teleport white-flash producers).

---

## 1. The paladin peer loops — LIVE (was: no-op until PlayerNet carries state)

**Gap.** `Main.DrawPaladinsShield` (`Main.cs:57196-57232`) walks
`Main.player[0..255]` (skipping `Main.myPlayer`) twice: the local shield
defends each same-team shield-bearing peer, and a `defendedByPaladin` local
player is reached by peers who `CanDefendWithPaladinsShield`. The base commit
ported both loops over a `PaladinPeer` snapshot but passed `&[]` because
`PlayerNet` carried no team/shield/dead state.

**Port.** `c28 ClientClone` landed the remote-player snapshots; this lane
wires the loops to them.

- `PlayerNet.team` / `PlayerNet.dead` (`net.rs`): fed by
  - `12 PlayerSpawn` (`MessageBuffer.cs:914-918` — the team byte + the
    `respawnTimer > 0 → dead` arm), applied in the host's `C2s::PlayerSpawn`
    arm AFTER the join insert (which resets the entry);
  - `45 TeamChange` (`MessageBuffer.cs:2325-2336`) — the host apply/relay arm
    and the client read arm land (both were no-op stubs); the team-click send
    (`Main.cs:42764-42767`) rides the `45` body (the `157` team-spawn tail is
    the unported seed);
  - the `13 PlayerState` port extension (team/dead appended to the two-client
    render tail, write/read symmetric) — the host/client `13` arms store them,
    the client's change-driven send test covers them, and `SyncOnePlayer`'s
    `45` row (`NetMessage.cs:2903`) rides the join bundle.
- `PlayerNet::paladin_shield()` (`net.rs`): re-derives `hasPaladinShield` from
  the synced armour block — a worn Paladin's/Frozen/Hero shield
  (938/3997/3998, `ApplyEquipFunctional` `Player.cs:15348-15353`) in a
  functional slot (3..10, the `UpdateEquips` loop `:12946-12952`) while above
  25% life (`statLife > statLifeMax2 * 0.25f`, strict `>`).
- The local armour block's `5 PlayerInventory` diff on the net cadence tick
  (`App.net_armor_last`): the C# sends on every equip change
  (`PlayerItemSlotID.CanRelay` covers the armour block,
  `PlayerItemSlotID.cs:168-186`); the port batches it. The host broadcasts +
  stores slot 0's copy; the client sends through `case 5`'s `CanRelay` arm.
- `draw.rs`: the peer slice is the `remote_players` snapshots in slot order
  (`mounted_center` from `x/y + WIDTH/HEIGHT`, the shield re-derivation).

**Deviation (recorded in ANALYSIS).** The port does not simulate remote
equips, so the shield flag is re-derived from the `5 PlayerInventory` copies
rather than read off the peer's own live `hasPaladinShield`; the armour `5`
diff batches on the cadence tick instead of the C#'s per-change send. The
`45` chat announcement (`MessageBuffer.cs:2343-2353`) is unported (no MP chat).

**Witnesses (written, not run).**
- `net.rs::the_paladin_shield_derives_from_the_synced_armour` — the
  938/3997/3998 grant, the strict quarter-life gate, the functional-slot gate.
- `net.rs::the_player_state_extension_carries_team_and_dead` — the `13`
  extension round-trip.
- `netglue.rs::the_paladin_peer_fields_ride_the_snapshot_and_the_inventory` —
  the loopback wire: team/dead ride the snapshot, the shield re-derives from
  the streamed armour, the quarter-life gate drops it.
- `netglue.rs::a_team_change_relays_to_the_host_peer_copy` — the `45` relay.

---

## 2. The lens flare — shadow offset + sun-scorch LIVE; occlusion mask CENSUSED

**Gap.** The base commit's `draw_lens_flare` drew at full temporal intensity
with shadow offset 0 and `sunScorchCounter` 0.

### 2a. The advanced-shadow offset — PORTED (self-contained)

`LensFlareElement.Draw` (`LensFlareElement.cs:34-37`) reads
`GetAdvancedShadow(0).Position - GetAdvancedShadow(min(4, count-1)).Position`
— the player's last ~4 ticks of movement — then
`Dot(v.SafeNormalize(UnitX), (sun - center).SafeNormalize(-UnitY)) * v.Length()`,
shifting the lerp fraction by `-0.0002 * num` (`:42`).

- `Player._advancedShadows` (`Player.cs:1898-1902`, MaxAdvancedShadows = 60):
  the port keeps only `Position` (the `EntityShadowInfo` rotation/origin/
  direction/frame fields feed the unported advanced-shadow DRAW layers,
  `PlayerDrawLayers.cs:2643+`). `availableAdvancedShadowsCount` +
  `_lastAddedAvancedShadow` (the C# typo) are the fill + write head.
- `update_advanced_shadows` (`Player.cs:4328-4340`) at `Player.Update`'s tail
  (`:28303`); `get_advanced_shadow` (`:4318-4326`, the `>` clamp kept
  verbatim, `ModulusPositive(60)`); `reset_advanced_shadows` (`:4342-4345`)
  on the teleport (`:37483`), spawn (`:37642`) and the `TR_XY` hook.
- `screen_fx.rs::lens_shadow_offset` projects the trail onto the sun axis;
  `draw_lens_flare` takes the local player (`None` = TR_NOPLAYER keeps 0).

### 2b. The sun-scorch override — PORTED (the vampire seed chain)

`AdjustIntensity` (`NextHorizonRenderer.cs:392-397`) overrides the cubed
intensity when `sunScorchCounter > 0`. The counter's source is the vampire
seed's sun-scorch chain (`Player.cs:28580-28719`), previously unported:

- `sun_scorch_counter` (0..300): `UpdateSunScorchValues` (`:28588`) ramps +1
  while burning, −6 (−2 dead) otherwise, clamped.
- `vampire_seed_sunlight_exposure` (`:28672-28719`): the 15-tile column probe
  from the feet — wall 0/21/318 burns, `SolidTile3` blocks (the loosest solid
  test, `stray.rs::solid_tile3`'s shape), the 946/4707 umbrella + the UFO
  mount (56) shield, the graveyard/glowshroom/MoonLordSky>0.5 gates.
- `update_sun_scorch` (`:28626-28670`) at the `UpdateEquips` seam (`:25634`):
  the ≥120 burn wipes `buffImmune`, grants 24/23/32 (3 ticks), dismounts, and
  strips wings/rockets.
- `zone_graveyard` + `moon_lord_sky_intensity` fed from the scene scan
  (`SceneMetrics.cs:299-310`, the closest core 398, `1 - SmoothStep(3000,
  6000, dist)`).
- The HeatDistortion branches (`SceneState.cs:98,146`) read the live counter
  through `SceneVisualInputs.sun_scorch_counter` (replaces the 0 constant).

**Deviations (recorded in ANALYSIS).** The exposure probe's invisible-block/
wall terms (`Tile.cs:528-559`) fold to false (no port bit); the sizzle-loop
sound (`:28590-28603`), the charred-frame visual swap (`:28639-28652`) and the
VampireOnFire particle (`:28656-28659`) are unmodelled draw/audio seams.

### 2c. The sun-occlusion pixel mask — CENSUSED, NOT ported (not a tile probe)

`HorizonHelper.UpdateSunVisibility` (`HorizonHelper.cs:35-69`) is a **GPU
render-target readback**, not a self-contained tile probe:
1. it samples a 128×128-px area around `LastCelestialBodyPosition` out of the
   frame's `Main.screenTarget` into a 64×64 mip-mapped Alpha8 `_tinyTarget`;
2. it blends that down to a 1×1 Alpha8 `_pixelTarget` with the
   `InverseSourceAlpha` horizon blend state;
3. `SunVisibilityEnabled` is `_targetUpToDate` (true only when
   `Main.ForegroundSunlightEffects` and a target exists), and the
   `GameShaders.Misc["LensFlare"]` pass samples `_pixelTarget` as `uImage1`
   (`NextHorizonRenderer.cs:378`) to modulate the flare by the sun's occlusion.

The Lead's port condition was "if self-contained tile-probe". It is not: the
mask reads the rendered frame's alpha (the sky/tile composite), so a faithful
port needs the `screenTarget` render-target family + a downsample/readback
pass. **The d69 co-lane (`task/W4-d69-layer-target-consumers`, S1-G-13) is
building exactly that infrastructure** (`layer_targets.rs`, `targets.rs`,
`RenderToTargets`, the `skyTarget` gate, `composite.rs`); the occlusion mask
is a consumer of it, deferred to that seam. The flare keeps drawing at full
temporal intensity (`SunVisibilityEnabled` assumed true) — the recorded
deviation stands, now with a named dependency.

**Witnesses (written, not run).**
- `screen_fx.rs::the_lens_shadow_offset_projects_the_trail_onto_the_sun_axis`
  — still → 0, toward-sun → +len, perpendicular → 0.
- `screen_fx.rs::the_advanced_shadow_ring_caps_and_resets` — the 60 cap, the
  index clamp, the reset.
- `player.rs::the_sun_scorch_counter_ramps_and_clamps` — the +1/−6/−2 ramps,
  the 0..300 clamp, the dead flag clear.
- `player.rs::the_vampire_exposure_probe_gates_on_seed_sky_and_shelter` — the
  seed/sky/rain/umbrella/wall gates.

---

## 3. The other screen shaders — CENSUS

The Lead named Shockwave / HeatDistortion / Ripple. The full
`ScreenEffectInitializer.Load` registry (`ScreenEffectInitializer.cs:14-48`)
is 29 `Filters.Scene`/`Overlays.Scene` rows; `post.rs::PASS_TABLE`
(`:140-179`) mirrors all 29. The census splits each row into (a) the WGSL
pass, (b) the activation reader, (c) the parameter formula.

### 3a. HeatDistortion — PORTED (the Lead's "not ported" premise is stale)

The `filters.rs` module doc claimed "HeatDistortion/WaterDistortion/Aurora
are phase-4/sky-manager work" — **stale**. Phase 4 landed them:
- the board slot `FilterId::HeatDistortion` (`EffectPriority.Low`);
- the activation `heat_distortion_active` (`SceneState.cs:98-99`:
  `UseHeatDistortion & (underworld | desert | sunScorch > 0)`), driven by
  `heat_zone_flags` (`:79-80`);
- the intensity `heat_distortion_intensity` (`:145-160`), `IsHidden =
  intensity <= 0` (`:161`);
- WGSL pass 8 (`post.wgsl::heat_uv`), the noise-driven uv offset.
The compiled `FilterHeatDistortion` HLSL amplitude is a visual approximation
(`HEAT_AMPLITUDE_PX = 1.5`, D-12). **W4-d70 DOC-SWEEP:** the branches read the
live `sun_scorch_counter` (was the 0 constant). The module doc is corrected.

### 3b. Ripple / WaterDistortion — the filter PORTED, the entity-wave buffer NOT

`WaterShaderData` (`WaterShaderData.cs`) is two halves:
- **The screen filter** (`FilterWaterDistortion`, `ScreenEffectInitializer.cs:41`):
  PORTED — the board slot (`EffectPriority.VeryHigh`), the activation
  `water_distortion_active` (`SceneState.cs:105`, `WaveQuality > 0`), the
  intensity `water_distortion_intensity` (`:128-132`), the `_progress` clock
  (`WaterShaderData.cs:126-127`, `WaterShader::update`), WGSL pass 9
  (`water_uv`) + the liquid-coverage water mask (`water_mask.rs`,
  `liquids.rs::water_mask_rects`). The compiled HLSL amplitude is a visual
  approximation (`WATER_AMPLITUDE_PX = 2.0`, D-12).
- **The entity-wave/ripple buffer** (`DrawWaves`/`StepLiquids`/`QueueRipple`,
  `WaterShaderData.cs:136-333,445-469`): NOT ported. This is the
  `_distortionTarget`/`_distortionTargetSwap` ping-pong that draws NPC/player/
  projectile wave quads + the `_rippleQueue` (the `RippleShape` Square/Circle
  source rects, `RippleShape.cs`) into a 0.25-scale buffer, the
  `WaterProcessor` misc shader, and the viscosity mask chain. The
  `QueueRipple` call sites (`Gore.cs:967`, `Projectile.cs:31754`,
  `FakeFishParticle.cs:185`, the NPC/player `wetCount` arms
  `WaterShaderData.cs:230,264`) have no port producer. The port's mask is the
  static liquid coverage, so `target_pos`/`image_offset` stay zero
  (`filters.rs:461-465`, the recorded deviation). **This is the real
  WaterDistortion gap** — a render-target + wave-quad system, not a tile probe.

### 3c. Shockwave — NOT a screen shader (a particle orchestra)

There is no `Filters.Scene["Shockwave"]` row. The only "Shockwave" in the
source is `ParticleOrchestraType.PaladinsHammerShockwave`
(`ParticleOrchestraType.cs:73`), spawned by the Paladin's Hammer projectile
(301) on tile hit (`Projectile.cs:19539`) via
`ParticleOrchestrator.Spawn_PaladinsHammerShockwave`
(`ParticleOrchestrator.cs:815-879`) — a dust/gore burst (57/43 dust, the
±3-tile `KillTile_MakeTileDust` fan, the 61-63 gore), NOT a screen filter.
The port has no particle orchestration (`netmodules.rs:447-450` — the
`Particles` net module round-trips and drops; the 301 projectile's
`aiStyle-3` tile-hit arm has no `RequestParticleSpawn` producer). This is a
dust/gore/particle-system gap, out of the screen-shader lane's scope; recorded
here so the "Shockwave" name is not mistaken for a missing filter.

### 3d. The full PASS_TABLE census (29 rows)

| Row(s) | WGSL pass | Activation reader | Status |
| --- | --- | --- | --- |
| Graveyard | 1 | `graveyard_visual_intensity > 0` (`SceneState.cs:100`) | LIVE |
| BloodMoon | 3 | `bloodMoon \|\| BloodMoonMonolith` (`:92`) | LIVE (the `bloodMoonMonolithShader` accessory term folds false) |
| Noir | 2 | **NONE** — `noir_monolith` scanned (`scene.rs:685`) but no board manage | pass + formula ported, reader MISSING |
| Sandstorm + overlay | 4/5 | `UseStormEffects && ShowSandstormVisuals` (`:97`) | LIVE |
| Blizzard + overlay | 6/7 | `UseStormEffects & (ZoneRain && ZoneSnow)` (`:96`) | LIVE |
| HeatDistortion | 8 | `UseHeatDistortion & (underworld\|desert\|sunScorch)` (`:98-99`) | LIVE (sun-scorch now live) |
| WaterDistortion | 9 | `WaveQuality > 0` (`:105`) | filter LIVE, entity-wave buffer MISSING |
| MoonLordShake | 10 | the ritual countdown (`Main.cs:64976-64984`) | LIVE (S1-G-6 + W4-d70 Apply half) |
| Nebula/Solar/Stardust/Vortex | 0 | `CloseEnoughTo*Tower` (`:81-84`) — no reader | inert (tower sky + filter both M4) |
| MonolithNebula/Solar/Stardust/Vortex | 0 | `ActiveMonolithType` + `*MonolithShader` (`:87-90`) — no reader | inert |
| MoonLord / MonolithMoonLord | 0 | `ClosestNPCPosition[398] != 0` / monolith (`:85,91`) — no reader | inert (the `FilterMoonLord` tint; the shake is the live row) |
| Sepia | 0 | `onlyDontStarveWorld ^ (dontStarveShader \|\| RadioThingMonolith)` (`:101`) — no reader | inert (the `SepiaScreenShaderData.Update` formula unported) |
| testInvert / Test / Test2 / Test3 / CRT | 0 | debug/monolith (`:103-104` for CRT/Test2) — no reader | inert (test content) |
| CrystalDestructionColor / Vortex / CrystalWin | 0 | the DD2 crystal win/loss scenes (`Main.cs:23996-24004`, `Projectile.cs:36346-36357`) — no reader | inert (the 672/713 scene projectiles unspawned) |

**Count:** 10 rows carry a real WGSL pass, 19 resolve to passthrough 0. The
research/41 D-13 row's old "Twenty of 29" is now 19 (MoonLordShake went live).
The activation readers are a SEPARATE gap from the WGSL passes: even a ported
pass (Noir) stays inert without its `ManageSpecialBiomeVisuals` reader.

**The accessory shader grants** (`Player.ApplyShader`, `Player.cs:14246-14283`:
3536-3539/4318/4054/5347/5598/5599/5655/5113 → the `*MonolithShader` /
`dontStarveShader` / `noirShader` / `CRTMonolithShader` / `retroMonolithShader`
flags) are unported — no `Player` field, no `ApplyEquipFunctional`/
`ApplyEquipVanity` arm. They are the `perspectivePlayer.*` terms of the
monolith/Sepia/Noir/CRT/Test2 activation conditions. The placed-monolith
tile metrics (`active_monolith_type`, `*_monolith`, `shimmer_monolith_state`)
ARE scanned; the accessory half is the gap.

### 3e. The ShimmerMonolithState terms — PORTED (a real gap found in the census)

`SceneState.UpdateShimmer` (`SceneState.cs:285-289`) reads
`ShimmerMonolithState == 1` (trigger) / `== 2` (suppress). The port scanned
`shimmer_monolith_state` (`scene.rs:682`) but `update_shimmer` ignored it.
This lane ports the two terms (state 1 triggers the ramps without the zone,
state 2 suppresses every trigger). The `perspectivePlayer.shimmerMonolithShader`
term folds to false (the accessory grant above is unported).

**Witness (written, not run).**
`scene.rs::the_shimmer_monolith_state_gates_the_ramps`.

### 3f. The sky-manager visuals (not filters) — census note

`ScreenEffectInitializer.LoadSkies` (`:54-76`) registers 18 `SkyManager`
entries. Ported: `AmbientSky` (`ambience.rs`). Unported: the four tower skies
(Nebula/Solar/Stardust/Vortex), the four monolith tower skies, MoonLordSky
(+ MonolithMoonLord), PartySky, MartianSky, SlimeSky, CreditsRollSky,
AuroraSky, SandstormSky, BlizzardSky, LanternSky. These are background/sky
draws, not screen filters — the tower/sky family is M4 content (the
`tower_active_*` world flags + the pillar NPCs exist; the sky tint does not).
`MoonLordSkyIntensity` (the `SceneMetrics.cs:299-310` distance fade) is now
computed for the sun-scorch gate (§2b) even though the sky draw itself is
unported. Out of this lane's scope; recorded for the M4 sky worklist.

---

## 4. The DrawWhite flash — the producers LIVE (was: no callers)

**Gap.** `MoonlordDeathDrama.DrawWhite` (`MoonlordDeathDrama.cs:219-225`) +
`DrawPieces`/`DrawExplosions` had draw halves and a `SceneState` update
(`update_moonlord_death_drama`) but NO producers — `request_light` /
`add_explosion` / `throw_pieces` were `#[allow(dead_code)]` with no callers,
so `whitening` never rose and the pieces/blasts never spawned.

**Port.** The C# call sites now queue through two new `Npc` request vectors
(`wants_drama_light: Vec<(f32,f32,f32)>`, `wants_drama_explosion:
Vec<(f32,f32)>`), drained into `SceneState` in the sim's NPC pass
(`world_sim/npcs.rs`), in NPC slot order like the C#'s array walk:

- **RequestLight ramps** (`MoonlordDeathDrama.cs:245-256`):
  - `AI_077_MoonLordCore`: the teleport re-entry `ai[1]/30` (`NPC.cs:41179`),
    the spawn-in `ai[1]/30` (`:41203`), the crash drama `ai[1]/60` (`:41314`)
    + `(ai[1]-480)/120` from 480 (`:41436`), the despawn `ai[1]/40` (`:41453`).
  - The DD2 crystal's win scene `GetLerpValue(480,570,ai[0])` (`:40794`) and
    loss scene `GetLerpValue(480,600,ai[0])` (`:40890`) — both were recorded
    deviations ("the MoonlordDeathDrama light is unmodelled"), now ported.
- **AddExplosion** (`MoonlordDeathDrama.cs:240-243`): the crash drama's
  dust-storm spot (`NPC.cs:41364`), gated on the same in-world/non-solid
  `flag3`. The spot math now computes (was draw-only RNG consumption); the
  draw order is unchanged (the unconditional top-level draws still ride the
  stream in the C#'s order). The frame-speed `Next(2,4)` rides the shared
  `Main.rand` stream (`app.rngs.global()`).
- **ThrowPieces** (`MoonlordDeathDrama.cs:227-238`): the core's drama-end
  `checkDead` (`NPC.cs:82679-82682`'s `HitEffect(0, 1337)` arm) flings the
  four body pieces, seeded by `Next(100)` on the death pass's `UpdateNPCs`
  stream (the death-cue precedent). The `netMode != 2` gate folds away (the
  port's host simulates).

The three `SceneState` methods lose their `dead_code` allows.

**Deviation (recorded in ANALYSIS).** The dust-storm's dust fan + the inner
`Dust.NewDust` streams stay unmodelled (the dust engine); only the
unconditional top-level draws + the explosion spot are consumed. The
`DrawWhite` quad itself is the renderer's `white_uv` (the `MagicPixel`
stand-in, the S1-E-16 convention).

**Note on "death/teleport white-flash".** The C#'s player death/teleport has
no white-flash — `Player.Teleport` uses `BlackFadeCameraTeleport`
(`Main.cs:62547`, a black fade) and the death screen is the `ScreenDarkness`
family. The white flash is specifically the `MoonlordDeathDrama.whitening`
(the Moon Lord death drama + the DD2 crystal win/loss scenes), which is what
this lane ports. The teleport/death black fades are the `ScreenDarkness` /
camera-transition systems (already ported / out of this lane).

**Witnesses (written, not run).**
- `world_sim/npcs.rs::the_core_spawn_ramp_requests_drama_light` — the spawn
  ramp's `ai[1]/30` request drains, the whitening fade begins.
- `world_sim/npcs.rs::the_core_death_flings_the_drama_pieces` — the drama-end
  `checkDead` flings 4 pieces + raises `downed_moon_lord`.

---

## Commits (this lane, on `task/W4-d70-moonlord-shake-screens`)

| Hash | Mechanic |
| --- | --- |
| `ee490593` | PlayerNet team/shield/dead fields + the paladin peer loops live |
| `67fb6d4c` | lens-flare advanced-shadow offset + the vampire sun-scorch chain |
| `954202e3` | the MoonlordDeathDrama producers (DrawWhite light/explosion/pieces) |
| (this) | the ShimmerMonolithState terms + the screen-shader census + docs |

## Verification

Code-only lane: **no cargo / tests / graphify run**. All witnesses are written
as `#[cfg(test)]` functions, not executed. The doc updates (ANALYSIS "The
screen effects (S1-E-16)", PORT_MAP, ROADMAP S1-E-16, research/02, research/41
D-7/D-13/D-16, research/INDEX) record the ports + the remaining deviations.
Five `anchors.txt` canary rows added (paladin_shield, update_advanced_shadows,
update_sun_scorch, wants_drama_light, shimmer_monolith_state).

## Remaining deviations (the named dependencies)

1. **The sun-occlusion pixel mask** — a GPU render-target readback
   (`HorizonHelper.cs:35-69`), deferred to the d69 S1-G-13 layer-target seam.
2. **The WaterDistortion entity-wave/ripple buffer** — the
   `_distortionTarget` ping-pong + `QueueRipple` producers
   (`WaterShaderData.cs:136-333,445-469`), a render-target + wave-quad system.
3. **The accessory shader grants** (`Player.ApplyShader`,
   `Player.cs:14246-14283`) — the `perspectivePlayer.*MonolithShader` terms of
   the monolith/Sepia/Noir/CRT/Test2 activation conditions.
4. **The tower/monolith/MoonLord/Sepia/CRT/Test2/Noir activation readers** —
   the `ManageSpecialBiomeVisuals` conditions (`SceneState.cs:81-104`) have no
   driver arm; the WGSL passes for the tower/monolith/MoonLord/Sepia/CRT rows
   are also untranslated (passthrough 0).
5. **The PaladinsHammerShockwave particle** — a dust/gore orchestra
   (`ParticleOrchestrator.cs:815-879`), not a screen filter; the particle
   system is unported.
6. **The sky-manager visuals** (the tower/MoonLord/Party/Martian/Slime/
   CreditsRoll/Aurora/Sandstorm/Blizzard/Lantern skies) — M4 background work.
