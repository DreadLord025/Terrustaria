//! S1-P-i18 — the special bullets' per-type effects (`Projectile.cs`):
//! **36 MeteorShot** (the one-bounce / reflect shot), **89 CrystalBullet**
//! (the two-shard death split), **104 CursedBullet** (CursedInferno 39),
//! **207 ChlorophyteBullet** (the homing flight), **242 BulletHighVelocity**
//! (the speed-scaled fade), **279 IchorBullet** (Ichor 69), **283
//! VenomBullet** (Venom 70 + PotentAcid 395 + the three-box debuff-spread
//! death burst), **284 PartyBullet** (the confetti death — visuals only),
//! **285 NanoBullet** (Confused 31 + the one-bounce re-target), **286
//! ExplosiveBullet** (the idStatic-60 AoE death burst) and **287
//! GoldenBullet** (Midas 72). The kind set is [`SPECIAL_BULLET_KINDS`]
//! (`projectile/data.rs`, the Lead-staged rows) and the dispatch seam in
//! `ProjectilePool::update`'s aiStyle-1 arm gates on [`covers`] ONLY —
//! chained after the ai001 A/B/C / frostmoon / moon-lord lists (all
//! kind-disjoint); every other kind keeps the generic `ai_arrow` flight
//! BYTE-IDENTICAL.
//!
//! Census base (`aa147ce0`): NONE of these kinds had a `proj_data` row, so
//! `spawn_with_data_ai_owned`'s `proj_data(kind)?` declined every special
//! bullet silently — the shots never spawned. The Lead staged the twelve
//! `SetDefaults` rows + constants + the `ProjLight` colour arms
//! (data.rs / proj_lights.rs, cited per row) and the pool-side death
//! hooks (`special_bullet_kill` / `venom_burst_spread` /
//! `exploding_bullet_burst`); this module ports the per-type AI arms and
//! the on-hit / spread debuff arms. **638 MoonlordBullet** (the Luminite
//! bullet's infinite pierce + 0.96 decay) is already S1-D-4's
//! (`projectile_ai_moonlord`) — FAITHFUL, excluded here. **90
//! CrystalShard** is aiStyle 24 — the existing `ai_crystal_storm` arm
//! flies it (its generic 0.96/15-tick arm IS the 90's C# arm,
//! `:26004-26013`).
//!
//! Structure mirrors `AI_001`'s own order (the ai001_a/c precedent):
//! 1. the hook group above the counter (the 242 fade `:55660-55671`),
//! 2. the `flag3` counter switch (`:55947-56032` — every kind here EXCEPT
//!    207 rests; 207 counts),
//! 3. the hook group below the counter (the 207 homing block
//!    `:56778-56887`, which CONSUMES the gravity chain — the C# else-if),
//! 4. the rotation tail (all default `:57748-57751` — no kind here has a
//!    per-type rotation arm) + the exact C# +16 clamp (`:57753-57768`,
//!    slice A/C's convention — `flag4` is 1006-only),
//! 5. the movement: the `HandleMovement` per-axis probe stand-in with the
//!    36 bounce arm (`:19278-19298`) and the 285 bounce + re-target arm
//!    (`:18563-18591`); the port's probe stands in for the swept
//!    `Collision.TileCollision` (the typhoon 409 precedent,
//!    projectile.rs `ai_typhoon`'s bounce comment).
//!
//! The on-hit debuffs ride `ProjectilePool::damage_pve`'s
//! `special_bullet_status_npc` call ([`bullet_on_hit_debuffs`] —
//! `StatusNPC` `:11537-11551` + `:11665-11677`, at `:13580` BEFORE
//! `StrikeNPC` `:13649`); the death effects ride the pool's `if !active`
//! hook block (`ProjectilePool::special_bullet_kill` — `Kill()`'s
//! `:75250-75268` / `:75911-76039`; the chain's outer gate is
//! `!Main.projPet[type]` `:74815`, NOT a tile-death gate — every death
//! path fires them, the `moonlord_arrow_kill` precedent). The 283
//! spread's debuff-only re-hit ([`venom_burst_debuffs`]) is the
//! `type == 283 && !active` early arm (`:12810-12814`).
//!
//! ## RNG (S1-A-10, the `UpdateProjectiles` stream `Main.cs:17775-17777`)
//!
//! The AI arms take NO rng parameter: every `Main.rand` draw in these
//! kinds' `AI_001` blocks feeds dust alone (the 207's trail dust 75
//! `:56782-56792`, the 36/285 bounce `HitTiles`/sound) — the dust-skip
//! convention (ai001_a/b/c), so the flight is RNG-free BY CONSTRUCTION.
//! The state-bearing draws live at their C# sites on the pool's pass
//! stream (`rng` = the update loop's `Main.rand`):
//! - the 285 on-hit Confused roll: `Next(3) == 0` → 180 else 60
//!   (`StatusNPC` `:11541-11551`) — ONE draw per landed hit, at the
//!   StatusNPC position (before the strike, `:13580` < `:13649`);
//! - the 89 death split: per shard `Next(40, 70)` then `Next(-20, 21)`
//!   for X, then the same pair for Y (`Kill()` `:75264-75265`) — 8 draws,
//!   shard 0's four before shard 1's four, in the argument evaluation
//!   order. These are STATE-BEARING (they place AND velocity the shard),
//!   so they ride the pass stream INLINE in the pool's
//!   `special_bullet_kill` (the Lead wiring), not here.
//!
//! The death-burst dust/gore BODIES and their draws are SKIPPED with
//! their layers (the ai001_a/b/c dust-skip convention — the pool has no
//! dust/gore draw layer): the 283's three 10-dust-171 loops
//! (`:75913-75939`), the 284 confetti's 10 dust + 5 gore
//! (`:75977-75998`, purely visual — no state, no spawn, no damage), the
//! 286's 7 dust-31 + 6 dust-6 + 1 gore (`:76003-76018`) and the common
//! bullet arm's `HitTiles` + Item10 (`:76035-76039`). The 286 burst's
//! DAMAGE half draws only the per-NPC `DamageVar` the pool's boxed walk
//! already spends. This is the ai001 convention (NOT the
//! projectile_ai_misc draw-census convention, which keeps explicit
//! visual draws) — the Lead's chosen wiring for this family; recorded
//! in ANALYSIS "S1-P-i18 special bullets".
//!
//! ## Deliberate skips / stand-ins (ANALYSIS "S1-P-i18 special bullets")
//!
//! - Dust, gore, `Lighting`/`PlotTileLine`, `netUpdate` — no such layers
//!   in the pool (the ai001 convention). The `Kill()` sound cues are
//!   skipped with the pool's death-hook sound convention
//!   (`moonlord_arrow_kill`); the 36 bounce's `SoundID.Item10` rides
//!   `wants_sound` (the legacy-Item facade carries it).
//! - `Projectile.appliesImmunityTimeOnSingleHits` (the 286's `:3754`)
//!   has no `ProjData` field: the pool's idStatic application gate reads
//!   a kind check (`max_penetrate != 1 || kind == EXPLODING_BULLET`) in
//!   both `damage_pve` and `damage_pve_boxed` — the Lead row note.
//! - The 283's `localAI[1] = -1f` write (`:75942`) has NO reader in the
//!   C# (the `== -1f` readers are aiStyle-10's `:12456` and other kinds'
//!   arms) — a verbatim dead write, ported for its shape.
//! - The 285 PvP Confused arm (`StatusPlayer` `:12288-12298`) — the port
//!   has no projectile-PVP layer (single-player); recorded, not ported.
//! - The 207's `Collision.CanHit` is the port's [`can_hit`] walk (the
//!   blaze acquisition precedent, ai001_c); the 285 re-target uses the
//!   shared [`find_target_with_line_of_sight`] (`:45627-45660`) with the
//!   slot's local-immunity row.
//! - The movement probe kills without the C#'s final `position +=
//!   velocity` step on the blocked tick (the port-wide `ai_style1_flight`
//!   stand-in, byte-identical to slices A/B/C); the bounce arms integrate
//!   the reflected velocity on the bounce tick (the typhoon precedent).
//! - The 283 burst's `oldVelocity.ToRotation()` reads `p.vel` (the port
//!   has no `oldVelocity` field); the bullet RESTS its counter and flies
//!   straight, so `oldVelocity == velocity` at death — faithful for every
//!   reachable 283 (the Lead `venom_burst_spread` note).
//!
//! CODE-ONLY MODE: the witness tests at the bottom are WRITTEN NOT RUN
//! (no compiler invoked on this branch); every expected value is
//! hand-derived from the cited C# lines.

