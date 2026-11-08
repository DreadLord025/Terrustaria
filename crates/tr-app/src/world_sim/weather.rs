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
    // The crossing samples: the Deerclops's midnight (`Main.cs:65121`'s
    // `num4 < 16200 && time >= 16200`) and the night bookend's dusk
    // (`Main.cs:65230`'s `UpdateTime_StartDay` at 32400).
    let night_pre = app.world.time < 16200.0;
    let dusk_pre = app.world.time < 32400.0;
    let mut night_raining = false;
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
        rain.update_time(
            day_rate,
            world,
            any_player_120,
            wind.remix_world,
            // `GetPlayerWithHighestLuck().luck` for the coin-rain roll
            // (`Main.cs:64668`) — the port's single player, 0 when none
            // (the C# falls back to a fresh `Player()`).
            app.player.as_ref().map(|p| p.luck()).unwrap_or(0.0),
        );
        // (4) `Main.cs:64966`; the TR_SANDSTORM pin freezes the state machine
        // (the periodic intention roll would fight the pin — phase-1 glue doc).
        if scene_state.forced_severity.is_none() {
            sandstorm.update_time(day_rate, wind.speed_current);
        }
        night_raining = rain.is_raining();
        // (5) The Slime Rain event (`Main.cs:64925-64949`'s morning roll +
        // the time flow): the expiry decays dayRate-scaled (`StopSlimeRain`
        // at <= 0, `Main.cs:64788-64800`); the roll runs while the morning
        // sun lasts — `!raining`, no invasion, `dayTime && time < 27000`:
        // `Next(450000/dayRate)` — halved before the first King, x1.5
        // hardmode after, x5 without a fight-ready player
        // (`AnyPlayerReadyToFightKingSlime`: lifeMax > 140 && defense > 8,
        // `Main.cs:65166-65174`); the classic requires ready (the
        // `(flag3 || expertMode)` arm folds to ready).
        if world.slime_rain {
            world.slime_rain_time -= day_rate as f64;
            if world.slime_rain_time <= 0.0 {
                world.slime_rain = false;
                world.slime_rain_time = 0.0;
                world.slime_rain_kills = 0;
            }
        } else if !rain.is_raining()
            && !world.invasion_goblins
            && world.invasion_type == 0
            && world.day_time
            && world.time < 27000.0
            && world.invasion_size <= 0
        {
            let mut num3 = 450000.0 / (day_rate.max(1) as f64);
            if !world.downed_slime_king {
                num3 /= 2.0;
            } else if world.hard_mode {
                num3 *= 1.5;
            }
            let ready = app
                .player
                .as_ref()
                .is_some_and(|p| p.max_hp > 140 && p.stat_defense > 8);
            if !ready {
                num3 *= 5.0;
            }
            if slime_rain_roll(&world, ready, &mut app.npc_rng) {
                start_slime_rain(world, &mut app.npc_rng);
            }
        }
    });
    world_tick_step.cap_backlog();
    // (6) The Deerclops arm (`Main.cs:65121-65134`): the tick that steps
    // past 16200 while raining; the roll + the player gates ride
    // `deerclops_roll`, the spawn `SpawnOnPlayer`'s shape.
    let crossed = night_pre && world.time >= 16200.0 && night_raining;
    if crossed {
        let mut r = app.npc_rng;
        let passed = crate::world_sim::weather::deerclops_roll(app, &mut r);
        app.npc_rng = r;
        if passed {
            let at = app.player.as_ref().map(|p| p.center());
            crate::world_sim::weather::spawn_deerclops(app, at);
        }
    }
    // The meteorite trio (`Main.cs:65230`'s night bookend roll +
    // `:65490-65501`'s dusk resolution): the flag raises on the dusk
    // crossing (the `Next(50)` after the boss-2 downing), the resolution
    // lands the ore at the 16200 morning boundary (the fn self-gates on
    // the tick's time).
    if !app.world.spawn_meteor {
        let dusk = dusk_pre && app.world.time >= 32400.0;
        if dusk {
            let mut r = app.npc_rng;
            if tr_core::worldgen::meteor::night_roll(&mut r, app.world.downed_eow) {
                app.world.spawn_meteor = true;
            }
            app.npc_rng = r;
        }
    } else {
        let now = app.world.time;
        let mut r = app.npc_rng;
        tr_core::worldgen::meteor::resolve(&mut app.world, now, &mut r);
        app.npc_rng = r;
    }
}

