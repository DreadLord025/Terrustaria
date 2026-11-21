//! `Projectile.cs`'s `Gore.NewGore*` spawn sites — wave E: sites 41-51 in
//! source order (`:62891`, `:68780`, `:68828`, `:70444`-`:70463`,
//! `:70894`, `:70912`-`:70915`). The 1085 mirror's four consecutive
//! `NewGore` lines (`:70912-70915`) are ONE keep-one group — the wave
//! carries 11 sites rather than split a C# statement block (the d55
//! party-hat-block precedent).
//!
//! The pool + the wave-A contract live in `projectile_gore_a.rs` (W4-g16,
//! the `npc_hit_effect_a.rs` precedent). Dust bodies are the dust lanes'
//! skip; argument draws precede the pool call, kicks roll inside
//! `new_gore`, post-writes land on the returned slot (sentinel included).
//!
//! ## Scope — sites 41-51 (source order)
//!
//! | # | C# site | arm |
//! | --- | --- | --- |
//! | 41 | `:62891` | `AI_026` 860 idle gore loop (3+Next(3) × 1218) |
//! | 42 | `:68780` | `AI_135_OgreStomp` tile gore (the wave-D row) |
//! | 43 | `:68828` | `AI_136_BetsyBreath` gore |
//! | 44-46 | `:70444`/`:70454`/`:70463` | `AI_145_BookStaffStorm` 3 gores |
//! | 47 | `:70894` | `Kill()` 1020/1105 corner gores (4 × Next(5)) |
//! | 48-51 | `:70912`-`:70915` | `Kill()` 1085 mirror (1394-1397) |
//!
//! ## RNG notes specific to this wave
//!
//! * The ogre-stomp tile gore (42) is the SAME row as the ground explosion
//!   (`:53456`) — keep-one onto `ground_explosion_tile_gore`; the caller's
//!   walk keeps the `num5 <= 0 || Next(3) == 0` gate (`:68755`) and the
//!   922 dust-only branch (`:68760-68777`).
//! * `Utils.RandomVector2` (`Utils.cs:2431-2434`) draws TWO `NextDouble`
//!   (the Betsy position jitter); the type `61 + Next(3)` evaluates AFTER
//!   the position argument (C# left-to-right).
//! * The book-storm gores (44-46) share the drill shape (wave B) but the
//!   gates are 5/7/7 and the post-writes differ (timeLeft 60/0/0, alpha
//!   50/80/80; only the FIRST adds `velocity.X += vx`). The 1007/1008
//!   SpecialAI==3 four draw inside the pool (`Gore.cs:1391-1402`).
//! * The mirror gores (48-51) read `base.oldVelocity` AFTER the `*= 0.2`
//!   and the position pullback (`:70905-70906`) — the caller passes the
//!   already-scaled vector; the four types are data, only kicks draw.
//!
//! See docs/ANALYSIS.md "The Projectile gore sites (W4-g16)".
#![allow(dead_code)] // wiring deferred (code-only mode)

use crate::projectile::Projectile;
use crate::projectile_gore_d::ground_explosion_tile_gore;
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

/// `Utils.NextVector2Square` (`Utils.cs:1251-1254`) — TWO `NextDouble`.
fn next_vector2_square(rng: &mut dyn DropRng, min: f32, max: f32) -> Vec2 {
    let x = (max - min) * rng.next_double() as f32 + min;
    let y = (max - min) * rng.next_double() as f32 + min;
    Vec2::new(x, y)
}

/// Site 41 — `AI_026`'s 860 idle gore loop (`Projectile.cs:62889-62893`):
/// the caller gates on `frame == 3 && localAI[0] == 820`; the count is
/// `3 + Next(3)`, each gore type 1218 at `(position.X, Center.Y - 10)`,
/// velocity ZERO; post `velocity = (Next(1,10)*0.3*-spriteDirection,
/// -(2 + Next(4)*0.3))` — the two rolls AFTER the kicks.
pub fn ai026_860_idle_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let count = 3 + rng.next_max(3);
    let pos = Vec2::new(p.pos.x, center(p).y - 10.0);
    for _ in 0..count {
        let idx = gore.new_gore(rng, world, pos, Vec2::ZERO, 1218, 1.0);
        let g = gore.slot_mut(idx);
        let vx = rng.next_range(1, 10) as f32 * 0.3 * -p.sprite_direction as f32;
        let vy = -(2.0 + rng.next_max(4) as f32 * 0.3);
        g.velocity = Vec2::new(vx, vy);
    }
}

/// Site 42 — `AI_135_OgreStomp`'s tile gore (`Projectile.cs:68778-68784`):
/// the SAME row as `CreateGroundExplosion` (`:53456`) — keep-one. The
/// caller's walk keeps the dust gates and the 922 branch.
pub fn ogre_stomp_tile_gore(
    p: &Projectile,
    i: i32,
    j: i32,
    num4: i32,
    num7: f32,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    ground_explosion_tile_gore(p, i, j, num4, num7, gore, rng, world);
}

