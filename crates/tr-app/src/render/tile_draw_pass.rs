//! W4-g32 — TileDrawing pass 5, UNION-REDUCED (docs/UNION-CONTRACT-
//! tilepass.md §1.3): the `Draw` head/tail STEP witnesses (`draw_steps`,
//! `TileDrawing.cs:465-533`/`:728-741`), `EnsureWindGridSize`
//! (`:7773-7777`), `DrawSingleTile`'s layer pick (`:877-887`) + the
//! per-cell `SetLayer` sequence (`:521-526`), the legacy draw caches
//! (`LegacyCaches`, `:82-102`/`:6588-6622`/`:6606-6611` — the storage
//! canonical the g29 `LEGACY_CLEAR_ORDER`/`layer_cache_clears` enums bind
//! onto), `DrawSpecialTilesLegacy`'s empty-cache early return
//! (`:7895-7899`) and the tile-PHASE frame orders (`frame_tile_draw_order`
//! / `render_target_tile_order`, `Main.cs:56949-56988`/`:57296-57305`).
//!
//! The union-exec on this branch DROPPED every piece g29's
//! `render/tile_pass.rs` owns (contract §1.3/§2): the `LAYER_*` /
//! `MAX_SPECIALS_LEGACY` constants (g29's identical copies), the
//! `ScreenDrawArea`/`get_screen_draw_area`/`draw_loop_bounds` maths (g29
//! canonical; this file's `needs_framing`/`needs_refresh` flags FOLDED
//! onto its struct), `martian_glow` (g29 `martian_glow_byte`; the
//! `[b,b,b,0]` quad assertion folded into its wrap witness),
//! `DrawBlackHelper` (g29's `BlackStrip` copy; the offset/row-break
//! witness REBOUND there), `draw_liquid_behind_tiles_cells` (g29's driver;
//! the inactive-cell/OOB assertions folded there), `PostPass`/
//! `post_draw_tiles_passes`/`post_pass_bucket` (g29 `POST_DRAW_NON_SOLID`
//! owns the outer order, g31 `drain_pass_bucket` owns the bucket map; the
//! `None`-arm assertions folded into g31's bucket witness). The g32
//! stale-claim note ("g29/g31 were EMPTY claims") is corrected in the
//! contract's header — both lanes pushed full ports; the keep-ones above
//! supersede it.
//!
//! UNWIRED (the S1-G-1e / special_tiles.rs pattern): the `draw_tiles` seam
//! (`render/tiles.rs` + `app/frame/draw.rs`) is contested by the in-flight
//! tile-draw siblings (g28-g31, d56-d60). Intended call sites at the
//! wiring step:
//! * [`ensure_wind_grid_size`] binds the g29
//!   `TilePassSeams::ensure_wind_grid_size` seam (`tile_pass.rs:476`
//!   head, a FRESH unpadded area probe);
//! * [`LegacyCaches::clear_cached_tile_draws`] — its `false`-arm `true`
//!   return is the g30 `SpecialPoints::clear_special_block_counts` call
//!   (the non-solid arm, `:6597`); the g29 `LegacyCacheClear`/
//!   `LayerCacheClear` steps bind onto the fields/methods here;
//! * [`single_tile_layer`] / [`cell_batch_layers`] — the per-cell
//!   `SetLayer` pick the port's single atlas pass currently folds away
//!   (the S1-G-13 layer-target family);
//! * [`draw_steps`] / [`frame_tile_draw_order`] — the head/tail + phase
//!   order witnesses the g29 driver is checked against;
//! * the TE-position dictionaries' lookups are the d57 furniture slice's
//!   seam; the `chest` key set is the orchestration-side mirror of d58's
//!   wired per-draw HashMap (the `LegacyCacheClear::ChestPositions` step
//!   binds its clear).
//!
//! Deviations (docs/ANALYSIS.md "The Draw orchestration (W4-g32 pass 5)",
//! union note appended):
//! * `TimeLogger` (FlushLogData/DrawCallLogData, `:467-468`) has no port
//!   counterpart — profiling only, no pixels.
//! * The C# null-tile arm (`:509-514`, lazy `new Tile()` + `mapTime += 60`)
//!   has no port counterpart (the tile array is dense) — g30's note.
//! * `DebugOptions.devLightTilesCheat` (`:487-495`) / `ShowUnbreakableWall`
//!   (`:517`) have no port toggles — fed `false` like the siblings.
//! * The section-framing side effects inside `GetScreenDrawArea`
//!   (`:6574-6585`) ride out as flags on g29's `ScreenDrawArea` (the
//!   union fold; the port frames in `tr-core::framing`).
//! * The fixed 1000 legacy slots (`:70`, `:100-102`) grow here instead of
//!   throwing `IndexOutOfRangeException`; the cap drops the excess (the
//!   g30 convention).
#![allow(dead_code)] // UNWIRED — see the module doc; wiring is the family merge step.

