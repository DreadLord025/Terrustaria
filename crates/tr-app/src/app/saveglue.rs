//! The App's save glue — the world/player autosave, the explored-map write
//! and its load-side twin. (A6: extracted from `main.rs`.)

use crate::app::App;
use crate::chest;
use crate::map;
use crate::persist;
use tr_core::save::{self, BannerKill, ChestItem, ChestRecord};

impl App {
    pub(crate) fn save_world(&self) {
        if self.world_path.extension().is_some_and(|e| e == "trwld") {
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
                    // The effective `Chest.name`: the rename submit's stored
                    // name, or the empty string a never-renamed chest carries —
                    // the tile default is derived, never stored.
                    name: chest::chest_name(self, idx as i32).to_string(),
                    // The store's implicit-air compression survives verbatim:
                    // the trailing airs are already dropped and the interior
                    // ones are explicit `stack == 0` entries (`Item.IsAir`'s
                    // type-0/stack-0 pair, `Item.cs:385-395`).
                    slots: chest
                        .slots
                        .iter()
                        .map(|slot| {
                            if slot.kind == 0 || slot.count == 0 {
                                ChestItem { stack: 0, kind: 0, prefix: 0 }
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
            let side = save::WorldSide { chests, banners };
            match save::save(&self.world_path, &self.world, &side) {
                Ok(()) => log::info!("saved {}", self.world_path.display()),
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
                    .kill_tracker
                    .pairs()
                    .map(|(id, count)| (id.to_string(), count))
                    .collect();
                if let Err(e) =
                    persist::save(&self.world_path, p, inv, self.coins, armor, &dye, &kills)
                {
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
        if !self.world_path.extension().is_some_and(|e| e == "trwld") || !self.settings.map_enabled
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

#[cfg(test)]
mod chest_save_tests {
    use super::*;
    use crate::assets;
    use std::path::PathBuf;
    use crate::item::{ItemSlot, EMPTY_SLOT};
    use crate::registry;
    use crate::settings;
    use tr_core::tile::{FLAG_ACTIVE, Tile};
    use tr_core::World;

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
    fn app_for(world: World, side: save::WorldSide, path: PathBuf) -> App {
        App::new(
            settings::Settings::default(),
            world,
            side,
            path,
            tr_render::Atlas::build(Vec::new(), 8192),
            None,
            assets::Lookups::default(),
            (0, 0, 0, 0),
        )
    }

    /// DW-2.2: the save → load round-trip through the App path — a stocked
    /// chest, an interior air and a prefixed item plus a renamed chest survive
    /// `save_world`/`load_world`/`App::new`, and the save's anchor wins over the
    /// load-time loot re-roll (`merge_store`'s collision rule).
    #[test]
    fn dw_2_2_a_saved_chest_round_trips_through_the_app() {
        let (ax, ay) = (55, 197); // the stocked chest
        let (bx, by) = (120, 197); // the renamed chest
        let mut world = chest_rig();
        put_chest(&mut world, ax, ay);
        put_chest(&mut world, bx, by);
        let path = std::env::temp_dir().join("tr-dw-2-2-chest-roundtrip.trwld");
        let _ = std::fs::remove_file(&path);

        let mut app = app_for(world, save::WorldSide::default(), path.clone());
        // The roll claims every chest tile, so the stocked chest is the rolled
        // entry the open path would have opened anyway, written through
        // `set_chest_slot` — including the interior air at slot 1 (the store
        // keeps it explicit, `SaveChests`'s unwritten-pair arm covers it).
        let stocked = chest::find_chest(&app.world_chests, ax, ay).expect("the anchor rolled");
        // The roll's own rows are wiped first (a chest the player emptied,
        // `ItemSlot.SetDefaults(0)`), so the saved stock is exactly what the
        // three writes below leave.
        app.world_chests[stocked].slots.clear();
        for (slot, item) in [
            (0usize, ItemSlot { kind: 9, name: registry::name(9), count: 6, prefix: 0 }),
            (1, EMPTY_SLOT),
            (
                2,
                ItemSlot { kind: 3507, name: registry::name(3507), count: 1, prefix: 84 },
            ),
        ] {
            assert!(chest::set_chest_slot(&mut app.world_chests, stocked, slot, item));
        }
        // The renamed chest takes the entry down and re-creates it, the
        // `CreateChest` call the open path makes for a tile the roll missed.
        let rolled_b = chest::find_chest(&app.world_chests, bx, by).expect("the second anchor rolled");
        chest::remove_chest(&mut app.world_chests, rolled_b);
        let renamed = chest::create_chest(&mut app.world_chests, bx, by);
        assert!(
            chest::set_chest_slot(
                &mut app.world_chests,
                renamed,
                0,
                ItemSlot { kind: 71, name: registry::name(71), count: 3, prefix: 0 },
            )
        );
        // `ChestUI.RenameChestSubmit`'s write (`:853-860`) into the side map.
        app.chest_names.insert(renamed, "Gold Stash".into());
        // Keep the scratch save clean: no player sidecar and no map sidecar.
        app.player = None;
        app.settings.map_enabled = false;
        app.save_world();

        let (world, side) = crate::app::load_world(&path).expect("the saved world reloads");
        let _ = std::fs::remove_file(&path);
        assert_eq!(side.chests.len(), 2, "one record per live store entry");
        let saved_b = side.chests
            .iter()
            .find(|r| (r.x, r.y) == (bx, by))
            .expect("the renamed chest's record");
        assert_eq!(&saved_b.name, "Gold Stash", "the effective name rides the record");
        assert_eq!(saved_b.slots[0].kind, 71);

        // The control: the same world re-rolls the anchor with its band rows,
        // so the equality below is the seeded store winning, not an empty roll.
        let roll = crate::chest_gen::generate_world_chests(&world);
        let rolled = roll
            .iter()
            .find(|c| c.tile == (ax, ay))
            .expect("the anchor re-rolls on load");
        assert!(
            rolled.slots.len() >= 2 && rolled.slots.len() != 3,
            "the roll stocks the anchor with its own band rows: {:?}",
            rolled.slots
        );

        let back = app_for(world, side, path);
        let stocked = chest::find_chest(&back.world_chests, ax, ay).expect("the saved anchor survives");
        assert_eq!(
            back.world_chests[stocked].slots,
            vec![
                ItemSlot { kind: 9, name: registry::name(9), count: 6, prefix: 0 },
                EMPTY_SLOT, // the interior air
                ItemSlot { kind: 3507, name: registry::name(3507), count: 1, prefix: 84 },
            ],
            "the seeded slots replace the roll exactly — no band rows appended"
        );
        let renamed = chest::find_chest(&back.world_chests, bx, by).expect("the renamed chest survives");
        assert_eq!(
            chest::chest_slot(&back.world_chests, renamed, 0),
            Some(ItemSlot { kind: 71, name: registry::name(71), count: 3, prefix: 0 })
        );
        assert_eq!(chest::chest_name(&back, renamed as i32), "Gold Stash");
        assert_eq!(back.chest_names.len(), 1, "only the renamed chest carries a name");
        assert_eq!(
            back.chest_names.get(&renamed),
            Some(&"Gold Stash".to_string()),
            "the name map is keyed by the post-merge index"
        );
    }

    /// DW-2.2: the anchor cull — the vanilla `LoadChests`/`FixAgainstExploits`
    /// validity sweep. A record whose `(x, y)` is not an active chest tile of
    /// kind 21/341 in the loaded world is dropped, name and slots with it.
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
                slots: vec![ChestItem { stack: 12, kind: 166, prefix: 0 }],
            },
            ChestRecord {
                x: bx,
                y: by,
                name: "Ghost".into(),
                slots: vec![ChestItem { stack: 5, kind: 9, prefix: 0 }],
            },
        ];
        let app = app_for(
            world,
            save::WorldSide { chests: records, banners: Vec::new() },
            PathBuf::new(),
        );

        let idx = chest::find_chest(&app.world_chests, ax, ay).expect("the valid record seeds the store");
        assert_eq!(
            chest::chest_slot(&app.world_chests, idx, 0),
            Some(ItemSlot { kind: 166, name: registry::name(166), count: 12, prefix: 0 })
        );
        assert_eq!(chest::chest_name(&app, idx as i32), "Vault");
        // The culled record leaves no entry and no name behind.
        assert!(
            chest::find_chest(&app.world_chests, bx, by).is_none(),
            "an anchor over plain air carries no chest"
        );
        assert!(
            !app.chest_names.values().any(|n| n == "Ghost"),
            "the culled record's name lands nowhere"
        );
        assert_eq!(app.chest_names.len(), 1);
    }

    /// DW-2.3: the save → load round-trip of the banner tally through the App
    /// path — the `BannerSystem.Save` counts (`BannerSystem.cs:172-186`) the
    /// kill pass accumulates survive `save_world`/`load_world`/`App::new`, and
    /// an empty tally loads none.
    #[test]
    fn dw_2_3_a_saved_banner_tally_round_trips_through_the_app() {
        let mut world = chest_rig();
        let path = std::env::temp_dir().join("tr-dw-2-3-banner-roundtrip.trwld");
        let _ = std::fs::remove_file(&path);

        let mut app = app_for(world, save::WorldSide::default(), path.clone());
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

        let (world, side) = crate::app::load_world(&path).expect("the saved world reloads");
        let _ = std::fs::remove_file(&path);
        assert_eq!(side.banners.len(), 2, "one record per tallied banner");
        let back = app_for(world, side, path);
        let pairs: Vec<(i32, i32)> = back.banners.pairs().collect();
        assert_eq!(pairs, vec![(69, 7), (151, 3)], "the counts ride the store");

        // The control: a fresh App over the same world starts empty — the
        // records above are what carried the tally, not a re-derived one.
        let mut world2 = chest_rig();
        let control = app_for(world2.clone(), save::WorldSide::default(), PathBuf::new());
        assert_eq!(control.banners.pairs().count(), 0, "no records — no tally");
    }
}

