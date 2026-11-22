//! Core data model for the Terraria reimplementation.
//!
//! The layout deliberately follows the original `Terraria.Tile` bit flags so that
//! world files can be mapped 1:1 without lossy conversion.

pub mod anim;
pub mod ball;
pub mod bcrypt;
pub mod armor_sets;
pub mod bestiary;
pub mod binio;
pub mod clock;
pub mod damage;
pub mod drawdata;
pub mod drawdata_anim;
pub mod drawdata_cages;
pub mod drawdata_furniture;
pub mod dd2;
pub mod dd2_victory;
pub mod dust_table;
pub mod edit;
pub mod extract;
pub mod falling;
pub mod flame;
pub mod framing;
// W4-g82 — TileFrameCosmetic's rope arm (WorldGen.cs:81894-81931), the
// tr-core home of render/ropes.rs's verbatim decision maths; wired into
// framing::frame_of_variant's merge-chain tail.
pub mod framing_rope;
pub mod generation_progress;
pub mod glowmask;
pub mod housing;
pub mod hit_tile;
pub mod lighting;
pub mod liquid;
pub mod liquid_draw;
pub mod localization;
pub mod loot;
pub mod map;
pub mod mapdata;
pub mod particles;
pub mod save;
pub mod scene;
pub mod seeds;
pub mod shimmer;
pub mod shimmer_tables;
pub mod sky;pub mod slot;
pub mod smart_cursor;
pub mod tile;
pub mod tile_light;
pub mod tiledata;
pub mod tool;
pub mod tileobject;
pub mod whip;
pub mod fishing;
pub mod ores;
pub mod spread;
pub mod stray;
pub mod growth;
pub mod pylons;
pub mod rng;
pub mod tile_entities;
pub mod torch;
pub mod unified_random;
pub mod tree;
pub mod waterfall;
pub mod wiring;
pub mod wind;
pub mod wld;
pub mod world;
pub mod worldgen;
pub mod worldgen_dust;
pub mod worldgen_dust_tile;
pub mod worldgen_dust_wall;

pub use tile::Tile;
pub use unified_random::UnifiedRandom;
pub use world::World;

/// Size of one tile in world pixels (the game renders everything in pixels).
pub const TILE_PX: f32 = 16.0;
/// Stride between tiles inside a tileset texture (16 px art + 2 px padding).
pub const TILE_STRIDE: i32 = 18;
pub mod golf;
pub mod golf_state;
pub mod gore; pub mod gore_id;
