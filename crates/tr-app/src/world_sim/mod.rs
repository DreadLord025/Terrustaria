//! The world simulation, split out of `main.rs` (plan stage A4): the shared
//! fixed-timestep accumulator ([`step`]), the world clock ([`clock`]), the
//! player ([`player`]), NPCs ([`npcs`]), item drops ([`items`]), projectiles
//! ([`projectiles`]), tile/wall editing ([`editing`]), liquids and waterfalls
//! ([`liquids`]), rain ([`weather`]), the camera pan ([`camera`]) and the
//! scene-metrics scan ([`scene_scan`]). Every
//! system is a free `pub(crate)` function driven by `App`'s fixed-step update
//! methods, in the original call order.

pub(crate) mod camera;
pub(crate) mod gfx_quality;
pub(crate) mod clock;
pub(crate) mod editing;
pub(crate) mod extract;
pub(crate) mod growth;
pub(crate) mod items;
pub(crate) mod liquids;
pub(crate) mod lunar;
pub(crate) mod npc_snapshot;
pub(crate) mod npcs;
pub(crate) mod player;
pub(crate) mod projectiles;
pub(crate) mod scene_scan;
pub(crate) mod smart_cursor;
// W4-g86: the smart-interact tile-targeting lookup (Player.SmartInteractLookup,
// Player.cs:17374-17398) — the SmartInteractTileCoords feed of the outline pass.
pub(crate) mod smart_interact;
pub(crate) mod tile_entities;
pub(crate) mod step;
pub(crate) mod weather;
pub(crate) mod wiring;
