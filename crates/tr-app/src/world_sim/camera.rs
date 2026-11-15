//! Camera (plan stage A4): the free-fly WASD pan (with the `TR_PAN` benchmark
//! hook) and the swapchain size.

use glam::Vec2;
use tr_core::TILE_PX;
use winit::keyboard::KeyCode;

use crate::app::App;

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
    if let Some(step) = crate::env::pan_step() {
        app.camera.x += step * TILE_PX;
    }
}

/// The draw-time camera state the C# keeps on `Main` (`Main.cs:463`/`:477`/
/// `:1097-1099`): the teleport-transition lerp, its timer pair, the
/// object-tracking pan and the vestigial `DebugCameraPan` accumulator pair
/// (nothing writes them in 1.4.5.8 — only `DoDraw_UpdateCameraPosition`'s
/// decay arms run — but the arms are ported for the 1:1 shape).
#[derive(Default)]
pub(crate) struct CameraState {
    pub camera_lerp: f32,
    pub camera_lerp_timer: i32,
    pub camera_lerp_time_toggle: i32,
    pub current_pan: Vec2,
    pub camera_x: f32,
    pub camera_y: f32,
}

impl CameraState {
    /// `Main.SetCameraLerp` (`Main.cs:3140-3144`).
    pub fn set_camera_lerp(&mut self, lerp: f32, time: i32) {
        self.camera_lerp = lerp;
        self.camera_lerp_timer = 0;
        self.camera_lerp_time_toggle = time;
    }
}

/// `Main.UpdateCameraPan` (`Main.cs:62673-62717`, the update tail
/// `:17814`): the NaN guards, then the pan walks toward its target — the
/// C#'s tracking sources (the spectate/drone/golf cameras, `:62722-62745`)
/// have no ported consumers, so the target stays `Vector2.Zero` with the
/// plain 36 px/tick speed (`:62684`). The `BlackFadeDist` snap arm
/// (`:62706-62711`) keeps its distance gate; the black fade itself is the
/// M6 cinematics scope.
pub(crate) fn update_camera_pan(app: &mut App, state: &mut CameraState) {
    if state.current_pan.x.is_nan() {
        state.current_pan.x = 0.0;
    }
    if state.current_pan.y.is_nan() {
        state.current_pan.y = 0.0;
    }
    let target_pan = Vec2::ZERO;
    let max_pan_speed = 36.0;
    // `:62706-62714`: the fade-distance snap, else `MoveTowards`.
    let black_fade_dist = {
        let (sw, sh) = screen_size(app);
        ((sw * sw + sh * sh).sqrt()) / 2.0 + 100.0
    };
    if state.current_pan.distance(target_pan) >= black_fade_dist {
        state.current_pan = target_pan;
    } else {
        let dist = state.current_pan.distance(target_pan);
        if dist <= max_pan_speed {
            state.current_pan = target_pan;
        } else {
            state.current_pan += (target_pan - state.current_pan).normalize() * max_pan_speed;
        }
    }
}

/// `Main.UpdateCameraLerp` (`Main.cs:18143-18156`, the update tail
/// `:17815`): the teleport transition's timer speeds the lerp up past its
/// delay and caps it at 1.
pub(crate) fn update_camera_lerp(app: &mut App, state: &mut CameraState) {
    let _ = app;
    if state.camera_lerp > 0.0 {
        state.camera_lerp_timer += 1;
        if state.camera_lerp_timer >= state.camera_lerp_time_toggle {
            state.camera_lerp +=
                ((state.camera_lerp_timer - state.camera_lerp_time_toggle) / 3 + 1) as f32 * 0.001;
        }
        if state.camera_lerp > 1.0 {
            state.camera_lerp = 1.0;
        }
    }
}

/// `Main.PlayerFocusedScreenPosition` (`Main.cs:62567-62570`) in the port's
/// centre form: `PlayerFocusedScreenPosition` is the view's TOP-LEFT
/// (`- UnscaledSize * 0.5`), and `app.camera` is the centre — the offset
/// cancels. `gfxOffY` rides the same line (`:62568`).
fn player_focused_center(app: &App, state: &CameraState) -> Vec2 {
    let Some(p) = app.player.as_ref() else {
        return app.camera;
    };
    let bottom = Vec2::new(p.x + crate::player::WIDTH as f32 * 0.5, p.y + crate::player::HEIGHT as f32);
    bottom + Vec2::new(0.0, p.gfx_off_y - 21.0)
        + Vec2::new(state.camera_x, state.camera_y)
        + state.current_pan
}

