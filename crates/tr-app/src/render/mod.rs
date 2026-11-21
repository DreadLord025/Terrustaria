//! The world-rendering passes, split out of `main.rs` (plan stage A3): the sky
//! and biome background ([`sky`]/[`bg`]), the wall-to-wall tile and liquid
//! passes ([`tiles`]/[`liquids`]), entities ([`entities`]), tree canopies
//! ([`foliage`]) and the debug overlay ([`debug`]). Every pass is a free
//! `pub(crate)` function driven by `App::frame`, which owns the renderer, the
//! light map and the per-frame caches.

pub(crate) mod bg;
// W4-g72 lane 3: the cavern background STYLE machine (pick + sheet tables +
// cross-fade), UNWIRED — the sheet variety overflows the 8192 atlas bound and
// the style rolls would shift the worldgen tail stream (module docs).
pub(crate) mod bg_styles;
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
pub(crate) mod tiles;
// W4-g72 lane 3: the hell parallax stack geometry (DrawUnderworldBackground /
// DrawUnderworldBackgroundLayer), UNWIRED — the Underworld sheets overflow the
// 8192 atlas bound (module docs).
pub(crate) mod underworld_bg;
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
