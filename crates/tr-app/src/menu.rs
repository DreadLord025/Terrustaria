//! `Main.DrawMenu`: the original immediate-mode menu.
//!
//! Geometry, font and colours follow `Main.DrawMenu` (`Main.cs`):
//!
//! * every item is a `FontAssets.DeathText` string centred on `Main.screenWidth / 2`;
//! * the first row sits at `num2` and rows are `num4` px apart, with a per-item
//!   `array4` y offset — those constants are per screen and live in [`Page::layout`];
//! * `array7` is the per-screen text scale (1.0 main menu, 0.75 settings, 0.78 general,
//!   0.6 interface/video) and multiplies the per-item `menuItemScale`;
//! * `menuItemScale` eases from 0.8 to 1.0 by 0.02 per frame for the focused item and
//!   back for the rest; the text alpha is `255 * (scale * 2 - 1)`, so unfocused items are
//!   dimmer, and the focused item is tinted `(255, 215, 0)`;
//! * each item is drawn five times — four black copies offset by ±2 px and the coloured
//!   pass — which is the original's drop shadow;
//! * the logo sits at `(screenWidth / 2, 100)` with its centre as origin and the
//!   `logoScale`/`logoRotation` easing, and the version string is the bottom-left
//!   `Mouse_Text` line (`Main.DrawVersionNumber`).
//!
//! The world backdrop behind the menu is ours (the original renders the world at the
//! surface); everything above it is the original layout.

use glam::Vec2;
use tr_render::SpriteRenderer;

use crate::assets;
use crate::settings::Settings;
use crate::ui::{color, sliders};

/// `Main.maxMenuItems` is 20; the largest screen here is the 12-row interface list.
pub const MAX_ITEMS: usize = 20;

/// The `Language.*` values of `Terraria.Localization.Content.en-US.json` in the
/// `Main.cs:48466-48477` (1212) / `:48497-48508` (1213) fill order — English,
/// German, Italian, French, Spanish, Russian, Chinese, Portuguese, Polish,
/// Japanese, Korean, ChineseTraditional. Note that is *not* the JSON's own key
/// order; both branches assign the twelve `Language.GetTextValue` rows
/// explicitly.
const LANGUAGES: [&str; 12] = [
    "English",
    "Deutsch (German)",
    "Italiano (Italian)",
    "Français (French)",
    "Español (Spanish)",
    "Русский (Russian)",
    "简体中文 (Simplified Chinese)",
    "Português brasileiro (Brazilian Portuguese)",
    "Polski (Polish)",
    "日本語 (Japanese)",
    "한국어 (Korean)",
    "繁體中文 (Traditional Chinese)",
];

/// The `Lang.menu` prefixes of the menuMode 27 rows (`Main.cs:50527-50540`):
/// the raw `en-US.Legacy.json` values — [74]..[85], [120], [130] — with
/// vanilla's trailing-space column alignment kept byte for byte.
const KEYBIND_ROWS: [&str; 14] = [
    "Up                ",    // `Lang.menu[74]`
    "Down             ",     // [75]
    "Left               ",   // [76]
    "Right             ",    // [77]
    "Jump             ",     // [78]
    "Throw            ",     // [79]
    "Inventory         ",    // [80]
    "Quick Heal       ",     // [81]
    "Quick Mana      ",      // [82]
    "Quick Buff       ",     // [83]
    "Grapple           ",    // [84]
    "Auto Select       ",    // [85] (the torch bind)
    "Smart Cursor    ",      // [120]
    "Mount             ",    // [130]
];

/// The `Lang.menu` prefixes of the menuMode 272727 rows
/// (`Main.cs:50425-50430`): [106]..[111], same trailing-space values.
const MAP_KEY_ROWS: [&str; 6] = [
    "Toggle Map Style           ",   // `Lang.menu[106]`
    "Toggle Fullscreen           ",  // [107]
    "Zoom In                      ", // [108]
    "Zoom Out                    ",  // [109]
    "Decrease Transparency     ",    // [110]
    "Increase Transparency      ",   // [111]
];

/// The nine `MinimapFrameManager` option keys in `CreateAndAdd` order
/// (`MinimapFrameManager.cs:25-33`) — the same order as
/// `ui::minimap::FRAME_STYLES`, so the stored key stays the live renderer's
/// input (`main.rs` maps it through `ui::minimap::style_index`).
pub const MINIMAP_FRAME_KEYS: [&str; 9] = [
    "Default",
    "Golden",
    "Remix",
    "Sticks",
    "StoneGold",
    "TwigLeaf",
    "Leaf",
    "Retro",
    "Valkyrie",
];

/// The six `PlayerResourceSetsManager` set keys in load order
/// (`PlayerResourceSetsManager.cs:42-47`).
pub const RESOURCE_SET_KEYS: [&str; 6] = [
    "New",
    "Default",
    "HorizontalBarsWithFullText",
    "HorizontalBarsWithText",
    "HorizontalBars",
    "NewWithText",
];

/// The `UI.HealthManaStyle_*` values (`en-US.json`), keyed by the set key —
/// the menuMode 1112 row composes `UI.SelectHealthStyle` with
/// `ActiveSetKeyName` (`Main.cs:49948`).
fn resource_set_label(key: &str) -> &'static str {
    match key {
        "New" => "Fancy",
        "Default" => "Classic",
        "HorizontalBarsWithFullText" => "Bars 3",
        "HorizontalBarsWithText" => "Bars 2",
        "HorizontalBars" => "Bars",
        "NewWithText" => "Fancy 2",
        // An unknown key normalizes to the first set
        // (`PlayerResourceSetsManager.cs:74-83`).
        _ => "Fancy",
    }
}

/// `SelectionHolder.CycleSelection` (`SelectionHolder.cs:44-58`) and its
/// `CycleResourceSet` twin (`PlayerResourceSetsManager.cs:84-101`): walk the
/// options, remember the last one *before* the active selection and activate
/// it — the cycle runs **backwards** through the registration order, and the
/// first option wraps to the last (`lastFrame == null -> Values.Last()`). An
/// unknown current key normalizes to the first option first
/// (`SetActiveFrame(string)`, `SelectionHolder.cs:67-78`), which also wraps to
/// the last.
fn cycle_previous(order: &[&str], current: &str) -> String {
    match order.iter().position(|key| *key == current) {
        Some(0) | None => order[order.len() - 1].to_string(),
        Some(i) => order[i - 1].to_string(),
    }
}

/// The `Main.cUp..cMount` defaults (`Main.cs:1979-2005`), seeded by the
/// static initializer (`Main.cs:67163-67176`) and restored by
/// `ResetKeyBindings` (`Main.cs:52459-52475`).
fn default_keybinds() -> [String; 14] {
    [
        "W", "S", "A", "D", "Space", "T", "Escape", "H", "J", "B", "E", "LeftShift",
        "LeftControl", "R",
    ]
    .map(String::from)
}

/// The `Main.cMap*` defaults (`Main.cs:67190-67195`), restored inline by the
/// menuMode 272727 Reset row (`Main.cs:50451-50456`).
fn default_map_binds() -> [String; 6] {
    ["Tab", "M", "Add", "Subtract", "PageUp", "PageDown"].map(String::from)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    Main,
    Settings,
    General,
    Interface,
    Video,
    Multiplayer,
    Host,
    Join,
    // The P5 settings sub-screens, keyed by their vanilla `menuMode` (the tasks
    // fill their rows in; Task 1 registers the pages so `TR_PAGE=<menuMode>`
    // can screenshot each one as it lands):
    /// menuMode 26 (`Main.cs:50362`).
    Volume,
    /// menuMode 28 (`Main.cs:50381`).
    Parallax,
    /// menuMode 1125 (`Main.cs:50242`).
    Cursor,
    /// menuMode 25 (`Main.cs:50318`).
    CursorColor,
    /// menuMode 252 (`Main.cs:50340`).
    BorderColor,
    /// menuMode 111 (`Main.cs:50169`).
    Resolution,
    /// menuMode 2008 (`Main.cs:50097`).
    Effects,
    /// menuMode 1213 (`Main.cs:48493`).
    Language,
    /// menuMode 1212 (`Main.cs:48458`).
    LanguageInitial,
    /// menuMode 1127 (`Main.cs:50638`).
    Controls,
    /// menuMode 27 (`Main.cs:50501`) — unreachable upstream, `TR_PAGE=27` only.
    Keybindings,
    /// menuMode 272727 (`Main.cs:50401`) — unreachable upstream, `TR_PAGE=272727` only.
    MapKeys,
    /// menuMode 13 (`Main.cs:50737`).
    JoinIp,
    /// menuMode 131 (`Main.cs:50884`).
    JoinPort,
    /// menuMode 30 (`Main.cs:48660`).
    HostPassword,
    /// menuMode 31 (`Main.cs:48536`).
    JoinPassword,
    /// menuMode 889 (`Main.cs:48726`).
    ServerSettings,
}

/// `num2` / `num4` / `array7` of one screen.
#[derive(Clone, Copy)]
pub struct Layout {
    pub y0: f32,
    pub row: f32,
    pub scale: f32,
}

impl Page {
    /// The per-screen constants from `Main.DrawMenu`. Custom screens (multiplayer)
    /// reuse the main-menu spacing.
    pub fn layout(self) -> Layout {
        match self {
            Page::Main => Layout { y0: 220.0, row: 52.0, scale: 1.0 },
            Page::Settings => Layout { y0: 210.0, row: 37.0, scale: 0.75 },
            Page::General => Layout { y0: 250.0, row: 52.0, scale: 0.78 },
            Page::Interface => Layout { y0: 210.0, row: 28.0, scale: 0.6 },
            Page::Video => Layout { y0: 186.0, row: 30.0, scale: 0.6 },
            Page::Multiplayer | Page::Host | Page::Join => {
                Layout { y0: 220.0, row: 52.0, scale: 1.0 }
            }
            // menuMode 26 (`Main.cs:50362-50365`).
            Page::Volume => Layout { y0: 200.0, row: 10.0, scale: 1.0 },
            // menuMode 28 (`Main.cs:50381-50384`).
            Page::Parallax => Layout { y0: 240.0, row: 60.0, scale: 1.0 },
            // menuMode 1125 (`Main.cs:50242-50246`).
            Page::Cursor => Layout { y0: 232.0, row: 38.0, scale: 0.73 },
            // menuMode 25 / 252 (`Main.cs:50318-50361`).
            Page::CursorColor | Page::BorderColor => {
                Layout { y0: 200.0, row: 10.0, scale: 1.0 }
            }
            // menuMode 111 (`Main.cs:50169-50173`).
            Page::Resolution => Layout { y0: 210.0, row: 55.0, scale: 0.85 },
            // menuMode 2008 (`Main.cs:50099-50107`): every `array7` is 0.6, the
            // header row overrides to 0.7.
            Page::Effects => Layout { y0: 210.0, row: 28.0, scale: 0.6 },
            // menuMode 1213 (`Main.cs:48525-48534`): `array7` 0.65, header 0.85,
            // Back 0.8.
            Page::Language => Layout { y0: 200.0, row: 26.0, scale: 0.65 },
            // menuMode 1212 (`Main.cs:48460-48488`): `array7` 0.65, header 1.0.
            Page::LanguageInitial => Layout { y0: 200.0, row: 30.0, scale: 0.65 },
            // menuMode 1127 (`Main.cs:50638-50642`).
            Page::Controls => Layout { y0: 250.0, row: 52.0, scale: 0.78 },
            // menuMode 27 (`Main.cs:50501-50505`).
            Page::Keybindings => Layout { y0: 176.0, row: 22.0, scale: 0.45 },
            // menuMode 272727 (`Main.cs:50401-50405`).
            Page::MapKeys => Layout { y0: 200.0, row: 30.0, scale: 0.55 },
            // menuMode 13 (`Main.cs:50737-50742`).
            Page::JoinIp => Layout { y0: 180.0, row: 30.0, scale: 1.0 },
            // menuMode 131 / 30 / 31 keep the vanilla `num2`/`num4` defaults of
            // 250/80 (`Main.cs:48419-48421`; `:50884`, `:48660`, `:48536`).
            Page::JoinPort | Page::HostPassword | Page::JoinPassword => {
                Layout { y0: 250.0, row: 80.0, scale: 1.0 }
            }
            // menuMode 889 (`Main.cs:48726-48809`).
            Page::ServerSettings => Layout { y0: 200.0, row: 60.0, scale: 1.0 },
        }
    }
}

pub struct Field {
    pub label: &'static str,
    pub value: String,
    pub secret: bool,
    pub digits: bool,
}

pub enum Action {
    None,
    Singleplayer,
    Host { port: u16, password: String },
    Join { addr: String, password: String },
    Quit,
}

/// One settings row. Every variant is a `Main`/`ItemSlot` field the screens toggle.
#[derive(Clone, Copy)]
enum Toggle {
    AutoSave,
    AutoPause,
    MapEnabled,
    HidePassword,
    PlayWhenUnfocused,
    PickupText,
    EventProgress,
    PlacementPreview,
    HighlightNew,
    TileGrid,
    GamepadInstructions,
    FrameSkip,
    LightMode,
    Quality,
    Background,
    Blood,
    MinersWobble,
    TilesSway,
    Zoom,
    // The menuMode 1125 rows (`Main.cs:50269-50308`).
    SmartCursorToggle,
    SmartAxeFirst,
    SmartBlocks,
    LockOnPriority,
    // The menuMode 2008 rows (`Main.cs:50113-50162`).
    UseStormEffects,
    ForegroundSunlight,
    UseHeatDistortion,
    WaveQuality,
    UseScreenShake,
    FlashyEffectsWorld,
    // The menuMode 1127 rows (`Main.cs:50647-50687`).
    ReversedArmorBonuses,
    QuickTrash,
    // The menuMode 1112 selector rows (`Main.cs:49941-49979`).
    MinimapFrame,
    ResourceSets,
    PortraitPreference,
    BossBarText,
    FlashyEffectsInterface,
}

enum Act {
    None,
    Nav(Page),
    /// The menuMode 1125 colour rows (`Main.cs:50253-50268`): seed
    /// `Main.selColor` from the stored colour, then open 25 or 252.
    PickColor(Page),
    Out(Action),
    Toggle(Toggle),
    /// The menuMode 27/272727 bind rows (`Main.cs:50565-50568`,
    /// `:50460-50463`): `Main.setKey = this.selectedMenu` — the row shows
    /// `"_"` and [`Menu::key`] fills the capture. The map twin is told apart
    /// by the page the row lives on.
    BindKey { index: usize },
    /// The "Reset to Default" rows (`Main.cs:50559-50564` / `:50449-50459`):
    /// restore the default binds and close the capture.
    ResetKeys { map: bool },
    /// A vanilla screen that is not ported yet: clicking shows the reason.
    Hint(&'static str),
}

struct Item {
    label: String,
    /// `array7`: text scale for this row.
    scale: f32,
    /// `array4`: vertical offset in pixels.
    y: f32,
    /// `array5`: horizontal offset in pixels.
    x: f32,
    /// `array[]`: drawn but not clickable (screen titles).
    header: bool,
    /// `array8[]` (`Main.cs:51773`): drawn left-aligned from
    /// `num3 + array5` with origin `(0, h/2)` instead of centred — the
    /// menuMode 27/272727 bind rows (`Main.cs:50543-50545`, `:50433-50435`).
    left_aligned: bool,
    act: Act,
}

impl Item {
    fn new(label: impl Into<String>, act: Act) -> Self {
        Self {
            label: label.into(),
            scale: 1.0,
            y: 0.0,
            x: 0.0,
            header: false,
            left_aligned: false,
            act,
        }
    }

    fn at(mut self, y: f32) -> Self {
        self.y = y;
        self
    }

