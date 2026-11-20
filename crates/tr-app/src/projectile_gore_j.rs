//! `Projectile.cs`'s `Gore.NewGore*` spawn sites — wave J: sites 97-107 in
//! source order (`:75709`, `:75734`, `:75744`, `:75754`-`:75784`,
//! `:75831`-`:75843`). The 281 six-gore block and the 162 four-gore block
//! are consecutive statement groups kept intact (the wave-E precedent).
//!
//! The pool + the wave-A contract live in `projectile_gore_a.rs` (W4-g16,
//! the `npc_hit_effect_a.rs` precedent). Dust bodies are the dust lanes'
//! skip; argument draws precede the pool call, kicks roll inside
//! `new_gore`, post-writes land on the returned slot (sentinel included).
//!
//! ## Scope — sites 97-107 (source order)
//!
//! | # | C# site | arm |
//! | --- | --- | --- |
//! | 97 | `:75709` | `Kill()` 12/955 star gores ×7 (the wave-I row) |
//! | 98 | `:75734` | `Kill()` 9/503 gores ×num723 |
//! | 99 | `:75744` | `Kill()` 12 low-damage gores ×3 |
//! | 100-101 | `:75754`/`:75756` | `Kill()` 281 head gores (76/77) |
//! | 102-105 | `:75772`-`:75784` | `Kill()` 281 corner gores (±1) |
//! | 106-109 | `:75831`-`:75843` | `Kill()` 162 corner gores (±1.5) |
//!
//! (The 162 block rides along intact — 13 sites in the wave.)
//!
//! ## RNG notes specific to this wave
//!
//! * The 9/503 row (98): `Next(16, 18)` ALWAYS draws, THEN the 503 flag
//!   overrides the value to 16 (`:75729-75733`) — the draw is not skipped.
//!   The count is num723 (3; 2 for 503, whose velocity the caller halved
//!   at `:75721` BEFORE the dust loop).
//! * The 281 head gores (100-101) draw TWO `Next(-20, 21)` each for the
//!   velocity (×0.2), types 76/77 data; post `velocity -= vel*0.5`. The
//!   corner gores (102-105) come AFTER the 20+10-pair dust loops (the dust
//!   lane's) — the wiring must keep that order on the stream.
//! * The corner rows are the goblin-bomb shape with different magnitudes:
//!   281 ±1 (`:75772-75787`), 162 ±1.5 (`:75831-75846`, the 1053 twin at
//!   1.5 scale). One keep-one fn, magnitude parameter.
//!
//! See docs/ANALYSIS.md "The Projectile gore sites (W4-g16)".
#![allow(dead_code)] // wiring deferred (code-only mode)

use crate::projectile::Projectile;
use crate::projectile_gore_i::star_kill_gores;
use glam::Vec2;
use tr_core::gore::GorePool;
use tr_core::{UnifiedRandom, World};

/// Site 97 — `Kill()`'s 12/955 star gores (`Projectile.cs:75704-75711`):
/// the wave-I star row ×7, keep-one (the caller gates on the screen).
pub fn kill12_star_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    star_kill_gores(p, gore, rng, world);
}

/// Site 98 — `Kill()`'s 9/503 gores (`Projectile.cs:75727-75735`):
/// `count` gores (num723: 3, or 2 for 503), each `Next(16, 18)` drawn then
/// OVERRIDDEN to 16 when `is_503`; velocity ×0.05 (the caller halved the
/// 503's velocity at `:75721` first).
pub fn kill9_503_gores(
    p: &Projectile,
    count: i32,
    is_503: bool,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let vel = Vec2::new(p.vel.x * 0.05, p.vel.y * 0.05);
    for _ in 0..count {
        let mut ty = rng.next_range(16, 18);
        if is_503 {
            ty = 16;
        }
        gore.new_gore(rng, world, p.pos, vel, ty, 1.0);
    }
}

/// Site 99 — `Kill()`'s 12 low-damage gores (`Projectile.cs:75742-75745`):
/// the caller gates on `type == 12 && damage < 100`; 3 × `Next(16, 18)`
/// at velocity ×0.05. The wave-I row without the halving — keep-one.
pub fn kill12_low_damage_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    crate::projectile_gore_i::falling_star_kill_gores(p, false, gore, rng, world);
}

/// Sites 100-101 — `Kill()`'s 281 head gores (`Projectile.cs:75754-
/// 75757`): types 76 then 77, each velocity `(Next(-20,21)*0.2,
/// Next(-20,21)*0.2)` (TWO draws), post `velocity -= vel*0.5`. The caller
/// gates on `ai[0] != -3`.
pub fn tombstone_head_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    for ty in [76, 77] {
        let vel = Vec2::new(
            rng.next_range(-20, 21) as f32 * 0.2,
            rng.next_range(-20, 21) as f32 * 0.2,
        );
        let idx = gore.new_gore(rng, world, p.pos, vel, ty, 1.0);
        gore.slot_mut(idx).velocity -= p.vel * 0.5;
    }
}

