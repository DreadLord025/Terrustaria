# 26d — W4-d10 DOC-SWEEP: the 94/97/98 spawn-source censuses

> Branch `task/W4-d10-proj-ai-91-100`, 2026-11-21. Code-only round. These
> are the item-source / use-arm / spawn-source verdicts for the three
> aiStyle ports whose producers live outside the projectile module (the
> Coin Portal 518, the Flying Piggy Bank 525, the Cultist Boss Particle
> 526). Each census reads the C# producer + the port's wiring; no behaviour
> changes beyond the stale-comment fix noted below.

## 94 — Coin Portal 518: the source is the POT break, NOT the Coin Gun

**Verdict: the 518 spawns from `WorldGen.SpawnThingsFromPot`'s luck-0 arm
(`WorldGen.cs:57176-57182`). The Coin Gun 905 is a DIFFERENT weapon that
fires the coin projectiles (158), never the portal.**

- The C# producer: `SpawnThingsFromPot` (`WorldGen.cs:57089-57811`, called
  from `CheckPot` `:56797` on a pot break). At `:57176`
  `Player.GetClosestRollLuck(i, j, range) == 0f` (the luck roll, range
  `500 / ((num+1)/2)` from the pot-style multiplier) → `:57180`
  `Projectile.NewProjectile(GetProjectileSource_TileBreak, i*16+16,
  j*16+16, 0f, -12f, 518, 0, 0f, Main.myPlayer)` (gated `Main.netMode != 1`).
  The portal rises at −12 velocity from the broken pot's cell.
- **The Coin Gun 905** (`Item.cs:11824-11841`): `useStyle = 5`,
  `shoot = 158`, `useAmmo = AmmoID.Coin`, `shootSpeed = 10`, `damage = 0`,
  `ranged = true`. It fires the coin projectile **158** (the copper coin
  shot), consuming coin ammo — NOT the portal 518. The dispatch's guess
  ("coin-gun 905?") is **FALSE**: the portal has no weapon source; it is a
  pot-loot Easter egg (the luck-0 roll replaces the pot's normal loot).
- The port: `world_sim/editing.rs::spawn_things_from_pot` (`:6392`) ports
  the full roll order; the luck-0 arm (`:6413`, `main_rng.next_range(0,
  range) == 0`) returns the portal flag, the caller pushes `(x, y)` and the
  break tail spawns `spawn_with_data(518, px*16+16, py*16+16, 0, -12, 0, 0)`
  (`:5720-5730`). The 518 row + aiStyle-94 arm landed W4-d10 (`5c229707`):
  the spawn resolves now. **Stale comment fixed** (`:5717-5719` said "the
  port has no 518 family yet — spawn_with_data declines it"; the family
  exists since `5c229707`). The `netMode != 1` gate folds to the port's
  single-player/host (no client pot-break path).

## 97 — Flying Piggy Bank 525: the source is the Money Trough 3213, NOT 4076

**Verdict: the 525 spawns from the Money Trough (item 3213, `shoot = 525`).
Item 4076 is the Void Vault (a placeable tile), NOT the piggy source. The
use-arm is the generic `ItemCheck_Shoot` → `fire_non_ammo_shot`.**

- The C# source: `Item.cs:28978-28989` case **3213** (the Money Trough,
  `ItemID.cs:8109`): `useStyle = 1`, `shootSpeed = 4f`, `shoot = 525`,
  `useAnimation = 28`, `useTime = 28`, `UseSound = Item59`. The ONLY
  `shoot = 525` site in the decompile (`Item.cs:28981`).
- **Item 4076** (`Item.cs:35741-35754`, the Void Vault `ItemID.cs:9835`):
  `consumable = true`, `createTile = 491`, `useStyle = 1`, `useTurn = true`
  — a placeable storage tile, NO `shoot`. The dispatch's guess ("item
  4076?") is **FALSE**. (The Void Bag 4077 summons the piggy via a different
  path; the 525's direct `shoot` source is the Money Trough 3213 alone.)
