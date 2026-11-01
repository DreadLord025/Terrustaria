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

// ---------------------------------------------------------------- grass
// `WorldGen.TileFrameCosmetic`'s grass family (WorldGen.cs:82184-84836). Grass
// frames *against* its base dirt instead of merging with it: the base
// neighbours are marked `-2` (WorldGen.cs:83380) and five override chains
// translate the resulting type/`-1`/`-2` pattern into the transition
// rectangles, so the green edge wraps steps and blends into the dirt mass
// instead of ending in a hard rim.

/// The eight neighbour slots of `TileFrameCosmetic`, mirroring the `ref int`
/// arguments of the original: the neighbour's framing type, `-1` once a slot
/// became "open" (`TileMergeAttemptWeird`, WorldGen.cs:82188) and `-2` for the
/// base material after the dirt marker (WorldGen.cs:83380).
#[derive(Clone, Copy)]
struct Nb {
    up: i32,
    down: i32,
    left: i32,
    right: i32,
    ul: i32,
    ur: i32,
    dl: i32,
    dr: i32,
}

impl Nb {
    fn slots(&mut self) -> [&mut i32; 8] {
        [
            &mut self.up,
            &mut self.down,
            &mut self.left,
            &mut self.right,
            &mut self.ul,
            &mut self.ur,
            &mut self.dl,
            &mut self.dr,
        ]
    }

    fn sides(&mut self) -> [&mut i32; 4] {
        [&mut self.up, &mut self.down, &mut self.left, &mut self.right]
    }
}

/// `TileMergeAttempt(myType, lookfor, ref …)` — the eight-slot int overload
/// (WorldGen.cs:81035): every slot equal to `lookfor` becomes `myType`.
fn merge_eq(n: &mut Nb, my: i32, lookfor: i32) {
    for v in n.slots() {
        if *v == lookfor {
            *v = my;
        }
    }
}

/// The `bool[] lookfor` overloads (WorldGen.cs:81071): every active slot in
/// the set becomes `myType`.
fn merge_set(n: &mut Nb, my: i32, lookfor: impl Fn(u16) -> bool) {
    for v in n.slots() {
        if *v > -1 && lookfor(*v as u16) {
            *v = my;
        }
    }
}

/// The grass-family gate of `TileFrameCosmetic` (WorldGen.cs:82185).
fn is_grass_family(num: u16) -> bool {
    matches!(num, 2 | 23 | 661 | 662 | 60 | 477 | 492 | 70 | 109 | 199)
        || td::TILE_MOSS.has(num)
        || td::SET_NEEDS_GRASS_FRAMING.has(num)
        || td::SET_TILE_MOSS_BRICK.has(num)
}

