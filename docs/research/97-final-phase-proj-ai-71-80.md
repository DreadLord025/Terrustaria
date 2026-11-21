# 97 — FINAL-PHASE lane 18 (proj AI 71-80): the drill split, the 434 verdict, the band census, the fountain-dust + 606-light cross-notes

W4-d8 FINAL phase, branch `task/W4-d8-proj-ai-71-80` (continues the DONE
`50dcaa75` doc-sweep). Code-only round — every witness is WRITTEN NOT RUN
(no compiler on this lane); each expected value is hand-derived from the
cited C# lines. This doc records the five final-phase items: the Drill
mining payload's port/split verdict (1), the 434 kept-row census (2), the
band 71-80 final census (3), the fountain-dust lane claim (4) and the
606-beam light-feed claim (5).

Commits (this phase):

| item | commit | scope |
| --- | --- | --- |
| (1) Drill ability half | `85e87499` | `DrillMountData`/`DrillBeam` + `UpdateDrill`/`AimAbility`/`UseAbility`/`UpdateEffects` gate + the 453 crosshair wiring + the player per-tick block + 2 witnesses |
| (2)-(5) census | this doc | the 434 verdict, the band table, the g64/e6 cross-notes |

---

## (1) The Drill mining payload — the ability half PORTED; the mining WRITE split-contracted

The DOC-SWEEP (3)/(4) deferral named three blockers for the Drill (8)
mining payload (`Mount.cs:3036-3180`): "the beam cooldowns, the
smart-cursor tile scans, the `PickTile` writes". The final-phase census
re-probes each against the CURRENT port:

- **Beam cooldowns** — pure state (`DrillBeam.cooldown`,
  `DrillMountData.beamCooldown`), ticked by `UpdateDrill` (`:3039-3056`).
  No external dependency. **Resolved** — ported as `DrillMountData`.
- **Smart-cursor tile scans** (`DrillSmartCursor_Blocks`/`_Walls`
  `:3181-3252`) — `Utils.PlotTileLine` (`Utils.cs:2337-2350`, the
  Bresenham `PlotLine` `:2238-2292` is ported in `npc_ai_town.rs:847`),
  `WorldGen.CanKillTile` (`edit::can_kill_tile`, `kill.rs:2783`),
  `Player.CanPlayerSmashWall` (`edit::can_player_smash_wall`,
  `wall.rs:415`). All three predicates exist. **Resolved** — but the
  scans are only CALLED by `UseDrill`, the write half.
- **`PickTile` writes** — `Player.PickTile` (`Player.cs:53757`) is ported
  as `pick_tile_hit` (`world_sim/editing.rs:774`), `PickWall`
  (`:45942`) as the `wall_hammer_tick` book path (`:1473`). **Resolved**
  — but they are App-level seams (`&mut App`), not pure mount state.

The NEW blocker the census surfaces: **`Tile.SmoothSlope`**
(`Tile.cs:792-865`, called by `UseDrill` `:3119` after every block pick)
is UNPORTED. The port has `edit::slope_tile`/`pound_tile`
(`hammer.rs:80`/`:108`, the `WorldGen.SlopeTile`/`PoundTile` halves) but
NOT the `Tile.SmoothSlope` neighbour-aware slope picker (the 4-neighbour
`flag`/`flag3`/`flag4`/`flag5` bitmask → the slope/halfBrick switch
`:814-845`). It also needs `WorldGen.TileEmpty` (`:69976`) and the
`SolidOrSlopedTile` reads (`collision.rs:551` has the tile form). The
`Hurt` on a pre-hardmode Demon Altar (`:3092-3096`, `PlayerDeathReason.
ByOther(4)`, `statLife/2` damage, `-direction` knockback) and the two
dust-230 sparks per pick (`:3108-3116`/`:3165-3173`, the `Main.rand.
Next(2)` side + `NextDouble` speed draws) are further write-half seams.

