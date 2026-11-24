# 96 — DOC-SWEEP lane 18 (proj misc): the five folded deviations, ported

W4-d8 DOC-SWEEP phase, branch `task/W4-d8-proj-ai-71-80` (continues the
DONE `e66b41b0` AI-arms round). The lane's five recorded deviations are
re-censused against the 1.4.5.8 decompile and the REAL gaps ported.
Code-only round — every witness below is WRITTEN NOT RUN (no compiler on
this lane); each expected value is hand-derived from the cited C# lines.

Commits (this phase):

| item | commit | scope |
| --- | --- | --- |
| (1) fountain jet-speed math | `9a293e21` | `ai_fountain` + `fountain_jet` + witness |
| (2) Bubble pop sound + spawn source | `42955200` | `bubble_pop_kill` + death-hook wiring + the BubbleGun volley arm + 2 witnesses |
| (3)+(4) Scutlix spawn ai-pin + crosshair ability calls | `f7687d54` | the 606 row + slice-B ramp/stick + the Mount ability half + the aiStyle-76 wiring + 3 new witnesses + 1 extended |
| (5) fountain tile-338 right-click | `8e455fcb` | `interact_tile`'s 338 arm + witness |

---

## (1) The fountain jet-speed computation — PORTED (was folded with the dust layer)

