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

    /// `FinishCreatingWorld` (`UIWorldCreation.cs:977-1000`) →
    /// `WorldGen.CreateNewWorld` (`WorldGen.cs:6607-6625`): the generation
    /// runs on a worker task while `UIWorldLoad` draws the progress
    /// (`Main.MenuUI.SetState(new UIWorldLoad())`, `:6616`); the callback's
    /// `menuMode = 6` tail (`:6599-6602`) reopens the world select on the
    /// next frame (`Main.cs:49468-49472`). The port starts the job
    /// (`App.world_gen_job`) and enters the loading screen; the frame's
    /// `update_world_load` polls the result and hops.
    ///
    /// The metadata pieces the port's `.trwld` header has no fields for —
    /// GameMode/evil (`WorldFile.CreateMetadata`) — are the documented
    /// deviation the listing reads as Classic.
    pub(crate) fn finish_creating_world(&mut self, draft: ui::world_creation::NewWorldDraft) {
        // `CreateNewWorld`'s `UIWorldSelect.NewlyGeneratedWorld` latch
        // (`WorldGen.cs:6609`) is the world-select's "new" badge — the port
        // has no such badge (save_list reads the disk), so nothing parks.
        match crate::app::worldgen_job::WorldGenJob::start(&draft) {
            Some(job) => {
                // W4-g24 — `worldGenCallback`'s HEAD (`WorldGen.cs:6593`):
                // `SoundEngine.PlaySound(10)` before `GenerateWorld` runs.
                // Case 10 (`LegacySoundPlayer.cs:750-758`) draws NOTHING —
                // the 2D menu play rides no RNG stream. The completion tail
                // (`:6603`) already plays in `update_world_load`.
                self.audio.play("Menu_Open");
                let mut state = ui::world_load::WorldLoadState::new();
                state.flags = job.flags;
                state.evil_crimson = job.crimson;
                state.random_evil = job.random_evil;
                self.world_gen_job = Some(job);
                self.front_backup = None;
                self.front = None;
                self.enter_front(FrontScreen::WorldLoad(Box::new(state)));
            }
            None => {
                // The worlds dir cannot resolve — the port's own failure
                // notice (the vanilla `Main.WorldPath` always resolves).
                self.front_hint =
                    Some("world creation failed: cannot resolve the worlds directory".into());
                self.front_backup = None;
                self.front = None;
                self.enter_front(FrontScreen::WorldSelect);
            }
        }
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

    /// `UICharacterListItem.OnFinishedSettingName`'s `_data.Rename(newName)`
    /// (`:208-214` -> `PlayerFileData.Rename`, `IO/PlayerFileData.cs:207-214`):
    /// the trimmed name lands on the card's `.plr` through the read-modify-
    /// write (`player_file::rename_player_file` — the port has no live
    /// `Player` per card, so the file IS the rename target, research/19 M1).
    /// The session stand-in IS the live player (`PlayerFileData.Rename` sets
    /// `Player.name`), so its rename lands on `player_name`/`session_draft`
    /// and survives the `OpenCharacterSelectUI` re-list. A `TR_FAKE_CHARS`
    /// fake renames in memory only — the re-list rebuilds it (the same shape
    /// as the fake erase). A failed write keeps the old name and notices
    /// (the port's `front_hint`, the vanilla `FancyErrorPrinter` dialog is
    /// out of scope).
    pub(crate) fn rename_player_entry(&mut self, index: Option<usize>, new_name: &str) {
        let Some(i) = index else {
            return;
        };
        let (path, old_name) = match self.players.get(i) {
            Some(e) => (e.path.clone(), e.name.clone()),
            None => return,
        };
        if let Some(path) = path {
            if let Err(e) = crate::player_file::rename_player_file(&path, new_name) {
                log::warn!("character rename failed: {e}");
                self.front_hint = Some(format!("renaming failed: {e}"));
                return;
            }
            if let Some(entry) = self.players.get_mut(i) {
                entry.name = new_name.to_string();
            }
        } else if old_name == self.session_entry().name {
            // The live-player arm (`PlayerFileData.cs:209-212`): the session
            // character's name is the App's `player_name`/creation draft.
            self.player_name = new_name.to_string();
            if let Some(draft) = self.session_draft.as_mut() {
                draft.name = new_name.to_string();
            }
            if let Some(entry) = self.players.get_mut(i) {
                entry.name = new_name.to_string();
            }
        } else if let Some(entry) = self.players.get_mut(i) {
            entry.name = new_name.to_string();
        }
    }

    /// `UIWorldListItem.OnFinishedSettingName`'s `_data.Rename(newDisplayName)`
    /// (`:427-432` -> `WorldFileData.Rename` -> `WorldGen.RenameWorld`,
    /// `IO/WorldFileData.cs:444-453`, `WorldGen.cs:6721-6753`): the trimmed
    /// name lands in the world file's header (`save_list::rename_world` —
    /// the load-rename-save chain's port, the file keeps its stem). The
    /// in-memory card mirrors the write; the caller's `menuMode = 6` reopen
    /// re-lists from the rewritten header. A failed write keeps the old name
    /// and notices (the C#'s `loadFailed` arm re-lists without writing,
    /// `WorldGen.cs:6735-6743`).
    pub(crate) fn rename_world_entry(&mut self, index: Option<usize>, new_name: &str) {
        let Some(i) = index else {
            return;
        };
        let path = match self.worlds.get(i) {
            Some(e) => e.path.clone(),
            None => return,
        };
        if let Some(path) = path {
            if !ui::save_list::rename_world(&path, new_name) {
                log::warn!("world rename failed: {}", path.display());
                self.front_hint =
                    Some("renaming failed: the world file could not be rewritten".into());
                return;
            }
        }
        if let Some(entry) = self.worlds.get_mut(i) {
            entry.name = new_name.to_string();
        }
    }

    /// `UIWorldLoad.Update`'s Inventory-abort arm (`UIWorldLoad.cs:54-58`):
    /// `QueuedAbort = true` once (the `!QueuedAbort` gate) + sound 11. The
    /// key path calls this on the Inventory press edge; the flag reaches
    /// the worker through the job's atomic (`WorldGenerator.cs:433-437`).
    pub(crate) fn world_load_queue_abort(&mut self) {
        let Some(job) = self.world_gen_job.as_ref() else {
            return;
        };
        if job.queued_abort() {
            return; // `!WorldGenerator.CurrentController.QueuedAbort` (`:54`)
        }
        job.queue_abort();
        // `SoundEngine.PlaySound(11)` (`:57`).
        self.audio.play_menu("Menu_Close");
        // The screen's own flag mirrors the controller's (`:86`'s read).
        if let Some(FrontScreen::WorldLoad(state)) = self.front.as_mut() {
            state.queued_abort = true;
        }
    }

    /// The loading screen's per-frame update — `UIWorldLoad.Update`
    /// (`UIWorldLoad.cs:46-65`) + `Draw`'s progress latch (`:69`) + the
    /// `worldGenCallback` completion tail (`WorldGen.cs:6591-6605`). Runs
    /// from `frame_shared_setup` while `FrontScreen::WorldLoad` is live:
    ///
    /// - the screen-height lerp (`:60-61`),
    /// - the progress snapshot latch (`:69`, `:83-85`) + the abort-flag
    ///   mirror (`:86`),
    /// - the result poll: on completion the job drops and the world select
    ///   reopens (`menuMode = 6` → `Main.cs:49468-49472`); on abort the
    ///   partial world is discarded and the select reopens the same way
    ///   (`worldGenCallback`'s `:6599-6602` runs on both arms).
    pub(crate) fn update_world_load(&mut self) {
        if !matches!(self.front, Some(FrontScreen::WorldLoad(_))) {
            return;
        }
        let (_sw, sh) = self.screen_size();
        // The result poll first — a finished job leaves the screen.
        let result = self.world_gen_job.as_ref().and_then(|j| j.take_result());
        if let Some(result) = result {
            use crate::app::worldgen_job::JobResult;
            match result {
                JobResult::Saved(path) => {
                    log::info!("created world -> {}", path.display());
                }
                JobResult::Aborted => {
                    log::info!("world generation aborted");
                }
                JobResult::Failed(e) => {
                    self.front_hint = Some(format!("world creation failed: {e}"));
                }
            }
            // `worldGenCallback`'s tail (`WorldGen.cs:6599-6604`): the
            // menu hops to 6, which `UpdateMenu` turns into the world
            // select (`Main.cs:49468-49472`); `SoundEngine.PlaySound(10)`
            // (`:6603`).
            self.world_gen_job = None;
            self.audio.play_menu("Menu_Open");
            self.front_backup = None;
            self.front = None;
            self.enter_front(FrontScreen::WorldSelect);
            return;
        }
        // Still generating: latch the snapshot + the layout lerp. With no
        // job (the `TR_UI=worldload` screenshot preset) the latched snapshot
        // stays — the C# reads the live `CurrentGenerationProgress` static,
        // which the preset stands in for.
        let snap = self.world_gen_job.as_ref().map(|j| j.snapshot());
        let aborted = self.world_gen_job.as_ref().is_some_and(|j| j.queued_abort());
        if let Some(FrontScreen::WorldLoad(state)) = self.front.as_mut() {
            state.update_layout(sh);
            if let Some(snap) = snap {
                state.queued_abort = aborted;
                state.set_progress(Some(ui::world_load::GenerationProgress {
                    total_progress: snap.total_progress,
                    value: snap.value,
                    message: snap.message,
                }));
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
        // `DateTime.UtcNow.ToBinary()` (`Player.cs:54909`) — the port tracks
        // no save clock; 0 is the documented "unknown" stamp (ANALYSIS).
        last_saved_binary: 0,
        // W4-e10 — the picked voice pair (`Player.cs:54924-54925`).
        voice_variant: draft.voice_variant,
        voice_pitch_offset: draft.voice_pitch_offset,
        // A fresh character has not earned the Torch God's Favor (W4-f33).
        unlocked_biome_torches: false,
        using_biome_torches: false,
        // The Journey research pairs: a fresh character has none.
        research: Vec::new(),
        powers: crate::player_file::CreativePowersSave::written_fresh(),
        // A freshly-created character is on loadout 0 with three empty sets
        // (`Player.cs:3200-3207`).
        current_loadout_index: 0,
        loadouts: Default::default(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;

    /// W4-g24 — `worldGenCallback`'s HEAD (`WorldGen.cs:6593`): the
    /// generation-start `PlaySound(10)` plays when the job starts (before
    /// `GenerateWorld` runs on the worker); the failure arm (no worlds dir)
    /// plays nothing. Case 10 draws NOTHING — the 2D menu play rides no
    /// RNG stream. The completion tail (`:6603`) is `update_world_load`'s
    /// (already landed, S1-E-17).
    #[test]
    fn w4_g24_the_generation_start_plays_the_menu_open() {
        let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut app = flow_app(None, vec![], vec![]);
        let mut draft = ui::world_creation::NewWorldDraft::new(0, 0);
        draft.name = format!("g24start{}", std::process::id());
        let before = app.audio.played_count();
        app.finish_creating_world(draft);
        if app.world_gen_job.is_some() {
            assert_eq!(
                app.audio.played_count(),
                before + 1,
                "the generation-start cue (:6593)"
            );
            // Tear the job down through the abort so no world file lands.
            app.world_gen_job.as_ref().unwrap().queue_abort();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
            while app.world_gen_job.is_some() {
                app.update_world_load();
                assert!(std::time::Instant::now() < deadline, "the abort must finish");
                std::thread::yield_now();
            }
            if let Some(dir) = tr_core::save::default_world_dir() {
                let _ = std::fs::remove_file(dir.join(format!("g24start{}.trwld", std::process::id())));
            }
        } else {
            // The worlds dir cannot resolve — no job, no cue.
            assert_eq!(app.audio.played_count(), before);
        }
    }
}
