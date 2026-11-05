//! Tile edits shared by the player, wire tools and (later) NPCs.
//!
//! A local re-frame is enough after a single block change: only the edited tile and its
//! neighbours (up to two cells for the merge rules) can change frames or wall cells.

use crate::framing;
use crate::tile::Tile;
use crate::tileobject;
use crate::tiledata as td;
use crate::world::World;

/// `WallID.UnbreakableBlockWall` / `WallID.Count - 1` region used by the dungeon
/// control lines. Vanilla refuses it in every removal/replacement path:
/// `Player.CanPlayerSmashWall` (hammering), `WorldGen.ReplaceWall`, and it makes any
/// tile in front of it unbreakable (`WorldGen.CanKillTile`, `IsTileReplaceable`).
pub const WALL_UNBREAKABLE: u16 = 350;

/// One object destruction the kill cascade performed — the world-state effect
/// of a `WorldGen.Check*` routine (`WorldGen.cs:57923/:86339/:86432/:43335/
/// :46973/:49187`): the whole object was removed and vanilla would spawn
/// `Item.NewItem` with the rect origin at `(x, y)` (tile coordinates; the
/// rect is 32x32 px for furniture, 16x16 for doors). `tile_kind` is the
/// object's tile id; the item id resolves app-side through
/// `registry::tile_drop` (style-0 drops: door 25, table 32, chair 34, bench
/// 36, chest 48 — `WorldGen.cs:69204/:42194/:42635/:42447/:41677`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectDrop {
    pub tile_kind: u16,
    pub x: i32,
    pub y: i32,
}

/// The result of `kill_tile`: the kind of the hit cell plus every object
/// destruction the cascade performed, in `SquareTileFrame` visit order.
#[derive(Clone, Debug)]
pub struct TileKill {
    /// Tile kind of the killed (hit) cell.
    pub kind: u16,
    pub object_drops: Vec<ObjectDrop>,
}

/// `SquareTileFrame`'s `TileFrame` visit order (`WorldGen.cs:80185-80196`):
/// row-major over the 3x3 neighbourhood, self last of its row.
/// The ring walk's re-entrancy cap (`square_tile_frame` → the check → the
/// sweep → the ring → `square_tile_frame` — the vanilla terminates through
/// the objects dying; the cap guards the port's pathologies).
thread_local! {
    static RING_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

const SQUARE_FRAME_ORDER: [(i32, i32); 9] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 0),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

/// `WorldGen.KillTile` (`WorldGen.cs:63517`): kills the active tile at
/// `(x, y)`, keeping its background wall, then runs the destruction cascade —
/// `SquareTileFrame(i, j)` (:63919 → :80185) frames the 3x3 neighbourhood in
/// fixed order and the per-object `Check*` routines destroy any object that
/// lost a cell or its support, dropping its item. `no_item` mirrors the C#
/// `noItem` flag (worldgen/replication suppress the drop reporting; the
/// cascade itself still runs). Returns `None` when nothing was killed.
pub fn kill_tile(world: &mut World, x: i32, y: i32, no_item: bool) -> Option<TileKill> {
    let mut destroy_object = false;
    kill_tile_cascading(world, x, y, no_item, &mut destroy_object)
}

/// `WorldGen.SquareTileFrame(i, j, resetFrame: false)` (`Wiring.cs:3212`,
/// `:3229` — the `DeActive`/`ReActive` tails): the 3×3 `TileFrame` walk with
/// the frame kept. Returns the object drops the support checks produced (a
/// torch or a piece of furniture whose anchor actuated away pops, exactly
/// like the C# `TileFrame` → `Check*` → `destroyObject` chain). The actuated
/// tile itself stays "active" (the inactive state is a separate bit), so the
/// cascade treats it as a normal cell.
pub fn actuation_square_frame(world: &mut World, x: i32, y: i32) -> Vec<ObjectDrop> {
    let mut destroy_object = false;
    let mut kill = TileKill {
        kind: world.tile(x, y).map(|t| t.kind).unwrap_or(0),
        object_drops: Vec::new(),
    };
    square_tile_frame(world, x, y, false, &mut destroy_object, &mut kill);
    after_edit(world, x, y);
    kill.object_drops
}

/// The body of `KillTile` plus the shared `WorldGen.destroyObject` flag
/// (:4236). The checks re-entered through the cascade's own `KillTile` calls
/// early-return while the flag is set, exactly like the C# static.
fn kill_tile_cascading(
    world: &mut World,
    x: i32,
    y: i32,
    no_item: bool,
    destroy_object: &mut bool,
) -> Option<TileKill> {
    // :63519-63521 (bounds) and :63529-63532 (inactive) — `remove_tile`
    // covers both; the cell keeps its wall and liquid (:63885-63917 clear
    // everything but wall/liquid state on the C# side).
    let kind = remove_tile(world, x, y)?;
    let mut kill = TileKill {
        kind,
        object_drops: Vec::new(),
    };
    square_tile_frame(world, x, y, no_item, destroy_object, &mut kill);
    // The reframing half of the C# `TileFrame` calls: neighbours re-derive
    // their frames; `frame_important` kinds keep theirs and unsupported
    // torches fall (`reframe_area` = the non-object arm of `TileFrame`).
    after_edit(world, x, y);
    Some(kill)
}

/// `WorldGen.SquareTileFrame` (`WorldGen.cs:80185-80196`) with the
/// `TileFrame` guards inline: cells within 5 of the world edge are never
/// framed (:81330), inactive cells return early (:81348-81351), and
/// frame-important object kinds reach `TileFrameImportant`'s `Check*`
/// routines (:81353-81359, dispatch :84962+). Kills found here run their own
/// recursive `KillTile` (vanilla's inner `SquareTileFrame` chain).
fn square_tile_frame(
    world: &mut World,
    x: i32,
    y: i32,
    no_item: bool,
    destroy_object: &mut bool,
    kill: &mut TileKill,
) {
    for (dx, dy) in SQUARE_FRAME_ORDER {
        let (cx, cy) = (x + dx, y + dy);
        // `TileFrame` guard (:81330) — no framing within 5 tiles of the edge,
        // so no cascade either (objects cannot legitimately stand there:
        // `CanPlace`'s 5-tile margin, `TileObject.cs:215`).
        if cx <= 5 || cy <= 5 || cx >= world.width - 5 || cy >= world.height - 5 {
            continue;
        }
        let Some(cell) = world.tile(cx, cy) else {
            continue;
        };
        if !cell.active() || crate::tileobject::def_for_tile(cell.kind).is_none() {
            continue;
        }
        // `Check*` re-entrancy guard (`WorldGen.destroyObject`, read at
        // :57926/:86341/:86435/:43338/:46976/:49190).
        if *destroy_object {
            continue;
        }
        let Some(plan) = check_object(world, cx, cy) else {
            continue;
        };
        if !no_item {
            kill.object_drops.push(plan.drop);
        }
        // The shared tail of every `Check*`: `destroyObject = true`, kill
        // every object cell of the type (`WorldGen.cs:57941-57952`,
        // :86414-86428, :86494-86502, :43466-43472, :47108-47115,
        // :49416-49423), `destroyObject = false`. Each `KillTile` is a full
        // recursive kill (inner `SquareTileFrame`), but the flag keeps the
        // re-entered checks silent until the loop ends — a supported object
        // whose support died inside the loop stays put until a later break
        // near it (`WorldGen.cs:49841` resets the flag last in `Check3x2`).
        *destroy_object = true;
        for (mx, my) in plan.cells {
            if let Some(inner) = kill_tile_cascading(world, mx, my, no_item, destroy_object) {
                kill.object_drops.extend(inner.object_drops);
            }
        }
        *destroy_object = false;
        // The `Check*` tail's `TileFrame` ring (`WorldGen.cs:49841-49849`,
        // :50161-50168, :47499-47506): walked with `destroyObject` reset so a
        // neighbouring piece whose anchor died inside this sweep reaches its
        // own `Check*`. The port caps the recursion depth — the vanilla's
        // ring is a bounded per-check walk and the port's sweep re-enters
        // through `kill_tile_cascading`, so the cap only guards the
        // pathological nests.
        if let Some(((rx0, ry0), (rx1, ry1))) = plan.ring {
            RING_DEPTH.with(|d| {
                let depth = d.get();
                if depth < 8 {
                    d.set(depth + 1);
                    let (mid_y, mid_x) = ((ry0 + ry1) / 2, (rx0 + rx1) / 2);
                    for (mx, my) in [
                        (rx0, ry0),
                        (rx1, ry0),
                        (rx0, ry1),
                        (rx1, ry1),
                        (mid_x, ry0),
                        (mid_x, ry1),
                        (rx0, mid_y),
                        (rx1, mid_y),
                    ] {
                        square_tile_frame(world, mx, my, no_item, destroy_object, kill);
                    }
                    d.set(depth);
                }
            });
        }
    }
}

/// `WorldGen.SolidTile(Tile)` (`WorldGen.cs:70111-70131`): an active, solid,
/// non-solidTop, non-halfBrick, slopeless tile. Out-of-world cells are not
/// solid (the C# try/catch turns the OOB into `false`). The C# `!inActive()`
/// term is vacuous here — the port has no actuated-tile flag.
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && td::TILE_SOLID.has(t.kind)
        && !td::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
}

/// `WorldGen.SolidTileAllowBottomSlope` (`WorldGen.cs:70167-70185`): an
/// active, solid or solidTop, non-halfBrick tile whose top is not a slope.
/// Out-of-world and never-initialized cells count as solid (`InWorld`/null →
/// `true` — vanilla quirk, kept 1:1). The `PlatformProperTopFrame`
/// refinement only reaches platforms with a top slope, which cannot occur.
fn solid_tile_allow_bottom_slope(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && (td::TILE_SOLID.has(t.kind) || td::TILE_SOLID_TOP.has(t.kind))
        && !t.top_slope()
        && !t.half_brick()
}

/// The destruction plan of one `Check*` routine: the drop vanilla would
/// spawn and the object cells to `KillTile` (active cells of the object's
/// type — the exact cell guard each C# kill loop applies).
struct CheckPlan {
    drop: ObjectDrop,
    cells: Vec<(i32, i32)>,
    /// The `Check*` tail's `TileFrame` ring — only the checks that CLOSE with
    /// one in C# (`Check3x2` `WorldGen.cs:49841-49849`, `Check3x4`
    /// :50161-50168, `Check4x2` :47499-47506; the others — CheckOnTable1x1,
    /// Check1xX, Check2xX, Check1x2Top, Check2x2, Check3x3, CheckChest — have
    /// no tail, so their remainders float until a later break, the vanilla
    /// deferred-pop semantics).
    ring: Option<((i32, i32), (i32, i32))>,
}

/// The `TileFrameImportant` object dispatch (`WorldGen.cs:84962`): runs the
/// `Check*` routine for the cell's registry kind and returns its destruction
/// plan. Cell routines: 10 → `CheckDoorClosed` (:86432), 11 →
/// `CheckDoorOpen` (:86339), 14 → `Check3x2` (:49187), 15 → `Check1x2`
/// (:43335), 18 → `Check2x1` (:46973), 21 → `CheckChest` (:57923, via
/// `TileID.Sets.BasicChest` :85616-85619).
fn check_object(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let kind = world.tile(x, y)?.kind;
    match kind {
        10 => check_door_closed(world, x, y),
        11 => check_door_open(world, x, y),
        13 | 33 | 49 => check_on_table_1x1(world, x, y),
        14 | 87 | 88 => check_table(world, x, y),
        15 => check_chair(world, x, y),
        18 => check_bench(world, x, y),
        21 => check_chest(world, x, y),
        34 => check_chandelier(world, x, y),
        42 => check_lantern(world, x, y),
        79 | 90 => check_bed_bathtub(world, x, y),
        93 => check_lamp(world, x, y),
        100 | 172 => check_candelabra_sink(world, x, y),
        101 => check_bookcase(world, x, y),
        104 => check_clock(world, x, y),
        _ => None,
    }
}

/// `WorldGen.AnchorValid(tile, AnchorType.Table)` (`WorldGen.cs:51852-51886`),
/// the Table half: the platform top-frame windows, solid-top tiles, and the
/// `Main.tileTable` flat blocks.
fn anchor_valid_table(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if !(t.active() || t.has_any_liquid()) {
        return false;
    }
    if td::TILE_SOLID.has(t.kind) && td::SET_PLATFORMS.has(t.kind) {
        let num = t.frame_x / 18;
        if (!t.half_brick() && (0..=7).contains(&num))
            || (12..=16).contains(&num)
            || (25..=26).contains(&num)
        {
            return true;
        }
    }
    if td::TILE_SOLID.has(t.kind) && td::TILE_SOLID_TOP.has(t.kind) {
        return true;
    }
    !td::SET_PLATFORMS.has(t.kind)
        && crate::tileobject::is_tile_table(t.kind)
        && tileobject::block_type(t) == 0
}

/// `WorldGen.CheckOnTable1x1` (`WorldGen.cs:45237-45277`) — the placed bottle
/// (13) and the candle family (33/49): a 1x1 tile on a table-type surface.
/// The platform top-slope edge and the half-brick refusals are verbatim; the
/// tile-78 solid-floor exemption is the campfire (not in the registry subset).
fn check_on_table_1x1(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let kind = world.tile(x, y)?.kind;
    let below = world.tile(x, y + 1)?;
    let kill = || {
        Some(CheckPlan {
            drop: ObjectDrop { tile_kind: kind, x, y },
            cells: vec![(x, y)],
            ring: None,
        })
    };
    if below.top_slope() {
        if td::SET_PLATFORMS.has(below.kind) {
            let left_ok = world
                .tile(x - 1, y + 1)
                .is_some_and(|n| n.active() && tileobject::block_type(n) == 0 && td::SET_PLATFORMS.has(n.kind));
            let right_ok = world
                .tile(x + 1, y + 1)
                .is_some_and(|n| n.active() && tileobject::block_type(n) == 0 && td::SET_PLATFORMS.has(n.kind));
            let bt = tileobject::block_type(&below);
            if (bt != 3 || !left_ok) && (bt != 2 || !right_ok) {
                return kill();
            }
            None
        } else {
            kill()
        }
    } else if below.half_brick() {
        kill()
    } else if anchor_valid_table(world, x, y + 1)
        || (below.active() && crate::tileobject::is_tile_table(below.kind))
    {
        None
    } else {
        kill()
    }
}

