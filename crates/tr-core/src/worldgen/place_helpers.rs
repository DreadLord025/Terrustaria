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

/// `WorldGen.Place3x1` (`WorldGen.cs:52022-52064`): the three-wide,
/// one-tall object centred on `x` — body `(x-1..=x+1, y)`,
/// `frameX = 54 * style + col * 18`, `frameY = 0`. The 5-tile bounds gate
/// (`:52024`), then per column: the body cell inactive AND `SolidTile2`
/// under it (`:52029-52047`). The C# null-tile news are the fixed-grid
/// stand-in (the bounds gate covers every read).
pub(crate) fn place_3x1(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    if x < 5 || x > world.width - 5 || y < 5 || y > world.height - 5 {
        return;
    }
    let mut flag = true;
    for i in (x - 1)..(x + 2) {
        if world.tile(i, y).is_some_and(|t| t.active()) {
            flag = false;
        }
        if !solid_tile2(world, i, y + 1) {
            flag = false;
        }
    }
    if flag {
        let num = (54 * style) as i16;
        // `:52051-52062` — left, middle, right, C# write order.
        for (col, frame_x) in [(x - 1, num), (x, num + 18), (x + 1, num + 36)] {
            let i = world.index(col, y);
            let t = &mut world.tiles[i];
            t.flags |= FLAG_ACTIVE;
            t.frame_y = 0;
            t.frame_x = frame_x;
            t.kind = kind;
        }
    }
}

/// `WorldGen.SolidTile(Tile)` (`WorldGen.cs:69957-69974`): the TILE
/// overload — null reads solid; active + `tileSolid` + NOT `tileSolidTop`
/// + no half brick + no slope. NOTE: unlike the `(int, int)` overload
/// (`:70111-70132`) this one has NO `inActive()` term — `CanPlaceSink`
/// calls this form, so an actuated-inactive floor still passes.
fn solid_tile_t(t: Option<&crate::tile::Tile>) -> bool {
    let Some(t) = t else {
        return true;
    };
    t.active()
        && TILE_SOLID.has(t.kind)
        && !TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
}

/// `WorldGen.SolidTile(int, int)` (`WorldGen.cs:70111-70132`, `noDoors`
/// default false): the coordinate overload — the Tile terms PLUS the
/// `inActive()` veto. Out-of-world reads solid.
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && TILE_SOLID.has(t.kind)
        && !TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

/// `WorldGen.CanPlaceSink` (`WorldGen.cs:51885-51910`): the 5-tile bounds
/// gate, then the 2×2 body at `(x-1, y-1)` empty and both floor columns
/// `(x-1, y+1)`/`(x, y+1)` active and `SolidTile` (the Tile overload —
/// no inactive veto).
fn can_place_sink(world: &World, x: i32, y: i32) -> bool {
    if x < 5 || x > world.width - 5 || y < 5 || y > world.height - 5 {
        return false;
    }
    let mut result = true;
    let (x, y) = (x - 1, y - 1);
    for i in 0..2 {
        for j in 0..2 {
            if world.tile(x + i, y + j).is_some_and(|t| t.active()) {
                result = false;
            }
        }
        let floor = world.tile(x + i, y + 2);
        if !floor.is_some_and(|t| t.active()) || !solid_tile_t(floor) {
            result = false;
        }
    }
    result
}

/// `WorldGen.Place2x2` (`WorldGen.cs:51330-51423`): the general 2×2
/// object at `(x-1, y-1)`. The hanging pair (95/126) bumps `y` FIRST
/// (`:51332-51335`), then the 5-tile bounds gate. The sink (172) rides
/// `CanPlaceSink`; every other kind scans the 2×2 body empty (the
/// campfire 98 also refuses any liquid, `:51354`) and per column:
/// - 95/126: the CEILING `(i, y-2)` active + `tileSolid` + not
///   `tileSolidTop` (`:51361-51370`);
/// - 132 (the hanging bottle): no floor term in the scan (`:51380`);
/// - default: the floor `(i, y+1)` active AND (`SolidTile2` OR
///   `tileTable`) (`:51371-51379`).
///
/// The 132 wall arm (`:51385-51408`): both floors must be active +
/// (solid2 or table); if not, all four body walls must be >= 1 — the C#
/// `:51404` tests `(x-1, y)` TWICE (the `(x, y)` cell is never read);
/// kept verbatim. The frame stride is 38 for the sink, 36 otherwise
/// (`:51411`): `frameX = k * 18`, `frameY = style * num + l * 18`.
pub(crate) fn place_2x2(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    let mut y = y;
    if kind == 95 || kind == 126 {
        y += 1;
    }
    if x < 5 || x > world.width - 5 || y < 5 || y > world.height - 5 {
        return;
    }
    if kind == 172 {
        if !can_place_sink(world, x, y) {
            return;
        }
    } else {
        for i in (x - 1)..(x + 1) {
            for j in (y - 1)..(y + 1) {
                let cell = world.tile(i, j);
                if cell.is_some_and(|t| t.active() || (kind == 98 && t.liquid > 0)) {
                    return;
                }
            }
            match kind {
                // `:51361-51370` — the ceiling arm.
                95 | 126 => {
                    let ceil = world.tile(i, y - 2);
                    if !ceil.is_some_and(|t| {
                        t.active() && TILE_SOLID.has(t.kind) && !TILE_SOLID_TOP.has(t.kind)
                    }) {
                        return;
                    }
                }
                // `:51380-51381` — the hanging bottle: no floor term.
                132 => {}
                // `:51371-51379` — the floor arm.
                _ => {
                    let floor = world.tile(i, y + 1);
                    let ok = floor.is_some_and(|t| t.active())
                        && (solid_tile2(world, i, y + 1)
                            || floor.is_some_and(|t| tile_table(t.kind)));
                    if !ok {
                        return;
                    }
                }
            }
        }
    }
    if kind == 132 {
        // `:51385-51408` — the floor-or-wall arm (the duplicated
        // `(x-1, y)` wall test kept verbatim).
        let mut flag = true;
        for col in [x - 1, x] {
            let floor = world.tile(col, y + 1);
            if !floor.is_some_and(|t| t.active())
                || !(solid_tile2(world, col, y + 1) || floor.is_some_and(|t| tile_table(t.kind)))
            {
                flag = false;
            }
        }
        let wall_ok = |wx: i32, wy: i32| world.tile(wx, wy).is_some_and(|t| t.wall >= 1);
        if !flag
            && (!wall_ok(x - 1, y - 1)
                || !wall_ok(x, y - 1)
                || !wall_ok(x - 1, y)
                || !wall_ok(x - 1, y))
        {
            return;
        }
    }
    let (x, y) = (x - 1, y - 1);
    let num = if kind == 172 { 38 } else { 36 };
    for k in 0..2 {
        for l in 0..2 {
            let i = world.index(x + k, y + l);
            let t = &mut world.tiles[i];
            t.flags |= FLAG_ACTIVE;
            t.frame_x = (k * 18) as i16;
            t.frame_y = (style * num + l * 18) as i16;
            t.kind = kind;
        }
    }
}