/// Sites 102-105 / 106-109 — the corner-gore rows: `Kill()`'s 281 four
/// (`:75772-75787`, magnitude 1.0) and 162 four (`:75831-75846`, magnitude
/// 1.5): each `Next(61, 64)` at the position, post `velocity *= 0.4` then
/// the sign pair ×magnitude in order (+,+ / -, + / +, - / -, -). One
/// keep-one fn; the caller ran the box resize first (162: 64×64).
pub fn corner_gores(
    p: &Projectile,
    magnitude: f32,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let signs = [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)];
    for (sx, sy) in signs {
        let ty = rng.next_range(61, 64);
        let idx = gore.new_gore(rng, world, p.pos, Vec2::ZERO, ty, 1.0);
        let g = gore.slot_mut(idx);
        g.velocity *= 0.4;
        g.velocity.x += sx * magnitude;
        g.velocity.y += sy * magnitude;
    }
}

/// Site 105b (the gap fill) — `Kill()`'s 240 corner four (`Projectile.cs:
/// 75871-75886`, between the 162 corners and the 284 confetti in source
/// order): the SAME row as the 281 corners (magnitude 1.0) — keep-one onto
/// [`corner_gores`]. The caller ran the dust loops first.
pub fn kill240_corner_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    corner_gores(p, 1.0, gore, rng, world);
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

    /// `Projectile.cs:75727-75735` — the 9/503 row: the Next(16,18) ALWAYS
    /// draws; the 503 override changes the value, not the stream.
    #[test]
    fn w4_g16_the_503_override_keeps_the_type_draw() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(63);
        kill9_503_gores(&p, 2, true, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(63);
        for _ in 0..2 {
            let _ = twin.next_range(16, 18); // drawn even though overridden
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek());
        assert_eq!(gore.slot(0).gore_type, 16);
        assert_eq!(gore.slot(1).gore_type, 16);
        // The non-503 arm keeps the drawn type.
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(63);
        kill9_503_gores(&p, 3, false, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(63);
        for i in 0..3 {
            let ty = twin.next_range(16, 18);
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
            assert_eq!(gore.slot(i).gore_type, ty);
        }
        assert_eq!(rng.peek(), twin.peek());
    }

    /// `Projectile.cs:75754-75757` — the 281 head gores: two Next(-20,21)
    /// per gore + the kicks; the -= vel*0.5 postlude is exact.
    #[test]
    fn w4_g16_the_tombstone_head_gores_draw_two_velocity_rolls() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(65);
        tombstone_head_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(65);
        for (i, ty) in [76, 77].iter().enumerate() {
            let vx = twin.next_range(-20, 21);
            let vy = twin.next_range(-20, 21);
            let ky = twin.next_range(10, 31);
            let kx = twin.next_range(-20, 21);
            let g = gore.slot(i);
            assert_eq!(g.gore_type, *ty);
            let base = Vec2::new(vx as f32 * 0.2, vy as f32 * 0.2);
            let kicked = base + Vec2::new(kx as f32 * 0.1, -(ky as f32 * 0.1));
            assert_eq!(g.velocity, kicked - p.vel * 0.5);
        }
        assert_eq!(rng.peek(), twin.peek());
    }

    /// `Projectile.cs:75772-75787` / `:75831-75846` — the corner rows:
    /// four type rolls + kicks, the ×0.4 then ±magnitude postlude.
    #[test]
    fn w4_g16_the_corner_rows_scale_by_magnitude() {
        let w = air();
        let p = proj();
        for mag in [1.0f32, 1.5] {
            let mut gore = GorePool::new();
            let mut rng = UnifiedRandom::new(67);
            corner_gores(&p, mag, &mut gore, &mut rng, &w);
            let mut twin = UnifiedRandom::new(67);
            let signs = [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)];
            for (i, (sx, sy)) in signs.iter().enumerate() {
                let _ = twin.next_range(61, 64);
                let ky = twin.next_range(10, 31);
                let kx = twin.next_range(-20, 21);
                let kicked = Vec2::new(kx as f32 * 0.1, -(ky as f32 * 0.1)) * 0.4;
                assert_eq!(
                    gore.slot(i).velocity,
                    kicked + Vec2::new(sx * mag, sy * mag)
                );
            }
            assert_eq!(rng.peek(), twin.peek());
        }
    }

    /// The keep-one delegations: the 12/955 star row (7 × float+Next(8)+
    /// kicks) and the 12 low-damage row (3 × Next(16,18)+kicks).
    #[test]
    fn w4_g16_the_kill12_delegations_match_their_rows() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(69);
        kill12_star_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(69);
        for _ in 0..7 {
            let _ = twin.next_float();
            let _ = twin.next_max(8);
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek());
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(71);
        kill12_low_damage_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(71);
        for _ in 0..3 {
            let _ = twin.next_range(16, 18);
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek());
    }
}
