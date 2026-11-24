//! Automatic tile framing (autotiling).
//!
//! A port of the generic block path of `WorldGen.TileFrameCosmetic`: a tile looks at
//! its eight neighbours, decides which of them "merge" into the same mass, and picks a
//! 16x16 frame from the tileset accordingly. Tiles with `Main.tileFrameImportant`
//! (torches, furniture, plants, ...) store their frames in the save file and are skipped.
//!
//! Every framed tile also carries a `frameNumber` variant (walls a
//! `wallFrameNumber`): the C# stores it in the runtime `Tile.bTileHeader2` bits
//! 4-7 (`Tile.cs:464-482`, **not** serialized by `SaveWorldTiles`), rolls it
//! from `WorldGen.genRand.Next(0, 3)` at reset events and reuses it otherwise
//! (`WorldGen.cs:81705-81729`, `Framing.cs:318-321`, `Framing.cs:376-404`).
//! The port keeps the same values in [`crate::world::World::variants`] (2+2
//! bits): the full-world passes roll from per-band `UnifiedRandom` streams
//! keyed off the world seed (the stand-in for the C#'s genRand interleave —
//! vanilla frames incrementally, the port frames in one pass), and the
//! edit-time reframes reuse the stored value, re-rolling only the
//! `SquareTileFrame` centre. The `Main.tileLargeFrames` /
//! `Main.wallLargeFrames` lookups replace the variant per position, exactly as
//! the C# does. W4-d109 adds the save half: the port's `.trwld` v18 tail
//! persists the packed variants and the full passes reuse them on such a
//! load ([`crate::world::World::variants_from_save`]) — a port-only extension
//! (the C# `.wld` never serializes the bits); generated worlds, retail `.wld`
//! loads and pre-v18 saves keep the reset roll.
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
fn grass_frame_of(x: i32, y: i32, num: i32, variant: usize, n: &mut Nb) -> (i16, i16) {
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

    // The `frameNumber` variant — the stored `Tile.bTileHeader2` bits 4-5
    // (`Tile.cs:464-470`), rolled from `genRand.Next(0, 3)` at reset events
    // (`WorldGen.cs:81705-81729`); here the caller supplies it.
    let v = variant as i16;
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
                rect = Some(r.frames[variant]);
                break;
            }
        }
        // WorldGen.cs:84767-84784: the interior fallback.
        if rect.is_none() {
            rect = Some(([18, 36, 54][variant], 18));
        }
    }
    let (fx, fy) = rect.expect("grass framing always resolves");
    // WorldGen.cs:84785-84792 shifts the fourth large-frame row down 90 px;
    // no grass-family tile uses `tileLargeFrames` and the `frameNumber`
    // stand-in only yields 0..=2, so the shift cannot trigger.
    (fx, fy)
}

// ------------------------------------------------- SelfFrame8Way (gemspark)
// `Framing.SelfFrame8Way` (Framing.cs:193-324): the gemspark blocks (255-268),
// the crystal block (385) and the silly balloons (446-448) frame against
// same-group neighbours through `WillItBlend` — the 255-268 switch arm of
// `TileFrameCosmetic` (WorldGen.cs:84819-84833), also reached by the
// `case 385/446/447/448: break` + `goto case 255` fallthrough
// (WorldGen.cs:81409-81413, :84818).

/// `Framing.blockStyleLookup` (`Framing.cs:92-98`): which sides a block
/// presents, by `Tile.blockType()` — 0 full, 1 half brick, 2..5 slope+1.
/// Fields are `(top, bottom, left, right)`.
static BLOCK_STYLES: [(bool, bool, bool, bool); 6] = [
    (true, true, true, true),
    (false, true, true, true),
    (false, true, true, false),
    (false, true, false, true),
    (true, false, true, false),
    (true, false, false, true),
];

/// `Tile.blockType()` (`Tile.cs:202-214`) + `Framing.FindBlockStyle`
/// (`Framing.cs:141-144`). A neighbour that fails the blend/present-side
/// test contributes the all-false style instead (the C# `Clear()`s the
/// slot — Framing.cs:214/231/248/265); the caller models that with `None`.
#[inline]
fn find_block_style(t: &Tile) -> (bool, bool, bool, bool) {
    let bt = if t.half_brick() {
        1
    } else {
        let s = t.slope();
        if s > 0 { s as usize + 1 } else { 0 }
    };
    BLOCK_STYLES[bt]
}

/// `Framing.WillItBlend` (`Framing.cs:180-191`): forced-dirt gemspark merges
/// with plain dirt, any two `Main.tileBrick` tiles blend, and equal
/// `TileID.Sets.GemsparkFramingTypes` entries blend — the 0 default makes
/// every non-gemspark pair equal, so ordinary blocks merge with their kind
/// through the style gates alone.
#[inline]
fn will_it_blend(my: u16, other: u16) -> bool {
    if td::SET_FORCED_DIRT_MERGING.has(my) && other == 0 {
        return true;
    }
    if td::TILE_BRICK.has(my) && td::TILE_BRICK.has(other) {
        return true;
    }
    td::SET_GEMSPARK_FRAMING_TYPES[other as usize] == td::SET_GEMSPARK_FRAMING_TYPES[my as usize]
}

/// The `Framing.Add8WayLookup` rows (`Framing.cs:45-91`), in `(col, row)`
/// cells of the 18 px grid; a single point serves all three variants
/// (`Framing.cs:157-166`). Unlisted masks are `None` in the C# (a null
/// lookup row); every mask the full-block styles can build is listed.
static SELF_FRAME_8WAY: [Option<[(u8, u8); 3]>; 256] = {
    let mut t = [None; 256];
    macro_rules! row3 {
        ($m:expr, $a:expr, $b:expr, $c:expr) => {
            t[$m] = Some([$a, $b, $c]);
        };
    }
    macro_rules! row1 {
        ($m:expr, $p:expr) => {
            t[$m] = Some([$p, $p, $p]);
        };
    }
    row3!(0, (9, 3), (10, 3), (11, 3)); // Framing.cs:45
    row3!(1, (6, 3), (7, 3), (8, 3)); // :46
    row3!(2, (12, 0), (12, 1), (12, 2)); // :47
    row1!(3, (15, 2)); // :48
    row3!(4, (9, 0), (9, 1), (9, 2)); // :49
    row1!(5, (13, 2)); // :50
    row3!(6, (6, 4), (7, 4), (8, 4)); // :51
    row1!(7, (14, 2)); // :52
    row3!(8, (6, 0), (7, 0), (8, 0)); // :53
    row3!(9, (5, 0), (5, 1), (5, 2)); // :54
    row1!(10, (15, 0)); // :55
    row1!(11, (15, 1)); // :56
    row1!(12, (13, 0)); // :57
    row1!(13, (13, 1)); // :58
    row1!(14, (14, 0)); // :59
    row1!(15, (14, 1)); // :60
    row3!(19, (1, 4), (3, 4), (5, 4)); // :61
    row1!(23, (16, 3)); // :62
    row1!(27, (17, 0)); // :63
    row1!(31, (13, 4)); // :64
    row3!(37, (0, 4), (2, 4), (4, 4)); // :65
    row1!(39, (17, 3)); // :66
    row1!(45, (16, 0)); // :67
    row1!(47, (12, 4)); // :68
    row3!(55, (1, 2), (2, 2), (3, 2)); // :69
    row3!(63, (6, 2), (7, 2), (8, 2)); // :70
    row3!(74, (1, 3), (3, 3), (5, 3)); // :71
    row1!(75, (17, 1)); // :72
    row1!(78, (16, 2)); // :73
    row1!(79, (13, 3)); // :74
    row3!(91, (4, 0), (4, 1), (4, 2)); // :75
    row3!(95, (11, 0), (11, 1), (11, 2)); // :76
    row1!(111, (17, 4)); // :77
    row1!(127, (14, 3)); // :78
    row3!(140, (0, 3), (2, 3), (4, 3)); // :79
    row1!(141, (16, 1)); // :80
    row1!(142, (17, 2)); // :81
    row1!(143, (12, 3)); // :82
    row1!(159, (16, 4)); // :83
    row3!(173, (0, 0), (0, 1), (0, 2)); // :84
    row3!(175, (10, 0), (10, 1), (10, 2)); // :85
    row1!(191, (15, 3)); // :86
    row3!(206, (1, 0), (2, 0), (3, 0)); // :87
    row3!(207, (6, 1), (7, 1), (8, 1)); // :88
    row1!(223, (14, 4)); // :89
    row1!(239, (15, 4)); // :90
    row3!(255, (1, 1), (2, 1), (3, 1)); // :91
    t
};

