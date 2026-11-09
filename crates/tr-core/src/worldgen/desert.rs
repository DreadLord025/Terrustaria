//! The desert pass family (M5-DESERT-structures): `Full Desert` and the
//! `Dunes` surface carver — the vanilla underground desert and the dune
//! shapes, ported under the golden rule.
//!
//! Call graph of the family (the C# `genRand` stream is the port's per-pass
//! [`Rng`]; the hive's own `FastRandom` is the exact LCG port):
//!
//! ```text
//! Full Desert (WorldGen.cs:12500) — the retry loop over DesertBiome.Place
//!   └── DesertBiome.Place (DesertBiome.cs:13-46)
//!         ├── DesertDescription.CreateFromPlacement (:40-76)  [invalid → retry]
//!         ├── SandMound.Place (:9-46)  — the sand dome + PlaceSandColumn
//!         ├── one entrance of four (desert_entrances.rs; :25-40)
//!         ├── DesertHive.Place (:200-204)
//!         │     ├── ClusterGroup.FromDescription → Generate (:34-190)
//!         │     └── PlaceClustersArea (:224-381)  — the density ladder
//!         └── AddTileVariance (:383-469)  — the exposed-sand + props pass
//! Dunes (WorldGen.cs:11517) — desert_dunes.rs, the dune carver
//! ```
//!
//! Deviations (documented per the golden rule): the special-seed arms
//! (`remixWorldGen`, `tenthAnniversaryWorld`, `surfaceIsDesert`,
//! `notTheBees`, `dualDungeons`, `extraLiquid`, `skyblock`) have no port
//! model and are skipped, so their branches and extra draws never run; the
//! C#'s per-tile `Tile.SmoothSlope` effect map (the `PostPlacementEffect.Smooth`
//! flags) has no port equivalent — the carve's slopes are left as-is, and
//! `CleanupArea`'s `SquareWallFrame`/`TileFrame` pair runs through
//! [`crate::framing::reframe_area`]; the `Main.tileSolid[484] = false` poke is
//! a load-order detail the port's tile data does not model.

use crate::framing::reframe_area;
use crate::tile::{FLAG_ACTIVE, FLAG_HALF_BRICK, FLAG_LIQUID_HONEY, FLAG_LIQUID_LAVA, FLAG_LIQUID_SHIMMER};
use crate::tile_light::FastRandom;
use crate::world::World;
use super::noise::Rng;
use super::pipeline::GenVars;
use super::{TILE_HARDENED_SAND, TILE_SAND, TILE_SANDSTONE, TILE_STONE, WALL_SANDSTONE};

// --- the ids the desert paints (TileID.cs / WallID.cs) ----------------------

/// `TileID.DesertFossil` (`TileID.cs:1437`) — the hive core's 1-in-15 pocket.
const TILE_DESERT_FOSSIL: u16 = 404;
/// `TileID.AntlionLarva` (`TileID.cs:1599`) — the exposed-sandstone prop.
const TILE_ANTLION: u16 = 485;
/// `TileID.RollingCactus` (`TileID.cs:1597`).
const TILE_ROLLING_CACTUS: u16 = 484;
/// `TileID.Stalactite` (`TileID.cs:959`).
const TILE_STALACTITE: u16 = 165;
/// The desert pile tile the variance pass plants on exposed sandstone
/// (`AddTileVariance`'s `WorldGen.PlaceTile(..., 187, style 29 + Next(6))` —
/// `TileID.LargePiles2`, `TileID.cs:1003`).
const TILE_PILE: u16 = 187;
/// `WallID.HardenedSand` (`WallID.cs:537`) — the hive's outer shell wall.
const WALL_HARDENED_SAND: u16 = 216;
/// `TileID.Sets.Clouds` (`TileID.cs:505`) — the surface scan skips these.
const CLOUD_KINDS: [u16; 6] = [189, 196, 460, 717, 718, 719];
/// `DesertDescription.RowHasInvalidTiles` rejects a row holding one of these
/// (mud 59 / jungle grass 60 / ice 161 / snow 147 — `TileID.cs`).
const INVALID_ROW_KINDS: [u16; 4] = [59, 60, 161, 147];
/// The `DesertHive` FastRandom modifier (`DesertHive.cs:226`).
const HIVE_MODIFIER: u64 = 57005;

/// One rectangle — the `Microsoft.Xna.Framework.Rectangle` every C# shape
/// carries: `Right = X + Width`, `Bottom = Y + Height`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Rect {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) w: i32,
    pub(crate) h: i32,
}

