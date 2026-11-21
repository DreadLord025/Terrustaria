//! W4-g30 — TileDrawing pass 3: the `Draw` loop's per-cell classification
//! switch (`TileDrawing.cs:534-723`) and the deferred special-point registry
//! it feeds (`TileCounterType` `:21-35`, `_specialsCount`/`_specialPositions`
//! `:78-80`, `AddSpecialPoint` `:183-185`, `ClearSpecialBlockCounts`
//! `:388-402`). Research/09 pass row 4: the port inlines grass/vines/trees and
//! loses C#'s "collect first, draw after" order — this module ports the
//! MISSING collect half.
//!
//! C# `Draw(solidLayer)` walks the screen area row-major (`:504-506`), gates
//! each cell on `active()` + `IsTileDrawLayerSolid` (`:517`, `:4469-4476`),
//! then classifies (`:534-723`): most special families register a point and
//! `continue` (the cell never draws through `DrawSingleTile`); the vine
//! threads crawl to the strip root first (`:744-782`); 491/597/617 register
//! AND fall through to `DrawSingleTile` (`break` arms); 530/705 fall through
//! only outside their multi-tile frame range; the default arm defers
//! `ShouldSwayInWind` tiles (`:7702-7717`) as WindyGrass. `PostDrawTiles`
//! (`:414-442`) later drains the buckets in its own pass order.
//!
//! UNWIRED (the S1-G-1e pattern): the `draw_tiles` seam is contested by the
//! in-flight pass siblings (g28/g29 passes 1-2, g31/g32 passes 4-5). Intended
//! call sites at the family merge:
//! * `render/tiles.rs:586-640` — the inline vine-root collection becomes
//!   `collect_special_points`' Vine bucket (same crawl, same dedup);
//! * `render/pylons.rs::special_kind` (`:172-180`) — its 491/597/617 gates
//!   duplicate `:695-712`; the passes consume the registry buckets instead;
//! * `render/tiles.rs:915+` — the inline sunflower draw becomes the
//!   MultiTileGrass bucket drain (`DrawMultiTileGrass`, pass 5);
//! * `render/tiles.rs:882-896` — the inline sway gate becomes
//!   `should_sway_in_wind` (the port currently applies the type-227 frame
//!   exceptions to EVERY kind; C# `:7712` gates them on `type == 227`).
//!
//! Deviations: `EmitLiquidDrops` (`:693`, `:6688+`) is the particle battery
//! (W4-d60) — the driver records the cells, rolls no RNG (the determinism
//! rule). The C# null-tile arm (`:509-514`, lazy `new Tile()` + `mapTime +=
//! 60`) has no port counterpart (the tile array is dense). The fixed 9000
//! slots per bucket (`:68`, `:194`) grow here instead of throwing
//! `IndexOutOfRangeException`; the cap drops the excess (unreachable in a
//! vanilla draw area).
#![allow(dead_code)] // UNWIRED — see the module doc; the wiring is the pass-family merge step.

use crate::render::vines;
use std::collections::HashSet;
use tr_core::tiledata as td;
use tr_core::World;

/// `TileDrawing.MAX_SPECIALS` (`TileDrawing.cs:68`): the per-bucket slot
/// count of the ctor's `new Point[9000]` arrays (`:194`).
pub(crate) const MAX_SPECIALS: usize = 9000;

/// `TileCounterType` (`TileDrawing.cs:21-35`) — the discriminants ARE the
/// `_specialsCount`/`_specialPositions` indices (`:185`), so the order is
/// load-bearing. `BiomeGrass` (5) is never registered by any C# call site
/// and is the one bucket `ClearSpecialBlockCounts` skips (`:388-402`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
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

/// The classification switch's outcome for one cell (`TileDrawing.cs:534-724`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CellClass {
    /// `DrawSingleTile` (`:724`) — the switch fell through (`break`/default).
    SingleTile,
    /// `AddSpecialPoint` + `continue`: the cell never draws inline.
    Deferred(TileCounterType),
    /// `AddSpecialPoint` + `break`: the special registers AND the cell draws
    /// through `DrawSingleTile` (the 491/597/617 arms, `:695-712`).
    DeferredAndSingle(TileCounterType),
    /// `continue` with NO registration — a gated multi-tile cell whose frame
    /// is not the object's anchor (e.g. tile 34 off the `% 54` grid).
    Skip,
    /// `CrawlToTopOfVineAndAddSpecialPoint` + `continue` (`:544`).
    VineCrawl,
    /// `CrawlToBottomOfReverseVineAndAddSpecialPoint` + `continue` (`:547`).
    ReverseVineCrawl,
    /// `EmitLiquidDrops` + `continue` (`:693`) — the drip particles
    /// (373/374/375/461/709); the driver records the cell for W4-d60.
    LiquidDrops,
}

