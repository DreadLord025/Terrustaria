# 67 — World-lane dust WIRING (W4-f6): keep-one + seam census

Status: FINAL (W4-f6, the world lane's wiring + final phase, branch
`task/W4-f6-dust-world-3`). The 40 staged emitters (research/65, commits
`885a6bfa`/`d6f555b8`/`9dec91b0`/`680ac9d8`) are called from the live
machines at their C# positions; the final phase applied the d54 drill-atan2
fix, published the g68 `tile_draw_rand` keep-one + the mount-abilities
activation contract, and ported the four unclaimed minecart spark variants
+ the staged fart cloud. Code-only: WRITTEN NOT RUN, the witnesses are owed
a compile window before merge.

## The e4 co-lane keep-one (mount 1-19 ⊂ f6's 27)

`origin/task/W4-e4-dust-sites-world-2` (research/66) staged a SECOND
independent port of the same Mount.cs sites: their "mount 1-19"
(`0b9ad269` batch 3 + `9eb72737` batch 4) covers `Mount.cs:3113-5071` —
the drill beam pair, the unicorn/wolf/witch dash rings + bodies, the
basilisk charge pair, the golf-cart smoke, the four minecart effects, the
witch-broom trinket pair and the drill-mount trail. **Every one of those
19 sites is inside f6's 27-site Mount census** (research/65): f6 ports
3113-6493 (the full movement + effects + spawn/dismount set).

**Keep-one decision: f6's `mount_dust.rs` is canonical for ALL Mount.cs
dust.** Rationale:

- f6's set is the superset (27 ⊃ 19) and is the one WIRED to the live
  `MountState` machine (`frame_extras`, `queue_mount_effect_dust`,
  `queue_spawn_dust`, the `pending_mount_dust` drain). e4's emitters are
  staged-only (no live caller in their branch).
