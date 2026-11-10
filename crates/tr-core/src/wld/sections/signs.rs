//! Section 5 (pointer 4): signs — `WorldFile.cs:1678` (`SaveSigns`) /
//! `WorldFile.cs:2828` (`LoadSigns`).
//!
//! Wire shape: `i16 count` of the placed signs, then per sign **`string text,
//! i32 x, i32 y`** — the text comes before the position
//! (`WorldFile.cs:1695-1697`), over `Main.sign[0..31999]` filtered
//! `text != null`.
//!
//! The store is `World.signs`: a `Vec<WldSign>` holding exactly the non-null
//! signs in slot order. `LoadSigns` defines every one of the 32 000 slots it
//! walks (a record writes its slot or nulls it, `:2831-2842`, and the tail is
//! nulled at `:2869-2873`), so the array's whole observable state is that
//! subsequence — a `Vec<Option<_>>` would mirror the C# array shape while
//! carrying 32 000 nulls, and a map would lose the slot order the writer
//! walks. The two halves below are the section's load fix-ups and save filter;
//! they read and write `World` directly, which is why this section needs no
//! app-level glue.
//!
//! **Save-side contract:** `write` reads [`SectionData`] (the frozen
//! `SAVE_DEFS` closure), so the caller materializes the bag with
//! [`from_world`] before invoking `wld::to_bytes`/`wld::save` — a world whose
//! signs were loaded from a `.wld` must not save an empty section.
//!
//! The file is untrusted input: a count above `Sign.maxSigns`, a sign whose
//! text prefix overruns, or any short read is refused rather than trusted,
//! and no record pre-allocates from a hostile length.

use super::{SectionData, SectionPayload};
use crate::binio::{Reader, Writer};
use crate::world::World;

/// `Sign.maxSigns` (`Sign.cs:8`): the length of the `Main.sign[]` array
/// `SaveSigns`/`LoadSigns` count and walk. A file carrying more signs than
/// slots is not a vanilla file (`LoadSigns:2841` would throw
/// `IndexOutOfRangeException`).
pub const MAX_SIGNS: i16 = 32_000;

/// One sign record: `(string text, i32 x, i32 y)` — text first
/// (`WorldFile.cs:1695-1697`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WldSign {
    pub text: String,
    pub x: i32,
    pub y: i32,
}

/// `Main.tileSign` (`Main.cs:10081-10084`): the tile kinds a sign text may sit
/// on (55 Sign, 85 Tattered Wood Sign, 425 Illuminant Hanging Sign, 573
/// Sunplate Sign). Hand-transcribed exactly as `chest.rs:65-67`'s
/// `is_tile_container` — the set is not in the generated tables.
fn is_sign_tile(kind: u16) -> bool {
    matches!(kind, 55 | 85 | 425 | 573)
}

pub fn implemented() -> bool {
    true
}

/// `LoadSigns:2828-2843`: the count, then each record's text and position.
/// The per-record tile check and the duplicate/tail passes need the world and
/// live in [`apply_loaded`].
pub fn read(r: &mut Reader, _version: i32) -> Option<Vec<WldSign>> {
    let count = r.i16()?;
    if !(0..=MAX_SIGNS).contains(&count) {
        return None; // `Main.sign[i]` has 32 000 slots (`LoadSigns:2841`)
    }
    let mut signs: Vec<WldSign> = Vec::new();
    for _ in 0..count {
        signs.push(WldSign {
            text: r.cs_string_lossy()?,
            x: r.i32()?,
            y: r.i32()?,
        });
    }
    Some(signs)
}

/// `SaveSigns:1678-1699` — the write half only. The C# writer walks
/// `Main.sign` in slot order and emits the non-null signs, which is exactly
/// what the caller's `from_world` list holds.
pub fn write(w: &mut Writer, data: &SectionData, _version: i32) {
    // `writer.Write(num)` (`WorldFile.cs:1693`) — the counted survivors.
    w.i16(data.signs.len() as i16);
    for sign in &data.signs {
        // `writer.Write(sign.text)` (`WorldFile.cs:1695`).
        w.cs_string(&sign.text);
        w.i32(sign.x);
        w.i32(sign.y);
    }
}

