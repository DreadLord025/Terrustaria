//! The input-profile data the manage-controls screen edits — W4-d86:
//! `GameInput/PlayerInputProfile.cs:9-354`, `GameInput/KeyConfiguration.cs`
//! (`SetupKeys` `:22-29`), `GameInput/PlayerInput.cs` (the profile registry
//! `Initialize` `:722-755`, `SetSelectedProfile` `:713-720`, the preset seed
//! tables `Reset` `:2046-2500`, `KnownTriggers` `:2558-2566`),
//! `GameInput/InputMode.cs:3-10`, `ButtonControlMode.cs:3-9`,
//! `Player.Settings.DashPreference` (`Player.cs:352-356`, `:411`) and
//! `Utils.CycleControlControl_OffOnClickHold` (`Utils.cs:159-176`) +
//! `UILinksInitializer.HandleSliderHorizontalInput` (`:86-91`).
//!
//! The screen itself is [`crate::ui::manage_controls`]; the five
//! `UIKeybinding*ListItem` row elements are d83's `ui/keybinding_rows.rs`
//! (research/18b rows 40-44) — at merge keep ONE `InputMode` (d83's copy and
//! this one cite the same `InputMode.cs`; this module is the data-layer home).
//!
//! Deliberate gaps (code-only lane, recorded in ANALYSIS "The manage-controls
//! screen"): the `input.json` persistence (`PlayerInput.Save/Load`
//! `:765-812`, a second config file beside `config.json`) is not ported — the
//! registry lives in memory exactly as a fresh vanilla boot seeds it; the
//! `TriggersSet` runtime consumption (`Processkey`/`CopyKeyState`,
//! `KeyConfiguration.cs:31-56`) stays with the input pass, which today reads
//! the 14+6 bind strings in [`crate::input_map`] — the screen's keyboard
//! edits reach that table at the wiring follow-up, not here.

#![allow(dead_code)] // wiring deferred — the screen mount + input_map feed are the follow-up

use std::collections::BTreeMap;

/// `InputMode` (`GameInput/InputMode.cs:3-10`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputMode {
    Keyboard,
    KeyboardUi,
    Mouse,
    XBoxGamepad,
    XBoxGamepadUi,
}

impl InputMode {
    /// The four `KeyConfiguration` slots of a profile
    /// (`PlayerInputProfile.cs:11-29` — the dictionary has NO `Mouse` entry;
    /// the C# would throw on `InputModes[Mouse]`, the port returns `None`).
    pub fn slot(self) -> Option<usize> {
        match self {
            Self::Keyboard => Some(0),
            Self::KeyboardUi => Some(1),
            Self::XBoxGamepad => Some(2),
            Self::XBoxGamepadUi => Some(3),
            Self::Mouse => None,
        }
    }

    /// The gamepad pair (`UIKeybindingListItem.cs:71`).
    pub fn is_gamepad(self) -> bool {
        matches!(self, Self::XBoxGamepad | Self::XBoxGamepadUi)
    }
}

/// `ButtonControlMode` (`ButtonControlMode.cs:3-9`) — the pan-control cycle
/// states. Discriminants are the persisted `config.json` integers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonControlMode {
    #[default]
    Hold,
    Click,
    OnAlways,
    OffAlways,
}

/// `Utils.CycleControlControl_OffOnClickHold` (`Utils.cs:159-176`): the
/// four-state ring Hold → Click → OnAlways → OffAlways → Hold (the `default`
/// arm lands on Hold). The sp21/sp22 rows click through it
/// (`UIManageControls.cs:669`, `:685`).
pub fn cycle_control_off_on_click_hold(mode: ButtonControlMode) -> ButtonControlMode {
    match mode {
        ButtonControlMode::Hold => ButtonControlMode::Click,
        ButtonControlMode::Click => ButtonControlMode::OnAlways,
        ButtonControlMode::OnAlways => ButtonControlMode::OffAlways,
        ButtonControlMode::OffAlways => ButtonControlMode::Hold,
    }
}

/// `Player.Settings.DashPreference` (`Player.cs:352-356`); the static default
/// is `AllowDoubleTap` (`Player.cs:411`). Persisted as `"DashControlMode"`
/// (`Main.cs:4210`/`:4547`) — the Settings field lands with the wiring.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DashPreference {
    #[default]
    AllowDoubleTap,
    OnlyThroughHotkeys,
}

/// The sp20 toggle (`UIManageControls.cs:643-654`): the two-state flip, no
/// `AllowEditing` gate (unlike sp14-sp17).
pub fn cycle_dash_control(dash: DashPreference) -> DashPreference {
    match dash {
        DashPreference::AllowDoubleTap => DashPreference::OnlyThroughHotkeys,
        DashPreference::OnlyThroughHotkeys => DashPreference::AllowDoubleTap,
    }
}

/// `PresetProfiles` (`GameInput/PresetProfiles.cs:3-10`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresetProfiles {
    None,
    Redigit,
    Yoraiz0r,
    ConsolePs,
    ConsoleXBox,
}

