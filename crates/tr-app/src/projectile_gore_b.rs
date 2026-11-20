//! `Projectile.cs`'s `Gore.NewGore*` spawn sites — wave B: sites 11-20 in
//! source order (`:24267`, `:24583`, `:25440`, `:28308`, `:40295`-`:40301`,
//! `:42325`, `:45305`, `:50155`).
//!
//! The pool + the wave-A contract live in `projectile_gore_a.rs` (W4-g16,
//! the `npc_hit_effect_a.rs` precedent): each site fn takes the slot's
//! `Projectile`, the `GorePool`, the caller's `UnifiedRandom` (the
//! `Main.UpdateProjectiles` stream) and the `World`; argument draws precede
//! the pool call, the `NewGore` kicks roll inside `new_gore`, post-writes
//! land on the returned slot (sentinel included). The dust bodies around
//! each site are the dust lanes' documented skip.
//!
//! ## Scope — sites 11-20 (source order)
//!
//! | # | C# site | arm |
//! | --- | --- | --- |
//! | 11 | `:24267` | `AI()` type 27 lava bounce |
//! | 12 | `:24583` | `AI()` type 22 (Aqua Scepter) lava bounce |
//! | 13 | `:25440` | `AI()` type 1092 `ai[0]==2` drill gore |
//! | 14 | `:28308` | `AI()` aiStyle 49 tombstone landing |
//! | 15-17 | `:40295`/`:40298`/`:40301` | `AI_197_HandleTileCollision` 1025 burst |
//! | 18 | `:42325` | `AI_189_Volcano` `ai[0]==1` gore loop |
//! | 19 | `:45305` | `AI_178_FireExplosion` 2-gore loop |
//! | 20 | `:50155` | `AI_067_FreakingPirates` jump gore (1-in-4) |
//!
//! ## RNG notes specific to this wave
//!
//! * `NextVector2Square` (`Utils.cs:1251-1254`) draws TWO `NextDouble`
//!   (NOT `NextFloat`); `RotatedByRandom` (`Utils.cs:1697-1700`) draws TWO
//!   `NextDouble` (`NextDouble*max - NextDouble*max`). The witnesses replay
//!   `next_double`, not `next_float`, for those.
//! * The lava-bounce gores (11/12) ride the ALREADY-LANDED census arms
//!   (`projectile.rs::ai_aqua_scepter` `:7491`, the type-27 twin): those
//!   censuses consume only the gate + type roll. The wiring wave REPLACES
//!   the census line with these pool fns (which add the two kicks + the
//!   post-writes) — the d55 census→pool upgrade.
//! * The 1025 burst (15-17) and the volcano (18) / fire-explosion (19)
//!   dust loops precede the gores on the stream; their draws are the dust
//!   lane's. The volcano's `num7 = Next(3, 7)` (`:42311`) is shared by the
//!   second dust loop and the gore loop, so it rides [`volcano_gores`] as
//!   an INPUT (the caller draws it between the two dust loops).
//! * The pirate jump (20) rolls the `Next(4)` branch gate and RETURNS
//!   whether it took; the else-branch's `:50170` pair is wave C.
//!
//! See docs/ANALYSIS.md "The Projectile gore sites (W4-g16)".
#![allow(dead_code)] // wiring deferred (code-only mode)

use crate::projectile::Projectile;
use crate::projectile_gore_a::tombstone_landing_gore;
use glam::Vec2;
use tr_core::gore::GorePool;
use tr_core::{UnifiedRandom, World};

/// `Entity.Center` (`Entity.cs:50-60`).
fn center(p: &Projectile) -> Vec2 {
    Vec2::new(p.pos.x + p.w / 2.0, p.pos.y + p.h / 2.0)
}

/// `Entity.Top` (`Entity.cs:86-96`).
fn top(p: &Projectile) -> Vec2 {
    Vec2::new(p.pos.x + p.w / 2.0, p.pos.y)
}

