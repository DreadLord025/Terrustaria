//! W4-g32 — TileDrawing pass 5: the `Draw` orchestration (the head/tail
//! sequences around the per-cell loop, `TileDrawing.cs:465-533`/`:728-741`),
//! the screen-area maths (`GetScreenDrawArea`, `:6549-6586`), the per-call
//! cache clears (`ClearCachedTileDraws` `:6588-6599` + `ClearLegacyCachedDraws`
//! `:6613-6622`), the legacy special-point registry (`AddSpecialLegacyPoint`
//! `:6606-6611`, `MAX_SPECIALS_LEGACY` `:70`), `DrawBlackHelper`
//! (`DrawBlackHelper.cs:5-38`), `DrawLiquidBehindTiles` (`:444-463`),
//! `EnsureWindGridSize` (`:7773-7777`), the batch-layer constants (`:791-794`)
//! + `DrawSingleTile`'s layer pick (`:877-887`), and the **post-pass order**:
//! `PostDrawTiles` (`:414-442`) with the `Main.cs` caller sequences
//! (`:56949-56988` drawToScreen, `:63280-63295` DoDraw, `:57329-57358`
//! render targets).
//!
//! The pass family: g28 (pass 1) landed the quality-light tier rebuild
//! (`tr-render::quality_light`, `:479-495`); g30 (pass 3) landed the
//! classification switch + `TileCounterType` registry
//! (`render/special_points.rs`, `:534-723`). g29 (pass 2, the screen-area
//! loop) and g31 (pass 4) were empty claims at this branch's base — the
//! screen-area maths is ported HERE because the orchestration consumes it;
//! the family merge keep-ones whichever copy survives.
//!
//! UNWIRED (the S1-G-1e / special_tiles.rs pattern): the `draw_tiles` seam
//! (`render/tiles.rs` + `app/frame/draw.rs`) is contested by the in-flight
//! tile-draw siblings (g28-g31, d56-d60). Intended call sites at the family
//! merge:
//! * [`get_screen_draw_area`] + [`draw_loop_bounds`] — replace the fused
//!   loop's floor/ceil rect (`tiles.rs:335-341`) with C#'s truncate + clamp
//!   + the `+4`/`-2`/`+2` walk margins;
//! * [`draw_steps`] — the head/tail step order around the g30
//!   `collect_special_points` walk (the two-layer split, `Main.cs:56957-
//!   56988`);
//! * [`LegacyCaches::clear_cached_tile_draws`] — its `false` return is the
//!   g30 `SpecialPoints::clear_special_block_counts` call (the non-solid
//!   arm, `:6597`); the registry itself stays g30's;
//! * [`post_draw_tiles_passes`] + [`post_pass_bucket`] — the drain order
//!   over the g30 buckets: the pylon/trophy/lens passes (`render/pylons.rs`)
//!   and the vine strips (`render/vines.rs`) consume their bucket instead
//!   of re-scanning the view rect;
//! * [`DrawBlackHelper`] — the dark-cell merge (`tiles.rs:769+`,
//!   `merge_dark_row`) becomes the C# strip helper on `LAYER_TILES`;
//! * [`single_tile_layer`] — the per-cell `SetLayer` pick the port's single
//!   atlas pass currently folds away (the S1-G-13 layer-target family).
//!
//! Deviations (docs/ANALYSIS.md "The Draw orchestration (W4-g32 pass 5)"):
//! * `TimeLogger` (FlushLogData/DrawCallLogData, `:467-468`) has no port
//!   counterpart — profiling only, no pixels.
//! * The C# null-tile arm (`:509-514`, lazy `new Tile()` + `mapTime += 60`)
//!   has no port counterpart (the tile array is dense) — g30's note.
//! * `DebugOptions.devLightTilesCheat` (`:487-495`) / `ShowUnbreakableWall`
//!   (`:517`) have no port toggles — fed `false` like the siblings.
//! * The section-framing side effects inside `GetScreenDrawArea`
//!   (`:6574-6585`) are returned as flags for the wiring step (the port's
//!   framing runs in `tr-core::framing`, not inside the draw rect query).
//! * The fixed 1000 legacy slots (`:70`, `:100-102`) grow here instead of
//!   throwing `IndexOutOfRangeException`; the cap drops the excess (the
//!   g30 convention).
#![allow(dead_code)] // UNWIRED — see the module doc; wiring is the family merge step.

