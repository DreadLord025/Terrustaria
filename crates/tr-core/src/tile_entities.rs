//! The tile-entity framework — `Terraria/DataStructures/TileEntity.cs` +
//! `TileEntitiesManager.cs` and the item-holding entity halves
//! (`GameContent/Tile_Entities/TEItemFrame.cs`, `TEWeaponsRack.cs`,
//! `TEFoodPlatter.cs`, `TEHatRack.cs`).
//!
//! Source map (line numbers into `decompiled/Terraria/`):
//! - registration order → the entity ids (`TileEntitiesManager.cs:27-40`):
//!   0 TrainingDummy, 1 ItemFrame, 2 LogicSensor, 3 DisplayDoll, 4
//!   WeaponsRack, 5 HatRack, 6 FoodPlatter, 7 TeleportationPylon, 8
//!   DeadCellsDisplayJar, 9 KiteAnchor, 10 CritterAnchor
//! - the `ByID`/`ByPosition` books (`TileEntity.cs:18-22`), the id assignment
//!   (`AssignNewID` `:40-43`)
//! - the per-TE `ValidTile` frame checks: ItemFrame (`TEItemFrame.cs:37-43`,
//!   tile 395, `frameY == 0 && frameX % 36 == 0`), WeaponsRack
//!   (`TEWeaponsRack.cs:33-36`, tile 471, `frameY == 0 && frameX % 54 == 0`),
//!   FoodPlatter (`TEFoodPlatter.cs:32-36`, tile 520, `frameY == 0`),
//!   HatRack (`TEHatRack.cs:407-411`, tile 475, `frameY == 0 &&
//!   frameX % 54 == 0`)
//! - the item wire/save shape: `short type, byte prefix, short stack`
//!   (`TEItemFrame.cs:47-56`), the HatRack presence bitsByte
//!   (`TEHatRack.cs:59-63`)
//! - the save/load record: `TileEntity.Write`/`Read` + `WriteInner`
//!   (`TileEntity.cs:173-197`) — the type byte, the StoredID, the position,
//!   then the TE's `WriteExtraData` — and the world-file section
//!   `SaveTileEntities`/`LoadTileEntities` (`WorldFile.cs:3302-3357`)
//! - the save/load halves: the books' `ByID`-order iteration and the
//!   one-record insert (`SaveTileEntities:3306-3310` /
//!   `LoadTileEntities:3316-3358`), with the entity-level wire format itself in
//!   `crate::wld::sections::tile_entities`.
//!
//! Deviations (ANALYSIS "Tile entities (M7-TE-1)"): the behaviour halves —
//! the dummy hit tracking, the logic sensors, the pylon network, the display
//! doll rendering and the leashed anchors — are M7-TE-2; the food gate
//! (`ItemID.Sets.IsFood`) is applied by the caller; items here are the
//! `(type, prefix, stack)` triples the wire/save format carries, mapped to
//! the app's `ItemSlot`s at the UI layer. Two kinds ride the `.wld` without a
//! behaviour half: the display doll's payload is read and discarded (it has no
//! home in `TeData`, so its entity is not kept), and the kite/critter
//! anchors' `short itemType` is kept as `TeData::Anchor`.

use std::collections::HashMap;

use crate::binio::{Reader, Writer};
use crate::world::World;

/// The wire/save item triple (`TEItemFrame.WriteExtraData`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TeItem {
    pub kind: u16,
    pub prefix: u8,
    pub stack: u16,
}

impl TeItem {
    pub fn is_air(&self) -> bool {
        self.kind == 0 || self.stack == 0
    }

    /// `WriteExtraData`: `short type, byte prefix, short stack` little-endian.
    pub fn write(&self, w: &mut Writer) {
        w.i16(self.kind as i16);
        w.u8(self.prefix);
        w.i16(self.stack as i16);
    }

    pub fn read(r: &mut Reader<'_>) -> Option<Self> {
        let kind = r.i16()? as u16;
        let prefix = r.u8()?;
        let stack = r.i16()? as u16;
        Some(Self {
            kind,
            prefix,
            stack,
        })
    }
}

/// The entity kinds in `TileEntitiesManager.RegisterAll` order — the enum
/// discriminant IS the network/save entity id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeKind {
    TrainingDummy = 0,
    ItemFrame = 1,
    LogicSensor = 2,
    DisplayDoll = 3,
    WeaponsRack = 4,
    HatRack = 5,
    FoodPlatter = 6,
    TeleportationPylon = 7,
    DeadCellsDisplayJar = 8,
    KiteAnchor = 9,
    CritterAnchor = 10,
}

impl TeKind {
    pub fn from_id(id: u8) -> Option<Self> {
        Some(match id {
            0 => TeKind::TrainingDummy,
            1 => TeKind::ItemFrame,
            2 => TeKind::LogicSensor,
            3 => TeKind::DisplayDoll,
            4 => TeKind::WeaponsRack,
            5 => TeKind::HatRack,
            6 => TeKind::FoodPlatter,
            7 => TeKind::TeleportationPylon,
            8 => TeKind::DeadCellsDisplayJar,
            9 => TeKind::KiteAnchor,
            10 => TeKind::CritterAnchor,
            _ => return None,
        })
    }

    /// The tile type the entity anchors to (`TE*.ValidTile`).
    pub fn tile(self) -> u16 {
        match self {
            TeKind::ItemFrame => 395,
            TeKind::WeaponsRack => 471,
            TeKind::FoodPlatter => 520,
            TeKind::HatRack => 475,
            // The behaviour halves (`TETrainingDummy.cs:95`,
            // `TELogicSensor.cs:160`, `TEDisplayDoll.cs:302`,
            // `TETeleportationPylon.cs:167`, `TEDeadCellsDisplayJar.cs:33`,
            // `TEKiteAnchor.cs:26`).
            TeKind::TrainingDummy => 378,
            TeKind::LogicSensor => 423,
            TeKind::DisplayDoll => 470,
            TeKind::TeleportationPylon => 597,
            TeKind::DeadCellsDisplayJar => 698,
            TeKind::KiteAnchor => 723,
            // `TECritterAnchor.IsTileValidForEntity`'s `tile.type == 724`
            // (`TECritterAnchor.cs:24-32`, the check at `:26`) — the released
            // critters place the anchor on a Lady Bug tile.
            TeKind::CritterAnchor => 724,
        }
    }

    /// The per-TE `ValidTile` frame check on the anchor tile.
    pub fn is_tile_valid(self, world: &World, x: i32, y: i32) -> bool {
        let Some(t) = world.tile(x, y) else {
            return false;
        };
        if !t.active() || t.kind != self.tile() {
            return false;
        }
        match self {
            // `frameY != 0 || frameX % 36 != 0` (`TEItemFrame.cs:39-42`).
            TeKind::ItemFrame => t.frame_y == 0 && t.frame_x % 36 == 0,
            // `frameY != 0 || frameX % 54 != 0` (`TEWeaponsRack.cs:35`).
            TeKind::WeaponsRack => t.frame_y == 0 && t.frame_x % 54 == 0,
            // `frameY != 0` (`TEFoodPlatter.cs:34-35`).
            TeKind::FoodPlatter => t.frame_y == 0,
            // `frameY != 0 || frameX % 54 != 0` (`TEHatRack.cs:409-410`).
            TeKind::HatRack => t.frame_y == 0 && t.frame_x % 54 == 0,
            // `frameY != 0 || frameX % 36 != 0` (`TETrainingDummy.cs:95`).
            TeKind::TrainingDummy => t.frame_y == 0 && t.frame_x % 36 == 0,
            // `frameY % 18 != 0 || frameX % 18 != 0` (`TELogicSensor.cs:158-
            // 160`).
            TeKind::LogicSensor => t.frame_y % 18 == 0 && t.frame_x % 18 == 0,
            // `frameY == 0 && frameX % 18 == 0` (`TEDeadCellsDisplayJar.cs:
            // 33-36`).
            TeKind::DeadCellsDisplayJar => t.frame_y == 0 && t.frame_x % 18 == 0,
            // `frameY != 0 || frameX % 54 != 0` (`TETeleportationPylon.cs:
            // 135-139`).
            TeKind::TeleportationPylon => t.frame_y == 0 && t.frame_x % 54 == 0,
            // `frameY != 0 || frameX % 36 != 0` (`TEDisplayDoll.cs:851-858`,
            // `IsTileValidForEntity`).
            TeKind::DisplayDoll => t.frame_y == 0 && t.frame_x % 36 == 0,
            // The kite anchor's `IsTileValidForEntity` is the anchor kind
            // alone (`TEKiteAnchor.cs:24-30`: `tile.active() && tile.type ==
            // 723`), which the head check above already settles.
            TeKind::KiteAnchor => true,
            // The critter anchor's is the kind 724 alone
            // (`TECritterAnchor.cs:24-32`), likewise settled above — the
            // recreation-release behaviour is M7-TE-2.
            TeKind::CritterAnchor => true,
        }
    }

    /// Whether the entity holds an item slot (`TEItemFrame`/`TEWeaponsRack`/
    /// `TEFoodPlatter` — one slot each; the HatRack pair via its own variant).
    pub fn has_item_slot(self) -> bool {
        matches!(
            self,
            TeKind::ItemFrame | TeKind::WeaponsRack | TeKind::FoodPlatter
        )
    }
}

