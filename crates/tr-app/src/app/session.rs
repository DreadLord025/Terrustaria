//! The App's save-session glue — the creation tails (`FinishCreating-
//! Character`/`FinishCreatingWorld`), the implicit session character as a
//! card entry and the erase handlers. (A6: extracted from `main.rs`.)

use crate::app::flow::FrontScreen;
use crate::app::App;
use crate::ui;

impl App {
    /// `FinishCreatingCharacter` (`UICharacterCreation.cs:1704-1711`) minus
    /// the out-of-scope pieces: `TryAutoAssigningHair` (the cloth-style
    /// default, `:631-637`), then the stat/inventory setup is the fresh
    /// player the port already runs, `PlayerFileData.CreateAndSave` has no
    /// `.plr` writer (the draft records into `App.session_draft` — the
    /// documented deviation), and `Main.LoadPlayers` + `menuMode = 1` reopen
    /// the character select through the real opener.
    pub(crate) fn finish_creating_character(&mut self) {
        let Some(FrontScreen::CharCreation(state)) = self.front.as_ref() else {
            return;
        };
        let mut draft = state.draft.clone();
        if !state.hair_picked {
            // `TryAutoAssigningHair` (`:630-637`).
            if let Some(hair) =
                ui::char_creation::default_hairstyle_for_cloth_style(draft.skin_variant)
            {
                draft.hair = hair;
            }
        }
        draft.name = draft.name.trim().to_string();
        self.player_name = draft.name.clone();
        self.session_draft = Some(draft);
        self.front_backup = None;
        self.front = None;
        self.enter_front(FrontScreen::CharSelect);
    }

    /// `FinishCreatingWorld` (`UIWorldCreation.cs:845-863`) minus the
    /// out-of-scope tail: `WorldGen.CreateNewWorld`'s `UIWorldLoad` loading
    /// screen is not ported (spec §9.4), so the generated world saves and
    /// world select re-opens (`OpenWorldSelectUI`'s effect, `:52362-52366`).
    /// The metadata pieces the port's `.trwld` header has no fields for —
    /// GameMode/evil (`WorldFile.CreateMetadata`, `:848-851`) — are the
    /// documented deviation the listing reads as Classic.
    pub(crate) fn finish_creating_world(&mut self, draft: ui::world_creation::NewWorldDraft) {
        match ui::world_creation::create_world(&draft) {
            Ok(path) => log::info!("created '{}' -> {}", draft.name.trim(), path.display()),
            Err(e) => {
                // `FrontHint`'s not-ported notice carries the failure — the
                // vanilla flow has no error UI on this hop.
                self.front_hint = Some(format!("world creation failed: {e:#}"));
            }
        }
        self.front_backup = None;
        self.front = None;
        self.enter_front(FrontScreen::WorldSelect);
    }

    /// The implicit session character as a card entry: the fresh `Player`
    /// defaults — difficulty 0 (`Player.cs:1654`), life 100 (`:1924`), mana 20
    /// (`:56075`) — named by the port's seeded `player_name`, overridden by
    /// the recorded creation draft once one is created
    /// (`FinishCreatingCharacter`'s port). No `.plr` exists, so it is not
    /// deletable (`Main.ErasePlayer` needs a file) and always playable.
    pub(crate) fn session_entry(&self) -> ui::save_list::PlayerEntry {
        ui::save_list::PlayerEntry {
            name: self
                .session_draft
                .as_ref()
                .map_or_else(|| self.player_name.clone(), |d| d.name.clone()),
            difficulty: self.session_draft.as_ref().map_or(0, |d| d.difficulty),
            life_max: self.player.as_ref().map_or(100, |p| p.max_hp),
            mana_max: 20,
            is_favorite: false,
            playable: true,
            deletable: false,
        }
    }

    /// `Main.ErasePlayer` (`Main.cs:4772-4791`) for the port: no `.plr`
    /// exists (plan §9.4), so `erase_player(None)` drops the entry in memory
    /// only — the reopen's re-list rebuilds from the (unchanged) disk state,
    /// which is what the vanilla call's file deletion achieves.
    pub(crate) fn erase_player_entry(&mut self, index: usize) {
        let _ = ui::save_list::erase_player(None);
        if index < self.players.len() {
            self.players.remove(index);
        }
    }

    /// `Main.EraseWorld` (`Main.cs:4796-4813`): the `.trwld` and its sidecars
    /// go (`save_list::erase_world`); the reopen's re-list then reads the
    /// directory without it.
    pub(crate) fn erase_world_entry(&mut self, index: usize) {
        if let Some(path) = self.worlds.get(index).and_then(|w| w.path.clone()) {
            if let Err(e) = ui::save_list::erase_world(&path) {
                log::warn!("world erase failed: {e:#}");
            }
        }
    }
}
