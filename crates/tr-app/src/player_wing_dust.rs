//! W4-g66 — the wing-trail dust drains: `WingAirVisuals`
//! (`Player.cs:30750-31013`, the flag19 flying tick). Same queue convention
//! as `player_dust` (W4-d51): the tick snapshots the box/velocity at the C#
//! read moment (BEFORE `WingMovement` shapes it); this module replays each
//! `Dust.NewDust` on the shared `UpdatePlayers` stream in exact C# draw
//! order.
//!
//! The port folds `wings` (the look id) into `wings_logic` (the wing grant's
//! note), so `wingsLogic != wings` is always false — every
//! `noLightEmittance = flag` write lands `false` (the default the pool
//! already writes; the field is set explicitly to keep the mapping). The
//! `shader` writes are dropped fields (`particles.rs`'s note). `gravDir` is
//! the vanilla `1f`, so the `gravDir == -1` height folds collapse. The C#
//! `base.height / 2` / `base.width / 2` are INTEGER divisions — the port
//! truncates the f32 hitbox first.

use crate::app::App;
use crate::player::{WingAirDust, WingFrameDust, WingGlideDust};
use crate::player_dust::{lerp_color, nd};
use glam::Vec2;
use tr_core::particles::Color;

/// The back-offset the mid-body arms share (`:30755-30759` et al.): 4 facing
/// left, −40 facing right, from the body centre line.
fn back_off(facing: i32) -> f32 {
    if facing == 1 { -40.0 } else { 4.0 }
}

/// The lower-trail offset (`:30832-30836` et al.): 6 / −30.
fn low_off(facing: i32) -> f32 {
    if facing == 1 { -30.0 } else { 6.0 }
}

/// The second-puff offset (`:30857-30861` et al.): −24 / +12.
fn second_off(facing: i32) -> f32 {
    if facing == 1 { 12.0 } else { -24.0 }
}

/// One `Main.rand.Next(n) == 0` gate on the `UpdatePlayers` stream.
fn roll(app: &mut App, n: i32) -> bool {
    app.rngs
        .stream(crate::app::RngKey::UpdatePlayers)
        .next_max(n)
        == 0
}

/// Drain `WingAirVisuals` (`Player.cs:30750-31013`). The arms are sequential
/// ifs in source order — a wing id matches at most one, and the ROLLS of the
/// non-matching arms never happen (each gate short-circuits on the id).
pub(crate) fn drain_wing_air(app: &mut App, b: WingAirDust) {
    let cx = b.x + (b.width as i32 / 2) as f32;
    let mid_y = b.y + (b.height as i32 / 2) as f32 - 15.0;
    match b.wings {
        // `:30753-30766`: wings 10 — 1/2 type-76, fadeIn 1.1, noGravity,
        // noLight, velocity ×0.3.
        10 => {
            if roll(app, 2) {
                let idx = nd(app, Vec2::new(cx + back_off(b.facing), mid_y), 30, 30, 76, 0.0, 0.0, 50, Color::default(), 0.6);
                let d = app.dust.slot_mut(idx);
                d.fade_in = 1.1;
                d.no_gravity = true;
                d.no_light = true;
                d.velocity *= 0.3;
            }
        }
        // `:30767-30781`: wings 34 — the same shape on type 261.
        34 => {
            if roll(app, 2) {
                let idx = nd(app, Vec2::new(cx + back_off(b.facing), mid_y), 30, 30, 261, 0.0, 0.0, 50, Color::default(), 0.6);
                let d = app.dust.slot_mut(idx);
                d.fade_in = 1.1;
                d.no_gravity = true;
                d.no_light = true;
                d.no_light_emittance = false;
                d.velocity *= 0.3;
            }
        }
        // `:30782-30796`: wings 51 — type 261 tinted (230,130,55).
        51 => {
            if roll(app, 2) {
                let idx = nd(app, Vec2::new(cx + back_off(b.facing), mid_y), 30, 30, 261, 0.0, 0.0, 50, Color::new(230, 130, 55, 255), 0.6);
                let d = app.dust.slot_mut(idx);
                d.fade_in = 1.1;
                d.no_gravity = true;
                d.no_light = true;
                d.no_light_emittance = false;
                d.velocity *= 0.3;
            }
        }
        // `:30805-30817`: wings 9 — 1/2 type-6, alpha 200, scale 2.
        9 => {
            if roll(app, 2) {
                let idx = nd(app, Vec2::new(cx + back_off(b.facing), mid_y), 30, 30, 6, 0.0, 0.0, 200, Color::default(), 2.0);
                let d = app.dust.slot_mut(idx);
                d.no_gravity = true;
                d.no_light_emittance = false;
                d.velocity *= 0.3;
            }
        }
        // `:30818-30829`: wings 6 — 1/4 type-55, alpha 200, default scale.
        6 => {
            if roll(app, 4) {
                let idx = nd(app, Vec2::new(cx + back_off(b.facing), mid_y), 30, 30, 55, 0.0, 0.0, 200, Color::default(), 1.0);
                let d = app.dust.slot_mut(idx);
                d.velocity *= 0.3;
                d.no_light_emittance = false;
            }
        }
        // `:30830-30841`: wings 5 — 1/3 type-58 at the lower box.
        5 => {
            if roll(app, 3) {
                let idx = nd(app, Vec2::new(cx + low_off(b.facing), b.y), 18, b.height as i32, 58, 0.0, 0.0, 255, Color::default(), 1.2);
                let d = app.dust.slot_mut(idx);
                d.no_light_emittance = false;
                d.velocity *= 0.3;
            }
        }
        // `:30842-30874`: wings 26 — the unconditional type-217 pair.
        26 => beetle_trail(app, &b),
        // `:30875-30919`: wings 37 — the Direct type-31 pair.
        37 => festive_trail(app, &b),
        // `:30920-30936`: wings 29 — 1/3 type-6 scale 2.4 + the 1/10 fadeIn.
        29 => {
            if roll(app, 3) {
                let idx = nd(app, Vec2::new(cx + back_off(b.facing), mid_y), 30, 30, 6, 0.0, 0.0, 100, Color::default(), 2.4);
                let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                let d = app.dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 0.3;
                d.no_light_emittance = false;
                if rng.next_max(10) == 0 {
                    d.fade_in = 2.0;
                }
            }
        }
        // `:30937-31012`: wings 31 — the four-roll nebula chain.
        31 => nebula_trail(app, &b),
        _ => {}
    }
}

/// `:30842-30874` — the Beetle (26) trail: one type-217 puff at the lower
/// box, then a 1/2 second at the half-height box. Post: `noGravity`,
/// `noLight`, `velocity /= 4`, `velocity -= playerVel`.
fn beetle_trail(app: &mut App, b: &WingAirDust) {
    let cx = b.x + (b.width as i32 / 2) as f32;
    let idx = nd(app, Vec2::new(cx + low_off(b.facing), b.y), 18, b.height as i32, 217, 0.0, 0.0, 100, Color::default(), 1.4);
    {
        let d = app.dust.slot_mut(idx);
        d.no_gravity = true;
        d.no_light = true;
        d.velocity /= 4.0;
        d.velocity -= Vec2::new(b.vx, b.vy);
    }
    if roll(app, 2) {
        let idx = nd(app, Vec2::new(cx + second_off(b.facing), b.y), 12, b.height as i32 / 2, 217, 0.0, 0.0, 100, Color::default(), 1.4);
        let d = app.dust.slot_mut(idx);
        d.no_gravity = true;
        d.no_light = true;
        d.velocity /= 4.0;
        d.velocity -= Vec2::new(b.vx, b.vy);
    }
}

/// One `Dust.NewDustDirect` on the `UpdatePlayers` stream.
fn ndd(
    app: &mut App,
    pos: Vec2,
    w: i32,
    h: i32,
    ty: i32,
    alpha: i32,
    color: Color,
    scale: f32,
) -> usize {
    let ctx = app.spawn_ctx();
    let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
    app.dust
        .new_dust_direct(rng, &ctx, &app.world, pos, w, h, ty, 0.0, 0.0, alpha, color, scale)
}

/// `:30875-30919` — the Festive (37) trail: one `NewDustDirect` of
/// `SelectRandom(31,31,31)` at the lower box (24 wide), then a 1/3 second
/// (16 wide, half height). Post: `noGravity`, `noLight`, `velocity /= 4`,
/// `velocity -= playerVel/2`. The `type == 55` re-tint arms are DEAD — the
/// roll list is all 31 (the SelectRandom draws still ride the stream).
fn festive_trail(app: &mut App, b: &WingAirDust) {
    let cx = b.x + (b.width as i32 / 2) as f32;
    let ty = {
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        [31, 31, 31][rng.next_max(3) as usize]
    };
    let idx = ndd(app, Vec2::new(cx + low_off(b.facing), b.y), 24, b.height as i32, ty, 100, Color::default(), 1.0);
    {
        let d = app.dust.slot_mut(idx);
        d.no_gravity = true;
        d.no_light = true;
        d.velocity /= 4.0;
        d.velocity -= Vec2::new(b.vx, b.vy) / 2.0;
        // The dead `type == 55` arm (`:30888-30893`): never fires.
    }
    if roll(app, 3) {
        let ty = {
            let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
            [31, 31, 31][rng.next_max(3) as usize]
        };
        let idx = ndd(app, Vec2::new(cx + second_off(b.facing), b.y), 16, b.height as i32 / 2, ty, 100, Color::default(), 1.0);
        let d = app.dust.slot_mut(idx);
        d.no_gravity = true;
        d.no_light = true;
        d.velocity /= 4.0;
        d.velocity -= Vec2::new(b.vx, b.vy) / 2.0;
    }
}

/// `:30937-31012` — the Nebula (31) chain: 1/6 type-86 + 1/3 type-240 at the
/// mid box, then (1/2 gate) 1/6 type-86 + 1/3 type-240 at the half box.
fn nebula_trail(app: &mut App, b: &WingAirDust) {
    let cx = b.x + (b.width as i32 / 2) as f32;
    let mid_y = b.y + (b.height as i32 / 2) as f32 - 15.0;
    if roll(app, 6) {
        let idx = nd(app, Vec2::new(cx + back_off(b.facing), mid_y), 30, 30, 86, 0.0, 0.0, 0, Color::default(), 1.0);
        let d = app.dust.slot_mut(idx);
        d.no_gravity = true;
        d.scale = 1.0;
        d.fade_in = 1.2;
        d.velocity *= 0.2;
        d.no_light = true;
    }
    if roll(app, 3) {
        let idx = nd(app, Vec2::new(cx + back_off(b.facing), mid_y), 30, 30, 240, 0.0, 0.0, 0, Color::default(), 1.0);
        let d = app.dust.slot_mut(idx);
        d.no_gravity = true;
        d.scale = 1.2;
        d.velocity *= 0.2;
        d.alpha = 200;
    }
    if !roll(app, 2) {
        return;
    }
    if roll(app, 6) {
        let idx = nd(app, Vec2::new(cx + second_off(b.facing), b.y), 12, b.height as i32 / 2, 86, 0.0, 0.0, 0, Color::default(), 1.0);
        let d = app.dust.slot_mut(idx);
        d.no_gravity = true;
        d.scale = 1.0;
        d.fade_in = 1.2;
        d.velocity *= 0.2;
        d.no_light = true;
    }
    if roll(app, 3) {
        let idx = nd(app, Vec2::new(cx + second_off(b.facing), b.y), 12, b.height as i32 / 2, 240, 0.0, 0.0, 0, Color::default(), 1.0);
        let d = app.dust.slot_mut(idx);
        d.no_gravity = true;
        d.scale = 1.2;
        d.velocity *= 0.2;
        d.alpha = 200;
    }
}

