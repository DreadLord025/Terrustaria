//! The world-rendering passes, split out of `main.rs` (plan stage A3): the sky
//! and biome background ([`sky`]/[`bg`]), the wall-to-wall tile and liquid
//! passes ([`tiles`]/[`liquids`]), entities ([`entities`]), tree canopies
//! ([`foliage`]) and the debug overlay ([`debug`]). Every pass is a free
//! `pub(crate)` function driven by `App::frame`, which owns the renderer, the
//! light map and the per-frame caches.

pub(crate) mod bg;
pub(crate) mod debug;
pub(crate) mod entities;
pub(crate) mod empress_draw;
pub(crate) mod foliage;
pub(crate) mod gore;
pub(crate) mod liquids;
pub(crate) mod multi_tile_grass;
pub(crate) mod multi_tile_vines;
pub(crate) mod player_layers;
pub(crate) mod pylons;
pub(crate) mod ropes;
pub(crate) mod screen_fx;
pub(crate) mod sky;
// W4-g30 pass 3: UNWIRED Draw classification switch + special-point registry
// (TileDrawing.cs:534-723/:183-195/:388-402/:764-782/:7702-7717) — the
// draw_tiles seam is contested by the in-flight pass siblings; wiring is the
// family's merge step (the module doc lists the intended call sites).
pub(crate) mod special_points;
// S1-G-1e: UNWIRED special-tile slice (DrawSpecialTilesLegacy + chest frames
// + platform slope side-fills) — the draw_tiles seam is contested by the
// in-flight S1-G-1 siblings; wiring is the family's merge step.
pub(crate) mod special_tiles;
// W4-g31 attach phase: the TileDrawing static-ctor DRAW residual
// (TileDrawing.cs:789-862 — noise[256] via System.Random(0), LinearNoise,
// the hash mixers, DistToWanderingCircle, LavaLightA for the 717 glowmask
// arm :1193-1197). The layer constants/_zero/registry halves of the ctor
// are covered by tile_pass/special_points; this closes the last draw.
pub(crate) mod tile_ctor;
// W4-g31 attach phase: the TR_TILEPASS shadow attach (default OFF) — the
// g29 draw_layer/post_draw_tiles drivers + the g30 registry + the g31
// drains run PLAN-ONLY at the C# PostDrawTiles position in tiles.rs
// (UNION-CONTRACT-tilepass §8); the pixel-binding merge is §8's script.
pub(crate) mod tile_attach;
// W4-d57: UNWIRED TE-furniture draw slice (the 395/520/471 legacy arms +
// the Part1 anchor collection + the ItemSlot icon colour chain) — the
// draw_tiles seam is contested by the in-flight tile-draw siblings; wiring
// is the family's merge step.
pub(crate) mod te_furniture;
pub(crate) mod tile_cracks;
// W4-g29 pass 2: UNWIRED Draw orchestration (prologue/loop bounds/black
// strips/epilogue/PostDrawTiles order) — the draw_tiles seam is contested
// by the in-flight pass siblings (g28/g30/g31/g32, f15); wiring is the
// pass-family merge step.
// W4-g31 pass 4: UNWIRED deferred bucket drains (TileDrawing.cs PostDrawTiles
// :414-442 bodies — DrawMultiTileVines(InWind)/DrawMultiTileGrass(InWind)/
// DrawGrass/DrawAnyDirectionalGrass/DrawReverseVines + ClimbCatTail). The
// draw_tiles seam is contested by the in-flight pass siblings (g28-g32,
// d56-d60); wiring is the family's merge step (the module doc lists the
// intended call sites + the DrainBuckets/DrainCellLight/PortedBodyDrain seams).
pub(crate) mod tile_drains;
// W4-g32 pass 5 (union-reduced): the Draw head/tail step witnesses + the
// g32-only pieces (EnsureWindGridSize, the layer pick, LegacyCaches, the
// tile-phase orders) — TileDrawing.cs:465-533/:728-741/:7773-7777/:877-887/
// :6606-6622, Main.cs:56949-56988. The duplicated area/black-helper/post
// orders keep-one onto tile_pass.rs (UNION-CONTRACT-tilepass §1.3).
pub(crate) mod tile_draw_pass;
// pass-family merge step. The g28 pass-1 head folds in here (its stand-in
// registry dropped onto g30's; its payload is tr_render::quality_light).
// W4-g28 pass 1: UNWIRED Draw main-pass orchestration (TileDrawing.cs:465-
// 742 head/tail, GetScreenDrawArea :6549-6586, ClearCachedTileDraws :6588-
// 6599, the :504-506 loop margins, the frozen PostDrawTiles order :414-442)
// — the draw_tiles seam is contested by the in-flight pass siblings
// (g29-g32 own passes 2-5); wiring is the family's merge step.
pub(crate) mod tile_pass;
pub(crate) mod tiles;
pub(crate) mod vines;
pub(crate) mod wire_kite;
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
