//! Section 6 (pointer 5): NPCs — `WorldFile.SaveNPCs:1703` /
//! `WorldFile.LoadNPCs:2884`.
//!
//! Phase 1 stub: the empty-section encoding of all three lists only. The
//! version gates (268 shimmer, 213 variation, 315 homelessDespawn, 140 towers)
//! are framing: they are pinned here so the empty encodings are already
//! correct for legacy files.

use super::{SectionData, SectionPayload};
use crate::binio::{Reader, Writer};

/// One town-NPC record (`SaveNPCs:1714-1735`). The wire form starts with the
/// record-leading `active` bool, which doubles as the list terminator
/// (`LoadNPCs:2895`); the surrounding list is what terminates, so the flag
/// itself is not a field of the stored record.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WldTownNpc {
    /// `NPC.netID` (`ReadInt32` for `>= 190`; older files store a legacy name —
    /// the chest-era NPC phase maps that shape).
    pub net_id: i32,
    pub given_name: String,
    /// `NPC.position` in world pixels (`SaveNPCs:1722-1723`).
    pub x: f32,
    pub y: f32,
    pub homeless: bool,
    pub home_x: i32,
    pub home_y: i32,
    /// `BitsByte` bit 0 (`townNPC`) gates whether the variation index
    /// follows (`SaveNPCs:1727-1733`, read `>= 213`).
    pub variation_index: i32,
    /// `NPC.homelessDespawn` (read `>= 315`, `LoadNPCs:2917-2920`).
    pub homeless_despawn: bool,
}

/// One `SavesAndLoads` non-town NPC (a lunar tower, `SaveNPCs:1738-1747`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WldTower {
    pub net_id: i32,
    pub x: f32,
    pub y: f32,
}

/// The three lists in write order (`SaveNPCs:1703-1749`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WldNpcs {
    /// `NPC.ShimmeredTownNPCs` truth indices, `>= 268`
    /// (`SaveNPCs:1705-1713`).
    pub shimmered: Vec<i32>,
    pub town: Vec<WldTownNpc>,
    /// `>= 140` (`SaveNPCs:1738-1748`).
    pub towers: Vec<WldTower>,
}

impl WldNpcs {
    pub fn is_empty(&self) -> bool {
        self.shimmered.is_empty() && self.town.is_empty() && self.towers.is_empty()
    }
}

pub fn implemented() -> bool {
    false
}

pub fn read(r: &mut Reader, version: i32) -> Option<WldNpcs> {
    // 1. Shimmered town NPCs (`LoadNPCs:2886-2893`).
    if version >= 268 {
        let count = r.i32()?;
        if count != 0 {
            return None; // this stub decodes empty sections only
        }
    }
    // 2. Town NPCs: the record-leading bool is also the terminator.
    if r.boolean()? {
        return None;
    }
    // 3. Savable non-town NPCs (`LoadNPCs:2924-2942`).
    if version >= 140 && r.boolean()? {
        return None;
    }
    Some(WldNpcs::default())
}

pub fn write(w: &mut Writer, data: &SectionData, version: i32) {
    if version >= 268 {
        w.i32(data.npcs.shimmered.len() as i32);
    }
    w.boolean(false); // town list terminator
    if version >= 140 {
        w.boolean(false); // tower list terminator
    }
}

pub fn empty(data: &SectionData) -> bool {
    data.npcs.is_empty()
}

pub fn payload(value: WldNpcs) -> SectionPayload {
    SectionPayload::Npcs(value)
}
