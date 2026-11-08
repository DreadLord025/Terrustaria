//! Section 11 (pointer 10): creative powers —
//! `WorldFile.SaveCreativePowers:3412` / `WorldFile.LoadCreativePowers:3418`
//! (`CreativePowerManager.cs:125-151`).
//!
//! Phase 1 stub: the empty-section encoding only. The records are
//! terminator-delimited, not counted: `{ bool true; ushort id; payload }` until
//! `bool false`, and an unknown id aborts the load loop (`:144-149`).

use super::{SectionData, SectionPayload};
use crate::binio::{Reader, Writer};

/// One per-world persistent power value (`CreativePowerManager.cs:125-143`):
/// the id is the manager's registration-order u16, the payload a bool or f32.
#[derive(Clone, Debug, PartialEq)]
pub enum WldCreativeValue {
    Bool(bool),
    F32(f32),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WldCreativePowers {
    pub records: Vec<(u16, WldCreativeValue)>,
}

impl WldCreativePowers {
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

pub fn implemented() -> bool {
    false
}

pub fn read(r: &mut Reader, _version: i32) -> Option<WldCreativePowers> {
    if r.boolean()? {
        return None; // this stub decodes empty sections only
    }
    Some(WldCreativePowers::default())
}

pub fn write(w: &mut Writer, data: &SectionData, _version: i32) {
    w.boolean(false); // terminator
    let _ = data;
}

pub fn empty(data: &SectionData) -> bool {
    data.creative_powers.is_empty()
}

pub fn payload(value: WldCreativePowers) -> SectionPayload {
    SectionPayload::CreativePowers(value)
}
