//! `Projectile.cs`'s `Gore.NewGore*` spawn sites — wave F: sites 52-63 in
//! source order (`:71617`, `:71890`, `:71928`-`:71940`, `:72120`,
//! `:72219`, `:72321`, `:72634`-`:72638`). The 681 goblin-bomb four
//! (`:71928-71940`) and the 590 triple (`:72634-72638`) are consecutive
//! statement blocks kept intact (the wave-E mirror precedent) — 12 sites.
//!
//! The pool + the wave-A contract live in `projectile_gore_a.rs` (W4-g16,
//! the `npc_hit_effect_a.rs` precedent). Dust bodies are the dust lanes'
//! skip; argument draws precede the pool call, kicks roll inside
//! `new_gore`, post-writes land on the returned slot (sentinel included).
//!
//! ## Scope — sites 52-63 (source order)
//!
//! | # | C# site | arm |
//! | --- | --- | --- |
//! | 52 | `:71617` | `Kill()` 856 Stellar Tune gore |
//! | 53 | `:71890` | `Kill()` 664/666/668/706 DD2 loop (num127) |
//! | 54-57 | `:71928`-`:71940` | `Kill()` 681 goblin-bomb corners |
//! | 58 | `:72120` | `Kill()` 645 gore (the wave-C row) |
//! | 59 | `:72219` | `Kill()` 608 gores ×2 (the wave-B row) |
//! | 60 | `:72321` | `Kill()` 617 gores ×2 (the wave-B row) |
//! | 61-63 | `:72634`-`:72638` | `Kill()` 590 triple (375/61 base) |
//!
//! ## RNG notes specific to this wave
//!
//! * The 856 gore (52) has NO `Next` gate — only the screen intersect; the
//!   `SelectRandom(16)` STILL draws `Next(1)` (`Utils.cs:2459-2462` picks
//!   `choices[Next(len)]` even for a one-element list).
//! * The DD2 loop (53) is the fire-explosion row (wave B) PLUS `position
//!   -= (16,16)` and a `Next(2)` Y-drop gate (`:71892-71896`); the count
//!   `num127` (4, or 8 with `setApprenticeT3`) is the caller's input — the
//!   flag also resizes the box BEFORE the dust loops (`:71844-71846`).
//! * The goblin-bomb corners (54-57) read the RESIZED 22×22 position (the
//!   caller ran `:71904-71910` first); the four sign pairs are data.
//! * The 590 triple (61-63) draws ONE `NextFloat` for the shared scale
//!   (`:72628`) BEFORE the first gore; the base type is 375 (61 when
//!   `ai[2] == 1`) — the 375-377 sticky arm's alpha=100 is the pool's.
//!
//! See docs/ANALYSIS.md "The Projectile gore sites (W4-g16)".
#![allow(dead_code)] // wiring deferred (code-only mode)

use crate::projectile::Projectile;
use crate::projectile_gore_b::fire_explosion_gores;
use crate::projectile_gore_c::celeb2_gore;
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

/// `Utils.RotatedBy` (`Utils.cs:1675-1684`), centre (0,0).
fn rotated_by(v: Vec2, radians: f64) -> Vec2 {
    let c = radians.cos() as f32;
    let s = radians.sin() as f32;
    Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}

/// `Utils.RotatedByRandom` (`Utils.cs:1697-1700`) — TWO `NextDouble`.
fn rotated_by_random(rng: &mut dyn DropRng, v: Vec2, max: f64) -> Vec2 {
    let a = rng.next_double() * max;
    let b = rng.next_double() * max;
    rotated_by(v, a - b)
}

/// `Utils.NextVector2Unit` (`Utils.cs:1261-1264`) — ONE `NextFloat`.
fn next_vector2_unit(rng: &mut dyn DropRng) -> Vec2 {
    let angle = std::f32::consts::PI * 2.0 * rng.next_float();
    Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32)
}

/// Site 52 — `Kill()`'s 856 Stellar Tune gore (`Projectile.cs:71612-
/// 71619`): the caller gates on the padded-screen intersect (NO Next
/// gate); velocity = `CircularEdge(0.5, 0.5) * 3` (ONE NextFloat), type
/// `SelectRandom(16)` — the one-element pick STILL draws `Next(1)`.
pub fn stellar_tune_gore(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let unit = next_vector2_unit(rng);
    let vel = Vec2::new(unit.x * 0.5, unit.y * 0.5) * 3.0;
    let _pick = rng.next_max(1); // SelectRandom(16) — Next(1), always 0
    gore.new_gore(&mut crate::projectile::DynGoreRng(rng), world, p.pos, vel, 16, 1.0);
}

