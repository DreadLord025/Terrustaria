//! W4-g29 — TileDrawing pass 2: the `Draw` ORCHESTRATION structure
//! (`TileDrawing.cs:465-742` around the per-cell body), the screen-area
//! probe (`GetScreenDrawArea` `:6549-6586`), the loop bounds (`:504-506`),
//! the black-strip coalescer (`DrawBlackHelper.cs:5-38`), the cache-clear
//! order (`ClearLegacyCachedDraws` `:6613-6622`, `ClearCachedTileDraws`
//! `:6588-6599`), the per-layer epilogue (`:728-741`), `PostDrawTiles`'
//! pass order (`:414-442`) and the frame-level two-layer ordering
//! (`Main.cs:63265-63301` + the render-target path `:57329-57368`).
//! Research/09 pass row 2: the port has ONE fused row-major loop
//! (`render/tiles.rs::draw_tiles`) instead of C#'s two layer passes with
//! the prologue/epilogue scaffolding — this module ports the MISSING
//! orchestration half.
//!
//! Lane split (the pass family): g28 (pass 1) owns the quality-light tier
//! maths (`tr_render::quality_light`, UNWIRED on its branch); g30 (pass 3)
//! owns the classification switch + special-point registry
//! (`render/special_points.rs`, UNWIRED on its branch); g31/g32 (passes
//! 4-5) own the deferred bucket drains; f15 owns the glow-texture bind.
//! This module owns the STRUCTURE that sequences them — it computes the
//! area/bounds/black strips/order itself and delegates the tier rebuild,
//! the per-cell classification and every draw call through
//! [`TilePassSeams`]. At the family merge the seams bind to the sibling
//! modules; nothing here imports them (they are not on main yet).
//!
//! UNWIRED (the S1-G-1e / `special_tiles.rs:35` pattern): the `draw_tiles`
//! seam is contested by the in-flight pass siblings. Intended call sites
//! at the family merge:
//! * `render/tiles.rs::draw_tiles` splits into `draw_layer(solid: true)`
//!   + `draw_layer(solid: false)` driven by [`walls_tiles_npcs_order`];
//!   the G6 cached range keeps feeding the camera inputs — the +4/-2
//!   margins here are C#'s, the cached-range key is untouched.
//! * [`TilePassSeams::rebuild_quality_light_tiers`] binds to
//!   `tr_render::quality_light::quality_light_tiers` (g28).
//! * [`TilePassSeams::classify_and_draw_cell`] binds to g30's
//!   `classify_tile` + `apply_cell_class` + the `DrawSingleTile` port;
//!   g30's `collect_special_points` driver then loses its own loop/gate
//!   (this module's `:517` gate is the same C# line) and keeps the
//!   per-cell half.
//! * [`TilePassSeams::post_draw_pass`] binds to the g31/g32 drains +
//!   the already-ported `pylons.rs`/`vines.rs`/`foliage.rs` passes.
//!
//! Deviations (recorded in docs/ANALYSIS.md "The Draw orchestration
//! (W4-g29 pass 2)"):
//! * The C# null-tile arm (`:509-514`, lazy `new Tile()` + `mapTime +=
//!   60`) has no port counterpart — the tile array is dense; an
//!   out-of-bounds cell is skipped (the g30 convention).
//! * `GetScreenDrawArea`'s section-framing side effects (`:6574-6585`,
//!   `SectionTileFrameWithCheck`/`RefreshSections`) are the caller's
//!   concern — the port has no `sectionManager`; the pure rect maths is
//!   ported here, the framing seam stays with the framing family.
//! * TimeLogger timestamps / draw-call counters (`:467-468`, `:728-729`
//!   bookkeeping, `TileDrawingBase.cs:25-50`) are profiler-only — the
//!   batch restarts are kept as seam calls, the counters are dropped.
//! * `DebugOptions.ShowUnbreakableWall` (`:517`, wall 350) and
//!   `hideTiles`/`hideTiles2` have no port toggles — fed `false`/`true`
//!   respectively by the caller, like g28's `devLightTilesCheat`.
//! * The legacy special-point cap (`MAX_SPECIALS_LEGACY` 1000, `:70`,
//!   `:100-102`) drops the excess instead of throwing
//!   `IndexOutOfRangeException` (the g30 9000-cap convention).
#![allow(dead_code)] // UNWIRED — see the module doc; the wiring is the pass-family merge step.

use tr_core::World;

// ---------------------------------------------------------------------------
// Layer constants — TileDrawing.cs:60-66, static ctor :789-794
// ---------------------------------------------------------------------------

