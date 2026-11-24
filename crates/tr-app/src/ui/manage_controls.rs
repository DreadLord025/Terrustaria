//! The manage-controls screen — `UIManageControls`
//! (`GameContent/UI/States/UIManageControls.cs:20-1061`), W4-d86.
//!
//! The screen's own state machine: the keyboard/gamepad × gameplay/UI tab
//! pair (`OnKeyboard`/`OnGameplay`, `:77-79`), the four bind-panel groups
//! (`AssembleBindPanels` `:269-315` + the row pairing `CreateElementGroup`
//! `:388-433`), the 22 special-control rows (`CreatePanel` `:435-692`), the
//! dpad snap/radial toggle pair (`SnapButtonClick`/`RadialButtonClick`
//! `:740-810`), the border hover colours (`:828-890`) and the profile/glyph
//! button labels. The profile DATA the rows edit lives in
//! [`crate::input_profiles`] (`PlayerInputProfile`/`ProfileRegistry`); the
//! five `UIKeybinding*ListItem` row ELEMENTS are d83's `ui/keybinding_rows.rs`
//! (research/18b rows 40-44) — this module cites their behaviour and stays
//! self-contained until the merge; the merger keeps d83's row module.
//!
//! Deliberate gaps (code-only lane, ANALYSIS "The manage-controls screen
//! (W4-d86)"): the sprite draw + `UIList` scrolling ride the shared UI seams
//! at the wiring follow-up (the layout below is the state half only);
//! `SetupGamepadPoints` (`:949-1048`), `OnActivate`'s `ChangePoint(3002)`
//! (`:706-709`) and the `ForceMoveTo` static (`:69`, `:1043-1047`) are the
//! Stage-1 gamepad exclusion; `GoBackClick`'s `menuMode = 1127` +
//! `IngameFancyUI.Close()` (`:937-941`) returns the caller's routing value —
//! 1127 is the legacy Controls menu (`Main.cs:50638-50647`).

#![allow(dead_code)] // wiring deferred — the front-end mount + input_map feed are the follow-up

use crate::input_profiles::{
    cycle_control_off_on_click_hold, cycle_dash_control, cycle_glyph_style,
    handle_slider_horizontal_input, ButtonControlMode, DashPreference, InputMode, ProfileRegistry,
};
use crate::ui::char_creation::lerp_rgb;
use crate::ui::color;
use crate::ui::uistate::{self, Style, FANCY_UI_FAT_BUTTON_MOUSE_OVER, OUR_FAVORITE_COLOR};
use tr_core::scene::get_lerp_value;

/// The 22 special-control row ids (`UIManageControls.SpecialControls`,
/// `:22-67`) — the `sp1`..`sp22` strings the bind lists carry.
pub const SPECIAL_CONTROLS: [&str; 22] = [
    "sp1", "sp2", "sp3", "sp4", "sp5", "sp6", "sp7", "sp8", "sp9", "sp10",
    "sp11", "sp12", "sp13", "sp14", "sp15", "sp16", "sp17", "sp18", "sp19", "sp20",
    "sp21", "sp22",
];

/// `_BindingsFullLine` (static ctor `:1053-1058`) — the rows that take the
/// whole line even when the pairing pass would halve them.
pub const BINDINGS_FULL_LINE: [&str; 26] = [
    "Throw", "Inventory", "RadialHotbar", "RadialQuickbar", "LockOn", "ToggleCreativeMenu", "Loadout1", "Loadout2", "Loadout3", "ToggleCameraMode",
    "sp3", "sp4", "sp5", "sp6", "sp7", "sp8", "sp18", "sp19", "sp9", "sp10",
    "sp11", "sp12", "sp13", "ArmorSetAbility", "sp22", "sp21",
];

/// `_BindingsHalfSingleLine` (`:1059`) — half-width but ALONE on their line
/// (the reset rows + the dash toggle); checked BEFORE the full-line list, so
/// `sp9`..`sp13` land here despite also being in `_BindingsFullLine`.
pub const BINDINGS_HALF_SINGLE_LINE: [&str; 6] = ["sp9", "sp10", "sp11", "sp12", "sp13", "sp20"];

/// `AssembleBindPanels`' six bind lists (`:271-294`), in source order.
/// The keyboard gameplay list (`bindings`, `:271-276`) — also the
/// keyboard-UI list's source (`:306`).
pub const GAMEPLAY_BINDS_KEYBOARD: [&str; 30] = [
    "MouseLeft", "MouseRight", "Up", "Down", "Left", "Right", "Jump", "Grapple", "SmartSelect", "SmartCursor",
    "QuickMount", "QuickHeal", "QuickMana", "QuickBuff", "Throw", "Inventory", "ToggleCreativeMenu", "ViewZoomIn", "ViewZoomOut", "Loadout1",
    "Loadout2", "Loadout3", "NextLoadout", "PreviousLoadout", "ToggleCameraMode", "ArmorSetAbility", "Dash", "sp20", "sp21", "sp9",
];

/// The gamepad gameplay list (`bindings2`, `:277-282`) — LockOn replaces
/// QuickBuff/Throw's slot order, sp22 replaces sp21.
pub const GAMEPLAY_BINDS_GAMEPAD: [&str; 28] = [
    "MouseLeft", "MouseRight", "Up", "Down", "Left", "Right", "Jump", "Grapple", "SmartSelect", "SmartCursor",
    "QuickMount", "QuickHeal", "QuickMana", "QuickBuff", "LockOn", "Throw", "Inventory", "Loadout1", "Loadout2", "Loadout3",
    "NextLoadout", "PreviousLoadout", "ToggleCameraMode", "ArmorSetAbility", "Dash", "sp20", "sp22", "sp9",
];

/// The hotbar list (`bindings3`, `:283-287`).
pub const HOTBAR_BINDS: [&str; 13] = [
    "HotbarMinus", "HotbarPlus", "Hotbar1", "Hotbar2", "Hotbar3", "Hotbar4", "Hotbar5", "Hotbar6", "Hotbar7", "Hotbar8",
    "Hotbar9", "Hotbar10", "sp10",
];

