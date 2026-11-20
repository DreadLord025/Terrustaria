//! `Projectile.cs`'s `Gore.NewGore*` spawn sites — wave H: sites 75-84 in
//! source order (`:73625`, `:73668`, `:73691`-`:73692`, `:73754`-`:73758`,
//! `:73843`, `:73895`, `:74580`). The 510 pair, the 408 triple and the
//! confetti loop are consecutive blocks kept intact (the wave-E precedent).
//!
//! The pool + the wave-A contract live in `projectile_gore_a.rs` (W4-g16,
//! the `npc_hit_effect_a.rs` precedent). Dust bodies are the dust lanes'
//! skip; argument draws precede the pool call, kicks roll inside
//! `new_gore`, post-writes land on the returned slot (sentinel included).
//!
//! ## Scope — sites 75-84 (source order)
//!
//! | # | C# site | arm |
//! | --- | --- | --- |
//! | 75 | `:73625` | `Kill()` 448 gore (the wave-C row ×1) |
//! | 76 | `:73668` | `Kill()` 616 gore (the wave-C row ×1) |
//! | 77-78 | `:73691`/`:73692` | `Kill()` 510 pair (704/705) |
//! | 79-81 | `:73754`/`:73756`/`:73758` | `Kill()` 408 triple (584-586) |
//! | 82 | `:73843` | `Kill()` 424-426 gores ×2 (the wave-C row) |
//! | 83 | `:73895` | `Kill()` 1037/1049 gores ×3 (circular reposition) |
//! | 84 | `:74580` | `Kill()` 178 confetti ×40 |
//!
//! ## RNG notes specific to this wave
//!
//! * The 408 triple (79-81) divides each gore's `timeLeft` by 10 AFTER the
//!   spawn (`:73755` etc.) — the pool's sticky timeLeft (600) becomes 60.
//! * The nebula gores (83) are the celeb2 argument row PLUS a
//!   `Circular(25,25)` position reposition (TWO NextFloat) before the
//!   velocity writes — the 1078 twin at `:75030` (wave I) is the same row.
//! * The confetti (84) post-writes ADD before MULTIPLY (`:74581-74587`) —
//!   the OPPOSITE order of the enchant-7 site (`:17453-17457` multiplies
//!   first); seven rolls either way, the order is the site's own.
//!
//! See docs/ANALYSIS.md "The Projectile gore sites (W4-g16)".
#![allow(dead_code)] // wiring deferred (code-only mode)

use crate::projectile::Projectile;
use crate::projectile_gore_g::kill_row_gores;
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

/// Sites 75/76/82 — `Kill()`'s 448 ×1 (`:73623-73629`), 616 ×1 (`:73666-
/// 73672`), 424-426 ×2 (`:73841-73847`) gores: the wave-C celeb2 row,
/// keep-one. The caller ran the box resize first (448: 112; 616: 80;
/// 424-426: `(int)(128*scale)`).
pub fn kill448_616_424_gores(
    p: &Projectile,
    count: i32,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    kill_row_gores(p, count, gore, rng, world);
}

/// Sites 77-78 — `Kill()`'s 510 pair (`Projectile.cs:73691-73692`): two
/// `NewGore(Center, -oldVelocity*0.2, 704/705)`. The types are data — only
/// the kicks draw. The item-spawn tail after is the wiring wave's.
pub fn star_cannon_gores(
    p: &Projectile,
    old_velocity: Vec2,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let c = center(p);
    for ty in [704, 705] {
        gore.new_gore(rng, world, c, -old_velocity * 0.2, ty, 1.0);
    }
}

/// Sites 79-81 — `Kill()`'s 408 triple (`Projectile.cs:73752-73759`):
/// 584 at velocity ×0.8, 585 ×0.9, 586 ×1.0, each at the centre; post
/// `timeLeft /= 10` (the pool's sticky 600 → 60). The 15-dust loop before
/// is the dust lane's.
pub fn phoenix_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let c = center(p);
    for (ty, mul) in [(584, 0.8), (585, 0.9), (586, 1.0)] {
        let idx = gore.new_gore(rng, world, c, p.vel * mul, ty, 1.0);
        gore.slot_mut(idx).time_left /= 10;
    }
}

/// Site 83 — `Kill()`'s 1037/1049 gores (`Projectile.cs:73893-73900`): 3 ×
/// the celeb2 argument row (Next(100) pair + Next(61,64) + kicks), post
/// `position = Center + Circular(25,25) + (-20,-20)` (TWO NextFloat),
/// `velocity *= 0.3`, the two `Next(-10,11)*0.05` kicks. The caller ran
/// the Resize (1049: 112, 1037: 64) first.
pub fn nebula_kill_gores(
    p: &Projectile,
    count: i32,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    for _ in 0..count {
        let off = Vec2::new(
            (p.w * rng.next_max(100) as f32) / 100.0,
            (p.h * rng.next_max(100) as f32) / 100.0,
        );
        let ty = rng.next_range(61, 64);
        let idx = gore.new_gore(rng, world, p.pos + off - Vec2::splat(10.0), Vec2::ZERO, ty, 1.0);
        let g = gore.slot_mut(idx);
        g.position = center(p) + next_vector2_circular(rng, 25.0, 25.0) - Vec2::splat(20.0);
        g.velocity *= 0.3;
        g.velocity.x += rng.next_range(-10, 11) as f32 * 0.05;
        g.velocity.y += rng.next_range(-10, 11) as f32 * 0.05;
    }
}

