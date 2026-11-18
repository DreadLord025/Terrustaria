//! The capture-mode frame driver — the App-side glue that pumps the
//! [`crate::capture::CaptureManager`] state machine once per frame at the
//! C# call sites: `CaptureManager.Instance.DrawTick()` at the `DoDraw` head
//! (`Main.cs:61729`) and `CaptureManager.Instance.Update(context)` in the
//! interface pass (`DrawInterface_10_CheckCaptureManager`, `Main.cs:46187`;
//! the fullscreen-map arm, `Main.cs:62166`).
//!
//! S1-E-17: the manager + interface + camera plumbing are ported whole
//! (`crate::capture`); the GPU render half (`Main.DrawCapture`,
//! `Main.cs:56673-57066`, the chunk renders into the 2048² framebuffer) is
//! the S1-G-13 render-target seam. Until that lands the camera's
//! [`crate::capture::CaptureRenderer`] is a no-op stub, so a started capture
//! drains its queue without producing pixels — the selection UI, the
//! countdown and the `IsCapturing` scene-metrics gate (`Main.cs:62518`) are
//! live, the write-out waits on the targets.

use glam::Vec2;

use crate::app::App;
use crate::capture::{
    CaptureInput, CaptureRenderer, CaptureSettings, CaptureWorldInfo, SelectionContext,
    ToolbarAction,
};

/// The S1-G-13 placeholder renderer: `render_chunk` returns `None` (no GPU
/// targets yet), so `CaptureCamera.DrawTick` advances the tile counter and
/// drains the queue but blits nothing. The real wgpu implementation lands
/// with the render-target family and replaces this stub.
struct PendingCaptureRenderer;

impl CaptureRenderer for PendingCaptureRenderer {
    fn render_chunk(
        &mut self,
        _chunk: &crate::capture::CaptureChunk,
        _settings: &CaptureSettings,
        _scaled_framebuffer_size: i32,
    ) -> Option<Vec<u8>> {
        None
    }
}

impl App {
    /// The per-frame capture update — `CaptureManager.Instance.Update`
    /// (`CaptureManager.cs:70-73`) at the interface-pass call site
    /// (`Main.cs:46187` world / `:62166` map). Runs only in the world branch
    /// (the capture UI is an in-game interface; the front end never draws
    /// it). The input snapshot mirrors `Main.mouseX/Y`, `mouseLeft/Right`,
    /// `mouseLeftRelease`, `MouseWorld`, the `ToggleCameraMode` trigger and
    /// the player gates (`CaptureInterface.cs:984`).
    ///
    /// The `ToggleCameraMode` trigger is the port's capture-toggle key. The
    /// vanilla binds it to F4 (`PlayerInput.cs:2091`); the port's F4 is the
    /// debug-overlay toggle (`loop.rs:652`, a documented deviation), so the
    /// capture toggle rides F6 here (documented, ANALYSIS "The capture
    /// mode"). The key is read as a held flag; the interface's own
    /// `KeyToggleActiveHeld` latch (`CaptureInterface.cs:899`, `:984-988`)
    /// turns it into a press edge.
    pub(crate) fn update_capture(&mut self, screen: Vec2) {
        if self.capture.capture_mode_disabled {
            return;
        }
        let (sw, sh) = (screen.x, screen.y);
        // `Main.MouseWorld` — the screen→world transform the click cluster
        // uses (`clicks.rs:1702-1705`): camera + (mouse - screen/2) / zoom.
        let mouse_world_x = self.camera.x + (self.mouse_pos.x - sw * 0.5) / self.zoom;
        let mouse_world_y = self.camera.y + (self.mouse_pos.y - sh * 0.5) / self.zoom;
        // `Main.screenPosition` — the view's world-px top-left.
        let screen_position_x = self.camera.x - sw * 0.5 / self.zoom;
        let screen_position_y = self.camera.y - sh * 0.5 / self.zoom;
        let map_fullscreen = self.map.as_ref().is_some_and(|m| m.open);
        let input = CaptureInput {
            mouse_x: self.mouse_pos.x,
            mouse_y: self.mouse_pos.y,
            mouse_left: self.mine_down,
            mouse_right: self.place_down,
            // `Main.mouseLeftRelease` — the press-edge flag the loop sets.
            mouse_left_release: self.ui_left_press,
            mouse_world_x,
            mouse_world_y,
            toggle_camera_mode: self.keys.contains(&winit::keyboard::KeyCode::F6),
            map_fullscreen,
            mouse_item_empty: self.mouse_item.kind == 0,
            player_dead: self.player.as_ref().is_some_and(|p| p.dead),
            // `Player.ghost` is unported (the master-mode ghost); false.
            player_ghost: false,
            // `Player.gravDir` is unported (the gravity potion); normal.
            gravity_reversed: false,
            is_fullscreen: false,
            map_enabled: self.settings.map_enabled,
            // `WiresUI.Settings.DrawWires` is unported (the port always
            // draws wires, `frame/draw.rs:816`); CaptureMech reads true.
            draw_wires: true,
        };
        let info = CaptureWorldInfo {
            max_tiles_x: self.world.width,
            max_tiles_y: self.world.height,
            screen_width: sw as i32,
            screen_height: sh as i32,
            screen_position_x,
            screen_position_y,
        };
        let context = if map_fullscreen {
            SelectionContext::Map
        } else {
            SelectionContext::World
        };
        let effects = self.capture.update(context, &input, &info);
        for cue in effects.cues {
            self.audio.play(cue);
        }
        if let Some(action) = effects.toolbar {
            self.run_capture_toolbar(action, &info);
        }
    }

