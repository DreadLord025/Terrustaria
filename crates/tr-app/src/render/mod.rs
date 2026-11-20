//! The world-rendering passes, split out of `main.rs` (plan stage A3): the sky
//! and biome background ([`sky`]/[`bg`]), the wall-to-wall tile and liquid
//! passes ([`tiles`]/[`liquids`]), entities ([`entities`]), tree canopies
//! ([`foliage`]) and the debug overlay ([`debug`]). Every pass is a free
//! `pub(crate)` function driven by `App::frame`, which owns the renderer, the
//! light map and the per-frame caches.

pub(crate) mod bg;
pub(crate) mod debug;
pub(crate) mod entities;
pub(crate) mod foliage;
pub(crate) mod gore;
pub(crate) mod liquids;
pub(crate) mod player_layers;
pub(crate) mod pylons;
pub(crate) mod ropes;
pub(crate) mod screen_fx;
pub(crate) mod sky;
// S1-G-1e: UNWIRED special-tile slice (DrawSpecialTilesLegacy + chest frames
// + platform slope side-fills) — the draw_tiles seam is contested by the
// in-flight S1-G-1 siblings; wiring is the family's merge step.
pub(crate) mod special_tiles;
pub(crate) mod tile_cracks;
// W4-g31 pass 4: UNWIRED deferred bucket drains (TileDrawing.cs PostDrawTiles
// :414-442 bodies — DrawMultiTileVines(InWind)/DrawMultiTileGrass(InWind)/
// DrawGrass/DrawAnyDirectionalGrass/DrawReverseVines + ClimbCatTail). The
// draw_tiles seam is contested by the in-flight pass siblings (g28-g32,
// d56-d60); wiring is the family's merge step (the module doc lists the
// intended call sites + the DrainBuckets/DrainCellLight/PortedBodyDrain seams).
pub(crate) mod tile_drains;
pub(crate) mod tiles;
pub(crate) mod vines;
pub(crate) mod wires;

/// A decoded sprite rect on the atlas: pixel size plus the normalised UV quad.
/// Shared by every render module (the `Background_*`/`Player_*`/`NPC_*` sheet
/// handles of the original's draw paths).
#[derive(Clone, Copy)]
pub(crate) struct BgTex {
    pub(crate) uv: [f32; 4],
    pub(crate) w: f32,
    pub(crate) h: f32,
}
