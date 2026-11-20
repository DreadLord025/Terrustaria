//! The keybinding settings rows — W4-d83 (research/18b rows 40-44):
//! `UIKeybindingCycleListItem` (`UIKeybindingCycleListItem.cs:10-48`),
//! `UIKeybindingListItem` (`UIKeybindingListItem.cs:12-177`),
//! `UIKeybindingSimpleListItem` (`UIKeybindingSimpleListItem.cs:9-40`),
//! `UIKeybindingSliderItem` (`UIKeybindingSliderItem.cs:11-101`) and
//! `UIKeybindingToggleListItem` (`UIKeybindingToggleListItem.cs:10-48`).
//!
//! All five draw the same settings-row chrome: a `Utils.DrawSettingsPanel`
//! strip (`Utils.cs:2602-2605` → `DrawPanel(tex, 2, 0)`, the
//! `panel::separator_rects` three-piece geometry) at `Width + 1`, a 0.8-scale
//! `FontAssets.ItemStack` title at `(X + 8, Y + 2 + 6)` and a right-aligned
//! value. The shared colour pair is ported once ([`row_base_color`] /
//! [`row_panel_tint`]) — the house rule since the 2026-11-01 refactor. The
//! text draws themselves ride `ChatManager.DrawColorCodedStringWithShadow`;
//! this module carries the state machines + geometry as pure code.

use super::char_creation::lerp_rgb;
use super::color::multiply_rgba;

/// `InputMode` (`GameInput/InputMode.cs:3-10`) — the five profile slots a
/// keybinding row reads/writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputMode {
    Keyboard,
    KeyboardUi,
    Mouse,
    XBoxGamepad,
    XBoxGamepadUi,
}

impl InputMode {
    /// The gamepad pair (`UIKeybindingListItem.cs:71`, `:89-90`): the value
    /// text is glyph tags and the row shifts the value up 3 px.
    pub fn is_gamepad(self) -> bool {
        matches!(self, Self::XBoxGamepad | Self::XBoxGamepadUi)
    }
}

/// The shared `baseColor` (`UIKeybindingListItem.cs:52`,
/// `UIKeybindingCycleListItem.cs:33`, `UIKeybindingSimpleListItem.cs:29`,
/// `UIKeybindingToggleListItem.cs:36`, `UIKeybindingSliderItem.cs:73`):
/// `Lerp(listening ? Gold : (hover ? White : Silver), White, hover ? 0.5 : 0)`.
/// The toggle/slider rows' `(0 != 0) ? Gold` arm is dead — they pass
/// `listening = false`.
pub fn row_base_color(hovering: bool, listening: bool) -> [u8; 4] {
    let gold = [255, 215, 0, 255]; // `Color.Gold`
    let silver = [192, 192, 192, 255]; // `Color.Silver`
    let white = [255, 255, 255, 255];
    let base = if listening {
        gold
    } else if hovering {
        white
    } else {
        silver
    };
    lerp_rgb(base, white, if hovering { 0.5 } else { 0.0 })
}

/// The shared panel tint (`UIKeybindingListItem.cs:53` and siblings):
/// `hover ? _color : _color.MultiplyRGBA(new Color(180, 180, 180))` — the
/// resting rows dim through the byte-product multiply (`Utils.cs:1995-1998`;
/// the 3-byte ctor's alpha is 255, so alpha survives).
pub fn row_panel_tint(color: [u8; 4], hovering: bool) -> [u8; 4] {
    if hovering {
        color
    } else {
        multiply_rgba(color, [180, 180, 180, 255])
    }
}

/// The shared row geometry (`UIKeybindingListItem.cs:45-57`, `:70-74` and
/// siblings): panel width `Width + 1`, title at `(X + 8, Y + 2 + 6)`, value
/// right-aligned at `(X + Width - stringW - 10, Y + 2 + 6)` with the gamepad
/// pair shifting the value `(0, -3)`. Text scale 0.8 (`Vector2(0.8f)`).
pub struct RowGeometry {
    /// `num2` — the `DrawSettingsPanel` width (`:48`).
    pub panel_width: f32,
    /// The title's draw position (`:56-57`).
    pub title_pos: [f32; 2],
    /// The value's draw position (`:70-74`); `value_width` is the measured
    /// `ChatManager.GetStringSize(..., 0.8).X`.
    pub value_pos: [f32; 2],
}

