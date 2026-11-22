//! W4-g28 — the `TileDrawing.Draw` main-pass orchestration core
//! (`TileDrawing.cs:465-742`): the deferred special-point registry
//! (`TileCounterType` `:21-35`, `AddSpecialPoint` `:183-186`,
//! `ClearSpecialBlockCounts` `:388-402`, `ClearLegacyCachedDraws`
//! `:6613-6622`), the screen draw-area margins (`GetScreenDrawArea`
//! `:6549-6586`), the solid/non-solid layer classification
//! (`IsTileDrawLayerSolid` `:4469-4476`) and the frozen pass order
//! (`Draw`'s tail `:728-741`, `PostDrawTiles` `:414-442`).
//!
//! **UNWIRED module (code-only, W4-g28).** The `draw_tiles` seam is
//! contested by the in-flight tile-draw siblings (d56-d60 landed their
//! slices unwired; g29-g32 own passes 2-5). This module is the pass-1
//! ORCHESTRATION the siblings plug into: pure registry/decision state,
//! no sprite pushes. The wiring step (the family merge) drives it from
//! `render/tiles.rs::draw_tiles` in the C# order recorded below; the
//! composite order stays FROZEN per S1-G-11 (d68's `tr_render::batch`
//! layered sort — the layer constants here mirror `TileDrawing.cs:791-794`
//! and must not be reordered).
//!
//! ## The C# pass order this module pins
//!
//! `Main.DrawTiles(solidLayer)` (`Main.cs:21820-21831`) wraps
//! `TileDrawing.Draw` in `Begin`/`End`; the frame runs the NON-SOLID layer
//! FIRST (`DoDraw_WallsTilesNPCs`, `Main.cs:63280` → `DoDraw_Tiles_NonSolid`
//! `:63358-63382`), then the solid layer (`:63295` → `DoDraw_Tiles_Solid`
//! `:63322-63356`), each followed by `DrawTileEntities(solidLayer)` →
//! `PostDrawTiles` (`Main.cs:21758-21761`). The capture path repeats the
//! same order (`:56957-56988`). The order is load-bearing for the registry:
//! `ClearCachedTileDraws` clears the special counts ONLY before the
//! non-solid pass (`TileDrawing.cs:6588-6599`), so the solid pass's tree
//! points survive into `PostDrawTiles(false)` alongside the non-solid
//! points — and `PostDrawTiles(true)` (HatRacks/DisplayDolls) runs LAST.
//!
//! `Draw(solidLayer)` (`:465-742`), in order:
//! 1. quality-light tier rebuild (`:479-495`, `tr_render::quality_light`);
//! 2. `GetScreenDrawArea` (`:496`) + `DrawBlackHelper` (`:497`);
//! 3. the row-major section loop `i in firstTileY..lastTileY+4`,
//!    `j in firstTileX-2..lastTileX+2` (`:504-506`):
//!    * null tile → allocate + `mapTime += 60` (`:509-514`);
//!    * inactive / wrong-layer / unbreakable-wall-debug → skip (`:517`);
//!    * solid layer: `SetLayer(Layer_LiquidBehindTiles)` +
//!      `DrawTile_LiquidBehindTile` (`:521-525`);
//!    * `SetLayer(Layer_Tiles)` (`:526`), texture load (`:530-533`);
//!    * the classification switch (`:534-723`) — g30's
//!      `render::special_points::classify_tile` (`CellClass`) owns the arms;
//!      `continue` arms register a special point and skip the inline
//!      draw; `break` arms fall through to `DrawSingleTile`. This module
//!      drives the loop through that seam, it does not re-port the switch;
//!    * `DrawSingleTile` (`:724`);
//! 4. `drawBlackHelper.EndStrip()` + `RestartLayeredBatch` (`:728-729`);
//! 5. solid only: `DrawTileCracks(hitReplace)` then `DrawTileCracks(hitTile)`
//!    + `RestartSpriteBatch` (`:730-735`, `render/tile_cracks.rs` wired);
//! 6. `DrawSpecialTilesLegacy` (`:736`, `render/special_tiles.rs`);
//! 7. the placement preview (`:737-741`, unported — no `objectPreview`).
//!
//! `PostDrawTiles(false)` (`:414-434`) draws the deferred passes in the
//! FROZEN order [`POST_DRAW_ORDER`]: MultiTileVines → MultiTileGrass →
//! VoidLenses → TeleportationPylons → MasterTrophies → Grass →
//! AnyDirectionalGrass → Trees → Vines → ReverseVines.
//! `PostDrawTiles(true)` (`:435-441`): HatRacks then DisplayDolls.
//!
//! ## Module split inside the pass-1 family
//!
//! g30's `render::special_points` (landed `cd7f7d31` on its own branch)
//! owns the classification switch ([`CellClass`] arms), the vine crawls'
//! registry side and the per-layer collect walk. THIS module owns the
//! frame-level pieces g30 does not: the screen-area maths
//! (`GetScreenDrawArea` + the `+4/-2/+2` loop margins), the per-`Draw`
//! clear semantics (`ClearCachedTileDraws` / `ClearLegacyCachedDraws`),
//! the quality-tier rebuild call, the layer constants and the frozen
//! post-draw order. At the family merge the two registries reconcile onto
//! g30's `SpecialPoints` shape (its `single_tiles` / `liquid_drop_cells`
//! wiring witnesses); the duplicate registry below is the pre-merge
//! stand-in this module's witnesses run against.
#![allow(dead_code)] // UNWIRED — see the module doc; wiring is the family merge step.

