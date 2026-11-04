//! The town-NPC housing — `WorldGen.QuickFindHome`'s three-pass scan
//! (`WorldGen.cs:5759-5805`) and the home assignment (`MoveTownNPC`'s tail,
//! `:4848-4854`), consumed by the homeless town NPCs.

use tr_core::housing::{check_room, room_needs};
use tr_core::world::World;

/// `QuickFindHome` (`WorldGen.cs:5759-5805`): the three-pass room scan
/// around `(tile_x, tile_y)` — the exact tile, the ±1 ring, the ±10 ring
/// with stride 2 — the first pass whose room both scans and furnishes
/// wins. Returns the `(best_x, best_y)` spot.
pub fn quick_find_home(world: &World, tile_x: i32, tile_y: i32) -> Option<(i32, i32)> {
    if tile_x <= 10 || tile_y <= 10 || tile_x >= world.width - 10 || tile_y >= world.height - 10 {
        return None;
    }
    // The pass 1: the exact tile (the scan reads `homeTileY - 1`).
    if let Some((bx, by)) = scan_spot(world, tile_x, tile_y - 1) {
        return Some((bx, by));
    }
    // The pass 2: the ±1 ring.
    for i in (tile_x - 1)..=(tile_x + 1) {
        for j in (tile_y - 1)..=(tile_y + 1) {
            if let Some((bx, by)) = scan_spot(world, i, j) {
                return Some((bx, by));
            }
        }
    }
    // The pass 3: the ±10 ring with stride 2.
    let num = 10;
    for k in (tile_x - num..=(tile_x + num)).step_by(2) {
        for l in (tile_y - num..=(tile_y + num)).step_by(2) {
            if let Some((bx, by)) = scan_spot(world, k, l) {
                return Some((bx, by));
            }
        }
    }
    None
}

/// One scan spot: the room check + the furniture needs.
fn scan_spot(world: &World, x: i32, y: i32) -> Option<(i32, i32)> {
    let scan = check_room(world, x, y).ok()?;
    if !room_needs(&scan.house_tiles) {
        return None;
    }
    // `WorldGen.bestX/bestY` = the scan's centre (the room's found tile).
    Some((x, y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_world_helpers::{build_furnished_room, build_furnished_room_with, ground};

    #[test]
    fn the_homeless_guide_finds_a_furnished_room() {
        // A furnished room in the world: the three-pass scan finds it from
        // the Guide's position up to 10 tiles away.
        let mut w = ground();
        build_furnished_room(&mut w, 60, 100, 12, 9);
        let home = quick_find_home(&w, 72, 108);
        assert!(home.is_some(), "the scan finds the room");
        let (hx, hy) = home.unwrap();
        assert!((60..=71).contains(&hx), "hx {hx}");
        assert!((99..=108).contains(&hy), "hy {hy}");
    }

    #[test]
    fn an_unfurnished_room_rejects() {
        // A room without furniture: the scan fails everywhere.
        let mut w = ground();
        build_furnished_room_with(&mut w, 60, 100, 12, 9, &[]);
        assert!(quick_find_home(&w, 72, 108).is_none());
    }
}
