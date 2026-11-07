//! The town-NPC housing checker — `WorldGen.StartRoomCheck`/`CheckRoom`
//! (`WorldGen.cs:6041-6243`) plus `WorldGen.RoomNeeds` (`:5717-5756`) and
//! the `TileID.Sets.RoomNeeds.*` furniture sets (`TileID.cs:148-169`).
//!
//! The move-in evaluation (`Main.UpdateTime_SpawnTownNPCs`) lands with the
//! town-NPC kinds; this module is the pure room validator they consume.

use crate::tiledata::TILE_SOLID;
use crate::world::World;

/// `WorldGen.maxRoomTiles` (`WorldGen.cs:86924`).
pub const MAX_ROOM_TILES: i32 = 750;
/// `WorldGen.maxRoomSize` (`WorldGen.cs:86926`).
pub const MAX_ROOM_SIZE: i32 = 100;
/// `StartRoomCheck`'s small-room floor (`:6101-6105`).
pub const MIN_ROOM_TILES: i32 = 60;

/// `Main.wallHouse` (`Main.cs:10403-10470` + `:9741-9860`): the walls a
/// house may be enclosed by.
pub fn is_house_wall(wall: u16) -> bool {
    matches!(
        wall,
        23..=27 | 29..=39 | 41..=47 | 60 | 66..=68 | 72..=74 | 84 | 85 | 88..=93 | 100..=128
            | 173 | 179 | 181 | 183 | 184 | 186 | 224..=227 | 231..=235 | 237..=241 | 245
            | 312 | 313 | 315..=317 | 323..=346
    )
}

/// `TileID.Sets.HousingWalls` (`TileID.cs:603`): the tile kinds that count
/// as walls for the ±2 scan (the gates).
fn is_housing_tile(kind: u16) -> bool {
    matches!(kind, 11 | 386 | 389)
}

/// `TileID.Sets.RoomNeeds.CountsAsChairTypes` (`TileID.cs:150`).
pub const CHAIR_TYPES: &[u16] = &[15, 79, 89, 102, 487, 497];
/// `CountsAsTableTypes` (`:152-157`).
pub const TABLE_TYPES: &[u16] = &[14, 18, 87, 88, 90, 101, 354, 355, 464, 469, 487, 699];
/// `CountsAsTorchTypes` (`:158-164`).
pub const TORCH_TYPES: &[u16] = &[
    4, 33, 34, 35, 42, 49, 93, 95, 98, 100, 149, 173, 174, 270, 271, 316, 317, 318, 92, 372, 646,
    405, 592, 572, 581, 660,
];
/// `CountsAsDoorTypes` (`:165-168`).
pub const DOOR_TYPES: &[u16] = &[10, 11, 19, 386, 387, 388, 389, 427, 435, 436, 437, 438, 439];

/// `TownNPCRoomCheckFailureReason` — the failure taxonomy the C# records.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RoomFailure {
    TooCloseToWorldEdge,
    StartedInASolidTile,
    RoomIsTooBig,
    RoomIsTooSmall,
    HoleInWallIsTooBig,
    TooManyUnsafeWalls,
}

/// The successful scan's output — the bounds, the tile count and the
/// `houseTile` table `RoomNeeds` reads.
pub struct RoomScan {
    pub x1: i32,
    pub x2: i32,
    pub y1: i32,
    pub y2: i32,
    pub num_tiles: i32,
    /// The distinct active tile kinds inside the room (`WorldGen.houseTile`).
    pub house_tiles: Vec<u16>,
}

/// `WorldGen.RoomNeeds` (`:5717-5756`): the house-tile table against the
/// four furniture sets — all present means a valid home.
pub fn room_needs(house_tiles: &[u16]) -> bool {
    let has = |types: &[u16]| house_tiles.iter().any(|k| types.contains(k));
    has(CHAIR_TYPES) && has(TABLE_TYPES) && has(TORCH_TYPES) && has(DOOR_TYPES)
}

/// `StartRoomCheck` (`WorldGen.cs:6041-6117`): the 8-neighborhood flood
/// fill from `(x, y)` classifying every tile; the wall ±2 scan of
/// `CheckRoom` (`:6215-6241`) closes the room against holes.
pub fn check_room(world: &World, x: i32, y: i32) -> Result<RoomScan, RoomFailure> {
    if x < 10 || y < 10 || x >= world.width - 10 || y >= world.height - 10 {
        return Err(RoomFailure::TooCloseToWorldEdge);
    }
    if world
        .tile(x, y)
        .is_some_and(|t| t.active() && TILE_SOLID.has(t.kind))
    {
        return Err(RoomFailure::StartedInASolidTile);
    }
    let mut scan = RoomScan {
        x1: x,
        x2: x,
        y1: y,
        y2: y,
        num_tiles: 0,
        house_tiles: Vec::new(),
    };
    let mut visited = std::collections::HashSet::new();
    let mut stack = vec![(x, y)];
    while let Some((px, py)) = stack.pop() {
        if !check_tile(world, &mut scan, px, py, x, y, &mut visited)? {
            continue;
        }
        for j in -1..=1 {
            for k in -1..=1 {
                if j != 0 || k != 0 {
                    stack.push((px + j, py + k));
                }
            }
        }
    }
    if scan.num_tiles < MIN_ROOM_TILES {
        return Err(RoomFailure::RoomIsTooSmall);
    }
    Ok(scan)
}