use glam::Vec2;
use tr_core::World;

use crate::npc::Npc;
use crate::projectile::{
    can_hit, chaseable, chaseable_held, find_target_with_line_of_sight, npc_center, Projectile,
    CHLOROPHYTE_BULLET, CURSED_BULLET, GOLDEN_BULLET, HIGH_VELOCITY_BULLET, ICHOR_BULLET,
    METEOR_SHOT, NANO_BULLET, SPECIAL_BULLET_KINDS, VENOM_BULLET,
};

/// Whether this module owns the kind's aiStyle-1 arm. Disjoint from the
/// ai001 A/B/C kind lists, the riders, the frostmoon set and the
/// moon-lord set (the seam chains this gate after them).
pub(crate) fn covers(kind: u16) -> bool {
    SPECIAL_BULLET_KINDS.contains(&kind)
}

/// `AI_001`'s special-bullet body for one slot, in the C#'s own block
/// order. `npcs` is the pool's NPC slice (the 207 homing scan
/// `:56824-56865` and the 285 bounce re-target `:18583-18590`);
/// `local_immunity` is the slot's 256-entry local-NPC cooldown row (the
/// re-target's `localNPCImmunity[i] != 0` exclusion, `:45645-45648`).
/// Every early `Kill(); return` is the port's death convention
/// (`active = false`, `time_left = 0`) plus the `kill_time_left` capture
/// (`Kill()`'s `num`, `:70649`) the pool's death hooks read.
pub(crate) fn step(
    p: &mut Projectile,
    world: &World,
    npcs: &[Npc],
    local_immunity: &[i32],
) {
    // ---- 1. The hook group above the counter ------------------------------
    // `:55660-55671` — the 242 HighVelocity fade (the 302/438/462/592
    // siblings are other modules' kinds): `num109 = (float)Math.Sqrt(
    // vx*vx + vy*vy)` (a DOUBLE sqrt cast back to float), `alpha -=
    // (byte)((double)num109 * 0.9)` while positive, clamp 0. The byte
    // cast truncates; speeds stay under the ±16 clamp so the product
    // never wraps 256 (the RAY_GUNNER 438 precedent, ai001_a).
    if p.kind == HIGH_VELOCITY_BULLET {
        let speed = ((p.vel.x * p.vel.x + p.vel.y * p.vel.y) as f64).sqrt() as f32;
        if p.alpha > 0 {
            p.alpha -= (speed as f64 * 0.9) as u8 as i32;
        }
        if p.alpha < 0 {
            p.alpha = 0;
        }
    }

    // ---- 2. The flight counter (`:55945-56041`) ---------------------------
    // The `flag3 = false` list carries 36 (`:55952`), 89 (`:55958`), 104
    // (`:55961`), 242 (`:55969`), 279 (`:55975`), 283-287
    // (`:55976-55980`) — they rest. 207 is NOT on the list: it counts
    // like an arrow (its homing block consumes the gravity chain, so the
    // generic `ai[0] >= 15f` arm never fires for it either).
    if p.kind == CHLOROPHYTE_BULLET {
        p.ai[0] += 1.0;
    }

    // ---- 3. The hook group below the counter ------------------------------
    // `:56778-56887` — the 207 Chlorophyte homing block (the 837 Book of
    // Skulls sibling is `ai_book_of_skulls`' kind). It sits in the C#'s
    // pre-gravity else-if chain (`:56753` `if (type == 357) … else if
    // (type == 207 || type == 837)`), so a homing bullet NEVER reaches
    // the generic gravity arm — the steer replaces it.
    if p.kind == CHLOROPHYTE_BULLET {
        chlorophyte_homing(p, world, npcs);
    }

    // ---- 4. The rotation tail + clamp -------------------------------------
    // `:57748-57751` — the default tail: `rotation = Atan2(vy, vx) +
    // 1.57f` (the literal 1.57, the C# spells the constant). No kind here
    // has a per-type rotation arm.
    p.rot = p.vel.y.atan2(p.vel.x) + 1.57;
    // `:57753-57768` — `flag8` true (only 577 opts out), `flag4` false
    // (1006-only): just the +16 bound (slice A/C's exact-tail convention).
    if p.vel.y > 16.0 {
        p.vel.y = 16.0;
    }

    // ---- 5. The movement (the HandleMovement stand-in) --------------------
    move_bullet(p, world, npcs, local_immunity);
}

