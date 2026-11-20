//! `Projectile.cs`'s `Gore.NewGore*` spawn sites — wave C: sites 21-30 in
//! source order (`:50170`, `:50203`, `:51166`, `:52136`, `:52233`,
//! `:53151`-`:53301`).
//!
//! The pool + the wave-A contract live in `projectile_gore_a.rs` (W4-g16,
//! the `npc_hit_effect_a.rs` precedent). The dust bodies around each site
//! are the dust lanes' documented skip; argument draws precede the pool
//! call, the kicks roll inside `new_gore`, post-writes land on the returned
//! slot (sentinel included).
//!
//! ## Scope — sites 21-30 (source order)
//!
//! | # | C# site | arm |
//! | --- | --- | --- |
//! | 21 | `:50170` | `AI_067` jump else-branch (2 × Next(11,14)) |
//! | 22 | `:50203` | `AI_067` flag8 jump (2 × Next(11,14)) |
//! | 23 | `:51166` | `AI_067` idle gore (Next(900,1200) gate) |
//! | 24 | `:52136` | `AI_155_SpawnRopeIn` rope gore |
//! | 25 | `:52233` | `AI_153_ToiletPop` gore |
//! | 26 | `:53151` | `AI_147_Explosion` state 0 gore |
//! | 27 | `:53177` | `AI_147_Explosion` state 1 gores (×3) |
//! | 28 | `:53242` | `AI_147_Explosion` state 2 big-burst gore |
//! | 29 | `:53265` | `AI_147_Explosion` state 2 small-burst gore |
//! | 30 | `:53301` | `AI_147_Explosion` state 3 gore |
//!
//! ## RNG notes specific to this wave
//!
//! * The `AI_147` gore row (`celeb2_gore`) is IDENTICAL at all seven state
//!   sites (`:53151`/`:53177`/`:53242`/`:53265`/`:53301`/`:53327`/`:53391`)
//!   — one keep-one fn, the landed census's `gore(rng)` shape
//!   (`projectile_ai_misc.rs:490-498`) upgraded to the pool: the census
//!   consumed 7 range draws (position pair, type, kicks, velocity kicks);
//!   the pool fn consumes the SAME 7 (the kicks roll inside `new_gore`).
//!   The wiring wave replaces each census `gore(rng)` call with this fn.
//! * The idle gate (23) ALWAYS draws `Next(900, 1200)` (`:51157`); the
//!   comparison reads the caller's `localAI[0]`. The reset (`localAI[0] =
//!   0`) is the AI arm's state write, returned as the fn's bool.
//! * The toilet-pop count (25) is the caller's (`ai[0] == 0` → 1; else the
//!   `Next(5)` gate at `:52216` — that draw precedes this fn on the stream,
//!   the `volcano_gores` num7 convention).
//!
//! See docs/ANALYSIS.md "The Projectile gore sites (W4-g16)".
#![allow(dead_code)] // wiring deferred (code-only mode)

use crate::projectile::Projectile;
use glam::Vec2;
use tr_core::gore::GorePool;
use tr_core::{UnifiedRandom, World};

/// `Entity.Center` (`Entity.cs:50-60`).
fn center(p: &Projectile) -> Vec2 {
    Vec2::new(p.pos.x + p.w / 2.0, p.pos.y + p.h / 2.0)
}

/// `Utils.NextVector2Unit` (`Utils.cs:1261-1264`) — ONE `NextFloat`.
fn next_vector2_unit(rng: &mut UnifiedRandom) -> Vec2 {
    let angle = std::f32::consts::PI * 2.0 * rng.next_float();
    Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32)
}

/// `Utils.NextVector2Circular` (`Utils.cs:1266-1269`) — unit + radius.
fn next_vector2_circular(rng: &mut UnifiedRandom, hw: f32, hh: f32) -> Vec2 {
    let unit = next_vector2_unit(rng);
    let r = rng.next_float();
    Vec2::new(unit.x * hw * r, unit.y * hh * r)
}

/// Sites 21/22 — the `AI_067` jump small-gore pair (`Projectile.cs:50168-
/// 50173` else-branch / `:50201-50206` flag8 branch): 2 gores, each
/// `Next(11, 14)` at `Center + UnitX * -direction * 8`, post `velocity /=
/// 3`, `velocity.X = -abs * direction`. The 10-dust burst before each pair
/// is the dust lane's.
pub fn pirate_jump_small_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let pos = center(p) + Vec2::new(-p.direction as f32 * 8.0, 0.0);
    for _ in 0..2 {
        let ty = rng.next_range(11, 14);
        let idx = gore.new_gore(rng, world, pos, Vec2::ZERO, ty, 1.0);
        let g = gore.slot_mut(idx);
        g.velocity /= 3.0;
        g.velocity.x = -g.velocity.x.abs() * p.direction as f32;
    }
}

