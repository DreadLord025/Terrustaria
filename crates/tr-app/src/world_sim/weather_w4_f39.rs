// W4-f39 census — the `Main.UpdateTime` slime-rain slots (`Main.cs:64846-
// 64951`): the outer surface gate, the clock-before-rain order, the
// branch-entry ladder sampling, the lantern arms, the morning-roll gates
// and `StartSlimeRain`'s own gate edges. Written-not-run (code-only lane).
// Every RNG pin mirrors the draw on a cloned `UnifiedRandom` and compares
// the full stream state (the S1-A-11 witness convention).

use super::*;
use crate::app::flow::front_flow_tests::flow_app;
use crate::rain::Rain;

const STEP: f32 = 1.0 / 60.0;

/// A flow app with a real surface (`Main.cs:2788`'s `worldSurface > 50.0`)
/// and a fight-ready player — the morning roll's gates open.
fn ready_app() -> App {
    let mut app = flow_app(None, vec![], vec![]);
    app.world.world_surface = 200.0;
    app.world.day_time = true;
    app.world.time = 10000.0;
    let mut p = crate::player::Player::spawn(&app.world);
    p.max_hp = 200; // `statLifeMax > 140` (`Main.cs:65170`)
    p.stat_defense = 10; // `statDefense > 8`
    app.player = Some(p);
    app
}

#[test]
fn w4_f39_the_surface_gate_edges_are_exclusive_50() {
    // `Main.cs:64846`: `(isThereAWorldSurface || remixWorld)` —
    // `isThereAWorldSurface` is `worldSurface > 50.0` (`Main.cs:2788`),
    // EXCLUSIVE. 50.0 refuses the whole block (the clock included);
    // 50.5 opens it. The clock decay is the observable: no gate, no decay.
    for (surface, decays) in [(50.0f64, false), (50.5, true), (2.0, false)] {
        let mut app = ready_app();
        app.world.world_surface = surface;
        app.world.slime_rain = true;
        app.world.slime_rain_time = 1000.0;
        update_world_tick(&mut app, STEP);
        let want = if decays { 999.0 } else { 1000.0 };
        assert_eq!(app.world.slime_rain_time, want, "surface {surface}");
    }
    // The remix half opens the gate even without a surface (`|| remixWorld`).
    let mut app = ready_app();
    app.world.world_surface = 2.0;
    app.wind.remix_world = true;
    app.world.slime_rain = true;
    app.world.slime_rain_time = 1000.0;
    update_world_tick(&mut app, STEP);
    assert_eq!(app.world.slime_rain_time, 999.0, "remix opens the gate");
}

#[test]
fn w4_f39_the_clock_decays_before_the_rain_section_and_stops_on_update_time() {
    // `Main.cs:64848-64854`: the positive clock decays `dayRate` and the
    // `<= 0` expiry calls `StopSlimeRain` — ONE `Next(3024, 6048)` draw on
    // the `UpdateTime` stream, ahead of the travel roll (`:65025`). The
    // rain section is not running (`raining` false, the start arms miss on
    // this seed), so the stream is exactly [cooldown, travel].
    let mut app = ready_app();
    app.world.slime_rain = true;
    app.world.slime_rain_time = 1.0; // one dayRate-1 tick expires it
    let mut twin = UnifiedRandom::new(app.world.seed as i32);
    let want_cooldown = -(twin.next_range(3024, 6048) as f64) * 100.0;
    let _travel = twin.next_max((27000.0 / 1.0) as i32 * 4);
    // The homeless-arrival roll follows the visit roll on the SAME
    // `Main.rand` stream (`Main.cs:65043-65057`: `Next(maxValue)` with
    // `maxValue = (int)(27000.0 / dayRate)`).
    let _homeless = twin.next_max((27000.0 / 1.0) as i32);
    update_world_tick(&mut app, STEP);
    assert!(!app.world.slime_rain, "the expiry stopped the rain");
    assert_eq!(app.world.slime_rain_time, want_cooldown);
    assert_eq!(
        *app.rngs.stream(crate::app::RngKey::UpdateTime),
        twin,
        "the stop drew Next(3024,6048) BEFORE the travel roll"
    );
    // The kill counter survives the stop (`Main.cs:64787-64820` clears
    // only the flag/clock/warning — the counter resets in StartSlimeRain).
    assert_eq!(app.world.slime_rain_kills, 0);
}

