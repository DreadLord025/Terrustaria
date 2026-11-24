//! W4-f42 (sandstorm second half) — the Desert Djinn's curse projectile.
//!
//! `ProjectileID.DesertDjinnCurse` (596): the Djinn's `% 30` volley
//! (`NPC.cs:21117-21162`) spawns it at a searched tile centre with
//! `ai0 = this.target`; the flight is `AI()`'s aiStyle-107 arm
//! (`Projectile.cs:33553-33678`, the 596 branch `:33582-33651` + the shared
//! homing tail `:33653-33677`), and the death burst is `Kill()`'s 596 arm
//! (`Projectile.cs:72470-72501`).
//!
//! Port notes (the projectile-arm dust convention, `projectile_ai026_a`
//! module doc): the dust VISUALS and their pool-gated `NewDust` internals
//! are skipped — the projectile update holds no `DustPool` — but every
//! EXPLICIT arm-level `Main.rand` draw stays on the caller's
//! `UpdateProjectiles` stream in the C#'s order:
//! * the three `Next(3)` orbit gates (`:33619`) — a non-zero roll RETURNS
//!   from the whole AI (the homing tail included), so the gates are
//!   behaviour, not just census;
//! * the `timeLeft < 4` burst's ten `SelectRandom(27, 6)` picks
//!   (`:33644`, one `Next(2)` each — `Utils.cs:2459-2462`);
//! * the Kill arm's `RotatedByRandom(π)` pairs (two `NextDouble` each,
//!   `Utils.cs:1697-1700`) + the first two loops' radius `NextDouble`
//!   (`:72486`/`:72491`; the third loop `:72498` has NO radius draw).
//!
//! `num805 = ai[1] / 180f * 2π` (`:33616`) is computed and never read in
//! the 596 branch — dead, dropped with this citation.

use glam::Vec2;
use tr_core::World;
use tr_core::loot::DropRng;

use crate::player::Player;
use crate::projectile::{DESERT_DJINN_CURSE, Projectile};

