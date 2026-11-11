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

pub use emissive::{EMISSIVE, WALL_LIGHT};
pub use merge::{GRASS_DIRT, LARGE_FRAMES, merges};
pub use sets::{
    MERGES_WITH_ALL, SET_ALLOWS_SAVE_COMPRESSION_BATCHING, SET_ALLOW_LIGHT_IN_WATER, SET_ASH,
    SET_CAN_BE_CLEARED_DURING_GENERATION, SET_CHECKS_FOR_MERGE, SET_CLOUDS,
    SET_CONVERSION_HARDENED_SAND, SET_CONVERSION_MOSS, SET_CONVERSION_SAND,
    SET_CONVERSION_SANDSTONE, SET_CONVERSION_STONE, SET_CRACKED_BRICKS, SET_DIRT, SET_FALLING,
    SET_FORCED_DIRT_MERGING, SET_GRASS_SPECIAL, SET_HELL_SPECIAL, SET_ICE_SKATE_SLIPPERY,
    SET_IS_A_TREE_TRUNK,
    SET_ICES_SNOW, SET_IS_VINE, SET_JUNGLE_SPECIAL, SET_MERGES_WITH_CLOUDS,
    SET_MERGES_WITH_DIRT_IN_A_SPECIAL_WAY, SET_MUD,
    SET_NEEDS_GRASS_FRAMING, SET_ORE, SET_ORE_MERGES_WITH_MUD, SET_PLATFORMS, SET_SNOW,
    SET_TILE_MOSS_BRICK, SET_TORCHES, TILE_BLEND_ALL, TILE_BLOCK_LIGHT, TILE_BRICK,
    TILE_LIGHTED, TILE_MERGE_DIRT, TILE_MOSS, TILE_NO_SUN_LIGHT, TILE_PILE, TILE_ROPE,
    TILE_SOLID, TILE_SOLID_TOP, TILE_STONE,
};
// `SET_GRASS` is `TileID.Sets.Conversion.Grass` (TileID.cs:86); it is re-exported
// under its source name to keep the mapping discoverable.
pub use sets::SET_GRASS;
