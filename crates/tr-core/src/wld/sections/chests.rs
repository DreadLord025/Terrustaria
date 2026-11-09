//! Section 4 (pointer 3): chests — `WorldFile.SaveChests:1606` (write) /
//! `WorldFile.LoadChests:2723` (read).
//!
//! Wire shape: `i16 count` of survivors, then per chest `(i32 x, i32 y,
//! string name, i32 maxItems)` (the `maxItems` field from version 294; older
//! files carry one shared slot-count `i16` before the loop), then per slot a
//! bare `i16 0` for air or a banned item, else `i16 stack` followed — only
//! when the stack is positive — by `(i32 type, u8 prefix)`.
//!
//! The file is untrusted input: nothing here pre-allocates from a hostile
//! count, a record count above [`MAX_CHESTS`] or a `maxItems` outside
//! `0..=AbsoluteMaxItems…` is refused rather than trusted, and every arm
//! mirrors a C# branch so a crafted stream cannot desynchronise the cursor.

use super::{SectionData, SectionPayload};
use crate::binio::{Reader, Writer};

/// `Chest.DefaultMaxItems` (`Chest.cs:64`): the per-chest slot count a chest
/// carries until the file says otherwise.
pub const DEFAULT_MAX_ITEMS: i32 = 40;
/// `Chest.AbsoluteMaxItemsWeCanEverReachInAChestForNow` (`Chest.cs:69`): the
/// vanilla ceiling on a `Resize` (`Chest.cs:118-130`) — the port refuses a
/// file that asks for more (the C# would `Array.Resize` a hostile size).
pub const ABSOLUTE_MAX_ITEMS: i32 = 200;
/// `Main.maxChests` (`Main.cs:1033`) — the array `SaveChests`/`LoadChests`
/// index; a file claiming more chests than slots is not a vanilla file
/// (the C# would throw `IndexOutOfRangeException`).
pub const MAX_CHESTS: i16 = 8000;

/// One slot's wire form (`SaveChests:1655-1672`): an air or banned slot is the
/// bare `i16 0`; a stored item writes `stack` then, only when it is positive,
/// `(type, prefix)`. A negative stack is the legacy read path's shape
/// (`LoadChests:2760-2769`) and is clamped to 1 on write, exactly as the C#
/// mutates its `item.stack` (`SaveChests:1663-1666`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WldChestSlot {
    pub stack: i16,
    pub kind: i32,
    pub prefix: u8,
}

/// One chest record: the `(x, y, name, maxItems)` head plus its slots
/// (`SaveChests:1650-1673`). `max_items` is [`DEFAULT_MAX_ITEMS`] for the
/// pre-294 shape, which carries no per-chest field.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WldChest {
    pub x: i32,
    pub y: i32,
    pub name: String,
    pub max_items: i32,
    pub slots: Vec<WldChestSlot>,
}

pub fn implemented() -> bool {
    true
}

pub fn payload(value: Vec<WldChest>) -> SectionPayload {
    SectionPayload::Chests(value)
}

/// `LoadChests:2723-2780`. Every value the C# reads lands in the wire struct;
/// the item-level fix-ups (`Chest.FixLoadedData`, `Item.FixAgainstExploit`)
/// and the duplicate/tail/dresser passes need the app's registry and tiles
/// and live in `crate::chest` — see `chest::from_wld` / `fix_dresser_chests`.
pub fn read(r: &mut Reader, version: i32) -> Option<Vec<WldChest>> {
    let count = r.i16()?;
    if !(0..=MAX_CHESTS).contains(&count) {
        return None; // `Main.chest[i]` has 8000 slots (`LoadChests:2734`)
    }
    // Pre-294 files carry the shared slot count before the loop
    // (`LoadChests:2726-2729`); every later file's per-chest count replaces it
    // (`num2 = num3`, `:2741`).
    let mut shared = DEFAULT_MAX_ITEMS;
    if version < 294 {
        shared = r.i16()?.into();
        if shared < 0 {
            return None;
        }
    }
    let mut chests: Vec<WldChest> = Vec::new();
    for _ in 0..count {
        chests.push(read_chest(r, version, &mut shared)?);
    }
    Some(chests)
}

