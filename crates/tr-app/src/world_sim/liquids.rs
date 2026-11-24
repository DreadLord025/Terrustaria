//! Liquid and waterfall simulation (plan stage A4): the cellular
//! `Liquid.UpdateLiquid` flow at the original 30 Hz rate and the fixed 60 Hz
//! `WaterfallManager.UpdateFrame` / `FindWaterfalls` scan.

use glam::Vec2;

use crate::app::App;

/// Fixed 60 Hz liquid stepping; `Liquid.UpdateLiquid` itself runs every second
/// tick, matching the original 30 Hz rate.
pub(crate) fn update_liquids(app: &mut App, dt: f32) {
    let liquid_tick = &mut app.liquid_tick;
    let liquids = &mut app.liquids;
    let world = &mut app.world;
    let audio = &mut app.audio;
    app.liquid_step.tick(dt, |_| {
        *liquid_tick = liquid_tick.wrapping_add(1);
        if *liquid_tick % 2 == 0 {
            liquids.update(world);
            // `Liquid.cs:1345-1347`: the live merge plays its
            // `PlayLiquidChangeSound` inline — the port's engine is pure,
            // so it queues the events and the tick plays them here
            // (S1-E-25). The C#'s `!Main.gameMenu` half of the gate is the
            // caller's own: the settlement pass never runs in the menu.
            for (kind, x, y) in liquids.drain_merge_events() {
                audio.play_liquid_change(kind, x, y);
            }
        }
    });
    app.liquid_step.cap_backlog();
}

/// Fixed 60 Hz `WaterfallManager.UpdateFrame` / `FindWaterfalls`, matching the
/// original's per-update tick and 30-tick rescan cadence.
pub(crate) fn update_waterfalls(app: &mut App, dt: f32, screen: Vec2) {
    let waterfall = &mut app.waterfall;
    let steps = app
        .waterfall_step
        .tick(dt.min(0.25), |_| waterfall.update_tick());
    app.waterfall_step.cap_backlog();
    if steps == 0 {
        return;
    }
    // `WaterfallManager.cs:119-122`: the scan rectangle truncates the raw
    // unscaled `Main.screenPosition`/screen size (`(int)` casts, no zoom
    // divide, no floor/ceil); the `-1`/`+2` margins live inside
    // `find_waterfalls`. The view's top-left is camera - screen/2, the same
    // S1-E-19 `Camera.UnscaledPosition` convention the draw pass feeds
    // `ScreenInfo`.
    let view = tr_core::waterfall::WaterfallManager::scan_view(
        [app.camera.x - screen.x * 0.5, app.camera.y - screen.y * 0.5],
        screen.x,
        screen.y,
    );
    // `Main.cs:61841`: `FindWaterfalls(Main.instantBGTransitionCounter == 10)`
    // — the spawn/teleport instant-BG window (`Player.cs:37569` /
    // `Main.cs:62559` set 10; `DrawBG` decrements at `:64039-64041`, which
    // runs after this pass in the port's frame order).
    let forced = app.bg_instant == 10;
    app.waterfall.find_waterfalls(&app.world, view, forced);
}
