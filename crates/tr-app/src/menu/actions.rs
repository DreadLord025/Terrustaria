//! The menu's input semantics — the [`Action`]/[`Toggle`]/[`Act`] row
//! behaviours and the keyboard/activate handlers behind them (A7:
//! extracted from `menu.rs`).

use super::pages::{
    cycle_previous, default_keybinds, default_map_binds, port_is_valid,
    MINIMAP_FRAME_KEYS, RESOURCE_SET_KEYS,
};
use super::{Menu, Page};

pub enum Action {
    None,
    Singleplayer,
    Host { port: u16, password: String },
    Join { addr: String, password: String },
    Quit,
    /// The confirm pages' Yes row (`selectedMenu == 1`, `Main.cs:49431`,
    /// `:49681`, `:49454`) — the App runs the erase and reopens the screen the
    /// confirm came from.
    ConfirmYes,
    /// The confirm pages' No row (`selectedMenu == 2`) — also the target of
    /// the screens' gamepad Back, which the port maps to Escape (`flag5`
    /// ORs into the No branch, `Main.cs:49435-49438`, `:49687-49690`,
    /// `:49464-49467`).
    ConfirmNo,
}

/// One settings row. Every variant is a `Main`/`ItemSlot` field the screens toggle.
#[derive(Clone, Copy)]
pub(super) enum Toggle {
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
    // The menuMode 889 "Steam Multiplayer" row (`Main.cs:48784-48787`): the
    // `ServerMode.Lobby` toggle. Menu state, not a config setting.
    ServerLobby,
}

pub(super) enum Act {
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

impl Menu {
    pub(super) fn parent(page: Page) -> Option<Page> {
        match page {
            Page::Main => None,
            Page::Settings => Some(Page::Main),
            Page::General | Page::Interface | Page::Video => Some(Page::Settings),
            Page::Multiplayer => Some(Page::Main),
            // The Escape targets of the Task 2-6 screens — for the multiplayer
            // chain these are the port's own Back rows, which vanilla routes
            // through the character/world selects instead (Task 6 deviations).
            Page::Volume | Page::Cursor | Page::Language | Page::Controls
            | Page::Keybindings | Page::MapKeys => Some(Page::Settings),
            Page::Parallax | Page::Resolution | Page::Effects => Some(Page::Video),
            Page::CursorColor | Page::BorderColor => Some(Page::Cursor),
            Page::LanguageInitial => None, // the first-launch picker has no Back
            Page::JoinIp | Page::HostPassword | Page::ServerSettings => Some(Page::Multiplayer),
            Page::JoinPort => Some(Page::JoinIp),
            Page::JoinPassword => Some(Page::JoinPort),
            // The confirms have no Back row and no parent: their nav happens
            // through the Yes/No actions, Escape included — vanilla's
            // gamepad Back (`flag5`) ORs into the No branch
            // (`Main.cs:49435-49438`, `:49687-49690`, `:49464-49467`).
            Page::ConfirmDeletePlayer | Page::ConfirmDeleteWorld | Page::ConfirmDiscard => None,
            // menuMode 3000's Back row (`flag5` included) returns to
            // menuMode 0 (`Main.cs:49700-49703`).
            Page::Credits => Some(Page::Main),
        }
    }