/// One record (`LoadChests:2734-2780`): the head, the `Resize` and the slot
/// records. `shared` is the file's slot count — per-chest from 294 on, the
/// loop-wide one below it — and the read/skip split below follows it.
fn read_chest(r: &mut Reader, version: i32, shared: &mut i32) -> Option<WldChest> {
    let x = r.i32()?;
    let y = r.i32()?;
    let name = r.cs_string_lossy()?;
    // `chest.Resize(num3)` (`LoadChests:2738-2740`). The pre-294 shape carries
    // no per-chest field, so the chest keeps `CreateWorldChest`'s default
    // (`Chest.cs:107-109`) and the shared count alone drives the read below.
    let max_items = if version >= 294 {
        let n = r.i32()?;
        // The vanilla ceiling on a `Resize` (`Chest.cs:69`), refused rather
        // than trusted.
        if !(0..=ABSOLUTE_MAX_ITEMS).contains(&n) {
            return None;
        }
        // `num2 = num3` (`LoadChests:2741`): the split below then works off the
        // per-chest count, so a modern file always reads exactly `maxItems`
        // records.
        *shared = n;
        n
    } else {
        DEFAULT_MAX_ITEMS
    };
    // `LoadChests:2744-2753`: the file's slot count may be below the chest's
    // size (`num4` records read) or above it (`num5` skipped).
    let (read_n, skip_n) = if *shared < max_items {
        (*shared, 0)
    } else {
        (max_items, *shared - max_items)
    };
    let mut slots: Vec<WldChestSlot> = Vec::with_capacity(read_n as usize);
    for _ in 0..read_n {
        slots.push(read_slot(r)?);
    }
    // `LoadChests:2772-2779`: the surplus is consumed without storing.
    for _ in 0..skip_n {
        let stack = r.i16()?;
        if stack > 0 {
            r.i32()?;
            r.u8()?;
        }
    }
    Some(WldChest {
        x,
        y,
        name,
        max_items,
        slots,
    })
}

/// `LoadChests:2754-2771`: a positive stack reads `(type, prefix)`; a negative
/// one is the legacy shape — the type and prefix are still there and the stack
/// becomes 1 (`item.stack = 1`).
fn read_slot(r: &mut Reader) -> Option<WldChestSlot> {
    let stack = r.i16()?;
    if stack > 0 {
        return Some(WldChestSlot {
            stack,
            kind: r.i32()?,
            prefix: r.u8()?,
        });
    }
    if stack < 0 {
        return Some(WldChestSlot {
            stack: 1,
            kind: r.i32()?,
            prefix: r.u8()?,
        });
    }
    Some(WldChestSlot::default()) // bare `i16 0` — air (or a banned item)
}

/// `SaveChests:1650-1673` — the write half only. The pre-pass that destroys
/// chests with dead anchors (`SaveChests:1609-1642`) mutates the app store and
/// lives in `chest::destroy_invalid_anchors`; the caller runs it before
/// building [`SectionData`].
pub fn write(w: &mut Writer, data: &SectionData, _version: i32) {
    // The writer emits the current (`v326`) shape, as every other section's
    // write half does: the pre-294 shared-count form is a read-side concern.
    // `writer.Write(num)` (`SaveChests:1643`) — the survivors the pre-pass left.
    // Tombstones never reach here: `chest::to_wld` builds this list.
    w.i16(data.chests.len() as i16);
    for chest in &data.chests {
        w.i32(chest.x);
        w.i32(chest.y);
        w.cs_string(&chest.name);
        w.i32(chest.max_items);
        // `for (int l = 0; l < chest.maxItems; l++)` (`:1655`): the C# array is
        // `maxItems` long, so the section always carries exactly that many
        // records — a short wire list pads with air, or the reader could not
        // read the section back.
        for i in 0..chest.max_items.max(0) {
            let air = WldChestSlot::default();
            write_slot(w, chest.slots.get(i as usize).unwrap_or(&air));
        }
    }
}

