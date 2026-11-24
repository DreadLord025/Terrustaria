//! The App's save glue — the world/player autosave, the explored-map write
//! and its load-side twin. (A6: extracted from `main.rs`.)

//! The App's save glue — the world/player autosave, the explored-map write
//! and its load-side twin. (A6: extracted from `main.rs`.)

use crate::app::{App, is_native_trwld};
use crate::chest;
use crate::map;
use crate::persist;
use tr_core::save::{self, BannerKill, ChestItem, ChestRecord};
use tr_core::wld::sections::SectionData;

/// `Main.WorldRollingBackupsCountToKeep`'s shipped default — the vanilla
/// settings file's `WorldRollbacksToKeep` (2); the settings row lands with
/// the settings work (S1-J-4).
const WORLD_ROLLING_BACKUPS_TO_KEEP: usize = 2;

/// `WorldFile.ValidateWorld`'s port analogue (`WorldFile.cs:941-948`): the
/// written bytes re-read through the full parser — the port's loaders are
/// the validators (`save::parse` for `.trwld`, `wld::parse` for `.wld`).
fn validate_world_bytes(path: &std::path::Path, is_valid: impl Fn(&[u8]) -> bool) -> bool {
    std::fs::read(path)
        .map(|bytes| is_valid(&bytes))
        .unwrap_or(false)
}

/// The `.bak` chain's slot names — the C#'s own numbering: slot 1 is the
/// bare `.bak` (`DoRollingBackups`'s `i == 1` override, `WorldFile.cs:979`)
/// and slots 2..=9 are `.bak2`..`.bak9` (the destinations are
/// `backupWorldWritePath + (num3 + 1)`, `:992` — the `.bak1` name never
/// occurs in the vanilla rotation).
fn backup_path(path: &std::path::Path, i: usize) -> std::path::PathBuf {
    if i <= 1 {
        std::path::PathBuf::from(format!("{}.bak", path.display()))
    } else {
        std::path::PathBuf::from(format!("{}.bak{}", path.display(), i))
    }
}

/// `WorldFile.DoRollingBackups` (`WorldFile.cs:966-1004`): the numbered
/// rotation — `.bak` shifts into `.bak1`, `.bak1` into `.bak2`, up to
/// `keep.min(9)` slots, then the caller writes the previous world file into
/// the freed `.bak` (`InternalSaveWorld`'s `WriteAllBytes(text, array2)`).
fn rolling_backups(path: &std::path::Path, keep: usize, old_world: Option<&[u8]>) {
    if keep <= 1 {
        return;
    }
    let num = keep.min(9);
    let bak = backup_path(path, 0);
    // The chain's first missing slot (`:975-984`; `i == 1` reads the bare
    // `.bak`).
    let mut num2 = 1;
    for i in 1..num {
        if !backup_path(path, i).exists() {
            break;
        }
        num2 = i + 1;
    }
    // The shift walk (`:986-998`): slot `num3` moves into `num3 + 1`, the
    // top slot first (`.bak` is slot 1).
    for num3 in (1..num2).rev() {
        let from = backup_path(path, num3);
        let to = backup_path(path, num3 + 1);
        if from.exists() {
            let _ = std::fs::rename(&from, &to);
        }
    }
    // The previous world file becomes the fresh `.bak` (`:958-962`).
    if let Some(bytes) = old_world {
        let _ = std::fs::write(&bak, bytes);
    }
}

impl App {
    /// `WorldGen.SaveWorld` (`WorldFile.cs:825`'s entry) over the loaded file:
    /// the port writes its native `.trwld` (`save::save`, whose v7-v11 side
    /// records carry the chests/banners/bestiary/NPCs/ore-tiers), a retail
    /// `.wld` writes every section the port carries (`wld::save`), and nothing
    /// else is written for either (a `.wld`'s player state and explored map
    /// have no port format yet — the `.trplayer`/`.map` sidecars stay the
    /// `.trwld` path's; recorded in `docs/ANALYSIS.md`).
    ///
    /// Every exit route funnels through here — the autosave tick
    /// (`frame.rs`, `Main.cs:19751-19765`'s 600 s), `F9`, the window's close
    /// request and the menu's Quit — so the dispatch below is the one place a
    /// save path is decided.
    pub(crate) fn save_world(&mut self) {
        // `Player.InternalSavePlayer`'s first statement (`Player.cs:54700`):
        // `Main.Achievements.Save()`. Achievements are global (one file, not
        // per-world), so every save route writes it before the format branch.
        self.achievements.save();
        // `Player.SavePlayer` (`Player.cs:54700-54742`): the active
        // character's `.plr` — the Journey research pairs ride its
        // creativeTracker block (`:54911`), the per-player powers its
        // SaveToPlayer block (`:54913`). The port's session character is
        // the `session_draft` (the created character's name); a disk-backed
        // card's file is re-saved under its own path. The W4-f44 seam's
        // `PlayerFileWrite` carries the research/powers fields.
        self.save_player_plr();
        // `SetTempToOngoing` (`WorldFile.cs:1053-1059`, called from
        // `InternalSaveWorld:913` before the write): the save snapshots the live
        // weather state into the world's file payload — `SetOngoingToTemps`
        // (`:159-172`) restores it on load. The `.trwld` v14 block carries the
        // rain triple (S1-J-7); the `.wld` flags block now carries the same
        // triple (`:1312-1314`) plus the sandstorm quad (`:1369-1372`).
        self.world.raining = self.rain.is_raining();
        self.world.rain_time = self.rain.rain_time();
        self.world.max_raining = self.rain.max_raining();
        self.world.sandstorm_happening = self.sandstorm.happening;
        self.world.sandstorm_time_left = self.sandstorm.time_left;
        self.world.sandstorm_severity = self.sandstorm.severity;
        self.world.sandstorm_intended_severity = self.sandstorm.intended_severity;
        // `WorldGen.SavedOreTiers` (`WorldFile.cs:1315-1317` + `:1388-1391`):
        // the altar smash advances the App's runtime copy (`editing.rs`'s
        // `smash_altar`), so the save snapshots it into the world payload —
        // the `.trwld` v11 section and the `.wld` flags block both read it
        // from `World.ore_tiers` (S1-J-3b).
        self.world.ore_tiers = self.ore_tiers;
        // `InternalSaveWorld`'s `array2` (`WorldFile.cs:916-918`): the previous
        // world file, captured before the write — the `.bak` source and the
        // failed-write restore source (S1-J-4).
        let old_world = std::fs::read(&self.world_path).ok();
        if is_native_trwld(&self.world_path) {
            // `SaveChests` (`WorldFile.cs:1606-1676`): the world chests ride the
            // same stream as the world itself — a u32 count, then per chest the
            // anchor, the name and the slots, an unwritten `itemType`/`prefix`
            // pair standing in for an air (`stack == 0`) slot.
            let mut chests: Vec<ChestRecord> = Vec::with_capacity(self.world_chests.len());
            for (idx, chest) in self.world_chests.iter().enumerate() {
                // The C# walks the whole `Main.chest` array and skips the null
                // slots; the port's tombstones keep a coordinate nothing can
                // match, so the `FindChest` read is the null test
                // (`Chest.cs:513-520`).
                if chest::find_chest(&self.world_chests, chest.tile.0, chest.tile.1) != Some(idx) {
                    continue;
                }
                chests.push(ChestRecord {
                    x: chest.tile.0,
                    y: chest.tile.1,
                    // The stored `Chest.name`: the rename submit's name, or the
                    // empty string a never-renamed chest carries — the tile
                    // default is derived, never stored (M6-SAVE-1 phase 3's
                    // fold put the name on the entity).
                    name: chest.name.clone(),
                    // The store's implicit-air compression survives verbatim:
                    // the trailing airs are already dropped and the interior
                    // ones are explicit `stack == 0` entries (`Item.IsAir`'s
                    // type-0/stack-0 pair, `Item.cs:385-395`).
                    slots: chest
                        .slots
                        .iter()
                        .map(|slot| {
                            if slot.kind == 0 || slot.count == 0 {
                                ChestItem {
                                    stack: 0,
                                    kind: 0,
                                    prefix: 0,
                                }
                            } else {
                                ChestItem {
                                    // The stack cap is `Item.maxStack`
                                    // (9999 for stackables, `Item.cs:159`),
                                    // far inside the widened `i32` — the port's
                                    // `u32` counts saturate at the top.
                                    stack: slot.count.min(i32::MAX as u32) as i32,
                                    kind: slot.kind,
                                    // `Item.prefix` is a `byte` in the C#
                                    // (`Item.cs:305`); the port's widened `u16`
                                    // is clamped back.
                                    prefix: slot.prefix.min(u8::MAX as u16) as u8,
                                }
                            }
                        })
                        .collect(),
                });
            }
            // `BannerSystem.Save` (`BannerSystem.cs:172-186`): the per-banner
            // kill counts. The vanilla rides the world-flags block; the port's
            // sparse map gets its own v8 section, so each record carries its
            // banner id.
            let banners: Vec<BannerKill> = self
                .banners
                .pairs()
                .map(|(banner, kills)| BannerKill { banner, kills })
                .collect();
            // `NPCKillsTracker.Save` (`NPCKillsTracker.cs:59-70`): the bestiary
            // kill counts ride the world file's section 10 in vanilla; the port
            // had them on the `.trplayer` sidecar, whose line stays written
            // below for builds that read it (the back-compat fallback).
            let bestiary: Vec<save::BestiaryKill> = self
                .bestiary
                .kills
                .pairs()
                .map(|(id, kills)| save::BestiaryKill {
                    credit_id: id.to_string(),
                    kills,
                })
                .collect();
            // `SaveNPCs`'s block B (`WorldFile.cs:1703-1750`): the live town
            // NPCs — the kind, the given name, the pixel position, the
            // homelessness and the home tile. The shimmered-town block A and
            // the `SavesAndLoads` passives (block C) ride the C# arrays the
            // port does not model.
            let npcs: Vec<save::NpcRecord> = self
                .npcs
                .iter()
                // W4-g10 lane 3: `SaveNPCs:1717`'s `active && townNPC &&
                // type != 368` filter — the town pets write `townNPC = true`
                // too, so they ride the town block (the resident-only filter
                // would drop them; the C# saves every townNPC).
                .filter(|n| n.kind.is_town_resident() || n.kind.is_town_pet())
                .map(|n| save::NpcRecord {
                    kind: n.kind.to_u16(),
                    given_name: n.given_name.clone(),
                    x: n.x,
                    y: n.y,
                    homeless: n.home.is_none(),
                    home_x: n.home.map_or(-1, |(hx, _)| hx),
                    home_y: n.home.map_or(-1, |(_, hy)| hy),
                })
                .collect();
            // `WorldGen.SavedOreTiers` (`WorldGen.cs:3283-3297`): the seven
            // metal tiers in the field order (the C# rides them in the world
            // flags block, `WorldFile.cs:1315-1321` + `:1444-1450`).
            let ore = &self.ore_tiers;
            let ore_tiers = [
                ore.copper,
                ore.iron,
                ore.silver,
                ore.gold,
                ore.cobalt,
                ore.mythril,
                ore.adamantite,
            ];
            let side = save::WorldSide {
                chests,
                banners,
                bestiary,
                npcs,
                ore_tiers: Some(ore_tiers),
            };
            match save::save(&self.world_path, &self.world, &side) {
                Ok(()) => {
                    // `InternalSaveWorld`'s post-write walk (`WorldFile.cs:940-962`):
                    // re-read the written bytes and validate them through the full
                    // parser; a broken write restores the previous world file, a
                    // good one rotates it into the `.bak` chain (S1-J-4).
                    let ok = validate_world_bytes(&self.world_path, |b| save::parse(b).is_ok());
                    if ok {
                        rolling_backups(
                            &self.world_path,
                            WORLD_ROLLING_BACKUPS_TO_KEEP,
                            old_world.as_deref(),
                        );
                        log::info!("saved {}", self.world_path.display());
                    } else {
                        log::warn!("save failed validation: {}", self.world_path.display());
                    }
                }
                Err(e) => log::warn!("save failed: {e:#}"),
            }
            if let Some(p) = &self.player {
                let inv =
                    std::array::from_fn(|i| (self.inventory[i].kind, self.inventory[i].count));
                // `Player.armor[0..3]`'s kinds — the equipped armor persists
                // with the rest of the character (vanilla `Player.SavePlayer`).
                let armor: [u16; 3] = std::array::from_fn(|i| self.equip.armor[i].kind);
                // `Player.dye[0..10]`'s kinds — version 6 of the sidecar (the
                // vanilla writes the dye array straight after the armor,
                // `Player.cs:54818-54822`).
                let dye: [u16; 10] = std::array::from_fn(|i| self.equip.dye[i].kind);
                let kills: Vec<(String, i32)> = self
                    .bestiary
                    .kills
                    .pairs()
                    .map(|(id, count)| (id.to_string(), count))
                    .collect();
                if let Err(e) = persist::save(
                    &self.world_path,
                    p,
                    inv,
                    self.coins,
                    armor,
                    &dye,
                    &kills,
                    // The live equip set's share flags ride the v7 loadout
                    // block (`Player.cs:3200-3207`).
                    self.equip_share_flags,
                ) {
                    log::warn!("player save failed: {e}");
                }
            }
            // `Player.InternalSaveMap` (`Player.cs:55046`): the explored map is
            // written alongside the player/world save.
            self.save_map();
        } else {
            let sections = self.wld_sections();
            match tr_core::wld::save(&self.world_path, &self.world, &sections) {
                Ok(()) => {
                    let ok =
                        validate_world_bytes(&self.world_path, |b| tr_core::wld::parse(b).is_ok());
                    if ok {
                        rolling_backups(
                            &self.world_path,
                            WORLD_ROLLING_BACKUPS_TO_KEEP,
                            old_world.as_deref(),
                        );
                        log::info!("saved {}", self.world_path.display());
                    } else {
                        log::warn!("save failed validation: {}", self.world_path.display());
                    }
                }
                Err(e) => log::warn!("save failed: {e:#}"),
            }
        }
    }

