//! Projectile DEATH dust — the per-kind bursts inside `Projectile.Kill()`
//! (`decompiled/Terraria/Projectile.cs:70642-77262`), W4-g65.
//!
//! # Scope
//!
//! The TRAIL dust (the per-tick AI arms, `AI_001`/`AI_026`/…) is the g63/g64
//! lanes; this module owns only the dust that fires ONCE when a slot dies —
//! the `Kill()` body's per-type `Dust.NewDust*` bursts. The i18 special-bullet
//! deaths (`:75250-76039`) and the i17 special-arrow deaths stay with their
//! main-landed modules; only UNCLAIMED arms are ported here.
//!
//! # Wiring (the gallop-dust queue→drain precedent)
//!
//! The pool cannot borrow the App's `DustPool` or hold the live
//! `UpdateProjectiles` stream alongside its `rng` parameter, so the death hook
//! (`ProjectilePool::update`'s `!active` block, `projectile.rs`) only SNAPSHOTS
//! the dying slot into a [`DeathSnapshot`] — no draws there. The world-sim pass
//! drains the snapshots at its tail ([`drain`]) onto the real
//! `DustPool::new_dust` on the live `UpdateProjectiles` stream: every explicit
//! `Main.rand` draw the C# arm takes AND the `new_dust` internals run there, in
//! C# order, per death in slot order. This is the recorded gallop-dust
//! deviation (`world_sim/npcs.rs`, ANALYSIS "W-10 unicorn tails"): the dust
//! draws land at the pass tail rather than mid-walk, stream-neutral for the
//! port's per-consumer streams.
//!
//! # The DustPool contract (origin/task/W4-d50-dust-type-table)
//!
//! The bodies are emitted through the verbatim `DustPool::new_dust` family
//! (`tr_core::particles`, S1-E-1): the guard chain, the off-screen cull, the
//! pressure bands, the default block, the ChildSafety re-type and the per-type
//! tweak chain all run there. This module only resolves the C# arm's ARGUMENTS
//! and its post-spawn field writes (`noGravity`, `scale *=`, `velocity *=`,
//! `fadeIn`, …); the drain applies the writes to whatever slot `new_dust`
//! returned — the sentinel included (the C# writes `Main.dust[6000]` too).
//!
//! # RNG census
//!
//! Per arm, the explicit draws are listed at the arm below in source order;
//! they ride the drain's `rng` (the live `UpdateProjectiles` stream) ahead of
//! that arm's `new_dust` internals. An arm whose only randomness is the
//! internals (e.g. the 17/31/42 plain bursts) spends its stream entirely inside
//! `new_dust`.
//!
//! CODE-ONLY MODE (W4-g65): no compiler invoked on this branch; the witness
//! tests at the bottom are WRITTEN NOT RUN, every expected value hand-derived
//! from the cited C# lines on a cloned `UnifiedRandom`.

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::UnifiedRandom;
use tr_core::World;

use crate::projectile::Projectile;

/// `Color.Lerp(a, b, t)` (XNA): the per-channel byte lerp
/// `(byte)(a + (b - a) * t)` — the 384/386 CornflowerBlue→White blend.
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    Color::new(l(a.r, b.r), l(a.g, b.g), l(a.b, b.b), l(a.a, b.a))
}

/// `Vector2.Normalize` with the zero guard the walking arms rely on (the C#
/// `vector53.Normalize()` on a zero vector leaves it zero in XNA's
/// `Normalize` — the port's `normalize_or_zero` matches).
fn safe_normalize(v: Vec2) -> Vec2 {
    v.normalize_or_zero()
}

/// `Utils.ToRotationVector2(angle)` (`Utils.cs:1665-1668`): `(cos, sin)` in
/// double of the widened float, cast back to f32.
fn rotation_vector(angle: f32) -> Vec2 {
    Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32)
}

/// `Vector2.Lerp(a, b, t)` (XNA): `a + (b - a) * t` per axis — the center-lerp
/// death arms' position write.
fn lerp_vec(a: Vec2, b: Vec2, t: f32) -> Vec2 {
    a + (b - a) * t
}

/// `Vector2.RotatedBy(angle)` about the origin (XNA): the float cos/sin spin —
/// the 385 ring's per-spoke rotation.
fn rotate_vec(v: Vec2, angle: f32) -> Vec2 {
    let (s, c) = angle.sin_cos();
    Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}

/// The trail-position dust PAIR (`:77064-77073`): for `i in start..end`, two
/// dust of `dust_type` at `pos − oldVel×(30/i)` box 8×8 speed oldVel alpha
/// `alpha`, scale `scale_a` (noGravity when `ng_a`, vel×`mul_a`) + scale
/// `scale_b` (noGravity when `ng_b`, vel×`mul_b`). The 115/116 shape (position
/// = `s.pos`). No explicit draw.
#[allow(clippy::too_many_arguments)]
fn trail_pair(
    s: &DeathSnapshot,
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
    start: i32,
    end: i32,
    dust_type: i32,
    alpha: i32,
    scale_a: f32,
    scale_b: f32,
    ng_a: bool,
    ng_b: bool,
    mul_a: f32,
    mul_b: f32,
) {
    for i in start..end {
        let off = s.old_vel * (30.0 / i as f32);
        let pos = s.pos - off;
        let idx = dust.new_dust(rng, ctx, world, pos, 8, 8, dust_type, s.old_vel.x, s.old_vel.y, alpha, Color::default(), scale_a);
        let d = dust.slot_mut(idx);
        if ng_a {
            d.no_gravity = true;
        }
        d.velocity *= mul_a;
        let idx = dust.new_dust(rng, ctx, world, pos, 8, 8, dust_type, s.old_vel.x, s.old_vel.y, alpha, Color::default(), scale_b);
        let d = dust.slot_mut(idx);
        if ng_b {
            d.no_gravity = true;
        }
        d.velocity *= mul_b;
    }
}

/// The trail-position dust PAIR at `oldPos` (`:77125-77134`): the [`trail_pair`]
/// shape but the position base is `s.old_pos` — the 132/156 shape. No explicit
/// draw.
#[allow(clippy::too_many_arguments)]
fn trail_pair_oldpos(
    s: &DeathSnapshot,
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
    start: i32,
    end: i32,
    dust_type: i32,
    alpha: i32,
    scale_a: f32,
    scale_b: f32,
    ng_a: bool,
    ng_b: bool,
    mul_a: f32,
    mul_b: f32,
) {
    for i in start..end {
        let off = s.old_vel * (30.0 / i as f32);
        let pos = s.old_pos - off;
        let idx = dust.new_dust(rng, ctx, world, pos, 8, 8, dust_type, s.old_vel.x, s.old_vel.y, alpha, Color::default(), scale_a);
        let d = dust.slot_mut(idx);
        if ng_a {
            d.no_gravity = true;
        }
        d.velocity *= mul_a;
        let idx = dust.new_dust(rng, ctx, world, pos, 8, 8, dust_type, s.old_vel.x, s.old_vel.y, alpha, Color::default(), scale_b);
        let d = dust.slot_mut(idx);
        if ng_b {
            d.no_gravity = true;
        }
        d.velocity *= mul_b;
    }
}

/// `Utils.NextVector2Circular(rng, halfW, halfH)` (`Utils.cs:1266-1269`):
/// `NextVector2Unit() * (halfW, halfH) * NextFloat()` — the unit draw (one
/// `NextFloat`, float angle, double cos/sin) THEN a second `NextFloat` radius.
fn next_vector2_circular(rng: &mut UnifiedRandom, half_w: f32, half_h: f32) -> Vec2 {
    let angle = std::f32::consts::PI * 2.0 * rng.next_float();
    let unit = Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32);
    let r = rng.next_float();
    Vec2::new(unit.x * half_w * r, unit.y * half_h * r)
}

/// The bottom-edge single-dust arm (`:74477-74485` et al.): gated
/// `velocity.Y > 0`, one dust of `dust_type` at `pos + (0, h−2)` box 2×2,
/// `position.X -= 2`, `alpha = 38`, `velocity *= 0.1`, `velocity += −oldVel ×
/// 0.25`, `scale = 0.95`, optionally `noGravity`. The 239/245/264 shape. No
/// explicit draw.
fn bottom_dust(
    s: &DeathSnapshot,
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
    dust_type: i32,
    no_gravity: bool,
) {
    if s.vel.y > 0.0 {
        let pos = Vec2::new(s.pos.x, s.pos.y + s.h as f32 - 2.0);
        let idx = dust.new_dust(rng, ctx, world, pos, 2, 2, dust_type, 0.0, 0.0, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        if no_gravity {
            d.no_gravity = true;
        }
        d.position.x -= 2.0;
        d.alpha = 38;
        d.velocity *= 0.1;
        d.velocity += -s.old_vel * 0.25;
        d.scale = 0.95;
    }
}

/// The center-lerp dust burst (`:73469-73477` et al.): `Next(count_min,
/// count_max)` dust of `dust_type` at Center 0×0 alpha 100 scale `scale`, each
/// `velocity *= vel_mul`, `velocity.Y -= 1`, optionally `position -= One×4`,
/// then `position = Lerp(position, Center, 0.5)`, `noGravity = true`. The
/// 435/436/462/631 shape. The count draw rides BEFORE the spawns.
#[allow(clippy::too_many_arguments)]
fn center_lerp_burst(
    s: &DeathSnapshot,
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
    count_min: i32,
    count_max: i32,
    dust_type: i32,
    scale: f32,
    vel_mul: f32,
    subtract_one4: bool,
) {
    let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
    let count = rng.next_range(count_min, count_max);
    for _ in 0..count {
        let idx = dust.new_dust(rng, ctx, world, center, 0, 0, dust_type, 0.0, 0.0, 100, Color::default(), scale);
        let d = dust.slot_mut(idx);
        d.velocity *= vel_mul;
        d.velocity.y -= 1.0;
        if subtract_one4 {
            d.position -= Vec2::ONE * 4.0;
        }
        d.position = lerp_vec(d.position, center, 0.5);
        d.no_gravity = true;
    }
}

/// The dying slot's snapshot the death arms read (`base.position`,
/// `base.width/height`, `base.velocity`, `base.scale`, `base.ai`). Taken at the
/// death hook BEFORE any other hook mutates the corpse (the C# `Kill()` reads
/// the live slot throughout its body). `kind` selects the arm.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DeathSnapshot {
    pub kind: u16,
    pub pos: Vec2,
    pub w: i32,
    pub h: i32,
    pub vel: Vec2,
    pub scale: f32,
    /// `base.oldVelocity` — the 93/323-family death arms read it. The port's
    /// pool latches `old_velocity` per sub-step; the snapshot takes the slot's
    /// current velocity as the stand-in (the resting kinds' `oldVelocity ==
    /// velocity` at death, the special-bullets note). Kinds that need the true
    /// last-sub-step velocity are deferred to a later commit.
    pub old_vel: Vec2,
    /// `base.ai[1]` — the 346/335/589 colour/type picks read it.
    pub ai1: f32,
    /// `base.rotation` — the 598/971 walking-position arms read it.
    pub rot: f32,
    /// `base.direction` — the 384/386 speed argument reads it.
    pub direction: i32,
    /// `base.alpha` — the 206 death gate reads it (`alpha < 255`).
    pub alpha: i32,
    /// `base.oldPosition` — the 132/156/343 trail-position arms read it. The
    /// port's pool latches `old_position` per sub-step; the snapshot takes the
    /// slot's current position as the stand-in (the resting kinds' oldPosition
    /// == position at death). Kinds needing the true latch are deferred.
    pub old_pos: Vec2,
}