    /// The toolbar actions `UpdateButtons` reports
    /// (`CaptureInterface.cs:1075-1134`). The capture-start actions build
    /// the `CaptureSettings` and call `StartCamera`; the mode picks
    /// re-toggle the camera; the rest are the focus/map/folder/close arms.
    fn run_capture_toolbar(&mut self, action: ToolbarAction, info: &CaptureWorldInfo) {
        match action {
            ToolbarAction::QuickScreenshot => {
                // `QuickScreenshot` (`:1161-1172`): the full-screen area.
                let area = crate::capture::CaptureInterface::full_screen_area(
                    (info.screen_position_x, info.screen_position_y),
                    (info.screen_width as f32 / self.zoom, info.screen_height as f32 / self.zoom),
                );
                self.start_capture_with_area(area);
            }
            ToolbarAction::CaptureArea => {
                // The area capture (`:1079-1095`): the pinned selection.
                let area = self.capture.interface.get_area();
                self.start_capture_with_area(area);
            }
            ToolbarAction::SelectMode(mode) => {
                // `SoundEngine.PlaySound(12)` + the mode pick + ToggleCamera
                // (`:1096-1123`).
                self.audio.play("Menu_Tick");
                self.capture.interface.selected_mode = mode;
                if let Some(cue) = self.capture.interface.toggle_camera(true) {
                    self.audio.play(cue);
                }
            }
            ToolbarAction::ResetFocus => {
                self.audio.play("Menu_Tick");
                self.capture.interface.reset_focus();
            }
            ToolbarAction::ToggleMapFullscreen => {
                self.audio.play("Menu_Tick");
                if let Some(m) = self.map.as_mut() {
                    let open = !m.open;
                    m.set_open(open);
                }
            }
            ToolbarAction::OpenFolder => {
                // `Utils.OpenFolder(SavePath/Captures)` (`:1127`) — the port
                // has no folder-open seam; the cue plays, the action is a
                // documented no-op (ANALYSIS).
                self.audio.play("Menu_Tick");
            }
            ToolbarAction::Close => {
                // `ToggleCamera(false)` + blockMouse (`:1129-1134`).
                if let Some(cue) = self.capture.interface.toggle_camera(false) {
                    self.audio.play(cue);
                }
            }
        }
    }

    /// `StartCamera` (`CaptureInterface.cs:1589-1596`) over the resolved
    /// area: the settings pick the biome (`CaptureBiome.GetCaptureBiome`),
    /// the entity/background/mech flags read the interface settings, and
    /// the camera locks. The shutter cue (sound 40) plays.
    fn start_capture_with_area(&mut self, area: crate::capture::CaptureRect) {
        let settings = &self.capture.interface.settings;
        let bg_style = self.preferred_bg_style();
        let water_style = crate::biome::calculate_water_style(&self.world, self.camera) as i32;
        let player_center_x = self
            .player
            .as_ref()
            .map_or(self.camera.x, |p| p.x + crate::player::WIDTH * 0.5);
        // `Main.treeX` is unported (the `.wld` reader skips it); [0;3]
        // reproduces the no-treeX fall-through (CaptureBiome.cs:206-222).
        let biome = crate::capture::CaptureBiome::get_capture_biome(
            settings.biome_choice_index,
            bg_style,
            water_style,
            player_center_x,
            [0; 3],
        );
        let capture_settings = CaptureSettings {
            area,
            use_scaling: settings.pack_image,
            capture_entities: settings.include_entities,
            capture_background: !settings.transparent_background,
            // `WiresUI.Settings.DrawWires` is unported (wires always draw).
            capture_mech: true,
            biome,
            ..CaptureSettings::default()
        };
        let full_screen = crate::capture::CaptureInterface::full_screen_area(
            (
                self.camera.x - self.screen_size().0 * 0.5 / self.zoom,
                self.camera.y - self.screen_size().1 * 0.5 / self.zoom,
            ),
            (self.screen_size().0 / self.zoom, self.screen_size().1 / self.zoom),
        );
        let cue = self.capture.interface.start_camera(capture_settings, full_screen);
        self.audio.play(cue);
    }

