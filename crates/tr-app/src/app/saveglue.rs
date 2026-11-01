//! The App's save glue — the world/player autosave, the explored-map write
//! and its load-side twin. (A6: extracted from `main.rs`.)

use crate::app::App;
use crate::map;
use crate::persist;
use tr_core::save;

impl App {
    pub(crate) fn save_world(&self) {
        if self.world_path.extension().is_some_and(|e| e == "trwld") {
            match save::save(&self.world_path, &self.world) {
                Ok(()) => log::info!("saved {}", self.world_path.display()),
                Err(e) => log::warn!("save failed: {e:#}"),
            }
            if let Some(p) = &self.player {
                let inv =
                    std::array::from_fn(|i| (self.inventory[i].kind, self.inventory[i].count));
                if let Err(e) = persist::save(&self.world_path, p, inv, self.coins) {
                    log::warn!("player save failed: {e}");
                }
            }
            // `Player.InternalSaveMap` (`Player.cs:55046`): the explored map is
            // written alongside the player/world save.
            self.save_map();
        } else {
            log::info!("not saving: the loaded world is not a native .trwld");
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
        if !self.world_path.extension().is_some_and(|e| e == "trwld")
            || !self.settings.map_enabled
        {
            return;
        }
        let path = persist::map_path(&self.world_path);
        let Some(m) = self.map.as_mut() else { return };
        if m.load(&self.world, &path) {
            log::info!("map: loaded {}", path.display());
        }
    }
}