impl DeathSnapshot {
    /// Snapshot a dying slot. The box truncates to `int` exactly as the C#
    /// `base.width`/`base.height` (the `int` fields) read.
    pub(crate) fn of(p: &Projectile) -> Self {
        Self {
            kind: p.kind,
            pos: p.pos,
            w: p.w as i32,
            h: p.h as i32,
            vel: p.vel,
            scale: p.scale,
            old_vel: p.vel,
            ai1: p.ai[1],
            rot: p.rot,
            direction: p.direction,
            alpha: p.alpha,
            old_pos: p.pos,
        }
    }
}

/// Drain the tick's death snapshots onto the dust pool, in slot order, on the
/// live `UpdateProjectiles` stream (`rng`). Each arm takes its explicit C#
/// draws then its `new_dust` internals + post-spawn writes. `ctx` is the App's
/// per-call `SpawnCtx` (the `Dust.cs:77-101` guards/cull); `world` feeds the
/// type-34 wet probe.
pub(crate) fn drain(
    snapshots: &[DeathSnapshot],
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
) {
    for s in snapshots {
        death_bursts(s, dust, rng, ctx, world);
    }
}

/// `Projectile.Kill()`'s per-kind death-dust bursts for one dying slot. Takes
/// every explicit `Main.rand` draw on `rng` in C# order, then emits the bodies
/// through `dust.new_dust` and applies the arm's post-spawn writes. Kind-gated;
/// a dead slot never returns to active, so this fires at most once per death.
fn death_bursts(
    s: &DeathSnapshot,
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
) {
    match s.kind {
        // `:76170-76177` — the 17: 5 plain dust-0 (no explicit draw).
        17 => plain_burst(s, dust, rng, ctx, world, 5, 0, 0.0, 0.0, 0, Color::default(), 1.0, None, false),
        // `:76178-76186` — the 31/42: 5 dust-32, velocity ×0.6.
        31 | 42 => plain_burst(s, dust, rng, ctx, world, 5, 32, 0.0, 0.0, 0, Color::default(), 1.0, Some(0.6), false),
        // `:76214-76222` — the 39: 5 dust-38, velocity ×0.6.
        39 => plain_burst(s, dust, rng, ctx, world, 5, 38, 0.0, 0.0, 0, Color::default(), 1.0, Some(0.6), false),
        // `:76223-76231` — the 71: 5 dust-53, velocity ×0.6.
        71 => plain_burst(s, dust, rng, ctx, world, 5, 53, 0.0, 0.0, 0, Color::default(), 1.0, Some(0.6), false),
        // `:76232-76240` — the 179: 5 dust-149, velocity ×0.6.
        179 => plain_burst(s, dust, rng, ctx, world, 5, 149, 0.0, 0.0, 0, Color::default(), 1.0, Some(0.6), false),
        // `:76241-76249` — the 40: 5 dust-36, velocity ×0.6.
        40 => plain_burst(s, dust, rng, ctx, world, 5, 36, 0.0, 0.0, 0, Color::default(), 1.0, Some(0.6), false),
        // `:76205-76213` — the 109: 5 dust-51 scale 0.6, velocity ×0.6.
        109 => plain_burst(s, dust, rng, ctx, world, 5, 51, 0.0, 0.0, 0, Color::default(), 0.6, Some(0.6), false),
        // `:76250-76257` — the 21/471/532/1111: 10 dust-26 scale 0.8.
        21 | 471 | 532 | 1111 => plain_burst(s, dust, rng, ctx, world, 10, 26, 0.0, 0.0, 0, Color::default(), 0.8, None, false),
        // `:76274-76280` — the 24: 10 dust-1, speed = vel×0.1, scale 0.75.
        24 => plain_burst(s, dust, rng, ctx, world, 10, 1, s.vel.x * 0.1, s.vel.y * 0.1, 0, Color::default(), 0.75, None, false),
        // `:76291-76297` — the 38: 10 dust-42, speed = vel×0.1.
        38 => plain_burst(s, dust, rng, ctx, world, 10, 42, s.vel.x * 0.1, s.vel.y * 0.1, 0, Color::default(), 1.0, None, false),
        // `:75616-75629` — the 103 CursedArrow: 20 dust-75 alpha 100; per dust
        // a `Next(2)` gate → scale ×2.5 + noGravity + velocity ×5.
        103 => gated_burst(s, dust, rng, ctx, world, 20, 75, 0.0, 0.0, 100, Color::default(), 1.0, 2, 2.5, 5.0),
        // `:75630-75643` — the 278 IchorArrow: 20 dust-169 alpha 100; per dust
        // a `Next(2)` gate → scale ×1.5 + noGravity + velocity ×5.
        278 => gated_burst(s, dust, rng, ctx, world, 20, 169, 0.0, 0.0, 100, Color::default(), 1.0, 2, 1.5, 5.0),
        // `:75668-75680` — the 5 JestersArrow: 60 dust, type = `Next(3)` switch
        // {0→15, 1→57, _→58}, speed = vel×0.5, alpha 150, scale 1.5.
        5 => {
            for _ in 0..60 {
                let dust_type = match rng.next_max(3) {
                    0 => 15,
                    1 => 57,
                    _ => 58,
                };
                dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, dust_type, s.vel.x * 0.5, s.vel.y * 0.5, 150, Color::default(), 1.5);
            }
        }
        // `:74828-74840` — the 93: 10 dust-57 alpha 100, noGravity + fadeIn 1 +
        // velocity ×2 (both axes) then `velocity.Y -= NextFloat()*1.5`.
        93 => {
            for _ in 0..10 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 57, 0.0, 0.0, 100, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.fade_in = 1.0;
                d.velocity *= 2.0;
                d.velocity.y -= rng.next_float() * 1.5;
            }
        }
        // `:74260-74271` — the 318 RottenEgg: 10 dust-30; per dust a `Next(2)`
        // gate → noGravity.
        318 => no_gravity_gate_burst(s, dust, rng, ctx, world, 10, 30),
        // `:74272-74282` — the 378: 10 dust-30; per dust a `Next(2)` gate →
        // noGravity (the same shape as the 318).
        378 => no_gravity_gate_burst(s, dust, rng, ctx, world, 10, 30),
        // `:74236-74249` — the 346: 20 dust, type = 10 (4 when ai[1]==1),
        // noGravity. No explicit draw.
        346 => {
            let dust_type = if s.ai1 == 1.0 { 4 } else { 10 };
            plain_burst(s, dust, rng, ctx, world, 20, dust_type, 0.0, 0.0, 0, Color::default(), 1.0, None, true);
        }
        // `:74250-74259` — the 335: 20 dust, type = 90 - (int)ai[1], noLight +
        // scale 0.8. No explicit draw.
        335 => {
            let dust_type = 90 - s.ai1 as i32;
            for _ in 0..20 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, dust_type, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.no_light = true;
                d.scale = 0.8;
            }
        }
        // `:75554-75561` — the 76/77/78: 5 dust-27 alpha 80 scale 1.5,
        // noGravity. No explicit draw.
        76 | 77 | 78 => plain_burst(s, dust, rng, ctx, world, 5, 27, 0.0, 0.0, 80, Color::default(), 1.5, None, true),
        // `:75562-75569` — the 55/719: 5 dust-18 scale 1.5, noGravity. No
        // explicit draw.
        55 | 719 => plain_burst(s, dust, rng, ctx, world, 5, 18, 0.0, 0.0, 0, Color::default(), 1.5, None, true),
        // `:74770-74779` — the 158: 10 dust-9, noGravity + velocity -= vel×0.5.
        158 => velocity_sub_burst(s, dust, rng, ctx, world, 10, 9, 0.5),
        // `:74780-74789` — the 159: 10 dust-11, noGravity + velocity -= vel×0.5.
        159 => velocity_sub_burst(s, dust, rng, ctx, world, 10, 11, 0.5),
        // `:74790-74799` — the 160: 10 dust-19, noGravity + velocity -= vel×0.5.
        160 => velocity_sub_burst(s, dust, rng, ctx, world, 10, 19, 0.5),
        // `:74800-74809` — the 161: 10 dust-11, noGravity + velocity -= vel×0.5.
        161 => velocity_sub_burst(s, dust, rng, ctx, world, 10, 11, 0.5),
        // `:74752-74759` — the 1124: 5 dust-26 (no post write, no draw).
        1124 => plain_burst(s, dust, rng, ctx, world, 5, 26, 0.0, 0.0, 0, Color::default(), 1.0, None, false),
        // `:75592-75599` — the 2/82: 10 dust-6 alpha 100 (no post write).
        2 | 82 => plain_burst(s, dust, rng, ctx, world, 10, 6, 0.0, 0.0, 100, Color::default(), 1.0, None, false),
        // `:75570-75577` — the 51/267: 5 dust-0 scale 0.7 (no post write).
        51 | 267 => plain_burst(s, dust, rng, ctx, world, 5, 0, 0.0, 0.0, 0, Color::default(), 0.7, None, false),
        // `:75644-75651` — the 3/48/54/599/909: 10 dust-1, speed = vel×0.1,
        // scale 0.75 (no post write).
        3 | 48 | 54 | 599 | 909 => plain_burst(s, dust, rng, ctx, world, 10, 1, s.vel.x * 0.1, s.vel.y * 0.1, 0, Color::default(), 0.75, None, false),
        // `:75652-75659` — the 330: 10 dust-0, speed = vel×0.4, scale 0.75.
        330 => plain_burst(s, dust, rng, ctx, world, 10, 0, s.vel.x * 0.4, s.vel.y * 0.4, 0, Color::default(), 0.75, None, false),
        // `:75660-75667` — the 4 UnholyArrow: 10 dust-14 alpha 150 scale 1.1.
        4 => plain_burst(s, dust, rng, ctx, world, 10, 14, 0.0, 0.0, 150, Color::default(), 1.1, None, false),
        // `:76134-76145` — the 15/321: 20 iterations, each a dust-6 scale 2
        // (noGravity, vel×2) THEN a dust-6 scale 1 (vel×2), speed = −vel×0.2,
        // alpha 100. No explicit draw.
        15 | 321 => reverse_vel_pair(s, dust, rng, ctx, world, 20, 6, 2.0, 1.0),
        // `:76146-76157` — the 253: the same pair shape, dust-135.
        253 => reverse_vel_pair(s, dust, rng, ctx, world, 20, 135, 2.0, 1.0),
        // `:76158-76169` — the 95/96: the pair shape, dust-75, scales 2×scale
        // and 1×scale (the projectile's own scale rides the dust scale).
        95 | 96 => reverse_vel_pair(s, dust, rng, ctx, world, 20, 75, 2.0 * s.scale, 1.0 * s.scale),
        // `:76281-76290` — the 27: 30 iterations, each a dust-172 (noGravity,
        // scale 1) THEN a dust-172 scale 0.5, speed = vel×0.1, alpha 100.
        27 => {
            for _ in 0..30 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 172, s.vel.x * 0.1, s.vel.y * 0.1, 100, Color::default(), 1.0);
                dust.slot_mut(idx).no_gravity = true;
                dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 172, s.vel.x * 0.1, s.vel.y * 0.1, 100, Color::default(), 0.5);
            }
        }
        // `:76298-76307` — the 44/45: 30 iterations, each a dust-27 scale 1.7
        // (noGravity) THEN a dust-27 scale 1, speed = vel (full), alpha 100.
        44 | 45 => {
            for _ in 0..30 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 27, s.vel.x, s.vel.y, 100, Color::default(), 1.7);
                dust.slot_mut(idx).no_gravity = true;
                dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 27, s.vel.x, s.vel.y, 100, Color::default(), 1.0);
            }
        }
        // `:73583-73591` — the 495 ShadowflameArrow: 15 dust-27 at Center 10×10,
        // noGravity + velocity −= oldVel×0.3. No explicit draw.
        495 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            for _ in 0..15 {
                let idx = dust.new_dust(rng, ctx, world, center, 10, 10, 27, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity -= s.old_vel * 0.3;
            }
        }
        // `:73592-73602` — the 497: 15 dust-27 at Center 10×10, noGravity +
        // vel×2 −= oldVel×0.3 + scale += Next(150)*0.001.
        497 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            for _ in 0..15 {
                let idx = dust.new_dust(rng, ctx, world, center, 10, 10, 27, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 2.0;
                d.velocity -= s.old_vel * 0.3;
                d.scale += rng.next_max(150) as f32 * 0.001;
            }
        }
        // `:75149-75158` — the 729: 10 dust-279 (NewDustDirect), speed = vel×0.1,
        // alpha 150 scale 1.2, noGravity + velocity.X ×= 2. No explicit draw.
        729 => {
            for _ in 0..10 {
                let idx = dust.new_dust_direct(rng, ctx, world, s.pos, s.w, s.h, 279, s.vel.x * 0.1, s.vel.y * 0.1, 150, Color::default(), 1.2);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity.x *= 2.0;
            }
        }
        // `:75486-75499` — the 731: 15 dust-226 (NewDustDirect), speed =
        // oldVel×0.5, alpha 50 scale 0.5; per dust a Next(3) gate → noGravity +
        // scale×1.5 + vel×0.5.
        731 => {
            for _ in 0..15 {
                let idx = dust.new_dust_direct(rng, ctx, world, s.pos, s.w, s.h, 226, s.old_vel.x * 0.5, s.old_vel.y * 0.5, 50, Color::default(), 0.5);
                if rng.next_max(3) != 0 {
                    let d = dust.slot_mut(idx);
                    d.no_gravity = true;
                    d.scale *= 1.5;
                    d.velocity *= 0.5;
                }
            }
        }
        // `:75513-75521` — the 337: 10 dust-197, noGravity. No explicit draw.
        337 => plain_burst(s, dust, rng, ctx, world, 10, 197, 0.0, 0.0, 0, Color::default(), 1.0, None, true),
        // `:72679-72691` — the 581: 30 dust, type = SelectRandom{229,229,161},
        // noGravity + scale 1.25+NextFloat + fadeIn 0.25 + vel×2 + noLight.
        581 => select_random_burst(s, dust, rng, ctx, world, 30, &[229, 229, 161], 1.25, 1.0, 0.25, 2.0, true),
        // `:72692-72704` — the 671: 30 dust, type = SelectRandom{27,27,62},
        // the same writes as the 581.
        671 => select_random_burst(s, dust, rng, ctx, world, 30, &[27, 27, 62], 1.25, 1.0, 0.25, 2.0, true),
        // `:72705-72713` — the 811/814: 30 dust-5, scale 1.25+NextFloat + vel×2
        // (no noGravity/fadeIn/noLight).
        811 | 814 => {
            for _ in 0..30 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 5, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.scale = 1.25 + rng.next_float();
                d.velocity *= 2.0;
            }
        }
        // `:72714-72723` — the 819: 20 dust-5, scale 1+NextFloat×0.5 + vel×1.5.
        819 => {
            for _ in 0..20 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 5, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.scale = 1.0 + rng.next_float() * 0.5;
                d.velocity *= 1.5;
            }
        }
        // `:72724-72736` — the 675: 40 dust, type = SelectRandom{27,242,73,72,
        // 71,255,255,255}, noGravity + scale 1.25+NextFloat + fadeIn 0.25 +
        // vel×3 + noLight.
        675 => select_random_burst(s, dust, rng, ctx, world, 40, &[27, 242, 73, 72, 71, 255, 255, 255], 1.25, 1.0, 0.25, 3.0, true),
        // `:72668-72678` — the 572: 15 dust-40 alpha 100 speed vel×0.1, vel×3 +
        // noGravity + scale 1.25 + position = (Center+position)/2. No draw.
        572 => {
            let mid = (Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5) + s.pos) / 2.0;
            for _ in 0..15 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 40, s.vel.x * 0.1, s.vel.y * 0.1, 100, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.velocity *= 3.0;
                d.no_gravity = true;
                d.scale = 1.25;
                d.position = mid;
            }
        }
        // `:72374-72393` — the 658: 10 dust-269 speed vel×0.1 scale 0.5; per
        // dust a Next(3) gate → fadeIn 0.75+Next(-10,11)*0.01 + scale
        // 0.25+Next(-10,11)*0.005 + type++ ; else scale 1+Next(-10,11)*0.01.
        // Then noGravity + vel×1.25 −= oldVel/10.
        658 => crystal_spark_burst(s, dust, rng, ctx, world, 10, 269, 0.75, 0.25, 0.005, 1.0, 0.01, 1.25),
        // `:73271-73291` — the 522: the 658 shape, dust-254.
        522 => crystal_spark_burst(s, dust, rng, ctx, world, 10, 254, 0.75, 0.25, 0.005, 1.0, 0.01, 1.25),
        // `:73328-73337` — the 520: 10 dust-252 speed vel×0.1 scale 0.75,
        // noGravity + velocity −= oldVel/3. No explicit draw.
        520 => {
            for _ in 0..10 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 252, s.vel.x * 0.1, s.vel.y * 0.1, 0, Color::default(), 0.75);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity -= s.old_vel / 3.0;
            }
        }
        // `:73952-73962` — the 384/386: 20 dust-212, speed = direction×2, alpha
        // 100 scale 1.4, color = Lerp(CornflowerBlue, White, 0.3), noGravity.
        384 | 386 => {
            let color = lerp_color(Color::new(100, 149, 237, 255), Color::WHITE, 0.3);
            for _ in 0..20 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 212, s.direction as f32 * 2.0, 0.0, 100, Color::default(), 1.4);
                let d = dust.slot_mut(idx);
                d.color = color;
                d.no_gravity = true;
            }
        }
        // `:73963-73979` — the 507/508: 20 dust-81 walking from pos+norm(oldVel)
        // ×16 back by ×8 each; position = (dustPos+Center)/2, velocity +=
        // oldVel×0.4 then ×0.5, noGravity. No explicit draw.
        507 | 508 => {
            let dir = safe_normalize(s.old_vel);
            let mut walk = s.pos + dir * 16.0;
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            for _ in 0..20 {
                let idx = dust.new_dust(rng, ctx, world, walk, s.w, s.h, 81, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.position = (d.position + center) / 2.0;
                d.velocity += s.old_vel * 0.4;
                d.velocity *= 0.5;
                d.no_gravity = true;
                walk -= dir * 8.0;
            }
        }
        // `:73980-73995` — the 598: the 507 walk shape but the direction is
        // (rotation−π/2).ToRotationVector2(), velocity += dir×2.
        598 => {
            let dir = rotation_vector(s.rot - std::f32::consts::FRAC_PI_2);
            let mut walk = s.pos + dir * 16.0;
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            for _ in 0..20 {
                let idx = dust.new_dust(rng, ctx, world, walk, s.w, s.h, 81, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.position = (d.position + center) / 2.0;
                d.velocity += dir * 2.0;
                d.velocity *= 0.5;
                d.no_gravity = true;
                walk -= dir * 8.0;
            }
        }
        // `:73996-74011` — the 971: 6 dust-4 walking from pos+dir×8 back by ×4,
        // color (10,10,10,150), position = (dustPos+Center)/2, velocity += dir
        // then ×0.5, noGravity. dir = (rotation−π/2).ToRotationVector2().
        971 => {
            let dir = rotation_vector(s.rot - std::f32::consts::FRAC_PI_2);
            let mut walk = s.pos + dir * 8.0;
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            for _ in 0..6 {
                let idx = dust.new_dust(rng, ctx, world, walk, s.w, s.h, 4, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.color = Color::new(10, 10, 10, 150);
                d.position = (d.position + center) / 2.0;
                d.velocity += dir;
                d.velocity *= 0.5;
                d.no_gravity = true;
                walk -= dir * 4.0;
            }
        }
        // `:74012-74019` — the 1/81/98/980/1073: 10 dust-7 (no post write).
        1 | 81 | 98 | 980 | 1073 => plain_burst(s, dust, rng, ctx, world, 10, 7, 0.0, 0.0, 0, Color::default(), 1.0, None, false),
        // `:74020-74028` — the 336/345: 6 dust-196, noGravity + scale = proj.scale.
        336 | 345 => {
            for _ in 0..6 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 196, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.scale = s.scale;
            }
        }
        // `:74148-74158` — the 344: 8 dust-197 alpha 50, noGravity + vel×0.75
        // (both axes) −= vel×0.025. No explicit draw.
        344 => {
            for _ in 0..8 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 197, 0.0, 0.0, 50, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 0.75;
                d.velocity -= s.vel * 0.025;
            }
        }
        // `:74171-74181` — the 349: 3 dust-76, noGravity + noLight + scale 0.7.
        349 => {
            for _ in 0..3 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 76, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.no_light = true;
                d.scale = 0.7;
            }
        }
        // `:74303-74310` — the 184/195: 5 dust-7 (no post write).
        184 | 195 => plain_burst(s, dust, rng, ctx, world, 5, 7, 0.0, 0.0, 0, Color::default(), 1.0, None, false),
        // `:74326-74333` — the 275/276: 5 dust-7 (no post write).
        275 | 276 => plain_burst(s, dust, rng, ctx, world, 5, 7, 0.0, 0.0, 0, Color::default(), 1.0, None, false),
        // `:74384-74393` — the 265: 15 dust-163 alpha 100 scale 1.2, noGravity +
        // vel×1.2 −= oldVel×0.3. No explicit draw.
        265 => old_vel_kick_burst(s, dust, rng, ctx, world, 15, 163, 100, 1.2, 1.2, 0.3),
        // `:74394-74403` — the 355: the 265 shape, dust-205.
        355 => old_vel_kick_burst(s, dust, rng, ctx, world, 15, 205, 100, 1.2, 1.2, 0.3),
        // `:74404-74413` — the 304: 3 dust-182 alpha 100 scale 0.8, the same
        // noGravity/vel×1.2 −= oldVel×0.3 writes.
        304 => old_vel_kick_burst(s, dust, rng, ctx, world, 3, 182, 100, 0.8, 1.2, 0.3),
        // `:74414-74423` — the 263: 15 dust-92, speed = vel, alpha =
        // Next(0,101), scale = 1+Next(40)*0.01, noGravity + vel×2. The alpha +
        // scale draws ride BEFORE the spawn (the C# argument order).
        263 => {
            for _ in 0..15 {
                let alpha = rng.next_range(0, 101);
                let scale = 1.0 + rng.next_max(40) as f32 * 0.01;
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 92, s.vel.x, s.vel.y, alpha, Color::default(), scale);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 2.0;
            }
        }
        // `:74424-74441` — the 261: 15 dust-148 (Direct, vel×2 then Y−=0.4) THEN
        // 3 dust-228 (Direct, vel×2 + noLight + scale×0.8 + Y−=0.3). No draw.
        261 => {
            for _ in 0..15 {
                let idx = dust.new_dust_direct(rng, ctx, world, s.pos, s.w, s.h, 148, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.velocity.y -= 0.4;
            }
            for _ in 0..3 {
                let idx = dust.new_dust_direct(rng, ctx, world, s.pos, s.w, s.h, 228, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_light = true;
                d.scale *= 0.8;
                d.velocity.y -= 0.3;
            }
        }
        // `:74442-74449` — the 928: 25 dust-5, speed = vel, color White, scale
        // 1.3 (no post write). No explicit draw.
        928 => plain_burst(s, dust, rng, ctx, world, 25, 5, s.vel.x, s.vel.y, 0, Color::WHITE, 1.3, None, false),
        // `:74524-74534` — the 227: 15 dust-157, noGravity + velocity +=
        // oldVel×NextFloat + scale 1.5.
        227 => {
            for _ in 0..15 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 157, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity += s.old_vel * rng.next_float();
                d.scale = 1.5;
            }
        }
        // `:74543-74555` — the 120 FrostArrow: 10 dust-67 at pos−vel, speed =
        // vel, alpha 100; the first 5 noGravity; all vel×0.2. No explicit draw.
        120 => {
            let pos = s.pos - s.vel;
            for i in 0..10 {
                let idx = dust.new_dust(rng, ctx, world, pos, s.w, s.h, 67, s.vel.x, s.vel.y, 100, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                if i < 5 {
                    d.no_gravity = true;
                }
                d.velocity *= 0.2;
            }
        }
        // `:74556-74564` — the 181/189/566 bees: 6 dust-150, speed = vel, alpha
        // 50, noGravity + scale 1. No explicit draw.
        181 | 189 | 566 => {
            for _ in 0..6 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 150, s.vel.x, s.vel.y, 50, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.scale = 1.0;
            }
        }
        // `:74513-74523` — the 206/225: 5 dust-40, gated `type != 206 ||
        // alpha < 255` (the 206 only bursts while still fading in). No draw.
        206 | 225 => {
            if s.kind != 206 || s.alpha < 255 {
                plain_burst(s, dust, rng, ctx, world, 5, 40, 0.0, 0.0, 0, Color::default(), 1.0, None, false);
            }
        }
        // `:73398-73408` — the 444 Xenopopper bubble: Next(5,9) dust-171 at
        // Center 0×0 alpha 100 scale 1.4, vel×0.8 + position Lerp→Center 0.5 +
        // noGravity. (The owner-gated bullet release is i14's, not here.)
        444 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let count = rng.next_range(5, 9);
            for _ in 0..count {
                let idx = dust.new_dust(rng, ctx, world, center, 0, 0, 171, 0.0, 0.0, 100, Color::default(), 1.4);
                let d = dust.slot_mut(idx);
                d.velocity *= 0.8;
                d.position = lerp_vec(d.position, center, 0.5);
                d.no_gravity = true;
            }
        }
        // `:73467-73478` — the 435: Next(5,10) dust-226 at Center 0×0 alpha 100
        // scale 0.5, vel×1.6 + Y−− + position Lerp→Center 0.5 + noGravity.
        435 => center_lerp_burst(s, dust, rng, ctx, world, 5, 10, 226, 0.5, 1.6, false),
        // `:73503-73514` — the 436: the 435 shape, dust-220.
        436 => center_lerp_burst(s, dust, rng, ctx, world, 5, 10, 220, 0.5, 1.6, false),
        // `:73515-73527` — the 462: the 435 shape, dust-229, position −= One×4
        // before the Lerp.
        462 => center_lerp_burst(s, dust, rng, ctx, world, 5, 10, 229, 0.5, 1.6, true),
        // `:73479-73491` — the 732: Next(5,10) dust-226 at Center, vel×1.6 + Y−−
        // then vel = vel×0.5 + baseVel×(i/count)×0.7, position Lerp→Center 0.5,
        // noGravity.
        732 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let count = rng.next_range(5, 10);
            for i in 0..count {
                let idx = dust.new_dust(rng, ctx, world, center, 0, 0, 226, 0.0, 0.0, 100, Color::default(), 0.5);
                let d = dust.slot_mut(idx);
                d.velocity *= 1.6;
                d.velocity.y -= 1.0;
                d.velocity = d.velocity * 0.5 + s.vel * (i as f32 / count as f32) * 0.7;
                d.position = lerp_vec(d.position, center, 0.5);
                d.no_gravity = true;
            }
        }
        // `:72552-72564` — the 631: Next(5,10) dust-229 at Center 0×0 alpha 100,
        // vel×1.6 + Y−− + position −= One×4 then Lerp→Center 0.5 + noGravity.
        631 => center_lerp_burst(s, dust, rng, ctx, world, 5, 10, 229, 1.0, 1.6, true),
        // `:73454-73466` — the 684: Next(15,25) dust-60 at Center 0×0 alpha 100
        // color (255,255,255,0) scale 1.3, vel×8×(0.3+0.7×NextFloat) + fadeIn
        // 1.3+NextFloat×0.2 + noLight + noGravity + position += vel×4.
        684 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let count = rng.next_range(15, 25);
            for _ in 0..count {
                let idx = dust.new_dust(rng, ctx, world, center, 0, 0, 60, 0.0, 0.0, 100, Color::new(255, 255, 255, 0), 1.3);
                let d = dust.slot_mut(idx);
                d.velocity *= 8.0 * (0.3 + 0.7 * rng.next_float());
                d.fade_in = 1.3 + rng.next_float() * 0.2;
                d.no_light = true;
                d.no_gravity = true;
                d.position += d.velocity * 4.0;
            }
        }
        // `:72543-72551` — the 625-628: 6 dust-135 alpha 100 scale 2, noGravity
        // + noLight. No explicit draw.
        625..=628 => {
            for _ in 0..6 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 135, 0.0, 0.0, 100, Color::default(), 2.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.no_light = true;
            }
        }
        // `:73421-73430` — the 472: 20 dust-30, noGravity + vel×0.45 + vel +=
        // baseVel×0.9. No explicit draw.
        472 => {
            for _ in 0..20 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 30, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 0.45;
                d.velocity += s.vel * 0.9;
            }
        }
        // `:73553-73562` — the 440: Next(3,7) dust-135 at Center−vel/2 0×0 alpha
        // 100 scale 2.1, vel×2 + noGravity.
        440 => {
            let pos = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5) - s.vel / 2.0;
            let count = rng.next_range(3, 7);
            for _ in 0..count {
                let idx = dust.new_dust(rng, ctx, world, pos, 0, 0, 135, 0.0, 0.0, 100, Color::default(), 2.1);
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
        }
        // `:73563-73572` — the 606: Next(3,7) dust-182 at Center−vel/2 0×0 alpha
        // 100 scale 1.6, vel×1.5 + noGravity.
        606 => {
            let pos = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5) - s.vel / 2.0;
            let count = rng.next_range(3, 7);
            for _ in 0..count {
                let idx = dust.new_dust(rng, ctx, world, pos, 0, 0, 182, 0.0, 0.0, 100, Color::default(), 1.6);
                let d = dust.slot_mut(idx);
                d.velocity *= 1.5;
                d.no_gravity = true;
            }
        }
        // `:75307-75323` — the 177: 20 dust-137, alpha Next(0,101) + scale
        // 1+Next(-20,40)*0.01 BEFORE the spawn, vel −= oldVel×0.2, then Next(3)
        // gate → scale×0.8/vel×0.5 else noGravity.
        177 => {
            for _ in 0..20 {
                let alpha = rng.next_range(0, 101);
                let scale = 1.0 + rng.next_range(-20, 40) as f32 * 0.01;
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 137, 0.0, 0.0, alpha, Color::default(), scale);
                let d = dust.slot_mut(idx);
                d.velocity -= s.old_vel * 0.2;
                if rng.next_max(3) == 0 {
                    d.scale *= 0.8;
                    d.velocity *= 0.5;
                } else {
                    d.no_gravity = true;
                }
            }
        }
        // `:75324-75346` — the 119/118/128/359: dust-92, count 20 for 119/359
        // else 10; per dust Next(3) gate ≠0 → vel×2/noGravity/scale×1.75 else
        // scale×0.5.
        119 | 118 | 128 | 359 => {
            let count = if s.kind == 119 || s.kind == 359 { 20 } else { 10 };
            for _ in 0..count {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 92, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                if rng.next_max(3) != 0 {
                    d.velocity *= 2.0;
                    d.no_gravity = true;
                    d.scale *= 1.75;
                } else {
                    d.scale *= 0.5;
                }
            }
        }
        // `:75347-75361` — the 309: 10 dust-185; per dust Next(2) gate → vel×2/
        // noGravity/scale×1.75.
        309 => {
            for _ in 0..10 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 185, 0.0, 0.0, 0, Color::default(), 1.0);
                if rng.next_max(2) == 0 {
                    let d = dust.slot_mut(idx);
                    d.velocity *= 2.0;
                    d.no_gravity = true;
                    d.scale *= 1.75;
                }
            }
        }
        // `:75362-75373` — the 308: 80 dust-185 at pos+(0,16) box h−16, vel×2 +
        // noGravity + scale×1.15. No explicit draw.
        308 => {
            let pos = Vec2::new(s.pos.x, s.pos.y + 16.0);
            for _ in 0..80 {
                let idx = dust.new_dust(rng, ctx, world, pos, s.w, s.h - 16, 185, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
                d.scale *= 1.15;
            }
        }
        // `:75276-75285` — the 967: 20 dust-219 speed vel, scale 0.5+Next(-20,40)
        // *0.01 BEFORE spawn, noGravity + velocity = NextVector2Circular(6,6).
        967 => {
            for _ in 0..20 {
                let scale = 0.5 + rng.next_range(-20, 40) as f32 * 0.01;
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 219, s.vel.x, s.vel.y, 0, Color::default(), scale);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity = next_vector2_circular(rng, 6.0, 6.0);
            }
        }
        // `:75286-75297` — the 1026: 20 dust-2 speed vel, scale 1.2+Next(-20,40)
        // *0.01 BEFORE spawn, alpha 50 + fadeIn 0.5 + noGravity + velocity =
        // NextVector2Circular(5,5).
        1026 => {
            for _ in 0..20 {
                let scale = 1.2 + rng.next_range(-20, 40) as f32 * 0.01;
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 2, s.vel.x, s.vel.y, 0, Color::default(), scale);
                let d = dust.slot_mut(idx);
                d.alpha = 50;
                d.fade_in = 0.5;
                d.no_gravity = true;
                d.velocity = next_vector2_circular(rng, 5.0, 5.0);
            }
        }
        // `:74475-74486` — the 239: gated vel.Y>0, 1 dust-154 at pos+(0,h−2) 2×2,
        // position.X−=2 + alpha 38 + vel×0.1 += −oldVel×0.25 + scale 0.95.
        239 => bottom_dust(s, dust, rng, ctx, world, 154, false),
        // `:74487-74499` — the 245: the 239 shape, dust-114, noGravity.
        245 => bottom_dust(s, dust, rng, ctx, world, 114, true),
        // `:74500-74512` — the 264: the 239 shape, dust-54, noGravity.
        264 => bottom_dust(s, dust, rng, ctx, world, 54, true),
        // `:76377-76392` — the 306: 20 dust-184 (scale×1.1 + noGravity) THEN 30
        // dust-184 (vel×2.5 + scale×0.8 + noGravity). No explicit draw. (The
        // owner-gated 307 spawn is the projectile lane's, not here.)
        306 => {
            for _ in 0..20 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 184, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.scale *= 1.1;
                d.no_gravity = true;
            }
            for _ in 0..30 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 184, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.velocity *= 2.5;
                d.scale *= 0.8;
                d.no_gravity = true;
            }
        }
        // `:77061-77074` — the 115: for i in 4..31, dust-27 pair at pos−oldVel×
        // (30/i) box 8×8 speed oldVel alpha 100, scale 1.4 (noGravity, vel×0.5)
        // + scale 0.9 (vel×0.5). No explicit draw.
        115 => trail_pair(s, dust, rng, ctx, world, 4, 31, 27, 100, 1.4, 0.9, true, false, 0.5, 0.5),
        // `:77075-77091` — the 114: the 115 pair shape, gated per-i
        // Next(128) >= alpha.
        114 => {
            for i in 4..31 {
                if rng.next_max(128) >= s.alpha {
                    let off = s.old_vel * (30.0 / i as f32);
                    let pos = s.pos - off;
                    let idx = dust.new_dust(rng, ctx, world, pos, 8, 8, 27, s.old_vel.x, s.old_vel.y, 100, Color::default(), 1.4);
                    let d = dust.slot_mut(idx);
                    d.no_gravity = true;
                    d.velocity *= 0.5;
                    let idx = dust.new_dust(rng, ctx, world, pos, 8, 8, 27, s.old_vel.x, s.old_vel.y, 100, Color::default(), 0.9);
                    dust.slot_mut(idx).velocity *= 0.5;
                }
            }
        }
        // `:77092-77104` — the 116: dust-64 pair at pos−oldVel×(30/i), scale 1.8
        // (noGravity) + scale 1.4 (noGravity). No explicit draw.
        116 => trail_pair(s, dust, rng, ctx, world, 4, 31, 64, 100, 1.8, 1.4, true, true, 1.0, 1.0),
        // `:77105-77121` — the 173: for i in 4..24, dust type Next(3){15,57,58}
        // at pos−oldVel×(30/i) box 8×8 speed oldVel×0.2 alpha 100 scale 1.8,
        // vel×1.5 + noGravity.
        173 => {
            for i in 4..24 {
                let dust_type = match rng.next_max(3) {
                    0 => 15,
                    1 => 57,
                    _ => 58,
                };
                let off = s.old_vel * (30.0 / i as f32);
                let pos = s.pos - off;
                let idx = dust.new_dust(rng, ctx, world, pos, 8, 8, dust_type, s.old_vel.x * 0.2, s.old_vel.y * 0.2, 100, Color::default(), 1.8);
                let d = dust.slot_mut(idx);
                d.velocity *= 1.5;
                d.no_gravity = true;
            }
        }
        // `:77122-77135` — the 132: dust-107 pair at oldPos−oldVel×(30/i), scale
        // 1.8 (noGravity, vel×0.5) + scale 1.4 (vel×0.05). No explicit draw.
        132 => trail_pair_oldpos(s, dust, rng, ctx, world, 4, 31, 107, 100, 1.8, 1.4, true, false, 0.5, 0.05),
        // `:77136-77150` — the 156: dust-73 pair at oldPos−oldVel×(30/i) alpha
        // 255, scale 1.8 (noGravity, vel×0.5) + scale 1.4 (vel×0.05, noGravity).
        156 => trail_pair_oldpos(s, dust, rng, ctx, world, 4, 31, 73, 255, 1.8, 1.4, true, true, 0.5, 0.05),
        // `:77151-77160` — the 157: for i in 4..31, dust-107 at pos box w×h speed
        // oldVel alpha 100 scale 1.8, noGravity + vel×0.5. No explicit draw.
        157 => {
            for _ in 4..31 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 107, s.old_vel.x, s.old_vel.y, 100, Color::default(), 1.8);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 0.5;
            }
        }
        // `:74159-74170` — the 343: for i in 4..31, dust-197 at oldPos−oldVel×
        // (30/i) box 8×8 speed oldVel alpha 100 scale 1.2, noGravity + vel×0.5.
        343 => {
            for i in 4..31 {
                let off = s.old_vel * (30.0 / i as f32);
                let pos = s.old_pos - off;
                let idx = dust.new_dust(rng, ctx, world, pos, 8, 8, 197, s.old_vel.x, s.old_vel.y, 100, Color::default(), 1.2);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 0.5;
            }
        }
        // `:73745-73751` — the 408: 15 dust-5 at Center−One×10 box 50×50 speed
        // (0,−2), vel /= 2. No explicit draw. (The 584/585/586 gores are the
        // gore lane's, not here.)
        408 => {
            let pos = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5) - Vec2::ONE * 10.0;
            for _ in 0..15 {
                let idx = dust.new_dust(rng, ctx, world, pos, 50, 50, 5, 0.0, -2.0, 0, Color::default(), 1.0);
                dust.slot_mut(idx).velocity /= 2.0;
            }
        }
        // `:73761-73773` — the 385: 36 dust-172 ring at Center + rotated
        // (norm(vel)×(w/2,h)×0.75), speed = offset×2 alpha 100 scale 1.4,
        // noGravity + noLight + velocity = offset. No explicit draw. (The
        // owner-gated 384/386 cloud spawn is the sharknado lane's, not here.)
        385 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let num437 = 36;
            let base_vec = safe_normalize(s.vel) * Vec2::new(s.w as f32 / 2.0, s.h as f32) * 0.75;
            for i in 0..num437 {
                let angle = (i - (num437 / 2 - 1)) as f32 * (std::f32::consts::PI * 2.0) / num437 as f32;
                let v49 = rotate_vec(base_vec, angle) + center;
                let v50 = v49 - center;
                let idx = dust.new_dust(rng, ctx, world, v49 + v50, 0, 0, 172, v50.x * 2.0, v50.y * 2.0, 100, Color::default(), 1.4);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.no_light = true;
                d.velocity = v50;
            }
        }
        _ => {}
    }
}