/// Drain the wing-glide dust (`Player.cs:27040-27315`). The tick ran the
/// frame machines (`wing_glide_frames`); this replays the rolls in source
/// order. The wings-4 Item13 cadence (`:27151-27156`) is an audio seam.
pub(crate) fn drain_wing_glide(app: &mut App, b: WingGlideDust) {
    let cx = b.x + (b.width as i32 / 2) as f32;
    let mid_y = b.y + (b.height as i32 / 2) as f32 - 15.0;
    // `:27040-27053`: wings 10 — 1/3 type-76 (the WingAirVisuals shape).
    if b.wings == 10 && roll(app, 3) {
        let idx = nd(app, Vec2::new(cx + back_off(b.facing), mid_y), 30, 30, 76, 0.0, 0.0, 50, Color::default(), 0.6);
        let d = app.dust.slot_mut(idx);
        d.fade_in = 1.1;
        d.no_gravity = true;
        d.no_light = true;
        d.velocity *= 0.3;
    }
    // `:27054-27068`: wings 34 — the always-animated gate BEFORE the roll.
    if b.wings == 34 && b.always_animated && roll(app, 3) {
        let idx = nd(app, Vec2::new(cx + back_off(b.facing), mid_y), 30, 30, 261, 0.0, 0.0, 50, Color::default(), 0.6);
        let d = app.dust.slot_mut(idx);
        d.fade_in = 1.1;
        d.no_gravity = true;
        d.no_light = true;
        d.no_light_emittance = false;
        d.velocity *= 0.3;
    }
    // `:27069-27083`: wings 51 — 1/5, the tinted 261.
    if b.wings == 51 && roll(app, 5) {
        let idx = nd(app, Vec2::new(cx + back_off(b.facing), mid_y), 30, 30, 261, 0.0, 0.0, 50, Color::new(230, 130, 55, 255), 0.6);
        let d = app.dust.slot_mut(idx);
        d.fade_in = 1.1;
        d.no_gravity = true;
        d.no_light = true;
        d.no_light_emittance = false;
        d.velocity *= 0.3;
    }
    // `:27092-27104`: wings 9 — 1/3 type-6, offset 8/−40 (NOT 4).
    if b.wings == 9 && roll(app, 3) {
        let off = if b.facing == 1 { -40.0 } else { 8.0 };
        let idx = nd(app, Vec2::new(cx + off, mid_y), 30, 30, 6, 0.0, 0.0, 200, Color::default(), 2.0);
        let d = app.dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= 0.3;
        d.no_light_emittance = false;
    }
    // `:27105-27121`: wings 29 — 1/3 type-6 scale 2.4, offset 8/−40.
    if b.wings == 29 && roll(app, 3) {
        let off = if b.facing == 1 { -40.0 } else { 8.0 };
        let idx = nd(app, Vec2::new(cx + off, mid_y), 30, 30, 6, 0.0, 0.0, 100, Color::default(), 2.4);
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let d = app.dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= 0.3;
        d.no_light_emittance = false;
        if rng.next_max(10) == 0 {
            d.fade_in = 2.0;
        }
    }
    // `:27122-27148`: wings 6 → 1/10 type-55 at mid−12; ELSE wings 5 → 1/6
    // type-58 (the else-if chain — 6 never reaches the 5 arm).
    if b.wings == 6 {
        if roll(app, 10) {
            let idx = nd(app, Vec2::new(cx + back_off(b.facing), b.y + (b.height as i32 / 2) as f32 - 12.0), 30, 20, 55, 0.0, 0.0, 200, Color::default(), 1.0);
            let d = app.dust.slot_mut(idx);
            d.no_light_emittance = false;
            d.velocity *= 0.3;
        }
    } else if b.wings == 5 && roll(app, 6) {
        let idx = nd(app, Vec2::new(cx + low_off(b.facing), b.y), 18, b.height as i32, 58, 0.0, 0.0, 255, Color::default(), 1.2);
        let d = app.dust.slot_mut(idx);
        d.velocity *= 0.3;
        d.no_light_emittance = false;
    }
    // `:27149-27191`: wings 4 — the jetpack glide flame. The 1/2 side roll
    // picks the inner box (+8/−20, y+6); post: `velocity.X *= 0.3`,
    // `velocity.Y += 10`, noGravity.
    if b.wings == 4 {
        let mut x = cx + 16.0;
        if b.facing > 0 {
            x = cx - 26.0;
        }
        let mut y = b.y + b.height - 18.0;
        let side = {
            let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
            rng.next_max(2)
        };
        if side == 1 {
            x = cx + 8.0;
            if b.facing > 0 {
                x = cx - 20.0;
            }
            y += 6.0;
        }
        let idx = nd(app, Vec2::new(x, y), 8, 8, 6, 0.0, 0.0, 100, Color::default(), 1.5);
        let d = app.dust.slot_mut(idx);
        d.velocity.x *= 0.3;
        d.velocity.y += 10.0;
        d.no_gravity = true;
        d.no_light_emittance = false;
    } else if b.wings == 26 {
        // `:27240-27273`: the Beetle glide pair (the WingAirVisuals shape).
        beetle_trail(app, &glide_as_air(&b));
    } else if b.wings == 37 {
        // `:27274-27316`: the Festive glide pair — the colour roll runs
        // FIRST (`Lerp(Black, White, NextFloat)`), scale 0.7, second alpha
        // 140; the `type == 55` re-tint is dead (the list is all 31).
        festive_glide(app, &b);
    }
}

/// The glide snapshot re-viewed as a `WingAirDust` (the shared 26 pair).
fn glide_as_air(b: &WingGlideDust) -> WingAirDust {
    WingAirDust {
        wings: b.wings,
        x: b.x,
        y: b.y,
        width: b.width,
        height: b.height,
        vx: b.vx,
        vy: b.vy,
        facing: b.facing,
    }
}

/// `:27274-27316` — the Festive glide pair with the shared HSL colour roll.
/// The colour applies only on the dead `type == 55` arms, but the
/// `NextFloat` draw rides the stream FIRST regardless.
fn festive_glide(app: &mut App, b: &WingGlideDust) {
    let cx = b.x + (b.width as i32 / 2) as f32;
    let t = app
        .rngs
        .stream(crate::app::RngKey::UpdatePlayers)
        .next_float();
    let _color = lerp_color(Color::new(0, 0, 0, 255), Color::WHITE, t);
    let ty = {
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        [31, 31, 31][rng.next_max(3) as usize]
    };
    let idx = nd(app, Vec2::new(cx + low_off(b.facing), b.y), 24, b.height as i32, ty, 0.0, 0.0, 100, Color::default(), 0.7);
    {
        let d = app.dust.slot_mut(idx);
        d.no_gravity = true;
        d.no_light = true;
        d.velocity /= 4.0;
        d.velocity -= Vec2::new(b.vx, b.vy);
        // The dead `type == 55` re-tint (`:27288-27291`).
    }
    if roll(app, 3) {
        let ty = {
            let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
            [31, 31, 31][rng.next_max(3) as usize]
        };
        let idx = nd(app, Vec2::new(cx + second_off(b.facing), b.y), 12, b.height as i32 / 2, ty, 0.0, 0.0, 140, Color::default(), 0.7);
        let d = app.dust.slot_mut(idx);
        d.no_gravity = true;
        d.no_light = true;
        d.velocity /= 4.0;
        d.velocity -= Vec2::new(b.vx, b.vy);
    }
}

/// Drain the `WingFrame` dust arms (`Player.cs:29468-30527`). The tick ran
/// the frame machines (`wing_frame_visuals`); this replays the rolls.
pub(crate) fn drain_wing_frame(app: &mut App, b: WingFrameDust) {
    let cx = b.x + (b.width as i32 / 2) as f32;
    match b.wings {
        // `:29481-29525`: the jetpack flap flames — 2 puffs (4 while
        // hover-up), the i=1/3 inner box (+8/−20, y+6), i>1 adds vy.
        // Post: X ×0.1, Y = Y + 2 − vy*0.3, noGravity; the 4-count adds
        // Y += 6. The type-6 tweak chain rides each spawn.
        4 => {
            let count = if b.hover_up { 4 } else { 2 };
            for i in 0..count {
                let mut x = cx + 16.0;
                if b.facing > 0 {
                    x = cx - 26.0;
                }
                let mut y = b.y + b.height - 18.0;
                if i == 1 || i == 3 {
                    x = cx + 8.0;
                    if b.facing > 0 {
                        x = cx - 20.0;
                    }
                    y += 6.0;
                }
                if i > 1 {
                    y += b.vy;
                }
                let idx = nd(app, Vec2::new(x, y), 8, 8, 6, 0.0, 0.0, 100, Color::default(), 1.75);
                let d = app.dust.slot_mut(idx);
                d.velocity.x *= 0.1;
                d.velocity.y = d.velocity.y + 2.0 - b.vy * 0.3;
                d.no_gravity = true;
                d.no_light_emittance = false;
                if count == 4 {
                    d.velocity.y += 6.0;
                }
            }
        }
        // `:29816-29858`: the Bat sparkles — two 4-iteration loops at the
        // centre, 1/4 then 1/8 gates; the rotation-vector offsets, the
        // position/velocity overwrites, the 1/10 fadeIn 0.5 (the else arm
        // sets customData — a dropped field, the draw still happens).
        30 => bat_sparkles(app, &b),
        // `:30195-30203`: the Rainbow glow — 1/8, the centred 40×24 box at
        // Bottom, type 43 White×0.5 scale 0.2, fadeIn 0.4, velocity +=
        // playerVel then ×0.35.
        45 => {
            if roll(app, 8) {
                let bx = b.x + (b.width as i32 / 2) as f32;
                let by = b.y + b.height;
                // `Utils.CenteredRectangle`: (int)(center - size/2).
                let rx = (bx - 20.0) as i32;
                let ry = (by - 12.0) as i32;
                let idx = ndd(app, Vec2::new(rx as f32, ry as f32), 40, 24, 43, 0, Color::new(127, 127, 127, 127), 0.2);
                let d = app.dust.slot_mut(idx);
                d.fade_in = 0.4;
                d.velocity += Vec2::new(b.vx, b.vy);
                d.velocity *= 0.35;
            }
        }
        // `:30289-30306`: the Spooky shadow puffs — count 0/1/5 at the
        // centred 50×20 box 10 above Bottom, type 31 Black, scale 0.7,
        // velocity ×0.4 then Y += 0.5.
        39 => {
            let bx = b.x + (b.width as i32 / 2) as f32;
            let by = b.y + b.height - 10.0;
            let rx = (bx - 25.0) as i32;
            let ry = (by - 10.0) as i32;
            for _ in 0..b.count {
                let idx = ndd(app, Vec2::new(rx as f32, ry as f32), 50, 20, 31, 0, Color::new(0, 0, 0, 255), 1.0);
                let d = app.dust.slot_mut(idx);
                d.scale = 0.7;
                d.velocity *= 0.4;
                d.velocity.y += 0.5;
            }
        }
        // W4-g66 gap-fill — `:30309-30360`: the Fairy-33 eye-colour pair.
        33 => fairy_eye_pair(app, &b),
        // W4-g66 gap-fill — `:30408-30445`: the Leaf-38 orbit triples.
        38 => leaf_orbits(app, &b),
        // W4-g66 gap-fill — `:30529-30607`: the Chicken Bones-47 bursts.
        47 => chicken_bones(app, &b),
        // W4-g66 gap-fill — `:30609-30691`: the Heroicis-49 light-gated rolls.
        49 => heroicis(app, &b),
        // W4-g66 gap-fill — `:30693-30735`: the Kazzymodus-50 sparkle rolls.
        50 => kazzymodus(app, &b),
        // `:30505-30521`: the Nebula-29 rest roll — 1/5 type-6 scale 2.4 at
        // the mid box, the 1/10 fadeIn 2.
        29 => {
            if roll(app, 5) {
                let mid_y = b.y + (b.height as i32 / 2) as f32 - 15.0;
                let idx = nd(app, Vec2::new(cx + back_off(b.facing), mid_y), 30, 30, 6, 0.0, 0.0, 100, Color::default(), 2.4);
                let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                let d = app.dust.slot_mut(idx);
                d.no_gravity = true;
                d.no_light_emittance = false;
                d.velocity *= 0.3;
                if rng.next_max(10) == 0 {
                    d.fade_in = 2.0;
                }
            }
        }
        _ => {}
    }
}

