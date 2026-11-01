//! Weather (plan stage A4): the rain field's per-frame update.

use glam::Vec2;

use crate::biome::is_snow_biome;
use crate::app::App;

/// Rain runs on its own 60 Hz accumulator; drops are simulated at the original
/// per-tick velocity regardless of frame rate.
pub(crate) fn update_rain(app: &mut App, dt: f32, screen: Vec2) {
    let snow = is_snow_biome(&app.world, app.camera);
    app.rain
        .update(&app.world, app.camera, screen, app.zoom, dt, snow);
}
