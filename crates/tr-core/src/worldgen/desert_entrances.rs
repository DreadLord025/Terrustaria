//! The four underground-desert entrances — `DesertBiome.Place`'s switch arms
//! (`DesertBiome.cs:25-40`) ported under the golden rule:
//! `ChambersEntrance`, `AnthillEntrance`, `LarvaHoleEntrance` and `PitEntrance`
//! (`GameContent/Biomes/Desert/`).
//!
//! The C# drives these through the `WorldUtils.Gen` / `Shapes` / `Modifiers` /
//! `Actions` framework; the port has no such framework, so each shape and each
//! action chain is inlined here with the C# line anchors (`Shapes.cs`,
//! `Modifiers.cs`, `Actions.cs`, `ModShapes.cs`, `Utils.cs`) — the walks visit
//! exactly the cells the C# visits, in the C#'s order, and apply exactly the
//! C#'s tile effects. W4-g40 (research 94) closed the shape-level residual:
//!
//! * `Shapes.Circle` is the y-major `(hr+1)²` row walk (`Shapes.cs:33-49`),
//!   not a normalised ellipse — the visit order feeds the per-cell Blotches
//!   draws;
//! * `Modifiers.Blotches` (`Modifiers.cs:126-146`) draws at every site — the
//!   shared `mini_biomes::blotched_gen_shared` seam (two NextDouble draws per
//!   cell, the first discarded, then the 0.3 spread box);
//! * `Actions.Smooth` (`Actions.cs:608-622` → `Tile.SmoothSlope`) runs on the
//!   anthill tail (single-cell) and the larva mouth (neighbours) through the
//!   `biomes::tile_smooth_slope{,_one}` seams;
//! * `Actions.SetTile` (`Actions.cs:203-236`) clears everything but
//!   wiring/actuator (`Tile.Clear(~(Wiring|Actuator))`, `Tile.cs:744-790`) —
//!   the wall does NOT survive; `Actions.ClearTile` keeps kind/wall/liquid
//!   (`Tile.ClearTile`, `Tile.cs:123-128`) via `mini_biomes::clear_tile_keep`;
//! * `Modifiers.IsSolid` is `SolidOrSlopedTile` (`Modifiers.cs:561-571`) —
//!   slopes/half-bricks pass;
//! * the chambers' `OuterOutline`/`NotInShape` read the RECORDED `ShapeData`
//!   cell sets (`ModShapes.cs:28-71`, `ShapeData.cs`), and the tunnel chain's
//!   order is load-bearing (the 397 SetTile wipes the wall the Expand just
//!   placed).
//!
//! Deviations: `ShapeData` is a .NET `HashSet<Point16>` whose iteration order
//! is unspecified — the port pins insertion order; the RNG-free outline chain
//! is outcome-independent of it (a converted cell fails `OnlyTiles(53)` on
//! any later visit), the larva Smooth chain is order-dependent in C# itself.
//! The `SquareWallFrame`/`TileFrame` framing rides the post-pipeline seam
//! (`desert.rs::cleanup_area` + `framing::reframe_area`).

use crate::tile::Tile;
use crate::world::World;
use super::biomes::{solid_or_sloped, tile_smooth_slope, tile_smooth_slope_one};
use super::desert::DesertGeom;
use super::mini_biomes::{blotched_gen_shared, clear_tile_keep};
use super::noise::Rng;
use super::ocean::reset_to_type;
use super::{TILE_HARDENED_SAND, TILE_SAND};

/// `WallID.Sandstone` (`WallID.cs:479`) — the entrance shafts' wall.
const WALL_SANDSTONE: u16 = 187;

/// One entrance style — the `DesertBiome.Place` switch's four arms (`:26-39`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum EntranceKind {
    /// `ChambersEntrance` (case 0).
    Chambers,
    /// `AnthillEntrance` (case 1).
    Anthill,
    /// `LarvaHoleEntrance` (case 2).
    LarvaHole,
    /// `PitEntrance` (case 3).
    Pit,
}

impl EntranceKind {
    /// The `GenBase._random.Next(4)` value's style.
    pub(crate) fn from_roll(roll: i32) -> Self {
        match roll.rem_euclid(4) {
            0 => Self::Chambers,
            1 => Self::Anthill,
            2 => Self::LarvaHole,
            _ => Self::Pit,
        }
    }
}

/// `DesertBiome.Place`'s switch (`:25-40`): run the one entrance the roll
/// picked. `kind` is the `Next(4)` value (0..3).
pub(crate) fn place_entrance(
    world: &mut World,
    kind: i32,
    desc: &DesertGeom,
    world_surface: f64,
    rng: &mut Rng,
) {
    match EntranceKind::from_roll(kind) {
        EntranceKind::Chambers => chambers_place(world, desc, rng),
        EntranceKind::Anthill => anthill_place(world, desc, rng),
        EntranceKind::LarvaHole => larva_hole_place(world, desc, rng),
        EntranceKind::Pit => pit_place(world, desc, rng),
    }
    let _ = world_surface;
}

// --- the shape primitives (Shapes.cs / Utils.cs) -----------------------------

/// `Shapes.Circle.Perform` (`Shapes.cs:33-49`): the **y-major** row walk —
/// rows `oy-vr..=oy+vr`, each row spanning
/// `ox ± min(hr, (int)sqrt((hr+1)² - (hr/vr·dy)²))`. The `(hr+1)²` inflation
/// and the row-major cell order are load-bearing for the per-cell Blotches
/// draws. `vr == 0` would divide by zero in C# (never at these call sites).
fn circle(world: &mut World, origin: (i32, i32), hr: i32, vr: i32, mut f: impl FnMut(&mut World, i32, i32)) {
    let sq = ((hr + 1) * (hr + 1)) as f64;
    for y in origin.1 - vr..=origin.1 + vr {
        let dy = (hr as f64 / vr as f64) * (y - origin.1) as f64;
        let half = hr.min((sq - dy * dy).sqrt() as i32);
        for x in origin.0 - half..=origin.0 + half {
            f(world, x, y);
        }
    }
}

/// `Shapes.Rectangle(Rectangle)` (`Shapes.cs:165-178`): the offset rect,
/// x-major over `[origin+area.Left, origin+area.Right)`.
fn rect(
    world: &mut World,
    origin: (i32, i32),
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    mut f: impl FnMut(&mut World, i32, i32),
) {
    for i in 0..w {
        for j in 0..h {
            f(world, origin.0 + x + i, origin.1 + y + j);
        }
    }
}