/// `TileDrawing.Layer_LiquidBehindTiles` (`TileDrawing.cs:791`).
pub(crate) const LAYER_LIQUID_BEHIND_TILES: u32 = 0;
/// `TileDrawing.Layer_BehindTiles` (`TileDrawing.cs:792`).
pub(crate) const LAYER_BEHIND_TILES: u32 = 1;
/// `TileDrawing.Layer_Tiles` (`TileDrawing.cs:793`) — the black helper's
/// layer (`:497`) and the per-cell tile layer (`:526`).
pub(crate) const LAYER_TILES: u32 = 2;
/// `TileDrawing.Layer_OverTiles` (`TileDrawing.cs:794`).
pub(crate) const LAYER_OVER_TILES: u32 = 3;

/// `TileDrawing.MAX_SPECIALS_LEGACY` (`TileDrawing.cs:70`) — the
/// `_specialTileX/_specialTileY` slot count (`:100-102`).
pub(crate) const MAX_SPECIALS_LEGACY: usize = 1000;

// ---------------------------------------------------------------------------
// GetScreenDrawArea — TileDrawing.cs:6549-6586 (the pure rect half)
// ---------------------------------------------------------------------------

/// `GetScreenDrawArea`'s outputs (`TileDrawing.cs:6549`). `draw_offset`
/// is `(offScreenRange, offScreenRange)` when drawing into render targets
/// (`!Main.drawToScreen`), zero otherwise (`:6553`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScreenDrawArea {
    pub draw_offset: [f32; 2],
    pub first_tile_x: i32,
    pub last_tile_x: i32,
    pub first_tile_y: i32,
    pub last_tile_y: i32,
}

/// `TileDrawing.GetScreenDrawArea` (`TileDrawing.cs:6549-6573`). The
/// `(int)` casts TRUNCATE toward zero (the camera position can be
/// negative near the world edge — the clamp floors at 4 afterwards).
/// `scaled_position`/`scaled_size` are `Main.Camera.ScaledPosition` /
/// `ScaledSize`; the section-framing side effects (`:6574-6585`) are the
/// caller's seam (see the module doc).
pub(crate) fn get_screen_draw_area(
    use_offscreen_range: bool,
    scaled_position: [f32; 2],
    scaled_size: [f32; 2],
    off_screen_range: i32,
    max_tiles_x: i32,
    max_tiles_y: i32,
) -> ScreenDrawArea {
    // TileDrawing.cs:6553 — the offscreen pad, both axes.
    let pad = if use_offscreen_range {
        off_screen_range as f32
    } else {
        0.0
    };
    let draw_offset = [pad, pad];
    // TileDrawing.cs:6554-6557 — the truncating casts and the asymmetric
    // +2/+5 far-edge pads (the Draw loop adds its own +4/-2 on top).
    let mut first_tile_x = ((scaled_position[0] - pad) / 16.0 - 1.0) as i32;
    let mut last_tile_x = ((scaled_position[0] + scaled_size[0] + pad) / 16.0) as i32 + 2;
    let mut first_tile_y = ((scaled_position[1] - pad) / 16.0 - 1.0) as i32;
    let mut last_tile_y = ((scaled_position[1] + scaled_size[1] + pad) / 16.0) as i32 + 5;
    // TileDrawing.cs:6558-6573 — the four world-edge clamps (4-cell
    // border on every side; the far clamp is maxTiles - 4, NOT - 3).
    if first_tile_x < 4 {
        first_tile_x = 4;
    }
    if last_tile_x > max_tiles_x - 4 {
        last_tile_x = max_tiles_x - 4;
    }
    if first_tile_y < 4 {
        first_tile_y = 4;
    }
    if last_tile_y > max_tiles_y - 4 {
        last_tile_y = max_tiles_y - 4;
    }
    ScreenDrawArea {
        draw_offset,
        first_tile_x,
        last_tile_x,
        first_tile_y,
        last_tile_y,
    }
}

/// The `Draw` loop's padded bounds (`TileDrawing.cs:504-506`): rows
/// `firstTileY ..< lastTileY + 4`, columns `firstTileX - 2 ..< lastTileX
/// + 2` — returned as `(x0, x1_exclusive, y0, y1_exclusive)`. The SAME
/// padding feeds `PrepareForAreaDrawing` (`:212-214`) and
/// `DrawLiquidBehindTiles` (`:449-451`).
pub(crate) fn draw_loop_bounds(area: &ScreenDrawArea) -> (i32, i32, i32, i32) {
    (
        area.first_tile_x - 2,
        area.last_tile_x + 2,
        area.first_tile_y,
        area.last_tile_y + 4,
    )
}

// ---------------------------------------------------------------------------
// The martian glow byte — TileDrawing.cs:498-499
// ---------------------------------------------------------------------------

