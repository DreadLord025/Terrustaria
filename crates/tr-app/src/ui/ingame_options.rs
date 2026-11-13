//! The in-game options window — `IngameOptions` (`decompiled/Terraria/IngameOptions.cs`,
//! drawn from `Main.DrawInterface_11_IngameOptionsMenu`, `Main.cs:46160-46168`).
//!
//! The vanilla window is a 670x480 panel split in two columns
//! (`IngameOptions.Draw`, `IngameOptions.cs:103-1441`):
//!
//! * the **left** column is the navigation list — `General`, `Interface`, `Video`,
//!   `Controls`, plus the action rows `Keybinds`, `Achievements`, `Close Menu`,
//!   `Save & Exit` (`:214-330`; the `Invite Friends` row is the social-API arm
//!   `flag5 = SocialAPI.Network != null && SocialAPI.Network.CanInvite()`
//!   (`:178`, `num7 = 5 + (flag5 ? 1 : 0) + 2` `:179`) and stays out — AGENTS.md
//!   non-goals, so the port's eight rows are the vanilla's no-social count);
//! * the **right** column is the selected category's page, one row per setting
//!   (`:400-1441`, 17/14/14/15 slots by `num11` at `:296-330`).
//!
//! Both columns animate a per-row scale towards a hover plateau — the left
//! `0.7..0.8` at `0.01`/frame (`:157-199`), the right by the same rule against the
//! per-category `num4/num5/num6` (`:333-390`), with the locale shrink the vanilla
//! applies for the long translated labels (`:105-120`, `:345-357`).
//!
//! The port mounts every row whose setting exists in [`crate::settings::Settings`];
//! the rows whose state the port does not model yet (window mode / resolution, the
//! colour pickers, the door-opening preference, auto-reuse) are drawn as the
//! vanilla's disabled-until-implemented rows and listed in
//! `docs/ANALYSIS.md` "The in-game options window (M7-UI-4)".

use crate::assets::Lookups;
use crate::settings::Settings;
use crate::ui::sliders;
use crate::ui::text;
use glam::Vec2;
use tr_core::localization::GameCulture;
use tr_render::SpriteRenderer;

/// The C# panel size (`IngameOptions.cs:126`'s `new Vector2(670f, 480f)`).
pub const PANEL_W: f32 = 670.0;
pub const PANEL_H: f32 = 480.0;
/// `num3 = 20` (`:127`) — the padding and the left/right layout unit.
const PAD: f32 = 20.0;

/// The left column's label anchor: `vector4.X = vector3.X + vector2.X / 4f`
/// (`:207`) and `vector4.Y = vector3.Y + num3 * 5 / 2` (`:208`).
pub const LEFT_ANCHOR_X: f32 = PANEL_W / 4.0;
pub const COLUMN_ANCHOR_Y: f32 = PAD * 5.0 / 2.0;
/// The left column's row offset denominator is `num7 + 1` with
/// `num7 = 5 + 2 = 7` (`:203-206`: the five categories + `Close Menu` + `Save &
/// Exit` + the achievements row, without the social invite arm).
pub const LEFT_SLOTS: usize = 8;
/// The right column's label anchor: `vector6.X = vector3.X + vector2.X * 3f / 4f`
/// (`:359`), `vector6.Y = vector3.Y + num3 * 5 / 2` (`:360`).
pub const RIGHT_ANCHOR_X: f32 = PANEL_W * 3.0 / 4.0;

/// `num` — the 70 px the C# shoves the right column left when a value bar is
/// drawn on its row (`vector6.X -= num`, `:417`, returned by `:418`).
const BAR_SHIFT: f32 = 70.0;

/// The C# `num11` — the right column's slot count per category (`:296-330`).
/// The video page draws 15 rows into a 14-slot pitch and compensates with its own
/// `vector7.Y -= 2f` (`:362-365`); both quirks are kept verbatim.
pub fn right_slots(category: u8) -> usize {
    match category {
        0 => 17,
        1 => 14,
        2 => 14,
        _ => 15,
    }
}

/// `IngameOptions.DrawLeftSide`'s scale band (`:1457-1481`): `minscale` and
/// `maxscale` default to 0.7/0.8 at `scalespeed = 0.01`.
pub const LEFT_SCALE_MIN: f32 = 0.7;
pub const LEFT_SCALE_MAX: f32 = 0.8;
pub const LEFT_SCALE_SPEED: f32 = 0.01;

/// The left column's row labels, in the C# order (`:214-330`):
/// `Lang.menu[114]` General, `[210]` Interface, `[63]` Video, `[218]` Controls,
/// `[66]` Controls→keybinds, `[131]` Achievements, `[118]` Close Menu,
/// `Lang.inter[35]` Save & Exit.
pub const LEFT_ROWS: [LeftRow; 8] = [
    LeftRow { menu: Some(114), inter: None, action: LeftAction::Category(0) },
    LeftRow { menu: Some(210), inter: None, action: LeftAction::Category(1) },
    LeftRow { menu: Some(63), inter: None, action: LeftAction::Category(2) },
    LeftRow { menu: Some(218), inter: None, action: LeftAction::Category(3) },
    LeftRow { menu: Some(66), inter: None, action: LeftAction::Keybinds },
    LeftRow { menu: Some(131), inter: None, action: LeftAction::Achievements },
    LeftRow { menu: Some(118), inter: None, action: LeftAction::Close },
    LeftRow { menu: None, inter: Some(35), action: LeftAction::SaveAndExit },
];

/// One left-column navigation row (`IngameOptions.DrawLeftSide`'s caller args,
/// `:214-330`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LeftRow {
    /// The `Lang.menu[i]` key (`:214-306`).
    pub menu: Option<u16>,
    /// The `Lang.inter[i]` key (`:311`'s Save & Exit).
    pub inter: Option<u16>,
    pub action: LeftAction,
}

/// What a left row does on click (`:219-330`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeftAction {
    /// `IngameOptions.category = n` + `SoundEngine.PlaySound(10)` (`:222`).
    Category(u8),
    /// `Close()` + `IngameFancyUI.OpenKeybinds()` (`:252-256`) — M7-UI-5.
    Keybinds,
    /// `Close()` + `IngameFancyUI.OpenAchievements()` (`:288-292`).
    Achievements,
    /// `Close()` (`:295-299`).
    Close,
    /// `Close()` + `Main.menuMode = 10` + `WorldGen.SaveAndQuit()` (`:300-310`).
    SaveAndExit,
}

/// One right-column row. `act` decides how it draws and clicks; the label is
/// resolved through [`crate::lang`] so the port follows the active culture.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Row {
    pub label: Label,
    pub act: RowAct,
}

/// The label of a right-column row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Label {
    /// `Lang.menu[i]` (`Lang.cs:481`'s `LegacyMenu.{i}`).
    Menu(u16),
    /// `Lang.inter[i]` (`:485`'s `LegacyInterface.{i}`).
    Inter(u16),
    /// A `Language.GetTextValue(key)` row.
    Text(&'static str),
    /// A bare `""` spacer (`:507`, `:602`, `:686`).
    Blank,
}

/// What a right row is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowAct {
    /// A section caption (`Lang.menu[65]` Volume, `GameUI.Gameplay`, …): drawn,
    /// `skipRightSlot = true`, never clickable (`:404-406`).
    Header,
    /// A `DrawValueBar` row.
    Bar(Bar),
    /// An on/off row whose label flips between two keys.
    Toggle(Toggle),
    /// A multi-state row (`Lang.menu[247 + mode]`, `Lang.menu[59 + qaStyle]`, …).
    Cycle(Cycle),
    /// A vanilla row the port does not model yet — drawn dimmed, inert.
    Parked,
}

/// The value bars (`DrawValueBar`, `:1537-1603`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bar {
    /// `Lang.menu[99]` + `Main.musicVolume` (`:407-439`) → `music_volume`.
    Music,
    /// `Lang.menu[98]` + `Main.soundVolume` (`:440-472`) → `sound_volume`.
    Sound,
    /// `Lang.menu[119]` + `Main.ambientVolume` (`:473-506`) → `ambient_volume`.
    Ambient,
    /// `GameUI.GameZoom` + `Main.GameZoomTarget - 1f` (`:510-560`) → `zoom`.
    Zoom,
    /// `GameUI.UIScale` + `Main.UIScale` (`:561-601`) → `ui_scale` (the wanted
    /// scale; the bar's track spans the C# slider's 0.5..2.0 range).
    UiScale,
    /// `Lang.menu[52]` + `Main.bgScroll` (`:911-945`) → `parallax_scroll`.
    Parallax,
}

/// The on/off rows, mapped onto the port's [`Settings`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Toggle {
    /// `Lang.menu[67]/[68]` + `Main.autoSave` (`:608-616`).
    AutoSave,
    /// `Lang.menu[69]/[70]` + `Main.autoPause` (`:617-625`).
    AutoPause,
    /// `UI.PlayWhenUnfocusedOn/Off` + `Main.SettingPlayWhenUnfocused` (`:626-634`).
    PlayWhenUnfocused,
    /// `Lang.menu[220]/[221]` + `Main.ReversedUpDownArmorSetBonuses` (`:635-643`).
    ReversedArmorBonuses,
    /// `Lang.menu[71]/[72]` + `Main.hidePlayerCraftingMenu`-style pickup text
    /// (`:693-701`, the Interface page's first row).
    PickupText,
    /// `Lang.menu[123]` `Main.showItemToolTips`-paired progress-bar row
    /// (`:702-714`).
    EventProgress,
    /// `Lang.menu[128]/[129]` + `Main.placePreview` (`:715-723`).
    PlacementPreview,
    /// `Lang.inter[117]/[116]` + `Main.highlightNewItems` (`:724-732`).
    HighlightNewItems,
    /// `Lang.menu[229]/[230]` + `Main.tileGrid` (`:733-741`).
    TileGrid,
    /// `Lang.menu[241]/[242]` + `SettingsGamepad` (`:742-750`).
    GamepadInstructions,
    /// The big progress bar's text (`:823-831`) → `boss_bar_text`.
    BossBarText,
    /// `Main.FlashyEffectsInterface` (`:863-871`) → `flashy_effects_interface`.
    FlashyEffectsInterface,
    /// `UI.AutoReuseAll{On,Off}` + `Main.SettingsEnabled_AutoReuseAllItems`
    /// (`:677-684`) → `auto_reuse_all`.
    AutoReuseAll,
    /// `Lang.menu[100]/[101]` + `Main.BackgroundEnabled` (`:977-985`).
    Background,
    /// `Lang.menu[132]/[133]` + `ChildSafety.Disabled` (`:986-994`).
    Blood,
    /// `GameUI.ForegroundSunlightEffects` (`:995-1003`).
    ForegroundSunlight,
    /// `GameUI.HeatDistortion` (`:1004-1012`).
    HeatDistortion,
    /// `GameUI.StormEffects` (`:1013-1021`).
    StormEffects,
    /// `UI.TilesSwayInWind` (`:1037-1045`).
    TilesSway,
    /// `GameUI.ScreenShake` (`:1046-1054`).
    ScreenShake,
    /// `Main.FlashyEffectsWorld` (`:1055-1063`).
    FlashyEffectsWorld,
}

/// The multi-state rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cycle {
    /// `Lang.menu[247 + Main.FrameSkipMode]` (`:946-954`) → `frame_skip`.
    FrameSkip,
    /// `"UI.LightMode_" + Lighting.Mode` (`:955-963`) → `light_mode`.
    LightMode,
    /// `Lang.menu[59 + Main.qaStyle]` (`:964-976`) → `quality`.
    Quality,
    /// `GameUI.WaveQuality` (`:1022-1036`) → `wave_quality`.
    WaveQuality,
    /// `Lang.menu[253/224/223]` — the quick-trash key (`:841-862`).
    QuickTrash,
    /// `Lang.menu[215/216]` — smart block placement (`:1381-1389`).
    SmartBlocks,
    /// `Lang.menu[121/122]` — smart cursor mode (`:1390-1398`).
    SmartCursorMode,
    /// `Lang.menu[214/213]` — the smart-cursor priority (`:1399-1440`).
    SmartCursorPriority,
    /// `UI.SmartDoors{Disabled,Gamepad,Enabled}` — `DoorOpeningHelper`'s
    /// `PreferenceSettings` (`:644-657`, `DoorOpeningHelper.cs:262-276`).
    SmartDoors,
    /// `UI.HoverControlSettingIs{Hold,Click}` — `Player.Settings.HoverControl`
    /// (`:659-675`, `Player.cs:379-383`).
    HoverControl,
}

