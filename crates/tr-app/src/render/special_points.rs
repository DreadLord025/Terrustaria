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