/// `Draw`'s per-call `_martianGlow` rebuild (`TileDrawing.cs:498-499`):
/// `b = (byte)(100f + 150f * Main.martianLight)`, colour `(b, b, b, 0)`.
/// The net40 unchecked byte cast WRAPS (the g28 `quality_light` finding);
/// `martianLight` is clamped to 0..1 by its updater (`Main.cs:18251-
/// 18259`) so the wrap is unreachable in vanilla, but the cast is spelled
/// out the same way for fidelity.
pub(crate) fn martian_glow_byte(martian_light: f32) -> u8 {
    ((100.0f32 + 150.0f32 * martian_light) as i32) as u8
}

// ---------------------------------------------------------------------------
// DrawBlackHelper — DrawBlackHelper.cs:5-38
// ---------------------------------------------------------------------------

/// One coalesced black strip: the world-pixel rect `EndStrip` draws
/// (`DrawBlackHelper.cs:34-36`): position `(startX << 4, y << 4) -
/// screenPosition + drawOffset`, size `((endX - startX) << 4, 16)`,
/// `Color.Black` on the helper's layer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BlackStrip {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub layer: u32,
}

/// `DrawBlackHelper` (`DrawBlackHelper.cs:5-38`) — the run-length
/// coalescer for unlit invisible cells (`DrawSingleTile`'s `:988-990`
/// arm feeds it). A strip extends ONLY when the next cell is the same
/// row and exactly `endX` (`:19`); anything else ends the strip and
/// starts a new one. The port returns the strip instead of drawing it —
/// the wiring step pushes it through the sprite pipeline.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DrawBlackHelper {
    layer: u32,
    draw_offset: [f32; 2],
    y: i32,
    start_x: i32,
    end_x: i32,
}

impl DrawBlackHelper {
    /// The primary ctor (`DrawBlackHelper.cs:5`, `TileDrawing.cs:497`):
    /// `y = startX = endX = 0` — an empty strip (`startX == endX`).
    pub(crate) fn new(layer: u32, draw_offset: [f32; 2]) -> Self {
        Self {
            layer,
            draw_offset,
            y: 0,
            start_x: 0,
            end_x: 0,
        }
    }

    /// `DrawBlack` (`DrawBlackHelper.cs:17-28`). Returns the strip the
    /// C# would flush via the internal `EndStrip` when the run breaks
    /// (`None` when the run extends or nothing was pending).
    pub(crate) fn draw_black(
        &mut self,
        x: i32,
        y: i32,
        screen_position: [f32; 2],
    ) -> Option<BlackStrip> {
        // DrawBlackHelper.cs:19-23 — the same-row contiguous extension.
        if y == self.y && x == self.end_x {
            self.end_x += 1;
            return None;
        }
        // DrawBlackHelper.cs:24-27 — flush the old run, start the new.
        let flushed = self.end_strip(screen_position);
        self.y = y;
        self.start_x = x;
        self.end_x = x + 1;
        flushed
    }

    /// `EndStrip` (`DrawBlackHelper.cs:30-38`): draws only when
    /// `startX != endX`; does NOT reset the run (the C# leaves the
    /// fields — a second `EndStrip` re-emits the same strip; the Draw
    /// epilogue calls it once, `:728`).
    pub(crate) fn end_strip(&mut self, screen_position: [f32; 2]) -> Option<BlackStrip> {
        if self.start_x == self.end_x {
            return None;
        }
        // DrawBlackHelper.cs:34 — the i32 shifts BEFORE the float math.
        Some(BlackStrip {
            position: [
                (self.start_x << 4) as f32 - screen_position[0] + self.draw_offset[0],
                (self.y << 4) as f32 - screen_position[1] + self.draw_offset[1],
            ],
            size: [((self.end_x - self.start_x) << 4) as f32, 16.0],
            layer: self.layer,
        })
    }
}

// ---------------------------------------------------------------------------
// The cache-clear order — TileDrawing.cs:6588-6622
// ---------------------------------------------------------------------------

/// `ClearLegacyCachedDraws`'s steps in C# order (`TileDrawing.cs:6613-
/// 6622`). Called at the head of EVERY `Draw` pass (`:477`), both layers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LegacyCacheClear {
    ChestPositions,
    TrainingDummyTileEntityPositions,
    FoodPlatterTileEntityPositions,
    ItemFrameTileEntityPositions,
    DeadCellsDisplayJarTileEntityPositions,
    WeaponRackTileEntityPositions,
    /// `_specialTilesCount = 0` (`:6621`) — the legacy point count.
    SpecialTilesCount,
}

/// The exact `ClearLegacyCachedDraws` order (`TileDrawing.cs:6615-6621`).
pub(crate) const LEGACY_CLEAR_ORDER: [LegacyCacheClear; 7] = [
    LegacyCacheClear::ChestPositions,
    LegacyCacheClear::TrainingDummyTileEntityPositions,
    LegacyCacheClear::FoodPlatterTileEntityPositions,
    LegacyCacheClear::ItemFrameTileEntityPositions,
    LegacyCacheClear::DeadCellsDisplayJarTileEntityPositions,
    LegacyCacheClear::WeaponRackTileEntityPositions,
    LegacyCacheClear::SpecialTilesCount,
];