/// `PlayerInput.KnownTriggers` (`PlayerInput.cs:2558-2566`), in source order —
/// `KeyConfiguration.SetupKeys` seeds one empty list per name
/// (`KeyConfiguration.cs:22-29`).
pub const KNOWN_TRIGGERS: [&str; 60] = [
    "MouseLeft", "MouseRight", "Up", "Down", "Left", "Right", "Jump", "Throw", "Inventory", "Grapple",
    "SmartSelect", "SmartCursor", "QuickMount", "QuickHeal", "QuickMana", "QuickBuff", "MapZoomIn", "MapZoomOut", "MapAlphaUp", "MapAlphaDown",
    "MapFull", "MapStyle", "Hotbar1", "Hotbar2", "Hotbar3", "Hotbar4", "Hotbar5", "Hotbar6", "Hotbar7", "Hotbar8",
    "Hotbar9", "Hotbar10", "HotbarMinus", "HotbarPlus", "DpadRadial1", "DpadRadial2", "DpadRadial3", "DpadRadial4", "RadialHotbar", "RadialQuickbar",
    "DpadSnap1", "DpadSnap2", "DpadSnap3", "DpadSnap4", "MenuUp", "MenuDown", "MenuLeft", "MenuRight", "LockOn", "ViewZoomIn",
    "ViewZoomOut", "Loadout1", "Loadout2", "Loadout3", "PreviousLoadout", "NextLoadout", "ToggleCameraMode", "ToggleCreativeMenu", "ArmorSetAbility", "Dash",
];

/// One mode's `KeyConfiguration.KeyStatus` (`KeyConfiguration.cs:8`) — a
/// trigger → bound-key-names map. `BTreeMap` keeps the iteration deterministic
/// (the C# `Dictionary` order only matters to `Save`, which is not ported).
pub type KeyStatus = BTreeMap<String, Vec<String>>;

/// `PlayerInputProfile` (`PlayerInputProfile.cs:9-65`) — the field defaults
/// are the C# field initializers (`:31-61`).
#[derive(Clone, Debug)]
pub struct PlayerInputProfile {
    /// `InputModes` (`:11-29`) — the four slots, [`InputMode::slot`] order.
    pub input_modes: [KeyStatus; 4],
    /// `Name` (`:31`).
    pub name: String,
    /// `AllowEditing` (`:33`, default true).
    pub allow_editing: bool,
    /// `HotbarRadialHoldTimeRequired` (`:35`, default 16; -1 = radial off).
    pub hotbar_radial_hold_time_required: i32,
    /// `TriggersDeadzone` (`:37`, default 0.3).
    pub triggers_deadzone: f32,
    /// `InterfaceDeadzoneX` (`:39`, default 0.2).
    pub interface_deadzone_x: f32,
    /// `LeftThumbstickDeadzoneX` (`:41`, default 0.25).
    pub left_thumbstick_deadzone_x: f32,
    /// `LeftThumbstickDeadzoneY` (`:43`, default 0.4).
    pub left_thumbstick_deadzone_y: f32,
    /// `RightThumbstickDeadzoneX` (`:45`, default 0).
    pub right_thumbstick_deadzone_x: f32,
    /// `RightThumbstickDeadzoneY` (`:47`, default 0).
    pub right_thumbstick_deadzone_y: f32,
    /// `LeftThumbstickInvertX` (`:49`).
    pub left_thumbstick_invert_x: bool,
    /// `LeftThumbstickInvertY` (`:51`).
    pub left_thumbstick_invert_y: bool,
    /// `RightThumbstickInvertX` (`:53`).
    pub right_thumbstick_invert_x: bool,
    /// `RightThumbstickInvertY` (`:55`).
    pub right_thumbstick_invert_y: bool,
    /// `InventoryMoveCD` (`:57`, default 6).
    pub inventory_move_cd: i32,
    /// `PanControl_Keyboard` (`:59`, default `Hold` = 0).
    pub pan_control_keyboard: ButtonControlMode,
    /// `PanControl_Gamepad` (`:61`, default `OnAlways`).
    pub pan_control_gamepad: ButtonControlMode,
}

impl PlayerInputProfile {
    /// The ctor (`:67-70`) + field initializers; the maps start empty (the
    /// C# `KeyConfiguration`s are empty until `Initialize`).
    pub fn new(name: &str) -> Self {
        Self {
            input_modes: Default::default(),
            name: name.to_string(),
            allow_editing: true,
            hotbar_radial_hold_time_required: 16,
            triggers_deadzone: 0.3,
            interface_deadzone_x: 0.2,
            left_thumbstick_deadzone_x: 0.25,
            left_thumbstick_deadzone_y: 0.4,
            right_thumbstick_deadzone_x: 0.0,
            right_thumbstick_deadzone_y: 0.0,
            left_thumbstick_invert_x: false,
            left_thumbstick_invert_y: false,
            right_thumbstick_invert_x: false,
            right_thumbstick_invert_y: false,
            inventory_move_cd: 6,
            pan_control_keyboard: ButtonControlMode::Hold,
            pan_control_gamepad: ButtonControlMode::OnAlways,
        }
    }

    /// `ShowName => Name` (`:63`).
    pub fn show_name(&self) -> &str {
        &self.name
    }

    /// `HotbarAllowsRadial => HotbarRadialHoldTimeRequired != -1` (`:65`).
    pub fn hotbar_allows_radial(&self) -> bool {
        self.hotbar_radial_hold_time_required != -1
    }

    /// `Initialize` (`:72-79`): `SetupKeys` (one empty list per known
    /// trigger) then the preset `Reset` adds, per mode.
    pub fn initialize(&mut self, style: PresetProfiles) {
        for mode in [
            InputMode::Keyboard,
            InputMode::KeyboardUi,
            InputMode::XBoxGamepad,
            InputMode::XBoxGamepadUi,
        ] {
            let slot = mode.slot().expect("the four profile modes");
            self.input_modes[slot] = KNOWN_TRIGGERS
                .iter()
                .map(|t| ((*t).to_string(), Vec::new()))
                .collect();
            for (trigger, key) in preset_adds(style, mode) {
                self.input_modes[slot]
                    .get_mut(*trigger)
                    .expect("SetupKeys seeded every known trigger")
                    .push((*key).to_string());
            }
        }
    }

