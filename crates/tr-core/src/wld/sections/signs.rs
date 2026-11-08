//! Section 5 (pointer 4): signs — `WorldFile.SaveSigns:1678` /
//! `WorldFile.LoadSigns:2828`.
//!
//! Phase 1 stub: the empty-section encoding only. Note the C# field order —
//! the text comes before the position (`SaveSigns:1695-1697`).

use super::{SectionData, SectionPayload};
use crate::binio::{Reader, Writer};

/// One sign record: `(string text, i32 x, i32 y)` — text first
/// (`SaveSigns:1695-1697`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WldSign {
    pub text: String,
    pub x: i32,
    pub y: i32,
}

pub fn implemented() -> bool {
    false
}

pub fn read(r: &mut Reader, _version: i32) -> Option<Vec<WldSign>> {
    if r.i16()? != 0 {
        return None; // this stub decodes empty sections only
    }
    Some(Vec::new())
}

pub fn write(w: &mut Writer, data: &SectionData, _version: i32) {
    w.i16(data.signs.len() as i16);
}

pub fn empty(data: &SectionData) -> bool {
    data.signs.is_empty()
}

pub fn payload(value: Vec<WldSign>) -> SectionPayload {
    SectionPayload::Signs(value)
}