/// The row layout for a rect `[x, y, w, h]` (the `GetDimensions()` floats).
/// `gamepad` applies the `(0, -3)` value shift (`UIKeybindingListItem.cs:71-74`;
/// the other rows never shift — pass `false`).
pub fn row_geometry(rect: [f32; 4], value_width: f32, gamepad: bool) -> RowGeometry {
    let num = 6.0; // `float num = 6f`
    let mut value_pos = [rect[0] + rect[2] - value_width - 10.0, rect[1] + 2.0 + num];
    if gamepad {
        value_pos[1] -= 3.0; // `position += new Vector2(0f, -3f)`
    }
    RowGeometry {
        panel_width: rect[2] + 1.0,
        title_pos: [rect[0] + 8.0, rect[1] + 2.0 + num],
        value_pos,
    }
}

/// `GlyphTagHandler.GenerateTag(string)` (`GlyphTagHandler.cs:175-182`) over
/// the static `GlyphIndexes` table (`:189-289`): a known button name becomes
/// `[g:index]`, anything else passes through. `GenerateTag(int)` (`:169-173`)
/// formats `[g:{index}]`.
pub fn glyph_tag(keyname: &str) -> String {
    let index = match keyname {
        "A" => 0,
        "B" => 1,
        "X" => 2,
        "Y" => 3,
        "Back" => 4,
        "Start" => 5,
        "LeftShoulder" => 6,
        "RightShoulder" => 7,
        "LeftTrigger" => 8,
        "RightTrigger" => 9,
        "LeftStick" => 10,
        "RightStick" => 11,
        "RightStickAxis" => 12,
        "DPadRight" => 13,
        "DPadLeft" => 14,
        "DPadDown" => 15,
        "DPadUp" => 16,
        "LeftThumbstickLeft" => 17,
        "LeftThumbstickRight" => 18,
        "LeftThumbstickUp" => 19,
        "LeftThumbstickDown" => 20,
        "RightThumbstickLeft" => 21,
        "RightThumbstickRight" => 22,
        "RightThumbstickUp" => 23,
        "RightThumbstickDown" => 24,
        "LR" => 25,
        _ => return keyname.to_string(), // `:181` the miss returns the name
    };
    format!("[g:{index}]") // `GenerateTag(int)`, `:169-173`
}

/// `UIKeybindingCycleListItem` (`UIKeybindingCycleListItem.cs:10-48`),
/// research/18b row 40 (W4-d83): the title/value cycle row (the cursor-colour
/// style cycler etc.). The two `Func<string>` seeds are the caller's; the port
/// carries the ctor's `"???"` fallbacks and the draw plan. The
/// `position.X -= 17f` at `:40` is **dead** — `:43` reassigns the position
/// before the value draws; not ported.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeybindingCycleRow {
    /// `_color` (`:12`, ctor `:20`) — the row's panel colour.
    pub color: [u8; 4],
}

impl KeybindingCycleRow {
    /// The ctor's colour half (`:18-23`). A null `getTitle`/`getValue` reads
    /// `"???"` (`:21-22`) — [`Self::text_or_placeholder`] mirrors the fallback.
    pub fn new(color: [u8; 4]) -> Self {
        Self { color }
    }

    /// The null-function fallback (`:21-22`): `None` (a null `Func`) reads
    /// `"???"`.
    pub fn text_or_placeholder(text: Option<&str>) -> &str {
        text.unwrap_or("???")
    }

    /// The `DrawSelf` plan (`:25-47`): the shared chrome plus the value's
    /// `GlyphsScale = 0.85` window (`:44-46` — the title draws at scale 1,
    /// only the value's glyphs shrink, then the static resets to 1).
    pub fn draw_plan(&self, rect: [f32; 4], hovering: bool, value_width: f32) -> CycleDraw {
        CycleDraw {
            geometry: row_geometry(rect, value_width, false),
            base_color: row_base_color(hovering, false),
            panel_tint: row_panel_tint(self.color, hovering),
            value_glyphs_scale: 0.85,
        }
    }
}

