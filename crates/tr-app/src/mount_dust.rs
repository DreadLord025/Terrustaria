//! W4-e4 — the world dust sites' SECOND batch, the Mount.cs lane:
//! `Dust.NewDust*` sites 1-10 (lines 3113-4123, research/66's claim doc;
//! batch 3 — the drill/dash/basilisk frame arms). Sites 11-19 (the golf
//! cart, the minecart effects, the witch broom, the drill-mount trail)
//! land in batch 4. The orchestrator sites 41-61 live in
//! `orchestrator_dust.rs`; WorldGen.cs is W4-g14's, Main.cs is
//! W4-g11/g12/g13's, d54 keeps the first-40 orchestrator lane.
//!
//! Contract (the S1-E-1 pool, `tr_core::particles`, d50's DustPool):
//! every draw rides the caller's `UpdatePlayers` stream in C# statement
//! order (`Mount.UseDrill` ← `Player.cs:28290`, `Mount.UpdateFrame` ←
//! `:36200-36216`, `Mount.UpdateEffects` ← `:25658` — all inside the
//! player pass) — the explicit `Main.rand` argument draws FIRST, then
//! the `NewDust` internals inside `DustPool::new_dust` (`Dust.cs:75-256`),
//! then the post-spawn writes (the sentinel 6000 takes them like the
//! C#). The mount STATE MACHINES (the frameExtra counters, the
//! `_mountSpecificData` latches, the basiliskCharge ramp) stay the
//! wiring's — these emitters take the tick-moment snapshots the C#
//! reads, exactly like d51's `JumpDust`/`DashDust` convention. The
//! `dust.shader` writes are dropped fields (the port has no draw-shader
//! chain, `particles.rs`'s cited drop); `customData` likewise.
//!
//! Batch ledger (C# `Dust.NewDust*` line = site id):
//! - 3113/3170 — `UseDrill`'s block/wall spark pairs (`Mount.cs:
//!   3106-3116`/`:3163-3173`).
//! - 3878/3896 — `UpdateFrame` case 10/40/41/42/47's dash ring + the
//!   type-10 body puff (`:3866-3902`).
//! - 3939/3970 — case 44's wolf dash ring + gated ground puff
//!   (`:3931-3977`).
//! - 4000/4023 — case 45's witch dash ring + gated SelectRandom puff
//!   (`:3992-4041`).
//! - 4103/4123 — case 14's basilisk charge puffs + charge trail
//!   (`:4099-4139`); the `FloorVisuals` call between them (`:4113`) is
//!   d51's player-dust lane — the two emitters stay SPLIT so the wiring
//!   interleaves it in C# order.
//! (Batch 4 adds: 4626 golf-cart smoke, 4703/4719/4750/4782 the
//! gem/steam/exhaust/confetti minecart effects, 4824/4849 the witch-broom
//! trinket, 5061/5071 the drill-mount lerp trail.)
//! WRITTEN NOT RUN (code-only mode): the witnesses below are compiled by
//! the merge-verify wave, not executed here.

#![allow(dead_code)] // unwired until the mount frame/effects drains land (code-only mode)

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

use crate::bow_launch::{rotated_by, safe_normalize};

/// `Math.Sign(float)` (`System.Math`): −1 / 0 / +1 as an INTEGER — the
/// zero case matters (Rust's `f32::signum` returns ±1.0 for ±0.0, the
/// C# returns 0; the dash kicks read it at rest).
fn math_sign(v: f32) -> i32 {
    if v < 0.0 {
        -1
    } else if v > 0.0 {
        1
    } else {
        0
    }
}

