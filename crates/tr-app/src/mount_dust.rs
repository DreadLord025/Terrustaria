//! W4-f6 batch 1 — the Mount.cs MOVEMENT dust sites (10): the drill mount's
//! beam puffs (`UseDrill`, `Mount.cs:3113`/`:3170`) and the `UpdateFrame`
//! (`:3581`) per-type dash/gallop arms — the Unicorn/WoF-Goat ring + body
//! (`:3878`/`:3896`), the Pirate Ship ring + run puff (`:3939`/`:3970`), the
//! Spooky Wood ring + body (`:4000`/`:4023`) and the Basilisk run + charge
//! dusts (`:4103`/`:4123`).
//!
//! Contract (the S1-E-1 pool, d50's `DustPool`): every draw rides the
//! caller's `UpdatePlayers` stream in C# statement order — `Mount.UseDrill`
//! runs at `Player.cs:28290` and `Mount.UpdateFrame` at `:36200-36216`,
//! both inside `Player.Update` under the `Main.SwapRandom("UpdatePlayers")`
//! swap (`Main.cs:17754`). The explicit `Main.rand` argument draws run
//! FIRST (C# evaluates arguments before the call), then the `NewDust`
//! internals roll inside `DustPool::new_dust` (`Dust.cs:75-256`), then the
//! post-spawn writes (which read `Main.dust[6000]` when the pool refuses —
//! the sentinel slot takes the writes exactly like the C#).
//!
//! STAGED emitters (the e2 Skeletron/Brain precedent): the enclosing
//! machines are unported — `UseDrill`/`DrillMountData` is M2-MOUNT-4, and
//! the port's `mount.rs::update_frame` carries the generic state selector
//! but not the per-type extras (`:3637-4142`). Each fn below takes the
//! player/mount snapshot at the C# read moment; the wiring wave calls them
//! from the machines' C# positions on the `UpdatePlayers` stream. The
//! state-machine halves stay caller-side and are cited at each emitter:
//! the drill's beam cooldowns/target scan, the dash `_frameExtra`
//! increments and `_mountSpecificData` latches, the Basilisk charge
//! accumulator (ported here as the pure [`basilisk_charge_update`], the
//! `Player.basiliskCharge` field is the wiring wave's).
//!
//! Documented drops (each cited at its site): `dust.shader =
//! GameShaders.Armor.GetSecondaryShader(...)` (no draw-shader chain, the
//! `particles.rs` field drop), `dust.customData = mountedPlayer` (the
//! follower arms are d50's `customData` drop), the case-14 `Lighting.
//! AddLight` (`Mount.cs:4097`, the light lane's) and the `cMount == 0`
//! FloorVisuals hop (`:4110-4115` — the temporary ±24 px position shift
//! around `Player.FloorVisuals(Falling: true)`; the port's `floor_visuals`
//! is the caller's, the hop is a wiring note).
//!
//! `gravDir` is the vanilla `1f` (the port has no gravity flip, the
//! `sticky_movement` convention); none of these ten sites reads it.
//! WRITTEN NOT RUN (code-only mode): the witnesses below are compiled by
//! the merge-verify wave, not executed here.

use crate::bow_launch::{rotated_by, safe_normalize};
use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

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
    // `:3107` — `(vector - mountedPlayer.Center).ToRotation()` (atan2).
    let d = target - player_center;
    let num = d.y.atan2(d.x);
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

#[cfg(test)]
mod tests {
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
        let num = d.y.atan2(d.x);
        for _ in 0..2 {
            let side = if mirror.next_max(2) == 1 { -1.0 } else { 1.0 };
            let num2 = num + side * (std::f32::consts::PI / 2.0);
            let num3 = mirror.next_double() as f32 * 2.0 + 2.0;
            let v = Vec2::new(
                (num2 as f64).cos() as f32 * num3,
                (num2 as f64).sin() as f32 * num3,
            );
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
            // The speed is NextDouble()*2+2 in [2, 4).
            let len = pool.slot(i).velocity.length();
            assert!(len >= 2.0 && len < 4.01, "speed {len}");
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
}