/// `ClearCachedTileDraws(solidLayer)` (`TileDrawing.cs:6588-6599`): the
/// SOLID pass clears the two tile-entity dictionaries; the NON-SOLID
/// pass clears the special-point registry (`ClearSpecialBlockCounts`,
/// g30 owns the bucket order — BiomeGrass 5 survives).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LayerCacheClear {
    DisplayDollTileEntityPositions,
    HatRackTileEntityPositions,
    /// `ClearSpecialBlockCounts()` (`:6597`) — the g30 registry clear.
    SpecialBlockCounts,
}

/// The per-layer clear list (`TileDrawing.cs:6590-6598`).
pub(crate) fn layer_cache_clears(solid_layer: bool) -> &'static [LayerCacheClear] {
    if solid_layer {
        &[
            LayerCacheClear::DisplayDollTileEntityPositions,
            LayerCacheClear::HatRackTileEntityPositions,
        ]
    } else {
        &[LayerCacheClear::SpecialBlockCounts]
    }
}

/// `AddSpecialLegacyPoint` (`TileDrawing.cs:6606-6611`) + the
/// `_specialTileX/_specialTileY` arrays (`:100-102`). C# indexes a fixed
/// 1000-slot array and would throw past the cap; the port drops the
/// excess (the g30 convention, see the module doc).
#[derive(Default)]
pub(crate) struct LegacySpecialPoints {
    pub points: Vec<(i32, i32)>,
}

impl LegacySpecialPoints {
    pub(crate) fn add(&mut self, x: i32, y: i32) {
        if self.points.len() >= MAX_SPECIALS_LEGACY {
            return;
        }
        self.points.push((x, y));
    }

    /// `_specialTilesCount = 0` (`:6621`) — the count reset, not the
    /// arrays (C# keeps the stale slots; the port truncates).
    pub(crate) fn clear(&mut self) {
        self.points.clear();
    }
}

// ---------------------------------------------------------------------------
// PostDrawTiles — TileDrawing.cs:414-442
// ---------------------------------------------------------------------------

/// `PostDrawTiles(false)`'s pass order (`TileDrawing.cs:421-433`) and
/// `PostDrawTiles(true)`'s (`:438-439`). The non-solid list runs inside
/// one `SpriteBatchBeginner` scope (`:419-420`), ends the batch (`:431`)
/// and THEN runs the nature renderer's after-all-objects hook (`:433`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PostDrawStep {
    // --- non-solid, in order (:421-430) ---
    DrawMultiTileVines,
    DrawMultiTileGrass,
    DrawVoidLenses,
    DrawTeleportationPylons,
    DrawMasterTrophies,
    DrawGrass,
    DrawAnyDirectionalGrass,
    DrawTrees,
    DrawVines,
    DrawReverseVines,
    /// `Main.spriteBatch.End()` (`:431`).
    EndBatch,
    /// `_natureRenderer.DrawAfterAllObjects(beginner)` (`:433`).
    NatureAfterAllObjects,
    // --- solid, in order (:438-439) ---
    DrawEntitiesHatRacks,
    DrawEntitiesDisplayDolls,
}

/// The non-solid `PostDrawTiles` order (`TileDrawing.cs:421-433`).
pub(crate) const POST_DRAW_NON_SOLID: [PostDrawStep; 12] = [
    PostDrawStep::DrawMultiTileVines,
    PostDrawStep::DrawMultiTileGrass,
    PostDrawStep::DrawVoidLenses,
    PostDrawStep::DrawTeleportationPylons,
    PostDrawStep::DrawMasterTrophies,
    PostDrawStep::DrawGrass,
    PostDrawStep::DrawAnyDirectionalGrass,
    PostDrawStep::DrawTrees,
    PostDrawStep::DrawVines,
    PostDrawStep::DrawReverseVines,
    PostDrawStep::EndBatch,
    PostDrawStep::NatureAfterAllObjects,
];

/// The solid `PostDrawTiles` order (`TileDrawing.cs:438-439`) — hat
/// racks BEFORE display dolls.
pub(crate) const POST_DRAW_SOLID: [PostDrawStep; 2] = [
    PostDrawStep::DrawEntitiesHatRacks,
    PostDrawStep::DrawEntitiesDisplayDolls,
];

// ---------------------------------------------------------------------------
// The placement-preview tail — TileDrawing.cs:737-741 + Player.cs:43389-43400
// ---------------------------------------------------------------------------