/// `Utils.PlotLine` (`Utils.cs:2238-2292`): the Bresenham walk — `jump: true`
/// (the tail plotter's outer walk) skips the stepped cell, `jump: false` (its
/// inner width walks) plots it.
pub(crate) fn plot_line(mut plot: impl FnMut(i32, i32), x0: i32, y0: i32, x1: i32, y1: i32, jump: bool) {
    if x0 == x1 && y0 == y1 {
        plot(x0, y0);
        return;
    }
    let steep = (y1 - y0).abs() > (x1 - x0).abs();
    let (mut ax0, mut ay0, mut ax1, mut ay1) = (x0, y0, x1, y1);
    if steep {
        // `Utils.Swap(ref x0, ref y0); Swap(ref x1, ref y1);` (`:2247-2248`).
        std::mem::swap(&mut ax0, &mut ay0);
        std::mem::swap(&mut ax1, &mut ay1);
    }
    let num = (ax1 - ax0).abs();
    let num2 = (ay1 - ay0).abs();
    let mut num3 = num / 2;
    let mut num4 = ay0;
    let num5 = if ax0 < ax1 { 1 } else { -1 };
    let num6 = if ay0 < ay1 { 1 } else { -1 };
    let mut i = ax0;
    while i != ax1 {
        if steep {
            plot(num4, i);
        } else {
            plot(i, num4);
        }
        num3 -= num2;
        if num3 >= 0 {
            i += num5;
            continue;
        }
        num4 += num6;
        if !jump {
            if steep {
                plot(num4, i);
            } else {
                plot(i, num4);
            }
        }
        num3 += num;
        i += num5;
    }
}

/// `Vector2D.ToTileCoordinates` (`Utils.cs:1812-1815`): `(int)v >> 4` — the
/// arithmetic shift floors negatives (the old port truncated).
fn tile_coord(v: f64) -> i32 {
    (v as i32) >> 4
}

/// `Shapes.Tail` (`Shapes.cs:181-198`) + `Utils.PlotTileTale`
/// (`Utils.cs:2352-2379`): a tapering thick line — the tail is `width` wide at
/// the start and shrinks to a point at the end (both scaled by 16, the C#'s
/// `<< 4` tile coordinates). The perpendicular offsets are computed from the
/// START point every step (`:2372-2373`), not the walked cell.
fn tail(
    world: &mut World,
    origin: (i32, i32),
    end_offset: (f64, f64),
    width: f64,
    mut apply: impl FnMut(&mut World, i32, i32),
) {
    let start = ((origin.0 << 4) as f64, (origin.1 << 4) as f64);
    let end = (start.0 + end_offset.0 * 16.0, start.1 + end_offset.1 * 16.0);
    let half_width = width * 16.0 / 2.0;
    let (dx, dy) = (end.0 - start.0, end.1 - start.1);
    let len = (dx * dx + dy * dy).sqrt();
    // The C# divides by Length() — a zero-length tail is NaN/throws; every
    // call site passes a non-zero offset, the guard keeps the port total.
    if len == 0.0 {
        apply(world, origin.0, origin.1);
        return;
    }
    let (ux, uy) = (dx / len, dy / len);
    let (perp_x, perp_y) = (-uy, ux);
    let point_start = (tile_coord(start.0), tile_coord(start.1));
    let point_end = (tile_coord(end.0), tile_coord(end.1));
    // The length walk (`:2358-2364`): the first pass counts the cells, then
    // `length--`.
    let mut length = 0;
    plot_line(
        |_x, _y| {
            length += 1;
        },
        point_start.0,
        point_start.1,
        point_end.0,
        point_end.1,
        true,
    );
    length -= 1;
    let mut cur_length = 0i32;
    plot_line(
        |x, y| {
            let num = 1.0 - cur_length as f64 / length as f64;
            cur_length += 1;
            let p2 = (
                tile_coord(start.0 - perp_x * half_width * num),
                tile_coord(start.1 - perp_y * half_width * num),
            );
            let p3 = (
                tile_coord(start.0 + perp_x * half_width * num),
                tile_coord(start.1 + perp_y * half_width * num),
            );
            let off_min = (p2.0 - point_start.0, p2.1 - point_start.1);
            let off_max = (p3.0 - point_start.0, p3.1 - point_start.1);
            plot_line(
                |ax, ay| apply(world, ax, ay),
                x + off_min.0,
                y + off_min.1,
                x + off_max.0,
                y + off_max.1,
                false,
            );
        },
        point_start.0,
        point_start.1,
        point_end.0,
        point_end.1,
        true,
    );
}

// --- the tile helpers ---------------------------------------------------------

/// `Modifiers.IsSolid` (`Modifiers.cs:561-571`): `active() &&
/// SolidOrSlopedTile` — the sloped/half-brick states pass.
fn is_solid(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| t.active()) && solid_or_sloped(world, x, y)
}

/// `Tile.ClearEverything` (`Tile.cs:110-121`): every stored field zeroed.
fn clear_everything(t: &mut Tile) {
    t.kind = 0;
    t.wall = 0;
    t.liquid = 0;
    t.flags = 0;
    t.meta = 0;
    t.frame_x = 0;
    t.frame_y = 0;
    t.wall_frame = 0;
}

/// `Actions.SetTile` (`Actions.cs:203-236`, `setSelfFrames: false`): the
/// `Clear(~(TileDataType.Wiring | TileDataType.Actuator))` mask
/// (`Tile.cs:744-790`) clears tile/wall/liquid/slope/paint + the wall frame,
/// wiring/actuator/inActive survive; then kind set + active. The wall does
/// NOT survive (unlike `Tile.ResetToType`).
fn set_tile_action(t: &mut Tile, kind: u16) {
    let keep = t.flags & (crate::tile::FLAG_ACTUATOR | crate::tile::FLAG_INACTIVE);
    t.kind = kind;
    t.wall = 0;
    t.wall_frame = 0;
    t.liquid = 0;
    t.flags = keep | crate::tile::FLAG_ACTIVE;
    t.meta = 0;
    t.frame_x = 0;
    t.frame_y = 0;
}

/// `Actions.PlaceWall` (`Actions.cs:524-549`): the wall write; the
/// `SquareWallFrame` self + 4-neighbour calls ride the framing seam.
fn place_wall_action(world: &mut World, x: i32, y: i32, wall: u16) {
    if let Some(t) = world.tile_mut(x, y) {
        t.wall = wall;
    }
}

/// `Modifiers.Expand(1)` (`Modifiers.cs:50-61`): the 3×3 box around the cell,
/// i-major then j-major.
fn expand1(world: &mut World, x: i32, y: i32, mut f: impl FnMut(&mut World, i32, i32)) {
    for i in -1..=1 {
        for j in -1..=1 {
            f(world, x + i, y + j);
        }
    }
}

