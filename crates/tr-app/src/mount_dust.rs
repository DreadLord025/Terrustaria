//! W4-e4 — the world dust sites' SECOND batch, the Mount.cs lane:
//! `Dust.NewDust*` sites 1-19 (lines 3113-5071, research/66's claim doc;
//! batches 3-4). The orchestrator sites 41-61 live in
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
//! - 4626 — `EmitGolfCartSmoke` (`:4623-4639`).
//! - 4703/4719/4750/4782 — the gem/steam/exhaust/confetti minecart
//!   effects (`:4697-4793`), the RotatedRelativePoint anchors ride the
//!   port's `Player::rotated_relative_point`.
//! - 4824/4849 — `UpdateEffects` case 23's witch-broom trinket sparkle
//!   + trail (`:4815-4856`), the RNG-free sway helper ported with it.
//! - 5061/5071 — case 11's drill-mount lerp trail (`:5050-5077`).
//! WRITTEN NOT RUN (code-only mode): the witnesses below are compiled by
//! the merge-verify wave, not executed here.

#![allow(dead_code)] // unwired until the mount frame/effects drains land (code-only mode)

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};
use crate::bow_launch::{rotated_by, safe_normalize};
use crate::orchestrator_dust::{next_vector2_circular, next_vector2_square, to_rotation_vector2};