/// `Utils.NextVector2Square` (`Utils.cs:1251-1254`) — TWO `NextDouble`.
fn next_vector2_square(rng: &mut UnifiedRandom, min: f32, max: f32) -> Vec2 {
    let x = (max - min) * rng.next_double() as f32 + min;
    let y = (max - min) * rng.next_double() as f32 + min;
    Vec2::new(x, y)
}

/// `Utils.RotatedBy` (`Utils.cs:1675-1684`), centre (0,0): cos/sin are
/// FLOAT casts of the double radians, the products float.
fn rotated_by(v: Vec2, radians: f64) -> Vec2 {
    let c = radians.cos() as f32;
    let s = radians.sin() as f32;
    Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}

/// `Utils.RotatedByRandom` (`Utils.cs:1697-1700`) — TWO `NextDouble`:
/// `RotatedBy(NextDouble*max - NextDouble*max)`.
fn rotated_by_random(rng: &mut UnifiedRandom, v: Vec2, max: f64) -> Vec2 {
    let a = rng.next_double() * max;
    let b = rng.next_double() * max;
    rotated_by(v, a - b)
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

/// Sites 11/12 — the lava-bounce gore (`Projectile.cs:24265-24271` type 27,
/// `:24581-24587` type 22): the `Next(3)` gate, `Next(61, 64)` type at the
/// position with default velocity, then `position.X -= 6`, `velocity *=
/// 0.4`, `velocity.Y--`. The caller gates on the centre-cell lava probe +
/// the bounce (the LCG velocity reversal is the AI arm's, not a draw here).
pub fn lava_bounce_gore(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    if rng.next_max(3) == 0 {
        let ty = rng.next_range(61, 64);
        let idx = gore.new_gore(rng, world, p.pos, Vec2::ZERO, ty, 1.0);
        let g = gore.slot_mut(idx);
        g.position.x -= 6.0;
        g.velocity *= 0.4;
        g.velocity.y -= 1.0;
    }
}

/// Site 13 — the type-1092 `ai[0]==2` drill gore (`Projectile.cs:25438-
/// 25444`): the `Next(5)` gate, `NewGoreDirect` at `TopLeft +
/// NextVector2Square(0,1) * Size`, velocity `(vx*1.5, -NextFloat*16)`,
/// `SelectRandom(1007, 1008, 1008)`; post `timeLeft = 60`, `alpha = 50`,
/// `velocity.X += vx`.
pub fn drill_unit_gore(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    if rng.next_max(5) == 0 {
        let pos = p.pos + next_vector2_square(rng, 0.0, 1.0) * Vec2::new(p.w, p.h);
        let vy = (0.0 - rng.next_float()) * 16.0;
        let ty = [1007, 1008, 1008][rng.next_max(3) as usize];
        let idx = gore.new_gore_direct(rng, world, pos, Vec2::new(p.vel.x * 1.5, vy), ty, 1.0);
        let g = gore.slot_mut(idx);
        g.time_left = 60;
        g.alpha = 50;
        g.velocity.x += p.vel.x;
    }
}

/// Site 14 — the aiStyle-49 tombstone landing (`Projectile.cs:28306-28313`):
/// identical to the wave-A 162/281 row (scale 1.5). Delegates to keep one
/// implementation (the house keep-one rule).
pub fn ai49_tombstone_landing(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    tombstone_landing_gore(p, 1.5, gore, rng, world);
}

/// Sites 15-17 — `AI_197_HandleTileCollision`'s type-1025 burst
/// (`Projectile.cs:40273-40304`): 3 iterations from `Top - UnitY*62`, the
/// per-iteration base type 1143/1137/1128 and X offset -20/0/+20. Each
/// iteration spawns the two `num3 + Next(3)` gores (sites 15/16) and the
/// 1146 crystal (site 17). The 30-dust loop before (`:40258-40272`) is the
/// dust lane's. `NewGorePerfect` overwrites position/velocity after the
/// kicks (the pool's own `new_gore_perfect`).
pub fn tile_collision_burst(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let vector2 = top(p) - Vec2::new(0.0, 62.0);
    for j in 0..3 {
        // `:40279-40294` — the base type + the X offset per iteration.
        let (num3, offset) = match j {
            0 => (1143, Vec2::new(-20.0, 6.0)),
            1 => (1137, Vec2::ZERO),
            _ => (1128, Vec2::new(20.0, 6.0)),
        };
        let vector5 = vector2 + offset;
        // `:40295-40297` — site 15: Circular(2,2) velocity, num3+Next(3).
        let vel = next_vector2_circular(rng, 2.0, 2.0);
        let ty = num3 + rng.next_max(3);
        let idx = gore.new_gore_perfect(rng, world, vector5, vel, ty, 1.0);
        let g = gore.slot_mut(idx);
        g.velocity.y -= rng.next_float();
        g.velocity *= 2.0;
        // `:40298-40300` — site 16: the (-4+Next(5), -4+Next(5)) jitter.
        let jitter = Vec2::new(-4.0 + rng.next_max(5) as f32, -4.0 + rng.next_max(5) as f32);
        let vel = next_vector2_circular(rng, 2.0, 2.0);
        let ty = num3 + rng.next_max(3);
        let idx = gore.new_gore_perfect(rng, world, vector5 + jitter, vel, ty, 1.0);
        let g = gore.slot_mut(idx);
        g.velocity.y -= rng.next_float();
        g.velocity *= 2.0;
        // `:40301-40303` — site 17: the 1146 crystal, +10 on the Y jitter.
        let jitter = Vec2::new(-4.0 + rng.next_max(5) as f32, -4.0 + rng.next_max(5) as f32 + 10.0);
        let vel = next_vector2_circular(rng, 2.0, 2.0);
        let idx = gore.new_gore_perfect(rng, world, vector5 + jitter, vel, 1146, 1.0);
        let g = gore.slot_mut(idx);
        g.velocity.y -= rng.next_float();
        g.velocity *= 0.5;
    }
}

/// Site 15b (the gap fill) — the aiStyle-132 crystal-destruction burst
/// (`Projectile.cs:36326-36342`, between the 1025 burst and the volcano in
/// source order): the caller gates on `localAI[1] == 0 && localAI[0] ==
/// 105`; 20 gores, per gore the angle `i/20 * -PI + NextFloat*0.1 - 0.05`
/// (ONE float) and the radius `NextFloat*10` (ONE float), vector =
/// `(radius, 0).RotatedBy(angle)`; `NewGoreDirect(Center + vector*3,
/// vector, SelectRandom(1027,1028,1029,1030))` = Next(4); post the Y-flip
/// chain (`> 0 → *= -0.5`, `< -5 → Y *= 0.8`, then `Y *= 1.1`, `X *= 0.88`).
pub fn crystal_destruction_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let c = center(p);
    for i in 0..20 {
        // `:36330` — the angle draw then the radius draw (argument order).
        let angle = i as f32 / 20.0 * -std::f32::consts::PI + rng.next_float() * 0.1 - 0.05;
        let radius = rng.next_float() * 10.0;
        let vector = rotated_by(Vec2::new(radius, 0.0), angle as f64);
        let ty = [1027, 1028, 1029, 1030][rng.next_max(4) as usize];
        let idx = gore.new_gore_direct(rng, world, c + vector * 3.0, vector, ty, 1.0);
        let g = gore.slot_mut(idx);
        // `:36332-36341` — the flip chain, in order.
        if g.velocity.y > 0.0 {
            g.velocity *= -0.5;
        }
        if g.velocity.y < -5.0 {
            g.velocity.y *= 0.8;
        }
        g.velocity.y *= 1.1;
        g.velocity.x *= 0.88;
    }
}