- The use-arm: the Money Trough is `useStyle = 1` with no ammo → the generic
  `Player.ItemCheck_Shoot` (`Player.cs:47056+`) → the port's
  `fire_non_ammo_shot` (`world_sim/editing.rs:3015`, the S1-P-i20 shared
  seam): the aim normalized to `shootSpeed` (4), the player-centre origin
  (+ dir*10), `weaponDamage` (the class multiplier — the trough is
  `DamageClass::None`, damage −1 → the shot deals no damage), the item
  knockback (0). The 525 row + aiStyle-97 arm landed W4-d10 (`14b14488`):
  the throw 0.94 decay + the rest latch + the owner-facing bob. The piggy's
  container UI (`TryGetContainerIndex` `Projectile.cs:78815`) + the
  `HadAnActiveInteractableProjectile` flag (`:33070`) are the recorded
  no-consumer skips (the module doc).
- The port registry: `itemdata.rs:3379` Money Trough `projectile: Some(525)`,
  `shoot_speed: 4.0`, `use_style: 1` — verbatim. The generic fire path reads
  `registry::projectile(held.kind)` (`editing.rs:2737`) → 525 → the spawn.

## 98 — Cultist Boss Particle 526: the source is the CultistTablet 437 (aiStyle 83), NOT the boss AI

**Verdict: the 526 spawns from the CultistTablet (NPC 437, aiStyle 83)'s
ritual despawn — NOT the Lunatic Cultist boss (439, aiStyle 84) that c1
ports. c1's `cultist_ai_step` does NOT cover the 526 source. The tablet
arm is CLAIMED (W4-d31) — the remaining work is the ACTIVATION CONTRACT
below, not a re-port.**

**Claim census (2026-11-21 re-read):**
- `task/W4-d31-npc-ai-styles-75-83` (`a31ff86e`, arm 2) ported the whole
  aiStyle-83 machine as **unwired** `npc_ai_style83.rs`
  (`style83_step`, `NPC.cs:36757-36925` verbatim): the 437 tablet's
  `localAI[3]` latch + `CultistRitual.CheckFloor`, the 379/438 ritual-four
  spawns, the struck-devotee release, the all-gone boss summon (ONE
  `Next(2)` side roll), the 300-tick dissolve — **including all three 526
  bursts** (`ring_particle` ×6 at the death + the 20 gated launches +
  the `%10==1 && >120` trickle). The particle RNG (speed `NextFloat`,
  the `RotatedByRandom(TAU)` pair, the launch's radius `NextFloat`, the
  `Next(2)` gate) is consumed IN-STEP on the arm's `UnifiedRandom` — the
  draws belong to the tablet lane, exactly as this census predicted.
  The 526 requests ride `Style83Out::shots` as `HostileShotAi`
  (`kind: 526, damage: 0, ai0/ai1 = the target point`).
- `task/W4-d30-npc-ai-styles-65-72` and `task/W4-d32-npc-ai-styles-85-93`
  do NOT touch 83 (d30 ends at 72; d32 starts at 85) — no overlap.
- `task/W4-d40-npc-kinds-421-490` (`7fec14aa`, batch 2) landed the
  **437/438 SetDefaults rows** (tablet 54×54 aiStyle 83
  dontTakeDamage+noGravity+noTileCollide, devotee 22×40 aiStyle 83
  lavaImmune) — the kinds exist, the machine stays UNWIRED (the style73
  precedent: "tablet/devotee UNWIRED").

**The activation contract (what the wiring wave owes, in order):**
1. **Dispatch:** route NPC kinds 437/438 to `style83_step` on the
   `UpdateNPCs` stream (the style75 wiring precedent,
   `world_sim/npcs.rs:4465-4488`); resolve `Style83Out::spawns`/`links`
   (the carrier slot+1 convention), `retargets`, `died`,
   `tablet_destroyed` per the module's wiring-seam doc.
2. **The 526 drain — the owner re-pin:** the C# call sites
   (`NPC.cs:36863`/`:36874`/`:36885`) pass **`Main.myPlayer`** as the
   owner — NOT the stock hostile default. The sim's generic
   `wants_hostile_shot_ai` drain (`world_sim/npcs.rs:5947-5956`,
   `spawn_hostile_ai`) hardcodes owner 255; for the 526 the wiring must
   spawn with **owner 0** (`spawn_with_data_ai_owned(526, 0, …)` — the
   fishron `spawn_hostile_ai_owned` precedent) so the local-owner
   simulation gate + `NewProjectile`'s `TryUpdateChannel` tail
   (`Projectile.cs:11097-11100`, owner == myPlayer) match. In
   single-player the 255 + `hostile_local` path simulates identically
   (the composition witness pins this), but the channel-track tail and
   the owner semantics differ — the re-pin is the contract.
