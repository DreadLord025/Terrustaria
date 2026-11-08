//! Section 10 (pointer 9): bestiary — `WorldFile.SaveBestiary:3396` /
//! `WorldFile.LoadBestiary:3402` (`BestiaryUnlocksTracker.cs:10-19`).
//!
//! Phase 1 stub: the empty-section encoding only. Three sub-lists, all keyed
//! by bestiary credit-id strings (the wire is the credit id, never a net id).

use super::{SectionData, SectionPayload};
use crate::binio::{Reader, Writer};

/// The three credit-id trackers in write order: kills (count + `(string, i32)`
/// pairs), sights (count + strings), chats (count + strings)
/// (`NPCKillsTracker.cs:59-66`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WldBestiary {
    pub kills: Vec<(String, i32)>,
    pub sights: Vec<String>,
    pub chats: Vec<String>,
}

impl WldBestiary {
    pub fn is_empty(&self) -> bool {
        self.kills.is_empty() && self.sights.is_empty() && self.chats.is_empty()
    }
}

pub fn implemented() -> bool {
    false
}

pub fn read(r: &mut Reader, _version: i32) -> Option<WldBestiary> {
    // Kills: `i32 count` + pairs, then the two string lists.
    if r.i32()? != 0 {
        return None; // this stub decodes empty sections only
    }
    if r.i32()? != 0 {
        return None;
    }
    if r.i32()? != 0 {
        return None;
    }
    Some(WldBestiary::default())
}

pub fn write(w: &mut Writer, data: &SectionData, _version: i32) {
    w.i32(data.bestiary.kills.len() as i32);
    w.i32(data.bestiary.sights.len() as i32);
    w.i32(data.bestiary.chats.len() as i32);
}

pub fn empty(data: &SectionData) -> bool {
    data.bestiary.is_empty()
}

pub fn payload(value: WldBestiary) -> SectionPayload {
    SectionPayload::Bestiary(value)
}