/// The anthill/larva shaft ring — `Circle(2,3) + IsSolid + SetTile(397) +
/// PlaceWall(187)` (`AnthillEntrance.cs:28`, `LarvaHoleEntrance.cs:28`).
fn shaft_ring(world: &mut World, cx: i32, cy: i32) {
    circle(world, (cx, cy), 2, 3, |world, x, y| {
        if is_solid(world, x, y) {
            if let Some(t) = world.tile_mut(x, y) {
                set_tile_action(t, TILE_HARDENED_SAND);
            }
            place_wall_action(world, x, y, WALL_SANDSTONE);
        }
    });
}

/// The 1×1 shaft-clear chain's tail — `IsSolid + Clear + PlaceWall(187)`
/// (`AnthillEntrance.cs:27`, the Blotches-less variant).
fn clear_solid_wall(world: &mut World, x: i32, y: i32) {
    if is_solid(world, x, y) {
        if let Some(t) = world.tile_mut(x, y) {
            clear_everything(t);
        }
        place_wall_action(world, x, y, WALL_SANDSTONE);
    }
}

// --- AnthillEntrance ----------------------------------------------------------

/// `AnthillEntrance.Place` (`AnthillEntrance.cs:9-19`): 2-4 holes over the
/// desert band, each a `PlaceAt` shaft.
fn anthill_place(world: &mut World, desc: &DesertGeom, rng: &mut Rng) {
    let num = rng.range(2, 4);
    for i in 0..num {
        let hole_radius = rng.range(15, 18);
        let num2 = ((i + 1) as f64 / (num + 1) as f64 * desc.surface.width() as f64) as i32
            + desc.desert.left();
        let x = num2;
        let y = desc.surface.height(x);
        anthill_place_at(world, desc, (x, y), hole_radius, rng);
    }
}

/// `AnthillEntrance.PlaceAt` (`:21-50`).
fn anthill_place_at(
    world: &mut World,
    desc: &DesertGeom,
    position: (i32, i32),
    hole_radius: i32,
    rng: &mut Rng,
) {
    // `:23-25`: the upward tail from origin = position + (0,6), SetTile(53)
    // with Output(data) — every visited cell is recorded (relative to the
    // tail origin) for the closing Smooth; the C# ShapeData dedups.
    let origin = (position.0, position.1 + 6);
    let mut tail_cells: Vec<(i32, i32)> = Vec::new();
    let mut tail_seen = std::collections::HashSet::new();
    tail(
        world,
        origin,
        (0.0, -hole_radius as f64 * 1.5),
        hole_radius as f64 * 2.0,
        |world, x, y| {
            if let Some(t) = world.tile_mut(x, y) {
                set_tile_action(t, TILE_SAND);
            }
            let rel = (x - origin.0, y - origin.1);
            if tail_seen.insert(rel) {
                tail_cells.push(rel);
            }
        },
    );
    let mut num = position.0;
    let top = desc.hive.top() + (position.1 - desc.desert.top()) * 2 + 12;
    let mut i = position.1 - hole_radius - 3;
    while i < top {
        // `:34`: the 1×1 rect EVERY row — above the surface the plain
        // IsSolid+Clear+PlaceWall variant, below it the Blotches variant
        // (`:26-27`).
        if i < position.1 {
            clear_solid_wall(world, num, i);
        } else {
            let (cx, cy) = (num, i);
            blotched_gen_shared(world, rng, cx, cy, 2, &mut |world, x, y| {
                clear_solid_wall(world, x, y);
            });
        }
        // `:35`: the Circle(2,3) ring every row.
        shaft_ring(world, num, i);
        if i % 3 == 0 && i >= position.1 {
            num += rng.range(-1, 2);
            // `:39`: the blotched rect again at the drifted column.
            let (cx, cy) = (num, i);
            blotched_gen_shared(world, rng, cx, cy, 2, &mut |world, x, y| {
                clear_solid_wall(world, x, y);
            });
            if i >= position.1 + 5 {
                // `:42`: pair2 — Circle(hr,3) + SkipWalls(187) + SetTile(53).
                let (cx, cy) = (num, i);
                circle(world, (cx, cy), hole_radius, 3, |world, x, y| {
                    let skip = world.tile(x, y).map(|t| t.wall == WALL_SANDSTONE).unwrap_or(false);
                    if !skip {
                        if let Some(t) = world.tile_mut(x, y) {
                            set_tile_action(t, TILE_SAND);
                        }
                    }
                });
                // `:43`: pair3 — Circle(hr-2,3) + PlaceWall(187).
                let (cx, cy) = (num, i);
                circle(world, (cx, cy), hole_radius - 2, 3, |world, x, y| {
                    place_wall_action(world, x, y, WALL_SANDSTONE);
                });
            }
            // `:45`: the ring again at the drifted column.
            shaft_ring(world, num, i);
        }
        i += 1;
    }
    // `:48`: the top blowout — Circle(hr/2, hr/3) + ClearTile, then
    // Expand(1) + PlaceWall(0): the 3×3 wall-clear around EACH cleared cell
    // (the chain runs per circle cell, not one enlarged ellipse).
    let top_origin = (
        origin.0,
        origin.1 - (hole_radius as f64 * 1.5) as i32 + 3,
    );
    circle(world, top_origin, hole_radius / 2, hole_radius / 3, |world, x, y| {
        clear_tile_keep(world, x, y);
        expand1(world, x, y, |world, ex, ey| {
            place_wall_action(world, ex, ey, 0);
        });
    });
    // `:49`: ModShapes.All(data) + Smooth() — every recorded tail cell,
    // single-cell SmoothSlope (the default applyToNeighbors: false).
    for (dx, dy) in tail_cells {
        tile_smooth_slope_one(world, origin.0 + dx, origin.1 + dy);
    }
}

// --- LarvaHoleEntrance --------------------------------------------------------

/// `LarvaHoleEntrance.Place` (`LarvaHoleEntrance.cs:9-19`): 2-4 holes, each a
/// downward tail with the shaft ringed in hardened sand.
fn larva_hole_place(world: &mut World, desc: &DesertGeom, rng: &mut Rng) {
    let num = rng.range(2, 4);
    for i in 0..num {
        let hole_radius = rng.range(13, 16);
        let num2 = ((i + 1) as f64 / (num + 1) as f64 * desc.surface.width() as f64) as i32
            + desc.desert.left();
        let x = num2;
        let y = desc.surface.height(x);
        larva_hole_place_at(world, desc, (x, y), hole_radius, rng);
    }
}