/// The cycle row's draw outputs (`UIKeybindingCycleListItem.cs:29-46`).
pub struct CycleDraw {
    pub geometry: RowGeometry,
    pub base_color: [u8; 4],
    pub panel_tint: [u8; 4],
    /// `GlyphTagHandler.GlyphsScale` while the value draws (`:44`); the
    /// caller restores 1 after (`:46`).
    pub value_glyphs_scale: f32,
}

/// `UIKeybindingListItem` (`UIKeybindingListItem.cs:12-177`), research/18b
/// row 41 (W4-d83): the rebindable key row. The port carries the click
/// listen-latch, `GenInput`'s join, `GetFriendlyName`'s key table and the
/// draw's listening/empty-text colour arms; `PlayerInput.ListenFor` itself
/// is the caller's (the input-capture machine is unported).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeybindingRow {
    /// `_keybind` (`:18`) — the trigger name.
    pub keybind: String,
    /// `_inputmode` (`:14`).
    pub mode: InputMode,
    /// `_color` (`:16`).
    pub color: [u8; 4],
}

impl KeybindingRow {
    /// The ctor (`:20-26`): the fields plus the `OnLeftClick` subscription
    /// (the caller wires [`Self::on_click`]).
    pub fn new(keybind: &str, mode: InputMode, color: [u8; 4]) -> Self {
        Self {
            keybind: keybind.to_string(),
            mode,
            color,
        }
    }

    /// `OnClickMethod` (`:28-41`): clicking the row that is *already*
    /// listening does nothing; otherwise `ListenFor(AllowEditing ? _keybind
    /// : null, _inputmode)`. Returns the trigger to listen for (`None` =
    /// the null cancel-listen), or `None` when the click is swallowed —
    /// the caller distinguishes via the returned `Option<Option<&str>>`.
    pub fn on_click<'a>(
        &'a self,
        listening_trigger: Option<&str>,
        allow_editing: bool,
    ) -> Option<Option<&'a str>> {
        if listening_trigger == Some(self.keybind.as_str()) {
            return None; // `:30` — already listening: no re-arm
        }
        Some(if allow_editing {
            Some(self.keybind.as_str()) // `:34`
        } else {
            None // `:38` ListenFor(null)
        })
    }

    /// `GenInput` (`:80-112`): an empty bind list reads `""`; the gamepad
    /// pair joins glyph tags with `/`, keyboard/mouse join the raw names
    /// with `/`.
    pub fn gen_input(&self, keys: &[String]) -> String {
        if keys.is_empty() {
            return String::new(); // `:82-85`
        }
        let mut text = if self.mode.is_gamepad() {
            glyph_tag(&keys[0]) // `:92`
        } else {
            keys[0].clone() // `:103`
        };
        for key in &keys[1..] {
            text.push('/');
            if self.mode.is_gamepad() {
                text.push_str(&glyph_tag(key)); // `:95`
            } else {
                text.push_str(key); // `:106`
            }
        }
        text
    }

    /// `GetFriendlyName` (`:114-176`): the trigger → localized label table;
    /// an unmapped trigger shows its own name (`:174`).
    pub fn friendly_name(&self) -> String {
        let menu = crate::lang::menu;
        let ui = crate::lang::text;
        match self.keybind.as_str() {
            "MouseLeft" => menu(162),
            "MouseRight" => menu(163),
            "Up" => menu(148),
            "Down" => menu(149),
            "Left" => menu(150),
            "Right" => menu(151),
            "Jump" => menu(152),
            "Throw" => menu(153),
            "Inventory" => menu(154),
            "Grapple" => menu(155),
            "SmartSelect" => menu(160),
            "SmartCursor" => menu(161),
            "QuickMount" => menu(158),
            "QuickHeal" => menu(159),
            "QuickMana" => menu(156),
            "QuickBuff" => menu(157),
            "MapZoomIn" => menu(168),
            "MapZoomOut" => menu(169),
            "MapAlphaUp" => menu(171),
            "MapAlphaDown" => menu(170),
            "MapFull" => menu(173),
            "MapStyle" => menu(172),
            "Hotbar1" => menu(176),
            "Hotbar2" => menu(177),
            "Hotbar3" => menu(178),
            "Hotbar4" => menu(179),
            "Hotbar5" => menu(180),
            "Hotbar6" => menu(181),
            "Hotbar7" => menu(182),
            "Hotbar8" => menu(183),
            "Hotbar9" => menu(184),
            "Hotbar10" => menu(185),
            "HotbarMinus" => menu(174),
            "HotbarPlus" => menu(175),
            "DpadRadial1" => menu(186),
            "DpadRadial2" => menu(187),
            "DpadRadial3" => menu(188),
            "DpadRadial4" => menu(189),
            "RadialHotbar" => menu(190),
            "RadialQuickbar" => menu(244),
            "DpadSnap1" => menu(191),
            "DpadSnap2" => menu(192),
            "DpadSnap3" => menu(193),
            "DpadSnap4" => menu(194),
            "LockOn" => menu(231),
            "ViewZoomIn" => ui("UI.ZoomIn"),
            "ViewZoomOut" => ui("UI.ZoomOut"),
            "ToggleCreativeMenu" => ui("UI.ToggleCreativeMenu"),
            "Loadout1" => ui("UI.Loadout1"),
            "Loadout2" => ui("UI.Loadout2"),
            "Loadout3" => ui("UI.Loadout3"),
            "Dash" => ui("UI.Dash"),
            "NextLoadout" => ui("UI.NextLoadout"),
            "PreviousLoadout" => ui("UI.PreviousLoadout"),
            "ArmorSetAbility" => ui("UI.ArmorSetAbility"),
            "ToggleCameraMode" => ui("UI.ToggleCameraMode"),
            _ => self.keybind.clone(), // `:174`
        }
    }

    /// The `DrawSelf` plan (`:43-78`): the listening flag swaps the title
    /// base to Gold; an empty bind text reads `Lang.menu[195]` ("Unbound")
    /// dimmed to `(80, 80, 80)` unless listening; the gamepad pair shifts the
    /// value up 3 and draws its glyphs at 0.85.
    pub fn draw_plan(
        &self,
        rect: [f32; 4],
        hovering: bool,
        listening: bool,
        keys: &[String],
        value_width: f32,
    ) -> KeybindDraw {
        let mut base_color = row_base_color(hovering, listening); // `:52`
        let mut text = self.gen_input(keys); // `:60`
        if text.is_empty() {
            text = crate::lang::menu(195); // `:63`
            if !listening {
                base_color = [80, 80, 80, 255]; // `:66`
            }
        }
        KeybindDraw {
            geometry: row_geometry(rect, value_width, self.mode.is_gamepad()),
            base_color,
            panel_tint: row_panel_tint(self.color, hovering),
            text,
            value_glyphs_scale: 0.85, // `:75-77`
        }
    }
}