    /// One mode's bound keys for a trigger (the `KeyStatus[bind]` reads the
    /// rows draw). Unknown triggers read empty — the C# would throw; every
    /// screen bind is a known trigger.
    pub fn keys(&self, mode: InputMode, trigger: &str) -> &[String] {
        mode.slot()
            .and_then(|s| self.input_modes[s].get(trigger))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// `CopyKeysFrom` (`:325-335`): per key, only when the SOURCE has the
    /// trigger — clear the target list and append the source's names.
    fn copy_keys_from(&mut self, profile: &PlayerInputProfile, mode: InputMode, keys: &[&str]) {
        let Some(slot) = mode.slot() else { return };
        for key in keys {
            if let Some(value) = profile.input_modes[slot].get(*key) {
                let target = self.input_modes[slot]
                    .get_mut(*key)
                    .expect("SetupKeys seeded every known trigger");
                target.clear();
                target.extend(value.iter().cloned());
            }
        }
    }

    /// `CopyGameplaySettingsFrom` (`:277-285`) — the 27 gameplay keys, the
    /// sp9 reset row (`UIManageControls.cs:499-508`).
    pub fn copy_gameplay_settings_from(&mut self, profile: &PlayerInputProfile, mode: InputMode) {
        self.copy_keys_from(profile, mode, &GAMEPLAY_KEYS);
    }

    /// `CopyHotbarSettingsFrom` (`:287-294`) — sp10 (`UIManageControls.cs:509-518`).
    pub fn copy_hotbar_settings_from(&mut self, profile: &PlayerInputProfile, mode: InputMode) {
        self.copy_keys_from(profile, mode, &HOTBAR_KEYS);
    }

    /// `CopyMapSettingsFrom` (`:296-299`) — sp11 (`UIManageControls.cs:519-528`).
    pub fn copy_map_settings_from(&mut self, profile: &PlayerInputProfile, mode: InputMode) {
        self.copy_keys_from(profile, mode, &MAP_KEYS);
    }

    /// `CopyGamepadSettingsFrom` (`:301-306`) — sp12: the 10 radial/dpad keys
    /// across BOTH gamepad modes (the `mode` argument is ignored by the C#).
    pub fn copy_gamepad_settings_from(&mut self, profile: &PlayerInputProfile, _mode: InputMode) {
        self.copy_keys_from(profile, InputMode::XBoxGamepad, &GAMEPAD_KEYS);
        self.copy_keys_from(profile, InputMode::XBoxGamepadUi, &GAMEPAD_KEYS);
    }

    /// `CopyGamepadAdvancedSettingsFrom` (`:308-323`) — sp13: the deadzone /
    /// invert / move-CD / pan fields, NOT the keys.
    pub fn copy_gamepad_advanced_settings_from(&mut self, profile: &PlayerInputProfile, _mode: InputMode) {
        self.triggers_deadzone = profile.triggers_deadzone;
        self.interface_deadzone_x = profile.interface_deadzone_x;
        self.left_thumbstick_deadzone_x = profile.left_thumbstick_deadzone_x;
        self.left_thumbstick_deadzone_y = profile.left_thumbstick_deadzone_y;
        self.right_thumbstick_deadzone_x = profile.right_thumbstick_deadzone_x;
        self.right_thumbstick_deadzone_y = profile.right_thumbstick_deadzone_y;
        self.left_thumbstick_invert_x = profile.left_thumbstick_invert_x;
        self.left_thumbstick_invert_y = profile.left_thumbstick_invert_y;
        self.right_thumbstick_invert_x = profile.right_thumbstick_invert_x;
        self.right_thumbstick_invert_y = profile.right_thumbstick_invert_y;
        self.inventory_move_cd = profile.inventory_move_cd;
        self.pan_control_keyboard = profile.pan_control_keyboard;
        self.pan_control_gamepad = profile.pan_control_gamepad;
    }

    /// `UsingDpadMovekeys` (`:346-353`) — the sp1 toggle's status: all four
    /// `DpadSnap*` bound to their canonical direction in BOTH gamepad modes.
    pub fn using_dpad_movekeys(&self) -> bool {
        dpad_family_bound(self, "DpadSnap")
    }

    /// `UsingDpadHotbar` (`:337-344`) — the sp2 toggle's status, `DpadRadial*`.
    pub fn using_dpad_hotbar(&self) -> bool {
        dpad_family_bound(self, "DpadRadial")
    }
}

/// The shared `Contains` chain of `UsingDpadHotbar`/`UsingDpadMovekeys`
/// (`PlayerInputProfile.cs:339-341`/`:348-350`): family index 1..4 must
/// contain DPadUp/Right/Down/Left in both gamepad modes.
fn dpad_family_bound(profile: &PlayerInputProfile, family: &str) -> bool {
    const DIRS: [&str; 4] = ["DPadUp", "DPadRight", "DPadDown", "DPadLeft"];
    for mode in [InputMode::XBoxGamepad, InputMode::XBoxGamepadUi] {
        for (i, dir) in DIRS.iter().enumerate() {
            let trigger = format!("{family}{}", i + 1);
            if !profile.keys(mode, &trigger).iter().any(|k| k == dir) {
                return false;
            }
        }
    }
    true
}

/// `CopyGameplaySettingsFrom`'s 27-key list (`PlayerInputProfile.cs:279-284`).
pub const GAMEPLAY_KEYS: [&str; 27] = [
    "MouseLeft", "MouseRight", "Up", "Down", "Left", "Right", "Jump", "Grapple", "SmartSelect", "SmartCursor",
    "QuickMount", "QuickHeal", "QuickMana", "QuickBuff", "Throw", "Inventory", "ViewZoomIn", "ViewZoomOut", "Loadout1", "Loadout2",
    "Loadout3", "NextLoadout", "PreviousLoadout", "ToggleCreativeMenu", "ToggleCameraMode", "ArmorSetAbility", "Dash",
];

/// `CopyHotbarSettingsFrom`'s 12-key list (`:289-293`).
pub const HOTBAR_KEYS: [&str; 12] = [
    "HotbarMinus", "HotbarPlus", "Hotbar1", "Hotbar2", "Hotbar3", "Hotbar4", "Hotbar5", "Hotbar6", "Hotbar7", "Hotbar8",
    "Hotbar9", "Hotbar10",
];

/// `CopyMapSettingsFrom`'s 6-key list (`:298`).
pub const MAP_KEYS: [&str; 6] = ["MapZoomIn", "MapZoomOut", "MapAlphaUp", "MapAlphaDown", "MapFull", "MapStyle"];

/// `CopyGamepadSettingsFrom`'s 10-key list (`:303`).
pub const GAMEPAD_KEYS: [&str; 10] = [
    "RadialHotbar", "RadialQuickbar", "DpadSnap1", "DpadSnap2", "DpadSnap3", "DpadSnap4", "DpadRadial1", "DpadRadial2", "DpadRadial3", "DpadRadial4",
];

// --- The preset seed tables (PlayerInput.Reset, :2046-2500) ------------------
// Each table is the (trigger, key) ADD sequence of one preset+mode arm, in
// source order (the order matters: a trigger with two adds keeps that order).
// The Keyboard arm is byte-identical in all four presets (:2053-2092 =
// :2166-2205 = :2278-2317 = :2390-2429) and so is KeyboardUI (:2093-2110 =
// :2206-2223; the Console twins :2318-2335/:2430-2447 add Menu* before
// Inventory, but each trigger's list is unchanged) — one shared table each.

const KB: &[(&str, &str)] = &[
    ("MouseLeft", "Mouse1"), ("MouseRight", "Mouse2"), ("Up", "W"), ("Down", "S"), ("Left", "A"), ("Right", "D"),
    ("Jump", "Space"), ("Inventory", "Escape"), ("Grapple", "E"), ("SmartSelect", "LeftShift"), ("SmartCursor", "LeftControl"),
    ("QuickMount", "R"), ("QuickHeal", "H"), ("QuickMana", "J"), ("QuickBuff", "B"), ("MapStyle", "Tab"), ("MapFull", "M"),
    ("MapZoomIn", "Add"), ("MapZoomOut", "Subtract"), ("MapAlphaUp", "PageUp"), ("MapAlphaDown", "PageDown"),
    ("Hotbar1", "D1"), ("Hotbar2", "D2"), ("Hotbar3", "D3"), ("Hotbar4", "D4"), ("Hotbar5", "D5"),
    ("Hotbar6", "D6"), ("Hotbar7", "D7"), ("Hotbar8", "D8"), ("Hotbar9", "D9"), ("Hotbar10", "D0"),
    ("ViewZoomOut", "OemMinus"), ("ViewZoomIn", "OemPlus"), ("ToggleCreativeMenu", "C"),
    ("Loadout1", "F1"), ("Loadout2", "F2"), ("Loadout3", "F3"), ("ToggleCameraMode", "F4"),
];

const KB_UI: &[(&str, &str)] = &[
    ("MouseLeft", "Mouse1"), ("MouseLeft", "Space"), ("MouseRight", "Mouse2"),
    ("Up", "W"), ("Up", "Up"), ("Down", "S"), ("Down", "Down"), ("Left", "A"), ("Left", "Left"), ("Right", "D"), ("Right", "Right"),
    ("Inventory", "Escape"), ("MenuUp", "DPadUp"), ("MenuDown", "DPadDown"), ("MenuLeft", "DPadLeft"), ("MenuRight", "DPadRight"),
];

/// Redigit's gamepad arm (`PlayerInput.cs:2111-2133`).
const REDIGIT_GP: &[(&str, &str)] = &[
    ("MouseLeft", "RightTrigger"), ("MouseRight", "B"),
    ("Up", "LeftThumbstickUp"), ("Down", "LeftThumbstickDown"), ("Left", "LeftThumbstickLeft"), ("Right", "LeftThumbstickRight"),
    ("Jump", "LeftTrigger"), ("Inventory", "Y"), ("Grapple", "B"), ("LockOn", "X"), ("QuickMount", "A"),
    ("SmartSelect", "RightStick"), ("SmartCursor", "LeftStick"), ("HotbarMinus", "LeftShoulder"), ("HotbarPlus", "RightShoulder"),
    ("MapFull", "Start"), ("DpadSnap1", "DPadUp"), ("DpadSnap3", "DPadDown"), ("DpadSnap4", "DPadLeft"), ("DpadSnap2", "DPadRight"),
    ("MapStyle", "Back"),
];

/// Redigit's gamepad-UI arm (`:2134-2158`).
const REDIGIT_GP_UI: &[(&str, &str)] = &[
    ("MouseLeft", "A"), ("MouseRight", "LeftShoulder"), ("SmartCursor", "RightShoulder"),
    ("Up", "LeftThumbstickUp"), ("Down", "LeftThumbstickDown"), ("Left", "LeftThumbstickLeft"), ("Right", "LeftThumbstickRight"),
    ("Inventory", "B"), ("Inventory", "Y"), ("HotbarMinus", "LeftTrigger"), ("HotbarPlus", "RightTrigger"),
    ("Grapple", "X"), ("MapFull", "Start"), ("SmartSelect", "Back"), ("QuickMount", "RightStick"),
    ("DpadSnap1", "DPadUp"), ("DpadSnap3", "DPadDown"), ("DpadSnap4", "DPadLeft"), ("DpadSnap2", "DPadRight"),
    ("MenuUp", "DPadUp"), ("MenuDown", "DPadDown"), ("MenuLeft", "DPadLeft"), ("MenuRight", "DPadRight"),
];

/// Yoraiz0r's gamepad arm (`:2224-2245`) — Grapple/Smart sticks/QuickMount/
/// QuickHeal/RadialHotbar differ from Redigit; no LockOn.
const YORAIZ0R_GP: &[(&str, &str)] = &[
    ("MouseLeft", "RightTrigger"), ("MouseRight", "B"),
    ("Up", "LeftThumbstickUp"), ("Down", "LeftThumbstickDown"), ("Left", "LeftThumbstickLeft"), ("Right", "LeftThumbstickRight"),
    ("Jump", "LeftTrigger"), ("Inventory", "Y"), ("Grapple", "LeftShoulder"), ("SmartSelect", "LeftStick"), ("SmartCursor", "RightStick"),
    ("QuickMount", "X"), ("QuickHeal", "A"), ("RadialHotbar", "RightShoulder"), ("MapFull", "Start"),
    ("DpadSnap1", "DPadUp"), ("DpadSnap3", "DPadDown"), ("DpadSnap4", "DPadLeft"), ("DpadSnap2", "DPadRight"),
    ("MapStyle", "Back"),
];

/// Yoraiz0r's gamepad-UI arm (`:2246-2270`) — Redigit's plus LockOn=B, and
/// Inventory keeps only Y (no B).
const YORAIZ0R_GP_UI: &[(&str, &str)] = &[
    ("MouseLeft", "A"), ("MouseRight", "LeftShoulder"), ("SmartCursor", "RightShoulder"),
    ("Up", "LeftThumbstickUp"), ("Down", "LeftThumbstickDown"), ("Left", "LeftThumbstickLeft"), ("Right", "LeftThumbstickRight"),
    ("LockOn", "B"), ("Inventory", "Y"), ("HotbarMinus", "LeftTrigger"), ("HotbarPlus", "RightTrigger"),
    ("Grapple", "X"), ("MapFull", "Start"), ("SmartSelect", "Back"), ("QuickMount", "RightStick"),
    ("DpadSnap1", "DPadUp"), ("DpadSnap3", "DPadDown"), ("DpadSnap4", "DPadLeft"), ("DpadSnap2", "DPadRight"),
    ("MenuUp", "DPadUp"), ("MenuDown", "DPadDown"), ("MenuLeft", "DPadLeft"), ("MenuRight", "DPadRight"),
];

/// Console (Playstation)'s gamepad arm (`:2336-2357`).
const CONSOLE_PS_GP: &[(&str, &str)] = &[
    ("MouseLeft", "RightShoulder"), ("MouseRight", "B"),
    ("Up", "LeftThumbstickUp"), ("Down", "LeftThumbstickDown"), ("Left", "LeftThumbstickLeft"), ("Right", "LeftThumbstickRight"),
    ("Jump", "A"), ("LockOn", "X"), ("Inventory", "Y"), ("Grapple", "LeftShoulder"),
    ("SmartSelect", "LeftStick"), ("SmartCursor", "RightStick"), ("HotbarMinus", "LeftTrigger"), ("HotbarPlus", "RightTrigger"),
    ("MapFull", "Start"), ("DpadRadial1", "DPadUp"), ("DpadRadial3", "DPadDown"), ("DpadRadial4", "DPadLeft"), ("DpadRadial2", "DPadRight"),
    ("QuickMount", "Back"),
];

/// Console (Playstation)'s gamepad-UI arm (`:2358-2382`); the Console (Xbox)
/// UI arm (`:2470-2494`) is byte-identical — one shared table.
const CONSOLE_GP_UI: &[(&str, &str)] = &[
    ("MouseLeft", "A"), ("MouseRight", "LeftShoulder"), ("SmartCursor", "RightShoulder"),
    ("Up", "LeftThumbstickUp"), ("Down", "LeftThumbstickDown"), ("Left", "LeftThumbstickLeft"), ("Right", "LeftThumbstickRight"),
    ("Inventory", "B"), ("Inventory", "Y"), ("HotbarMinus", "LeftTrigger"), ("HotbarPlus", "RightTrigger"),
    ("Grapple", "X"), ("MapFull", "Start"), ("SmartSelect", "Back"), ("QuickMount", "RightStick"),
    ("DpadRadial1", "DPadUp"), ("DpadRadial3", "DPadDown"), ("DpadRadial4", "DPadLeft"), ("DpadRadial2", "DPadRight"),
    ("MenuUp", "DPadUp"), ("MenuDown", "DPadDown"), ("MenuLeft", "DPadLeft"), ("MenuRight", "DPadRight"),
];

/// Console (Xbox)'s gamepad arm (`:2448-2469`).
const CONSOLE_XBOX_GP: &[(&str, &str)] = &[
    ("MouseLeft", "RightTrigger"), ("MouseRight", "B"),
    ("Up", "LeftThumbstickUp"), ("Down", "LeftThumbstickDown"), ("Left", "LeftThumbstickLeft"), ("Right", "LeftThumbstickRight"),
    ("Jump", "A"), ("LockOn", "X"), ("Inventory", "Y"), ("Grapple", "LeftTrigger"),
    ("SmartSelect", "LeftStick"), ("SmartCursor", "RightStick"), ("HotbarMinus", "LeftShoulder"), ("HotbarPlus", "RightShoulder"),
    ("MapFull", "Start"), ("DpadRadial1", "DPadUp"), ("DpadRadial3", "DPadDown"), ("DpadRadial4", "DPadLeft"), ("DpadRadial2", "DPadRight"),
    ("QuickMount", "Back"),
];

/// `PlayerInput.Reset(c, style, mode)` (`PlayerInput.cs:2046-2500`) as data:
/// the add sequence for one preset+mode. `None`/`Mouse` add nothing (the
/// `Mouse` arms are empty `break`s, `:2159-2160` etc.).
pub fn preset_adds(style: PresetProfiles, mode: InputMode) -> &'static [(&'static str, &'static str)] {
    match mode {
        InputMode::Keyboard => {
            if style == PresetProfiles::None { &[] } else { KB }
        }
        InputMode::KeyboardUi => {
            if style == PresetProfiles::None { &[] } else { KB_UI }
        }
        InputMode::XBoxGamepad => match style {
            PresetProfiles::Redigit => REDIGIT_GP,
            PresetProfiles::Yoraiz0r => YORAIZ0R_GP,
            PresetProfiles::ConsolePs => CONSOLE_PS_GP,
            PresetProfiles::ConsoleXBox => CONSOLE_XBOX_GP,
            PresetProfiles::None => &[],
        },
        InputMode::XBoxGamepadUi => match style {
            PresetProfiles::Redigit => REDIGIT_GP_UI,
            PresetProfiles::Yoraiz0r => YORAIZ0R_GP_UI,
            PresetProfiles::ConsolePs | PresetProfiles::ConsoleXBox => CONSOLE_GP_UI,
            PresetProfiles::None => &[],
        },
        InputMode::Mouse => &[],
    }
}