/// The entity's payload. The HatRack carries its two hat slots and two dye
/// slots; the dummy carries its tied NPC (`TETrainingDummy.npc`, −1 = none);
/// the logic sensor carries its check type, the LED state and the liquid
/// grace counter (`TELogicSensor.logicCheck`/`On`/`CountedData`).
#[derive(Clone, Debug, Default, PartialEq)]
pub enum TeData {
    #[default]
    None,
    Item(Option<TeItem>),
    HatRack([Option<TeItem>; 2], [Option<TeItem>; 2]),
    /// `TETrainingDummy.npc` (−1 = no dummy spawned).
    Dummy(i32),
    /// `TELogicSensor`: the check type (`LogicCheckType` 0-6), the LED state
    /// and the liquid grace counter.
    Logic {
        check: u8,
        on: bool,
        counted: i32,
    },
    /// `TELeashedEntityAnchorWithItem.itemType`
    /// (`TELeashedEntityAnchorWithItem.cs:12`): the kite/critter anchor's held
    /// item type (0 = none). The anchor behaviour halves are M7-TE-2; the
    /// field rides the `.wld` so a file round-trips byte-exactly.
    Anchor(i32),
}

impl TeData {
    /// `WriteExtraData` per variant — the base class writes nothing
    /// (`TileEntity.cs:209-211`), and the record's type byte is what selects
    /// the half that reads it back (`TileEntity.cs:179-185`).
    pub fn write(&self, w: &mut Writer) {
        match self {
            // `TEItemFrame.cs:40-45` / `TEWeaponsRack.cs:53-58` /
            // `TEFoodPlatter.cs:41-46`: `short type, byte prefix, short
            // stack`. The C# always carries a live `Item`; the port's
            // `Option<TeItem>` fronts it with a presence byte so an empty
            // slot (the frame's `new Item()`) stays distinct from a stored
            // item.
            TeData::Item(Some(item)) => {
                w.u8(1);
                item.write(w);
            }
            TeData::Item(None) => w.u8(0),
            // `TEHatRack.cs:56-84`: the presence bitsByte, then each present
            // slot's triple.
            TeData::HatRack(items, dyes) => hat_rack_write(items, dyes, w),
            // `TETrainingDummy.cs:119-122` writes `(short)this.npc`; the port
            // keeps the tie as the `i32` NPC slot and widens the field.
            TeData::Dummy(npc) => w.i32(*npc),
            // `TELogicSensor.cs:380-387` writes `logicCheck` + `On`. The
            // liquid grace (`CountedData`) is runtime-only in the C# — the
            // port persists it so a load restores the LED state exactly.
            TeData::Logic { check, on, counted } => {
                w.u8(*check);
                w.boolean(*on);
                w.i32(*counted);
            }
            // `TELeashedEntityAnchorWithItem.cs:12-20` writes `short itemType`
            // (`TEItemFrame`'s item type without the prefix/stack); the port
            // widens the field to `i32` like the Dummy's NPC slot.
            TeData::Anchor(item) => w.i16(*item as i16),
            // `TileEntity.cs:209-211`: the pylon and the Dead Cells jar
            // carry no extra data.
            TeData::None => {}
        }
    }

    /// `ReadExtraData` for the record's type byte — the inverse of
    /// [`TeData::write`]. An unknown kind never reaches here: `TileEntity::read`
    /// maps the byte through `TeKind::from_id` first.
    pub fn read(r: &mut Reader<'_>, kind: TeKind) -> Option<Self> {
        Some(match kind {
            // `TEItemFrame.cs:47-53` / `TEWeaponsRack.cs:60-66` /
            // `TEFoodPlatter.cs:48-54`: the triple, behind the port's
            // presence byte.
            TeKind::ItemFrame | TeKind::WeaponsRack | TeKind::FoodPlatter => {
                if r.u8()? != 0 {
                    TeData::Item(Some(TeItem::read(r)?))
                } else {
                    TeData::Item(None)
                }
            }
            // `TEHatRack.cs:86-111`: the bitsByte, then the present triples.
            TeKind::HatRack => {
                let (items, dyes) = hat_rack_read(r)?;
                TeData::HatRack(items, dyes)
            }
            // `TETrainingDummy.cs:124-126`.
            TeKind::TrainingDummy => TeData::Dummy(r.i32()?),
            // `TELogicSensor.cs:389-396`.
            TeKind::LogicSensor => TeData::Logic {
                check: r.u8()?,
                on: r.boolean()?,
                counted: r.i32()?,
            },
            // `TELeashedEntityAnchorWithItem.cs:12-20` writes `short itemType`
            // — the kite (9) and critter (10) anchors carry it; the write arm
            // in `TeData::write` is the twin of this one.
            TeKind::KiteAnchor | TeKind::CritterAnchor => {
                TeData::Anchor(r.i16()? as i32)
            }
            // `TileEntity.cs:209-211`: DisplayDoll 3, TeleportationPylon 7 and
            // DeadCellsDisplayJar 8 write no extra data
            // (`TileEntitiesManager.cs:27-40`'s ids run 0..=10;
            // `TeKind::from_id` refuses anything past them).
            _ => TeData::None,
        })
    }
}

/// One placed entity (`TileEntity.ID/Position/type`, `TileEntity.cs:11-17`).
#[derive(Clone, Debug, PartialEq)]
pub struct TileEntity {
    pub id: u32,
    pub x: i16,
    pub y: i16,
    pub kind: TeKind,
    pub data: TeData,
}

impl TileEntity {
    /// `TileEntity.Write` + `WriteInner` (`TileEntity.cs:173-197`): the type
    /// byte, the StoredID, the position, then the entity's extra data. The
    /// save path takes `WriteInner`'s `!networkSend` half, so the id rides
    /// (the network send omits it, `:190-192`).
    pub fn write(&self, w: &mut Writer) {
        w.u8(self.kind as u8);
        w.u32(self.id);
        w.i16(self.x);
        w.i16(self.y);
        self.data.write(w);
    }

    /// `TileEntity.Read` + `ReadInner` (`TileEntity.cs:179-186`): the type
    /// byte comes first and selects the payload shape — the C# manager fails
    /// to `GenerateInstance` an unknown type, the port returns `None` so the
    /// caller bails on the corrupt record. `None` also covers a truncated
    /// stream.
    pub fn read(r: &mut Reader<'_>) -> Option<Self> {
        // `TileEntity.cs:181`'s `reader.ReadByte()` → `from_id`; the ids run
        // 0..=10 (`TileEntitiesManager.cs:27-40`).
        let kind = TeKind::from_id(r.u8()?)?;
        let id = r.u32()?;
        let x = r.i16()?;
        let y = r.i16()?;
        let data = TeData::read(r, kind)?;
        Some(Self { id, x, y, kind, data })
    }
}

/// The world's entity books — `TileEntity.ByID` / `ByPosition`
/// (`TileEntity.cs:15-16`).
#[derive(Clone, Debug, Default)]
pub struct TileEntities {
    by_id: Vec<TileEntity>,
    by_position: HashMap<(i16, i16), u32>,
    next_id: u32,
}

impl TileEntities {
    /// The placed entities in id order (the update walks).
    pub fn entities(&self) -> impl Iterator<Item = &TileEntity> {
        self.by_id.iter()
    }

    /// The mutable entity by its id (the state pokes).
    pub fn by_id_mut(&mut self, id: u32) -> Option<&mut TileEntity> {
        self.by_id.get_mut(id as usize)
    }

    pub fn get_by_id(&self, id: u32) -> Option<&TileEntity> {
        self.by_id.get(id as usize)
    }

    pub fn get_at(&self, x: i16, y: i16) -> Option<&TileEntity> {
        self.by_position
            .get(&(x, y))
            .map(|&id| &self.by_id[id as usize])
    }

    /// The mutable entity at a position (the dummy's npc poke).
    pub fn get_at_mut(&mut self, x: i16, y: i16) -> Option<&mut TileEntity> {
        let &id = self.by_position.get(&(x, y))?;
        self.by_id.get_mut(id as usize)
    }

    /// The books' insert without the validity gate — the fixtures and the
    /// worldgen paths that pre-validate.
    pub fn place_unchecked(&mut self, kind: TeKind, x: i16, y: i16) -> u32 {
        let data = match kind {
            TeKind::ItemFrame | TeKind::WeaponsRack | TeKind::FoodPlatter => TeData::Item(None),
            TeKind::HatRack => TeData::HatRack([None, None], [None, None]),
            TeKind::TrainingDummy => TeData::Dummy(-1),
            TeKind::LogicSensor => TeData::Logic {
                check: 0,
                on: false,
                counted: 0,
            },
            // `TELeashedEntityAnchorWithItem`'s ctor: no item held.
            TeKind::KiteAnchor | TeKind::CritterAnchor => TeData::Anchor(0),
            _ => TeData::None,
        };
        let id = self.next_id;
        self.next_id += 1;
        self.by_id.push(TileEntity {
            id,
            x,
            y,
            kind,
            data,
        });
        self.by_position.insert((x, y), id);
        id
    }