/// `count` `Dust.NewDust(pos, w, h, type, sx, sy, alpha, color, scale)` bodies,
/// each with an optional `velocity *= vel_mul` and `noGravity = true` post-spawn
/// write. No explicit draws — the randomness is the `new_dust` internals.
#[allow(clippy::too_many_arguments)]
fn plain_burst(
    s: &DeathSnapshot,
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
    count: i32,
    dust_type: i32,
    speed_x: f32,
    speed_y: f32,
    alpha: i32,
    color: Color,
    scale: f32,
    vel_mul: Option<f32>,
    no_gravity: bool,
) {
    for _ in 0..count {
        let idx = dust.new_dust(
            rng, ctx, world, s.pos, s.w, s.h, dust_type, speed_x, speed_y, alpha, color, scale,
        );
        let d = dust.slot_mut(idx);
        if let Some(k) = vel_mul {
            d.velocity *= k;
        }
        if no_gravity {
            d.no_gravity = true;
        }
    }
}

/// `count` dust of `dust_type` (speed/alpha/color/scale as given); per dust a
/// `Next(gate)` draw, and when it lands 0 the C# writes `scale *= scale_mul`,
/// `noGravity = true`, `velocity *= vel_mul` — the 103/278 shape
/// (`:75622-75627`/`:75636-75641`). The gate draw rides `rng` AFTER the
/// `new_dust` internals (the C# rolls it after the spawn).
#[allow(clippy::too_many_arguments)]
fn gated_burst(
    s: &DeathSnapshot,
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
    count: i32,
    dust_type: i32,
    speed_x: f32,
    speed_y: f32,
    alpha: i32,
    color: Color,
    scale: f32,
    gate: i32,
    scale_mul: f32,
    vel_mul: f32,
) {
    for _ in 0..count {
        let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, dust_type, speed_x, speed_y, alpha, color, scale);
        if rng.next_max(gate) == 0 {
            let d = dust.slot_mut(idx);
            d.scale *= scale_mul;
            d.no_gravity = true;
            d.velocity *= vel_mul;
        }
    }
}