/// `PlayerInput.Profiles` + `OriginalProfiles` + `_selectedProfile`
/// (`PlayerInput.cs:248-250`, `Initialize` `:722-755`): five editable
/// profiles in insertion order, four immutable originals (no "Custom"), and
/// "Custom" selected at boot (`:753`). The `Load` override (`:776-812`) is
/// the unported persistence.
#[derive(Clone, Debug)]
pub struct ProfileRegistry {
    /// `Profiles` — insertion-ordered (the `profileButtonClick` cycle walks
    /// `Profiles.Keys.ToList()`, `UIManageControls.cs:915`).
    pub profiles: Vec<PlayerInputProfile>,
    /// `OriginalProfiles` (`:741-752`) — the reset rows' copy sources.
    pub originals: Vec<PlayerInputProfile>,
    /// `_selectedProfile` (`:717`).
    pub selected: String,
}

/// The five profile names in `Initialize`'s insertion order (`:726-740`).
pub const PROFILE_NAMES: [&str; 5] = [
    "Redigit's Pick",
    "Yoraiz0r's Pick",
    "Console (Playstation)",
    "Console (Xbox)",
    "Custom",
];

impl Default for ProfileRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileRegistry {
    /// `PlayerInput.Initialize`'s profile half (`:726-753`): each preset
    /// seeds one editable profile; the first four presets seed the originals
    /// again; "Custom" initializes from Redigit and is selected.
    pub fn new() -> Self {
        let presets = [
            PresetProfiles::Redigit,
            PresetProfiles::Yoraiz0r,
            PresetProfiles::ConsolePs,
            PresetProfiles::ConsoleXBox,
            PresetProfiles::Redigit, // "Custom" (`:739`)
        ];
        let profiles: Vec<PlayerInputProfile> = PROFILE_NAMES
            .iter()
            .zip(presets)
            .map(|(name, preset)| {
                let mut p = PlayerInputProfile::new(name);
                p.initialize(preset);
                p
            })
            .collect();
        let originals = profiles[..4].to_vec();
        Self {
            profiles,
            originals,
            selected: "Custom".to_string(),
        }
    }

