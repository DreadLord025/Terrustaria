//! Automatic tile framing (autotiling).
//!
//! A port of the generic block path of `WorldGen.TileFrameCosmetic`: a tile looks at
//! its eight neighbours, decides which of them "merge" into the same mass, and picks a
//! 16x16 frame from the tileset accordingly. Tiles with `Main.tileFrameImportant`
//! (torches, furniture, plants, ...) store their frames in the save file and are skipped.
//!
//! The merge relation itself is generated into [`crate::tiledata`] from the decompiled
//! source, so it stays in sync with the game's `TileMergeAttempt` table.

use crate::tile::Tile;
use crate::tiledata as td;
use crate::world::World;
use rayon::prelude::*;

/// Neighbour slot bits used by the frame table.
const UP: u8 = 1 << 0;
const DOWN: u8 = 1 << 1;
const LEFT: u8 = 1 << 2;
const RIGHT: u8 = 1 << 3;
const UP_LEFT: u8 = 1 << 4;
const UP_RIGHT: u8 = 1 << 5;
const DOWN_LEFT: u8 = 1 << 6;
const DOWN_RIGHT: u8 = 1 << 7;

/// One row of the block frame table: the bits that must be tested and their values.
/// The first matching row wins, mirroring the `if/else if` chain in the original.
struct Rule {
    need: u8,
    want: u8,
    /// Frames for `frameNumber` 0, 1 and 2.
    frames: [(i16, i16); 3],
}

const fn rule(need: u8, want: u8, frames: [(i16, i16); 3]) -> Rule {
    Rule { need, want, frames }
}

/// The generic block frameset, in original evaluation order.
///
/// Values are `(frameX, frameY)` inside the tileset; frames sit on an 18 px grid.
static RULES: [Rule; 20] = [
    // All four sides merge: only the inner corners are visible.
    rule(
        UP | DOWN | LEFT | RIGHT | UP_LEFT | UP_RIGHT,
        UP | DOWN | LEFT | RIGHT,
        [(108, 18), (126, 18), (144, 18)],
    ),
    rule(
        UP | DOWN | LEFT | RIGHT | DOWN_LEFT | DOWN_RIGHT,
        UP | DOWN | LEFT | RIGHT,
        [(108, 36), (126, 36), (144, 36)],
    ),
    rule(
        UP | DOWN | LEFT | RIGHT | UP_LEFT | DOWN_LEFT,
        UP | DOWN | LEFT | RIGHT,
        [(180, 0), (180, 18), (180, 36)],
    ),
    rule(
        UP | DOWN | LEFT | RIGHT | UP_RIGHT | DOWN_RIGHT,
        UP | DOWN | LEFT | RIGHT,
        [(198, 0), (198, 18), (198, 36)],
    ),
    rule(
        UP | DOWN | LEFT | RIGHT,
        UP | DOWN | LEFT | RIGHT,
        [(18, 18), (36, 18), (54, 18)],
    ),
    // A single open side.
    rule(
        DOWN | LEFT | RIGHT,
        DOWN | LEFT | RIGHT,
        [(18, 0), (36, 0), (54, 0)],
    ),
    rule(
        UP | LEFT | RIGHT,
        UP | LEFT | RIGHT,
        [(18, 36), (36, 36), (54, 36)],
    ),
    rule(
        UP | DOWN | RIGHT,
        UP | DOWN | RIGHT,
        [(0, 0), (0, 18), (0, 36)],
    ),
    rule(
        UP | DOWN | LEFT,
        UP | DOWN | LEFT,
        [(72, 0), (72, 18), (72, 36)],
    ),
    // Two open sides, opposing.
    rule(DOWN | RIGHT, DOWN | RIGHT, [(0, 54), (36, 54), (72, 54)]),
    rule(DOWN | LEFT, DOWN | LEFT, [(18, 54), (54, 54), (90, 54)]),
    rule(UP | RIGHT, UP | RIGHT, [(0, 72), (36, 72), (72, 72)]),
    rule(UP | LEFT, UP | LEFT, [(18, 72), (54, 72), (90, 72)]),
    rule(
        LEFT | RIGHT,
        LEFT | RIGHT,
        [(108, 72), (126, 72), (144, 72)],
    ),
    rule(UP | DOWN, UP | DOWN, [(90, 0), (90, 18), (90, 36)]),
    // Two open sides, adjacent (outside corners).
    rule(UP, UP, [(108, 54), (126, 54), (144, 54)]),
    rule(DOWN, DOWN, [(108, 0), (126, 0), (144, 0)]),
    rule(RIGHT, RIGHT, [(162, 0), (162, 18), (162, 36)]),
    rule(LEFT, LEFT, [(216, 0), (216, 18), (216, 36)]),
    // Isolated tile.
    rule(0, 0, [(162, 54), (180, 54), (198, 54)]),
];