    fn scaled(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// `array5[i] = n`: the horizontal offset (the bind rows' -80/-140 nudge,
    /// `Main.cs:50545` / `:50435`).
    fn offset_x(mut self, x: f32) -> Self {
        self.x = x;
        self
    }

    /// `array8[i] = true` (`Main.cs:50543` / `:50433`): draw and hit-test from
    /// the left edge `num3 + array5` instead of centred (`Main.cs:51773`,
    /// `:51797-51806`).
    fn left(mut self) -> Self {
        self.left_aligned = true;
        self
    }

    /// `array[i] = true`: drawn but not clickable (screen titles,
    /// `Main.cs:51790-51796`).
    fn heading(mut self) -> Self {
        self.header = true;
        self
    }
}

/// `Math.Round(volume * 100f) + "%"` (`Main.cs:51501-51510`): the product is
/// computed in `f32` like the C#, then widened to double for `Math.Round`,
/// which rounds ties to even — 0.125 prints "12%", not "13%" (Review Focus 2).
pub fn volume_percent(v: f32) -> String {
    let product = (v * 100.0f32) as f64;
    format!("{:.0}%", product.round_ties_even())
}

/// The volume/parallax label colour (`Main.cs:48253-48254` + `:51515-51525`):
/// `b = (255 + 2 * tileColor.R) / 3` of the backdrop grey, then the draw-site
/// `R = (255 + b) / 2`; the final
/// `new Color((byte)num85, (byte)num85, (byte)num85, (byte)num84)` repeats
/// `color10.R` for all three channels, so the G/B assignments at
/// `:51516-51518` are dead stores (the parallax twin: `:51596-51606`).
pub fn slider_label_color(backdrop_r: u8) -> [u8; 4] {
    // `byte b = (byte)((255 + Main.tileColor.R * 2) / 3)` (`Main.cs:48253`).
    let b = ((255u32 + 2 * backdrop_r as u32) / 3) as u8;
    // `color10.R = (byte)((255 + color10.R) / 2)` (`Main.cs:51515-51516`); the
    // G/B twins at `:51516-51518` are overwritten before anything is drawn.
    let r = ((255u32 + b as u32) / 2) as u8;
    [r, r, r, 255]
}

pub struct Menu {
    pub page: Page,
    /// Keyboard cursor (`focusMenu` while the mouse is elsewhere).
    pub selected: usize,
    /// Item under the mouse, if any.
    pub hover: Option<usize>,
    /// True once the player used the mouse or the arrow keys: until then no row is
    /// focused, which is how `Main.DrawMenu` starts (every item at 0.8).
    focus_keyboard: bool,
    item_scale: [f32; MAX_ITEMS],
    /// Screen-space boxes of the last frame, for hit-testing.
    boxes: Vec<[f32; 4]>,
    pub fields: Vec<Field>,
    pub error: Option<String>,
    pub connecting: bool,
    pub name: String,
    /// Host/join address shown after the server starts.
    pub status: Option<String>,
    /// The settings screens edit this; `App` copies it when the menu closes.
    pub settings: Settings,
    /// `Main.mouseX/mouseY` at draw time: the pointer (the `TR_MOUSE` pin when
    /// one is set), kept current by [`Menu::update_hover`] so the value bars
    /// draw and drag in the same pass as the original.
    pub mouse: Vec2,
    /// `Main.mouseLeft` (`Main.cs:51101`): the real left button, set by `App`
    /// on the mouse-input events while the menu is open.
    pub mouse_down: bool,
    /// `IngameOptions.rightLock` (`Main.cs:51527-51572`): the sticky value-bar
    /// grab — 0 music, 1 sound, 2 ambient, 3 parallax (`rightLock` 3/2/4 and
    /// the parallax block's 2). Cleared on release and on page changes.
    pub bar_grab: Option<u8>,
    /// `Main.grabColorSlider` (`Main.cs:51104-51115`): the sticky HSL grab —
    /// 1 hue, 2 sat, 3 lum, 4 alpha; 0 when idle. Reset whenever the menu is
    /// not on a colour screen.
    pub color_grab: u8,
    /// `Main.hBar`/`sBar`/`lBar`/`aBar` (`Main.cs:51104-51137`): the HSL bar
    /// values, `-1.0` meaning "unseeded" until the first slider frame seeds
    /// them from `rgbToHsl(selColor)`.
    pub hsl_bars: sliders::HslBars,
    /// `Main.selColor` (`Main.cs:50255` / `:50266` / `:51279-51284`): the live
    /// edit colour of the 25/252 screens. The struct default is all-zero, like
    /// XNA `Color`'s.
    pub sel_color: [u8; 4],
    /// `Main.setKey` (`Main.cs:50408-50410` / `:50523-50526`): the bind row
    /// being rebound on the menuMode 27/272727 lists — the row shows `"_"`
    /// and the next key press fills it ([`Menu::key`]); `None` when idle.
    pub set_key: Option<usize>,
    /// `Main.cUp..cMount` (`Main.cs:1979-2005`): the 14 gameplay binds of the
    /// menuMode 27 list, defaulted from the static seed (`:67163-67176`).
    /// Vanilla saves them in the player file, not `config.json`, so the port
    /// keeps them session-only.
    pub keybinds: [String; 14],
    /// `Main.cMapStyle..cMapAlphaDown` (`Main.cs:2051-2061`): the 6 map binds
    /// of the menuMode 272727 list, seeded at `:67190-67195`. Session-only.
    pub map_binds: [String; 6],
    /// `Main.screenWidth`/`screenHeight`, refreshed every [`Menu::draw`] — the
    /// menuMode 111 row prints the pending resolution, which vanilla keeps at
    /// the current window size (`Main.cs:66399-66400`). The default is the
    /// port's 1600×900 reference frame (`Main.cs:67078-67079` seeds vanilla's
    /// own 800×600 static before `Initialize` overwrites it).
    pub screen_size: Vec2,
    /// "Not ported yet" notice for the vanilla screens we do not have.
    pub hint: Option<String>,
    /// Write `config.json` on every toggle (off in unit tests).
    persist: bool,
    time: f32,
    logo_scale: f32,
    logo_dir: f32,
    logo_speed: f32,
    logo_rot: f32,
    rot_dir: f32,
    rot_speed: f32,
}

impl Menu {
    pub fn new(name: String) -> Self {
        let mut me = Self {
            page: Page::Main,
            selected: 0,
            hover: None,
            focus_keyboard: false,
            item_scale: [0.8; MAX_ITEMS],
            boxes: Vec::new(),
            fields: Vec::new(),
            error: None,
            connecting: false,
            name,
            status: None,
            settings: Settings::load(),
            mouse: Vec2::ZERO,
            mouse_down: false,
            bar_grab: None,
            color_grab: 0,
            hsl_bars: sliders::HslBars { h: -1.0, s: -1.0, l: -1.0, a: -1.0 },
            sel_color: [0, 0, 0, 0],
            set_key: None,
            keybinds: default_keybinds(),
            map_binds: default_map_binds(),
            screen_size: Vec2::new(1600.0, 900.0),
            hint: None,
            persist: true,
            time: 0.0,
            logo_scale: 0.1,
            logo_dir: 1.0,
            logo_speed: 0.0,
            logo_rot: 0.0,
            rot_dir: 1.0,
            rot_speed: 0.0,
        };
        // `TR_PAGE=settings|general|interface|video|multiplayer` or the numeric
        // `menuMode` names: open a screen directly so the layout can be
        // screenshotted deterministically.
        if let Ok(focus) = std::env::var("TR_FOCUS") {
            if let Ok(index) = focus.parse::<usize>() {
                me.selected = index;
                me.focus_keyboard = true;
            }
        }
        if let Ok(page) = std::env::var("TR_PAGE") {
            if let Some(p) = Self::page_from_name(&page) {
                me.page = p;
            }
        }
        me
    }

    /// The `TR_PAGE` name → page table: the word aliases plus the numeric
    /// `menuMode` names of every P5 screen, so each one can be screenshotted as
    /// `TR_PAGE=<menuMode>` while its rows land (plan Task 1). Unknown names
    /// leave the page untouched (the main menu).
    fn page_from_name(name: &str) -> Option<Page> {
        Some(match name.to_ascii_lowercase().as_str() {
            "settings" => Page::Settings,
            "general" => Page::General,
            "interface" => Page::Interface,
            "video" => Page::Video,
            "multiplayer" => Page::Multiplayer,
            "26" => Page::Volume,
            "28" => Page::Parallax,
            "1125" => Page::Cursor,
            "25" => Page::CursorColor,
            "252" => Page::BorderColor,
            "111" => Page::Resolution,
            "2008" => Page::Effects,
            "1213" => Page::Language,
            "1212" => Page::LanguageInitial,
            "1127" => Page::Controls,
            "27" => Page::Keybindings,
            "272727" => Page::MapKeys,
            "12" => Page::Multiplayer,
            "13" => Page::JoinIp,
            "131" => Page::JoinPort,
            "30" => Page::HostPassword,
            "31" => Page::JoinPassword,
            "889" => Page::ServerSettings,
            _ => return None,
        })
    }

    fn open_host(&mut self) {
        self.page = Page::Host;
        self.selected = 0;
        self.fields = vec![
            Field { label: "Port", value: "7777".into(), secret: false, digits: true },
            Field { label: "Password", value: String::new(), secret: true, digits: false },
        ];
    }

    fn open_join(&mut self) {
        self.page = Page::Join;
        self.selected = 0;
        self.fields = vec![
            Field { label: "Server IP", value: "127.0.0.1".into(), secret: false, digits: false },
            Field { label: "Port", value: "7777".into(), secret: false, digits: true },
            Field { label: "Password", value: String::new(), secret: true, digits: false },
        ];
    }

