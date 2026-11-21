//! `Projectile.cs`'s `Gore.NewGore*` spawn sites — wave I: sites 86-95 in
//! source order (`:74605`, `:74812`, `:74898`-`:74910`, `:75030`,
//! `:75145`, `:75172`, `:75204`, `:75221`). The 1053 lava-boulder four is
//! a consecutive block kept intact (the wave-E precedent).
//!
//! The pool + the wave-A contract live in `projectile_gore_a.rs` (W4-g16,
//! the `npc_hit_effect_a.rs` precedent). Dust bodies are the dust lanes'
//! skip; argument draws precede the pool call, kicks roll inside
//! `new_gore`, post-writes land on the returned slot (sentinel included).
//!
//! ## Scope — sites 86-95 (source order)
//!
//! | # | C# site | arm |
//! | --- | --- | --- |
//! | 86 | `:74605` | `Kill()` 289 confetti ×15 (the wave-H row) |
//! | 87 | `:74812` | `Kill()` 191-194 tombstone gore |
//! | 88-91 | `:74898`-`:74910` | `Kill()` 1053 lava-boulder four |
//! | 92 | `:75030` | `Kill()` 1078 gores ×3 (the wave-H nebula row) |
//! | 93 | `:75145` | `Kill()` 728 star gores ×7 |
//! | 94 | `:75172` | `Kill()` 723 gores ×3 (velocity ×2) |
//! | 95 | `:75204` | `Kill()` 724/725/726/907/1133 gores ×3 |
//! | 96 | `:75221` | `Kill()` 91/92 gores ×3 (the :75204 row) |
//!
//! (Site 96 rides along: the 91/92 loop is the same row without the 907
//! flag — one keep-one fn, the delegation keeps the boundary at the row.)
//!
//! ## RNG notes specific to this wave
//!
//! * The tombstone gore (87) reads INTEGER `width / 2` / `height / 2`
//!   (`:74812`'s `(float)(base.width / 2)` — the int division truncates
//!   BEFORE the float cast) and passes `this.scale` as the spawn scale.
//! * The star gores (93) are screen-gated (NO Next gate): per gore ONE
//!   `NextFloat` (the CircularEdge unit) + `Next(8)` (the 8-element
//!   SelectRandom) + the kicks; the velocity is the unit ×0.5 ×
//!   `velocity.Length()` (the length is read per gore, no draw).
//! * The 723 gores (94) use `NewGoreDirect(...).velocity *= 2` — the ×2
//!   lands on the KICKED velocity (the pool's kicks are already in).
//! * The :75204/:75221 row (95/96) is one keep-one fn with the 907 flag:
//!   `Next(16, 18)` per gore at velocity ×0.05; 907 halves AFTER.
//!
//! See docs/ANALYSIS.md "The Projectile gore sites (W4-g16)".
#![allow(dead_code)] // wiring deferred (code-only mode)

use crate::projectile::Projectile;
use crate::projectile_gore_h::{confetti_gores, nebula_kill_gores};
use glam::Vec2;
use tr_core::gore::GorePool;
use tr_core::World;
use tr_core::loot::DropRng;
#[cfg(test)]
use tr_core::UnifiedRandom;

/// `Utils.NextVector2Unit` (`Utils.cs:1261-1264`) — ONE `NextFloat`.
fn next_vector2_unit(rng: &mut dyn DropRng) -> Vec2 {
    let angle = std::f32::consts::PI * 2.0 * rng.next_float();
    Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32)
}

/// Site 86 — `Kill()`'s 289 confetti (`Projectile.cs:74603-74613`): the
/// wave-H row at count 15, keep-one.
pub fn kill289_confetti(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    confetti_gores(p, 15, gore, rng, world);
}

/// Site 87 — `Kill()`'s 191-194 tombstone gore (`Projectile.cs:74810-
/// 74814`): ONE gore at `pos - (width/2, height/2)` (INTEGER division),
/// velocity ZERO, type `Next(61, 64)`, scale `this.scale`; post
/// `velocity *= 0.1`.
pub fn tombstone_kill_gore(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let pos = Vec2::new(
        p.pos.x - (p.w as i32 / 2) as f32,
        p.pos.y - (p.h as i32 / 2) as f32,
    );
    let ty = rng.next_range(61, 64);
    let idx = gore.new_gore(rng, world, pos, Vec2::ZERO, ty, p.scale);
    gore.slot_mut(idx).velocity *= 0.1;
}

