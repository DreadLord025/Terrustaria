//! `WorldGen.CheckTorch`: the attachment frame of every torch, and its removal when
//! nothing supports it.
//!
//! Ported from `WorldGen.CheckTorch` (`WorldGen.cs`) together with the helpers it calls:
//! `WorldGen.TopEdgeCanBeAttachedTo`, `WorldGen.PlatformProperTopFrame` and
//! `WorldGen.IsTreeType`. `TileFrame` runs this for `TileID.Sets.Torches` tiles, so the
//! frame depends on the neighbours, not on the stored save frame.
//!
//! The three small tables below are literal extractions of `Main.tileNoAttach[...] = true`,
//! `TileID.Sets.IsBeam` and `TileID.Sets.IsATreeTrunk`. `tools/GenTileData` cannot emit
//! them yet: the current decompilation writes `Main.`-prefixed assignments and lives under
//! `decompiled/Terraria/ID/`, which the generator's regexes do not match (see
//! `docs/ANALYSIS.md`, "Observations / risks").

use crate::tiledata as td;
use crate::world::World;

/// `Main.tileNoAttach`: tiles that never accept an attachment (trees, platforms, doors…).
const NO_ATTACH: &[u16] = &[
    3, 4, 10, 13, 14, 15, 16, 17, 18, 19, 20, 21, 27, 50, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95,
    96, 97, 98, 99, 101, 102, 110, 114, 134, 387, 388, 390, 427, 441, 467, 468, 469, 486, 487, 488,
    489, 490, 497, 564, 565, 568, 569, 570, 572, 580, 590, 593, 594, 595, 615, 620, 704, 707,
];

/// `TileID.Sets.IsBeam`.
const BEAMS: &[u16] = &[124, 561, 574, 575, 576, 577, 578];

/// `TileID.Sets.IsATreeTrunk` (`WorldGen.IsTreeType`).
const TREE_TRUNKS: &[u16] = &[5, 72, 583, 584, 585, 586, 587, 588, 589, 596, 616, 634];

/// `TileObjectData.PlatformFrameWidth()`.
const PLATFORM_FRAME_WIDTH: i32 = 18;

fn is_solid(kind: u16) -> bool {
    td::TILE_SOLID.has(kind)
}

fn no_attach(kind: u16) -> bool {
    NO_ATTACH.contains(&kind)
}

fn is_beam(kind: u16) -> bool {
    BEAMS.contains(&kind)
}

fn is_tree_type(kind: i32) -> bool {
    kind >= 0 && TREE_TRUNKS.contains(&(kind as u16))
}

/// `WorldGen.PlatformProperTopFrame`.
fn platform_proper_top_frame(frame_x: i16) -> bool {
    let n = frame_x as i32 / PLATFORM_FRAME_WIDTH;
    if (n < 0 || n > 7) && (n < 12 || n > 16) {
        return n >= 25 && n <= 26;
    }
    true
}

/// `WorldGen.TopEdgeCanBeAttachedTo`.
fn top_edge_can_be_attached_to(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && (td::TILE_SOLID.has(t.kind) || td::TILE_SOLID_TOP.has(t.kind))
        && (!t.top_slope()
            || (td::SET_PLATFORMS.has(t.kind) && platform_proper_top_frame(t.frame_x)))
        && !t.half_brick()
}