    /// The rows of the current screen, in `Main.DrawMenu` order.
    fn items(&self) -> Vec<Item> {
        let s = &self.settings;
        let on_off = |b: bool, on: &str, off: &str| -> String {
            if b { on.to_string() } else { off.to_string() }
        };
        match self.page {
            Page::Main => vec![
                Item::new("Single Player", Act::Out(Action::Singleplayer)),
                Item::new("Multiplayer", Act::Nav(Page::Multiplayer)),
                Item::new("Achievements", Act::Hint("Achievements are not ported yet")),
                Item::new("Resource Packs", Act::Hint("Resource packs are not ported yet")),
                Item::new("Settings", Act::Nav(Page::Settings)),
                Item::new("Credits", Act::Hint("Credits are not ported yet")),
                Item::new("Exit", Act::Out(Action::Quit)),
            ],
            // `menuMode == 11`.
            Page::Settings => vec![
                Item::new("General", Act::Nav(Page::General)).scaled(0.75),
                Item::new("Interface", Act::Nav(Page::Interface)).scaled(0.75),
                Item::new("Video", Act::Nav(Page::Video)).scaled(0.75),
                Item::new("Volume", Act::Nav(Page::Volume)).scaled(0.75),
                // menuMode 11 row 4 (`Main.cs:49753-49759`): opens 1125
                // (`Main.cs:50309-50316` is the 1125 Back twin).
                Item::new("Cursor", Act::Nav(Page::Cursor)).scaled(0.75),
                // Row 5 (`Main.cs:49752-49758`): `Lang.menu[219]` opens 1127.
                Item::new("Controls", Act::Nav(Page::Controls)).scaled(0.75),
                // Row 6 (`Main.cs:49759-49763`): `Lang.menu[103]` opens 1213.
                Item::new("Language", Act::Nav(Page::Language)).scaled(0.75),
                Item::new("Back", Act::Nav(Page::Main)).scaled(0.75).at(8.0),
            ],
            // `menuMode == 112`.
            Page::General => vec![
                Item::new(on_off(s.auto_save, "Autosave On", "Autosave Off"), Act::Toggle(Toggle::AutoSave)).scaled(0.78),
                Item::new(on_off(s.auto_pause, "Autopause On", "Autopause Off"), Act::Toggle(Toggle::AutoPause)).scaled(0.78),
                Item::new(on_off(s.map_enabled, "Map Enabled", "Map Disabled"), Act::Toggle(Toggle::MapEnabled)).scaled(0.78),
                Item::new(on_off(s.hide_password, "Passwords: Hidden", "Passwords: Visible"), Act::Toggle(Toggle::HidePassword)).scaled(0.78),
                Item::new(on_off(s.play_when_unfocused, "Pause When Unfocused: Off", "Pause When Unfocused: On"), Act::Toggle(Toggle::PlayWhenUnfocused)).scaled(0.78),
                Item::new("Back", Act::Nav(Page::Settings)).scaled(0.78).at(18.0),
            ],
            // `menuMode == 1112`.
            Page::Interface => vec![
                Item::new(on_off(s.pickup_text, "Pickup Text On", "Pickup Text Off"), Act::Toggle(Toggle::PickupText)).scaled(0.6),
                Item::new(
                    format!("Event Progress Bar {}", ["Off", "Timed", "On"][s.event_progress as usize % 3]),
                    Act::Toggle(Toggle::EventProgress),
                ).scaled(0.6),
                Item::new(on_off(s.placement_preview, "Placement Preview On", "Placement Preview Off"), Act::Toggle(Toggle::PlacementPreview)).scaled(0.6),
                Item::new(on_off(s.highlight_new_items, "Highlight New Items On", "Highlight New Items Off"), Act::Toggle(Toggle::HighlightNew)).scaled(0.6),
                Item::new(on_off(s.tile_grid, "Tile Grid On", "Tile Grid Off"), Act::Toggle(Toggle::TileGrid)).scaled(0.6),
                Item::new(on_off(s.gamepad_instructions, "Gamepad Instructions On", "Gamepad Instructions Off"), Act::Toggle(Toggle::GamepadInstructions)).scaled(0.6),
                // Row 6 (`Main.cs:49941-49946`): `UI.SelectMapBorder`
                // ("Minimap Border: {0}") with the active `UI.MinimapFrame_*`
                // name; the click calls `CycleSelection` with no sound. The
                // stored key feeds `ui::minimap::style_index` live, so the
                // minimap frame re-renders as the row cycles.
                Item::new(
                    format!(
                        "Minimap Border: {}",
                        // An unknown config name normalizes to the first
                        // option (`SelectionHolder.cs:71-78`).
                        if MINIMAP_FRAME_KEYS.contains(&s.minimap_frame.as_str()) {
                            s.minimap_frame.as_str()
                        } else {
                            MINIMAP_FRAME_KEYS[0]
                        }
                    ),
                    Act::Toggle(Toggle::MinimapFrame),
                )
                .scaled(0.6),
                // Row 7 (`Main.cs:49947-49952`): `UI.SelectHealthStyle` with
                // `UI.HealthManaStyle_{ActiveSetKeyName}`; no click sound.
                Item::new(
                    format!("Health and Mana Style: {}", resource_set_label(&s.resource_sets)),
                    Act::Toggle(Toggle::ResourceSets),
                )
                .scaled(0.6),
                // Row 8 (`Main.cs:49953-49965`): the plain `UI.Portraits*`
                // labels — the `switch`'s `_` arm reads Detailed; the click
                // runs `CycleNPCPortraitMode` (`Main.cs:40297-40315`).
                Item::new(
                    match s.portrait_preference {
                        1 => "Portraits Style: Profile",
                        2 => "Portraits Style: Retro",
                        3 => "Portraits Style: Disabled",
                        _ => "Portraits Style: Detailed",
                    },
                    Act::Toggle(Toggle::PortraitPreference),
                )
                .scaled(0.6),
                // Row 9 (`Main.cs:49966-49971`): `BigProgressBarSystem.
                // ToggleShowText()` with no sound. The flag is stored only —
                // P4's boss bar owns the numbers rendering.
                Item::new(
                    on_off(
                        s.boss_bar_text,
                        "Boss Health Bar Numbers: On",
                        "Boss Health Bar Numbers: Off",
                    ),
                    Act::Toggle(Toggle::BossBarText),
                )
                .scaled(0.6),
                // Row 10 (`Main.cs:49972-49979`): the
                // `UI.FlashyEffectsInterfaceOn/Off` pair.
                Item::new(
                    on_off(
                        s.flashy_effects_interface,
                        "Inventory Effects: On",
                        "Inventory Effects: Reduced",
                    ),
                    Act::Toggle(Toggle::FlashyEffectsInterface),
                )
                .scaled(0.6),
                Item::new("Back", Act::Nav(Page::Settings)).scaled(0.6).at(16.0),
            ],
            // `menuMode == 1111`. Note the original leaves `array7` at 1.0 for the Back
            // row here (only the first ten rows are set to 0.6), so Back is bigger.
            Page::Video => vec![
                // Row 0 (`Main.cs:49992-49996`): `Lang.menu[51]` opens 111.
                Item::new("Resolution", Act::Nav(Page::Resolution)).scaled(0.6),
                Item::new(
                    format!("Zoom: {:.0}%", s.zoom * 100.0),
                    Act::Toggle(Toggle::Zoom),
                )
                .scaled(0.6),
                // menuMode 1111 row 1 (`Main.cs:49996-50002`): `Lang.menu[52]`
                // opens the parallax slider (28).
                Item::new("Parallax", Act::Nav(Page::Parallax)).scaled(0.6),
                Item::new(["Frame Skip Off", "Frame Skip On", "Frame Skip Subtle"][s.frame_skip as usize % 3], Act::Toggle(Toggle::FrameSkip)).scaled(0.6),
                Item::new(["Lighting: Color", "Lighting: White", "Lighting: Retro", "Lighting: Trippy"][s.light_mode as usize % 4], Act::Toggle(Toggle::LightMode)).scaled(0.6),
                Item::new(["Quality: Auto", "Quality: High", "Quality: Medium", "Quality: Low"][s.quality as usize % 4], Act::Toggle(Toggle::Quality)).scaled(0.6),
                Item::new(on_off(s.background, "Background On", "Background Off"), Act::Toggle(Toggle::Background)).scaled(0.6),
                Item::new(on_off(s.blood, "Blood and Gore On", "Blood and Gore Off"), Act::Toggle(Toggle::Blood)).scaled(0.6),
                Item::new(on_off(s.miners_wobble, "Miner's Wobble: Enabled", "Miner's Wobble: Disabled"), Act::Toggle(Toggle::MinersWobble)).scaled(0.6),
                Item::new(on_off(s.tiles_sway, "Windy Environment: Enabled", "Windy Environment: Disabled"), Act::Toggle(Toggle::TilesSway)).scaled(0.6),
                // Row 10 (`Main.cs:50072-50077`): `UI.Effects` opens 2008.
                Item::new("Effects", Act::Nav(Page::Effects)).scaled(0.6),
                Item::new("Back", Act::Nav(Page::Settings)).at(8.0),
            ],
            Page::Multiplayer => vec![
                Item::new("Host & Play", Act::Nav(Page::Host)),
                Item::new("Join", Act::Nav(Page::Join)),
                Item::new("Back", Act::Nav(Page::Main)),
            ],
            Page::Host => vec![
                Item::new("Start Server", Act::Out(Action::Host {
                    port: self.fields.first().and_then(|f| f.value.trim().parse().ok()).unwrap_or(7777),
                    password: self.fields.get(1).map(|f| f.value.clone()).unwrap_or_default(),
                })),
                Item::new("Back", Act::Nav(Page::Multiplayer)),
            ],
            Page::Join => {
                let host = self.fields.first().map(|f| f.value.trim().to_string()).unwrap_or_default();
                let port = self.fields.get(1).and_then(|f| f.value.trim().parse().ok()).unwrap_or(7777);
                let addr = if host.contains(':') { host } else { format!("{host}:{port}") };
                vec![
                    Item::new("Connect", Act::Out(Action::Join {
                        addr,
                        password: self.fields.get(2).map(|f| f.value.clone()).unwrap_or_default(),
                    })),
                    Item::new("Back", Act::Nav(Page::Multiplayer)),
                ]
            }
            // menuMode 26 (`Main.cs:50362-50380`): `num5 = 3`, `array9[0] = ""`,
            // `array9[1] = Lang.menu[65]` with `array4[1] = 10` and
            // `array[1] = true`, `array9[2] = Lang.menu[5]` with
            // `array4[2] = 250`; Back opens 11 (`:50375-50379`).
            Page::Volume => vec![
                Item::new("", Act::None),
                Item::new("Volume", Act::None).at(10.0).heading(),
                Item::new("Back", Act::Nav(Page::Settings)).at(250.0),
            ],
            // menuMode 28 (`Main.cs:50381-50400`): the branch refreshes
            // `Main.caveParallax = 1 - bgScroll / 500` (`:50382`), `num5 = 3`,
            // `array9[1] = Lang.menu[52]` (header), `array4[2] = 170`; Back
            // opens 1111 (`:50395-50399`).
            Page::Parallax => vec![
                Item::new("", Act::None),
                Item::new("Parallax", Act::None).at(10.0).heading(),
                Item::new("Back", Act::Nav(Page::Video)).at(170.0),
            ],
            // menuMode 1125 (`Main.cs:50242-50317`): `num5 = 7`,
            // `array4[6] = 18` (`:50245`), every `array7` 0.73 (`:50247-50251`).
            Page::Cursor => vec![
                // Rows 0-1 (`Main.cs:50253-50268`): the colour rows seed
                // `Main.selColor` from the stored colour, then open 25 / 252.
                Item::new("Cursor Color", Act::PickColor(Page::CursorColor)).scaled(0.73),
                Item::new("Border Color", Act::PickColor(Page::BorderColor)).scaled(0.73),
                // Row 2 (`Main.cs:50269-50277`):
                // `cSmartCursorModeIsToggleAndNotHold ? Lang.menu[121] : [122]`.
                Item::new(
                    on_off(
                        s.smart_cursor_toggle,
                        "Smart Cursor Mode: Toggle",
                        "Smart Cursor Mode: Hold",
                    ),
                    Act::Toggle(Toggle::SmartCursorToggle),
                )
                .scaled(0.73),
                // Row 3 (`Main.cs:50278-50285`):
                // `SmartAxeAfterPickaxe ? Lang.menu[214] : [213]`.
                Item::new(
                    on_off(
                        s.smart_axe_first,
                        "Smart Cursor Priority: Axe → Pickaxe",
                        "Smart Cursor Priority: Pickaxe → Axe",
                    ),
                    Act::Toggle(Toggle::SmartAxeFirst),
                )
                .scaled(0.73),
                // Row 4 (`Main.cs:50286-50293`):
                // `SmartBlocksEnabled ? Lang.menu[215] : [216]`.
                Item::new(
                    on_off(
                        s.smart_blocks,
                        "Smart Block Placement: To Cursor",
                        "Smart Block Placement: Disabled",
                    ),
                    Act::Toggle(Toggle::SmartBlocks),
                )
                .scaled(0.73),
                // Row 5 (`Main.cs:50294-50308`): the `LockOnHelper.UseMode`
                // switch — FocusTarget [232], TargetClosest [233], ThreeDS [234]
                // (`LockOnHelper.cs:13-17`). FocusTarget only shows if the
                // stored value is 0, which `LoadSettings` rewrites to ThreeDS
                // (`Main.cs:4502-4505`).
                Item::new(
                    match s.lock_on_priority {
                        0 => "Lock On Priority: Focus Target",
                        1 => "Lock On Priority: Target Closest",
                        _ => "Lock On Priority: Clearest Line",
                    },
                    Act::Toggle(Toggle::LockOnPriority),
                )
                .scaled(0.73),
                // Row 6 (`Main.cs:50309-50316`): Back opens 11.
                Item::new("Back", Act::Nav(Page::Settings)).scaled(0.73).at(18.0),
            ],
            // menuMode 25 (`Main.cs:50318-50339`): `num5 = 3`,
            // `array9[1] = Lang.menu[64]` with `array4[1] = 10` and
            // `array[1] = true`, `array4[2] = 250`; the HSL sliders draw before
            // the rows and Back opens 1125 (`:50333-50338`).
            Page::CursorColor => vec![
                Item::new("", Act::None),
                Item::new("Cursor Color", Act::None).at(10.0).heading(),
                Item::new("Back", Act::Nav(Page::Cursor)).at(250.0),
            ],
            // menuMode 252 (`Main.cs:50340-50361`): the `Lang.menu[217]` twin —
            // same rows, full-range luminosity and the alpha bar.
            Page::BorderColor => vec![
                Item::new("", Act::None),
                Item::new("Border Color", Act::None).at(10.0).heading(),
                Item::new("Back", Act::Nav(Page::Cursor)).at(250.0),
            ],
            // menuMode 111 (`Main.cs:50169-50241`): `num2 = 210`, `num4 = 55`,
            // every `array7` 0.85 (`:50170-50174`). The window-resolution
            // system is out of scope (spec §9.1), so rows 0-3 keep their
            // vanilla labels but show the not-ported notice where the original
            // cycles display modes (`:50179-50191`), toggles
            // `PendingBorderlessState` (`:50198-50204`), calls
            // `ToggleFullScreen` (`:50206-50210`) or `SetResolution`
            // (`:50212-50227`).
            Page::Resolution => vec![
                // Row 0 (`Main.cs:50178`): `Lang.menu[73] + ": " +
                // PendingResolutionWidth + "x" + PendingResolutionHeight`; the
                // pending size mirrors the live window (`:66399-66400`).
                Item::new(
                    format!(
                        "Fullscreen Resolution: {}x{}",
                        self.screen_size.x as u32,
                        self.screen_size.y as u32
                    ),
                    Act::Hint("Window resolution is not ported yet"),
                )
                .scaled(0.85),
                // Row 1 (`Main.cs:50196-50205`): `Lang.menu[246]` while
                // `PendingBorderlessState` is false (a plain static, default
                // false, `Main.cs:1905`; `LoadSettings` re-syncs it from
                // `screenBorderless`, `:4296`); the row always shows —
                // `IsBorderlessDisplayAvailable` is `Platform.IsWindows`
                // (`Main.cs:52305-52312`).
                Item::new(
                    "Borderless Window: Disabled",
                    Act::Hint("Borderless windows are not ported yet"),
                )
                .scaled(0.85),
                // Row 2 (`Main.cs:50206-50210`): `Lang.menu[50]` while
                // windowed; the port always runs windowed.
                Item::new("Go Fullscreen", Act::Hint("Fullscreen is not ported yet")).scaled(0.85),
                // Row 3 (`Main.cs:50212-50227`): `array4 = 100`, label
                // `Lang.menu[134]`.
                Item::new("Apply", Act::Hint("Resolution switching is not ported yet"))
                    .scaled(0.85)
                    .at(100.0),
                // Row 4 (`Main.cs:50229-50239`): `array4 = 100`; Back resets
                // the pending state and opens 1111 (`:50237`).
                Item::new("Back", Act::Nav(Page::Video)).scaled(0.85).at(100.0),
            ],
            // menuMode 2008 (`Main.cs:50097-50168`): `num2 = 210`, `num4 = 28`,
            // `num5 = 9` with `array4[num5 - 1] = 24` (`:50099-50102`) — but
            // `array9[8]` is never assigned, and the item loop skips null
            // labels (`:51659-51662`), so the screen has eight visible rows and
            // Back sits at `210 + 28 * 7` = 406 (Review Focus 3; porting "9
            // rows" would draw a phantom duplicate).
            Page::Effects => vec![
                // Row 0 (`Main.cs:50108-50111`): the `UI.Effects` header,
                // `array[0] = true`, `array7 = 0.7`, `array4 -= 8`.
                Item::new("Effects", Act::None).at(-8.0).heading().scaled(0.7),
                // Row 1 (`Main.cs:50113-50118`):
                // `GameUI.StormEffects` with `GameUI.Enabled`/`Disabled`.
                Item::new(
                    on_off(s.use_storm_effects, "Storm Effects: Enabled", "Storm Effects: Disabled"),
                    Act::Toggle(Toggle::UseStormEffects),
                )
                .scaled(0.6),
                // Row 2 (`Main.cs:50120-50125`): `GameUI.ForegroundSunlightEffects`.
                Item::new(
                    on_off(
                        s.foreground_sunlight,
                        "Sunlight Effects: Enabled",
                        "Sunlight Effects: Disabled",
                    ),
                    Act::Toggle(Toggle::ForegroundSunlight),
                )
                .scaled(0.6),
                // Row 3 (`Main.cs:50127-50132`): `GameUI.HeatDistortion`.
                Item::new(
                    on_off(
                        s.use_heat_distortion,
                        "Heat Distortion: Enabled",
                        "Heat Distortion: Disabled",
                    ),
                    Act::Toggle(Toggle::UseHeatDistortion),
                )
                .scaled(0.6),
                // Row 4 (`Main.cs:50134-50146`): `GameUI.WaveQuality` with
                // 1 Low, 2 Medium, 3 High, anything else Off (`_` arm).
                Item::new(
                    format!(
                        "Waves Quality: {}",
                        match s.wave_quality {
                            1 => "Low",
                            2 => "Medium",
                            3 => "High",
                            _ => "Off",
                        }
                    ),
                    Act::Toggle(Toggle::WaveQuality),
                )
                .scaled(0.6),
                // Row 5 (`Main.cs:50150-50155`): `GameUI.ScreenShake`.
                Item::new(
                    on_off(
                        s.use_screen_shake,
                        "Screen Shake: Enabled",
                        "Screen Shake: Disabled",
                    ),
                    Act::Toggle(Toggle::UseScreenShake),
                )
                .scaled(0.6),
                // Row 6 (`Main.cs:50157-50162`): `UI.FlashyEffectsWorldOn` /
                // `UI.FlashyEffectsWorldOff` ("Thunder Effects: On/Reduced").
                Item::new(
                    on_off(s.flashy_effects_world, "Thunder Effects: On", "Thunder Effects: Reduced"),
                    Act::Toggle(Toggle::FlashyEffectsWorld),
                )
                .scaled(0.6),
                // Row 7 (`Main.cs:50164-50169`): Back opens 1111.
                Item::new("Back", Act::Nav(Page::Video)).scaled(0.6),
            ],
            // menuMode 1213 (`Main.cs:48493-48535`): the `Lang.menu[102]`
            // header at `array4[0] = -20` (`:48527`), the twelve
            // `Language.*` rows, then `array9[num10] = Lang.menu[5]` Back with
            // `array4[13] = 9` (`:48510`, `:48528`) — draw y `200 + 26*13 + 9`
            // = 547. `array7` 0.65, header 0.85, Back 0.8 (`:48529-48534`).
            // Language switching is out of scope (spec §9.1), so the rows show
            // the not-ported notice (`Main.cs:48519-48523` would call
            // `LanguageManager.SetLanguage`).
            Page::Language => {
                let mut rows = vec![
                    Item::new("Select language", Act::None).at(-20.0).heading().scaled(0.85),
                ];
                for language in LANGUAGES {
                    rows.push(Item::new(language, Act::Hint("Only English is bundled")).scaled(0.65));
                }
                rows.push(Item::new("Back", Act::Nav(Page::Settings)).scaled(0.8).at(9.0));
                rows
            }
            // menuMode 1212 (`Main.cs:48458-48491`): the first-launch picker —
            // the same `Lang.menu[102]` header (`array4[0] = -20`, `:48464`)
            // and twelve `Language.*` rows at `num2 = 200`, `num4 = 30`
            // (`:48460-48462`), no Back row, `array7[0] = 1f` (`:48483`),
            // languages 0.65. The original enters it on first launch
            // (`Main.cs:10655`); here it is reachable only via `TR_PAGE=1212`,
            // and a language row shows the same notice (the original would
            // `SetLanguage` and jump to menuMode 0, `:48484-48490`).
            Page::LanguageInitial => {
                let mut rows = vec![Item::new("Select language", Act::None).at(-20.0).heading()];
                for language in LANGUAGES {
                    rows.push(Item::new(language, Act::Hint("Only English is bundled")).scaled(0.65));
                }
                rows
            }
            // menuMode 1127 (`Main.cs:50638-50698`): `num2 = 250`, `num4 = 52`,
            // `num5 = 4`, `array4[3] = 18`, every `array7` 0.78
            // (`:50639-50646`). Draw y = `250 + 52*i + array4[i]`
            // (`Main.cs:51769`): 250, 302, 354, 424.
            Page::Controls => vec![
                // Row 0 (`Main.cs:50647-50656`):
                // `ReversedUpDownArmorSetBonuses ? [220] : [221]`.
                Item::new(
                    on_off(
                        s.reversed_armor_bonuses,
                        "Activate Set Bonuses: Up",
                        "Activate Set Bonuses: Down",
                    ),
                    Act::Toggle(Toggle::ReversedArmorBonuses),
                )
                .scaled(0.78),
                // Row 1 (`Main.cs:50656-50687`): the three-state quick-trash
                // cycle — `DisableQuickTrash ? [253] : (DisableLeftShiftTrashCan
                // ? [224] : [223])`.
                Item::new(
                    if s.quick_trash {
                        "Quick Trash: Disabled"
                    } else if s.disable_left_shift_trash {
                        "Quick Trash: Left Control"
                    } else {
                        "Quick Trash: Left Shift"
                    },
                    Act::Toggle(Toggle::QuickTrash),
                )
                .scaled(0.78),
                // Row 2 (`Main.cs:50688-50694`): `Lang.menu[222]` opens the
                // menuMode 888 `ManageControlsMenu` UIState (`:50661-50666`) —
                // out of scope (spec §9.1), so the row shows the notice.
                Item::new(
                    "Keybindings",
                    Act::Hint("The manage-controls screen is not ported yet"),
                )
                .scaled(0.78),
                // Row 3 (`Main.cs:50695-50698`): Back opens 11.
                Item::new("Back", Act::Nav(Page::Settings)).scaled(0.78).at(18.0),
            ],
            // menuMode 27 (`Main.cs:50501-50637`): `num2 = 176`, `num4 = 22`,
            // `num5 = 16`. Rows 0-13 are the bind list — the raw `Lang.menu`
            // value (trailing spaces kept) + the bind name
            // (`:50527-50540`), left-aligned at `array5 = -80` with scale 0.45
            // (`:50543-50545`); the row being rebound shows `"_"` instead of
            // its bind (`:50523-50526`). Unreachable upstream — no
            // `menuMode = 27` assignment exists under `decompiled/` (the
            // Settings "Controls" row opens 1127, whose Keybindings row opens
            // the 888 UIState) — `TR_PAGE=27` only.
            Page::Keybindings => {
                let mut rows = Vec::new();
                for (i, prefix) in KEYBIND_ROWS.iter().enumerate() {
                    let bind =
                        if self.set_key == Some(i) { "_" } else { self.keybinds[i].as_str() };
                    rows.push(
                        Item::new(format!("{prefix}{bind}"), Act::BindKey { index: i })
                            .scaled(0.45)
                            .offset_x(-80.0)
                            .left(),
                    );
                }
                // Row 14 (`Main.cs:50547-50549`): `array7` 0.8,
                // `array4 = 6`; resets the binds and closes the capture
                // (`:50559-50564`).
                rows.push(
                    Item::new("Reset to Default", Act::ResetKeys { map: false }).scaled(0.8).at(6.0),
                );
                // Row 15 (`Main.cs:50550-50552`): Back opens 11.
                rows.push(Item::new("Back", Act::Nav(Page::Settings)).scaled(0.8).at(16.0));
                rows
            }
            // menuMode 272727 (`Main.cs:50401-50500`): `num2 = 200`,
            // `num4 = 30`, `num5 = 14` — but only rows 0-7 get labels and the
            // item loop skips the nulls (`Main.cs:51654`), so the screen shows
            // eight rows. The six map binds are left-aligned at
            // `array5 = -140`, scale 0.55 (`:50433-50435`), labels [106..111]
            // + bind (`:50425-50430`); the row being rebound shows `"_"`.
            // Unreachable upstream — `TR_PAGE=272727` only.
            Page::MapKeys => {
                let mut rows = Vec::new();
                for (i, prefix) in MAP_KEY_ROWS.iter().enumerate() {
                    let bind = if self.set_key == Some(i) { "_" } else { self.map_binds[i].as_str() };
                    rows.push(
                        Item::new(format!("{prefix}{bind}"), Act::BindKey { index: i })
                            .scaled(0.55)
                            .offset_x(-140.0)
                            .left(),
                    );
                }
                // Row 6 (`Main.cs:50439-50442`): `array7` 0.8,
                // `array4 = 6`; restores the `:67190-67195` seeds
                // (`:50449-50459`).
                rows.push(
                    Item::new("Reset to Default", Act::ResetKeys { map: true }).scaled(0.8).at(6.0),
                );
                // Row 7 (`Main.cs:50441-50444`): `array4 = 16` but `array7[7]`
                // is never set here (the branch sets `array7[6]` twice,
                // `:50439-50440`), so Back keeps the 1.0 default — unlike the
                // mode-27 twin's 0.8 (`:50550`).
                rows.push(Item::new("Back", Act::Nav(Page::Settings)).at(16.0));
                rows
            }
            // The remaining P5 screens land with Task 6; Task 1 registers the
            // pages so `TR_PAGE=<menuMode>` resolves and each screen can be
            // screenshotted as its rows are filled in.
            Page::JoinIp
            | Page::JoinPort
            | Page::HostPassword
            | Page::JoinPassword
            | Page::ServerSettings => Vec::new(),
        }
    }

    fn parent(page: Page) -> Option<Page> {
        match page {
            Page::Main => None,
            Page::Settings => Some(Page::Main),
            Page::General | Page::Interface | Page::Video => Some(Page::Settings),
            Page::Multiplayer => Some(Page::Main),
            Page::Host | Page::Join => Some(Page::Multiplayer),
            // The Back rows of the Task 2-6 tables; the pages stay row-less
            // until their tasks fill `items()` in.
            Page::Volume | Page::Cursor | Page::Language | Page::Controls
            | Page::Keybindings | Page::MapKeys => Some(Page::Settings),
            Page::Parallax | Page::Resolution | Page::Effects => Some(Page::Video),
            Page::CursorColor | Page::BorderColor => Some(Page::Cursor),
            Page::LanguageInitial => None, // the first-launch picker has no Back
            Page::JoinIp | Page::HostPassword | Page::ServerSettings => Some(Page::Multiplayer),
            Page::JoinPort => Some(Page::JoinIp),
            Page::JoinPassword => Some(Page::JoinPort),
        }
    }

    fn activate(&mut self, index: usize) -> Action {
        let items = self.items();
        let Some(item) = items.get(index) else {
            return Action::None;
        };
        if item.header {
            return Action::None;
        }
        self.hint = None;
        match &item.act {
            Act::None => Action::None,
            Act::Nav(page) => {
                self.nav_to(*page);
                Action::None
            }
            Act::PickColor(page) => {
                // `Main.selColor = Main.mouseColor; ... Main.menuMode = 25;`
                // (`Main.cs:50254-50258`) / `Main.selColor =
                // Main.MouseBorderColor; ... Main.menuMode = 252;`
                // (`:50263-50267`). The `ColorSlidersSet.SetHSL` calls there
                // are gamepad chrome and are not ported.
                self.sel_color = match page {
                    Page::CursorColor => self.settings.mouse_color,
                    _ => self.settings.mouse_border_color,
                };
                self.nav_to(*page);
                Action::None
            }
            Act::Out(action) => match action {
                Action::Singleplayer => Action::Singleplayer,
                Action::Quit => Action::Quit,
                Action::Host { port, password } => {
                    Action::Host { port: *port, password: password.clone() }
                }
                Action::Join { addr, password } => {
                    Action::Join { addr: addr.clone(), password: password.clone() }
                }
                Action::None => Action::None,
            },
            Act::Toggle(toggle) => {
                self.toggle(*toggle);
                Action::None
            }
            // The menuMode 27/272727 bind rows (`Main.cs:50565-50568`,
            // `:50460-50463`): the clicked row opens the capture.
            Act::BindKey { index } => {
                self.set_key = Some(*index);
                Action::None
            }
            // The "Reset to Default" rows: `Main.ResetKeyBindings()` and
            // `setKey = -1` (`Main.cs:50559-50564`), or the map twin restoring
            // the `:67190-67195` seeds inline (`:50449-50459`).
            Act::ResetKeys { map } => {
                if *map {
                    self.map_binds = default_map_binds();
                } else {
                    self.keybinds = default_keybinds();
                }
                self.set_key = None;
                Action::None
            }
            Act::Hint(message) => {
                self.hint = Some((*message).to_string());
                Action::None
            }
        }
    }