impl Rect {
    pub(crate) fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }
    pub(crate) fn left(&self) -> i32 {
        self.x
    }
    pub(crate) fn right(&self) -> i32 {
        self.x + self.w
    }
    pub(crate) fn top(&self) -> i32 {
        self.y
    }
    pub(crate) fn bottom(&self) -> i32 {
        self.y + self.h
    }
    pub(crate) fn center_x(&self) -> i32 {
        self.x + self.w / 2
    }
    /// `Rectangle.Inflate` (`DesertBiome.cs:50-51`): grows both axes.
    pub(crate) fn inflate(&mut self, dx: i32, dy: i32) {
        self.x -= dx;
        self.y -= dy;
        self.w += dx * 2;
        self.h += dy * 2;
    }
}

/// The `Main.worldSurface` cap the surface line clamps its `Bottom` to —
/// set by the driver before the desert pass runs (the pass runs after the
/// terrain fill, so the field is already on the world; the copy keeps the
/// port's passes free of the whole-world read).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct DesertEnv {
    pub(crate) world_surface: f64,
}

/// `SurfaceMap` (`Desert/SurfaceMap.cs:6-74`): the per-column surface row the
/// desert's geometry and its entrances follow.
#[derive(Clone)]
pub(crate) struct SurfaceLine {
    heights: Vec<i32>,
    x: i32,
}

impl SurfaceLine {
    /// `SurfaceMap.FromArea` (`SurfaceMap.cs:44-74`): the first non-cloud
    /// active tile per column, scanning y = 50 … 50 + maxTilesY/2; a column
    /// that finds none ends at `maxTilesY/2 + 50`.
    pub(crate) fn from_area(world: &World, start_x: i32, width: i32) -> Self {
        let scan = world.height / 2;
        let mut heights = Vec::with_capacity(width.max(0) as usize);
        for i in start_x..start_x + width {
            let mut found = false;
            let mut num2 = 0i32;
            let mut j = 50;
            while j < 50 + scan {
                if let Some(t) = world.tile(i, j) {
                    if t.active() {
                        if CLOUD_KINDS.contains(&t.kind) {
                            found = false;
                        } else if !found {
                            num2 = j;
                            found = true;
                        }
                    }
                }
                if !found {
                    num2 = scan + 50;
                }
                j += 1;
            }
            heights.push(num2);
        }
        Self {
            heights,
            x: start_x,
        }
    }

    /// `this[absoluteX]` (`SurfaceMap.cs:19`).
    pub(crate) fn height(&self, x: i32) -> i32 {
        let idx = (x - self.x) as usize;
        if idx < self.heights.len() {
            self.heights[idx]
        } else {
            *self.heights.last().unwrap_or(&0)
        }
    }

    /// `Bottom` (`SurfaceMap.cs:34-38`): the deepest row, clamped to
    /// `Main.worldSurface - 10`.
    pub(crate) fn bottom(&self, world_surface: f64) -> i32 {
        let max = *self.heights.iter().max().unwrap_or(&0);
        max.min((world_surface - 10.0) as i32)
    }

    /// `Top` (`SurfaceMap.cs:39`): the shallowest row.
    pub(crate) fn top(&self) -> i32 {
        *self.heights.iter().min().unwrap_or(&0)
    }

    /// `Width` (`SurfaceMap.cs:17`): the line's column count.
    pub(crate) fn width(&self) -> i32 {
        self.heights.len() as i32
    }

    /// `Average` (`SurfaceMap.cs:40`): the mean row as a double.
    pub(crate) fn average(&self) -> f64 {
        let sum: i64 = self.heights.iter().map(|h| *h as i64).sum();
        sum as f64 / self.heights.len() as f64
    }
}

/// `DesertDescription` (`Desert/DesertDescription.cs:7-106`): the desert's
/// geometry — the combined band, the sand `Desert` half and the underground
/// `Hive`, plus the block grid the cluster walker fills.
#[derive(Clone)]
pub(crate) struct DesertGeom {
    pub(crate) combined: Rect,
    pub(crate) desert: Rect,
    pub(crate) hive: Rect,
    pub(crate) block_columns: i32,
    pub(crate) block_rows: i32,
    /// The surface line the geometry was placed against; re-scanned after the
    /// mound carves (`UpdateSurfaceMap`, `:35-38`).
    pub(crate) surface: SurfaceLine,
    pub(crate) valid: bool,
}