/// The page table for a category (`:400-1441`). Rows are in the vanilla order and
/// every one carries its C# line range on its [`RowAct`] variant.
pub fn page(category: u8) -> &'static [Row] {
    match category {
        0 => GENERAL,
        1 => INTERFACE,
        2 => VIDEO,
        _ => CONTROLS,
    }
}

/// The `General` page (`:400-689`, `num11 = 17`).
static GENERAL: &[Row] = &[
    // 0 `Lang.menu[65]` "Volume" (`:403`).
    Row { label: Label::Menu(65), act: RowAct::Header },
    // 1 `Lang.menu[99]` music bar (`:407`).
    Row { label: Label::Menu(99), act: RowAct::Bar(Bar::Music) },
    // 2 `Lang.menu[98]` sound bar (`:440`).
    Row { label: Label::Menu(98), act: RowAct::Bar(Bar::Sound) },
    // 3 `Lang.menu[119]` ambient bar (`:473`).
    Row { label: Label::Menu(119), act: RowAct::Bar(Bar::Ambient) },
    // 4 `""` spacer (`:507`).
    Row { label: Label::Blank, act: RowAct::Header },
    // 5 `GameUI.ZoomCategory` (`:510`).
    Row { label: Label::Text("GameUI.ZoomCategory"), act: RowAct::Header },
    // 6 `GameUI.GameZoom` bar (`:519`).
    Row { label: Label::Text("GameUI.GameZoom"), act: RowAct::Bar(Bar::Zoom) },
    // 7 `GameUI.UIScale` bar (`:561`).
    Row { label: Label::Text("GameUI.UIScale"), act: RowAct::Bar(Bar::UiScale) },
    // 8 `""` spacer (`:602`).
    Row { label: Label::Blank, act: RowAct::Header },
    // 9 `GameUI.Gameplay` (`:605`).
    Row { label: Label::Text("GameUI.Gameplay"), act: RowAct::Header },
    // 10 autosave (`:608`).
    Row { label: Label::Menu(67), act: RowAct::Toggle(Toggle::AutoSave) },
    // 11 autopause (`:617`).
    Row { label: Label::Menu(69), act: RowAct::Toggle(Toggle::AutoPause) },
    // 12 play-when-unfocused (`:626`).
    Row {
        label: Label::Text("UI.PlayWhenUnfocusedOn"),
        act: RowAct::Toggle(Toggle::PlayWhenUnfocused),
    },
    // 13 set-bonus direction (`:635`).
    Row { label: Label::Menu(220), act: RowAct::Toggle(Toggle::ReversedArmorBonuses) },
    // 14 the door-opening preference (`:644-657`) — a cycle over
    // `DoorAutoOpeningMode`.
    Row { label: Label::Blank, act: RowAct::Cycle(Cycle::SmartDoors) },
    // 15 hover control (`:659-675`).
    Row { label: Label::Blank, act: RowAct::Cycle(Cycle::HoverControl) },
    // 16 autofire-all (`:677-684`).
    Row { label: Label::Blank, act: RowAct::Toggle(Toggle::AutoReuseAll) },
    // 16 `""` spacer (`:686`).
    Row { label: Label::Blank, act: RowAct::Header },
];

/// The `Interface` page (`:690-872`, `num11 = 14`).
static INTERFACE: &[Row] = &[
    // 0 pickup text (`:693`).
    Row { label: Label::Menu(71), act: RowAct::Toggle(Toggle::PickupText) },
    // 1 event progress bar (`:702`).
    Row { label: Label::Menu(123), act: RowAct::Toggle(Toggle::EventProgress) },
    // 2 placement preview (`:715`).
    Row { label: Label::Menu(128), act: RowAct::Toggle(Toggle::PlacementPreview) },
    // 3 highlight new items (`:724`).
    Row { label: Label::Inter(117), act: RowAct::Toggle(Toggle::HighlightNewItems) },
    // 4 tile grid (`:733`).
    Row { label: Label::Menu(229), act: RowAct::Toggle(Toggle::TileGrid) },
    // 5 gamepad instructions (`:742`).
    Row { label: Label::Menu(241), act: RowAct::Toggle(Toggle::GamepadInstructions) },
    // 6 `UI.SelectMapBorder` — the map-frame cycle (`:751-765`) — no port field.
    Row { label: Label::Text("UI.SelectMapBorder"), act: RowAct::Parked },
    // 7 the map-frame colour picker (`:766-798`).
    Row { label: Label::Blank, act: RowAct::Parked },
    // 8 `UI.SelectHealthStyle` (`:799-807`).
    Row { label: Label::Text("UI.SelectHealthStyle"), act: RowAct::Parked },
    // 9 the dialogue-portrait preference (`:808-822`).
    Row { label: Label::Text("UI.PortraitPreference"), act: RowAct::Parked },
    // 10 the big progress-bar text (`:823-831`).
    Row { label: Label::Text("GameUI.BigProgressBar"), act: RowAct::Toggle(Toggle::BossBarText) },
    // 11 the opaque tooltip box (`:832-840`).
    Row { label: Label::Text("UI.OpaqueBoxBehindTooltips"), act: RowAct::Parked },
    // 12 the quick-trash key (`:841-862`).
    Row { label: Label::Menu(253), act: RowAct::Cycle(Cycle::QuickTrash) },
    // 13 flashy interface effects (`:863-871`).
    Row {
        label: Label::Text("UI.FlashyEffectsInterface"),
        act: RowAct::Toggle(Toggle::FlashyEffectsInterface),
    },
];

/// The `Video` page (`:873-1064`, `num11 = 14` and 15 drawn rows — the vanilla's
/// own `vector7.Y -= 2f` compensation at `:362-365`).
static VIDEO: &[Row] = &[
    // 0 windowed/fullscreen (`:876-884`) — the port has no display-mode switch.
    Row { label: Label::Menu(49), act: RowAct::Parked },
    // 1 resolution (`:885-910`) — no port field.
    Row { label: Label::Menu(51), act: RowAct::Parked },
    // 2 parallax (`:911-945`).
    Row { label: Label::Menu(52), act: RowAct::Bar(Bar::Parallax) },
    // 3 frame skip (`:946-954`).
    Row { label: Label::Menu(247), act: RowAct::Cycle(Cycle::FrameSkip) },
    // 4 lighting mode (`:955-963`).
    Row { label: Label::Text("UI.LightMode_0"), act: RowAct::Cycle(Cycle::LightMode) },
    // 5 quality (`:964-976`).
    Row { label: Label::Menu(59), act: RowAct::Cycle(Cycle::Quality) },
    // 6 background (`:977-985`).
    Row { label: Label::Menu(100), act: RowAct::Toggle(Toggle::Background) },
    // 7 blood and gore (`:986-994`).
    Row { label: Label::Menu(132), act: RowAct::Toggle(Toggle::Blood) },
    // 8 foreground sunlight (`:995-1003`).
    Row { label: Label::Text("GameUI.ForegroundSunlightEffects"), act: RowAct::Toggle(Toggle::ForegroundSunlight) },
    // 9 heat distortion (`:1004-1012`).
    Row { label: Label::Text("GameUI.HeatDistortion"), act: RowAct::Toggle(Toggle::HeatDistortion) },
    // 10 storm effects (`:1013-1021`).
    Row { label: Label::Text("GameUI.StormEffects"), act: RowAct::Toggle(Toggle::StormEffects) },
    // 11 wave quality (`:1022-1036`).
    Row { label: Label::Text("GameUI.WaveQuality"), act: RowAct::Cycle(Cycle::WaveQuality) },
    // 12 tiles sway in wind (`:1037-1045`).
    Row { label: Label::Text("UI.TilesSwayInWind"), act: RowAct::Toggle(Toggle::TilesSway) },
    // 13 screen shake (`:1046-1054`).
    Row { label: Label::Text("GameUI.ScreenShake"), act: RowAct::Toggle(Toggle::ScreenShake) },
    // 14 flashy world effects (`:1055-1063`).
    Row { label: Label::Text("UI.FlashyEffectsWorld"), act: RowAct::Toggle(Toggle::FlashyEffectsWorld) },
];

/// The `Controls` page (`:1065-1441`, `num11 = 15`): the cursor-colour block
/// (`:1075-1202`), the border-colour block (`:1203-1355`), a caption row
/// (`:1356-1371`) and the three smart-cursor rows (`:1372-1440`).
static CONTROLS: &[Row] = &[
    // 0 `Lang.menu[64]` "Cursor Color" (`:1075`).
    Row { label: Label::Menu(64), act: RowAct::Header },
    // 1-4 the four cursor-colour bars R/G/B/A (`:1079-1194`).
    Row { label: Label::Text("UI.MouseColorR"), act: RowAct::Parked },
    Row { label: Label::Text("UI.MouseColorG"), act: RowAct::Parked },
    Row { label: Label::Text("UI.MouseColorB"), act: RowAct::Parked },
    Row { label: Label::Text("UI.MouseColorA"), act: RowAct::Parked },
    // 5 `Lang.menu[217]` "Border Color" (`:1203`).
    Row { label: Label::Menu(217), act: RowAct::Header },
    // 6-10 the four border-colour bars + the caption row (`:1207-1371`).
    Row { label: Label::Text("UI.MouseBorderR"), act: RowAct::Parked },
    Row { label: Label::Text("UI.MouseBorderG"), act: RowAct::Parked },
    Row { label: Label::Text("UI.MouseBorderB"), act: RowAct::Parked },
    Row { label: Label::Text("UI.MouseBorderA"), act: RowAct::Parked },
    Row { label: Label::Blank, act: RowAct::Parked },
    // 11-14 the smart-cursor block (`:1372-1440`).
    Row { label: Label::Menu(215), act: RowAct::Cycle(Cycle::SmartBlocks) },
    Row { label: Label::Menu(121), act: RowAct::Cycle(Cycle::SmartCursorMode) },
    Row { label: Label::Menu(214), act: RowAct::Cycle(Cycle::SmartCursorPriority) },
    Row { label: Label::Blank, act: RowAct::Header },
];

/// `SoundEngine.PlaySound` ids the window emits (`:421-436`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cue {
    /// `SoundEngine.PlaySound(12)` — the hover tick (`:187-195`).
    Tick,
    /// `SoundEngine.PlaySound(10)` — a category click (`:222`).
    Open,
}

/// The live window state, the port's `IngameOptions` statics (`:22-54`).
#[derive(Clone, Debug)]
pub struct IngameOptions {
    /// `Main.ingameOptionsWindow` (`Main.cs:2025`).
    pub open: bool,
    /// `IngameOptions.category` (`:48`) — 0 General, 1 Interface, 2 Video, 3 Controls.
    pub category: u8,
    /// `IngameOptions.leftHover` (`:30`), `-1` when none.
    pub left_hover: i32,
    /// `IngameOptions.rightHover` (`:32`), `-1` when none.
    pub right_hover: i32,
    /// `IngameOptions.rightLock` (`:38`) — the sticky bar grab.
    pub right_lock: i32,
    /// `IngameOptions.leftScale` (`:22`).
    pub left_scale: [f32; LEFT_SLOTS],
    /// `IngameOptions.rightScale` (`:24`) — 17 slots is the widest page.
    pub right_scale: [f32; 17],
    /// The action a left row asked for, drained by the caller (the vanilla calls
    /// `IngameFancyUI.OpenKeybinds`/`OpenAchievements`/`SaveAndQuit` inline).
    pub pending: Option<LeftAction>,
    /// `Main.exitScale` (`Main.cs:45541`): the Settings button's animated scale,
    /// `0.8` at rest and `0.96` hovered.
    pub button_scale: f32,
    /// `Main.mouseExit` (`:45540`): whether the button was hovered last frame.
    pub button_mouse_over: bool,
    /// `Main._settingsButtonIsPushedToSide` (`:45520`), refreshed every frame by
    /// [`refresh_button`] and read by the inventory's accessory column count.
    pub button_pushed_to_side: bool,
    /// The button geometry [`refresh_button`] measured this frame — the draw and
    /// the click hit test both read it, so neither can drift from the other.
    pub button_layout: ButtonLayout,
    old_left_hover: i32,
    old_right_hover: i32,
}