/// The `Draw` epilogue's object-preview gate inputs (`TileDrawing.cs:737`):
/// `TileObject.objectPreview.Active && Main.LocalPlayer.cursorItemIconEnabled
/// && Main.placementPreview && !CaptureManager.Instance.Active`.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PlacementPreviewGate {
    pub object_preview_active: bool,
    pub cursor_item_icon_enabled: bool,
    pub placement_preview: bool,
    pub capture_active: bool,
}

impl PlacementPreviewGate {
    /// The four-condition gate (`TileDrawing.cs:737`).
    pub(crate) fn draws(&self) -> bool {
        self.object_preview_active
            && self.cursor_item_icon_enabled
            && self.placement_preview
            && !self.capture_active
    }
}

/// `Player.GetPlacementPreviewOpacity` (`Player.cs:43389-43400`): 1 for
/// non-kites, 0 for a held-style kite, else the clamped 0..30-frame
/// fade `Utils.Remap(frames, 0, 30, 1, 0)` (`Utils.cs:345-348` →
/// `GetLerpValue` clamped `:315-343` → `MathHelper.Lerp(1, 0, t)`).
/// `frames` is `framesInMovementForPlacementPreview`, clamped 0..33 by
/// its updater (`Player.cs:43407`) — at 33 the fade is already 0.
pub(crate) fn placement_preview_opacity(
    held_item_is_kite: bool,
    held_item_hold_style: i32,
    frames_in_movement: i32,
) -> f32 {
    // Player.cs:43391-43394 — ItemID.Sets.IsAKite[HeldItem.type].
    if !held_item_is_kite {
        return 1.0;
    }
    // Player.cs:43395-43398 — holdStyle != 0 hides the preview outright.
    if held_item_hold_style != 0 {
        return 0.0;
    }
    // Utils.cs:345-348 + :315-343 — the clamped lerp, then Lerp(1, 0, t)
    // = 1 + (0 - 1) * t (MathHelper.Lerp is value1 + (value2-value1)*t).
    let from = 0.0f32;
    let to = 30.0f32;
    let t = frames_in_movement as f32;
    let amount = if t < from {
        0.0
    } else if t > to {
        1.0
    } else {
        (t - from) / (to - from)
    };
    1.0 + (0.0 - 1.0) * amount
}

// ---------------------------------------------------------------------------
// The frame-level two-layer order — Main.cs:63265-63301, :57329-57368
// ---------------------------------------------------------------------------

/// `Main.DoDraw_WallsTilesNPCs`'s tile-relevant skeleton
/// (`Main.cs:63265-63301`). The NON-SOLID layer draws FIRST in the
/// draw-to-screen path (`:63280` before `:63295`); each `DoDraw_Tiles_*`
/// internally runs `DrawTiles(layer)` then `DrawTileEntities(layer)` =
/// `PostDrawTiles(layer)` (`:63322-63382`, `:21758-21761`). The
/// behind-tiles NPC pass flips around the solid layer on
/// `detectCreature` (`:63290-63299`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FrameTileStep {
    /// `DoDraw_WallsAndBlacks` (`:63272`).
    WallsAndBlacks,
    /// `DrawCachedNPCs(DrawCacheNPCsBehindNonSolidTiles, behindTiles:
    /// true)` (`:63279`).
    CachedNpcsBehindNonSolidTiles,
    /// `DoDraw_Tiles_NonSolid` (`:63280`) = `DrawTiles(false)` +
    /// `PostDrawTiles(false)` (`:63358-63382`).
    TilesNonSolid,
    /// `DoDraw_Waterfalls` (`:63281`).
    Waterfalls,
    /// `DoDraw_DrawNPCsBehindTiles` (`:63293`/`:63298`) — BEFORE the
    /// solid layer when `!detectCreature`, AFTER when `detectCreature`.
    NpcsBehindTiles,
    /// `DoDraw_Tiles_Solid` (`:63295`) = `DrawTiles(true)` +
    /// `PostDrawTiles(true)` + the fresh HitTile animations
    /// (`:63322-63356`).
    TilesSolid,
    /// `DrawPlayers_BehindNPCs` (`:63300`).
    PlayersBehindNpcs,
    /// `DoDraw_DrawNPCsOverTiles` (`:63301`).
    NpcsOverTiles,
}

/// The draw-to-screen tile ordering (`Main.cs:63272-63301`), with the
/// `detectCreature` flip (`:63290-63299`). The WoF/back-gore/Moonlord
/// steps between the walls and the cached NPCs (`:63274-63277`) are
/// entity-family passes outside this lane — they keep their existing
/// port sites; the skeleton pins only the tile-relative order.
pub(crate) fn walls_tiles_npcs_order(detect_creature: bool) -> Vec<FrameTileStep> {
    let mut steps = vec![
        FrameTileStep::WallsAndBlacks,
        FrameTileStep::CachedNpcsBehindNonSolidTiles,
        FrameTileStep::TilesNonSolid,
        FrameTileStep::Waterfalls,
    ];
    if !detect_creature {
        steps.push(FrameTileStep::NpcsBehindTiles);
    }
    steps.push(FrameTileStep::TilesSolid);
    if detect_creature {
        steps.push(FrameTileStep::NpcsBehindTiles);
    }
    steps.push(FrameTileStep::PlayersBehindNpcs);
    steps.push(FrameTileStep::NpcsOverTiles);
    steps
}

