//! Core data model for the Terraria reimplementation.
//!
//! The layout deliberately follows the original `Terraria.Tile` bit flags so that
//! world files can be mapped 1:1 without lossy conversion.

pub mod anim;
pub mod binio;
pub mod drawdata;
pub mod edit;
pub mod flame;
pub mod framing;
pub mod lighting;
pub mod liquid;
pub mod liquid_draw;
pub mod localization;
pub mod map;
pub mod mapdata;
pub mod save;
pub mod sky;
pub mod tile;
pub mod tile_light;
pub mod tiledata;
pub mod tileobject;
pub mod torch;
pub mod tree;
pub mod waterfall;
pub mod wind;
pub mod wld;
pub mod world;
pub mod worldgen;

pub use tile::Tile;
pub use world::World;

/// Size of one tile in world pixels (the game renders everything in pixels).
pub const TILE_PX: f32 = 16.0;
/// Stride between tiles inside a tileset texture (16 px art + 2 px padding).
pub const TILE_STRIDE: i32 = 18;