use super::tile_pass::{
    LayerCacheClear, LegacyCacheClear, PlacementPreviewGate, LAYER_BEHIND_TILES,
    LAYER_LIQUID_BEHIND_TILES, LAYER_OVER_TILES, LAYER_TILES, MAX_SPECIALS_LEGACY,
};
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// EnsureWindGridSize (TileDrawing.cs:7773-7777) — KEEP (contract §1.3)
// ---------------------------------------------------------------------------

/// `EnsureWindGridSize` (`TileDrawing.cs:7773-7777`): re-query the area and
/// `SetSize(lastTileX - firstTileX, lastTileY - firstTileY)` — the
/// UNPADDED clamped extents, not the walk bounds. `WindGrid::set_size`
/// (`tr-core::wind`, `WindGrid.cs:25-30`) only grows. The union binds the
/// g29 seam to this body; the area comes from the canonical
/// `tile_pass::get_screen_draw_area` (a FRESH probe, `:476`).
pub(crate) fn ensure_wind_grid_size(
    area: &super::tile_pass::ScreenDrawArea,
    grid: &mut tr_core::wind::WindGrid,
) {
    grid.set_size(
        area.last_tile_x - area.first_tile_x,
        area.last_tile_y - area.first_tile_y,
    );
}

// ---------------------------------------------------------------------------
// The layer pick (TileDrawing.cs:877-887, :521-526) — KEEP (g32-only)
// ---------------------------------------------------------------------------

/// `DrawSingleTile`'s batch-layer pick (`TileDrawing.cs:877-887`):
/// `tileTop < 0` → OverTiles; else `tileTop + tileHeight <= 16` → Tiles;
/// else BehindTiles. The inputs are the `GetTileDrawData` outputs.
pub(crate) fn single_tile_layer(tile_top: i32, tile_height: i32) -> u32 {
    if tile_top < 0 {
        LAYER_OVER_TILES
    } else if tile_top + tile_height <= 16 {
        LAYER_TILES
    } else {
        LAYER_BEHIND_TILES
    }
}

/// The per-cell `SetLayer` sequence before classification
/// (`TileDrawing.cs:521-526`): the solid layer draws the behind-tile liquid
/// first on layer 0, then both layers set `Layer_Tiles` for the cell body.
/// `waterStyleOverride` rides the liquid call (`:524`), not the layer pick.
pub(crate) fn cell_batch_layers(solid_layer: bool) -> &'static [u32] {
    if solid_layer {
        &[LAYER_LIQUID_BEHIND_TILES, LAYER_TILES]
    } else {
        &[LAYER_TILES]
    }
}

// ---------------------------------------------------------------------------
// LegacyCaches (TileDrawing.cs:82-102, :6588-6622) — the storage canonical
// ---------------------------------------------------------------------------