/// Sites 3113/3170 — `UseDrill`'s spark pair (`Mount.cs:3106-3116` /
/// `:3163-3173`), one per picked tile. `aim` = `(tileCentre -
/// player.Center).ToRotation()` computed by the caller at the pick
/// moment; per spark: `Next(2) == 1` picks −π/2 else +π/2 (the sign
/// draw FIRST), `NextDouble` scales 2..4, the velocity is
/// `(cos(num2)*num3, sin(num2)*num3)` — Math.Cos/Sin in DOUBLE of the
/// widened f32 angle. Type 230, box 0×0, noGravity; `customData` dropped.
pub(crate) fn drill_sparks(
    tile_centre: Vec2,
    aim: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for _ in 0..2 {
        // `:3110`/`:3167` — the sign draw, then the angle add.
        let sign = if rng.next_max(2) == 1 { -1.0f32 } else { 1.0 };
        let num2 = aim + sign * (std::f32::consts::PI / 2.0);
        // `:3111`/`:3168` — (float)NextDouble() * 2f + 2f.
        let num3 = rng.next_double() as f32 * 2.0 + 2.0;
        let vel = Vec2::new((num2 as f64).cos() as f32 * num3, (num2 as f64).sin() as f32 * num3);
        let idx = dust.new_dust(rng, ctx, world, tile_centre, 0, 0, 230, vel.x, vel.y, 0, Color::default(), 1.0);
        dust.slot_mut(idx).no_gravity = true; // `:3114`/`:3171`
        // `customData = mountedPlayer` (`:3115`/`:3172`): dropped field.
    }
}

/// Site 3878 — `UpdateFrame` case 10/40/41/42/47's dash ring
/// (`Mount.cs:3866-3892`). The caller's gates: state == 1, `flag7`
/// (|vx| > DashSpeed − RunSpeed/2), `flag8` (frameExtra < 6 BEFORE the
/// increment), type 10 or 47. `frame_extra` is the POST-increment value
/// the ring reads (`:3860`). Type 10 rolls `SelectRandom(176,177,179)`
/// (one Next(3)) BEFORE the ring; type 47 keeps 6. One phase float, then
/// 14 dusts: `UnitY.RotatedBy(num22 * 2π / 14 + phase)` (f32 angle
/// chain), ×0.2×frameExtra, position = anchor + ring×(40,30), velocity =
/// ring + (RunSpeed − Sign(vx)×frameExtra×2, 0), noGravity, type 47 →
/// noLightEmittance, scale = 1 + NextFloat×0.8, fadeIn = NextFloat×2.
#[allow(clippy::too_many_arguments)]
pub(crate) fn unicorn_dash_ring(
    mount_type: i32,
    centre: Vec2,
    width: f32,
    direction: i32,
    vx: f32,
    run_speed: f32,
    frame_extra: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let mut dtype = 6; // `:3868`
    if mount_type == 10 {
        // `:3871` — Utils.SelectRandom (one Next(3), Utils.cs:2459-2462).
        dtype = [176, 177, 179][rng.next_max(3) as usize];
    }
    let anchor = centre + Vec2::new(width * direction as f32, 0.0); // `:3873`
    let extents = Vec2::new(40.0, 30.0); // `:3874`
    let phase = std::f32::consts::PI * 2.0 * rng.next_float(); // `:3875`
    let mut num22 = 0.0f32;
    while num22 < 14.0 {
        let idx = dust.new_dust(rng, ctx, world, anchor, 0, 0, dtype, 0.0, 0.0, 0, Color::default(), 1.0);
        // `:3879-3880` — the ring vector (UnitY spun, then scaled).
        let angle = num22 * (std::f32::consts::PI * 2.0) / 14.0 + phase;
        let mut ring = rotated_by(Vec2::new(0.0, 1.0), angle as f64);
        ring *= 0.2 * frame_extra as f32;
        let d = dust.slot_mut(idx);
        d.position = anchor + ring * extents; // `:3881`
        // `:3882` — Math.Sign(vx) * frameExtra * 2, INTEGER product then
        // the float cast (the C# `(float)(Math.Sign(...) * … * 2)`).
        let kick = run_speed - (math_sign(vx) * frame_extra * 2) as f32;
        d.velocity = ring + Vec2::new(kick, 0.0);
        d.no_gravity = true; // `:3883`
        if mount_type == 47 {
            d.no_light_emittance = true; // `:3884-3887`
        }
        d.scale = 1.0 + rng.next_float() * 0.8; // `:3888`
        d.fade_in = rng.next_float() * 2.0; // `:3889`
        num22 += 1.0;
    }
}

