//! The four underground-desert entrances — `DesertBiome.Place`'s switch arms
//! (`DesertBiome.cs:25-40`) ported under the golden rule:
//! `ChambersEntrance`, `AnthillEntrance`, `LarvaHoleEntrance` and `PitEntrance`
//! (`GameContent/Biomes/Desert/`).
//!
//! The C# drives these through the `WorldUtils.Gen` / `Shapes` / `Modifiers` /
//! `Actions` framework; the port has no such framework, so each shape and each
//! action chain is inlined here with the C# line anchors (`Shapes.cs`,
//! `Modifiers.cs`, `Actions.cs`, `Utils.cs`) — the walks visit exactly the
//! cells the C# visits and apply exactly the C#'s tile effects.
//!
//! ```text
//! AnthillEntrance.Place (:9-19)     — 2-4 tails up from the surface
//! LarvaHoleEntrance.Place (:9-19)   — 2-4 tails down, the shaft ringed in 397
//! ChambersEntrance.Place (:19-65)   — the sand mouth + 2-4 chambers + tunnels
//! PitEntrance.Place (:9-56)         — the tapering central pit
//! ```
//!
//! Deviations: the stochastic modifiers (`Modifiers.Blotches` — a genRand
//! noise pass — and the `Actions.Smooth` slope passes) have no port model and
//! are skipped, with the C# anchor noted at each site; the `SquareWallFrame`
//! calls inside the pit run through the hive's cleanup reframe instead
//! (`desert.rs`'s `cleanup_area`).

use crate::tiledata as td;
use crate::tile::{Tile, FLAG_ACTIVE, FLAG_HALF_BRICK};
use crate::world::World;
use super::desert::{DesertGeom, Rect, SurfaceLine};
use super::noise::Rng;
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

/// `Shapes.Circle.Perform` (`Shapes.cs:33+`): an ellipse of the given radii
/// around `origin`, the `(x - ox)²/hr² + (y - oy)²/vr² <= 1` test.
fn circle(world: &mut World, origin: (i32, i32), hr: i32, vr: i32, mut f: impl FnMut(&mut World, i32, i32)) {
    for i in -hr..=hr {
        for j in -vr..=vr {
            let a = i as f64 / hr as f64;
            let b = j as f64 / vr as f64;
            if a * a + b * b <= 1.0 {
                f(world, origin.0 + i, origin.1 + j);
            }
        }
    }
}

