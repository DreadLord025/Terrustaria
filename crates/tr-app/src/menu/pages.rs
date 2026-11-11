//! The menu screens' row tables — [`Menu::items`]'s one `Main.DrawMenu`
//! arm per page (A7: extracted from `menu.rs`), plus the vanilla row-name
//! constants the screens and their toggles share.

use super::actions::{Act, Action, Toggle};
use super::{Item, Menu, Page};

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
pub(super) const MINIMAP_FRAME_KEYS: [&str; 9] = [
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
pub(super) const RESOURCE_SET_KEYS: [&str; 6] = [
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
/// The stored `SelectionHolder` key name as the `UI.HealthManaStyle_{key}`
/// json key spells it (the label lookup, not the human text).
fn resource_set_label_name(key: &str) -> &str {
    if RESOURCE_SET_KEYS.contains(&key) {
        key
    } else {
        RESOURCE_SET_KEYS[0]
    }
}

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
pub(super) fn cycle_previous(order: &[&str], current: &str) -> String {
    match order.iter().position(|key| *key == current) {
        Some(0) | None => order[order.len() - 1].to_string(),
        Some(i) => order[i - 1].to_string(),
    }
}

/// The `Main.cUp..cMount` defaults (`Main.cs:1979-2005`), seeded by the
/// static initializer (`Main.cs:67163-67176`) and restored by
/// `ResetKeyBindings` (`Main.cs:52459-52475`).
pub(super) fn default_keybinds() -> [String; 14] {
    // The one table (`Main.cs:67163-67176` seeded into the menuMode 27 row
    // order, `:50505-50520`) — `crate::input_map` owns it so the Reset row and
    // the input pass cannot drift.
    crate::input_map::default_binds()
}

/// The `Main.cMap*` defaults (`Main.cs:67190-67195`), restored inline by the
/// menuMode 272727 Reset row (`Main.cs:50451-50456`).
pub(super) fn default_map_binds() -> [String; 6] {
    crate::input_map::default_map_binds()
}


/// The menuMode 131 Accept gate (`Main.cs:50905-50923`): `array2[2]` stays
/// `true` unless `Convert.ToInt32(Main.getPort)` lands in `1..=65535`.
/// `Convert.ToInt32` accepts surrounding whitespace and a sign
/// (`NumberStyles.Integer`), so the port trims before parsing.
pub(super) fn port_is_valid(text: &str) -> bool {
    text.trim().parse::<i32>().is_ok_and(|port| port > 0 && port <= 65535)
}

/// The password row text of the menuMode 30/31 screens
/// (`Main.cs:48691-48699` / `:48567-48575`): the raw `Netplay.ServerPassword`,
/// or one `*` per character while `Main.HidePassword` (`Main.cs:271`) is set —
/// before the blinker suffix is appended.
fn password_display(m: &Menu) -> String {
    let value = match m.page {
        Page::HostPassword => &m.server_password,
        _ => &m.join_password,
    };
    if m.settings.hide_password {
        "*".repeat(value.chars().count())
    } else {
        value.clone()
    }
}

/// `Main.DrawMenu`'s on/off row label pair, shared by the settings arms.
/// The `GameUI.Enabled`/`GameUI.Disabled`-style key a two-state format row's
/// `{0}` argument resolves to.
fn on_off_key(b: bool, on: &'static str, off: &'static str) -> &'static str {
    if b { on } else { off }
}

fn on_off(b: bool, on: &str, off: &str) -> String {
    if b { on.to_string() } else { off.to_string() }
}

impl Menu {
    /// The rows of the current screen, in `Main.DrawMenu` order.
    pub(super) fn items(&self) -> Vec<Item> {
        match self.page {
            Page::Main => self.main_items(),
            Page::Settings => self.settings_items(),
            Page::General => self.general_items(),
            Page::Interface => self.interface_items(),
            Page::Video => self.video_items(),
            Page::Multiplayer => self.multiplayer_items(),
            Page::JoinIp => self.join_ip_items(),
            Page::Volume => self.volume_items(),
            Page::Parallax => self.parallax_items(),
            Page::Cursor => self.cursor_items(),
            Page::CursorColor => self.cursor_color_items(),
            Page::BorderColor => self.border_color_items(),
            Page::Resolution => self.resolution_items(),
            Page::Effects => self.effects_items(),
            Page::Language => self.language_items(),
            Page::LanguageInitial => self.language_initial_items(),
            Page::Controls => self.controls_items(),
            Page::Keybindings => self.keybindings_items(),
            Page::MapKeys => self.map_keys_items(),
            Page::JoinPort => self.join_port_items(),
            Page::HostPassword => self.host_password_items(),
            Page::JoinPassword => self.join_password_items(),
            Page::ServerSettings => self.server_settings_items(),
            Page::ConfirmDeletePlayer => self.confirm_delete_player_items(),
            Page::ConfirmDeleteWorld => self.confirm_delete_world_items(),
            Page::ConfirmDiscard => self.confirm_discard_items(),
            Page::Credits => self.credits_items(),
        }
    }

    // `menuMode == 11`.
    fn main_items(&self) -> Vec<Item> {
        vec![
            // `Lang.menu[12]`/`[13]`/`[131]` + `UI.ResourcePacks`, `Lang.menu[14]`,
            // `UI.Credits`, `Lang.menu[15]` (`Main.cs:48908-48971`).
            Item::new(crate::lang::menu(12), Act::Out(Action::Singleplayer)),
            Item::new(crate::lang::menu(13), Act::Nav(Page::Multiplayer)),
            // `Lang.menu[131]` (`Main.cs:48928-48935`): `menuMode = 888`,
            // `Main.MenuUI.SetState(Main.AchievementsMenu)` — the
            // achievements menu overlay (M6-ACH-2).
            Item::new(crate::lang::menu(131), Act::Out(Action::Achievements)),
            Item::new(
                crate::lang::text("UI.ResourcePacks"),
                Act::Hint("Resource packs are not ported yet"),
            ),
            Item::new(crate::lang::menu(14), Act::Nav(Page::Settings)),
            // Row 5 (`Main.cs:48964-48971`): `UI.Credits` sets
            // `menuMode = 3000` (`:48967-48969`); the
            // `SkyManager.Activate("CreditsRoll")` cinematic is the
            // documented deviation — the port's credits page draws the
            // world backdrop + the Back row.
            Item::new(crate::lang::text("UI.Credits"), Act::Nav(Page::Credits)),
            Item::new(crate::lang::menu(15), Act::Out(Action::Quit)),
        ]
    }

    // `menuMode == 112`.
    fn settings_items(&self) -> Vec<Item> {
        vec![
            // `Lang.menu[114]`/`[210]`/`[63]`/`[65]`/`[218]`/`[219]`/`[103]`/`[5]`
            // (`Main.cs:49710-49768`).
            Item::new(crate::lang::menu(114), Act::Nav(Page::General)).scaled(0.75),
            Item::new(crate::lang::menu(210), Act::Nav(Page::Interface)).scaled(0.75),
            Item::new(crate::lang::menu(63), Act::Nav(Page::Video)).scaled(0.75),
            Item::new(crate::lang::menu(65), Act::Nav(Page::Volume)).scaled(0.75),
            // menuMode 11 row 4 (`Main.cs:49753-49759`): opens 1125
            // (`Main.cs:50309-50316` is the 1125 Back twin).
            Item::new(crate::lang::menu(218), Act::Nav(Page::Cursor)).scaled(0.75),
            // Row 5 (`Main.cs:49752-49758`): `Lang.menu[219]` opens 1127.
            Item::new(crate::lang::menu(219), Act::Nav(Page::Controls)).scaled(0.75),
            // Row 6 (`Main.cs:49759-49763`): `Lang.menu[103]` opens 1213.
            Item::new(crate::lang::menu(103), Act::Nav(Page::Language)).scaled(0.75),
            Item::new(crate::lang::menu(5), Act::Nav(Page::Main)).scaled(0.75).at(8.0),
        ]
    }

    // `menuMode == 1112`.
    fn general_items(&self) -> Vec<Item> {
        let s = &self.settings;
        vec![
            Item::new(on_off(s.auto_save, &crate::lang::menu(67), &crate::lang::menu(68)), Act::Toggle(Toggle::AutoSave)).scaled(0.78),
            Item::new(on_off(s.auto_pause, &crate::lang::menu(69), &crate::lang::menu(70)), Act::Toggle(Toggle::AutoPause)).scaled(0.78),
            Item::new(on_off(s.map_enabled, &crate::lang::menu(112), &crate::lang::menu(113)), Act::Toggle(Toggle::MapEnabled)).scaled(0.78),
            Item::new(on_off(s.hide_password, &crate::lang::menu(212), &crate::lang::menu(211)), Act::Toggle(Toggle::HidePassword)).scaled(0.78),
            Item::new(on_off(s.play_when_unfocused, &crate::lang::text("UI.PlayWhenUnfocusedOn"), &crate::lang::text("UI.PlayWhenUnfocusedOff")), Act::Toggle(Toggle::PlayWhenUnfocused)).scaled(0.78),
            Item::new(crate::lang::menu(5), Act::Nav(Page::Settings)).scaled(0.78).at(18.0),
        ]
    }

    // `menuMode == 1111`. Note the original leaves `array7` at 1.0 for the Back
    // row here (only the first ten rows are set to 0.6), so Back is bigger.
    fn interface_items(&self) -> Vec<Item> {
        let s = &self.settings;
        vec![
            Item::new(on_off(s.pickup_text, &crate::lang::menu(71), &crate::lang::menu(72)), Act::Toggle(Toggle::PickupText)).scaled(0.6),
            Item::new(
                format!("{} {}", crate::lang::menu(123), crate::lang::menu(124 + s.event_progress.min(2) as usize)),
                Act::Toggle(Toggle::EventProgress),
            ).scaled(0.6),
            Item::new(on_off(s.placement_preview, &crate::lang::menu(128), &crate::lang::menu(129)), Act::Toggle(Toggle::PlacementPreview)).scaled(0.6),
            Item::new(on_off(s.highlight_new_items, &crate::lang::text("LegacyInterface.117"), &crate::lang::text("LegacyInterface.116")), Act::Toggle(Toggle::HighlightNew)).scaled(0.6),
            Item::new(on_off(s.tile_grid, &crate::lang::menu(229), &crate::lang::menu(230)), Act::Toggle(Toggle::TileGrid)).scaled(0.6),
            Item::new(on_off(s.gamepad_instructions, &crate::lang::menu(242), &crate::lang::menu(241)), Act::Toggle(Toggle::GamepadInstructions)).scaled(0.6),
            // Row 6 (`Main.cs:49941-49946`): `UI.SelectMapBorder`
            // ("Minimap Border: {0}") with the active `UI.MinimapFrame_*`
            // name; the click calls `CycleSelection` with no sound. The
            // stored key feeds `ui::minimap::style_index` live, so the
            // minimap frame re-renders as the row cycles.
            Item::new(
                crate::lang::text_args(
                    "UI.SelectMapBorder",
                    &[crate::lang::text(&format!(
                        "UI.MinimapFrame_{}",
                        if MINIMAP_FRAME_KEYS.contains(&s.minimap_frame.as_str()) {
                            s.minimap_frame.as_str()
                        } else {
                            MINIMAP_FRAME_KEYS[0]
                        }
                    ))],
                ),
                Act::Toggle(Toggle::MinimapFrame),
            )
            .scaled(0.6),
            // Row 7 (`Main.cs:49947-49952`): `UI.SelectHealthStyle` with
            // `UI.HealthManaStyle_{ActiveSetKeyName}`; no click sound.
            Item::new(
                crate::lang::text_args("UI.SelectHealthStyle", &[crate::lang::text(&format!("UI.HealthManaStyle_{}", resource_set_label_name(&s.resource_sets)))]),
                Act::Toggle(Toggle::ResourceSets),
            )
            .scaled(0.6),
            // Row 8 (`Main.cs:49953-49965`): the plain `UI.Portraits*`
            // labels — the `switch`'s `_` arm reads Detailed; the click
            // runs `CycleNPCPortraitMode` (`Main.cs:40297-40315`).
            Item::new(
                crate::lang::text(match s.portrait_preference {
                    1 => "UI.PortraitsCloseUp",
                    2 => "UI.PortraitsFullBody",
                    3 => "UI.PortraitsDisabled",
                    _ => "UI.PortraitsDetailed",
                }),
                Act::Toggle(Toggle::PortraitPreference),
            )
            .scaled(0.6),
            // Row 9 (`Main.cs:49966-49971`): `BigProgressBarSystem.
            // ToggleShowText()` with no sound. The flag is stored only —
            // P4's boss bar owns the numbers rendering.
            Item::new(
                on_off(
                    s.boss_bar_text,
                    &crate::lang::text("UI.ShowBossLifeTextOn"),
                    &crate::lang::text("UI.ShowBossLifeTextOff"),
                ),
                Act::Toggle(Toggle::BossBarText),
            )
            .scaled(0.6),
            // Row 10 (`Main.cs:49972-49979`): the
            // `UI.FlashyEffectsInterfaceOn/Off` pair.
            Item::new(
                on_off(
                    s.flashy_effects_interface,
                    &crate::lang::text("UI.FlashyEffectsInterfaceOn"),
                    &crate::lang::text("UI.FlashyEffectsInterfaceOff"),
                ),
                Act::Toggle(Toggle::FlashyEffectsInterface),
            )
            .scaled(0.6),
            Item::new(crate::lang::menu(5), Act::Nav(Page::Settings)).scaled(0.6).at(16.0),
        ]
    }

    // menuMode 12 (`Main.cs:50699-50736`): `num41` is 0 without Steam
    // networking (`SocialAPI.Network == null`, `:50700`), so the
    // `Lang.menu[145]` "Join via Steam" row stored at index 1
    // (`:50703`) is immediately overwritten by `Lang.menu[88]` at
    // `1 + num41` (`:50704`) — the non-Steam screen is
    // Join / Host & Play / Back (`:50702`, `:50704`, `:50705`).
    // Vanilla routes both joins through the character select
    // (`:50713-50730`, then `:66241`/`:66234-66236`); the port has no
    // character picker yet (P6), so the rows open the input screens
    // directly (plan Task 6 deviations).
    fn video_items(&self) -> Vec<Item> {
        let s = &self.settings;
        vec![
            // Row 0 (`Main.cs:49992-49996`): `Lang.menu[51]` opens 111.
            Item::new(crate::lang::menu(51), Act::Nav(Page::Resolution)).scaled(0.6),
            Item::new(
                format!("Zoom: {:.0}%", s.zoom * 100.0),
                Act::Toggle(Toggle::Zoom),
            )
            .scaled(0.6),
            // menuMode 1111 row 1 (`Main.cs:49996-50002`): `Lang.menu[52]`
            // opens the parallax slider (28).
            Item::new(crate::lang::menu(52), Act::Nav(Page::Parallax)).scaled(0.6),
            Item::new(crate::lang::menu(247 + s.frame_skip.min(2) as usize), Act::Toggle(Toggle::FrameSkip)).scaled(0.6),
            Item::new(crate::lang::text(match s.light_mode {
        1 => "UI.LightMode_White",
        2 => "UI.LightMode_Retro",
        3 => "UI.LightMode_Trippy",
        _ => "UI.LightMode_Color",
    }), Act::Toggle(Toggle::LightMode)).scaled(0.6),
            Item::new(crate::lang::menu(59 + s.quality.min(3) as usize), Act::Toggle(Toggle::Quality)).scaled(0.6),
            Item::new(on_off(s.background, &crate::lang::menu(100), &crate::lang::menu(101)), Act::Toggle(Toggle::Background)).scaled(0.6),
            Item::new(on_off(s.blood, &crate::lang::menu(132), &crate::lang::menu(133)), Act::Toggle(Toggle::Blood)).scaled(0.6),
            Item::new(on_off(s.miners_wobble, &crate::lang::menu(250), &crate::lang::menu(251)), Act::Toggle(Toggle::MinersWobble)).scaled(0.6),
            Item::new(on_off(s.tiles_sway, &crate::lang::text("UI.TilesSwayInWindOn"), &crate::lang::text("UI.TilesSwayInWindOff")), Act::Toggle(Toggle::TilesSway)).scaled(0.6),
            // Row 10 (`Main.cs:50072-50077`): `UI.Effects` opens 2008.
            Item::new(crate::lang::text("UI.Effects"), Act::Nav(Page::Effects)).scaled(0.6),
            Item::new(crate::lang::menu(5), Act::Nav(Page::Settings)).at(8.0),
        ]
    }

    // menuMode 13 (`Main.cs:50737-50883`, the keyboard branch from
    // `:50790`): `num5 = 11`, `num2 = 180`, `num4 = 30`
    // (`:50798-50800`). Row 0 is the `Lang.menu[89]` header
    // (`:50801-50802`), row 1 the input text with the blinker cursor
    // (`:50803-50814`), rows 2-8 the recent servers at scale 0.6
    // (`:50834-50848` — the port keeps no recents, so they stay the
    // blank non-clickable rows), row 9 `Lang.menu[4]` "Accept"
    // (`:50815-50817`) and row 10 `Lang.menu[5]` "Back"
    // (`:50832-50833`).
    fn multiplayer_items(&self) -> Vec<Item> {
        vec![
            // Row 0 (`:50702`): `Lang.menu[87]` "Join" — vanilla opens the
            // character select, then 13 (`:50714-50718`, `:66241`).
            Item::new(crate::lang::menu(87), Act::Nav(Page::JoinIp)),
            // Row 1 (`:50704`, the `1 + num41` overwrite): `Lang.menu[88]`
            // "Host & Play" — vanilla opens the character select with
            // `menuServer` (`:50720-50726`), the port goes to 30.
            Item::new(crate::lang::menu(88), Act::Nav(Page::HostPassword)),
            // Row 2 (`:50705`, the `2 + num41` Back): `Lang.menu[5]`
            // returns to menuMode 0 (`:50727-50733`).
            Item::new(crate::lang::menu(5), Act::Nav(Page::Main)),
        ]
    }

    // menuMode 26 (`Main.cs:50362-50380`): `num5 = 3`, `array9[0] = ""`,
    // `array9[1] = Lang.menu[65]` with `array4[1] = 10` and
    // `array[1] = true`, `array9[2] = Lang.menu[5]` with
    // `array4[2] = 250`; Back opens 11 (`:50375-50379`).
    fn join_ip_items(&self) -> Vec<Item> {
            // `array9[1] = Main.getIP` plus `|` on the blink frame — the
            // `|` frame also nudges the row one pixel right
            // (`array5[1] = 1`, `:50805-50813`).
            let input = if self.blink_on {
                Item::new(format!("{}|", self.get_ip), Act::None).at(19.0).heading().offset_x(1.0)
            } else {
                Item::new(format!("{} ", self.get_ip), Act::None).at(19.0).heading()
            };
            // `array2[9]` disables Accept while the IP is blank or a lone
            // space (`:50817-50831`); the branch also wipes a leading
            // space off the text itself (`:50818-50823`).
            let accept_ready =
                !self.get_ip.is_empty() && !self.get_ip.starts_with(' ');
            vec![
                Item::new(crate::lang::menu(89), Act::None).heading(),
                input,
                // Rows 2-8 (`:50834-50848`): `array4 = 40`, `array7 = 0.6`,
                // blank + `array[i] = true` without a recent entry.
                Item::new("", Act::None).at(40.0).heading().scaled(0.6),
                Item::new("", Act::None).at(40.0).heading().scaled(0.6),
                Item::new("", Act::None).at(40.0).heading().scaled(0.6),
                Item::new("", Act::None).at(40.0).heading().scaled(0.6),
                Item::new("", Act::None).at(40.0).heading().scaled(0.6),
                Item::new("", Act::None).at(40.0).heading().scaled(0.6),
                Item::new("", Act::None).at(40.0).heading().scaled(0.6),
                // Row 9 (`:50815-50817`): `array4[9] = 44`; a disabled
                // Accept row is non-clickable like a header
                // (`Main.cs:51790-51796` covers `array2` too).
                if accept_ready {
                    Item::new(crate::lang::menu(4), Act::Nav(Page::JoinPort)).at(44.0)
                } else {
                    Item::new(crate::lang::menu(4), Act::None).at(44.0).heading()
                },
                // Row 10 (`:50832-50833`): `array4[10] = 64`; vanilla
                // returns to the character select (`:50859-50864`), the
                // port returns to the multiplayer menu (documented
                // deviation, pinned by the escape test).
                Item::new(crate::lang::menu(5), Act::Nav(Page::Multiplayer)).at(64.0),
            ]
    }

    // menuMode 28 (`Main.cs:50381-50400`): the branch refreshes
    // `Main.caveParallax = 1 - bgScroll / 500` (`:50382`), `num5 = 3`,
    // `array9[1] = Lang.menu[52]` (header), `array4[2] = 170`; Back
    // opens 1111 (`:50395-50399`).
    fn volume_items(&self) -> Vec<Item> {
        vec![
            Item::new("", Act::None),
            Item::new(crate::lang::menu(65), Act::None).at(10.0).heading(),
            Item::new(crate::lang::menu(5), Act::Nav(Page::Settings)).at(250.0),
        ]
    }

    // menuMode 1125 (`Main.cs:50242-50317`): `num5 = 7`,
    // `array4[6] = 18` (`:50245`), every `array7` 0.73 (`:50247-50251`).
    fn parallax_items(&self) -> Vec<Item> {
        vec![
            Item::new("", Act::None),
            Item::new(crate::lang::menu(52), Act::None).at(10.0).heading(),
            Item::new(crate::lang::menu(5), Act::Nav(Page::Video)).at(170.0),
        ]
    }

    // menuMode 25 (`Main.cs:50318-50339`): `num5 = 3`,
    // `array9[1] = Lang.menu[64]` with `array4[1] = 10` and
    // `array[1] = true`, `array4[2] = 250`; the HSL sliders draw before
    // the rows and Back opens 1125 (`:50333-50338`).
    fn cursor_items(&self) -> Vec<Item> {
        let s = &self.settings;
        vec![
            // Rows 0-1 (`Main.cs:50253-50268`): the colour rows seed
            // `Main.selColor` from the stored colour, then open 25 / 252.
            Item::new(crate::lang::menu(64), Act::PickColor(Page::CursorColor)).scaled(0.73),
            Item::new(crate::lang::menu(217), Act::PickColor(Page::BorderColor)).scaled(0.73),
            // Row 2 (`Main.cs:50269-50277`):
            // `cSmartCursorModeIsToggleAndNotHold ? Lang.menu[121] : [122]`.
            Item::new(
                on_off(
                    s.smart_cursor_toggle,
                    &crate::lang::menu(121),
                    &crate::lang::menu(122),
                ),
                Act::Toggle(Toggle::SmartCursorToggle),
            )
            .scaled(0.73),
            // Row 3 (`Main.cs:50278-50285`):
            // `SmartAxeAfterPickaxe ? Lang.menu[214] : [213]`.
            Item::new(
                on_off(
                    s.smart_axe_first,
                    &crate::lang::menu(214),
                    &crate::lang::menu(213),
                ),
                Act::Toggle(Toggle::SmartAxeFirst),
            )
            .scaled(0.73),
            // Row 4 (`Main.cs:50286-50293`):
            // `SmartBlocksEnabled ? Lang.menu[215] : [216]`.
            Item::new(
                on_off(
                    s.smart_blocks,
                    &crate::lang::menu(215),
                    &crate::lang::menu(216),
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
                crate::lang::menu(match s.lock_on_priority {
                    0 => 232,
                    1 => 233,
                    _ => 234,
                }),
                Act::Toggle(Toggle::LockOnPriority),
            )
            .scaled(0.73),
            // Row 6 (`Main.cs:50309-50316`): Back opens 11.
            Item::new(crate::lang::menu(5), Act::Nav(Page::Settings)).scaled(0.73).at(18.0),
        ]
    }

    // menuMode 252 (`Main.cs:50340-50361`): the `Lang.menu[217]` twin —
    // same rows, full-range luminosity and the alpha bar.
    fn cursor_color_items(&self) -> Vec<Item> {
        vec![
            Item::new("", Act::None),
            Item::new(crate::lang::menu(64), Act::None).at(10.0).heading(),
            Item::new(crate::lang::menu(5), Act::Nav(Page::Cursor)).at(250.0),
        ]
    }

    // menuMode 111 (`Main.cs:50169-50241`): `num2 = 210`, `num4 = 55`,
    // every `array7` 0.85 (`:50170-50174`). The window-resolution
    // system is out of scope (spec §9.1), so rows 0-3 keep their
    // vanilla labels but show the not-ported notice where the original
    // cycles display modes (`:50179-50191`), toggles
    // `PendingBorderlessState` (`:50198-50204`), calls
    // `ToggleFullScreen` (`:50206-50210`) or `SetResolution`
    // (`:50212-50227`).
    fn border_color_items(&self) -> Vec<Item> {
        vec![
            Item::new("", Act::None),
            Item::new(crate::lang::menu(217), Act::None).at(10.0).heading(),
            Item::new(crate::lang::menu(5), Act::Nav(Page::Cursor)).at(250.0),
        ]
    }

    // menuMode 2008 (`Main.cs:50097-50168`): `num2 = 210`, `num4 = 28`,
    // `num5 = 9` with `array4[num5 - 1] = 24` (`:50099-50102`) — but
    // `array9[8]` is never assigned, and the item loop skips null
    // labels (`:51659-51662`), so the screen has eight visible rows and
    // Back sits at `210 + 28 * 7` = 406 (Review Focus 3; porting "9
    // rows" would draw a phantom duplicate).
    fn resolution_items(&self) -> Vec<Item> {
        vec![
            // Row 0 (`Main.cs:50178`): `Lang.menu[73] + ": " +
            // PendingResolutionWidth + "x" + PendingResolutionHeight`; the
            // pending size mirrors the live window (`:66399-66400`).
            Item::new(
                format!(
                    "{}: {}x{}",
                    crate::lang::menu(73),
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
                crate::lang::menu(246),
                Act::Hint("Borderless windows are not ported yet"),
            )
            .scaled(0.85),
            // Row 2 (`Main.cs:50206-50210`): `Lang.menu[50]` while
            // windowed; the port always runs windowed.
            Item::new(crate::lang::menu(50), Act::Hint("Fullscreen is not ported yet")).scaled(0.85),
            // Row 3 (`Main.cs:50212-50227`): `array4 = 100`, label
            // `Lang.menu[134]`.
            Item::new(crate::lang::menu(134), Act::Hint("Resolution switching is not ported yet"))
                .scaled(0.85)
                .at(100.0),
            // Row 4 (`Main.cs:50229-50239`): `array4 = 100`; Back resets
            // the pending state and opens 1111 (`:50237`).
            Item::new(crate::lang::menu(5), Act::Nav(Page::Video)).scaled(0.85).at(100.0),
        ]
    }

    // menuMode 1213 (`Main.cs:48493-48535`): the `Lang.menu[102]`
    // header at `array4[0] = -20` (`:48527`), the twelve
    // `Language.*` rows, then `array9[num10] = Lang.menu[5]` Back with
    // `array4[13] = 9` (`:48510`, `:48528`) — draw y `200 + 26*13 + 9`
    // = 547. `array7` 0.65, header 0.85, Back 0.8 (`:48529-48534`).
    // Language switching is out of scope (spec §9.1), so the rows show
    // the not-ported notice (`Main.cs:48519-48523` would call
    // `LanguageManager.SetLanguage`).
    fn effects_items(&self) -> Vec<Item> {
        let s = &self.settings;
        vec![
            // Row 0 (`Main.cs:50108-50111`): the `UI.Effects` header,
            // `array[0] = true`, `array7 = 0.7`, `array4 -= 8`.
            Item::new(crate::lang::text("UI.Effects"), Act::None).at(-8.0).heading().scaled(0.7),
            // Row 1 (`Main.cs:50113-50118`):
            // `GameUI.StormEffects` with `GameUI.Enabled`/`Disabled`.
            Item::new(
                crate::lang::text_args("GameUI.StormEffects", &[crate::lang::text(on_off_key(s.use_storm_effects, "GameUI.Enabled", "GameUI.Disabled"))]),
                Act::Toggle(Toggle::UseStormEffects),
            )
            .scaled(0.6),
            // Row 2 (`Main.cs:50120-50125`): `GameUI.ForegroundSunlightEffects`.
            Item::new(
                crate::lang::text_args("GameUI.ForegroundSunlightEffects", &[crate::lang::text(on_off_key(s.foreground_sunlight, "GameUI.Enabled", "GameUI.Disabled"))]),
                Act::Toggle(Toggle::ForegroundSunlight),
            )
            .scaled(0.6),
            // Row 3 (`Main.cs:50127-50132`): `GameUI.HeatDistortion`.
            Item::new(
                crate::lang::text_args("GameUI.HeatDistortion", &[crate::lang::text(on_off_key(s.use_heat_distortion, "GameUI.Enabled", "GameUI.Disabled"))]),
                Act::Toggle(Toggle::UseHeatDistortion),
            )
            .scaled(0.6),
            // Row 4 (`Main.cs:50134-50146`): `GameUI.WaveQuality` with
            // 1 Low, 2 Medium, 3 High, anything else Off (`_` arm).
            Item::new(
                crate::lang::text_args(
                    "GameUI.WaveQuality",
                    &[crate::lang::text(match s.wave_quality {
                        1 => "GameUI.QualityLow",
                        2 => "GameUI.QualityMedium",
                        3 => "GameUI.QualityHigh",
                        _ => "GameUI.QualityOff",
                    })],
                ),
                Act::Toggle(Toggle::WaveQuality),
            )
            .scaled(0.6),
            // Row 5 (`Main.cs:50150-50155`): `GameUI.ScreenShake`.
            Item::new(
                crate::lang::text_args("GameUI.ScreenShake", &[crate::lang::text(on_off_key(s.use_screen_shake, "GameUI.Enabled", "GameUI.Disabled"))]),
                Act::Toggle(Toggle::UseScreenShake),
            )
            .scaled(0.6),
            // Row 6 (`Main.cs:50157-50162`): `UI.FlashyEffectsWorldOn` /
            // `UI.FlashyEffectsWorldOff` ("Thunder Effects: On/Reduced").
            Item::new(
                on_off(s.flashy_effects_world, &crate::lang::text("UI.FlashyEffectsWorldOn"), &crate::lang::text("UI.FlashyEffectsWorldOff")),
                Act::Toggle(Toggle::FlashyEffectsWorld),
            )
            .scaled(0.6),
            // Row 7 (`Main.cs:50164-50169`): Back opens 1111.
            Item::new(crate::lang::menu(5), Act::Nav(Page::Video)).scaled(0.6),
        ]
    }

    // menuMode 1212 (`Main.cs:48458-48491`): the first-launch picker —
    // the same `Lang.menu[102]` header (`array4[0] = -20`, `:48464`)
    // and twelve `Language.*` rows at `num2 = 200`, `num4 = 30`
    // (`:48460-48462`), no Back row, `array7[0] = 1f` (`:48483`),
    // languages 0.65. The original enters it on first launch
    // (`Main.cs:10655`); here it is reachable only via `TR_PAGE=1212`,
    // and a language row shows the same notice (the original would
    // `SetLanguage` and jump to menuMode 0, `:48484-48490`).
    fn language_items(&self) -> Vec<Item> {
        // `array9[0] = Lang.menu[102]` (`Main.cs:48495`) and the twelve
        // `Language.GetTextValue("Language.*")` rows in the `:48497-48508`
        // order, then `Lang.menu[5]` Back (`:48510`). A row calls
        // `LanguageManager.SetLanguage(selectedMenu)` (`:48520`) — the
        // `GameCulture` legacy id — and the page stays up so the rows redraw
        // localized.
        let mut rows = vec![
            Item::new(crate::lang::menu(102), Act::None)
                .at(-20.0)
                .heading()
                .scaled(0.85),
        ];
        for culture in crate::lang::cultures() {
            rows.push(
                Item::new(
                    crate::lang::text(culture.language_key()),
                    Act::SetLanguage(culture.legacy_id()),
                )
                .scaled(0.65),
            );
        }
        rows.push(Item::new(crate::lang::menu(5), Act::Nav(Page::Settings)).scaled(0.8).at(9.0));
        rows
    }

    // menuMode 1127 (`Main.cs:50638-50698`): `num2 = 250`, `num4 = 52`,
    // `num5 = 4`, `array4[3] = 18`, every `array7` 0.78
    // (`:50639-50646`). Draw y = `250 + 52*i + array4[i]`
    // (`Main.cs:51769`): 250, 302, 354, 424.
    fn language_initial_items(&self) -> Vec<Item> {
        // menuMode 1212 (`Main.cs:48458-48491`): the same `Lang.menu[102]`
        // header and twelve `Language.*` rows, no Back row; a row
        // `SetLanguage`s and jumps to the title (`:48484-48490`).
        let mut rows = vec![
            Item::new(crate::lang::menu(102), Act::None).at(-20.0).heading(),
        ];
        for culture in crate::lang::cultures() {
            rows.push(
                Item::new(
                    crate::lang::text(culture.language_key()),
                    Act::SetLanguage(culture.legacy_id()),
                )
                .scaled(0.65),
            );
        }
        rows
    }

    // menuMode 27 (`Main.cs:50501-50637`): `num2 = 176`, `num4 = 22`,
    // `num5 = 16`. Rows 0-13 are the bind list — the raw `Lang.menu`
    // value (trailing spaces kept) + the bind name
    // (`:50527-50540`), left-aligned at `array5 = -80` with scale 0.45
    // (`:50543-50545`); the row being rebound shows `"_"` instead of
    // its bind (`:50523-50526`). Unreachable upstream — no
    // `menuMode = 27` assignment exists under `decompiled/` (the
    // Settings "Controls" row opens 1127, whose Keybindings row opens
    // the 888 UIState) — `TR_PAGE=27` only.
    fn controls_items(&self) -> Vec<Item> {
        let s = &self.settings;
        vec![
            // Row 0 (`Main.cs:50647-50656`):
            // `ReversedUpDownArmorSetBonuses ? [220] : [221]`.
            Item::new(
                on_off(
                    s.reversed_armor_bonuses,
                    &crate::lang::menu(220),
                    &crate::lang::menu(221),
                ),
                Act::Toggle(Toggle::ReversedArmorBonuses),
            )
            .scaled(0.78),
            // Row 1 (`Main.cs:50656-50687`): the three-state quick-trash
            // cycle — `DisableQuickTrash ? [253] : (DisableLeftShiftTrashCan
            // ? [224] : [223])`.
            Item::new(
                if s.quick_trash {
                    crate::lang::menu(253)
                } else if s.disable_left_shift_trash {
                    crate::lang::menu(224)
                } else {
                    crate::lang::menu(223)
                },
                Act::Toggle(Toggle::QuickTrash),
            )
            .scaled(0.78),
            // Row 2 (`Main.cs:50688-50694`): `Lang.menu[222]` opens the
            // menuMode 888 `ManageControlsMenu` UIState (`:50661-50666`) —
            // out of scope (spec §9.1), so the row shows the notice.
            Item::new(
                crate::lang::menu(222),
                Act::Hint("The manage-controls screen is not ported yet"),
            )
            .scaled(0.78),
            // Row 3 (`Main.cs:50695-50698`): Back opens 11.
            Item::new(crate::lang::menu(5), Act::Nav(Page::Settings)).scaled(0.78).at(18.0),
        ]
    }

    // menuMode 272727 (`Main.cs:50401-50500`): `num2 = 200`,
    // `num4 = 30`, `num5 = 14` — but only rows 0-7 get labels and the
    // item loop skips the nulls (`Main.cs:51654`), so the screen shows
    // eight rows. The six map binds are left-aligned at
    // `array5 = -140`, scale 0.55 (`:50433-50435`), labels [106..111]
    // + bind (`:50425-50430`); the row being rebound shows `"_"`.
    // Unreachable upstream — `TR_PAGE=272727` only.
    fn keybindings_items(&self) -> Vec<Item> {
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
                Item::new(crate::lang::menu(86), Act::ResetKeys { map: false }).scaled(0.8).at(6.0),
            );
            // Row 15 (`Main.cs:50550-50552`): Back opens 11.
            rows.push(Item::new(crate::lang::menu(5), Act::Nav(Page::Settings)).scaled(0.8).at(16.0));
            rows
    }

    // menuMode 131 (`Main.cs:50884-50964`): keeps the vanilla
    // `num2`/`num4` defaults of 250/80 (`:48419-48421`). Row 0 is the
    // `Lang.menu[90]` header (`:50903`, `array[0] = true` at
    // `:50947`), row 1 the input with the blinker (`:50937-50948`,
    // `array4[1] = -20` at `:50949`), row 2 `Lang.menu[4]` "Accept"
    // (`:50950-50951`, `array4[2] = 20`) and row 3 `Lang.menu[5]`
    // "Back" (`:50952`).
    fn map_keys_items(&self) -> Vec<Item> {
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
                Item::new(crate::lang::menu(86), Act::ResetKeys { map: true }).scaled(0.8).at(6.0),
            );
            // Row 7 (`Main.cs:50441-50444`): `array4 = 16` but `array7[7]`
            // is never set here (the branch sets `array7[6]` twice,
            // `:50439-50440`), so Back keeps the 1.0 default — unlike the
            // mode-27 twin's 0.8 (`:50550`).
            rows.push(Item::new(crate::lang::menu(5), Act::Nav(Page::Settings)).at(16.0));
            rows
    }

    // menuMode 30 (`Main.cs:48660-48725`): the host password screen —
    // `num5 = 4` over the 250/80 defaults, the `Lang.menu[7]` header
    // (`:48677`), the masked input (`:48691-48707`), "Accept"
    // (`:48712-48713`) and "Back" (`:48714`). Accept runs
    // `HostAndPlay` (`:48721-48724`), whose listen port is fixed at
    // 7777 (`Main.cs:52011`) — the plan's `Action::Host { port: 7777 }`.
    fn join_port_items(&self) -> Vec<Item> {
            // `array2[2]` disables Accept unless the text parses to a port
            // in 1..=65535 (`:50904-50923`).
            let accept_ready = port_is_valid(&self.get_port);
            vec![
                Item::new(crate::lang::menu(90), Act::None).heading(),
                // `array9[1] = Main.getPort` plus the blinker cursor and
                // the `array5[1] = 1` nudge (`:50937-50946`).
                if self.blink_on {
                    Item::new(format!("{}|", self.get_port), Act::None)
                        .at(-20.0)
                        .heading()
                        .offset_x(1.0)
                } else {
                    Item::new(format!("{} ", self.get_port), Act::None).at(-20.0).heading()
                },
                if accept_ready {
                    Item::new(crate::lang::menu(4), Act::Nav(Page::JoinPassword)).at(20.0)
                } else {
                    Item::new(crate::lang::menu(4), Act::None).at(20.0).heading()
                },
                // Row 3 (`:50954-50959`): vanilla Back returns to the
                // character select, the port returns to the IP screen.
                Item::new(crate::lang::menu(5), Act::Nav(Page::JoinIp)),
            ]
    }

    // menuMode 31 (`Main.cs:48536-48601`): the join twin — the
    // `Lang.menu[3]` header (`:48553`), the masked input
    // (`:48567-48583`), "Accept" (`:48587-48589`) and "Back"
    // (`:48590`). Accept answers the demand (`:48597-48600`,
    // `OnSubmitServerPasswordFromRequest` `:51944-51948`); vanilla
    // reaches the screen when a server demands a password
    // (`MessageBuffer.cs:2214`), the port always shows it after the
    // port screen (plan deviation).
    fn host_password_items(&self) -> Vec<Item> {
        vec![
            Item::new(crate::lang::menu(91), Act::None).heading(),
            // `array9[1] = Netplay.ServerPassword`, `*`-masked while
            // `Main.HidePassword` is set (`:48691-48707`).
            if self.blink_on {
                Item::new(format!("{}|", password_display(self)), Act::None)
                    .at(-20.0)
                    .heading()
                    .offset_x(1.0)
            } else {
                Item::new(format!("{} ", password_display(self)), Act::None).at(-20.0).heading()
            },
            Item::new(
                crate::lang::menu(4),
                Act::Out(Action::Host {
                    port: 7777, // `Netplay.ListenPort = 7777` (`Main.cs:52011`)
                    password: self.server_password.clone(),
                }),
            )
            .at(20.0),
            // Row 3 (`:48716-48720`): vanilla Back runs
            // `ExitServerPasswordMenu` (Steam 889, else the world select,
            // `:52018-52028`); the port returns to the multiplayer menu.
            Item::new(crate::lang::menu(5), Act::Nav(Page::Multiplayer)),
        ]
    }

    // menuMode 889 (`Main.cs:48726-48809`): `num2 = 200`, `num4 = 60`
    // (`:48728-48729`), `array4 = [_, 30, 30, 30, 70, 70]`
    // (`:48730-48734`). The `Lang.menu[135]` header (`:48736`,
    // `array[0] = true` at `:48739`), the Lobby toggle row
    // (`:48745`/`:48751`), two blank rows without Steam
    // (`:48743-48747`), `Lang.menu[144]` "Start" (`:48737`) and
    // "Back" (`:48738`).
    fn join_password_items(&self) -> Vec<Item> {
        vec![
            Item::new(crate::lang::menu(3), Act::None).heading(),
            if self.blink_on {
                Item::new(format!("{}|", password_display(self)), Act::None)
                    .at(-20.0)
                    .heading()
                    .offset_x(1.0)
            } else {
                Item::new(format!("{} ", password_display(self)), Act::None).at(-20.0).heading()
            },
            Item::new(
                crate::lang::menu(4),
                Act::Out(Action::Join {
                    // `OnSubmitServerPort` connects to `Main.getIP` on the
                    // parsed port (`Main.cs:52047-52054`).
                    addr: format!("{}:{}", self.get_ip, self.get_port),
                    password: self.join_password.clone(),
                }),
            )
            .at(20.0),
            // Row 3 (`:48592-48596`): vanilla Back runs
            // `CanceledGivingServerPassword` (back to menuMode 0 and
            // disconnects, `:51950-51957`); the port returns to the port
            // screen (plan deviation).
            Item::new(crate::lang::menu(5), Act::Nav(Page::JoinIp)),
        ]
    }

    // menuMode 5 (`Main.cs:49420-49439`): `array9[0] = Lang.menu[46] +
    // " " + Main.PlayerList[Main.selectedPlayer].Player.name + "?"`
    // with `array[0] = true` (`:49421-49422`), rows 1-2
    // `Lang.menu[104]` "Yes" (`:49423`) and `Lang.menu[105]` "No"
    // (`:49424`), `num5 = 3` — the vanilla 250/80 defaults.
    fn server_settings_items(&self) -> Vec<Item> {
        vec![
            Item::new(crate::lang::menu(135), Act::None).heading(),
            // Row 1: `Lang.menu[136]` "Steam Multiplayer: Disabled", or
            // [137] "…: Enabled" while the Lobby bit is set. The click
            // XORs the flag (`:48784-48787`).
            Item::new(
                on_off(
                    self.server_lobby,
                    &crate::lang::menu(137),
                    &crate::lang::menu(136),
                ),
                Act::Toggle(Toggle::ServerLobby),
            )
            .at(30.0),
            // Rows 2-3 (`:48743-48747`): blank and non-clickable without
            // Steam — the vanilla lobby branch (`:48749-48777`) fills them
            // with the Allowed Users rows only under Steam networking,
            // which the port does not have (spec §9.1).
            Item::new("", Act::None).at(30.0).heading(),
            Item::new("", Act::None).at(30.0).heading(),
            // Row 4 (`:48737`, `array4[4] = 70`): "Start" clears the
            // password and opens the host password screen
            // (`:48796-48802`).
            Item::new(crate::lang::menu(144), Act::Nav(Page::HostPassword)).at(70.0),
            // Row 5 (`:48738`, `array4[5] = 70`): "Back" — vanilla opens
            // the world select (`:48804-48806`), the port returns to the
            // multiplayer menu (plan deviation).
            Item::new(crate::lang::menu(5), Act::Nav(Page::Multiplayer)).at(70.0),
        ]
    }

    // menuMode 9 (`Main.cs:49674-49692`): the world twin — the header
    // names `Main.WorldList[Main.selectedWorld].Name` (`:49675`), the
    // same Yes/No rows (`:49677-49678`).
    fn confirm_delete_player_items(&self) -> Vec<Item> {
        vec![
            Item::new(
                format!("Delete {}?", self.confirm_name.as_deref().unwrap_or_default()),
                Act::None,
            )
            .heading(),
            Item::new("Yes", Act::Out(Action::ConfirmYes)),
            Item::new("No", Act::Out(Action::ConfirmNo)),
        ]
    }

    // menuMode 40 (`Main.cs:49440-49467`): the discard confirm —
    // `UI.PlayerCreateDiscardWarningNoName` while the draft has no
    // name, `UI.PlayerCreateDiscardWarningWithName` ("Discard {0}?")
    // otherwise (`:49441-49448`), rows `…WarningYes`
    // "Yes, discard edits!" (`:49449`) and `…WarningNo`
    // "No, keep editing..." (`:49450`).
    fn confirm_delete_world_items(&self) -> Vec<Item> {
        vec![
            Item::new(
                format!("Delete {}?", self.confirm_name.as_deref().unwrap_or_default()),
                Act::None,
            )
            .heading(),
            Item::new("Yes", Act::Out(Action::ConfirmYes)),
            Item::new("No", Act::Out(Action::ConfirmNo)),
        ]
    }

    // menuMode 3000 (`Main.cs:49693-49704`): `num5 = 1`, `num2 = 500`,
    // `array7[0] = 0.9f`, `array9[0] = Lang.menu[5]` "Back" — the
    // click (or `flag5`) returns to menuMode 0 (`:49700-49703`).
    fn confirm_discard_items(&self) -> Vec<Item> {
        vec![
            Item::new(
                match &self.confirm_name {
                    Some(name) => format!("Discard {name}?"),
                    None => "Discard your new player?".to_string(),
                },
                Act::None,
            )
            .heading(),
            Item::new("Yes, discard edits!", Act::Out(Action::ConfirmYes)),
            Item::new("No, keep editing...", Act::Out(Action::ConfirmNo)),
        ]
    }

    fn credits_items(&self) -> Vec<Item> {
        vec![Item::new(crate::lang::menu(5), Act::Nav(Page::Main)).scaled(0.9)]
    }
}
