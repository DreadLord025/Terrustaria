//! The Lihzahrd temple — the M5-BIOME phase-6 seam (the faithful
//! `LihzahrdTemple` placement search, `WorldGen.cs:15834-15926`) plus the
//! M5-TEMPLE phase-1 interior port: the room walk and the brick generation
//! (`makeTemple`, `WorldGen.cs:33440-33548` + `:34238-34462`).
//!
//! The C# temple (`makeTemple`, `WorldGen.cs:33435-34237`): the brick shell
//! (the Lihzahrd brick 226, the walls 87 unsafe / 112 safe) buried in the
//! jungle cavern below the rock layer, the interior rooms carved, the altar
//! (237) in the main room, the dart traps (137) in the walls, the
//! teleporter pads, the chests. Ported so far: the placement search
//! (M5-BIOME), the room walk + the depth-unioned brick fills + the corner
//! Line walkers (phase 1). Still to port (the M5-TEMPLE phases 2-3): the
//! jitter carve sweeps (`:33550-33684`), the `templePather` corridors
//! (`:33686-33787`), the `outerTempled` sweeps (`:33819-33850`), the
//! entrance shaft + the style-11 door (`:33852-33857`), the `templeCleaner`
//! sweeps (`:33954-33961`, `:33335`), the altar publication (`:33988-34068`),
//! the spike loop (`:34069-34217`), `templePart2` (`:34464`) and the
//! `LihzahrdAltar` re-write (`:21943`).
//!
//! What publishes state: the bounds `t_left/t_right/t_top/t_bottom` = the
//! room-union ± 10 (the C# publishes the filled extent ± 10,
//! `:33788-33817`, published `:34231-34235`), `t_rooms` = the final walk
//! count (`:34235`) and the altar anchor `l_altar_x/l_altar_y` = the 3×2
//! altar's top-left (`:34000-34001`, hand-built fallback `:34038-34039`) —
//! the fields `LihzahrdTemplePart2` (`:17701`) and `LihzahrdAltar`
//! (`:21943`) consume (M5-TEMPLE's row owns the full interior). Until the
//! phase-2 altar port lands, the pass still publishes the stub shell's
//! bounds + anchor (kept adjacent to the faithful walk so the worldgen
//! stays coherent mid-port).
//!
//! Deviations (ANALYSIS "Lihzahrd temple (M1.4-WGEN-6)"): the C#'s
//! multi-room crawl with the doors and the teleporters is the shape-level
//! approximation (the box + the 3 corridor rooms); the Plantera-gate (the
//! altar needs the Plantera downed to interact) waits for M5-HARDFORCE;
//! the trap wiring arms ride the wiring engine (the traps 137 are the
//! case-137 variants the wiring trip fires).

use super::pipeline::TempleBounds;
use crate::tiledata as td;
use crate::tile::{FLAG_ACTIVE, Tile};
use crate::world::World;

pub(crate) const TILE_TEMPLE_BRICK: u16 = 226;
pub(crate) const TILE_ALTAR: u16 = 237;
pub(crate) const WALL_TEMPLE_UNSAFE: u16 = 87;
pub(crate) const WALL_TEMPLE: u16 = 112;

/// The vanilla `DungeonSide.Right` (`WorldBuilding/DungeonSide.cs:13-15` —
/// Left −1 / None 0 / Right 1). The port's dungeon sits on the right
/// (`plan_spans`' `dungeon_x`; decision M5-BIOME #10), so the search's
/// `-dungeonSide` multiplier is the constant −1 and the jungle (the temple's
/// host) is on the dungeon-opposite side.
const DUNGEON_SIDE_RIGHT: i32 = 1;

/// The `UnifiedRandom.NextDouble()` stand-in: one xorshift draw scaled to
/// [0, 1). The xorshift-for-`UnifiedRandom` substitution is the recorded
/// stream-model deviation (`Streams::pass_rng`'s docs); the fraction mapping
/// is the port-level convention.
fn next_double(rng: &mut super::Rng) -> f64 {
    f64::from(rng.next_u32()) / 4294967296.0
}

/// The faithful `LihzahrdTemple` placement search (`WorldGen.cs:15834-15926`):
/// per attempt sample `y` from `[rockLayerClamped, maxTilesY - 600)` and `x`
/// from the dungeon-opposite reach `((NextDouble() * num3 + 0.1) *
/// -dungeonSide + 0.5) * maxTilesX`, accept the first sample landing on
/// active jungle grass (tile 60), widen `num3` by 0.05 after every 1,000,000
/// failed attempts (capped 0.35 — `Math.Min` lands it exactly on 0.35, so the
/// `== 0.35` check fires on the next overflow) and give up to the fallback
/// position after 11 overflows at the cap. Returns the bury point
/// `makeTemple(x, y)` receives.
///
/// Deviations (the sub-vanilla valve, decision M5-BIOME #12's shape): when
/// `maxTilesY - 600 <= 0` the vanilla y band cannot exist in-world (vanilla's
/// smallest world is 4200×1200, where the band floor is 600) — every sample
/// would miss and the C# loop would spin its full ~13M-attempt termination
/// before falling through to the fallback below, so the port skips straight
/// to it. The remix branch (`:15852-15876`) is seed-gated (`remixWorldGen`/
/// `notTheBees`; M5-SEED scope) and never runs on the classic chain.
pub(crate) fn find_temple_site(
    world: &World,
    dungeon_x: i32,
    rock_layer: f64,
    rng: &mut super::Rng,
) -> (i32, i32) {
    let mut found = false;
    let mut site = (0, 0);
    // The sub-vanilla valve (see the doc comment): `num5 <= 0` means the
    // vanilla band `[(int)rockLayer, maxTilesY - 600)` has no in-world rows.
    if world.height - 600 > 0 {
        // `:15837-15840`: the widen-stage counter, the failed-attempt counter
        // and the x-reach factor.
        let mut num: i32 = 0;
        let mut num2: u64 = 0;
        let mut num3: f64 = 0.25;
        loop {
            // `:15842-15847`: the y band (recomputed per iteration in the C#;
            // `(int)Main.rockLayer` truncates like `as i32`, and `num4` is
            // clamped so `Next(num4, num5)` stays a valid 1-wide band).
            let num5 = world.height - 600;
            let mut num4 = rock_layer as i32;
            if num4 > num5 - 1 {
                num4 = num5 - 1;
            }
            // `:15848`: the y draw first, then the x fraction.
            let num6 = rng.range(num4, num5);
            // `:15848`: the x formula — `NextDouble()` then the
            // dungeon-opposite reach; the C# `(int)` cast truncates toward
            // zero like `as i32` (the value is positive: the reach is
            // [0.05·maxTilesX, 0.4·maxTilesX] for `dungeonSide == Right`).
            let num7 = (((next_double(rng) * num3 + 0.1) * -(DUNGEON_SIDE_RIGHT as f64) + 0.5)
                * world.width as f64) as i32;
            // `:15877-15882` (the remix branch is above): accept the first
            // active jungle-grass tile.
            if world
                .tile(num7, num6)
                .is_some_and(|t| t.active() && t.kind == 60)
            {
                found = true;
                site = (num7, num6);
                break;
            }
            // `:15900-15912`: the widening valve — the post-increment compare,
            // then `num3 = Math.Min(0.35, num3 + 0.05)` with the overflow
            // counter giving up after 11 stages at the cap.
            let old = num2;
            num2 += 1;
            if old > 1_000_000 {
                if num3 == 0.35 {
                    num += 1;
                    if num > 10 {
                        break;
                    }
                }
                num3 = (num3 + 0.05).min(0.35);
                num2 = 0;
            }
        }
    }
    if !found {
        // `:15914-15923`: the fallback — the dungeon-mirrored x
        // (`generatingDungeonPositionX` is the dungeon entrance x,
        // `DungeonCrawler.cs:352/:399`; the port's stand-in is
        // `BiomeSpans.dungeon_x`) and 100 rows below the rock layer.
        site = (world.width - dungeon_x, rock_layer as i32 + 100);
    }
    site
}

/// `WorldGen.InWorld(i, j, fluff)` (`WorldGen.cs:9688-9701`): the cell
/// plus `fluff` tiles of margin must sit inside the world.
pub(crate) fn in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// The C# `Rectangle` (`System.Drawing`): `x`/`y` the top-left, `w`/`h` the
/// size. `Right`/`Bottom` are exclusive ends (`x + w`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    fn right(&self) -> i32 {
        self.x + self.w
    }

    fn bottom(&self) -> i32 {
        self.y + self.h
    }

    /// `Rectangle.Intersects` — the half-open axis-aligned overlap test
    /// (edges touching do not count).
    fn intersects(&self, other: &Rect) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }
}

/// One walk room + the depth level the brick union groups it by — the C#'s
/// `array[i]` (the rect) with `array2[i]` (the depth, `WorldGen.cs:33537`).
#[derive(Clone, Copy, Debug)]
pub(crate) struct TempleRoom {
    pub rect: Rect,
    pub depth: i32,
}

/// The per-depth `DungeonBounds` slice (`DungeonBounds.cs:21-101`): the room
/// union at one depth level. `Inflate(amount)` (`:108-111`) moves all four
/// edges by `± amount`; the port applies it at build time.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Bounds {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

