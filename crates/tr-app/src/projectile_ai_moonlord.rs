//! S1-D-4 moon-lord projectile family completeness — the three Moon Lord
//! drop-weapon shots' `AI_001` per-type arms (`Projectile.cs:54002-57798`):
//! **638 MoonlordBullet** (the S.D.M.G.'s bullet), **639 MoonlordArrow** (the
//! Phantasm's arrow) and **640 MoonlordArrowTrail** (the arrow's ghost).
//!
//! Census (the family's other members are already armed on main):
//! - 452 PhantasmalEye / 454 PhantasmalSphere / 455 PhantasmalDeathray /
//!   456 MoonLeech — rows `projectile/data.rs` + arms `ai_phasmal_eye` /
//!   `ai_phasmal_sphere` / `ai_phasmal_deathray` / `ai_moon_leech`
//!   (projectile.rs, the aiStyle-82/83/84/85 dispatch).
//! - 462 PhantasmalBolt / 622 BlowupSmokeMoonlord — `projectile_ai001_b`.
//! - 641 MoonlordTurret / 642 MoonlordTurretLaser — S1-D-4b-3
//!   (`ai_floating_turrets` / `ai_moonlord_turret_laser`).
//! - 645 LunarFlare — the row exists under the misnamed `POISON_DART`
//!   constant (the C# `ProjectileID.PoisonDart` is 98, `ProjectileID.cs:750`;
//!   645 is `LunarFlare`, `:1844`); the placed-cloud arm is in flight on
//!   `task/S1-D-5-aistyle1-types-c` — EXCLUDED here (kind-disjoint).
//! - 634/635 NebulaBlaze — ai001-C (R19b), EXCLUDED.
//! - 615 VortexBeater / 616 VortexBeaterRocket / 607 SolarFlareRay /
//!   625-628 StardustDragon — lunar WEAPON projectiles, deferred (the census
//!   table + C# cites live in ANALYSIS "The Moon Lord projectile family
//!   (S1-D-4)"). The 615's style-75 dispatch currently falls into the Last
//!   Prism arm — a census finding recorded there, not changed here.
//!
//! The seam: `ProjectilePool::update`'s aiStyle-1 arm gates on [`covers`]
//! ONLY, chained after the ai001 A/B kind lists (all disjoint — A owns the
//! low-id arrow family, B the 449-980 slice, C the 110-685 mid-range slice;
//! none lists 638/639/640). Every other kind keeps the generic `ai_arrow`
//! flight BYTE-IDENTICAL.
//!
//! Structure mirrors `AI_001`'s own order (the ai001_b precedent):
//! 1. the hook group above the counter (638 `:55864-55899`, 639
//!    `:55415-55429`, 640 `:55430-55482`),
//! 2. the `flag3` counter switch (`:55945-56041` — 638 `:56018` and 639
//!    `:56019` REST; 640 is NOT on the list and counts),
//! 3. the gravity else-if chain (639's own arm `:57542-57548`; 640 takes the
//!    generic `ai[0] >= 15` arm `:57568-57579` with the C# pin),
//! 4. the rotation tail (640's own `:57737-57743`, the others the default
//!    `:57750`) + the ±16 clamp (`:57760-57767`, the port convention) +
//!    movement (the `HandleMovement` stand-in; 640's stick arm
//!    `:18308-18316` + the `:17590` collision skip).
//!
//! The 640's `ai[1]` spawn-delay countdown (`Projectile.cs:16679-16683`) is
//! pool-side (it skips the WHOLE sub-step, ahead of the out-of-world guard)
//! — it lives in `ProjectilePool::update`, cited there. The 639's `Kill()`
//! trail spawn (`:73431-73452`) is the pool's death hook
//! (`moonlord_arrow_kill`, projectile.rs); the 638/639/640 on-hit damage
//! decay (`:14177-14180`) rides `damage_pve`'s landed block.
//!
//! CODE-ONLY MODE (S1-D-4): the witness tests at the bottom are WRITTEN NOT
//! RUN (no compiler invoked on this branch); they were verified by re-reading
//! the cited C# blocks and hand arithmetic.
//!
//! RNG by construction: this module takes NO rng parameter and draws
//! nothing. Every C# `Main.rand` draw in the three arms is dust-only and is
//! SKIPPED with the dust layer (the ai001_a/b convention, cited at each
//! point): the 638's proximity burst `Next(15, 31)` (`:55889`), the 640's
//! `Next(extraUpdates)` / `Next(2|6)` / `Next(2)` / `NextDouble` / `Next(10)`
//! dust gates (`:55453-55470`) and its `numUpdates == 1` trail dust
//! (`:55473-55481`, no `numUpdates` field in the port), the 639/640 `Kill()`
//! dust `Next(5, 10)` (`:73433`), the 638 `Kill()` dust `Next(2, 5)`
//! (`:76123`). None of these draws gates observable state — the dust bodies
//! are the only consumers — so skipping them keeps every state-bearing
//! decision identical; the argument is recorded in ANALYSIS.
//!
//! Other deliberate skips (ANALYSIS "The Moon Lord projectile family"):
//! - Dust, light, sound, `netUpdate` — no such layers in the pool.
//! - The 638's `base.wet = false` (`:16786-16789`) — no wet state.
//! - The 640's `TrailCacheLength` is 20 (`ProjectileID.cs:522`); the port's
//!   history ring is fixed 10 and shifts once per update call (the recorded
//!   trail-density deviation, projectile.rs `shift_trailing_history`). The
//!   all-equal kill check reads the port's 10-entry ring.

