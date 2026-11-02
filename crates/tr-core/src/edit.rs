//! Tile edits shared by the player, wire tools and (later) NPCs.
//!
//! A local re-frame is enough after a single block change: only the edited tile and its
//! neighbours (up to two cells for the merge rules) can change frames or wall cells.

use crate::framing;
use crate::tile::Tile;
use crate::tiledata as td;
use crate::world::World;

/// `WallID.UnbreakableBlockWall` / `WallID.Count - 1` region used by the dungeon
/// control lines. Vanilla refuses it in every removal/replacement path:
/// `Player.CanPlayerSmashWall` (hammering), `WorldGen.ReplaceWall`, and it makes any
/// tile in front of it unbreakable (`WorldGen.CanKillTile`, `IsTileReplaceable`).
pub const WALL_UNBREAKABLE: u16 = 350;

/// Breaks the active tile at `(x, y)`, keeping its background wall. Returns the tile id.
pub fn break_tile(world: &mut World, x: i32, y: i32) -> Option<u16> {
    let kind = remove_tile(world, x, y)?;
    after_edit(world, x, y);
    Some(kind)
}

/// Clears the active tile at `(x, y)` and keeps the wall and the liquid. The caller is
/// responsible for re-framing; `break_tile` is the version that does it.
pub(crate) fn remove_tile(world: &mut World, x: i32, y: i32) -> Option<u16> {
    let t = *world.tile(x, y)?;
    if !t.active() || t.wall == WALL_UNBREAKABLE {
        return None;
    }
    let kind = t.kind;
    let i = world.index(x, y);
    world.tiles[i] = Tile {
        wall: t.wall,
        liquid: t.liquid,
        flags: liquid_flags(t.flags),
        ..Default::default()
    };
    world.revision = world.revision.wrapping_add(1);
    world.mark_chunk(x, y);
    Some(kind)
}

/// Removes the background wall at `(x, y)`. Returns the wall id.
pub fn break_wall(world: &mut World, x: i32, y: i32) -> Option<u16> {
    let t = *world.tile(x, y)?;
    if t.wall == 0 || t.wall == WALL_UNBREAKABLE {
        return None;
    }
    let wall = t.wall;
    let i = world.index(x, y);
    world.tiles[i].wall = 0;
    world.tiles[i].set_wall_frame(0, 0);
    after_wall_edit(world, x, y);
    Some(wall)
}

/// Places a background wall into a cell that has none. The caller checks reach and
/// support; a wall may sit behind an active tile.
pub fn place_wall(world: &mut World, x: i32, y: i32, wall: u16) -> bool {
    let Some(t) = world.tile(x, y).copied() else {
        return false;
    };
    if t.wall != 0 || wall == 0 {
        return false;
    }
    let i = world.index(x, y);
    world.tiles[i].wall = wall;
    world.tiles[i].set_wall_frame(0, 0);
    after_wall_edit(world, x, y);
    true
}

/// Places a block/torch into an empty cell, preserving the wall. The caller checks
/// reach, support and player overlap.
pub fn place_tile(world: &mut World, x: i32, y: i32, kind: u16) -> bool {
    let Some(t) = world.tile(x, y).copied() else {
        return false;
    };
    if t.active() || t.wall == WALL_UNBREAKABLE {
        return false;
    }
    let i = world.index(x, y);
    let mut tile = Tile {
        kind,
        wall: t.wall,
        liquid: t.liquid,
        frame_x: -1,
        frame_y: -1,
        flags: liquid_flags(t.flags),
        ..Default::default()
    };
    tile.flags |= crate::tile::FLAG_ACTIVE;
    if kind == 4 {
        tile.frame_x = 0;
        tile.frame_y = 0;
        if world.frame_important.len() > 4 {
            world.frame_important[4] = true;
        }
    }
    world.tiles[i] = tile;
    touch(world, x, y);
    // `WorldGen.TileFrame` runs `CheckTorch` for torch tiles: the frame follows the
    // attachment, and an unsupported torch is removed again (the caller then keeps the
    // item instead of consuming it).
    let kept = !td::SET_TORCHES.has(kind) || crate::torch::check_torch(world, x, y);
    after_edit(world, x, y);
    kept
}