/// Sites 88-91 — `Kill()`'s 1053 lava-boulder four (`Projectile.cs:74898-
/// 74913`): four gores at `pos + (width/2 - 24, height/2 - 24)` (INTEGER
/// division; the caller resized to 64×64 first), type `Next(61, 64)`;
/// post `scale = 1.5`, the sign pairs ±1.5 in order (+,+ / -, + / +, - /
/// -, -).
pub fn lava_boulder_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let pos = Vec2::new(
        p.pos.x + (p.w as i32 / 2) as f32 - 24.0,
        p.pos.y + (p.h as i32 / 2) as f32 - 24.0,
    );
    let signs = [(1.5, 1.5), (-1.5, 1.5), (1.5, -1.5), (-1.5, -1.5)];
    for (sx, sy) in signs {
        let ty = rng.next_range(61, 64);
        let idx = gore.new_gore(rng, world, pos, Vec2::ZERO, ty, 1.0);
        let g = gore.slot_mut(idx);
        g.scale = 1.5;
        g.velocity.x += sx;
        g.velocity.y += sy;
    }
}

/// Site 92 — `Kill()`'s 1078 gores (`Projectile.cs:75028-75035`): the
/// wave-H nebula row ×3, keep-one (the caller ran the `(int)(192*scale)`
/// resize + `Resize(64, 64)` first).
pub fn kill1078_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    nebula_kill_gores(p, 3, gore, rng, world);
}

/// Site 93 — `Kill()`'s 728 star gores (`Projectile.cs:75140-75147`): the
/// caller gates on the padded-screen intersect (NO Next gate); 7 gores,
/// each velocity = `CircularEdge(0.5, 0.5) * velocity.Length()` (ONE
/// NextFloat), type `SelectRandom(16, 17×7)` = `Next(8)`.
pub fn star_kill_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let len = p.vel.length();
    for _ in 0..7 {
        let unit = next_vector2_unit(rng);
        let vel = Vec2::new(unit.x * 0.5, unit.y * 0.5) * len;
        let ty = [16, 17, 17, 17, 17, 17, 17, 17][rng.next_max(8) as usize];
        gore.new_gore(rng, world, p.pos, vel, ty, 1.0);
    }
}

/// Site 94 — `Kill()`'s 723 gores (`Projectile.cs:75170-75173`): 3 ×
/// `NewGoreDirect(pos, velocity*0.05, SelectRandom(16, 17, 17))` then
/// `.velocity *= 2` on the KICKED velocity.
pub fn shadow_kill_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let vel = Vec2::new(p.vel.x * 0.05, p.vel.y * 0.05);
    for _ in 0..3 {
        let ty = [16, 17, 17][rng.next_max(3) as usize];
        let idx = gore.new_gore_direct(rng, world, p.pos, vel, ty, 1.0);
        gore.slot_mut(idx).velocity *= 2.0;
    }
}