/// `WorldGen.InvalidTileForPilesOrSpeleothems` (`WorldGen.cs:38979-38994`):
/// `InWorld(x, y, 2)`, then an active tile is invalid iff it is a boulder.
fn invalid_tile_for_piles(world: &World, x: i32, y: i32) -> bool {
    if !in_world(world, x, y, 2) {
        return false;
    }
    world
        .tile(x, y)
        .is_some_and(|t| t.active() && crate::tiledata::SET_BOULDERS.has(t.kind))
}

/// `WorldGen.Place3x2` (`WorldGen.cs:52066-52196`): the general 3×2
/// object centred on `x`. The 5-tile bounds gate. The picnic table
/// (14 style 25) is the ONE-ROW arm (`flag`, `:52074-52082`): only row `y`
/// is scanned and only row `y` is written (`frameY = 0`, `:52155-52169`).
/// The body scan (`:52083-52100`): every cell inactive, and a campfire
/// kind (`TileID.Sets.Campfires`) also refuses liquid. Per-column floor
/// arms (`:52105-52139`):
/// - 285/286/298/299/310/361-364 AND 582/619 (the `goto case 285`):
///   `SolidTile2` OR an active `tileSolidTop` with `frameY == 0`;
/// - default: the piles pair (186/187) refuses a boulder floor
///   (`InvalidTileForPilesOrSpeleothems`), the altar (26) refuses a
///   boulder floor directly, and `SolidTile2` is required.
///
/// The chest arm (88, `:52141-52151`): the C# `Chest.CreateChest` ledger
/// write + the `netMode == 1` SendData are runtime/chest-lane concerns —
/// the port's worldgen chest sites ride `structures::place_chest` +
/// `chest_stock` (the gen-time ledger seam); this wrapper places the
/// tiles and the ledger stays with the chest lane (documented deviation,
/// the worldgen chest cap is never hit so the C# gate never refuses).
/// Frames: `frameX = 54 * style + col * 18`, `frameY = row * 18`.
pub(crate) fn place_3x2(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    if x < 5 || x > world.width - 5 || y < 5 || y > world.height - 5 {
        return;
    }
    let one_row = kind == 14 && style == 25;
    let num = if one_row { y } else { y - 1 };
    let mut flag2 = true;
    for i in (x - 1)..(x + 2) {
        for j in num..(y + 1) {
            let cell = world.tile(i, j);
            if cell.is_some_and(|t| t.active()) {
                flag2 = false;
            }
            if crate::tiledata::SET_CAMPFIRES.has(kind)
                && cell.is_some_and(|t| t.liquid > 0)
            {
                flag2 = false;
            }
        }
        let floor = world.tile(i, y + 1);
        match kind {
            // `:52125-52138` (+ the 582/619 goto, `:52108-52124`): the
            // solidTop-with-frameY-0 alternative floor.
            285 | 286 | 298 | 299 | 310 | 361 | 362 | 363 | 364 | 582 | 619 => {
                let solid_top_ok = floor.is_some_and(|t| {
                    t.active() && TILE_SOLID_TOP.has(t.kind) && t.frame_y == 0
                });
                if !solid_tile2(world, i, y + 1) && !solid_top_ok {
                    flag2 = false;
                }
            }
            _ => {
                if (kind == 186 || kind == 187) && invalid_tile_for_piles(world, i, y + 1) {
                    flag2 = false;
                }
                if kind == 26 && floor.is_some_and(|t| crate::tiledata::SET_BOULDERS.has(t.kind)) {
                    flag2 = false;
                }
                if !solid_tile2(world, i, y + 1) {
                    flag2 = false;
                }
            }
        }
    }
    if flag2 {
        let num2 = (54 * style) as i16;
        if one_row {
            // `:52157-52168` — the single row, frameY 0.
            for (col, fx) in [(x - 1, num2), (x, num2 + 18), (x + 1, num2 + 36)] {
                let i = world.index(col, y);
                let t = &mut world.tiles[i];
                t.flags |= FLAG_ACTIVE;
                t.frame_y = 0;
                t.frame_x = fx;
                t.kind = kind;
            }
            return;
        }
        // `:52171-52194` — row-major, top row then bottom row.
        for (col, row) in [
            (x - 1, y - 1),
            (x, y - 1),
            (x + 1, y - 1),
            (x - 1, y),
            (x, y),
            (x + 1, y),
        ] {
            let fx = num2 + ((col - (x - 1)) * 18) as i16;
            let fy = if row == y { 18 } else { 0 };
            let i = world.index(col, row);
            let t = &mut world.tiles[i];
            t.flags |= FLAG_ACTIVE;
            t.frame_y = fy;
            t.frame_x = fx;
            t.kind = kind;
        }
    }
}