/// The legacy draw caches `ClearLegacyCachedDraws`/`ClearCachedTileDraws`
/// reset (`TileDrawing.cs:82-102`). C# keeps `Dictionary<Point, int>` (TE
/// index values); the orchestration only owns the KEYS — the TE lookups
/// are the d57 furniture slice's seam. `legacy_points` is the
/// `_specialTileX/Y` pair (`:100-102`) drained by `DrawSpecialTilesLegacy`
/// (`:7900-7907`). UNION: the g29 `LEGACY_CLEAR_ORDER`/`layer_cache_clears`
/// enums stay the ORDER canonical and bind step-by-step onto these
/// fields/methods ([`LegacyCaches::clear_legacy_step`] /
/// [`LegacyCaches::clear_layer_step`]); g29's `LegacySpecialPoints` and
/// g28's stand-in legacy Vec both dropped onto this list.
#[derive(Default)]
pub(crate) struct LegacyCaches {
    /// `_chestPositions` (`:96`) — filled by `CacheSpecialDraws_Part2`
    /// (`:4300-4303`, the d58 slice; d58's wired per-draw HashMap is the
    /// implementation, this key set is the orchestration-side mirror).
    pub chest: HashSet<(i32, i32)>,
    /// `_trainingDummyTileEntityPositions` (`:86`).
    pub training_dummy: HashSet<(i32, i32)>,
    /// `_foodPlatterTileEntityPositions` (`:92`).
    pub food_platter: HashSet<(i32, i32)>,
    /// `_itemFrameTileEntityPositions` (`:88`).
    pub item_frame: HashSet<(i32, i32)>,
    /// `_deadCellsDisplayJarTileEntityPositions` (`:90`).
    pub dead_cells_jar: HashSet<(i32, i32)>,
    /// `_weaponRackTileEntityPositions` (`:94`).
    pub weapon_rack: HashSet<(i32, i32)>,
    /// `_displayDollTileEntityPositions` (`:82`) — SOLID-layer clear only.
    pub display_doll: HashSet<(i32, i32)>,
    /// `_hatRackTileEntityPositions` (`:84`) — SOLID-layer clear only.
    pub hat_rack: HashSet<(i32, i32)>,
    /// `_specialTileX/Y[.._specialTilesCount]` (`:98-102`).
    pub legacy_points: Vec<(i32, i32)>,
}

impl LegacyCaches {
    /// `AddSpecialLegacyPoint` (`TileDrawing.cs:6606-6611`): append, no
    /// dedup (the per-kind dictionaries dedup upstream). The C# indexes a
    /// fixed `int[1000]` and would throw past the cap; the port drops the
    /// excess (the g30 convention).
    pub fn add_special_legacy_point(&mut self, x: i32, y: i32) {
        if self.legacy_points.len() >= MAX_SPECIALS_LEGACY {
            return;
        }
        self.legacy_points.push((x, y));
    }

    /// `ClearLegacyCachedDraws` (`TileDrawing.cs:6613-6622`): the six
    /// legacy dictionaries in the C#'s order, then `_specialTilesCount = 0`
    /// — run at the head of EVERY `Draw` call, both layers (`:477`).
    pub fn clear_legacy_cached_draws(&mut self) {
        for step in super::tile_pass::LEGACY_CLEAR_ORDER {
            self.clear_legacy_step(step);
        }
    }

    /// One `ClearLegacyCachedDraws` step — the g29
    /// `TilePassSeams::clear_legacy_cache(step)` binding (contract §1.3:
    /// the g29 enum is the ORDER canonical, this is the storage).
    pub fn clear_legacy_step(&mut self, step: LegacyCacheClear) {
        match step {
            LegacyCacheClear::ChestPositions => self.chest.clear(),
            LegacyCacheClear::TrainingDummyTileEntityPositions => self.training_dummy.clear(),
            LegacyCacheClear::FoodPlatterTileEntityPositions => self.food_platter.clear(),
            LegacyCacheClear::ItemFrameTileEntityPositions => self.item_frame.clear(),
            LegacyCacheClear::DeadCellsDisplayJarTileEntityPositions => self.dead_cells_jar.clear(),
            LegacyCacheClear::WeaponRackTileEntityPositions => self.weapon_rack.clear(),
            // `_specialTilesCount = 0` (`:6621`) — the count reset, not
            // the arrays (C# keeps the stale slots; the port truncates).
            LegacyCacheClear::SpecialTilesCount => self.legacy_points.clear(),
        }
    }

    /// `ClearCachedTileDraws` (`TileDrawing.cs:6588-6599`): the solid arm
    /// clears the display-doll + hat-rack dictionaries; the non-solid arm
    /// calls `ClearSpecialBlockCounts` (`:6597`) — the g30 registry's
    /// clear, bound at wiring to `special_points::SpecialPoints::
    /// clear_special_block_counts`. Returns `true` when the caller must
    /// run that seam.
    pub fn clear_cached_tile_draws(&mut self, solid_layer: bool) -> bool {
        if solid_layer {
            self.clear_layer_step(LayerCacheClear::DisplayDollTileEntityPositions);
            self.clear_layer_step(LayerCacheClear::HatRackTileEntityPositions);
            false
        } else {
            true
        }
    }

