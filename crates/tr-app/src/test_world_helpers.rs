//! The shared test-world builders — the ground + the furnished-room
//! fixtures the housing/town tests compose.

use tr_core::tile::FLAG_ACTIVE;
use tr_core::Tile;
use tr_core::World;

/// A 200x200 world with the dirt ground at y=120.
pub fn ground() -> World {
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

/// Builds a wooden room with the plank walls (27), a wood frame and the
/// furniture kinds listed on the floor.
pub fn build_furnished_room(w: &mut World, x0: i32, y0: i32, w_: i32, h: i32) {
    build_furnished_room_with(w, x0, y0, w_, h, &[15, 14, 4, 10]);
}

/// The furniture-variant overload.
pub fn build_furnished_room_with(
    w: &mut World,
    x0: i32,
    y0: i32,
    w_: i32,
    h: i32,
    furniture: &[u16],
) {
    for x in x0..x0 + w_ {
        for y in y0..y0 + h {
            let i = w.index(x, y);
            w.tiles[i].wall = 27;
        }
    }
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
