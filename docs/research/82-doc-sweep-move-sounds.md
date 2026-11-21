# 82 — DOC-SWEEP lane 11: movement / tile sounds (W4-c14 continuation)

Code-only census + the two W4-c14 deviation closures, 2026-11-21. Branch
`task/W4-c14-tile-break-footstep-sounds` (base DONE `4efab5b8`). No compiler
invoked — witnesses WRITTEN, NOT RUN. This lane re-censuses the Player
movement-sound family against the C# and closes the two deviations the
W4-c14 wave recorded (ANALYSIS "The tile-break fail + movement-sound wave"):
the unported vanity-rocket producer and the 53/55 playing guards.

## 1. The vanity-rocket producer — CLOSED (was deviation 1)

The W4-c14 rocket-cadence cue (`Player.cs:26884-26896`) read the GAMEPLAY
tier (`rocketBoots`) as a stand-in because `vanityRocketBoots` had no
producer. The C# cue reads the VANITY tier:

```
26886  if (this.vanityRocketBoots == 1 || this.vanityRocketBoots == 5)
26888      this.rocketSoundDelay = 30;  SoundEngine.PlaySound(SoundID.Item13, ...)
26891  else if (this.vanityRocketBoots == 2 || ... == 3 || ... == 4 || ... == 6)
26893      this.rocketSoundDelay = 15;  SoundEngine.PlaySound(SoundID.Item24, ...)
```

### Producer census (every `vanityRocketBoots` writer)

| Site | Method | Write | Condition |
| --- | --- | --- | --- |
| `Player.cs:2760` | field | `public int vanityRocketBoots` | — |
| `:18354` | `ResetEffects` | `= 0` | per-tick clear |
| `:26630` | `Update` mount block | `= 0` | `mount.Active` |
| `:14420` | `ApplyEquipFunctional` | `rocketBoots = (vanityRocketBoots = 2)` | Fairy Boots 3993 |
| `:15023` | `ApplyEquipFunctional` | `= 1` (both) | Rocket Boots 128 |
| `:15222` | `ApplyEquipFunctional` | `= 2` (both) | Spectre Boots 405 |
| `:15380` | `ApplyEquipFunctional` | `= 2` (both) | Lightning Boots 898 |
| `:15386` | `ApplyEquipFunctional` | `= 3` (both) | Frostspark Boots 1862 |
| `:15393` | `ApplyEquipFunctional` | `= 4` (both) | Terraspark Boots 5000 |
| `:15400` | `ApplyEquipFunctional` | `= 5` (both) | Hellfire Treads 4874 |
| `:13090` | `UpdateEquips` tail | `= 6` | Red's Leggings 668 in leg slot 2 OR vanity slot 12 (`GetEffectiveArmor(2/12).type == 668`) |
| `:36709` | `UpdateBootVisualEffects` | `= 1` | Rocket Boots 128 |
| `:36717` | `UpdateBootVisualEffects` | `= 2` | Spectre 405 / Lightning 898 |
| `:36721` | `UpdateBootVisualEffects` | `= 5` | Hellfire 4874 (+ `hellfireTreads`) |
| `:36726` | `UpdateBootVisualEffects` | `= 3` | Frostspark 1862 |
| `:36730` | `UpdateBootVisualEffects` | `= 4` | Terraspark 5000 |
| `:36748` | `UpdateBootVisualEffects` | `= 2` | Fairy 3993 (+ `fairyBoots`) |

`UpdateBootVisualEffects` (`:36704-36751`) is called from TWO paths:
`ApplyEquipFunctional`'s visible half (`:14378-14380`, gated
`!hideVisibleAccessory[itemSlot]`) and `ApplyEquipVanity` (`:14234`, the
social slots 13..20 loop `:13009-13015`, NO hide gate). The functional
equip rows ALSO set the tier directly (the `rocketBoots = (vanityRocketBoots
= N)` chained assignment), so a worn boot sets both tiers equal even when
hidden; the vanity pass re-derives the visual tier from the visible boot.

**Tier 6 is the Red's Leggings 668** (a vanity leg item, `itemdata.rs:834`
`leg_slot: 25, vanity: true`), NOT a "Bee costume" — the W4-c14 note
mislabelled it. The bee-costume rocket VISUAL is `RocketBootVisuals`
(`:29330-29428`, dust type 186 for tier 6) and the `SpawnFastRunParticles`
wings-3/tier-6 dust (`:20217`); both are particle seams, not sound.

### Port (commit `4cbd37bf`)

- `player.rs::vanity_rocket_boots` field (`:2760`), spawn 0.
- `ResetEffects` clear (`:18354`) beside `rocket_boots = 0`; both mount
  blocks zero it (`:26630` — the cart block + `update_mount`).