use std::collections::HashSet;
use tr_core::tiledata as td;
use tr_core::World;

/// `TileDrawing.TileCounterType` (`TileDrawing.cs:21-35`) — the deferred
/// special-point buckets, in enum order (the array indices the C# uses).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum TileCounterType {
    Tree = 0,
    WindyGrass = 1,
    MultiTileGrass = 2,
    MultiTileVine = 3,
    Vine = 4,
    BiomeGrass = 5,
    VoidLens = 6,
    ReverseVine = 7,
    TeleportationPylon = 8,
    MasterTrophy = 9,
    AnyDirectionalGrass = 10,
}

/// `TileCounterType.Count` (`TileDrawing.cs:34`).
pub(crate) const TILE_COUNTER_COUNT: usize = 11;

/// `TileDrawing.MAX_SPECIALS` (`TileDrawing.cs:68`) — the per-bucket
/// capacity the C# allocates (`:192-195`). The port's buckets grow
/// dynamically; the constant pins the census bound only.
pub(crate) const MAX_SPECIALS: usize = 9000;

/// `TileDrawing.MAX_SPECIALS_LEGACY` (`TileDrawing.cs:70`) — the
/// `_specialTileX/Y` array size (`:100-102`).
pub(crate) const MAX_SPECIALS_LEGACY: usize = 1000;

/// The deferred special-point registry: `_specialPositions` /
/// `_specialsCount` (`TileDrawing.cs:78-80`) plus the vine-root dedup lists
/// (`_vineRootsPositions` / `_reverseVineRootsPositions`, `:118-120`) and
/// the legacy point list (`_specialTileX/Y` + `_specialTilesCount`,
/// `:98-102`). Rebuilt per `Draw` call — `ClearCachedTileDraws(false)`
/// clears the counters (`:6595-6598`), `ClearLegacyCachedDraws` clears the
/// legacy list on BOTH layers (`:477`).
#[derive(Default)]
pub(crate) struct SpecialPoints {
    /// `_specialPositions[(int)type]` in registration order (the C# array
    /// slots; the port keeps a Vec per bucket — the draw passes walk them
    /// in the same order).
    pub buckets: [Vec<(i32, i32)>; TILE_COUNTER_COUNT],
    /// `_vineRootsPositions` (`:757-760`) — the Vine-bucket dedup set.
    pub vine_roots_seen: HashSet<(i32, i32)>,
    /// `_reverseVineRootsPositions` (`:777-780`) — the ReverseVine dedup.
    pub reverse_vine_roots_seen: HashSet<(i32, i32)>,
    /// `_specialTileX/Y` pairs (`AddSpecialLegacyPoint`, `:6606-6611`) in
    /// registration order — `DrawSpecialTilesLegacy` walks them (`:7900`).
    pub legacy: Vec<(i32, i32)>,
}

impl SpecialPoints {
    /// `AddSpecialPoint` (`TileDrawing.cs:183-186`): append to the bucket.
    /// The C# has no dedup here — only the two vine crawls dedup before
    /// calling it (`:757`/`:777`).
    pub fn add(&mut self, x: i32, y: i32, kind: TileCounterType) {
        self.buckets[kind as usize].push((x, y));
    }

    /// `AddSpecialLegacyPoint` (`TileDrawing.cs:6606-6611`).
    pub fn add_legacy(&mut self, x: i32, y: i32) {
        self.legacy.push((x, y));
    }

    /// `ClearSpecialBlockCounts` (`TileDrawing.cs:388-402`): both root
    /// lists first (`:390-391`), then the counts in the C#'s exact order
    /// 3,2,6,4,1,10,0,7,8,9 (`:392-401`). Index 5 (BiomeGrass) is NOT
    /// cleared — it is never registered by `Draw` either, so the bucket
    /// stays empty in practice; the witness pins the skip.
    pub fn clear_counts(&mut self) {
        self.vine_roots_seen.clear();
        self.reverse_vine_roots_seen.clear();
        for idx in [3usize, 2, 6, 4, 1, 10, 0, 7, 8, 9] {
            self.buckets[idx].clear();
        }
    }

    /// `ClearLegacyCachedDraws` (`TileDrawing.cs:6613-6622`): the legacy
    /// point list (the TE-position dictionaries clear with it — the port's
    /// TE lookups ride `TileEntities::get_at`, no per-frame cache yet).
    pub fn clear_legacy(&mut self) {
        self.legacy.clear();
    }
}