/// `WorldGen.Check1xX` (`WorldGen.cs:37788-37878`) for the lamp (93): three
/// cells with the style-row frames and a solid floor below.
fn check_lamp(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let kind = world.tile(x, y)?.kind;
    let height = 3; // the lamppost's 6 waits for its registry entry
    let frame_x = world.tile(x, y)?.frame_x;
    let mut num2 = world.tile(x, y)?.frame_y as i32;
    let mut style_row = 0i32;
    while num2 >= 18 * height {
        num2 -= 18 * height;
        style_row += 1;
    }
    let oy = y - num2 / 18;
    let mut broken = false;
    let mut region = Vec::with_capacity(height as usize);
    for l in oy..oy + height {
        region.push((x, l));
        // The C# scan sets `flag` and CONTINUES (`WorldGen.cs:37809-37827`).
        let broken_cell = world.tile(x, l).is_none_or(|c| {
            !c.active()
                || c.kind != kind
                || c.frame_y != ((l - oy) * 18 + style_row * height * 18) as i16
                || c.frame_x != frame_x
        });
        if broken_cell {
            broken = true;
        }
    }
    if !solid_tile_allow_bottom_slope(world, x, oy + height) {
        broken = true;
    }
    if !broken {
        return None;
    }
    Some(CheckPlan {
        drop: ObjectDrop { tile_kind: kind, x, y },
        ring: None,
        cells: killable_cells(world, &region, kind),
         // floats until a later break (`WorldGen.cs`'s check_lamp).
    })
}

/// `WorldGen.Check2xX` (`WorldGen.cs:37993-38147`) for the grandfather clock
/// (104): the 2-wide column pair, five cells tall, the style column from
/// `frameX % 36`, and the two solid floor tiles below.
fn check_clock(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let kind = world.tile(x, y)?.kind;
    let t = world.tile(x, y)?;
    let frame_x = t.frame_x;
    let mut ox = x;
    if frame_x % 36 == 18 {
        ox -= 1;
    }
    let mut num3 = world.tile(ox, y)?.frame_y as i32 / 18;
    let height = 5;
    let mut style_offset = 0i32;
    while num3 >= height {
        num3 -= height;
        style_offset += height * 18;
    }
    let oy = y - num3;
    let top_frame_x = world.tile(ox, y)?.frame_x;
    let mut broken = false;
    let mut region = Vec::with_capacity(10);
    for l in oy..oy + height {
        for (dx, fx) in [(0i32, top_frame_x), (1, top_frame_x + 18)] {
            let k = ox + dx;
            region.push((k, l));
            // The C# scan sets `flag` and CONTINUES — no early break, or the
            // sweep's cell list loses the rest of the row
            // (`WorldGen.cs:38003-38040`).
            let broken_cell = world.tile(k, l).is_none_or(|c| {
                !c.active()
                    || c.kind != kind
                    || c.frame_y != ((l - oy) * 18 + style_offset) as i16
                    || c.frame_x != fx
            });
            if broken_cell {
                broken = true;
            }
        }
    }
    if !solid_tile_allow_bottom_slope(world, ox, oy + height)
        || !solid_tile_allow_bottom_slope(world, ox + 1, oy + height)
    {
        broken = true;
    }
    if !broken {
        return None;
    }
    Some(CheckPlan {
        drop: ObjectDrop { tile_kind: kind, x, y },
        ring: None,
        cells: killable_cells(world, &region, kind),
         // floats until a later break (`WorldGen.cs`'s check_clock).
    })
}

/// `WorldGen.Check3x4` (`WorldGen.cs:50076-50162`) for the bookcase (101).
fn check_bookcase(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let kind = world.tile(x, y)?.kind;
    let t = world.tile(x, y)?;
    let mut style_col = t.frame_x as i32 / 18;
    let mut style_row = t.frame_y as i32 / 18;
    let mut sx = 0i32;
    let mut sy = 0i32;
    while style_col >= 3 {
        style_col -= 3;
        sx += 1;
    }
    while style_row >= 4 {
        style_row -= 4;
        sy += 1;
    }
    let ox = x - style_col;
    let oy = y - style_row;
    let mut broken = false;
    let mut region = Vec::with_capacity(12);
    for k in ox..ox + 3 {
        for l in oy..oy + 4 {
            region.push((k, l));
            let expected = (
                (sx * 54 + (k - ox) * 18) as i16,
                (sy * 72 + (l - oy) * 18) as i16,
            );
            if world
                .tile(k, l)
                .is_none_or(|c| !c.active() || c.kind != kind || (c.frame_x, c.frame_y) != expected)
            {
                broken = true;
            }
        }
        if !solid_tile_allow_bottom_slope(world, k, oy + 4) {
            broken = true;
        }
    }
    if !broken {
        return None;
    }
    Some(CheckPlan {
        drop: ObjectDrop { tile_kind: kind, x, y },
        cells: killable_cells(world, &region, kind),
        ring: Some(region_ring(&region)),
    })
}

/// `WorldGen.Check1x2Top` (`WorldGen.cs:46706-46776`) for the hanging lantern
/// (42): the two cells carry the style-row frames and the tile ABOVE is a
/// solid (or a platform — the placement alternate's anchor).
fn check_lantern(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let kind = world.tile(x, y)?.kind;
    let mut num2 = world.tile(x, y)?.frame_y as i32 / 18;
    let mut style_row = 0i32;
    while num2 >= 2 {
        num2 -= 2;
        style_row += 1;
    }
    let oy = y - num2;
    let top_frame = (style_row * 36) as i16;
    let mut broken = false;
    let mut region = vec![(x, oy), (x, oy + 1)];
    let top = world.tile(x, oy);
    let bot = world.tile(x, oy + 1);
    if !top.is_some_and(|c| c.active() && c.kind == kind && c.frame_y == top_frame)
        || !bot.is_some_and(|c| c.active() && c.kind == kind && c.frame_y == top_frame + 18)
    {
        broken = true;
    }
    // The anchor: solid above; the platform flag mirrors the placement
    // alternate (`flag4`, `WorldGen.cs:46752-46756` — the rope branch is the
    // Dead-Cells jar's, not the lantern's).
    let above = world.tile(x, oy - 1);
    let platform_ok = above.is_some_and(|a| {
        a.active() && (td::SET_PLATFORMS.has(a.kind) || a.kind == 380)
    });
    let solid_ok = above.is_some_and(|a| {
        (a.active() || a.has_any_liquid()) && td::TILE_SOLID.has(a.kind) && !td::TILE_SOLID_TOP.has(a.kind)
    });
    if !platform_ok && !solid_ok {
        broken = true;
    }
    if !broken {
        return None;
    }
    Some(CheckPlan {
        drop: ObjectDrop { tile_kind: kind, x, y },
        ring: None,
        cells: killable_cells(world, &region, kind),
         // floats until a later break (`WorldGen.cs`'s check_lantern).
    })
}

/// `WorldGen.Check4x2` (`WorldGen.cs:47408-47500`) for the bed (79) and the
/// bathtub (90): the four cells carry the style frames (the second style
/// column continues at frameX 72) and the floor row is solid-or-platform.
fn check_bed_bathtub(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let kind = world.tile(x, y)?.kind;
    let t = world.tile(x, y)?;
    let frame_x = t.frame_x as i32;
    let mut ox = x + frame_x / 18 * -1;
    if (kind == 79 || kind == 90) && frame_x >= 72 {
        ox += 4;
    }
    let mut num2 = t.frame_y as i32 / 18;
    let mut style_row = 0i32;
    while num2 > 1 {
        num2 -= 2;
        style_row += 1;
    }
    let oy = y - num2;
    let second_col = frame_x >= 72;
    let mut broken = false;
    let mut region = Vec::with_capacity(8);
    for k in ox..ox + 4 {
        for l in oy..oy + 2 {
            region.push((k, l));
            let fx = if second_col { (k - ox + 4) * 18 } else { (k - ox) * 18 };
            let expected = (fx as i16, ((l - oy) * 18 + style_row * 36) as i16);
            if world
                .tile(k, l)
                .is_none_or(|c| !c.active() || c.kind != kind || (c.frame_x, c.frame_y) != expected)
            {
                broken = true;
            }
        }
        // `SolidTileAllowBottomSlope` or a platform (`:47456-47463`).
        let floor = world.tile(k, oy + 2);
        let floor_ok = floor.is_some_and(|f| {
            solid_tile_allow_bottom_slope(world, k, oy + 2)
                || (f.active() && td::SET_PLATFORMS.has(f.kind))
        });
        if !floor_ok {
            broken = true;
        }
    }
    if !broken {
        return None;
    }
    Some(CheckPlan {
        drop: ObjectDrop { tile_kind: kind, x, y },
        cells: killable_cells(world, &region, kind),
        ring: Some(region_ring(&region)),
    })
}

/// `WorldGen.Check2x2`'s default branch (`WorldGen.cs:48397-48470`) for the
/// candelabra (100) and the sink (172): the 2x2 cells with the style frames
/// (the sink's full height is 38) and the solid-or-table floor without
/// half-bricks.
fn check_candelabra_sink(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let kind = world.tile(x, y)?.kind;
    let t = world.tile(x, y)?;
    let full = if kind == 172 { 38 } else { 36 };
    let mut num = 0i32;
    let mut ox = x - t.frame_x as i32 / 18;
    if ox < x - 1 {
        ox += 2;
        num = 36;
    }
    let mut num4 = t.frame_y as i32;
    let mut style_row = 0i32;
    while num4 >= full {
        num4 -= full;
        style_row += 1;
    }
    let oy = y - num4 / 18;
    let mut broken = false;
    let mut region = Vec::with_capacity(4);
    for k in ox..ox + 2 {
        for l in oy..oy + 2 {
            region.push((k, l));
            let expected = (
                ((k - ox) * 18 + num) as i16,
                ((l - oy) * 18 + style_row * full) as i16,
            );
            if world
                .tile(k, l)
                .is_none_or(|c| !c.active() || c.kind != kind || (c.frame_x, c.frame_y) != expected)
            {
                broken = true;
            }
        }
        // `:48447-48456`: solid or `Main.tileTable`, no half-bricks.
        let floor = world.tile(k, oy + 2);
        let floor_ok = floor.is_some_and(|f| {
            f.active() && (td::TILE_SOLID.has(f.kind) || crate::tileobject::is_tile_table(f.kind))
        }) && !floor.is_some_and(|f| f.half_brick());
        if !floor_ok {
            broken = true;
        }
    }
    if !broken {
        return None;
    }
    Some(CheckPlan {
        drop: ObjectDrop { tile_kind: kind, x, y },
        ring: None,
        cells: killable_cells(world, &region, kind),
         // floats until a later break (`WorldGen.cs`'s check_candelabra_sink).
    })
}

/// `WorldGen.Check3x3`'s else branch (`WorldGen.cs:53059-53145`) for the
/// chandelier (34): the nine cells carry the style frames and the ceiling
/// tile above the middle column is a solid non-solid-top.
fn check_chandelier(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let kind = world.tile(x, y)?.kind;
    let t = world.tile(x, y)?;
    let mut style_col = t.frame_x as i32 / 18;
    let mut sx = 0i32;
    while style_col >= 3 {
        style_col -= 3;
        sx += 1;
    }
    let ox = x - style_col;
    let style_x = 54 * sx;
    let style_row = t.frame_y as i32 / 54;
    let oy = y - (t.frame_y as i32 % 54 / 18);
    let mut broken = false;
    let mut region = Vec::with_capacity(9);
    for k in ox..ox + 3 {
        for l in oy..oy + 3 {
            region.push((k, l));
            let expected = (
                ((k - ox) * 18 + style_x) as i16,
                ((l - oy) * 18 + style_row * 54) as i16,
            );
            if world
                .tile(k, l)
                .is_none_or(|c| !c.active() || c.kind != kind || (c.frame_x, c.frame_y) != expected)
            {
                broken = true;
            }
        }
    }
    // `:53117-53124`: the ceiling anchor above the origin column.
    let ceil = world.tile(ox + 1, oy - 1);
    let ceil_ok = ceil.is_some_and(|c| {
        (c.active() || c.has_any_liquid()) && td::TILE_SOLID.has(c.kind) && !td::TILE_SOLID_TOP.has(c.kind)
    });
    if !ceil_ok {
        broken = true;
    }
    if !broken {
        return None;
    }
    Some(CheckPlan {
        drop: ObjectDrop { tile_kind: kind, x, y },
        ring: None,
        cells: killable_cells(world, &region, kind),
         // floats until a later break (`WorldGen.cs`'s check_chandelier).
    })
}

/// The `Check*` tail's `TileFrame` ring: the piece's bbox expanded by one.
fn region_ring(region: &[(i32, i32)]) -> ((i32, i32), (i32, i32)) {
    let (mut x0, mut y0) = (i32::MAX, i32::MAX);
    let (mut x1, mut y1) = (i32::MIN, i32::MIN);
    for (x, y) in region {
        x0 = x0.min(*x);
        y0 = y0.min(*y);
        x1 = x1.max(*x);
        y1 = y1.max(*y);
    }
    (x0 - 1, y0 - 1);
    (x1 + 1, y1 + 1);
    ((x0 - 1, y0 - 1), (x1 + 1, y1 + 1))
}

/// Cells of `region` that are active and of `kind` — the cell guard shared by
/// every C# kill loop.
fn killable_cells(world: &World, region: &[(i32, i32)], kind: u16) -> Vec<(i32, i32)> {
    region
        .iter()
        .copied()
        .filter(|&(x, y)| {
            world
                .tile(x, y)
                .is_some_and(|t| t.active() && t.kind == kind)
        })
        .collect()
}

/// `WorldGen.CheckDoorClosed` (`WorldGen.cs:86432-86511`): the 3-tall closed
/// door needs `SolidTile` above and below its column and all three cells
/// active type-10. The drop is `DropDoorItem(i, j)` (:86508) — 16x16 at the
/// trigger cell; the style-0 door item is 25 (`GetDoorItem`, :69200-69204,
/// `doorStyle = frameY / 54 + frameX / 54 * 36` :86438 — the port's registry
/// slice is style 0).
fn check_door_closed(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let t = world.tile(x, y)?;
    let oy = y - (t.frame_y as i32 % 54) / 18; // :86439
    let broken = !solid_tile(world, x, oy - 1)
        || !solid_tile(world, x, oy + 3)
        || (0..3).any(|dy| {
            world
                .tile(x, oy + dy)
                .is_none_or(|c| !c.active() || c.kind != 10)
        });
    if !broken {
        return None;
    }
    let region: Vec<(i32, i32)> = (0..3).map(|dy| (x, oy + dy)).collect();
    Some(CheckPlan {
        drop: ObjectDrop {
            tile_kind: 10,
            x,
            y,
        },
        ring: None,
        cells: killable_cells(world, &region, 10),
         // floats until a later break (`WorldGen.cs`'s check_door_closed).
    })
}