    /// Shared body of [`Act::Nav`] / [`Act::PickColor`].
    fn nav_to(&mut self, page: Page) {
        if page == Page::Host {
            self.open_host();
        } else if page == Page::Join {
            self.open_join();
        } else {
            self.page = page;
            self.selected = 0;
            self.fields.clear();
        }
        // A grab never follows a page change (vanilla resets `rightLock`
        // in the `!Main.mouseLeft` guard of whichever screen runs next).
        self.bar_grab = None;
        self.error = None;
    }

    /// `Main`/`ItemSlot` toggles. Every row changes the stored setting; the ones the
    /// renderer reads (`LightMode`, `BackgroundEnabled`, `MapEnabled`, …) are applied by
    /// `App` when the menu closes.
    fn toggle(&mut self, toggle: Toggle) {
        let s = &mut self.settings;
        match toggle {
            Toggle::AutoSave => s.auto_save = !s.auto_save,
            Toggle::AutoPause => s.auto_pause = !s.auto_pause,
            Toggle::MapEnabled => s.map_enabled = !s.map_enabled,
            Toggle::HidePassword => s.hide_password = !s.hide_password,
            Toggle::PlayWhenUnfocused => s.play_when_unfocused = !s.play_when_unfocused,
            Toggle::PickupText => s.pickup_text = !s.pickup_text,
            Toggle::EventProgress => s.event_progress = (s.event_progress + 1) % 3,
            Toggle::PlacementPreview => s.placement_preview = !s.placement_preview,
            Toggle::HighlightNew => s.highlight_new_items = !s.highlight_new_items,
            Toggle::TileGrid => s.tile_grid = !s.tile_grid,
            Toggle::GamepadInstructions => {
                s.gamepad_instructions = !s.gamepad_instructions
            }
            Toggle::FrameSkip => s.frame_skip = (s.frame_skip + 1) % 3,
            Toggle::LightMode => s.light_mode = (s.light_mode + 1) % 4,
            Toggle::Quality => s.quality = (s.quality + 1) % 4,
            Toggle::Background => s.background = !s.background,
            Toggle::Blood => s.blood = !s.blood,
            Toggle::MinersWobble => s.miners_wobble = !s.miners_wobble,
            Toggle::TilesSway => s.tiles_sway = !s.tiles_sway,
            // `Main.GameZoomTarget` steps 0.02 per key press; the row snaps to 25 %.
            Toggle::Zoom => {
                let steps = [1.0f32, 1.25, 1.5, 1.75, 2.0];
                let next = steps
                    .iter()
                    .position(|z| *z > s.zoom + 0.001)
                    .unwrap_or(0);
                s.zoom = steps[next];
            }
            // The menuMode 1125 rows (`Main.cs:50269-50308`).
            // `Main.cSmartCursorModeIsToggleAndNotHold = !...` (`:50273-50275`).
            Toggle::SmartCursorToggle => s.smart_cursor_toggle = !s.smart_cursor_toggle,
            // `Player.SmartCursorSettings.SmartAxeAfterPickaxe = !...`
            // (`Main.cs:50282-50284`).
            Toggle::SmartAxeFirst => s.smart_axe_first = !s.smart_axe_first,
            // `Player.SmartCursorSettings.SmartBlocksEnabled = !...`
            // (`Main.cs:50287-50289`).
            Toggle::SmartBlocks => s.smart_blocks = !s.smart_blocks,
            // `LockOnHelper.CycleUseModes()` (`Main.cs:50301-50303`): the
            // vanilla quirk — ThreeDS cycles back to TargetClosest, not
            // FocusTarget (`LockOnHelper.cs:103-116`).
            Toggle::LockOnPriority => {
                s.lock_on_priority = match s.lock_on_priority {
                    0 => 1, // FocusTarget -> TargetClosest
                    1 => 2, // TargetClosest -> ThreeDS
                    _ => 1, // ThreeDS -> TargetClosest
                };
            }
            // The menuMode 2008 rows (`Main.cs:50113-50162`).
            // `Main.UseStormEffects = !Main.UseStormEffects` (`:50117`).
            Toggle::UseStormEffects => s.use_storm_effects = !s.use_storm_effects,
            // `Main.ForegroundSunlightEffects = !...` (`:50124`).
            Toggle::ForegroundSunlight => s.foreground_sunlight = !s.foreground_sunlight,
            // `Main.UseHeatDistortion = !...` (`:50131`).
            Toggle::UseHeatDistortion => s.use_heat_distortion = !s.use_heat_distortion,
            // `Main.WaveQuality = (Main.WaveQuality + 1) % 4` (`:50144`).
            Toggle::WaveQuality => s.wave_quality = (s.wave_quality + 1) % 4,
            // `Main.UseScreenShake = !...` (`:50154`).
            Toggle::UseScreenShake => s.use_screen_shake = !s.use_screen_shake,
            // `Main.FlashyEffectsWorld = !...` (`:50161`).
            Toggle::FlashyEffectsWorld => s.flashy_effects_world = !s.flashy_effects_world,
            // The menuMode 1127 rows (`Main.cs:50647-50687`).
            // `Main.ReversedUpDownArmorSetBonuses = !...` (`:50652-50654`).
            Toggle::ReversedArmorBonuses => s.reversed_armor_bonuses = !s.reversed_armor_bonuses,
            // The three-state quick-trash cycle (`:50665-50684`): Left Control
            // -> Left Shift -> Disabled -> Left Control.
            Toggle::QuickTrash => {
                if s.quick_trash {
                    s.quick_trash = false;
                    s.disable_left_shift_trash = true;
                } else if s.disable_left_shift_trash {
                    s.disable_left_shift_trash = false;
                } else {
                    s.quick_trash = true;
                    s.disable_left_shift_trash = false;
                }
            }
            // The menuMode 1112 selector rows (`Main.cs:49941-49979`).
            // `CycleSelection` activates the option *before* the current one,
            // wrapping the first to the last (`SelectionHolder.cs:44-58`); the
            // stored key feeds `ui::minimap::style_index` live.
            Toggle::MinimapFrame => {
                s.minimap_frame = cycle_previous(&MINIMAP_FRAME_KEYS, &s.minimap_frame)
            }
            // `CycleResourceSet` — the same backwards walk
            // (`PlayerResourceSetsManager.cs:84-101`).
            Toggle::ResourceSets => {
                s.resource_sets = cycle_previous(&RESOURCE_SET_KEYS, &s.resource_sets)
            }
            // `Main.CycleNPCPortraitMode()` (`Main.cs:40297-40315`): Detailed
            // -> CloseUp -> FullBodyRetro -> Disabled -> Detailed.
            Toggle::PortraitPreference => s.portrait_preference = (s.portrait_preference + 1) % 4,
            // `BigProgressBarSystem.ToggleShowText()`
            // (`BigProgressBarSystem.cs:239-242`). Stored only — consumed by
            // the boss bar in P4.
            Toggle::BossBarText => s.boss_bar_text = !s.boss_bar_text,
            // `Main.FlashyEffectsInterface = !...` (`:49977-49978`).
            Toggle::FlashyEffectsInterface => {
                s.flashy_effects_interface = !s.flashy_effects_interface
            }
        }
        if self.persist {
            let _ = s.save();
        }
    }

    pub fn key(&mut self, code: winit::keyboard::KeyCode) -> Action {
        use winit::keyboard::KeyCode as K;
        // The menuMode 27/272727 capture (`Main.cs:50569-50576`,
        // `:50464-50471`): while `setKey` is open, the first pressed key is
        // stored under its name and the capture closes. Navigation keys are
        // captured too — vanilla's `GetPressedKeys` sees them first (only
        // XNA's `Keys.None` is skipped at `:50575`/`:50470`, which has no
        // winit equivalent).
        if self.set_key.is_some() && matches!(self.page, Page::Keybindings | Page::MapKeys) {
            let index = self.set_key.unwrap();
            let name = format!("{code:?}");
            if self.page == Page::Keybindings {
                if let Some(slot) = self.keybinds.get_mut(index) {
                    *slot = name;
                }
            } else if let Some(slot) = self.map_binds.get_mut(index) {
                *slot = name;
            }
            self.set_key = None;
            return Action::None;
        }
        let count = self.items().len();
        match code {
            K::ArrowUp | K::KeyW => {
                if count > 0 {
                    self.selected = (self.selected + count - 1) % count;
                    self.focus_keyboard = true;
                }
                return Action::None;
            }
            K::ArrowDown | K::KeyS => {
                if count > 0 {
                    self.selected = (self.selected + 1) % count;
                    self.focus_keyboard = true;
                }
                return Action::None;
            }
            K::Tab => {
                if !self.fields.is_empty() && !self.connecting {
                    self.selected = 0;
                }
                return Action::None;
            }
            K::Backspace => {
                if matches!(self.page, Page::Host | Page::Join) && !self.connecting {
                    if let Some(f) = self.fields.get_mut(0) {
                        f.value.pop();
                    }
                }
                return Action::None;
            }
            K::Escape => {
                if let Some(parent) = Self::parent(self.page) {
                    self.page = parent;
                    self.selected = 0;
                    self.fields.clear();
                    self.hint = None;
                    self.bar_grab = None;
                }
                return Action::None;
            }
            K::Enter | K::NumpadEnter => {}
            _ => return Action::None,
        }
        self.activate(self.selected)
    }

    /// Text typed into the host/join fields (the original uses `UIInput`).
    pub fn character(&mut self, c: char) {
        if !matches!(self.page, Page::Host | Page::Join) || self.connecting {
            return;
        }
        if self.selected != 0 {
            return;
        }
        let field = if self.fields.len() > 1 { 1 } else { 0 };
        if let Some(f) = self.fields.get_mut(field) {
            if f.digits && !c.is_ascii_digit() {
                return;
            }
            if f.value.len() < 64 {
                f.value.push(c);
            }
        }
    }

    /// Hit-tests the boxes recorded by the last [`Menu::draw`]. The position is
    /// also what the value bars read at draw time (`Main.mouseX/mouseY`).
    pub fn update_hover(&mut self, pos: Vec2) {
        self.mouse = pos;
        self.hover = self.boxes.iter().position(|b| {
            pos.x >= b[0] && pos.x < b[0] + b[2] && pos.y >= b[1] && pos.y < b[1] + b[3]
        });
        // Heading rows are drawn but not clickable: the C# hit-test answers
        // `focusMenu = -1` for them (`Main.cs:51790-51796`), so a pointer over
        // a title neither highlights it nor moves the cursor.
        if let Some(i) = self.hover {
            if self.items().get(i).is_some_and(|item| item.header) {
                self.hover = None;
            } else {
                self.selected = i;
                self.focus_keyboard = true;
            }
        }
    }

    /// Left click on the hovered row.
    pub fn click(&mut self) -> Action {
        match self.hover {
            Some(i) => self.activate(i),
            None => Action::None,
        }
    }

    fn step_animation(&mut self, dt: f32) {
        let f = (dt * 60.0).clamp(0.0, 3.0);
        self.time += dt;

        // `Main.DrawMenu`: the logo scale eases between 1.0 and 1.35 and the rotation
        // between ±0.08 rad; both speeds ramp by ±1 per frame up to ±50 / ±20.
        if self.logo_scale < 0.98 {
            self.logo_scale *= 1.05;
        }
        self.logo_scale += self.logo_speed * 9.0e-6 * f;
        if self.logo_scale > 1.35 {
            self.logo_dir = -1.0;
        } else if self.logo_scale < 1.0 {
            self.logo_dir = 1.0;
        }
        if self.logo_speed < 50.0 && self.logo_dir == 1.0 {
            self.logo_speed += f;
        } else if self.logo_speed > -50.0 && self.logo_dir == -1.0 {
            self.logo_speed -= f;
        }
        self.logo_rot += self.rot_speed * 4.0e-6 * f;
        if self.logo_rot > 0.08 {
            self.rot_dir = -1.0;
        } else if self.logo_rot < -0.08 {
            self.rot_dir = 1.0;
        }
        if self.rot_speed < 20.0 && self.rot_dir == 1.0 {
            self.rot_speed += f;
        } else if self.rot_speed > -20.0 && self.rot_dir == -1.0 {
            self.rot_speed -= f;
        }
    }

    /// Draws the menu over the paused world.
    pub fn draw(
        &mut self,
        r: &mut SpriteRenderer,
        lookups: &assets::Lookups,
        screen: Vec2,
        dt: f32,
    ) {
        self.step_animation(dt);
        // `Main.PendingResolutionWidth = Main.screenWidth; ...` (`Main.cs
        // :66399-66400`): the menuMode 111 row reads the live window size.
        self.screen_size = screen;

        // `if (!flag) { this.grabColorSlider = 0; Main.hBar = -1f; Main.sBar
        // = -1f; Main.lBar = -1f; Main.aBar = -1f; }` (`Main.cs:51104-51109`):
        // the colour screens reset their slider state whenever DrawMenu runs
        // anywhere else, so re-entering 25/252 reseeds from `selColor`. (The
        // character-creation pickers 17-24 share the block and set the same
        // `flag`; they land with P6.)
        if !matches!(self.page, Page::CursorColor | Page::BorderColor) {
            self.color_grab = 0;
            self.hsl_bars = sliders::HslBars { h: -1.0, s: -1.0, l: -1.0, a: -1.0 };
        }

        // `TextureAssets.Logo` at (screenWidth / 2, 100), centre origin.
        if let Some((uv, w, h)) = lookups.logo {
            let scale = self.logo_scale;
            let (dw, dh) = (w * scale, h * scale);
            let pos = [(screen.x - dw) * 0.5, 100.0 - dh * 0.5];
            r.push_ui_sprite_rot(pos, [dw, dh], uv, [1.0; 4], self.logo_rot);
        }

        let items = self.items();
        let layout = self.page.layout();
        let Some(font) = lookups.death_font.as_ref() else {
            return;
        };
        let focus = self
            .hover
            .or(if self.focus_keyboard { Some(self.selected) } else { None })
            .filter(|i| *i < items.len());

        // `menuItemScale`: +0.02 per frame toward 1.0 for the focused row, -0.02 back.
        for (i, scale) in self.item_scale.iter_mut().enumerate() {
            let target = if Some(i) == focus { 1.0 } else { 0.8 };
            let step = 0.02 * (dt * 60.0).clamp(0.0, 3.0);
            if *scale < target {
                *scale = (*scale + step).min(target);
            } else if *scale > target {
                *scale = (*scale - step).max(target);
            }
        }

        // menuMode 26 (`Main.cs:51478-51577`) / 28 (`Main.cs:51578-51653`): the
        // value bars draw before the rows, like `DrawMenu`'s `flag2`/`flag3`
        // blocks, and drag in the same pass (Review Focus 1). The colour
        // screens' HSL block (`Main.cs:51102-51284`) sits even earlier in
        // `DrawMenu` and does the same.
        if self.page == Page::Volume {
            self.draw_volume_chrome(r, lookups, font, screen);
        } else if self.page == Page::Parallax {
            self.draw_parallax_chrome(r, lookups, font, screen);
        } else if matches!(self.page, Page::CursorColor | Page::BorderColor) {
            self.draw_color_screen_chrome(r, lookups, screen);
        }

        self.boxes.clear();
        let cx = screen.x * 0.5;
        for (i, item) in items.iter().enumerate() {
            let measured = crate::font_width(font, &item.label, 1.0);
            let origin = [measured * 0.5, font.line_spacing * 0.5];
            let scale_now = self.item_scale[i] * item.scale;
            let row_y = layout.y0 + layout.row * i as f32 + item.y;
            // `origin.Y * array7` keeps the row centred while `menuItemScale` eases.
            // The `array8` rows draw from the left edge `num3 + array5` with origin
            // `(0, h/2)` instead (`Main.cs:51773`).
            let top = if item.left_aligned {
                [cx + item.x, row_y + origin[1] * item.scale * (1.0 - self.item_scale[i])]
            } else {
                [
                    cx + item.x - origin[0] * scale_now,
                    row_y + origin[1] * item.scale * (1.0 - self.item_scale[i]),
                ]
            };
            // The `array8` hit test runs `num3 + array5` ..
            // `num3 + array5 + measured * array7` (`Main.cs:51797-51806`).
            self.boxes.push(if item.left_aligned {
                [cx + item.x, row_y, measured * item.scale, 50.0 * item.scale]
            } else {
                [
                    cx - origin[0] * item.scale + item.x,
                    row_y,
                    measured * item.scale,
                    50.0 * item.scale,
                ]
            });

            // `num101 = 255 * (menuItemScale * 2 - 1)`.
            let alpha = (255.0 * (self.item_scale[i] * 2.0 - 1.0)).clamp(0.0, 255.0);
            let dim = (255.0 - alpha) / 255.0;
            let mut main = [1.0f32, 1.0, 1.0];
            if Some(i) == focus {
                let t = alpha / 255.0;
                main[0] = main[0] * (1.0 - t) + t;
                main[1] = main[1] * (1.0 - t) + (215.0 / 255.0) * t;
                main[2] *= 1.0 - t;
            }
            let tint = [
                (main[0] - dim).max(0.0),
                (main[1] - dim).max(0.0),
                (main[2] - dim).max(0.0),
                alpha / 255.0,
            ];
            // Four black passes at ±2 px, then the coloured pass.
            let shadow = [0.0, 0.0, 0.0, alpha / 255.0];
            for (dx, dy) in [(-2.0, 0.0), (2.0, 0.0), (0.0, -2.0), (0.0, 2.0)] {
                crate::push_font_text(
                    r,
                    font,
                    top[0] + dx,
                    top[1] + dy,
                    scale_now,
                    shadow,
                    &item.label,
                );
            }
            crate::push_font_text(r, font, top[0], top[1], scale_now, tint, &item.label);
        }

        // Host/join text fields (the original uses `UIInput` with `Mouse_Text`).
        if matches!(self.page, Page::Host | Page::Join) {
            let start = cx - 230.0;
            let mut y = layout.y0 + layout.row * items.len() as f32 + 10.0;
            for (i, f) in self.fields.iter().enumerate() {
                let active = i == 1 && self.selected == 0 && !self.connecting;
                crate::hud_text(r, lookups, start, y, 0.9, [0.78, 0.82, 0.88, 1.0], f.label);
                let shown = if f.secret {
                    "*".repeat(f.value.chars().count())
                } else {
                    f.value.clone()
                };
                let box_x = start + 140.0;
                r.push_ui_rect(
                    [box_x, y - 6.0],
                    [300.0, 34.0],
                    if active { [0.85, 0.75, 0.3, 0.95] } else { [0.15, 0.16, 0.2, 0.95] },
                );
                crate::hud_text(r, lookups, box_x + 8.0, y, 0.9, [1.0; 4], &shown);
                y += 44.0;
            }
        }

        // `Main.DrawVersionNumber`: bottom-left `Mouse_Text`.
        if let Some(ui) = lookups.ui_font.as_ref() {
            let text = "v1.4.5.8";
            let h = ui.line_spacing;
            crate::hud_text(
                r,
                lookups,
                10.0,
                screen.y - h - 2.0,
                1.0,
                [0.85, 0.85, 0.85, 0.7],
                text,
            );
        }

        // Not-ported notices and connection state (ours, not in the original).
        let mut note_y = screen.y - 64.0;
        if let Some(hint) = &self.hint {
            let w = crate::hud_text_width(r, lookups, hint, 0.8);
            crate::hud_text(
                r,
                lookups,
                (screen.x - w) * 0.5,
                note_y,
                0.8,
                [1.0, 0.8, 0.45, 1.0],
                hint,
            );
            note_y -= 24.0;
        }
        if let Some(err) = &self.error {
            let w = crate::hud_text_width(r, lookups, err, 0.9);
            crate::hud_text(
                r,
                lookups,
                (screen.x - w) * 0.5,
                note_y,
                0.9,
                [1.0, 0.45, 0.4, 1.0],
                err,
            );
            note_y -= 24.0;
        }
        if self.connecting {
            let msg = "Connecting…";
            let w = crate::hud_text_width(r, lookups, msg, 1.0);
            crate::hud_text(
                r,
                lookups,
                (screen.x - w) * 0.5,
                screen.y * 0.5 - 10.0,
                1.0,
                [1.0, 0.9, 0.5, 1.0],
                msg,
            );
        }
        if let Some(status) = &self.status {
            let w = crate::hud_text_width(r, lookups, status, 0.9);
            crate::hud_text(
                r,
                lookups,
                (screen.x - w) * 0.5,
                note_y,
                0.9,
                [0.7, 0.95, 0.7, 1.0],
                status,
            );
        }
    }