/// The item-use placement route (`Player.PlaceThing_Tiles_TryPlacing`,
/// `Player.cs:39746-39788`, entering `PlaceThing_Tiles_PlaceIt` :40108): a tile
/// with registered `TileObjectData` (`TileObjectData.CustomPlace`,
/// `TileObjectData.cs:5016`) is placed through `TileObject.CanPlace`/`Place`;
/// every other tile keeps the legacy single-tile write of `WorldGen.PlaceTile`
/// (`WorldGen.cs:59342`). `style` is the item's `placeStyle`, `dir` the player
/// direction (`Player.direction`: -1 left, 1 right). `false` = refused, the
/// world is unchanged.
pub fn place_item_tile(world: &mut World, x: i32, y: i32, kind: u16, style: i32, dir: i32) -> bool {
    // Registry route: the whole wooden set lands here.
    if let Some(def) = crate::tileobject::def_for_tile(kind) {
        let Some(instance) = crate::tileobject::can_place(world, x, y, def, style, dir, None)
        else {
            return false;
        };
        if !crate::tileobject::place(world, &instance) {
            return false;
        }
        // `WorldGen.SquareTileFrame(tileTargetX, tileTargetY)`
        // (`Player.cs:40121`); `reframe_area` leaves `frame_important` tiles
        // intact, so the object's stored frames survive (DW-1.6 guarantee).
        framing::reframe_area(world, x, y, 2);
        return true;
    }
    // Legacy single-tile route (`WorldGen.PlaceTile`).
    place_tile(world, x, y, kind)
}

fn liquid_flags(flags: u8) -> u8 {
    use crate::tile::{FLAG_LIQUID_HONEY, FLAG_LIQUID_LAVA, FLAG_LIQUID_SHIMMER};
    flags & (FLAG_LIQUID_LAVA | FLAG_LIQUID_HONEY | FLAG_LIQUID_SHIMMER)
}

/// Marks the cell as changed for the caches (`World::revision`, chunk occupancy).
fn touch(world: &mut World, x: i32, y: i32) {
    world.revision = world.revision.wrapping_add(1);
    world.mark_chunk(x, y);
}

fn after_wall_edit(world: &mut World, x: i32, y: i32) {
    touch(world, x, y);
    framing::reframe_area(world, x, y, 2);
}