/// `WorldGen.CheckDoorOpen` (`WorldGen.cs:86339-86430`): the open door is a
/// 2x3 region; the hinge column follows the `frameX % 72` switch
/// (:86355-86368), solidity is checked above/below the hinge
/// (:86376-86387), and all six cells must be active type-11 (:86396-86412).
/// Drop = `DropDoorItem(i, j)` (:86415), 16x16 at the trigger cell.
fn check_door_open(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let t = world.tile(x, y)?;
    let oy = y - (t.frame_y as i32 % 54) / 18; // :86349
    let (hinge, leftmost) = match t.frame_x as i32 % 72 {
        0 => (x, x),
        18 => (x - 1, x - 1),
        36 => (x + 1, x),
        54 => (x, x - 1),
        // Valid open-door frames are 18 px multiples; anything else degrades
        // to the first switch case (the C# would read an uninitialised `num`).
        _ => (x, x),
    };
    let mut broken = !solid_tile(world, hinge, oy - 1) || !solid_tile(world, hinge, oy + 3);
    let mut region = Vec::with_capacity(6);
    for mx in leftmost..leftmost + 2 {
        for my in oy..oy + 3 {
            region.push((mx, my));
            if world
                .tile(mx, my)
                .is_none_or(|c| !c.active() || c.kind != 11)
            {
                broken = true;
            }
        }
    }
    if !broken {
        return None;
    }
    Some(CheckPlan {
        drop: ObjectDrop {
            tile_kind: 11,
            x,
            y,
        },
        ring: None,
        cells: killable_cells(world, &region, 11),
         // floats until a later break (`WorldGen.cs`'s check_door_open).
    })
}

/// `WorldGen.Check3x2` (`WorldGen.cs:49187-49412`) for the wooden table
/// (type 14): origin from `frameY % 36 / 18` and `frameX / 18` mod 3
/// (:49199-49209 — the hardcoded 36 is the `Style3x2` full height; table
/// styles are horizontal with `StyleWrapLimit = 0`, so `frameY` only ever
/// encodes the row), every cell must be active type-14 with the expected
/// frames (:49219-49226), and `SolidTileAllowBottomSlope` below each column
/// (the final `else if`, :49396-49399). Drop = `GetItemDrop_Tables(style)`
/// (:49424-49426), 32x32 at the trigger cell; style 0 → 32 (:42194).
fn check_table(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let kind = world.tile(x, y)?.kind;
    let t = world.tile(x, y)?;
    let frame_x = t.frame_x as i32;
    let frame_y = t.frame_y as i32;
    let oy = y - frame_y % 36 / 18;
    let style_row = frame_y / 36;
    let mut dx = frame_x / 18;
    let mut style = 0;
    while dx > 2 {
        dx -= 3;
        style += 1;
    }
    let ox = x - dx;
    let mut broken = false;
    let mut region = Vec::with_capacity(6);
    for k in ox..ox + 3 {
        for l in oy..oy + 2 {
            region.push((k, l));
            let expected = (
                ((k - ox) * 18 + style * 54) as i16,
                ((l - oy) * 18 + style_row * 36) as i16,
            );
            if world
                .tile(k, l)
                .is_none_or(|c| !c.active() || c.kind != kind || (c.frame_x, c.frame_y) != expected)
            {
                broken = true;
            }
        }
        if !solid_tile_allow_bottom_slope(world, k, oy + 2) {
            broken = true;
        }
    }
    if !broken {
        return None;
    }
    // `GetItemDrop_Tables/Pianos/Dressers(style)` — the slice covers style 0.
    let _ = style;
    Some(CheckPlan {
        drop: ObjectDrop {
            tile_kind: kind,
            x,
            y,
        },
        cells: killable_cells(world, &region, kind),
        ring: Some(region_ring(&region)),
    })
}

/// `WorldGen.Check1x2` (`WorldGen.cs:43335-43501`) for the wooden chair
/// (type 15): origin row from `frameY % 40` (:43343-43349 — 40 is the
/// chair's `CoordinateFullHeight`, so `frameY / 40` is the style row), both
/// cells must be type-15 with frames `40*style` / `40*style + 18`
/// (:43367-43369), and the support below allows platforms explicitly
/// (:43457-43459). Drop = `GetItemDrop_Chair(style)` (:43494), 32x32 at
/// (trigger x, **origin y**); style 0 → 34 (:42635).
fn check_chair(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let t = world.tile(x, y)?;
    let frame_y = t.frame_y as i32;
    let style_row = frame_y / 40;
    let oy = if frame_y % 40 == 18 { y - 1 } else { y };
    let top = world.tile(x, oy);
    let bottom = world.tile(x, oy + 1);
    let intact = top.is_some_and(|c| c.frame_y as i32 == style_row * 40 && c.kind == 15)
        && bottom.is_some_and(|c| c.frame_y as i32 == style_row * 40 + 18 && c.kind == 15);
    let supported = solid_tile_allow_bottom_slope(world, x, oy + 2)
        || world
            .tile(x, oy + 2)
            .is_some_and(|c| td::SET_PLATFORMS.has(c.kind));
    if intact && supported {
        return None;
    }
    let region = [(x, oy), (x, oy + 1)];
    Some(CheckPlan {
        drop: ObjectDrop {
            tile_kind: 15,
            x,
            y: oy,
        },
        ring: None,
        cells: killable_cells(world, &region, 15),
         // floats until a later break (`WorldGen.cs`'s check_chair).
    })
}

/// `WorldGen.Check2x1` (`WorldGen.cs:46973-47140`) for the work bench
/// (type 18): origin column from `frameX / 18 % 2` (:46981-46988), the right
/// cell's `frameX` must continue the left's by 18 with both active type-18
/// (:47016-47020), and `SolidTileAllowBottomSlope` below both cells (the
/// default support branch, :47026-47031). Drop = `GetItemDrop_Workbenches
/// (frameX / 36)` (:47121-47124), 32x32 at (**origin x**, trigger y); style
/// 0 → 36 (:42447).
fn check_bench(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let t = world.tile(x, y)?;
    let frame_x = t.frame_x as i32;
    let ox = x - frame_x / 18 % 2;
    let left = world.tile(ox, y);
    let right = world.tile(ox + 1, y);
    let intact = right.is_some_and(|c| {
        c.frame_x as i32 == left.map(|l| l.frame_x as i32).unwrap_or(i32::MIN) + 18
    }) && left.is_some_and(|c| c.active() && c.kind == 18)
        && right.is_some_and(|c| c.active() && c.kind == 18)
        && solid_tile_allow_bottom_slope(world, ox, y + 1)
        && solid_tile_allow_bottom_slope(world, ox + 1, y + 1);
    if intact {
        return None;
    }
    let region = [(ox, y), (ox + 1, y)];
    Some(CheckPlan {
        drop: ObjectDrop {
            tile_kind: 18,
            x: ox,
            y,
        },
        ring: None,
        cells: killable_cells(world, &region, 18),
         // floats until a later break (`WorldGen.cs`'s check_bench).
    })
}

/// `WorldGen.CheckChest` (`WorldGen.cs:57923-57984`): the 2x2 chest needs
/// every cell active type-21 with frames `frameX / 18 % 2 == dx` and
/// `frameY == dy * 18` (:57936-57949), and `Main.tileSolid` below both
/// columns (:57950-57958 — platforms pass: `Main.tileSolid[19] = true`,
/// `Main.cs:7893`). The `Chest.CanDestroyChest` gate (:57958) is always true
/// with no chest entities (M3-CHEST-UI seam). Drop =
/// `GetItemDrop_Chests(frameX / 36)` (:57969-57970), 32x32 at the trigger
/// cell; style 0 → 48 (:41677-41681).
fn check_chest(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let t = world.tile(x, y)?;
    let frame_x = t.frame_x as i32;
    let frame_y = t.frame_y as i32;
    let ox = x - (frame_x / 18 % 2);
    let oy = y - frame_y / 18;
    let mut broken = false;
    let mut region = Vec::with_capacity(6); // the kill loop spans 2x3 (:57942-57952)
    for k in ox..ox + 2 {
        for l in oy..oy + 2 {
            region.push((k, l));
            let expected = (((k - ox) * 18) as i16, ((l - oy) * 18) as i16);
            if world
                .tile(k, l)
                .is_none_or(|c| !c.active() || c.kind != 21 || (c.frame_x, c.frame_y) != expected)
            {
                broken = true;
            }
        }
        // Support row (:57950-57958).
        if world
            .tile(k, oy + 2)
            .is_none_or(|c| !c.active() || !td::TILE_SOLID.has(c.kind))
        {
            broken = true;
        }
        region.push((k, oy + 2));
    }
    if !broken {
        return None;
    }
    Some(CheckPlan {
        drop: ObjectDrop {
            tile_kind: 21,
            x,
            y,
        },
        ring: None,
        cells: killable_cells(world, &region, 21),
         // floats until a later break (`WorldGen.cs`'s check_chest).
    })
}

/// Clears the active tile at `(x, y)` and keeps the wall and the liquid. The caller is
/// responsible for re-framing; `kill_tile` is the version that does it.
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

/// The result of the right-click interaction dispatcher: what the click did,
/// and every object destruction its cleanup kills performed (the port of
/// vanilla's `Item.NewItem` side effects — `WorldGen.cs:86508` and peers).
/// `None` = the tile has no interaction; the caller falls through to the
/// place path.
#[derive(Debug)]
pub enum TileInteraction {
    None,
    /// `WorldGen.OpenDoor` succeeded (`SoundEngine.PlaySound(8)`, `:37732`).
    DoorOpened {
        object_drops: Vec<ObjectDrop>,
    },
    /// `WorldGen.CloseDoor` succeeded (`SoundEngine.PlaySound(9)`, `:31906`).
    DoorClosed {
        object_drops: Vec<ObjectDrop>,
    },
    /// The door branch ran but the toggle was refused (blocked, partial
    /// door). Vanilla still consumes the click (`Player.cs:32836` sets
    /// `flag2` before attempting) and the validate-cleanup kills may have
    /// dropped objects; no sound plays on this path (`:37732`/`:31906` sit
    /// behind every failure return).
    DoorBlocked {
        object_drops: Vec<ObjectDrop>,
    },
    /// The dial branch fired: `Main.Sundialing()`/`Moondialing()` armed the
    /// fast-forward (`Main.cs:6178-6180`/:6192-6194, SP branch). `sundial`
    /// picks which dial — the cue is the same `SoundID.Item4` for both
    /// (`Player.cs:32642`/:32652).
    DialStarted {
        sundial: bool,
    },
    /// The candle lit-toggle ran (`Player.cs:32378-32390`): `frameX +=
    /// (frameX > 0) ? -18 : 18`. No sound; the flame overlay and the light
    /// emission follow the frame.
    CandleToggled,
    /// The chair sit decoded (`PlayerSittingHelper.GetSittingTargetInfo`'s
    /// case 15, `PlayerSittingHelper.cs:163-182`): the seat world position is
    /// the chair's bottom-centre shifted `targetDirection * 6` px, the
    /// facing flips with `frameX`.
    ChairSit {
        seat_x: f32,
        seat_y: f32,
        direction: i32,
    },
    /// The bed branch (`Player.cs:32654-32692`). `set_spawn` picks the half:
    /// the bottom side (`IsHoveringOverABottomSideOfABed`) runs
    /// `FindSpawn`/`CheckSpawn`/`ChangeSpawn`; the top half is
    /// `sleeping.StartSleeping` — the sleep state itself is not ported yet,
    /// so the click is consumed and inert (documented in ANALYSIS).
    BedInteract {
        set_spawn: bool,
        spawn_x: i32,
        spawn_y: i32,
    },
    /// The grandfather clock's time read (`Player.cs:32725-32759`).
    ClockRead {
        text: String,
    },
    /// The dial branch ran but the gate refused (already fast-forwarding to
    /// that target or the cooldown is running, `Player.cs:32638`/:32648).
    /// Vanilla still consumes the click (`flag2 = true`, `:32637`); no state
    /// changes and no sound plays. No object drops — the dial arms kill no
    /// tiles.
    DialRefused,
}

