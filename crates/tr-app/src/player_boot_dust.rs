//! W4-g66 — the rocket-boots flame drain (`Player.RocketBootVisuals`,
//! `Player.cs:29330-29458`). Same queue convention as `player_dust`
//! (W4-d51): the tick snapshots at the C# read moment (the flame pair BEFORE
//! the thrust velocity writes, `:26918`); this module replays on the shared
//! `UpdatePlayers` stream in exact C# draw order.
//!
//! `netOffset` folds to zero (no net smoothing on the local player);
//! `gravDir` is the vanilla `1f`; the `shader` writes are dropped fields
//! (`particles.rs`'s note). The tier-4 `cShoe` dye pin reads the port's
//! `c_shoe` (no `shoeSlot` producer yet — the dye deviation family).

use crate::app::App;
use crate::player::RocketFlameDust;
use crate::player_dust::lerp_color;
use glam::Vec2;
use tr_core::particles::Color;

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

/// Drain `RocketBootVisuals` (`Player.cs:29330-29458`): the two flame puffs
/// (i=0 left, i=1 right) at the foot boxes `(x−4 | x+width−4, y+height−10)`,
/// 8×8, shifted −4 more while facing left. The per-tier switch picks
/// type/scale/alpha, the shared Direct spawn runs, then the per-tier
/// post-arms; the fairyBoots tail forces noGravity + noLightEmittance.
pub(crate) fn drain_rocket_flames(app: &mut App, b: RocketFlameDust) {
    // `:29332-29335`: no vanity tier → no flames at all.
    if b.vanity == 0 {
        return;
    }
    for i in 0..2 {
        // `:29343-29349`: the box + the facing shift; `num2` is the outward
        // kick sign (i=0 → +2, i=1 → −2).
        let num2: f32 = if i == 0 { 2.0 } else { -2.0 };
        let mut rx = if i != 0 {
            b.x as i32 + b.width as i32 - 4
        } else {
            b.x as i32 - 4
        };
        let mut ry = b.y as i32 + b.height as i32 - 10;
        if b.facing == -1 {
            rx -= 4;
        }
        // `:29350-29354`: the defaults + the kick vector.
        let mut ty = 6;
        let mut scale = 2.5f32;
        let mut alpha = 100;
        let mut num3 = 1.0f32;
        let kick = Vec2::new(-num2 - b.vx * 0.3, 2.0 - b.vy * 0.3);
        // `:29356-29423`: the per-tier pre-switch.
        match b.vanity {
            5 => {
                ty = 6;
                scale = 2.5;
            }
            1 => {
                if b.social_shadow {
                    ty = 27;
                    scale = 1.5;
                }
            }
            3 => {
                ty = 76;
                scale = 1.0;
                alpha = 20;
            }
            6 => {
                ty = 186;
                scale = 1.5;
                alpha = 20;
            }
            2 => {
                if b.fairy_boots {
                    // ONE type roll per puff (`NextFromList`, `Next(6)`).
                    ty = {
                        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                        [61, 61, 61, 242, 64, 63][rng.next_max(6) as usize]
                    };
                    scale = 2.0;
                    alpha = 120;
                } else {
                    ty = 16;
                    scale = 1.5;
                    alpha = 20;
                }
            }
            4 => {
                // `:29393-29421`: the Flame Waker roll — `Next(6)` ALWAYS
                // draws, then the dye pin, then the +2 box shift.
                let mut num4 = {
                    let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                    rng.next_max(6)
                };
                if b.c_shoe > 0 {
                    num4 = 2;
                }
                ry += 2;
                if num4 == 0 || num4 == 1 {
                    // The green spark arm: Lerp(LimeGreen, White,
                    // NextFloat*0.3), scale 0.66, the velocity folds, the
                    // 1/5 fadeIn 0.8 — then `continue` (no shared spawn).
                    let t = app
                        .rngs
                        .stream(crate::app::RngKey::UpdatePlayers)
                        .next_float()
                        * 0.3;
                    let color = lerp_color(Color::new(50, 205, 50, 255), Color::WHITE, t);
                    let idx = ndd(app, Vec2::new(rx as f32, ry as f32), 8, 8, 278, 100, color, 1.0);
                    let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                    let d = app.dust.slot_mut(idx);
                    d.scale = 0.66;
                    d.no_gravity = true;
                    d.velocity *= 0.25;
                    d.velocity -= Vec2::new(b.vx, b.vy) * 0.5;
                    d.velocity += kick * 0.5;
                    d.position += d.velocity * 4.0;
                    if rng.next_max(5) == 0 {
                        d.fade_in = 0.8;
                    }
                    continue;
                }
                ty = 107;
                alpha = 100;
                scale = 0.7;
                num3 = 0.5;
            }
            _ => {}
        }
        // `:29424-29427`: the shared Direct spawn + the kick folds.
        let idx = ndd(app, Vec2::new(rx as f32, ry as f32), 8, 8, ty, alpha, Color::default(), scale);
        {
            let d = app.dust.slot_mut(idx);
            d.velocity += kick;
            d.velocity *= num3;
        }
        // `:29428-29452`: the per-tier post-switch.
        match b.vanity {
            5 | 1 => {
                app.dust.slot_mut(idx).no_gravity = true;
            }
            2 | 6 => {
                app.dust.slot_mut(idx).velocity *= 0.1;
            }
            3 => {
                let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                let d = app.dust.slot_mut(idx);
                d.velocity *= 0.05;
                d.velocity.y += 0.15;
                d.no_light = true;
                if rng.next_max(2) == 0 {
                    d.no_gravity = true;
                    d.scale = 1.75;
                }
            }
            _ => {}
        }
        // `:29453-29457`: the fairyBoots tail.
        if b.fairy_boots {
            let d = app.dust.slot_mut(idx);
            d.no_gravity = true;
            d.no_light_emittance = true;
        }
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

    fn burst(vanity: i32) -> RocketFlameDust {
        RocketFlameDust {
            c_shoe: 0,
            vanity,
            x: 300.0,
            y: 600.0,
            width: 20.0,
            height: 42.0,
            vx: 3.0,
            vy: -5.01,
            facing: 1,
            fairy_boots: false,
            social_shadow: false,
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
        scale: f32,
    ) -> (Vec2, Vec2, f32) {
        let px = pos.x + rng.next_max(4) as f32 + 4.0;
        let py = pos.y + rng.next_max(4) as f32 + 4.0;
        let vx = rng.next_range(-20, 21) as f32 * 0.1;
        let vy = rng.next_range(-20, 21) as f32 * 0.1;
        let _ = rng.next_max(3);
        let sc = (1.0 + rng.next_range(-20, 21) as f32 * 0.01) * scale;
        (Vec2::new(px, py), Vec2::new(vx, vy), sc)
    }

    /// `:29332-29335` — vanity 0 spawns nothing, draws nothing.
    #[test]
    fn w4_g66_no_vanity_no_flames() {
        let mut app = rig(0x5EED);
        drain_rocket_flames(&mut app, burst(0));
        assert_eq!(live(&app).len(), 0);
        assert_eq!(stream_of(&mut app), UnifiedRandom::new(0x5EED));
    }

    /// `:29358-29435` — the tier-5 pair: two type-6 flames, the kick folds,
    /// noGravity; the type-6 tweak chain rides each spawn (7 draws each).
    #[test]
    fn w4_g66_the_hellfire_flame_pair_kicks_outward() {
        let mut app = rig(0x5EED);
        let b = burst(5);
        drain_rocket_flames(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 2);
        let mut rng = UnifiedRandom::new(0x5EED);
        for (i, d) in slots.iter().enumerate() {
            assert_eq!(d.dust_type, 6);
            let num2 = if i == 0 { 2.0 } else { -2.0 };
            let rx = if i == 0 { 296.0 } else { 316.0 };
            let (pos, vel, mut sc) =
                default_block(&mut rng, Vec2::new(rx, 632.0), 2.5);
            let vy = rng.next_range(-10, 6) as f32 * 0.1;
            sc *= 0.7;
            let kick = Vec2::new(-num2 - b.vx * 0.3, 2.0 - b.vy * 0.3);
            assert_eq!(d.position, pos);
            assert_eq!(d.velocity, Vec2::new(vel.x * 0.3, vy) + kick);
            assert_eq!(d.scale, sc);
            assert!(d.no_gravity);
        }
        assert_eq!(stream_of(&mut app), rng);
    }

    /// `:29369-29373` + `:29442-29451` — the tier-3 pair: type 76, ×0.05 +
    /// Y+0.15, noLight, the 1/2 noGravity/scale-1.75 roll per puff.
    #[test]
    fn w4_g66_the_frost_flame_pair_rolls_the_float() {
        let mut app = rig(0x5EED);
        let b = burst(3);
        drain_rocket_flames(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 2);
        let mut rng = UnifiedRandom::new(0x5EED);
        for (i, d) in slots.iter().enumerate() {
            assert_eq!(d.dust_type, 76);
            assert_eq!(d.alpha, 20);
            let num2 = if i == 0 { 2.0 } else { -2.0 };
            let rx = if i == 0 { 296.0 } else { 316.0 };
            let (pos, vel, sc) = default_block(&mut rng, Vec2::new(rx, 632.0), 1.0);
            let kick = Vec2::new(-num2 - b.vx * 0.3, 2.0 - b.vy * 0.3);
            let mut v = (vel + kick) * 0.05;
            v.y += 0.15;
            assert_eq!(d.position, pos);
            assert_eq!(d.velocity, v);
            assert!(d.no_light);
            if rng.next_max(2) == 0 {
                assert!(d.no_gravity);
                assert_eq!(d.scale, 1.75);
            } else {
                assert!(!d.no_gravity);
                assert_eq!(d.scale, sc);
            }
        }
        assert_eq!(stream_of(&mut app), rng);
    }

    /// `:29379-29392` — the tier-2 fairy arm: ONE Next(6) type roll per
    /// puff, the ×0.1 fold, the fairyBoots noGravity/noLightEmittance tail.
    #[test]
    fn w4_g66_the_fairy_flame_pair_rolls_per_puff() {
        let mut app = rig(0x5EED);
        let mut b = burst(2);
        b.fairy_boots = true;
        drain_rocket_flames(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 2);
        let mut rng = UnifiedRandom::new(0x5EED);
        for (i, d) in slots.iter().enumerate() {
            let ty = [61, 61, 61, 242, 64, 63][rng.next_max(6) as usize];
            assert_eq!(d.dust_type, ty);
            assert_eq!(d.alpha, 120);
            let num2 = if i == 0 { 2.0 } else { -2.0 };
            let rx = if i == 0 { 296.0 } else { 316.0 };
            let (pos, vel, sc) = default_block(&mut rng, Vec2::new(rx, 632.0), 2.0);
            let kick = Vec2::new(-num2 - b.vx * 0.3, 2.0 - b.vy * 0.3);
            let mut v = vel + kick;
            let mut sc = sc;
            if matches!(ty, 61 | 64 | 63) {
                // The type-6-family tweak (`Dust.cs:214-219`).
                let vy = rng.next_range(-10, 6) as f32 * 0.1;
                v = Vec2::new(v.x * 0.3, vy);
                sc *= 0.7;
            }
            assert_eq!(d.position, pos);
            assert_eq!(d.velocity, v * 0.1);
            assert_eq!(d.scale, sc);
            assert!(d.no_gravity && d.no_light_emittance);
        }
        assert_eq!(stream_of(&mut app), rng);
    }

    /// `:29393-29421` — the tier-4 Flame Waker: the Next(6) always draws;
    /// 0/1 → the green 278 spark (the colour float + the 1/5 fadeIn), else
    /// the 107 flame at ×0.5. The dye pin forces the 107 arm.
    #[test]
    fn w4_g66_the_flame_waker_roll_splits_spark_and_flame() {
        // Find a seed whose first Next(6) is 0 (the spark arm).
        let mut seed = 1;
        while UnifiedRandom::new(seed).next_max(6) != 0 {
            seed += 1;
        }
        let mut app = rig(seed);
        let b = burst(4);
        drain_rocket_flames(&mut app, b);
        let mut rng = UnifiedRandom::new(seed);
        let slots = live(&app);
        assert!(!slots.is_empty());
        let d = slots[0];
        assert_eq!(d.dust_type, 278);
        let _ = rng.next_max(6); // 0
        let t = rng.next_float() * 0.3;
        let color = lerp_color(Color::new(50, 205, 50, 255), Color::WHITE, t);
        assert_eq!(d.color, color);
        let (pos, vel, _) = default_block(&mut rng, Vec2::new(296.0, 634.0), 1.0);
        let kick = Vec2::new(-2.0 - b.vx * 0.3, 2.0 - b.vy * 0.3);
        let mut v = vel * 0.25 - Vec2::new(b.vx, b.vy) * 0.5 + kick * 0.5;
        assert_eq!(d.position, pos + v * 4.0);
        assert_eq!(d.velocity, v);
        assert_eq!(d.scale, 0.66);
        assert!(d.no_gravity);
        let fade = if rng.next_max(5) == 0 { 0.8 } else { 0.0 };
        assert_eq!(d.fade_in, fade);
        // The dye pin: cShoe > 0 forces num4 = 2 → the 107 flame.
        let mut app = rig(seed);
        let mut b = burst(4);
        b.c_shoe = 1;
        drain_rocket_flames(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 2);
        assert!(slots.iter().all(|d| d.dust_type == 107));
        let mut rng = UnifiedRandom::new(seed);
        for (i, d) in slots.iter().enumerate() {
            let _ = rng.next_max(6);
            let num2 = if i == 0 { 2.0 } else { -2.0 };
            let rx = if i == 0 { 296.0 } else { 316.0 };
            let (pos, vel, sc) = default_block(&mut rng, Vec2::new(rx, 634.0), 0.7);
            let kick = Vec2::new(-num2 - b.vx * 0.3, 2.0 - b.vy * 0.3);
            assert_eq!(d.position, pos);
            assert_eq!(d.velocity, (vel + kick) * 0.5);
            assert_eq!(d.scale, sc);
        }
        assert_eq!(stream_of(&mut app), rng);
    }

    /// The tick side: a thrust tick queues the flame pair BEFORE the thrust
    /// velocity writes and sets rocketFrame; a rest tick clears it.
    #[test]
    fn w4_g66_the_thrust_tick_queues_the_flame_pair() {
        let w = World::new("t".into(), 64, 128);
        let mut p = crate::player::Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, crate::player::PlayerInput::default());
        }
        p.pinned_rocket_boots = 2;
        p.rocket_boots = 2;
        p.vanity_rocket_boots = 2;
        p.rocket_time = 7;
        let held = crate::player::PlayerInput { jump: true, ..Default::default() };
        p.update(&w, held);
        p.update(&w, crate::player::PlayerInput::default());
        assert!(p.rocket_release);
        p.update(&w, held); // the trigger: rocketDelay = 10, then the thrust
        let f = p.take_rocket_flame_dust().expect("the thrust queues the pair");
        assert_eq!(f.vanity, 2);
        assert!(p.rocket_frame);
        // The snapshot's vy is the PRE-thrust velocity.
        assert!(f.vy > p.vy, "the thrust wrote after the snapshot");
        // A grounded rest tick clears the frame.
        for _ in 0..200 {
            p.update(&w, crate::player::PlayerInput::default());
            p.take_rocket_flame_dust();
        }
        assert!(!p.rocket_frame);
    }
}