    /// The `.wld` save halves of every section, from the live state — the
    /// C#'s "read into the globals, then write them back out" chain
    /// (`SaveWorld_Version2:1150`). Each owner module's seam is called in the
    /// section order so the records land in the file the same way the C#'
    /// writers walk their globals; the only pre-pass is the chest anchor
    /// destruction (`SaveChests:1609-1642`), which mutates the live store
    /// exactly as the C# mutates `Main.chest`.
    fn wld_sections(&mut self) -> SectionData {
        let mut data = SectionData::default();
        // `SaveChests:1609-1642`: a chest whose four anchor cells are no longer
        // all active container tiles is destroyed *before* the write.
        crate::chest::destroy_invalid_anchors(&mut self.world_chests, &self.world);
        data.chests = crate::chest::to_wld(&self.world_chests);
        data.signs = tr_core::wld::sections::signs::from_world(&self.world);
        // `SaveNPCs:1703-1749` over the live NPCs, in `Main.npc` order.
        data.npcs = crate::npc::to_wld(&self.npcs);
        data.npcs.shimmered = self.shimmered_npcs.to_wld_indices();
        data.tile_entities = tr_core::wld::sections::tile_entities::from_world(&self.world);
        data.pressure_plates = tr_core::wld::sections::pressure_plates::from_world(&self.world);
        data.town_rooms = self.town_rooms.to_wld_rooms();
        // The bestiary kills ride the world file for a `.wld` (owner decision
        // 2026-11-08); the sights/chats ride with them, as in vanilla.
        data.bestiary = tr_core::wld::sections::bestiary::to_wld(&self.bestiary);
        data.creative_powers = crate::creative::persistent_powers(&self.creative_powers);
        data
    }

    /// `Player.SavePlayer` (`Player.cs:54700-54742`) — the active character's
    /// `.plr` write. The Journey research pairs (`research::save_pairs`, the
    /// `ItemsSacrificedUnlocksTracker.Save` shape) and the per-player powers
    /// (the `CreativePowersSave` shape) ride the W4-f44 seam's
    /// `PlayerFileWrite`. The session character's path is the
    /// `default_player_path` (`Main.GetPlayerPathFromName`, `Main.cs:4820`);
    /// a disk-backed card re-saves under its own path. The inventory/stats
    /// halves stay the session's (the port's `.plr` writer is the
    /// created-character one — the full live-state write is the W4-f44
    /// lane's; this seam owns the research/powers fields).
    fn save_player_plr(&self) {
        let Some(draft) = self.session_draft.as_ref() else {
            return; // no session character — nothing to save
        };
        let Some(path) = tr_core::save::default_player_path(&draft.name) else {
            return;
        };
        // The existing file's stats/inventory halves are preserved (the
        // port's writer is the created-character one); only the
        // research/powers fields are refreshed from the live state.
        let mut file = crate::player_file::read_player_file(&path)
            .map(|f| crate::player_file::PlayerFileWrite {
                name: f.name,
                difficulty: f.difficulty,
                skin_variant: f.skin_variant,
                hair: f.hair,
                life: f.life,
                life_max: f.life_max,
                mana: f.mana,
                mana_max: f.mana_max,
                hair_color: f.hair_color,
                skin_color: f.skin_color,
                eye_color: f.eye_color,
                shirt_color: f.shirt_color,
                undershirt_color: f.undershirt_color,
                pants_color: f.pants_color,
                shoe_color: f.shoe_color,
                inventory: f.inventory,
                inv_favorites: f.inv_favorites,
                last_saved_binary: 0,
                research: Vec::new(),
                powers: crate::player_file::CreativePowersSave::written_fresh(),
            })
            .unwrap_or_default();
        file.name = draft.name.clone();
        file.difficulty = draft.difficulty;
        file.research = self.research.save_pairs();
        file.powers = crate::player_file::CreativePowersSave {
            godmode: self.creative_powers.god_mode(),
            far_placement: self.creative_powers.far_placement_range(),
            spawn_rate: self.creative_powers.slider(crate::creative::PowerId::SpawnRate),
        };
        if let Err(e) = crate::player_file::write_player_file(&path, &file) {
            log::warn!("player save failed: {e}");
        }
    }

    /// Binds `AchievementManager._savePath` (`AchievementManager.cs:25`,
    /// `:51`): the C# constructor resolves `Main.SavePath/achievements.dat`
    /// when the manager is created, so the port's production bootstrap
    /// (`main` and the world swap) calls this. A headless test App — built
    /// through the same `App::new` — keeps `None` and therefore never writes
    /// the developer's real `achievements.trdat`.
    pub(crate) fn bind_achievements_save_path(&mut self) {
        self.achievements.save_path = tr_core::save::achievements_path();
    }

    /// `WorldGen.SaveAndQuit` (`WorldGen.cs:6701-6719`) + its
    /// `SaveAndQuitCallBack` (`WorldGen.cs:6664-6687`), run synchronously —
    /// the Settings window's Save & Exit row (`IngameOptions.cs:296-305`:
    /// `Close()` + `Main.menuMode = 10` + `Main.gameMenu = true` +
    /// `WorldGen.SaveAndQuit()`). The C# queues the callback on the thread
    /// pool behind the mode-10 "saving" screen; the port saves inline and
    /// lands straight on `menuMode = 0` (the main menu), the same menu-reopen
    /// shape `net_disconnect` uses. `CaptureInterface.ResetFocus` is not
    /// modelled; the frame's audio listener flips to menu mode off
    /// `menu.is_some()` (`frame.rs:256`), covering the ¼ s ambience tick —
    /// the IMMEDIATE `StopWaterfallAmbienceAudio`/`StopTrackedSounds` halves
    /// are wired below (the W4-g24/g73 quit-stop keep-one: ONE `:6704`
    /// call site, the g24 seam set + this branch's tracked drain).
    /// are wired below (W4-g24 doc-sweep).
    pub(crate) fn save_and_quit(&mut self) {
        // `WorldGen.cs:6703`: `SoundEngine.PlaySound(11)`.
        self.audio.play("Menu_Close");
        // `WorldGen.cs:6704`: `StopWaterfallAmbienceAudio()` — the 34/35
        // sentinel pair stops both liquid loops at the quit moment
        // (`WorldGen.cs:6689-6699`, the `LegacySoundPlayer.cs:1257`/:1287
        // `x == -1 || y == -1` stop arms).
        self.audio.stop_waterfall_ambience();
        // `WorldGen.cs:6705-6708`: the invasion bar resets.
        self.invasion_progress = -1;
        self.invasion_progress_display_left = 0;
        self.invasion_progress_alpha = 0.0;
        self.invasion_progress_icon = 0;
        // `WorldGen.cs:6714`: `InGameNotificationsTracker.Clear()` — the
        // unlock toasts never survive onto the menu (S1-E-26 wave A).
        self.notifications.clear();
        // `WorldGen.cs:6715`: `SoundEngine.StopTrackedSounds()` — the
        // tracked pool empties (`SoundPlayer.StopAll`, `SoundPlayer.cs:
        // 79-86`). The legacy one-shot cues already playing ride out their
        // tails (the C# does not stop those either).
        self.audio.stop_tracked_sounds();
        // `SaveAndQuitCallBack` (`WorldGen.cs:6669-6677`): StopPlayTimer +
        // `Player.SavePlayer` + `WorldFile.SaveWorld` (single-player's
        // `Main.netMode == 0` arm) — the port's one save funnel writes the
        // world, the player sidecar and the explored map together.
        // `Rain.ClearRain` (`:6672`) is visual-only; the save already
        // snapshotted the live rain into the file payload above.
        self.save_world();
        // `WorldGen.cs:6676`: `SoundEngine.PlaySound(10)` after the world write.
        self.audio.play("Menu_Open");
        // `WorldGen.cs:6678-6679`: both fast-forward flags drop.
        self.world.fast_forward_to_dawn = false;
        self.world.fast_forward_to_dusk = false;
        // `WorldGen.cs:6682`: `Main.menuMode = 0` — back to the main menu
        // (`Main.gameMenu` stays true; the port's `menu.is_some()` is that
        // flag, `frame.rs:256`).
        if self.menu.is_none() {
            self.menu = Some(crate::menu::Menu::new(
                self.settings.clone(),
                self.player_name.clone(),
            ));
        }
    }

