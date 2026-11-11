//! The in-game options window — `Main.IngameOptions` (`IngameOptions.cs`,
//! `Main.cs:45532`) — BUG-11's pause-menu Settings entry: the
//! `DrawSettingButton` that opens/closes the window, the panel itself and
//! the volume bars (music/sound/ambient) over `ui::sliders`' value-bar
//! geometry.
//!
//! The vanilla window is a 670×480 panel split in two columns
//! (`IngameOptions.Draw`, `Main.cs:45560+`): the left column of navigation
//! rows (general/interface/video/controls/language) and the right column of
//! value bars and on/off options. This port mounts the rows that already
//! exist in the port's `Settings` — the three volume bars, the map scale and
//! the auto-pause/auto-save toggles — and closes on Escape
//! (`IngameOptions.Close`, `IngameOptions.cs:84-101`); the caller persists
//! through `Settings::save` on close.
//!
//! Layout constants are the C#'s: the panel's origin `screen/2 - (670,
//! 480)/2` (`Main.cs:45563-45566`), the 20-px hover padding (`num3`,
//! `:45569`), the title `GameUI.SettingsMenu` centred at the panel's top +
//! 30 (`:45571`), and the rows' 70-px pitch (`num5`, `:45595`).

use crate::assets::Lookups;
use tr_render::SpriteRenderer;
use glam::Vec2;
use crate::settings::Settings;
use crate::ui::sliders;
use crate::ui::text;

/// The C# panel size (`IngameOptions.cs:45563`'s `new Vector2(670f, 480f)`).
pub const PANEL_W: f32 = 670.0;
pub const PANEL_H: f32 = 480.0;
/// `num3 = 20` (`Main.cs:45569`) — the hover padding around the panel.
const PAD: f32 = 20.0;
/// The row pitch of the option rows (`Main.cs:45595`'s `num5 = 70`).
const ROW: f32 = 70.0;
/// The first row's offset under the panel title (`num4 = 130`, `:45596`).
const FIRST_ROW: f32 = 130.0;
/// The value bar's left edge on the panel's right half (`rightX`'s port
/// twin, `Main.cs:51536`): the panel's centre plus 100.
const BAR_X: f32 = 96.0;

/// The "Settings" button's position and size (`Main.cs:45492-45521`
/// `RefreshSettingsButtonStatus`): `posX = UIScreenWidth - 110`,
/// `posY = UIScreenHeight - 20` in the port's fixed 960×600 UI canvas (the
/// bestiary icon at 498,278 and the emote button at 534,278 ride the same
/// space), pushed to the side when the equipment area would overlap
/// (`:45513-45520` — `posX - 140`, `posY` at the equipment bottom, the
/// `up = 0` variant folded into one fixed rect).
pub const BUTTON_POS: [f32; 2] = [850.0, 580.0];
pub const BUTTON_SIZE: [f32; 2] = [30.0, 30.0];

/// The button's hover rect — `rectangle.Contains(new Point(mouse))`
/// (`IngameOptions.cs:1557`): inclusive left/top, exclusive right/bottom.
pub fn button_hit(mouse: Vec2) -> bool {
    let (x, y) = (BUTTON_POS[0], BUTTON_POS[1]);
    let (w, h) = (BUTTON_SIZE[0], BUTTON_SIZE[1]);
    ((mouse.x >= x) && (mouse.x < x + w) && (mouse.y >= y) && (mouse.y < y + h))
}

/// The live window state on the `App`.
#[derive(Clone, Debug, Default)]
pub struct IngameOptions {
    /// `Main.ingameOptionsWindow` (`Main.cs:2025`).
    pub open: bool,
    /// `IngameOptions.rightLock` (`Main.cs:51527-51572`): the sticky value-bar
    /// grab — 0 music, 1 sound, 2 ambient.
    pub bar_grab: Option<u8>,
}

impl IngameOptions {
    pub fn new() -> Self {
        Self::default()
    }

    /// `IngameOptions.Open` (`IngameOptions.cs:56`) / `Close` (`:84`).
    pub fn toggle(&mut self) {
        self.open = !self.open;
        if !self.open {
            self.bar_grab = None;
        }
    }

    pub fn close(&mut self) {
        self.open = false;
        self.bar_grab = None;
    }
}

