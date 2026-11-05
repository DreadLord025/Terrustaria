//! The Lihzahrd temple — the compact port of the `LihzahrdTemple` +
//! `LihzahrdTemplePart2` passes (`WorldGen.cs:15834-17862`) and
//! `AddLihzahrdAltar` (`:32071-32100`).
//!
//! The C# temple: the brick box (the Lihzahrd brick 226, the walls 87
//! unsafe / 112 safe) buried in the jungle cavern below the rock layer,
//! the interior rooms carved, the altar (237) in the main room, the dart
//! traps (137) in the walls, the teleporter pads, the chests. The port
//! reproduces the shape: the outer box, the room corridors, the altar
//! room, the traps, the chests.
//!
//! Deviations (ANALYSIS "Lihzahrd temple (M1.4-WGEN-6)"): the C#'s
//! multi-room crawl with the doors and the teleporters is the shape-level
//! approximation (the box + the 3 corridor rooms); the Plantera-gate (the
//! altar needs the Plantera downed to interact) waits for M5-HARDFORCE;
//! the trap wiring arms ride the wiring engine (the traps 137 are the
//! case-137 variants the wiring trip fires).

use super::structures::place_chest;
use crate::tile::{FLAG_ACTIVE, Tile};
use crate::world::World;

pub(crate) const TILE_TEMPLE_BRICK: u16 = 226;
pub(crate) const TILE_ALTAR: u16 = 237;
pub(crate) const WALL_TEMPLE_UNSAFE: u16 = 87;
pub(crate) const WALL_TEMPLE: u16 = 112;

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

/// `AddLihzahrdAltar` (`WorldGen.cs:32071-32100`): the 3×2 altar (237)
/// with the 3-wide brick base row.
pub(crate) fn place_lihzahrd_altar(world: &mut World, x: i32, y: i32) {
    for i in 0..3 {
        for j in 0..2 {
            let px = x + i;
            let py = y + j;
            if px < 2 || py < 2 || px >= world.width - 2 || py >= world.height - 2 {
                return;
            }
            let wall = world.tile(px, py).map(|t| t.wall).unwrap_or(0);
            let i = world.index(px, py);
            world.tiles[i] = Tile {
                kind: TILE_ALTAR,
                frame_x: (i * 18) as i16,
                frame_y: (j * 18) as i16,
                ..Default::default()
            };
            world.tiles[i].flags |= FLAG_ACTIVE;
            world.tiles[i].wall = wall;
        }
        // The base row (`:32090-32100`).
        let py = y + 2;
        if x + i < 2 || py >= world.height - 2 {
            continue;
        }
        let i2 = world.index(x + i, py);
        let t = &mut world.tiles[i2];
        t.kind = TILE_TEMPLE_BRICK;
        t.flags |= FLAG_ACTIVE;
        t.set_slope(0);
    }
}

/// The `LihzahrdTemple` pass (`WorldGen.cs:15834+`): the temple box in the
/// jungle cavern below the rock layer — the outer brick shell (the walls
/// 87), the carved room corridors (the walls 112), the altar room with the
/// 3×2 altar, the dart traps (137) embedded in the walls, the chests in
/// the rooms. Returns the altar's tile position.
pub(crate) fn pass_lihzahrd_temple(
    world: &mut World,
    spans: &super::biomes::BiomeSpans,
    rock_layer: f64,
    rng: &mut super::Rng,
) -> Option<(i32, i32)> {
    // `:15840-15848`: the y between the rock layer and `maxTilesY − 600`
    // (the port clamps to the height), the x on the dungeon-opposite side
    // (`dungeonSide` — the port puts it under the jungle band's deep half).
    let y_lo = (rock_layer as i32 + 60).min(world.height - 420);
    let y_hi = (world.height - 260).max(y_lo + 40);
    if y_hi <= y_lo {
        return None;
    }
    let cy = rng.range(y_lo, y_hi);
    let cx_lo = spans.jungle_start + 60;
    let cx_hi = (spans.jungle_end + 240).min(world.width - 120);
    if cx_hi <= cx_lo {
        return None;
    }
    let cx = rng.range(cx_lo, cx_hi);
    // The outer box: the brick shell with the unsafe walls.
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
    Some((altar_x, altar_y - 2))
}