/// The world-interaction dispatcher — the port of the right-click tile hook:
/// `Player.LookForTileInteractions` (`Player.cs:31937`) →
/// `TileInteractionsCheck` (`:32031`, active tiles only) →
/// `TileInteractionsUse` (`:32118`), whose type chain routes tile 10/11
/// through the door branch (`:32835-32865`) and tiles 356/663 through the
/// dial branches (`:32636-32653`). The port dispatches the doors on the
/// registry def's `interaction` tag instead of a type chain (Decision #3) so
/// chests/signs can join as data later; the dials keep the C# kind-literal
/// shape (see the arm below). `direction` is `Player.direction`
/// (`:32841`), `cell_blocked` the entity half of `Collision.EmptyTile`
/// (`Collision.cs:1359`, called with `ignoreTiles: true` — the door ops port
/// the tile half themselves and delegate entities to the caller).
pub fn interact_at(
    world: &mut World,
    i: i32,
    j: i32,
    direction: i32,
    cell_blocked: &dyn Fn(i32, i32) -> bool,
) -> TileInteraction {
    // `TileInteractionsCheck` (`Player.cs:32031-32041`): active tiles only.
    let Some(t) = world.tile(i, j) else {
        return TileInteraction::None;
    };
    if !t.active() {
        return TileInteraction::None;
    }
    // The dial branches (`Player.cs:32636-32653`): the C# chain tests
    // `Main.tile[myX, myY].type` literals here. The port's registry has no
    // defs for 356/663 (they place through the legacy single-tile route), so
    // this arm reads the kind the same way, before the registry lookup.
    if t.kind == 356 || t.kind == 663 {
        return dial_interaction(world, t.kind == 356);
    }
    // The port routes on the registry def's `interaction` tag (Decision #3)
    // where the C# chain tests `Main.tile[myX, myY].type` literals
    // (`Player.cs:32835`/:32857 for 10/11).
    let Some(def) = crate::tileobject::def_for_tile(t.kind) else {
        return TileInteraction::None;
    };
    match def.interaction {
        crate::tileobject::Interaction::Candle => {
            // `Player.cs:32378-32390`: `num27 = (frameX > 0) ? -18 : 18;
            // tile.frameX += num27` — the lit/unlit toggle, no sound.
            if let Some(mt) = world.tile_mut(i, j) {
                if mt.frame_x > 0 {
                    mt.frame_x -= 18;
                } else {
                    mt.frame_x += 18;
                }
            }
            TileInteraction::CandleToggled
        }
        crate::tileobject::Interaction::Chair => {
            // `Player.cs:32313-32321` + `PlayerSittingHelper.SitDown`'s
            // target decode (case 15, `PlayerSittingHelper.cs:163-182`).
            // The range gate (`IsWithinSnappngRangeToTile(myX, myY, 40)`)
            // is the caller's — the click distance is already smart-gated.
            chair_sit_target(world, i, j)
        }
        crate::tileobject::Interaction::Bed => {
            // `Player.cs:32654-32692`: the frame decode finds the bed's
            // spawn cell; the bottom half sets/removes the spawn, the top
            // half starts sleeping (the sleep state is not ported — the
            // click stays consumed, `flag2 = true` at `:32655`).
            let frame_x = t.frame_x as i32;
            let mut num36 = i + frame_x / 18 * -1;
            if frame_x < 72 {
                num36 += 2;
            } else {
                num36 += 4 + 1;
            }
            let mut num38 = t.frame_y as i32 / 18;
            while num38 > 1 {
                num38 -= 2;
            }
            let num37 = j - num38 + 2;
            let bottom_side = {
                let flip = frame_x / 72 == 1;
                let mut flag2 = frame_x % 72 < 36;
                if flip {
                    flag2 = !flag2;
                }
                flag2
            };
            TileInteraction::BedInteract {
                set_spawn: bottom_side,
                spawn_x: num36,
                spawn_y: num37,
            }
        }
        crate::tileobject::Interaction::Clock => {
            // `Player.cs:32725-32759`: the in-game time, `h:mm AM/PM`.
            let mut num44 = world.time;
            if !world.day_time {
                num44 += 54000.0;
            }
            num44 = num44 / 86400.0 * 24.0 - 7.5 - 12.0;
            if num44 < 0.0 {
                num44 += 24.0;
            }
            let mut am_pm = "AM";
            if num44 >= 12.0 {
                am_pm = "PM";
            }
            let mut num45 = num44 as i32;
            let num46 = ((num44 - num45 as f64) * 60.0) as i32;
            if num45 > 12 {
                num45 -= 12;
            }
            if num45 == 0 {
                num45 = 12;
            }
            TileInteraction::ClockRead {
                text: format!("{num45}:{num46:02} {am_pm}"),
            }
        }
        crate::tileobject::Interaction::None => TileInteraction::None,
        crate::tileobject::Interaction::Door => {
            let mut drops = Vec::new();
            if t.kind == 10 {
                // The closed-door branch (`Player.cs:32835-32856`): open with
                // the player's facing; while the tile is still a closed door,
                // retry with the opposite direction. After a validate-kill
                // the cell no longer has kind 10 (`KillTile` resets `type`,
                // `WorldGen.cs:63915`), so the cleanup path skips the retry.
                if open_door(world, i, j, direction, &mut drops) {
                    return TileInteraction::DoorOpened {
                        object_drops: drops,
                    };
                }
                if world.tile(i, j).is_some_and(|t| t.kind == 10)
                    && open_door(world, i, j, -direction, &mut drops)
                {
                    return TileInteraction::DoorOpened {
                        object_drops: drops,
                    };
                }
                TileInteraction::DoorBlocked {
                    object_drops: drops,
                }
            } else if close_door(world, i, j, false, cell_blocked, &mut drops) {
                // The open-door branch (`Player.cs:32857-32864`).
                TileInteraction::DoorClosed {
                    object_drops: drops,
                }
            } else {
                TileInteraction::DoorBlocked {
                    object_drops: drops,
                }
            }
        }
    }
}

/// `PlayerSittingHelper.GetSittingTargetInfo`'s chair case
/// (`PlayerSittingHelper.cs:163-182` + the tail `:317-319`): the seat is the
/// chair's bottom cell centre shifted `direction * 6` px; the facing is -1
/// on the left-facing frame (`frameX == 0`) and 1 otherwise; the clicked
/// bottom cell (`frameY % 40 != 0`) reads the row above.
fn chair_sit_target(world: &World, x: i32, y: i32) -> TileInteraction {
    let t = world.tile(x, y).expect("interact_at checked the tile");
    let mut num = x;
    let mut num2 = y;
    let mut target_direction = -1i32;
    if t.frame_x != 0 {
        target_direction = 1;
    }
    if t.frame_y % 40 != 0 {
        num2 -= 1;
    }
    // `playerSittingPosition = new Point(num, num2 + 1).ToWorldCoordinates(8, 16)
    //  .X += targetDirection * 6` — the bottom-centre of the cell below the
    // chair's seat row.
    TileInteraction::ChairSit {
        seat_x: num as f32 * 16.0 + 8.0 + target_direction as f32 * 6.0,
        seat_y: (num2 + 1) as f32 * 16.0 + 16.0,
        direction: target_direction,
    }
}

/// The dial use-trigger (`Player.cs:32636-32653`): the click is consumed
/// unconditionally (`flag2 = true`, `:32637`); the arm fires only under the
/// compound gate `!fastForwardTimeToX && cooldown == 0` (`:32638`/:32648 —
/// the `netMode == 1` disjunct is the net client's, not ported). On fire,
/// `Main.Sundialing()`/`Moondialing()` (`Main.cs:6169-6196`, SP branch) arm
/// the fast-forward and start the 8-day cooldown; a refused click changes
/// nothing and plays no sound. `sundial` selects the dial (356) over the
/// moondial (663).
fn dial_interaction(world: &mut World, sundial: bool) -> TileInteraction {
    let (already_ff, cooling) = if sundial {
        (
            world.fast_forward_to_dawn,
            world.sundial_cooldown != 0, // `Main.sundialCooldown == 0`, `:32638`
        )
    } else {
        (
            world.fast_forward_to_dusk,
            world.moondial_cooldown != 0, // `Main.moondialCooldown == 0`, `:32648`
        )
    };
    if already_ff || cooling {
        // Refused: the gate at `:32638`/:32648 is false — the click was still
        // consumed, but no dial arms and no sound plays.
        return TileInteraction::DialRefused;
    }
    if sundial {
        world.sundialing(); // Main.Sundialing, Main.cs:6169-6181
    } else {
        world.moondialing(); // Main.Moondialing, Main.cs:6184-6196
    }
    TileInteraction::DialStarted { sundial }
}

/// `WorldGen.OpenDoor` (`WorldGen.cs:37630-37784`): turns the closed door
/// (tile 10) covering `(i, j)` into the open door (tile 11) leaning to
/// `direction` (+1 right, -1 left). No `KillTile` on the door cells — the
/// six open-door cells are written directly; `KillTile` appears only in the
/// validate-cleanup helper and the cut/drip sweep in the opening column.
/// `drops` collects the object destructions of those kills (vanilla's
/// `Item.NewItem` side effects). `false` = refused, door cells untouched.
pub fn open_door(
    world: &mut World,
    i: i32,
    j: i32,
    direction: i32,
    drops: &mut Vec<ObjectDrop>,
) -> bool {
    // The per-cell null-init guards (:37634-37644) are managed-Tile engine
    // noise; the port reads through `World::tile`'s `Option` instead.
    let Some(tile) = world.tile(i, j) else {
        return false;
    };
    if tile.kind != 10 {
        return false; // :37645-37647
    }
    if is_locked_door(tile) {
        return false; // :37649-37652 → IsLockedDoor :69191
    }
    // Frame decode (:37653-37668): `frameY` splits into a 54 px style row
    // (`num4`) and an 18 px in-door row (`num3`); `frameX >= 54` carries
    // horizontal door styles that shift the open door's sheet cell.
    let mut num3 = i32::from(tile.frame_y);
    let mut num4 = 0i32;
    let mut num2 = 0i32;
    while num3 >= 54 {
        num3 -= 54;
        num4 += 1;
    }
    if tile.frame_x >= 54 {
        let num5 = i32::from(tile.frame_x) / 54;
        num4 += 36 * num5;
        num2 += 72 * num5;
    }
    let num = j - num3 / 18; // :37669 — the door's top row
    let mut num6 = i;
    // :37672-37675 — the 1x3 closed door must be intact, or whatever matches
    // is killed and the open refused.
    if !validate_tile_square(world, num6, num, 1, 3, 10, drops) {
        return false;
    }
    // Paint caches (:37676-37690), read before any write — always at column
    // `i` (`num6` is reassigned only afterwards), one per door row, so the
    // closed door's paint carries onto both written columns in either
    // direction.
    let cache: [u8; 3] = [
        world.tile(i, num).map_or(0, |t| t.paint()),
        world.tile(i, num + 1).map_or(0, |t| t.paint()),
        world.tile(i, num + 2).map_or(0, |t| t.paint()),
    ];
    let num7;
    if direction == -1 {
        num6 = i - 1; // :37698-37702
        num2 += 36;
        num7 = i - 1;
    } else {
        num6 = i; // :37703-37706
        num7 = i + 1;
    }
    // The opening column must hold no real blocker (:37702-37710): active
    // tiles that are not cut / 165 / drip refuse the open.
    for k in num..num + 3 {
        if let Some(t) = world.tile(num7, k) {
            if t.active()
                && !crate::tileobject::tile_cut(t.kind)
                && t.kind != 165
                && !is_drip_tile(t.kind)
            {
                return false;
            }
        }
    }
    // Cut / 165 / drip tiles in the way die with a full `KillTile`
    // (:37711-37718; default flags — no suppression exists in the C# body).
    for l in num..num + 3 {
        if let Some(t) = world.tile(num7, l) {
            if t.active()
                && (crate::tileobject::tile_cut(t.kind) || t.kind == 165 || is_drip_tile(t.kind))
            {
                if let Some(kill) = kill_tile(world, num7, l, false) {
                    drops.extend(kill.object_drops);
                }
            }
        }
    }
    // `Wiring.SkipWire` ×6 (:37721-37729) — no wiring in the port.
    let num8 = num4 % 36 * 54; // :37731 — the open door's style row
    // `SoundEngine.PlaySound(8, i*16, j*16)` (:37732) — app-side, via
    // `TileInteraction::DoorOpened`.
    // Fail-neutral world-edge guard (the C# `Main.tile[...]` writes would
    // throw; `CanPlace`'s 5-tile margin makes the case unreachable in play).
    if !world.in_bounds(num6, num) || !world.in_bounds(num6 + 1, num + 2) {
        return false;
    }
    // The 2x3 open door (:37733-37776): `frameX = num2 + dx*18`, `frameY =
    // num8 + dy*18`, `UseBlockColors(cache)` row by row.
    for dy in 0..3i32 {
        for dx in 0..2i32 {
            let t = world.tile_mut(num6 + dx, num + dy).unwrap();
            t.flags |= crate::tile::FLAG_ACTIVE;
            t.kind = 11;
            t.frame_y = (num8 + dy * 18) as i16;
            t.frame_x = (num2 + dx * 18) as i16;
            t.set_paint(cache[dy as usize]);
            touch(world, num6 + dx, num + dy);
        }
    }
    // The trailing `WorldGen.TileFrame` 4x4 loop (:37778-37784): door cells
    // keep their frames (frame-important), neighbours re-derive theirs.
    ensure_frame_important(world, 11);
    framing::reframe_area(world, num6, num, 2);
    true
}

/// `WorldGen.CloseDoor` (`WorldGen.cs:31805-31907`): turns the open door
/// (tile 11) covering `(i, j)` back into the closed door (tile 10) at the
/// hinge column the open frames encode. `forced` skips the `Collision.
/// EmptyTile` occupancy gate (the wired caller, `Wiring.cs:1463`); the port
/// has no wiring so the interaction path always sends `false`. `cell_blocked`
/// is the player/NPC overlap check of `EmptyTile` (`Collision.cs:1359` —
/// with `ignoreTiles: true` the tile check is skipped, entities are not).
pub fn close_door(
    world: &mut World,
    i: i32,
    j: i32,
    forced: bool,
    cell_blocked: &dyn Fn(i32, i32) -> bool,
    drops: &mut Vec<ObjectDrop>,
) -> bool {
    let Some(tile) = world.tile(i, j) else {
        return false;
    };
    let frame_x = i32::from(tile.frame_x); // :31816
    if tile.kind != 11 {
        return false; // :31818-31820
    }
    // Frame decode (:31821-31833). `num5` is written but never read in the
    // decompile — kept out; `num6` is the closed-door style offset the hinge
    // cell receives.
    let mut num4 = i32::from(tile.frame_y);
    while num4 >= 54 {
        num4 -= 54;
    }
    let mut num6 = 0i32;
    if frame_x >= 72 {
        num6 += 54 * (frame_x / 72);
    }
    let num3 = j - num4 / 18; // :31833 — the door's top row
    // `switch (frameX % 72)` (:31834-31851): which column of the open pair is
    // the hinge (`num2`) and which way the door swung (`num`, only -1 is
    // tested). The C# default case leaves `num = 0`, `num2 = i`.
    let mut num2 = i;
    let mut num = 0i32;
    match frame_x % 72 {
        0 => {
            num2 = i;
            num = 1;
        }
        18 => {
            num2 = i - 1;
            num = 1;
        }
        36 => {
            num2 = i + 1;
            num = -1;
        }
        54 => {
            num2 = i;
            num = -1;
        }
        _ => {}
    }
    let mut num7 = num2; // :31852-31854 — the pair's left column
    if num == -1 {
        num7 = num2 - 1;
    }
    // `Collision.EmptyTile(num2, k, ignoreTiles: true)` (:31856-31864): with
    // `ignoreTiles` the tile check is skipped — only player/NPC overlap can
    // refuse the close (:1359-1378). `forced` (the wired caller,
    // `Wiring.cs:1463`) skips the gate.
    if !forced {
        for k in num3..num3 + 3 {
            if cell_blocked(num2, k) {
                return false;
            }
        }
    }
    // :31866-31869 — the 2x3 open door must be intact, or whatever matches is
    // killed and the close refused.
    if !validate_tile_square(world, num7, num3, 2, 3, 11, drops) {
        return false;
    }
    // The write (:31871-31891): the hinge column becomes the closed door —
    // `frameX = genRand.Next(3) * 18 + num6`, `frameY` deliberately NOT
    // rewritten (vanilla quirk) — and the other column deactivates. The port
    // has no runtime `WorldGen.genRand` stream to draw from (and introducing
    // one would touch the determinism invariant), so the frame variant is
    // always 0 — the same value a fresh style-0 placement writes.
    for l in num7..num7 + 2 {
        for m in num3..num3 + 3 {
            let Some(t) = world.tile_mut(l, m) else {
                continue;
            };
            if t.active() && t.kind == 11 {
                if l == num2 {
                    t.kind = 10;
                    t.frame_x = num6 as i16;
                } else {
                    t.flags &= !crate::tile::FLAG_ACTIVE;
                }
                touch(world, l, m);
            }
        }
    }
    // `Wiring.SkipWire` ×3 (:31893-31898) — no wiring in the port.
    // The trailing `WorldGen.TileFrame` 3x4 loop (:31900-31905).
    ensure_frame_important(world, 10);
    framing::reframe_area(world, num2, num3, 2);
    // `SoundEngine.PlaySound(9, i*16, j*16)` (:31906) — app-side, via
    // `TileInteraction::DoorClosed`.
    true
}