    /// `WorldGen.JustQuit` (`WorldGen.cs:6627-6662`): exit the world WITHOUT
    /// saving — the `/quit` debug chat command (`ToolkitDebugCommands.cs:
    /// 232-237`, `CommandRequirement.Client`: single-player or a net client)
    /// is the ONLY vanilla caller; the port's F3 debug panel is the chat
    /// dispatcher's stand-in (the S1-E-29 recorded deviation), so the World
    /// tab's quit row drives this. The C# body in C# order: the inline
    /// `StopWaterfallAmbienceAudio` pair (`:6631-6632`), the invasion reset
    /// (`:6637-6640`), the mode-10/gameMenu/Disconnect/netMode-0 block
    /// (`:6642-6645` — the port's `net_disconnect` drop shape), the tracked
    /// stop (`:6646`), the fast-forward clears (`:6649-6650`), then the
    /// single-player arm's `GoToWorldSelect` + the player-position reset
    /// (`:6653-6657`) or the multiplayer arm's mode 0 (`:6660`).
    /// `CaptureInterface.ResetFocus`/`StopPlayTimer`/`UpdateTimeRate`/
    /// `ClearVisualPostProcessEffects` are engine-side (not modelled — the
    /// port keeps no play timer and the frame's listener flip covers the
    /// menu transition).
    // The only production caller is the F3 debug panel's quit row (the
    // `debug` feature, OFF by default); the test module exercises it in
    // every build. The allow keeps the default build warning-clean
    // (the g24 `7ca8b087` fold, kept by the quit-stop keep-one).
    // every build. The allow keeps the default build warning-clean.
    #[allow(dead_code)]
    pub(crate) fn just_quit(&mut self) {
        // `WorldGen.cs:6631-6632`: the inlined 34/35 sentinel pair (the
        // `StopWaterfallAmbienceAudio` body, `:6693-6694`).
        self.audio.stop_waterfall_ambience();
        // `WorldGen.cs:6637-6640`: the invasion bar resets.
        self.invasion_progress = -1;
        self.invasion_progress_display_left = 0;
        self.invasion_progress_alpha = 0.0;
        self.invasion_progress_icon = 0;
        // `WorldGen.cs:6641`: `flag = Main.netMode == 0` — read BEFORE the
        // netMode drop, so a client quit takes the main-menu arm.
        let single = matches!(self.net, crate::app::NetRole::Off);
        // `WorldGen.cs:6644-6645`: `Netplay.Disconnect = true; Main.netMode
        // = 0` — the port's role drop (the `net_disconnect` shape, without
        // its reason banner: the C# quit shows none).
        self.net_dropped = true;
        self.net_state = crate::net::client_state::NOT_CONNECTED;
        self.remote_players.clear();
        self.local_id = 0;
        self.net = crate::app::NetRole::Off;
        // `WorldGen.cs:6646`: `SoundEngine.StopTrackedSounds()`.
        self.audio.stop_tracked_sounds();
        // `WorldGen.cs:6649-6650`: both fast-forward flags drop.
        self.world.fast_forward_to_dawn = false;
        self.world.fast_forward_to_dusk = false;
        if single {
            // `WorldGen.cs:6655`: `Main.GoToWorldSelect()` (`Main.cs:
            // 48176-48180`: menuMode 888 + the world-select UIState) — the
            // port's front-screen hop re-lists the worlds.
            self.enter_front(crate::app::flow::FrontScreen::WorldSelect);
            // `WorldGen.cs:6656`: the local player parks at the origin.
            if let Some(p) = self.player.as_mut() {
                p.x = 0.0;
                p.y = 0.0;
            }
        } else {
            // `WorldGen.cs:6660`: `Main.menuMode = 0` — the main menu.
            if self.menu.is_none() {
                self.menu = Some(crate::menu::Menu::new(
                    self.settings.clone(),
                    self.player_name.clone(),
                ));
            }
        }
    }

    /// `MapHelper.SaveMap` (`MapHelper.cs:3007`): write `<world>.map` next to the
    /// world. Gated on `Main.mapEnabled` (`MapHelper.cs:3009`).
    pub(crate) fn save_map(&self) {
        if !self.settings.map_enabled {
            return;
        }
        let Some(map) = self.map.as_ref() else { return };
        let path = persist::map_path(&self.world_path);
        if let Err(e) = map::save_map_file(&self.world, &map.tiles, &path) {
            log::warn!("map save failed: {e}");
        }
    }

