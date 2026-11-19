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
            TrUiPreset::WorldLoad => {
                // The loading-screen screenshot (`TR_UI=worldload`): a
                // deterministic mid-generation snapshot — no worker thread
                // (the shot must be SHA256-stable), the progress latched at
                // 42.5% overall / 60% current with a real pass label. The
                // bar reads the corrupt-evil colour (the classic default).
                let mut state = ui::world_load::WorldLoadState::new();
                state.set_progress(Some(ui::world_load::GenerationProgress {
                    total_progress: 0.425,
                    value: 0.6,
                    message: crate::lang::text("LegacyWorldGen.11"),
                }));
                let (sw, sh) = self.screen_size();
                let _ = sw;
                state.update_layout(sh);
                self.front_backup = None;
                self.front = None;
                self.enter_front(FrontScreen::WorldLoad(Box::new(state)));
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
        // `TR_LIGHT_MODE` pins the mode for screenshots (env.rs:761): like
        // `TR_ZOOM`'s `zoom_pinned` guard below, the config value must not
        // clobber the pin — the boot `apply_settings()` (main.rs:199) runs
        // after `App::new` seeded the pin.
        let new_mode = crate::env::light_mode().unwrap_or(
            tr_core::lighting::LightMode::ALL[(self.settings.light_mode as usize).min(3)],
        );
        if new_mode != self.light_mode {
            // The `Lighting.Mode` setter (`Lighting.cs:29-61`) resets
            // `renderCount`; the settings path also clears the presented
            // values so no previous mode's light survives (S1-E-19).
            if let Some(engine) = self.legacy_light.as_mut() {
                engine.set_mode(new_mode);
                engine.clear();
            }
        }
        self.light_mode = new_mode;
        if !self.zoom_pinned {
            self.zoom = self.settings.zoom;
        }
        // The light cache and the world range depend on the mode.
        self.world_key = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use tr_core::lighting::LightMode;

    /// S1-E-19 wave B: `TR_LIGHT_MODE` survives `apply_settings`. The boot
    /// order is `App::new` (seeds the pin, `mod.rs:3361`) then
    /// `apply_settings` (`main.rs:199`); before the fix the settings read
    /// clobbered the pin back to the config's mode, so every per-mode
    /// screenshot came out Color-identical.
    #[test]
    fn s1_e_19b_light_mode_pin_survives_apply_settings() {
        let mut app = flow_app(None, vec![], vec![]);
        app.settings.light_mode = 0; // the port ordinal for Color
        let _env = crate::app::tr_hooks::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        unsafe { std::env::set_var("TR_LIGHT_MODE", "retro") };
        app.apply_settings();
        assert_eq!(app.light_mode, LightMode::Retro, "the pin wins");
        // A second apply (the settings-screen close path) keeps it too.
        app.settings.light_mode = 1;
        app.apply_settings();
        assert_eq!(app.light_mode, LightMode::Retro, "still pinned");
        unsafe { std::env::remove_var("TR_LIGHT_MODE") };
        // Without the pin the config value applies (port ordinal 1 = White).
        app.apply_settings();
        assert_eq!(app.light_mode, LightMode::White);
        app.settings.light_mode = 0;
        app.apply_settings();
        assert_eq!(app.light_mode, LightMode::Color, "config Color restores");
    }
}