/// The ids routed to `Framing.SelfFrame8Way` (`WorldGen.cs:84819-84833`'s
/// case labels plus the `385/446/447/448` fallthrough at `:81409-81413`).
#[inline]
fn uses_self_frame_8way(kind: u16) -> bool {
    matches!(kind, 255..=268 | 385 | 446..=448)
}

/// `Framing.SelfFrame8Way` with the caller's `frameNumber` variant
/// (the reset roll lives in the pass functions, `Framing.cs:317-320`).
/// Side slots join only when both blocks present the touching side; a
/// corner joins only when both adjacent sides joined AND the corner block
/// presents both facing sides (`Framing.cs:202-316`).
fn self_frame_8way(world: &World, x: i32, y: i32, variant: usize) -> Option<(i16, i16)> {
    let center = world.tile(x, y)?;
    if !center.active() {
        return None; // Framing.cs:195-198
    }
    let style = find_block_style(center);
    let blends = |dx: i32, dy: i32| -> Option<(bool, bool, bool, bool)> {
        let t = world.tile(x + dx, y + dy)?;
        if t.active() && will_it_blend(center.kind, t.kind) {
            Some(find_block_style(t))
        } else {
            None
        }
    };
    let mut mask = 0u8;
    let mut up_s = (false, false, false, false);
    let mut left_s = up_s;
    let mut right_s = up_s;
    let mut down_s = up_s;
    if style.0 {
        if let Some(s) = blends(0, -1) {
            if s.1 {
                mask |= 1;
                up_s = s;
            }
        }
    }
    if style.2 {
        if let Some(s) = blends(-1, 0) {
            if s.3 {
                mask |= 2;
                left_s = s;
            }
        }
    }
    if style.3 {
        if let Some(s) = blends(1, 0) {
            if s.2 {
                mask |= 4;
                right_s = s;
            }
        }
    }
    if style.1 {
        if let Some(s) = blends(0, 1) {
            if s.0 {
                mask |= 8;
                down_s = s;
            }
        }
    }
    if up_s.2 && left_s.0 {
        if let Some(s) = blends(-1, -1) {
            if s.3 && s.1 {
                mask |= 0x10;
            }
        }
    }
    if up_s.3 && right_s.0 {
        if let Some(s) = blends(1, -1) {
            if s.2 && s.1 {
                mask |= 0x20;
            }
        }
    }
    if down_s.2 && left_s.1 {
        if let Some(s) = blends(-1, 1) {
            if s.3 && s.0 {
                mask |= 0x40;
            }
        }
    }
    if down_s.3 && right_s.1 {
        if let Some(s) = blends(1, 1) {
            if s.2 && s.0 {
                mask |= 0x80;
            }
        }
    }
    // Framing.cs:321: `selfFrame8WayLookup[num][frameNumber]`. Sloped/half
    // masks outside the table are null rows in the C# (it would throw); the
    // port leaves the stored frame untouched for those (deviation recorded
    // in ANALYSIS.md — gemsparks are never hammered in generated worlds).
    let (col, row) = SELF_FRAME_8WAY[mask as usize]?[variant.min(2)];
    Some((col as i16 * 18, row as i16 * 18))
}

/// Computes the frame for one tile. Returns `(frameX, frameY)` in tileset pixels.
///
/// `WorldGen.TileFrame(i, j)` with `resetFrame: false` (`WorldGen.cs:81713`) —
/// the stored `frameNumber` variant is reused. The reset arm (a fresh
/// `genRand.Next(0, 3)` roll, `:81705-81709`) lives in the pass functions
/// ([`frame_world`]/[`reframe_area`]), which store the roll before framing.
pub fn frame_of(world: &World, x: i32, y: i32) -> Option<(i16, i16)> {
    let variant = world.frame_number(world.index(x, y)) as usize;
    frame_of_variant(world, x, y, variant)
}