    /// `MapHelper.LoadMap`: load the explored map at world load. The wiped-map
    /// symptom (the map resetting on every restart) came from never calling this.
    pub(crate) fn load_map(&mut self) {
        if !is_native_trwld(&self.world_path) || !self.settings.map_enabled {
            return;
        }
        let path = persist::map_path(&self.world_path);
        let Some(m) = self.map.as_mut() else { return };
        if m.load(&self.world, &path) {
            log::info!("map: loaded {}", path.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::WorldSections;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::craft::WorldChest;
    use crate::item::ItemSlot;
    use crate::registry;
    use crate::test_world_helpers::{build_furnished_room, ground};
    use tr_core::World;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// A per-test scratch dir under the system temp dir (the shared
    /// `test_world_helpers` helper — unique per pid + test name, so parallel
    /// gates in sibling worktrees never share a path).
    fn scratch(name: &str) -> std::path::PathBuf {
        crate::test_world_helpers::test_scratch(&format!("saveglue-p7-{name}"))
    }

    fn slot(kind: u16, count: u32) -> ItemSlot {
        ItemSlot {
            kind,
            name: registry::name(kind),
            count,
            prefix: 0,
        }
    }

    /// A 2×2 anchor tile at the corner with the style-0 frames (`chest.rs`'s
    /// fixture shape: the 0/18 corners of `Chest.cs`'s frame layout).
    fn put_anchor(world: &mut World, x: i32, y: i32, kind: u16) {
        for (dx, dy) in [(0i32, 0i32), (1, 0), (0, 1), (1, 1)] {
            let i = world.index(x + dx, y + dy);
            world.tiles[i] = Tile {
                kind,
                frame_x: if dx == 1 { 18 } else { 0 },
                frame_y: if dy == 1 { 18 } else { 0 },
                ..Default::default()
            };
            world.tiles[i].flags |= FLAG_ACTIVE;
        }
    }

    /// A world that can carry every section: the ground, a chest quad, a sign
    /// tile, an item frame and a kite anchor on the surface, a pressure-plate
    /// point, and one furnished room for a town NPC's home.
    fn section_world() -> World {
        let mut w = ground();
        put_anchor(&mut w, 20, 20, 21); // chest — `Main.tileContainer`
        put_anchor(&mut w, 30, 20, 55); // sign tile
        put_anchor(&mut w, 40, 20, 395); // item frame (TE)
        put_anchor(&mut w, 50, 20, 723); // kite anchor (TE)
        build_furnished_room(&mut w, 60, 100, 12, 9);
        w
    }

    /// A full-live-state App over [`section_world`]: every App-owned section's
    /// source has content, so no save lane can be empty by accident.
    fn stocked_app() -> App {
        let world = section_world();
        let mut app = flow_app(None, vec![], vec![]);
        app.world = world;
        // Chests: one named, stocked chest with a non-default slot count.
        app.world_chests = vec![WorldChest {
            tile: (20, 20),
            slots: vec![slot(9, 6), slot(71, 3)],
            name: "Loot".into(),
            max_items: 20,
            ..Default::default()
        }];
        // NPCs: a housed Merchant and a homeless Nurse (the re-validation's
        // two arms), both named, one carrying the shimmer payload.
        let mut merchant = crate::npc::Npc::new(
            crate::npc::NpcKind::Merchant,
            65.0 * 16.0,
            130.0 * 16.0,
            1,
            0.0,
        );
        merchant.given_name = "Hal".into();
        merchant.home = Some((65, 105));
        merchant.variation_index = 3;
        let mut nurse =
            crate::npc::Npc::new(crate::npc::NpcKind::Nurse, 70.0 * 16.0, 90.0 * 16.0, 1, 0.0);
        nurse.given_name = "Nurse".into();
        nurse.home = None; // homeless
        nurse.homeless_despawn = true;
        app.npcs = vec![merchant, nurse];
        app.shimmered_npcs.set_shimmered(17, true);
        app.town_rooms.set_room(17, 65, 105);
        // Signs (world state) and pressure plates.
        app.world.signs = vec![tr_core::wld::sections::WldSign {
            text: "hello".into(),
            x: 30,
            y: 20,
        }];
        app.world.pressure_plates = vec![tr_core::wld::sections::WldPressurePlate { x: 33, y: 21 }];
        // Tile entities: the item frame's item and the kite anchor's item type.
        let frame_id = app
            .world
            .tile_entities
            .place(
                &app.world.clone(),
                tr_core::tile_entities::TeKind::ItemFrame,
                40,
                20,
            )
            .expect("the item frame anchor is valid");
        app.world.tile_entities.set_item(
            frame_id,
            Some(tr_core::tile_entities::TeItem {
                kind: 9,
                prefix: 0,
                stack: 1,
            }),
        );
        let world_for_te = app.world.clone();
        app.world
            .tile_entities
            .place(
                &world_for_te,
                tr_core::tile_entities::TeKind::KiteAnchor,
                50,
                20,
            )
            .expect("the kite anchor is valid");
        // Bestiary: all three halves.
        app.bestiary.kills.set_kill_count_directly("BlueSlime", 7);
        app.bestiary.set_was_seen_directly("GreenSlime");
        app.bestiary.set_was_chat_with_directly("Pinky");
        // Creative powers: the toggle and the slider.
        app.creative_powers.set(
            crate::creative::PowerId::FreezeTime,
            crate::creative::PowerState::Toggle(true),
        );
        app.creative_powers.set(
            crate::creative::PowerId::TimeRate,
            crate::creative::PowerState::Slider(12.0),
        );
        // The weather freezes + the difficulty strength ride the section too
        // (W4-c24 phase 4): the rain freeze on, the wind freeze off, the
        // difficulty at the x1 seam (cache 0.33 → strength 1.0).
        app.creative_powers.set(
            crate::creative::PowerId::FreezeRain,
            crate::creative::PowerState::Toggle(true),
        );
        app.creative_powers.set(
            crate::creative::PowerId::Difficulty,
            crate::creative::PowerState::Slider(1.0),
        );
        app
    }

    /// The live books' tile entities as `(kind, x, y)` triples in id order —
    /// the ids themselves are reassigned on load (`LoadTileEntities:3323`),
    /// so they are not a persisted field.
    fn te_triples(app: &App) -> Vec<(tr_core::tile_entities::TeKind, i16, i16)> {
        app.world
            .tile_entities
            .live_entities()
            .map(|e| (e.kind, e.x, e.y))
            .collect()
    }

    #[test]
    fn test_DW_7_1_a_full_world_round_trips_every_section() {
        // DW-7.1: synthesize a full `World` + App state, write it as a `.wld`,
        // reload, and assert every persisted field is equal — all eleven
        // sections exercised (chests/signs/NPCs/TEs/plates/town rooms/bestiary/
        // creative powers beside the header, flags and tiles).
        let dir = scratch("round-trip");
        let path = dir.join("full.wld");
        let mut app = stocked_app();
        app.world_path = path.clone();
        app.save_world();

        let loaded = tr_core::wld::load_with_sections(&path).expect("the saved file reads back");
        let sections = loaded.sections;
        // --- chests (section 4): the file's records are the live store -----
        let chests = crate::chest::from_wld(&sections.chests);
        assert_eq!(chests.len(), 1, "one live chest in, one record out");
        assert_eq!(chests[0].tile, (20, 20));
        assert_eq!(chests[0].name, "Loot");
        assert_eq!(chests[0].max_items, 20);
        assert_eq!(chests[0].slots[0], slot(9, 6));
        assert_eq!(chests[0].slots[1], slot(71, 3));
        assert_eq!(chests[0].slots.len(), 20, "`maxItems` real slots on load");
        // --- signs (section 5): the text and its tile ----------------------
        assert_eq!(
            sections.signs,
            vec![tr_core::wld::sections::WldSign {
                text: "hello".into(),
                x: 30,
                y: 20
            }]
        );
        // The world-state halves are the app's load job (`load_world`), so
        // call them the way the startup does and compare the store.
        let mut world = loaded.world;
        tr_core::wld::sections::signs::apply_loaded(&mut world, &sections.signs);
        assert_eq!(world.signs, sections.signs, "applied on load");
        tr_core::wld::sections::pressure_plates::apply_loaded(
            &mut world,
            &sections.pressure_plates,
        );
        assert_eq!(world.pressure_plates, sections.pressure_plates);
        // --- NPCs (section 6): both records, the town rooms, the bitmap ----
        assert_eq!(sections.npcs.town.len(), 2, "both town NPCs");
        assert_eq!(sections.npcs.shimmered, vec![17], "the shimmer truth index");
        assert_eq!(sections.npcs.town[0].given_name, "Hal");
        assert_eq!(
            (sections.npcs.town[0].home_x, sections.npcs.town[0].home_y),
            (65, 105)
        );
        assert_eq!(sections.npcs.town[1].given_name, "Nurse");
        assert!(
            sections.npcs.town[1].homeless,
            "the homeless record says so"
        );
        assert!(sections.npcs.town[1].homeless_despawn);
        assert_eq!(
            sections.town_rooms,
            app.town_rooms.to_wld_rooms(),
            "the stored room rides section 9"
        );
        let npcs = crate::npc::from_wld(&sections.npcs);
        assert_eq!(npcs.len(), 2);
        assert_eq!(npcs[0].given_name, "Hal");
        assert_eq!(npcs[0].home, Some((65, 105)), "the room still fits");
        assert_eq!(npcs[0].variation_index, 3);
        assert_eq!(npcs[1].home, None, "the homeless record stays homeless");
        // --- tile entities (section 7): both payloads ----------------------
        assert_eq!(
            sections.tile_entities.len(),
            2,
            "the two entities survive the removal pass"
        );
        let kinds: Vec<u8> = sections.tile_entities.iter().map(|e| e.ty).collect();
        assert!(kinds.contains(&(tr_core::tile_entities::TeKind::ItemFrame as u8)));
        assert!(kinds.contains(&(tr_core::tile_entities::TeKind::KiteAnchor as u8)));
        let mut reloaded = stocked_app();
        tr_core::wld::sections::tile_entities::apply_loaded(
            &mut reloaded.world,
            &sections.tile_entities,
        );
        assert_eq!(
            te_triples(&reloaded),
            te_triples(&app),
            "the anchored pair lands back with their payloads"
        );
        // --- pressure plates (section 8) ----------------------------------
        assert_eq!(
            sections.pressure_plates,
            vec![tr_core::wld::sections::WldPressurePlate { x: 33, y: 21 }]
        );
        assert_eq!(world.pressure_plates, sections.pressure_plates);
        // --- bestiary (section 10): kills, sights and chats ----------------
        assert_eq!(sections.bestiary.kills, vec![("BlueSlime".to_string(), 7)]);
        assert_eq!(sections.bestiary.sights, vec!["GreenSlime".to_string()]);
        assert_eq!(sections.bestiary.chats, vec!["Pinky".to_string()]);
        // --- creative powers (section 11): the toggle + the slider cache ---
        let by_id = |id: u16| {
            sections
                .creative_powers
                .records
                .iter()
                .find(|(i, _)| *i == id)
                .map(|(_, v)| v.clone())
        };
        assert_eq!(
            by_id(0),
            Some(tr_core::wld::sections::WldCreativeValue::Bool(true)),
            "the freeze toggle"
        );
        assert_eq!(
            by_id(8),
            Some(tr_core::wld::sections::WldCreativeValue::F32(12.0)),
            "the rate round-trips — the bag holds the ported value, the wire
             its [0,1] cache (pinned by creative_powers::test_DW_7_6_*)"
        );
        assert_eq!(
            by_id(13),
            Some(tr_core::wld::sections::WldCreativeValue::Bool(false)),
            "the biome-spread freeze"
        );
        // The weather freezes + the difficulty strength (W4-c24 phase 4).
        assert_eq!(
            by_id(9),
            Some(tr_core::wld::sections::WldCreativeValue::Bool(true)),
            "the rain freeze"
        );
        assert_eq!(
            by_id(10),
            Some(tr_core::wld::sections::WldCreativeValue::Bool(false)),
            "the wind freeze"
        );
        assert_eq!(
            by_id(12),
            Some(tr_core::wld::sections::WldCreativeValue::F32(1.0)),
            "the difficulty strength — the bag holds the ported value, the
             wire its [0,1] cache (the codec's canonical inverse)"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_DW_7_3_a_wld_save_writes_every_section_and_increments_the_revision() {
        // DW-7.3: a `.wld`-loaded world survives autosave/F9/quit without the
        // "not saving" log — the save writes the file, and `FileMetadata`'s
        // revision counter increments as vanilla's `IncrementAndWrite` does.
        let dir = scratch("revision");
        let path = dir.join("auto.wld");
        let mut app = stocked_app();
        app.world_path = path.clone();
        app.save_world();
        assert!(path.exists(), "the first save wrote the world file");

        let first = std::fs::read(&path).expect("read back");
        let table = tr_core::wld::SectionTable::open(&first).expect("opens");
        assert_eq!(table.metadata().revision, 1, "a first save starts at 1");
        assert_eq!(table.metadata().ty, 2, "FileType.World");
        // The written sections hold the live state (the autosave path saved
        // them, not just the header/tiles).
        let sections = tr_core::wld::load_sections(&table).expect("walks its sections");
        assert_eq!(sections.chests.len(), 1);
        assert_eq!(sections.npcs.town.len(), 2);
        assert_eq!(sections.signs.len(), 1);
        assert_eq!(sections.tile_entities.len(), 2);

        // The second save (F9) writes the same bytes with the revision bumped:
        // deterministic apart from `FileMetadata`'s counter.
        app.save_world();
        let second = std::fs::read(&path).expect("read back");
        let table2 = tr_core::wld::SectionTable::open(&second).expect("opens");
        assert_eq!(table2.metadata().revision, 2, "increments per save");
        // Everything past the 24-byte version+metadata head is identical.
        assert_eq!(
            second[24..],
            first[24..],
            "the revision is the only thing that moved"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_empty_world_saves_every_section_at_its_empty_encoding() {
        // The dirty arm of DW-7.1: a world with zero chests/NPCs/signs/TEs and
        // default state saves its empty sections and reads them back as the
        // defaults — no section claims a record it did not write.
        let dir = scratch("empty");
        let path = dir.join("empty.wld");
        let mut app = flow_app(None, vec![], vec![]);
        app.world_path = path.clone();
        app.save_world();
        let loaded = tr_core::wld::load_with_sections(&path).expect("the empty world writes");
        assert!(loaded.sections.chests.is_empty(), "no chests");
        assert!(loaded.sections.signs.is_empty(), "no signs");
        assert!(loaded.sections.npcs.town.is_empty(), "no town NPCs");
        assert!(
            loaded.sections.npcs.shimmered.is_empty(),
            "no shimmered NPCs"
        );
        assert!(loaded.sections.tile_entities.is_empty(), "no tile entities");
        assert!(loaded.sections.pressure_plates.is_empty(), "no plates");
        assert!(loaded.sections.town_rooms.is_empty(), "no town rooms");
        assert!(loaded.sections.bestiary.is_empty(), "no bestiary state");
        // The creative section is never empty in the port: the registry holds
        // the six per-world powers at their defaults, and the C# writes every
        // `IPersistentPerWorldContent` power regardless (`SaveToWorld:127-136`).
        let ids: Vec<u16> = loaded
            .sections
            .creative_powers
            .records
            .iter()
            .map(|(id, _)| *id)
            .collect();
        assert_eq!(ids, [0, 8, 9, 10, 12, 13], "the six per-world powers, ascending");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_legacy_pointer_table_keeps_the_missing_sections_at_their_defaults() {
        // The `.wld` whose pointer table stops early (`sectionCount < 11`): the
        // walk takes only the pointers the table has, so the missing sections
        // stay at their defaults and the app glue applies nothing for them.
        // Built from the port's own v326 output with the table truncated to
        // six pointers and the bodies past the NPCs section dropped — the
        // reader's own legacy-shape tests live in `wld/mod.rs`
        // (`test_DW_2_2_the_old_version_gates_walk_the_flags_block`,
        // `load_sections_walks_a_legacy_six_pointer_table`); this one pins
        // what the *app glue* does with such a file.
        let dir = scratch("legacy");
        let world = World::new("Legacy".into(), 8, 6);
        let buf = tr_core::wld::to_bytes(&world, &SectionData::default(), 1).expect("writes");
        // `pointers[i]` is section i's end offset, in the prologue's table at
        // `26 + 4 * i` (4 version + 20 metadata + 2 count).
        let pointer = |i: usize| {
            u32::from_le_bytes(buf[26 + 4 * i..26 + 4 * i + 4].try_into().unwrap()) as usize
        };
        const KEEP: usize = 6; // header, tiles, chests, signs, NPCs
        // The prologue, rebuilt with a six-pointer table: version + metadata +
        // the shorter pointer table + the importance bitmap the table's own
        // `pointers[0]` lands on. In the modern buffer the bitmap sits after
        // the 44-byte eleven-pointer table (`26 + 44 .. points[0]`).
        const TABLE: usize = 26 + 4 * 11; // the eleven-pointer table's extent
        let mut file = Vec::new();
        file.extend_from_slice(&buf[..24]); // version + FileMetadata
        file.extend_from_slice(&(KEEP as i16).to_le_bytes()); // the count
        let ptr_at = file.len();
        file.extend_from_slice(&[0u8; 4 * KEEP]); // the six pointer placeholders
        file.extend_from_slice(&buf[TABLE..pointer(0)]); // the importance bitmap
        // Sections 0..5 verbatim, then the footer.
        file.extend_from_slice(&buf[pointer(0)..pointer(KEEP - 1)]);
        // Every offset shifts down by the 20 pointer slots the shorter table
        // dropped (`4 * (11 - KEEP)`), so the section end offsets stay true.
        let shift = 4 * (11 - KEEP) as isize;
        for i in 0..KEEP {
            let at = ptr_at + 4 * i;
            let value = pointer(i) as isize - shift;
            file[at..at + 4].copy_from_slice(&(value as i32).to_le_bytes());
        }
        file.push(1); // the footer's leading `true`
        let name = b"Legacy";
        file.push(name.len() as u8);
        file.extend_from_slice(name);
        file.extend_from_slice(&0i32.to_le_bytes());

        let path = dir.join("legacy.wld");
        std::fs::write(&path, &file).expect("write the legacy file");
        let loaded = tr_core::wld::load_with_sections(&path).expect("the legacy file loads");
        assert_eq!(loaded.world.name, "Legacy");
        assert!(
            loaded.sections.chests.is_empty(),
            "no chest section present"
        );
        assert!(loaded.sections.signs.is_empty(), "no sign section present");
        assert!(
            loaded.sections.tile_entities.is_empty(),
            "the missing TE section stays at its default"
        );
        assert!(
            loaded.sections.bestiary.is_empty(),
            "the missing bestiary section stays at its default"
        );
        assert!(
            loaded.sections.creative_powers.records.is_empty(),
            "the missing creative section stays at its default"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_DW_7_2_the_trwld_path_is_unchanged() {
        // DW-7.2: the native `.trwld` branch keeps its exact behaviour — the
        // world and its sidecar round-trip, and a double-run pair is equal
        // (both writers are deterministic: no clock, no revision counter).
        // The workspace carries no hashing crate, so the pair is asserted as
        // byte equality — strictly stronger than the SHA256 equality the
        // determinism rule asks for (`tools/pixel_diff.py`'s pairs are taken
        // at the tool level, outside the test suite). The branch's body is the
        // pre-phase code verbatim; the only changes are the `&mut self`
        // receiver and the field rename (see the discovery file's diff note).
        let dir = scratch("trwld");
        let path = dir.join("native.trwld");
        let mut app = stocked_app();
        app.world_path = path.clone();
        // A deterministic player so the sidecar's bytes are stable too.
        app.player = Some(crate::player::Player::spawn(&app.world));
        app.save_world();
        let one = std::fs::read(&path).expect("world written");
        let sidecar_one = std::fs::read(crate::persist::player_path(&path))
            .expect("the .trplayer sidecar written");
        app.save_world();
        let two = std::fs::read(&path).expect("world written");
        let sidecar_two = std::fs::read(crate::persist::player_path(&path))
            .expect("the .trplayer sidecar written");
        assert_eq!(one, two, "the world pair is byte-equal");
        assert_eq!(sidecar_one, sidecar_two, "the sidecar pair is byte-equal");
        // The world round-trips through the native reader, and no `.wld`-only
        // write appeared beside it.
        let (world, _native) = tr_core::save::load(&path).expect("reads back");
        assert_eq!(world.name, "t", "the `ground()` fixture world");
        assert!(
            !dir.join("native.wld").exists(),
            "no `.wld` beside a `.trwld`"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The quit paths' audio stops (the W4-g24/g73 quit-stop keep-one):
    /// `SaveAndQuit` (`WorldGen.cs:6704`/:6715) calls
    /// `StopWaterfallAmbienceAudio` then `StopTrackedSounds`; `JustQuit`
    /// (`:6631-6632`/:6646) inlines the same pair. Both drain the tracked
    /// pool; the sentinel-stop counter pins the immediate liquid-loop stop
    /// (the frame's ¼ s tick is NOT the seam under test).
    /// W4-g24 doc-sweep — the quit paths' audio stops: `SaveAndQuit`
    /// (`WorldGen.cs:6704`/:6715) calls `StopWaterfallAmbienceAudio` then
    /// `StopTrackedSounds`; `JustQuit` (`:6631-6632`/:6646) inlines the same
    /// pair. Both drain the tracked pool; the sentinel-stop counter pins
    /// the immediate liquid-loop stop (the frame's ¼ s tick is NOT the
    /// seam under test).
    #[test]
    fn w4_g24_the_quit_paths_stop_the_ambience_and_the_tracked_pool() {
        let style = crate::audio::menu_accept_style();
        // Save & Quit: the world write runs, then the menu reopens.
        let dir = scratch("quit-stop");
        let mut app = stocked_app();
        app.world_path = dir.join("native.trwld");
        app.player = Some(crate::player::Player::spawn(&app.world));
        app.audio.insert_test_cue("menu_accept");
        let _ = app.audio.play_tracked_global(style.clone());
        assert_eq!(app.audio.tracked_count_for(&style), 1);
        app.save_and_quit();
        assert_eq!(app.audio.liquid_ambience_stop_count(), 1, "the :6704 stop");
        assert_eq!(app.audio.tracked_count_for(&style), 0, "the :6715 drain");
        assert!(app.menu.is_some(), "the main menu reopened");
        let _ = std::fs::remove_dir_all(&dir);
        // Just Quit (single-player): no save, the world-select hop, the
        // player parked at the origin (`:6656`).
        let mut app = stocked_app();
        app.player = Some(crate::player::Player::spawn(&app.world));
        app.audio.insert_test_cue("menu_accept");
        let _ = app.audio.play_tracked_global(style.clone());
        app.world.fast_forward_to_dawn = true;
        app.just_quit();
        assert_eq!(app.audio.liquid_ambience_stop_count(), 1, "the :6631 pair");
        assert_eq!(app.audio.tracked_count_for(&style), 0, "the :6646 drain");
        assert!(matches!(
            app.front,
            Some(crate::app::flow::FrontScreen::WorldSelect)
        ), "GoToWorldSelect (:6655)");
        let p = app.player.as_ref().unwrap();
        assert_eq!((p.x, p.y), (0.0, 0.0), "the position reset (:6656)");
        assert!(!app.world.fast_forward_to_dawn, "the :6649 clear");
        assert!(matches!(app.net, crate::app::NetRole::Off), "netMode 0");
    }
}
mod chest_save_tests {
    use super::*;
    use crate::app::WorldSections;
    use crate::assets;
    use crate::item::{EMPTY_SLOT, ItemSlot};
    use crate::npc::{Npc, NpcKind};
    use crate::player::Player;
    use crate::registry;
    use crate::settings;
    use std::path::PathBuf;
    use tr_core::World;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// A per-test scratch dir under the system temp dir (the shared
    /// `test_world_helpers` helper — unique per pid + test name, so parallel
    /// gates in sibling worktrees never share a path).
    fn scratch(name: &str) -> std::path::PathBuf {
        crate::test_world_helpers::test_scratch(&format!("saveglue-{name}"))
    }

    /// The `chest_gen` test rig's world: a floor row at y = 200, so the anchors
    /// on y = 197/198 sit on it and inside the surface band.
    fn chest_rig() -> World {
        let mut w = World::new("chests".into(), 200, 300);
        w.world_surface = 200.0;
        w.rock_layer = 240.0;
        w.seed = 42;
        w.frame_important = vec![false; tr_core::tiledata::TILE_COUNT];
        // `Main.tileFrameImportant[21] = true` (`Main.cs:7656`): the framer
        // keeps the chest object's own 2x2 frames — without the bit the load's
        // `frame_world` pass rewrites the anchor cell and the roll cannot see
        // it.
        w.frame_important[21] = true;
        for x in 0..200 {
            let i = w.index(x, 200);
            w.tiles[i] = Tile {
                kind: 1,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        // The NPC spawners' surface read ('s
        // ) needs the column cache —  does not
        // compute it and the load path computes it before .
        w.compute_column_surface();
        w
    }

    /// The 2x2 chest tile object (`chest_gen`'s `put_chest`): the anchor cell
    /// carries frames 0/0, the pair `generate_world_chests` scans for.
    fn put_chest(world: &mut World, x: i32, y: i32) {
        for (dx, dy, fx, fy) in [
            (0i32, 0i32, 0i16, 0i16),
            (1, 0, 18, 0),
            (0, 1, 0, 18),
            (1, 1, 18, 18),
        ] {
            let i = world.index(x + dx, y + dy);
            world.tiles[i] = Tile {
                kind: 21,
                frame_x: fx,
                frame_y: fy,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
    }

    /// A headless `App` over the world and its loaded records — the real
    /// constructor, with an empty atlas and blank lookups (all the headless
    /// path reads).
    fn app_for(world: World, sections: WorldSections, path: PathBuf) -> App {
        App::new(
            settings::Settings::default(),
            world,
            sections,
            path,
            tr_render::Atlas::build(Vec::new(), 8192),
            None,
            assets::Lookups::default(),
            (0, 0, 0, 0),
        )
    }

    /// DW-2.2: the save → load round-trip through the App path — a stocked
    /// chest, an interior air and a prefixed item plus a renamed chest survive
    /// `save_world`/`load_world`/`App::new`. S1-A-4: the store is the file's
    /// own records (the gen-time loot the save carried) — the fixture plays
    /// the generated world's records with empty slots.
    #[test]
    fn dw_2_2_a_saved_chest_round_trips_through_the_app() {
        let (ax, ay) = (55, 197); // the stocked chest
        let (bx, by) = (120, 197); // the renamed chest
        let mut world = chest_rig();
        put_chest(&mut world, ax, ay);
        put_chest(&mut world, bx, by);
        let dir = scratch("dw-2-2-chest-roundtrip");
        let path = dir.join("world.trwld");

        let mut app = app_for(
            world,
            WorldSections::native(save::WorldSide {
                chests: vec![
                    save::ChestRecord {
                        x: ax,
                        y: ay,
                        name: String::new(),
                        slots: Vec::new(),
                    },
                    save::ChestRecord {
                        x: bx,
                        y: by,
                        name: String::new(),
                        slots: Vec::new(),
                    },
                ],
                ..Default::default()
            }),
            path.clone(),
        );
        // The gen-time record is the store entry (nothing rolls at load),
        // written through `set_chest_slot` — including the interior air at
        // slot 1 (the store keeps it explicit, `SaveChests`'s
        // unwritten-pair arm covers it).
        let stocked =
            chest::find_chest(&app.world_chests, ax, ay).expect("the anchor's record materialised");
        // The roll's own rows are wiped first (a chest the player emptied,
        // `ItemSlot.SetDefaults(0)`), so the saved stock is exactly what the
        // three writes below leave.
        app.world_chests[stocked].slots.clear();
        for (slot, item) in [
            (
                0usize,
                ItemSlot {
                    kind: 9,
                    name: registry::name(9),
                    count: 6,
                    prefix: 0,
                },
            ),
            (1, EMPTY_SLOT),
            (
                2,
                ItemSlot {
                    kind: 3507,
                    name: registry::name(3507),
                    count: 1,
                    prefix: 84,
                },
            ),
        ] {
            assert!(chest::set_chest_slot(
                &mut app.world_chests,
                stocked,
                slot,
                item
            ));
        }
        // The renamed chest takes the entry down and re-creates it, the
        // `CreateChest` call the open path makes for a tile the roll missed.
        let rolled_b =
            chest::find_chest(&app.world_chests, bx, by).expect("the second anchor rolled");
        chest::remove_chest(&mut app.world_chests, rolled_b);
        let renamed = chest::create_chest(&mut app.world_chests, bx, by);
        assert!(chest::set_chest_slot(
            &mut app.world_chests,
            renamed,
            0,
            ItemSlot {
                kind: 71,
                name: registry::name(71),
                count: 3,
                prefix: 0
            },
        ));
        // `ChestUI.RenameChestSubmit`'s write (`:853-860`) into the entity's
        // name (M6-SAVE-1 phase 3's fold).
        app.world_chests[renamed].name = "Gold Stash".into();
        // Keep the scratch save clean: no player sidecar and no map sidecar.
        app.player = None;
        app.settings.map_enabled = false;
        app.save_world();

        let (world, sections) = crate::app::load_world(&path).expect("the saved world reloads");
        let _ = std::fs::remove_file(&path);
        assert_eq!(
            sections.native.chests.len(),
            2,
            "one record per live store entry"
        );
        let saved_b = sections
            .native
            .chests
            .iter()
            .find(|r| (r.x, r.y) == (bx, by))
            .expect("the renamed chest's record");
        assert_eq!(
            &saved_b.name, "Gold Stash",
            "the effective name rides the record"
        );
        assert_eq!(saved_b.slots[0].kind, 71);

        // S1-A-4: nothing re-rolls at load — the store IS the file's records
        // (the seeded store wins by construction), so the old "the roll
        // stocks the anchor" control is structural now.

        let back = app_for(world, sections, path);
        let stocked =
            chest::find_chest(&back.world_chests, ax, ay).expect("the saved anchor survives");
        assert_eq!(
            back.world_chests[stocked].slots,
            vec![
                ItemSlot {
                    kind: 9,
                    name: registry::name(9),
                    count: 6,
                    prefix: 0
                },
                EMPTY_SLOT, // the interior air
                ItemSlot {
                    kind: 3507,
                    name: registry::name(3507),
                    count: 1,
                    prefix: 84
                },
            ],
            "the seeded slots replace the roll exactly — no band rows appended"
        );
        let renamed =
            chest::find_chest(&back.world_chests, bx, by).expect("the renamed chest survives");
        assert_eq!(
            chest::chest_slot(&back.world_chests, renamed, 0),
            Some(ItemSlot {
                kind: 71,
                name: registry::name(71),
                count: 3,
                prefix: 0
            })
        );
        assert_eq!(chest::chest_name(&back, renamed as i32), "Gold Stash");
        assert_eq!(
            back.world_chests[renamed].name, "Gold Stash",
            "the name rides the entity (the old side map is gone)"
        );
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// DW-2.2: the anchor cull — the vanilla `LoadChests`/`FixAgainstExploits`
    /// validity sweep. A record whose `(x, y)` is not an active `BasicChest`
    /// tile of kind 21/467 (`TileID.cs:570`) in the loaded world is dropped,
    /// name and slots with it.
    #[test]
    fn dw_2_2_a_record_with_a_missing_anchor_is_dropped() {
        let (ax, ay) = (55, 197);
        let (bx, by) = (120, 197);
        let mut world = chest_rig();
        put_chest(&mut world, ax, ay);
        put_chest(&mut world, bx, by);
        // The stale record's anchor: the tile is gone (a broken chest's cell
        // after an explosion, say).
        let i = world.index(bx, by);
        world.tiles[i] = Tile::default();

        let records = vec![
            ChestRecord {
                x: ax,
                y: ay,
                name: "Vault".into(),
                slots: vec![ChestItem {
                    stack: 12,
                    kind: 166,
                    prefix: 0,
                }],
            },
            ChestRecord {
                x: bx,
                y: by,
                name: "Ghost".into(),
                slots: vec![ChestItem {
                    stack: 5,
                    kind: 9,
                    prefix: 0,
                }],
            },
        ];
        let app = app_for(
            world,
            WorldSections::native(save::WorldSide {
                chests: records,
                banners: Vec::new(),
                bestiary: Vec::new(),
                npcs: Vec::new(),
                ore_tiers: None,
            }),
            PathBuf::new(),
        );

        let idx =
            chest::find_chest(&app.world_chests, ax, ay).expect("the valid record seeds the store");
        assert_eq!(
            chest::chest_slot(&app.world_chests, idx, 0),
            Some(ItemSlot {
                kind: 166,
                name: registry::name(166),
                count: 12,
                prefix: 0
            })
        );
        assert_eq!(chest::chest_name(&app, idx as i32), "Vault");
        // The culled record leaves no entry and no name behind.
        assert!(
            chest::find_chest(&app.world_chests, bx, by).is_none(),
            "an anchor over plain air carries no chest"
        );
        assert!(
            !app.world_chests.iter().any(|c| c.name == "Ghost"),
            "the culled record's name lands nowhere"
        );
        assert_eq!(
            app.world_chests
                .iter()
                .filter(|c| !c.name.is_empty())
                .count(),
            1,
            "only the surviving record's name rides an entity"
        );
    }

    /// DW-2.3: the save → load round-trip of the banner tally through the App
    /// path — the `BannerSystem.Save` counts (`BannerSystem.cs:172-186`) the
    /// kill pass accumulates survive `save_world`/`load_world`/`App::new`, and
    /// an empty tally loads none.
    #[test]
    fn dw_2_3_a_saved_banner_tally_round_trips_through_the_app() {
        let mut world = chest_rig();
        let dir = scratch("dw-2-3-banner-roundtrip");
        let path = dir.join("world.trwld");

        let mut app = app_for(
            world,
            WorldSections::native(save::WorldSide::default()),
            path.clone(),
        );
        // Two NPC families (the Blue Slime 1 → the banner 69, the Green Slime
        // -4 → the banner 151, `bestiary::npc_to_banner`) — the production
        // call the death pass makes (`world_sim/npcs.rs`'s kill hook).
        for _ in 0..7 {
            app.banners.add_npc_kill_by(1, "Tester");
        }
        for _ in 0..3 {
            app.banners.add_npc_kill_by(-4, "Tester");
        }
        app.player = None;
        app.settings.map_enabled = false;
        app.save_world();

        let (world, sections) = crate::app::load_world(&path).expect("the saved world reloads");
        let _ = std::fs::remove_file(&path);
        assert_eq!(
            sections.native.banners.len(),
            2,
            "one record per tallied banner"
        );
        let back = app_for(world, sections, path);
        let pairs: Vec<(i32, i32)> = back.banners.pairs().collect();
        assert_eq!(pairs, vec![(69, 7), (151, 3)], "the counts ride the store");

        // The control: a fresh App over the same world starts empty — the
        // records above are what carried the tally, not a re-derived one.
        let mut world2 = chest_rig();
        let control = app_for(
            world2.clone(),
            WorldSections::native(save::WorldSide::default()),
            PathBuf::new(),
        );
        assert_eq!(control.banners.pairs().count(), 0, "no records — no tally");
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// DW-2.4: the bestiary tally's save → load round-trip through the App
    /// path — the `NPCKillsTracker.Save` counts the kill pass accumulates ride
    /// the world file now (the vanilla's section-10 home; the port had them on
    /// the `.trplayer` sidecar), and the loaded side seeds the tracker.
    #[test]
    fn dw_2_4_a_saved_bestiary_tally_round_trips_through_the_app() {
        let world = chest_rig();
        let dir = scratch("dw-2-4-bestiary-roundtrip");
        let path = dir.join("world.trwld");

        let mut app = app_for(
            world,
            WorldSections::native(save::WorldSide::default()),
            path.clone(),
        );
        // The kill pass's registration (`world_sim/npcs.rs`'s death hook):
        // two credit ids, one of them past the 999999999 clamp range's floor.
        for _ in 0..9 {
            app.bestiary.kills.register_kill("BlueSlime");
        }
        app.bestiary.kills.register_kill("KingSlime");
        app.player = None;
        app.settings.map_enabled = false;
        app.save_world();

        let (world, sections) = crate::app::load_world(&path).expect("the saved world reloads");
        let _ = std::fs::remove_file(&path);
        assert_eq!(
            sections.native.bestiary.len(),
            2,
            "one record per tallied credit id"
        );
        let back = app_for(world, sections, path);
        assert_eq!(back.bestiary.kills.get_kill_count("BlueSlime"), 9);
        assert_eq!(back.bestiary.kills.get_kill_count("KingSlime"), 1);

        // The control: a fresh App starts empty — the records carried the tally.
        let control = app_for(
            chest_rig(),
            WorldSections::native(save::WorldSide::default()),
            PathBuf::new(),
        );
        assert_eq!(control.bestiary.kills.get_kill_count("BlueSlime"), 0);
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// S1-J-7: the rain event's save → load round-trip through the App path —
    /// the C#'s `SetTempToOngoing` (`WorldFile.cs:1058-1060`) snapshots the
    /// live state before the write and `SetOngoingToTemps` (`:159-162`)
    /// restores it on load, `Main.cloudAlpha` := `maxRaining`. The port's
    /// `save_world` syncs the live `Rain` into the world's file payload and
    /// the loaded world re-seeds the `Rain` (`enter_loaded_world`).
    #[test]
    fn s1_j_7_a_saved_rain_event_round_trips_through_the_app() {
        let world = chest_rig();
        let dir = scratch("s1-j-7-rain-roundtrip");
        let path = dir.join("world.trwld");

        let mut app = app_for(
            world,
            WorldSections::native(save::WorldSide::default()),
            path.clone(),
        );
        // The load-time restore shape (S1-J-7's own seam) doubles as the
        // event setup: an active rain event at 0.75 target.
        app.rain.restore_saved(true, 12_345, 0.75);
        app.player = None;
        app.settings.map_enabled = false;
        app.save_world();

        let (world, sections) = crate::app::load_world(&path).expect("the saved world reloads");
        let _ = std::fs::remove_file(&path);
        // The file payload carried the snapshot.
        assert!(world.raining && world.rain_time == 12_345, "the file block");
        assert!((world.max_raining - 0.75).abs() < f32::EPSILON);
        // The reload applies it (`SetOngoingToTemps`'s effect).
        let back = app_for(world, sections, path);
        assert!(back.rain.is_raining(), "the event survives the reload");
        assert_eq!(back.rain.rain_time(), 12_345);
        assert!((back.rain.max_raining() - 0.75).abs() < f32::EPSILON);
        assert!(
            (back.rain.visual_alpha() - 0.75).abs() < f32::EPSILON,
            "cloudAlpha := maxRaining (WorldFile.cs:162)"
        );
        crate::test_world_helpers::remove_scratch(&dir);
    }

    #[test]
    fn s1_j_7_the_wld_weather_snapshot_round_trips_through_the_app() {
        // The `.wld` flags block: `SaveWorldFlags:1312-1314` (rain) and
        // `:1369-1372` (sandstorm) write the save-time temps, filled by
        // `SetTempToOngoing:1053-1059`; the loader restores them through
        // `SetOngoingToTemps:159-172`. Both events are ported (`App.rain`,
        // `App.sandstorm`), so a `.wld` save must not reset the weather.
        let world = chest_rig();
        let dir = scratch("s1-j-7-wld-weather-roundtrip");
        let path = dir.join("world.wld");

        let mut app = app_for(
            world,
            WorldSections::native(save::WorldSide::default()),
            path.clone(),
        );
        app.rain.restore_saved(true, 12_345, 0.75);
        app.sandstorm.happening = true;
        app.sandstorm.time_left = 4_321;
        app.sandstorm.severity = 0.5;
        app.sandstorm.intended_severity = 0.625;
        app.player = None;
        app.settings.map_enabled = false;
        app.save_world();

        // The file carried the snapshot (the `.wld` on disk, not the live App).
        let file = tr_core::wld::load(&path).expect("the saved .wld reads back");
        assert!(file.raining && file.rain_time == 12_345, "the rain block");
        assert!((file.max_raining - 0.75).abs() < f32::EPSILON);
        assert!(file.sandstorm_happening, "the sandstorm block");
        assert_eq!(file.sandstorm_time_left, 4_321);
        assert!((file.sandstorm_severity - 0.5).abs() < f32::EPSILON);
        assert!((file.sandstorm_intended_severity - 0.625).abs() < f32::EPSILON);

        // The reload applies it (`SetOngoingToTemps`'s effect).
        let (world, sections) = crate::app::load_world(&path).expect("the saved world reloads");
        let _ = std::fs::remove_file(&path);
        let back = app_for(world, sections, path);
        assert!(back.rain.is_raining(), "the rain event survives the reload");
        assert_eq!(back.rain.rain_time(), 12_345);
        assert!(back.sandstorm.happening, "the sandstorm survives the reload");
        assert_eq!(back.sandstorm.time_left, 4_321);
        assert!((back.sandstorm.severity - 0.5).abs() < f32::EPSILON);
        assert!((back.sandstorm.intended_severity - 0.625).abs() < f32::EPSILON);
        crate::test_world_helpers::remove_scratch(&dir);
    }

    #[test]
    fn s1_j_4_two_saves_rotate_the_previous_world_into_the_bak_chain() {
        // `DoRollingBackups` (`WorldFile.cs:966-1004`) + the `.bak` write
        // (`:958-962`): the previous save's bytes land in `.bak` and the
        // chain shifts into `.bak2` (the C#'s own slot numbering — the bare
        // `.bak` is slot 1 and the numbered slots start at 2).
        let world = chest_rig();
        let dir = scratch("s1-j-4-rotation");
        let path = dir.join("world.trwld");

        let mut app = app_for(
            world,
            WorldSections::native(save::WorldSide::default()),
            path.clone(),
        );
        app.player = None;
        app.settings.map_enabled = false;
        app.save_world();
        let first = std::fs::read(&path).unwrap();

        // The second save mutates the world (the flag flip rides the file) —
        // the first save's bytes become `.bak`.
        app.world.downed_queen_slime = true;
        app.save_world();
        let second = std::fs::read(&path).unwrap();
        assert_ne!(first, second, "the two saves differ");
        assert_eq!(
            std::fs::read(backup_path(&path, 0)).unwrap(),
            first,
            ".bak carries the previous save"
        );
        // The third save shifts `.bak` into `.bak2` (the C#'s numbering) and
        // writes the second save into `.bak`.
        app.save_world();
        assert_eq!(
            std::fs::read(&path).unwrap(),
            second,
            "the fresh file carries the latest save"
        );
        assert_eq!(
            std::fs::read(backup_path(&path, 0)).unwrap(),
            second,
            ".bak carries the previous save"
        );
        assert_eq!(
            std::fs::read(backup_path(&path, 2)).unwrap(),
            first,
            ".bak2 carries the two-saves-ago bytes"
        );
        crate::test_world_helpers::remove_scratch(&dir);
    }

    #[test]
    fn s1_j_4_the_written_world_passes_the_post_write_validation() {
        // `InternalSaveWorld`'s post-write walk (`WorldFile.cs:941-948`): the
        // written bytes re-read through the full parser — a real save
        // validates and a corrupted file does not (the failure arm restores
        // the captured previous bytes via the same helpers).
        let world = chest_rig();
        let dir = scratch("s1-j-4-validate");
        let path = dir.join("world.trwld");
        let mut app = app_for(
            world,
            WorldSections::native(save::WorldSide::default()),
            path.clone(),
        );
        app.player = None;
        app.settings.map_enabled = false;
        app.save_world();
        let old_world = std::fs::read(&path).unwrap();
        assert!(validate_world_bytes(&path, |b| tr_core::save::parse(b).is_ok()));
        // A corrupted write fails the walk.
        std::fs::write(&path, b"garbage").unwrap();
        assert!(!validate_world_bytes(&path, |b| tr_core::save::parse(b).is_ok()));
        // The failure arm's restore: the previous bytes return.
        std::fs::write(&path, &old_world).unwrap();
        assert!(validate_world_bytes(&path, |b| tr_core::save::parse(b).is_ok()));
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// DW-2.4 (the back-compat fallback): a world saved before v9 (the
    /// version field patched) carries no bestiary section, and the player
    /// sidecar's kill line still seeds the tracker — the deviation's original
    /// home stays the fallback for migrated builds.
    #[test]
    fn dw_2_4_a_v8_world_falls_back_to_the_sidecar_kills() {
        let dir = scratch("dw-2-4-sidecar-fallback");
        let path = dir.join("world.trwld");
        // A v8 payload: the current writer's stream with the version patched.
        let mut app = app_for(
            chest_rig(),
            WorldSections::native(save::WorldSide::default()),
            path.clone(),
        );
        app.player = None;
        app.settings.map_enabled = false;
        app.save_world();
        let mut bytes = std::fs::read(&path).expect("the v9 payload on disk");
        bytes[8..12].copy_from_slice(&8u32.to_le_bytes());
        std::fs::write(&path, &bytes).expect("the v8 payload rewritten");

        // The sidecar with the vintage kill line (`persist::save`'s kills arg).
        let player = Player::spawn(&chest_rig());
        let inv: [(u16, u32); 50] = std::array::from_fn(|_| (0, 0));
        let kills = vec![("Pinky".to_string(), 4), ("BlueSlime".to_string(), 25)];
        persist::save(&path, &player, inv, 0, [0, 0, 0], &[0; 10], &kills, 0)
            .expect("the sidecar writes");

        let (world, sections) = crate::app::load_world(&path).expect("the v8 world reloads");
        let _ = std::fs::remove_file(&path);
        assert!(sections.native.bestiary.is_empty(), "no section below v9");
        let back = app_for(world, sections, path);
        assert_eq!(
            back.bestiary.kills.get_kill_count("Pinky"),
            4,
            "the sidecar's line seeds the tracker"
        );
        assert_eq!(back.bestiary.kills.get_kill_count("BlueSlime"), 25);
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// DW-2.5: the town roster's save → load round-trip through the App path —
    /// `SaveNPCs`'s block-B records (`WorldFile.cs:1703-1750`) restore the
    /// town NPC with its given name, its home and its position, and a fresh
    /// world (no records) keeps the vanilla-gen Guide arrival.
    #[test]
    fn dw_2_5_a_town_npc_round_trips_through_the_app() {
        let dir = scratch("dw-2-5-npc-roundtrip");
        let path = dir.join("world.trwld");

        let mut app = app_for(
            chest_rig(),
            WorldSections::native(save::WorldSide::default()),
            path.clone(),
        );
        // A housed Merchant with a given name at a solid column (the town
        // spawner's surface scan) — the rig's floor row at y=200.
        let x_tile = 100;
        let mut merchant = Npc::spawn_town(&app.world, x_tile, NpcKind::Merchant)
            .expect("the town spawner's surface scan");
        merchant.given_name = "Bsmith".into();
        merchant.home = Some((x_tile, 190));
        app.npcs.push(merchant);
        app.player = None;
        app.settings.map_enabled = false;
        app.save_world();

        let (world, sections) = crate::app::load_world(&path).expect("the saved world reloads");
        let _ = std::fs::remove_file(&path);
        assert_eq!(
            sections.native.npcs.len(),
            2,
            "the seeded Merchant plus the vanilla-gen Guide both ride the save"
        );
        let record = sections
            .native
            .npcs
            .iter()
            .find(|r| !r.given_name.is_empty())
            .expect("the given-name record is the Merchant");
        assert_eq!(record.given_name, "Bsmith");
        assert!(!record.homeless);
        assert_eq!((record.home_x, record.home_y), (x_tile, 190));

        let back = app_for(world, sections, path);
        let restored = back
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::Merchant)
            .expect("the merchant rides the load");
        assert_eq!(restored.given_name, "Bsmith", "the given name survives");
        assert_eq!(restored.home, Some((x_tile, 190)), "the home survives");
        // The reload's roster starts from the saved records — the home-less
        // Guide rides its record (the arrival pass's absence check never
        // spawns a second one), and the other town kinds still arrive on
        // their gates. Exactly one Guide and one Merchant exist either way.
        let guides = back
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::Guide)
            .count();
        assert_eq!(guides, 1, "the saved Guide blocks the second arrival");
        let merchants = back
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::Merchant)
            .count();
        assert_eq!(merchants, 1, "the saved Merchant blocks the second arrival");
        let guide = back
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::Guide)
            .expect("the saved Guide rides the record");
        assert!(guide.home.is_none(), "the Guide's recorded homelessness");
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// DW-2.5 (the back-compat arm): a world without NPC records (a fresh
    /// world, or a pre-v10 save) still spawns the vanilla-gen homeless Guide
    /// at the spawn point.
    #[test]
    fn dw_2_5_a_recordless_world_spawns_the_guide() {
        let mut world = chest_rig();
        world.spawn_x = 50;
        let path = PathBuf::new();
        let app = app_for(
            world,
            WorldSections::native(save::WorldSide::default()),
            path,
        );
        let guide = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::Guide)
            .expect("the vanilla-gen Guide arrival");
        assert!(guide.home.is_none(), "the arrival Guide is homeless");
    }

    /// DW-2.6: the saved ore tiers' save → load round-trip through the App
    /// path — the altar smash's hardmode tiers (`WorldGen.SavedOreTiers`,
    /// `WorldGen.cs:3283-3297`) survive, where the static defaults would have
    /// reset them (the recorded pre-fix loss).
    #[test]
    fn dw_2_6_the_ore_tiers_round_trip_through_the_app() {
        let dir = scratch("dw-2-6-ore-tiers");
        let path = dir.join("world.trwld");

        let mut app = app_for(
            chest_rig(),
            WorldSections::native(save::WorldSide::default()),
            path.clone(),
        );
        // The altar smash's outcome (the App's runtime copy `smash_altar`
        // advances): the hardmode trio flips off the -1 sentinels.
        app.ore_tiers.cobalt = 107;
        app.ore_tiers.mythril = 108;
        app.ore_tiers.adamantite = 111;
        app.ore_tiers.copper = 166; // the alternate-metal flip
        app.player = None;
        app.settings.map_enabled = false;
        app.save_world();

        let (world, sections) = crate::app::load_world(&path).expect("the saved world reloads");
        let tiers = sections
            .native
            .ore_tiers
            .expect("the v11 section rides the tiers");
        assert_eq!(tiers, [166, 6, 9, 8, 107, 108, 111]);
        assert_eq!(world.ore_tiers.cobalt, 107);

        let back = app_for(world, sections, path);
        assert_eq!(back.ore_tiers.copper, 166, "the alternate flip survives");
        assert_eq!(
            back.ore_tiers.cobalt, 107,
            "the smashed hardmode tier survives"
        );
        assert_eq!(back.ore_tiers.mythril, 108);
        assert_eq!(back.ore_tiers.adamantite, 111);
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// DW-2.6 (the back-compat arm): a world whose save carries no ore section
    /// (pre-v11) starts the runtime copy at the static defaults — the C#'s own
    /// reset arm (`WorldFile.cs:796-814`).
    #[test]
    fn dw_2_6_a_v10_world_starts_at_the_static_defaults() {
        let mut world = chest_rig();
        world.ore_tiers.copper = 166; // a generation-time roll the old save dropped
        let dir = scratch("dw-2-6-defaults");
        let path = dir.join("world.trwld");
        let mut app = app_for(
            world,
            WorldSections::native(save::WorldSide::default()),
            path.clone(),
        );
        app.player = None;
        app.settings.map_enabled = false;
        app.save_world();
        let mut bytes = std::fs::read(&path).expect("the v11 payload on disk");
        bytes[8..12].copy_from_slice(&10u32.to_le_bytes());
        std::fs::write(&path, &bytes).expect("the v10 payload rewritten");

        let (world, sections) = crate::app::load_world(&path).expect("the v10 world reloads");
        let _ = std::fs::remove_file(&path);
        assert!(sections.native.ore_tiers.is_none());
        assert_eq!(world.ore_tiers, tr_core::ores::SavedOreTiers::new());
        let back = app_for(world, sections, PathBuf::new());
        assert_eq!(
            back.ore_tiers.copper, 7,
            "the static default, not the dropped roll"
        );
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// A per-test scratch dir for the achievements writes (the sibling `tests`
    /// module's pattern).
    fn ach_scratch(name: &str) -> std::path::PathBuf {
        crate::test_world_helpers::test_scratch(&format!("saveglue-ach-{name}"))
    }

    /// S1-E-18: `Player.InternalSavePlayer`'s first statement is
    /// `Main.Achievements.Save()` (`Player.cs:54700`), so every save route must
    /// write the manager's file — partial condition progress included.
    #[test]
    fn save_world_writes_the_achievement_condition_state() {
        use crate::achievements::{self, AchievementManager, CondKind};
        let dir = ach_scratch("achievements-save");
        let path = dir.join("world.trwld");
        let ach_path = dir.join("achievements.trdat");
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.world_path = path.clone();
        app.player = None;
        app.settings.map_enabled = false;
        app.achievements.save_path = Some(ach_path.clone());
        // One of SLAYER_OF_WORLDS' NPC conditions — the row itself stays open,
        // so only the save call site can persist the partial state.
        app.achievements.notify(CondKind::NpcKilled(13));
        assert!(
            !app.achievements
                .list
                .iter()
                .any(|a| a.name == "SLAYER_OF_WORLDS" && a.completed),
            "the row stays open, so only a save call site can write this"
        );
        app.save_world();
        assert!(ach_path.exists(), "save_world wrote the achievements file");
        let mut fresh = AchievementManager::registry();
        achievements::load(&ach_path, &mut fresh);
        let slayer = fresh
            .list
            .iter()
            .find(|a| a.name == "SLAYER_OF_WORLDS")
            .unwrap();
        assert!(
            slayer.conditions[0].completed,
            "the partial condition rode the save"
        );
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// `AchievementManager.AchievementCompleted` (`AchievementManager.cs:180-187`)
    /// saves on the completion event itself — the second vanilla call site.
    #[test]
    fn completing_an_achievement_writes_the_file_immediately() {
        use crate::achievements::{self, AchievementManager, CondKind};
        let dir = ach_scratch("achievements-completion");
        let ach_path = dir.join("achievements.trdat");
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.achievements.save_path = Some(ach_path.clone());
        assert_eq!(
            app.achievements.notify(CondKind::NpcKilled(4)),
            Some("EYE_ON_YOU")
        );
        assert!(ach_path.exists(), "the completion hook wrote the file");
        let mut fresh = AchievementManager::registry();
        achievements::load(&ach_path, &mut fresh);
        assert!(
            fresh
                .list
                .iter()
                .any(|a| a.name == "EYE_ON_YOU" && a.completed),
            "the completed row is on disk"
        );
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// W4-c36 — the row-targeted `complete_condition` saves on completion
    /// too (`AchievementManager.cs:180-187` — the `AchievementCompleted`
    /// event fires from `Achievement.OnConditionComplete` regardless of
    /// which hook closed the last condition).
    #[test]
    fn w4_c36_the_row_targeted_completion_writes_the_file() {
        use crate::achievements::{self, AchievementManager};
        let dir = ach_scratch("achievements-row-targeted");
        let ach_path = dir.join("achievements.trdat");
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.achievements.save_path = Some(ach_path.clone());
        assert_eq!(
            app.achievements.complete_condition("I_AM_LOOT", "Peek"),
            Some("I_AM_LOOT")
        );
        assert!(ach_path.exists(), "the completion hook wrote the file");
        let mut fresh = AchievementManager::registry();
        achievements::load(&ach_path, &mut fresh);
        assert!(
            fresh.list.iter().any(|a| a.name == "I_AM_LOOT" && a.completed),
            "the completed row is on disk"
        );
        crate::test_world_helpers::remove_scratch(&dir);
    }

    /// W4-c36 — the save-route audit: `Player.InternalSavePlayer`'s first
    /// statement (`Player.cs:54700`) is the ONLY external `Save()` call site
    /// in the C# (the others are the manager's own completion/Clear paths).
    /// The port's `save_and_quit` funnels through `save_world`
    /// (`WorldGen.cs:6669-6677`'s `Player.SavePlayer` + `WorldFile.SaveWorld`
    /// pair), so the quit route writes the achievements file too.
    #[test]
    fn w4_c36_save_and_quit_writes_the_achievement_state() {
        use crate::achievements::{self, AchievementManager, CondKind};
        let dir = ach_scratch("achievements-quit");
        let path = dir.join("world.trwld");
        let ach_path = dir.join("achievements.trdat");
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.world_path = path.clone();
        app.player = None;
        app.settings.map_enabled = false;
        app.achievements.save_path = Some(ach_path.clone());
        // Partial progress only — one of SLAYER_OF_WORLDS' 15 conditions,
        // the row stays open so NO completion event fires; only the quit
        // route's save call can persist this.
        app.achievements.notify(CondKind::NpcKilled(13));
        assert!(!ach_path.exists(), "no completion save fired");
        app.save_and_quit();
        assert!(ach_path.exists(), "the quit route wrote the achievements file");
        let mut fresh = AchievementManager::registry();
        achievements::load(&ach_path, &mut fresh);
        let slayer = fresh
            .list
            .iter()
            .find(|a| a.name == "SLAYER_OF_WORLDS")
            .unwrap();
        assert!(slayer.conditions[0].completed, "the partial condition rode the quit save");
        assert!(!slayer.completed, "the row stays open");
        crate::test_world_helpers::remove_scratch(&dir);
    }
}