/// `makeTemple`'s room walk (`WorldGen.cs:33440-33548`): `num2` rooms
/// (`Next((int)(scale*10), (int)(scale*16))`, `scale = maxTilesX / 4200`)
/// stepping sideways from the bury point, flipping the horizontal/vertical
/// stagger every `Next(1, 3)` rooms and drifting downward (`num19`, the ±100
/// ladder), each rect re-rolled until it misses every earlier room. The last
/// room (or the first one hitting the underworld ceiling) collapses to the
/// big altar room. Returns the rects with their depth levels — the
/// `makeTemple_GenerateBricks` (`WorldGen.cs:34238+`) input.
///
/// The C# shares one `genRand` across the whole world; the port's per-pass
/// `pass_rng` stream is the recorded deviation — the draws inside follow the
/// C# statement order 1:1.
pub(crate) fn make_temple_rooms(
    rng: &mut super::Rng,
    scale: f64,
    underworld_layer: i32,
    x: i32,
    y: i32,
) -> TempleWalk {
    // `:33440-33452`: the room count, the first direction and the flip
    // period. `num3` is the current direction; `num4` the per-room copy the
    // stagger flips.
    let mut num2 = rng.range((scale * 10.0) as i32, (scale * 16.0) as i32);
    let num3 = if rng.range(0, 2) == 0 { -1 } else { 1 };
    let mut num4 = num3;
    let (mut num7, mut num8) = (x, y);
    let mut num9 = rng.range(1, 3);
    let mut num10 = 0;
    let mut num13 = 0;
    let mut num19 = -10;
    let start_dir = num3;
    let mut rooms: Vec<TempleRoom> = Vec::new();
    let mut i = 0;
    while i < num2 {
        num10 += 1;
        let mut num14 = num4;
        let mut num15 = num7;
        let mut num16 = num8;
        let mut rect = Rect {
            x: num15,
            y: num16,
            w: 0,
            h: 0,
        };
        let mut flag = true;
        while flag {
            num15 = num7;
            num16 = num8;
            let mut num17 = rng.range(25, 50);
            let mut num18 = rng.range(20, 35);
            if num18 > num17 {
                num18 = num17;
            }
            // `:33472`: the underworld-ceiling test on the CURRENT position
            // (`UnderworldLayer` = maxTilesY - 200, passed in).
            let flag2 = num16 + 70 >= underworld_layer - 10;
            // `:33475-33486`: the last-room resize (the altar room).
            if i == num2 - 1 || flag2 {
                num2 = i + 1;
                num17 = rng.range(55, 65);
                num18 = rng.range(45, 50);
                if num18 > num17 {
                    num18 = num17;
                }
                num17 = (num17 as f64 * 1.6) as i32;
                num18 = (num18 as f64 * 1.35) as i32;
                num16 += rng.range(5, 10);
            }
            if num10 > num9 {
                num16 += rng.range(num18 + 1, num18 + 3) + num19;
                num15 += rng.range(-5, 6);
                num14 = num3 * -1;
            } else {
                num15 += (rng.range(num17 + 1, num17 + 3) + num19) * num14;
                num16 += rng.range(-5, 6);
            }
            flag = false;
            rect = Rect {
                x: num15 - num17 / 2,
                y: num16 - num18 / 2,
                w: num17,
                h: num18,
            };
            // `:33512`: the underworld trigger skips the intersection scan
            // (the big room overlaps everything by design).
            if flag2 {
                break;
            }
            // `:33515-33522`: the retry loop — every earlier rect, and the
            // 1/100 downward drift (`num19` walks-wide).
            for j in 0..i as usize {
                if rooms[j].rect.intersects(&rect) {
                    flag = true;
                }
                if rng.range(0, 100) == 0 {
                    num19 += 1;
                }
            }
        }
        // `:33537`: the last room of the walk carries depth + 1 (in practice
        // the shrink above already made `i == num2` unreachable — the C#
        // ternary stays verbatim).
        let depth = if i == num2 { num13 + 1 } else { num13 };
        if num10 > num9 {
            num9 += 1;
            num10 = 1;
            num13 += 1;
        }
        rooms.push(TempleRoom { rect, depth });
        num4 = num14;
        num7 = num15;
        num8 = num16;
        i += 1;
    }
    TempleWalk {
        rooms,
        start_dir,
        end_dir: num3,
    }
}

/// `makeTemple_GenerateBricks`'s first loop (`WorldGen.cs:34244-34265`): the
/// per-depth `DungeonBounds` unions — one level per depth value that occurs,
/// the null levels of the C#'s sparse array skipped.
fn depth_bounds(rooms: &[TempleRoom]) -> Vec<Option<Bounds>> {
    let mut levels: Vec<Option<Bounds>> = Vec::new();
    for room in rooms {
        let idx = room.depth as usize;
        if levels.len() <= idx {
            levels.resize(idx + 1, None);
        }
        let (left, top) = (room.rect.x, room.rect.y);
        let (right, bottom) = (room.rect.right(), room.rect.bottom());
        levels[idx] = Some(match levels[idx] {
            // `DungeonBounds.SetBounds` (`DungeonBounds.cs:249-256`) for the
            // first room at the depth, `UpdateBounds` (`:285-297`) for the
            // rest.
            None => Bounds {
                left,
                top,
                right,
                bottom,
            },
            Some(prev) => Bounds {
                left: prev.left.min(left),
                top: prev.top.min(top),
                right: prev.right.max(right),
                bottom: prev.bottom.max(bottom),
            },
        });
    }
    levels
}

/// The brick write of the GenerateBricks family — the C#'s
/// `ClearEverything(); active(true); type = 226` (`WorldGen.cs:34272-34275`,
/// `:34366-34369`, the FillY/FillX bodies `:34404-34407`): everything
/// resets, so the wall drops to 0.
fn clear_and_brick(world: &mut World, x: i32, y: i32) {
    if !in_world(world, x, y, 0) {
        return;
    }
    let i = world.index(x, y);
    world.tiles[i] = Tile {
        kind: TILE_TEMPLE_BRICK,
        flags: FLAG_ACTIVE,
        ..Default::default()
    };
}

/// `makeTemple_GenerateBricks` (`WorldGen.cs:34238-34299`): the depth unions
/// inflated by 10, the brick fill over each union box, then the four
/// corner-to-corner `Line_StrictAngles` between every pair of levels
/// (bridging the depth stagger into one shell).
pub(crate) fn make_temple_generate_bricks(world: &mut World, rooms: &[TempleRoom]) {
    let mut levels: Vec<Bounds> = depth_bounds(rooms)
        .into_iter()
        .flatten()
        .map(|mut b| {
            // `DungeonBounds.Inflate(10)` (`DungeonBounds.cs:108-111`).
            b.left -= 10;
            b.top -= 10;
            b.right += 10;
            b.bottom += 10;
            b
        })
        .collect();
    for b in &levels {
        for k in b.left..=b.right {
            for l in b.top..=b.bottom {
                clear_and_brick(world, k, l);
            }
        }
    }
    for m in 0..levels.len() {
        let a = levels[m];
        for n in m + 1..levels.len() {
            let b = levels[n];
            generate_bricks_line_strict_angles(world, a.left, a.top, b.left, b.top, false, false);
            generate_bricks_line_strict_angles(world, a.right, a.top, b.right, b.top, true, false);
            generate_bricks_line_strict_angles(world, a.left, a.bottom, b.left, b.bottom, false, true);
            generate_bricks_line_strict_angles(
                world,
                a.right,
                a.bottom,
                b.right,
                b.bottom,
                true,
                true,
            );
        }
    }
}

/// The squared distance — the C# compares `origin.Distance(...)` values
/// (`Vector2.Distance`, f32); the squared compare is equivalent for the
/// non-negative distances here.
fn dist2(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let (dx, dy) = (bx - ax, by - ay);
    dx * dx + dy * dy
}

/// `makeTemple_GenerateBricks_Line_StrictAngles`
/// (`WorldGen.cs:34301-34339`): the 500-step L-corner staircase between the
/// two anchors, with the straight `_Line` fallback when the corner isn't
/// reached in time (or the anchors share an axis).
fn generate_bricks_line_strict_angles(
    world: &mut World,
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
    fill_left: bool,
    fill_upwards: bool,
) {
    if start_x == end_x || start_y == end_y {
        generate_bricks_line(world, start_x, start_y, end_x, end_y, fill_left, fill_upwards);
        return;
    }
    // `:34309-34315`: the corner origin (the opposite corner in the fill
    // axes) and which endpoint walks to the other.
    let (ox, oy) = if fill_upwards {
        (end_x as f32, start_y as f32)
    } else {
        (start_x as f32, end_y as f32)
    };
    let (sx, sy) = (start_x as f32, start_y as f32);
    let (ex, ey) = (end_x as f32, end_y as f32);
    let (mut vector, mut vector2) = ((sx, sy), (ex, ey));
    if dist2(ox, oy, ex, ey) > dist2(ox, oy, sx, sy) {
        std::mem::swap(&mut vector, &mut vector2);
    }
    let mut vector3 = vector;
    // `:34319-34320`: the per-axis step sign (never 0 — the axes differ).
    let vector4 = (
        if vector2.0 > vector.0 { 1.0 } else { -1.0 },
        if vector2.1 > vector.1 { 1.0 } else { -1.0 },
    );
    let mut num3 = 500;
    while vector3.0 != vector2.0 && vector3.1 != vector2.1 {
        num3 -= 1;
        if num3 < 0 {
            break;
        }
        vector3.0 += vector4.0;
        vector3.1 += vector4.1;
    }
    if num3 < 0 {
        generate_bricks_line(world, start_x, start_y, end_x, end_y, fill_left, fill_upwards);
        return;
    }
    let (corner_x, corner_y) = (vector3.0 as i32, vector3.1 as i32);
    generate_bricks_line(world, start_x, start_y, corner_x, corner_y, fill_left, fill_upwards);
    generate_bricks_line(world, corner_x, corner_y, end_x, end_y, fill_left, fill_upwards);
}

/// `makeTemple_GenerateBricks_Line` (`WorldGen.cs:34341-34380`): the
/// normalized walk from start to end (the 2000-step cap), a brick at every
/// `Math.Round`'d step (banker's rounding), the FillY/FillX gap closers, and
/// the `InWorld(x, y, 5)` guard that ends the walk at the world edge.
fn generate_bricks_line(
    world: &mut World,
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
    fill_left: bool,
    fill_upwards: bool,
) {
    let vector = (start_x as f32, start_y as f32);
    let vector2 = ((end_x as f32) - vector.0, (end_y as f32) - vector.1);
    if vector2 == (0.0, 0.0) {
        return;
    }
    // `:34355-34357`: `SafeNormalize(Vector2.UnitX)` — the delta is non-zero
    // here, so this is the plain normalize; `num2` is the unit step length.
    let num = (vector2.0 * vector2.0 + vector2.1 * vector2.1).sqrt();
    let vector3 = (vector2.0 / num, vector2.1 / num);
    let num2 = (vector3.0 * vector3.0 + vector3.1 * vector3.1).sqrt();
    let mut num3 = 2000;
    let mut last_x_adjusted = -1;
    let mut last_y_adjusted = -1;
    let mut at = vector;
    let mut remaining = num;
    while remaining > 0.0 {
        num3 -= 1;
        if num3 <= 0 {
            break;
        }
        let x4 = crate::damage::round_bankers(at.0 as f64) as i32;
        let y5 = crate::damage::round_bankers(at.1 as f64) as i32;
        if !in_world(world, x4, y5, 5) {
            break;
        }
        clear_and_brick(world, x4, y5);
        generate_bricks_line_fill_y(world, fill_upwards, last_x_adjusted, x4, y5);
        generate_bricks_line_fill_x(world, fill_left, last_y_adjusted, x4, y5);
        last_x_adjusted = x4;
        last_y_adjusted = y5;
        at.0 += vector3.0;
        at.1 += vector3.1;
        remaining -= num2;
    }
}

/// `makeTemple_GenerateBricks_Line_FillY` (`WorldGen.cs:34382-34421`): march
/// from the point to the next brick in the fill direction (300-step cap, the
/// 10-tile edge guard) and brick the gap.
fn generate_bricks_line_fill_y(
    world: &mut World,
    fill_upwards: bool,
    last_x_adjusted: i32,
    current_x: i32,
    current_y: i32,
) {
    if current_x == last_x_adjusted {
        return;
    }
    let step = if fill_upwards { -1 } else { 1 };
    // `:34394-34408`: the scan to the next brick.
    let mut scan = current_y + step;
    let mut num3 = 0;
    while !is_temple_brick(world, current_x, scan) {
        num3 += 1;
        scan += step;
        if world.tile(current_x, scan).is_none() || !in_world(world, current_x, scan, 5) {
            break;
        }
    }
    if num3 >= 300 || scan <= 10 || scan >= world.height - 10 {
        return;
    }
    // `:34413-34421`: the gap fill.
    let mut write = current_y + step;
    while !is_temple_brick(world, current_x, write) {
        clear_and_brick(world, current_x, write);
        write += step;
        if world.tile(current_x, write).is_none() || !in_world(world, current_x, write, 5) {
            break;
        }
    }
}

