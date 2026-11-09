//! Section 7 (pointer 6): tile entities — `WorldFile.cs:3302`
//! (`SaveTileEntities`) / `WorldFile.cs:3315` (`LoadTileEntities`).
//!
//! Wire shape: `i32 count`, then per entity `TileEntity.Write`
//! (`TileEntity.cs:173-175`): `byte type`, `i32 ID`, `i16 X`, `i16 Y`, then the
//! kind's `WriteExtraData`. **The stored ids are discarded on load** and
//! reassigned sequentially (`WorldFile.cs:3323-3324`), which is why the record
//! carries an `id` the loader throws away; the writer emits the live book's
//! ids, gaps included, exactly as the C#'s `ByID` keys do.
//!
//! The payload shapes (`TEItemFrame.cs:47-56`, `TEHatRack.cs:59-80`,
//! `TETrainingDummy.cs`, `TELogicSensor.cs`,
//! `TELeashedEntityAnchorWithItem.cs:12-21`) are read and written here over the
//! kinds the port models in `crate::tile_entities::TeData`. Two kinds are not
//! modelled:
//! - `TEDisplayDoll` (id 3) — three presence bitsBytes, a pose byte and up to
//!   nineteen item triples behind the 307/308/311 gates (`:157-213`). The
//!   record's bytes are consumed exactly (so a foreign file stays in step) but
//!   the port keeps no state for them, so the entity leaves the books at load
//!   rather than being written back as a record no vanilla reader could parse.
//! - nothing else: the kite/critter anchors' single `short itemType` is carried
//!   (`TeData::Anchor`), and the pylon has no extra data at all.
//!
//! The load half ([`apply_loaded`]) writes `World` directly — the section has
//! no app-level glue — and the save half ([`from_world`]) reads it back.
//!
//! **Save-side contract:** `write` reads [`SectionData`] (the frozen
//! `SAVE_DEFS` closure), so the caller materializes the bag with
//! [`from_world`] before invoking `wld::to_bytes`/`wld::save`.
//!
//! The file is untrusted input: an unknown type byte (the C#'
//! `GenerateInstance` returns null and the read dies), a short read anywhere in
//! a payload, or a hostile count is refused rather than trusted, and nothing
//! pre-allocates from the file's count.

use super::{SectionData, SectionPayload};
use crate::binio::{Reader, Writer};
use crate::tile_entities::{hat_rack_read, TeData, TeItem, TeKind, TileEntities};
use crate::world::World;

/// The item-holder item triple: `(i16 type, u8 prefix, i16 stack)`, written
/// unconditionally for item frames / weapons racks / food platters / display
/// jars (`TEItemFrame.cs:47-56`; the port already round-trips it in
/// `tile_entities.rs:48-64`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WldTeItem {
    pub kind: i16,
    pub prefix: u8,
    pub stack: i16,
}

/// The extra payload of an entity kind, exactly the bytes `WriteExtraData`
/// emits. [`WldTeExtra::Unported`] is the display doll: its shape is consumed
/// on read and re-emitted empty on write, so a stream stays in step.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum WldTeExtra {
    /// No extra data at all — `TETeleportationPylon` overrides neither
    /// `WriteExtraData` nor `ReadExtraData` (`TileEntity.cs:229-237`).
    #[default]
    None,
    /// Item holders: item frames, weapons racks, food platters, display jars.
    Item { item: WldTeItem },
    /// Hat rack: a presence bitsByte, then up to four triples — two items and
    /// two dyes (`TEHatRack.cs:59-80`).
    HatRack {
        items: [Option<WldTeItem>; 2],
        dyes: [Option<WldTeItem>; 2],
    },
    /// Training dummy: the tied npc id (`TETrainingDummy.npc`, on the wire as a
    /// `short`).
    Dummy { npc: i32 },
    /// Logic sensor: the check byte and the on flag (`TELogicSensor.cs`).
    Logic { check: u8, on: bool },
    /// Kite/critter anchor: the held item type
    /// (`TELeashedEntityAnchorWithItem.cs:12-21`, a bare `short`).
    Anchor { item_type: i16 },
    /// `TEDisplayDoll`: the record's bytes are read and discarded.
    Unported,
}

/// One entity record head (`TileEntity.Write`, `SaveTileEntities:3309`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WldTileEntity {
    /// `TileEntity.ID` (`WriteInner:186-189`) — discarded by the load, which
    /// reassigns 0..count-1.
    pub id: i32,
    /// `TileEntity.type`, the `TileEntitiesManager.cs:26-40` registration
    /// order.
    pub ty: u8,
    /// Position; stored x/y are i16 (`Point16`).
    pub x: i16,
    pub y: i16,
    pub extra: WldTeExtra,
}

pub fn implemented() -> bool {
    true
}

/// `LoadTileEntities:3316-3333`: the count and the records. `TileEntity.Clear`
/// and the post-loop passes need the world and live in [`apply_loaded`].
pub fn read(r: &mut Reader, version: i32) -> Option<Vec<WldTileEntity>> {
    let count = r.i32()?;
    if count < 0 {
        return None; // a hostile count (the C# `ByID.Count` is never negative)
    }
    let mut entities: Vec<WldTileEntity> = Vec::new();
    for _ in 0..count {
        entities.push(read_entity(r, version)?);
    }
    Some(entities)
}

/// One record: the head, then the kind's extra data
/// (`TileEntity.Read:178-185` → `ReadInner:196-201` → `ReadExtraData`).
fn read_entity(r: &mut Reader, version: i32) -> Option<WldTileEntity> {
    let ty = r.u8()?;
    let id = r.i32()?;
    let x = r.i16()?;
    let y = r.i16()?;
    let extra = read_extra(r, ty, version)?;
    Some(WldTileEntity {
        id,
        ty,
        x,
        y,
        extra,
    })
}