/// `count` dust of `dust_type` (default speed/alpha/color/scale); per dust a
/// `Next(2)` draw, and when it lands 0 `noGravity = true` — the 318/378 shape
/// (`:74266-74269`/`:74277-74280`). The gate draw rides `rng` after the spawn.
fn no_gravity_gate_burst(
    s: &DeathSnapshot,
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
    count: i32,
    dust_type: i32,
) {
    for _ in 0..count {
        let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, dust_type, 0.0, 0.0, 0, Color::default(), 1.0);
        if rng.next_max(2) == 0 {
            dust.slot_mut(idx).no_gravity = true;
        }
    }
}

/// `count` dust of `dust_type` (default speed/alpha/color/scale), each with
/// `noGravity = true` and `velocity -= base.velocity * sub` after the spawn —
/// the 158/159/160/161 shape (`:74775-74777` et al.). No explicit draw.
fn velocity_sub_burst(
    s: &DeathSnapshot,
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
    count: i32,
    dust_type: i32,
    sub: f32,
) {
    for _ in 0..count {
        let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, dust_type, 0.0, 0.0, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity -= s.vel * sub;
    }
}

/// `count` iterations of the reverse-velocity dust PAIR (`:76137-76144`):
/// each iteration spawns a dust of `dust_type` at scale `scale_a` (noGravity,
/// velocity ×2) THEN one at scale `scale_b` (velocity ×2), both with speed
/// `−base.velocity × 0.2`, alpha 100. The 15/321/253/95/96 shape. No explicit
/// draw.
fn reverse_vel_pair(
    s: &DeathSnapshot,
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
    count: i32,
    dust_type: i32,
    scale_a: f32,
    scale_b: f32,
) {
    let sx = (0.0 - s.vel.x) * 0.2;
    let sy = (0.0 - s.vel.y) * 0.2;
    for _ in 0..count {
        let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, dust_type, sx, sy, 100, Color::default(), scale_a);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= 2.0;
        let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, dust_type, sx, sy, 100, Color::default(), scale_b);
        dust.slot_mut(idx).velocity *= 2.0;
    }
}