/// Normalised framing type: stone variants frame as stone, a few aliases for legacy ids.
#[inline]
fn framing_type(kind: u16) -> u16 {
    if td::TILE_STONE.has(kind) {
        1
    } else if kind == 668 {
        0
    } else if kind == 697 {
        51
    } else {
        kind
    }
}

/// `-1` when the tile is not active, otherwise its framing type.
#[inline]
fn neighbour(t: &Tile) -> i32 {
    if t.active() {
        framing_type(t.kind) as i32
    } else {
        -1
    }
}

/// Deterministic per-position variant, standing in for the `frameNumber` stored in the
/// save file (we do not decode that field yet). Uses a proper avalanche hash so the
/// pattern does not form visible periodic lattices.
#[inline]
fn variant(x: i32, y: i32) -> usize {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1) ^ (y as u32).wrapping_mul(0x85EB_CA77);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7FEB_352D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846C_A68B);
    h ^= h >> 16;
    (h % 3) as usize
}

/// The grass-edge rectangle table from `WorldGen.TileFrame`: `(frameX, frameY)` for a
/// grass tile whose dirt base is adjacent on one side. `None` when the generic chain
/// should be used (the original leaves the rectangle untouched in those cases).
fn grass_frame(world: &World, x: i32, y: i32) -> Option<(i16, i16)> {
    const GRASS: i32 = 2;
    const DIRT: i32 = 0;
    let ty = |dx: i32, dy: i32| {
        world
            .tile(x + dx, y + dy)
            .filter(|t| t.active())
            .map_or(-1, |t| t.kind as i32)
    };
    let up = ty(0, -1);
    let down = ty(0, 1);
    let left = ty(-1, 0);
    let right = ty(1, 0);
    let ul = ty(-1, -1);
    let ur = ty(1, -1);
    let dl = ty(-1, 1);
    let dr = ty(1, 1);
    let v = variant(x, y) as i16;
    let v18 = v * 18;
    let is_dirt = |k: i32| k == DIRT;
    let is_grass = |k: i32| k == GRASS;
    let rect = if !is_grass(up) && !is_dirt(up) && (is_grass(down) || is_dirt(down)) {
        // Top edge: the green line overhangs the exposed side.
        if is_dirt(left) && is_grass(right) {
            (v18, 198)
        } else if is_grass(left) && is_dirt(right) {
            (54 + v18, 198)
        } else {
            return None;
        }
    } else if !is_grass(down) && !is_dirt(down) && (is_grass(up) || is_dirt(up)) {
        // Bottom edge.
        if is_dirt(left) && is_grass(right) {
            (v18, 216)
        } else if is_grass(left) && is_dirt(right) {
            (54 + v18, 216)
        } else {
            return None;
        }
    } else if !is_grass(left) && !is_dirt(left) && (is_grass(right) || is_dirt(right)) {
        // Left edge.
        if is_dirt(up) && is_grass(down) {
            (72, 144 + v18)
        } else if is_grass(up) && is_dirt(down) {
            (72, 90 + v18)
        } else {
            return None;
        }
    } else if !is_grass(right) && !is_dirt(right) && (is_grass(left) || is_dirt(left)) {
        // Right edge.
        if is_dirt(up) && is_grass(down) {
            (90, 144 + v18)
        } else if is_grass(up) && is_dirt(down) {
            (90, 90 + v18)
        } else {
            return None;
        }
    } else if is_grass(up) && is_grass(down) && is_grass(left) && is_grass(right) {
        // Interior grass with only an inner dirt corner visible.
        if ul != GRASS && ur != GRASS && dl != GRASS && dr != GRASS {
            if is_dirt(dr) {
                (108 + v18, 324)
            } else if is_dirt(ur) {
                (108 + v18, 342)
            } else if is_dirt(dl) {
                (108 + v18, 360)
            } else if is_dirt(ul) {
                (108 + v18, 378)
            } else {
                (144 + v * 54, 234)
            }
        } else {
            return None;
        }
    } else {
        return None;
    };
    Some(rect)
}