/// Site 18 — `AI_189_Volcano`'s `ai[0]==1` gore loop (`Projectile.cs:42323-
/// 42331`): `num7` gores (the caller draws `Next(3, 7)` at `:42311` between
/// the two dust loops; the 1125 flag divides it by 3). Per gore:
/// `NewGorePerfect` at `Center + Circular(8,8) - (16,16)`, velocity
/// `vel.RotatedBy(NextFloatDirection * (PI/4) * num4) * (4*NextFloat)`,
/// type `61 + Next(3)`; post `velocity = RotatedBy(num3)` (num3 = 0, the
/// identity) and the 1125 flag's `*= 1.7`. `num4` = 0.6 (0.3 for 1125).
pub fn volcano_gores(
    p: &Projectile,
    num7: i32,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    let flag = p.kind == 1125;
    let num4 = if flag { 0.3 } else { 0.6 };
    let c = center(p);
    for _ in 0..num7 {
        let pos = c + next_vector2_circular(rng, 8.0, 8.0) - Vec2::new(16.0, 16.0);
        let radians = rng.next_float_direction() * (std::f32::consts::PI / 4.0) * num4;
        let vel = rotated_by(p.vel, radians as f64) * (4.0 * rng.next_float());
        let ty = 61 + rng.next_max(3);
        let idx = gore.new_gore_perfect(rng, world, pos, vel, ty, 1.0);
        let g = gore.slot_mut(idx);
        // `:42326` — RotatedBy(num3 = 0): cos 0 = 1, sin 0 = 0, identity.
        g.velocity = rotated_by(g.velocity, 0.0);
        if flag {
            g.velocity *= 1.7;
        }
    }
}