- f6's emitters carry the post-spawn writes + the draw-mirror witnesses
  pinned to the same `DustPool` contract; merging both would double-draw
  every Mount site (two `NewDust` calls per C# site = a determinism break).

**Merger contract:** when e4 lands, DROP its `mount_dust.rs` Mount
emitters (the 19) in favour of f6's; KEEP e4's
`orchestrator_dust.rs` (ParticleOrchestrator 41-61, `fc2f64a2`/`e0d6d870`)
— that file is e4's exclusive lane (f6 never ports orchestrator sites).
The e4 claim doc's "mount 5071 is e4's, 5099+ stays f6's" boundary is
SUPERSEDED: f6 owns the whole Mount file (its 27-site census predates the
wiring and was pushed first, `960fc28d`). e4 wires orch 41-61 only.

The shared `GetFairyQueenWeaponsColorFull` / `NextVector2Square/Circular`
/ `DirectionTo` / `witch_broom_trinket_rotation` helpers: f6's
`mount_dust.rs` copies are canonical (e4's `witch_broom_trinket_rotation`
is the same RNG-free port; keep f6's, drop e4's duplicate).

## Seam census (where each staged emitter now fires)

| Emitter (research/65) | Live seam | RNG pass |
| --------------------- | --------- | -------- |
| Mount spawn/dismount (6382/6391/6404/6413/6493) | `player.rs::queue_spawn_dust` ← `set_mount` (:6300) / `dismount` (:6169) / `start_death` (KillMe :38907) | UpdatePlayers |
| Mount failed-dismount (6339) | `player.rs::quick_mount` ladder (:6058) | UpdatePlayers |
| Mount movement (3878/3896/3939/3970/4000/4023/4103/4123) | `mount.rs::frame_extras` ← `player.rs::mount_update_frame` (the 5 PlayerFrame/TryLanding sites) | UpdatePlayers |
| Mount drill beam (3113/3170) | STAGED — `UseDrill`/DrillMountData is M2-MOUNT-4 (no live caller); the exact activation seam is published below ("The mount ABILITIES co-lane") for c14/d8's successor lane | UpdatePlayers (when wired) |
| Mount effects (4703/4719/4750/4782/4824/4849/5061/5071/5099/5149) | `player.rs::queue_mount_effect_dust` ← `update_mount` (UpdateEffects :4804) | UpdatePlayers |
| Mount golf-cart smoke (4626) | DEAD — `EmitGolfCartSmoke` has zero callers (verified) | n/a |
| Chest unlock (420) | `chest.rs::unlock_dust` ← both `unlock_chest` sites (:33178/:33204) | **UpdatePlayers** (re-pin: ItemCheck→TileInteractionsUse, NOT bare global) |
| Utils.PoofOfSmoke (1089) | `bug_net.rs::catch_npc` StatuePoof/MysticFrog (:79912) | UpdatePlayers |
| Pylon teleport (390) | `render/pylons.rs::draw_pylons_trophies_void` pylon gate (:8669-8673) | private `tile_draw_rand` (gates) + bare global (spawn) |
| Void-lens (48/63) | `render/pylons.rs` void-lens Update (:8717-8720) | bare global (`Main.rand`) |
| Lava bubbles (560/564) | `render/tiles.rs::draw_tiles` ← `liquid_draw::build_with_bubbles` (:552-577) | private `liquid_rand` (`LiquidRenderer._random`) |
| PopupText fancy flash (461/470/479/488) | STAGED — g57's PopupText module is on `task/W4-g57-ui-reforge-anim`, NOT in this branch; the reforge contexts are RNG-zero (`PrepareEffects` ItemCraft-only), so the flash arm has no live trigger here | UpdateFloatingText (when g57 lands) |
| Rain splash (122) | SKIPPED — d130 (`task/W4-d130-weather-ambience`) owns the rain machine | bare global (draw) |
| Nebula level-up (3697) | STAGED — MessageBuffer case 102 is MP-only net dispatch; f12's buff-scan (`task/W4-f12-buff-frame-wiring`) owns the nebula tier state, the net-102 handler is unported in SP | net dispatch pass |

## New private streams (the clock-seed → world-seed deviation)

`App.tile_draw_rand` (`TileDrawing._rand`, `TileDrawing.cs:104`/:191) and
`App.liquid_rand` (`LiquidRenderer._random`, `LiquidRenderer.cs:132`) are
the two PRIVATE draw-side streams the C# seeds from `Environment.TickCount`.
The port seeds both from the world seed (the `mine_crack_rng` determinism
convention, ANALYSIS "The tile-crack overlay"). Neither is a `SwapRandom`
key. The pylon dust GATES draw `tile_draw_rand`; the pylon SPAWN + the
void-lens draw the bare global (`NamedRngs::global()`); the lava bubbles
draw `liquid_rand`.

### The g68 `tile_draw_rand` same-name collision — KEEP ONE FIELD

`origin/task/W4-g68-weather-wind-leaves` (tip `d247a1c9`, wiring-1
`ea9e784f`) independently added `App.tile_draw_rand` for the
`EmitLiquidDrops` per-draw gates (`foliage::emit_liquid_drops_area`,
`TileDrawing.cs:7609` `_rand.Next(num*2)`). **The C# census says these are
the SAME stream, not two:**

- `TileDrawing._rand` is ONE private field (`TileDrawing.cs:104`, seeded
  once `:191`) on the ONE `Main.TilesRenderer` instance (`Main.cs:2301`,
  constructed once `Main.cs:6541`). The drip gate (:7609, inside
  `Draw(solidLayer:false)`'s tile switch :693) and the pylon crystal gates
  (:8669-8670, inside `PostDrawTiles(false)` → `DrawTeleportationPylons`
  :424) both read `this._rand` on that instance — renderer-lifetime,
  shared, no second field anywhere in the class.
- `Mount.cs` has ZERO private RNG fields (grepped `private.*Random|new
  UnifiedRandom|new FastRandom|_rand\b` — no hits): every Mount dust roll
  rides `Main.rand` (the UpdatePlayers pass). The "two-fields rename"
  alternative does not apply — there is no Mount draw stream to separate.
- The distinct `LiquidRenderer._random` (f6's `App.liquid_rand`) stays a
  SEPARATE field: different C# class, different instance, different seed
  site (`LiquidRenderer.cs:132`). That two-field split is already correct.

**Keep-one decision: ONE `App.tile_draw_rand`, BOTH consumers** (g68's
drip gates + f6's pylon gates). Both branches declare the identical field
(`pub(crate) tile_draw_rand: tr_core::UnifiedRandom`, seeded
`UnifiedRandom::new(world_seed as i32)`), so the declaration conflict
resolves by keeping ONE declaration with the union of the doc comments —
NO rename, NO second field. Splitting into two streams would be a
determinism break (the C# interleaves both consumers' draws on one
stream).

**Merge-order pin (the interleaving):** C# runs `Draw(solidLayer:false)`
(the per-tile drip gates, row-major) BEFORE `PostDrawTiles(false)` (the
per-visible-pylon gates). The port's two seams already sit in that order
in `app/frame/draw.rs`: g68's `emit_liquid_drops_area` inserts right after
the `tile_cracks` overlay (:1221 at f6 tip), f6's
`draw_pylons_trophies_void` follows (:1243). The merger keeps drip-first,
pylon-second, both drawing `&mut self.tile_draw_rand` in the same frame;
neither call may move inside the G6-cached range (both are per-DRAW
cadence, `WorldFrameKey` untouched). The d60 tile-pass-attach contract
(g68's rescan stand-in) does not change this order — the attached cells
replay the same row-major sequence.

## The mount ABILITIES co-lane — the drill-beam activation contract

Co-lanes: `origin/task/W4-c14-tile-break-footstep-sounds` (tip `36732286`,
Santank `:20957` is its double-jump SOUND site — `play_double_jump_sound`,
not the ability machine) and `origin/task/W4-d8-proj-ai-71-80` (tip
`50dcaa75`, "d8c" — the Scutlix ability half WIRED: `ability_charging`,
`set_ability_active`, `stop_ability_charge`, `clamp_to_dead_zone`,
`aim_ability_scutlix`, `scutlix_laser_fire`; its commit `f7687d54` states
"Drill mining payload stays M2-MOUNT-4"). Neither lands the Drill
mining machine. **f6's `drill_beam_dust` stays STAGED; the exact seam for
whichever lane lands the mining machine (M2-MOUNT-4 or a wiring wave):**

- **Call site:** `Mount.UseDrill` (`Mount.cs:3059-3179`) is called from
  `Player.cs:28288-28291` — AFTER `ItemCheckWrapped` (:28285) +
  `PlayerFrame` (:28286) + `DoDeadCellsBeheadedParticles` (:28287), inside
  `Player.Update` under the `UpdatePlayers` swap (`Main.cs:17754`). Gate:
  `_type == 8 && _abilityActive` (:3061). `UpdateDrill` (:3036, the
  cooldown tick + diode lerp) runs earlier at `Player.cs:26683-26686`
  (movement block) and draws NO dust.
- **The two beam arms:** `controlUseItem` → blocks (:3067-3126): per free
  beam, `DrillSmartCursor_Blocks` pick → `PickTile` (:3103, local only,
  the DemonAltar hurt :3092-3096 + `noBuilding` refusals skip the pick but
  NOT the dust) → **dust pair :3108-3116** → `SmoothSlope` (:3119, local
  only, AFTER the dust) → `cooldown = drillPickTime`, `lastPurpose = 0`,
  break. `controlUseTile` → walls (:3127-3178): same shape via
  `DrillSmartCursor_Walls`/`PickWall`, dust pair :3165-3173,
  `lastPurpose = 1`. The dust fires even when the pick was refused
  (`flag2`/`flag3` false) and on remote players (`flag` false skips only
  the pick/slope).
- **The exact call:** `mount_dust::drill_beam_dust(target, player_center,
  rng, dust, ctx, world)` — `target = ((point.X << 4) + 8, (point.Y << 4)
  + 8)` (:3106/:3163), `player_center = mountedPlayer.Center` (:3107).
  Signature KEEPS `(target, player_center)` — the double-atan2 fix
  (research/131 §7.1, applied this branch `76aeaf3c`) lives inside. ONE
  call per beam-accept, TWO dusts per call; RNG order per dust:
  `Next(2)` side → `NextDouble` speed → the `NewDust` internals, all on
  `UpdatePlayers` (the module-doc contract).
- **Rust seam options (pick ONE, do not double-fire):** (a) direct call
  inside the ported `UseDrill` on the `UpdatePlayers` stream — the C#
  statement order is trivially preserved; or (b) a new
  `MountDustRequest::DrillBeam { target, center }` variant queued by the
  `UseDrill` port and drained by `mount_dust::drain_mount_dust`
  (`world_sim/player.rs:1569`) — legal because `UseDrill` runs late in
  `Player.Update` (:28290) and the drain sits at the tick tail: no other
  mount emitter queues between the beam arms and the drain, so the
  request order equals the C# order. If a future emitter is inserted
  between :28290 and the tick tail, prefer (a).
- **State the mining lane owns (not f6's):** `DrillMountData`
  (`Mount.cs:36-50`: `beams[]` cooldown/curTileTarget/lastPurpose,
  `beamCooldown`, diode rotation), `drillPickPower`/`drillPickTime`/
  `amountOfBeamsAtOnce`, `DrillSmartCursor_Blocks/Walls` (:3181+, the
  `PlotTileLine` scan). d8's `set_ability_active` (kind-8 gated) is the
  `_abilityActive` writer the gate reads; f6's `MountState` already
  carries `ability_active` (`mount.rs:2110`) — at merge keep ONE field
  (d8's accessors + f6's declaration are the same C# `_abilityActive`).

## The minecart-track dust — the unclaimed delegate variants PORTED

Census (f6's Mount.cs lane, final phase): the track-side sparks are NOT
Mount.cs `NewDust` sites — they ride the `MountDelegatesData.MinecartDust`
delegate (`Mount.cs:104`), invoked by `Minecart.WheelSparks`
(`Minecart.cs:1623-1630`) from the three Player.cs sites (the rolling arms
`:19877-19959`, the landing burst `Minecart.cs:812` via
`Player.cs:28110`, the track-hook burst `:22688`). The base `Sparks`
(type 213) was already ported + wired (`minecart.rs`, S1-E-1 phase 2).
The FOUR variant delegates were unclaimed port-wide (checked d54/e4/f11/
e20/c14/d55/d50 tips — f11/e20 are Wiring.cs device lanes, c14 ported the
fart/meow SOUND halves only and explicitly left "the fart cloud … the
visual seam"):

| Delegate | C# | Row | Body |
| -------- | -- | --- | ---- |
| `SparksFart` | `DelegateMethods.cs:328-346` | 51 (`Mount.cs:1099`) | dust 211, alpha 50, scale 0.8, two 50% +25 bumps, noLight, Next(3)==0 gravity, NO fadeIn |
| `SparksTerraFart` | `:348-371` | 53 (`:1107`) | 50% SparksFart branch (rolled FIRST), else dust 107 alpha 100 scale 0.8 green-lerp colour, noLightEmittance |
| `SparksMech` | `:373-392` | 11 (`:1487`) | dust 260, the 213 body with +0.5 fadeIn |
| `SparksMeow` | `:394-414` | 33 (`:1013`) | the 213 body + the 2870 shader (the documented `Dust.shader` drop) |

**Ported this branch** (`minecart.rs`): `SparkVariant` + `spark_variant_of`
(the row map), the `SparkSpawn.body` resolved at QUEUE time (the TerraFart
branch pick precedes the side/speed rolls — the C# `:350` order), the four
drain bodies (`spawn_spark` dispatch), the tick arms (260 = the 213 body;
211 = fall + accumulate-rotation + generic −0.01 fade + the `:729`
noLight-solid-kill; 107 = the `velocity *= 0.9` gravity arm `:1821-1824`),
and the draw (213/260 self-lit gray via `dust_table::get_alpha`, 211/107
the light-tinted generic path + GetColor overlay + the full-dark kill;
`minecart::draw` now takes the LightMap, the `sand::draw` convention).
The four live call sites pass `spark_variant_of(self.mount.kind())`; the
base-213 stream is byte-identical (the variant adds no rolls for Sparks).

**`SpawnFartCloud` (`DelegateMethods.cs:258-306`) STAGED**
(`minecart.rs::spawn_fart_cloud`): the 3-gore 435-437 cloud (the
`(Width/2−18, Height−16)` anchor, the 0/−10/+10 X offsets, the −4 lift,
the ×0.2/+carry/Y×0.75 post-writes, the 20-tick `fartKartCloudDelay` gate)
+ the 53-only 15-dust 107 tail. Activation seam: c14's
`JumpingSoundFart`/`LandingSoundFart` (useDelay false) / `BumperSoundFart`
(useDelay true) delegates call it inline on the `UpdatePlayers` stream;
the delay decrements per track-rider tick (`Player.cs:28098`). The
`Player.fartKartCloudDelay` field + the decrement are the wiring lane's
(c14 owns the delegates; the port has no field yet). The Fart-in-a-Jar
accessory's OWN jump cloud (`Player.cs:20855-20863`, 3 gores + the 188
dusts) is the d51 player-dust lane's, NOT this delegate.

Witnesses (WRITTEN NOT RUN): `w4_f6_spark_variant_of_maps_the_cart_rows`,
`w4_f6_mech_spark_is_dust_260_with_the_half_fade_band`,
`w4_f6_fart_spark_is_dust_211_with_the_alpha_bumps`,
`w4_f6_terra_spark_green_arm_is_dust_107_with_the_lerp_colour`,
`w4_f6_terra_fart_branch_rolls_before_the_side_and_speeds`,
`w4_f6_variant_ticks_follow_the_c_sharp_arms`,
`w4_f6_fart_spark_dies_over_solid_when_no_light`,
`w4_f6_fart_cloud_spawns_three_gores_and_gates_on_the_delay`,
`w4_f6_terra_fart_cloud_adds_the_fifteen_green_dusts` (9 new).

## New player state (the wiring's fields)

- `Player.basilisk_charge` (`Player.cs:842`) — PLAYER state, survives the
  dismount (folded player-side before `frame_extras`).
- `Player.time_for_visuals` / `wind_for_visuals` / `wind_in_place` — the
  Pixie tail's even-tick gate + the Witch Broom trinket sway inputs, fed
  per-tick by `world_sim/player.rs` (the `frame_inputs` convention).
- `MountState.dash_latch` — the 44/45 `_mountSpecificData` bool, reset on
  dismount (`Mount.cs:6151`/`:6212`).
- `Player.pending_mount_dust` — the request channel (the
  `pending_sticky_dust` convention), drained on UpdatePlayers.

## Witnesses (WRITTEN NOT RUN, code-only)

The wiring adds drain-parity witnesses (the request channel changes no
draws vs the direct emitter): `w4_f6_drain_spawn_matches_the_direct_wolf_emitter`,
`w4_f6_drain_failed_dismount_matches_the_direct_emitter`,
`w4_f6_drain_basilisk_matches_the_direct_pair`,
`w4_f6_drain_pixie_tail_matches_the_direct_emitter`,
`w4_f6_pylon_dust_box_centers_the_crystal_rect`,
`w4_f6_bubble_cells_match_build_and_collect_lava`. The 32 original
draw-mirror witnesses (research/65) still stand.

## FINAL phase record (2026-11-21, this branch)

The five final-phase items, in order (one commit each):

1. **Drill-atan2 fix** (`76aeaf3c`) — d54 research/131 §7.1 EXACT hunks
   applied at the fetched d54 tip `c89aea9f`: `mount_dust.rs`
   `drill_beam_dust` aim → `(d.y as f64).atan2(d.x as f64) as f32` + the
   drill-beam witness mirror re-pin + the `Utils.cs:1655-1658` citation
   (re-read verbatim: `(float)Math.Atan2(v.Y, v.X)`, DOUBLE of the widened
   floats). Signature keeps `(target, player_center)`; no RNG-order change
   (the atan2 is draw-free). The §7.1 line numbers shifted with the wiring
   commits (the witness mirror is now :1849, was :1598 at `5daa1d70`) —
   the hunks applied by content, both sites verified.
2. **`tile_draw_rand` g68 collision keep-one** (`8a34f664`) — ONE field,
   BOTH consumers (the census above): `TileDrawing._rand` is one private
   field on the one `Main.TilesRenderer`; Mount.cs has zero private RNG
   fields. No rename, no second field; drips-before-pylons merge order.
3. **Mount ABILITIES activation contract** (`a67d1363`) — c14 (`36732286`)
   / d8 (`50dcaa75`) censused: neither lands `UseDrill`; the exact seam
   published above for the mining-machine lane. `ability_active` keep-one
   with d8's accessors.
4. **Minecart-track dust** (`573b7e7f`) — the four unclaimed delegate
   variants ported + the staged fart cloud (the census above), 9 witnesses.
5. **This FINAL record** — the status header + the phase log.

Verification (code-only mode): no cargo/tests/rustfmt run (the task's
constraint). Each code change was re-read against the cited C# at the
decompiled source; the witnesses are WRITTEN NOT RUN and owed the
merge-verify compile window. The base-213 spark stream is byte-identical
(the variant adds zero rolls for `SparkVariant::Sparks`); the drill-atan2
fix changes only the aim angle's last-ulp precision (the cos/sin velocity
bytes), no draw count.