    /// The live entities in `TileEntity.ByID` order — the insertion order of
    /// the id book, exactly the order `SaveTileEntities:3306-3310`'s
    /// `foreach (KeyValuePair<int, TileEntity> item in TileEntity.ByID)`
    /// iterates. An id slot whose position book no longer points at it is a
    /// tombstone (`remove_at`), which the C# `Remove` also drops from `ByID`;
    /// the filter is what keeps the two books' membership identical, so the
    /// writer's count and record set never claim an entity that is not placed.
    pub fn live_entities(&self) -> impl Iterator<Item = &TileEntity> {
        self.by_id
            .iter()
            .filter(|e| matches!(self.by_position.get(&(e.x, e.y)), Some(&id) if id == e.id))
    }

    /// `TileEntity.ByID.Count` — the live entities the writer counts.
    pub fn next_id(&self) -> u32 {
        self.next_id
    }

    /// One record landing in the books — `LoadTileEntities:3322-3330`: the
    /// stored id is discarded, the record takes the next sequential slot, and
    /// an entity already at the position is removed first
    /// (`TryGetAt` → `TileEntity.Remove`), which leaves the replaced slot as a
    /// tombstone so the id book stays dense.
    pub fn load_entity(&mut self, kind: TeKind, x: i16, y: i16, data: TeData) -> u32 {
        if self.by_position.contains_key(&(x, y)) {
            self.remove_at(x, y);
        }
        let id = self.next_id;
        self.next_id += 1;
        self.by_id.push(TileEntity {
            id,
            x,
            y,
            kind,
            data,
        });
        self.by_position.insert((x, y), id);
        id
    }

    /// `TileEntity.OnWorldLoaded` (`TileEntity.cs:333`), walked over the
    /// survivors in `ByID` order (`LoadTileEntities:3355-3358`). The ported
    /// kinds have no load hook; the leashed anchors' `FitsItem` re-check
    /// (`TELeashedEntityAnchorWithItem.cs:38-43`) lands with M7-TE-2, so this
    /// is the seam that hook takes over — kept explicit so the load path reads
    /// as the C# does.
    pub fn on_world_loaded(_world: &mut World) {}

    /// `TileEntitiesManager.CheckValidTile` + `Place` (`:51-60`): the
    /// entity's `ValidTile` must pass and the position must be free.
    /// `kind` selects the entity; the returned id is the `ByID` slot.
    pub fn place(&mut self, world: &World, kind: TeKind, x: i16, y: i16) -> Option<u32> {
        if self.by_position.contains_key(&(x, y)) {
            return None;
        }
        if !kind.is_tile_valid(world, x as i32, y as i32) {
            return None;
        }
        let data = match kind {
            TeKind::ItemFrame | TeKind::WeaponsRack | TeKind::FoodPlatter => TeData::Item(None),
            TeKind::HatRack => TeData::HatRack([None, None], [None, None]),
            // `TETrainingDummy`'s ctor: `npc = -1`.
            TeKind::TrainingDummy => TeData::Dummy(-1),
            // `TELogicSensor`'s ctor: `logicCheck = None, On = false`.
            TeKind::LogicSensor => TeData::Logic {
                check: 0,
                on: false,
                counted: 0,
            },
            // `TELeashedEntityAnchorWithItem`'s ctor: no item held.
            TeKind::KiteAnchor | TeKind::CritterAnchor => TeData::Anchor(0),
            _ => TeData::None,
        };
        let id = self.next_id;
        self.next_id += 1;
        self.by_id.push(TileEntity {
            id,
            x,
            y,
            kind,
            data,
        });
        self.by_position.insert((x, y), id);
        Some(id)
    }

    /// Removing the anchor tile: the entity is dropped from both books and
    /// its payload returned for the item drop (`TEItemFrame.DropItem` etc.).
    pub fn remove_at(&mut self, x: i16, y: i16) -> Option<(TeKind, TeData)> {
        let id = self.by_position.remove(&(x, y))?;
        let entity = &self.by_id[id as usize];
        let out = (entity.kind, entity.data.clone());
        self.by_id[id as usize].data = TeData::None;
        Some(out)
    }

    /// The one-slot entities' item write (`TEItemFrame.item = …`).
    pub fn set_item(&mut self, id: u32, item: Option<TeItem>) -> bool {
        let Some(entity) = self.by_id.get_mut(id as usize) else {
            return false;
        };
        if !entity.kind.has_item_slot() {
            return false;
        }
        entity.data = TeData::Item(item);
        true
    }

    pub fn item(&self, id: u32) -> Option<&Option<TeItem>> {
        let entity = self.get_by_id(id)?;
        match &entity.data {
            TeData::Item(item) => Some(item),
            _ => None,
        }
    }

    /// `SaveTileEntities` (`WorldFile.cs:3302-3313`): the entity count, then a
    /// record per entity — the port's `by_id` is the `ByID` walk and already
    /// holds them in id order.
    ///
    /// `remove_at` leaves a cleared slot in `by_id` (the C# `ByID.Remove`
    /// deletes the entry outright), so the walk skips the slots `by_position`
    /// no longer maps to their id: otherwise a removal would come back from
    /// the save as a payload-less entity still holding the cell. The same
    /// filter drops the unreachable earlier half of a duplicated cell (see
    /// [`TileEntities::read_all`]).
    pub fn write_all(&self, w: &mut Writer) {
        let records: Vec<&TileEntity> = self
            .by_id
            .iter()
            .filter(|e| self.by_position.get(&(e.x, e.y)) == Some(&e.id))
            .collect();
        w.u32(records.len() as u32);
        for e in records {
            e.write(w);
        }
    }

    /// `LoadTileEntities` (`WorldFile.cs:3315-3357`): the count, a record per
    /// entity, then the re-validation pass — the C# drops the entities whose
    /// cell left the world (`WorldGen.InWorld(..., 1)`, `:3334`) or whose
    /// `CheckValidTile` fails (`:3338`), which the port reads as the bounds
    /// band plus [`TeKind::is_tile_valid`]. A truncated stream returns `None`
    /// so the caller bails instead of half-loading the books.
    ///
    /// Deviations from the C# load:
    /// - the C# reassigns the ids on load (`tileEntity.ID = num2++`, `:3323`)
    ///   and then sets `TileEntitiesNextID = num` (`:3330`); the port keeps
    ///   the **stored** id so a save round-trips them exactly, and `next_id`
    ///   runs to `max(id) + 1` (0 when the section is empty). The drops do
    ///   not rewind it — the C#'s next id is the read count, likewise
    ///   unmoved by the validation pass.
    /// - `by_id` is indexed by id (`get_by_id`/`by_id_mut` take it as the
    ///   index), so an id that ends up with no live entity keeps its slot
    ///   rather than shifting every later id down: a dropped record leaves
    ///   the same cleared shell [`TileEntities::remove_at`] leaves behind (the
    ///   payload emptied, the cell no longer mapped in `by_position`, so
    ///   `place`/`get_at` never see it), and an id with no record at all (a
    ///   gap in a hand-written file) leaves a void slot.
    /// - a duplicated cell: the C# removes the earlier entity from *both*
    ///   books before the add (`:3324-3327`), so only the last survives; the
    ///   port keeps the insert semantics of `place`/`place_unchecked` —
    ///   `by_position`'s insert overwrites, so the last record is the one the
    ///   position book reaches and the earlier one lingers in `by_id`
    ///   unreachable (and is skipped by the next `write_all`).
    pub fn read_all(r: &mut Reader<'_>, world: &World) -> Option<Self> {
        let count = r.u32()?;
        let mut records: Vec<TileEntity> = Vec::new();
        for _ in 0..count {
            // A short read anywhere in the section is a corrupt file.
            records.push(TileEntity::read(r)?);
        }
        // `write_all` emits the records in id order; a hand-written file need
        // not be, and the books are indexed by id — sort so the slots line
        // up (stable, so a duplicated cell keeps its file order).
        records.sort_by_key(|e| e.id);

        let mut out = TileEntities::default();
        let mut iter = records.into_iter().peekable();
        while let Some(mut rec) = iter.next() {
            let id = rec.id;
            // The books are a vector indexed by id, so an id band is only
            // sane up to the ceiling; the vanilla per-chunk cap
            // (`MaxEntitiesPerChunk = 1000`, `TileEntity.cs:14`) keeps real
            // ids far below it. A band past it is a corrupt file.
            if id as usize >= MAX_TILE_ENTITIES {
                return None;
            }
            // Ids with no record at all: the void slot that keeps `by_id`
            // index-aligned with the stored ids.
            while out.next_id < id {
                out.by_id.push(cleared_slot(out.next_id));
                out.next_id += 1;
            }
            // The re-validation pass (`WorldFile.cs:3332-3342`).
            let (x, y) = (rec.x as i32, rec.y as i32);
            if world.in_bounds(x, y) && rec.kind.is_tile_valid(world, x, y) {
                out.by_position.insert((rec.x, rec.y), id);
            } else {
                // Dropped: the id keeps its slot with the payload emptied —
                // `remove_at`'s shape.
                rec.data = TeData::None;
            }
            out.by_id.push(rec);
            out.next_id = id + 1;
            // `ByID` is keyed by id: a repeated id is corrupt, the first
            // record wins and the rest are dropped.
            while iter.peek().is_some_and(|e| e.id == id) {
                iter.next();
            }
        }
        Some(out)
    }
}

