//! Camera (plan stage A4): the free-fly WASD pan (with the `TR_PAN` benchmark
//! hook) and the swapchain size.

use glam::Vec2;
use tr_core::TILE_PX;
use winit::keyboard::KeyCode;

use crate::app::{pan_step, App};

/// The swapchain size in pixels (the deterministic fallback mirrors the
/// `TR_SHOT`/headless default of 1280x720).
pub(crate) fn screen_size(app: &App) -> (f32, f32) {
    app.gpu
        .as_ref()
        .map_or((1280.0, 720.0), |g| (g.config.width as f32, g.config.height as f32))
}

pub(crate) fn pan_camera(app: &mut App, dt: f32) {
    let mut dir = Vec2::ZERO;
    if app.keys.contains(&KeyCode::KeyA) || app.keys.contains(&KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    if app.keys.contains(&KeyCode::KeyD) || app.keys.contains(&KeyCode::ArrowRight) {
        dir.x += 1.0;
    }
    if app.keys.contains(&KeyCode::KeyW) || app.keys.contains(&KeyCode::ArrowUp) {
        dir.y -= 1.0;
    }
    if app.keys.contains(&KeyCode::KeyS) || app.keys.contains(&KeyCode::ArrowDown) {
        dir.y += 1.0;
    }
    if dir != Vec2::ZERO {
        app.camera += dir.normalize() * 900.0 * dt / app.zoom;
    }
    if let Some(step) = pan_step() {
        app.camera.x += step * TILE_PX;
    }
}