/// Computes the frame for one tile. Returns `(frameX, frameY)` in tileset pixels.
pub fn frame_of(world: &World, x: i32, y: i32) -> Option<(i16, i16)> {
    let tile = world.tile(x, y)?;
    if !tile.active() {
        return None;
    }
    let me = framing_type(tile.kind);
    let num = me as i32;

    let up_t = world.tile(x, y - 1);
    let down_t = world.tile(x, y + 1);
    let left_t = world.tile(x - 1, y);
    let right_t = world.tile(x + 1, y);
    let ul = world.tile(x - 1, y - 1);
    let ur = world.tile(x + 1, y - 1);
    let dl = world.tile(x - 1, y + 1);
    let dr = world.tile(x + 1, y + 1);

    let slope = tile.slope();
    let mut up = up_t.map_or(-1, neighbour);
    let mut down = down_t.map_or(-1, neighbour);
    let mut left = left_t.map_or(-1, neighbour);
    let mut right = right_t.map_or(-1, neighbour);

    // Sloped neighbours do not contribute their open side.
    if let Some(t) = left_t {
        if t.slope() == 1 || t.slope() == 3 {
            left = -1;
        }
    }
    if let Some(t) = right_t {
        if t.slope() == 2 || t.slope() == 4 {
            right = -1;
        }
    }
    if let Some(t) = up_t {
        if t.slope() == 3 || t.slope() == 4 {
            up = -1;
        }
    }
    if let Some(t) = down_t {
        if t.slope() == 1 || t.slope() == 2 {
            down = -1;
        }
    }
    // The tile's own slope hides two of its sides.
    match slope {
        1 => {
            up = -1;
            right = -1;
        }
        2 => {
            up = -1;
            left = -1;
        }
        3 => {
            down = -1;
            right = -1;
        }
        4 => {
            down = -1;
            left = -1;
        }
        _ => {}
    }

    // Neighbour slots: up, down, left, right, then the four corners.
    let mut n = [
        up,
        down,
        left,
        right,
        ul.map_or(-1, neighbour),
        ur.map_or(-1, neighbour),
        dl.map_or(-1, neighbour),
        dr.map_or(-1, neighbour),
    ];

    // `WorldGen.TileMergeAttempt`: any slot whose type matches becomes `me`.
    fn merge<F: Fn(u16) -> bool>(n: &mut [i32; 8], me: i32, pred: F) {
        for v in n.iter_mut() {
            if *v >= 0 && pred(*v as u16) {
                *v = me;
            }
        }
    }

    // Same type always merges, plus the directed pairs extracted from
    // `TileFrameCosmetic` (mostly the per-type `switch (num)` cases).
    for v in n.iter_mut() {
        if *v >= 0 && *v != num && (td::merges(me, *v as u16) || td::merges(*v as u16, me)) {
            *v = num;
        }
    }

    // Set-based merge families, in the same order as the original.
    if td::TILE_BLEND_ALL.has(me) && !td::TILE_ROPE.has(me) {
        merge(&mut n, num, |v| td::MERGES_WITH_ALL.has(v));
    }
    if td::SET_FORCED_DIRT_MERGING.has(me) {
        merge(&mut n, num, |v| td::SET_DIRT.has(v));
    }
    if td::SET_DIRT.has(me) {
        merge(&mut n, num, |v| {
            td::TILE_MERGE_DIRT.has(v)
                || v == 147
                || td::SET_DIRT.has(v)
                || td::SET_MUD.has(v)
                || v == 191
                || td::SET_FORCED_DIRT_MERGING.has(v)
                || td::SET_MERGES_WITH_DIRT_IN_A_SPECIAL_WAY.has(v)
        });
    }
    if td::TILE_MERGE_DIRT.has(me) {
        merge(&mut n, num, |v| td::SET_DIRT.has(v));
        if me == 1 {
            // Stone blends with ash everywhere, and with mud below the rock layer.
            merge(&mut n, num, |v| td::SET_ASH.has(v));
            if y as f32 > world.height as f32 * 0.6 {
                merge(&mut n, num, |v| td::SET_MUD.has(v));
            }
        }
    }
    if td::SET_MUD.has(me) {
        merge(&mut n, num, |v| td::SET_ORE_MERGES_WITH_MUD.has(v));
    } else if td::SET_ORE_MERGES_WITH_MUD.has(me) {
        merge(&mut n, num, |v| td::SET_MUD.has(v));
    }
    if td::TILE_MOSS.has(me) || td::SET_TILE_MOSS_BRICK.has(me) {
        merge(&mut n, num, |v| td::SET_DIRT.has(v));
    } else if td::SET_DIRT.has(me) {
        merge(&mut n, num, |v| {
            td::TILE_MOSS.has(v) || td::SET_TILE_MOSS_BRICK.has(v)
        });
    }
    // Grass draws a dirt-coloured border, so it merges with its base dirt.
    if td::SET_NEEDS_GRASS_FRAMING.has(me) {
        let base = td::GRASS_DIRT[me as usize];
        merge(&mut n, num, |v| td::SET_DIRT.has(v) || v == base);
    }

    let mut mask = 0u8;
    for (i, bit) in [
        UP, DOWN, LEFT, RIGHT, UP_LEFT, UP_RIGHT, DOWN_LEFT, DOWN_RIGHT,
    ]
    .iter()
    .enumerate()
    {
        if n[i] == num {
            mask |= *bit;
        }
    }

    // `frameNumber` is stored per tile in the save file but we approximate it with a
    // position hash. `Main.tileLargeFrames` overrides it with a fixed per-position
    // pattern (used by tiles that must not tile their texture).
    // `WorldGen.TileFrame`'s grass overrides: grass tiles use a dedicated set of
    // side/top/corner frames so the green edge wraps around a step instead of ending
    // in a plain dirt face. Only tile 2 (forest grass) has the matching Tiles_2 layout.
    if tile.kind == 2 {
        if let Some(f) = grass_frame(world, x, y) {
            return Some(f);
        }
    }

    let mut variant = variant(x, y);
    match td::LARGE_FRAMES[me as usize] {
        1 => {
            const LARGE_4: [[usize; 3]; 4] = [[1, 3, 1], [0, 2, 0], [1, 1, 3], [0, 0, 2]];
            variant = LARGE_4[y.rem_euclid(4) as usize][x.rem_euclid(3) as usize];
        }
        2 => variant = (x.rem_euclid(2) + y.rem_euclid(2) * 2) as usize,
        _ => {}
    }
    let variant = variant.min(2);

    for r in RULES.iter() {
        if mask & r.need == r.want {
            return Some(r.frames[variant]);
        }
    }
    Some(([18, 36, 54][variant], 18))
}