**The verdict (the lane's question):** the Drill payload SPLITS exactly
along the Scutlix precedent's line —

- **The ability half is SELF-CONTAINED and PORTED** (`85e87499`): the
  `DrillMountData` payload (`:36-58`) + `DrillBeam` (`:20-34`),
  `UpdateDrill`'s cooldown tick + diode lerp (`:3036-3057`),
  `AimAbility`'s Drill case (`:5466-5494`, the diode target + the ±2π
  short-way wrap + the raw crosshair pin), `UseAbility`'s toggle + the
  453 spawn tuple (`:3314-3335`), and `UpdateEffects`'s no-crosshair
  clear (`:5010-5015`). These are pure state the ALREADY-PORTED 453
  crosshair AI (`Projectile.cs:30621-30670`, `ai_mount_crosshair`) and
  the player per-tick mount block (`Player.cs:26683-26686`) consume — the
  same shape as the Scutlix ability half. Zero draws.
- **The mining WRITE stays M2-MOUNT-4** (the split-contract): `UseDrill`
  (`:3059-3180`) — the `controlUseItem` block loop (`PickTile` +
  `Tile.SmoothSlope` + the altar `Hurt` + dust-230) and the
  `controlUseTile` wall loop (`PickWall` + dust-230). It needs the
  unported `Tile.SmoothSlope`, the App-level `pick_tile_hit`/
  `wall_hammer_tick` seams, the `Hurt` lane and the dust pool. The
  `ItemCheck_HandleMount` press path (`Player.cs:52922-52950`, the
  `noItems = true` + `StartChanneling()` + the `releaseUseItem &&
  releaseUseTile` → `UseAbility(toggleOn: true)` 453 spawn) is the
  write-half's entry — the port's `use_ability_drill` returns the spawn
  tuple ready for it, but the press wiring (the channel arming + the
  release-latch reconciliation with the hoisted `update_editing` clear)
  lands with the write.

### The activation contract (what the write-half inherits)

The ported ability half exposes, on `MountState`:

- `drill_data() -> &DrillMountData` — the eight beams (each
  `cur_tile_target`/`cooldown`/`last_purpose`), `beam_cooldown`,
  `diode_rotation`/`diode_rotation_target`, `crosshair_position`.
- `update_drill()` — the per-tick maintenance (already wired into
  `Player::update_mount`).