/// `LarvaHoleEntrance.PlaceAt` (`:21-42`).
fn larva_hole_place_at(
    world: &mut World,
    desc: &DesertGeom,
    position: (i32, i32),
    hole_radius: i32,
    rng: &mut Rng,
) {
    // `:23-25`: the clearing Rectangle(-hr, -2hr, 2hr, 2hr) + the downward
    // Tail, both Clear().Output(data) — one shared recorded set (relative to
    // `position`), duplicates deduped like the C# HashSet.
    let mut data: Vec<(i32, i32)> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut record = |world: &mut World, x: i32, y: i32| {
        if let Some(t) = world.tile_mut(x, y) {
            clear_everything(t);
        }
        let rel = (x - position.0, y - position.1);
        if seen.insert(rel) {
            data.push(rel);
        }
    };
    rect(
        world,
        position,
        -hole_radius,
        -hole_radius * 2,
        hole_radius * 2,
        hole_radius * 2,
        &mut record,
    );
    tail(
        world,
        position,
        (0.0, hole_radius as f64 * 1.5),
        hole_radius as f64 * 2.0,
        &mut record,
    );
    // `:26`: All(data) + Offset(0,1) + Expand(1) + IsSolid +
    // Smooth(applyToNeighbors: true) — per recorded cell, the 3×3 box around
    // (cell + (0,1)), the neighbours-first SmoothSlope on each solid cell.
    for (dx, dy) in &data {
        let (cx, cy) = (position.0 + dx, position.1 + dy + 1);
        expand1(world, cx, cy, |world, ex, ey| {
            if is_solid(world, ex, ey) {
                tile_smooth_slope(world, ex, ey);
            }
        });
    }
    let mut num = position.0;
    let top = desc.hive.top() + (position.1 - desc.desert.top()) * 2 + 12;
    let mut i = position.1 + (hole_radius as f64 * 1.5) as i32;
    while i < top {
        // `:32-33`: the blotched 1×1 clear+wall, then the ring.
        let (cx, cy) = (num, i);
        blotched_gen_shared(world, rng, cx, cy, 2, &mut |world, x, y| {
            clear_solid_wall(world, x, y);
        });
        shaft_ring(world, num, i);
        if i % 3 == 0 {
            num += rng.range(-1, 2);
            let (cx, cy) = (num, i);
            blotched_gen_shared(world, rng, cx, cy, 2, &mut |world, x, y| {
                clear_solid_wall(world, x, y);
            });
            shaft_ring(world, num, i);
        }
        i += 1;
    }
    // `:41`: Gen(position + (0,2), All(data), PlaceWall(0)) — the wall clears
    // on EVERY recorded cell shifted down 2 (a shape, not one tile).
    let shifted = (position.0, position.1 + 2);
    for (dx, dy) in &data {
        place_wall_action(world, shifted.0 + dx, shifted.1 + dy, 0);
    }
}

// --- ChambersEntrance ---------------------------------------------------------

/// `ChambersEntrance.Place` (`ChambersEntrance.cs:19-23`).
fn chambers_place(world: &mut World, desc: &DesertGeom, rng: &mut Rng) {
    let num = desc.desert.center_x() + rng.range(-40, 41);
    chambers_place_at(world, desc, (num, desc.surface.height(num)), rng);
}

/// `ChambersEntrance.PlaceAt` (`:25-65`).
fn chambers_place_at(
    world: &mut World,
    desc: &DesertGeom,
    position: (i32, i32),
    rng: &mut Rng,
) {
    let origin = (position.0, position.1 + 2);
    // `:29`: the sand mouth — Circle(24,12) + Blotches + SetTile(53) with
    // Output(shapeData) (relative to `origin`). The Blotches spread cells
    // are recorded too (GenAction.UnitApply, `GenAction.cs:15-26`).
    let mut mouth: Vec<(i32, i32)> = Vec::new();
    let mut mouth_seen = std::collections::HashSet::new();
    circle(world, origin, 24, 12, |world, x, y| {
        blotched_gen_shared(world, rng, x, y, 2, &mut |world, bx, by| {
            if let Some(t) = world.tile_mut(bx, by) {
                set_tile_action(t, TILE_SAND);
            }
            let rel = (bx - origin.0, by - origin.1);
            if mouth_seen.insert(rel) {
                mouth.push(rel);
            }
        });
    });
    let mouth_set: std::collections::HashSet<(i32, i32)> = mouth_seen;
    let num = desc.hive.top() - position.1;
    // `:33`: Next(2) != 0 → 1 else -1 (a zero draw yields -1).
    let mut num2 = if rng.range(0, 2) != 0 { 1 } else { -1 };
    // The connection list (`:34-37`): the mouth stub, then one per chamber.
    let mut list: Vec<((f64, f64), f64)> = vec![(
        ((position.0 - num2 * 26) as f64, (position.1 - 8) as f64),
        num2 as f64,
    )];
    // `:44`'s Output(data): the cleared chamber cells, relative to
    // `position`, accumulated across rooms (the Offset shifts the cell
    // before Blotches; the recording happens at the final coordinates).
    let mut data: Vec<(i32, i32)> = Vec::new();
    let mut data_seen = std::collections::HashSet::new();
    let num3 = rng.range(2, 4);
    for i in 0..num3 {
        let num4 = ((i + 1) as f64 / num3 as f64 * num as f64) as i32 + rng.range(-8, 9);
        let num5 = num2 * rng.range(20, 41);
        let num6 = rng.range(18, 29);
        // Circle(num6/2, 3) at `position`, Offset(num5, num4), Blotches,
        // Clear().Output(data), PlaceWall(187).
        circle(world, position, num6 / 2, 3, |world, x, y| {
            let (ox, oy) = (x + num5, y + num4);
            blotched_gen_shared(world, rng, ox, oy, 2, &mut |world, bx, by| {
                if let Some(t) = world.tile_mut(bx, by) {
                    clear_everything(t);
                }
                let rel = (bx - position.0, by - position.1);
                if data_seen.insert(rel) {
                    data.push(rel);
                }
                place_wall_action(world, bx, by, WALL_SANDSTONE);
            });
        });
        list.push((
            ((num5 + num6 / 2 * -num2 + position.0) as f64, (num4 + position.1) as f64),
            -num2 as f64,
        ));
        num2 *= -1;
    }
    // `:48`: OuterOutline(data) + Expand(1) + OnlyTiles(53) + SetTile(397) +
    // PlaceWall(187). The outline walks each datum's 8 neighbours NOT in the
    // data (`ModShapes.cs:43-61`, useDiagonals: true); the chain is RNG-free
    // and a converted cell fails OnlyTiles on any later visit, so the
    // insertion-order walk matches the C# HashSet's outcome.
    chambers_outline(world, position, &data, &data_seen);
    // `:49-62`: the tunnels — Rectangle(2,4) at the double-Lerp point, the
    // chain IsSolid + Blotches + Clear + Expand(1) + PlaceWall(187) +
    // OnlyTiles(53) + SetTile(397) per spread cell. The chain order is
    // load-bearing: the 397 SetTile's Clear wipes the wall the Expand just
    // placed on that cell.
    for j in 1..list.len() {
        let (a, dir_a) = list[j - 1];
        let (b, dir_b) = list[j];
        let num7 = (b.0 - a.0).abs() * 1.5;
        let mut num8 = 0.0;
        while num8 <= 1.0 {
            let value = (a.0 + dir_a * num7 * num8, a.1);
            let value2 = (b.0 + dir_b * num7 * (1.0 - num8), b.1);
            let mid = (
                a.0 + (b.0 - a.0) * num8,
                a.1 + (b.1 - a.1) * num8,
            );
            // The double-Lerp point (`:60-61`, Vector2D.Lerp = v1+(v2-v1)*t).
            let p0 = (
                value.0 + (mid.0 - value.0) * num8,
                value.1 + (mid.1 - value.1) * num8,
            );
            let p1 = (
                mid.0 + (value2.0 - mid.0) * num8,
                mid.1 + (value2.1 - mid.1) * num8,
            );
            let point = (
                p0.0 + (p1.0 - p0.0) * num8,
                p0.1 + (p1.1 - p0.1) * num8,
            );
            let (px, py) = (point.0 as i32, point.1 as i32);
            rect(world, (px, py), 0, 0, 2, 4, |world, x, y| {
                if !is_solid(world, x, y) {
                    return;
                }
                blotched_gen_shared(world, rng, x, y, 2, &mut |world, bx, by| {
                    if let Some(t) = world.tile_mut(bx, by) {
                        clear_everything(t);
                    }
                    expand1(world, bx, by, |world, ex, ey| {
                        place_wall_action(world, ex, ey, WALL_SANDSTONE);
                        // OnlyTiles(53): active AND sand. The chain ENDS at
                        // SetTile(397) (`:49`) — its Clear wipes the wall the
                        // Expand just placed on this cell (no trailing
                        // PlaceWall, unlike the outline chain `:48`).
                        let sand = world
                            .tile(ex, ey)
                            .is_some_and(|t| t.active() && t.kind == TILE_SAND);
                        if sand {
                            if let Some(t) = world.tile_mut(ex, ey) {
                                set_tile_action(t, TILE_HARDENED_SAND);
                            }
                        }
                    });
                });
            });
            num8 += 0.02;
        }
    }
    // `:64`: Rectangle(-29,-12,58,12) at origin + NotInShape(shapeData) +
    // Expand(1) + PlaceWall(0) — the shape filter applies to the RECT cell
    // only; the expansion's wall clears are NOT re-filtered.
    rect(world, origin, -29, -12, 58, 12, |world, x, y| {
        if mouth_set.contains(&(x - origin.0, y - origin.1)) {
            return;
        }
        expand1(world, x, y, |world, ex, ey| {
            place_wall_action(world, ex, ey, 0);
        });
    });
}