    /// One `ClearCachedTileDraws` step — the g29
    /// `TilePassSeams::clear_layer_cache(step)` binding. The
    /// `SpecialBlockCounts` arm is the g30 registry seam flag (this
    /// struct holds no buckets); it returns `true` so the driver chains
    /// `clear_special_block_counts`.
    pub fn clear_layer_step(&mut self, step: LayerCacheClear) -> bool {
        match step {
            LayerCacheClear::DisplayDollTileEntityPositions => {
                self.display_doll.clear();
                false
            }
            LayerCacheClear::HatRackTileEntityPositions => {
                self.hat_rack.clear();
                false
            }
            LayerCacheClear::SpecialBlockCounts => true,
        }
    }
}

// ---------------------------------------------------------------------------
// The Draw head/tail step order (TileDrawing.cs:465-533, :728-741) — KEEP
// ---------------------------------------------------------------------------

/// One orchestration step of a `Draw(solidLayer, ..)` call, in C# order.
/// The per-cell classification (g30) and the single-tile pipeline live
/// inside [`DrawStep::CellLoop`]; the quality tiers inside
/// [`DrawStep::QualityLightTiers`] are the g28 seam
/// (`tr_render::quality_light::quality_light_tiers`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DrawStep {
    /// `:469` — `_isActiveAndNotPaused = FocusHelper.AllowTileDrawingToEmitEffects`
    /// (`FocusHelper.cs:27`: `!WorldUpdateStepper.DrawnThisUpdate`).
    EffectsActiveGate,
    /// `:470` — `_perspectivePlayer = Main.SceneMetrics.PerspectivePlayer`.
    PerspectivePlayer,
    /// `:472-475` — `Main.critterCage = false`, NON-SOLID only.
    ResetCritterCage,
    /// `:476` — [`ensure_wind_grid_size`].
    EnsureWindGridSize,
    /// `:477` — [`LegacyCaches::clear_legacy_cached_draws`].
    ClearLegacyCachedDraws,
    /// `:478` — [`LegacyCaches::clear_cached_tile_draws`] (the non-solid
    /// arm chains the g30 registry clear).
    ClearCachedTileDraws,
    /// `:479-495` — the g28 `quality_light_tiers` rebuild (+ the
    /// devLightTilesCheat 255 pin, fed false).
    QualityLightTiers,
    /// `:496` — `tile_pass::get_screen_draw_area` with `!Main.drawToScreen`.
    ScreenDrawArea,
    /// `:497` — `new DrawBlackHelper(Layer_Tiles, drawOffSet)` (the g29
    /// `tile_pass::DrawBlackHelper`).
    NewBlackHelper,
    /// `:498-499` — `tile_pass::martian_glow_byte` (+ the `[b,b,b,0]`
    /// quad, the seam binding).
    MartianGlow,
    /// `:500-503` — `_lastPaintLookupKey.TileType = -1` (the paint cache
    /// key reset; the paint system itself is unported).
    ResetPaintKey,
    /// `:504-727` — the padded row-major walk
    /// (`tile_pass::draw_loop_bounds`): per-cell layer ops
    /// ([`cell_batch_layers`]), texture load (`:530-533`), the g30
    /// classification, `DrawSingleTile` (`:724`).
    CellLoop,
    /// `:728` — `drawBlackHelper.EndStrip()`.
    BlackEndStrip,
    /// `:729` — `base.RestartLayeredBatch()` (`TileDrawingBase.cs:34-40`:
    /// `Main.tileBatch.Restart()`).
    RestartLayeredBatch,
    /// `:732` — `DrawTileCracks(1, hitReplace)`, SOLID only.
    TileCracksHitReplace,
    /// `:733` — `DrawTileCracks(1, hitTile)`, SOLID only.
    TileCracksHitTile,
    /// `:734` — `base.RestartSpriteBatch()` (`TileDrawingBase.cs:42-50`),
    /// SOLID only.
    RestartSpriteBatch,
    /// `:736` — `DrawSpecialTilesLegacy` (`:7893-8024`): early-returns
    /// when `_specialTilesCount == 0` (`:7895-7898`), else
    /// `RestartLayeredBatch` (`:7899`) + per-point `SetLayer(0, 0)`
    /// (`:7908`). The arms are the S1-G-1e/d57 slices.
    SpecialTilesLegacy,
    /// `:737-741` — the placement preview ([`placement_preview_draws`]).
    PlacementPreview,
}