/// Site 43 — `AI_136_BetsyBreath`'s gore (`Projectile.cs:68826-68831`):
/// the `Next(5)` gate (the caller ALSO gates `ai[0] >= 15` — the C# `&&`
/// evaluates Next(5) FIRST); `NewGoreDirect` at `Center +
/// rotation.ToRotationVector2()*300 - RandomVector2(-20,20)` (TWO
/// NextDouble), type `61 + Next(3)`, scale 0.5; post `velocity *= 0.3`,
/// `velocity += rotvec * 4`.
pub fn betsy_breath_gore(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    if rng.next_max(5) != 0 {
        return;
    }
    // `rotation.ToRotationVector2()` (`Utils.cs:1665-1668`): cos/sin of the
    // f32 angle widened to double, cast back.
    let rotvec = Vec2::new(
        (p.rot as f64).cos() as f32,
        (p.rot as f64).sin() as f32,
    );
    // `Utils.RandomVector2` (`Utils.cs:2431-2434`) — two NextDouble.
    let jx = 40.0 * rng.next_double() as f32 - 20.0;
    let jy = 40.0 * rng.next_double() as f32 - 20.0;
    let pos = center(p) + rotvec * 300.0 - Vec2::new(jx, jy);
    let ty = 61 + rng.next_max(3);
    let idx = gore.new_gore_direct(rng, world, pos, Vec2::ZERO, ty, 0.5);
    let g = gore.slot_mut(idx);
    g.velocity *= 0.3;
    g.velocity += rotvec * 4.0;
}

/// Sites 44-46 — `AI_145_BookStaffStorm`'s three gores (`Projectile.cs:
/// 70440-70467`): the shared drill shape with per-site gate (5/7/7),
/// timeLeft (60/0/0), alpha (50/80/80) and the first-only `velocity.X +=
/// vx`. One keep-one fn; the caller runs it three times in order.
pub fn book_storm_gore(
    p: &Projectile,
    gate_max: i32,
    time_left: i32,
    alpha: i32,
    add_vx: bool,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    if rng.next_max(gate_max) != 0 {
        return;
    }
    let pos = p.pos + next_vector2_square(rng, 0.0, 1.0) * Vec2::new(p.w, p.h);
    let vy = (0.0 - rng.next_float()) * 16.0;
    let ty = [1007, 1008, 1008][rng.next_max(3) as usize];
    let idx = gore.new_gore_direct(rng, world, pos, Vec2::new(p.vel.x * 1.5, vy), ty, 1.0);
    let g = gore.slot_mut(idx);
    g.time_left = time_left;
    g.alpha = alpha;
    if add_vx {
        g.velocity.x += p.vel.x;
    }
}

/// Site 47 — `Kill()`'s 1020/1105 corner gores (`Projectile.cs:70888-
/// 70900`): the double loop `n = -1..1 step 2`, `num10 = -1..1 step 2`,
/// each a `Next(5)` gate; taken → `NewGoreDirect(pos, Zero, Next(61,64))`,
/// post `velocity *= 0.2`, `scale *= 0.65`, `velocity += (n, num10)*0.5`.
/// The dust loops before are the dust lane's.
pub fn rocket_corner_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let mut n = -1.0f32;
    while n <= 1.0 {
        let mut m = -1.0f32;
        while m <= 1.0 {
            if rng.next_max(5) == 0 {
                let ty = rng.next_range(61, 64);
                let idx = gore.new_gore_direct(rng, world, p.pos, Vec2::ZERO, ty, 1.0);
                let g = gore.slot_mut(idx);
                g.velocity *= 0.2;
                g.scale *= 0.65;
                g.velocity += Vec2::new(n, m) * 0.5;
            }
            m += 2.0;
        }
        n += 2.0;
    }
}

