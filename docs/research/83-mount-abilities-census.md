# 83 — Mount special-ability census (M2-MOUNT-4, W4-c14 sound lane)

Status: **SEALED** (2026-11-21, W4-c14 lane close). CENSUS + the two projectile
abilities PORTED (Scutlix 9 + Santank 46, commits `0e290cae3` facade /
`8ff589aad` machine / `968ae4edd` Meowcart-invariance witness / `b6c99ed11`
docs), code-only (WRITTEN NOT RUN). The lane owns the two Santank ability cues
(`Mount.cs:3300` Item89 / `:3311` Item11, research/82's two M2-MOUNT-4 gaps);
the ability machinery is small enough to port whole, so it is ported here
rather than handed to a mount lane. The Drill (8) and the dash-contact /
slime-stomp combat abilities stay unported with their contracts recorded.
Seal additions: §6 the f6 mutual receipt (f6 tip `ebdcbd96d`), §7 the
dash/stomp combat-seam OWNER CONTRACT + task seed, §8 the 441/453 crosshair
MP-scope record (with the 453 correction), §9 the mount-abilities family
bible. This file is the family's canonical census — extend it, do not fork it.

## 1. `Mount.UseAbility` — the whole switch (Mount.cs:3254-3338)

`UseAbility(mountedPlayer, mousePosition, toggleOn)` has exactly THREE arms:

| case | mount | effect | sound |
| ---- | ----- | ------ | ----- |
| 9 | Scutlix | 2× laser 606 (dmg 150, speed 14) from the eye pair keyed by `_frameExtra` | NONE |
| 46 | Santank | gift-box 930 (dmg 120) when `_abilityCooldown <= 10` + missile 14 (dmg 100) every fire | Item89 `:3300` (gift) + Item11 `:3311` (missile), both `volumeScale 0.2` |
| 8 | Drill | toggle `_abilityActive`; on activate spawn the 453 crosshair at the mouse | NONE |

Every other mount id falls through the switch with no arm. The two cues
research/82 flagged (`:3300`/`:3311`) are BOTH in case 46 — the gift-box drop
and the missile barrage the task named. There is no separate "gift" ability
key; `Mount.AbilityUse` is not a C# symbol — the ability entry points are
`UseAbility` (fire), `UpdateEffects` case 9/46 (the auto-aim scan that calls
it), `AimAbility`, `StartAbilityCharge`/`StopAbilityCharge`, `AbilityRecovery`.

### The auto-fire machine (ported whole)

`UpdateEffects` case 9/46 (`Mount.cs:4879-4947`) runs every tick while mounted:
case 46 first sets `hasJumpOption_Santank` (`:4884`, already in
`player.rs::update_mount`), then scans `Main.npc` for the nearest chaseable NPC
within 850 px, inside the facing cone (±60°: `|ToRotation| <= 1.0472` facing
right, `>= 2.0944` facing left) with a clear `Collision.CanHitLine`. On a hit:
`flag4 = cooldown == 0` (type 9) or `cooldown % 10 == 0` (type 46); if `flag4`
and local, `AimAbility(target)` then (if `cooldown == 0`) `StopAbilityCharge()`
then `UseAbility(target)`; else `AimAbility(target)` + `_abilityCharging = true`.
On no target: `_abilityCharging = false` + `ResetHeadPosition()`.

- `AimAbility` (`:5335-5499`): case 9 folds the clamped aim angle to a facing +
  `_flipDraw` + `_frameExtra` (6/7/8/9/10 by the 22.5°/67.5° bands); case 46
  sets facing + `_flipDraw` only (frameExtra unchanged). Both add eye-glow
  `Lighting.AddLight` heads (`:5413`/`:5457`) — the light lane's, dropped here.
  Case 8 writes the Drill diode rotation — the Drill contract below.
- `ClampToDeadZone` (`:5296-5333`): pushes an aim inside the mount's texture
  half-box out to the box edge. Scutlix half-box (45, 54), Santank (23, 2),
  Drill (80, 80) — `(int)textureSize.Y` is the Y half-extent, `.X` the X.
- `StopAbilityCharge` (`:2922-2931`): type 9/46 → `_abilityCharging = false`,
  `_abilityCooldown = row.abilityCooldown`, `_abilityDuration = row.abilityDuration`.
- `ResetHeadPosition` (`:5283-5294`): if `_aiming`, clear it; `_frameExtra = 0`
  unless type 46; `_flipDraw = false`.
- `AbilityRecovery` (`:2938-2959`) is ALREADY ported
  (`mount.rs::ability_recovery`); it ticks after the fire (C# `:26664`).
- `StartAbilityCharge` (`:2903-2920`) spawns the 441 Scutlix crosshair. Its ONLY
  caller is `Projectile.cs:30662`, the aiStyle-76 REMOTE-owner branch
  (`Main.myPlayer != owner`). Single-player never reaches it, so the 441
  crosshair + `StartAbilityCharge` are DEAD in the port (the f6
  `EmitGolfCartSmoke` dead-code precedent) — documented, not ported. The live
  `_abilityCharging` writer is the scan's else-arm (`:4939`), ported.
  **The 453 is NOT SP-dead** — see §8 for the corrected two-crosshair scope
  record (the earlier blanket "441/453 dead" line was half wrong).

### The fire geometry (UseAbility, ported)

- Scutlix (case 9): `vector6 = (pos.X + width/2, pos.Y + height)` (bottom-centre);
  `num3 = (_frameExtra - 6) * 2` indexes the eye pair; each eye offset is the
  `scutlixEyePositions` table MINUS `scutlixTextureSize (45, 54)`
  (`Mount.cs:1398-1421`), mirrored by facing, plus `xOffset`/`yOffset`. The aim
  `normalize(ClampToDeadZone(target) - eye) * 14` spawns 606 at `eye + aim`.
- Santank (case 46): gift at `Center + (width * -direction, 26)`, velocity
  `(0, -4).RotatedByRandom(0.1)` (TWO `NextDouble` draws), spawn 930 dmg 120 +
  Item89 at the gift position `volumeScale 0.2` — only when `cooldown <= 10`.
  Missile at `(pos.X + width/2 + width*direction, pos.Y + height - 12)`, velocity
  `SafeNormalize(ClampToDeadZone(target) - muzzle) * 12` then
  `.RotatedByRandom(0.2)` (TWO more `NextDouble` draws), spawn 14 dmg 100 +
  Item11 at the muzzle `volumeScale 0.2`. Gift draws precede missile draws.

The `mousePosition` argument is the SCAN's target centre, not the cursor — the
auto-fire aims at the locked NPC. All four `NextDouble` draws ride the
`UpdatePlayers` stream (`Main.cs:17754`, the fire runs inside `Player.Update`).

## 2. Other mount ABILITIES — census + the two smallest ported

Beyond the three `UseAbility` arms, the mount "abilities" are the dash-contact
damage and the slime stomp, all in `Player.cs` (NOT `Mount.cs`), all gated on
`CollideWithNPCs`/`ApplyDamageToNPC` — the combat seam the port lacks:

| mount(s) | site | effect | status |
| -------- | ---- | ------ | ------ |
| 10 Unicorn / 47 WoFGoat / 40-42 horses | `Player.cs:20108-20130` | dash contact, dmg 60 (horses 30) × minionDamage, kb 10 (horses 7), sourceItem 4787 | UNPORTED (combat seam) |
| 44 Pirate Ship | `:20131-20141` | dash contact dmg 100, kb 12, sourceItem 4792 | UNPORTED |
| 45 Spooky Wood | `:20142-20142` | dash contact dmg 120, kb 12, sourceItem 4793 | UNPORTED |
| 14 Basilisk | `:20153-20163` | charge contact dmg 90, kb 10, sourceItem 3771 | UNPORTED |
| 17 (item 4264) | `:20164-20174` | contact dmg 40, kb 10, sourceItem 4264 | UNPORTED |
| 3 Slime / 50 Queen Slime | `JumpMovement :20533-20565` | falling stomp dmg 40, kb 5, bounce vy=-10, sourceItem 2430 | UNPORTED |
| 8 Drill | `UseDrill :3059-3252` | beam mining (PickTile/SmoothSlope/Hurt) | UNPORTED (contract below) |

The dash/stomp abilities all need `CollideWithNPCs` (`:20381-20409`) →
`ApplyDamageToNPC` (`:20432-20461`) → `NPC.StrikeNPC` + the banner/OnHit/OnKill
tails + `GiveImmuneTimeForCollisionAttack` (`:20494-20511`, the `_immuneStrikes`
3-strike gate) + `CanNPCBeHitByPlayerOrPlayerProjectile` (`:20513-20529`, the
`CountsAsCritter` set + `dontHurtCritters`). That is a combat lane, not a sound
lane. **The two smallest abilities are the two projectile ones — Scutlix (9)
and Santank (46)** — which share the auto-fire machine and need no combat seam;
both are ported here (item 1's Santank + item 2's smallest-two). The rest are
listed unported above; their dash DUST is already f6's (batch 1).

### Drill (8) contract — for the mount lane

`UseAbility` case 8 only toggles `_abilityActive` and spawns the 453 crosshair
(aiStyle 76). The real consumer is `UseDrill` (`Mount.cs:3059-3252`, called
`Player.cs:28290` when `mount.Type == 8`): it walks `DrillMountData.beams[8]`
(`Mount.cs:36-58`), picks a tile via `DrillSmartCursor_Blocks`/`_Walls`, calls
`Player.PickTile`/`PickWall` with `drillPickPower 210`, `Tile.SmoothSlope`, the
Demon Altar pre-hardmode `Hurt` (`:3092-3096`), and spawns the beam dust
(`:3113`/`:3170`, type 230). `UpdateDrill` (`:3036-3057`) decays the beam
cooldowns + lerps the diode rotation. The port carries NO `DrillMountData`
payload (`mount.rs:2118` notes it M2-MOUNT-4). Contract: the mount lane ports
`DrillMountData` + `UpdateDrill` + `UseDrill` + the 453 aiStyle-76 crosshair;
the two beam-dust sites are ALREADY staged in f6's `mount_dust.rs` (see §4).
The Drill's `AimAbility` case 8 (diode rotation target) rides that payload.

## 3. Meowcart × minecart-track interplay — speed→pitch census

The task asked whether the cart-on-tracks speed scales the sound PITCH. It does
NOT. Full census of the cart ride's sound sites:

- Rolling rattle Item55 (`Player.cs:19870`/`:19928`): plays on EVERY gate-passing
  tick (`mount.Active && AnyTrackRider && !onWrongGround && onTrack &&
  itemAnimation == 0 && vy == 0` and the direction-opposite vx sign). Case-2
  pitch is `-Next(-20, -11) * 0.02` (`LegacySoundPlayer.cs:616`) — a FIXED random
  window, no velocity term. Volume ×0.75×0.75 (`:554-557`). Speed only gates the
  SPARK density: `|vx| > maxRunSpeed*0.66` → three 50% rolls, `> *0.33` → two
  2-in-3 rolls, else one guaranteed spark (`:19873-19902`). Ported:
  `minecart.rs::rolling_wheel_sparks` (the f64 threshold compares verbatim).
- Landing Item53 (`DelegateMethods.cs:250`): pitch `Next(-20, -11) * 0.02`
  (`:613`), fixed. Bumper Item56 (`:255`): default `Next(-6, 7) * 0.01` (`:632`),
  fixed. Meowcart case-37 (`Mount.cs:661`/`:666`): `Next(-40, 41) * 0.01`
  (`LegacySoundPlayer.cs:1333`), fixed. Fart trio: the Item16/53/56 case-2 windows.

**No cart sound pitch scales with speed anywhere in the C#.** The port's
minecart pair (W4-c14) + `minecart.rs` already reproduce every fixed window; the
speed→spark-density ladder is ported. Nothing to add — the census closes the
question with a witness pinning the rattle's fixed window (§5).

## 4. f6 co-lane (origin/task/W4-f6-dust-world-3) — keep-one

f6 DELIVERED the 27-site Mount.cs dust census (`docs/research/65`, commits
`885a6bfa`/`d6f555b8`/`9dec91b0`/`680ac9d8`) as staged emitters in
`mount_dust.rs`. Overlap check against this ability port:

- f6's 27 sites are ALL `NewDust*`/`CloneDust` in `UpdateFrame`/`UpdateEffects`/
  `DoSpawnDust`/`UseDrill`/the minecart effects. NONE is in `UseAbility`,
  `AimAbility`, `ClampToDeadZone`, `StopAbilityCharge` or the scan — the ability
  machinery has no dust. The `AimAbility` eye-glow `Lighting.AddLight` heads
  (`:5413`/`:5457`) are the LIGHT lane's, dropped by both.
- The Drill beam dust `:3113`/`:3170` is f6's (batch 1, staged unwired). This
  port does NOT touch `UseDrill`; the Drill contract (§2) hands the consumer to
  the mount lane, which wires f6's staged beam emitters. KEEP-ONE: f6 owns those
  two sites; no re-port here.
- The dash-contact mounts' dash DUST (Unicorn/Goat ring `:3878`, Pirate/Spooky
  `:3939`/`:4000`, Basilisk `:4103`/`:4123`) is f6's batch 1. This port leaves the
  dash-contact DAMAGE unported (§2); when the combat lane lands it, f6's staged
  dash emitters wire to it. KEEP-ONE: f6 owns the dash dust.

No site is claimed by both lanes. The ability port adds only the fire/aim/scan
state + the two Santank cues + the 606/930 projectile consumers.

## 5. Verification (code-only — WRITTEN NOT RUN)

Witnesses (no cargo/rustc/rustfmt/tests/graphify invoked, the lane directive):
- `mount.rs`: the dead-zone clamp, the Santank/Scutlix aim bands, the fire
  geometry + the 4-draw RNG order, stop-charge / reset-head.
- `audio.rs`: the Santank case-2 façade at `volumeScale 0.2` (Item89/Item11),
  gate-before-draw, the `Item_89` bank join.
- `projectile/data.rs` + `projectile.rs`: the 606/930 rows, the 930 homing reach
  650 + the plain-rocket kill/prepare arms.
- `world_sim/player.rs`: the scan→aim→fire driver, the cooldown cadence
  (Santank fires every 10 ticks, gift every 40), the no-target reset.
- `minecart.rs`: the rattle's fixed pitch window (no speed term).

## 6. f6 mutual receipt (SEAL — f6 tip `ebdcbd96d`, research/67 FINAL)

Both lanes' keep-one boundaries re-read at seal time; every item below was
verified against the f6 branch tip and the C#, not just the commit messages:

1. **UseDrill activation seam = c14's contract, f6's published pin.** f6
   `a67d13638` publishes the exact seam: `Player.cs:28288-28291` — after
   `PlayerFrame` under the `UpdatePlayers` swap, `if (this.mount.Type == 8)
   { this.mount.UseDrill(this); }` (re-read verbatim at seal). This MATCHES
   §2's Drill contract: the mining-machine lane ports `DrillMountData` +
   `UpdateDrill` + `UseDrill` + the 453 crosshair and calls at that seam;
   the `_type == 8 && _abilityActive` gate is inside `UseDrill` (`:3061`).
   c14 RECEIPTS the seam as the contract's call site — no c14 code touches it.
2. **SpawnFartCloud STAGED = f6's body, c14's delegate seam.** f6 `573b7e7f2`
   stages `minecart.rs::spawn_fart_cloud` (`#[allow(dead_code)]`, the 3 gores
   `Main.rand.Next(435, 438)` + the 53-only 15-dust-107 tail,
   `DelegateMethods.cs:258-306`). The LIVE callers are c14's three sound
   delegates (`minecart.rs::minecart_jumping_sound_fart` `:1368` /
   `minecart_landing_sound_fart` `:1385` / `minecart_bumper_sound_fart`
   `:1311`), wired at `player.rs:7556`/`:7572`/`:7628`. The `useDelay` split
   is part of the seam: Jumping/Landing pass `useDelay: false`
   (`DelegateMethods.cs:311`/`:318` — the cloud fires EVERY delegate call),
   Bumper uses the default `true` (`:325` — the `fartKartCloudDelay` 20-tick
   gate, `Player.cs:1614`/`:1616`, decremented per track-rider tick
   `:28098`). At merge, f6's staged body wires INTO c14's delegate call
   sites with that split; the delay field + decay are f6's (the visual
   payload), the cue order stays c14's. KEEP-ONE: no second cloud.
