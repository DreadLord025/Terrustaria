//! Weather (plan stage A4): the rain field's per-frame update. M1.4-BG-22
//! adds the shared 60 Hz world tick ([`update_world_tick`]) — the dayRate
//! cluster vanilla runs inside one `Main.Update`, on one `FixedStepper`
//! (`App.world_tick_step`).
//!
//! M5-SEED-2 seam — `rainsForAYear` (`SecretSeed.DoRainsForAYear`,
//! `WorldGen.cs:1872-1888`, called from `WorldGen.Reset` `:11235`) is NOT
//! wired: it writes world-size statics that the port keeps OUTSIDE the
//! `World` — `Main.rainTime = 1892160000` and `Main.raining = true` are
//! `Rain::rain_time`/`raining` (`rain.rs`), `Main.numClouds = 200`,
//! `Main.ChangeRain` and `Main.cloudAlpha = 0.9f` the `Rain` target/intensity,
//! and `Main.windSpeedTarget = ±0.8f` the `Wind` here. tr-core's worldgen
//! therefore has no field to set (there is no `World.raining`), and the
//! runtime owner (`App::rain`, `App::wind`) is not reachable from it. The arm
//! lands with a `Rain::start_year_long_event`-style entry on this module once
//! the worldgen can receive a callback (the same shape the
//! `moreLightning`/`noLightning` `ThunderInputs` feed below uses).

use glam::Vec2;

use crate::app::App;
use crate::biome::is_snow_biome;
use tr_core::{UnifiedRandom, TILE_PX};

