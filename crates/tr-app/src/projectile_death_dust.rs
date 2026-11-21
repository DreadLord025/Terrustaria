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

/// `Utils.RotatedBy(radians)` (`Utils.cs:1675-1684`, centre = 0): the cos/sin
/// run in DOUBLE of the widened float angle, cast back to f32, then the f32
/// component multiply-add — the firework rings' per-spoke spin (the
/// `projectile_ai_empress::rotated_by` precision order, distinct from the
/// f32-native [`rotate_vec`] the 385 arm keeps).
fn rotated_by(v: Vec2, angle: f32) -> Vec2 {
    let c = (angle as f64).cos() as f32;
    let s = (angle as f64).sin() as f32;
    Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}

/// `Vector2.ToRotation` (`Utils.cs:1655-1658`): `(float)Math.Atan2(Y, X)`.
fn to_rotation(v: Vec2) -> f32 {
    (v.y as f64).atan2(v.x as f64) as f32
}

/// `MathHelper.Lerp(a, b, t)` (scalar): `a + (b - a) * t`, f32 order.
fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// `Utils.Remap(fromValue, fromMin, fromMax, toMin, toMax)` (`Utils.cs:345-
/// 348`, clamped): `Lerp(toMin, toMax, GetLerpValue(fromMin, fromMax,
/// fromValue, clamped))` — the 985 fan's radius ramp.
fn remap_f32(from_value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    lerp_f32(to_min, to_max, crate::projectile::get_lerp_value(from_min, from_max, from_value))
}

/// `MathHelper.SmoothStep(v1, v2, t)` — the zero-tangent Hermite
/// `(2v1-2v2)t³ + (3v2-3v1)t² + v1` with the clamped amount and the
/// near-zero/one guards (the `projectile_ai_empress::smooth_step` form).
fn smooth_step_f32(v1: f32, v2: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let t3 = t * t * t;
    let t2 = t * t;
    if t.abs() <= f32::from_bits(1) {
        return v1;
    }
    if t == 1.0 {
        return v2;
    }
    (2.0 * v1 - 2.0 * v2) * t3 + (3.0 * v2 - 3.0 * v1) * t2 + v1
}

/// `Vector2.SmoothStep` — the per-component [`smooth_step_f32`] (the 418
/// firework's spoke blend).
fn smooth_step_vec(a: Vec2, b: Vec2, t: f32) -> Vec2 {
    Vec2::new(smooth_step_f32(a.x, b.x, t), smooth_step_f32(a.y, b.y, t))
}

/// The firework rings' shared velocity kick (`:76489-76500` et al.): read the
/// dust's OWN post-spawn velocity, guard an exactly-zero vector to `(1, 0)`,
/// rescale it to `speed` (the `speed / sqrt(x²+y²)` factor), apply the per-axis
/// `xmul`/`ymul` (left-associated f32, the C#'s `x * factor * 1.25f`), then
/// `velocity *= damp` and add the rescaled vector. The 167/168/169-tail/170/
/// 415-tail/418-tail shape.
fn ring_kick(d: &mut tr_core::particles::Dust, speed: f32, damp: f32, xmul: f32, ymul: f32) {
    let mut x = d.velocity.x;
    let mut y = d.velocity.y;
    if x == 0.0 && y == 0.0 {
        x = 1.0;
    }
    let f = speed / (x * x + y * y).sqrt();
    let sx = x * f * xmul;
    let sy = y * f * ymul;
    d.velocity *= damp;
    d.velocity.x += sx;
    d.velocity.y += sy;
}

/// `Utils.NextVector2Square(min, max)` (`Utils.cs:1251-1254`): TWO
/// `NextDouble` draws, `(max-min)*NextDouble()+min` per axis — the
/// queen-slime/sparkle rings' position jitter.
fn next_vector2_square(rng: &mut UnifiedRandom, min: f32, max: f32) -> Vec2 {
    let x = (max - min) * rng.next_double() as f32 + min;
    let y = (max - min) * rng.next_double() as f32 + min;
    Vec2::new(x, y)
}

/// `Utils.RotatedByRandom(maxRadians)` (`Utils.cs:1697-1700`): TWO
/// `NextDouble` draws, `RotatedBy(NextDouble()*max − NextDouble()*max)` —
/// the 659 spirit-flame ring's position scatter. Returns the rotated vector
/// (the double-angle [`rotated_by`] precision).
fn rotated_by_random(rng: &mut UnifiedRandom, v: Vec2, max_radians: f32) -> Vec2 {
    let a = rng.next_double() * max_radians as f64 - rng.next_double() * max_radians as f64;
    let c = a.cos() as f32;
    let s = a.sin() as f32;
    Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}

/// `NPC.AI_121_QueenSlime_GetDustColor` (`NPC.cs:45121-45124`): TWO
/// `NextFloat` draws — `Lerp((0,160,255), Lerp((200,200,200),(255,80,255),
/// NextFloat), NextFloat)`. The inner lerp's float draws FIRST (the C#
/// argument evaluation order: the inner `Color.Lerp` is `value2`).
fn queen_slime_dust_color(rng: &mut UnifiedRandom) -> Color {
    let inner_t = rng.next_float();
    let outer_t = rng.next_float();
    let inner = lerp_color(Color::new(200, 200, 200, 255), Color::new(255, 80, 255, 255), inner_t);
    lerp_color(Color::new(0, 160, 255, 255), inner, outer_t)
}

