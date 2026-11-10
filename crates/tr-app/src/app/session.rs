//! The App's save-session glue — the creation tails (`FinishCreating-
//! Character`/`FinishCreatingWorld`), the implicit session character as a
//! card entry and the erase handlers. (A6: extracted from `main.rs`.)

use crate::app::App;
use crate::app::flow::FrontScreen;
use crate::ui;

impl App {
    /// `FinishCreatingCharacter` (`UICharacterCreation.cs:1704-1711`) minus
    /// the out-of-scope pieces: `TryAutoAssigningHair` (the cloth-style
    /// default, `:631-637`), then the stat/inventory setup is the fresh
    /// player the port already runs, and `Main.LoadPlayers` + `menuMode = 1`
    /// reopen the character select through the real opener. The draft also
    /// saves through `PlayerFileData.CreateAndSave` (`PlayerFileData.cs:50-63`)
    /// as a real `.plr`: the file carries the fresh-player defaults (life 100,
    /// mana 20) and the ctor's starter items (`Player.cs:54356-54360`).
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
        // The live `favorite_slots` ride the new `.plr` bitfield
        // (`Player.cs:54820-54824`) so a re-loaded character keeps its stars
        // (BUG-03).
        save_created_character(&draft, self.favorite_bits());
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
    /// (`FinishCreatingCharacter`'s port). No `.plr` backs it directly, so it
    /// is not deletable (`Main.ErasePlayer` needs a file) and always
    /// playable; a same-named saved file replaces it in the list.
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
            path: None,
            // The session character's appearance: the creation draft's
            // colours when the session came out of character creation
            // (`Main.PendingPlayer` carries them, `UICharacterSelect.cs:110`),
            // else `new Player()`'s initializers (`Player.cs:2554-2566`).
            // The session character's favourites live on the `App`
            // (`favorite_slots`); the file's bitfield hydrates the disk
            // cards (`Main.PendingPlayer`'s slots), so the stand-in starts
            // empty here.
            inv_favorites: 0,
            appearance: match &self.session_draft {
                Some(d) => ui::save_list::Appearance {
                    hair: d.hair,
                    skin_variant: d.skin_variant,
                    colors: [
                        [d.hair_color[0], d.hair_color[1], d.hair_color[2]],
                        [d.skin_color[0], d.skin_color[1], d.skin_color[2]],
                        [d.eye_color[0], d.eye_color[1], d.eye_color[2]],
                        [d.shirt_color[0], d.shirt_color[1], d.shirt_color[2]],
                        [d.undershirt_color[0], d.undershirt_color[1], d.undershirt_color[2]],
                        [d.pants_color[0], d.pants_color[1], d.pants_color[2]],
                        [d.shoe_color[0], d.shoe_color[1], d.shoe_color[2]],
                    ],
                },
                None => ui::save_list::Appearance::default(),
            },
        }
    }

    /// `Main.ErasePlayer` (`Main.cs:4772-4791`): a disk-backed card deletes
    /// its `.plr` (the `.bak` and the folder side ride along in
    /// `save_list::erase_player`); a disk-less entry (the session stand-in or
    /// a fake) drops in memory only — the reopen's re-list rebuilds from the
    /// (unchanged) disk state.
    pub(crate) fn erase_player_entry(&mut self, index: usize) {
        let path = self.players.get(index).and_then(|e| e.path.clone());
        let _ = ui::save_list::erase_player(path.as_deref());
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

/// `PlayerFileData.CreateAndSave` (`PlayerFileData.cs:50-63`) — the finished
/// draft saves under `Main.PlayerPath` via `Main.GetPlayerPathFromName`
/// (`Main.cs:4820`). The session inventory stays as the port runs it; the
/// file records the vanilla fresh defaults.
fn save_created_character(draft: &ui::char_creation::NewPlayerDraft, favorites: u64) {
    let mut inventory = [(0i32, 0i32, 0u8); 58];
    inventory[0] = (3507, 1, 0); // Copper Shortsword (`Player.cs:54356`)
    inventory[1] = (3509, 1, 0); // Copper Axe (`:54358`)
    inventory[2] = (3506, 1, 0); // Copper Pickaxe (`:54360`)
    let file = crate::player_file::PlayerFileWrite {
        name: draft.name.clone(),
        difficulty: draft.difficulty,
        skin_variant: draft.skin_variant as u8,
        hair: draft.hair,
        life: 100,     // `Player.cs:1924`
        life_max: 100, // `:1924`
        mana: 20,      // `Player.cs:56075`
        mana_max: 20,  // `:56075`
        hair_color: rgb(draft.hair_color),
        skin_color: rgb(draft.skin_color),
        eye_color: rgb(draft.eye_color),
        shirt_color: rgb(draft.shirt_color),
        undershirt_color: rgb(draft.undershirt_color),
        pants_color: rgb(draft.pants_color),
        shoe_color: rgb(draft.shoe_color),
        inventory,
        // `Item.favorited`'s bitfield (`Player.cs:54820-54824`) — the App's
        // live `favorite_slots`, flattened (`App::favorite_bits`); a fresh
        // character has none.
        inv_favorites: favorites,
    };
    match tr_core::save::default_player_path(&draft.name) {
        Some(path) => {
            if let Err(e) = crate::player_file::write_player_file(&path, &file) {
                log::warn!("character save failed: {e}");
            }
        }
        None => log::warn!("no players dir; '{}' not saved", draft.name),
    }
}

/// `Color.R/G/B` — the writer stores the three bytes, the draft carries RGBA.
fn rgb(c: [u8; 4]) -> [u8; 3] {
    [c[0], c[1], c[2]]
}
