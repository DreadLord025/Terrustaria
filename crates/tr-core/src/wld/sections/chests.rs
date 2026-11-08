//! Section 4 (pointer 3): chests — `WorldFile.SaveChests:1606` /
//! `WorldFile.LoadChests:2723`.
//!
//! Phase 1 stub: the empty-section encoding only. The records (x, y, name,
//! maxItems, per-slot stacks) and every load fix-up belong to the chest phase,
//! which replaces this file's bodies.

use super::{SectionData, SectionPayload};
use crate::binio::{Reader, Writer};

/// One slot's wire form (`SaveChests:1655-1672`): an air or banned slot is the
/// bare `i16 0`; a stored item writes `stack` then, only when it is positive,
/// `(type, prefix)`. The legacy negative-stack read path is a chest-phase
/// concern (`LoadChests:2760-2766`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WldChestSlot {
    pub stack: i16,
    pub kind: i32,
    pub prefix: u8,
}

/// One chest record: the `(x, y, name, maxItems)` head plus its slots
/// (`SaveChests:1650-1673`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WldChest {
    pub x: i32,
    pub y: i32,
    pub name: String,
    pub max_items: i32,
    pub slots: Vec<WldChestSlot>,
}

pub fn implemented() -> bool {
    false
}

pub fn read(r: &mut Reader, version: i32) -> Option<Vec<WldChest>> {
    if r.i16()? != 0 {
        return None; // this stub decodes empty sections only
    }
    // Pre-294 files carry a shared slot count before the loop
    // (`LoadChests:2726-2729`).
    if version < 294 {
        r.i16()?;
    }
    Some(Vec::new())
}

pub fn write(w: &mut Writer, data: &SectionData, _version: i32) {
    w.i16(data.chests.len() as i16);
}

pub fn empty(data: &SectionData) -> bool {
    data.chests.is_empty()
}

pub fn payload(value: Vec<WldChest>) -> SectionPayload {
    SectionPayload::Chests(value)
}
