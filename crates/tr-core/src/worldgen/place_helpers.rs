//! The shared WorldGen placement wrappers (W4-g39, `docs/research/12j`).
//!
//! The C# `WorldGen.Place*` helper family the worldgen passes call. The
//! per-module narrow copies (`temple.rs`'s chair/workbench slices,
//! `mini_biomes.rs`'s 141/411 slices, `evil.rs`'s altar `place_3x2`) stay
//! where they are — these are the GENERAL verbatim wrappers the pending
//! d10x/g37 passes (furniture, statues, houses) will call. Every RNG-taking
//! helper draws from the caller's pass stream (`&mut Rng` = the C#
//! `WorldGen.genRand` at the call site); no stream is invented here.
//!
//! Unwired until a consumer pass lands: no executed step calls these, so
//! both world-hash pins stay unchanged (the `world_hash.rs` note).

use super::noise::Rng;
use crate::tile::FLAG_ACTIVE;
use crate::tiledata::{TILE_SOLID, TILE_SOLID_TOP};
use crate::world::World;

/// `WorldGen.InWorld(int, int, int)` (`WorldGen.cs:9688-9695`): the fluff
/// margin gate (the same copy the sibling worldgen modules carry).
fn in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// `WorldGen.SolidTile2(int, int)` via `SolidTile2(Tile)`
/// (`WorldGen.cs:70134-70151`): a null/out-of-world tile reads as solid;
/// active + `tileSolid` + no slope + no half brick + not actuated-inactive.
/// No `tileSolidTop` exclusion (that is `SolidTile`'s extra term).
fn solid_tile2(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && TILE_SOLID.has(t.kind)
        && t.slope() == 0
        && !t.half_brick()
        && !t.inactive()
}

/// `Main.tileTable` (`Main.cs:6879-6942`, `:7758-7759`, `:8032-8052`,
/// `:8307`, `:9982`, `:9986-9995`): the table-top set the on-table
/// placement arms test. A local copy — `tiledata/` is generated and carries
/// no `TILE_TABLE` row (the same idiom as `mini_biomes.rs`'s
/// `general_placement_tile`). The `:9992` loop adds 435..=439.
fn tile_table(kind: u16) -> bool {
    matches!(
        kind,
        14 | 18
            | 19
            | 87
            | 88
            | 101
            | 114
            | 275
            | 276
            | 277
            | 278
            | 279
            | 280
            | 281
            | 285
            | 286
            | 296
            | 297
            | 298
            | 299
            | 309
            | 310
            | 339
            | 358
            | 359
            | 361
            | 362
            | 363
            | 364
            | 376
            | 380
            | 391
            | 392
            | 393
            | 394
            | 405
            | 413
            | 414
            | 427
            | 435
            | 436
            | 437
            | 438
            | 439
            | 469
            | 532
            | 533
            | 538
            | 542
            | 544
            | 550
            | 551
            | 553
            | 554
            | 555
            | 556
            | 558
            | 559
            | 582
            | 599
            | 600
            | 601
            | 602
            | 603
            | 604
            | 605
            | 606
            | 607
            | 608
            | 609
            | 610
            | 611
            | 612
            | 619
            | 629
            | 632
            | 640
            | 643
            | 644
            | 645
            | 710
    )
}

/// `WorldGen.Place1x2` (`WorldGen.cs:46652-46679`): the two-tall object
/// standing on a solid floor — the body is `(x, y-1)`/`(x, y)`,
/// `frameY = 40 * style + row * 18`. The sapling (type 20) rolls
/// `genRand.Next(3) * 18` into `frameX` FIRST (`:46655-46658`), before any
/// gate — the draw fires even when the placement is refused. The C#
/// null-tile news (`:46659-46666`) are the fixed-grid stand-in: the port
/// bounds-guards the two body cells instead (the sibling `temple.rs`
/// `place_chair` adaptation). Gate: `SolidTile2(x, y+1)` and the top cell
/// inactive (`:46667`). No `tileFrameImportant` write — the C# wrapper has
/// none (the boot statics carry it).
pub(crate) fn place_1x2(world: &mut World, rng: &mut Rng, x: i32, y: i32, kind: u16, style: i32) {
    // `:46654-46658` — the sapling variant roll precedes every gate.
    let frame_x = if kind == 20 {
        (rng.range(0, 3) * 18) as i16
    } else {
        0
    };
    if !in_world(world, x, y - 1, 0) || !in_world(world, x, y, 0) {
        return;
    }
    // `:46667` — the floor solid and the top cell empty.
    if !solid_tile2(world, x, y + 1) || world.tile(x, y - 1).is_some_and(|t| t.active()) {
        return;
    }
    let num = (style * 40) as i16;
    // `:46670-46677` — the top row then the bottom row, C# write order.
    for (row, frame_y) in [(y - 1, num), (y, num + 18)] {
        let i = world.index(x, row);
        let t = &mut world.tiles[i];
        t.flags |= FLAG_ACTIVE;
        t.frame_y = frame_y;
        t.frame_x = frame_x;
        t.kind = kind;
    }
}