/// The map list (`bindings4`, `:288`).
pub const MAP_BINDS: [&str; 7] = ["MapZoomIn", "MapZoomOut", "MapAlphaUp", "MapAlphaDown", "MapFull", "MapStyle", "sp11"];

/// The gamepad radial/dpad list (`bindings5`, `:289`).
pub const GAMEPAD_BINDS: [&str; 5] = ["sp1", "sp2", "RadialHotbar", "RadialQuickbar", "sp12"];

/// The gamepad advanced list (`bindings6`, `:290-294`).
pub const GAMEPAD_ADVANCED_BINDS: [&str; 13] = [
    "sp3", "sp4", "sp5", "sp6", "sp7", "sp8", "sp14", "sp15", "sp16", "sp17",
    "sp18", "sp19", "sp13",
];

/// `CreateBindingGroup`'s per-group tint (`:337-354`): the panel colour
/// `new Color(33, 43, 79) * 0.8f` lerped 0.18 toward the group's accent.
/// The XNA named colours: Green (0,128,0), Goldenrod (218,165,32),
/// HotPink (255,105,180), Indigo (75,0,130), Turquoise (64,224,208).
pub fn group_panel_color(element_index: usize) -> [u8; 4] {
    let base = color::mul([33, 43, 79, 255], 0.8);
    let accent = match element_index {
        0 => [0, 128, 0, 255],
        1 => [218, 165, 32, 255],
        2 => [255, 105, 180, 255],
        3 => [75, 0, 130, 255],
        4 => [64, 224, 208, 255],
        _ => return base,
    };
    lerp_rgb(base, accent, 0.18)
}

/// `CreateElementGroup`'s row colour (`:355-356`): the group tint AFTER the
/// lerp, `MultiplyRGBA(new Color(111, 111, 111))` — XNA's per-channel
/// `(byte)((a * b) / 255)` integer multiply (the same op `slot.rs`'s shadow
/// tint documents for `MultiplyRGBA(Color.Black)`).
pub fn row_color(element_index: usize) -> [u8; 4] {
    let c = group_panel_color(element_index);
    let f = |x: u8| ((x as u32 * 111) / 255) as u8;
    [f(c[0]), f(c[1]), f(c[2]), f(c[3])]
}

/// `CreateBindingGroup`'s title key (`:357-375`): group 0 splits gameplay
/// (`Lang.menu[164]`) vs menu (`Lang.menu[243]`) by mode; the rest are fixed.
pub fn group_title_menu_index(element_index: usize, mode: InputMode) -> Option<usize> {
    match element_index {
        0 => Some(if matches!(mode, InputMode::Keyboard | InputMode::XBoxGamepad) { 164 } else { 243 }),
        1 => Some(165),
        2 => Some(166),
        3 => Some(167),
        4 => Some(198),
        _ => None,
    }
}

/// One placed row of a bind group — the `CreateElementGroup` output
/// (`:388-433`): the bind id, the width style and the snap-point name.
/// Thin rows pair two-per-line (`Width.Set(-5f, 0.5f)`, second `HAlign=1`);
/// half-single rows sit alone at half width centred; full rows take the line.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlacedRow {
    pub bind: &'static str,
    pub width: Style,
    pub h_align: f32,
    /// `"Wide"` for half-single/full rows, `"Thin"` for the paired ones
    /// (`:404`, `:413`, `:420`, `:429`).
    pub snap: &'static str,
}