    /// The menuMode 26 volume block (`Main.cs:51478-51577`): the six DeathText
    /// labels and the three value bars, drawn before the rows like `DrawMenu`'s
    /// `flag2` block. The drag pass runs after the bar draws, mirroring the C#
    /// order — `DrawValueBar` renders the stored value, then
    /// `Main.musicVolume = num86` stores (`:51535-51543` and the twins) — so
    /// the knob follows the pointer exactly one frame behind, like the
    /// original.
    fn draw_volume_chrome(
        &mut self,
        r: &mut SpriteRenderer,
        lookups: &assets::Lookups,
        font: &assets::UiFont,
        screen: Vec2,
    ) {
        // `float x = Main.screenWidth / 2 - 40 + 37` (`Main.cs:51479`).
        let bar_x = screen.x / 2.0 - 40.0 + 37.0;
        // `int num83 = 370 + Main.screenWidth / 2 - 400 + 37` (`Main.cs:51486`).
        let label_x = 370.0 + screen.x / 2.0 - 400.0 + 37.0;
        // `color` is the backdrop tint (`Main.cs:48253-48254`): the menu world
        // is sunlit, so `Main.tileColor.R` tops out at 255 — the same base the
        // item rows draw their text with.
        let label = color::tint(slider_label_color(255));
        // The six labels at scale 0.5 (`Main.cs:51489-51525`): the three names
        // and their percentages, 90 px right of the names (`:51502`, `:51506`,
        // `:51511`).
        border_string_four_way(r, font, label_x, 350.0, label, "Sound:"); // [98] (`:51490`)
        border_string_four_way(r, font, label_x, 320.0, label, "Music:"); // [99] (`:51494`)
        border_string_four_way(r, font, label_x, 380.0, label, "Ambient:"); // [119] (`:51497`)
        border_string_four_way(
            r,
            font,
            label_x + 90.0,
            320.0,
            label,
            &volume_percent(self.settings.music_volume),
        );
        border_string_four_way(
            r,
            font,
            label_x + 90.0,
            350.0,
            label,
            &volume_percent(self.settings.sound_volume),
        );
        border_string_four_way(
            r,
            font,
            label_x + 90.0,
            380.0,
            label,
            &volume_percent(self.settings.ambient_volume),
        );
        // The bars ride `num80 - 18 + {30, 60, 90}` (`Main.cs:51480`, `:51533`,
        // `:51545`, `:51557`): music, sound, ambient.
        sliders::draw_value_bar(r, lookups, bar_x, 332.0, self.settings.music_volume, self.mouse);
        sliders::draw_value_bar(r, lookups, bar_x, 362.0, self.settings.sound_volume, self.mouse);
        sliders::draw_value_bar(
            r,
            lookups,
            bar_x,
            392.0,
            self.settings.ambient_volume,
            self.mouse,
        );
        self.drag_volume_screen(screen.x);
    }

    /// The menuMode 28 parallax block (`Main.cs:51578-51653`): the five-pass
    /// label and the value bar, drawn before the rows like `DrawMenu`'s
    /// `flag3` block.
    fn draw_parallax_chrome(
        &mut self,
        r: &mut SpriteRenderer,
        lookups: &assets::Lookups,
        font: &assets::UiFont,
        screen: Vec2,
    ) {
        // `int num92 = 370 + Main.screenWidth / 2 - 400` (`Main.cs:51586`).
        let label_x = 370.0 + screen.x / 2.0 - 400.0;
        // `Lang.menu[52].Value + ": " + Main.bgScroll` (`Main.cs:51588`).
        let text = format!("Parallax: {}", self.settings.parallax_scroll);
        let label = color::tint(slider_label_color(255));
        // The manual five-pass shadow (`Main.cs:51589-51631`) is the same
        // four-black-plus-colour pattern as `DrawBorderStringFourWay`.
        border_string_four_way(r, font, label_x, 400.0, label, &text);
        // `IngameOptions.valuePosition = new Vector2(Main.screenWidth / 2 - 40,
        // num89 + 12)` (`Main.cs:51633`); the bar value is
        // `(float)Main.bgScroll / 100f` (`:51635`).
        sliders::draw_value_bar(
            r,
            lookups,
            screen.x / 2.0 - 40.0,
            412.0,
            self.settings.parallax_scroll as f32 / 100.0,
            self.mouse,
        );
        self.drag_parallax_screen(screen.x);
    }

    /// The volume drag (`Main.cs:51527-51572`): `rightLock` sticks while the
    /// button is held and the value follows the mouse (Review Focus 1). Writes
    /// `config.json` once per drag, on release — the port's write-per-edit
    /// rule (`Menu::toggle`), not per frame.
    fn drag_volume_screen(&mut self, screen_w: f32) {
        if self.page != Page::Volume {
            return; // `flag2` (`Main.cs:51478`).
        }
        let mut volumes = [
            self.settings.music_volume,
            self.settings.sound_volume,
            self.settings.ambient_volume,
        ];
        let had_grab = self.bar_grab.is_some();
        volume_drag_pass(screen_w, self.mouse, self.mouse_down, &mut self.bar_grab, &mut volumes);
        self.settings.music_volume = volumes[0];
        self.settings.sound_volume = volumes[1];
        self.settings.ambient_volume = volumes[2];
        if had_grab && !self.mouse_down && self.persist {
            let _ = self.settings.save();
        }
    }

    /// The parallax drag (`Main.cs:51598-51652`): the same sticky `rightLock`
    /// pattern, storing `bgScroll = (int)(value * 100f)` (`:51642`).
    fn drag_parallax_screen(&mut self, screen_w: f32) {
        if self.page != Page::Parallax {
            return; // `flag3` (`Main.cs:51578`).
        }
        // `Main.caveParallax = 1f - (float)Main.bgScroll / 500f` (`:50382`,
        // `:51643`) is recomputed where the backdrop reads the scroll.
        let had_grab = self.bar_grab.is_some();
        parallax_drag_pass(
            screen_w,
            self.mouse,
            self.mouse_down,
            &mut self.bar_grab,
            &mut self.settings.parallax_scroll,
        );
        if had_grab && !self.mouse_down && self.persist {
            let _ = self.settings.save();
        }
    }

    /// The menuMode 25/252 HSL block (`Main.cs:51102-51284`), drawn before the
    /// rows like the volume chrome. One frame runs in the original order: the
    /// menuMode branch stores `selColor` into the page's colour
    /// (`Main.cs:50324` / `:50346`), the `-1f` bars seed from
    /// `rgbToHsl(selColor)` (`:51118-51131`), [`sliders::draw_color_sliders`]
    /// draws and drags in one pass (Review Focus 1), and `selColor` recomposes
    /// from the bars (`:51279-51284`) — so the stored colour trails the knobs by
    /// one frame, like the original.
    fn draw_color_screen_chrome(
        &mut self,
        r: &mut SpriteRenderer,
        lookups: &assets::Lookups,
        screen: Vec2,
    ) {
        let alpha_bar = self.page == Page::BorderColor;
        // The branch copy (`Main.cs:50324` / `:50346`).
        if alpha_bar {
            self.settings.mouse_border_color = self.sel_color;
        } else {
            self.settings.mouse_color = self.sel_color;
        }
        // The seeding (`Main.cs:51118-51131`).
        seed_hsl_bars(self.sel_color, &mut self.hsl_bars);
        // `num59 = 0.15f` (`Main.cs:51238-51239`), `0f` for 252 (`:51240-51241`).
        let lum_floor = if alpha_bar { 0.0 } else { 0.15 };
        let had_grab = self.color_grab != 0;
        sliders::draw_color_sliders(
            r,
            lookups,
            screen.x,
            320.0, // `num9` (`Main.cs:50319` / `:50341`)
            &mut self.hsl_bars,
            lum_floor,
            alpha_bar,
            self.mouse,
            self.mouse_down,
            &mut self.color_grab,
        );
        // `Main.selColor = Main.hslToRgb(Main.hBar, Main.sBar, Main.lBar)`
        // (`Main.cs:51279-51284`).
        self.sel_color = recompose_sel_color(alpha_bar, &self.hsl_bars);
        // One config write per drag, on release — the Task 2 write-per-edit
        // rule, not per frame.
        if had_grab && !self.mouse_down && self.persist {
            let _ = self.settings.save();
        }
    }
}

/// The `-1f` seeding of the HSL bars (`Main.cs:51118-51131`): on the first
/// slider frame the four bars take `rgbToHsl(selColor)` — the `Vector3` fields
/// stored into the `float` bars — and `selColor.A / 255f`. Any bar still at
/// `-1f` reseeds all four; otherwise the stored bars win (`:51133-51137`).
fn seed_hsl_bars(sel_color: [u8; 4], bars: &mut sliders::HslBars) {
    if bars.h == -1.0 || bars.s == -1.0 || bars.l == -1.0 || bars.a == -1.0 {
        let (h, s, l) = color::rgb_to_hsl(sel_color);
        *bars = sliders::HslBars {
            h: h as f32,
            s: s as f32,
            l: l as f32,
            // `float num57 = (float)(int)Main.selColor.A / 255f` (`Main.cs:51122`).
            a: sel_color[3] as f32 / 255.0,
        };
    }
}

/// `Main.selColor = Main.hslToRgb(Main.hBar, Main.sBar, Main.lBar)`
/// (`Main.cs:51279-51282`); the menuMode 252 twin also stores the alpha bar,
/// `selColor.A = (byte)(Main.aBar * 255f)` (`:51282-51284` — truncation).
fn recompose_sel_color(alpha_bar: bool, bars: &sliders::HslBars) -> [u8; 4] {
    let mut c = color::hsl_to_rgb(bars.h as f64, bars.s as f64, bars.l as f64, 255);
    if alpha_bar {
        c[3] = (bars.a * 255.0f32) as u8;
    }
    c
}

/// XNA `Rectangle.Contains(Point)` (`IngameOptions.cs:1557`): inclusive
/// left/top, exclusive right/bottom.
fn rect_contains(rect: [f32; 4], p: Vec2) -> bool {
    p.x >= rect[0] && p.x < rect[0] + rect[2] && p.y >= rect[1] && p.y < rect[1] + rect[3]
}

/// One `IngameOptions.DrawValueBar` row (`Main.cs:51533-51543` and the
/// sound/ambient twins, `:51635-51646` for parallax) as a pure pass: hover is
/// `inBar || rightLock == id` (`:51536`) and the value follows the mouse while
/// `flag7 && rightLock == id` (`:51540`), the sticky lock keeping the drag
/// alive outside the bar while the button is held (Review Focus 1). `grab`
/// mirrors `rightLock`; the caller clears it when the button is up
/// (`:51529-51531`) and re-sets it from the last hover (`:51570-51572`).
/// Returns `(hovered, Some(stored))` — `stored` only when the value moved.
fn value_bar_drag(
    right_x: f32,
    center_y: f32,
    mouse: Vec2,
    mouse_down: bool,
    grab: Option<u8>,
    id: u8,
) -> (bool, Option<f32>) {
    let (rect, hit) = sliders::value_bar_rects(right_x, center_y);
    // `rectangle.Contains(new Point(Main.mouseX, Main.mouseY))` over the
    // post-`Inflate(-5, 0)` rect (`IngameOptions.cs:1556-1559`).
    let in_bar = rect_contains(hit, mouse);
    let hovered = in_bar || grab == Some(id);
    // `Main.musicVolume = num86` (`Main.cs:51540-51541` and the twins): the
    // ratio over the inflated hit geometry.
    let stored = if hovered && mouse_down && grab == Some(id) {
        Some(sliders::value_bar_value(rect, mouse.x))
    } else {
        None
    };
    (hovered, stored)
}

/// The menuMode 26 drag pass (`Main.cs:51527-51572`): clears the sticky lock on
/// release, lets the last hovered-or-grabbed bar win `rightHover` (music →
/// sound → ambient, `:51570-51572`) and stores the dragged volumes.
fn volume_drag_pass(
    screen_w: f32,
    mouse: Vec2,
    mouse_down: bool,
    grab: &mut Option<u8>,
    volumes: &mut [f32; 3],
) {
    // `if (!Main.mouseLeft) { IngameOptions.rightLock = -1; }` (`Main.cs:51529-51531`).
    if !mouse_down {
        *grab = None;
    }
    // `float x = Main.screenWidth / 2 - 40 + 37` (`Main.cs:51479`).
    let right_x = screen_w / 2.0 - 40.0 + 37.0;
    let mut right_hover: Option<u8> = None;
    // `num80 - 18 + {30, 60, 90}` (`Main.cs:51480`, `:51533`, `:51545`,
    // `:51557`); `rightLock` 3/2/4 are the port's 0/1/2. Later matches
    // overwrite `rightHover`, like the C# sequence.
    for (i, dy, id) in [(0usize, 30.0f32, 0u8), (1, 60.0, 1), (2, 90.0, 2)] {
        let (hovered, stored) =
            value_bar_drag(right_x, 320.0 - 18.0 + dy, mouse, mouse_down, *grab, id);
        if let Some(v) = stored {
            volumes[i] = v;
        }
        if hovered {
            right_hover = Some(id);
        }
    }
    // `if (IngameOptions.rightHover != -1) IngameOptions.rightLock =
    // IngameOptions.rightHover` (`Main.cs:51570-51572`).
    if let Some(h) = right_hover {
        *grab = Some(h);
    }
}

/// The menuMode 28 drag pass (`Main.cs:51598-51652`): the parallax bar reuses
/// the value-bar mechanics, storing `bgScroll = (int)(value * 100f)`
/// (`Main.cs:51642`) into the 0..=100 slider.
fn parallax_drag_pass(
    screen_w: f32,
    mouse: Vec2,
    mouse_down: bool,
    grab: &mut Option<u8>,
    parallax_scroll: &mut u8,
) {
    // `if (!Main.mouseLeft) { IngameOptions.rightLock = -1; }` (`Main.cs:51601-51603`).
    if !mouse_down {
        *grab = None;
    }
    // `IngameOptions.valuePosition = new Vector2(Main.screenWidth / 2 - 40,
    // num89 + 12)` (`Main.cs:51633`); the bar value is `bgScroll / 100`
    // (`:51635`).
    let (hovered, stored) = value_bar_drag(screen_w / 2.0 - 40.0, 412.0, mouse, mouse_down, *grab, 3);
    if let Some(v) = stored {
        // `Main.bgScroll = (int)(num98 * 100f)` (`Main.cs:51642`).
        *parallax_scroll = (v * 100.0) as u8;
    }
    // `if (IngameOptions.rightHover != -1) IngameOptions.rightLock =
    // IngameOptions.rightHover` (`Main.cs:51650-51652`).
    if hovered {
        *grab = Some(3);
    }
}

/// `Utils.DrawBorderStringFourWay` (`Utils.cs:2464-2494`) at the volume block's
/// `scale: 0.5f` (`Main.cs:51525`, `:51628`): four black passes at ±2 px and
/// the coloured centre pass, origin zero. The parallax label inlines the same
/// five passes (`Main.cs:51589-51631`).
fn border_string_four_way(
    r: &mut SpriteRenderer,
    font: &assets::UiFont,
    x: f32,
    y: f32,
    tint: [f32; 4],
    text: &str,
) {
    for (dx, dy) in [(-2.0, 0.0), (2.0, 0.0), (0.0, -2.0), (0.0, 2.0)] {
        crate::push_font_text(r, font, x + dx, y + dy, 0.5, [0.0, 0.0, 0.0, 1.0], text);
    }
    crate::push_font_text(r, font, x, y, 0.5, tint, text);
}

/// `Player.name` style random character names ("Zealot", "Terra", …): two syllables
/// from the game's name lists, so every launch shows a different character.
pub fn random_name() -> String {
    const A: &[&str] = &[
        "Al", "Ai", "Ar", "Bel", "Bra", "Ca", "Cle", "Da", "Dra", "E", "El", "Fa", "Fi", "Ga",
        "Gra", "Ha", "Ig", "Ja", "Ka", "Kri", "Lu", "Ly", "Ma", "Mi", "Na", "Ne", "Ni", "No",
        "O", "Or", "Pa", "Qu", "Ra", "Re", "Ri", "Sa", "Se", "Si", "Sky", "Ta", "Te", "Tha",
        "Ti", "To", "Tri", "Ul", "Va", "Ve", "Vi", "Wa", "Xa", "Za", "Ze",
    ];
    const B: &[&str] = &[
        "bert", "dan", "dor", "dric", "fen", "fred", "gar", "gus", "ius", "lan", "las", "lios",
        "lon", "mar", "mon", "nar", "nath", "ndra", "nel", "nia", "nor", "phin", "ran", "rax",
        "ric", "ril", "ron", "rus", "san", "sin", "ster", "sus", "tia", "tor", "tra", "tus",
        "vin", "vor", "wick", "wyn", "zar",
    ];
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
        ^ ((std::process::id() as u64) << 32);
    let mut s = seed | 1;
    let mut next = || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        s
    };
    let a = A[(next() % A.len() as u64) as usize];
    let b = B[(next() % B.len() as u64) as usize];
    format!("{a}{b}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn menu(page: Page) -> Menu {
        let mut m = Menu::new("Test".into());
        m.page = page;
        m.persist = false;
        m
    }