/// `WorldGen.ValidateTileSquareIsActiveAndOfType` (`WorldGen.cs:37869-37917`):
/// `true` when every cell of the `width`×`height` square at `(x, y)` is
/// active and of `kind`. Otherwise every matching cell is `KillTile`d (full
/// cascade — drops collected) and `false` returned; an out-of-world square
/// returns `false` without killing (`WorldGen.InWorld`, `:9688`).
fn validate_tile_square(
    world: &mut World,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    kind: u16,
    drops: &mut Vec<ObjectDrop>,
) -> bool {
    // :37874-37890 — an out-of-world square refuses without killing
    // (`WorldGen.InWorld` fluff 0, `:9688`); a mismatched cell marks the
    // square dirty.
    let mut mismatch = false;
    'check: for i in 0..width {
        for jj in 0..height {
            let (cx, cy) = (x + i, y + jj);
            if !world.in_bounds(cx, cy) {
                return false;
            }
            if !world.is_active(cx, cy) || world.tile(cx, cy).is_some_and(|t| t.kind != kind) {
                mismatch = true;
                break 'check;
            }
        }
    }
    if mismatch {
        // :37891-37914 — every active matching cell dies with a full
        // `KillTile` (default flags — the cascade runs, drops are collected).
        for i in 0..width {
            for jj in 0..height {
                let (cx, cy) = (x + i, y + jj);
                if world.is_active(cx, cy) && world.tile(cx, cy).is_some_and(|t| t.kind == kind) {
                    if let Some(kill) = kill_tile(world, cx, cy, false) {
                        drops.extend(kill.object_drops);
                    }
                }
            }
        }
        return false;
    }
    true
}

/// Mirrors `Main.tileFrameImportant[type]` for the door kind a toggle writes
/// (`Main.cs:7637-7638` — a boot-time static both door tiles carry; `place`
/// mirrors the same flag for its own def, `tileobject.rs` DW-1.3, and the
/// toggle switches kinds under the same guarantee).
fn ensure_frame_important(world: &mut World, kind: u16) {
    if world.frame_important.len() <= kind as usize {
        world
            .frame_important
            .resize(crate::tiledata::TILE_COUNT, false);
    }
    world.frame_important[kind as usize] = true;
}

/// `WorldGen.IsLockedDoor` (`WorldGen.cs:69191-69197`): the golden-door
/// frames (tile 10, `frameY` 594-646, `frameX` < 54). The wooden door never
/// matches; the guard is ported because `OpenDoor` calls it.
fn is_locked_door(tile: &Tile) -> bool {
    tile.kind == 10 && (594..=646).contains(&tile.frame_y) && tile.frame_x < 54
}