/// The full grass-family framing: merge preparation, the five override chains,
/// the `-2` marker, the `-1`/`-2` merge tables and the generic chain fallback.
/// `n` carries the raw framing types of the eight neighbours (`-1` when
/// inactive), with the side slots slope-adjusted like the original
/// (WorldGen.cs:81440-81507).
fn grass_frame_of(x: i32, y: i32, num: i32, n: &mut Nb) -> (i16, i16) {
    // `TileMergeAttempt(0, TileID.Sets.Dirt, …)` (WorldGen.cs:81516): every
    // dirt-family neighbour counts as plain dirt before the type logic.
    merge_set(n, 0, |v| td::SET_DIRT.has(v));
    // Per-type pre-merges of the big `switch (num)` (WorldGen.cs:82004-82006):
    // only jungle grass has a case among the grass ids.
    if num == 60 {
        merge_eq(n, 59, 211);
    }
    // Moss/brick vs dirt (WorldGen.cs:82131-82142; the dirt/stone arms concern
    // other tiles and stay in the generic path).
    if td::TILE_MOSS.has(num as u16) || td::SET_TILE_MOSS_BRICK.has(num as u16) {
        merge_set(n, num, |v| td::SET_DIRT.has(v));
    }
    // Grass spreads over ore and ore over grass (WorldGen.cs:82143-82150).
    if td::SET_GRASS.has(num as u16) {
        merge_set(n, num, |v| td::SET_ORE.has(v));
    }
    // WorldGen.cs:82188: every non-solid neighbour becomes "open".
    for v in n.slots() {
        if *v > -1 && *v != num && !td::TILE_SOLID.has(*v as u16) {
            *v = -1;
        }
    }
    // The base material: `TileID.Sets.NeedsGrassFramingDirt` (WorldGen.cs:82189),
    // with the mud/moss/brick overrides and the grass↔corrupt-grass unification.
    let mut base = td::GRASS_DIRT[num as usize] as i32;
    if matches!(num, 60 | 70 | 661 | 662) {
        base = 59;
    } else if td::TILE_MOSS.has(num as u16) {
        base = 1;
    } else if td::SET_TILE_MOSS_BRICK.has(num as u16) {
        base = 38;
    } else if matches!(num, 2 | 477) {
        merge_eq(n, base, 23);
    } else if num == 23 {
        merge_eq(n, base, 2);
    }

    let v = variant(x, y) as i16;
    let v18 = v * 18;
    let v36 = v * 36;
    let v54 = v * 54;
    let (up, down, left, right) = (n.up, n.down, n.left, n.right);
    let (ul, ur, dl, dr) = (n.ul, n.ur, n.dl, n.dr);
    let in_base = |v: i32| v == num || v == base;

    // Override chain A (WorldGen.cs:82216-82821).
    let mut rect = if up != num && up != base && (down == num || down == base) {
        // Top edge: the green line overhangs the exposed side.
        if left == base && right == num {
            Some((v18, 198))
        } else if left == num && right == base {
            Some((54 + v18, 198))
        } else {
            None
        }
    } else if down != num && down != base && (up == num || up == base) {
        // Bottom edge.
        if left == base && right == num {
            Some((v18, 216))
        } else if left == num && right == base {
            Some((54 + v18, 216))
        } else {
            None
        }
    } else if left != num && left != base && (right == num || right == base) {
        // Left edge. The original's second branch (WorldGen.cs:82314) repeats
        // the first condition verbatim and is unreachable; only the
        // dirt-above/grass-below case ever frames here.
        if up == base && down == num {
            Some((72, 144 + v18))
        } else {
            None
        }
    } else if right != num && right != base && (left == num || left == base) {
        // Right edge; the second condition is `down == num && right == up`
        // verbatim (WorldGen.cs:82353).
        if up == base && down == num {
            Some((90, 144 + v18))
        } else if down == num && right == up {
            Some((90, 90 + v18))
        } else {
            None
        }
    } else if up == num && down == num && left == num && right == num {
        // Interior grass.
        if ul != num && ur != num && dl != num && dr != num {
            if dr == base {
                Some((108 + v18, 324))
            } else if ur == base {
                Some((108 + v18, 342))
            } else if dl == base {
                Some((108 + v18, 360))
            } else if ul == base {
                Some((108 + v18, 378))
            } else {
                Some((144 + v54, 234))
            }
        } else if ul != num && dr != num {
            Some((36 + v18, 306))
        } else if ur != num && dl != num {
            Some((90 + v18, 306))
        } else if ul != num && ur == num && dl == num && dr == num {
            Some((54, 108 + v36))
        } else if ul == num && ur != num && dl == num && dr == num {
            Some((36, 108 + v36))
        } else if ul == num && ur == num && dl != num && dr == num {
            Some((54, 90 + v36))
        } else if ul == num && ur == num && dl == num && dr != num {
            Some((36, 90 + v36))
        } else {
            None
        }
    } else if up == num && down == base && left == num && right == num && ul == -1 && ur == -1 {
        Some((108 + v18, 18))
    } else if up == base && down == num && left == num && right == num && dl == -1 && dr == -1 {
        Some((108 + v18, 36))
    } else if up == num && down == num && left == base && right == num && ur == -1 && dr == -1 {
        Some((198, v18))
    } else if up == num && down == num && left == num && right == base && ul == -1 && dl == -1 {
        Some((180, v18))
    } else if up == num && down == base && left == num && right == num {
        if ur != -1 {
            Some((54, 108 + v36))
        } else if ul != -1 {
            Some((36, 108 + v36))
        } else {
            None
        }
    } else if up == base && down == num && left == num && right == num {
        if dr != -1 {
            Some((54, 90 + v36))
        } else if dl != -1 {
            Some((36, 90 + v36))
        } else {
            None
        }
    } else if up == num && down == num && left == num && right == base {
        if ul != -1 {
            Some((54, 90 + v36))
        } else if dl != -1 {
            Some((54, 108 + v36))
        } else {
            None
        }
    } else if up == num && down == num && left == base && right == num {
        if ur != -1 {
            Some((36, 90 + v36))
        } else if dr != -1 {
            Some((36, 108 + v36))
        } else {
            None
        }
    } else if (up == base && down == num && left == num && right == num)
        || (up == num && down == base && left == num && right == num)
        || (up == num && down == num && left == base && right == num)
        || (up == num && down == num && left == num && right == base)
    {
        // Exactly one base side, the rest grass (WorldGen.cs:82804).
        Some((18 + v18, 18))
    } else {
        None
    };

    // Override chain B (WorldGen.cs:82822-82896): an independent `if`; a miss
    // leaves the previous rectangle untouched.
    if in_base(up) && in_base(down) && in_base(left) && in_base(right) {
        let b = if ul != num && ul != base && in_base(ur) && in_base(dl) && in_base(dr) {
            Some((54, 108 + v36))
        } else if ur != num && ur != base && in_base(ul) && in_base(dl) && in_base(dr) {
            Some((36, 108 + v36))
        } else if dl != num && dl != base && in_base(ul) && in_base(ur) && in_base(dr) {
            Some((54, 90 + v36))
        } else if dr != num && dr != base && in_base(ul) && in_base(ur) && in_base(dl) {
            Some((36, 90 + v36))
        } else {
            None
        };
        if b.is_some() {
            rect = b;
        }
    }

    // Override chain C (WorldGen.cs:82897-83184): an unconditional `if` in the
    // original — a match replaces any earlier rectangle.
    {
        let c = if up != base && up != num && down == num && left != base && left != num
            && right == num
            && dr != base
            && dr != num
        {
            Some((90 + v18, 270))
        } else if up != base
            && up != num
            && down == num
            && left == num
            && right != base
            && right != num
            && dl != base
            && dl != num
        {
            Some((144 + v18, 270))
        } else if down != base
            && down != num
            && up == num
            && left != base
            && left != num
            && right == num
            && ur != base
            && ur != num
        {
            Some((90 + v18, 288))
        } else if down != base
            && down != num
            && up == num
            && left == num
            && right != base
            && right != num
            && ul != base
            && ul != num
        {
            Some((144 + v18, 288))
        } else if up != num
            && up != base
            && down == num
            && left == num
            && right == num
            && dl != num
            && dl != base
            && dr != num
            && dr != base
        {
            Some((144 + v54, 216))
        } else if down != num
            && down != base
            && up == num
            && left == num
            && right == num
            && ul != num
            && ul != base
            && ur != num
            && ur != base
        {
            Some((144 + v54, 252))
        } else if left != num
            && left != base
            && down == num
            && up == num
            && right == num
            && ur != num
            && ur != base
            && dr != num
            && dr != base
        {
            Some((126 + v54, 234))
        } else if right != num
            && right != base
            && down == num
            && up == num
            && left == num
            && ul != num
            && ul != base
            && dl != num
            && dl != base
        {
            Some((162 + v54, 234))
        } else if up != base
            && up != num
            && in_base(down)
            && left == base
            && right == base
        {
            Some((36 + v18, 270))
        } else if down != base
            && down != num
            && in_base(up)
            && left == base
            && right == base
        {
            Some((36 + v18, 288))
        } else if left != base
            && left != num
            && in_base(right)
            && up == base
            && down == base
        {
            Some((0, 270 + v18))
        } else if right != base
            && right != num
            && in_base(left)
            && up == base
            && down == base
        {
            Some((18, 270 + v18))
        } else if up == num && down == base && left == base && right == base {
            Some((198 + v18, 288))
        } else if up == base && down == num && left == base && right == base {
            Some((198 + v18, 270))
        } else if up == base && down == base && left == num && right == base {
            Some((198 + v18, 306))
        } else if up == base && down == base && left == base && right == num {
            Some((144 + v18, 306))
        } else {
            None
        };
        if c.is_some() {
            rect = c;
        }
    }

    // Override chain D (WorldGen.cs:83185-83340): three grass sides, one open,
    // with a base corner — also an unconditional `if`.
    {
        let d = if up != num && up != base && down == num && left == num && right == num {
            if in_base(dl) && !in_base(dr) {
                Some((v18, 324))
            } else if in_base(dr) && !in_base(dl) {
                Some((54 + v18, 324))
            } else {
                None
            }
        } else if down != num && down != base && up == num && left == num && right == num {
            if in_base(ul) && !in_base(ur) {
                Some((v18, 342))
            } else if in_base(ur) && !in_base(ul) {
                Some((54 + v18, 342))
            } else {
                None
            }
        } else if left != num && left != base && up == num && down == num && right == num {
            if in_base(ur) && !in_base(dr) {
                Some((54 + v18, 360))
            } else if in_base(dr) && !in_base(ur) {
                Some((v18, 360))
            } else {
                None
            }
        } else if right != num && right != base && up == num && down == num && left == num {
            if in_base(ul) && !in_base(dl) {
                Some((v18, 378))
            } else if in_base(dl) && !in_base(ul) {
                Some((54 + v18, 378))
            } else {
                None
            }
        } else {
            None
        };
        if d.is_some() {
            rect = d;
        }
    }

    // Override chain E (WorldGen.cs:83341-83379): the mowed-grass checkerboard.
    if in_base(up) && in_base(down) && in_base(left) && in_base(right)
        && ul != -1 && ur != -1 && dl != -1 && dr != -1
    {
        rect = if (x + y) % 2 == 1 {
            Some((108 + v18, 198))
        } else {
            Some((18 + v18, 18))
        };
    }

    // WorldGen.cs:83380-83392: mark the base material as `-2` before the
    // generic merge, so the tables below can tell "base" from "another solid".
    if base >= 0 && td::SET_DIRT.has(base as u16) {
        merge_set(n, -2, |v| td::SET_DIRT.has(v));
    } else if base >= 0 && td::SET_MUD.has(base as u16) {
        merge_set(n, -2, |v| td::SET_MUD.has(v));
    } else {
        merge_eq(n, -2, base);
    }
    // `TileMergeAttempt(num, Main.tileMerge[num], …)` (WorldGen.cs:83394).
    merge_set(n, num, |v| td::merges(num as u16, v));

    if rect.is_none()
        && (td::TILE_MERGE_DIRT.has(num as u16) || td::SET_CHECKS_FOR_MERGE.has(num as u16))
    {
        // WorldGen.cs:83395-83417: `flag7` is already true for the grass
        // family, so the second Weird merge is skipped; every remaining side
        // that is not the grass itself becomes "open".
        for v in n.sides() {
            if *v > -1 && *v != num {
                *v = -1;
            }
        }
        let (up, down, left, right) = (n.up, n.down, n.left, n.right);
        let (ul, ur, dl, dr) = (n.ul, n.ur, n.dl, n.dr);
        let sides_full = up != -1 && down != -1 && left != -1 && right != -1;
        let t = if sides_full {
            // The `-2` side/corner tables (WorldGen.cs:83419-83797).
            if up == -2 && down == num && left == num && right == num {
                Some((144 + v18, 108))
            } else if up == num && down == -2 && left == num && right == num {
                Some((144 + v18, 90))
            } else if up == num && down == num && left == -2 && right == num {
                Some((162, 126 + v36))
            } else if up == num && down == num && left == num && right == -2 {
                Some((144, 126 + v36))
            } else if up == -2 && down == num && left == -2 && right == num {
                Some((36, 90 + v36))
            } else if up == -2 && down == num && left == num && right == -2 {
                Some((54, 90 + v36))
            } else if up == num && down == -2 && left == -2 && right == num {
                Some((36, 108 + v36))
            } else if up == num && down == -2 && left == num && right == -2 {
                Some((54, 108 + v36))
            } else if up == num && down == num && left == -2 && right == -2 {
                Some((180, 126 + v36))
            } else if up == -2 && down == -2 && left == num && right == num {
                Some((144 + v18, 180))
            } else if up == -2 && down == num && left == -2 && right == -2 {
                Some((198, 90 + v36))
            } else if up == num && down == -2 && left == -2 && right == -2 {
                Some((198, 144 + v18))
            } else if up == -2 && down == -2 && left == num && right == -2 {
                Some((216, 144 + v18))
            } else if up == -2 && down == -2 && left == -2 && right == num {
                Some((216, 90 + v18))
            } else if up == -2 && down == -2 && left == -2 && right == -2 {
                Some((108 + v18, 198))
            } else if up == num && down == num && left == num && right == num {
                // Base corners (WorldGen.cs:83723-83797): four independent
                // `if`s, each overwriting the previous rectangle.
                let mut r = None;
                if ul == -2 {
                    r = Some((18, 108 + v36));
                }
                if ur == -2 {
                    r = Some((0, 108 + v36));
                }
                if dl == -2 {
                    r = Some((18, 90 + v36));
                }
                if dr == -2 {
                    r = Some((0, 90 + v36));
                }
                r
            } else {
                None
            }
        } else {
            // One or more open sides (WorldGen.cs:83799-84241).
            let open = if up != -1 && down != -1 && left == -1 && right == num {
                if up == -2 && down == num {
                    Some((72, 144 + v18))
                } else if down == -2 && up == num {
                    Some((72, 90 + v18))
                } else {
                    None
                }
            } else if up != -1 && down != -1 && left == num && right == -1 {
                if up == -2 && down == num {
                    Some((90, 144 + v18))
                } else if down == -2 && up == num {
                    Some((90, 90 + v18))
                } else {
                    None
                }
            } else if up == -1 && down == num && left != -1 && right != -1 {
                if left == -2 && right == num {
                    Some((v18, 198))
                } else if right == -2 && left == num {
                    Some((54 + v18, 198))
                } else {
                    None
                }
            } else if up == num && down == -1 && left != -1 && right != -1 {
                if left == -2 && right == num {
                    Some((v18, 216))
                } else if right == -2 && left == num {
                    Some((54 + v18, 216))
                } else {
                    None
                }
            } else if up != -1 && down != -1 && left == -1 && right == -1 {
                if up == -2 && down == -2 {
                    Some((108, 216 + v36))
                } else if up == -2 {
                    Some((126, 144 + v18))
                } else if down == -2 {
                    Some((126, 90 + v18))
                } else {
                    None
                }
            } else if up == -1 && down == -1 && left != -1 && right != -1 {
                if left == -2 && right == -2 {
                    Some((162 + v18, 198))
                } else if left == -2 {
                    Some((v18, 252))
                } else if right == -2 {
                    Some((54 + v18, 252))
                } else {
                    None
                }
            } else if up == -2 && down == -1 && left == -1 && right == -1 {
                Some((108, 144 + v18))
            } else if up == -1 && down == -2 && left == -1 && right == -1 {
                Some((108, 90 + v18))
            } else if up == -1 && down == -1 && left == -2 && right == -1 {
                Some((v18, 234))
            } else if up == -1 && down == -1 && left == -1 && right == -2 {
                Some((54 + v18, 234))
            } else {
                None
            };
            if open.is_some() {
                open
            } else if !matches!(num, 2 | 23 | 60 | 70 | 109 | 199 | 477 | 492 | 633 | 661 | 662)
            {
                // The non-grass arm (WorldGen.cs:83801-83878) — moss and moss
                // brick land here.
                if up == -1 && down == -2 && left == num && right == num {
                    Some((234 + v18, 0))
                } else if up == -2 && down == -1 && left == num && right == num {
                    Some((234 + v18, 18))
                } else if up == num && down == num && left == -1 && right == -2 {
                    Some((234 + v18, 36))
                } else if up == num && down == num && left == -2 && right == -1 {
                    Some((234 + v18, 54))
                } else {
                    None
                }
            } else {
                None
            }
        };
        if t.is_some() {
            rect = t;
        }
    }

    // WorldGen.cs:84390-84836. `TileID.Sets.HasSlopeFrames` (WorldGen.cs:84245)
    // has no grass-family member — their slope look comes from the slope draw
    // path — so the slope-frame block is not ported here.
    if rect.is_none() {
        // WorldGen.cs:84398-84402: for the grass family the base `-2` slots
        // rejoin the mass before the generic chain runs.
        if matches!(num, 2 | 23 | 60 | 70 | 109 | 199 | 477 | 492 | 633 | 661 | 662)
            || td::TILE_MOSS.has(num as u16)
            || td::SET_TILE_MOSS_BRICK.has(num as u16)
        {
            merge_eq(n, num, -2);
        }
        // The generic chain (WorldGen.cs:84403-84765) with `-1` = open.
        let mut mask = 0u8;
        for (i, bit) in [
            UP, DOWN, LEFT, RIGHT, UP_LEFT, UP_RIGHT, DOWN_LEFT, DOWN_RIGHT,
        ]
        .iter()
        .enumerate()
        {
            if *n.slots()[i] == num {
                mask |= *bit;
            }
        }
        for r in RULES.iter() {
            if mask & r.need == r.want {
                rect = Some(r.frames[variant(x, y)]);
                break;
            }
        }
        // WorldGen.cs:84767-84784: the interior fallback.
        if rect.is_none() {
            rect = Some(([18, 36, 54][variant(x, y)], 18));
        }
    }
    let (fx, fy) = rect.expect("grass framing always resolves");
    // WorldGen.cs:84785-84792 shifts the fourth large-frame row down 90 px;
    // no grass-family tile uses `tileLargeFrames` and the `frameNumber`
    // stand-in only yields 0..=2, so the shift cannot trigger.
    (fx, fy)
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
    let n = [
        up,
        down,
        left,
        right,
        ul.map_or(-1, neighbour),
        ur.map_or(-1, neighbour),
        dl.map_or(-1, neighbour),
        dr.map_or(-1, neighbour),
    ];
    // The grass family branches off here with the raw neighbour types
    // (WorldGen.cs:82184): its chains compare against the bare base-dirt id,
    // which the generic merges below would fold away.
    if is_grass_family(me) {
        let mut nb = Nb {
            up: n[0],
            down: n[1],
            left: n[2],
            right: n[3],
            ul: n[4],
            ur: n[5],
            dl: n[6],
            dr: n[7],
        };
        return Some(grass_frame_of(x, y, num, &mut nb));
    }
    let mut n = n;

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

    /// A lone grass tile on dirt takes the dirt-blend bottom edge
    /// (`up == -1 && down == -2 && left == -1 && right == -1`,
    /// WorldGen.cs:84185-84202) instead of the old green rim.
    #[test]
    fn lone_grass_over_dirt_blends() {
        let mut w = World::new("t".into(), 11, 11);
        for y in 6..10 {
            for x in 1..10 {
                solid(&mut w, x, y, 0);
            }
        }
        solid(&mut w, 5, 5, 2);
        frame_world(&mut w);
        let t = *w.tile(5, 5).unwrap();
        let v = variant(5, 5) as i16;
        assert_eq!((t.frame_x, t.frame_y), (108, 90 + v * 18),);
    }

    /// A grass step whose left neighbour is dirt wraps the green edge over it
    /// (`up != num && up != base && down == base, left == base, right == num`,
    /// WorldGen.cs:82216-82254).
    #[test]
    fn grass_step_wraps_over_dirt() {
        let mut w = World::new("t".into(), 11, 11);
        for y in 5..10 {
            for x in 1..10 {
                solid(&mut w, x, y, 0);
            }
        }
        solid(&mut w, 4, 4, 0); // dirt shoulder at the step
        solid(&mut w, 5, 4, 2);
        solid(&mut w, 6, 4, 2);
        frame_world(&mut w);
        let t = *w.tile(5, 4).unwrap();
        let v = variant(5, 4) as i16;
        assert_eq!((t.frame_x, t.frame_y), (v * 18, 198));
    }

    /// Grass fully enclosed by dirt resolves through the mowed checkerboard
    /// (WorldGen.cs:83341-83379): `(i + j) % 2` picks the pattern frame.
    #[test]
    fn enclosed_grass_uses_checkerboard() {
        let mut w = World::new("t".into(), 11, 11);
        for y in 1..10 {
            for x in 1..10 {
                solid(&mut w, x, y, 0);
            }
        }
        solid(&mut w, 5, 5, 2);
        solid(&mut w, 5, 6, 2);
        frame_world(&mut w);
        let v5 = variant(5, 5) as i16;
        let v6 = variant(5, 6) as i16;
        let even = *w.tile(5, 5).unwrap();
        let odd = *w.tile(5, 6).unwrap();
        assert_eq!((even.frame_x, even.frame_y), (18 + v5 * 18, 18));
        assert_eq!((odd.frame_x, odd.frame_y), (108 + v6 * 18, 198));
    }

    /// Grass sandwiched between dirt above and below with open sides takes the
    /// double-blend frame (`up == -2 && down == -2 && left == -1 && right == -1`,
    /// WorldGen.cs:84044-84242).
    #[test]
    fn grass_between_dirt_blends_both_ways() {
        let mut w = World::new("t".into(), 11, 11);
        solid(&mut w, 5, 4, 0);
        solid(&mut w, 5, 5, 2);
        solid(&mut w, 5, 6, 0);
        frame_world(&mut w);
        let t = *w.tile(5, 5).unwrap();
        let v = variant(5, 5) as i16;
        assert_eq!((t.frame_x, t.frame_y), (108, 216 + v * 36));
    }
}