/// `makeTemple_GenerateBricks_Line_FillX` (`WorldGen.cs:34423-34462`): the
/// horizontal twin of `_FillY` — the `Main.maxTilesX - 10` edge guard.
fn generate_bricks_line_fill_x(
    world: &mut World,
    fill_left: bool,
    last_y_adjusted: i32,
    current_x: i32,
    current_y: i32,
) {
    if current_y == last_y_adjusted {
        return;
    }
    let step = if fill_left { -1 } else { 1 };
    let mut scan = current_x + step;
    let mut num3 = 0;
    while !is_temple_brick(world, scan, current_y) {
        num3 += 1;
        scan += step;
        if world.tile(scan, current_y).is_none() || !in_world(world, scan, current_y, 5) {
            break;
        }
    }
    if num3 >= 300 || scan <= 10 || scan >= world.width - 10 {
        return;
    }
    let mut write = current_x + step;
    while !is_temple_brick(world, write, current_y) {
        clear_and_brick(world, write, current_y);
        write += step;
        if world.tile(write, current_y).is_none() || !in_world(world, write, current_y, 5) {
            break;
        }
    }
}

/// The C#'s `tile.active() && tile.type == 226` test the Fill scans use
/// (`WorldGen.cs:34400` / `:34441`).
fn is_temple_brick(world: &World, x: i32, y: i32) -> bool {
    world
        .tile(x, y)
        .is_some_and(|t| t.active() && t.kind == TILE_TEMPLE_BRICK)
}

/// `WorldGen.SolidTile(i, j)` (`WorldGen.cs:70111-70133`) — no RNG; an
/// out-of-world cell reads as the C#'s null tile (`true`). During
/// `makeTemple` the C# `tileSolid` table sits at its defaults (162
/// BreakableIce and 226 LihzahrdBrick both solid — nothing before the
/// temple pass toggles them, `Main.cs:7145`), which the port's generated
/// sets match; the temple-part-2 windows arrive with that pass.
fn solid_tile_at(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && td::TILE_SOLID.has(t.kind)
        && !td::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && (t.flags & crate::tile::FLAG_INACTIVE) == 0
}

/// `WorldGen.SolidTile2(i, j)` (`WorldGen.cs:70134-70150`): the solid-only
/// twin (no `tileSolidTop` exclusion); an out-of-world cell reads as solid.
fn solid_tile2_at(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && td::TILE_SOLID.has(t.kind)
        && t.slope() == 0
        && !t.half_brick()
        && (t.flags & crate::tile::FLAG_INACTIVE) == 0
}

/// The C#'s `SolidOrSlopedTile` (`WorldGen.cs:69985-70200`) slice the
/// temple's floor scans use: active, solid-or-solid-top, no half brick —
/// with the `includePlatforms = false` default the platform variants drop.
fn solid_or_sloped(t: &Tile) -> bool {
    t.active()
        && (t.kind as usize) < td::TILE_COUNT
        && (td::TILE_SOLID.has(t.kind) || td::TILE_SOLID_TOP.has(t.kind))
        && !t.half_brick()
}

/// The temple carve — `active(false); wall = 87` (`WorldGen.cs:33630-33631`
/// and the corridor/shaft/cleaner bodies): the kind and frames stay stored
/// under the inactive flag, only the wall is written.
fn carve_cell(world: &mut World, x: i32, y: i32) {
    if !in_world(world, x, y, 0) {
        return;
    }
    let i = world.index(x, y);
    let t = &mut world.tiles[i];
    t.flags &= !FLAG_ACTIVE;
    t.wall = WALL_TEMPLE_UNSAFE;
}

/// The temple brick — `active(true); type = 226; liquid = 0; slope(0);
/// halfBrick(false)` (`WorldGen.cs:33557-33564`, `:33886-33889`): the wall
/// is kept (unlike the GenerateBricks `ClearEverything` write).
fn brick_cell(world: &mut World, x: i32, y: i32) {
    if !in_world(world, x, y, 0) {
        return;
    }
    let i = world.index(x, y);
    let t = &mut world.tiles[i];
    t.kind = TILE_TEMPLE_BRICK;
    t.flags |= FLAG_ACTIVE;
    t.liquid = 0;
    t.set_slope(0);
    t.flags &= !crate::tile::FLAG_HALF_BRICK;
}

/// The walk result the shaft and corridor steps read besides the rooms: the
/// C#'s `num4` — the initial direction (`WorldGen.cs:33444`, never
/// reassigned in the walk, so the shaft can walk its opposite) — and `num3`
/// — the direction after the last room (`:33546`, the corridor door's
/// 0.2/0.8 width bias).
#[derive(Clone, Debug)]
pub(crate) struct TempleWalk {
    pub rooms: Vec<TempleRoom>,
    pub start_dir: i32,
    pub end_dir: i32,
}

/// `makeTemple`'s jitter carve (`WorldGen.cs:33549-33684`): every room is
/// brick-filled first, then its interior is carved by two sweeps — the
/// forward one and the reversed one over the same drifting edges — each
/// step re-rolling the four inset edges by ±1 on a 1/20 chance and clamping
/// them to the room and its centre, carving the cells inside the jittered
/// box with the unsafe wall (87).
fn carve_room_interiors(world: &mut World, rooms: &[TempleRoom], rng: &mut super::Rng) {
    for room in rooms {
        let r = room.rect;
        for l in r.x..r.right() {
            for m in r.y..r.bottom() {
                brick_cell(world, l, m);
            }
        }
        // `:33564-33568`: the inner inset by `Next(3, 8)` on each side.
        let x2 = r.x + rng.range(3, 8);
        let num20 = r.right() - rng.range(3, 8);
        let y2 = r.y + rng.range(3, 8);
        let num21 = r.bottom() - rng.range(3, 8);
        let (mut num22, mut num23) = (x2, num20);
        let (mut num24, mut num25) = (y2, num21);
        let num26 = (x2 + num20) / 2;
        let num27 = (y2 + num21) / 2;
        // `:33576-33632`: the forward sweep — n ascending, num28 ascending.
        for n in x2..num20 {
            for num28 in y2..num21 {
                if rng.range(0, 20) == 0 {
                    num24 += rng.range(-1, 2);
                }
                if rng.range(0, 20) == 0 {
                    num25 += rng.range(-1, 2);
                }
                if rng.range(0, 20) == 0 {
                    num22 += rng.range(-1, 2);
                }
                if rng.range(0, 20) == 0 {
                    num23 += rng.range(-1, 2);
                }
                clamp_carve_edges(&mut num22, &mut num23, &mut num24, &mut num25, x2, num20, y2, num21, num26, num27);
                if n >= num22 && n < num23 && num28 >= num24 && num28 <= num25 {
                    carve_cell(world, n, num28);
                }
            }
        }
        // `:33634-33682`: the reversed sweep — num29 descending, num30
        // descending, the same four rolls and clamps over the same edges.
        let mut num29 = num21;
        while num29 > y2 {
            let mut num30 = num20;
            while num30 > x2 {
                if rng.range(0, 20) == 0 {
                    num24 += rng.range(-1, 2);
                }
                if rng.range(0, 20) == 0 {
                    num25 += rng.range(-1, 2);
                }
                if rng.range(0, 20) == 0 {
                    num22 += rng.range(-1, 2);
                }
                if rng.range(0, 20) == 0 {
                    num23 += rng.range(-1, 2);
                }
                clamp_carve_edges(&mut num22, &mut num23, &mut num24, &mut num25, x2, num20, y2, num21, num26, num27);
                if num30 >= num22 && num30 < num23 && num29 >= num24 && num29 <= num25 {
                    carve_cell(world, num30, num29);
                }
                num30 -= 1;
            }
            num29 -= 1;
        }
    }
}

/// The eight clamps of the carve sweeps (`WorldGen.cs:33588-33611` /
/// `:33646-33669`): the left/right edges stay between the inset and the
/// centre, the top/bottom between the inset and the centre.
#[allow(clippy::too_many_arguments)]
fn clamp_carve_edges(
    num22: &mut i32,
    num23: &mut i32,
    num24: &mut i32,
    num25: &mut i32,
    x2: i32,
    num20: i32,
    y2: i32,
    num21: i32,
    num26: i32,
    num27: i32,
) {
    if *num22 < x2 {
        *num22 = x2;
    }
    if *num23 > num20 {
        *num23 = num20;
    }
    if *num24 < y2 {
        *num24 = y2;
    }
    if *num25 > num21 {
        *num25 = num21;
    }
    if *num22 > num26 {
        *num22 = num26;
    }
    if *num23 < num26 {
        *num23 = num26;
    }
    if *num24 > num27 {
        *num24 = num27;
    }
    if *num25 < num27 {
        *num25 = num27;
    }
}

/// `WorldGen.templePather` (`WorldGen.cs:33372-33408`): up to `Next(5, 20)`
/// unit steps toward the destination, carving a `[x - Next(2, 5),
/// x + Next(2, 5))` square of unsafe wall at every step. Returns the
/// integer position the C#'s `Vector2D` carries (the path keeps double
/// coordinates; every value is integral).
fn temple_pather(
    world: &mut World,
    path: (i32, i32),
    dest_x: i32,
    dest_y: i32,
    rng: &mut super::Rng,
) -> (i32, i32) {
    let (mut num, mut num2) = path;
    let mut num3 = rng.range(5, 20);
    let num4 = rng.range(2, 5);
    while num3 > 0 && (num != dest_x || num2 != dest_y) {
        num3 -= 1;
        if num > dest_x {
            num -= 1;
        }
        if num < dest_x {
            num += 1;
        }
        if num2 > dest_y {
            num2 -= 1;
        }
        if num2 < dest_y {
            num2 += 1;
        }
        for i in (num - num4)..(num + num4) {
            for j in (num2 - num4)..(num2 + num4) {
                carve_cell(world, i, j);
            }
        }
    }
    (num, num2)
}

/// The C#'s `Vector2D`-valued path check (`WorldGen.cs:33697-33700`): the
/// double compare collapses to the integer one (every value is integral).
fn at_dest(path: (i32, i32), dest_x: i32, dest_y: i32) -> bool {
    path == (dest_x, dest_y)
}