/// Sites 95/96 — `Kill()`'s 724/725/726/907/1133 (`:75202-75209`) and
/// 91/92 (`:75219-75222`) gores: 3 × `NewGore(pos, velocity*0.05,
/// Next(16, 18))`; the 907 flag halves the velocity AFTER (the 91/92 arm
/// passes false). One keep-one fn.
pub fn falling_star_kill_gores(
    p: &Projectile,
    halve: bool,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let vel = Vec2::new(p.vel.x * 0.05, p.vel.y * 0.05);
    for _ in 0..3 {
        let ty = rng.next_range(16, 18);
        let idx = gore.new_gore(rng, world, p.pos, vel, ty, 1.0);
        if halve {
            gore.slot_mut(idx).velocity *= 0.5;
        }
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
        p.scale = 1.0;
        p
    }

    /// `Projectile.cs:74810-74814` — the tombstone gore: the INTEGER
    /// half-size offset, the spawn scale, the ×0.1 postlude.
    #[test]
    fn w4_g16_the_tombstone_kill_uses_integer_halves() {
        let w = air();
        let mut p = proj();
        p.w = 21.0; // int 21 / 2 = 10, NOT 10.5
        p.h = 21.0;
        p.scale = 0.8;
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(53);
        tombstone_kill_gore(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(53);
        let ty = twin.next_range(61, 64);
        let _ = twin.next_range(10, 31);
        let _ = twin.next_range(-20, 21);
        assert_eq!(rng.peek(), twin.peek());
        let g = gore.slot(0);
        assert_eq!(g.gore_type, ty);
        assert_eq!(g.position, Vec2::new(90.0, 90.0));
        assert_eq!(g.scale, 0.8);
    }

    /// `Projectile.cs:74898-74913` — the lava boulder: four type rolls +
    /// kicks, scale 1.5, the ±1.5 sign pairs in order.
    #[test]
    fn w4_g16_the_lava_boulder_four_signs() {
        let w = air();
        let mut p = proj();
        p.w = 64.0;
        p.h = 64.0;
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(55);
        lava_boulder_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(55);
        let signs = [(1.5, 1.5), (-1.5, 1.5), (1.5, -1.5), (-1.5, -1.5)];
        for (i, (sx, sy)) in signs.iter().enumerate() {
            let _ = twin.next_range(61, 64);
            let ky = twin.next_range(10, 31);
            let kx = twin.next_range(-20, 21);
            let g = gore.slot(i);
            assert_eq!(g.scale, 1.5);
            // pos + (64/2 - 24) = pos + 8.
            assert_eq!(g.position, Vec2::new(108.0, 108.0));
            let kicked = Vec2::new(kx as f32 * 0.1, -(ky as f32 * 0.1));
            assert_eq!(g.velocity, kicked + Vec2::new(*sx, *sy));
        }
        assert_eq!(rng.peek(), twin.peek());
    }

    /// `Projectile.cs:75140-75147` — the star kill: 7 × (one float + Next(8)
    /// + kicks), the velocity scaled by the length.
    #[test]
    fn w4_g16_the_star_kill_draws_one_float_plus_next_eight() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(57);
        star_kill_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(57);
        for _ in 0..7 {
            let _ = twin.next_float();
            let pick = twin.next_max(8);
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
            let _ = pick;
        }
        assert_eq!(rng.peek(), twin.peek());
        let active = gore.slots()[..7].iter().filter(|g| g.active).count();
        assert_eq!(active, 7);
    }

    /// `Projectile.cs:75170-75173` — the 723 kill: 3 × (Next(3) + kicks),
    /// the ×2 lands on the kicked velocity.
    #[test]
    fn w4_g16_the_shadow_kill_doubles_the_kicked_velocity() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(59);
        shadow_kill_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(59);
        for i in 0..3 {
            let pick = twin.next_max(3);
            let ky = twin.next_range(10, 31);
            let kx = twin.next_range(-20, 21);
            let g = gore.slot(i);
            assert_eq!(g.gore_type, [16, 17, 17][pick as usize]);
            let base = Vec2::new(p.vel.x * 0.05, p.vel.y * 0.05);
            let kicked = base + Vec2::new(kx as f32 * 0.1, -(ky as f32 * 0.1));
            assert_eq!(g.velocity, kicked * 2.0);
        }
        assert_eq!(rng.peek(), twin.peek());
    }

    /// `Projectile.cs:75202-75209` / `:75219-75222` — the shared row:
    /// 3 × (Next(16,18) + kicks); the 907 halving lands after, the 91/92
    /// arm leaves the kicked velocity.
    #[test]
    fn w4_g16_the_falling_star_kill_halves_only_for_907() {
        let w = air();
        let p = proj();
        for halve in [true, false] {
            let mut gore = GorePool::new();
            let mut rng = UnifiedRandom::new(61);
            falling_star_kill_gores(&p, halve, &mut gore, &mut rng, &w);
            let mut twin = UnifiedRandom::new(61);
            for i in 0..3 {
                let ty = twin.next_range(16, 18);
                let ky = twin.next_range(10, 31);
                let kx = twin.next_range(-20, 21);
                let g = gore.slot(i);
                assert_eq!(g.gore_type, ty);
                let base = Vec2::new(p.vel.x * 0.05, p.vel.y * 0.05);
                let mut kicked = base + Vec2::new(kx as f32 * 0.1, -(ky as f32 * 0.1));
                if halve {
                    kicked *= 0.5;
                }
                assert_eq!(g.velocity, kicked);
            }
            assert_eq!(rng.peek(), twin.peek());
        }
    }
}