    /// `CaptureManager.Instance.DrawTick()` (`CaptureManager.cs:99-103`) at
    /// the `DoDraw` head (`Main.cs:61729`): the countdown advances and a
    /// started capture drains one chunk. The GPU render is the S1-G-13
    /// seam (the stub renderer produces no pixels yet); the `GlobalTimerPaused`
    /// pause/unpause (`CaptureCamera.cs:112`/`:390`) is the port's
    /// `game_paused`-adjacent clock gate — documented as pending the targets
    /// (the stub drains instantly, so the pause would never be observable).
    pub(crate) fn capture_draw_tick(&mut self) {
        let mut renderer = PendingCaptureRenderer;
        let (_start, _finished) = self.capture.draw_tick(&mut renderer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::app::flow::FrontScreen;

    /// The capture toggle key (F6) activates the interface through the
    /// per-frame update; a second held frame does not re-toggle (the
    /// `KeyToggleActiveHeld` latch, `CaptureInterface.cs:984-988`).
    #[test]
    fn s1_e_17_the_capture_toggle_key_activates_the_interface() {
        let mut app = flow_app(None, vec![], vec![]);
        app.front = None;
        app.menu = None;
        let screen = Vec2::new(1280.0, 720.0);
        // No key: inactive.
        app.update_capture(screen);
        assert!(!app.capture.interface.active);
        // F6 held: the press edge activates.
        app.keys.insert(winit::keyboard::KeyCode::F6);
        app.update_capture(screen);
        assert!(app.capture.interface.active);
        assert!(app.capture.interface.just_activated);
        // Still held: no re-toggle.
        app.update_capture(screen);
        assert!(app.capture.interface.active);
        // Release + re-press: deactivates.
        app.keys.remove(&winit::keyboard::KeyCode::F6);
        app.update_capture(screen);
        app.keys.insert(winit::keyboard::KeyCode::F6);
        app.update_capture(screen);
        assert!(!app.capture.interface.active);
    }

    /// The `IsCapturing` draw-side scene-metrics gate (`Main.cs:62518-62521`):
    /// a started capture parks the metrics scan (the update-side freeze is the
    /// DoUpdate gate, `Main.cs:16886-16890`, in frame.rs). The camera's active
    /// flag is the gate's input; this test pins the draw-tick drain + clear.
    #[test]
    fn s1_e_17_a_started_capture_gates_the_scene_metrics() {
        let mut app = flow_app(None, vec![], vec![]);
        assert!(!app.capture.is_capturing());
        // Start a capture directly through the camera (the queue machine).
        let settings = CaptureSettings {
            area: crate::capture::CaptureRect::new(0, 0, 10, 10),
            ..CaptureSettings::default()
        };
        app.capture.capture(settings).unwrap();
        assert!(app.capture.is_capturing());
        // The draw-tick drains the queue (the stub renders nothing) and
        // finishes; the gate clears.
        app.capture_draw_tick();
        assert!(!app.capture.is_capturing());
    }

    /// The toolbar's area-capture action builds the settings and locks the
    /// camera (`StartCamera`, `CaptureInterface.cs:1589-1596`).
    #[test]
    fn s1_e_17_the_area_capture_locks_the_camera() {
        let mut app = flow_app(None, vec![], vec![]);
        app.capture.interface.active = true;
        app.capture.interface.edge_a_pinned = true;
        app.capture.interface.edge_b_pinned = true;
        app.capture.interface.edge_a = (100, 200);
        app.capture.interface.edge_b = (150, 250);
        let info = CaptureWorldInfo {
            max_tiles_x: app.world.width,
            max_tiles_y: app.world.height,
            screen_width: 1280,
            screen_height: 720,
            screen_position_x: 0.0,
            screen_position_y: 0.0,
        };
        app.run_capture_toolbar(ToolbarAction::CaptureArea, &info);
        assert!(app.capture.interface.camera_lock);
        let parked = app.capture.interface.camera_settings.as_ref().unwrap();
        assert_eq!(parked.area, crate::capture::CaptureRect::new(100, 200, 51, 51));
    }
}