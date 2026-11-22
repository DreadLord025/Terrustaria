//! `Projectile.cs`'s `Gore.NewGore*` spawn sites — wave G: sites 64-73 in
//! source order (`:72758`-`:72770`, `:72890`, `:73006`, `:73039`,
//! `:73086`, `:73133`, `:73242`, `:73367`). The 676 coin-pile's four
//! consecutive loops (`:72756-72771`) are one keep-one group (the wave-E
//! mirror precedent).
//!
//! The pool + the wave-A contract live in `projectile_gore_a.rs` (W4-g16,
//! the `npc_hit_effect_a.rs` precedent). Dust bodies are the dust lanes'
//! skip; argument draws precede the pool call, kicks roll inside
//! `new_gore`, post-writes land on the returned slot (sentinel included).
//!
//! ## Scope — sites 64-73 (source order)
//!
//! | # | C# site | arm |
//! | --- | --- | --- |
//! | 64-67 | `:72758`/`:72762`/`:72766`/`:72770` | `Kill()` 676 coin pile |
//! | 68 | `:72890` | `Kill()` 501 cursed-flame gores ×5 |
//! | 69 | `:73006` | `Kill()` 452 gore (the wave-C row) |
//! | 70 | `:73039` | `Kill()` 454 gores ×3 (the wave-C row) |
//! | 71 | `:73086` | `Kill()` 467 gores ×2 (the wave-B row) |
//! | 72 | `:73133` | `Kill()` 468 gores ×2 (the wave-B row) |
//! | 73 | `:73242` | `Kill()` 483 gore (integer kicks) |
//! | 74 | `:73367` | `Kill()` 459/709 gores ×num362 (the wave-C row) |
//!
//! (Site 74 rides along: the 459/709 loop is the same wave-C row with a
//! caller count — the delegation keeps the wave boundary at the row.)
//!
//! ## RNG notes specific to this wave
//!
//! * The coin pile (64-67): per gore TWO `NextFloat` (the Lerp X + the Y),
//!   the type is data (1024/1025/1026/1026), counts 10/10/10/20; the last
//!   loop's Lerp is (-0.5, 0.5) and the Y scale ×2 (not ×5).
//! * The cursed-flame gores (68) read the GROWN box (the caller ran
//!   `:72878-72881`: pos −20, size +40); `Next(width)`/`Next(height)` are
//!   the position rolls (NOT the ×Next(100)/100 shape), type Next(435,438);
//!   post ×0.5, the per-index sign pair, ×0.5 again. The 5th gore takes
//!   the switch's DEFAULT (-,-) — same as index 3.
//! * The 483 gore (73) kicks are INTEGER `Next(-1, 2)` (not ×0.05).
//! * The 459/709 count (74) is num362: 0 below scale 1, 2 at/above (the
//!   caller's `:73343-73352` resize runs first).
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

/// Sites 64-67 — `Kill()`'s 676 coin pile (`Projectile.cs:72756-72771`):
/// four loops at the centre, velocity `(Lerp(-5,5,NextFloat),
/// -NextFloat*5)`, types 1024 ×10, 1025 ×10, 1026 ×10, then 1026 ×20 with
/// `Lerp(-0.5,0.5,NextFloat)` and `-NextFloat*2`. The 120-dust loop before
/// is the dust lane's; the buff sweep after is the wiring wave's.
pub fn coin_pile_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let c = center(p);
    // `:72756-72767` — the three 10-loops, Lerp(-5,5) / Y ×5.
    for ty in [1024, 1025, 1026] {
        for _ in 0..10 {
            let vx = -5.0 + 10.0 * rng.next_float();
            let vy = (0.0 - rng.next_float()) * 5.0;
            gore.new_gore_direct(rng, world, c, Vec2::new(vx, vy), ty, 1.0);
        }
    }
    // `:72768-72771` — the 20-loop, Lerp(-0.5,0.5) / Y ×2.
    for _ in 0..20 {
        let vx = -0.5 + 1.0 * rng.next_float();
        let vy = (0.0 - rng.next_float()) * 2.0;
        gore.new_gore_direct(rng, world, c, Vec2::new(vx, vy), 1026, 1.0);
    }
}

/// Site 68 — `Kill()`'s 501 cursed-flame gores (`Projectile.cs:72888-
/// 72912`): 5 gores at `pos + (Next(width), Next(height))` (the GROWN box —
/// the caller ran `:72878-72881`), type `Next(435, 438)`; post `velocity *=
/// 0.5`, the sign pair by index (+,+ / -, + / +, - / -, - / -, -), then
/// `velocity *= 0.5` again.
pub fn cursed_flame_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    for i in 0..5 {
        let pos = Vec2::new(
            p.pos.x + rng.next_max(p.w as i32) as f32,
            p.pos.y + rng.next_max(p.h as i32) as f32,
        );
        let ty = rng.next_range(435, 438);
        let idx = gore.new_gore(rng, world, pos, Vec2::ZERO, ty, 1.0);
        let g = gore.slot_mut(idx);
        g.velocity *= 0.5;
        // `:72892-72910` — the switch; index 4 takes the default (-,-).
        let (sx, sy) = match i {
            0 => (1.0, 1.0),
            1 => (-1.0, 1.0),
            2 => (1.0, -1.0),
            _ => (-1.0, -1.0),
        };
        g.velocity.x += sx;
        g.velocity.y += sy;
        g.velocity *= 0.5;
    }
}