/// Rain runs on its own 60 Hz accumulator; drops are simulated at the original
/// per-tick velocity regardless of frame rate. The visual ramps scale by
/// `max(1, day_rate)` (`Main.cs:61902-61906`).
pub(crate) fn update_rain(app: &mut App, dt: f32, screen: Vec2) {
    let snow = is_snow_biome(&app.world, app.camera);
    let day_rate = app.world.day_rate();
    let shimmer_alpha = app.scene_state.shimmer_alpha;
    let atmo = crate::render::sky::atmosphere_factor(
        app.camera.y,
        app.world.world_surface,
        app.world.width,
    );
    let snow_tile_count = app.scene_metrics.snow_tile_count;
    let zone_rain = app.scene_metrics.zone_rain;
    // The snow spawn cap scales with the gfx quality (S1-B-6,
    // `Main.cs:13057`).
    app.rain.gfx_quality = app.gfx_quality.quality;
    app.rain.update(
        &app.world,
        app.camera,
        screen,
        app.zoom,
        dt,
        snow,
        day_rate,
        shimmer_alpha,
        atmo,
        snow_tile_count,
        zone_rain,
    );
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
        // `Main.thunderSkipSound` (`Main.cs:58759`): only the unported
        // `ParticleOrchestrator.cs:1043` sets it true, so the port pins it.
        thunder_skip_sound: false,
        // The thunder strike's position inputs (`Main.cs:58752-58764`):
        // the scene centre and `Main.worldSurface * 16`.
        scene_center: [
            app.scene_metrics.center[0],
            app.scene_metrics.center[1],
        ],
        world_surface_px: app.world.world_surface as f32 * TILE_PX,
        // M5-SEED-2: the lightning micro seeds (`Main.moreLightningSeed` /
        // `Main.noLightningSeed`, set by `InitializeSecretSeeds`
        // `WorldGen.cs:580-586`) plus `Main.IsItRaining` — the three
        // `WorldGen.SpawnStormLightning` (`:71866-71881`) inputs `thunder_step`
        // reads. A classic world leaves them off, so its chain is unchanged.
        raining: app.rain.is_raining(),
        more_lightning: app
            .world
            .micro_seeds
            .contains(tr_core::seeds::MicroSeedFlags::MORE_LIGHTNING),
        no_lightning: app
            .world
            .micro_seeds
            .contains(tr_core::seeds::MicroSeedFlags::NO_LIGHTNING),
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
    // player (`Main.player[i].statLifeMax`). The same flag feeds
    // `Wind.any_player_120_life` (`Main.cs:58863-58871`, `:58889-58892`) — the
    // 0.35 walk cap applies until some player crosses 120 max life (S1-L-2).
    let any_player_120 = app.player.as_ref().is_some_and(|p| p.max_hp >= 120);
    app.wind.any_player_120_life = any_player_120;
    // `FreezeWindDirectionAndStrength.Enabled` (`Main.cs:58854`): the walk
    // gate — the drift toward the target still runs while frozen
    // (`Wind::update_rain`'s `frozen` early-return sits after the drift).
    app.wind.frozen = app.creative_powers.freeze_wind();
    // The crossing samples: the Deerclops's midnight (`Main.cs:65121`'s
    // `num4 < 16200 && time >= 16200`) and the night bookend's dusk
    // (`Main.cs:65230`'s `UpdateTime_StartDay` at 32400).
    let night_pre = app.world.time < 16200.0;
    let dusk_pre = app.world.time < 32400.0;
    let mut night_raining = false;
    // The homeless-arrival NO_HOBO count (W4-d119 phase 2) — fired after
    // the tick closure (the `notify_progression_net` &mut App seam).
    let mut homeless_events = 0u32;
    // `Main.cs:65043`'s `Main.netMode != 1` gate — the whole homeless block
    // is server/single-player only (computed before the field destructure).
    let not_client = !app.is_client();
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
        // `FreezeRainPower.Enabled` (`Main.cs:64866`/`:64893` read the manager
        // live per arm; the port snapshots per step like the other inputs).
        rain.set_freeze_rain(app.creative_powers.freeze_rain());
        // (1) `Main.cs:58739`: `currentDayRateIteration == 0` — thunder once.
        // M6-AUDIO-1: the strike returns its cue and position
        // (`Main.cs:58752-58761`), played here against the current listener.
        // S1-E-25: the case-43 re-roll loop reads the engine's live thunder
        // instances (`LegacySoundPlayer.cs:456-464`), and the pick replaces
        // the slot's instance (`:466-476`'s Stop() + CreateInstance).
        if let Some(strike) = rain.thunder_step(weather, &|slot| {
            app.audio.thunder_playing(slot)
        }) {
            app.audio
                .play_thunder(strike.slot, strike.cue, strike.x, strike.y, strike.pitch);
        }
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
        // (4) `Main.cs:64963-64966`: the event walks — CultistRitual
        // (the delay decay + the recheck window's AnyDanger arm),
        // BirthdayParty/LanternNight (the persistence clears riding the
        // sky edges), then Sandstorm; the TR_SANDSTORM pin freezes the
        // state machine (the periodic intention roll would fight the pin —
        // phase-1 glue doc). S1-F-14.
        // The cultist delay's decay: the `recheck` window's `AnyDanger` arm
        // walks the port's boss-flagged kinds (`CultistRitual.UpdateTime`).
        let any_danger = world.invasion_type > 0
            || app.npcs
                .iter()
                .any(|n| n.hp > 0 && crate::revenge::BOSS_KINDS.contains(&n.kind));
        if crate::world_events::update_time_cultist(&mut app.world_events, day_rate, any_danger) {
            // The ritual window opened — the spawn rides the caller's
            // floor/LOS gates (S1-F-14's TrySpawning predicate contract).
        }
        crate::world_events::update_time_party(&mut app.world_events);
        // `LanternNight.UpdateTime`'s persistence gate: the lanterns ride
        // the night (`LanternsCanPersist` = `!dayTime && LanternsCanStart()`).
        let lantern_can_persist = !world.day_time;
        let _ = crate::world_events::update_time_lanterns(
            &mut app.world_events,
            lantern_can_persist,
        );
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
        // `Main.cs:64848-64863`: the slime-rain clock — the positive
        // duration decays dayRate-scaled to the stop (`StopSlimeRain`), the
        // negative cooldown climbs back to zero.
        if world.slime_rain_time > 0.0 {
            world.slime_rain_time -= day_rate as f64;
            if world.slime_rain_time <= 0.0 {
                // The expiry's `StopSlimeRain` runs inside `Main.UpdateTime`'s
                // slime clock (`Main.cs:64853`) — the `UpdateTime` stream.
                stop_slime_rain(world, app.rngs.stream(crate::app::RngKey::UpdateTime));
            }
        } else if world.slime_rain_time < 0.0 {
            world.slime_rain_time += day_rate as f64;
            if world.slime_rain_time > 0.0 {
                world.slime_rain_time = 0.0;
            }
        }
        // `Main.cs:64925`'s gate: `!Main.raining && !NPC.BusyWithAnyInvasionOfSorts()`
        // (`NPC.cs:7022-7029` = `slimeRainTime == 0 && !bloodMoon && !eclipse
        // && !snowMoon && !pumpkinMoon && invasionType == 0 && !DD2Event.Ongoing`),
        // then `dayTime && time < 27000 && dayRate > 0`.
        if !rain.is_raining()
            && world.slime_rain_time == 0.0
            && !world.blood_moon
            && !world.eclipse
            && !world.snow_moon
            && !world.pumpkin_moon
            && world.invasion_type == 0
            && !world.dd2_event_ongoing
            && world.day_time
            && world.time < 27000.0
            && day_rate > 0
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
            // The roll + `StartSlimeRain` ride `UpdateTime` (`Main.cs:64945`
            // inside the rain section of `Main.UpdateTime`). `StartSlimeRain`
            // re-checks the rain/gate arms itself (`:64744-64768`).
            if slime_rain_roll(
                &world,
                ready,
                app.rngs.stream(crate::app::RngKey::UpdateTime),
            ) {
                start_slime_rain(
                    world,
                    rain.is_raining(),
                    app.rngs.stream(crate::app::RngKey::UpdateTime),
                );
            }
        }
        // `Main.UpdateSlimeRainWarning` (`Main.cs:65006`'s call, body at
        // `:65504-65522`): the armed countdown's announcement. The C# ticks it
        // in every `UpdateTime`, outside the slime block, so it runs here
        // unconditionally.
        if let Some(key) = update_slime_rain_warning(world) {
            crate::chat::broadcast_death_text(
                &mut app.chat,
                &crate::lang::text(key),
                crate::chat::COLOR_WORLD,
            );
        }
        // (7) `Main.cs:65004-65042`: the Travelling Merchant's visit
        // cycle. `NPC.travelNPC` (`NPC.cs:6541`) is raised alive while a
        // 368 exists (`NPC.cs:90314-90317`) and this block consumes it
        // once per tick; the departure runs first, the morning roll
        // second (the C#'s if/else-if ladder).
        let travel_present = app
            .npcs
            .iter()
            .any(|n| n.kind == crate::npc::NpcKind::TravellingMerchant);
        if travel_present {
            if !world.day_time || world.time > 48600.0 {
                unspawn_travel_npc(&mut app.npcs, app.player.as_ref(), &mut app.chat);
            }
        } else if !world.is_fast_forwarding()
            && world.day_time
            && world.time < 27000.0
            && travel_visit_roll(world, app.rngs.stream(crate::app::RngKey::UpdateTime))
        {
            // `:65027-65034`'s resident gate — the town NPCs, the Old
            // Man (37) and the Skeleton Merchant (453) excluded (neither
            // is ported).
            let residents = app
                .npcs
                .iter()
                .filter(|n| n.kind.is_town_resident())
                .count();
            if residents >= 2 {
                // `WorldGen.SpawnTravelNPC` is called from `Main.UpdateTime`
                // (`Main.cs:65037`) — the visit roll, the stock regeneration
                // and the site pick all ride `UpdateTime`.
                spawn_travel_npc(
                    world,
                    &mut app.npcs,
                    app.rngs.stream(crate::app::RngKey::UpdateTime),
                    app.player.as_ref(),
                    &mut app.chat,
                );
            }
        }
        // (7b) `Main.cs:65043-65062`: the HOMELESS arrival/departure pair,
        // right after the travel block (W4-d119 phase 2). The departure
        // runs at night / past 48600; the arrival rolls
        // `Next(27000/dayRate) == 0` over the first half of the day, gated
        // on `!IsFastForwardingTime() && GetWorldUpdateRate() > 0` — the
        // roll AND the whole `SpawnHomelessNPC` body ride `UpdateTime`.
        // The NO_HOBO notifications accumulate and fire after the closure
        // (the `notify_progression_net` borrow seam, the growth precedent).
        if not_client && (!world.day_time || world.time > 48600.0) {
            crate::town_spawn::unspawn_homeless_npc(
                &mut app.npcs,
                app.player.as_ref(),
                &mut app.chat,
            );
        } else if not_client
            && world.day_time
            && world.time < 27000.0
            && !world.is_fast_forwarding()
            // `GetWorldUpdateRate() > 0` (`WorldGen.cs:72020-72030`): the
            // FreezeTime power / a zero time-rate slider stop it.
            && app.creative_powers.time_rate() > 0.0
        {
            // `:65052-65057`: `num8 = max(dayRate, 1)`,
            // `maxValue = (int)(27000.0 / num8)`, `Next(maxValue) == 0`.
            let num8 = world.day_rate().max(1);
            let max_value = (27000.0 / num8 as f64) as i32;
            if app
                .rngs
                .stream(crate::app::RngKey::UpdateTime)
                .next_max(max_value)
                == 0
            {
                homeless_events += crate::town_spawn::spawn_homeless_npc(
                    world,
                    &mut app.npcs,
                    &mut app.town_rooms,
                    &mut app.town_arrival,
                    &mut app.town_spawn,
                    app.player.as_ref(),
                    app.rngs.stream(crate::app::RngKey::UpdateTime),
                    &mut app.chat,
                );
            }
        }
    });
    world_tick_step.cap_backlog();
    // The homeless-arrival NO_HOBO rows (`WorldGen.cs:5460`/`:5780`), after
    // the mutable pass releases the field borrows.
    for _ in 0..homeless_events {
        let _ = app.notify_progression_net(8);
    }
    // (6) The Deerclops arm (`Main.cs:65121-65134`): the tick that steps
    // past 16200 while raining; the roll + the player gates ride
    // `deerclops_roll`, the spawn `SpawnOnPlayer`'s shape.
    let crossed = night_pre && world.time >= 16200.0 && night_raining;
    if crossed {
        let zone_snow = app.scene_metrics.zone_snow;
        let passed = deerclops_roll(
            &app.world,
            app.player.as_ref(),
            zone_snow,
            // `Main.cs:65121-65134` runs inside `Main.UpdateTime` — the
            // `UpdateTime` stream.
            app.rngs.stream(crate::app::RngKey::UpdateTime),
        );
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
            // `UpdateTime_StartNight`'s roll (`Main.cs:65230`) — the
            // `UpdateTime` stream (the arm rides the port's weather phase;
            // the within-stream order deviation is the ANALYSIS note).
            if tr_core::worldgen::meteor::night_roll(
                app.rngs.stream(crate::app::RngKey::UpdateTime),
                app.world.downed_eow,
            ) {
                app.world.spawn_meteor = true;
            }
        }
    } else {
        // `HandleMeteorFall` runs inside `Main.UpdateTime` (`Main.cs:65141`)
        // — the resolution draws `UpdateTime`.
        let now = app.world.time;
        tr_core::worldgen::meteor::resolve(
            &mut app.world,
            now,
            app.rngs.stream(crate::app::RngKey::UpdateTime),
        );
    }
}