/// `CreateElementGroup` (`:388-433`) as data: walk the bind list, emit the
/// rows with their pairing. The `i++` inside the thin arm consumes the
/// second row of the pair; an odd tail emits a single thin row.
pub fn place_rows(bindings: &[&'static str]) -> Vec<PlacedRow> {
    let mut rows = Vec::new();
    let mut i = 0;
    while i < bindings.len() {
        let bind = bindings[i];
        if BINDINGS_HALF_SINGLE_LINE.contains(&bind) {
            rows.push(PlacedRow { bind, width: Style::new(0.0, 0.5), h_align: 0.5, snap: "Wide" });
            i += 1;
            continue;
        }
        if BINDINGS_FULL_LINE.contains(&bind) {
            rows.push(PlacedRow { bind, width: Style::new(0.0, 1.0), h_align: 0.0, snap: "Wide" });
            i += 1;
            continue;
        }
        rows.push(PlacedRow { bind, width: Style::new(-5.0, 0.5), h_align: 0.0, snap: "Thin" });
        i += 1;
        if i < bindings.len() {
            rows.push(PlacedRow { bind: bindings[i], width: Style::new(-5.0, 0.5), h_align: 1.0, snap: "Thin" });
            i += 1;
        }
    }
    rows
}

/// One assembled bind group — `CreateBindingGroup` (`:317-386`): the mode,
/// the placed rows and the group's resolved height. The C# builds a
/// `UISortableElement` of 2000 px, recalculates, then sets
/// `Height = GetTotalHeight() + 30 + 16` (`:382-384`); `GetTotalHeight` is
/// `UIList.RecalculateChildren`'s sum (`UIList.cs:116-128`): every row is
/// 30 px (`:395`), `ListPadding = 5` (`:334`), and the padding is added
/// AFTER every item including the last — `rows * 35` exactly.
#[derive(Clone, Debug)]
pub struct BindGroup {
    pub element_index: usize,
    pub mode: InputMode,
    pub rows: Vec<PlacedRow>,
}

impl BindGroup {
    /// The group's resolved outer height (`:384`): total list height + the
    /// 30 px title plate + the 16 px panel inset. `UIList.RecalculateChildren`
    /// (`UIList.cs:119-127`) adds `ListPadding` after EVERY item — except a
    /// single-item list, where `num2 = 0` (`:122`).
    pub fn height(&self) -> f32 {
        // One 30-px sortable row per PAIR of thin rows / per wide row
        // (`:393-397` — the sortable wrapper is created once per loop pass).
        let lines = self.rows.iter().filter(|r| r.h_align != 1.0).count();
        let padding = if lines == 1 { 0.0 } else { 5.0 };
        let total = lines as f32 * (30.0 + padding);
        total + 30.0 + 16.0
    }
}

/// `AssembleBindPanels` (`:269-315`): the four mode lists. Keyboard and
/// keyboard-UI carry groups 0/1/2; the gamepads add 3 (radial) and 4
/// (advanced). Group 0's bind list differs by device, not by UI-vs-gameplay.
pub fn assemble_groups(mode: InputMode) -> Vec<BindGroup> {
    let gameplay: &[&'static str] = match mode {
        InputMode::Keyboard | InputMode::KeyboardUi => &GAMEPLAY_BINDS_KEYBOARD,
        _ => &GAMEPLAY_BINDS_GAMEPAD,
    };
    let mut groups = vec![
        BindGroup { element_index: 0, mode, rows: place_rows(gameplay) },
        BindGroup { element_index: 1, mode, rows: place_rows(&MAP_BINDS) },
        BindGroup { element_index: 2, mode, rows: place_rows(&HOTBAR_BINDS) },
    ];
    if mode.is_gamepad() {
        groups.push(BindGroup { element_index: 3, mode, rows: place_rows(&GAMEPAD_BINDS) });
        groups.push(BindGroup { element_index: 4, mode, rows: place_rows(&GAMEPAD_ADVANCED_BINDS) });
    }
    groups
}

/// The screen state — `UIManageControls`' instance fields (`:77-123`) minus
/// the UIElement tree (the draw seam's half). `OnKeyboard`/`OnGameplay`
/// default true (`:77-79`); the profile registry + the dash preference +
/// the glyph style are the statics the rows edit (`PlayerInput.Profiles`,
/// `Player.Settings.DashControl`, `GlyphTagHandler.GlyphStyle`).
#[derive(Clone, Debug)]
pub struct ManageControlsState {
    pub on_keyboard: bool,
    pub on_gameplay: bool,
    pub registry: ProfileRegistry,
    /// `Player.Settings.DashControl` (`Player.cs:411`, default AllowDoubleTap).
    pub dash_control: DashPreference,
    /// `GlyphTagHandler.GlyphStyle` (`GlyphTagHandler.cs:145-147`, default -1).
    pub glyph_style: i32,
    /// `UILinkPointNavigator.Shortcuts.INV_MOVE_OPTION_CD` — the sp19
    /// gamepad-step cooldown (`:624-637`). Gamepad-only; 0 at rest.
    pub inv_move_option_cd: i32,
    /// `OnActivate`'s container styles (`:694-705`): gameMenu → top 220 /
    /// height -220, in-game → top 120 / height -120.
    pub outer_top: Style,
    pub outer_height: Style,
}

impl Default for ManageControlsState {
    fn default() -> Self {
        Self::new()
    }
}

impl ManageControlsState {
    /// The field initializers (`:77-79`) + the `OnInitialize` container
    /// (`:131-136`: width 80% max 600, top 220, height -200) — `OnActivate`
    /// overrides top/height on entry.
    pub fn new() -> Self {
        Self {
            on_keyboard: true,
            on_gameplay: true,
            registry: ProfileRegistry::new(),
            dash_control: DashPreference::AllowDoubleTap,
            glyph_style: crate::input_profiles::DEFAULT_GLYPH_STYLE,
            inv_move_option_cd: 0,
            outer_top: Style::px(220.0),
            outer_height: Style::new(-200.0, 1.0),
        }
    }

    /// `OnActivate` (`:694-705`): the container re-styles by `Main.gameMenu`.
    /// The `ChangePoint(3002)` gamepad arm (`:706-709`) is the Stage-1
    /// exclusion.
    pub fn on_activate(&mut self, game_menu: bool) {
        if game_menu {
            self.outer_top = Style::px(220.0);
            self.outer_height = Style::new(-220.0, 1.0);
        } else {
            self.outer_top = Style::px(120.0);
            self.outer_height = Style::new(-120.0, 1.0);
        }
    }

    /// The outer container's width style (`:132-133`): 80% of the screen,
    /// max 600 px.
    pub fn outer_width(&self) -> (Style, f32) {
        (Style::new(0.0, 0.8), 600.0)
    }

    /// `FillList`'s selection (`:722-738`): the active mode from the two
    /// tab flags. The list itself is rebuilt by [`assemble_groups`].
    pub fn active_mode(&self) -> InputMode {
        match (self.on_keyboard, self.on_gameplay) {
            (true, true) => InputMode::Keyboard,
            (false, true) => InputMode::XBoxGamepad,
            (true, false) => InputMode::KeyboardUi,
            (false, false) => InputMode::XBoxGamepadUi,
        }
    }

    /// `KeyboardButtonClick`/`GamepadButtonClick` (`:812-826`): set the tab
    /// and refill. The frame swaps are the draw seam's half.
    pub fn click_keyboard_tab(&mut self) {
        self.on_keyboard = true;
    }

    pub fn click_gamepad_tab(&mut self) {
        self.on_keyboard = false;
    }

    /// `VsGameplayButtonClick`/`VsMenuButtonClick` (`:852-866`).
    pub fn click_gameplay_tab(&mut self) {
        self.on_gameplay = true;
    }

    pub fn click_menu_tab(&mut self) {
        self.on_gameplay = false;
    }

    /// The keyboard/gamepad border pair (`ManageBorderKeyboardOn/Off` +
    /// `ManageBorderGamepadOn/Off`, `:828-850`) — returns
    /// `(border1, border2)`. Hovering one button paints it
    /// `Main.OurFavoriteColor`; the resting colours are Silver on the
    /// ACTIVE tab, Black on the inactive one.
    pub fn keyboard_borders(&self, hover: Option<bool>) -> ([u8; 4], [u8; 4]) {
        tab_borders(self.on_keyboard, hover)
    }

    /// The gameplay/menu border pair (`:868-890`) — the same rule over
    /// `OnGameplay`.
    pub fn gameplay_borders(&self, hover: Option<bool>) -> ([u8; 4], [u8; 4]) {
        tab_borders(self.on_gameplay, hover)
    }

    /// `profileButtonClick` (`:912-922`) — the registry's cycle.
    pub fn click_profile(&mut self) {
        self.registry.cycle_profile();
    }

    /// `controllerGlyphButtonClick` (`:903-910`).
    pub fn click_glyph_style(&mut self) {
        self.glyph_style = cycle_glyph_style(self.glyph_style);
    }

    /// The glyph button's label key (`controllerGlyphStyle`, `:892-901`).
    pub fn glyph_label_key(&self) -> &'static str {
        crate::input_profiles::glyph_style_key(self.glyph_style)
    }

    /// `GoBackClick` (`:937-941`): `Main.menuMode = 1127` +
    /// `IngameFancyUI.Close()`. The port returns the menu mode for the
    /// caller's flow routing (1127 = the legacy Controls menu,
    /// `Main.cs:50638`).
    pub fn click_back(&self) -> i32 {
        1127
    }

    /// `SnapButtonClick` (sp1, `:740-774`): gated on `AllowEditing`, plays
    /// sound 12 (the returned flag — the caller owns the side effect, the
    /// `world_load.rs` convention). If the Snap family is fully bound in
    /// BOTH gamepad modes, clear it; otherwise clear the Radial family and
    /// bind Snap to the canonical directions in both modes.
    pub fn click_snap_toggle(&mut self) -> bool {
        if !self.registry.current().allow_editing {
            return false;
        }
        let cur = self.registry.current_mut();
        if cur.using_dpad_movekeys() {
            clear_dpad_family(cur, "DpadSnap");
        } else {
            clear_dpad_family(cur, "DpadRadial");
            bind_dpad_family(cur, "DpadSnap");
        }
        true
    }

    /// `RadialButtonClick` (sp2, `:776-810`): the mirror image — the Radial
    /// family toggles, the Snap family is the one cleared on enable.
    pub fn click_radial_toggle(&mut self) -> bool {
        if !self.registry.current().allow_editing {
            return false;
        }
        let cur = self.registry.current_mut();
        if cur.using_dpad_hotbar() {
            clear_dpad_family(cur, "DpadRadial");
        } else {
            clear_dpad_family(cur, "DpadSnap");
            bind_dpad_family(cur, "DpadRadial");
        }
        true
    }

    /// The sp9-sp13 reset rows (`:499-548`): copy from
    /// `OriginalProfiles[GetCopyableProfileName()]` in the row's mode.
    /// Returns false when the row id is not a reset row.
    pub fn click_reset_row(&mut self, sp: &str, mode: InputMode) -> bool {
        let name = self.registry.copyable_profile_name().to_string();
        let source = self.registry.original(&name).clone();
        let cur = self.registry.current_mut();
        match sp {
            "sp9" => cur.copy_gameplay_settings_from(&source, mode),
            "sp10" => cur.copy_hotbar_settings_from(&source, mode),
            "sp11" => cur.copy_map_settings_from(&source, mode),
            "sp12" => cur.copy_gamepad_settings_from(&source, mode),
            "sp13" => cur.copy_gamepad_advanced_settings_from(&source, mode),
            _ => return false,
        }
        true
    }

    /// The sp14-sp17 invert toggles (`:549-596`): `AllowEditing`-gated flips.
    pub fn click_invert_toggle(&mut self, sp: &str) -> bool {
        let cur = self.registry.current_mut();
        if !cur.allow_editing {
            return false;
        }
        match sp {
            "sp14" => cur.left_thumbstick_invert_x = !cur.left_thumbstick_invert_x,
            "sp15" => cur.left_thumbstick_invert_y = !cur.left_thumbstick_invert_y,
            "sp16" => cur.right_thumbstick_invert_x = !cur.right_thumbstick_invert_x,
            "sp17" => cur.right_thumbstick_invert_y = !cur.right_thumbstick_invert_y,
            _ => return false,
        }
        true
    }

    /// The sp20 dash toggle (`:640-655`): the two-state flip, NO
    /// `AllowEditing` gate (it edits `Player.Settings`, not the profile).
    pub fn click_dash_toggle(&mut self) {
        self.dash_control = cycle_dash_control(self.dash_control);
    }

    /// The sp21/sp22 pan-control cycles (`:657-688`):
    /// `Utils.CycleControlControl_OffOnClickHold`, no editing gate.
    pub fn click_pan_cycle(&mut self, sp: &str) -> bool {
        let cur = self.registry.current_mut();
        match sp {
            "sp21" => cur.pan_control_keyboard = cycle_control_off_on_click_hold(cur.pan_control_keyboard),
            "sp22" => cur.pan_control_gamepad = cycle_control_off_on_click_hold(cur.pan_control_gamepad),
            _ => return false,
        }
        true
    }

    /// The sp21/sp22 label key (`:659-666`/`:675-682`).
    pub fn pan_label_key(mode: ButtonControlMode) -> &'static str {
        match mode {
            ButtonControlMode::OnAlways => "UI.PanControlMode_OnAlways",
            ButtonControlMode::OffAlways => "UI.PanControlMode_OffAlways",
            ButtonControlMode::Hold => "UI.PanControlMode_Hold",
            ButtonControlMode::Click => "UI.PanControlMode_Click",
        }
    }

    /// The sp3-sp8 deadzone sliders' mouse set (`:452-498`): the value is
    /// the raw fraction; the label appends the `"P1"` percent. The gamepad
    /// step is [`Self::step_deadzone_slider`].
    pub fn set_deadzone_slider(&mut self, sp: &str, value: f32) -> bool {
        let cur = self.registry.current_mut();
        match sp {
            "sp3" => cur.triggers_deadzone = value,
            "sp4" => cur.interface_deadzone_x = value,
            "sp5" => cur.left_thumbstick_deadzone_x = value,
            "sp6" => cur.left_thumbstick_deadzone_y = value,
            "sp7" => cur.right_thumbstick_deadzone_x = value,
            "sp8" => cur.right_thumbstick_deadzone_y = value,
            _ => return false,
        }
        true
    }

    /// The sp3-sp8 gamepad step (`:457`, `:465`, `:473` etc.): all six ride
    /// `HandleSliderHorizontalInput(value, 0, 0.95, deadZone, 0.35)` — sp4
    /// uses the CONSTANT 0.35 deadzone, the other five the profile's
    /// `InterfaceDeadzoneX`. `stick_x` is the caller's gamepad poll.
    pub fn step_deadzone_slider(&mut self, sp: &str, stick_x: f32) -> bool {
        let cur = self.registry.current_mut();
        let deadzone = if sp == "sp4" { 0.35 } else { cur.interface_deadzone_x };
        let value = match sp {
            "sp3" => cur.triggers_deadzone,
            "sp4" => cur.interface_deadzone_x,
            "sp5" => cur.left_thumbstick_deadzone_x,
            "sp6" => cur.left_thumbstick_deadzone_y,
            "sp7" => cur.right_thumbstick_deadzone_x,
            "sp8" => cur.right_thumbstick_deadzone_y,
            _ => return false,
        };
        let stepped = handle_slider_horizontal_input(value, 0.0, 0.95, deadzone, 0.35, stick_x);
        self.set_deadzone_slider(sp, stepped)
    }

    /// The sp18 radial-hold slider (`:597-617`): the fraction maps to
    /// `(int)(f * 301)` ticks; a full 301 lands on -1 (radial OFF). The
    /// getter reads -1 as 1.0.
    pub fn set_radial_hold_fraction(&mut self, f: f32) {
        let cur = self.registry.current_mut();
        cur.hotbar_radial_hold_time_required = (f * 301.0) as i32;
        if cur.hotbar_radial_hold_time_required as f32 == 301.0 {
            cur.hotbar_radial_hold_time_required = -1;
        }
    }

    /// The sp18 getter fraction (`:602`): -1 → 1, else ticks/301.
    pub fn radial_hold_fraction(&self) -> f32 {
        let t = self.registry.current().hotbar_radial_hold_time_required;
        if t == -1 { 1.0 } else { t as f32 / 301.0 }
    }

    /// The sp18 gamepad step (`:609-617`): the 3-arg
    /// `HandleSliderHorizontalInput` form — deadzone = `InterfaceDeadzoneX`,
    /// sensitivity = the 0.5 default.
    pub fn step_radial_hold(&mut self, stick_x: f32) {
        let deadzone = self.registry.current().interface_deadzone_x;
        let f = handle_slider_horizontal_input(self.radial_hold_fraction(), 0.0, 1.0, deadzone, 0.5, stick_x);
        self.set_radial_hold_fraction(f);
    }

    /// The sp19 inventory-move slider's mouse set (`:619-621`):
    /// `(int)Math.Round(MathHelper.Lerp(4, 12, f))` — banker's rounding.
    pub fn set_inventory_move_fraction(&mut self, f: f32) {
        let lerped = 4.0 + (12.0 - 4.0) * f;
        self.registry.current_mut().inventory_move_cd =
            tr_core::damage::round_bankers(lerped as f64) as i32;
    }

    /// The sp19 getter fraction (`:619`): `GetLerpValue(4, 12, cd, clamped)`.
    pub fn inventory_move_fraction(&self) -> f32 {
        get_lerp_value(4.0, 12.0, self.registry.current().inventory_move_cd as f32, true)
    }

    /// The sp19 gamepad step (`:622-638`): the INV_MOVE_OPTION_CD cooldown
    /// decrements first; at 0 the slider steps and, when the fraction
    /// actually moved, the cooldown re-arms to 8 and the CD moves ONE tick
    /// toward the step, clamped 4..12.
    pub fn step_inventory_move(&mut self, stick_x: f32) {
        if self.inv_move_option_cd > 0 {
            self.inv_move_option_cd -= 1;
        }
        if self.inv_move_option_cd == 0 {
            let deadzone = self.registry.current().interface_deadzone_x;
            let lerp_value = self.inventory_move_fraction();
            let num = handle_slider_horizontal_input(lerp_value, 0.0, 1.0, deadzone, 0.5, stick_x);
            if lerp_value != num {
                self.inv_move_option_cd = 8;
                let sign = (num - lerp_value).signum() as i32;
                let cd = self.registry.current().inventory_move_cd + sign;
                self.registry.current_mut().inventory_move_cd = cd.clamp(4, 12);
            }
        }
    }
}

