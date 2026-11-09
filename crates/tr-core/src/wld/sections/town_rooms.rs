//! Section 9 (pointer 8): TownManager rooms — `WorldFile.cs:3385`
//! (`SaveTownManager`, which is `TownRoomManager.Save`) /
//! `WorldFile.cs:3391` (`LoadTownManager`, `TownRoomManager.Load`).
//!
//! Wire shape: `i32 count` of the `_roomLocationPairs` list, then per room
//! `i32 npcID, i32 X, i32 Y` (`TownRoomManager.cs:98-102`). The load clears the
//! fast table and re-reads (`:108-114`), so the pairs arrive in file order and
//! `HasRoomQuick` agrees with the list on a fresh load.
//!
//! The state itself lives in the `tr-app` owner module: Phase 4 landed
//! `town::TownRooms::{from_wld_rooms,to_wld_rooms}` over this section's
//! [`WldTownRoom`], so this file is only the codec — which is why the phase
//! that owns the town NPCs could not own it and why it lands here.
//!
//! The file is untrusted input: `_hasRoom[npcID]` is a `bool[NPCID.Count]`, so
//! a room whose npc id is outside `0..NPCID_COUNT` is refused exactly as the
//! C#'s indexing would throw (`TownRoomManager.cs:110`), and no count
//! pre-allocates.

use super::{SectionData, SectionPayload};
use crate::binio::{Reader, Writer};
use crate::wld::sections::npcs::NPCID_COUNT;

/// One room: the npc id that owns it plus its room's top-left tile
/// (`TownRoomManager.cs:98-102`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WldTownRoom {
    pub npc_id: i32,
    pub x: i32,
    pub y: i32,
}

pub fn implemented() -> bool {
    true
}

/// `TownRoomManager.Load:108-114`: the count, then the `(npcID, X, Y)` triples.
/// `Clear()` is the caller's half — `TownRooms::from_wld_rooms` builds a fresh
/// table from the pairs, which is the same rewrite.
pub fn read(r: &mut Reader, _version: i32) -> Option<Vec<WldTownRoom>> {
    let count = r.i32()?;
    if count < 0 {
        return None; // a hostile count (a `List.Count` is never negative)
    }
    let mut rooms: Vec<WldTownRoom> = Vec::new();
    for _ in 0..count {
        let room = WldTownRoom {
            npc_id: r.i32()?,
            x: r.i32()?,
            y: r.i32()?,
        };
        // `this._hasRoom[num2] = true` (`:111`).
        if !(0..NPCID_COUNT as i32).contains(&room.npc_id) {
            return None;
        }
        rooms.push(room);
    }
    Some(rooms)
}

/// `SaveTownManager:3385-3390` → `TownRoomManager.Save:94-104`: the count, then
/// every pair's `Item1` (the npc id), `Item2.X` and `Item2.Y`.
pub fn write(w: &mut Writer, data: &SectionData, _version: i32) {
    // `writer.Write(this._roomLocationPairs.Count)` (`TownRoomManager.cs:98`).
    w.i32(data.town_rooms.len() as i32);
    for room in &data.town_rooms {
        // `writer.Write(roomLocationPair.Item1)` / `Item2.X` / `Item2.Y`
        // (`:99-102`).
        w.i32(room.npc_id);
        w.i32(room.x);
        w.i32(room.y);
    }
}

pub fn empty(data: &SectionData) -> bool {
    data.town_rooms.is_empty()
}

