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
            // `NPCKillsTracker.Save` (`NPCKillsTracker.cs:59-70`): the bestiary
            // kill counts ride the world file's section 10 in vanilla; the port
            // had them on the `.trplayer` sidecar, whose line stays written
            // below for builds that read it (the back-compat fallback).
            let bestiary: Vec<save::BestiaryKill> = self
                .bestiary
                .kills
                .pairs()
                .map(|(id, kills)| save::BestiaryKill { credit_id: id.to_string(), kills })
                .collect();
            // `SaveNPCs`'s block B (`WorldFile.cs:1703-1750`): the live town
            // NPCs — the kind, the given name, the pixel position, the
            // homelessness and the home tile. The shimmered-town block A and
            // the `SavesAndLoads` passives (block C) ride the C# arrays the
            // port does not model.
            let npcs: Vec<save::NpcRecord> = self
                .npcs
                .iter()
                .filter(|n| n.kind.is_town_resident())
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
            let side = save::WorldSide { chests, banners, bestiary, npcs, ore_tiers: Some(ore_tiers) };
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
                    .bestiary
                    .kills
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
            let sections = self.wld_sections();
            match tr_core::wld::save(&self.world_path, &self.world, &sections) {
                Ok(()) => log::info!("saved {}", self.world_path.display()),
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
    use tr_core::tile::{Tile, FLAG_ACTIVE};
    use tr_core::World;

    /// A per-test scratch dir under the system temp dir (tr-app has no
    /// tempfile dev-dependency — the `wld/mod.rs` tests' pattern).
    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("tr-saveglue-p7-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir
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
        }];
        // NPCs: a housed Merchant and a homeless Nurse (the re-validation's
        // two arms), both named, one carrying the shimmer payload.
        let mut merchant =
            crate::npc::Npc::new(crate::npc::NpcKind::Merchant, 65.0 * 16.0, 130.0 * 16.0, 1, 0.0);
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
        app.world.pressure_plates =
            vec![tr_core::wld::sections::WldPressurePlate { x: 33, y: 21 }];
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
            vec![
                tr_core::wld::sections::WldSign {
                    text: "hello".into(),
                    x: 30,
                    y: 20
                }
            ]
        );
        // The world-state halves are the app's load job (`load_world`), so
        // call them the way the startup does and compare the store.
        let mut world = loaded.world;
        tr_core::wld::sections::signs::apply_loaded(&mut world, &sections.signs);
        assert_eq!(world.signs, sections.signs, "applied on load");
        tr_core::wld::sections::pressure_plates::apply_loaded(&mut world, &sections.pressure_plates);
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
        assert!(sections.npcs.town[1].homeless, "the homeless record says so");
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
        let table =
            tr_core::wld::SectionTable::open(&first).expect("opens");
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
        assert!(loaded.sections.npcs.shimmered.is_empty(), "no shimmered NPCs");
        assert!(loaded.sections.tile_entities.is_empty(), "no tile entities");
        assert!(loaded.sections.pressure_plates.is_empty(), "no plates");
        assert!(loaded.sections.town_rooms.is_empty(), "no town rooms");
        assert!(loaded.sections.bestiary.is_empty(), "no bestiary state");
        // The creative section is never empty in the port: the registry holds
        // the three ported persistent powers at their defaults, and the C#
        // writes every `IPersistentPerWorldContent` power regardless
        // (`SaveToWorld:127-136`) — the three unported ids (9/10/12) are the
        // only records a vanilla file has that the port does not.
        let ids: Vec<u16> = loaded
            .sections
            .creative_powers
            .records
            .iter()
            .map(|(id, _)| *id)
            .collect();
        assert_eq!(ids, [0, 8, 13], "the ported three, ascending");
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
        assert!(loaded.sections.chests.is_empty(), "no chest section present");
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
        assert!(!dir.join("native.wld").exists(), "no `.wld` beside a `.trwld`");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
mod chest_save_tests {
    use super::*;
    use crate::app::WorldSections;
    use crate::assets;
    use crate::npc::{Npc, NpcKind};
    use crate::player::Player;
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

        let mut app = app_for(world, WorldSections::native(save::WorldSide::default()), path.clone());
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
        // `ChestUI.RenameChestSubmit`'s write (`:853-860`) into the entity's
        // name (M6-SAVE-1 phase 3's fold).
        app.world_chests[renamed].name = "Gold Stash".into();
        // Keep the scratch save clean: no player sidecar and no map sidecar.
        app.player = None;
        app.settings.map_enabled = false;
        app.save_world();

        let (world, sections) = crate::app::load_world(&path).expect("the saved world reloads");
        let _ = std::fs::remove_file(&path);
        assert_eq!(sections.native.chests.len(), 2, "one record per live store entry");
        let saved_b = sections.native.chests
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