/// Sites 69/70/74 — `Kill()`'s 452 ×1 (`:73004-73010`), 454 ×3 (`:73037-
/// 73043`), 459/709 ×num362 (`:73365-73371`) gores: the wave-C celeb2 row,
/// keep-one. The caller ran the box resize first (452: 144; 454: 208;
/// 459/709: 144 at scale ≥ 1 with count 2, else count 0).
pub fn kill_row_gores(
    p: &Projectile,
    count: i32,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    for _ in 0..count {
        celeb2_gore(p, gore, rng, world);
    }
}

/// Sites 71/72 — `Kill()`'s 467 (`:73084-73091`) / 468 (`:73131-73138`)
/// gore pairs: the wave-B fire-explosion row ×2, keep-one (the caller ran
/// the 176×176 resize first).
pub fn kill467_468_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    fire_explosion_gores(p, gore, rng, world);
}

/// Site 73 — `Kill()`'s 483 gore (`Projectile.cs:73242-73245`): ONE gore
/// at the position, type `Next(61, 64)`; post `velocity *= 0.3`,
/// `velocity.X += Next(-1, 2)` (INTEGER), `velocity.Y += Next(-1, 2)`.
pub fn kill483_gore(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let ty = rng.next_range(61, 64);
    let idx = gore.new_gore(rng, world, p.pos, Vec2::ZERO, ty, 1.0);
    let g = gore.slot_mut(idx);
    g.velocity *= 0.3;
    g.velocity.x += rng.next_range(-1, 2) as f32;
    g.velocity.y += rng.next_range(-1, 2) as f32;
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

    /// `Projectile.cs:72756-72771` — the coin pile: 50 gores, two floats
    /// each (the Lerp X + the Y), the types in loop order.
    #[test]
    fn w4_g16_the_coin_pile_spawns_fifty_in_loop_order() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(33);
        coin_pile_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(33);
        for _ in 0..50 {
            let _ = twin.next_float();
            let _ = twin.next_float();
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek());
        let types: Vec<i32> = gore.slots()[..50].iter().map(|g| g.gore_type).collect();
        assert_eq!(types[..10], [1024; 10]);
        assert_eq!(types[10..20], [1025; 10]);
        assert_eq!(types[20..30], [1026; 10]);
        assert_eq!(types[30..50], [1026; 20]);
    }

    /// `Projectile.cs:72888-72912` — the cursed flame: per gore Next(w) +
    /// Next(h) + Next(435,438) + kicks; the sign pairs land by index and
    /// the double ×0.5 sandwich is exact.
    #[test]
    fn w4_g16_the_cursed_flame_gores_sandwich_the_signs() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(35);
        cursed_flame_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(35);
        let signs = [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0), (-1.0, -1.0)];
        for (i, (sx, sy)) in signs.iter().enumerate() {
            let px = twin.next_max(20);
            let py = twin.next_max(20);
            let ty = twin.next_range(435, 438);
            let ky = twin.next_range(10, 31);
            let kx = twin.next_range(-20, 21);
            let g = gore.slot(i);
            assert_eq!(g.gore_type, ty);
            assert_eq!(g.position, Vec2::new(100.0 + px as f32, 100.0 + py as f32));
            let kicked = Vec2::new(kx as f32 * 0.1, -(ky as f32 * 0.1)) * 0.5;
            assert_eq!(g.velocity, (kicked + Vec2::new(*sx, *sy)) * 0.5);
        }
        assert_eq!(rng.peek(), twin.peek(), "the 5-gore stream mirrors");
    }

    /// `Projectile.cs:73242-73245` — the 483 gore: type + kicks + the two
    /// INTEGER Next(-1,2) kicks (no ×0.05).
    #[test]
    fn w4_g16_the_483_gore_kicks_are_integers() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(37);
        kill483_gore(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(37);
        let ty = twin.next_range(61, 64);
        let ky = twin.next_range(10, 31);
        let kx = twin.next_range(-20, 21);
        let ix = twin.next_range(-1, 2);
        let iy = twin.next_range(-1, 2);
        assert_eq!(rng.peek(), twin.peek());
        let g = gore.slot(0);
        assert_eq!(g.gore_type, ty);
        let kicked = Vec2::new(kx as f32 * 0.1, -(ky as f32 * 0.1)) * 0.3;
        assert_eq!(g.velocity, kicked + Vec2::new(ix as f32, iy as f32));
    }

    /// The keep-one delegations: count × the wave-C row (7 ranges each),
    /// the 467/468 pair = the wave-B stream. Count 0 draws NOTHING.
    #[test]
    fn w4_g16_the_kill_row_delegations_scale_by_count() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(39);
        kill_row_gores(&p, 3, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(39);
        for _ in 0..3 {
            let _ = twin.next_range(0, 100);
            let _ = twin.next_range(0, 100);
            let _ = twin.next_range(61, 64);
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
            let _ = twin.next_range(-10, 11);
            let _ = twin.next_range(-10, 11);
        }
        assert_eq!(rng.peek(), twin.peek());
        // Count 0 (the 459/709 below-scale-1 case): no draws, no spawns.
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(39);
        kill_row_gores(&p, 0, &mut gore, &mut rng, &w);
        assert_eq!(rng.peek(), UnifiedRandom::new(39).peek());
        // 467/468: the fire-explosion pair.
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(41);
        kill467_468_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(41);
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