    /// `SetSelectedProfile` (`:713-720`): only a known name switches.
    pub fn set_selected_profile(&mut self, name: &str) {
        if self.profiles.iter().any(|p| p.name == name) {
            self.selected = name.to_string();
        }
    }

    /// `CurrentProfile` (`:339`-area getter) — the selected profile.
    pub fn current(&self) -> &PlayerInputProfile {
        self.profiles
            .iter()
            .find(|p| p.name == self.selected)
            .expect("SetSelectedProfile keeps a known name")
    }

    /// The mutable `CurrentProfile`.
    pub fn current_mut(&mut self) -> &mut PlayerInputProfile {
        let selected = self.selected.clone();
        self.profiles
            .iter_mut()
            .find(|p| p.name == selected)
            .expect("SetSelectedProfile keeps a known name")
    }

    /// `UIManageControls.profileButtonClick` (`UIManageControls.cs:912-922`):
    /// `IndexOf(name) + 1`, wrap at the count, `SetSelectedProfile`.
    pub fn cycle_profile(&mut self) {
        let idx = self
            .profiles
            .iter()
            .position(|p| p.name == self.selected)
            .unwrap_or(0);
        let mut next = idx + 1;
        if next >= self.profiles.len() {
            next -= self.profiles.len();
        }
        let name = self.profiles[next].name.clone();
        self.set_selected_profile(&name);
    }