impl DesertGeom {
    /// `CreateFromPlacement` (`DesertDescription.cs:40-76`): the band's width
    /// `80 * maxTilesX/4200`, the height `(NextDouble()*0.5 + 1.5) * 170 * n`,
    /// the block scale (4, 2), the `RowHasInvalidTiles` rejection and the hive
    /// top `(Average + Bottom)/2 + Next(40,60)`.
    pub(crate) fn create_from_placement(
        world: &World,
        origin_x: i32,
        world_surface: f64,
        skip_check: bool,
        rng: &mut Rng,
    ) -> Self {
        let num = world.width as f64 / 4200.0;
        let columns = (80.0 * num) as i32;
        let rows = ((rng.next_f64() * 0.5 + 1.5) * 170.0 * num) as i32;
        // `DefaultBlockScale = (4.0, 2.0)` (`:105`).
        let num4 = (4.0 * columns as f64) as i32;
        let num5 = (2.0 * rows as f64) as i32;
        let origin_x = origin_x - num4 / 2;
        let surface = SurfaceLine::from_area(world, origin_x - 5, num4 + 10);
        if Self::row_has_invalid_tiles(world, origin_x, surface.bottom(world_surface), num4, skip_check)
        {
            return Self::invalid(surface);
        }
        let num6 = (surface.average() + surface.bottom(world_surface) as f64) as i32 / 2;
        let origin_y = num6 + rng.range(40, 60);
        Self {
            combined: Rect::new(origin_x, num6, num4, origin_y + num5 - num6),
            hive: Rect::new(origin_x, origin_y, num4, num5),
            desert: Rect::new(origin_x, num6, num4, origin_y + num5 / 2 - num6),
            block_columns: columns,
            block_rows: rows,
            surface,
            valid: true,
        }
    }

    fn invalid(surface: SurfaceLine) -> Self {
        Self {
            combined: Rect::new(0, 0, 0, 0),
            desert: Rect::new(0, 0, 0, 0),
            hive: Rect::new(0, 0, 0, 0),
            block_columns: 0,
            block_rows: 0,
            surface,
            valid: false,
        }
    }

    /// `RowHasInvalidTiles` (`DesertDescription.cs:78-97`): a candidate row over
    /// mud/jungle grass/ice/snow is rejected (the `skipDesertTileCheck` escape
    /// after two failed sides, `:80-83`).
    fn row_has_invalid_tiles(
        world: &World,
        start_x: i32,
        start_y: i32,
        width: i32,
        skip_check: bool,
    ) -> bool {
        if skip_check {
            return false;
        }
        for i in start_x..start_x + width {
            let kind = match world.tile(i, start_y) {
                Some(t) if t.active() => t.kind,
                _ => continue,
            };
            if INVALID_ROW_KINDS.contains(&kind) {
                return true;
            }
        }
        false
    }

    /// `UpdateSurfaceMap` (`DesertDescription.cs:35-38`).
    pub(crate) fn update_surface(&mut self, world: &World) {
        self.surface = SurfaceLine::from_area(world, self.combined.x - 5, self.combined.w + 10);
    }
}

// --- SandMound ---------------------------------------------------------------

/// `SandMound` (`Desert/SandMound.cs:9-46`): the sand dome above the hive —
/// clears the mound's interior, then drops a sand column per column with the
/// quartic falloff and the ±10 random walk.
fn sand_mound_place(world: &mut World, desc: &DesertGeom, rng: &mut Rng) {
    let mut desert = desc.desert;
    desert.h = desert.h.min(desc.hive.h / 2);
    let mut desert2 = desc.desert;
    desert2.y = desert.bottom();
    desert2.h = (desc.desert.bottom() - desert.bottom()).max(0);
    let surface = &desc.surface;
    let mut num = 0i32;
    let mut num2 = 0i32;
    for i in -5..desert.w + 5 {
        let num4 = (((i + 5) as f64 / (desert.w + 10) as f64) * 2.0 - 1.0)
            .abs()
            .clamp(-1.0, 1.0);
        if i % 3 == 0 {
            num = (num + rng.range(-1, 2)).clamp(-10, 10);
        }
        num2 = (num2 + rng.range(-1, 2)).clamp(-10, 10);
        let num5 = (1.0 - num4 * num4 * num4 * num4).max(0.0).sqrt();
        let num6 = desert.bottom() - (num5 * desert.h as f64) as i32 + num;
        if num4.abs() < 1.0 {
            // `Utils.UnclampedSmoothStep(0.5, 0.8, |num4|)` (`Utils.cs:141-144`).
            let t = ((num4.abs() - 0.5) / 0.3).clamp(0.0, 1.0);
            let num7 = t * t * (3.0 - 2.0 * t);
            let num8 = (10 + (desert.top() as f64 - num7 * num7 * num7 * 20.0) as i32 + num2)
                .min(num6);
            let x = i + desert.x;
            let mut j = surface.height(x) - 1;
            while j < num8 {
                if let Some(t) = world.tile_mut(x, j) {
                    t.flags &= !FLAG_ACTIVE;
                    t.wall = 0;
                }
                j += 1;
            }
        }
        place_sand_column(world, i + desert.x, num6, desert2.bottom() - num6);
    }
}

/// `PlaceSandColumn` (`SandMound.cs:48-70`): sand from the column's top down to
/// its bottom, the liquid zeroed (the non-remix arm) and the wall framed.
fn place_sand_column(world: &mut World, start_x: i32, start_y: i32, height: i32) {
    let mut num = start_y + height - 1;
    while num >= start_y {
        if let Some(t) = world.tile_mut(start_x, num) {
            t.liquid = 0;
            t.kind = TILE_SAND;
            t.set_slope(0);
            t.flags &= !FLAG_HALF_BRICK;
            t.flags |= FLAG_ACTIVE;
        }
        num -= 1;
    }
}