/// Frames every non-important tile in the world (in place).
///
/// `Main.tileFrameImportant` tiles keep the frames stored in the save file.
#[allow(clippy::needless_range_loop)] // `x` also indexes the world, not just `row`
pub fn frame_world(world: &mut World) -> usize {
    let important = world.frame_important.clone();
    let (w, h) = (world.width as usize, world.height as usize);
    if w <= 2 || h <= 2 {
        return 0;
    }

    // C4: frames depend only on kinds/flags/slopes, never on other frames, so a band
    // of rows can be framed in parallel. The immutable world is shared by the tasks;
    // results land in the band buffer and are copied in afterwards. 128 rows is
    // ~3.3 MB of scratch for a 6400-wide world.
    const BAND: usize = 128;
    let mut band = Vec::new();
    let mut framed = 0usize;
    let mut y0 = 1usize;
    while y0 + 1 < h {
        let y1 = (y0 + BAND).min(h - 1);
        let rows = y1 - y0;
        band.clear();
        band.resize(rows * w, (i16::MIN, i16::MIN));
        {
            let world = &*world;
            band.par_chunks_mut(w).enumerate().for_each(|(row_i, row)| {
                let y = (y0 + row_i) as i32;
                for x in 1..w - 1 {
                    let t = &world.tiles[y as usize * w + x];
                    if !t.active() || important.get(t.kind as usize).copied().unwrap_or(false) {
                        continue;
                    }
                    // Slopes and half bricks frame exactly like full blocks; the draw
                    // pass clips the sprite into the slope/half silhouette.
                    if let Some((fx, fy)) = frame_of(world, x as i32, y) {
                        row[x] = (fx, fy);
                    }
                }
            });
        }
        for (row_i, row) in band.chunks(w).enumerate() {
            let y = y0 + row_i;
            for x in 1..w - 1 {
                let (fx, fy) = row[x];
                if fx == i16::MIN {
                    continue;
                }
                let t = &mut world.tiles[y * w + x];
                t.frame_x = fx;
                t.frame_y = fy;
                framed += 1;
            }
        }
        y0 = y1;
    }
    framed
}