/// Site 53 — `Kill()`'s 664/666/668/706 DD2 gore loop (`Projectile.cs:
/// 71888-71900`): `count` = num127 (4, or 8 with setApprenticeT3 — the
/// caller's, the box resize at `:71844-71846` precedes the dust loops).
/// Per gore: the fire-explosion argument row (Next(100) pair + Next(61,64)
/// + kicks), post `position = Center + UnitY.RotatedByRandom(PI) *
/// NextDouble * w/2` (three doubles), `position -= (16,16)`, the `Next(2)`
/// Y-drop gate, `velocity *= 0.3`, the two `Next(-10,11)*0.05` kicks.
pub fn dd2_explosion_gores(
    p: &Projectile,
    count: i32,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    for _ in 0..count {
        let off = Vec2::new(
            (p.w * rng.next_max(100) as f32) / 100.0,
            (p.h * rng.next_max(100) as f32) / 100.0,
        );
        let ty = rng.next_range(61, 64);
        let idx = gore.new_gore(&mut crate::projectile::DynGoreRng(rng), world, p.pos + off - Vec2::splat(10.0), Vec2::ZERO, ty, 1.0);
        let g = gore.slot_mut(idx);
        let dir = rotated_by_random(rng, Vec2::new(0.0, 1.0), std::f64::consts::PI);
        g.position = center(p) + dir * (rng.next_double() as f32 * p.w / 2.0);
        g.position -= Vec2::splat(16.0);
        if rng.next_max(2) == 0 {
            g.position.y -= 30.0;
        }
        g.velocity *= 0.3;
        g.velocity.x += rng.next_range(-10, 11) as f32 * 0.05;
        g.velocity.y += rng.next_range(-10, 11) as f32 * 0.05;
    }
}

/// Sites 54-57 — `Kill()`'s 681 goblin-bomb corner gores (`Projectile.cs:
/// 71928-71943`): four `NewGore(pos, Zero, Next(61,64))` at the RESIZED
/// 22×22 position (the caller ran `:71904-71910`); post `velocity *= 0.4`
/// then the sign pair (+,+), (-,+), (+,-), (-,-) in order.
pub fn goblin_bomb_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let signs = [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)];
    for (sx, sy) in signs {
        let ty = rng.next_range(61, 64);
        let idx = gore.new_gore(&mut crate::projectile::DynGoreRng(rng), world, p.pos, Vec2::ZERO, ty, 1.0);
        let g = gore.slot_mut(idx);
        g.velocity *= 0.4;
        g.velocity.x += sx;
        g.velocity.y += sy;
    }
}

/// Site 58 — `Kill()`'s 645 gore (`Projectile.cs:72118-72124`): the wave-C
/// celeb2 row ×1, keep-one. The 645 Kill arm runs NO resize and NO `Damage()`
/// (`:72094-72125`) — the gore reads the CURRENT box: the placed cloud's
/// 140×140 (`:56983`) or the flight's 10×10 (`:6955-6956`). (The earlier
/// "Resize(144)" note was the 452 arm's `:72990`, not this one — corrected
/// at wiring against the source.)
pub fn kill645_gore(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    celeb2_gore(p, gore, rng, world);
}

/// Sites 59/60 — `Kill()`'s 608 (`:72217-72224`) / 617 (`:72319-72326`)
/// gore pairs: the wave-B fire-explosion row ×2, keep-one. The caller ran
/// the box resize (617: 176×176 at `:72280-72282`) before the dust loops.
pub fn kill608_617_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    fire_explosion_gores(p, gore, rng, world);
}