// --- DesertHive: the cluster walker ------------------------------------------

/// `DesertHive.Block` (`DesertHive.cs:12-15`): one cluster cell, the jittered
/// position in block space.
#[derive(Clone, Copy)]
struct Block {
    x: f64,
    y: f64,
}

/// `DesertHive.ClusterGroup` (`DesertHive.cs:21-191`): the ellipse mask, the
/// flood clusters, the border claiming and the ±0.25 jitter — the density
/// field the hive carves from.
pub(crate) struct ClusterGroup {
    width: i32,
    height: i32,
    clusters: Vec<Vec<Block>>,
}

impl ClusterGroup {
    /// `FromDescription` (`:34-37`): the block grid is the description's
    /// column/row counts.
    fn from_description(desc: &DesertGeom, rng: &mut Rng) -> Self {
        Self::new(desc.block_columns, desc.block_rows, rng)
    }

    fn new(width: i32, height: i32, rng: &mut Rng) -> Self {
        Self {
            width,
            height,
            clusters: generate_clusters(width, height, rng),
        }
    }

    fn len(&self) -> usize {
        self.clusters.len()
    }
}

/// `ClusterGroup.Generate` (`DesertHive.cs:79-190`): the whole
/// mask/cluster pipeline on one `genRand` stream.
fn generate_clusters(width: i32, height: i32, rng: &mut Rng) -> Vec<Vec<Block>> {
    let cells = (width.max(0) * height.max(0)) as usize;
    // The ellipse mask (`:87-95`): one `Next(2)` per in-ellipse cell.
    let mut mask = vec![false; cells];
    let num = width / 2 - 1;
    let num2 = height / 2 - 1;
    let num3 = (num + 1) * (num + 1);
    let cx = num;
    let cy = num2;
    let mut i = cy - num2;
    while i <= cy + num2 {
        let num4 = num as f64 / num2 as f64 * (i - cy) as f64;
        let root = (num3 as f64 - num4 * num4).sqrt() as i32;
        let num5 = root.min(num);
        let mut j = cx - num5;
        while j <= cx + num5 {
            if j >= 0 && j < width && i >= 0 && i < height {
                mask[(j + i * width) as usize] = rng.chance(2);
            }
            j += 1;
        }
        i += 1;
    }
    // Cluster seeds (`:97-111`): a second `Next(2)` per cell gates the flood.
    let mut point_clusters: Vec<Vec<(i32, i32)>> = Vec::new();
    let mut k = 0;
    while k < width {
        let mut l = 0;
        while l < height {
            let idx = (k + l * width) as usize;
            if mask[idx] && rng.chance(2) {
                let mut cluster = Vec::new();
                search_for_cluster(&mut mask, width, height, &mut cluster, k, l, 2);
                if cluster.len() > 2 {
                    point_clusters.push(cluster);
                }
            }
            l += 1;
        }
        k += 1;
    }
    // The index map and the border claims (`:112-155`).
    let mut index_map = vec![-1i32; cells];
    for (n, cluster) in point_clusters.iter().enumerate() {
        for &(x, y) in cluster {
            index_map[(x + y * width) as usize] = n as i32;
        }
    }
    for n in 0..point_clusters.len() {
        for &(x, y) in &point_clusters[n] {
            if index_map[(x + y * width) as usize] == -1 {
                break;
            }
            let index = index_map[(x + y * width) as usize];
            if x > 0 {
                attempt_claim(&mut index_map, &point_clusters, width, height, x - 1, y, index, rng);
            }
            if x < width - 1 {
                attempt_claim(&mut index_map, &point_clusters, width, height, x + 1, y, index, rng);
            }
            if y > 0 {
                attempt_claim(&mut index_map, &point_clusters, width, height, x, y - 1, index, rng);
            }
            if y < height - 1 {
                attempt_claim(&mut index_map, &point_clusters, width, height, x, y + 1, index, rng);
            }
        }
    }
    // Rebuild (`:156-176`): the claimed cells regroup, clusters under 4 blocks
    // drop.
    for cluster in &mut point_clusters {
        cluster.clear();
    }
    for x in 0..width {
        for y in 0..height {
            let idx = (x + y * width) as usize;
            if index_map[idx] != -1 {
                let n = index_map[idx] as usize;
                point_clusters[n].push((x, y));
            }
        }
    }
    for cluster in &mut point_clusters {
        if cluster.len() < 4 {
            cluster.clear();
        }
    }
    // Jittered blocks (`:177-189`).
    let mut clusters = Vec::new();
    for cluster in &point_clusters {
        if cluster.is_empty() {
            continue;
        }
        let mut blocks = Vec::with_capacity(cluster.len());
        for &(x, y) in cluster {
            blocks.push(Block {
                x: x as f64 + (rng.next_f64() - 0.5) * 0.5,
                y: y as f64 + (rng.next_f64() - 0.5) * 0.5,
            });
        }
        clusters.push(blocks);
    }
    clusters
}

