//! The town-NPC housing — `WorldGen.QuickFindHome`'s three-pass scan
//! (`WorldGen.cs:5759-5805`) and the home assignment (`MoveTownNPC`'s tail,
//! `:4848-4854`), consumed by the homeless town NPCs.
//!
//! Also the two world-file states that ride this module (M6-SAVE-1's section
//! 6/9): `TownRooms` = `WorldGen.TownManager` (`TownRoomManager.cs`) with the
//! load-path `FixAgainstExploits` home re-validation, and `ShimmeredTownNpcs`
//! = `NPC.ShimmeredTownNPCs` (`NPC.cs:6071`).

use tr_core::housing::{check_room, room_needs, start_room_check};
use tr_core::wld::sections::npcs::NPCID_COUNT;
use tr_core::world::World;
use tr_core::wld::sections::WldTownRoom;

use crate::npc::Npc;

/// `QuickFindHome` (`WorldGen.cs:5759-5805`): the three-pass room scan
/// around `(tile_x, tile_y)` — the exact tile, the ±1 ring, the ±10 ring
/// with stride 2 — the first pass whose room both scans and furnishes
/// wins. Returns the `(best_x, best_y)` spot.
pub fn quick_find_home(world: &World, tile_x: i32, tile_y: i32) -> Option<(i32, i32)> {
    if tile_x <= 10 || tile_y <= 10 || tile_x >= world.width - 10 || tile_y >= world.height - 10 {
        return None;
    }
    // The pass 1: the exact tile (the scan reads `homeTileY - 1`).
    if let Some((bx, by)) = scan_spot(world, tile_x, tile_y - 1) {
        return Some((bx, by));
    }
    // The pass 2: the ±1 ring.
    for i in (tile_x - 1)..=(tile_x + 1) {
        for j in (tile_y - 1)..=(tile_y + 1) {
            if let Some((bx, by)) = scan_spot(world, i, j) {
                return Some((bx, by));
            }
        }
    }
    // The pass 3: the ±10 ring with stride 2.
    let num = 10;
    for k in (tile_x - num..=(tile_x + num)).step_by(2) {
        for l in (tile_y - num..=(tile_y + num)).step_by(2) {
            if let Some((bx, by)) = scan_spot(world, k, l) {
                return Some((bx, by));
            }
        }
    }
    None
}

/// One scan spot: the room check + the furniture needs.
fn scan_spot(world: &World, x: i32, y: i32) -> Option<(i32, i32)> {
    let scan = check_room(world, x, y).ok()?;
    if !room_needs(&scan.house_tiles) {
        return None;
    }
    // `WorldGen.bestX/bestY` = the scan's centre (the room's found tile).
    Some((x, y))
}

/// `WorldGen.TownManager` (`TownRoomManager.cs:11-13`): one room per town NPC
/// — the `_roomLocationPairs` list and the `_hasRoom` fast table the load path
/// rebuilds from the file (`TownRoomManager.cs:94`/`:108`). Both are keyed by
/// the NPC's *type* id (the wire's `npcID`).
#[derive(Clone, Debug, PartialEq)]
pub struct TownRooms {
    /// The `(npcID, X, Y)` pairs in registration order (`Save:99-103`).
    rooms: Vec<(i32, i32, i32)>,
    /// `_hasRoom[npcID]` (`TownRoomManager.cs:12`): the `HasRoomQuick` table.
    has_room: Vec<bool>,
}

impl TownRooms {
    pub fn new() -> Self {
        Self {
            rooms: Vec::new(),
            has_room: vec![false; NPCID_COUNT],
        }
    }

    /// `TownRoomManager.Load` (`:108-114`): the pairs come in file order and
    /// the fast table is rebuilt from them, so `HasRoomQuick` agrees with the
    /// list on a fresh load.
    pub fn from_wld_rooms(rooms: &[WldTownRoom]) -> Self {
        let mut state = Self::new();
        for room in rooms {
            if !(0..NPCID_COUNT as i32).contains(&room.npc_id) {
                continue; // `_hasRoom[npcID]` would throw on a foreign file
            }
            state.rooms.push((room.npc_id, room.x, room.y));
            state.has_room[room.npc_id as usize] = true;
        }
        state
    }

    /// `TownRoomManager.Save` (`:94-104`): the pairs in registration order.
    pub fn to_wld_rooms(&self) -> Vec<WldTownRoom> {
        self.rooms
            .iter()
            .map(|&(npc_id, x, y)| WldTownRoom { npc_id, x, y })
            .collect()
    }