/// The shared wall-painting core (`WorldGen.Place{3x2,3x3,2x3,4x3,6x4}Wall`
/// — `WorldGen.cs:44725-44756`/`:44254-44292`/`:44568-44599`/
/// `:44854-44886`/`:44946-44979`): every body cell must be inactive AND
/// carry a wall (`wall != 0`), then the rect is written active with
/// `frameX = fx_base + col * 18`, `frameY = fy_base + row * 18`,
/// column-major (the C# loop order). The C# `break` exits only the inner
/// loop, but `flag = false` is sticky — a full scan is equivalent. The
/// port bounds-guards the rect (the C# null-tile reads would throw; every
/// worldgen caller sits well inside the world).
fn place_wall_rect(
    world: &mut World,
    x0: i32,
    y0: i32,
    w: i32,
    h: i32,
    kind: u16,
    fx_base: i32,
    fy_base: i32,
) {
    for i in x0..(x0 + w) {
        for j in y0..(y0 + h) {
            let ok = world
                .tile(i, j)
                .is_some_and(|t| !t.active() && t.wall != 0);
            if !ok {
                return;
            }
        }
    }
    for k in x0..(x0 + w) {
        for l in y0..(y0 + h) {
            let i = world.index(k, l);
            let t = &mut world.tiles[i];
            t.flags |= FLAG_ACTIVE;
            t.kind = kind;
            t.frame_x = (fx_base + 18 * (k - x0)) as i16;
            t.frame_y = (fy_base + 18 * (l - y0)) as i16;
        }
    }
}

/// `WorldGen.Place3x2Wall` (`WorldGen.cs:44725-44756`): the 3×2 painting
/// at `(x-1, y)` — `frameX = col * 18`, `frameY = 36 * style + row * 18`.
pub(crate) fn place_3x2_wall(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    place_wall_rect(world, x - 1, y, 3, 2, kind, 0, style * 36);
}

/// `WorldGen.Place3x3Wall` (`WorldGen.cs:44254-44292`): the 3×3 painting
/// at `(x-1, y-1)` — the style wraps every 36 rows (`while style > 35`:
/// `:44275-44279`): `frameX = (style % 36) * 54 + col * 18`,
/// `frameY = (style / 36) * 54 + row * 18` (the C# subtractive loop keeps
/// the `> 35` boundary: style 36 wraps once, style 35 does not — the
/// modulo form matches for non-negative styles).
pub(crate) fn place_3x3_wall(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    let mut style = style;
    let mut rows = 0;
    while style > 35 {
        rows += 1;
        style -= 36;
    }
    place_wall_rect(world, x - 1, y - 1, 3, 3, kind, style * 54, rows * 54);
}

/// `WorldGen.Place2x3Wall` (`WorldGen.cs:44568-44599`): the 2×3 painting
/// at `(x, y-1)` — `frameX = 36 * style + col * 18`, `frameY = row * 18`.
pub(crate) fn place_2x3_wall(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    place_wall_rect(world, x, y - 1, 2, 3, kind, style * 36, 0);
}

/// `WorldGen.Place4x3Wall` (`WorldGen.cs:44854-44886`): the 4×3 painting
/// at `(x-1, y-1)` — `frameX = col * 18` (the C# `num3` stays 0),
/// `frameY = 54 * style + row * 18`.
pub(crate) fn place_4x3_wall(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    place_wall_rect(world, x - 1, y - 1, 4, 3, kind, 0, style * 54);
}

/// `WorldGen.Place6x4Wall` (`WorldGen.cs:44946-44979`): the 6×4 painting
/// at `(x-2, y-2)` — the 27-per-sheet-row wrap (`:44966-44968`):
/// `frameX = (style / 27) * 108 + col * 18`,
/// `frameY = (style % 27) * 72 + row * 18`.
pub(crate) fn place_6x4_wall(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    place_wall_rect(
        world,
        x - 2,
        y - 2,
        6,
        4,
        kind,
        style / 27 * 108,
        style % 27 * 72,
    );
}

/// `WorldGen.Place6x3` (`WorldGen.cs:50465-50512`): the 6-wide, 3-tall
/// bed — body `(x-3..=x+2, y-2..=y)` empty, each floor column
/// `SolidTile2` OR an active `tileSolidTop` with `frameY == 0`
/// (`:50489`). The 5-tile bounds gate. The C# `direction`/`style`
/// parameters are UNUSED in the body (the bed sheet has one orientation);
/// the port keeps them out of the signature — the callers pass the
/// defaults. Frames: column-major `frameX = col * 18`,
/// `frameY = row * 18` (the C# loop order, `:50499-50511`).
pub(crate) fn place_6x3(world: &mut World, x: i32, y: i32, kind: u16) {
    if x < 5 || x > world.width - 5 || y < 5 || y > world.height - 5 {
        return;
    }
    let mut flag = true;
    for i in (x - 3)..(x + 3) {
        for j in (y - 2)..=y {
            if world.tile(i, j).is_some_and(|t| t.active()) {
                flag = false;
            }
        }
        let solid_top_ok = world
            .tile(i, y + 1)
            .is_some_and(|t| t.active() && TILE_SOLID_TOP.has(t.kind) && t.frame_y == 0);
        if !solid_tile2(world, i, y + 1) && !solid_top_ok {
            flag = false;
        }
    }
    if !flag {
        return;
    }
    // `:50498-50511` — column-major write order.
    let mut num = 0i16;
    for k in (x - 3)..(x + 3) {
        let mut num2 = 0i16;
        for l in (y - 2)..=y {
            let i = world.index(k, l);
            let t = &mut world.tiles[i];
            t.flags |= FLAG_ACTIVE;
            t.frame_y = num2;
            t.frame_x = num;
            t.kind = kind;
            num2 += 18;
        }
        num += 18;
    }
}