/// The per-kind `ReadExtraData`. An unknown type byte refuses the whole
/// section: `TileEntity.manager.GenerateInstance` returns null on one
/// (`TileEntitiesManager.cs:65-72`) and the C#'s `ReadInner` call on null dies.
fn read_extra(r: &mut Reader, ty: u8, version: i32) -> Option<WldTeExtra> {
    let kind = TeKind::from_id(ty)?;
    Some(match kind {
        // `TEItemFrame/TEWeaponsRack/TEFoodPlatter/TEDeadCellsDisplayJar.
        // WriteExtraData`: the triple, unconditional.
        TeKind::ItemFrame
        | TeKind::WeaponsRack
        | TeKind::FoodPlatter
        | TeKind::DeadCellsDisplayJar => WldTeExtra::Item {
            item: read_item(r)?,
        },
        // `TEHatRack.cs:59-80`: the presence bitsByte, then each present slot.
        TeKind::HatRack => {
            let (items, dyes) = hat_rack_read(r)?;
            WldTeExtra::HatRack {
                items: wire_slots_from(&items),
                dyes: wire_slots_from(&dyes),
            }
        }
        // `TETrainingDummy.WriteExtraData`: `(short)this.npc`.
        TeKind::TrainingDummy => WldTeExtra::Dummy {
            npc: r.i16()? as i32,
        },
        // `TELogicSensor.WriteExtraData`: `(byte)logicCheck`, then `On`.
        TeKind::LogicSensor => WldTeExtra::Logic {
            check: r.u8()?,
            on: r.boolean()?,
        },
        // `TELeashedEntityAnchorWithItem.WriteExtraData`: `(short)itemType`.
        TeKind::KiteAnchor | TeKind::CritterAnchor => WldTeExtra::Anchor {
            item_type: r.i16()?,
        },
        // The pylon carries no extra bytes at all.
        TeKind::TeleportationPylon => WldTeExtra::None,
        // `TEDisplayDoll.ReadExtraData` (`:185-213`): the shape is consumed,
        // the contents are not kept.
        TeKind::DisplayDoll => {
            skip_display_doll(r, version)?;
            WldTeExtra::Unported
        }
    })
}

/// `TEDisplayDoll.ReadExtraData` verbatim, minus the item stores: two presence
/// bitsBytes, the pose byte (`>= 307`), a third bitsByte (`>= 308`), then the
/// present slots of the nine-slot equip and dye arrays, the single misc slot
/// and — for version 311 exactly — the equip slot the third bitsByte's bit 1
/// displaced to the tail (`:170-173`, `:208-213`).
fn skip_display_doll(r: &mut Reader, version: i32) -> Option<()> {
    let equip_bits = r.u8()?;
    let dye_bits = r.u8()?;
    let _pose = if version >= 307 { r.u8()? } else { 0 };
    let mut misc_bits = if version >= 308 { r.u8()? } else { 0 };
    // `gameVersion == 311`: bit 1 is lifted out of the misc mask and its triple
    // is read last.
    let tail_item = version == 311 && misc_bits & (1 << 1) != 0;
    if tail_item {
        misc_bits &= !(1 << 1);
    }
    // `(byte)bitsByte | (bitsByte3[1] ? 256 : 0)` / `(byte)bitsByte2 |
    // (bitsByte3[2] ? 256 : 0)` — the ninth slot's presence rides the third
    // bitsByte's bits 1 and 2, so the mask reaches 9 bits.
    let equip_mask = equip_bits as u32 | if misc_bits & (1 << 1) != 0 { 256 } else { 0 };
    let dye_mask = dye_bits as u32 | if misc_bits & (1 << 2) != 0 { 256 } else { 0 };
    for bit in 0..9 {
        if equip_mask & (1 << bit) != 0 {
            read_item(r)?;
        }
    }
    for bit in 0..9 {
        if dye_mask & (1 << bit) != 0 {
            read_item(r)?;
        }
    }
    // `this._misc` has one slot (`TEDisplayDoll.cs:113`).
    if misc_bits & 1 != 0 {
        read_item(r)?;
    }
    if tail_item {
        read_item(r)?;
    }
    Some(())
}

/// The triple's wire form (`TEItemFrame.cs:47-50`).
fn read_item(r: &mut Reader) -> Option<WldTeItem> {
    let item = TeItem::read(r)?;
    Some(WldTeItem {
        kind: item.kind as i16,
        prefix: item.prefix,
        stack: item.stack as i16,
    })
}

/// `SaveTileEntities:3302-3314` — the write half. The count is the live
/// `ByID` set the caller's `from_world` collected (the C# writes
/// `TileEntity.ByID.Count`).
pub fn write(w: &mut Writer, data: &SectionData, version: i32) {
    // `writer.Write(TileEntity.ByID.Count)` (`:3305`).
    w.i32(data.tile_entities.len() as i32);
    for entity in &data.tile_entities {
        // `TileEntity.Write` (`TileEntity.cs:173-175`).
        w.u8(entity.ty);
        w.i32(entity.id);
        w.i16(entity.x);
        w.i16(entity.y);
        write_extra(w, &entity.extra, version);
    }
}

/// The per-kind `WriteExtraData`, mirroring [`read_extra`] byte for byte.
fn write_extra(w: &mut Writer, extra: &WldTeExtra, version: i32) {
    match extra {
        WldTeExtra::None => {}
        WldTeExtra::Item { item } => {
            w.i16(item.kind);
            w.u8(item.prefix);
            w.i16(item.stack);
        }
        WldTeExtra::HatRack { items, dyes } => {
            // `TEHatRack.cs:59-63`: one presence byte over the four slots,
            // then each present slot's triple.
            let mut bits = 0u8;
            for (bit, item) in items.iter().chain(dyes.iter()).enumerate() {
                if item.is_some() {
                    bits |= 1 << bit;
                }
            }
            w.u8(bits);
            for item in items.iter().chain(dyes.iter()).flatten() {
                w.i16(item.kind);
                w.u8(item.prefix);
                w.i16(item.stack);
            }
        }
        WldTeExtra::Dummy { npc } => w.i16(*npc as i16),
        WldTeExtra::Logic { check, on } => {
            w.u8(*check);
            w.boolean(*on);
        }
        WldTeExtra::Anchor { item_type } => w.i16(*item_type),
        // The empty display doll: three cleared bitsBytes and the pose byte
        // (`TEDisplayDoll.cs:196-205` writes the pose unconditionally from
        // 307 on). No load ever keeps one, so this arm only fires on a
        // hand-built record — but an empty form keeps that record parseable.
        WldTeExtra::Unported => {
            w.u8(0);
            w.u8(0);
            if version >= 307 {
                w.u8(0);
            }
            if version >= 308 {
                w.u8(0);
            }
        }
    }
}

