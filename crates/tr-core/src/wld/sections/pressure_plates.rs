//! Section 8 (pointer 7): weighted pressure plates —
//! `WorldFile.cs:3359` (`SaveWeightedPressurePlates`) /
//! `WorldFile.cs:3373` (`LoadWeightedPressurePlates`).
//!
//! Wire shape: `i32 count`, then per pressed plate **only `i32 X, i32 Y`** —
//! the pressure arrays never persist (`WorldFile.cs:3377-3382` rebuilds a fresh
//! `bool[255]` per plate), so the section is a point list.
//!
//! The store is `World.pressure_plates`: the pressed points in file order,
//! which is the order the C# `Dictionary<Point, bool[]>` enumerates
//! (`SaveWeightedPressurePlates:3363-3368`). The load half below writes
//! `World` directly — this section has no app-level glue.
//!
//! **Save-side contract:** `write` reads [`SectionData`] (the frozen
//! `SAVE_DEFS` closure), so the caller materializes the bag with
//! [`from_world`] before invoking `wld::to_bytes`/`wld::save`.

use super::{SectionData, SectionPayload};
use crate::binio::{Reader, Writer};
use crate::world::World;

/// One pressed plate: just its tile position (`WorldFile.cs:3366-3367`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WldPressurePlate {
    pub x: i32,
    pub y: i32,
}

pub fn implemented() -> bool {
    true
}

/// `LoadWeightedPressurePlates:3373-3384`: `Reset()` (the caller's fresh
/// store replaces any earlier one — see [`apply_loaded`]) and
/// `NeedsFirstUpdate = true` are load-time state the port has no system for,
/// so this half is the count and the points.
///
/// A repeated point is refused: the C#'s `PressurePlatesPressed.Add` would
/// throw `ArgumentException` on the duplicate key (`:3381`), so a stream
/// carrying one is not a stream the vanilla reader finishes.
pub fn read(r: &mut Reader, _version: i32) -> Option<Vec<WldPressurePlate>> {
    let count = r.i32()?;
    if count < 0 {
        return None; // a hostile count (no C# collection is negative)
    }
    let mut plates: Vec<WldPressurePlate> = Vec::new();
    for _ in 0..count {
        let plate = WldPressurePlate {
            x: r.i32()?,
            y: r.i32()?,
        };
        if plates.contains(&plate) {
            return None;
        }
        plates.push(plate);
    }
    Some(plates)
}

/// `SaveWeightedPressurePlates:3359-3372` — the write half. The C# writer
/// iterates the dictionary under `EntityCreationLock` and writes every key's
/// `X` then `Y`.
pub fn write(w: &mut Writer, data: &SectionData, _version: i32) {
    // `writer.Write(PressurePlateHelper.PressurePlatesPressed.Count)` (`:3364`).
    w.i32(data.pressure_plates.len() as i32);
    for plate in &data.pressure_plates {
        // `writer.Write(item.Key.X)` / `.Y` (`:3366-3367`).
        w.i32(plate.x);
        w.i32(plate.y);
    }
}

/// `LoadWeightedPressurePlates:3374-3383`'s store half: `Reset()` followed by
/// one `Add(key, new bool[255])` per record. The pressure arrays are runtime
/// state (no ported system reads them yet — the mechanic itself is a
/// follow-up), so the store is the points alone and the replace is the whole
/// of `Reset()`.
pub fn apply_loaded(world: &mut World, records: &[WldPressurePlate]) {
    world.pressure_plates = records.to_vec();
}

/// `SaveWeightedPressurePlates:3359-3372`'s input: the live store, in the order
/// it was built (the dictionary's enumeration order).
pub fn from_world(world: &World) -> Vec<WldPressurePlate> {
    world.pressure_plates.clone()
}

pub fn empty(data: &SectionData) -> bool {
    data.pressure_plates.is_empty()
}

