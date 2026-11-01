//! Liquid and waterfall simulation (plan stage A4): the cellular
//! `Liquid.UpdateLiquid` flow at the original 30 Hz rate and the fixed 60 Hz
//! `WaterfallManager.UpdateFrame` / `FindWaterfalls` scan.

use glam::Vec2;
use tr_core::TILE_PX;

use crate::app::App;

/// Fixed 60 Hz liquid stepping; `Liquid.UpdateLiquid` itself runs every second
/// tick, matching the original 30 Hz rate.
pub(crate) fn update_liquids(app: &mut App, dt: f32) {
    let liquid_tick = &mut app.liquid_tick;
    let liquids = &mut app.liquids;
    let world = &mut app.world;
    app.liquid_step.tick(dt, |_| {
        *liquid_tick = liquid_tick.wrapping_add(1);
        if *liquid_tick % 2 == 0 {
            liquids.update(world);
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
    let half = screen * 0.5 / app.zoom;
    let view = [
        ((app.camera.x - half.x) / TILE_PX).floor() as i32 - 1,
        ((app.camera.y - half.y) / TILE_PX).floor() as i32 - 1,
        ((app.camera.x + half.x) / TILE_PX).ceil() as i32 + 1,
        ((app.camera.y + half.y) / TILE_PX).ceil() as i32 + 1,
    ];
    app.waterfall.find_waterfalls(&app.world, view, false);
}