/// The render-target path's layer order (`Main.cs:57296-57306`):
/// `tileTarget` (SOLID, `RenderTiles` `:57329-57348`) refreshes BEFORE
/// `tile2Target` (non-solid, `RenderTiles2` `:57350-57368`) — the
/// OPPOSITE of the draw-to-screen composite order. Returns
/// `(first_solid_layer, hide_tiles_gate, hide_tiles2_gate)` witnesses:
/// the solid target is gated on `!DebugOptions.hideTiles` (`:57334`),
/// the non-solid on `!DebugOptions.hideTiles2` (`:57355`).
pub(crate) fn render_target_layer_order() -> [bool; 2] {
    // Main.cs:57296-57297 / :57301-57305 — tileTarget then tile2Target.
    [true, false]
}

// ---------------------------------------------------------------------------
// The Draw orchestration driver — TileDrawing.cs:465-742
// ---------------------------------------------------------------------------

/// The per-layer `Draw` inputs the orchestration reads (the C# statics /
/// fields resolved at the call head). The camera fields are
/// `Main.Camera.ScaledPosition`/`ScaledSize`; `screen_position` is
/// `Main.screenPosition` (the black strips' anchor).
#[derive(Clone, Copy, Debug)]
pub(crate) struct DrawLayerInputs {
    pub solid_layer: bool,
    /// `intoRenderTargets` (`:465`) — C#'s `Draw` body never reads it
    /// (only `Main.DrawTiles`' `Begin` does, `Main.cs:21825`); recorded
    /// for the seam, drives `useOffscreenRange` via `draw_to_screen`.
    pub into_render_targets: bool,
    pub gfx_quality: f32,
    pub dev_light_tiles_cheat: bool,
    pub martian_light: f32,
    /// `Main.drawToScreen` — `GetScreenDrawArea(!drawToScreen)` (`:496`).
    pub draw_to_screen: bool,
    pub off_screen_range: i32,
    pub scaled_position: [f32; 2],
    pub scaled_size: [f32; 2],
    pub screen_position: [f32; 2],
    /// `FocusHelper.AllowTileDrawingToEmitEffects` (`:469`,
    /// `FocusHelper.cs:27` = `!WorldUpdateStepper.DrawnThisUpdate`) —
    /// the particle-emission gate the `DrawSingleTile` port reads.
    pub allow_emit_effects: bool,
    /// `ShouldSwayInWind`'s enabled input (g30's `sway_enabled`).
    pub sway_enabled: bool,
    pub preview_gate: PlacementPreviewGate,
    pub preview_is_kite: bool,
    pub preview_hold_style: i32,
    pub preview_frames_in_movement: i32,
}

/// Which `HitTile` the solid epilogue's crack pass draws
/// (`TileDrawing.cs:732-733` — hitReplace FIRST, then hitTile).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CrackSite {
    HitReplace,
    HitTile,
}