#[test]
fn w4_f39_the_negative_cooldown_climbs_and_snaps_to_zero() {
    // `Main.cs:64856-64862`: the negative clock climbs `dayRate` per tick;
    // an overshoot past zero snaps to EXACTLY 0.0 (`:64861`) — the next
    // tick's `slimeRainTime == 0` BusyWithAnyInvasion gate then opens.
    let mut app = ready_app();
    app.world.slime_rain_time = -2.0;
    update_world_tick(&mut app, STEP); // -2 + 1 = -1, still negative
    assert_eq!(app.world.slime_rain_time, -1.0);
    update_world_tick(&mut app, STEP); // -1 + 1 = 0, the boundary
    assert_eq!(app.world.slime_rain_time, 0.0);
    // The overshoot arm: -0.5 + 1 = +0.5 → snapped to 0, NOT left positive
    // (a positive clock would decay as a running event next tick).
    app.world.slime_rain_time = -0.5;
    update_world_tick(&mut app, STEP);
    assert_eq!(app.world.slime_rain_time, 0.0, "the overshoot snaps");
    // The climb draws nothing — the clock walk is RNG-free.
    let mut app = ready_app();
    app.world.slime_rain_time = -100.0;
    let pristine = UnifiedRandom::new(app.world.seed as i32);
    let mut twin = pristine.clone();
    let _travel = twin.next_max(27000 * 4); // the travel roll
    // The homeless-arrival roll (`Main.cs:65052-65057`, `Next(27000)`) rides
    // the same stream right after it.
    let _homeless = twin.next_max(27000);
    update_world_tick(&mut app, STEP);
    assert_eq!(app.world.slime_rain_time, -99.0);
    assert_eq!(
        *app.rngs.stream(crate::app::RngKey::UpdateTime),
        twin,
        "the cooldown climb consumed no draw"
    );
}

#[test]
fn w4_f39_a_rain_ending_this_tick_does_not_open_the_slime_roll() {
    // The C# ladder (`Main.cs:64864`/`:64890`) picks its arm ONCE off the
    // branch-entry `Main.raining`: a rain that expires INSIDE
    // `Rain::update_time` (`:64880`) leaves the tick in the raining arm —
    // the start arms and the slime roll (`:64925`, inside the else-if)
    // stay closed until the next tick. A fire-seeded slime roll would
    // otherwise start the event on the very tick the rain died.
    // `day_time = false` keeps the travel roll (`:65017`) gated too, so
    // the UpdateTime stream is EXACTLY the slime-roll draw (or nothing).
    let mut app = ready_app();
    app.world.day_time = false;
    app.world.downed_slime_king = true; // num3 = 450000 at dayRate 1
    app.rain.arm_raining_for_test(1, 0.8); // expires this tick (drain 1 → 0)
    // Find a seed whose FIRST UpdateTime draw hits Next(450000) == 0 —
    // if the roll ran this tick it would fire; the ladder must refuse it.
    let fire = (1..2_000_000i32)
        .find(|&s| UnifiedRandom::new(s).next_max(450_000) == 0)
        .expect("a fire seed exists");
    app.rngs.reset(fire);
    let pristine = UnifiedRandom::new(fire);
    update_world_tick(&mut app, STEP);
    assert!(!app.rain.is_raining(), "the rain expired inside the tick");
    assert!(!app.world.slime_rain, "the roll stayed closed this tick");
    assert_eq!(
        *app.rngs.stream(crate::app::RngKey::UpdateTime),
        pristine,
        "the raining-arm tick consumed NO UpdateTime draw"
    );
    // The next tick (now entering NOT raining, and the morning sun back)
    // runs the roll — the same stream's FIRST draw is the fire hit, so the
    // event starts; the travel roll (`:65025`) follows the duration draw.
    app.world.day_time = true;
    update_world_tick(&mut app, STEP);
    assert!(app.world.slime_rain, "the next tick's roll fired");
    let mut twin = UnifiedRandom::new(fire);
    let _hit = twin.next_max(450_000);
    let _duration = twin.next_range(32400, 54000);
    let _travel = twin.next_max(27000 * 4);
    // The homeless-arrival roll (`Main.cs:65052-65057`) closes the tick's
    // `Main.rand` draws after the visit roll.
    let _homeless = twin.next_max(27000);
    assert_eq!(
        *app.rngs.stream(crate::app::RngKey::UpdateTime),
        twin,
        "the roll + the StartSlimeRain duration + the travel roll drew in order"
    );
}