/// Site 3896 — the type-10 dash body puff (`Mount.cs:3894-3902`), gated
/// `(type == 10) & flag7` (every dash tick, no frameExtra gate). The
/// `SelectRandom(176,177,179)` draw is the NewDust TYPE ARGUMENT (before
/// the internals); box = the player's full width×height at position;
/// velocity Zero, noGravity, scale = 0.5 + NextFloat×0.8, fadeIn = 1 +
/// NextFloat×2.
pub(crate) fn unicorn_dash_body(
    position: Vec2,
    width: i32,
    height: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let dtype = [176, 177, 179][rng.next_max(3) as usize]; // `:3896`
    let idx = dust.new_dust(rng, ctx, world, position, width, height, dtype, 0.0, 0.0, 0, Color::default(), 1.0);
    let d = dust.slot_mut(idx);
    d.velocity = Vec2::ZERO; // `:3897`
    d.no_gravity = true; // `:3898`
    d.scale = 0.5 + rng.next_float() * 0.8; // `:3899`
    d.fade_in = 1.0 + rng.next_float() * 2.0; // `:3900`
}

/// The wolf/witch dash rings' shared body (case 44 `:3931-3957`, case 45
/// `:3992-4018`): the caller's latch (`!flag && speedGate`) fires once;
/// one phase float, then 20 dusts of `dtype`: `UnitY.RotatedBy(num *
/// 2π / 20 + phase)` ×0.8, position = anchor + ring×(40,30), velocity =
/// ring + (RunSpeed − Sign(velocity.Length()), 0) — the sign of the
/// LENGTH (+1 unless zero), vx > 0 flips X, `Next(2) == 0` halves the
/// velocity, noGravity, scale = 1.5 + NextFloat×0.8, fadeIn 0.
#[allow(clippy::too_many_arguments)]
fn dash_ring_20(
    dtype: i32,
    centre: Vec2,
    width: f32,
    direction: i32,
    vx: f32,
    speed_len: f32,
    run_speed: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let anchor = centre + Vec2::new(width * direction as f32, 0.0);
    let extents = Vec2::new(40.0, 30.0);
    let phase = std::f32::consts::PI * 2.0 * rng.next_float();
    let mut n = 0.0f32;
    while n < 20.0 {
        let idx = dust.new_dust(rng, ctx, world, anchor, 0, 0, dtype, 0.0, 0.0, 0, Color::default(), 1.0);
        let angle = n * (std::f32::consts::PI * 2.0) / 20.0 + phase;
        let mut ring = rotated_by(Vec2::new(0.0, 1.0), angle as f64);
        ring *= 0.8;
        let d = dust.slot_mut(idx);
        d.position = anchor + ring * extents;
        // `:3943`/`:4004` — Math.Sign(velocity.Length()).
        let kick = run_speed - math_sign(speed_len) as f32;
        d.velocity = ring + Vec2::new(kick, 0.0);
        if vx > 0.0 {
            d.velocity.x *= -1.0; // `:3944-3947`/`:4005-4008`
        }
        if rng.next_max(2) == 0 {
            d.velocity *= 0.5; // `:3948-3951`/`:4009-4012`
        }
        d.no_gravity = true;
        d.scale = 1.5 + rng.next_float() * 0.8;
        d.fade_in = 0.0;
        n += 1.0;
    }
}

/// Site 3939 — case 44's wolf dash ring, type 228 (`Mount.cs:3939`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn wolf_dash_ring(
    centre: Vec2,
    width: f32,
    direction: i32,
    vx: f32,
    speed_len: f32,
    run_speed: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    dash_ring_20(228, centre, width, direction, vx, speed_len, run_speed, rng, dust, ctx, world);
}

/// Site 4000 — case 45's witch dash ring, type 6 (`Mount.cs:4000`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn witch_dash_ring(
    centre: Vec2,
    width: f32,
    direction: i32,
    vx: f32,
    speed_len: f32,
    run_speed: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    dash_ring_20(6, centre, width, direction, vx, speed_len, run_speed, rng, dust, ctx, world);
}