use glam::Vec2;
use tr_core::World;

use crate::projectile::{
    MOONLORD_ARROW, MOONLORD_ARROW_TRAIL, MOONLORD_BULLET, Projectile,
};

/// The moon-lord drop-shot kind set this module owns. Disjoint from the
/// ai001 A/B/C kind lists and from every other aiStyle-1 rider.
pub(crate) const MOONLORD_SHOT_KINDS: [u16; 3] = [
    MOONLORD_BULLET,      // 638
    MOONLORD_ARROW,       // 639
    MOONLORD_ARROW_TRAIL, // 640
];

/// Whether this module owns the kind's aiStyle-1 arm.
pub(crate) fn covers(kind: u16) -> bool {
    MOONLORD_SHOT_KINDS.contains(&kind)
}

/// The `flag3` skip switch (`Projectile.cs:55947-56032`): 638 (`:56018`)
/// and 639 (`:56019`) rest their counters — the 639's `ai[0]` holds the
/// latched spawn velocity X, so counting would corrupt the trail spawn.
/// 640 is NOT on the C# list — it counts like an arrow.
fn counts_flight(kind: u16) -> bool {
    kind == MOONLORD_ARROW_TRAIL
}

/// The shared flight tail — the port's `ai_style1_flight` stand-in
/// (projectile.rs:5244-5267, the ai001_b `finish_flight` replication): the
/// ±16 Y clamp (the port convention; the C# clamps the upper bound only for
/// non-1006 kinds, `:57760-57767`), the default rotation (`:57750`
/// `atan2(vy, vx) + 1.57f`) and the kill-on-tile integration (the
/// `HandleMovement` default arm `:19849-19853`: one last step into the tile,
/// then `Kill()`). `kill_time_left` captures the C# `Kill()`'s `num`
/// (`:70649`) for the 639's death-hook trail spawn.
fn finish_flight(p: &mut Projectile, world: &World) {
    p.vel.y = p.vel.y.clamp(-16.0, 16.0);
    p.rot = p.vel.y.atan2(p.vel.x) + 1.57;
    if p.tile_collide {
        let nx = p.pos.x + p.vel.x;
        let ny = p.pos.y + p.vel.y;
        let hit_x = crate::player::collides_rect(world, nx, p.pos.y, p.w, p.h);
        let hit_y = crate::player::collides_rect(world, p.pos.x, ny, p.w, p.h);
        if hit_x || hit_y {
            p.kill_time_left = p.time_left;
            p.active = false;
            p.time_left = 0;
        } else {
            // S1-P-i57 — the `ai_style1_flight` swap rides here too (the
            // replica stays byte-identical): `:20441`'s `wet ? wetVelocity :
            // velocity` via `wet_velocity` (the `:17612-17655` drag wet).
            p.pos += crate::projectile_wet::wet_velocity(p, world);
        }
    } else {
        p.pos += p.vel;
    }
}