#[test]
fn w4_f39_the_slime_rain_flag_gates_the_rain_start_arms() {
    // `Main.cs:64890`: `!Main.slimeRain` gates the start arms — while the
    // event is up, no rain can start (and the slime roll is closed too).
    // A fire-seeded Rain stream would start rain on a dry tick; with the
    // slime flag up the same seed draws NOTHING from the Rain stream.
    let fire = (1..2_000_000u64)
        .find(|&s| {
            let mut r = (s ^ 0x2545_F491_4F6C_DD1D).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
            r = r.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = r;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            (z ^ (z >> 31)) % 496_800 == 0
        })
        .expect("a fire seed exists");
    let mut app = ready_app();
    app.rain = Rain::with_forced(fire, None, false);
    app.world.slime_rain = true;
    app.world.slime_rain_time = 30000.0;
    let rain_state = app.rain.rng_state_for_test();
    update_world_tick(&mut app, STEP);
    assert!(!app.rain.is_raining(), "the slime flag gated the start arms");
    assert_eq!(app.rain.rng_state_for_test(), rain_state, "no start-arm draw was consumed");
}

#[test]
fn w4_f39_lanterns_up_stops_the_rain_and_gates_the_arms() {
    // `Main.cs:64868-64871`: `LanternsUp → StopRain()` — no drain, no
    // strength roll, the rain ends at once. `:64890`: `!LanternsUp &&
    // !NextNightIsLanternNight` gate the start arms and the slime roll.
    let mut app = ready_app();
    app.rain.arm_raining_for_test(5000, 0.8);
    // `manual_lanterns` (the lantern item) — `LanternNight.UpdateTime`'s
    // persistence clear only drops `genuine_lanterns` (`:64965`), so the
    // flag stays up across the tick and the gate is observable.
    app.world_events.manual_lanterns = true;
    let rain_state = app.rain.rng_state_for_test();
    update_world_tick(&mut app, STEP);
    assert!(!app.rain.is_raining(), "the lantern night stopped the rain");
    assert_eq!(app.rain.rain_time_for_test(), 0, "StopRain pins rainTime (`:64648`)");
    assert_eq!(app.rain.rng_state_for_test(), rain_state, "the stop arm draws nothing");
    // The next-night flag alone (lanterns NOT up) gates the start arms:
    // the same fire seed that would start rain stays dry.
    let fire = (1..2_000_000u64)
        .find(|&s| {
            let mut r = (s ^ 0x2545_F491_4F6C_DD1D).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
            r = r.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = r;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            (z ^ (z >> 31)) % 496_800 == 0
        })
        .expect("a fire seed exists");
    let mut app = ready_app();
    app.rain = Rain::with_forced(fire, None, false);
    app.world_events.next_night_is_lantern_night = true;
    let rain_state = app.rain.rng_state_for_test();
    update_world_tick(&mut app, STEP);
    assert!(!app.rain.is_raining(), "the next-night flag gated the arms");
    assert_eq!(app.rain.rng_state_for_test(), rain_state, "no draw under the gate");
}