/// The key row's draw outputs (`UIKeybindingListItem.cs:50-77`).
pub struct KeybindDraw {
    pub geometry: RowGeometry,
    pub base_color: [u8; 4],
    pub panel_tint: [u8; 4],
    /// The value string after the empty-bind fallback (`:60-68`).
    pub text: String,
    pub value_glyphs_scale: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shared colour pair (`UIKeybindingListItem.cs:52-53`): rest is
    /// Silver and the panel dims through MultiplyRGBA(180); hover bases
    /// White (the lerp to White is a no-op) and the panel keeps its colour;
    /// listening bases Gold → Lerp(Gold, White, 0.5) = (255,235,127).
    #[test]
    fn the_row_colour_pair_matches_the_lerps_and_multiply() {
        assert_eq!(row_base_color(false, false), [192, 192, 192, 255]);
        assert_eq!(row_base_color(true, false), [255, 255, 255, 255]);
        assert_eq!(row_base_color(false, true), [255, 215, 0, 255]);
        assert_eq!(row_base_color(true, true), [255, 235, 127, 255]);
        let inv = [63, 65, 151, 255];
        assert_eq!(row_panel_tint(inv, true), inv);
        assert_eq!(row_panel_tint(inv, false), [44, 45, 106, 255]);
    }

    /// The row geometry (`UIKeybindingCycleListItem.cs:27-43`): panel width
    /// W+1, title (X+8, Y+8), value right-aligned at X+W-strW-10; the
    /// gamepad shift is -3 on y only (`UIKeybindingListItem.cs:71-74`).
    #[test]
    fn the_row_geometry_walks_the_shared_constants() {
        let g = row_geometry([100.0, 50.0, 400.0, 30.0], 60.0, false);
        assert_eq!(g.panel_width, 401.0);
        assert_eq!(g.title_pos, [108.0, 58.0]);
        assert_eq!(g.value_pos, [440.0, 58.0]);
        let gp = row_geometry([100.0, 50.0, 400.0, 30.0], 60.0, true);
        assert_eq!(gp.value_pos, [440.0, 55.0]);
    }