/// `Main.DoDraw_UpdateCameraPosition` (`Main.cs:62590-62649`) — the camera
/// update moved to DRAW time (S1-B-4): the vestigial `cameraX`/`cameraY`
/// decay arms, the snap to the player-focused centre, the teleport
/// transition's lerp arm (`:62638-62646`), the `(int)` truncation
/// (`:62647-62648`) and the world clamp (`ClampScreenPositionToWorld`,
/// `:63941-63950` — `WorldUtils.GetWorldPlayArea`'s 640 px margins,
/// `WorldUtils.cs:16-22`, in the centre form). The C#'s
/// `CameraModifiers.ApplyTo` (`:62647`) has no ported modifier set.
pub(crate) fn update_camera_at_draw(app: &mut App, state: &mut CameraState) {
    // `gameMenu`'s gate (`:62592-62595`); `netMode == 2` is out (no server
    // head draws). The port's front-end screens are the gameMenu analogue.
    if app.menu.is_some() || app.front.is_some() {
        return;
    }
    // The free-fly debug pan keeps the rig's own camera (the `follow_player`
    // toggle is the port's stand-in for `DebugOptions.noLimits`'s freedom),
    // and `TR_CAM`'s pinned centre owns it for the screenshot rig.
    if !app.follow_player || crate::env::camera_pin().is_some() {
        return;
    }
    // `:62596-62623` — the `cameraX`/`cameraY` decay pair.
    let Some(p) = app.player.as_ref() else {
        return;
    };
    if !p.pulley {
        state.camera_x = 0.0;
    } else if state.camera_x > 0.0 {
        state.camera_x -= 1.0;
        if state.camera_x < 0.0 {
            state.camera_x = 0.0;
        }
    } else if state.camera_x < 0.0 {
        state.camera_x += 1.0;
        if state.camera_x > 0.0 {
            state.camera_x = 0.0;
        }
    }
    if state.camera_y > 0.0 {
        state.camera_y -= 2.0;
        if state.camera_y < 0.0 {
            state.camera_y = 0.0;
        }
    } else if state.camera_y < 0.0 {
        state.camera_y += 2.0;
        if state.camera_y > 0.0 {
            state.camera_y = 0.0;
        }
    }
    // `:62626-62646` — the snap, then the transition lerp.
    let value = app.camera;
    let focused = player_focused_center(app, state);
    let mut screen = focused;
    if state.camera_lerp > 0.0 {
        if value.distance(focused) - Vec2::new(p.vx, p.vy).length() < 0.25 {
            state.camera_lerp = 0.0;
        } else {
            screen = value.lerp(focused, state.camera_lerp);
        }
    }
    // `:62647-62648` — the `(int)` truncation (the centre form truncates the
    // same integer offset the top-left form does).
    screen.x = screen.x.trunc();
    screen.y = screen.y.trunc();
    // `ClampScreenPositionToWorld` (`:63941-63950`): the play area's
    // 640 px margins, in the centre form (the C# clamps the top-left within
    // `[TL - translation, BR - ScaledSize - translation]`; the centre form
    // clamps within `[TL + size/2, BR - size/2]` — the port's zoom reads the
    // same centre, the C#'s `GameViewMatrix.Translation` is its zoom frame's
    // own offset).
    let (sw, sh) = screen_size(app);
    let right_world = app.world.width as f32 * TILE_PX;
    let bottom_world = app.world.height as f32 * TILE_PX;
    let tl = Vec2::new(640.0, 640.0) + Vec2::new(sw * 0.5, sh * 0.5);
    let br = Vec2::new(right_world - 640.0, bottom_world - 640.0)
        - Vec2::new(sw * 0.5, sh * 0.5);
    screen.x = screen.x.clamp(tl.x.min(br.x), tl.x.max(br.x));
    screen.y = screen.y.clamp(tl.y.min(br.y), tl.y.max(br.y));
    app.camera = screen;
}

/// `Main.StartCameraTransitionForPlayerTeleport`
/// (`Main.cs:62572-62588`, the C# call site `Player.cs:37463` —
/// `Player.Teleport`, the potion-recall `lerpSpeedupDelay = 10`): the
/// tracking-arm's pan compensation has no ported tracking consumers, so the
/// port runs the distance gate and the lerp set.
pub(crate) fn start_camera_transition_for_player_teleport(
    state: &mut CameraState,
    old_position: Vec2,
    lerp: f32,
    lerp_speedup_delay: i32,
) {
    let _ = old_position; // the tracking arm's pan compensation (`:62576-62579`)
    state.set_camera_lerp(lerp, lerp_speedup_delay);
}

