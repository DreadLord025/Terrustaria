// W4-d129 census — the `any_player_120_life` writer wire
// (`weather.rs::update_world_tick`, `Main.cs:58863-58871` / `:58889-58892`,
// S1-L-2). Written-not-run (code-only lane): the >=120 boundary, the
// no-player fold, the per-frame (not per-iteration) write position and the
// 0.35 walk cap the flag feeds. The flag write itself draws no randomness.

use super::*;
use crate::app::flow::front_flow_tests::flow_app;

const STEP: f32 = 1.0 / 60.0;

#[test]
fn w4_d129_the_writer_boundary_is_stat_life_max_ge_120() {
    // `Main.cs:58866`: `Main.player[i].active && statLifeMax >= 120` — the
    // port folds the 255-player scan to its single player
    // (`weather.rs:135`, `p.max_hp >= 120`). 119 refuses, 120 arms.
    let mut app = flow_app(None, vec![], vec![]);
    app.wind.frozen = true; // observe the flag, not the walk
    for (hp, expected) in [(100i32, false), (119, false), (120, true), (400, true)] {
        let mut p = crate::player::Player::spawn(&app.world);
        p.max_hp = hp;
        app.player = Some(p);
        update_world_tick(&mut app, STEP);
        assert_eq!(app.wind.any_player_120_life, expected, "max_hp {hp}");
    }
    // No active player: the C# scan finds none (`is_some_and` → false).
    app.player = None;
    update_world_tick(&mut app, STEP);
    assert!(!app.wind.any_player_120_life);
}

#[test]
fn w4_d129_the_flag_write_is_per_frame_and_rng_free() {
    // The writer sits AHEAD of the FixedStepper tick (`weather.rs:135-136`
    // before `world_tick_step.tick`): even a zero-dt frame (no 60 Hz step
    // fires) refreshes the flag, and the write itself consumes no wind
    // draws — with the walk frozen the counter/target pair is untouched.
    let mut app = flow_app(None, vec![], vec![]);
    app.wind.frozen = true;
    app.wind.any_player_120_life = true;
    let before = (app.wind.wind_counter(), app.wind.speed_target.to_bits());
    let mut p = crate::player::Player::spawn(&app.world);
    p.max_hp = 100;
    app.player = Some(p);
    update_world_tick(&mut app, 0.0); // no step fires at dt 0
    assert!(!app.wind.any_player_120_life, "the write runs without a step");
    assert_eq!(
        (app.wind.wind_counter(), app.wind.speed_target.to_bits()),
        before,
        "the flag write draws nothing"
    );
}

#[test]
fn w4_d129_the_035_cap_reads_the_flag_after_the_walk() {
    // `Main.cs:58889-58892`: `if (!flag && |windSpeedTarget| > 0.35) target =
    // 0.35 * Sign(target)` — the cap applies AFTER the random delta
    // (`:58877-58888`), sign-preserving. Two same-seed winds burn identical
    // draws; only the flag differs. One walk (counter armed to 1) keeps the
    // extreme branch off (its counter starts 10..30, `Main.cs:58698-58706`).
    let mut capped = tr_core::wind::Wind::new(0xD129);
    let mut free = tr_core::wind::Wind::new(0xD129);
    for w in [&mut capped, &mut free] {
        w.counter = 1;
        w.speed_target = 0.5;
        w.speed_current = 0.5;
    }
    capped.any_player_120_life = false;
    free.any_player_120_life = true;
    capped.update_rain(0.0);
    free.update_rain(0.0);
    // The delta is at most ±0.1 (`Next(-100,101) * 0.001`), so the free
    // wind stays above 0.35 (0.4..0.6) while the capped one lands on it.
    assert!(
        capped.speed_target.abs() <= 0.35 + 1e-6,
        "capped {}",
        capped.speed_target
    );
    assert!(free.speed_target.abs() > 0.35, "free {}", free.speed_target);
    assert!(capped.speed_target > 0.0, "the cap preserves the sign");
    // The negative side mirrors (`Math.Sign`, `:58891`).
    let mut capped = tr_core::wind::Wind::new(0xD129);
    capped.counter = 1;
    capped.speed_target = -0.5;
    capped.speed_current = -0.5;
    capped.any_player_120_life = false;
    capped.update_rain(0.0);
    assert!(capped.speed_target < 0.0);
    assert!(capped.speed_target.abs() <= 0.35 + 1e-6);
}

#[test]
fn w4_d129_the_rain_start_gate_shares_the_same_flag() {
    // `Main.cs:64896-64908`: the not-raining start-roll gate re-scans the
    // same `statLifeMax >= 120` condition; the port passes the ONE folded
    // flag into `Rain::update_time` (`weather.rs:178-187`) instead of a
    // second scan — the census pins the shared value, not a second writer.
    let mut app = flow_app(None, vec![], vec![]);
    app.wind.frozen = true;
    let mut p = crate::player::Player::spawn(&app.world);
    p.max_hp = 120;
    app.player = Some(p);
    update_world_tick(&mut app, STEP);
    assert!(app.wind.any_player_120_life);
    // The flag the rain section reads is the wind field's value — one
    // source, both consumers (`:58889` cap + `:64905` start gate).
    let shared = app.wind.any_player_120_life;
    assert_eq!(shared, app.player.as_ref().is_some_and(|p| p.max_hp >= 120));
}
