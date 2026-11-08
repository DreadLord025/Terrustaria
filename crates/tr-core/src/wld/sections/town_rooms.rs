//! Section 9 (pointer 8): TownManager rooms — `WorldFile.SaveTownManager:3385` /
//! `WorldFile.LoadTownManager:3391` (`TownRoomManager.cs:93-113`).
//!
//! Phase 1 stub: the empty-section encoding only.

use super::{SectionData, SectionPayload};
use crate::binio::{Reader, Writer};

/// One room: the npc id that owns it plus its room's top-left tile
/// (`TownRoomManager.cs:98-102`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WldTownRoom {
    pub npc_id: i32,
    pub x: i32,
    pub y: i32,
}

pub fn implemented() -> bool {
    false
}

pub fn read(r: &mut Reader, _version: i32) -> Option<Vec<WldTownRoom>> {
    if r.i32()? != 0 {
        return None; // this stub decodes empty sections only
    }
    Some(Vec::new())
}

pub fn write(w: &mut Writer, data: &SectionData, _version: i32) {
    w.i32(data.town_rooms.len() as i32);
}

pub fn empty(data: &SectionData) -> bool {
    data.town_rooms.is_empty()
}

pub fn payload(value: Vec<WldTownRoom>) -> SectionPayload {
    SectionPayload::TownRooms(value)
}