    /// `UIManageControls.GetCopyableProfileName` (`:712-720`): the current
    /// name when it is an original preset, else "Redigit's Pick" ("Custom"
    /// resets from Redigit).
    pub fn copyable_profile_name(&self) -> &str {
        if self.originals.iter().any(|p| p.name == self.selected) {
            &self.selected
        } else {
            "Redigit's Pick"
        }
    }

    /// The original the reset rows copy from (`OriginalProfiles[name]`,
    /// `UIManageControls.cs:505` etc.).
    pub fn original(&self, name: &str) -> &PlayerInputProfile {
        self.originals
            .iter()
            .find(|p| p.name == name)
            .expect("GetCopyableProfileName only returns original names")
    }
}

/// `UILinksInitializer.HandleSliderHorizontalInput`
/// (`UILinksInitializer.cs:86-91`): the gamepad slider step. `stick_x` is
/// `PlayerInput.GamepadThumbstickLeft.X` passed in (the port has no gamepad
/// poll yet — Stage-1 exclusion — so the caller supplies it, 0 at rest).
/// Defaults: `deadZone = 0.2`, `sensitivity = 0.5` (the sp18/sp19 rows use
/// the 4-arg form).
pub fn handle_slider_horizontal_input(
    current_value: f32,
    min: f32,
    max: f32,
    dead_zone: f32,
    sensitivity: f32,
    stick_x: f32,
) -> f32 {
    // `:89` — inside the deadzone the step is 0; outside it lerps
    // 0..sensitivity/60 by the normalized overshoot, signed.
    let x = if !(stick_x < -dead_zone) && !(stick_x > dead_zone) {
        0.0
    } else {
        let t = (stick_x.abs() - dead_zone) / (1.0 - dead_zone);
        (sensitivity / 60.0) * t * stick_x.signum()
    };
    // `:90` — MathHelper.Clamp is min(max(v, min), max) with no NaN guard.
    let frac = ((current_value - min) / (max - min) + x).min(1.0).max(0.0);
    frac * (max - min) + min
}