#[test]
fn w4_f39_the_morning_roll_gates_are_the_busy_invasion_set() {
    // `Main.cs:64925` + `NPC.cs:7022-7029`: the roll refuses while any of
    // the BusyWithAnyInvasionOfSorts flags is up, in the afternoon, at
    // night, or with a spent/cooldown slime clock. Each gate consumes NO
    // UpdateTime draw beyond the travel roll (the roll's `&&` chain
    // short-circuits before `Next(num3)`).
    for gate in [
        "blood_moon",
        "eclipse",
        "snow_moon",
        "pumpkin_moon",
        "invasion",
        "dd2",
        "afternoon",
        "night",
        "cooldown",
    ] {
        let mut app = ready_app();
        match gate {
            "blood_moon" => app.world.blood_moon = true,
            "eclipse" => app.world.eclipse = true,
            "snow_moon" => app.world.snow_moon = true,
            "pumpkin_moon" => app.world.pumpkin_moon = true,
            "invasion" => app.world.invasion_type = 1,
            "dd2" => app.world.dd2_event_ongoing = true,
            "afternoon" => app.world.time = 30000.0,
            "night" => app.world.day_time = false,
            "cooldown" => app.world.slime_rain_time = -100.0,
            _ => unreachable!(),
        }
        let mut twin = UnifiedRandom::new(app.world.seed as i32);
        if gate != "afternoon" && gate != "night" {
            let _travel = twin.next_max(27000 * 4);
            // The homeless-arrival roll shares the day window and stream
            // (`Main.cs:65043-65057`, `Next(27000)`).
            let _homeless = twin.next_max(27000);
        }
        update_world_tick(&mut app, STEP);
        assert!(!app.world.slime_rain, "{gate} refused the roll");
        assert_eq!(
            *app.rngs.stream(crate::app::RngKey::UpdateTime),
            twin,
            "{gate}: no slime draw beyond the travel roll"
        );
    }
}

#[test]
fn w4_f39_the_ready_gate_edges_are_exclusive_and_the_not_ready_arm_draws_nothing() {
    // `Main.cs:65170`: `statLifeMax > 140 && statDefense > 8` — BOTH
    // exclusive. 140/8 refuses, 141/9 arms. The classic not-ready arm
    // (`(flag3 || expertMode)`, `:64945`) short-circuits before the draw.
    for (hp, def, ready) in [(140i32, 9i32, false), (141, 8, false), (141, 9, true)] {
        let mut app = ready_app();
        app.player.as_mut().unwrap().max_hp = hp;
        app.player.as_mut().unwrap().stat_defense = def;
        app.world.downed_slime_king = true;
        let mut twin = UnifiedRandom::new(app.world.seed as i32);
        // `Main.cs:64945`'s `Next(num3) == 0` draws only when ready (the
        // `flag3 || expertMode` gate); a hit then `StartSlimeRain`'s
        // `Next(32400, 54000)` (`:64767`).
        if ready && twin.next_max(450_000) == 0 {
            let _duration = twin.next_range(32400, 54000);
        }
        let _travel = twin.next_max(27000 * 4);
        // The homeless-arrival roll (`Main.cs:65052-65057`).
        let _homeless = twin.next_max(27000);
        update_world_tick(&mut app, STEP);
        assert_eq!(
            *app.rngs.stream(crate::app::RngKey::UpdateTime),
            twin,
            "hp {hp} def {def}: ready={ready}"
        );
    }
}

#[test]
fn w4_f39_start_slime_rain_refuses_remix_and_the_50_surface_edge() {
    // `Main.cs:64744`: `remixWorld || !isThereAWorldSurface || slimeRain`
    // — the remix seed refuses outright (W4-f39: the port HAS the seed
    // flag now), and the surface edge is `> 50.0` (`Main.cs:2788`).
    let mut world = tr_core::World::new("t".into(), 64, 64);
    world.world_surface = 200.0;
    world.seeds.insert(tr_core::seeds::SeedFlags::REMIX);
    let mut rng = UnifiedRandom::new(7);
    let pristine = rng.clone();
    assert!(!super::start_slime_rain(&mut world, false, &mut rng));
    assert_eq!(rng, pristine, "the remix refusal draws nothing");
    world.seeds = tr_core::seeds::SeedFlags::none();
    world.world_surface = 50.0; // the exclusive edge
    assert!(!super::start_slime_rain(&mut world, false, &mut rng));
    assert_eq!(rng, pristine, "the 50.0 edge draws nothing");
    world.world_surface = 50.5;
    assert!(super::start_slime_rain(&mut world, false, &mut rng));
    assert!(world.slime_rain);
}