/// `TileDrawing.GetScreenDrawArea` (`TileDrawing.cs:6549-6586`): the
/// draw-area tile bounds. `scaled_position`/`scaled_size` are
/// `Main.Camera.ScaledPosition/ScaledSize` (world pixels of the view's
/// top-left and extent); `off_screen_range` is `Main.offScreenRange` when
/// drawing into render targets (`useOffscreenRange = !Main.drawToScreen`),
/// else 0. The C# truncates toward zero (`(int)`), then clamps to the
/// 4-tile world border. The +4/+2 loop margins are applied by the caller
/// (`Draw` `:504-506`: `i < lastTileY + 4`, `j from firstTileX - 2 to
/// lastTileX + 2` exclusive) — [`draw_area_loop_bounds`] folds them in.
///
/// The section-framing half (`:6574-6585`, `SectionTileFrameWithCheck` /
/// `RefreshSections`) is NOT ported here — the port frames tiles eagerly
/// on edit (`tr_core::framing`), no `sectionManager` exists; recorded
/// deviation (docs/ANALYSIS.md "The Draw main pass (W4-g28)").
pub(crate) fn screen_draw_area(
    scaled_position: [f32; 2],
    scaled_size: [f32; 2],
    use_offscreen_range: bool,
    off_screen_range: f32,
    max_tiles_x: i32,
    max_tiles_y: i32,
) -> ([f32; 2], i32, i32, i32, i32) {
    // TileDrawing.cs:6553
    let off = if use_offscreen_range {
        [off_screen_range, off_screen_range]
    } else {
        [0.0, 0.0]
    };
    // TileDrawing.cs:6554-6557 — the (int) truncations toward zero.
    let first_x = ((scaled_position[0] - off[0]) / 16.0 - 1.0) as i32;
    let last_x = ((scaled_position[0] + scaled_size[0] + off[0]) / 16.0) as i32 + 2;
    let first_y = ((scaled_position[1] - off[1]) / 16.0 - 1.0) as i32;
    let last_y = ((scaled_position[1] + scaled_size[1] + off[1]) / 16.0) as i32 + 5;
    // TileDrawing.cs:6558-6573 — the 4-tile border clamps.
    let first_x = first_x.max(4);
    let last_x = last_x.min(max_tiles_x - 4);
    let first_y = first_y.max(4);
    let last_y = last_y.min(max_tiles_y - 4);
    (off, first_x, last_x, first_y, last_y)
}

/// The `Draw` section-loop bounds (`TileDrawing.cs:504-506`): rows
/// `firstTileY ..= lastTileY + 3` (`i < lastTileY + 4`), columns
/// `firstTileX - 2 ..= lastTileX + 1` (`j < lastTileX + 2`). Returns
/// `(x_start, x_end_exclusive, y_start, y_end_exclusive)`; the caller
/// clamps to the world array (the C# `Main.tile[j, i]` null arm allocates
/// instead — the port's packed world reads out-of-range as inactive).
pub(crate) fn draw_area_loop_bounds(
    first_x: i32,
    last_x: i32,
    first_y: i32,
    last_y: i32,
) -> (i32, i32, i32, i32) {
    (first_x - 2, last_x + 2, first_y, last_y + 4)
}

/// `TileDrawing.IsTileDrawLayerSolid` (`TileDrawing.cs:4469-4476`): the
/// `TileID.Sets.DrawTileInSolidLayer` nullable override wins; else
/// `Main.tileSolid[type]`. The generated table encodes the nullable bool
/// as -1/0/1 (`tools/GenTileData/Program.cs:96-98`) — but the C# factory
/// only ever sets `true` rows (11/470/475/78/579, `TileID.cs:516-528`),
/// so a 0 row means "no override" (null), NOT false.
pub(crate) fn is_tile_draw_layer_solid(kind: u16) -> bool {
    let i = kind as usize;
    if i < td::TILE_COUNT {
        let v = td::SET_DRAW_TILE_IN_SOLID_LAYER[i];
        if v != 0 {
            return v > 0;
        }
        return td::TILE_SOLID.has(kind);
    }
    false
}

/// The `Draw` loop's per-tile skip gate (`TileDrawing.cs:517`):
/// `!tile.active() || IsTileDrawLayerSolid(type) != solidLayer ||
/// (DebugOptions.ShowUnbreakableWall && tile.wall == 350)`.
/// `show_unbreakable_wall` is the debug toggle (no port toggle — fed
/// `false`); wall 350 is `WallID.SandstoneWall`-class unbreakable
/// (`WallID.cs` — the literal 350 of the C# check).
pub(crate) fn draw_layer_skips(
    active: bool,
    kind: u16,
    wall: u16,
    solid_layer: bool,
    show_unbreakable_wall: bool,
) -> bool {
    !active
        || is_tile_draw_layer_solid(kind) != solid_layer
        || (show_unbreakable_wall && wall == 350)
}

