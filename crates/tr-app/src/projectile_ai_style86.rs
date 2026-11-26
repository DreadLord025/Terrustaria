//! W4-d9 aiStyle 86 — the CultistBossIceMist 464 (`AI()`'s inline arm
//! `Projectile.cs:31827-31900`). Census: aiStyles 82/83/84/85/88 were armed
//! on main; 86 was a gap (the row + id landed with the W4-d9 census commit).
//!
//! Two forms, split on `ai[1]`:
//! * `ai[1] == 1` the rotating spawner (`:31835-31866`): the alpha ramp
//!   (−10/tick to 130, +10/tick after), the 150-tick kill, the 30-tick child
//!   464 spawn (the mist form, `ai0=0, ai1=0`) at the rotation's unit vector,
//!   and the `PI/30` rotation step. The C# `return`s before any movement —
//!   the spawner hangs stationary.
//! * else the six-spoke expanding mist (`:31867-31899`): the position
//!   unwinds (`position -= velocity`), the alpha ramp (−40/tick to 40,
//!   +3/tick after), the 45-tick kill, and the six spokes' dust at the
//!   `(0,-720).RotatedBy(velocity.ToRotation()) * (ai[0]%45/45)` offset.
//!
//! The first-tick sound latch (`:31829-31833`, Item120) rides `wants_sound`
//! (the pool drains it on the `UpdateProjectiles` stream).
//!
//! The six-spoke hitbox is `Colliding`'s 464 arm (`:16095-16108`): six 30×30
//! boxes around the centre at the expanding spoke offsets, OR'd with the
//! base-rect test (`:15888`). The spawner form falls through to the base
//! rect only.
//!
//! Movement: the mist form's `position -= velocity` inside the AI cancels
//! the dispatch's `kill_step` `position += velocity` (the C# `UpdatePosition`
//! `:20445` adds it back — net zero, the mist hangs at its spawn point while
//! the spokes expand). The spawner form returns before movement; the
//! dispatch skips its `kill_step` (the C# `return` at `:31865`).
//!
//! RNG contract (the `UpdateProjectiles` stream): the mist's per-spoke dust
//! draws two `RandomVector2(-8,8)` floats ×2 dusts ×6 spokes = 24 NextDouble
//! per tick (`:31896`, `Utils.cs:2431-2434`). Dust-only, consumed in order.
//! The spawner form draws nothing.
//!
//! Code-only round (W4-d9): the witness tests below are WRITTEN NOT RUN.

#![allow(dead_code)] // wiring deferred (W4-d9, code-only mode)

use std::f32::consts::PI;

use glam::Vec2;
use tr_core::loot::DropRng;

use crate::projectile::{rotate, Projectile, CULTIST_BOSS_ICE_MIST};

/// `Kill()` (`Projectile.cs:70644-70648`) — the sibling-module deactivation
/// (the `projectile_ai_spear` precedent; `Projectile::kill` is private).
fn kill(p: &mut Projectile) {
    p.kill_time_left = p.time_left;
    p.active = false;
    p.time_left = 0;
}

/// `Entity.Center` for the pool's slots (`pos + size/2`).
fn center(p: &Projectile) -> Vec2 {
    Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5)
}

/// `AI()`'s aiStyle-86 arm (`Projectile.cs:31827-31900`). The child 464
/// spawn rides `p.wants_dart_spawns` (the pool resolves it after the
/// sub-steps). Returns true when the arm ran the spawner form (the dispatch
/// skips the movement step — the C# `return`s at `:31865`).
pub(crate) fn ai_ice_mist(p: &mut Projectile, rng: &mut dyn DropRng) -> bool {
    // `:31829-31833` — the first-tick sound latch (Item120, style 2).
    if p.local_ai[1] == 0.0 {
        p.local_ai[1] = 1.0;
        p.wants_sound = Some((2, 120, p.pos.x, p.pos.y, 0.0, 1.0));
    }
    p.ai[0] += 1.0;
    if p.ai[1] == 1.0 {
        // `:31835-31866` — the rotating spawner.
        if p.ai[0] >= 130.0 {
            p.alpha += 10;
        } else {
            p.alpha -= 10;
        }
        p.alpha = p.alpha.clamp(0, 255);
        if p.ai[0] >= 150.0 {
            kill(p);
            return true;
        }
        // `:31858-31862` — the 30-tick child 464 (ai0=0, ai1=0: the mist
        // form). The C# spawns at the centre with the rotation's unit
        // vector; NewProjectile re-centres the box (`:10876-10877`), so the
        // spawn site subtracts the 60×60 child's half-box.
        if p.ai[0] % 30.0 == 0.0 {
            let dir = Vec2::new(p.rot.cos(), p.rot.sin());
            let c = center(p);
            p.wants_dart_spawns.push((
                CULTIST_BOSS_ICE_MIST,
                c.x - 30.0,
                c.y - 30.0,
                dir.x,
                dir.y,
                p.damage,
                p.knock_back,
                0.0,
                0.0,
                0,
                0.0,
            ));
        }
        p.rot += PI / 30.0;
        // `:31864` — the light (skipped: the ProjLight seam owns lights).
        return true;
    }
    // `:31867-31899` — the six-spoke expanding mist.
    p.pos -= p.vel;
    if p.ai[0] >= 40.0 {
        p.alpha += 3;
    } else {
        p.alpha -= 40;
    }
    p.alpha = p.alpha.clamp(0, 255);
    if p.ai[0] >= 45.0 {
        kill(p);
        return false;
    }
    // `:31889-31899` — the six spokes' dust (skipped bodies, consumed
    // draws: two RandomVector2 floats per dust, two dusts per spoke).
    let spinningpoint = rotate(Vec2::new(0.0, -720.0), p.vel.to_angle()) * (p.ai[0] % 45.0 / 45.0);
    for num706 in 0..6 {
        let _spoke = center(p) + rotate(spinningpoint, num706 as f32 * (PI * 2.0) / 6.0);
        for _ in 0..2 {
            // RandomVector2(-8,8)/2 — two NextDouble draws (Utils.cs:2433).
            let _ = rng.next_float();
            let _ = rng.next_float();
        }
    }
    false
}