/// `_specialsCount`/`_specialPositions` (`TileDrawing.cs:78-80`) plus the two
/// crawl dedup lists `_vineRootsPositions`/`_reverseVineRootsPositions`
/// (`:120` area, cleared at `:390-391`). C# clears the counts only before the
/// NON-solid `Draw` pass (`ClearCachedTileDraws`, `:6592-6601`); the solid
/// pass's tree points (the trunk kinds are solid-layer) survive into
/// `PostDrawTiles(false)` (`:414-442`) alongside the non-solid pass's points.
/// The port drives both layers through the same instance and clears once per
/// frame, before the solid walk (the `Draw(solid:true)` call site).
pub(crate) struct SpecialPoints {
    /// The 11 buckets, indexed by `TileCounterType as usize` (`:185`).
    pub buckets: [Vec<(i32, i32)>; 11],
    /// `_vineRootsPositions` — the forward-crawl dedup (`:757`).
    pub vine_roots_seen: HashSet<(i32, i32)>,
    /// `_reverseVineRootsPositions` — the reverse-crawl dedup (`:777`).
    pub reverse_vine_roots_seen: HashSet<(i32, i32)>,
    /// Port-side record of the `EmitLiquidDrops` cells (`:693`) — the
    /// particle battery (W4-d60) consumes `(x, y, kind)`; no C# counterpart
    /// (C# emits immediately).
    pub liquid_drop_cells: Vec<(i32, i32, u16)>,
    /// Port-side count of the cells that reached `DrawSingleTile` (`:724`) —
    /// a wiring witness, no C# counterpart.
    pub single_tiles: u32,
}

impl SpecialPoints {
    /// The ctor's 11 empty buckets (`TileDrawing.cs:192-195`).
    pub fn new() -> Self {
        Self {
            buckets: std::array::from_fn(|_| Vec::new()),
            vine_roots_seen: HashSet::new(),
            reverse_vine_roots_seen: HashSet::new(),
            liquid_drop_cells: Vec::new(),
            single_tiles: 0,
        }
    }

    /// `AddSpecialPoint` (`TileDrawing.cs:183-185`): append, no dedup (only
    /// the crawls dedup). The C# indexes a fixed `Point[9000]` and would
    /// throw past the cap; the port drops the excess (see the module doc).
    pub fn add_special_point(&mut self, x: i32, y: i32, kind: TileCounterType) {
        let bucket = &mut self.buckets[kind as usize];
        if bucket.len() >= MAX_SPECIALS {
            return;
        }
        bucket.push((x, y));
    }

    /// `ClearSpecialBlockCounts` (`TileDrawing.cs:388-402`): the two root
    /// lists first (`:390-391`), then the counts in the C#'s exact order
    /// 3,2,6,4,1,10,0,7,8,9 — index 5 (`BiomeGrass`) is NOT cleared (it is
    /// never written either, so the bucket stays empty in practice). The
    /// port-side records reset with the draw.
    pub fn clear_special_block_counts(&mut self) {
        self.vine_roots_seen.clear();
        self.reverse_vine_roots_seen.clear();
        for idx in [3usize, 2, 6, 4, 1, 10, 0, 7, 8, 9] {
            self.buckets[idx].clear();
        }
        self.liquid_drop_cells.clear();
        self.single_tiles = 0;
    }
}

/// `IsTileDrawLayerSolid` (`TileDrawing.cs:4469-4476`): the nullable
/// `TileID.Sets.DrawTileInSolidLayer` override wins, else `Main.tileSolid`
/// (`_tileSolid`, `:175`). The generated table encodes the five explicit
/// `true` ids (11/78/470/475/579, `TileID.cs:516-527`) as 1 and the `null`
/// default as 0 — the C# set carries no `false` entries, so nonzero ⇔
/// `HasValue`. An out-of-table kind falls to the solid set (`TileSet::has`
/// bounds-checks).
pub(crate) fn is_tile_draw_layer_solid(kind: u16) -> bool {
    let idx = kind as usize;
    if idx < td::TILE_COUNT && td::SET_DRAW_TILE_IN_SOLID_LAYER[idx] != 0 {
        return td::SET_DRAW_TILE_IN_SOLID_LAYER[idx] == 1;
    }
    td::TILE_SOLID.has(kind)
}