/// `Math.Sign(float)` (`System.Math`): −1 / 0 / +1 as an INTEGER — the
/// zero case matters (Rust's `f32::signum` returns ±1.0 for ±0.0, the
/// C# returns 0; the dash kicks read it at rest).
pub(crate) fn math_sign(v: f32) -> i32 {
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
pub(crate) fn unicorn_dash_body_v1(
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
fn dash_ring_20_v1(
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
    dash_ring_20_v1(228, centre, width, direction, vx, speed_len, run_speed, rng, dust, ctx, world);
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
    dash_ring_20_v1(6, centre, width, direction, vx, speed_len, run_speed, rng, dust, ctx, world);
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

/// Site 4626 — `EmitGolfCartSmoke` (`Mount.cs:4623-4639`). `gravDir` is
/// the vanilla 1f (the port has no gravity flip — d51's cited
/// convention), so `Directions = (direction, 1)`. Position = Bottom +
/// (−direction×34, −12); NewDustDirect 0×0 type 31, speedX = −direction,
/// speedY = −0.24, alpha 100; then position pinned, velocity ×0.1 +=
/// (−direction, −0.25), scale 0.5; vx != 0 adds Sign(vx)×1.3 X; rushing
/// sets fadeIn 0.8.
pub(crate) fn golf_cart_smoke_v1(
    bottom: Vec2,
    direction: i32,
    pvx: f32,
    rushing: bool,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let position = bottom + Vec2::new(-direction as f32 * 34.0, -12.0); // `:4625`
    let idx = dust.new_dust_direct(
        rng, ctx, world, position, 0, 0, 31,
        -direction as f32, -0.24, 100, Color::default(), 1.0,
    ); // `:4626`
    let d = dust.slot_mut(idx);
    d.position = position; // `:4627`
    d.velocity *= 0.1; // `:4628`
    d.velocity += Vec2::new(-direction as f32, -0.25); // `:4629`
    d.scale = 0.5; // `:4630`
    if pvx != 0.0 {
        d.velocity += Vec2::new(math_sign(pvx) as f32 * 1.3, 0.0); // `:4631-4634`
    }
    if rushing {
        d.fade_in = 0.8; // `:4635-4638`
    }
}

/// Site 4703 — `DoGemMinecartEffect` (`Mount.cs:4697-4710`). The
/// `Next(10) == 0` gate spends its draw either way; a pass draws
/// `NextVector2Square(-1,1)` (TWO NextDouble, X first) × (22,10), then
/// NewDustPerfect at `RotatedRelativePoint(Center + (0,10)*Directions +
/// jitter)` — the rotation is the caller's `Player` method (the random
/// offset is INSIDE it), type = the gem row (86/87/88/89…), noGravity,
/// fadeIn 0.6, scale 0.4, velocity ×0.25. Returns whether it fired.
pub(crate) fn gem_minecart_effect(
    p: &crate::player::Player,
    dust_type: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) -> bool {
    if rng.next_max(10) != 0 {
        return false; // `:4699`
    }
    let jitter = next_vector2_square(rng, -1.0, 1.0) * Vec2::new(22.0, 10.0); // `:4701`
    let dirs = Vec2::new(p.facing as f32, 1.0); // `:4702` — gravDir 1.
    let (cx, cy) = p.center();
    let anchor = p.rotated_relative_point(Vec2::new(cx, cy) + Vec2::new(0.0, 10.0) * dirs + jitter, 0);
    let idx = dust.new_dust_perfect(rng, ctx, world, anchor, dust_type, None, 0, Color::default(), 1.0);
    let d = dust.slot_mut(idx);
    d.no_gravity = true; // `:4704`
    d.fade_in = 0.6; // `:4705`
    d.scale = 0.4; // `:4706`
    d.velocity *= 0.25; // `:4707`
    true
}

/// Site 4719 — `DoSteamMinecartEffect` (`Mount.cs:4712-4727`). The gate
/// is DRAW-FREE: `|vx| >= 1 && (|vx| >= 6 || frame == 0)` (the C#'s
/// `!(num < 1f) && (!(num < 6f) || this._frame == 0)`). A pass draws the
/// square jitter × (3,3), anchor = RotatedRelativePoint(Center +
/// (−10,−4)*Directions + jitter); noGravity, fadeIn 0.6, scale 1.8,
/// velocity ×0.25 then Y −= 2. Returns whether it fired.
pub(crate) fn steam_minecart_effect(
    p: &crate::player::Player,
    frame: i32,
    dust_type: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) -> bool {
    let num = p.vx.abs(); // `:4714`
    if num < 1.0 || (num < 6.0 && frame != 0) {
        return false; // `:4715`
    }
    let jitter = next_vector2_square(rng, -1.0, 1.0) * Vec2::new(3.0, 3.0); // `:4717`
    let dirs = Vec2::new(p.facing as f32, 1.0);
    let (cx, cy) = p.center();
    let anchor = p.rotated_relative_point(Vec2::new(cx, cy) + Vec2::new(-10.0, -4.0) * dirs + jitter, 0);
    let idx = dust.new_dust_perfect(rng, ctx, world, anchor, dust_type, None, 0, Color::default(), 1.0);
    let d = dust.slot_mut(idx);
    d.no_gravity = true; // `:4720`
    d.fade_in = 0.6; // `:4721`
    d.scale = 1.8; // `:4722`
    d.velocity *= 0.25; // `:4723`
    d.velocity.y -= 2.0; // `:4724`
    true
}

/// Site 4750 — `DoExhaustMinecartEffect` (`Mount.cs:4729-4761`). `num` =
/// |velocity|; the gate draws `Next(4)` ONLY when num < 1 (the `&&`
/// short-circuit, `:4732`). `num2 = 1 + (int)num / 6` — the cast binds
/// BEFORE the division (C# `(int)num / 6`); the while loop decrements
/// FIRST, then per puff: square jitter × (3,3), base (−18,20)*Directions,
/// num > 6 → X += 4×direction, num2 > 0 → += velocity × (num2 / 3)
/// (INTEGER division), anchor = RotatedRelativePoint(Center + base +
/// jitter); noGravity, fadeIn 0.6, scale 1.2, velocity ×0.2, num < 1 →
/// X −= 0.5×direction.
pub(crate) fn exhaust_minecart_effect(
    p: &crate::player::Player,
    dust_type: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let vel = Vec2::new(p.vx, p.vy);
    let num = vel.length(); // `:4731`
    if num < 1.0 && rng.next_max(4) != 0 {
        return; // `:4732-4735`
    }
    let mut num2 = 1 + (num as i32) / 6; // `:4736`
    let dirs = Vec2::new(p.facing as f32, 1.0);
    let (cx, cy) = p.center();
    while num2 > 0 {
        num2 -= 1; // `:4739`
        let jitter = next_vector2_square(rng, -1.0, 1.0) * Vec2::new(3.0, 3.0); // `:4740`
        let mut base = Vec2::new(-18.0, 20.0) * dirs; // `:4741`
        if num > 6.0 {
            base.x += 4.0 * p.facing as f32; // `:4742-4745`
        }
        if num2 > 0 {
            base += vel * (num2 / 3) as f32; // `:4746-4749` — integer /3.
        }
        let anchor = p.rotated_relative_point(Vec2::new(cx, cy) + base + jitter, 0);
        let idx = dust.new_dust_perfect(rng, ctx, world, anchor, dust_type, None, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.no_gravity = true; // `:4751`
        d.fade_in = 0.6; // `:4752`
        d.scale = 1.2; // `:4753`
        d.velocity *= 0.2; // `:4754`
        if num < 1.0 {
            d.velocity.x -= 0.5 * p.facing as f32; // `:4755-4758`
        }
    }
}

/// Site 4782 — `DoConfettiMinecartEffect` (`Mount.cs:4763-4793`). The
/// gate is TWO short-circuit draws: `(num < 1 && Next(6) != 0) ||
/// (num < 3 && Next(3) != 0)` (`:4766`) — the second draw happens only
/// when the first clause is false. num2 = 1 + (int)num/6; per puff
/// (decrement first): num3 = NextFloat×2, square jitter × (3,8), base
/// (−18,4)*Directions, X += num×direction×0.5 + (direction×num2)×num3,
/// num2 > 0 → += velocity×(num2/3); type = 139 + Next(4) drawn as the
/// NewDustPerfect TYPE ARGUMENT; noGravity, fadeIn 0.6, scale = 0.5 +
/// num3/2, velocity ×0.2, num < 1 → X −= 0.5×direction.
pub(crate) fn confetti_minecart_effect(
    p: &crate::player::Player,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let vel = Vec2::new(p.vx, p.vy);
    let num = vel.length(); // `:4765`
    if (num < 1.0 && rng.next_max(6) != 0) || (num < 3.0 && rng.next_max(3) != 0) {
        return; // `:4766-4769`
    }
    let mut num2 = 1 + (num as i32) / 6; // `:4770`
    let dirs = Vec2::new(p.facing as f32, 1.0);
    let (cx, cy) = p.center();
    while num2 > 0 {
        num2 -= 1; // `:4773`
        let num3 = rng.next_float() * 2.0; // `:4774`
        let jitter = next_vector2_square(rng, -1.0, 1.0) * Vec2::new(3.0, 8.0); // `:4775`
        let mut base = Vec2::new(-18.0, 4.0) * dirs; // `:4776`
        base.x += num * p.facing as f32 * 0.5 + (p.facing * num2) as f32 * num3; // `:4777`
        if num2 > 0 {
            base += vel * (num2 / 3) as f32; // `:4778-4781`
        }
        let anchor = p.rotated_relative_point(Vec2::new(cx, cy) + base + jitter, 0);
        let dtype = 139 + rng.next_max(4); // `:4782` — the type argument.
        let idx = dust.new_dust_perfect(rng, ctx, world, anchor, dtype, None, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.no_gravity = true; // `:4783`
        d.fade_in = 0.6; // `:4784`
        d.scale = 0.5 + num3 / 2.0; // `:4785`
        d.velocity *= 0.2; // `:4786`
        if num < 1.0 {
            d.velocity.x -= 0.5 * p.facing as f32; // `:4787-4790`
        }
    }
}

/// `Mount.GetWitchBroomTrinketRotation` (`Mount.cs:3562-3574`) — the
/// broom's sway angle, RNG-FREE. `num` = Clamp(vx/10, −1, 1); `num3` =
/// 0.5, or 1 when the centre tile is in a place with wind (the caller's
/// probe); `num2` = the wind sway (sin of miscCounter/300 × 2π × 3, ×π/4
/// × |WindForVisuals| × 0.5, plus π/4 × −WindForVisuals × 0.5); the
/// return is num × sin(miscCounter/150 × 2π × 3) × π/4 × 0.5 + num ×
/// π/4 × 0.5 + num2 × num3. All the sin arguments are f32 chains widened
/// for Math.Sin.
pub(crate) fn witch_broom_trinket_rotation_v1(vx: f32, misc_counter: u32, wind: f32, in_wind: bool) -> f32 {
    let num = (vx / 10.0).clamp(-1.0, 1.0); // `:3564`
    let num3 = if in_wind { 1.0f32 } else { 0.5 }; // `:3567-3571`
    let pi = std::f32::consts::PI;
    let a300 = misc_counter as f32 / 300.0 * (pi * 2.0) * 3.0;
    let num2 = (a300 as f64).sin() as f32 * (pi / 4.0) * wind.abs() * 0.5
        + pi / 4.0 * (0.0 - wind) * 0.5; // `:3572`
    let a150 = misc_counter as f32 / 150.0 * (pi * 2.0) * 3.0;
    num * (a150 as f64).sin() as f32 * (pi / 4.0) * 0.5 + num * (pi / 4.0) * 0.5 + num2 * num3 // `:3573`
}

/// `Utils.DirectionTo` (`Utils.cs:1955-1958`) — `Normalize(Target −
/// Origin)`; the zero vector stays zero (XNA Normalize), the town
/// module's convention.
fn direction_to_v1(origin: Vec2, target: Vec2) -> Vec2 {
    (target - origin).normalize_or_zero()
}

/// Sites 4824/4849 — `UpdateEffects` case 23's witch-broom trinket
/// (`Mount.cs:4815-4856`). `origin` = RotatedRelativePoint(Center +
/// (29×direction, −4) + (rotation + π/2).ToRotationVector2()×11) — the
/// caller computes it from [`witch_broom_trinket_rotation_v1`] (the
/// AddLight at `:4820` is the lights lane's). The `Next(45) == 0` gate
/// (`:4821`) spends its draw either way; a pass draws Circular(4,4),
/// NewDustPerfect 43 at origin+jitter, velocity Zero ARGUMENT, alpha
/// 254, colour (255,255,0,255), scale 0.3; jitter != Zero → velocity =
/// origin.DirectionTo(position)×0.2; fadeIn 0.3, noLightEmittance.
/// Then the trail head: num12 = 0.1 + |velocity|/30 (`:4834`), pos4 =
/// RotatedRelativePoint(Center + (18 − 20×NextFloat×direction, 12)),
/// pos5 = RotatedRelativePoint(Center + (52×direction, −6)); the
/// `NextFloat <= num12` gate (`:4839`) draws even on failure; a pass
/// draws num13 = NextFloat, then 8 steps of 0.125: `Next(15) == 0` →
/// ring = ((2π×num14 + num13).ToRotationVector2() × (0.5,1) × 4)
/// .RotatedBy(fullRotation), NewDustPerfect 43 at pos4+ring (same
/// alpha/colour/scale), velocity = ring×0.025 + pos5.DirectionTo(
/// position)×0.5, fadeIn 0.3, noLightEmittance.
#[allow(clippy::too_many_arguments)]
pub(crate) fn witch_broom_trinket(
    p: &crate::player::Player,
    origin: Vec2,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let yellow = Color::new(255, 255, 0, 255);
    if rng.next_max(45) == 0 {
        // `:4821-4833`
        let jitter = next_vector2_circular(rng, 4.0, 4.0); // `:4823`
        let idx = dust.new_dust_perfect(rng, ctx, world, origin + jitter, 43, Some(Vec2::ZERO), 254, yellow, 0.3);
        let d = dust.slot_mut(idx);
        if jitter != Vec2::ZERO {
            d.velocity = direction_to_v1(origin, d.position) * 0.2; // `:4825-4828`
        }
        d.fade_in = 0.3; // `:4829`
        d.no_light_emittance = true; // `:4830`
    }
    let num12 = 0.1 + Vec2::new(p.vx, p.vy).length() / 30.0; // `:4834`
    let (cx, cy) = p.center();
    let centre = Vec2::new(cx, cy);
    let off4 = Vec2::new(18.0 - 20.0 * rng.next_float() * p.facing as f32, 12.0); // `:4835`
    let pos4 = p.rotated_relative_point(centre + off4, 0); // `:4837`
    let pos5 = p.rotated_relative_point(centre + Vec2::new(52.0 * p.facing as f32, -6.0), 0); // `:4836-4838`
    if !(rng.next_float() <= num12) {
        return; // `:4839-4842`
    }
    let num13 = rng.next_float(); // `:4843`
    let mut num14 = 0.0f32;
    while num14 < 1.0 {
        if rng.next_max(15) == 0 {
            // `:4846-4854`
            let ring = rotated_by(
                to_rotation_vector2(std::f32::consts::PI * 2.0 * num14 + num13) * Vec2::new(0.5, 1.0) * 4.0,
                p.full_rotation as f64,
            );
            let idx = dust.new_dust_perfect(rng, ctx, world, pos4 + ring, 43, Some(Vec2::ZERO), 254, yellow, 0.3);
            let d = dust.slot_mut(idx);
            d.velocity = ring * 0.025 + direction_to_v1(pos5, d.position) * 0.5; // `:4850`
            d.fade_in = 0.3; // `:4851`
            d.no_light_emittance = true; // `:4852`
        }
        num14 += 0.125;
    }
}

/// Sites 5061/5071 — `UpdateEffects` case 11's drill-mount trail
/// (`Mount.cs:5050-5077`). The caller computes the two anchors from the
/// C#'s num9/num8 fold (`:5035-5051`): `current` = Bottom + vector6 +
/// vector7, `old` = oldPosition + Size×(0.5,1) + vector6 + vector7.
/// Distance > 3 → num10 = (int)distance/3, +1 when distance % 3 != 0
/// (`:5054-5058`), then num10 dusts lerping old→current at
/// num11/(float)num10; else ONE dust at `current`. Every dust: type 182
/// at Center, position overwritten, noGravity, velocity Zero.
pub(crate) fn drill_mount_trail_v1(
    centre: Vec2,
    current: Vec2,
    old: Vec2,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let dist = current.distance(old); // `:5052`
    if dist > 3.0 {
        let mut num10 = dist as i32 / 3; // `:5054`
        if dist % 3.0 != 0.0 {
            num10 += 1; // `:5055-5058`
        }
        let mut num11 = 1.0f32;
        while num11 <= num10 as f32 {
            let idx = dust.new_dust(rng, ctx, world, centre, 0, 0, 182, 0.0, 0.0, 0, Color::default(), 1.0);
            let d = dust.slot_mut(idx);
            d.position = old.lerp(current, num11 / num10 as f32); // `:5062`
            d.no_gravity = true; // `:5063`
            d.velocity = Vec2::ZERO; // `:5064`
            num11 += 1.0;
        }
    } else {
        let idx = dust.new_dust(rng, ctx, world, centre, 0, 0, 182, 0.0, 0.0, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.position = current; // `:5072`
        d.no_gravity = true; // `:5073`
        d.velocity = Vec2::ZERO; // `:5074`
    }
}

/// W4-e4 — the player-tick mount-dust drain: resolves one
/// [`crate::mount::MountDustRequest`] into its emitter on the caller's
/// `UpdatePlayers` stream. The state machines ran inline in
/// `MountState::update_frame_extras` / `Player::update_mount`; the emitter
/// bodies roll here (the App owns the dust pool). The `&Player` snapshot is
/// the tick-moment one the minecart/trinket anchors read
/// (`rotated_relative_point`); the WoF goat's tile probe re-reads `world`.
pub(crate) fn drain_mount_request(
    req: crate::mount::MountDustRequest,
    p: &crate::player::Player,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    use crate::mount::MountDustRequest::*;
    match req {
        UnicornRing { mount_type, centre, width, direction, vx, run_speed, frame_extra } => {
            unicorn_dash_ring(mount_type, centre, width, direction, vx, run_speed, frame_extra, rng, dust, ctx, world);
        }
        UnicornBody { position, width, height } => {
            unicorn_dash_body_v1(position, width, height, rng, dust, ctx, world);
        }
        WolfRing { centre, width, direction, vx, speed_len, run_speed } => {
            wolf_dash_ring(centre, width, direction, vx, speed_len, run_speed, rng, dust, ctx, world);
        }
        WolfPuff { bottom, width, vx, run_speed } => {
            wolf_ground_puff(bottom, width, vx, run_speed, rng, dust, ctx, world);
        }
        WitchRing { centre, width, direction, vx, speed_len, run_speed } => {
            witch_dash_ring(centre, width, direction, vx, speed_len, run_speed, rng, dust, ctx, world);
        }
        WitchPuff { centre, pvx, pvy } => {
            witch_gated_puff(centre, pvx, pvy, rng, dust, ctx, world);
        }
        BasiliskPuffs { bottom_left, width, vx } => {
            basilisk_charge_puffs(bottom_left, width, vx, rng, dust, ctx, world);
        }
        BasiliskTrail { bottom_left, centre, width, sign, pvx, pvy, charge, run_speed } => {
            basilisk_charge_trail(bottom_left, centre, width, sign, pvx, pvy, charge, run_speed, rng, dust, ctx, world);
        }
        WofGoatFlames { tx, ty, position } => {
            // `Mount.cs:3907-3908` — the tile-below probe: active, dry,
            // SolidTileAllowBottomSlope. A pass spawns the goat flames.
            let pass = world.tile(tx, ty).is_some_and(|t| {
                t.active()
                    && t.liquid == 0
                    && crate::projectile_ai026_c::solid_tile_allow_bottom_slope(world, tx, ty)
            });
            if pass {
                let s = crate::orchestrator_dust::OrchestraSettings {
                    position,
                    movement: Vec2::ZERO,
                    info: 0,
                    invoker: 0,
                };
                crate::orchestrator_dust::wof_goat_mount_flames(&s, rng, dust, ctx, world);
            }
        }
        WitchBroom { wind_speed, in_wind } => {
            // `Mount.cs:4817-4819` — the origin: Center + (29×dir, −4) +
            // (rotation + π/2).ToRotationVector2()×11, then
            // RotatedRelativePoint. The rotation is the RNG-free sway helper.
            let rot = witch_broom_trinket_rotation_v1(p.vx, p.misc_counter, wind_speed, in_wind);
            let (cx, cy) = p.center();
            let centre = Vec2::new(cx, cy);
            let offset = Vec2::new(29.0 * p.facing as f32, -4.0);
            let spin = to_rotation_vector2(rot + std::f32::consts::PI / 2.0) * 11.0;
            let origin = p.rotated_relative_point(centre + offset + spin, 0);
            witch_broom_trinket(p, origin, rng, dust, ctx, world);
        }
        GemMinecart { dust_type } => {
            gem_minecart_effect(p, dust_type, rng, dust, ctx, world);
        }
        SteamMinecart { dust_type } => {
            steam_minecart_effect(p, p.mount.frame(), dust_type, rng, dust, ctx, world);
        }
        ExhaustMinecart { dust_type } => {
            exhaust_minecart_effect(p, dust_type, rng, dust, ctx, world);
        }
        ConfettiMinecart => {
            confetti_minecart_effect(p, rng, dust, ctx, world);
        }
        DrillTrail => {
            // `Mount.cs:5035-5051` — the num9/num8 fold + the two rotated
            // offsets, then the lerp trail. num8 = Sign(vx) (or direction at
            // rest); num9 = −24 (−22 when direction != num8, +1 when num8
            // == −1). vector6 = (num9×num8, −19).RotatedBy(fullRotation);
            // vector7 = the two Lerps of fullRotation/(π/4), rotated, scaled
            // by Lerp(1, 0.6, |…|) when num8 == Sign(fullRotation).
            let mut num8 = math_sign(p.vx);
            if num8 == 0 {
                num8 = p.facing;
            }
            let mut num9 = -24.0f32;
            if p.facing != num8 {
                num9 = -22.0;
            }
            if num8 == -1 {
                num9 += 1.0;
            }
            let fr = p.full_rotation;
            let vector6 = rotated_by(Vec2::new(num9 * num8 as f32, -19.0), fr as f64);
            let t = fr / (std::f32::consts::PI / 4.0);
            let mut vector7 = rotated_by(
                Vec2::new(lerp_f32(0.0, -8.0, t), lerp_f32(0.0, 2.0, t.abs())),
                fr as f64,
            );
            if num8 == math_sign(fr) {
                vector7 *= lerp_f32(1.0, 0.6, t.abs());
            }
            let (cx, cy) = p.center();
            let centre = Vec2::new(cx, cy);
            let (bx, by) = p.bottom();
            let bottom = Vec2::new(bx, by);
            let current = bottom + vector6 + vector7;
            let old = Vec2::new(p.old_x, p.old_y)
                + Vec2::new(p.width * 0.5, p.height)
                + vector6
                + vector7;
            drill_mount_trail_v1(centre, current, old, rng, dust, ctx, world);
        }
    }
}

/// `MathHelper.Lerp` (`MonoGame MathHelper.cs`): `value1 + (value2 − value1)
/// × amount` in f32 — the drill-mount fold's two Lerps (`Mount.cs:5045`).
fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
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
    /// (internals + Next(2) + 1 scale float); 228 rides the tweak chain
    /// (7), 6 too (7), fadeIn is the constant 0 (no draw) → 1 + 20×9 =
    /// 181 each. The vx > 0 X-flip and the Next(2) halving are mirrored
    /// per dust (twin-pool parity, so the count is implicit).
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

    /// A player parked so `rotated_relative_point` is the identity:
    /// fullRotation 0, gfxOffY 0, mount inactive (PlayerOffset 0 → the
    /// (0,−4)+(0,4) pivot cancels). center = (x+10, y+21).
    fn parked_player(w: &World) -> crate::player::Player {
        let mut p = crate::player::Player::spawn(w);
        p.x = 890.0;
        p.y = 879.0; // center = (900, 900)
        p.vx = 0.0;
        p.vy = 0.0;
        p.facing = 1;
        p.full_rotation = 0.0;
        p.gfx_off_y = 0.0;
        p
    }

    /// W4-e4 — site 4626: the golf-cart smoke pins position, folds the
    /// velocity ×0.1 then adds the (−dir, −0.25) kick + the Sign(vx)×1.3
    /// X nudge, scale 0.5, rushing fadeIn 0.8. Twin-pool parity.
    #[test]
    fn w4_e4_golf_cart_smoke_folds_and_kicks() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(127);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(127);
        let bottom = Vec2::new(900.0, 940.0);
        golf_cart_smoke_v1(bottom, 1, 5.0, true, &mut rng, &mut pool, &c, &w);
        let position = bottom + Vec2::new(-34.0, -12.0);
        let idx = mpool.new_dust_direct(&mut mirror, &c, &w, position, 0, 0, 31, -1.0, -0.24, 100, Color::default(), 1.0);
        let md = mpool.slot_mut(idx);
        md.position = position;
        md.velocity *= 0.1;
        md.velocity += Vec2::new(-1.0, -0.25);
        md.scale = 0.5;
        md.velocity += Vec2::new(1.3, 0.0); // Sign(5) = +1
        md.fade_in = 0.8;
        let d = pool.slot(0);
        assert_eq!(d.dust_type, 31);
        assert_eq!(d.position, position);
        assert_eq!(d.velocity, md.velocity);
        assert_eq!(d.scale, 0.5);
        assert_eq!(d.fade_in, 0.8);
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e4 — site 4703: the Next(10) gate spends its draw either way;
    /// a pass draws the square jitter (2 NextDouble) then the Perfect
    /// internals; the identity rotation pins the anchor at center +
    /// (0,10) + jitter×(22,10); noGravity/fadeIn0.6/scale0.4/vel×0.25.
    #[test]
    fn w4_e4_gem_minecart_gates_then_jitters() {
        let w = world();
        let c = ctx();
        let p = parked_player(&w);
        // Find a seed whose Next(10) gate passes.
        let mut seed = 0i32;
        loop {
            let mut probe = UnifiedRandom::new(seed);
            if probe.next_max(10) == 0 {
                break;
            }
            seed += 1;
        }
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(seed);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(seed);
        assert!(gem_minecart_effect(&p, 86, &mut rng, &mut pool, &c, &w));
        let _gate = mirror.next_max(10);
        let jitter = next_vector2_square(&mut mirror, -1.0, 1.0) * Vec2::new(22.0, 10.0);
        let (cx, cy) = p.center();
        let anchor = Vec2::new(cx, cy) + Vec2::new(0.0, 10.0) + jitter;
        let idx = mpool.new_dust_perfect(&mut mirror, &c, &w, anchor, 86, None, 0, Color::default(), 1.0);
        let d = pool.slot(0);
        assert_eq!(d.dust_type, 86);
        assert_eq!(d.position, anchor);
        assert!(d.no_gravity);
        assert_eq!(d.fade_in, 0.6);
        assert_eq!(d.scale, 0.4);
        assert_eq!(d.velocity, mpool.slot(idx).velocity * 0.25);
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e4 — site 4782: the confetti gate is two short-circuit draws;
    /// at |v| >= 3 neither gate draw fires, num2 = 1 + (int)|v|/6 puffs,
    /// each draws num3 + square + the 139+Next(4) type argument. The
    /// scale = 0.5 + num3/2 and the X nudge mirror.
    #[test]
    fn w4_e4_confetti_minecart_rolls_the_type_per_puff() {
        let w = world();
        let c = ctx();
        let mut p = parked_player(&w);
        p.vx = 8.0; // |v| = 8 ≥ 3 → no gate draws; num2 = 1 + 8/6 = 2.
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(131);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(131);
        confetti_minecart_effect(&p, &mut rng, &mut pool, &c, &w);
        let slots = active(&pool);
        assert_eq!(slots.len(), 2);
        let (cx, cy) = p.center();
        let centre = Vec2::new(cx, cy);
        let vel = Vec2::new(8.0, 0.0);
        let mut num2 = 2i32;
        for &i in &slots {
            num2 -= 1;
            let num3 = mirror.next_float() * 2.0;
            let jitter = next_vector2_square(&mut mirror, -1.0, 1.0) * Vec2::new(3.0, 8.0);
            let mut base = Vec2::new(-18.0, 4.0);
            base.x += 8.0 * 0.5 + (num2) as f32 * num3;
            if num2 > 0 {
                base += vel * (num2 / 3) as f32;
            }
            let anchor = centre + base + jitter;
            let dtype = 139 + mirror.next_max(4);
            let idx = mpool.new_dust_perfect(&mut mirror, &c, &w, anchor, dtype, None, 0, Color::default(), 1.0);
            let d = pool.slot(i);
            assert_eq!(d.dust_type, dtype);
            assert_eq!(d.position, anchor);
            assert_eq!(d.scale, 0.5 + num3 / 2.0);
            assert_eq!(d.fade_in, 0.6);
            assert_eq!(d.velocity, mpool.slot(idx).velocity * 0.2);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e4 — sites 5061/5071: distance > 3 lerps old→current over
    /// (int)dist/3 (+1 when dist%3 != 0) dusts; else one at current.
    /// Type 182, noGravity, velocity Zero, position pinned. No RNG
    /// beyond the internals (6 each, 182 outside the tweak chain).
    #[test]
    fn w4_e4_drill_mount_trail_lerps_or_snaps() {
        let w = world();
        let c = ctx();
        let centre = Vec2::new(900.0, 900.0);
        // dist = 10 → num10 = 10/3 = 3, 10%3 = 1 != 0 → 4 dusts.
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(137);
        let old = Vec2::new(900.0, 900.0);
        let current = Vec2::new(910.0, 900.0);
        drill_mount_trail_v1(centre, current, old, &mut rng, &mut pool, &c, &w);
        let slots = active(&pool);
        assert_eq!(slots.len(), 4);
        for (k, &i) in slots.iter().enumerate() {
            let t = (k + 1) as f32 / 4.0;
            assert_eq!(pool.slot(i).position, old.lerp(current, t));
            assert_eq!(pool.slot(i).dust_type, 182);
            assert!(pool.slot(i).no_gravity);
            assert_eq!(pool.slot(i).velocity, Vec2::ZERO);
        }
        // dist = 2 ≤ 3 → one dust at current.
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(137);
        drill_mount_trail_v1(centre, Vec2::new(902.0, 900.0), old, &mut rng, &mut pool, &c, &w);
        let slots = active(&pool);
        assert_eq!(slots.len(), 1);
        assert_eq!(pool.slot(slots[0]).position, Vec2::new(902.0, 900.0));
    }

    /// W4-e4 — sites 4824/4849: the witch-broom trinket's Next(45) gate,
    /// the Circular(4,4) jitter + Perfect 43, then the num12 gate +
    /// num13 + 8×Next(15) trail. Twin-pool parity over the whole arm;
    /// the yellow colour / alpha 254 / scale 0.3 / fadeIn 0.3 /
    /// noLightEmittance pin on every spawned dust.
    #[test]
    fn w4_e4_witch_broom_trinket_gates_and_trails() {
        let w = world();
        let c = ctx();
        let mut p = parked_player(&w);
        p.vx = 4.0; // num12 = 0.1 + 4/30 ≈ 0.233
        let origin = Vec2::new(929.0, 896.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(139);
        let mut pool2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(139);
        witch_broom_trinket(&p, origin, &mut rng, &mut pool, &c, &w);
        witch_broom_trinket(&p, origin, &mut rng2, &mut pool2, &c, &w);
        let slots = active(&pool);
        let slots2 = active(&pool2);
        assert_eq!(slots.len(), slots2.len());
        for (&i, &j) in slots.iter().zip(slots2.iter()) {
            let d = pool.slot(i);
            assert_eq!(d.dust_type, 43);
            assert_eq!(d.alpha, 254);
            assert_eq!(d.color, Color::new(255, 255, 0, 255));
            assert_eq!(d.scale, 0.3);
            assert_eq!(d.fade_in, 0.3);
            assert!(d.no_light_emittance);
            assert_eq!(d.position, pool2.slot(j).position);
            assert_eq!(d.velocity, pool2.slot(j).velocity);
        }
        assert_eq!(rng.next_range(0, 1000), rng2.next_range(0, 1000));
    }
}


/// `Math.Sign(float)` as the C# int (-1/0/1 — `f32::signum` differs at 0).
fn sign_i(v: f32) -> i32 {
    if v > 0.0 {
        1
    } else if v < 0.0 {
        -1
    } else {
        0
    }
}

/// Sites 3113/3170 — the drill mount's beam puff pair
/// (`Mount.cs:3106-3116` / `:3163-3173`, identical bodies): per beam hit,
/// `num = (target - player.Center).ToRotation()`, then TWO dusts — each a
/// `Next(2)` side pick (`== 1` → -1 else +1) times π/2 off `num`, a
/// `NextDouble() * 2 + 2` speed, the cos/sin velocity, one
/// `NewDust(target, 0, 0, 230, vx, vy)` and `noGravity = true`. The
/// `customData = mountedPlayer` write is the dropped field. `target` is
/// the beam's tile centre (`(point.X << 4) + 8`, the caller's).
pub fn drill_beam_dust(
    target: Vec2,
    player_center: Vec2,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:3107` — `(vector - mountedPlayer.Center).ToRotation()`:
    // Utils.cs:1655-1658 `(float)Math.Atan2(v.Y, v.X)` — Math.Atan2 is
    // DOUBLE of the widened floats, cast back (e4 to_rotation precedent,
    // orchestrator_dust.rs:76-78).
    let d = target - player_center;
    let num = (d.y as f64).atan2(d.x as f64) as f32;
    for _ in 0..2 {
        // `:3110` — the Next(2) side pick, the f32 π/2 offset.
        let side = if rng.next_max(2) == 1 { -1.0 } else { 1.0 };
        let num2 = num + side * (std::f32::consts::PI / 2.0);
        // `:3111` — `(float)NextDouble() * 2f + 2f` (cast FIRST, f32 maths).
        let num3 = rng.next_double() as f32 * 2.0 + 2.0;
        // `:3112` — the double cos/sin of the f32 angle.
        let v = Vec2::new(
            (num2 as f64).cos() as f32 * num3,
            (num2 as f64).sin() as f32 * num3,
        );
        // `:3113` — NewDust(vector, 0, 0, 230, v.X, v.Y).
        let idx = dust.new_dust(
            rng, ctx, world, target, 0, 0, 230, v.x, v.y, 0, Color::default(), 1.0,
        );
        dust.slot_mut(idx).no_gravity = true; // `:3114`
        // `:3115` — customData = mountedPlayer (dropped field).
    }
}

/// Site 3878 — the Unicorn (10) / WoF-Goat (47) dash ring
/// (`Mount.cs:3866-3892`): inside `(type == 10 || type == 47) & flag8`
/// (the dash-start window, `_frameExtra < 6` before the increment — the
/// caller's state machine). `type` is 6, or the Unicorn's
/// `SelectRandom(176, 177, 179)` (ONE `Next(3)` before the loop);
/// `num21 = 2π * NextFloat()` also before the loop. 14 dusts: each a
/// `NewDust(anchor, 0, 0, type)` whose position/velocity are then
/// overwritten — `vector11 = UnitY.RotatedBy(i * 2π/14 + num21) *
/// (0.2 * frameExtra)`, `position = anchor + vector11 * (40, 30)`,
/// `velocity = vector11 + (RunSpeed - Sign(vx) * frameExtra * 2, 0)`;
/// `noGravity`, the 47's `noLightEmittance`, `scale = 1 + NextFloat()*0.8`,
/// `fadeIn = NextFloat()*2`. `frame_extra` is the POST-increment value
/// (`:3860` runs before the ring). `anchor` = `Center + (width*direction,
/// 0)` (the caller's, `:3873`).
pub fn unicorn_goat_dash_ring(
    kind: i32,
    anchor: Vec2,
    frame_extra: i32,
    run_speed: f32,
    vx: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:3868-3872` — the type pick (the Unicorn only).
    let mut ty = 6;
    if kind == 10 {
        ty = rng.next_from_slice(&[176, 177, 179]);
    }
    // `:3875` — the ring's shared phase, ONE NextFloat before the loop.
    let num21 = std::f32::consts::PI * 2.0 * rng.next_float();
    let spread = Vec2::new(40.0, 30.0); // `:3874`
    for i in 0..14 {
        // `:3878` — NewDust(vector9, 0, 0, type): the internals roll even
        // though the position/velocity are overwritten below.
        let idx = dust.new_dust(
            rng, ctx, world, anchor, 0, 0, ty, 0.0, 0.0, 0, Color::default(), 1.0,
        );
        // `:3879-3880` — the spoke, scaled by 0.2 * frameExtra.
        let angle = i as f32 * (std::f32::consts::PI * 2.0) / 14.0 + num21;
        let mut spoke = rotated_by(Vec2::new(0.0, 1.0), angle as f64);
        spoke *= 0.2 * frame_extra as f32;
        let d = dust.slot_mut(idx);
        d.position = anchor + spoke * spread; // `:3881`
        // `:3882` — Sign(vx) * frameExtra * 2 is INTEGER maths, cast once.
        d.velocity = spoke
            + Vec2::new(run_speed - (sign_i(vx) * frame_extra * 2) as f32, 0.0);
        d.no_gravity = true; // `:3883`
        if kind == 47 {
            d.no_light_emittance = true; // `:3884-3887`
        }
        d.scale = 1.0 + rng.next_float() * 0.8; // `:3888`
        d.fade_in = rng.next_float() * 2.0; // `:3889`
        // `:3890` — the shader write (dropped).
    }
}

/// Site 3896 — the Unicorn's dash body dust (`Mount.cs:3894-3902`):
/// `(type == 10) & flag7` (the dash-speed gate, the caller's). ONE
/// `NewDust(player.position, width, height, SelectRandom(176, 177, 179))`
/// — the `Next(3)` pick is the ARGUMENT draw, before the internals — then
/// `velocity = Zero`, `noGravity`, `scale = 0.5 + NextFloat()*0.8`,
/// `fadeIn = 1 + NextFloat()*2`.
pub fn unicorn_dash_body(
    pos: Vec2,
    width: i32,
    height: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let ty = rng.next_from_slice(&[176, 177, 179]); // `:3896` argument
    let idx = dust.new_dust(
        rng, ctx, world, pos, width, height, ty, 0.0, 0.0, 0, Color::default(), 1.0,
    );
    let d = dust.slot_mut(idx);
    d.velocity = Vec2::ZERO; // `:3897`
    d.no_gravity = true; // `:3898`
    d.scale = 0.5 + rng.next_float() * 0.8; // `:3899`
    d.fade_in = 1.0 + rng.next_float() * 2.0; // `:3900`
    // `:3901` — the shader write (dropped).
}

/// Sites 3939/4000 — the Pirate Ship (44, type 228) / Spooky Wood (45,
/// type 6) dash-start ring (`Mount.cs:3931-3957` / `:3992-4018`, identical
/// bodies bar the type): inside the `_mountSpecificData` dash-start latch
/// (the caller's). `num5 = 2π * NextFloat()` before the loop; 20 dusts,
/// each `NewDust(anchor, 0, 0, type)` with the overwritten
/// `vector4 = UnitY.RotatedBy(i * 2π/20 + num5) * 0.8`,
/// `position = anchor + vector4 * (40, 30)`,
/// `velocity = vector4 + (RunSpeed - Sign(velocity.Length()), 0)`;
/// `velocity.X > 0` flips X, a `Next(2) == 0` halves the velocity,
/// `noGravity`, `scale = 1.5 + NextFloat()*0.8`, `fadeIn = 0`.
/// `anchor` = `Center + (width*direction, 0)` (the caller's).
pub fn dash_ring_20(
    dust_type: i32,
    anchor: Vec2,
    run_speed: f32,
    vel: Vec2,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let phase = std::f32::consts::PI * 2.0 * rng.next_float(); // `:3936`/`:3997`
    let spread = Vec2::new(40.0, 30.0);
    for i in 0..20 {
        let idx = dust.new_dust(
            rng, ctx, world, anchor, 0, 0, dust_type, 0.0, 0.0, 0, Color::default(), 1.0,
        );
        let angle = i as f32 * (std::f32::consts::PI * 2.0) / 20.0 + phase;
        let mut spoke = rotated_by(Vec2::new(0.0, 1.0), angle as f64);
        spoke *= 0.8; // `:3941`/`:4002`
        let d = dust.slot_mut(idx);
        d.position = anchor + spoke * spread; // `:3942`/`:4003`
        // `:3943`/`:4004` — Sign(velocity.Length()) is 0 or 1.
        d.velocity = spoke + Vec2::new(run_speed - sign_i(vel.length()) as f32, 0.0);
        if vel.x > 0.0 {
            d.velocity.x *= -1.0; // `:3944-3947`/`:4005-4008`
        }
        if rng.next_max(2) == 0 {
            d.velocity *= 0.5; // `:3948-3951`/`:4009-4012`
        }
        d.no_gravity = true; // `:3952`/`:4013`
        d.scale = 1.5 + rng.next_float() * 0.8; // `:3953`/`:4014`
        d.fade_in = 0.0; // `:3954`/`:4015`
        // `:3955`/`:4016` — the shader write (dropped).
    }
}

/// Site 3970 — the Pirate Ship's run puff (`Mount.cs:3958-3977`): EVERY
/// tick of case 44 draws `Next(num7)` where
/// `num7 = (int)RunSpeed - (int)|vx|` floored at 1; on a 0, one
/// `NewDust(bottom - (num9, 16), num9*2, 22, 228)` with
/// `num9 = width/2 + 10` (INTEGER width), `velocity = Zero`, `noGravity`,
/// `noLight`, `scale = 0.25 + NextFloat()*0.8`,
/// `fadeIn = 0.5 + NextFloat()*2`. Returns whether the puff fired (the
/// gate draw runs regardless — the caller's stream census).
pub fn pirate_run_puff(
    bottom: Vec2,
    width: i32,
    run_speed: f32,
    vx: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) -> bool {
    // `:3958-3962` — the int subtraction, the <= 0 floor.
    let mut num7 = run_speed as i32 - vx.abs() as i32;
    if num7 <= 0 {
        num7 = 1;
    }
    if rng.next_max(num7) != 0 {
        return false; // `:3963` — the gate draw still ran.
    }
    let num9 = width / 2 + 10; // `:3966` — integer division.
    let pos = Vec2::new(bottom.x - num9 as f32, bottom.y - (22 - 6) as f32);
    let idx = dust.new_dust(
        rng, ctx, world, pos, num9 * 2, 22, 228, 0.0, 0.0, 0, Color::default(), 1.0,
    );
    let d = dust.slot_mut(idx);
    d.velocity = Vec2::ZERO; // `:3971`
    d.no_gravity = true; // `:3972`
    d.no_light = true; // `:3973`
    d.scale = 0.25 + rng.next_float() * 0.8; // `:3974`
    d.fade_in = 0.5 + rng.next_float() * 2.0; // `:3975`
    // `:3976` — the shader write (dropped).
    true
}

/// Site 4023 — the Spooky Wood's dash body dust (`Mount.cs:4019-4041`):
/// inside `flag5` (|vx| > DashSpeed*0.9, the caller's). ONE
/// `SelectRandom(6, 6, 31)` (`Next(3)`) argument pick, then
/// `NewDust(center - (6, -6), 12, 12, type)`; `velocity = player.velocity
/// * 0.1`, a `Next(2) == 0` sets `noGravity`,
/// `scale = 0.7 + NextFloat()*0.8`, a `Next(3) == 0` sets `fadeIn = 0.1`;
/// the 31 arm overrides `noGravity`, `scale *= 1.5`, `fadeIn = 0.2`.
pub fn spooky_dash_body(
    center: Vec2,
    vel: Vec2,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let ty = rng.next_from_slice(&[6i16, 6, 31]) as i32; // `:4021`
    // `:4022-4023` — num20 = 6; center - (num20, num20 - 12).
    let pos = center - Vec2::new(6.0, -6.0);
    let idx = dust.new_dust(
        rng, ctx, world, pos, 12, 12, ty, 0.0, 0.0, 0, Color::default(), 1.0,
    );
    let d = dust.slot_mut(idx);
    d.velocity = vel * 0.1; // `:4024`
    if rng.next_max(2) == 0 {
        d.no_gravity = true; // `:4025-4028`
    }
    d.scale = 0.7 + rng.next_float() * 0.8; // `:4029`
    if rng.next_max(3) == 0 {
        d.fade_in = 0.1; // `:4030-4033`
    }
    if ty == 31 {
        d.no_gravity = true; // `:4034-4039`
        d.scale *= 1.5;
        d.fade_in = 0.2;
    }
    // `:4040` — the shader write (dropped).
}

/// Site 4103 — the Basilisk's run dust pair (`Mount.cs:4099-4109`):
/// inside `flag2 && velocity.Y == 0` where `flag2 = |vx| > RunSpeed/2`
/// (the charge gate below shares it). TWO `NewDust(BottomLeft, width, 6,
/// 31)` — `velocity = (vx*0.15, NextFloat()*-2)`, `noLight`,
/// `scale = 0.5 + NextFloat()*0.8`, `fadeIn = 0.5 + NextFloat()*1`.
/// The `cMount == 0` FloorVisuals hop (`:4110-4115`) is the caller's
/// (module doc). `bottom_left` = `(position.X, position.Y + height)`.
pub fn basilisk_run_dust(
    bottom_left: Vec2,
    width: i32,
    vx: f32,
    vy: f32,
    run_speed: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let flag2 = vx.abs() > run_speed / 2.0; // `:4083`
    if !(flag2 && vy == 0.0) {
        return; // `:4099`
    }
    for _ in 0..2 {
        let idx = dust.new_dust(
            rng, ctx, world, bottom_left, width, 6, 31, 0.0, 0.0, 0, Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.velocity = Vec2::new(vx * 0.15, rng.next_float() * -2.0); // `:4104`
        d.no_light = true; // `:4105`
        d.scale = 0.5 + rng.next_float() * 0.8; // `:4106`
        d.fade_in = 0.5 + rng.next_float() * 1.0; // `:4107`
        // `:4108` — the shader write (dropped).
    }
}

/// The Basilisk charge accumulator (`Mount.cs:4087-4094`), the pure half
/// of case 14: `!flag2` zeroes it, else `Clamp(charge + 1/180, 0, 1)`.
/// Runs BEFORE the run dust and the charge dust (the C# order); the
/// wiring wave stores the result on `Player.basiliskCharge`
/// (`Player.cs:842`, unported field).
pub fn basilisk_charge_update(charge: f32, vx: f32, run_speed: f32) -> f32 {
    let flag2 = vx.abs() > run_speed / 2.0; // `:4083`
    if !flag2 {
        0.0 // `:4089`
    } else {
        (charge + 1.0 / 180.0).clamp(0.0, 1.0) // `:4093`
    }
}

/// Site 4123 — the Basilisk's charge dust (`Mount.cs:4117-4139`): the
/// `num != direction` break first (`num = Sign(vx)`, `:4084`), then
/// `(int)(3 * charge)` dusts. Each: `NewDust(BottomLeft, width, 6, 6)`,
/// `position = Center + (num*38, 6 + NextFloat()*12)`,
/// `velocity = SafeNormalize(position - (Center + (num*40, 12))) *
/// (3.5 + NextFloat()*0.5)`; a negative Y gets `*= 1 + 2*NextFloat()`;
/// then `+= player.velocity*0.55`, `*= velocity.Length()/RunSpeed`,
/// `*= charge`; `noGravity`, `noLight`, `scale = 0.5 + NextFloat()*0.8`,
/// `fadeIn = 0.5 + NextFloat()*1`. `charge` is the POST-update value.
#[allow(clippy::too_many_arguments)]
pub fn basilisk_charge_dust(
    charge: f32,
    center: Vec2,
    bottom_left: Vec2,
    width: i32,
    facing: i32,
    vel: Vec2,
    run_speed: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let num = sign_i(vel.x); // `:4084`
    if num != facing {
        return; // `:4117-4120` — the break before the loop.
    }
    let count = (3.0 * charge) as i32; // `:4121`
    let aim = center + Vec2::new(num as f32 * 40.0, 12.0); // `:4124`
    for _ in 0..count {
        let idx = dust.new_dust(
            rng, ctx, world, bottom_left, width, 6, 6, 0.0, 0.0, 0, Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        // `:4125` — num3 = 40, num2 = 12; the y jitter is a NextFloat.
        d.position = center
            + Vec2::new(num as f32 * (40.0 - 2.0), 12.0 - 6.0 + rng.next_float() * 12.0);
        // `:4126` — the aim-relative unit vector, the 3.5 + 0.5*wobble speed.
        d.velocity =
            safe_normalize(d.position - aim, Vec2::ZERO) * (3.5 + rng.next_float() * 0.5);
        if d.velocity.y < 0.0 {
            d.velocity.y *= 1.0 + 2.0 * rng.next_float(); // `:4127-4130`
        }
        d.velocity += vel * 0.55; // `:4131`
        d.velocity *= vel.length() / run_speed; // `:4132`
        d.velocity *= charge; // `:4133`
        d.no_gravity = true; // `:4134`
        d.no_light = true; // `:4135`
        d.scale = 0.5 + rng.next_float() * 0.8; // `:4136`
        d.fade_in = 0.5 + rng.next_float() * 1.0; // `:4137`
        // `:4138` — the shader write (dropped).
    }
}

// ---------------------------------------------------------------------------
// W4-f6 batch 2 — the Mount EFFECTS dust sites (10): the golf-cart smoke
// (`EmitGolfCartSmoke`, `Mount.cs:4626` — DEAD, zero callers), the four
// minecart effects (`DoGemMinecartEffect` :4703 / `DoSteamMinecartEffect`
// :4719 / `DoExhaustMinecartEffect` :4750 / `DoConfettiMinecartEffect` :4782)
// and the `UpdateEffects` (:4795) arms — the Witch Broom sparkle + trail
// (:4824/:4849), the Drill-mount trail (:5061/:5071) and the Lava-cart
// trail (:5099). Same staged-emitter contract as batch 1: the minecart rows
// (25-31/32/34/36) are M2-MOUNT-3 and the broom/drill/lava `UpdateEffects`
// arms are unported, so each fn takes the snapshot at the C# read moment
// and the wiring wave calls them on the `UpdatePlayers` stream.
//
// `RotatedRelativePoint` is the caller's (`player.rs::rotated_relative_point`,
// the `fullRotation`/`gfxOffY`/mount-offset fold) — the emitters take the
// already-rotated spawn point. `Directions` = `(direction, gravDir)` =
// `(facing, 1)` in the port (no gravity flip). The `shader` writes are the
// dropped field; the broom's `Lighting.AddLight` (:4820) and the drill/lava
// `AddLight` heads are the light lane's.
// ---------------------------------------------------------------------------


/// `Vector2.DirectionTo` (`Utils.cs`): `Normalize(Target - Origin)` — the
/// zero vector normalizes to zero (XNA's `Normalize` guards the length).
fn direction_to(origin: Vec2, target: Vec2) -> Vec2 {
    (target - origin).normalize_or_zero()
}

/// `Mount.GetWitchBroomTrinketRotation` (`Mount.cs:3562-3574`), verbatim —
/// RNG-FREE (the wind/miscCounter sway). `wind_in_place` is
/// `WorldGen.InAPlaceWithWind(Center.ToTileCoordinates(), 1, 1)`,
/// `wind_for_visuals` is `Main.WindForVisuals` (= `windSpeedCurrent`).
pub(crate) fn witch_broom_trinket_rotation(
    vx: f32,
    misc_counter: u32,
    wind_for_visuals: f32,
    wind_in_place: bool,
) -> f32 {
    let num = (vx / 10.0).clamp(-1.0, 1.0); // `:3564`
    let num3 = if wind_in_place { 1.0 } else { 0.5 }; // `:3567-3571`
    // `:3572` — the wind sway (the double-sin of the f32 miscCounter/300).
    let num2 = (misc_counter as f32 / 300.0 * (std::f32::consts::PI * 2.0) * 3.0).sin()
        * (std::f32::consts::PI / 4.0)
        * wind_for_visuals.abs()
        * 0.5
        + (std::f32::consts::PI / 4.0) * (0.0 - wind_for_visuals) * 0.5;
    // `:3573` — the velocity sway + the wind term.
    num * (misc_counter as f32 / 150.0 * (std::f32::consts::PI * 2.0) * 3.0).sin()
        * (std::f32::consts::PI / 4.0)
        * 0.5
        + num * (std::f32::consts::PI / 4.0) * 0.5
        + num2 * num3
}

/// `Mount.GetWitchBroomTrinketOriginOffset` (`Mount.cs:3576-3579`):
/// `(29 * direction, -4)`.
pub(crate) fn witch_broom_trinket_origin_offset(facing: i32) -> Vec2 {
    Vec2::new(29.0 * facing as f32, -4.0)
}

/// Site 4626 — the golf-cart smoke (`Mount.cs:4623-4639`). **DEAD CODE**:
/// `EmitGolfCartSmoke` has ZERO callers in the decompile (the golf-cart
/// `UpdateFrame_GolfCart` :4570-4621 calls `EmitGolfCartWheelDust` +
/// `EmitGolfCartlight`, never this). Ported verbatim for the census, staged
/// unwired. ONE `NewDustDirect(position, 0, 0, 31, -direction,
/// -gravDir*0.24, 100)`, then `position = position` (the anchor),
/// `velocity *= 0.1`, `velocity += (-direction, -gravDir*0.25)`,
/// `scale = 0.5`; a non-zero `vx` adds `Sign(vx)*1.3` X; `rushing` sets
/// `fadeIn = 0.8`. `position` = `Bottom + (-direction*34, -gravDir*12)`.
pub fn golf_cart_smoke(
    position: Vec2,
    facing: i32,
    vx: f32,
    rushing: bool,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let dir = facing as f32;
    // `:4626` — gravDir = 1 in the port; SpeedY = -0.24.
    let idx = dust.new_dust_direct(
        rng, ctx, world, position, 0, 0, 31, -dir, -0.24, 100, Color::default(), 1.0,
    );
    let d = dust.slot_mut(idx);
    d.position = position; // `:4627`
    d.velocity *= 0.1; // `:4628`
    d.velocity += Vec2::new(-dir, -0.25); // `:4629`
    d.scale = 0.5; // `:4630`
    if vx != 0.0 {
        d.velocity += Vec2::new(sign_i(vx) as f32 * 1.3, 0.0); // `:4631-4634`
    }
    if rushing {
        d.fade_in = 0.8; // `:4635-4638`
    }
}

/// Site 4703 — the gem-minecart sparkle (`Mount.cs:4697-4710`): a
/// `Next(10) == 0` gate EVERY tick; on a 0, ONE
/// `NextVector2Square(-1, 1) * (22, 10)` offset, then
/// `NewDustPerfect(RotatedRelativePoint(Center + (0, 10)*Directions +
/// offset), dustType)`; `noGravity`, `fadeIn = 0.6`, `scale = 0.4`,
/// `velocity *= 0.25`. `spawn_point` is the already-rotated anchor (the
/// caller folds `Center + (0, 10)*Directions` through
/// `rotated_relative_point`); `dust_type` is the per-cart gem id
/// (86-91/262, `UpdateEffects` :4858-4878). Returns whether the gate fired.
pub fn gem_minecart_sparkle(
    spawn_point: Vec2,
    dust_type: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) -> bool {
    if rng.next_max(10) != 0 {
        return false; // `:4699` — the gate draw still ran.
    }
    // `:4701` — the square offset, scaled (22, 10).
    let offset = next_vector2_square(rng, -1.0, 1.0) * Vec2::new(22.0, 10.0);
    // `:4703` — NewDustPerfect(point + offset, dustType): the internals
    // roll, then position is overwritten to the exact point.
    let idx = dust.new_dust_perfect(
        rng, ctx, world, spawn_point + offset, dust_type, None, 0, Color::default(), 1.0,
    );
    let d = dust.slot_mut(idx);
    d.no_gravity = true; // `:4704`
    d.fade_in = 0.6; // `:4705`
    d.scale = 0.4; // `:4706`
    d.velocity *= 0.25; // `:4707`
    // `:4708` — the shader write (dropped).
    true
}

/// Site 4719 — the steam-minecart puff (`Mount.cs:4712-4727`): the gate is
/// `|vx| >= 1 && (|vx| >= 6 || frame == 0)` (NO rand in the gate). On a
/// pass, ONE `NextVector2Square(-1, 1) * (3, 3)` offset, then
/// `NewDustPerfect(RotatedRelativePoint(Center + (-10, -4)*Directions +
/// offset), dustType)`; `noGravity`, `fadeIn = 0.6`, `scale = 1.8`,
/// `velocity *= 0.25`, `velocity.Y -= 2`. `dust_type` is 303 (the steam
/// cart 36, :5125). Returns whether the gate passed.
pub fn steam_minecart_puff(
    spawn_point: Vec2,
    dust_type: i32,
    vx: f32,
    frame: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) -> bool {
    let num = vx.abs(); // `:4714`
    // `:4715` — `!(num < 1) && (!(num < 6) || frame == 0)`.
    if num < 1.0 || (num < 6.0 && frame != 0) {
        return false;
    }
    let offset = next_vector2_square(rng, -1.0, 1.0) * Vec2::new(3.0, 3.0); // `:4717`
    let idx = dust.new_dust_perfect(
        rng, ctx, world, spawn_point + offset, dust_type, None, 0, Color::default(), 1.0,
    );
    let d = dust.slot_mut(idx);
    d.no_gravity = true; // `:4720`
    d.fade_in = 0.6; // `:4721`
    d.scale = 1.8; // `:4722`
    d.velocity *= 0.25; // `:4723`
    d.velocity.y -= 2.0; // `:4724`
    // `:4725` — the shader write (dropped).
    true
}

/// Site 4750 — the exhaust-minecart smoke (`Mount.cs:4729-4761`): the gate
/// is `num < 1 && Next(4) != 0` → return (the `Next(4)` draws ONLY when
/// `num < 1`, the `&&` short-circuit). `num2 = 1 + (int)num/6` dusts; each
/// a `NextVector2Square(-1, 1) * (3, 3)` offset, the anchor
/// `(-18, 20)*Directions` (+ `4*direction` X when `num > 6`, + `velocity *
/// (num2/3)` when `num2 > 0` — INTEGER `num2/3`), then
/// `NewDustPerfect(RotatedRelativePoint(Center + anchor + offset),
/// dustType)`; `noGravity`, `fadeIn = 0.6`, `scale = 1.2`,
/// `velocity *= 0.2`, and `num < 1` subtracts `0.5*direction` X.
/// `dust_type` is 31 (the exhaust cart 32, :5128). `num2` decrements
/// BEFORE each spawn (the `while` body's first statement).
#[allow(clippy::too_many_arguments)]
pub fn exhaust_minecart_smoke(
    center: Vec2,
    facing: i32,
    vel: Vec2,
    dust_type: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let num = vel.length(); // `:4731`
    if num < 1.0 && rng.next_max(4) != 0 {
        return; // `:4732-4735` — the Next(4) only when slow.
    }
    let mut num2 = 1 + (num as i32) / 6; // `:4736`
    while num2 > 0 {
        num2 -= 1; // `:4739` — decrement BEFORE the spawn.
        let offset = next_vector2_square(rng, -1.0, 1.0) * Vec2::new(3.0, 3.0); // `:4740`
        let mut anchor = Vec2::new(-18.0, 20.0) * Vec2::new(facing as f32, 1.0); // `:4741`
        if num > 6.0 {
            anchor.x += 4.0 * facing as f32; // `:4742-4745`
        }
        if num2 > 0 {
            anchor += vel * (num2 / 3) as f32; // `:4746-4749` — integer div.
        }
        let idx = dust.new_dust_perfect(
            rng, ctx, world, center + anchor + offset, dust_type, None, 0,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true; // `:4751`
        d.fade_in = 0.6; // `:4752`
        d.scale = 1.2; // `:4753`
        d.velocity *= 0.2; // `:4754`
        if num < 1.0 {
            d.velocity.x -= 0.5 * facing as f32; // `:4755-4758`
        }
        // `:4759` — the shader write (dropped).
    }
}

/// Site 4782 — the confetti-minecart burst (`Mount.cs:4763-4793`): the gate
/// is `(num < 1 && Next(6) != 0) || (num < 3 && Next(3) != 0)` → return
/// (the `Next(6)` draws when `num < 1`; the `Next(3)` draws when the first
/// clause is false AND `num < 3`). `num2 = 1 + (int)num/6` dusts; each a
/// `NextFloat()*2` wobble (`num3`), a `NextVector2Square(-1, 1) * (3, 8)`
/// offset, the anchor `(-18, 4)*Directions` with
/// `X += num*direction*0.5 + (direction*num2)*num3` (+ `velocity*(num2/3)`
/// when `num2 > 0`), then `NewDustPerfect(RotatedRelativePoint(Center +
/// anchor + offset), 139 + Next(4))`; `noGravity`, `fadeIn = 0.6`,
/// `scale = 0.5 + num3/2`, `velocity *= 0.2`, `num < 1` subtracts
/// `0.5*direction` X. `num2` decrements BEFORE each spawn.
#[allow(clippy::too_many_arguments)]
pub fn confetti_minecart_burst(
    center: Vec2,
    facing: i32,
    vel: Vec2,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let num = vel.length(); // `:4765`
    // `:4766` — the two-clause gate; each Next draws only when its `num`
    // threshold holds and the earlier clause did not already return.
    if num < 1.0 && rng.next_max(6) != 0 {
        return;
    }
    if num < 3.0 && rng.next_max(3) != 0 {
        return;
    }
    let dir = facing as f32;
    let mut num2 = 1 + (num as i32) / 6; // `:4770`
    while num2 > 0 {
        num2 -= 1; // `:4773`
        let num3 = rng.next_float() * 2.0; // `:4774`
        let offset = next_vector2_square(rng, -1.0, 1.0) * Vec2::new(3.0, 8.0); // `:4775`
        let mut anchor = Vec2::new(-18.0, 4.0) * Vec2::new(dir, 1.0); // `:4776`
        // `:4777` — (direction * num2) is INTEGER, cast once.
        anchor.x += num * dir * 0.5 + (facing * num2) as f32 * num3;
        if num2 > 0 {
            anchor += vel * (num2 / 3) as f32; // `:4778-4781`
        }
        let ty = 139 + rng.next_max(4); // `:4782` — the type ARGUMENT draw.
        let idx = dust.new_dust_perfect(
            rng, ctx, world, center + anchor + offset, ty, None, 0, Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true; // `:4783`
        d.fade_in = 0.6; // `:4784`
        d.scale = 0.5 + num3 / 2.0; // `:4785`
        d.velocity *= 0.2; // `:4786`
        if num < 1.0 {
            d.velocity.x -= 0.5 * dir; // `:4787-4790`
        }
        // `:4791` — the shader write (dropped).
    }
}

/// Site 4824 — the Witch Broom's trinket sparkle (`Mount.cs:4815-4833`):
/// case 23, a `Next(45) == 0` gate EVERY tick; on a 0, ONE
/// `NextVector2Circular(4, 4)` offset, then `NewDustPerfect(trinket_point +
/// offset, 43, Zero, 254, Color(255, 255, 0, 255), 0.3)`; a NON-zero offset
/// sets `velocity = DirectionTo(trinket_point, dust.position) * 0.2`;
/// `fadeIn = 0.3`, `noLightEmittance`. `trinket_point` is the caller's
/// `RotatedRelativePoint(Center + (29*direction, -4) +
/// (trinket_rotation + π/2).ToRotationVector2() * 11)` — the broom trinket
/// swing (`GetWitchBroomTrinketRotation` :3562, the wind/miscCounter fold).
/// Returns whether the gate fired.
pub fn broom_trinket_sparkle(
    trinket_point: Vec2,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) -> bool {
    if rng.next_max(45) != 0 {
        return false; // `:4821` — the gate draw still ran.
    }
    let offset = next_vector2_circular(rng, 4.0, 4.0); // `:4823`
    // `:4824` — NewDustPerfect(point + offset, 43, Zero, 254, yellow, 0.3).
    let idx = dust.new_dust_perfect(
        rng, ctx, world, trinket_point + offset, 43, Some(Vec2::ZERO), 254,
        Color::new(255, 255, 0, 255), 0.3,
    );
    let d = dust.slot_mut(idx);
    if offset != Vec2::ZERO {
        d.velocity = direction_to(trinket_point, d.position) * 0.2; // `:4825-4828`
    }
    d.fade_in = 0.3; // `:4829`
    d.no_light_emittance = true; // `:4830`
    // `:4831-4832` — customData + shader (dropped).
    true
}

/// Site 4849 — the Witch Broom's trinket trail (`Mount.cs:4834-4855`): the
/// pos4 anchor draws ONE `NextFloat()` jitter FIRST (`:4835`, `Center +
/// (18 - 20*NextFloat()*direction, 12)`, folded through the caller's
/// `RotatedRelativePoint` via `rot4`), THEN the gate `NextFloat() <= num12`
/// where `num12 = 0.1 + velocity.Length()/30` (a fail breaks). On a pass,
/// ONE `NextFloat()` phase (`num13`), then 8 steps (`num14 = 0..1 step
/// 0.125`), each a `Next(15) == 0` gate; on a 0, the spoke
/// `((2π*num14 + num13).ToRotationVector2() * (0.5, 1) * 4).RotatedBy(
/// fullRotation)`, then `NewDustPerfect(pos4 + spoke, 43, Zero, 254,
/// yellow, 0.3)`; `velocity = spoke*0.025 + DirectionTo(pos5,
/// dust.position)*0.5`, `fadeIn = 0.3`, `noLightEmittance`. `pos4_base` is
/// the un-jittered `Center + (18, 12)`; `pos5` the caller's rotated
/// `Center + (52*direction, -6)`. Returns whether the gate passed.
#[allow(clippy::too_many_arguments)]
pub fn broom_trinket_trail(
    pos4_base: Vec2,
    facing: i32,
    rot4: impl Fn(Vec2) -> Vec2,
    pos5: Vec2,
    full_rotation: f32,
    vel_len: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) -> bool {
    // `:4835` — the pos4 jitter NextFloat draws BEFORE the gate (:4839).
    let pos4 = rot4(pos4_base + Vec2::new(-20.0 * rng.next_float() * facing as f32, 0.0));
    let num12 = 0.1 + vel_len / 30.0; // `:4834`
    if !(rng.next_float() <= num12) {
        return false; // `:4839-4842` — the NextFloat still ran.
    }
    let num13 = rng.next_float(); // `:4843`
    let mut num14 = 0.0f32;
    while num14 < 1.0 {
        if rng.next_max(15) == 0 {
            // `:4848` — the spoke, rotated by the player's fullRotation.
            let angle = std::f32::consts::PI * 2.0 * num14 + num13;
            let unit = Vec2::new(
                (angle as f64).cos() as f32,
                (angle as f64).sin() as f32,
            );
            let spoke = rotated_by(unit * Vec2::new(0.5, 1.0) * 4.0, full_rotation as f64);
            let idx = dust.new_dust_perfect(
                rng, ctx, world, pos4 + spoke, 43, Some(Vec2::ZERO), 254,
                Color::new(255, 255, 0, 255), 0.3,
            );
            let d = dust.slot_mut(idx);
            d.velocity = spoke * 0.025 + direction_to(pos5, d.position) * 0.5; // `:4850`
            d.fade_in = 0.3; // `:4851`
            d.no_light_emittance = true; // `:4852`
            // `:4853` — the shader write (dropped).
        }
        num14 += 0.125; // `:4844` — the f32 loop step.
    }
    true
}

/// Sites 5061/5071 — the Drill-mount's exhaust trail (`Mount.cs:5016-5078`,
/// case 11): the trail anchors `vector8 = Bottom + vector6 + vector7` and
/// `vector9 = oldPosition + Size*(0.5, 1) + vector6 + vector7` (the
/// `vector6`/`vector7` fullRotation folds are the caller's, :5044-5049).
/// When `Distance(vector8, vector9) > 3`, `num10 = (int)Distance/3` (+1 when
/// `Distance % 3 != 0`) dusts at `Lerp(vector9, vector8, num11/num10)` for
/// `num11 = 1..=num10`; else ONE dust at `vector8`. Each:
/// `NewDust(Center, 0, 0, 182)`, `position` overwritten, `noGravity`,
/// `velocity = Zero`. The `AddLight` head (:5030-5034) is the light lane's.
pub fn drill_mount_trail(
    center: Vec2,
    anchor_now: Vec2,
    anchor_old: Vec2,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let dist = (anchor_now - anchor_old).length(); // `:5052`
    if dist > 3.0 {
        // `:5054-5058` — the integer step count, +1 on a remainder.
        let mut num10 = dist as i32 / 3;
        if dist % 3.0 != 0.0 {
            num10 += 1;
        }
        let mut num11 = 1.0f32;
        while num11 <= num10 as f32 {
            let idx = dust.new_dust(
                rng, ctx, world, center, 0, 0, 182, 0.0, 0.0, 0, Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            // `:5062` — Lerp(old, now, num11/num10).
            d.position = anchor_old + (anchor_now - anchor_old) * (num11 / num10 as f32);
            d.no_gravity = true; // `:5063`
            d.velocity = Vec2::ZERO; // `:5064`
            // `:5065-5066` — customData + shader (dropped).
            num11 += 1.0;
        }
    } else {
        let idx = dust.new_dust(
            rng, ctx, world, center, 0, 0, 182, 0.0, 0.0, 0, Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.position = anchor_now; // `:5072`
        d.no_gravity = true; // `:5073`
        d.velocity = Vec2::ZERO; // `:5074`
        // `:5075-5076` — customData + shader (dropped).
    }
}

/// Site 5099 — the Lava-cart's bubble trail (`Mount.cs:5080-5114`, case
/// 22): `num5 = Clamp(velocity.Length()/5, 0, 1)`; the f32 `num6 = 0..=1
/// step 0.1` loop runs 10 iterations (the accumulated f32 error pushes the
/// 11th value past 1.0), each a `NextFloat() < num5` gate — on a FAIL
/// (`!(NextFloat() < num5)`) ONE `NewDustPerfect(Lerp(value, vector2, num6),
/// 65, NextVector2Circular(0.5, 0.5)*num5)`; `scale = 0.6`, `fadeIn = 0`,
/// `velocity *= -1`, `noGravity`, `velocity -= vector4`; a `Next(10) == 0`
/// sets `fadeIn = 1.3` AND `velocity = NextVector2Circular(3, 3)*num5`.
/// `value`/`vector2`/`vector4` are the caller's shadow-offset anchors
/// (:5089-5093). NOTE the C# spawns on the gate FAIL (the `!(...)` body).
#[allow(clippy::too_many_arguments)]
pub fn lava_cart_trail(
    value: Vec2,
    anchor: Vec2,
    delta: Vec2,
    vel_len: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let num5 = (vel_len / 5.0).clamp(0.0, 1.0); // `:5094`
    let mut num6 = 0.0f32;
    while num6 <= 1.0 {
        if !(rng.next_float() < num5) {
            // `:5099` — the circular velocity is the ARGUMENT draw.
            let vel = next_vector2_circular(rng, 0.5, 0.5) * num5;
            let idx = dust.new_dust_perfect(
                rng, ctx, world, value + (anchor - value) * num6, 65, Some(vel), 0,
                Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            d.scale = 0.6; // `:5100`
            d.fade_in = 0.0; // `:5101`
            // `:5102` — customData (dropped).
            d.velocity *= -1.0; // `:5103`
            d.no_gravity = true; // `:5104`
            d.velocity -= delta; // `:5105`
            // `:5106` — the shader write (dropped).
            if rng.next_max(10) == 0 {
                d.fade_in = 1.3; // `:5107-5109`
                d.velocity = next_vector2_circular(rng, 3.0, 3.0) * num5; // `:5110`
            }
        }
        num6 += 0.1; // `:5095` — the f32 loop step.
    }
}

// ---------------------------------------------------------------------------
// W4-f6 batch 3 — the Mount SPAWN/DISMOUNT dust sites (7): the Pixie tail
// (`UpdateEffects` case 61, `Mount.cs:5149`), the failed-dismount burst
// (`DoFailedDismountDust`, :6339) and `DoSpawnDust` (:6371) — the Wolf 52
// loop + clone (:6382/:6391), the Chillet 62-65 loop + clone (:6404/:6413)
// and the generic tail (:6493). The Pixie/Bat arms read
// `Projectile.GetFairyQueenWeaponsColorFull` (:21665), ported here as
// [`fairy_queen_weapons_color_full`] (the dev-name switch included).
//
// `DoSpawnDust` runs from `SetMount` (:6300, isDismounting false) and
// `Dismount` (:6169, true) — both inside the player tick (`UpdatePlayers`).
// `DoFailedDismountDust` runs from `Player.cs:6058` (the FailedNoSpace
// 3-strike ladder, `FailedNoSpaceCount * 20` dusts). The `netMode == 2`
// head guards (:6322/:6373) are the single-player drop. The cart
// `Gore.NewGore` siblings (:6466/:6551/:6560) are W4-d55's lane — their
// ARGUMENT draws are consumed here in order (the e2 spin hand-off split),
// the `NewGore` bodies are d55's.
// ---------------------------------------------------------------------------

/// `MathHelper.Lerp(a, b, t)` — XNA's `a + (b - a) * t`.
fn mh_lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// `Utils.Turn01ToCyclic010` (`Utils.cs`): `1 - (cos(value*2π)*0.5 + 0.5)`,
/// the DOUBLE cos of the f32 product cast back.
fn turn_01_to_cyclic010(value: f32) -> f32 {
    1.0 - (((value * (std::f32::consts::PI * 2.0)) as f64).cos() as f32 * 0.5 + 0.5)
}

/// `Utils.PingPongFrom01To010` (`Utils.cs`): the f32 `% 1` wrap, the < 0
/// increment, then the >= 0.5 mirror.
fn ping_pong_from01_to010(value: f32) -> f32 {
    let mut v = value % 1.0;
    if v < 0.0 {
        v += 1.0;
    }
    if v >= 0.5 {
        2.0 - v * 2.0
    } else {
        v * 2.0
    }
}

/// `MathHelper.SmoothStep(value1, value2, amount)` — XNA's cubic Hermite
/// over the clamped amount (the `Vector2.SmoothStep` scalar twin,
/// `projectile_ai_empress.rs::smooth_step`).
fn mh_smooth_step(v1: f32, v2: f32, amount: f32) -> f32 {
    let amount = amount.clamp(0.0, 1.0);
    let num = amount * amount * amount;
    let num2 = amount * amount;
    if amount.abs() <= f32::from_bits(1) {
        return v1;
    }
    if amount == 1.0 {
        return v2;
    }
    (2.0 * v1 - 2.0 * v2) * num + (3.0 * v2 - 3.0 * v1) * num2 + v1
}

/// `new Color(float, float, float[, float])` — XNA's `(byte)(v*255)` per
/// channel (the `Color(Vector3)` ctor, the pylons `color_bytes` convention).
fn color_f(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color::new(
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
        (a * 255.0) as u8,
    )
}

/// `Color.Lerp(a, b, amount)` — XNA's per-channel BYTE lerp, truncated
/// (the pylons `lerp_bytes` convention), alpha included.
fn color_lerp(a: Color, b: Color, amount: f32) -> Color {
    Color::new(
        (a.r as f32 + (b.r as f32 - a.r as f32) * amount) as u8,
        (a.g as f32 + (b.g as f32 - a.g as f32) * amount) as u8,
        (a.b as f32 + (b.b as f32 - a.b as f32) * amount) as u8,
        (a.a as f32 + (b.a as f32 - a.a as f32) * amount) as u8,
    )
}

/// `Color * float` — XNA's `(byte)(c*scale)` per channel, alpha included
/// (the pylons `scale_bytes` convention).
fn color_scale(c: Color, scale: f32) -> Color {
    Color::new(
        (c.r as f32 * scale) as u8,
        (c.g as f32 * scale) as u8,
        (c.b as f32 * scale) as u8,
        (c.a as f32 * scale) as u8,
    )
}

/// `Projectile.GetFairyQueenWeaponsColorFull` (`Projectile.cs:21665-21889`):
/// the Fairy Queen's weapon colour — the `(hueRaw + 0.5) % 1` hue, the
/// dev-name switch (the early-return Cenx/Artyrian/Bame/Unit One arms
/// included), then `hslToRgb(num, 1, luminance) * Opacity`, the optional
/// white lerp and the alpha-channel multiply. `player_name` is the local
/// player's name (`Main.player[playerIndex].name`; the port's
/// `app.player_name`), `player_center` the jopojelly arm's offset anchor.
/// `player_active` folds the `player.active` gate (true for the local
/// player in the sim). ZERO rand draws — a pure colour function.
#[allow(clippy::too_many_arguments)]
pub fn fairy_queen_weapons_color_full(
    player_name: &str,
    player_active: bool,
    player_center: Vec2,
    center: Vec2,
    hue_raw: f32,
    alpha_channel_multiplier: f32,
    lerp_to_white: f32,
    opacity: f32,
    luminance: f32,
) -> Color {
    let saturation = 1.0f32; // `:21667`
    let mut num = (hue_raw + 0.5) % 1.0; // `:21668`
    let mut luminance = luminance;
    let mut alpha = alpha_channel_multiplier;
    // The dev-name switch (`:21674-21878`), only for an active player.
    if player_active {
        // The cos(hueRaw*2π)*0.5+0.5 term several arms share.
        let cos_term = ((hue_raw * (std::f32::consts::PI * 2.0)) as f64).cos() as f32 * 0.5 + 0.5;
        match player_name {
            "Cenx" => {
                // `:21676-21685` — the double-SmoothStep PingPong lerp.
                let amount = mh_smooth_step(
                    0.0, 1.0,
                    mh_smooth_step(0.0, 1.0, ping_pong_from01_to010(num)),
                );
                let mut c = color_lerp(color_f(0.3, 1.0, 0.2, 1.0), Color::new(255, 105, 180, 255), amount);
                if lerp_to_white != 0.0 {
                    c = color_lerp(c, Color::WHITE, lerp_to_white);
                }
                c.a = (c.a as f32 * alpha) as u8;
                return c;
            }
            "Crowno" => {
                luminance = mh_lerp(0.25, 0.4, turn_01_to_cyclic010(num));
                num = mh_lerp(127.0 / 180.0, 47.0 / 60.0, turn_01_to_cyclic010(num));
                alpha = mh_lerp(alpha, 0.5, 0.5);
            }
            "Yoraiz0r" => num = mh_lerp(0.9, 0.95, turn_01_to_cyclic010(num)),
            "Jaxrud" => num = mh_lerp(13.0 / 72.0, 0.43611112, turn_01_to_cyclic010(num)),
            "Lazure" => num = mh_lerp(8.0 / 15.0, 83.0 / 90.0, turn_01_to_cyclic010(num)),
            "Leinfors" => num = mh_lerp(0.7, 0.77, turn_01_to_cyclic010(num)),
            "Grox The Great" => {
                num = mh_lerp(0.31, 0.5, turn_01_to_cyclic010(num));
                alpha = mh_lerp(alpha, 1.0, 0.8);
            }
            "Acamaeda" => {
                num = mh_lerp(0.06, 0.28, turn_01_to_cyclic010(num));
                alpha = mh_lerp(alpha, 0.6, 0.5);
            }
            "Alchemystics" => {
                num = mh_lerp(0.74, 0.96, turn_01_to_cyclic010(num));
                luminance = 0.6;
                alpha = mh_lerp(alpha, 0.6, 0.5);
            }
            "Antithesis" => {
                num = 0.51;
                luminance = mh_lerp(0.0, 0.5, cos_term);
            }
            "Artyrian" => {
                let amount = mh_smooth_step(
                    0.0, 1.0,
                    mh_smooth_step(0.0, 1.0, ping_pong_from01_to010(num)),
                );
                let mut c = color_lerp(color_f(1.0, 0.31, 0.0, 1.0), color_f(0.7, 0.0, 1.0, 1.0), amount);
                if lerp_to_white != 0.0 {
                    c = color_lerp(c, Color::WHITE, lerp_to_white);
                }
                c.a = (c.a as f32 * alpha) as u8;
                return c;
            }
            "Aurora3500" => {
                num = mh_lerp(0.33, 0.8, turn_01_to_cyclic010(num));
                alpha = mh_lerp(alpha, 0.5, 0.5);
            }
            "Bame" => {
                let amount = mh_smooth_step(
                    0.0, 1.0,
                    mh_smooth_step(0.0, 1.0, ping_pong_from01_to010(num)),
                );
                let mut c = color_lerp(Color::new(255, 255, 0, 255), color_f(0.4, 0.0, 0.75, 1.0), amount);
                if lerp_to_white != 0.0 {
                    c = color_lerp(c, Color::WHITE, lerp_to_white);
                }
                c.a = (c.a as f32 * alpha) as u8;
                return c;
            }
            "CheesyPaninie" => {
                num = mh_lerp(0.88, 1.09, turn_01_to_cyclic010(num));
                alpha = mh_lerp(alpha, 0.5, 0.5);
            }
            "Criddle" => {
                num = mh_lerp(0.05, 0.15, turn_01_to_cyclic010(num));
                alpha = mh_lerp(alpha, 0.5, 0.5);
            }
            "Darthkitten" => {
                num = 1.0;
                luminance = mh_lerp(1.0, 0.4, cos_term);
            }
            "darthmorf" => {
                num = 0.0;
                luminance = mh_lerp(0.0, 0.2, cos_term);
            }
            "Discipile" => {
                num = 0.53;
                luminance = mh_lerp(0.05, 0.5, cos_term);
            }
            "Doylee" => {
                num = mh_lerp(0.68, 1.0, turn_01_to_cyclic010(num));
                alpha = mh_lerp(alpha, 0.6, 0.5);
            }
            "Ghostar" => {
                num = 0.66;
                luminance = mh_lerp(0.15, 0.85, cos_term);
            }
            "GreyL1me" => {
                num = 0.13;
                luminance = mh_lerp(0.4, 0.6, cos_term);
                alpha = 0.3;
            }
            "Jenosis" => {
                num = mh_lerp(0.9, 1.13, turn_01_to_cyclic010(num)) % 1.0;
                alpha = mh_lerp(alpha, 0.5, 0.5);
            }
            "jopojelly" | "Jopojelly" => {
                // `:21781-21789` — the player-center X offset lerp.
                let v = player_center - center;
                let amount = turn_01_to_cyclic010(num);
                let t = (v.x / 10.0).clamp(-1.0, 1.0) * 0.5 + 0.5;
                num = mh_lerp(mh_lerp(0.072, 0.086, amount), mh_lerp(0.47, 0.55, amount), t);
            }
            "Kazzymodus" => {
                num = 0.33;
                luminance = mh_lerp(0.15, 0.4, cos_term);
            }
            "Khaios" => {
                num = 0.33;
                luminance = mh_lerp(0.0, 0.2, cos_term);
            }
            "Loki" => {
                num = 0.0;
                luminance = mh_lerp(0.0, 0.25, cos_term);
            }
            "Luna" => num = mh_lerp(0.49, 0.8, turn_01_to_cyclic010(num)),
            "ManaUser" => num = mh_lerp(0.41, 0.57, turn_01_to_cyclic010(num)),
            "Mid" => {
                num = 0.0;
                luminance = mh_lerp(0.0, 0.9, cos_term);
            }
            "Nimbus" => {
                num = mh_lerp(0.75, 1.0, turn_01_to_cyclic010(num));
                luminance = 1.0;
                alpha = mh_lerp(alpha, 0.5, 0.8);
            }
            "Nike Leon" => {
                num = mh_lerp(0.04, 0.1, turn_01_to_cyclic010(num));
                alpha = mh_lerp(alpha, 0.5, 0.5);
            }
            "ppowersteef" => {
                num = mh_lerp(0.0, 0.15, turn_01_to_cyclic010(num));
                alpha = mh_lerp(alpha, 0.6, 0.5);
            }
            "RBrandon" => {
                num = 0.03;
                luminance = 0.3;
                alpha = mh_lerp(alpha, 0.6, 0.5);
            }
            "Redigit" => num = 0.7,
            "Serenity" => {
                num = 0.85;
                luminance = mh_lerp(1.0, 0.5, cos_term);
            }
            "Shadouette" => num = mh_lerp(0.91, 0.51, turn_01_to_cyclic010(num)),
            "Sigma" => {
                num = mh_lerp(0.0, 0.12, turn_01_to_cyclic010(num));
                luminance = 0.5;
                alpha = mh_lerp(alpha, 0.6, 0.5);
            }
            "teiull" => {
                num = mh_lerp(0.66, 1.0, turn_01_to_cyclic010(num));
                luminance = 0.5;
            }
            "Unit One" => {
                let amount = mh_smooth_step(0.0, 1.0, ping_pong_from01_to010(num));
                let mut c = color_lerp(Color::new(255, 255, 0, 255), Color::new(0, 0, 255, 255), amount);
                if lerp_to_white != 0.0 {
                    c = color_lerp(c, Color::WHITE, lerp_to_white);
                }
                c.a = (c.a as f32 * alpha) as u8;
                return c;
            }
            "Vulpes Inculta" => {
                num = mh_lerp(0.65, 0.75, turn_01_to_cyclic010(num));
                luminance = 0.5;
                alpha = mh_lerp(alpha, 0.5, 0.5);
            }
            "Waze3174" => {
                num = mh_lerp(0.33, 0.0, turn_01_to_cyclic010(num));
                luminance = 0.3;
                alpha = mh_lerp(alpha, 0.6, 0.5);
            }
            "Xman101" => {
                num = 0.06;
                luminance = mh_lerp(0.0, 0.5, cos_term);
            }
            "Zoomo" => {
                num = 0.77;
                luminance = 0.5;
                alpha = mh_lerp(alpha, 0.6, 0.5);
            }
            _ => {}
        }
    }
    // `:21881-21888` — hslToRgb(num, 1, luminance) * Opacity, white lerp,
    // alpha multiply. `ui::color::hsl_to_rgb` is the verbatim Main port.
    let rgb = crate::ui::color::hsl_to_rgb(num as f64, saturation as f64, luminance as f64, 255);
    let mut c = color_scale(Color::new(rgb[0], rgb[1], rgb[2], rgb[3]), opacity);
    if lerp_to_white != 0.0 {
        c = color_lerp(c, Color::WHITE, lerp_to_white);
    }
    c.a = (c.a as f32 * alpha) as u8;
    c
}

/// Site 5149 — the Pixie mount's dust tail (`Mount.cs:5142-5164`, case 61):
/// `flag = Next(15) == 0` EVERY tick; the spawn gate is
/// `(int)timeForVisualEffects % 2 == 0 && (flag || (Next(6) + 1) <
/// velocity.Length())` — the `Next(6)` draws ONLY when `flag` is false (the
/// `||` short-circuit) AND the time gate passed. On a pass, ONE
/// `NewDustDirect(Center, 0, 0, 278, 0, 0, 200, Color.Lerp(fairyA, fairyB,
/// NextFloat()), 0.65)` where fairyA/fairyB are the two
/// `GetFairyQueenWeaponsColorFull` calls (0.45 vs 0 lerpToWhite); then
/// `position = Center + (0, -2)`, `flag` ? `velocity *= 0.4` :
/// `velocity *= 0.04 * velocity.Length()`; `velocity += player.velocity *
/// 0.3`, `position += player.velocity * 0.7`, `position +=
/// (NextFloat()*2π).ToRotationVector2() * NextFloat() * 2`, `noGravity`,
/// `noLight`. Returns whether the spawn fired.
#[allow(clippy::too_many_arguments)]
pub fn pixie_tail(
    center: Vec2,
    vel: Vec2,
    time_for_visual_effects: f64,
    player_name: &str,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) -> bool {
    let flag = rng.next_max(15) == 0; // `:5146`
    // `:5147` — the time gate first; the Next(6) only when !flag.
    let time_gate = (time_for_visual_effects as i32) % 2 == 0;
    if !time_gate {
        return false;
    }
    let speed_gate = flag || ((rng.next_max(6) + 1) as f32) < vel.length();
    if !speed_gate {
        return false;
    }
    // `:5149` — the two fairy colours + the NextFloat lerp ARGUMENT.
    let fairy_a = fairy_queen_weapons_color_full(
        player_name, true, center, center, 0.41, 1.0, 0.45, 1.0, 0.7,
    );
    let fairy_b = fairy_queen_weapons_color_full(
        player_name, true, center, center, 0.41, 1.0, 0.0, 1.0, 0.7,
    );
    let color = color_lerp(fairy_a, fairy_b, rng.next_float());
    let idx = dust.new_dust_direct(
        rng, ctx, world, center, 0, 0, 278, 0.0, 0.0, 200, color, 0.65,
    );
    let d = dust.slot_mut(idx);
    d.position = center + Vec2::new(0.0, -2.0); // `:5150`
    if flag {
        d.velocity *= 0.4; // `:5151-5154`
    } else {
        d.velocity *= 0.04 * vel.length(); // `:5155-5158`
    }
    d.velocity += vel * 0.3; // `:5159`
    d.position += vel * 0.7; // `:5160`
    // `:5161` — the (NextFloat*2π) unit vector * NextFloat * 2.
    let angle = rng.next_float() * (std::f32::consts::PI * 2.0);
    let unit = Vec2::new(
        (angle as f64).cos() as f32,
        (angle as f64).sin() as f32,
    );
    d.position += unit * rng.next_float() * 2.0;
    d.no_gravity = true; // `:5162`
    d.no_light = true; // `:5163`
    true
}

/// Site 6339 — the failed-dismount burst (`Mount.cs:6320-6369`): the
/// `netMode == 2` head is the SP drop. `newColor` is Transparent, Black for
/// the Bat (56), the fairy colour for the Pixie (61). The hitbox inflates
/// (10, 10); `dust_amount` dusts (the caller's `FailedNoSpaceCount * 20`),
/// each `NewDust(hitbox.TopLeft(), hitbox.Width, hitbox.Height,
/// spawnDust, 0, 0, 0, newColor)`; `scale += Next(-10, 21)*0.01`; the Bat
/// pins `scale = 0.7`, `velocity *= 0.4`, `velocity.Y += gravDir*0.5`; the
/// Pixie `scale *= 1 + 0.3*NextFloat()`, `velocity += player.velocity *
/// NextFloat()`; `spawnDustNoGravity` ? `noGravity` : a `Next(2) == 0` ?
/// (`scale *= 1.3`, `noGravity`) : `velocity *= 0.5`; then
/// `velocity += player.velocity * 0.8`.
#[allow(clippy::too_many_arguments)]
pub fn failed_dismount_dust(
    kind: i32,
    spawn_dust: i32,
    spawn_dust_no_gravity: bool,
    hitbox_top_left: Vec2,
    hitbox_w: i32,
    hitbox_h: i32,
    dust_amount: i32,
    vel: Vec2,
    player_name: &str,
    center: Vec2,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:6326-6334` — the colour pick.
    let mut new_color = Color::TRANSPARENT;
    if kind == 56 {
        new_color = Color::new(0, 0, 0, 255); // Color.Black
    }
    if kind == 61 {
        new_color = fairy_queen_weapons_color_full(
            player_name, true, center, center, 0.27, 1.0, 0.15, 1.0, 0.7,
        );
    }
    // `:6335-6336` — the inflated hitbox (the caller passes the inflated
    // box; the TopLeft/Width/Height are the C#'s `hitbox` reads).
    for _ in 0..dust_amount {
        let idx = dust.new_dust(
            rng, ctx, world, hitbox_top_left, hitbox_w, hitbox_h, spawn_dust,
            0.0, 0.0, 0, new_color, 1.0,
        );
        let d = dust.slot_mut(idx);
        d.scale += rng.next_range(-10, 21) as f32 * 0.01; // `:6340`
        if kind == 56 {
            d.scale = 0.7; // `:6341-6347`
            d.velocity *= 0.4;
            d.velocity.y += 0.5; // gravDir = 1
        }
        if kind == 61 {
            d.scale *= 1.0 + 0.3 * rng.next_float(); // `:6348-6353`
            d.velocity += vel * rng.next_float();
        }
        if spawn_dust_no_gravity {
            d.no_gravity = true; // `:6354-6357`
        } else if rng.next_max(2) == 0 {
            d.scale *= 1.3; // `:6358-6362`
            d.no_gravity = true;
        } else {
            d.velocity *= 0.5; // `:6363-6366`
        }
        d.velocity += vel * 0.8; // `:6367`
    }
}

/// Sites 6382/6391 — the Wolf's (52) spawn/dismount burst
/// (`Mount.cs:6377-6397`): 100 dusts, each
/// `NewDustDirect((position.X - 20, position.Y), width + 40, height, 267,
/// 0, 0, 60, Color(130, 60, 255, 70))`; `scale += Next(-10, 21)*0.01`,
/// `noGravity`, `velocity += player.velocity*0.8`, `velocity *= NextFloat()`,
/// `velocity.Y += 2*NextFloatDirection()`, `noLight`; a `Next(3) == 0`
/// clones (`CloneDust`, site 6391) with `color = White`, `scale *= 0.5`,
/// `alpha = 0`. The `_ = spawnDust` read (:6381) is a no-op.
#[allow(clippy::too_many_arguments)]
pub fn wolf_spawn_dust(
    pos: Vec2,
    width: i32,
    height: i32,
    vel: Vec2,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let anchor = Vec2::new(pos.x - 20.0, pos.y); // `:6382`
    for _ in 0..100 {
        let idx = dust.new_dust_direct(
            rng, ctx, world, anchor, width + 40, height, 267, 0.0, 0.0, 60,
            Color::new(130, 60, 255, 70), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.scale += rng.next_range(-10, 21) as f32 * 0.01; // `:6383`
        d.no_gravity = true; // `:6384`
        d.velocity += vel * 0.8; // `:6385`
        d.velocity *= rng.next_float(); // `:6386`
        d.velocity.y += 2.0 * rng.next_float_direction(); // `:6387`
        d.no_light = true; // `:6388`
        if rng.next_max(3) == 0 {
            // `:6389-6395` — the clone (site 6391).
            let cidx = dust.clone_dust(rng, ctx, world, idx);
            let cd = dust.slot_mut(cidx);
            cd.color = Color::WHITE;
            cd.scale *= 0.5;
            cd.alpha = 0;
        }
    }
}

/// Sites 6404/6413 — the Chillet's (62-65) spawn/dismount burst
/// (`Mount.cs:6399-6419`): 100 dusts, each
/// `NewDustDirect((position.X - 20, position.Y), width + 40, height, 306,
/// 0, 0, 60, Color(100, 227, 255, 127), 2)`; `scale += Next(-10, 21)*0.01`,
/// `noGravity`, `velocity += player.velocity*0.8`, `velocity *= NextFloat()`,
/// `velocity.Y += 4*NextFloatDirection()`, `noLight`; a `Next(3) == 0`
/// clones (site 6413) with `color = White`, `scale *= 0.5`, `alpha = 0`.
#[allow(clippy::too_many_arguments)]
pub fn chillet_spawn_dust(
    pos: Vec2,
    width: i32,
    height: i32,
    vel: Vec2,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let anchor = Vec2::new(pos.x - 20.0, pos.y); // `:6404`
    for _ in 0..100 {
        let idx = dust.new_dust_direct(
            rng, ctx, world, anchor, width + 40, height, 306, 0.0, 0.0, 60,
            Color::new(100, 227, 255, 127), 2.0,
        );
        let d = dust.slot_mut(idx);
        d.scale += rng.next_range(-10, 21) as f32 * 0.01; // `:6405`
        d.no_gravity = true; // `:6406`
        d.velocity += vel * 0.8; // `:6407`
        d.velocity *= rng.next_float(); // `:6408`
        d.velocity.y += 4.0 * rng.next_float_direction(); // `:6409`
        d.no_light = true; // `:6410`
        if rng.next_max(3) == 0 {
            // `:6411-6417` — the clone (site 6413).
            let cidx = dust.clone_dust(rng, ctx, world, idx);
            let cd = dust.slot_mut(cidx);
            cd.color = Color::WHITE;
            cd.scale *= 0.5;
            cd.alpha = 0;
        }
    }
}

/// Site 6493 — the generic `DoSpawnDust` tail (`Mount.cs:6420-6541`): the
/// colour pick (Yellow for the Broom 23, Black for the Bat 56, fairy for
/// the Pixie 61), the count (100, or 40 for the roller skates 57-60), the
/// box `r` (the (position.X-20, position.Y, width+40, height) default; the
/// Bat/Pixie use the inflated hitbox or the dismount bottom-box; the
/// skates use the bottom strip). Per dust: the cart arm (the `Cart` set)
/// spawns a gore every 10th iteration and CONTINUES (no dust — the gore
/// ARGUMENT draws are consumed, d55's body); else the type/scale/alpha pick
/// (the 40/41/42 bee arm: `Next(2) != 0` ? 16 : 31, scale 0.9, alpha 50,
/// the 42 pins 31, the 41 pins 16), the Pixie's `Next(isDismounting ? 2 :
/// 5) != 0` continue, then `NewDust(r.TopLeft(), r.Width, r.Height, type,
/// 0, 0, alpha, newColor, scale)`; `scale += Next(-10, 21)*0.01`; the
/// skates' velocity chain; the Bat/Pixie arms; the noGravity/Next(2) gate;
/// `velocity += player.velocity*0.8`; the 40/41/42 `velocity *= NextFloat()`.
/// The trailing 40/41/42 + 23 gore loops (:6542-6564) are d55's (their
/// argument draws are consumed here). `r` is the caller's resolved box.
#[allow(clippy::too_many_arguments)]
pub fn generic_spawn_dust(
    kind: i32,
    spawn_dust: i32,
    spawn_dust_no_gravity: bool,
    is_cart: bool,
    is_dismounting: bool,
    r_top_left: Vec2,
    r_w: i32,
    r_h: i32,
    vel: Vec2,
    player_name: &str,
    center: Vec2,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:6421-6433` — the colour pick.
    let mut new_color = Color::TRANSPARENT;
    if kind == 23 {
        new_color = Color::new(255, 255, 0, 255);
    }
    if kind == 56 {
        new_color = Color::new(0, 0, 0, 255);
    }
    if kind == 61 {
        new_color = fairy_queen_weapons_color_full(
            player_name, true, center, center, 0.27, 1.0, 0.15, 1.0, 0.7,
        );
    }
    // `:6434-6438` — the count (the caller resolves the roller-skate 40).
    let count = if crate::mount::is_roller_skates(kind) { 40 } else { 100 };
    for k in 0..count {
        if is_cart {
            // `:6462-6471` — every 10th iteration a gore (d55's body), the
            // ARGUMENT draws consumed: Next(61, 64) type + NextDouble angle.
            if k % 10 == 0 {
                let _gore_type = rng.next_range(61, 64); // `:6466`
                let _angle = rng.next_double() * 6.2831854820251465; // `:6468`
            }
            continue;
        }
        // `:6472-6488` — the type/scale/alpha pick.
        let mut ty = spawn_dust;
        let mut scale = 1.0f32;
        let mut alpha = 0i32;
        if matches!(kind, 40 | 41 | 42) {
            ty = if rng.next_max(2) != 0 { 16 } else { 31 }; // `:6477`
            scale = 0.9;
            alpha = 50;
            if kind == 42 {
                ty = 31; // `:6480-6483`
            }
            if kind == 41 {
                ty = 16; // `:6484-6487`
            }
        }
        // `:6489-6492` — the Pixie's continue gate.
        if kind == 61 && rng.next_max(if is_dismounting { 2 } else { 5 }) != 0 {
            continue;
        }
        let idx = dust.new_dust(
            rng, ctx, world, r_top_left, r_w, r_h, ty, 0.0, 0.0, alpha, new_color, scale,
        );
        let d = dust.slot_mut(idx);
        d.scale += rng.next_range(-10, 21) as f32 * 0.01; // `:6494`
        if kind > 0 && crate::mount::is_roller_skates(kind) {
            // `:6495-6503` — the skates' velocity chain.
            d.velocity *= 0.8;
            d.velocity.y *= 0.75;
            d.velocity.y += 0.25; // gravDir = 1
            d.no_light_emittance = true;
            d.no_gravity = true;
        }
        if kind == 56 {
            d.scale = 0.7; // `:6504-6510`
            d.velocity *= 0.4;
            d.velocity.y += 0.5;
        }
        if kind == 61 {
            d.scale *= 1.0 + 0.3 * rng.next_float(); // `:6511-6522`
            if !is_dismounting {
                // `:6515-6520` — the lerp toward the box centre.
                let r_center = r_top_left + Vec2::new(r_w as f32 * 0.5, r_h as f32 * 0.5);
                d.position = d.position + (r_center - d.position) * 0.7;
                d.velocity *= -2.0;
                d.velocity.y -= 1.0;
            }
            d.velocity += vel * rng.next_float();
        }
        if spawn_dust_no_gravity {
            d.no_gravity = true; // `:6523-6526`
        } else if rng.next_max(2) == 0 {
            d.scale *= 1.3; // `:6527-6531`
            d.no_gravity = true;
        } else {
            d.velocity *= 0.5; // `:6532-6535`
        }
        d.velocity += vel * 0.8; // `:6536`
        if matches!(kind, 40 | 41 | 42) {
            d.velocity *= rng.next_float(); // `:6537-6540`
        }
    }
    // `:6542-6564` — the trailing gore loops (d55's bodies); the ARGUMENT
    // draws are consumed here in order.
    if matches!(kind, 40 | 41 | 42) {
        for _ in 0..5 {
            let mut type2 = rng.next_range(61, 64); // `:6546`
            if kind == 41 || (kind == 40 && rng.next_max(2) == 0) {
                type2 = rng.next_range(11, 14); // `:6547-6550`
            }
            let _ = type2;
            let _angle = rng.next_double() * 6.2831854820251465; // `:6553`
        }
    }
    if kind == 23 {
        for _ in 0..4 {
            let _gore_type = rng.next_range(61, 64); // `:6560`
            let _angle = rng.next_double() * 6.2831854820251465; // `:6562`
        }
    }
}

// ---------------------------------------------------------------------------
// W4-f6 WIRING — the `pending_mount_dust` request channel (the
// `pending_sticky_dust` convention, `player.rs:576`). The pure player tick
// queues a snapshot at the C# read moment; the `world_sim` drain calls the
// emitter on the `UpdatePlayers` stream (`Main.cs:17754` — `SetMount`,
// `Dismount`, `DoFailedDismountDust`, `UpdateFrame` and `UpdateEffects` all
// run inside `Player.Update`). Argument draws + pool internals + post-arms
// stay inside the emitters, so the drain order IS the C# draw order.
// ---------------------------------------------------------------------------

/// The player read at the `Mount.UpdateFrame` call moment (`Player.cs:
/// 36200-36216`), for the per-type extras' anchors. `basilisk_charge` is
/// the POST-fold value (`Mount.cs:4089-4093` runs before the dusts).
#[derive(Clone, Copy, Debug)]
pub(crate) struct MountFrameSnap {
    pub pos_x: f32,
    pub pos_y: f32,
    pub center_x: f32,
    pub center_y: f32,
    pub width: f32,
    pub height: f32,
    pub facing: i32,
    pub basilisk_charge: f32,
}

/// One queued mount dust event. `Spawn` carries the resolved `DoSpawnDust`
/// box (`Mount.cs:6439-6459` — the caller computes it BEFORE the mount
/// state clears); `FailedDismount` the inflated hitbox (`:6335-6336`); the
/// movement variants the `UpdateFrame` per-type extras (`:3637-4142`).
#[derive(Clone, Copy, Debug)]
pub(crate) enum MountDustRequest {
    /// `Mount.cs:3878` — the Unicorn (10) / Goat (47) dash ring.
    UnicornGoatRing { kind: i32, anchor: Vec2, frame_extra: i32, run_speed: f32, vx: f32 },
    /// `Mount.cs:3896` — the Unicorn dash body.
    UnicornBody { pos: Vec2, width: i32, height: i32 },
    /// `Mount.cs:3939`/`:4000` — the Pirate/Spooky 20-dust ring.
    DashRing20 { dust_type: i32, anchor: Vec2, run_speed: f32, vel: Vec2 },
    /// `Mount.cs:3970` — the Pirate run puff (gate draws every tick).
    PirateRunPuff { bottom: Vec2, width: i32, run_speed: f32, vx: f32 },
    /// `Mount.cs:4023` — the Spooky dash body.
    SpookyBody { center: Vec2, vel: Vec2 },
    /// `Mount.cs:4103`/`:4123` — the Basilisk run pair + charge dust.
    Basilisk {
        charge: f32, center: Vec2, bottom_left: Vec2, width: i32,
        facing: i32, vel: Vec2, vx: f32, vy: f32, run_speed: f32,
    },
    /// `Mount.cs:4703` — the gem-minecart sparkle (the caller folds
    /// `Center + (0,10)*Directions` through `RotatedRelativePoint`).
    GemSparkle { spawn_point: Vec2, dust_type: i32 },
    /// `Mount.cs:4719` — the steam-minecart puff (`Center + (-10,-4)*Dir`).
    SteamPuff { spawn_point: Vec2, dust_type: i32, vx: f32, frame: i32 },
    /// `Mount.cs:4750` — the exhaust-minecart smoke (the caller folds the
    /// per-dust anchor; `anchor_base` = `Center + (-18,20)*Dir` rotated).
    ExhaustSmoke { center: Vec2, facing: i32, vel: Vec2, dust_type: i32 },
    /// `Mount.cs:4782` — the confetti-minecart burst.
    ConfettiBurst { center: Vec2, facing: i32, vel: Vec2 },
    /// `Mount.cs:4824` — the Witch Broom trinket sparkle (the caller folds
    /// the trinket swing through `RotatedRelativePoint`).
    BroomSparkle { trinket_point: Vec2 },
    /// `Mount.cs:4849` — the Witch Broom trinket trail. `pos4_base` is the
    /// un-jittered `Center + (18, 12)`; the emitter draws the pos4 NextFloat
    /// (`:4835`, BEFORE the gate) and the drain folds it through the
    /// player's `RotatedRelativePoint` (the `player` arg).
    BroomTrail {
        pos4_base: Vec2, pos5: Vec2, facing: i32, full_rotation: f32, vel_len: f32,
    },
    /// `Mount.cs:5061`/`:5071` — the Drill-mount exhaust trail (the caller
    /// folds the `vector6`/`vector7` fullRotation anchors).
    DrillTrail { center: Vec2, anchor_now: Vec2, anchor_old: Vec2 },
    /// `Mount.cs:5099` — the Lava-cart bubble trail (the caller folds the
    /// shadow-offset anchors `value`/`vector2`/`vector4`).
    LavaTrail { value: Vec2, anchor: Vec2, delta: Vec2, vel_len: f32 },
    /// `Mount.cs:5149` — the Pixie tail (the even-tick + speed gates).
    PixieTail { center: Vec2, vel: Vec2, time_for_visuals: f64 },
    /// `Mount.DoSpawnDust` (`:6371`) from `SetMount` (`:6300`,
    /// `is_dismounting` false) or `Dismount` (`:6169`, true).
    Spawn {
        kind: i32,
        spawn_dust: i32,
        spawn_dust_no_gravity: bool,
        is_cart: bool,
        is_dismounting: bool,
        pos: Vec2,
        width: i32,
        height: i32,
        r_top_left: Vec2,
        r_w: i32,
        r_h: i32,
        vel: Vec2,
        center: Vec2,
    },
    /// `Mount.DoFailedDismountDust` (`:6320`) from the QuickMount ladder
    /// (`Player.cs:6058`, `FailedNoSpaceCount * 20`).
    FailedDismount {
        kind: i32,
        spawn_dust: i32,
        spawn_dust_no_gravity: bool,
        hitbox_top_left: Vec2,
        hitbox_w: i32,
        hitbox_h: i32,
        dust_amount: i32,
        vel: Vec2,
        center: Vec2,
    },
}

/// The drain: one request → its emitter, on the caller's `UpdatePlayers`
/// stream. `player_name` is `Main.player[whoAmI].name` (the fairy-color
/// dev-name switch reads it); `player` resolves the Witch Broom's pos4
/// `RotatedRelativePoint` fold at drain time (the jitter NextFloat draws
/// inside the emitter, so the fold cannot be pre-computed). The
/// `netMode == 2` heads (`:6322`/`:6373`) are the single-player drop.
pub(crate) fn drain_mount_dust(
    requests: Vec<MountDustRequest>,
    player_name: &str,
    player: &crate::player::Player,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for req in requests {
        match req {
            MountDustRequest::UnicornGoatRing { kind, anchor, frame_extra, run_speed, vx } => {
                unicorn_goat_dash_ring(
                    kind, anchor, frame_extra, run_speed, vx, rng, dust, ctx, world,
                );
            }
            MountDustRequest::UnicornBody { pos, width, height } => {
                unicorn_dash_body(pos, width, height, rng, dust, ctx, world);
            }
            MountDustRequest::DashRing20 { dust_type, anchor, run_speed, vel } => {
                dash_ring_20(dust_type, anchor, run_speed, vel, rng, dust, ctx, world);
            }
            MountDustRequest::PirateRunPuff { bottom, width, run_speed, vx } => {
                pirate_run_puff(bottom, width, run_speed, vx, rng, dust, ctx, world);
            }
            MountDustRequest::SpookyBody { center, vel } => {
                spooky_dash_body(center, vel, rng, dust, ctx, world);
            }
            MountDustRequest::Basilisk {
                charge, center, bottom_left, width, facing, vel, vx, vy, run_speed,
            } => {
                // `:4099-4116` — the run pair first, then `:4117-4139` the
                // charge dust (the emitter carries the facing break).
                basilisk_run_dust(
                    bottom_left, width, vx, vy, run_speed, rng, dust, ctx, world,
                );
                basilisk_charge_dust(
                    charge, center, bottom_left, width, facing, vel, run_speed,
                    rng, dust, ctx, world,
                );
            }
            MountDustRequest::GemSparkle { spawn_point, dust_type } => {
                gem_minecart_sparkle(spawn_point, dust_type, rng, dust, ctx, world);
            }
            MountDustRequest::SteamPuff { spawn_point, dust_type, vx, frame } => {
                steam_minecart_puff(spawn_point, dust_type, vx, frame, rng, dust, ctx, world);
            }
            MountDustRequest::ExhaustSmoke { center, facing, vel, dust_type } => {
                exhaust_minecart_smoke(center, facing, vel, dust_type, rng, dust, ctx, world);
            }
            MountDustRequest::ConfettiBurst { center, facing, vel } => {
                confetti_minecart_burst(center, facing, vel, rng, dust, ctx, world);
            }
            MountDustRequest::BroomSparkle { trinket_point } => {
                broom_trinket_sparkle(trinket_point, rng, dust, ctx, world);
            }
            MountDustRequest::BroomTrail { pos4_base, pos5, facing, full_rotation, vel_len } => {
                broom_trinket_trail(
                    pos4_base, facing, |p| player.rotated_relative_point(p, 0),
                    pos5, full_rotation, vel_len, rng, dust, ctx, world,
                );
            }
            MountDustRequest::DrillTrail { center, anchor_now, anchor_old } => {
                drill_mount_trail(center, anchor_now, anchor_old, rng, dust, ctx, world);
            }
            MountDustRequest::LavaTrail { value, anchor, delta, vel_len } => {
                lava_cart_trail(value, anchor, delta, vel_len, rng, dust, ctx, world);
            }
            MountDustRequest::PixieTail { center, vel, time_for_visuals } => {
                pixie_tail(center, vel, time_for_visuals, player_name, rng, dust, ctx, world);
            }
            MountDustRequest::Spawn {
                kind, spawn_dust, spawn_dust_no_gravity, is_cart, is_dismounting,
                pos, width, height, r_top_left, r_w, r_h, vel, center,
            } => {
                // `:6377-6420` — the Wolf / Chillet early-return arms.
                if kind == 52 {
                    wolf_spawn_dust(pos, width, height, vel, rng, dust, ctx, world);
                } else if matches!(kind, 62..=65) {
                    chillet_spawn_dust(pos, width, height, vel, rng, dust, ctx, world);
                } else {
                    generic_spawn_dust(
                        kind, spawn_dust, spawn_dust_no_gravity, is_cart, is_dismounting,
                        r_top_left, r_w, r_h, vel, player_name, center, rng, dust, ctx, world,
                    );
                }
            }
            MountDustRequest::FailedDismount {
                kind, spawn_dust, spawn_dust_no_gravity, hitbox_top_left,
                hitbox_w, hitbox_h, dust_amount, vel, center,
            } => failed_dismount_dust(
                kind, spawn_dust, spawn_dust_no_gravity, hitbox_top_left,
                hitbox_w, hitbox_h, dust_amount, vel, player_name, center,
                rng, dust, ctx, world,
            ),
        }
    }
}

#[cfg(test)]
mod tests_union_b {
    use super::*;

    fn world() -> World {
        World::new("f6-mount".into(), 200, 200)
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

    /// W4-f6 — sites 3113/3170: two type-230 noGravity dusts per beam,
    /// each consuming Next(2) + NextDouble BEFORE the pool internals; a
    /// mirrored stream through the same pool call lands identical records.
    #[test]
    fn w4_f6_drill_beam_spawns_two_and_mirrors_the_draw_order() {
        let w = world();
        let c = ctx();
        let target = Vec2::new(640.0, 480.0);
        let center = Vec2::new(600.0, 460.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        drill_beam_dust(target, center, &mut rng, &mut pool, &c, &w);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(7);
        let d = target - center;
        let num = (d.y as f64).atan2(d.x as f64) as f32;
        let mut speeds = Vec::new();
        for _ in 0..2 {
            let side = if mirror.next_max(2) == 1 { -1.0 } else { 1.0 };
            let num2 = num + side * (std::f32::consts::PI / 2.0);
            let num3 = mirror.next_double() as f32 * 2.0 + 2.0;
            let v = Vec2::new(
                (num2 as f64).cos() as f32 * num3,
                (num2 as f64).sin() as f32 * num3,
            );
            speeds.push(v.length());
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, target, 0, 0, 230, v.x, v.y, 0, Color::default(), 1.0,
            );
            mpool.slot_mut(idx).no_gravity = true;
        }
        assert_eq!(active(&pool).len(), 2, "two beam dusts");
        for i in 0..2 {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).dust_type, 230);
            assert!(pool.slot(i).no_gravity);
            // The ARGUMENT speed is NextDouble()*2+2 in [2, 4) (`:3111`);
            // the record adds NewDust's `Next(-20,21)*0.1` jitter on top.
            let len = speeds[i];
            assert!(len >= 2.0 && len < 4.0, "argument speed {len}");
        }
        assert_eq!(rng.next(), mirror.next(), "the streams stay aligned");
    }

    /// W4-f6 — site 3878: the Unicorn ring picks ONE type via Next(3)
    /// before the loop, one NextFloat phase, then 14 × (internals +
    /// NextFloat scale + NextFloat fadeIn). The Goat (47) skips the type
    /// pick and sets noLightEmittance.
    #[test]
    fn w4_f6_unicorn_ring_mirrors_and_the_goat_skips_the_pick() {
        let w = world();
        let c = ctx();
        let anchor = Vec2::new(500.0, 400.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(13);
        unicorn_goat_dash_ring(10, anchor, 3, 12.0, -4.0, &mut rng, &mut pool, &c, &w);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(13);
        let ty = mirror.next_from_slice(&[176, 177, 179]);
        let phase = std::f32::consts::PI * 2.0 * mirror.next_float();
        for i in 0..14 {
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, anchor, 0, 0, ty, 0.0, 0.0, 0, Color::default(), 1.0,
            );
            let angle = i as f32 * (std::f32::consts::PI * 2.0) / 14.0 + phase;
            let mut spoke = rotated_by(Vec2::new(0.0, 1.0), angle as f64);
            spoke *= 0.2 * 3.0;
            let md = mpool.slot_mut(idx);
            md.position = anchor + spoke * Vec2::new(40.0, 30.0);
            md.velocity = spoke + Vec2::new(12.0 - (-1 * 3 * 2) as f32, 0.0);
            md.no_gravity = true;
            md.scale = 1.0 + mirror.next_float() * 0.8;
            md.fade_in = mirror.next_float() * 2.0;
        }
        assert_eq!(active(&pool).len(), 14);
        for i in 0..14 {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).scale, mpool.slot(i).scale);
            assert_eq!(pool.slot(i).fade_in, mpool.slot(i).fade_in);
            assert!(!pool.slot(i).no_light_emittance, "the Unicorn emits");
        }
        assert_eq!(rng.next(), mirror.next());
        // The Goat: type 6 fixed, no Next(3), noLightEmittance on all 14.
        let mut gpool = DustPool::new();
        let mut grng = UnifiedRandom::new(13);
        unicorn_goat_dash_ring(47, anchor, 3, 12.0, -4.0, &mut grng, &mut gpool, &c, &w);
        let mut gmpool = DustPool::new();
        let mut gmirror = UnifiedRandom::new(13);
        let gphase = std::f32::consts::PI * 2.0 * gmirror.next_float();
        for i in 0..14 {
            let idx = gmpool.new_dust(
                &mut gmirror, &c, &w, anchor, 0, 0, 6, 0.0, 0.0, 0, Color::default(), 1.0,
            );
            let angle = i as f32 * (std::f32::consts::PI * 2.0) / 14.0 + gphase;
            let mut spoke = rotated_by(Vec2::new(0.0, 1.0), angle as f64);
            spoke *= 0.2 * 3.0;
            let md = gmpool.slot_mut(idx);
            md.position = anchor + spoke * Vec2::new(40.0, 30.0);
            md.velocity = spoke + Vec2::new(12.0 - (-1 * 3 * 2) as f32, 0.0);
            md.no_gravity = true;
            md.no_light_emittance = true;
            md.scale = 1.0 + gmirror.next_float() * 0.8;
            md.fade_in = gmirror.next_float() * 2.0;
        }
        assert_eq!(active(&gpool).len(), 14);
        for i in 0..14 {
            assert_eq!(gpool.slot(i).dust_type, 6);
            assert!(gpool.slot(i).no_light_emittance);
            assert_eq!(gpool.slot(i).position, gmpool.slot(i).position);
            assert_eq!(gpool.slot(i).velocity, gmpool.slot(i).velocity);
        }
        assert_eq!(grng.next(), gmirror.next(), "no type pick for the Goat");
    }

    /// W4-f6 — site 3896: one dust, the Next(3) argument pick first, the
    /// velocity zeroed, scale/fadeIn drawn after the internals.
    #[test]
    fn w4_f6_unicorn_body_mirrors_the_argument_pick() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(300.0, 300.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(5);
        unicorn_dash_body(pos, 20, 42, &mut rng, &mut pool, &c, &w);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(5);
        let ty = mirror.next_from_slice(&[176, 177, 179]);
        let idx = mpool.new_dust(
            &mut mirror, &c, &w, pos, 20, 42, ty, 0.0, 0.0, 0, Color::default(), 1.0,
        );
        let md = mpool.slot_mut(idx);
        md.velocity = Vec2::ZERO;
        md.no_gravity = true;
        md.scale = 0.5 + mirror.next_float() * 0.8;
        md.fade_in = 1.0 + mirror.next_float() * 2.0;
        assert_eq!(active(&pool).len(), 1);
        assert_eq!(pool.slot(0).velocity, Vec2::ZERO);
        assert!(pool.slot(0).no_gravity);
        assert_eq!(pool.slot(0).scale, mpool.slot(0).scale);
        assert_eq!(pool.slot(0).fade_in, mpool.slot(0).fade_in);
        assert_eq!(rng.next(), mirror.next());
    }

    /// W4-f6 — sites 3939/4000: 20 dusts, the shared NextFloat phase, per
    /// dust the Next(2) half-gate AFTER the velocity writes; a rightward
    /// player flips X. The mirror lands identical records.
    #[test]
    fn w4_f6_dash_ring_20_mirrors_the_gate_and_flip() {
        let w = world();
        let c = ctx();
        let anchor = Vec2::new(700.0, 500.0);
        let vel = Vec2::new(9.0, 0.0); // rightward: the X flip fires
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(29);
        dash_ring_20(228, anchor, 14.0, vel, &mut rng, &mut pool, &c, &w);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(29);
        let phase = std::f32::consts::PI * 2.0 * mirror.next_float();
        for i in 0..20 {
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, anchor, 0, 0, 228, 0.0, 0.0, 0, Color::default(), 1.0,
            );
            let angle = i as f32 * (std::f32::consts::PI * 2.0) / 20.0 + phase;
            let mut spoke = rotated_by(Vec2::new(0.0, 1.0), angle as f64);
            spoke *= 0.8;
            let md = mpool.slot_mut(idx);
            md.position = anchor + spoke * Vec2::new(40.0, 30.0);
            md.velocity = spoke + Vec2::new(14.0 - 1.0, 0.0);
            md.velocity.x *= -1.0; // vel.x > 0
            if mirror.next_max(2) == 0 {
                md.velocity *= 0.5;
            }
            md.no_gravity = true;
            md.scale = 1.5 + mirror.next_float() * 0.8;
            md.fade_in = 0.0;
        }
        assert_eq!(active(&pool).len(), 20);
        for i in 0..20 {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).scale, mpool.slot(i).scale);
            assert_eq!(pool.slot(i).fade_in, 0.0);
            assert!(pool.slot(i).no_gravity);
        }
        assert_eq!(rng.next(), mirror.next());
    }

    /// W4-f6 — site 3970: the Next(num7) gate draws EVERY tick; num7 = 1
    /// (RunSpeed 1, vx 0) always fires. A failed gate spawns nothing but
    /// still advances the stream.
    #[test]
    fn w4_f6_pirate_puff_gate_draws_and_fires_at_num7_one() {
        let w = world();
        let c = ctx();
        let bottom = Vec2::new(400.0, 600.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(3);
        // RunSpeed 1, vx 0 -> num7 = 1 -> Next(1) == 0 always.
        assert!(pirate_run_puff(bottom, 20, 1.0, 0.0, &mut rng, &mut pool, &c, &w));
        assert_eq!(active(&pool).len(), 1);
        let d = pool.slot(0);
        assert_eq!(d.dust_type, 228);
        assert_eq!(d.velocity, Vec2::ZERO);
        assert!(d.no_gravity && d.no_light);
        // num9 = 20/2 + 10 = 20; the box anchor is bottom - (20, 16).
        assert!(d.position.x >= 380.0 && d.position.x < 380.0 + 40.0);
        assert!(d.position.y >= 584.0 && d.position.y < 584.0 + 22.0);
        // A failed gate: RunSpeed 6, vx 0 -> num7 = 6; probe the seed's
        // first Next(6) — if non-zero the puff refuses, stream advanced.
        let mut probe = UnifiedRandom::new(11);
        let gate = probe.next_max(6);
        let mut fpool = DustPool::new();
        let mut frng = UnifiedRandom::new(11);
        let fired =
            pirate_run_puff(bottom, 20, 6.0, 0.0, &mut frng, &mut fpool, &c, &w);
        assert_eq!(fired, gate == 0);
        if !fired {
            assert!(active(&fpool).is_empty());
            assert_eq!(frng.next(), probe.next(), "the gate draw ran");
        }
    }

    /// W4-f6 — site 4023: the SelectRandom(6,6,31) pick, the Next(2)
    /// noGravity gate, the Next(3) fadeIn gate, and the 31 override. The
    /// mirror replays the full chain for whichever type the seed picks.
    #[test]
    fn w4_f6_spooky_body_mirrors_the_full_chain() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(500.0, 500.0);
        let vel = Vec2::new(8.0, -1.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(17);
        spooky_dash_body(center, vel, &mut rng, &mut pool, &c, &w);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(17);
        let ty = mirror.next_from_slice(&[6i16, 6, 31]) as i32;
        let idx = mpool.new_dust(
            &mut mirror, &c, &w, center - Vec2::new(6.0, -6.0), 12, 12, ty,
            0.0, 0.0, 0, Color::default(), 1.0,
        );
        let md = mpool.slot_mut(idx);
        md.velocity = vel * 0.1;
        if mirror.next_max(2) == 0 {
            md.no_gravity = true;
        }
        md.scale = 0.7 + mirror.next_float() * 0.8;
        if mirror.next_max(3) == 0 {
            md.fade_in = 0.1;
        }
        if ty == 31 {
            md.no_gravity = true;
            md.scale *= 1.5;
            md.fade_in = 0.2;
        }
        assert_eq!(active(&pool).len(), 1);
        assert_eq!(pool.slot(0).dust_type, ty);
        assert_eq!(pool.slot(0).velocity, mpool.slot(0).velocity);
        assert_eq!(pool.slot(0).scale, mpool.slot(0).scale);
        assert_eq!(pool.slot(0).fade_in, mpool.slot(0).fade_in);
        assert_eq!(pool.slot(0).no_gravity, mpool.slot(0).no_gravity);
        assert_eq!(rng.next(), mirror.next());
    }

    /// W4-f6 — site 4103: the flag2 && vy == 0 gate; two type-31 noLight
    /// dusts with the (vx*0.15, NextFloat*-2) velocity. A slow or airborne
    /// player spawns nothing and draws nothing.
    #[test]
    fn w4_f6_basilisk_run_gates_and_mirrors() {
        let w = world();
        let c = ctx();
        let bl = Vec2::new(300.0, 542.0);
        // Slow: |vx| 2 <= RunSpeed/2 = 5 -> no dust, no draws.
        let mut slow = DustPool::new();
        let mut srng = UnifiedRandom::new(2);
        basilisk_run_dust(bl, 20, 2.0, 0.0, 10.0, &mut srng, &mut slow, &c, &w);
        assert!(active(&slow).is_empty());
        let mut probe = UnifiedRandom::new(2);
        assert_eq!(srng.next(), probe.next(), "the gate draws nothing");
        // Airborne: vy != 0 -> no dust.
        let mut air = DustPool::new();
        let mut arng = UnifiedRandom::new(2);
        basilisk_run_dust(bl, 20, 8.0, 3.0, 10.0, &mut arng, &mut air, &c, &w);
        assert!(active(&air).is_empty());
        // Fast + grounded: two dusts, mirrored.
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(2);
        basilisk_run_dust(bl, 20, 8.0, 0.0, 10.0, &mut rng, &mut pool, &c, &w);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(2);
        for _ in 0..2 {
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, bl, 20, 6, 31, 0.0, 0.0, 0, Color::default(), 1.0,
            );
            let md = mpool.slot_mut(idx);
            md.velocity = Vec2::new(8.0 * 0.15, mirror.next_float() * -2.0);
            md.no_light = true;
            md.scale = 0.5 + mirror.next_float() * 0.8;
            md.fade_in = 0.5 + mirror.next_float() * 1.0;
        }
        assert_eq!(active(&pool).len(), 2);
        for i in 0..2 {
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).scale, mpool.slot(i).scale);
            assert!(pool.slot(i).no_light);
            assert_eq!(pool.slot(i).dust_type, 31);
        }
        assert_eq!(rng.next(), mirror.next());
    }

    /// W4-f6 — the charge accumulator: zero below the gate, +1/180 above,
    /// clamped at 1. The dust loop counts (int)(3*charge) and breaks on a
    /// facing mismatch.
    #[test]
    fn w4_f6_basilisk_charge_accumulates_and_counts() {
        assert_eq!(basilisk_charge_update(0.5, 2.0, 10.0), 0.0, "slow zeroes");
        let c1 = basilisk_charge_update(0.0, 8.0, 10.0);
        assert!((c1 - 1.0 / 180.0).abs() < 1e-7);
        assert_eq!(basilisk_charge_update(1.0, 8.0, 10.0), 1.0, "the clamp");
        let w = world();
        let c = ctx();
        let center = Vec2::new(500.0, 500.0);
        let bl = Vec2::new(490.0, 542.0);
        // Facing mismatch: Sign(vx) = 1 != -1 -> no dust, no draws.
        let mut miss = DustPool::new();
        let mut mrng = UnifiedRandom::new(4);
        basilisk_charge_dust(
            1.0, center, bl, 20, -1, Vec2::new(8.0, 0.0), 10.0,
            &mut mrng, &mut miss, &c, &w,
        );
        assert!(active(&miss).is_empty());
        let mut probe = UnifiedRandom::new(4);
        assert_eq!(mrng.next(), probe.next());
        // charge 1 -> 3 dusts; the mirror replays the per-dust chain.
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(4);
        let vel = Vec2::new(8.0, 0.0);
        basilisk_charge_dust(
            1.0, center, bl, 20, 1, vel, 10.0, &mut rng, &mut pool, &c, &w,
        );
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(4);
        let aim = center + Vec2::new(40.0, 12.0);
        for _ in 0..3 {
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, bl, 20, 6, 6, 0.0, 0.0, 0, Color::default(), 1.0,
            );
            let md = mpool.slot_mut(idx);
            md.position = center + Vec2::new(38.0, 6.0 + mirror.next_float() * 12.0);
            md.velocity =
                safe_normalize(md.position - aim, Vec2::ZERO) * (3.5 + mirror.next_float() * 0.5);
            if md.velocity.y < 0.0 {
                md.velocity.y *= 1.0 + 2.0 * mirror.next_float();
            }
            md.velocity += vel * 0.55;
            md.velocity *= vel.length() / 10.0;
            md.velocity *= 1.0;
            md.no_gravity = true;
            md.no_light = true;
            md.scale = 0.5 + mirror.next_float() * 0.8;
            md.fade_in = 0.5 + mirror.next_float() * 1.0;
        }
        assert_eq!(active(&pool).len(), 3);
        for i in 0..3 {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).scale, mpool.slot(i).scale);
            assert_eq!(pool.slot(i).fade_in, mpool.slot(i).fade_in);
        }
        assert_eq!(rng.next(), mirror.next());
    }

    /// W4-f6 — the refusal path: an off-screen cull returns the sentinel
    /// and the post-writes land on slot 6000 exactly as the C#'s
    /// `Main.dust[6000]` does (the e2 precedent).
    #[test]
    fn w4_f6_culled_spawn_writes_the_sentinel() {
        let w = world();
        let mut c = ctx();
        c.screen_position = Vec2::new(10000.0, 10000.0);
        c.screen_size = Vec2::new(100.0, 100.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(9);
        drill_beam_dust(
            Vec2::new(100.0, 100.0),
            Vec2::new(90.0, 90.0),
            &mut rng,
            &mut pool,
            &c,
            &w,
        );
        assert!(active(&pool).is_empty(), "both spawns culled");
        assert!(pool.slot(6000).no_gravity, "the sentinel takes the write");
        assert!(!pool.slot(6000).active, "the sentinel stays dead");
    }

    // ---- W4-f6 batch 2 witnesses (the Mount effects sites) ----

    /// W4-f6 — site 4626 (DEAD EmitGolfCartSmoke): one type-31 Direct dust,
    /// the velocity chain lands vx' = (-dir + jitter)*0.1 - dir + Sign(vx)*1.3.
    /// The mirror replays the same pool call + post-writes.
    #[test]
    fn w4_f6_golf_smoke_mirrors_the_velocity_chain() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(400.0, 600.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(31);
        golf_cart_smoke(pos, 1, 5.0, true, &mut rng, &mut pool, &c, &w);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(31);
        let idx = mpool.new_dust_direct(
            &mut mirror, &c, &w, pos, 0, 0, 31, -1.0, -0.24, 100, Color::default(), 1.0,
        );
        let md = mpool.slot_mut(idx);
        md.position = pos;
        md.velocity *= 0.1;
        md.velocity += Vec2::new(-1.0, -0.25);
        md.scale = 0.5;
        md.velocity += Vec2::new(1.3, 0.0); // Sign(5) = 1
        md.fade_in = 0.8;
        assert_eq!(active(&pool).len(), 1);
        assert_eq!(pool.slot(0).position, pos);
        assert_eq!(pool.slot(0).velocity, mpool.slot(0).velocity);
        assert_eq!(pool.slot(0).scale, 0.5);
        assert_eq!(pool.slot(0).fade_in, 0.8);
        assert_eq!(pool.slot(0).dust_type, 31);
        assert_eq!(rng.next(), mirror.next());
        // The non-rushing arm leaves fadeIn at the NewDust default (0).
        let mut npool = DustPool::new();
        let mut nrng = UnifiedRandom::new(31);
        golf_cart_smoke(pos, 1, 0.0, false, &mut nrng, &mut npool, &c, &w);
        assert_eq!(npool.slot(0).fade_in, 0.0, "no rushing, no fadeIn");
        // vx == 0 adds no X kick: vx' = (-1 + jitter)*0.1 - 1, jitter in
        // [-2, 2] (the NewDust default-block velocity roll, Dust.cs:170),
        // so vx' in [-1.3, -0.9].
        let vx0 = npool.slot(0).velocity.x;
        assert!(vx0 >= -1.31 && vx0 <= -0.89, "no Sign kick at vx 0: {vx0}");
    }

    /// W4-f6 — site 4703: the Next(10) gate draws every tick; a pass spawns
    /// one Perfect dust with the square offset (2 NextDouble) and the
    /// noGravity/0.6/0.4/×0.25 chain. The mirror lands identical records.
    #[test]
    fn w4_f6_gem_sparkle_gates_and_mirrors() {
        let w = world();
        let c = ctx();
        let anchor = Vec2::new(500.0, 500.0);
        // Find a seed whose first Next(10) is 0 (the gate fires).
        let mut seed = 0;
        loop {
            let mut probe = UnifiedRandom::new(seed);
            if probe.next_max(10) == 0 {
                break;
            }
            seed += 1;
        }
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(seed);
        assert!(gem_minecart_sparkle(anchor, 86, &mut rng, &mut pool, &c, &w));
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(seed);
        let _ = mirror.next_max(10);
        let offset = next_vector2_square(&mut mirror, -1.0, 1.0) * Vec2::new(22.0, 10.0);
        let idx = mpool.new_dust_perfect(
            &mut mirror, &c, &w, anchor + offset, 86, None, 0, Color::default(), 1.0,
        );
        let md = mpool.slot_mut(idx);
        md.no_gravity = true;
        md.fade_in = 0.6;
        md.scale = 0.4;
        md.velocity *= 0.25;
        assert_eq!(active(&pool).len(), 1);
        assert_eq!(pool.slot(0).position, anchor + offset);
        assert_eq!(pool.slot(0).velocity, mpool.slot(0).velocity);
        assert_eq!(pool.slot(0).scale, 0.4);
        assert_eq!(pool.slot(0).fade_in, 0.6);
        assert!(pool.slot(0).no_gravity);
        assert_eq!(rng.next(), mirror.next());
        // A failed gate spawns nothing but advances the stream one draw.
        let mut fseed = seed;
        loop {
            let mut probe = UnifiedRandom::new(fseed);
            if probe.next_max(10) != 0 {
                break;
            }
            fseed += 1;
        }
        let mut fpool = DustPool::new();
        let mut frng = UnifiedRandom::new(fseed);
        assert!(!gem_minecart_sparkle(anchor, 86, &mut frng, &mut fpool, &c, &w));
        assert!(active(&fpool).is_empty());
        let mut probe = UnifiedRandom::new(fseed);
        let _ = probe.next_max(10);
        assert_eq!(frng.next(), probe.next(), "the gate draw ran");
    }

    /// W4-f6 — site 4719: the steam gate is rand-free (|vx| >= 1 and
    /// (|vx| >= 6 or frame 0)); a pass spawns one Perfect dust with the
    /// (3,3) square offset and the Y -= 2 tail.
    #[test]
    fn w4_f6_steam_puff_gates_and_mirrors() {
        let w = world();
        let c = ctx();
        let anchor = Vec2::new(600.0, 500.0);
        // Slow + non-zero frame: no dust, no draws.
        let mut slow = DustPool::new();
        let mut srng = UnifiedRandom::new(2);
        assert!(!steam_minecart_puff(anchor, 303, 0.5, 1, &mut srng, &mut slow, &c, &w));
        assert!(active(&slow).is_empty());
        let mut probe = UnifiedRandom::new(2);
        assert_eq!(srng.next(), probe.next(), "the gate draws nothing");
        // Mid-speed + frame 0: fires.
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(2);
        assert!(steam_minecart_puff(anchor, 303, 3.0, 0, &mut rng, &mut pool, &c, &w));
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(2);
        let offset = next_vector2_square(&mut mirror, -1.0, 1.0) * Vec2::new(3.0, 3.0);
        let idx = mpool.new_dust_perfect(
            &mut mirror, &c, &w, anchor + offset, 303, None, 0, Color::default(), 1.0,
        );
        let md = mpool.slot_mut(idx);
        md.no_gravity = true;
        md.fade_in = 0.6;
        md.scale = 1.8;
        md.velocity *= 0.25;
        md.velocity.y -= 2.0;
        assert_eq!(active(&pool).len(), 1);
        assert_eq!(pool.slot(0).position, anchor + offset);
        assert_eq!(pool.slot(0).velocity, mpool.slot(0).velocity);
        assert_eq!(pool.slot(0).scale, 1.8);
        assert_eq!(rng.next(), mirror.next());
        // Fast (>= 6) fires regardless of frame.
        let mut fast = DustPool::new();
        let mut frng = UnifiedRandom::new(2);
        assert!(steam_minecart_puff(anchor, 303, 7.0, 3, &mut frng, &mut fast, &c, &w));
        assert_eq!(active(&fast).len(), 1);
    }

    /// W4-f6 — site 4750: the exhaust gate's Next(4) draws ONLY when slow;
    /// num2 = 1 + (int)num/6 dusts, each decrementing num2 before the spawn.
    /// A fast player (num >= 1) skips the gate draw entirely.
    #[test]
    fn w4_f6_exhaust_smoke_counts_and_mirrors() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(500.0, 500.0);
        // num = 7 -> num2 = 2 dusts; the gate draw is skipped (num >= 1).
        let vel = Vec2::new(7.0, 0.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(8);
        exhaust_minecart_smoke(center, 1, vel, 31, &mut rng, &mut pool, &c, &w);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(8);
        let num = vel.length();
        let mut num2 = 1 + (num as i32) / 6;
        let mut count = 0;
        while num2 > 0 {
            num2 -= 1;
            let offset = next_vector2_square(&mut mirror, -1.0, 1.0) * Vec2::new(3.0, 3.0);
            let mut anchor = Vec2::new(-18.0, 20.0);
            if num > 6.0 {
                anchor.x += 4.0;
            }
            if num2 > 0 {
                anchor += vel * (num2 / 3) as f32;
            }
            let idx = mpool.new_dust_perfect(
                &mut mirror, &c, &w, center + anchor + offset, 31, None, 0,
                Color::default(), 1.0,
            );
            let md = mpool.slot_mut(idx);
            md.no_gravity = true;
            md.fade_in = 0.6;
            md.scale = 1.2;
            md.velocity *= 0.2;
            count += 1;
        }
        assert_eq!(count, 2, "num2 = 1 + 7/6 = 2");
        assert_eq!(active(&pool).len(), 2);
        for i in 0..2 {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).scale, 1.2);
        }
        assert_eq!(rng.next(), mirror.next());
        // A slow player (num < 1) draws the Next(4) gate; on a fail, nothing.
        let mut fseed = 0;
        loop {
            let mut probe = UnifiedRandom::new(fseed);
            if probe.next_max(4) != 0 {
                break;
            }
            fseed += 1;
        }
        let mut fpool = DustPool::new();
        let mut frng = UnifiedRandom::new(fseed);
        exhaust_minecart_smoke(center, 1, Vec2::new(0.3, 0.0), 31, &mut frng, &mut fpool, &c, &w);
        assert!(active(&fpool).is_empty(), "the slow gate refused");
        let mut probe = UnifiedRandom::new(fseed);
        let _ = probe.next_max(4);
        assert_eq!(frng.next(), probe.next(), "the Next(4) ran");
    }

    /// W4-f6 — site 4782: the confetti gate's two Next draws (Next(6) when
    /// num < 1, Next(3) when num < 3 and the first clause passed); each dust
    /// draws NextFloat wobble + square offset + Next(4) type. The mirror
    /// replays the full chain.
    #[test]
    fn w4_f6_confetti_burst_mirrors_the_chain() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(500.0, 500.0);
        // num = 4 -> both gates skipped (num >= 3); num2 = 1 dust.
        let vel = Vec2::new(4.0, 0.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(12);
        confetti_minecart_burst(center, -1, vel, &mut rng, &mut pool, &c, &w);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(12);
        let num = vel.length();
        let dir = -1.0f32;
        let mut num2 = 1 + (num as i32) / 6;
        let mut count = 0;
        while num2 > 0 {
            num2 -= 1;
            let num3 = mirror.next_float() * 2.0;
            let offset = next_vector2_square(&mut mirror, -1.0, 1.0) * Vec2::new(3.0, 8.0);
            let mut anchor = Vec2::new(-18.0, 4.0) * Vec2::new(dir, 1.0);
            anchor.x += num * dir * 0.5 + (-1 * num2) as f32 * num3;
            if num2 > 0 {
                anchor += vel * (num2 / 3) as f32;
            }
            let ty = 139 + mirror.next_max(4);
            let idx = mpool.new_dust_perfect(
                &mut mirror, &c, &w, center + anchor + offset, ty, None, 0,
                Color::default(), 1.0,
            );
            let md = mpool.slot_mut(idx);
            md.no_gravity = true;
            md.fade_in = 0.6;
            md.scale = 0.5 + num3 / 2.0;
            md.velocity *= 0.2;
            count += 1;
        }
        assert_eq!(count, 1, "num2 = 1 + 4/6 = 1");
        assert_eq!(active(&pool).len(), 1);
        assert_eq!(pool.slot(0).position, mpool.slot(0).position);
        assert_eq!(pool.slot(0).velocity, mpool.slot(0).velocity);
        assert_eq!(pool.slot(0).scale, mpool.slot(0).scale);
        assert_eq!(pool.slot(0).dust_type, mpool.slot(0).dust_type);
        assert!(pool.slot(0).dust_type >= 139 && pool.slot(0).dust_type <= 142);
        assert_eq!(rng.next(), mirror.next());
    }

    /// W4-f6 — site 4824: the Next(45) gate, the circular offset (2 draws),
    /// the Perfect yellow 43 dust with the DirectionTo velocity when the
    /// offset is non-zero. The mirror lands identical records.
    #[test]
    fn w4_f6_broom_sparkle_gates_and_mirrors() {
        let w = world();
        let c = ctx();
        let tp = Vec2::new(520.0, 480.0);
        let mut seed = 0;
        loop {
            let mut probe = UnifiedRandom::new(seed);
            if probe.next_max(45) == 0 {
                break;
            }
            seed += 1;
        }
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(seed);
        assert!(broom_trinket_sparkle(tp, &mut rng, &mut pool, &c, &w));
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(seed);
        let _ = mirror.next_max(45);
        let offset = next_vector2_circular(&mut mirror, 4.0, 4.0);
        let idx = mpool.new_dust_perfect(
            &mut mirror, &c, &w, tp + offset, 43, Some(Vec2::ZERO), 254,
            Color::new(255, 255, 0, 255), 0.3,
        );
        let md = mpool.slot_mut(idx);
        if offset != Vec2::ZERO {
            md.velocity = direction_to(tp, md.position) * 0.2;
        }
        md.fade_in = 0.3;
        md.no_light_emittance = true;
        assert_eq!(active(&pool).len(), 1);
        assert_eq!(pool.slot(0).position, tp + offset);
        assert_eq!(pool.slot(0).velocity, mpool.slot(0).velocity);
        assert_eq!(pool.slot(0).color, Color::new(255, 255, 0, 255));
        assert_eq!(pool.slot(0).alpha, 254);
        assert_eq!(pool.slot(0).fade_in, 0.3);
        assert!(pool.slot(0).no_light_emittance);
        assert_eq!(rng.next(), mirror.next());
    }

    /// W4-f6 — site 4849: the NextFloat gate (draws every tick), the phase
    /// NextFloat, then 8 Next(15) gates; a fired gate spawns the rotated
    /// spoke dust. The mirror replays the whole loop.
    #[test]
    fn w4_f6_broom_trail_mirrors_the_loop() {
        let w = world();
        let c = ctx();
        let pos4 = Vec2::new(500.0, 500.0);
        let pos5 = Vec2::new(552.0, 494.0);
        // vel_len 0 -> num12 = 0.1. The impl's FIRST NextFloat is the pos4
        // jitter (`:4835`, inert at facing 0); the gate is the SECOND.
        let mut seed = 0;
        loop {
            let mut probe = UnifiedRandom::new(seed);
            let _ = probe.next_float(); // the pos4 jitter
            if probe.next_float() <= 0.1 {
                break;
            }
            seed += 1;
        }
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(seed);
        assert!(broom_trinket_trail(pos4, 0, |v| v, pos5, 0.0, 0.0, &mut rng, &mut pool, &c, &w));
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(seed);
        let _ = mirror.next_float(); // the pos4 jitter (`:4835`)
        let _ = mirror.next_float(); // the gate (`:4839`)
        let num13 = mirror.next_float();
        let mut num14 = 0.0f32;
        let mut fired = 0;
        while num14 < 1.0 {
            if mirror.next_max(15) == 0 {
                let angle = std::f32::consts::PI * 2.0 * num14 + num13;
                let unit = Vec2::new(
                    (angle as f64).cos() as f32,
                    (angle as f64).sin() as f32,
                );
                let spoke = rotated_by(unit * Vec2::new(0.5, 1.0) * 4.0, 0.0);
                let idx = mpool.new_dust_perfect(
                    &mut mirror, &c, &w, pos4 + spoke, 43, Some(Vec2::ZERO), 254,
                    Color::new(255, 255, 0, 255), 0.3,
                );
                let md = mpool.slot_mut(idx);
                md.velocity = spoke * 0.025 + direction_to(pos5, md.position) * 0.5;
                md.fade_in = 0.3;
                md.no_light_emittance = true;
                fired += 1;
            }
            num14 += 0.125;
        }
        assert_eq!(active(&pool).len(), fired);
        for i in 0..fired {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
        }
        assert_eq!(rng.next(), mirror.next());
        // A failed gate (the SECOND NextFloat > num12) spawns nothing, two
        // draws (the pos4 jitter + the gate).
        let mut fseed = seed;
        loop {
            let mut probe = UnifiedRandom::new(fseed);
            let _ = probe.next_float(); // the pos4 jitter
            if probe.next_float() > 0.1 {
                break;
            }
            fseed += 1;
        }
        let mut fpool = DustPool::new();
        let mut frng = UnifiedRandom::new(fseed);
        assert!(!broom_trinket_trail(pos4, 0, |v| v, pos5, 0.0, 0.0, &mut frng, &mut fpool, &c, &w));
        assert!(active(&fpool).is_empty());
        let mut probe = UnifiedRandom::new(fseed);
        let _ = probe.next_float(); // the pos4 jitter
        let _ = probe.next_float(); // the gate
        assert_eq!(frng.next(), probe.next());
    }

    /// W4-f6 — sites 5061/5071: a > 3 px gap spawns ceil(dist/3) lerped
    /// dusts; a <= 3 px gap spawns one at the now-anchor. Each is a type-182
    /// noGravity zero-velocity dust.
    #[test]
    fn w4_f6_drill_trail_lerps_and_single() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(500.0, 500.0);
        let old = Vec2::new(500.0, 600.0);
        let now = Vec2::new(500.0, 590.0); // dist 10 -> ceil(10/3) = 4
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(6);
        drill_mount_trail(center, now, old, &mut rng, &mut pool, &c, &w);
        assert_eq!(active(&pool).len(), 4, "ceil(10/3) = 4");
        // The lerp positions: old + (now-old)*(i/4) for i = 1..4.
        for i in 0..4 {
            let t = (i + 1) as f32 / 4.0;
            let want = old + (now - old) * t;
            assert_eq!(pool.slot(i).position, want);
            assert_eq!(pool.slot(i).dust_type, 182);
            assert!(pool.slot(i).no_gravity);
            assert_eq!(pool.slot(i).velocity, Vec2::ZERO);
        }
        // A short gap (dist 2) spawns one at the now-anchor.
        let mut spool = DustPool::new();
        let mut srng = UnifiedRandom::new(6);
        let near = Vec2::new(500.0, 598.0);
        drill_mount_trail(center, near, old, &mut srng, &mut spool, &c, &w);
        assert_eq!(active(&spool).len(), 1);
        assert_eq!(spool.slot(0).position, near);
        assert_eq!(spool.slot(0).dust_type, 182);
    }

    /// W4-f6 — site 5099: the lava trail spawns on the NextFloat gate FAIL;
    /// num5 = clamp(vel/5, 0, 1). At vel 0 (num5 0) every gate fails, so all
    /// 10 f32-step iterations spawn. The mirror replays the per-step
    /// circular velocity + the Next(10) override.
    #[test]
    fn w4_f6_lava_trail_spawns_on_gate_fail() {
        let w = world();
        let c = ctx();
        let value = Vec2::new(480.0, 520.0);
        let anchor = Vec2::new(520.0, 510.0);
        let delta = Vec2::new(40.0, -10.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(19);
        lava_cart_trail(value, anchor, delta, 0.0, &mut rng, &mut pool, &c, &w);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(19);
        let num5 = 0.0f32;
        let mut num6 = 0.0f32;
        let mut count = 0;
        while num6 <= 1.0 {
            if !(mirror.next_float() < num5) {
                let vel = next_vector2_circular(&mut mirror, 0.5, 0.5) * num5;
                let idx = mpool.new_dust_perfect(
                    &mut mirror, &c, &w, value + (anchor - value) * num6, 65, Some(vel), 0,
                    Color::default(), 1.0,
                );
                let md = mpool.slot_mut(idx);
                md.scale = 0.6;
                md.fade_in = 0.0;
                md.velocity *= -1.0;
                md.no_gravity = true;
                md.velocity -= delta;
                if mirror.next_max(10) == 0 {
                    md.fade_in = 1.3;
                    md.velocity = next_vector2_circular(&mut mirror, 3.0, 3.0) * num5;
                }
                count += 1;
            }
            num6 += 0.1;
        }
        assert_eq!(active(&pool).len(), count);
        assert_eq!(count, 10, "the f32 0.1 step runs 10 iterations");
        for i in 0..count {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).scale, 0.6);
            assert!(pool.slot(i).no_gravity);
        }
        assert_eq!(rng.next(), mirror.next());
    }

    // ---- W4-f6 batch 3 witnesses (the Mount spawn/dismount sites) ----

    /// W4-f6 — the fairy colour's generic path (no dev name): the
    /// (hueRaw+0.5)%1 hue, hslToRgb * opacity, white lerp, alpha multiply.
    /// The 0.41/0.15/0.7 Pixie arm lands a fixed colour (zero rand).
    #[test]
    fn w4_f6_fairy_color_generic_path_is_fixed() {
        let c = fairy_queen_weapons_color_full(
            "SomePlayer", true, Vec2::ZERO, Vec2::ZERO, 0.41, 1.0, 0.15, 1.0, 0.7,
        );
        // The generic path: num = (0.41+0.5)%1, hslToRgb(num, 1, 0.7),
        // opacity 1 (no-op), white-lerp 0.15 (byte trunc), alpha 255. Derive
        // the expectation through the SAME hsl_to_rgb the fn uses so the
        // f32 hue rounding cannot diverge (the value is ~ (255, 124, 195)).
        let num = (0.41f32 + 0.5) % 1.0;
        let rgb = crate::ui::color::hsl_to_rgb(num as f64, 1.0, 0.7, 255);
        let want = Color::new(
            (rgb[0] as f32 + (255.0 - rgb[0] as f32) * 0.15) as u8,
            (rgb[1] as f32 + (255.0 - rgb[1] as f32) * 0.15) as u8,
            (rgb[2] as f32 + (255.0 - rgb[2] as f32) * 0.15) as u8,
            255,
        );
        assert_eq!(c, want);
        // The inactive-player gate skips the dev switch (same generic path).
        let inactive = fairy_queen_weapons_color_full(
            "Redigit", false, Vec2::ZERO, Vec2::ZERO, 0.41, 1.0, 0.15, 1.0, 0.7,
        );
        assert_eq!(inactive, c, "inactive skips the dev-name switch");
        // A dev name with no early return (Redigit pins num 0.7) differs.
        let redigit = fairy_queen_weapons_color_full(
            "Redigit", true, Vec2::ZERO, Vec2::ZERO, 0.41, 1.0, 0.0, 1.0, 0.5,
        );
        let want = crate::ui::color::hsl_to_rgb(0.7, 1.0, 0.5, 255);
        assert_eq!(redigit, Color::new(want[0], want[1], want[2], 255));
    }

    /// W4-f6 — site 5149: the Next(15) flag draws every tick; the time gate
    /// (even timeForVisualEffects) precedes the Next(6) speed gate (which
    /// draws only when !flag). A fired spawn mirrors the full chain.
    #[test]
    fn w4_f6_pixie_tail_gates_and_mirrors() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(500.0, 500.0);
        let vel = Vec2::new(6.0, 0.0);
        // Odd time: no spawn, only the Next(15) draws.
        let mut odd = DustPool::new();
        let mut orng = UnifiedRandom::new(3);
        assert!(!pixie_tail(center, vel, 1.0, "Player", &mut orng, &mut odd, &c, &w));
        assert!(active(&odd).is_empty());
        let mut probe = UnifiedRandom::new(3);
        let _ = probe.next_max(15);
        assert_eq!(orng.next(), probe.next(), "only the Next(15) ran");
        // Even time + fast speed: find a seed where the spawn fires.
        let mut seed = 0;
        loop {
            let mut p = UnifiedRandom::new(seed);
            let flag = p.next_max(15) == 0;
            if flag || ((p.next_max(6) + 1) as f32) < vel.length() {
                break;
            }
            seed += 1;
        }
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(seed);
        assert!(pixie_tail(center, vel, 0.0, "Player", &mut rng, &mut pool, &c, &w));
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(seed);
        let flag = mirror.next_max(15) == 0;
        if !flag {
            let _ = mirror.next_max(6);
        }
        let fa = fairy_queen_weapons_color_full("Player", true, center, center, 0.41, 1.0, 0.45, 1.0, 0.7);
        let fb = fairy_queen_weapons_color_full("Player", true, center, center, 0.41, 1.0, 0.0, 1.0, 0.7);
        let color = color_lerp(fa, fb, mirror.next_float());
        let idx = mpool.new_dust_direct(
            &mut mirror, &c, &w, center, 0, 0, 278, 0.0, 0.0, 200, color, 0.65,
        );
        let md = mpool.slot_mut(idx);
        md.position = center + Vec2::new(0.0, -2.0);
        if flag {
            md.velocity *= 0.4;
        } else {
            md.velocity *= 0.04 * vel.length();
        }
        md.velocity += vel * 0.3;
        md.position += vel * 0.7;
        let angle = mirror.next_float() * (std::f32::consts::PI * 2.0);
        let unit = Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32);
        md.position += unit * mirror.next_float() * 2.0;
        md.no_gravity = true;
        md.no_light = true;
        assert_eq!(active(&pool).len(), 1);
        assert_eq!(pool.slot(0).position, mpool.slot(0).position);
        assert_eq!(pool.slot(0).velocity, mpool.slot(0).velocity);
        assert_eq!(pool.slot(0).color, mpool.slot(0).color);
        assert_eq!(pool.slot(0).dust_type, 278);
        assert!(pool.slot(0).no_gravity && pool.slot(0).no_light);
        assert_eq!(rng.next(), mirror.next());
    }

    /// W4-f6 — site 6339: the failed-dismount burst spawns dust_amount
    /// dusts; the Bat (56) pins scale 0.7 + the velocity chain, the
    /// noGravity/Next(2) gate draws per dust. The mirror replays the chain.
    #[test]
    fn w4_f6_failed_dismount_mirrors_the_bat_chain() {
        let w = world();
        let c = ctx();
        let tl = Vec2::new(480.0, 480.0);
        let vel = Vec2::new(3.0, -1.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(23);
        // Bat 56, spawnDust 31, noGravity false, 3 dusts.
        failed_dismount_dust(
            56, 31, false, tl, 40, 62, 3, vel, "Player", Vec2::new(500.0, 500.0),
            &mut rng, &mut pool, &c, &w,
        );
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(23);
        for _ in 0..3 {
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, tl, 40, 62, 31, 0.0, 0.0, 0,
                Color::new(0, 0, 0, 255), 1.0,
            );
            let md = mpool.slot_mut(idx);
            md.scale += mirror.next_range(-10, 21) as f32 * 0.01;
            md.scale = 0.7;
            md.velocity *= 0.4;
            md.velocity.y += 0.5;
            // spawnDustNoGravity false -> the Next(2) gate.
            if mirror.next_max(2) == 0 {
                md.scale *= 1.3;
                md.no_gravity = true;
            } else {
                md.velocity *= 0.5;
            }
            md.velocity += vel * 0.8;
        }
        assert_eq!(active(&pool).len(), 3);
        for i in 0..3 {
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).scale, mpool.slot(i).scale);
            assert_eq!(pool.slot(i).no_gravity, mpool.slot(i).no_gravity);
            assert_eq!(pool.slot(i).color, Color::new(0, 0, 0, 255));
        }
        assert_eq!(rng.next(), mirror.next());
    }

    /// W4-f6 — sites 6382/6391: the Wolf burst spawns 100 type-267 dusts
    /// plus a clone per Next(3) == 0. The mirror replays the per-dust chain
    /// and the clone's white/half-scale/zero-alpha writes.
    #[test]
    fn w4_f6_wolf_burst_mirrors_the_clones() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(500.0, 500.0);
        let vel = Vec2::new(2.0, 0.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(41);
        wolf_spawn_dust(pos, 20, 42, vel, &mut rng, &mut pool, &c, &w);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(41);
        let anchor = Vec2::new(pos.x - 20.0, pos.y);
        let mut total = 0;
        let mut clones = 0;
        for _ in 0..100 {
            let idx = mpool.new_dust_direct(
                &mut mirror, &c, &w, anchor, 60, 42, 267, 0.0, 0.0, 60,
                Color::new(130, 60, 255, 70), 1.0,
            );
            let md = mpool.slot_mut(idx);
            md.scale += mirror.next_range(-10, 21) as f32 * 0.01;
            md.no_gravity = true;
            md.velocity += vel * 0.8;
            md.velocity *= mirror.next_float();
            md.velocity.y += 2.0 * mirror.next_float_direction();
            md.no_light = true;
            total += 1;
            if mirror.next_max(3) == 0 {
                let cidx = mpool.clone_dust(&mut mirror, &c, &w, idx);
                let cd = mpool.slot_mut(cidx);
                cd.color = Color::WHITE;
                cd.scale *= 0.5;
                cd.alpha = 0;
                clones += 1;
                total += 1;
            }
        }
        assert_eq!(active(&pool).len(), total);
        assert!(clones > 0, "some Next(3) gates fire over 100 dusts");
        // The first dust's record matches the mirror.
        assert_eq!(pool.slot(0).velocity, mpool.slot(0).velocity);
        assert_eq!(pool.slot(0).scale, mpool.slot(0).scale);
        assert_eq!(pool.slot(0).dust_type, 267);
        assert!(pool.slot(0).no_light);
        assert_eq!(rng.next(), mirror.next());
    }

    /// W4-f6 — sites 6404/6413: the Chillet burst spawns 100 type-306 dusts
    /// (scale 2 argument) plus clones; the Y kick is 4*NextFloatDirection.
    #[test]
    fn w4_f6_chillet_burst_mirrors_the_clones() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(600.0, 500.0);
        let vel = Vec2::new(-2.0, 1.0);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(53);
        chillet_spawn_dust(pos, 20, 42, vel, &mut rng, &mut pool, &c, &w);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(53);
        let anchor = Vec2::new(pos.x - 20.0, pos.y);
        let mut total = 0;
        for _ in 0..100 {
            let idx = mpool.new_dust_direct(
                &mut mirror, &c, &w, anchor, 60, 42, 306, 0.0, 0.0, 60,
                Color::new(100, 227, 255, 127), 2.0,
            );
            let md = mpool.slot_mut(idx);
            md.scale += mirror.next_range(-10, 21) as f32 * 0.01;
            md.no_gravity = true;
            md.velocity += vel * 0.8;
            md.velocity *= mirror.next_float();
            md.velocity.y += 4.0 * mirror.next_float_direction();
            md.no_light = true;
            total += 1;
            if mirror.next_max(3) == 0 {
                let cidx = mpool.clone_dust(&mut mirror, &c, &w, idx);
                let cd = mpool.slot_mut(cidx);
                cd.color = Color::WHITE;
                cd.scale *= 0.5;
                cd.alpha = 0;
                total += 1;
            }
        }
        assert_eq!(active(&pool).len(), total);
        assert_eq!(pool.slot(0).dust_type, 306);
        assert_eq!(pool.slot(0).velocity, mpool.slot(0).velocity);
        assert_eq!(pool.slot(0).scale, mpool.slot(0).scale);
        assert_eq!(rng.next(), mirror.next());
    }

    /// W4-f6 — site 6493: the generic tail's cart arm spawns NO dust (the
    /// gore argument draws only); the roller-skate count is 40; the Pixie
    /// continue gate draws Next(5) per non-dismount iteration.
    #[test]
    fn w4_f6_generic_tail_cart_and_skates_and_pixie() {
        let w = world();
        let c = ctx();
        let tl = Vec2::new(480.0, 480.0);
        let vel = Vec2::new(2.0, 0.0);
        let center = Vec2::new(500.0, 500.0);
        // Cart: 100 iterations, every 10th a gore (2 draws), no dust.
        let mut cart = DustPool::new();
        let mut crng = UnifiedRandom::new(61);
        generic_spawn_dust(
            6, 213, false, true, false, tl, 60, 42, vel, "Player", center,
            &mut crng, &mut cart, &c, &w,
        );
        assert!(active(&cart).is_empty(), "the cart arm spawns no dust");
        let mut cprobe = UnifiedRandom::new(61);
        for k in 0..100 {
            if k % 10 == 0 {
                let _ = cprobe.next_range(61, 64);
                let _ = cprobe.next_double() * 6.2831854820251465;
            }
        }
        assert_eq!(crng.next(), cprobe.next(), "10 gore argument pairs");
        // Roller skates (57): 40 dusts, the velocity chain, no Next(2) gate
        // (spawnDustNoGravity true for the skates' row? use false to exercise
        // the gate). The mirror replays the 40-dust chain.
        let mut skate = DustPool::new();
        let mut srng = UnifiedRandom::new(61);
        generic_spawn_dust(
            57, 31, false, false, false, tl, 40, 10, vel, "Player", center,
            &mut srng, &mut skate, &c, &w,
        );
        let mut smpool = DustPool::new();
        let mut smirror = UnifiedRandom::new(61);
        for _ in 0..40 {
            let idx = smpool.new_dust(
                &mut smirror, &c, &w, tl, 40, 10, 31, 0.0, 0.0, 0,
                Color::TRANSPARENT, 1.0,
            );
            let md = smpool.slot_mut(idx);
            md.scale += smirror.next_range(-10, 21) as f32 * 0.01;
            md.velocity *= 0.8;
            md.velocity.y *= 0.75;
            md.velocity.y += 0.25;
            md.no_light_emittance = true;
            md.no_gravity = true;
            if smirror.next_max(2) == 0 {
                md.scale *= 1.3;
                md.no_gravity = true;
            } else {
                md.velocity *= 0.5;
            }
            md.velocity += vel * 0.8;
        }
        assert_eq!(active(&skate).len(), 40, "the skates' count is 40");
        for i in 0..40 {
            assert_eq!(skate.slot(i).velocity, smpool.slot(i).velocity);
            assert_eq!(skate.slot(i).scale, smpool.slot(i).scale);
        }
        assert_eq!(srng.next(), smirror.next());
    }

    /// W4-f6 WIRING — the request-channel drain: a Spawn request routed
    /// through `drain_mount_dust` lands byte-identical records + stream
    /// position as the direct emitter call (the world_sim handoff changes
    /// no draws). Wolf 52 rides the early-return arm.
    #[test]
    fn w4_f6_drain_spawn_matches_the_direct_wolf_emitter() {
        let w = world();
        let c = ctx();
        let pos = Vec2::new(400.0, 300.0);
        let vel = Vec2::new(2.0, -1.0);
        let req = MountDustRequest::Spawn {
            kind: 52,
            spawn_dust: 267,
            spawn_dust_no_gravity: false,
            is_cart: false,
            is_dismounting: false,
            pos,
            width: 20,
            height: 42,
            r_top_left: pos,
            r_w: 60,
            r_h: 42,
            vel,
            center: pos + Vec2::new(10.0, 21.0),
        };
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(97);
        let pl = crate::player::Player::spawn(&w);
        drain_mount_dust(vec![req], "Terrarian", &pl, &mut rng, &mut pool, &c, &w);
        let mut dpool = DustPool::new();
        let mut drng = UnifiedRandom::new(97);
        wolf_spawn_dust(pos, 20, 42, vel, &mut drng, &mut dpool, &c, &w);
        assert_eq!(active(&pool).len(), active(&dpool).len());
        for i in active(&pool) {
            assert_eq!(pool.slot(i).position, dpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, dpool.slot(i).velocity);
            assert_eq!(pool.slot(i).scale, dpool.slot(i).scale);
            assert_eq!(pool.slot(i).color, dpool.slot(i).color);
        }
        assert_eq!(rng.next(), drng.next(), "the drain changes no draws");
    }

    /// W4-f6 WIRING — the movement drain: a Basilisk request runs the run
    /// pair THEN the charge dust (the C# :4099/:4121 order), byte-identical
    /// to the direct emitter pair on the same stream.
    #[test]
    fn w4_f6_drain_basilisk_matches_the_direct_pair() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(500.0, 400.0);
        let bottom_left = Vec2::new(490.0, 442.0);
        let vel = Vec2::new(6.0, 0.0);
        let req = MountDustRequest::Basilisk {
            charge: 1.0,
            center,
            bottom_left,
            width: 20,
            facing: 1,
            vel,
            vx: 6.0,
            vy: 0.0,
            run_speed: 8.0,
        };
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(53);
        let pl = crate::player::Player::spawn(&w);
        drain_mount_dust(vec![req], "Terrarian", &pl, &mut rng, &mut pool, &c, &w);
        let mut dpool = DustPool::new();
        let mut drng = UnifiedRandom::new(53);
        basilisk_run_dust(bottom_left, 20, 6.0, 0.0, 8.0, &mut drng, &mut dpool, &c, &w);
        basilisk_charge_dust(
            1.0, center, bottom_left, 20, 1, vel, 8.0, &mut drng, &mut dpool, &c, &w,
        );
        assert_eq!(active(&pool).len(), active(&dpool).len());
        assert_eq!(active(&pool).len(), 5, "2 run + (int)(3*1) charge");
        for i in active(&pool) {
            assert_eq!(pool.slot(i).position, dpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, dpool.slot(i).velocity);
            assert_eq!(pool.slot(i).dust_type, dpool.slot(i).dust_type);
        }
        assert_eq!(rng.next(), drng.next(), "the drain changes no draws");
    }

    /// W4-f6 WIRING — the FailedDismount request: the drain routes to
    /// `failed_dismount_dust` unchanged (the Bat arm, count 40 = strike 2).
    #[test]
    fn w4_f6_drain_failed_dismount_matches_the_direct_emitter() {
        let w = world();
        let c = ctx();
        let tl = Vec2::new(390.0, 290.0);
        let vel = Vec2::new(1.5, 0.0);
        let center = Vec2::new(410.0, 321.0);
        let req = MountDustRequest::FailedDismount {
            kind: 56,
            spawn_dust: 31,
            spawn_dust_no_gravity: false,
            hitbox_top_left: tl,
            hitbox_w: 40,
            hitbox_h: 38,
            dust_amount: 40,
            vel,
            center,
        };
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(101);
        let pl = crate::player::Player::spawn(&w);
        drain_mount_dust(vec![req], "Terrarian", &pl, &mut rng, &mut pool, &c, &w);
        let mut dpool = DustPool::new();
        let mut drng = UnifiedRandom::new(101);
        failed_dismount_dust(
            56, 31, false, tl, 40, 38, 40, vel, "Terrarian", center,
            &mut drng, &mut dpool, &c, &w,
        );
        assert_eq!(active(&pool).len(), active(&dpool).len());
        for i in active(&pool) {
            assert_eq!(pool.slot(i).velocity, dpool.slot(i).velocity);
            assert_eq!(pool.slot(i).scale, dpool.slot(i).scale);
            // `:6341-6347` pins 0.7, then the no-gravity coin (`:6358-6362`)
            // may multiply it by 1.3 - the record holds either.
            let s = pool.slot(i).scale;
            assert!(
                (s - 0.7).abs() < 1e-6 || (s - 0.91).abs() < 1e-6,
                "the Bat's scale is 0.7 (or 0.7*1.3 after the coin): {s}"
            );
        }
        assert_eq!(rng.next(), drng.next(), "the drain changes no draws");
    }

    /// W4-f6 WIRING — the Pixie-tail effect request: the drain routes to
    /// `pixie_tail` unchanged (the even-tick gate passes at time 0).
    #[test]
    fn w4_f6_drain_pixie_tail_matches_the_direct_emitter() {
        let w = world();
        let c = ctx();
        let center = Vec2::new(500.0, 400.0);
        let vel = Vec2::new(8.0, 0.0);
        let req = MountDustRequest::PixieTail { center, vel, time_for_visuals: 0.0 };
        let pl = crate::player::Player::spawn(&w);
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(71);
        drain_mount_dust(vec![req], "Terrarian", &pl, &mut rng, &mut pool, &c, &w);
        let mut dpool = DustPool::new();
        let mut drng = UnifiedRandom::new(71);
        pixie_tail(center, vel, 0.0, "Terrarian", &mut drng, &mut dpool, &c, &w);
        assert_eq!(active(&pool).len(), active(&dpool).len());
        for i in active(&pool) {
            assert_eq!(pool.slot(i).position, dpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, dpool.slot(i).velocity);
            assert_eq!(pool.slot(i).color, dpool.slot(i).color);
            assert_eq!(pool.slot(i).dust_type, 278);
        }
        assert_eq!(rng.next(), drng.next(), "the drain changes no draws");
    }
}