/// `Main.cs:64932-64947`'s roll shape: `num3 = (int)(450000.00000000006 /
/// dayRate)` — halved before the first King, x1.5 hardmode after, x5 without
/// a fight-ready player; the classic requires ready (`(flag3 || expertMode)`
/// folds to `ready`), and the roll is `Main.rand.Next(num3) == 0` — one
/// `Next(num3)` draw, consumed only when the gates pass (`&&` order).
pub(crate) fn slime_rain_roll(
    world: &tr_core::World,
    ready: bool,
    rng: &mut UnifiedRandom,
) -> bool {
    let mut num3 = (450000.00000000006f64 / (world.day_rate().max(1) as f64)) as i32;
    if !world.downed_slime_king {
        num3 /= 2;
    } else if world.hard_mode {
        num3 = (num3 as f64 * 1.5) as i32;
    }
    if !ready {
        num3 *= 5;
    }
    ready && num3 > 0 && rng.next_max(num3) == 0
}

/// `Main.StopSlimeRain()` (`Main.cs:64787-64820`): the single-player arm —
/// the flag down, the clock set to the negative cooldown
/// `-Next(3024, 6048) * 100` (it climbs back to zero in the tick's slime
/// clock), and the announcement armed (`slimeWarningTime = slimeWarningDelay`,
/// `:64811`). The kill counter is NOT cleared here — the C# only resets it in
/// `StartSlimeRain`. A no-op when the rain is already down (`:64789-64792`),
/// so the guard draws nothing.
pub(crate) fn stop_slime_rain(world: &mut tr_core::World, rng: &mut UnifiedRandom) {
    if !world.slime_rain {
        return;
    }
    world.slime_rain_time = -(rng.next_range(3024, 6048) as f64) * 100.0;
    world.slime_rain = false;
    world.slime_warning_time = SLIME_WARNING_DELAY;
}

/// `Main.slimeWarningDelay` (`Main.cs:66802`'s `420`).
pub(crate) const SLIME_WARNING_DELAY: i32 = 420;

/// `Main.UpdateSlimeRainWarning` (`Main.cs:65504-65522`): the 420-tick
/// countdown after a start/stop, then the announcement — `Lang.gen[74]`
/// ("Slime is falling from the sky!") while the rain clock is still
/// positive, `Lang.gen[75]` once it has gone negative. Returns the message
/// key to broadcast, if the countdown just expired.
pub(crate) fn update_slime_rain_warning(world: &mut tr_core::World) -> Option<&'static str> {
    if world.slime_warning_time <= 0 {
        return None;
    }
    world.slime_warning_time -= 1;
    if world.slime_warning_time > 0 {
        return None;
    }
    Some(if world.slime_rain_time > 0.0 {
        "LegacyWorldGen.74"
    } else {
        "LegacyWorldGen.75"
    })
}

/// `StartSlimeRain` (`Main.cs:64742-64785`) — the single-player arm the
/// port models: the `remixWorld`/`isThereAWorldSurface`/already-raining/
/// `Main.raining` gates (`:64744`, `:64761-64764`), the `slimeRainTime <= 0`
/// re-arm to `Next(32400, 54000)` (`:64765-64768`), the flag + the kill
/// counter reset and the announcement arm (`:64769-64777`). Returns whether
/// the rain started.
pub(crate) fn start_slime_rain(
    world: &mut tr_core::World,
    raining: bool,
    rng: &mut UnifiedRandom,
) -> bool {
    if world.slime_rain {
        return false;
    }
    // `Main.remixWorld || !Main.isThereAWorldSurface` (`:64744`): the port has
    // no remix worldgen, so only a world without a surface refuses.
    if world.world_surface <= 0.0 {
        return false;
    }
    if raining {
        return false;
    }
    // `:64765-64768`: re-arm only from a spent clock — a running duration or
    // a live cooldown keeps its value (`StartSlimeRain(announce: false)` from
    // the save/network path relies on that).
    if world.slime_rain_time <= 0.0 {
        world.slime_rain_time = rng.next_range(32400, 54000) as f64;
    }
    world.slime_rain = true;
    world.slime_rain_kills = 0;
    world.slime_warning_time = SLIME_WARNING_DELAY;
    true
}