impl Default for IngameOptions {
    fn default() -> Self {
        Self {
            open: false,
            category: 0,
            left_hover: -1,
            right_hover: -1,
            right_lock: -1,
            // `leftScale`/`rightScale` start at their `minscale` (`:157-199`,
            // `:333-380` subtract then clamp).
            left_scale: [LEFT_SCALE_MIN; LEFT_SLOTS],
            right_scale: [0.0; 17],
            pending: None,
            button_scale: BUTTON_SCALE_MIN,
            button_mouse_over: false,
            button_pushed_to_side: false,
            button_layout: ButtonLayout {
                pos: [0.0; 2],
                w: 0.0,
                h: 0.0,
                word_scale: 1.0,
                pushed_to_side: false,
            },
            old_left_hover: -1,
            old_right_hover: -1,
        }
    }
}

impl IngameOptions {
    pub fn new() -> Self {
        Self::default()
    }

    /// `IngameOptions.Open` (`:56-83`) / `Close` (`:89-101`).
    pub fn toggle(&mut self) {
        if self.open {
            self.close();
        } else {
            self.open = true;
        }
    }

    pub fn close(&mut self) {
        self.open = false;
        self.right_lock = -1;
        self.pending = None;
    }
}

/// The `flag` of `:105-110` — the cultures whose long labels shrink the columns.
fn long_label_culture() -> bool {
    matches!(
        crate::lang::active_culture(),
        GameCulture::Russian
            | GameCulture::Portuguese
            | GameCulture::Polish
            | GameCulture::French
    )
}

fn german_culture() -> bool {
    matches!(crate::lang::active_culture(), GameCulture::German)
}

fn italian_or_spanish() -> bool {
    matches!(
        crate::lang::active_culture(),
        GameCulture::Italian | GameCulture::Spanish
    )
}

/// The left column's `(minscale, maxscale, scalespeed)` — `num4 = 0.7`,
/// `num5 = 0.8`, `num6 = 0.01` (`:157-166`) with the long-culture shrink
/// (`:167-180`: 0.4/0.44) and the German one (`:181-185`: 0.55/0.6).
pub fn left_scale_band() -> (f32, f32, f32) {
    let mut min = LEFT_SCALE_MIN;
    let mut max = LEFT_SCALE_MAX;
    if long_label_culture() {
        min = 0.4;
        max = 0.44;
    }
    if german_culture() {
        min = 0.55;
        max = 0.6;
    }
    (min, max, LEFT_SCALE_SPEED)
}

/// The right column's `(minscale, maxscale, scalespeed)` — the per-category
/// `num4/num5/num6` switch (`:296-343`, all categories 1.0/1.001/0.001) with the
/// same culture shrink (`:345-357`; the German/It-Sp deltas are category-bound).
pub fn right_scale_band(category: u8) -> (f32, f32, f32) {
    let mut min = 1.0f32;
    let mut max = 1.001f32;
    if long_label_culture() {
        min -= 0.1;
        max -= 0.1;
    }
    if german_culture() && category == 3 {
        min -= 0.15;
        max -= 0.15;
    }
    if italian_or_spanish() && (category == 0 || category == 3) {
        min -= 0.2;
        max -= 0.2;
    }
    (min, max, 0.001)
}

/// `vector3 = vector / 2f - vector2 / 2f` (`:125-126`).
pub fn panel_origin(screen: Vec2) -> [f32; 2] {
    [
        screen.x * 0.5 - PANEL_W * 0.5,
        screen.y * 0.5 - PANEL_H * 0.5,
    ]
}

/// `IngameOptions._GUIHover` (`:128-129`): the padded panel.
pub fn panel_rect(screen: Vec2) -> [f32; 4] {
    let o = panel_origin(screen);
    [
        o[0] - PAD,
        o[1] - PAD,
        PANEL_W + PAD * 2.0,
        PANEL_H + PAD * 2.0,
    ]
}

/// XNA `Rectangle.Contains(Point)` (`:1557`): inclusive left/top, exclusive
/// right/bottom.
pub fn contains(p: Vec2, rect: [f32; 4]) -> bool {
    p.x >= rect[0] && p.x < rect[0] + rect[2] && p.y >= rect[1] && p.y < rect[1] + rect[3]
}

/// `vector4` (`:207-208`): the left column's label anchor, shifted 55 px left for
/// the long cultures (`:210-212`).
pub fn left_anchor(screen: Vec2) -> [f32; 2] {
    let o = panel_origin(screen);
    let x = if long_label_culture() {
        o[0] + LEFT_ANCHOR_X - 55.0
    } else {
        o[0] + LEFT_ANCHOR_X
    };
    [x, o[1] + COLUMN_ANCHOR_Y]
}

/// `vector5 = (0, vector2.Y - num3 * 5) / (num7 + 1)` (`:209`): the left pitch.
pub fn left_pitch() -> f32 {
    (PANEL_H - PAD * 5.0) / (LEFT_SLOTS as f32 + 1.0)
}

/// Row `i`'s label anchor in the left column — `anchor + offset * (1 + i)`
/// (`DrawLeftSide`'s `anchor + offset * (1 + i)`, `:1468`).
pub fn left_row_anchor(screen: Vec2, i: usize) -> [f32; 2] {
    let a = left_anchor(screen);
    [a[0], a[1] + left_pitch() * (1 + i) as f32]
}

/// The left row's hover rect — the label's own line (the C# measures the drawn
/// string; the port uses the row pitch's first 26 px, the text height at these
/// scales).
pub fn left_row_rect(screen: Vec2, i: usize) -> [f32; 4] {
    let a = left_row_anchor(screen, i);
    [a[0], a[1] - 2.0, 120.0, left_pitch() - 8.0]
}

/// `vector6` (`:359-360`): the right column's label anchor.
pub fn right_anchor(screen: Vec2, category: u8) -> [f32; 2] {
    let o = panel_origin(screen);
    let x = if long_label_culture() {
        o[0] + PANEL_W * 2.0 / 3.0
    } else {
        o[0] + RIGHT_ANCHOR_X
    };
    let _ = category;
    [x, o[1] + COLUMN_ANCHOR_Y]
}

/// `vector7 = (0, vector2.Y - num3 * 3) / (num11 + 1)` (`:361`), with the video
/// page's own `Y -= 2f` (`:362-365`).
pub fn right_pitch(category: u8) -> f32 {
    let p = (PANEL_H - PAD * 3.0) / (right_slots(category) as f32 + 1.0);
    if category == 2 {
        p - 2.0
    } else {
        p
    }
}

/// Row `i`'s label anchor in the right column — `anchor + offset * (1 + i)`
/// (`DrawRightSide`'s `anchor + offset * (1 + i)`, `:1488`).
///
/// A **bar** row's anchor is `BAR_SHIFT` (`num = 70`) further left: the C# moves
/// the shared `vector6.X` before a bar group and restores it before the next
/// non-bar row (`:406`/`:506`/`:513` on General, `:760`/`:798` on Interface,
/// `:911`/`:926` etc. on Video). Every row inside a shifted span turns out to be
/// the bar itself, so "bars shift, everything else does not" reproduces the
/// vanilla layout without replaying the anchor's state machine.
pub fn right_row_anchor(screen: Vec2, category: u8, i: usize) -> [f32; 2] {
    let a = right_anchor(screen, category);
    let shift = if matches!(page(category)[i].act, RowAct::Bar(_)) {
        BAR_SHIFT
    } else {
        0.0
    };
    [
        a[0] - shift,
        a[1] + right_pitch(category) * (1 + i) as f32,
    ]
}

/// `Color.Lerp(Color.Gray, Color.White, t)` (`:1465`, `:1485`).
fn gray_to_white(t: f32) -> [f32; 4] {
    let t = t.clamp(0.0, 1.0);
    let g = 128.0 / 255.0;
    [g + (1.0 - g) * t, g + (1.0 - g) * t, g + (1.0 - g) * t, 1.0]
}

/// `Color.Gold` — the selected category's tint (`:1469`).
const GOLD: [f32; 4] = [1.0, 215.0 / 255.0, 0.0, 1.0];

/// `Utils.DrawBorderStringBig`'s centred anchor (`origin 0.5, 0.5`, `:1470`).
fn centred_text(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    anchor: [f32; 2],
    scale: f32,
    colour: [f32; 4],
    label: &str,
) {
    let w = text::hud_text_width(r, lookups, label, scale);
    let h = lookups
        .ui_font
        .as_ref()
        .map_or(26.0, |f| f.line_spacing)
        * scale;
    text::hud_text(
        r,
        lookups,
        anchor[0] - w * 0.5,
        anchor[1] - h * 0.5,
        scale,
        colour,
        label,
    );
}

/// The right row's hover rect — centred on the **drawn** anchor, so the bar
/// rows' box follows them left (`DrawRightSide` hit-tests the string it draws).
pub fn right_row_rect(screen: Vec2, category: u8, i: usize) -> [f32; 4] {
    let a = right_row_anchor(screen, category, i);
    [
        a[0] - 120.0,
        a[1] - right_pitch(category) * 0.5,
        240.0,
        right_pitch(category),
    ]
}

/// `IngameOptions.valuePosition.X = vector3.X + vector2.X - num3 / 2 - 20`
/// (`:437`): the value bar's track right edge.
pub fn bar_right_x(screen: Vec2) -> f32 {
    panel_origin(screen)[0] + PANEL_W - PAD * 0.5 - 20.0
}

/// `Language.GetTextValue` through the port's table ([`crate::lang`]).
pub fn label_text(label: Label) -> String {
    match label {
        Label::Menu(i) => crate::lang::menu(i as usize),
        Label::Inter(i) => crate::lang::inter(i as usize),
        Label::Text(k) => crate::lang::text(k),
        Label::Blank => String::new(),
    }
}

/// The `GameUI.Enabled` / `GameUI.Disabled` words the `:995-1063` rows append.
fn enabled_word(on: bool) -> String {
    crate::lang::text(if on {
        "GameUI.Enabled"
    } else {
        "GameUI.Disabled"
    })
}

/// A bar's current `0..1` value.
pub fn bar_value(settings: &Settings, bar: Bar) -> f32 {
    match bar {
        Bar::Music => settings.music_volume,
        Bar::Sound => settings.sound_volume,
        Bar::Ambient => settings.ambient_volume,
        // `Main.GameZoomTarget - 1f` (`:530`): the track spans the 1.0..2.0 range.
        Bar::Zoom => (settings.zoom - 1.0).clamp(0.0, 1.0),
        // `Main.temporaryGUIScaleSlider` tracks 0.5..2.0
        // (`num14 * 1.5f + 0.5f`, `:577`) and the bar fraction is
        // `(slider - 0.5f) / 1.5f` clamped 0..1 (`:571`); the port's stored
        // `ui_scale` is that slider's value.
        Bar::UiScale => ((settings.ui_scale - 0.5) / 1.5).clamp(0.0, 1.0),
        // `Main.bgScroll` 0..100 (`:911-945`).
        Bar::Parallax => f32::from(settings.parallax_scroll) / 100.0,
    }
}

/// Write a bar's value back (`:424`, `:456`, `:489`, `:535`).
pub fn set_bar_value(settings: &mut Settings, bar: Bar, v: f32) {
    match bar {
        Bar::Music => settings.music_volume = v,
        Bar::Sound => settings.sound_volume = v,
        Bar::Ambient => settings.ambient_volume = v,
        Bar::Zoom => settings.zoom = 1.0 + v,
        Bar::Parallax => settings.parallax_scroll = (v * 100.0).round().clamp(0.0, 100.0) as u8,
        // `Main.temporaryGUIScaleSlider = num14 * 1.5f + 0.5f` then truncated
        // to two decimals (`(float)(int)(x * 100f) / 100f`, `:577-578`); the
        // provisional slider becomes `Main.UIScale` on the drop (`:585`).
        Bar::UiScale => settings.ui_scale = ((v * 1.5 + 0.5) * 100.0).trunc() / 100.0,
    }
}

/// A toggle's current state.
pub fn toggle_state(settings: &Settings, t: Toggle) -> bool {
    match t {
        Toggle::AutoSave => settings.auto_save,
        Toggle::AutoPause => settings.auto_pause,
        Toggle::PlayWhenUnfocused => settings.play_when_unfocused,
        Toggle::ReversedArmorBonuses => settings.reversed_armor_bonuses,
        Toggle::PickupText => settings.pickup_text,
        Toggle::EventProgress => settings.event_progress > 0,
        Toggle::PlacementPreview => settings.placement_preview,
        Toggle::HighlightNewItems => settings.highlight_new_items,
        Toggle::TileGrid => settings.tile_grid,
        Toggle::GamepadInstructions => settings.gamepad_instructions,
        Toggle::BossBarText => settings.boss_bar_text,
        Toggle::FlashyEffectsInterface => settings.flashy_effects_interface,
        Toggle::Background => settings.background,
        Toggle::Blood => settings.blood,
        Toggle::ForegroundSunlight => settings.foreground_sunlight,
        Toggle::HeatDistortion => settings.use_heat_distortion,
        Toggle::StormEffects => settings.use_storm_effects,
        Toggle::TilesSway => settings.tiles_sway,
        Toggle::ScreenShake => settings.use_screen_shake,
        Toggle::FlashyEffectsWorld => settings.flashy_effects_world,
        // `Main.SettingsEnabled_AutoReuseAllItems` (`:677-684`).
        Toggle::AutoReuseAll => settings.auto_reuse_all,
    }
}