use std::collections::HashSet;
use tr_core::World;

// ---------------------------------------------------------------------------
// Batch layers + registry caps
// ---------------------------------------------------------------------------

/// `TileDrawing.Layer_LiquidBehindTiles` (`TileDrawing.cs:791`).
pub(crate) const LAYER_LIQUID_BEHIND_TILES: u32 = 0;
/// `TileDrawing.Layer_BehindTiles` (`TileDrawing.cs:792`).
pub(crate) const LAYER_BEHIND_TILES: u32 = 1;
/// `TileDrawing.Layer_Tiles` (`TileDrawing.cs:793`).
pub(crate) const LAYER_TILES: u32 = 2;
/// `TileDrawing.Layer_OverTiles` (`TileDrawing.cs:794`).
pub(crate) const LAYER_OVER_TILES: u32 = 3;

/// `MAX_SPECIALS_LEGACY` (`TileDrawing.cs:70`): the `_specialTileX/Y`
/// `new int[1000]` slot count (`:100-102`).
pub(crate) const MAX_SPECIALS_LEGACY: usize = 1000;

// ---------------------------------------------------------------------------
// GetScreenDrawArea (TileDrawing.cs:6549-6586)
// ---------------------------------------------------------------------------

/// `GetScreenDrawArea`'s outputs (`TileDrawing.cs:6549`): the draw offset
/// plus the clamped first/last tile columns/rows. The `Draw` walk pads
/// these further ([`draw_loop_bounds`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScreenDrawArea {
    /// `drawOffSet` (`:6553`): `(offScreenRange, offScreenRange)` when
    /// drawing into targets, else zero.
    pub draw_offset: [f32; 2],
    pub first_x: i32,
    pub last_x: i32,
    pub first_y: i32,
    pub last_y: i32,
    /// `Main.sectionManager.AnyUnfinishedSections` (`:6574`) — the caller
    /// ran `SectionTileFrameWithCheck` over the rect; returned for the
    /// wiring step (the port frames outside the query).
    pub needs_framing: bool,
    /// `Main.sectionManager.AnyNeedRefresh` (`:6580`) — `RefreshSections`.
    pub needs_refresh: bool,
}

/// `GetScreenDrawArea` (`TileDrawing.cs:6549-6573`), the pure rect maths.
/// `scaled_pos`/`scaled_size` are `Main.Camera.ScaledPosition/ScaledSize`
/// (`Camera.cs:8-14`: unscaled + view-matrix translation). C# `(int)f`
/// truncates toward zero — Rust `as i32` matches (saturating at the i32
/// bounds, unreachable for screen coordinates). Note the asymmetry: the
/// `-1f` sits INSIDE the first casts (`:6554`/`:6556`) while `+2`/`+5` sit
/// OUTSIDE the last casts (`:6555`/`:6557`). The four clamps (`:6558-6573`)
/// pin the rect 4 tiles inside each world edge; no first<=last guarantee.
pub(crate) fn get_screen_draw_area(
    scaled_pos: [f32; 2],
    scaled_size: [f32; 2],
    use_offscreen_range: bool,
    off_screen_range: f32,
    max_tiles_x: i32,
    max_tiles_y: i32,
    any_unfinished_sections: bool,
    any_need_refresh: bool,
) -> ScreenDrawArea {
    let off = if use_offscreen_range { off_screen_range } else { 0.0 };
    let first_x = ((scaled_pos[0] - off) / 16.0 - 1.0) as i32;
    let last_x = ((scaled_pos[0] + scaled_size[0] + off) / 16.0) as i32 + 2;
    let first_y = ((scaled_pos[1] - off) / 16.0 - 1.0) as i32;
    let last_y = ((scaled_pos[1] + scaled_size[1] + off) / 16.0) as i32 + 5;
    ScreenDrawArea {
        draw_offset: [off, off],
        first_x: first_x.max(4),
        last_x: last_x.min(max_tiles_x - 4),
        first_y: first_y.max(4),
        last_y: last_y.min(max_tiles_y - 4),
        needs_framing: any_unfinished_sections,
        needs_refresh: any_need_refresh,
    }
}