/// Sites 61-63 — `Kill()`'s 590 triple (`Projectile.cs:72628-72639`): ONE
/// `NextFloat` for the shared scale `0.6 + NextFloat*0.4`, base type 375
/// (61 when `ai[2] == 1`), three gores base/+1/+2 at the position with
/// velocity ZERO, post `velocity *= 0.3` each. The 375-377 sticky arm's
/// alpha=100 is the pool's (`Gore.cs:1345-1349`).
pub fn kill590_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let scale = 0.6 + rng.next_float() * 0.4;
    let base = if p.ai[2] == 1.0 { 61 } else { 375 };
    for off in 0..3 {
        let idx = gore.new_gore(&mut crate::projectile::DynGoreRng(rng), world, p.pos, Vec2::ZERO, base + off, scale);
        gore.slot_mut(idx).velocity *= 0.3;
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

    /// `Projectile.cs:71617` — the Stellar Tune gore: ONE NextFloat (the
    /// edge unit) + Next(1) (the one-element SelectRandom) + kicks.
    #[test]
    fn w4_g16_the_stellar_tune_pick_still_draws_next_one() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(21);
        stellar_tune_gore(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(21);
        let _ = twin.next_float(); // CircularEdge unit
        let _ = twin.next_max(1); // SelectRandom(16)
        let _ = twin.next_range(10, 31);
        let _ = twin.next_range(-20, 21);
        assert_eq!(rng.peek(), twin.peek());
        assert_eq!(gore.slot(0).gore_type, 16);
    }

    /// `Projectile.cs:71888-71900` — the DD2 loop: per gore the argument
    /// pair + type + kicks, then THREE doubles (RotatedByRandom + radius),
    /// the Next(2) Y-drop gate and the two velocity kicks.
    #[test]
    fn w4_g16_the_dd2_loop_draws_the_y_drop_gate() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(23);
        dd2_explosion_gores(&p, 4, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(23);
        for _ in 0..4 {
            let _ = twin.next_max(100);
            let _ = twin.next_max(100);
            let _ = twin.next_range(61, 64);
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
            let _ = twin.next_double();
            let _ = twin.next_double();
            let _ = twin.next_double();
            let _ = twin.next_max(2);
            let _ = twin.next_range(-10, 11);
            let _ = twin.next_range(-10, 11);
        }
        assert_eq!(rng.peek(), twin.peek());
        let active = gore.slots()[..4].iter().filter(|g| g.active).count();
        assert_eq!(active, 4);
    }

    /// `Projectile.cs:71928-71943` — the goblin bomb: four type rolls +
    /// kicks, the sign pairs land in order (+,+ / -, + / +, - / -, -).
    #[test]
    fn w4_g16_the_goblin_bomb_corners_sign_in_order() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(25);
        goblin_bomb_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(25);
        for _ in 0..4 {
            let _ = twin.next_range(61, 64);
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek());
        // The kicked velocity ×0.4 plus the sign: check the sign of the
        // X offset against the kick (the kicks are ±2.0 max, ×0.4 = ±0.8,
        // the ±1 sign dominates only when the kick is small — assert the
        // exact C# expression instead).
        let mut twin2 = UnifiedRandom::new(25);
        let signs = [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)];
        for (i, (sx, sy)) in signs.iter().enumerate() {
            let _ = twin2.next_range(61, 64);
            let ky = twin2.next_range(10, 31);
            let kx = twin2.next_range(-20, 21);
            let kicked = Vec2::new(kx as f32 * 0.1, -(ky as f32 * 0.1)) * 0.4;
            assert_eq!(gore.slot(i).velocity, kicked + Vec2::new(*sx, *sy));
        }
    }

    /// `Projectile.cs:72628-72639` — the 590 triple: ONE NextFloat scale,
    /// then 3 × kicks; the types are base/+1/+2 (375 default, 61 bounced).
    #[test]
    fn w4_g16_the_590_triple_shares_one_scale_float() {
        let w = air();
        let mut p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(27);
        kill590_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(27);
        let s = 0.6 + twin.next_float() * 0.4;
        for _ in 0..3 {
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek());
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![375, 376, 377]);
        for g in &gore.slots()[..3] {
            assert_eq!(g.scale, s);
            assert_eq!(g.alpha, 100); // the 375-377 sticky arm (Gore.cs:1345)
        }
        // The bounced base: ai[2] == 1 → 61/62/63.
        p.ai[2] = 1.0;
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(27);
        kill590_gores(&p, &mut gore, &mut rng, &w);
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![61, 62, 63]);
    }

    /// The keep-one delegations (58/59/60) forward to the wave-B/C rows:
    /// one celeb2 gore = 7 ranges, two fire-explosion gores = the wave-B
    /// stream. A single smoke check per delegation.
    #[test]
    fn w4_g16_the_kill_delegations_match_their_rows() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(29);
        kill645_gore(&p, &mut gore, &mut rng, &w);
        // The celeb2 row's exact ranges (the wave-C witness pins them).
        let mut twin = UnifiedRandom::new(29);
        let _ = twin.next_range(0, 100);
        let _ = twin.next_range(0, 100);
        let _ = twin.next_range(61, 64);
        let _ = twin.next_range(10, 31);
        let _ = twin.next_range(-20, 21);
        let _ = twin.next_range(-10, 11);
        let _ = twin.next_range(-10, 11);
        assert_eq!(rng.peek(), twin.peek());
        // 608/617: the fire-explosion pair (the wave-B witness pins it).
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(31);
        kill608_617_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(31);
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
        assert_eq!(rng.peek(), twin.peek());
    }
}