/// `LoadSigns:2831-2873`: the section's load fix-ups against the live world.
/// Three passes, mirroring the C# one for one:
/// 1. a record survives only if the tile at its position is active and is a
///    sign tile (`LoadSigns:2838`, `tile.active() && Main.tileSign[tile.type]`);
/// 2. the survivors are walked in slot order and a later sign on an already
///    seen position is nulled (`LoadSigns:2845-2865` — the first wins);
/// 3. the tail past the last record is cleared (`LoadSigns:2869-2873`).
/// Because the C# array is fully defined by this load, replacing the store
/// wholesale (2+3) is exactly the C#'s array rewrite.
pub fn apply_loaded(world: &mut World, records: &[WldSign]) {
    let mut kept: Vec<WldSign> = Vec::with_capacity(records.len());
    let mut seen: Vec<(i32, i32)> = Vec::new();
    for sign in records {
        // `Tile tile = Main.tile[num2, num3]` (`LoadSigns:2837`): a position
        // outside the world has no tile, so it drops like a removed one (the
        // C# would throw).
        let Some(tile) = world.tile(sign.x, sign.y) else {
            continue;
        };
        if !tile.active() || !is_sign_tile(tile.kind) {
            continue; // `sign = null` (`LoadSigns:2847`)
        }
        if seen.contains(&(sign.x, sign.y)) {
            continue; // `Main.sign[j] = null` (`LoadSigns:2861`)
        }
        seen.push((sign.x, sign.y));
        kept.push(sign.clone());
    }
    world.signs = kept;
}

/// `SaveSigns:1670-1691`'s filter over the live store: every stored sign is a
/// non-null `Sign` whose text is a string (an empty text still rides the file
/// — `Sign.ReadSign` creates signs with `text = ""`, `Sign.cs:52`).
pub fn from_world(world: &World) -> Vec<WldSign> {
    world.signs.clone()
}

pub fn empty(data: &SectionData) -> bool {
    data.signs.is_empty()
}