/// Flip a toggle (`Main.autoSave = !Main.autoSave`, `:613` and the twins).
pub fn flip_toggle(settings: &mut Settings, t: Toggle) {
    let on = !toggle_state(settings, t);
    match t {
        Toggle::AutoSave => settings.auto_save = on,
        Toggle::AutoPause => settings.auto_pause = on,
        Toggle::PlayWhenUnfocused => settings.play_when_unfocused = on,
        Toggle::ReversedArmorBonuses => settings.reversed_armor_bonuses = on,
        Toggle::PickupText => settings.pickup_text = on,
        Toggle::EventProgress => settings.event_progress = u8::from(on),
        Toggle::PlacementPreview => settings.placement_preview = on,
        Toggle::HighlightNewItems => settings.highlight_new_items = on,
        Toggle::TileGrid => settings.tile_grid = on,
        Toggle::GamepadInstructions => settings.gamepad_instructions = on,
        Toggle::BossBarText => settings.boss_bar_text = on,
        Toggle::FlashyEffectsInterface => settings.flashy_effects_interface = on,
        Toggle::Background => settings.background = on,
        Toggle::Blood => settings.blood = on,
        Toggle::ForegroundSunlight => settings.foreground_sunlight = on,
        Toggle::HeatDistortion => settings.use_heat_distortion = on,
        Toggle::StormEffects => settings.use_storm_effects = on,
        Toggle::TilesSway => settings.tiles_sway = on,
        Toggle::ScreenShake => settings.use_screen_shake = on,
        Toggle::FlashyEffectsWorld => settings.flashy_effects_world = on,
        Toggle::AutoReuseAll => settings.auto_reuse_all = on,
    }
}

/// A toggle row's label for the current state — the `cond ? on : off` key pairs
/// of the C# (`:608`, `:617`, `:626`, `:635`, `:693-741`, `:977-994`).
pub fn toggle_label(settings: &Settings, t: Toggle) -> String {
    let on = toggle_state(settings, t);
    match t {
        Toggle::AutoSave => crate::lang::menu(if on { 67 } else { 68 }),
        Toggle::AutoPause => crate::lang::menu(if on { 69 } else { 70 }),
        Toggle::PlayWhenUnfocused => crate::lang::text(if on {
            "UI.PlayWhenUnfocusedOn"
        } else {
            "UI.PlayWhenUnfocusedOff"
        }),
        Toggle::ReversedArmorBonuses => crate::lang::menu(if on { 220 } else { 221 }),
        Toggle::PickupText => crate::lang::menu(if on { 71 } else { 72 }),
        Toggle::EventProgress => crate::lang::text_args(
            "GameUI.EventProgressBar",
            &[enabled_word(on)],
        ),
        Toggle::PlacementPreview => crate::lang::menu(if on { 128 } else { 129 }),
        Toggle::HighlightNewItems => {
            crate::lang::inter(if on { 117 } else { 116 })
        }
        Toggle::TileGrid => crate::lang::menu(if on { 229 } else { 230 }),
        // `Lang.menu[242]` "Gamepad Instructions On" / `[241]` "… Off".
        Toggle::GamepadInstructions => crate::lang::menu(if on { 242 } else { 241 }),
        Toggle::BossBarText => crate::lang::text_args(
            "GameUI.BigProgressBar",
            &[enabled_word(on)],
        ),
        Toggle::FlashyEffectsInterface => crate::lang::text_args(
            "UI.FlashyEffectsInterface",
            &[enabled_word(on)],
        ),
        Toggle::Background => crate::lang::menu(if on { 100 } else { 101 }),
        Toggle::Blood => crate::lang::menu(if on { 132 } else { 133 }),
        Toggle::ForegroundSunlight => crate::lang::text_args(
            "GameUI.ForegroundSunlightEffects",
            &[enabled_word(on)],
        ),
        Toggle::HeatDistortion => crate::lang::text_args(
            "GameUI.HeatDistortion",
            &[enabled_word(on)],
        ),
        Toggle::StormEffects => {
            crate::lang::text_args("GameUI.StormEffects", &[enabled_word(on)])
        }
        Toggle::TilesSway => {
            crate::lang::text_args("UI.TilesSwayInWind", &[enabled_word(on)])
        }
        Toggle::ScreenShake => {
            crate::lang::text_args("GameUI.ScreenShake", &[enabled_word(on)])
        }
        Toggle::FlashyEffectsWorld => crate::lang::text_args(
            "UI.FlashyEffectsWorld",
            &[enabled_word(on)],
        ),
        // `Main.SettingsEnabled_AutoReuseAllItems ? "UI.AutoReuseAllOn" :
        // "UI.AutoReuseAllOff"` (`:677`).
        Toggle::AutoReuseAll => crate::lang::text(if on {
            "UI.AutoReuseAllOn"
        } else {
            "UI.AutoReuseAllOff"
        }),
    }
}

/// A cycle row's current index and its label.
pub fn cycle_label(settings: &Settings, c: Cycle) -> String {
    match c {
        // `Lang.menu[(int)(247 + Main.FrameSkipMode)]` (`:946`).
        Cycle::FrameSkip => crate::lang::menu(247 + settings.frame_skip as usize),
        // `"UI.LightMode_" + Lighting.Mode` (`:955`).
        Cycle::LightMode => crate::lang::text(&format!("UI.LightMode_{}", settings.light_mode)),
        // `Lang.menu[59 + Main.qaStyle]` (`:964`).
        Cycle::Quality => crate::lang::menu(59 + settings.quality as usize),
        // `GameUI.WaveQuality` (`:1022-1036`): the vanilla switches on the level;
        // the port names the level through the same Enabled/Disabled pair the
        // neighbouring rows use, plus the level's own ordinal.
        Cycle::WaveQuality => format!(
            "{} {}",
            crate::lang::text("GameUI.WaveQuality"),
            settings.wave_quality
        ),
        // `Lang.menu[253]` Disabled / `[224]` Left Control / `[223]` Left Shift
        // (`:841-862`): the C# `PreferenceSettings` switch.
        Cycle::QuickTrash => {
            let idx = if !settings.quick_trash {
                253
            } else if settings.disable_left_shift_trash {
                224
            } else {
                223
            };
            crate::lang::menu(idx)
        }
        // `Lang.menu[215]` To Cursor / `[216]` Disabled (`:1381-1389`).
        Cycle::SmartBlocks => crate::lang::menu(if settings.smart_blocks { 215 } else { 216 }),
        // `Lang.menu[121]` Toggle / `[122]` Hold (`:1390-1398`).
        Cycle::SmartCursorMode => {
            crate::lang::menu(if settings.smart_cursor_toggle { 121 } else { 122 })
        }
        // `Lang.menu[214]` Axe to Pickaxe / `[213]` Pickaxe to Axe (`:1399-1440`).
        Cycle::SmartCursorPriority => {
            crate::lang::menu(if settings.smart_axe_first { 214 } else { 213 })
        }
        // `:644-657`'s `PreferenceSettings switch` → `UI.SmartDoorsDisabled`
        // for `_`, `UI.SmartDoorsGamepad` for `EnabledForGamepadOnly` and
        // `UI.SmartDoorsEnabled` for `EnabledForEverything`.
        Cycle::SmartDoors => crate::lang::text(match settings.smart_doors {
            2 => "UI.SmartDoorsEnabled",
            1 => "UI.SmartDoorsGamepad",
            _ => "UI.SmartDoorsDisabled",
        }),
        // `:659-675`: `HoverControl != Hold` picks the Click wording.
        Cycle::HoverControl => crate::lang::text(if settings.hover_control == 0 {
            "UI.HoverControlSettingIsHold"
        } else {
            "UI.HoverControlSettingIsClick"
        }),
    }
}

/// Advance a cycle row to its next state (the C#'s click arms).
pub fn advance_cycle(settings: &mut Settings, c: Cycle) {
    match c {
        Cycle::FrameSkip => settings.frame_skip = (settings.frame_skip + 1) % 3,
        Cycle::LightMode => settings.light_mode = (settings.light_mode + 1) % 4,
        Cycle::Quality => settings.quality = (settings.quality + 1) % 4,
        Cycle::WaveQuality => settings.wave_quality = (settings.wave_quality + 1) % 3,
        Cycle::QuickTrash => {
            // Disabled -> Left Control -> Left Shift -> Disabled (`:841-862`).
            if !settings.quick_trash {
                settings.quick_trash = true;
                settings.disable_left_shift_trash = true;
            } else if settings.disable_left_shift_trash {
                settings.disable_left_shift_trash = false;
            } else {
                settings.quick_trash = false;
                settings.disable_left_shift_trash = false;
            }
        }
        Cycle::SmartBlocks => settings.smart_blocks = !settings.smart_blocks,
        Cycle::SmartCursorMode => {
            // `Main.SmartCursorMode`'s two states (`:1390-1398`).
            settings.smart_cursor_toggle = !settings.smart_cursor_toggle;
        }
        Cycle::SmartCursorPriority => settings.smart_axe_first = !settings.smart_axe_first,
        // `DoorOpeningHelper.CyclePreferences` (`DoorOpeningHelper.cs:262-276`):
        // Disabled -> EnabledForEverything -> EnabledForGamepadOnly -> Disabled.
        Cycle::SmartDoors => {
            settings.smart_doors = match settings.smart_doors {
                0 => 2,
                2 => 1,
                _ => 0,
            }
        }
        // `Player.Settings.CycleHoverControl` (`Player.cs:379-383`) — the row
        // only distinguishes Hold from everything else (`IngameOptions.cs:660`).
        Cycle::HoverControl => settings.hover_control = if settings.hover_control == 0 { 1 } else { 0 },
    }
}

/// A right row's drawn label, with the bar percentage the C# concatenates
/// (`Lang.menu[99].Value + " " + Math.Round(Main.musicVolume * 100f) + "%"`,
/// `:407`).
pub fn row_label(row: &Row, settings: &Settings) -> String {
    match row.act {
        RowAct::Toggle(t) => toggle_label(settings, t),
        RowAct::Cycle(c) => cycle_label(settings, c),
        RowAct::Bar(bar) => {
            let pct = (bar_value(settings, bar) * 100.0).round();
            match bar {
                // `GameUI.GameZoom` takes the **target zoom in percent**, not the
                // bar's fraction (`:512`:
                // `Language.GetTextValue("GameUI.GameZoom",
                // Math.Round(Main.GameZoomTarget * 100f),
                // Math.Round(Main.GameViewMatrix.Zoom.X * 100f))`) while the bar
                // itself is fed `GameZoomTarget - 1f` (`:529`). The port has no
                // separate live matrix zoom, so both numbers are the target.
                Bar::Zoom => {
                    let zoom_pct = (settings.zoom * 100.0).round();
                    crate::lang::text_args(
                        "GameUI.GameZoom",
                        &[format!("{zoom_pct}"), format!("{zoom_pct}")],
                    )
                }
                // `GameUI.UIScale` takes the provisional and the applied scale
                // (`Math.Round(temporaryGUIScaleSlider * 100f)`,
                // `Math.Round(Main.UIScale * 100f)`, `:556`). The port's
                // immediate-mode bar has no separate provisional field, so both
                // numbers are the stored (wanted) scale — the same simplification
                // the zoom row makes above.
                Bar::UiScale => {
                    let pct = (settings.ui_scale * 100.0).round();
                    crate::lang::text_args(
                        "GameUI.UIScale",
                        &[format!("{pct}"), format!("{pct}")],
                    )
                }
                _ => format!("{} {pct}%", label_text(row.label)),
            }
        }
        RowAct::Header | RowAct::Parked => label_text(row.label),
    }
}

