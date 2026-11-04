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
            // The critter anchor is placed by the released critters, not by
            // a tile (`TECritterAnchor` has no ValidTile).
            TeKind::CritterAnchor => 0,
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
            // The doll/pylon/anchors: the frame checks are the multi-tile
            // registries (the doll = the 3×4 rack, the pylon = the 3×4
            // network side); the anchor-tile checks land with their
            // follow-ups.
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
        self.by_position.get(&(x, y)).map(|&id| &self.by_id[id as usize])
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
            // `TETrainingDummy`'s ctor: `npc = -1`.
            TeKind::TrainingDummy => TeData::Dummy(-1),
            // `TELogicSensor`'s ctor: `logicCheck = None, On = false`.
            TeKind::LogicSensor => TeData::Logic {
                check: 0,
                on: false,
                counted: 0,
            },
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
            player_boxes
                .iter()
                .any(|&(px, py, pw, ph)| bx < px + pw && bx + bw > px && by < py + ph && by + bh > py)
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
            if let TeData::Logic { on: o, counted: c, .. } = &mut e.data {
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

#[cfg(test)]
mod behaviour_tests {
    //! The logic sensors (`TELogicSensor.cs`) and the training dummies
    //! (`TETrainingDummy.cs`) — the M7-TE-2 halves.

    use super::*;
    use crate::tile::{FLAG_ACTIVE, FLAG_LIQUID_LAVA, Tile};
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
        assert_eq!(w.tile(10, 22).unwrap().kind, 421, "the dawn tripped the wire");
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
        w.tile_entities.place_unchecked(TeKind::TrainingDummy, 20, 30);
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