/// The ceiling on the books' slots — a guard so a corrupt id band cannot make
/// [`TileEntities::read_all`] synthesize slots without end (the C# keys
/// `ByID` by id in a dictionary and needs no such bound; the port's `by_id`
/// is a vector indexed by id). `MaxEntitiesPerChunk = 1000`
/// (`TileEntity.cs:14`) is the vanilla per-chunk hint — the largest vanilla
/// world (8400x2400 = 525 chunks) tops out near 525k ids, so this sits above
/// any real world's entity count.
const MAX_TILE_ENTITIES: usize = 1 << 20;

/// The cleared `ByID` slot an id with no live entity leaves behind — the shape
/// [`TileEntities::remove_at`] leaves: the id stays reserved, the payload is
/// empty and no cell is claimed, so the update walks (which match on the
/// `TeData` variants) and the position lookups skip it.
fn cleared_slot(id: u32) -> TileEntity {
    TileEntity {
        id,
        x: -1,
        y: -1,
        kind: TeKind::TeleportationPylon,
        data: TeData::None,
    }
}

/// `TELogicSensor.LogicCheckType` (`TELogicSensor.cs:9-18`): the enum values
/// ride the tile's `frameY / 18` (`FigureCheckType` `:168-200`).
pub const SENSOR_DAY: u8 = 0;
pub const SENSOR_NIGHT: u8 = 1;
pub const SENSOR_PLAYER_ABOVE: u8 = 2;
pub const SENSOR_WATER: u8 = 3;
pub const SENSOR_LAVA: u8 = 4;
pub const SENSOR_HONEY: u8 = 5;
pub const SENSOR_LIQUID: u8 = 6;
/// The liquid grace (`TELogicSensor.cs:255-262`): the sensor stays On for 15
/// ticks after its liquid disappears.
const SENSOR_LIQUID_GRACE: i32 = 15;

/// `TELogicSensor.GetState` (`:202-267`): the raw state for the sensor's
/// position and check type. `player_boxes` are the live players' hitboxes in
/// pixels `(x, y, w, h)`; the sensor's On state feeds the liquid grace.
fn sensor_state(
    world: &World,
    x: i32,
    y: i32,
    check: u8,
    on: bool,
    counted: &mut i32,
    player_boxes: &[(i32, i32, i32, i32)],
    day_time: bool,
) -> bool {
    match check {
        SENSOR_DAY => day_time,
        SENSOR_NIGHT => !day_time,
        SENSOR_PLAYER_ABOVE => {
            // `:219-232`: the box `(x*16 - 33, y*16 - 161, 82, 162)` vs the
            // player hitboxes.
            let (bx, by) = (x * 16 - 32 - 1, y * 16 - 160 - 1);
            let (bw, bh) = (82, 162);
            player_boxes.iter().any(|&(px, py, pw, ph)| {
                bx < px + pw && bx + bw > px && by < py + ph && by + bh > py
            })
        }
        SENSOR_WATER | SENSOR_LAVA | SENSOR_HONEY | SENSOR_LIQUID => {
            let mut flag = false;
            if let Some(t) = world.tile(x, y) {
                let lava = t.flags & crate::tile::FLAG_LIQUID_LAVA != 0;
                let honey = t.flags & crate::tile::FLAG_LIQUID_HONEY != 0;
                let shimmer = t.flags & crate::tile::FLAG_LIQUID_SHIMMER != 0;
                if t.liquid != 0 {
                    flag = true;
                }
                if !lava && check == SENSOR_LAVA {
                    flag = false;
                }
                if !honey && check == SENSOR_HONEY {
                    flag = false;
                }
                if (honey || lava || shimmer) && check == SENSOR_WATER {
                    flag = false;
                }
            }
            // `:253-262`: the grace keeps the On state for 15 ticks after
            // the liquid disappears.
            if !flag && on {
                if *counted == 0 {
                    *counted = SENSOR_LIQUID_GRACE;
                } else if *counted > 0 {
                    *counted -= 1;
                }
                flag = *counted > 0;
            }
            flag
        }
        _ => false,
    }
}

/// `TELogicSensor.Update` + `UpdateEndInternal` (`TELogicSensor.cs:103-152`):
/// the per-sensor state scan — the Day/Night kinds trip the wire on the
/// rising edge only, the rest on every change — then the trips fire through
/// `HitSwitch` in one pass, with the PlayerAbove trips setting
/// `blockPlayerTeleportationForOneIteration` for their hit. `player_boxes`
/// are the live players' pixel hitboxes; `day_time_in` is `Main.dayTime`.
pub fn update_logic_sensors(
    world: &mut World,
    wiring: &mut crate::wiring::Wiring,
    day_time_in: bool,
    player_boxes: &[(i32, i32, i32, i32)],
) {
    let mut trips: Vec<((i16, i16), bool)> = Vec::new();
    let entities: Vec<(i16, i16, u8, bool, i32, u32)> = {
        let mut out = Vec::new();
        for e in world.tile_entities.entities() {
            if let TeData::Logic { check, on, counted } = &e.data {
                out.push((e.x, e.y, *check, *on, *counted, e.id));
            }
        }
        out
    };
    for (x, y, check, on, counted, id) in entities {
        let mut counted = counted;
        let state = sensor_state(
            world,
            x as i32,
            y as i32,
            check,
            on,
            &mut counted,
            player_boxes,
            day_time_in,
        );
        let trip = match check {
            SENSOR_DAY | SENSOR_NIGHT => !on && state,
            _ => on != state,
        };
        // The grace counter rides every scan (`CountedData` lives on the
        // instance, not on the change); the state change rewrites the LED.
        if on != state {
            // `ChangeState` (`:122-137`): the LED rides `frameX` 0/18.
            if let Some(t) = world.tile_mut(x as i32, y as i32) {
                t.frame_x = if state { 18 } else { 0 };
            }
        }
        if let Some(e) = world.tile_entities.by_id_mut(id) {
            if let TeData::Logic {
                on: o, counted: c, ..
            } = &mut e.data
            {
                *o = state;
                *c = counted;
            }
        }
        if trip {
            trips.push(((x, y), check == SENSOR_PLAYER_ABOVE));
        }
    }
    // `UpdateEndInternal` (`:135-152`): the trips fire after the scan; the
    // PlayerAbove trips block the teleporter iteration for their hit.
    for ((tx, ty), player_above) in trips {
        if player_above {
            wiring.set_block_player_teleportation(true);
        }
        wiring.hit_switch(world, tx as i32, ty as i32);
        wiring.set_block_player_teleportation(false);
    }
}

/// `OnPlaced` (`TELogicSensor.OnPlaced` → `FigureCheckState`): the check
/// type rides the tile's `frameY / 18`, the LED starts at the current state —
/// no wire trip on placement. `player_boxes`/`day_time` feed the initial
/// `GetState`.
pub fn figure_sensor_state(
    tes: &mut TileEntities,
    world: &World,
    x: i16,
    y: i16,
    player_boxes: &[(i32, i32, i32, i32)],
    day_time: bool,
) -> bool {
    let Some(t) = world.tile(x as i32, y as i32) else {
        return false;
    };
    let check = (t.frame_y as i32 / 18).clamp(0, 6) as u8;
    let mut counted = 0;
    let state = sensor_state(
        world,
        x as i32,
        y as i32,
        check,
        false,
        &mut counted,
        player_boxes,
        day_time,
    );
    let Some(e) = tes.get_at_mut(x, y) else {
        return false;
    };
    if e.kind != TeKind::LogicSensor {
        return false;
    }
    e.data = TeData::Logic {
        check,
        on: state,
        counted: 0,
    };
    state
}

/// `TETrainingDummy.Update` (`TETrainingDummy.cs:43-75`): the tied NPC is
/// verified by `npc_tied` (the app knows its entity list); the free dummy
/// spawns when a player's hitbox intersects the 32×48 box inflated by 1600
/// px. Returns the spawn anchors in pixels `(x*16 + 16, y*16 + 48)`; the app
/// spawns the NPC and pokes the id back with `set_dummy_npc`.
pub fn update_training_dummies(
    world: &mut World,
    player_boxes: &[(i32, i32, i32, i32)],
    npc_tied: impl Fn(i32) -> bool,
) -> Vec<(i32, i32)> {
    let mut spawns = Vec::new();
    let entities: Vec<(i16, i16, i32)> = {
        let mut out = Vec::new();
        for e in world.tile_entities.entities() {
            if let TeData::Dummy(npc) = e.data {
                out.push((e.x, e.y, npc));
            }
        }
        out
    };
    for (x, y, npc) in entities {
        if npc != -1 {
            // `:45-53`: the lost tie deactivates.
            if !npc_tied(npc) {
                if let Some(e) = world.tile_entities.get_at_mut(x, y) {
                    e.data = TeData::Dummy(-1);
                }
            }
            continue;
        }
        // `:56-74`: the spawn scan — the 32×48 box inflated by 1600 px.
        let (bx, by) = (x as i32 * 16 - 1600, y as i32 * 16 - 1600);
        let (bw, bh) = (32 + 3200, 48 + 3200);
        let near = player_boxes
            .iter()
            .any(|&(px, py, pw, ph)| bx < px + pw && bx + bw > px && by < py + ph && by + bh > py);
        if near {
            spawns.push((x as i32 * 16 + 16, y as i32 * 16 + 48));
        }
    }
    spawns
}