/// `TileDrawing`'s layer constants (`TileDrawing.cs:791-794`) — mirrors of
/// d68's `tr_render::batch` constants (S1-G-11, the FROZEN composite
/// order). Kept here so the orchestration reads without the batch module
/// on this branch; the wiring step uses one or the other, never a third
/// numbering.
pub(crate) const LAYER_LIQUID_BEHIND_TILES: u32 = 0;
pub(crate) const LAYER_BEHIND_TILES: u32 = 1;
pub(crate) const LAYER_TILES: u32 = 2;
pub(crate) const LAYER_OVER_TILES: u32 = 3;

/// The FROZEN `PostDrawTiles(false)` pass order (`TileDrawing.cs:421-430`),
/// as the bucket each pass consumes. The wiring step walks this list
/// verbatim — reordering it changes the composite order (S1-G-11).
pub(crate) const POST_DRAW_ORDER: [TileCounterType; 10] = [
    TileCounterType::MultiTileVine,   // DrawMultiTileVines      :421
    TileCounterType::MultiTileGrass,  // DrawMultiTileGrass      :422
    TileCounterType::VoidLens,        // DrawVoidLenses          :423
    TileCounterType::TeleportationPylon, // DrawTeleportationPylons :424
    TileCounterType::MasterTrophy,    // DrawMasterTrophies      :425
    TileCounterType::WindyGrass,      // DrawGrass               :426
    TileCounterType::AnyDirectionalGrass, // DrawAnyDirectionalGrass :427
    TileCounterType::Tree,            // DrawTrees               :428
    TileCounterType::Vine,            // DrawVines               :429
    TileCounterType::ReverseVine,     // DrawReverseVines        :430
];

/// `TileDrawing.ShouldSwayInWind` (`TileDrawing.cs:7702-7717`): the
/// default-switch arm's WindyGrass gate (`:717`). The setting gate, the
/// `SwaysInWindBasic` set, then the type-227 dye-plant frame exceptions
/// (204/238/408/442/476 — the harvestable/blooming columns never sway).
pub(crate) fn should_sway_in_wind(
    sway_setting_enabled: bool,
    kind: u16,
    frame_x: i16,
) -> bool {
    if !sway_setting_enabled {
        return false;
    }
    if !tr_core::wind::SWAYS_IN_WIND_BASIC.has(kind) {
        return false;
    }
    if kind == 227 && matches!(frame_x, 204 | 238 | 408 | 442 | 476) {
        return false;
    }
    true
}

/// `TileDrawing.CrawlToBottomOfReverseVineAndAddSpecialPoint`
/// (`TileDrawing.cs:764-782`): crawl DOWN from the reverse-vine cell
/// (type 549) until a tile whose top edge accepts an attachment (or an
/// inactive cell) is found; the root is the row ABOVE it (`y = k - 1`).
/// The root registers once per frame (`_reverseVineRootsPositions`,
/// `:777-780`). The loop runs `k from j to maxTilesY` exclusive (`:767`);
/// an out-of-world cell reads attachable (`WorldGen.cs:70385-70388`), so
/// the crawl always terminates. If no break fires the root stays `j`
/// (the C# `y` initialiser — unreachable in a bounded world, kept).
pub(crate) fn collect_reverse_vine_root(
    world: &World,
    x: i32,
    j: i32,
    points: &mut SpecialPoints,
) {
    let mut y = j;
    // `for (int k = j; k < Main.maxTilesY; k++)` (`:767`).
    for k in j..world.height {
        // `WorldGen.TopEdgeCanBeAttachedTo(i, k) || !tile.active()`
        // (`:770`); the null arm reads attachable
        // (`WorldGen.cs:70385-70388`), matching `world.tile` None.
        let attach = tr_core::torch::top_edge_can_be_attached_to(world, x, k);
        let active = world.tile(x, k).is_some_and(|t| t.active());
        if attach || !active {
            y = k - 1;
            break;
        }
    }
    if points.reverse_vine_roots_seen.insert((x, y)) {
        points.add(x, y, TileCounterType::ReverseVine);
    }
}

/// `TileDrawing.CrawlToTopOfVineAndAddSpecialPoint`
/// (`TileDrawing.cs:744-762`): crawl UP from the vine cell (52/62/115/205/
/// 382/528/636/638) until a tile whose bottom edge accepts an attachment
/// (or an inactive cell) is found; the root is the row BELOW it
/// (`y = num + 1`, `:752`). The loop runs `num from j - 1 down to 1`
/// exclusive of 0 (`:747` — row 0 is never tested, so a vine hanging from
/// the world ceiling roots at `y = j`, the initialiser). Dedup on
/// `_vineRootsPositions` (`:757-760`). The forward crawl delegates to the
/// S1-G-1a port (`render::vines::collect_vine_root`, the same `:744-762`
/// walk) and mirrors its new roots into the Vine bucket — g30's
/// `apply_cell_class` does the identical delegation.
pub(crate) fn collect_vine_root(world: &World, x: i32, j: i32, points: &mut SpecialPoints) {
    let mut roots = Vec::new();
    crate::render::vines::collect_vine_root(world, x, j, &mut points.vine_roots_seen, &mut roots);
    for root in roots {
        points.add(root.0, root.1, TileCounterType::Vine);
    }
}