/// `AI()`'s aiStyle-107 arm for the 596 curse (`Projectile.cs:33582-33677`),
/// one sub-step. `player` is the session player (the C# reads
/// `Main.player[(int)ai[0]]`; the port's single player is slot 0 — any
/// other index reads an inactive slot, the orphan branch, the
/// `ai_sharknado_bolt` single-player fold).
pub fn curse_ai(p: &mut Projectile, world: &World, player: Option<&Player>, rng: &mut dyn DropRng) {
    if p.kind != DESERT_DJINN_CURSE {
        return;
    }
    // `:33555-33557` + `:33584-33585` — the 596 overrides: inertia 10,
    // thrust 7.5, the homing dead zone 40.
    let num801 = 10.0f32;
    let num802 = 7.5f32;
    let num803 = 40.0f32;
    // `:33586-33597` — the fade-in: −25/tick while the life stands above
    // 30; a solid overlap pins the alpha at 128 (the curse hides in
    // blocks until it clears them).
    if p.time_left > 30 && p.alpha > 0 {
        p.alpha -= 25;
    }
    if p.time_left > 30
        && p.alpha < 128
        && crate::player::solid_collision(world, p.pos.x, p.pos.y, p.w, p.h)
    {
        p.alpha = 128;
    }
    if p.alpha < 0 {
        p.alpha = 0;
    }
    // `:33598-33605` — the 4-frame strip at 5 ticks per frame.
    p.frame_counter += 1;
    if p.frame_counter > 4 {
        p.frame_counter = 0;
        p.frame += 1;
        if p.frame >= 4 {
            p.frame = 0;
        }
    }
    // `:33606-33614` — the orbit dust's scale ladder (the dust bodies are
    // the documented skip; the ladder feeds nothing else in the port).
    let mut num804 = 0.5f32;
    if p.time_left < 120 {
        num804 = 1.1;
    }
    if p.time_left < 60 {
        num804 = 1.6;
    }
    let _ = num804;
    // `:33615` — the spin counter (the C#'s `num805` angle is dead code).
    p.ai[1] += 1.0;
    // `:33617-33630` — the three orbit dusts: each gate draws `Next(3)`;
    // a non-zero roll RETURNS from the whole AI — the homing tail below
    // never runs that sub-step (the C#'s own early exit).
    for _ in 0..3 {
        if rng.next_range(0, 3) != 0 {
            return;
        }
        // `Dust.NewDust(Center, 0, 0, 27, 0, -2)` + the orbit placement /
        // noGravity / DirectionFrom velocity / scale / fadeIn 0.5 /
        // alpha 200 (`:33623-33629`) — the dust-body skip (module doc).
    }
    // `:33631-33651` — the last three ticks arm the burst: the 60×60
    // recentre (`position = Center; width = height = 60; Center =
    // position` — the new top-left is the OLD centre − 30), damage 30
    // (expert 22), ten 27/6 dusts (one `Next(2)` pick each).
    if p.time_left < 4 {
        let expert = player.is_some_and(|pl| pl.buffs.expert_mode);
        let num807 = if expert { 22 } else { 30 };
        let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
        p.w = 60.0;
        p.h = 60.0;
        p.pos = center - Vec2::splat(30.0);
        p.damage = num807;
        for _ in 0..10 {
            // `Utils.SelectRandom<int>(Main.rand, 27, 6)` (`:33644`).
            let _pick = rng.next_range(0, 2);
            // The dust body + the `DirectionTo * 3` velocity arm
            // (`:33645-33649`) — the dust-body skip (module doc).
        }
    }
    // `:33653-33677` — the shared homing tail.
    let num809 = p.ai[0] as i32;
    let live_target = num809 == 0 && player.is_some_and(|pl| !pl.dead);
    if live_target {
        let pl = player.unwrap();
        let (pcx, pcy) = pl.center();
        let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
        let target = Vec2::new(pcx, pcy);
        // `base.Distance(player.Center)` — the centre-to-centre length.
        if center.distance(target) > num803 {
            let mut dir = target - center;
            if dir.x.is_nan() || dir.y.is_nan() {
                dir = Vec2::Y;
            } else {
                dir = dir.normalize_or_zero();
                // `DirectionTo` normalises; a zero delta stays zero in
                // XNA (`SafeNormalize` is NOT used here — `Vector2.
                // Normalize` of zero yields NaN, caught by HasNaNs
                // above). The port's normalize_or_zero yields zero; the
                // zero-distance case cannot pass the >40 gate anyway.
            }
            p.vel = (p.vel * (num801 - 1.0) + dir * num802) / num801;
        }
    } else {
        // `:33666-33677` — the orphan: the life caps at 30, ai[0] latches
        // −1 once (the netUpdate rides the slot's flag).
        if p.time_left > 30 {
            p.time_left = 30;
        }
        if p.ai[0] != -1.0 {
            p.ai[0] = -1.0;
            p.net_update = true;
        }
    }
}

/// `Kill()`'s 596 arm (`Projectile.cs:72470-72501`): the 60×60 recentre +
/// damage 30/22 (the caller applies the boxed player hit and the Item14
/// cue), then the three dust loops' EXPLICIT draw census — 4×3 + 20×3 +
/// 10×2 = 92 `NextDouble` draws (the `RotatedByRandom(π)` pairs + the
/// first two loops' radius rolls; the third loop's placement
/// `UnitX.RotatedByRandom(π).RotatedBy(velocity.ToRotation()) * width/2`
/// carries no radius draw). The dust bodies are the module-doc skip.
pub fn curse_kill_draws(rng: &mut dyn DropRng) {
    // `:72483-72487` — the four alpha-100 scale-1.5 dust-31s.
    for _ in 0..4 {
        let _ = rng.next_double(); // RotatedByRandom's first roll
        let _ = rng.next_double(); // …second roll
        let _ = rng.next_double(); // `NextDouble() * width / 2` radius
    }
    // `:72488-72494` — the twenty noGravity scale-2.5 dust-27s (×2 vel).
    for _ in 0..20 {
        let _ = rng.next_double();
        let _ = rng.next_double();
        let _ = rng.next_double();
    }
    // `:72495-72501` — the ten noGravity scale-1.5 dust-31s (×2 vel), the
    // velocity-rotated ring — two rolls each, NO radius draw.
    for _ in 0..10 {
        let _ = rng.next_double();
        let _ = rng.next_double();
    }
}