/// The full step sequence of one `Draw` call (`TileDrawing.cs:465-741`).
/// `preview_gates` = `(objectPreview.Active, cursorItemIconEnabled,
/// placementPreview, captureActive)` (`:737`). The TimeLogger selections
/// (`:467-468`) are profiling-only — omitted (module doc). UNION: this is
/// the full head/tail enumeration witness (incl. `EffectsActiveGate`/
/// `PerspectivePlayer` which g29 folds into `DrawLayerInputs`); the g29
/// `draw_layer` driver is the executable canonical.
pub(crate) fn draw_steps(solid_layer: bool, preview_gates: (bool, bool, bool, bool)) -> Vec<DrawStep> {
    use DrawStep::*;
    let mut steps = vec![EffectsActiveGate, PerspectivePlayer];
    if !solid_layer {
        steps.push(ResetCritterCage);
    }
    steps.extend([
        EnsureWindGridSize,
        ClearLegacyCachedDraws,
        ClearCachedTileDraws,
        QualityLightTiers,
        ScreenDrawArea,
        NewBlackHelper,
        MartianGlow,
        ResetPaintKey,
        CellLoop,
        BlackEndStrip,
        RestartLayeredBatch,
    ]);
    if solid_layer {
        steps.extend([TileCracksHitReplace, TileCracksHitTile, RestartSpriteBatch]);
    }
    steps.push(SpecialTilesLegacy);
    if placement_preview_draws(preview_gates) {
        steps.push(PlacementPreview);
    }
    steps
}

/// The placement-preview gate (`TileDrawing.cs:737`):
/// `TileObject.objectPreview.Active && Main.LocalPlayer.cursorItemIconEnabled
/// && Main.placementPreview && !CaptureManager.Instance.Active`. UNION
/// (contract §1.3): the driver binds to the g29
/// `tile_pass::PlacementPreviewGate::draws`; this tuple form stays as the
/// truth-table witness only.
pub(crate) fn placement_preview_draws(gates: (bool, bool, bool, bool)) -> bool {
    gates.0 && gates.1 && gates.2 && !gates.3
}

/// `DrawSpecialTilesLegacy`'s head (`TileDrawing.cs:7895-7899`): the pass
/// restarts the layered batch ONLY when at least one legacy point is
/// cached; an empty cache returns before any batch op.
pub(crate) fn legacy_restarts_batch(legacy_count: usize) -> bool {
    legacy_count != 0
}

// ---------------------------------------------------------------------------
// The tile-phase frame orders (Main.cs:56949-56988, :57296-57305) — KEEP
// ---------------------------------------------------------------------------

/// One tile-draw phase of the drawToScreen frame (`Main.cs:56949-56988` /
/// the `DoDraw` path `:63280-63295`): NON-SOLID first, its post passes,
/// the waterfalls, then SOLID and its post passes. UNION: this tile-PHASE
/// granularity complements the g29 `tile_pass::walls_tiles_npcs_order`
/// NPC-relative skeleton (its `render_target_layer_order` dropped here).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FrameTileStep {
    /// `Main.cs:57527` — `DrawLiquidBehindTiles(waterStyle)` inside the
    /// BACKGROUND `DrawLiquid` (`:57524-57528`), before the tile passes.
    LiquidBehindTilesPass,
    /// `Main.cs:56957`/`:56961` (`DoDraw_Tiles_NonSolid`, `:63363`).
    DrawNonSolid,
    /// `Main.cs:56963` (`DrawTileEntities(false)`, `:21758-21761` →
    /// `PostDrawTiles(false)`; gated by `!DebugOptions.hideTiles2`
    /// `:63372-63375` in the DoDraw path).
    PostNonSolid,
    /// `Main.cs:56968-56969` — `FindWaterfalls(forced: true)` + `Draw`
    /// (`DoDraw_Waterfalls`, `:63310-63319`).
    Waterfalls,
    /// `Main.cs:56982`/`:56986` (`DoDraw_Tiles_Solid`, `:63328`).
    DrawSolid,
    /// `Main.cs:56988` (`DrawTileEntities(true)`; gated by
    /// `!DebugOptions.hideTiles` `:63341-63344` in the DoDraw path).
    PostSolid,
}