    pub(super) fn activate(&mut self, index: usize) -> Action {
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
                // The confirm pages' rows: the App resolves the page against
                // its front state (the erase and the reopen, `Main.cs
                // :49431-49439`, `:49681-49692`, `:49454-49467`).
                Action::ConfirmYes => Action::ConfirmYes,
                Action::ConfirmNo => Action::ConfirmNo,
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
    pub(super) fn nav_to(&mut self, page: Page) {
        self.page = page;
        self.selected = 0;
        // Vanilla clears the password on every entry to the password screens:
        // 31 is entered only through the demand, which resets
        // `Netplay.ServerPassword` first (`MessageBuffer.cs:2212-2214`), and 30
        // only through the 889 Start row, which does the same
        // (`Main.cs:48797-48799`).
        match page {
            Page::HostPassword => self.server_password.clear(),
            Page::JoinPassword => self.join_password.clear(),
            _ => {}
        }
        // A grab never follows a page change (vanilla resets `rightLock`
        // in the `!Main.mouseLeft` guard of whichever screen runs next).
        self.bar_grab = None;
        self.error = None;
    }

    /// `Main`/`ItemSlot` toggles. Every row changes the stored setting; the ones the
    /// renderer reads (`LightMode`, `BackgroundEnabled`, `MapEnabled`, …) are applied by
    /// `App` when the menu closes.
    pub(super) fn toggle(&mut self, toggle: Toggle) {
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
            // The menuMode 889 Lobby row XORs `Main.MenuServerMode & Lobby`
            // (`Main.cs:48784-48787`) — a `Main` static, not a config value,
            // so nothing is persisted.
            Toggle::ServerLobby => {
                self.server_lobby = !self.server_lobby;
                return;
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
            K::Backspace => {
                // `Main.GetInputText` pops the last character of the string the
                // screen edits (`Main.cs:50793`, `:50898`, `:48672`, `:48548`).
                if !self.connecting {
                    let target = match self.page {
                        Page::JoinIp => Some(&mut self.get_ip),
                        Page::JoinPort => Some(&mut self.get_port),
                        Page::HostPassword => Some(&mut self.server_password),
                        Page::JoinPassword => Some(&mut self.join_password),
                        _ => None,
                    };
                    if let Some(target) = target {
                        target.pop();
                    }
                }
                return Action::None;
            }
            K::Escape => {
                // The confirm pages walk no parent: vanilla's gamepad Back
                // (`flag5`) ORs into the No branch (`Main.cs:49435-49438`,
                // `:49687-49690`, `:49464-49467`), so Escape resolves the No
                // row and the App handles it per page.
                if matches!(
                    self.page,
                    Page::ConfirmDeletePlayer | Page::ConfirmDeleteWorld | Page::ConfirmDiscard
                ) {
                    return self.activate(2);
                }
                if let Some(parent) = Self::parent(self.page) {
                    self.page = parent;
                    self.selected = 0;
                    self.hint = None;
                    self.bar_grab = None;
                }
                return Action::None;
            }
            K::Enter | K::NumpadEnter => {
                // The input screens submit on Enter no matter which row the
                // cursor sits on (`Main.inputTextEnter`): 13 calls
                // `OnSubmitServerIP` (`:50865-50868` — the `!array2[2]` of the
                // guard is a vanilla quirk that never disables it, since
                // `array2[2]` is only ever set on 131), 131 keeps its
                // `!array2[2]` gate (`:50960-50963`), and 30/31 submit
                // unconditionally (`:48721-48724`, `:48597-48600`).
                match self.page {
                    Page::JoinIp => {
                        self.submit_join_ip();
                        return Action::None;
                    }
                    Page::JoinPort => {
                        if port_is_valid(&self.get_port) {
                            self.nav_to(Page::JoinPassword);
                        }
                        return Action::None;
                    }
                    Page::HostPassword => return self.host_and_play(),
                    Page::JoinPassword => return self.send_join_password(),
                    _ => {}
                }
            }
            _ => return Action::None,
        }
        self.activate(self.selected)
    }

    /// `Main.OnSubmitServerIP` (`Main.cs:52056-52062`): store the text, then
    /// `menuMode = 131` and `clrInput()` (the key-state reset, `:20001-20004`).
    /// No validation — the vanilla quirk above lets a blank IP through.
    fn submit_join_ip(&mut self) {
        self.nav_to(Page::JoinPort);
    }

    /// `Main.HostAndPlay` (`Main.cs:51972-52016`) as the port's host action:
    /// the listen port is fixed at 7777 (`:52011`); the password travels with
    /// the request (`:48677` seeds it from `Netplay.ServerPassword`).
    fn host_and_play(&mut self) -> Action {
        Action::Host { port: 7777, password: self.server_password.clone() }
    }

    /// `Main.OnSubmitServerPasswordFromRequest` (`Main.cs:51944-51948`): the
    /// join continues with the typed password against the address collected on
    /// 13/131 (`OnSubmitServerPort`, `:52047-52054`).
    fn send_join_password(&mut self) -> Action {
        Action::Join {
            addr: format!("{}:{}", self.get_ip, self.get_port),
            password: self.join_password.clone(),
        }
    }
}
