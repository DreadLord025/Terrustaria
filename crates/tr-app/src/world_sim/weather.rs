//! Weather (plan stage A4): the rain field's per-frame update. M1.4-BG-22
//! adds the shared 60 Hz world tick ([`update_world_tick`]) — the dayRate
//! cluster vanilla runs inside one `Main.Update`, on one `FixedStepper`
//! (`App.world_tick_step`).

use glam::Vec2;

use crate::app::App;
use crate::biome::is_snow_biome;

/// Rain runs on its own 60 Hz accumulator; drops are simulated at the original
/// per-tick velocity regardless of frame rate. The visual ramps scale by
/// `max(1, day_rate)` (`Main.cs:61902-61906`).
pub(crate) fn update_rain(app: &mut App, dt: f32, screen: Vec2) {
    let snow = is_snow_biome(&app.world, app.camera);
    let day_rate = app.world.day_rate();
    app.rain
        .update(&app.world, app.camera, screen, app.zoom, dt, snow, day_rate);
}

/// The cross-system inputs of the vanilla lightning chain
/// (`Main.UpdateWeather`, `Main.cs:58784-58829`): the vanilla reads them
/// from Main statics; the port gathers them from `App` once per frame (the
/// same values `update_rain` used to feed `Rain::update` directly).
fn thunder_inputs(app: &App) -> crate::rain::ThunderInputs {
    crate::rain::ThunderInputs {
        storming: app.wind.storming,
        wind_target: app.wind.speed_target,
        max_raining: app.rain.max_raining(),
        atmo: crate::render::sky::atmosphere_factor(
            app.camera.y,
            app.world.world_surface,
            app.world.width,
        ),
        above_rock_layer: (app.scene_metrics.tile_center.1 as f64) < app.world.rock_layer,
        graveyard_intensity: app.scene_state.graveyard_visual_intensity,
        shimmer_alpha: app.scene_state.shimmer_alpha,
    }
}