fn after_edit(world: &mut World, x: i32, y: i32) {
    world.recompute_column_surface(x);
    framing::reframe_area(world, x, y, 2);
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)] // DW-IDs are part of the test names (traceability)
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    fn ground_world() -> World {
        let mut w = World::new("t".into(), 32, 32);
        w.frame_important = vec![false; 754];
        for x in 0..32 {
            for y in 20..32 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    #[test]
    fn break_then_place_round_trips_a_tile() {
        let mut w = ground_world();
        assert_eq!(break_tile(&mut w, 10, 20), Some(1));
        assert!(!w.tile(10, 20).unwrap().active());
        assert_eq!(w.surface_of(10), 21);
        assert!(place_tile(&mut w, 10, 20, 0));
        let t = *w.tile(10, 20).unwrap();
        assert!(t.active() && t.kind == 0);
        // Local framing assigned a real frame from the generic table.
        assert!(t.frame_x >= 0 && t.frame_y >= 0);
        assert_eq!(w.surface_of(10), 20);
        assert!(w.revision >= 2);
    }

    #[test]
    fn torch_keeps_its_explicit_frame() {
        let mut w = ground_world();
        assert!(place_tile(&mut w, 5, 19, 4));
        let t = *w.tile(5, 19).unwrap();
        assert!(t.active());
        assert_eq!((t.frame_x, t.frame_y), (0, 0));
    }

    #[test]
    fn walls_place_and_break_and_reframe_neighbours() {
        let mut w = ground_world();
        assert!(place_wall(&mut w, 10, 15, 2));
        assert_eq!(w.tile(10, 15).unwrap().wall, 2);
        assert!(!place_wall(&mut w, 10, 15, 1), "already walled");
        assert!(place_wall(&mut w, 11, 15, 2));
        // The shared edge reframes to a connecting cell.
        assert_ne!(w.tile(10, 15).unwrap().wall_col(), 0);
        assert_eq!(break_wall(&mut w, 10, 15), Some(2));
        assert_eq!(w.tile(10, 15).unwrap().wall, 0);
        assert_eq!(break_wall(&mut w, 10, 15), None);
    }

    #[test]
    fn placing_into_an_active_cell_fails() {
        let mut w = ground_world();
        assert!(!place_tile(&mut w, 5, 25, 0));
        assert!(!place_tile(&mut w, 999, 25, 0));
    }

    #[test]
    fn unbreakable_wall_blocks_wall_and_tile_edits() {
        let mut w = ground_world();
        // Wall 350 (`WallID.UnbreakableBlockWall`) cannot be hammered away
        // (`Player.CanPlayerSmashWall`).
        let i = w.index(10, 15);
        w.tiles[i].wall = WALL_UNBREAKABLE;
        w.compute_chunks();
        assert_eq!(break_wall(&mut w, 10, 15), None);
        assert_eq!(w.tile(10, 15).unwrap().wall, WALL_UNBREAKABLE);
        // A tile in front of it cannot be mined (`WorldGen.CanKillTile`,
        // `IsTileReplaceable`), and nothing may be placed into that cell.
        let i = w.index(10, 20);
        w.tiles[i].wall = WALL_UNBREAKABLE;
        w.compute_chunks();
        assert_eq!(break_tile(&mut w, 10, 20), None);
        assert!(w.tile(10, 20).unwrap().active());
        assert_eq!(break_wall(&mut w, 10, 20), None);
        assert_eq!(w.tile(10, 20).unwrap().wall, WALL_UNBREAKABLE);
    }

    // ---- Phase 2 (DW-2.1 / DW-2.4): the item-use placement route -----------

    /// 40x40 world with a stone floor at y = 20 — deep enough inside the world
    /// for `CanPlace`'s 5-tile edge margin (`TileObject.cs:215`).
    fn object_world() -> World {
        let mut w = World::new("t".into(), 40, 40);
        w.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        for x in 0..40 {
            let i = w.index(x, 20);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    /// `object_world` plus a stone ceiling at y = 16 — a 3-tall doorway
    /// opening (rows 17..=19) for the door tests.
    fn doorway_world() -> World {
        let mut w = object_world();
        for x in 0..40 {
            let i = w.index(x, 16);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    fn set_active(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    #[test]
    fn test_DW_2_1_object_kinds_route_through_the_registry() {
        // The work bench (tile 18, registry def 2x1) fills both cells through
        // the TileObject route; the legacy single-tile write would leave the
        // second cell inactive.
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 18, 0, 1));
        for x in [10, 11] {
            let t = w.tile(x, 19).unwrap();
            assert!(t.active() && t.kind == 18, "bench cell ({x}, 19)");
        }
    }

    #[test]
    fn test_DW_2_1_non_registry_kinds_keep_the_legacy_path() {
        // Dirt (tile 0) has no registry entry: the legacy single-tile write
        // runs, generic framing assigned.
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 5, 19, 0, 0, 1));
        let t = *w.tile(5, 19).unwrap();
        assert!(t.active() && t.kind == 0);
        assert!(t.frame_x >= 0 && t.frame_y >= 0);
        // The torch keeps its explicit frame through the same fallback
        // (`WorldGen.PlaceTile` case 4 lives in the legacy path).
        assert!(place_item_tile(&mut w, 7, 19, 4, 0, 1));
        let torch = w.tile(7, 19).unwrap();
        assert_eq!((torch.frame_x, torch.frame_y), (0, 0));
    }

    #[test]
    fn test_DW_2_4_route_place_frames_and_frame_important() {
        // Bench frames: `coordinate_full_width` 18, style 0
        // (`TileObjectData.cs:2818-2825`) — columns (0, 0) and (18, 0).
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 18, 0, 1));
        assert_eq!(w.tile(10, 19).unwrap().frame_x, 0);
        assert_eq!(w.tile(11, 19).unwrap().frame_x, 18);
        assert_eq!(w.tile(10, 19).unwrap().frame_y, 0);
        // `place` marks the whole object `frame_important` (Phase 1 DW-1.3
        // guarantee, exercised through the route).
        assert!(w.frame_important[18]);

        // Chest (tile 21): 2x2, origin (0,1) — targeting (14, 19) fills
        // rows 18..=19 with frames x 0/18, y 0/18.
        assert!(place_item_tile(&mut w, 14, 19, 21, 0, 1));
        assert_eq!(w.tile(14, 18).unwrap().frame_x, 0);
        assert_eq!(w.tile(15, 18).unwrap().frame_x, 18);
        assert_eq!(w.tile(14, 19).unwrap().frame_y, 18);
        assert!(w.tile(15, 19).unwrap().active() && w.tile(15, 19).unwrap().kind == 21);
    }

    #[test]
    fn test_DW_2_4_chair_direction_picks_the_alternate() {
        // Facing right, the `PlaceRight` alternate wins (`TileObject.cs:209`):
        // placement style 1 → frame_x 18. Facing left keeps the base (0).
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 15, 0, 1));
        assert!(w.tile(10, 18).unwrap().active() && w.tile(10, 19).unwrap().active());
        assert_eq!(w.tile(10, 18).unwrap().frame_x, 18);
        let mut w2 = object_world();
        assert!(place_item_tile(&mut w2, 10, 19, 15, 0, -1));
        assert_eq!(w2.tile(10, 18).unwrap().frame_x, 0);
    }

    #[test]
    fn test_DW_2_4_door_click_points() {
        // Targeting the middle cell of the opening resolves the mid
        // alternate; the whole 3-tall door is written with column frames
        // (0, 18, 36 — `coordinate_heights` 16 + padding 2).
        let mut w = doorway_world();
        assert!(place_item_tile(&mut w, 10, 18, 10, 0, 1));
        for (y, frame_y) in [(17i32, 0i16), (18, 18), (19, 36)] {
            let t = w.tile(10, y).unwrap();
            assert!(t.active() && t.kind == 10, "door cell {y}");
            assert_eq!(t.frame_y, frame_y, "door frame row {y}");
        }
        // Targeting the bottom cell resolves the bottom-origin alternate.
        let mut w2 = doorway_world();
        assert!(place_item_tile(&mut w2, 10, 19, 10, 0, 1));
        for y in 17..=19 {
            assert!(w2.tile(10, y).unwrap().active(), "door cell {y}");
        }
    }

    #[test]
    fn test_DW_2_4_partial_overlap_refuses_without_world_change() {
        // A block occupies one of the bench's two cells: the route refuses and
        // nothing is written — not even the free origin cell.
        let mut w = object_world();
        set_active(&mut w, 11, 19, 1);
        let before = w.revision;
        assert!(!place_item_tile(&mut w, 10, 19, 18, 0, 1));
        assert!(!w.tile(10, 19).unwrap().active(), "origin cell untouched");
        assert_eq!(w.tile(11, 19).unwrap().kind, 1, "occupant untouched");
        assert_eq!(w.revision, before, "no world change on refusal");
    }

    #[test]
    fn test_DW_2_4_world_edge_refuses() {
        // `CanPlace`'s 5-tile margin (`TileObject.cs:215`) rejects x = 3.
        let mut w = object_world();
        assert!(!place_item_tile(&mut w, 3, 19, 18, 0, 1));
        assert!(!w.tile(3, 19).unwrap().active());
        assert!(!place_item_tile(&mut w, 200, 19, 18, 0, 1), "out of world");
    }

    #[test]
    fn place_style_shifts_the_frame_column() {
        // The item's `placeStyle` feeds `CalculatePlacementStyle`
        // (`TileObjectData.cs:1990-1998`): style 1 shifts the horizontal
        // style column by the bench's 36 px full width (2 cells x 18).
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 18, 1, 1));
        assert_eq!(w.tile(10, 19).unwrap().frame_x, 36);
        assert_eq!(w.tile(11, 19).unwrap().frame_x, 54);
    }
}
