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
use super::structures::place_chest;
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
) -> Vec<TempleRoom> {
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
    rooms
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

fn put_brick(world: &mut World, x: i32, y: i32, wall: u16) {
    if x < 2 || y < 2 || x >= world.width - 2 || y >= world.height - 2 {
        return;
    }
    let i = world.index(x, y);
    let t = &mut world.tiles[i];
    t.kind = TILE_TEMPLE_BRICK;
    t.flags |= FLAG_ACTIVE;
    t.liquid = 0;
    t.set_slope(0);
    t.wall = wall;
}

fn carve(world: &mut World, x: i32, y: i32, wall: u16) {
    if x < 2 || y < 2 || x >= world.width - 2 || y >= world.height - 2 {
        return;
    }
    let i = world.index(x, y);
    world.tiles[i] = Tile::default();
    world.tiles[i].wall = wall;
}

/// `AddLihzahrdAltar`'s altar write — the 3×2 altar (237) with the 3-wide
/// brick base row; the frame layout matches the C# `PlaceTile(237)` result
/// (`frameX = i*18`, `frameY = j*18`; the hand-built fallback writes the
/// same 3×2, `WorldGen.cs:34049-34062`) and the anchor it leaves is the
/// top-left tile the C# publishes as `lAltarX/lAltarY` (`:34000-34001`).
pub(crate) fn place_lihzahrd_altar(world: &mut World, x: i32, y: i32) {
    for i in 0..3 {
        for j in 0..2 {
            let px = x + i;
            let py = y + j;
            if px < 2 || py < 2 || px >= world.width - 2 || py >= world.height - 2 {
                return;
            }
            let wall = world.tile(px, py).map(|t| t.wall).unwrap_or(0);
            let idx = world.index(px, py);
            // The 3×2 frame layout the C# `PlaceTile(237)` result and the
            // hand-built fallback share (`frameX = i*18`, `frameY = j*18`,
            // `WorldGen.cs:34049-34062`). (The pre-phase-6 code shadowed the
            // loop `i` with the world index here — the garbage frames were
            // invisible while 237 was re-framed at load; the phase-6 boot
            // static make 237 frame-important, so the stored layout must be
            // the faithful one.)
            world.tiles[idx] = Tile {
                kind: TILE_ALTAR,
                frame_x: (i * 18) as i16,
                frame_y: (j * 18) as i16,
                ..Default::default()
            };
            world.tiles[idx].flags |= FLAG_ACTIVE;
            world.tiles[idx].wall = wall;
        }
        // The base row (`:32090-32100`).
        let py = y + 2;
        if x + i < 2 || py >= world.height - 2 {
            continue;
        }
        let idx2 = world.index(x + i, py);
        let t = &mut world.tiles[idx2];
        t.kind = TILE_TEMPLE_BRICK;
        t.flags |= FLAG_ACTIVE;
        t.set_slope(0);
    }
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
    // `WorldGen.cs:33440-33548` + `:34238-34462` — the faithful room walk and
    // the brick fills (the M5-TEMPLE phase-1 port). The pre-phase-2 body
    // below (the box/hall/altar write anchored on the same bury point) is the
    // interim stub shell; the faithful carve/corridor/shaft/altar/spike code
    // replaces it in the next phase and publishes the bounds below.
    let rooms = make_temple_rooms(
        rng,
        world.width as f64 / 4200.0,
        world.height - 200,
        cx,
        cy,
    );
    make_temple_generate_bricks(world, &rooms);
    // The shape-level room body (the M5-TEMPLE seam — the module docs carry
    // the split): the outer box with the unsafe walls.
    let hw = rng.range(34, 46);
    let hh = rng.range(22, 30);
    for px in (cx - hw)..=(cx + hw) {
        for py in (cy - hh)..=(cy + hh) {
            put_brick(world, px, py, WALL_TEMPLE_UNSAFE);
        }
    }
    // The corridor rooms: the central hall + the two side rooms, carved
    // with the safe walls (`LihzahrdTemplePart2`'s room carving).
    let hall_hw = hw - 6;
    let hall_hh = hh - 8;
    for px in (cx - hall_hw)..=(cx + hall_hw) {
        for py in (cy - hall_hh)..=(cy + hall_hh) {
            carve(world, px, py, WALL_TEMPLE);
        }
    }
    // The pillars: the brick columns inside the hall (the temple look).
    for px in (cx - hall_hw + 6..=cx + hall_hw - 6).step_by(9) {
        for py in (cy - hall_hh)..=(cy + hall_hh) {
            put_brick(world, px, py, WALL_TEMPLE);
            put_brick(world, px, py, WALL_TEMPLE);
        }
    }
    // The altar room: the west end of the hall.
    let altar_x = cx - hall_hw + 4;
    let altar_y = cy + hall_hh - 5;
    for px in (altar_x - 3)..=(altar_x + 6) {
        for py in (altar_y - 4)..=altar_y {
            carve(world, px, py, WALL_TEMPLE);
        }
    }
    place_lihzahrd_altar(world, altar_x, altar_y - 2);
    // The dart traps (137) embedded in the hall's brick walls facing the
    // rooms (the case-137 wiring variants fire them).
    for px in (cx - hall_hw + 3..=cx + hall_hw - 3).step_by(12) {
        let py = cy - hall_hh + 2;
        let i = world.index(px, py);
        world.tiles[i] = Tile {
            kind: 137, // `TileID.Trap` — the dart variant (frameY 0)
            ..Default::default()
        };
        world.tiles[i].flags |= FLAG_ACTIVE;
        world.tiles[i].wall = WALL_TEMPLE;
    }
    // The chests in the hall's corners.
    let mut chests = 0;
    for (dx, dy) in [(-hall_hw + 2, -hall_hh + 1), (hall_hw - 3, hall_hh - 2)] {
        if place_chest(world, cx + dx, cy + dy) {
            chests += 1;
        }
    }
    let _ = chests;
    // The GenVars publication — the M5-TEMPLE seam: the bounds are the
    // filled extent ± 10 (the C# room-union ± 10, `:33788-33817`, published
    // `:34231-34235`), `t_rooms` the carved-room count (the C# publishes the
    // final walk count, `:34235`) and the altar anchor the 3×2's top-left
    // (`:34000-34001`).
    TempleBounds {
        t_left: cx - hw - 10,
        t_right: cx + hw + 10,
        t_top: cy - hh - 10,
        t_bottom: cy + hh + 10,
        t_rooms: 2,
        l_altar_x: altar_x,
        l_altar_y: altar_y - 2,
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
        let rooms = make_temple_rooms(&mut rng, 1.0, 1000, 2000, 300);
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
        let rooms = make_temple_rooms(&mut rng, 1.0, 300, 2000, 220);
        assert_eq!(rooms.len(), 1, "the walk collapses to one room");
        let room = rooms[0].rect;
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
        let rooms = make_temple_rooms(&mut collapsed, 1.0, 300, 2000, 220);
        assert_eq!(rooms.len(), 1, "220 + 70 >= 300 - 10");
        let mut normal = Rng(0xDEAD_BEEF);
        let rooms = make_temple_rooms(&mut normal, 1.0, 301, 2000, 220);
        assert!(rooms.len() > 1, "220 + 70 < 301 - 10");
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

    /// DW-6.1: the shell publishes the GenVars the M5-TEMPLE passes read —
    /// `t_rooms` is the carved-room count, the bounds are the filled extent
    /// ± 10 (`WorldGen.cs:33788-33817`, published `:34231-34235`) and
    /// contain both the bury point and the altar anchor; the 3×2 altar (237)
    /// sits at the anchor with the faithful frame layout
    /// (`frameX = i*18`, `frameY = j*18`, `:34049-34062`) over the brick
    /// base row. The valve path makes the anchor deterministic for the
    /// assertions that pin ranges (the `hw`/`hh` box rolls stay rng-drawn).
    #[test]
    fn test_DW_6_1_shell_publishes_bounds_and_altar_anchor() {
        let mut world = World::new("t".into(), 500, 500);
        let mut rng = Rng(7);
        let bounds = pass_lihzahrd_temple(&mut world, 300, 50.0, &mut rng);
        let (bx, by) = (200, 150); // the deterministic fallback bury point
        assert_eq!(bounds.t_rooms, 2);
        // The box ± 10 ranges: hw ∈ [34, 46], hh ∈ [22, 30].
        assert!(
            (144..=156).contains(&bounds.t_left)
                && (244..=256).contains(&bounds.t_right)
                && (110..=118).contains(&bounds.t_top)
                && (182..=190).contains(&bounds.t_bottom),
            "the published bounds {:?}",
            bounds
        );
        assert!(
            bounds.t_left <= bx
                && bx <= bounds.t_right
                && bounds.t_top <= by
                && by <= bounds.t_bottom,
            "the bury point sits inside the bounds"
        );
        // The altar anchor: hall_hw = hw - 6 ∈ [28, 40], hall_hh = hh - 8 ∈
        // [14, 22] → the anchor ranges are deterministic.
        assert!(
            (164..=176).contains(&bounds.l_altar_x)
                && (157..=165).contains(&bounds.l_altar_y),
            "the altar anchor ({},{})",
            bounds.l_altar_x,
            bounds.l_altar_y
        );
        assert!(
            bounds.t_left <= bounds.l_altar_x
                && bounds.l_altar_x + 2 < bounds.t_right
                && bounds.t_top <= bounds.l_altar_y
                && bounds.l_altar_y + 1 < bounds.t_bottom,
            "the anchor sits inside the bounds"
        );
        // The 3×2 altar with the faithful frames over the brick base row.
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
            let base = world.tile(bounds.l_altar_x + i, bounds.l_altar_y + 2).unwrap();
            assert!(
                base.active() && base.kind == TILE_TEMPLE_BRICK,
                "the brick base row at +2"
            );
        }
    }
}
