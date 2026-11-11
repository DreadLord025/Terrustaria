//! The eleven `.wld` sections in `SaveWorld_Version2` order
//! (`WorldFile.cs:1150-1165`): the plain-data structs, the per-section stub
//! modules and the registry both directions walk.
//!
//! **Stub-lane contract:** this file is frozen after Phase 1. Each section
//! phase replaces only the bodies in its own module file — the registry below
//! wires the seams once, so the phases' file scopes stay disjoint.

use crate::binio::{Reader, Writer};

pub mod bestiary;
pub mod chests;
pub mod creative_powers;
pub mod flags;
pub mod header;
pub mod npcs;
pub mod pressure_plates;
pub mod signs;
pub mod tile_entities;
pub mod tiles;
pub mod town_rooms;

pub use bestiary::WldBestiary;
pub use chests::{WldChest, WldChestSlot};
pub use creative_powers::{WldCreativePowers, WldCreativeValue};
pub use npcs::{WldNpcs, WldTower, WldTownNpc};
pub use pressure_plates::WldPressurePlate;
pub use signs::WldSign;
pub use tile_entities::{WldTeExtra, WldTeItem, WldTileEntity};
pub use town_rooms::WldTownRoom;

/// The eight sections past the header/flags/tiles, as one plain-data bag: one
/// field per section, each typed by its own stub module's struct (a section
/// phase grows that struct, never this file).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SectionData {
    pub chests: Vec<WldChest>,
    pub signs: Vec<WldSign>,
    pub npcs: WldNpcs,
    pub tile_entities: Vec<WldTileEntity>,
    pub pressure_plates: Vec<WldPressurePlate>,
    pub town_rooms: Vec<WldTownRoom>,
    pub bestiary: WldBestiary,
    pub creative_powers: WldCreativePowers,
}

impl SectionData {
    /// Folds one section's decoded payload into the bag.
    pub(crate) fn apply(&mut self, payload: SectionPayload) {
        match payload {
            SectionPayload::Chests(v) => self.chests = v,
            SectionPayload::Signs(v) => self.signs = v,
            SectionPayload::Npcs(v) => self.npcs = v,
            SectionPayload::TileEntities(v) => self.tile_entities = v,
            SectionPayload::PressurePlates(v) => self.pressure_plates = v,
            SectionPayload::TownRooms(v) => self.town_rooms = v,
            SectionPayload::Bestiary(v) => self.bestiary = v,
            SectionPayload::CreativePowers(v) => self.creative_powers = v,
        }
    }
}

/// What one section's `read` decodes; the variant IS the section
/// ([`SectionData`] field it fills).
pub enum SectionPayload {
    Chests(Vec<WldChest>),
    Signs(Vec<WldSign>),
    Npcs(WldNpcs),
    TileEntities(Vec<WldTileEntity>),
    PressurePlates(Vec<WldPressurePlate>),
    TownRooms(Vec<WldTownRoom>),
    Bestiary(WldBestiary),
    CreativePowers(WldCreativePowers),
}

/// The load side of one section (pointer 3..10).
pub struct SectionDef {
    /// Index into the C# pointer table (`positions[..]`).
    pub pointer: usize,
    /// Section name, for error messages.
    pub name: &'static str,
    /// The C# version gate: below it the section is not read at all
    /// (`LoadWorld_Version2:1796-1849`).
    pub gate: fn(i32) -> bool,
    /// The section codec. `None` = short read / unimplemented content.
    pub read: fn(&mut Reader, i32) -> Option<SectionPayload>,
    /// Whether the codec above is real. While false the loader skip-reads the
    /// section (the pointer table is its length) and `read` only serves the
    /// empty-section unit tests; the section phase flips this in its own file.
    pub implemented: fn() -> bool,
}

// Version gates, verbatim from `LoadWorld_Version2`.
fn gate_116(version: i32) -> bool {
    version >= 116
}
fn gate_170(version: i32) -> bool {
    version >= 170
}
fn gate_189(version: i32) -> bool {
    version >= 189
}
fn gate_210(version: i32) -> bool {
    version >= 210
}
fn gate_220(version: i32) -> bool {
    version >= 220
}

