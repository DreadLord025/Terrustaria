//! `Projectile.cs`'s `Gore.NewGore*` spawn sites — wave K: sites 110-119 in
//! source order (`:75990`, `:76015`, `:76116`, `:76323`-`:76327`,
//! `:76439`-`:76451`, `:76881`). The 183 bee four is a consecutive block
//! kept intact (the wave-E precedent).
//!
//! The pool + the wave-A contract live in `projectile_gore_a.rs` (W4-g16,
//! the `npc_hit_effect_a.rs` precedent). Dust bodies are the dust lanes'
//! skip; argument draws precede the pool call, kicks roll inside
//! `new_gore`, post-writes land on the returned slot (sentinel included).
//!
//! ## Scope — sites 110-119 (source order)
//!
//! | # | C# site | arm |
//! | --- | --- | --- |
//! | 110 | `:75990` | `Kill()` 284 confetti ×5 (vel −vel×0.3) |
//! | 111 | `:76015` | `Kill()` 286 gore (pos −10/−10) |
//! | 112 | `:76116` | `Kill()` 712 gores ×3 (1008, scale jitter) |
//! | 113-114 | `:76323`/`:76327` | `Kill()` 41 gores ×2 |
//! | 115-118 | `:76439`-`:76451` | `Kill()` 183 bee four (signs THEN ×0.3) |
//! | 119 | `:76881` | `Kill()` 1023 barrel gores ×8 |
//!
//! ## RNG notes specific to this wave
//!
//! * The 284 confetti (110) is the wave-H row with velocity `-vel*0.3`
//!   (`confetti_gores_vel`, the keep-one parameterization).
//! * The 286 gore (111) is NOT the celeb2 row — NO `Next(100)` position
//!   pair; the position is the fixed `pos - (10,10)`. Type + kicks + ×0.3
//!   + the two `Next(-10,11)*0.05` kicks.
//! * The 712 gores (112) pass scale `1 + NextFloatDirection*0.2` (ONE
//!   NextDouble); type 1008 is SpecialAI==3 → the pool draws the four leaf
//!   rolls (`Gore.cs:1391-1402`); the `.velocity *= 4` lands after.
//! * The 183 bee four (115-118) applies the signs FIRST then ×0.3 — the
//!   OPPOSITE order of the wave-J corner rows (×0.4 then signs). A distinct
//!   fn, not a keep-one onto `corner_gores`.
//! * The 1023 barrel (119): position `Center + Circular(w,h)*0.125` (two
//!   floats), type `61 + Next(3)`, kicks, then the velocity OVERWRITE
//!   `(PI*2*i/8).ToRotationVector2() * NextFloat * 8` (one more float).
//!
//! See docs/ANALYSIS.md "The Projectile gore sites (W4-g16)".
#![allow(dead_code)] // wiring deferred (code-only mode)

use crate::projectile::Projectile;
use crate::projectile_gore_h::confetti_gores_vel;
use glam::Vec2;
use tr_core::gore::GorePool;
use tr_core::World;
use tr_core::loot::DropRng;
#[cfg(test)]
use tr_core::UnifiedRandom;

/// `Entity.Center` (`Entity.cs:50-60`).
fn center(p: &Projectile) -> Vec2 {
    Vec2::new(p.pos.x + p.w / 2.0, p.pos.y + p.h / 2.0)
}

/// `Utils.NextVector2Unit` (`Utils.cs:1261-1264`) — ONE `NextFloat`.
fn next_vector2_unit(rng: &mut dyn DropRng) -> Vec2 {
    let angle = std::f32::consts::PI * 2.0 * rng.next_float();
    Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32)
}

/// `Utils.NextVector2Circular` (`Utils.cs:1266-1269`) — unit + radius.
fn next_vector2_circular(rng: &mut dyn DropRng, hw: f32, hh: f32) -> Vec2 {
    let unit = next_vector2_unit(rng);
    let r = rng.next_float();
    Vec2::new(unit.x * hw * r, unit.y * hh * r)
}

/// Site 110 — `Kill()`'s 284 confetti (`Projectile.cs:75988-75998`): the
/// wave-H confetti row ×5 at velocity `-vel*0.3`, keep-one.
pub fn kill284_confetti(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    confetti_gores_vel(p, 5, -p.vel * 0.3, gore, rng, world);
}