/// `SaveChests:1657-1672`: an air slot (no stack and no stored item) writes the
/// bare `i16 0`; a negative stack is clamped to 1 first, and any positive
/// stack is followed by `(type, prefix)`.
fn write_slot(w: &mut Writer, slot: &WldChestSlot) {
    if slot.kind == 0 {
        w.i16(0);
        return;
    }
    let stack = if slot.stack < 0 { 1 } else { slot.stack };
    w.i16(stack);
    if stack > 0 {
        w.i32(slot.kind);
        w.u8(slot.prefix);
    }
}

pub fn empty(data: &SectionData) -> bool {
    data.chests.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Format version the current C# writes (`SaveFileFormatHeader:1174`).
    const VER: i32 = 326;

    /// A `.NET`-style string: 7-bit length prefix + UTF-8 payload.
    fn cs_string(s: &str) -> Vec<u8> {
        let mut v = vec![s.len() as u8];
        v.extend_from_slice(s.as_bytes());
        v
    }

    fn slot(kind: i32, stack: i16, prefix: u8) -> WldChestSlot {
        WldChestSlot {
            stack,
            kind,
            prefix,
        }
    }

    fn chest(x: i32, y: i32, name: &str, max: i32, slots: Vec<WldChestSlot>) -> WldChest {
        WldChest {
            x,
            y,
            name: name.into(),
            max_items: max,
            slots,
        }
    }

    fn section(data: &SectionData) -> Vec<u8> {
        let mut w = Writer::new();
        write(&mut w, data, VER);
        w.into_inner()
    }

    fn data(chests: Vec<WldChest>) -> SectionData {
        SectionData {
            chests,
            ..Default::default()
        }
    }

    /// The empty encoding: the current form is `i16 0`; a pre-294 reader also
    /// consumes the shared slot count (`LoadChests:2726-2729`), which is the
    /// shape the P1 fixture's legacy files carry.
    #[test]
    fn the_empty_section_is_the_count_only() {
        assert_eq!(section(&data(Vec::new())), vec![0, 0]);
        assert_eq!(
            read(&mut Reader::new(&[0, 0, 0, 0]), 293),
            Some(Vec::new()),
            "the 4-byte legacy empty section reads"
        );
    }

    // --- DW-3.1 -----------------------------------------------------------

    #[test]
    fn test_DW_3_1_a_multi_chest_store_round_trips_byte_equal() {
        // Two chests: a full 40-slot one with an empty name, airs and a
        // prefixed item, and a non-40 chest with a named slot layout.
        let mut full = Vec::new();
        full.push(slot(9, 6, 0)); // wood
        full.push(EMPTYish());
        full.push(slot(619, 4, 1)); // prefixed
        full.push(slot(619, 4, 0)); // same kind, no prefix — a distinct slot
        full.resize(40, EMPTYish());
        let small = vec![slot(71, 99, 0), slot(72, 1, 0)];
        let store = data(vec![
            chest(10, 18, "", 40, full.clone()),
            chest(30, 40, "Loot", 2, small.clone()),
        ]);
        let bytes = section(&store);
        let mut r = Reader::new(&bytes);
        let back = read(&mut r, VER).expect("reads back");
        assert_eq!(r.pos(), bytes.len(), "the section is fully consumed");
        assert_eq!(
            back,
            vec![
                chest(10, 18, "", 40, full.clone()),
                chest(30, 40, "Loot", 2, small.clone())
            ],
            "every field of every record survives"
        );
        // The writer is a pure function of the decoded store: re-encoding the
        // read-back value reproduces the identical bytes (byte-equal both ways).
        let mut w = Writer::new();
        write(&mut w, &data(back), VER);
        assert_eq!(w.into_inner(), bytes, "re-encode is byte-equal");
    }

    #[test]
    fn test_DW_3_1_the_wire_bytes_are_hand_computed() {
        // Hand-computed from `SaveChests:1644-1673`: one chest at (10, 18)
        // named "" with 40 slots — slot 0 a prefixed 619×4, the rest air.
        let mut slots = vec![slot(619, 4, 1)];
        slots.resize(40, EMPTYish());
        let bytes = section(&data(vec![chest(10, 18, "", 40, slots)]));
        let mut want = Vec::new();
        want.extend_from_slice(&1i16.to_le_bytes()); // count
        want.extend_from_slice(&10i32.to_le_bytes()); // x
        want.extend_from_slice(&18i32.to_le_bytes()); // y
        want.extend(cs_string("")); // name (the empty string: one 0 byte)
        want.extend_from_slice(&40i32.to_le_bytes()); // maxItems
        want.extend_from_slice(&4i16.to_le_bytes()); // slot 0: stack
        want.extend_from_slice(&619i32.to_le_bytes()); // type
        want.push(1); // prefix
        want.extend_from_slice(&[0u8; 2 * 39]); // 39 air slots
        assert_eq!(bytes, want);
    }

    // --- DW-3.2: the hostile read paths ------------------------------------

    #[test]
    fn test_DW_3_2_the_legacy_shared_count_skips_the_surplus_byte_exact() {
        // A pre-294 file: one shared slot count of 50 for one chest of 40.
        // `LoadChests:2744-2779` reads 40 records and skips 10.
        let mut b = Vec::new();
        b.extend_from_slice(&1i16.to_le_bytes()); // chest count
        b.extend_from_slice(&50i16.to_le_bytes()); // the shared slot count
        b.extend_from_slice(&10i32.to_le_bytes());
        b.extend_from_slice(&18i32.to_le_bytes());
        b.extend(cs_string("Old"));
        // 40 stored slots + 10 air records to skip.
        for i in 0..50 {
            if i % 5 == 0 {
                b.extend_from_slice(&7i16.to_le_bytes());
                b.extend_from_slice(&9i32.to_le_bytes());
                b.push(0);
            } else {
                b.extend_from_slice(&0i16.to_le_bytes());
            }
        }
        let mut r = Reader::new(&b);
        let back = read(&mut r, 293).expect("a legacy file reads");
        assert_eq!(r.pos(), b.len(), "the surplus is consumed byte-exact");
        assert_eq!(
            back[0].max_items, 40,
            "the C# default carries the pre-294 shape"
        );
        assert_eq!(back[0].slots.len(), 40, "only the chest's size is stored");
        assert_eq!(back[0].slots[0].kind, 9);
        assert_eq!(back[0].slots[1].stack, 0, "the air slot is the bare 0");
    }

    #[test]
    fn test_DW_3_2_a_legacy_negative_stack_reads_as_one() {
        // `LoadChests:2764-2769`: the type and prefix are still written for a
        // negative stack, and the loaded stack is 1. (A pre-294 file, where
        // the shape is the legacy one: the shared count drives the loop.)
        let mut b = Vec::new();
        b.extend_from_slice(&1i16.to_le_bytes()); // chest count
        b.extend_from_slice(&2i16.to_le_bytes()); // the shared slot count
        b.extend_from_slice(&10i32.to_le_bytes());
        b.extend_from_slice(&18i32.to_le_bytes());
        b.extend(cs_string("Old"));
        b.extend_from_slice(&(-5i16).to_le_bytes()); // the legacy negative stack
        b.extend_from_slice(&9i32.to_le_bytes()); // the type is still there
        b.push(0); // and the prefix
        b.extend_from_slice(&0i16.to_le_bytes()); // one air slot
        let mut r = Reader::new(&b);
        let back = read(&mut r, 293).expect("reads");
        assert_eq!(
            back[0].max_items, 40,
            "the C# default carries the pre-294 shape"
        );
        assert_eq!(back[0].slots[0].stack, 1, "`item.stack = 1`");
        assert_eq!(back[0].slots[0].kind, 9);
        assert_eq!(r.pos(), b.len());
    }

    #[test]
    fn test_DW_3_2_a_short_or_implausible_record_refuses() {
        // No pre-allocation from a hostile count: every malformed shape is a
        // clean `None`, never a panic (the section's untrusted entry).
        let mut b = Vec::new();
        b.extend_from_slice(&1i16.to_le_bytes());
        b.extend_from_slice(&10i32.to_le_bytes());
        b.extend_from_slice(&18i32.to_le_bytes());
        b.extend(cs_string(""));
        b.extend_from_slice(&(ABSOLUTE_MAX_ITEMS + 1).to_le_bytes()); // Resize past the C# ceiling
        assert!(read(&mut Reader::new(&b), VER).is_none());
        let mut b = b.clone();
        let n = b.len() - 4;
        b[n..].copy_from_slice(&(-1i32).to_le_bytes()); // a negative maxItems
        assert!(read(&mut Reader::new(&b), VER).is_none());
        let mut b = Vec::new();
        b.extend_from_slice(&(MAX_CHESTS + 1).to_le_bytes()); // more chests than slots
        assert!(read(&mut Reader::new(&b), VER).is_none());
        assert!(
            read(&mut Reader::new(&[1, 0]), VER).is_none(),
            "a truncated record"
        );
    }

    // --- DW-3.3 -----------------------------------------------------------

    #[test]
    fn test_DW_3_3_a_negative_stack_writes_as_one() {
        // `SaveChests:1663-1676`: the mutation fires before the write, so the
        // file carries stack 1 *and* the type/prefix pair (1 > 0).
        let bytes = section(&data(vec![chest(10, 18, "", 40, vec![slot(9, -5, 3)])]));
        let mut want = Vec::new();
        want.extend_from_slice(&1i16.to_le_bytes());
        want.extend_from_slice(&10i32.to_le_bytes());
        want.extend_from_slice(&18i32.to_le_bytes());
        want.extend(cs_string(""));
        want.extend_from_slice(&40i32.to_le_bytes());
        want.extend_from_slice(&1i16.to_le_bytes()); // the clamped stack
        want.extend_from_slice(&9i32.to_le_bytes());
        want.push(3);
        want.extend_from_slice(&[0u8; 2 * 39]); // the 39 remaining air records
        assert_eq!(bytes, want);
    }

    #[test]
    fn a_short_wire_list_pads_to_max_items() {
        // The C# `for l < maxItems` loop writes `maxItems` records whatever the
        // list carries, so a chest that claims 3 slots but stores 1 still emits
        // a section the reader can consume (`LoadChests:2754`'s `num4` loop).
        let bytes = section(&data(vec![chest(1, 2, "", 3, vec![slot(9, 6, 0)])]));
        let back = read(&mut Reader::new(&bytes), VER).expect("the padded section reads");
        assert_eq!(back[0].slots.len(), 3);
        assert_eq!(back[0].slots[0].kind, 9);
        assert_eq!(back[0].slots[1], EMPTYish());
    }

    // --- the framing both directions -----------------------------------------

    #[test]
    fn test_DW_3_4_a_framed_file_carries_the_chests_section() {
        // The whole-file cycle (`SaveWorld_Version2:1150` /
        // `LoadWorld_Version2:1781-1785`): the section's end offset is written
        // into pointer 3 and the loader verifies it after reading every record.
        let world = crate::world::World::new("C".into(), 8, 6);
        // The C# array shape the writer emits: exactly `maxItems` records per
        // chest (the app-side `chest::to_wld` builds this same shape).
        let mut full = vec![slot(9, 6, 0)];
        full.resize(40, EMPTYish());
        let small = vec![EMPTYish(), slot(619, 4, 1), EMPTYish()];
        let store = data(vec![
            chest(10, 18, "", 40, full.clone()),
            chest(30, 40, "Vault", 3, small.clone()),
        ]);
        let buf = crate::wld::to_bytes(&world, &store, 1).expect("writes");
        let table = crate::wld::SectionTable::open(&buf).expect("opens");
        let back = crate::wld::load_sections(&table).expect("the framed file walks");
        assert_eq!(back.chests, store.chests);
        // Pointer 3 records the section's end, and the tiles' pointer already
        // bounds its start: the section is exactly the bytes between them.
        let start = table_pointers(&buf, 2);
        let end = table_pointers(&buf, 3);
        let mut r = Reader::new(&buf[start..end]);
        assert_eq!(read(&mut r, VER), Some(store.chests.clone()));
        assert_eq!(r.pos(), end - start);
    }

    /// Pointer `i` of an assembled file (the prologue's table).
    fn table_pointers(buf: &[u8], i: usize) -> usize {
        let at = 26 + 4 * i;
        i32::from_le_bytes(buf[at..at + 4].try_into().unwrap()) as usize
    }

    /// An air slot's wire form (the C#'s `item == null` / zero-stack slot).
    fn EMPTYish() -> WldChestSlot {
        WldChestSlot::default()
    }
}