// ---------------------------------------------------------------- walls

/// `Framing.wallFrameLookup` — `(column, row)` cells of the 36 px wall grid for each
/// neighbour mask, with three random variants. Ported from `Framing.AddWallFrameLookup`.
static WALL_FRAMES: [[(u8, u8); 3]; 20] = [
    [(9, 3), (10, 3), (11, 3)],
    [(6, 3), (7, 3), (8, 3)],
    [(12, 0), (12, 1), (12, 2)],
    [(1, 4), (3, 4), (5, 4)],
    [(9, 0), (9, 1), (9, 2)],
    [(0, 4), (2, 4), (4, 4)],
    [(6, 4), (7, 4), (8, 4)],
    [(1, 2), (2, 2), (3, 2)],
    [(6, 0), (7, 0), (8, 0)],
    [(5, 0), (5, 1), (5, 2)],
    [(1, 3), (3, 3), (5, 3)],
    [(4, 0), (4, 1), (4, 2)],
    [(0, 3), (2, 3), (4, 3)],
    [(0, 0), (0, 1), (0, 2)],
    [(1, 0), (2, 0), (3, 0)],
    [(1, 1), (2, 1), (3, 1)],
    [(6, 1), (7, 1), (8, 1)],
    [(6, 2), (7, 2), (8, 2)],
    [(10, 0), (10, 1), (10, 2)],
    [(11, 0), (11, 1), (11, 2)],
];

/// `Framing.centerWallFrameLookup` — offsets added to mask 15 for fully enclosed walls.
static CENTER_WALL_FRAMES: [[u8; 3]; 3] = [[2, 0, 0], [0, 1, 4], [0, 3, 0]];

/// Deterministic wall variant (Terraria stores a random `wallFrameNumber` per wall).
#[inline]
fn wall_variant(x: i32, y: i32) -> usize {
    let mut h = (x as u32).wrapping_mul(0xC2B2_AE35) ^ (y as u32).wrapping_mul(0x27D4_EB2F);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2545_F491);
    h ^= h >> 13;
    (h % 3) as usize
}