/// `LoadTileEntities:3316-3358`: the section's load fix-ups against the live
/// world. `TileEntity.Clear` is the fresh books; the loop is every record
/// landing in them with the next sequential id and a same-position record
/// replacing its predecessor; the removal pass then drops what the C#'s two
/// reasons (`WorldGen.InWorld(x, y, 1)`, `TileEntitiesManager.CheckValidTile`)
/// drop — plus the shapes this port cannot store — and the survivors get their
/// `OnWorldLoaded` walk, in `ByID` order.
pub fn apply_loaded(world: &mut World, records: &[WldTileEntity]) {
    let mut books = TileEntities::default(); // `TileEntity.Clear()` (`:3316`)
    for record in records {
        let Some(kind) = TeKind::from_id(record.ty) else {
            continue; // `read` already refused; defensive on a hand-built list
        };
        // `tileEntity.ID = num2++` (`:3323-3324`): the stored id is discarded,
        // the slot is sequential. `TryGetAt` → `Remove` inside `load_entity`.
        books.load_entity(kind, record.x, record.y, store_data(&record.extra));
    }
    drop_unstorable(world, &mut books);
    world.tile_entities = books;
    // `foreach (TileEntity value in TileEntity.ByID.Values) value.OnWorldLoaded();`
    // (`:3355-3358`).
    TileEntities::on_world_loaded(world);
}

/// `LoadTileEntities:3335-3353`: the removal pass, in the C#'s order (the
/// `ByPosition` walk, each failure collected before any removal), plus the
/// port's third reason — a kind whose payload is not stored. The port walks
/// the id book instead of `by_position` (a `HashMap` whose iteration order is
/// not defined); the set removed is the same, because a position holds exactly
/// one entity and a removal never renumbers a slot.
fn drop_unstorable(world: &World, books: &mut TileEntities) {
    let doomed: Vec<(i16, i16)> = books
        .live_entities()
        .filter(|e| {
            !in_world(world, e.x, e.y)
                || !e.kind.is_tile_valid(world, e.x as i32, e.y as i32)
                || !is_storable(e.kind)
        })
        .map(|e| (e.x, e.y))
        .collect();
    for (x, y) in doomed {
        books.remove_at(x, y);
    }
}

/// `WorldGen.InWorld(x, y, fluff: 1)` (`WorldGen.cs:9688-9695`, called from
/// `LoadTileEntities:3339`): the one-tile margin a loadable entity needs. The
/// port has no fluff variant, so the margin is spelled out here.
fn in_world(world: &World, x: i16, y: i16) -> bool {
    let (x, y) = (x as i32, y as i32);
    x >= 1 && y >= 1 && x < world.width - 1 && y < world.height - 1
}

/// Whether the port keeps the kind's payload, so a loaded entity can be
/// written back as a record any reader can parse. Only the display doll says
/// no: its `WriteExtraData` shape (three bitsBytes, a pose byte and up to
/// nineteen triples) has no home in `TeData`, and a hollow record would
/// desynchronise the vanilla reader.
fn is_storable(kind: TeKind) -> bool {
    !matches!(kind, TeKind::DisplayDoll)
}

/// The wire payload → the stored payload. The one-slot item holders' triple is
/// unconditional on the wire, so an all-zero triple is the port's air
/// (`TeData::Item(None)`) and every other triple is stored verbatim — a
/// bijection, so any file round-trips byte-exactly. The hat rack's presence is
/// its bitsByte bit, so a set bit yields the triple exactly as read.
fn store_data(extra: &WldTeExtra) -> TeData {
    match extra {
        WldTeExtra::None | WldTeExtra::Unported => TeData::None,
        WldTeExtra::Item { item } => TeData::Item(store_item(item)),
        WldTeExtra::HatRack { items, dyes } => {
            TeData::HatRack(store_slots(items), store_slots(dyes))
        }
        WldTeExtra::Dummy { npc } => TeData::Dummy(*npc),
        WldTeExtra::Logic { check, on } => TeData::Logic {
            check: *check,
            on: *on,
            // `TELogicSensor`'s liquid grace is runtime state, reset on load.
            counted: 0,
        },
        WldTeExtra::Anchor { item_type } => TeData::Anchor(*item_type as i32),
    }
}

fn store_item(item: &WldTeItem) -> Option<TeItem> {
    if item.kind == 0 && item.prefix == 0 && item.stack == 0 {
        return None; // `item.netDefaults(0)` — the air item
    }
    Some(TeItem {
        kind: item.kind as u16,
        prefix: item.prefix,
        stack: item.stack as u16,
    })
}

/// The hat rack's two arrays, presence per slot (the bitsByte bit decides, so
/// `and_then` keeps the wire's airness for the ported item kind).
fn store_slots(slots: &[Option<WldTeItem>; 2]) -> [Option<TeItem>; 2] {
    [
        slots[0].as_ref().and_then(store_item),
        slots[1].as_ref().and_then(store_item),
    ]
}

/// `SaveTileEntities:3306-3310`: the live books in `ByID` order (see
/// `TileEntities::live_entities` for the ordering guarantee), each entity's
/// payload mapped back to its wire form.
pub fn from_world(world: &World) -> Vec<WldTileEntity> {
    world
        .tile_entities
        .live_entities()
        .map(|entity| WldTileEntity {
            id: entity.id as i32,
            ty: entity.kind as u8,
            x: entity.x,
            y: entity.y,
            extra: wire_data(&entity.data),
        })
        .collect()
}