/// `AI_001`'s moon-lord-shot body for one slot, in the C#'s own block order.
/// Every early `Kill(); return` is the port's death convention
/// (`active = false`, `time_left = 0`) plus the `kill_time_left` capture.
pub(crate) fn step(p: &mut Projectile, world: &World) {
    // ---- 1. The hook group above the counter ------------------------------
    // `:55864-55899` — the Moonlord Bullet (638): the speed-scaled alpha
    // fade `alpha -= (byte)((double)velocity.Length() * 0.3)` (the double
    // product truncated toward zero by the byte cast — < 256 for every
    // reachable speed, so the i32 cast is exact), then the velocity-offset
    // hitbox NPC scan and its `Next(15, 31)` dust-229 burst
    // (`:55875-55898`, SKIPPED — dust-only, the draw goes with the layer).
    if p.kind == MOONLORD_BULLET {
        let speed = p.vel.length();
        if p.alpha > 0 {
            p.alpha -= (speed as f64 * 0.3) as i32;
        }
        if p.alpha < 0 {
            p.alpha = 0;
        }
    }
    // `:55415-55429` — the Moonlord Arrow (639): the one-shot latch (both
    // localAI zero → localAI[0..1] = Centre, ai[0..1] = velocity — the
    // Kill() trail spawn reads them back), then the alpha fade 25.
    if p.kind == MOONLORD_ARROW {
        if p.local_ai[0] == 0.0 && p.local_ai[1] == 0.0 {
            p.local_ai[0] = p.pos.x + p.w * 0.5;
            p.local_ai[1] = p.pos.y + p.h * 0.5;
            p.ai[0] = p.vel.x;
            p.ai[1] = p.vel.y;
        }
        p.alpha -= 25;
        if p.alpha < 0 {
            p.alpha = 0;
        }
    }
    // `:55430-55482` — the Moonlord Arrow Trail (640): the alpha fade 25,
    // then the at-rest block — `ai[0] = 0`, the oldPos all-equal scan
    // (`:55441-55447`, indices 1..len against [0]) and the `Kill(); return`
    // when every entry matches. The dust gates (`:55453-55470`: the
    // `Next(extraUpdates)` roll, the `Next(2|6)` at-rest roll, the two
    // `Next(2)` sign picks + `NextDouble` speeds, the `Next(10)` dust-31
    // roll) and the `numUpdates == 1` trail dust (`:55473-55481`) are
    // SKIPPED — dust-only draws, no `numUpdates` field in the port.
    if p.kind == MOONLORD_ARROW_TRAIL {
        p.alpha -= 25;
        if p.alpha < 0 {
            p.alpha = 0;
        }
        if p.vel == Vec2::ZERO {
            p.ai[0] = 0.0;
            let settled = p.old_pos[1..].iter().all(|&q| q == p.old_pos[0]);
            if settled {
                p.kill_time_left = p.time_left;
                p.active = false;
                p.time_left = 0;
                return;
            }
        }
    }

    // ---- 2. The flight counter (`:55945-56041`) ---------------------------
    if counts_flight(p.kind) {
        p.ai[0] += 1.0;
    }

    // ---- 3. The gravity else-if chain -------------------------------------
    // `:57542-57548` — the 639's own arm: from `timeLeft <= MaxUpdates * 45
    // - 14` (= 2 * 45 - 14 = 76, the row's `MaxUpdates = 2`, `:6881`) the
    // +0.1 gravity. No counter pin (the ai[0] holds the latched velocity).
    // The 638 rests its counter at 0 and never reaches the generic arm.
    if p.kind == MOONLORD_ARROW {
        if p.time_left <= ((1 + p.extra_updates) as i32) * 45 - 14 {
            p.vel.y += 0.1;
        }
    } else if p.kind == MOONLORD_ARROW_TRAIL && p.ai[0] >= 15.0 {
        // The generic arm (`:57568-57579`): the counter pinned at 15, the
        // +0.1 gravity (flag4's −0.1 is the 1006-only reverse — no kind
        // here sets it). While the trail rests, the at-rest hook resets
        // ai[0] to 0 each sub-step BEFORE this counter runs (→ 1), so the
        // gravity never fires on a stuck trail — it waits for the ring
        // settle and dies, exactly the C#'s shape.
        p.ai[0] = 15.0;
        p.vel.y += 0.1;
    }

    // ---- 4. The rotation tail + clamp (AI_001) then movement --------------
    if p.kind == MOONLORD_ARROW_TRAIL {
        // The ±16 clamp (`:57760-57767`, the port convention) and the 640's
        // own rotation tail (`:57737-57743`): the flight angle + π/2 ONLY
        // while moving — both run inside AI_001, BEFORE HandleMovement
        // (so the stick tick still records the flight heading).
        p.vel.y = p.vel.y.clamp(-16.0, 16.0);
        if p.vel != Vec2::ZERO {
            p.rot = p.vel.y.atan2(p.vel.x) + std::f32::consts::FRAC_PI_2;
        }
        // `HandleMovement`'s 466/580/640 arm (`:18308-18316`): on the first
        // blocked move (`localAI[1] < 1`) the counter jumps to 2, the
        // projectile takes ONE last full-velocity step and the velocity
        // zeroes (the stick). While `localAI[1] > 0` the `:17590` gate
        // skips `TileCollision` entirely — the resolved velocity never
        // differs, so the arm cannot re-fire and `UpdatePosition`
        // (`:19858`) integrates freely. S1-P-i57: the stuck arm's integrate
        // rides `wet_velocity` — the `:17590` 640-latch gate exempts it from
        // the drag (the un-scaled capture, wet AND dry byte-identical).
        if p.local_ai[1] > 0.0 {
            p.pos += crate::projectile_wet::wet_velocity(p, world);
        } else {
            let last = p.vel;
            let hit_x =
                crate::player::collides_rect(world, p.pos.x + last.x, p.pos.y, p.w, p.h);
            let hit_y =
                crate::player::collides_rect(world, p.pos.x, p.pos.y + last.y, p.w, p.h);
            if hit_x || hit_y {
                // `:18310-18314` — `localAI[1] += 2f; position += velocity;
                // velocity = Vector2.Zero` (the PRE-collision velocity).
                // S1-P-i57: the last step rides `wet_velocity` (`:20441`'s
                // wet swap — the `:17612-17655` drag while wet; the latch
                // has not fired yet, the drag applies).
                p.local_ai[1] += 2.0;
                p.pos += crate::projectile_wet::wet_velocity(p, world);
                p.vel = Vec2::ZERO;
            } else {
                // S1-P-i57 — `:20441`'s wet swap on the unblocked move.
                p.pos += crate::projectile_wet::wet_velocity(p, world);
            }
        }
    } else {
        // 638/639 keep the default rotation (`:57750`) the shared tail
        // writes; the movement is the kill-on-tile default arm.
        finish_flight(p, world);
    }
}