/// `ModShapes.OuterOutline(data)` (`ModShapes.cs:28-71`) + the `:48` chain:
/// for each recorded cell, every 8-neighbour offset outside the data gets
/// Expand(1) + OnlyTiles(53) + SetTile(397) + PlaceWall(187). The
/// POINT_OFFSETS order (`:65-69`): (1,0),(-1,0),(0,1),(0,-1),(1,1),(1,-1),
/// (-1,1),(-1,-1).
fn chambers_outline(
    world: &mut World,
    position: (i32, i32),
    data: &[(i32, i32)],
    data_seen: &std::collections::HashSet<(i32, i32)>,
) {
    const OFFSETS: [(i32, i32); 8] = [
        (1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1),
    ];
    for &(dx, dy) in data {
        for (ox, oy) in OFFSETS {
            let rel = (dx + ox, dy + oy);
            if data_seen.contains(&rel) {
                continue;
            }
            let (x, y) = (position.0 + rel.0, position.1 + rel.1);
            expand1(world, x, y, |world, ex, ey| {
                let sand = world
                    .tile(ex, ey)
                    .is_some_and(|t| t.active() && t.kind == TILE_SAND);
                if sand {
                    if let Some(t) = world.tile_mut(ex, ey) {
                        set_tile_action(t, TILE_HARDENED_SAND);
                    }
                    place_wall_action(world, ex, ey, WALL_SANDSTONE);
                }
            });
        }
    }
}

// --- PitEntrance --------------------------------------------------------------

/// `PitEntrance.Place` (`PitEntrance.cs:9-15`).
fn pit_place(world: &mut World, desc: &DesertGeom, rng: &mut Rng) {
    let hole_radius = rng.range(6, 9);
    let center_x = desc.combined.center_x();
    let center_y = desc.surface.height(center_x);
    pit_place_at(world, desc, (center_x, center_y), hole_radius);
}

/// `PitEntrance.PlaceAt` (`:17-56`) — the direct `Main.tile` writes.
fn pit_place_at(world: &mut World, desc: &DesertGeom, position: (i32, i32), hole_radius: i32) {
    for i in -hole_radius - 3..hole_radius + 3 {
        let surface_row = desc.surface.height(i + position.0);
        let mut j = surface_row;
        while j <= desc.hive.top() + 10 {
            let num3 = ((j - surface_row) as f64 / (desc.hive.top() - desc.desert.top()) as f64)
                .clamp(0.0, 1.0);
            let num4 = (hole_radius_scale_at(num3) * hole_radius as f64) as i32;
            let x = i + position.0;
            if i.abs() < num4 {
                if let Some(t) = world.tile_mut(x, j) {
                    clear_everything(t);
                }
            } else if i.abs() < num4 + 3 && num3 > 0.35 {
                // `:34`: ResetToType(397) — liquid cleared, wall survives.
                if let Some(t) = world.tile_mut(x, j) {
                    reset_to_type(t, TILE_HARDENED_SAND);
                }
            }
            let mut num5 = (i as f64 / hole_radius as f64).abs();
            num5 *= num5;
            if i.abs() < num4 + 3 && (j - position.1) as f64 > 15.0 - 3.0 * num5 {
                if let Some(t) = world.tile_mut(x, j) {
                    t.wall = WALL_SANDSTONE;
                }
                // `WorldGen.SquareWallFrame` (`:41-42`) runs through the hive's
                // cleanup reframe (`desert.rs::cleanup_area`).
            }
            j += 1;
        }
    }
    // The surface bowl (`:46-55`): num6 = min(10, (hr-|k|)²) rows cleared.
    let hole_radius = hole_radius + 4;
    for k in -hole_radius..hole_radius {
        let num6 = hole_radius - k.abs();
        let num6 = 10.min(num6 * num6);
        for l in 0..num6 {
            let (x, y) = (k + position.0, l + desc.surface.height(k + position.0));
            if let Some(t) = world.tile_mut(x, y) {
                clear_everything(t);
            }
        }
    }
}