/// `WorldGen.Place4x2` (`WorldGen.cs:50514-50584`): the 4-wide, 2-tall
/// object at `(x-1..=x+2, y-1..=y)` — body empty, `SolidTile2` under all
/// four columns; the 5-tile bounds gate. `direction == 1` flips to the
/// right-facing sheet half (`frameX += 72`, `:50544-50547`);
/// `frameY = 36 * style + row * 18`, `frameX = col * 18 + num`.
pub(crate) fn place_4x2(world: &mut World, x: i32, y: i32, kind: u16, direction: i32, style: i32) {
    if x < 5 || x > world.width - 5 || y < 5 || y > world.height - 5 {
        return;
    }
    let mut flag = true;
    for i in (x - 1)..(x + 3) {
        for j in (y - 1)..(y + 1) {
            if world.tile(i, j).is_some_and(|t| t.active()) {
                flag = false;
            }
        }
        if !solid_tile2(world, i, y + 1) {
            flag = false;
        }
    }
    let num: i16 = if direction == 1 { 72 } else { 0 };
    let num2 = (36 * style) as i16;
    if flag {
        // `:50551-50582` — row-major, top row then bottom row.
        for (col, row) in [
            (x - 1, y - 1),
            (x, y - 1),
            (x + 1, y - 1),
            (x + 2, y - 1),
            (x - 1, y),
            (x, y),
            (x + 1, y),
            (x + 2, y),
        ] {
            let fx = num + ((col - (x - 1)) * 18) as i16;
            let fy = num2 + if row == y { 18 } else { 0 };
            let i = world.index(col, row);
            let t = &mut world.tiles[i];
            t.flags |= FLAG_ACTIVE;
            t.frame_y = fy;
            t.frame_x = fx;
            t.kind = kind;
        }
    }
}

/// `WorldGen.Place3x4` (`WorldGen.cs:51912-51961`): the 3-wide, 4-tall
/// object — body `(x-1..=x+1, y-3..=y)` empty, `SolidTile2` under all
/// three columns; the 5-tile bounds gate. `frameX = 54 * style + col*18`,
/// `frameY = (3 + k) * 18` for the row offset `k` in `-3..=0`.
pub(crate) fn place_3x4(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    if x < 5 || x > world.width - 5 || y < 5 || y > world.height - 5 {
        return;
    }
    let mut flag = true;
    for i in (x - 1)..(x + 2) {
        for j in (y - 3)..(y + 1) {
            if world.tile(i, j).is_some_and(|t| t.active()) {
                flag = false;
            }
        }
        if !solid_tile2(world, i, y + 1) {
            flag = false;
        }
    }
    if flag {
        let num = style * 54;
        for k in -3..=0 {
            let frame_y = ((3 + k) * 18) as i16;
            for (col, fx) in [(x - 1, num), (x, num + 18), (x + 1, num + 36)] {
                let i = world.index(col, y + k);
                let t = &mut world.tiles[i];
                t.flags |= FLAG_ACTIVE;
                t.frame_y = frame_y;
                t.frame_x = fx as i16;
                t.kind = kind;
            }
        }
    }
}

/// `WorldGen.Place5x4` (`WorldGen.cs:51963-52020`): the 5-wide, 4-tall
/// object — body `(x-2..=x+2, y-3..=y)` empty, `SolidTile2` under all five
/// columns; the 5-tile bounds gate. The frame columns are CENTRED on
/// `54 * style`: `num-36, num-18, num, num+18, num+36` (the style-0 sheet
/// offset goes negative — kept verbatim, the C# writes the same).
pub(crate) fn place_5x4(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    if x < 5 || x > world.width - 5 || y < 5 || y > world.height - 5 {
        return;
    }
    let mut flag = true;
    for i in (x - 2)..(x + 3) {
        for j in (y - 3)..(y + 1) {
            if world.tile(i, j).is_some_and(|t| t.active()) {
                flag = false;
            }
        }
        if !solid_tile2(world, i, y + 1) {
            flag = false;
        }
    }
    if flag {
        let num = style * 54;
        for k in -3..=0 {
            let frame_y = ((3 + k) * 18) as i16;
            for (col, fx) in [
                (x - 2, num - 36),
                (x - 1, num - 18),
                (x, num),
                (x + 1, num + 18),
                (x + 2, num + 36),
            ] {
                let i = world.index(col, y + k);
                let t = &mut world.tiles[i];
                t.flags |= FLAG_ACTIVE;
                t.frame_y = frame_y;
                t.frame_x = fx as i16;
                t.kind = kind;
            }
        }
    }
}