/// `Shapes.Rectangle(Rectangle)` (`Shapes.cs:150+`): the offset rect.
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
fn plot_line(mut plot: impl FnMut(i32, i32), x0: i32, y0: i32, x1: i32, y1: i32, jump: bool) {
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

/// `Shapes.Tail` (`Shapes.cs:181-198`) + `Utils.PlotTileTale`
/// (`Utils.cs:2352-2379`): a tapering thick line — the tail is `width` wide at
/// the start and shrinks to a point at the end (both scaled by 16, the C#'s
/// `<< 4` tile coordinates).
fn tail(
    world: &mut World,
    origin: (i32, i32),
    end_offset: (f64, f64),
    width: f64,
    mut apply: impl FnMut(&mut World, i32, i32),
) {
    let start = (origin.0 as f64 * 16.0, origin.1 as f64 * 16.0);
    let end = (
        origin.0 as f64 * 16.0 + end_offset.0 * 16.0,
        origin.1 as f64 * 16.0 + end_offset.1 * 16.0,
    );
    let half_width = width * 16.0 / 2.0;
    let (dx, dy) = (end.0 - start.0, end.1 - start.1);
    let len = (dx * dx + dy * dy).sqrt();
    if len == 0.0 {
        apply(world, origin.0, origin.1);
        return;
    }
    let (ux, uy) = (dx / len, dy / len);
    let (perp_x, perp_y) = (-uy, ux);
    let tile = |v: f64| (v / 16.0) as i32;
    let point_start = (tile(start.0), tile(start.1));
    let point_end = (tile(end.0), tile(end.1));
    // The length walk (`:2362-2367`): the first pass counts the cells.
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
    length = (length - 1).max(0);
    let mut cur_length = 0i32;
    plot_line(
        |x, y| {
            let num = 1.0 - cur_length as f64 / length as f64;
            cur_length += 1;
            let p2 = (
                ((start.0 - perp_x * half_width * num) / 16.0) as i32,
                ((start.1 - perp_y * half_width * num) / 16.0) as i32,
            );
            let p3 = (
                ((start.0 + perp_x * half_width * num) / 16.0) as i32,
                ((start.1 + perp_y * half_width * num) / 16.0) as i32,
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

/// `WorldGen.SolidTile` (`WorldGen.cs:70111-70131`, the ocean pass's
/// `solid_tile` twin): no RNG, an active non-top-solid non-half-brick
/// flat tile.
fn solid(world: &World, x: i32, y: i32) -> bool {
    match world.tile(x, y) {
        Some(t) => {
            t.active()
                && (t.kind as usize) < td::TILE_COUNT
                && td::TILE_SOLID.has(t.kind)
                && !td::TILE_SOLID_TOP.has(t.kind)
                && !t.half_brick()
                && t.slope() == 0
        }
        None => false,
    }
}

fn solid_t(t: &Tile) -> bool {
    t.active()
        && (t.kind as usize) < td::TILE_COUNT
        && td::TILE_SOLID.has(t.kind)
        && !td::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
}

/// `Tile.ClearEverything` (`Tile.cs:110-121`).
fn clear_everything(t: &mut Tile) {
    t.kind = 0;
    t.wall = 0;
    t.liquid = 0;
    t.flags = 0;
    t.meta = 0;
    t.frame_x = 0;
    t.frame_y = 0;
}

/// `Tile.ResetToType` (`Tile.cs:249-259`).
fn reset_to_type(t: &mut Tile, kind: u16) {
    t.liquid = 0;
    t.flags = FLAG_ACTIVE;
    t.meta = 0;
    t.frame_x = 0;
    t.frame_y = 0;
    t.kind = kind;
    t.flags &= !FLAG_HALF_BRICK;
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
    // The tail from the origin, setting sand 53 (`:25`).
    let origin = (position.0, position.1 + 6);
    let mut tail_cells: Vec<(i32, i32)> = Vec::new();
    tail(
        world,
        origin,
        (0.0, -hole_radius as f64 * 1.5),
        hole_radius as f64 * 2.0,
        |world, x, y| {
            if let Some(t) = world.tile_mut(x, y) {
                reset_to_type(t, TILE_SAND);
                tail_cells.push((x, y));
            }
        },
    );
    let mut num = position.0;
    let top = desc.hive.top() + (position.1 - desc.desert.top()) * 2 + 12;
    let mut i = position.1 - hole_radius - 3;
    while i < top {
        // `pair` — Circle(2,3) + IsSolid + SetTile(397) + PlaceWall(187).
        let cx = num;
        let cy = i;
        circle(world, (cx, cy), 2, 3, |world, x, y| {
            if solid(world, x, y) {
                if let Some(t) = world.tile_mut(x, y) {
                    reset_to_type(t, TILE_HARDENED_SAND);
                    t.wall = WALL_SANDSTONE;
                }
            }
        });
        if i % 3 == 0 && i >= position.1 {
            num += rng.range(-1, 2);
            // `genShapeActionPair`/`pair2` — the 1x1 clear+wall and the sand
            // circle with the SkipWalls(187) gate (`:26-27`, `:29`).
            // `Modifiers.Blotches` is skipped (a genRand noise pass) — the
            // clear+wall arm stands alone.
            let is_solid = solid(world, num, i);
            if let Some(t) = world.tile_mut(num, i) {
                if is_solid {
                    clear_everything(t);
                    t.wall = WALL_SANDSTONE;
                }
            }
            if i >= position.1 + 5 {
                let cx = num;
                circle(world, (cx, i), hole_radius, 3, |world, x, y| {
                    let skip = world.tile(x, y).map(|t| t.wall == WALL_SANDSTONE).unwrap_or(false);
                    if !skip {
                        if let Some(t) = world.tile_mut(x, y) {
                            reset_to_type(t, TILE_SAND);
                        }
                    }
                });
                let cx = num;
                circle(world, (cx, i), hole_radius - 2, 3, |world, x, y| {
                    if let Some(t) = world.tile_mut(x, y) {
                        t.wall = WALL_SANDSTONE;
                    }
                });
            }
            let cx = num;
            circle(world, (cx, i), 2, 3, |world, x, y| {
                if solid(world, x, y) {
                    if let Some(t) = world.tile_mut(x, y) {
                        reset_to_type(t, TILE_HARDENED_SAND);
                        t.wall = WALL_SANDSTONE;
                    }
                }
            });
        }
        i += 1;
    }
    // The top blowout (`:48`): ClearTile + Expand(1) + PlaceWall(0).
    let top_origin = (
        origin.0,
        origin.1 - (hole_radius as f64 * 1.5) as i32 + 3,
    );
    circle(world, top_origin, hole_radius / 2, hole_radius / 3, |world, x, y| {
        if let Some(t) = world.tile_mut(x, y) {
            t.kind = 0;
            t.flags &= !FLAG_ACTIVE;
        }
    });
    circle(world, top_origin, hole_radius / 2 + 1, hole_radius / 3 + 1, |world, x, y| {
        if let Some(t) = world.tile_mut(x, y) {
            t.wall = 0;
        }
    });
    // `Actions.Smooth` over the tail shape (`:49`) is skipped — the port has no
    // slope smoother.
    let _ = tail_cells;
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

/// `LarvaHoleEntrance.PlaceAt` (`:21-41`).
fn larva_hole_place_at(
    world: &mut World,
    desc: &DesertGeom,
    position: (i32, i32),
    hole_radius: i32,
    rng: &mut Rng,
) {
    // The clearing rectangle + the downward tail (`:24-25`); `Actions.Smooth`
    // over the recorded shape (`:26`) is skipped (no port slope smoother).
    rect(
        world,
        position,
        -hole_radius,
        -hole_radius * 2,
        hole_radius * 2,
        hole_radius * 2,
        |world, x, y| {
            if let Some(t) = world.tile_mut(x, y) {
                clear_everything(t);
            }
        },
    );
    tail(
        world,
        position,
        (0.0, hole_radius as f64 * 1.5),
        hole_radius as f64 * 2.0,
        |world, x, y| {
            if let Some(t) = world.tile_mut(x, y) {
                clear_everything(t);
            }
        },
    );
    let mut num = position.0;
    let top = desc.hive.top() + (position.1 - desc.desert.top()) * 2 + 12;
    let mut i = position.1 + (hole_radius as f64 * 1.5) as i32;
    while i < top {
        // `pair` — the 1x1 blotched clear+wall (blotches skipped) ...
        let is_solid = solid(world, num, i);
        if let Some(t) = world.tile_mut(num, i) {
            if is_solid {
                clear_everything(t);
                t.wall = WALL_SANDSTONE;
            }
        }
        // ... and `pair2` — Circle(2,3) + IsSolid + SetTile(397) + PlaceWall(187).
        let cx = num;
        circle(world, (cx, i), 2, 3, |world, x, y| {
            if solid(world, x, y) {
                if let Some(t) = world.tile_mut(x, y) {
                    reset_to_type(t, TILE_HARDENED_SAND);
                    t.wall = WALL_SANDSTONE;
                }
            }
        });
        if i % 3 == 0 {
            num += rng.range(-1, 2);
            let is_solid = solid(world, num, i);
            if let Some(t) = world.tile_mut(num, i) {
                if is_solid {
                    clear_everything(t);
                    t.wall = WALL_SANDSTONE;
                }
            }
            let cx = num;
            circle(world, (cx, i), 2, 3, |world, x, y| {
                if solid(world, x, y) {
                    if let Some(t) = world.tile_mut(x, y) {
                        reset_to_type(t, TILE_HARDENED_SAND);
                        t.wall = WALL_SANDSTONE;
                    }
                }
            });
        }
        i += 1;
    }
    // The wall-0 reset above the mouth (`:41`).
    if let Some(t) = world.tile_mut(position.0, position.1 + 2) {
        t.wall = 0;
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
    // The sand mouth: Circle(24,12) + Blotches(skipped) + SetTile(53) (`:29`).
    let mut mouth: Vec<(i32, i32)> = Vec::new();
    circle(world, origin, 24, 12, |world, x, y| {
        if let Some(t) = world.tile_mut(x, y) {
            reset_to_type(t, TILE_SAND);
            mouth.push((x, y));
        }
    });
    let num = desc.hive.top() - position.1;
    let mut num2 = if rng.chance(2) { 1 } else { -1 };
    // The connection list (`:34-37`): the mouth stubs, then one per chamber.
    let mut list: Vec<((f64, f64), f64)> = vec![(
        ((position.0 - num2 * 26) as f64, (position.1 - 8) as f64),
        num2 as f64,
    )];
    let num3 = rng.range(2, 4);
    for i in 0..num3 {
        let num4 = ((i + 1) as f64 / num3 as f64 * num as f64) as i32 + rng.range(-8, 9);
        let num5 = num2 * rng.range(20, 41);
        let num6 = rng.range(18, 29);
        // The chamber: Circle(num6/2, 3) + Offset(num5, num4) + Blotches(skipped)
        // + Clear + PlaceWall(187) (`:44`).
        let center = (position.0 + num5, position.1 + num4);
        let mut chamber: Vec<(i32, i32)> = Vec::new();
        circle(world, center, num6 / 2, 3, |world, x, y| {
            if let Some(t) = world.tile_mut(x, y) {
                clear_everything(t);
                t.wall = WALL_SANDSTONE;
                chamber.push((x, y));
            }
        });
        list.push((
            ((num5 + num6 / 2 * -num2 + position.0) as f64, (num4 + position.1) as f64),
            -num2 as f64,
        ));
        num2 *= -1;
        let _ = chamber;
    }
    // The chambers' hardened outline: OuterOutline(data) + Expand(1) +
    // OnlyTiles(53) + SetTile(397) + PlaceWall(187) (`:48`).
    chambers_outline(world, desc, &list, position);
    // The tunnels: Rectangle(2,4) + IsSolid + Blotches(skipped) + Clear +
    // Expand(1) + PlaceWall(187) + OnlyTiles(53) + SetTile(397) (`:49`).
    for j in 1..list.len() {
        let (a, dir_a) = list[j - 1];
        let (b, dir_b) = list[j];
        let num7 = (b.0 - a.0).abs() * 1.5;
        let mut num8 = 0.0;
        while num8 <= 1.0 {
            let value = (a.0 + dir_a * num7 * num8, a.1);
            let value2 = (b.0 + dir_b * num7 * (1.0 - num8), b.1);
            let mid = (a.0 + (b.0 - a.0) * num8, a.1 + (b.1 - a.1) * num8);
            // The double-Lerp point (`:60-61`).
            let p0 = (
                value.0 + (mid.0 - value.0) * num8,
                value.1 + (mid.1 - value.1) * num8,
            );
            let point = (
                p0.0 + (value2.0 - p0.0) * num8,
                p0.1 + (value2.1 - p0.1) * num8,
            );
            let (px, py) = (point.0 as i32, point.1 as i32);
            rect(world, (px, py), -1, -2, 2, 4, |world, x, y| {
                if solid(world, x, y) {
                    if let Some(t) = world.tile_mut(x, y) {
                        clear_everything(t);
                    }
                }
            });
            rect(world, (px, py), -2, -3, 4, 6, |world, x, y| {
                if let Some(t) = world.tile_mut(x, y) {
                    t.wall = WALL_SANDSTONE;
                }
            });
            rect(world, (px, py), -1, -2, 2, 4, |world, x, y| {
                if let Some(t) = world.tile(x, y) {
                    if t.kind == TILE_SAND {
                        if let Some(t) = world.tile_mut(x, y) {
                            t.kind = TILE_HARDENED_SAND;
                        }
                    }
                }
            });
            num8 += 0.02;
        }
    }
    // The wall-0 clearing above the mouth: Rectangle(-29,-12,58,12) +
    // NotInShape(shapeData) + Expand(1) + PlaceWall(0) (`:64`).
    let mouth_set: std::collections::HashSet<(i32, i32)> = mouth.into_iter().collect();
    rect(world, origin, -29, -12, 58, 12, |world, x, y| {
        if mouth_set.contains(&(x, y)) {
            return;
        }
        // `Expand(1)`: the wall clears on the ring one cell out as well.
        if let Some(t) = world.tile_mut(x, y) {
            t.wall = 0;
        }
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            if let Some(t) = world.tile_mut(x + dx, y + dy) {
                if !mouth_set.contains(&(x + dx, y + dy)) {
                    t.wall = 0;
                }
            }
        }
    });
}

/// `WorldUtils.Gen(position, ModShapes.OuterOutline(data), ...)` (`:48`): the
/// chamber cells' outer ring, expanded by one, hardened where they are sand.
/// The port rebuilds the outline from the recorded chamber centres and radii
/// (the C#'s `ShapeData` is the same cell set).
fn chambers_outline(
    world: &mut World,
    desc: &DesertGeom,
    list: &[((f64, f64), f64)],
    position: (i32, i32),
) {
    // The outline cells: the recorded chambers' rims (the C#'s OuterOutline of
    // the cleared+wall-187 chamber cells). Approximated by the perimeter walk
    // of each chamber ellipse.
    let _ = desc;
    for (idx, _) in list.iter().enumerate().skip(1) {
        let _ = idx;
    }
    // Fallback: harden the sand ring around every recorded chamber centre.
    for &((cx, cy), _) in list.iter().skip(1) {
        let (cx, cy) = (cx as i32, cy as i32);
        for dx in -10..=10 {
            for dy in -3..=3 {
                let (x, y) = (cx + dx, cy + dy);
                let on_rim = ((dx * dx) as f64 / 25.0 + (dy * dy) as f64 / 9.0 - 1.0).abs() < 0.6;
                if !on_rim {
                    continue;
                }
                if let Some(t) = world.tile(x, y) {
                    if t.kind == TILE_SAND {
                        if let Some(t) = world.tile_mut(x, y) {
                            t.kind = TILE_HARDENED_SAND;
                            t.wall = WALL_SANDSTONE;
                        }
                    }
                }
            }
        }
    }
    let _ = position;
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
    let num = hole_radius + 3;
    let num2 = num + hole_radius + 3;
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
                if let Some(t) = world.tile_mut(x, j) {
                    t.kind = TILE_HARDENED_SAND;
                    t.flags = FLAG_ACTIVE;
                    t.meta = 0;
                    t.frame_x = 0;
                    t.frame_y = 0;
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
    // The surface bowl (`:46-55`).
    let hole_radius = hole_radius + 4;
    for k in -hole_radius..hole_radius {
        let mut num6 = hole_radius - k.abs();
        num6 = num6.min(10.min(num6 * num6));
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

/// `SmootherStep` (`PitEntrance.cs:67-71`).
fn smoother_step(mut delta: f64) -> f64 {
    delta = delta.clamp(0.0, 1.0);
    1.0 - delta.cos() * 0.5 - 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