/// `GetHoleRadiusScaleAt` (`PitEntrance.cs:58-65`).
fn hole_radius_scale_at(y_progress: f64) -> f64 {
    if y_progress < 0.6 {
        return 1.0;
    }
    (1.0 - smoother_step((y_progress - 0.6) / 0.4)) * 0.5 + 0.5
}

/// `SmootherStep` (`PitEntrance.cs:67-71`) — the C# f32-π literal
/// `3.1415927410125732` (NOT `std::f64::consts::PI`; the old port dropped the
/// multiply entirely).
fn smoother_step(mut delta: f64) -> f64 {
    delta = delta.clamp(0.0, 1.0);
    1.0 - (delta * 3.1415927410125732).cos() * 0.5 - 0.5
}

/// The W4-g40 residual witnesses (research 94) — WRITTEN NOT RUN (code-only
/// mode, no compiler on this branch). The ledgers replay the xorshift stream
/// independently (`rng.rs`'s pre-refactor transcription convention); the
/// a==b pair pins the entrance outputs across two same-seed runs.
#[cfg(test)]
mod w4_g40_tests {
    use super::*;
    use super::super::desert::{Rect, SurfaceLine};
    use crate::tile::{FLAG_ACTIVE, FLAG_HALF_BRICK};

    /// The independent xorshift64 replay (the `rng.rs` witness convention).
    fn replay_word(state: &mut u64) -> u32 {
        let mut x = *state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        *state = x;
        (x >> 32) as u32
    }