/// The drawToScreen tile-phase order (`Main.cs:56949-56988`): the
/// behind-tile liquid rides the background-water pass, then non-solid
/// Draw + Post, waterfalls, solid Draw + Post. Each `DrawTiles` wraps
/// `TilesRenderer.Begin/Draw/End` (`Main.cs:21820-21831`).
pub(crate) fn frame_tile_draw_order() -> &'static [FrameTileStep] {
    use FrameTileStep::*;
    &[LiquidBehindTilesPass, DrawNonSolid, PostNonSolid, Waterfalls, DrawSolid, PostSolid]
}

/// The render-target capture order (`Main.cs:57296-57305`:
/// `tileTarget.UpdateContent(RenderTiles)` then
/// `tile2Target.UpdateContent(RenderTiles2)`; the bodies `:57329-57358`):
/// SOLID first (`RenderTiles`, `:57336`), then non-solid (`RenderTiles2`,
/// `:57357`) — the REVERSE of the screen path, and NO post passes here
/// (the `PostDrawTiles` calls run at composite time, `:63343`/`:63374`).
/// The g29 fold (contract §1.3): the solid target is gated on
/// `!DebugOptions.hideTiles` (`:57334`), the non-solid on
/// `!DebugOptions.hideTiles2` (`:57355`) — no port toggles, fed true.
pub(crate) fn render_target_tile_order() -> &'static [FrameTileStep] {
    use FrameTileStep::*;
    &[DrawSolid, DrawNonSolid]
}

#[cfg(test)]
mod tests {
    use super::*;
    use DrawStep::*;

    fn area(fx: i32, lx: i32, fy: i32, ly: i32) -> super::super::tile_pass::ScreenDrawArea {
        super::super::tile_pass::ScreenDrawArea {
            draw_offset: [0.0, 0.0],
            first_tile_x: fx,
            last_tile_x: lx,
            first_tile_y: fy,
            last_tile_y: ly,
            needs_framing: false,
            needs_refresh: false,
        }
    }

    /// Witness: `EnsureWindGridSize` (`TileDrawing.cs:7773-7777`) sizes
    /// the grid to the UNPADDED extents and `SetSize` only grows
    /// (`WindGrid.cs:25-30` — a same-or-smaller size keeps the stamps, a
    /// growth reallocates and zeroes them).
    #[test]
    fn w4_g32_the_wind_grid_sizes_to_the_unpadded_extents() {
        let mut grid = tr_core::wind::WindGrid::new();
        ensure_wind_grid_size(&area(9, 22, 9, 23), &mut grid); // 13 x 14
        grid.set_wind_time(5, 5, 1, 1);
        assert_eq!(grid.get_wind_time(5, 5, 20).direction_x, 1);
        // Same extents: kept. The walk bounds (17x18 padded) are NOT used.
        ensure_wind_grid_size(&area(9, 22, 9, 23), &mut grid);
        assert_eq!(grid.get_wind_time(5, 5, 20).direction_x, 1, "same-size keeps");
        // Smaller area: still kept (SetSize never shrinks).
        ensure_wind_grid_size(&area(9, 12, 9, 13), &mut grid);
        assert_eq!(grid.get_wind_time(5, 5, 20).direction_x, 1, "no shrink");
        // Growth reallocates -> the stamp is zeroed.
        ensure_wind_grid_size(&area(9, 40, 9, 40), &mut grid);
        assert_eq!(grid.get_wind_time(5, 5, 20).direction_x, 0, "realloc zeroes");
    }