fn wire_data(data: &TeData) -> WldTeExtra {
    match data {
        TeData::None => WldTeExtra::None,
        TeData::Item(item) => WldTeExtra::Item {
            item: item.as_ref().map_or(WldTeItem::default(), wire_item),
        },
        TeData::HatRack(items, dyes) => WldTeExtra::HatRack {
            items: wire_slots_from(items),
            dyes: wire_slots_from(dyes),
        },
        TeData::Dummy(npc) => WldTeExtra::Dummy { npc: *npc },
        TeData::Logic { check, on, .. } => WldTeExtra::Logic {
            check: *check,
            on: *on,
        },
        TeData::Anchor(item_type) => WldTeExtra::Anchor {
            item_type: *item_type as i16,
        },
    }
}

fn wire_item(item: &TeItem) -> WldTeItem {
    WldTeItem {
        kind: item.kind as i16,
        prefix: item.prefix,
        stack: item.stack as i16,
    }
}

/// The store's hat-rack arrays → the wire's, one record per present slot.
fn wire_slots_from(slots: &[Option<TeItem>; 2]) -> [Option<WldTeItem>; 2] {
    [
        slots[0].as_ref().map(wire_item),
        slots[1].as_ref().map(wire_item),
    ]
}

pub fn empty(data: &SectionData) -> bool {
    data.tile_entities.is_empty()
}