pub fn payload(value: Vec<WldTownRoom>) -> SectionPayload {
    SectionPayload::TownRooms(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Format version the current C# writes (`SaveFileFormatHeader:1174`).
    const VER: i32 = 326;

    fn room(npc_id: i32, x: i32, y: i32) -> WldTownRoom {
        WldTownRoom { npc_id, x, y }
    }

    fn data(rooms: Vec<WldTownRoom>) -> SectionData {
        SectionData {
            town_rooms: rooms,
            ..Default::default()
        }
    }

    fn section(rooms: &[WldTownRoom]) -> Vec<u8> {
        let mut w = Writer::new();
        write(&mut w, &data(rooms.to_vec()), VER);
        w.into_inner()
    }

    fn read_at(bytes: &[u8]) -> (Option<Vec<WldTownRoom>>, usize) {
        let mut r = Reader::new(bytes);
        (read(&mut r, VER), r.pos())
    }

    // --- the stub lane ----------------------------------------------------

    #[test]
    fn test_DW_5_5_the_section_is_implemented() {
        assert!(implemented(), "phase 5 replaces the stub in place");
        assert!(empty(&data(Vec::new())));
        assert!(!empty(&data(vec![room(17, 1, 1)])));
    }

    // --- DW-5.6: the round-trip -------------------------------------------

    #[test]
    fn test_DW_5_6_the_rooms_round_trip_byte_equal() {
        let rooms = vec![
            room(17, 40, 41), // the Merchant
            room(228, 100, 101),
            room(0, 1, 2),
        ];
        let bytes = section(&rooms);
        let (read_back, pos) = read_at(&bytes);
        assert_eq!(read_back.as_ref(), Some(&rooms), "the structs survive");
        assert_eq!(pos, bytes.len(), "the read consumes every byte");
        // 4 (count) + 12 per room.
        assert_eq!(bytes.len(), 4 + 3 * 12);
    }

    #[test]
    fn test_DW_5_6_the_hand_computed_rooms_match_the_c_sharp_layout() {
        // Hand-computed `TownRoomManager.Save:98-102`: count 2, then the two
        // `(npcID, X, Y)` triples, twelve bytes each and nothing else.
        let want = [
            &2i32.to_le_bytes()[..],
            &17i32.to_le_bytes()[..],
            &40i32.to_le_bytes()[..],
            &41i32.to_le_bytes()[..],
            &228i32.to_le_bytes()[..],
            &102i32.to_le_bytes()[..],
            &103i32.to_le_bytes()[..],
        ]
        .concat();
        assert_eq!(section(&[room(17, 40, 41), room(228, 102, 103)]), want);
        // The empty encoding the whole file relies on.
        assert_eq!(section(&[]), 0i32.to_le_bytes().to_vec());
        assert_eq!(read_at(&0i32.to_le_bytes()).0, Some(Vec::new()));
    }

    #[test]
    fn test_DW_5_6_the_codec_round_trips_what_the_town_rooms_seam_carries() {
        // The Phase-4 app seam this section feeds is `TownRooms::from_wld_rooms`
        // / `to_wld_rooms` (landed in `tr-app/src/town.rs:82`/`:95` over this
        // struct); both directions are lists of `WldTownRoom`, so the codec's
        // byte-equality over the struct is the seam's byte-equality.
        let rooms = vec![room(17, 40, 41), room(228, 102, 103)];
        let bytes = section(&rooms);
        let (back, pos) = read_at(&bytes);
        assert_eq!(back, Some(rooms));
        assert_eq!(pos, bytes.len());
    }

    // --- the defensive arms -----------------------------------------------

    #[test]
    fn a_room_npc_id_outside_the_table_is_refused() {
        // `_hasRoom` has `NPCID.Count` (697) entries; a room for a foreign npc
        // id is a hostile file (`TownRoomManager.cs:111` throws).
        let bytes = [
            &1i32.to_le_bytes()[..],
            &(NPCID_COUNT as i32).to_le_bytes()[..],
            &1i32.to_le_bytes()[..],
            &2i32.to_le_bytes()[..],
        ]
        .concat();
        assert!(read_at(&bytes).0.is_none());
        let bytes = [
            &1i32.to_le_bytes()[..],
            &(-1i32).to_le_bytes()[..],
            &0i32.to_le_bytes()[..],
            &0i32.to_le_bytes()[..],
        ]
        .concat();
        assert!(read_at(&bytes).0.is_none());
    }

    #[test]
    fn a_negative_count_is_refused() {
        assert!(read_at(&(-1i32).to_le_bytes()).0.is_none());
    }

    #[test]
    fn a_truncated_room_is_refused() {
        let bytes = [
            &1i32.to_le_bytes()[..],
            &17i32.to_le_bytes()[..],
            &40i32.to_le_bytes()[..],
        ]
        .concat();
        assert!(read_at(&bytes).0.is_none(), "y is missing");
    }

    #[test]
    fn a_hostile_count_does_not_allocate() {
        assert!(read_at(&i32::MAX.to_le_bytes()).0.is_none());
    }

    #[test]
    fn duplicate_rooms_for_one_npc_are_kept() {
        // `Load` does not dedup: `SetRoom` is what keeps one pair per npc, and
        // the file may carry more than one. The C#'s `_hasRoom[npcID] = true`
        // is idempotent, so the reader must not refuse.
        let rooms = vec![room(17, 40, 41), room(17, 50, 51)];
        let (back, pos) = read_at(&section(&rooms));
        assert_eq!(back, Some(rooms));
        assert_eq!(pos, 4 + 2 * 12);
    }

    /// The house rule's citation probe.
    #[test]
    fn test_DW_5_5_the_citations_are_present() {
        let src = include_str!("town_rooms.rs");
        assert!(src.contains("WorldFile.cs:3385"), "the writer's anchor");
        assert!(src.contains("WorldFile.cs:3391"), "the reader's anchor");
        for anchor in [
            "// `TownRoomManager.Load:108-114`",
            "// `SaveTownManager:3385-3390`",
            "// `TownRoomManager.cs:111`",
        ] {
            assert!(src.contains(anchor), "missing {anchor}");
        }
    }
}