/// The shared 60 Hz world tick (M1.4-BG-22): one `FixedStepper` on `App`
/// (`world_tick_step`, 60 Hz, cap 8) drives, in the exact C# per-tick order,
/// the dayRate cluster vanilla runs inside one `Main.Update`:
///
/// ```text
/// for (int i = 0; i < Main.dayRate; i++) { UpdateWeather(gameTime, i); }
/// ```                                                    // Main.cs:17228-17231
///
/// 1. the thunder arm — `UpdateWeather`'s `currentDayRateIteration == 0`
///    branch (`Main.cs:58737-58829`) — runs ONCE per tick, never per
///    iteration;
/// 2. the wind part (`Main.cs:58830-58950`) runs once per iteration, i.e.
///    `world.day_rate()` times per tick, fed the real `Main.maxRaining`
///    (`:58832` `windSpeedTarget * (1 + 5/9 * maxRaining)`);
/// 3. `Rain::update_time` (`Main.cs:64833-64949`) — `Main.UpdateTime`'s rain
///    section: the `ladyBugRainBoost` drain, the `rainTime` drain + the
///    strength-change roll while raining, the ×5.75/×4.25/ladybug start rolls
///    when not, and `StartRain`/`ChangeRain`/`StopRain`;
/// 4. `Sandstorm.UpdateTime` (`Main.cs:64966`) follows with `Main.dayRate` as
///    its first argument; while the TR_SANDSTORM pin holds
///    (`scene_state.forced_severity`) the state machine stays frozen — the
///    same gate `scene_scan` used before the call moved here.
///
/// Headless sessions (no gpu/renderer) tick too: the old draw-block wind tick
/// never ran there, the accumulator does. `frame()` calls this once per frame
/// with the raw dt, before `update_rain`.
pub(crate) fn update_world_tick(app: &mut App, dt: f32) {
    // The lightning chain's cross-system inputs (vanilla reads Main statics
    // live per call; the port snapshots them per frame like `update_rain`).
    let weather = thunder_inputs(app);
    // `Main.cs:64899`'s `statLifeMax >= 120` scan, folded to the port's one
    // player (`Main.player[i].statLifeMax`).
    let any_player_120 = app.player.as_ref().is_some_and(|p| p.max_hp >= 120);
    let App {
        world_tick_step,
        world,
        rain,
        wind,
        sandstorm,
        scene_state,
        ..
    } = app;
    world_tick_step.tick(dt, |_| {
        // (1) `Main.cs:58739`: `currentDayRateIteration == 0` — thunder once.
        rain.thunder_step(weather);
        // (2) `Main.cs:17228-17231` + `:58830-58950`: the wind walk, once per
        // dayRate iteration, with the real `Main.maxRaining` (`:58832`).
        let day_rate = world.day_rate();
        let max_raining = rain.max_raining();
        for _ in 0..day_rate {
            wind.update_rain(max_raining);
        }
        // (3) `Main.cs:64833-64949`: Main.UpdateTime's rain section — the
        // `ladyBugRainBoost` drain, the `rainTime` drain + strength-change roll
        // while raining, the ×5.75/×4.25/ladybug start rolls when not (the
        // `86400/dayRate` denominators), and `StartRain`/`ChangeRain`/`StopRain`.
        // Runs after the weather loop, before Sandstorm.UpdateTime
        // (`Main.cs:64966`); `remixWorld` reads Wind's stand-in field.
        rain.update_time(day_rate, world, any_player_120, wind.remix_world);
        // (4) `Main.cs:64966`; the TR_SANDSTORM pin freezes the state machine
        // (the periodic intention roll would fight the pin — phase-1 glue doc).
        if scene_state.forced_severity.is_none() {
            sandstorm.update_time(day_rate, wind.speed_current);
        }
    });
    world_tick_step.cap_backlog();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::rain::Rain;

    /// One 60 Hz step, as `frame()` feeds the tick on a shot run.
    const STEP: f32 = 1.0 / 60.0;

    fn arm_sandstorm(app: &mut App, time_left: i32) {
        app.sandstorm.happening = true;
        app.sandstorm.time_left = time_left;
    }

    #[test]
    fn test_DW_1_1_wind_loop_runs_before_the_sandstorm_in_one_tick() {
        // Order pin through real behavior: with `maxRaining` 0.8 the wind
        // drift (`Main.cs:58832`'s `5/9 * maxRaining` boost) lifts
        // `speed_current` across the sandstorm's 0.6 gate
        // (`Sandstorm.cs:22-25`) INSIDE this tick, so `UpdateTime` must see
        // sufficient wind and decay exactly `day_rate` — not the
        // `15 * day_rate` dead-wind penalty a sandstorm-first order would add.
        let mut app = flow_app(None, vec![], vec![]);
        app.world.fast_forward_to_dawn = true; // day_rate 60
        app.rain = Rain::with_forced(7, Some(0.8), false); // maxRaining 0.8
                                                           // The forced pin materializes inside `Rain::tick` — run one drop-sim
                                                           // tick so `max_raining()` really reads 0.8.
        app.rain.update(
            &app.world,
            app.camera,
            Vec2::new(1600.0, 900.0),
            1.0,
            STEP,
            false,
            1,
        );
        assert_eq!(app.rain.max_raining(), 0.8);
        app.wind.speed_current = 0.55;
        app.wind.speed_target = 0.8;
        app.wind.frozen = true; // no windCounter walk: only the drift moves current
        arm_sandstorm(&mut app, 10_000);
        update_world_tick(&mut app, STEP);
        assert!(
            app.wind.speed_current >= 0.6,
            "the drift crossed the gate inside the tick: {}",
            app.wind.speed_current
        );
        assert_eq!(
            app.sandstorm.time_left,
            10_000 - 60,
            "sufficient wind: decay = dayRate"
        );
    }

    #[test]
    fn test_DW_1_1_sandstorm_gets_the_dead_wind_penalty_at_day_rate_1() {
        // The contrast arm: at dayRate 1 the same drift stays under the gate,
        // so `UpdateTime` applies the 15×/tick penalty (`Sandstorm.cs:44-46`).
        let mut app = flow_app(None, vec![], vec![]);
        app.rain = Rain::with_forced(7, Some(0.8), false);
        // Materialize the forced pin (`Rain::tick` applies it) — see the 60×
        // test above.
        app.rain.update(
            &app.world,
            app.camera,
            Vec2::new(1600.0, 900.0),
            1.0,
            STEP,
            false,
            1,
        );
        app.wind.speed_current = 0.55;
        app.wind.speed_target = 0.8;
        app.wind.frozen = true;
        arm_sandstorm(&mut app, 10_000);
        update_world_tick(&mut app, STEP);
        assert!(app.wind.speed_current < 0.6);
        assert_eq!(app.sandstorm.time_left, 10_000 - 1 - 15);
    }

    #[test]
    fn test_DW_1_2_wind_counter_walk_fires_within_one_tick_at_60_not_at_1() {
        // `Main.windCounter--` runs once per dayRate iteration
        // (`Main.cs:58852`): armed at 60, one 60× tick walks the target and
        // resets the counter into its 900..2701 window (`Main.cs:58701`); at
        // dayRate 1 the same tick leaves 59 — the walk is 900+ ticks away.
        let mut fast = flow_app(None, vec![], vec![]);
        fast.world.fast_forward_to_dawn = true;
        fast.wind.counter = 60;
        update_world_tick(&mut fast, STEP);
        assert!(
            fast.wind.wind_counter() >= 900,
            "the walk fired within one 60× tick: {}",
            fast.wind.wind_counter()
        );

        let mut slow = flow_app(None, vec![], vec![]);
        slow.wind.counter = 60;
        update_world_tick(&mut slow, STEP);
        assert_eq!(
            slow.wind.wind_counter(),
            59,
            "one decrement at dayRate=1 — no walk"
        );
    }

    #[test]
    fn test_DW_1_4_headless_tick_advances_wind_and_sandstorm_without_gpu() {
        // `flow_app` has no gpu/renderer — the old draw-block wind tick never
        // ran here; the accumulator does. The tick also never touches the
        // world clock: TR_DAY's frozen time is frame()'s gate, not the tick's
        // (clock/tick independence).
        let mut app = flow_app(None, vec![], vec![]);
        assert!(app.gpu.is_none());
        let counter_before = app.wind.wind_counter();
        let time_before = app.world.time;
        arm_sandstorm(&mut app, 5_000);
        update_world_tick(&mut app, STEP);
        assert!(
            app.wind.wind_counter() < counter_before,
            "the wind ticked headless"
        );
        assert!(
            app.sandstorm.time_left < 5_000,
            "the sandstorm ticked headless"
        );
        assert_eq!(
            app.world.time, time_before,
            "the tick leaves the clock alone"
        );
    }

    #[test]
    fn test_DW_2_2_tick_feeds_the_player_scan() {
        // `Main.cs:64894-64909` through the tick: the port's one player's
        // `max_hp` is the `statLifeMax >= 120` scan. With a 100-HP player the
        // start rolls are gated off even on a fire-seeded stream; at 120 the
        // same seed starts the rain.
        let fire = (1..2_000_000u64)
            .find(|&s| {
                // The `Rain` stream replay (see rain.rs tests): first draw
                // ≡ 0 mod 8280 = the ×5.75 bound at dayRate 60.
                let mut r = (s ^ 0x2545_F491_4F6C_DD1D).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
                r = r.wrapping_add(0x9E37_79B9_7F4A_7C15);
                let mut z = r;
                z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
                (z ^ (z >> 31)) % 8280 == 0
            })
            .expect("a fire seed exists");
        let mut app = flow_app(None, vec![], vec![]);
        app.world.fast_forward_to_dawn = true; // day_rate 60 → num2 = 1440
        app.world.world_surface = 200.0; // `isThereAWorldSurface`
        app.rain = Rain::with_forced(fire, None, false);
        let mut player = crate::player::Player::spawn(&app.world);
        player.max_hp = 100; // below the `statLifeMax >= 120` gate
        app.player = Some(player);
        update_world_tick(&mut app, STEP);
        assert!(
            !app.rain.is_raining(),
            "a 100-HP player gates the start rolls off"
        );

        app.rain = Rain::with_forced(fire, None, false);
        let mut player = crate::player::Player::spawn(&app.world);
        player.max_hp = 120;
        app.player = Some(player);
        update_world_tick(&mut app, STEP);
        assert!(
            app.rain.is_raining(),
            "statLifeMax >= 120 arms the start rolls"
        );
    }
}