pub fn payload(value: Vec<WldTileEntity>) -> SectionPayload {
    SectionPayload::TileEntities(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::{Tile, FLAG_ACTIVE};

    /// Format version the current C# writes (`SaveFileFormatHeader:1174`).
    const VER: i32 = 326;

    /// `i16` little-endian.
    fn i16_bytes(v: i16) -> Vec<u8> {
        v.to_le_bytes().to_vec()
    }

    fn item(kind: i16, prefix: u8, stack: i16) -> WldTeItem {
        WldTeItem {
            kind,
            prefix,
            stack,
        }
    }

    /// One record's head: `byte type, i32 ID, i16 X, i16 Y`.
    fn head(ty: u8, id: i32, x: i16, y: i16) -> Vec<u8> {
        [
            &[ty][..],
            &id.to_le_bytes()[..],
            &x.to_le_bytes()[..],
            &y.to_le_bytes()[..],
        ]
        .concat()
    }

    fn entity(ty: u8, id: i32, x: i16, y: i16, extra: WldTeExtra) -> WldTileEntity {
        WldTileEntity {
            id,
            ty,
            x,
            y,
            extra,
        }
    }

    fn data(entities: Vec<WldTileEntity>) -> SectionData {
        SectionData {
            tile_entities: entities,
            ..Default::default()
        }
    }

    fn section(entities: &[WldTileEntity]) -> Vec<u8> {
        let mut w = Writer::new();
        write(&mut w, &data(entities.to_vec()), VER);
        w.into_inner()
    }

    fn read_at(bytes: &[u8], version: i32) -> (Option<Vec<WldTileEntity>>, usize) {
        let mut r = Reader::new(bytes);
        (read(&mut r, version), r.pos())
    }

    /// A 32×32 world with an item frame tile (395) at (10, 10) whose anchor
    /// frames pass `TEItemFrame.ValidTile` (frameX 0, frameY 0). The tile at
    /// (10, 11) is the 2×2 frame's second column, so `frameX 36` is an anchor
    /// there too.
    fn frame_world() -> World {
        let mut w = World::new("t".into(), 32, 32);
        for (x, y, frame_x) in [(10, 10, 0), (10, 11, 36)] {
            let i = w.index(x, y);
            w.tiles[i] = Tile {
                kind: 395,
                frame_x,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w
    }

    /// A hat rack tile (475) at (5, 5): `frameY == 0 && frameX % 54 == 0`.
    fn hat_rack_world() -> World {
        let mut w = frame_world();
        let i = w.index(5, 5);
        w.tiles[i] = Tile {
            kind: 475,
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
        assert!(!empty(&data(vec![entity(1, 0, 1, 1, WldTeExtra::None)])));
    }

    // --- DW-5.3: the round-trip and the id reassignment -------------------

    #[test]
    fn test_DW_5_3_the_records_round_trip_byte_equal() {
        // One of every ported payload shape, each on a record of its own.
        let records = vec![
            entity(
                1,
                0,
                10,
                10,
                WldTeExtra::Item {
                    item: item(75, 42, 250),
                },
            ),
            entity(
                5,
                1,
                5,
                5,
                WldTeExtra::HatRack {
                    items: [Some(item(100, 1, 1)), None],
                    dyes: [None, Some(item(3454, 0, 1))],
                },
            ),
            entity(0, 2, 7, 8, WldTeExtra::Dummy { npc: 7 }),
            entity(2, 3, 9, 10, WldTeExtra::Logic { check: 2, on: true }),
            entity(9, 4, 11, 12, WldTeExtra::Anchor { item_type: 4743 }),
            entity(7, 5, 13, 14, WldTeExtra::None),
        ];
        let bytes = section(&records);
        let (read_back, pos) = read_at(&bytes, VER);
        assert_eq!(read_back.as_ref(), Some(&records), "the structs survive");
        assert_eq!(pos, bytes.len(), "the read consumes every byte");
    }

    #[test]
    fn test_DW_5_3_a_record_head_is_hand_computed() {
        // `TileEntity.Write` + `WriteInner` (`TileEntity.cs:173-191`): type,
        // id, x, y — then the kind's payload. The head is 9 bytes.
        let want = [
            &[1u8][..],
            &0i32.to_le_bytes()[..],
            &10i16.to_le_bytes()[..],
            &11i16.to_le_bytes()[..],
            &75i16.to_le_bytes()[..],
            &[42u8][..],
            &250i16.to_le_bytes()[..],
        ]
        .concat();
        let bytes = section(&[entity(
            1,
            0,
            10,
            11,
            WldTeExtra::Item {
                item: item(75, 42, 250),
            },
        )]);
        assert_eq!(&bytes[..4], &1i32.to_le_bytes()[..], "`i32 count` first");
        assert_eq!(&bytes[4..], want, "then the record");
    }

    #[test]
    fn test_DW_5_3_the_stored_ids_are_discarded_and_reassigned() {
        // `LoadTileEntities:3323-3324`: a file's ids are read, dropped and
        // replaced by 0..count-1 in file order.
        let mut world = frame_world();
        let bytes = [
            &2i32.to_le_bytes()[..],
            &head(1, 7, 10, 10)[..],
            &75i16.to_le_bytes()[..],
            &[0u8][..],
            &1i16.to_le_bytes()[..],
            // The second record's anchor is frameX 36 on the same tile kind,
            // so it stays in the books (`TEItemFrame.ValidTile`).
            &head(1, 42, 10, 11)[..],
            &76i16.to_le_bytes()[..],
            &[1u8][..],
            &2i16.to_le_bytes()[..],
        ]
        .concat();
        let (records, pos) = read_at(&bytes, VER);
        assert_eq!(pos, bytes.len());
        let records = records.expect("reads");
        assert_eq!(records[0].id, 7, "the codec keeps what the file carried");
        assert_eq!(records[1].id, 42);
        apply_loaded(&mut world, &records);
        let ids: Vec<u32> = world.tile_entities.live_entities().map(|e| e.id).collect();
        assert_eq!(ids, vec![0, 1], "the load reassigns sequentially");
        // Both anchors are valid tiles here, so both survive — and the counter
        // still ends at the record count.
        assert_eq!(world.tile_entities.live_entities().count(), 2);
        assert_eq!(world.tile_entities.next_id(), 2);
    }

    #[test]
    fn test_DW_5_3_next_id_is_the_record_count_after_load() {
        // `TileEntity.TileEntitiesNextID = num` (`LoadTileEntities:3334`): the
        // counter ends at the record count, not at the survivor count.
        let mut world = frame_world();
        let records = vec![
            entity(
                1,
                0,
                10,
                10,
                WldTeExtra::Item {
                    item: item(75, 0, 1),
                },
            ),
            // (30, 30) is air: the anchor check drops this one.
            entity(
                1,
                1,
                30,
                30,
                WldTeExtra::Item {
                    item: item(75, 0, 1),
                },
            ),
        ];
        apply_loaded(&mut world, &records);
        assert_eq!(world.tile_entities.live_entities().count(), 1);
        assert_eq!(
            world.tile_entities.next_id(),
            2,
            "the counter keeps the count"
        );
    }

    #[test]
    fn test_DW_5_3_the_hat_rack_and_item_frame_payloads_survive() {
        // The two ported payload shapes ride the record, not just the stub's
        // standalone helpers: bitsByte + 5 + 5 for the rack, the unconditional
        // triple for the frame.
        let rack = entity(
            5,
            0,
            5,
            5,
            WldTeExtra::HatRack {
                items: [Some(item(100, 1, 1)), Some(item(200, 2, 2))],
                dyes: [Some(item(3454, 0, 1)), None],
            },
        );
        let frame = entity(
            1,
            1,
            10,
            10,
            WldTeExtra::Item {
                item: item(75, 42, 250),
            },
        );
        let bytes = section(&[rack.clone(), frame.clone()]);
        let (back, pos) = read_at(&bytes, VER);
        assert_eq!(back, Some(vec![rack, frame]));
        assert_eq!(pos, bytes.len());
        // The air item holder still writes its triple (5 bytes after the
        // 9-byte head): `TEItemFrame.WriteExtraData` is unconditional.
        let air = section(&[entity(
            1,
            0,
            10,
            10,
            WldTeExtra::Item {
                item: item(0, 0, 0),
            },
        )]);
        let want = [
            &[1u8][..],
            &0i32.to_le_bytes()[..],
            &10i16.to_le_bytes()[..],
            &10i16.to_le_bytes()[..],
            &0i16.to_le_bytes()[..],
            &[0u8][..],
            &0i16.to_le_bytes()[..],
        ]
        .concat();
        assert_eq!(&air[4..], want);
        assert_eq!(air.len(), 4 + 14);
    }

    #[test]
    fn test_DW_5_3_an_invalid_anchor_tile_drops_the_entity() {
        // `LoadTileEntities:3342-3346`: `CheckValidTile` failures are removed.
        // A hat rack tile at a non-anchor frame (frameX 18) is not its own
        // anchor, and a bare rock tile is not a frame tile at all.
        let mut world = hat_rack_world();
        world.tile_mut(5, 5).unwrap().frame_x = 18;
        let records = vec![entity(
            5,
            0,
            5,
            5,
            WldTeExtra::HatRack {
                items: [None, None],
                dyes: [None, None],
            },
        )];
        apply_loaded(&mut world, &records);
        assert!(world.tile_entities.get_at(5, 5).is_none(), "frameX 18");

        let mut world = frame_world();
        let records = vec![entity(
            1,
            0,
            2,
            2,
            WldTeExtra::Item {
                item: item(75, 0, 1),
            },
        )];
        apply_loaded(&mut world, &records);
        assert!(world.tile_entities.get_at(2, 2).is_none(), "no frame tile");
    }

    #[test]
    fn test_DW_5_3_an_entity_outside_the_world_is_dropped() {
        // `WorldGen.InWorld(x, y, 1)` (`LoadTileEntities:3339`): the one-tile
        // margin. (0, 0) fails it; so does (31, 31) on a 32-wide world.
        let mut world = frame_world();
        let records = vec![
            entity(
                1,
                0,
                0,
                0,
                WldTeExtra::Item {
                    item: item(75, 0, 1),
                },
            ),
            entity(
                1,
                1,
                31,
                31,
                WldTeExtra::Item {
                    item: item(75, 0, 1),
                },
            ),
            // Inside the margin but not on a frame tile — dropped below.
            entity(
                1,
                2,
                10,
                10,
                WldTeExtra::Item {
                    item: item(75, 0, 1),
                },
            ),
        ];
        apply_loaded(&mut world, &records);
        let live: Vec<(i16, i16)> = world
            .tile_entities
            .live_entities()
            .map(|e| (e.x, e.y))
            .collect();
        assert_eq!(live, vec![(10, 10)]);
    }

    #[test]
    fn test_DW_5_3_a_same_position_record_replaces_the_earlier_one() {
        // `LoadTileEntities:3326-3328`: `TryGetAt` finds the earlier entity and
        // `Remove` takes it out before the new one lands, so the position holds
        // exactly one entity and the id book keeps both slots.
        let mut world = frame_world();
        let first = entity(
            1,
            0,
            10,
            10,
            WldTeExtra::Item {
                item: item(75, 0, 1),
            },
        );
        let second = entity(
            1,
            1,
            10,
            10,
            WldTeExtra::Item {
                item: item(200, 3, 4),
            },
        );
        apply_loaded(&mut world, &[first, second.clone()]);
        assert_eq!(world.tile_entities.live_entities().count(), 1);
        assert_eq!(world.tile_entities.next_id(), 2);
        let kept = world.tile_entities.get_at(10, 10).expect("kept");
        assert_eq!(kept.data, second_extra_data(), "the later record wins");
    }

    /// `second`'s payload as the store holds it.
    fn second_extra_data() -> TeData {
        TeData::Item(Some(TeItem {
            kind: 200,
            prefix: 3,
            stack: 4,
        }))
    }

    #[test]
    fn test_DW_5_3_the_written_section_reads_back_itself() {
        // The save half over the live store: `from_world` → `write` → `read`.
        let mut world = hat_rack_world();
        apply_loaded(
            &mut world,
            &[
                entity(
                    5,
                    0,
                    5,
                    5,
                    WldTeExtra::HatRack {
                        items: [Some(item(100, 1, 1)), None],
                        dyes: [None, Some(item(3454, 0, 1))],
                    },
                ),
                entity(
                    1,
                    1,
                    10,
                    10,
                    WldTeExtra::Item {
                        item: item(75, 42, 250),
                    },
                ),
            ],
        );
        let records = from_world(&world);
        let bytes = section(&records);
        let (back, pos) = read_at(&bytes, VER);
        assert_eq!(back, Some(records));
        assert_eq!(pos, bytes.len());
    }

    // --- DW-5.4: the parse-and-discard skip-reads -------------------------

    #[test]
    fn test_DW_5_4_a_display_doll_record_is_skipped_and_the_cursor_lands_on_the_next_record() {
        // A hand-coded `TEDisplayDoll` record (`ReadExtraData:185-213`) with a
        // trailer behind it: the reader consumes exactly the doll's bytes, so
        // the cursor lands on the next record — the byte-offset assertion the
        // `tile_entities.rs:726` precedent sets.
        //
        // Presence: equip bits 0 and 2, dye bits 1 and (through the third
        // bitsByte's bit 2) 8, misc slot 0 → five triples.
        let mut stream = Vec::new();
        stream.extend_from_slice(&2i32.to_le_bytes());
        stream.extend_from_slice(&head(3, 11, 5, 6));
        stream.push(0b0000_0101); // equip presence
        stream.push(0b0000_0010); // dye presence
        stream.push(3); // the pose byte (`>= 307`)
        stream.push(0b0000_0101); // misc presence: bit 0 and bit 2
        for (kind, prefix, stack) in [
            (75i16, 1u8, 1i16),
            (76, 2, 2),   // equip slots 0 and 2
            (3454, 0, 1), // dye slot 1
            (3460, 1, 1), // dye slot 8 (the third bitsByte's bit 2)
            (150, 0, 5),  // misc slot 0
        ] {
            stream.extend_from_slice(&kind.to_le_bytes());
            stream.push(prefix);
            stream.extend_from_slice(&stack.to_le_bytes());
        }
        // A second record behind the doll proves where the cursor stopped.
        stream.extend_from_slice(&head(0, 12, 7, 8));
        stream.extend_from_slice(&7i16.to_le_bytes());
        let doll_end = stream.len();
        let trailer = b"TRAILER";
        stream.extend_from_slice(trailer);

        let (records, pos) = read_at(&stream, VER);
        let records = records.expect("reads");
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].ty, 3);
        assert_eq!(records[0].extra, WldTeExtra::Unported);
        assert_eq!(records[1].ty, 0, "the dummy behind it parsed");
        assert_eq!(records[1].extra, WldTeExtra::Dummy { npc: 7 });
        assert_eq!(pos, doll_end, "the cursor lands on the next record");
        assert_eq!(&stream[pos..], trailer);
    }

    #[test]
    fn test_DW_5_4_the_doll_skip_is_version_gated() {
        // `gameVersion >= 307` reads the pose byte and `>= 308` the third
        // bitsByte (`TEDisplayDoll.cs:195-201`), so an older file's record is
        // shorter by exactly those bytes — the skip must follow the same gates.
        fn doll_stream(version: i32) -> Vec<u8> {
            let mut v = Vec::new();
            v.extend_from_slice(&1i32.to_le_bytes());
            v.extend_from_slice(&head(3, 0, 1, 1));
            v.push(0b0000_0001); // equip slot 0 present
            v.push(0); // no dyes
            if version >= 307 {
                v.push(3); // the pose byte
            }
            if version >= 308 {
                v.push(0); // no misc slots
            }
            v.extend_from_slice(&75i16.to_le_bytes());
            v.push(0);
            v.extend_from_slice(&1i16.to_le_bytes());
            v.push(9); // a marker the reader must land on
            v
        }
        // The payload is 7 bytes below 307 (two presence bytes + one triple),
        // 8 from 307 (the pose byte) and 9 from 308 (the third bitsByte) —
        // the skip must consume exactly that, and then land on the marker.
        for (version, payload_len) in [(306, 7), (307, 8), (VER, 9)] {
            let stream = doll_stream(version);
            let (records, pos) = read_at(&stream, version);
            let records = records.expect("reads");
            assert_eq!(records.len(), 1, "version {version}");
            assert_eq!(records[0].extra, WldTeExtra::Unported);
            assert_eq!(
                pos,
                4 + 9 + payload_len,
                "version {version} consumes its own payload"
            );
            assert_eq!(stream[pos], 9, "version {version} lands on the marker");
        }
    }

    #[test]
    fn test_DW_5_4_a_dummy_and_a_sensor_record_ride_the_same_stream() {
        // The plan names the dummy and the sensor beside the doll: their
        // payloads are ported, so they ride the same stream and the cursor
        // still lands exactly at the end.
        let mut stream = Vec::new();
        stream.extend_from_slice(&3i32.to_le_bytes());
        // The doll first: three cleared bitsBytes and the pose.
        stream.extend_from_slice(&head(3, 4, 1, 1));
        stream.extend_from_slice(&[0, 0, 0, 0]);
        // Then the dummy (`short npc`) and the sensor (`byte check, bool on`).
        stream.extend_from_slice(&head(0, 5, 2, 2));
        stream.extend_from_slice(&(-1i16).to_le_bytes());
        stream.extend_from_slice(&head(2, 6, 3, 3));
        stream.push(4); // `SENSOR_LAVA`
        stream.push(1); // On
        let trailer = b"END";
        stream.extend_from_slice(trailer);

        let (records, pos) = read_at(&stream, VER);
        let records = records.expect("reads");
        assert_eq!(
            records.iter().map(|r| r.ty).collect::<Vec<_>>(),
            vec![3, 0, 2]
        );
        assert_eq!(
            records[1].extra,
            WldTeExtra::Dummy { npc: -1 },
            "−1 = no dummy spawned"
        );
        assert_eq!(records[2].extra, WldTeExtra::Logic { check: 4, on: true });
        assert_eq!(pos, stream.len() - trailer.len());
        assert_eq!(&stream[pos..], trailer);
    }

    #[test]
    fn test_DW_5_4_the_skipped_doll_leaves_the_books_on_load() {
        // The doll's record is consumed but its entity is not kept: it would
        // have to be written back from state the port does not hold.
        let mut world = frame_world();
        apply_loaded(
            &mut world,
            &[
                entity(3, 0, 10, 10, WldTeExtra::Unported),
                entity(
                    1,
                    1,
                    10,
                    10,
                    WldTeExtra::Item {
                        item: item(75, 0, 1),
                    },
                ),
            ],
        );
        assert_eq!(world.tile_entities.live_entities().count(), 1);
        assert_eq!(
            world.tile_entities.get_at(10, 10).map(|e| e.kind),
            Some(TeKind::ItemFrame)
        );
        // The id slots still count every record read.
        assert_eq!(world.tile_entities.next_id(), 2);
    }

    #[test]
    fn test_DW_5_4_the_empty_doll_form_stays_parseable() {
        // The defensive writer: a hand-built unported record re-emits the
        // cleared bitsBytes, so the stream stays in step.
        let bytes = section(&[entity(3, 0, 1, 1, WldTeExtra::Unported)]);
        let want = [
            &[3u8][..],
            &0i32.to_le_bytes()[..],
            &1i16.to_le_bytes()[..],
            &1i16.to_le_bytes()[..],
            &[0u8, 0, 0, 0][..], // three cleared bitsBytes and the pose
        ]
        .concat();
        assert_eq!(&bytes[4..], want);
        let (back, pos) = read_at(&bytes, VER);
        assert_eq!(back.map(|v| v[0].extra.clone()), Some(WldTeExtra::Unported));
        assert_eq!(pos, bytes.len());
    }

    #[test]
    fn test_DW_5_4_an_unknown_type_byte_refuses_the_section() {
        // `GenerateInstance(id)` returns null outside 0..10
        // (`TileEntitiesManager.cs:65-72`) and the C# dies on the null.
        let mut stream = Vec::new();
        stream.extend_from_slice(&1i32.to_le_bytes());
        stream.extend_from_slice(&head(11, 0, 1, 1));
        assert!(read_at(&stream, VER).0.is_none());
        let mut stream = Vec::new();
        stream.extend_from_slice(&1i32.to_le_bytes());
        stream.extend_from_slice(&head(255, 0, 1, 1));
        assert!(read_at(&stream, VER).0.is_none());
    }

    // --- the defensive arms -----------------------------------------------

    #[test]
    fn a_negative_count_is_refused() {
        assert!(read_at(&(-1i32).to_le_bytes(), VER).0.is_none());
    }

    #[test]
    fn a_truncated_payload_is_refused_at_every_kind() {
        // A record head with no payload bytes behind it: every kind that has a
        // payload refuses, and the pylon — which has none — reads.
        for ty in 0..=10u8 {
            let mut stream = Vec::new();
            stream.extend_from_slice(&1i32.to_le_bytes());
            stream.extend_from_slice(&head(ty, 0, 1, 1));
            if ty != 7 {
                stream.pop(); // drop the record's last payload byte
            }
            let (records, _) = read_at(&stream, VER);
            if ty == 7 {
                assert_eq!(
                    records.map(|v| v.len()),
                    Some(1),
                    "the pylon carries no extra data"
                );
            } else {
                assert!(records.is_none(), "type {ty} short read");
            }
        }
    }

    #[test]
    fn a_hostile_count_does_not_allocate() {
        assert!(read_at(&i32::MAX.to_le_bytes(), VER).0.is_none());
    }

    #[test]
    fn the_anchor_item_type_round_trips_as_a_short() {
        // `TELeashedEntityAnchorWithItem.cs:12-21` writes a bare `short`, and
        // the store carries it as the wider `Anchor(i32)` — the file decides
        // the width.
        let bytes = section(&[entity(9, 0, 1, 1, WldTeExtra::Anchor { item_type: 4743 })]);
        let want = [
            &[9u8][..],
            &0i32.to_le_bytes()[..],
            &1i16.to_le_bytes()[..],
            &1i16.to_le_bytes()[..],
            &4743i16.to_le_bytes()[..],
        ]
        .concat();
        assert_eq!(&bytes[4..], want);
        let (back, pos) = read_at(&bytes, VER);
        assert_eq!(
            back.map(|v| v[0].extra.clone()),
            Some(WldTeExtra::Anchor { item_type: 4743 })
        );
        assert_eq!(pos, bytes.len());
    }

    #[test]
    fn the_air_triple_maps_to_the_stores_air_item() {
        // The bijection: an all-zero triple is the port's air, and every other
        // triple is kept verbatim — so any file round-trips byte-exactly.
        let mut world = frame_world();
        // A weapons rack tile (471) at (5, 5): `frameY == 0 && frameX % 54 == 0`.
        let i = world.index(5, 5);
        world.tiles[i] = Tile {
            kind: 471,
            frame_x: 54,
            ..Default::default()
        };
        world.tiles[i].flags |= FLAG_ACTIVE;
        apply_loaded(
            &mut world,
            &[
                entity(
                    1,
                    0,
                    10,
                    10,
                    WldTeExtra::Item {
                        item: item(0, 0, 0),
                    },
                ),
                entity(
                    4,
                    1,
                    5,
                    5,
                    WldTeExtra::Item {
                        item: item(500, 0, 0),
                    },
                ),
            ],
        );
        // A weapons rack tile (471) at (5, 5): `frameY == 0 && frameX % 54 == 0`.
        let air = world.tile_entities.get_at(10, 10).expect("kept");
        assert_eq!(air.data, TeData::Item(None));
        let kept = world.tile_entities.get_at(5, 5).expect("kept");
        assert_eq!(
            kept.data,
            TeData::Item(Some(TeItem {
                kind: 500,
                prefix: 0,
                stack: 0
            }))
        );
        // Both write their triple back out unchanged.
        let bytes = section(&from_world(&world));
        let (back, pos) = read_at(&bytes, VER);
        assert_eq!(back, Some(from_world(&world)));
        assert_eq!(pos, bytes.len());
    }

    // --- DW-5.7: the registry flip in one framed file ---------------------

    #[test]
    fn test_DW_5_7_all_four_sections_participate_in_one_framed_file() {
        // The four sections of this phase, plus the two landed before them, in
        // one `.wld`: `to_bytes` writes them all and `load_sections` reads them
        // all back through the end-offset check.
        let mut world = frame_world();
        // A sign tile at (2, 2) so the sign survives its load fix-up.
        let i = world.index(2, 2);
        world.tiles[i] = Tile {
            kind: 55,
            ..Default::default()
        };
        world.tiles[i].flags |= FLAG_ACTIVE;
        crate::wld::sections::signs::apply_loaded(
            &mut world,
            &[crate::wld::sections::WldSign {
                text: "hi".into(),
                x: 2,
                y: 2,
            }],
        );
        crate::wld::sections::pressure_plates::apply_loaded(
            &mut world,
            &[crate::wld::sections::WldPressurePlate { x: 3, y: 4 }],
        );
        crate::wld::sections::tile_entities::apply_loaded(
            &mut world,
            &[entity(
                1,
                0,
                10,
                10,
                WldTeExtra::Item {
                    item: item(75, 0, 1),
                },
            )],
        );
        let mut bag = SectionData::default();
        bag.signs = crate::wld::sections::signs::from_world(&world);
        bag.pressure_plates = crate::wld::sections::pressure_plates::from_world(&world);
        bag.tile_entities = crate::wld::sections::tile_entities::from_world(&world);
        bag.town_rooms = vec![crate::wld::sections::WldTownRoom {
            npc_id: 17,
            x: 40,
            y: 41,
        }];
        let file = crate::wld::to_bytes(&world, &bag, 1).expect("writes");
        let table = crate::wld::SectionTable::open(&file).expect("opens");
        let loaded = crate::wld::load_sections(&table).expect("walks its sections");
        assert_eq!(loaded.signs, bag.signs);
        assert_eq!(loaded.pressure_plates, bag.pressure_plates);
        assert_eq!(loaded.tile_entities, bag.tile_entities);
        assert_eq!(loaded.town_rooms, bag.town_rooms);

        // …and the flip is visible: pointers 3..8 are real codecs, and Phase 6
        // (bestiary/creative powers) flipped 9..10, so every section's codec
        // has landed.
        for def in crate::wld::sections::DEFS {
            assert!(
                (def.implemented)(),
                "pointer {} ({}) has no reader",
                def.pointer,
                def.name
            );
        }
        for def in crate::wld::sections::SAVE_DEFS {
            assert!(
                (def.implemented)() || (def.empty)(&bag),
                "pointer {} has no writer but carries data",
                def.pointer
            );
        }
    }

    /// The house rule's citation probe.
    #[test]
    fn test_DW_5_5_the_citations_are_present() {
        let src = include_str!("tile_entities.rs");
        assert!(src.contains("WorldFile.cs:3302"), "the writer's anchor");
        assert!(src.contains("WorldFile.cs:3315"), "the reader's anchor");
        for anchor in [
            "// `LoadTileEntities:3316-3333`",
            "// `SaveTileEntities:3302-3314`",
            "// `LoadTileEntities:3316-3358`",
            "// `LoadTileEntities:3335-3353`",
            "// `LoadTileEntities:3339`",
            "// `TileEntity.Write` (`TileEntity.cs:173-175`)",
            "// `TEDisplayDoll.ReadExtraData` (`:185-213`)",
        ] {
            assert!(src.contains(anchor), "missing {anchor}");
        }
    }
}