    /// `HasRoom(npcID, out roomPosition)` (`TownRoomManager.cs:44-55`): the
    /// first pair for that NPC, or `None` when the fast table is clear.
    pub fn has_room(&self, npc_id: i32) -> Option<(i32, i32)> {
        if !self.has_room_quick(npc_id) {
            return None;
        }
        self.rooms
            .iter()
            .find(|(id, _, _)| *id == npc_id)
            .map(|&(_, x, y)| (x, y))
    }

    /// `HasRoomQuick` (`TownRoomManager.cs:40-43`).
    pub fn has_room_quick(&self, npc_id: i32) -> bool {
        (0..NPCID_COUNT as i32).contains(&npc_id) && self.has_room[npc_id as usize]
    }

    /// `SetRoom(npcID, X, Y)` (`TownRoomManager.cs:57-64`): the NPC's earlier
    /// room goes, the new one appends.
    pub fn set_room(&mut self, npc_id: i32, x: i32, y: i32) {
        if !(0..NPCID_COUNT as i32).contains(&npc_id) {
            return;
        }
        self.rooms.retain(|&(id, _, _)| id != npc_id);
        self.rooms.push((npc_id, x, y));
        self.has_room[npc_id as usize] = true;
    }

    /// `KickOut(npcType)` (`TownRoomManager.cs:80-85`): the pair is dropped and
    /// the fast table cleared — the port's `KickOut(NPC)` does both.
    pub fn kick_out(&mut self, npc_id: i32) {
        self.rooms.retain(|&(id, _, _)| id != npc_id);
        if (0..NPCID_COUNT as i32).contains(&npc_id) {
            self.has_room[npc_id as usize] = false;
        }
    }

    /// `Clear` (`TownRoomManager.cs:106-112`).
    pub fn clear(&mut self) {
        self.rooms.clear();
        self.has_room.iter_mut().for_each(|b| *b = false);
    }

    /// The room list, for inspection.
    pub fn rooms(&self) -> &[(i32, i32, i32)] {
        &self.rooms
    }
}

impl Default for TownRooms {
    fn default() -> Self {
        Self::new()
    }
}

/// `NPC.ShimmeredTownNPCs` (`NPC.cs:6071`, `NPC.cs:95163`): the
/// `bool[NPCID.Count]` bitmap the section's first list persists as truth
/// indices (`SaveNPCs:1705-1713`). `AI_007_TownEntities` writes it per tick
/// (`NPC.cs:52967`); the file only says which are set.
#[derive(Clone, Debug, PartialEq)]
pub struct ShimmeredTownNpcs(Vec<bool>);

impl ShimmeredTownNpcs {
    pub fn new() -> Self {
        Self(vec![false; NPCID_COUNT])
    }

    /// `LoadNPCs:2888-2892`: the saved indices become the bitmap.
    pub fn from_wld_indices(indices: &[i32]) -> Self {
        let mut state = Self::new();
        for &index in indices {
            if (0..NPCID_COUNT as i32).contains(&index) {
                state.0[index as usize] = true;
            }
        }
        state
    }

    /// `SaveNPCs:1706-1713`: the truth indices, ascending (`array[i]` walks
    /// the array from 0).
    pub fn to_wld_indices(&self) -> Vec<i32> {
        self.0
            .iter()
            .enumerate()
            .filter(|(_, b)| **b)
            .map(|(i, _)| i as i32)
            .collect()
    }

    /// `ShimmeredTownNPCs[type]`.
    pub fn is_shimmered(&self, npc_id: i32) -> bool {
        (0..NPCID_COUNT as i32).contains(&npc_id) && self.0[npc_id as usize]
    }

    /// `NPC.ShimmeredTownNPCs[this.type] = this.IsShimmerVariant`
    /// (`NPC.cs:52967`); out-of-range ids are ignored (the C# would throw).
    pub fn set_shimmered(&mut self, npc_id: i32, value: bool) {
        if (0..NPCID_COUNT as i32).contains(&npc_id) {
            self.0[npc_id as usize] = value;
        }
    }
}

impl Default for ShimmeredTownNpcs {
    fn default() -> Self {
        Self::new()
    }
}