/// Site 19 — `AI_178_FireExplosion`'s 2-gore loop (`Projectile.cs:45303-
/// 45310`): per gore the position `pos + (w*Next(100)/100, h*Next(100)/100)
/// - (10,10)`, type `Next(61, 64)`; post `position = Center +
/// UnitY.RotatedByRandom(PI) * NextDouble * w/2`, `velocity *= 0.3`,
/// `velocity.X += Next(-10,11)*0.05`, `velocity.Y += Next(-10,11)*0.05`.
/// The dust loops before (`:45272-45302`) are the dust lane's.
pub fn fire_explosion_gores(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    for _ in 0..2 {
        let off = Vec2::new(
            (p.w * rng.next_max(100) as f32) / 100.0,
            (p.h * rng.next_max(100) as f32) / 100.0,
        );
        let ty = rng.next_range(61, 64);
        let idx = gore.new_gore(rng, world, p.pos + off - Vec2::splat(10.0), Vec2::ZERO, ty, 1.0);
        let g = gore.slot_mut(idx);
        // `:45306` — RotatedByRandom(PI) = two NextDouble, then NextDouble radius.
        let dir = rotated_by_random(rng, Vec2::new(0.0, 1.0), std::f64::consts::PI);
        g.position = center(p) + dir * (rng.next_double() as f32 * p.w / 2.0);
        g.velocity *= 0.3;
        g.velocity.x += rng.next_range(-10, 11) as f32 * 0.05;
        g.velocity.y += rng.next_range(-10, 11) as f32 * 0.05;
    }
}