3. **The row + arm are d10's, already live:** `proj_data(526)`
   (`data.rs`, `Projectile.cs:5962-5970`: 8×8, aiStyle 98, tileCollide
   false, timeLeft 120, alpha 255, stock penetrate 1, friendly/hostile
   both false — the visual marker) + `ai_cultist_particle`
   (`8814cb4e`): the `(ai0, ai1)` target seek, the dist<speed arrival
   kill BEFORE the trail, the 15-speed 0.1-lerp, the 2×`NextFloat` dust
   scales on the `UpdateProjectiles` stream. The payload d31 packs is
   exactly what this arm reads — composition verified (the witness
   `w4_d10_s98_the_tablet_payload_composes`).
4. **No stream conflict:** the tablet's particle draws ride
   `UpdateNPCs` in d31's arm; the particle's own flight draws ride
   `UpdateProjectiles` in d10's arm; c1's boss-84 stream is untouched.
   The 526 deals no damage (damage 0 + the row's hostile false), so the
   PVE/PVP walks no-op — the marker is inert exactly as the C#.

- The C# producer: `NPC.cs`'s `AI()` aiStyle-83 block, `this.type == 437`
  (the CultistTablet, `NPCID.cs:11549`), the `ai[0] == -1` despawn countdown
  (`:36847-36887`):
  - `:36857-36864` — at `ai[3] > 300` the tablet dies; a 6-particle burst
    spawns 526 at `Center + UnitY.RotatedByRandom(2π)*30`, velocity
    `dir * (3 + NextFloat()*6)`, `ai0/ai1 = Main.npc[ai[2]].Center` (the
    summoned boss 439's centre, `:36840-36842` spawns it and packs `ai[2]`).
  - `:36865-36876` — a 20-iteration `Next(2) != 0` gate: a second 526 burst
    at `Center + dir*30`, velocity `(dir.X * speed, -20)`, the target
    `Center + dir*(NextFloat()*45+45) + UnitY*20`.
  - `:36879-36886` — the steady drip: `ai[3] % 10 == 1 && ai[3] > 120` → one
    526 at `Center + dir*25`, velocity `dir * (3 + NextFloat()*6)`, target
    the boss centre.
  The 526's `(ai0, ai1)` target-point seek (the d10 98-arm, `8814cb4e`)
  reads exactly these packed targets.
- **c1's claim is the BOSS, not the tablet:** `task/W4-c1-cultist-ai-wire`
  (`f61243c0`) wires aiStyle **84** to `npc_ai_cultist::cultist_ai_step`
  (the 439 boss, `NPC.cs:64414-65388`, states 4/7/8 deferred). The 526
  source is aiStyle **83** / type **437** — a DIFFERENT NPC arm c1 does not
  touch. (Historical note, superseded by the claim census above: at this
  doc's first writing the port had no aiStyle-83 arm; W4-d31 has since
  ported it unwired and W4-d40 rowed the kinds — the 98-arm's producer
  now EXISTS on the d31/d40 branches, pending the wiring wave.)
- **Cross-note:** the 526 particle's RNG (the `RotatedByRandom(2π)` pair +
  the `NextFloat()` speed per particle, the `Next(2)` gate) belongs to the
  tablet's aiStyle-83 arm, NOT the boss's `UpdateNPCs` stream position c1
  manages. When the tablet lane lands it owns those draws; the d10 98-arm
  itself draws only the 2-dust trail's `NextFloat` per dust (`:33150`, the
  particle's own flight). No stream conflict with c1.

## Witnesses

Census verdicts, not new behaviour — the stale 518 comment fix
(`editing.rs:5717`) was the first round's only code change; the existing
pot-loot witnesses cover the portal flag + the spawn. The §98 activation
contract adds ONE composition witness (WRITTEN NOT RUN, code-only):
`w4_d10_s98_the_tablet_payload_composes`
(`projectile_ai_91_100.rs`) — a d31-shaped `HostileShotAi` payload
spawned through the pool's seam flies the d10 seek and arrives at the
packed target, both drain owner shapes (the stock 255 + hostile_local
and the contract's owner-0 re-pin) simulate identically in
single-player.