3. **The 4 minecart spark delegates = f6's dust, c14's sounds only.** f6
   `573b7e7f2` ports SparksFart (51, dust-211) / SparksTerraFart (53, the
   queue-time 50% branch + dust-107) / SparksMech (11, dust-260) / SparksMeow
   (33, the 213 body) — `DelegateMethods.cs:328-414`. c14's port of the same
   delegate family is the CUE half only (Item16/53/56 pairs + the Meowcart
   case-37 pair); zero dust in c14's cart code. Receipt: the two halves
   compose at the same delegate call sites, no overlap.
4. **tile_draw_rand = ONE field, f6's verdict accepted.** f6 `8a34f6642`:
   `TileDrawing._rand` is ONE private field (`TileDrawing.cs:104`/`:191`) on
   the ONE `Main.TilesRenderer` — g68's drip gate and f6's pylon gates share
   it renderer-lifetime; ONE `App.tile_draw_rand` (f6 branch `app/mod.rs:723`,
   world-seeded `:3926`), drips-before-pylons merge-order pin. c14 has NO
   `TileDrawing._rand` consumer (grep of the c14 tip: zero hits) — receipt:
   c14 adds no field, no rename; if a future c14 sound site ever needs the
   draw stream it joins f6's single field. `Mount.cs` has zero private RNG
   fields (all `Main.rand` UpdatePlayers), so the ability port's draws are
   unaffected by the renderer-stream split.