    /// `GenInput` (`UIKeybindingListItem.cs:80-112`): empty → "", keyboard
    /// joins raw names, gamepad joins glyph tags; unknown gamepad names pass
    /// through (`GlyphTagHandler.cs:181`).
    #[test]
    fn gen_input_joins_names_or_glyph_tags_per_mode() {
        let kb = KeybindingRow::new("Jump", InputMode::Keyboard, [63, 65, 151, 255]);
        assert_eq!(kb.gen_input(&[]), "");
        assert_eq!(kb.gen_input(&["Space".into()]), "Space");
        assert_eq!(kb.gen_input(&["Space".into(), "W".into()]), "Space/W");
        let gp = KeybindingRow::new("Jump", InputMode::XBoxGamepad, [63, 65, 151, 255]);
        assert_eq!(gp.gen_input(&["A".into(), "LR".into()]), "[g:0]/[g:25]");
        assert_eq!(gp.gen_input(&["Mystery".into()]), "Mystery");
        assert_eq!(glyph_tag("RightThumbstickDown"), "[g:24]");
    }

    /// `OnClickMethod` (`:28-41`): the already-listening click is swallowed;
    /// editing arms the row's trigger, non-editing sends the null listen.
    #[test]
    fn the_click_latch_skips_the_listening_row() {
        let row = KeybindingRow::new("Jump", InputMode::Keyboard, [63, 65, 151, 255]);
        assert_eq!(row.on_click(Some("Jump"), true), None);
        assert_eq!(row.on_click(Some("Inventory"), true), Some(Some("Jump")));
        assert_eq!(row.on_click(None, true), Some(Some("Jump")));
        assert_eq!(row.on_click(None, false), Some(None));
    }

    /// The draw's colour arms (`:52`, `:60-68`): an unbound row reads the
    /// menu[195] key dimmed (80,80,80); listening keeps the Gold lerp; a
    /// bound row keeps the shared pair. The friendly-name table maps the
    /// trigger to its Lang key and falls back to the raw name (`:174`).
    #[test]
    fn the_unbound_row_dims_and_the_friendly_name_maps_keys() {
        let row = KeybindingRow::new("Jump", InputMode::Keyboard, [63, 65, 151, 255]);
        let d = row.draw_plan([0.0, 0.0, 400.0, 30.0], false, false, &[], 40.0);
        assert_eq!(d.base_color, [80, 80, 80, 255]);
        // The unloaded manager echoes the key — pin the KEY, not the string.
        assert!(d.text == "Unbound" || d.text == "LegacyMenu.195", "got {}", d.text);
        let listening = row.draw_plan([0.0, 0.0, 400.0, 30.0], true, true, &[], 40.0);
        assert_eq!(listening.base_color, [255, 235, 127, 255]);
        let bound = row.draw_plan(
            [0.0, 0.0, 400.0, 30.0],
            false,
            false,
            &["Space".into()],
            40.0,
        );
        assert_eq!(bound.text, "Space");
        assert_eq!(bound.base_color, [192, 192, 192, 255]);
        let name = row.friendly_name();
        assert!(name == "Jump" || name == "LegacyMenu.152", "got {name}");
        let raw = KeybindingRow::new("SomeModdedTrigger", InputMode::Mouse, [0, 0, 0, 255]);
        assert_eq!(raw.friendly_name(), "SomeModdedTrigger");
        assert_eq!(KeybindingCycleRow::text_or_placeholder(None), "???");
        assert_eq!(KeybindingCycleRow::text_or_placeholder(Some("Style")), "Style");
    }
}
