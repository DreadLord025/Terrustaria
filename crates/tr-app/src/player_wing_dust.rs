//! W4-g66 вЂ” the wing-trail dust drains: `WingAirVisuals`
//! (`Player.cs:30750-31013`, the flag19 flying tick). Same queue convention
//! as `player_dust` (W4-d51): the tick snapshots the box/velocity at the C#
//! read moment (BEFORE `WingMovement` shapes it); this module replays each
//! `Dust.NewDust` on the shared `UpdatePlayers` stream in exact C# draw
//! order.
//!
//! The port folds `wings` (the look id) into `wings_logic` (the wing grant's
//! note), so `wingsLogic != wings` is always false вЂ” every
//! `noLightEmittance = flag` write lands `false` (the default the pool
//! already writes; the field is set explicitly to keep the mapping). The
//! `shader` writes are dropped fields (`particles.rs`'s note). `gravDir` is
//! the vanilla `1f`, so the `gravDir == -1` height folds collapse. The C#
//! `base.height / 2` / `base.width / 2` are INTEGER divisions вЂ” the port
//! truncates the f32 hitbox first.

use crate::app::App;
use crate::player::{WingAirDust, WingGlideDust};
use crate::player_dust::nd;
use glam::Vec2;
use tr_core::particles::Color;

/// The back-offset the mid-body arms share (`:30755-30759` et al.): 4 facing
/// left, в€’40 facing right, from the body centre line.
fn back_off(facing: i32) -> f32 {
    if facing == 1 { -40.0 } else { 4.0 }
}

/// The lower-trail offset (`:30832-30836` et al.): 6 / в€’30.
fn low_off(facing: i32) -> f32 {
    if facing == 1 { -30.0 } else { 6.0 }
}

/// The second-puff offset (`:30857-30861` et al.): в€’24 / +12.
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
/// ifs in source order вЂ” a wing id matches at most one, and the ROLLS of the
/// non-matching arms never happen (each gate short-circuits on the id).
pub(crate) fn drain_wing_air(app: &mut App, b: WingAirDust) {
    let cx = b.x + (b.width as i32 / 2) as f32;
    let mid_y = b.y + (b.height as i32 / 2) as f32 - 15.0;
    match b.wings {
        // `:30753-30766`: wings 10 вЂ” 1/2 type-76, fadeIn 1.1, noGravity,
        // noLight, velocity Г—0.3.
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
        // `:30767-30781`: wings 34 вЂ” the same shape on type 261.
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
        // `:30782-30796`: wings 51 вЂ” type 261 tinted (230,130,55).
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
        // `:30805-30817`: wings 9 вЂ” 1/2 type-6, alpha 200, scale 2.
        9 => {
            if roll(app, 2) {
                let idx = nd(app, Vec2::new(cx + back_off(b.facing), mid_y), 30, 30, 6, 0.0, 0.0, 200, Color::default(), 2.0);
                let d = app.dust.slot_mut(idx);
                d.no_gravity = true;
                d.no_light_emittance = false;
                d.velocity *= 0.3;
            }
        }
        // `:30818-30829`: wings 6 вЂ” 1/4 type-55, alpha 200, default scale.
        6 => {
            if roll(app, 4) {
                let idx = nd(app, Vec2::new(cx + back_off(b.facing), mid_y), 30, 30, 55, 0.0, 0.0, 200, Color::default(), 1.0);
                let d = app.dust.slot_mut(idx);
                d.velocity *= 0.3;
                d.no_light_emittance = false;
            }
        }
        // `:30830-30841`: wings 5 вЂ” 1/3 type-58 at the lower box.
        5 => {
            if roll(app, 3) {
                let idx = nd(app, Vec2::new(cx + low_off(b.facing), b.y), 18, b.height as i32, 58, 0.0, 0.0, 255, Color::default(), 1.2);
                let d = app.dust.slot_mut(idx);
                d.no_light_emittance = false;
                d.velocity *= 0.3;
            }
        }
        // `:30842-30874`: wings 26 вЂ” the unconditional type-217 pair.
        26 => beetle_trail(app, &b),
        // `:30875-30919`: wings 37 вЂ” the Direct type-31 pair.
        37 => festive_trail(app, &b),
        // `:30920-30936`: wings 29 вЂ” 1/3 type-6 scale 2.4 + the 1/10 fadeIn.
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
        // `:30937-31012`: wings 31 вЂ” the four-roll nebula chain.
        31 => nebula_trail(app, &b),
        _ => {}
    }
}

/// `:30842-30874` вЂ” the Beetle (26) trail: one type-217 puff at the lower
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

/// `:30875-30919` вЂ” the Festive (37) trail: one `NewDustDirect` of
/// `SelectRandom(31,31,31)` at the lower box (24 wide), then a 1/3 second
/// (16 wide, half height). Post: `noGravity`, `noLight`, `velocity /= 4`,
/// `velocity -= playerVel/2`. The `type == 55` re-tint arms are DEAD вЂ” the
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

/// `:30937-31012` вЂ” the Nebula (31) chain: 1/6 type-86 + 1/3 type-240 at the
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
    }
}#[cfg(test)]
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
                // velocity.X Г—0.3, scale Г—0.7.
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

    /// `:30818-30829` вЂ” wings 6: the 1/4 gate, type 55, velocity Г—0.3, no
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

    /// `:30842-30874` вЂ” the Beetle pair: the first puff unconditional, the
    /// second on a 1/2 roll; both Г·4 then в€’playerVel.
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
        p.equipped[3] = 822; // Frozen Wings вЂ” wings_logic 10
        p.wing_time = 100.0;
        let held = crate::player::PlayerInput { jump: true, ..Default::default() };
        p.update(&w, held); // the ground jump
        p.update(&w, held); // flag19: the flight
        let s = p.take_wing_air_dust().expect("the flying tick queues");
        assert_eq!(s.wings, 10);
        assert_eq!(s.facing, p.facing);
    }
    /// `:30875-30919` вЂ” the Festive pair: the SelectRandom draws (all 31),
    /// the Direct spawn, Г·4 then в€’playerVel/2; the second on a 1/3 roll.
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

    /// `:30920-30936` вЂ” wings 29: the 1/3 gate, the type-6 tweak chain, the
    /// Г—0.3 fold and the 1/10 fadeIn 2 roll.
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
        // The type-6 tweak: one draw, velocity.Y overwrite, X Г—0.3, Г—0.7.
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

    /// `:30937-31012` вЂ” the Nebula 31 chain: the four sequential gates ride
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

    /// `:27040-27083` вЂ” the glide 10/34/51 arms: 1/3, 1/3 (always-animated
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

    /// `:27122-27148` вЂ” the 6/5 else-if: wings 6 rolls 1/10 at midв€’12; wings
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

    /// `:27149-27191` вЂ” the jetpack glide flame: the 1/2 side roll picks the
    /// box, the type-6 tweak chain rides new_dust, then X Г—0.3 / Y += 10.
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
    /// pins вЂ” the wings-26/37 glide pins frame 2, the default arm too.
    #[test]
    fn w4_g66_the_glide_tick_queues_and_pins_the_frame() {
        let w = World::new("t".into(), 64, 128);
        let mut p = crate::player::Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, crate::player::PlayerInput::default());
        }
        p.equipped[3] = 2609; // Beetle Wings вЂ” wings_logic 26
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
}