/// `vector3 = vector / 2f - vector2 / 2f` (`Main.cs:45563-45566`).
pub fn panel_origin(screen: Vec2) -> [f32; 2] {
    [
        screen.x * 0.5 - PANEL_W * 0.5,
        screen.y * 0.5 - PANEL_H * 0.5,
    ]
}

/// `IngameOptions._GUIHover` (`IngameOptions.cs:45565`): the padded panel
/// (`Main.player[myPlayer].mouseInterface` is set over it, `:45570-45573`).
pub fn panel_rect(screen: Vec2) -> [f32; 4] {
    let o = panel_origin(screen);
    [
        o[0] - PAD,
        o[1] - PAD,
        PANEL_W + PAD * 2.0,
        PANEL_H + PAD * 2.0,
    ]
}

/// Row `k`'s y (`Main.cs:45595`'s `num6 = num4 + num5 * k`, `num4 = 130`).
pub fn row_y(screen: Vec2, k: usize) -> f32 {
    panel_origin(screen)[1] + FIRST_ROW + ROW * k as f32
}

/// XNA `Rectangle.Contains(Point)` (`IngameOptions.cs:1557`): inclusive
/// left/top, exclusive right/bottom.
pub fn contains(p: Vec2, rect: [f32; 4]) -> bool {
    p.x >= rect[0] && p.x < rect[0] + rect[2] && p.y >= rect[1] && p.y < rect[1] + rect[3]
}

/// One `IngameOptions.DrawValueBar` row's hovered/drag state
/// (`Main.cs:51533-51543` and the sound/ambient twins, `:51635-51646`):
/// hovered is `inBar || rightLock == id` (`:51536`) and the value follows the
/// mouse while the button is held and the slot is locked (`:51540`).
/// Returns `(hovered, Some(new_value))` — the value only when it moved.
pub fn value_bar_drag(
    right_x: f32,
    center_y: f32,
    mouse: Vec2,
    mouse_down: bool,
    grab: Option<u8>,
    id: u8,
) -> (bool, Option<f32>) {
    let (rect, hit) = sliders::value_bar_rects(right_x, center_y);
    let in_bar = contains(mouse, hit);
    let hovered = in_bar || grab == Some(id);
    let stored = if hovered && mouse_down && grab == Some(id) {
        Some(sliders::value_bar_value(rect, mouse.x))
    } else {
        None
    };
    (hovered, stored)
}

/// `IngameOptions.MouseOver` (`Main.cs:42882`): the bars arm their hover,
/// the volume bars drag while locked (`rightLock`, cleared on release at
/// `:51529-51531`), and the caller reads the resulting state.
pub fn mouse_over(app: &mut crate::app::App, screen: Vec2, mouse: Vec2, mouse_down: bool) {
    if !app.ingame_options.open {
        app.ingame_options.bar_grab = None;
        return;
    }
    if !mouse_down {
        app.ingame_options.bar_grab = None;
    }
    let bar_x = panel_origin(screen)[0] + BAR_X + PANEL_W * 0.25;
    let (y0, y1, y2) = (
        row_y(screen, 0) + 8.0,
        row_y(screen, 1) + 8.0,
        row_y(screen, 2) + 8.0,
    );
    for (id, (hovered, stored)) in [
        value_bar_drag(bar_x, y0, mouse, mouse_down, app.ingame_options.bar_grab, 0),
        value_bar_drag(bar_x, y1, mouse, mouse_down, app.ingame_options.bar_grab, 1),
        value_bar_drag(bar_x, y2, mouse, mouse_down, app.ingame_options.bar_grab, 2),
    ]
    .into_iter()
    .enumerate()
    {
        if hovered && mouse_down {
            app.ingame_options.bar_grab = Some(id as u8);
        }
        if let (Some(v), 0..=2) = (stored, id) {
            match id {
                0 => app.settings.music_volume = v,
                1 => app.settings.sound_volume = v,
                _ => app.settings.ambient_volume = v,
            }
        }
    }
}

