//! The App's settings/UI-preset glue — the `TR_UI` screen presets for
//! screenshots and the immediate renderer settings apply. (A6: extracted
//! from `main.rs`.)

use crate::app::flow::{FrontScreen, TrUiPreset};
use crate::app::App;
use crate::menu;
use crate::ui;

impl App {
    /// Applies a `TR_UI` preset (App init): the screens re-list through the
    /// real openers; the legacy pages park the front exactly as their vanilla
    /// entries do — modes 5/9 clear the 888 state (`Main.cs:51066-51069`'s
    /// else branch), mode 40 parks it in `front_backup`
    /// (`UICharacterCreation.BackupConfirmationState`).
    pub(crate) fn preset_ui(&mut self, preset: TrUiPreset) {
        match preset {
            TrUiPreset::CharSelect => self.enter_front(FrontScreen::CharSelect),
            TrUiPreset::WorldSelect => self.enter_front(FrontScreen::WorldSelect),
            // Task 7's screen: `NewCharacterClick`'s fresh draft
            // (`UICharacterSelect.cs:108-114`).
            TrUiPreset::CharNew => {
                self.enter_front(FrontScreen::CharCreation(
                    ui::char_creation::CreationState::new(),
                ));
            }
            // Task 8's screen (`NewWorldClick`, `UIWorldSelect.cs:110-116`):
            // the stand-in name reads `Main.WorldList.Count + 1`, so the list
            // populates first; the Journey default reads the session player
            // (`UIWorldCreation.cs:319-322`).
            TrUiPreset::WorldNew => {
                self.enter_front(FrontScreen::WorldSelect);
                let count = self.worlds.len();
                let active = self.session_entry().difficulty;
                self.enter_front(FrontScreen::WorldCreation(
                    ui::world_creation::NewWorldDraft::new(count, active),
                ));
            }
            TrUiPreset::Keyboard => {
                // The keyboard screenshot (`TR_UI=vk`): the naming request the
                // character creator opens with (`Lang.menu[45]`, empty text —
                // `Player.cs:826`'s fresh name, so the Shift state seeds),
                // Cancel returning to the char select. `TR_VK_GRID=1` shows
                // the 50-key grid layer instead (Review Focus 6).
                self.open_keyboard(
                    ui::keyboard::Request {
                        label: ui::keyboard::enter_character_name(),
                        initial: String::new(),
                        allow_empty: true,
                        max_length: 20,
                        context: ui::keyboard::Context::Text,
                    },
                    FrontScreen::CharSelect,
                );
            }
            TrUiPreset::Credits => {
                self.front = None;
                self.front_backup = None;
                self.menu = Some(menu::Menu::open_credits(self.settings.clone()));
            }
            TrUiPreset::DelPlayer => {
                self.enter_front(FrontScreen::CharSelect);
                let name =
                    self.players.first().map(|p| p.name.clone()).unwrap_or_default();
                self.front = None;
                self.menu = Some(menu::Menu::open_confirm_delete_player(
                    self.settings.clone(),
                    name,
                ));
            }
            TrUiPreset::DelWorld => {
                self.enter_front(FrontScreen::WorldSelect);
                let name = self.worlds.first().map(|w| w.name.clone()).unwrap_or_default();
                self.front = None;
                self.menu = Some(menu::Menu::open_confirm_delete_world(
                    self.settings.clone(),
                    name,
                ));
            }
            TrUiPreset::Discard => {
                // Mode 40's screenshot: the parked state is the character
                // creation — vanilla parks the live `UICharacterCreation`
                // (`BackupConfirmationState`, `UICharacterCreation.cs:1215`),
                // named so the header shows `Discard {0}?`.
                let mut state = ui::char_creation::CreationState::new();
                state.draft.name = "Zealot".into();
                state.draft.refresh_dirty();
                self.front = None;
                self.front_backup = Some(FrontScreen::CharCreation(state));
                self.menu = Some(menu::Menu::open_confirm_discard(
                    self.settings.clone(),
                    Some("Zealot".into()),
                ));
            }
        }
    }

    /// Applies the options the renderer reads right away.
    pub(crate) fn apply_settings(&mut self) {
        // `PlayerInput.CurrentProfile`'s `KeyStatus` re-seed (`Main.cs:4602`):
        // the settings screen's edits become the live table.
        self.keybinds = self.settings.keybinds.clone();
        self.map_binds = self.settings.map_binds.clone();
        self.light_mode = tr_core::lighting::LightMode::ALL
            [(self.settings.light_mode as usize).min(3)];
        if !self.zoom_pinned {
            self.zoom = self.settings.zoom;
        }
        // The light cache and the world range depend on the mode.
        self.world_key = None;
    }
}