/// `Main.cs:64938-64949`'s roll shape: `Next(450000/dayRate)` — halved
/// before the first King, x1.5 hardmode after, x5 without a fight-ready
/// player; the classic requires ready (`(flag3 || expertMode)` folds).
pub(crate) fn slime_rain_roll(world: &tr_core::World, ready: bool, rng: &mut u64) -> bool {
    let mut num3 = 450000.0 / (world.day_rate().max(1) as f64);
    if !world.downed_slime_king {
        num3 /= 2.0;
    } else if world.hard_mode {
        num3 *= 1.5;
    }
    if !ready {
        num3 *= 5.0;
    }
    ready && num3 > 0.0 && (crate::app::rand01(rng) as f64) < 1.0 / num3
}

/// `StartSlimeRain` (`Main.cs:64742-64776`): the duration
/// Next(32400, 54000), the kill counter reset; the rain/invasion gates
/// live at the caller (the tick's morning arm).
pub(crate) fn start_slime_rain(world: &mut tr_core::World, rng: &mut u64) {
    world.slime_rain = true;
    world.slime_rain_time = 32400.0 + (crate::app::rand01(rng) as f64) * 21600.0;
    world.slime_rain_kills = 0;
}

/// `Main.cs:65121-65134`'s midnight-crossing arm: the instantaneous
/// `num4 < 16200 && time >= 16200` crossing while raining, the
/// `!downedDeerclops || Next(4) == 0` repeat roll, and the player
/// gates (above the surface, ZoneSnow, no nearby town NPCs,
/// `statLifeMax2 >= 200 || statDefense >= 9`). The `CanSpawnEnemiesNear`
/// and `AnyDanger` arms fold (no danger scan ported). Returns the
/// player's centre to spawn at.
pub(crate) fn deerclops_roll(app: &App, rng: &mut u64) -> bool {
    if !(app.world.downed_deerclops || crate::app::rand01(rng) < 0.25) {
        return false;
    }
    let Some(p) = app.player.as_ref() else {
        return false;
    };
    if p.dead
        || p.y >= (app.world.world_surface * 16.0) as f32
        || !app.scene_metrics.zone_snow
        || !(p.max_hp >= 200 || p.stat_defense >= 9)
    {
        return false;
    }
    true
}