    #[test]
    fn screens_use_the_original_row_metrics() {
        // `num2`, `num4` and `array7` from `Main.DrawMenu`.
        let main = Page::Main.layout();
        assert_eq!((main.y0, main.row), (220.0, 52.0));
        let settings = Page::Settings.layout();
        assert_eq!((settings.y0, settings.row), (210.0, 37.0));
        let general = Page::General.layout();
        assert_eq!((general.y0, general.row), (250.0, 52.0));
        let interface = Page::Interface.layout();
        assert_eq!((interface.y0, interface.row), (210.0, 28.0));
        let video = Page::Video.layout();
        assert_eq!((video.y0, video.row), (186.0, 30.0));
    }

    #[test]
    fn main_menu_has_the_seven_vanilla_rows() {
        let items = menu(Page::Main).items();
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert_eq!(
            labels,
            [
                "Single Player",
                "Multiplayer",
                "Achievements",
                "Resource Packs",
                "Settings",
                "Credits",
                "Exit"
            ]
        );
        assert!(items.iter().all(|i| i.scale == 1.0));
    }

    #[test]
    fn settings_screen_matches_the_vanilla_rows() {
        let items = menu(Page::Settings).items();
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert_eq!(
            labels,
            ["General", "Interface", "Video", "Volume", "Cursor", "Controls", "Language", "Back"]
        );
        assert!(items.iter().all(|i| i.scale == 0.75));
        assert_eq!(items[7].y, 8.0, "array4[num5 - 1] = 8");
    }

    #[test]
    fn general_rows_are_toggles_with_state_labels() {
        let mut m = menu(Page::General);
        assert_eq!(m.items().len(), 6);
        assert_eq!(m.items()[0].label, "Autosave On");
        m.settings.auto_save = false;
        assert_eq!(m.items()[0].label, "Autosave Off");
        assert_eq!(m.items()[5].y, 18.0, "array4[num5 - 1] = 18");
    }

    #[test]
    fn interface_screen_has_twelve_rows() {
        let m = menu(Page::Interface);
        let items = m.items();
        assert_eq!(items.len(), 12);
        assert!(items[..11].iter().all(|i| i.scale == 0.6));
        assert_eq!(items[11].label, "Back");
        assert_eq!(items[11].y, 16.0);
    }

    #[test]
    fn video_screen_leaves_back_at_scale_one() {
        let m = menu(Page::Video);
        let items = m.items();
        assert_eq!(items.len(), 12);
        assert!(items[..11].iter().all(|i| i.scale == 0.6));
        // The original only sets `array7` for the first ten rows; our Zoom row
        // shares their 0.6 scale, so only Back keeps 1.0.
        assert_eq!(items[11].label, "Back");
        assert_eq!(items[11].scale, 1.0);
        assert_eq!(items[11].y, 8.0);
    }

    #[test]
    fn toggles_cycle_the_vanilla_modes() {
        let mut m = menu(Page::Video);
        m.toggle(Toggle::LightMode);
        assert_eq!(m.settings.light_mode, 1);
        m.toggle(Toggle::LightMode);
        m.toggle(Toggle::LightMode);
        m.toggle(Toggle::LightMode);
        assert_eq!(m.settings.light_mode, 0, "four modes");
        m.toggle(Toggle::FrameSkip);
        m.toggle(Toggle::FrameSkip);
        m.toggle(Toggle::FrameSkip);
        assert_eq!(m.settings.frame_skip, 0, "three modes");
    }

    #[test]
    fn back_navigates_to_the_parent_screen() {
        let mut m = menu(Page::General);
        m.activate(5);
        assert_eq!(m.page, Page::Settings);
        let mut m = menu(Page::Settings);
        m.activate(7);
        assert_eq!(m.page, Page::Main);
        let mut m = menu(Page::Video);
        m.activate(11);
        assert_eq!(m.page, Page::Settings);
    }

    #[test]
    fn unported_screens_show_a_notice_instead_of_navigating() {
        // The 1127 "Keybindings" row opens the menuMode 888
        // `ManageControlsMenu` UIState (`Main.cs:50661-50666`) — the one nav
        // target the port does not have.
        let mut m = menu(Page::Controls);
        m.activate(2);
        assert_eq!(m.page, Page::Controls);
        assert!(m.hint.is_some());
    }

    #[test]
    fn tr_page_numeric_names_resolve_the_p5_screens() {
        // Every new screen is screenshot-able as `TR_PAGE=<menuMode>` (plan
        // Task 1); the word aliases keep working and unknown names resolve to
        // nothing (the page stays on the main menu).
        assert_eq!(Menu::page_from_name("26"), Some(Page::Volume));
        assert_eq!(Menu::page_from_name("28"), Some(Page::Parallax));
        assert_eq!(Menu::page_from_name("1125"), Some(Page::Cursor));
        assert_eq!(Menu::page_from_name("25"), Some(Page::CursorColor));
        assert_eq!(Menu::page_from_name("252"), Some(Page::BorderColor));
        assert_eq!(Menu::page_from_name("111"), Some(Page::Resolution));
        assert_eq!(Menu::page_from_name("2008"), Some(Page::Effects));
        assert_eq!(Menu::page_from_name("1213"), Some(Page::Language));
        assert_eq!(Menu::page_from_name("1212"), Some(Page::LanguageInitial));
        assert_eq!(Menu::page_from_name("1127"), Some(Page::Controls));
        assert_eq!(Menu::page_from_name("27"), Some(Page::Keybindings));
        assert_eq!(Menu::page_from_name("272727"), Some(Page::MapKeys));
        assert_eq!(Menu::page_from_name("12"), Some(Page::Multiplayer));
        assert_eq!(Menu::page_from_name("13"), Some(Page::JoinIp));
        assert_eq!(Menu::page_from_name("131"), Some(Page::JoinPort));
        assert_eq!(Menu::page_from_name("30"), Some(Page::HostPassword));
        assert_eq!(Menu::page_from_name("31"), Some(Page::JoinPassword));
        assert_eq!(Menu::page_from_name("889"), Some(Page::ServerSettings));
        assert_eq!(Menu::page_from_name("Settings"), Some(Page::Settings));
        assert_eq!(Menu::page_from_name("VIDEO"), Some(Page::Video));
        assert_eq!(Menu::page_from_name("nonsense"), None);
    }

    #[test]
    fn p5_pages_register_layouts_parents_and_stay_row_less() {
        // The layout constants come from the Task 2-6 tables; the pages stay
        // row-less until their tasks fill `items()` in.
        let volume = Page::Volume.layout();
        assert_eq!((volume.y0, volume.row, volume.scale), (200.0, 10.0, 1.0));
        let parallax = Page::Parallax.layout();
        assert_eq!((parallax.y0, parallax.row, parallax.scale), (240.0, 60.0, 1.0));
        let cursor = Page::Cursor.layout();
        assert_eq!((cursor.y0, cursor.row, cursor.scale), (232.0, 38.0, 0.73));
        let color = Page::CursorColor.layout();
        assert_eq!((color.y0, color.row, color.scale), (200.0, 10.0, 1.0));
        let resolution = Page::Resolution.layout();
        assert_eq!((resolution.y0, resolution.row, resolution.scale), (210.0, 55.0, 0.85));
        let effects = Page::Effects.layout();
        assert_eq!((effects.y0, effects.row, effects.scale), (210.0, 28.0, 0.6));
        let language = Page::Language.layout();
        assert_eq!((language.y0, language.row, language.scale), (200.0, 26.0, 0.65));
        let initial = Page::LanguageInitial.layout();
        assert_eq!((initial.y0, initial.row, initial.scale), (200.0, 30.0, 0.65));
        let controls = Page::Controls.layout();
        assert_eq!((controls.y0, controls.row, controls.scale), (250.0, 52.0, 0.78));
        let keybindings = Page::Keybindings.layout();
        assert_eq!((keybindings.y0, keybindings.row, keybindings.scale), (176.0, 22.0, 0.45));
        let map_keys = Page::MapKeys.layout();
        assert_eq!((map_keys.y0, map_keys.row, map_keys.scale), (200.0, 30.0, 0.55));
        let join_ip = Page::JoinIp.layout();
        assert_eq!((join_ip.y0, join_ip.row, join_ip.scale), (180.0, 30.0, 1.0));
        let join_port = Page::JoinPort.layout();
        assert_eq!((join_port.y0, join_port.row, join_port.scale), (250.0, 80.0, 1.0));
        let server = Page::ServerSettings.layout();
        assert_eq!((server.y0, server.row, server.scale), (200.0, 60.0, 1.0));

        // Resolution/Effects/Language/LanguageInitial grew their rows with
        // Task 4 and Controls/Keybindings/MapKeys with Task 5; the rest stay
        // row-less until Task 6 fills `items()` in.
        for page in [
            Page::JoinIp,
            Page::JoinPort,
            Page::HostPassword,
            Page::JoinPassword,
            Page::ServerSettings,
        ] {
            assert!(menu(page).items().is_empty(), "{page:?} has no rows yet");
        }
        // The Back rows of the Task 2-6 tables.
        assert_eq!(Menu::parent(Page::Volume), Some(Page::Settings));
        assert_eq!(Menu::parent(Page::Parallax), Some(Page::Video));
        assert_eq!(Menu::parent(Page::Cursor), Some(Page::Settings));
        assert_eq!(Menu::parent(Page::CursorColor), Some(Page::Cursor));
        assert_eq!(Menu::parent(Page::BorderColor), Some(Page::Cursor));
        assert_eq!(Menu::parent(Page::Resolution), Some(Page::Video));
        assert_eq!(Menu::parent(Page::Effects), Some(Page::Video));
        assert_eq!(Menu::parent(Page::Language), Some(Page::Settings));
        assert_eq!(Menu::parent(Page::LanguageInitial), None);
        assert_eq!(Menu::parent(Page::Controls), Some(Page::Settings));
        assert_eq!(Menu::parent(Page::Keybindings), Some(Page::Settings));
        assert_eq!(Menu::parent(Page::MapKeys), Some(Page::Settings));
        assert_eq!(Menu::parent(Page::JoinIp), Some(Page::Multiplayer));
        assert_eq!(Menu::parent(Page::JoinPort), Some(Page::JoinIp));
        assert_eq!(Menu::parent(Page::HostPassword), Some(Page::Multiplayer));
        assert_eq!(Menu::parent(Page::JoinPassword), Some(Page::JoinPort));
        assert_eq!(Menu::parent(Page::ServerSettings), Some(Page::Multiplayer));
    }

    #[test]
    fn volume_screen_matches_the_vanilla_rows() {
        // `Main.cs:50362-50380`: `array9 = ["", Lang.menu[65], Lang.menu[5]]`,
        // `array4 = [0, 10, 250]`, `array[1] = true`.
        let items = menu(Page::Volume).items();
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert_eq!(labels, ["", "Volume", "Back"]);
        let ys: Vec<f32> = items.iter().map(|i| i.y).collect();
        assert_eq!(ys, [0.0, 10.0, 250.0]);
        assert!(items[1].header, "array[1] = true");
        assert!(!items[0].header && !items[2].header);
        // Back opens 11 (`Main.cs:50375-50379`).
        let mut m = menu(Page::Volume);
        m.activate(2);
        assert_eq!(m.page, Page::Settings);
    }

    #[test]
    fn parallax_screen_matches_the_vanilla_rows() {
        // `Main.cs:50381-50400`: `array9 = ["", Lang.menu[52], Lang.menu[5]]`,
        // `array4 = [0, 10, 170]`, `array[1] = true`.
        let items = menu(Page::Parallax).items();
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert_eq!(labels, ["", "Parallax", "Back"]);
        let ys: Vec<f32> = items.iter().map(|i| i.y).collect();
        assert_eq!(ys, [0.0, 10.0, 170.0]);
        assert!(items[1].header, "array[1] = true");
        // Back opens 1111 (`Main.cs:50395-50399`).
        let mut m = menu(Page::Parallax);
        m.activate(2);
        assert_eq!(m.page, Page::Video);
    }

    #[test]
    fn volume_and_parallax_rows_navigate_from_their_parents() {
        // The Settings "Volume" row and the Video "Parallax" row open 26 and 28
        // (`Main.cs:50375-50379`, `:49996-50002`).
        let mut m = menu(Page::Settings);
        m.activate(3);
        assert_eq!(m.page, Page::Volume);
        assert!(m.hint.is_none());
        let mut m = menu(Page::Video);
        m.activate(2);
        assert_eq!(m.page, Page::Parallax);
        assert!(m.hint.is_none());
    }

    #[test]
    fn volume_percent_rounds_ties_to_even() {
        // `Math.Round(0.125f * 100f)` = `Math.Round(12.5)` = 12 — ties to even,
        // so half-up would wrongly print "13%" (Review Focus 2).
        assert_eq!(volume_percent(0.125), "12%");
        assert_eq!(volume_percent(0.75), "75%");
        assert_eq!(volume_percent(1.0), "100%");
        assert_eq!(volume_percent(0.0), "0%");
        // 0.135 is not a tie in f32 (0.135 * 100 = 13.5000005): it rounds up.
        assert_eq!(volume_percent(0.135), "14%");
    }

    #[test]
    fn slider_label_color_repeats_the_red_channel() {
        assert_eq!(slider_label_color(255), [255, 255, 255, 255]);
        // b = (255 + 2*85)/3 = 141, r = (255 + 141)/2 = 198; the final
        // `new Color((byte)num85, (byte)num85, (byte)num85, (byte)num84)`
        // repeats `color10.R` for G and B (`Main.cs:51519-51525`) — the G/B
        // twins at `:51516-51518` are dead stores.
        assert_eq!(slider_label_color(85), [198, 198, 198, 255]);
    }

    #[test]
    fn headings_never_take_the_hover_focus() {
        // The C# hit-test answers `focusMenu = -1` for the non-clickable rows
        // (`if (array[num99] || array2[num99])`, `Main.cs:51790-51796`), so a
        // pointer over a title neither highlights it nor moves the cursor.
        let mut m = menu(Page::Parallax);
        m.selected = 0;
        // Boxes in `items()` order: row 0 is the empty label, row 1 the
        // "Parallax" title (`heading()`), row 2 "Back".
        m.boxes.push([0.0, 240.0, 1600.0, 30.0]);
        m.boxes.push([0.0, 300.0, 1600.0, 30.0]);
        m.boxes.push([0.0, 360.0, 1600.0, 30.0]);
        m.update_hover(Vec2::new(500.0, 310.0));
        assert!(m.hover.is_none());
        assert_eq!(m.selected, 0);
        m.update_hover(Vec2::new(500.0, 370.0));
        assert_eq!(m.hover, Some(2));
        assert_eq!(m.selected, 2);
    }

    #[test]
    fn volume_drag_sticks_and_releases() {
        let mut m = menu(Page::Volume);
        // The music bar: right_x = 800 - 40 + 37 = 797 (`Main.cs:51479`), centre
        // y 332 (`:51533`) — rect [619, 324, 178, 16], hit [624, 324, 168, 16].
        // Press while hovering: the first frame takes the lock from the hover
        // (`Main.cs:51536-51539`, `:51570-51572`); the store waits for the lock
        // it checks (`:51540`).
        m.mouse_down = true;
        m.mouse = Vec2::new(708.0, 332.0); // 84 px into the hit box: value 0.5
        m.drag_volume_screen(1600.0);
        assert_eq!(m.bar_grab, Some(0));
        assert!((m.settings.music_volume - 0.75).abs() < 1e-6);
        m.drag_volume_screen(1600.0);
        assert!((m.settings.music_volume - 0.5).abs() < 1e-6);
        // Sticky: dragging out past the bar's right edge keeps following — 797
        // reads 1.0 through the positional clamp (`IngameOptions.cs:1575-1579`).
        m.mouse = Vec2::new(797.0, 332.0);
        m.drag_volume_screen(1600.0);
        assert_eq!(m.settings.music_volume, 1.0);
        // Releasing clears the lock (`Main.cs:51529-51531`) — the pointer sits
        // outside the hit box here (797 > 624 + 168), so no hover re-takes it.
        m.mouse_down = false;
        m.drag_volume_screen(1600.0);
        assert_eq!(m.bar_grab, None);
        // The next press in the sound bar grabs the second id.
        m.mouse = Vec2::new(708.0, 362.0);
        m.mouse_down = true;
        m.drag_volume_screen(1600.0);
        assert_eq!(m.bar_grab, Some(1));
        m.drag_volume_screen(1600.0);
        assert!((m.settings.sound_volume - 0.5).abs() < 1e-6);
        // The ambient bar stayed put throughout.
        assert!((m.settings.ambient_volume - 0.75).abs() < 1e-6);
    }

    #[test]
    fn parallax_drag_maps_value_to_the_scroll() {
        // `(int)(num98 * 100f)` (`Main.cs:51642`): a 0.37 bar value stores
        // bgScroll 37; the backdrop then scrolls by `cave_parallax(37) = 0.926`
        // (the mapping is pinned against `main.rs`'s own test).
        assert_eq!((0.37f32 * 100.0) as u8, 37);
        let mut m = menu(Page::Parallax);
        // Mid-bar press: right_x = 800 - 40 = 760 (`Main.cs:51633`), centre y
        // 412 — the hit box starts 5 px in, so x = 587 + 84 reads 0.5.
        m.mouse_down = true;
        m.mouse = Vec2::new(671.0, 412.0);
        m.drag_parallax_screen(1600.0);
        assert_eq!(m.bar_grab, Some(3));
        m.drag_parallax_screen(1600.0);
        assert_eq!(m.settings.parallax_scroll, 50);
        // The bar shows `bgScroll / 100` back (`Main.cs:51635`).
        assert!((m.settings.parallax_scroll as f32 / 100.0 - 0.5).abs() < 1e-6);
        // Releasing the button does not drop the lock while the pointer still
        // rides the bar — the hover re-takes `rightLock` every frame
        // (`Main.cs:51650-51652`); it clears once the pointer leaves
        // (`:51601-51603`).
        m.mouse_down = false;
        m.drag_parallax_screen(1600.0);
        assert_eq!(m.bar_grab, Some(3), "hover keeps the lock");
        m.mouse = Vec2::new(100.0, 100.0);
        m.drag_parallax_screen(1600.0);
        assert_eq!(m.bar_grab, None);
    }

