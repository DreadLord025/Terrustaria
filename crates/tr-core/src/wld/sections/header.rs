//! Section 1 (pointer 0) tail + section 2 (pointer 1) head —
//! `WorldFile.SaveFileFormatHeader:1181-1204` and `WorldFile.SaveWorldHeader:1221`.
//!
//! Phase 1 stubs: both bodies empty. The 70-byte prologue itself (version,
//! `FileMetadata`, `short 11`, the 11 pointer placeholders) is framing and is
//! assembled by `super::save` (`SaveFileFormatHeader:1170-1180`); the header +
//! flags writer phase fills these in — the tile count + importance bitmap here,
//! the world-header fields in `write_world_head`, and the ~180 flag fields in
//! `flags::write`.

use crate::binio::Writer;
use crate::world::World;

pub fn implemented() -> bool {
    false
}

/// The `TileID.Count` + importance bitmap that follows the prologue
/// (`SaveFileFormatHeader:1181-1204`). Phase 1 writes the empty form (zero
/// tile types, no bitmap) so the saved file's pointer 0 is self-consistent;
/// the real body reads `world.frame_important` and `TileID.Count`.
pub fn write_bitmap(w: &mut Writer, _world: &World) {
    w.u16(0);
}

/// The world-header head: name, seed text, generator version, GUID, world id,
/// the world bounds and dimensions (`SaveWorldHeader:1221-1233`).
pub fn write_world_head(_w: &mut Writer, _world: &World) {}
