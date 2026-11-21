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
// W4-g85: the shadow orb / crimson heart draw slice — the ambient
// TileDrawing.cs:7335-7347 arm + the shine colours are WIRED at the
// DrawSingleTile emit slot (render/tiles.rs, the :965-968 quarter-roll
// entry gate); the KillTile hammer-break stage burst stays UNWIRED
// (W4-d54 owns the KillTile dust slot — keep-one at the merge).
pub(crate) mod orb_draw;
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
