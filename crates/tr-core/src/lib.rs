//! Core data model for the Terraria reimplementation.
//!
//! The layout deliberately follows the original `Terraria.Tile` bit flags so that
//! world files can be mapped 1:1 without lossy conversion.

pub mod framing;
pub mod tile;
pub mod wld;
pub mod world;

pub use tile::Tile;
pub use world::World;

/// Size of one tile in world pixels (the game renders everything in pixels).
pub const TILE_PX: f32 = 16.0;
/// Stride between tiles inside a tileset texture (16 px art + 2 px padding).
pub const TILE_STRIDE: i32 = 18;