    #[test]
    fn cursor_screen_matches_the_vanilla_rows() {
        // `Main.cs:50242-50317`: `num5 = 7`, `array4[6] = 18`, every `array7`
        // 0.73. The defaults column: a fresh `Menu` shows Toggle [121] (the
        // `:67178` default) and Disabled [216] (the `Player.cs:331` default).
        // The lock-on row reads "Clearest Line" [234], not the plan's Focus
        // Target: `LoadSettings` clamps the keyless -1 to 0 and remaps
        // FocusTarget to ThreeDS (`Main.cs:4496-4505`), so a fresh boot never
        // sits on FocusTarget.
        let items = menu(Page::Cursor).items();
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert_eq!(
            labels,
            [
                "Cursor Color",
                "Border Color",
                "Smart Cursor Mode: Toggle",
                "Smart Cursor Priority: Pickaxe → Axe",
                "Smart Block Placement: Disabled",
                "Lock On Priority: Clearest Line",
                "Back",
            ]
        );
        let ys: Vec<f32> = items.iter().map(|i| i.y).collect();
        assert_eq!(ys, [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 18.0]);
        assert!(items.iter().all(|i| i.scale == 0.73));
        assert!(!items[0].header && !items[6].header);
        // The toggled twins (`Main.cs:50271-50293`): `? 122 : 121` /
        // `? 214 : 213` / `? 215 : 216`.
        let mut m = menu(Page::Cursor);
        m.settings.smart_cursor_toggle = false;
        m.settings.smart_axe_first = true;
        m.settings.smart_blocks = true;
        m.settings.lock_on_priority = 1;
        let labels: Vec<String> = m.items().iter().map(|i| i.label.clone()).collect();
        assert_eq!(labels[2], "Smart Cursor Mode: Hold");
        assert_eq!(labels[3], "Smart Cursor Priority: Axe → Pickaxe");
        assert_eq!(labels[4], "Smart Block Placement: To Cursor");
        assert_eq!(labels[5], "Lock On Priority: Target Closest");
    }

    #[test]
    fn colour_screens_match_the_vanilla_rows() {
        // `Main.cs:50318-50339` / `:50340-50361`: `num5 = 3`,
        // `array9 = ["", Lang.menu[64]/[217], Lang.menu[5]]`,
        // `array4 = [0, 10, 250]`, `array[1] = true`; Back opens 1125.
        for (page, header) in
            [(Page::CursorColor, "Cursor Color"), (Page::BorderColor, "Border Color")]
        {
            let items = menu(page).items();
            let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
            assert_eq!(labels, ["", header, "Back"]);
            let ys: Vec<f32> = items.iter().map(|i| i.y).collect();
            assert_eq!(ys, [0.0, 10.0, 250.0]);
            assert!(items[1].header, "array[1] = true");
            assert!(!items[0].header && !items[2].header);
        }
        let mut m = menu(Page::CursorColor);
        m.activate(2);
        assert_eq!(m.page, Page::Cursor);
        let mut m = menu(Page::BorderColor);
        m.activate(2);
        assert_eq!(m.page, Page::Cursor);
    }

    #[test]
    fn colour_rows_seed_sel_color_and_navigate() {
        // Rows 0-1 (`Main.cs:50253-50268`): `selColor = mouseColor` → 25,
        // `selColor = MouseBorderColor` → 252; the Settings "Cursor" row opens
        // 1125.
        let mut m = menu(Page::Cursor);
        m.settings.mouse_color = [10, 20, 30, 255];
        m.settings.mouse_border_color = [1, 2, 3, 4];
        m.activate(0);
        assert_eq!(m.page, Page::CursorColor);
        assert_eq!(m.sel_color, [10, 20, 30, 255]);
        let mut m = menu(Page::Cursor);
        m.settings.mouse_color = [10, 20, 30, 255];
        m.settings.mouse_border_color = [1, 2, 3, 4];
        m.activate(1);
        assert_eq!(m.page, Page::BorderColor);
        assert_eq!(m.sel_color, [1, 2, 3, 4]);
        let mut m = menu(Page::Settings);
        m.activate(4);
        assert_eq!(m.page, Page::Cursor);
        assert!(m.hint.is_none());
    }

    #[test]
    fn lock_on_cycles_to_target_closest_not_focus_target() {
        // `LockOnHelper.CycleUseModes` (`LockOnHelper.cs:103-116`): FocusTarget
        // → TargetClosest → ThreeDS → TargetClosest — the vanilla quirk pins
        // the third hop to 1, not 0.
        let mut m = menu(Page::Cursor);
        m.settings.lock_on_priority = 0;
        m.activate(5);
        assert_eq!(m.settings.lock_on_priority, 1);
        m.activate(5);
        assert_eq!(m.settings.lock_on_priority, 2);
        m.activate(5);
        assert_eq!(
            m.settings.lock_on_priority, 1,
            "ThreeDS cycles to TargetClosest, not FocusTarget"
        );
        assert_eq!(m.items()[5].label, "Lock On Priority: Target Closest");
    }

    #[test]
    fn smart_cursor_rows_flip_their_settings() {
        // `Main.cs:50273-50275` / `:50282-50284` / `:50287-50289`.
        let mut m = menu(Page::Cursor);
        m.activate(2);
        assert!(!m.settings.smart_cursor_toggle);
        assert_eq!(m.items()[2].label, "Smart Cursor Mode: Hold");
        m.activate(3);
        assert!(m.settings.smart_axe_first);
        m.activate(4);
        assert!(m.settings.smart_blocks);
    }

    #[test]
    fn hsl_bars_seed_from_sel_color() {
        // `Main.cs:51118-51131`: the `-1f` bars take `rgbToHsl(selColor)` and
        // `selColor.A / 255f` on the first slider frame — asserted through
        // `color.rs`'s own tested `rgb_to_hsl`, not literals.
        let mut bars = sliders::HslBars { h: -1.0, s: -1.0, l: -1.0, a: -1.0 };
        seed_hsl_bars([255, 50, 95, 255], &mut bars);
        let (h, s, l) = color::rgb_to_hsl([255, 50, 95, 255]);
        assert_eq!(bars.h, h as f32);
        assert_eq!(bars.s, s as f32);
        assert_eq!(bars.l, l as f32);
        // Hand-computed anchors for (255, 50, 95): h = 1185/1230 (not the
        // plan's 0.9556), s = 1, l = 305/510.
        assert!((bars.h - 1185.0 / 1230.0).abs() < 1e-6);
        assert!((bars.s - 1.0).abs() < 1e-6);
        assert!((bars.l - 305.0 / 510.0).abs() < 1e-6);
        assert_eq!(bars.a, 1.0);
        // The HSL round trip is byte-exact for the default cursor colour, so
        // the entry-frame recompose leaves it alone.
        assert_eq!(color::hsl_to_rgb(h, s, l, 255), [255, 50, 95, 255]);
        // A border colour seeds the alpha bar from its stored byte
        // (`(float)(int)selColor.A / 255f`).
        let mut bars = sliders::HslBars { h: -1.0, s: -1.0, l: -1.0, a: -1.0 };
        seed_hsl_bars([64, 64, 64, 64], &mut bars);
        assert!((bars.a - 64.0 / 255.0).abs() < 1e-6);
        assert_eq!(bars.h, 0.0, "grey has no hue");
        // Already-seeded bars stay put (`Main.cs:51133-51137` else branch).
        let mut bars = sliders::HslBars { h: 0.25, s: 0.5, l: 0.75, a: 0.5 };
        seed_hsl_bars([255, 50, 95, 255], &mut bars);
        assert_eq!(bars, sliders::HslBars { h: 0.25, s: 0.5, l: 0.75, a: 0.5 });
    }

    #[test]
    fn menu_25_floors_luminosity_and_252_does_not() {
        // `Main.cs:51238-51241` (`num59 = 0.15`, `0f` for 252) with the
        // `:51226-51232` store: a grab from inside the row clamps to 0 left of
        // the bar, then re-floors — 0.15 on 25, 0.0 on 252.
        for (alpha_bar, floor) in [(false, 0.15f32), (true, 0.0f32)] {
            let mut bars = sliders::HslBars { h: -1.0, s: -1.0, l: -1.0, a: -1.0 };
            seed_hsl_bars([64, 64, 64, 64], &mut bars);
            let mut grab = 0u8;
            let left = sliders::color_slider_left(1600.0);
            let tops = sliders::color_slider_tops(320.0);
            // Press mid-row to take the sticky grab (`Main.cs:51222-51224`).
            let mouse = Vec2::new(left + 89.0, tops[2] + 8.0);
            sliders::color_sliders_update(
                1600.0, 320.0, &mut bars, floor, alpha_bar, mouse, true, &mut grab,
            );
            assert_eq!(grab, 3);
            // Drag far left while held: the sticky grab clamps to 0 and the
            // floor applies.
            let mouse = Vec2::new(left - 20.0, tops[2] + 8.0);
            sliders::color_sliders_update(
                1600.0, 320.0, &mut bars, floor, alpha_bar, mouse, true, &mut grab,
            );
            assert_eq!(bars.l, floor, "lum floor {floor}");
        }
    }

    #[test]
    fn menu_252_stores_alpha_and_25_leaves_it_255() {
        // `Main.cs:51279-51284`: `selColor = hslToRgb(h, s, l)`, then — only
        // when the alpha bar shows (252) — `selColor.A = (byte)(aBar * 255f)`.
        let mut bars = sliders::HslBars { h: -1.0, s: -1.0, l: -1.0, a: -1.0 };
        seed_hsl_bars([64, 64, 64, 64], &mut bars);
        let mut grab = 0u8;
        let left = sliders::color_slider_left(1600.0);
        let tops = sliders::color_slider_tops(320.0);
        // Drag the alpha row to t = 0.5 (`Main.cs:51264-51269`).
        let mouse = Vec2::new(left + 89.0, tops[3] + 8.0);
        sliders::color_sliders_update(1600.0, 320.0, &mut bars, 0.0, true, mouse, true, &mut grab);
        assert_eq!(grab, 4);
        assert!((bars.a - 0.5).abs() < 1e-6);
        // 252 recomposes with the dragged alpha: (byte)(0.5f * 255f) = 127.
        assert_eq!(recompose_sel_color(true, &bars)[3], 127);
        // 25 always recomposes opaque (the 3-arg `hslToRgb` defaults `a` to 255).
        assert_eq!(recompose_sel_color(false, &bars)[3], 255);
    }

    #[test]
    fn one_frame_of_the_colour_screens_recomposes_byte_exact() {
        // The full state sequence of a no-input frame: seed, update with the
        // pointer outside every hit box, recompose — the stored colours survive
        // the HSL round trip byte-exactly, so opening a colour screen and
        // leaving it never drifts the settings.
        for (sel, alpha_bar) in [([255u8, 50, 95, 255], false), ([64, 64, 64, 64], true)] {
            let mut bars = sliders::HslBars { h: -1.0, s: -1.0, l: -1.0, a: -1.0 };
            seed_hsl_bars(sel, &mut bars);
            let mut grab = 0u8;
            sliders::color_sliders_update(
                1600.0,
                320.0,
                &mut bars,
                if alpha_bar { 0.0 } else { 0.15 },
                alpha_bar,
                Vec2::new(10.0, 10.0),
                true,
                &mut grab,
            );
            assert_eq!(grab, 0, "no hit box at (10, 10)");
            assert_eq!(recompose_sel_color(alpha_bar, &bars), sel);
        }
    }

    #[test]
    fn resolution_screen_matches_the_vanilla_rows() {
        // `Main.cs:50169-50241`: `num2 = 210`, `num4 = 55` (`:50175-50176`),
        // every `array7` 0.85 (`:50170-50174`); `array4 = 100` on Apply
        // (`:50212`) and Back (`:50230`). The pending resolution mirrors the
        // current window (`:66399-66400`), so a fresh `Menu` reads the
        // 1600x900 reference frame.
        let items = menu(Page::Resolution).items();
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert_eq!(
            labels,
            [
                "Fullscreen Resolution: 1600x900",
                "Borderless Window: Disabled",
                "Go Fullscreen",
                "Apply",
                "Back",
            ]
        );
        let ys: Vec<f32> = items.iter().map(|i| i.y).collect();
        assert_eq!(ys, [0.0, 0.0, 0.0, 100.0, 100.0]);
        assert!(items.iter().all(|i| i.scale == 0.85));
        assert!(!items.iter().any(|i| i.header));
        // Draw y = `num2 + num4 * i + array4[i]` (`Main.cs:51769`): Apply at
        // 210 + 55*3 + 100 = 475, Back at 210 + 55*4 + 100 = 530.
        let layout = Page::Resolution.layout();
        let draw_y = |i: usize, items: &[Item]| layout.y0 + layout.row * i as f32 + items[i].y;
        assert_eq!(draw_y(3, &items), 475.0);
        assert_eq!(draw_y(4, &items), 530.0);
        // Rows 0-3 show the not-ported notice (the original cycles display
        // modes, toggles borderless, goes fullscreen and applies — `Main.cs
        // :50179-50227`); Back opens 1111 (`:50237`).
        let mut m = menu(Page::Resolution);
        for row in 0..4 {
            m.activate(row);
            assert!(m.hint.is_some(), "row {row} is a hint");
            assert_eq!(m.page, Page::Resolution);
        }
        m.activate(4);
        assert_eq!(m.page, Page::Video);
        assert!(m.hint.is_none());
    }

    #[test]
    fn resolution_row_tracks_the_window_size() {
        // `Main.cs:50178` concatenates the pending resolution; the port's
        // `screen_size` refreshes from the draw target (`:66399-66400`).
        let mut m = menu(Page::Resolution);
        m.screen_size = Vec2::new(1280.0, 720.0);
        assert_eq!(m.items()[0].label, "Fullscreen Resolution: 1280x720");
    }