/// `makeTemple`'s corridor loop (`WorldGen.cs:33686-33787`): from the bury
/// point, per room a random (or centre) target chased by `templePather`,
/// then a corridor to the next room's edge (the 0.2/0.8 width bias by
/// direction and relative x) or its midpoint. The room inset is ± 8
/// (`:33688-33691`).
fn temple_corridors(
    world: &mut World,
    walk: &TempleWalk,
    bury: (i32, i32),
    rng: &mut super::Rng,
) {
    let rooms = &walk.rooms;
    let num2 = rooms.len() as i32;
    let mut path = bury;
    for num31 in 0..num2 {
        let k = num31 as usize;
        // `:33688-33691`: the current room inset ± 8.
        let mut r2 = rooms[k].rect;
        r2.x += 8;
        r2.y += 8;
        r2.w -= 16;
        r2.h -= 16;
        let mut flag3 = true;
        while flag3 {
            let mut num32 = rng.range(r2.x, r2.right());
            let mut num33 = rng.range(r2.y, r2.bottom());
            if num31 == num2 - 1 {
                // `:33695-33698`: the last room's target is its centre ± 10.
                num32 = r2.x + r2.w / 2 + rng.range(-10, 11);
                num33 = r2.y + r2.h / 2 + rng.range(-10, 11);
            }
            path = temple_pather(world, path, num32, num33, rng);
            if at_dest(path, num32, num33) {
                flag3 = false;
            }
        }
        if num31 >= num2 - 1 {
            continue;
        }
        if rng.range(0, 3) != 0 {
            // `:33703-33730`: the next-room edge corridor.
            let num34 = (num31 + 1) as usize;
            if rooms[num34].rect.y >= rooms[k].rect.bottom() {
                r2.x = rooms[num34].rect.x;
                if num31 == 0 {
                    if walk.end_dir > 0 {
                        r2.x += (rooms[num34].rect.w as f64 * 0.8) as i32;
                    } else {
                        r2.x += (rooms[num34].rect.w as f64 * 0.2) as i32;
                    }
                } else if rooms[num34].rect.x < rooms[k].rect.x {
                    r2.x += (rooms[num34].rect.w as f64 * 0.2) as i32;
                } else {
                    r2.x += (rooms[num34].rect.w as f64 * 0.8) as i32;
                }
                r2.y = rooms[num34].rect.y;
            } else {
                r2.x = (rooms[k].rect.x + rooms[k].rect.w / 2
                    + rooms[num34].rect.x
                    + rooms[num34].rect.w / 2)
                    / 2;
                r2.y = (rooms[num34].rect.y as f64 + rooms[num34].rect.h as f64 * 0.8) as i32;
            }
            let (x3, y3) = (r2.x, r2.y);
            flag3 = true;
            while flag3 {
                let num35 = rng.range(x3 - 6, x3 + 7);
                let num36 = rng.range(y3 - 6, y3 + 7);
                path = temple_pather(world, path, num35, num36, rng);
                if at_dest(path, num35, num36) {
                    flag3 = false;
                }
            }
            continue;
        }
        // `:33748-33758`: the midpoint corridor (`Next(3) == 0`).
        let num37 = (num31 + 1) as usize;
        let num38 = (rooms[k].rect.x + rooms[k].rect.w / 2
            + rooms[num37].rect.x
            + rooms[num37].rect.w / 2)
            / 2;
        let num39 = (rooms[k].rect.y + rooms[k].rect.h / 2
            + rooms[num37].rect.y
            + rooms[num37].rect.h / 2)
            / 2;
        flag3 = true;
        while flag3 {
            let num40 = rng.range(num38 - 6, num38 + 7);
            let num41 = rng.range(num39 - 6, num39 + 7);
            path = temple_pather(world, path, num40, num41, rng);
            if at_dest(path, num40, num41) {
                flag3 = false;
            }
        }
    }
}

/// `WorldGen.outerTempled` (`WorldGen.cs:33409-33433`): if the cell is
/// already temple brick (active 226) or unsafe wall (87) it is left alone;
/// otherwise the first cell in the ± 6 square that is inactive with wall 87
/// becomes the brick — the temple's outer skin converting the carved shell.
fn outer_templed(world: &mut World, x: i32, y: i32) {
    if !in_world(world, x, y, 0) {
        return;
    }
    let i = world.index(x, y);
    let here_active_brick = world.tiles[i].active() && world.tiles[i].kind == TILE_TEMPLE_BRICK;
    if here_active_brick || world.tiles[i].wall == WALL_TEMPLE_UNSAFE {
        return;
    }
    for i_x in (x - 6)..=(x + 6) {
        for j in (y - 6)..=(y + 6) {
            let Some(t) = world.tile(i_x, j) else {
                continue;
            };
            if !t.active() && t.wall == WALL_TEMPLE_UNSAFE {
                brick_cell(world, i_x, j);
                return;
            }
        }
    }
}

/// The C#'s `num42..num45` bounds loop (`WorldGen.cs:33789-33817`): the union
/// of every non-empty room rect (all walk rects qualify) — the value the
/// sweeps iterate and the publication inflates by ± 10.
fn temple_bounds(world: &World, rooms: &[TempleRoom]) -> Bounds {
    // `WorldGen.cs:33789-33792`: the C# seeds the four trackers with
    // `maxTilesX - 20` / `20` / `maxTilesY - 20` — a walk that rolls no room
    // (below the vanilla envelope) keeps those inverted defaults, and with
    // rooms the min/max union applies.
    let mut b = Bounds {
        left: world.width - 20,
        top: world.height - 20,
        right: 20,
        bottom: 20,
    };
    for room in rooms {
        if room.rect.w == 0 || room.rect.h == 0 {
            continue;
        }
        b.left = b.left.min(room.rect.x);
        b.right = b.right.max(room.rect.right());
        b.top = b.top.min(room.rect.y);
        b.bottom = b.bottom.max(room.rect.bottom());
    }
    b
}

/// The four `outerTempled` sweeps (`WorldGen.cs:33819-33850`): two x-major
/// (ascending, then descending) and two y-major (ascending, then descending)
/// over the bounds — the order is the C#'s, the conversion site differs per
/// sweep direction.
fn outer_templed_sweeps(world: &mut World, b: Bounds) {
    for num48 in b.left..b.right {
        for num49 in b.top..b.bottom {
            outer_templed(world, num48, num49);
        }
    }
    let mut num50 = b.right;
    while num50 >= b.left {
        for num51 in b.top..b.bottom {
            outer_templed(world, num50, num51);
        }
        num50 -= 1;
    }
    for num52 in b.top..b.bottom {
        for num53 in b.left..b.right {
            outer_templed(world, num53, num52);
        }
    }
    let mut num54 = b.bottom;
    while num54 >= b.top {
        for num55 in b.left..b.right {
            outer_templed(world, num55, num54);
        }
        num54 -= 1;
    }
}

/// `WorldGen.PlaceTile`'s door branch (`WorldGen.cs:59837-59897`) +
/// `WorldGen.PlaceDoor` (`WorldGen.cs:31773-31806`): the anchor check (three
/// clear rows above or below, the solid frame at the far end) selects the
/// branch, then the three-row closed door (tile 10) writes
/// `frameY = 54 * (style % 36) + row * 18` and
/// `frameX = 54 * (style / 36) + Next(3) * 18` (the swing variant). The
/// Lihzahrd door is style 11 (frameY 594/612/630). `false` = refused — no
/// door (the vanilla no-anchor case).
fn place_temple_door(world: &mut World, x: i32, y: i32, style: i32, rng: &mut super::Rng) -> bool {
    let tile_solid_at = |world: &World, x: i32, y: i32| {
        world
            .tile(x, y)
            .is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind))
    };
    let clear = |world: &World, x: i32, y: i32| {
        !world.tile(x, y).is_some_and(|t| t.active())
    };
    // `:59852-59858`: the up branch — j-1/j-2 clear, j-3 solid.
    if clear(world, x, y - 1) && clear(world, x, y - 2) && tile_solid_at(world, x, y - 3) {
        return place_door(world, x, y - 1, style, rng);
    }
    // `:59886-59890`: the down branch — j+1/j+2 clear, j+3 solid.
    if clear(world, x, y + 1) && clear(world, x, y + 2) && tile_solid_at(world, x, y + 3) {
        return place_door(world, x, y + 1, style, rng);
    }
    false
}

/// `WorldGen.PlaceDoor` (`WorldGen.cs:31773-31806`): the three-row door from
/// its TOP row; the anchor is the solid tile two rows above and the solid
/// floor two rows below (`:31790`), each row's `frameX` rolling the swing.
fn place_door(world: &mut World, x: i32, y: i32, style: i32, rng: &mut super::Rng) -> bool {
    let num = style / 36;
    let num2 = style % 36;
    let num3 = 54 * num;
    let num4 = 54 * num2;
    let anchored = world
        .tile(x, y - 2)
        .is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind))
        && solid_tile2_at(world, x, y + 2);
    if !anchored {
        return false;
    }
    for row in 0..3 {
        if !in_world(world, x, y + row, 0) {
            return false;
        }
        let i = world.index(x, y + row);
        let t = &mut world.tiles[i];
        t.kind = 10;
        t.flags |= FLAG_ACTIVE;
        t.frame_y = (num4 + row * 18) as i16;
        t.frame_x = (num3 + rng.range(0, 3) * 18) as i16;
    }
    true
}

/// `makeTemple`'s entrance shaft (`WorldGen.cs:33851-33857`): the
/// horizontal walk from the bury point (opposite the initial direction,
/// every `Next(9, 14)` steps one row up), the band carve of width
/// `± Next(2, 5)`, then the drop column, the brick column, the carved mouth
/// and the Lihzahrd door at the mouth cell.
fn shaft_and_door(
    world: &mut World,
    bury: (i32, i32),
    start_dir: i32,
    rng: &mut super::Rng,
) {
    let (num5, num6) = bury;
    let mut num56 = num6;
    let num3 = -start_dir;
    let mut path = (num5, num6);
    let num57 = rng.range(2, 5);
    let mut num58 = 0;
    let num59 = rng.range(9, 14);
    let mut flag4 = true;
    while flag4 {
        num58 += 1;
        if num58 >= num59 {
            num58 = 0;
            path.1 -= 1;
        }
        path.0 += num3;
        let num60 = path.0;
        flag4 = false;
        for num61 in (path.1 - num57)..(path.1 + num57) {
            let Some(t) = world.tile(num60, num61) else {
                continue;
            };
            if t.wall == WALL_TEMPLE_UNSAFE
                || (t.active() && t.kind == TILE_TEMPLE_BRICK)
            {
                flag4 = true;
            }
            if t.active() && t.kind == TILE_TEMPLE_BRICK {
                if num61 > num56 {
                    num56 = num61;
                }
                carve_cell(world, num60, num61);
            }
        }
    }
    num56 += 2;
    // `:33872-33880`: the drop column down from the bury point.
    let num62 = num5;
    let mut num63 = num6;
    while !world
        .tile(num62, num63)
        .is_some_and(|t| t.active())
    {
        num63 += 1;
        if num63 >= num56 {
            num63 = num56;
            let i = world.index(num62, num63);
            world.tiles[i] = Tile {
                kind: TILE_TEMPLE_BRICK,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
            break;
        }
    }
    num63 -= 4;
    // `:33881-33883`: the first non-temple cell above (the shaft's ceiling).
    let mut num64 = num63;
    let is_temple = |t: &Tile| (t.active() && t.kind == TILE_TEMPLE_BRICK) || t.wall == WALL_TEMPLE_UNSAFE;
    while world.tile(num62, num64).is_some_and(|t| is_temple(t)) {
        num64 -= 1;
    }
    num64 += 2;
    // `:33884-33890`: the brick column.
    for num65 in (num62 - 1)..=(num62 + 1) {
        for num66 in num64..=num63 {
            brick_cell(world, num65, num66);
        }
    }
    // `:33891-33895`: the carved mouth.
    for num67 in (num62 - 4)..=(num62 + 4) {
        for num68 in (num63 - 1)..(num63 + 3) {
            carve_cell(world, num67, num68);
        }
    }
    // `:33896-33900`: the brick frame around the mouth.
    for num69 in (num62 - 1)..=(num62 + 1) {
        for num70 in (num63 - 5)..=(num63 + 8) {
            brick_cell(world, num69, num70);
        }
    }
    // `:33901-33909`: the doorway carve (the door's 3×3 opening).
    for num71 in (num62 - 3)..=(num62 + 3) {
        for num72 in (num63 - 2)..(num63 + 3) {
            if num72 >= num63 || num71 < num5 - 1 || num71 > num5 + 1 {
                carve_cell(world, num71, num72);
            }
        }
    }
    // `:33950`: the door (style 11 — the Lihzahrd material).
    place_temple_door(world, num62, num63, 11, rng);
}

/// `WorldGen.templeCleaner` (`WorldGen.cs:33335-33370`): a brick with ≤ 1
/// brick neighbours falls away (carve), an empty cell with exactly 3 brick
/// neighbours becomes a brick — the shell's edge tidying.
fn temple_cleaner(world: &mut World, x: i32, y: i32) {
    if !in_world(world, x, y, 0) {
        return;
    }
    let mut num = 0;
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        if world
            .tile(x + dx, y + dy)
            .is_some_and(|t| t.active() && t.kind == TILE_TEMPLE_BRICK)
        {
            num += 1;
        }
    }
    let i = world.index(x, y);
    let here = world.tiles[i];
    if here.active() && here.kind == TILE_TEMPLE_BRICK {
        if num <= 1 {
            carve_cell(world, x, y);
        }
    } else if !here.active() && num == 3 {
        brick_cell(world, x, y);
    }
}