/// Site 111 — `Kill()`'s 286 gore (`Projectile.cs:76015-76018`): ONE gore
/// at `pos - (10,10)` (NO Next(100) pair), type `Next(61, 64)`; post
/// `velocity *= 0.3`, the two `Next(-10,11)*0.05` kicks.
pub fn kill286_gore(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let ty = rng.next_range(61, 64);
    let pos = Vec2::new(p.pos.x - 10.0, p.pos.y - 10.0);
    let idx = gore.new_gore(rng, world, pos, Vec2::ZERO, ty, 1.0);
    let g = gore.slot_mut(idx);
    g.velocity *= 0.3;
    g.velocity.x += rng.next_range(-10, 11) as f32 * 0.05;
    g.velocity.y += rng.next_range(-10, 11) as f32 * 0.05;
}

/// Site 112 — `Kill()`'s 712 gores (`Projectile.cs:76114-76117`): 3 ×
/// `NewGoreDirect(pos, Zero, 1008, 1 + NextFloatDirection*0.2)` then
/// `.velocity *= 4`. The 1008 SpecialAI==3 four draw inside the pool.
pub fn kill712_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    for _ in 0..3 {
        let scale = 1.0 + rng.next_float_direction() * 0.2;
        let idx = gore.new_gore_direct(rng, world, p.pos, Vec2::ZERO, 1008, scale);
        gore.slot_mut(idx).velocity *= 4.0;
    }
}

/// Sites 113-114 — `Kill()`'s 41 gores (`Projectile.cs:76323-76330`): 2 ×
/// `Next(61, 64)` at the position; post `velocity *= 0.4`, the two
/// `Next(-10,11)*0.1` kicks (×0.1, NOT the celeb2 ×0.05).
pub fn kill41_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    for _ in 0..2 {
        let ty = rng.next_range(61, 64);
        let idx = gore.new_gore(rng, world, p.pos, Vec2::ZERO, ty, 1.0);
        let g = gore.slot_mut(idx);
        g.velocity *= 0.4;
        g.velocity.x += rng.next_range(-10, 11) as f32 * 0.1;
        g.velocity.y += rng.next_range(-10, 11) as f32 * 0.1;
    }
}

/// Sites 115-118 — `Kill()`'s 183 bee four (`Projectile.cs:76439-76454`):
/// four `Next(61, 64)` at the position; post the sign pair ±1 FIRST then
/// `velocity *= 0.3` (the OPPOSITE order of the wave-J corner rows).
pub fn bee_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let signs = [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)];
    for (sx, sy) in signs {
        let ty = rng.next_range(61, 64);
        let idx = gore.new_gore(rng, world, p.pos, Vec2::ZERO, ty, 1.0);
        let g = gore.slot_mut(idx);
        g.velocity.x += sx;
        g.velocity.y += sy;
        g.velocity *= 0.3;
    }
}

/// Site 119 — `Kill()`'s 1023 barrel gores (`Projectile.cs:76879-76882`):
/// 8 gores, position `Center + Circular(w,h)*0.125` (two floats), type
/// `61 + Next(3)`, kicks; post the velocity OVERWRITE
/// `(PI*2*i/8).ToRotationVector2() * NextFloat * 8` (one more float). The
/// caller resized to 192×192 first.
pub fn barrel_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let c = center(p);
    for i in 0..8 {
        let pos = c + next_vector2_circular(rng, p.w, p.h) * 0.125;
        let ty = 61 + rng.next_max(3);
        let idx = gore.new_gore_direct(rng, world, pos, Vec2::ZERO, ty, 1.0);
        let angle = std::f32::consts::PI * 2.0 * i as f32 / 8.0;
        let dir = Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32);
        gore.slot_mut(idx).velocity = dir * rng.next_float() * 8.0;
    }
}

// ---- witness tests (WRITTEN, NOT RUN — code-only mode, no compiler) ----
#[cfg(test)]
mod tests {
    use super::*;

    fn air() -> World {
        World::new("t".into(), 200, 200)
    }

    fn proj() -> Projectile {
        let mut p = Projectile::default();
        p.active = true;
        p.pos = Vec2::new(100.0, 100.0);
        p.vel = Vec2::new(2.0, -1.0);
        p.w = 20.0;
        p.h = 20.0;
        p
    }

