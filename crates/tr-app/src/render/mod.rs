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
pub(crate) mod multi_tile_grass;
pub(crate) mod multi_tile_vines;
// W4-g85: the shadow orb / crimson heart draw slice — the ambient
// TileDrawing.cs:7335-7347 arm + the shine colours are WIRED at the
// DrawSingleTile emit slot (render/tiles.rs, the :965-968 quarter-roll
// entry gate); the KillTile hammer-break stage burst stays UNWIRED
// (W4-d54 owns the KillTile dust slot — keep-one at the merge).
pub(crate) mod orb_draw;
pub(crate) mod player_layers;
pub(crate) mod pylons;
pub(crate) mod ropes;
// W4-c40: the SandstormSky tint quad (`GameContent/Skies/SandstormSky.cs`)
// — the SkyManager "Sandstorm" sky's full-screen tint, drawn at the
// sky-depth ladder's DrawToDepth(1f) boundary.
pub(crate) mod sandstorm_sky;
pub(crate) mod screen_fx;
pub(crate) mod sky;
// S1-G-1e: UNWIRED special-tile slice (DrawSpecialTilesLegacy + chest frames
// + platform slope side-fills) — the draw_tiles seam is contested by the
// in-flight S1-G-1 siblings; wiring is the family's merge step.
pub(crate) mod special_tiles;
// W4-d57: UNWIRED TE-furniture draw slice (the 395/520/471 legacy arms +
// the Part1 anchor collection + the ItemSlot icon colour chain) — the
// draw_tiles seam is contested by the in-flight tile-draw siblings; wiring
// is the family's merge step.
pub(crate) mod te_furniture;
pub(crate) mod tile_cracks;
// W4-g28 pass 1: UNWIRED Draw main-pass orchestration (TileDrawing.cs:465-
// 742 head/tail, GetScreenDrawArea :6549-6586, ClearCachedTileDraws :6588-
// 6599, the :504-506 loop margins, the frozen PostDrawTiles order :414-442)
// — the draw_tiles seam is contested by the in-flight pass siblings
// (g29-g32 own passes 2-5); wiring is the family's merge step.
pub(crate) mod tile_pass;
// W4-g32 pass 5: UNWIRED Draw orchestration + PostDrawTiles post-pass order
// (TileDrawing.cs:465-533/:728-741/:414-442/:6549-6622, DrawBlackHelper.cs,
// Main.cs:56949-56988) — the draw_tiles seam is contested by the in-flight
// pass siblings; wiring is the family's merge step (module doc lists the
// intended call sites).
pub(crate) mod tile_draw_pass;
pub(crate) mod tiles;
pub(crate) mod vines;
pub(crate) mod wires;
// W4-d127: `Main.DrawWoF` (`Main.cs:38377-38525`) — the wall's body sweep,
// the Hungry rope leashes and the tongue chains; the seam sits between the
// wall pass and the behind-tiles NPC pass (`tiles.rs`).
pub(crate) mod wof;

/// A decoded sprite rect on the atlas: pixel size plus the normalised UV quad.
/// Shared by every render module (the `Background_*`/`Player_*`/`NPC_*` sheet
/// handles of the original's draw paths).
#[derive(Clone, Copy)]
pub(crate) struct BgTex {
    pub(crate) uv: [f32; 4],
    pub(crate) w: f32,
    pub(crate) h: f32,
}