/// The `Draw` walk bounds (`TileDrawing.cs:504-506`): rows
/// `firstTileY ..< lastTileY + 4`, columns `firstTileX - 2 ..< lastTileX + 2`
/// — returned as `(x_begin, x_end_exclusive, y_begin, y_end_exclusive)`.
/// `DrawLiquidBehindTiles` (`:449-451`) and `PrepareForAreaDrawing`
/// (`:212-214`) use the same padding.
pub(crate) fn draw_loop_bounds(area: &ScreenDrawArea) -> (i32, i32, i32, i32) {
    (area.first_x - 2, area.last_x + 2, area.first_y, area.last_y + 4)
}

/// `EnsureWindGridSize` (`TileDrawing.cs:7773-7777`): re-query the area and
/// `SetSize(lastTileX - firstTileX, lastTileY - firstTileY)` — the
/// UNPADDED clamped extents, not the walk bounds. `WindGrid::set_size`
/// (`tr-core::wind`, `WindGrid.cs:25-30`) only grows.
pub(crate) fn ensure_wind_grid_size(
    area: &ScreenDrawArea,
    grid: &mut tr_core::wind::WindGrid,
) {
    grid.set_size(area.last_x - area.first_x, area.last_y - area.first_y);
}

// ---------------------------------------------------------------------------
// The per-Draw rebuilds (TileDrawing.cs:498-503, :877-887)
// ---------------------------------------------------------------------------

/// `_martianGlow` (`TileDrawing.cs:498-499`): `b = (byte)(100f + 150f *
/// Main.martianLight)`, `new Color(b, b, b, 0)`. `martianLight` is clamped
/// to [0, 1] by its update (`Main.cs:18251-18259`), so the byte cast never
/// wraps here (100..=250); the truncation still rides `as u8` semantics
/// via i32 (the g28 net40 convention).
pub(crate) fn martian_glow(martian_light: f32) -> [u8; 4] {
    let b = (100.0 + 150.0 * martian_light) as i32 as u8;
    [b, b, b, 0]
}

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
// The cache clears (TileDrawing.cs:6588-6622, :6606-6611)
// ---------------------------------------------------------------------------

/// The legacy draw caches `ClearLegacyCachedDraws`/`ClearCachedTileDraws`
/// reset (`TileDrawing.cs:82-102`). C# keeps `Dictionary<Point, int>` (TE
/// index values); the orchestration only owns the KEYS — the TE lookups
/// are the d57 furniture slice's seam. `legacy_points` is the
/// `_specialTileX/Y` pair (`:100-102`) drained by `DrawSpecialTilesLegacy`
/// (`:7900-7907`).
#[derive(Default)]
pub(crate) struct LegacyCaches {
    /// `_chestPositions` (`:96`) — filled by `CacheSpecialDraws_Part2`
    /// (`:4300-4303`, the d58 slice).
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
        self.chest.clear();
        self.training_dummy.clear();
        self.food_platter.clear();
        self.item_frame.clear();
        self.dead_cells_jar.clear();
        self.weapon_rack.clear();
        self.legacy_points.clear();
    }

    /// `ClearCachedTileDraws` (`TileDrawing.cs:6588-6599`): the solid arm
    /// clears the display-doll + hat-rack dictionaries; the non-solid arm
    /// calls `ClearSpecialBlockCounts` (`:6597`) — the g30 registry's
    /// clear, which this module cannot reach UNWIRED. Returns `true` when
    /// the caller must run the g30 `clear_special_block_counts` seam.
    pub fn clear_cached_tile_draws(&mut self, solid_layer: bool) -> bool {
        if solid_layer {
            self.display_doll.clear();
            self.hat_rack.clear();
            false
        } else {
            true
        }
    }
}

// ---------------------------------------------------------------------------
// DrawBlackHelper (DrawBlackHelper.cs:5-38)
// ---------------------------------------------------------------------------

/// `DrawBlackHelper` (`DrawBlackHelper.cs:5-38`): merges consecutive
/// same-row black cells into one strip quad. The flush returns the quad
/// `[x, y, w, h]` in screen space (`:34-36`: `(startX << 4, y << 4) -
/// screenPosition + drawOffset`, width `(endX - startX) << 4`, height 16)
/// for the wiring step to push on [`LAYER_TILES`] with `Color.Black`.
/// Faithful quirk: `EndStrip` does NOT reset the strip — calling it twice
/// emits the same quad twice (the C# struct leaves `startX`/`endX` alone,
/// `:30-37`); `Draw` constructs a fresh helper per call (`:497`).
pub(crate) struct DrawBlackHelper {
    /// The ctor's `layer` (`:5`) — `Draw` passes `Layer_Tiles` (`:497`).
    pub layer: u32,
    pub draw_offset: [f32; 2],
    y: i32,
    start_x: i32,
    end_x: i32,
}

