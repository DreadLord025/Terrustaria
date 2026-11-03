//! Player simulation (plan stage A4): the deterministic input assembly and the
//! fixed 60 Hz `Player::update` stepping with the render interpolation the
//! camera follows.

use glam::Vec2;
use winit::keyboard::KeyCode;

use crate::app::App;
use crate::player::PlayerInput;

pub(crate) fn update_player(app: &mut App, dt: f32) {
    // Deterministic held input for screenshots (`TR_HOLD=right,jump`).
    static HOLD: std::sync::OnceLock<(bool, bool)> = std::sync::OnceLock::new();
    let (hold_right, hold_jump) = *HOLD.get_or_init(|| {
        let v = std::env::var("TR_HOLD")
            .unwrap_or_default()
            .to_ascii_lowercase();
        (v.contains("right"), v.contains("jump"))
    });
    // `Player.cs:24772-24803`: while the map is fullscreen every movement control
    // is zeroed, so the player stands still and the WASD keys pan the map.
    let map_open = app.map.as_ref().is_some_and(|m| m.open);
    let input = if map_open {
        PlayerInput {
            left: false,
            right: false,
            jump: false,
            down: false,
        }
    } else {
        PlayerInput {
            left: app.keys.contains(&KeyCode::KeyA) || app.keys.contains(&KeyCode::ArrowLeft),
            right: hold_right
                || app.keys.contains(&KeyCode::KeyD)
                || app.keys.contains(&KeyCode::ArrowRight),
            jump: hold_jump
                || app.keys.contains(&KeyCode::Space)
                || app.keys.contains(&KeyCode::KeyW)
                || app.keys.contains(&KeyCode::ArrowUp),
            down: app.keys.contains(&KeyCode::KeyS) || app.keys.contains(&KeyCode::ArrowDown),
        }
    };
    if let Some(p) = app.player.as_mut() {
        // `Player.Update` -> `UpdateDead` (`Player.cs:16919`): the dead player
        // runs the respawn-timer tick instead of the normal simulation. The
        // boss check reads the live NPC list; the danger-NPC gate has no
        // `DangerThatPreventsOtherDangers` members in the ported set yet.
        if p.dead {
            let boss_near = p.boss_hinders(&app.npcs);
            let mut respawn_now = false;
            app.player_step.tick(dt, |_| {
                respawn_now = p.update_dead(input.jump, boss_near, false) || respawn_now;
            });
            if respawn_now {
                p.revive();
                app.npcs.clear();
                app.npc_timer = 4.0;
                if app.follow_player {
                    let (cx, cy) = p.center();
                    app.camera = Vec2::new(cx, cy);
                }
            }
            if app.follow_player {
                let (cx, cy) = p.center();
                app.camera = app.camera.lerp(Vec2::new(cx, cy), 1.0 - (-dt * 12.0).exp());
            }
            return;
        }
        let step = 1.0 / 60.0;
        app.player_step.tick(dt, |_| p.update(&app.world, input));
        app.player_step.drop_backlog();
        if app.follow_player {
            // Follow the interpolated position so the camera is smooth between
            // the 60 Hz simulation steps.
            let alpha = (app.player_step.residual() / step).clamp(0.0, 1.0);
            let render = p.interpolated(alpha);
            let (cx, cy) = render.center();
            app.camera = app.camera.lerp(Vec2::new(cx, cy), 1.0 - (-dt * 12.0).exp());
        }
    }
    if map_open {
        app.update_map_input();
    }
}
