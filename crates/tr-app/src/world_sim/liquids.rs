//! Liquid and waterfall simulation (plan stage A4): the cellular
//! `Liquid.UpdateLiquid` flow at the original 30 Hz rate and the fixed 60 Hz
//! `WaterfallManager.UpdateFrame` / `FindWaterfalls` scan.

use glam::Vec2;
use tr_core::TILE_PX;

use crate::app::App;

/// Fixed 60 Hz liquid stepping; the `WorldGen.UpdateWorld` driver
/// (`WorldGen.cs:71526-71534`) runs `Liquid.UpdateLiquid` every second tick
/// via `Liquid.skipCount` — the counter rides `LiquidSystem` itself (the C#
/// static, `Liquid.cs:19`), so `ReInit` resets the window on a world swap.
pub(crate) fn update_liquids(app: &mut App, dt: f32) {
    let liquids = &mut app.liquids;
    let world = &mut app.world;
    let audio = &mut app.audio;
    // The retirement-tail kills drain inside the tick (the `liquids` borrow)
    // and resolve after it (the `app` borrow) — the two cannot overlap.
    let mut pending_kills = Vec::new();
    app.liquid_step.tick(dt, |_| {
        // `WorldGen.cs:71526-71533` — `skipCount++; if (skipCount > 1)
        // { using (Main.SwapRandom("UpdateLiquid")) UpdateLiquid();
        // skipCount = 0; }`, verbatim on the engine driver. The RNG swap is
        // this caller's seam: the drained kills below resolve on
        // `RngKey::UpdateLiquid` (the stream the C# `KillTile` drops draw).
        if liquids.update_world_tick(world) {
            // `Liquid.cs:1345-1347`: the live merge plays its
            // `PlayLiquidChangeSound` inline — the port's engine is pure,
            // so it queues the events and the tick plays them here
            // (S1-E-25). The C#'s `!Main.gameMenu` half of the gate is the
            // caller's own: the settlement pass never runs in the menu.
            for (kind, x, y) in liquids.drain_merge_events() {
                audio.play_liquid_change(kind, x, y);
            }
            // `Liquid.cs:1580-1598` / `WorldGen.cs:80435-80448`: the
            // retirement-tail herb/lily-pad/liquid-death kills spawn their
            // drops inline under the UpdateLiquid swap — the port queues
            // the records and resolves them after the tick (W4-g47).
            pending_kills.extend(liquids.drain_edit_kills());
        }
    });
    app.liquid_step.cap_backlog();
    crate::world_sim::editing::resolve_liquid_edit_kills(app, pending_kills);
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
