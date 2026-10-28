//! Tile edits shared by the player, wire tools and (later) NPCs.
//!
//! A local re-frame is enough after a single block change: only the edited tile and its
//! neighbours (up to two cells for the merge rules) can change frames or wall cells.

use crate::framing;
use crate::tile::Tile;
use crate::world::World;

/// Breaks the active tile at `(x, y)`, keeping its background wall. Returns the tile id.
pub fn break_tile(world: &mut World, x: i32, y: i32) -> Option<u16> {
    let t = *world.tile(x, y)?;
    if !t.active() {
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
    after_edit(world, x, y);
    Some(kind)
}

/// Places a block/torch into an empty cell, preserving the wall. The caller checks
/// reach, support and player overlap.
pub fn place_tile(world: &mut World, x: i32, y: i32, kind: u16) -> bool {
    let Some(t) = world.tile(x, y).copied() else {
        return false;
    };
    if t.active() {
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
    after_edit(world, x, y);
    true
}

fn liquid_flags(flags: u8) -> u8 {
    use crate::tile::{FLAG_LIQUID_HONEY, FLAG_LIQUID_LAVA, FLAG_LIQUID_SHIMMER};
    flags & (FLAG_LIQUID_LAVA | FLAG_LIQUID_HONEY | FLAG_LIQUID_SHIMMER)
}

fn after_edit(world: &mut World, x: i32, y: i32) {
    world.revision = world.revision.wrapping_add(1);
    world.mark_chunk(x, y);
    world.recompute_column_surface(x);
    framing::reframe_area(world, x, y, 2);
}

#[cfg(test)]
mod tests {
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
    fn placing_into_an_active_cell_fails() {
        let mut w = ground_world();
        assert!(!place_tile(&mut w, 5, 25, 0));
        assert!(!place_tile(&mut w, 999, 25, 0));
    }
}
