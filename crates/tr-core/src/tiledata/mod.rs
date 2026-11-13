//! Tile-id lookup tables needed for automatic framing.
//!
//! Everything here is derived mechanically from `TileID.Sets.*` and `Main.tile*`
//! by `tools/GenTileData`, which writes the sibling `sets`/`emissive`/`merge`
//! data files — do not edit those by hand, re-run the generator instead. This
//! `mod.rs` carries only the shared counters, the bitset type and the
//! re-exports.

/// Number of tile types (`TileID.Count`).
pub const TILE_COUNT: usize = 754;

/// Number of wall types (`WallID.Count`).
pub const WALL_COUNT: usize = 367;

/// Bitset over tile ids.
pub struct TileSet(&'static [u64]);

impl TileSet {
    /// Const constructor for hand-transcribed sets (e.g. `wind.rs`'s
    /// `SwaysInWindBasic`); the generated tables use the tuple form directly.
    pub const fn from_words(words: &'static [u64]) -> Self {
        Self(words)
    }


    #[inline]
    pub fn has(&self, id: u16) -> bool {
        let i = id as usize;
        i < TILE_COUNT && (self.0[i >> 6] >> (i & 63)) & 1 != 0
    }

    #[inline]
    pub fn any(&self) -> bool {
        self.0.iter().any(|w| *w != 0)
    }
}

mod emissive;
mod merge;
mod sets;

// S1-F-8: the full `TileID.Sets` emission — every bool set the static
// constructors assign plus the value tables (`CreateIntSet`/`CreateUshortSet`
// /`List<int>`/`CreateCustomSet<bool?>`), generated into `sets.rs` (the
// `--check` mode pins the regeneration to an empty diff). The re-export
// glob below keeps the committed hand-listed names importable.
pub use sets::*;

pub use emissive::{EMISSIVE, WALL_LIGHT};
pub use merge::{GRASS_DIRT, LARGE_FRAMES, WALL_LARGE_FRAMES, merges};
// `SET_GRASS` is `TileID.Sets.Conversion.Grass` (TileID.cs:86); it is re-exported
// under its source name to keep the mapping discoverable.
pub use sets::SET_GRASS;

#[cfg(test)]
mod s1_f_8_tests {
    use super::*;

    /// S1-F-8 — the set coverage: `sets.rs` emits EVERY `TileID.Sets` table
    /// the static constructors assign (`TileID.cs` — 134 `CreateBoolSet`
    /// sites + the value sets), not the 46 the old wanted-list filtered.
    /// The count pins the full emission; the spot checks carry the C#'s
    /// sampled members (`Conversion.Corrupt`'s evil family, the touch-damage
    /// int table, the gem tree ground set).
    #[test]
    fn s1_f_8_every_tile_id_sets_table_is_generated() {
        // The generated file's public table count (bool sets + value tables):
        // the walk reads sets.rs so a hand-removed table fails the count.
        let sets_src = include_str!("sets.rs");
        let count = sets_src.matches("pub static SET_").count()
            + sets_src.matches("pub static TILE_").count();
        assert!(
            count >= 150,
            "the full TileID.Sets emission ({count} tables, was 46)"
        );
        // Spot checks against the C# (`TileID.cs:83-96`/`:588`/`:605-609`):
        assert!(
            crate::tiledata::sets::SET_CRIMSON.has(199),
            "Sets.Crimson carries the crimson grass 199 (`:588`)"
        );
        assert!(
            crate::tiledata::sets::SET_CORRUPT.has(23),
            "Sets.Corrupt carries the corrupt grass 23 (`:579`)"
        );
        assert!(
            crate::tiledata::sets::SET_TOUCH_DAMAGE_HOT.has(37),
            "the touch-damage hot set carries the torch 37"
        );
        assert_eq!(
            crate::tiledata::sets::SET_CONVEYOR_DIRECTION[421],
            1,
            "the conveyor int table (`:421` is the left-facing belt)"
        );
        assert_eq!(
            crate::tiledata::sets::SET_CORRUPT_COUNT_COLLECTION[23],
            1,
            "the corrupt count collection opens with the corrupt grass 23"
        );
    }
}