/// `TileID.Sets.IsADripTile` (`TileID.cs:485`): the vine/drip set `OpenDoor`'s
/// destination sweep kills like cut tiles (`WorldGen.cs:37704-37717`).
fn is_drip_tile(kind: u16) -> bool {
    matches!(kind, 373 | 374 | 375 | 461 | 709)
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
        assert_eq!(kill_tile(&mut w, 10, 20, true).map(|k| k.kind), Some(1));
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
        assert!(kill_tile(&mut w, 10, 20, true).is_none());
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
    fn the_furniture_family_places_on_their_anchors() {
        // The `TileObjectData` verbatim data (see `tileobject.rs`): each kind
        // fills its footprint with the style-0 frames.
        // Piano 87 (3x2, origin (1,1)) at (10, 19): rows 18-19, x 9-11.
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 87, 0, 1));
        for x in 9..=11 {
            for y in 18..=19 {
                let t = w.tile(x, y).unwrap();
                assert!(t.active() && t.kind == 87, "piano {x},{y}");
            }
        }
        // Dresser 88 (3x2), sink 172 (2x2, origin (0,1)), candelabra 100
        // (2x2), bookcase 101 (3x4, origin (1,3)), lamp 93 (1x3, origin
        // (0,2)), clock 104 (2x5, origin (0,4)) — the footprints fill.
        let cases: &[(u16, i32, i32, usize, usize)] = &[
            (88, 10, 19, 3, 2),
            (172, 10, 19, 2, 2),
            (100, 10, 19, 2, 2),
            (101, 10, 19, 3, 4),
            (93, 10, 19, 1, 3),
            (104, 10, 19, 2, 5),
        ];
        for &(kind, tx, ty, w_, h) in cases {
            let mut w = object_world();
            assert!(place_item_tile(&mut w, tx, ty, kind, 0, 1), "place {kind}");
            let def = crate::tileobject::def_for_tile(kind).unwrap();
            let ox = tx - def.origin.x;
            let oy = ty - def.origin.y;
            for x in ox..ox + w_ as i32 {
                for y in oy..oy + h as i32 {
                    let t = w.tile(x, y).unwrap();
                    assert!(
                        t.active() && t.kind == kind,
                        "kind {kind} cell {x},{y}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_bed_places_with_the_facing_alternate() {
        // `Style4x2` (`TileObjectData.cs:2555-2568`): facing left = the base
        // (frames 0/18/36/54), facing right = the `PlaceRight` alternate —
        // the second style column continues at frameX 72 (`Check4x2`'s
        // `frameX >= 72` branch, `WorldGen.cs:47416-47420`).
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 11, 19, 79, 0, -1));
        // The origin (1, 1): the cells span x 10-13, rows 18-19; the frames
        // are the column strides 0/18/36/54.
        assert_eq!(w.tile(10, 18).unwrap().frame_x, 0);
        assert_eq!(w.tile(13, 19).unwrap().frame_x, 54);
        let mut w2 = object_world();
        assert!(place_item_tile(&mut w2, 11, 19, 79, 0, 1));
        // The `PlaceRight` alternate = style 1: the second style column
        // starts at frameX `CoordinateFullWidth` = 72 (`Check4x2`'s
        // `frameX >= 72` branch, `WorldGen.cs:47416-47420`).
        assert_eq!(w2.tile(10, 18).unwrap().frame_x, 72);
        assert_eq!(w2.tile(13, 19).unwrap().frame_x, 72 + 54);
        // The bottom row is 18 tall (`coordinate_heights [16, 18]`).
        assert_eq!(w2.tile(10, 19).unwrap().frame_y, 18);
    }

    #[test]
    fn the_candle_places_on_a_table_only() {
        // `StyleOnTable1x1` (`TileObjectData.cs:2325-2334`): the anchor is
        // `Table` — a bare floor refuses, a placed table accepts.
        let mut w = object_world();
        assert!(!place_item_tile(&mut w, 10, 19, 33, 0, 1));
        assert!(place_item_tile(&mut w, 10, 19, 14, 0, 1)); // the table
        assert!(place_item_tile(&mut w, 11, 17, 33, 0, 1)); // on its top
        assert!(w.tile(11, 17).unwrap().active() && w.tile(11, 17).unwrap().kind == 33);
    }

    #[test]
    fn the_chandelier_hangs_from_the_ceiling() {
        // `TileObjectData.cs:3139-3160`: `AnchorTop = (SolidTile | SolidSide,
        // 1, 1)` + `AnchorBottom Empty` — the doorway world's stone ceiling
        // at y = 16 carries the middle column.
        let mut w = doorway_world();
        assert!(place_item_tile(&mut w, 11, 17, 34, 0, 1));
        for x in 10..=12 {
            for y in 17..=19 {
                let t = w.tile(x, y).unwrap();
                assert!(t.active() && t.kind == 34, "chandelier {x},{y}");
            }
        }
        // The lantern (42, `Style1x2Top`) hangs the same way, 1x2.
        let mut w2 = doorway_world();
        assert!(place_item_tile(&mut w2, 10, 17, 42, 0, 1));
        assert!(w2.tile(10, 17).unwrap().kind == 42 && w2.tile(10, 18).unwrap().kind == 42);
    }

    #[test]
    fn the_candle_toggle_flips_the_lit_frame() {
        // `Player.cs:32378-32390`: `frameX += (frameX > 0) ? -18 : 18`.
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 14, 0, 1));
        assert!(place_item_tile(&mut w, 11, 17, 33, 0, 1));
        assert_eq!(w.tile(11, 17).unwrap().frame_x, 0);
        let blocked = |_: i32, _: i32| false;
        let hit = interact_at(&mut w, 11, 17, 1, &blocked);
        assert!(matches!(hit, TileInteraction::CandleToggled));
        assert_eq!(w.tile(11, 17).unwrap().frame_x, 18);
        let hit = interact_at(&mut w, 11, 17, 1, &blocked);
        assert!(matches!(hit, TileInteraction::CandleToggled));
        assert_eq!(w.tile(11, 17).unwrap().frame_x, 0);
    }

    #[test]
    fn the_chair_sit_decodes_the_seat() {
        // `PlayerSittingHelper.cs:163-182` + `:317-319`: the seat is the
        // chair's bottom cell centre shifted `direction * 6`; the facing is
        // -1 on the left-facing frame (frameX == 0).
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 15, 0, -1)); // the left style
        let blocked = |_: i32, _: i32| false;
        // Clicking the chair's bottom cell (y 19; the top is 18).
        let hit = interact_at(&mut w, 10, 19, 1, &blocked);
        let TileInteraction::ChairSit {
            seat_x,
            seat_y,
            direction,
        } = hit
        else {
            panic!("not a chair sit");
        };
        assert!((seat_x - (10.0 * 16.0 + 8.0 - 6.0)).abs() < 0.01, "{seat_x}");
        assert!((seat_y - (19.0 * 16.0 + 16.0)).abs() < 0.01, "{seat_y}");
        assert_eq!(direction, -1);
        // The right-facing chair flips both.
        let mut w2 = object_world();
        assert!(place_item_tile(&mut w2, 10, 19, 15, 0, 1));
        let hit = interact_at(&mut w2, 10, 19, 1, &blocked);
        let TileInteraction::ChairSit { seat_x, direction, .. } = hit else {
            panic!("not a chair sit");
        };
        assert!((seat_x - (10.0 * 16.0 + 8.0 + 6.0)).abs() < 0.01);
        assert_eq!(direction, 1);
    }

    #[test]
    fn the_bed_decodes_the_spawn_cell_and_the_halves() {
        // `Player.cs:32657-32668`: the left-facing bed at (11, 19) spans
        // x 9-12; the bottom half (`frameX % 72 < 36`) targets the spawn cell
        // `x + 2`, the top half sleeps.
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 11, 19, 79, 0, -1));
        let blocked = |_: i32, _: i32| false;
        // The bottom-left cell (9, 19): frameX 0 → the bottom side; the spawn
        let hit = interact_at(&mut w, 10, 19, 1, &blocked);
        let TileInteraction::BedInteract {
            set_spawn,
            spawn_x,
            spawn_y,
        } = hit
        else {
            panic!("not a bed interact");
        };
        // The bottom half (`frameX % 72 < 36`): the spawn cell decodes to
        // `myX - frameX/18 + 2 = 12`, the floor row `myY - 1 + 2 = 20`.
        assert!(set_spawn);
        assert_eq!(spawn_x, 12);
        assert_eq!(spawn_y, 20);
        // The sleep half: the RIGHT columns (frameX >= 36 — the vanilla keys
        // the halves on frameX only, `IsHoveringOverABottomSideOfABed`).
        let hit = interact_at(&mut w, 12, 18, 1, &blocked);
        let TileInteraction::BedInteract { set_spawn, .. } = hit else {
            panic!("not a bed interact");
        };
        assert!(!set_spawn);
    }

    #[test]
    fn the_clock_reads_the_time() {
        // `Player.cs:32725-32759`: noon (time 27000, day) → 12:00 PM.
        let mut w = object_world();
        w.time = 27000.0;
        w.day_time = true;
        assert!(place_item_tile(&mut w, 10, 19, 104, 0, 1));
        let blocked = |_: i32, _: i32| false;
        let hit = interact_at(&mut w, 10, 19, 1, &blocked);
        let TileInteraction::ClockRead { text } = hit else {
            panic!("not a clock read");
        };
        assert_eq!(text, "12:00 PM");
    }

    #[test]
    fn the_kill_cascade_takes_the_whole_piece() {
        // Breaking the floor under one column destroys the entire object and
        // yields its drop — the `Check*` routines' shared shape.
        for (kind, tx, ty) in [(87u16, 10i32, 19i32), (101, 10, 19), (104, 10, 19), (79, 11, 19)] {
            let mut w = object_world();
            assert!(place_item_tile(&mut w, tx, ty, kind, 0, 1), "place {kind}");
            let def = crate::tileobject::def_for_tile(kind).unwrap();
            let ox = tx - def.origin.x;
            let oy = ty - def.origin.y;
            // `kill_tile` on one of the piece's cells runs its `Check*`
            // routine (`WorldGen.KillTile`'s object dispatch) — the whole
            // piece sweeps.
            kill_tile(&mut w, ox, oy + def.height - 1, true);
            let mut alive = 0;
            for x in ox..ox + def.width {
                for y in oy..oy + def.height {
                    if w
                        .tile(x, y)
                        .is_some_and(|t| t.active() && t.kind == kind)
                    {
                        alive += 1;
                    }
                }
            }
            assert_eq!(alive, 0, "kind {kind} fully destroyed");
        }
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

    // ---- Phase 3 (DW-3.1 / DW-3.3): the KillTile destruction cascade -------

    /// Writes a synthetic object cell with explicit frames (the DW-3.3
    /// sanctioned construction for states no item route produces).
    fn set_object_cell(w: &mut World, x: i32, y: i32, kind: u16, frame_x: i16, frame_y: i16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            frame_x,
            frame_y,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        // `Main.tileFrameImportant[type]` is a static every vanilla world
        // honours — `place` mirrors it into `world.frame_important` (Phase 1
        // DW-1.3) and a loaded save carries it — so synthetic states must set
        // it too, or `reframe_area` mangles the frames the cascade reads.
        if w.frame_important.len() <= kind as usize {
            w.frame_important.resize(crate::tiledata::TILE_COUNT, false);
        }
        w.frame_important[kind as usize] = true;
    }

    /// Asserts every cell of `cells` is inactive (the object is gone).
    fn assert_all_gone(w: &World, cells: &[(i32, i32)]) {
        for &(x, y) in cells {
            assert!(
                !w.tile(x, y).is_some_and(|t| t.active()),
                "cell ({x}, {y}) survived"
            );
        }
    }

    /// The cell whose `Check*` routine runs first for a hit: the object's
    /// first surviving cell in `SquareTileFrame` visit order around the hit —
    /// the anchor the vanilla drop position is derived from. The hit cell
    /// itself is already inactive when the scan reaches it.
    fn expected_trigger(cells: &[(i32, i32)], hit: (i32, i32)) -> (i32, i32) {
        for (dx, dy) in SQUARE_FRAME_ORDER {
            let probe = (hit.0 + dx, hit.1 + dy);
            if probe != hit && cells.contains(&probe) {
                return probe;
            }
        }
        panic!("no surviving object cell in the 3x3 of {hit:?}");
    }

    /// The per-check drop position rule: the bench drops at
    /// (origin x, trigger y) (:47121-47124), the chair at (trigger x,
    /// origin y) (:43494), everything else at the trigger cell.
    fn expected_drop(cells: &[(i32, i32)], hit: (i32, i32), kind: u16) -> (i32, i32) {
        let trigger = expected_trigger(cells, hit);
        match kind {
            18 => (cells.iter().map(|c| c.0).min().unwrap(), trigger.1),
            15 => (trigger.0, cells.iter().map(|c| c.1).min().unwrap()),
            _ => trigger,
        }
    }

    #[test]
    fn test_DW_3_1_bench_cascades_from_every_cell() {
        // `Check2x1` (`WorldGen.cs:46973`): hitting either cell of the 2x1
        // bench destroys the whole bench. The drop is 32x32 at
        // (**origin x**, trigger y) (:47121-47124) — both hits resolve the
        // trigger to the surviving cell, so the drop is (10, 19) either way.
        let cells = [(10i32, 19i32), (11, 19)];
        for hit in cells {
            let mut w = object_world();
            assert!(place_item_tile(&mut w, 10, 19, 18, 0, 1));
            let kill = kill_tile(&mut w, hit.0, hit.1, false).unwrap();
            assert_eq!(kill.kind, 18);
            assert_all_gone(&w, &cells);
            let (dx, dy) = expected_drop(&cells, hit, 18);
            assert_eq!(
                kill.object_drops,
                vec![ObjectDrop {
                    tile_kind: 18,
                    x: dx,
                    y: dy
                }]
            );
        }
    }

    #[test]
    fn test_DW_3_1_table_cascades_from_every_cell() {
        // `Check3x2` (`WorldGen.cs:49187`): any of the 3x2 table's six cells
        // triggers the full destruction (`SolidTileAllowBottomSlope` support
        // per column, :49396-49399). Drop at the check's trigger cell — the
        // first surviving cell in `SquareTileFrame` order (:49424-49426).
        let cells: Vec<(i32, i32)> = (9..=11)
            .flat_map(|x| (18..=19).map(move |y| (x, y)))
            .collect();
        for hit in &cells {
            let mut w = object_world();
            assert!(place_item_tile(&mut w, 10, 19, 14, 0, 1));
            let kill = kill_tile(&mut w, hit.0, hit.1, false).unwrap();
            assert_eq!(kill.kind, 14);
            assert_all_gone(&w, &cells);
            let (dx, dy) = expected_drop(&cells, *hit, 14);
            assert_eq!(
                kill.object_drops,
                vec![ObjectDrop {
                    tile_kind: 14,
                    x: dx,
                    y: dy
                }]
            );
        }
    }

    #[test]
    fn test_DW_3_1_chest_cascades_from_every_cell() {
        // `CheckChest` (`WorldGen.cs:57923`): frame-exact 2x2 integrity plus
        // `Main.tileSolid` below both columns (:57950-57958 — the floor's
        // stone passes). Drop at the check's trigger cell (:57969-57972).
        let cells: Vec<(i32, i32)> = (10..=11)
            .flat_map(|x| (18..=19).map(move |y| (x, y)))
            .collect();
        for hit in &cells {
            let mut w = object_world();
            assert!(place_item_tile(&mut w, 10, 19, 21, 0, 1));
            let kill = kill_tile(&mut w, hit.0, hit.1, false).unwrap();
            assert_eq!(kill.kind, 21);
            assert_all_gone(&w, &cells);
            let (dx, dy) = expected_drop(&cells, *hit, 21);
            assert_eq!(
                kill.object_drops,
                vec![ObjectDrop {
                    tile_kind: 21,
                    x: dx,
                    y: dy
                }]
            );
        }
    }

    #[test]
    fn test_DW_3_1_chair_cascades_with_origin_row_drop() {
        // `Check1x2` (`WorldGen.cs:43335`): the chair drop spawns at
        // (trigger x, **origin y**) (:43494) — the one wooden drop position
        // that is not the trigger cell.
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 15, 0, 1));
        let kill = kill_tile(&mut w, 10, 18, false).unwrap();
        assert_eq!(kill.kind, 15);
        assert_all_gone(&w, &[(10, 18), (10, 19)]);
        assert_eq!(
            kill.object_drops,
            vec![ObjectDrop {
                tile_kind: 15,
                x: 10,
                y: 18
            }]
        );
    }

    #[test]
    fn test_DW_3_1_closed_door_cascades_from_every_cell() {
        // `CheckDoorClosed` (`WorldGen.cs:86432`): `SolidTile` above and
        // below the column (:86475-86479) and all three cells type-10. Drop
        // = `DropDoorItem` at the check's trigger cell (:86508).
        let cells = [(10i32, 17i32), (10, 18), (10, 19)];
        for hit in cells {
            let mut w = doorway_world();
            assert!(place_item_tile(&mut w, 10, 18, 10, 0, 1));
            let kill = kill_tile(&mut w, hit.0, hit.1, false).unwrap();
            assert_eq!(kill.kind, 10);
            assert_all_gone(&w, &cells);
            let (dx, dy) = expected_drop(&cells, hit, 10);
            assert_eq!(
                kill.object_drops,
                vec![ObjectDrop {
                    tile_kind: 10,
                    x: dx,
                    y: dy
                }]
            );
        }
    }

    /// Converts a freshly placed closed door into the open state with the
    /// frames `WorldGen.OpenDoor` writes (:37727-37770): `frameX = num2 +
    /// dx*18` (`num2` 0 for the right leaf, 36 for the left), `frameY =
    /// dy*18` for style row 0.
    fn open_the_door(w: &mut World, left_leaf: bool) -> Vec<(i32, i32)> {
        let (x0, num2) = if left_leaf { (9, 36) } else { (10, 0) };
        let mut cells = Vec::new();
        for dy in 0..3i32 {
            for dx in 0..2i32 {
                set_object_cell(
                    w,
                    x0 + dx,
                    17 + dy,
                    11,
                    num2 + (dx * 18) as i16,
                    (dy * 18) as i16,
                );
                cells.push((x0 + dx, 17 + dy));
            }
        }
        cells
    }

    #[test]
    fn test_DW_3_3_open_door_cascade_right_leaf_from_every_cell() {
        // `CheckDoorOpen` (`WorldGen.cs:86339`): the right-opening leaf
        // (`frameX % 72` 0/18 → hinge = leftmost column) dies from any of
        // its six cells; the drop is at the check's trigger cell (:86415).
        let cells = [
            (10i32, 17i32),
            (11, 17),
            (10, 18),
            (11, 18),
            (10, 19),
            (11, 19),
        ];
        for hit in cells {
            let mut w = doorway_world();
            assert!(place_item_tile(&mut w, 10, 18, 10, 0, 1));
            let open_cells = open_the_door(&mut w, false);
            assert_eq!(open_cells, cells.to_vec());
            let kill = kill_tile(&mut w, hit.0, hit.1, false).unwrap();
            assert_eq!(kill.kind, 11);
            assert_all_gone(&w, &open_cells);
            let (dx, dy) = expected_drop(&cells, hit, 11);
            assert_eq!(
                kill.object_drops,
                vec![ObjectDrop {
                    tile_kind: 11,
                    x: dx,
                    y: dy
                }]
            );
        }
    }

    #[test]
    fn test_DW_3_3_open_door_cascade_left_leaf() {
        // The left-opening leaf (`frameX % 72` 36/54 → hinge = rightmost
        // column, :86363-86368): killing the hinge-column top cell still
        // destroys the whole 2x3 at columns 9..10. The check triggers on
        // (9, 18) — the first surviving cell in scan order.
        let mut w = doorway_world();
        assert!(place_item_tile(&mut w, 10, 18, 10, 0, 1));
        let open_cells = open_the_door(&mut w, true);
        let kill = kill_tile(&mut w, 9, 17, false).unwrap();
        assert_eq!(kill.kind, 11);
        assert_all_gone(&w, &open_cells);
        let (dx, dy) = expected_drop(&open_cells, (9, 17), 11);
        assert_eq!(
            kill.object_drops,
            vec![ObjectDrop {
                tile_kind: 11,
                x: dx,
                y: dy
            }]
        );
    }

    #[test]
    fn test_DW_3_1_support_break_kills_the_object_above() {
        // Mining the ground under a chest column: the cascade reaches the
        // chest through `SquareTileFrame`'s neighbourhood and `CheckChest`'s
        // support test destroys it (:57950-57958).
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 21, 0, 1));
        let kill = kill_tile(&mut w, 10, 20, false).unwrap();
        assert_eq!(kill.kind, 1, "the floor block is the hit cell");
        assert_all_gone(&w, &[(10, 18), (11, 18), (10, 19), (11, 19)]);
        assert_eq!(kill.object_drops.len(), 1);
        assert_eq!(kill.object_drops[0].tile_kind, 21);
    }

    #[test]
    fn test_DW_3_1_adjacent_intact_object_survives() {
        // Two benches side by side: destroying one leaves the other intact
        // and produces exactly one drop (`Check2x1`'s integrity holds for
        // the survivor).
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 18, 0, 1));
        assert!(place_item_tile(&mut w, 12, 19, 18, 0, 1));
        let kill = kill_tile(&mut w, 10, 19, false).unwrap();
        assert_all_gone(&w, &[(10, 19), (11, 19)]);
        for x in [12, 13] {
            assert!(w.tile(x, 19).unwrap().active(), "bench B cell ({x}, 19)");
        }
        assert_eq!(kill.object_drops.len(), 1);
    }

    #[test]
    fn test_DW_3_1_destroy_object_guard_defers_supported_object_pops() {
        // `WorldGen.destroyObject` (:4236) stays true through a check's whole
        // kill loop (`Check3x2` resets it only at :49841), so the chair's
        // re-entered `Check1x2` (via the inner `KillTile` → `SquareTileFrame`
        // → `TileFrame` chain) early-returns. A chair standing on a chest
        // therefore SURVIVES the chest's cascade, floating, until a later
        // break near it runs the check unguarded — both vanilla steps here.
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 21, 0, 1));
        // Chair at (10, 16..17), bottom resting on the chest's top-left cell
        // (10, 18): frames `frameY 0/18`, facing left (`frameX 0`).
        set_object_cell(&mut w, 10, 16, 15, 0, 0);
        set_object_cell(&mut w, 10, 17, 15, 0, 18);
        // A stone block next to the chair: a later break of it reaches the
        // chair's cells through `SquareTileFrame` (a placement next to an
        // intact chair does not pop it — its support still exists).
        assert!(place_tile(&mut w, 11, 17, 1));
        let kill = kill_tile(&mut w, 11, 19, false).unwrap();
        assert_all_gone(&w, &[(10, 18), (11, 18), (10, 19), (11, 19)]);
        assert!(w.tile(10, 16).unwrap().active(), "chair top floats");
        assert!(w.tile(10, 17).unwrap().active(), "chair bottom floats");
        assert_eq!(kill.object_drops.len(), 1, "the chest only");
        assert_eq!(
            (
                kill.object_drops[0].tile_kind,
                kill.object_drops[0].x,
                kill.object_drops[0].y
            ),
            (21, 10, 18)
        );
        // The later break: the stone next to the chair — the chair's check
        // runs unguarded now, finds the support gone, and pops it.
        let kill = kill_tile(&mut w, 11, 17, false).unwrap();
        assert_eq!(kill.kind, 1);
        assert_all_gone(&w, &[(10, 16), (10, 17)]);
        assert_eq!(
            kill.object_drops,
            vec![ObjectDrop {
                tile_kind: 15,
                x: 10,
                y: 16
            }]
        );
    }

    #[test]
    fn test_DW_3_1_no_item_suppresses_drop_records_but_not_the_cascade() {
        // `noItem` (`WorldGen.cs:63546-63549`) gates the drop reporting;
        // the destruction itself always runs.
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 18, 0, 1));
        let kill = kill_tile(&mut w, 10, 19, true).unwrap();
        assert_eq!(kill.kind, 18);
        assert_all_gone(&w, &[(10, 19), (11, 19)]);
        assert!(kill.object_drops.is_empty());
    }

    #[test]
    fn test_DW_3_3_world_edge_skips_the_cascade_like_tile_frame() {
        // `TileFrame`'s guard (:81330): cells within 5 tiles of the edge are
        // never framed, so a synthetic bench straddling the guard line dies
        // only cell-wise when the surviving cell is outside the guard's
        // reach — and cascades normally one tile further in.
        let mut w = object_world();
        set_object_cell(&mut w, 5, 19, 18, 0, 0);
        set_object_cell(&mut w, 6, 19, 18, 18, 0);
        let kill = kill_tile(&mut w, 6, 19, false).unwrap();
        assert_all_gone(&w, &[(6, 19)]);
        assert!(w.tile(5, 19).unwrap().active(), "guard skips x = 5");
        assert!(kill.object_drops.is_empty(), "no check ran");

        let mut w = object_world();
        set_object_cell(&mut w, 6, 19, 18, 0, 0);
        set_object_cell(&mut w, 7, 19, 18, 18, 0);
        let kill = kill_tile(&mut w, 7, 19, false).unwrap();
        assert_all_gone(&w, &[(6, 19), (7, 19)]);
        assert_eq!(kill.object_drops.len(), 1);
    }

    #[test]
    fn test_DW_3_1_lone_object_cell_drops_nothing() {
        // Vanilla quirk kept 1:1: a lone chest cell (the rest of the object
        // already gone — synthetic state) is killed without a drop — no
        // sibling cell survives to run the check, and the per-cell drop
        // table is empty for object kinds (`WorldGen.cs:65748-66091`).
        let mut w = object_world();
        set_object_cell(&mut w, 10, 19, 21, 0, 0);
        let kill = kill_tile(&mut w, 10, 19, false).unwrap();
        assert_eq!(kill.kind, 21);
        assert!(w.tile(10, 19).is_some_and(|t| !t.active()));
        assert!(kill.object_drops.is_empty());
    }

    #[test]
    fn test_object_tiles_at_resolves_origins_through_the_frames() {
        // The anchor lookup generalizes the C# per-type frame math
        // (`WorldGen.cs:57929-57937` etc.): any cell of the object resolves
        // the same top-left origin.
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 14, 19, 21, 0, 1)); // chest origin (14, 18)
        for cell in [(14i32, 18i32), (15, 18), (14, 19), (15, 19)] {
            let tiles = crate::tileobject::object_tiles_at(&w, cell.0, cell.1).unwrap();
            assert_eq!(tiles.origin.x, 14, "chest cell {cell:?}");
            assert_eq!(tiles.origin.y, 18, "chest cell {cell:?}");
            assert_eq!(tiles.def.kind, 21);
            let mut got: Vec<(i32, i32)> = tiles.cells().collect();
            got.sort_unstable();
            assert_eq!(
                got,
                vec![(14, 18), (14, 19), (15, 18), (15, 19)],
                "cells() of the 2x2 chest"
            );
        }
        // Inactive and unregistered cells resolve to nothing.
        assert!(crate::tileobject::object_tiles_at(&w, 20, 19).is_none());
        assert!(crate::tileobject::object_tiles_at(&w, 200, 200).is_none());
        // The open door's left leaf (frameX 54): dx = 54/18 % 2 = 1 → origin
        // one column left — matches the C# `frameX % 72` case 54 mapping
        // (:86366-86368).
        let mut w2 = doorway_world();
        assert!(place_item_tile(&mut w2, 10, 18, 10, 0, 1));
        open_the_door(&mut w2, true);
        let tiles = crate::tileobject::object_tiles_at(&w2, 10, 18).unwrap();
        assert_eq!(tiles.def.kind, 11);
        assert_eq!((tiles.origin.x, tiles.origin.y), (9, 17));
    }

    #[test]
    fn legacy_tiles_keep_their_single_cell_break() {
        // A non-registry kind breaks as before: one cell, the legacy drop
        // position (the hit cell), no cascade records.
        let mut w = object_world();
        let kill = kill_tile(&mut w, 10, 20, false).unwrap();
        assert_eq!(kill.kind, 1);
        assert!(!w.tile(10, 20).unwrap().active());
        assert!(
            w.tile(9, 20).unwrap().active(),
            "neighbouring floor untouched"
        );
        assert!(kill.object_drops.is_empty());
        // Inactive and out-of-world cells return None (:63519-63532).
        assert!(kill_tile(&mut w, 10, 20, false).is_none());
        assert!(kill_tile(&mut w, -1, 20, false).is_none());
        assert!(kill_tile(&mut w, 10, 200, false).is_none());
    }

    #[test]
    fn platforms_count_as_support_and_their_death_still_pops_the_sitter() {
        // `CheckChest`'s support test passes on platforms
        // (`Main.tileSolid[19] = true`, `Main.cs:7893`, :57953-57958) and
        // `Check3x2`'s `SolidTileAllowBottomSlope` includes solidTop
        // (:70167-70185, :49396-49399). Both objects survive a neighbour kill
        // on the platform row; breaking the platform under a column still
        // destroys the sitter (the support-inactive path).
        let mut w = object_world();
        for x in 9..=17 {
            set_object_cell(&mut w, x, 19, 19, 0, 0); // a platform row
        }
        // Chest (10..11, 17..18) and table (14..16, 17..18) with canonical
        // style-0 frames, standing on the platform.
        for (x, y, kind, fx_off) in [(10, 17, 21, 0), (14, 17, 14, 0)] {
            for dy in 0..2i32 {
                for dx in 0..kind_widths(kind) {
                    set_object_cell(
                        &mut w,
                        x + dx,
                        y + dy,
                        kind,
                        (fx_off + dx * 18) as i16,
                        (dy * 18) as i16,
                    );
                }
            }
        }
        // A neighbour kill on the platform row leaves both intact.
        let kill = kill_tile(&mut w, 12, 19, false).unwrap();
        assert_eq!(kill.kind, 19);
        assert!(kill.object_drops.is_empty(), "both sitters supported");
        for x in 10..=11 {
            assert!(w.tile(x, 17).unwrap().active(), "chest {x}");
        }
        for x in 14..=16 {
            assert!(w.tile(x, 17).unwrap().active(), "table {x}");
        }
        // Killing the platform under the chest's left column pops the chest;
        // the table keeps its platform.
        let kill = kill_tile(&mut w, 10, 19, false).unwrap();
        assert_all_gone(&w, &[(10, 17), (11, 17), (10, 18), (11, 18)]);
        assert_eq!(kill.object_drops.len(), 1);
        assert_eq!(kill.object_drops[0].tile_kind, 21);
        for x in 14..=16 {
            assert!(w.tile(x, 17).unwrap().active(), "table {x} survives");
        }
        // And the platform's death under a table column pops the table.
        let kill = kill_tile(&mut w, 15, 19, false).unwrap();
        assert_all_gone(
            &w,
            &[(14, 17), (15, 17), (16, 17), (14, 18), (15, 18), (16, 18)],
        );
        assert_eq!(kill.object_drops.len(), 1);
        assert_eq!(kill.object_drops[0].tile_kind, 14);
    }

    /// The wooden def widths for the synthetic frames above.
    fn kind_widths(kind: u16) -> i32 {
        match kind {
            21 => 2,
            14 => 3,
            _ => 1,
        }
    }

    /// A fresh closed door at column 10 (rows 17-19) through the placement
    /// route — the frames `open_door` decodes: `frameX` 0, `frameY`
    /// 0/18/36.
    fn closed_door_world() -> World {
        let mut w = doorway_world();
        assert!(place_item_tile(&mut w, 10, 18, 10, 0, 1));
        w
    }

    fn cells_of(w: &World, cells: &[(i32, i32)]) -> Vec<(bool, u16, i16, i16)> {
        cells
            .iter()
            .map(|&(x, y)| {
                let t = w.tile(x, y).unwrap();
                (t.active(), t.kind, t.frame_x, t.frame_y)
            })
            .collect()
    }

    fn no_blocker(_: i32, _: i32) -> bool {
        false
    }

    #[test]
    fn test_DW_4_2_open_door_right_leaf_frames() {
        // `WorldGen.OpenDoor` direction +1 (`WorldGen.cs:37733-37776`): the
        // hinge column stays at i with `frameX = num2` (0) and the leaf opens
        // into i+1 with `frameX = num2 + 18`; `frameY` rows 0/18/36.
        let mut w = closed_door_world();
        w.tile_mut(10, 17).unwrap().set_paint(3);
        w.tile_mut(10, 18).unwrap().set_paint(5);
        let mut drops = Vec::new();
        assert!(open_door(&mut w, 10, 18, 1, &mut drops));
        assert!(drops.is_empty());
        let row_paint = [3u8, 5, 0]; // cache rows (:37676-37690)
        for (k, y, fy) in [(0usize, 17i32, 0i16), (1, 18, 18), (2, 19, 36)] {
            let hinge = w.tile(10, y).unwrap();
            assert!(hinge.active() && hinge.kind == 11, "hinge row {y}");
            assert_eq!((hinge.frame_x, hinge.frame_y), (0, fy), "hinge row {y}");
            // `UseBlockColors(cache)`: each written row takes its own closed
            // door row's paint.
            assert_eq!(hinge.paint(), row_paint[k], "hinge paint row {y}");
            let leaf = w.tile(11, y).unwrap();
            assert!(leaf.active() && leaf.kind == 11, "leaf row {y}");
            assert_eq!((leaf.frame_x, leaf.frame_y), (18, fy), "leaf row {y}");
            assert_eq!(leaf.paint(), row_paint[k], "leaf paint row {y}");
        }
        // Nothing else changed: the closed-door cells were overwritten, not
        // killed (no KillTile on the door body in 1.4.5.8).
        assert!(!w.tile(12, 18).unwrap().active());
    }

    #[test]
    fn test_DW_4_2_open_door_left_leaf_frames() {
        // Direction -1 (:37698-37702): the door leans left — hinge column
        // i-1 gets `frameX = 36`, column i `frameX = 54`.
        let mut w = closed_door_world();
        w.tile_mut(10, 17).unwrap().set_paint(3);
        let mut drops = Vec::new();
        assert!(open_door(&mut w, 10, 18, -1, &mut drops));
        for (y, fy) in [(17i32, 0i16), (18, 18), (19, 36)] {
            let hinge = w.tile(9, y).unwrap();
            assert!(hinge.active() && hinge.kind == 11, "hinge row {y}");
            assert_eq!((hinge.frame_x, hinge.frame_y), (36, fy), "hinge row {y}");
            let leaf = w.tile(10, y).unwrap();
            assert!(leaf.active() && leaf.kind == 11, "leaf row {y}");
            assert_eq!((leaf.frame_x, leaf.frame_y), (54, fy), "leaf row {y}");
        }
        // The paint caches are read at column `i` before `num6` is
        // reassigned (:37676-37690), so direction -1 carries the closed
        // door's paint onto the new hinge column too.
        assert_eq!(w.tile(10, 17).unwrap().paint(), 3);
        assert_eq!(w.tile(9, 17).unwrap().paint(), 3);
    }

    #[test]
    fn test_DW_4_2_close_door_hinge_from_both_leaves() {
        // Right leaf (`frameX % 72 == 0`/`18`, `:31834-31851`): the hinge is
        // the pair's left column and becomes the closed door; the other
        // column deactivates. `frameY` is deliberately not rewritten — the
        // style-0 open rows equal the closed rows, so the frames come out as
        // a fresh placement's.
        for click in [(10i32, 18i32), (11, 18)] {
            let mut w = doorway_world();
            let cells = open_the_door(&mut w, false);
            let mut drops = Vec::new();
            assert!(close_door(
                &mut w,
                click.0,
                click.1,
                false,
                &no_blocker,
                &mut drops
            ));
            assert!(drops.is_empty());
            let written: Vec<_> = (0..3)
                .flat_map(|dy| [(10, 17 + dy), (11, 17 + dy)])
                .collect();
            let got = cells_of(&w, &written);
            for (k, (active, kind, fx, fy)) in got.iter().enumerate() {
                if k % 2 == 0 {
                    assert!(*active && *kind == 10, "hinge cell {k}");
                    assert_eq!(*fx, 0, "hinge cell {k}"); // genRand variant 0 (see the port note)
                    // `frameY` is not rewritten (:31880-31886): the style-0
                    // open rows 0/18/36 stay.
                    assert_eq!(*fy, (k / 2) as i16 * 18, "hinge cell {k}");
                } else {
                    assert!(!*active, "leaf cell {k}");
                }
            }
            assert_eq!(cells.len(), 6); // the state the close consumed
        }
        // Left leaf (`frameX % 72 == 36`/`54`): hinge is the pair's RIGHT
        // column (`num2 = i` for 54, `i + 1` for 36).
        let mut w = doorway_world();
        open_the_door(&mut w, true);
        let mut drops = Vec::new();
        assert!(close_door(&mut w, 10, 18, false, &no_blocker, &mut drops));
        for dy in 0..3i32 {
            let hinge = w.tile(10, 17 + dy).unwrap();
            assert!(hinge.active() && hinge.kind == 10, "left hinge row {dy}");
            assert_eq!(hinge.frame_x, 0);
            assert!(!w.tile(9, 17 + dy).unwrap().active(), "left leaf row {dy}");
        }
    }

    #[test]
    fn test_DW_4_2_door_toggle_round_trip() {
        // Closed → open → close → open, all through the dispatcher — the
        // repeated-click (toggle) edge case; frames identical each open.
        let mut w = closed_door_world();
        let blocked = no_blocker as fn(i32, i32) -> bool;
        let expected = [(0i16, 0i16), (18, 0), (0, 18), (18, 18), (0, 36), (18, 36)];
        for round in 0..2 {
            match interact_at(&mut w, 10, 18, 1, &blocked) {
                TileInteraction::DoorOpened { object_drops } => {
                    assert!(object_drops.is_empty());
                }
                other => panic!("round {round}: expected DoorOpened, got {other:?}"),
            }
            let mut got = Vec::new();
            for dy in 0..3i32 {
                for dx in 0..2i32 {
                    let t = w.tile(10 + dx, 17 + dy).unwrap();
                    got.push((t.frame_x, t.frame_y));
                }
            }
            assert_eq!(got, expected, "round {round}");
            if round == 0 {
                match interact_at(&mut w, 11, 18, 1, &blocked) {
                    TileInteraction::DoorClosed { object_drops } => {
                        assert!(object_drops.is_empty());
                    }
                    other => panic!("expected DoorClosed, got {other:?}"),
                }
                assert!(w.tile(10, 18).unwrap().active() && w.tile(10, 18).unwrap().kind == 10);
            }
        }
    }

    #[test]
    fn test_DW_4_2_open_door_blocked_by_solid() {
        // A real blocker in the opening column refuses the open and leaves
        // the door untouched (:37702-37710).
        let mut w = closed_door_world();
        for y in 17..=19 {
            set_active(&mut w, 11, y, 1);
        }
        let mut drops = Vec::new();
        assert!(!open_door(&mut w, 10, 18, 1, &mut drops));
        assert!(drops.is_empty());
        for y in 17..=19 {
            let t = w.tile(10, y).unwrap();
            assert!(t.active() && t.kind == 10, "door row {y}");
            assert_eq!(
                (t.frame_x, t.frame_y),
                (0, (y - 17) as i16 * 18),
                "door row {y}"
            );
        }
        // The opposite direction's column is free — the open succeeds there
        // (the dispatcher-level retry is DW-4.1's test).
        assert!(open_door(&mut w, 10, 18, -1, &mut drops));
    }

    #[test]
    fn test_DW_4_2_open_door_kills_cut_tiles_in_the_way() {
        // Cut tiles in the opening column die with a full `KillTile`
        // (:37711-37718) and the open proceeds.
        let mut w = closed_door_world();
        set_active(&mut w, 11, 18, 3); // kind 3 ∈ Main.tileCut
        let mut drops = Vec::new();
        assert!(open_door(&mut w, 10, 18, 1, &mut drops));
        assert!(drops.is_empty()); // a cut plant carries no object drop
        let t = w.tile(11, 18).unwrap();
        assert!(t.active() && t.kind == 11);
    }

    #[test]
    fn test_DW_4_2_close_door_blocked_by_entity() {
        // `Collision.EmptyTile(ignoreTiles: true)` (`Collision.cs:1359`):
        // entities refuse the close, `forced` (the wired caller's flag)
        // ignores them.
        let mut w = doorway_world();
        open_the_door(&mut w, false);
        let blocked = |x: i32, y: i32| x == 10 && y == 18;
        let mut drops = Vec::new();
        assert!(!close_door(&mut w, 10, 18, false, &blocked, &mut drops));
        assert!(w.tile(10, 18).unwrap().kind == 11, "door stays open");
        assert!(close_door(&mut w, 10, 18, true, &blocked, &mut drops));
        assert!(w.tile(10, 18).unwrap().kind == 10, "forced closes");
    }

    #[test]
    fn test_DW_4_2_wrong_kind_returns_false() {
        // Cross-kind calls refuse without touching the world (:37645/:31818).
        let mut w = closed_door_world();
        let mut drops = Vec::new();
        assert!(!open_door(&mut w, 40, 40, 1, &mut drops)); // out of bounds
        assert!(!close_door(&mut w, 10, 18, false, &no_blocker, &mut drops));
        let mut w2 = doorway_world();
        open_the_door(&mut w2, false);
        assert!(!open_door(&mut w2, 10, 18, 1, &mut drops));
    }

    #[test]
    fn test_DW_4_2_style_offset_frames() {
        // A style-1 closed door (`frameY` 54/72/90, the C# `num4`/`num8`
        // decode, :37657-37731): the open door lands in sheet row 54.
        let mut w = doorway_world();
        for (dy, fy) in [(0i32, 54i16), (1, 72), (2, 90)] {
            set_object_cell(&mut w, 10, 17 + dy, 10, 0, fy);
        }
        let mut drops = Vec::new();
        assert!(open_door(&mut w, 10, 17, 1, &mut drops));
        for dy in 0..3i32 {
            let t = w.tile(10, 17 + dy).unwrap();
            assert_eq!(t.frame_y, 54 + dy as i16 * 18, "style row {dy}");
            assert_eq!(t.frame_x, 0);
            let leaf = w.tile(11, 17 + dy).unwrap();
            assert_eq!(leaf.frame_x, 18);
        }
    }

    #[test]
    fn test_DW_4_2_partial_door_validate_cleanup() {
        // `ValidateTileSquareIsActiveAndOfType` (:37869-37917): a door with a
        // broken cell has its remaining door cells killed (full `KillTile` —
        // the cascade records the object destruction) and the open refused.
        let mut w = closed_door_world();
        let broken = w.index(10, 19);
        w.tiles[broken] = crate::tile::Tile::default();
        let mut drops = Vec::new();
        assert!(!open_door(&mut w, 10, 18, 1, &mut drops));
        for y in 17..=19 {
            assert!(!w.tile(10, y).unwrap().active(), "row {y} cleaned up");
        }
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].tile_kind, 10);
        // The drop record sits at the `Check*` visit cell the cascade ran
        // from (Phase 3's `square_tile_frame` order).
        assert_eq!((drops[0].x, drops[0].y), (10, 18));
    }

    #[test]
    fn test_DW_4_1_interact_at_ignores_non_interaction_tiles() {
        // The fall-through contract: a stone tile, an air cell and a chest
        // (registry kind with `Interaction::None`) all return `None`.
        let mut w = closed_door_world();
        assert!(place_item_tile(&mut w, 14, 19, 21, 0, 1));
        let blocked = no_blocker as fn(i32, i32) -> bool;
        for (x, y) in [(10i32, 20i32), (12, 18), (14, 19)] {
            assert!(matches!(
                interact_at(&mut w, x, y, 1, &blocked),
                TileInteraction::None
            ));
        }
    }

    #[test]
    fn test_DW_4_1_interact_at_dispatches_door_kinds() {
        let mut w = closed_door_world();
        let blocked = no_blocker as fn(i32, i32) -> bool;
        assert!(matches!(
            interact_at(&mut w, 10, 18, 1, &blocked),
            TileInteraction::DoorOpened { .. }
        ));
        assert!(w.tile(10, 17).unwrap().kind == 11, "opened");
        assert!(matches!(
            interact_at(&mut w, 10, 18, 1, &blocked),
            TileInteraction::DoorClosed { .. }
        ));
        assert!(w.tile(10, 17).unwrap().kind == 10, "closed");
    }

    #[test]
    fn test_DW_4_1_interact_at_retries_opposite_direction() {
        // `Player.cs:32841-32852`: open with the facing; while the tile is
        // still a closed door, retry with the opposite direction.
        let mut w = closed_door_world();
        for y in 17..=19 {
            set_active(&mut w, 11, y, 1);
        }
        let blocked = no_blocker as fn(i32, i32) -> bool;
        match interact_at(&mut w, 10, 18, 1, &blocked) {
            TileInteraction::DoorOpened { object_drops } => assert!(object_drops.is_empty()),
            other => panic!("expected DoorOpened via retry, got {other:?}"),
        }
        for dy in 0..3i32 {
            let hinge = w.tile(9, 17 + dy).unwrap();
            assert!(hinge.active() && hinge.kind == 11);
            assert_eq!(hinge.frame_x, 36);
        }
    }

    #[test]
    fn test_DW_4_1_interact_at_blocked_door_consumes_without_drops() {
        // Both directions blocked: the click is still consumed (vanilla sets
        // `flag2` before attempting, `Player.cs:32836`) — `DoorBlocked` with
        // no drops and no world change.
        let mut w = closed_door_world();
        for y in 17..=19 {
            set_active(&mut w, 9, y, 1);
            set_active(&mut w, 11, y, 1);
        }
        let blocked = no_blocker as fn(i32, i32) -> bool;
        match interact_at(&mut w, 10, 18, 1, &blocked) {
            TileInteraction::DoorBlocked { object_drops } => assert!(object_drops.is_empty()),
            other => panic!("expected DoorBlocked, got {other:?}"),
        }
        assert!(w.tile(10, 17).unwrap().kind == 10, "door intact");
    }

    #[test]
    fn test_DW_4_3_open_door_survives_missing_wall() {
        // Vanilla's `OpenDoor` has no wall/support check — a door whose wall
        // was mined away still opens (the closed-door square check is
        // type-only, `WorldGen.cs:37672-37675`).
        let mut w = closed_door_world();
        for y in 17..=19 {
            let t = w.tile_mut(10, y).unwrap();
            t.wall = 0; // no wall behind the door anywhere
        }
        let mut drops = Vec::new();
        assert!(open_door(&mut w, 10, 18, 1, &mut drops));
        assert!(w.tile(10, 17).unwrap().kind == 11);
    }

    #[test]
    fn test_DW_4_3_broken_door_interaction_cleans_up_and_blocks() {
        // Right-clicking a partially-broken door through the dispatcher: the
        // validate cleanup kills the rest (drop recorded) and the click is
        // consumed without a cue (`DoorBlocked`).
        let mut w = closed_door_world();
        let broken = w.index(10, 19);
        w.tiles[broken] = crate::tile::Tile::default();
        let blocked = no_blocker as fn(i32, i32) -> bool;
        match interact_at(&mut w, 10, 18, 1, &blocked) {
            TileInteraction::DoorBlocked { object_drops } => {
                assert_eq!(object_drops.len(), 1);
                assert_eq!(object_drops[0].tile_kind, 10);
            }
            other => panic!("expected DoorBlocked, got {other:?}"),
        }
    }

    // ---- Phase 2 (M1.4-BG-21): the dial triggers (356/663) -----------------

    /// Plan-assumption check: the dials have no registry def, so
    /// `place_item_tile` takes the legacy single-tile route and the placed
    /// tile is a plain active 356/663 — exactly the shape `interact_at`'s
    /// dial arm reads (`Player.cs:32636`/`:32646`).
    #[test]
    fn dial_tiles_place_through_the_legacy_route() {
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 356, 0, 1));
        let sundial = *w.tile(10, 19).unwrap();
        assert!(sundial.active() && sundial.kind == 356);
        assert!(place_item_tile(&mut w, 14, 19, 663, 0, 1));
        let moondial = *w.tile(14, 19).unwrap();
        assert!(moondial.active() && moondial.kind == 663);
    }

    #[test]
    fn dw_2_1_lmb_sundial_arms_fast_forward_with_cooldown() {
        // `Player.cs:32636-32645`: the click on tile 356 fires
        // `Main.Sundialing()` (SP branch, `Main.cs:6178-6180`) — the
        // FF-to-dawn flag arms and the 8-day cooldown starts.
        let mut w = object_world();
        set_active(&mut w, 10, 19, 356);
        let blocked = no_blocker as fn(i32, i32) -> bool;
        match interact_at(&mut w, 10, 19, 1, &blocked) {
            TileInteraction::DialStarted { sundial } => assert!(sundial),
            other => panic!("expected DialStarted, got {other:?}"),
        }
        assert!(w.fast_forward_to_dawn);
        assert!(!w.fast_forward_to_dusk);
        assert_eq!(w.sundial_cooldown, 8);
        assert_eq!(w.moondial_cooldown, 0);
    }

    #[test]
    fn dw_2_2_retrigger_while_dialing_is_consumed_without_effect() {
        // `Player.cs:32637-32638`: the second click while
        // `fastForwardTimeToDawn` runs is still consumed (`flag2`) but the
        // gate refuses — no state change, no cue.
        let mut w = object_world();
        set_active(&mut w, 10, 19, 356);
        let blocked = no_blocker as fn(i32, i32) -> bool;
        assert!(matches!(
            interact_at(&mut w, 10, 19, 1, &blocked),
            TileInteraction::DialStarted { .. }
        ));
        match interact_at(&mut w, 10, 19, 1, &blocked) {
            TileInteraction::DialRefused => {}
            other => panic!("expected DialRefused, got {other:?}"),
        }
        assert!(w.fast_forward_to_dawn, "the first dial still runs");
        assert_eq!(w.sundial_cooldown, 8, "the cooldown did not restart");
    }

    #[test]
    fn dw_2_3_cooldown_blocks_the_dial_without_cue() {
        // `Player.cs:32638`: `Main.sundialCooldown == 0` is the gate's other
        // conjunct — a running cooldown refuses the dial (consumed silently,
        // no cue since `SoundID.Item4` sits behind the gate).
        let mut w = object_world();
        set_active(&mut w, 10, 19, 356);
        w.sundial_cooldown = 3;
        let blocked = no_blocker as fn(i32, i32) -> bool;
        match interact_at(&mut w, 10, 19, 1, &blocked) {
            TileInteraction::DialRefused => {}
            other => panic!("expected DialRefused, got {other:?}"),
        }
        assert!(!w.fast_forward_to_dawn);
        assert_eq!(w.sundial_cooldown, 3, "the cooldown did not reset");
    }

    #[test]
    fn dw_2_4_moondial_mirrors_sundial() {
        // `Player.cs:32646-32653` + `Main.Moondialing` (`Main.cs:6184-6196`):
        // tile 663 arms the FF-to-dusk and its own 8-day cooldown.
        let mut w = object_world();
        set_active(&mut w, 10, 19, 663);
        let blocked = no_blocker as fn(i32, i32) -> bool;
        match interact_at(&mut w, 10, 19, 1, &blocked) {
            TileInteraction::DialStarted { sundial } => assert!(!sundial),
            other => panic!("expected DialStarted, got {other:?}"),
        }
        assert!(w.fast_forward_to_dusk);
        assert!(!w.fast_forward_to_dawn);
        assert_eq!(w.moondial_cooldown, 8);
        assert_eq!(w.sundial_cooldown, 0);

        // The refused mirror: a re-click mid-dial is consumed, changes
        // nothing (`:32648`).
        match interact_at(&mut w, 10, 19, 1, &blocked) {
            TileInteraction::DialRefused => {}
            other => panic!("expected DialRefused, got {other:?}"),
        }
        assert!(w.fast_forward_to_dusk);
        assert_eq!(w.moondial_cooldown, 8);
    }

    #[test]
    fn dw_2_6_adjacent_non_dial_tile_has_no_interaction() {
        // Dirty check: the dispatcher routes on the clicked cell's own kind —
        // a stone tile beside the dial, the floor under it and plain air all
        // fall through as `None` with no dial state touched.
        let mut w = object_world();
        set_active(&mut w, 10, 19, 356);
        set_active(&mut w, 11, 19, 1); // stone next to the sundial
        let blocked = no_blocker as fn(i32, i32) -> bool;
        for (x, y) in [(11i32, 19i32), (10, 20), (12, 19)] {
            assert!(
                matches!(
                    interact_at(&mut w, x, y, 1, &blocked),
                    TileInteraction::None
                ),
                "({x}, {y}) has no interaction"
            );
        }
        assert!(!w.fast_forward_to_dawn && !w.fast_forward_to_dusk);
        assert_eq!(w.sundial_cooldown, 0);
        assert_eq!(w.moondial_cooldown, 0);
    }

    #[test]
    fn inactive_dial_tile_has_no_interaction() {
        // `TileInteractionsCheck` gates on active tiles only
        // (`Player.cs:32031-32041`): an inactive dial cell is invisible to
        // the use path.
        let mut w = object_world();
        let i = w.index(10, 19);
        w.tiles[i] = Tile {
            kind: 356,
            ..Default::default()
        }; // no FLAG_ACTIVE
        let blocked = no_blocker as fn(i32, i32) -> bool;
        assert!(matches!(
            interact_at(&mut w, 10, 19, 1, &blocked),
            TileInteraction::None
        ));
        assert!(!w.fast_forward_to_dawn);
    }
}