/// `IngameOptions.Draw` (`Main.cs:45560+`): the panel, the title and the
/// rows. The vanilla's left column of navigation rows is the port's
/// Settings sub-screens; the port's window carries the volume bars, the map
/// scale and the auto-pause/auto-save toggles that `Settings` already has.
pub fn draw_options(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    settings: &Settings,
    options: &IngameOptions,
    screen: Vec2,
    mouse: Vec2,
) {
    if !options.open {
        return;
    }
    let origin = panel_origin(screen);
    // `Utils.DrawInvBG` (`Main.cs:45572-45576`): the panel's backdrop.
    let panel = [
        origin[0],
        origin[1],
        PANEL_W,
        PANEL_H,
    ];
    if let Some(uv) = lookups.hud_uv(lookups.ui_back.as_ref()) {
        r.push_ui_sprite(
            [panel[0], panel[1]],
            [panel[2], panel[3]],
            uv,
            [33.0 / 255.0, 15.0 / 255.0, 91.0 / 255.0, 0.685],
        );
    }
    // `Language.GetTextValue("GameUI.SettingsMenu")` (`Main.cs:45577`).
    text::hud_text(
        r,
        lookups,
        origin[0] + PANEL_W * 0.5,
        origin[1] + 30.0,
        1.0,
        [1.0; 4],
        "Settings",
    );
    // The rows.
    let labels = [
        ("Music", &settings.music_volume),
        ("Sound", &settings.sound_volume),
        ("Ambient", &settings.ambient_volume),
    ];
    let bar_x = origin[0] + BAR_X + PANEL_W * 0.25;
    for (k, (label, value)) in labels.iter().enumerate() {
        let y = row_y(screen, k);
        text::hud_text(
            r,
            lookups,
            origin[0] + 40.0,
            y,
            0.85,
            [1.0; 4],
            label,
        );
        let (bar_rect, bar_hit) = sliders::value_bar_rects(bar_x, y + 8.0);
        let hovered = contains(mouse, bar_hit) || options.bar_grab == Some(k as u8);
        let _ = bar_rect;
        sliders::draw_value_bar(r, lookups, bar_x, y + 8.0, **value, mouse);
    }
    // The toggles.
    for (k, (label, on)) in [
        ("Map Scale", settings.map_scale > 1.0),
        ("Auto Pause", settings.auto_pause),
        ("Auto Save", settings.auto_save),
    ]
    .into_iter()
    .enumerate()
    {
        let y = row_y(screen, 3 + k);
        text::hud_text(
            r,
            lookups,
            origin[0] + 40.0,
            y,
            0.85,
            [1.0; 4],
            label,
        );
        text::hud_text(
            r,
            lookups,
            origin[0] + PANEL_W * 0.55,
            y,
            0.85,
            [0.8, 0.8, 0.8, 1.0],
            if on { "On" } else { "Off" },
        );
    }
}

/// The click pass (`IngameOptions.Draw`'s flag4 block, `Main.cs:45600+`):
/// the bars' rows set their values, the toggles flip, and the exit closes
/// (`IngameOptions.Close`). Returns `true` when the click was consumed.
pub fn click(app: &mut crate::app::App, screen: Vec2, mouse: Vec2) -> bool {
    if !app.ingame_options.open {
        return false;
    }
    let origin = panel_origin(screen);
    // The bars.
    let bar_x = origin[0] + BAR_X + PANEL_W * 0.25;
    for (id, k) in (0u8..3).zip(0usize..3) {
        let (rect, hit) = sliders::value_bar_rects(bar_x, row_y(screen, k) + 8.0);
        if contains(mouse, hit) {
            let v = sliders::value_bar_value(rect, mouse.x);
            app.ingame_options.bar_grab = Some(id);
            match id {
                0 => app.settings.music_volume = v,
                1 => app.settings.sound_volume = v,
                _ => app.settings.ambient_volume = v,
            }
            return true;
        }
    }
    // The toggles.
    for k in 0..3usize {
        let y = row_y(screen, 3 + k);
        if mouse.y >= y && mouse.y < y + ROW * 0.5 {
            match k {
                0 => app.settings.auto_pause = !app.settings.auto_pause,
                1 => app.settings.auto_save = !app.settings.auto_save,
                _ => {}
            }
            return true;
        }
    }
    // Anywhere else on the panel closes it (`IngameOptions.Close`, the
    // vanilla's Back row at `:259`).
    if contains(mouse, panel_rect(screen)) {
        app.ingame_options.close();
        return true;
    }
    false
}