/// `ShouldSwayInWind` (`TileDrawing.cs:7702-7717`): the settings gate, the
/// `SwaysInWindBasic` set, and the type-227 dye-plant frame exceptions —
/// the exceptions are gated on `type == 227` in C# (the inline port at
/// `tiles.rs:882-883` applies them to every kind; this is the faithful gate).
pub(crate) fn should_sway_in_wind(kind: u16, frame_x: i16, sway_enabled: bool) -> bool {
    // `Main.SettingsEnabled_TilesSwayInWind` (`:7704`) — the port's
    // `Wind::sway_enabled` rides the same setting.
    if !sway_enabled {
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

/// The `Draw` classification switch (`TileDrawing.cs:534-723`) for one
/// active, layer-matched cell. `frame_x`/`frame_y` are the raw `tile.frameX/
/// frameY` (`:528-529`); C# `%`/`/` truncate toward zero, which Rust's i16
/// operators reproduce. `sway_enabled` feeds the default arm's
/// `ShouldSwayInWind` (`:717`).
pub(crate) fn classify_tile(
    kind: u16,
    frame_x: i16,
    frame_y: i16,
    sway_enabled: bool,
) -> CellClass {
    use CellClass::*;
    use TileCounterType::*;
    let fx = frame_x as i32;
    let fy = frame_y as i32;
    match kind {
        // `:536-545` — the VineThreads crawl (the set order differs from
        // `TileID.Sets.IsVine`; the switch lists them explicitly).
        52 | 62 | 115 | 205 | 382 | 528 | 636 | 638 => VineCrawl,
        // `:546-548`.
        549 => ReverseVineCrawl,
        // `:549-554` — hanging planter boxes (3×3). The `continue` sits
        // OUTSIDE the gate (`:554`): an off-anchor cell skips, never sways.
        34 if fx % 54 == 0 && fy % 54 == 0 => Deferred(MultiTileVine),
        34 => Skip,
        // `:555-560` — the 698 hanging pot (1-wide anchor row).
        698 if fx % 18 == 0 && fy == 0 => Deferred(MultiTileVine),
        698 => Skip,
        // `:561-566` — the 454 hanging chain lantern (4×3).
        454 if fx % 72 == 0 && fy % 54 == 0 => Deferred(MultiTileVine),
        454 => Skip,
        // `:567-577` — the 1×2 hanging family.
        42 | 270 | 271 | 572 | 581 | 660 if fx % 18 == 0 && fy % 36 == 0 => {
            Deferred(MultiTileVine)
        }
        42 | 270 | 271 | 572 | 581 | 660 => Skip,
        // `:578-583` — the 91 hanging bottle (1×3).
        91 if fx % 18 == 0 && fy % 54 == 0 => Deferred(MultiTileVine),
        91 => Skip,
        // `:584-591` — the 2×2 hanging family.
        95 | 126 | 444 if fx % 36 == 0 && fy % 36 == 0 => Deferred(MultiTileVine),
        95 | 126 | 444 => Skip,
        // `:592-599` — the 2×3 hanging family.
        465 | 591 | 592 if fx % 36 == 0 && fy % 54 == 0 => Deferred(MultiTileVine),
        465 | 591 | 592 => Skip,
        // `:600-605` — the sunflower (2×4, anchor row 0).
        27 if fx % 36 == 0 && fy == 0 => Deferred(MultiTileGrass),
        27 => Skip,
        // `:606-613` — the 2×2 potted plants.
        236 | 238 | 702 if fx % 36 == 0 && fy == 0 => Deferred(MultiTileGrass),
        236 | 238 | 702 => Skip,
        // `:614-623` — the 233 jungle plant: TWO independent anchors (the
        // 3×2 top at frameY 0, the 2×2 bulb at frameY 36).
        233 => {
            if fy == 0 && fx % 54 == 0 {
                return Deferred(MultiTileGrass);
            }
            if fy == 36 && fx % 36 == 0 {
                return Deferred(MultiTileGrass);
            }
            Skip
        }
        // `:624-629` / `:630-635` — the 652/651 tall grasses (no row gate).
        652 if fx % 36 == 0 => Deferred(MultiTileGrass),
        652 => Skip,
        651 if fx % 54 == 0 => Deferred(MultiTileGrass),
        651 => Skip,
        // `:636-645` — 530 OasisPlants: the multi-tile half lives below
        // frameX 270; the rest of the sheet `break`s straight to
        // DrawSingleTile (`:645`) — NO default-arm sway check.
        530 if fx < 270 => {
            if fx % 54 == 0 && fy == 0 {
                return Deferred(MultiTileGrass);
            }
            Skip
        }
        530 => SingleTile,
        // `:646-655` — 705 SeaOats: the same split modulo the 486 style block.
        705 if fx % 486 < 270 => {
            if fx % 54 == 0 && fy % 36 == 0 {
                return Deferred(MultiTileGrass);
            }
            Skip
        }
        705 => SingleTile,
        // `:656-663` — the 485/489/490 2×1 wind toys.
        485 | 489 | 490 if fy == 0 && fx % 36 == 0 => Deferred(MultiTileGrass),
        485 | 489 | 490 => Skip,
        // `:664-675` — the 521-527 2×1 crystal shards.
        521..=527 if fy == 0 && fx % 36 == 0 => Deferred(MultiTileGrass),
        521..=527 => Skip,
        // `:676-681` — the 493 golf flag (1-wide, row 0).
        493 if fy == 0 && fx % 18 == 0 => Deferred(MultiTileGrass),
        493 => Skip,
        // `:682-687` — the 519 campfire: columns 0-4 register; the
        // `continue` (`:687`) is outside the gate, so the high columns skip.
        519 => {
            if fx / 18 <= 4 {
                return Deferred(MultiTileGrass);
            }
            Skip
        }
        // `:688-694` — the drip tiles: EmitLiquidDrops, then continue.
        373 | 374 | 375 | 461 | 709 => LiquidDrops,
        // `:695-700` — the void lens: the 18/18 sub-cell registers AND draws.
        491 => {
            if fx == 18 && fy == 18 {
                return DeferredAndSingle(VoidLens);
            }
            SingleTile
        }
        // `:701-706` — the pylon anchor (3-wide, row 0), register AND draw.
        597 => {
            if fx % 54 == 0 && fy == 0 {
                return DeferredAndSingle(TeleportationPylon);
            }
            SingleTile
        }
        // `:707-712` — the master trophy anchor (3×4), register AND draw.
        617 => {
            if fx % 54 == 0 && fy % 72 == 0 {
                return DeferredAndSingle(MasterTrophy);
            }
            SingleTile
        }
        // `:713-715` — every 184 cell is an AnyDirectionalGrass point.
        184 => Deferred(AnyDirectionalGrass),
        // `:716-722` — the default arm: sway candidates defer as WindyGrass.
        _ => {
            if should_sway_in_wind(kind, frame_x, sway_enabled) {
                Deferred(WindyGrass)
            } else {
                SingleTile
            }
        }
    }
}

/// `CrawlToBottomOfReverseVineAndAddSpecialPoint` (`TileDrawing.cs:764-782`):
/// crawl DOWN from the reverse-vine cell until a tile whose TOP edge accepts
/// an attachment (or an inactive cell) is found; the root is the row ABOVE it
/// (`y = k - 1`, `:772`). Unlike the forward crawl the loop starts AT `j`
/// (`:767`) and runs to `Main.maxTilesY` exclusive. Dedup on
/// `_reverseVineRootsPositions` (`:777-780`).
pub(crate) fn collect_reverse_vine_root(
    world: &World,
    x: i32,
    j: i32,
    reg: &mut SpecialPoints,
) {
    let mut y = j;
    // `for (int k = j; k < Main.maxTilesY; k++)` (`:767`).
    for k in j..world.height {
        // `WorldGen.TopEdgeCanBeAttachedTo(i, k) || !tile.active()` (`:770`).
        let attach = tr_core::torch::top_edge_can_be_attached_to(world, x, k);
        let active = world.tile(x, k).is_some_and(|t| t.active());
        if attach || !active {
            y = k - 1;
            break;
        }
    }
    if reg.reverse_vine_roots_seen.insert((x, y)) {
        reg.add_special_point(x, y, TileCounterType::ReverseVine);
    }
}

/// Apply one cell's `CellClass` to the registry — the switch body's side
/// effects (`TileDrawing.cs:544-722`). Returns whether `DrawSingleTile`
/// runs for the cell (`:724`). The forward crawl delegates to the ported
/// `vines::collect_vine_root` (S1-G-1a, the same `:744-762` port) and copies
/// its new roots into the Vine bucket so both views stay in sync.
pub(crate) fn apply_cell_class(
    world: &World,
    x: i32,
    y: i32,
    kind: u16,
    class: CellClass,
    reg: &mut SpecialPoints,
) -> bool {
    match class {
        CellClass::SingleTile => true,
        CellClass::Deferred(t) => {
            reg.add_special_point(x, y, t);
            false
        }
        CellClass::DeferredAndSingle(t) => {
            reg.add_special_point(x, y, t);
            true
        }
        CellClass::Skip => false,
        CellClass::VineCrawl => {
            // The S1-G-1a crawl keeps its own Vec/HashSet shape; feed it the
            // registry's dedup set and mirror new roots into bucket 4.
            let mut roots = Vec::new();
            vines::collect_vine_root(world, x, y, &mut reg.vine_roots_seen, &mut roots);
            for root in roots {
                reg.add_special_point(root.0, root.1, TileCounterType::Vine);
            }
            false
        }
        CellClass::ReverseVineCrawl => {
            collect_reverse_vine_root(world, x, y, reg);
            false
        }
        CellClass::LiquidDrops => {
            // `EmitLiquidDrops(i, j, tile, type)` (`:693`): the particle
            // battery owns the rolls; the driver records the cell only.
            reg.liquid_drop_cells.push((x, y, kind));
            false
        }
    }
}

/// `CacheSpecialDraws_Part1`'s Tree tail (`TileDrawing.cs:4260-4284`), called
/// from `DrawSingleTile` (`:992`) with the `GetTileDrawData`-corrected
/// `tileFrameX/tileFrameY` (the overrides included — the port's caller passes
/// `frame_x_override.unwrap_or(tile.frame_x)`). The 323 palm fronds
/// (`88 <= frameX <= 132`) and the trunk-branch cells of the tree family
/// (`frameY >= 198 && frameX >= 22`) register a Tree point so `DrawTrees`
/// (`:8052`) draws the canopy/branches there. The legacy TE arms above the
/// `skipDraw` gate (`:4154-4255`) need the TileEntity store — out of scope
/// (research/09 row "Tile-entity-bearing furniture"); this is the Tree half.
pub(crate) fn cache_tree_point(
    x: i32,
    y: i32,
    kind: u16,
    draw_frame_x: i16,
    draw_frame_y: i16,
    reg: &mut SpecialPoints,
) {
    let fx = draw_frame_x as i32;
    let fy = draw_frame_y as i32;
    let tree = match kind {
        // `case 323` (`:4262-4267`).
        323 => fx <= 132 && fx >= 88,
        // `case 5/583-589/596/616/634` (`:4268-4283`).
        5 | 583 | 584 | 585 | 586 | 587 | 588 | 589 | 596 | 616 | 634 => fy >= 198 && fx >= 22,
        _ => false,
    };
    if tree {
        reg.add_special_point(x, y, TileCounterType::Tree);
    }
}

/// The `Draw` loop's classification walk (`TileDrawing.cs:504-727`) over the
/// screen area `[x0..=x1] × [y0..=y1]` for ONE layer. Row-major, y outer /
/// x inner (`:504-506`). Per cell: inactive or layer-mismatched cells skip
/// (`:517`), then the switch classifies and the registry collects. The
/// `solid_layer` flag selects the layer gate — C# calls `Draw` twice per
/// frame (`Main.cs`'s two `TilesRenderer.Draw` calls), the registry cleared
/// only before the non-solid pass (`ClearCachedTileDraws`, `:6592-6601`).
///
/// The C# screen-area margins (`+4` rows below, `-2/+2` columns, `:504-506`)
/// are the CALLER's concern — `GetScreenDrawArea`'s outputs are the loop
/// bounds in C#, so the port's caller passes the same padded rect. This
/// driver walks exactly the rect it is given.
pub(crate) fn collect_special_points(
    world: &World,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    solid_layer: bool,
    sway_enabled: bool,
    reg: &mut SpecialPoints,
) {
    for y in y0..=y1 {
        for x in x0..=x1 {
            // `Main.tile[j, i]` (`:508`); the null-tile arm (`:509-514`)
            // lazily allocates + bumps mapTime — the dense port array has no
            // null cells, an out-of-bounds read is simply skipped.
            let Some(tile) = world.tile(x, y) else {
                continue;
            };
            // `!tile.active() || IsTileDrawLayerSolid(type) != solidLayer`
            // (`:517`). The `ShowUnbreakableWall` debug arm (`tile.wall ==
            // 350`) has no port toggle — skipped like the other DebugOptions.
            if !tile.active() || is_tile_draw_layer_solid(tile.kind) != solid_layer {
                continue;
            }
            let class = classify_tile(tile.kind, tile.frame_x, tile.frame_y, sway_enabled);
            if apply_cell_class(world, x, y, tile.kind, class, reg) {
                reg.single_tiles += 1;
            }
        }
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

    fn put(w: &mut World, x: i32, y: i32, kind: u16, fx: i16, fy: i16) {
        let i = w.index(x, y);
        w.tiles[i].kind = kind;
        w.tiles[i].flags = FLAG_ACTIVE;
        w.tiles[i].frame_x = fx;
        w.tiles[i].frame_y = fy;
    }

    /// Witness: the enum discriminants ARE the C# bucket indices
    /// (`TileDrawing.cs:21-35` vs `:185`'s `(int)type` index).
    #[test]
    fn w4_g30_the_counter_discriminants_are_the_bucket_indices() {
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
    }

    /// Witness: `AddSpecialPoint` (`:183-185`) appends without dedup and
    /// honours the `MAX_SPECIALS` 9000 slot cap (`:68`, `:194`).
    #[test]
    fn w4_g30_add_special_point_appends_and_caps_at_9000() {
        let mut reg = SpecialPoints::new();
        reg.add_special_point(1, 2, TileCounterType::WindyGrass);
        reg.add_special_point(1, 2, TileCounterType::WindyGrass);
        assert_eq!(reg.buckets[1], vec![(1, 2), (1, 2)]);
        for i in 0..(MAX_SPECIALS + 5) {
            reg.add_special_point(i as i32, 0, TileCounterType::MultiTileGrass);
        }
        assert_eq!(reg.buckets[2].len(), MAX_SPECIALS);
        assert_eq!(reg.buckets[2][0], (0, 0));
    }

    /// Witness: `ClearSpecialBlockCounts` (`:388-402`) clears the two root
    /// lists and the ten buckets in the C# order — index 5 (BiomeGrass) is
    /// the one bucket the method never touches.
    #[test]
    fn w4_g30_clear_skips_the_biome_grass_bucket() {
        let mut reg = SpecialPoints::new();
        for idx in 0..11 {
            reg.buckets[idx].push((idx as i32, 0));
        }
        reg.vine_roots_seen.insert((1, 1));
        reg.reverse_vine_roots_seen.insert((2, 2));
        reg.liquid_drop_cells.push((3, 3, 373));
        reg.single_tiles = 7;
        reg.clear_special_block_counts();
        for idx in [0usize, 1, 2, 3, 4, 6, 7, 8, 9, 10] {
            assert!(reg.buckets[idx].is_empty(), "bucket {idx}");
        }
        assert_eq!(reg.buckets[5], vec![(5, 0)]);
        assert!(reg.vine_roots_seen.is_empty());
        assert!(reg.reverse_vine_roots_seen.is_empty());
        assert!(reg.liquid_drop_cells.is_empty());
        assert_eq!(reg.single_tiles, 0);
    }

    /// Witness: `IsTileDrawLayerSolid` (`:4469-4476`) — the five explicit
    /// `DrawTileInSolidLayer` overrides (`TileID.cs:516-527`: 11/78/470/475/
    /// 579 all `true`) win over `Main.tileSolid`; everything else falls to
    /// the solid set. 11 (open door) and 470/475 (display doll/hat rack) are
    /// NOT in `tileSolid`, so the override is observable.
    #[test]
    fn w4_g30_the_solid_layer_override_beats_tile_solid() {
        for kind in [11u16, 78, 470, 475, 579] {
            assert!(is_tile_draw_layer_solid(kind), "override kind {kind}");
        }
        // Dirt (0) is solid, grass-covered dirt included; a vine (52) and a
        // torch (4) are not — no override, so the solid set decides.
        assert!(is_tile_draw_layer_solid(0));
        assert!(!is_tile_draw_layer_solid(52));
        assert!(!is_tile_draw_layer_solid(4));
        // Out-of-table kinds fall through without panicking.
        assert!(!is_tile_draw_layer_solid(700));
        assert!(!is_tile_draw_layer_solid(u16::MAX));
    }

    /// Witness: `ShouldSwayInWind` (`:7702-7717`) — the settings gate, the
    /// `SwaysInWindBasic` membership, and the frame exceptions gated on
    /// `type == 227` ONLY (the inline port's every-kind exception is the
    /// deviation this gate fixes).
    #[test]
    fn w4_g30_sway_gate_is_settings_set_and_227_only() {
        // 83 is a SwaysInWindBasic herb; 227 is the dye plant.
        assert!(should_sway_in_wind(83, 0, true));
        assert!(!should_sway_in_wind(83, 0, false));
        assert!(!should_sway_in_wind(0, 0, true)); // dirt never sways
        // The five exception frames kill the sway on 227 only.
        for fx in [204i16, 238, 408, 442, 476] {
            assert!(!should_sway_in_wind(227, fx, true), "227 frame {fx}");
            // 83 at the same frameX still sways — C# gates on type == 227.
            assert!(should_sway_in_wind(83, fx, true), "83 frame {fx}");
        }
        assert!(should_sway_in_wind(227, 0, true));
        assert!(should_sway_in_wind(227, 18, true));
    }

    /// Witness: the vine arms (`:536-548`) — all eight VineThreads ids crawl
    /// forward, 549 crawls reverse, regardless of frame.
    #[test]
    fn w4_g30_vine_threads_crawl_and_549_reverses() {
        for kind in [52u16, 62, 115, 205, 382, 528, 636, 638] {
            assert_eq!(classify_tile(kind, 0, 0, true), CellClass::VineCrawl);
            assert_eq!(classify_tile(kind, 17, 35, true), CellClass::VineCrawl);
        }
        assert_eq!(classify_tile(549, 0, 0, true), CellClass::ReverseVineCrawl);
    }

    /// Witness: the MultiTileVine anchor gates (`:549-599`) — each family's
    /// `%` grid, and the `continue` OUTSIDE the gate: an off-anchor cell
    /// skips, it never falls to the default sway arm.
    #[test]
    fn w4_g30_multi_tile_vine_anchors_and_off_grid_skips() {
        use CellClass::*;
        use TileCounterType::MultiTileVine as V;
        // 34: 54×54 grid; 698: 18-wide row 0; 454: 72×54.
        assert_eq!(classify_tile(34, 0, 0, true), Deferred(V));
        assert_eq!(classify_tile(34, 54, 108, true), Deferred(V));
        assert_eq!(classify_tile(34, 18, 0, true), Skip);
        assert_eq!(classify_tile(698, 18, 0, true), Deferred(V));
        assert_eq!(classify_tile(698, 0, 18, true), Skip);
        assert_eq!(classify_tile(454, 72, 54, true), Deferred(V));
        assert_eq!(classify_tile(454, 36, 0, true), Skip);
        // The 1×2 family: 18×36; 91: 18×54; 95/126/444: 36×36; 465/591/592: 36×54.
        for kind in [42u16, 270, 271, 572, 581, 660] {
            assert_eq!(classify_tile(kind, 18, 36, true), Deferred(V));
            assert_eq!(classify_tile(kind, 0, 18, true), Skip);
        }
        assert_eq!(classify_tile(91, 18, 54, true), Deferred(V));
        assert_eq!(classify_tile(91, 18, 36, true), Skip);
        for kind in [95u16, 126, 444] {
            assert_eq!(classify_tile(kind, 36, 36, true), Deferred(V));
            assert_eq!(classify_tile(kind, 18, 36, true), Skip);
        }
        for kind in [465u16, 591, 592] {
            assert_eq!(classify_tile(kind, 36, 54, true), Deferred(V));
            assert_eq!(classify_tile(kind, 36, 36, true), Skip);
        }
    }

    /// Witness: the MultiTileGrass anchors (`:600-687`) — the sunflower 27
    /// (36×row0), the potted plants, the 233 TWO-anchor jungle plant, the
    /// no-row-gate 652/651, the wind toys, the shards, the golf flag and the
    /// campfire column split.
    #[test]
    fn w4_g30_multi_tile_grass_anchors() {
        use CellClass::*;
        use TileCounterType::MultiTileGrass as G;
        assert_eq!(classify_tile(27, 36, 0, true), Deferred(G));
        assert_eq!(classify_tile(27, 0, 18, true), Skip);
        for kind in [236u16, 238, 702] {
            assert_eq!(classify_tile(kind, 36, 0, true), Deferred(G));
            assert_eq!(classify_tile(kind, 36, 18, true), Skip);
        }
        // 233: the 54-wide top at row 0 AND the 36-wide bulb at row 36.
        assert_eq!(classify_tile(233, 54, 0, true), Deferred(G));
        assert_eq!(classify_tile(233, 36, 36, true), Deferred(G));
        assert_eq!(classify_tile(233, 36, 0, true), Skip);
        assert_eq!(classify_tile(233, 54, 36, true), Skip);
        // 652/651: column gates only, no row gate.
        assert_eq!(classify_tile(652, 36, 54, true), Deferred(G));
        assert_eq!(classify_tile(652, 18, 0, true), Skip);
        assert_eq!(classify_tile(651, 54, 18, true), Deferred(G));
        assert_eq!(classify_tile(651, 36, 0, true), Skip);
        for kind in [485u16, 489, 490] {
            assert_eq!(classify_tile(kind, 36, 0, true), Deferred(G));
            assert_eq!(classify_tile(kind, 36, 18, true), Skip);
        }
        for kind in 521u16..=527 {
            assert_eq!(classify_tile(kind, 36, 0, true), Deferred(G));
            assert_eq!(classify_tile(kind, 0, 18, true), Skip);
        }
        assert_eq!(classify_tile(493, 18, 0, true), Deferred(G));
        assert_eq!(classify_tile(493, 18, 18, true), Skip);
        // 519: columns 0-4 register (frameX/18 <= 4), the rest skip.
        assert_eq!(classify_tile(519, 72, 0, true), Deferred(G)); // col 4
        assert_eq!(classify_tile(519, 90, 0, true), Skip); // col 5
    }

    /// Witness: the 530/705 frame-range split (`:636-655`) — below the cut
    /// the multi-tile gate applies (off-anchor skips); at/above the cut the
    /// `break` goes STRAIGHT to DrawSingleTile (`:645`/`:655`), never through
    /// the default arm.
    #[test]
    fn w4_g30_oasis_and_sea_oats_split_at_the_frame_cut() {
        use CellClass::*;
        use TileCounterType::MultiTileGrass as G;
        assert_eq!(classify_tile(530, 54, 0, true), Deferred(G));
        assert_eq!(classify_tile(530, 18, 0, true), Skip);
        assert_eq!(classify_tile(530, 270, 0, true), SingleTile);
        assert_eq!(classify_tile(530, 288, 18, true), SingleTile);
        // 705: the cut is modulo the 486 style block.
        assert_eq!(classify_tile(705, 54, 36, true), Deferred(G));
        assert_eq!(classify_tile(705, 18, 36, true), Skip);
        assert_eq!(classify_tile(705, 270, 0, true), SingleTile);
        assert_eq!(classify_tile(705, 486, 0, true), Deferred(G)); // 486%486=0
        assert_eq!(classify_tile(705, 486 + 270, 0, true), SingleTile);
    }

    /// Witness: the `break` arms (`:695-712`) — 491/597/617 register AND
    /// draw when on-anchor, draw alone otherwise; the drip tiles (`:688-694`)
    /// emit and skip; 184 (`:713-715`) defers unconditionally.
    #[test]
    fn w4_g30_break_arms_register_and_draw_drips_record() {
        use CellClass::*;
        assert_eq!(
            classify_tile(491, 18, 18, true),
            DeferredAndSingle(TileCounterType::VoidLens)
        );
        assert_eq!(classify_tile(491, 0, 0, true), SingleTile);
        assert_eq!(classify_tile(491, 18, 0, true), SingleTile);
        assert_eq!(
            classify_tile(597, 54, 0, true),
            DeferredAndSingle(TileCounterType::TeleportationPylon)
        );
        assert_eq!(classify_tile(597, 0, 18, true), SingleTile);
        assert_eq!(
            classify_tile(617, 54, 72, true),
            DeferredAndSingle(TileCounterType::MasterTrophy)
        );
        assert_eq!(classify_tile(617, 54, 36, true), SingleTile);
        for kind in [373u16, 374, 375, 461, 709] {
            assert_eq!(classify_tile(kind, 0, 0, true), LiquidDrops);
        }
        assert_eq!(
            classify_tile(184, 17, 35, true),
            Deferred(TileCounterType::AnyDirectionalGrass)
        );
    }

    /// Witness: the default arm (`:716-722`) — sway candidates defer as
    /// WindyGrass, everything else reaches DrawSingleTile; the settings gate
    /// flips the whole arm.
    #[test]
    fn w4_g30_the_default_arm_defers_sway_candidates() {
        assert_eq!(
            classify_tile(83, 0, 0, true),
            CellClass::Deferred(TileCounterType::WindyGrass)
        );
        assert_eq!(classify_tile(83, 0, 0, false), CellClass::SingleTile);
        assert_eq!(classify_tile(0, 0, 0, true), CellClass::SingleTile);
        // 227's exception frames fall through the default arm to SingleTile.
        assert_eq!(classify_tile(227, 204, 0, true), CellClass::SingleTile);
    }

    /// Witness: `CacheSpecialDraws_Part1`'s Tree tail (`:4260-4284`) — the
    /// 323 frond column band and the trunk family's branch rows; the gates
    /// read the DRAW-DATA frames (the caller passes the overrides).
    #[test]
    fn w4_g30_the_tree_tail_registers_canopy_cells() {
        let mut reg = SpecialPoints::new();
        // 323 palm: 88 <= frameX <= 132 inclusive, any row.
        cache_tree_point(1, 1, 323, 88, 0, &mut reg);
        cache_tree_point(2, 1, 323, 132, 54, &mut reg);
        cache_tree_point(3, 1, 323, 87, 0, &mut reg);
        cache_tree_point(4, 1, 323, 133, 0, &mut reg);
        // The trunk family: frameY >= 198 AND frameX >= 22.
        cache_tree_point(5, 1, 5, 22, 198, &mut reg);
        cache_tree_point(6, 1, 589, 44, 220, &mut reg);
        cache_tree_point(7, 1, 5, 22, 197, &mut reg);
        cache_tree_point(8, 1, 5, 21, 198, &mut reg);
        // Non-tree kinds never register.
        cache_tree_point(9, 1, 0, 22, 198, &mut reg);
        assert_eq!(reg.buckets[0], vec![(1, 1), (2, 1), (5, 1), (6, 1)]);
    }

    /// A 40x60 world with a reverse-vine (549) column at x=10, rows 20-22,
    /// standing on dirt at (10, 23).
    fn reverse_vine_world() -> World {
        let mut w = world(40, 60);
        put(&mut w, 10, 23, 0, 0, 0); // dirt — the ground attachment
        for y in 20..=22 {
            put(&mut w, 10, y, 549, 0, 0);
        }
        w
    }

    /// Witness: `CrawlToBottomOfReverseVineAndAddSpecialPoint` (`:764-782`)
    /// — every cell of one column resolves the SAME root (the row above the
    /// attachable dirt), deduped once; the loop starts AT j (`:767`), so an
    /// attachable cell at j itself roots at j-1 (the forward crawl starts
    /// at j-1 and would never test j).
    #[test]
    fn w4_g30_the_reverse_crawl_roots_above_the_attachment() {
        let w = reverse_vine_world();
        let mut reg = SpecialPoints::new();
        for y in 20..=22 {
            collect_reverse_vine_root(&w, 10, y, &mut reg);
        }
        assert_eq!(reg.buckets[TileCounterType::ReverseVine as usize], vec![(10, 22)]);
        // The k = j start: a solid cell at j roots immediately at j - 1.
        let mut w2 = world(40, 60);
        put(&mut w2, 10, 20, 0, 0, 0);
        let mut reg2 = SpecialPoints::new();
        collect_reverse_vine_root(&w2, 10, 20, &mut reg2);
        assert_eq!(reg2.buckets[TileCounterType::ReverseVine as usize], vec![(10, 19)]);
        // A lone vine cell with empty ground below: k = j + 1 is inactive,
        // so the root is the cell's own row (y = k - 1 = j).
        let mut w3 = world(40, 60);
        put(&mut w3, 10, 20, 549, 0, 0);
        let mut reg3 = SpecialPoints::new();
        collect_reverse_vine_root(&w3, 10, 20, &mut reg3);
        assert_eq!(reg3.buckets[TileCounterType::ReverseVine as usize], vec![(10, 20)]);
    }

    /// Witness: the driver (`:504-727`) — row-major order, the layer gate
    /// (`:517`), the vine crawl feeding bucket 4 with the S1-G-1a dedup, and
    /// the single-tile count. Fixture: dirt (solid) at (5,5), a torch
    /// (non-solid) at (6,5), a sunflower anchor + sub-cell at (7,5)/(8,5),
    /// and a vine strip hanging from dirt at (12,4).
    #[test]
    fn w4_g30_the_driver_walks_layers_and_collects_in_order() {
        let mut w = world(40, 40);
        put(&mut w, 5, 5, 0, 0, 0); // dirt — solid layer
        put(&mut w, 6, 5, 4, 0, 0); // torch — non-solid
        put(&mut w, 7, 5, 27, 0, 0); // sunflower anchor
        put(&mut w, 8, 5, 27, 18, 18); // sunflower sub-cell
        put(&mut w, 12, 4, 0, 0, 0); // vine attachment
        for y in 5..=7 {
            put(&mut w, 12, y, 52, 0, 0);
        }
        let mut reg = SpecialPoints::new();
        collect_special_points(&w, 0, 0, 39, 39, false, true, &mut reg);
        assert_eq!(reg.single_tiles, 1); // the torch only
        assert_eq!(reg.buckets[2], vec![(7, 5)]); // the sunflower anchor
        assert_eq!(reg.buckets[4], vec![(12, 5)]); // one deduped vine root
        // The solid pass adds BOTH dirt cells to the single count; the
        // registry keeps the non-solid points (C# clears only before the
        // non-solid pass, and the port clears once per frame before the
        // solid walk).
        collect_special_points(&w, 0, 0, 39, 39, true, true, &mut reg);
        assert_eq!(reg.single_tiles, 3);
        assert_eq!(reg.buckets[2], vec![(7, 5)]);
        // Row-major order: two anchors on one row collect left-to-right.
        let mut w2 = world(40, 40);
        put(&mut w2, 9, 5, 27, 0, 0);
        put(&mut w2, 3, 5, 27, 0, 0);
        put(&mut w2, 6, 4, 27, 0, 0);
        let mut reg2 = SpecialPoints::new();
        collect_special_points(&w2, 0, 0, 39, 39, false, true, &mut reg2);
        assert_eq!(reg2.buckets[2], vec![(6, 4), (3, 5), (9, 5)]);
    }
}