/// `WorldGen.Place2x1` (`WorldGen.cs:47368-47406`): the two-wide, one-tall
/// object at `(x, y)`/`(x+1, y)`, `frameX = 36 * style + col * 18`,
/// `frameY = 0`. Two floor arms (`:47386-47394`): the general kinds need
/// `SolidTile2` under BOTH columns; the on-table pair (29 candle / 103
/// water candle) instead needs both floor cells active and `tileTable`.
/// Both arms also need both body cells inactive. The C# null-tile news
/// (`:47370-47385`) are the fixed-grid stand-in (the body-cell bounds
/// guard; the floor reads fall through `world.tile`'s `Option`).
pub(crate) fn place_2x1(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    if !in_world(world, x, y, 0) || !in_world(world, x + 1, y, 0) {
        return;
    }
    let body_empty = !world.tile(x, y).is_some_and(|t| t.active())
        && !world.tile(x + 1, y).is_some_and(|t| t.active());
    // `:47387` — the general solid-floor arm (29/103 excluded).
    let general_arm = kind != 29
        && kind != 103
        && solid_tile2(world, x, y + 1)
        && solid_tile2(world, x + 1, y + 1)
        && body_empty;
    // `:47391` — the on-table arm: both floors active AND tables.
    let table_arm = (kind == 29 || kind == 103)
        && world
            .tile(x, y + 1)
            .is_some_and(|t| t.active() && tile_table(t.kind))
        && world
            .tile(x + 1, y + 1)
            .is_some_and(|t| t.active() && tile_table(t.kind))
        && body_empty;
    if !(general_arm || table_arm) {
        return;
    }
    // `:47397-47404` — the left cell then the right cell, C# write order.
    for (col, frame_x) in [(x, (36 * style) as i16), (x + 1, (36 * style + 18) as i16)] {
        let i = world.index(col, y);
        let t = &mut world.tiles[i];
        t.flags |= FLAG_ACTIVE;
        t.frame_y = 0;
        t.frame_x = frame_x;
        t.kind = kind;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small world with a stone floor row at `y = 20`.
    fn floor_world() -> World {
        let mut w = World::new("ph".into(), 60, 60);
        for x in 0..w.width {
            let i = w.index(x, 20);
            w.tiles[i].flags |= FLAG_ACTIVE;
            w.tiles[i].kind = 1; // stone — tileSolid, not solidTop
        }
        w
    }

    /// `Place1x2` lays the chair (15) two-tall over the floor with the
    /// 40-px style stride, and the sapling draw fires before the gates.
    #[test]
    fn place_1x2_lays_the_two_rows_with_the_40_stride() {
        let mut w = floor_world();
        let mut rng = Rng(7);
        place_1x2(&mut w, &mut rng, 10, 19, 15, 3);
        let top = w.tile(10, 18).unwrap();
        let bottom = w.tile(10, 19).unwrap();
        assert!(top.active() && bottom.active());
        assert_eq!(top.kind, 15);
        assert_eq!(bottom.kind, 15);
        assert_eq!(top.frame_y, 120); // 40 * 3
        assert_eq!(bottom.frame_y, 138); // 40 * 3 + 18
        assert_eq!(top.frame_x, 0);
        // The sapling (20) draws one Next(3) even when the floor refuses:
        // the post-call stream equals one range(0,3) consumed from seed 7.
        let mut w2 = World::new("ph".into(), 60, 60); // no floor
        let mut a = Rng(7);
        place_1x2(&mut w2, &mut a, 10, 19, 20, 0);
        let mut b = Rng(7);
        b.range(0, 3);
        assert_eq!(a.0, b.0);
        assert!(!w2.tile(10, 18).unwrap().active());
    }

    /// `Place2x1`'s general arm needs both floor columns solid; the
    /// candle (29) instead needs two active table tops.
    #[test]
    fn place_2x1_general_and_table_arms() {
        let mut w = floor_world();
        place_2x1(&mut w, 10, 19, 18, 2);
        let left = w.tile(10, 19).unwrap();
        let right = w.tile(11, 19).unwrap();
        assert!(left.active() && right.active());
        assert_eq!(left.frame_x, 72); // 36 * 2
        assert_eq!(right.frame_x, 90); // 36 * 2 + 18
        assert_eq!(left.frame_y, 0);
        // The candle refuses on stone (not a table) …
        let mut w2 = floor_world();
        place_2x1(&mut w2, 20, 19, 29, 0);
        assert!(!w2.tile(20, 19).unwrap().active());
        // … and places on two table (14) tops.
        for x in [30, 31] {
            let i = w2.index(x, 20);
            w2.tiles[i].kind = 14;
        }
        place_2x1(&mut w2, 30, 19, 29, 1);
        assert!(w2.tile(30, 19).unwrap().active());
        assert_eq!(w2.tile(30, 19).unwrap().frame_x, 36);
        assert_eq!(w2.tile(31, 19).unwrap().frame_x, 54);
    }
}