/// Site 20 — `AI_067_FreakingPirates`' jump gore (`Projectile.cs:50147-
/// 50159`): the `Next(4)` branch gate; the taken branch's 14-dust burst is
/// the dust lane's, then ONE gore `Next(580, 583)` at `Center + UnitX *
/// -direction * 8`, post `velocity /= 2`, `velocity.Y = abs`,
/// `velocity.X = -abs * direction`. Returns whether the 1-in-4 branch ran
/// (the else-branch's `:50170` pair is wave C). The `velocity.X +=
/// NextFloatDirection * NextFloat * 5` (`:50146`) is the AI arm's movement.
pub fn pirate_jump_gore(
    p: &Projectile,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) -> bool {
    if rng.next_max(4) != 0 {
        return false;
    }
    let ty = rng.next_range(580, 583);
    let pos = center(p) + Vec2::new(-p.direction as f32 * 8.0, 0.0);
    let idx = gore.new_gore(rng, world, pos, Vec2::ZERO, ty, 1.0);
    let g = gore.slot_mut(idx);
    g.velocity /= 2.0;
    g.velocity.y = g.velocity.y.abs();
    g.velocity.x = -g.velocity.x.abs() * p.direction as f32;
    true
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
        p.direction = 1;
        p
    }

    /// `Projectile.cs:24265-24271` — the lava bounce: Next(3) gate, the
    /// Next(61,64) type + kicks, then the three post-writes (no draws).
    #[test]
    fn w4_g16_the_lava_bounce_gore_gates_on_three() {
        let w = air();
        let p = proj();
        let mut seed = 0;
        while UnifiedRandom::new(seed).next_max(3) != 0 {
            seed += 1;
        }
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(seed);
        lava_bounce_gore(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(seed);
        let _ = twin.next_max(3);
        let ty = twin.next_range(61, 64);
        let ky = twin.next_range(10, 31);
        let kx = twin.next_range(-20, 21);
        assert_eq!(rng.peek(), twin.peek());
        let g = gore.slot(0);
        assert_eq!(g.gore_type, ty);
        assert_eq!(g.position, Vec2::new(94.0, 100.0)); // pos.X - 6
        let kicked = Vec2::new(kx as f32 * 0.1, -(ky as f32 * 0.1));
        assert_eq!(g.velocity, kicked * 0.4 - Vec2::new(0.0, 1.0));
    }

    /// `Projectile.cs:25438-25444` — the drill gore: Next(5) gate, TWO
    /// NextDouble (the square), one NextFloat (the Y velocity), Next(3)
    /// (the type), the kicks; the 1007/1008 SpecialAI==3 arm then draws
    /// FOUR more inside the pool (`Gore.cs:1391-1402`: two NextFloat +
    /// Next(8) row + Next(8) counter); post timeLeft/alpha/velocity.X draw
    /// nothing.
    #[test]
    fn w4_g16_the_drill_gore_uses_two_doubles_for_the_square() {
        let w = air();
        let p = proj();
        let mut seed = 0;
        while UnifiedRandom::new(seed).next_max(5) != 0 {
            seed += 1;
        }
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(seed);
        drill_unit_gore(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(seed);
        let _ = twin.next_max(5);
        let sx = twin.next_double();
        let sy = twin.next_double();
        let _vy = twin.next_float();
        let pick = twin.next_max(3);
        let _ky = twin.next_range(10, 31);
        let _kx = twin.next_range(-20, 21);
        // The SpecialAI==3 arm (1007/1008 are both in the set, GoreID.cs:20).
        let _vel_x = twin.next_float();
        let _vel_y = twin.next_float();
        let _row = twin.next_max(8);
        let _counter = twin.next_max(8);
        assert_eq!(rng.peek(), twin.peek(), "square = two NextDouble + the SpecialAI four");
        let g = gore.slot(0);
        assert_eq!(g.gore_type, [1007, 1008, 1008][pick as usize]);
        assert_eq!(g.time_left, 60);
        assert_eq!(g.alpha, 50);
        let expect_pos = p.pos + Vec2::new(sx as f32 * p.w, sy as f32 * p.h);
        assert_eq!(g.position, expect_pos);
    }

    /// `Projectile.cs:40295-40303` — the 1025 burst: 9 gores (3 per
    /// iteration), the per-gore draw rows in order. The twin replays the
    /// whole 3-iteration stream.
    #[test]
    fn w4_g16_the_tile_collision_burst_spawns_nine_gores_in_order() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(4);
        tile_collision_burst(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(4);
        let circular = |r: &mut UnifiedRandom| {
            let a = std::f32::consts::PI * 2.0 * r.next_float();
            let _u = Vec2::new((a as f64).cos() as f32, (a as f64).sin() as f32);
            let _rad = r.next_float();
        };
        for _ in 0..3 {
            // site 15: Circular + Next(3) + kicks + drop.
            circular(&mut twin);
            let _ = twin.next_max(3);
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
            let _ = twin.next_float();
            // site 16: Next(5) pair + Circular + Next(3) + kicks + drop.
            let _ = twin.next_max(5);
            let _ = twin.next_max(5);
            circular(&mut twin);
            let _ = twin.next_max(3);
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
            let _ = twin.next_float();
            // site 17: Next(5) pair + Circular + kicks + drop (no type).
            let _ = twin.next_max(5);
            let _ = twin.next_max(5);
            circular(&mut twin);
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
            let _ = twin.next_float();
        }
        assert_eq!(rng.peek(), twin.peek(), "the 9-gore stream mirrors");
        let active = gore.slots()[..9].iter().filter(|g| g.active).count();
        assert_eq!(active, 9);
        // The crystal gores (indices 2, 5, 8) are type 1146.
        assert_eq!(gore.slot(2).gore_type, 1146);
        assert_eq!(gore.slot(5).gore_type, 1146);
        assert_eq!(gore.slot(8).gore_type, 1146);
    }

    /// `Projectile.cs:36326-36342` — the crystal burst (the gap fill):
    /// 20 gores, per gore the angle float + the radius float + Next(4) +
    /// kicks; the flip chain draws nothing.
    #[test]
    fn w4_g16_the_crystal_burst_draws_angle_then_radius() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(83);
        crystal_destruction_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(83);
        for _ in 0..20 {
            let _ = twin.next_float(); // the angle jitter
            let _ = twin.next_float(); // the radius
            let _ = twin.next_max(4); // SelectRandom
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek());
        let active = gore.slots()[..20].iter().filter(|g| g.active).count();
        assert_eq!(active, 20);
    }

    /// `Projectile.cs:45303-45310` — the fire explosion: 2 gores, each the
    /// Next(100) pair + Next(61,64) + kicks, then RotatedByRandom's TWO
    /// NextDouble + the radius NextDouble + the two Next(-10,11) kicks.
    #[test]
    fn w4_g16_the_fire_explosion_gores_draw_the_random_rotation() {
        let w = air();
        let p = proj();
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(6);
        fire_explosion_gores(&p, &mut gore, &mut rng, &w);
        let mut twin = UnifiedRandom::new(6);
        for _ in 0..2 {
            let _ = twin.next_max(100);
            let _ = twin.next_max(100);
            let _ = twin.next_range(61, 64);
            let _ = twin.next_range(10, 31);
            let _ = twin.next_range(-20, 21);
            let _ = twin.next_double(); // RotatedByRandom a
            let _ = twin.next_double(); // RotatedByRandom b
            let _ = twin.next_double(); // the radius
            let _ = twin.next_range(-10, 11);
            let _ = twin.next_range(-10, 11);
        }
        assert_eq!(rng.peek(), twin.peek(), "the 2-gore stream mirrors");
        let active = gore.slots()[..2].iter().filter(|g| g.active).count();
        assert_eq!(active, 2);
    }

    /// `Projectile.cs:50147-50159` — the pirate jump: the Next(4) gate; a
    /// taken branch draws Next(580,583) + kicks and returns true, a missed
    /// gate draws ONLY the Next(4) and returns false.
    #[test]
    fn w4_g16_the_pirate_jump_gore_returns_the_branch() {
        let w = air();
        let p = proj();
        // Taken seed.
        let mut seed = 0;
        while UnifiedRandom::new(seed).next_max(4) != 0 {
            seed += 1;
        }
        let mut gore = GorePool::new();
        let mut rng = UnifiedRandom::new(seed);
        assert!(pirate_jump_gore(&p, &mut gore, &mut rng, &w));
        let mut twin = UnifiedRandom::new(seed);
        let _ = twin.next_max(4);
        let ty = twin.next_range(580, 583);
        let _ = twin.next_range(10, 31);
        let _ = twin.next_range(-20, 21);
        assert_eq!(rng.peek(), twin.peek());
        assert_eq!(gore.slot(0).gore_type, ty);
        // Missed seed: only the gate draws.
        let mut miss = 0;
        while UnifiedRandom::new(miss).next_max(4) == 0 {
            miss += 1;
        }
        let mut gore2 = GorePool::new();
        let mut rng2 = UnifiedRandom::new(miss);
        assert!(!pirate_jump_gore(&p, &mut gore2, &mut rng2, &w));
        let mut twin2 = UnifiedRandom::new(miss);
        let _ = twin2.next_max(4);
        assert_eq!(rng2.peek(), twin2.peek());
    }
}
