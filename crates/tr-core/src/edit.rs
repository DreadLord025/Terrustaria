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
        14 => check_table(world, x, y),
        15 => check_chair(world, x, y),
        18 => check_bench(world, x, y),
        21 => check_chest(world, x, y),
        _ => None,
    }
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
        cells: killable_cells(world, &region, 10),
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
        cells: killable_cells(world, &region, 11),
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
                .is_none_or(|c| !c.active() || c.kind != 14 || (c.frame_x, c.frame_y) != expected)
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
    let _ = style; // `GetItemDrop_Tables(style)` — the slice covers style 0 (→ 32)
    Some(CheckPlan {
        drop: ObjectDrop {
            tile_kind: 14,
            x,
            y,
        },
        cells: killable_cells(world, &region, 14),
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
        cells: killable_cells(world, &region, 15),
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
        cells: killable_cells(world, &region, 18),
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
        cells: killable_cells(world, &region, 21),
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
            w.frame_important
                .resize(crate::tiledata::TILE_COUNT, false);
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
        assert_all_gone(&w, &[(14, 17), (15, 17), (16, 17), (14, 18), (15, 18), (16, 18)]);
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
}