/// `WorldGen.Place3x3` (`WorldGen.cs:53534-53634`): the 3×3 object with
/// TWO orientations. The standing family (106/212/219/220/228/231/243/
/// 247/283/300-308/354/355/642/733/753, `:53538`) sits on the floor:
/// `num = -2`, the body `(x-1..=x+1, y-2..=y)` empty, and each floor
/// column `SolidTile2` — EXCEPT a 753 floor tile under another 753
/// (the stackable ammo box, `:53562`). Every other kind HANGS: the body
/// `(x-1..=x+1, y..=y+2)` empty and the ceiling `(x, y-1)` active +
/// `tileSolid` + not `tileSolidTop` (`:53589`). No bounds gate in the C#
/// (the null-tile news cover it); the port guards the written cells.
/// `frameX = 54 * style + col * 18`, `frameY = row * 18`.
pub(crate) fn place_3x3(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    let mut flag = true;
    let standing = matches!(
        kind,
        106 | 212
            | 219
            | 220
            | 228
            | 231
            | 243
            | 247
            | 283
            | 300..=308
            | 354
            | 355
            | 642
            | 733
            | 753
    );
    let num = if standing { -2 } else { 0 };
    if standing {
        for i in (x - 1)..(x + 2) {
            for j in (y - 2)..(y + 1) {
                if world.tile(i, j).is_some_and(|t| t.active()) {
                    flag = false;
                }
            }
        }
        for k in (x - 1)..(x + 2) {
            // `:53562` — the 753-on-753 stacking exemption.
            let stack_ok = kind == 753
                && world.tile(k, y + 1).is_some_and(|t| t.active() && t.kind == 753);
            if !stack_ok && !solid_tile2(world, k, y + 1) {
                flag = false;
                break;
            }
        }
    } else {
        for l in (x - 1)..(x + 2) {
            for m in y..(y + 3) {
                if world.tile(l, m).is_some_and(|t| t.active()) {
                    flag = false;
                }
            }
        }
        let ceil = world.tile(x, y - 1);
        if !ceil.is_some_and(|t| {
            t.active() && TILE_SOLID.has(t.kind) && !TILE_SOLID_TOP.has(t.kind)
        }) {
            flag = false;
        }
    }
    if flag {
        let num2 = style * 18 * 3;
        // `:53597-53632` — row-major over the three rows at y+num..y+num+2.
        for row in 0..3 {
            for (col, fx) in [(x - 1, num2), (x, num2 + 18), (x + 1, num2 + 36)] {
                let py = y + num + row;
                if !in_world(world, col, py, 0) {
                    continue;
                }
                let i = world.index(col, py);
                let t = &mut world.tiles[i];
                t.flags |= FLAG_ACTIVE;
                t.frame_y = (row * 18) as i16;
                t.frame_x = fx as i16;
                t.kind = kind;
            }
        }
    }
}

/// `WorldGen.Place2x2Style` (`WorldGen.cs:52198-52256`): the style-column
/// 2×2 at `(x-1, y-1)` — the 5-tile bounds gate, the body empty, both
/// floors `SolidTile` (the coordinate overload — WITH the inactive veto).
/// The lihzahrd plate (254) adds the position-derived
/// `frameY = (x % 12 / 2) * 36` (`:52205-52208`) and demands a grass floor
/// (2/477/492/109, `:52231-52234`). `frameX = 36 * style + col * 18`,
/// `frameY = num + row * 18`.
pub(crate) fn place_2x2_style(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    if x < 5 || x > world.width - 5 || y < 5 || y > world.height - 5 {
        return;
    }
    let num: i16 = if kind == 254 {
        ((x % 12 / 2) * 36) as i16
    } else {
        0
    };
    let mut flag = true;
    for i in (x - 1)..(x + 1) {
        for j in (y - 1)..(y + 1) {
            if world.tile(i, j).is_some_and(|t| t.active()) {
                flag = false;
            }
        }
        if !solid_tile(world, i, y + 1) {
            flag = false;
        }
        if kind == 254
            && !world
                .tile(i, y + 1)
                .is_some_and(|t| matches!(t.kind, 2 | 477 | 492 | 109))
        {
            flag = false;
        }
    }
    if flag {
        let num2 = (36 * style) as i16;
        // `:52239-52254` — row-major, C# write order.
        for (col, row) in [(x - 1, y - 1), (x, y - 1), (x - 1, y), (x, y)] {
            let fx = num2 + if col == x { 18 } else { 0 };
            let fy = num + if row == y { 18 } else { 0 };
            let i = world.index(col, row);
            let t = &mut world.tiles[i];
            t.flags |= FLAG_ACTIVE;
            t.frame_y = fy;
            t.frame_x = fx;
            t.kind = kind;
        }
    }
}