/// Sites 48-51 — `Kill()`'s 1085 mirror gores (`Projectile.cs:70912-
/// 70915`): four consecutive `NewGore(Center, -oldVelocity, 1394..1397)`.
/// `old_velocity` is the ALREADY-scaled `base.oldVelocity` (the caller ran
/// `*= 0.2` and the position pullback at `:70905-70906`); the 10-dust loop
/// before is the dust lane's. The types are data — only the kicks draw.
pub fn mirror_gores(
    p: &Projectile,
    old_velocity: Vec2,
    gore: &mut GorePool,
    rng: &mut dyn DropRng,
    world: &World,
) {
    let c = center(p);
    for ty in 1394..=1397 {
        gore.new_gore(rng, world, c, -old_velocity, ty, 1.0);
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
        p.sprite_direction = -1;
        p
    }

    /// `Projectile.cs:62889-62893` — the 860 loop: Next(3) count, then per
    /// gore the kicks + Next(1,10) + Next(4); the velocity overwrite lands
    /// after (the kicks' values are discarded, the draws are not).
    #[test]
    fn w4_g16_the_860_idle_loop_draws_the_count_then_two_per_gore() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(7);
        ai026_860_idle_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(7);
        let count = 3 + twin.next_max(3);
        for _ in 0..count {
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
            let _ = twin.next_range(1, 10);
            let _ = twin.next_max(4);
        }
        assert_eq!(rng.peek(), twin.peek());
        let active = gore.slots()[..count as usize].iter().filter(|g| g.active).count();
        assert_eq!(active, count as usize);
        for g in &gore.slots()[..count as usize] {
            assert_eq!(g.gore_type, 1218);
            // spriteDirection -1 → vx positive; vy always negative.
            assert!(g.velocity.x > 0.0);
            assert!(g.velocity.y < -2.0);
        }
    }

    /// `Projectile.cs:68826-68831` — the Betsy gore: Next(5) gate, TWO
    /// NextDouble (RandomVector2), Next(3) type, kicks; the post-writes
    /// draw nothing.
    #[test]
    fn w4_g16_the_betsy_gore_jitters_with_two_doubles() {
        let w = air();
        let p = proj();
        let mut seed = 0;
        while UnifiedRandom::new(seed).next_max(5) != 0 {
            seed += 1;
        }
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(seed);
        betsy_breath_gore(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(seed);
        let _ = twin.next_max(5);
        let _ = twin.next_double();
        let _ = twin.next_double();
        let ty = 61 + twin.next_max(3);
        let _ = twin.next_range(10, 31);
        let _ = twin.next_range(-20, 21);
        assert_eq!(rng.peek(), twin.peek());
        let g = gore.slot(0);
        assert_eq!(g.gore_type, ty);
        assert_eq!(g.scale, 0.5);
    }

    /// `Projectile.cs:70440-70467` — the book-storm triple: gates 5/7/7 in
    /// order; a taken gore draws the square (2 doubles) + NextFloat + Next(3)
    /// + kicks + the SpecialAI four. The post-writes differ per site.
    #[test]
    fn w4_g16_the_book_storm_triple_keeps_the_per_site_postwrites() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(10);
        book_storm_gore(&p, 5, 60, 50, true, &mut gore, &mut rng, &w);
        book_storm_gore(&p, 7, 0, 80, false, &mut gore, &mut rng, &w);
        book_storm_gore(&p, 7, 0, 80, false, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(10);
        let mut spawned = 0usize;
        for (gate, tl, alpha) in [(5, 60, 50), (7, 0, 80), (7, 0, 80)] {
            if twin.next_max(gate) == 0 {
                let _ = twin.next_double();
                let _ = twin.next_double();
                let _ = twin.next_float();
                let _ = twin.next_max(3);
                let _ = twin.next_range(10, 31);
                let _ = twin.next_range(-20, 21);
                // SpecialAI==3 (1007/1008): two floats + two Next(8).
                let _ = twin.next_float();
                let _ = twin.next_float();
                let _ = twin.next_max(8);
                let _ = twin.next_max(8);
                let g = gore.slot(spawned);
                assert_eq!(g.time_left, tl);
                assert_eq!(g.alpha, alpha);
                spawned += 1;
            }
        }
        assert_eq!(rng.peek(), twin.peek(), "the triple's stream mirrors");
    }

    /// `Projectile.cs:70888-70900` — the corner gores: FOUR Next(5) gates
    /// always draw (the double loop), each taken branch adds type + kicks.
    #[test]
    fn w4_g16_the_corner_gores_draw_four_gates() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(13);
        rocket_corner_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(13);
        let mut spawned = 0usize;
        for _ in 0..4 {
            if twin.next_max(5) == 0 {
                let ty = twin.next_range(61, 64);
                let _ = twin.next_range(10, 31);
                let _ = twin.next_range(-20, 21);
                let g = gore.slot(spawned);
                assert_eq!(g.gore_type, ty);
                assert_eq!(g.scale, 0.65);
                spawned += 1;
            }
        }
        assert_eq!(rng.peek(), twin.peek());
    }

    /// `Projectile.cs:70912-70915` — the mirror: four gores 1394-1397 at
    /// the centre with -oldVelocity; only the kicks draw (2 per gore).
    #[test]
    fn w4_g16_the_mirror_gores_spawn_the_four_types() {
        let w = air();
        let p = proj();
        let ov = Vec2::new(0.4, -0.2);
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(17);
        mirror_gores(&p, ov, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(17);
        for _ in 0..4 {
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek());
        let types: Vec<i32> = gore.slots()[..4].iter().map(|g| g.gore_type).collect();
        assert_eq!(types, vec![1394, 1395, 1396, 1397]);
        for g in &gore.slots()[..4] {
            assert_eq!(g.position, Vec2::new(110.0, 110.0));
        }
    }
}