#[cfg(test)]
mod tests {
    use super::*;
        /// S1-B-4 — `DoDraw_UpdateCameraPosition`'s snap: the camera centres on
    /// `PlayerFocusedScreenPosition` (`Main.cs:62567-62570`, the centre form)
    /// — `player.Bottom + (0, gfxOffY - 21) + (cameraX, cameraY) + CurrentPan`
    /// — truncated per the C#'s `(int)` (`:62647-62648`) and clamped into the
    /// play area (`ClampScreenPositionToWorld`, `:63941-63950`, the 640 px
    /// margins of `WorldUtils.cs:16-22`).
    #[test]
    fn s1_b_4_the_draw_time_camera_snaps_to_the_player_focused_center() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.world = crate::test_world_helpers::ground();
        let mut player = crate::player::Player::spawn(&app.world);
        app.player = Some(player);
        app.follow_player = true;
        let p = app.player.as_mut().unwrap();
        // Inside the play area (the 200x200 ground world's margins leave the
        // centre band 1280..1920 x 1000..2200 — the unclamped assertions need
        // the focused centre in it).
        p.x = 1600.0;
        p.y = 1600.0;
        let expected = Vec2::new(
            (p.x + crate::player::WIDTH as f32 * 0.5).trunc(),
            (p.y + crate::player::HEIGHT as f32 + p.gfx_off_y - 21.0).trunc(),
        );
        let mut state = CameraState::default();
        update_camera_at_draw(&mut app, &mut state);
        assert_eq!(app.camera, expected, "the focused centre, truncated");
        // `CurrentPan` shifts the centre (`:62569`'s `+ Main.CurrentPan`).
        state.current_pan = Vec2::new(96.0, -64.0);
        let mut state2 = state;
        update_camera_at_draw(&mut app, &mut state2);
        assert_eq!(app.camera, expected + Vec2::new(96.0, -64.0));
    }

    /// S1-B-4 — the world clamp: a pan that pushes the view past the play
    /// area's 640 px margins snaps back inside (`:63941-63950`).
    #[test]
    fn s1_b_4_the_camera_clamps_to_the_play_area() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.world = crate::test_world_helpers::ground();
        let mut player = crate::player::Player::spawn(&app.world);
        app.player = Some(player);
        app.follow_player = true;
        let mut state = CameraState::default();
        if let Some(p) = app.player.as_mut() {
            p.x = 700.0;
            p.y = 700.0;
        }
        state.current_pan = Vec2::new(-5000.0, -5000.0);
        update_camera_at_draw(&mut app, &mut state);
        let (sw, sh) = screen_size(&app);
        let tl = Vec2::new(640.0 + sw * 0.5, 640.0 + sh * 0.5);
        assert!(
            app.camera.x >= tl.x - 0.001 && app.camera.y >= tl.y - 0.001,
            "the camera clamps into the play area ({:?} vs {:?})",
            app.camera,
            tl
        );
    }

    /// S1-B-4 — the teleport transition: `SetCameraLerp` (`:3140-3144`) arms
    /// the draw arm's lerp (`:62638-62646`), `UpdateCameraLerp`
    /// (`:18143-18156`) speeds it up past the delay and caps it at 1, and
    /// once the camera catches the player the arm clears the lerp
    /// (`:62640-62642`'s `< 0.25f` gate).
    #[test]
    fn s1_b_4_the_teleport_transition_lerps_then_clears() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.world = crate::test_world_helpers::ground();
        let mut player = crate::player::Player::spawn(&app.world);
        app.player = Some(player);
        app.follow_player = true;
        let mut state = CameraState::default();
        start_camera_transition_for_player_teleport(&mut state, Vec2::ZERO, 0.1, 0);
        assert_eq!(state.camera_lerp, 0.1);
        // The draw arm lerps toward the player while the lerp lives (the
        // player sits inside the play area — the clamp holds the camera on
        // the focused centre there).
        if let Some(p) = app.player.as_mut() {
            p.x = 1600.0;
            p.y = 1600.0;
        }
        app.camera = Vec2::new(100.0, 100.0);
        update_camera_at_draw(&mut app, &mut state);
        assert!(
            (app.camera.x - 100.0).abs() > 1.0,
            "the lerp moved the camera off the old position ({})",
            app.camera.x
        );
        // The update walk's timer runs past the delay and caps the lerp.
        for _ in 0..3200 {
            update_camera_lerp(&mut app, &mut state);
        }
        assert_eq!(state.camera_lerp, 1.0, "the lerp caps at 1 (`:18152`)");
        // Drive the lerp draws until the camera catches the player: the
        // gate (`:62639-62642`) compares the frame's old camera against the
        // focused centre minus the velocity's length.
        for _ in 0..2000 {
            update_camera_at_draw(&mut app, &mut state);
            if state.camera_lerp == 0.0 {
                break;
            }
        }
        assert_eq!(
            state.camera_lerp, 0.0,
            "the < 0.25 catch clears the transition once the camera lands"
        );
    }
}