/// The two `templeCleaner` sweeps (`WorldGen.cs:33954-33961`): forward over
/// the bounds, then the full reverse.
fn temple_cleaner_sweeps(world: &mut World, b: Bounds) {
    for num77 in b.left..b.right {
        for num78 in b.top..b.bottom {
            temple_cleaner(world, num77, num78);
        }
    }
    let mut num75 = b.bottom;
    while num75 >= b.top {
        let mut num76 = b.right;
        while num76 >= b.left {
            temple_cleaner(world, num76, num75);
            num76 -= 1;
        }
        num75 -= 1;
    }
}

/// The wall-87 widen pass (`WorldGen.cs:33963-33986`): a cell whose whole
/// 3×3 is temple brick or unsafe wall becomes unsafe wall — sealing the
/// shell's inner surface.
fn temple_widen(world: &mut World, b: Bounds) {
    for num77 in b.left..b.right {
        for num78 in b.top..b.bottom {
            let mut flag5 = true;
            'edges: for num79 in (num77 - 1)..=(num77 + 1) {
                for num80 in (num78 - 1)..=(num78 + 1) {
                    let ok = world
                        .tile(num79, num80)
                        .is_some_and(|t| {
                            (t.active() && t.kind == TILE_TEMPLE_BRICK) || t.wall == WALL_TEMPLE_UNSAFE
                        });
                    if !ok {
                        flag5 = false;
                        break 'edges;
                    }
                }
            }
            if flag5 {
                if in_world(world, num77, num78, 0) {
                    let i = world.index(num77, num78);
                    world.tiles[i].wall = WALL_TEMPLE_UNSAFE;
                }
            }
        }
    }
}

/// `WorldGen.Place3x2`'s default path (`WorldGen.cs:52066-52195`) for the
/// altar 237: the bounds guard, the 3×2 body must be empty, the floor solid
/// under all three columns, then the frame write
/// `frameX = 54 * style + col * 18`, `frameY = row * 18`. `false` when the
/// C# would refuse (nothing written).
fn place_altar(world: &mut World, x: i32, y: i32) -> bool {
    if x < 5 || x > world.width - 5 || y < 5 || y > world.height - 5 {
        return false;
    }
    let mut flag2 = true;
    for i in (x - 1)..(x + 2) {
        for j in (y - 1)..(y + 1) {
            if world.tile(i, j).is_some_and(|t| t.active()) {
                flag2 = false;
            }
        }
        if !solid_tile2_at(world, i, y + 1) {
            flag2 = false;
        }
    }
    if !flag2 {
        return false;
    }
    for (dx, frame_x) in [(0, 0), (1, 18), (2, 36)] {
        for (dy, frame_y) in [(0, 0), (1, 18)] {
            brick_tile_at(world, x - 1 + dx, y - 1 + dy, frame_x, frame_y);
        }
    }
    true
}

/// The 3×2 frame write `frameX = num2 + dx * 18`, `frameY = dy * 18`
/// (`WorldGen.cs:52168-52195`) — inactive cells only, the wall and liquid
/// are kept.
fn brick_tile_at(world: &mut World, x: i32, y: i32, frame_x: i16, frame_y: i16) {
    if !in_world(world, x, y, 0) {
        return;
    }
    let i = world.index(x, y);
    if !world.tiles[i].active() {
        let t = &mut world.tiles[i];
        t.kind = TILE_ALTAR;
        t.flags |= FLAG_ACTIVE;
        t.frame_x = frame_x;
        t.frame_y = frame_y;
    }
}

/// The altar placement loop (`WorldGen.cs:33988-34068`): up to 1000 tries at
/// `centre + 15 - Next(30)` per axis in the last room; on success the
/// published anchor is `x - frameX / 18`, `y - frameY / 18` (the 3×2's
/// top-left). The 1000th failure runs the hand-built fallback (the 3-wide
/// brick pad, the cleared 5×3, the 3×2 written directly with
/// `frameX = i * 18`, `frameY = j * 18`).
fn altar_spot(
    world: &mut World,
    rooms: &[TempleRoom],
    rng: &mut super::Rng,
) -> (i32, i32) {
    // The C# reads `array[num2 - 1]` and throws when the walk rolled no room
    // (a world narrower than 420 tiles — below the vanilla envelope and the
    // port's own smallest test world); the port stays total instead.
    let Some(&rectangle3) = rooms.last() else {
        return (0, 0);
    };
    let rectangle3 = rectangle3.rect;
    let num82 = rectangle3.w / 2;
    let num83 = rectangle3.h / 2;
    let mut num81 = 0;
    loop {
        num81 += 1;
        let num84 = rectangle3.x + num82 + 15 - rng.range(0, 30);
        let num85 = rectangle3.y + num83 + 15 - rng.range(0, 30);
        place_altar(world, num84, num85);
        if world
            .tile(num84, num85)
            .is_some_and(|t| t.kind == TILE_ALTAR && t.active())
        {
            let t = world.tile(num84, num85).unwrap();
            let anchor = (num84 - (t.frame_x as i32) / 18, num85 - (t.frame_y as i32) / 18);
            return anchor;
        }
        if num81 < 1000 {
            continue;
        }
        // The fallback (`:34014-34068`).
        let mut num84 = rectangle3.x + num82;
        let mut num85 = rectangle3.y + num83;
        num84 += rng.range(-10, 11);
        num85 += rng.range(-10, 11);
        while num85 < rectangle3.bottom() - 2
            && world.tile(num84, num85).is_some_and(|t| t.active())
        {
            num85 += 1;
        }
        // The port's edge guard: the C# walks until a tile answers (it throws
        // past the world's bottom); the walk stops at the world edge.
        while num85 < world.height - 1 && !world.tile(num84, num85).is_some_and(|t| t.active()) {
            num85 += 1;
        }
        for dx in -1..=1 {
            let px = num84 + dx;
            if in_world(world, px, num85, 0) {
                let i = world.index(px, num85);
                let t = &mut world.tiles[i];
                t.flags |= FLAG_ACTIVE;
                t.set_slope(0);
                t.flags &= !crate::tile::FLAG_HALF_BRICK;
                t.kind = TILE_TEMPLE_BRICK;
            }
        }
        num85 -= 2;
        num84 -= 1;
        for num86 in -1..=3 {
            for num87 in -1..=1 {
                carve_cell(world, num84 + num86, num85 + num87);
            }
        }
        for num88 in 0..=2 {
            for num89 in 0..=1 {
                let px = num84 + num88;
                let py = num85 + num89;
                if !in_world(world, px, py, 0) {
                    continue;
                }
                let i = world.index(px, py);
                let t = &mut world.tiles[i];
                t.flags |= FLAG_ACTIVE;
                t.kind = TILE_ALTAR;
                t.frame_x = (num88 * 18) as i16;
                t.frame_y = (num89 * 18) as i16;
            }
        }
        for num88 in 0..=2 {
            for num89 in 0..=1 {
                square_frame_altar(world, num84 + num88, num85 + num89);
            }
        }
        return (num84, num85);
    }
}

/// The C#'s six `SquareTileFrame` calls over the altar (`WorldGen.cs:34067-34068`
/// and the `LihzahrdAltar` pass `:21969-21974`) — the framer keeps the
/// stored 237 frames (frame-important, `Main.cs:10234`).
fn square_frame_altar(_world: &mut World, _x: i32, _y: i32) {
    // The altar 237 is a plain 3×2 frame tile: `TileFrame` finds the full
    // object already framed and the stored frames stand; the call is a
    // no-op on the port's tile storage (the next phase wires the shared
    // `edit::square_tile_frame` seam).
}