/// The per-frame hover/animation pass — the state half of `IngameOptions.Draw`
/// (`:157-199` left, `:333-390` right), split out so `frame()` can run it
/// before the HUD borrows the state. Returns the sound cue the frame should
/// play (`SoundEngine.PlaySound(12)`, `:187`/`:191`).
pub fn tick_hover(
    state: &mut IngameOptions,
    screen: Vec2,
    mouse: Vec2,
    mouse_down: bool,
) -> Option<Cue> {
    if !state.open {
        return None;
    }
    // `if (!Main.mouseLeft) rightLock = -1;` (`:385-388`).
    if !mouse_down {
        state.right_lock = -1;
    }
    let mut cue = None;
    // The left column (`:187-199`).
    let (lmin, lmax, lspeed) = left_scale_band();
    state.left_hover = (0..LEFT_SLOTS)
        .find(|i| contains(mouse, left_row_rect(screen, *i)))
        .map_or(-1, |i| i as i32);
    for i in 0..LEFT_SLOTS {
        let target = state.left_hover == i as i32;
        let s = &mut state.left_scale[i];
        *s += if target { lspeed } else { -lspeed };
        *s = s.clamp(lmin, lmax);
    }
    if state.old_left_hover != state.left_hover && state.left_hover != -1 {
        cue = Some(Cue::Tick);
    }
    state.old_left_hover = state.left_hover;

    // The right column (`:371-390`): a row is hovered over its label or, for a
    // bar row, over the bar's own hit rect.
    let category = state.category;
    let page = page(category);
    let (rmin, rmax, rspeed) = right_scale_band(category);
    let mut hover = -1i32;
    for (i, row) in page.iter().enumerate() {
        let hit = contains(mouse, right_row_rect(screen, category, i))
            || matches!(row.act, RowAct::Bar(_))
                && contains(mouse, sliders::value_bar_rects(bar_right_x(screen), right_row_anchor(screen, category, i)[1]).1);
        if hit {
            hover = i as i32;
            break;
        }
    }
    state.right_hover = hover;
    for i in 0..17 {
        let locked = state.right_lock == i as i32;
        let target = locked || (state.right_hover == i as i32 && state.right_lock == -1);
        let s = &mut state.right_scale[i];
        *s += if target { rspeed } else { -rspeed };
        *s = s.clamp(rmin, rmax);
    }
    if state.old_right_hover != state.right_hover && state.right_hover != -1 {
        cue = Some(Cue::Tick);
    }
    state.old_right_hover = state.right_hover;
    if state.right_lock != -1 && state.right_hover == -1 && cue.is_none() {
        // The C# re-points the lock onto the hovered row while dragging; the
        // port keeps the lock until the release (the caller clears it).
    }
    cue
}

/// The per-frame tick the frame loop runs while the window is open: the hover
/// animation ([`tick_hover`]) plus the sticky bar drag (the C#'s
/// `Main.musicVolume = musicVolume` while `mouseLeft && rightLock == slot`,
/// `:424`, `:456`, `:489`). Returns the cue's name for the caller to play.
pub fn update(app: &mut crate::app::App, mouse_down: bool) -> Option<&'static str> {
    if !app.ingame_options.open {
        return None;
    }
    let (sw, sh) = app.screen_size();
    let screen = Vec2::new(sw, sh);
    let mouse = app.mouse_pos;
    let cue = tick_hover(&mut app.ingame_options, screen, mouse, mouse_down);
    let category = app.ingame_options.category;
    if mouse_down {
        let locked = app.ingame_options.right_lock;
        if locked >= 0 && (locked as usize) < page(category).len() {
            let i = locked as usize;
            if let RowAct::Bar(bar) = page(category)[i].act {
                let (rect, _hit) =
                    sliders::value_bar_rects(bar_right_x(screen), right_row_anchor(screen, category, i)[1]);
                let v = sliders::value_bar_value(rect, mouse.x);
                set_bar_value(&mut app.settings, bar, v);
            }
        }
    }
    cue.map(|c| match c {
        Cue::Tick => "Menu_Tick",
        Cue::Open => "Menu_Open",
    })
}

/// The click pass (`:132`'s `flag4`): a left row selects a category or arms an
/// action, a right bar grabs and sets its value, a toggle flips and a cycle
/// advances. Returns `true` when the click was consumed.
pub fn click(app: &mut crate::app::App, screen: Vec2, mouse: Vec2) -> bool {
    if !app.ingame_options.open {
        return false;
    }
    // The left column.
    let row_hit = (0..LEFT_SLOTS).find(|i| contains(mouse, left_row_rect(screen, *i)));
    if let Some(i) = row_hit {
        let action = LEFT_ROWS[i].action;
        match action {
            LeftAction::Category(c) => {
                if app.ingame_options.category != c {
                    // `:307-311`: a category change resets every right scale.
                    app.ingame_options.right_scale = [0.0; 17];
                }
                app.ingame_options.category = c;
            }
            other => {
                app.ingame_options.pending = Some(other);
                match other {
                    LeftAction::Close => app.ingame_options.close(),
                    LeftAction::Achievements => {
                        app.ingame_options.close();
                        app.achievements_open = true;
                    }
                    // `:252-256` / `:300-310`: the keybinding screen is M7-UI-5
                    // and the save-and-quit path is the front-end flow's; both
                    // stay on `pending` for the caller.
                    LeftAction::Keybinds | LeftAction::SaveAndExit => {
                        app.ingame_options.close();
                    }
                    LeftAction::Category(_) => unreachable!(),
                }
            }
        }
        return true;
    }
    // The right column.
    let category = app.ingame_options.category;
    let page = page(category);
    for (i, row) in page.iter().enumerate() {
        let y = right_row_anchor(screen, category, i)[1];
        match row.act {
            RowAct::Bar(bar) => {
                let (rect, hit) = sliders::value_bar_rects(bar_right_x(screen), y);
                if contains(mouse, hit) {
                    let v = sliders::value_bar_value(rect, mouse.x);
                    app.ingame_options.right_lock = i as i32;
                    set_bar_value(&mut app.settings, bar, v);
                    return true;
                }
            }
            RowAct::Toggle(t) => {
                if contains(mouse, right_row_rect(screen, category, i)) {
                    flip_toggle(&mut app.settings, t);
                    return true;
                }
            }
            RowAct::Cycle(c) => {
                if contains(mouse, right_row_rect(screen, category, i)) {
                    advance_cycle(&mut app.settings, c);
                    return true;
                }
            }
            RowAct::Header | RowAct::Parked => {}
        }
    }
    // Anywhere else on the panel is swallowed (the vanilla's `mouseInterface`
    // set over `_GUIHover`, `:130-133`); off the panel the window closes.
    if contains(mouse, panel_rect(screen)) {
        return true;
    }
    app.ingame_options.close();
    true
}

// ---------------------------------------------------------------------------
// The Settings button — `Main.DrawSettingButton` (`Main.cs:45532-45580`)
// ---------------------------------------------------------------------------
//
// The button is **not** a framed box: it is a bare word in the death font with
// a 4-way black outline (`:45562-45584`), its scale animating `0.8 -> 0.96`
// while hovered (`:45547-45555`), positioned by `RefreshSettingsButtonStatus`
// (`:45492-45527`) — bottom-right, pushed left/up when it would collide with the
// accessory column. The port used to draw a 30x30 `Inventory_Back` box at a
// hard-coded `(850, 580)`, which is what made its placement wrong at every
// resolution but 1600x900.

/// `Main.exitScale`'s resting value (`:45547-45555`).
pub const BUTTON_SCALE_MIN: f32 = 0.8;
/// ... and its hovered value.
pub const BUTTON_SCALE_MAX: f32 = 0.96;
/// The per-frame step between them (`:45551`, `:45555`).
pub const BUTTON_SCALE_STEP: f32 = 0.02;

/// `Main.DrawSettingButton`'s `ref float scale` step (`:45547-45555`): up while
/// hovered, down while not, clamped to `[0.8, 0.96]`.
pub fn advance_button_scale(scale: &mut f32, over: bool) {
    if over {
        if *scale < BUTTON_SCALE_MAX {
            *scale += BUTTON_SCALE_STEP;
        }
    } else if *scale > BUTTON_SCALE_MIN {
        *scale -= BUTTON_SCALE_STEP;
    }
    *scale = scale.clamp(BUTTON_SCALE_MIN, BUTTON_SCALE_MAX);
}

/// `Main.RefreshSettingsButtonStatus` (`Main.cs:45492-45527`): the label's centre
/// and whether the button had to be pushed beside the accessory column.
///
/// `extra_accessory` is `Player.GetAmountOfExtraAccessorySlotsToShow()` (0 in the
/// port — `EXTRA_ACCESSORY_SLOTS`) and `push_equipment_up` is
/// `PlayerInput.SettingsForUI.PushEquipmentAreaUp` (not modelled), both kept as
/// parameters so the branches stay faithful.
pub fn settings_button_pos(
    screen: Vec2,
    m_h: f32,
    dead: bool,
    options_open: bool,
    extra_accessory: i32,
    push_equipment_up: bool,
) -> ([f32; 2], bool) {
    let mut pos_x = screen.x - 110.0; // `:45496`
    let mut pos_y = screen.y - 20.0; // `:45497`
    if dead {
        pos_x += 40.0; // `:45499-45503`
        return ([pos_x, pos_y], false);
    }
    if push_equipment_up {
        pos_y -= 30.0; // `:45506`
    }
    if options_open {
        // `:45507`'s `if (!Main.ingameOptionsWindow)`: the column-avoidance
        // branch runs only while the window is closed.
        return ([pos_x, pos_y], false);
    }
    // `:45509-45514`: the accessory column's bottom row, in ints like the C#.
    let mut num = 174 + m_h as i32;
    if screen.y < 950.0 && extra_accessory == 2 {
        num -= (47.600_002f32 * (extra_accessory - 1) as f32) as i32;
    }
    let num2 = num + (47.600_002f32 * (8 + extra_accessory) as f32) as i32;
    if pos_y - 28.0 <= num2 as f32 {
        // `:45516-45521`: the button would sit on the last accessory row, so it
        // moves left of the column and up onto it.
        pos_x -= 140.0;
        pos_y = (num2 - 47 + 30) as f32;
        return ([pos_x, pos_y], true);
    }
    ([pos_x, pos_y], false)
}

/// Everything the button's draw and hit test need. The three sizes are the
/// *unscaled* `MeasureString` results the C# keeps in `vector3` (the death-font
/// label) and `vector` (the mouse-font "Settings" matcher, `:45535-45537`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonLayout {
    /// The label's centre (`posX`, `posY`).
    pub pos: [f32; 2],
    /// `DeathText.MeasureString(Lang.inter[62])` — width.
    pub w: f32,
    /// ... and height.
    pub h: f32,
    /// `vector.X / vector2.X` (`:45538`): the death-font scale that makes the
    /// label as wide as the mouse-font "Settings" matcher.
    pub word_scale: f32,
    /// `Main._settingsButtonIsPushedToSide` (`:45520`).
    pub pushed_to_side: bool,
}

impl ButtonLayout {
    /// `posX > x - w/2 < posX + w/2`, `posY - h/2 < y < posY + h/2 - 10`
    /// (`:45589`) — note the `- 10` on the bottom edge and the strict compares.
    pub fn hit(&self, mouse: Vec2) -> bool {
        mouse.x > self.pos[0] - self.w * 0.5
            && mouse.x < self.pos[0] + self.w * 0.5
            && mouse.y > self.pos[1] - self.h * 0.5
            && mouse.y < self.pos[1] + self.h * 0.5 - 10.0
    }
}

/// [`settings_button_pos`] plus the measurements of [`crate::lang::inter`]`(62)`.
pub fn button_layout(
    screen: Vec2,
    m_h: f32,
    dead: bool,
    options_open: bool,
    extra_accessory: i32,
    push_equipment_up: bool,
    measure: (f32, f32, f32),
) -> ButtonLayout {
    let (pos, pushed_to_side) = settings_button_pos(
        screen,
        m_h,
        dead,
        options_open,
        extra_accessory,
        push_equipment_up,
    );
    let (w, h, word_w) = measure;
    let word_scale = if w > 0.0 && word_w > 0.0 {
        word_w / w
    } else {
        1.0
    };
    ButtonLayout {
        pos,
        w,
        h,
        word_scale,
        pushed_to_side,
    }
}