## 7. Combat-seam OWNER CONTRACT + task seed (SEAL)

The dash-contact / slime-stomp abilities (§2 table) gate on a player→NPC
contact-damage seam the port lacks. **Which lane owns it? NONE of the three
live lanes — it is unclaimed, and this section claims it for a new seed.**
Re-read at seal against both co-lane tips:

- **d116 (`task/W4-d116-player-hurt-armor`, tip `dd9ad7da7`) owns INBOUND
  damage only**: research/04's scope is `Player.Hurt` (`Player.cs:38197-38567`,
  the whole inbound pipeline) + the `StrikeNPC_Inner` damage MATH
  (`damage.rs`, defence ×0.5 / crit ×2 / floor 1, landed `bafd827c1`-era).
  It does NOT port `CollideWithNPCs`/`ApplyDamageToNPC` — its census lists
  NPC contact only as a `hurt` CALLER (`npc.rs:12333`), the reverse direction.
- **g59 (`task/W4-g59-npc-death-effects-1`, tip `d277f782b`) owns POST-DEATH
  visuals only**: `npc_death_dust.rs` + the `checkDead` gore/dust activations.
  It consumes a death, it does not deal damage.
- **The seam itself** — `CollideWithNPCs` (`Player.cs:20381-20409`),
  `ApplyDamageToNPC` (`:20432-20461`), `GiveImmuneTimeForCollisionAttack`
  (`:20494-20511`), `CanNPCBeHitByPlayerOrPlayerProjectile` (`:20513-20529`)
  — is in NO pushed lane's diff. Grep of the c14 tip: no
  `collide_with_npcs`/`apply_damage_to_npc`/`immune_strikes` anywhere.