/// The spike loop (`WorldGen.cs:34069-34217`): `num2 * 1.1 * (1 +
/// Next(-25, 26) * 0.01)` wooden spikes (232), placed on the room walls in
/// `Next(3, 10)`-wide staircase runs — the vertical branch walks down to the
/// floor, the horizontal one sideways, each run alternating the neighbour
/// column and requiring solid neighbours on both sides.
fn temple_spikes(world: &mut World, rooms: &[TempleRoom], b: Bounds, rng: &mut super::Rng) {
    let num2 = rooms.len() as f64;
    let mut num92 = num2 * 1.1 * (1.0 + (rng.range(-25, 26) as f64) * 0.01);
    let mut num94 = 0;
    while num92 > 0.0 {
        num94 += 1;
        let num95 = rng.range(0, rooms.len() as i32) as usize;
        let r = rooms[num95].rect;
        let num96 = rng.range(r.x, r.right());
        let num97 = rng.range(r.y, r.bottom());
        if world
            .tile(num96, num97)
            .is_some_and(|t| t.wall == WALL_TEMPLE_UNSAFE && !t.active())
        {
            let mut flag6 = false;
            if rng.range(0, 2) == 0 {
                // The vertical spike staircase.
                let num98 = if rng.range(0, 2) == 0 { 1 } else { -1 };
                let mut y = num97;
                while !world.tile(num96, y).is_some_and(|t| t.active()) {
                    y += num98;
                }
                y -= num98;
                let num99 = rng.range(0, 2);
                let num100 = rng.range(3, 10);
                let mut flag7 = true;
                for num101 in (num96 - num100)..(num96 + num100) {
                    for num102 in (y - num100)..(y + num100) {
                        if world
                            .tile(num101, num102)
                            .is_some_and(|t| t.active() && (t.kind == 10 || t.kind == TILE_ALTAR))
                        {
                            flag7 = false;
                            break;
                        }
                    }
                    if !flag7 {
                        break;
                    }
                }
                if flag7 {
                    let mut flip = num99;
                    for num103 in (num96 - num100)..(num96 + num100) {
                        for num104 in (y - num100)..(y + num100) {
                            let here = world.tile(num103, num104).copied();
                            let above = world.tile(num103, num104 - num98).copied();
                            let (Some(here), Some(above)) = (here, above) else {
                                continue;
                            };
                            if !solid_tile_at(world, num103, num104)
                                || here.kind == 232
                                || solid_tile_at(world, num103, num104 - num98)
                            {
                                continue;
                            }
                            spike_cell(world, num103, num104);
                            flag6 = true;
                            if flip == 0 {
                                spike_cell(world, num103, num104 - 1);
                            } else {
                                spike_cell(world, num103, num104 + 1);
                            }
                            flip += 1;
                            if flip > 1 {
                                flip = 0;
                            }
                        }
                    }
                }
            } else {
                // The horizontal spike staircase.
                let num105 = if rng.range(0, 2) == 0 { 1 } else { -1 };
                let mut x = num96;
                while !world.tile(x, num97).is_some_and(|t| t.active()) {
                    x += num105;
                }
                x -= num105;
                let num106 = rng.range(0, 2);
                let num107 = rng.range(3, 10);
                let mut flag8 = true;
                for num108 in (x - num107)..(x + num107) {
                    for num109 in (num97 - num107)..(num97 + num107) {
                        if world
                            .tile(num108, num109)
                            .is_some_and(|t| t.active() && t.kind == 10)
                        {
                            flag8 = false;
                            break;
                        }
                    }
                    if !flag8 {
                        break;
                    }
                }
                if flag8 {
                    let mut flip = num106;
                    for num110 in (x - num107)..(x + num107) {
                        for num111 in (num97 - num107)..(num97 + num107) {
                            let here = world.tile(num110, num111).copied();
                            let beside = world.tile(num110 - num105, num111).copied();
                            let (Some(here), Some(beside)) = (here, beside) else {
                                continue;
                            };
                            if !solid_tile_at(world, num110, num111)
                                || here.kind == 232
                                || solid_tile_at(world, num110 - num105, num111)
                            {
                                continue;
                            }
                            spike_cell(world, num110, num111);
                            flag6 = true;
                            if flip == 0 {
                                spike_cell(world, num110 - 1, num111);
                            } else {
                                spike_cell(world, num110 + 1, num111);
                            }
                            flip += 1;
                            if flip > 1 {
                                flip = 0;
                            }
                        }
                    }
                }
            }
            if flag6 {
                num94 = 0;
                num92 -= 1.0;
            }
        }
        if num94 > 1000 {
            num94 = 0;
            num92 -= 1.0;
        }
    }
    let _ = b;
}

/// The spike write — `type = 232` on the floor tile plus the alternating
/// neighbour (`WorldGen.cs:34108-34119` / `:34191-34204`); the drunk-world
/// second row is seed scope.
fn spike_cell(world: &mut World, x: i32, y: i32) {
    if !in_world(world, x, y, 0) {
        return;
    }
    let i = world.index(x, y);
    let t = &mut world.tiles[i];
    t.kind = 232;
    t.flags |= FLAG_ACTIVE;
}

