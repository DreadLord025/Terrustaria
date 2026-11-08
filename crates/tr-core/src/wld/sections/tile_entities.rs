//! Section 7 (pointer 6): tile entities — `WorldFile.SaveTileEntities:3302` /
//! `WorldFile.LoadTileEntities:3315`.
//!
//! Phase 1 stub: the empty-section encoding only. Stored ids are discarded on
//! load and reassigned sequentially (`LoadTileEntities:3323`) — the entity
//! record head is `(byte type, i32 ID, i16 X, i16 Y)` followed by the kind's
//! extra data, whose shapes the tile-entity phase fills in.

use super::{SectionData, SectionPayload};
use crate::binio::{Reader, Writer};

/// The item-holder item triple: `(i16 type, u8 prefix, i16 stack)`, written
/// unconditionally for item frames / racks / platters
/// (`TEItemFrame.cs:47-56`; the port already round-trips it in
/// `tile_entities.rs:48-64`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WldTeItem {
    pub kind: i16,
    pub prefix: u8,
    pub stack: i16,
}

/// The extra payload of a ported entity kind. Unported shapes (display doll,
/// sensor, dummy) are parse-and-skipped by the tile-entity phase; the ported
/// shapes below are `TileEntity.WriteExtraData`'s.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum WldTeExtra {
    #[default]
    None,
    /// Item holders: item frames, weapons racks, food platters.
    Item { item: WldTeItem },
    /// Hat rack: a presence bitsByte, then up to four triples — two items and
    /// two dyes (`TEHatRack.cs:59-80`).
    HatRack {
        items: [Option<WldTeItem>; 2],
        dyes: [Option<WldTeItem>; 2],
    },
    /// Training dummy: the tied npc id (`TETrainingDummy.cs`).
    Dummy { npc: i32 },
    /// Logic sensor: the check byte and the on flag (`TELogicSensor.cs`).
    Logic { check: u8, on: bool },
}

/// One entity record head (`TileEntity.Write`, `SaveTileEntities:3309`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WldTileEntity {
    /// `TileEntitiesManager.cs:26` registration order.
    pub ty: u8,
    /// Position; stored x/y are i16 (`Point16`).
    pub x: i16,
    pub y: i16,
    pub extra: WldTeExtra,
}

pub fn implemented() -> bool {
    false
}

pub fn read(r: &mut Reader, _version: i32) -> Option<Vec<WldTileEntity>> {
    if r.i32()? != 0 {
        return None; // this stub decodes empty sections only
    }
    Some(Vec::new())
}

pub fn write(w: &mut Writer, data: &SectionData, _version: i32) {
    w.i32(data.tile_entities.len() as i32);
}

pub fn empty(data: &SectionData) -> bool {
    data.tile_entities.is_empty()
}

pub fn payload(value: Vec<WldTileEntity>) -> SectionPayload {
    SectionPayload::TileEntities(value)
}