/// `SearchForCluster` (`DesertHive.cs:39-63`): the recursive flood with the
/// two-level depth cap.
fn search_for_cluster(
    mask: &mut [bool],
    width: i32,
    height: i32,
    out: &mut Vec<(i32, i32)>,
    x: i32,
    y: i32,
    level: i32,
) {
    if x < 0 || y < 0 || x >= width || y >= height {
        return;
    }
    let idx = (x + y * width) as usize;
    if !mask[idx] {
        return;
    }
    out.push((x, y));
    mask[idx] = false;
    let level = level - 1;
    if level != -1 {
        search_for_cluster(mask, width, height, out, x - 1, y, level);
        search_for_cluster(mask, width, height, out, x + 1, y, level);
        search_for_cluster(mask, width, height, out, x, y - 1, level);
        search_for_cluster(mask, width, height, out, x, y + 1, level);
    }
}

/// `AttemptClaim` (`DesertHive.cs:65-77`): a border cell steals its
/// neighbour's cluster on a `Next(2)` coin.
fn attempt_claim(
    index_map: &mut [i32],
    clusters: &[Vec<(i32, i32)>],
    width: i32,
    height: i32,
    x: i32,
    y: i32,
    index: i32,
    rng: &mut Rng,
) {
    if x < 0 || y < 0 || x >= width || y >= height {
        return;
    }
    let num = index_map[(x + y * width) as usize];
    if num == -1 || num == index {
        return;
    }
    let num2 = if rng.chance(2) { -1 } else { index };
    for &(px, py) in &clusters[num as usize] {
        index_map[(px + py * width) as usize] = num2;
    }
}