/// The `LihzahrdTemple` pass (`WorldGen.cs:15834+` → `makeTemple(x, y)`):
/// the temple box in the jungle cavern below the rock layer — the outer
/// brick shell (the walls 87), the carved room corridors (the walls 112),
/// the altar room with the 3×2 altar, the dart traps (137) embedded in the
/// walls, the chests in the rooms. The bury point comes from the faithful
/// search ([`find_temple_site`]); the published [`TempleBounds`] is the
/// M5-TEMPLE seam (see the module docs for the verbatim/shape-level split).
pub(crate) fn pass_lihzahrd_temple(
    world: &mut World,
    dungeon_x: i32,
    rock_layer: f64,
    rng: &mut super::Rng,
) -> TempleBounds {
    // `WorldGen.cs:15834-15930`: the placement search (or its fallback).
    let (cx, cy) = find_temple_site(world, dungeon_x, rock_layer, rng);
    // `makeTemple` (`WorldGen.cs:33435-34237`), the faithful statement order:
    // the room walk, the brick generation, the jitter carve, the corridors,
    // the outerTempled sweeps, the entrance shaft + door, the cleaner sweeps,
    // the widen, the altar, the spikes — then the GenVars publication.
    let walk = make_temple_rooms(
        rng,
        world.width as f64 / 4200.0,
        world.height - 200,
        cx,
        cy,
    );
    make_temple_generate_bricks(world, &walk.rooms);
    carve_room_interiors(world, &walk.rooms, rng);
    temple_corridors(world, &walk, (cx, cy), rng);
    // `:33788-33817`: the room-union bounds (inflated ± 10 at publication).
    let bounds = temple_bounds(world, &walk.rooms);
    outer_templed_sweeps(world, bounds);
    shaft_and_door(world, (cx, cy), walk.start_dir, rng);
    temple_cleaner_sweeps(world, bounds);
    temple_widen(world, bounds);
    let (l_altar_x, l_altar_y) = altar_spot(world, &walk.rooms, rng);
    temple_spikes(world, &walk.rooms, bounds, rng);
    // `:34231-34235`: the publication — the filled extent ± 10, the final
    // walk count and the altar anchor.
    TempleBounds {
        t_left: bounds.left - 10,
        t_right: bounds.right + 10,
        t_top: bounds.top - 10,
        t_bottom: bounds.bottom + 10,
        t_rooms: walk.rooms.len() as i32,
        l_altar_x,
        l_altar_y,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::noise::Rng;

    /// An active solid-kind tile patch — the synthetic jungle-grass stand-in.
    fn paint(world: &mut World, x0: i32, x1: i32, y0: i32, y1: i32, kind: u16) {
        for x in x0..x1 {
            for y in y0..y1 {
                let i = world.index(x, y);
                world.tiles[i] = Tile {
                    kind,
                    flags: FLAG_ACTIVE,
                    ..Default::default()
                };
            }
        }
    }

    /// DW-1.1: the room walk on the vanilla large world (scale 1.0 →
    /// `Next(10, 16)` rooms): the room count, the draw ledger (the stream
    /// state — any draw order change shifts it) and the first/last room's
    /// geometry (`WorldGen.cs:33440-33548`). The last room is the resized
    /// altar room (`(int)(Next(55,65) * 1.6)` × `(int)(Next(45,50) * 1.35)`),
    /// the last depth level the C# publishes as the walk's room count.
    #[test]
    fn test_DW_1_1_room_walk_draw_ledger() {
        let mut rng = Rng(0xDEAD_BEEF);
        let walk = make_temple_rooms(&mut rng, 1.0, 1000, 2000, 300);
        let rooms = &walk.rooms;
        assert_eq!(rooms.len(), 15, "the walk's room count");
        assert_eq!(rng.0, 0xcd25_41e9_adf9_5601, "the draw ledger");
        let first = rooms[0].rect;
        assert_eq!(
            (first.x, first.y, first.w, first.h),
            (2014, 284, 43, 24),
            "the first room"
        );
        assert_eq!(rooms[0].depth, 0);
        let last = rooms[rooms.len() - 1].rect;
        assert_eq!(
            (last.x, last.y, last.w, last.h),
            (1620, 422, 97, 63),
            "the resized altar room"
        );
        assert_eq!(rooms[rooms.len() - 1].depth, 3);
        // The room shapes stay inside the C#'s rolls and never intersect, and
        // the depth level only grows (the `num13` flip ladder).
        for (i, room) in rooms.iter().enumerate() {
            let (w, h) = (room.rect.w, room.rect.h);
            if i + 1 == rooms.len() {
                assert!((88..=104).contains(&w) && (60..=67).contains(&h));
            } else {
                assert!((25..50).contains(&w) && (20..35).contains(&h) || w == h);
                assert!(h <= w, "the `num18 = min(num18, num17)` cap");
            }
            for earlier in &rooms[..i] {
                assert!(
                    !room.rect.intersects(&earlier.rect),
                    "room {i} intersects an earlier room"
                );
            }
            if i > 0 {
                assert!(room.depth >= rooms[i - 1].depth, "the depth ladder");
            }
        }
    }

    /// DW-1.1: the last-room resize also fires on the underworld ceiling
    /// (`flag2 = num16 + 70 >= UnderworldLayer - 10`, `WorldGen.cs:33472`):
    /// with the bury point exactly on the boundary the walk collapses to the
    /// single altar room and consumes no intersection draws (the flag2
    /// break skips the retry loop, `:33512`).
    #[test]
    fn test_DW_1_1_underworld_ceiling_collapses_the_walk() {
        // At (2000, 220) with layer 300: 220 + 70 = 290 >= 290 → flag2.
        let mut rng = Rng(0xDEAD_BEEF);
        let walk = make_temple_rooms(&mut rng, 1.0, 300, 2000, 220);
        assert_eq!(walk.rooms.len(), 1, "the walk collapses to one room");
        let room = walk.rooms[0].rect;
        assert!((88..=104).contains(&room.w) && (60..=67).contains(&room.h));
        // The draw ledger: the count draw, the direction, the flip period,
        // the two resize rolls, the Next(5,10) and the two stagger rolls —
        // no `Next(100)` intersection draws (the ledger is pinned by the
        // state; a retry-loop entry would add two more draws).
        let mut fresh = Rng(0xDEAD_BEEF);
        for _ in 0..10 {
            fresh.next_u32();
        }
        assert_eq!(rng.0, fresh.0, "10 draws, no intersection retry");
    }

    /// DW-1.2: the depth unions + `Inflate(10)` brick the exact C# shape
    /// (`makeTemple_GenerateBricks` `WorldGen.cs:34238-34276`): one union box
    /// per depth level that occurs, the null levels of the C#'s sparse
    /// `DungeonBounds` array skipped — a depth value with no rooms fills
    /// nothing and adds no lines.
    #[test]
    fn test_DW_1_2_depth_union_and_fill() {
        let mut world = World::new("t".into(), 300, 300);
        let rooms = vec![
            TempleRoom {
                rect: Rect { x: 100, y: 100, w: 40, h: 30 },
                depth: 0,
            },
            TempleRoom {
                rect: Rect { x: 130, y: 110, w: 20, h: 20 },
                depth: 0,
            },
            TempleRoom {
                rect: Rect { x: 200, y: 150, w: 10, h: 10 },
                depth: 2, // the level 1 stays empty
            },
        ];
        make_temple_generate_bricks(&mut world, &rooms);
        let brick = |world: &World, x: i32, y: i32| {
            world.tile(x, y).is_some_and(|t| t.active() && t.kind == 226)
        };
        // The level-0 union (100..150 × 100..130) inflated to (90..160 ×
        // 90..140): inside filled, the cells just outside not.
        assert!(brick(&world, 95, 95) && brick(&world, 90, 90) && brick(&world, 160, 140));
        assert!(!brick(&world, 89, 90) && !brick(&world, 90, 89) && !brick(&world, 90, 141));
        // The level-2 union (200..210 × 150..160) inflated to (190..220 ×
        // 140..170).
        assert!(brick(&world, 190, 140) && brick(&world, 220, 170));
        assert!(!brick(&world, 189, 140) && !brick(&world, 220, 171));
        // The empty level 1 fills nothing where no room union reaches, and
        // the depth-0↔2 corner lines bridge the two levels.
        assert!(!brick(&world, 40, 40) && !brick(&world, 260, 260));
        // The brick write clears the wall (`ClearEverything`, :34271-34274).
        let t = world.tile(95, 95).unwrap();
        assert_eq!(t.wall, 0);
    }

    /// DW-1.3: `_Line` (`WorldGen.cs:34341-34380`) bricks the straight walk
    /// and nothing off it.
    #[test]
    fn test_DW_1_3_line_straight() {
        let mut world = World::new("t".into(), 300, 300);
        generate_bricks_line(&mut world, 10, 10, 30, 10, false, false);
        for x in 10..=29 {
            let t = world.tile(x, 10).unwrap();
            assert!(t.active() && t.kind == 226, "the brick at ({x}, 10)");
        }
        // The walk is [start, end) — the `num -= unit step` loop stops before
        // the end cell (`WorldGen.cs:34371-34377`).
        assert!(!world.tile(30, 10).unwrap().active());
        assert!(!world.tile(20, 9).unwrap().active());
    }

    /// DW-1.3: `_Line_StrictAngles` (`WorldGen.cs:34301-34339`) writes the
    /// L: the axis-aligned leg to the corner, then the diagonal leg; the
    /// unused corner-adjacent cells stay untouched.
    #[test]
    fn test_DW_1_3_line_strict_angles_corner() {
        let mut world = World::new("t".into(), 300, 300);
        // start (10,10) → end (30,25), fill_left/fill_upwards false: the
        // origin is (10,25); the end is farther, so the walk starts there and
        // stops on the start's y-axis → the corner (15,10).
        generate_bricks_line_strict_angles(&mut world, 10, 10, 30, 25, false, false);
        let brick = |world: &World, x: i32, y: i32| {
            world.tile(x, y).is_some_and(|t| t.active() && t.kind == 226)
        };
        for x in 10..=15 {
            assert!(brick(&world, x, 10), "the horizontal leg at ({x}, 10)");
        }
        assert!(brick(&world, 30, 25), "the diagonal leg reaches the end");
        assert!(!brick(&world, 20, 10), "the corner branch is not a straight line");
        assert!(!brick(&world, 15, 15));
    }

    /// DW-1.3: the StrictAngles 500-step cap falls back to the straight line
    /// (`WorldGen.cs:34326-34328`) — the long diagonal still bricks.
    #[test]
    fn test_DW_1_3_line_strict_angles_cap_falls_back_to_straight() {
        let mut world = World::new("t".into(), 3000, 3000);
        // The corner walk would need ~1950 steps (> 500).
        generate_bricks_line_strict_angles(&mut world, 100, 100, 2000, 2050, false, false);
        let brick = |world: &World, x: i32, y: i32| {
            world.tile(x, y).is_some_and(|t| t.active() && t.kind == 226)
        };
        assert!(brick(&world, 101, 101));
        assert!(!brick(&world, 2000, 2050) && !brick(&world, 1500, 1600), "the cap cut the line");
    }

    /// DW-1.3: `_Line`'s 2000-step cap (`WorldGen.cs:34364-34368`) stops the
    /// walk; the cells past the cap stay untouched.
    #[test]
    fn test_DW_1_3_line_cap_aborts() {
        let mut world = World::new("t".into(), 3000, 3000);
        // A ~4242-step diagonal: the cap stops at write 1999 (≈ (1423,
        // 1423)); (1500, 1500) is past it.
        generate_bricks_line(&mut world, 10, 10, 2900, 2900, false, false);
        let brick = |world: &World, x: i32, y: i32| {
            world.tile(x, y).is_some_and(|t| t.active() && t.kind == 226)
        };
        assert!(brick(&world, 1423, 1423), "the last capped write");
        assert!(!brick(&world, 1500, 1500), "past the cap");
        assert!(!brick(&world, 2900, 2900), "the line never reaches the end");
    }

    /// DW-1.3: `_Line_FillY` (`WorldGen.cs:34382-34421`) closes a vertical
    /// gap down to the next brick and stops there.
    #[test]
    fn test_DW_1_3_fill_y_closes_the_gap() {
        let mut world = World::new("t".into(), 300, 300);
        // The pre-existing brick the scan stops at.
        let i = world.index(50, 60);
        world.tiles[i].kind = 226;
        world.tiles[i].flags |= FLAG_ACTIVE;
        generate_bricks_line_fill_y(&mut world, false, -1, 50, 20);
        for y in 21..60 {
            let t = world.tile(50, y).unwrap();
            assert!(t.active() && t.kind == 226, "the gap fill at (50, {y})");
        }
        assert!(!world.tile(50, 61).unwrap().active(), "the fill stops at the brick");
        assert!(!world.tile(50, 20).unwrap().active(), "the anchor cell is not filled");
    }

    /// DW-1.3: `_Line_FillX` (`WorldGen.cs:34423-34462`) closes a horizontal
    /// gap to the right, up to the next brick.
    #[test]
    fn test_DW_1_3_fill_x_closes_the_gap() {
        let mut world = World::new("t".into(), 300, 300);
        let i = world.index(70, 50);
        world.tiles[i].kind = 226;
        world.tiles[i].flags |= FLAG_ACTIVE;
        generate_bricks_line_fill_x(&mut world, false, -1, 30, 50);
        for x in 31..70 {
            let t = world.tile(x, 50).unwrap();
            assert!(t.active() && t.kind == 226, "the gap fill at ({x}, 50)");
        }
        assert!(!world.tile(71, 50).unwrap().active(), "the fill stops at the brick");
        assert!(!world.tile(30, 50).unwrap().active(), "the anchor cell is not filled");
    }

    /// DW-1.3: the FillY scan guards — the 300-step cap and the world edge
    /// leave the cell untouched (`WorldGen.cs:34408-34410`), so a gap with no
    /// brick at the end fills nothing.
    #[test]
    fn test_DW_1_3_fill_y_guards_abort_without_writes() {
        let mut world = World::new("t".into(), 300, 300);
        generate_bricks_line_fill_y(&mut world, false, -1, 50, 20);
        for y in 21..300 {
            assert!(
                !world.tile(50, y).unwrap().active() && world.tile(50, y).unwrap().kind == 0,
                "the scan burned its 300 steps and wrote nothing at y={y}"
            );
        }
        // The edge guard: an anchor close to the bottom aborts too.
        let mut near = World::new("t".into(), 300, 300);
        generate_bricks_line_fill_y(&mut near, true, -1, 50, 280);
        for y in 0..280 {
            assert!(!near.tile(50, y).unwrap().active(), "the edge guard at y={y}");
        }
    }

    /// DW-1.4: the underworld-ceiling test uses the value the caller passes
    /// — `UnderworldLayer = maxTilesY - 200` (`Main.cs:2790`) is computed by
    /// the step, and the boundary is exact: at the layer the walk collapses,
    /// one row lower it does not.
    #[test]
    fn test_DW_1_4_last_room_trigger_uses_depth_minus_200() {
        let mut collapsed = Rng(0xDEAD_BEEF);
        let walk = make_temple_rooms(&mut collapsed, 1.0, 300, 2000, 220);
        assert_eq!(walk.rooms.len(), 1, "220 + 70 >= 300 - 10");
        let mut normal = Rng(0xDEAD_BEEF);
        let walk = make_temple_rooms(&mut normal, 1.0, 301, 2000, 220);
        assert!(walk.rooms.len() > 1, "220 + 70 < 301 - 10");
    }

    /// DW-6.1: the acceptance semantics — with the whole vanilla y band
    /// painted as jungle grass (60), the first sample is accepted: the bury
    /// point sits on active tile 60, inside the C# x reach for `num3 = 0.25`
    /// (`(0.15·maxTilesX, 0.4·maxTilesX]` — `dungeonSide` fixed Right) and
    /// the y band `[(int)rockLayer, maxTilesY - 600)`. The draw ledger: the
    /// search consumed exactly 2 draws (the y `Next` then the x
    /// `NextDouble`), so the stream state equals a fresh rng after 2 draws.
    #[test]
    fn test_DW_6_1_search_accepts_jungle_grass() {
        let mut world = World::new("t".into(), 800, 800);
        // The band rows (50..200) painted slightly past the x reach edges
        // (118..322 covers [120, 320]) so the first sample always hits.
        paint(&mut world, 118, 322, 48, 202, 60);
        let mut rng = Rng(0xDEAD_BEEF);
        let site = find_temple_site(&world, 300, 50.0, &mut rng);
        assert!(site.0 >= 120 && site.0 <= 320, "x reach ({})", site.0);
        assert!(site.1 >= 50 && site.1 < 200, "y band ({})", site.1);
        let t = world.tile(site.0, site.1).unwrap();
        assert!(t.active() && t.kind == 60, "the bury point is jungle grass");
        // The draw ledger: `Next(num4, num5)` then `NextDouble()`.
        let mut fresh = Rng(0xDEAD_BEEF);
        fresh.next_u32();
        fresh.next_u32();
        assert_eq!(rng.0, fresh.0, "the search draws exactly y then x-fraction");
    }

    /// DW-6.1: the widening valve + the tile-60-only acceptance. Jungle grass
    /// exists only in the `num3 = 0.35`-exclusive reach `[0.05·W, 0.10·W)`
    /// (at 0.25 the reach starts past 0.15·W, at 0.30 past 0.10·W); every
    /// earlier sample lands on mud (59) and must not accept. The site can
    /// therefore only come from the two 1,000,000-attempt widening stages
    /// (0.25 → 0.30 → 0.35) — the C# counter semantics
    /// (`WorldGen.cs:15900-15912`, `Math.Min` landing `num3` exactly on 0.35).
    #[test]
    fn test_DW_6_1_search_widens_and_accepts_only_tile_60() {
        let mut world = World::new("t".into(), 800, 800);
        // Mud across the whole reach, then grass only in the widened reach.
        paint(&mut world, 40, 322, 48, 202, 59);
        paint(&mut world, 40, 80, 48, 202, 60);
        let mut rng = Rng(42);
        let site = find_temple_site(&world, 300, 50.0, &mut rng);
        assert!(site.0 > 40 && site.0 < 80, "the widened-reach x ({})", site.0);
        assert!(site.1 >= 50 && site.1 < 200, "y band ({})", site.1);
        let t = world.tile(site.0, site.1).unwrap();
        assert!(t.active() && t.kind == 60, "the bury point is jungle grass");
    }

    /// DW-6.1: the sub-vanilla valve — on `maxTilesY - 600 <= 0` the vanilla
    /// y band cannot exist in-world, so the search goes straight to the C#
    /// fallback (`WorldGen.cs:15914-15923`): `x = maxTilesX -
    /// generatingDungeonPositionX`, `y = (int)rockLayer + 100`, consuming no
    /// draws at all.
    #[test]
    fn test_DW_6_1_degenerate_band_goes_to_the_fallback() {
        let world = World::new("t".into(), 500, 500);
        let mut rng = Rng(999);
        let site = find_temple_site(&world, 300, 50.0, &mut rng);
        assert_eq!(site, (500 - 300, 50 + 100), "the fallback position");
        assert_eq!(rng.0, 999, "the valve skips the loop without drawing");
    }

    /// DW-7.1 (the batch-5-6 review note): the 11-stage give-up branch
    /// (`WorldGen.cs:15900-15912`) — a world with no jungle grass anywhere in
    /// the sampled band never accepts: the search widens 0.25 → 0.30 → 0.35,
    /// then runs 11 more 1,000,000-attempt stages at the cap (`num > 10`)
    /// and gives up to the C# fallback (`:15914-15923`). The draw ledger pins
    /// the full counter ladder — 13 widen-stage runs × 1,000,002 attempts ×
    /// 2 draws; a break one stage early would be 2,000,004 draws short.
    #[test]
    fn test_DW_7_1_temple_search_gives_up_after_eleven_cap_stages() {
        let world = World::new("t".into(), 800, 800); // no tile 60 anywhere
        let mut rng = Rng(0x5EED_5EED);
        let site = find_temple_site(&world, 300, 50.0, &mut rng);
        assert_eq!(site, (800 - 300, 50 + 100), "the C# fallback position");
        // The exact attempt ladder: two stages to reach the 0.35 cap + the
        // eleven cap stages, every stage 1,000,002 attempts (the
        // post-increment compare), 2 draws per attempt (the y `Next` + the
        // x `NextDouble`).
        let mut fresh = Rng(0x5EED_5EED);
        for _ in 0..13 * 1_000_002 {
            fresh.next_u32();
            fresh.next_u32();
        }
        assert_eq!(rng.0, fresh.0, "the full 11-stage give-up ladder consumed");
    }

    /// DW-6.1 → DW-2.2: the pass now publishes the FAITHFUL bound — the
    /// room-union ± 10 of the walk (`WorldGen.cs:33788-33817`, published
    /// `:34231-34235`) instead of the stub box, `t_rooms` is the final walk
    /// count and the altar anchor is the 3×2's top-left (`:33999-34001`).
    /// The 512-tall world has no vanilla y band (`maxTilesY - 600 <= 0`), so
    /// the search goes to the fallback (`WorldGen.cs:15926-15930`: x =
    /// maxTilesX - dungeon_x, y = rockLayer + 100) and the walk rolls a
    /// single resized room (the sub-vanilla scale collapses `Next(10, 16)`
    /// to one room).
    #[test]
    fn test_DW_2_2_step_publishes_faithful_bounds_and_altar_anchor() {
        let mut world = World::new("t".into(), 512, 512);
        let mut rng = Rng(7);
        let bounds = pass_lihzahrd_temple(&mut world, 300, 50.0, &mut rng);
        let (bx, by) = (212, 150); // the deterministic fallback bury point
        assert_eq!(bounds.t_rooms, 1, "t_rooms — the sub-vanilla one-room walk");
        // The single resized room (95 × 63) ± 10.
        assert_eq!(
            (
                bounds.t_left,
                bounds.t_right,
                bounds.t_top,
                bounds.t_bottom
            ),
            (240, 354, 113, 195),
            "the published bounds {:?}",
            bounds
        );
        let _ = (bx, by);
        assert!(
            bounds.t_left <= bounds.l_altar_x
                && bounds.l_altar_x + 2 < bounds.t_right
                && bounds.t_top <= bounds.l_altar_y
                && bounds.l_altar_y + 1 < bounds.t_bottom,
            "the anchor sits inside the bounds ({},{})",
            bounds.l_altar_x,
            bounds.l_altar_y
        );
        // The 3×2 altar with the faithful frames (frameX = i·18, frameY =
        // j·18 — `Place3x2` writes the same layout style 0).
        for i in 0..3i32 {
            for j in 0..2i32 {
                let t = world
                    .tile(bounds.l_altar_x + i, bounds.l_altar_y + j)
                    .unwrap();
                assert!(
                    t.active()
                        && t.kind == TILE_ALTAR
                        && t.frame_x == (i * 18) as i16
                        && t.frame_y == (j * 18) as i16,
                    "the altar tile ({i},{j}) kind {} frames ({},{})",
                    t.kind,
                    t.frame_x,
                    t.frame_y
                );
            }
        }
    }

    /// DW-2.1: the full pass on the fallback world writes the temple family —
    /// the Lihzahrd brick (226), the unsafe wall (87, the only wall the C#
    /// writes — `templeCleaner`/corridors/carve), the 3×2 altar (237), the
    /// Lihzahrd door (10, three rows) and the wooden spikes (232). The
    /// faithful C# writes NO wall 112 (the "safe" constant is only read
    /// back, `WorldGen.cs:61300`).
    #[test]
    fn test_DW_2_1_full_temple_family_on_the_fallback_world() {
        let mut world = World::new("t".into(), 512, 512);
        let mut rng = Rng(7);
        let bounds = pass_lihzahrd_temple(&mut world, 300, 50.0, &mut rng);
        let count = |kind: u16| {
            world
                .tiles
                .iter()
                .filter(|t| t.active() && t.kind == kind)
                .count()
        };
        let walls = |wall: u16| world.tiles.iter().filter(|t| t.wall == wall).count();
        assert!(count(226) > 800, "the temple brick ({})", count(226));
        assert_eq!(count(237), 6, "the altar tiles ({})", count(237));
        assert!(walls(87) > 500, "the unsafe temple walls ({})", walls(87));
        assert_eq!(walls(112), 0, "the faithful C# writes no safe wall");
        assert!(count(232) >= 5, "the spikes ({})", count(232));
        // The door: three rows of tile 10 with the Lihzahrd style (frameY
        // 594/612/630 = 54·11 + row·18, frameX the swing variant).
        let door: Vec<_> = world
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| t.active() && t.kind == 10)
            .map(|(i, t)| (i, *t))
            .collect();
        assert_eq!(door.len(), 3, "the door tiles ({})", door.len());
        for (_, t) in &door {
            assert!(
                (594..=630).contains(&t.frame_y) && (t.frame_y - 594) % 18 == 0,
                "the Lihzahrd door frameY ({})",
                t.frame_y
            );
            assert!((0..=36).contains(&t.frame_x));
        }
        // The bounds contain the whole family.
        assert!(bounds.t_left > 0 && bounds.t_right < world.width);
    }

    /// DW-2.3: the door branch — with the up-anchor solid the door writes;
    /// without any anchor it refuses (the vanilla no-door case, no tiles
    /// written, no rng consumed).
    #[test]
    fn test_DW_2_3_door_refuses_without_an_anchor() {
        let mut world = World::new("t".into(), 64, 64);
        let mut rng = Rng(11);
        // The solid frame at rows 17-19 and the mouth floor at row 23: the
        // up branch reads j-3 = 19 solid, j-2 = 20 and j-1 = 21 clear, and
        // `PlaceDoor`'s own check wants the floor at j+1 = 23.
        for x in 30..34 {
            for y in [17, 18, 19, 23] {
                let i = world.index(x, y);
                world.tiles[i] = Tile {
                    kind: 1,
                    flags: FLAG_ACTIVE,
                    ..Default::default()
                };
            }
        }
        assert!(place_temple_door(&mut world, 32, 22, 11, &mut rng));
        let door = world
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == 10)
            .count();
        assert_eq!(door, 3);
        // The refusal: a flat empty world has no anchor.
        let mut empty = World::new("t".into(), 64, 64);
        let mut rng = Rng(11);
        let before = rng.0;
        assert!(!place_temple_door(&mut empty, 32, 22, 11, &mut rng));
        assert_eq!(rng.0, before, "no draws on refusal");
    }

    /// DW-2.4: determinism — the same seed twice gives identical worlds.
    #[test]
    fn test_DW_2_4_temple_is_deterministic() {
        let run = || {
            let mut world = World::new("t".into(), 512, 512);
            let mut rng = Rng(7);
            let b = pass_lihzahrd_temple(&mut world, 300, 50.0, &mut rng);
            (b, world.tiles.clone())
        };
        let (a, tiles_a) = run();
        let (b, tiles_b) = run();
        assert_eq!(a, b, "the published bounds");
        assert!(tiles_a == tiles_b, "the tile-for-tile worlds");
    }

    /// DW-2.5: the altar's 1000-try cap — a last room whose 3×2 body can
    /// never be empty forces the hand-built fallback (`WorldGen.cs:34014-34068`):
    /// the brick pad, the cleared 5×3 and the 3×2 written directly with
    /// `frameX = i·18`, `frameY = j·18`.
    #[test]
    fn test_DW_2_5_altar_falls_back_after_1000_tries() {
        let mut world = World::new("t".into(), 300, 300);
        // The room fully bricked: every Place3x2 attempt finds an occupied
        // body and refuses.
        for x in 100..200 {
            for y in 100..160 {
                let i = world.index(x, y);
                world.tiles[i] = Tile {
                    kind: TILE_TEMPLE_BRICK,
                    flags: FLAG_ACTIVE,
                    ..Default::default()
                };
            }
        }
        let rooms = vec![TempleRoom {
            rect: Rect {
                x: 100,
                y: 100,
                w: 100,
                h: 60,
            },
            depth: 0,
        }];
        let mut rng = Rng(3);
        let anchor = altar_spot(&mut world, &rooms, &mut rng);
        // The fallback's 3×2 with the direct frames.
        for i in 0..3i32 {
            for j in 0..2i32 {
                let t = world
                    .tile(anchor.0 + i, anchor.1 + j)
                    .expect("the fallback altar tile");
                assert!(
                    t.active()
                        && t.kind == TILE_ALTAR
                        && t.frame_x == (i * 18) as i16
                        && t.frame_y == (j * 18) as i16,
                    "the fallback altar ({i},{j}) frames ({},{})",
                    t.frame_x,
                    t.frame_y
                );
            }
        }
    }

    /// DW-2.5: the sub-vanilla near-bound guard — a small world whose walk
    /// runs the bounds to the edge keeps every temple write inside the world
    /// (the port's `in_world` guards stand where the C# relies on the jungle
    /// cavern's margin).
    #[test]
    fn test_DW_2_5_sub_vanilla_bounds_stay_in_world() {
        let mut world = World::new("t".into(), 512, 512);
        // Jungle grass everywhere so the search accepts the first sample.
        paint(&mut world, 0, 512, 0, 512, 60);
        let mut rng = Rng(5);
        // The fallback bury point mirrors the dungeon entrance (512 - 10,
        // 150): the walk's room extents run past the world's east edge —
        // the C# carries the bounds past the world and writes through them
        // unguarded (it would throw on the same world); the port clamps
        // every write with `in_world`.
        let bounds = pass_lihzahrd_temple(&mut world, 10, 50.0, &mut rng);
        assert_eq!(bounds.t_rooms, 1, "the one-room walk");
        assert!(bounds.t_left > 0 && bounds.t_right >= world.width, "{:?}", bounds);
        assert!(bounds.t_top > 0 && bounds.t_bottom < world.height, "{:?}", bounds);
        assert!(world.tiles.len() == world.width as usize * world.height as usize);
    }
}
