//! `Projectile.cs`'s `Gore.NewGore*` spawn sites — wave O: sites 152-164,
//! the LAST thirteen of `Kill_Bombs_DoUsualKillCode` (`:78292`-`:78493`).
//! This completes the 165-site `Projectile.cs` census.
//!
//! The pool + the wave-A contract live in `projectile_gore_a.rs` (W4-g16,
//! the `npc_hit_effect_a.rs` precedent). Dust bodies are the dust lanes'
//! skip; argument draws precede the pool call, kicks roll inside
//! `new_gore`, post-writes land on the returned slot (sentinel included).
//!
//! ## Scope — sites 152-164 (source order)
//!
//! | # | C# site | arm |
//! | --- | --- | --- |
//! | 152 | `:78292` | 773 scarab bomb, 4 gores + dig-direction push |
//! | 153-156 | `:78351`-`:78363` | 28/30/37/75/102/164/397/516-519 bomb corners |
//! | 157-160 | `:78420`-`:78432` | 29/108/470/637/1002/1086/1087 big bomb, 2×4 |
//! | 161-164 | `:78472`-`:78493` | 588 confetti bomb, 4 independent gates |
//!
//! (The big-bomb block is 4 call sites run twice — 8 spawn events; the
//! confetti block 4 — 13 call sites total, the census tail kept intact.)
//!
//! ## RNG notes specific to this wave
//!
//! * The scarab (152): ONE pass, four `NewGoreDirect(Center - (20,20),
//!   Zero, Next(61,64))`; post `velocity = velocity*0.5 + digDir*3` then
//!   `+= sign*0.2`. `digDir` is `(num70 - PI/2).ToRotationVector2()` =
//!   the normalized dig-direction snap — the caller resolves it (the
//!   `GetScarabBombDigDirectionSnap8` seam) and passes the unit vector.
//! * The bomb corners (153-156) are the wave-J `corner_gores(p, 1.0)` row
//!   verbatim (×0.4 then ±1) — keep-one.
//! * The big bomb (157-160) is the wave-I `lava_boulder_gores` row run
//!   TWICE (the `num94 < 2` loop): offset position, scale 1.5, ±1.5 signs,
//!   NO velocity scale. Keep-one ×2. The caller resized (29: 200, 1086/
//!   1087: 240) first.
//! * The confetti bomb (161-164): FOUR INDEPENDENT `Next(2)` gates (not a
//!   loop), each `Next(276,283)` type (confetti, NOT 61-64), ×0.4 then the
//!   ±1 sign. A distinct fn — the gate count and type range differ from
//!   every corner row.
//!
//! See docs/ANALYSIS.md "The Projectile gore sites (W4-g16)".
#![allow(dead_code)] // wiring deferred (code-only mode)

use crate::projectile::Projectile;
use crate::projectile_gore_i::lava_boulder_gores;
use crate::projectile_gore_j::corner_gores;
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

/// Site 152 — the 773 scarab bomb's four gores (`Projectile.cs:78286-
/// 78297`): ONE pass, four `NewGoreDirect(Center - (20,20), Zero,
/// Next(61,64))`; post `velocity = velocity*0.5 + dig_dir*3` then
/// `+= sign*0.2`. `dig_dir` is the normalized dig-direction snap (the
/// caller's `GetScarabBombDigDirectionSnap8` seam, `:78285`'s vector3).
pub fn scarab_bomb_gores(
    p: &Projectile,
    dig_dir: Vec2,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let pos = center(p) - Vec2::splat(20.0);
    let signs = [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)];
    for (sx, sy) in signs {
        let ty = rng.next_range(61, 64);
        let idx = gore.new_gore_direct(rng, world, pos, Vec2::ZERO, ty, 1.0);
        let g = gore.slot_mut(idx);
        g.velocity = g.velocity * 0.5 + dig_dir * 3.0;
        g.velocity += Vec2::new(sx, sy) * 0.2;
    }
}

/// Sites 153-156 — the 28/30/37/75/102/164/397/516-519 bomb's four corners
/// (`Projectile.cs:78351-78366`): the wave-J corner row at magnitude 1.0,
/// keep-one.
pub fn bomb_corner_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    corner_gores(p, 1.0, gore, rng, world);
}