/// `DesertHive.PlaceClustersArea` (`DesertHive.cs:224-381`): the metaball carve
/// — the summed inverse-square cluster density decides the concentric shells.
/// Returns the carved bounds (`UpdateDesertHiveBounds`, `WorldGen.cs:10989`).
fn place_clusters_area(
    world: &mut World,
    desc: &DesertGeom,
    clusters: &ClusterGroup,
    area: Rect,
    world_surface: f64,
    fast: &mut FastRandom,
) -> (i32, i32, i32, i32) {
    let hive_w = desc.hive.w as f64;
    let hive_h = desc.hive.h as f64;
    let cluster_w = clusters.width as f64;
    let cluster_h = clusters.height as f64;
    // `description.BlockScale / 2.0` (`:229`) — (2.0, 1.0).
    let mut bounds = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
    for i in area.left()..area.right() {
        for j in area.top()..area.bottom() {
            if i <= 0 || j <= 0 || i >= world.width - 1 || j >= world.height - 1 {
                continue;
            }
            let mut num = 0.0;
            let mut best_index: i32 = -1;
            let mut num3 = 0.0;
            let mut tile_type: u16 = TILE_SAND;
            if fast.next(3) == 0 {
                tile_type = TILE_HARDENED_SAND;
            }
            let num4 = i - desc.hive.x;
            let num5 = j - desc.hive.y;
            let value_x = (num4 as f64 - 2.0) / hive_w * cluster_w;
            let value_y = (num5 as f64 - 1.0) / hive_h * cluster_h;
            for (k, cluster) in clusters.clusters.iter().enumerate() {
                let Some(first) = cluster.first() else { continue };
                if (first.x - value_x).abs() > 10.0 || (first.y - value_y).abs() > 10.0 {
                    continue;
                }
                let mut num6 = 0.0;
                for block in cluster {
                    let dx = block.x - value_x;
                    let dy = block.y - value_y;
                    num6 += 1.0 / (dx * dx + dy * dy);
                }
                if num6 > num {
                    if num > num3 {
                        num3 = num;
                    }
                    num = num6;
                    best_index = k as i32;
                } else if num6 > num3 {
                    num3 = num6;
                }
            }
            let num7 = num + num3;
            // The ellipse-boundary test (`:279`): `length(uv * 2 - 1) >= 0.8`.
            let ex = (num4 as f64 - 2.0) / hive_w * 2.0 - 1.0;
            let ey = (num5 as f64 - 1.0) / hive_h * 2.0 - 1.0;
            let flag = (ex * ex + ey * ey).sqrt() >= 0.8;
            let mut flag2 = true;
            let mut write_tile: Option<u16> = None;
            let mut wall: Option<u16> = None;
            if num7 > 3.5 {
                // `tile.ClearEverything()` (`Tile.cs:110-121`) then the wall.
                if let Some(t) = world.tile_mut(i, j) {
                    t.kind = 0;
                    t.flags = 0;
                    t.meta = 0;
                    t.liquid = 0;
                    t.frame_x = 0;
                    t.frame_y = 0;
                    t.wall = WALL_SANDSTONE;
                }
                if best_index % 15 == 2 {
                    write_tile = Some(TILE_DESERT_FOSSIL);
                }
            } else if num7 > 1.8 {
                wall = Some(WALL_SANDSTONE);
                if (j as f64) < world_surface {
                    if let Some(t) = world.tile_mut(i, j) {
                        t.liquid = 0;
                    }
                } else if let Some(t) = world.tile_mut(i, j) {
                    // `tile.lava(lava: true)` (`Tile.cs:345`).
                    t.liquid = 255;
                    t.flags &= !(FLAG_LIQUID_HONEY | FLAG_LIQUID_SHIMMER);
                    t.flags |= FLAG_LIQUID_LAVA;
                }
                if !flag || world.tile(i, j).is_some_and(|t| t.active()) {
                    write_tile = Some(TILE_SANDSTONE);
                }
            } else if num7 > 0.7 || !flag {
                wall = Some(WALL_HARDENED_SAND);
                if let Some(t) = world.tile_mut(i, j) {
                    t.liquid = 0;
                }
                if !flag || world.tile(i, j).is_some_and(|t| t.active()) {
                    write_tile = Some(tile_type);
                }
            } else if num7 > 0.25 {
                let mut derived = fast.with_modifier(num4, num5);
                let num8 = (num7 - 0.25) / 0.45;
                if derived.next_double() < num8 {
                    wall = Some(WALL_SANDSTONE);
                    if (j as f64) < world_surface {
                        if let Some(t) = world.tile_mut(i, j) {
                            t.liquid = 0;
                        }
                    } else if let Some(t) = world.tile_mut(i, j) {
                        t.liquid = 255;
                        t.flags &= !(FLAG_LIQUID_HONEY | FLAG_LIQUID_SHIMMER);
                        t.flags |= FLAG_LIQUID_LAVA;
                    }
                    if world.tile(i, j).is_some_and(|t| t.active()) {
                        write_tile = Some(tile_type);
                    }
                }
            } else {
                flag2 = false;
            }
            if let Some(kind) = write_tile {
                // `tile.ResetToType(kind)` (`Tile.cs:249-259`).
                if let Some(t) = world.tile_mut(i, j) {
                    t.kind = kind;
                    t.flags = FLAG_ACTIVE;
                    t.meta = 0;
                    t.liquid = 0;
                    t.frame_x = 0;
                    t.frame_y = 0;
                }
            }
            if let Some(wall_kind) = wall {
                if let Some(t) = world.tile_mut(i, j) {
                    t.wall = wall_kind;
                }
            }
            if flag2 {
                bounds.0 = bounds.0.min(i);
                bounds.1 = bounds.1.min(j);
                bounds.2 = bounds.2.max(i);
                bounds.3 = bounds.3.max(j);
            }
        }
    }
    if bounds.0 == i32::MAX {
        (0, 0, 0, 0)
    } else {
        bounds
    }
}

/// `AddTileVariance` (`DesertHive.cs:383-469`): the exposed sand hardens and
/// the exposed sandstone grows props.
fn add_tile_variance(world: &mut World, desc: &DesertGeom, rng: &mut Rng) {
    // The first sweep (`:385-402`): sand with a non-solid tile below hardens.
    for i in -20..desc.hive.w + 20 {
        for j in -20..desc.hive.h + 20 {
            let (x, y) = (i + desc.hive.x, j + desc.hive.y);
            let Some(t) = world.tile(x, y) else { continue };
            if t.kind != TILE_SAND {
                continue;
            }
            let below1 = world.tile(x, y + 1).is_some_and(|t| t.active());
            let below2 = world.tile(x, y + 2).is_some_and(|t| t.active());
            if !below1 || !below2 {
                if let Some(t) = world.tile_mut(x, y) {
                    t.kind = TILE_HARDENED_SAND;
                }
            }
        }
    }
    // The second sweep (`:403-465`): props on exposed sandstone.
    for i in -20..desc.hive.w + 20 {
        for j in -20..desc.hive.h + 20 {
            let (x, y) = (i + desc.hive.x, j + desc.hive.y);
            // `InWorld(x, y, 5)` (`:409`).
            if x < 5 || y < 5 || x >= world.width - 5 || y >= world.height - 5 {
                continue;
            }
            let Some(t) = world.tile(x, y) else { continue };
            if !t.active() || t.kind != TILE_SANDSTONE {
                continue;
            }
            let mut flag = true;
            for num5 in -3..0 {
                for dx in [0, 1] {
                    if world.tile(x + dx, y + num5).is_some_and(|t| t.active()) {
                        flag = false;
                    }
                }
            }
            let mut flag2 = true;
            for m in 1..=3 {
                for dx in [0, 1] {
                    if world.tile(x + dx, y + m).is_some_and(|t| t.active()) {
                        flag2 = false;
                    }
                }
            }
            if flag && rng.chance(20) {
                // The Antlion Larva (or its 1-in-30 palworld variant — no port
                // sheet, so the style roll stands alone).
                let style = rng.range(0, 4);
                place_tile(world, x, y - 1, TILE_ANTLION, style);
            } else if flag && rng.chance(5) {
                place_tile(world, x, y - 1, TILE_ROLLING_CACTUS, 0);
            } else if (flag ^ flag2) && rng.chance(5) {
                place_tile(world, x, y + if flag { -1 } else { 1 }, TILE_STALACTITE, 0);
            } else if flag && rng.chance(5) {
                let style = 29 + rng.range(0, 6);
                place_tile(world, x, y - 1, TILE_PILE, style);
            }
        }
    }
}