/// `Kill()`'s 664/666/668/706 DD2 explosion dust (`:71844-71900`). The
/// caller computes the resized box first: `position = Center` with the OLD
/// box, then width = height = 16×num129, then `Center = position` — so the
/// new top-left = oldCentre − (16×num129)/2 and the centre stays the old
/// centre (`Entity.cs:50-59`). The T3 variant doubles the counts/radii and
/// swaps the mid dust to 270 (`:71832-71843`); the flag rides the snapshot
/// from the owner. Five dust loops then the trailing gore loop — the gore
/// DRAWS are consumed here in C# order (the beenade census precedent), the
/// bodies stay the gore lane's.
fn dd2_explosion_dust(
    s: &DeathSnapshot,
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
) {
    let (mut n31a, mut n6, mut nmid, mut n6x, mut n31x, mut ngore) = (4, 20, 20, 10, 20, 4);
    let mut fade = 1.5f32;
    let mut box_size = 6;
    let mut mid_type = 6;
    if s.apprentice_t3 {
        n31a += 4;
        n31x += 10;
        n6 += 20;
        nmid += 30;
        n6x /= 2;
        ngore += 4;
        fade += 0.5;
        box_size += 7;
        mid_type = 270;
    }
    let size = (16 * box_size) as f32;
    let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
    let pos = center - Vec2::splat(size * 0.5);
    let w = 16 * box_size;
    let pi = std::f32::consts::PI;
    let vel_rot = to_rotation(s.vel);
    for _ in 0..n31a {
        let idx = dust.new_dust(rng, ctx, world, pos, w, w, 31, 0.0, 0.0, 100, Color::default(), 1.5);
        let scatter = rotated_by_random(rng, Vec2::Y, pi);
        dust.slot_mut(idx).position = center + scatter * rng.next_double() as f32 * size / 2.0;
    }
    for _ in 0..n6 {
        let idx = dust.new_dust_direct(rng, ctx, world, pos, w, w, 6, 0.0, 0.0, 200, Color::default(), 2.5);
        let d = dust.slot_mut(idx);
        let scatter = rotated_by_random(rng, Vec2::Y, pi);
        d.position = center + scatter * rng.next_double() as f32 * size / 10.0;
        d.velocity *= 16.0;
        if d.velocity.y > -2.0 {
            d.velocity.y *= -0.4;
        }
        d.no_light = true;
        d.no_gravity = true;
    }
    for _ in 0..nmid {
        let idx = dust.new_dust_direct(rng, ctx, world, pos, w, w, mid_type, 0.0, 0.0, 100, Color::default(), 1.5);
        let d = dust.slot_mut(idx);
        let scatter = rotated_by_random(rng, Vec2::Y, pi);
        d.position = center + scatter * rng.next_double() as f32 * size / 2.0;
        d.velocity *= 2.0;
        d.no_gravity = true;
        d.fade_in = fade;
    }
    for _ in 0..n6x {
        let idx = dust.new_dust(rng, ctx, world, pos, w, w, 6, 0.0, 0.0, 0, Color::default(), 2.7);
        let d = dust.slot_mut(idx);
        let scatter = rotated_by_random(rng, Vec2::X, pi);
        d.position = center + rotated_by(scatter, vel_rot) * size / 2.0;
        d.no_gravity = true;
        d.velocity *= 3.0;
    }
    for _ in 0..n31x {
        let idx = dust.new_dust(rng, ctx, world, pos, w, w, 31, 0.0, 0.0, 0, Color::default(), 1.5);
        let d = dust.slot_mut(idx);
        let scatter = rotated_by_random(rng, Vec2::X, pi);
        d.position = center + rotated_by(scatter, vel_rot) * size / 2.0;
        d.no_gravity = true;
        d.velocity *= 3.0;
    }
    // `:71888-71900` — the trailing gore loop (scatter + lift variant).
    gore_census(rng, ngore, true, true);
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
    /// `base.ai[0]` — the 659 spirit-flame death gate reads it (`ai[0] >= 0`).
    pub ai0: f32,
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
    /// `base.localAI[1]` — the 985 Terraspark death gate reads it
    /// (`localAI[1] != 1f` suppresses the fan).
    pub local_ai1: f32,
    /// `Main.player[owner].setApprenticeT3` — the 664/666/668/706 DD2
    /// explosion's big-burst variant (`:71832-71843`). Populated at the
    /// death hook from the local player (the pool only dispatches
    /// locally-owned slots, so owner == myPlayer folds — the beenade
    /// precedent); no player → false.
    pub apprentice_t3: bool,
    /// `this.MaxUpdates` (`extraUpdates + 1`) — the 634/635 lunar-flare
    /// death's kick vector scales by it (`:72004`).
    pub max_updates: f32,
    /// `base.ai[2]` — the 590 shadow-flame death's gore/dust variant gate
    /// (`ai[2] == 1f` → dust 31 + gore 61, `:72618`).
    pub ai2: f32,
    /// `base.wet` — the 1037/1049 death's `!wet` dust gate (`:73901`). The
    /// pool's centre-cell liquid probe stand-in (the S1-P-i35 field doc).
    pub wet: bool,
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
            ai0: p.ai[0],
            rot: p.rot,
            direction: p.direction,
            alpha: p.alpha,
            old_pos: p.pos,
            local_ai1: p.local_ai[1],
            apprentice_t3: false,
            max_updates: (p.extra_updates + 1) as f32,
            ai2: p.ai[2],
            wet: p.wet,
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
        // `:76471-76503` — the 167 Red Rocket: 400× dust-130 6×6 alpha 100,
        // the ring kick at speed 4/8/12/16 by the <100/<200/<300 bands,
        // damp 0.5, scale 1.3 + noGravity. No explicit draw.
        167 => {
            for i in 0..400 {
                let speed = if i < 100 { 4.0 } else if i < 200 { 8.0 } else if i < 300 { 12.0 } else { 16.0 };
                let idx = dust.new_dust(rng, ctx, world, s.pos, 6, 6, 130, 0.0, 0.0, 100, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                ring_kick(d, speed, 0.5, 1.0, 1.0);
                d.scale = 1.3;
                d.no_gravity = true;
            }
        }
        // `:76505-76544` — the 168 Green Rocket: 400× dust-131, speed
        // 2×(i/100) then 10 (>100) then 13 (>250); the >200 band kicks
        // (×1.25, ×0.75); only i>100 gets scale 1.3 + noGravity. No draw.
        168 => {
            for i in 0..400 {
                let mut speed = 2.0 * (i as f32 / 100.0);
                if i > 100 {
                    speed = 10.0;
                }
                if i > 250 {
                    speed = 13.0;
                }
                let (xm, ym) = if i <= 200 { (1.0, 1.0) } else { (1.25, 0.75) };
                let idx = dust.new_dust(rng, ctx, world, s.pos, 6, 6, 131, 0.0, 0.0, 100, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                ring_kick(d, speed, 0.5, xm, ym);
                if i > 100 {
                    d.scale = 1.3;
                    d.no_gravity = true;
                }
            }
        }
        // `:76546-76605` — the 169 Blue Rocket star: one NextDouble heading +
        // Next(5,9) spokes + Next(12,17) speed + Next(3,7) inner speed, then
        // TWO spoke passes (2 sides × 20 lerp steps each, the second pass at
        // half speed), each step a dust-133 scale 1.3 vel×0.1 noGravity +
        // Lerp(spoke,side,t)×Lerp(speed,inner,t); then 100× dust-132 ring
        // kicks at the outer speed. (The i16 draw census is deduped onto
        // this arm — the pool hook no longer rolls the four draws.)
        169 => {
            let tau = std::f32::consts::PI * 2.0;
            let mut heading = rotation_vector(rng.next_double() as f32 * tau);
            let spokes = rng.next_range(5, 9) as f32;
            let speed = rng.next_range(12, 17) as f32;
            let inner = rng.next_range(3, 7) as f32;
            for pass in 0..2 {
                let half = if pass == 0 { 1.0 } else { 0.5 };
                for _ in 0..spokes as i32 {
                    for side in 0..2 {
                        let sign = if side == 0 { 1.0 } else { -1.0 };
                        let edge = rotated_by(heading, sign * tau / (spokes * 2.0));
                        for step in 0..20 {
                            let t = step as f32 / 20.0;
                            let dir = lerp_vec(heading, edge, t);
                            let mag = lerp_f32(speed, inner, t) * half;
                            let idx = dust.new_dust(rng, ctx, world, s.pos, 6, 6, 133, 0.0, 0.0, 100, Color::default(), 1.3);
                            let d = dust.slot_mut(idx);
                            d.velocity *= 0.1;
                            d.no_gravity = true;
                            d.velocity += dir * mag;
                        }
                    }
                    heading = rotated_by(heading, tau / spokes);
                }
            }
            for _ in 0..100 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, 6, 6, 132, 0.0, 0.0, 100, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                ring_kick(d, speed, 0.5, 1.0, 1.0);
                d.scale = 1.3;
                d.no_gravity = true;
            }
        }
        // `:76607-76673` — the 170 Yellow Rocket: 400× dust, type/speed by
        // the >100/>200/>300 bands (133/16, 134/11, 133/8, 134/5), the kick
        // axis-squash alternates ×0.7 per band, damp 0.5; per dust a Next(3)
        // gate ≠0 → scale 1.3 + noGravity. (The i16 400-gate census is
        // deduped onto this arm.)
        170 => {
            for i in 0..400 {
                let (dust_type, speed, xm, ym) = if i > 300 {
                    (134, 5.0, 0.7, 1.0)
                } else if i > 200 {
                    (133, 8.0, 1.0, 0.7)
                } else if i > 100 {
                    (134, 11.0, 0.7, 1.0)
                } else {
                    (133, 16.0, 1.0, 0.7)
                };
                let idx = dust.new_dust(rng, ctx, world, s.pos, 6, 6, dust_type, 0.0, 0.0, 100, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                ring_kick(d, speed, 0.5, xm, ym);
                if rng.next_max(3) != 0 {
                    d.scale = 1.3;
                    d.no_gravity = true;
                }
            }
        }
        // `:76675-76717` — the 415 Red Firework: NextDouble heading +
        // Next(5,9) spokes + Next(10,15)×0.66 outer + Next(4,7)/2 inner; the
        // heading rotates EVERY 30th particle (i%30==0, INCLUDING i=0), each
        // a dust-130 vel×0.1 + heading×Lerp(inner,outer,(i%30)/30) scale 1.3
        // noGravity; then 100 ring kicks (i<30 at (inner+outer)/2).
        415 => {
            let tau = std::f32::consts::PI * 2.0;
            let mut heading = rotation_vector(rng.next_double() as f32 * tau);
            let spokes = rng.next_range(5, 9) as f32;
            let outer = rng.next_range(10, 15) as f32 * 0.66;
            let inner = rng.next_range(4, 7) as f32 / 2.0;
            let count = (30.0 * spokes) as i32;
            for i in 0..count {
                if i % 30 == 0 {
                    heading = rotated_by(heading, tau / spokes);
                }
                let mag = lerp_f32(inner, outer, (i % 30) as f32 / 30.0);
                let idx = dust.new_dust(rng, ctx, world, s.pos, 6, 6, 130, 0.0, 0.0, 100, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.velocity *= 0.1;
                d.velocity += heading * mag;
                d.scale = 1.3;
                d.no_gravity = true;
            }
            for i in 0..100 {
                let speed = if i < 30 { (inner + outer) / 2.0 } else { outer };
                let idx = dust.new_dust(rng, ctx, world, s.pos, 6, 6, 130, 0.0, 0.0, 100, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                ring_kick(d, speed, 0.5, 1.0, 1.0);
                d.scale = 1.3;
                d.no_gravity = true;
            }
        }
        // `:76719-76748` — the 416 Green Firework spiral: NextDouble heading
        // + Next(3,6)×2 spokes + Next(2) sign; every 20th particle rotates
        // the BASE by sign×(τ/spokes) and flips the scale flag, the rest
        // rotate the WORKING vector by (τ/(20×spokes))×sign×3; each a
        // dust-131 scale 1.4 vel×0.1 + dir×Lerp(1,8,(i%20)/20), flag →
        // scale 0.9, noGravity.
        416 => {
            let tau = std::f32::consts::PI * 2.0;
            let mut work = rotation_vector(rng.next_double() as f32 * tau);
            let mut base = work;
            let spokes = (rng.next_range(3, 6) * 2) as f32;
            let sign = if rng.next_max(2) == 0 { 1.0 } else { -1.0 };
            let mut flag = true;
            let count = (20.0 * spokes) as i32;
            for i in 0..count {
                if i % 20 == 0 {
                    base = rotated_by(base, sign * (tau / spokes));
                    work = base;
                    flag = !flag;
                } else {
                    work = rotated_by(work, tau / (20.0 * spokes) * sign * 3.0);
                }
                let mag = lerp_f32(1.0, 8.0, (i % 20) as f32 / 20.0);
                let idx = dust.new_dust(rng, ctx, world, s.pos, 6, 6, 131, 0.0, 0.0, 100, Color::default(), 1.4);
                let d = dust.slot_mut(idx);
                d.velocity *= 0.1;
                d.velocity += work * mag;
                if flag {
                    d.scale = 0.9;
                }
                d.no_gravity = true;
            }
        }
        // `:76750-76802` — the 417 Blue Firework: TWO NextDouble angles +
        // TWO NextDouble radii (4+NextDouble×3 each); 150× dust, type/speed
        // by the >50/>100 bands (132/max, 133/r2, 132/r1); the dust's own
        // velocity Normalize()×speed, >100 squashes X×0.5 + rotates by
        // angle1, >50 squashes Y×0.5 + rotates by angle2; vel×0.2 + kick,
        // scale 1.3 + noGravity (the i<=200 gate always passes).
        417 => {
            let tau = std::f32::consts::PI * 2.0;
            let angle1 = rng.next_double() as f32 * tau;
            let angle2 = rng.next_double() as f32 * tau;
            let r1 = 4.0 + rng.next_double() as f32 * 3.0;
            let r2 = 4.0 + rng.next_double() as f32 * 3.0;
            let rmax = r1.max(r2);
            for i in 0..150 {
                let (dust_type, speed) = if i > 100 {
                    (132, r1)
                } else if i > 50 {
                    (133, r2)
                } else {
                    (132, rmax)
                };
                let idx = dust.new_dust(rng, ctx, world, s.pos, 6, 6, dust_type, 0.0, 0.0, 100, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                let mut kick = safe_normalize(d.velocity) * speed;
                if i > 100 {
                    kick.x *= 0.5;
                    kick = rotated_by(kick, angle1);
                } else if i > 50 {
                    kick.y *= 0.5;
                    kick = rotated_by(kick, angle2);
                }
                d.velocity *= 0.2;
                d.velocity += kick;
                d.scale = 1.3;
                d.no_gravity = true;
            }
        }
        // `:76804-76855` — the 418 Yellow Firework: NextDouble heading +
        // Next(5,12) spokes + Next(9,14)×0.66 outer + Next(2,4)×0.66 inner;
        // 2 sides × 15 SmoothStep steps per spoke (dust-134 scale 1.3,
        // vel×0.1 noGravity + SmoothStep(spoke,side,t)×SmoothStep(outer,
        // inner,t)); then 120 ring kicks damp 0.2 (i<80: dust-133 at
        // inner−0.5, else dust-131 at outer).
        418 => {
            let tau = std::f32::consts::PI * 2.0;
            let mut heading = rotation_vector(rng.next_double() as f32 * tau);
            let spokes = rng.next_range(5, 12) as f32;
            let outer = rng.next_range(9, 14) as f32 * 0.66;
            let inner = rng.next_range(2, 4) as f32 * 0.66;
            for _ in 0..spokes as i32 {
                for side in 0..2 {
                    let sign = if side == 0 { 1.0 } else { -1.0 };
                    let edge = rotated_by(heading, sign * tau / (spokes * 2.0));
                    for step in 0..15 {
                        let t = step as f32 / 15.0;
                        let dir = smooth_step_vec(heading, edge, t);
                        let mag = smooth_step_f32(outer, inner, t);
                        let idx = dust.new_dust(rng, ctx, world, s.pos, 6, 6, 134, 0.0, 0.0, 100, Color::default(), 1.3);
                        let d = dust.slot_mut(idx);
                        d.velocity *= 0.1;
                        d.no_gravity = true;
                        d.velocity += dir * mag;
                    }
                }
                heading = rotated_by(heading, tau / spokes);
            }
            for i in 0..120 {
                let (dust_type, speed) = if i < 80 { (133, inner - 0.5) } else { (131, outer) };
                let idx = dust.new_dust(rng, ctx, world, s.pos, 6, 6, dust_type, 0.0, 0.0, 100, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                ring_kick(d, speed, 0.2, 1.0, 1.0);
                d.scale = 1.3;
                d.no_gravity = true;
            }
        }
        // `:74817-74826` — the 1024 Sparkle ring: 12× dust-306 (Direct),
        // scale 0.8+NextFloat×1.7 BEFORE the spawn, colour Transparent;
        // velocity = SafeNormalize(vel,Zero).RotatedBy(τ×i/12)×8×(0.1+
        // NextFloat×1.2) AFTER, colour Lerp(White,(140,234,180),i/10),
        // noGravity. Two explicit floats per dust (scale then speed).
        1024 => {
            let tau = std::f32::consts::PI * 2.0;
            let dir = safe_normalize(s.vel);
            for i in 0..12 {
                let scale = 0.8 + rng.next_float() * 1.7;
                let idx = dust.new_dust_direct(rng, ctx, world, s.pos, s.w, s.h, 306, 0.0, 0.0, 0, Color::TRANSPARENT, scale);
                let d = dust.slot_mut(idx);
                let angle = tau * i as f32 / 12.0;
                d.velocity = rotated_by(dir, angle) * 8.0 * (0.1 + rng.next_float() * 1.2);
                d.color = lerp_color(Color::WHITE, Color::new(140, 234, 180, 255), i as f32 / 10.0);
                d.no_gravity = true;
            }
        }
        // `:71490-71520` — the 873 Prismatic Bolt ring: 20× dust-267
        // NewDustPerfect at Center, fadeIn 1/noGravity/alpha 100, colour
        // Lerp(AI_171_GetColor, White, NextFloat×0.4); the spoke heading
        // starts at vel.ToRotation() and climbs τ/20 per dust; i%4==0 →
        // vel = heading×3.2 scale 2.3, i%2==0 → ×1.8/1.9, else ×1/1.6;
        // vel += baseVel×NextFloat×0.5. Two explicit floats per dust.
        // AI_171_GetColor (`:45560-45567`): the enrage gate reads
        // ShouldEmpressBeEnraged — non-remix = Main.dayTime; the remix arm
        // scans for a live 636, and the port has NO live Empress (the
        // npc_ai_empress wiring remainder), so the scan finds none and the
        // sticky latch stays false → NOT enraged (recorded). Enraged:
        // Lerp(White, OurFavoriteColor, GetLerpValue(0,60,(int)time)) ×
        // Opacity; else hslToRgb((ai[1]+0.5)%1, 1, 0.5) × Opacity.
        873 => {
            let tau = std::f32::consts::PI * 2.0;
            let opacity = 1.0 - s.alpha as f32 / 255.0;
            let remix = world.seeds.contains(tr_core::seeds::SeedFlags::REMIX);
            let enraged = !remix && world.day_time;
            let base_color = if enraged {
                let t = crate::projectile::get_lerp_value(0.0, 60.0, world.time as i32 as f32);
                let c = lerp_color(Color::WHITE, Color::new(255, 231, 69, 255), t);
                let m = crate::ui::color::mul([c.r, c.g, c.b, c.a], opacity);
                Color::new(m[0], m[1], m[2], m[3])
            } else {
                let h = (s.ai1 + 0.5) % 1.0;
                let c = crate::ui::color::hsl_to_rgb(h as f64, 1.0, 0.5, 255);
                let m = crate::ui::color::mul(c, opacity);
                Color::new(m[0], m[1], m[2], m[3])
            };
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let mut heading = to_rotation(s.vel);
            for i in 0..20 {
                let idx = dust.new_dust_perfect(rng, ctx, world, center, 267, None, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.fade_in = 1.0;
                d.no_gravity = true;
                d.alpha = 100;
                d.color = lerp_color(base_color, Color::WHITE, rng.next_float() * 0.4);
                if i % 4 == 0 {
                    d.velocity = rotation_vector(heading) * 3.2;
                    d.scale = 2.3;
                } else if i % 2 == 0 {
                    d.velocity = rotation_vector(heading) * 1.8;
                    d.scale = 1.9;
                } else {
                    d.velocity = rotation_vector(heading);
                    d.scale = 1.6;
                }
                heading += tau / 20.0;
                d.velocity += s.vel * rng.next_float() * 0.5;
            }
        }
        // `:72503-72541` — the 659 Spirit Flame ring (gated ai[0] >= 0):
        // resize to 80×80 centred (the dust reads the resized box), 30×
        // dust-27 scale 2+NextFloat×0.5 noGravity; the first 15 are the
        // RING (fadeIn 1.5+NextFloat×0.5, position=Center, vel =
        // UnitY.RotatedBy((i+1)/15×τ)×(5+NextFloat×1.5)); the last 15
        // scatter (position = Center + UnitY.RotatedByRandom(π)×NextFloat×
        // w/3, fadeIn 0.5+NextFloat×0.5, vel×2); then 10× dust-31 scale 1.5
        // (position = Center + UnitX.RotatedByRandom(π).RotatedBy(vel rot)×
        // w/3, fadeIn 0.5+NextFloat×0.5, noGravity, vel×1.5). The resize +
        // Damage stay unported state (the pool's PVE walk already ran).
        659 => {
            if s.ai0 >= 0.0 {
                let tau = std::f32::consts::PI * 2.0;
                let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
                let pos = center - Vec2::splat(40.0);
                let w = 80;
                for i in 0..30 {
                    let scale = 2.0 + rng.next_float() * 0.5;
                    let idx = dust.new_dust(rng, ctx, world, pos, w, w, 27, 0.0, 0.0, 0, Color::default(), scale);
                    let d = dust.slot_mut(idx);
                    d.no_gravity = true;
                    if i < 15 {
                        let angle = (i + 1) as f32 / 15.0 * tau;
                        d.fade_in = 1.5 + rng.next_float() * 0.5;
                        d.position = center;
                        d.velocity = rotated_by(Vec2::Y, angle) * (5.0 + rng.next_float() * 1.5);
                    } else {
                        let scatter = rotated_by_random(rng, Vec2::Y, std::f32::consts::PI);
                        d.position = center + scatter * rng.next_float() * w as f32 / 3.0;
                        d.fade_in = 0.5 + rng.next_float() * 0.5;
                        d.velocity *= 2.0;
                    }
                }
                for _ in 0..10 {
                    let idx = dust.new_dust(rng, ctx, world, pos, w, w, 31, 0.0, 0.0, 0, Color::default(), 1.5);
                    let d = dust.slot_mut(idx);
                    let scatter = rotated_by_random(rng, Vec2::X, std::f32::consts::PI);
                    d.position = center + rotated_by(scatter, to_rotation(s.vel)) * w as f32 / 3.0;
                    d.fade_in = 0.5 + rng.next_float() * 0.5;
                    d.no_gravity = true;
                    d.velocity *= 1.5;
                }
            }
        }
        // `:71524-71549` — the 1012 First Fractal: 40×, each a UnitY.
        // RotatedBy(i×τ)×(2+NextFloat×0.5) + vel×NextFloat spoke; a Next(3)
        // gate: 0 → dust-284 (Direct) colour Next(3){(226,116,56),(243,175,
        // 93),(254,210,146)}, position pos+NextVector2Circular(18,18)×scale,
        // 2×2, speed spoke, scale NextFloat+1, position −= vel, fadeIn 1;
        // else spoke×0.3 → dust-138 (Direct) scale NextFloat×0.3+0.6,
        // position −= vel.
        1012 => {
            let tau = std::f32::consts::PI * 2.0;
            for i in 0..40 {
                let mut spoke = rotated_by(Vec2::Y, i as f32 * tau) * (2.0 + rng.next_float() * 0.5);
                spoke += s.vel * rng.next_float();
                if rng.next_max(3) == 0 {
                    let color = match rng.next_max(3) {
                        0 => Color::new(226, 116, 56, 255),
                        1 => Color::new(243, 175, 93, 255),
                        _ => Color::new(254, 210, 146, 255),
                    };
                    let pos = s.pos + next_vector2_circular(rng, 18.0, 18.0) * s.scale;
                    let scale = rng.next_float() + 1.0;
                    let idx = dust.new_dust_direct(rng, ctx, world, pos, 2, 2, 284, spoke.x, spoke.y, 0, color, scale);
                    let d = dust.slot_mut(idx);
                    d.position -= s.vel;
                    d.fade_in = 1.0;
                } else {
                    spoke *= 0.3;
                    let pos = s.pos + next_vector2_circular(rng, 18.0, 18.0) * s.scale;
                    let scale = rng.next_float() * 0.3 + 0.6;
                    let idx = dust.new_dust_direct(rng, ctx, world, pos, 2, 2, 138, spoke.x, spoke.y, 0, Color::default(), scale);
                    dust.slot_mut(idx).position -= s.vel;
                }
            }
        }
        // `:71551-71558` — the 920 Queen Slime gel: 6× dust-4, position
        // pos + Size×NextVector2Square(0,1) box 4×4, vel = UnitY.RotatedBy
        // (i×τ + NextFloat×τ×0.5)×(2+NextFloat×0.5), alpha 80 colour
        // (78,136,255,150) scale 1.5, noGravity.
        920 => {
            let tau = std::f32::consts::PI * 2.0;
            let size = Vec2::new(s.w as f32, s.h as f32);
            for i in 0..6 {
                let jitter = next_vector2_square(rng, 0.0, 1.0);
                let angle = i as f32 * tau + rng.next_float() * tau * 0.5;
                let speed = 2.0 + rng.next_float() * 0.5;
                let vel = rotated_by(Vec2::Y, angle) * speed;
                let pos = s.pos + size * jitter;
                let idx = dust.new_dust(rng, ctx, world, pos, 4, 4, 4, vel.x, vel.y, 80, Color::new(78, 136, 255, 150), 1.5);
                dust.slot_mut(idx).no_gravity = true;
            }
        }
        // `:71560-71567` — the 921: 12× dust-243 box 6×6, the 920 shape at
        // speed 3+NextFloat×0.5, alpha 80 default colour scale 1.5.
        921 => {
            let tau = std::f32::consts::PI * 2.0;
            let size = Vec2::new(s.w as f32, s.h as f32);
            for i in 0..12 {
                let jitter = next_vector2_square(rng, 0.0, 1.0);
                let angle = i as f32 * tau + rng.next_float() * tau * 0.5;
                let speed = 3.0 + rng.next_float() * 0.5;
                let vel = rotated_by(Vec2::Y, angle) * speed;
                let pos = s.pos + size * jitter;
                let idx = dust.new_dust(rng, ctx, world, pos, 6, 6, 243, vel.x, vel.y, 80, Color::default(), 1.5);
                dust.slot_mut(idx).no_gravity = true;
            }
        }
        // `:71569-71576` — the 926: 22× dust-31 at pos−(8,8) box (w+16)×
        // (h+16), the 921 spoke shape, alpha 40 QueenSlime_GetDustColor
        // (TWO NextFloat) scale 1.5, noGravity.
        926 => {
            let tau = std::f32::consts::PI * 2.0;
            let pos = s.pos - Vec2::splat(8.0);
            for i in 0..22 {
                let angle = i as f32 * tau + rng.next_float() * tau * 0.5;
                let speed = 3.0 + rng.next_float() * 0.5;
                let vel = rotated_by(Vec2::Y, angle) * speed;
                let color = queen_slime_dust_color(rng);
                let idx = dust.new_dust(rng, ctx, world, pos, s.w + 16, s.h + 16, 31, vel.x, vel.y, 40, color, 1.5);
                dust.slot_mut(idx).no_gravity = true;
            }
        }
        // `:71578-71585` — the 937: 12× dust-31, the 926 shape.
        937 => {
            let tau = std::f32::consts::PI * 2.0;
            let pos = s.pos - Vec2::splat(8.0);
            for i in 0..12 {
                let angle = i as f32 * tau + rng.next_float() * tau * 0.5;
                let speed = 3.0 + rng.next_float() * 0.5;
                let vel = rotated_by(Vec2::Y, angle) * speed;
                let color = queen_slime_dust_color(rng);
                let idx = dust.new_dust(rng, ctx, world, pos, s.w + 16, s.h + 16, 31, vel.x, vel.y, 40, color, 1.5);
                dust.slot_mut(idx).no_gravity = true;
            }
        }
        // `:71587-71596` — the 818 Stellar Tune: TWO f32-step rings
        // (num += 0.34, 3 iterations each), dust-278 NewDustPerfect at
        // pos + Size×NextVector2Square(0,1), vel = UnitY.RotatedBy(num×τ +
        // NextFloat×τ×0.5)×(2+NextFloat×0.5) [ring 1] / (1+NextFloat×0.5)
        // [ring 2], alpha 150, colour Lerp(White, Gold, NextFloat) scale 0.4
        // [ring 1] / Lerp(White, OurFavoriteColor, NextFloat) scale 0.7
        // [ring 2], noGravity.
        818 => {
            let tau = std::f32::consts::PI * 2.0;
            let size = Vec2::new(s.w as f32, s.h as f32);
            let mut num = 0.0f32;
            while num < 1.0 {
                let jitter = next_vector2_square(rng, 0.0, 1.0);
                let angle = num * tau + rng.next_float() * tau * 0.5;
                let speed = 2.0 + rng.next_float() * 0.5;
                let vel = rotated_by(Vec2::Y, angle) * speed;
                let color = lerp_color(Color::WHITE, Color::new(255, 215, 0, 255), rng.next_float());
                let pos = s.pos + size * jitter;
                let idx = dust.new_dust_perfect(rng, ctx, world, pos, 278, Some(vel), 150, color, 0.4);
                dust.slot_mut(idx).no_gravity = true;
                num += 0.34;
            }
            let mut num = 0.0f32;
            while num < 1.0 {
                let jitter = next_vector2_square(rng, 0.0, 1.0);
                let angle = num * tau + rng.next_float() * tau * 0.5;
                let speed = 1.0 + rng.next_float() * 0.5;
                let vel = rotated_by(Vec2::Y, angle) * speed;
                let color = lerp_color(Color::WHITE, Color::new(255, 231, 69, 255), rng.next_float());
                let pos = s.pos + size * jitter;
                let idx = dust.new_dust_perfect(rng, ctx, world, pos, 278, Some(vel), 150, color, 0.7);
                dust.slot_mut(idx).no_gravity = true;
                num += 0.34;
            }
        }
        // `:71598-71611` — the 856 Sparkle Guitar: 6× dust-58 alpha 150
        // scale 0.8; then TWO f32-step rings (num += 0.34) of dust-278
        // NewDustPerfect at Center, vel = UnitY.RotatedBy(num×τ + NextFloat×
        // τ×0.5)×(4+NextFloat×2) [ring 1] / (2+NextFloat×1) [ring 2], alpha
        // 150, colour Lerp(White, HotPink, NextFloat×0.5+0.5) [ring 1] /
        // Lerp(White, Orange, NextFloat×0.5+0.5) [ring 2], noGravity. (The
        // gore + resize + Damage stay the gore lane's / unported state.)
        856 => {
            let tau = std::f32::consts::PI * 2.0;
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            for _ in 0..6 {
                dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 58, 0.0, 0.0, 150, Color::default(), 0.8);
            }
            let mut num = 0.0f32;
            while num < 1.0 {
                let angle = num * tau + rng.next_float() * tau * 0.5;
                let speed = 4.0 + rng.next_float() * 2.0;
                let vel = rotated_by(Vec2::Y, angle) * speed;
                let color = lerp_color(Color::WHITE, Color::new(255, 105, 180, 255), rng.next_float() * 0.5 + 0.5);
                let idx = dust.new_dust_perfect(rng, ctx, world, center, 278, Some(vel), 150, color, 1.0);
                dust.slot_mut(idx).no_gravity = true;
                num += 0.34;
            }
            let mut num = 0.0f32;
            while num < 1.0 {
                let angle = num * tau + rng.next_float() * tau * 0.5;
                let speed = 2.0 + rng.next_float() * 1.0;
                let vel = rotated_by(Vec2::Y, angle) * speed;
                let color = lerp_color(Color::WHITE, Color::new(255, 165, 0, 255), rng.next_float() * 0.5 + 0.5);
                let idx = dust.new_dust_perfect(rng, ctx, world, center, 278, Some(vel), 150, color, 1.0);
                dust.slot_mut(idx).no_gravity = true;
                num += 0.34;
            }
        }
        // `:75125-75139` — the 728 Star Veil: 7× dust-58 speed vel×0.1
        // alpha 150 scale 0.8; then a f32-step ring (num += 0.125, 8 iters)
        // of dust-278 NewDustPerfect at Center, vel = UnitY.RotatedBy(num×τ
        // + NextFloat×0.5)×(4+NextFloat×4), alpha 150 CornflowerBlue,
        // noGravity; then a ring (num += 0.25, 4 iters) vel×(2+NextFloat×3)
        // Gold. (The gore stays the gore lane's.)
        728 => {
            let tau = std::f32::consts::PI * 2.0;
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            for _ in 0..7 {
                dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 58, s.vel.x * 0.1, s.vel.y * 0.1, 150, Color::default(), 0.8);
            }
            let mut num = 0.0f32;
            while num < 1.0 {
                let angle = num * tau + rng.next_float() * 0.5;
                let speed = 4.0 + rng.next_float() * 4.0;
                let vel = rotated_by(Vec2::Y, angle) * speed;
                let idx = dust.new_dust_perfect(rng, ctx, world, center, 278, Some(vel), 150, Color::new(100, 149, 237, 255), 1.0);
                dust.slot_mut(idx).no_gravity = true;
                num += 0.125;
            }
            let mut num = 0.0f32;
            while num < 1.0 {
                let angle = num * tau + rng.next_float() * 0.5;
                let speed = 2.0 + rng.next_float() * 3.0;
                let vel = rotated_by(Vec2::Y, angle) * speed;
                let idx = dust.new_dust_perfect(rng, ctx, world, center, 278, Some(vel), 150, Color::new(255, 215, 0, 255), 1.0);
                dust.slot_mut(idx).no_gravity = true;
                num += 0.25;
            }
        }
        // `:70996-71041` — the 985 Terraspark fan (gated localAI[1] != 1):
        // THREE passes (20/10/10) of dust-107 NewDustPerfect. Per dust:
        // NextFloatDirection → num26 = 1−|dir|, angle = rot + dir×(π/2)×0.9,
        // position = Center + angle.ToRotationVector2()×Remap(NextFloat,0,1,
        // rmin,rmax)×scale − rot.ToRotationVector2()×pull×70×scale; a
        // DISCARDED colour NextFloat; vel = baseVel.RotatedBy(dir×π/4)×0.2×
        // NextFloat, alpha 100 scale 1.4×num26; position −= vel×NextFloat×3;
        // scale ×= 1−pull. Pass 1 pull=NextFloat rmin/rmax 60/85; pass 2
        // pull=0.2+NextFloat×0.25 70/85; pass 3 pull=0.5+NextFloat×0.25
        // 60/75.
        985 => {
            if s.local_ai1 != 1.0 {
                let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
                let rot_vec = rotation_vector(s.rot);
                for (count, rmin, rmax, pull_base, pull_mul) in [
                    (20, 60.0f32, 85.0f32, 0.0f32, 1.0f32),
                    (10, 70.0, 85.0, 0.2, 0.25),
                    (10, 60.0, 75.0, 0.5, 0.25),
                ] {
                    for _ in 0..count {
                        let dir = rng.next_float_direction();
                        let num26 = 1.0 - dir.abs();
                        let angle = s.rot + dir * (std::f32::consts::FRAC_PI_2) * 0.9;
                        let radius = remap_f32(rng.next_float(), 0.0, 1.0, rmin, rmax);
                        let pull = if pull_base == 0.0 && pull_mul == 1.0 {
                            rng.next_float()
                        } else {
                            pull_base + rng.next_float() * pull_mul
                        };
                        let mut pos = center + rotation_vector(angle) * radius * s.scale;
                        pos -= rot_vec * pull * 70.0 * s.scale;
                        let _ = rng.next_float(); // the discarded colour lerp
                        let vel = rotated_by(s.vel, dir * (std::f32::consts::FRAC_PI_4)) * 0.2 * rng.next_float();
                        let idx = dust.new_dust_perfect(rng, ctx, world, pos, 107, Some(vel), 100, Color::default(), 1.4 * num26);
                        let d = dust.slot_mut(idx);
                        d.position -= d.velocity * rng.next_float() * 3.0;
                        d.scale *= 1.0 - pull;
                    }
                }
            }
        }
        // `:71652-71679` — the 710 DD2 ballista explosion: resize 40×40
        // centred (position=Center OLD box, then Center=position NEW box —
        // the centre stays), ai[1]=-1 + Damage stay unported state. 2×
        // dust-31 alpha 100 scale 1.5 scatter UnitY.RotatedByRandom(π)×
        // NextDouble×w/2 (NO noGravity); 10× dust-6 alpha 0 scale 2.5 the
        // same scatter noGravity vel×2; 5× dust-31 alpha 0 scale 1.5
        // UnitX.RotatedByRandom(π).RotatedBy(vel rot)×w/2 noGravity vel×2.
        710 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let pos = center - Vec2::splat(20.0);
            let pi = std::f32::consts::PI;
            for _ in 0..2 {
                let idx = dust.new_dust(rng, ctx, world, pos, 40, 40, 31, 0.0, 0.0, 100, Color::default(), 1.5);
                let scatter = rotated_by_random(rng, Vec2::Y, pi);
                dust.slot_mut(idx).position = center + scatter * rng.next_double() as f32 * 40.0 / 2.0;
            }
            for _ in 0..10 {
                let idx = dust.new_dust(rng, ctx, world, pos, 40, 40, 6, 0.0, 0.0, 0, Color::default(), 2.5);
                let d = dust.slot_mut(idx);
                let scatter = rotated_by_random(rng, Vec2::Y, pi);
                d.position = center + scatter * rng.next_double() as f32 * 40.0 / 2.0;
                d.no_gravity = true;
                d.velocity *= 2.0;
            }
            let vel_rot = to_rotation(s.vel);
            for _ in 0..5 {
                let idx = dust.new_dust(rng, ctx, world, pos, 40, 40, 31, 0.0, 0.0, 0, Color::default(), 1.5);
                let d = dust.slot_mut(idx);
                let scatter = rotated_by_random(rng, Vec2::X, pi);
                d.position = center + rotated_by(scatter, vel_rot) * 40.0 / 2.0;
                d.no_gravity = true;
                d.velocity *= 2.0;
            }
        }
        // `:71780-71799` — the 662/685 DD2 flame trail: dir =
        // oldVelocity.SafeNormalize(Zero), walk from pos+dir×16; 16 steps,
        // each a Next(2) coin — 0 skips the dust (the walk STILL steps back
        // 8); else dust-11 Direct, position = (own pos + Center)/2, vel +=
        // oldVel×0.4 then ×0.5, noGravity; step back dir×8 either way.
        662 | 685 => {
            let dir = safe_normalize(s.old_vel);
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let mut walk = s.pos + dir * 16.0;
            for _ in 0..16 {
                if rng.next_max(2) == 0 {
                    walk -= dir * 8.0;
                    continue;
                }
                let idx = dust.new_dust_direct(rng, ctx, world, walk, s.w, s.h, 11, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.position = (d.position + center) / 2.0;
                d.velocity += s.old_vel * 0.4;
                d.velocity *= 0.5;
                d.no_gravity = true;
                walk -= dir * 8.0;
            }
        }
        // `:71800-71820` — the 680: the 662 trail + one dust-11 Direct at
        // pos, alpha 0 Red scale 1.6 noGravity.
        680 => {
            let dir = safe_normalize(s.old_vel);
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let mut walk = s.pos + dir * 16.0;
            for _ in 0..16 {
                if rng.next_max(2) == 0 {
                    walk -= dir * 8.0;
                    continue;
                }
                let idx = dust.new_dust_direct(rng, ctx, world, walk, s.w, s.h, 11, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.position = (d.position + center) / 2.0;
                d.velocity += s.old_vel * 0.4;
                d.velocity *= 0.5;
                d.no_gravity = true;
                walk -= dir * 8.0;
            }
            let idx = dust.new_dust_direct(rng, ctx, world, s.pos, s.w, s.h, 11, 0.0, 0.0, 0, Color::new(255, 0, 0, 255), 1.6);
            dust.slot_mut(idx).no_gravity = true;
        }
        // `:71821-71901` — the 664/666/668/706 DD2 apprentice explosion:
        // the T3 flag doubles the burst (see `dd2_explosion_dust`); the
        // resize + Damage stay unported state, the trailing gore loop is
        // the draws-only census inside the helper.
        664 | 666 | 668 | 706 => dd2_explosion_dust(s, dust, rng, ctx, world),
        // `:71952-71966` — the 669 DD2 flameburst: 12× dust-4 Direct at
        // Center−(15,15) box 30×30 alpha 100 (255,255,255,110) scale 1.1
        // vel×1.5; then 80 PAIRS: dust-4 alpha 50 (245,200,30,155) scale
        // 1.2 noGravity vel×=Next(10,71)×0.1, then scale 0.8 (gravity ON)
        // vel×=Next(10,41)×0.1. The 144 resize + buffs + Damage AFTER the
        // dust stay unported state (the dust reads the ORIGINAL centre).
        669 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let pos = center - Vec2::splat(15.0);
            for _ in 0..12 {
                let idx = dust.new_dust_direct(rng, ctx, world, pos, 30, 30, 4, 0.0, 0.0, 100, Color::new(255, 255, 255, 110), 1.1);
                dust.slot_mut(idx).velocity *= 1.5;
            }
            for _ in 0..80 {
                let idx = dust.new_dust_direct(rng, ctx, world, pos, 30, 30, 4, 0.0, 0.0, 50, Color::new(245, 200, 30, 155), 1.2);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= rng.next_range(10, 71) as f32 * 0.1;
                let idx = dust.new_dust_direct(rng, ctx, world, pos, 30, 30, 4, 0.0, 0.0, 50, Color::new(245, 200, 30, 155), 0.8);
                dust.slot_mut(idx).velocity *= rng.next_range(10, 41) as f32 * 0.1;
            }
        }
        // `:72146-72154` — the 614: 10× dust-229 at pos box w×h defaults,
        // noGravity vel×3.
        614 => {
            for _ in 0..10 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 229, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 3.0;
            }
        }
        // `:71995-72050` — the 634/635 lunar flare: resize 50×50 centred,
        // kick = (rot−π/2).ToRotationVector2()×|vel|×MaxUpdates (635: ×0.5).
        // 40 PAIRS: a SelectRandom type (634: 242/73/72/71/255, 635:
        // 242/59/88 — the head ALSO rolls one, consumed before the resize)
        // dust alpha 200 scale 1.7/3.7 Y-scatter noGravity vel×3 + kick×
        // NextFloat; then type 255/88 dust alpha 100 scale 0.8/1.5
        // Y-scatter vel×2 noGravity fadeIn 1 Crimson×0.5 + kick×NextFloat.
        // Then 20× type 255/88 alpha 0 scale 2/2.2 X-scatter÷3 noGravity
        // vel×0.5 + kick×(0.6+0.6×NextFloat).
        634 | 635 => {
            let is635 = s.kind == 635;
            // The head roll (`:71997`) runs for BOTH kinds; the 635 then
            // RE-rolls the 3-set (`:72009`) — two draws for the 635.
            let _ = rng.next_from_slice(&[242, 73, 72, 71, 255]);
            if is635 {
                let _ = rng.next_from_slice(&[242, 59, 88]);
            }
            let mid_type = if is635 { 88 } else { 255 };
            let (big_scale, mid_scale, tail_scale) = if is635 { (3.7, 1.5, 2.2) } else { (1.7, 0.8, 2.0) };
            let mut kick = rotation_vector(s.rot - std::f32::consts::FRAC_PI_2) * s.vel.length() * s.max_updates;
            if is635 {
                kick *= 0.5;
            }
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let pos = center - Vec2::splat(25.0);
            let vel_rot = to_rotation(s.vel);
            for _ in 0..40 {
                // `:72024-72028` — the 5-set roll, then the 635's 3-set
                // re-roll (two draws per pair for the 635).
                let mut dust_type = rng.next_from_slice(&[242, 73, 72, 71, 255]);
                if is635 {
                    dust_type = rng.next_from_slice(&[242, 59, 88]);
                }
                let idx = scatter_y_dust(dust, rng, ctx, world, pos, 50, center, dust_type, 200, big_scale, 2.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 3.0;
                d.velocity += kick * rng.next_float();
                let idx = scatter_y_dust(dust, rng, ctx, world, pos, 50, center, mid_type, 100, mid_scale, 2.0);
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
                d.fade_in = 1.0;
                d.color = crimson_half();
                d.velocity += kick * rng.next_float();
            }
            for _ in 0..20 {
                let idx = scatter_x_dust(dust, rng, ctx, world, pos, 50, center, vel_rot, mid_type, 0, tail_scale, 3.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 0.5;
                d.velocity += kick * (0.6 + 0.6 * rng.next_float());
            }
        }
        // `:72094-72125` — the 645: flag8 = SolidTile(pos/16); 4× dust-31
        // alpha 100 scale 1.5; 4 PAIRS dust-229 (alpha 0 scale 2.5 noGravity
        // vel×3, then alpha 100 scale 1.5 vel×2 noGravity), flag8 → noLight
        // on both; then ONE gore (draws-only census, no scatter/lift).
        645 => {
            let flag8 = crate::projectile::solid_tile(world, s.pos.x as i32 / 16, s.pos.y as i32 / 16);
            for _ in 0..4 {
                dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 31, 0.0, 0.0, 100, Color::default(), 1.5);
            }
            for _ in 0..4 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 229, 0.0, 0.0, 0, Color::default(), 2.5);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 3.0;
                if flag8 {
                    d.no_light = true;
                }
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 229, 0.0, 0.0, 100, Color::default(), 1.5);
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
                if flag8 {
                    d.no_light = true;
                }
            }
            gore_census(rng, 1, false, false);
        }
        // `:72226-72277` — the 661 nebula blaze explosion: resize 160×160
        // centred; the spawn box is Center−(20,20) 40×40 (NOT the resized
        // box), the scatter radius num185/2 = 20. 4× dust-240 alpha 100
        // scale 1.5 Y-scatter (gravity ON); 20 PAIRS dust-62 (alpha 200
        // scale 3.7 noGravity noLight vel×3 + DirectionTo(dust pos)×(2+
        // NextFloat×4); then alpha 100 scale 1.5 vel×2 noGravity fadeIn 1
        // Crimson×0.5 noLight + DirectionTo×8); 20× dust-62 alpha 0 scale
        // 2.7 X-scatter noGravity noLight vel×3 + DirectionTo×2; 70×
        // dust-240 alpha 0 scale 1.5 X-scatter noGravity vel×3 +
        // DirectionTo×3. DirectionTo = Normalize(dest − Center)
        // (`Entity.cs:207`), zero-safe.
        661 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let pos = center - Vec2::splat(20.0);
            let vel_rot = to_rotation(s.vel);
            for _ in 0..4 {
                scatter_y_dust(dust, rng, ctx, world, pos, 40, center, 240, 100, 1.5, 2.0);
            }
            for _ in 0..20 {
                let idx = scatter_y_dust(dust, rng, ctx, world, pos, 40, center, 62, 200, 3.7, 2.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.no_light = true;
                d.velocity *= 3.0;
                let dir = safe_normalize(d.position - center);
                d.velocity += dir * (2.0 + rng.next_float() * 4.0);
                let idx = scatter_y_dust(dust, rng, ctx, world, pos, 40, center, 62, 100, 1.5, 2.0);
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
                d.fade_in = 1.0;
                d.color = crimson_half();
                d.no_light = true;
                let dir = safe_normalize(d.position - center);
                d.velocity += dir * 8.0;
            }
            for _ in 0..20 {
                let idx = scatter_x_dust(dust, rng, ctx, world, pos, 40, center, vel_rot, 62, 0, 2.7, 2.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.no_light = true;
                d.velocity *= 3.0;
                let dir = safe_normalize(d.position - center);
                d.velocity += dir * 2.0;
            }
            for _ in 0..70 {
                let idx = scatter_x_dust(dust, rng, ctx, world, pos, 40, center, vel_rot, 240, 0, 1.5, 2.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 3.0;
                let dir = safe_normalize(d.position - center);
                d.velocity += dir * 3.0;
            }
        }
        // `:72278-72326` — the 617 nebula arcanum explosion: resize 176×176
        // centred; 4× dust-240 alpha 100 scale 1.5 Y-scatter÷2 (gravity ON);
        // 30 PAIRS dust-62 alpha 200 scale 3.7 noGravity vel×3 + dust-90
        // alpha 100 scale 1.5 vel×2 noGravity fadeIn 1 Crimson×0.5; 10×
        // dust-62 alpha 0 scale 2.7 X-scatter÷2 noGravity vel×3; 10×
        // dust-240 alpha 0 scale 1.5 X-scatter÷2 noGravity vel×3; 2 gores
        // (scatter, no lift). The owner-gated 618 chain-kill + the 620
        // spawn burst (`:72327-72372`) stay unported state — their draws
        // are NOT consumed here (the spawn lane owns them when it lands).
        617 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let pos = center - Vec2::splat(88.0);
            let vel_rot = to_rotation(s.vel);
            for _ in 0..4 {
                scatter_y_dust(dust, rng, ctx, world, pos, 176, center, 240, 100, 1.5, 2.0);
            }
            for _ in 0..30 {
                let idx = scatter_y_dust(dust, rng, ctx, world, pos, 176, center, 62, 200, 3.7, 2.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 3.0;
                let idx = scatter_y_dust(dust, rng, ctx, world, pos, 176, center, 90, 100, 1.5, 2.0);
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
                d.fade_in = 1.0;
                d.color = crimson_half();
            }
            for _ in 0..10 {
                let idx = scatter_x_dust(dust, rng, ctx, world, pos, 176, center, vel_rot, 62, 0, 2.7, 2.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 3.0;
            }
            for _ in 0..10 {
                let idx = scatter_x_dust(dust, rng, ctx, world, pos, 176, center, vel_rot, 240, 0, 1.5, 2.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 3.0;
            }
            gore_census(rng, 2, true, false);
        }
        // `:72394-72417` — the 620/618 nebula spark: 10× dust-(int)ai[0]
        // speed vel×0.1 scale 0.5, the crystal-spark Next(3) gate (0 →
        // fadeIn 0.75+Next(-10,11)×0.01, scale 0.25+×0.005, type++; else
        // scale 1+×0.01), noGravity vel×1.25 −= oldVel/10. The 618's
        // ai[0]=86 write happens BEFORE the dust (`:72396-72399`) — the
        // snapshot's ai0 is pre-write, so the arm uses 86 for the 618.
        620 | 618 => {
            let dust_type = if s.kind == 618 { 86 } else { s.ai0 as i32 };
            crystal_spark_burst(s, dust, rng, ctx, world, 10, dust_type, 0.75, 0.25, 0.005, 1.0, 0.01, 1.25);
        }
        // `:72418-72437` — the 619: 20× the 620 shape at fadeIn 1.1 / scale
        // 0.35+×0.01 / big 1.2 / vel×2.5. The owner-gated 620 spawn burst
        // (`:72438-72452`) stays unported state (the spawn lane's draws).
        619 => {
            crystal_spark_burst(s, dust, rng, ctx, world, 20, s.ai0 as i32, 1.1, 0.35, 0.01, 1.2, 0.01, 2.5);
        }
        // `:72454-72469` — the 601 portal: colour GetPortalColor(owner,
        // (int)ai[0]) with A=255 (single-player: portal!=0 → hsl(0.52,1,0.6)
        // else hsl(0.12,1,0.5)); 6× dust-263 NewDust at Center 0×0 defaults,
        // per dust the kick UnitY.RotatedByRandom(τ)×(3×NextFloat) BEFORE
        // the spawn, then position=Center, vel=kick+vel/5, colour, scale 2
        // noLight noGravity.
        601 => {
            let mut color = portal_color(s.ai0 as i32);
            color.a = 255;
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let tau = std::f32::consts::PI * 2.0;
            for _ in 0..6 {
                let kick = rotated_by_random(rng, Vec2::Y, tau) * (3.0 * rng.next_float());
                let idx = dust.new_dust(rng, ctx, world, center, 0, 0, 263, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.position = center;
                d.velocity = kick + s.vel / 5.0;
                d.color = color;
                d.scale = 2.0;
                d.no_light = true;
                d.no_gravity = true;
            }
        }
        // `:72470-72502` — the 596: resize 60×60 centred (the expert damage
        // 22 vs 30 + Damage stay unported state); 4× dust-31 alpha 100
        // scale 1.5 Y-scatter÷2 (gravity ON); 20× dust-27 alpha 0 scale 2.5
        // Y-scatter÷2 noGravity vel×2; 10× dust-31 alpha 0 scale 1.5
        // X-scatter÷2 noGravity vel×2.
        596 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let pos = center - Vec2::splat(30.0);
            let vel_rot = to_rotation(s.vel);
            for _ in 0..4 {
                scatter_y_dust(dust, rng, ctx, world, pos, 60, center, 31, 100, 1.5, 2.0);
            }
            for _ in 0..20 {
                let idx = scatter_y_dust(dust, rng, ctx, world, pos, 60, center, 27, 0, 2.5, 2.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 2.0;
            }
            for _ in 0..10 {
                let idx = scatter_x_dust(dust, rng, ctx, world, pos, 60, center, vel_rot, 31, 0, 1.5, 2.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 2.0;
            }
        }
        // `:72565-72598` — the 539: resize 80×80 centred; 4× dust-31 alpha
        // 100 scale 1.5 Y-scatter÷2 (gravity ON); 20× dust-176 alpha 200
        // scale 3.7 Y-scatter÷2 noGravity vel×3; 20× dust-180 alpha 0 scale
        // 2.7 X-scatter÷2 noGravity vel×3; 10× dust-31 alpha 0 scale 1.5
        // X-scatter÷2 noGravity vel×3.
        539 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let pos = center - Vec2::splat(40.0);
            let vel_rot = to_rotation(s.vel);
            for _ in 0..4 {
                scatter_y_dust(dust, rng, ctx, world, pos, 80, center, 31, 100, 1.5, 2.0);
            }
            for _ in 0..20 {
                let idx = scatter_y_dust(dust, rng, ctx, world, pos, 80, center, 176, 200, 3.7, 2.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 3.0;
            }
            for _ in 0..20 {
                let idx = scatter_x_dust(dust, rng, ctx, world, pos, 80, center, vel_rot, 180, 0, 2.7, 2.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 3.0;
            }
            for _ in 0..10 {
                let idx = scatter_x_dust(dust, rng, ctx, world, pos, 80, center, vel_rot, 31, 0, 1.5, 2.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 3.0;
            }
        }
        // `:72599-72613` — the 585: 20 PAIRS dust-26 at pos alpha 100
        // defaults noGravity vel×1.2 scale 1.3 −= oldVel×0.3, then dust-27
        // at pos+(4,4) box (w−8)×(h−8) alpha 100 scale 2 noGravity vel×3.
        585 => {
            for _ in 0..20 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 26, 0.0, 0.0, 100, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 1.2;
                d.scale = 1.3;
                d.velocity -= s.old_vel * 0.3;
                let idx = dust.new_dust(rng, ctx, world, s.pos + Vec2::splat(4.0), s.w - 8, s.h - 8, 27, 0.0, 0.0, 100, Color::default(), 2.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 3.0;
            }
        }
        // `:72614-72640` — the 590 shadowflame: 10× dust-165 (ai[2]==1 →
        // 31) alpha 50 scale 1.5 vel×2 noGravity; then ONE NextFloat scale
        // 0.6+×0.4 shared by THREE gores 375/376/377 (ai[2]==1 → 61/62/63)
        // at pos Zero vel — draws-only census: the kicks Next(10,31) Y then
        // Next(-20,21) X per gore (`Gore.cs:1327-1328`), bodies skipped.
        590 => {
            let alt = s.ai2 == 1.0;
            let dust_type = if alt { 31 } else { 165 };
            for _ in 0..10 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, dust_type, 0.0, 0.0, 50, Color::default(), 1.5);
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
            let _ = rng.next_float(); // the shared gore scale (`:72628`)
            for _ in 0..3 {
                let _ = rng.next_range(10, 31);
                let _ = rng.next_range(-20, 21);
            }
        }
        // `:72641-72667` — the 587: colour hslToRgb(ai[1],1,0.5) A=200 (the
        // NPC slime-tint scan is unported state, no draws); 10 PAIRS dust-76
        // at pos alpha 0 colour defaults noGravity vel×1.2 scale 0.9 −=
        // oldVel×0.3, then dust-76 at pos+(4,4) box (w−8)×(h−8) alpha 0
        // colour scale 1.1 noGravity vel×2.
        587 => {
            let c = crate::ui::color::hsl_to_rgb(s.ai1 as f64, 1.0, 0.5, 200);
            let color = Color::new(c[0], c[1], c[2], c[3]);
            for _ in 0..10 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 76, 0.0, 0.0, 0, color, 1.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 1.2;
                d.scale = 0.9;
                d.velocity -= s.old_vel * 0.3;
                let idx = dust.new_dust(rng, ctx, world, s.pos + Vec2::splat(4.0), s.w - 8, s.h - 8, 76, 0.0, 0.0, 0, color, 1.1);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 2.0;
            }
        }
        // `:72737-72771` — the 676: 120× SelectRandom(4,256) dust alpha 100
        // defaults, scale 0.8+NextFloat×0.6 fadeIn 0.5 vel×4.5 noLight;
        // vel.Y>0 → vel×=−0.5; type 4 → colour (80,170,40,120). Then 50
        // gore-Direct rows (10×1024 + 10×1025 + 10×1026 at Lerp(−5,5,
        // NextFloat) X / −NextFloat×5 Y, then 20×1026 at Lerp(−0.5,0.5) /
        // −NextFloat×2) — each NewGoreDirect wraps NewGore (`Gore.cs:1281-
        // 1284`), so the two kick draws per gore stay; bodies skipped.
        676 => {
            for _ in 0..120 {
                let dust_type = rng.next_from_slice(&[4, 256]);
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, dust_type, 0.0, 0.0, 100, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.scale = 0.8 + rng.next_float() * 0.6;
                d.fade_in = 0.5;
                d.velocity *= 4.5;
                d.no_light = true;
                if d.velocity.y > 0.0 {
                    d.velocity *= -0.5;
                }
                if d.dust_type == 4 {
                    d.color = Color::new(80, 170, 40, 120);
                }
            }
            for _ in 0..30 {
                let _ = rng.next_float(); // the X lerp
                let _ = rng.next_float(); // the Y kick
                let _ = rng.next_range(10, 31);
                let _ = rng.next_range(-20, 21);
            }
            for _ in 0..20 {
                let _ = rng.next_float();
                let _ = rng.next_float();
                let _ = rng.next_range(10, 31);
                let _ = rng.next_range(-20, 21);
            }
        }
        // `:72849-72873` — the 405 chlorophyte ring: 20× at r=10.
        405 => chlorophyte_ring(s, dust, rng, ctx, world, 20, 10.0),
        // `:72874-72918` — the 501: the box inflates +20 each side (pos−20,
        // w/h+40) BEFORE the dust; 20× dust-188 alpha 100 scale 1.5 vel×0.5
        // (gravity ON); then 5 gores at pos+(Next(w),Next(h)) type
        // Next(435,438) — draws-only census (the X/Y position rolls, the
        // type argument, the two kicks); the second +40 inflate + Damage
        // stay unported state.
        501 => {
            let pos = s.pos - Vec2::splat(20.0);
            let w = s.w + 40;
            let h = s.h + 40;
            for _ in 0..20 {
                let idx = dust.new_dust(rng, ctx, world, pos, w, h, 188, 0.0, 0.0, 100, Color::default(), 1.5);
                dust.slot_mut(idx).velocity *= 0.5;
            }
            for _ in 0..5 {
                let _ = rng.next_max(w);
                let _ = rng.next_max(h);
                let _ = rng.next_range(435, 438);
                let _ = rng.next_range(10, 31);
                let _ = rng.next_range(-20, 21);
            }
        }
        // `:72919-72943` — the 410: the 405 ring at count 10, r =
        // (int)(10×ai[1]).
        410 => {
            let r = (10.0 * s.ai1) as i32 as f32;
            chlorophyte_ring(s, dust, rng, ctx, world, 10, r);
        }
        // `:72987-73013` — the 452: Resize(144,144) centred; 4× dust-31
        // alpha 100 scale 1.5 (NO scatter, gravity ON); 40 PAIRS dust-229
        // (alpha 0 scale 2.5 noGravity vel×3, then alpha 100 scale 1.5
        // vel×2 noGravity); ONE gore census; the Resize(30,30) + Damage
        // AFTER stay unported state.
        452 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let pos = center - Vec2::splat(72.0);
            for _ in 0..4 {
                dust.new_dust(rng, ctx, world, pos, 144, 144, 31, 0.0, 0.0, 100, Color::default(), 1.5);
            }
            for _ in 0..40 {
                let idx = dust.new_dust(rng, ctx, world, pos, 144, 144, 229, 0.0, 0.0, 0, Color::default(), 2.5);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 3.0;
                let idx = dust.new_dust(rng, ctx, world, pos, 144, 144, 229, 0.0, 0.0, 100, Color::default(), 1.5);
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
            gore_census(rng, 1, false, false);
        }
        // `:73014-73045` — the 454: resize 208×208 centred; 7× dust-31
        // alpha 100 scale 1.5 (gravity ON) + 60 PAIRS dust-229 (alpha 0
        // scale 2.5 / alpha 100 scale 1.5, both vel×1 noGravity), each
        // position = Center + (w/2,0).RotatedBy(τ×NextDouble)×NextDouble
        // (the double-angle spin, radius draw AFTER); 3 gores census.
        454 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let pos = center - Vec2::splat(104.0);
            for _ in 0..7 {
                let idx = dust.new_dust(rng, ctx, world, pos, 208, 208, 31, 0.0, 0.0, 100, Color::default(), 1.5);
                dust.slot_mut(idx).position = double_angle_ring_pos(rng, center, 104.0);
            }
            for _ in 0..60 {
                let idx = dust.new_dust(rng, ctx, world, pos, 208, 208, 229, 0.0, 0.0, 0, Color::default(), 2.5);
                let d = dust.slot_mut(idx);
                d.position = double_angle_ring_pos(rng, center, 104.0);
                d.no_gravity = true;
                d.velocity *= 1.0;
                let idx = dust.new_dust(rng, ctx, world, pos, 208, 208, 229, 0.0, 0.0, 100, Color::default(), 1.5);
                let d = dust.slot_mut(idx);
                d.position = double_angle_ring_pos(rng, center, 104.0);
                d.velocity *= 1.0;
                d.no_gravity = true;
            }
            gore_census(rng, 3, false, false);
        }
        // `:73046-73092` — the 467: resize 176×176 centred; 4× dust-31
        // alpha 100 scale 1.5 Y-scatter÷2 (gravity ON); 30 PAIRS dust-6
        // (alpha 200 scale 3.7 noGravity vel×3, then alpha 100 scale 1.5
        // vel×2 noGravity fadeIn 2.5); 10× dust-6 alpha 0 scale 2.7
        // X-scatter÷2 noGravity vel×3; 10× dust-31 alpha 0 scale 1.5
        // X-scatter÷2 noGravity vel×3; 2 gores (scatter, no lift).
        467 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let pos = center - Vec2::splat(88.0);
            let vel_rot = to_rotation(s.vel);
            for _ in 0..4 {
                scatter_y_dust(dust, rng, ctx, world, pos, 176, center, 31, 100, 1.5, 2.0);
            }
            for _ in 0..30 {
                let idx = scatter_y_dust(dust, rng, ctx, world, pos, 176, center, 6, 200, 3.7, 2.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 3.0;
                let idx = scatter_y_dust(dust, rng, ctx, world, pos, 176, center, 6, 100, 1.5, 2.0);
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
                d.fade_in = 2.5;
            }
            for _ in 0..10 {
                let idx = scatter_x_dust(dust, rng, ctx, world, pos, 176, center, vel_rot, 6, 0, 2.7, 2.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 3.0;
            }
            for _ in 0..10 {
                let idx = scatter_x_dust(dust, rng, ctx, world, pos, 176, center, vel_rot, 31, 0, 1.5, 2.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 3.0;
            }
            gore_census(rng, 2, true, false);
        }
        // `:73093-73139` — the 468: the 467 shape at 20 pairs of dust-27
        // (fadeIn 2.5 mid) + 10 dust-27 X + 10 dust-31 X.
        468 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let pos = center - Vec2::splat(88.0);
            let vel_rot = to_rotation(s.vel);
            for _ in 0..4 {
                scatter_y_dust(dust, rng, ctx, world, pos, 176, center, 31, 100, 1.5, 2.0);
            }
            for _ in 0..20 {
                let idx = scatter_y_dust(dust, rng, ctx, world, pos, 176, center, 27, 200, 3.7, 2.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 3.0;
                let idx = scatter_y_dust(dust, rng, ctx, world, pos, 176, center, 27, 100, 1.5, 2.0);
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
                d.fade_in = 2.5;
            }
            for _ in 0..10 {
                let idx = scatter_x_dust(dust, rng, ctx, world, pos, 176, center, vel_rot, 27, 0, 2.7, 2.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 3.0;
            }
            for _ in 0..10 {
                let idx = scatter_x_dust(dust, rng, ctx, world, pos, 176, center, vel_rot, 31, 0, 1.5, 2.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 3.0;
            }
            gore_census(rng, 2, true, false);
        }
        // `:73140-73148` — the 485: 15× dust-6 defaults noGravity, vel −=
        // oldVel×Next(20,60)×0.01 (one range draw per dust AFTER the spawn).
        485 => {
            for _ in 0..15 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 6, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity -= s.old_vel * (rng.next_range(20, 60) as f32 * 0.01);
            }
        }
        // `:73149-73158` — the 484: 5× dust-78 defaults noGravity, vel −=
        // oldVel/5, scale 0.85.
        484 => {
            for _ in 0..5 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 78, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity -= s.old_vel / 5.0;
                d.scale = 0.85;
            }
        }
        // `:73159-73256` — the 483: the owner-gated 484 homing-spawn burst
        // (`:73162-73221`, the NPC scan + shuffle draws) stays unported
        // state — the spawn lane owns those draws. The dust: 20× dust-78
        // defaults noGravity vel×4; 7× dust-31 alpha 100 scale 1.5 vel×0.9
        // scale 0.9 (gravity ON); 3 PAIRS dust-6 alpha 100 (scale 2.5
        // noGravity vel×3, then scale 1.5 vel×2 GRAVITY ON); ONE gore at
        // pos (no position draws) Next(61,64) + the kicks + the Next(-1,2)
        // X/Y nudges (draws-only). The owner-gated +100 inflate + Damage
        // stay unported state.
        483 => {
            for _ in 0..20 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 78, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 4.0;
            }
            for _ in 0..7 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 31, 0.0, 0.0, 100, Color::default(), 1.5);
                let d = dust.slot_mut(idx);
                d.velocity *= 0.9;
                d.scale = 0.9;
            }
            for _ in 0..3 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 6, 0.0, 0.0, 100, Color::default(), 2.5);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 3.0;
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 6, 0.0, 0.0, 100, Color::default(), 1.5);
                dust.slot_mut(idx).velocity *= 2.0;
            }
            let _ = rng.next_range(61, 64);
            let _ = rng.next_range(10, 31);
            let _ = rng.next_range(-20, 21);
            let _ = rng.next_range(-1, 2);
            let _ = rng.next_range(-1, 2);
        }
        // `:73257-73270` — the 523: 25× dust-256 defaults noGravity,
        // position = (own pos + base.position)/2, vel = Normalize(Next
        // (-100,101) pair)×Next(1,30)×0.1, alpha = the corpse's alpha.
        523 => {
            for _ in 0..25 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 256, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.position = (d.position + s.pos) / 2.0;
                let kick = Vec2::new(rng.next_range(-100, 101) as f32, rng.next_range(-100, 101) as f32);
                d.velocity = safe_normalize(kick) * (rng.next_range(1, 30) as f32 * 0.1);
                d.alpha = s.alpha;
            }
        }
        // `:73292-73311` — the 521: 20× the crystal-spark shape at 254,
        // fadeIn 1.1 / scale 0.35+×0.01 / big 1.2 / vel×2.5. The owner-gated
        // 522 spawn burst (`:73312-73326`) stays the spawn lane's draws.
        521 => {
            crystal_spark_burst(s, dust, rng, ctx, world, 20, 254, 1.1, 0.35, 0.01, 1.2, 0.01, 2.5);
        }
        // `:73338-73371` — the 459/709: scale >= 1 → resize 144×144 centred
        // + counts 7/30/2 gores (else 3/10/0, the ORIGINAL box). 3-or-7×
        // dust-31 alpha 100 scale 1.5 (gravity ON) + 10-or-30× dust-226
        // alpha 0 scale 1.5 noGravity vel×1, each position = Center +
        // (w/2,0).RotatedBy(τ×NextDouble)×NextDouble (the integer w/2);
        // the gore census count 0-or-2. The 709's owner-gated 1110 pull
        // (`:73372-73383`) is unported state, no draws.
        459 | 709 => {
            let big = s.scale >= 1.0;
            let (n31, n226, ngore) = if big { (7, 30, 2) } else { (3, 10, 0) };
            let (pos, w, h, center) = if big {
                let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
                (center - Vec2::splat(72.0), 144, 144, center)
            } else {
                (s.pos, s.w, s.h, Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5))
            };
            let radius = (w / 2) as f32;
            for _ in 0..n31 {
                let idx = dust.new_dust(rng, ctx, world, pos, w, h, 31, 0.0, 0.0, 100, Color::default(), 1.5);
                dust.slot_mut(idx).position = double_angle_ring_pos(rng, center, radius);
            }
            for _ in 0..n226 {
                let idx = dust.new_dust(rng, ctx, world, pos, w, h, 226, 0.0, 0.0, 0, Color::default(), 1.5);
                let d = dust.slot_mut(idx);
                d.position = double_angle_ring_pos(rng, center, radius);
                d.no_gravity = true;
                d.velocity *= 1.0;
            }
            gore_census(rng, ngore, false, false);
        }
        // `:73431-73441` — the 639 moonlord arrow: Next(5,10)× dust-220
        // at Center 0×0 alpha 100 scale 0.5, vel×1.6 then Y−−, position =
        // Lerp(own, Center, 0.5), noGravity. (The moonlord module's census
        // skip of this draw is RECLAIMED here — the hook never consumed
        // it.) The 639's owner-gated 640 ghost spawn stays
        // `moonlord_arrow_kill`'s (no draws).
        639 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let count = rng.next_range(5, 10);
            for _ in 0..count {
                let idx = dust.new_dust(rng, ctx, world, center, 0, 0, 220, 0.0, 0.0, 100, Color::default(), 0.5);
                let d = dust.slot_mut(idx);
                d.velocity *= 1.6;
                d.velocity.y -= 1.0;
                d.position = lerp_vec(d.position, center, 0.5);
                d.no_gravity = true;
            }
        }
        // `:73431-73441` — the 640 moonlord ghost: the same dust arm as
        // the 639 (the C# `type == 639 || type == 640`).
        640 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let count = rng.next_range(5, 10);
            for _ in 0..count {
                let idx = dust.new_dust(rng, ctx, world, center, 0, 0, 220, 0.0, 0.0, 100, Color::default(), 0.5);
                let d = dust.slot_mut(idx);
                d.velocity *= 1.6;
                d.velocity.y -= 1.0;
                d.position = lerp_vec(d.position, center, 0.5);
                d.no_gravity = true;
            }
        }
        // `:73492-73502` — the 682: 22× dust-272 at Center 0×0 alpha 0
        // scale 0.5, vel×1.6 Y−−, position = Lerp(own, Center, 0.5) —
        // GRAVITY ON (no noGravity write, unlike the 639).
        682 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            for _ in 0..22 {
                let idx = dust.new_dust(rng, ctx, world, center, 0, 0, 272, 0.0, 0.0, 0, Color::default(), 0.5);
                let d = dust.slot_mut(idx);
                d.velocity *= 1.6;
                d.velocity.y -= 1.0;
                d.position = lerp_vec(d.position, center, 0.5);
            }
        }
        // `:73573-73582` — the 449: Next(3,7)× dust-228 at Center−vel/2
        // 0×0 alpha 100 scale 2.1, vel×2 noGravity.
        449 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let pos = center - s.vel / 2.0;
            let count = rng.next_range(3, 7);
            for _ in 0..count {
                let idx = dust.new_dust(rng, ctx, world, pos, 0, 0, 228, 0.0, 0.0, 100, Color::default(), 2.1);
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
        }
        // `:73603-73631` — the 448: resize 112×112 centred; 4× dust-31
        // alpha 100 scale 1.5 (NO scatter, gravity ON); 40 PAIRS dust-228
        // (alpha 0 scale 2.5 noGravity vel×3, then alpha 100 scale 1.5
        // vel×2 noGravity); ONE gore census.
        448 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let pos = center - Vec2::splat(56.0);
            for _ in 0..4 {
                dust.new_dust(rng, ctx, world, pos, 112, 112, 31, 0.0, 0.0, 100, Color::default(), 1.5);
            }
            for _ in 0..40 {
                let idx = dust.new_dust(rng, ctx, world, pos, 112, 112, 228, 0.0, 0.0, 0, Color::default(), 2.5);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 3.0;
                let idx = dust.new_dust(rng, ctx, world, pos, 112, 112, 228, 0.0, 0.0, 100, Color::default(), 1.5);
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
            gore_census(rng, 1, false, false);
        }
        // `:73646-73674` — the 616: resize 80×80 centred; 4× dust-31 alpha
        // 100 scale 1.5 (gravity ON); 40 PAIRS dust-229 BOTH alpha 200
        // (scale 2.5 noGravity vel×2, then scale 1.5 vel×1.2 noGravity);
        // ONE gore census.
        616 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let pos = center - Vec2::splat(40.0);
            for _ in 0..4 {
                dust.new_dust(rng, ctx, world, pos, 80, 80, 31, 0.0, 0.0, 100, Color::default(), 1.5);
            }
            for _ in 0..40 {
                let idx = dust.new_dust(rng, ctx, world, pos, 80, 80, 229, 0.0, 0.0, 200, Color::default(), 2.5);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 2.0;
                let idx = dust.new_dust(rng, ctx, world, pos, 80, 80, 229, 0.0, 0.0, 200, Color::default(), 1.5);
                let d = dust.slot_mut(idx);
                d.velocity *= 1.2;
                d.no_gravity = true;
            }
            gore_census(rng, 1, false, false);
        }
        // `:73729-73737` — the 836: 3× dust-31 alpha 50 White scale 0.8,
        // vel×0.5 noGravity.
        836 => {
            for _ in 0..3 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 31, 0.0, 0.0, 50, Color::WHITE, 0.8);
                let d = dust.slot_mut(idx);
                d.velocity *= 0.5;
                d.no_gravity = true;
            }
        }
        // `:73738-73744` — the 1092: 6× dust-269 Direct at base.Left
        // (pos.X, pos.Y+h/2) box w×4 alpha 0 White scale 1.2 noLight.
        1092 => {
            let left = Vec2::new(s.pos.x, s.pos.y + s.h as f32 / 2.0);
            for _ in 0..6 {
                let idx = dust.new_dust_direct(rng, ctx, world, left, s.w, 4, 269, 0.0, 0.0, 0, Color::WHITE, 1.2);
                dust.slot_mut(idx).no_light = true;
            }
        }
        // `:73819-73861` — the 424-426: the box recentres on the old centre
        // at (int)(128×scale); 8× dust-31 alpha 100 scale 1.5 (gravity ON);
        // 32 PAIRS dust-6 alpha 100 (scale 2.5 noGravity vel×3, then scale
        // 1.5 vel×2 noGravity); 2 gores census; the owner-gated
        // localAI[1]=-1 + Damage are unported state (no draws); THEN 5×
        // SelectRandom(6,259,158) dust speed (2.5×direction, −2.5) alpha
        // 200 vel×2.4 scale += NextFloat (the type argument draws BEFORE
        // the spawn internals).
        424..=426 => {
            let size = (128.0 * s.scale) as i32;
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let pos = center - Vec2::splat((size / 2) as f32);
            for _ in 0..8 {
                dust.new_dust(rng, ctx, world, pos, size, size, 31, 0.0, 0.0, 100, Color::default(), 1.5);
            }
            for _ in 0..32 {
                let idx = dust.new_dust(rng, ctx, world, pos, size, size, 6, 0.0, 0.0, 100, Color::default(), 2.5);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 3.0;
                let idx = dust.new_dust(rng, ctx, world, pos, size, size, 6, 0.0, 0.0, 100, Color::default(), 1.5);
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
            gore_census(rng, 2, false, false);
            for _ in 0..5 {
                let dust_type = rng.next_from_slice(&[6, 259, 158]);
                let idx = dust.new_dust(rng, ctx, world, pos, size, size, dust_type, 2.5 * s.direction as f32, -2.5, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.alpha = 200;
                d.velocity *= 2.4;
                d.scale += rng.next_float();
            }
        }
        // `:73862-73920` — the 1037/1049 cursed-flame death: the double
        // resize (192×scale then 64/112), 6 dust-31 + 3 circular-scatter
        // gores + the !wet-gated 24 pairs + 4 SelectRandom fan (see
        // `cursed_flame_death`).
        1037 | 1049 => cursed_flame_death(s, dust, rng, ctx, world),
        // `:73921-73941` — the 399: 5× dust-12 Red at Center−(10,10) box
        // 20×20 defaults; 10× dust-31 alpha 100 scale 1.5 vel×1.4; 20
        // PAIRS dust-6 alpha 100 (scale 2.5 noGravity vel×5, then scale
        // 1.5 vel×3 GRAVITY ON). The owner-gated 6-shard spawn burst
        // (`:73942-73950`) stays the spawn lane's draws.
        399 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let pos = center - Vec2::splat(10.0);
            for _ in 0..5 {
                dust.new_dust(rng, ctx, world, pos, 20, 20, 12, 0.0, 0.0, 0, Color::new(255, 0, 0, 255), 1.0);
            }
            for _ in 0..10 {
                let idx = dust.new_dust(rng, ctx, world, pos, 20, 20, 31, 0.0, 0.0, 100, Color::default(), 1.5);
                dust.slot_mut(idx).velocity *= 1.4;
            }
            for _ in 0..20 {
                let idx = dust.new_dust(rng, ctx, world, pos, 20, 20, 6, 0.0, 0.0, 100, Color::default(), 2.5);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 5.0;
                let idx = dust.new_dust(rng, ctx, world, pos, 20, 20, 6, 0.0, 0.0, 100, Color::default(), 1.5);
                dust.slot_mut(idx).velocity *= 3.0;
            }
        }
        // `:74029-74062` — the 358: the smoke puff, 50× dust-211 alpha 75
        // scale 1.2, small 0.6, vel += newVel.
        358 => smoke_puff(s, dust, rng, ctx, world, 50, 211, 75, Color::default(), 1.2, 0.6, 1.0, false),
        // `:74063-74098` — the 406: the smoke puff, 40× dust-4 alpha 175
        // (0,80,255,100) scale 1.6, small 0.6.
        406 => smoke_puff(s, dust, rng, ctx, world, 40, 4, 175, Color::new(0, 80, 255, 100), 1.6, 0.6, 1.0, false),
        // `:74099-74147` — the 1042: the smoke puff, 50× dust-324 alpha 75
        // scale 1.25, small 0.8, vel += newVel×0.8, + the Next(6) colour.
        1042 => smoke_puff(s, dust, rng, ctx, world, 50, 324, 75, Color::default(), 1.25, 0.8, 0.8, true),
        // `:74182-74203` — the 323: 20× dust-7 defaults; Next(2) gate 0 →
        // noGravity scale 1.3 vel×1.5 −= oldVel×0.5 vel×1.5; else vel×0.75
        // −= oldVel×0.25 scale 0.8 (gravity ON).
        323 => {
            for _ in 0..20 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 7, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                if rng.next_max(2) == 0 {
                    d.no_gravity = true;
                    d.scale = 1.3;
                    d.velocity *= 1.5;
                    d.velocity -= s.old_vel * 0.5;
                    d.velocity *= 1.5;
                } else {
                    d.velocity *= 0.75;
                    d.velocity -= s.old_vel * 0.25;
                    d.scale = 0.8;
                }
            }
        }
        // `:74204-74235` — the 589: colour Red / ai1==1 Green / ==2 Purple
        // / ==3 Gold / ==4 White, A=100; 30× dust-11 colour defaults vel×=
        // 1+NextFloat; the first 10 noGravity vel×0.5.
        589 => {
            // The C# compares ai[1] == 1f etc. EXACTLY (float equality).
            let mut color = if s.ai1 == 1.0 {
                Color::new(0, 255, 0, 255)
            } else if s.ai1 == 2.0 {
                Color::new(128, 0, 128, 255)
            } else if s.ai1 == 3.0 {
                Color::new(255, 215, 0, 255)
            } else if s.ai1 == 4.0 {
                Color::WHITE
            } else {
                Color::new(255, 0, 0, 255)
            };
            color.a = 100;
            for i in 0..30 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 11, 0.0, 0.0, 0, color, 1.0);
                let d = dust.slot_mut(idx);
                d.velocity *= 1.0 + rng.next_float();
                if i < 10 {
                    d.no_gravity = true;
                    d.velocity *= 0.5;
                }
            }
        }
        // `:74283-74302` — the 311/316: 5× dust-189/195 defaults scale
        // 0.85 noGravity vel += baseVel×0.5.
        311 | 316 => {
            let dust_type = if s.kind == 311 { 189 } else { 195 };
            for _ in 0..5 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, dust_type, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.scale = 0.85;
                d.no_gravity = true;
                d.velocity += s.vel * 0.5;
            }
        }
        // `:74311-74325` — the 1097: 6× dust-6 defaults. The owner-gated
        // ai[1]==1 eight-child spawn burst (`:74318-74324`) stays the
        // spawn lane's draws.
        1097 => {
            for _ in 0..6 {
                dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 6, 0.0, 0.0, 0, Color::default(), 1.0);
            }
        }
        // `:74348-74383` — the 270/837: the 585 pair shape — 20 pairs
        // dust-26 alpha 100 noGravity vel×1.2 scale 1.3 −= oldVel×0.3,
        // then dust-5 (270, scale 1.5) / dust-6 (837, scale 2) at
        // pos+(4,4) box (w−8)×(h−8) alpha 100 noGravity vel×3. The 837's
        // penetrate!=maxPenetrate sound gate draws nothing.
        270 | 837 => {
            let (second_type, second_scale) = if s.kind == 270 { (5, 1.5) } else { (6, 2.0) };
            for _ in 0..20 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 26, 0.0, 0.0, 100, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 1.2;
                d.scale = 1.3;
                d.velocity -= s.old_vel * 0.3;
                let idx = dust.new_dust(rng, ctx, world, s.pos + Vec2::splat(4.0), s.w - 8, s.h - 8, second_type, 0.0, 0.0, 100, Color::default(), second_scale);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 3.0;
            }
        }
        // `:74450-74474` — the 229: the owner-gated 32-shard spawn burst
        // (`:74453-74466`) stays the spawn lane's draws; 25× dust-157
        // defaults noGravity vel×1.5 scale 1.5.
        229 => {
            for _ in 0..25 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 157, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 1.5;
                d.scale = 1.5;
            }
        }
        // `:74565-74589` — the 178: 85 confetti dust + 40 gore rows.
        178 => confetti_burst(s, dust, rng, ctx, world, 85, 40),
        // `:74590-74614` — the 289: 30 confetti dust + 15 gore rows.
        289 => confetti_burst(s, dust, rng, ctx, world, 30, 15),
        // `:74615-74678` — the 475/505/506: the trail walk at dust 3 /
        // 239 / 30.
        475 => trail_walk_burst(s, dust, rng, ctx, world, 3),
        505 => trail_walk_burst(s, dust, rng, ctx, world, 239),
        506 => trail_walk_burst(s, dust, rng, ctx, world, 30),
        // `:74679-74733` — the 171: the trail walk at dust 129.
        171 => trail_walk_burst(s, dust, rng, ctx, world, 129),
        // `:74734-74751` — the 117: 10× dust-26 defaults. The owner-gated
        // 2-shard spawn burst (`:74741-74750`) stays the spawn lane's.
        117 => {
            for _ in 0..10 {
                dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 26, 0.0, 0.0, 0, Color::default(), 1.0);
            }
        }
        // `:74760-74769` — the 166: 10× dust-76 defaults noGravity,
        // vel −= oldVel×0.25.
        166 => {
            for _ in 0..10 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 76, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity -= s.old_vel * 0.25;
            }
        }
        // `:74841-74949` — the 99-family default arm (99/1013/727/1014/
        // 1021/1048): the dust type is the kind switch (1013→243, 727→40,
        // 1014→12, 1021→323, 1048→322, else 1); 30× that dust, a Next(2)
        // gate → scale×1.4. The per-dust `base.velocity *= 1.9` is a dead-
        // corpse mutation (the dust spawns at speed 0, so it feeds nothing
        // observable — recorded). The 1014/1057 item drops + the 1048
        // dirt delegate draw nothing here (the delegate is unported state).
        99 | 1013 | 727 | 1014 | 1021 | 1048 => {
            let dust_type = match s.kind {
                1013 => 243,
                727 => 40,
                1014 => 12,
                1021 => 323,
                1048 => 322,
                _ => 1,
            };
            for _ in 0..30 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, dust_type, 0.0, 0.0, 0, Color::default(), 1.0);
                if rng.next_max(2) == 0 {
                    dust.slot_mut(idx).scale *= 1.4;
                }
            }
        }
        // `:74921-74927` — the 1054: 20× dust-30 speed (0,−1) defaults.
        1054 => {
            for _ in 0..20 {
                dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 30, 0.0, -1.0, 0, Color::default(), 1.0);
            }
        }
        // `:74928-74937` — the 1055: 50× dust-192 at pos+(4,4) box (w−8)×
        // (h−8) speed vel×0.5 alpha 100 (100,100,100,100), scale×1.5
        // vel×2 noGravity.
        1055 => {
            for _ in 0..50 {
                let idx = dust.new_dust(rng, ctx, world, s.pos + Vec2::splat(4.0), s.w - 8, s.h - 8, 192, s.vel.x * 0.5, s.vel.y * 0.5, 100, Color::new(100, 100, 100, 100), 1.0);
                let d = dust.slot_mut(idx);
                d.scale *= 1.5;
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
        }
        // `:74970-74982` — the 1005: 10× dust-1 defaults, a Next(2) gate →
        // scale×1.1; the per-dust base.velocity×1.6 is a dead-corpse
        // mutation (recorded, the 99-family note).
        1005 => {
            for _ in 0..10 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 1, 0.0, 0.0, 0, Color::default(), 1.0);
                if rng.next_max(2) == 0 {
                    dust.slot_mut(idx).scale *= 1.1;
                }
            }
        }
        // `:75037-75066` — the 772 geode: 6× dust-1 defaults (the dead
        // base.velocity×1.9 mutation); then 20 + 7 dust-NextFromList(86,
        // 87,88,89,90) Direct alpha 0 scale 0.7 noGravity vel×2.9 Y×0.8
        // fadeIn 1.1 (the last 7 also noLight). The owner-gated
        // DropGeodeGems loot (`:78561-78572`) draws FOUR (the two item
        // picks Next(7)/Next(6) + the two stacks Next(3,7)) — consumed
        // here as the loot census (the bodies are the loot lane's).
        772 => {
            for _ in 0..6 {
                dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 1, 0.0, 0.0, 0, Color::default(), 1.0);
            }
            for i in 0..27 {
                let dust_type = rng.next_from_slice(&[86, 87, 88, 89, 90]);
                let idx = dust.new_dust_direct(rng, ctx, world, s.pos, s.w, s.h, dust_type, 0.0, 0.0, 0, Color::default(), 0.7);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 2.9;
                d.velocity.y *= 0.8;
                d.fade_in = 1.1;
                if i >= 20 {
                    d.no_light = true;
                }
            }
            let _ = rng.next_max(7); // the first gem pick
            let _ = rng.next_range(3, 7); // the first stack
            let _ = rng.next_max(6); // the second gem pick (one removed)
            let _ = rng.next_range(3, 7); // the second stack
        }
        // `:75067-75079` — the 763: 4× dust-40 defaults, a Next(2) gate →
        // scale×0.8, noGravity; the dead base.velocity×1.9 mutation.
        763 => {
            for _ in 0..4 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 40, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                if rng.next_max(2) == 0 {
                    d.scale *= 0.8;
                }
                d.no_gravity = true;
            }
        }
        // `:75159-75178` — the 723: 10× dust-88 Direct speed vel×0.1 alpha
        // 150 scale 1.2 noGravity X×2; 3 gore-Direct rows (the
        // SelectRandom(16,17,17) type + the kicks, vel×2 body skipped);
        // the owner-gated ai[2]==1 item request draws nothing.
        723 => {
            for _ in 0..10 {
                let idx = dust.new_dust_direct(rng, ctx, world, s.pos, s.w, s.h, 88, s.vel.x * 0.1, s.vel.y * 0.1, 150, Color::default(), 1.2);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity.x *= 2.0;
            }
            for _ in 0..3 {
                let _ = rng.next_from_slice(&[16, 17, 17]);
                let _ = rng.next_range(10, 31);
                let _ = rng.next_range(-20, 21);
            }
        }
        // `:75179-75210` — the 724/725/726/907/1133: 10× dust-58 (1133 →
        // 57) speed vel×0.1 alpha 150 scale 1.2 (907 → vel×0.75); 3 gore
        // rows Next(16,18) type + the kicks (907 → vel×0.5 body skipped).
        724 | 725 | 726 | 907 | 1133 => {
            let dust_type = if s.kind == 1133 { 57 } else { 58 };
            for _ in 0..10 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, dust_type, s.vel.x * 0.1, s.vel.y * 0.1, 150, Color::default(), 1.2);
                if s.kind == 907 {
                    dust.slot_mut(idx).velocity *= 0.75;
                }
            }
            for _ in 0..3 {
                let _ = rng.next_range(16, 18);
                let _ = rng.next_range(10, 31);
                let _ = rng.next_range(-20, 21);
            }
        }
        // `:74877-74920` — the 1053 lava boulder: resize 64×64 centred;
        // 25× dust-31 alpha 100 scale 2 vel×1.4 (gravity ON); 40 PAIRS
        // dust-6 alpha 100 (scale 3 noGravity vel×5, then scale 2 vel×3
        // GRAVITY ON); 4 gores at Center−(24,24) (the Next(61,64) type +
        // the kicks, the ±1.5 nudges are body writes); the 31-resize +
        // LavaBoulderExplosion (PlaceLiquid, no draws) stay unported.
        1053 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let pos = center - Vec2::splat(32.0);
            for _ in 0..25 {
                let idx = dust.new_dust(rng, ctx, world, pos, 64, 64, 31, 0.0, 0.0, 100, Color::default(), 2.0);
                dust.slot_mut(idx).velocity *= 1.4;
            }
            for _ in 0..40 {
                let idx = dust.new_dust(rng, ctx, world, pos, 64, 64, 6, 0.0, 0.0, 100, Color::default(), 3.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 5.0;
                let idx = dust.new_dust(rng, ctx, world, pos, 64, 64, 6, 0.0, 0.0, 100, Color::default(), 2.0);
                dust.slot_mut(idx).velocity *= 3.0;
            }
            for _ in 0..4 {
                let _ = rng.next_range(61, 64);
                let _ = rng.next_range(10, 31);
                let _ = rng.next_range(-20, 21);
            }
        }
        // `:70749-70757` — the 1098/1123: 50× dust-306 Direct alpha 0
        // (100,210,255,127) defaults, vel×2 X×0.5.
        1098 | 1123 => {
            for _ in 0..50 {
                let idx = dust.new_dust_direct(rng, ctx, world, s.pos, s.w, s.h, 306, 0.0, 0.0, 0, Color::new(100, 210, 255, 127), 1.0);
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.velocity.x *= 0.5;
            }
        }
        // `:70855-70901` — the 1020/1105: 15× dust-31 Direct alpha 100
        // Transparent scale 0.8 fadeIn 0 vel×0.5; 5 PAIRS dust-228 Direct
        // alpha 100 Transparent (scale 2.5 noGravity vel×2.5, then scale
        // 1.1 vel×2 noGravity); 3× dust-226 Direct alpha 100 Transparent
        // scale 1.1 vel×2 noGravity; then a 2×2 corner loop, each a
        // Next(5)==0 gate → one gore-Direct (Next(61,64) type + kicks,
        // bodies skipped). The 1020 achievement/sound-stop draw nothing.
        1020 | 1105 => {
            for _ in 0..15 {
                let idx = dust.new_dust_direct(rng, ctx, world, s.pos, s.w, s.h, 31, 0.0, 0.0, 100, Color::TRANSPARENT, 0.8);
                let d = dust.slot_mut(idx);
                d.fade_in = 0.0;
                d.velocity *= 0.5;
            }
            for _ in 0..5 {
                let idx = dust.new_dust_direct(rng, ctx, world, s.pos, s.w, s.h, 228, 0.0, 0.0, 100, Color::TRANSPARENT, 2.5);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 2.5;
                let idx = dust.new_dust_direct(rng, ctx, world, s.pos, s.w, s.h, 228, 0.0, 0.0, 100, Color::TRANSPARENT, 1.1);
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
            for _ in 0..3 {
                let idx = dust.new_dust_direct(rng, ctx, world, s.pos, s.w, s.h, 226, 0.0, 0.0, 100, Color::TRANSPARENT, 1.1);
                let d = dust.slot_mut(idx);
                d.velocity *= 2.0;
                d.no_gravity = true;
            }
            for _ in 0..4 {
                if rng.next_max(5) == 0 {
                    let _ = rng.next_range(61, 64);
                    let _ = rng.next_range(10, 31);
                    let _ = rng.next_range(-20, 21);
                }
            }
        }
        // `:70902-70923` — the 1085 broken mirror: oldVelocity×0.2 THEN
        // position −= oldVelocity (the dust reads the shifted box); 10×
        // dust-330 defaults vel −= oldVelocity; 4 gores 1394-1397 at
        // Center vel −oldVelocity (the kicks census, fixed types no draw);
        // the bad-luck buff is unported state.
        1085 => {
            let old_vel = s.old_vel * 0.2;
            let pos = s.pos - old_vel;
            for _ in 0..10 {
                let idx = dust.new_dust(rng, ctx, world, pos, s.w, s.h, 330, 0.0, 0.0, 0, Color::default(), 1.0);
                dust.slot_mut(idx).velocity -= old_vel;
            }
            for _ in 0..4 {
                let _ = rng.next_range(10, 31);
                let _ = rng.next_range(-20, 21);
            }
        }
        // `:70926-70954` — the 1080: 30× dust-329 at Center box 1×1
        // defaults, vel×0.8 Y×0.2 Y −= NextFloat×3; a Next(2) gate 0 →
        // noGravity scale NextFloat×0.6+0.6, else scale NextFloat×0.5+0.5.
        // The buff + active=false/return are unported state.
        1080 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            for _ in 0..30 {
                let idx = dust.new_dust(rng, ctx, world, center, 1, 1, 329, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.velocity *= 0.8;
                d.velocity.y *= 0.2;
                d.velocity.y -= rng.next_float() * 3.0;
                if rng.next_max(2) == 0 {
                    d.no_gravity = true;
                    d.scale = rng.next_float() * 0.6 + 0.6;
                } else {
                    d.scale = rng.next_float() * 0.5 + 0.5;
                }
            }
        }
        // `:71043-71051` — the 1038: 4× dust-166 Perfect at Center +
        // NextVector2Circular(10,10)×scale, vel Zero, scale 1.5+NextFloat×
        // 0.25 noGravity.
        1038 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            for _ in 0..4 {
                let pos = center + next_vector2_circular(rng, 10.0, 10.0) * s.scale;
                let idx = dust.new_dust_perfect(rng, ctx, world, pos, 166, Some(Vec2::ZERO), 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.scale = 1.5 + rng.next_float() * 0.25;
                d.no_gravity = true;
            }
        }
        // `:71068-71076` — the 1036: 3× dust-5 Perfect at Center +
        // NextVector2Circular(4,4)×scale, vel Zero, scale 1.5 noGravity.
        1036 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            for _ in 0..3 {
                let pos = center + next_vector2_circular(rng, 4.0, 4.0) * s.scale;
                let idx = dust.new_dust_perfect(rng, ctx, world, pos, 5, Some(Vec2::ZERO), 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.scale = 1.5;
                d.no_gravity = true;
            }
        }
        // `:71077-71087` — the 756: a f32-step ring (num += 0.025, 40
        // iters) of dust-5 Perfect at Center + Circular(16,16)×scale +
        // vel.SafeNormalize(UnitY)×num×200×scale, vel = Circular(3,3),
        // Y −= 0.3, += baseVel×0.2, scale 1 alpha 100.
        756 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let dir = if s.vel == Vec2::ZERO { Vec2::Y } else { safe_normalize(s.vel) };
            let mut num = 0.0f32;
            while num < 1.0 {
                let pos = center + next_vector2_circular(rng, 16.0, 16.0) * s.scale + dir * num * 200.0 * s.scale;
                let vel = next_vector2_circular(rng, 3.0, 3.0);
                let idx = dust.new_dust_perfect(rng, ctx, world, pos, 5, Some(vel), 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.velocity.y += -0.3;
                d.velocity += s.vel * 0.2;
                d.scale = 1.0;
                d.alpha = 100;
                num += 0.025;
            }
        }
        // `:71088-71098` — the 961: the 756 shape at dust-16, num += 0.25
        // (4 iters).
        961 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let dir = if s.vel == Vec2::ZERO { Vec2::Y } else { safe_normalize(s.vel) };
            let mut num = 0.0f32;
            while num < 1.0 {
                let pos = center + next_vector2_circular(rng, 16.0, 16.0) * s.scale + dir * num * 200.0 * s.scale;
                let vel = next_vector2_circular(rng, 3.0, 3.0);
                let idx = dust.new_dust_perfect(rng, ctx, world, pos, 16, Some(vel), 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.velocity.y += -0.3;
                d.velocity += s.vel * 0.2;
                d.scale = 1.0;
                d.alpha = 100;
                num += 0.25;
            }
        }
        // `:75425-75485` — the gem bolts (121-126/597): the dust type is
        // kind−121+86 (597 → 262); dir = vel.SafeNormalize(Zero), zeroed
        // when |vel| < 1. 15× that dust speed dir alpha 50 scale 1.2
        // noGravity scale×1.25 vel×2.2 += dir×1.5 vel×= NextFloat. The
        // AoeExplosion feature (ai[0] bit 3) adds 30× more (noGravity =
        // Next(5)!=0, scale×=1+NextFloat, vel×4.2, the gravity ones
        // scale×0.6 fadeIn 0 noLight, vel×=NextFloat). The AoE DAMAGE half
        // (the 160/22 resizes + Damage) is `gem_bolt_aoe`'s — the dust
        // here reads the ORIGINAL box (the C# dust precedes the resize).
        121..=126 | 597 => {
            let dust_type = if s.kind == 597 { 262 } else { s.kind as i32 - 121 + 86 };
            let mut dir = safe_normalize(s.vel);
            if s.vel.length() < 1.0 {
                dir = Vec2::ZERO;
            }
            for _ in 0..15 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, dust_type, dir.x, dir.y, 50, Color::default(), 1.2);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.scale *= 1.25;
                d.velocity *= 2.2;
                d.velocity += dir * 1.5;
                d.velocity *= rng.next_float();
            }
            if crate::projectile::GemStaffFeatures::from_ai(s.ai0).aoe_explosion() {
                for _ in 0..30 {
                    let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, dust_type, dir.x, dir.y, 50, Color::default(), 1.2);
                    let d = dust.slot_mut(idx);
                    d.no_gravity = rng.next_max(5) != 0;
                    d.scale *= 1.0 + rng.next_float();
                    d.velocity *= 4.2;
                    if !d.no_gravity {
                        d.scale *= 0.6;
                        d.fade_in = 0.0;
                        d.no_light = true;
                    }
                    d.velocity *= rng.next_float();
                }
            }
        }
        // `:75500-75512` — the 1115: num690 = Remap(ai[0],0,60,0,1), count
        // = 10+(int)(20×num690); count× dust-197 defaults vel×=1+num690×2
        // += baseVel×NextFloat×num690 noGravity.
        1115 => {
            let t = remap_f32(s.ai0, 0.0, 60.0, 0.0, 1.0);
            let count = 10 + (20.0 * t) as i32;
            for _ in 0..count {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 197, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.velocity *= 1.0 + t * 2.0;
                d.velocity += s.vel * rng.next_float() * t;
                d.no_gravity = true;
            }
        }
        // `:75522-75532` — the 379/377: 5× dust-171 alpha 100 defaults,
        // scale = Next(1,10)×0.1 noGravity fadeIn 1.5 vel×0.75.
        379 | 377 => {
            for _ in 0..5 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 171, 0.0, 0.0, 100, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.scale = rng.next_range(1, 10) as f32 * 0.1;
                d.no_gravity = true;
                d.fade_in = 1.5;
                d.velocity *= 0.75;
            }
        }
        // `:75749-75803` — the 281 bomb/dynamite: gated ai[0] != −3; TWO
        // leading gores 76/77 (each a Next(−20,21) X/Y velocity pair + the
        // kicks, fixed types); 20× dust-31 alpha 100 scale 1.5 vel×1.4;
        // 10 PAIRS dust-6 alpha 100 (scale 2.5 noGravity vel×5, then scale
        // 1.5 vel×3 GRAVITY); 4 corner gores Next(61,64) + kicks. The
        // damage>0 128-resize + Damage stay unported state.
        281 => {
            if s.ai0 != -3.0 {
                for _ in 0..2 {
                    let _ = rng.next_range(-20, 21);
                    let _ = rng.next_range(-20, 21);
                    let _ = rng.next_range(10, 31);
                    let _ = rng.next_range(-20, 21);
                }
                for _ in 0..20 {
                    let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 31, 0.0, 0.0, 100, Color::default(), 1.5);
                    dust.slot_mut(idx).velocity *= 1.4;
                }
                for _ in 0..10 {
                    let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 6, 0.0, 0.0, 100, Color::default(), 2.5);
                    let d = dust.slot_mut(idx);
                    d.no_gravity = true;
                    d.velocity *= 5.0;
                    let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 6, 0.0, 0.0, 100, Color::default(), 1.5);
                    dust.slot_mut(idx).velocity *= 3.0;
                }
                for _ in 0..4 {
                    let _ = rng.next_range(61, 64);
                    let _ = rng.next_range(10, 31);
                    let _ = rng.next_range(-20, 21);
                }
            }
        }
        // `:75804-75854` — the 162: resize 64×64 centred BEFORE the dust;
        // 20× dust-31 alpha 100 scale 1.5 vel×2.5; 50× dust-6 alpha 100
        // scale 2.5 noGravity vel×= Next(50,151)×0.1; 10× dust-6 alpha 100
        // scale 1.5 vel×3 Y−− (GRAVITY); 4 corner gores. The 144-resize +
        // Damage AFTER stay unported.
        162 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let pos = center - Vec2::splat(32.0);
            for _ in 0..20 {
                let idx = dust.new_dust(rng, ctx, world, pos, 64, 64, 31, 0.0, 0.0, 100, Color::default(), 1.5);
                dust.slot_mut(idx).velocity *= 2.5;
            }
            for _ in 0..50 {
                let idx = dust.new_dust(rng, ctx, world, pos, 64, 64, 6, 0.0, 0.0, 100, Color::default(), 2.5);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= rng.next_range(50, 151) as f32 * 0.1;
            }
            for _ in 0..10 {
                let idx = dust.new_dust(rng, ctx, world, pos, 64, 64, 6, 0.0, 0.0, 100, Color::default(), 1.5);
                let d = dust.slot_mut(idx);
                d.velocity *= 3.0;
                d.velocity.y -= 1.0;
            }
            for _ in 0..4 {
                let _ = rng.next_range(61, 64);
                let _ = rng.next_range(10, 31);
                let _ = rng.next_range(-20, 21);
            }
        }
        // `:75855-75894` — the 240: the 281 dust shape on the ORIGINAL box
        // (20× dust-31 vel×1.4 + 10 pairs dust-6 + 4 corner gores); the
        // 96-resize + Damage AFTER stay unported.
        240 => {
            for _ in 0..20 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 31, 0.0, 0.0, 100, Color::default(), 1.5);
                dust.slot_mut(idx).velocity *= 1.4;
            }
            for _ in 0..10 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 6, 0.0, 0.0, 100, Color::default(), 2.5);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 5.0;
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 6, 0.0, 0.0, 100, Color::default(), 1.5);
                dust.slot_mut(idx).velocity *= 3.0;
            }
            for _ in 0..4 {
                let _ = rng.next_range(61, 64);
                let _ = rng.next_range(10, 31);
                let _ = rng.next_range(-20, 21);
            }
        }
        // `:75899-75910` — the 282: 10× dust-171 alpha 100 scale Next(1,10)
        // ×0.1 noGravity fadeIn 1.5 vel×0.75 (the 379 shape at count 10).
        282 => {
            for _ in 0..10 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, 171, 0.0, 0.0, 100, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.scale = rng.next_range(1, 10) as f32 * 0.1;
                d.no_gravity = true;
                d.fade_in = 1.5;
                d.velocity *= 0.75;
            }
        }
        // `:76067-76082` — the 660: Next(4,10)× dust-180 at Center 0×0
        // alpha 100 defaults, vel×1.6 Y−− += −baseVel×(NextFloat×2−1)×0.5,
        // scale 2 fadeIn 0.5 noGravity. HitTiles draws nothing here.
        660 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let count = rng.next_range(4, 10);
            for _ in 0..count {
                let idx = dust.new_dust(rng, ctx, world, center, 0, 0, 180, 0.0, 0.0, 100, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.velocity *= 1.6;
                d.velocity.y -= 1.0;
                d.velocity += -s.vel * (rng.next_float() * 2.0 - 1.0) * 0.5;
                d.scale = 2.0;
                d.fade_in = 0.5;
                d.no_gravity = true;
            }
        }
        // `:76119-76133` — the 638 moonlord bullet: Next(2,5)× dust-229 at
        // Center 0×0 alpha 100 defaults, vel×1.6 Y−−, position −= (4,4)
        // then Lerp(own, Center, 0.5), noGravity. (The moonlord module's
        // census skip of this draw is RECLAIMED here.)
        638 => {
            let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
            let count = rng.next_range(2, 5);
            for _ in 0..count {
                let idx = dust.new_dust(rng, ctx, world, center, 0, 0, 229, 0.0, 0.0, 100, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.velocity *= 1.6;
                d.velocity.y -= 1.0;
                d.position -= Vec2::splat(4.0);
                d.position = lerp_vec(d.position, center, 0.5);
                d.no_gravity = true;
            }
        }
        // `:76187-76204` — the 411-414: dust type 9 (412/414 → 11, 413 →
        // 19); 5× that dust speed (0, vel.Y/2) defaults noGravity vel −=
        // baseVel×0.5.
        411..=414 => {
            let dust_type = match s.kind {
                412 | 414 => 11,
                413 => 19,
                _ => 9,
            };
            for _ in 0..5 {
                let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, dust_type, 0.0, s.vel.y / 2.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity -= s.vel * 0.5;
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

/// XNA `Color.Crimson` = (220, 20, 60, 255).
const CRIMSON: Color = Color { r: 220, g: 20, b: 60, a: 255 };

/// `Color.Crimson * 0.5f` — the XNA byte multiply truncates (`ui::color::mul`):
/// (110, 10, 30, 127). The 634/635/661/617 mid-loop dust colour.
fn crimson_half() -> Color {
    let m = crate::ui::color::mul([CRIMSON.r, CRIMSON.g, CRIMSON.b, CRIMSON.a], 0.5);
    Color::new(m[0], m[1], m[2], m[3])
}

/// The resize-centred explosion's Y-scatter dust (`:72241` et al.): spawn at
/// `pos` box `size`, then position = Center + UnitY.RotatedByRandom(π) ×
/// NextDouble × size/div. Returns the slot index for the caller's writes.
fn scatter_y_dust(
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
    pos: Vec2,
    size: i32,
    center: Vec2,
    dust_type: i32,
    alpha: i32,
    scale: f32,
    div: f32,
) -> usize {
    let idx = dust.new_dust(rng, ctx, world, pos, size, size, dust_type, 0.0, 0.0, alpha, Color::default(), scale);
    let scatter = rotated_by_random(rng, Vec2::Y, std::f32::consts::PI);
    dust.slot_mut(idx).position = center + scatter * rng.next_double() as f32 * size as f32 / div;
    idx
}

/// The X-scatter variant (`:72263` et al.): UnitX.RotatedByRandom(π)
/// .RotatedBy(velRot) × size/div, no NextDouble radius.
fn scatter_x_dust(
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
    pos: Vec2,
    size: i32,
    center: Vec2,
    vel_rot: f32,
    dust_type: i32,
    alpha: i32,
    scale: f32,
    div: f32,
) -> usize {
    let idx = dust.new_dust(rng, ctx, world, pos, size, size, dust_type, 0.0, 0.0, alpha, Color::default(), scale);
    let scatter = rotated_by_random(rng, Vec2::X, std::f32::consts::PI);
    dust.slot_mut(idx).position = center + rotated_by(scatter, vel_rot) * size as f32 / div;
    idx
}

/// `Kill()`'s trailing gore loop, draws-only census (`:72120-72123` /
/// `:72319-72325` / `:71890-71899`): per gore the position Next(100) X/Y
/// pair, the Next(61,64) TYPE argument (before the call), the NewGore kicks
/// Next(10,31) Y then Next(-20,21) X (`Gore.cs:1327-1328`); `scatter` adds
/// the RotatedByRandom(π)+NextDouble position scatter and `lift` the
/// Next(2) −30 gate; then the two Next(-10,11) velocity nudges. The bodies
/// stay the gore lane's (the beenade precedent).
fn gore_census(rng: &mut UnifiedRandom, count: i32, scatter: bool, lift: bool) {
    for _ in 0..count {
        let _ = rng.next_max(100);
        let _ = rng.next_max(100);
        let _ = rng.next_range(61, 64);
        let _ = rng.next_range(10, 31);
        let _ = rng.next_range(-20, 21);
        if scatter {
            let _ = rotated_by_random(rng, Vec2::Y, std::f32::consts::PI);
            let _ = rng.next_double();
        }
        if lift && rng.next_max(2) == 0 {
            // The −30 lift is a body write; the gate draw stays.
        }
        let _ = rng.next_range(-10, 11);
        let _ = rng.next_range(-10, 11);
    }
}

/// `PortalHelper.GetPortalColor(player, portal)` (`PortalHelper.cs:378-
/// 392`), single-player (`Main.netMode == 0`): portal != 0 →
/// hslToRgb(0.52, 1, 0.6), else hslToRgb(0.12, 1, 0.5); A = 66 (the 601
/// caller overrides A to 255). No draws.
fn portal_color(portal: i32) -> Color {
    let c = if portal != 0 {
        crate::ui::color::hsl_to_rgb(0.52, 1.0, 0.6, 66)
    } else {
        crate::ui::color::hsl_to_rgb(0.12, 1.0, 0.5, 66)
    };
    Color::new(c[0], c[1], c[2], c[3])
}

/// The 454/459/709 double-angle ring position (`:73024` / `:73356`):
/// Center + (radius,0).RotatedBy(τ×NextDouble)×NextDouble — the FULL
/// double angle into cos/sin (`Utils.cs:1675`), cast back, f32 multiply;
/// the radius draw rides AFTER the angle. `radius` is the C# `base.width
/// / 2` INTEGER division.
fn double_angle_ring_pos(rng: &mut UnifiedRandom, center: Vec2, radius: f32) -> Vec2 {
    let tau = std::f32::consts::PI * 2.0;
    let angle = tau as f64 * rng.next_double();
    let c = angle.cos() as f32;
    let s = angle.sin() as f32;
    let spun = Vec2::new(radius * c, radius * s);
    center + spun * rng.next_double() as f32
}

/// `Kill()`'s 405/410 chlorophyte ring (`:72853-72872` / `:72923-72942`):
/// `count` dust-212 at Center−(r,r) box 2r×2r defaults; dir = Normalize
/// (own pos − Center) (zero-safe), position = Center + dir×r×scale, vel =
/// dir×|own vel| (the `< 30` branch always taken — the counts are 20/10),
/// colour Lerp(hslToRgb((float)(0.4+NextDouble×0.2), 0.9, 0.5), White,
/// 0.3), noGravity scale 0.7.
fn chlorophyte_ring(
    s: &DeathSnapshot,
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
    count: i32,
    r: f32,
) {
    let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
    let pos = center - Vec2::splat(r);
    let box_size = (r * 2.0) as i32;
    for _ in 0..count {
        let idx = dust.new_dust(rng, ctx, world, pos, box_size, box_size, 212, 0.0, 0.0, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        let dir = safe_normalize(d.position - center);
        d.position = center + dir * r * s.scale;
        d.velocity = dir * d.velocity.length();
        let h = 0.4000000059604645 + rng.next_double() * 0.20000000298023224;
        let c = crate::ui::color::hsl_to_rgb(h, 0.9, 0.5, 255);
        d.color = lerp_color(Color::new(c[0], c[1], c[2], c[3]), Color::WHITE, 0.3);
        d.no_gravity = true;
        d.scale = 0.7;
    }
}

/// `Kill()`'s 358/406/1042 smoke puff (`:74029-74147`): the corpse velocity
/// is REWRITTEN to oldVelocity×0.2 first; `count` dust at pos alpha
/// `alpha` scale `scale`; THREE Next(2) alpha+25 gates, then a Next(2)
/// scale=`small_scale` ELSE noGravity gate; vel×0.3 += newVel×`vel_add`,
/// ×= 1+Next(−100,101)×0.01, X/Y += Next(−50,51)×0.015 each; position =
/// Center. The 1042 adds a trailing Next(6) colour pick.
fn smoke_puff(
    s: &DeathSnapshot,
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
    count: i32,
    dust_type: i32,
    alpha: i32,
    color: Color,
    scale: f32,
    small_scale: f32,
    vel_add: f32,
    color_pick: bool,
) {
    let new_vel = s.old_vel * 0.2;
    let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
    for _ in 0..count {
        let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, dust_type, 0.0, 0.0, alpha, color, scale);
        let d = dust.slot_mut(idx);
        for _ in 0..3 {
            if rng.next_max(2) == 0 {
                d.alpha += 25;
            }
        }
        if rng.next_max(2) == 0 {
            d.scale = small_scale;
        } else {
            d.no_gravity = true;
        }
        d.velocity *= 0.3;
        d.velocity += new_vel * vel_add;
        d.velocity *= 1.0 + rng.next_range(-100, 101) as f32 * 0.01;
        d.velocity.x += rng.next_range(-50, 51) as f32 * 0.015;
        d.velocity.y += rng.next_range(-50, 51) as f32 * 0.015;
        d.position = center;
        if color_pick {
            d.color = match rng.next_max(6) {
                0 => Color::new(255, 255, 220, 255),
                1 => Color::new(230, 255, 255, 255),
                2 => Color::new(255, 210, 255, 255),
                _ => Color::new(225, 210, 255, 255),
            };
        }
    }
}

/// `Kill()`'s 1037/1049 death (`:73862-73920`): the box first inflates to
/// (int)(192×scale) centred, then Resize(64/112) recentres AGAIN — the
/// final box is 64 (1037) / 112 (1049) at the old centre. 6× dust-31
/// alpha 100 scale 1.5 (gravity ON); 3 gores with the NextVector2Circular
/// (25,25) scatter (TWO NextFloat — unit then radius — AFTER the kicks);
/// then `!wet` gates 24 pairs dust-6 alpha 100 + 4 SelectRandom(6,259,158)
/// alpha 200 vel×2.4 scale+=NextFloat. The owner-gated localAI[1]=−1 +
/// Damage are unported state (no draws).
fn cursed_flame_death(
    s: &DeathSnapshot,
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
) {
    let size = if s.kind == 1049 { 112 } else { 64 };
    let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
    let pos = center - Vec2::splat(size as f32 * 0.5);
    for _ in 0..6 {
        dust.new_dust(rng, ctx, world, pos, size, size, 31, 0.0, 0.0, 100, Color::default(), 1.5);
    }
    for _ in 0..3 {
        let _ = rng.next_max(100);
        let _ = rng.next_max(100);
        let _ = rng.next_range(61, 64);
        let _ = rng.next_range(10, 31);
        let _ = rng.next_range(-20, 21);
        let _ = next_vector2_circular(rng, 25.0, 25.0); // the gore scatter
        let _ = rng.next_range(-10, 11);
        let _ = rng.next_range(-10, 11);
    }
    if !s.wet {
        for _ in 0..24 {
            let idx = dust.new_dust(rng, ctx, world, pos, size, size, 6, 0.0, 0.0, 100, Color::default(), 2.5);
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.velocity *= 3.0;
            let idx = dust.new_dust(rng, ctx, world, pos, size, size, 6, 0.0, 0.0, 100, Color::default(), 1.5);
            let d = dust.slot_mut(idx);
            d.velocity *= 2.0;
            d.no_gravity = true;
        }
        for _ in 0..4 {
            let dust_type = rng.next_from_slice(&[6, 259, 158]);
            let idx = dust.new_dust(rng, ctx, world, pos, size, size, dust_type, 2.5 * s.direction as f32, -2.5, 0, Color::default(), 1.0);
            let d = dust.slot_mut(idx);
            d.alpha = 200;
            d.velocity *= 2.4;
            d.scale += rng.next_float();
        }
    }
}

/// `Kill()`'s 178/289 confetti (`:74565-74614`): `count` dust of type
/// Next(139,143) (the type argument draws BEFORE the spawn) speed =
/// baseVel alpha 0 scale 1.2; per dust X/Y += Next(−50,51)×0.01, X/Y ×=
/// 1+Next(−50,51)×0.01, X/Y += Next(−50,51)×0.05, scale ×= 1+Next(−30,31)
/// ×0.01. Then `gores` gore rows (draws-only): the Next(276,283) type
/// argument, the NewGore kicks, then the same X/Y nudge ladder with the
/// scale ×= 1+Next(−20,21)×0.01 BEFORE the final ×0.05 nudges.
fn confetti_burst(
    s: &DeathSnapshot,
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
    count: i32,
    gores: i32,
) {
    for _ in 0..count {
        let dust_type = rng.next_range(139, 143);
        let idx = dust.new_dust(rng, ctx, world, s.pos, s.w, s.h, dust_type, s.vel.x, s.vel.y, 0, Color::default(), 1.2);
        let d = dust.slot_mut(idx);
        d.velocity.x += rng.next_range(-50, 51) as f32 * 0.01;
        d.velocity.y += rng.next_range(-50, 51) as f32 * 0.01;
        d.velocity.x *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
        d.velocity.y *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
        d.velocity.x += rng.next_range(-50, 51) as f32 * 0.05;
        d.velocity.y += rng.next_range(-50, 51) as f32 * 0.05;
        d.scale *= 1.0 + rng.next_range(-30, 31) as f32 * 0.01;
    }
    for _ in 0..gores {
        let _ = rng.next_range(276, 283);
        let _ = rng.next_range(10, 31);
        let _ = rng.next_range(-20, 21);
        let _ = rng.next_range(-50, 51);
        let _ = rng.next_range(-50, 51);
        let _ = rng.next_range(-50, 51);
        let _ = rng.next_range(-50, 51);
        let _ = rng.next_range(-20, 21);
        let _ = rng.next_range(-50, 51);
        let _ = rng.next_range(-50, 51);
    }
}

/// `Kill()`'s 475/505/506/171 trail-walk (`:74615-74733`): gated ai[1] <
/// 10; the walk starts at the centre with dir = −baseVel, the gravity
/// ramp 0.4×min(1, ai[0]/30) accumulates per step; each step rescales dir
/// to 5.6 (the |x|+|y|<1 damp), and when step > ai[1] spawns FOUR dust of
/// `dust_type` at the walk point noGravity vel×0.3; the walk advances by
/// dir, dir resets to −baseVel + the accumulated ramp. No explicit draws
/// beyond the spawns. (The discarded `Math.Atan2` draws nothing.)
fn trail_walk_burst(
    s: &DeathSnapshot,
    dust: &mut DustPool,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
    dust_type: i32,
) {
    if s.ai1 >= 10.0 {
        return;
    }
    let mut walk = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
    let mut dx = -s.vel.x;
    let mut dy = -s.vel.y;
    let count = (30.0 * if s.ai0 <= 17.0 { s.ai0 / 17.0 } else { 1.0 }) as i32;
    let ramp = 0.4 * if s.ai0 <= 30.0 { s.ai0 / 30.0 } else { 1.0 };
    let mut gravity = ramp;
    dy += gravity;
    for i in 0..count {
        let len = (dx * dx + dy * dy).sqrt();
        let mut speed = 5.6f32;
        if dx.abs() + dy.abs() < 1.0 {
            speed *= dx.abs() + dy.abs() / 1.0;
        }
        let f = speed / len;
        dx *= f;
        dy *= f;
        if i as f32 > s.ai1 {
            for _ in 0..4 {
                let idx = dust.new_dust(rng, ctx, world, walk, s.w, s.h, dust_type, 0.0, 0.0, 0, Color::default(), 1.0);
                let d = dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 0.3;
            }
        }
        walk.x += dx;
        walk.y += dy;
        dx = -s.vel.x;
        dy = -s.vel.y;
        gravity += ramp;
        dy += gravity;
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
            ai0: 0.0,
            rot: 0.0,
            direction: 1,
            alpha: 0,
            old_pos: Vec2::new(100.0, 100.0),
            local_ai1: 0.0,
            apprentice_t3: false,
            max_updates: 1.0,
            ai2: 0.0,
            wet: false,
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

    /// The 167/168/170 400-dust rings: exact counts, the type bands and the
    /// post-spawn scale/noGravity writes. The 170's Next(3) gate rides AFTER
    /// each spawn — the stream advances by the internals + 400 gates.
    #[test]
    fn w4_g65_the_167_168_170_rings() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(167)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 400);
        assert!(active.iter().all(|d| d.dust_type == 130 && d.no_gravity && d.scale == 1.3));
        let mut dust = DustPool::new();
        drain(&[snap(168)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 400);
        assert!(active.iter().all(|d| d.dust_type == 131));
        // i <= 100 keeps the default block's scale and gravity.
        assert_eq!(active.iter().filter(|d| d.no_gravity).count(), 299);
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        drain(&[snap(170)], &mut dust, &mut rng, &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 400);
        assert!(active.iter().all(|d| matches!(d.dust_type, 133 | 134)));
        // The band boundaries: dust 0..=100 type 133, 101..=200 type 134,
        // 201..=300 type 133, 301..=399 type 134.
        assert_eq!(active[0].dust_type, 133);
        assert_eq!(active[101].dust_type, 134);
        assert_eq!(active[201].dust_type, 133);
        assert_eq!(active[301].dust_type, 134);
    }

    /// The 169 star: 2 spoke passes × spokes × 2 sides × 20 steps of
    /// dust-133 + 100 dust-132 tail; the four head draws ride BEFORE the
    /// spawns. Count = 80×spokes + 100 with spokes = Next(5,9).
    #[test]
    fn w4_g65_the_169_star_ring() {
        let w = world();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        // Replay the four head draws to learn the spoke count.
        let mut probe = UnifiedRandom::new(7);
        let _ = probe.next_double();
        let spokes = probe.next_range(5, 9);
        drain(&[snap(169)], &mut dust, &mut rng, &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), (80 * spokes + 100) as usize);
        let spokes_dust = active.iter().filter(|d| d.dust_type == 133).count();
        assert_eq!(spokes_dust, (80 * spokes) as usize);
        assert!(active.iter().filter(|d| d.dust_type == 133).all(|d| d.no_gravity));
        let tail: Vec<_> = active.iter().filter(|d| d.dust_type == 132).collect();
        assert_eq!(tail.len(), 100);
        assert!(tail.iter().all(|d| d.no_gravity && d.scale == 1.3));
    }

    /// The 415/416/418 fireworks: 415 = 30×spokes + 100 dust-130; 416 =
    /// 20×spokes dust-131 (spokes = 2×Next(3,6)); 418 = 30×spokes dust-134
    /// + 80 dust-133 + 40 dust-131. All noGravity.
    #[test]
    fn w4_g65_the_415_416_418_fireworks() {
        let w = world();
        for kind in [415u16, 416, 418] {
            let mut probe = UnifiedRandom::new(7);
            let _ = probe.next_double();
            let spokes = match kind {
                415 => probe.next_range(5, 9),
                416 => probe.next_range(3, 6) * 2,
                _ => probe.next_range(5, 12),
            };
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            match kind {
                415 => {
                    assert_eq!(active.len(), (30 * spokes + 100) as usize);
                    assert!(active.iter().all(|d| d.dust_type == 130 && d.no_gravity));
                }
                416 => {
                    assert_eq!(active.len(), (20 * spokes) as usize);
                    assert!(active.iter().all(|d| d.dust_type == 131 && d.no_gravity));
                }
                _ => {
                    assert_eq!(active.len(), (30 * spokes + 120) as usize);
                    assert_eq!(active.iter().filter(|d| d.dust_type == 134).count(), (30 * spokes) as usize);
                    assert_eq!(active.iter().filter(|d| d.dust_type == 133).count(), 80);
                    assert_eq!(active.iter().filter(|d| d.dust_type == 131).count(), 40);
                    assert!(active.iter().all(|d| d.no_gravity));
                }
            }
        }
    }

    /// The 417: exactly 150 dust, the >50/>100 type bands (132/133/132),
    /// four NextDouble head draws, all scale 1.3 + noGravity.
    #[test]
    fn w4_g65_the_417_firework() {
        let w = world();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        drain(&[snap(417)], &mut dust, &mut rng, &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 150);
        assert_eq!(active[0].dust_type, 132);
        assert_eq!(active[51].dust_type, 133);
        assert_eq!(active[101].dust_type, 132);
        assert!(active.iter().all(|d| d.no_gravity && d.scale == 1.3));
        // Determinism: a second identical run is byte-equal (the four
        // NextDouble head draws + the 150 spawns replay identically).
        let mut dust2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(7);
        drain(&[snap(417)], &mut dust2, &mut rng2, &ctx(), &w);
        assert_eq!(dust.slots(), dust2.slots());
        assert_eq!(rng, rng2);
    }

    /// The 1024 sparkle ring: 12 dust-306, all noGravity, two explicit
    /// floats per dust (scale BEFORE the spawn, speed AFTER).
    #[test]
    fn w4_g65_the_1024_sparkle_ring() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(1024)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 12);
        assert!(active.iter().all(|d| d.dust_type == 306 && d.no_gravity));
        // The colour lerp at i=0 is White exactly (t=0).
        assert_eq!(active[0].color, Color::WHITE);
    }

    /// The 873 prismatic ring: 20 dust-267 at the corpse centre, fadeIn 1,
    /// alpha 100, noGravity; the i%4 spokes are the 3.2-speed 2.3-scale
    /// ones; night (non-remix) → the hsl colour arm, no enrage.
    #[test]
    fn w4_g65_the_873_prismatic_ring() {
        let w = world();
        let mut s = snap(873);
        s.ai1 = 0.25;
        let mut dust = DustPool::new();
        drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 20);
        assert!(active.iter().all(|d| d.dust_type == 267 && d.no_gravity && d.alpha == 100 && d.fade_in == 1.0));
        assert_eq!(active.iter().filter(|d| d.scale == 2.3).count(), 5);
        assert_eq!(active.iter().filter(|d| d.scale == 1.9).count(), 5);
        assert_eq!(active.iter().filter(|d| d.scale == 1.6).count(), 10);
        // The centre: every dust sits exactly at the corpse centre
        // (NewDustPerfect overwrites the position).
        let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
        assert!(active.iter().all(|d| d.position == center));
    }

    /// The 659 spirit-flame ring: gated ai[0] >= 0; 30 dust-27 (15 ring at
    /// the centre + 15 scatter) + 10 dust-31, all noGravity. ai[0] < 0 →
    /// nothing.
    #[test]
    fn w4_g65_the_659_spirit_flame_ring() {
        let w = world();
        let mut s = snap(659);
        s.ai0 = 3.0;
        let mut dust = DustPool::new();
        drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 40);
        assert_eq!(active.iter().filter(|d| d.dust_type == 27).count(), 30);
        assert_eq!(active.iter().filter(|d| d.dust_type == 31).count(), 10);
        assert!(active.iter().all(|d| d.no_gravity));
        // The first 15 sit exactly at the corpse centre (the ring).
        let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
        assert!(active[..15].iter().all(|d| d.position == center));
        // The gate: ai[0] < 0 spawns nothing.
        let mut s2 = snap(659);
        s2.ai0 = -1.0;
        let mut dust2 = DustPool::new();
        drain(&[s2], &mut dust2, &mut UnifiedRandom::new(7), &ctx(), &w);
        assert!(dust2.slots().iter().all(|d| !d.active));
    }

    /// The 1012 First Fractal: 40 dust, each 284 or 138 by the Next(3) gate.
    #[test]
    fn w4_g65_the_1012_first_fractal() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(1012)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 40);
        assert!(active.iter().all(|d| matches!(d.dust_type, 284 | 138)));
    }

    /// The 920/921/926/937 queen-slime rings: 6/12/22/12 dust, all
    /// noGravity, the right types.
    #[test]
    fn w4_g65_the_queen_slime_rings() {
        let w = world();
        for (kind, count, dust_type) in [(920u16, 6, 4), (921, 12, 243), (926, 22, 31), (937, 12, 31)] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), count, "kind {kind} count");
            assert!(active.iter().all(|d| d.dust_type == dust_type && d.no_gravity), "kind {kind}");
        }
    }

    /// The 818/856/728 sparkle rings: 818 = 3+3 dust-278; 856 = 6 dust-58 +
    /// 3+3 dust-278; 728 = 7 dust-58 + 8+4 dust-278. The f32-step loops run
    /// 3 (0.34), 8 (0.125), 4 (0.25) iterations.
    #[test]
    fn w4_g65_the_sparkle_rings() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(818)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 6);
        assert!(active.iter().all(|d| d.dust_type == 278 && d.no_gravity));
        let mut dust = DustPool::new();
        drain(&[snap(856)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 12);
        assert_eq!(active.iter().filter(|d| d.dust_type == 58).count(), 6);
        assert_eq!(active.iter().filter(|d| d.dust_type == 278).count(), 6);
        let mut dust = DustPool::new();
        drain(&[snap(728)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 19);
        assert_eq!(active.iter().filter(|d| d.dust_type == 58).count(), 7);
        assert_eq!(active.iter().filter(|d| d.dust_type == 278).count(), 12);
    }

    /// The 985 Terraspark fan: gated localAI[1] != 1; 20+10+10 = 40 dust-107.
    /// localAI[1] == 1 → nothing.
    #[test]
    fn w4_g65_the_985_terraspark_fan() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(985)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 40);
        assert!(active.iter().all(|d| d.dust_type == 107));
        let mut s = snap(985);
        s.local_ai1 = 1.0;
        let mut dust2 = DustPool::new();
        drain(&[s], &mut dust2, &mut UnifiedRandom::new(7), &ctx(), &w);
        assert!(dust2.slots().iter().all(|d| !d.active));
    }

    /// The 710 ballista burst: 2 dust-31 alpha 100 (gravity ON) + 10
    /// dust-6 + 5 dust-31 alpha 0 (all noGravity), every dust inside the
    /// resized 40×40 box centred on the corpse centre.
    #[test]
    fn w4_g65_the_710_ballista_burst() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(710)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 17);
        assert_eq!(active.iter().filter(|d| d.dust_type == 31).count(), 7);
        assert_eq!(active.iter().filter(|d| d.dust_type == 6).count(), 10);
        // The first two keep gravity; the other 15 are noGravity.
        assert!(active[..2].iter().all(|d| !d.no_gravity && d.alpha == 100));
        assert!(active[2..].iter().all(|d| d.no_gravity));
    }

    /// The 662/685/680 flame trails: ≤16 dust-11 (the Next(2) gates skip
    /// some), all noGravity; the 680 adds one Red scale-1.6 dust-11.
    /// Determinism: identical replays are byte-equal.
    #[test]
    fn w4_g65_the_662_685_680_flame_trails() {
        let w = world();
        for kind in [662u16, 685, 680] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(7);
            drain(&[snap(kind)], &mut dust, &mut rng, &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            let cap = if kind == 680 { 17 } else { 16 };
            assert!(active.len() <= cap, "kind {kind}");
            assert!(active.iter().all(|d| d.dust_type == 11 && d.no_gravity), "kind {kind}");
            if kind == 680 {
                let red = active.iter().filter(|d| d.color == Color::new(255, 0, 0, 255)).count();
                assert_eq!(red, 1);
            }
            let mut dust2 = DustPool::new();
            let mut rng2 = UnifiedRandom::new(7);
            drain(&[snap(kind)], &mut dust2, &mut rng2, &ctx(), &w);
            assert_eq!(dust.slots(), dust2.slots());
            assert_eq!(rng, rng2);
        }
    }

    /// The 664-family DD2 explosion: base 74 dust (24× 31, 50× 6), T3 133
    /// dust (38× 31, 45× 6, 50× 270). The trailing gore census draws ride
    /// the same stream — determinism pins the whole arm.
    #[test]
    fn w4_g65_the_664_family_dd2_explosion() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(664)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 74);
        assert_eq!(active.iter().filter(|d| d.dust_type == 31).count(), 24);
        assert_eq!(active.iter().filter(|d| d.dust_type == 6).count(), 50);
        let mut s = snap(666);
        s.apprentice_t3 = true;
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        drain(&[s], &mut dust, &mut rng, &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 133);
        assert_eq!(active.iter().filter(|d| d.dust_type == 31).count(), 38);
        assert_eq!(active.iter().filter(|d| d.dust_type == 6).count(), 45);
        assert_eq!(active.iter().filter(|d| d.dust_type == 270).count(), 50);
        // Determinism (the gore census draws included).
        let mut dust2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(7);
        drain(&[s], &mut dust2, &mut rng2, &ctx(), &w);
        assert_eq!(dust.slots(), dust2.slots());
        assert_eq!(rng, rng2);
    }

    /// The 669 flameburst: 172 dust-4 — the first 12 alpha 100 scale 1.1
    /// with gravity, then 80 noGravity scale-1.2 / 80 gravity scale-0.8
    /// alternating pairs.
    #[test]
    fn w4_g65_the_669_flameburst() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(669)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 172);
        assert!(active.iter().all(|d| d.dust_type == 4));
        assert!(active[..12].iter().all(|d| d.alpha == 100 && d.scale == 1.1 && !d.no_gravity));
        assert_eq!(active[12..].iter().filter(|d| d.no_gravity).count(), 80);
        assert!(active[12..].iter().filter(|d| d.no_gravity).all(|d| d.scale == 1.2));
        assert!(active[12..].iter().filter(|d| !d.no_gravity).all(|d| d.scale == 0.8));
    }

    /// The 614: 10 dust-229, all noGravity.
    #[test]
    fn w4_g65_the_614_burst() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(614)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 10);
        assert!(active.iter().all(|d| d.dust_type == 229 && d.no_gravity));
    }

    /// The 634/635 lunar flare: 40 pairs + 20 tail = 100 dust. The 634's
    /// big dust picks the 5-set, the 635's the 3-set (242/59/88); the mid
    /// dust is 255 (634) / 88 (635) with Crimson×0.5 = (110,10,30,127).
    #[test]
    fn w4_g65_the_634_635_lunar_flare() {
        let w = world();
        for kind in [634u16, 635] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 100, "kind {kind}");
            // Every big dust picks its kind's set (the mid type can also
            // appear there, so pin membership, not counts).
            let big_set: &[i32] = if kind == 635 { &[242, 59, 88] } else { &[242, 73, 72, 71, 255] };
            assert!(active.iter().all(|d| big_set.contains(&d.dust_type)), "kind {kind}");
            let mid: Vec<_> = active.iter().filter(|d| d.color == Color::new(110, 10, 30, 127)).collect();
            assert_eq!(mid.len(), 40, "kind {kind}");
            assert!(mid.iter().all(|d| d.fade_in == 1.0 && d.no_gravity));
        }
    }

    /// The 645: 4 dust-31 + 8 dust-229 = 12 dust (the gore census draws
    /// no dust); the 229s are all noGravity.
    #[test]
    fn w4_g65_the_645_burst() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(645)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 12);
        assert_eq!(active.iter().filter(|d| d.dust_type == 31).count(), 4);
        assert_eq!(active.iter().filter(|d| d.dust_type == 229).count(), 8);
        assert!(active.iter().filter(|d| d.dust_type == 229).all(|d| d.no_gravity));
    }

    /// The 661 nebula blaze: 4 + 40 + 20 + 70 = 134 dust; 74 dust-240,
    /// 60 dust-62; the 4 head dusts keep gravity, the rest noGravity.
    #[test]
    fn w4_g65_the_661_nebula_blaze() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(661)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 134);
        assert_eq!(active.iter().filter(|d| d.dust_type == 240).count(), 74);
        assert_eq!(active.iter().filter(|d| d.dust_type == 62).count(), 60);
        assert!(active[..4].iter().all(|d| !d.no_gravity));
        assert!(active[4..].iter().all(|d| d.no_gravity));
        assert_eq!(active.iter().filter(|d| d.color == Color::new(110, 10, 30, 127)).count(), 20);
    }

    /// The 617 nebula arcanum: 4 + 60 + 10 + 10 = 84 dust; 14 dust-240,
    /// 40 dust-62, 30 dust-90 (Crimson×0.5, fadeIn 1).
    #[test]
    fn w4_g65_the_617_nebula_arcanum() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(617)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 84);
        assert_eq!(active.iter().filter(|d| d.dust_type == 240).count(), 14);
        assert_eq!(active.iter().filter(|d| d.dust_type == 62).count(), 40);
        let mid: Vec<_> = active.iter().filter(|d| d.dust_type == 90).collect();
        assert_eq!(mid.len(), 30);
        assert!(mid.iter().all(|d| d.fade_in == 1.0 && d.color == Color::new(110, 10, 30, 127)));
    }

    /// The 620/618/619 nebula sparks: the crystal-spark shape at the
    /// ai[0] type (618 folds 86 — the pre-dust ai write). Counts 10/10/20.
    #[test]
    fn w4_g65_the_nebula_sparks() {
        let w = world();
        let mut s = snap(620);
        s.ai0 = 86.0;
        let mut dust = DustPool::new();
        drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 10);
        assert!(active.iter().all(|d| matches!(d.dust_type, 86 | 87) && d.no_gravity));
        // The 618's ai0 snapshot is pre-write; the arm folds 86.
        let mut s = snap(618);
        s.ai0 = 0.0;
        let mut dust = DustPool::new();
        drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 10);
        assert!(active.iter().all(|d| matches!(d.dust_type, 86 | 87)));
        let mut s = snap(619);
        s.ai0 = 90.0;
        let mut dust = DustPool::new();
        drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 20);
        assert!(active.iter().all(|d| matches!(d.dust_type, 90 | 91) && d.no_gravity));
    }

    /// The 601 portal: 6 dust-263 at the centre, scale 2 noLight noGravity,
    /// A=255; ai0=0 → the orange hsl(0.12,1,0.5) arm, ai0=1 → the blue
    /// hsl(0.52,1,0.6) arm (the colours differ).
    #[test]
    fn w4_g65_the_601_portal() {
        let w = world();
        let mut s = snap(601);
        s.ai0 = 0.0;
        let mut dust = DustPool::new();
        drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 6);
        assert!(active.iter().all(|d| d.dust_type == 263 && d.scale == 2.0 && d.no_light && d.no_gravity && d.color.a == 255));
        let center = Vec2::new(s.pos.x + s.w as f32 * 0.5, s.pos.y + s.h as f32 * 0.5);
        assert!(active.iter().all(|d| d.position == center));
        let orange = active[0].color;
        let mut s = snap(601);
        s.ai0 = 1.0;
        let mut dust = DustPool::new();
        drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_ne!(active[0].color, orange);
    }

    /// The 596: 4 + 20 + 10 = 34 dust; 14 dust-31, 20 dust-27; the 4 head
    /// dusts keep gravity.
    #[test]
    fn w4_g65_the_596_burst() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(596)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 34);
        assert_eq!(active.iter().filter(|d| d.dust_type == 31).count(), 14);
        assert_eq!(active.iter().filter(|d| d.dust_type == 27).count(), 20);
        assert!(active[..4].iter().all(|d| !d.no_gravity));
        assert!(active[4..].iter().all(|d| d.no_gravity));
    }

    /// The 539: 4 + 20 + 20 + 10 = 54 dust; 14 dust-31, 20 dust-176, 20
    /// dust-180; the 4 head dusts keep gravity.
    #[test]
    fn w4_g65_the_539_burst() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(539)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 54);
        assert_eq!(active.iter().filter(|d| d.dust_type == 31).count(), 14);
        assert_eq!(active.iter().filter(|d| d.dust_type == 176).count(), 20);
        assert_eq!(active.iter().filter(|d| d.dust_type == 180).count(), 20);
        assert!(active[..4].iter().all(|d| !d.no_gravity));
        assert!(active[4..].iter().all(|d| d.no_gravity));
    }

    /// The 585/587 pair bursts: 40 dust each (20 pairs), all noGravity;
    /// the 585 alternates 26/27 (scale 1.3 / 2), the 587 is all 76 with
    /// the hsl colour A=200 (scale 0.9 / 1.1).
    #[test]
    fn w4_g65_the_585_587_pairs() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(585)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 40);
        assert_eq!(active.iter().filter(|d| d.dust_type == 26).count(), 20);
        assert_eq!(active.iter().filter(|d| d.dust_type == 27).count(), 20);
        assert!(active.iter().all(|d| d.no_gravity));
        assert!(active.iter().filter(|d| d.dust_type == 26).all(|d| d.scale == 1.3));
        let mut s = snap(587);
        s.ai1 = 0.3;
        let mut dust = DustPool::new();
        drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 40);
        assert!(active.iter().all(|d| d.dust_type == 76 && d.no_gravity && d.color.a == 200));
        assert_eq!(active.iter().filter(|d| d.scale == 0.9).count(), 20);
        assert_eq!(active.iter().filter(|d| d.scale == 1.1).count(), 20);
    }

    /// The 590 shadowflame: 10 dust, type 165 default / 31 when ai[2]==1;
    /// all noGravity scale 1.5. The three gore census draws ride after.
    #[test]
    fn w4_g65_the_590_shadowflame() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(590)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 10);
        assert!(active.iter().all(|d| d.dust_type == 165 && d.no_gravity && d.scale == 1.5));
        let mut s = snap(590);
        s.ai2 = 1.0;
        let mut dust = DustPool::new();
        drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert!(active.iter().all(|d| d.dust_type == 31));
    }

    /// The 676: 120 dust of types 4/256, fadeIn 0.5 noLight; the type-4
    /// ones carry (80,170,40,120); every dust with post-kick Y>0 was
    /// flipped (final Y ≤ 0).
    #[test]
    fn w4_g65_the_676_burst() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(676)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 120);
        assert!(active.iter().all(|d| matches!(d.dust_type, 4 | 256) && d.fade_in == 0.5 && d.no_light));
        assert!(active.iter().filter(|d| d.dust_type == 4).all(|d| d.color == Color::new(80, 170, 40, 120)));
        assert!(active.iter().all(|d| d.velocity.y <= 0.0));
    }

    /// The 405/410 chlorophyte rings: 20/10 dust-212, noGravity scale 0.7,
    /// every dust exactly r×scale from the centre (the 410's r =
    /// (int)(10×ai[1])).
    #[test]
    fn w4_g65_the_405_410_chlorophyte_rings() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(405)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 20);
        assert!(active.iter().all(|d| d.dust_type == 212 && d.no_gravity && d.scale == 0.7));
        let center = Vec2::new(105.0, 105.0);
        // The radius is r×scale; a dust spawned exactly at the centre
        // normalizes to zero and stays there (XNA's zero-safe Normalize).
        assert!(active.iter().all(|d| {
            let len = (d.position - center).length();
            (len - 10.0).abs() < 0.01 || len < 0.01
        }));
        let mut s = snap(410);
        s.ai1 = 1.5;
        let mut dust = DustPool::new();
        drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 10);
        assert!(active.iter().all(|d| {
            let len = (d.position - center).length();
            (len - 15.0).abs() < 0.01 || len < 0.01
        }));
    }

    /// The 501: 20 dust-188, gravity ON, scale 1.5 (the gore census draws
    /// no dust).
    #[test]
    fn w4_g65_the_501_burst() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(501)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 20);
        assert!(active.iter().all(|d| d.dust_type == 188 && !d.no_gravity && d.scale == 1.5));
    }

    /// The 452/454: 84/127 dust; the 452 = 4 dust-31 (gravity) + 80
    /// dust-229; the 454 = 7 dust-31 (gravity) + 120 dust-229, every
    /// dust-229 noGravity.
    #[test]
    fn w4_g65_the_452_454_bursts() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(452)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 84);
        assert_eq!(active.iter().filter(|d| d.dust_type == 31).count(), 4);
        assert!(active[..4].iter().all(|d| !d.no_gravity));
        assert!(active[4..].iter().all(|d| d.dust_type == 229 && d.no_gravity));
        let mut dust = DustPool::new();
        drain(&[snap(454)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 127);
        assert_eq!(active.iter().filter(|d| d.dust_type == 31).count(), 7);
        assert!(active[..7].iter().all(|d| !d.no_gravity));
        assert!(active[7..].iter().all(|d| d.dust_type == 229 && d.no_gravity));
    }

    /// The 467/468: 84/64 dust; the 467 = 14 dust-31 + 70 dust-6, the 468
    /// = 14 dust-31 + 50 dust-27; the fadeIn-2.5 mids number 30/20.
    #[test]
    fn w4_g65_the_467_468_bursts() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(467)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 84);
        assert_eq!(active.iter().filter(|d| d.dust_type == 31).count(), 14);
        assert_eq!(active.iter().filter(|d| d.dust_type == 6).count(), 70);
        assert_eq!(active.iter().filter(|d| d.fade_in == 2.5).count(), 30);
        let mut dust = DustPool::new();
        drain(&[snap(468)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 64);
        assert_eq!(active.iter().filter(|d| d.dust_type == 31).count(), 14);
        assert_eq!(active.iter().filter(|d| d.dust_type == 27).count(), 50);
        assert_eq!(active.iter().filter(|d| d.fade_in == 2.5).count(), 20);
    }

    /// The 485/484: 15 dust-6 / 5 dust-78, all noGravity; the 484 scale
    /// 0.85.
    #[test]
    fn w4_g65_the_485_484_bursts() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(485)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 15);
        assert!(active.iter().all(|d| d.dust_type == 6 && d.no_gravity));
        let mut dust = DustPool::new();
        drain(&[snap(484)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 5);
        assert!(active.iter().all(|d| d.dust_type == 78 && d.no_gravity && d.scale == 0.85));
    }

    /// The 483: 20 dust-78 + 7 dust-31 + 6 dust-6 = 33 dust; the 31s
    /// scale 0.9 with gravity, the 6-pair seconds keep gravity too.
    #[test]
    fn w4_g65_the_483_burst() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(483)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 33);
        assert_eq!(active.iter().filter(|d| d.dust_type == 78).count(), 20);
        assert_eq!(active.iter().filter(|d| d.dust_type == 31).count(), 7);
        assert_eq!(active.iter().filter(|d| d.dust_type == 6).count(), 6);
        assert!(active.iter().filter(|d| d.dust_type == 31).all(|d| d.scale == 0.9 && !d.no_gravity));
        assert!(active.iter().filter(|d| d.dust_type == 78).all(|d| d.no_gravity));
    }

    /// The 523: 25 dust-256, noGravity, alpha = the corpse's alpha, the
    /// speed ≤ 2.9 (Next(1,30)×0.1).
    #[test]
    fn w4_g65_the_523_burst() {
        let w = world();
        let mut s = snap(523);
        s.alpha = 120;
        let mut dust = DustPool::new();
        drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 25);
        assert!(active.iter().all(|d| d.dust_type == 256 && d.no_gravity && d.alpha == 120));
        assert!(active.iter().all(|d| d.velocity.length() <= 2.91));
    }

    /// The 521: 20 dust of types 254/255, all noGravity.
    #[test]
    fn w4_g65_the_521_burst() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(521)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 20);
        assert!(active.iter().all(|d| matches!(d.dust_type, 254 | 255) && d.no_gravity));
    }

    /// The 459/709: scale < 1 → 13 dust (3× 31 + 10× 226) on the original
    /// box; scale >= 1 → 37 dust (7× 31 + 30× 226) on the 144 resize.
    #[test]
    fn w4_g65_the_459_709_bursts() {
        let w = world();
        for kind in [459u16, 709] {
            let mut s = snap(kind);
            s.scale = 0.5;
            let mut dust = DustPool::new();
            drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 13, "kind {kind} small");
            assert_eq!(active.iter().filter(|d| d.dust_type == 31).count(), 3);
            assert_eq!(active.iter().filter(|d| d.dust_type == 226).count(), 10);
            let mut s = snap(kind);
            s.scale = 1.0;
            let mut dust = DustPool::new();
            drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 37, "kind {kind} big");
            assert_eq!(active.iter().filter(|d| d.dust_type == 31).count(), 7);
            assert_eq!(active.iter().filter(|d| d.dust_type == 226).count(), 30);
            assert!(active.iter().filter(|d| d.dust_type == 226).all(|d| d.no_gravity));
        }
    }

    /// The 639/640 moonlord arrow: Next(5,10) dust-220, noGravity scale
    /// 0.5, each position the midpoint of its spawn and the centre.
    #[test]
    fn w4_g65_the_639_640_moonlord_dust() {
        let w = world();
        for kind in [639u16, 640] {
            let mut probe = UnifiedRandom::new(7);
            let count = probe.next_range(5, 10);
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), count as usize, "kind {kind}");
            assert!(active.iter().all(|d| d.dust_type == 220 && d.no_gravity && d.scale == 0.5));
        }
    }

    /// The 682: 22 dust-272, GRAVITY ON (no noGravity write), scale 0.5.
    #[test]
    fn w4_g65_the_682_burst() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(682)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 22);
        assert!(active.iter().all(|d| d.dust_type == 272 && !d.no_gravity && d.scale == 0.5));
    }

    /// The 449: Next(3,7) dust-228, noGravity scale 2.1.
    #[test]
    fn w4_g65_the_449_burst() {
        let w = world();
        let mut probe = UnifiedRandom::new(7);
        let count = probe.next_range(3, 7);
        let mut dust = DustPool::new();
        drain(&[snap(449)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), count as usize);
        assert!(active.iter().all(|d| d.dust_type == 228 && d.no_gravity && d.scale == 2.1));
    }

    /// The 448/616: 84 dust each (4× 31 gravity + 80× 228/229 noGravity);
    /// the 616's pairs are BOTH alpha 200.
    #[test]
    fn w4_g65_the_448_616_bursts() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(448)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 84);
        assert_eq!(active.iter().filter(|d| d.dust_type == 228).count(), 80);
        assert!(active[..4].iter().all(|d| d.dust_type == 31 && !d.no_gravity));
        assert!(active[4..].iter().all(|d| d.no_gravity));
        let mut dust = DustPool::new();
        drain(&[snap(616)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 84);
        assert_eq!(active.iter().filter(|d| d.dust_type == 229).count(), 80);
        assert!(active[4..].iter().all(|d| d.no_gravity && d.alpha == 200));
    }

    /// The 836/1092: 3 dust-31 White noGravity / 6 dust-269 White noLight.
    #[test]
    fn w4_g65_the_836_1092_bursts() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(836)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 3);
        assert!(active.iter().all(|d| d.dust_type == 31 && d.no_gravity && d.color == Color::WHITE && d.scale == 0.8));
        let mut dust = DustPool::new();
        drain(&[snap(1092)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 6);
        assert!(active.iter().all(|d| d.dust_type == 269 && d.no_light && d.color == Color::WHITE && d.scale == 1.2));
    }

    /// The 424-426: 8 + 64 + 5 = 77 dust; 8 dust-31 gravity, 64 dust-6
    /// noGravity, 5 SelectRandom(6,259,158) alpha 200 (gravity ON).
    #[test]
    fn w4_g65_the_424_426_bursts() {
        let w = world();
        for kind in [424u16, 425, 426] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 77, "kind {kind}");
            assert!(active[..8].iter().all(|d| d.dust_type == 31 && !d.no_gravity));
            assert!(active[8..72].iter().all(|d| d.dust_type == 6 && d.no_gravity));
            assert!(active[72..].iter().all(|d| matches!(d.dust_type, 6 | 259 | 158) && d.alpha == 200 && !d.no_gravity));
        }
    }

    /// The 1037/1049: dry → 6 + 48 + 4 = 58 dust; wet → only the 6 dust-31
    /// (the gore census draws ride either way — determinism pins them).
    #[test]
    fn w4_g65_the_1037_1049_cursed_flame() {
        let w = world();
        for kind in [1037u16, 1049] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 58, "kind {kind} dry");
            assert!(active[..6].iter().all(|d| d.dust_type == 31 && !d.no_gravity));
            assert!(active[6..54].iter().all(|d| d.dust_type == 6 && d.no_gravity));
            let mut s = snap(kind);
            s.wet = true;
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(7);
            drain(&[s], &mut dust, &mut rng, &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 6, "kind {kind} wet");
            let mut dust2 = DustPool::new();
            let mut rng2 = UnifiedRandom::new(7);
            drain(&[s], &mut dust2, &mut rng2, &ctx(), &w);
            assert_eq!(dust.slots(), dust2.slots());
            assert_eq!(rng, rng2);
        }
    }

    /// The 399: 5 dust-12 Red + 10 dust-31 + 40 dust-6 = 55 dust; the
    /// pair-seconds keep gravity.
    #[test]
    fn w4_g65_the_399_burst() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(399)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 55);
        assert!(active[..5].iter().all(|d| d.dust_type == 12 && d.color == Color::new(255, 0, 0, 255)));
        assert_eq!(active.iter().filter(|d| d.dust_type == 31).count(), 10);
        assert_eq!(active.iter().filter(|d| d.dust_type == 6).count(), 40);
        assert_eq!(active[15..].iter().filter(|d| d.no_gravity).count(), 20);
    }

    /// The 358/406/1042 smoke puffs: 50/40/50 dust at the corpse centre,
    /// alpha 75/175/75 (+0/25/50/75 from the three gates), the scale-or-
    /// noGravity split; the 1042's colours are the 4-way pick.
    #[test]
    fn w4_g65_the_smoke_puffs() {
        let w = world();
        let center = Vec2::new(105.0, 105.0);
        for (kind, count, dust_type, alpha) in [(358u16, 50, 211, 75), (406, 40, 4, 175), (1042, 50, 324, 75)] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), count, "kind {kind}");
            assert!(active.iter().all(|d| d.dust_type == dust_type && d.position == center), "kind {kind}");
            assert!(active.iter().all(|d| d.alpha >= alpha && d.alpha <= alpha + 75 && (d.alpha - alpha) % 25 == 0), "kind {kind}");
            // Each dust is EITHER small-scale with gravity OR noGravity.
            let small = if kind == 1042 { 0.8 } else { 0.6 };
            assert!(active.iter().all(|d| (d.scale == small && !d.no_gravity) || d.no_gravity), "kind {kind}");
        }
    }

    /// The 323: 20 dust-7; the noGravity ones scale 1.3, the gravity ones
    /// 0.8.
    #[test]
    fn w4_g65_the_323_burst() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(323)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 20);
        assert!(active.iter().all(|d| d.dust_type == 7));
        assert!(active.iter().all(|d| (d.no_gravity && d.scale == 1.3) || (!d.no_gravity && d.scale == 0.8)));
    }

    /// The 589: 30 dust-11, A=100, the ai1 colour ladder; the first 10
    /// noGravity.
    #[test]
    fn w4_g65_the_589_burst() {
        let w = world();
        let mut s = snap(589);
        s.ai1 = 3.0;
        let mut dust = DustPool::new();
        drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 30);
        assert!(active.iter().all(|d| d.dust_type == 11 && d.color == Color::new(255, 215, 0, 100)));
        assert!(active[..10].iter().all(|d| d.no_gravity));
        assert!(active[10..].iter().all(|d| !d.no_gravity));
        // The exact-equality gate: ai1 = 1.5 keeps Red.
        let mut s = snap(589);
        s.ai1 = 1.5;
        let mut dust = DustPool::new();
        drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert!(active.iter().all(|d| d.color == Color::new(255, 0, 0, 100)));
    }

    /// The 311/316: 5 dust-189/195, scale 0.85 noGravity.
    #[test]
    fn w4_g65_the_311_316_bursts() {
        let w = world();
        for (kind, dust_type) in [(311u16, 189), (316, 195)] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 5);
            assert!(active.iter().all(|d| d.dust_type == dust_type && d.scale == 0.85 && d.no_gravity));
        }
    }

    /// The 1097/229: 6 dust-6 / 25 dust-157 (noGravity scale 1.5); the
    /// owner-gated spawn bursts draw nothing here.
    #[test]
    fn w4_g65_the_1097_229_bursts() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(1097)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 6);
        assert!(active.iter().all(|d| d.dust_type == 6));
        let mut dust = DustPool::new();
        drain(&[snap(229)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 25);
        assert!(active.iter().all(|d| d.dust_type == 157 && d.no_gravity && d.scale == 1.5));
    }

    /// The 270/837: 40 dust (20 pairs); the seconds are dust-5 scale 1.5
    /// (270) / dust-6 scale 2 (837).
    #[test]
    fn w4_g65_the_270_837_pairs() {
        let w = world();
        for (kind, second_type, second_scale) in [(270u16, 5, 1.5f32), (837, 6, 2.0)] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 40, "kind {kind}");
            assert_eq!(active.iter().filter(|d| d.dust_type == 26).count(), 20);
            assert!(active.iter().filter(|d| d.dust_type == second_type).all(|d| d.scale == second_scale && d.no_gravity));
        }
    }

    /// The 178/289 confetti: 85/30 dust of types 139-142; the gore census
    /// draws ride the same stream (determinism).
    #[test]
    fn w4_g65_the_178_289_confetti() {
        let w = world();
        for (kind, count) in [(178u16, 85), (289, 30)] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(7);
            drain(&[snap(kind)], &mut dust, &mut rng, &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), count, "kind {kind}");
            assert!(active.iter().all(|d| (139..=142).contains(&d.dust_type)), "kind {kind}");
            let mut dust2 = DustPool::new();
            let mut rng2 = UnifiedRandom::new(7);
            drain(&[snap(kind)], &mut dust2, &mut rng2, &ctx(), &w);
            assert_eq!(dust.slots(), dust2.slots());
            assert_eq!(rng, rng2);
        }
    }

    /// The trail walks: ai0=30/ai1=0 → 30 steps, steps 1..29 spawn 4
    /// each = 116 dust; ai1 >= 10 → nothing. Types 3/239/30/129.
    #[test]
    fn w4_g65_the_trail_walks() {
        let w = world();
        for (kind, dust_type) in [(475u16, 3), (505, 239), (506, 30), (171, 129)] {
            let mut s = snap(kind);
            s.ai0 = 30.0;
            s.ai1 = 0.0;
            let mut dust = DustPool::new();
            drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 116, "kind {kind}");
            assert!(active.iter().all(|d| d.dust_type == dust_type && d.no_gravity), "kind {kind}");
            let mut s = snap(kind);
            s.ai1 = 10.0;
            let mut dust = DustPool::new();
            drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            assert!(dust.slots().iter().all(|d| !d.active), "kind {kind} gated");
        }
    }

    /// The 117/166: 10 dust-26 defaults / 10 dust-76 noGravity.
    #[test]
    fn w4_g65_the_117_166_bursts() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(117)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 10);
        assert!(active.iter().all(|d| d.dust_type == 26));
        let mut dust = DustPool::new();
        drain(&[snap(166)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 10);
        assert!(active.iter().all(|d| d.dust_type == 76 && d.no_gravity));
    }

    /// The 99-family default arm: 30 dust of the kind-switch type; the
    /// Next(2) gate scales some ×1.4.
    #[test]
    fn w4_g65_the_99_family_default() {
        let w = world();
        for (kind, dust_type) in [(99u16, 1), (1013, 243), (727, 40), (1014, 12), (1021, 323), (1048, 322)] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 30, "kind {kind}");
            assert!(active.iter().all(|d| d.dust_type == dust_type), "kind {kind}");
            // The spawn's default scale is (1+Next(-20,21)*0.01); the gate
            // multiplies some by 1.4 — so the range is [0.8, 1.68].
            assert!(active.iter().all(|d| d.scale >= 0.79 && d.scale <= 1.69), "kind {kind}");
        }
    }

    /// The 1054/1055: 20 dust-30 / 50 dust-192 (noGravity, the grey
    /// colour, scale 1.5).
    #[test]
    fn w4_g65_the_1054_1055_bursts() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(1054)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 20);
        assert!(active.iter().all(|d| d.dust_type == 30));
        let mut dust = DustPool::new();
        drain(&[snap(1055)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 50);
        assert!(active.iter().all(|d| d.dust_type == 192 && d.no_gravity && d.color == Color::new(100, 100, 100, 100)));
    }

    /// The 1005/763: 10 dust-1 / 4 dust-40 (noGravity); the scale gates
    /// ride the randomized base (range checks).
    #[test]
    fn w4_g65_the_1005_763_bursts() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(1005)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 10);
        assert!(active.iter().all(|d| d.dust_type == 1));
        let mut dust = DustPool::new();
        drain(&[snap(763)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 4);
        assert!(active.iter().all(|d| d.dust_type == 40 && d.no_gravity));
    }

    /// The 772 geode: 6 dust-1 + 27 dust-86..90 (noGravity fadeIn 1.1, the
    /// last 7 noLight); the 4 loot census draws ride the stream
    /// (determinism).
    #[test]
    fn w4_g65_the_772_geode() {
        let w = world();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        drain(&[snap(772)], &mut dust, &mut rng, &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 33);
        assert_eq!(active.iter().filter(|d| d.dust_type == 1).count(), 6);
        let gems: Vec<_> = active.iter().filter(|d| (86..=90).contains(&d.dust_type)).collect();
        assert_eq!(gems.len(), 27);
        assert!(gems.iter().all(|d| d.no_gravity && d.fade_in == 1.1));
        assert_eq!(gems.iter().filter(|d| d.no_light).count(), 7);
        let mut dust2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(7);
        drain(&[snap(772)], &mut dust2, &mut rng2, &ctx(), &w);
        assert_eq!(dust.slots(), dust2.slots());
        assert_eq!(rng, rng2);
    }

    /// The 723/724-family: 10 dust each (88 / 58 / 57), the gore census
    /// draws ride the stream (determinism).
    #[test]
    fn w4_g65_the_723_family() {
        let w = world();
        for (kind, dust_type) in [(723u16, 88), (724, 58), (725, 58), (726, 58), (907, 58), (1133, 57)] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(7);
            drain(&[snap(kind)], &mut dust, &mut rng, &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 10, "kind {kind}");
            assert!(active.iter().all(|d| d.dust_type == dust_type), "kind {kind}");
            let mut dust2 = DustPool::new();
            let mut rng2 = UnifiedRandom::new(7);
            drain(&[snap(kind)], &mut dust2, &mut rng2, &ctx(), &w);
            assert_eq!(dust.slots(), dust2.slots());
            assert_eq!(rng, rng2);
        }
    }

    /// The 1053 lava boulder: 25 dust-31 (gravity) + 80 dust-6 = 105; the
    /// pair-seconds keep gravity (40 of the 80).
    #[test]
    fn w4_g65_the_1053_lava_boulder() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(1053)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 105);
        assert_eq!(active.iter().filter(|d| d.dust_type == 31).count(), 25);
        assert!(active[..25].iter().all(|d| !d.no_gravity));
        assert_eq!(active[25..].iter().filter(|d| d.no_gravity).count(), 40);
    }

    /// The 1098/1123: 50 dust-306, the (100,210,255,127) colour.
    #[test]
    fn w4_g65_the_1098_1123_bursts() {
        let w = world();
        for kind in [1098u16, 1123] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 50, "kind {kind}");
            assert!(active.iter().all(|d| d.dust_type == 306 && d.color == Color::new(100, 210, 255, 127)));
        }
    }

    /// The 1020/1105: 15 + 10 + 3 = 28 dust (31/228/226), all Transparent;
    /// the 228/226 noGravity, the 31 gravity. The corner gore census
    /// rides the stream (determinism).
    #[test]
    fn w4_g65_the_1020_1105_bursts() {
        let w = world();
        for kind in [1020u16, 1105] {
            let mut dust = DustPool::new();
            let mut rng = UnifiedRandom::new(7);
            drain(&[snap(kind)], &mut dust, &mut rng, &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 28, "kind {kind}");
            assert_eq!(active.iter().filter(|d| d.dust_type == 31).count(), 15);
            assert_eq!(active.iter().filter(|d| d.dust_type == 228).count(), 10);
            assert_eq!(active.iter().filter(|d| d.dust_type == 226).count(), 3);
            assert!(active.iter().all(|d| d.color == Color::TRANSPARENT));
            assert!(active[..15].iter().all(|d| !d.no_gravity));
            assert!(active[15..].iter().all(|d| d.no_gravity));
            let mut dust2 = DustPool::new();
            let mut rng2 = UnifiedRandom::new(7);
            drain(&[snap(kind)], &mut dust2, &mut rng2, &ctx(), &w);
            assert_eq!(dust.slots(), dust2.slots());
            assert_eq!(rng, rng2);
        }
    }

    /// The 1085/1080: 10 dust-330 / 30 dust-329; the 1080's scale gate
    /// gives [0.5,1.2] ranges.
    #[test]
    fn w4_g65_the_1085_1080_bursts() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(1085)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 10);
        assert!(active.iter().all(|d| d.dust_type == 330));
        let mut dust = DustPool::new();
        drain(&[snap(1080)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 30);
        assert!(active.iter().all(|d| d.dust_type == 329 && d.scale >= 0.49 && d.scale <= 1.21));
    }

    /// The 1038/1036/756/961 Perfect rings: 4/3/40/4 dust, all noGravity
    /// (the 756/961 alpha 100 scale 1).
    #[test]
    fn w4_g65_the_perfect_rings() {
        let w = world();
        for (kind, count, dust_type) in [(1038u16, 4, 166), (1036, 3, 5), (756, 40, 5), (961, 4, 16)] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), count, "kind {kind}");
            assert!(active.iter().all(|d| d.dust_type == dust_type), "kind {kind}");
            if kind == 1038 || kind == 1036 {
                assert!(active.iter().all(|d| d.no_gravity), "kind {kind}");
            } else {
                assert!(active.iter().all(|d| d.alpha == 100 && d.scale == 1.0), "kind {kind}");
            }
        }
    }

    /// The gem bolts: 15 dust of kind−121+86 (597 → 262); the AoE feature
    /// (ai0 bit 3 = 8.0) adds 30 more = 45. All noGravity in the base 15.
    #[test]
    fn w4_g65_the_gem_bolts() {
        let w = world();
        for kind in [121u16, 122, 123, 124, 125, 126, 597] {
            let dust_type = if kind == 597 { 262 } else { kind as i32 - 121 + 86 };
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 15, "kind {kind}");
            assert!(active.iter().all(|d| d.dust_type == dust_type && d.no_gravity), "kind {kind}");
            let mut s = snap(kind);
            s.ai0 = 8.0; // the AoeExplosion bit
            let mut dust = DustPool::new();
            drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 45, "kind {kind} aoe");
        }
    }

    /// The 1115: count = 10+(int)(20×Remap(ai0,0,60,0,1)); ai0=0 → 10,
    /// ai0=60 → 30; all dust-197 noGravity.
    #[test]
    fn w4_g65_the_1115_burst() {
        let w = world();
        for (ai0, count) in [(0.0f32, 10), (60.0, 30), (30.0, 20)] {
            let mut s = snap(1115);
            s.ai0 = ai0;
            let mut dust = DustPool::new();
            drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), count, "ai0 {ai0}");
            assert!(active.iter().all(|d| d.dust_type == 197 && d.no_gravity));
        }
    }

    /// The 379/377: 5 dust-171, noGravity fadeIn 1.5, scale in [0.1,0.9].
    #[test]
    fn w4_g65_the_379_377_bursts() {
        let w = world();
        for kind in [379u16, 377] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 5, "kind {kind}");
            assert!(active.iter().all(|d| d.dust_type == 171 && d.no_gravity && d.fade_in == 1.5));
            assert!(active.iter().all(|d| d.scale >= 0.09 && d.scale <= 0.91));
        }
    }

    /// The 281/240 bomb dust: 20 dust-31 + 20 dust-6 = 40; the 281's
    /// ai0==−3 gate spawns nothing. The 162: 20 + 50 + 10 = 80.
    #[test]
    fn w4_g65_the_bomb_bursts() {
        let w = world();
        for kind in [281u16, 240] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 40, "kind {kind}");
            assert_eq!(active.iter().filter(|d| d.dust_type == 31).count(), 20);
            assert_eq!(active.iter().filter(|d| d.dust_type == 6).count(), 20);
        }
        let mut s = snap(281);
        s.ai0 = -3.0;
        let mut dust = DustPool::new();
        drain(&[s], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        assert!(dust.slots().iter().all(|d| !d.active));
        let mut dust = DustPool::new();
        drain(&[snap(162)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 80);
        assert_eq!(active.iter().filter(|d| d.dust_type == 31).count(), 20);
        assert_eq!(active.iter().filter(|d| d.dust_type == 6).count(), 60);
    }

    /// The 282: 10 dust-171 noGravity fadeIn 1.5.
    #[test]
    fn w4_g65_the_282_burst() {
        let w = world();
        let mut dust = DustPool::new();
        drain(&[snap(282)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 10);
        assert!(active.iter().all(|d| d.dust_type == 171 && d.no_gravity && d.fade_in == 1.5));
    }

    /// The 660/638: Next(4,10) dust-180 / Next(2,5) dust-229, noGravity.
    #[test]
    fn w4_g65_the_660_638_bursts() {
        let w = world();
        let mut probe = UnifiedRandom::new(7);
        let count = probe.next_range(4, 10);
        let mut dust = DustPool::new();
        drain(&[snap(660)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), count as usize);
        assert!(active.iter().all(|d| d.dust_type == 180 && d.no_gravity && d.scale == 2.0));
        let mut probe = UnifiedRandom::new(7);
        let count = probe.next_range(2, 5);
        let mut dust = DustPool::new();
        drain(&[snap(638)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), count as usize);
        assert!(active.iter().all(|d| d.dust_type == 229 && d.no_gravity));
    }

    /// The 411-414: 5 dust of type 9/11/19, noGravity.
    #[test]
    fn w4_g65_the_411_414_bursts() {
        let w = world();
        for (kind, dust_type) in [(411u16, 9), (412, 11), (413, 19), (414, 11)] {
            let mut dust = DustPool::new();
            drain(&[snap(kind)], &mut dust, &mut UnifiedRandom::new(7), &ctx(), &w);
            let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
            assert_eq!(active.len(), 5, "kind {kind}");
            assert!(active.iter().all(|d| d.dust_type == dust_type && d.no_gravity), "kind {kind}");
        }
    }
}