        let back = app_for(world, sections, path);
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
        assert_eq!(
            back.world_chests[renamed].name, "Gold Stash",
            "the name rides the entity (the old side map is gone)"
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
            WorldSections::native(save::WorldSide {
                chests: records,
                banners: Vec::new(),
                bestiary: Vec::new(),
                npcs: Vec::new(),
                ore_tiers: None,
            }),
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
            !app.world_chests.iter().any(|c| c.name == "Ghost"),
            "the culled record's name lands nowhere"
        );
        assert_eq!(
            app.world_chests.iter().filter(|c| !c.name.is_empty()).count(),
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
        let path = std::env::temp_dir().join("tr-dw-2-3-banner-roundtrip.trwld");
        let _ = std::fs::remove_file(&path);

        let mut app = app_for(world, WorldSections::native(save::WorldSide::default()), path.clone());
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
        assert_eq!(sections.native.banners.len(), 2, "one record per tallied banner");
        let back = app_for(world, sections, path);
        let pairs: Vec<(i32, i32)> = back.banners.pairs().collect();
        assert_eq!(pairs, vec![(69, 7), (151, 3)], "the counts ride the store");

        // The control: a fresh App over the same world starts empty — the
        // records above are what carried the tally, not a re-derived one.
        let mut world2 = chest_rig();
        let control = app_for(world2.clone(), WorldSections::native(save::WorldSide::default()), PathBuf::new());
        assert_eq!(control.banners.pairs().count(), 0, "no records — no tally");
    }

    /// DW-2.4: the bestiary tally's save → load round-trip through the App
    /// path — the `NPCKillsTracker.Save` counts the kill pass accumulates ride
    /// the world file now (the vanilla's section-10 home; the port had them on
    /// the `.trplayer` sidecar), and the loaded side seeds the tracker.
    #[test]
    fn dw_2_4_a_saved_bestiary_tally_round_trips_through_the_app() {
        let world = chest_rig();
        let path = std::env::temp_dir().join("tr-dw-2-4-bestiary-roundtrip.trwld");
        let _ = std::fs::remove_file(&path);

        let mut app = app_for(world, WorldSections::native(save::WorldSide::default()), path.clone());
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
        assert_eq!(sections.native.bestiary.len(), 2, "one record per tallied credit id");
        let back = app_for(world, sections, path);
        assert_eq!(back.bestiary.kills.get_kill_count("BlueSlime"), 9);
        assert_eq!(back.bestiary.kills.get_kill_count("KingSlime"), 1);

        // The control: a fresh App starts empty — the records carried the tally.
        let control = app_for(chest_rig(), WorldSections::native(save::WorldSide::default()), PathBuf::new());
        assert_eq!(control.bestiary.kills.get_kill_count("BlueSlime"), 0);
    }

    /// DW-2.4 (the back-compat fallback): a world saved before v9 (the
    /// version field patched) carries no bestiary section, and the player
    /// sidecar's kill line still seeds the tracker — the deviation's original
    /// home stays the fallback for migrated builds.
    #[test]
    fn dw_2_4_a_v8_world_falls_back_to_the_sidecar_kills() {
        let path = std::env::temp_dir().join("tr-dw-2-4-sidecar-fallback.trwld");
        let _ = std::fs::remove_file(&path);
        // A v8 payload: the current writer's stream with the version patched.
        let mut app = app_for(chest_rig(), WorldSections::native(save::WorldSide::default()), path.clone());
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
        persist::save(&path, &player, inv, 0, [0, 0, 0], &[0; 10], &kills)
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
    }

    /// DW-2.5: the town roster's save → load round-trip through the App path —
    /// `SaveNPCs`'s block-B records (`WorldFile.cs:1703-1750`) restore the
    /// town NPC with its given name, its home and its position, and a fresh
    /// world (no records) keeps the vanilla-gen Guide arrival.
    #[test]
    fn dw_2_5_a_town_npc_round_trips_through_the_app() {
        let path = std::env::temp_dir().join("tr-dw-2-5-npc-roundtrip.trwld");
        let _ = std::fs::remove_file(&path);

        let mut app = app_for(chest_rig(), WorldSections::native(save::WorldSide::default()), path.clone());
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
    }

    /// DW-2.5 (the back-compat arm): a world without NPC records (a fresh
    /// world, or a pre-v10 save) still spawns the vanilla-gen homeless Guide
    /// at the spawn point.
    #[test]
    fn dw_2_5_a_recordless_world_spawns_the_guide() {
        let mut world = chest_rig();
        world.spawn_x = 50;
        let path = PathBuf::new();
        let app = app_for(world, WorldSections::native(save::WorldSide::default()), path);
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
        let path = std::env::temp_dir().join("tr-dw-2-6-ore-tiers.trwld");
        let _ = std::fs::remove_file(&path);

        let mut app = app_for(chest_rig(), WorldSections::native(save::WorldSide::default()), path.clone());
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
        let tiers = sections.native.ore_tiers.expect("the v11 section rides the tiers");
        assert_eq!(tiers, [166, 6, 9, 8, 107, 108, 111]);
        assert_eq!(world.ore_tiers.cobalt, 107);

        let back = app_for(world, sections, path);
        assert_eq!(back.ore_tiers.copper, 166, "the alternate flip survives");
        assert_eq!(back.ore_tiers.cobalt, 107, "the smashed hardmode tier survives");
        assert_eq!(back.ore_tiers.mythril, 108);
        assert_eq!(back.ore_tiers.adamantite, 111);
    }

    /// DW-2.6 (the back-compat arm): a world whose save carries no ore section
    /// (pre-v11) starts the runtime copy at the static defaults — the C#'s own
    /// reset arm (`WorldFile.cs:796-814`).
    #[test]
    fn dw_2_6_a_v10_world_starts_at_the_static_defaults() {
        let mut world = chest_rig();
        world.ore_tiers.copper = 166; // a generation-time roll the old save dropped
        let path = std::env::temp_dir().join("tr-dw-2-6-defaults.trwld");
        let _ = std::fs::remove_file(&path);
        let mut app = app_for(world, WorldSections::native(save::WorldSide::default()), path.clone());
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
        assert_eq!(back.ore_tiers.copper, 7, "the static default, not the dropped roll");
    }
}