/// The orchestration's delegation seam. Every method is one C# call the
/// structure sequences; the family merge binds them to the sibling
/// ports (see the module doc). The driver itself owns only the order,
/// the bounds, the gates and the black-strip coalescing.
pub(crate) trait TilePassSeams {
    /// `EnsureWindGridSize` (`:476`, `:7773-7777`) — the caller sizes
    /// `WindGrid::set_size(lastX - firstX, lastY - firstY)` (tr_core
    /// `wind.rs:467`) from a FRESH `GetScreenDrawArea` probe.
    fn ensure_wind_grid_size(&mut self, width: i32, height: i32);
    /// `ClearLegacyCachedDraws` (`:477`) — one call per ordered step.
    fn clear_legacy_cache(&mut self, step: LegacyCacheClear);
    /// `ClearCachedTileDraws(solidLayer)` (`:478`) — one call per step.
    fn clear_layer_cache(&mut self, step: LayerCacheClear);
    /// The tier rebuild (`:479-495`) — binds to g28's
    /// `quality_light_tiers(gfx_quality, dev_light_tiles_cheat)`.
    fn rebuild_quality_light_tiers(&mut self, gfx_quality: f32, dev_cheat: bool);
    /// `Main.critterCage = false` (`:472-475`, non-solid head only).
    fn reset_critter_cage(&mut self);
    /// `_martianGlow = new Color(b, b, b, 0)` (`:498-499`).
    fn set_martian_glow(&mut self, b: u8);
    /// `_lastPaintLookupKey.TileType = -1` (`:500-503`) — the paint
    /// cache's per-pass reset (the paint family's seam).
    fn reset_paint_lookup_key(&mut self);
    /// `Main.tileBatch.SetLayer(Layer_LiquidBehindTiles, 0)` +
    /// `DrawTile_LiquidBehindTile(solidLayer, ...)` (`:521-525`) — the
    /// SOLID-layer per-cell front step; binds to
    /// `render/liquids.rs::draw_tile_liquid_behind`.
    fn draw_tile_liquid_behind(&mut self, x: i32, y: i32);
    /// `Main.tileBatch.SetLayer(Layer_Tiles, 0)` (`:526`) + the
    /// classification switch (`:534-723`, g30) + `DrawSingleTile`
    /// (`:724`). The cell may feed the black helper (`:988-990`).
    fn classify_and_draw_cell(&mut self, x: i32, y: i32, black: &mut DrawBlackHelper);
    /// A coalesced black strip flush (`DrawBlackHelper.cs:36`).
    fn draw_black_strip(&mut self, strip: BlackStrip);
    /// `base.RestartLayeredBatch()` (`:729`, `TileDrawingBase.cs:34-40`).
    fn restart_layered_batch(&mut self);
    /// `Main.instance.DrawTileCracks(1, site)` (`:732-733`) — binds to
    /// `render/tile_cracks.rs::draw_tile_cracks`.
    fn draw_tile_cracks(&mut self, site: CrackSite);
    /// `base.RestartSpriteBatch()` (`:734`, `TileDrawingBase.cs:42-50`).
    fn restart_sprite_batch(&mut self);
    /// `DrawSpecialTilesLegacy` (`:736`) — binds to
    /// `render/special_tiles.rs` (S1-G-1e).
    fn draw_special_tiles_legacy(&mut self);
    /// `TileObject.DrawPreview(opacity, ...)` (`:740`).
    fn draw_object_preview(&mut self, opacity: f32);
    /// One `PostDrawTiles` pass (`:421-439`) — binds to the g31/g32
    /// drains + the ported pylons/vines/foliage passes.
    fn post_draw_pass(&mut self, step: PostDrawStep);
}

