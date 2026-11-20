//! `Projectile.cs`'s `Gore.NewGore*` spawn sites — wave N: sites 142-151 of
//! `Kill_Bombs_DoUsualKillCode` (`:78012`-`:78199`).
//!
//! The pool + the wave-A contract live in `projectile_gore_a.rs` (W4-g16,
//! the `npc_hit_effect_a.rs` precedent). Dust bodies are the dust lanes'
//! skip; argument draws precede the pool call, kicks roll inside
//! `new_gore`, post-writes land on the returned slot (sentinel included).
//!
//! ## Scope — sites 142-151 (source order)
//!
//! | # | C# site | arm |
//! | --- | --- | --- |
//! | 142 | `:78012` | 779/783/862/863 rocket corners (the wave-E row) |
//! | 143 | `:78057` | 776-804 cluster, 4 corners at ×0.8 only |
//! | 144-147 | `:78124`-`:78136` | 133-930 dynamite, the wave-L two-pass row |
//! | 148-151 | `:78187`-`:78199` | 139-341 sticky bomb, 12-gore offset row |
//!
//! ## RNG notes specific to this wave
//!
//! * The rocket corners (142) are the wave-E `rocket_corner_gores` row
//!   verbatim (4 × `Next(5)` gate, `NewGoreDirect`, ×0.2 / scale×0.65 /
//!   ±0.5 signs) — keep-one.
//! * The cluster (143) runs the two-pass loop with `num49 = 2..=2` — ONE
//!   pass, so the scale is ALWAYS 0.8 (the `num49 == 1` arm is dead). Four
//!   corners, scale-before-signs. A distinct fn (the two-pass row would
//!   spawn eight).
//! * The dynamite (144-147) is the wave-L `kill312_gores` row verbatim
//!   (two passes 0.4/0.8 × four corners) — keep-one.
//! * The sticky bomb (148-151) is a THREE-pass block at the OFFSET position
//!   `pos + (width/2 - 24, height/2 - 24)` (INTEGER division; the caller
//!   resized to 80×80), scales 0.33/0.66/1.0 (NOT the bundle's thirds),
//!   then ±1 signs. A distinct fn (offset + different scale values).
//!
//! See docs/ANALYSIS.md "The Projectile gore sites (W4-g16)".
#![allow(dead_code)] // wiring deferred (code-only mode)

use crate::projectile::Projectile;
use crate::projectile_gore_e::rocket_corner_gores;
use crate::projectile_gore_l::kill312_gores;
use glam::Vec2;
use tr_core::gore::GorePool;
use tr_core::{UnifiedRandom, World};

/// Site 142 — the 779/783/862/863 rocket corners (`Projectile.cs:78006-
/// 78018`): the wave-E rocket row verbatim, keep-one.
pub fn rocket_bomb_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    rocket_corner_gores(p, gore, rng, world);
}

/// Site 143 — the 776-804 cluster's four corners (`Projectile.cs:78051-
/// 78062`): the two-pass loop with `num49 = 2..=2` — ONE pass at scale 0.8,
/// four `NewGoreDirect(pos, Zero, Next(61,64))`, scale-before-signs ±1.
pub fn cluster_bomb_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let signs = [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)];
    for (sx, sy) in signs {
        let ty = rng.next_range(61, 64);
        let idx = gore.new_gore_direct(rng, world, p.pos, Vec2::ZERO, ty, 1.0);
        let g = gore.slot_mut(idx);
        g.velocity *= 0.8;
        g.velocity.x += sx;
        g.velocity.y += sy;
    }
}

/// Sites 144-147 — the 133-930 dynamite's eight-gore two-pass block
/// (`Projectile.cs:78117-78140`): the wave-L `kill312_gores` row verbatim,
/// keep-one.
pub fn dynamite_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    kill312_gores(p, gore, rng, world);
}

