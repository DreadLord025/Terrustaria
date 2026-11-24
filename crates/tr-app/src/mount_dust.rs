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

/// Site 4626 — `EmitGolfCartSmoke` (`Mount.cs:4623-4639`). `gravDir` is
/// the vanilla 1f (the port has no gravity flip — d51's cited
/// convention), so `Directions = (direction, 1)`. Position = Bottom +
/// (−direction×34, −12); NewDustDirect 0×0 type 31, speedX = −direction,
/// speedY = −0.24, alpha 100; then position pinned, velocity ×0.1 +=
/// (−direction, −0.25), scale 0.5; vx != 0 adds Sign(vx)×1.3 X; rushing
/// sets fadeIn 0.8.
pub(crate) fn golf_cart_smoke(
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
pub(crate) fn witch_broom_trinket_rotation(vx: f32, misc_counter: u32, wind: f32, in_wind: bool) -> f32 {
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
fn direction_to(origin: Vec2, target: Vec2) -> Vec2 {
    (target - origin).normalize_or_zero()
}

/// Sites 4824/4849 — `UpdateEffects` case 23's witch-broom trinket
/// (`Mount.cs:4815-4856`). `origin` = RotatedRelativePoint(Center +
/// (29×direction, −4) + (rotation + π/2).ToRotationVector2()×11) — the
/// caller computes it from [`witch_broom_trinket_rotation`] (the
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
            d.velocity = direction_to(origin, d.position) * 0.2; // `:4825-4828`
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
            d.velocity = ring * 0.025 + direction_to(pos5, d.position) * 0.5; // `:4850`
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
pub(crate) fn drill_mount_trail(
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
            unicorn_dash_body(position, width, height, rng, dust, ctx, world);
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
                wof_goat_mount_flames(&s, rng, dust, ctx, world);
            }
        }
        WitchBroom { wind_speed, in_wind } => {
            // `Mount.cs:4817-4819` — the origin: Center + (29×dir, −4) +
            // (rotation + π/2).ToRotationVector2()×11, then
            // RotatedRelativePoint. The rotation is the RNG-free sway helper.
            let rot = witch_broom_trinket_rotation(p.vx, p.misc_counter, wind_speed, in_wind);
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
            drill_mount_trail(centre, current, old, rng, dust, ctx, world);
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
        golf_cart_smoke(bottom, 1, 5.0, true, &mut rng, &mut pool, &c, &w);
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
        drill_mount_trail(centre, current, old, &mut rng, &mut pool, &c, &w);
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
        drill_mount_trail(centre, Vec2::new(902.0, 900.0), old, &mut rng, &mut pool, &c, &w);
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
