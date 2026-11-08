//! Section 1 (pointer 0) tail + section 2 (pointer 1) head —
//! `SaveFileFormatHeader:1181-1204` (`WorldFile.cs`) and
//! `SaveWorldHeader:1221-1233` (`WorldFile.cs`).
//!
//! The 70-byte prologue itself (version, `FileMetadata`, `short 11`, the 11
//! pointer placeholders) is framing and is assembled by `super::save`
//! (`SaveFileFormatHeader:1170-1180`); what follows it in pointer 0's body is
//! the tile count + importance bitmap, and pointer 1 opens with the world
//! header before [`super::flags::write`] continues the block.

use crate::binio::Writer;
use crate::tiledata::TILE_COUNT;
use crate::world::World;

pub fn implemented() -> bool {
    true
}

/// Writes the `TileID.Count` + importance bitmap that follows the prologue
/// (`WorldFile.cs:1181-1204`): `Main.tileFrameImportant` as one bit per
/// tile type, LSB-first, with the partial trailing byte suppressed only when
/// the count fills whole bytes (`count % 8 == 0`).
pub fn write_bitmap(w: &mut Writer, world: &World) {
    // `SaveFileFormatHeader:1181-1204`. The C# writes `TileID.Count` and
    // indexes its fixed `Main.tileFrameImportant` array; the port's table is
    // the world's own bitmap, so a shorter or absent one reads as "not
    // important" — the emitted length is always the C#'s.
    let count = TILE_COUNT;
    w.u16(count as u16);
    let mut byte = 0u8;
    let mut mask = 1u8;
    for kind in 0..count {
        if world.frame_important.get(kind).copied().unwrap_or(false) {
            byte |= mask;
        }
        if mask == 128 {
            w.u8(byte);
            byte = 0;
            mask = 1;
        } else {
            mask <<= 1;
        }
    }
    if mask != 1 {
        w.u8(byte);
    }
}

/// The world-header head: name, seed text, generator version, GUID, world id,
/// the world bounds and dimensions (`WorldFile.cs:1221-1233`).
pub fn write_world_head(w: &mut Writer, world: &World) {
    w.cs_string(&world.name);
    // `Main.ActiveWorldFileData.SeedText` — the port's seed is a u64 whose
    // canonical text form is its decimal digits (the port parses the text back
    // with `parse::<u64>`; see `super::flags::read`). Deviation: a non-numeric
    // seed text cannot be represented, and lands as 0 on read.
    w.cs_string(&world.seed.to_string());
    // `Main.ActiveWorldFileData.WorldGeneratorVersion` — unowned (the port has
    // no generator build); `WorldFileData.cs:355`'s init value.
    w.u64(0);
    // `Main.ActiveWorldFileData.UniqueId.ToByteArray()` — unowned; `Guid.Empty`.
    w.bytes(&[0u8; 16]);
    // `Main.ActiveWorldFileData.WorldId` — the port keeps no world id.
    w.i32(0);
    // The world bounds (`WorldGen.cs:6551-6554`: right = maxTilesX * 16,
    // bottom = maxTilesY * 16, left/top stay 0) — derived from the owned dims.
    w.i32(0);
    w.i32(world.width * 16);
    w.i32(0);
    w.i32(world.height * 16);
    // Y before X in the file (`SaveWorldHeader:1232-1233`).
    w.i32(world.height);
    w.i32(world.width);
}