impl DrawBlackHelper {
    /// The primary ctor (`DrawBlackHelper.cs:5`): `y = startX = endX = 0`
    /// (`:11-15`).
    pub fn new(layer: u32, draw_offset: [f32; 2]) -> Self {
        Self { layer, draw_offset, y: 0, start_x: 0, end_x: 0 }
    }

    /// `DrawBlack` (`DrawBlackHelper.cs:17-28`): extend the strip when the
    /// cell continues the current row (`y == this.y && x == endX`),
    /// else flush and start a new strip. Returns the flushed quad, if any.
    pub fn draw_black(&mut self, x: i32, y: i32, screen_pos: [f32; 2]) -> Option<[f32; 4]> {
        if y == self.y && x == self.end_x {
            self.end_x += 1;
            return None;
        }
        let flushed = self.end_strip(screen_pos);
        self.y = y;
        self.start_x = x;
        self.end_x = x + 1;
        flushed
    }

    /// `EndStrip` (`DrawBlackHelper.cs:30-37`): emit the pending strip iff
    /// `startX != endX`. No state reset (see the struct doc).
    pub fn end_strip(&mut self, screen_pos: [f32; 2]) -> Option<[f32; 4]> {
        if self.start_x == self.end_x {
            return None;
        }
        Some([
            ((self.start_x << 4) as f32) - screen_pos[0] + self.draw_offset[0],
            ((self.y << 4) as f32) - screen_pos[1] + self.draw_offset[1],
            ((self.end_x - self.start_x) << 4) as f32,
            16.0,
        ])
    }
}

// ---------------------------------------------------------------------------
// The Draw head/tail step order (TileDrawing.cs:465-533, :728-741)
// ---------------------------------------------------------------------------

/// One orchestration step of a `Draw(solidLayer, ..)` call, in C# order.
/// The per-cell classification (g30) and the single-tile pipeline live
/// inside [`DrawStep::CellLoop`]; the quality tiers inside
/// [`DrawStep::QualityLightTiers`] are the g28 seam.
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
    /// `:496` — [`get_screen_draw_area`] with `!Main.drawToScreen`.
    ScreenDrawArea,
    /// `:497` — `new DrawBlackHelper(Layer_Tiles, drawOffSet)`.
    NewBlackHelper,
    /// `:498-499` — [`martian_glow`].
    MartianGlow,
    /// `:500-503` — `_lastPaintLookupKey.TileType = -1` (the paint cache
    /// key reset; the paint system itself is unported).
    ResetPaintKey,
    /// `:504-727` — the padded row-major walk ([`draw_loop_bounds`]):
    /// per-cell layer ops ([`cell_batch_layers`]), texture load (`:530-
    /// 533`), the g30 classification, `DrawSingleTile` (`:724`).
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
/// (`:467-468`) are profiling-only — omitted (module doc).
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
/// && Main.placementPreview && !CaptureManager.Instance.Active`. The draw
/// itself (`TileObject.DrawPreview`, `:740`, opacity =
/// `GetPlacementPreviewOpacity()`) is the wiring step.
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
// DrawLiquidBehindTiles (TileDrawing.cs:444-463)
// ---------------------------------------------------------------------------

/// The `DrawLiquidBehindTiles` cell walk (`TileDrawing.cs:449-458`): the
/// same padded rect as `Draw`, but the ONLY gate is `tile != null` — no
/// `active()`, no layer match (inactive cells still draw their behind-
/// liquid). Every cell sets layer 0 (`:456`) and calls
/// `DrawTile_LiquidBehindTile(solidLayer: false, ..)` (`:457`, the ported
/// `render/liquids.rs::draw_tile_liquid_behind`). The head/tail are
/// `Main.tileBatch.Restart()` (`:446`) / `End()` (`:461`). The dense port
/// array maps `tile != null` to an in-bounds read.
pub(crate) fn draw_liquid_behind_tiles_cells(world: &World, area: &ScreenDrawArea) -> Vec<(i32, i32)> {
    let (x0, x1, y0, y1) = draw_loop_bounds(area);
    let mut cells = Vec::new();
    for y in y0..y1 {
        for x in x0..x1 {
            if world.tile(x, y).is_some() {
                cells.push((x, y));
            }
        }
    }
    cells
}