/// Site 84 — `Kill()`'s 178 confetti (`Projectile.cs:74578-74588`):
/// `count` gores (40 for 178; the 289 twin at `:74603-74613` passes 15),
/// type `Next(276, 283)` at the position with the velocity; post ADD then
/// MULTIPLY: `X += Next(-50,51)*0.01`, `Y += …`, `X *= 1 + Next(-50,51)*
/// 0.01`, `Y *= …`, `scale *= 1 + Next(-20,21)*0.01`, `X += Next(-50,51)*
/// 0.05`, `Y += …` — seven rolls in that order.
pub fn confetti_gores(
    p: &Projectile,
    count: i32,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    for _ in 0..count {
        let ty = rng.next_range(276, 283);
        let idx = gore.new_gore(rng, world, p.pos, p.vel, ty, 1.0);
        let g = gore.slot_mut(idx);
        g.velocity.x += rng.next_range(-50, 51) as f32 * 0.01;
        g.velocity.y += rng.next_range(-50, 51) as f32 * 0.01;
        g.velocity.x *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
        g.velocity.y *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
        g.scale *= 1.0 + rng.next_range(-20, 21) as f32 * 0.01;
        g.velocity.x += rng.next_range(-50, 51) as f32 * 0.05;
        g.velocity.y += rng.next_range(-50, 51) as f32 * 0.05;
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

    /// `Projectile.cs:73691-73692` — the 510 pair: two types at the centre
    /// with -oldVelocity×0.2; only the kicks draw.
    #[test]
    fn w4_g16_the_star_cannon_pair_spawns_704_705() {
        let w = air();
        let p = proj();
        let ov = Vec2::new(3.0, -2.0);
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(43);
        star_cannon_gores(&p, ov, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(43);
        for _ in 0..2 {
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek());
        assert_eq!(gore.slot(0).gore_type, 704);
        assert_eq!(gore.slot(1).gore_type, 705);
    }

    /// `Projectile.cs:73752-73759` — the 408 triple: 584/585/586 at ×0.8/
    /// ×0.9/×1.0, each timeLeft divided by 10 (the sticky 600 → 60).
    #[test]
    fn w4_g16_the_phoenix_triple_divides_the_timeleft() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(45);
        phoenix_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(45);
        for _ in 0..3 {
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek());
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![584, 585, 586]);
        for g in &gore.slots()[..3] {
            assert_eq!(g.time_left, 60);
        }
    }

    /// `Projectile.cs:73893-73900` — the nebula gores: the celeb2 argument
    /// row + the Circular(25,25) reposition (two floats) + the two kicks.
    #[test]
    fn w4_g16_the_nebula_gores_reposition_by_circular() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(47);
        nebula_kill_gores(&p, 3, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(47);
        for _ in 0..3 {
            let _ = twin.next_max(100);
            let _ = twin.next_max(100);
            let _ = twin.next_range(61, 64);
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
            let _ = twin.next_float(); // Circular unit
            let _ = twin.next_float(); // Circular radius
            let _ = twin.next_range(-10, 11);
            let _ = twin.next_range(-10, 11);
        }
        assert_eq!(rng.peek(), twin.peek());
        let active = gore.slots()[..3].iter().filter(|g| g.active).count();
        assert_eq!(active, 3);
    }

    /// `Projectile.cs:74578-74588` — the confetti: type + kicks + SEVEN
    /// post rolls in the add-before-multiply order; the 289 twin shares
    /// the row at count 15.
    #[test]
    fn w4_g16_the_confetti_postrolls_add_before_multiply() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(49);
        confetti_gores(&p, 2, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(49);
        for i in 0..2 {
            let ty = twin.next_range(276, 283);
            let ky = twin.next_range(10, 31);
            let kx = twin.next_range(-20, 21);
            let ax = twin.next_range(-50, 51);
            let ay = twin.next_range(-50, 51);
            let mx = twin.next_range(-50, 51);
            let my = twin.next_range(-50, 51);
            let ms = twin.next_range(-20, 21);
            let bx = twin.next_range(-50, 51);
            let by = twin.next_range(-50, 51);
            let g = gore.slot(i);
            assert_eq!(g.gore_type, ty);
            // Replay the C# postlude on the kicked velocity.
            let mut v = p.vel + Vec2::new(kx as f32 * 0.1, -(ky as f32 * 0.1));
            v.x += ax as f32 * 0.01;
            v.y += ay as f32 * 0.01;
            v.x *= 1.0 + mx as f32 * 0.01;
            v.y *= 1.0 + my as f32 * 0.01;
            let s = 1.0 * (1.0 + ms as f32 * 0.01);
            v.x += bx as f32 * 0.05;
            v.y += by as f32 * 0.05;
            assert_eq!(g.velocity, v);
            assert_eq!(g.scale, s);
        }
        assert_eq!(rng.peek(), twin.peek());
    }

    /// The 448/616/424 delegation forwards to the wave-C row by count.
    #[test]
    fn w4_g16_the_kill448_delegation_matches_the_row() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(51);
        kill448_616_424_gores(&p, 1, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(51);
        let _ = twin.next_range(0, 100);
        let _ = twin.next_range(0, 100);
        let _ = twin.next_range(61, 64);
        let _ = twin.next_range(10, 31);
        let _ = twin.next_range(-20, 21);
        let _ = twin.next_range(-10, 11);
        let _ = twin.next_range(-10, 11);
        assert_eq!(rng.peek(), twin.peek());
    }
}
