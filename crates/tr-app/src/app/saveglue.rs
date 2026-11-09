//! The App's save glue — the world/player autosave, the explored-map write
//! and its load-side twin. (A6: extracted from `main.rs`.)

use crate::app::{App, is_native_trwld};
use crate::map;
use crate::persist;
use tr_core::save;
use tr_core::wld::sections::SectionData;

impl App {
    /// `WorldGen.SaveWorld` (`WorldFile.cs:825`'s entry) over the loaded file:
    /// the port writes its native `.trwld` (`save::save`), a retail `.wld`
    /// writes every section the port carries (`wld::save`), and nothing else
    /// is written for either (a `.wld`'s player state and explored map have no
    /// port format yet — the `.trplayer`/`.map` sidecars stay the `.trwld`
    /// path's; recorded in `docs/ANALYSIS.md`).
    ///
    /// Every exit route funnels through here — the autosave tick
    /// (`frame.rs`, `Main.cs:19751-19765`'s 600 s), `F9`, the window's close
    /// request and the menu's Quit — so the dispatch below is the one place a
    /// save path is decided.
    pub(crate) fn save_world(&mut self) {
        if is_native_trwld(&self.world_path) {
            match save::save(&self.world_path, &self.world) {
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
        merchant.given_name = Some("Hal".into());
        merchant.home = Some((65, 105));
        merchant.variation_index = 3;
        let mut nurse =
            crate::npc::Npc::new(crate::npc::NpcKind::Nurse, 70.0 * 16.0, 90.0 * 16.0, 1, 0.0);
        nurse.given_name = Some("Nurse".into());
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
        assert_eq!(npcs[0].given_name.as_deref(), Some("Hal"));
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
        let world = tr_core::save::load(&path).expect("reads back");
        assert_eq!(world.name, "t", "the `ground()` fixture world");
        assert!(!dir.join("native.wld").exists(), "no `.wld` beside a `.trwld`");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