/// The shared border rule of the four `ManageBorder*` handlers
/// (`:828-850`, `:868-890`): `hover` = None (no hover), Some(true) (first
/// button hovered), Some(false) (second hovered). Silver (192,192,192) marks
/// the active tab at rest, Black the inactive; the hovered border is
/// `Main.OurFavoriteColor` regardless.
fn tab_borders(first_active: bool, hover: Option<bool>) -> ([u8; 4], [u8; 4]) {
    const SILVER: [u8; 4] = [192, 192, 192, 255];
    const BLACK: [u8; 4] = [0, 0, 0, 255];
    let resting = |active: bool| if active { SILVER } else { BLACK };
    match hover {
        None => (resting(first_active), resting(!first_active)),
        Some(true) => (OUR_FAVORITE_COLOR, resting(!first_active)),
        Some(false) => (resting(first_active), OUR_FAVORITE_COLOR),
    }
}

/// The `Buttons.DPad*` names in family order 1..4 (`SnapButtonClick`
/// `:765-772`): Up, Right, Down, Left.
const DPAD_DIRS: [&str; 4] = ["DPadUp", "DPadRight", "DPadDown", "DPadLeft"];

/// Clear one dpad family in BOTH gamepad modes (`:747-754` etc.).
fn clear_dpad_family(profile: &mut crate::input_profiles::PlayerInputProfile, family: &str) {
    for mode in [InputMode::XBoxGamepad, InputMode::XBoxGamepadUi] {
        let slot = mode.slot().expect("gamepad modes have slots");
        for i in 1..=4 {
            profile.input_modes[slot]
                .get_mut(&format!("{family}{i}"))
                .expect("SetupKeys seeded every known trigger")
                .clear();
        }
    }
}

