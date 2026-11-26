//! `Projectile.cs`'s `Gore.NewGore*` spawn sites — wave M: sites 132-141,
//! the first ten of `Kill_Bombs_DoUsualKillCode` (`:77674`-`:77972`).
//!
//! The pool + the wave-A contract live in `projectile_gore_a.rs` (W4-g16,
//! the `npc_hit_effect_a.rs` precedent). Dust bodies are the dust lanes'
//! skip; argument draws precede the pool call, kicks roll inside
//! `new_gore`, post-writes land on the returned slot (sentinel included).
//!
//! ## Scope — sites 132-141 (source order)
//!
//! | # | C# site | arm |
//! | --- | --- | --- |
//! | 132-135 | `:77674`-`:77686` | 1077 frost bomb, 4 corners (the wave-J row) |
//! | 136 | `:77721` | 910/911 dirt bomb, 8-gore two-pass (the wave-L row) |
//! | 137 | `:77765` | 784-903 water bomb, the same two-pass row |
//! | 138 | `:77804` | 787-904 lava bomb, the same two-pass row |
//! | 139 | `:77843` | 790-905 honey bomb, the same two-pass row |
//! | 140 | `:77882` | 799-906 dry bomb, the same two-pass row |
//! | 141 | `:77972` | 793-809 bomb bundle, 12-gore three-pass |
//!
//! ## RNG notes specific to this wave
//!
//! * The bomb family's gore blocks are all `NewGore(Direct)(pos, Zero,
//!   Next(61,64))` with a scale-then-sign postlude; only the pass count,
//!   the scale values and the sign magnitude differ. The 4-corner ×0.4
//!   block is the wave-J `corner_gores(p, 1.0)`; the 8-gore two-pass
//!   (0.4/0.8) block is the wave-L `kill312_gores`. Both are keep-ones.
//! * The bomb bundle (141) is a THREE-pass block: `num38` 1..3, scale
//!   `num38/3` (1/3, 2/3, 1), then the ±1 signs — 12 gores. A distinct fn
//!   (the two-pass row cannot express the third pass / fractional scale).
//! * Every block's caller ran `Resize(22, 22)` (or the bundle's 50×50)
//!   first; the dust loops before each are the dust lane's.
//!
//! See docs/ANALYSIS.md "The Projectile gore sites (W4-g16)".
#![allow(dead_code)] // wiring deferred (code-only mode)

use crate::projectile::Projectile;
use crate::projectile_gore_j::corner_gores;
use crate::projectile_gore_l::kill312_gores;
use glam::Vec2;
use tr_core::gore::GorePool;
use tr_core::World;
use tr_core::loot::DropRng;
#[cfg(test)]
use tr_core::UnifiedRandom;

/// Sites 132-135 — the 1077 frost bomb's four corners (`Projectile.cs:
/// 77674-77689`): the wave-J corner row at magnitude 1.0, keep-one.
pub fn frost_bomb_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    corner_gores(p, 1.0, gore, rng, world);
}

/// Sites 136-140 — the dirt/water/lava/honey/dry bombs' eight-gore two-pass
/// block (`Projectile.cs:77715-77726` / `:77759-77770` / `:77798-77809` /
/// `:77837-77848` / `:77876-77887`): the wave-L `kill312_gores` row
/// verbatim (two passes 0.4/0.8 × four corners, scale before signs),
/// keep-one. The five arms differ only in their dust type / spread
/// delegate, not the gore block.
pub fn fluid_bomb_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    kill312_gores(p, gore, rng, world);
}

/// Site 141 — the 793-809 bomb bundle's twelve-gore three-pass block
/// (`Projectile.cs:77966-77977`): `num38` 1..3, each four corners
/// `NewGoreDirect(pos, Zero, Next(61,64))`, post `velocity *= num38/3`
/// then the ±1 signs. The caller ran `Resize(50, 50)` first.
pub fn bomb_bundle_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let signs = [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)];
    for pass in 1..=3 {
        let scale = pass as f32 / 3.0;
        for (sx, sy) in signs {
            let ty = rng.next_range(61, 64);
            let idx = gore.new_gore_direct(&mut crate::projectile::DynGoreRng(rng), world, p.pos, Vec2::ZERO, ty, 1.0);
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
        p.w = 22.0;
        p.h = 22.0;
        p
    }

    /// `Projectile.cs:77966-77977` — the bundle: 12 gores, three passes at
    /// scale 1/3, 2/3, 1, each four corners scale-before-signs.
    #[test]
    fn w4_g16_the_bomb_bundle_three_passes_fractional_scale() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(93);
        bomb_bundle_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(93);
        let signs = [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)];
        for pass in 1..=3 {
            let scale = pass as f32 / 3.0;
            for (i, (sx, sy)) in signs.iter().enumerate() {
                let _ = twin.next_range(61, 64);
                let ky = twin.next_range(10, 31);
                let kx = twin.next_range(-20, 21);
                let kicked = Vec2::new(kx as f32 * 0.1, -(ky as f32 * 0.1)) * scale;
                let idx = (pass - 1) * 4 + i;
                assert_eq!(gore.slot(idx).velocity, kicked + Vec2::new(*sx, *sy));
            }
        }
        assert_eq!(rng.peek(), twin.peek());
        let active = gore.slots()[..12].iter().filter(|g| g.active).count();
        assert_eq!(active, 12);
    }

    /// The keep-one delegations: the frost bomb = the corner row (4 gores),
    /// the fluid bombs = the two-pass row (8 gores).
    #[test]
    fn w4_g16_the_bomb_delegations_match_their_rows() {
        let w = air();
        let p = proj();
        // Frost: 4 gores, ×0.4 then ±1.
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(95);
        frost_bomb_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(95);
        for _ in 0..4 {
            let _ = twin.next_range(61, 64);
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek());
        assert_eq!(gore.slots()[..4].iter().filter(|g| g.active).count(), 4);
        // Fluid: 8 gores, two passes.
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(97);
        fluid_bomb_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(97);
        for _ in 0..8 {
            let _ = twin.next_range(61, 64);
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek());
        assert_eq!(gore.slots()[..8].iter().filter(|g| g.active).count(), 8);
    }
}