    /// `Projectile.cs:76015-76018` — the 286 gore: NO Next(100) pair; the
    /// fixed pos−10 offset, type + kicks + ×0.3 + the two ×0.05 kicks.
    #[test]
    fn w4_g16_the_286_gore_has_no_position_pair() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(73);
        kill286_gore(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(73);
        let ty = twin.next_range(61, 64);
        let ky = twin.next_range(10, 31);
        let kx = twin.next_range(-20, 21);
        let vx = twin.next_range(-10, 11);
        let vy = twin.next_range(-10, 11);
        assert_eq!(rng.peek(), twin.peek());
        let g = gore.slot(0);
        assert_eq!(g.gore_type, ty);
        assert_eq!(g.position, Vec2::new(90.0, 90.0));
        let kicked = Vec2::new(kx as f32 * 0.1, -(ky as f32 * 0.1)) * 0.3;
        assert_eq!(g.velocity, kicked + Vec2::new(vx as f32 * 0.05, vy as f32 * 0.05));
    }

    /// `Projectile.cs:76114-76117` — the 712 gores: per gore ONE
    /// NextFloatDirection (scale) + kicks + the SpecialAI four; ×4 after.
    #[test]
    fn w4_g16_the_712_gores_draw_the_scale_and_special_ai() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(75);
        kill712_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(75);
        for _ in 0..3 {
            let _ = twin.next_float_direction(); // the scale jitter
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
            // 1008 SpecialAI==3: two floats + two Next(8).
            let _ = twin.next_float();
            let _ = twin.next_float();
            let _ = twin.next_max(8);
            let _ = twin.next_max(8);
        }
        assert_eq!(rng.peek(), twin.peek());
        for g in &gore.slots()[..3] {
            assert_eq!(g.gore_type, 1008);
        }
    }

    /// `Projectile.cs:76323-76330` — the 41 gores: ×0.4 then the ×0.1 kicks.
    #[test]
    fn w4_g16_the_41_gores_kick_by_tenth() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(77);
        kill41_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(77);
        for i in 0..2 {
            let _ = twin.next_range(61, 64);
            let ky = twin.next_range(10, 31);
            let kx = twin.next_range(-20, 21);
            let vx = twin.next_range(-10, 11);
            let vy = twin.next_range(-10, 11);
            let kicked = Vec2::new(kx as f32 * 0.1, -(ky as f32 * 0.1)) * 0.4;
            assert_eq!(
                gore.slot(i).velocity,
                kicked + Vec2::new(vx as f32 * 0.1, vy as f32 * 0.1)
            );
        }
        assert_eq!(rng.peek(), twin.peek());
    }

    /// `Projectile.cs:76439-76454` — the bee four: signs FIRST then ×0.3
    /// (the opposite order of the corner rows).
    #[test]
    fn w4_g16_the_bee_four_signs_before_scale() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(79);
        bee_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(79);
        let signs = [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)];
        for (i, (sx, sy)) in signs.iter().enumerate() {
            let _ = twin.next_range(61, 64);
            let ky = twin.next_range(10, 31);
            let kx = twin.next_range(-20, 21);
            let kicked = Vec2::new(kx as f32 * 0.1, -(ky as f32 * 0.1));
            assert_eq!(gore.slot(i).velocity, (kicked + Vec2::new(*sx, *sy)) * 0.3);
        }
        assert_eq!(rng.peek(), twin.peek());
    }

    /// `Projectile.cs:76879-76882` — the barrel: per gore Circular (two
    /// floats) + Next(3) + kicks + the velocity-overwrite NextFloat.
    #[test]
    fn w4_g16_the_barrel_gores_overwrite_the_velocity() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(81);
        barrel_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(81);
        for _ in 0..8 {
            let _ = twin.next_float(); // Circular unit
            let _ = twin.next_float(); // Circular radius
            let _ = twin.next_max(3);
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
            let _ = twin.next_float(); // the velocity overwrite
        }
        assert_eq!(rng.peek(), twin.peek());
        let active = gore.slots()[..8].iter().filter(|g| g.active).count();
        assert_eq!(active, 8);
    }
}