    /// Witness: `DrawSingleTile`'s layer pick (`TileDrawing.cs:877-887`)
    /// — tileTop < 0 wins FIRST (OverTiles even for tall tiles), the
    /// <= 16 sum boundary is inclusive, everything else BehindTiles.
    #[test]
    fn w4_g32_the_single_tile_layer_pick() {
        assert_eq!(single_tile_layer(-1, 40), LAYER_OVER_TILES);
        assert_eq!(single_tile_layer(0, 16), LAYER_TILES);
        assert_eq!(single_tile_layer(4, 12), LAYER_TILES, "sum == 16 is Tiles");
        assert_eq!(single_tile_layer(0, 17), LAYER_BEHIND_TILES);
        assert_eq!(single_tile_layer(1, 16), LAYER_BEHIND_TILES);
        assert_eq!(
            (LAYER_LIQUID_BEHIND_TILES, LAYER_BEHIND_TILES, LAYER_TILES, LAYER_OVER_TILES),
            (0, 1, 2, 3),
            "the static ctor values (:791-794)"
        );
    }

    /// Witness: the per-cell `SetLayer` sequence (`TileDrawing.cs:521-
    /// 526`): solid cells draw the behind-liquid on layer 0 first, then
    /// both layers switch to Layer_Tiles for the cell body.
    #[test]
    fn w4_g32_the_cell_layer_sequence() {
        assert_eq!(cell_batch_layers(true), &[0, 2]);
        assert_eq!(cell_batch_layers(false), &[2]);
    }

    /// Witness (REBIND, contract §3): `AddSpecialLegacyPoint`
    /// (`TileDrawing.cs:6606-6611`) appends without dedup and honours the
    /// 1000-slot cap (`:70`) — retargeted onto the `LegacyCaches` storage
    /// canonical (the g29 `LegacySpecialPoints` dropped).
    #[test]
    fn w4_g32_the_legacy_points_append_and_cap_at_1000() {
        let mut c = LegacyCaches::default();
        c.add_special_legacy_point(1, 2);
        c.add_special_legacy_point(1, 2);
        assert_eq!(c.legacy_points, vec![(1, 2), (1, 2)]);
        for i in 0..(MAX_SPECIALS_LEGACY + 5) {
            c.add_special_legacy_point(i as i32, 0);
        }
        assert_eq!(c.legacy_points.len(), MAX_SPECIALS_LEGACY);
        assert_eq!(c.legacy_points[2], (0, 0));
    }

    /// Witness: `ClearLegacyCachedDraws` (`TileDrawing.cs:6613-6622`)
    /// clears the six legacy caches + the point count but NOT the two
    /// clothing dictionaries; `ClearCachedTileDraws(true)` (`:6590-6594`)
    /// clears exactly those two and does NOT chain the registry clear.
    /// UNION: also pins the per-step g29-enum bindings (the ORDER
    /// canonical driving this storage).
    #[test]
    fn w4_g32_the_clear_split_between_legacy_and_solid() {
        let mut c = LegacyCaches::default();
        for set in [
            &mut c.chest, &mut c.training_dummy, &mut c.food_platter,
            &mut c.item_frame, &mut c.dead_cells_jar, &mut c.weapon_rack,
            &mut c.display_doll, &mut c.hat_rack,
        ] {
            set.insert((3, 4));
        }
        c.add_special_legacy_point(3, 4);
        c.clear_legacy_cached_draws();
        assert!(c.legacy_points.is_empty());
        assert!(c.chest.is_empty() && c.training_dummy.is_empty() && c.food_platter.is_empty());
        assert!(c.item_frame.is_empty() && c.dead_cells_jar.is_empty() && c.weapon_rack.is_empty());
        assert!(!c.display_doll.is_empty() && !c.hat_rack.is_empty(), ":6613-6622 leaves the clothing pair");
        assert!(!c.clear_cached_tile_draws(true), "solid: no registry clear");
        assert!(c.display_doll.is_empty() && c.hat_rack.is_empty());
        // The g29 per-step binding: each LayerCacheClear step clears
        // exactly its dictionary; SpecialBlockCounts is the seam flag.
        c.display_doll.insert((1, 1));
        assert!(!c.clear_layer_step(LayerCacheClear::DisplayDollTileEntityPositions));
        assert!(c.display_doll.is_empty());
        assert!(c.clear_layer_step(LayerCacheClear::SpecialBlockCounts));
    }