/// `:29816-29858` — the Bat-30 sparkle loops. Loop 1: 4 iterations, 1/4
/// gate, the `(-0.74539816 + PI/8*j + 0.03*j)` rotation scaled by
/// `(-direction*20, 20)`; type 229 White scale 0.8, position = centre +
/// offset, velocity = DirectionTo(position)*2. Loop 2: 1/8 gate, the
/// `(-0.7053982 + PI/8*k + 0.03*k)` rotation scaled by `(direction*20, 24)`
/// plus `(-direction*16, 0)`; scale 0.5, velocity = Normalize(position −
/// centre − (-direction*16, 0))*2, position += velocity*5. Both: the 1/10
/// fadeIn-0.5-vs-customData draw (customData is dropped).
fn bat_sparkles(app: &mut App, b: &WingFrameDust) {
    let center = Vec2::new(b.x + b.width * 0.5, b.y + b.height * 0.5);
    let dir = b.facing as f32;
    for j in 0..4 {
        if !roll(app, 4) {
            continue;
        }
        let ang = -0.74539816f32 + std::f32::consts::PI / 8.0 * j as f32 + 0.03 * j as f32;
        let off = Vec2::new(ang.cos(), ang.sin()) * Vec2::new(-dir * 20.0, 20.0);
        let idx = nd(app, center, 0, 0, 229, 0.0, 0.0, 100, Color::WHITE, 0.8);
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let d = app.dust.slot_mut(idx);
        d.no_gravity = true;
        d.no_light_emittance = false;
        d.position = center + off;
        let delta = d.position - center;
        d.velocity = if delta == Vec2::ZERO { Vec2::ZERO } else { delta.normalize() * 2.0 };
        if rng.next_max(10) == 0 {
            d.fade_in = 0.5;
        }
    }
    for k in 0..4 {
        if !roll(app, 8) {
            continue;
        }
        let ang = -0.7053982f32 + std::f32::consts::PI / 8.0 * k as f32 + 0.03 * k as f32;
        let shift = Vec2::new(-dir * 16.0, 0.0);
        let off = Vec2::new(ang.cos(), ang.sin()) * Vec2::new(dir * 20.0, 24.0) + shift;
        let idx = nd(app, center, 0, 0, 229, 0.0, 0.0, 100, Color::WHITE, 0.5);
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let d = app.dust.slot_mut(idx);
        d.no_gravity = true;
        d.no_light_emittance = false;
        d.position = center + off;
        let delta = d.position - center - shift;
        d.velocity = if delta == Vec2::ZERO { Vec2::ZERO } else { delta.normalize() * 2.0 };
        d.position += d.velocity * 5.0;
        if rng.next_max(10) == 0 {
            d.fade_in = 0.5;
        }
    }
}

/// W4-g66 gap-fill — `:30324-30358`: the Fairy-33 eye-colour pair. The
/// colour is `hslToRgb(rgbToHsl(eyeColor).X, 1, 0.5)` (the tested
/// `ui::color` round-trip); TWO type-182 puffs over the body box (speed =
/// player velocity, alpha 127), each pinned to a foot corner (facing-left
/// shifts X −4), velocity ±2 X / +2 Y minus 0.3× the player velocity, then
/// a `CloneDust` at ×0.65 scale/fadeIn re-tinted White. The clone's
/// `dustIndex != 6000` guard is the pool's own sentinel early-return.
fn fairy_eye_pair(app: &mut App, b: &WingFrameDust) {
    let (h, _, _) = crate::ui::color::rgb_to_hsl(b.eye_color);
    let c = crate::ui::color::hsl_to_rgb(h, 1.0, 0.5, 255);
    let new_color = Color::new(c[0], c[1], c[2], c[3]);
    let x_off = if b.facing != 1 { -4.0 } else { 0.0 };
    for m in 0..2 {
        let idx = nd(
            app,
            Vec2::new(b.x, b.y),
            b.width as i32,
            b.height as i32,
            182,
            b.vx,
            b.vy,
            127,
            new_color,
            1.0,
        );
        {
            let d = app.dust.slot_mut(idx);
            d.no_gravity = true;
            d.fade_in = 1.0;
            d.scale = 1.0;
            d.no_light = true;
            d.no_light_emittance = false;
            let corner_x = if m == 0 { b.x + x_off } else { b.x + b.width + x_off };
            d.position = Vec2::new(corner_x, b.y + b.height);
            // `:30340`/`:30345`: X = X*1 ∓ 2 − vx*0.3 (m0 −2, m1 +2).
            let sign = if m == 0 { -2.0 } else { 2.0 };
            d.velocity.x = d.velocity.x + sign - b.vx * 0.3;
            d.velocity.y = d.velocity.y + 2.0 - b.vy * 0.3;
        }
        // `:30349-30357`: the clone (sentinel-guarded inside the pool).
        let ctx = app.spawn_ctx();
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let clone = app.dust.clone_dust(rng, &ctx, &app.world, idx);
        if clone != idx {
            let d = app.dust.slot_mut(clone);
            d.scale *= 0.65;
            d.fade_in *= 0.65;
            d.color = Color::new(255, 255, 255, 255);
        }
    }
}

/// W4-g66 gap-fill — `:30408-30445`: the Leaf-38 orbit triples. THREE
/// iterations × THREE `NewDustPerfect` type-267 puffs tinted
/// `underShirtColor`: the orbit anchors (−39,6)/(−23,2)/(−31,−6) scaled by
/// `(direction, gravDir)` plus a 2-px rotation vector at
/// `miscCounter*direction / −15` (the third: /−20) turns, velocity =
/// 0.5× the player velocity plus the iteration's −0.33/−0.66 fold.
/// Scales 0.47/0.35/0.49; noGravity + noLight.
fn leaf_orbits(app: &mut App, b: &WingFrameDust) {
    let center = Vec2::new(b.x + b.width * 0.5, b.y + b.height * 0.5);
    let dirs = Vec2::new(b.facing as f32, 1.0);
    let value = Vec2::new(b.vx, b.vy) * 0.5;
    let c = b.under_shirt_color;
    let tint = Color::new(c[0], c[1], c[2], c[3]);
    let num34 = b.misc_counter as i32 * b.facing;
    let tau = std::f32::consts::PI * 2.0;
    for n in 0..3 {
        let fold = match n {
            1 => Vec2::new(b.vx, b.vy) * -0.33,
            2 => Vec2::new(b.vx, b.vy) * -0.66,
            _ => Vec2::ZERO,
        };
        for (anchor, denom, scale) in [
            (Vec2::new(-39.0, 6.0), -15.0f32, 0.47f32),
            (Vec2::new(-23.0, 2.0), -15.0, 0.35),
            (Vec2::new(-31.0, -6.0), -20.0, 0.49),
        ] {
            let ang = num34 as f32 / denom * tau;
            let off = anchor * dirs + Vec2::new(2.0, 0.0).rotate(Vec2::from_angle(ang));
            let ctx = app.spawn_ctx();
            let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
            let idx = app.dust.new_dust_perfect(
                rng, &ctx, &app.world, center + off + fold, 267, Some(value), 0, tint, 1.0,
            );
            let d = app.dust.slot_mut(idx);
            d.no_gravity = true;
            d.no_light = true;
            d.no_light_emittance = false;
            d.scale = scale;
        }
    }
}

/// W4-g66 gap-fill — the frame-row bob table the Chicken Bones/Heroicis
/// share (`:30535-30568`/`:30615-30648`): rows 3/6 +0.5, 4/5 +1, 8 −0.5,
/// 9 −1, 10 −0.5; the rest 0.
fn dev_row_bob(frame: i32) -> f32 {
    match frame {
        3 | 6 => 0.5,
        4 | 5 => 1.0,
        8 | 10 => -0.5,
        9 => -1.0,
        _ => 0.0,
    }
}