/// The 207's homing block (`Projectile.cs:56778-56887`), in the C#'s own
/// order: the trail-dust window (SKIPPED — dust-only, `:56780-56793`),
/// the speed latch, the alpha fade, the acquire scan, the held-target
/// validity check and the steer blend.
fn chlorophyte_homing(p: &mut Projectile, world: &World, npcs: &[Npc]) {
    // `:56802-56808` — `num185 = (float)Math.Sqrt(vx*vx + vy*vy)`; the
    // first tick latches `localAI[0]` to the launch speed (the steer
    // keeps THAT speed, not the current one).
    let launch = ((p.vel.x * p.vel.x + p.vel.y * p.vel.y) as f64).sqrt() as f32;
    if p.local_ai[0] == 0.0 {
        p.local_ai[0] = launch;
    }
    let speed = p.local_ai[0];
    // `:56809-56816` — `alpha -= 25` while positive, clamp 0.
    if p.alpha > 0 {
        p.alpha -= 25;
    }
    if p.alpha < 0 {
        p.alpha = 0;
    }
    // `:56817-56821` — the scan state: `num187/num188` the target centre
    // (seeded at the projectile's own position), `num189 = 300f` the
    // Manhattan range, `flag5` the found flag, `num190` the picked index.
    let cx = p.pos.x + p.w * 0.5;
    let cy = p.pos.y + p.h * 0.5;
    let mut tx = p.pos.x;
    let mut ty = p.pos.y;
    let mut best = 300.0f32;
    let mut found = false;
    // `:56822-56846` — the acquire scan, only while `ai[1] == 0f`: every
    // chaseable NPC inside the Manhattan 300 with a clear `Collision.
    // CanHit` line; the running minimum wins (a true nearest — the
    // later index takes an exact tie, `<` not `<=`). The C# writes
    // `ai[1] = num190 + 1` AFTER the loop (`:56841-56844`) and resets
    // `flag5 = false` (`:56845`) — the held-target block below re-sets
    // it from the fresh lock, so the acquire tick DOES steer.
    if p.ai[1] == 0.0 {
        let mut picked = 0usize;
        for (i, n) in npcs.iter().enumerate() {
            if !chaseable(n) {
                continue;
            }
            let (nw, nh) = n.size();
            let nx = n.x + nw * 0.5;
            let ny = n.y + nh * 0.5;
            let manhattan = (cx - nx).abs() + (cy - ny).abs();
            if manhattan < best && can_hit(world, cx, cy, 1.0, 1.0, n.x, n.y, nw, nh) {
                best = manhattan;
                tx = nx;
                ty = ny;
                found = true;
                picked = i;
            }
        }
        if found {
            // `:56843` — `ai[1] = num190 + 1` (the 1-based lock).
            p.ai[1] = (picked + 1) as f32;
        }
        // `:56845` — `flag5 = false`.
        found = false;
    }
    // `:56847-56865` — the held-target validity: `npc[ai[1]-1]` active
    // and `CanBeChasedBy(ignoreDontTakeDamage: true)` and
    // `!dontTakeDamage`, inside the Manhattan 1000; else the lock drops
    // (`ai[1] = 0f`).
    if p.ai[1] > 0.0 {
        let idx = (p.ai[1] - 1.0) as usize;
        let mut held = false;
        if let Some(n) = npcs.get(idx) {
            if chaseable_held(n) && !n.dont_take_damage {
                let nc = npc_center(n);
                if (cx - nc.x).abs() + (cy - nc.y).abs() < 1000.0 {
                    held = true;
                    tx = nc.x;
                    ty = nc.y;
                }
            }
        }
        if held {
            found = true;
        } else {
            p.ai[1] = 0.0;
        }
    }
    // `:56866-56869` — `if (!this.friendly) flag5 = false`: a hostile
    // 207 never steers (the pool's friendly default is true).
    if !p.friendly {
        found = false;
    }
    // `:56870-56886` — the steer: the to-target direction at the LATCHED
    // speed, blended `(velocity * (num202 - 1) + d) / num202` with
    // `num202 = 8` (the 837 sibling uses 20 — not this kind).
    if found {
        let dx = tx - cx;
        let dy = ty - cy;
        // `:56876` — `num201 = num198 / (float)Math.Sqrt(dx*dx + dy*dy)`
        // (the double sqrt; an exactly-coincident centre divides by zero
        // in the C# too — the port keeps the unguarded division's NaN
        // out of the state by the epsilon guard the pool's scans use,
        // the `steer_blend` precedent).
        let len = ((dx * dx + dy * dy) as f64).sqrt() as f32;
        let k = speed / len.max(f32::EPSILON);
        let (dx, dy) = (dx * k, dy * k);
        p.vel.x = (p.vel.x * 7.0 + dx) / 8.0;
        p.vel.y = (p.vel.y * 7.0 + dy) / 8.0;
    }
}