/// Sites 148-151 — the 139-341 sticky bomb's twelve-gore three-pass block
/// (`Projectile.cs:78176-78203`): `num67` 0..2, scales 0.33/0.66/1.0, each
/// four corners at the OFFSET position `pos + (width/2 - 24, height/2 -
/// 24)` (INTEGER division), `NewGore`, scale-before-signs ±1. The caller
/// resized to 80×80 first.
pub fn sticky_bomb_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let pos = Vec2::new(
        p.pos.x + (p.w as i32 / 2) as f32 - 24.0,
        p.pos.y + (p.h as i32 / 2) as f32 - 24.0,
    );
    let signs = [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)];
    for pass in 0..3 {
        let scale = match pass {
            0 => 0.33,
            1 => 0.66,
            _ => 1.0,
        };
        for (sx, sy) in signs {
            let ty = rng.next_range(61, 64);
            let idx = gore.new_gore(rng, world, pos, Vec2::ZERO, ty, 1.0);
            let g = gore.slot_mut(idx);
            g.velocity *= scale;
            g.velocity.x += sx;
            g.velocity.y += sy;
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
        p.w = 80.0;
        p.h = 80.0;
        p
    }

    /// `Projectile.cs:78051-78062` — the cluster: ONE pass at ×0.8, four
    /// corners scale-before-signs.
    #[test]
    fn w4_g16_the_cluster_bomb_single_pass_at_eight_tenths() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(99);
        cluster_bomb_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(99);
        let signs = [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)];
        for (i, (sx, sy)) in signs.iter().enumerate() {
            let _ = twin.next_range(61, 64);
            let ky = twin.next_range(10, 31);
            let kx = twin.next_range(-20, 21);
            let kicked = Vec2::new(kx as f32 * 0.1, -(ky as f32 * 0.1)) * 0.8;
            assert_eq!(gore.slot(i).velocity, kicked + Vec2::new(*sx, *sy));
        }
        assert_eq!(rng.peek(), twin.peek());
        assert_eq!(gore.slots()[..4].iter().filter(|g| g.active).count(), 4);
    }

    /// `Projectile.cs:78176-78203` — the sticky bomb: 12 gores at the
    /// offset position, three passes 0.33/0.66/1.0.
    #[test]
    fn w4_g16_the_sticky_bomb_offset_three_passes() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(101);
        sticky_bomb_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(101);
        let signs = [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)];
        // offset = pos + (80/2 - 24) = pos + 16.
        let off = Vec2::new(116.0, 116.0);
        for pass in 0..3 {
            let scale = match pass { 0 => 0.33, 1 => 0.66, _ => 1.0 };
            for (i, (sx, sy)) in signs.iter().enumerate() {
                let _ = twin.next_range(61, 64);
                let ky = twin.next_range(10, 31);
                let kx = twin.next_range(-20, 21);
                let kicked = Vec2::new(kx as f32 * 0.1, -(ky as f32 * 0.1)) * scale;
                let idx = pass * 4 + i;
                assert_eq!(gore.slot(idx).position, off);
                assert_eq!(gore.slot(idx).velocity, kicked + Vec2::new(*sx, *sy));
            }
        }
        assert_eq!(rng.peek(), twin.peek());
        assert_eq!(gore.slots()[..12].iter().filter(|g| g.active).count(), 12);
    }

    /// The keep-one delegations: rocket = the gated corner row, dynamite =
    /// the two-pass row.
    #[test]
    fn w4_g16_the_bomb_delegations_n_match_their_rows() {
        let w = air();
        let p = proj();
        // Rocket: 4 Next(5) gates always draw.
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(103);
        rocket_bomb_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(103);
        for _ in 0..4 {
            if twin.next_max(5) == 0 {
                let _ = twin.next_range(61, 64);
                let _ = twin.next_range(10, 31);
                let _ = twin.next_range(-20, 21);
            }
        }
        assert_eq!(rng.peek(), twin.peek());
        // Dynamite: 8 gores, two passes.
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(105);
        dynamite_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(105);
        for _ in 0..8 {
            let _ = twin.next_range(61, 64);
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek());
    }
}