/// W4-g66 gap-fill — `:30529-30607`: the Chicken Bones-47 dust. The
/// `justSwitched && frame == 6 && controlJump` burst: 8 type-264 puffs
/// (10×10 box, tint (50,120,255,200), scale 1.1×(1+0.25×NextFloatDirection))
/// at the centre ± the (−25/−5)×direction, 20-down offsets; the spawn's
/// X-delta doubles into Y, velocity = (x*0.5, 3). Then the 1/6 rest roll
/// (30×50 box at centre +(−30×direction, 0)): velocity ×0.1, Y/pos
/// += bob*2, velocity += playerVel*0.3*NextFloat. `customData` dropped.
fn chicken_bones(app: &mut App, b: &WingFrameDust) {
    if b.wing_frame == 0 {
        return;
    }
    let bob = dev_row_bob(b.wing_frame);
    let center = Vec2::new(b.x + b.width * 0.5, b.y + b.height * 0.5);
    let dir = b.facing as f32;
    let tint = Color::new(50, 120, 255, 200);
    if b.just_switched && b.wing_frame == 6 && b.control_jump {
        for i in 0..8 {
            let off = if i < 4 {
                Vec2::new(-25.0 * dir, 20.0)
            } else {
                Vec2::new(-5.0 * dir, 20.0)
            };
            let scale = {
                let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                1.1 * (1.0 + 0.25 * rng.next_float_direction())
            };
            let idx = ndd(app, center - Vec2::new(5.0, 5.0) + off, 10, 10, 264, 0, tint, scale);
            let d = app.dust.slot_mut(idx);
            let x = (d.position - center - off).x;
            d.position.y += x * 2.0;
            d.no_gravity = true;
            d.no_light_emittance = false;
            d.no_light = true;
            d.fade_in = 0.0;
            d.velocity = Vec2::new(x * 0.5, 3.0);
        }
    }
    if roll(app, 6) {
        let off = Vec2::new(-30.0 * dir, 0.0);
        let scale = {
            let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
            1.1 * (1.0 + 0.25 * rng.next_float_direction())
        };
        let idx = ndd(app, center - Vec2::new(15.0, 25.0) + off, 30, 50, 264, 0, tint, scale);
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let d = app.dust.slot_mut(idx);
        d.no_gravity = true;
        d.no_light_emittance = false;
        d.no_light = true;
        d.velocity *= 0.1;
        d.fade_in = 0.0;
        d.velocity.y += bob * 2.0;
        d.position.y += bob * 2.0;
        d.velocity += Vec2::new(b.vx, b.vy) * 0.3 * rng.next_float();
    }
}

/// W4-g66 gap-fill — `Lighting.GetColor(Point)` (`Lighting.cs:180-187`) as
/// XNA bytes: the engine sample × `GlobalBrightness` (1.2, or 1.0 under
/// the Darkness debuff — `:109-116`), the float colour ctor truncates.
/// The Color-mode style transforms are draw-pass conventions (`gore.rs`'s
/// `light_color` note); the raw engine read is the gameplay sample. Out of
/// the computed rect the legacy engine reads 0 (`legacy.rs:341-350`).
fn heroicis_light_lum(app: &App, b: &WingFrameDust) -> f32 {
    let tx = b.mounted_cx as i32 >> 4;
    let ty = b.mounted_cy as i32 >> 4;
    let sample = if let Some(legacy) = app.legacy_light.as_ref() {
        legacy.get_color(tx, ty)
    } else {
        app.light.get(tx, ty)
    };
    let brightness = if b.blind { 1.0 } else { tr_core::lighting::GLOBAL_BRIGHTNESS };
    // `Lighting.GetColor` returns BYTES: `(int)(v * brightness * 255)` clamped
    // (`Lighting.cs:216-242`; the `main_dust::light_color_bytes` convention).
    // The map stores normalised 0..1 floats, so the *255 is required - without
    // it every lit sample truncates to 0/1 and the HSL luminance never leaves 0.
    let num = brightness * 255.0;
    let byte = |v: f32| ((v * num) as i32).clamp(0, 255) as u8;
    let bytes = [byte(sample[0]), byte(sample[1]), byte(sample[2]), 255];
    let (_, _, l) = crate::ui::color::rgb_to_hsl(bytes);
    // `Utils.Remap(l, 0.2, 0.4, 0, 1)` (`Utils.cs:345-348`, clamped):
    // the lerp collapses to the clamped inverse lerp.
    ((l as f32 - 0.2) / 0.2).clamp(0.0, 1.0)
}

/// W4-g66 gap-fill — `:30609-30691`: the Heroicis-49 dust — the Chicken
/// Bones shape gated on the mounted-centre light: the burst's each
/// iteration draws NextFloat and skips when `lum < draw`; the rest roll
/// needs `Next(6) == 0` AND `lum >= NextFloat()`. Tint White×200, scale
/// 0.6×(1+0.45×dir), the burst box 10×5 at (−20/−10)×dir, 15 down,
/// velocity (x*0.5, 2); the rest box 30×40, velocity.Y += bob*1.5,
/// position.Y += bob*2.
fn heroicis(app: &mut App, b: &WingFrameDust) {
    if b.wing_frame == 0 {
        return;
    }
    let bob = dev_row_bob(b.wing_frame);
    // `:30653`: num4 reads AFTER the frame-0 gate; no RNG draws in it.
    let lum = heroicis_light_lum(app, b);
    let center = Vec2::new(b.x + b.width * 0.5, b.y + b.height * 0.5);
    let dir = b.facing as f32;
    let tint = Color::new(255, 255, 255, 200);
    if b.just_switched && b.wing_frame == 6 && b.control_jump {
        for i in 0..8 {
            let gate = app
                .rngs
                .stream(crate::app::RngKey::UpdatePlayers)
                .next_float();
            if lum < gate {
                continue;
            }
            let off = if i < 4 {
                Vec2::new(-20.0 * dir, 15.0)
            } else {
                Vec2::new(-10.0 * dir, 15.0)
            };
            // `:30662`: the scale ARGUMENT's NextFloatDirection draw lands
            // BEFORE the spawn's default-block draws (C# argument order).
            let scale = {
                let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                0.6 * (1.0 + 0.45 * rng.next_float_direction())
            };
            let idx = ndd(app, center - Vec2::new(5.0, 2.5) + off, 10, 5, 264, 0, tint, scale);
            let d = app.dust.slot_mut(idx);
            let x = (d.position - center - off).x;
            d.position.y += x * 2.0;
            d.no_gravity = true;
            d.no_light_emittance = false;
            d.no_light = true;
            d.fade_in = 0.0;
            d.velocity = Vec2::new(x * 0.5, 2.0);
        }
    }
    if roll(app, 6) {
        let gate = app
            .rngs
            .stream(crate::app::RngKey::UpdatePlayers)
            .next_float();
        if lum >= gate {
            let off = Vec2::new(-30.0 * dir, 0.0);
            let scale = {
                let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                0.6 * (1.0 + 0.45 * rng.next_float_direction())
            };
            let idx = ndd(app, center - Vec2::new(15.0, 20.0) + off, 30, 40, 264, 0, tint, scale);
            let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
            let d = app.dust.slot_mut(idx);
            d.no_gravity = true;
            d.no_light_emittance = false;
            d.no_light = true;
            d.velocity *= 0.1;
            d.fade_in = 0.0;
            d.velocity.y += bob * 1.5;
            d.position.y += bob * 2.0;
            d.velocity += Vec2::new(b.vx, b.vy) * 0.3 * rng.next_float();
        }
    }
}