/// `Colliding`'s type-464 arm (`Projectile.cs:16095-16108`): the mist form
/// (`ai[1] != 1`) hits via six 30×30 boxes around the centre at the
/// expanding spoke offsets. Returns true when any spoke overlaps the
/// target rect. The spawner form falls through to the base-rect test.
pub(crate) fn ice_mist_colliding(p: &Projectile, target: tr_core::whip::Rect) -> bool {
    if p.kind != CULTIST_BOSS_ICE_MIST || p.ai[1] == 1.0 {
        return false;
    }
    let spinningpoint = rotate(Vec2::new(0.0, -720.0), p.vel.to_angle()) * (p.ai[0] % 45.0 / 45.0);
    let c = center(p);
    for num43 in 0..6 {
        let num44 = num43 as f32 * (PI * 2.0) / 6.0;
        let sc = c + rotate(spinningpoint, num44);
        // Utils.CenteredRectangle(sc, (30,30)) (`Utils.cs:947-950`).
        let attacker = tr_core::whip::Rect::new(
            (sc.x - 15.0) as i32,
            (sc.y - 15.0) as i32,
            30,
            30,
        );
        if attacker.intersects(target) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::{proj_data, ProjectilePool, FlailInputs, YoyoInputs};
    use tr_core::{World, TILE_PX};

    struct Recorder {
        floats: usize,
    }
    impl DropRng for Recorder {
        fn next_range(&mut self, _min: i32, _max: i32) -> i32 {
            1
        }
        fn next_float(&mut self) -> f32 {
            self.floats += 1;
            0.5
        }
    }

    fn world() -> World {
        World::new("t".into(), 512, 256)
    }

    fn proj(vx: f32, vy: f32) -> Projectile {
        let d = proj_data(CULTIST_BOSS_ICE_MIST).expect("the 464 row");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = CULTIST_BOSS_ICE_MIST;
        p.owner = 255;
        p.friendly = false;
        p.pos = Vec2::new(10.0 * TILE_PX, 10.0 * TILE_PX);
        p.vel = Vec2::new(vx, vy);
        p.w = d.w;
        p.h = d.h;
        p.damage = 40;
        p.knock_back = 3.0;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.alpha = d.alpha;
        p.tile_collide = d.tile_collide;
        p.ignore_water = d.ignore_water;
        p
    }

    /// The row matches `SetDefaults` verbatim (`Projectile.cs:5351-5361`).
    #[test]
    fn w4_d9_the_ice_mist_row_matches_set_defaults() {
        let d = proj_data(CULTIST_BOSS_ICE_MIST).unwrap();
        assert_eq!((d.w, d.h), (60.0, 60.0));
        assert_eq!(d.ai_style, 86);
        assert!(!d.tile_collide);
        assert_eq!(d.penetrate, -1);
        assert_eq!(d.alpha, 255);
        assert!(d.ignore_water);
        assert_eq!(d.time_left, 3600);
    }

    /// The spawner: the 30-tick child spawn + the 150-tick kill + the sound
    /// latch + the rotation step.
    #[test]
    fn w4_d9_the_ice_mist_spawner() {
        let mut p = proj(0.0, 0.0);
        p.ai[1] = 1.0;
        p.rot = 0.0;
        let mut rng = Recorder { floats: 0 };
        let spawner = ai_ice_mist(&mut p, &mut rng);
        assert!(spawner, "the spawner form returns true");
        assert_eq!(p.wants_sound, Some((2, 120, p.pos.x, p.pos.y, 0.0, 1.0)));
        assert!((p.rot - PI / 30.0).abs() < 1e-6, "the rotation step");
        // 30 ticks: one child at ai[0]=30.
        for _ in 0..29 {
            ai_ice_mist(&mut p, &mut rng);
        }
        assert_eq!(p.wants_dart_spawns.len(), 1, "the 30-tick child");
        let child = p.wants_dart_spawns[0];
        assert_eq!(child.0, CULTIST_BOSS_ICE_MIST);
        assert_eq!(child.7, 0.0, "the child's ai0");
        assert_eq!(child.8, 0.0, "the child's ai1 (the mist form)");
        assert_eq!(rng.floats, 0, "the spawner draws nothing");
        // To 150: the kill.
        for _ in 0..120 {
            ai_ice_mist(&mut p, &mut rng);
        }
        assert!(!p.active, "the spawner dies at 150");
    }

    /// The mist form: the position unwinds, the alpha ramps, the 45-tick
    /// kill, and the six spokes' draws are consumed (24 floats).
    #[test]
    fn w4_d9_the_ice_mist_form() {
        let mut p = proj(2.0, 0.0);
        p.ai[1] = 0.0;
        let before = p.pos;
        let mut rng = Recorder { floats: 0 };
        let spawner = ai_ice_mist(&mut p, &mut rng);
        assert!(!spawner, "the mist form returns false");
        assert_eq!(p.pos, before - Vec2::new(2.0, 0.0), "the position unwinds");
        assert_eq!(rng.floats, 24, "6 spokes × 2 dusts × 2 floats");
        for _ in 0..44 {
            ai_ice_mist(&mut p, &mut rng);
        }
        assert!(!p.active, "the mist dies at 45");
    }

    /// The mist form's net movement is zero (the AI unwinds, kill_step adds
    /// back — the C# UpdatePosition net).
    #[test]
    fn w4_d9_the_ice_mist_hangs_stationary() {
        let mut p = proj(2.0, 0.0);
        p.ai[1] = 0.0;
        let before = p.pos;
        let mut rng = Recorder { floats: 0 };
        ai_ice_mist(&mut p, &mut rng);
        // The dispatch's kill_step (tileCollide false) adds the velocity.
        p.pos += p.vel;
        assert_eq!(p.pos, before, "the mist hangs at its spawn point");
    }

    /// The six-spoke hitbox: a target on a spoke offset hits, the spawner
    /// form falls through.
    #[test]
    fn w4_d9_the_ice_mist_six_spoke_hitbox() {
        let mut p = proj(0.0, 1.0); // velocity down → ToRotation = PI/2
        p.ai[1] = 0.0;
        p.ai[0] = 22.0; // ai[0]%45/45 ≈ 0.489, the spokes ~352 px out
        // A target far from the centre but on a spoke hits.
        let c = center(&p);
        let spinningpoint = rotate(Vec2::new(0.0, -720.0), p.vel.to_angle()) * (p.ai[0] % 45.0 / 45.0);
        let spoke0 = c + spinningpoint;
        let target = tr_core::whip::Rect::new(spoke0.x as i32 - 5, spoke0.y as i32 - 5, 10, 10);
        assert!(ice_mist_colliding(&p, target), "a spoke offset hits");
        // The spawner form falls through (no spoke test).
        p.ai[1] = 1.0;
        assert!(!ice_mist_colliding(&p, target));
    }

    /// The pool resolves the spawner's child (the wants_dart_spawns drain).
    #[test]
    fn w4_d9_the_pool_resolves_the_ice_mist_child() {
        let mut pool = ProjectilePool::new();
        let mut w = world();
        let slot = pool
            .spawn_hostile_ai(CULTIST_BOSS_ICE_MIST, Vec2::new(200.0, 200.0), Vec2::ZERO, 40, 3.0, 0.0, 1.0)
            .unwrap();
        let mut rng = Recorder { floats: 0 };
        for _ in 0..30 {
            pool.update(
                &mut w,
                &mut [],
                None,
                None,
                None,
                YoyoInputs::default(),
                FlailInputs::default(),
                &mut rng,
                0,
                true,
            );
        }
        let children: Vec<_> = pool
            .slots()
            .iter()
            .enumerate()
            .filter(|(i, p)| p.active && p.kind == CULTIST_BOSS_ICE_MIST && *i != slot)
            .collect();
        assert_eq!(children.len(), 1, "the spawner's child reached the pool");
    }
}