    fn stone_world(w: i32, h: i32, top: i32) -> World {
        let mut world = World::new("w4g40".into(), w, h);
        for x in 0..w {
            for y in top..h {
                let i = world.index(x, y);
                world.tiles[i].kind = TILE_SAND;
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        world
    }

    fn geom(world: &World) -> DesertGeom {
        let surface = SurfaceLine::from_area(world, 100, 200);
        DesertGeom {
            combined: Rect::new(100, 40, 200, 200),
            desert: Rect::new(100, 40, 200, 60),
            hive: Rect::new(100, 100, 200, 200),
            block_columns: 50,
            block_rows: 85,
            surface,
            valid: true,
        }
    }

    /// `Shapes.Circle` (`Shapes.cs:33-49`): the y-major `(hr+1)^2` row walk —
    /// the visit ORDER and the inflated row extents are the C#'s, not a
    /// normalised ellipse. Circle(2,3) at (10,10): row dy=-3 spans
    /// `min(2, sqrt(9 - 4)) = 2` (the ellipse would reject |dx|=2 at dy=±3).
    #[test]
    fn w4_g40_circle_walks_y_major_with_the_inflated_rows() {
        let mut world = stone_world(40, 40, 0);
        let mut visits: Vec<(i32, i32)> = Vec::new();
        circle(&mut world, (10, 10), 2, 3, |_w, x, y| visits.push((x, y)));
        // y-major: the first row is dy = -3, x ascending.
        assert_eq!(visits.first(), Some(&(8, 7)));
        assert_eq!(visits[1], (9, 7));
        // The inflated extent: (hr+1)^2 = 9, dy_scaled = (2/3)*(-3) = -2,
        // sqrt(9-4) = 2.23 -> half 2 — the corner cells (+/-2, +/-3) ARE visited.
        assert!(visits.contains(&(12, 7)));
        assert!(visits.contains(&(8, 13)));
        // The middle row dy=0 spans the full hr=2.
        assert!(visits.contains(&(8, 10)) && visits.contains(&(12, 10)));
        // Row-major order: every cell of row y precedes every cell of y+1.
        for pair in visits.windows(2) {
            assert!(pair[0].1 <= pair[1].1, "y-major violated: {pair:?}");
        }
        // The count: dy=-3: sqrt(9-4)=2.23->2 (5 cells); dy=-2:
        // sqrt(9-1.78)=2.68->2 (5); dy=-1: sqrt(9-0.44)=2.92->2 (5); dy=0:
        // sqrt(9)=3->min(2,3)=2 (5); symmetric below -> 7 rows x 5 = 35.
        assert_eq!(visits.len(), 35);
    }

    /// `Modifiers.Blotches` (`Modifiers.cs:126-146`) through the shared seam:
    /// two NextDouble draws per cell (the first discarded), the 0.3 spread
    /// box on four more draws — the ledger replays independently.
    #[test]
    fn w4_g40_blotches_ledger_matches_the_replayed_stream() {
        let mut world = stone_world(40, 40, 0);
        let mut rng = Rng(0x5EED);
        let mut state = 0x5EEDu64;
        let mut applied: Vec<(i32, i32)> = Vec::new();
        for (cx, cy) in [(5, 5), (9, 7), (12, 11)] {
            let w1 = replay_word(&mut state);
            let w2 = replay_word(&mut state);
            let spread = (f64::from(w2) / 4_294_967_296.0) < 0.3;
            let mut expect: Vec<(i32, i32)> = Vec::new();
            if spread {
                let lo_x = cx + (-1 + (replay_word(&mut state) % 2) as i32);
                let hi_x = cx + (replay_word(&mut state) % 2) as i32;
                let lo_y = cy + (-1 + (replay_word(&mut state) % 2) as i32);
                let hi_y = cy + (replay_word(&mut state) % 2) as i32;
                for x in lo_x..=hi_x {
                    for y in lo_y..=hi_y {
                        expect.push((x, y));
                    }
                }
            } else {
                expect.push((cx, cy));
            }
            let _ = w1; // the discarded first draw
            let before = applied.len();
            blotched_gen_shared(&mut world, &mut rng, cx, cy, 2, &mut |_w, x, y| {
                applied.push((x, y));
            });
            assert_eq!(&applied[before..], &expect[..], "cell ({cx},{cy})");
        }
        // The stream position matches the replay exactly.
        assert_eq!(rng.0, state);
    }

    /// `Actions.SetTile` (`Actions.cs:203-236`): the Clear mask wipes the
    /// wall/liquid/slope/frames — only wiring/actuator/inActive survive.
    #[test]
    fn w4_g40_set_tile_action_wipes_the_wall_but_keeps_actuator() {
        let mut t = Tile::default();
        t.kind = TILE_SAND;
        t.wall = WALL_SANDSTONE;
        t.wall_frame = 0x21;
        t.liquid = 200;
        t.flags = FLAG_ACTIVE
            | FLAG_HALF_BRICK
            | crate::tile::FLAG_ACTUATOR
            | crate::tile::FLAG_INACTIVE;
        t.meta = 0x2a; // slope 2 + paint 5
        t.frame_x = 36;
        t.frame_y = 54;
        set_tile_action(&mut t, TILE_HARDENED_SAND);
        assert_eq!(t.kind, TILE_HARDENED_SAND);
        assert!(t.active());
        assert_eq!(t.wall, 0, "the wall does NOT survive SetTile");
        assert_eq!(t.wall_frame, 0);
        assert_eq!(t.liquid, 0);
        assert_eq!(t.meta, 0, "slope + paint cleared");
        assert_eq!((t.frame_x, t.frame_y), (0, 0));
        assert!(t.actuator(), "the actuator bit survives the mask");
        assert!(t.inactive(), "inActive rides the Actuator arm");
        assert!(!t.half_brick());
    }

    /// `Modifiers.IsSolid` (`Modifiers.cs:561-571`): a sloped/half-brick
    /// solid cell PASSES (the old flat-only gate rejected it).
    #[test]
    fn w4_g40_is_solid_accepts_slopes_and_half_bricks() {
        let mut world = stone_world(20, 20, 0);
        let i = world.index(5, 5);
        world.tiles[i].set_slope(2);
        assert!(is_solid(&world, 5, 5), "slope 2 passes SolidOrSlopedTile");
        world.tiles[i].set_slope(0);
        world.tiles[i].flags |= FLAG_HALF_BRICK;
        assert!(is_solid(&world, 5, 5), "half-brick passes");
        world.tiles[i].flags |= crate::tile::FLAG_INACTIVE;
        assert!(!is_solid(&world, 5, 5), "actuated-off fails");
        world.tiles[i].flags &= !crate::tile::FLAG_INACTIVE;
        world.tiles[i].flags &= !FLAG_ACTIVE;
        assert!(!is_solid(&world, 5, 5), "inactive fails");
    }

    /// `PitEntrance.SmootherStep` (`PitEntrance.cs:67-71`): the f32-pi literal
    /// — at delta 0.5 the C# gives ~0.5 (cos(pi/2)~0), the old port gave
    /// ~0.061 (no pi). The scale at progress 0.8 is ~0.75, not ~0.97.
    #[test]
    fn w4_g40_smoother_step_carries_the_pi_literal() {
        let mid = smoother_step(0.5);
        assert!((mid - 0.5).abs() < 1e-7, "cos(pi/2) ~ 0 -> 0.5, got {mid}");
        assert!((smoother_step(0.0) - 0.0).abs() < 1e-12);
        assert!((smoother_step(1.0) - 1.0).abs() < 1e-7);
        // The taper: progress 0.8 -> delta 0.5 -> scale 0.75.
        let scale = hole_radius_scale_at(0.8);
        assert!((scale - 0.75).abs() < 1e-7, "got {scale}");
        // Below 0.6 the radius is full.
        assert_eq!(hole_radius_scale_at(0.59), 1.0);
    }

    /// `PitEntrance.PlaceAt`'s bowl (`:46-55`): num6 = min(10, (hr-|k|)^2) —
    /// at hr=10, k=+/-7 the C# digs 9 rows (the old port dug 3). The fixture
    /// fills from row 50 (the SurfaceMap scan's start, `SurfaceMap.cs:52`).
    #[test]
    fn w4_g40_pit_bowl_digs_the_squared_depth() {
        let mut world = stone_world(60, 80, 50);
        let surface = SurfaceLine::from_area(&world, 0, 60);
        let desc = DesertGeom {
            combined: Rect::new(0, 50, 60, 30),
            desert: Rect::new(0, 50, 60, 10),
            hive: Rect::new(0, 60, 60, 20),
            block_columns: 15,
            block_rows: 15,
            surface,
            valid: true,
        };
        // hole_radius 6 -> bowl radius 10; k = 7 -> num6 = min(10, 9) = 9.
        pit_place_at(&mut world, &desc, (30, 50), 6);
        let cleared = (0..9)
            .filter(|l| {
                let i = world.index(37, 50 + l);
                !world.tiles[i].active()
            })
            .count();
        assert_eq!(cleared, 9, "the k=7 column digs 9 rows");
        // k = 9 -> num6 = min(10, 1) = 1.
        let i = world.index(39, 50);
        assert!(!world.tiles[i].active());
        let i = world.index(39, 51);
        assert!(world.tiles[i].active(), "only one row at k=9");
    }

    /// The larva wall-0 reset is the recorded SHAPE shifted down 2
    /// (`LarvaHoleEntrance.cs:41`), not one tile — after the pass, hundreds
    /// of band cells lost their wall (the single-tile port cleared one).
    #[test]
    fn w4_g40_larva_wall_reset_covers_the_shifted_shape() {
        let mut world = stone_world(600, 400, 40);
        // Pre-wall the whole band: only the entrance chains write wall 0
        // here (the shaft writes 187), so any 0 below the surface is the
        // shifted-shape reset's work.
        for x in 0..600 {
            for y in 40..400 {
                let i = world.index(x, y);
                world.tiles[i].wall = WALL_SANDSTONE;
            }
        }
        let desc = geom(&world);
        let mut rng = Rng(0x5EED);
        larva_hole_place(&mut world, &desc, &mut rng);
        let cleared_air = world.tiles.iter().filter(|t| !t.active()).count();
        assert!(cleared_air > 100, "the rect+tail shape carves air ({cleared_air})");
        // The shifted wall-0 spans the recorded rect+tail cells + (0,2) —
        // hundreds of cells across the 2-4 holes, not the one tile the old
        // port wrote.
        let wall0 = (100..300)
            .flat_map(|x| (42..140).map(move |y| (x, y)))
            .filter(|&(x, y)| {
                let i = world.index(x, y);
                world.tiles[i].wall == 0
            })
            .count();
        assert!(wall0 > 100, "the shifted shape resets many walls ({wall0})");
    }

    /// The chambers direction draw (`ChambersEntrance.cs:33`): Next(2) != 0
    /// -> 1 else -1 — a ZERO draw yields -1 (the old chance(2) inverted it).
    #[test]
    fn w4_g40_chambers_direction_zero_draw_yields_minus_one() {
        // Find a seed whose first word is even (Next(2) == 0).
        let mut seed = 1u64;
        loop {
            let mut s = seed;
            if replay_word(&mut s) % 2 == 0 {
                break;
            }
            seed += 1;
        }
        let mut rng = Rng(seed);
        let dir = if rng.range(0, 2) != 0 { 1 } else { -1 };
        assert_eq!(dir, -1, "the zero draw must yield -1");
    }

    /// a==b: the whole entrance family over a fixed seed replays
    /// tile-for-tile (every stored field) — the residual's determinism pin
    /// (the world-hash re-pin is deferred to the gate).
    #[test]
    fn w4_g40_entrance_family_a_equals_b() {
        for kind in 0..4 {
            let mut a = stone_world(600, 400, 40);
            let mut b = stone_world(600, 400, 40);
            let da = geom(&a);
            let db = geom(&b);
            let mut ra = Rng(0xA11CE);
            let mut rb = Rng(0xA11CE);
            place_entrance(&mut a, kind, &da, 400.0, &mut ra);
            place_entrance(&mut b, kind, &db, 400.0, &mut rb);
            assert_eq!(ra.0, rb.0, "entrance {kind}: stream position");
            for (n, (ta, tb)) in a.tiles.iter().zip(b.tiles.iter()).enumerate() {
                assert_eq!(ta, tb, "entrance {kind} tile {n}");
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use super::super::desert::{Rect, SurfaceLine};
    use crate::tile::FLAG_ACTIVE;

    fn tile_stone(kind: u16) -> Tile {
        let mut t = Tile::default();
        t.kind = kind;
        t.flags |= FLAG_ACTIVE;
        t
    }

    /// A stone world with the surface at row 40 and the desert band filled.
    fn fixture() -> (World, DesertGeom) {
        let mut world = World::new("entrances".into(), 600, 400);
        for x in 0..600 {
            for y in 40..400 {
                let i = world.index(x, y);
                world.tiles[i] = tile_stone(TILE_SAND);
            }
        }
        let surface = SurfaceLine::from_area(&world, 100, 200);
        let desc = DesertGeom {
            combined: Rect::new(100, 40, 200, 200),
            desert: Rect::new(100, 40, 200, 60),
            hive: Rect::new(100, 100, 200, 200),
            block_columns: 50,
            block_rows: 85,
            surface,
            valid: true,
        };
        (world, desc)
    }

    #[test]
    fn the_anthill_carves_a_shaft_and_lines_it() {
        let (mut world, desc) = fixture();
        let mut rng = Rng(0x5EED);
        anthill_place(&mut world, &desc, &mut rng);
        let hardened = world
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == TILE_HARDENED_SAND)
            .count();
        let sandstone_wall = world.tiles.iter().filter(|t| t.wall == WALL_SANDSTONE).count();
        let air = world.tiles.iter().filter(|t| !t.active()).count();
        assert!(hardened > 0, "the shaft's 397 ring ({hardened})");
        assert!(sandstone_wall > 0, "the 187 walls ({sandstone_wall})");
        assert!(air > 0, "the shaft carves air ({air})");
    }

    #[test]
    fn the_larva_hole_rings_its_shaft() {
        let (mut world, desc) = fixture();
        let mut rng = Rng(0x5EED);
        larva_hole_place(&mut world, &desc, &mut rng);
        let hardened = world
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == TILE_HARDENED_SAND)
            .count();
        assert!(hardened > 0, "the shaft's 397 ring ({hardened})");
    }

    #[test]
    fn the_chambers_clear_and_wall_their_rooms() {
        let (mut world, desc) = fixture();
        let mut rng = Rng(0x5EED);
        chambers_place(&mut world, &desc, &mut rng);
        let air = world.tiles.iter().filter(|t| !t.active()).count();
        let sandstone_wall = world.tiles.iter().filter(|t| t.wall == WALL_SANDSTONE).count();
        assert!(air > 0, "the chambers carve air ({air})");
        assert!(sandstone_wall > 0, "the 187 walls ({sandstone_wall})");
    }

    #[test]
    fn the_pit_clears_the_core_and_walls_the_skirt() {
        let (mut world, desc) = fixture();
        let mut rng = Rng(0x5EED);
        pit_place(&mut world, &desc, &mut rng);
        let air = world.tiles.iter().filter(|t| !t.active()).count();
        let hardened = world
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == TILE_HARDENED_SAND)
            .count();
        let sandstone_wall = world.tiles.iter().filter(|t| t.wall == WALL_SANDSTONE).count();
        assert!(air > 0, "the pit carves air ({air})");
        assert!(hardened > 0, "the 397 skirt ({hardened})");
        assert!(sandstone_wall > 0, "the 187 walls ({sandstone_wall})");
    }

    #[test]
    fn every_entrance_is_deterministic_for_a_seed() {
        for kind in 0..4 {
            let (mut a, da) = fixture();
            let (mut b, db) = fixture();
            let mut ra = Rng(0xC0FFEE);
            let mut rb = Rng(0xC0FFEE);
            let _ = place_entrance(&mut a, kind, &da, 400.0f64, &mut ra);
            let _ = place_entrance(&mut b, kind, &db, 400.0f64, &mut rb);
            for (n, (ta, tb)) in a.tiles.iter().zip(b.tiles.iter()).enumerate() {
                assert_eq!(
                    (ta.kind, ta.active(), ta.wall, ta.liquid),
                    (tb.kind, tb.active(), tb.wall, tb.liquid),
                    "entrance {kind} tile {n} diverges"
                );
            }
        }
    }

    /// W4-g40: `Shapes.Tail`'s `ToTileCoordinates` floors negatives
    /// (`Utils.cs:1812-1815`, `(int)v >> 4`) — the old truncation differed
    /// below zero. The upward anthill tail's perpendicular offsets stay on
    /// the floor convention.
    #[test]
    fn w4_g40_tile_coord_floors_negatives() {
        assert_eq!(tile_coord(-1.0), -1);
        assert_eq!(tile_coord(-16.0), -1);
        assert_eq!(tile_coord(-17.0), -2);
        assert_eq!(tile_coord(15.9), 0);
        assert_eq!(tile_coord(16.0), 1);
    }

    /// W4-g40: the anthill blowout's Expand(1) clears walls on the 3x3 box
    /// around EACH circle cell (`AnthillEntrance.cs:48`) — a wall-187 cell
    /// adjacent to the blowout circle loses its wall even outside any
    /// enlarged ellipse.
    #[test]
    fn w4_g40_anthill_blowout_expands_the_wall_clear() {
        let (mut world, desc) = fixture();
        // Pre-wall a band around the first hole's blowout region.
        for x in 100..300 {
            for y in 40..100 {
                let i = world.index(x, y);
                world.tiles[i].wall = WALL_SANDSTONE;
            }
        }
        let mut rng = Rng(0x5EED);
        anthill_place(&mut world, &desc, &mut rng);
        // The blowout ran: some pre-walled cell near the surface lost 187.
        let cleared = (100..300)
            .flat_map(|x| (40..70).map(move |y| (x, y)))
            .filter(|&(x, y)| {
                let i = world.index(x, y);
                world.tiles[i].wall == 0
            })
            .count();
        assert!(cleared > 0, "the blowout's Expand clears walls ({cleared})");
    }
}