/// Site 23 — the `AI_067` idle gore (`Projectile.cs:51157-51170`): the
/// `Next(900, 1200)` gate ALWAYS draws; when `local_ai0 >= roll` the 6-dust
/// burst (the dust lane's) then ONE `Next(580, 583)` gore at `Center +
/// UnitX * -direction * 8`, post `velocity /= 2`, `velocity.Y = abs`,
/// `velocity.X = -abs * direction`. Returns true when the arm ran (the
/// caller resets `localAI[0] = 0`, `:51159`).
pub fn pirate_idle_gore(
    p: &Projectile,
    local_ai0: f32,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) -> bool {
    let roll = rng.next_range(900, 1200);
    if local_ai0 < roll as f32 {
        return false;
    }
    let ty = rng.next_range(580, 583);
    let pos = center(p) + Vec2::new(-p.direction as f32 * 8.0, 0.0);
    let idx = gore.new_gore(rng, world, pos, Vec2::ZERO, ty, 1.0);
    let g = gore.slot_mut(idx);
    g.velocity /= 2.0;
    g.velocity.y = g.velocity.y.abs();
    g.velocity.x = -g.velocity.x.abs() * p.direction as f32;
    true
}

/// Site 24 — `AI_155_SpawnRopeIn` (`Projectile.cs:52133-52141`): the
/// `Next(4) != 0` gate (3-in-4 TAKEN), `NewGoreDirect` at the tile centre
/// `(x*16+8, y*16+8)`, `Next(61, 64)` type; post `velocity =
/// Circular(1.5, 0.8)`, `velocity.X += 0.1 * sign`, `position = vector -
/// (16,16)`, `alpha = 170`. The 5-iteration dust loop after is the dust
/// lane's.
pub fn rope_spawn_gore(
    x: i32,
    y: i32,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    if rng.next_max(4) == 0 {
        return;
    }
    let vector = Vec2::new(x as f32 * 16.0 + 8.0, y as f32 * 16.0 + 8.0);
    let ty = rng.next_range(61, 64);
    let idx = gore.new_gore_direct(rng, world, vector, Vec2::ZERO, ty, 1.0);
    let g = gore.slot_mut(idx);
    g.velocity = next_vector2_circular(rng, 1.5, 0.8);
    g.velocity.x += 0.1 * if g.velocity.x > 0.0 { 1.0 } else { -1.0 };
    g.position = vector - Vec2::new(16.0, 16.0);
    g.alpha = 170;
}

/// Site 25 — `AI_153_ToiletPop` (`Projectile.cs:52231-52235`): `count`
/// gores (the caller's `num2`: 1 when `ai[0] == 0`, else the `Next(5)` gate
/// at `:52216` decides — that draw precedes this fn), each
/// `NewGoreDirect(center, UnitY*-3 + Circular(1,1), Next(580,583))`, post
/// `velocity.Y = -abs`.
pub fn toilet_pop_gores(
    p: &Projectile,
    count: i32,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let c = center(p);
    for _ in 0..count {
        let vel = Vec2::new(0.0, -3.0) + next_vector2_circular(rng, 1.0, 1.0);
        let ty = rng.next_range(580, 583);
        let idx = gore.new_gore_direct(rng, world, c, vel, ty, 1.0);
        gore.slot_mut(idx).velocity.y = -gore.slot(idx).velocity.y.abs();
    }
}