/// `WorldGen.PlaceTile(x, y, type, style)`'s world write (`AddTileVariance`'s
/// props): an active tile with an explicit style frame.
fn place_tile(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    if let Some(t) = world.tile_mut(x, y) {
        t.kind = kind;
        t.flags |= FLAG_ACTIVE;
        t.set_slope(0);
        t.frame_x = (style * 18) as i16;
        t.frame_y = 0;
    }
}

/// `CleanupArea` (`DesertBiome.cs:55-71`): the hive ±20 wall/tile reframe.
fn cleanup_area(world: &mut World, hive: Rect) {
    reframe_area(world, hive.center_x(), hive.y + hive.h / 2, hive.w / 2 + 20);
}

/// `Full Desert` (`WorldGen.cs:12500-12543` + `DesertBiome.Place`): one
/// underground desert, placed opposite the dungeon with the vanilla retry
/// loop. Writes `GenVars.UndergroundDesertLocation`/`HiveLocation` and the
/// hive bounds through `gv`.
pub(crate) fn full_desert(
    world: &mut World,
    gv: &mut GenVars,
    dungeon_side: i32,
    world_surface: f64,
    rng: &mut Rng,
) -> bool {
    let mut num = 0;
    let mut num2 = dungeon_side;
    let num3 = world.width / 2;
    let mut x = num3 + (rng.range(0, num3) / 8 + num3 / 8) * -num2;
    let mut num4 = 0;
    let mut skip_check = false;
    let mut placed = false;
    loop {
        let mut desc = DesertGeom::create_from_placement(
            world,
            x,
            world_surface,
            skip_check,
            rng,
        );
        if desc.valid {
            // `ExportDescriptionToEngine` (`:48-53`).
            let mut underground = desc.combined;
            underground.inflate(10, 10);
            gv.underground_desert = Some(underground);
            gv.desert_hive = Some(desc.hive);
            // `SandMound.Place` (`:21`).
            sand_mound_place(world, &desc, rng);
            // `UpdateSurfaceMap` (`:22`).
            desc.update_surface(world);
            // The entrance roll (`:23-40`): the JSON `ChanceOfEntrance` 0.5
            // gate, then the `Next(4)` style pick.
            if rng.next_f64() <= 0.5 {
                let kind = rng.range(0, 4);
                super::desert_entrances::place_entrance(
                    world,
                    kind,
                    &desc,
                    world_surface,
                    rng,
                );
            }
            // `DesertHive.Place` (`:41`).
            let clusters = ClusterGroup::from_description(&desc, rng);
            let mut hive = desc.hive;
            hive.inflate(20, 20);
            let mut fast = FastRandom::new(world.seed as i32 as u64).with_modifier_u64(HIVE_MODIFIER);
            let (left, top, right, bottom) =
                place_clusters_area(world, &desc, &clusters, hive, world_surface, &mut fast);
            if clusters.len() > 0 {
                gv.desert_hive_bounds = Some((left, top, right, bottom));
            }
            // `AddTileVariance` (`:44` via `DesertHive.Place`).
            add_tile_variance(world, &desc, rng);
            cleanup_area(world, desc.hive);
            placed = true;
            break;
        }
        x = num3 + (rng.range(0, num3) / 2 + num3 / 8 + rng.range(0, num4 / 12)) * -num2;
        num4 += 1;
        if num4 > world.width / 4 {
            num2 *= -1;
            num4 = 0;
            num += 1;
            if num >= 2 {
                skip_check = true;
            }
        }
    }
    placed
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A flat stone world with a sand top — the desert pass's fixture.
    fn flat(w: i32, h: i32) -> World {
        let mut world = World::new("desert".into(), w, h);
        for x in 0..w {
            for y in 40..h {
                let i = world.index(x, y);
                world.tiles[i].kind = TILE_STONE;
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        world
    }

    #[test]
    fn surface_line_finds_the_first_solid_row() {
        let world = flat(200, 300);
        let line = SurfaceLine::from_area(&world, 10, 50);
        // The scan starts at y = 50 (`SurfaceMap.cs:51`), so a column filled
        // from row 40 down still reports 50.
        assert_eq!(line.height(10), 50);
        assert_eq!(line.average(), 50.0);
        assert_eq!(line.top(), 50);
        assert_eq!(line.bottom(world.world_surface), 50);
    }

    #[test]
    fn desert_geometry_uses_the_block_scale_quartet() {
        // The 4200-wide world: columns = 80, rows ∈ [1.5, 2.0) * 170, the band
        // is 4×80 wide and 2×rows tall (`DesertDescription.cs:43-70`).
        let world = flat(4200, 1200);
        let mut rng = Rng(0x5EED);
        let desc = DesertGeom::create_from_placement(&world, 2100, world.world_surface, false, &mut rng);
        assert!(desc.valid);
        assert_eq!(desc.block_columns, 80);
        assert!((151..=340).contains(&desc.block_rows));
        assert_eq!(desc.combined.w, 320);
        assert_eq!(desc.hive.w, 320);
        assert_eq!(desc.hive.h, desc.block_rows * 2);
        // The `Desert` half is NOT clamped here — the C# builds it
        // `origin.Y + num5/2 - num6` tall (`:69`), which overshoots
        // `Hive.Height / 2` by the 40-60 hive-top offset; `SandMound.Place`
        // clamps it (`SandMound.cs:12`).
        assert_eq!(desc.desert.h, desc.hive.h / 2 + (desc.hive.y - desc.combined.y));
        // The hive top sits 40-60 rows under the band top.
        assert!((40..=60).contains(&(desc.hive.y - desc.combined.y)));
    }

    #[test]
    fn sand_mound_drops_sand_columns_under_the_cleared_dome() {
        let mut world = flat(4200, 1200);
        let mut rng = Rng(0x5EED);
        let mut desc =
            DesertGeom::create_from_placement(&world, 2100, world.world_surface, false, &mut rng);
        desc.update_surface(&mut world);
        let before = world
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == TILE_SAND)
            .count();
        sand_mound_place(&mut world, &desc, &mut rng);
        let after = world
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == TILE_SAND)
            .count();
        assert!(after > before, "the mound drops sand ({before} -> {after})");
    }

    #[test]
    fn the_hive_ladder_lays_the_concentric_shells() {
        let mut world = flat(4200, 1200);
        world.world_surface = 400.0;
        let mut rng = Rng(0x5EED);
        let desc =
            DesertGeom::create_from_placement(&world, 2100, world.world_surface, false, &mut rng);
        let clusters = ClusterGroup::new(desc.block_columns, desc.block_rows, &mut rng);
        assert!(clusters.len() > 0, "the ellipse seeds clusters");
        let mut hive = desc.hive;
        hive.inflate(20, 20);
        let mut fast = FastRandom::new(world.seed as i32 as u64).with_modifier_u64(HIVE_MODIFIER);
        let surface_cap = world.world_surface;
        let (left, top, right, bottom) =
            place_clusters_area(&mut world, &desc, &clusters, hive, surface_cap, &mut fast);
        let count = |kind: u16| {
            world
                .tiles
                .iter()
                .filter(|t| t.active() && t.kind == kind)
                .count()
        };
        let walls = |wall: u16| world.tiles.iter().filter(|t| t.wall == wall).count();
        // The shells: the hollow core (sand/fossil), the sandstone mid band and
        // the hardened-sand outer band with their walls.
        assert!(count(TILE_SANDSTONE) > 0, "no sandstone band");
        assert!(walls(WALL_SANDSTONE) > 0, "no sandstone walls");
        assert!(walls(WALL_HARDENED_SAND) > 0, "no hardened-sand walls");
        assert!(right >= left && bottom >= top, "empty bounds");
        // The core cavity carved air inside the hive.
        let air = world
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| !t.active())
            .count();
        assert!(air > 0, "the hive carves air");
    }

    #[test]
    fn cluster_group_is_deterministic_for_a_seed() {
        let mut a = Rng(0xBEEF);
        let mut b = Rng(0xBEEF);
        let ga = ClusterGroup::new(20, 10, &mut a);
        let gb = ClusterGroup::new(20, 10, &mut b);
        assert_eq!(ga.len(), gb.len());
        for (ca, cb) in ga.clusters.iter().zip(gb.clusters.iter()) {
            assert_eq!(ca.len(), cb.len());
            for (ba, bb) in ca.iter().zip(cb.iter()) {
                assert_eq!(ba.x.to_bits(), bb.x.to_bits());
                assert_eq!(ba.y.to_bits(), bb.y.to_bits());
            }
        }
    }
}