    /// Witness: `ClearCachedTileDraws(false)` (`TileDrawing.cs:6595-6598`)
    /// signals the `ClearSpecialBlockCounts` chain (the g30 registry seam)
    /// and leaves the clothing dictionaries alone.
    #[test]
    fn w4_g32_the_non_solid_clear_chains_the_registry() {
        let mut c = LegacyCaches::default();
        c.display_doll.insert((1, 1));
        c.hat_rack.insert((2, 2));
        assert!(c.clear_cached_tile_draws(false), "the g30 clear seam fires");
        assert!(!c.display_doll.is_empty() && !c.hat_rack.is_empty());
    }

    /// Witness: the NON-SOLID `Draw` step order (`TileDrawing.cs:465-
    /// 741`): the critter-cage reset is non-solid only, no crack steps,
    /// the legacy pass after RestartLayeredBatch, preview gated off.
    #[test]
    fn w4_g32_the_non_solid_draw_step_order() {
        assert_eq!(
            draw_steps(false, (false, false, false, false)),
            vec![
                EffectsActiveGate, PerspectivePlayer, ResetCritterCage,
                EnsureWindGridSize, ClearLegacyCachedDraws, ClearCachedTileDraws,
                QualityLightTiers, ScreenDrawArea, NewBlackHelper, MartianGlow,
                ResetPaintKey, CellLoop, BlackEndStrip, RestartLayeredBatch,
                SpecialTilesLegacy,
            ]
        );
    }

    /// Witness: the SOLID `Draw` step order (`TileDrawing.cs:730-735`):
    /// hitReplace cracks BEFORE hitTile, RestartSpriteBatch after both,
    /// then the legacy pass; the preview arm appends when all four gates
    /// pass (`:737`).
    #[test]
    fn w4_g32_the_solid_draw_step_order() {
        assert_eq!(
            draw_steps(true, (true, true, true, false)),
            vec![
                EffectsActiveGate, PerspectivePlayer,
                EnsureWindGridSize, ClearLegacyCachedDraws, ClearCachedTileDraws,
                QualityLightTiers, ScreenDrawArea, NewBlackHelper, MartianGlow,
                ResetPaintKey, CellLoop, BlackEndStrip, RestartLayeredBatch,
                TileCracksHitReplace, TileCracksHitTile, RestartSpriteBatch,
                SpecialTilesLegacy, PlacementPreview,
            ]
        );
    }

    /// Witness: the placement-preview gate (`TileDrawing.cs:737`) — all
    /// three flags required, an active capture blocks it; the legacy pass
    /// restarts the batch only with cached points (`:7895-7899`). UNION:
    /// also pins the tuple form against the g29 gate struct (the driver
    /// binds to `PlacementPreviewGate::draws`).
    #[test]
    fn w4_g32_the_preview_gate_truth_table() {
        assert!(placement_preview_draws((true, true, true, false)));
        assert!(!placement_preview_draws((true, true, true, true)), "capture blocks");
        assert!(!placement_preview_draws((false, true, true, false)));
        assert!(!placement_preview_draws((true, false, true, false)));
        assert!(!placement_preview_draws((true, true, false, false)));
        assert!(!legacy_restarts_batch(0));
        assert!(legacy_restarts_batch(1));
        // The g29 canonical gate agrees on every row of the truth table.
        for a in [false, true] {
            for b in [false, true] {
                for c in [false, true] {
                    for d in [false, true] {
                        let g = PlacementPreviewGate {
                            object_preview_active: a,
                            cursor_item_icon_enabled: b,
                            placement_preview: c,
                            capture_active: d,
                        };
                        assert_eq!(g.draws(), placement_preview_draws((a, b, c, d)));
                    }
                }
            }
        }
    }

    /// Witness: the frame-level tile phase order — drawToScreen runs
    /// NON-SOLID first (`Main.cs:56957-56988`), the render-target capture
    /// runs SOLID first (`Main.cs:57296-57305`) with NO post passes.
    #[test]
    fn w4_g32_the_frame_tile_phase_orders() {
        use FrameTileStep::*;
        assert_eq!(
            frame_tile_draw_order(),
            &[LiquidBehindTilesPass, DrawNonSolid, PostNonSolid, Waterfalls, DrawSolid, PostSolid]
        );
        assert_eq!(render_target_tile_order(), &[DrawSolid, DrawNonSolid]);
    }
}