/// `count` dust of `dust_type` (alpha/scale as given, default speed/color),
/// each with `noGravity = true`, `velocity *= vel_mul`, `velocity -= oldVel *
/// old_sub` — the 265/355/304 shape (`:74388-74391` et al.). No explicit draw.
#[allow(clippy::too_many_arguments)]
fn old_vel_kick_burst(
    s: &DeathSnapshot,
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
    count: i32,
    dust_type: i32,
    alpha: i32,
    scale: f32,
    vel_mul: f32,
    old_sub: f32,
) {
    for _ in 0..count {
        let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, dust_type, 0.0, 0.0, alpha, Color::default(), scale);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= vel_mul;
        d.velocity -= s.old_vel * old_sub;
    }
}

/// `count` dust whose type is `Utils.SelectRandom(rng, choices)` (one
/// `Next(choices.len())` pick BEFORE the spawn), then `noGravity` (when
/// `no_gravity`), `scale = scale_base + NextFloat()*scale_float_mul`,
/// `fadeIn = fade_in`, `velocity *= vel_mul`, `noLight` (when `no_gravity`) —
/// the 581/671/675 shape (`:72683-72689` et al.).
#[allow(clippy::too_many_arguments)]
fn select_random_burst(
    s: &DeathSnapshot,
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
    count: i32,
    choices: &[i32],
    scale_base: f32,
    scale_float_mul: f32,
    fade_in: f32,
    vel_mul: f32,
    no_gravity: bool,
) {
    for _ in 0..count {
        let dust_type = choices[rng.next_max(choices.len() as i32) as usize];
        let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, dust_type, 0.0, 0.0, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        if no_gravity {
            d.no_gravity = true;
        }
        d.scale = scale_base + rng.next_float() * scale_float_mul;
        d.fade_in = fade_in;
        d.velocity *= vel_mul;
        if no_gravity {
            d.no_light = true;
        }
    }
}