The DONE round's `ai_fountain` kept every `Main.rand` draw but folded the
jet-speed math out with the dust bodies ("no slot state, so the
computation folds out"). The sweep's ruling: the velocity calc is
GAMEPLAY math, not a dust body — port it now; the dust slots themselves
(position/scale/colour writes) stay g-lanes.

C# (`Projectile.cs:30490-30495`):

```csharp
float num569 = 2f;
float num570 = (float)this.timeLeft / 60f;
if (num570 < 1f) { num569 *= num570; }
```

The per-dust jet vector (`:30500` yellow / `:30520` red / `:30546` blue /
`:30589` rainbow):

```csharp
Vector2 vector55 = (new Vector2(0f, 0f - num569)
    * (0.85f + (float)Main.rand.NextDouble() * 0.2f))
    .RotatedBy((Main.rand.NextDouble() - 0.5) * 1.5707963705062866);
```

(the blue's literal is `0.7853981852531433` = π/4). The blue burst's
shared scale (`:30563`) + the deterministic spoke fan (`:30566`):
`new Vector2((float)(num579 - 4) / 5f, (0f - num569) * num578)`.

Port: `ai_fountain` now computes `jet_speed` (the f32 division + the
`< 1.0` compare, verbatim) and every jet vector through the new
`fountain_jet(jet_speed, half_turn, rng)` helper — TWO draws in the C#'s
order (the speed scale, then the rotation), the spin via the existing
`bow_launch::rotated_by` (`Utils.cs:1675-1684`'s float-cos/sin). The
half-turn literals are the C# float values widened to f64
(1.5707963705062866 / 0.7853981852531433). The draw census is UNCHANGED
(the draws were already kept) — the witnesses pin both the count and the
computed vector (CensusRng's 0.0 doubles → scale 0.85, angle −half_turn/2).

Witness: `w4_d8_the_fountain_jet_speed_math` (the spun X/Y at full speed,
the blue's halved angle, the timeLeft-30 life scale, the timeLeft-60
boundary — `num570 == 1.0` is NOT `< 1`, no scale).

## (2) The Bubble pop sound + spawn source — PORTED; the 405-vs-410 census

**Census — the two bubbles are DISTINCT kinds with TWIN Kill arms:**

- `ProjectileID.FlaironBubble = 405` (`ProjectileID.cs:1364`) — the
  Flairon's bubble (d7's 69-Flairon lane spawns it, `flail_spawn`).
  Kill arm `Projectile.cs:72849-72873`: `SoundID.Item54` at
  `base.position`, then **20** dust-212 on the fixed 10-px ring box.
- `ProjectileID.Bubble = 410` (`ProjectileID.cs:1374`) — the **Bubble
  Gun's** bubble (item 2623, `Item.cs:25686` `shoot = 410`). Kill arm
  `:72919-72943`: the SAME `SoundID.Item54`, then **10** dust-212 on the
  `(int)(10f * ai[1])` ring box (`:72925`).
- There is NO "BubbleMachine mount" — the 410 spawner is the Bubble Gun
  ITEM, not a mount. The mount crosshairs are 441/453 (item (4)).

Both arms' `if (num < 30)` velocity branch is ALWAYS taken (the loops cap
at 20/10), so the `Main.rand.Next(45, 91)` else (`:72866`/`:72936`) is
DEAD CODE — zero range draws. The only explicit draw per dust is the
hslToRgb hue `NextDouble` (`:72868`/`:72938`). `SoundID.Item54` =
`LegacySoundStyle(2, 54)` (`SoundID.cs:1437`), already in the port's
`soundid.rs:206` / `id_tables.rs:412`.

**Spawn source census:** the Bubble Gun's volley
(`Player.cs:48393-48401`, inside `ItemCheck_Shoot` `:47056`) fires THREE
bubbles per use, each at the shared launch point with the aim plus
`Next(-40, 41) * 0.1f` per axis (no renormalize), source
`projectileSource_Item_WithPotentialAmmo` (`:47152` →
`GetProjectileSource_Item_WithPotentialAmmo` `:9805-9808` =
`EntitySource_ItemUse_WithAmmo`). The port's spawn seam carries no source
context (the i22 census convention) — the arm is ported, the source is
the recorded fold.

Port: `bubble_pop_kill(p, rng)` (the kind-gated cue + hue census, the
`wants_sound` intent pattern) wired into the pool's death hook in C#
body order (the 405/410 arms sit ABOVE the 444 Xenopopper arm `:73398`;
the kinds are disjoint). The BubbleGun 2623 arm lands in
`fire_non_ammo_shot`'s per-item chain (the Bat Scepter 1801 precedent —
the same 3-shot spread shape, ×0.1 instead of ×0.05).

Witnesses: `w4_d8_the_bubble_pop_cue_and_hue_census` (the (2,54) cue at
the top-left, 10 vs 20 doubles, zero ranges, the non-bubble no-op) +
`test_dw_5_1_the_bubble_gun_fires_the_three_bubble_volley` (3× kind 410,
damage 70, the ±4 spread window).

## (3) The 434 Scutlix spawn ai-pin — CENSUSED: 434 has NO spawner; the mount fires 606

**The census verdict (the lane's biggest finding):** a full-decompile
search for `NewProjectile` calls with type 434 finds NONE. The
`ScutlixLaserFriendly` id (434) is live in the AI/damage/movement tails
(`:5047` SetDefaults, `:10927` the ai-pin, `:12448` the damage refusal,
`:13975`/`:17079`/`:17991` the numUpdates exits, `Main.cs:34603` the
beam draw) but NOTHING spawns it in 1.4.5.8. The Scutlix mount's ACTUAL
laser is **606 `ScutlixLaser`** (`ProjectileID.cs:1766`):

`Mount.UseAbility` case 9 (`Mount.cs:3258-3289`): TWO beams from the
head's eyes toward the `ClampToDeadZone`-clamped cursor, each
`NewProjectile(GetProjectileSpawnSource(mountedPlayer), eye.X, eye.Y,
aim.X, aim.Y, 606, 150, 0f, Main.myPlayer)` — the aim is the normalized
eye→cursor delta × 14, the spawn 14 px along it. The eye anchor:
`(position.X + width/2, position.Y + height)` + the
`scutlixEyePositions[(frameExtra − 6) * 2 + i]` offset (X mirrored by the
facing ± `xOffset`, Y + `yOffset`). The spawn source is
`EntitySource_Mount` (`Mount.cs:2898-2901`).

The 434 row + aiStyle-74 arm STAY (the DONE round's census is correct
for the id's live tails) — but the mount's beam is 606, and the sweep
ports it:

- **The 606 row** (`Projectile.cs:6530-6541`): 5×5, aiStyle 1, friendly,
  alpha 255, extraUpdates 2, scale 1, timeLeft 600, ignoreWater.
- **The AI_001 ramp** (`:54772-54828`, the Martian-laser group
  440/449/606): the 606 joins slice B's existing 449 arm — the fade 25,
  the `localAI[0]` ramp at `num54 = 150` / `num55 = 5` (`:54796-54800`),
  the one-shot `localAI[0] == 5` four-dust-182 burst (`:54804-54813`,
  bodies skip, zero explicit draws), the `ai[1] != 0` countdown with the
  ≤ 0 kill. The 606 is on the `flag3` resting-counter list (`:56014`).
- **The blocked-move stick** (`HandleMovement` `:18278-18286`): on the
  first blocked move the beam latches `ai[1] = 1`, takes ONE last
  full-velocity step and restores the pre-collision velocity; the
  `:17590` gate then skips TileCollision and `UpdatePosition`'s `:20435`
  skip holds the position — the beam sticks in the tile and the ramp
  counts down in place (the moonlord-trail precedent). Ported in slice
  B's `finish_flight` for BOTH 449 and 606 (the 449's stick was the same
  C# arm, previously folded to the kill-on-tile default — the sweep
  fixes both).
- **The fire** (`MountState::scutlix_laser_fire`): the self-contained
  two-beam calc returns the `(x, y, vx, vy)` pairs; the crosshair's
  channel-release parks them on the slot's `wants_scutlix_beams` intent
  and the dispatch spawns them through `spawn_with_data_ai_owned`
  (damage 150, knockback 0, the 5×5 half-box recentre).

**Self-containment verdict (the lane's question):** the Scutlix laser
fire IS self-contained given the player's position/direction/width/
height + the mount's `frameExtra` + the row's x/yOffset — it needs NO
mount frame-state machine beyond the `frameExtra` the aim already
writes. The `AimAbility` head aim (the facing fold + the frame buckets)
is likewise pure state. Both are ported. The Drill (8) mining payload
(`DrillMountData`, `UpdateDrill`/`UseDrill` `Mount.cs:3036-3180`) and the
Santank (46) ability are NOT self-contained (the beam cooldowns, the
smart-cursor tile scans, the `PickTile` writes) and stay M2-MOUNT-4.

Witnesses: `the_scutlix_beam_ramps_five_to_150_and_counts_down` +
`the_scutlix_beam_sticks_in_the_tile_and_holds` (slice B) +
`w4_d8_the_scutlix_ability_math` (mount.rs: the eye table, the clamp, the
aim buckets, the two-beam fire, the charge-stop).

## (4) The 76 Crosshairs mount-ability calls — PORTED (the Scutlix half); NOT a Sniper scope

**Census:** the aiStyle-76 crosshairs are the MOUNT crosshairs (441
Scutlix / 453 Drill), NOT a Sniper scope. The C# arm
(`Projectile.cs:30621-30670`) calls:

- local owner: `mount.AimAbility(player6, position)` (`:30648`), and on
  the channel release `mount.UseAbility(player6, position, toggleOn:
  false)` + `Kill()` (`:30649-30653`);
- remote owner: the 441 `StartAbilityCharge` when not charging
  (`:30658-30664`), the 453 `UseAbility(toggleOn: false)` when not active
  (`:30665-30668`), then `AimAbility` (`:30669`).

The DONE round folded ALL of these out ("no ability layer"). The sweep
ports the self-contained Scutlix half (item (3)'s machinery):

- `AimAbility` → `MountState::aim_ability_scutlix` (`Mount.cs:5340-5421`):
  the `_aiming = true` head, the dead-zone-clamped cursor angle in
  degrees, the ±90° facing fold (the write rides the player), the
  flip-draw flag (facing against the velocity), the `_frameExtra`
  buckets (8 level / 9,10 up / 7,6 down at the 22.5°/67.5°/112.5° gates),
  the change verdict. The eye-glow light (`:5394-5414`) is the light
  layer's — skipped. The Drill (8) half writes the unported
  `DrillMountData.diodeRotation` (`:5466-5494`) — folds out.
- `UseAbility` (the 441 release) → the two-beam fire (item (3)).
- `StartAbilityCharge` (the remote 441) → the `_abilityCharging` latch
  (`Mount.cs:2916-2919` — the remote branch only sets the flag; the
  local-owner 441 spawn is the auto-aim's `UpdateEffects` path,
  `Mount.cs:4879-4947`, M2-MOUNT-4).
- the 453's `UseAbility(toggleOn: false)` → the `_abilityActive` clear
  (`Mount.cs:3317-3335`) — the flag the Drill mining layer reads.
- **The Kill arms** (`Projectile.cs:73390-73397`, ABOVE the 444 arm):
  the remote 453's death clears the Drill flag; EVERY 441 death calls
  `StopAbilityCharge` (`Mount.cs:2922-2931`: the charge flag drops, the
  row's cooldown 20 / duration 0 re-arm). Both wired into the death hook.

Witness: the extended `w4_d8_the_crosshair_pins_aims_and_releases` (the
release fires 2 beams at speed 14, the aim writes the facing + frame, the
remote 441 sets the charge flag).

## (5) The fountain TILE 338 use/wiring — the proj-spawn side PORTED

**Census:** tile 338 = `TileID.FireworkFountain` (the 2×2 firework
fountain). Two activation paths:

- **Right-click** (`Player.TileInteractionsUse` `:32118`, the 338 arm
  `:32355-32377`): the anchor shifts UP one when the clicked cell's
  `frameY == 18` (the bottom row), the pool dedupe (an active aiStyle-73
  shot with the same `ai[0]/ai[1]` anchor skips the spawn), then ONE
  fountain `419 + Main.rand.Next(4)` at `(myX*16 + 8, num25*16 + 2)`,
  zero velocity, Damage 0, ai0/ai1 = the anchor, source
  `GetProjectileSource_TileInteraction`. The
  `originatedFromActivableTile = true` write (`:32375`) has NO reader in
  the decompile (the field `Projectile.cs:450` is written at 5 sites —
  `WorldGen.cs:50700`/`:62175`/`:62193`, `Player.cs:32169`/`:32375` —
  and read nowhere) — the port drops it (the census note).
- **Wiring** (`Wiring.cs:1522-1546`): the same dedupe + `419 + Next(4)`
  spawn, already ported in tr-core's `firework_fountain`
  (`wiring.rs:1660`) + the world_sim dedupe (`world_sim/wiring.rs:430`,
  decision D1 — the roll happens before the dedupe, the recorded
  deviation).

The lane owns the PROJ-SPAWN side: the right-click arm was missing
(`interact_tile` had the golf-tee 494 + the altar 237, no 338). Ported
after the golf-tee arm (the C# body order: 494 `:32342` < 338 `:32355`),
before the altar. The `SwitchFountain` tile 207 (the WATER fountain's
biome switch, `WorldGen.cs:51150-51193`) is e20's lane — untouched.

Witness: `w4_d8_the_fountain_right_click_spawns_and_dedupes` (the
bottom-row anchor shift, the 419-422 kind roll, ai0/ai1 = the shifted
anchor, damage 0, timeLeft 600, the dedupe on both rows, the dead-anchor
re-spawn).

---

## RNG contract (unchanged by the sweep)

Every port keeps the DONE round's `UpdateProjectiles`-stream contract:
the explicit `Main.rand` draws stay in the C#'s order, the dust BODIES
skip (g-lanes), the `Dust.NewDust*` internals go with the dust pool. The
sweep ADDS no draws — item (1) computes the vectors the kept draws
already fed; item (2)'s pop keeps the per-dust hue `NextDouble` (the
`Next(45,91)` else is dead); item (3)'s fire/aim/ramp are deterministic
(zero draws); item (5)'s `Next(4)` rides the `UpdatePlayers` stream (the
`TileInteractionsUse` ride, `Main.cs:17754`).

## Deliberate folds (recorded)

- The spawn-source contexts (`EntitySource_ItemUse_WithAmmo`,
  `EntitySource_Mount`, `EntitySource_TileInteraction`) — the port's
  spawn seam carries no source context (the i22 census convention).
- `originatedFromActivableTile` — no C# reader (item (5)).
- The Drill (8) mining payload + the Santank (46) ability + the
  local-owner auto-aim `UpdateEffects` path — M2-MOUNT-4 (item (3)/(4)).
- The eye-glow / beam lights (`Lighting.AddLight`) — the light layer.
- The 434 row's `NewProjectile` ai-pin (`:10927-10931`) — no spawner
  exists; the pin stays unported until a 434 spawner is found (the DONE
  round's note stands, now census-confirmed).