/// `GlyphTagHandler.GlyphStyle` (`GlyphTagHandler.cs:145-147`, static ctor
/// `:184-188`): default -1 (auto). The style button cycles ++ then wraps
/// `> 2 → -1` (`UIManageControls.cs:903-910`).
pub const DEFAULT_GLYPH_STYLE: i32 = -1;

/// `controllerGlyphButtonClick` (`UIManageControls.cs:903-910`).
pub fn cycle_glyph_style(style: i32) -> i32 {
    let next = style + 1;
    if next > 2 { -1 } else { next }
}

/// `controllerGlyphStyle` (`UIManageControls.cs:892-901`): the button's
/// label key — -1 Auto, 1 PlayStation, 2 Switch, anything else (0) Xbox.
pub fn glyph_style_key(style: i32) -> &'static str {
    match style {
        -1 => "UI.ControllerGlyphAuto",
        1 => "UI.ControllerGlyphPlayStation",
        2 => "UI.ControllerGlyphSwitch",
        _ => "UI.ControllerGlyphXbox",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Initialize` (`PlayerInput.cs:726-753`): five profiles, four originals,
    /// "Custom" selected and Redigit-seeded; the Custom keyboard Jump is
    /// Space and the gamepad Grapple is B (Redigit, not Yoraiz0r).
    #[test]
    fn the_registry_boots_like_player_input_initialize() {
        let r = ProfileRegistry::new();
        assert_eq!(r.profiles.len(), 5);
        assert_eq!(r.originals.len(), 4);
        assert_eq!(r.selected, "Custom");
        assert_eq!(r.current().keys(InputMode::Keyboard, "Jump"), ["Space"]);
        assert_eq!(r.current().keys(InputMode::XBoxGamepad, "Grapple"), ["B"]);
        // SetupKeys seeded every trigger, even unbound ones.
        assert_eq!(r.current().input_modes[0].len(), KNOWN_TRIGGERS.len());
        assert!(r.current().keys(InputMode::Keyboard, "Dash").is_empty());
        // The two-add triggers keep their order.
        assert_eq!(r.current().keys(InputMode::KeyboardUi, "MouseLeft"), ["Mouse1", "Space"]);
        assert_eq!(r.current().keys(InputMode::XBoxGamepadUi, "Inventory"), ["B", "Y"]);
    }

    /// The preset differences (`Reset`'s gamepad arms): Yoraiz0r moves
    /// Grapple to LeftShoulder and adds RadialHotbar; the Consoles use the
    /// DpadRadial family and QuickMount=Back; Yoraiz0r's UI Inventory is Y
    /// only while Redigit's is B+Y.
    #[test]
    fn the_preset_tables_keep_their_per_preset_differences() {
        let r = ProfileRegistry::new();
        let yor = &r.profiles[1];
        assert_eq!(yor.keys(InputMode::XBoxGamepad, "Grapple"), ["LeftShoulder"]);
        assert_eq!(yor.keys(InputMode::XBoxGamepad, "RadialHotbar"), ["RightShoulder"]);
        assert!(yor.keys(InputMode::XBoxGamepad, "LockOn").is_empty());
        assert_eq!(yor.keys(InputMode::XBoxGamepadUi, "Inventory"), ["Y"]);
        assert_eq!(yor.keys(InputMode::XBoxGamepadUi, "LockOn"), ["B"]);
        let ps = &r.profiles[2];
        assert_eq!(ps.keys(InputMode::XBoxGamepad, "MouseLeft"), ["RightShoulder"]);
        assert_eq!(ps.keys(InputMode::XBoxGamepad, "DpadRadial1"), ["DPadUp"]);
        assert!(ps.keys(InputMode::XBoxGamepad, "DpadSnap1").is_empty());
        let xb = &r.profiles[3];
        assert_eq!(xb.keys(InputMode::XBoxGamepad, "Grapple"), ["LeftTrigger"]);
        assert_eq!(xb.keys(InputMode::XBoxGamepad, "HotbarMinus"), ["LeftShoulder"]);
        // The Console UI twins are identical.
        assert_eq!(ps.input_modes[3], xb.input_modes[3]);
        // The keyboard arm is shared by all four presets.
        assert_eq!(r.profiles[0].input_modes[0], r.profiles[3].input_modes[0]);
    }

    /// `profileButtonClick` (`UIManageControls.cs:912-922`): the cycle walks
    /// the insertion order and wraps Custom → Redigit's Pick.
    #[test]
    fn the_profile_button_cycles_and_wraps() {
        let mut r = ProfileRegistry::new();
        r.cycle_profile(); // Custom (index 4) wraps to 0
        assert_eq!(r.selected, "Redigit's Pick");
        r.cycle_profile();
        assert_eq!(r.selected, "Yoraiz0r's Pick");
        // GetCopyableProfileName: originals copy from themselves, Custom
        // falls back to Redigit's Pick.
        assert_eq!(r.copyable_profile_name(), "Yoraiz0r's Pick");
        r.set_selected_profile("Custom");
        assert_eq!(r.copyable_profile_name(), "Redigit's Pick");
        r.set_selected_profile("Not A Profile");
        assert_eq!(r.selected, "Custom", "SetSelectedProfile ignores unknown names");
    }

    /// The reset rows (`CopyKeysFrom` `:325-335` + the key lists): a rebind
    /// is undone by sp9, the hotbar/map lists are disjoint, sp12 copies BOTH
    /// gamepad modes, sp13 copies fields not keys.
    #[test]
    fn the_reset_rows_copy_the_source_lists() {
        let mut r = ProfileRegistry::new();
        let redigit = r.original("Redigit's Pick").clone();
        {
            let cur = r.current_mut();
            cur.input_modes[0].get_mut("Jump").unwrap().clear();
            cur.input_modes[0].get_mut("Jump").unwrap().push("Z".into());
            cur.inventory_move_cd = 12;
            cur.triggers_deadzone = 0.9;
        }
        let mode = InputMode::Keyboard;
        let source = redigit.clone();
        r.current_mut().copy_gameplay_settings_from(&source, mode);
        assert_eq!(r.current().keys(mode, "Jump"), ["Space"]);
        assert_eq!(r.current().inventory_move_cd, 12, "sp9 copies keys only");
        r.current_mut().copy_gamepad_advanced_settings_from(&source, mode);
        assert_eq!(r.current().inventory_move_cd, 6);
        assert_eq!(r.current().triggers_deadzone, 0.3);
        // sp12: the radial family lands in both gamepad modes.
        let ps = r.original("Console (Playstation)").clone();
        r.current_mut().copy_gamepad_settings_from(&ps, mode);
        assert_eq!(r.current().keys(InputMode::XBoxGamepad, "DpadRadial2"), ["DPadRight"]);
        assert_eq!(r.current().keys(InputMode::XBoxGamepadUi, "DpadRadial4"), ["DPadLeft"]);
    }

    /// The sp1/sp2 statuses (`UsingDpadMovekeys`/`UsingDpadHotbar`): the
    /// Redigit seed binds the Snap family in both modes (sp1 on, sp2 off);
    /// the Console seed binds Radial (sp2 on, sp1 off).
    #[test]
    fn the_dpad_toggle_statuses_read_both_gamepad_modes() {
        let r = ProfileRegistry::new();
        assert!(r.profiles[0].using_dpad_movekeys());
        assert!(!r.profiles[0].using_dpad_hotbar());
        assert!(!r.profiles[2].using_dpad_movekeys());
        assert!(r.profiles[2].using_dpad_hotbar());
        // One missing direction in ONE mode breaks the chain.
        let mut p = r.profiles[0].clone();
        p.input_modes[3].get_mut("DpadSnap4").unwrap().clear();
        assert!(!p.using_dpad_movekeys());
    }

    /// `HandleSliderHorizontalInput` (`:86-91`): inside the deadzone nothing
    /// moves; a full-deflection stick steps sensitivity/60 per call toward
    /// the clamp; the value never leaves [min, max].
    #[test]
    fn the_slider_step_deadzones_and_clamps() {
        // At rest the value is unchanged (but re-emitted through the clamp).
        assert_eq!(handle_slider_horizontal_input(0.3, 0.0, 0.95, 0.2, 0.35, 0.0), 0.3);
        // Inside the deadzone: no step.
        assert_eq!(handle_slider_horizontal_input(0.3, 0.0, 0.95, 0.2, 0.35, 0.19), 0.3);
        // Full right deflection: (0.35/60) * (1-0.2)/(1-0.2) = 0.35/60 of the range.
        let stepped = handle_slider_horizontal_input(0.0, 0.0, 1.0, 0.2, 0.5, 1.0);
        assert!((stepped - 0.5 / 60.0).abs() < 1e-6, "got {stepped}");
        // Negative stick steps down and clamps at min.
        assert_eq!(handle_slider_horizontal_input(0.0, 0.0, 1.0, 0.2, 0.5, -1.0), 0.0);
        // Top clamp.
        assert_eq!(handle_slider_horizontal_input(1.0, 0.0, 1.0, 0.2, 0.5, 1.0), 1.0);
    }

    /// The cycle rings: pan Hold→Click→OnAlways→OffAlways→Hold
    /// (`Utils.cs:159-176`), dash two-state (`UIManageControls.cs:645-653`),
    /// glyph -1→0→1→2→-1 (`:903-910`) with the label keys (`:892-901`).
    #[test]
    fn the_cycle_rings_walk_their_source_orders() {
        let mut m = ButtonControlMode::Hold;
        for expect in [ButtonControlMode::Click, ButtonControlMode::OnAlways, ButtonControlMode::OffAlways, ButtonControlMode::Hold] {
            m = cycle_control_off_on_click_hold(m);
            assert_eq!(m, expect);
        }
        assert_eq!(cycle_dash_control(DashPreference::AllowDoubleTap), DashPreference::OnlyThroughHotkeys);
        assert_eq!(cycle_dash_control(DashPreference::OnlyThroughHotkeys), DashPreference::AllowDoubleTap);
        let mut g = DEFAULT_GLYPH_STYLE;
        for expect in [0, 1, 2, -1] {
            g = cycle_glyph_style(g);
            assert_eq!(g, expect);
        }
        assert_eq!(glyph_style_key(-1), "UI.ControllerGlyphAuto");
        assert_eq!(glyph_style_key(0), "UI.ControllerGlyphXbox");
        assert_eq!(glyph_style_key(1), "UI.ControllerGlyphPlayStation");
        assert_eq!(glyph_style_key(2), "UI.ControllerGlyphSwitch");
    }
}