/// Site 3970 — case 44's gated ground puff (`Mount.cs:3958-3977`).
/// `num7 = (int)RunSpeed − (int)|vx|`, floored at 1; `Next(num7) == 0`
/// gates. Box: num9 = width/2 + 10 (INTEGER), num8 = 22; anchor =
/// Bottom − (num9, num8 − 6); type 228, velocity Zero, noGravity,
/// noLight, scale = 0.25 + NextFloat×0.8, fadeIn = 0.5 + NextFloat×2.
/// Returns whether the gate passed (the draw is spent either way).
pub(crate) fn wolf_ground_puff(
    bottom: Vec2,
    width: i32,
    vx: f32,
    run_speed: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) -> bool {
    let mut num7 = run_speed as i32 - vx.abs() as i32; // `:3958`
    if num7 <= 0 {
        num7 = 1; // `:3959-3962`
    }
    if rng.next_max(num7) != 0 {
        return false; // `:3963`
    }
    let num8 = 22; // `:3965`
    let num9 = width / 2 + 10; // `:3966` — integer division.
    let anchor = Vec2::new(bottom.x - num9 as f32, bottom.y - (num8 - 6) as f32);
    let idx = dust.new_dust(rng, ctx, world, anchor, num9 * 2, num8, 228, 0.0, 0.0, 0, Color::default(), 1.0);
    let d = dust.slot_mut(idx);
    d.velocity = Vec2::ZERO;
    d.no_gravity = true;
    d.no_light = true;
    d.scale = 0.25 + rng.next_float() * 0.8; // `:3974`
    d.fade_in = 0.5 + rng.next_float() * 2.0; // `:3975`
    true
}

/// Site 4023 — case 45's gated puff (`Mount.cs:4019-4041`), every
/// speed-gated tick. `SelectRandom([6,6,31])` (one Next(3)) picks the
/// type FIRST; box 12×12 at Center − (6, −6); velocity = player
/// velocity ×0.1; `Next(2) == 0` → noGravity; scale = 0.7 +
/// NextFloat×0.8; `Next(3) == 0` → fadeIn 0.1; type 31 forces noGravity,
/// scale ×1.5, fadeIn 0.2 (the LAST writes win).
pub(crate) fn witch_gated_puff(
    centre: Vec2,
    pvx: f32,
    pvy: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let num19 = [6, 6, 31][rng.next_max(3) as usize]; // `:4021`
    let num20 = 6; // `:4022`
    let anchor = centre - Vec2::new(num20 as f32, (num20 - 12) as f32);
    let idx = dust.new_dust(rng, ctx, world, anchor, num20 * 2, num20 * 2, num19, 0.0, 0.0, 0, Color::default(), 1.0);
    let d = dust.slot_mut(idx);
    d.velocity = Vec2::new(pvx, pvy) * 0.1; // `:4024`
    if rng.next_max(2) == 0 {
        d.no_gravity = true; // `:4025-4028`
    }
    d.scale = 0.7 + rng.next_float() * 0.8; // `:4029`
    if rng.next_max(3) == 0 {
        d.fade_in = 0.1; // `:4030-4033`
    }
    if num19 == 31 {
        // `:4034-4039`
        d.no_gravity = true;
        d.scale *= 1.5;
        d.fade_in = 0.2;
    }
}

