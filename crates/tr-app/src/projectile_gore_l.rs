//! `Projectile.cs`'s `Gore.NewGore*` spawn sites — wave L: sites 120-131 in
//! source order (`:76919`-`:76931`, `:76960`, `:77003`, `:77172`,
//! `:77189`). The 312 eight-gore block (two passes × four corners) is kept
//! intact (the wave-E precedent).
//!
//! The pool + the wave-A contract live in `projectile_gore_a.rs` (W4-g16,
//! the `npc_hit_effect_a.rs` precedent). Dust bodies are the dust lanes'
//! skip; argument draws precede the pool call, kicks roll inside
//! `new_gore`, post-writes land on the returned slot (sentinel included).
//!
//! ## Scope — sites 120-131 (source order)
//!
//! | # | C# site | arm |
//! | --- | --- | --- |
//! | 120-127 | `:76919`-`:76931` | `Kill()` 312 two-pass corners (×0.4/×0.8) |
//! | 128 | `:76960` | `Kill()` 246 gore (the wave-G 483 row) |
//! | 129 | `:77003` | `Kill()` 249 gore (×0.2 + integer kicks) |
//! | 130 | `:77172` | `Kill()` 370 gores ×30 (331, sticky off) |
//! | 131 | `:77189` | `Kill()` 371 gores ×30 (435-437, ×0.4 vector) |
//!
//! ## RNG notes specific to this wave
//!
//! * The 312 block (120-127): TWO passes (`num916` 0/1), each four corners;
//!   the scale `num917` is 0.4 then 0.8, applied BEFORE the ±1 signs (the
//!   goblin-bomb order, NOT the bee order). The caller resized to 22×22.
//! * The 246 gore (128) is the wave-G `kill483_gore` row verbatim (×0.3 +
//!   integer `Next(-1,2)` kicks) — keep-one.
//! * The 370/371 gores (130/131): per gore TWO `Next(-10,11)` (the
//!   direction vector, normalized — no draw), `Next(4,9)` (the speed),
//!   `Next(40,141)`/`Next(20,100)` (the scale), then the kicks; post
//!   `sticky = false`. The 371 vector is ×0.4 BEFORE both the position and
//!   velocity uses (`:77188`); the 370 type is the literal 331, the 371
//!   type draws `Next(435,438)` AFTER the speed (argument order).
//!
//! See docs/ANALYSIS.md "The Projectile gore sites (W4-g16)".
#![allow(dead_code)] // wiring deferred (code-only mode)

use crate::projectile::Projectile;
use crate::projectile_gore_g::kill483_gore;
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

/// Sites 120-127 — `Kill()`'s 312 two-pass corners (`Projectile.cs:76912-
/// 76935`): two passes at scale 0.4 then 0.8, each four `Next(61, 64)`
/// gores at the position; post `velocity *= num917` FIRST then the ±1
/// sign pairs (+,+ / -, + / +, - / -, -). The caller resized to 22×22.
pub fn kill312_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let signs = [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)];
    for pass in 0..2 {
        let scale = if pass == 1 { 0.8 } else { 0.4 };
        for (sx, sy) in signs {
            let ty = rng.next_range(61, 64);
            let idx = gore.new_gore(&mut crate::projectile::DynGoreRng(rng), world, p.pos, Vec2::ZERO, ty, 1.0);
            let g = gore.slot_mut(idx);
            g.velocity *= scale;
            g.velocity.x += sx;
            g.velocity.y += sy;
        }
    }
}

/// Site 128 — `Kill()`'s 246 gore (`Projectile.cs:76960-76963`): the
/// wave-G 483 row verbatim (×0.3 + integer `Next(-1,2)` kicks), keep-one.
pub fn kill246_gore(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    kill483_gore(p, gore, rng, world);
}

/// Site 129 — `Kill()`'s 249 gore (`Projectile.cs:77003-77006`): ONE gore
/// at the position, type `Next(61, 64)`; post `velocity *= 0.2` then the
/// integer `Next(-1, 2)` kicks.
pub fn kill249_gore(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let ty = rng.next_range(61, 64);
    let idx = gore.new_gore(&mut crate::projectile::DynGoreRng(rng), world, p.pos, Vec2::ZERO, ty, 1.0);
    let g = gore.slot_mut(idx);
    g.velocity *= 0.2;
    g.velocity.x += rng.next_range(-1, 2) as f32;
    g.velocity.y += rng.next_range(-1, 2) as f32;
}