/// Sites 26-30 (and the wave-D `:53327`/`:53391` twins) — the `AI_147`
/// firework gore row (`Projectile.cs:53151-53154` shape, identical at all
/// seven sites): position `pos + (w*Next(100)/100, h*Next(100)/100) -
/// (10,10)`, type `Next(61, 64)`; post `velocity *= 0.3`, `velocity.X +=
/// Next(-10,11)*0.05`, `velocity.Y += Next(-10,11)*0.05`. The landed census
/// (`projectile_ai_misc.rs::celeb2_explosion`'s `gore(rng)`) consumed these
/// same 7 range draws; the wiring wave swaps each census call for this fn.
pub fn celeb2_gore(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let off = Vec2::new(
        (p.w * rng.next_max(100) as f32) / 100.0,
        (p.h * rng.next_max(100) as f32) / 100.0,
    );
    let ty = rng.next_range(61, 64);
    let idx = gore.new_gore(rng, world, p.pos + off - Vec2::splat(10.0), Vec2::ZERO, ty, 1.0);
    let g = gore.slot_mut(idx);
    g.velocity *= 0.3;
    g.velocity.x += rng.next_range(-10, 11) as f32 * 0.05;
    g.velocity.y += rng.next_range(-10, 11) as f32 * 0.05;
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
        p.direction = 1;
        p
    }

    /// `Projectile.cs:50168-50173` — the small-gore pair: 2 × (Next(11,14)
    /// + kicks), the post-writes draw nothing; the X sign flips by
    /// direction.
    #[test]
    fn w4_g16_the_pirate_small_gores_spawn_two_with_signed_x() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(2);
        pirate_jump_small_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(2);
        for _ in 0..2 {
            let _ = twin.next_range(11, 14);
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek());
        for i in 0..2 {
            let g = gore.slot(i);
            assert!((11..14).contains(&g.gore_type));
            // direction 1 → velocity.X negative (the -abs * dir write).
            assert!(g.velocity.x <= 0.0);
        }
    }

    /// `Projectile.cs:51157-51170` — the idle gate ALWAYS draws
    /// Next(900,1200); a below-threshold localAI[0] draws nothing more.
    #[test]
    fn w4_g16_the_pirate_idle_gate_always_draws() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(8);
        // localAI[0] = 0 < any roll (900..1199): gate only.
        assert!(!pirate_idle_gore(&p, 0.0, &mut gore, &mut rng, &w));
        let mut twin = UnifiedRandom::new(8);
        let _ = twin.next_range(900, 1200);
        assert_eq!(rng.peek(), twin.peek());
        // localAI[0] = 1200 >= any roll: the gore row runs.
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(8);
        assert!(pirate_idle_gore(&p, 1200.0, &mut gore, &mut rng, &w));
        let mut twin = UnifiedRandom::new(8);
        let _ = twin.next_range(900, 1200);
        let ty = twin.next_range(580, 583);
        let _ = twin.next_range(10, 31);
        let _ = twin.next_range(-20, 21);
        assert_eq!(rng.peek(), twin.peek());
        assert_eq!(gore.slot(0).gore_type, ty);
    }

    /// `Projectile.cs:52133-52141` — the rope gore: the Next(4) != 0 gate
    /// (3-in-4 taken), the type + kicks, then Circular(1.5, 0.8) (unit
    /// NextFloat + radius NextFloat); the position/alpha writes draw none.
    #[test]
    fn w4_g16_the_rope_gore_takes_three_in_four() {
        let w = air();
        let mut seed = 0;
        while UnifiedRandom::new(seed).next_max(4) == 0 {
            seed += 1;
        }
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(seed);
        rope_spawn_gore(10, 12, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(seed);
        let _ = twin.next_max(4);
        let ty = twin.next_range(61, 64);
        let _ = twin.next_range(10, 31);
        let _ = twin.next_range(-20, 21);
        let _ = twin.next_float(); // Circular unit
        let _ = twin.next_float(); // Circular radius
        assert_eq!(rng.peek(), twin.peek());
        let g = gore.slot(0);
        assert_eq!(g.gore_type, ty);
        assert_eq!(g.alpha, 170);
        // position = (10*16+8, 12*16+8) - (16,16) = (152, 184).
        assert_eq!(g.position, Vec2::new(152.0, 184.0));
        // The missed gate draws only Next(4).
        let mut miss = 0;
        while UnifiedRandom::new(miss).next_max(4) != 0 {
            miss += 1;
        }
        let mut gore2 = GorePool::new();
        let mut rng2 = UnifiedRandom::new(miss);
        rope_spawn_gore(10, 12, &mut gore2, &mut rng2, &w);
        let mut twin2 = UnifiedRandom::new(miss);
        let _ = twin2.next_max(4);
        assert_eq!(rng2.peek(), twin2.peek());
    }

    /// `Projectile.cs:52231-52235` — the toilet pop: per gore Circular(1,1)
    /// (2 floats) + Next(580,583) + kicks; the Y flip draws nothing.
    #[test]
    fn w4_g16_the_toilet_pop_gores_flip_y_up() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(12);
        toilet_pop_gores(&p, 1, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(12);
        let _ = twin.next_float();
        let _ = twin.next_float();
        let ty = twin.next_range(580, 583);
        let _ = twin.next_range(10, 31);
        let _ = twin.next_range(-20, 21);
        assert_eq!(rng.peek(), twin.peek());
        let g = gore.slot(0);
        assert_eq!(g.gore_type, ty);
        assert!(g.velocity.y <= 0.0, "the flip keeps Y up: {:?}", g.velocity);
        assert_eq!(g.position, Vec2::new(110.0, 110.0)); // the centre
    }

    /// `Projectile.cs:53151-53154` — the celeb2 row: the SAME 7 range draws
    /// the landed census consumed (position pair, type, kicks, velocity
    /// kicks), now feeding the pool.
    #[test]
    fn w4_g16_the_celeb2_gore_matches_the_census_seven_draws() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(15);
        celeb2_gore(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(15);
        let px = twin.next_range(0, 100);
        let py = twin.next_range(0, 100);
        let ty = twin.next_range(61, 64);
        let ky = twin.next_range(10, 31);
        let kx = twin.next_range(-20, 21);
        let vx = twin.next_range(-10, 11);
        let vy = twin.next_range(-10, 11);
        assert_eq!(rng.peek(), twin.peek(), "the census's 7 draws, in order");
        let g = gore.slot(0);
        assert_eq!(g.gore_type, ty);
        let expect_pos = p.pos
            + Vec2::new(p.w * px as f32 / 100.0, p.h * py as f32 / 100.0)
            - Vec2::splat(10.0);
        assert_eq!(g.position, expect_pos);
        let kicked = Vec2::new(kx as f32 * 0.1, -(ky as f32 * 0.1)) * 0.3;
        assert_eq!(
            g.velocity,
            kicked + Vec2::new(vx as f32 * 0.05, vy as f32 * 0.05)
        );
    }
}