/// `Framing.WallFrame` for one tile. `None` when there is no wall.
pub fn wall_frame_of(world: &World, x: i32, y: i32) -> Option<(u8, u8)> {
    let tile = world.tile(x, y)?;
    if tile.wall == 0 {
        return None;
    }
    let mut mask = 0u8;
    if world.tile(x, y - 1).is_some_and(|t| t.wall != 0) {
        mask |= 1;
    }
    if world.tile(x - 1, y).is_some_and(|t| t.wall != 0) {
        mask |= 2;
    }
    if world.tile(x + 1, y).is_some_and(|t| t.wall != 0) {
        mask |= 4;
    }
    if world.tile(x, y + 1).is_some_and(|t| t.wall != 0) {
        mask |= 8;
    }
    if mask == 15 {
        mask += CENTER_WALL_FRAMES[x.rem_euclid(3) as usize][y.rem_euclid(3) as usize];
    }
    Some(WALL_FRAMES[mask as usize][wall_variant(x, y)])
}

/// Re-frames a small square after a tile edit. Only the affected area is touched, so
/// mining/placing does not need a full-world framing pass.
pub fn reframe_area(world: &mut World, cx: i32, cy: i32, radius: i32) {
    let r = radius.clamp(1, 4);
    let important = world.frame_important.clone();
    let mut tiles = Vec::new();
    let mut walls = Vec::new();
    let mut torches = Vec::new();
    for y in (cy - r).max(1)..=(cy + r).min(world.height - 2) {
        for x in (cx - r).max(1)..=(cx + r).min(world.width - 2) {
            let Some(t) = world.tile(x, y) else {
                continue;
            };
            if t.active() && !important.get(t.kind as usize).copied().unwrap_or(false) {
                if let Some(f) = frame_of(world, x, y) {
                    tiles.push((x, y, f));
                }
            }
            if t.active() && td::SET_TORCHES.has(t.kind) {
                torches.push((x, y));
            }
            if t.wall != 0 {
                if let Some(w) = wall_frame_of(world, x, y) {
                    walls.push((x, y, w));
                }
            }
        }
    }
    for (x, y, (fx, fy)) in tiles {
        let idx = world.index(x, y);
        let t = &mut world.tiles[idx];
        t.frame_x = fx;
        t.frame_y = fy;
    }
    for (x, y, (col, row)) in walls {
        let idx = world.index(x, y);
        let t = &mut world.tiles[idx];
        t.set_wall_frame(col, t.wall_row());
        t.set_wall_frame(t.wall_col(), row);
    }
    // `WorldGen.TileFrame` re-runs `CheckTorch` for the torches it touches: the frame
    // follows the attachment and a torch whose support is gone is removed.
    for (x, y) in torches {
        crate::torch::check_torch(world, x, y);
    }
}