pub fn payload(value: Vec<WldSign>) -> SectionPayload {
    SectionPayload::Signs(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::{Tile, FLAG_ACTIVE};

    /// Format version the current C# writes (`SaveFileFormatHeader:1174`).
    const VER: i32 = 326;

    /// A `.NET`-style string: 7-bit length prefix + UTF-8 payload.
    fn cs_string(s: &str) -> Vec<u8> {
        let mut v = vec![s.len() as u8];
        v.extend_from_slice(s.as_bytes());
        v
    }

    fn sign(text: &str, x: i32, y: i32) -> WldSign {
        WldSign {
            text: text.into(),
            x,
            y,
        }
    }

    fn data(signs: Vec<WldSign>) -> SectionData {
        SectionData {
            signs,
            ..Default::default()
        }
    }

    fn section(signs: &[WldSign]) -> Vec<u8> {
        let mut w = Writer::new();
        write(&mut w, &data(signs.to_vec()), VER);
        w.into_inner()
    }

    fn read_at(bytes: &[u8]) -> (Option<Vec<WldSign>>, usize) {
        let mut r = Reader::new(bytes);
        (read(&mut r, VER), r.pos())
    }

    /// A world with a tile at (x, y): sign kind 55 when `sign_tile`, plain
    /// rock (kind 0) otherwise.
    fn world_with(x: i32, y: i32, sign_tile: bool) -> World {
        let mut w = World::new("t".into(), 32, 32);
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind: if sign_tile { 55 } else { 0 },
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w
    }

    // --- the stub lane ----------------------------------------------------

    #[test]
    fn test_DW_5_5_the_section_is_implemented() {
        assert!(implemented(), "phase 5 replaces the stub in place");
        assert!(empty(&data(Vec::new())));
        assert!(!empty(&data(vec![sign("hi", 1, 1)])));
    }

    // --- DW-5.1: the round-trip and the load fix-ups ----------------------

    #[test]
    fn test_DW_5_1_the_section_round_trips_byte_equal() {
        let signs = vec![sign("Entering the caves", 12, 30), sign("", 4, 4)];
        let bytes = section(&signs);
        // `i16 count` + per record `(string, i32, i32)`.
        assert_eq!(bytes[..2], 2i16.to_le_bytes());
        let (read_back, pos) = read_at(&bytes);
        assert_eq!(read_back.as_ref(), Some(&signs), "the struct survives");
        assert_eq!(pos, bytes.len(), "the read consumes every byte");
    }

    #[test]
    fn test_DW_5_1_the_hand_computed_record_bytes() {
        // Hand-computed `SaveSigns:1693-1697`: count 2, then the two records
        // with the text before the position.
        let want = [
            &2i16.to_le_bytes()[..],
            &cs_string("hi")[..],
            &7i32.to_le_bytes()[..],
            &9i32.to_le_bytes()[..],
            &cs_string("")[..],
            &1i32.to_le_bytes()[..],
            &2i32.to_le_bytes()[..],
        ]
        .concat();
        assert_eq!(section(&[sign("hi", 7, 9), sign("", 1, 2)]), want);
        // The record's field order is the one thing the stub warned about:
        // text first, so the two i32s behind it are x and y in that order.
        let fresh = section(&[sign("hi", 7, 9)]);
        let text_len = fresh[2] as usize;
        assert_eq!(&fresh[3 + text_len..], &[7, 0, 0, 0, 9, 0, 0, 0]);
    }

    #[test]
    fn test_DW_5_1_a_sign_on_a_non_sign_tile_is_dropped_on_load() {
        // `LoadSigns:2838`: only an active sign tile keeps its record.
        let mut world = world_with(5, 5, false);
        apply_loaded(&mut world, &[sign("x", 5, 5)]);
        assert!(world.signs.is_empty(), "a rock tile holds no sign text");
        let mut world = world_with(5, 5, true);
        apply_loaded(&mut world, &[sign("x", 5, 5)]);
        assert_eq!(world.signs.len(), 1, "the sign tile keeps it");
    }

    #[test]
    fn test_DW_5_1_a_sign_whose_tile_was_removed_is_dropped_on_load() {
        // An inactive tile of the right kind is still no anchor
        // (`tile.active() && Main.tileSign[tile.type]` — both must hold).
        let mut world = world_with(6, 6, true);
        let i = world.index(6, 6);
        world.tiles[i].flags &= !FLAG_ACTIVE;
        apply_loaded(&mut world, &[sign("x", 6, 6)]);
        assert!(world.signs.is_empty());
    }

    #[test]
    fn test_DW_5_1_a_later_duplicate_position_is_nulled() {
        // `LoadSigns:2845-2865`: the ascending walk keeps the first sign on a
        // position and nulls the later ones.
        let mut world = world_with(3, 3, true);
        apply_loaded(
            &mut world,
            &[
                sign("first", 3, 3),
                sign("second", 3, 3),
                sign("third", 3, 3),
                sign("", 9, 9),
            ],
        );
        assert_eq!(
            world.signs,
            vec![sign("first", 3, 3)],
            "the first record on a position wins; (9,9) is not a sign tile"
        );
    }

    #[test]
    fn test_DW_5_1_the_duplicate_walk_keeps_distinct_positions() {
        // Two records on different positions both survive; the third is a
        // repeat of the first.
        let mut world = World::new("t".into(), 32, 32);
        for (x, y) in [(3, 3), (4, 3)] {
            let i = world.index(x, y);
            world.tiles[i] = Tile {
                kind: 55,
                ..Default::default()
            };
            world.tiles[i].flags |= FLAG_ACTIVE;
        }
        apply_loaded(
            &mut world,
            &[
                sign("first", 3, 3),
                sign("second", 4, 3),
                sign("third", 3, 3),
            ],
        );
        assert_eq!(
            world.signs,
            vec![sign("first", 3, 3), sign("second", 4, 3)],
            "the first record on a position wins"
        );
    }

    #[test]
    fn test_DW_5_1_the_tail_is_cleared_and_the_store_replaced() {
        // `LoadSigns:2869-2873` nulls every slot past the record count, so a
        // store already holding signs is replaced, not merged.
        let mut world = world_with(3, 3, true);
        let i = world.index(20, 20);
        world.tiles[i] = Tile {
            kind: 55,
            ..Default::default()
        };
        world.tiles[i].flags |= FLAG_ACTIVE;
        world.signs = vec![sign("stale", 20, 20)];
        apply_loaded(&mut world, &[sign("fresh", 3, 3)]);
        assert_eq!(world.signs, vec![sign("fresh", 3, 3)]);
    }

    #[test]
    fn test_DW_5_1_the_store_survives_a_whole_wld_round_trip() {
        // The registry flip end to end: the section bytes land back in the bag
        // through `load_sections`' end-offset check.
        let mut world = World::new("S".into(), 8, 6);
        let i = world.index(2, 2);
        world.tiles[i] = Tile {
            kind: 55,
            ..Default::default()
        };
        world.tiles[i].flags |= FLAG_ACTIVE;
        apply_loaded(&mut world, &[sign("hello", 2, 2)]);
        let file = crate::wld::to_bytes(&world, &data(from_world(&world)), 1).expect("writes");
        let table = crate::wld::SectionTable::open(&file).expect("opens");
        let loaded = crate::wld::load_sections(&table).expect("walks its sections");
        assert_eq!(loaded.signs, vec![sign("hello", 2, 2)]);
        // …and the same store writes back out again unchanged. The revision
        // rides the 20-byte metadata, so both writes take the same one to keep
        // the comparison on the sections.
        let file2 = crate::wld::to_bytes(&world, &data(loaded.signs), 1).expect("writes");
        assert_eq!(file, file2, "the round-trip is byte-stable");
    }

    // --- the defensive arms -----------------------------------------------

    #[test]
    fn a_sign_count_past_the_array_is_refused() {
        // `Main.sign[i]` has 32 000 slots; a count past it is not a vanilla
        // file, and a negative one is a hostile count.
        let mut bytes = 32_001i16.to_le_bytes().to_vec();
        bytes.extend_from_slice(&vec![0u8; 32_001 * 8]);
        assert!(read_at(&bytes).0.is_none());
        assert!(read_at(&(-1i16).to_le_bytes()).0.is_none());
    }

    #[test]
    fn a_truncated_record_is_refused() {
        let mut bytes = 1i16.to_le_bytes().to_vec();
        bytes.extend_from_slice(&cs_string("half"));
        assert!(read_at(&bytes).0.is_none(), "the position is missing");
    }

    #[test]
    fn a_hostile_string_prefix_does_not_allocate() {
        // A 5-byte 7-bit length prefix asks for ~2^28 bytes; the read must
        // refuse (and must not pre-allocate) inside a 5-byte buffer.
        let mut bytes = 1i16.to_le_bytes().to_vec();
        bytes.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0x7f]);
        assert!(read_at(&bytes).0.is_none());
    }

    #[test]
    fn a_sign_outside_the_world_is_dropped_on_load() {
        // `Main.tile[num2, num3]` throws out of bounds in the C#; the port's
        // tile probe drops the record instead.
        let mut world = World::new("t".into(), 8, 8);
        apply_loaded(&mut world, &[sign("far", -5, 3), sign("far", 3, 99)]);
        assert!(world.signs.is_empty());
    }

    #[test]
    fn an_empty_text_sign_still_rides_the_file() {
        // `Sign.ReadSign` creates signs with `text = ""` (`Sign.cs:52`) and
        // `SaveSigns` filters on `text != null`, so an empty text is written
        // and read back.
        let (read_back, pos) = read_at(&section(&[sign("", 1, 2)]));
        assert_eq!(read_back, Some(vec![sign("", 1, 2)]));
        assert_eq!(pos, 2 + 1 + 8);
    }

    /// The house rule's citation probe: the source carries the anchors the
    /// golden rule demands for every ported block.
    #[test]
    fn test_DW_5_5_the_citations_are_present() {
        let src = include_str!("signs.rs");
        // The DW item's named anchors, in the house style.
        assert!(src.contains("WorldFile.cs:1678"), "the writer's anchor");
        assert!(src.contains("WorldFile.cs:2828"), "the reader's anchor");
        for anchor in [
            "// `LoadSigns:2828-2843`",
            "// `SaveSigns:1678-1699`",
            "// `LoadSigns:2831-2873`",
            "// `LoadSigns:2845-2865`",
            "// `SaveSigns:1670-1691`'s filter",
            "// `Main.tileSign` (`Main.cs:10081-10084`)",
        ] {
            assert!(src.contains(anchor), "missing {anchor}");
        }
    }
}