/// `vector3`/`vector`/`vector2` (`:45535-45537`) from the loaded fonts. The
/// bitmap fallbacks keep the button usable in the PNG-only runs; the estimate is
/// `Mouse_Text`'s 7 px advance at scale 1 (`text::font_width`'s average).
pub fn measure_button(lookups: &Lookups) -> (f32, f32, f32) {
    let label = crate::lang::inter(62);
    let w = text::death_font_width(lookups, &label).unwrap_or(7.0 * label.chars().count() as f32);
    let h = lookups.death_font.as_ref().map_or(17.0, |f| f.line_spacing);
    let word = text::ui_font_width(lookups, "Settings", 1.0).unwrap_or(7.0 * 8.0);
    (w, h, word)
}

/// The per-frame half of `Main.DrawInterface_29_SettingsButton`
/// (`Main.cs:45512-45527`) and of `DrawSettingButton`'s `ref` state
/// (`:45547-45555`, `:45589-45613`): recompute the position, step `exitScale`
/// from the *previous* frame's hover and latch this frame's for the next, and
/// remember the geometry so the draw and the click agree.
///
/// The C# does all of this inside the draw; the port splits it out because the
/// click pass runs before the HUD is built (`App::frame`), and because the
/// inventory's accessory column count reads `_settingsButtonIsPushedToSide`.
pub fn refresh_button(
    state: &mut IngameOptions,
    lookups: &Lookups,
    screen: Vec2,
    m_h: f32,
    dead: bool,
    mouse: Vec2,
    extra_accessory: i32,
    push_equipment_up: bool,
) -> ButtonLayout {
    let layout = button_layout(
        screen,
        m_h,
        dead,
        state.open,
        extra_accessory,
        push_equipment_up,
        measure_button(lookups),
    );
    state.button_layout = layout;
    state.button_pushed_to_side = layout.pushed_to_side;
    // `:45547-45555`: the scale steps from last frame's `mouseExit`, so the
    // value the draw reads is the one this frame's hover just produced.
    advance_button_scale(&mut state.button_scale, state.button_mouse_over);
    state.button_mouse_over = layout.hit(mouse);
    layout
}

/// `Main.DrawSettingButton`'s text (`:45562-45584`): the death-font label drawn
/// five times — four black passes at `(+-2, +-2)` and the white pass on top —
/// at `(scale - 0.2) * num`, centred on `pos` (the XNA origin is
/// `MeasureString(label) / 2` in *texture* units, so the top-left moves by half
/// the **scaled** size).
pub fn draw_settings_button(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    layout: &ButtonLayout,
    scale: f32,
) {
    let label = crate::lang::inter(62);
    let s = (scale - 0.2) * layout.word_scale;
    let x = layout.pos[0] - layout.w * 0.5 * s;
    let y = layout.pos[1] - layout.h * 0.5 * s;
    text::draw_border_string(
        r,
        lookups,
        text::BorderFont::Death,
        x,
        y,
        s,
        &[
            [-2.0, 0.0],
            [2.0, 0.0],
            [0.0, -2.0],
            [0.0, 2.0],
        ],
        [1.0; 4],
        &label,
    );
}

/// `IngameOptions.Draw` (`:103-1441`): the panel, the title, the two column
/// backdrops and the rows of the selected page.
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
    // `Utils.DrawInvBG(sb, x - 20, y - 20, w + 40, h + 40,
    // new Color(33, 15, 91, 255) * 0.685f)` (`:130`) — a nine-slice of
    // `Inventory_Back13` (`Utils.cs:2548-2572`), not a stretched back sheet.
    crate::ui::tooltip::draw_inv_bg(
        r,
        lookups,
        [
            origin[0] - PAD,
            origin[1] - PAD,
            PANEL_W + PAD * 2.0,
            PANEL_H + PAD * 2.0,
        ],
        [33.0 / 255.0, 15.0 / 255.0, 91.0 / 255.0, 0.685],
    );
    // `Utils.DrawBorderString(sb, Language.GetTextValue("GameUI.SettingsMenu"),
    // vector3 + vector2 * new Vector2(0.5f, 0f), Color.White, 1f, 0.5f)`
    // (`:134`): anchored `0.5` in X, so the title is **centred** on the panel's
    // top edge — the old code left-aligned it at the panel centre.
    let title = crate::lang::text("GameUI.SettingsMenu");
    let title_w = text::hud_text_width(r, lookups, &title, 1.0);
    text::border_string(
        r,
        lookups,
        origin[0] + PANEL_W * 0.5 - title_w * 0.5,
        origin[1],
        [1.0; 4],
        &title,
    );
    // The two column backdrops (`:136-146`): the long cultures split them 1/3+2/3.
    let (left_w, right_x) = if long_label_culture() {
        (PANEL_W / 3.0 - PAD, origin[0] + PANEL_W / 3.0 + PAD)
    } else {
        (PANEL_W / 2.0 - PAD, origin[0] + PANEL_W / 2.0 + PAD)
    };
    // `:136-146`: both column backdrops are plain `Utils.DrawInvBG(..)` calls,
    // i.e. the default `(63, 65, 151, 255) * 0.785` tint of the same nine-slice.
    let top = origin[1] + PAD * 5.0 / 2.0;
    let h = PANEL_H - PAD * 3.0;
    let column_tint = [63.0 / 255.0, 65.0 / 255.0, 151.0 / 255.0, 0.785];
    crate::ui::tooltip::draw_inv_bg(r, lookups, [origin[0] + PAD * 0.5, top, left_w, h], column_tint);
    crate::ui::tooltip::draw_inv_bg(
        r,
        lookups,
        [
            right_x,
            top,
            origin[0] + PANEL_W - right_x - PAD * 0.5,
            h,
        ],
        column_tint,
    );
    // The left column (`:214-330`): `DrawLeftSide` centres each label on its
    // anchor, lerps Gray->White by the scale fraction and paints the selected
    // category Gold (`:1465-1470`).
    let (lmin, lmax, _) = left_scale_band();
    for (i, row) in LEFT_ROWS.iter().enumerate() {
        let a = left_row_anchor(screen, i);
        let scale = options.left_scale[i];
        let label = match (row.menu, row.inter) {
            (Some(m), _) => crate::lang::menu(m as usize),
            (None, Some(k)) => crate::lang::inter(k as usize),
            _ => String::new(),
        };
        let selected = matches!(row.action, LeftAction::Category(c) if c == options.category);
        let colour = if selected {
            GOLD
        } else {
            gray_to_white((scale - lmin) / (lmax - lmin))
        };
        centred_text(r, lookups, a, scale, colour, &label);
    }
    // The right column (`:400-1441`): `DrawRightSide` uses the same centring and
    // the `colorScale` lerp (`:1483-1504`).
    let category = options.category;
    let (rmin, rmax, _) = right_scale_band(category);
    for (i, row) in page(category).iter().enumerate() {
        let a = right_row_anchor(screen, category, i);
        let idx = i.min(options.right_scale.len() - 1);
        let scale = options.right_scale[idx];
        let label = row_label(row, settings);
        let dim = matches!(row.act, RowAct::Parked);
        let colour = if dim {
            [0.45, 0.45, 0.45, 1.0]
        } else {
            gray_to_white((scale - rmin) / (rmax - rmin))
        };
        // The bar rows carry their track on the panel's right edge
        // (`valuePosition.X = ...`, `:437`), the label centred as usual.
        centred_text(r, lookups, a, scale, colour, &label);
        if let RowAct::Bar(bar) = row.act {
            sliders::draw_value_bar(
                r,
                lookups,
                bar_right_x(screen),
                a[1],
                bar_value(settings, bar),
                mouse,
            );
        }
    }
}

