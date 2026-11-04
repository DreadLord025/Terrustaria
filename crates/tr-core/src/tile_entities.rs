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
//!
//! Deviations (ANALYSIS "Tile entities (M7-TE-1)"): the behaviour halves —
//! the dummy hit tracking, the logic sensors, the pylon network, the display
//! doll rendering and the leashed anchors — are M7-TE-2; the food gate
//! (`ItemID.Sets.IsFood`) is applied by the caller; items here are the
//! `(type, prefix, stack)` triples the wire/save format carries, mapped to
//! the app's `ItemSlot`s at the UI layer.

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
        Some(Self { kind, prefix, stack })
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
            // The behaviour-half entities' tiles land with M7-TE-2 (their
            // placement validity is registered, the data half is empty).
            _ => 0,
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
            // The behaviour-half entities: the placement check is registered
            // but their data half (and the tile map beyond the anchor tile)
            // lands with M7-TE-2.
            _ => true,
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

/// The entity's payload. The behaviour-half entities carry no data yet
/// (M7-TE-2); the HatRack carries its two hat slots and two dye slots.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum TeData {
    #[default]
    None,
    Item(Option<TeItem>),
    HatRack([Option<TeItem>; 2], [Option<TeItem>; 2]),
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

/// The world's entity books — `TileEntity.ByID` / `ByPosition`
/// (`TileEntity.cs:15-16`).
#[derive(Clone, Debug, Default)]
pub struct TileEntities {
    by_id: Vec<TileEntity>,
    by_position: HashMap<(i16, i16), u32>,
    next_id: u32,
}

impl TileEntities {
    pub fn get_by_id(&self, id: u32) -> Option<&TileEntity> {
        self.by_id.get(id as usize)
    }

    pub fn get_at(&self, x: i16, y: i16) -> Option<&TileEntity> {
        self.by_position.get(&(x, y)).map(|&id| &self.by_id[id as usize])
    }

    /// `TileEntitiesManager.CheckValidTile` + `Place` (`:51-60`): the
    /// entity's `ValidTile` must pass and the position must be free.
    /// `kind` selects the entity; the returned id is the `ByID` slot.
    pub fn place(
        &mut self,
        world: &World,
        kind: TeKind,
        x: i16,
        y: i16,
    ) -> Option<u32> {
        if self.by_position.contains_key(&(x, y)) {
            return None;
        }
        if !kind.is_tile_valid(world, x as i32, y as i32) {
            return None;
        }
        let data = match kind {
            TeKind::ItemFrame | TeKind::WeaponsRack | TeKind::FoodPlatter => {
                TeData::Item(None)
            }
            TeKind::HatRack => TeData::HatRack([None, None], [None, None]),
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
}

/// `WriteExtraData`/`ReadExtraData` round-trip for the HatRack payload:
/// a bitsByte of the four slot-presence flags, then each present slot as the
/// item triple (`TEHatRack.cs:59-80`).
pub fn hat_rack_write(
    items: &[Option<TeItem>; 2],
    dyes: &[Option<TeItem>; 2],
    w: &mut Writer,
) {
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
    use crate::tile::{FLAG_ACTIVE, Tile};

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
        assert!(TeKind::ItemFrame.is_tile_valid(&w, 10, 10) || {
            TeKind::ItemFrame.is_tile_valid(&w, 10, 10)
        });
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
        assert!(tes.set_item(id, Some(TeItem { kind: 50, prefix: 0, stack: 1 })));
        let (kind, data) = tes.remove_at(10, 10).unwrap();
        assert_eq!(kind, TeKind::ItemFrame);
        assert_eq!(data, TeData::Item(Some(TeItem { kind: 50, prefix: 0, stack: 1 })));
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
            Some(TeItem { kind: 100, prefix: 1, stack: 1 }),
            None,
        ];
        let dyes = [None, Some(TeItem { kind: 3454, prefix: 0, stack: 1 })];
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
}