/// `TileDrawing.ClearCachedTileDraws(solidLayer)` (`TileDrawing.cs:6588-
/// 6599`): the SOLID pass clears only the hat-rack/display-doll TE caches
/// (`:6592-6593` — no port counterpart, `TileEntities::get_at` lookups);
/// the NON-SOLID pass clears the special-point counts (`:6597`). The
/// frame order (non-solid first, `Main.cs:63280` before `:63295`) makes
/// this the once-per-frame registry reset: solid-pass Tree points survive
/// into `PostDrawTiles(false)`.
pub(crate) fn clear_cached_tile_draws(solid_layer: bool, points: &mut SpecialPoints) {
    if !solid_layer {
        points.clear_counts();
    }
}

/// One `TileDrawing.Draw(solidLayer)` call's orchestration state — the
/// pieces the pass-1 wiring drives before/around g30's
/// `collect_special_points` walk (`TileDrawing.cs:465-506`). No sprite
/// pushes: the tiers feed `DrawSingleTile_SlicedBlock`'s arm pick, the
/// bounds feed the section loop, the clears feed the registry.
pub(crate) struct DrawPass {
    /// `:479-495` — rebuilt per `Draw` call, never cached across frames.
    pub tiers: tr_render::quality_light::QualityLightTiers,
    /// `:496` — `drawOffSet` (`GetScreenDrawArea`'s first output).
    pub draw_offset: [f32; 2],
    /// `:504-506` — the padded section-loop rect, `(x0, x1, y0, y1)`
    /// INCLUSIVE (g30's `collect_special_points` walks `x0..=x1` rows
    /// `y0..=y1`): `firstTileX-2 ..= lastTileX+1`,
    /// `firstTileY ..= lastTileY+3`.
    pub loop_bounds: (i32, i32, i32, i32),
}