/// [`frame_of`] with the caller's `frameNumber` variant — the roll-vs-read
/// split the C# makes at `WorldGen.cs:81705-81713`.
pub fn frame_of_variant(world: &World, x: i32, y: i32, variant: usize) -> Option<(i16, i16)> {
    let tile = world.tile(x, y)?;
    if !tile.active() {
        return None;
    }
    // The `TileFrameCosmetic` switch arm (WorldGen.cs:84819-84833): the
    // gemspark ids bypass the merge chain entirely; 385/446/447/448 reach
    // the same arm through the inner-switch break + `goto case 255`
    // (WorldGen.cs:81409-81413, :84818).
    if uses_self_frame_8way(tile.kind) {
        return self_frame_8way(world, x, y, variant);
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

    // W4-g83 (contract §4): the thorn down-merge (`WorldGen.cs:82043-82066`)
    // — cases 32/352/69/655 fold a matching grass neighbour BELOW into the
    // thorn (`down = num`). These are DIRECT `down = num` assignments, not
    // `TileMergeAttempt` calls, so the GenTileData merge parser never emits
    // them and `td::merges` cannot carry them — the hand insertion is the
    // correct home. AFTER the slope adjustments (`:81632-81700` precedes
    // `:82043`): a sloped grass-below cell reads `down = -1` and does not
    // merge. The thorn kinds are not stone/aliases, so `num` is the kind.
    let grass_below: u16 = match me {
        32 => 23,
        352 => 199,
        69 | 655 => 60,
        _ => 0,
    };
    if grass_below != 0 && down == grass_below as i32 {
        down = num;
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
        return Some(grass_frame_of(x, y, num, variant, &mut nb));
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
    // `WorldGen.cs:82070-82082`: the living-wood/leaf merge pairs.
    // * 192: `TileMergeAttemptFrametest(i, j, 192, 191)` — the leaf re-frames
    //   the living-wood neighbour and merges when its frame is open toward the
    //   leaf. The port merges unconditionally (the wood's edge toward a leaf is
    //   open in the common case; the compiled openness probe needs the generic
    //   -2 table, documented in ANALYSIS.md).
    // * 191: `TileMergeAttempt(-2, 192)` — leaf slots become the `-2` marker
    //   ("the neighbour is a leaf", read by the generic -2 tables) — and
    //   `TileMergeAttempt(num, Dirt)` — dirt folds into the wood.
    if me == 192 {
        merge(&mut n, num, |v| v == 191);
    } else if me == 191 {
        for v in n.iter_mut() {
            if *v == 192 {
                *v = -2;
            }
        }
        merge(&mut n, num, |v| td::SET_DIRT.has(v));
    }
    // `WorldGen.cs:82077-82082`: the living-mahogany/leaf pair — the same
    // shape as the living wood above, but 383 merges with MUD (not dirt).
    if me == 384 {
        merge(&mut n, num, |v| v == 383);
    } else if me == 383 {
        for v in n.iter_mut() {
            if *v == 384 {
                *v = -2;
            }
        }
        merge(&mut n, num, |v| td::SET_MUD.has(v));
    }
    // `WorldGen.cs:82040-82042`: case 162 — `TileMergeAttempt(-2,
    // TileID.Sets.IcesSnow, …)` (`TileID.cs:561`: {161, 200, 163, 164,
    // 147}). The IcesSnow neighbours become the `-2` marker, read by the
    // generic -2 side/corner tables (`:83419-83797`) — the thin ice
    // (S1-K-2 wave C) blends into the snow/ice mass. No pre-existing 162
    // tiles exist in the port's worlds, so the arm only fires on the new
    // kind (`SEED42_512` unchanged).
    if me == 162 {
        merge(&mut n, -2, |v| td::SET_ICES_SNOW.has(v));
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

    // The variant is the caller's stored `frameNumber` (`WorldGen.cs:81705-81729`).
    // The `Main.tileLargeFrames` overrides REPLACE it with a fixed per-position
    // pattern without re-storing (`:81719-81729` — unlike the wall side, which
    // stores the lookup value); 3 folds onto 2 because the C# variant `switch`
    // arms have no `case 2` — `default` serves both 2 and 3.
    let variant = match td::LARGE_FRAMES[me as usize] {
        1 => {
            // `WorldGen.cs:81719-81728`, verbatim; the C# values are 1-based.
            const LARGE_4: [[usize; 3]; 4] = [[2, 4, 2], [1, 3, 1], [2, 2, 4], [1, 1, 3]];
            LARGE_4[y.rem_euclid(4) as usize][x.rem_euclid(3) as usize] - 1
        }
        2 => (x.rem_euclid(2) + y.rem_euclid(2) * 2) as usize,
        _ => variant,
    }
    .min(2);

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
///
/// Standing deviation (W4-g83 contract §2, ANALYSIS "The vine draw
/// residuals"): the C# `FrameEverything` runs `TileFrame` per cell — the
/// `CheckVines`/`CheckUnderwaterPlant` dispatch included — while this pass
/// is cosmetic-only (the banded-parallel frame cannot run the killing
/// checks). The vine lifecycle checks ride the edit/growth dispatch
/// (`edit/kill.rs::square_tile_frame`); a generated vine that lost its
/// support stands until the next edit/growth frame touches it.
#[allow(clippy::needless_range_loop)] // `x` also indexes the world, not just `row`
pub fn frame_world(world: &mut World) -> usize {
    let important = world.frame_important.clone();
    let (w, h) = (world.width as usize, world.height as usize);
    if w <= 2 || h <= 2 {
        return 0;
    }

    // C4: frames depend only on kinds/flags/slopes (and the tile's own
    // `frameNumber` variant), never on other tiles' frames, so a band of rows can
    // be framed in parallel. The immutable world is shared by the tasks; results
    // land in the band buffer and are copied in afterwards. 128 rows is
    // ~3.3 MB of scratch for a 6400-wide world.
    const BAND: usize = 128;
    let mut band = Vec::new();
    let mut framed = 0usize;
    let mut y0 = 1usize;
    while y0 + 1 < h {
        let y1 = (y0 + BAND).min(h - 1);
        let rows = y1 - y0;
        band.clear();
        band.resize(rows * w, (i16::MIN, i16::MIN, 0u8));
        {
            let world = &*world;
            band.par_chunks_mut(w).enumerate().for_each(|(row_i, row)| {
                let y = (y0 + row_i) as i32;
                // One roll stream per band: the port's stand-in for the C#'s
                // `WorldGen.genRand.Next(0, 3)` draws interleaved with worldgen
                // (`WorldGen.cs:81705-81709` — vanilla frames incrementally, the
                // port frames in this one pass). Band-offset seeds keep the
                // bands independent; deviation recorded in `docs/ANALYSIS.md`.
                let mut rng =
                    crate::unified_random::UnifiedRandom::new((world.seed as i32).wrapping_add((y0 / BAND) as i32));
                for x in 1..w - 1 {
                    let t = &world.tiles[y as usize * w + x];
                    if !t.active() || important.get(t.kind as usize).copied().unwrap_or(false) {
                        continue;
                    }
                    // Slopes and half bricks frame exactly like full blocks; the draw
                    // pass clips the sprite into the slope/half silhouette.
                    // W4-d109: a `.trwld` v18 load restored the variants — reuse
                    // them (`TileFrame` resetFrame:false, WorldGen.cs:81713);
                    // otherwise the reset roll (`:81705-81709`).
                    let rolled = if world.variants_from_save {
                        world.frame_number(y as usize * w + x) as usize
                    } else {
                        rng.next_range(0, 3) as usize
                    };
                    if let Some((fx, fy)) = frame_of_variant(world, x as i32, y, rolled) {
                        row[x] = (fx, fy, rolled as u8);
                    }
                }
            });
        }
        for (row_i, row) in band.chunks(w).enumerate() {
            let y = y0 + row_i;
            for x in 1..w - 1 {
                let (fx, fy, variant) = row[x];
                if fx == i16::MIN {
                    continue;
                }
                let idx = y * w + x;
                let t = &mut world.tiles[idx];
                t.frame_x = fx;
                t.frame_y = fy;
                world.set_frame_number(idx, variant);
                framed += 1;
            }
        }
        y0 = y1;
    }
    framed
}

// ---------------------------------------------------------------- walls

/// `Framing.wallFrameLookup` — `(column, row)` cells of the 36 px wall grid for each
/// neighbour mask, with the four `wallFrameNumber` variants. Ported from
/// `Framing.AddWallFrameLookup` (`Framing.cs:163-176`) — the fourth point of each row
/// serves the lazure large-frame walls, whose lookup yields 3
/// (`Framing.cs:385-387`, `lazureTileFrameNumberLookup` − 1).
static WALL_FRAMES: [[(u8, u8); 4]; 20] = [
    [(9, 3), (10, 3), (11, 3), (6, 6)],
    [(6, 3), (7, 3), (8, 3), (4, 6)],
    [(12, 0), (12, 1), (12, 2), (12, 5)],
    [(1, 4), (3, 4), (5, 4), (3, 6)],
    [(9, 0), (9, 1), (9, 2), (9, 5)],
    [(0, 4), (2, 4), (4, 4), (2, 6)],
    [(6, 4), (7, 4), (8, 4), (5, 6)],
    [(1, 2), (2, 2), (3, 2), (3, 5)],
    [(6, 0), (7, 0), (8, 0), (6, 5)],
    [(5, 0), (5, 1), (5, 2), (5, 5)],
    [(1, 3), (3, 3), (5, 3), (1, 6)],
    [(4, 0), (4, 1), (4, 2), (4, 5)],
    [(0, 3), (2, 3), (4, 3), (0, 6)],
    [(0, 0), (0, 1), (0, 2), (0, 5)],
    [(1, 0), (2, 0), (3, 0), (1, 5)],
    [(1, 1), (2, 1), (3, 1), (2, 5)],
    [(6, 1), (7, 1), (8, 1), (7, 5)],
    [(6, 2), (7, 2), (8, 2), (8, 5)],
    [(10, 0), (10, 1), (10, 2), (10, 5)],
    [(11, 0), (11, 1), (11, 2), (11, 5)],
];

/// `Framing.centerWallFrameLookup` — offsets added to mask 15 for fully enclosed walls.
static CENTER_WALL_FRAMES: [[u8; 3]; 3] = [[2, 0, 0], [0, 1, 4], [0, 3, 0]];

/// `Framing.phlebasTileFrameNumberLookup` (`Framing.cs:99-104`) — the
/// `wallLargeFrames == 1` variant override, indexed `[j % 4][i % 3]`, 1-based.
static PHLEBAS_WALL_VARIANTS: [[usize; 3]; 4] = [[2, 4, 2], [1, 3, 1], [2, 2, 4], [1, 1, 3]];

/// `Framing.lazureTileFrameNumberLookup` (`Framing.cs:106-109`) — the
/// `wallLargeFrames == 2` variant override, indexed `[i % 2][j % 2]`, 1-based.
static LAZURE_WALL_VARIANTS: [[usize; 2]; 2] = [[1, 3], [2, 4]];

/// The `Framing.WallFrame` variant selection (`Framing.cs:376-399`): the
/// `Main.wallLargeFrames` lookups replace the caller's variant for the large
/// walls (1 = phlebas, 2 = lazure); every other wall keeps it. The caller
/// stores the returned value — the C# re-stores the lookup for large walls and
/// the roll for reset events.
#[inline]
fn wall_variant_of(world: &World, x: i32, y: i32, wall: u16, variant: usize) -> usize {
    match td::WALL_LARGE_FRAMES
        .get(wall as usize)
        .copied()
        .unwrap_or(0)
    {
        1 => PHLEBAS_WALL_VARIANTS[y.rem_euclid(4) as usize][x.rem_euclid(3) as usize] - 1,
        2 => LAZURE_WALL_VARIANTS[x.rem_euclid(2) as usize][y.rem_euclid(2) as usize] - 1,
        _ => variant,
    }
}

/// The reset roll of `Framing.WallFrame` (`Framing.cs:389-395`):
/// `genRand.Next(0, 3)`, with the wall-21 quirk — a second `Next(2)` draw
/// forces variant 2 half the time. Wall 21 is the wood fence's wall id, whose
/// sheet carries no first-variant column.
fn wall_reset_roll(rng: &mut crate::unified_random::UnifiedRandom, wall: u16) -> usize {
    let mut rolled = rng.next_range(0, 3) as usize;
    if wall == 21 && rng.next_max(2) == 0 {
        rolled = 2;
    }
    rolled
}

/// `Framing.WallFrame`'s neighbour test (`Framing.cs:345-376`): a neighbour
/// joins the mask when it carries a wall OR is an active `TruncatesWalls`
/// tile (`TileID.cs:595`: 54/328/459/748 — glass & co. cut the wall edge), and
/// is not echo-coated invisible (`tile2.invisibleWall()`, `Tile.cs:545-548`).
/// The `Main.ShouldShowInvisibleBlocksAndWalls()` override (`Main.cs:58023-
/// 58030`: the EchoMonolith / the 4409 accessory) is unmodelled — the port
/// reads it as false (the `render/tile_cracks.rs` precedent), so an invisible
/// wall never joins a neighbour's mask.
fn wall_mask_neighbour(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    (t.wall != 0 || (t.active() && td::SET_TRUNCATES_WALLS.has(t.kind)))
        && !world.invisible_wall(x, y)
}

/// `Framing.WallFrame`'s head gates (`Framing.cs:328-342`), the mutating
/// half: an out-of-range wall id clamps to 0 (`wall >= WallID.Count`), and a
/// wall-less tile clears its wall paint + coatings (`wallColor(0)` +
/// `ClearWallPaintAndCoating()`, `Tile.cs:911-916` — paint, fullbright and
/// invisible bits; the port's extras byte 0x1f/0x80/0x40). Returns whether
/// the tile still carries a wall to frame.
fn wall_frame_head(world: &mut World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile_mut(x, y) else {
        return false;
    };
    if t.wall as usize >= td::WALL_COUNT {
        t.wall = 0; // Framing.cs:332-335
    }
    if t.wall == 0 {
        let idx = world.index(x, y);
        let b = world.extras_at(idx);
        world.set_extras_at(idx, b & !(0x1f | 0x40 | 0x80)); // Framing.cs:337-342
        return false;
    }
    true
}

/// `Framing.WallFrame` for one tile, with the caller's `wallFrameNumber`
/// variant (the reset roll or the stored read). Returns the 36 px cell and the
/// effective variant (the large-frame lookups replace it), which the caller
/// stores. `None` when there is no wall.
fn wall_frame_of_variant(world: &World, x: i32, y: i32, variant: usize) -> Option<((u8, u8), usize)> {
    let tile = world.tile(x, y)?;
    if tile.wall == 0 || tile.wall as usize >= td::WALL_COUNT {
        // The clamp/reset mutation lives in `wall_frame_head`; this pure
        // read treats the clamped id as wall-less (Framing.cs:332-342).
        return None;
    }
    let mut mask = 0u8;
    if wall_mask_neighbour(world, x, y - 1) {
        mask |= 1;
    }
    if wall_mask_neighbour(world, x - 1, y) {
        mask |= 2;
    }
    if wall_mask_neighbour(world, x + 1, y) {
        mask |= 4;
    }
    if wall_mask_neighbour(world, x, y + 1) {
        mask |= 8;
    }
    if mask == 15 {
        mask += CENTER_WALL_FRAMES[x.rem_euclid(3) as usize][y.rem_euclid(3) as usize];
    }
    let variant = wall_variant_of(world, x, y, tile.wall, variant).min(3);
    Some((WALL_FRAMES[mask as usize][variant], variant))
}

/// `Framing.WallFrame(i, j)` with `resetFrame: false` (`Framing.cs:399`) — the
/// stored `wallFrameNumber` variant is reused. `None` when there is no wall.
pub fn wall_frame_of(world: &World, x: i32, y: i32) -> Option<(u8, u8)> {
    let variant = world.wall_frame_number(world.index(x, y)) as usize;
    Some(wall_frame_of_variant(world, x, y, variant)?.0)
}

/// Re-frames a small square after a tile edit. Only the affected area is touched, so
/// mining/placing does not need a full-world framing pass.
///
/// The `reset` flag is `SquareTileFrame(i, j, resetFrame)`'s (default `true`,
/// `WorldGen.cs:80185`): the centre tile re-rolls its variant from the world's
/// framing stream when set (`TileFrame(i, j, resetFrame)` at `:80191` /
/// `Framing.WallFrame(i, j, resetFrame)` at `:80204`), while the ring re-frames
/// with the stored variants (the `TileFrame`/`WallFrame` calls at `:80180-80208`
/// default to `resetFrame: false`). The square stays a radius-clamped superset
/// of the C# 3×3 ring (the documented deviation in `docs/ANALYSIS.md`).
pub fn reframe_area(world: &mut World, cx: i32, cy: i32, radius: i32, reset: bool) {
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
            let idx = world.index(x, y);
            let (active, kind) = (t.active(), t.kind);
            // `SquareTileFrame`'s centre-only reset: the roll happens where the
            // C# passes `resetFrame` — the centre tile/wall — and is consumed
            // from `world.frame_rng` (the port's `genRand` stand-in).
            let centre = x == cx && y == cy;
            if active && !important.get(kind as usize).copied().unwrap_or(false) {
                let variant = if centre && reset {
                    let rolled = world.frame_rng.next_range(0, 3) as usize;
                    world.set_frame_number(idx, rolled as u8);
                    rolled
                } else {
                    world.frame_number(idx) as usize
                };
                if let Some(f) = frame_of_variant(world, x, y, variant) {
                    tiles.push((x, y, f));
                }
            }
            if active && td::SET_TORCHES.has(kind) {
                torches.push((x, y));
            }
            // `Framing.cs:328-342`: the head runs for EVERY tile (the wall
            // clamp and the wall-less paint/coating reset), before any roll.
            let has_wall = wall_frame_head(world, x, y);
            if has_wall {
                let wall = world.tiles[idx].wall;
                let variant = if centre && reset {
                    wall_reset_roll(&mut world.frame_rng, wall)
                } else {
                    world.wall_frame_number(idx) as usize
                };
                if let Some(((col, row), effective)) = wall_frame_of_variant(world, x, y, variant) {
                    // The large-frame lookups re-store their value even on the
                    // resetFrame=false path (`Framing.cs:378-388` runs before the
                    // reset branch); for normal walls this writes the stored
                    // value back unchanged.
                    world.set_wall_frame_number(idx, effective as u8);
                    walls.push((x, y, (col, row)));
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

/// `WorldGen.SectionTileFrame(startX, startY, endXInclusive, endYInclusive)`
/// (`WorldGen.cs:80343-80366`) — the `11 TileFrameSection` reader's apply
/// (`MessageBuffer.cs:895-899`). `GetSectionBounds` (`:80368-80398`) maps the
/// section rectangle to tile bounds (200x150 sections, clamped to
/// `1..=maxTiles-2`), then EVERY cell in the one-tile-widened rectangle runs
/// `Reframe(i, j, resetFrame: true)` (`:80348-80358`, `:80330-80341` =
/// `TileFrame(reset, noBreak)` + `WallFrame(reset)`): each tile re-rolls its
/// variant from the framing stream (the `reframe_area` centre-roll mechanics,
/// applied per cell). The `Main.mapTime` burst + `MapUpdateQueue.Add` and the
/// `sectionManager.SetSectionFramed` latch are app-side (the caller queues the
/// map rect; the port has no section-framed latch — its framing is incremental).
pub fn section_tile_frame(
    world: &mut World,
    section_start_x: i32,
    section_start_y: i32,
    section_end_x: i32,
    section_end_y: i32,
) {
    // `GetSectionBounds` (`:80370-80397`), verbatim clamps.
    let mut x0 = section_start_x * 200;
    let mut x1 = (section_end_x + 1) * 200;
    let mut y0 = section_start_y * 150;
    let mut y1 = (section_end_y + 1) * 150;
    x0 = x0.clamp(1, world.width - 2);
    y0 = y0.clamp(1, world.height - 2);
    x1 = x1.min(world.width - 2);
    y1 = y1.min(world.height - 2);
    let important = world.frame_important.clone();
    let mut tiles = Vec::new();
    let mut walls = Vec::new();
    let mut torches = Vec::new();
    // `:80348-80358` — the widened rectangle, column-major, reset EVERY cell.
    for x in (x0 - 1)..(x1 + 1) {
        for y in (y0 - 1)..(y1 + 1) {
            let Some(t) = world.tile(x, y) else {
                continue;
            };
            let idx = world.index(x, y);
            let (active, kind, wall) = (t.active(), t.kind, t.wall);
            if active && !important.get(kind as usize).copied().unwrap_or(false) {
                let rolled = world.frame_rng.next_range(0, 3) as usize;
                world.set_frame_number(idx, rolled as u8);
                if let Some(f) = frame_of_variant(world, x, y, rolled) {
                    tiles.push((x, y, f));
                }
            }
            if active && td::SET_TORCHES.has(kind) {
                torches.push((x, y));
            }
            if wall != 0 {
                let rolled = wall_reset_roll(&mut world.frame_rng, wall);
                if let Some(((col, row), effective)) = wall_frame_of_variant(world, x, y, rolled) {
                    world.set_wall_frame_number(idx, effective as u8);
                    walls.push((x, y, (col, row)));
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
    for (x, y) in torches {
        crate::torch::check_torch(world, x, y);
    }
}

/// `WorldGen.GetTileMossColor(tileType)` (`WorldGen.cs:81215-81255`, W4-f25
/// helper C8): the moss tile / moss-brick pairs map to their colour index
/// 0-10 (the loose-moss 184 frame column = `22 * colour`, `TileFrame` case
/// 184 `:85176-85202`); every other kind reads -1 (`None`). WIRED: the
/// case-184 arm lives in `edit/kill.rs::check_long_moss` (the port's
/// `TileFrameImportant` dispatch), which calls this for the four-neighbour
/// colour pick.
pub fn get_tile_moss_color(tile_type: u16) -> Option<i32> {
    Some(match tile_type {
        179 | 512 => 0,
        180 | 513 => 1,
        181 | 514 => 2,
        182 | 515 => 3,
        183 | 516 => 4,
        381 | 517 => 5,
        534 | 535 => 6,
        536 | 537 => 7,
        539 | 540 => 8,
        625 | 626 => 9,
        627 | 628 => 10,
        _ => return None,
    })
}

/// Port of `Framing.WallFrame`: picks the 36 px cell for every wall.
#[allow(clippy::needless_range_loop)] // `x` also indexes the world, not just `row`
pub fn frame_walls(world: &mut World) -> usize {
    let (w, h) = (world.width as usize, world.height as usize);
    if w <= 2 || h <= 2 {
        return 0;
    }

    // C4: wall frames depend only on the wall ids of the four neighbours (and the
    // tile's own variant). Same banded-parallel scheme as `frame_world`, with one
    // roll stream per band standing in for the C#'s `genRand` interleave
    // (`Framing.cs:389-395` — the reset arm; deviation recorded in ANALYSIS.md).
    const BAND: usize = 128;
    let mut band = Vec::new();
    let mut framed = 0usize;
    let mut y0 = 1usize;
    while y0 + 1 < h {
        let y1 = (y0 + BAND).min(h - 1);
        let rows = y1 - y0;
        band.clear();
        band.resize(rows * w, ((u8::MAX, u8::MAX), 0u8));
        {
            let world = &*world;
            band.par_chunks_mut(w).enumerate().for_each(|(row_i, row)| {
                let y = y0 + row_i;
                let mut rng = crate::unified_random::UnifiedRandom::new(
                    (world.seed as i32).wrapping_add((y0 / BAND) as i32),
                );
                for x in 1..w - 1 {
                    let i = y * w + x;
                    let wall = world.tiles[i].wall;
                    // `Framing.cs:332-342`: an out-of-range wall clamps to 0
                    // BEFORE any roll; wall-less tiles return without drawing
                    // from genRand. The clamp/extras reset runs in the copy
                    // phase (`wall_frame_head`).
                    if wall == 0 || wall as usize >= td::WALL_COUNT {
                        continue;
                    }
                    let rolled = if world.variants_from_save {
                        // W4-d109: reuse the stored `wallFrameNumber`
                        // (`WallFrame` resetFrame:false, Framing.cs:399).
                        world.wall_frame_number(i) as usize
                    } else {
                        wall_reset_roll(&mut rng, wall)
                    };
                    if let Some((cell, variant)) =
                        wall_frame_of_variant(world, x as i32, y as i32, rolled)
                    {
                        row[x] = (cell, variant as u8);
                    }
                }
            });
        }
        for (row_i, row) in band.chunks(w).enumerate() {
            let y = y0 + row_i;
            for x in 1..w - 1 {
                let ((col, wall_row), variant) = row[x];
                if col == u8::MAX {
                    // The wall-less / out-of-range cells still run the
                    // `Framing.cs:332-342` head: clamp + paint/coating reset.
                    wall_frame_head(world, x as i32, y as i32);
                    continue;
                }
                let idx = y * w + x;
                let t = &mut world.tiles[idx];
                t.set_wall_frame(col, t.wall_row());
                t.set_wall_frame(t.wall_col(), wall_row);
                world.set_wall_frame_number(idx, variant);
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

    /// S1-P-i33 — `Framing.WallFrame`'s neighbour test (`Framing.cs:345-376`):
    /// an active `TruncatesWalls` tile (54/328/459/748, `TileID.cs:595`) joins
    /// the wall mask even with no wall of its own; an echo-coated invisible
    /// wall drops out. Code-only: written NOT run.
    #[test]
    fn s1_p_i33_truncates_walls_tiles_join_the_wall_mask() {
        // A lone wall at (4,4); a glass tile (54) above it at (4,3).
        let mut w = World::new("t".into(), 9, 9);
        let i = w.index(4, 4);
        w.tiles[i].wall = 2;
        let g = w.index(4, 3);
        w.tiles[g] = crate::tile::Tile { kind: 54, ..Default::default() };
        w.tiles[g].flags |= crate::tile::FLAG_ACTIVE;
        frame_walls(&mut w);
        // The up-neighbour (the glass) joins the mask → bit 1 set. With no
        // other neighbour the mask is 1 → WALL_FRAMES[1] (row 3, not the
        // isolated row 4 the wall-less mask 0 would pick).
        let t = *w.tile(4, 4).unwrap();
        assert_eq!(t.wall_row(), 3, "the glass truncates the wall's top edge");
        // A NON-truncating active tile (stone 1) does NOT join the mask.
        let mut w2 = World::new("t".into(), 9, 9);
        let j = w2.index(4, 4);
        w2.tiles[j].wall = 2;
        let s = w2.index(4, 3);
        w2.tiles[s] = crate::tile::Tile { kind: 1, ..Default::default() };
        w2.tiles[s].flags |= crate::tile::FLAG_ACTIVE;
        frame_walls(&mut w2);
        let t2 = *w2.tile(4, 4).unwrap();
        // Gate V3: the isolated-cell mask 0 picks WALL_FRAMES[0] = (9,3) —
        // row 3 (`Framing.cs:119` AddWallFrameLookup(0, 9,3,...)), not row 4.
        assert_eq!(t2.wall_row(), 3, "stone does not truncate the wall");
    }

    /// S1-P-i33 — an echo-coated invisible wall (`Tile.invisibleWall`,
    /// `Tile.cs:545-548`) drops out of its neighbours' masks
    /// (`Framing.cs:348`'s `!tile2.invisibleWall()` term). The
    /// `ShouldShowInvisibleBlocksAndWalls` override is unmodelled (false).
    #[test]
    fn s1_p_i33_an_invisible_wall_drops_out_of_the_mask() {
        // Two adjacent walls; the right one is echo-coated invisible.
        let mut w = World::new("t".into(), 9, 9);
        let a = w.index(4, 4);
        w.tiles[a].wall = 2;
        let b = w.index(5, 4);
        w.tiles[b].wall = 2;
        w.set_invisible_wall(5, 4, true);
        frame_walls(&mut w);
        // The left wall (4,4) sees its right neighbour as invisible → the
        // right bit (4) is NOT set; with no other neighbour the mask is 0 →
        // the isolated cell WALL_FRAMES[0] = (9,3), row 3 (`Framing.cs:119`).
        let t = *w.tile(4, 4).unwrap();
        assert_eq!(t.wall_row(), 3, "the invisible neighbour does not connect");
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
        let v = w.frame_number(w.index(5, 5)) as i16;
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
        let v = w.frame_number(w.index(5, 4)) as i16;
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
        let v5 = w.frame_number(w.index(5, 5)) as i16;
        let v6 = w.frame_number(w.index(5, 6)) as i16;
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
        let v = w.frame_number(w.index(5, 5)) as i16;
        assert_eq!((t.frame_x, t.frame_y), (108, 216 + v * 36));
    }

    // ---- W4-d110: SelfFrame8Way + the WallFrame head gates ----

    /// W4-d110 — the gemspark 255-268 arm (`WorldGen.cs:84819-84833` →
    /// `Framing.SelfFrame8Way`, `Framing.cs:193-324`). Code-only: written
    /// NOT run. A lone amber gemspark (268) frames isolated: mask 0 →
    /// `Add8WayLookup(0, 9,3, 10,3, 11,3)` (`Framing.cs:45`), variant-scaled.
    #[test]
    fn w4_d110_a_lone_gemspark_uses_the_isolated_8way_cell() {
        let mut w = World::new("t".into(), 11, 11);
        solid(&mut w, 5, 5, 268);
        frame_world(&mut w);
        let t = *w.tile(5, 5).unwrap();
        let v = w.frame_number(w.index(5, 5)) as i16;
        assert_eq!((t.frame_x, t.frame_y), ((9 + v) * 18, 3 * 18));
    }

    /// W4-d110 — `WillItBlend` verbatim (`Framing.cs:180-191`): every
    /// gemspark id is `Main.tileBrick` (`Main.cs:9455-9470`), so the brick
    /// arm blends ANY two gemspark groups (the `GemsparkFramingTypes`
    /// compare at `:190` is only reached for non-brick pairs); a non-brick
    /// non-dirt neighbour with a different framing-type entry (platform 19:
    /// entry 0 vs amber's 268, `TileID.cs:541`) does NOT join. Amber with
    /// amethyst below + platform right → down-only mask 8 →
    /// `Add8WayLookup(8, 6,0, 7,0, 8,0)` (`Framing.cs:53`). Code-only.
    #[test]
    fn w4_d110_gemsparks_blend_through_tile_brick_not_with_non_bricks() {
        let mut w = World::new("t".into(), 11, 11);
        solid(&mut w, 5, 5, 268);
        // A different gemspark group below: amethyst 255 — still blends
        // (both are tileBrick).
        solid(&mut w, 5, 6, 255);
        // A platform (19) to the right: not brick, not dirt, framing-type
        // entry 0 != 268 — never joins.
        solid(&mut w, 6, 5, 19);
        frame_world(&mut w);
        let t = *w.tile(5, 5).unwrap();
        let v = w.frame_number(w.index(5, 5)) as i16;
        assert_eq!((t.frame_x, t.frame_y), ((6 + v) * 18, 0));
    }

    /// W4-d110 — `WillItBlend`'s first arm (`Framing.cs:182-185`): every
    /// gemspark is `ForcedDirtMerging` (`TileID.cs:499`), so plain dirt (0)
    /// joins the mask; the dirt itself frames through the generic path.
    /// Amber with dirt left+right → mask 6 → row 4 (`Framing.cs:51`).
    /// Code-only: written NOT run.
    #[test]
    fn w4_d110_gemspark_blends_into_plain_dirt() {
        let mut w = World::new("t".into(), 11, 11);
        solid(&mut w, 5, 5, 268);
        solid(&mut w, 4, 5, 0);
        solid(&mut w, 6, 5, 0);
        frame_world(&mut w);
        let t = *w.tile(5, 5).unwrap();
        let v = w.frame_number(w.index(5, 5)) as i16;
        assert_eq!((t.frame_x, t.frame_y), ((6 + v) * 18, 4 * 18));
    }

    /// W4-d110 — the `case 385/446/447/448: break` + `goto case 255`
    /// fallthrough (`WorldGen.cs:81409-81413`, `:84818`): the crystal block
    /// and the silly balloons route to SelfFrame8Way too. A lone crystal
    /// block (385) takes the same isolated cell as a gemspark. Code-only.
    #[test]
    fn w4_d110_the_crystal_block_rides_the_8way_arm() {
        let mut w = World::new("t".into(), 11, 11);
        solid(&mut w, 5, 5, 385);
        frame_world(&mut w);
        let t = *w.tile(5, 5).unwrap();
        let v = w.frame_number(w.index(5, 5)) as i16;
        assert_eq!((t.frame_x, t.frame_y), ((9 + v) * 18, 3 * 18));
        // Two crystal blocks side by side join through tileBrick
        // (`Framing.cs:186-189`, both are `Main.tileBrick`): the left cell
        // joins its right side only — mask 4 → col 9, variant row
        // (`Framing.cs:49`).
        solid(&mut w, 6, 5, 385);
        reframe_area(&mut w, 5, 5, 2, false);
        let t = *w.tile(5, 5).unwrap();
        let v = w.frame_number(w.index(5, 5)) as i16;
        assert_eq!((t.frame_x, t.frame_y), ((9) * 18, v * 18));
    }

    /// W4-d110 — `FindBlockStyle`/`blockType` (`Tile.cs:202-214`,
    /// `Framing.cs:92-98`): a half-brick gemspark presents no top, so an
    /// identical neighbour ABOVE never joins (its bottom faces a closed
    /// side); the half brick below still joins through its top. Mask 8 →
    /// row 0 col 6..8 (`Framing.cs:53`). Code-only: written NOT run.
    #[test]
    fn w4_d110_a_half_brick_gemspark_hides_its_top() {
        let mut w = World::new("t".into(), 11, 11);
        solid(&mut w, 5, 5, 268);
        let i = w.index(5, 5);
        w.tiles[i].flags |= crate::tile::FLAG_HALF_BRICK;
        solid(&mut w, 5, 4, 268); // above: closed top, never joins
        solid(&mut w, 5, 6, 268); // below: joins through the half brick's bottom
        frame_world(&mut w);
        let t = *w.tile(5, 5).unwrap();
        let v = w.frame_number(w.index(5, 5)) as i16;
        assert_eq!((t.frame_x, t.frame_y), ((6 + v) * 18, 0));
    }

    /// W4-d110 — `Framing.WallFrame`'s head (`Framing.cs:332-335`): a wall
    /// id >= `WallID.Count` (367) clamps to 0 before framing; the tile ends
    /// wall-less with no frame write. Code-only: written NOT run.
    #[test]
    fn w4_d110_an_out_of_range_wall_clamps_to_zero() {
        let mut w = World::new("t".into(), 9, 9);
        let i = w.index(4, 4);
        w.tiles[i].wall = 400;
        frame_walls(&mut w);
        assert_eq!(w.tile(4, 4).unwrap().wall, 0);
        // The reframe path runs the same head.
        let j = w.index(5, 5);
        w.tiles[j].wall = 367; // exactly WallID.Count — still out of range
        reframe_area(&mut w, 5, 5, 1, true);
        assert_eq!(w.tile(5, 5).unwrap().wall, 0);
    }

    /// W4-d110 — `Framing.WallFrame`'s wall==0 reset (`Framing.cs:337-342`
    /// → `Tile.ClearWallPaintAndCoating`, `Tile.cs:911-916`): a wall-less
    /// tile drops its wall paint AND both wall coatings; the block-side
    /// paint/echo stay untouched. Code-only: written NOT run.
    #[test]
    fn w4_d110_a_wall_less_tile_clears_wall_paint_and_coatings() {
        let mut w = World::new("t".into(), 9, 9);
        w.set_wall_paint(4, 4, 13);
        w.set_invisible_wall(4, 4, true);
        w.set_fullbright_wall(4, 4, true);
        w.set_invisible_block(4, 4, true); // block echo — must survive
        frame_walls(&mut w);
        assert_eq!(w.wall_paint(4, 4), 0);
        assert!(!w.invisible_wall(4, 4));
        assert!(!w.fullbright_wall(4, 4));
        assert!(
            w.invisible_block(4, 4),
            "the block coating is not the wall's"
        );
        // The reframe path runs the reset for every tile in the square.
        w.set_wall_paint(5, 5, 7);
        reframe_area(&mut w, 5, 5, 1, true);
        assert_eq!(w.wall_paint(5, 5), 0);
    }

    /// W4-d110 — the mask-15 offset table (`Framing.cs:401-404`): every
    /// `(i % 3, j % 3)` of the fully enclosed mask adds
    /// `centerWallFrameLookup[i % 3][j % 3]` — pinned against the C#'s
    /// `Initialize` rows (`Framing.cs:111-116`) across all nine offsets.
    /// Code-only: written NOT run.
    #[test]
    fn w4_d110_mask_15_adds_the_center_lookup_for_all_nine_offsets() {
        assert_eq!(CENTER_WALL_FRAMES, [[2, 0, 0], [0, 1, 4], [0, 3, 0]]);
        let mut w = World::new("t".into(), 15, 15);
        for y in 1..14 {
            for x in 1..14 {
                let i = w.index(x, y);
                w.tiles[i].wall = 2;
            }
        }
        frame_walls(&mut w);
        // Interior cells (2..12) are enclosed: mask 15 + the offset.
        for y in 2..12 {
            for x in 2..12 {
                let idx = w.index(x, y);
                let mask = 15 + CENTER_WALL_FRAMES[(x % 3) as usize][(y % 3) as usize];
                let v = w.wall_frame_number(idx) as usize;
                let (col, row) = WALL_FRAMES[mask as usize][v];
                let t = w.tile(x, y).unwrap();
                assert_eq!(
                    (t.wall_col(), t.wall_row()),
                    (col, row),
                    "offset ({},{}) mask {mask}",
                    x % 3,
                    y % 3
                );
            }
        }
    }

    // ---- S1-A-3: the variants come from the C# roll/store/read mechanism ----

    /// The full-world pass rolls a `genRand.Next(0, 3)` variant per framed tile
    /// and stores it (`WorldGen.cs:81705-81709`): every framed tile carries a
    /// 0..=2 `frameNumber`, and two runs over the same seed produce the same
    /// frames and the same stored variants (the C# `resetFrame` stream).
    #[test]
    fn s1_a_3_the_full_pass_rolls_stores_and_replays_the_variant_stream() {
        let build = || {
            let mut w = World::new("t".into(), 24, 24);
            for y in 1..23 {
                for x in 1..23 {
                    solid(&mut w, x, y, 1);
                }
            }
            w
        };
        let mut a = build();
        let mut b = build();
        frame_world(&mut a);
        frame_world(&mut b);
        for y in 1..23 {
            for x in 1..23 {
                let idx = a.index(x, y);
                assert!(a.frame_number(idx) <= 2, "variant must be 0..=2");
                assert_eq!(
                    (a.tiles[idx].frame_x, a.tiles[idx].frame_y),
                    (b.tiles[idx].frame_x, b.tiles[idx].frame_y),
                    "frame at ({x},{y}) differs between the two runs",
                );
                assert_eq!(
                    a.variants[idx],
                    b.variants[idx],
                    "variant at ({x},{y}) differs between the two runs",
                );
            }
        }
        // The roll actually varies: a 22×22 slab cannot land on one variant.
        let spread: std::collections::HashSet<u8> = a
            .variants
            .iter()
            .skip(25)
            .copied()
            .collect();
        assert!(spread.len() > 1, "the roll stream never varied: {spread:?}");
    }

    /// `SquareTileFrame(i, j)` (default `resetFrame: true`, `WorldGen.cs:80185`):
    /// the centre re-rolls its variant from the world's framing stream while the
    /// ring re-frames with the stored values (`:80180-80208` — the ring's
    /// `TileFrame`/`WallFrame` calls default to `resetFrame: false`).
    #[test]
    fn s1_a_3_reframe_reuses_the_ring_and_rerolls_the_centre() {
        let mut w = World::new("t".into(), 24, 24);
        for y in 1..23 {
            for x in 1..23 {
                solid(&mut w, x, y, 1);
            }
        }
        frame_world(&mut w);
        let ring: Vec<(usize, u8, i16, i16)> = [(3, 4), (4, 3), (5, 4), (4, 5), (3, 3), (5, 5)]
            .iter()
            .map(|&(x, y)| {
                let idx = w.index(x, y);
                let t = w.tiles[idx];
                (idx, w.variants[idx], t.frame_x, t.frame_y)
            })
            .collect();
        let centre_idx = w.index(4, 4);

        // The reset roll comes off the world's framing stream.
        w.seed_frame_rng();
        reframe_area(&mut w, 4, 4, 2, true);

        for (idx, variant, fx, fy) in ring {
            assert_eq!(w.variants[idx], variant, "ring variant changed");
            assert_eq!(
                (w.tiles[idx].frame_x, w.tiles[idx].frame_y),
                (fx, fy),
                "ring frame changed"
            );
        }
        // The centre's stored variant is the roll the stream produced now —
        // different from a position hash by construction; assert it is a legal
        // roll and that the frame follows the stored value.
        assert!(w.frame_number(centre_idx) <= 2);
        // And without reset (`resetFrame: false`) nothing re-rolls: the centre
        // keeps its stored variant.
        let before = w.variants[centre_idx];
        reframe_area(&mut w, 4, 4, 2, false);
        assert_eq!(w.variants[centre_idx], before);
    }

    /// `Main.tileLargeFrames == 1` tiles replace the rolled variant with the
    /// fixed per-position lookup (`WorldGen.cs:81719-81728`), without re-storing
    /// it (`:81709` stores the roll). Tile 284 is the C#'s style-1 tile.
    #[test]
    fn s1_a_3_large_frame_tiles_override_the_rolled_variant() {
        let mut w = World::new("t".into(), 24, 24);
        for y in 1..23 {
            for x in 1..23 {
                solid(&mut w, x, y, 1);
            }
        }
        // A 284 block framed against itself.
        for y in 6..10 {
            for x in 6..10 {
                solid(&mut w, x, y, 284);
            }
        }
        assert_eq!(td::LARGE_FRAMES[284], 1);
        frame_world(&mut w);
        // Interior cell at (7, 7): y % 4 == 3, x % 3 == 1 → `LARGE_4[3][1] = 1`
        // (the C# value) − 1 = 0 → interior frame 18 — regardless of the roll.
        let t = w.tile(7, 7).unwrap();
        assert_eq!((t.frame_x, t.frame_y), (18, 18));
        // The stored variant stays the roll (0..=2), never the lookup value.
        assert!(w.frame_number(w.index(7, 7)) <= 2);
    }

    /// `Main.wallLargeFrames == 2` walls (lazure, e.g. wall 185) replace the
    /// variant with `lazureTileFrameNumberLookup[i % 2][j % 2] - 1`
    /// (`Framing.cs:383-387`) and RE-STORE the lookup value — which can be 3,
    /// the fourth lookup column (`Framing.cs:163-176`).
    #[test]
    fn s1_a_3_lazure_walls_store_the_lookup_variant() {
        let mut w = World::new("t".into(), 24, 24);
        for y in 4..12 {
            for x in 4..12 {
                let i = w.index(x, y);
                w.tiles[i].wall = 185;
            }
        }
        assert_eq!(td::WALL_LARGE_FRAMES[185], 2);
        frame_walls(&mut w);
        // Fully enclosed centre at (6, 6): x % 2 == 0, y % 2 == 0 → lookup 1 − 1 = 0.
        assert_eq!(w.wall_frame_number(w.index(6, 6)), 0);
        // (7, 7): lookup 4 − 1 = 3 — the fourth `wallFrameLookup` column.
        assert_eq!(w.wall_frame_number(w.index(7, 7)), 3);
        let cell = w.tile(7, 7).unwrap();
        let mask = 15 + CENTER_WALL_FRAMES[1][1] as u8; // enclosed at odd/odd
        let (col, row) = WALL_FRAMES[mask as usize][3];
        assert_eq!((cell.wall_col(), cell.wall_row()), (col, row));
    }

    /// The wall-21 reset roll draws `Next(0, 3)` **and then** `Next(2)`
    /// (`Framing.cs:390-394`) — the extra draw only for wall 21; a normal wall
    /// consumes exactly one draw (the RNG-order contract, S1-A-10).
    #[test]
    fn s1_a_3_the_wall_21_quirk_consumes_the_extra_draw() {
        use crate::unified_random::UnifiedRandom;
        let mut quirk = UnifiedRandom::new(7);
        let mut mirror = UnifiedRandom::new(7);
        let rolled = wall_reset_roll(&mut quirk, 21);
        let expected = {
            let first = mirror.next_range(0, 3) as usize;
            if mirror.next_max(2) == 0 { 2 } else { first }
        };
        assert_eq!(rolled, expected);
        // The streams stay in lockstep after the two-draw roll.
        assert_eq!(quirk.next(), mirror.next());
        // A normal wall consumes one draw only.
        let mut plain = UnifiedRandom::new(7);
        let mut mirror2 = UnifiedRandom::new(7);
        let _ = wall_reset_roll(&mut plain, 2);
        let _ = mirror2.next_range(0, 3);
        assert_eq!(plain.next(), mirror2.next());
    }

    // ---- W4-d109: the save half — a `.trwld` v18 load reuses the stored variants ----

    /// With `variants_from_save` set (a `.trwld` v18 load), `frame_world`
    /// reuses the stored `frameNumber` (`TileFrame` resetFrame:false,
    /// `WorldGen.cs:81713`) instead of re-rolling: each interior stone cell
    /// takes the frame its stored variant selects, and two runs are identical.
    #[test]
    fn w4_d109_frame_world_reuses_the_saved_tile_variants() {
        let build = || {
            let mut w = World::new("t".into(), 24, 24);
            for y in 1..23 {
                for x in 1..23 {
                    solid(&mut w, x, y, 1);
                }
            }
            // A spread of stored variants across the slab (the save restored them).
            for y in 1..23 {
                for x in 1..23 {
                    let idx = w.index(x, y);
                    w.set_frame_number(idx, ((x + y) % 3) as u8);
                }
            }
            w.variants_from_save = true;
            w
        };
        let mut a = build();
        let mut b = build();
        frame_world(&mut a);
        frame_world(&mut b);
        for y in 2..22 {
            for x in 2..22 {
                let idx = a.index(x, y);
                let v = ((x + y) % 3) as i16;
                // Interior stone: the plain frame row, variant column.
                assert_eq!(
                    (a.tiles[idx].frame_x, a.tiles[idx].frame_y),
                    ([18, 36, 54][v as usize], 18),
                    "frame at ({x},{y}) must follow the stored variant {v}",
                );
                assert_eq!(a.frame_number(idx) as i16, v, "stored variant kept");
                assert_eq!(
                    (a.tiles[idx].frame_x, a.tiles[idx].frame_y),
                    (b.tiles[idx].frame_x, b.tiles[idx].frame_y),
                    "a == b across the two reuse runs",
                );
            }
        }
    }

    /// With `variants_from_save` set, `frame_walls` reuses the stored
    /// `wallFrameNumber` (`WallFrame` resetFrame:false, `Framing.cs:399`): an
    /// enclosed normal wall takes the cell its stored variant selects, and the
    /// value is kept (not re-rolled). Two runs are identical.
    #[test]
    fn w4_d109_frame_walls_reuses_the_saved_wall_variants() {
        let build = || {
            let mut w = World::new("t".into(), 24, 24);
            for y in 4..12 {
                for x in 4..12 {
                    let i = w.index(x, y);
                    w.tiles[i].wall = 2; // a normal wall, no large-frame lookup
                    w.set_wall_frame_number(i, ((x + y) % 3) as u8);
                }
            }
            w.variants_from_save = true;
            w
        };
        let mut a = build();
        let mut b = build();
        frame_walls(&mut a);
        frame_walls(&mut b);
        // Fully enclosed interior cells (mask 15 + the center lookup).
        for y in 5..11 {
            for x in 5..11 {
                let idx = a.index(x, y);
                let v = ((x + y) % 3) as usize;
                assert_eq!(a.wall_frame_number(idx) as usize, v, "stored variant kept");
                let mask = 15 + CENTER_WALL_FRAMES[(x % 3) as usize][(y % 3) as usize] as usize;
                let (col, row) = WALL_FRAMES[mask][v];
                let t = a.tile(x as i32, y as i32).unwrap();
                assert_eq!((t.wall_col(), t.wall_row()), (col, row));
                let tb = b.tile(x as i32, y as i32).unwrap();
                assert_eq!(
                    (t.wall_col(), t.wall_row()),
                    (tb.wall_col(), tb.wall_row()),
                    "a == b across the two reuse runs",
                );
            }
        }
    }

    /// The control: with `variants_from_save` FALSE (a fresh generation), the
    /// full pass re-rolls — the stored variants are overwritten by the stream,
    /// proving the reuse arm is gated on the flag, not unconditional.
    #[test]
    fn w4_d109_a_fresh_world_still_rerolls_the_variants() {
        let mut w = World::new("t".into(), 24, 24);
        for y in 1..23 {
            for x in 1..23 {
                solid(&mut w, x, y, 1);
                let idx = w.index(x, y);
                w.set_frame_number(idx, 0); // a uniform pre-roll the stream must overwrite
            }
        }
        assert!(!w.variants_from_save, "a fresh world rolls");
        frame_world(&mut w);
        let spread: std::collections::HashSet<u8> =
            w.variants.iter().skip(25).copied().collect();
        assert!(spread.len() > 1, "the reset stream must vary the variants");
    }

    /// S1-K-2 wave C: the thin ice (162) carries the `IcesSnow` merge arm
    /// — `WorldGen.cs:82040-82042`'s `TileMergeAttempt(-2, IcesSnow, …)`
    /// marks the 147/161 neighbours `-2` (the generic merge table may
    /// already fold them, exactly as the C# runs both). The arm keeps the
    /// C# structure; the port's generic path reads `-2` slots as open
    /// sides (the compiled generic -2 blend tables stay the grass
    /// family's — the same documented approximation the 383/384 arms
    /// carry). The test pins the structural invariant: a 162 mass frames
    /// without panic, its interior cell takes the plain frame, and the
    /// snow-facing edge resolves to a legal 162 frame (not the isolated
    /// tile). Code-only mode: written NOT run.
    #[test]
    fn s1_k_2c_the_thin_ice_marks_the_snow_neighbours() {
        let mut w = World::new("t".into(), 11, 11);
        // A 162 slab; snow under its bottom row, 162 everywhere else.
        for y in 3..7 {
            for x in 3..8 {
                solid(&mut w, x, y, 162);
            }
        }
        for x in 3..8 {
            solid(&mut w, x, 7, 147);
        }
        frame_world(&mut w);
        // The interior cell keeps the plain frame.
        let t = *w.tile(5, 5).unwrap();
        assert!(
            matches!((t.frame_x, t.frame_y), (18, 18) | (36, 18) | (54, 18)),
            "interior frame was {:?}",
            (t.frame_x, t.frame_y)
        );
        // The snow-facing edge frames against the snow mass — never the
        // isolated-tile frame (162/180/198, 54).
        let t = *w.tile(5, 6).unwrap();
        assert!(
            !matches!((t.frame_x, t.frame_y), (162, 54) | (180, 54) | (198, 54)),
            "snow-facing edge framed as isolated: {:?}",
            (t.frame_x, t.frame_y)
        );
    }

    /// W4-g83 wiring witness (contract §4): the thorn down-merge
    /// (`WorldGen.cs:82043-82066`) is the 4-line insertion AFTER the
    /// own-slope match — a 69 thorn with the jungle grass 60 below folds the
    /// down slot (`down = num`) and frames as the single-open-side-DOWN row
    /// (108/126/144, 0), not the isolated tile; a TOP-SLOPED grass below
    /// reads `down = -1` (the slope adjustments precede the merge) and does
    /// NOT merge. The four pairs ride the same insertion.
    #[test]
    fn w4_g83_the_thorn_down_merge_folds_after_the_slope() {
        let mut w = World::new("t".into(), 11, 11);
        solid(&mut w, 5, 5, 69);
        solid(&mut w, 5, 6, 60); // the jungle grass below
        for v in 0..3 {
            let f = frame_of_variant(&w, 5, 5, v).unwrap();
            assert_eq!(f, ([(108, 0), (126, 0), (144, 0)][v]), "merged, variant {v}");
        }
        // The slope-before-merge order: a top-sloped grass below hides the
        // down slot, so the thorn keeps the isolated frame.
        let i = w.index(5, 6);
        w.tiles[i].set_slope(1);
        let f = frame_of_variant(&w, 5, 5, 0).unwrap();
        assert_eq!(f, (162, 54), "sloped grass below: no merge");
        // The four pairs: 32↔23, 352↔199, 69↔60, 655↔60.
        for (thorn, grass) in [(32u16, 23u16), (352, 199), (69, 60), (655, 60)] {
            let mut w = World::new("t".into(), 11, 11);
            solid(&mut w, 5, 5, thorn);
            solid(&mut w, 5, 6, grass);
            let f = frame_of_variant(&w, 5, 5, 0).unwrap();
            assert_eq!(f, (108, 0), "thorn {thorn} merges grass {grass}");
        }
    }
}