/// Bind one dpad family to the canonical directions in BOTH gamepad modes
/// (`:765-772` etc.) — a fresh single-element list per trigger.
fn bind_dpad_family(profile: &mut crate::input_profiles::PlayerInputProfile, family: &str) {
    for mode in [InputMode::XBoxGamepad, InputMode::XBoxGamepadUi] {
        let slot = mode.slot().expect("gamepad modes have slots");
        for (i, dir) in DPAD_DIRS.iter().enumerate() {
            profile.input_modes[slot].insert(format!("{family}{}", i + 1), vec![(*dir).to_string()]);
        }
    }
}

// --- The row labels (the lang reads the rows draw) ----------------------------

/// The sp3-sp8 slider labels (`:452`, `:460`, `:468`, `:476`, `:484`,
/// `:492`): `Lang.menu[199..204] + " (" + value.ToString("P1") + ")"`.
pub fn deadzone_label(sp: &str, value: f32) -> Option<String> {
    let menu = match sp {
        "sp3" => 199,
        "sp4" => 200,
        "sp5" => 201,
        "sp6" => 202,
        "sp7" => 203,
        "sp8" => 204,
        _ => return None,
    };
    Some(format!("{} ({})", crate::lang::menu(menu), format_percent_p1(value)))
}

/// The sp18 label (`:598-601`): -1 → `Lang.menu[228]` (radial off), else
/// `Lang.menu[227] + " (" + (ticks/60).ToString("F2") + "s)"`.
pub fn radial_hold_label(ticks: i32) -> String {
    if ticks == -1 {
        crate::lang::menu(228)
    } else {
        format!("{} ({}s)", crate::lang::menu(227), format_fixed_f2(ticks as f32 / 60.0))
    }
}