/// The `Draw` head (`TileDrawing.cs:477-506`), in the C# order:
/// 1. `ClearLegacyCachedDraws` (`:477`) — BOTH layers clear the legacy
///    point list (`DrawSpecialTilesLegacy` re-collects per layer);
/// 2. `ClearCachedTileDraws(solidLayer)` (`:478`) — the counts clear on
///    the non-solid pass only ([`clear_cached_tile_draws`]);
/// 3. the quality-light tier rebuild (`:479-495`);
/// 4. `GetScreenDrawArea(!drawToScreen)` (`:496`) + the `:504-506` loop
///    margins ([`screen_draw_area`] / [`draw_area_loop_bounds`]).
/// The `DrawBlackHelper` construction (`:497`), `_martianGlow` (`:498-
/// 499`) and the paint-lookup reset (`:500-503`) are draw-side state the
/// wiring step owns at the sprite seam — not registry state, unported
/// here (docs/ANALYSIS.md "The Draw main pass (W4-g28)").
pub(crate) fn begin_draw_pass(
    solid_layer: bool,
    gfx_quality: f32,
    scaled_position: [f32; 2],
    scaled_size: [f32; 2],
    draw_to_screen: bool,
    off_screen_range: f32,
    max_tiles_x: i32,
    max_tiles_y: i32,
    points: &mut SpecialPoints,
) -> DrawPass {
    points.clear_legacy(); // :477
    clear_cached_tile_draws(solid_layer, points); // :478
    // :479-495 — DebugOptions.devLightTilesCheat has no port toggle.
    let tiers = tr_render::quality_light::quality_light_tiers(gfx_quality, false);
    // :496 — useOffscreenRange = !Main.drawToScreen.
    let (off, fx, lx, fy, ly) = screen_draw_area(
        scaled_position,
        scaled_size,
        !draw_to_screen,
        off_screen_range,
        max_tiles_x,
        max_tiles_y,
    );
    // :504-506 — the exclusive ends fold to g30's inclusive walk rect.
    let (x0, xe, y0, ye) = draw_area_loop_bounds(fx, lx, fy, ly);
    DrawPass {
        tiers,
        draw_offset: off,
        loop_bounds: (x0, xe - 1, y0, ye - 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::FLAG_ACTIVE;

    fn world(w: i32, h: i32) -> World {
        let mut world = World::new("t".into(), w, h);
        world.compute_column_surface();
        world.compute_chunks();
        world
    }

    fn put(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i].kind = kind;
        w.tiles[i].flags = FLAG_ACTIVE;
    }

    /// Witness: the enum discriminants ARE the C# bucket indices
    /// (`TileDrawing.cs:21-35` vs `:185`'s `(int)type` index).
    #[test]
    fn w4_g28_the_counter_discriminants_are_the_bucket_indices() {
        assert_eq!(TileCounterType::Tree as usize, 0);
        assert_eq!(TileCounterType::WindyGrass as usize, 1);
        assert_eq!(TileCounterType::MultiTileGrass as usize, 2);
        assert_eq!(TileCounterType::MultiTileVine as usize, 3);
        assert_eq!(TileCounterType::Vine as usize, 4);
        assert_eq!(TileCounterType::BiomeGrass as usize, 5);
        assert_eq!(TileCounterType::VoidLens as usize, 6);
        assert_eq!(TileCounterType::ReverseVine as usize, 7);
        assert_eq!(TileCounterType::TeleportationPylon as usize, 8);
        assert_eq!(TileCounterType::MasterTrophy as usize, 9);
        assert_eq!(TileCounterType::AnyDirectionalGrass as usize, 10);
        assert_eq!(TILE_COUNTER_COUNT, 11);
    }

    /// Witness: `AddSpecialPoint` (`TileDrawing.cs:183-186`) appends
    /// without dedup — only the two crawls dedup (`:757`/`:777`).
    #[test]
    fn w4_g28_add_appends_without_dedup() {
        let mut p = SpecialPoints::default();
        p.add(1, 2, TileCounterType::WindyGrass);
        p.add(1, 2, TileCounterType::WindyGrass);
        assert_eq!(p.buckets[1], vec![(1, 2), (1, 2)]);
        p.add_legacy(3, 4);
        p.add_legacy(3, 4);
        assert_eq!(p.legacy, vec![(3, 4), (3, 4)]);
    }

    /// Witness: `ClearSpecialBlockCounts` (`TileDrawing.cs:388-402`)
    /// clears the two root lists and the ten buckets in the C# order —
    /// index 5 (BiomeGrass) is the one bucket the method never touches;
    /// the legacy list is NOT cleared here (`ClearLegacyCachedDraws`
    /// owns it, `:6613-6622`).
    #[test]
    fn w4_g28_clear_counts_skips_biome_grass_and_legacy() {
        let mut p = SpecialPoints::default();
        for idx in 0..TILE_COUNTER_COUNT {
            p.buckets[idx].push((idx as i32, 0));
        }
        p.vine_roots_seen.insert((1, 1));
        p.reverse_vine_roots_seen.insert((2, 2));
        p.add_legacy(9, 9);
        p.clear_counts();
        for idx in [0usize, 1, 2, 3, 4, 6, 7, 8, 9, 10] {
            assert!(p.buckets[idx].is_empty(), "bucket {idx}");
        }
        assert_eq!(p.buckets[5], vec![(5, 0)]);
        assert!(p.vine_roots_seen.is_empty());
        assert!(p.reverse_vine_roots_seen.is_empty());
        assert_eq!(p.legacy, vec![(9, 9)]); // untouched
        p.clear_legacy();
        assert!(p.legacy.is_empty());
    }

    /// Witness: `ClearCachedTileDraws` (`TileDrawing.cs:6588-6599`) —
    /// the counts clear ONLY on the non-solid pass; the solid pass keeps
    /// the registry (its Tree points ride into `PostDrawTiles(false)`).
    #[test]
    fn w4_g28_the_counts_clear_only_before_the_non_solid_pass() {
        let mut p = SpecialPoints::default();
        p.add(1, 1, TileCounterType::Tree);
        p.vine_roots_seen.insert((1, 1));
        clear_cached_tile_draws(true, &mut p);
        assert_eq!(p.buckets[0], vec![(1, 1)]); // solid: survives
        assert!(!p.vine_roots_seen.is_empty());
        clear_cached_tile_draws(false, &mut p);
        assert!(p.buckets[0].is_empty()); // non-solid: reset
        assert!(p.vine_roots_seen.is_empty());
    }

    /// Witness: `GetScreenDrawArea` (`TileDrawing.cs:6549-6573`) — the
    /// `(int)` truncations toward zero, the +2/+5 far-edge pads and the
    /// 4-tile border clamps. Camera at world pixel (160, 320), view
    /// 320×160: firstX = (int)(160/16 - 1) = 9, lastX = (int)(480/16)+2
    /// = 32, firstY = 19, lastY = (int)(480/16)+5 = 35.
    #[test]
    fn w4_g28_the_screen_area_truncates_pads_and_clamps() {
        let (off, fx, lx, fy, ly) =
            screen_draw_area([160.0, 320.0], [320.0, 160.0], false, 64.0, 8400, 2400);
        assert_eq!(off, [0.0, 0.0]); // drawToScreen — no offscreen pad
        assert_eq!((fx, lx, fy, ly), (9, 32, 19, 35));
        // Fractional camera: (int) truncates toward zero, never rounds.
        let (_, fx, lx, fy, ly) =
            screen_draw_area([160.9, 320.9], [320.0, 160.0], false, 0.0, 8400, 2400);
        assert_eq!((fx, lx, fy, ly), (9, 32, 19, 35));
        // The 4-tile border clamps (`:6558-6573`).
        let (_, fx, lx, fy, ly) =
            screen_draw_area([0.0, 0.0], [64.0, 64.0], false, 0.0, 100, 100);
        assert_eq!((fx, lx, fy, ly), (4, 6, 4, 9));
        let (_, fx, lx, fy, ly) =
            screen_draw_area([1600.0, 1600.0], [320.0, 160.0], false, 0.0, 100, 100);
        assert_eq!((fx, lx, fy, ly), (99, 96, 99, 96)); // last < first: empty walk
    }

    /// Witness: the offscreen pad (`TileDrawing.cs:6553`) — drawing into
    /// render targets offsets BOTH edges by `Main.offScreenRange`
    /// (`useOffscreenRange = !Main.drawToScreen`).
    #[test]
    fn w4_g28_the_offscreen_pad_widens_both_edges() {
        let (off, fx, lx, fy, ly) =
            screen_draw_area([160.0, 320.0], [320.0, 160.0], true, 64.0, 8400, 2400);
        assert_eq!(off, [64.0, 64.0]);
        // firstX = (int)((160-64)/16 - 1) = 5; lastX = (int)(544/16)+2 = 36.
        // firstY = (int)((320-64)/16 - 1) = 15; lastY = (int)(544/16)+5 = 39.
        assert_eq!((fx, lx, fy, ly), (5, 36, 15, 39));
    }

    /// Witness: the `Draw` loop margins (`TileDrawing.cs:504-506`) —
    /// `i in firstTileY..lastTileY+4` (rows), `j in firstTileX-2..
    /// lastTileX+2` (columns), both ends exclusive.
    #[test]
    fn w4_g28_the_loop_bounds_fold_the_margins() {
        assert_eq!(draw_area_loop_bounds(9, 32, 19, 35), (7, 34, 19, 39));
    }

    /// Witness: `IsTileDrawLayerSolid` (`TileDrawing.cs:4469-4476`) —
    /// the five explicit `DrawTileInSolidLayer` overrides
    /// (`TileID.cs:516-528`: 11/470/475/78/579 all `true`) win over
    /// `Main.tileSolid`; a 0 row means null (no override), NOT false.
    #[test]
    fn w4_g28_the_solid_layer_override_beats_tile_solid() {
        for kind in [11u16, 78, 470, 475, 579] {
            assert!(is_tile_draw_layer_solid(kind), "override kind {kind}");
        }
        assert!(is_tile_draw_layer_solid(0)); // dirt — the solid set
        assert!(!is_tile_draw_layer_solid(52)); // vine — non-solid
        assert!(!is_tile_draw_layer_solid(4)); // torch — non-solid
        assert!(!is_tile_draw_layer_solid(700)); // out of table
        assert!(!is_tile_draw_layer_solid(u16::MAX));
    }

    /// Witness: the `:517` skip gate — inactive, layer mismatch, and the
    /// `ShowUnbreakableWall && wall == 350` debug arm (fed false in the
    /// port, but the predicate keeps the C# shape).
    #[test]
    fn w4_g28_the_skip_gate_matches_the_layer_and_debug_arm() {
        assert!(draw_layer_skips(false, 0, 0, true, false)); // inactive
        assert!(draw_layer_skips(true, 0, 0, false, false)); // dirt on non-solid
        assert!(!draw_layer_skips(true, 0, 0, true, false)); // dirt on solid
        assert!(draw_layer_skips(true, 11, 0, false, false)); // door override
        assert!(!draw_layer_skips(true, 11, 0, true, false));
        assert!(draw_layer_skips(true, 0, 350, true, true)); // debug wall
        assert!(!draw_layer_skips(true, 0, 350, true, false));
        assert!(!draw_layer_skips(true, 0, 349, true, true)); // only 350
    }

    /// Witness: the layer constants (`TileDrawing.cs:791-794`) and the
    /// FROZEN `PostDrawTiles(false)` order (`:421-430`) — reordering
    /// either changes the composite (S1-G-11).
    #[test]
    fn w4_g28_the_layers_and_post_draw_order_are_frozen() {
        assert_eq!(LAYER_LIQUID_BEHIND_TILES, 0);
        assert_eq!(LAYER_BEHIND_TILES, 1);
        assert_eq!(LAYER_TILES, 2);
        assert_eq!(LAYER_OVER_TILES, 3);
        assert_eq!(
            POST_DRAW_ORDER.map(|t| t as usize),
            [3, 2, 6, 8, 9, 1, 10, 0, 4, 7]
        );
    }

    /// Witness: `ShouldSwayInWind` (`TileDrawing.cs:7702-7717`) — the
    /// settings gate, the `SwaysInWindBasic` membership, and the frame
    /// exceptions gated on `type == 227` ONLY (`:7712`).
    #[test]
    fn w4_g28_sway_gate_is_settings_set_and_227_only() {
        assert!(should_sway_in_wind(true, 83, 0)); // herb
        assert!(!should_sway_in_wind(false, 83, 0)); // setting off
        assert!(!should_sway_in_wind(true, 0, 0)); // dirt never sways
        for fx in [204i16, 238, 408, 442, 476] {
            assert!(!should_sway_in_wind(true, 227, fx), "227 frame {fx}");
            assert!(should_sway_in_wind(true, 83, fx), "83 frame {fx}");
        }
        assert!(should_sway_in_wind(true, 227, 0));
        assert!(should_sway_in_wind(true, 227, 18));
    }

    /// Witness: `CrawlToTopOfVineAndAddSpecialPoint` (`TileDrawing.cs:
    /// 744-762`) via the S1-G-1a delegation — every cell of one strip
    /// resolves the SAME root (the row below the attachable dirt),
    /// deduped once into bucket 4.
    #[test]
    fn w4_g28_the_forward_crawl_roots_below_the_attachment() {
        let mut w = world(40, 60);
        put(&mut w, 10, 19, 0); // dirt — the attachment
        for y in 20..=22 {
            put(&mut w, 10, y, 52);
        }
        let mut p = SpecialPoints::default();
        for y in 20..=22 {
            collect_vine_root(&w, 10, y, &mut p);
        }
        assert_eq!(p.buckets[TileCounterType::Vine as usize], vec![(10, 20)]);
        // Row 0 is never tested (`num > 0`, `:747`): a strip from the
        // ceiling roots at its own first row (the y = j initialiser).
        let mut w2 = world(40, 60);
        for y in 0..=2 {
            put(&mut w2, 10, y, 52);
        }
        let mut p2 = SpecialPoints::default();
        collect_vine_root(&w2, 10, 1, &mut p2);
        assert_eq!(p2.buckets[TileCounterType::Vine as usize], vec![(10, 1)]);
    }

    /// Witness: `CrawlToBottomOfReverseVineAndAddSpecialPoint`
    /// (`TileDrawing.cs:764-782`) — the loop starts AT j (`:767`), the
    /// root is the row above the attachment, deduped once; a lone cell
    /// with empty ground below roots at its own row (k = j+1 inactive).
    #[test]
    fn w4_g28_the_reverse_crawl_roots_above_the_attachment() {
        let mut w = world(40, 60);
        put(&mut w, 10, 23, 0); // dirt — the ground attachment
        for y in 20..=22 {
            put(&mut w, 10, y, 549);
        }
        let mut p = SpecialPoints::default();
        for y in 20..=22 {
            collect_reverse_vine_root(&w, 10, y, &mut p);
        }
        assert_eq!(p.buckets[TileCounterType::ReverseVine as usize], vec![(10, 22)]);
        // The k = j start: an attachable cell AT j roots at j - 1.
        let mut w2 = world(40, 60);
        put(&mut w2, 10, 20, 0);
        let mut p2 = SpecialPoints::default();
        collect_reverse_vine_root(&w2, 10, 20, &mut p2);
        assert_eq!(p2.buckets[TileCounterType::ReverseVine as usize], vec![(10, 19)]);
        // A lone 549 cell: k = j + 1 is inactive → root = j.
        let mut w3 = world(40, 60);
        put(&mut w3, 10, 20, 549);
        let mut p3 = SpecialPoints::default();
        collect_reverse_vine_root(&w3, 10, 20, &mut p3);
        assert_eq!(p3.buckets[TileCounterType::ReverseVine as usize], vec![(10, 20)]);
    }

    /// Witness: the `Draw` head order (`TileDrawing.cs:477-506`) — the
    /// legacy list clears on BOTH layers, the counts only on non-solid,
    /// the tiers rebuild per call, and the loop rect carries the margins.
    /// Frame order: non-solid first (`Main.cs:63280`), then solid
    /// (`:63295`) — the solid pass's Tree points survive the second head.
    #[test]
    fn w4_g28_the_draw_head_clears_and_builds_in_order() {
        let mut p = SpecialPoints::default();
        p.add_legacy(5, 5);
        p.add(1, 1, TileCounterType::WindyGrass);
        // The non-solid pass head (:477-478 both fire).
        let pass = begin_draw_pass(
            false, 0.5, [160.0, 320.0], [320.0, 160.0], true, 64.0, 8400, 2400, &mut p,
        );
        assert!(p.legacy.is_empty());
        assert!(p.buckets[1].is_empty());
        assert_eq!(pass.tiers.high, [142, 156, 171]); // q = 0.5
        assert_eq!(pass.tiers.medium, [26, 28, 31]);
        assert_eq!(pass.draw_offset, [0.0, 0.0]); // drawToScreen
        // (9, 32, 19, 35) + margins → x 7..=33, y 19..=38 inclusive.
        assert_eq!(pass.loop_bounds, (7, 33, 19, 38));
        // The solid pass head: legacy clears again, counts SURVIVE.
        p.add(2, 2, TileCounterType::Tree);
        p.add_legacy(6, 6);
        let pass2 = begin_draw_pass(
            true, 0.0, [160.0, 320.0], [320.0, 160.0], false, 64.0, 8400, 2400, &mut p,
        );
        assert!(p.legacy.is_empty());
        assert_eq!(p.buckets[0], vec![(2, 2)]); // Tree survives
        assert_eq!(pass2.tiers.high, [255, 24, 50]); // q = 0 wrap
        assert_eq!(pass2.draw_offset, [64.0, 64.0]); // render targets
        assert_eq!(pass2.loop_bounds, (3, 37, 15, 42));
    }
}