    #[test]
    fn effects_screen_matches_the_vanilla_rows() {
        // `Main.cs:50097-50168`: `num2 = 210`, `num4 = 28`, `num5 = 9` with
        // `array4[num5 - 1] = 24` (`:50099-50102`) — but `array9[8]` stays
        // null and the item loop skips it (`Main.cs:51654`), so there are
        // eight visible rows and Back draws at `210 + 28*7` = 406, not 458
        // (Review Focus 3: porting "9 rows" draws a phantom duplicate).
        let m = menu(Page::Effects);
        let items = m.items();
        assert_eq!(items.len(), 8, "the phantom ninth row is skipped");
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert_eq!(
            labels,
            [
                "Effects",
                "Storm Effects: Enabled",
                "Sunlight Effects: Enabled",
                "Heat Distortion: Enabled",
                "Waves Quality: High",
                "Screen Shake: Enabled",
                "Thunder Effects: On",
                "Back",
            ]
        );
        let ys: Vec<f32> = items.iter().map(|i| i.y).collect();
        assert_eq!(ys, [-8.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        // `array7` 0.6 with the header at 0.7 (`Main.cs:50103-50110`).
        assert!(items[1..].iter().all(|i| i.scale == 0.6));
        assert_eq!(items[0].scale, 0.7);
        assert!(items[0].header, "array[0] = true");
        assert!(!items[7].header);
        // Draw y = `num2 + num4 * i + array4[i]` (`Main.cs:51769`).
        let layout = Page::Effects.layout();
        let draw_y = |i: usize| layout.y0 + layout.row * i as f32 + items[i].y;
        assert_eq!(draw_y(0), 202.0, "array4[0] -= 8");
        assert_eq!(draw_y(1), 238.0);
        assert_eq!(draw_y(7), 406.0, "Back sits at 406, not 458");
    }

    #[test]
    fn effects_rows_toggle_the_six_settings() {
        // `Main.cs:50117`, `:50124`, `:50131`, `:50154`, `:50161` flip the
        // flags; `:50144` cycles `WaveQuality = (q + 1) % 4`.
        let mut m = menu(Page::Effects);
        m.activate(1);
        assert!(!m.settings.use_storm_effects);
        assert_eq!(m.items()[1].label, "Storm Effects: Disabled");
        m.activate(2);
        assert!(!m.settings.foreground_sunlight);
        assert_eq!(m.items()[2].label, "Sunlight Effects: Disabled");
        m.activate(3);
        assert!(!m.settings.use_heat_distortion);
        assert_eq!(m.items()[3].label, "Heat Distortion: Disabled");
        m.activate(5);
        assert!(!m.settings.use_screen_shake);
        assert_eq!(m.items()[5].label, "Screen Shake: Disabled");
        m.activate(6);
        assert!(!m.settings.flashy_effects_world);
        assert_eq!(m.items()[6].label, "Thunder Effects: Reduced");
        // The wave cycle: 3 High -> 0 Off -> 1 Low -> 2 Medium -> 3 High.
        assert_eq!(m.settings.wave_quality, 3);
        m.activate(4);
        assert_eq!(m.settings.wave_quality, 0);
        assert_eq!(m.items()[4].label, "Waves Quality: Off");
        m.activate(4);
        assert_eq!(m.settings.wave_quality, 1);
        assert_eq!(m.items()[4].label, "Waves Quality: Low");
        m.activate(4);
        assert_eq!(m.settings.wave_quality, 2);
        assert_eq!(m.items()[4].label, "Waves Quality: Medium");
        m.activate(4);
        assert_eq!(m.settings.wave_quality, 3);
        assert_eq!(m.items()[4].label, "Waves Quality: High");
        // Back opens 1111 (`Main.cs:50169`).
        m.activate(7);
        assert_eq!(m.page, Page::Video);
    }

    #[test]
    fn language_screen_lists_the_twelve_vanilla_rows() {
        // `Main.cs:48493-48535`: the `Lang.menu[102]` header (`array4[0] =
        // -20`, `array7` 0.85), the twelve `Language.*` rows at 0.65, then
        // Back (`array4[13] = 9`, `array7` 0.8) — draw y `200 + 26*13 + 9` =
        // 547 (`:48510`, `:48525-48534`).
        let m = menu(Page::Language);
        let items = m.items();
        assert_eq!(items.len(), 14);
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert_eq!(&labels[..13], &["Select language", "English", "Deutsch (German)", "Italiano (Italian)", "Français (French)", "Español (Spanish)", "Русский (Russian)", "简体中文 (Simplified Chinese)", "Português brasileiro (Brazilian Portuguese)", "Polski (Polish)", "日本語 (Japanese)", "한국어 (Korean)", "繁體中文 (Traditional Chinese)"]);
        assert_eq!(labels[13], "Back");
        assert!(items[0].header);
        assert_eq!(items[0].scale, 0.85);
        assert_eq!(items[0].y, -20.0);
        assert!(items[1..13].iter().all(|i| i.scale == 0.65));
        assert_eq!(items[13].scale, 0.8);
        assert_eq!(items[13].y, 9.0);
        // Draw y (`Main.cs:51769`): header 180, English 226, Back 547.
        let layout = Page::Language.layout();
        assert_eq!((layout.y0, layout.row), (200.0, 26.0));
        let draw_y = |i: usize| layout.y0 + layout.row * i as f32 + items[i].y;
        assert_eq!(draw_y(0), 180.0);
        assert_eq!(draw_y(1), 226.0);
        assert_eq!(draw_y(13), 547.0);
        // Language rows show the notice (the original calls
        // `LanguageManager.SetLanguage`, `Main.cs:48519-48523`); Back opens 11
        // (`:48512-48517`).
        let mut m = menu(Page::Language);
        m.activate(1);
        assert!(m.hint.is_some());
        assert_eq!(m.page, Page::Language);
        m.activate(13);
        assert_eq!(m.page, Page::Settings);
        assert!(m.hint.is_none());
    }

    #[test]
    fn language_initial_picker_differs_from_the_language_screen() {
        // `Main.cs:48458-48491`: the same header and twelve rows but
        // `num4 = 30` (`:48462`), no Back row, `array7[0] = 1f` (`:48483`) —
        // versus 1213's `num4 = 26`, header 0.85, Back 0.8 (`:48525-48534`).
        let items = menu(Page::LanguageInitial).items();
        assert_eq!(items.len(), 13, "the first-launch picker has no Back");
        assert!(items[0].header);
        assert_eq!(items[0].scale, 1.0);
        assert_eq!(items[0].y, -20.0);
        assert!(items[1..].iter().all(|i| i.scale == 0.65));
        assert_eq!(items[1].label, "English");
        let initial = Page::LanguageInitial.layout();
        assert_eq!((initial.y0, initial.row), (200.0, 30.0));
        // Draw y (`Main.cs:51769`): header 180, English 230, last row 560 —
        // 1213 puts English at 226 and adds Back at 547.
        let draw_y = |i: usize| initial.y0 + initial.row * i as f32 + items[i].y;
        assert_eq!(draw_y(1), 230.0);
        assert_eq!(draw_y(12), 560.0);
        // A language row still only shows the notice (the original would
        // `SetLanguage` and jump to menuMode 0, `Main.cs:48484-48490`).
        let mut m = menu(Page::LanguageInitial);
        m.activate(1);
        assert!(m.hint.is_some());
        assert_eq!(m.page, Page::LanguageInitial);
    }

    #[test]
    fn video_and_settings_rows_navigate_to_the_task_4_screens() {
        // Video row 0 opens 111 (`Main.cs:49992-49996`), row 10 opens 2008
        // (`:50072-50077`); the Settings "Language" row opens 1213
        // (`:49759-49763`).
        let mut m = menu(Page::Video);
        m.activate(0);
        assert_eq!(m.page, Page::Resolution);
        assert!(m.hint.is_none());
        let mut m = menu(Page::Video);
        m.activate(10);
        assert_eq!(m.page, Page::Effects);
        assert!(m.hint.is_none());
        let mut m = menu(Page::Settings);
        m.activate(6);
        assert_eq!(m.page, Page::Language);
        assert!(m.hint.is_none());
    }

    #[test]
    fn controls_screen_matches_the_vanilla_rows() {
        // `Main.cs:50638-50698`: `num2 = 250`, `num4 = 52`, `num5 = 4`,
        // `array4[3] = 18`, every `array7` 0.78 (`:50639-50646`). The defaults
        // column: `ReversedUpDownArmorSetBonuses` is a plain static
        // (`Main.cs:273`) and the `ItemSlot.Options` ctor sets
        // DisableLeftShiftTrashCan = true / DisableQuickTrash = false
        // (`ItemSlot.cs:36-37`), so row 0 reads Down [221] and row 1 Left
        // Control [224]. Draw y (`Main.cs:51769`): 250, 302, 354, 424 — the
        // plan table's 464 for Back drops the `52 * 3` term.
        let items = menu(Page::Controls).items();
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert_eq!(
            labels,
            [
                "Activate Set Bonuses: Down",
                "Quick Trash: Left Control",
                "Keybindings",
                "Back",
            ]
        );
        let ys: Vec<f32> = items.iter().map(|i| i.y).collect();
        assert_eq!(ys, [0.0, 0.0, 0.0, 18.0]);
        assert!(items.iter().all(|i| i.scale == 0.78));
        let layout = Page::Controls.layout();
        let draw_y = |i: usize| layout.y0 + layout.row * i as f32 + items[i].y;
        assert_eq!(draw_y(0), 250.0);
        assert_eq!(draw_y(1), 302.0);
        assert_eq!(draw_y(2), 354.0);
        assert_eq!(draw_y(3), 424.0, "250 + 52*3 + 18");
        // Back opens 11 (`Main.cs:50695-50698`).
        let mut m = menu(Page::Controls);
        m.activate(3);
        assert_eq!(m.page, Page::Settings);
        // The toggled twins (`Main.cs:50649` / `:50656-50662`).
        let mut m = menu(Page::Controls);
        m.settings.reversed_armor_bonuses = true;
        m.settings.disable_left_shift_trash = false;
        assert_eq!(m.items()[0].label, "Activate Set Bonuses: Up");
        assert_eq!(m.items()[1].label, "Quick Trash: Left Shift");
        let mut m = menu(Page::Controls);
        m.settings.quick_trash = true;
        assert_eq!(m.items()[1].label, "Quick Trash: Disabled");
    }

    #[test]
    fn settings_controls_row_opens_1127() {
        // `Main.cs:49752-49758`: `Lang.menu[219]` sets `menuMode = 1127`.
        let mut m = menu(Page::Settings);
        m.activate(5);
        assert_eq!(m.page, Page::Controls);
        assert!(m.hint.is_none());
    }

    #[test]
    fn quick_trash_cycles_left_control_left_shift_disabled() {
        // `Main.cs:50665-50684`: Left Control -> Left Shift -> Disabled ->
        // Left Control, starting from the ctor defaults (`ItemSlot.cs:36-37`).
        let mut m = menu(Page::Controls);
        assert_eq!(m.items()[1].label, "Quick Trash: Left Control");
        m.activate(1);
        assert_eq!(m.items()[1].label, "Quick Trash: Left Shift");
        assert!(!m.settings.disable_left_shift_trash);
        assert!(!m.settings.quick_trash);
        m.activate(1);
        assert_eq!(m.items()[1].label, "Quick Trash: Disabled");
        assert!(m.settings.quick_trash);
        m.activate(1);
        assert_eq!(m.items()[1].label, "Quick Trash: Left Control");
        assert!(!m.settings.quick_trash);
        assert!(m.settings.disable_left_shift_trash);
    }

    #[test]
    fn keybindings_screen_lists_the_fourteen_binds() {
        // `Main.cs:50501-50637`: `num2 = 176`, `num4 = 22`, `num5 = 16`; the
        // bind rows are left-aligned at `array5 = -80` with scale 0.45
        // (`:50543-50545`), the raw `Lang.menu` values ([74]..[85], [120],
        // [130], trailing spaces kept) + the bind (`:50527-50540`); Reset
        // (`array7` 0.8, `array4 = 6`, `:50547-50549`) and Back (0.8,
        // `array4 = 16`, `:50550-50552`) stay centred. Draw y
        // (`Main.cs:51769`): Reset 176 + 22*14 + 6 = 490, Back 522.
        let m = menu(Page::Keybindings);
        let items = m.items();
        assert_eq!(items.len(), 16);
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert_eq!(
            labels,
            [
                "Up                W",
                "Down             S",
                "Left               A",
                "Right             D",
                "Jump             Space",
                "Throw            T",
                "Inventory         Escape",
                "Quick Heal       H",
                "Quick Mana      J",
                "Quick Buff       B",
                "Grapple           E",
                "Auto Select       LeftShift",
                "Smart Cursor    LeftControl",
                "Mount             R",
                "Reset to Default",
                "Back",
            ]
        );
        for (i, item) in items.iter().enumerate() {
            if i < 14 {
                assert_eq!(item.x, -80.0, "array5[{i}] = -80");
                assert!(item.left_aligned, "array8[{i}] = true");
                assert_eq!(item.scale, 0.45);
                assert_eq!(item.y, 0.0);
            } else {
                assert_eq!(item.x, 0.0, "Reset/Back stay centred");
                assert!(!item.left_aligned);
                assert_eq!(item.scale, 0.8);
            }
        }
        assert_eq!(items[14].y, 6.0);
        assert_eq!(items[15].y, 16.0);
        let layout = Page::Keybindings.layout();
        let draw_y = |i: usize| layout.y0 + layout.row * i as f32 + items[i].y;
        assert_eq!(draw_y(0), 176.0);
        assert_eq!(draw_y(14), 490.0);
        assert_eq!(draw_y(15), 522.0);
        // Reset restores the `:67163-67176` seeds and closes the capture
        // (`Main.cs:50559-50564`); Back opens 11 (`:50553-50558`).
        let mut m = menu(Page::Keybindings);
        m.keybinds[0] = "F3".into();
        m.set_key = Some(3);
        m.activate(14);
        assert_eq!(m.keybinds[0], "W");
        assert_eq!(m.set_key, None);
        m.activate(15);
        assert_eq!(m.page, Page::Settings);
    }

    #[test]
    fn set_key_capture_fills_the_clicked_row() {
        // `Main.cs:50565-50568` opens the capture on the clicked row, the row
        // shows `_` (`:50523-50526`), and the first pressed key is stored
        // under its name and closes it (`:50569-50576` — `string.Concat(key)`,
        // winit's debug name here).
        let mut m = menu(Page::Keybindings);
        m.activate(0);
        assert_eq!(m.set_key, Some(0));
        assert_eq!(m.items()[0].label, "Up                _");
        m.key(winit::keyboard::KeyCode::KeyG);
        assert_eq!(m.set_key, None);
        assert_eq!(m.keybinds[0], "KeyG");
        assert_eq!(m.items()[0].label, "Up                KeyG");
        // Navigation keys are captured too — vanilla's `GetPressedKeys` sees
        // Escape before anything else, so Escape rebinding to "Escape" (the
        // Inventory default) beats closing the screen.
        m.activate(6);
        assert_eq!(m.set_key, Some(6));
        m.key(winit::keyboard::KeyCode::Escape);
        assert_eq!(m.keybinds[6], "Escape");
        assert_eq!(m.page, Page::Keybindings);
        // The 272727 twin stores into the map binds
        // (`Main.cs:50460-50496`).
        let mut m = menu(Page::MapKeys);
        m.activate(3);
        assert_eq!(m.set_key, Some(3));
        assert_eq!(m.items()[3].label, "Zoom Out                    _");
        m.key(winit::keyboard::KeyCode::KeyN);
        assert_eq!(m.map_binds[3], "KeyN");
        assert_eq!(m.items()[3].label, "Zoom Out                    KeyN");
    }

    #[test]
    fn map_keys_screen_lists_the_six_map_binds() {
        // `Main.cs:50401-50500`: `num2 = 200`, `num4 = 30`, `num5 = 14` — but
        // rows 8-13 stay null and the item loop skips them (`Main.cs:51654`),
        // so eight rows show. The bind rows are left-aligned at
        // `array5 = -140`, scale 0.55 (`:50433-50435`), labels [106..111] +
        // the `:67190-67195` seeds (`:50425-50430`). Reset: `array7` 0.8,
        // `array4 = 6` (`:50439-50442`); Back keeps the 1.0 default — the
        // branch sets `array7[6]` twice (`:50439-50440`) and never touches
        // `array7[7]`, unlike the mode-27 twin (`:50550`). Draw y
        // (`Main.cs:51769`): Reset 200 + 30*6 + 6 = 386, Back 426.
        let m = menu(Page::MapKeys);
        let items = m.items();
        assert_eq!(items.len(), 8);
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert_eq!(
            labels,
            [
                "Toggle Map Style           Tab",
                "Toggle Fullscreen           M",
                "Zoom In                      Add",
                "Zoom Out                    Subtract",
                "Decrease Transparency     PageUp",
                "Increase Transparency      PageDown",
                "Reset to Default",
                "Back",
            ]
        );
        for (i, item) in items.iter().enumerate() {
            if i < 6 {
                assert_eq!(item.x, -140.0, "array5[{i}] = -140");
                assert!(item.left_aligned, "array8[{i}] = true");
                assert_eq!(item.scale, 0.55);
                assert_eq!(item.y, 0.0);
            }
        }
        assert_eq!(items[6].scale, 0.8);
        assert_eq!(items[6].y, 6.0);
        assert_eq!(items[7].scale, 1.0, "array7[7] is never set on 272727");
        assert_eq!(items[7].y, 16.0);
        let layout = Page::MapKeys.layout();
        let draw_y = |i: usize| layout.y0 + layout.row * i as f32 + items[i].y;
        assert_eq!(draw_y(0), 200.0);
        assert_eq!(draw_y(6), 386.0);
        assert_eq!(draw_y(7), 426.0);
        // Reset restores the `:67190-67195` seeds (`Main.cs:50449-50459`);
        // Back opens 11 (`:50445-50448`).
        let mut m = menu(Page::MapKeys);
        m.map_binds[0] = "F4".into();
        m.activate(6);
        assert_eq!(m.map_binds[0], "Tab");
        assert_eq!(m.map_binds[5], "PageDown");
        m.activate(7);
        assert_eq!(m.page, Page::Settings);
    }

    #[test]
    fn selector_rows_show_their_default_labels() {
        // `Main.cs:49941-49979` with the defaults: the minimap key and the
        // resource-set key default to their first entries here (the port's
        // resource-set default is "Default" — the recorded deviation), the
        // portrait enum's 0 is Detailed, `ShowText` is true from the static
        // ctor (`BigProgressBarSystem.cs:259`) and
        // `Main.FlashyEffectsInterface = true` (`Main.cs:66578`).
        let m = menu(Page::Interface);
        let items = m.items();
        assert_eq!(items[6].label, "Minimap Border: Default");
        assert_eq!(items[7].label, "Health and Mana Style: Classic");
        assert_eq!(items[8].label, "Portraits Style: Detailed");
        assert_eq!(items[9].label, "Boss Health Bar Numbers: On");
        assert_eq!(items[10].label, "Inventory Effects: On");
        // An unknown stored name normalizes to the first option for the label
        // (`SelectionHolder.cs:71-78` / `PlayerResourceSetsManager.cs:74-83`).
        let mut m = menu(Page::Interface);
        m.settings.minimap_frame = "Nope".into();
        m.settings.resource_sets = "Nope".into();
        assert_eq!(m.items()[6].label, "Minimap Border: Default");
        assert_eq!(m.items()[7].label, "Health and Mana Style: Fancy");
    }

    #[test]
    fn minimap_frame_cycle_walks_backwards_and_wraps() {
        // `SelectionHolder.CycleSelection` (`SelectionHolder.cs:44-58`)
        // activates the option *before* the active one and wraps the first to
        // the last — cycling from `Valkyrie` lands on `Retro`, and the wrap
        // is `Default -> Valkyrie` (the plan's "cycling from Valkyrie lands
        // on Default" drops the wrap direction). The full cycle from the
        // default key visits all nine and returns.
        let mut m = menu(Page::Interface);
        let mut keys = vec![m.settings.minimap_frame.clone()];
        for _ in 0..8 {
            m.activate(6);
            keys.push(m.settings.minimap_frame.clone());
        }
        assert_eq!(
            keys,
            [
                "Default",
                "Valkyrie",
                "Retro",
                "Leaf",
                "TwigLeaf",
                "StoneGold",
                "Sticks",
                "Remix",
                "Golden",
            ]
        );
        m.activate(6);
        assert_eq!(m.settings.minimap_frame, "Default", "the ninth hop closes the cycle");
        // Cycling from the last key lands on its predecessor.
        let mut m = menu(Page::Interface);
        m.settings.minimap_frame = "Valkyrie".into();
        m.activate(6);
        assert_eq!(m.settings.minimap_frame, "Retro");
        // An unknown key normalizes to the first option first, so it wraps to
        // the last (`SelectionHolder.cs:67-78`).
        let mut m = menu(Page::Interface);
        m.settings.minimap_frame = "Nope".into();
        m.activate(6);
        assert_eq!(m.settings.minimap_frame, "Valkyrie");
    }

    #[test]
    fn resource_set_cycle_walks_backwards_and_wraps() {
        // `PlayerResourceSetsManager.CycleResourceSet`
        // (`PlayerResourceSetsManager.cs:84-101`) — the same backwards walk
        // over the `:42-47` load order.
        let mut m = menu(Page::Interface);
        let mut keys = vec![m.settings.resource_sets.clone()];
        for _ in 0..5 {
            m.activate(7);
            keys.push(m.settings.resource_sets.clone());
        }
        assert_eq!(
            keys,
            [
                "Default",
                "New",
                "NewWithText",
                "HorizontalBars",
                "HorizontalBarsWithText",
                "HorizontalBarsWithFullText",
            ]
        );
        m.activate(7);
        assert_eq!(m.settings.resource_sets, "Default", "the sixth hop closes the cycle");
        // The label tracks the set (`Main.cs:49948`).
        assert_eq!(m.items()[7].label, "Health and Mana Style: Classic");
        m.activate(7);
        assert_eq!(m.items()[7].label, "Health and Mana Style: Fancy");
    }

    #[test]
    fn portrait_preference_cycles_the_four_modes() {
        // `Main.CycleNPCPortraitMode` (`Main.cs:40297-40315`): Detailed ->
        // CloseUp -> FullBodyRetro -> Disabled -> Detailed, labelled by the
        // `UI.Portraits*` values (`Main.cs:49954-49965`).
        let mut m = menu(Page::Interface);
        assert_eq!(m.items()[8].label, "Portraits Style: Detailed");
        m.activate(8);
        assert_eq!(m.settings.portrait_preference, 1);
        assert_eq!(m.items()[8].label, "Portraits Style: Profile");
        m.activate(8);
        assert_eq!(m.settings.portrait_preference, 2);
        assert_eq!(m.items()[8].label, "Portraits Style: Retro");
        m.activate(8);
        assert_eq!(m.settings.portrait_preference, 3);
        assert_eq!(m.items()[8].label, "Portraits Style: Disabled");
        m.activate(8);
        assert_eq!(m.settings.portrait_preference, 0);
        assert_eq!(m.items()[8].label, "Portraits Style: Detailed");
    }

    #[test]
    fn boss_bar_and_inventory_effects_rows_toggle_their_flags() {
        // `Main.cs:49966-49979`: `ToggleShowText()` (no sound) and the
        // `FlashyEffectsInterface` flip. The boss-bar flag is stored only —
        // P4's bar owns the numbers rendering.
        let mut m = menu(Page::Interface);
        assert!(m.settings.boss_bar_text);
        m.activate(9);
        assert!(!m.settings.boss_bar_text);
        assert_eq!(m.items()[9].label, "Boss Health Bar Numbers: Off");
        m.activate(9);
        assert_eq!(m.items()[9].label, "Boss Health Bar Numbers: On");
        m.activate(10);
        assert!(!m.settings.flashy_effects_interface);
        assert_eq!(m.items()[10].label, "Inventory Effects: Reduced");
        m.activate(10);
        assert_eq!(m.items()[10].label, "Inventory Effects: On");
    }
}
