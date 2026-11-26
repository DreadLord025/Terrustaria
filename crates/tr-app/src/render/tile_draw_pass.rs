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
//!   (`:6574-6585`) ride out as flags on g29's `ScreenDrawArea` (the
//!   union fold; the port frames in `tr-core::framing`).
//!   (`:6574-6585`) are returned as flags for the wiring step (the port's
//!   framing runs in `tr-core::framing`, not inside the draw rect query).
//! * The fixed 1000 legacy slots (`:70`, `:100-102`) grow here instead of
//!   throwing `IndexOutOfRangeException`; the cap drops the excess (the
//!   g30 convention).
#![allow(dead_code)] // UNWIRED — see the module doc; wiring is the family merge step.

// The g29 seam's `LegacyCacheClear` + the per-layer `LayerCacheClear` stay
// bound here; the LAYER_* caps are this module's own consts below (the
// g32/batch-P canonical shape) — the duplicate re-import from tile_pass is
// dropped at the union.
use super::tile_pass::{LayerCacheClear, LegacyCacheClear};
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
pub(crate) fn ensure_wind_grid_size_for_tile_pass(
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
// LegacyCaches (TileDrawing.cs:82-102, :6588-6622) — the storage canonical
// The cache clears (TileDrawing.cs:6588-6622, :6606-6611)
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
/// [`DrawStep::QualityLightTiers`] are the g28 seam
/// (`tr_render::quality_light::quality_light_tiers`).
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
    /// `:496` — `tile_pass::get_screen_draw_area` with `!Main.drawToScreen`.
    ScreenDrawArea,
    /// `:497` — `new DrawBlackHelper(Layer_Tiles, drawOffSet)` (the g29
    /// `tile_pass::DrawBlackHelper`).
    NewBlackHelper,
    /// `:498-499` — [`martian_glow`] (+ the `[b,b,b,0]` quad, the seam binding).
    MartianGlow,
    /// `:500-503` — `_lastPaintLookupKey.TileType = -1` (the paint cache
    /// key reset; the paint system itself is unported).
    ResetPaintKey,
    /// `:504-727` — the padded row-major walk
    /// (`tile_pass::draw_loop_bounds`): per-cell layer ops
    /// ([`cell_batch_layers`]), texture load (`:530-533`), the g30
    /// classification, `DrawSingleTile` (`:724`).
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
/// (`:467-468`) are profiling-only — omitted (module doc). UNION: this is
/// the full head/tail enumeration witness (incl. `EffectsActiveGate`/
/// `PerspectivePlayer` which g29 folds into `DrawLayerInputs`); the g29
/// `draw_layer` driver is the executable canonical.
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
/// && Main.placementPreview && !CaptureManager.Instance.Active`. UNION
/// (contract §1.3): the driver binds to the g29
/// `tile_pass::PlacementPreviewGate::draws`; this tuple form stays as the
/// truth-table witness only.
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
// The tile-phase frame orders (Main.cs:56949-56988, :57296-57305) — KEEP
// ---------------------------------------------------------------------------

/// One tile-draw phase of the drawToScreen frame (`Main.cs:56949-56988` /
/// the `DoDraw` path `:63280-63295`): NON-SOLID first, its post passes,
/// the waterfalls, then SOLID and its post passes. UNION: this tile-PHASE
/// granularity complements the g29 `tile_pass::walls_tiles_npcs_order`
/// NPC-relative skeleton (its `render_target_layer_order` dropped here).
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
    use crate::render::tile_pass::PlacementPreviewGate;
    use DrawStep::*;
    use tr_core::tile::FLAG_ACTIVE;
    use PostPass::*;

    fn area(fx: i32, lx: i32, fy: i32, ly: i32) -> super::ScreenDrawArea {
        super::ScreenDrawArea {
            draw_offset: [0.0, 0.0],
            first_x: fx,
            last_x: lx,
            first_y: fy,
            last_y: ly,
            needs_framing: false,
            needs_refresh: false,
        }
    }

    /// Witness: `GetScreenDrawArea` (`TileDrawing.cs:6551-6573`) — the
    /// `-1f` inside the first casts, `+2`/`+5` outside the last, the four
    /// edge clamps, and the offscreen offset only when drawing to targets.
    #[test]
    fn w4_g32_the_screen_area_truncates_and_clamps() {
        // pos 160,160 size 160,128: first = 160/16-1 = 9; last_x =
        // (160+160)/16+2 = 22; last_y = (160+128)/16+5 = 23.
        let a = get_screen_draw_area([160.0, 160.0], [160.0, 128.0], false, 192.0, 1000, 800, false, false);
        assert_eq!((a.first_x, a.last_x, a.first_y, a.last_y), (9, 22, 9, 23));
        assert_eq!(a.draw_offset, [0.0, 0.0], "drawToScreen: no offset (:6553)");
        // Into targets: off = 192 both axes; first_x = (160-192)/16-1 =
        // -3 -> clamp 4; last_x = (160+160+192)/16+2 = 34.
        let b = get_screen_draw_area([160.0, 160.0], [160.0, 128.0], true, 192.0, 1000, 800, false, false);
        assert_eq!(b.draw_offset, [192.0, 192.0]);
        assert_eq!((b.first_x, b.last_x), (4, 34));
        // The world-edge clamps (:6562-6573): a tiny world pins last to
        // max-4 even when first > last (no reorder in C#).
        let c = get_screen_draw_area([160.0, 160.0], [160.0, 128.0], false, 0.0, 20, 20, true, true);
        assert_eq!((c.first_x, c.last_x, c.first_y, c.last_y), (9, 16, 9, 16));
        assert!(c.needs_framing && c.needs_refresh, ":6574-6585 flags ride out");
    }

    /// Witness: C# `(int)f` truncates toward ZERO, not floor
    /// (`TileDrawing.cs:6554-6557`): pos -100 + size 92 = -8 -> -0.5 ->
    /// `(int)(-0.5) = 0`, so last_x = 0+2 = 2 (floor would give 1) and
    /// last_y = 0+5 = 5. The +2/+5 sit OUTSIDE the cast.
    #[test]
    fn w4_g32_the_area_cast_truncates_toward_zero() {
        let a = get_screen_draw_area([-100.0, -100.0], [92.0, 92.0], false, 0.0, 1000, 800, false, false);
        assert_eq!(a.last_x, 2, "trunc(-0.5)+2, NOT floor(-0.5)+2");
        assert_eq!(a.last_y, 5, "trunc(-0.5)+5");
        assert_eq!(a.first_x, 4, "(-6.25-1) trunc -7 -> clamp 4");
    }

    /// Witness: the `Draw` walk pads the clamped area (`TileDrawing.cs:
    /// 504-506`): rows `firstY ..< lastY+4`, cols `firstX-2 ..< lastX+2`.
    #[test]
    fn w4_g32_the_walk_bounds_pad_the_area() {
        assert_eq!(draw_loop_bounds(&area(9, 22, 9, 23)), (7, 24, 9, 27));
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

    /// Witness: `_martianGlow` (`TileDrawing.cs:498-499`) — the byte
    /// `(byte)(100f + 150f * martianLight)` on all three channels, alpha 0.
    #[test]
    fn w4_g32_the_martian_glow_is_the_byte_ramp() {
        assert_eq!(martian_glow(0.0), [100, 100, 100, 0]);
        assert_eq!(martian_glow(0.5), [175, 175, 175, 0]);
        assert_eq!(martian_glow(1.0), [250, 250, 250, 0]);
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
    /// Witness: `AddSpecialLegacyPoint` (`TileDrawing.cs:6606-6611`)
    /// appends without dedup and honours the 1000-slot cap (`:70`).
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

    /// Witness: `DrawBlackHelper` (`DrawBlackHelper.cs:17-37`) merges
    /// consecutive same-row cells, flushes on a gap with the `<< 4` pixel
    /// maths, and `EndStrip` does NOT reset (a second call re-emits).
    #[test]
    fn w4_g32_the_black_helper_merges_strips() {
        let mut h = DrawBlackHelper::new(LAYER_TILES, [0.0, 0.0]);
        let sp = [0.0, 0.0];
        assert_eq!(h.draw_black(5, 10, sp), None, "the fresh strip is empty");
        assert_eq!(h.draw_black(6, 10, sp), None);
        assert_eq!(h.draw_black(7, 10, sp), None);
        // Gap at 8: flush 5..8 -> x 80, y 160, w 48, h 16.
        assert_eq!(h.draw_black(9, 10, sp), Some([80.0, 160.0, 48.0, 16.0]));
        assert_eq!(h.end_strip(sp), Some([144.0, 160.0, 16.0, 16.0]));
        // The C# quirk: EndStrip leaves startX/endX, so it re-emits.
        assert_eq!(h.end_strip(sp), Some([144.0, 160.0, 16.0, 16.0]));
        // A fresh helper (Draw constructs one per call, :497) emits nothing.
        assert_eq!(DrawBlackHelper::new(LAYER_TILES, [0.0, 0.0]).end_strip(sp), None);
    }

    /// Witness: the strip flush applies `- screenPosition + drawOffset`
    /// (`DrawBlackHelper.cs:34`) and a row change breaks the strip even
    /// at the same x (`:19`).
    #[test]
    fn w4_g32_the_black_strip_offset_and_row_break() {
        let mut h = DrawBlackHelper::new(LAYER_TILES, [192.0, 192.0]);
        let sp = [32.0, 16.0];
        assert_eq!(h.draw_black(5, 10, sp), None);
        // Row change at the SAME x: flush the one-cell strip.
        assert_eq!(h.draw_black(5, 11, sp), Some([240.0, 336.0, 16.0, 16.0]));
        assert_eq!(h.end_strip(sp), Some([240.0, 352.0, 16.0, 16.0]));
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
    /// restarts the batch only with cached points (`:7895-7899`).
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

    /// Witness: `DrawLiquidBehindTiles` (`TileDrawing.cs:449-458`) walks
    /// the same padded rect but gates ONLY on the tile existing — an
    /// INACTIVE cell still draws its behind-liquid; out-of-bounds rows
    /// are skipped (the dense-array stand-in for `tile != null`).
    #[test]
    fn w4_g32_the_behind_liquid_walk_includes_inactive_cells() {
        let mut world = World::new("t".into(), 8, 8);
        world.compute_column_surface();
        world.compute_chunks();
        let i = world.index(3, 4);
        world.tiles[i].flags = FLAG_ACTIVE;
        // (4,4) stays INACTIVE — it must still be listed.
        let a = area(4, 4, 4, 6); // bounds x 2..6, y 4..10; rows 8,9 OOB.
        let cells = draw_liquid_behind_tiles_cells(&world, &a);
        assert_eq!(cells.len(), 16, "4 cols x 4 in-bounds rows");
        assert!(cells.contains(&(3, 4)));
        assert!(cells.contains(&(4, 4)), "no active() gate (:454)");
        assert!(!cells.contains(&(2, 8)), "OOB row skipped");
        assert_eq!(cells[0], (2, 4), "row-major, y outer (:449-451)");
    }

    /// Witness: `PostDrawTiles(false)` (`TileDrawing.cs:421-433`) — the
    /// ten-pass drain order, NatureAfterAll LAST after the batch End.
    #[test]
    fn w4_g32_the_non_solid_post_pass_order() {
        assert_eq!(
            post_draw_tiles_passes(false),
            &[
                MultiTileVines, MultiTileGrass, VoidLenses, TeleportationPylons,
                MasterTrophies, Grass, AnyDirectionalGrass, Trees, Vines,
                ReverseVines, NatureAfterAll,
            ]
        );
    }

    /// Witness: `PostDrawTiles(true)` (`TileDrawing.cs:435-441`) — ONLY
    /// the two clothing passes, HAT RACKS FIRST.
    #[test]
    fn w4_g32_the_solid_post_pass_order() {
        assert_eq!(post_draw_tiles_passes(true), &[HatRacks, DisplayDolls]);
    }

    /// Witness: each post pass drains its `int num = N` bucket
    /// (`TileDrawing.cs:8056/:8330/:8387/:8599/:8641/:8710/:8742/:8833/
    /// :8894/:8907`) — the g30 TileCounterType discriminants (`:21-35`).
    #[test]
    fn w4_g32_the_post_pass_bucket_indices() {
        assert_eq!(post_pass_bucket(Trees), Some(0));
        assert_eq!(post_pass_bucket(Grass), Some(1));
        assert_eq!(post_pass_bucket(MultiTileGrass), Some(2));
        assert_eq!(post_pass_bucket(MultiTileVines), Some(3));
        assert_eq!(post_pass_bucket(Vines), Some(4));
        assert_eq!(post_pass_bucket(VoidLenses), Some(6));
        assert_eq!(post_pass_bucket(ReverseVines), Some(7));
        assert_eq!(post_pass_bucket(TeleportationPylons), Some(8));
        assert_eq!(post_pass_bucket(MasterTrophies), Some(9));
        assert_eq!(post_pass_bucket(AnyDirectionalGrass), Some(10));
        // BiomeGrass (5) has no post pass — never registered (g30 note).
        for p in post_draw_tiles_passes(false) {
            assert_ne!(post_pass_bucket(*p), Some(5));
        }
        assert_eq!(post_pass_bucket(NatureAfterAll), None);
        assert_eq!(post_pass_bucket(HatRacks), None);
        assert_eq!(post_pass_bucket(DisplayDolls), None);
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