/// W4-g66 gap-fill — `:30693-30735`: the Kazzymodus-50 dust. The
/// `justSwitched && frame == 1` burst: 16 type-6 puffs (30×10 box at
/// centre +((−25/−5)×dir, −20), alpha 100, tint (50,120,255,200), scale
/// 1.2), velocity = (NextFloatDirection×5, NextFloatDirection×2), fadeIn
/// 1.2, noGravity. Then the 1/4 rest roll: the Next(2) side pick
/// (−30/+10 ×dir), the frame-1 ? −25 : −5 Y row, box 25×20, velocity =
/// (NextFloatDirection×2.5, NextFloat×3).
fn kazzymodus(app: &mut App, b: &WingFrameDust) {
    if b.wing_frame == 0 {
        return;
    }
    let center = Vec2::new(b.x + b.width * 0.5, b.y + b.height * 0.5);
    let dir = b.facing as f32;
    let tint = Color::new(50, 120, 255, 200);
    if b.just_switched && b.wing_frame == 1 {
        for i in 0..16 {
            let x_off = if i < 4 { -25.0 * dir } else { -5.0 * dir };
            let pos = center + Vec2::new(x_off, -20.0) - Vec2::new(15.0, 5.0);
            let idx = ndd(app, pos, 30, 10, 6, 100, tint, 1.2);
            let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
            let d = app.dust.slot_mut(idx);
            d.velocity.x = rng.next_float_direction() * 5.0;
            d.velocity.y = rng.next_float_direction() * 2.0;
            d.fade_in = 1.2;
            d.no_gravity = true;
            d.no_light_emittance = false;
        }
    }
    if roll(app, 4) {
        let side = roll(app, 2);
        let x_off = dir * if side { -30.0 } else { 10.0 };
        let y_row = if b.wing_frame == 1 { -25.0 } else { -5.0 };
        let pos = center + Vec2::new(x_off, y_row) - Vec2::new(12.5, 10.0);
        let idx = ndd(app, pos, 25, 20, 6, 100, tint, 1.2);
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let d = app.dust.slot_mut(idx);
        d.velocity.x = rng.next_float_direction() * 2.5;
        d.velocity.y = rng.next_float() * 3.0;
        d.fade_in = 1.2;
        d.no_gravity = true;
        d.no_light_emittance = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use tr_core::{UnifiedRandom, World};

    fn rig(seed: i32) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = World::new("dust".into(), 64, 128);
        app.gfx_quality.max_dust_to_draw = 6000;
        app.camera = Vec2::new(320.0, 640.0);
        *app.rngs.stream(crate::app::RngKey::UpdatePlayers) = UnifiedRandom::new(seed);
        app
    }

    fn burst(wings: i32) -> WingAirDust {
        WingAirDust {
            wings,
            x: 300.0,
            y: 600.0,
            width: 20.0,
            height: 42.0,
            vx: 3.0,
            vy: -5.01,
            facing: 1,
        }
    }

    fn live(app: &App) -> Vec<&tr_core::particles::Dust> {
        app.dust.slots().iter().filter(|d| d.active).collect()
    }

    fn stream_of(app: &mut App) -> UnifiedRandom {
        app.rngs.stream(crate::app::RngKey::UpdatePlayers).clone()
    }

    /// One `NewDust` default block on the mirror stream (six draws).
    fn default_block(
        rng: &mut UnifiedRandom,
        pos: Vec2,
        w: i32,
        h: i32,
        scale: f32,
    ) -> (Vec2, Vec2, f32) {
        let w = w.max(5);
        let h = h.max(5);
        let px = pos.x + rng.next_max(w - 4) as f32 + 4.0;
        let py = pos.y + rng.next_max(h - 4) as f32 + 4.0;
        let vx = rng.next_range(-20, 21) as f32 * 0.1;
        let vy = rng.next_range(-20, 21) as f32 * 0.1;
        let _ = rng.next_max(3);
        let sc = (1.0 + rng.next_range(-20, 21) as f32 * 0.01) * scale;
        (Vec2::new(px, py), Vec2::new(vx, vy), sc)
    }

    /// The mid-body single-puff arms (10/34/51/9): one gate roll, then the
    /// six default draws; the per-arm type/alpha/scale/post table.
    #[test]
    fn w4_g66_the_mid_body_wing_arms_roll_once_then_spawn() {
        // (wings, gate, type, alpha, scale, fadeIn, no_light, tint)
        let cases = [
            (10, 2, 76, 50, 0.6f32, 1.1f32, true, None),
            (34, 2, 261, 50, 0.6, 1.1, true, None),
            (51, 2, 261, 50, 0.6, 1.1, true, Some(Color::new(230, 130, 55, 255))),
            (9, 2, 6, 200, 2.0, 0.0, false, None),
        ];
        for (wings, gate, ty, alpha, scale, fade, no_light, tint) in cases {
            // Find a seed whose first gate roll passes.
            let mut seed = 1;
            while UnifiedRandom::new(seed).next_max(gate) != 0 {
                seed += 1;
            }
            let mut app = rig(seed);
            let b = burst(wings);
            drain_wing_air(&mut app, b);
            let slots = live(&app);
            assert_eq!(slots.len(), 1, "wings {wings}");
            let d = slots[0];
            assert_eq!(d.dust_type, ty);
            assert_eq!(d.alpha, alpha);
            let mut rng = UnifiedRandom::new(seed);
            let _ = rng.next_max(gate);
            let cx = b.x + 10.0;
            let mid_y = b.y + 21.0 - 15.0;
            let (pos, vel, mut sc) =
                default_block(&mut rng, Vec2::new(cx - 40.0, mid_y), 30, 30, scale);
            if ty == 6 {
                // The type-6 tweak chain rides inside new_dust
                // (`Dust.cs:214-219`): one extra draw, velocity.Y overwrite,
                // velocity.X ×0.3, scale ×0.7.
                let vy = rng.next_range(-10, 6) as f32 * 0.1;
                sc *= 0.7;
                assert_eq!(d.velocity, Vec2::new(vel.x * 0.3, vy) * 0.3);
            } else {
                assert_eq!(d.velocity, vel * 0.3);
            }
            assert_eq!(d.position, pos);
            assert_eq!(d.scale, sc);
            assert_eq!(d.fade_in, fade);
            assert!(d.no_gravity);
            assert_eq!(d.no_light, no_light);
            if let Some(c) = tint {
                assert_eq!(d.color, c);
            }
            assert_eq!(stream_of(&mut app), rng);
            // The gate refused: one draw, no spawn.
            let mut refuse = seed + 1;
            while UnifiedRandom::new(refuse).next_max(gate) == 0 {
                refuse += 1;
            }
            let mut app = rig(refuse);
            drain_wing_air(&mut app, burst(wings));
            assert_eq!(live(&app).len(), 0);
            let mut rng = UnifiedRandom::new(refuse);
            let _ = rng.next_max(gate);
            assert_eq!(stream_of(&mut app), rng);
        }
    }

    /// `:30818-30829` — wings 6: the 1/4 gate, type 55, velocity ×0.3, no
    /// noGravity (the arm never sets it).
    #[test]
    fn w4_g66_the_bone_wing_arm_keeps_gravity() {
        let mut seed = 1;
        while UnifiedRandom::new(seed).next_max(4) != 0 {
            seed += 1;
        }
        let mut app = rig(seed);
        let b = burst(6);
        drain_wing_air(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 1);
        let d = slots[0];
        assert_eq!(d.dust_type, 55);
        assert!(!d.no_gravity);
        let mut rng = UnifiedRandom::new(seed);
        let _ = rng.next_max(4);
        let (pos, vel, sc) =
            default_block(&mut rng, Vec2::new(b.x + 10.0 - 40.0, b.y + 21.0 - 15.0), 30, 30, 1.0);
        assert_eq!(d.position, pos);
        assert_eq!(d.velocity, vel * 0.3);
        assert_eq!(d.scale, sc);
        assert_eq!(stream_of(&mut app), rng);
    }

    /// `:30842-30874` — the Beetle pair: the first puff unconditional, the
    /// second on a 1/2 roll; both ÷4 then −playerVel.
    #[test]
    fn w4_g66_the_beetle_trail_folds_the_player_velocity() {
        for facing in [1, -1] {
            let mut app = rig(0x5EED);
            let mut b = burst(26);
            b.facing = facing;
            drain_wing_air(&mut app, b);
            let mut rng = UnifiedRandom::new(0x5EED);
            let cx = b.x + 10.0;
            let low = if facing == 1 { -30.0 } else { 6.0 };
            let second = if facing == 1 { 12.0 } else { -24.0 };
            let vel = Vec2::new(b.vx, b.vy);
            let slots = live(&app);
            let mut expect_count = 1;
            let (pos, v, sc) =
                default_block(&mut rng, Vec2::new(cx + low, b.y), 18, 42, 1.4);
            let d = slots[0];
            assert_eq!(d.dust_type, 217);
            assert_eq!(d.position, pos);
            assert_eq!(d.velocity, v / 4.0 - vel);
            assert_eq!(d.scale, sc);
            assert!(d.no_gravity && d.no_light);
            if rng.next_max(2) == 0 {
                expect_count += 1;
                let (pos, v, sc) =
                    default_block(&mut rng, Vec2::new(cx + second, b.y), 12, 21, 1.4);
                let d = slots[1];
                assert_eq!(d.position, pos);
                assert_eq!(d.velocity, v / 4.0 - vel);
                assert_eq!(d.scale, sc);
            }
            assert_eq!(slots.len(), expect_count);
            assert_eq!(stream_of(&mut app), rng);
        }
    }

    /// The tick side: a flag19 flying tick queues the WingAirVisuals
    /// snapshot BEFORE WingMovement drains the budget.
    #[test]
    fn w4_g66_the_flying_tick_queues_the_wing_air_snapshot() {
        let w = World::new("t".into(), 64, 128);
        let mut p = crate::player::Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, crate::player::PlayerInput::default());
        }
        p.equipped[3] = 822; // Frozen Wings — wings_logic 10
        p.wing_time = 100.0;
        let held = crate::player::PlayerInput { jump: true, ..Default::default() };
        p.update(&w, held); // the ground jump
        p.update(&w, held); // flag19: the flight
        let s = p.take_wing_air_dust().expect("the flying tick queues");
        assert_eq!(s.wings, 10);
        assert_eq!(s.facing, p.facing);
    }
    /// `:30875-30919` — the Festive pair: the SelectRandom draws (all 31),
    /// the Direct spawn, ÷4 then −playerVel/2; the second on a 1/3 roll.
    #[test]
    fn w4_g66_the_festive_trail_halves_the_velocity_fold() {
        let mut app = rig(0x5EED);
        let b = burst(37);
        drain_wing_air(&mut app, b);
        let mut rng = UnifiedRandom::new(0x5EED);
        let cx = b.x + 10.0;
        let vel = Vec2::new(b.vx, b.vy);
        let slots = live(&app);
        let mut expect = 1;
        let ty = [31, 31, 31][rng.next_max(3) as usize];
        assert_eq!(ty, 31);
        let (pos, v, sc) = default_block(&mut rng, Vec2::new(cx - 30.0, b.y), 24, 42, 1.0);
        let d = slots[0];
        assert_eq!(d.dust_type, 31);
        assert_eq!(d.position, pos);
        assert_eq!(d.velocity, v / 4.0 - vel / 2.0);
        assert_eq!(d.scale, sc);
        assert!(d.no_gravity && d.no_light);
        if rng.next_max(3) == 0 {
            expect += 1;
            let _ = rng.next_max(3); // the second SelectRandom
            let (pos, v, sc) = default_block(&mut rng, Vec2::new(cx + 12.0, b.y), 16, 21, 1.0);
            let d = slots[1];
            assert_eq!(d.position, pos);
            assert_eq!(d.velocity, v / 4.0 - vel / 2.0);
            assert_eq!(d.scale, sc);
        }
        assert_eq!(slots.len(), expect);
        assert_eq!(stream_of(&mut app), rng);
    }

    /// `:30920-30936` — wings 29: the 1/3 gate, the type-6 tweak chain, the
    /// ×0.3 fold and the 1/10 fadeIn 2 roll.
    #[test]
    fn w4_g66_the_nebula_hover_arm_rolls_the_fade() {
        let mut seed = 1;
        while UnifiedRandom::new(seed).next_max(3) != 0 {
            seed += 1;
        }
        let mut app = rig(seed);
        let b = burst(29);
        drain_wing_air(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 1);
        let d = slots[0];
        assert_eq!(d.dust_type, 6);
        let mut rng = UnifiedRandom::new(seed);
        let _ = rng.next_max(3);
        let (pos, vel, mut sc) =
            default_block(&mut rng, Vec2::new(b.x + 10.0 - 40.0, b.y + 21.0 - 15.0), 30, 30, 2.4);
        // The type-6 tweak: one draw, velocity.Y overwrite, X ×0.3, ×0.7.
        let vy = rng.next_range(-10, 6) as f32 * 0.1;
        sc *= 0.7;
        assert_eq!(d.position, pos);
        assert_eq!(d.velocity, Vec2::new(vel.x * 0.3, vy) * 0.3);
        assert_eq!(d.scale, sc);
        assert!(d.no_gravity);
        let fade = if rng.next_max(10) == 0 { 2.0 } else { 0.0 };
        assert_eq!(d.fade_in, fade);
        assert_eq!(stream_of(&mut app), rng);
    }

    /// `:30937-31012` — the Nebula 31 chain: the four sequential gates ride
    /// the stream in order; the mirror replays whichever pass.
    #[test]
    fn w4_g66_the_nebula_chain_gates_in_source_order() {
        let mut app = rig(0x5EED);
        let b = burst(31);
        drain_wing_air(&mut app, b);
        let mut rng = UnifiedRandom::new(0x5EED);
        let cx = b.x + 10.0;
        let mid_y = b.y + 21.0 - 15.0;
        let mut expect = Vec::new();
        // Gate 1: 1/6 type-86 mid.
        if rng.next_max(6) == 0 {
            let (pos, v, _) = default_block(&mut rng, Vec2::new(cx - 40.0, mid_y), 30, 30, 1.0);
            expect.push((86, pos, v * 0.2, 1.0f32, 1.2f32, 0));
        }
        // Gate 2: 1/3 type-240 mid.
        if rng.next_max(3) == 0 {
            let (pos, v, _) = default_block(&mut rng, Vec2::new(cx - 40.0, mid_y), 30, 30, 1.0);
            expect.push((240, pos, v * 0.2, 1.2, 0.0, 200));
        }
        // Gate 3: the 1/2 half-box gate.
        if rng.next_max(2) == 0 {
            if rng.next_max(6) == 0 {
                let (pos, v, _) = default_block(&mut rng, Vec2::new(cx + 12.0, b.y), 12, 21, 1.0);
                expect.push((86, pos, v * 0.2, 1.0, 1.2, 0));
            }
            if rng.next_max(3) == 0 {
                let (pos, v, _) = default_block(&mut rng, Vec2::new(cx + 12.0, b.y), 12, 21, 1.0);
                expect.push((240, pos, v * 0.2, 1.2, 0.0, 200));
            }
        }
        let slots = live(&app);
        assert_eq!(slots.len(), expect.len());
        for (d, (ty, pos, vel, sc, fade, alpha)) in slots.iter().zip(&expect) {
            assert_eq!(d.dust_type, *ty);
            assert_eq!(d.position, *pos);
            assert_eq!(d.velocity, *vel);
            assert_eq!(d.scale, *sc);
            assert_eq!(d.fade_in, *fade);
            if *alpha != 0 {
                assert_eq!(d.alpha, *alpha);
            }
            assert!(d.no_gravity);
        }
        assert_eq!(stream_of(&mut app), rng);
    }
    /// The glide snapshot helper.
    fn glide(wings: i32) -> WingGlideDust {
        WingGlideDust {
            wings,
            x: 300.0,
            y: 600.0,
            width: 20.0,
            height: 42.0,
            vx: 3.0,
            vy: 2.0,
            facing: 1,
            always_animated: true,
        }
    }

    /// `:27040-27083` — the glide 10/34/51 arms: 1/3, 1/3 (always-animated
    /// gated), 1/5 gates; the 34 gate refuses WITHOUT a roll when grounded.
    #[test]
    fn w4_g66_the_glide_mid_arms_gate_per_wing() {
        for (wings, gate, ty) in [(10, 3, 76), (34, 3, 261), (51, 5, 261)] {
            let mut seed = 1;
            while UnifiedRandom::new(seed).next_max(gate) != 0 {
                seed += 1;
            }
            let mut app = rig(seed);
            drain_wing_glide(&mut app, glide(wings));
            let slots = live(&app);
            assert_eq!(slots.len(), 1, "wings {wings}");
            assert_eq!(slots[0].dust_type, ty);
            assert_eq!(slots[0].fade_in, 1.1);
            let mut rng = UnifiedRandom::new(seed);
            let _ = rng.next_max(gate);
            let (pos, vel, sc) = default_block(
                &mut rng,
                Vec2::new(310.0 - 40.0, 600.0 + 21.0 - 15.0),
                30,
                30,
                0.6,
            );
            assert_eq!(slots[0].position, pos);
            assert_eq!(slots[0].velocity, vel * 0.3);
            assert_eq!(slots[0].scale, sc);
            assert_eq!(stream_of(&mut app), rng);
        }
        // The 34 always-animated gate refuses before the roll.
        let mut app = rig(0x5EED);
        let mut g = glide(34);
        g.always_animated = false;
        drain_wing_glide(&mut app, g);
        assert_eq!(live(&app).len(), 0);
        assert_eq!(stream_of(&mut app), UnifiedRandom::new(0x5EED));
    }

    /// `:27122-27148` — the 6/5 else-if: wings 6 rolls 1/10 at mid−12; wings
    /// 5 rolls 1/6 at the lower box. The 6 arm never reaches the 5 roll.
    #[test]
    fn w4_g66_the_glide_bone_and_fairy_arms_are_exclusive() {
        let mut seed = 1;
        while UnifiedRandom::new(seed).next_max(10) != 0 {
            seed += 1;
        }
        let mut app = rig(seed);
        drain_wing_glide(&mut app, glide(6));
        let slots = live(&app);
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].dust_type, 55);
        let mut rng = UnifiedRandom::new(seed);
        let _ = rng.next_max(10);
        let (pos, vel, sc) = default_block(
            &mut rng,
            Vec2::new(310.0 - 40.0, 600.0 + 21.0 - 12.0),
            30,
            20,
            1.0,
        );
        assert_eq!(slots[0].position, pos);
        assert_eq!(slots[0].velocity, vel * 0.3);
        assert_eq!(slots[0].scale, sc);
        assert_eq!(stream_of(&mut app), rng);
        // Wings 5: the 1/6 gate.
        let mut seed = 1;
        while UnifiedRandom::new(seed).next_max(6) != 0 {
            seed += 1;
        }
        let mut app = rig(seed);
        drain_wing_glide(&mut app, glide(5));
        let slots = live(&app);
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].dust_type, 58);
        let mut rng = UnifiedRandom::new(seed);
        let _ = rng.next_max(6);
        let (pos, vel, sc) =
            default_block(&mut rng, Vec2::new(310.0 - 30.0, 600.0), 18, 42, 1.2);
        assert_eq!(slots[0].position, pos);
        assert_eq!(slots[0].velocity, vel * 0.3);
        assert_eq!(slots[0].scale, sc);
        assert_eq!(stream_of(&mut app), rng);
    }

    /// `:27149-27191` — the jetpack glide flame: the 1/2 side roll picks the
    /// box, the type-6 tweak chain rides new_dust, then X ×0.3 / Y += 10.
    #[test]
    fn w4_g66_the_jetpack_glide_flame_picks_a_side() {
        for side in [0, 1] {
            let mut seed = 1;
            while UnifiedRandom::new(seed).next_max(2) != side {
                seed += 1;
            }
            let mut app = rig(seed);
            let g = glide(4);
            drain_wing_glide(&mut app, g);
            let slots = live(&app);
            assert_eq!(slots.len(), 1);
            let d = slots[0];
            assert_eq!(d.dust_type, 6);
            let mut rng = UnifiedRandom::new(seed);
            let _ = rng.next_max(2);
            let (x, y) = if side == 1 {
                (310.0 - 20.0, 600.0 + 42.0 - 18.0 + 6.0)
            } else {
                (310.0 - 26.0, 600.0 + 42.0 - 18.0)
            };
            let (pos, vel, mut sc) =
                default_block(&mut rng, Vec2::new(x, y), 8, 8, 1.5);
            let vy = rng.next_range(-10, 6) as f32 * 0.1;
            sc *= 0.7;
            assert_eq!(d.position, pos);
            assert_eq!(d.velocity, Vec2::new(vel.x * 0.3 * 0.3, vy + 10.0));
            assert_eq!(d.scale, sc);
            assert!(d.no_gravity);
            assert_eq!(stream_of(&mut app), rng);
        }
    }

    /// The tick side: the glide arm queues the snapshot and runs the frame
    /// pins — the wings-26/37 glide pins frame 2, the default arm too.
    #[test]
    fn w4_g66_the_glide_tick_queues_and_pins_the_frame() {
        let w = World::new("t".into(), 64, 128);
        let mut p = crate::player::Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, crate::player::PlayerInput::default());
        }
        p.equipped[3] = 2609; // Beetle Wings — wings_logic 26
        p.update(&w, crate::player::PlayerInput::default());
        assert_eq!(p.wings_logic, 26);
        // Airborne, falling, jump held, no wing budget: the glide arm.
        p.y -= 200.0;
        p.vy = 2.0;
        p.wing_time = 0.0;
        p.jump = 0;
        p.update(
            &w,
            crate::player::PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        let g = p.take_wing_glide_dust().expect("the glide queues");
        assert_eq!(g.wings, 26);
        assert!(g.vy > 0.0, "the pre-integration velocity");
        assert_eq!(p.wing_frame, 2, "the Beetle glide pin");
    }
    /// `:27240-27316` — the glide Beetle/Festive pairs: the Beetle reuses
    /// the WingAirVisuals shape; the Festive draws the colour NextFloat
    /// FIRST, then the SelectRandom, scale 0.7, second alpha 140.
    #[test]
    fn w4_g66_the_glide_beetle_and_festive_pairs() {
        // Beetle: the shared pair mirror.
        let mut app = rig(0x5EED);
        drain_wing_glide(&mut app, glide(26));
        let mut rng = UnifiedRandom::new(0x5EED);
        let vel = Vec2::new(3.0, 2.0);
        let slots = live(&app);
        let mut expect = 1;
        let (pos, v, sc) = default_block(&mut rng, Vec2::new(310.0 - 30.0, 600.0), 18, 42, 1.4);
        assert_eq!(slots[0].dust_type, 217);
        assert_eq!(slots[0].position, pos);
        assert_eq!(slots[0].velocity, v / 4.0 - vel);
        assert_eq!(slots[0].scale, sc);
        if rng.next_max(2) == 0 {
            expect += 1;
            let (pos, v, sc) = default_block(&mut rng, Vec2::new(310.0 + 12.0, 600.0), 12, 21, 1.4);
            assert_eq!(slots[1].position, pos);
            assert_eq!(slots[1].velocity, v / 4.0 - vel);
            assert_eq!(slots[1].scale, sc);
        }
        assert_eq!(slots.len(), expect);
        assert_eq!(stream_of(&mut app), rng);
        // Festive: the colour float, then the type roll, then the spawn.
        let mut app = rig(0x5EED);
        drain_wing_glide(&mut app, glide(37));
        let mut rng = UnifiedRandom::new(0x5EED);
        let _ = rng.next_float(); // the dead-arm colour roll
        let slots = live(&app);
        let mut expect = 1;
        let _ = rng.next_max(3); // SelectRandom
        let (pos, v, sc) = default_block(&mut rng, Vec2::new(310.0 - 30.0, 600.0), 24, 42, 0.7);
        assert_eq!(slots[0].dust_type, 31);
        assert_eq!(slots[0].position, pos);
        assert_eq!(slots[0].velocity, v / 4.0 - vel);
        assert_eq!(slots[0].scale, sc);
        if rng.next_max(3) == 0 {
            expect += 1;
            let _ = rng.next_max(3);
            let (pos, v, sc) = default_block(&mut rng, Vec2::new(310.0 + 12.0, 600.0), 12, 21, 0.7);
            assert_eq!(slots[1].position, pos);
            assert_eq!(slots[1].velocity, v / 4.0 - vel);
            assert_eq!(slots[1].scale, sc);
            assert_eq!(slots[1].alpha, 140);
        }
        assert_eq!(slots.len(), expect);
        assert_eq!(stream_of(&mut app), rng);
    }
    fn frame_burst(wings: i32) -> WingFrameDust {
        WingFrameDust {
            wings,
            x: 300.0,
            y: 600.0,
            width: 20.0,
            height: 42.0,
            vx: 3.0,
            vy: -5.01,
            facing: 1,
            hover_up: false,
            count: 1,
            eye_color: [105, 90, 75, 255],
            under_shirt_color: [160, 180, 215, 255],
            misc_counter: 0,
            just_switched: false,
            control_jump: true,
            wing_frame: 2,
            mounted_cx: 310.0,
            mounted_cy: 621.0,
            blind: false,
        }
    }

    /// `:29481-29525` — the jetpack flap flames: 2 puffs (4 hover-up), the
    /// inner-box pick, the type-6 tweak, the Y folds.
    #[test]
    fn w4_g66_the_jetpack_flap_flames_pick_the_inner_box() {
        for hover_up in [false, true] {
            let mut app = rig(0x5EED);
            let mut b = frame_burst(4);
            b.hover_up = hover_up;
            drain_wing_frame(&mut app, b);
            let count = if hover_up { 4 } else { 2 };
            let slots = live(&app);
            assert_eq!(slots.len(), count);
            let mut rng = UnifiedRandom::new(0x5EED);
            for (i, d) in slots.iter().enumerate() {
                assert_eq!(d.dust_type, 6);
                let mut x = 310.0 - 26.0;
                let mut y = 600.0 + 42.0 - 18.0;
                if i == 1 || i == 3 {
                    x = 310.0 - 20.0;
                    y += 6.0;
                }
                if i > 1 {
                    y += b.vy;
                }
                let (pos, vel, mut sc) =
                    default_block(&mut rng, Vec2::new(x, y), 8, 8, 1.75);
                let vy = rng.next_range(-10, 6) as f32 * 0.1;
                sc *= 0.7;
                let mut v = Vec2::new(vel.x * 0.3 * 0.1, vy + 2.0 - b.vy * 0.3);
                if count == 4 {
                    v.y += 6.0;
                }
                assert_eq!(d.position, pos);
                assert_eq!(d.velocity, v);
                assert_eq!(d.scale, sc);
                assert!(d.no_gravity);
            }
            assert_eq!(stream_of(&mut app), rng);
        }
    }

    /// `:29816-29858` — the Bat sparkle loops: the 4+4 gates ride the
    /// stream in order; the passing ones overwrite position/velocity.
    #[test]
    fn w4_g66_the_bat_sparkles_overwrite_the_ring() {
        let mut app = rig(0x5EED);
        let b = frame_burst(30);
        drain_wing_frame(&mut app, b);
        let mut rng = UnifiedRandom::new(0x5EED);
        let center = Vec2::new(310.0, 621.0);
        let dir = 1.0f32;
        let mut expect = Vec::new();
        for j in 0..4 {
            if rng.next_max(4) != 0 {
                continue;
            }
            let ang = -0.74539816f32 + std::f32::consts::PI / 8.0 * j as f32 + 0.03 * j as f32;
            let off = Vec2::new(ang.cos(), ang.sin()) * Vec2::new(-dir * 20.0, 20.0);
            let (_, _, sc) = default_block(&mut rng, center, 5, 5, 0.8);
            let pos = center + off;
            // `:29825` - `DirectionTo(dust.position)` is `Normalize(position -
            // Center)`, i.e. the cancellation of the big centre is part of the
            // C# float result (not a bare `off.normalize()`).
            let vel = (pos - center).normalize() * 2.0;
            let fade = if rng.next_max(10) == 0 { 0.5 } else { 0.0 };
            expect.push((pos, vel, sc, fade));
        }
        for k in 0..4 {
            if rng.next_max(8) != 0 {
                continue;
            }
            let ang = -0.7053982f32 + std::f32::consts::PI / 8.0 * k as f32 + 0.03 * k as f32;
            let shift = Vec2::new(-dir * 16.0, 0.0);
            let off = Vec2::new(ang.cos(), ang.sin()) * Vec2::new(dir * 20.0, 24.0) + shift;
            let (_, _, sc) = default_block(&mut rng, center, 5, 5, 0.5);
            let pos0 = center + off;
            let vel = (pos0 - center - shift).normalize() * 2.0;
            let pos = pos0 + vel * 5.0;
            let fade = if rng.next_max(10) == 0 { 0.5 } else { 0.0 };
            expect.push((pos, vel, sc, fade));
        }
        let slots = live(&app);
        assert_eq!(slots.len(), expect.len());
        for (d, (pos, vel, sc, fade)) in slots.iter().zip(&expect) {
            assert_eq!(d.dust_type, 229);
            assert_eq!(d.position, *pos);
            assert_eq!(d.velocity, *vel);
            assert_eq!(d.scale, *sc);
            assert_eq!(d.fade_in, *fade);
            assert!(d.no_gravity);
        }
        assert_eq!(stream_of(&mut app), rng);
    }

    /// `:30195-30203` + `:30289-30306` — the Rainbow glow and the Spooky
    /// shadow puffs.
    #[test]
    fn w4_g66_the_rainbow_glow_and_spooky_shadows() {
        // Rainbow: find a passing 1/8 seed.
        let mut seed = 1;
        while UnifiedRandom::new(seed).next_max(8) != 0 {
            seed += 1;
        }
        let mut app = rig(seed);
        let b = frame_burst(45);
        drain_wing_frame(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 1);
        let d = slots[0];
        assert_eq!(d.dust_type, 43);
        assert_eq!(d.fade_in, 0.4);
        assert_eq!(d.color, Color::new(127, 127, 127, 127));
        let mut rng = UnifiedRandom::new(seed);
        let _ = rng.next_max(8);
        let (pos, vel, sc) =
            default_block(&mut rng, Vec2::new(290.0, 630.0), 40, 24, 0.2);
        assert_eq!(d.position, pos);
        assert_eq!(d.velocity, (vel + Vec2::new(b.vx, b.vy)) * 0.35);
        assert_eq!(d.scale, sc);
        assert_eq!(stream_of(&mut app), rng);
        // Spooky: count 5 at the foot box, scale 0.7, Y += 0.5.
        let mut app = rig(0x5EED);
        let mut b = frame_burst(39);
        b.count = 5;
        drain_wing_frame(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 5);
        let mut rng = UnifiedRandom::new(0x5EED);
        for d in &slots {
            assert_eq!(d.dust_type, 31);
            assert_eq!(d.color, Color::new(0, 0, 0, 255));
            let (pos, vel, _) =
                default_block(&mut rng, Vec2::new(285.0, 622.0), 50, 20, 1.0);
            assert_eq!(d.position, pos);
            let mut v = vel * 0.4;
            v.y += 0.5;
            assert_eq!(d.velocity, v);
            assert_eq!(d.scale, 0.7);
        }
        assert_eq!(stream_of(&mut app), rng);
        // Count 0: no draws.
        let mut app = rig(0x5EED);
        let mut b = frame_burst(39);
        b.count = 0;
        drain_wing_frame(&mut app, b);
        assert_eq!(live(&app).len(), 0);
        assert_eq!(stream_of(&mut app), UnifiedRandom::new(0x5EED));
    }

    /// The tick side: the jetpack-4 flap arm queues the flames and runs its
    /// own 0..2 frame machine (not the default 0..3).
    #[test]
    fn w4_g66_the_jetpack_tick_runs_its_own_machine() {
        let w = World::new("t".into(), 64, 128);
        let mut p = crate::player::Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, crate::player::PlayerInput::default());
        }
        p.equipped[3] = 748; // Jetpack — wings_logic 4
        p.wing_time = 100.0;
        let held = crate::player::PlayerInput { jump: true, ..Default::default() };
        p.update(&w, held); // the ground jump: jump > 0 → the flap arm
        let f = p.take_wing_frame_dust().expect("the jump burst queues flames");
        assert_eq!(f.wings, 4);
        // The machine wraps at 3, not 4.
        p.wing_frame = 2;
        p.wing_frame_counter = 4;
        p.update(&w, held);
        assert_eq!(p.wing_frame, 0, "the wrap at 3");
        assert_eq!(p.wing_frame_counter, 0);
    }

    // -- W4-g66 gap-fill: the wings-33/38/47/49/50 appearance arms ----------
    // Witnesses WRITTEN NOT RUN (code-only mode).

    /// `:30324-30358` — the Fairy-33 pair: the eye-colour HSL round-trip,
    /// the two foot-corner pins, the ±2 velocity folds and the ×0.65 White
    /// clones. Each source spawn = the six default draws + the clone's own
    /// six (the clone re-spawns at scale 1 then overwrites).
    #[test]
    fn w4_g66_the_fairy_pair_pins_the_corners_and_clones() {
        let mut app = rig(0x5EED);
        let mut b = frame_burst(33);
        b.eye_color = [200, 40, 40, 255]; // a red-ish eye → hue ≈ 0
        drain_wing_frame(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 4, "two sources + two clones");
        let (h, _, _) = crate::ui::color::rgb_to_hsl(b.eye_color);
        let c = crate::ui::color::hsl_to_rgb(h, 1.0, 0.5, 255);
        let tint = Color::new(c[0], c[1], c[2], c[3]);
        let mut rng = UnifiedRandom::new(0x5EED);
        for m in 0..2 {
            let d = slots[m * 2];
            assert_eq!(d.dust_type, 182);
            assert_eq!(d.color, tint);
            assert_eq!(d.alpha, 127);
            let (_, vel, _) = default_block(&mut rng, Vec2::new(300.0, 600.0), 20, 42, 1.0);
            let corner_x = if m == 0 { 300.0 } else { 320.0 };
            assert_eq!(d.position, Vec2::new(corner_x, 642.0));
            assert!(d.no_gravity && d.no_light);
            assert_eq!(d.fade_in, 1.0);
            assert_eq!(d.scale, 1.0);
            // `:30329`: the spawn speeds are the player velocity; `:30340`/
            // `:30345` fold ±2 X / +2 Y minus 0.3× it.
            let sign = if m == 0 { -2.0 } else { 2.0 };
            let v = Vec2::new(
                vel.x + b.vx + sign - b.vx * 0.3,
                vel.y + b.vy + 2.0 - b.vy * 0.3,
            );
            assert_eq!(d.velocity, v);
            // The clone: its own default block at the pinned position,
            // then the copied-field overwrite + the ×0.65 White re-tint.
            let cl = slots[m * 2 + 1];
            let _ = default_block(&mut rng, Vec2::new(corner_x, 642.0), 5, 5, 1.0);
            assert_eq!(cl.position, d.position);
            assert_eq!(cl.velocity, d.velocity);
            assert_eq!(cl.scale, 0.65);
            assert_eq!(cl.fade_in, 0.65);
            assert_eq!(cl.color, Color::new(255, 255, 255, 255));
        }
        assert_eq!(stream_of(&mut app), rng);
        // Facing left shifts both corners −4 (`:30325`).
        let mut app = rig(0x5EED);
        let mut b = frame_burst(33);
        b.facing = -1;
        drain_wing_frame(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots[0].position.x, 296.0);
        assert_eq!(slots[2].position.x, 316.0);
    }

    /// `:30408-30445` — the Leaf-38 orbits: nine type-267 perfects tinted
    /// underShirtColor, the miscCounter phase, the three anchors/scales,
    /// the −0.33/−0.66 velocity folds.
    #[test]
    fn w4_g66_the_leaf_orbits_ride_the_misc_phase() {
        let mut app = rig(0x5EED);
        let mut b = frame_burst(38);
        b.misc_counter = 7;
        drain_wing_frame(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 9);
        let mut rng = UnifiedRandom::new(0x5EED);
        let center = Vec2::new(310.0, 621.0);
        let value = Vec2::new(b.vx, b.vy) * 0.5;
        let tau = std::f32::consts::PI * 2.0;
        let tint = Color::new(160, 180, 215, 255);
        for n in 0..3 {
            let fold = match n {
                1 => Vec2::new(b.vx, b.vy) * -0.33,
                2 => Vec2::new(b.vx, b.vy) * -0.66,
                _ => Vec2::ZERO,
            };
            for (k, (anchor, denom, scale)) in [
                (Vec2::new(-39.0, 6.0), -15.0f32, 0.47f32),
                (Vec2::new(-23.0, 2.0), -15.0, 0.35),
                (Vec2::new(-31.0, -6.0), -20.0, 0.49),
            ]
            .into_iter()
            .enumerate()
            {
                let d = slots[n * 3 + k];
                let ang = 7.0 / denom * tau;
                let off = anchor + Vec2::new(2.0, 0.0).rotate(Vec2::from_angle(ang));
                let _ = default_block(&mut rng, center + off + fold, 5, 5, 1.0);
                assert_eq!(d.dust_type, 267);
                assert_eq!(d.color, tint);
                assert_eq!(d.position, center + off + fold);
                assert_eq!(d.velocity, value);
                assert_eq!(d.scale, scale);
                assert!(d.no_gravity && d.no_light);
            }
        }
        assert_eq!(stream_of(&mut app), rng);
    }

    /// `:30573-30606` — the Chicken Bones-47 burst + rest roll: the scale
    /// argument's NextFloatDirection precedes the default block; the
    /// X-delta doubles into Y; the rest roll's bob and NextFloat fold.
    #[test]
    fn w4_g66_the_chicken_bones_burst_and_rest_roll() {
        let mut app = rig(0x5EED);
        let mut b = frame_burst(47);
        b.wing_frame = 6;
        b.just_switched = true;
        drain_wing_frame(&mut app, b);
        let slots = live(&app);
        assert!(slots.len() >= 8, "the 8-burst always spawns");
        let mut rng = UnifiedRandom::new(0x5EED);
        let center = Vec2::new(310.0, 621.0);
        for (i, d) in slots.iter().take(8).enumerate() {
            assert_eq!(d.dust_type, 264);
            assert_eq!(d.color, Color::new(50, 120, 255, 200));
            let off = if i < 4 {
                Vec2::new(-25.0, 20.0)
            } else {
                Vec2::new(-5.0, 20.0)
            };
            let sc_arg = 1.1 * (1.0 + 0.25 * rng.next_float_direction());
            let (pos, _, sc) =
                default_block(&mut rng, center - Vec2::new(5.0, 5.0) + off, 10, 10, sc_arg);
            let x = (pos - center - off).x;
            assert_eq!(d.position, Vec2::new(pos.x, pos.y + x * 2.0));
            assert_eq!(d.velocity, Vec2::new(x * 0.5, 3.0));
            assert_eq!(d.scale, sc);
            assert_eq!(d.fade_in, 0.0);
            assert!(d.no_gravity && d.no_light);
        }
        // The rest gate: Next(6); on a pass, the scale draw + block + bob.
        if rng.next_max(6) == 0 {
            assert_eq!(slots.len(), 9);
            let d = slots[8];
            let off = Vec2::new(-30.0, 0.0);
            let sc_arg = 1.1 * (1.0 + 0.25 * rng.next_float_direction());
            let (pos, vel, sc) =
                default_block(&mut rng, center - Vec2::new(15.0, 25.0) + off, 30, 50, sc_arg);
            let bob = 0.5f32; // frame 6
            let fold = rng.next_float();
            let mut v = vel * 0.1;
            v.y += bob * 2.0;
            v += Vec2::new(b.vx, b.vy) * 0.3 * fold;
            assert_eq!(d.position, Vec2::new(pos.x, pos.y + bob * 2.0));
            assert_eq!(d.velocity, v);
            assert_eq!(d.scale, sc);
        } else {
            assert_eq!(slots.len(), 8);
        }
        assert_eq!(stream_of(&mut app), rng);
        // Frame 0 never draws (`:30531`).
        let mut app = rig(0x5EED);
        let mut b = frame_burst(47);
        b.wing_frame = 0;
        drain_wing_frame(&mut app, b);
        assert_eq!(live(&app).len(), 0);
        assert_eq!(stream_of(&mut app), UnifiedRandom::new(0x5EED));
    }

    /// `:30653-30690` — the Heroicis-49 light gate: an unlit map (lum 0)
    /// draws the per-iteration NextFloat gates and NOTHING else; a lit
    /// map (lum 1) spawns all eight. The gate draw precedes the scale draw.
    #[test]
    fn w4_g66_the_heroicis_rolls_gate_on_the_light() {
        // Dark: the default LightMap reads 0 outside its rect → lum 0.
        // Mirror the gate draws exactly (a 0.0 NextFloat passes `lum >=`).
        let mut app = rig(0x5EED);
        let mut b = frame_burst(49);
        b.wing_frame = 6;
        b.just_switched = true;
        drain_wing_frame(&mut app, b);
        let mut rng = UnifiedRandom::new(0x5EED);
        let mut expect = 0usize;
        let center = Vec2::new(310.0, 621.0);
        for i in 0..8 {
            if 0.0f32 < rng.next_float() {
                continue;
            }
            let off = if i < 4 {
                Vec2::new(-20.0, 15.0)
            } else {
                Vec2::new(-10.0, 15.0)
            };
            let sc_arg = 0.6 * (1.0 + 0.45 * rng.next_float_direction());
            let _ = default_block(&mut rng, center - Vec2::new(5.0, 2.5) + off, 10, 5, sc_arg);
            expect += 1;
        }
        if rng.next_max(6) == 0 && 0.0f32 >= rng.next_float() {
            let sc_arg = 0.6 * (1.0 + 0.45 * rng.next_float_direction());
            let _ = default_block(
                &mut rng,
                center - Vec2::new(15.0, 20.0) + Vec2::new(-30.0, 0.0),
                30,
                40,
                sc_arg,
            );
            let _ = rng.next_float();
            expect += 1;
        }
        assert_eq!(live(&app).len(), expect, "the dark map starves the rolls");
        assert_eq!(stream_of(&mut app), rng);
        // Lit: compute a bright surface map over the mounted centre tile
        // (310/16=19, 621/16=38) — sky 1.0 above the surface reads ≥ 0.4
        // luminance after the ×1.2 brightness, so lum pins to 1.
        let mut app = rig(0x5EED);
        app.world.world_surface = 60.0;
        app.light = tr_core::lighting::LightMap::compute(
            &app.world,
            10,
            30,
            30,
            45,
            &tr_core::lighting::LightParams { sky: [1.0; 3], day_time: true, ..Default::default() },
        );
        let mut b = frame_burst(49);
        b.wing_frame = 6;
        b.just_switched = true;
        let lum = heroicis_light_lum(&app, &b);
        assert!(lum > 0.0, "the sky-lit centre passes some gates: {lum}");
        drain_wing_frame(&mut app, b);
        let slots = live(&app);
        let mut rng = UnifiedRandom::new(0x5EED);
        let center = Vec2::new(310.0, 621.0);
        let mut expect: Vec<(Vec2, Vec2, f32)> = Vec::new();
        for i in 0..8 {
            if lum < rng.next_float() {
                continue;
            }
            let off = if i < 4 {
                Vec2::new(-20.0, 15.0)
            } else {
                Vec2::new(-10.0, 15.0)
            };
            let sc_arg = 0.6 * (1.0 + 0.45 * rng.next_float_direction());
            let (pos, _, sc) =
                default_block(&mut rng, center - Vec2::new(5.0, 2.5) + off, 10, 5, sc_arg);
            let x = (pos - center - off).x;
            expect.push((Vec2::new(pos.x, pos.y + x * 2.0), Vec2::new(x * 0.5, 2.0), sc));
        }
        let mut rest = None;
        if rng.next_max(6) == 0 && lum >= rng.next_float() {
            let off = Vec2::new(-30.0, 0.0);
            let sc_arg = 0.6 * (1.0 + 0.45 * rng.next_float_direction());
            let (pos, vel, sc) =
                default_block(&mut rng, center - Vec2::new(15.0, 20.0) + off, 30, 40, sc_arg);
            let bob = 0.5f32; // frame 6
            let mut v = vel * 0.1;
            v.y += bob * 1.5;
            v += Vec2::new(b.vx, b.vy) * 0.3 * rng.next_float();
            rest = Some((Vec2::new(pos.x, pos.y + bob * 2.0), v, sc));
        }
        assert_eq!(slots.len(), expect.len() + rest.is_some() as usize);
        for (d, (pos, vel, sc)) in slots.iter().zip(&expect) {
            assert_eq!(d.dust_type, 264);
            assert_eq!(d.color, Color::new(255, 255, 255, 200));
            assert_eq!(d.position, *pos);
            assert_eq!(d.velocity, *vel);
            assert_eq!(d.scale, *sc);
            assert!(d.no_gravity && d.no_light);
        }
        if let Some((pos, vel, sc)) = rest {
            let d = slots[expect.len()];
            assert_eq!(d.position, pos);
            assert_eq!(d.velocity, vel);
            assert_eq!(d.scale, sc);
        }
        assert_eq!(stream_of(&mut app), rng);
    }

    /// `:30700-30734` — the Kazzymodus-50 burst + rest roll: the 16-spawn
    /// velocity overwrites (direction×5 / direction×2), the Next(2) side
    /// pick and the frame-1 Y row.
    #[test]
    fn w4_g66_the_kazzymodus_burst_overwrites_velocity() {
        let mut app = rig(0x5EED);
        let mut b = frame_burst(50);
        b.wing_frame = 1;
        b.just_switched = true;
        drain_wing_frame(&mut app, b);
        let slots = live(&app);
        assert!(slots.len() >= 16);
        let mut rng = UnifiedRandom::new(0x5EED);
        let center = Vec2::new(310.0, 621.0);
        for (i, d) in slots.iter().take(16).enumerate() {
            let x_off = if i < 4 { -25.0 } else { -5.0 };
            let pos = center + Vec2::new(x_off, -20.0) - Vec2::new(15.0, 5.0);
            let (_, _, sc) = default_block(&mut rng, pos, 30, 10, 1.2);
            // The type-6 tweak chain (`Dust.cs:214-224`): the extra vy draw,
            // X ×0.3, scale ×0.7 — then the arm's velocity OVERWRITES.
            let _ = rng.next_range(-10, 6);
            assert_eq!(d.dust_type, 6);
            assert_eq!(d.alpha, 100);
            assert_eq!(d.color, Color::new(50, 120, 255, 200));
            assert_eq!(d.velocity.x, rng.next_float_direction() * 5.0);
            assert_eq!(d.velocity.y, rng.next_float_direction() * 2.0);
            assert_eq!(d.scale, sc * 0.7);
            assert_eq!(d.fade_in, 1.2);
            assert!(d.no_gravity);
        }
        if rng.next_max(4) == 0 {
            assert_eq!(slots.len(), 17);
            let d = slots[16];
            let side = rng.next_max(2) == 0;
            let x_off = if side { -30.0 } else { 10.0 };
            let pos = center + Vec2::new(x_off, -25.0) - Vec2::new(12.5, 10.0);
            let (_, _, sc) = default_block(&mut rng, pos, 25, 20, 1.2);
            let _ = rng.next_range(-10, 6);
            assert_eq!(d.velocity.x, rng.next_float_direction() * 2.5);
            assert_eq!(d.velocity.y, rng.next_float() * 3.0);
            assert_eq!(d.scale, sc * 0.7);
            assert_eq!(d.fade_in, 1.2);
        } else {
            assert_eq!(slots.len(), 16);
        }
        assert_eq!(stream_of(&mut app), rng);
    }
}
