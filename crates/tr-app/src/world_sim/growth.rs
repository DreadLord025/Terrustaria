//! The world-growth pass (`WorldGen.UpdateWorld`, see `tr_core::growth` for
//! the source map): the 60 Hz fixed step over the rate loops — the herb
//! growth rides the overground picks. The vines/moss and the biome spread
//! are M7-GROWTH-2.

use crate::app::App;
use tr_core::tile_light::FastRandom;

pub(crate) fn update_growth(app: &mut App, dt: f32) {
    let raining = app.rain.is_raining();
    let day_time = app.world.day_time;
    let mut rng = app.growth_rng;
    rng.next_seed_in_place();
    app.growth_step.tick(dt.min(0.25), |_| {
        let world = &mut app.world;
        // The port has no freeze-time power and runs at the day rate 1
        // (`Main.desiredWorldTilesUpdateRate` = `dayRate`).
        tr_core::growth::update_world(world, 1, raining, false, day_time, &mut rng);
    });
    app.growth_step.cap_backlog();
}