/// `WorldFile.FixAgainstExploits:1864-1896` — the town-NPC arm, run after both
/// the NPC section and the town rooms are loaded (the C# runs it from
/// `LoadWorld_LastMinuteFixes`, at the very end of the load). Reusable: the
/// same re-validation is what a mid-game `moveRoom` needs.
///
/// For every live town NPC with a home: `StartRoomCheck(homeTileX,
/// homeTileY - 1)`; on failure the stored room is offered (teleport in, retry),
/// and if that still fails the NPC is kicked out and goes homeless.
pub fn fix_town_npc_homes(world: &World, npcs: &mut [Npc], rooms: &mut TownRooms) {
    for npc in npcs.iter_mut() {
        // `!nPC.active || !nPC.townNPC || nPC.type == 368 || nPC.type == 37 ||
        // nPC.homeless` (`:1881`). The port's live list holds only active NPCs,
        // `is_town_npc` is the modelled `townNPC` set, and the Travelling
        // Merchant (368) and Old Man (37) have no ported kind — every guard
        // is satisfied or vacuous, none dropped.
        if !npc.kind.is_town_npc() || npc.home.is_none() {
            continue; // homeless NPCs are never re-validated
        }
        let (home_x, home_y) = npc.home.unwrap();
        let mut fits = start_room_check(world, home_x, home_y - 1);
        if !fits {
            // `HasRoom(type, out var roomPosition)` (`:1885-1887`).
            if let Some((room_x, room_y)) = rooms.has_room(npc.kind.net_id()) {
                npc.home = Some((room_x, room_y));
                fits = start_room_check(world, room_x, room_y - 1);
            }
            if !fits {
                // `KickOut(nPC)` + `homeless = true` (`:1893-1894`).
                rooms.kick_out(npc.kind.net_id());
                npc.home = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_world_helpers::{build_furnished_room, build_furnished_room_with, ground};

    #[test]
    fn the_homeless_guide_finds_a_furnished_room() {
        // A furnished room in the world: the three-pass scan finds it from
        // the Guide's position up to 10 tiles away.
        let mut w = ground();
        build_furnished_room(&mut w, 60, 100, 12, 9);
        let home = quick_find_home(&w, 72, 108);
        assert!(home.is_some(), "the scan finds the room");
        let (hx, hy) = home.unwrap();
        assert!((60..=71).contains(&hx), "hx {hx}");
        assert!((99..=108).contains(&hy), "hy {hy}");
    }

    #[test]
    fn an_unfurnished_room_rejects() {
        // A room without furniture: the scan fails everywhere.
        let mut w = ground();
        build_furnished_room_with(&mut w, 60, 100, 12, 9, &[]);
        assert!(quick_find_home(&w, 72, 108).is_none());
    }

    // --- the room state (`TownRoomManager`) -------------------------------

    /// A town NPC at a pixel position with a home tile.
    fn town_npc(kind: crate::npc::NpcKind, x: f32, y: f32, home: Option<(i32, i32)>) -> Npc {
        let mut npc = Npc::new(kind, x, y, 1, 0.0);
        npc.home = home;
        npc.given_name = "Test".into();
        npc
    }

    #[test]
    fn test_DW_4_1_the_rooms_round_trip_through_the_wld_seam() {
        // The section-9 wire order (`TownRoomManager.Save:99-103`) survives
        // the load's clear-then-refill (`:108-114`).
        let rooms = vec![
            WldTownRoom {
                npc_id: 17,
                x: 65,
                y: 105,
            },
            WldTownRoom {
                npc_id: 228,
                x: 80,
                y: 120,
            },
        ];
        let state = TownRooms::from_wld_rooms(&rooms);
        assert_eq!(state.rooms(), &[(17, 65, 105), (228, 80, 120)]);
        assert_eq!(state.to_wld_rooms(), rooms);
        // `HasRoom`/`HasRoomQuick` agree with the list.
        assert_eq!(state.has_room(17), Some((65, 105)));
        assert!(state.has_room_quick(17));
        assert!(state.has_room_quick(228));
        assert_eq!(state.has_room(18), None, "no room for a third NPC");
        assert!(!state.has_room_quick(18));
    }

    #[test]
    fn set_room_replaces_the_npcs_previous_room() {
        // `SetRoom` removes the NPC's earlier pair before appending
        // (`TownRoomManager.cs:60-64`): one room per NPC, always.
        let mut state = TownRooms::new();
        state.set_room(17, 65, 105);
        state.set_room(17, 90, 140);
        assert_eq!(state.rooms(), &[(17, 90, 140)]);
        assert_eq!(state.has_room(17), Some((90, 140)));
    }

    #[test]
    fn kick_out_clears_the_pair_and_the_fast_table() {
        let mut state = TownRooms::new();
        state.set_room(17, 65, 105);
        state.set_room(228, 80, 120);
        state.kick_out(17);
        assert_eq!(state.rooms(), &[(228, 80, 120)]);
        assert!(!state.has_room_quick(17));
        assert!(state.has_room_quick(228));
        state.clear();
        assert!(state.rooms().is_empty());
        assert!(!state.has_room_quick(228));
    }

    #[test]
    fn an_out_of_range_room_npc_id_is_dropped() {
        // `_hasRoom[npcID]` would throw on a foreign file; the port drops the
        // room instead (the C# never sees one).
        let state = TownRooms::from_wld_rooms(&[WldTownRoom {
            npc_id: 99_999,
            x: 1,
            y: 2,
        }]);
        assert!(state.rooms().is_empty());
        assert!(!state.has_room_quick(99_999));
        assert!(!state.has_room_quick(-1));
    }

    #[test]
    fn test_DW_4_1_the_shimmered_bitmap_round_trips() {
        // `LoadNPCs:2888-2892` ↔ `SaveNPCs:1706-1713`.
        let state = ShimmeredTownNpcs::from_wld_indices(&[4, 68, 300]);
        assert_eq!(state.to_wld_indices(), vec![4, 68, 300]);
        assert!(state.is_shimmered(4) && state.is_shimmered(300));
        assert!(!state.is_shimmered(5));
        let mut live = state;
        live.set_shimmered(228, true);
        live.set_shimmered(68, false);
        live.set_shimmered(99_999, true); // ignored, not a panic
        assert_eq!(live.to_wld_indices(), vec![4, 228, 300]);
    }

    // --- the `FixAgainstExploits` re-validation ---------------------------

    #[test]
    fn test_DW_4_3_an_invalid_home_with_no_room_kicks_the_npc_out() {
        // A home in solid stone, no stored room: `KickOut` + homeless
        // (`WorldFile.cs:1864-1896`).
        let mut w = ground();
        build_furnished_room(&mut w, 60, 100, 12, 9); // a real room, unregistered
        let mut npcs = vec![town_npc(crate::npc::NpcKind::Merchant, 65.0 * 16.0, 105.0 * 16.0, Some((30, 130)))];
        let mut rooms = TownRooms::new();
        fix_town_npc_homes(&w, &mut npcs, &mut rooms);
        assert_eq!(npcs[0].home, None, "kicked out and made homeless");
        assert!(!npcs[0].homeless_despawn, "the despawn flag is not this path");
        assert!(!rooms.has_room_quick(17));
    }

    #[test]
    fn test_DW_4_3_an_invalid_home_with_a_stored_room_teleports_in() {
        // The stored room fits, so the NPC moves into it (`:1885-1892`).
        let mut w = ground();
        build_furnished_room(&mut w, 60, 100, 12, 9);
        let mut npcs = vec![town_npc(crate::npc::NpcKind::Merchant, 30.0 * 16.0, 130.0 * 16.0, Some((30, 130)))];
        let mut rooms = TownRooms::new();
        rooms.set_room(17, 65, 105);
        fix_town_npc_homes(&w, &mut npcs, &mut rooms);
        assert_eq!(npcs[0].home, Some((65, 105)), "teleported into the room");
        assert!(rooms.has_room_quick(17), "the room survives a good teleport");
    }

    #[test]
    fn test_DW_4_3_an_unusable_stored_room_still_kicks_out() {
        // The stored room does not fit either: the retry fails and the NPC
        // is kicked out (the C# checks `flag` again after the teleport).
        let mut w = ground();
        build_furnished_room(&mut w, 60, 100, 12, 9);
        let mut npcs = vec![town_npc(crate::npc::NpcKind::Merchant, 30.0 * 16.0, 130.0 * 16.0, Some((30, 130)))];
        let mut rooms = TownRooms::new();
        rooms.set_room(17, 30, 130); // solid stone
        fix_town_npc_homes(&w, &mut npcs, &mut rooms);
        assert_eq!(npcs[0].home, None);
        assert!(!rooms.has_room_quick(17), "the bust room is released");
    }

    #[test]
    fn test_DW_4_3_a_valid_home_is_left_alone() {
        // A home inside a room passes `StartRoomCheck` and is not touched —
        // including an unfurnished (but enclosed) room, because the
        // re-validation does not run `RoomNeeds`.
        let mut furnished = ground();
        build_furnished_room(&mut furnished, 60, 100, 12, 9);
        let mut bare = ground();
        build_furnished_room_with(&mut bare, 60, 100, 12, 9, &[]);
        for w in [&furnished, &bare] {
            let mut npcs = vec![town_npc(crate::npc::NpcKind::Guide, 65.0 * 16.0, 105.0 * 16.0, Some((65, 105)))];
            let mut rooms = TownRooms::new();
            fix_town_npc_homes(w, &mut npcs, &mut rooms);
            assert_eq!(npcs[0].home, Some((65, 105)));
        }
    }

    #[test]
    fn test_DW_4_3_a_homeless_npc_is_never_revalidated() {
        // `nPC.homeless` short-circuits the loop (`:1881`): a homeless NPC
        // with no home at all keeps its position and no room is consumed.
        let w = ground();
        let mut npcs = vec![town_npc(crate::npc::NpcKind::Nurse, 30.0 * 16.0, 130.0 * 16.0, None)];
        let mut rooms = TownRooms::new();
        rooms.set_room(18, 30, 130);
        fix_town_npc_homes(&w, &mut npcs, &mut rooms);
        assert_eq!(npcs[0].home, None);
        assert!(rooms.has_room_quick(18), "the room is left for later");
    }

    #[test]
    fn test_DW_4_3_a_home_outside_the_world_bounds_kicks_out() {
        // A hostile file's home past the world edge: `StartRoomCheck` refuses
        // (TooCloseToWorldEdge), no stored room fits → kicked out.
        let w = ground();
        let mut npcs = vec![town_npc(crate::npc::NpcKind::Painter, 100.0 * 16.0, 100.0 * 16.0, Some((5000, 5000)))];
        let mut rooms = TownRooms::new();
        fix_town_npc_homes(&w, &mut npcs, &mut rooms);
        assert_eq!(npcs[0].home, None);
    }

    #[test]
    fn a_non_town_npc_is_never_revalidated() {
        // The filter is `townNPC`: a wandering zombie with a home tile (which
        // it can't have) is untouched.
        let w = ground();
        let mut npcs = vec![town_npc(crate::npc::NpcKind::Zombie, 30.0 * 16.0, 130.0 * 16.0, Some((30, 130)))];
        let mut rooms = TownRooms::new();
        fix_town_npc_homes(&w, &mut npcs, &mut rooms);
        assert_eq!(npcs[0].home, Some((30, 130)));
    }

    #[test]
    fn test_DW_4_3_the_load_path_heals_a_bogus_home() {
        // The whole load-side arm on a synthesized section: the Merchant's
        // file home is in solid stone, the town rooms carry a good room, so
        // the NPC ends up housed there (`Main.npc[num2++]` +
        // `FixAgainstExploits`).
        let section = tr_core::wld::sections::WldNpcs {
            town: vec![tr_core::wld::sections::WldTownNpc {
                net_id: 17,
                given_name: "Bob".into(),
                x: 30.0 * 16.0,
                y: 130.0 * 16.0,
                homeless: false,
                home_x: 30,
                home_y: 130,
                ..Default::default()
            }],
            ..Default::default()
        };
        let mut w = ground();
        build_furnished_room(&mut w, 60, 100, 12, 9);
        let mut npcs = crate::npc::from_wld(&section);
        assert_eq!(npcs.len(), 1, "the record becomes one live NPC");
        assert_eq!(npcs[0].home, Some((30, 130)), "the file's home, unhealed");
        let mut rooms = TownRooms::from_wld_rooms(&[WldTownRoom {
            npc_id: 17,
            x: 65,
            y: 105,
        }]);
        fix_town_npc_homes(&w, &mut npcs, &mut rooms);
        assert_eq!(npcs[0].home, Some((65, 105)));
    }

    /// The citations the golden rule demands for the `FixAgainstExploits` port.
    #[test]
    fn test_DW_4_5_the_citations_are_present() {
        let src = include_str!("town.rs");
        assert!(src.contains("WorldFile.FixAgainstExploits:1864"));
        assert!(src.contains("`TownRoomManager.cs:11-13`"));
        assert!(src.contains("`NPC.cs:6071`"));
        assert!(src.contains("// `SetRoom(npcID, X, Y)` (`TownRoomManager.cs:57-64`)"));
    }
}