/// `WorldGen.Place1x1` (`WorldGen.cs:45188-45235`): the general 1x1
/// frame-important object. `InWorld(x, y, 2)` (`:45190`). Two arms:
/// - the seashell (324, `:45204-45213`): a `SolidTile2` floor OR an active
///   `tileTable` top; NO empty-cell gate (it overwrites); `frameX =
///   22 * Next(3)` (the plain three-way roll — NOT
///   `RollRandomSeaShellStyle`, which only the dead inner switch arm
///   calls), `frameY = 22 * style`.
/// - the general arm (`:45214-45233`): not a mechanism-on-boulder pair
///   (`TileID.Sets.Wiring.IsAMechanism` / `Sets.Boulders`), a `SolidTile2`
///   floor and an empty cell; `frameX` stays stored except the 36/144/239
///   timers (`style * 18`, `frameY = 0`); the default writes
///   `frameY = style * 18` only.
///
/// The C# inner `case 324` (`:45220-45223`) is DEAD — the outer
/// `if (type == 324)` catches 324 first — so the port omits it (the
/// `RollRandomSeaShellStyle` draw never fires from this wrapper).
/// `tile2.type < 0` (`:45214`) is always false (`Tile.type` is `ushort`).
pub(crate) fn place_1x1(world: &mut World, rng: &mut Rng, x: i32, y: i32, kind: i32, style: i32) {
    if !in_world(world, x, y, 2) {
        return;
    }
    let below = world.tile(x, y + 1);
    if kind == 324 {
        // `:45206` — the floor solid OR an active table top.
        let table_top = below.is_some_and(|t| t.active() && tile_table(t.kind));
        if solid_tile2(world, x, y + 1) || table_top {
            // `:45208-45211` — the plain Next(3) variant roll.
            let frame_x = (22 * rng.range(0, 3)) as i16;
            let i = world.index(x, y);
            let t = &mut world.tiles[i];
            t.flags |= FLAG_ACTIVE;
            t.kind = kind as u16;
            t.frame_x = frame_x;
            t.frame_y = (22 * style) as i16;
        }
    } else {
        // `:45214` — the mechanism-on-boulder veto, the floor and the
        // empty-cell gate.
        let mechanism_on_boulder = kind >= 0
            && crate::tiledata::SET_IS_A_MECHANISM.has(kind as u16)
            && below.is_some_and(|t| crate::tiledata::SET_BOULDERS.has(t.kind));
        let empty = !world.tile(x, y).is_some_and(|t| t.active());
        if !mechanism_on_boulder && solid_tile2(world, x, y + 1) && empty {
            let i = world.index(x, y);
            let t = &mut world.tiles[i];
            t.flags |= FLAG_ACTIVE;
            t.kind = kind as u16;
            match kind {
                // `:45224-45229` — the timers: style into frameX, frameY 0.
                36 | 144 | 239 => {
                    t.frame_x = (style * 18) as i16;
                    t.frame_y = 0;
                }
                // `:45230-45232` — the default: frameY only, frameX stored.
                _ => {
                    t.frame_y = (style * 18) as i16;
                }
            }
        }
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

    /// `Place3x1` lays the three columns with the 54-px style stride and
    /// refuses when any floor column is missing.
    #[test]
    fn place_3x1_lays_three_columns_over_the_floor() {
        let mut w = floor_world();
        place_3x1(&mut w, 10, 19, 79, 2);
        for (col, fx) in [(9, 108), (10, 126), (11, 144)] {
            let t = w.tile(col, 19).unwrap();
            assert!(t.active());
            assert_eq!(t.kind, 79);
            assert_eq!(t.frame_x, fx);
            assert_eq!(t.frame_y, 0);
        }
        // A gap in the floor refuses the whole placement.
        let mut w2 = floor_world();
        let i = w2.index(21, 20);
        w2.tiles[i].flags &= !FLAG_ACTIVE;
        place_3x1(&mut w2, 20, 19, 79, 0);
        assert!(!w2.tile(20, 19).unwrap().active());
    }

    /// `Place1x1`: the seashell draws one plain Next(3) (22-px stride) and
    /// overwrites an occupied cell; the general arm keeps frameX stored
    /// and writes frameY = 18*style; the timer arm writes frameX instead.
    #[test]
    fn place_1x1_seashell_general_and_timer_arms() {
        let mut w = floor_world();
        let mut rng = Rng(11);
        let mut probe = Rng(11);
        let variant = probe.range(0, 3);
        place_1x1(&mut w, &mut rng, 10, 19, 324, 1);
        let t = w.tile(10, 19).unwrap();
        assert!(t.active() && t.kind == 324);
        assert_eq!(t.frame_x, (22 * variant) as i16);
        assert_eq!(t.frame_y, 22);
        assert_eq!(rng.0, probe.0); // exactly one draw
        // The general arm: frameX keeps its stored value, frameY = style*18.
        let i = w.index(12, 19);
        w.tiles[i].frame_x = 7;
        place_1x1(&mut w, &mut rng, 12, 19, 4, 3);
        let t = w.tile(12, 19).unwrap();
        assert!(t.active() && t.kind == 4);
        assert_eq!(t.frame_x, 7);
        assert_eq!(t.frame_y, 54);
        // The timer arm (36): frameX = style*18, frameY = 0.
        place_1x1(&mut w, &mut rng, 14, 19, 36, 2);
        let t = w.tile(14, 19).unwrap();
        assert_eq!(t.frame_x, 36);
        assert_eq!(t.frame_y, 0);
        // An occupied cell refuses the general arm (no overwrite).
        place_1x1(&mut w, &mut rng, 12, 19, 4, 0);
        assert_eq!(w.tile(12, 19).unwrap().frame_y, 54);
    }

    /// `Place2x2` lays the 2×2 at (x-1, y-1) with the 36-px style stride;
    /// the sink rides the 38-px stride and the Tile-overload floor (no
    /// inactive veto); the campfire refuses liquid.
    #[test]
    fn place_2x2_general_sink_and_campfire_arms() {
        let mut w = floor_world();
        place_2x2(&mut w, 10, 19, 79, 2);
        for (col, row, fx, fy) in [(9, 18, 0, 72), (10, 18, 18, 72), (9, 19, 0, 90), (10, 19, 18, 90)]
        {
            let t = w.tile(col, row).unwrap();
            assert!(t.active() && t.kind == 79);
            assert_eq!((t.frame_x, t.frame_y), (fx, fy));
        }
        // The sink (172): 38-px stride, and an actuated-inactive floor
        // still passes (the Tile overload has no inActive term).
        let mut w2 = floor_world();
        for col in [20, 21] {
            let i = w2.index(col, 20);
            w2.tiles[i].flags |= crate::tile::FLAG_INACTIVE;
        }
        place_2x2(&mut w2, 21, 19, 172, 1);
        assert!(w2.tile(20, 18).unwrap().active());
        assert_eq!(w2.tile(20, 19).unwrap().frame_y, 56); // 38 * 1 + 18
        // The campfire (98) refuses a body cell carrying liquid.
        let mut w3 = floor_world();
        let i = w3.index(29, 18);
        w3.tiles[i].liquid = 5;
        place_2x2(&mut w3, 30, 19, 98, 0);
        assert!(!w3.tile(29, 18).unwrap().active());
    }

    /// `Place2x2Style` lays the style columns; the lihzahrd plate (254)
    /// needs a grass floor and carries the position-derived frameY.
    #[test]
    fn place_2x2_style_and_the_lihzahrd_plate() {
        let mut w = floor_world();
        place_2x2_style(&mut w, 10, 19, 246, 3);
        let t = w.tile(9, 18).unwrap();
        assert!(t.active() && t.kind == 246);
        assert_eq!(t.frame_x, 108); // 36 * 3
        assert_eq!(t.frame_y, 0);
        assert_eq!(w.tile(10, 19).unwrap().frame_x, 126);
        // 254 refuses stone …
        place_2x2_style(&mut w, 20, 19, 254, 0);
        assert!(!w.tile(19, 18).unwrap().active());
        // … and places on grass with frameY = (x % 12 / 2) * 36.
        for col in [29, 30] {
            let i = w.index(col, 20);
            w.tiles[i].kind = 2;
        }
        place_2x2_style(&mut w, 30, 19, 254, 0);
        let t = w.tile(29, 18).unwrap();
        assert!(t.active() && t.kind == 254);
        assert_eq!(t.frame_y, ((30 % 12 / 2) * 36) as i16); // 108
        assert_eq!(w.tile(29, 19).unwrap().frame_y, 126);
    }

    /// `Place3x2` lays the six cells row-major; the picnic table (14/25)
    /// writes only row y; the altar refuses a boulder floor; the
    /// solidTop-frameY-0 arm accepts a platform floor.
    #[test]
    fn place_3x2_general_picnic_and_floor_arms() {
        let mut w = floor_world();
        place_3x2(&mut w, 10, 19, 26, 1);
        for (col, row, fx, fy) in [
            (9, 18, 54, 0),
            (10, 18, 72, 0),
            (11, 18, 90, 0),
            (9, 19, 54, 18),
            (10, 19, 72, 18),
            (11, 19, 90, 18),
        ] {
            let t = w.tile(col, row).unwrap();
            assert!(t.active() && t.kind == 26);
            assert_eq!((t.frame_x, t.frame_y), (fx, fy));
        }
        // The picnic table: only row y, frameY 0 everywhere.
        let mut w2 = floor_world();
        place_3x2(&mut w2, 20, 19, 14, 25);
        assert!(!w2.tile(19, 18).unwrap().active());
        assert!(w2.tile(19, 19).unwrap().active());
        assert_eq!(w2.tile(21, 19).unwrap().frame_x, (54 * 25 + 36) as i16);
        // The altar refuses a boulder (138) floor.
        let mut w3 = floor_world();
        let i = w3.index(30, 20);
        w3.tiles[i].kind = 138;
        place_3x2(&mut w3, 30, 19, 26, 0);
        assert!(!w3.tile(30, 18).unwrap().active());
        // The 285 arm accepts an active solidTop floor with frameY 0.
        let mut w4 = floor_world();
        for col in [39, 40, 41] {
            let i = w4.index(col, 20);
            w4.tiles[i].kind = 19; // platform: tileSolidTop, not tileSolid
            w4.tiles[i].frame_y = 0;
        }
        place_3x2(&mut w4, 40, 19, 285, 0);
        assert!(w4.tile(40, 18).unwrap().active());
        // … but a framed platform (frameY != 0) refuses.
        let mut w5 = floor_world();
        for col in [44, 45, 46] {
            let i = w5.index(col, 20);
            w5.tiles[i].kind = 19;
            w5.tiles[i].frame_y = 18;
        }
        place_3x2(&mut w5, 45, 19, 285, 0);
        assert!(!w5.tile(45, 18).unwrap().active());
    }

    /// `Place3x4`/`Place5x4` lay the tall bodies bottom-anchored at y with
    /// the 54-px style stride; the 5x4 frame columns centre on the stride.
    #[test]
    fn place_3x4_and_5x4_tall_bodies() {
        let mut w = floor_world();
        place_3x4(&mut w, 10, 19, 79, 1);
        let top = w.tile(9, 16).unwrap();
        assert!(top.active() && top.kind == 79);
        assert_eq!(top.frame_y, 0); // (3 + -3) * 18
        assert_eq!(top.frame_x, 54);
        assert_eq!(w.tile(11, 19).unwrap().frame_y, 54);
        assert_eq!(w.tile(11, 19).unwrap().frame_x, 90);
        let mut w2 = floor_world();
        place_5x4(&mut w2, 20, 19, 244, 1);
        assert!(w2.tile(18, 16).unwrap().active());
        assert_eq!(w2.tile(18, 16).unwrap().frame_x, 18); // 54 - 36
        assert_eq!(w2.tile(22, 19).unwrap().frame_x, 90); // 54 + 36
        // Style 0 keeps the negative sheet offset verbatim.
        let mut w3 = floor_world();
        place_5x4(&mut w3, 30, 19, 244, 0);
        assert_eq!(w3.tile(28, 16).unwrap().frame_x, -36);
    }

    /// `Place3x3`: the standing family sits on the floor (num=-2); the
    /// hanging family needs a solid ceiling; 753 stacks on 753.
    #[test]
    fn place_3x3_standing_hanging_and_the_753_stack() {
        let mut w = floor_world();
        place_3x3(&mut w, 10, 19, 219, 2); // standing: body y-2..y
        assert!(w.tile(9, 17).unwrap().active());
        assert_eq!(w.tile(9, 17).unwrap().frame_x, 108); // 54 * 2
        assert_eq!(w.tile(11, 19).unwrap().frame_y, 36);
        // Hanging (e.g. 42 lantern): needs the ceiling at (x, y-1).
        let mut w2 = floor_world();
        place_3x3(&mut w2, 20, 10, 42, 0); // no ceiling — refused
        assert!(!w2.tile(20, 10).unwrap().active());
        let i = w2.index(20, 9);
        w2.tiles[i].flags |= FLAG_ACTIVE;
        w2.tiles[i].kind = 1;
        place_3x3(&mut w2, 20, 10, 42, 1);
        assert!(w2.tile(19, 10).unwrap().active());
        assert_eq!(w2.tile(21, 12).unwrap().frame_y, 36);
        // 753 on an active 753 floor passes without SolidTile2.
        let mut w3 = World::new("ph".into(), 60, 60);
        let i = w3.index(30, 20);
        w3.tiles[i].flags |= FLAG_ACTIVE;
        w3.tiles[i].kind = 753;
        place_3x3(&mut w3, 30, 19, 753, 0);
        assert!(w3.tile(30, 17).unwrap().active());
    }

    /// `Place6x3` lays the bed column-major and accepts a platform floor;
    /// `Place4x2` flips to the +72 sheet half on direction 1.
    #[test]
    fn place_6x3_bed_and_4x2_direction_flip() {
        let mut w = floor_world();
        place_6x3(&mut w, 10, 19, 79);
        assert!(w.tile(7, 17).unwrap().active());
        assert_eq!(w.tile(7, 17).unwrap().frame_x, 0);
        assert_eq!(w.tile(12, 19).unwrap().frame_x, 90); // col 5 * 18
        assert_eq!(w.tile(12, 19).unwrap().frame_y, 36);
        // The platform (solidTop, frameY 0) floor passes.
        let mut w2 = floor_world();
        for col in 17..23 {
            let i = w2.index(col, 20);
            w2.tiles[i].kind = 19;
            w2.tiles[i].frame_y = 0;
        }
        place_6x3(&mut w2, 20, 19, 79);
        assert!(w2.tile(20, 17).unwrap().active());
        // 4x2: direction 1 shifts every frameX by 72.
        let mut w3 = floor_world();
        place_4x2(&mut w3, 30, 19, 90, 1, 2);
        let t = w3.tile(29, 18).unwrap();
        assert!(t.active() && t.kind == 90);
        assert_eq!(t.frame_x, 72);
        assert_eq!(t.frame_y, 72); // 36 * 2
        assert_eq!(w3.tile(32, 19).unwrap().frame_x, 126); // 72 + 54
        assert_eq!(w3.tile(32, 19).unwrap().frame_y, 90);
    }

    /// The wall paintings need a wall under every body cell; the frame
    /// strides and the style wraps match the C# per wrapper.
    #[test]
    fn wall_paintings_need_walls_and_wrap_their_styles() {
        let mut w = World::new("ph".into(), 60, 60);
        for x in 0..w.width {
            for y in 0..w.height {
                let i = w.index(x, y);
                w.tiles[i].wall = 1;
            }
        }
        place_3x2_wall(&mut w, 10, 10, 240, 2);
        assert!(w.tile(9, 10).unwrap().active());
        assert_eq!(w.tile(9, 10).unwrap().frame_y, 72); // 36 * 2
        assert_eq!(w.tile(11, 11).unwrap().frame_x, 36);
        // A wall-less cell refuses the whole painting.
        let i = w.index(20, 10);
        w.tiles[i].wall = 0;
        place_3x3_wall(&mut w, 20, 10, 241, 0);
        assert!(!w.tile(19, 9).unwrap().active());
        // 3x3 style wrap: style 36 → frameX 0, frameY 54.
        place_3x3_wall(&mut w, 30, 10, 241, 36);
        assert_eq!(w.tile(29, 9).unwrap().frame_x, 0);
        assert_eq!(w.tile(29, 9).unwrap().frame_y, 54);
        // 2x3 at (x, y-1): frameX = 36*style.
        place_2x3_wall(&mut w, 40, 10, 242, 1);
        assert_eq!(w.tile(40, 9).unwrap().frame_x, 36);
        assert_eq!(w.tile(41, 11).unwrap().frame_y, 36);
        // 4x3: frameY = 54*style.
        place_4x3_wall(&mut w, 50, 10, 243, 1);
        assert_eq!(w.tile(49, 9).unwrap().frame_y, 54);
        assert_eq!(w.tile(52, 11).unwrap().frame_x, 54);
        // 6x4 at (x-2, y-2): style 28 → frameX 108, frameY 72.
        place_6x4_wall(&mut w, 10, 30, 244, 28);
        assert_eq!(w.tile(8, 28).unwrap().frame_x, 108);
        assert_eq!(w.tile(8, 28).unwrap().frame_y, 72);
        assert_eq!(w.tile(13, 31).unwrap().frame_x, 198); // 108 + 90
    }
}