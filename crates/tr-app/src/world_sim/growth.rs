//! The world-growth pass (`WorldGen.UpdateWorld`, see `tr_core::growth` for
//! the source map): the 60 Hz fixed step over the rate loops — the herb
//! growth rides the overground picks. The vines/moss and the biome spread
//! are M7-GROWTH-2.

use crate::app::App;
use tr_core::tile_light::FastRandom;

pub(crate) fn update_growth(app: &mut App, dt: f32) {
    // The hardmode switch (`WorldGen.StartHardmode`, `WorldGen.cs:31701-
    // 31720`): the dev trigger `TR_HARDMODE=1` (the WoF kill rides the
    // M4-NPC families) runs the runner half once.
    if !app.world.hard_mode && crate::env::hardmode_forced() {
        app.world.hard_mode = true;
        let corruption = (app.world.seed >> 56) & 1 == 1;
        let mut rng = app.growth_rng;
        rng.next_seed_in_place();
        tr_core::spread::initialize_hard_mode(&mut app.world, corruption, &mut rng);
    }
    let raining = app.rain.is_raining();
    let day_time = app.world.day_time;
    let mut rng = app.growth_rng;
    rng.next_seed_in_place();
    app.growth_step.tick(dt.min(0.25), |_| {
        let world = &mut app.world;
        // The port has no freeze-time power and runs at the day rate 1
        // (`Main.desiredWorldTilesUpdateRate` = `dayRate`).
        tr_core::growth::update_world(world, 1, raining, false, day_time, &mut rng);
        // `hardUpdateWorld`'s spread arms ride the same picks (the port
        // runs them unconditionally — the hardmode gate waits for
        // M5-HARDFORCE; `downedPlantBoss` false = the full rate).
        let sx = rng.next_range(10, world.width - 10);
        let sy = rng.next_range(world.rock_layer as i32, world.height - 40);
        tr_core::spread::hard_update_world(world, sx, sy, false, true, &mut rng);
    });
    app.growth_step.cap_backlog();
}