- The seven functional equip rows set both tiers equal (the chained
  assignment); the `TR_BOOTS` pin stands in for the equip row (sets both).
- `boot_visual_tier(kind)` ports the `UpdateBootVisualEffects` tier arms
  (`:36704-36751`); the functional loop calls it behind the
  `!hide_visible_accessory[m]` gate (`:14378`), the social loop 13..20 calls
  it ungated (`:13009-13015`). The `CancelAllBootRunVisualEffects` dash
  flags (`sailDash`/`coldDash`/`desertDash`/`fairyBoots`/`hellfireTreads`,
  `:36664-36671`) + the Hermes-54 cancel stay run-particle visual seams.
- The Red's Leggings 668 row (`:13088-13091`) runs AFTER the loops (wins).
- The cue arm reads `vanity_rocket_boots` (`:26886`/`:26891`), tier 6 →
  Item24 cadence 15.

Witness `player.rs::w4_c14_the_vanity_rocket_tier_produces_the_cue`: the
equip rows set both equal, the social slot re-derives, 668 wins tier 6 over
a tier-1 pin, and the divergent cue plays Item24/15 while the gameplay tier
stays the thrust source. The existing cadence witness
(`w4_c14_the_rocket_cue_rides_its_own_cadence`) is unchanged (the pin sets
both tiers equal, so its pinned tiers still drive the cue).

## 2. The 53/55/159 playing guards — CLOSED (was deviation 2)

The W4-c14 cart-rattle/landing/bumper cues (Item53/55/56) and every item-use
play ride `play_use_sound` → `item_use_case2`. The C# case 2 refuses a fresh
53/55/159 play while the style's instance is live:

```
552  case 53: case 55:
554      num2 *= 0.75f;  if (num == 55) num2 *= 0.75f;
559      if (this.SoundInstanceItem[num] != null && ...State == SoundState.Playing)
561          return null;                       // BEFORE the pitch draw
582  case 159:
583      if (this.SoundInstanceItem[num] != null && ...State == SoundState.Playing)
585          return null;
587      num2 *= 0.75f;
589  default:
590      if (this.SoundInstanceItem[num] != null) this.SoundInstanceItem[num].Stop();
595  case 9/10/24/26/34/43/103/156/162/205: break;   // stack-allowed, no Stop
607  this.SoundInstanceItem[num] = this.SoundItem[num].Value.CreateInstance();
612  case 53: Pitch = Next(-20,-11)*0.02;  case 55: Pitch = -Next(-20,-11)*0.02;
```