- `aim_ability_drill(center, mouse) -> bool` — the diode aim (already
  wired into the 453 crosshair's local + remote aim).
- `use_ability_drill(player_pos, mouse, toggle_on, local_owner) ->
  Option<(x, y, ai0, ai1)>` — the toggle; `Some` = the 453 spawn the
  press path owes the pool (the `scutlix_laser_fire` intent precedent).
- `drill_update_effects_gate(crosshair_count)` — the no-453 clear
  (already wired into `Player::update_mount`).
- `ability_active()` / `set_ability_active(bool)` — the mining gate
  `UseDrill` reads (`:3061`).

The write-half's `UseDrill` port reads `ability_active()` +
`drill_data().beams`/`beam_cooldown`/`crosshair_position`, calls the
smart-cursor scans (the ported predicates + `PlotTileLine`), then
`pick_tile_hit`/`wall_hammer_tick` + the NEW `Tile.SmoothSlope` + the
altar `Hurt` + the dust-230 sparks, and writes back
`beams[j].cur_tile_target`/`cooldown = drillPickTime (1)`/`last_purpose`
(`Mount.cs:3087`/`:3121-3122`/`:3149`/`:3174-3175`). The statics:
`drillPickPower = 210`, `drillPickTime = 1`, `amountOfBeamsAtOnce = 2`
(`Mount.cs:6657-6659`).

Witnesses: `w4_d8_the_drill_ability_math` (mount.rs: the fresh payload,
the cooldown tick + target latch, the diode lerp, the aim wrap + pin, the
toggle + spawn tuple, the no-crosshair clear) + the extended
`w4_d8_the_crosshair_pins_aims_and_releases` (the 453 local aim writes the
diode + pin, the release clears the flag).

## (2) The 434 ScutlixLaserFriendly kept-row — FINAL census: a legacy id, live tails, NO spawner

The DOC-SWEEP (3) census found no 434 spawner; the final-phase re-probe
CONFIRMS it and pins WHY the C# keeps the row. A full-decompile search for
`NewProjectile` with type 434 (or `ProjectileID.ScutlixLaserFriendly`)
finds NONE. The id (`ProjectileID.cs:1422`) is live in SEVEN tails:

| site | C# | what it does |
| --- | --- | --- |
| `ProjectileID.cs:1422` | `ScutlixLaserFriendly = 434` | the id constant |
| `Projectile.cs:5047-5055` | SetDefaults | 1×1, aiStyle 74, friendly, extraUpdates 100, penetrate −1 |
| `Projectile.cs:10927-10931` | NewProjectile ai-pin | `ai[0]/ai[1] = position` (the beam-draw anchor) |
| `Projectile.cs:12448` | Damage_CanDealDamage | `type == 434 && localAI[0] != 0` → no damage |
| `Projectile.cs:13975-13978` | on-hit | `numUpdates = 0` (stop sub-stepping) |
| `Projectile.cs:17079-17086` | Update tail | the charge latch: extraUpdates 1, velocity 0, localAI[0]=1, localAI[1]=0.9999 |
| `Projectile.cs:17991-17995` | HandleMovement | blocked move: integrate + numUpdates 0 (stick, no kill) |
| `Main.cs:3477-3480` | the draw-offset switch | `num = 0; Y -= 2*gravdir` |
| `Main.cs:34603-34619` | DrawProj | the MagicPixel beam from `ai[0]/ai[1]` to position, red+white, `localAI[0]` alpha |

**The verdict:** 434 is the **1.3-era Scutlix mount laser**, kept live in
1.4.5.8's AI/damage/movement/draw tails but with its spawner REMOVED. The
1.4 Scutlix mount fires **606 `ScutlixLaser`** instead (`Mount.cs:3264`,
`ProjectileID.cs:1766`) — a DIFFERENT row (5×5, aiStyle 1, alpha 255,
extraUpdates 2, timeLeft 600, `Projectile.cs:6530-6541`) on the
Martian-laser AI_001 ramp, NOT the 434's aiStyle-74 charge-decay. The two
are distinct kinds: 434 is a hitscan-style beam that latches to its spawn
anchor and decays (`localAI[1] -= 0.001`/tick, the aiStyle-74 arm
`:30605-30616`); 606 is a projectile that flies, sticks in a tile and
ramps its damage 5→150. The C# keeps 434's tails because the id is still
referenced by the draw/damage/movement machinery (a mod or a save could
carry one), but NOTHING in vanilla 1.4.5.8 spawns it.

**The port's stance (unchanged, now census-confirmed):** the 434 row +
aiStyle-74 arm STAY (`data.rs:1753`, `ai_scutlix_laser`,
`projectile.rs:17630` — the charge-decay kill, the blocked-move stick, the
Update-tail latch `:19965`, the damage refusal `:22712`, the on-hit
numUpdates-0 `:23238`). The `NewProjectile` ai-pin (`:10927-10931`) stays
UNPORTED — it is a spawn-seam concern and no spawner exists; the AI arm
never reads `ai[0]/ai[1]` (only the unported draw does), so the stock
spawn arguments stand. This is a **legacy-keep**, not a gap: the row is
armed for any future 434 spawner (a mod lane), the pin lands with it.

## (3) The band 71-80 final census — ALL TEN styles armed

Every aiStyle in 71-80 has a Rust AI arm. The dispatch
(`projectile.rs`) and the rows (`projectile/data.rs`):

| style | C# arm | Rust fn (file:line) | kinds | folds / notes |
| --- | --- | --- | --- | --- |
| 71 | `AI()` `:30355-30454` Typhoon | `ai_typhoon` (projectile.rs:4461, dispatch :17600) | 409 | the lock/turn/accel + full tile reflect; the dust trail is g63's (kind ≤ 400) |
| 72 | `:30455-30479` Bubble | `ai_bubble` (projectile_ai_misc.rs, dispatch :17607) | 410 | the fade/×0.96 drag/scale roll/box resize; the pop Kill is the DOC-SWEEP (2) port |
| 73 | `:30480-30604` Fountains | `ai_fountain` (projectile_ai_misc.rs:820, dispatch :17616) | 419-422 | the jet-math PORTED (DOC-SWEEP (1)); the dust BODIES are g64's (item (4)) |
| 74 | `:30605-30616` Scutlix laser | `ai_scutlix_laser` (projectile_ai_misc.rs, dispatch :17630) | 434 | the charge-decay kill + the blocked-move stick + the Update-tail latch; NO spawner (item (2)) |
| 75 | `AI_075` `:66244-66538` | `ai_last_prism`/`ai_drill`/`charged_blaster_step` (dispatch :17814) | 633/632, 445, 460/459/461 | the 445 Laser Drill's full laser AI (tile-target snap, reach clamp, beam light) is a recorded follow-up; the prism/blaster arms are the i22/i29 ports |
| 76 | `:30621-30670` Mount crosshairs | `ai_mount_crosshair` (projectile_ai_misc.rs:980, dispatch :17662) | 441/453 | the Scutlix ability half (DOC-SWEEP (3)/(4)) + the Drill ability half (FINAL (1)); the local-owner 441 auto-aim spawn is M2-MOUNT-4 |
| 77 | `:30671-30761` Electrosphere | `sphere_pulse` (projectile_ai_darts.rs:438, dispatch :17569) | 443 | the fade/pulse/frame machine + the dust draws (the i16 keep-one) |
| 78 | `:30762-30790` Xenopopper bubble | inline (projectile.rs:17458) | 444 | the alpha fade, the ai[0] angle-turn ×0.96 drag, the owner-gated life clamp; the Kill payload release is the i14 port |
| 79 | `:30791-30910` Saucer deathray | `ai_saucer_deathray` (projectile.rs:12872, dispatch :17575) | 447 | the owner check + the ground-hanging beam; the dust trail is g64's |
| 80 | `:30911-31000` Saucer missile | `ai_saucer_missile` (projectile.rs:12971, dispatch :17578) | 448 | the arming delay, the homing turn, the contact |

**No unarmed-rowed style in the band.** Every style 71-80 dispatches to a
ported arm. The residual folds are the DUST/LIGHT bodies (the g-lanes and
the light lanes, items (4)/(5)) and the two ability-spawn paths
(M2-MOUNT-4: the 441 auto-aim, the Drill mining write) — none is an
unarmed AI row. The 76 crosshairs landed in the DOC-SWEEP (3)/(4) + FINAL
(1); the 73 fountain jet-math in DOC-SWEEP (1).

## (4) The fountain 419-422 dust bodies — g64 CLAIMS them (UNROWED, self-contained)

The DOC-SWEEP (1) ported the fountain JET-MATH (`ai_fountain`'s
`jet_speed` + `fountain_jet`) and folded the dust SLOT bodies to the
g-lanes. The final-phase cross-note pins WHICH lane:

**g64 (`task/W4-g64-proj-trail-dust-2`) owns the fountain dust.** Its
claim doc (`docs/research/66-proj-trail-dust-claims.md`) rows 419/420/421/
422 under the kind-range split "g63 = kinds ≤ 400, g64 = kinds ≥ 401,
g65 = death effects", and its `proj_trail_dust.rs` ports `trail_419`/
`trail_420`/`trail_421`/`trail_422` (the four per-kind choreographies,
`Projectile.cs:30496-30603`). e3 (`task/W4-e3-dust-sites-proj-2`) and f5
(`task/W4-f5-dust-proj-3`) do NOT claim them — their site-range windows
(`:22308-24318` / `:24328-26472`) sit BELOW the aiStyle-73 arm (`:30480`),
and the d53 site-lane doc (`70`) explicitly cuts "g63/g64 own the AI()
region per-kind trail arms (`:23199+`) and AI_001 (`:54002+`)".

**The activation contract:** g64's `trail_419..422` are SELF-CONTAINED —
each re-derives the jet speed via its OWN `pillar_lift(time_left)` helper
(the `num569 = 2f` × `timeLeft/60f` below 1, `:30490-30495`) and the jet
vector via its OWN magnitude/spin `NextDouble` pair, then spawns the dust
through `DustPool::new_dust` with the full per-dust draw order (the dead
scale write's `NextDouble`, the `Next(-2,3)` jitter, the 420/422 type
roll). g64's doc lists 419-422 as **UNROWED**: "their emitters stay cited
+ witnessed, the wiring waits on the AI lane's rows."

**The cross-note (the merge keep-one):** the AI lane's rows NOW EXIST
(this branch's `ai_fountain`, the DOC-SWEEP round). At the g64 ⨉ d8 merge
the fountain dust wiring must KEEP ONE jet-math + draw census:

- d8's `ai_fountain` KEEPS the draws (the `fountain_jet` magnitude/spin
  pair, the scale `NextDouble`, the jitter `Next(-2,3)`, the 420/422 type
  roll, the 421 ring's shared magnitude) but SKIPS the bodies.
- g64's `trail_419..422` RE-DERIVE the same draws AND spawn the bodies.

Wiring g64's emitters into the 73 dispatch WITHOUT dropping d8's kept
draws would DOUBLE every fountain draw on the UpdateProjectiles stream.
The keep-one: the g64 emitter REPLACES d8's draw-kept skip (the g64 doc's
own "642 (84, the draws-kept skip REPLACED)" precedent) — `ai_fountain`
keeps the anchor-kill + the jet-speed computation, the per-kind dust loop
delegates to `trail_419..422` (which own the draws + bodies), and d8's
`fountain_jet`/draw census folds into g64's `pillar_lift`/vector math
(the two are byte-identical: same `0.85 + NextDouble*0.2` magnitude, same
`(NextDouble-0.5)*half_turn` spin, same `bow_launch::rotated_by` float
cos/sin). The 421 ring's deterministic spoke fan (`(spoke-4)/5f,
-num569*num578`) is shared. d8's jet-math witness
(`w4_d8_the_fountain_jet_speed_math`) and g64's per-kind witnesses
(`w4_g64_419..422_*`) pin the SAME values — the merge keeps g64's body
witnesses, d8's jet-speed boundary witness (the timeLeft-60 `num570 == 1`
no-scale) stays as the math proof.

## (5) The 606-beam AddLight — e6/f7 classified it ABSENT; the d8 row NOW exists (the activation trigger)

The DOC-SWEEP (3) ported the 606 row + the AI_001 ramp/stick and folded
the light to "the light layer". The final-phase cross-note pins the
light-feed claim and the NEW activation state:

**The C# light site:** `Projectile.cs:54790-54793` — inside the
440/449/606 shared AI_001 arm (`:54772`), the 606 sub-if:
`Lighting.AddLight(base.Center, 0.7f, 0.3f, 0.3f)`. This is the
`AddLight(Vector2, …)` overload — it truncates the QUOTIENT
(`(int)(Center.X/16f)`, `Lighting.cs:140-143`), NOT the coordinate before
the divide (the 440/449 arms' `(int)Center.X/16` form). The 606's red
(0.7, 0.3, 0.3) is distinct from the 449's orange (0.7, 0.65, 0.3,
`:54788`) and the 440's blue (0.25, 0.4, 0.7, `:54784`).

**The lane claims:** e6 (`task/W4-e6-addlight-proj-2`) owns the AI_001
light feed (`proj_ai_lights.rs`). Its census PORTED the 449 (`:54788`)
and 438 (`:54831`) but classified the 606 (`:54792`) and 440 (`:54784`)
as **"no row. Absent."** — the feed gates on a `proj_data` row existing,
and at e6's base the 606 had NONE. f7 (`task/W4-f7-addlight-proj-3`)
mirrors this in its census test (`proj_addlight_census.rs:54792` pinned)
and its ANALYSIS "absent" list (440/606/593/...). c19
(`task/W4-c19-addlight-projectiles`) owns the OTHER light families (the
mount-gated chillet, the rand-gated 409/849/912, the swing rects) — NOT
the AI_001 beam arm.

**The cross-note (the activation trigger):** the d8 DOC-SWEEP (3) ADDED
the 606 row (`data.rs:1772`, `SCUTLIX_LASER`). e6/f7's "Absent" verdict
for `:54792` is now STALE — the row exists, so the light feed's
row-existence gate PASSES. At the e6/f7 ⨉ d8 merge the 606 light must
ACTIVATE: `proj_ai_lights.rs` gains the 606 sub-if (`AddLight(Center,
0.7, 0.3, 0.3)`, the Vector2 quotient-truncation anchor), the f7 census
test's "absent" assertion for 606 flips to "present", and the ANALYSIS
absent-list drops 606. The 440 stays absent (no row on any branch). The
d8 slice-B arm (`projectile_ai001_b.rs:220`) already cites the light as
"skipped" — the keep-one at merge: the light feed (e6) owns the AddLight,
the slice-B ramp keeps the fade/ramp/stick (the light is the feed's, not
the AI arm's — the same split the 449 already uses).

**The eye-glow light (the Scutlix aim, `Mount.cs:5394-5414`):** a
SEPARATE site — `Lighting.AddLight((int)(eye.X/16f), (int)(eye.Y/16f),
1f*abilityCharge, 0f, 0f)` per eye while `AbilityCharge > 0`, the
coordinate-before-divide form. It lives in `AimAbility`'s Scutlix case
(the mount layer), NOT the projectile AI_001 arm. NO light lane claims it
(c19/e6/f7 census the PROJECTILE lights; the mount aim's eye-glow is the
mount draw layer's). The d8 `aim_ability_scutlix` folds it (the DOC-SWEEP
(4) note "the eye-glow light is the light layer's — skipped"). It stays
unclaimed until a mount-light lane activates it — recorded here so the
gap is visible, not silently dropped.

---

## RNG contract (unchanged by the final phase)

Item (1)'s drill ability half adds ZERO draws (the cooldown tick, the
diode lerp/wrap, the toggle, the gate are deterministic). The 453 spawn
tuple is deterministic (the mouse offset). The mining WRITE's draws (the
dust-230 `Next(2)` side + `NextDouble` speed per pick, `Mount.cs:3110-
3111`/`:3167-3168`) ride the UpdatePlayers stream with the write half
(M2-MOUNT-4) — not added here. Items (2)-(5) are census/cross-notes, no
code draws. The fountain (4) and 606-light (5) keep-one contracts preserve
the UpdateProjectiles stream (the g64 emitter replaces d8's draw-kept
skip; the e6 light feed is pure, no draws).

## Deliberate folds (recorded, final phase)

- The Drill `UseDrill` mining WRITE (`PickTile`/`PickWall`/
  `Tile.SmoothSlope`/altar `Hurt`/dust-230) + the `ItemCheck_HandleMount`
  press path — M2-MOUNT-4 (item (1), the split-contract above).
  `Tile.SmoothSlope` (`Tile.cs:792-865`) is the new unported blocker.
- The 434 `NewProjectile` ai-pin (`:10927-10931`) — no spawner exists
  (item (2), the legacy-keep verdict).
- The fountain dust BODIES — g64's `trail_419..422` (item (4), the
  keep-one contract at merge).
- The 606 beam AddLight (`:54792`) — e6's light feed, NOW activatable
  (item (5), the d8 row exists). The Scutlix eye-glow (`Mount.cs:5394-
  5414`) — unclaimed, the mount draw layer's.
- The Drill `outerRingRotation` (`Mount.cs:42`, the draw's ring spin
  `:3524-3533`/`:5811`) — carried in the payload, ticked by the unported
  draw seam (stays 0).