/// The movement half — the port's per-axis destination-box probe (the
/// `ai_style1_flight` stand-in, `HandleMovement`'s default arm
/// `:19849-19853`) with the two bounce arms substituted: the 36's
/// one-bounce reflect (`:19278-19298`) and the 285's bounce + re-target
/// (`:18563-18591`). The C# `lastVelocity` is the PRE-collision velocity
/// (`:17587`); the port's probe reads the blocked axis the same way the
/// typhoon 409 arm does (projectile.rs `ai_typhoon`).
fn move_bullet(p: &mut Projectile, world: &World, npcs: &[Npc], local_immunity: &[i32]) {
    if !p.tile_collide {
        p.pos += p.vel;
        return;
    }
    let last = p.vel;
    let nx = p.pos.x + p.vel.x;
    let ny = p.pos.y + p.vel.y;
    let hit_x = crate::player::collides_rect(world, nx, p.pos.y, p.w, p.h);
    let hit_y = crate::player::collides_rect(world, p.pos.x, ny, p.w, p.h);
    if !hit_x && !hit_y {
        p.pos.x = nx;
        p.pos.y = ny;
        return;
    }
    match p.kind {
        // `:19278-19298` — the MeteorShot bounce: while `penetrate > 1`
        // the shot reflects the blocked axis at FULL strength
        // (`0f - lastVelocity`), spends one penetrate and lives; at
        // `penetrate <= 1` it dies. The `HitTiles` + `SoundID.Item10`
        // (`:19282-19283`) ride the cue queue (the dust is SKIPPED).
        METEOR_SHOT => {
            if p.penetrate > 1 {
                p.wants_sound = Some((2, 10, p.pos.x, p.pos.y, 0.0, 1.0));
                p.penetrate -= 1;
                if hit_x {
                    p.vel.x = -last.x;
                }
                if hit_y {
                    p.vel.y = -last.y;
                }
                // The C# `UpdatePosition` (`:20445`) integrates the
                // POST-bounce velocity on the bounce tick (the typhoon
                // precedent); the probe's blocked-axis stand-in keeps
                // the other axis sliding.
                p.pos += p.vel;
            } else {
                kill(p);
            }
        }
        // `:18563-18591` — the NanoBullet bounce: `ai[1]++`; the FIRST
        // bounce (`ai[1] == 1f`) scales the damage ×0.66 (the `(int)`
        // truncation); the SECOND (`ai[1] >= 2f`) kills BEFORE the
        // reflect. The reflect is full-strength on each blocked axis,
        // then `FindTargetWithLineOfSight()` (the 800f default,
        // `:45627-45660`) re-aims the velocity at the nearest chaseable
        // NPC with a clear line and no local immunity — the speed
        // survives (`DirectionTo(...).SafeNormalize(-UnitY) *
        // velocity.Length()`).
        NANO_BULLET => {
            p.ai[1] += 1.0;
            if p.ai[1] == 1.0 {
                p.damage = (p.damage as f32 * 0.66) as i32;
            }
            if p.ai[1] >= 2.0 {
                kill(p);
                return;
            }
            if hit_x {
                p.vel.x = -last.x;
            }
            if hit_y {
                p.vel.y = -last.y;
            }
            let target = find_target_with_line_of_sight(
                world,
                npcs,
                p.pos,
                p.w,
                p.h,
                800.0,
                local_immunity,
            );
            if target != -1 {
                let tc = npc_center(&npcs[target as usize]);
                let c = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
                let speed = p.vel.length();
                let dir = tc - c;
                // `SafeNormalize(-Vector2.UnitY)` (`:18588`): a zero
                // delta (the centre exactly on the NPC) falls back to
                // (0, -1).
                let dir = if dir == Vec2::ZERO {
                    Vec2::new(0.0, -1.0)
                } else {
                    dir.normalize()
                };
                p.vel = dir * speed;
            }
            p.pos += p.vel;
        }
        // Every other special bullet keeps the generic kill-on-tile
        // default arm (`:19849-19853`'s `position += velocity; Kill()`
        // — the port's probe stand-in kills without the final step, the
        // ai_style1_flight convention, byte-identical to slices A/B/C).
        _ => kill(p),
    }
}

/// The port's death convention plus `Kill()`'s `num = timeLeft` capture
/// (`:70649`, taken BEFORE the `:70650` zeroing) — the pool's death
/// hooks read `kill_time_left`.
fn kill(p: &mut Projectile) {
    p.kill_time_left = p.time_left;
    p.active = false;
    p.time_left = 0;
}