/// Port of `Framing.WallFrame`: picks the 36 px cell for every wall.
#[allow(clippy::needless_range_loop)] // `x` also indexes the world, not just `row`
pub fn frame_walls(world: &mut World) -> usize {
    let (w, h) = (world.width as usize, world.height as usize);
    if w <= 2 || h <= 2 {
        return 0;
    }

    // C4: wall frames depend only on the wall ids of the four neighbours. Same
    // banded-parallel scheme as `frame_world`.
    const BAND: usize = 128;
    let mut band = Vec::new();
    let mut framed = 0usize;
    let mut y0 = 1usize;
    while y0 + 1 < h {
        let y1 = (y0 + BAND).min(h - 1);
        let rows = y1 - y0;
        band.clear();
        band.resize(rows * w, (u8::MAX, u8::MAX));
        {
            let world = &*world;
            band.par_chunks_mut(w).enumerate().for_each(|(row_i, row)| {
                let y = y0 + row_i;
                for x in 1..w - 1 {
                    let i = y * w + x;
                    if world.tiles[i].wall == 0 {
                        continue;
                    }
                    let mut mask = 0u8;
                    if world.tiles[i - w].wall != 0 {
                        mask |= 1;
                    }
                    if world.tiles[i - 1].wall != 0 {
                        mask |= 2;
                    }
                    if world.tiles[i + 1].wall != 0 {
                        mask |= 4;
                    }
                    if world.tiles[i + w].wall != 0 {
                        mask |= 8;
                    }
                    if mask == 15 {
                        mask += CENTER_WALL_FRAMES[x % 3][y % 3];
                    }
                    row[x] = WALL_FRAMES[mask as usize][wall_variant(x as i32, y as i32)];
                }
            });
        }
        for (row_i, row) in band.chunks(w).enumerate() {
            let y = y0 + row_i;
            for x in 1..w - 1 {
                let (col, wall_row) = row[x];
                if col == u8::MAX {
                    continue;
                }
                let t = &mut world.tiles[y * w + x];
                t.set_wall_frame(col, t.wall_row());
                t.set_wall_frame(t.wall_col(), wall_row);
                framed += 1;
            }
        }
        y0 = y1;
    }
    framed
}
#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i].kind = kind;
        w.tiles[i].flags |= crate::tile::FLAG_ACTIVE;
    }

    #[test]
    fn block_interior_uses_plain_frames() {
        let mut w = World::new("t".into(), 11, 11);
        for y in 1..10 {
            for x in 1..10 {
                solid(&mut w, x, y, 1);
            }
        }
        frame_world(&mut w);

        // Middle of the mass touches its own type on all eight sides.
        let t = *w.tile(5, 5).unwrap();
        assert!(
            matches!((t.frame_x, t.frame_y), (18, 18) | (36, 18) | (54, 18)),
            "interior frame was {:?}",
            (t.frame_x, t.frame_y)
        );

        // Top-left corner: open up and left, joined down and right.
        let t = *w.tile(1, 1).unwrap();
        assert!([0, 36, 72].contains(&t.frame_x), "x {}", t.frame_x);
        assert_eq!(t.frame_y, 54);
    }

    #[test]
    fn block_merges_dirt_with_stone() {
        let mut w = World::new("t".into(), 11, 11);
        for y in 1..10 {
            for x in 1..10 {
                solid(&mut w, x, y, 0);
            }
        }
        solid(&mut w, 5, 5, 1);
        frame_world(&mut w);
        let t = *w.tile(5, 5).unwrap();
        assert!(
            matches!((t.frame_x, t.frame_y), (18, 18) | (36, 18) | (54, 18)),
            "merged stone frame was {:?}",
            (t.frame_x, t.frame_y)
        );
    }

    #[test]
    fn important_tiles_are_left_alone() {
        let mut w = World::new("t".into(), 7, 7);
        w.frame_important = vec![false; td::TILE_COUNT];
        w.frame_important[4] = true;
        for y in 1..6 {
            for x in 1..6 {
                solid(&mut w, x, y, 1);
            }
        }
        solid(&mut w, 3, 3, 4);
        let i = w.index(3, 3);
        w.tiles[i].frame_x = 22;
        w.tiles[i].frame_y = 44;
        frame_world(&mut w);
        let t = *w.tile(3, 3).unwrap();
        assert_eq!((t.frame_x, t.frame_y), (22, 44));
    }

    #[test]
    fn walls_pick_lookup_cells() {
        let mut w = World::new("t".into(), 9, 9);
        for y in 2..5 {
            for x in 2..5 {
                let i = w.index(x, y);
                w.tiles[i].wall = 2;
            }
        }
        assert_eq!(frame_walls(&mut w), 9);

        // Enclosed wall at (3,3): mask 15 + centerWallFrameLookup[0][0] == 17.
        let center = *w.tile(3, 3).unwrap();
        assert!(
            (6..=8).contains(&center.wall_col()),
            "col {}",
            center.wall_col()
        );
        assert_eq!(center.wall_row(), 2);

        // Top-left corner (2,2): mask 12 -> WALL_FRAMES[12].
        let corner = *w.tile(2, 2).unwrap();
        assert!(
            [0, 2, 4].contains(&corner.wall_col()),
            "col {}",
            corner.wall_col()
        );
        assert_eq!(corner.wall_row(), 3);
    }
}