/// The sp19 label (`:619`): `Lang.menu[252] + " (" + (cd/60).ToString("F2") + "s)"`.
pub fn inventory_move_label(cd: i32) -> String {
    format!("{} ({}s)", crate::lang::menu(252), format_fixed_f2(cd as f32 / 60.0))
}

/// .NET's `"P1"` percent format on the deadzone floats: ×100, one decimal,
/// `%` appended — the same half-away-from-zero custom-format rounding the
/// loading screen's `"0.0%"` documents (`generation_progress::format_percent`).
pub fn format_percent_p1(value: f32) -> String {
    let scaled = (value as f64 * 100.0 * 10.0).round() / 10.0;
    format!("{scaled:.1}%")
}

/// .NET's `"F2"` fixed format: two decimals, half-away-from-zero on the
/// scaled value (the custom-format rule, as `"P1"` above).
pub fn format_fixed_f2(value: f32) -> String {
    let scaled = (value as f64 * 100.0).round() / 100.0;
    format!("{scaled:.2}")
}

/// The Back button's hover pair (`FadedMouseOver`/`FadedMouseOut`,
/// `:924-935`): over → `(73, 94, 171)` + `Colors.FancyUIFatButtonMouseOver`
/// (sound 12 is the caller's side effect); out → `(63, 82, 151) * 0.7f` +
/// black. Returns `(background, border)`.
pub fn back_button_colors(hover: bool) -> ([u8; 4], [u8; 4]) {
    if hover {
        ([73, 94, 171, 255], FANCY_UI_FAT_BUTTON_MOUSE_OVER)
    } else {
        (color::mul([63, 82, 151, 255], 0.7), uistate::TEXT_PANEL_BORDER)
    }
}