/// `Main.cs:65121-65134`'s midnight-crossing arm: the instantaneous
/// `num4 < 16200 && time >= 16200` crossing while raining, the
/// `!downedDeerclops || Next(4) == 0` repeat roll (`:65123` — the first
/// encounter is guaranteed and draws nothing; afterwards one `Next(4)`
/// draw, 1-in-4; the pre-P2 arm had this polarity flipped and now resolves
/// toward the C#), and the player gates (above the surface, ZoneSnow, no
/// nearby town NPCs, `statLifeMax2 >= 200 || statDefense >= 9`). The
/// `CanSpawnEnemiesNear` and `AnyDanger` arms fold (no danger scan
/// ported). The world/player/zone inputs are discrete params so the
/// caller can borrow the `UpdateTime` stream alongside them.
pub(crate) fn deerclops_roll(
    world: &tr_core::World,
    player: Option<&crate::player::Player>,
    zone_snow: bool,
    rng: &mut UnifiedRandom,
) -> bool {
    if world.downed_deerclops && rng.next_max(4) != 0 {
        return false;
    }
    let Some(p) = player else {
        return false;
    };
    if p.dead
        || p.y >= (world.world_surface * 16.0) as f32
        || !zone_snow
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

/// `Main.cs:65024-65034`'s day-rate roll shape: `num6 = (int)(27000 /
/// dayRate)` and `Main.rand.Next(num6 * 4) == 0` — one `Next(num6*4)` draw
/// per tick over the first half of the day.
pub(crate) fn travel_visit_roll(world: &tr_core::World, rng: &mut UnifiedRandom) -> bool {
    let num6 = (27000.0 / (world.day_rate().max(1) as f64)) as i32;
    rng.next_max(num6 * 4) == 0
}

/// `WorldGen.SpawnTravelNPC` (`WorldGen.cs:5251-5407`): the merchant's
/// arrival — the gates, the stock regeneration, the home pick and the
/// surface scan.
pub(crate) fn spawn_travel_npc(
    world: &mut tr_core::World,
    npcs: &mut Vec<crate::npc::Npc>,
    rng: &mut UnifiedRandom,
    player: Option<&crate::player::Player>,
    chat: &mut crate::chat::ChatMonitor,
) {
    // `:5253-5256`'s gates: the eclipse (not ported), the daylight, and an
    // invasion that has actually started (`invasionDelay == 0 &&
    // invasionSize > 0`).
    if !world.day_time
        || (world.invasion_type > 0 && world.invasion_delay == 0 && world.invasion_size > 0)
    {
        return;
    }
    // `:5257-5263`'s single-merchant early return.
    if npcs
        .iter()
        .any(|n| n.kind == crate::npc::NpcKind::TravellingMerchant)
    {
        return;
    }
    // `:5264-5265`: the stock roll + the shop sync (the port has no network
    // layer for it — the World's array rides the same tick).
    let luck = player.map_or(0.0, |p| p.luck());
    world.travel_shop = crate::shop::setup_travel_shop(world, luck, rng);
    // `:5267-5279`: the anchored homes of the resident town NPCs (the port
    // carries the home on `Npc.home`).
    let mut homes: Vec<(usize, (i32, i32))> = npcs
        .iter()
        .enumerate()
        .filter(|(_, n)| n.kind.is_town_resident() && n.home.is_some())
        .map(|(i, n)| (i, n.home.unwrap_or((0, 0))))
        .collect();
    // `:5280-5296`: with no resident housed, the homeless town NPCs that own
    // a room anywhere (the port's `quick_find_home` scan).
    if homes.is_empty() {
        for (i, n) in npcs.iter().enumerate() {
            if !n.kind.is_town_resident() {
                continue;
            }
            let tx = (n.x / tr_core::TILE_PX) as i32;
            let ty = (n.y / tr_core::TILE_PX) as i32;
            // `ScoreRoom`'s `ignoreNPC` — the asking NPC is not an occupant.
            if let Some(h) =
                crate::town::quick_find_home(world, npcs, tx, ty, Some(i), n.kind.net_id(), false)
            {
                homes.push((i, h));
            }
        }
    }
    if homes.is_empty() {
        return;
    }
    // `Main.rand.Next(num)` over the anchored homes (`WorldGen.cs:5298`).
    let pick = rng.next_max(homes.len() as i32) as usize;
    let (_, (home_x, home_y)) = homes[pick];
    let mut best_x = home_x;
    let best_y = home_y;
    // `:5310-5314`: walk left from the home while the column is solid.
    let solid = |x: i32, y: i32| {
        world
            .tile(x, y)
            .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind))
    };
    let mut min_x = best_x;
    let mut x = best_x;
    while x > best_x - 10
        && (solid(x, best_y) || solid(x, best_y - 1) || solid(x, best_y - 2) || solid(x, best_y - 3))
    {
        min_x = x;
        x -= 1;
    }
    // `:5315-5318`: the same walk right.
    let mut max_x = best_x;
    let mut x = best_x;
    while x < best_x + 10
        && (solid(x, best_y) || solid(x, best_y - 1) || solid(x, best_y - 2) || solid(x, best_y - 3))
    {
        max_x = x;
        x += 1;
    }
    // `:5319-5335`: the 30-attempt pick inside that span — the first 20
    // attempts reject the ±1 band around the home tile, the last 10 accept
    // anything but the exact home tile.
    for attempt in 0..30 {
        let roll = rng.next_range(min_x, max_x + 1);
        if attempt < 20 {
            if roll < best_x - 1 || roll > best_x + 1 {
                best_x = roll;
                break;
            }
        } else if roll != best_x {
            best_x = roll;
            break;
        }
    }
    // `:5336-5392`: the surface scan — only when the home sits above the
    // surface line; the scan steps outward from the home row and picks the
    // first clear spot with no player in the box.
    let mut spawn_x = best_x;
    let mut spawn_y = best_y;
    let mut found = false;
    if !found && (spawn_y as f64) <= world.world_surface {
        for n in 20..500 {
            for side in 0..2 {
                let cand_x = if side != 0 {
                    best_x - n * 2
                } else {
                    best_x + n * 2
                };
                if cand_x <= 10 || cand_x >= world.width as i32 - 10 {
                    continue;
                }
                let top = (best_y - n).max(10);
                let bottom = ((best_y + n) as f64).min(world.world_surface) as i32;
                let mut y = top;
                while y < bottom {
                    spawn_y = y;
                    if !solid(cand_x, y) {
                        y += 1;
                        continue;
                    }
                    let liquid = (0..3).any(|k| {
                        world
                            .tile(cand_x, y - 1 - k)
                            .is_some_and(|t| t.liquid > 0)
                    });
                    if liquid {
                        break;
                    }
                    found = true;
                    spawn_x = cand_x;
                    break;
                }
                if found {
                    break;
                }
            }
            if found {
                break;
            }
        }
    }
    if !found {
        spawn_y = best_y;
    }
    // `:5393-5405`: the spawn, the home tiles, `homeless` (the port keeps
    // the home tile — it has no per-NPC homeless flag) and the facing.
    let direction = if spawn_x < best_x {
        1
    } else if spawn_x > best_x {
        -1
    } else {
        1
    };
    let mut m = crate::npc::Npc::new(
        crate::npc::NpcKind::TravellingMerchant,
        spawn_x as f32 * tr_core::TILE_PX,
        spawn_y as f32 * tr_core::TILE_PX,
        direction,
        0.0,
    );
    m.hp = m.max_hp;
    m.home = Some((best_x, best_y));
    npcs.push(m);
    // `:5406`: the `Announcement.HasArrived` broadcast (the literal stands in
    // for the localization key).
    crate::chat::broadcast_death_text(
        chat,
        "The Travelling Merchant has arrived!",
        crate::chat::COLOR_COMMAND,
    );
}