/// `CheckRoom` (`WorldGen.cs:6113-6243`) for one tile: `false` = the tile is
/// settled (the fill does not branch from it), `Err` = the room fails.
#[allow(clippy::too_many_arguments)]
fn check_tile(
    world: &World,
    scan: &mut RoomScan,
    x: i32,
    y: i32,
    sx: i32,
    sy: i32,
    visited: &mut std::collections::HashSet<(i32, i32)>,
) -> Result<bool, RoomFailure> {
    if x < 10 || y < 10 || x >= world.width - 10 || y >= world.height - 10 {
        return Err(RoomFailure::TooCloseToWorldEdge);
    }
    // `roomTiles.InBounds` — the RoomTilesIndexedSet window spans
    // maxRoomSize around the start point (the span check below dominates
    // for the oversized rooms; the window is the C#'s hard cut).
    if (x - sx).abs() >= MAX_ROOM_SIZE || (y - sy).abs() >= MAX_ROOM_SIZE {
        return Err(RoomFailure::RoomIsTooBig);
    }
    if !visited.insert((x, y)) {
        return Ok(false);
    }
    scan.num_tiles += 1;
    if scan.num_tiles >= MAX_ROOM_TILES {
        return Err(RoomFailure::RoomIsTooBig);
    }
    if x < scan.x1 {
        scan.x1 = x;
    }
    if x > scan.x2 {
        scan.x2 = x;
    }
    if y < scan.y1 {
        scan.y1 = y;
    }
    if y > scan.y2 {
        scan.y2 = y;
    }
    if scan.x2 - scan.x1 >= MAX_ROOM_SIZE || scan.y2 - scan.y1 >= MAX_ROOM_SIZE {
        return Err(RoomFailure::RoomIsTooBig);
    }
    if let Some(t) = world.tile(x, y) {
        if t.active() {
            if !scan.house_tiles.contains(&t.kind) {
                scan.house_tiles.push(t.kind);
            }
            // `:6176-6195`: only a SOLID tile settles the fill (the wall the
            // fill butts against) — plus the open gates (11 at the frameX
            // ends 0/54/72/126, 389, and 386 at the open frame pairs).
            // Non-solid furniture keeps the fill running — the bed-spawn
            // check starts its flood ON the bed's cell (`Player.cs:54571`'s
            // `StartRoomCheck(x, y - 1)`), so settling on any active tile
            // would fail every bed room.
            if TILE_SOLID.has(t.kind)
                || (t.kind == 11 && matches!(t.frame_x, 0 | 54 | 72 | 126))
                || t.kind == 389
                || (t.kind == 386
                    && ((t.frame_x < 36 && t.frame_y == 18)
                        || (t.frame_x >= 36 && t.frame_y == 0)))
            {
                return Ok(false);
            }
        }
    }
    // The ±2 wall scan (`:6215-6241`): a house wall or a solid/housing tile
    // within two tiles horizontally AND vertically closes the room.
    let mut flag_h = false;
    let mut flag_v = false;
    for i in -2..3 {
        if let Some(t) = world.tile(x + i, y) {
            if is_house_wall(t.wall) {
                flag_h = true;
            }
            if t.active() && (TILE_SOLID.has(t.kind) || is_housing_tile(t.kind)) {
                flag_h = true;
            }
        }
        if let Some(t) = world.tile(x, y + i) {
            if is_house_wall(t.wall) {
                flag_v = true;
            }
            if t.active() && (TILE_SOLID.has(t.kind) || is_housing_tile(t.kind)) {
                flag_v = true;
            }
        }
    }
    if !flag_h || !flag_v {
        let wall = world.tile(x, y).map(|t| t.wall).unwrap_or(0);
        return Err(if wall > 0 {
            RoomFailure::TooManyUnsafeWalls
        } else {
            RoomFailure::HoleInWallIsTooBig
        });
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::Tile;
    use crate::tile::FLAG_ACTIVE;

    fn ground() -> World {
        let mut w = World::new("t".into(), 200, 200);
        for x in 0..200 {
            for y in 120..200 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w
    }

    /// Builds a wooden room: the plank walls (27), a wood frame, the floor
    /// from the ground, plus the furniture kinds listed.
    fn carve_room(w: &mut World, x0: i32, y0: i32, w_: i32, h: i32, furniture: &[u16]) {
        // The walls behind the room.
        for x in x0..x0 + w_ {
            for y in y0..y0 + h {
                let i = w.index(x, y);
                w.tiles[i].wall = 27;
            }
        }
        // The frame: the wood (9) top/bottom/sides.
        for x in x0..x0 + w_ {
            for y in [y0, y0 + h - 1] {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 9,
                    frame_x: -1,
                    frame_y: -1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for y in y0..y0 + h {
            for x in [x0, x0 + w_ - 1] {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 9,
                    frame_x: -1,
                    frame_y: -1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        // The furniture on the floor.
        for (n, kind) in furniture.iter().enumerate() {
            let i = w.index(x0 + 1 + n as i32, y0 + h - 2);
            w.tiles[i] = Tile {
                kind: *kind,
                frame_x: 0,
                frame_y: 0,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
    }

    #[test]
    fn a_furnished_room_passes_and_reports_its_bounds() {
        let mut w = ground();
        carve_room(&mut w, 50, 100, 12, 9, &[15, 14, 4, 10]);
        let scan = check_room(&w, 55, 105).expect("the room scans");
        assert_eq!(scan.x1, 50);
        assert_eq!(scan.x2, 61);
        assert!(scan.num_tiles >= MIN_ROOM_TILES);
        assert!(room_needs(&scan.house_tiles));
    }

    #[test]
    fn the_room_needs_fail_without_furniture() {
        let mut w = ground();
        carve_room(&mut w, 50, 100, 12, 9, &[15, 14, 4]); // no door
        let scan = check_room(&w, 55, 105).unwrap();
        assert!(!room_needs(&scan.house_tiles));
        carve_room(&mut w, 80, 100, 12, 9, &[14, 4, 10]); // no chair
        let scan = check_room(&w, 85, 105).unwrap();
        assert!(!room_needs(&scan.house_tiles));
    }

    #[test]
    fn the_failure_reasons_match_the_source() {
        // Solid start (`:6069-6074`).
        let w = ground();
        assert!(matches!(
            check_room(&w, 50, 130),
            Err(RoomFailure::StartedInASolidTile)
        ));
        // Too close to the world edge (`:6063-6068`).
        assert!(matches!(
            check_room(&w, 5, 100),
            Err(RoomFailure::TooCloseToWorldEdge)
        ));
        // A hole: no walls at all — the open field fails the ±2 scan
        // (HoleInWallIsTooBig, the wall == 0 arm `:6236-6240`).
        assert!(matches!(
            check_room(&w, 30, 100),
            Err(RoomFailure::HoleInWallIsTooBig)
        ));
    }

    #[test]
    fn unsafe_walls_fail_the_room() {
        // The dirt-walled room: enclosed (the wood frame stops the fill),
        // but the ±2 scan finds no HOUSE wall — the interior tile's own
        // dirt wall (3) > 0 → TooManyUnsafeWalls ().
        let mut w = ground();
        carve_room(&mut w, 50, 100, 12, 9, &[15, 14, 4, 10]);
        for x in 50..62 {
            for y in 100..109 {
                let i = w.index(x, y);
                w.tiles[i].wall = 3;
            }
        }
        assert!(matches!(
            check_room(&w, 55, 105),
            Err(RoomFailure::TooManyUnsafeWalls)
        ));
    }

    #[test]
    fn a_breach_to_the_outside_fails_with_a_hole() {
        // A 5-tall wall-and-tile breach in the roof: the fill escapes into
        // the open sky, whose tiles carry no wall — HoleInWallIsTooBig
        // (the wall == 0 arm, `:6236-6240`).
        let mut w = ground();
        carve_room(&mut w, 50, 100, 12, 9, &[15, 14, 4, 10]);
        for dy in 0..5i32 {
            let i = w.index(55, 100 - dy);
            w.tiles[i].flags &= !FLAG_ACTIVE;
            w.tiles[i].wall = 0;
        }
        assert!(matches!(
            check_room(&w, 55, 105),
            Err(RoomFailure::HoleInWallIsTooBig)
        ));
    }

    #[test]
    fn the_candelabra_counts_solid_for_the_spawn_check() {
        // `Player.CheckSpawn`'s `tileSolid[379]` window
        // (`Player.cs:54535-54538` + the frame toggle `Main.cs:9468`,
        // `:17753`, `:17799`): the candelabra is solid at base, non-solid
        // only during the `UpdateWorld_Players` window and re-forced solid
        // for the whole `CheckSpawn_Internal`. The generated `TILE_SOLID`
        // carries the C# base state (379 set), so the plain scan already
        // matches `CheckSpawn`'s forced view — no special-casing.
        let mut w = ground();
        let i = w.index(55, 105);
        w.tiles[i] = Tile {
            kind: 379,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        assert!(matches!(
            check_room(&w, 55, 105),
            Err(RoomFailure::StartedInASolidTile)
        ));
    }
}
