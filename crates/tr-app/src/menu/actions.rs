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
    /// `Lang.menu[131]`'s row (`Main.cs:48928-48935`): `menuMode = 888` +
    /// `Main.MenuUI.SetState(Main.AchievementsMenu)` — the port opens the
    /// `UIAchievementsMenu` overlay over the live menu (M6-ACH-2).
    Achievements,
    /// menuMode 11 row 4's no-Steam branch (`Main.cs:48949-48957`):
    /// `UI.ResourcePacks` → `Main.OpenResourcePacksMenu(null)` — the port
    /// opens `FrontScreen::ResourcePackSelection` (W4-d84 row 5).
    ResourcePacks,
    /// The in-world keybinds screen's Back/Escape (`IngameFancyUI.Close`,
    /// `IngameFancyUI.cs:162-188`): the menu closes and gameplay resumes —
    /// only reachable on the menu the options window's Keybinds row opened
    /// (`Menu::in_game_keybinds`).
    ReturnToWorld,
    /// W4-g22 DOC-SWEEP — the connecting overlay's Cancel
    /// (`Main.cs:48615-48635` menuMode 14 / `:48818-48825` menuMode 15):
    /// `Netplay.Disconnect = true` + the socket close, `menuMode = 0`,
    /// `netMode = 0`. The 11 (`:48621`/`:48822`) rides `pending_cue`.
    CancelConnect,
    /// W4-g22 DOC-SWEEP — menuMode 200's "Load Backup" row
    /// (`Main.cs:48837-48853`): the `.bak` probe decides — restore + 10 +
    /// `WorldGen.playWorld` (`:48843-48845`) or 11 + menuMode 0
    /// (`:48849-48851`). The App owns the file moves.
    LoadBackup,
    /// W4-g22 DOC-SWEEP — menuMode 200's Cancel / 201's Back
    /// (`Main.cs:48854-48860`, `:48873-48879`): 11 + menuMode 0 +
    /// netMode 0. The `flag5` OR rides the same branch.
    CancelLoadFailed,
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
    /// The language rows (`Main.cs:48519-48523`, `:48484-48490`):
    /// `LanguageManager.SetLanguage(legacy id)` + `Main.SaveSettings()`, the
    /// row index being the `GameCulture` legacy id (1..=12).
    SetLanguage(i32),
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
            // W4-g22 DOC-SWEEP — 200/201 walk no parent: the gamepad Back
            // ORs into the Cancel/Back row (the `activate(2)` arm above).
            Page::LoadFailedBackup | Page::LoadFailedNoBackup => None,
        }
    }

    pub(super) fn activate(&mut self, index: usize) -> Action {
        let items = self.items();
        let Some(item) = items.get(index) else {
            return Action::None;
        };
        // W4-g22 DOC-SWEEP — while the connecting overlay is live the C#
        // draws ONLY the status + Cancel rows (`Main.cs:48602-48635`,
        // menuMode 14 / netMode 1): every click lands on Cancel, which
        // disconnects and plays 11 (`:48621`). The port keeps the page's
        // rows drawn under the overlay (deviation) but the click answers
        // the same Cancel — BEFORE the header gate, since the overlay's
        // own rows replace the page's headers too.
        if self.connecting {
            self.pending_cue = Some("Menu_Close"); // `:48621`/`:48822`
            return Action::CancelConnect;
        }
        if item.header {
            return Action::None;
        }
        self.hint = None;
        // W4-g22 — every activation starts cue-less; the arms below queue
        // the C# site's own `PlaySound` (the g22 window `Main.cs:48489-
        // 49727`). The App drains `pending_cue` after the click/key.
        self.pending_cue = None;
        match &item.act {
            Act::None => Action::None,
            Act::Nav(page) => {
                // The in-world keybinds screen's Back row returns to gameplay
                // (`IngameFancyUI.Close`) instead of opening menuMode 11.
                if self.in_game_keybinds {
                    // W4-g22 — `IngameFancyUI.Close`'s loud arm plays 11
                    // (`IngameFancyUI.cs:167`).
                    self.pending_cue = Some("Menu_Close");
                    return Action::ReturnToWorld;
                }
                // W4-g22 — the nav cue is the C# site's own `PlaySound`:
                // forward opens play 10 (`:48917`/`:48925`/`:48962`/
                // `:49720`/`:49727`/`:48802`), Back rows play 11 (`:48516`/
                // `:48806`/`:49200`/`:49265`/`:49702`). A Back row is
                // `Act::Nav(parent)` — the CURRENT page's parent is the
                // target. The exceptions are per-site, not per-relation:
                //  * the Settings→Volume row plays 11 (`:49741`, the C#
                //    quirk — a forward open with the close cue);
                //  * the input screens' Accept rows play NOTHING
                //    (`OnSubmitServerPort` `:52047-52054` and the password
                //    submits `:51944-51948`/`HostAndPlay` `:51972-52018`
                //    have no `PlaySound`) — EXCEPT the IP Accept, below;
                //  * the input screens' Back rows play 11 in vanilla
                //    (`:50862`/`:50957`, `CanceledGivingServerPassword`
                //    `:51952`, `ExitServerPasswordMenu` `:52020-52031` is
                //    SILENT) — the port's Back targets are documented
                //    deviations but the cue follows the vanilla row.
                // W4-g23 union — the IP Accept row (`:50865-50868`,
                // `selectedMenu == 9`) calls `OnSubmitServerIP`, whose
                // `PlaySound(12)` (`:52059`) is g23's tail site. g22 left
                // this arm cue-less for g23's seam; the union lands it on
                // THIS queue (one activation drain), the Enter twin queues
                // the same id in `submit_join_ip`.
                let ip_submit = self.page == Page::JoinIp && *page == Page::JoinPort;
                let back = Self::parent(self.page) == Some(*page);
                let accept = self.page == Page::JoinPort && *page == Page::JoinPassword;
                // W4-g23 union fix — the JoinPassword Back row targets
                // `Page::JoinIp` (`pages.rs`, the documented plan deviation;
                // g22's tuple named `JoinPort`, which no row produces). The
                // cue is `CanceledGivingServerPassword`'s 11 (`:51952`,
                // `:48592-48596`) either way.
                let password_back = matches!(
                    (self.page, *page),
                    (Page::JoinIp, Page::Multiplayer)
                        | (Page::JoinPort, Page::JoinIp)
                        | (Page::JoinPassword, Page::JoinIp)
                );
                // The host-password Back (`ExitServerPasswordMenu`,
                // `:52020-52031`) plays NOTHING; the port's Back row targets
                // the multiplayer page (deviation) and stays silent.
                let host_password_back =
                    self.page == Page::HostPassword && *page == Page::Multiplayer;
                // W4-g23 union — the two Video-page forward opens that play
                // the CLOSE cue in vanilla: Parallax (`:50002`) and Effects
                // (`:50075`), the `:49741` Volume quirk's twins the g22
                // census diff surfaced (the 75-site replay).
                let close_quirk = (self.page == Page::Settings && *page == Page::Volume)
                    || (self.page == Page::Video
                        && matches!(*page, Page::Parallax | Page::Effects));
                self.pending_cue = if ip_submit {
                    Some("Menu_Tick") // sound 12, `:52059`
                } else if accept || host_password_back {
                    None
                } else if back || password_back || close_quirk {
                    Some("Menu_Close") // sound 11
                } else {
                    Some("Menu_Open") // sound 10
                };
                self.nav_to(*page);
                Action::None
            }
            Act::PickColor(page) => {
                // `Main.selColor = Main.mouseColor; ... Main.menuMode = 25;`
                // (`Main.cs:50254-50258`) / `Main.selColor =
                // Main.MouseBorderColor; ... Main.menuMode = 252;`
                // (`:50263-50267`). The `ColorSlidersSet.SetHSL` calls there
                // are gamepad chrome and are not ported.
                // W4-g22 — both opens play 10 (`:50256`/`:50265`).
                self.pending_cue = Some("Menu_Open");
                self.sel_color = match page {
                    Page::CursorColor => self.settings.mouse_color,
                    _ => self.settings.mouse_border_color,
                };
                self.nav_to(*page);
                Action::None
            }
            Act::Out(action) => match action {
                // W4-g22 — `Main.cs:48915-48919`: the Singleplayer row plays
                // 10 BEFORE `menuMode = 1` (the char-select open).
                Action::Singleplayer => {
                    self.pending_cue = Some("Menu_Open");
                    Action::Singleplayer
                }
                // `Main.cs:48973-48978`: the Quit row sets
                // `GameAskedToQuit = true` — NO `PlaySound`.
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
                // W4-g22 — Yes plays 10 (`:49430`/`:49457`/`:49684`), No
                // plays 11 (`:49436`/`:49463`/`:49689`); the gamepad-Back
                // `flag5` OR rides the No branch, so Escape queues 11 too.
                Action::ConfirmYes => {
                    self.pending_cue = Some("Menu_Open");
                    Action::ConfirmYes
                }
                Action::ConfirmNo => {
                    self.pending_cue = Some("Menu_Close");
                    Action::ConfirmNo
                }
                // `Lang.menu[131]`'s row: the App opens the
                // `UIAchievementsMenu` overlay (M6-ACH-2).
                // W4-g22 — `Main.cs:48930-48934`: the row plays 10.
                Action::Achievements => {
                    self.pending_cue = Some("Menu_Open");
                    Action::Achievements
                }
                // menuMode 11 row 4 (`Main.cs:48951-48956`): `SoundEngine.
                // PlaySound(10)` then `OpenResourcePacksMenu(null)` — the App
                // opens the pack-selection screen (W4-d84).
                Action::ResourcePacks => {
                    self.pending_cue = Some("Menu_Open"); // `:48954`
                    Action::ResourcePacks
                }
                // No row carries it — only the in-game keybinds' Back/Escape
                // arms produce it. W4-g22 — `IngameFancyUI.Close`'s loud arm
                // plays 11 (`IngameFancyUI.cs:167`).
                Action::ReturnToWorld => {
                    self.pending_cue = Some("Menu_Close");
                    Action::ReturnToWorld
                }
                // No row carries it — the connecting-overlay guard above
                // `activate` produces it (W4-g22 DOC-SWEEP).
                Action::CancelConnect => Action::CancelConnect,
                // W4-g22 DOC-SWEEP — menuMode 200's Load Backup
                // (`Main.cs:48837-48853`): the cue is per-branch (10 when
                // the `.bak` exists, 11 when it does not) and the App
                // owns the probe + the file moves, so the row queues
                // NOTHING here — the App plays after the probe.
                Action::LoadBackup => Action::LoadBackup,
                // W4-g22 DOC-SWEEP — 200's Cancel / 201's Back play 11
                // (`:48857`/`:48876`) before menuMode 0.
                Action::CancelLoadFailed => {
                    self.pending_cue = Some("Menu_Close");
                    Action::CancelLoadFailed
                }
            },
            Act::Toggle(toggle) => {
                // W4-g22 — every settings toggle row plays 12 before the
                // flip (the menuMode 112/1112/28/2008/1125/1127 arms, e.g.
                // `Main.cs:49796`/`:49817`/`:50116`/`:50274`/`:50666`); the
                // 889 lobby XORs are the same cue (`:48786`/`:48790`/
                // `:48794`). The cycle rows that play NOTHING in the C#
                // (`:49945` MinimapFrame, `:49951` ResourceSets, `:49970`
                // BossBarText — the cycle helpers draw no sound) keep the
                // cue-less activation; the port's invented Zoom row has no
                // vanilla twin (zoom is keyboard-only, `:16808-16812`).
                if !matches!(
                    toggle,
                    Toggle::MinimapFrame | Toggle::ResourceSets | Toggle::BossBarText | Toggle::Zoom
                ) {
                    self.pending_cue = Some("Menu_Tick");
                }
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
                // W4-g23 union — BOTH Reset rows play 11 (`:50458` map /
                // `:50563` gameplay — the 75-site census rows g22's window
                // ended before); the restore itself draws nothing.
                self.pending_cue = Some("Menu_Close");
                if *map {
                    self.map_binds = default_map_binds();
                } else {
                    self.keybinds = default_keybinds();
                }
                self.set_key = None;
                Action::None
            }
            Act::Hint(message) => {
                // W4-g23 union — the not-ported rows keep their vanilla
                // activation cue (the C# plays BEFORE the unported body;
                // the cue follows the vanilla row even where the port's
                // action deviates — g22's Back-target principle):
                //  * Resolution cycle `:50181` and Borderless `:50201` play
                //    12; Apply `:50225` plays 11 (its `menuMode = 1111` hop
                //    stays unported with the SetResolution body — the row
                //    keeps the notice, the cue is the C#'s);
                //  * ManageControls `:50686` and Steam join `:50726` play 10.
                // The Fullscreen row (`:50206-50210`) delegates to
                // `ToggleFullScreen` → `SetDisplayMode` (`:66253-66432`),
                // which has NO `PlaySound` — it stays cue-less.
                match (self.page, message.contains("Resolution switching")) {
                    (Page::Resolution, true) => {
                        self.pending_cue = Some("Menu_Close"); // `:50225`
                    }
                    (Page::Resolution, false) if !message.contains("Fullscreen") => {
                        self.pending_cue = Some("Menu_Tick"); // `:50181`/`:50201`
                    }
                    (Page::Controls | Page::Multiplayer, _) => {
                        self.pending_cue = Some("Menu_Open"); // `:50686`/`:50726`
                    }
                    _ => {}
                }
                self.hint = Some((*message).to_string());
                Action::None
            }
            Act::SetLanguage(legacy_id) => {
                // `Main.changeTheTitle = true; SetLanguage; SaveSettings`
                // (`Main.cs:48519-48523`). 1213 stays on the language page so
                // the rows redraw in the new language; the first-launch 1212
                // jumps to the title (`Main.cs:48484-48490`).
                // W4-g22 — the cue splits by page: the first-launch picker
                // plays 10 (`:48489`), the settings language page plays 12
                // (`:48522`).
                self.pending_cue = Some(if self.page == Page::LanguageInitial {
                    "Menu_Open"
                } else {
                    "Menu_Tick"
                });
                let culture = crate::lang::culture_from_legacy_id(*legacy_id);
                crate::lang::set_language(culture);
                self.settings.language = culture.name().to_string();
                if self.persist {
                    let _ = self.settings.save();
                }
                if self.page == Page::LanguageInitial {
                    self.nav_to(Page::Main);
                }
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
        //
        // The name is the XNA `Keys` spelling (`string.Concat(pressedKeys[0])`,
        // `:50572`), not winit's debug name: the same string is what
        // `config.json` carries and what [`crate::input_map`] matches against.
        if self.set_key.is_some() && matches!(self.page, Page::Keybindings | Page::MapKeys) {
            let index = self.set_key.unwrap();
            let name = crate::input_map::key_name(code);
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
                        // W4-g22 — the change-diff cue (`Main.cs:50794-50797`
                        // / `:50899-50902` / `:48549-48552` / `:48673-48676`):
                        // 12 iff the pop CHANGED the string; an empty-string
                        // backspace is silent.
                        if target.pop().is_some() {
                            self.pending_cue = Some("Menu_Tick");
                        }
                    }
                }
                return Action::None;
            }
            K::Escape => {
                // W4-g22 DOC-SWEEP — the connecting overlay's gamepad Back
                // (`flag5` ORs into the Cancel branch, `Main.cs:48615`):
                // disconnect + 11 (`:48621`), before every other arm.
                if self.connecting {
                    self.pending_cue = Some("Menu_Close");
                    return Action::CancelConnect;
                }
                // The in-world keybinds screen closes back to gameplay
                // (`IngameFancyUI.Close`'s GoBack).
                if self.in_game_keybinds {
                    // W4-g22 — the loud close plays 11
                    // (`IngameFancyUI.cs:167`).
                    self.pending_cue = Some("Menu_Close");
                    return Action::ReturnToWorld;
                }
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
                // W4-g22 DOC-SWEEP — 200/201's gamepad Back ORs into the
                // Cancel/Back row (`Main.cs:48854`/`:48873`'s `| flag5`),
                // the third row on both pages.
                if matches!(self.page, Page::LoadFailedBackup | Page::LoadFailedNoBackup) {
                    return self.activate(2);
                }
                if let Some(parent) = Self::parent(self.page) {
                    // W4-g22 — Escape stands in for the gamepad Back
                    // (`flag5`, `Main.cs:48439`): every Back arm it ORs into
                    // plays 11 (`:48516`/`:49702`/`:49770`/`:50085`/...).
                    self.pending_cue = Some("Menu_Close");
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
    /// W4-g23 union: the `PlaySound(12)` (`:52059`) rides g22's `pending_cue`
    /// (ONE activation drain) — the Enter path (`:50865-50868`) queues here;
    /// the Accept row queues the same id in `activate`'s `Act::Nav` arm.
    fn submit_join_ip(&mut self) {
        // `main_sounds_batch3::on_submit_server_ip_cue()` pins the id (12).
        self.pending_cue = Some("Menu_Tick"); // `:52059`
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