/// The Kill arm's resize + damage (`Projectile.cs:72472-72480`): the
/// 60×60 recentre keeps the old centre, damage 30 (expert 22). The
/// caller runs the boxed `Damage()` hit afterwards.
pub fn curse_kill_resize(p: &mut Projectile, expert: bool) {
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    p.w = 60.0;
    p.h = 60.0;
    p.pos = center - Vec2::splat(30.0);
    p.damage = if expert { 22 } else { 30 };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::ProjectilePool;
    use tr_core::UnifiedRandom;

    fn air() -> World {
        World::new("curse".into(), 200, 120)
    }

    fn curse(pool: &mut ProjectilePool, x: f32, y: f32, target: f32) -> usize {
        // The Djinn's call site (`NPC.cs:21155`): damage 0, knockback 1,
        // ai0 = the NPC's target; the hostile seam folds the owner to 255.
        pool.spawn_hostile_ai(
            DESERT_DJINN_CURSE,
            Vec2::new(x - 4.0, y - 4.0),
            Vec2::ZERO,
            0,
            1.0,
            target,
            0.0,
        )
        .unwrap()
    }

    /// A seed whose first three `Next(3)` rolls are all 0 (the orbit
    /// gates pass, the homing tail runs). Found by search — the witness
    /// pins the gate semantics, not a magic seed.
    fn passing_seed() -> i32 {
        for seed in 1..10000 {
            let mut r = UnifiedRandom::new(seed);
            if r.next_range(0, 3) == 0 && r.next_range(0, 3) == 0 && r.next_range(0, 3) == 0 {
                return seed;
            }
        }
        panic!("no passing seed");
    }

    /// A seed whose FIRST `Next(3)` roll is non-zero (the early return).
    fn failing_seed() -> i32 {
        for seed in 1..10000 {
            let mut r = UnifiedRandom::new(seed);
            if r.next_range(0, 3) != 0 {
                return seed;
            }
        }
        panic!("no failing seed");
    }

    #[test]
    fn the_curse_row_is_the_csharp_setdefaults() {
        // `Projectile.cs:6450-6460`.
        let d = crate::projectile::proj_data(DESERT_DJINN_CURSE).unwrap();
        assert_eq!((d.w, d.h), (8.0, 8.0));
        assert_eq!(d.ai_style, 107);
        assert_eq!(d.time_left, 180);
        assert_eq!(d.alpha, 255);
        assert!(d.ignore_water);
        assert!(!d.tile_collide);
        assert_eq!(d.penetrate, 1);
    }

    #[test]
    fn the_alpha_fades_25_per_tick_and_the_frame_cycles_at_5() {
        let w = air();
        let mut pool = ProjectilePool::new();
        let i = curse(&mut pool, 1600.0, 800.0, 0.0);
        let mut rng = UnifiedRandom::new(passing_seed());
        let pl = Player::spawn(&w);
        // Tick 1: alpha 255 → 230, frameCounter 1.
        curse_ai(&mut pool.slots_mut()[i], &w, Some(&pl), &mut rng);
        assert_eq!(pool.slots()[i].alpha, 230);
        assert_eq!(pool.slots()[i].frame_counter, 1);
        // Five more ticks: the counter wraps at >4 and the frame advances
        // (ticks 2-5 reach 5, tick 6 rolls 6 > 4 → counter 0, frame 1).
        for _ in 0..5 {
            let mut r = UnifiedRandom::new(passing_seed());
            curse_ai(&mut pool.slots_mut()[i], &w, Some(&pl), &mut r);
        }
        assert_eq!(pool.slots()[i].frame_counter, 0);
        assert_eq!(pool.slots()[i].frame, 1);
        assert_eq!(pool.slots()[i].alpha, 230 - 25 * 5);
    }

    #[test]
    fn a_nonzero_gate_roll_returns_before_the_homing_tail() {
        // `:33619-33622` — the early `return` skips the homing blend.
        let w = air();
        let mut pool = ProjectilePool::new();
        // Far from the player: a passing tick would blend the velocity.
        let i = curse(&mut pool, 1600.0, 800.0, 0.0);
        let mut pl = Player::spawn(&w);
        pl.x = 100.0;
        pl.y = 100.0;
        let mut rng = UnifiedRandom::new(failing_seed());
        curse_ai(&mut pool.slots_mut()[i], &w, Some(&pl), &mut rng);
        assert_eq!(pool.slots()[i].vel, Vec2::ZERO, "the gate returned early");
        // The spin counter still advanced (it precedes the gates).
        assert_eq!(pool.slots()[i].ai[1], 1.0);
    }

    #[test]
    fn the_homing_blends_nine_parts_inertia_one_part_thrust() {
        // `:33656-33664` — outside 40 px the velocity lerps 9/10 + 1/10.
        let w = air();
        let mut pool = ProjectilePool::new();
        let i = curse(&mut pool, 1600.0, 800.0, 0.0);
        let mut pl = Player::spawn(&w);
        pl.x = 100.0;
        pl.y = 800.0 - 21.0; // the player centre sits left of the curse
        let mut rng = UnifiedRandom::new(passing_seed());
        curse_ai(&mut pool.slots_mut()[i], &w, Some(&pl), &mut rng);
        let p = &pool.slots()[i];
        let center = Vec2::new(p.pos.x + 4.0, p.pos.y + 4.0);
        let (pcx, pcy) = pl.center();
        let dir = (Vec2::new(pcx, pcy) - center).normalize();
        assert_eq!(p.vel, dir * 7.5 / 10.0);
    }

    #[test]
    fn the_orphan_caps_the_life_and_latches_ai0() {
        // `:33666-33677` — no live target: timeLeft caps at 30, ai[0] −1.
        let w = air();
        let mut pool = ProjectilePool::new();
        let i = curse(&mut pool, 1600.0, 800.0, 3.0); // target slot 3: inactive
        let mut rng = UnifiedRandom::new(passing_seed());
        let pl = Player::spawn(&w);
        curse_ai(&mut pool.slots_mut()[i], &w, Some(&pl), &mut rng);
        assert_eq!(pool.slots()[i].time_left, 30);
        assert_eq!(pool.slots()[i].ai[0], -1.0);
        assert!(pool.slots()[i].net_update);
    }

    #[test]
    fn the_last_ticks_arm_the_60_box_and_the_expert_damage() {
        // `:33631-33641` — timeLeft < 4: the recentred 60×60 box, damage
        // 30 classic / 22 expert, ten Next(2) picks.
        let w = air();
        let mut pool = ProjectilePool::new();
        let i = curse(&mut pool, 1600.0, 800.0, 0.0);
        pool.slots_mut()[i].time_left = 3;
        let mut pl = Player::spawn(&w);
        let mut rng = UnifiedRandom::new(passing_seed());
        curse_ai(&mut pool.slots_mut()[i], &w, Some(&pl), &mut rng);
        let p = &pool.slots()[i];
        assert_eq!((p.w, p.h), (60.0, 60.0));
        assert_eq!(p.pos, Vec2::new(1600.0 - 30.0, 800.0 - 30.0));
        assert_eq!(p.damage, 30);
        // Expert: 22.
        let j = curse(&mut pool, 1600.0, 800.0, 0.0);
        pool.slots_mut()[j].time_left = 3;
        pl.buffs.expert_mode = true;
        let mut rng = UnifiedRandom::new(passing_seed());
        curse_ai(&mut pool.slots_mut()[j], &w, Some(&pl), &mut rng);
        assert_eq!(pool.slots()[j].damage, 22);
    }

    #[test]
    fn the_kill_census_is_92_double_draws() {
        // `:72483-72501` — 4×3 + 20×3 + 10×2 = 92 NextDouble draws.
        let mut a = UnifiedRandom::new(7);
        let mut b = UnifiedRandom::new(7);
        curse_kill_draws(&mut a);
        for _ in 0..92 {
            let _ = b.next_double();
        }
        // The streams agree iff the census consumed exactly 92 draws.
        assert_eq!(a.next_double(), b.next_double());
    }

    #[test]
    fn the_kill_resize_recentres_the_60_box() {
        let mut pool = ProjectilePool::new();
        let i = curse(&mut pool, 1600.0, 800.0, 0.0);
        curse_kill_resize(&mut pool.slots_mut()[i], false);
        let p = &pool.slots()[i];
        assert_eq!((p.w, p.h), (60.0, 60.0));
        assert_eq!(p.pos, Vec2::new(1570.0, 770.0));
        assert_eq!(p.damage, 30);
    }
}