/// Sites 157-160 — the 29/108/470/637/1002/1086/1087 big bomb's eight gores
/// (`Projectile.cs:78418-78436`): the `num94 < 2` loop runs the wave-I
/// `lava_boulder_gores` row TWICE (offset position, scale 1.5, ±1.5 signs,
/// no velocity scale), keep-one ×2.
pub fn big_bomb_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    lava_boulder_gores(p, gore, rng, world);
    lava_boulder_gores(p, gore, rng, world);
}

/// Sites 161-164 — the 588 confetti bomb's four gores (`Projectile.cs:
/// 78470-78497`): FOUR INDEPENDENT `Next(2)` gates, each taken →
/// `NewGore(pos, Zero, Next(276,283))`, post `velocity *= 0.4` then the
/// ±1 sign (+,+ / -, + / +, - / -, -). The type range is the confetti
/// 276-283, NOT the corner 61-64. The caller recentered first (`position =
/// Center; width = height = 22; Center = position`, `:78452-78454`), so
/// `p.pos` here IS the pre-resize center.
pub fn confetti_bomb_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let signs = [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)];
    for (sx, sy) in signs {
        if rng.next_max(2) != 0 {
            continue;
        }
        let ty = rng.next_range(276, 283);
        let idx = gore.new_gore(rng, world, p.pos, Vec2::ZERO, ty, 1.0);
        let g = gore.slot_mut(idx);
        g.velocity *= 0.4;
        g.velocity.x += sx;
        g.velocity.y += sy;
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

    /// `Projectile.cs:78286-78297` — the scarab: 4 gores at Center−20, the
    /// velocity = kicked×0.5 + digDir×3 + sign×0.2 postlude.
    #[test]
    fn w4_g16_the_scarab_pushes_along_the_dig_direction() {
        let w = air();
        let p = proj();
        let dig = Vec2::new(1.0, 0.0);
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(107);
        scarab_bomb_gores(&p, dig, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(107);
        let signs = [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)];
        for (i, (sx, sy)) in signs.iter().enumerate() {
            let _ = twin.next_range(61, 64);
            let ky = twin.next_range(10, 31);
            let kx = twin.next_range(-20, 21);
            let kicked = Vec2::new(kx as f32 * 0.1, -(ky as f32 * 0.1));
            let expect = kicked * 0.5 + dig * 3.0 + Vec2::new(*sx, *sy) * 0.2;
            assert_eq!(gore.slot(i).velocity, expect);
            assert_eq!(gore.slot(i).position, Vec2::new(91.0, 91.0)); // center 111 - 20
        }
        assert_eq!(rng.peek(), twin.peek());
    }

    /// `Projectile.cs:78418-78436` — the big bomb: 8 gores (two lava-boulder
    /// passes), scale 1.5, ±1.5 signs, no velocity scale.
    #[test]
    fn w4_g16_the_big_bomb_runs_the_boulder_row_twice() {
        let w = air();
        let mut p = proj();
        p.w = 200.0;
        p.h = 200.0;
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(109);
        big_bomb_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(109);
        let signs = [(1.5, 1.5), (-1.5, 1.5), (1.5, -1.5), (-1.5, -1.5)];
        for pass in 0..2 {
            for (i, (sx, sy)) in signs.iter().enumerate() {
                let _ = twin.next_range(61, 64);
                let ky = twin.next_range(10, 31);
                let kx = twin.next_range(-20, 21);
                let kicked = Vec2::new(kx as f32 * 0.1, -(ky as f32 * 0.1));
                let idx = pass * 4 + i;
                assert_eq!(gore.slot(idx).scale, 1.5);
                assert_eq!(gore.slot(idx).velocity, kicked + Vec2::new(*sx, *sy));
            }
        }
        assert_eq!(rng.peek(), twin.peek());
        assert_eq!(gore.slots()[..8].iter().filter(|g| g.active).count(), 8);
    }

    /// `Projectile.cs:78470-78497` — the confetti bomb: four independent
    /// Next(2) gates, confetti types 276-283, ×0.4 then ±1.
    #[test]
    fn w4_g16_the_confetti_bomb_four_independent_gates() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(111);
        confetti_bomb_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(111);
        let signs = [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)];
        let mut spawned = 0usize;
        for (sx, sy) in signs.iter() {
            if twin.next_max(2) == 0 {
                let ty = twin.next_range(276, 283);
                let ky = twin.next_range(10, 31);
                let kx = twin.next_range(-20, 21);
                let kicked = Vec2::new(kx as f32 * 0.1, -(ky as f32 * 0.1)) * 0.4;
                assert_eq!(gore.slot(spawned).gore_type, ty);
                assert!((276..283).contains(&gore.slot(spawned).gore_type));
                assert_eq!(gore.slot(spawned).velocity, kicked + Vec2::new(*sx, *sy));
                spawned += 1;
            }
        }
        assert_eq!(rng.peek(), twin.peek());
    }

    /// The bomb-corner delegation forwards to the wave-J corner row.
    #[test]
    fn w4_g16_the_bomb_corner_delegation_matches_the_row() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(113);
        bomb_corner_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(113);
        for _ in 0..4 {
            let _ = twin.next_range(61, 64);
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek());
        assert_eq!(gore.slots()[..4].iter().filter(|g| g.active).count(), 4);
    }

    // ---- W4-g16 SEAL compose witnesses (research/360 §3, WRITTEN NOT RUN) ----
    // Each pins a g16 body fn's draw row against the g65 `gore_census` stub
    // shape it supersedes at merge (projectile_death_dust.rs @ f6e07bb90):
    // the stub's let-_ discards become these real spawns, draw-for-draw.

    /// The plain `gore_census(rng, n, false, false)` stub (g65 :4756) vs the
    /// celeb2 kill row (`kill_row_gores`, the 645/452/454/448/459 hosts):
    /// Next(100) X/Y pair, Next(61,64) type, the kicks Next(10,31) Y then
    /// Next(-20,21) X, the two Next(-10,11) nudges - seven ranges per gore.
    #[test]
    fn w4_g16_seal_the_g65_plain_census_stub_matches_the_celeb2_kill_row() {
        let w = air();
        let mut p = proj();
        p.w = 144.0;
        p.h = 144.0;
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(211);
        crate::projectile_gore_g::kill_row_gores(&p, 3, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(211);
        for _ in 0..3 {
            let _ = twin.next_max(100);
            let _ = twin.next_max(100);
            let _ = twin.next_range(61, 64);
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
            let _ = twin.next_range(-10, 11);
            let _ = twin.next_range(-10, 11);
        }
        assert_eq!(rng.peek(), twin.peek(), "the 454 three-row census");
        assert_eq!(gore.slots()[..3].iter().filter(|g| g.active).count(), 3);
    }

    /// The scatter+lift stub (`gore_census(rng, ngore, true, true)`, g65's
    /// dd2_explosion_dust :269) vs `dd2_explosion_gores` (:71888-71900):
    /// the plain row, then UnitY.RotatedByRandom(PI) (TWO NextDouble) +
    /// NextDouble radius, then the Next(2) Y-drop gate, then the nudges.
    /// Count 4, or 8 with setApprenticeT3 (the caller's flag).
    #[test]
    fn w4_g16_seal_the_g65_scatter_lift_stub_matches_the_dd2_row() {
        let w = air();
        let mut p = proj();
        p.w = 96.0;
        p.h = 96.0;
        for count in [4, 8] {
            let mut gore = GorePool::new();
            let mut rng = UnifiedRandom::new(223);
            crate::projectile_gore_f::dd2_explosion_gores(&p, count, &mut gore, &mut rng, &w);
            let mut twin = UnifiedRandom::new(223);
            for _ in 0..count {
                let _ = twin.next_max(100);
                let _ = twin.next_max(100);
                let _ = twin.next_range(61, 64);
                let _ = twin.next_range(10, 31);
                let _ = twin.next_range(-20, 21);
                let _ = twin.next_double(); // RotatedByRandom draw 1
                let _ = twin.next_double(); // RotatedByRandom draw 2
                let _ = twin.next_double(); // the radius
                let _ = twin.next_max(2); // the Y-drop gate
                let _ = twin.next_range(-10, 11);
                let _ = twin.next_range(-10, 11);
            }
            assert_eq!(rng.peek(), twin.peek(), "the dd2 census at count {count}");
            assert_eq!(
                gore.slots()[..count as usize].iter().filter(|g| g.active).count(),
                count as usize
            );
        }
    }

    /// The scatter-only stub (`gore_census(rng, 2, true, false)`, g65's 617
    /// :2152 / 467 :2458 / 468 :2492) vs `kill608_617_gores` (the
    /// fire_explosion_gores keep-one): the plain row + the three-double
    /// scatter, NO lift gate, then the nudges.
    #[test]
    fn w4_g16_seal_the_g65_scatter_stub_matches_the_617_pair() {
        let w = air();
        let mut p = proj();
        p.w = 176.0;
        p.h = 176.0;
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(227);
        crate::projectile_gore_f::kill608_617_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(227);
        for _ in 0..2 {
            let _ = twin.next_max(100);
            let _ = twin.next_max(100);
            let _ = twin.next_range(61, 64);
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
            let _ = twin.next_double();
            let _ = twin.next_double();
            let _ = twin.next_double();
            let _ = twin.next_range(-10, 11);
            let _ = twin.next_range(-10, 11);
        }
        assert_eq!(rng.peek(), twin.peek(), "the 617 scatter census");
        assert_eq!(gore.slots()[..2].iter().filter(|g| g.active).count(), 2);
    }

    /// The confetti stub (g65's confetti_burst :4968 / the 284 inline
    /// :3804) vs `confetti_gores_vel`: Next(276,283) type, the two kicks,
    /// then the ADD-then-MULTIPLY ladder (four Next(-50,51)), the
    /// Next(-20,21) scale, the two Next(-50,51) nudges - ten ranges.
    /// The 178 passes the velocity, the 284 passes -vel*0.3.
    #[test]
    fn w4_g16_seal_the_g65_confetti_stub_matches_the_178_284_rows() {
        let w = air();
        let p = proj();
        for (count, vel) in [(40, p.vel), (5, -p.vel * 0.3)] {
            let mut gore = GorePool::new();
            let mut rng = UnifiedRandom::new(229);
            crate::projectile_gore_h::confetti_gores_vel(&p, count, vel, &mut gore, &mut rng, &w);
            let mut twin = UnifiedRandom::new(229);
            for _ in 0..count {
                let _ = twin.next_range(276, 283);
                let _ = twin.next_range(10, 31);
                let _ = twin.next_range(-20, 21);
                let _ = twin.next_range(-50, 51);
                let _ = twin.next_range(-50, 51);
                let _ = twin.next_range(-50, 51);
                let _ = twin.next_range(-50, 51);
                let _ = twin.next_range(-20, 21);
                let _ = twin.next_range(-50, 51);
                let _ = twin.next_range(-50, 51);
            }
            assert_eq!(rng.peek(), twin.peek(), "the confetti census at count {count}");
            assert_eq!(
                gore.slots()[..count as usize].iter().filter(|g| g.active).count(),
                count as usize
            );
        }
    }

    /// The corner stub (g65's 681 :4409 / 240 :3368 inline 4x3 loops) vs
    /// `goblin_bomb_gores` / `kill240_corner_gores`: per gore Next(61,64)
    /// type + the two kicks; the sign postlude draws NOTHING.
    #[test]
    fn w4_g16_seal_the_g65_corner_stub_matches_the_681_240_fours() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(233);
        crate::projectile_gore_f::goblin_bomb_gores(&p, &mut gore, &mut rng, &w);
        crate::projectile_gore_j::kill240_corner_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(233);
        for _ in 0..8 {
            let _ = twin.next_range(61, 64);
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the two corner fours");
        assert_eq!(gore.slots()[..8].iter().filter(|g| g.active).count(), 8);
    }

    /// The star stub (g65's 955 :3896 / 12 :3742 inline loops) vs
    /// `kill12_star_gores`: per gore ONE NextFloat (the CircularEdge unit),
    /// the SelectRandom(16,17x7) Next(8) pick, the two kicks. The g65 stub
    /// folded the screen gate true; the g16 body gates on the real padded
    /// screen (the caller's `trail_on_screen`) - the drop makes the
    /// off-screen case C#-correct (zero rows).
    #[test]
    fn w4_g16_seal_the_g65_star_stub_matches_the_955_seven() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(239);
        crate::projectile_gore_j::kill12_star_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(239);
        for _ in 0..7 {
            let _ = twin.next_float(); // CircularEdge unit
            let _ = twin.next_range(0, 8); // SelectRandom(16,17x7)
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the 955 star census");
        assert_eq!(gore.slots()[..7].iter().filter(|g| g.active).count(), 7);
    }
}