`SoundInstanceItem` is `new SoundEffectInstance[SoundID.ItemSoundCount]`
(`LegacySoundPlayer.cs:72`, `ItemSoundCount = 211` `SoundID.cs:1595`). The
guard reads `.State == SoundState.Playing` — the XNA instance state, NOT a
`Main.soundInstanceItem` field (the W4-c14 task's "SoundEngine instance
check" is this `SoundInstanceItem[]` array on the legacy player).

### Port (commit `4cbd37bf`)

- `sound.rs::item_case2_playing_guard(style, playing)` — the pure decision
  (`matches!(style, 53|55|159) && playing`), the `npc_hit_case3` 57-guard /
  `mech_case28` precedent. `item_use_case2` stays the pure volume/pitch
  decision (its signature is unchanged — every item-use caller is untouched).
- `audio.rs::item_slots: HashMap<usize, rodio::Sink>` — the
  `SoundInstanceItem[]` pool (the `npc_killed_slots` map precedent).
  `play_use_sound` reads the slot's `!sink.empty()` (≈ `State == Playing`),
  refuses a guarded style BEFORE the case-2 pitch draw, then stops +
  re-creates the slot on a fresh play; the stack-allowed styles
  (`:595-605`) never touch a slot (the old source keeps playing).
- Headless (no output device) no slot ever fills, so the guard never trips
  — the engine's fresh-instance behaviour (the case-28 witness precedent).

Witness `audio.rs::w4_c14_the_cart_rattle_guard_refuses_while_the_slot_is_live`:
two headless 55 rattles draw one pitch each; the pure guard refuses 53/55/159
when live and plays them idle, never guards 1/24/37/56/168; a muted rattle is
draw-free.

**Residual D4**: the guard + pool now model the 53/55/159 refusal and the
default-arm Stop/re-create. The per-frame re-pan and the
`SoundInstanceGarbageCollector` remain the standing tracked-sound deviations
(ANALYSIS S1-E-23) — unchanged by this lane.

## 3. Remaining Player movement sounds — census verdicts

### Swim stroke sounds — NONE EXIST (no gap)

`swimTime` (`Player.cs:2768`) is a pure animation timer: set 30 on the
merman swim jump (`:20659-20661`) and the flipper swim (`:20683-20685`),
decremented in `PlayerFrame` (`:35757-35762`, zeroed when `!wet`), read by
the swim pose (`:36221`/`:36531-36539`). NO `PlaySound` rides it. The only
water sounds are the liquid entry/exit SPLASH (case 19, `:27806`/`:27819`/
`:27832`/`:27846` entry, `:27900`/`:27913`/`:27926`/`:27940` exit) — already
ported (`player.rs::pending_splash_sound` → `audio.rs::play_splash_sound`,
S1-E-21f, the case-19 playing guard + shimmer ×0.75 + pitch). The shimmer
EMERGENCE splash (`:18306`, case 19 style 3 on `shimmerTransparency == 0.8`)
rides the shimmer-absorption state machine (`shimmering`), which is the
shimmer-emergence lane's host feature — not a movement sound this lane owns.

### Landing thud (fall damage) — the hurt sound IS the landing sound

The fall-damage block (`Player.cs:25360-25375`) plays NO dedicated thud — it
calls `Hurt(ByOther(0), num17, 0)`, whose `PlayHurtSound` is the landing
audio. The port's `hurt_fall` (`player.rs:4496`) → `hurt_inner` queues
`pending_hurt_sound = "Player_Hit_0"` (`:4463`) — ALREADY PORTED. The
petrified fall (`:25347-25358`) likewise rides `Hurt`. There is no separate
fall-thud cue to port.

**The one floor-landing sound** is `MakeFloorDust`'s poop/poulder thud
(`Player.cs:23830`: `SoundEngine.PlaySound(SoundID.Item177, (int)Center.X,
(int)Bottom.Y)`), gated `Falling && (type == 666 || type == 712)` (PoopBlock
666 / Poulder 712, both `tileSolid` `Main.cs:9584`/`:7469`). It is a
FLOOR-VISUALS sound (the dust lane's `MakeFloorDust`, `:23790`), reached only
through `FloorVisuals(Falling)` (`:23742`, gated `!wet && !mount.Cart` + the
slope carve-out `:23736-23738`). The port's `floor_visuals`
(`player.rs:10707`) samples the flags ONLY — `MakeFloorDust` is explicitly
deferred to the particle tasks ("The dust visuals (MakeFloorDust) ... stay
with the particle tasks"). The `Falling` flag (`:28073-28081`,
`|velocity.Y| > gravity` captured pre-integration) has no port latch. This
is a floor-visuals-lane gap (the Item177 cue + the `Falling` seam + the
666/712 stink buff `:23732-23734`), NOT a movement-core gap — deferred to
that lane with this citation. `Item_177` is already in the bank
(`audio.rs:47`, `soundid.rs:330`) and the kill-cue map
(`world_sim/editing.rs:5541`), so the future port has its asset.

### Grappling hook fire/reel — BOTH ALREADY PORTED (i39/S1-E-21f)

- **Fire**: `QuickGrapple` (`Player.cs:6408`) plays `item.UseSound` at
  `base.position` after the fire-time gates. Ported:
  `world_sim/player.rs:779-789` (`quick_grapple_item_kind` →
  `initial_use_sound` → `play_use_sound`, the top-left position, the
  `UpdatePlayers` stream). The `TR_HOOK` pin (no granting item) is silent,
  like the C#'s null-item return (`:6320-6323`).
- **Latch**: `AI_007_GrapplingHooks` (`Projectile.cs:52757`) plays
  `PlaySound(0, i*16, j*16)` (the case-0 Dig). Ported:
  `grapple.rs:670` (`slot.wants_sound = Some((0, 0, i*16, j*16, 0.0))`) →
  `projectile.rs:19786` forwards to `pending_proj_sounds` →
  `world_sim/projectiles.rs:468-469` (`play_place_sound((0,0), ...)`, the
  `Next(3)` variation + `Next(-10,11)*0.01` pitch on `UpdateProjectiles`).
- **Reel**: the retract arm (`Projectile.cs:52787-52842`) plays NO sound —
  it only scales the velocity toward the player and kills at `< 24f`
  (`:52834-52837`). There is no reel cue to port.

### Mount ambient loops beyond d80b — NONE in Mount.cs (no gap)

The d80b lane (W4-d80, commits `83a43d36`..`075b954d`) ported the
tracked-sound ENTITY loops: drill/chainsaw aiStyle-20 (Item190/15 +
Item189/20, `Projectile.cs:25510-25522`), Laser Drill 445 (Item132/23,
`:66246-66250`), the DD2 NPC loops (Betsy wind/breath, portal, kobold,
wither), the bomb fuse, the vampire sizzle. Those are PROJECTILE/NPC sounds,
not mount sounds.

`Mount.cs` has exactly FIVE `SoundEngine.PlaySound` sites (full census):

| Site | Cue | Status |
| --- | --- | --- |
| `:661` MeowcartLandingSound | `PlaySound(37, ..., 5)` | PORTED (W4-c14, `play_meowcart_sound`) |
| `:666` MeowcartBumperSound | `PlaySound(37, ..., 3)` | PORTED (W4-c14) |
| `:3300` Santank ability | `Item89.SoundId/Style, 0.2f` | PORTED (W4-c14 M2-MOUNT-4, `play_mount_ability_sound`) |
| `:3311` Santank ability | `Item11.SoundId/Style, 0.2f` | PORTED (W4-c14 M2-MOUNT-4) |
| `:6285` SetMount Chillet | `SoundID.PalChillet` | PORTED (S1-P-i41, `play_chillet_sound`) |

The cart delegations (`:123-125` Jumping/Landing/Bumper + the fart trio
`:1100-1110` + the Meowcart pair `:1014-1015`) are the W4-c14 cart cues —
all ported. There is NO mount ambient LOOP in `Mount.cs` (no
`PlayTrackedSound`, no looped `SoundEngine` call). The drill mount's
`UpdateDrill` (`:3036-3057`) is pure beam-cooldown state — its sound is the
aiStyle-20 projectile loop d80b ported. The Santank's two ability cues
(`:3300`/`:3311`) ride `UseAbility` case 46 — **PORTED (W4-c14 M2-MOUNT-4,
research/371)**: the Scutlix/Santank auto-fire machine is ported whole
(`mount.rs` aim/dead-zone/scan + `Player::update_mount_ability`), the cues
play through `play_mount_ability_sound` — the INT overload
(`SoundEngine.cs:102`, NO `GetRandomPitch` draw), volumeScale 0.2 after the
case-2 tweak (`LegacySoundPlayer.cs:1411`), one `Next(-6,7)*0.01` pitch roll
on the `UpdatePlayers` stream. `Item_89` joined the SFX bank (the 88/90 gap).
The Drill (8) ability stays unported (the `UseDrill` mining consumer is the
mount lane's — research/371 §2 contract). **No mount ambient loop beyond d80b
exists to port.**

## 4. Other movement-window PlaySound sites (census completeness)

The remaining `Player.cs` movement-window plays are NOT movement-core sounds
and are either ported or belong to other lanes:

- `:20214` fast-run footstep (`hermesStepSound`, case 17 / Item24) — PORTED
  (`play_step_sound`, S1-E-21 wave D; the Moon Lord legs 140 override
  `:35959-35964`).
- `:20793-21004` extra-jump chain (case 16) + `:20846` Fart Item16 — PORTED
  (`play_double_jump_sound` / `pending_fart_sound`, S1-E-21f).
- `:26850` wing flap Item32 — PORTED (`pending_flap_sound`, S1-E-21f).
- `:26703/:26710/:26723/:26730` gravity-flip Item8 — the gravity-flip
  feature (`gravControl`/`gravControl2`) is unported (no gravity flip in the
  port); gravity-flip-lane gap.
- `:23257` drowning gasp (case 23) — PORTED (`play_drown_sound`, S1-E-21f).
- `:22287` Wall of Flesh proximity (case 4 style 10) — the WoF boss lane.
- `:28333/:28396/:28401/:28406` accessory recharge cues (TimerCrit /
  SnappingStone / Pyroclast / ArmletOfRuin) — the accessory-recharge lane.
- `:31724` shield-parry cooldown (case 25) — PORTED (W4-f48).
- `:19552` max-mana (case 25) — PORTED (W4-f48).
- `:28820/:28827` smart-cursor toggle (case 12) — the smart-cursor lane.
- `:32598` ammo-box buff (Item4) — the tile-interaction lane.

## 5. Verification

Code-only (the Lead's constraint): no cargo/rustc/rustfmt, no test runs, no
graphify refresh. Witnesses WRITTEN, NOT RUN:
- `player.rs::w4_c14_the_vanity_rocket_tier_produces_the_cue`
- `audio.rs::w4_c14_the_cart_rattle_guard_refuses_while_the_slot_is_live`

The existing `w4_c14_the_rocket_cue_rides_its_own_cadence` witness is
unchanged in behaviour (the pin sets both tiers equal). `item_use_case2`'s
signature is unchanged — every item-use caller compiles against the same
pure decision; the guard + pool live in the facade.

## 6. Commits

- `4cbd37bf` — the vanity-rocket producer + the 53/55/159 playing guards
  (3 files: `sound.rs`, `audio.rs`, `player.rs`).
- (this report + the doc/anchor updates) — DOC-SWEEP lane 11 census.