/// Sites 130/131 — `Kill()`'s 370 (`:77168-77174`) / 371 (`:77184-77191`)
/// gores: 30 each. Per gore: the direction `(Next(-10,11), Next(-10,11))`
/// normalized (the 371 scales it ×0.4), position `Center + dir*10`,
/// velocity `dir * Next(4,9) * 0.66 + UnitY*1.5`, the 370 type 331 with
/// scale `Next(40,141)*0.01`, the 371 type `Next(435,438)` with scale
/// `Next(20,100)*0.01`; post `sticky = false`. One keep-one fn, the 371
/// flag picks the type/scale/vector differences.
pub fn duck_gores(
    p: &Projectile,
    is_371: bool,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let c = center(p);
    for _ in 0..30 {
        let mut dir = Vec2::new(
            rng.next_range(-10, 11) as f32,
            rng.next_range(-10, 11) as f32,
        );
        dir = dir.normalize_or_zero();
        if is_371 {
            dir *= 0.4;
        }
        let speed = rng.next_range(4, 9) as f32 * 0.66;
        let vel = dir * speed + Vec2::new(0.0, 1.5);
        let (ty, scale) = if is_371 {
            (rng.next_range(435, 438), rng.next_range(20, 100) as f32 * 0.01)
        } else {
            (331, rng.next_range(40, 141) as f32 * 0.01)
        };
        let idx = gore.new_gore(&mut crate::projectile::DynGoreRng(rng), world, c + dir * 10.0, vel, ty, scale);
        gore.slot_mut(idx).sticky = false;
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

    /// `Projectile.cs:76912-76935` — the 312 block: 8 gores, the scale
    /// (0.4 pass 1, 0.8 pass 2) BEFORE the ±1 signs.
    #[test]
    fn w4_g16_the_312_two_passes_scale_before_signs() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(85);
        kill312_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(85);
        let signs = [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)];
        for pass in 0..2 {
            let scale = if pass == 1 { 0.8 } else { 0.4 };
            for (i, (sx, sy)) in signs.iter().enumerate() {
                let _ = twin.next_range(61, 64);
                let ky = twin.next_range(10, 31);
                let kx = twin.next_range(-20, 21);
                let kicked = Vec2::new(kx as f32 * 0.1, -(ky as f32 * 0.1)) * scale;
                let idx = pass * 4 + i;
                assert_eq!(gore.slot(idx).velocity, kicked + Vec2::new(*sx, *sy));
            }
        }
        assert_eq!(rng.peek(), twin.peek());
    }

    /// `Projectile.cs:77003-77006` — the 249 gore: ×0.2 then integer kicks.
    #[test]
    fn w4_g16_the_249_gore_scales_by_fifth() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(87);
        kill249_gore(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(87);
        let _ = twin.next_range(61, 64);
        let ky = twin.next_range(10, 31);
        let kx = twin.next_range(-20, 21);
        let ix = twin.next_range(-1, 2);
        let iy = twin.next_range(-1, 2);
        assert_eq!(rng.peek(), twin.peek());
        let kicked = Vec2::new(kx as f32 * 0.1, -(ky as f32 * 0.1)) * 0.2;
        assert_eq!(gore.slot(0).velocity, kicked + Vec2::new(ix as f32, iy as f32));
    }

    /// `Projectile.cs:77168-77174` / `:77184-77191` — the duck gores: per
    /// gore two Next(-10,11) + Next(4,9) + the scale roll (+ the 371 type
    /// roll) + kicks; sticky off.
    #[test]
    fn w4_g16_the_duck_gores_draw_direction_speed_scale() {
        let w = air();
        let p = proj();
        for is_371 in [false, true] {
            let mut gore = GorePool::new();
            let mut rng = UnifiedRandom::new(89);
            duck_gores(&p, is_371, &mut gore, &mut rng, &w);
            let mut twin = UnifiedRandom::new(89);
            for _ in 0..30 {
                let _ = twin.next_range(-10, 11);
                let _ = twin.next_range(-10, 11);
                let _ = twin.next_range(4, 9);
                if is_371 {
                    let _ = twin.next_range(435, 438);
                    let _ = twin.next_range(20, 100);
                } else {
                    let _ = twin.next_range(40, 141);
                }
                let _ = twin.next_range(10, 31);
                let _ = twin.next_range(-20, 21);
            }
            assert_eq!(rng.peek(), twin.peek(), "is_371={is_371}");
            let active = gore.slots()[..30].iter().filter(|g| g.active).count();
            assert_eq!(active, 30);
            for g in &gore.slots()[..30] {
                assert!(!g.sticky);
                if !is_371 {
                    assert_eq!(g.gore_type, 331);
                }
            }
        }
    }

    /// The 246 delegation forwards to the wave-G 483 row.
    #[test]
    fn w4_g16_the_246_delegation_matches_the_483_row() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(91);
        kill246_gore(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(91);
        let _ = twin.next_range(61, 64);
        let _ = twin.next_range(10, 31);
        let _ = twin.next_range(-20, 21);
        let _ = twin.next_range(-1, 2);
        let _ = twin.next_range(-1, 2);
        assert_eq!(rng.peek(), twin.peek());
    }
}