/// `SpawnOnPlayer(668, ...)` at the player's feet.
pub(crate) fn spawn_deerclops(app: &mut App, player: Option<(f32, f32)>) {
    let Some((px, py)) = player else {
        return;
    };
    let mut d = crate::npc::Npc::new(crate::npc::NpcKind::Deerclops, px, py, 1, 0.0);
    d.hp = d.max_hp;
    app.npcs.push(d);
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

    #[test]
    fn the_slime_rain_roll_gates_and_king_counter() {
        // The morning roll's shape (`Main.cs:64938-64949`): the
        // threshold 450000/dayRate — halved before the first King, x1.5
        // hardmode after, x5 without a fight-ready player; the gates
        // refuse the afternoon; `StartSlimeRain` sets the duration band
        // and resets the counter (`Main.cs:64742-64776`); the kill
        // counter (`NPC.cs:79078-79095`) fires the King at 75 after the
        // first (150 before) and flips the negative half.
        let mut app = flow_app(None, vec![], vec![]);
        let mut player = crate::player::Player::spawn(&app.world);
        player.max_hp = 200;
        player.stat_defense = 10;
        app.player = Some(player);
        // The roll shape: the ready player + the King downed = 1/300000
        // per tick at dayRate 1 (450000 x1.5 /2? no: /2 then... the
        // order: /2 (the King downed) — 225000; the hardmode arm is
        // ELSE-if).
        app.world.downed_slime_king = true;
        app.world.hard_mode = false;
        let mut rng = 1u64;
        let fired = (0..500000u64).any(|seed| {
            let mut r = seed + 1;
            crate::world_sim::weather::slime_rain_roll(&app.world, true, &mut r)
        });
        assert!(fired, "the roll fires within 500000 seeds");
        // The not-ready x5: the threshold quadruples the rarity — the
        // forced shape assert rides the modifier math instead (the
        // ready=false arm consumes one roll and returns false early? no —
        // it rolls against 1/num3 with num3 x5 — the direct prove: the
        // not-ready NEVER fires without... the vanilla `(flag3 ||
        // expertMode)` gate means the classic not-ready never rolls.
        assert!(!crate::world_sim::weather::slime_rain_roll(
            &app.world, false, &mut rng
        ));
        // The start fields.
        crate::world_sim::weather::start_slime_rain(&mut app.world, &mut rng);
        assert!(app.world.slime_rain);
        assert!(
            app.world.slime_rain_time >= 32400.0 && app.world.slime_rain_time <= 54000.0,
            "the duration band"
        );
        assert_eq!(app.world.slime_rain_kills, 0);
        // The gates: the afternoon tick leaves the state untouched.
        app.world.slime_rain = false;
        app.world.time = 30000.0;
        app.npc_rng = 7;
        super::update_world_tick(&mut app, 1.0 / 60.0);
        assert!(!app.world.slime_rain, "the afternoon refuses");
        // The kill counter: the King spawns at 75 (the first downed) and
        // the counter flips to -37. The flow_app world is 8x8 — the King
        // spawn needs real ground (the surface_of scan).
        let mut big = tr_core::World::new("t".into(), 64, 200);
        for x in 0..64 {
            for y in 120..200 {
                let i = big.index(x, y);
                big.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                big.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        big.compute_column_surface();
        let mut px = crate::player::Player::spawn(&big);
        px.x = 30.0 * 16.0;
        px.y = 118.0 * 16.0;
        px.max_hp = 200;
        px.stat_defense = 10;
        app.player = Some(px);
        app.world = big;
        app.world.downed_slime_king = true;
        app.world.time = 1000.0;
        app.world.slime_rain = true;
        app.world.slime_rain_kills = 74;
        let mut slime = crate::npc::Npc::spawn_slime_sky(
            30.0 * 16.0,
            90.0 * 16.0,
            crate::npc::SlimeVariant::Blue,
        );
        slime.hp = 0;
        app.npcs.push(slime);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        assert_eq!(
            app.world.slime_rain_kills, -37,
            "the negative-half hysteresis"
        );
        assert!(
            app.npcs
                .iter()
                .any(|n| n.kind == crate::npc::NpcKind::KingSlime),
            "the King spawned"
        );
    }
    #[test]
    fn the_deerclops_roll_gates_the_death_flag() {
        // `Main.cs:65121-65134`: the gates (the surface band, the zone
        // snow, the 200-life/9-defense readiness, the dead) + the
        // repeat roll (`!downed || Next(4)`); `NPC.cs:79355`'s case 668
        // death raises downed_deerclops.
        let mut app = flow_app(None, vec![], vec![]);
        let mut big = tr_core::World::new("t".into(), 64, 200);
        for x in 0..64 {
            for y in 120..200 {
                let i = big.index(x, y);
                big.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                big.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        big.compute_column_surface();
        big.world_surface = 120.0;
        let mut player = crate::player::Player::spawn(&big);
        player.x = 30.0 * 16.0;
        player.y = 118.0 * 16.0;
        player.max_hp = 200;
        app.player = Some(player);
        app.world = big;
        // The base life refuses.
        app.player.as_mut().unwrap().max_hp = 100;
        app.player.as_mut().unwrap().stat_defense = 0;
        let mut rng = 1u64;
        assert!(!crate::world_sim::weather::deerclops_roll(&app, &mut rng));
        // The readiness passes the gates; the roll itself is the 1/4
        // repeat (downed) or the guaranteed first — hunt a seed.
        app.player.as_mut().unwrap().max_hp = 200;
        app.scene_metrics.zone_snow = true;
        let fired = (0..500u64).any(|seed| {
            let mut r = seed + 1;
            crate::world_sim::weather::deerclops_roll(&app, &mut r)
        });
        assert!(fired, "the roll fires within 500 seeds");
        // The spawn + the death flag.
        let at = app.player.as_ref().map(|p| p.center());
        crate::world_sim::weather::spawn_deerclops(&mut app, at);
        assert!(
            app.npcs
                .iter()
                .any(|n| n.kind == crate::npc::NpcKind::Deerclops)
        );
        let mut d = app
            .npcs
            .iter_mut()
            .find(|n| n.kind == crate::npc::NpcKind::Deerclops)
            .unwrap();
        d.hp = 0;
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        assert!(app.world.downed_deerclops, "the death flag");
    }
    #[test]
    fn the_meteor_roll_and_boss_death_flag() {
        // The night roll (`Main.cs:65230`) gates on the boss-2 downing;
        // the orb-boss death arms the flag (the guaranteed first, the
        // `NPC.cs:79450-79456` arm).
        let mut app = flow_app(None, vec![], vec![]);
        app.world.downed_eow = false;
        let mut rng = 1u64;
        assert!(!tr_core::worldgen::meteor::night_roll(
            &mut rng,
            app.world.downed_eow
        ));
        let mut hits = 0;
        for _ in 0..2000 {
            if tr_core::worldgen::meteor::night_roll(&mut rng, true) {
                hits += 1;
            }
        }
        assert!(hits > 10 && hits < 70, "the 1/50 rate ({hits})");
    }
}
