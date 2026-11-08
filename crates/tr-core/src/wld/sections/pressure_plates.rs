//! Section 8 (pointer 7): weighted pressure plates —
//! `WorldFile.SaveWeightedPressurePlates:3359` /
//! `WorldFile.LoadWeightedPressurePlates:3373`.
//!
//! Phase 1 stub: the empty-section encoding only. Only the pressed positions
//! persist — the pressure arrays are rebuilt on load (`LoadWeightedPressurePlates:3377-3382`).

use super::{SectionData, SectionPayload};
use crate::binio::{Reader, Writer};

/// One pressed plate: just its tile position (`SaveWeightedPressurePlates:3366-3367`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WldPressurePlate {
    pub x: i32,
    pub y: i32,
}

pub fn implemented() -> bool {
    false
}

pub fn read(r: &mut Reader, _version: i32) -> Option<Vec<WldPressurePlate>> {
    if r.i32()? != 0 {
        return None; // this stub decodes empty sections only
    }
    Some(Vec::new())
}

pub fn write(w: &mut Writer, data: &SectionData, _version: i32) {
    w.i32(data.pressure_plates.len() as i32);
}

pub fn empty(data: &SectionData) -> bool {
    data.pressure_plates.is_empty()
}

pub fn payload(value: Vec<WldPressurePlate>) -> SectionPayload {
    SectionPayload::PressurePlates(value)
}
