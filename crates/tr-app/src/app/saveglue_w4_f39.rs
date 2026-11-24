// W4-f39 census — the menu-return `StopSlimeRain` (`Main.cs:48216-48219`,
// `DrawMenu`'s `menuMode <= 1 && Main.slimeRain` arm). The port's
// `save_and_quit` is the in-world → menuMode-0 transition. Written-not-run
// (code-only lane).

use crate::app::flow::front_flow_tests::flow_app;

/// A flow app with a scratch save path (the save funnel writes the world).
fn quit_app(name: &str) -> crate::app::App {
    let mut app = flow_app(None, vec![], vec![]);
    let dir = crate::test_world_helpers::test_scratch(&format!("saveglue-f39-{name}"));
    app.world_path = dir.join("world.trwld");
    app
}

#[test]
fn w4_f39_save_and_quit_stops_a_running_slime_rain() {
    // `Main.cs:48216-48219`: returning to the main menu while the event is
    // up calls `StopSlimeRain` — the flag down, the negative cooldown
    // (`-Next(3024,6048)*100`), the warning armed. The draw rides the bare
    // Main.rand stand-in (`rngs.global()`), the menu frame being outside
    // every SwapRandom block.
    let mut app = quit_app("stop");
    app.world.slime_rain = true;
    app.world.slime_rain_time = 30000.0;
    app.world.slime_rain_kills = 42;
    // The twin: the global stream's first Next(3024, 6048).
    let mut twin = tr_core::UnifiedRandom::new(app.world.seed as i32);
    let want = -(twin.next_range(3024, 6048) as f64) * 100.0;
    app.save_and_quit();
    assert!(!app.world.slime_rain, "the menu return stopped the rain");
    assert_eq!(app.world.slime_rain_time, want, "the negative cooldown");
    assert_eq!(app.world.slime_rain_kills, 42, "the stop keeps the counter");
    assert_eq!(app.world.slime_warning_time, 420, "the warning armed");
    assert!(app.menu.is_some(), "the menu opened");
}

#[test]
fn w4_f39_save_and_quit_with_no_rain_draws_nothing() {
    // The `Main.slimeRain` gate (`:48216`): no event up → no stop, no draw.
    let mut app = quit_app("norain");
    app.world.slime_rain = false;
    app.world.slime_rain_time = -100.0; // a live cooldown stays put
    let pristine = tr_core::UnifiedRandom::new(app.world.seed as i32);
    app.save_and_quit();
    assert_eq!(app.world.slime_rain_time, -100.0, "the cooldown untouched");
    assert_eq!(
        *app.rngs.global(),
        pristine,
        "the no-rain menu return drew nothing"
    );
}