// ---------------------------------------------------------------------------
// PostDrawTiles + the caller order (TileDrawing.cs:414-442, Main.cs)
// ---------------------------------------------------------------------------

/// One `PostDrawTiles` pass (`TileDrawing.cs:414-442`), in C# call order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PostPass {
    /// `:421` — `DrawMultiTileVines` (`:8829`).
    MultiTileVines,
    /// `:422` — `DrawMultiTileGrass` (`:8738`).
    MultiTileGrass,
    /// `:423` — `DrawVoidLenses` (`:8708`).
    VoidLenses,
    /// `:424` — `DrawTeleportationPylons` (`:8639`).
    TeleportationPylons,
    /// `:425` — `DrawMasterTrophies` (`:8597`).
    MasterTrophies,
    /// `:426` — `DrawGrass` (`:8326`).
    Grass,
    /// `:427` — `DrawAnyDirectionalGrass` (`:8383`).
    AnyDirectionalGrass,
    /// `:428` — `DrawTrees` (`:8052`).
    Trees,
    /// `:429` — `DrawVines` (`:8890`).
    Vines,
    /// `:430` — `DrawReverseVines` (`:8903`).
    ReverseVines,
    /// `:433` — `_natureRenderer.DrawAfterAllObjects(beginner)` (the
    /// LitNature shader flush, after `spriteBatch.End()` `:431`).
    NatureAfterAll,
    /// `:438` — `DrawEntities_HatRacks` (`:8039`), SOLID layer, FIRST.
    HatRacks,
    /// `:439` — `DrawEntities_DisplayDolls` (`:8026`), SOLID layer, SECOND.
    DisplayDolls,
}

/// `PostDrawTiles(solidLayer)` (`TileDrawing.cs:414-442`): the non-solid
/// arm drains the ten nature/object passes in this exact order inside one
/// Deferred/AlphaBlend batch (`:419-431`), then the nature flush (`:433`);
/// the solid arm draws ONLY the two clothing TE passes, hat racks BEFORE
/// display dolls (`:438-439`), each in its own Immediate-mode batch
/// (`:8028`/`:8041`).
pub(crate) fn post_draw_tiles_passes(solid_layer: bool) -> &'static [PostPass] {
    use PostPass::*;
    if solid_layer {
        &[HatRacks, DisplayDolls]
    } else {
        &[
            MultiTileVines,
            MultiTileGrass,
            VoidLenses,
            TeleportationPylons,
            MasterTrophies,
            Grass,
            AnyDirectionalGrass,
            Trees,
            Vines,
            ReverseVines,
            NatureAfterAll,
        ]
    }
}

/// The registry bucket each post pass drains — the `int num = N` head of
/// each pass (`TileDrawing.cs:8056`/`:8330`/`:8387`/`:8599`/`:8641`/
/// `:8710`/`:8742`/`:8833`/`:8894`/`:8907`). The indices are the g30
/// `TileCounterType` discriminants (`:21-35`); spelled as literals here so
/// this module compiles without the g30 branch — the family merge
/// keep-ones the enum. `None` = no bucket (the nature flush + the two TE
/// passes iterate their own dictionaries).
pub(crate) fn post_pass_bucket(pass: PostPass) -> Option<usize> {
    use PostPass::*;
    match pass {
        Trees => Some(0),
        Grass => Some(1),
        MultiTileGrass => Some(2),
        MultiTileVines => Some(3),
        Vines => Some(4),
        VoidLenses => Some(6),
        ReverseVines => Some(7),
        TeleportationPylons => Some(8),
        MasterTrophies => Some(9),
        AnyDirectionalGrass => Some(10),
        NatureAfterAll | HatRacks | DisplayDolls => None,
    }
}

/// One tile-draw phase of the drawToScreen frame (`Main.cs:56949-56988` /
/// the `DoDraw` path `:63280-63295`): NON-SOLID first, its post passes,
/// the waterfalls, then SOLID and its post passes.
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
pub(crate) fn render_target_tile_order() -> &'static [FrameTileStep] {
    use FrameTileStep::*;
    &[DrawSolid, DrawNonSolid]
}