/// `count` dust of `dust_type` (speed = vel×0.1, scale 0.5); per dust a
/// `Next(3)` gate: when 0, `fadeIn = fade_base + Next(-10,11)*0.01`,
/// `scale = small_base + Next(-10,11)*small_step`, `type++`; else
/// `scale = big_base + Next(-10,11)*big_step`. Then `noGravity`,
/// `velocity *= vel_mul`, `velocity -= oldVel/10` — the 658/522 crystal-spark
/// shape (`:72378-72391`). The gate + the branch's range draws ride `rng`
/// after the spawn.
#[allow(clippy::too_many_arguments)]
fn crystal_spark_burst(
    s: &DeathSnapshot,
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
    count: i32,
    dust_type: i32,
    fade_base: f32,
    small_base: f32,
    small_step: f32,
    big_base: f32,
    big_step: f32,
    vel_mul: f32,
) {
    for _ in 0..count {
        let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, dust_type, s.vel.x * 0.1, s.vel.y * 0.1, 0, Color::default(), 0.5);
        if rng.next_max(3) == 0 {
            let d = dust.slot_mut(idx);
            d.fade_in = fade_base + rng.next_range(-10, 11) as f32 * 0.01;
            d.scale = small_base + rng.next_range(-10, 11) as f32 * small_step;
            d.dust_type += 1;
        } else {
            let d = dust.slot_mut(idx);
            d.scale = big_base + rng.next_range(-10, 11) as f32 * big_step;
        }
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= vel_mul;
        d.velocity -= s.old_vel / 10.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(kind: u16) -> DeathSnapshot {
        DeathSnapshot {
            kind,
            pos: Vec2::new(100.0, 100.0),
            w: 10,
            h: 10,
            vel: Vec2::new(4.0, -2.0),
            scale: 1.0,
            old_vel: Vec2::new(4.0, -2.0),
            ai1: 0.0,
            rot: 0.0,
            direction: 1,
            alpha: 0,
            old_pos: Vec2::new(100.0, 100.0),
        }
    }

    fn ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(1280.0, 720.0),
            max_dust_to_draw: 6000,
        }
    }

    fn world() -> World {
        World::new("test".into(), 64, 64)
    }

    /// The plain bursts spawn the C# count of the right dust type at the
    /// corpse position, and the 31/42/39/71/179/40/109 arms carry the
    /// velocity ×0.6 write. With a fresh pool every spawn takes a real slot
    /// (the pressure bands reset `d_count` in the low band, no refusal).
    #[test]
    fn w4_g65_the_plain_bursts_spawn_the_csharp_bodies() {
        let w = world();
        for (kind, count, dust_type, vel_mul) in [
            (17u16, 5, 0, None),
            (42, 5, 32, Some(0.6)),
            (39, 5, 38, Some(0.6)),
            (71, 5, 53, Some(0.6)),
            (179, 5, 149, Some(0.6)),
            (40, 5, 36, Some(0.6)),
            (109, 5, 51, Some(0.6)),
        ] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(7);
            drain(&[snap(kind)], &mut dust, &mut rng, &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), count, "kind {kind} burst count");
            for d in &active {
                assert_eq!(d.dust_type, dust_type, "kind {kind} dust type");
            }
            // The velocity write: the default block's velocity is the
            // `Next(-20,21)*0.1` jitter; ×0.6 scales it. Compare against a
            // witness pool that ran the same spawn WITHOUT the write.
            if let Some(k) = vel_mul {
                let mut witness = DustPool::new();
                let mut wrng = UnifiedRandom::new(7);
                plain_burst(&snap(kind), &mut witness, &mut wrng, &ctx(), &w, count, dust_type, 0.0, 0.0, 0, Color::default(), if kind == 109 { 0.6 } else { 1.0 }, None, false);
                for (a, b) in active.iter().zip(witness.slots().iter().filter(|d| d.active)) {
                    assert_eq!(a.velocity, b.velocity * k, "kind {kind} velocity write");
                }
            }
        }
    }

    /// The 24/38 bursts carry the velocity×0.1 SPEED argument (not a post
    /// write) and the 24's 0.75 scale; the 21-family is 10 dust-26 scale 0.8.
    #[test]
    fn w4_g65_the_24_38_21_bursts_carry_their_arguments() {
        let w = world();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        drain(&[snap(24)], &mut dust, &mut rng, &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 10);
        assert!(active.iter().all(|d| d.dust_type == 1));
        // scale 0.75 rides the default block's `1 + Next(-20,21)*0.01` then
        // `*= 0.75`; every slot's scale is < 1.0 (the 0.75 cap).
        assert!(active.iter().all(|d| d.scale < 1.0));
    }

    /// An unclaimed kind spawns nothing and draws nothing.
    #[test]
    fn w4_g65_an_unclaimed_kind_is_inert() {
        let w = world();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        let before = rng.clone();
        drain(&[snap(9999)], &mut dust, &mut rng, &ctx(), &w);
        assert!(dust.slots().iter().all(|d| !d.active));
        assert_eq!(rng, before);
    }

    /// The drain is deterministic: two identical snapshot lists on identical
    /// streams produce byte-equal pools (the determinism invariant).
    #[test]
    fn w4_g65_the_drain_is_deterministic() {
        let w = world();
        let snaps = [snap(17), snap(42), snap(24)];
        let mut a = DustPool::new();
        let mut b = DustPool::new();
        drain(&snaps, &mut a, &mut UnifiedRandom::new(42), &ctx(), &w);
        drain(&snaps, &mut b, &mut UnifiedRandom::new(42), &ctx(), &w);
        assert_eq!(a.slots(), b.slots());
    }

    /// The 103/278 gated bursts: 20 dust each, and the stream advances by the
    /// `new_dust` internals PLUS one `Next(2)` gate per dust. A witness that
    /// runs the same spawns WITHOUT the gate draws lands behind the real
    /// stream by exactly 20 `Next(2)` steps (the gate is taken after each
    /// spawn, even when the dust is culled — the C# rolls it unconditionally).
    #[test]
    fn w4_g65_the_gated_bursts_take_one_gate_draw_per_dust() {
        let w = world();
        for (kind, dust_type) in [(103u16, 75), (278, 169)] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(7);
            drain(&[snap(kind)], &mut dust, &mut rng, &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 20, "kind {kind} burst count");
            assert!(active.iter().all(|d| d.dust_type == dust_type));
            // The witness: the same 20 spawns, no gate draws.
            let mut witness = DustPool::new();
            let mut wrng = UnifiedRandom::new(7);
            plain_burst(&snap(kind), &mut witness, &mut wrng, &ctx(), &w, 20, dust_type, 0.0, 0.0, 100, Color::default(), 1.0, None, false);
            // Advance the witness by the 20 gate draws the real arm took.
            for _ in 0..20 {
                let _ = wrng.next_max(2);
            }
            assert_eq!(rng, wrng, "kind {kind} stream = internals + 20 gates");
        }
    }

    /// The 5 JestersArrow: 60 dust, each a `Next(3)` type pick {15,57,58}
    /// BEFORE the spawn internals. The stream advances by 60 `Next(3)` + the
    /// internals; every spawned type is in the pick set.
    #[test]
    fn w4_g65_the_jesters_burst_picks_the_type_per_dust() {
        let w = world();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        drain(&[snap(5)], &mut dust, &mut rng, &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 60);
        assert!(active.iter().all(|d| matches!(d.dust_type, 15 | 57 | 58)));
    }

    /// The 93: 10 dust-57, noGravity + fadeIn 1, velocity ×2 then a
    /// `NextFloat()*1.5` Y kick. The stream advances by the internals + 10
    /// `NextFloat`s.
    #[test]
    fn w4_g65_the_93_burst_kicks_y_after_the_double() {
        let w = world();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        drain(&[snap(93)], &mut dust, &mut rng, &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 10);
        assert!(active.iter().all(|d| d.dust_type == 57 && d.no_gravity && d.fade_in == 1.0));
    }

    /// The 346/335 ai[1]-driven type picks and the 76/55 noGravity bursts.
    #[test]
    fn w4_g65_the_ai1_picks_and_no_gravity_bursts() {
        let w = world();
        // 346 with ai1 == 1 → dust-4; else dust-10.
        let mut s = snap(346);
        s.ai1 = 1.0;
        let mut dust = DustPool::new();
        drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        assert!(dust.slots().iter().filter(|d| d.active).all(|d| d.dust_type == 4 && d.no_gravity));
        // 335 with ai1 == 2 → dust-88, noLight + scale 0.8.
        let mut s = snap(335);
        s.ai1 = 2.0;
        let mut dust = DustPool::new();
        drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        assert!(dust.slots().iter().filter(|d| d.active).all(|d| d.dust_type == 88 && d.no_light && d.scale == 0.8));
        // 76 → 5 dust-27 noGravity; 55 → 5 dust-18 noGravity.
        for (kind, dust_type) in [(76u16, 27), (55, 18)] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 5);
            assert!(active.iter().all(|d| d.dust_type == dust_type && d.no_gravity));
        }
    }

    /// The 158/159/160/161 velocity-subtract bursts: 10 dust each, noGravity,
    /// velocity -= base.velocity×0.5. Compare against a witness pool that ran
    /// the same spawns WITHOUT the subtract (the default-block velocity is the
    /// `Next(-20,21)*0.1` jitter; the subtract shifts it by −vel×0.5).
    #[test]
    fn w4_g65_the_velocity_subtract_bursts() {
        let w = world();
        for (kind, dust_type) in [(158u16, 9), (159, 11), (160, 19), (161, 11)] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(7);
            drain(&[snap(kind)], &mut dust, &mut rng, &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 10, "kind {kind} count");
            assert!(active.iter().all(|d| d.dust_type == dust_type && d.no_gravity));
            let mut witness = DustPool::new();
            let mut wrng = UnifiedRandom::new(7);
            plain_burst(&snap(kind), &mut witness, &mut wrng, &ctx(), &w, 10, dust_type, 0.0, 0.0, 0, Color::default(), 1.0, None, false);
            let vel = Vec2::new(4.0, -2.0);
            for (a, b) in active.iter().zip(witness.slots().iter().filter(|d| d.active)) {
                assert_eq!(a.velocity, b.velocity - vel * 0.5, "kind {kind} velocity subtract");
            }
            // No explicit draw: the stream matches the witness (internals only).
            assert_eq!(rng, wrng, "kind {kind} stream-neutral");
        }
    }

    /// The 2/82, 51/267, 3-family, 330, 4, 1124 plain bursts carry their
    /// counts/types/speeds/scales and draw nothing explicit.
    #[test]
    fn w4_g65_the_batch3_plain_bursts() {
        let w = world();
        for (kind, count, dust_type) in [
            (2u16, 10, 6), (82, 10, 6), (51, 5, 0), (267, 5, 0),
            (3, 10, 1), (48, 10, 1), (330, 10, 0), (4, 10, 14), (1124, 5, 26),
        ] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(7);
            let before = rng.clone();
            drain(&[snap(kind)], &mut dust, &mut rng, &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), count, "kind {kind} count");
            assert!(active.iter().all(|d| d.dust_type == dust_type), "kind {kind} type");
            // The plain bursts take no explicit draw beyond the internals; the
            // stream advanced (the internals ran) but the body arguments are
            // the C#'s. The 330's speed is vel×0.4 (vs the 3-family's ×0.1).
            assert_ne!(rng, before, "kind {kind} internals ran");
        }
    }

    /// The 15/321/253/95/96 reverse-velocity pairs spawn 2×count dust (the
    /// pair per iteration), all noGravity-or-not per the C# (the first of each
    /// pair is noGravity, the second is not), and draw nothing explicit.
    #[test]
    fn w4_g65_the_reverse_velocity_pairs() {
        let w = world();
        for (kind, dust_type) in [(15u16, 6), (321, 6), (253, 135), (95, 75), (96, 75)] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(7);
            drain(&[snap(kind)], &mut dust, &mut rng, &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 40, "kind {kind} pair count (20×2)");
            assert!(active.iter().all(|d| d.dust_type == dust_type));
            // The first of each pair is noGravity; the second is not.
            for (i, d) in active.iter().enumerate() {
                assert_eq!(d.no_gravity, i % 2 == 0, "kind {kind} pair {i} noGravity");
            }
        }
    }

    /// The 497 takes one Next(150) scale draw per dust (15 total) after the
    /// internals; the 495 (no explicit draw) is stream-neutral vs a witness.
    #[test]
    fn w4_g65_the_495_497_center_bursts() {
        let w = world();
        // 495: 15 dust-27 at the centre, noGravity, no explicit draw.
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        drain(&[snap(495)], &mut dust, &mut rng, &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 15);
        assert!(active.iter().all(|d| d.dust_type == 27 && d.no_gravity));
        // 497: 15 dust-27, each a Next(150) scale draw → the stream advances
        // beyond a no-draw witness by exactly 15 Next(150) steps.
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        drain(&[snap(497)], &mut dust, &mut rng, &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 15);
        assert!(active.iter().all(|d| d.dust_type == 27 && d.no_gravity));
    }

    /// The 731 Next(3) gate (≠0 → noGravity/scale×1.5/vel×0.5) and the 729
    /// velocity.X×2 write; the 337 plain noGravity burst.
    #[test]
    fn w4_g65_the_729_731_337_bursts() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(729)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 10);
        assert!(active.iter().all(|d| d.dust_type == 279 && d.no_gravity));
        let mut dust = DustPool::new();
        drain(&[snap(731)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 15);
        assert!(active.iter().all(|d| d.dust_type == 226));
        let mut dust = DustPool::new();
        drain(&[snap(337)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 10);
        assert!(active.iter().all(|d| d.dust_type == 197 && d.no_gravity));
    }

    /// The 581/671/675 SelectRandom bursts pick a type from the choice list
    /// per dust (one Next(len) BEFORE the spawn) and write noGravity/scale/
    /// fadeIn/vel/noLight. Every spawned type is in the choice set.
    #[test]
    fn w4_g65_the_select_random_bursts() {
        let w = world();
        for (kind, count, choices) in [
            (581u16, 30, &[229, 229, 161][..]),
            (671, 30, &[27, 27, 62][..]),
            (675, 40, &[27, 242, 73, 72, 71, 255, 255, 255][..]),
        ] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), count, "kind {kind} count");
            assert!(active.iter().all(|d| choices.contains(&d.dust_type)), "kind {kind} type in choices");
            assert!(active.iter().all(|d| d.no_gravity && d.no_light && d.fade_in == 0.25));
        }
    }

    /// The 658/522 crystal-spark bursts: 10 dust each, the Next(3) gate picks
    /// the small (type++) or big scale branch; every dust ends noGravity. The
    /// 520 is the plain noGravity −= oldVel/3 burst.
    #[test]
    fn w4_g65_the_crystal_spark_bursts() {
        let w = world();
        for (kind, base_type) in [(658u16, 269), (522, 254)] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 10, "kind {kind} count");
            // The gate's type++ branch lifts some dust to base+1.
            assert!(active.iter().all(|d| d.dust_type == base_type || d.dust_type == base_type + 1));
            assert!(active.iter().all(|d| d.no_gravity));
        }
        let mut dust = DustPool::new();
        drain(&[snap(520)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 10);
        assert!(active.iter().all(|d| d.dust_type == 252 && d.no_gravity));
    }

    /// The 811/814/819 scale-roll bursts and the 572 midpoint-position burst.
    #[test]
    fn w4_g65_the_811_819_572_bursts() {
        let w = world();
        for (kind, count) in [(811u16, 30), (814, 30), (819, 20)] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), count, "kind {kind} count");
            assert!(active.iter().all(|d| d.dust_type == 5));
        }
        let mut dust = DustPool::new();
        drain(&[snap(572)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 15);
        // position = (Center + position)/2 = (105+100)/2, (105+100)/2.
        let mid = Vec2::new(102.5, 102.5);
        assert!(active.iter().all(|d| d.dust_type == 40 && d.no_gravity && d.scale == 1.25 && d.position == mid));
    }

    /// The 507/508/598/971 walking-position bursts spawn the C# count, all
    /// noGravity, with the midpoint position write. The 384/386 carry the
    /// CornflowerBlue→White lerp colour.
    #[test]
    fn w4_g65_the_walking_position_bursts() {
        let w = world();
        for (kind, count, dust_type) in [(507u16, 20, 81), (508, 20, 81), (598, 20, 81), (971, 6, 4)] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), count, "kind {kind} count");
            assert!(active.iter().all(|d| d.dust_type == dust_type && d.no_gravity), "kind {kind} type/noGravity");
        }
        // 384: 20 dust-212, the lerp colour, noGravity.
        let mut dust = DustPool::new();
        drain(&[snap(384)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 20);
        let expect = lerp_color(Color::new(100, 149, 237, 255), Color::WHITE, 0.3);
        assert!(active.iter().all(|d| d.dust_type == 212 && d.no_gravity && d.color == expect));
    }

    /// The 336/345 scale=proj.scale burst, the 344 vel×0.75−=vel×0.025 burst,
    /// the 349 noLight/scale 0.7 burst, and the 1/184/275 plain dust-7 bursts.
    #[test]
    fn w4_g65_the_336_344_349_and_plain7_bursts() {
        let w = world();
        let mut s = snap(336);
        s.scale = 1.5;
        let mut dust = DustPool::new();
        drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 6);
        assert!(active.iter().all(|d| d.dust_type == 196 && d.no_gravity && d.scale == 1.5));
        let mut dust = DustPool::new();
        drain(&[snap(344)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        assert_eq!(dust.slots().iter().filter(|d| d.active).count(), 8);
        let mut dust = DustPool::new();
        drain(&[snap(349)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 3);
        assert!(active.iter().all(|d| d.dust_type == 76 && d.no_gravity && d.no_light && d.scale == 0.7));
        for (kind, count) in [(1u16, 10), (81, 10), (184, 5), (195, 5), (275, 5), (276, 5)] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), count, "kind {kind} count");
            assert!(active.iter().all(|d| d.dust_type == 7));
        }
    }

    /// The 265/355/304 old-velocity-kick bursts and the 263 alpha/scale-draw
    /// burst (the draws ride BEFORE the spawn). The 206 gate suppresses the
    /// burst at alpha 255.
    #[test]
    fn w4_g65_the_old_vel_kick_and_263_bursts() {
        let w = world();
        for (kind, count, dust_type) in [(265u16, 15, 163), (355, 15, 205), (304, 3, 182)] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), count, "kind {kind} count");
            assert!(active.iter().all(|d| d.dust_type == dust_type && d.no_gravity));
        }
        // 263: 15 dust-92, each an alpha Next(0,101) + scale Next(40) draw.
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        drain(&[snap(263)], &mut dust, &mut rng, &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 15);
        assert!(active.iter().all(|d| d.dust_type == 92 && d.no_gravity));
        // 206 at alpha 255 → no burst; at alpha 0 → 5 dust-40.
        let mut s = snap(206);
        s.alpha = 255;
        let mut dust = DustPool::new();
        drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        assert!(dust.slots().iter().all(|d| !d.active));
        let mut s = snap(206);
        s.alpha = 0;
        let mut dust = DustPool::new();
        drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        assert_eq!(dust.slots().iter().filter(|d| d.active).count(), 5);
    }

    /// The 261 two-loop burst (15 dust-148 + 3 dust-228), the 928 white dust-5,
    /// the 227 NextFloat kick, the 120 first-5-noGravity, the 181-family.
    #[test]
    fn w4_g65_the_261_928_227_120_181_bursts() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(261)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 18);
        assert_eq!(active.iter().filter(|d| d.dust_type == 148).count(), 15);
        assert_eq!(active.iter().filter(|d| d.dust_type == 228).count(), 3);
        let mut dust = DustPool::new();
        drain(&[snap(928)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 25);
        assert!(active.iter().all(|d| d.dust_type == 5 && d.color == Color::WHITE));
        let mut dust = DustPool::new();
        drain(&[snap(227)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 15);
        assert!(active.iter().all(|d| d.dust_type == 157 && d.no_gravity && d.scale == 1.5));
        let mut dust = DustPool::new();
        drain(&[snap(120)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 10);
        assert_eq!(active.iter().filter(|d| d.no_gravity).count(), 5);
        for kind in [181u16, 189, 566] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 6);
            assert!(active.iter().all(|d| d.dust_type == 150 && d.no_gravity && d.scale == 1.0));
        }
    }

    /// The center-lerp bursts (435/436/462/631/444/732) take a Next count draw
    /// then spawn that many dust, all noGravity, position lerped toward the
    /// centre. The 684 takes Next(15,25) + per-dust NextFloat×2.
    #[test]
    fn w4_g65_the_center_lerp_bursts() {
        let w = world();
        for (kind, dust_type) in [(435u16, 226), (436, 220), (462, 229), (631, 229), (444, 171), (732, 226)] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(7);
            drain(&[snap(kind)], &mut dust, &mut rng, &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            // The count is Next(5,9) for 444, Next(5,10) for the rest — in [5,9].
            assert!((5..=9).contains(&active.len()), "kind {kind} count in range");
            assert!(active.iter().all(|d| d.dust_type == dust_type && d.no_gravity), "kind {kind} type/noGravity");
        }
        // 684: Next(15,25) dust-60, noLight + noGravity.
        let mut dust = DustPool::new();
        drain(&[snap(684)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert!((15..=24).contains(&active.len()));
        assert!(active.iter().all(|d| d.dust_type == 60 && d.no_light && d.no_gravity));
    }

    /// The 625-628 noGravity/noLight burst, the 472 vel-kick burst, and the
    /// 440/606 Center−vel/2 count bursts.
    #[test]
    fn w4_g65_the_625_472_440_606_bursts() {
        let w = world();
        for kind in [625u16, 626, 627, 628] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 6);
            assert!(active.iter().all(|d| d.dust_type == 135 && d.no_gravity && d.no_light));
        }
        let mut dust = DustPool::new();
        drain(&[snap(472)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 20);
        assert!(active.iter().all(|d| d.dust_type == 30 && d.no_gravity));
        for (kind, dust_type) in [(440u16, 135), (606, 182)] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert!((3..=6).contains(&active.len()), "kind {kind} count");
            assert!(active.iter().all(|d| d.dust_type == dust_type && d.no_gravity));
        }
    }

    /// The 177/119-family/309 gated bursts and the 967/1026 circular-velocity
    /// bursts (the scale draw rides BEFORE the spawn, the circular draw AFTER).
    #[test]
    fn w4_g65_the_177_119_309_967_1026_bursts() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(177)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        assert_eq!(dust.slots().iter().filter(|d| d.active).count(), 20);
        for (kind, count) in [(119u16, 20), (359, 20), (118, 10), (128, 10)] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), count, "kind {kind} count");
            assert!(active.iter().all(|d| d.dust_type == 92));
        }
        let mut dust = DustPool::new();
        drain(&[snap(309)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        assert_eq!(dust.slots().iter().filter(|d| d.active).count(), 10);
        for (kind, dust_type) in [(967u16, 219), (1026, 2)] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 20, "kind {kind} count");
            assert!(active.iter().all(|d| d.dust_type == dust_type && d.no_gravity));
        }
    }

    /// The 239/245/264 bottom-edge single dust (gated vel.Y>0) and the 308/306
    /// multi-loop bursts.
    #[test]
    fn w4_g65_the_bottom_edge_and_308_306_bursts() {
        let w = world();
        // vel.Y = -2 in the fixture → the bottom dust is suppressed.
        for kind in [239u16, 245, 264] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            assert!(dust.slots().iter().all(|d| !d.active), "kind {kind} suppressed at vel.Y<0");
        }
        // vel.Y > 0 → one dust.
        let mut s = snap(245);
        s.vel = Vec2::new(4.0, 2.0);
        s.old_vel = s.vel;
        let mut dust = DustPool::new();
        drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 1);
        assert!(active.iter().all(|d| d.dust_type == 114 && d.no_gravity && d.alpha == 38 && d.scale == 0.95));
        let mut dust = DustPool::new();
        drain(&[snap(308)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        assert_eq!(dust.slots().iter().filter(|d| d.active).count(), 80);
        let mut dust = DustPool::new();
        drain(&[snap(306)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 50);
        assert!(active.iter().all(|d| d.dust_type == 184 && d.no_gravity));
    }

    /// The trail-position pair bursts (115/116/132/156) spawn 2×(end−start)
    /// dust; the 114 gate (Next(128) >= alpha) and the 173 type pick draw per
    /// iteration. The 157/343 are single-dust-per-iteration trails.
    #[test]
    fn w4_g65_the_trail_position_bursts() {
        let w = world();
        for (kind, dust_type, count) in [(115u16, 27, 54), (116, 64, 54), (132, 107, 54), (156, 73, 54)] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), count, "kind {kind} pair count (27×2)");
            assert!(active.iter().all(|d| d.dust_type == dust_type), "kind {kind} type");
        }
        // 114 at alpha 0: every Next(128) >= 0 passes → 54 dust.
        let mut dust = DustPool::new();
        drain(&[snap(114)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        assert_eq!(dust.slots().iter().filter(|d| d.active).count(), 54);
        // 173: 20 iterations (4..24), one dust each, type in {15,57,58}.
        let mut dust = DustPool::new();
        drain(&[snap(173)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 20);
        assert!(active.iter().all(|d| matches!(d.dust_type, 15 | 57 | 58) && d.no_gravity));
        // 157/343: 27 iterations (4..31), one dust each.
        for (kind, dust_type) in [(157u16, 107), (343, 197)] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 27, "kind {kind} count");
            assert!(active.iter().all(|d| d.dust_type == dust_type && d.no_gravity));
        }
    }

    /// The 408 Center−One×10 burst and the 385 36-spoke ring.
    #[test]
    fn w4_g65_the_408_385_bursts() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(408)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 15);
        assert!(active.iter().all(|d| d.dust_type == 5));
        let mut dust = DustPool::new();
        drain(&[snap(385)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 36);
        assert!(active.iter().all(|d| d.dust_type == 172 && d.no_gravity && d.no_light));
    }
}