/// `WorldGen.UnspawnTravelNPC` (`WorldGen.cs:5000-5034`): the merchant leaves
/// at night once no player stands in his box — the safe-range rect around his
/// centre tile.
pub(crate) fn unspawn_travel_npc(
    npcs: &mut Vec<crate::npc::Npc>,
    player: Option<&crate::player::Player>,
    chat: &mut crate::chat::ChatMonitor,
) {
    let Some(idx) = npcs
        .iter()
        .position(|n| n.kind == crate::npc::NpcKind::TravellingMerchant)
    else {
        return;
    };
    let n = &npcs[idx];
    let (cx, cy) = n.center();
    let tx = (cx / tr_core::TILE_PX) as i32;
    let ty = (cy / tr_core::TILE_PX) as i32;
    // `NPC.safeRangeX/Y` (`NPC.cs:935-936`) over the 1920x1200 screen box
    // (`NPC.cs:6767-6769`): 62 x 39 tiles.
    const SAFE_X: i32 = 62;
    const SAFE_Y: i32 = 39;
    let left = (tx * 16 + 8 - 960 - SAFE_X) as f32;
    let top = (ty * 16 + 8 - 600 - SAFE_Y) as f32;
    let width = (1920 + SAFE_X * 2) as f32;
    let height = (1200 + SAFE_Y * 2) as f32;
    let player_near = player.is_some_and(|p| {
        p.x < left + width
            && p.x + crate::player::WIDTH > left
            && p.y < top + height
            && p.y + crate::player::HEIGHT > top
    });
    if player_near {
        return;
    }
    npcs.remove(idx);
    // `:5029`'s `Lang.misc[35]` "has left" broadcast.
    crate::chat::broadcast_death_text(
        chat,
        "The Travelling Merchant has left!",
        crate::chat::COLOR_COMMAND,
    );
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
            0.0,
            1.0,
            0,
            true,
        );
        assert_eq!(app.rain.max_raining(), 0.8);
        app.wind.speed_current = 0.55;
        app.wind.speed_target = 0.8;
        // No windCounter walk: only the drift moves current — the freeze
        // POWER is the source now (the tick copies it into `wind.frozen`).
        app.creative_powers.set(
            crate::creative::PowerId::FreezeWind,
            crate::creative::PowerState::Toggle(true),
        );
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
            0.0,
            1.0,
            0,
            true,
        );
        app.wind.speed_current = 0.55;
        app.wind.speed_target = 0.8;
        app.creative_powers.set(
            crate::creative::PowerId::FreezeWind,
            crate::creative::PowerState::Toggle(true),
        );
        arm_sandstorm(&mut app, 10_000);
        update_world_tick(&mut app, STEP);
        assert!(app.wind.speed_current < 0.6);
        assert_eq!(app.sandstorm.time_left, 10_000 - 1 - 15);
    }


    /// `Main.cs:65024-65034`'s day roll shape: `num6 = (int)(27000/dayRate)`
    /// and `Main.rand.Next(num6 * 4) == 0` — the fast-forward (dayRate 60)
    /// puts one hit per 1800 ticks; the seed-42 stream's 20000 draws land 14
    /// hits (the splitmix seed hunt re-pinned to the named-stream prefix,
    /// S1-A-11 P2). The afternoon refuses through the caller's gate (the
    /// DW-2.1 tick pins pin that arm).
    #[test]
    fn the_travel_merchant_roll_is_day_rate_scaled() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world.fast_forward_to_dawn = true; // dayRate 60 -> num6 = 450
        let mut rng = UnifiedRandom::new(42);
        let mut hits = 0;
        for _ in 0..20_000 {
            if super::travel_visit_roll(&app.world, &mut rng) {
                hits += 1;
            }
        }
        assert_eq!(hits, 14, "the Next(1800) hit count on the seed-42 prefix");
        let mut r = UnifiedRandom::new(5);
        app.world.fast_forward_to_dawn = false;
        assert!(
            super::travel_visit_roll(&app.world, &mut r) || true,
            "the dayRate-1 roll never panics"
        );
        // The afternoon gate: the arm's `world.time < 27000` refuses — the
        // caller reads it, so the shape is asserted through the tick below.
    }

    /// `WorldGen.SpawnTravelNPC` (`WorldGen.cs:5251-5407`): the spawn needs a
    /// housed town-NPC anchor, regenerates the 40-slot stock and lands the
    /// merchant homeless-but-anchored with the arrival announcement; a second
    /// call is a no-op and the all-day gate holds.
    #[test]
    fn the_travelling_merchant_spawns_with_his_stock() {
        let mut app = flow_app(None, vec![], vec![]);
        let mut first = crate::npc::Npc::new(crate::npc::NpcKind::Guide, 64.0, 64.0, 1, 0.0);
        first.home = Some((4, 4));
        let mut second = crate::npc::Npc::new(crate::npc::NpcKind::Merchant, 80.0, 64.0, 1, 0.0);
        second.home = Some((5, 4));
        app.npcs.push(first);
        app.npcs.push(second);
        let mut rng = UnifiedRandom::new(12345);
        super::spawn_travel_npc(
            &mut app.world,
            &mut app.npcs,
            &mut rng,
            app.player.as_ref(),
            &mut app.chat,
        );
        assert!(
            app.npcs
                .iter()
                .any(|n| n.kind == crate::npc::NpcKind::TravellingMerchant),
            "the merchant arrives"
        );
        // The stock: the generated 40 slots (a non-zero run, no duplicates).
        let stock: Vec<u16> = app.world.travel_shop.iter().copied().filter(|v| *v != 0).collect();
        assert!(!stock.is_empty(), "the stock is generated");
        for (i, id) in stock.iter().enumerate() {
            assert!(!stock[i + 1..].contains(id), "no duplicate {id}");
        }
        // The arrival announcement.
        assert!(
            app.chat
                .messages
                .iter()
                .any(|m| m.text.contains("Travelling Merchant has arrived")),
            "the Announcement.HasArrived broadcast"
        );
        // A second call is the vanilla's 368-alive early return.
        let count = app.npcs.len();
        super::spawn_travel_npc(
            &mut app.world,
            &mut app.npcs,
            &mut rng,
            app.player.as_ref(),
            &mut app.chat,
        );
        assert_eq!(app.npcs.len(), count, "no second merchant");
    }

    /// `WorldGen.cs:5253`'s daylight gate and `Main.cs:65012`'s departure:
    /// the night/late-day call arms `UnspawnTravelNPC` (`WorldGen.cs:5000`),
    /// which keeps the merchant while a player stands in his safe-range rect
    /// and drops him once they walk away.
    #[test]
    fn the_travelling_merchant_leaves_at_night_when_clear() {
        let mut app = flow_app(None, vec![], vec![]);
        let mut m = crate::npc::Npc::new(
            crate::npc::NpcKind::TravellingMerchant,
            64.0,
            64.0,
            1,
            0.0,
        );
        m.home = Some((4, 4));
        app.npcs.push(m);
        // A player standing on the merchant: the safe-range rect (62 x 39
        // tiles around his centre tile) keeps him.
        app.player = Some(crate::player::Player::spawn(&app.world));
        super::unspawn_travel_npc(&mut app.npcs, app.player.as_ref(), &mut app.chat);
        assert_eq!(app.npcs.len(), 1, "the nearby player keeps him");
        // Walk the player far away and the merchant leaves.
        if let Some(p) = app.player.as_mut() {
            p.x = 5000.0;
            p.y = 5000.0;
        }
        super::unspawn_travel_npc(&mut app.npcs, app.player.as_ref(), &mut app.chat);
        assert!(app.npcs.is_empty(), "the merchant left");
        assert!(
            app.chat
                .messages
                .iter()
                .any(|m| m.text.contains("Travelling Merchant has left")),
            "the Lang.misc[35] broadcast"
        );
        // The whole per-tick arm: the night + the far player drops him
        // through `update_world_tick` (`Main.cs:65012-65015`).
        let mut m = crate::npc::Npc::new(
            crate::npc::NpcKind::TravellingMerchant,
            64.0,
            64.0,
            1,
            0.0,
        );
        app.npcs.push(m);
        app.world.day_time = false;
        app.world.time = 5000.0;
        app.chat.messages.clear();
        update_world_tick(&mut app, STEP);
        assert!(
            app.npcs
                .iter()
                .all(|n| n.kind != crate::npc::NpcKind::TravellingMerchant),
            "the night tick departs him"
        );
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
        // The roll shape on the named stream (`Main.cs:64945`'s
        // `Next(num3) == 0`; the King downed + classic = num3 450000 at
        // dayRate 1 — the `/2` arm is the NOT-downed one, `Main.cs:64936`):
        // the seed-42 stream pins the first hit and every draw before it
        // (the splitmix seed hunt is gone, S1-A-11 P2).
        app.world.downed_slime_king = true;
        app.world.hard_mode = false;
        let mut rng = UnifiedRandom::new(42);
        let fired_at = (0..500_000usize)
            .find(|_| crate::world_sim::weather::slime_rain_roll(&app.world, true, &mut rng))
            .expect("the stream hits within 500000 draws");
        assert_eq!(fired_at, 115_297, "the first Next(450000) hit on seed 42");
        // The not-ready arm consumes no draw — the C#'s `(flag3 ||
        // expertMode)` gate short-circuits before `Next(num3)`
        // (`Main.cs:64945`), so the classic not-ready never rolls.
        let mut fresh = UnifiedRandom::new(42);
        let pristine = fresh.clone();
        assert!(!crate::world_sim::weather::slime_rain_roll(
            &app.world, false, &mut fresh
        ));
        assert_eq!(fresh, pristine, "the not-ready gate draws nothing");
        // The start fields: one `Next(32400, 54000)` duration draw
        // (`Main.cs:64767`) and the kill counter reset.
        assert!(crate::world_sim::weather::start_slime_rain(
            &mut app.world,
            false,
            &mut fresh
        ));
        assert!(app.world.slime_rain);
        assert_eq!(
            app.world.slime_rain_time, 46831.0,
            "the Next(32400, 54000) duration on seed 42"
        );
        assert_eq!(app.world.slime_rain_kills, 0);
        // The announcement armed (`Main.cs:64776`): the 420-tick countdown.
        assert_eq!(app.world.slime_warning_time, 420);
        // The gates: the afternoon tick leaves the state untouched.
        app.world.slime_rain = false;
        app.world.time = 30000.0;
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
        // The base life refuses (the player gates run after the roll arm).
        app.player.as_mut().unwrap().max_hp = 100;
        app.player.as_mut().unwrap().stat_defense = 0;
        let mut rng = UnifiedRandom::new(42);
        assert!(!crate::world_sim::weather::deerclops_roll(
            &app.world,
            app.player.as_ref(),
            true,
            &mut rng,
        ));
        // The guaranteed first: `!downedDeerclops` passes `Main.cs:65123`'s
        // leftmost `||` operand with NO draw — twin equality pins it (the
        // pre-P2 arm had this polarity flipped; resolved toward the C#).
        app.player.as_mut().unwrap().max_hp = 200;
        app.scene_metrics.zone_snow = true;
        let mut fresh = UnifiedRandom::new(42);
        let pristine = fresh.clone();
        assert!(crate::world_sim::weather::deerclops_roll(
            &app.world,
            app.player.as_ref(),
            app.scene_metrics.zone_snow,
            &mut fresh,
        ));
        assert_eq!(fresh, pristine, "the first encounter draws nothing");
        // The repeat: one `Next(4)` draw, 1-in-4 — the seed-42 stream hits
        // on its second draw (the first misses).
        app.world.downed_deerclops = true;
        assert!(!crate::world_sim::weather::deerclops_roll(
            &app.world,
            app.player.as_ref(),
            app.scene_metrics.zone_snow,
            &mut fresh,
        ));
        assert!(crate::world_sim::weather::deerclops_roll(
            &app.world,
            app.player.as_ref(),
            app.scene_metrics.zone_snow,
            &mut fresh,
        ));
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
    fn the_world_tick_writes_any_player_120_life_into_wind() {
        // `Main.cs:58863-58871`: the target-walk scan writes the
        // `any_player_120_life` flag the 0.35 cap reads (`Wind::update_rain`,
        // S1-L-2). The port folds the 255-player scan to its single player.
        let mut app = flow_app(None, vec![], vec![]);
        app.world.world_surface = 200.0;
        // Observe the flag, not the walk — the freeze power is the source.
        app.creative_powers.set(
            crate::creative::PowerId::FreezeWind,
            crate::creative::PowerState::Toggle(true),
        );
        app.wind.any_player_120_life = true;
        let mut low = crate::player::Player::spawn(&app.world);
        low.max_hp = 100;
        app.player = Some(low);
        update_world_tick(&mut app, STEP);
        assert!(
            !app.wind.any_player_120_life,
            "a <120-HP player clears the strong-wind flag"
        );
        let mut high = crate::player::Player::spawn(&app.world);
        high.max_hp = 120;
        app.player = Some(high);
        update_world_tick(&mut app, STEP);
        assert!(
            app.wind.any_player_120_life,
            "statLifeMax >= 120 arms the strong-wind flag"
        );
    }

    #[test]
    fn the_meteor_roll_and_boss_death_flag() {
        // The night roll (`Main.cs:65230`) gates on the boss-2 downing;
        // the orb-boss death arms the flag (the guaranteed first, the
        // `NPC.cs:79450-79456` arm).
        let mut app = flow_app(None, vec![], vec![]);
        app.world.downed_eow = false;
        // `Main.cs:65230`: the `Next(50)` draw happens before the boss-flag
        // read — a gated-off call still consumes exactly one draw.
        let mut rng = UnifiedRandom::new(42);
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
        assert_eq!(hits, 47, "the 1/50 rate over the seed-42 prefix");
        // The orb-boss death arm (`NPC.cs:79450-79456`): the guaranteed
        // first with no draw, then `Next(2)` — seed 42's first two draws
        // miss (1) then hit (0).
        let mut rng2 = UnifiedRandom::new(42);
        assert!(tr_core::worldgen::meteor::boss_death_roll(&mut rng2, false));
        assert!(!tr_core::worldgen::meteor::boss_death_roll(&mut rng2, true));
        assert!(tr_core::worldgen::meteor::boss_death_roll(&mut rng2, true));
    }

    /// DW-2.1: one weather tick consumes EXACTLY the weather rolls' draws
    /// from the `UpdateTime` stream — the slime-rain roll (`Main.cs:64945`,
    /// one `Next(num3)`) and the travel visit roll (`Main.cs:65025-65034`,
    /// one `Next(num6 * 4)`), in that order. Twin-stream equality proves the
    /// count and the order (no more, no fewer draws); the C# int pipelines
    /// are mirrored by hand so a shape drift breaks the equality. The
    /// crossings (deerclops, meteor night) do not fire on this tick and
    /// draw nothing.
    #[test]
    fn test_dw_2_1_weather_tick_pins_the_update_time_stream_prefix() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world.day_time = true;
        app.world.time = 10000.0;
        let mut player = crate::player::Player::spawn(&app.world);
        player.max_hp = 200; // AnyPlayerReadyToFightKingSlime: lifeMax > 140
        player.stat_defense = 10; // && statDefense > 8 — the roll draws
        app.player = Some(player);
        update_world_tick(&mut app, STEP);

        // The C# draw shapes, mirrored by hand: `num3`
        // (`Main.cs:64933-64943`) = (int)(450000.00000000006/dayRate), /2
        // (no King downed), the hardmode ELSE-if not taken, the ready
        // player so no x5; `num6` (`Main.cs:65028-65033`) =
        // (int)(27000/dayRate), `Next(num6 * 4)`.
        let day_rate = app.world.day_rate().max(1);
        let mut num3 = (450000.00000000006f64 / day_rate as f64) as i32;
        num3 /= 2;
        let num6 = (27000.0f64 / day_rate as f64) as i32;
        let mut twin = UnifiedRandom::new(app.world.seed as i32);
        let slime_fired = twin.next_max(num3) == 0;
        let _travel_fired = twin.next_max(num6 * 4) == 0;
        assert_eq!(
            *app.rngs.stream(crate::app::RngKey::UpdateTime),
            twin,
            "the tick consumed exactly Next(num3) + Next(num6*4) from UpdateTime"
        );
        assert_eq!(
            app.world.slime_rain, slime_fired,
            "the roll decided the slime rain"
        );
    }

    /// DW-2.1 (boundary): an afternoon tick consumes NOTHING from
    /// `UpdateTime` — the slime roll's `time < 27000` gate and the travel
    /// roll's day gate both refuse, no crossing fires, and the slime clock
    /// sits at zero.
    #[test]
    fn test_dw_2_1_afternoon_tick_consumes_no_update_time_draws() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world.day_time = true;
        app.world.time = 30000.0;
        let pristine = UnifiedRandom::new(app.world.seed as i32);
        update_world_tick(&mut app, STEP);
        assert_eq!(
            *app.rngs.stream(crate::app::RngKey::UpdateTime),
            pristine,
            "the afternoon tick drew nothing"
        );
    }

    /// DW-2.2: the meteor resolution rides `UpdateTime` — with the flag up,
    /// the tick's `HandleMeteorFall` arm draws `drop_meteor` + `meteor`
    /// from the named stream (the stream advanced; every other stage
    /// stream stays pristine), the flag clears and the ore lands.
    #[test]
    fn test_dw_2_2_meteor_resolve_rides_the_update_time_stream() {
        let mut app = flow_app(None, vec![], vec![]);
        let mut big = tr_core::World::new("t".into(), 400, 400);
        for x in 0..400 {
            for y in 120..400 {
                let i = big.index(x, y);
                big.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                big.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        big.compute_column_surface();
        big.time = 17000.0; // > 16200: the resolve window
        big.spawn_meteor = true;
        app.world = big;
        let pristine = UnifiedRandom::new(app.world.seed as i32);
        update_world_tick(&mut app, STEP);
        assert!(!app.world.spawn_meteor, "the flag cleared");
        let ore = app
            .world
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == tr_core::worldgen::meteor::METEORITE)
            .count();
        assert!(ore > 100, "the ore landed ({ore})");
        assert_ne!(
            *app.rngs.stream(crate::app::RngKey::UpdateTime),
            pristine,
            "the resolution drew UpdateTime"
        );
        // Every OTHER stage stream is untouched (the binding choice).
        for key in crate::app::RngKey::ALL {
            if key == crate::app::RngKey::UpdateTime {
                continue;
            }
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(app.world.seed as i32),
                "{} stayed pristine",
                key.name()
            );
        }
    }

    /// DW-2.3: two same-seed runs of `update_world_tick` produce identical
    /// worlds and identical stream states — the weather group's determinism
    /// pair.
    #[test]
    fn test_dw_2_3_weather_tick_determinism_pair() {
        let mut a = flow_app(None, vec![], vec![]);
        let mut b = flow_app(None, vec![], vec![]);
        for app in [&mut a, &mut b] {
            app.world.day_time = true;
            app.world.time = 10000.0;
            let mut player = crate::player::Player::spawn(&app.world);
            player.max_hp = 200;
            player.stat_defense = 10;
            app.player = Some(player);
        }
        update_world_tick(&mut a, STEP);
        update_world_tick(&mut b, STEP);
        assert_eq!(a.world.tiles, b.world.tiles, "tile grids diverged");
        assert_eq!(
            (a.world.world_surface, a.world.rock_layer),
            (b.world.world_surface, b.world.rock_layer)
        );
        assert_eq!(
            (
                a.world.slime_rain,
                a.world.slime_rain_time,
                a.world.slime_rain_kills
            ),
            (
                b.world.slime_rain,
                b.world.slime_rain_time,
                b.world.slime_rain_kills
            ),
            "the slime-rain state matches"
        );
        assert_eq!(
            *a.rngs.stream(crate::app::RngKey::UpdateTime),
            *b.rngs.stream(crate::app::RngKey::UpdateTime),
            "the UpdateTime streams match"
        );
        // And every other stage stream matches across the pair.
        for key in crate::app::RngKey::ALL {
            assert_eq!(
                *a.rngs.stream(key),
                *b.rngs.stream(key),
                "{} matches across the pair",
                key.name()
            );
        }
    }

    /// The batch-review blocker's edge case, now executed:
    /// `Main.StopSlimeRain`'s SP arm (`Main.cs:64787-64820`) sets the clock
    /// to `-Next(3024, 6048) * 100` (`:64805`) — ONE integer draw, negated
    /// and scaled. Twin equality pins the draw shape (the stream consumed
    /// exactly one `Next(3024, 6048)`), the exact seed-42 value pins the
    /// ×100 arithmetic, and the already-down no-op arm (`:64789-64792`)
    /// consumes nothing.
    #[test]
    fn the_stop_slime_rain_cooldown_is_one_next3024_6048_draw() {
        let mut world = tr_core::World::new("t".into(), 8, 8);
        world.slime_rain = true;
        world.slime_rain_time = 100.0; // a positive (raining) clock
        let mut rng = UnifiedRandom::new(42);
        crate::world_sim::weather::stop_slime_rain(&mut world, &mut rng);
        // The twin: exactly one `Next(3024, 6048)` draw, ×100 negative.
        let mut twin = UnifiedRandom::new(42);
        let want = -(twin.next_range(3024, 6048) as f64) * 100.0;
        assert_eq!(
            rng, twin,
            "the cooldown consumed exactly one Next(3024, 6048) draw"
        );
        assert!(!world.slime_rain, "the rain is down");
        assert_eq!(
            world.slime_rain_time, want,
            "the -Next(3024,6048)*100 clock"
        );
        assert_eq!(want, -504_400.0, "the seed-42 draw is 5044");
        // The already-down no-op arm (`Main.cs:64789-64792`) draws nothing
        // and leaves the cooldown alone.
        let pristine = rng.clone();
        crate::world_sim::weather::stop_slime_rain(&mut world, &mut rng);
        assert_eq!(rng, pristine, "the !slime_rain guard draws nothing");
        assert_eq!(world.slime_rain_time, want, "the cooldown untouched");
        // The announcement arms either way (`:64811`).
        assert_eq!(world.slime_warning_time, super::SLIME_WARNING_DELAY);
    }

    /// S1-E-37: `StartSlimeRain`'s own gates (`Main.cs:64744-64768`) — the
    /// surface, the rain, the already-raining arm and the `slimeRainTime <= 0`
    /// re-arm. The panel used to hardcode 43200 with none of these.
    #[test]
    fn s1_e_37_start_slime_rain_gates_and_rearm() {
        let mut world = tr_core::World::new("t".into(), 64, 64);
        let mut rng = UnifiedRandom::new(7);
        // `!Main.isThereAWorldSurface` (`:64744`).
        world.world_surface = 0.0;
        assert!(!super::start_slime_rain(&mut world, false, &mut rng));
        assert!(!world.slime_rain);
        // `Main.raining` (`:64761-64764`).
        world.world_surface = 200.0;
        assert!(!super::start_slime_rain(&mut world, true, &mut rng));
        assert!(!world.slime_rain);
        // The real start: one `Next(32400, 54000)` draw.
        let mut twin = UnifiedRandom::new(7);
        let want = twin.next_range(32400, 54000) as f64;
        assert!(super::start_slime_rain(&mut world, false, &mut rng));
        assert_eq!(world.slime_rain_time, want);
        assert!(want >= 32400.0 && want < 54000.0, "the band: {want}");
        // Already raining (`:64744`) — no draw, no change.
        let pristine = rng.clone();
        assert!(!super::start_slime_rain(&mut world, false, &mut rng));
        assert_eq!(rng, pristine, "the slimeRain guard draws nothing");
        // A live duration survives the call (`:64765-64768` re-arms only a
        // spent clock) — the `announce: false` save/network path relies on it.
        world.slime_rain = false;
        world.slime_rain_time = 40000.0;
        assert!(super::start_slime_rain(&mut world, false, &mut rng));
        assert_eq!(
            world.slime_rain_time, 40000.0,
            "a positive clock is not re-rolled"
        );
    }

    /// S1-E-37: `Main.UpdateSlimeRainWarning` (`Main.cs:65504-65522`) — the
    /// 420-tick countdown, then `Lang.gen[74]` while the clock is positive and
    /// `Lang.gen[75]` once it has gone negative.
    #[test]
    fn s1_e_37_the_slime_warning_counts_down_and_picks_the_text() {
        let mut world = tr_core::World::new("t".into(), 64, 64);
        assert_eq!(super::update_slime_rain_warning(&mut world), None, "idle");
        world.slime_warning_time = 2;
        world.slime_rain_time = 30000.0;
        assert_eq!(super::update_slime_rain_warning(&mut world), None);
        assert_eq!(world.slime_warning_time, 1);
        assert_eq!(
            super::update_slime_rain_warning(&mut world),
            Some("LegacyWorldGen.74"),
            "a positive clock = 'Slime is falling from the sky!'"
        );
        assert_eq!(world.slime_warning_time, 0);
        // The stopped arm picks the other row.
        world.slime_warning_time = 1;
        world.slime_rain_time = -504_400.0;
        assert_eq!(
            super::update_slime_rain_warning(&mut world),
            Some("LegacyWorldGen.75"),
            "a negative clock = 'Slime has stopped falling from the sky.'"
        );
    }

    /// The King-death caller (`NPC.cs:79405-79410`'s case 50 through the
    /// port's kill path, npcs.rs) consumes its duration draw from the
    /// `UpdateNPCs` stream — `checkDead` runs inside `NPC.UpdateNPC` under
    /// the `UpdateNPCs` swap (`Main.cs:17767-17769`) — and the rain goes
    /// down with the same negated-×100 cooldown. Exactly one `UpdateNPCs`
    /// draw for the whole death tick (the death path's only other named
    /// consumer is the orb-boss `boss_death_roll`, not this kill).
    #[test]
    fn the_king_death_stops_the_slime_rain_on_the_update_npcs_stream() {
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
        big.slime_rain = true;
        big.slime_rain_time = 30000.0;
        app.world = big;
        let mut player = crate::player::Player::spawn(&app.world);
        player.max_hp = 200;
        app.player = Some(player);
        let mut king = crate::npc::Npc::new(
            crate::npc::NpcKind::KingSlime,
            30.0 * 16.0,
            118.0 * 16.0,
            1,
            0.0,
        );
        king.hp = 0; // dead on arrival — the kill path runs this tick
        app.npcs.push(king);

        // The twin: the death sound's case-4 pitch draw FIRST
        // (`NPC.checkDead` `NPC.cs:78193` precedes `NPCLoot`/`DoDeathEvents`
        // — one `Next(-10, 11)`, `LegacySoundPlayer.cs:693`, S1-E-21 wave C),
        // then the one `Next(3024, 6048)` draw the stop consumes.
        let mut twin = UnifiedRandom::new(app.world.seed as i32);
        let _death_pitch = twin.next_range(-10, 11);
        let want_cooldown = -(twin.next_range(3024, 6048) as f64) * 100.0;

        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);

        assert!(app.world.downed_slime_king, "the boss flag raised");
        assert!(!app.world.slime_rain, "the King's death stopped the rain");
        assert_eq!(
            app.world.slime_rain_time, want_cooldown,
            "the negated-×100 cooldown from the UpdateNPCs draw"
        );
        assert_eq!(
            want_cooldown, -549_500.0,
            "the seed-0 sequence: the death pitch draw, then 5495 (the twin replay proved the shape)"
        );
        // The value pin above already proves the draw order: the death
        // sound's one pitch draw precedes the stop's `Next(3024, 6048)`
        // (a missing/extra prior draw would shift the cooldown off the
        // twin's replay). Since P3 the same tick's NPCLoot half
        // (coins/loot) also draws `UpdateNPCs` — the stream sits past the
        // twin by the King's loot sequence, pinned per-group in DW-3.2's
        // loot test.
        assert_ne!(
            *app.rngs.stream(crate::app::RngKey::UpdateNpcs),
            twin,
            "the death tick drew past the stop (the loot sequence follows)"
        );
        // The weather stream is untouched by the kill path (the two callers'
        // streams differ exactly as the C#'s swaps make them).
        assert_eq!(
            *app.rngs.stream(crate::app::RngKey::UpdateTime),
            UnifiedRandom::new(app.world.seed as i32),
            "UpdateTime untouched by the King's death"
        );
    }
}