/// The profile/style buttons' base colour (`:211`, `:221`):
/// `new Color(73, 94, 171, 255) * 0.9f`.
pub fn simple_button_color() -> [u8; 4] {
    color::mul([73, 94, 171, 255], 0.9)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `AssembleBindPanels` (`:295-314`): keyboard modes carry 3 groups,
    /// gamepads 5; group 0's list differs by device (LockOn only on the
    /// gamepad list, ToggleCreativeMenu only on the keyboard one).
    #[test]
    fn the_groups_assemble_per_mode() {
        let kb = assemble_groups(InputMode::Keyboard);
        assert_eq!(kb.len(), 3);
        assert_eq!(kb[0].rows.iter().filter(|r| r.bind == "LockOn").count(), 0);
        assert_eq!(kb[0].rows.iter().filter(|r| r.bind == "ToggleCreativeMenu").count(), 1);
        let gp = assemble_groups(InputMode::XBoxGamepad);
        assert_eq!(gp.len(), 5);
        assert_eq!(gp[0].rows.iter().filter(|r| r.bind == "LockOn").count(), 1);
        assert_eq!(gp[3].rows[0].bind, "sp1");
        assert_eq!(gp[4].rows[0].bind, "sp3");
        // The UI twins share the group shape.
        assert_eq!(assemble_groups(InputMode::KeyboardUi).len(), 3);
        assert_eq!(assemble_groups(InputMode::XBoxGamepadUi).len(), 5);
    }

    /// `CreateElementGroup` (`:388-433`): the thin pairs, the half-single
    /// rows (sp9/sp20 checked BEFORE the full-line list) and the full rows.
    /// The map list pairs its 6 binds into 3 thin lines + sp11 half-single.
    #[test]
    fn the_row_pairing_walks_the_source_lists() {
        let rows = place_rows(&MAP_BINDS);
        assert_eq!(rows.len(), 7);
        assert_eq!(rows[0].snap, "Thin");
        assert_eq!(rows[0].h_align, 0.0);
        assert_eq!(rows[1].h_align, 1.0);
        assert_eq!(rows[6].bind, "sp11");
        assert_eq!(rows[6].snap, "Wide");
        assert_eq!(rows[6].width, Style::new(0.0, 0.5));
        assert_eq!(rows[6].h_align, 0.5);
        // sp20 is half-single in the lists, but the thin-pair arm consumes
        // its partner WITHOUT re-checking the lists (`:423-431`) — Dash
        // pairs with sp20 as a thin row. sp21 IS full-line; sp9 half-single.
        let gp = place_rows(&GAMEPLAY_BINDS_KEYBOARD);
        let dash = gp.iter().position(|r| r.bind == "Dash").unwrap();
        assert_eq!(gp[dash].snap, "Thin");
        assert_eq!(gp[dash + 1].bind, "sp20");
        assert_eq!(gp[dash + 1].snap, "Thin", "the pair arm does not re-check the lists");
        assert_eq!(gp[dash + 1].h_align, 1.0);
        let sp21 = gp.iter().find(|r| r.bind == "sp21").unwrap();
        assert_eq!(sp21.width, Style::new(0.0, 1.0));
        // The hotbar list: 12 thin binds pair into 6 lines, sp10 alone.
        let hb = place_rows(&HOTBAR_BINDS);
        assert_eq!(hb.len(), 13);
        assert_eq!(hb[12].bind, "sp10");
        assert_eq!(hb[12].snap, "Wide");
    }

    /// `CreateBindingGroup`'s height (`:382-384` + `UIList.cs:116-128`):
    /// lines × (30 + 5) + 30 + 16. The map group: 4 lines → 186.
    #[test]
    fn the_group_height_sums_the_list_padding() {
        let groups = assemble_groups(InputMode::Keyboard);
        assert_eq!(groups[1].height(), 4.0 * 35.0 + 46.0);
        // The hotbar group: 7 lines (6 pairs + sp10).
        assert_eq!(groups[2].height(), 7.0 * 35.0 + 46.0);
    }

    /// The group tints (`:337-356`): base (33,43,79)*0.8 = (26,34,63,204),
    /// lerped 0.18 toward the accent, then MultiplyRGBA(111) for the rows.
    #[test]
    fn the_group_colours_lerp_then_multiply() {
        let base = color::mul([33, 43, 79, 255], 0.8);
        assert_eq!(base, [26, 34, 63, 204]);
        let g0 = group_panel_color(0);
        assert_eq!(g0, lerp_rgb(base, [0, 128, 0, 255], 0.18));
        let r0 = row_color(0);
        assert_eq!(r0[0], ((g0[0] as u32 * 111) / 255) as u8);
        assert_eq!(r0[3], ((204u32 * 111) / 255) as u8);
        // An unknown index keeps the base (the switch has no default arm).
        assert_eq!(group_panel_color(9), base);
    }

    /// The tab flags → `FillList`'s mode (`:722-738`) and the border rule
    /// (`:828-850`): Silver on the active tab, Black on the inactive,
    /// OurFavoriteColor on the hovered one.
    #[test]
    fn the_tabs_select_the_mode_and_paint_the_borders() {
        let mut s = ManageControlsState::new();
        assert_eq!(s.active_mode(), InputMode::Keyboard);
        s.click_gamepad_tab();
        assert_eq!(s.active_mode(), InputMode::XBoxGamepad);
        s.click_menu_tab();
        assert_eq!(s.active_mode(), InputMode::XBoxGamepadUi);
        s.click_keyboard_tab();
        assert_eq!(s.active_mode(), InputMode::KeyboardUi);
        s.click_gameplay_tab();
        assert_eq!(s.active_mode(), InputMode::Keyboard);
        let silver = [192, 192, 192, 255];
        let black = [0, 0, 0, 255];
        assert_eq!(s.keyboard_borders(None), (silver, black));
        assert_eq!(s.keyboard_borders(Some(false)), (silver, OUR_FAVORITE_COLOR));
        s.click_gamepad_tab();
        assert_eq!(s.keyboard_borders(None), (black, silver));
        assert_eq!(s.keyboard_borders(Some(true)), (OUR_FAVORITE_COLOR, silver));
    }

    /// `SnapButtonClick`/`RadialButtonClick` (`:740-810`): the Redigit
    /// seed has Snap bound (sp1 on) — clicking sp1 clears it; clicking sp2
    /// then clears Snap (already empty) and binds Radial in BOTH modes.
    /// AllowEditing=false blocks both (no sound).
    #[test]
    fn the_dpad_toggles_swap_the_families() {
        let mut s = ManageControlsState::new();
        assert!(s.registry.current().using_dpad_movekeys());
        assert!(s.click_snap_toggle()); // sound 12
        assert!(!s.registry.current().using_dpad_movekeys());
        assert!(s.registry.current().keys(InputMode::XBoxGamepad, "DpadSnap1").is_empty());
        assert!(s.click_radial_toggle());
        assert!(s.registry.current().using_dpad_hotbar());
        assert_eq!(s.registry.current().keys(InputMode::XBoxGamepadUi, "DpadRadial2"), ["DPadRight"]);
        // sp2 again clears the radial family.
        assert!(s.click_radial_toggle());
        assert!(!s.registry.current().using_dpad_hotbar());
        // The editing gate.
        s.registry.current_mut().allow_editing = false;
        assert!(!s.click_snap_toggle());
        assert!(!s.click_radial_toggle());
    }

    /// The sp18 slider (`:597-617`): the fraction maps through (int)(f*301)
    /// truncation; a full 1.0 lands on -1; the getter reads -1 back as 1.
    #[test]
    fn the_radial_hold_slider_truncates_and_wraps_to_off() {
        let mut s = ManageControlsState::new();
        assert_eq!(s.registry.current().hotbar_radial_hold_time_required, 16);
        s.set_radial_hold_fraction(0.5);
        assert_eq!(s.registry.current().hotbar_radial_hold_time_required, 150);
        s.set_radial_hold_fraction(1.0);
        assert_eq!(s.registry.current().hotbar_radial_hold_time_required, -1);
        assert_eq!(s.radial_hold_fraction(), 1.0);
        assert!(!s.registry.current().hotbar_allows_radial());
        s.set_radial_hold_fraction(0.0);
        assert_eq!(s.registry.current().hotbar_radial_hold_time_required, 0);
    }

    /// The sp19 slider (`:618-639`): the mouse set rounds banker's
    /// (Lerp(4,12,0.5)=8 exact; 0.0625→4.5→4 ties-to-even); the gamepad
    /// step moves ONE tick and re-arms the 8-tick cooldown.
    #[test]
    fn the_inventory_move_slider_rounds_and_cools_down() {
        let mut s = ManageControlsState::new();
        s.set_inventory_move_fraction(0.0625); // 4.5 → 4 (banker's)
        assert_eq!(s.registry.current().inventory_move_cd, 4);
        s.set_inventory_move_fraction(0.1875); // 5.5 → 6 (banker's)
        assert_eq!(s.registry.current().inventory_move_cd, 6);
        s.set_inventory_move_fraction(1.0);
        assert_eq!(s.registry.current().inventory_move_cd, 12);
        // Full-deflection stick: the fraction moves, CD steps down by 1,
        // cooldown 8; the next 8 calls only decrement.
        s.step_inventory_move(-1.0);
        assert_eq!(s.registry.current().inventory_move_cd, 11);
        assert_eq!(s.inv_move_option_cd, 8);
        for _ in 0..7 {
            s.step_inventory_move(-1.0);
        }
        assert_eq!(s.registry.current().inventory_move_cd, 11);
        assert_eq!(s.inv_move_option_cd, 1);
        s.step_inventory_move(-1.0);
        assert_eq!(s.registry.current().inventory_move_cd, 10);
        // At the floor the clamp holds 4.
        s.set_inventory_move_fraction(0.0);
        s.inv_move_option_cd = 0;
        s.step_inventory_move(-1.0);
        assert_eq!(s.registry.current().inventory_move_cd, 4);
    }

    /// The sp3-sp8 gamepad step (`:457` etc.): sp4 uses the constant 0.35
    /// deadzone, sp3 the profile's InterfaceDeadzoneX; at rest nothing moves.
    #[test]
    fn the_deadzone_sliders_step_with_their_own_deadzones() {
        let mut s = ManageControlsState::new();
        s.registry.current_mut().interface_deadzone_x = 0.9;
        // sp3's deadzone is 0.9 — a 0.5 stick is inside it, no step (the
        // value still rides the clamp round-trip, so compare with tolerance).
        let before = s.registry.current().triggers_deadzone;
        s.step_deadzone_slider("sp3", 0.5);
        assert!((s.registry.current().triggers_deadzone - before).abs() < 1e-6);
        // sp4's deadzone is the constant 0.35 — the same stick steps.
        let before = s.registry.current().interface_deadzone_x;
        s.step_deadzone_slider("sp4", 0.5);
        assert!(s.registry.current().interface_deadzone_x > before);
        assert!(!s.step_deadzone_slider("sp99", 1.0));
    }

    /// The reset rows (`:499-548`) route through GetCopyableProfileName;
    /// the invert toggles gate on AllowEditing while sp20/sp21/sp22 do not.
    #[test]
    fn the_reset_and_toggle_rows_keep_their_gates() {
        let mut s = ManageControlsState::new();
        s.registry.current_mut().input_modes[0].get_mut("Jump").unwrap().clear();
        assert!(s.click_reset_row("sp9", InputMode::Keyboard));
        assert_eq!(s.registry.current().keys(InputMode::Keyboard, "Jump"), ["Space"]);
        assert!(!s.click_reset_row("sp1", InputMode::Keyboard));
        assert!(s.click_invert_toggle("sp14"));
        assert!(s.registry.current().left_thumbstick_invert_x);
        s.registry.current_mut().allow_editing = false;
        assert!(!s.click_invert_toggle("sp14"));
        assert!(s.registry.current().left_thumbstick_invert_x, "the gate blocked the flip");
        // sp20 has no gate; sp21/sp22 cycle the pan rings.
        s.click_dash_toggle();
        assert_eq!(s.dash_control, DashPreference::OnlyThroughHotkeys);
        assert!(s.click_pan_cycle("sp21"));
        assert_eq!(s.registry.current().pan_control_keyboard, ButtonControlMode::Click);
        assert!(s.click_pan_cycle("sp22"));
        assert_eq!(s.registry.current().pan_control_gamepad, ButtonControlMode::OffAlways);
    }

    /// `OnActivate` (`:694-705`), the glyph cycle (`:903-910`), the profile
    /// cycle (`:912-922`) and `GoBackClick`'s 1127 (`:937-941`).
    #[test]
    fn the_screen_buttons_route_their_source_targets() {
        let mut s = ManageControlsState::new();
        s.on_activate(true);
        assert_eq!(s.outer_top, Style::px(220.0));
        assert_eq!(s.outer_height, Style::new(-220.0, 1.0));
        s.on_activate(false);
        assert_eq!(s.outer_top, Style::px(120.0));
        assert_eq!(s.outer_height, Style::new(-120.0, 1.0));
        assert_eq!(s.glyph_label_key(), "UI.ControllerGlyphAuto");
        s.click_glyph_style();
        assert_eq!(s.glyph_label_key(), "UI.ControllerGlyphXbox");
        s.click_profile();
        assert_eq!(s.registry.selected, "Redigit's Pick");
        assert_eq!(s.click_back(), 1127);
    }

    /// The label formats: `"P1"` ×100 one decimal half-away-from-zero,
    /// `"F2"` two decimals; the sp18 -1 arm reads Lang.menu[228].
    #[test]
    fn the_labels_format_like_the_dotnet_specifiers() {
        assert_eq!(format_percent_p1(0.3), "30.0%");
        assert_eq!(format_percent_p1(0.95), "95.0%");
        assert_eq!(format_fixed_f2(16.0 / 60.0), "0.27");
        assert_eq!(format_fixed_f2(0.5), "0.50");
        assert_eq!(format_fixed_f2(6.0 / 60.0), "0.10");
        let _guard = crate::lang::test_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        assert_eq!(radial_hold_label(-1), crate::lang::menu(228));
        assert_eq!(radial_hold_label(16), format!("{} (0.27s)", crate::lang::menu(227)));
        assert_eq!(inventory_move_label(6), format!("{} (0.10s)", crate::lang::menu(252)));
        assert_eq!(deadzone_label("sp3", 0.3), Some(format!("{} (30.0%)", crate::lang::menu(199))));
        assert_eq!(deadzone_label("sp9", 0.3), None);
    }

    /// The Back button's faded pair (`:924-935`) and the simple buttons'
    /// 0.9 base (`:211`, `:221`).
    #[test]
    fn the_button_colours_match_the_faded_pair() {
        let (bg, border) = back_button_colors(true);
        assert_eq!(bg, [73, 94, 171, 255]);
        assert_eq!(border, OUR_FAVORITE_COLOR);
        let (bg, border) = back_button_colors(false);
        assert_eq!(bg, [44, 57, 105, 178]);
        assert_eq!(border, [0, 0, 0, 255]);
        assert_eq!(simple_button_color(), [65, 84, 153, 229]);
    }
}