/// `StatusNPC`'s special-bullet debuff arms (`Projectile.cs:11537-11551`
/// + `:11665-11677`), called from `damage_pve`'s
/// `special_bullet_status_npc` at the StatusNPC position (`:13580` —
/// BEFORE `StrikeNPC` `:13649`). The C# order: the 287 Midas and the 285
/// Confused roll are standalone `if`s ABOVE the else-if chain; the
/// 104/279/283 arms are chain members (mutually exclusive — one kind
/// hits one arm).
///
/// RNG: the 285's `Main.rand.Next(3) == 0` (`:11543`) is ONE draw per
/// landed hit on the pass's `UpdateProjectiles` stream — 180 ticks on a
/// zero, 60 otherwise. Every other arm here is draw-free.
pub(crate) fn bullet_on_hit_debuffs(
    kind: u16,
    npc: &mut Npc,
    rng: &mut dyn tr_core::loot::DropRng,
) {
    // `:11537-11540` — GoldenBullet: Midas 72, 120 ticks.
    if kind == GOLDEN_BULLET {
        npc.buffs.add_buff(72, 120);
    }
    // `:11541-11551` — NanoBullet: Confused 31, `Next(3) == 0` → 180
    // else 60.
    if kind == NANO_BULLET {
        if rng.next_range(0, 3) == 0 {
            npc.buffs.add_buff(31, 180);
        } else {
            npc.buffs.add_buff(31, 60);
        }
    }
    // The else-if chain (`:11665-11677`) — the ported members only (the
    // 95/103/482, 278/280 and 282 siblings are unported kinds).
    if kind == CURSED_BULLET {
        // `:11665-11668` — CursedInferno 39, 420 ticks.
        npc.buffs.add_buff(39, 420);
    } else if kind == ICHOR_BULLET {
        // `:11669-11672` — Ichor 69, 600 ticks.
        npc.buffs.add_buff(69, 600);
    } else if kind == VENOM_BULLET {
        // `:11673-11677` — Venom 70 (600) + PotentAcid 395 (120).
        npc.buffs.add_buff(70, 600);
        npc.buffs.add_buff(395, 120);
    }
}

/// The 283 VenomBullet's death-burst debuff spread (`StatusNPC` called
/// from the burst's three `Damage()` walks through the
/// `type == 283 && !active` early arm, `Projectile.cs:12810-12814`): the
/// hit NPC takes Venom 70 (600) + PotentAcid 395 (120) and NOTHING else
/// — the early return skips the strike, the crit/variance rolls and the
/// rest of StatusNPC. Draw-free (the pool's `venom_spread_walk` calls it
/// directly, NOT through `bullet_on_hit_debuffs`, so no 285-style roll).
pub(crate) fn venom_burst_debuffs(npc: &mut Npc) {
    npc.buffs.add_buff(70, 600);
    npc.buffs.add_buff(395, 120);
}