/// `TileDrawing.Draw(solidLayer, intoRenderTargets, waterStyleOverride)`
/// (`TileDrawing.cs:465-742`) — the orchestration WITHOUT the per-cell
/// body (the seam owns classification + `DrawSingleTile`). Returns the
/// area it walked so the caller can feed the wind grid / liquid passes.
///
/// The `:517` gate (`!active() || IsTileDrawLayerSolid(type) !=
/// solidLayer`) is applied here through `is_tile_draw_layer_solid` —
/// the caller passes g30's `special_points::is_tile_draw_layer_solid`
/// at the merge (the five `DrawTileInSolidLayer` overrides win over
/// `tileSolid`). The `ShowUnbreakableWall` debug arm has no port toggle.
pub(crate) fn draw_layer(
    world: &World,
    inputs: &DrawLayerInputs,
    is_tile_draw_layer_solid: fn(u16) -> bool,
    seams: &mut impl TilePassSeams,
) -> ScreenDrawArea {
    // TileDrawing.cs:469-470 — the emission gate + perspective player
    // are inputs the per-cell seam reads; nothing to sequence here.
    // TileDrawing.cs:472-475 — the non-solid head resets critterCage.
    if !inputs.solid_layer {
        seams.reset_critter_cage();
    }
    // TileDrawing.cs:476 — EnsureWindGridSize (:7773-7777): its OWN
    // GetScreenDrawArea probe, then SetSize(lastX-firstX, lastY-firstY).
    let wind_area = get_screen_draw_area(
        !inputs.draw_to_screen,
        inputs.scaled_position,
        inputs.scaled_size,
        inputs.off_screen_range,
        world.width,
        world.height,
    );
    seams.ensure_wind_grid_size(
        wind_area.last_tile_x - wind_area.first_tile_x,
        wind_area.last_tile_y - wind_area.first_tile_y,
    );
    // TileDrawing.cs:477 — ClearLegacyCachedDraws, every pass, in order.
    for step in LEGACY_CLEAR_ORDER {
        seams.clear_legacy_cache(step);
    }
    // TileDrawing.cs:478 — ClearCachedTileDraws(solidLayer).
    for &step in layer_cache_clears(inputs.solid_layer) {
        seams.clear_layer_cache(step);
    }
    // TileDrawing.cs:479-495 — the tier rebuild (g28's maths).
    seams.rebuild_quality_light_tiers(inputs.gfx_quality, inputs.dev_light_tiles_cheat);
    // TileDrawing.cs:496 — the loop's own area probe.
    let area = get_screen_draw_area(
        !inputs.draw_to_screen,
        inputs.scaled_position,
        inputs.scaled_size,
        inputs.off_screen_range,
        world.width,
        world.height,
    );
    // TileDrawing.cs:497 — a FRESH black helper per pass, Layer_Tiles.
    let mut black = DrawBlackHelper::new(LAYER_TILES, area.draw_offset);
    // TileDrawing.cs:498-499 — the martian glow byte.
    seams.set_martian_glow(martian_glow_byte(inputs.martian_light));
    // TileDrawing.cs:500-503 — the paint lookup key reset.
    seams.reset_paint_lookup_key();
    // TileDrawing.cs:504-506 — the padded row-major walk.
    let (x0, x1, y0, y1) = draw_loop_bounds(&area);
    for y in y0..y1 {
        for x in x0..x1 {
            // TileDrawing.cs:508-514 — the null-tile arm: the dense port
            // array skips out-of-bounds cells (see the module doc).
            let Some(tile) = world.tile(x, y) else {
                continue;
            };
            // TileDrawing.cs:517 — the active + layer gate.
            if !tile.active() || is_tile_draw_layer_solid(tile.kind) != inputs.solid_layer {
                continue;
            }
            // TileDrawing.cs:521-525 — the solid layer's liquid-behind
            // front step (SetLayer(Layer_LiquidBehindTiles, 0)).
            if inputs.solid_layer {
                seams.draw_tile_liquid_behind(x, y);
            }
            // TileDrawing.cs:526-724 — SetLayer(Layer_Tiles, 0), the
            // texture-load probe (:530-533, the atlas is preloaded in
            // the port), the classification switch and DrawSingleTile.
            seams.classify_and_draw_cell(x, y, &mut black);
        }
    }
    // TileDrawing.cs:728 — flush the trailing black run.
    if let Some(strip) = black.end_strip(inputs.screen_position) {
        seams.draw_black_strip(strip);
    }
    // TileDrawing.cs:729 — RestartLayeredBatch.
    seams.restart_layered_batch();
    // TileDrawing.cs:730-735 — the SOLID-only crack pair + restart.
    if inputs.solid_layer {
        seams.draw_tile_cracks(CrackSite::HitReplace);
        seams.draw_tile_cracks(CrackSite::HitTile);
        seams.restart_sprite_batch();
    }
    // TileDrawing.cs:736 — DrawSpecialTilesLegacy, both layers.
    seams.draw_special_tiles_legacy();
    // TileDrawing.cs:737-741 — the object-preview tail.
    if inputs.preview_gate.draws() {
        let opacity = placement_preview_opacity(
            inputs.preview_is_kite,
            inputs.preview_hold_style,
            inputs.preview_frames_in_movement,
        );
        seams.draw_object_preview(opacity);
    }
    area
}

/// `TileDrawing.DrawLiquidBehindTiles(waterStyleOverride)`
/// (`TileDrawing.cs:444-463`) — the background-water pre-pass. NOTE the
/// gate difference from `Draw`: ONLY `tile != null` (`:454`), no
/// `active()`/layer gate — every cell in the padded area gets a
/// `DrawTile_LiquidBehindTile(solidLayer: false, ...)` call (`:457`).
pub(crate) fn draw_liquid_behind_tiles(
    world: &World,
    inputs: &DrawLayerInputs,
    seams: &mut impl TilePassSeams,
) {
    let area = get_screen_draw_area(
        !inputs.draw_to_screen,
        inputs.scaled_position,
        inputs.scaled_size,
        inputs.off_screen_range,
        world.width,
        world.height,
    );
    // TileDrawing.cs:449-451 — the same +4/-2 padding as Draw.
    let (x0, x1, y0, y1) = draw_loop_bounds(&area);
    for y in y0..y1 {
        for x in x0..x1 {
            // TileDrawing.cs:453-458 — null check only; SetLayer(0u, 0).
            if world.tile(x, y).is_some() {
                seams.draw_tile_liquid_behind(x, y);
            }
        }
    }
}

/// `TileDrawing.PostDrawTiles(solidLayer)` (`TileDrawing.cs:414-442`) —
/// the deferred-pass drain order. The non-solid list's batch scope
/// (`:419-420`/`:431`) is the `EndBatch` step; the nature renderer's
/// after-all-objects hook runs AFTER it (`:433`).
pub(crate) fn post_draw_tiles(solid_layer: bool, seams: &mut impl TilePassSeams) {
    if !solid_layer {
        for step in POST_DRAW_NON_SOLID {
            seams.post_draw_pass(step);
        }
    } else {
        for step in POST_DRAW_SOLID {
            seams.post_draw_pass(step);
        }
    }
}