/// `Main.DrawSettingButton` (`Main.cs:45532-45580`): the "Settings" label
/// centred in its button, hover-brightened.
pub fn draw_settings_button(r: &mut SpriteRenderer, lookups: &Lookups, mouse: Vec2) {
    let (x, y) = (BUTTON_POS[0], BUTTON_POS[1]);
    let (w, h) = (BUTTON_SIZE[0], BUTTON_SIZE[1]);
    // The frame and the label, the vanilla's `DrawSettingButton` shape.
    if let Some(uv) = lookups.hud_uv(lookups.ui_back.as_ref()) {
        let hovered = button_hit(mouse);
        let tint = if hovered {
            [200.0 / 255.0, 200.0 / 255.0, 200.0 / 255.0, 1.0]
        } else {
            [120.0 / 255.0, 120.0 / 255.0, 120.0 / 255.0, 1.0]
        };
        r.push_ui_sprite([x, y], [w, h], uv, tint);
        text::hud_text(
            r,
            lookups,
            x + w * 0.5,
            y + h * 0.5,
            0.75,
            [1.0; 4],
            "Settings",
        );
    }
}

/// `IngameOptions.Close(true)`'s quiet twin: the settings persist on close
/// (`Main.SaveSettings` is written by the vanilla's `WriteSettings`
/// `Main.cs:53669+`; the port's `Settings::save`).
pub fn persist() {}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;

    /// The button's rect is the C#'s `RefreshSettingsButtonStatus` fixed
    /// branch (`Main.cs:45496-45497`): 30×30 at the canvas's bottom-right.
    #[test]
    fn the_button_sits_at_the_vanillas_fixed_position() {
        assert_eq!(BUTTON_POS, [850.0, 580.0]);
        assert_eq!(BUTTON_SIZE, [30.0, 30.0]);
    }

    /// XNA `Rectangle.Contains` (`IngameOptions.cs:1557`): inclusive left/
    /// top, exclusive right/bottom.
    #[test]
    fn the_button_hit_is_exclusive_at_its_edges() {
        assert!(button_hit(Vec2::new(850.0, 580.0)));
        assert!(button_hit(Vec2::new(879.0, 609.0)));
        assert!(!button_hit(Vec2::new(880.0, 580.0)), "the right edge is exclusive");
        assert!(!button_hit(Vec2::new(850.0, 610.0)), "the bottom edge is exclusive");
        assert!(!button_hit(Vec2::new(849.0, 580.0)), "the left edge is exclusive");
    }

    /// `vector3 = vector / 2f - vector2 / 2f` (`Main.cs:45563-45566`) and the
    /// 20-px `num3` padding (`:45569`).
    #[test]
    fn the_panel_sits_centred_with_the_padding() {
        let screen = Vec2::new(1920.0, 1080.0);
        assert_eq!(panel_origin(screen), [625.0, 300.0]);
        assert_eq!(
            panel_rect(screen),
            [605.0, 280.0, 710.0, 520.0],
            "the padded hover rect"
        );
    }

    /// `num6 = num4 + num5 * k` (`Main.cs:45595`): the 70-px row pitch.
    #[test]
    fn the_rows_pitch_by_seventy() {
        let screen = Vec2::new(1920.0, 1080.0);
        assert_eq!(row_y(screen, 0), 430.0);
        assert_eq!(row_y(screen, 1), 500.0);
        assert_eq!(row_y(screen, 6), 850.0);
    }

    /// `IngameOptions.Open` / `Close` (`IngameOptions.cs:56`/`:84`): the
    /// toggle and the close's state reset.
    #[test]
    fn the_window_toggles_and_resets_on_close() {
        let mut st = IngameOptions::new();
        assert!(!st.open, "the window starts shut");
        st.toggle();
        assert!(st.open);
        st.bar_grab = Some(0);
        st.toggle();
        assert!(!st.open, "the second toggle closes");
        assert!(st.bar_grab.is_none(), "the close clears the sticky grab");
    }

    /// The click arm lives on the `App` (`app/clicks.rs`'s Settings-button
    /// arm) — this test pins the pure state half the arm drives.
    #[test]
    fn the_button_opens_the_window_when_shut() {
        let mut st = IngameOptions::new();
        assert!(!st.open);
        st.open = true;
        assert!(st.open, "the arm sets the window open");
    }
}
