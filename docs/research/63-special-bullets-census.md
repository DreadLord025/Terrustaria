# 63 — Special-bullet effects census & port (S1-P-i18, code-only)

Audit base: `aa147ce0` (origin/main). Worktree `TR-i18`, branch
`task/S1-P-i18-special-bullets`. Worker W3-i18b (retry of the silent
original). **Code-only mode: no cargo/rustc/rustfmt/tests/dotnet/graphify
invoked** — every claim is file-read evidence (C# `decompiled/Terraria/
Projectile.cs` / `Player.cs` / `ID/*.cs` vs the Rust tree). Reads kept to
≤200-line windows. Tests WRITTEN NOT RUN.

Scope: the **special bullet effects** — the `Player.ItemCheck_Shoot`
bullet-type branches and their projectile-side behaviour (Meteor bounce,
Crystal shard split, Cursed flame, Ichor debuff, Chlorophyte homing,
HighVelocity, Exploding, Party, Golden, Nano, Venom, Luminite pierce).
Bullet specials ONLY; the arrow specials are i17, the ammo pick/consume
is i15, the generic gun/bullet wire is i14.

Status legend (swarm vocabulary): **FAITHFUL** = 1:1 port present ·
**WRONG** = ported but behaviour differs · **MISSING** = no port
counterpart · **UNWIRED** = bound but no handler.

> **Coordination (load-bearing).** The Lead left `LEAD-I15-COORD.md`:
> **i15 (`task/S1-P-i15-ammo-consumption`, `6df7e170`) ALREADY fixed the
> i03 census A1 Nanite ×1.25 double-scale** — the resolver returns the
> unscaled `num`, the wire carries the single fused `(int)(Damage*1.25f)`
> after the add (`Player.cs:53552-53559`). That fix lives on i15's branch,
> NOT in this base (`aa147ce0` still has the double-scale at
> `ammo.rs:680-685` + `editing.rs:1892-1897`). **This task does NOT re-fix
> the Nanite scaling** — it is i15's, and re-fixing it here would conflict
> at merge. The Lead also pre-staged the twelve `SetDefaults` rows +
> constants + the `ProjLight` colour arms + the pool-side death hooks
> (`special_bullet_kill` / `venom_burst_spread` / `exploding_bullet_burst`)
> + the high-velocity gun override (`ammo.rs`) + the `npc_buffs` witness
> helpers; this worker ports the `projectile_special_bullets` module (the
> per-type AI arms + the on-hit / spread debuff arms) to complete the
> wiring. The two halves are kind-disjoint and compile together.

---

## 0. Verdict table (the eleven specials + Luminite)

The C# bullet specials are projectile kinds, not `ItemCheck_Shoot`
item-type branches: `PickAmmo` resolves the ammo's `Item.shoot`
(`Player.cs:53491-53494`) to the projectile id, and the per-type behaviour
lives in `Projectile.SetDefaults` / `AI_001` / `HandleMovement` /
`StatusNPC` / `Kill()`. The only `ItemCheck_Shoot` bullet-type branch is
the HighVelocity gun override (`Player.cs:47107-47118`).

| Kind | Item | Effect | C# cite | Verdict at base | Now |
| --- | --- | --- | --- | --- | --- |
| 36 MeteorShot | 234 Meteor Shot | one-bounce reflect, penetrate 2 | `Projectile.cs:1182-1197`, `:19278-19298` | **MISSING** (no row) | ported |
| 89 CrystalBullet | 515 Crystal Bullet | death split → 2× 90 shard | `:1715-1728`, `:75250-75268` | **MISSING** | ported |
| 90 CrystalShard | (spawn) | aiStyle 24 drag/fade | `:1729-1742`, `:26004-26013` | **MISSING** (no row) | row + existing `ai_crystal_storm` |
| 104 CursedBullet | 546 Cursed Bullet | CursedInferno 39/420 | `:1915-1928`, `:11665-11668` | **MISSING** | ported |
| 207 ChlorophyteBullet | 1179 Chlorophyte Bullet | homing (Manhattan 300 acquire / 1000 hold, blend /8) | `:2948-2961`, `:56778-56887` | **MISSING** | ported |
| 242 BulletHighVelocity | 1302 High Velocity Bullet | penetrate 3, extraUpdates 7, speed fade; Sniper/Venus/Uzi convert 14→242 | `:3238-3254`, `:55660-55671`, `Player.cs:47107-47118` | **MISSING** | ported |
| 279 IchorBullet | 1335 Ichor Bullet | Ichor 69/600 | `:3650-3663`, `:11669-11672` | **MISSING** | ported |
| 283 VenomBullet | 1342 Venom Bullet | Venom 70/600 + PotentAcid 395/120; 3-box debuff-spread death burst | `:3699-3712`, `:11673-11677`, `:75911-75974`, `:12810-12814` | **MISSING** | ported |
| 284 PartyBullet | 1349 Party Bullet | confetti death (visual only) | `:3713-3726`, `:75975-75999` | **MISSING** | row + documented visual skip |
| 285 NanoBullet | 1350 Nano Bullet | Confused 31 (Next(3)?180:60); one-bounce ×0.66 + re-target; PickAmmo ×1.25 | `:3727-3740`, `:11541-11551`, `:18563-18591`, `Player.cs:53556-53559` | **MISSING** (effects); ×1.25 **WRONG** (i15's) | effects ported; scaling left to i15 |
| 286 ExplosiveBullet | 1351 Exploding Bullet | idStatic-60 AoE death burst, appliesImmunityTimeOnSingleHits | `:3741-3757`, `:76000-76034` | **MISSING** | ported |
| 287 GoldenBullet | 1352 Golden Bullet | Midas 72/120 | `:3758-3771`, `:11537-11540` | **MISSING** | ported |
| 638 MoonlordBullet | 3567 Luminite Bullet | infinite pierce, ×0.96/hit decay | `:6858-6872`, `:14177-14180`, `:55864-55899` | **FAITHFUL** (S1-D-4) | unchanged |

**The structural finding:** at `aa147ce0` NONE of 36/89/90/104/207/242/
279/283/284/285/286/287 had a `proj_data` row. `spawn_with_data_ai_owned`
does `proj_data(kind)?` (`projectile.rs:13102`) and returns `None`, so the
shoot wire's `spawn_with_data_owned` (`editing.rs:1927`) silently declined
every special bullet — **the shots never spawned at all**. So the specials
were MISSING (not WRONG): there was no behaviour to diverge. Only 14
(musket ball, `data.rs:1521`) and 638 (Luminite, `data.rs:2596`, S1-D-4's
`projectile_ai_moonlord`) had rows. The fix is therefore additive: the
twelve rows + the per-type arms + the debuff/death hooks.

---

## 1. The C# shape (reference)

`ItemCheck_Shoot` (`Player.cs:47056-47082`) seeds `projToShoot =
sItem.shoot` and calls `PickAmmo` (`:47071`), which resolves the ammo's
`Item.shoot` (`:53491-53494`). The ONLY bullet-type branch in
`ItemCheck_Shoot` itself is the HighVelocity conversion: the Sniper Rifle
1254 / Venus Magnum 1255 / Uzi 1265 turn the musket-ball default 14 into
242 (`:47107-47118`), AFTER `PickAmmo` returns and only when the resolved
projectile is still 14 (a special bullet's own `shoot` already replaced
it). The Nanite ×1.25 (`:53556-53559`) is in `PickAmmo`, on the FUSED
damage — i15's fix, not this task's.

Every special bullet is `aiStyle 1` (the arrow/bullet family, `AI_001`
`Projectile.cs:54002-57798`) except the 90 shard (`aiStyle 24`). The
per-type behaviour:

- **SetDefaults** (`:1182-1197` / `:1715-1742` / `:1915-1928` /
  `:2948-2961` / `:3238-3254` / `:3650-3771`): the 4×4 box (90 is 6×6),
  `friendly`, `ranged`, `alpha 255` (90: 50), `timeLeft 600`, the per-kind
  penetrate / extraUpdates / scale / light / immunity pair.
- **AI_001 hooks**: the 242 speed fade (`:55660-55671`), the 207 homing
  block (`:56778-56887`). The `flag3` counter switch (`:55947-56032`)
  rests every special EXCEPT 207 (which counts, but its homing block
  consumes the gravity else-if chain so the generic `ai[0]>=15` arm never
  fires). No special has a per-type rotation arm — all take the default
  `Atan2(vy,vx)+1.57f` (`:57748-57751`).
- **HandleMovement**: the 36 one-bounce reflect (`:19278-19298`), the 285
  bounce + re-target (`:18563-18591`); the rest kill on tile contact
  (the default arm `:19849-19853`).
- **StatusNPC** (`:11173-11726`, called `:13580` BEFORE `StrikeNPC`
  `:13649`): the 287 Midas (`:11537-11540`), the 285 Confused roll
  (`:11541-11551`), the 104/279/283 else-if chain (`:11665-11677`).
- **Kill()** (`:70642-77263`, the death-effect chain's outer gate is
  `!Main.projPet[type]` `:74815` — NOT a tile-death gate, so EVERY death
  path fires them): the 89 split (`:75250-75268`), the 283 debuff-spread
  burst (`:75911-75974`), the 284 confetti (`:75975-75999`), the 286 AoE
  burst (`:76000-76034`), the common HitTiles+Item10 arm
  (`:76035-76039`). The 283 burst's `Damage()` re-hits through the
  `type == 283 && !active` early arm (`:12810-12814` → StatusNPC, no
  strike).
- **ProjLight** (`:16153-16661`): the per-kind colour arms
  (`:16294-16299` 104, `:16338-16342` 242, `:16391-16396` 36,
  `:16470-16498` 284/285/286/287/283, `:16535-16540` 89/90,
  `:16591-16595` 207, `:16642-16647` 279).

---

## 2. The port (what landed)

**Lead-staged (data + pool wiring):**
- `projectile/data.rs`: the 12 constants + `SPECIAL_BULLET_KINDS` + the
  12 `proj_data` rows (verbatim SetDefaults slices, cited per row) + the
  `set_defaults_light` amounts + the 242 `ignore_water` overlay.
- `proj_lights.rs`: the per-kind `ProjLight` colour arms (the 104 joins
  the 95/96 cursed-flame arm, the 242 joins the 14/110/180 musket arm,
  the 36/284/285/286/287/283/89/90/207/279 new arms).
- `projectile.rs`: the aiStyle-1 dispatch gate (`covers`, chained after
  A/B/C/frostmoon/moonlord), `special_bullet_status_npc` (the on-hit
  debuff call at `:13580`'s position, before the strike, in both
  `damage_pve` and `damage_pve_boxed`), the 286
  `appliesImmunityTimeOnSingleHits` kind-check in both idStatic gates,
  `special_bullet_kill` (the death hook: the 89 split with its 8
  state-bearing draws inline, the 283 `venom_burst_spread` +
  `venom_spread_walk`, the 286 `exploding_bullet_burst`), and the
  `npc_center` / `find_target_with_line_of_sight` visibility bumps.
- `ammo.rs`: the HighVelocity gun override (1254/1255/1265 + proj 14 →
  242, `Player.cs:47107-47118`, folded into the resolver tail).
- `npc_buffs.rs`: `has_buff` / `buff_time_of` witness helpers.
- `main.rs`: the `mod projectile_special_bullets;` declaration.

**This worker (`projectile_special_bullets.rs`):**
- `covers` / `step` — the per-type `AI_001` arms: the 242 fade, the 207
  homing (acquire / hold / steer), the default rotation + exact +16
  clamp, and the `HandleMovement` probe with the 36 bounce and the 285
  bounce + re-target (the shared `find_target_with_line_of_sight`).
- `bullet_on_hit_debuffs` — the StatusNPC per-type arms (287/285/104/
  279/283) with the 285's one-draw Confused roll.
- `venom_burst_debuffs` — the 283 spread's debuff-only re-hit
  (`:12810-12814`).
- 13 witness tests (`s1_p_i18_*`), WRITTEN NOT RUN.

---

## 3. RNG contract (S1-A-10)

Stream: the projectile pass's `UpdateProjectiles` (`Main.cs:17775-17777`),
the pool's `rng` parameter. Draw order:

- **The AI arms draw NOTHING** (no rng parameter): every `Main.rand` in
  these kinds' `AI_001` blocks feeds dust alone (the 207 trail dust 75
  `:56782-56792`, the 36/285 bounce HitTiles/sound) — the ai001_a/b/c
  dust-skip convention. The flight is RNG-free by construction.
- **The 285 on-hit Confused roll**: ONE `Next(3)` per landed hit
  (`:11543`), at the StatusNPC position (`:13580`, before the strike
  `:13649` and its `DamageVar`). 180 ticks on a zero, 60 otherwise.
- **The 89 death split**: 8 ranges — per shard `Next(40,70)` then
  `Next(-20,21)` for X, then the same pair for Y (`:75264-75265`), shard
  0's four before shard 1's (argument evaluation order). STATE-BEARING
  (they place AND velocity the shard), so they ride the pass stream
  inline in `special_bullet_kill`.
- **The death-burst dust/gore bodies AND their draws are SKIPPED** with
  their layers (the ai001 convention — the pool has no dust/gore draw
  layer): the 283's three 10-dust-171 loops (`:75913-75939`), the 284
  confetti (`:75977-75998`, purely visual), the 286's dust+gore
  (`:76003-76018`), the common arm's HitTiles+Item10 (`:76035-76039`).
  The 286 burst's DAMAGE half draws only the per-NPC `DamageVar` the
  boxed walk already spends. This is the Lead's chosen convention for
  this family (NOT the projectile_ai_misc draw-census convention that
  keeps explicit visual draws) — recorded in ANALYSIS.

The witness tests mirror every kept draw on a clone `FastRandom` stream
and pin the raw values (the ai001/golden-stream convention).

---

## 4. Deliberate skips / stand-ins (ANALYSIS "S1-P-i18 special bullets")

- Dust, gore, `Lighting`/`PlotTileLine`, `netUpdate`, the `Kill()` sound
  cues — no such layers in the pool (the ai001 / moonlord convention).
  The 36 bounce's `SoundID.Item10` rides `wants_sound`.
- `appliesImmunityTimeOnSingleHits` (the 286's `:3754`) has no `ProjData`
  field — the pool's idStatic gate reads a kind check
  (`max_penetrate != 1 || kind == EXPLODING_BULLET`).
- The 283's `localAI[1] = -1f` (`:75942`) is a verbatim DEAD write (no C#
  reader) — ported for its shape.
- The 285 PvP Confused arm (`StatusPlayer` `:12288-12298`) — no
  projectile-PVP layer (single-player); recorded, not ported.
- The 207 `Collision.CanHit` is the port's `can_hit` walk; the 285
  re-target uses the shared `find_target_with_line_of_sight`.
- The movement probe kills without the C#'s final `position += velocity`
  on the blocked tick (the port-wide `ai_style1_flight` stand-in); the
  bounce arms integrate the reflected velocity on the bounce tick (the
  typhoon precedent).
- The 283 burst's `oldVelocity.ToRotation()` reads `p.vel` (no
  `oldVelocity` field); the bullet rests its counter and flies straight,
  so `oldVelocity == velocity` at death — faithful for every reachable
  283.
- The 90 shard flies the existing `ai_crystal_storm` arm (its generic
  0.96/15-tick drag/fade IS the 90's C# arm `:26004-26013`); the 1134
  melee shard's first-tick sound/dust block (`:25938-25952`) is kind-gated
  to 1134, so the 90 skips it correctly.

---

## 5. Out of scope (recorded, not touched)

- **The Nanite ×1.25 double-scale (i03 census A1)** — i15's fix
  (`6df7e170`), on its branch. This base still has it
  (`ammo.rs:680-685` + `editing.rs:1892-1897`); left untouched per
  `LEAD-I15-COORD.md` to avoid a merge conflict. The 285 row + effects
  land here; the single fused scale lands with i15.
- The arrow specials (i17), the ammo pick/consume (i15), the generic
  gun/bullet wire (i14), the rocket/dart ammo (i16).
- The 302 SniperBullet / 981 SilverBullet / 110 BulletSnowman siblings
  (other tasks' kinds; 110 is ai001_c's).

## 6. Cross-references

- i03 census `docs/research/62-ammo-potion-census.md` (A1 Nanite, the
  ammo system shape) — on `task/S1-P-i03-ammo-potion-census`.
- i15 `task/S1-P-i15-ammo-consumption` (`6df7e170`) — the A1 fix + the
  ammo pick/consume census.
- S1-D-4 `projectile_ai_moonlord.rs` — the 638 Luminite bullet (FAITHFUL).
- S1-D-5 `projectile_ai001_a/b/c.rs` — the aiStyle-1 per-type arm
  precedent (the dust-skip convention, the exact +16 clamp).
- ROADMAP rows: S1-C-4 (the damage dispatch), S1-A-9 (the sub-step
  lifetime), S1-A-10 (the RNG-order contract).