/// The registry in `SaveWorld_Version2` order (pointer 3 first).
pub const DEFS: &[SectionDef] = &[
    SectionDef {
        pointer: 3,
        name: "chests",
        gate: |_| true,
        read: |r, v| chests::read(r, v).map(chests::payload),
        implemented: chests::implemented,
    },
    SectionDef {
        pointer: 4,
        name: "signs",
        gate: |_| true,
        read: |r, v| signs::read(r, v).map(signs::payload),
        implemented: signs::implemented,
    },
    SectionDef {
        pointer: 5,
        name: "npcs",
        gate: |_| true,
        read: |r, v| npcs::read(r, v).map(npcs::payload),
        implemented: npcs::implemented,
    },
    SectionDef {
        pointer: 6,
        name: "tile entities",
        gate: gate_116,
        read: |r, v| tile_entities::read(r, v).map(tile_entities::payload),
        implemented: tile_entities::implemented,
    },
    SectionDef {
        pointer: 7,
        name: "pressure plates",
        gate: gate_170,
        read: |r, v| pressure_plates::read(r, v).map(pressure_plates::payload),
        implemented: pressure_plates::implemented,
    },
    SectionDef {
        pointer: 8,
        name: "town rooms",
        gate: gate_189,
        read: |r, v| town_rooms::read(r, v).map(town_rooms::payload),
        implemented: town_rooms::implemented,
    },
    SectionDef {
        pointer: 9,
        name: "bestiary",
        gate: gate_210,
        read: |r, v| bestiary::read(r, v).map(bestiary::payload),
        implemented: bestiary::implemented,
    },
    SectionDef {
        pointer: 10,
        name: "creative powers",
        gate: gate_220,
        read: |r, v| creative_powers::read(r, v).map(creative_powers::payload),
        implemented: creative_powers::implemented,
    },
];

/// The save side of one of the eleven sections.
pub struct SaveDef {
    /// Pointer-table index (0..11).
    pub pointer: usize,
    pub name: &'static str,
    /// Writes this section's bytes. For pointers 0..2 this is the
    /// header/flags/tiles writer; for 3..10 the section codec's write half.
    pub write: fn(&mut Writer, &crate::world::World, &SectionData, i32),
    /// While false the writer emits only the empty encoding (Phase 1).
    pub implemented: fn() -> bool,
    /// Whether the section carries no data yet — the writer may only run while
    /// this holds or the codec is real, so `save` never emits a section that
    /// claims records it did not write.
    pub empty: fn(&SectionData) -> bool,
}

fn all_empty(_: &SectionData) -> bool {
    true
}

/// The registry in `SaveWorld_Version2` order (`WorldFile.cs:1150-1165`):
/// pointer 0 is the prologue (assembled by `super::save`, `SaveFileFormatHeader:1170-1180`)
/// plus this entry's tile-count + importance bitmap (`:1181-1204`); pointer 1
/// is the world-header writer followed by the flags writer.
pub const SAVE_DEFS: &[SaveDef] = &[
    SaveDef {
        pointer: 0,
        name: "file format header",
        write: |w, _world, _data, _version| header::write_bitmap(w, _world),
        implemented: header::implemented,
        empty: all_empty,
    },
    SaveDef {
        pointer: 1,
        name: "world header + flags",
        write: |w, _world, _data, _version| {
            header::write_world_head(w, _world);
            flags::write(w, _world);
        },
        implemented: header::implemented,
        empty: all_empty,
    },
    SaveDef {
        pointer: 2,
        name: "tiles",
        write: |w, _world, _data, _version| tiles::write(w, _world),
        implemented: tiles::implemented,
        empty: all_empty,
    },
    SaveDef {
        pointer: 3,
        name: "chests",
        write: |w, _world, _data, _version| chests::write(w, _data, _version),
        implemented: chests::implemented,
        empty: chests::empty,
    },
    SaveDef {
        pointer: 4,
        name: "signs",
        write: |w, _world, _data, _version| signs::write(w, _data, _version),
        implemented: signs::implemented,
        empty: signs::empty,
    },
    SaveDef {
        pointer: 5,
        name: "npcs",
        write: |w, _world, _data, _version| npcs::write(w, _data, _version),
        implemented: npcs::implemented,
        empty: npcs::empty,
    },
    SaveDef {
        pointer: 6,
        name: "tile entities",
        write: |w, _world, _data, _version| tile_entities::write(w, _data, _version),
        implemented: tile_entities::implemented,
        empty: tile_entities::empty,
    },
    SaveDef {
        pointer: 7,
        name: "pressure plates",
        write: |w, _world, _data, _version| pressure_plates::write(w, _data, _version),
        implemented: pressure_plates::implemented,
        empty: pressure_plates::empty,
    },
    SaveDef {
        pointer: 8,
        name: "town rooms",
        write: |w, _world, _data, _version| town_rooms::write(w, _data, _version),
        implemented: town_rooms::implemented,
        empty: town_rooms::empty,
    },
    SaveDef {
        pointer: 9,
        name: "bestiary",
        write: |w, _world, _data, _version| bestiary::write(w, _data, _version),
        implemented: bestiary::implemented,
        empty: bestiary::empty,
    },
    SaveDef {
        pointer: 10,
        name: "creative powers",
        write: |w, _world, _data, _version| creative_powers::write(w, _data, _version),
        implemented: creative_powers::implemented,
        empty: creative_powers::empty,
    },
];