// =====================================================================
// Witness tests — WRITTEN, NOT RUN (code-only mode: no cargo/rustc).
// Every expected value below is hand-derived from the cited C# lines;
// the RNG-order argument mirrors the kept draws on a clone stream.
// =====================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::{Npc, SlimeVariant};
    use crate::projectile::{
        proj_data, FlailInputs, ProjectilePool, YoyoInputs, CRYSTAL_BULLET, CRYSTAL_SHARD,
        EXPLODING_BULLET, MUSKET_BALL, PARTY_BULLET,
    };
    use tr_core::tile_light::FastRandom;
    use tr_core::{World, TILE_PX};

    /// A tile-free world: the flight witnesses run in open air.
    fn world() -> World {
        World::new("t".into(), 512, 64)
    }

    /// A slot carrying its `SetDefaults` row slice, spawned at (160, 160)
    /// with the given velocity. Field-by-field on `Default` — the struct
    /// has private bookkeeping fields (E0451 outside projectile.rs).
    fn proj(kind: u16, vx: f32, vy: f32) -> Projectile {
        let d = proj_data(kind).expect("the Lead-staged row");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(10.0 * TILE_PX, 10.0 * TILE_PX);
        p.vel = Vec2::new(vx, vy);
        p.w = d.w;
        p.h = d.h;
        p.tile_collide = d.tile_collide;
        p.alpha = d.alpha;
        p.scale = d.scale;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.max_penetrate = d.penetrate;
        p.extra_updates = d.extra_updates;
        p.uses_local_immunity = d.uses_local_immunity;
        p.local_hit_cooldown = d.local_hit_cooldown;
        p.uses_id_static_immunity = d.uses_id_static_immunity;
        p.id_static_hit_cooldown = d.id_static_hit_cooldown;
        p
    }

    fn step_pool(pool: &mut ProjectilePool, w: &mut World, npcs: &mut [Npc]) {
        let mut rng = FastRandom::new(7);
        pool.update(
            w,
            npcs,
            None,
            None,
            None,
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut rng,
            0,
            false,
        );
    }

    /// A chaseable blue slime at the given centre (defense 0 so the
    /// damage witnesses read the raw strike).
    fn slime(cx: f32, cy: f32) -> Npc {
        let mut n = Npc::spawn_slime_sky(cx, cy, SlimeVariant::Blue);
        n.defense_override = Some(0);
        n
    }

    /// The Lead-staged rows are the verbatim `SetDefaults` slices
    /// (`Projectile.cs:1182-1197` / `:1715-1742` / `:1915-1928` /
    /// `:2948-2961` / `:3238-3254` / `:3650-3663` / `:3699-3771`).
    #[test]
    fn s1_p_i18_the_rows_are_verbatim() {
        let m = proj_data(METEOR_SHOT).unwrap();
        assert_eq!((m.w, m.h, m.ai_style, m.penetrate), (4.0, 4.0, 1, 2));
        assert_eq!((m.time_left, m.extra_updates), (600, 1));
        assert!(m.uses_local_immunity);
        assert_eq!(m.local_hit_cooldown, 7);
        assert_eq!((m.alpha, m.scale), (255, 1.4));
        let c = proj_data(CRYSTAL_BULLET).unwrap();
        assert_eq!((c.penetrate, c.extra_updates, c.scale), (1, 1, 1.2));
        let s = proj_data(CRYSTAL_SHARD).unwrap();
        assert_eq!((s.w, s.h, s.ai_style), (6.0, 6.0, 24));
        assert!(!s.tile_collide);
        assert_eq!((s.alpha, s.scale), (50, 1.2));
        let cb = proj_data(CURSED_BULLET).unwrap();
        assert_eq!((cb.extra_updates, cb.scale), (2, 1.2));
        let ch = proj_data(CHLOROPHYTE_BULLET).unwrap();
        assert_eq!((ch.extra_updates, ch.scale), (2, 1.2));
        let hv = proj_data(HIGH_VELOCITY_BULLET).unwrap();
        assert_eq!((hv.penetrate, hv.extra_updates), (3, 7));
        assert_eq!(hv.scale, 1.18);
        assert!(hv.ignore_water);
        assert!(hv.uses_local_immunity);
        assert_eq!(hv.local_hit_cooldown, -1);
        let ic = proj_data(ICHOR_BULLET).unwrap();
        assert_eq!(ic.scale, 1.25);
        let v = proj_data(VENOM_BULLET).unwrap();
        assert_eq!(v.scale, 1.25);
        let pa = proj_data(PARTY_BULLET).unwrap();
        assert_eq!(pa.scale, 1.3);
        let na = proj_data(NANO_BULLET).unwrap();
        assert_eq!(na.scale, 1.3);
        let ex = proj_data(EXPLODING_BULLET).unwrap();
        assert!(ex.uses_id_static_immunity);
        assert_eq!(ex.id_static_hit_cooldown, 60);
        assert_eq!(ex.scale, 1.3);
        let g = proj_data(GOLDEN_BULLET).unwrap();
        assert_eq!(g.scale, 1.3);
    }

    /// The gate claims ONLY the eleven aiStyle-1 specials — never the
    /// ai001 A/B/C kinds, the riders, the moon-lord set, the 90 shard
    /// (aiStyle 24) or the plain musket ball.
    #[test]
    fn s1_p_i18_the_gate_is_disjoint() {
        for k in [1u16, 14, 81, 98, 184, 259, 270, 325, 389, 436, 438, 449,
                  462, 581, 601, 622, 680, 686, 962, 980, 110, 275, 276,
                  634, 635, 645, 662, 675, 685, 20, 88, 355, 660, 837,
                  638, 639, 640, 90] {
            assert!(!covers(k), "kind {k} belongs to another arm");
        }
        for k in SPECIAL_BULLET_KINDS {
            assert!(covers(k));
        }
    }

    /// The resting counters (`:55952-55980`): 36/89/104/242/279/283-287
    /// hold `ai[0]` at 0; the 207 counts (`:56038-56041`, not on the
    /// list). One sub-step at (2, 0) in open air.
    #[test]
    fn s1_p_i18_the_counters_rest_except_the_chlorophyte() {
        let w = world();
        for k in [METEOR_SHOT, CRYSTAL_BULLET, CURSED_BULLET, HIGH_VELOCITY_BULLET,
                  ICHOR_BULLET, VENOM_BULLET, PARTY_BULLET, NANO_BULLET,
                  EXPLODING_BULLET, GOLDEN_BULLET] {
            let mut p = proj(k, 2.0, 0.0);
            step(&mut p, &w, &[], &[0; 256]);
            assert_eq!(p.ai[0], 0.0, "kind {k} rests");
            assert_eq!(p.vel.y, 0.0, "no gravity — the generic arm is unreachable");
        }
        let mut ch = proj(CHLOROPHYTE_BULLET, 2.0, 0.0);
        step(&mut ch, &w, &[], &[0; 256]);
        assert_eq!(ch.ai[0], 1.0, "the 207 counts");
        assert_eq!(ch.vel.y, 0.0, "the homing block consumes the gravity chain");
    }

    /// The 242 fade (`:55660-55671`): speed 10 → `(byte)(10.0 * 0.9)` = 9
    /// per sub-step; 255 → 246 after one. The floor clamps at 0.
    #[test]
    fn s1_p_i18_the_high_velocity_fades_by_the_truncated_speed_product() {
        let w = world();
        let mut p = proj(HIGH_VELOCITY_BULLET, 10.0, 0.0);
        step(&mut p, &w, &[], &[0; 256]);
        assert_eq!(p.alpha, 246);
        p.vel = Vec2::ZERO;
        step(&mut p, &w, &[], &[0; 256]);
        assert_eq!(p.alpha, 246, "speed 0 → no fade");
        p.vel = Vec2::new(10.0, 0.0);
        p.alpha = 2;
        step(&mut p, &w, &[], &[0; 256]);
        assert_eq!(p.alpha, 0, "the floor clamps");
    }

    /// The 207 acquire (`:56822-56846`): a chaseable NPC inside the
    /// Manhattan 300 with a clear line latches `ai[1] = index + 1`; the
    /// latch tick steers through the held block (`:56847-56886`) at the
    /// LATCHED speed (`localAI[0]`), blend `(v * 7 + d) / 8`.
    #[test]
    fn s1_p_i18_the_chlorophyte_acquires_and_steers() {
        let w = world();
        // The bullet at (160, 160) with the 4×4 box → centre (162, 162),
        // flying +x at 4; the slime centre at (162, 260) — Manhattan 98
        // < 300, straight below.
        let mut n = slime(162.0, 260.0);
        let (nw, nh) = n.size();
        n.x = 162.0 - nw * 0.5;
        n.y = 260.0 - nh * 0.5;
        let mut p = proj(CHLOROPHYTE_BULLET, 4.0, 0.0);
        step(&mut p, &w, std::slice::from_ref(&n), &[0; 256]);
        assert_eq!(p.ai[1], 1.0, "the 1-based lock");
        assert_eq!(p.local_ai[0], 4.0, "the latched launch speed");
        assert_eq!(p.alpha, 230, "the fade 25");
        // The steer: d = (0, 4) (the to-target direction at speed 4);
        // vel = ((4*7 + 0)/8, (0*7 + 4)/8) = (3.5, 0.5).
        assert!((p.vel.x - 3.5).abs() < 1e-5, "vx {}", p.vel.x);
        assert!((p.vel.y - 0.5).abs() < 1e-5, "vy {}", p.vel.y);
        // The second tick keeps the lock (no re-scan) and keeps blending.
        step(&mut p, &w, std::slice::from_ref(&n), &[0; 256]);
        assert_eq!(p.ai[1], 1.0);
        assert!(p.vel.y > 0.5, "the blend keeps turning down");
    }

    /// The 207 lock drop (`:56861-56864`): a `dontTakeDamage` target
    /// clears `ai[1]` and the shot flies straight again.
    #[test]
    fn s1_p_i18_the_chlorophyte_drops_a_shielded_lock() {
        let w = world();
        let mut n = slime(162.0, 260.0);
        let (nw, nh) = n.size();
        n.x = 162.0 - nw * 0.5;
        n.y = 260.0 - nh * 0.5;
        let mut p = proj(CHLOROPHYTE_BULLET, 4.0, 0.0);
        step(&mut p, &w, std::slice::from_ref(&n), &[0; 256]);
        assert_eq!(p.ai[1], 1.0);
        n.dont_take_damage = true;
        let vx = p.vel.x;
        step(&mut p, &w, std::slice::from_ref(&n), &[0; 256]);
        assert_eq!(p.ai[1], 0.0, "the lock drops");
        assert_eq!(p.vel.x, vx, "no steer without a lock");
    }

    /// The 36 bounce (`:19278-19298`): penetrate 2 → the first blocked
    /// move reflects the blocked axis at full strength, spends one
    /// penetrate and queues Item10; the second blocked move kills.
    #[test]
    fn s1_p_i18_the_meteor_bounces_once_then_dies() {
        let mut w = world();
        // A wall at tile x = 12 (px 192): the 4-wide bullet at 186 flies
        // 4 px into it on the first sub-step.
        for y in 8..14 {
            let i = w.index(12, y);
            w.tiles[i].kind = 1;
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        let mut p = proj(METEOR_SHOT, 4.0, 0.0);
        p.pos = Vec2::new(186.0, 160.0);
        step(&mut p, &w, &[], &[0; 256]);
        assert!(p.active, "the first bounce lives");
        assert_eq!(p.penetrate, 1, "one penetrate spent");
        assert_eq!(p.vel.x, -4.0, "the full-strength reflect");
        assert_eq!(p.wants_sound, Some((2, 10, 186.0, 160.0, 0.0, 1.0)));
        // Fly back left into a second wall at tile x = 10 (px 160).
        for y in 8..14 {
            let i = w.index(10, y);
            w.tiles[i].kind = 1;
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        p.pos = Vec2::new(162.0, 160.0);
        p.wants_sound = None;
        step(&mut p, &w, &[], &[0; 256]);
        assert!(!p.active, "penetrate 1 → the second blocked move kills");
        assert_eq!(p.time_left, 0);
    }

    /// The 285 bounce (`:18563-18591`): the first bounce scales the
    /// damage ×0.66 (`(int)` truncation: 100 → 66), reflects and
    /// re-targets; the second kills BEFORE the reflect.
    #[test]
    fn s1_p_i18_the_nano_bounces_once_with_the_066_scale() {
        let mut w = world();
        for y in 8..14 {
            let i = w.index(12, y);
            w.tiles[i].kind = 1;
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        let mut p = proj(NANO_BULLET, 4.0, 0.0);
        p.pos = Vec2::new(186.0, 160.0);
        p.damage = 100;
        step(&mut p, &w, &[], &[0; 256]);
        assert!(p.active);
        assert_eq!(p.ai[1], 1.0);
        assert_eq!(p.damage, 66, "`(int)(100 * 0.66f)`");
        assert_eq!(p.vel.x, -4.0, "the reflect (no target in range)");
        // The second blocked move: ai[1] → 2 → Kill BEFORE the reflect.
        for y in 8..14 {
            let i = w.index(10, y);
            w.tiles[i].kind = 1;
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        p.pos = Vec2::new(162.0, 160.0);
        step(&mut p, &w, &[], &[0; 256]);
        assert!(!p.active, "the second bounce kills");
        assert_eq!(p.damage, 66, "no second scale");
    }

    /// The 285 re-target (`:18583-18590` + `:45627-45660`): a chaseable
    /// NPC inside the Euclidean 800 with a clear line re-aims the
    /// reflected velocity at it, the speed surviving.
    #[test]
    fn s1_p_i18_the_nano_retargets_after_the_bounce() {
        let mut w = world();
        for y in 8..14 {
            let i = w.index(12, y);
            w.tiles[i].kind = 1;
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        // The slime centre at (188, 260): below the bounce point, inside
        // 800, clear line (the wall is at x = 192, the NPC at 188).
        let mut n = slime(188.0, 260.0);
        let (nw, nh) = n.size();
        n.x = 188.0 - nw * 0.5;
        n.y = 260.0 - nh * 0.5;
        let mut p = proj(NANO_BULLET, 4.0, 0.0);
        p.pos = Vec2::new(186.0, 160.0);
        step(&mut p, &w, std::slice::from_ref(&n), &[0; 256]);
        assert!(p.active);
        // The reflected (-4, 0) re-aimed at (188, 260) from the centre
        // (188, 162): the direction (0, 98) normalised → (0, 1) × 4.
        assert!((p.vel.x - 0.0).abs() < 1e-4, "vx {}", p.vel.x);
        assert!((p.vel.y - 4.0).abs() < 1e-4, "vy {}", p.vel.y);
        assert!((p.vel.length() - 4.0).abs() < 1e-4, "the speed survives");
    }

    /// The on-hit debuffs (`StatusNPC` `:11537-11551` + `:11665-11677`):
    /// the durations and the 285's one-draw Confused roll. The roll is
    /// mirrored on a clone stream — the raw value pins the branch.
    #[test]
    fn s1_p_i18_the_on_hit_debuffs_carry_their_durations() {
        let mut rng = FastRandom::new(7);
        let mut n = slime(160.0, 160.0);
        bullet_on_hit_debuffs(GOLDEN_BULLET, &mut n, &mut rng);
        assert!(n.buffs.has_buff(72), "Midas");
        assert_eq!(n.buffs.buff_time_of(72), 120);
        let mut n = slime(160.0, 160.0);
        bullet_on_hit_debuffs(CURSED_BULLET, &mut n, &mut rng);
        assert!(n.buffs.has_buff(39), "CursedInferno");
        assert_eq!(n.buffs.buff_time_of(39), 420);
        let mut n = slime(160.0, 160.0);
        bullet_on_hit_debuffs(ICHOR_BULLET, &mut n, &mut rng);
        assert!(n.buffs.has_buff(69), "Ichor");
        assert_eq!(n.buffs.buff_time_of(69), 600);
        let mut n = slime(160.0, 160.0);
        bullet_on_hit_debuffs(VENOM_BULLET, &mut n, &mut rng);
        assert!(n.buffs.has_buff(70), "Venom");
        assert_eq!(n.buffs.buff_time_of(70), 600);
        assert!(n.buffs.has_buff(395), "PotentAcid");
        assert_eq!(n.buffs.buff_time_of(395), 120);
        // The 285: ONE Next(3) draw per hit — mirror it on a clone and
        // pin the branch the raw value selects.
        let mut mirror = FastRandom::new(7);
        let raw = mirror.next_range(0, 3);
        let mut n = slime(160.0, 160.0);
        bullet_on_hit_debuffs(NANO_BULLET, &mut n, &mut rng);
        assert!(n.buffs.has_buff(31), "Confused either way");
        assert_eq!(
            n.buffs.buff_time_of(31),
            if raw == 0 { 180 } else { 60 },
            "the roll's branch"
        );
        // A non-special kind draws nothing and applies nothing.
        let mut n = slime(160.0, 160.0);
        bullet_on_hit_debuffs(MUSKET_BALL, &mut n, &mut rng);
        assert!(!n.buffs.has_buff(31));
    }

    /// The 283 burst spread (`:12810-12814` → StatusNPC): Venom +
    /// PotentAcid only, draw-free.
    #[test]
    fn s1_p_i18_the_venom_burst_spreads_both_debuffs() {
        let mut n = slime(160.0, 160.0);
        venom_burst_debuffs(&mut n);
        assert!(n.buffs.has_buff(70));
        assert_eq!(n.buffs.buff_time_of(70), 600);
        assert!(n.buffs.has_buff(395));
        assert_eq!(n.buffs.buff_time_of(395), 120);
        assert!(
            !n.buffs.has_buff(31),
            "no Confused — the early arm is Venom-only"
        );
    }

    /// The pool seam: a special bullet takes the new arm through the
    /// dispatch (the 242 fade rides), the plain musket ball keeps slice
    /// A's resting-counter flight byte-identical.
    #[test]
    fn s1_p_i18_the_seam_chains_after_the_other_arms() {
        let mut w = world();
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data(HIGH_VELOCITY_BULLET, 160.0, 160.0, 10.0, 0.0, 200, 0.0);
        step_pool(&mut pool, &mut w, &mut []);
        // extraUpdates 7 → 8 sub-steps; the fade spends 9 each: 255 − 72.
        assert_eq!(pool.slots()[0].alpha, 183, "the seam routed 242 here");
        assert_eq!(pool.slots()[0].ai[0], 0.0, "the counter rests");

        let mut musket = ProjectilePool::new();
        musket.spawn_with_data(MUSKET_BALL, 160.0, 160.0, 2.0, 0.0, 200, 0.0);
        let mut w = world();
        for _ in 0..30 {
            step_pool(&mut musket, &mut w, &mut []);
        }
        assert_eq!(musket.slots()[0].ai[0], 0.0, "slice A still owns 14");
        assert_eq!(musket.slots()[0].vel.y, 0.0);
    }
}