pub fn payload(value: Vec<WldPressurePlate>) -> SectionPayload {
    SectionPayload::PressurePlates(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Format version the current C# writes (`SaveFileFormatHeader:1174`).
    const VER: i32 = 326;

    fn plate(x: i32, y: i32) -> WldPressurePlate {
        WldPressurePlate { x, y }
    }

    fn data(plates: Vec<WldPressurePlate>) -> SectionData {
        SectionData {
            pressure_plates: plates,
            ..Default::default()
        }
    }

    fn section(plates: &[WldPressurePlate]) -> Vec<u8> {
        let mut w = Writer::new();
        write(&mut w, &data(plates.to_vec()), VER);
        w.into_inner()
    }

    fn read_at(bytes: &[u8]) -> (Option<Vec<WldPressurePlate>>, usize) {
        let mut r = Reader::new(bytes);
        (read(&mut r, VER), r.pos())
    }

    // --- the stub lane ----------------------------------------------------

    #[test]
    fn test_DW_5_5_the_section_is_implemented() {
        assert!(implemented(), "phase 5 replaces the stub in place");
        assert!(empty(&data(Vec::new())));
        assert!(!empty(&data(vec![plate(1, 1)])));
    }

    // --- DW-5.2: the round-trip -------------------------------------------

    #[test]
    fn test_DW_5_2_the_section_round_trips_byte_equal() {
        let plates = vec![plate(10, 20), plate(300, 2), plate(-5, -5)];
        let bytes = section(&plates);
        let (read_back, pos) = read_at(&bytes);
        assert_eq!(read_back.as_ref(), Some(&plates), "the struct survives");
        assert_eq!(pos, bytes.len(), "the read consumes every byte");
    }

    #[test]
    fn test_DW_5_2_the_hand_computed_points_match_the_c_sharp_layout() {
        // Hand-computed `SaveWeightedPressurePlates:3366-3367`: count 2 then
        // the two points, x before y, nothing else.
        let want = [
            &2i32.to_le_bytes()[..],
            &10i32.to_le_bytes()[..],
            &20i32.to_le_bytes()[..],
            &30i32.to_le_bytes()[..],
            &40i32.to_le_bytes()[..],
        ]
        .concat();
        assert_eq!(section(&[plate(10, 20), plate(30, 40)]), want);
        // Each point is 8 bytes: the pressure arrays never persist.
        assert_eq!(section(&[plate(0, 0)]).len(), 12);
    }

    #[test]
    fn test_DW_5_2_the_empty_section_is_the_count_only() {
        // `writer.Write(0)` — the empty encoding the whole file relies on.
        assert_eq!(section(&[]), 0i32.to_le_bytes().to_vec());
        assert_eq!(read_at(&0i32.to_le_bytes()).0, Some(Vec::new()));
    }

    #[test]
    fn test_DW_5_2_the_store_is_the_points_alone() {
        // The load half is a replace: whatever the world held is gone, and the
        // store carries no pressure state (the arrays are rebuilt on load, and
        // nothing here allocates 255 bools per plate).
        let mut world = World::new("t".into(), 16, 16);
        world.pressure_plates = vec![plate(1, 1)];
        apply_loaded(&mut world, &[plate(4, 5), plate(6, 7)]);
        assert_eq!(world.pressure_plates, vec![plate(4, 5), plate(6, 7)]);
        apply_loaded(&mut world, &[]);
        assert!(world.pressure_plates.is_empty());
        assert_eq!(from_world(&world), Vec::new());
    }

    // --- the defensive arms -----------------------------------------------

    #[test]
    fn a_duplicate_point_is_refused() {
        // `PressurePlatesPressed.Add` throws on an existing key (`:3381`), so
        // the reader refuses rather than silently dropping the second point.
        let bytes = [
            &2i32.to_le_bytes()[..],
            &1i32.to_le_bytes()[..],
            &2i32.to_le_bytes()[..],
            &1i32.to_le_bytes()[..],
            &2i32.to_le_bytes()[..],
        ]
        .concat();
        assert!(read_at(&bytes).0.is_none());
    }

    #[test]
    fn a_negative_count_is_refused() {
        assert!(read_at(&(-1i32).to_le_bytes()).0.is_none());
    }

    #[test]
    fn a_truncated_point_is_refused() {
        let bytes = [&1i32.to_le_bytes()[..], &1i32.to_le_bytes()[..]].concat();
        assert!(read_at(&bytes).0.is_none(), "y is missing");
    }

    #[test]
    fn a_hostile_count_does_not_allocate() {
        // 2^31-1 points at 8 bytes each: the first short read refuses, and the
        // empty `Vec::new` never reserves anything.
        assert!(read_at(&i32::MAX.to_le_bytes()).0.is_none());
    }

    /// The house rule's citation probe.
    #[test]
    fn test_DW_5_5_the_citations_are_present() {
        let src = include_str!("pressure_plates.rs");
        assert!(src.contains("WorldFile.cs:3359"), "the writer's anchor");
        assert!(src.contains("WorldFile.cs:3373"), "the reader's anchor");
        for anchor in [
            "// `SaveWeightedPressurePlates:3359-3372`",
            "// `LoadWeightedPressurePlates:3373-3384`",
        ] {
            assert!(src.contains(anchor), "missing {anchor}");
        }
    }
}