/// `IngameOptions.Close(true)`'s quiet twin: the settings persist on close
/// (the vanilla's `Main.SaveSettings` → `WriteSettings`, `Main.cs:53669+`; the
/// port's `Settings::save` is called by the caller).
pub fn persist() {}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Main.cs:45496-45497`: `posX = uIScreenWidth - 110`, `posY = uIScreenHeight - 20`,
    /// with no collision at 1600x900 (`posY - 28 = 852` clears `num2 = 554`).
    #[test]
    fn the_button_sits_at_the_vanillas_bottom_right() {
        let (pos, pushed) = settings_button_pos(Vec2::new(1600.0, 900.0), 0.0, false, false, 0, false);
        assert_eq!(pos, [1490.0, 880.0]);
        assert!(!pushed);
        // A taller screen moves it with the edge, not to a fixed spot.
        let (pos, _) = settings_button_pos(Vec2::new(1920.0, 1080.0), 0.0, false, false, 0, false);
        assert_eq!(pos, [1810.0, 1060.0]);
    }

    /// `Main.cs:45516-45521`: when `posY - 28 <= num2` the label is pushed left
    /// of the accessory column and up onto its last row.
    ///
    /// At 1600x1024 with the minimap up (`m_h = 424`) `num = 598` and
    /// `num2 = 978`, so `1060 - 28 = 1032 > 978` — still no push. At 1600x800
    /// with `m_h = 200`: `num = 374`, `num2 = 754`, `780 - 28 = 752 <= 754`,
    /// which pushes.
    #[test]
    fn the_button_is_pushed_beside_the_accessory_column_when_it_would_collide() {
        let (pos, pushed) = settings_button_pos(Vec2::new(1600.0, 800.0), 200.0, false, false, 0, false);
        assert!(pushed, "`_settingsButtonIsPushedToSide` (`:45520`)");
        assert_eq!(pos, [1490.0 - 140.0, 754.0 - 47.0 + 30.0]);
        // With the options window open the branch is skipped (`:45507`).
        let (pos, pushed) = settings_button_pos(Vec2::new(1600.0, 800.0), 200.0, false, true, 0, false);
        assert!(!pushed);
        assert_eq!(pos, [1490.0, 780.0]);
    }

    /// `Main.cs:45499-45503`: a dead player's label moves right and skips the
    /// column arithmetic entirely.
    #[test]
    fn a_dead_players_button_moves_right() {
        let (pos, pushed) = settings_button_pos(Vec2::new(1600.0, 900.0), 300.0, true, false, 0, false);
        assert_eq!(pos, [1530.0, 880.0]);
        assert!(!pushed);
    }

    /// `PushEquipmentAreaUp` shifts the label up by 30 (`:45506`).
    #[test]
    fn pushing_the_equipment_area_up_lifts_the_button() {
        let (pos, _) = settings_button_pos(Vec2::new(1600.0, 900.0), 0.0, false, false, 0, true);
        assert_eq!(pos, [1490.0, 850.0]);
    }

    /// `Main.cs:45547-45555`: `0.02` per frame toward `0.96`/`0.8`.
    #[test]
    fn the_hover_scale_ramps_and_clamps() {
        let mut s = BUTTON_SCALE_MIN;
        advance_button_scale(&mut s, true);
        assert_eq!(s, 0.82);
        for _ in 0..20 {
            advance_button_scale(&mut s, true);
        }
        assert_eq!(s, BUTTON_SCALE_MAX, "clamped at 0.96");
        for _ in 0..20 {
            advance_button_scale(&mut s, false);
        }
        assert_eq!(s, BUTTON_SCALE_MIN, "clamped at 0.8");
    }

    /// `Main.cs:45589`: strict `>`/`<` box, `- 10` on the bottom edge, and the
    /// measure is the *unscaled* death-font size.
    #[test]
    fn the_button_hit_box_is_strict_and_shaved_at_the_bottom() {
        let l = ButtonLayout {
            pos: [1490.0, 880.0],
            w: 60.0,
            h: 17.0,
            word_scale: 1.0,
            pushed_to_side: false,
        };
        assert!(l.hit(Vec2::new(1490.0, 875.0)), "inside");
        assert!(!l.hit(Vec2::new(1490.0, 880.0 - 8.5)), "the top edge is exclusive");
        assert!(!l.hit(Vec2::new(1460.0, 875.0)), "the left edge is exclusive");
        // The bottom edge is `posY + h/2 - 10 = 878.5`, so the lower third of
        // the drawn word is not clickable — exactly the C# rule.
        assert!(l.hit(Vec2::new(1490.0, 878.0)));
        assert!(!l.hit(Vec2::new(1490.0, 878.5)));
        assert!(!l.hit(Vec2::new(1490.0, 884.0)), "the drawn descender is dead");
    }

    /// `vector.X / vector2.X` (`:45538`) normalises the death-font label to the
    /// mouse-font "Settings" width.
    #[test]
    fn the_button_normalises_the_label_to_the_settings_matcher() {
        let l = button_layout(
            Vec2::new(1600.0, 900.0),
            0.0,
            false,
            false,
            0,
            false,
            (120.0, 34.0, 60.0),
        );
        assert_eq!(l.word_scale, 0.5);
        // A zero measure must not divide by zero.
        let l = button_layout(Vec2::new(1600.0, 900.0), 0.0, false, false, 0, false, (0.0, 0.0, 0.0));
        assert_eq!(l.word_scale, 1.0);
    }

    /// [`refresh_button`] latches the hover for the *next* frame and records the
    /// geometry the draw and the click share.
    #[test]
    fn refresh_button_latches_the_hover_and_the_layout() {
        let lookups = Lookups::default();
        let mut st = IngameOptions::new();
        let screen = Vec2::new(1600.0, 900.0);
        // Frame 1: the cursor is on the label; the scale steps from the *old*
        // hover (false), so it stays at 0.8 and the latch turns on.
        let l = refresh_button(&mut st, &lookups, screen, 0.0, false, Vec2::new(1490.0, 875.0), 0, false);
        assert_eq!(st.button_scale, BUTTON_SCALE_MIN, "`mouseExit` was false on entry");
        assert!(st.button_mouse_over);
        assert_eq!(st.button_layout, l, "the click reads the draw's geometry");
        // Frame 2 now steps up.
        refresh_button(&mut st, &lookups, screen, 0.0, false, Vec2::new(1490.0, 875.0), 0, false);
        assert_eq!(st.button_scale, 0.82);
        // Off the label the scale steps *up* one more frame (the latch still
        // read `true` on entry — the C# has the same one-frame lag) and then
        // ramps back down.
        refresh_button(&mut st, &lookups, screen, 0.0, false, Vec2::new(400.0, 400.0), 0, false);
        assert_eq!(st.button_scale, 0.84, "one frame of lag, like `Main.exitScale`");
        for _ in 0..10 {
            refresh_button(&mut st, &lookups, screen, 0.0, false, Vec2::new(400.0, 400.0), 0, false);
        }
        assert_eq!(st.button_scale, BUTTON_SCALE_MIN);
        assert!(!st.button_mouse_over);
        assert!(!st.button_pushed_to_side);
    }

    /// `vector3 = vector / 2f - vector2 / 2f` (`:125-126`) and the 20 px `num3`
    /// padding (`:127-129`).
    #[test]
    fn the_panel_sits_centred_with_the_padding() {
        let screen = Vec2::new(1920.0, 1080.0);
        assert_eq!(panel_origin(screen), [625.0, 300.0]);
        assert_eq!(panel_rect(screen), [605.0, 280.0, 710.0, 520.0]);
    }

    /// The two columns' anchors are the C#'s quarters of the panel (`:207-208`
    /// and `:359-360`), both 50 px below the panel's top (`num3 * 5 / 2`).
    #[test]
    fn the_columns_are_quarter_split_and_start_fifty_below_the_top() {
        // The active culture is global state the sibling tests flip mid-run;
        // hold the guard the lang module's tests use and pin English, so the
        // culture branches inside these helpers stay deterministic.
        let _guard = crate::lang::test_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        crate::lang::set_language(GameCulture::English);
        let screen = Vec2::new(1920.0, 1080.0);
        let l = left_anchor(screen);
        assert_eq!(l[1] - 300.0, 50.0, "the left column's y is num3 * 5 / 2");
        let r = right_anchor(screen, 0);
        assert_eq!(r[1], l[1], "both columns share the anchor y");
        // The right anchor is the 3/4 mark; a bar row hangs 70 px left of it
        // (`vector6.X -= num`, `:417`).
        assert_eq!(right_anchor(screen, 0)[0], 625.0 + PANEL_W * 0.75);
    }

    /// The left pitch is `(480 - 100) / (7 + 1 + 1)` — `vector5` at `:209` with
    /// `num7 + 1` slots for the port's eight rows.
    #[test]
    fn the_left_pitch_is_the_c_sharp_slot_rule() {
        assert_eq!(LEFT_SLOTS, 8);
        assert!((left_pitch() - (PANEL_H - PAD * 5.0) / 9.0).abs() < 1e-6);
    }

    /// `num11` per category (`:296-330`) and the video page's own -2 px tweak
    /// (`:362-365`).
    #[test]
    fn the_right_slots_are_the_c_sharp_num11_and_video_is_two_px_tighter() {
        assert_eq!(right_slots(0), 17);
        assert_eq!(right_slots(1), 14);
        assert_eq!(right_slots(2), 14);
        assert_eq!(right_slots(3), 15);
        let general_pitch = (PANEL_H - PAD * 3.0) / (right_slots(0) as f32 + 1.0);
        let vid = (PANEL_H - PAD * 3.0) / (right_slots(2) as f32 + 1.0);
        assert!((right_pitch(0) - general_pitch).abs() < 1e-6);
        assert!((right_pitch(2) - (vid - 2.0)).abs() < 1e-6, "the video -2f");
    }

    /// Every page's row count is the C#'s — the tables are the port of the
    /// `:400-1441` sequential `DrawRightSide` calls.
    #[test]
    fn every_page_draws_the_vanillas_row_count() {
        // `General` ends with the `""` separator at index 17 (`:685-689`), so it
        // draws 18 rows over `num11 = 17` — the same "last index" shape as the
        // video page.
        assert_eq!(page(0).len(), 18, "General (`:400-689`)");
        assert_eq!(page(1).len(), 14, "Interface (`:690-872`)");
        assert_eq!(page(2).len(), 15, "Video draws 15 rows (`:873-1064`)");
        assert_eq!(page(3).len(), 15, "Controls (`:1065-1441`)");
        // The drawn count is what the pitch divides by, except the pages whose
        // `num11` is the *last index* (the video page's documented quirk and
        // General's trailing separator).
        for cat in [1u8, 3] {
            assert_eq!(page(cat).len(), right_slots(cat), "category {cat}");
        }
        assert_eq!(page(2).len(), right_slots(2) + 1, "the video quirk");
        assert_eq!(page(0).len(), right_slots(0) + 1, "General's trailing spacer");
    }

    /// The left column's labels and actions are the C#'s (`:214-330`): the four
    /// categories then Keybinds/Achievements/Close/Save & Exit, and the social
    /// invite row stays out (AGENTS.md non-goals).
    #[test]
    fn the_left_column_is_the_vanilla_navigation() {
        let cats: Vec<u8> = LEFT_ROWS
            .iter()
            .filter_map(|r| match r.action {
                LeftAction::Category(c) => Some(c),
                _ => None,
            })
            .collect();
        assert_eq!(cats, vec![0, 1, 2, 3]);
        assert_eq!(LEFT_ROWS[4].action, LeftAction::Keybinds);
        assert_eq!(LEFT_ROWS[5].action, LeftAction::Achievements);
        assert_eq!(LEFT_ROWS[6].action, LeftAction::Close);
        assert_eq!(LEFT_ROWS[7].action, LeftAction::SaveAndExit);
        // Save & Exit is the one `Lang.inter` row (`:311`).
        assert_eq!(LEFT_ROWS[7].inter, Some(35));
        assert!(LEFT_ROWS[..7].iter().all(|r| r.menu.is_some()));
    }

    /// The left column's slot count is the C#'s `num7` (`:178-179`):
    /// `flag5 = SocialAPI.Network != null && SocialAPI.Network.CanInvite()`
    /// and `num7 = 5 + (flag5 ? 1 : 0) + 2`. Without a social API (the port's
    /// deliberate non-goal) `flag5` is false, so the `Invite Friends` row
    /// (`:264`) and its `num9++` (`:273-276`) never run and `num7 = 7` →
    /// eight slots (`for j = 0; j <= num7` `:187`).
    #[test]
    fn the_left_column_matches_the_c_sharp_num7_without_the_invite_row() {
        // `SocialAPI.Network == null` in the port → `flag5 == false`.
        let flag5 = false;
        let num7 = 5 + if flag5 { 1 } else { 0 } + 2;
        assert_eq!(num7, 7, "`num7 = 5 + 0 + 2`");
        // The loop `for (int j = 0; j <= num7; j++)` (`:187`) → `num7 + 1`.
        assert_eq!(LEFT_SLOTS, num7 + 1);
        assert_eq!(LEFT_ROWS.len(), LEFT_SLOTS);
        // The invite row's label is `Lang.menu[147]` (`:264`) — no port row
        // may carry it while the social-API arm is absent.
        assert!(LEFT_ROWS.iter().all(|r| r.menu != Some(147)));
    }

    /// The scale bands: the left `0.7..0.8` at `0.01` and the right
    /// `1.0..1.001` at `0.001` (`:157-166`, `:296-343`).
    #[test]
    fn the_scale_bands_are_the_c_sharp_defaults() {
        // The active culture is global state the sibling tests flip mid-run;
        // hold the guard the lang module's tests use and pin English, so the
        // culture branches inside these helpers stay deterministic.
        let _guard = crate::lang::test_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        crate::lang::set_language(GameCulture::English);
        let (lmin, lmax, lspeed) = left_scale_band();
        assert!((lmin - 0.7).abs() < 1e-6);
        assert!((lmax - 0.8).abs() < 1e-6);
        assert!((lspeed - 0.01).abs() < 1e-6);
        let (rmin, rmax, rspeed) = right_scale_band(0);
        assert!((rmin - 1.0).abs() < 1e-6);
        assert!((rmax - 1.001).abs() < 1e-6);
        assert!((rspeed - 0.001).abs() < 1e-6);
    }

    /// `Open`/`Close` (`:56-101`): the close clears the sticky grab and any
    /// pending action.
    #[test]
    fn the_window_toggles_and_resets_on_close() {
        let mut st = IngameOptions::new();
        assert!(!st.open, "the window starts shut");
        st.toggle();
        assert!(st.open);
        st.right_lock = 3;
        st.pending = Some(LeftAction::Close);
        st.toggle();
        assert!(!st.open, "the second toggle closes");
        assert_eq!(st.right_lock, -1, "the close clears the sticky grab");
        assert!(st.pending.is_none(), "the close clears the pending action");
    }

    /// The hover pass advances the scales towards their plateaus and emits the
    /// vanilla's `PlaySound(12)` on a hover change (`:187-195`).
    #[test]
    fn hovering_a_left_row_tickles_its_scale_and_cues_the_sound() {
        // The active culture is global state the sibling tests flip mid-run;
        // hold the guard the lang module's tests use and pin English, so the
        // culture branches inside these helpers stay deterministic.
        let _guard = crate::lang::test_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        crate::lang::set_language(GameCulture::English);
        let screen = Vec2::new(1920.0, 1080.0);
        let mut st = IngameOptions::new();
        st.open = true;
        assert_eq!(tick_hover(&mut st, screen, Vec2::new(0.0, 0.0), false), None);
        let a = left_row_anchor(screen, 1);
        let cue = tick_hover(&mut st, screen, Vec2::new(a[0] + 4.0, a[1] + 2.0), true);
        assert_eq!(cue, Some(Cue::Tick), "the C#'s PlaySound(12) on enter");
        assert_eq!(st.left_hover, 1);
        assert!(st.left_scale[1] > LEFT_SCALE_MIN, "the hovered row grows");
        assert!((st.left_scale[0] - LEFT_SCALE_MIN).abs() < 1e-6, "and the others stay");
    }

    /// The right scale band clamps at its plateau and never overshoots
    /// (`:333-380`).
    #[test]
    fn the_right_scale_clamps_at_its_band() {
        // The active culture is global state the sibling tests flip mid-run;
        // hold the guard the lang module's tests use and pin English, so the
        // culture branches inside these helpers stay deterministic.
        let _guard = crate::lang::test_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        crate::lang::set_language(GameCulture::English);
        let screen = Vec2::new(1920.0, 1080.0);
        let mut st = IngameOptions::new();
        st.open = true;
        let a = right_row_anchor(screen, 0, 10);
        for _ in 0..50 {
            tick_hover(&mut st, screen, Vec2::new(a[0] + 5.0, a[1]), true);
        }
        let (_, rmax, _) = right_scale_band(0);
        assert!(st.right_scale[10] <= rmax + 1e-6);
        assert!(st.right_scale[10] > 1.0, "the hovered row rose above the floor");
        for _ in 0..50 {
            tick_hover(&mut st, screen, Vec2::new(0.0, 0.0), false);
        }
        let (rmin, _, _) = right_scale_band(0);
        assert!((st.right_scale[10] - rmin).abs() < 1e-6, "and decays back");
    }

    /// The bars map onto the settings in both directions, the zoom track over
    /// its 1.0..2.0 range (`Main.GameZoomTarget - 1f`, `:530`).
    #[test]
    fn the_bars_round_trip_through_the_settings() {
        let mut s = Settings::default();
        for (bar, v) in [
            (Bar::Music, 0.25),
            (Bar::Sound, 0.5),
            (Bar::Ambient, 0.75),
            (Bar::Parallax, 0.4),
        ] {
            set_bar_value(&mut s, bar, v);
            assert!((bar_value(&s, bar) - v).abs() < 0.01, "{bar:?}");
        }
        set_bar_value(&mut s, Bar::Zoom, 0.5);
        assert!((s.zoom - 1.5).abs() < 1e-6, "the zoom bar is the 1.0..2.0 range");
        assert!((bar_value(&s, Bar::Zoom) - 0.5).abs() < 1e-6);
        // S1-H-3: the UI-scale row rides the C# slider's 0.5..2.0 range
        // (`Main.temporaryGUIScaleSlider = num14 * 1.5f + 0.5f`, `:577`; the
        // track is `(slider - 0.5f) / 1.5f`, `:571`). The verify row probes
        // 0.8 / 1.0 / 1.2.
        for want in [0.8f32, 1.0, 1.2] {
            let frac = (want - 0.5) / 1.5;
            set_bar_value(&mut s, Bar::UiScale, frac);
            assert!((s.ui_scale - want).abs() < 1e-6, "the bar sets {want}");
            assert!((bar_value(&s, Bar::UiScale) - frac).abs() < 1e-6, "and reads it back");
        }
    }

    /// Every toggle flips and flips back — the C#'s `Main.x = !Main.x` arms
    /// (`:613-685`, `:693-1063`, `:1381-1389`).
    #[test]
    fn every_toggle_flips_and_names_itself() {
        let all = [
            Toggle::AutoSave,
            Toggle::AutoPause,
            Toggle::PlayWhenUnfocused,
            Toggle::ReversedArmorBonuses,
            Toggle::PickupText,
            Toggle::EventProgress,
            Toggle::PlacementPreview,
            Toggle::HighlightNewItems,
            Toggle::TileGrid,
            Toggle::GamepadInstructions,
            Toggle::BossBarText,
            Toggle::FlashyEffectsInterface,
            Toggle::Background,
            Toggle::Blood,
            Toggle::ForegroundSunlight,
            Toggle::HeatDistortion,
            Toggle::StormEffects,
            Toggle::TilesSway,
            Toggle::ScreenShake,
            Toggle::FlashyEffectsWorld,
        ];
        let mut s = Settings::default();
        for t in all {
            let before = toggle_state(&s, t);
            flip_toggle(&mut s, t);
            assert_eq!(toggle_state(&s, t), !before, "{t:?} flips");
            flip_toggle(&mut s, t);
            assert_eq!(toggle_state(&s, t), before, "{t:?} flips back");
        }
    }

    /// The cycles advance through their state ladders, the quick-trash one
    /// through the C#'s three-way `PreferenceSettings` switch (`:841-862`).
    #[test]
    fn the_cycles_advance_through_their_ladders() {
        let mut s = Settings::default();
        s.quick_trash = false;
        s.disable_left_shift_trash = false;
        advance_cycle(&mut s, Cycle::QuickTrash);
        assert!(s.quick_trash && s.disable_left_shift_trash, "Disabled -> Left Control");
        advance_cycle(&mut s, Cycle::QuickTrash);
        assert!(s.quick_trash && !s.disable_left_shift_trash, "-> Left Shift");
        advance_cycle(&mut s, Cycle::QuickTrash);
        assert!(!s.quick_trash, "-> Disabled");

        s.frame_skip = 0;
        advance_cycle(&mut s, Cycle::FrameSkip);
        assert_eq!(s.frame_skip, 1);
        s.frame_skip = 2;
        advance_cycle(&mut s, Cycle::FrameSkip);
        assert_eq!(s.frame_skip, 0, "the ladder wraps");

        s.light_mode = 3;
        advance_cycle(&mut s, Cycle::LightMode);
        assert_eq!(s.light_mode, 0);
        s.wave_quality = 2;
        advance_cycle(&mut s, Cycle::WaveQuality);
        assert_eq!(s.wave_quality, 0);

        // The three boolean cycles are two-state rows.
        s.smart_blocks = true;
        advance_cycle(&mut s, Cycle::SmartBlocks);
        assert!(!s.smart_blocks);
        s.smart_cursor_toggle = true;
        advance_cycle(&mut s, Cycle::SmartCursorMode);
        assert!(!s.smart_cursor_toggle);
        s.smart_axe_first = false;
        advance_cycle(&mut s, Cycle::SmartCursorPriority);
        assert!(s.smart_axe_first);
    }

    /// The cycle labels follow their state — `Lang.menu[247 + FrameSkipMode]`
    /// (`:946`), `[59 + qaStyle]` (`:964`) and the two-state pairs.
    #[test]
    fn the_cycle_labels_follow_their_state() {
        // The active culture is global state the sibling tests flip mid-run;
        // hold the guard the lang module's tests use and pin English, so the
        // culture branches inside these helpers stay deterministic.
        let _guard = crate::lang::test_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        crate::lang::set_language(GameCulture::English);
        let mut s = Settings::default();
        for mode in 0..3u8 {
            s.frame_skip = mode;
            assert_eq!(cycle_label(&s, Cycle::FrameSkip), crate::lang::menu(247 + mode as usize));
        }
        for qa in 0..4u8 {
            s.quality = qa;
            assert_eq!(cycle_label(&s, Cycle::Quality), crate::lang::menu(59 + qa as usize));
        }
        for mode in 0..4u8 {
            s.light_mode = mode;
            assert_eq!(
                cycle_label(&s, Cycle::LightMode),
                crate::lang::text(&format!("UI.LightMode_{mode}"))
            );
        }
        s.smart_axes_labels();
    }

    impl Settings {
        /// The three smart-cursor label states, asserted through the public
        /// accessor so the test stays a black box.
        fn smart_axes_labels(&mut self) {
            self.smart_blocks = true;
            assert_eq!(cycle_label(self, Cycle::SmartBlocks), crate::lang::menu(215));
            self.smart_blocks = false;
            assert_eq!(cycle_label(self, Cycle::SmartBlocks), crate::lang::menu(216));
            self.smart_cursor_toggle = true;
            assert_eq!(cycle_label(self, Cycle::SmartCursorMode), crate::lang::menu(121));
            self.smart_cursor_toggle = false;
            assert_eq!(cycle_label(self, Cycle::SmartCursorMode), crate::lang::menu(122));
            self.smart_axe_first = false;
            assert_eq!(
                cycle_label(self, Cycle::SmartCursorPriority),
                crate::lang::menu(213)
            );
            self.smart_axe_first = true;
            assert_eq!(
                cycle_label(self, Cycle::SmartCursorPriority),
                crate::lang::menu(214)
            );
        }
    }

    /// A bar row's label carries the percentage the C# concatenates (`:407`).
    #[test]
    fn the_bar_labels_carry_the_percentage() {
        let _guard = crate::lang::test_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        crate::lang::init(GameCulture::English);
        let mut s = Settings::default();
        s.music_volume = 0.42;
        let row = &page(0)[1];
        assert_eq!(row.act, RowAct::Bar(Bar::Music));
        let label = row_label(row, &s);
        assert!(label.ends_with(" 42%"), "{label}");
        assert!(label.starts_with(&crate::lang::menu(99)));
        // The zoom row is the `GameUI.GameZoom` template with its two args —
        // the **target percent** (`:512`), not the bar's `- 1f` fraction.
        s.zoom = 1.5;
        let zoom = &page(0)[6];
        assert_eq!(zoom.act, RowAct::Bar(Bar::Zoom));
        let label = row_label(zoom, &s);
        assert!(
            label.contains("150") && !label.contains(" 50%"),
            "the zoom label is `round(zoom * 100)`, not `round((zoom - 1) * 100)`: {label}"
        );
        // ... while the bar's own fraction is still `zoom - 1` (`:529`).
        assert_eq!(bar_value(&s, Bar::Zoom), 0.5);
    }

    /// A category click resets the right scales, the C#'s `:307-311`.
    #[test]
    fn a_category_change_resets_the_right_scales() {
        let mut st = IngameOptions::new();
        st.category = 0;
        st.right_scale[3] = 1.0;
        // The reset the click arm performs.
        st.right_scale = [0.0; 17];
        st.category = 1;
        assert_eq!(st.right_scale[3], 0.0);
        assert_eq!(st.category, 1);
    }

    /// Every row of every page sits inside the panel's padded rect — the
    /// columns must not spill out of the 670x480 box (`:125-129`).
    #[test]
    fn every_row_sits_inside_the_panel() {
        // The active culture is global state the sibling tests flip mid-run;
        // hold the guard the lang module's tests use and pin English, so the
        // culture branches inside these helpers stay deterministic.
        let _guard = crate::lang::test_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        crate::lang::set_language(GameCulture::English);
        let screen = Vec2::new(1920.0, 1080.0);
        let o = panel_origin(screen);
        let (top, bottom) = (o[1], o[1] + PANEL_H);
        for i in 0..LEFT_SLOTS {
            let a = left_row_anchor(screen, i);
            assert!(a[1] > top && a[1] < bottom, "left row {i} at y {}", a[1]);
        }
        for cat in 0..4u8 {
            for i in 0..page(cat).len() {
                let a = right_row_anchor(screen, cat, i);
                assert!(
                    a[1] > top && a[1] < bottom,
                    "category {cat} row {i} at y {} leaves the panel",
                    a[1]
                );
            }
        }
    }

    /// Every row of every page resolves a non-empty label except the explicit
    /// `""` spacers (a `Header` row with a `Blank` label), so no page draws a
    /// blank setting line.
    #[test]
    fn every_row_resolves_a_label() {
        let s = Settings::default();
        for cat in 0..4u8 {
            for (i, row) in page(cat).iter().enumerate() {
                let label = row_label(row, &s);
                let spacer = row.label == Label::Blank
                    && matches!(row.act, RowAct::Header | RowAct::Parked);
                if spacer {
                    assert!(label.is_empty(), "cat {cat} row {i} is a spacer");
                } else {
                    assert!(!label.is_empty(), "cat {cat} row {i} must have a label");
                }
            }
        }
    }

    /// The `General` page's three "parked" rows are real now: the doors, hover
    /// and autofire preferences carry the vanilla keys (`:644-684`) instead of a
    /// raw `UI.DoorOpeningPreference`, which the language files do not have and
    /// which therefore drew as the key itself.
    #[test]
    fn the_general_preferences_are_localised_and_clickable() {
        // The active culture is global state the sibling tests flip mid-run;
        // hold the guard the lang module's tests use and pin English, so the
        // culture branches inside these helpers stay deterministic.
        let _guard = crate::lang::test_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        crate::lang::set_language(GameCulture::English);
        let mut s = Settings::default();
        let rows = page(0);
        // `:644-657`: the doors row cycles Disabled -> Everything -> Gamepad.
        assert_eq!(
            row_label(&rows[14], &s),
            crate::lang::text("UI.SmartDoorsDisabled")
        );
        advance_cycle(&mut s, Cycle::SmartDoors);
        assert_eq!(s.smart_doors, 2);
        assert_eq!(
            row_label(&rows[14], &s),
            crate::lang::text("UI.SmartDoorsEnabled")
        );
        advance_cycle(&mut s, Cycle::SmartDoors);
        assert_eq!(s.smart_doors, 1, "EnabledForGamepadOnly");
        assert_eq!(
            row_label(&rows[14], &s),
            crate::lang::text("UI.SmartDoorsGamepad")
        );
        advance_cycle(&mut s, Cycle::SmartDoors);
        assert_eq!(s.smart_doors, 0, "back to Disabled");
        // `:659-675`: Hold <-> Click.
        assert_eq!(
            row_label(&rows[15], &s),
            crate::lang::text("UI.HoverControlSettingIsHold")
        );
        advance_cycle(&mut s, Cycle::HoverControl);
        assert_eq!(
            row_label(&rows[15], &s),
            crate::lang::text("UI.HoverControlSettingIsClick")
        );
        // `:677-684`: the autofire toggle.
        assert_eq!(
            row_label(&rows[16], &s),
            crate::lang::text("UI.AutoReuseAllOff")
        );
        flip_toggle(&mut s, Toggle::AutoReuseAll);
        assert_eq!(
            row_label(&rows[16], &s),
            crate::lang::text("UI.AutoReuseAllOn")
        );
        // None of the three is a raw, unresolvable key.
        for i in [14usize, 15, 16] {
            let l = row_label(&rows[i], &s);
            assert!(!l.starts_with("UI."), "row {i} still draws a bare key: {l}");
        }
    }
}