/// `WorldGen.CheckTorch`: sets `frameX` from the attachment and kills the torch when
/// nothing holds it. Returns `false` when the torch was removed.
pub fn check_torch(world: &mut World, x: i32, y: i32) -> bool {
    let Some(tile) = world.tile(x, y).copied() else {
        return false;
    };
    // The original returns early when any 3x3 neighbour is outside the world.
    for j in y - 1..=y + 1 {
        for i in x - 1..=x + 1 {
            if world.tile(i, j).is_none() {
                return true;
            }
        }
    }

    let below = world.tile(x, y + 1).copied().unwrap_or_default();
    let left = world.tile(x - 1, y).copied().unwrap_or_default();
    let right = world.tile(x + 1, y).copied().unwrap_or_default();
    let below_left = world.tile(x - 1, y + 1).copied().unwrap_or_default();
    let below_right = world.tile(x + 1, y + 1).copied().unwrap_or_default();
    let above_left = world.tile(x - 1, y - 1).copied().unwrap_or_default();
    let above_right = world.tile(x + 1, y - 1).copied().unwrap_or_default();

    // `short num = (tile.frameX >= 66) ? 66 : 0;` — the hanging/tall torch variants keep
    // their 66 offset when the attachment changes.
    let num: i16 = if tile.frame_x >= 66 { 66 } else { 0 };

    let mut num2: i32 = -1; // below: stands on this
    if below.active()
        && ((td::SET_PLATFORMS.has(below.kind) && top_edge_can_be_attached_to(world, x, y + 1))
            || (!below.half_brick() && !below.top_slope()))
    {
        num2 = below.kind as i32;
    }
    let mut num3: i32 = -1; // left: hangs on this
    if left.active() && (left.slope() == 0 || left.slope() % 2 != 1) {
        num3 = left.kind as i32;
    }
    let mut num4: i32 = -1; // right: hangs on this
    if right.active() && (right.slope() == 0 || right.slope() % 2 != 0) {
        num4 = right.kind as i32;
    }
    let tree = if below_left.active() {
        below_left.kind as i32
    } else {
        -1
    };
    let tree2 = if below_right.active() {
        below_right.kind as i32
    } else {
        -1
    };
    let tree3 = if above_left.active() {
        above_left.kind as i32
    } else {
        -1
    };
    let tree4 = if above_right.active() {
        above_right.kind as i32
    } else {
        -1
    };

    let frame_x = if num2 >= 0
        && is_solid(num2 as u16)
        && (!no_attach(num2 as u16) || td::SET_PLATFORMS.has(num2 as u16))
    {
        num
    } else if (num3 >= 0 && is_solid(num3 as u16) && !no_attach(num3 as u16))
        || (num3 >= 0 && is_beam(num3 as u16))
        || (is_tree_type(num3) && is_tree_type(tree3) && is_tree_type(tree))
    {
        22 + num
    } else if (num4 >= 0 && is_solid(num4 as u16) && !no_attach(num4 as u16))
        || (num4 >= 0 && is_beam(num4 as u16))
        || (is_tree_type(num4) && is_tree_type(tree4) && is_tree_type(tree2))
    {
        44 + num
    } else if tile.wall > 0 {
        num
    } else {
        // `WorldGen.KillTile(x, y)`: the torch drops and the cell keeps its wall.
        crate::edit::remove_tile(world, x, y);
        return false;
    };

    let idx = world.index(x, y);
    let t = &mut world.tiles[idx];
    t.frame_x = frame_x;
    t.frame_y = 0;
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    fn world() -> World {
        let mut w = World::new("torch".into(), 32, 32);
        w.frame_important = vec![true; td::TILE_COUNT];
        w
    }

    fn put(w: &mut World, x: i32, y: i32, kind: u16) -> crate::tile::Tile {
        let i = w.index(x, y);
        let mut t = crate::tile::Tile::default();
        t.kind = kind;
        t.flags |= FLAG_ACTIVE;
        w.tiles[i] = t;
        t
    }

    fn torch(w: &mut World, x: i32, y: i32) {
        put(w, x, y, 4);
    }

    fn frame_x(w: &World, x: i32, y: i32) -> i16 {
        w.tile(x, y).unwrap().frame_x
    }

    #[test]
    fn standing_torch_uses_frame_zero() {
        let mut w = world();
        put(&mut w, 10, 11, 1); // dirt below
        torch(&mut w, 10, 10);
        assert!(check_torch(&mut w, 10, 10));
        assert_eq!(frame_x(&w, 10, 10), 0);
    }

    #[test]
    fn torch_on_left_wall_uses_22() {
        let mut w = world();
        put(&mut w, 9, 10, 1);
        torch(&mut w, 10, 10);
        assert!(check_torch(&mut w, 10, 10));
        assert_eq!(frame_x(&w, 10, 10), 22);
    }

    #[test]
    fn torch_on_right_wall_uses_44() {
        let mut w = world();
        put(&mut w, 11, 10, 1);
        torch(&mut w, 10, 10);
        assert!(check_torch(&mut w, 10, 10));
        assert_eq!(frame_x(&w, 10, 10), 44);
    }

    #[test]
    fn torch_in_a_wall_cell_uses_frame_zero() {
        let mut w = world();
        torch(&mut w, 10, 10);
        let i = w.index(10, 10);
        w.tiles[i].wall = 2;
        assert!(check_torch(&mut w, 10, 10));
        assert_eq!(frame_x(&w, 10, 10), 0);
    }

    #[test]
    fn unsupported_torch_is_removed() {
        let mut w = world();
        torch(&mut w, 10, 10);
        assert!(!check_torch(&mut w, 10, 10));
        assert!(!w.tile(10, 10).unwrap().active());
    }

    #[test]
    fn no_attach_neighbour_does_not_hold_a_torch() {
        let mut w = world();
        // 4 is a torch: solid? no. Use a no-attach solid: 19 (platform-ish is solid top).
        // 387 is solid and `tileNoAttach`, so it must not hold a side torch.
        put(&mut w, 9, 10, 387);
        torch(&mut w, 10, 10);
        assert!(!check_torch(&mut w, 10, 10), "no-attach tiles never hold");
    }

    #[test]
    fn platform_below_holds_a_torch() {
        let mut w = world();
        put(&mut w, 10, 11, 19); // platform
        torch(&mut w, 10, 10);
        assert!(check_torch(&mut w, 10, 10));
        assert_eq!(frame_x(&w, 10, 10), 0);
    }

    #[test]
    fn half_brick_below_does_not_hold_a_torch() {
        let mut w = world();
        let mut brick = put(&mut w, 10, 11, 1);
        brick.flags |= crate::tile::FLAG_HALF_BRICK;
        let i = w.index(10, 11);
        w.tiles[i] = brick;
        torch(&mut w, 10, 10);
        assert!(!check_torch(&mut w, 10, 10), "half bricks have no top edge");
    }

    #[test]
    fn tall_variant_keeps_its_66_offset() {
        let mut w = world();
        put(&mut w, 10, 11, 1);
        torch(&mut w, 10, 10);
        let i = w.index(10, 10);
        w.tiles[i].frame_x = 66;
        assert!(check_torch(&mut w, 10, 10));
        assert_eq!(frame_x(&w, 10, 10), 66);
    }
}