/// Site 4103 — case 14's basilisk charge puffs (`Mount.cs:4099-4109`),
/// gated `flag2 && vy == 0`: two type-31 dusts at BottomLeft, box
/// width×6; velocity = (vx×0.15, NextFloat×−2), noLight, scale = 0.5 +
/// NextFloat×0.8, fadeIn = 0.5 + NextFloat×1. The `FloorVisuals` call
/// that follows (`:4110-4115`, cMount == 0) is d51's player-dust lane —
/// the wiring runs it BETWEEN this and [`basilisk_charge_trail`].
pub(crate) fn basilisk_charge_puffs(
    bottom_left: Vec2,
    width: i32,
    vx: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for _ in 0..2 {
        let idx = dust.new_dust(rng, ctx, world, bottom_left, width, 6, 31, 0.0, 0.0, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.velocity = Vec2::new(vx * 0.15, rng.next_float() * -2.0); // `:4104`
        d.no_light = true; // `:4105`
        d.scale = 0.5 + rng.next_float() * 0.8; // `:4106`
        d.fade_in = 0.5 + rng.next_float() * 1.0; // `:4107`
    }
}

/// Site 4123 — case 14's basilisk charge trail (`Mount.cs:4117-4139`),
/// gated `Sign(player.velocity.X) == direction`, `(int)(3 × charge)`
/// iterations. Per dust: type 6 at BottomLeft width×6, then position =
/// Center + (sign×38, 6 + NextFloat×12); velocity = (position − (Center
/// + (sign×40, 12))).SafeNormalize(Zero) × (3.5 + NextFloat×0.5); Y < 0
/// → Y ×= 1 + 2×NextFloat; += player velocity ×0.55; ×= |velocity| /
/// RunSpeed; ×= charge; noGravity, noLight, scale/fadeIn rolls.
#[allow(clippy::too_many_arguments)]
pub(crate) fn basilisk_charge_trail(
    bottom_left: Vec2,
    centre: Vec2,
    width: i32,
    sign: f32,
    pvx: f32,
    pvy: f32,
    charge: f32,
    run_speed: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let count = (3.0 * charge) as i32; // `:4121` — the float product truncates.
    let target = centre + Vec2::new(sign * 40.0, 12.0); // `:4124`
    let plen = Vec2::new(pvx, pvy).length();
    for _ in 0..count {
        let idx = dust.new_dust(rng, ctx, world, bottom_left, width, 6, 6, 0.0, 0.0, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        // `:4125` — num3 − 2 = 38, num2 − 6 = 6, + NextFloat×12.
        d.position = centre + Vec2::new(sign * (40.0 - 2.0), 12.0 - 6.0 + rng.next_float() * 12.0);
        // `:4126`
        let speed = 3.5 + rng.next_float() * 0.5;
        d.velocity = safe_normalize(d.position - target, Vec2::ZERO) * speed;
        if d.velocity.y < 0.0 {
            d.velocity.y *= 1.0 + 2.0 * rng.next_float(); // `:4127-4130`
        }
        d.velocity += Vec2::new(pvx, pvy) * 0.55; // `:4131`
        d.velocity *= plen / run_speed; // `:4132`
        d.velocity *= charge; // `:4133`
        d.no_gravity = true; // `:4134`
        d.no_light = true; // `:4135`
        d.scale = 0.5 + rng.next_float() * 0.8; // `:4136`
        d.fade_in = 0.5 + rng.next_float() * 1.0; // `:4137`
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> World {
        World::new("e4m".into(), 200, 200)
    }

    fn ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(3200.0, 3200.0),
            max_dust_to_draw: 6000,
        }
    }

    fn active(pool: &DustPool) -> Vec<usize> {
        (0..6000).filter(|&i| pool.slot(i).active).collect()
    }

    /// W4-e4 — sites 3113/3170: two sparks per pick, each spends the
    /// sign draw + the NextDouble scale + 6 internals (230 is outside
    /// the tweak chain) = 8 × 2 = 16; the velocity rides cos/sin of the
    /// ±π/2-offset aim, noGravity pinned.
    #[test]
    fn w4_e4_drill_sparks_spin_off_the_aim() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(101);
        let mut mirror = UnifiedRandom::new(101);
        let tile = Vec2::new(1000.0, 900.0);
        let aim = 0.7f32;
        drill_sparks(tile, aim, &mut rng, &mut pool, &c, &w);
        let slots = active(&pool);
        assert_eq!(slots.len(), 2);
        for &i in &slots {
            let sign = if mirror.next_max(2) == 1 { -1.0f32 } else { 1.0 };
            let num2 = aim + sign * (std::f32::consts::PI / 2.0);
            let num3 = mirror.next_double() as f32 * 2.0 + 2.0;
            // The internals draw position X/Y FIRST (the 0×0 box clamps
            // to 5 → Next(1) each), then the velocity jitter, frame row,
            // scale — mirror that exact order.
            let _px = mirror.next_max(1);
            let _py = mirror.next_max(1);
            let jx = mirror.next_range(-20, 21) as f32 * 0.1;
            let jy = mirror.next_range(-20, 21) as f32 * 0.1;
            let _frame = mirror.next_max(3);
            let _scale = mirror.next_range(-20, 21);
            let d = pool.slot(i);
            assert_eq!(d.dust_type, 230);
            assert!(d.no_gravity);
            let vx = (num2 as f64).cos() as f32 * num3 + jx;
            let vy = (num2 as f64).sin() as f32 * num3 + jy;
            assert_eq!(d.velocity, Vec2::new(vx, vy));
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e4 — site 3878: type 10 rolls the SelectRandom BEFORE the
    /// phase float; 14 dusts each 6 internals (176/177/179 outside the
    /// tweak chain) + 2 post floats = 1 + 1 + 14×8 = 114 draws; the ring
    /// positions ride the 0.2×frameExtra scale, type 47 keeps type 6 and
    /// sets noLightEmittance.
    #[test]
    fn w4_e4_unicorn_ring_rolls_the_type_first() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(103);
        unicorn_dash_ring(10, Vec2::new(900.0, 900.0), 20.0, 1, 6.0, 8.0, 3, &mut rng, &mut pool, &c, &w);
        let slots = active(&pool);
        assert_eq!(slots.len(), 14);
        let t = pool.slot(slots[0]).dust_type;
        assert!([176, 177, 179].contains(&t));
        for &i in &slots {
            assert_eq!(pool.slot(i).dust_type, t);
            assert!(pool.slot(i).no_gravity);
            assert!(!pool.slot(i).no_light_emittance);
        }
        let mut bare = UnifiedRandom::new(103);
        for _ in 0..114 {
            bare.next();
        }
        assert_eq!(rng.next_range(0, 1000), bare.next_range(0, 1000));
        // Type 47: no SelectRandom draw, type 6 (tweak chain → 7
        // internals), noLightEmittance set: 1 + 14×9 = 127 draws.
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(103);
        unicorn_dash_ring(47, Vec2::new(900.0, 900.0), 20.0, 1, 6.0, 8.0, 3, &mut rng, &mut pool, &c, &w);
        for &i in &active(&pool) {
            assert_eq!(pool.slot(i).dust_type, 6);
            assert!(pool.slot(i).no_light_emittance);
        }
        let mut bare = UnifiedRandom::new(103);
        for _ in 0..127 {
            bare.next();
        }
        assert_eq!(rng.next_range(0, 1000), bare.next_range(0, 1000));
    }

    /// W4-e4 — sites 3939/4000: the 20-dust rings spend 1 phase + 20 ×
    /// (internals + Next(2) + 2 floats); 228 rides the tweak chain (7),
    /// 6 too (7) → 1 + 20×10 = 201 each. The vx > 0 X-flip and the
    /// Next(2) halving are mirrored per dust.
    #[test]
    fn w4_e4_wolf_and_witch_rings_mirror_the_flip_and_halve() {
        let w = world();
        let c = ctx();
        for dtype in [228i32, 6] {
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(107);
            let mut mpool = DustPool::new();
            let mut mirror = UnifiedRandom::new(107);
            if dtype == 228 {
                wolf_dash_ring(Vec2::new(900.0, 900.0), 20.0, 1, 5.0, 6.0, 8.0, &mut rng, &mut pool, &c, &w);
            } else {
                witch_dash_ring(Vec2::new(900.0, 900.0), 20.0, 1, 5.0, 6.0, 8.0, &mut rng, &mut pool, &c, &w);
            }
            // Mirror: phase, then per dust the internals + gate + rolls.
            let phase = std::f32::consts::PI * 2.0 * mirror.next_float();
            let slots = active(&pool);
            assert_eq!(slots.len(), 20);
            for (k, &i) in slots.iter().enumerate() {
                let _idx = mpool.new_dust(&mut mirror, &c, &w, Vec2::new(920.0, 900.0), 0, 0, dtype, 0.0, 0.0, 0, Color::default(), 1.0);
                let angle = k as f32 * (std::f32::consts::PI * 2.0) / 20.0 + phase;
                let ring = rotated_by(Vec2::new(0.0, 1.0), angle as f64) * 0.8;
                let mut vel = ring + Vec2::new(8.0 - 1.0, 0.0);
                vel.x *= -1.0; // vx > 0
                if mirror.next_max(2) == 0 {
                    vel *= 0.5;
                }
                let _scale = mirror.next_float();
                let d = pool.slot(i);
                assert_eq!(d.dust_type, dtype);
                assert_eq!(d.velocity, vel);
                assert_eq!(d.position, Vec2::new(920.0, 900.0) + ring * Vec2::new(40.0, 30.0));
                assert_eq!(d.fade_in, 0.0);
            }
            assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
        }
    }

    /// W4-e4 — site 4023: the SelectRandom draw precedes the internals;
    /// the Next(2)/Next(3) gates and the type-31 forced writes mirror.
    /// Twin-pool parity over the whole arm.
    #[test]
    fn w4_e4_witch_puff_type31_forces_the_last_writes() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(109);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(109);
        witch_gated_puff(Vec2::new(900.0, 900.0), 4.0, -2.0, &mut rng, &mut pool, &c, &w);
        let num19 = [6, 6, 31][mirror.next_max(3) as usize];
        let idx = mpool.new_dust(&mut mirror, &c, &w, Vec2::new(894.0, 906.0), 12, 12, num19, 0.0, 0.0, 0, Color::default(), 1.0);
        let g2 = mirror.next_max(2) == 0;
        let sf = mirror.next_float();
        let g3 = mirror.next_max(3) == 0;
        let d = pool.slot(0);
        let md = mpool.slot(idx);
        assert_eq!(d.dust_type, num19);
        assert_eq!(d.velocity, Vec2::new(0.4, -0.2));
        let mut scale = 0.7 + sf * 0.8;
        let mut fade = if g3 { 0.1 } else { md.fade_in };
        let mut nograv = g2;
        if num19 == 31 {
            nograv = true;
            scale *= 1.5;
            fade = 0.2;
        }
        assert_eq!(d.no_gravity, nograv);
        assert_eq!(d.scale, scale);
        assert_eq!(d.fade_in, fade);
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e4 — sites 4103/4123: the puffs spend 2 × (6 + 3) = 18 (31
    /// outside the chain); the trail at charge 1 runs 3 iterations of
    /// (7 internals + 1 pos float + 1 speed float + conditional Y float
    /// + 2 shape floats), the SafeNormalize aim and the charge folds
    /// mirrored. The two emitters stay split for the FloorVisuals seam.
    #[test]
    fn w4_e4_basilisk_puffs_then_trail_fold_the_charge() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(113);
        basilisk_charge_puffs(Vec2::new(880.0, 940.0), 20, 6.0, &mut rng, &mut pool, &c, &w);
        assert_eq!(active(&pool).len(), 2);
        let mut bare = UnifiedRandom::new(113);
        for _ in 0..18 {
            bare.next();
        }
        assert_eq!(rng.next_range(0, 1000), bare.next_range(0, 1000));
        // The trail: charge 1 → 3 dusts; twin-pool parity.
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(113);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(113);
        basilisk_charge_trail(Vec2::new(880.0, 940.0), Vec2::new(900.0, 920.0), 20, 1.0, 6.0, 0.0, 1.0, 8.0, &mut rng, &mut pool, &c, &w);
        let slots = active(&pool);
        assert_eq!(slots.len(), 3);
        let target = Vec2::new(940.0, 932.0);
        for &i in &slots {
            let _idx = mpool.new_dust(&mut mirror, &c, &w, Vec2::new(880.0, 940.0), 20, 6, 6, 0.0, 0.0, 0, Color::default(), 1.0);
            let pos = Vec2::new(900.0 + 38.0, 920.0 + 6.0 + mirror.next_float() * 12.0);
            let speed = 3.5 + mirror.next_float() * 0.5;
            let mut vel = safe_normalize(pos - target, Vec2::ZERO) * speed;
            if vel.y < 0.0 {
                vel.y *= 1.0 + 2.0 * mirror.next_float();
            }
            vel += Vec2::new(6.0, 0.0) * 0.55;
            vel *= 6.0f32 / 8.0;
            vel *= 1.0;
            let _sf = mirror.next_float();
            let _ff = mirror.next_float();
            let d = pool.slot(i);
            assert_eq!(d.position, pos);
            assert_eq!(d.velocity, vel);
            assert!(d.no_gravity && d.no_light);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }
}