/// Pokes the tied NPC id back into the dummy entity (`Activate`'s
/// `this.npc = num`).
pub fn set_dummy_npc(world: &mut World, x: i16, y: i16, npc: i32) -> bool {
    let Some(e) = world.tile_entities.get_at_mut(x, y) else {
        return false;
    };
    if e.kind != TeKind::TrainingDummy {
        return false;
    }
    e.data = TeData::Dummy(npc);
    true
}

/// `WriteExtraData`/`ReadExtraData` round-trip for the HatRack payload:
/// a bitsByte of the four slot-presence flags, then each present slot as the
/// item triple (`TEHatRack.cs:59-80`).
pub fn hat_rack_write(items: &[Option<TeItem>; 2], dyes: &[Option<TeItem>; 2], w: &mut Writer) {
    let mut bits = 0u8;
    for (bit, item) in items.iter().chain(dyes.iter()).enumerate() {
        if item.is_some() {
            bits |= 1 << bit;
        }
    }
    w.u8(bits);
    for item in items.iter().chain(dyes.iter()).flatten() {
        item.write(w);
    }
}

pub fn hat_rack_read(r: &mut Reader<'_>) -> Option<([Option<TeItem>; 2], [Option<TeItem>; 2])> {
    let bits = r.u8()?;
    let mut slots: [Option<TeItem>; 4] = [None, None, None, None];
    for (bit, slot) in slots.iter_mut().enumerate() {
        if bits & (1 << bit) != 0 {
            *slot = Some(TeItem::read(r)?);
        }
    }
    Some((
        [slots[0].take(), slots[1].take()],
        [slots[2].take(), slots[3].take()],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::{Tile, FLAG_ACTIVE};

    fn world_with_frame() -> World {
        let mut w = World::new("t".into(), 32, 32);
        // Item frame tile 395, 2×2: the anchor at (10,10) with frameX 0,
        // frameY 0; the rest of the frame at frameX 36/72... only the anchor
        // check runs here.
        let i = w.index(10, 10);
        w.tiles[i] = Tile {
            kind: 395,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w
    }

    #[test]
    fn registration_order_matches_the_manager() {
        // `TileEntitiesManager.RegisterAll` (`:27-40`): the ids are the
        // registration order.
        assert_eq!(TeKind::TrainingDummy as u8, 0);
        assert_eq!(TeKind::ItemFrame as u8, 1);
        assert_eq!(TeKind::LogicSensor as u8, 2);
        assert_eq!(TeKind::DisplayDoll as u8, 3);
        assert_eq!(TeKind::WeaponsRack as u8, 4);
        assert_eq!(TeKind::HatRack as u8, 5);
        assert_eq!(TeKind::FoodPlatter as u8, 6);
        assert_eq!(TeKind::TeleportationPylon as u8, 7);
        assert_eq!(TeKind::DeadCellsDisplayJar as u8, 8);
        assert_eq!(TeKind::KiteAnchor as u8, 9);
        assert_eq!(TeKind::CritterAnchor as u8, 10);
        assert_eq!(TeKind::from_id(10), Some(TeKind::CritterAnchor));
        assert_eq!(TeKind::from_id(11), None);
    }

    #[test]
    fn item_frame_validity_checks_the_frames() {
        let mut w = world_with_frame();
        let tes = TileEntities::default();
        assert!(
            TeKind::ItemFrame.is_tile_valid(&w, 10, 10) || {
                TeKind::ItemFrame.is_tile_valid(&w, 10, 10)
            }
        );
        // frameX 36 (the second frame column) is valid too.
        w.tile_mut(10, 10).unwrap().frame_x = 36;
        assert!(TeKind::ItemFrame.is_tile_valid(&w, 10, 10));
        // frameX 18 is a non-anchor frame → invalid.
        w.tile_mut(10, 10).unwrap().frame_x = 18;
        assert!(!TeKind::ItemFrame.is_tile_valid(&w, 10, 10));
        w.tile_mut(10, 10).unwrap().frame_x = 0;
        // frameY 18 → invalid.
        w.tile_mut(10, 10).unwrap().frame_y = 18;
        assert!(!TeKind::ItemFrame.is_tile_valid(&w, 10, 10));
    }

    /// A world whose anchor cell (10,10) holds `kind` at the given frames.
    fn world_with_anchor(kind: u16, frame_x: i16, frame_y: i16) -> World {
        let mut w = World::new("t".into(), 32, 32);
        let i = w.index(10, 10);
        w.tiles[i] = Tile {
            kind,
            frame_x,
            frame_y,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w
    }

    #[test]
    fn the_critter_anchor_requires_the_lady_bug_tile() {
        // F1 (Phase-5 review N1): `TECritterAnchor.IsTileValidForEntity`
        // (`TECritterAnchor.cs:24-32`) returns `tile.type == 724` — an
        // anchor on any other active tile is dropped by
        // `LoadTileEntities`'s removal pass, so it must not survive here.
        let on_anchor = world_with_anchor(724, 0, 0);
        assert!(TeKind::CritterAnchor.is_tile_valid(&on_anchor, 10, 10));
        let on_dirt = world_with_anchor(0, 0, 0);
        assert!(!TeKind::CritterAnchor.is_tile_valid(&on_dirt, 10, 10));
        let on_item_frame = world_with_anchor(395, 0, 0);
        assert!(!TeKind::CritterAnchor.is_tile_valid(&on_item_frame, 10, 10));
        // The kind alone decides — frames are not part of the check.
        let off_frame = world_with_anchor(724, 18, 18);
        assert!(TeKind::CritterAnchor.is_tile_valid(&off_frame, 10, 10));
        // An inactive cell has no anchor (`tile.active()`'s head check).
        let mut cleared = world_with_anchor(724, 0, 0);
        cleared.tile_mut(10, 10).unwrap().flags &= !FLAG_ACTIVE;
        assert!(!TeKind::CritterAnchor.is_tile_valid(&cleared, 10, 10));
        // The tile id the port now reports is the C# anchor tile.
        assert_eq!(TeKind::CritterAnchor.tile(), 724);
        // The kite anchor keeps its own kind (723) and is likewise kind-only.
        let on_kite = world_with_anchor(723, 18, 0);
        assert!(TeKind::KiteAnchor.is_tile_valid(&on_kite, 10, 10));
        assert!(!TeKind::KiteAnchor.is_tile_valid(&on_kite, 11, 10));
    }

    #[test]
    fn the_display_doll_validity_checks_the_frames() {
        // The same check the load path runs, asserted directly: the doll
        // needs the 470 anchor at frameY 0 / frameX % 36 == 0
        // (`TEDisplayDoll.cs:851-858`).
        let mut w = world_with_anchor(470, 0, 0);
        assert!(TeKind::DisplayDoll.is_tile_valid(&w, 10, 10));
        w.tile_mut(10, 10).unwrap().frame_x = 18;
        assert!(!TeKind::DisplayDoll.is_tile_valid(&w, 10, 10));
        w.tile_mut(10, 10).unwrap().frame_x = 36;
        assert!(TeKind::DisplayDoll.is_tile_valid(&w, 10, 10));
        w.tile_mut(10, 10).unwrap().frame_y = 36;
        assert!(!TeKind::DisplayDoll.is_tile_valid(&w, 10, 10));
    }

    #[test]
    fn place_and_remove_round_trip_the_books() {
        let w = world_with_frame();
        let mut tes = TileEntities::default();
        let id = tes.place(&w, TeKind::ItemFrame, 10, 10).unwrap();
        assert_eq!(id, 0);
        assert!(tes.get_at(10, 10).is_some());
        assert_eq!(tes.get_at(10, 11), None);
        // The same position is rejected (ByPosition).
        assert!(tes.place(&w, TeKind::ItemFrame, 10, 10).is_none());
        // The invalid frame is rejected (ValidTile).
        assert!(tes.place(&w, TeKind::ItemFrame, 12, 12).is_none());
        // Removal returns the payload and clears both books.
        assert!(tes.set_item(
            id,
            Some(TeItem {
                kind: 50,
                prefix: 0,
                stack: 1
            })
        ));
        let (kind, data) = tes.remove_at(10, 10).unwrap();
        assert_eq!(kind, TeKind::ItemFrame);
        assert_eq!(
            data,
            TeData::Item(Some(TeItem {
                kind: 50,
                prefix: 0,
                stack: 1
            }))
        );
        assert!(tes.get_at(10, 10).is_none());
    }

    #[test]
    fn weapon_rack_and_food_platter_frames() {
        let mut w = World::new("t".into(), 32, 32);
        let i = w.index(5, 5);
        w.tiles[i] = Tile {
            kind: 471,
            frame_x: 54,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        assert!(TeKind::WeaponsRack.is_tile_valid(&w, 5, 5));
        w.tiles[i].frame_x = 18;
        assert!(!TeKind::WeaponsRack.is_tile_valid(&w, 5, 5));

        let j = w.index(8, 8);
        w.tiles[j] = Tile {
            kind: 520,
            ..Default::default()
        };
        w.tiles[j].flags |= FLAG_ACTIVE;
        assert!(TeKind::FoodPlatter.is_tile_valid(&w, 8, 8));
        w.tiles[j].frame_y = 18;
        assert!(!TeKind::FoodPlatter.is_tile_valid(&w, 8, 8));
    }

    #[test]
    fn the_item_triple_round_trips_the_c_sharp_layout() {
        let item = TeItem {
            kind: 75,
            prefix: 42,
            stack: 250,
        };
        let mut w = Writer::new();
        item.write(&mut w);
        let buf = w.into_inner();
        let mut r = Reader::new(&buf);
        assert_eq!(TeItem::read(&mut r), Some(item));
        // 2 + 1 + 2 = 5 bytes (`TEItemFrame.WriteExtraData`).
        assert_eq!(buf.len(), 5);
    }

    #[test]
    fn the_hat_rack_presence_bits_round_trip() {
        let items = [
            Some(TeItem {
                kind: 100,
                prefix: 1,
                stack: 1,
            }),
            None,
        ];
        let dyes = [
            None,
            Some(TeItem {
                kind: 3454,
                prefix: 0,
                stack: 1,
            }),
        ];
        let mut w = Writer::new();
        hat_rack_write(&items, &dyes, &mut w);
        let buf = w.into_inner();
        let mut r = Reader::new(&buf);
        let (items2, dyes2) = hat_rack_read(&mut r).unwrap();
        assert_eq!(items, items2);
        assert_eq!(dyes, dyes2);
        // bitsByte + 5 + 5 bytes.
        assert_eq!(buf.len(), 11);
    }

    /// One valid anchor tile per kind the save layout covers, each entity
    /// placed through the gated `place` and given its payload.
    fn world_with_entities() -> World {
        let mut w = World::new("t".into(), 64, 64);
        // (x, y, tile kind, frameX, frameY, entity kind) — every frame pair
        // passes its `ValidTile` check (`TEItemFrame.cs:39-42`,
        // `TEHatRack.cs:409-410`, `TETrainingDummy.cs:95`,
        // `TELogicSensor.cs:158-160`, `TETeleportationPylon.cs:135-139`).
        let anchors = [
            (5i16, 5i16, 378u16, 0i16, 0i16, TeKind::TrainingDummy),
            (10, 10, 395, 0, 0, TeKind::ItemFrame),
            (15, 15, 423, 0, 18, TeKind::LogicSensor),
            (20, 20, 597, 54, 0, TeKind::TeleportationPylon),
            (25, 25, 475, 54, 0, TeKind::HatRack),
        ];
        for &(x, y, kind, frame_x, frame_y, _) in &anchors {
            let i = w.index(x as i32, y as i32);
            w.tiles[i] = Tile {
                kind,
                frame_x,
                frame_y,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        // `place` reads the world and the books at once — take them apart the
        // way `world_with_sensor` does.
        let mut tes = std::mem::take(&mut w.tile_entities);
        for &(x, y, _, _, _, te) in &anchors {
            assert!(tes.place(&w, te, x, y).is_some(), "{te:?} must place");
        }
        w.tile_entities = tes;
        // The payloads: the dummy's NPC tie, the frame's item, the sensor's
        // check/LED/grace, the pylon's nothing, the hat rack's slots.
        assert!(set_dummy_npc(&mut w, 5, 5, 7));
        let frame = w.tile_entities.get_at(10, 10).unwrap().id;
        assert!(w.tile_entities.set_item(frame, Some(TeItem { kind: 42, prefix: 3, stack: 7 })));
        w.tile_entities.get_at_mut(15, 15).unwrap().data = TeData::Logic {
            check: SENSOR_NIGHT,
            on: true,
            counted: 3,
        };
        w.tile_entities.get_at_mut(25, 25).unwrap().data = TeData::HatRack(
            [Some(TeItem { kind: 100, prefix: 1, stack: 1 }), None],
            [None, Some(TeItem { kind: 3454, prefix: 0, stack: 1 })],
        );
        w
    }

    #[test]
    fn the_books_round_trip_the_save_layout() {
        let w = world_with_entities();
        let mut buf = Writer::new();
        w.tile_entities.write_all(&mut buf);
        let bytes = buf.into_inner();
        let mut r = Reader::new(&bytes);
        let loaded = TileEntities::read_all(&mut r, &w).expect("the records parse");
        assert_eq!(r.pos(), bytes.len(), "every byte is consumed");

        // ids, positions, kinds and payloads survive exactly.
        let before: Vec<&TileEntity> = w.tile_entities.entities().collect();
        let after: Vec<&TileEntity> = loaded.entities().collect();
        assert_eq!(before.len(), after.len());
        for (a, b) in before.iter().zip(&after) {
            assert_eq!((a.id, a.x, a.y, a.kind), (b.id, b.x, b.y, b.kind));
            assert_eq!(a.data, b.data);
            // both books are rebuilt: the stored id and the cell.
            assert_eq!(loaded.get_at(b.x, b.y).map(|e| e.id), Some(b.id));
            assert_eq!(loaded.get_by_id(b.id).map(|e| (e.x, e.y)), Some((b.x, b.y)));
        }
        assert_eq!(loaded.next_id, w.tile_entities.next_id);

        // The payloads, per kind.
        assert_eq!(loaded.get_by_id(0).unwrap().data, TeData::Dummy(7));
        assert_eq!(
            loaded.get_by_id(1).unwrap().data,
            TeData::Item(Some(TeItem { kind: 42, prefix: 3, stack: 7 }))
        );
        assert_eq!(
            loaded.get_by_id(2).unwrap().data,
            TeData::Logic { check: SENSOR_NIGHT, on: true, counted: 3 }
        );
        assert_eq!(loaded.get_by_id(3).unwrap().data, TeData::None);
        assert_eq!(
            loaded.get_by_id(4).unwrap().data,
            TeData::HatRack(
                [Some(TeItem { kind: 100, prefix: 1, stack: 1 }), None],
                [None, Some(TeItem { kind: 3454, prefix: 0, stack: 1 })]
            )
        );
    }

    #[test]
    fn write_all_is_deterministic_and_pins_the_layout() {
        let w = world_with_entities();
        let mut a = Writer::new();
        w.tile_entities.write_all(&mut a);
        let mut b = Writer::new();
        w.tile_entities.write_all(&mut b);
        let (a, b) = (a.into_inner(), b.into_inner());
        assert_eq!(a, b);

        // `SaveTileEntities` (`WorldFile.cs:3302-3313`): u32 count, then a
        // record per entity — u8 kind, u32 id, i16 x, i16 y (9 bytes) plus
        // the payload: dummy 4, item frame 1 + 5, sensor 1 + 1 + 4, pylon 0,
        // hat rack 1 + 5 + 5.
        assert_eq!(a.len(), 4 + (9 + 4) + (9 + 6) + (9 + 6) + 9 + (9 + 11));
        // The first record: the dummy — kind 0, id 0, (5, 5), npc 7.
        assert_eq!(&a[0..4], &5u32.to_le_bytes());
        assert_eq!(a[4], TeKind::TrainingDummy as u8);
        assert_eq!(&a[5..9], &0u32.to_le_bytes());
        assert_eq!(&a[9..11], &5i16.to_le_bytes());
        assert_eq!(&a[11..13], &5i16.to_le_bytes());
        assert_eq!(&a[13..17], &7i32.to_le_bytes());
    }

    #[test]
    fn the_load_drops_the_records_on_invalid_cells() {
        // `LoadTileEntities`' re-validation pass (`WorldFile.cs:3332-3342`):
        // a cell outside the world, a bare cell and a frame that fails its
        // `ValidTile` check are all dropped.
        let mut world = World::new("t".into(), 32, 32);
        let i = world.index(4, 4);
        world.tiles[i] = Tile { kind: 395, ..Default::default() };
        world.tiles[i].flags |= FLAG_ACTIVE;
        let j = world.index(12, 12);
        world.tiles[j] = Tile { kind: 395, frame_x: 18, ..Default::default() };
        world.tiles[j].flags |= FLAG_ACTIVE;
        assert!(!TeKind::ItemFrame.is_tile_valid(&world, 12, 12));

        let good = TileEntity {
            id: 0,
            x: 4,
            y: 4,
            kind: TeKind::ItemFrame,
            data: TeData::Item(Some(TeItem { kind: 50, prefix: 1, stack: 3 })),
        };
        let outside = TileEntity {
            id: 1,
            x: 900,
            y: 900,
            kind: TeKind::ItemFrame,
            data: TeData::Item(None),
        };
        let bare = TileEntity {
            id: 2,
            x: 8,
            y: 8,
            kind: TeKind::ItemFrame,
            data: TeData::Item(None),
        };
        let wrong_frame = TileEntity {
            id: 3,
            x: 12,
            y: 12,
            kind: TeKind::ItemFrame,
            data: TeData::Item(None),
        };
        let mut buf = Writer::new();
        buf.u32(4);
        for rec in [&good, &outside, &bare, &wrong_frame] {
            rec.write(&mut buf);
        }
        let bytes = buf.into_inner();

        let mut r = Reader::new(&bytes);
        let mut loaded = TileEntities::read_all(&mut r, &world).expect("the records parse");
        // The valid frame survives with its payload and its stored id.
        let frame = loaded.get_at(4, 4).expect("the valid frame stays");
        assert_eq!(frame.id, 0);
        assert_eq!(frame.data, TeData::Item(Some(TeItem { kind: 50, prefix: 1, stack: 3 })));
        // The dropped cells are free — no entity reaches them.
        assert!(loaded.get_at(900, 900).is_none());
        assert!(loaded.get_at(8, 8).is_none());
        assert!(loaded.get_at(12, 12).is_none());
        // Exactly one entity is reachable through the position book.
        let reachable = loaded
            .entities()
            .filter(|e| loaded.get_at(e.x, e.y) == Some(e))
            .count();
        assert_eq!(reachable, 1);
        // Their ids keep the cleared slot `remove_at` leaves (empty payload,
        // unclaimed cell), which is what keeps `by_id` index-aligned.
        for id in 1..4 {
            assert_eq!(loaded.get_by_id(id).map(|e| e.data.clone()), Some(TeData::None));
        }
        // `next_id` runs past the highest stored id, drops included.
        assert_eq!(loaded.next_id, 4);
        // The books stay aligned: the next placement lands on its own id.
        let k = world.index(6, 6);
        world.tiles[k] = Tile { kind: 395, ..Default::default() };
        world.tiles[k].flags |= FLAG_ACTIVE;
        let id = loaded.place(&world, TeKind::ItemFrame, 6, 6).unwrap();
        assert_eq!(id, 4);
        assert!(loaded.get_at(6, 6).is_some());
        assert_eq!(loaded.get_by_id(id).map(|e| (e.x, e.y)), Some((6, 6)));
    }

    #[test]
    fn a_record_with_an_unknown_kind_byte_is_refused() {
        // `TileEntity.Read`'s `manager.GenerateInstance(id)` fails on a type
        // the manager does not know (`TileEntity.cs:181-182`); the port's
        // `TeKind::from_id` stops at CritterAnchor 10
        // (`TileEntitiesManager.cs:27-40`).
        let w = World::new("t".into(), 32, 32);
        let mut buf = Writer::new();
        buf.u32(1);
        buf.u8(11); // no such entity kind
        buf.u32(0);
        buf.i16(1);
        buf.i16(1);
        let bytes = buf.into_inner();

        let mut r = Reader::new(&bytes);
        assert!(TileEntities::read_all(&mut r, &w).is_none());
        let mut rec = Reader::new(&bytes[4..]);
        assert!(TileEntity::read(&mut rec).is_none());
        // The valid kinds still map: the first and the last.
        assert_eq!(TeKind::from_id(0), Some(TeKind::TrainingDummy));
        assert_eq!(TeKind::from_id(10), Some(TeKind::CritterAnchor));
        assert_eq!(TeKind::from_id(11), None);

        // A truncated record stream bails instead of half-loading: one byte
        // short of the count, and one byte short of a full record.
        let mut short = bytes.clone();
        short.truncate(3);
        assert!(TileEntities::read_all(&mut Reader::new(&short), &w).is_none());
        let mut full = Writer::new();
        full.u32(1);
        good_dummy_record().write(&mut full);
        let mut full = full.into_inner();
        full.truncate(full.len() - 1);
        assert!(TileEntities::read_all(&mut Reader::new(&full), &w).is_none());
    }

    /// A well-formed dummy record (kind 0, id 0, (1, 1), npc -1) for the
    /// truncation checks.
    fn good_dummy_record() -> TileEntity {
        TileEntity {
            id: 0,
            x: 1,
            y: 1,
            kind: TeKind::TrainingDummy,
            data: TeData::Dummy(-1),
        }
    }

    #[test]
    fn a_duplicated_cell_keeps_the_port_insert_semantics() {
        // The C# load removes the earlier entity from both books before the
        // add (`WorldFile.cs:3324-3327`); the port keeps `place`'s insert
        // semantics — `by_position`'s insert overwrites, so the last record
        // is the one the position book reaches and the earlier one lingers in
        // `by_id` unreachable.
        let w = world_with_entities();
        let mut buf = Writer::new();
        buf.u32(w.tile_entities.entities().count() as u32 + 1);
        for e in w.tile_entities.entities() {
            e.write(&mut buf);
        }
        let dupe = TileEntity {
            id: 5,
            x: 10,
            y: 10,
            kind: TeKind::ItemFrame,
            data: TeData::Item(Some(TeItem { kind: 9, prefix: 0, stack: 1 })),
        };
        dupe.write(&mut buf);
        let bytes = buf.into_inner();

        let mut r = Reader::new(&bytes);
        let loaded = TileEntities::read_all(&mut r, &w).expect("the records parse");
        // The last record owns the cell.
        assert_eq!(loaded.get_at(10, 10).map(|e| e.id), Some(5));
        assert_eq!(
            loaded.get_at(10, 10).unwrap().data,
            TeData::Item(Some(TeItem { kind: 9, prefix: 0, stack: 1 }))
        );
        // The earlier one stays in `by_id`, unreachable through the position.
        assert_eq!(loaded.get_by_id(1).map(|e| (e.x, e.y)), Some((10, 10)));
        assert_eq!(loaded.get_by_id(5).map(|e| (e.x, e.y)), Some((10, 10)));
        assert_eq!(loaded.next_id, 6);
        // A re-save drops the unreachable half (write_all's filter).
        let mut again = Writer::new();
        loaded.write_all(&mut again);
        let resaved = again.into_inner();
        let mut r2 = Reader::new(&resaved);
        let reloaded = TileEntities::read_all(&mut r2, &w).expect("the records parse");
        assert_eq!(reloaded.get_at(10, 10).map(|e| e.id), Some(5));
        assert_eq!(reloaded.next_id, 6);
    }
}

#[cfg(test)]
mod behaviour_tests {
    //! The logic sensors (`TELogicSensor.cs`) and the training dummies
    //! (`TETrainingDummy.cs`) — the M7-TE-2 halves.

    use super::*;
    use crate::tile::{Tile, FLAG_ACTIVE, FLAG_LIQUID_LAVA};
    use crate::wiring::WireColor;

    fn world_with_sensor(check: u8) -> World {
        let mut w = World::new("t".into(), 64, 64);
        // The sensor tile 423 at (10, 20): the check rides `frameY / 18`,
        // the LED rides `frameX` (0 off / 18 on).
        let i = w.index(10, 20);
        w.tiles[i] = Tile {
            kind: 423,
            frame_y: (check as i16) * 18,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let valid = TeKind::LogicSensor.is_tile_valid(&w, 10, 20);
        assert!(valid);
        w.tile_entities.place_unchecked(TeKind::LogicSensor, 10, 20);
        let mut tes = std::mem::take(&mut w.tile_entities);
        let initial = figure_sensor_state(&mut tes, &w, 10, 20, &[], true);
        w.tile_entities = tes;
        assert_eq!(
            initial,
            check == SENSOR_DAY,
            "only the day sensor starts On"
        );
        w
    }

    #[test]
    fn the_sensor_kinds_and_tiles_are_registered() {
        // `TETrainingDummy.cs:95` (tile 378, `frameY == 0 && frameX % 36 == 0`),
        // `TELogicSensor.cs:158-160` (tile 423, the 18 px frames).
        assert_eq!(TeKind::TrainingDummy.tile(), 378);
        assert_eq!(TeKind::LogicSensor.tile(), 423);
        assert_eq!(TeKind::DisplayDoll.tile(), 470);
        assert_eq!(TeKind::TeleportationPylon.tile(), 597);
        assert_eq!(TeKind::DeadCellsDisplayJar.tile(), 698);
        assert_eq!(TeKind::KiteAnchor.tile(), 723);
        let mut w = World::new("t".into(), 32, 32);
        let i = w.index(8, 8);
        w.tiles[i] = Tile {
            kind: 378,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        assert!(TeKind::TrainingDummy.is_tile_valid(&w, 8, 8));
        w.tiles[i].frame_x = 18;
        assert!(!TeKind::TrainingDummy.is_tile_valid(&w, 8, 8));
    }

    #[test]
    fn the_day_sensor_trips_once_on_the_rising_edge() {
        // `:109-118`: the Day kind trips the wire when the day breaks
        // (`!On && state`), and only clears the LED on the dusk.
        let mut w = world_with_sensor(SENSOR_DAY);
        let mut wiring = crate::wiring::Wiring::new();
        // A lit lamp on the sensor's wire so the trip is observable.
        let i = w.index(10, 22);
        w.tiles[i] = Tile {
            kind: 422,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.wires.set(10, 22, WireColor::Red, true);
        w.wires.set(10, 21, WireColor::Red, true);
        w.wires.set(10, 20, WireColor::Red, true);
        // Day: the state is already On — no trip, but the LED settles.
        update_logic_sensors(&mut w, &mut wiring, true, &[]);
        assert!(matches!(
            w.tile_entities.get_at(10, 20).unwrap().data,
            TeData::Logic { on: true, .. }
        ));
        // Still day: no re-trip.
        update_logic_sensors(&mut w, &mut wiring, true, &[]);
        // Night: the LED clears without a trip.
        update_logic_sensors(&mut w, &mut wiring, false, &[]);
        assert!(matches!(
            w.tile_entities.get_at(10, 20).unwrap().data,
            TeData::Logic { on: false, .. }
        ));
        assert_eq!(w.tile(10, 22).unwrap().kind, 422, "no trip happened");
        // The dawn trips the wire: the lamp flips.
        update_logic_sensors(&mut w, &mut wiring, true, &[]);
        assert_eq!(
            w.tile(10, 22).unwrap().kind,
            421,
            "the dawn tripped the wire"
        );
    }

    #[test]
    fn the_liquid_sensor_keeps_the_grace() {
        // `:253-262`: the water sensor stays On for 15 ticks after the tile
        // dries.
        let mut w = world_with_sensor(SENSOR_WATER);
        let mut wiring = crate::wiring::Wiring::new();
        let i = w.index(10, 20);
        w.tiles[i].liquid = 200;
        update_logic_sensors(&mut w, &mut wiring, true, &[]);
        assert!(matches!(
            w.tile_entities.get_at(10, 20).unwrap().data,
            TeData::Logic { on: true, .. }
        ));
        // The tile dries: the grace holds the On state for 15 scans (the
        // first sets `CountedData = 15`, each of the next 14 decrements it).
        w.tiles[i].liquid = 0;
        for _ in 0..15 {
            update_logic_sensors(&mut w, &mut wiring, true, &[]);
            assert!(matches!(
                w.tile_entities.get_at(10, 20).unwrap().data,
                TeData::Logic { on: true, .. }
            ));
        }
        update_logic_sensors(&mut w, &mut wiring, true, &[]);
        assert!(
            matches!(
                w.tile_entities.get_at(10, 20).unwrap().data,
                TeData::Logic { on: false, .. }
            ),
            "the grace expired"
        );
    }

    #[test]
    fn the_lava_sensor_ignores_water() {
        let mut w = world_with_sensor(SENSOR_LAVA);
        let mut wiring = crate::wiring::Wiring::new();
        let i = w.index(10, 20);
        w.tiles[i].liquid = 200; // plain water — no lava flag
        update_logic_sensors(&mut w, &mut wiring, true, &[]);
        assert!(matches!(
            w.tile_entities.get_at(10, 20).unwrap().data,
            TeData::Logic { on: false, .. }
        ));
        // Lava arrives: the sensor trips.
        w.tiles[i].flags |= FLAG_LIQUID_LAVA;
        update_logic_sensors(&mut w, &mut wiring, true, &[]);
        assert!(matches!(
            w.tile_entities.get_at(10, 20).unwrap().data,
            TeData::Logic { on: true, .. }
        ));
    }

    #[test]
    fn the_player_above_sensor_scans_the_box() {
        // `:219-232`: the 82×162 box above the sensor vs the player
        // hitboxes.
        let mut w = world_with_sensor(SENSOR_PLAYER_ABOVE);
        let mut wiring = crate::wiring::Wiring::new();
        let sx = 10 * 16;
        let sy = 20 * 16;
        // A player well above: the box is y*16-161 .. y*16+1.
        let inside = (sx - 10, sy - 150, 20, 42);
        update_logic_sensors(&mut w, &mut wiring, true, &[inside]);
        assert!(matches!(
            w.tile_entities.get_at(10, 20).unwrap().data,
            TeData::Logic { on: true, .. }
        ));
        // The player leaves: the state clears and re-trips on the return.
        update_logic_sensors(&mut w, &mut wiring, true, &[]);
        assert!(matches!(
            w.tile_entities.get_at(10, 20).unwrap().data,
            TeData::Logic { on: false, .. }
        ));
        update_logic_sensors(&mut w, &mut wiring, true, &[inside]);
        assert!(matches!(
            w.tile_entities.get_at(10, 20).unwrap().data,
            TeData::Logic { on: true, .. }
        ));
    }

    #[test]
    fn the_dummy_spawns_near_players_and_deactivates_on_the_lost_tie() {
        // `TETrainingDummy.cs:43-75`: the spawn scan (the 32×48 box
        // inflated by 1600 px) and the deactivate on the lost tie.
        let mut w = World::new("t".into(), 64, 64);
        let i = w.index(20, 30);
        w.tiles[i] = Tile {
            kind: 378,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.tile_entities
            .place_unchecked(TeKind::TrainingDummy, 20, 30);
        // No player nearby: no spawn.
        let spawns = update_training_dummies(&mut w, &[], |_| false);
        assert!(spawns.is_empty());
        // The player walks in: the spawn anchor is the pixel front.
        let player = (20 * 16, 30 * 16, 20, 42);
        let spawns = update_training_dummies(&mut w, &[player], |_| false);
        assert_eq!(spawns, vec![(20 * 16 + 16, 30 * 16 + 48)]);
        // The tie pokes back; the alive tie holds.
        assert!(set_dummy_npc(&mut w, 20, 30, 7));
        let spawns = update_training_dummies(&mut w, &[player], |npc| npc == 7);
        assert!(spawns.is_empty(), "the tied dummy stays quiet");
        // The NPC dies: the dummy frees for a respawn.
        update_training_dummies(&mut w, &[player], |_| false);
        assert!(matches!(
            w.tile_entities.get_at(20, 30).unwrap().data,
            TeData::Dummy(-1)
        ));
    }
}

#[cfg(test)]
mod save_load_tests {
    //! The `.wld` book helpers (`TileEntities::live_entities`/`load_entity`/
    //! `next_id`/`on_world_loaded`) — the halves `SaveTileEntities:3306-3310`
    //! and `LoadTileEntities:3316-3358` stand on. The entity-level wire format
    //! itself is tested in `crate::wld::sections::tile_entities`.

    use super::*;
    use crate::tile::{Tile, FLAG_ACTIVE};

    /// A world with an item frame tile (395) at (10, 10) and a hat rack tile
    /// (475) at (5, 5), both on anchor frames.
    fn two_anchors() -> World {
        let mut w = World::new("t".into(), 32, 32);
        for (x, y, kind, frame_x) in [(10, 10, 395, 0), (5, 5, 475, 0)] {
            let i = w.index(x, y);
            w.tiles[i] = Tile {
                kind,
                frame_x,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w
    }

    #[test]
    fn the_live_entities_walk_the_id_book_and_skip_tombstones() {
        // `remove_at` leaves the id book's slot behind (`TileEntity.Remove`
        // drops it from both C# dictionaries), so the save-side iterator is the
        // place the two books' membership is reconciled.
        let mut books = TileEntities::default();
        books.place_unchecked(TeKind::ItemFrame, 10, 10);
        books.place_unchecked(TeKind::HatRack, 5, 5);
        books.place_unchecked(TeKind::FoodPlatter, 20, 20);
        let ids: Vec<u32> = books.live_entities().map(|e| e.id).collect();
        assert_eq!(ids, vec![0, 1, 2], "the id book is walked in order");
        assert_eq!(books.live_entities().count(), 3);

        books.remove_at(5, 5);
        let ids: Vec<u32> = books.live_entities().map(|e| e.id).collect();
        assert_eq!(ids, vec![0, 2], "the tombstoned slot drops out");
        assert_eq!(books.next_id(), 3, "the counter keeps its place");
    }

    #[test]
    fn load_entity_replaces_a_same_position_record() {
        // `LoadTileEntities:3326-3328`: the record lands in the next slot and
        // an entity already at the position is removed first.
        let world = two_anchors();
        let mut books = TileEntities::default();
        let first = books.load_entity(
            TeKind::ItemFrame,
            10,
            10,
            TeData::Item(Some(TeItem {
                kind: 75,
                prefix: 0,
                stack: 1,
            })),
        );
        let second = books.load_entity(
            TeKind::ItemFrame,
            10,
            10,
            TeData::Item(Some(TeItem {
                kind: 200,
                prefix: 3,
                stack: 4,
            })),
        );
        assert_eq!((first, second), (0, 1), "the slots are sequential");
        assert_eq!(books.live_entities().count(), 1, "one entity per position");
        assert_eq!(books.next_id(), 2, "both records consumed a slot");
        let kept = books.get_at(10, 10).expect("kept");
        assert_eq!(
            kept.data,
            TeData::Item(Some(TeItem {
                kind: 200,
                prefix: 3,
                stack: 4
            })),
            "the later record wins"
        );
        // The validity check is the caller's pass, not `load_entity`'s.
        assert!(TeKind::ItemFrame.is_tile_valid(&world, 10, 10));
    }

    #[test]
    fn the_id_book_is_dense_across_a_load_sequence() {
        // `tileEntity.ID = num2++` (`LoadTileEntities:3323-3324`): every record
        // read takes a slot, so the ids are 0..count-1 even when a later one
        // replaces an earlier position.
        let mut books = TileEntities::default();
        for (kind, x, y) in [
            (TeKind::ItemFrame, 10, 10),
            (TeKind::HatRack, 5, 5),
            (TeKind::ItemFrame, 10, 10),
            (TeKind::HatRack, 5, 5),
        ] {
            books.load_entity(kind, x, y, TeData::None);
        }
        assert_eq!(books.next_id(), 4, "the counter ends at the record count");
        assert_eq!(
            books.live_entities().map(|e| e.id).collect::<Vec<_>>(),
            vec![2, 3],
            "only the two surviving records are live"
        );
    }

    #[test]
    fn the_world_loaded_hook_leaves_the_books_alone() {
        // `LoadTileEntities:3355-3358` walks `OnWorldLoaded` over the
        // survivors. The ported kinds have no load hook (the leashed anchors'
        // `FitsItem` re-check is M7-TE-2), so the walk is a seam: it must not
        // disturb the books it walks.
        let mut world = two_anchors();
        let mut books = TileEntities::default();
        books.load_entity(TeKind::ItemFrame, 10, 10, TeData::Item(None));
        let before = books.clone();
        TileEntities::on_world_loaded(&mut world);
        assert_eq!(
            books.live_entities().count(),
            before.live_entities().count()
        );
        for (after, want) in books.live_entities().zip(before.live_entities()) {
            assert_eq!(after, want);
        }
    }
}