// ---------------------------------------------------------------------------
// Witness tests-as-code — WRITTEN NOT RUN (code-only mode, S1-D-4).
// Verified by re-reading the cited C# and hand arithmetic; run them with
// `cargo test -p tr-app projectile_ai_moonlord` once a compiler is available.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::{proj_data, ProjectilePool, YoyoInputs, FlailInputs};

    fn air() -> World {
        tr_core::World::new("t".into(), 200, 200)
    }

    /// A rowed moon-lord shot at (1600, 800) — tile (100, 50), open air in
    /// the 200×200 test world — flying right at 10 px/tick (the ai001_b
    /// `shot` fixture shape: field-by-field on `Default`, E0451).
    fn shot(kind: u16) -> Projectile {
        let d = proj_data(kind).expect("rowed kind");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(1600.0, 800.0);
        p.vel = Vec2::new(10.0, 0.0);
        p.w = d.w;
        p.h = d.h;
        p.time_left = d.time_left;
        p.extra_updates = d.extra_updates;
        p.tile_collide = d.tile_collide;
        p.alpha = d.alpha;
        p.scale = d.scale;
        p.penetrate = d.penetrate;
        p.max_penetrate = d.penetrate;
        p.arrow = d.arrow;
        p
    }

    /// A/B: the gate claims ONLY the three drop shots — never the ai001
    /// A/B/C kinds, the riders or the phantasmal family (the 462/622 ride
    /// ai001_b, the 452/454/455/456 their own styles).
    #[test]
    fn the_gate_is_disjoint_from_the_other_arms() {
        for k in [1u16, 14, 81, 98, 184, 259, 270, 325, 389, 436, 438, 449,
                  462, 581, 601, 622, 680, 686, 962, 980, 110, 275, 276,
                  634, 635, 645, 662, 675, 685, 20, 88, 355, 660, 837] {
            assert!(!covers(k), "kind {k} belongs to another arm");
        }
        for k in MOONLORD_SHOT_KINDS {
            assert!(covers(k));
        }
    }

    /// The three rows are the verbatim `SetDefaults` slices
    /// (`Projectile.cs:6858-6872` / `:6873-6888` / `:6889-6903`).
    #[test]
    fn the_rows_are_verbatim() {
        let b = proj_data(MOONLORD_BULLET).unwrap();
        assert_eq!((b.w, b.h), (4.0, 4.0));
        assert_eq!(b.ai_style, 1);
        assert_eq!(b.penetrate, -1);
        assert_eq!(b.time_left, 600);
        assert_eq!(b.extra_updates, 5);
        assert!(b.tile_collide);
        assert!(b.ignore_water);
        assert!(b.uses_local_immunity);
        assert_eq!(b.local_hit_cooldown, -1);
        assert_eq!(b.alpha, 255);
        let a = proj_data(MOONLORD_ARROW).unwrap();
        assert_eq!((a.w, a.h), (10.0, 10.0));
        assert!(a.arrow, "`this.arrow = true` (`:6875`)");
        assert_eq!(a.penetrate, 4);
        // `MaxUpdates = 2` (`:6881`) → extraUpdates 1; `timeLeft =
        // MaxUpdates * 45` = 90 (`:6882`).
        assert_eq!(a.extra_updates, 1);
        assert_eq!(a.time_left, 90);
        assert_eq!(a.alpha, 255);
        assert!(a.uses_local_immunity);
        assert_eq!(a.local_hit_cooldown, -1);
        let t = proj_data(MOONLORD_ARROW_TRAIL).unwrap();
        assert_eq!((t.w, t.h), (10.0, 10.0));
        assert!(!t.arrow);
        assert_eq!(t.penetrate, 4);
        assert_eq!(t.extra_updates, 2, "`MaxUpdates = 3` (`:6896`)");
        assert_eq!(t.time_left, 90);
        assert_eq!(t.alpha, 255);
        // The TrailingMode-0 ring entries (`ProjectileID.cs:521`).
        for k in MOONLORD_SHOT_KINDS {
            assert_eq!(crate::projectile::trailing_mode(k), 0, "kind {k}");
        }
    }

    /// The flag3 membership (`:56018`/`:56019`): the 638/639 rest, the 640
    /// counts. The 639's ai[0] must hold the latched velocity X, not a
    /// counter — one tick at (10, 0) leaves ai = [10, 0].
    #[test]
    fn flag3_holds_the_bullet_and_arrow_counters() {
        let w = air();
        let mut b = shot(MOONLORD_BULLET);
        step(&mut b, &w);
        assert_eq!(b.ai[0], 0.0, "the 638 rests (`:56018`)");
        let mut a = shot(MOONLORD_ARROW);
        step(&mut a, &w);
        assert_eq!(a.ai[0], 10.0, "the latched vx (`:55421`)");
        assert_eq!(a.ai[1], 0.0);
        assert_eq!(a.local_ai[0], 1605.0, "the latched Centre.X (`:55419`)");
        assert_eq!(a.local_ai[1], 805.0);
        let mut t = shot(MOONLORD_ARROW_TRAIL);
        step(&mut t, &w);
        assert_eq!(t.ai[0], 1.0, "the 640 counts (not on the C# list)");
    }

    /// 638 (`:55866-55874`): the byte-truncated speed fade — speed 10 →
    /// `(byte)(10.0 * 0.3)` = 3 per tick; 255 → 252 after one tick.
    #[test]
    fn the_bullet_fades_by_the_truncated_speed_product() {
        let w = air();
        let mut p = shot(MOONLORD_BULLET);
        step(&mut p, &w);
        assert_eq!(p.alpha, 252);
        // Speed 0 → no fade; the alpha floors at 0, never negative.
        p.vel = Vec2::ZERO;
        step(&mut p, &w);
        assert_eq!(p.alpha, 252);
        p.vel = Vec2::new(10.0, 0.0);
        p.alpha = 2;
        step(&mut p, &w);
        assert_eq!(p.alpha, 0);
    }

    /// 639 (`:57542-57548`): the gravity starts at `timeLeft <= 76` — at
    /// the 90 spawn it flies straight; the latch fires once only.
    #[test]
    fn the_arrow_drops_only_after_fourteen_sub_steps() {
        let w = air();
        let mut p = shot(MOONLORD_ARROW);
        step(&mut p, &w); // timeLeft 90 > 76 — no gravity
        assert_eq!(p.vel.y, 0.0);
        assert_eq!(p.alpha, 230, "the fade 25 (`:55424`)");
        p.time_left = 76;
        p.vel = Vec2::new(10.0, 0.0);
        step(&mut p, &w);
        assert_eq!(p.vel.y, 0.1, "`timeLeft <= 76` → +0.1 (`:57546`)");
        // The latch does not re-fire (localAI non-zero).
        assert_eq!(p.local_ai[0], 1605.0);
    }

    /// 640 (`:18308-18316`): the first blocked move sticks — localAI[1]
    /// jumps to 2, one last full-velocity step, velocity zeroed. The
    /// subsequent steps integrate freely (the `:17590` collision skip).
    #[test]
    fn the_trail_sticks_on_the_first_blocked_move() {
        let mut w = air();
        // A wall at tile x = 102 (px 1632): the 10-wide trail at 1620 flies
        // 10 px into it.
        for y in 40..60 {
            let i = w.index(102, y);
            w.tiles[i].kind = 1;
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        let mut p = shot(MOONLORD_ARROW_TRAIL);
        p.pos = Vec2::new(1620.0, 800.0);
        step(&mut p, &w);
        assert_eq!(p.local_ai[1], 2.0, "the stick latch (`:18312`)");
        assert_eq!(p.pos, Vec2::new(1630.0, 800.0), "the last step (`:18313`)");
        assert_eq!(p.vel, Vec2::ZERO, "the velocity zeroes (`:18314`)");
        assert!(p.active);
        // At rest: the hook resets ai[0] then the counter increments (→ 1,
        // never 15 — a stuck trail takes no gravity); the ring is not yet
        // all-equal — alive. (The pool shifts the ring per update; the
        // direct-step fixture records the newest entry by hand.)
        p.old_pos[0] = p.pos;
        step(&mut p, &w);
        assert_eq!(p.ai[0], 1.0);
        assert!(p.active);
    }

    /// 640 (`:55437-55451`): with every ring entry equal the at-rest trail
    /// dies; the kill captures timeLeft for the death hook's convention.
    #[test]
    fn the_trail_dies_when_the_ring_settles() {
        let w = air();
        let mut p = shot(MOONLORD_ARROW_TRAIL);
        p.vel = Vec2::ZERO;
        p.local_ai[1] = 2.0;
        p.old_pos = [Vec2::new(1600.0, 800.0); 10];
        step(&mut p, &w);
        assert!(!p.active, "the all-equal Kill (`:55450`)");
        assert_eq!(p.kill_time_left, 90, "the Kill() `num` capture (`:70649`)");
    }

    /// The pool seam: the 640's `ai[1]` countdown (`:16679-16683`) skips
    /// whole sub-steps — a trail spawned with ai[1] = 3 runs NO AI and pays
    /// NO timeLeft until the counter drains (3 sub-steps = 1 update at
    /// extraUpdates 2).
    #[test]
    fn the_trail_waits_out_its_spawn_delay() {
        let mut w = air();
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai_owned(
                MOONLORD_ARROW_TRAIL, 0, 1600.0, 800.0, 10.0, 0.0, 10, 0.0,
                0.0, 3.0,
            )
            .unwrap();
        pool.update(
            &mut w, &mut [], None, None, None,
            YoyoInputs::default(), FlailInputs::default(),
            &mut tr_core::tile_light::FastRandom::new(7), 0, false,
        );
        let p = &pool.slots()[slot];
        assert_eq!(p.ai[1], 0.0, "3 sub-steps drained the counter");
        assert_eq!(p.time_left, 90, "no lifetime paid while waiting");
        assert_eq!(p.pos, Vec2::new(1600.0, 800.0), "no movement while waiting");
        // The next update runs the AI: the trail moves and sticks or flies.
        pool.update(
            &mut w, &mut [], None, None, None,
            YoyoInputs::default(), FlailInputs::default(),
            &mut tr_core::tile_light::FastRandom::new(7), 0, false,
        );
        assert!(pool.slots()[slot].pos.x > 1600.0, "the AI runs after the wait");
    }

    /// The 639's death hook (`:73431-73452`): a tile-killed arrow spawns
    /// its 640 ghost at the latched centre with the latched velocity and
    /// `ai1 = timeLeft + 1` (+1 more when the free slot sits above the
    /// update loop index, `:73446-73449`).
    #[test]
    fn the_arrow_death_spawns_its_trail() {
        let mut w = air();
        for y in 40..60 {
            let i = w.index(102, y);
            w.tiles[i].kind = 1;
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai_owned(
                MOONLORD_ARROW, 0, 1620.0, 800.0, 10.0, 0.0, 25, 3.0,
                0.0, 0.0,
            )
            .unwrap();
        pool.update(
            &mut w, &mut [], None, None, None,
            YoyoInputs::default(), FlailInputs::default(),
            &mut tr_core::tile_light::FastRandom::new(7), 0, false,
        );
        assert!(!pool.slots()[slot].active, "the arrow dies on the wall");
        let trail = pool
            .slots()
            .iter()
            .find(|p| p.active && p.kind == MOONLORD_ARROW_TRAIL)
            .expect("the Kill() trail spawn (`:73451`)");
        assert_eq!(trail.pos, Vec2::new(1625.0, 805.0), "the latched centre");
        assert_eq!(trail.vel, Vec2::new(10.0, 0.0), "the latched velocity");
        assert_eq!(trail.damage, 25);
        assert_eq!(trail.knock_back, 3.0);
        // The wall sits 12 px ahead of the 10-wide arrow, so the FIRST
        // sub-step's move is already blocked — the movement kill runs
        // BEFORE that sub-step's lifetime block, so `num` = 90 (the
        // pre-zeroing timeLeft). The free slot (1) sits above the loop
        // index (0) → ai1 = 90 + 1 + 1 = 92. The ghost sits ABOVE the loop
        // index, so the SAME update reaches it and drains 3 sub-steps of
        // the counter (extraUpdates 2) — exactly the C#'s same-tick
        // double-update the +1 bump compensates for.
        assert_eq!(trail.ai[1], 89.0, "`num + 1` + the bump − 3 same-tick sub-steps");
    }

    /// The on-hit decay (`:14177-14180`): every landed hit multiplies the
    /// shot's damage by 0.96 with the C# `(int)` truncation — 100 → 96.
    #[test]
    fn the_landed_hit_decays_the_damage() {
        use crate::npc::{Npc, SlimeVariant};
        let mut w = air();
        let mut pool = ProjectilePool::new();
        let mut npc = Npc::spawn_slime_sky(1600.0, 800.0, SlimeVariant::Blue);
        npc.defense_override = Some(0);
        let slot = pool
            .spawn_with_data_ai_owned(
                MOONLORD_BULLET, 0, 1598.0, 798.0, 0.0, 0.0, 100, 0.0,
                0.0, 0.0,
            )
            .unwrap();
        pool.update(
            &mut w, std::slice::from_mut(&mut npc), None, None, None,
            YoyoInputs::default(), FlailInputs::default(),
            &mut tr_core::tile_light::FastRandom::new(7), 0, false,
        );
        assert!(npc.hp < npc.max_hp, "the bullet lands");
        assert_eq!(pool.slots()[slot].damage, 96, "`(int)(100 * 0.96)`");
    }
}