**Task seed M5-COMBAT-8 — the player→NPC contact-damage seam (M).** (Next
free id after research/04's M5-COMBAT-1..7.) Port, in C# order:

1. `CollideWithNPCs(myRect, Damage, Knockback, NPCImmuneTime, PlayerImmuneTime,
   sourceItemId)` — the `Main.maxNPCs` scan, first-hit `break`, the gate chain
   `active && !dontTakeDamage && !friendly && immune[whoAmI]==0 &&
   CanNPCBeHitByPlayerOrPlayerProjectile && rect.Intersects &&
   (noTileCollide || Collision.CanHit)`; direction from the velocity sign
   (fallback `base.direction`); the local-only `ApplyDamageToNPC`; then
   `npc.immune[whoAmI] = NPCImmuneTime` + `GiveImmuneTimeForCollisionAttack`
   on EVERY hit (remote players grant i-frames too).
2. `ApplyDamageToNPC` — banner buff ×`DamageDealt.Sample(difficulty)`,
   `OnHit`, armour pen (`checkArmorPenetration`, melee:false),
   `NPCKillAttempt`/`NPCDamageTracker.SetSourceForNextHit`, `StrikeNPC`,
   `accDreamCatcher` DPS, `lastCreatureHit` banner id, `OnKillNPC` on death.
3. `GiveImmuneTimeForCollisionAttack(time)` — the `_immuneStrikes` 3-strike
   gate: `<=20` ticks since the last grant increments, else resets to 1;
   `_timeSinceLastImmuneGet = 0`; grant `immune/immuneNoBlink/immuneTime`
   only when `_immuneStrikes < 3 && (!immune || immuneTime <= time)`. The
   counter ticks at `Player.cs:24663-24666` (cap 10000).
4. `CanNPCBeHitByPlayerOrPlayerProjectile` — the `dontHurtCritters` /
   minion/sentry-shot flag, the trap override, `IsCritterThatIsHostileToPlayers`,
   the `NPCID.Sets.CountsAsCritter` set.

**Boundary pins (the contract):**
- vs **d116**: the strike MATH is d116's landed `damage.rs` — call it, never
  re-derive. The i-frame grant writes the SAME General `immune/immuneTime`
  pair d116's `hurt_cd` stand-in models (`player.rs:2279` + `damage.rs:172`)
  — EXTEND that stand-in (plus `immuneNoBlink`), never add a second immunity
  system. The new `_immuneStrikes`/`_timeSinceLastImmuneGet` fields are
  M5-COMBAT-8's (no lane has them).
- vs **g59**: death VISUALS stay g59's — M5-COMBAT-8 ends at
  `Npc::strike_melee`/`strike_pooled` (`npc.rs:4468`/`:4512`) + the
  OnHit/OnKill/banner tails; `checkDead`'s gore/dust activations are g59's
  `npc_death_dust.rs`, untouched.
- vs **f6**: the dash DUST (Unicorn/Goat `Mount.cs:3878`, Pirate/Spooky
  `:3939`/`:4000`, Basilisk `:4103`/`:4123`) is f6's staged batch 1 — the
  combat lane wires f6's emitters, no re-port (§4 keep-one).
- **Consumers** (the mount lane's, after the seam lands): the five dash arms
  (`Player.cs:20108-20174`, `HorizontalMovement` — Unicorn/WoFGoat/horses
  60/30 dmg kb 10/7 item 4787; Pirate 100 kb 12 item 4792; Spooky 120 kb 12
  item 4793; Basilisk 90 kb 10 item 3771; 17: 40 kb 10 item 4264,
  PlayerImmuneTime **12** not 6; all NPCImmuneTime 30, PlayerImmuneTime 6
  otherwise) + the slime stomp (`JumpMovement` `:20533-20565` — an INLINED
  copy of the same scan, NOT a `CollideWithNPCs` call: dmg 40×minionDamage,
  kb 5, `npc.immune = 10`, the bounce `velocity.Y = -10f` BEFORE the
  i-frame grant(6), item 2430, gated `IsConsideredASlimeMount && wetSlime==0
  && velocity.Y > 0`). The stomp's inline scan must share the seam's gate
  helper, not duplicate it (house style: one mechanic, one helper).

## 8. The 441/453 crosshairs — MP-scope record (SEAL, with correction)

The earlier ANALYSIS/ROADMAP line "the 441/453 crosshairs are
single-player-dead" was HALF WRONG; re-read whole at seal, the corrected
record (both are aiStyle 76, Damage 0, KnockBack 0, `heldProj`):

- **441 (Scutlix charge crosshair): SP-DEAD, confirmed.** The ONLY spawn is
  `StartAbilityCharge`'s local arm (`Mount.cs:2909-2913`), and the ONLY
  caller of `StartAbilityCharge` in the whole tree is `Projectile.cs:30662`
  — the aiStyle-76 REMOTE-owner branch (`Main.myPlayer != owner`, a 441
  already in flight whose owner stopped charging). Single-player can never
  have a remote-owner projectile, so the spawn is unreachable. The live
  `_abilityCharging` writer is the scan's else-arm (`Mount.cs:4939`),
  ported. `Projectile.Kill()` also calls `StopAbilityCharge` for any 441
  (`Projectile.cs:73394-73396`) — moot while no 441 exists.
- **453 (Drill crosshair): SP-LIVE in the C#, PORT-dead only.** The spawn is
  `UseAbility` case 8's local activate arm (`Mount.cs:3327`), reached in
  single-player through the Drill toggle: `ItemCheck_HandleMount`
  (`Player.cs:52922`, the `mount.Type == 8` arm) → `controlUseItem ||
  controlUseTile` → `StartChanneling` → on `releaseUseItem && releaseUseTile`
  → `UseAbility(this, Vector2.Zero, toggleOn: true)` (`:52944`). It is dead
  IN THE PORT only because the Drill (8) is unported (§2 contract) — it is
  NOT dead code, and the mining lane MUST port it with `UseDrill`.
- **MP scope (recorded for the transport lane):** the aiStyle-76 machine
  (`Projectile.cs:30620-30670`) splits on `Main.myPlayer == owner`: local =
  mouse-track + `AimAbility` + the `!channel` → `UseAbility(false)` + `Kill`
  release; remote = position-from-ai + the 441 `StartAbilityCharge` /
  453 `!AbilityActive` → `UseAbility(false)` arms. The `Kill()` tail
  (`:73390-73396`) carries the remote 453-off (`owner != myPlayer &&
  AbilityActive` → `UseAbility(false)`) + the 441 `StopAbilityCharge`.
  These remote arms + the 441 spawn are the MP-only surface; the 453 spawn
  + the local release are single-player and ride the Drill contract.

## 9. Mount-abilities family bible (SEAL — the canonical index)

This file is the family's single census. Every member, its owner lane and
its status at seal (extend THIS file for new findings; do not fork a new
census):

| member | C# site | owner lane | status |
| ------ | ------- | ---------- | ------ |
| Scutlix (9) auto-fire + 606 laser | `Mount.cs:3254-3289`/`:4879-4947`/`:5335-5465` | **c14** | PORTED (`8ff589aad`), 606 tile-latch deviation (ANALYSIS) |
| Santank (46) auto-fire + 930 gift + 14 missile + the two cues | `Mount.cs:3290-3313` | **c14** | PORTED (`0e290cae3`/`8ff589aad`), INT-overload façade |
| Drill (8) toggle + 453 crosshair + `UseDrill` mining | `Mount.cs:3314-3336`/`:3036-3252`, `Player.cs:52922-52948`/`:28288-28291` | **mining/mount lane** (contract §2 + §6.1 seam) | UNPORTED; beam dust f6-staged |
| dash contact ×5 (10/47/40-42, 44, 45, 14, 17) | `Player.cs:20108-20174` | **M5-COMBAT-8 seed** (§7) | UNPORTED; dash dust f6 batch 1 |
| slime stomp (3/50) | `Player.cs:20533-20565` | **M5-COMBAT-8 seed** (§7) | UNPORTED |
| the contact-damage seam itself | `Player.cs:20381-20529` | **M5-COMBAT-8 seed** (§7, unclaimed by d116/g59) | UNPORTED |
| 441 charge crosshair + `StartAbilityCharge` | `Mount.cs:2903-2920`, `Projectile.cs:30662` | MP transport lane (§8) | SP-DEAD, documented not ported |
| Meowcart/minecart speed→pitch question | `Player.cs:19870`/`:19928` | **c14** | CLOSED — speed-invariant (§3, witness `968ae4edd`) |
| cart/mount sound sites (rattle, landing, bumper, fart trio, case-37) | research/82 census | **c14** | PORTED (the W4-c14 wave) |
| mount dust (27 sites) + fart cloud + 4 spark delegates | research/65/67 | **f6** | PORTED/STAGED on f6 (`ebdcbd96d`), §6 receipts |
| charge eye-glow lights | `Mount.cs:5413`/`:5457` | light lane | dropped by both, seam accessor kept |

Lane hashes at seal: c14 tip `b6c99ed11` (+ the `origin/main` sync merge),
f6 tip `ebdcbd96d`, d116 tip `dd9ad7da7`, g59 tip `d277f782b`.
