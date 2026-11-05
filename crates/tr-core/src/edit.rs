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
/// object's tile id; `item` carries the id only when the `Check*` body spawns
/// the item itself — the sunflower's `Item.NewItem(…, 63)`
/// (`WorldGen.cs:53787-53791`); `None` resolves app-side through
/// `registry::tile_drop` (style-0 drops: door 25, table 32, chair 34, bench
/// 36, chest 48 — `WorldGen.cs:69204/:42194/:42635/:42447/:41677`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ObjectDrop {
    pub tile_kind: u16,
    pub x: i32,
    pub y: i32,
    /// The item the check spawned with the destruction, if any (see above).
    pub item: Option<i32>,
}

/// One broken loot pot — `CheckPot`'s tail (`WorldGen.cs:57011-57014`): after
/// the 2x2 dies, vanilla runs `SpawnThingsFromPot(i, j, num, num2, num4)` at
/// the check's trigger cell with the pot's style row (`frameY / 36`). The app
/// resolves the loot rolls (phase 3) and plays the break sound
/// ([`pot_break_sound`]) from `style`; gated by `netMode != 1` only, so the
/// event ignores `kill_tile`'s `no_item`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct PotEvent {
    pub style: i32,
    pub x: i32,
    pub y: i32,
}

/// One cascade-killed herb cell — `CheckAlch`'s kill loop (`WorldGen.cs:46088-46091`)
/// whose `KillTile` runs the per-cell `KillTile_DropItems` herb rolls
/// (`KillTile_GetItemDrops`, `WorldGen.cs:65674-65698`: the herb `313 + frameX / 18`
/// plus the seed stack behind `IsHarvestableHerbWithSeed`). The app resolves the
/// RNG rolls (phase 3); `noItem`-gated like the records (the C# drop is a
/// `KillTile_DropItems` drop).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HerbEvent {
    pub kind: u16,
    /// `frameX / 18` — the herb style (0 Daybloom … 6 Shiverthorn).
    pub style: i32,
    pub x: i32,
    pub y: i32,
}

/// One broken 2x1 pile's loot block — `Check2x1`'s `Item.NewItem` table for
/// tile 185 (`WorldGen.cs:47130-47245`), keyed on the origin cell's `frameX`
/// (the right cell's when the origin already died). The app resolves the
/// `genRand` coin/ore counts (phase 3); a `Check*`-level `Item.NewItem`, so it
/// is not `noItem`-gated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct PileLootEvent {
    pub frame_x: i32,
    pub x: i32,
    pub y: i32,
}

/// The result of `kill_tile`: the kind of the hit cell plus every object
/// destruction the cascade performed, in `SquareTileFrame` visit order, every
/// loot pot the cascade broke, every cascade-killed herb cell and every broken
/// 2x1 pile's loot block.
#[derive(Clone, Debug, Default)]
pub struct TileKill {
    /// Tile kind of the killed (hit) cell.
    pub kind: u16,
    pub object_drops: Vec<ObjectDrop>,
    pub events: Vec<PotEvent>,
    pub herb_events: Vec<HerbEvent>,
    pub pile_events: Vec<PileLootEvent>,
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
        ..Default::default()
    };
    square_tile_frame(world, x, y, false, &mut destroy_object, &mut kill);
    after_edit(world, x, y);
    kill.object_drops
}

/// The body of `KillTile` plus the shared `WorldGen.destroyObject` flag
/// (:4236). The guarded checks re-entered through the cascade's own `KillTile`
/// calls early-return while the flag is set; the unguarded `CheckAlch`,
/// `CheckOnTable1x1` and `CheckPile`-1x1 run and kill straight through it —
/// exactly like the C# static.
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
        ..Default::default()
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
        // :43338/:46976/:49190/:53732/:56800/:57926/:86341/:86434): only the
        // multi-tile object checks carry the entry guard. `CheckAlch`
        // (:45972-46092), `CheckOnTable1x1` (:43503-43546) and `CheckPile`'s
        // 1x1 body (:46875-46971) have none — they run and kill while another
        // object's kill loop is in flight.
        let guarded = destroy_object_guarded(cell);
        if *destroy_object && guarded {
            continue;
        }
        let Some(plan) = check_object(world, cx, cy) else {
            continue;
        };
        if !no_item {
            if let Some(drop) = plan.drop {
                kill.object_drops.push(drop);
            }
            // The herb event stands for the killed cell's `KillTile_DropItems`
            // roll (`WorldGen.cs:63860-63864`), so it carries the same
            // `noItem` gate; the pile loot block is a `Check2x1`-level
            // `Item.NewItem` and is not gated (:47130-47245).
            if let Some(herb) = plan.herb_event {
                kill.herb_events.push(herb);
            }
        }
        if let Some(pile) = plan.pile_event {
            kill.pile_events.push(pile);
        }
        // `SpawnThingsFromPot`'s only gate is `Main.netMode != 1`
        // (:57011-57014) — the `noItem` flag never suppresses pot loot, so
        // the event passes through unchanged.
        if let Some(event) = plan.pot_event {
            kill.events.push(event);
        }
        // The guarded checks' shared tail: `destroyObject = true`, kill every
        // object cell of the type (`WorldGen.cs:57941-57952`, :86414-86428,
        // :86494-86502, :43466-43472, :47108-47115, :49416-49423),
        // `destroyObject = false`. Each `KillTile` is a full recursive kill
        // (inner `SquareTileFrame`), but the flag keeps the re-entered
        // guarded checks silent until the loop ends — a supported object
        // whose support died inside the loop stays put until a later break
        // near it (`WorldGen.cs:49841` resets the flag last in `Check3x2`).
        // The unguarded checks call `KillTile` plainly (:43513-43543, :46091,
        // :46910): the flag passes through unchanged, so a herb or a clay pot
        // dies inside another object's loop while that loop's own silence
        // holds.
        if guarded {
            *destroy_object = true;
            kill_plan_cells(world, &plan.cells, no_item, destroy_object, kill);
            *destroy_object = false;
        } else {
            kill_plan_cells(world, &plan.cells, no_item, destroy_object, kill);
        }
    }
}

/// Whether the cell's `Check*` routine carries the `WorldGen.destroyObject`
/// entry guard and wraps its kill loop in the flag. The multi-tile object
/// checks do (`Check1x2` :43337/:43442-43500, `Check2x1` :46976/:47108-47363,
/// `Check3x2` :49190/:49417-49841, `CheckSunflower` :53732/:53779-53791,
/// `CheckPot` :56800/:56855-57015, `CheckChest` :57926/:57970-57986,
/// `CheckDoorOpen` :86341/:86414-86428, `CheckDoorClosed` :86434/:86495-86510);
/// `CheckAlch` (:45972-46092), `CheckOnTable1x1` (:43503-43546) and
/// `CheckPile`'s 1x1 body (:46875-46971) have neither. `CheckPile` hands its
/// 2x1 variant (`frameY == 18`, :46899-46902 — the echo 649 is not in the
/// registry) to the guarded `Check2x1`, so only the 1x1 pile runs unwrapped.
fn destroy_object_guarded(cell: &Tile) -> bool {
    match cell.kind {
        78 | 82 | 83 | 84 => false,
        185 => cell.frame_y == 18,
        _ => true,
    }
}

/// Kills a plan's cells — one recursive `KillTile` per cell — and folds every
/// nested record into `kill`: a nested `KillTile` runs the full cascade in the
/// C#, so its drops, pot loot, herb rolls and pile loot all surface.
fn kill_plan_cells(
    world: &mut World,
    cells: &[(i32, i32)],
    no_item: bool,
    destroy_object: &mut bool,
    kill: &mut TileKill,
) {
    for &(mx, my) in cells {
        if let Some(inner) = kill_tile_cascading(world, mx, my, no_item, destroy_object) {
            kill.object_drops.extend(inner.object_drops);
            kill.events.extend(inner.events);
            kill.herb_events.extend(inner.herb_events);
            kill.pile_events.extend(inner.pile_events);
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

/// The destruction plan of one `Check*` routine: the object cells to
/// `KillTile` (active cells of the object's type — the exact cell guard each
/// C# kill loop applies), the drop vanilla would spawn (`None` for checks
/// whose loot is per-cell `KillTile_GetItemDrops`, an event, or RNG-deferred —
/// `check_alch`/`check_pot`/`check_pile*`/the piles) and the pot-loot event.
struct CheckPlan {
    drop: Option<ObjectDrop>,
    cells: Vec<(i32, i32)>,
    pot_event: Option<PotEvent>,
    herb_event: Option<HerbEvent>,
    pile_event: Option<PileLootEvent>,
}

/// The `TileFrameImportant` object dispatch (`WorldGen.cs:84962`): runs the
/// `Check*` routine for the cell's registry kind and returns its destruction
/// plan. Cell routines: 10 → `CheckDoorClosed` (:86432), 11 →
/// `CheckDoorOpen` (:86339), 14/186/187 → `Check3x2` (:49187, dispatch
/// :85980-85988), 15 → `Check1x2` (:43335), 18 → `Check2x1` (:46973), 78 →
/// `CheckOnTable1x1` (:43503, the 13/33/49/50/78/174/372/646/707 group at
/// :85604-85614), 21 → `CheckChest` (:57923, via `TileID.Sets.BasicChest`
/// :85616-85619), 27 → `CheckSunflower` (:85639), 28 → `CheckPot` (:85643),
/// 185 → `CheckPile` (:85275), 82/83/84 → `CheckAlch` (the `Main.tileAlch`
/// default arm, :85830-85832).
fn check_object(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let kind = world.tile(x, y)?.kind;
    match kind {
        10 => check_door_closed(world, x, y),
        11 => check_door_open(world, x, y),
        14 | 186 | 187 => check_3x2(world, x, y, kind),
        15 => check_chair(world, x, y),
        18 => check_bench(world, x, y),
        78 => check_on_table_1x1(world, x, y),
        21 => check_chest(world, x, y),
        27 => check_sunflower(world, x, y),
        28 => check_pot(world, x, y),
        82 | 83 | 84 => check_alch(world, x, y),
        185 => check_pile(world, x, y),
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
        drop: Some(ObjectDrop {
            tile_kind: 10,
            x,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, 10),
        pot_event: None,
        herb_event: None,
        pile_event: None,
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
        drop: Some(ObjectDrop {
            tile_kind: 11,
            x,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, 11),
        pot_event: None,
        herb_event: None,
        pile_event: None,
    })
}

/// `WorldGen.Check3x2` (`WorldGen.cs:49187-49430`) for the wooden table
/// (type 14) and the large piles (186/187 — dispatch `WorldGen.cs:85980-85988`):
/// origin from `frameY % 36 / 18` and `frameX / 18` mod 3 (:49199-49209 — the
/// hardcoded 36 is the `Style3x2` full height; styles are horizontal, so
/// `frameY` encodes the row), every cell must be active of `kind` with the
/// expected frames (:49219-49226), and the per-column support judge branches
/// on the kind (:49280-49373): 186/187 refuse `SolidTileAllowBottomSlope`
/// failures, Boulders (`InvalidTileForPilesOrSpeleothems` :38979-38994, set
/// `TileID.cs:504`) and each style's ground family; the table keeps the
/// default `SolidTileAllowBottomSlope` arm (the trailing `else if`,
/// :49396-49399). Table style 25 spans a single row (`flag2`, :49223-49229).
/// The 187→186 conversion (:49379-49393) runs before the early return: an
/// intact snowy 187 over non-dirt ground becomes a 186 (`frameX -= 378` on
/// all six cells) and survives — the kill loop below then matches zero cells
/// of `kind`. Drop: only the table carries one (`GetItemDrop_Tables(style)`,
/// :49424-49426, 32x32 at the trigger cell; style 0 → 32 :42194); the piles'
/// case group is empty (:65759+).
fn check_3x2(world: &mut World, x: i32, y: i32, kind: u16) -> Option<CheckPlan> {
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
    // `flag2` (:49223-49229): table style 25 is a single-row 3x1.
    let rows = if kind == 14 && style == 25 { 1 } else { 2 };
    let mut broken = false;
    let mut region = Vec::with_capacity(6);
    for k in ox..ox + 3 {
        for l in oy..oy + rows {
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
        // The per-column support judge (:49280-49373 + the trailing
        // `else if` :49396-49399). The 186/ style sets re-judge the ground
        // family from `TileID.Sets` (`TileID.cs:89-94/558`).
        match kind {
            186 => {
                if !solid_tile_allow_bottom_slope(world, k, oy + rows)
                    || invalid_tile_for_piles(world, k, oy + rows)
                {
                    broken = true;
                } else if let Some(g) = world.tile(k, oy + rows).filter(|g| g.active()) {
                    match style {
                        26..=31 => {
                            if !(td::SET_ICES_SNOW.has(g.kind)
                                || g.kind == 162
                                || g.kind == 224)
                            {
                                broken = true;
                            }
                        }
                        32..=34 => {
                            if !(td::SET_MUD.has(g.kind) || g.kind == 70) {
                                broken = true;
                            }
                        }
                        _ => {}
                    }
                }
            }
            187 => {
                if !solid_tile_allow_bottom_slope(world, k, oy + rows)
                    || invalid_tile_for_piles(world, k, oy + rows)
                {
                    broken = true;
                } else if let Some(g) = world.tile(k, oy + rows).filter(|g| g.active()) {
                    match style {
                        0..=5 => {
                            if !(td::SET_MUD.has(g.kind) || g.kind == 60 || g.kind == 226) {
                                broken = true;
                            }
                        }
                        6..=8 => {
                            if !matches!(g.kind, 57 | 58 | 75 | 76) {
                                broken = true;
                            }
                        }
                        29..=34 => {
                            if !(td::SET_CONVERSION_SAND.has(g.kind)
                                || td::SET_CONVERSION_HARDENED_SAND.has(g.kind)
                                || td::SET_CONVERSION_SANDSTONE.has(g.kind))
                            {
                                broken = true;
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {
                if !solid_tile_allow_bottom_slope(world, k, oy + rows) {
                    broken = true;
                }
            }
        }
    }
    // The 187→186 conversion (:49379-49393): the origin's `frameX` in
    // 756..=900 (the snowy LargePiles2 styles) and none of the six ground
    // cells is dirt/grass — every cell becomes a 186 with `frameX -= 378`.
    // Runs even when `flag` is set; the kill loop's kind filter then spares
    // the converted cells, exactly like the C# `type == 187` test.
    if kind == 187
        && world
            .tile(ox, oy)
            .is_some_and(|c| (756..=900).contains(&i32::from(c.frame_x)))
        && (0..3).all(|d| {
            world
                .tile(ox + d, oy + rows)
                .is_none_or(|g| !matches!(g.kind, 2 | 477 | 492))
        })
    {
        for l in oy..oy + rows {
            for k in ox..ox + 3 {
                let cell = world.tile_mut(k, l).unwrap();
                cell.kind = 186;
                cell.frame_x -= 378;
                touch(world, k, l);
            }
        }
    }
    if !broken {
        return None;
    }
    // The drop is the table's alone (`GetItemDrop_Tables(style)`,
    // :49424-49426 — the slice covers style 0 → 32); the piles' case group
    // is empty.
    Some(CheckPlan {
        drop: (kind == 14).then(|| ObjectDrop {
            tile_kind: 14,
            x,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, kind),
        pot_event: None,
        herb_event: None,
        pile_event: None,
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
        drop: Some(ObjectDrop {
            tile_kind: 15,
            x,
            y: oy,
            item: None,
        }),
        cells: killable_cells(world, &region, 15),
        pot_event: None,
        herb_event: None,
        pile_event: None,
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
        drop: Some(ObjectDrop {
            tile_kind: 18,
            x: ox,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, 18),
        pot_event: None,
        herb_event: None,
        pile_event: None,
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
        drop: Some(ObjectDrop {
            tile_kind: 21,
            x,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, 21),
        pot_event: None,
        herb_event: None,
        pile_event: None,
    })
}

/// `WorldGen.InvalidTileForPilesOrSpeleothems` (`WorldGen.cs:38979-38994`):
/// an active support cell whose kind is in `TileID.Sets.Boulders`
/// (`TileID.cs:504`) refuses to carry piles/speleothems. Inactive and
/// out-of-world cells are valid (`!active` / the `InWorld` guard → false).
fn invalid_tile_for_piles(world: &World, x: i32, y: i32) -> bool {
    /// `TileID.Sets.Boulders` (`TileID.cs:504`).
    const BOULDERS: &[u16] = &[138, 484, 664, 665, 711, 712, 713, 714, 715, 716];
    world
        .tile(x, y)
        .is_some_and(|t| t.active() && BOULDERS.contains(&t.kind))
}

/// `WorldGen.SolidTile2(Tile)` (`WorldGen.cs:70134-70147`; the indexed
/// overload :70492-70504): active, solid, slopeless (or a platform with a top
/// slope — the refinement cannot occur in the port), non-halfBrick, not
/// actuated. Unlike [`solid_tile`] solid-top tiles pass; an out-of-world cell
/// is refused (the C# try/catch returns false) — the in-world null case does
/// not exist in the port's dense array.
fn solid_tile2(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && td::TILE_SOLID.has(t.kind)
        && ((td::SET_PLATFORMS.has(t.kind) && (t.half_brick() || t.top_slope()))
            || t.slope() == 0)
        && !t.half_brick()
        && crate::tileobject::nactive(t)
}

/// `Tile.anyLava` (`Tile.cs:410-415`): a positive liquid amount whose kind
/// flag is lava (`Tile.lava()`, `Tile.cs:345-347`).
fn any_lava(tile: &Tile) -> bool {
    tile.liquid > 0 && tile.liquid_kind() == crate::tile::LiquidKind::Lava
}

/// The alch soil table — the per-variant ground families shared by
/// `WorldGen.PlaceAlch` (`WorldGen.cs:45393-45441`) and `CheckAlch`
/// (:45998-46086); every variant also accepts the clay pot 78 and the
/// planter box 380.
const ALCH_SOILS: [&[u16]; 7] = [
    &[2, 477, 492, 78, 380, 109],
    &[60, 78, 380],
    &[0, 59, 78, 380],
    &[661, 662, 199, 203, 23, 25, 78, 380],
    &[53, 78, 380, 116],
    &[57, 633, 78, 380],
    &[78, 380, 147, 161, 163, 164, 200],
];

/// `WorldGen.CheckSunflower` (`WorldGen.cs:53729-53792`): the 2x4 sunflower
/// needs all eight cells active type-27 with the origin derived from the
/// frames (:53739-53751 — the column from `frameX / 18` reduced mod 2, the
/// row from `frameY / 18`) and `nactive` ground of {2,477,492,109,60,633}
/// that is also `SolidTile` under both columns (:53756-53770 — 78/380 are
/// absent, unlike the herbs). On a break every matching cell dies
/// (:53773-53783) and the sunflower item spawns at the check's trigger cell
/// (:53787-53791 — `Item.NewItem(…, 63)`, 32x32 at `(i, j)`).
fn check_sunflower(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    /// :53758-53764 — the ground family.
    const SOILS: &[u16] = &[2, 477, 492, 109, 60, 633];
    let t = world.tile(x, y)?;
    let ox = x - i32::from(t.frame_x) / 18 % 2; // :53739-53747
    let oy = y - i32::from(t.frame_y) / 18; // :53741
    let mut broken = false;
    let mut region = Vec::with_capacity(8);
    for k in ox..ox + 2 {
        for l in oy..oy + 4 {
            region.push((k, l));
            // :53746-53755 — `nactive`, the type, the column and the row.
            let intact = world.tile(k, l).is_some_and(|c| {
                crate::tileobject::nactive(c)
                    && c.kind == 27
                    && i32::from(c.frame_x) / 18 % 2 == k - ox
                    && i32::from(c.frame_y) == (l - oy) * 18
            });
            if !intact {
                broken = true;
            }
        }
        // :53756-53770 — `nactive` + the soil set + `SolidTile`.
        let ground = world.tile(k, oy + 4);
        if !ground.is_some_and(|g| crate::tileobject::nactive(g) && SOILS.contains(&g.kind))
            || !solid_tile(world, k, oy + 4)
        {
            broken = true;
        }
    }
    if !broken {
        return None;
    }
    Some(CheckPlan {
        drop: Some(ObjectDrop {
            tile_kind: 27,
            x,
            y,
            item: Some(63), // :53787-53791
        }),
        cells: killable_cells(world, &region, 27),
        pot_event: None,
        herb_event: None,
        pile_event: None,
    })
}

/// The `CheckPot` break sound (`WorldGen.cs:56856-56867`): styles 7-9 break
/// with sound 6, 16-24 with 4, everything else with 13. The app plays it at
/// the event's cell.
pub fn pot_break_sound(style: i32) -> u16 {
    match style {
        7..=9 => 6,
        16..=24 => 4,
        _ => 13,
    }
}

/// `WorldGen.CheckPot` (`WorldGen.cs:56797-57016`) for tile 28 — the echo pot
/// 653 is out of scope. The 2x2 needs all four cells active type-28 with the
/// origin derived from the frames (:56808-56817 — the column from `frameX /
/// 18` reduced mod 2, the row reduced mod 2 while the style accumulates
/// `frameY / 36`) and `SolidTile2` ground under both columns (:56845-56849 —
/// solid-top platforms pass, `Main.tileSolid[19] = true`, `Main.cs:7893`).
/// On a break the sound plays by style (:56856-56867, [`pot_break_sound`]),
/// every matching cell dies (:56871-56877) and the loot event fires
/// (:57011-57014). The Gore shards (:56878-56953) have no port mechanism
/// (only the leaf `LeafGore`) and stay deferred — no item spawns here for
/// tile 28 (`Item.NewItem` exists only in the 653 arm, :56955-57009).
fn check_pot(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let t = world.tile(x, y)?;
    let ox = x - i32::from(t.frame_x) / 18 % 2; // :56808-56811
    let mut row = i32::from(t.frame_y) / 18; // :56812-56817
    let mut style = 0;
    while row > 1 {
        row -= 2;
        style += 1;
    }
    let oy = y - row;
    let mut broken = false;
    let mut region = Vec::with_capacity(4);
    for k in ox..ox + 2 {
        for l in oy..oy + 2 {
            region.push((k, l));
            // :56829-56837 — `active` (not `nactive`), the type, the column
            // and the style-row frame.
            let intact = world.tile(k, l).is_some_and(|c| {
                c.active()
                    && c.kind == 28
                    && i32::from(c.frame_x) / 18 % 2 == k - ox
                    && i32::from(c.frame_y) == (l - oy) * 18 + style * 36
            });
            if !intact {
                broken = true;
            }
        }
        if !solid_tile2(world, k, oy + 2) {
            broken = true; // :56845-56849
        }
    }
    if !broken {
        return None;
    }
    Some(CheckPlan {
        // No `Item.NewItem` for tile 28 — the loot runs through the event.
        drop: None,
        cells: killable_cells(world, &region, 28),
        pot_event: Some(PotEvent { style, x, y }),
        herb_event: None,
        pile_event: None,
    })
}

/// `WorldGen.CheckPile` (`WorldGen.cs:46875-46971`) for tile 185: a `frameY`
/// of 18 is the 2x1 variant and routes to [`check_pile_2x1`] (:46896);
/// otherwise the pile needs `SolidTileAllowBottomSlope` below (:46898-46902)
/// and, for the ground-judged styles, the soil family under it
/// (:46915-46969): 36-47 the snow/ice family (162/224 included), 54-59 and
/// 73-76 the sand families; the remaining styles judge nothing. The kill is
/// the single cell (`KillTile(i, y)`); no item spawns for 185 (the empty
/// per-cell group :65759+ — the coin-pile rolls of the 2x1 arm are RNG drops,
/// phase 3).
fn check_pile(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let t = *world.tile(x, y)?;
    if t.frame_y == 18 {
        return check_pile_2x1(world, x, y, t.kind);
    }
    if !solid_tile_allow_bottom_slope(world, x, y + 1) {
        return Some(CheckPlan {
            drop: None,
            cells: killable_cells(world, &[(x, y)], t.kind),
            pot_event: None,
            herb_event: None,
            pile_event: None,
        });
    }
    // :46904-46913 — only 185 carries the ground rules (the echo 649 is out
    // of scope); a dead ground cell below returns without a kill.
    if t.kind != 185 {
        return None;
    }
    let Some(below) = world.tile(x, y + 1).filter(|g| g.active()) else {
        return None;
    };
    let style = i32::from(t.frame_x) / 18;
    let grounded = match style {
        // :46915-46931 — the snow/ice styles.
        36..=47 => td::SET_ICES_SNOW.has(below.kind) || below.kind == 162 || below.kind == 224,
        // :46932-46952 — the sand styles.
        54..=59 | 73..=76 => {
            td::SET_CONVERSION_SAND.has(below.kind)
                || td::SET_CONVERSION_HARDENED_SAND.has(below.kind)
                || td::SET_CONVERSION_SANDSTONE.has(below.kind)
        }
        // :46953-46968 — the explicit empty cases and everything before 36.
        _ => true,
    };
    if grounded {
        return None;
    }
    Some(CheckPlan {
        drop: None,
        cells: killable_cells(world, &[(x, y)], t.kind),
        pot_event: None,
        herb_event: None,
        pile_event: None,
    })
}

/// The `Check2x1` type-185 arm (`WorldGen.cs:46973-47140`): the 2x1 pile
/// needs both cells active of `kind` with the right-hand frame continuing the
/// left's by 18 (:47000-47006), `SolidTileAllowBottomSlope` below both
/// columns (the default arm, :47026-47031), no Boulder under either column
/// and the style's ground family (`num5 = frameX / 36`, :47044-47086: 25-30
/// snow/ice, 41-46 and 62-64 sand). The grass-covered variants (`frameX`
/// 1368-1458) revert to the bare pile (`frameX -= 1368` on both cells) when
/// neither ground cell is dirt/grass (:47040-47048) — before the style is
/// re-judged. The coin-pile `Item.NewItem` rolls (:47142-47232) are RNG drops
/// — phase 3.
fn check_pile_2x1(world: &mut World, x: i32, y: i32, kind: u16) -> Option<CheckPlan> {
    let t = world.tile(x, y)?;
    let ox = x - i32::from(t.frame_x) / 18 % 2; // :46981-46988
    let left = world.tile(ox, y);
    let right = world.tile(ox + 1, y);
    // :47000-47006 — the pair intact.
    let mut broken = !right.is_some_and(|c| {
        c.frame_x == left.map(|l| l.frame_x).unwrap_or(i16::MIN) + 18
    }) || !left.is_some_and(|c| c.active() && c.kind == kind)
        || !right.is_some_and(|c| c.active() && c.kind == kind);
    // :47026-47031 — the default support arm.
    if !solid_tile_allow_bottom_slope(world, ox, y + 1)
        || !solid_tile_allow_bottom_slope(world, ox + 1, y + 1)
    {
        broken = true;
    }
    if kind == 185 {
        // :47041-47048 — the grass-covered variants revert on non-dirt
        // ground (both columns).
        let ground_not_dirt =
            |world: &World, dx: i32| {
                world
                    .tile(ox + dx, y + 1)
                    .is_none_or(|g| !matches!(g.kind, 2 | 477 | 492))
            };
        if left.is_some_and(|c| (1368..=1458).contains(&i32::from(c.frame_x)))
            && ground_not_dirt(world, 0)
            && ground_not_dirt(world, 1)
        {
            for dx in 0..2 {
                if let Some(cell) = world.tile_mut(ox + dx, y) {
                    cell.frame_x -= 1368;
                    touch(world, ox + dx, y);
                }
            }
        }
        // :47050-47086 — no Boulder below, then the style's ground family
        // (`num5` reads the origin after the conversion; it is only
        // load-bearing when the pair is intact, so the origin is active).
        let origin = world.tile(ox, y);
        let num5 = i32::from(origin.map(|c| c.frame_x).unwrap_or(0)) / 36
            + (i32::from(origin.map(|c| c.frame_y).unwrap_or(0)) / 18 - 1) * 52;
        for dx in 0..2 {
            if invalid_tile_for_piles(world, ox + dx, y + 1) {
                broken = true;
                break;
            }
            if let Some(g) = world.tile(ox + dx, y + 1).filter(|g| g.active()) {
                match num5 {
                    25..=30 => {
                        if !(td::SET_ICES_SNOW.has(g.kind) || g.kind == 162 || g.kind == 224) {
                            broken = true;
                        }
                    }
                    41..=46 | 62..=64 => {
                        if !(td::SET_CONVERSION_SAND.has(g.kind)
                            || td::SET_CONVERSION_HARDENED_SAND.has(g.kind)
                            || td::SET_CONVERSION_SANDSTONE.has(g.kind))
                        {
                            broken = true;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    if !broken {
        return None;
    }
    let region = [(ox, y), (ox + 1, y)];
    // `int frameX = Main.tile[num, y].frameX; if (!Main.tile[num, y].active())
    // frameX = Main.tile[num + 1, y].frameX;` (:47127-47131) — read before the
    // kills, after the grass-cover conversion. Only type 185 carries the loot
    // block (`if (type == 185)`, :47130).
    let pile_event = if kind == 185 {
        let frame_x = world
            .tile(ox, y)
            .filter(|c| c.active())
            .or_else(|| world.tile(ox + 1, y))
            .map(|c| i32::from(c.frame_x))
            .unwrap_or(0);
        Some(PileLootEvent {
            frame_x,
            x: ox,
            y,
        })
    } else {
        None
    };
    Some(CheckPlan {
        drop: None,
        cells: killable_cells(world, &region, kind),
        pot_event: None,
        herb_event: None,
        pile_event,
    })
}

/// `WorldGen.AnchorValid(tile, AnchorType.Table)` (`WorldGen.cs:51841-51871`)
/// reduced to the Table bit: `nactive`, then the platform-frame arm (the flat
/// platform segments — `PlatformFrameWidth()` is tile 19's full width 18,
/// `TileObjectData.cs:5121-5124`/`2127-2139`), the solid-top arm, or the plain
/// tile-table arm (`blockType() == 0`, [`crate::tileobject::block_type`]).
fn anchor_valid_table(below: &Tile) -> bool {
    if !crate::tileobject::nactive(below) {
        return false;
    }
    if td::SET_PLATFORMS.has(below.kind) {
        let seg = i32::from(below.frame_x) / 18;
        return (!below.half_brick() && (0..=7).contains(&seg))
            || (12..=16).contains(&seg)
            || (25..=26).contains(&seg);
    }
    if td::TILE_SOLID.has(below.kind) && td::TILE_SOLID_TOP.has(below.kind) {
        return true;
    }
    crate::tileobject::is_table(below.kind)
        && crate::tileobject::block_type(below) == 0
}

/// `WorldGen.CheckOnTable1x1` (`WorldGen.cs:43503-43546`) for the clay pot
/// (dispatch group 13/33/49/50/78/174/372/646/707 — :85604-85614; 78 is the
/// only group member the registry carries). The support cell below decides:
/// a top slope kills unless the platform corner rule holds (:43513-43515 —
/// `blockType` 3/slope 2 leans on an active flat platform at `x - 1`,
/// `blockType` 2/slope 1 on one at `x + 1`), a half brick kills
/// (:43525-43528), the table anchors survive before the type test
/// (:43530-43532), and — the type-78 exception, :43534-43540 — any
/// `SolidTileAllowBottomSlope` below carries the pot; the group's
/// unconditional kill (:43541-43543) is the non-78 path. The kill is the
/// single cell; no `Item.NewItem` runs here — the pot's 222 comes from the
/// per-cell `KillTile_GetItemDrops` `case 78` (:64711-64713), resolved
/// app-side through `registry::tile_drop` (phase 3), so the record carries
/// `item: None`.
fn check_on_table_1x1(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let kind = world.tile(x, y)?.kind;
    // `Tile tile = Main.tile[x, y + 1]` — an uninitialised support cell
    // returns without a kill (:43505-43508). The 1x1 object is its own
    // origin: no frame reconstruction.
    let below = *world.tile(x, y + 1)?;
    let kill = if below.top_slope() {
        // :43510-43524 — every top slope kills unless the platform corner
        // rule holds (a valid partner: an active platform with
        // `blockType() == 0`).
        let corner = |world: &World, dx: i32, slope: u8| {
            below.slope() == slope
                && world.tile(x + dx, y + 1).is_some_and(|n| {
                    n.active()
                        && crate::tileobject::block_type(n) == 0
                        && td::SET_PLATFORMS.has(n.kind)
                })
        };
        !(corner(world, -1, 2) || corner(world, 1, 1))
    } else if below.half_brick() {
        true // :43525-43528
    } else {
        // :43530-43532 — the table anchors survive before the type test.
        if anchor_valid_table(&below)
            || (below.active() && crate::tileobject::is_table(below.kind))
        {
            return None;
        }
        // :43534-43540 — the type-78 exception.
        !solid_tile_allow_bottom_slope(world, x, y + 1)
    };
    if !kill {
        return None;
    }
    Some(CheckPlan {
        drop: Some(ObjectDrop {
            tile_kind: kind,
            x,
            y,
            item: None,
        }),
        cells: killable_cells(world, &[(x, y)], kind),
        pot_event: None,
        herb_event: None,
        pile_event: None,
    })
}

/// `WorldGen.CheckAlch` (`WorldGen.cs:45972-46092`): the herb dies when the
/// soil below is not `nactive` or is a half brick (:45983-45990 — slopes
/// pass: no slope test exists here), or — per the `frameX / 18` variant —
/// when the soil is not in the variant's set ([`ALCH_SOILS`], :45998-46086)
/// or the herb cell itself holds lava (`anyLava` — every variant except 5).
/// Variant 5 (Fireblossom) never dies from lava: the cell blinks instead —
/// `liquid > 16` turns a mature 83 into the blooming 84, `liquid <= 16`
/// turns an 84 back (:46052-46075; skipped for the immature 82 and on
/// multiplayer clients — the port is single-player). The unconditional
/// `frameY = 0` (:45991) is kept as a gated write (the port's herbs always
/// carry it). The kill is the single cell; no item spawns here (the per-cell
/// herb drops are phase 3 — `KillTile_GetItemDrops` :65674-65698).
fn check_alch(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let herb = *world.tile(x, y)?;
    // :45983-45990 — the support pair.
    let mut flag = world
        .tile(x, y + 1)
        .is_none_or(|soil| !crate::tileobject::nactive(soil) || soil.half_brick());
    // :45991 — `frameY = 0`.
    if herb.frame_y != 0 {
        world.tile_mut(x, y).unwrap().frame_y = 0;
        touch(world, x, y);
    }
    if !flag {
        let style = i32::from(herb.frame_x) / 18;
        match style {
            s @ 0..=6 => {
                // :45998-46086 — the variant's soil set.
                if !world
                    .tile(x, y + 1)
                    .is_some_and(|soil| ALCH_SOILS[s as usize].contains(&soil.kind))
                {
                    flag = true;
                }
                if s == 5 {
                    // :46052-46075 — the Fireblossom blink (no lava death).
                    if herb.kind != 82 && any_lava(&herb) {
                        let cell = world.tile_mut(x, y).unwrap();
                        if herb.liquid > 16 {
                            if cell.kind == 83 {
                                cell.kind = 84;
                                touch(world, x, y);
                            }
                        } else if cell.kind == 84 {
                            cell.kind = 83;
                            touch(world, x, y);
                        }
                    }
                } else if any_lava(&herb) {
                    flag = true; // :46006-46008 and peers
                }
            }
            // No case matches: the C# switch falls through unchecked
            // (:45996) — the seed fall-through's mirror.
            _ => {}
        }
    }
    if !flag {
        return None;
    }
    Some(CheckPlan {
        drop: None,
        cells: killable_cells(world, &[(x, y)], herb.kind),
        pot_event: None,
        // The killed cell's per-cell drop rolls (`KillTile_GetItemDrops`,
        // `WorldGen.cs:65674-65698`) ride the herb event; the app resolves
        // them with the RNG streams and the harvest predicates.
        herb_event: Some(HerbEvent {
            kind: herb.kind,
            style: i32::from(herb.frame_x) / 18,
            x,
            y,
        }),
        pile_event: None,
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
/// tile 82 is registered but explicitly excluded from that route (:39772) and
/// the rest keeps the legacy `WorldGen.PlaceTile` paths (`WorldGen.cs:59342`):
/// the `Main.tileAlch` kinds through `PlaceAlch` (:60177-60181), tile 380
/// through its own case (:60226-60230), everything else through the legacy
/// single-tile write. `style` is the item's `placeStyle`, `dir` the player
/// direction (`Player.direction`: -1 left, 1 right). `false` = refused, the
/// world is unchanged.
pub fn place_item_tile(world: &mut World, x: i32, y: i32, kind: u16, style: i32, dir: i32) -> bool {
    // Registry route (`Player.cs:39770-39772`): a registered `TileObjectData`
    // def goes through `TileObject.CanPlace`/`Place` — except tile 82, which
    // vanilla routes to the legacy `PlaceTile` even though `TileObjectData`
    // registers it (the herb entry exists for the cascade gateway and frames).
    if kind != 82 {
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
    }
    // Legacy `WorldGen.PlaceTile` dispatcher (`WorldGen.cs:60177-60230`):
    // `Main.tileAlch` kinds go to `PlaceAlch`, tile 380 has its own case.
    if crate::tileobject::tile_alch(kind) {
        return place_alch(world, x, y, style);
    }
    if kind == 380 {
        return place_planter_box(world, x, y, style);
    }
    // Legacy single-tile route (`WorldGen.PlaceTile`).
    place_tile(world, x, y, kind)
}

/// `WorldGen.PlaceAlch` (`WorldGen.cs:45380-45479`): the seed write — plants
/// tile 82 (`frameX = 18 * style, frameY = 0`) on the style's soil, judged
/// 1:1 with the C# soil/liquid rules. `style` is the seed's `placeStyle`
/// (0 Daybloom … 6 Shiverthorn). `false` = refused, the world is unchanged.
pub fn place_alch(world: &mut World, x: i32, y: i32, style: i32) -> bool {
    // :45383-45388 — the C# null-creates both cells; the port requires them
    // in-world (an out-of-world write would be lost anyway).
    let Some(herb) = world.tile(x, y).copied() else {
        return false;
    };
    let Some(soil) = world.tile(x, y + 1).copied() else {
        return false;
    };
    // :45390-45392 — the herb cell must be empty and the soil below must be
    // `nactive()` — active and not actuated (`Tile.nactive`, `Tile.cs:240-246`;
    // wiring actuation keeps `FLAG_ACTIVE` and sets `FLAG_INACTIVE`) — and flat
    // (`!halfBrick() && slope() == 0`).
    if herb.active()
        || !crate::tileobject::nactive(&soil)
        || soil.half_brick()
        || soil.slope() != 0
    {
        return false;
    }
    // :45393-45441 — the per-style soil sets (every style also accepts the
    // clay pot 78 and the planter box 380); shared with `check_alch`.
    let soil_rejected = match style {
        s @ 0..=6 => !ALCH_SOILS[s as usize].contains(&soil.kind),
        // No case matches: the C# switch falls through with the reject flag
        // still false — an out-of-range style plants with no checks at all.
        _ => false,
    };
    // :45442-45455 — the liquid rules: styles 0-3 refuse any liquid, 4/6
    // refuse lava only, 5 (the ash style) refuses everything but lava.
    let herb_lava = herb.liquid_kind() == crate::tile::LiquidKind::Lava;
    let liquid_rejected = match style {
        0..=3 => herb.liquid > 0,
        4 | 6 => herb.liquid > 0 && herb_lava,
        5 => herb.liquid > 0 && !herb_lava,
        _ => false,
    };
    if soil_rejected || liquid_rejected {
        return false;
    }
    // :45458-45477 — the write: `active, type = 82, frameX = 18 * style,
    // frameY = 0` plus `CopyPaintAndCoating(tile2)` (`Tile.cs:867-873` —
    // color and fullbright; the port has no invisibleBlock state, which only
    // special-world tiles carry). Wall and liquid of the cell stay.
    let i = world.index(x, y);
    let cell = &mut world.tiles[i];
    cell.flags |= crate::tile::FLAG_ACTIVE;
    cell.kind = 82;
    cell.frame_x = (18 * style) as i16;
    cell.frame_y = 0;
    cell.set_paint(soil.paint());
    cell.flags = (cell.flags & !crate::tile::FLAG_FULLBRIGHT)
        | (soil.flags & crate::tile::FLAG_FULLBRIGHT);
    touch(world, x, y);
    // `Main.tileFrameImportant[82..84]` are boot statics (`Main.cs:7685-7687`);
    // mirror all three so the re-frame below keeps the fresh herb and any
    // sibling-stage herbs in the 5x5 (their frames are hand-written).
    for alch_kind in crate::tileobject::TILE_ALCH {
        ensure_frame_important(world, *alch_kind);
    }
    after_edit(world, x, y);
    true
}

/// `WorldGen.PlaceTile` case 380 (`WorldGen.cs:60226-60230`): the planter-box
/// legacy write — `frameY = 18 * style`, active, type 380. The C# case
/// touches no other field, so `frameX` stays exactly as the cell had it (a
/// virgin cell's frames are 0 — `Tile.cs:92-104`). Tile 380 is NOT
/// registered in `TileObjectData` (vanilla routes it through this legacy
/// branch), so it never joins [`crate::tileobject::TABLE`].
fn place_planter_box(world: &mut World, x: i32, y: i32, style: i32) -> bool {
    let Some(prev) = world.tile(x, y).copied() else {
        return false;
    };
    if prev.active() || prev.wall == WALL_UNBREAKABLE {
        return false;
    }
    let i = world.index(x, y);
    let cell = &mut world.tiles[i];
    cell.flags |= crate::tile::FLAG_ACTIVE;
    cell.kind = 380;
    cell.frame_y = (18 * style) as i16;
    touch(world, x, y);
    ensure_frame_important(world, 380); // `Main.cs:7632`
    after_edit(world, x, y);
    true
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
}

/// The world-interaction dispatcher — the port of the right-click tile hook:
/// `Player.LookForTileInteractions` (`Player.cs:31937`) →
/// `TileInteractionsCheck` (`:32031`, active tiles only) →
/// `TileInteractionsUse` (`:32118`), whose type chain routes tile 10/11
/// through the door branch (`:32835-32865`). The port dispatches on the
/// registry def's `interaction` tag instead of a type chain (Decision #3) so
/// chests/signs can join as data later. `direction` is `Player.direction`
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
    // The port routes on the registry def's `interaction` tag (Decision #3)
    // where the C# chain tests `Main.tile[myX, myY].type` literals
    // (`Player.cs:32835`/:32857 for 10/11).
    let Some(def) = crate::tileobject::def_for_tile(t.kind) else {
        return TileInteraction::None;
    };
    match def.interaction {
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
                    y: dy,
                item: None
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
                    y: dy,
                item: None
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
                    y: dy,
                item: None
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
                y: 18,
                item: None
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
                    y: dy,
                item: None
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
                    y: dy,
                item: None
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
                y: dy,
                item: None
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
                y: 16,
                item: None
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

    // ---- M1.4-TO-4 phase 1 (DW-1.1 / DW-1.3 / DW-1.5): the seed and
    // planter-box placement paths -------------------------------------------

    /// The stone floor of `object_world` repainted to `kind` — the soil row
    /// `PlaceAlch`/the planter-box case judge.
    fn soil_world(kind: u16) -> World {
        let mut w = object_world();
        for x in 0..40 {
            let i = w.index(x, 20);
            w.tiles[i].kind = kind;
        }
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    fn set_liquid(w: &mut World, x: i32, y: i32, amount: u8, lava: bool) {
        let ti = w.index(x, y);
        let t = &mut w.tiles[ti];
        t.liquid = amount;
        if lava {
            t.flags |= crate::tile::FLAG_LIQUID_LAVA;
        }
    }

    /// The C# soil table of `PlaceAlch` (`WorldGen.cs:45393-45441`) — style →
    /// the soils it accepts (78/380 included in every style).
    const ALCH_SOIL_TABLE: [(i32, &[u16]); 7] = [
        (0, &[2, 477, 492, 78, 380, 109]),
        (1, &[60, 78, 380]),
        (2, &[0, 59, 78, 380]),
        (3, &[661, 662, 203, 199, 23, 25, 78, 380]),
        (4, &[53, 78, 380, 116]),
        (5, &[57, 633, 78, 380]),
        (6, &[78, 380, 147, 161, 163, 164, 200]),
    ];

    #[test]
    fn test_DW_1_1_seeds_plant_on_csharp_soil_sets() {
        // Every seed style plants on every soil of its C# set — the tile
        // reads `82` with `frameX = 18*style, frameY = 0` (:45458-45477).
        for (style, soils) in ALCH_SOIL_TABLE {
            for soil in soils {
                let mut w = soil_world(*soil);
                assert!(
                    place_item_tile(&mut w, 10, 19, 82, style, 1),
                    "style {style} must plant on soil {soil}"
                );
                let t = *w.tile(10, 19).unwrap();
                assert!(t.active() && t.kind == 82, "soil {soil}");
                assert_eq!((t.frame_x, t.frame_y), ((18 * style) as i16, 0), "soil {soil}");
            }
        }
    }

    #[test]
    fn test_DW_1_1_seed_liquid_rules_by_style() {
        // :45442-45455 — styles 0-3 refuse ANY liquid in the herb cell, 4/6
        // refuse lava only, 5 (ash) refuses everything but lava. Each style
        // sits on the first soil of its own set so only the liquid verdict
        // differs.
        for style in 0..=6i32 {
            let soil = ALCH_SOIL_TABLE[style as usize].1[0];
            // Water.
            let mut w = soil_world(soil);
            set_liquid(&mut w, 10, 19, 200, false);
            assert_eq!(
                place_item_tile(&mut w, 10, 19, 82, style, 1),
                matches!(style, 4 | 6),
                "style {style} in water"
            );
            // Lava.
            let mut wl = soil_world(soil);
            set_liquid(&mut wl, 10, 19, 200, true);
            assert_eq!(
                place_item_tile(&mut wl, 10, 19, 82, style, 1),
                style == 5,
                "style {style} in lava"
            );
        }
        // The style-5 positive control actually wrote the herb.
        let mut w5 = soil_world(57);
        set_liquid(&mut w5, 10, 19, 200, true);
        assert!(place_item_tile(&mut w5, 10, 19, 82, 5, 1));
        let t = *w5.tile(10, 19).unwrap();
        assert!(t.active() && t.kind == 82 && t.liquid == 200, "liquid survives the write");
    }

    #[test]
    fn test_DW_1_1_seed_rejected_on_slope_or_half_brick_soil() {
        // :45390-45392 — the soil must be flat (`!halfBrick && slope == 0`).
        let cases: [(&str, fn(&mut Tile)); 2] = [
            ("half brick", |t: &mut Tile| t.flags |= crate::tile::FLAG_HALF_BRICK),
            ("slope", |t: &mut Tile| t.set_slope(1)),
        ];
        for (label, mutate) in cases {
            let mut w = soil_world(2);
            let i = w.index(10, 20);
            mutate(&mut w.tiles[i]);
            assert!(
                !place_item_tile(&mut w, 10, 19, 82, 0, 1),
                "{label} soil refuses the seed"
            );
            assert!(!w.tile(10, 19).unwrap().active(), "{label} soil writes nothing");
        }
    }

    #[test]
    fn test_DW_1_1_seed_rejected_on_actuated_soil() {
        // :45390-45392 — the soil must pass `tile2.nactive()` (`Tile.cs:240-246`:
        // active AND not inActive). `Wiring.Actuate` keeps `FLAG_ACTIVE` and sets
        // `FLAG_INACTIVE`, so an actuated soil reads `active()` but fails
        // `nactive()` — vanilla refuses the seed and nothing is written. The
        // same soil without the actuated flag accepts it.
        let mut w = soil_world(2);
        let i = w.index(10, 20);
        w.tiles[i].flags |= crate::tile::FLAG_INACTIVE;
        assert!(
            !place_item_tile(&mut w, 10, 19, 82, 0, 1),
            "actuated soil refuses the seed"
        );
        assert!(
            !w.tile(10, 19).unwrap().active(),
            "actuated soil writes nothing"
        );
        // `Wiring.ReActive` clears the actuation bit; the identical soil now plants.
        w.tiles[i].flags &= !crate::tile::FLAG_INACTIVE;
        assert!(
            place_item_tile(&mut w, 10, 19, 82, 0, 1),
            "de-actuated soil accepts the seed"
        );
        let t = *w.tile(10, 19).unwrap();
        assert!(t.active() && t.kind == 82 && t.frame_x == 0 && t.frame_y == 0);
    }

    #[test]
    fn test_DW_1_1_place_item_tile_routes_82_via_place_alch() {
        // `Player.cs:39772` excludes 82 from the TileObject route even though
        // the registry now has the def: the planter box (380) is accepted as
        // soil by `PlaceAlch` (:45395-45397) while the registry anchor list
        // (`AnchorAlternateTiles {78}`) would refuse it — the discriminator
        // proves the seed went through the legacy `PlaceAlch` branch.
        let mut w = soil_world(380);
        assert!(place_item_tile(&mut w, 10, 19, 82, 0, 1));
        assert_eq!(w.tile(10, 19).unwrap().kind, 82);
        // And a style-2 seed carries its own frames.
        let mut w2 = soil_world(0);
        assert!(place_item_tile(&mut w2, 10, 19, 82, 2, 1));
        assert_eq!(
            (
                w2.tile(10, 19).unwrap().frame_x,
                w2.tile(10, 19).unwrap().frame_y
            ),
            (36, 0)
        );
    }

    #[test]
    fn test_DW_1_1_alch_reframe_keeps_herb_frames() {
        // The post-placement 5x5 re-frame must leave the fresh herb — and any
        // sibling-stage herb — framed as written (`Main.tileFrameImportant
        // [82..84]` are boot statics, `Main.cs:7685-7687`).
        let mut w = soil_world(0); // dirt — a style-2 (Blinkroot) soil
        // A blooming sibling with explicit frames next to the planting spot.
        let i = w.index(12, 19);
        w.tiles[i] = Tile {
            kind: 84,
            frame_x: 90,
            frame_y: 0,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        assert!(place_item_tile(&mut w, 10, 19, 82, 2, 1));
        let fresh = *w.tile(10, 19).unwrap();
        assert_eq!((fresh.kind, fresh.frame_x, fresh.frame_y), (82, 36, 0));
        let sibling = *w.tile(12, 19).unwrap();
        assert_eq!(
            (sibling.kind, sibling.frame_x, sibling.frame_y),
            (84, 90, 0),
            "reframe_area must not re-frame the sibling herb"
        );
        for kind in [82u16, 83, 84] {
            assert!(w.frame_important[kind as usize], "frame_important[{kind}]");
        }
        // The paint rode over from the soil (`CopyPaintAndCoating`).
        let mut wp = soil_world(2);
        let i = wp.index(10, 20);
        wp.tiles[i].set_paint(7);
        assert!(place_item_tile(&mut wp, 10, 19, 82, 0, 1));
        assert_eq!(wp.tile(10, 19).unwrap().paint(), 7);
    }

    #[test]
    fn test_DW_1_3_planter_box_items_write_frame_y() {
        // `WorldGen.cs:60226-60230` — `frameY = 18 * style`, type 380, the
        // rest of the cell untouched (frames of a virgin cell are 0).
        for style in 0..=7i32 {
            let mut w = object_world();
            assert!(place_item_tile(&mut w, 10, 19, 380, style, 1), "style {style}");
            let t = *w.tile(10, 19).unwrap();
            assert!(t.active() && t.kind == 380, "style {style}");
            assert_eq!(t.frame_y, (18 * style) as i16, "style {style}");
            assert_eq!(t.frame_x, 0, "frameX stays as the cell had it");
            assert!(w.frame_important[380], "frame_important[380]");
        }
    }

    #[test]
    fn test_DW_1_3_planter_box_stays_out_of_the_registry() {
        // Tile 380 is not in `TileObjectData` (vanilla places it through the
        // legacy case): no registry def, and an occupied cell refuses.
        assert!(crate::tileobject::def_for_tile(380).is_none());
        let mut w = object_world();
        set_active(&mut w, 10, 19, 1);
        let before = w.revision;
        assert!(!place_item_tile(&mut w, 10, 19, 380, 0, 1));
        assert_eq!(w.revision, before, "a refused placement writes nothing");
        assert!(w.tile(10, 19).unwrap().active() && w.tile(10, 19).unwrap().kind == 1);
    }

    #[test]
    fn seed_style_out_of_range_runs_no_checks_like_the_csharp_switch() {
        // `WorldGen.cs:45393-45455` — a style outside 0-6 matches no switch
        // case, the reject flag stays false, and the herb plants unchecked
        // with `frameX = 18 * style`. Unreachable from the seed items (their
        // placeStyle is 0-6) — pinned 1:1 with the fall-through.
        let mut w = soil_world(1);
        assert!(place_item_tile(&mut w, 10, 19, 82, 9, 1));
        let t = *w.tile(10, 19).unwrap();
        assert_eq!((t.kind, t.frame_x, t.frame_y), (82, 162, 0));
    }

    #[test]
    fn test_DW_1_5_seed_rejected_on_wrong_soil_leaves_cell_untouched() {
        // Style 0 refuses sand (53) and stone (1); style 1 refuses dirt (2).
        // The cell stays empty and the world untouched (`PlaceAlch` returns
        // before any write, :45458).
        for (style, soil) in [(0i32, 53u16), (0, 1), (1, 2), (4, 2), (6, 2)] {
            let mut w = soil_world(soil);
            let before: Vec<Tile> = w.tiles.clone();
            assert!(
                !place_item_tile(&mut w, 10, 19, 82, style, 1),
                "style {style} must refuse soil {soil}"
            );
            assert_eq!(before, w.tiles, "soil {soil}: world unchanged");
        }
    }

    #[test]
    fn test_DW_1_5_lava_rules_refuse_by_style() {
        // Style 4 (waterleaf) in lava: refused, cell and liquid unchanged
        // (the DW-1.5 dirty case); style 5 (fireblossom) is the positive
        // control (covered in DW-1.1's liquid test).
        let mut w = soil_world(53);
        set_liquid(&mut w, 10, 19, 200, true);
        let before: Vec<Tile> = w.tiles.clone();
        assert!(!place_item_tile(&mut w, 10, 19, 82, 4, 1));
        assert_eq!(before, w.tiles, "world unchanged");
        assert!(w.tile(10, 19).unwrap().liquid == 200, "liquid untouched");
    }

    // ---- M1.4-TO-4 phase 2 (DW-2.1..2.5): the KillTile cascade arms --------

    /// Kills the active block at `(x, y)` — the neighbouring break whose
    /// `SquareTileFrame` pass frames the fixture object.
    fn trigger_at(w: &mut World, x: i32, y: i32) -> TileKill {
        kill_tile(w, x, y, false).unwrap()
    }

    /// A herb of `kind`/`frame_x` at (10, 19) on a `soil` floor with a stone
    /// trigger block at (11, 19) — killing the block frames the herb's cell
    /// (its 3x3 covers x 10..=12). All three alch kinds carry the
    /// frame-important boot static (`Main.cs:7685-7687`) because the
    /// Fireblossom blink rewrites the kind under the cascade's re-frame.
    /// The soil row rests on a stone row at y = 21: the pot soils 78/380 are
    /// registry kinds whose `CheckOnTable1x1` (:43534-43540) kills an
    /// unsupported pot mid-cascade — taking the herb with it (:45983-45990)
    /// — so the "survives on its own soils" scenario needs the pots
    /// supported, as a planted pot always is. The row is inert for the plain
    /// soils (nothing frames or judges y = 21).
    fn herb_world(soil: u16, kind: u16, frame_x: i16) -> World {
        let mut w = soil_world(soil);
        set_object_cell(&mut w, 10, 19, kind, frame_x, 0);
        for alch in crate::tileobject::TILE_ALCH {
            w.frame_important[*alch as usize] = true;
        }
        set_active(&mut w, 11, 19, 1);
        for x in 0..40 {
            set_active(&mut w, x, 21, 1);
        }
        w
    }

    #[test]
    fn test_DW_2_1_herb_dies_without_support() {
        // `CheckAlch` (:45983-45990): the soil below must be `nactive` —
        // breaking the soil block under the herb kills it through the
        // cascade. No drop and no pot event (the herb per-cell drops are
        // phase 3).
        let mut w = soil_world(2);
        set_object_cell(&mut w, 10, 19, 83, 0, 0);
        for alch in crate::tileobject::TILE_ALCH {
            w.frame_important[*alch as usize] = true;
        }
        let kill = kill_tile(&mut w, 10, 20, false).unwrap();
        assert_eq!(kill.kind, 2, "the dirt block is the hit cell");
        assert!(!w.tile(10, 19).unwrap().active(), "the herb died");
        assert!(kill.object_drops.is_empty());
        assert!(kill.events.is_empty());
        // A direct hit on the herb itself kills it alone.
        let mut w = soil_world(2);
        set_object_cell(&mut w, 10, 19, 84, 18, 0);
        for alch in crate::tileobject::TILE_ALCH {
            w.frame_important[*alch as usize] = true;
        }
        let kill = kill_tile(&mut w, 10, 19, false).unwrap();
        assert_eq!(kill.kind, 84);
        assert!(!w.tile(10, 19).unwrap().active());
        assert!(kill.object_drops.is_empty());
    }

    #[test]
    fn test_DW_2_1_herb_soil_sets_by_variant() {
        // `CheckAlch`'s per-variant soil sets (:45998-46086) — the same
        // families `PlaceAlch` plants on (78/380 included everywhere): a
        // mature herb survives a passing cascade on its own soils and dies
        // off them (stone 1 is in no set).
        for (style, soils) in ALCH_SOIL_TABLE {
            let frame_x = (18 * style) as i16;
            for soil in soils {
                let mut w = herb_world(*soil, 83, frame_x);
                trigger_at(&mut w, 11, 19);
                let t = w.tile(10, 19).unwrap();
                assert!(
                    t.active() && t.kind == 83 && t.frame_x == frame_x,
                    "style {style} survives on soil {soil}"
                );
            }
            let mut w = herb_world(1, 83, frame_x);
            trigger_at(&mut w, 11, 19);
            assert!(
                !w.tile(10, 19).unwrap().active(),
                "style {style} dies on stone"
            );
        }
    }

    #[test]
    fn test_DW_2_1_herb_half_brick_kills_slope_survives() {
        // :45983-45990 — the support test is `nactive` + `halfBrick` only:
        // a half-brick soil kills the herb, a sloped one does not (no slope
        // test exists in `CheckAlch`).
        let mut w = herb_world(2, 83, 0);
        let soil = w.index(10, 20);
        w.tiles[soil].flags |= crate::tile::FLAG_HALF_BRICK;
        trigger_at(&mut w, 11, 19);
        assert!(!w.tile(10, 19).unwrap().active(), "half brick kills");

        let mut w = herb_world(2, 83, 0);
        let soil = w.index(10, 20);
        w.tiles[soil].set_slope(2);
        trigger_at(&mut w, 11, 19);
        let t = w.tile(10, 19).unwrap();
        assert!(t.active() && t.kind == 83, "a sloped soil passes");
    }

    #[test]
    fn test_DW_2_1_lava_kills_by_style() {
        // Every variant except 5 dies when the herb cell itself holds lava
        // (`liquid > 0 && lava()`, :46006-46008 and peers); the Fireblossom
        // (style 5) never does — its lava handling is the blink.
        for style in 0..=6i32 {
            let soil = ALCH_SOIL_TABLE[style as usize].1[0];
            let mut w = herb_world(soil, 83, (18 * style) as i16);
            set_liquid(&mut w, 10, 19, 100, true);
            trigger_at(&mut w, 11, 19);
            assert_eq!(
                w.tile(10, 19).unwrap().active(),
                style == 5,
                "style {style} in lava"
            );
        }
        // Water is not a `CheckAlch` death (the global `WaterCheck` path is
        // out of scope): a style-0 herb in water survives the framing.
        let mut w = herb_world(2, 83, 0);
        set_liquid(&mut w, 10, 19, 200, false);
        trigger_at(&mut w, 11, 19);
        assert!(w.tile(10, 19).unwrap().active(), "water does not kill");
    }

    #[test]
    fn test_DW_2_1_fireblossom_blinks_liquid_16_17() {
        // :46052-46075 — the style-5 lava branch: `liquid > 16` turns a
        // mature 83 into the blooming 84, `liquid <= 16` turns an 84 back;
        // no death either way and the frames stay (the stage is the type).
        // The boundary pair 16/17 plus the guard cases (82 never blinks, no
        // lava no blink).
        let cases: [(u16, u8, u16); 6] = [
            (83, 17, 84), // 83 above the boundary blooms
            (84, 16, 83), // 84 at/below it reverts
            (84, 17, 84), // 84 above stays blooming
            (83, 16, 83), // 83 at/below stays mature
            (82, 17, 82), // the immature herb never blinks
            (83, 0, 83),  // no lava: the branch is skipped
        ];
        for (kind, liquid, expected) in cases {
            let mut w = herb_world(57, kind, 90); // style 5, ash soil
            set_liquid(&mut w, 10, 19, liquid, true);
            trigger_at(&mut w, 11, 19);
            let t = w.tile(10, 19).unwrap();
            assert!(t.active(), "kind {kind} liquid {liquid} survives");
            assert_eq!(t.kind, expected, "kind {kind} liquid {liquid}");
            assert_eq!((t.frame_x, t.frame_y), (90, 0), "frames untouched");
        }
    }

    #[test]
    fn herb_frame_y_is_normalized_and_out_of_range_style_unchecked() {
        // :45991 — `frameY = 0` unconditionally (a gated write in the port);
        // :45996 — a variant outside 0-6 matches no switch case and the herb
        // is judged on the support alone (the seed fall-through's mirror).
        let mut w = herb_world(2, 83, 0);
        w.tile_mut(10, 19).unwrap().frame_y = 36;
        trigger_at(&mut w, 11, 19);
        let t = w.tile(10, 19).unwrap();
        assert!(
            t.active() && (t.frame_x, t.frame_y) == (0, 0),
            "normalized"
        );

        let mut w = herb_world(1, 82, 162); // style 9 on stone
        trigger_at(&mut w, 11, 19);
        assert!(w.tile(10, 19).unwrap().active(), "no case: unchecked");
    }

    #[test]
    fn test_DW_2_1_herb_in_a_pot_dies_when_the_pot_cascade_kills() {
        // Neither `CheckOnTable1x1` (:43503-43546) nor `CheckAlch`
        // (:45972-46092) carries the `destroyObject` guard or wraps its
        // `KillTile` in the flag: breaking the floor under a potted herb
        // kills the pot through its own check, and the pot's plain `KillTile`
        // re-frames the herb cell, whose `CheckAlch` sees the vanished
        // support (78 is in every style's soil set, :45998-46086) and kills
        // it — the herb's drop roll (`KillTile_GetItemDrops` :65674-65698)
        // rides out of the nested kill as an event.
        let mut w = clay_pot_world();
        set_object_cell(&mut w, 10, 18, 83, 0, 0);
        let kill = kill_tile(&mut w, 10, 20, false).unwrap();
        assert_eq!(kill.kind, 1, "the floor block is the hit cell");
        assert_all_gone(&w, &[(10, 19), (10, 18)]);
        assert_eq!(
            kill.object_drops,
            vec![ObjectDrop {
                tile_kind: 78,
                x: 10,
                y: 19,
                item: None,
            }]
        );
        assert_eq!(
            kill.herb_events,
            vec![HerbEvent {
                kind: 83,
                style: 0,
                x: 10,
                y: 18
            }]
        );
    }

    /// The eight cells of the canonical style-0 sunflower at (10..11,
    /// 16..19) on a dirt floor.
    const SUNFLOWER_CELLS: [(i32, i32); 8] = [
        (10, 16),
        (11, 16),
        (10, 17),
        (11, 17),
        (10, 18),
        (11, 18),
        (10, 19),
        (11, 19),
    ];

    /// [`sunflower_world`] with the floor repainted to `soil` and the sheet
    /// column shifted to `style` (`frameX = dx*18 + style*36`).
    fn sunflower_world_with_soil(soil: u16, style: i32) -> World {
        let mut w = soil_world(soil);
        for dy in 0..4i32 {
            for dx in 0..2i32 {
                set_object_cell(
                    &mut w,
                    10 + dx,
                    16 + dy,
                    27,
                    (dx * 18 + style * 36) as i16,
                    (dy * 18) as i16,
                );
            }
        }
        w
    }

    fn sunflower_world() -> World {
        sunflower_world_with_soil(2, 0)
    }

    #[test]
    fn test_DW_2_2_sunflower_cascades_from_every_cell() {
        // `CheckSunflower` (:53729-53792): any of the eight cells triggers
        // the full destruction and the sunflower item drops at the check's
        // trigger cell (:53787-53791 — `Item.NewItem(…, 63)`), carried as
        // the record's `item`.
        for hit in SUNFLOWER_CELLS {
            let mut w = sunflower_world();
            let kill = kill_tile(&mut w, hit.0, hit.1, false).unwrap();
            assert_eq!(kill.kind, 27);
            assert_all_gone(&w, &SUNFLOWER_CELLS);
            let (dx, dy) = expected_drop(&SUNFLOWER_CELLS, hit, 27);
            assert_eq!(
                kill.object_drops,
                vec![ObjectDrop {
                    tile_kind: 27,
                    x: dx,
                    y: dy,
                    item: Some(63),
                }],
                "hit {hit:?}"
            );
            assert!(kill.events.is_empty());
        }
    }

    #[test]
    fn test_DW_2_2_sunflower_ground_set_and_solidtile() {
        // :53756-53770 — the ground must be `nactive`, in {2,477,492,109,60,
        // 633} AND `SolidTile`: dirt passes, stone (1) and the clay pot 78
        // are not in the sunflower set (unlike the herbs), a half brick
        // fails `SolidTile`. The frame column decode is style-agnostic
        // (`frameX / 18` mod 2 only) — the style-2 variant checks the same.
        let mut w = sunflower_world();
        set_active(&mut w, 9, 19, 1); // a stone block beside the stem
        kill_tile(&mut w, 9, 19, false).unwrap();
        for x in [10, 11] {
            assert!(w.tile(x, 19).unwrap().active(), "dirt ground holds");
        }

        for soil in [1u16, 78] {
            let mut w = sunflower_world_with_soil(soil, 0);
            set_active(&mut w, 9, 19, 1);
            kill_tile(&mut w, 9, 19, false).unwrap();
            assert!(
                !w.tile(10, 19).unwrap().active(),
                "soil {soil} refuses the sunflower"
            );
        }

        let mut w = sunflower_world();
        let soil = w.index(10, 20);
        w.tiles[soil].flags |= crate::tile::FLAG_HALF_BRICK;
        set_active(&mut w, 9, 19, 1);
        kill_tile(&mut w, 9, 19, false).unwrap();
        assert!(!w.tile(10, 19).unwrap().active(), "half brick ground kills");

        // The style-2 sheet column (frames 72/90) passes the same checks.
        let mut w = sunflower_world_with_soil(2, 2);
        set_active(&mut w, 9, 19, 1);
        kill_tile(&mut w, 9, 19, false).unwrap();
        for (x, y) in SUNFLOWER_CELLS {
            assert!(w.tile(x, y).unwrap().active(), "({x}, {y})");
        }
    }

    #[test]
    fn test_DW_2_2_support_break_drops_63() {
        // Mining the ground under one stem column: the cascade reaches the
        // sunflower through `SquareTileFrame`, all eight cells die and the
        // item-63 record spawns (per-cell drops stay empty — 27 is in the
        // empty `KillTile_GetItemDrops` group, :65759+).
        let mut w = sunflower_world();
        let kill = kill_tile(&mut w, 10, 20, false).unwrap();
        assert_eq!(kill.kind, 2, "the dirt block is the hit cell");
        assert_all_gone(&w, &SUNFLOWER_CELLS);
        assert_eq!(kill.object_drops.len(), 1);
        assert_eq!(
            (
                kill.object_drops[0].tile_kind,
                kill.object_drops[0].x,
                kill.object_drops[0].y,
                kill.object_drops[0].item
            ),
            (27, 10, 19, Some(63))
        );
    }

    /// A 2x2 pot of `style` at (10..11, 18..19) on the stone floor.
    fn pot_world(style: i32) -> World {
        let mut w = object_world();
        for dy in 0..2i32 {
            for dx in 0..2i32 {
                set_object_cell(
                    &mut w,
                    10 + dx,
                    18 + dy,
                    28,
                    (dx * 18) as i16,
                    (dy * 18 + style * 36) as i16,
                );
            }
        }
        w
    }

    /// The four cells of [`pot_world`].
    const POT_CELLS: [(i32, i32); 4] = [(10, 18), (11, 18), (10, 19), (11, 19)];

    #[test]
    fn test_DW_2_3_pot_cascades_from_every_cell() {
        // `CheckPot` (:56797-57016): any of the four cells triggers the full
        // destruction; no item record exists for tile 28 (the loot runs
        // through the pot event, :57011-57014) — `object_drops` stays empty
        // and the event carries the check's trigger cell.
        for hit in POT_CELLS {
            let mut w = pot_world(0);
            let kill = kill_tile(&mut w, hit.0, hit.1, false).unwrap();
            assert_eq!(kill.kind, 28);
            assert_all_gone(&w, &POT_CELLS);
            assert!(kill.object_drops.is_empty(), "no items for tile 28");
            let (tx, ty) = expected_trigger(&POT_CELLS, hit);
            assert_eq!(
                kill.events,
                vec![PotEvent { style: 0, x: tx, y: ty }],
                "hit {hit:?}"
            );
        }
    }

    #[test]
    fn test_DW_2_3_pot_style_decode_and_event() {
        // :56812-56817 — the style is the `frameY / 36` row (`num4`),
        // independent of the `frameX` sheet column. A support break yields
        // the same event.
        for style in [0i32, 8, 20] {
            let mut w = pot_world(style);
            let kill = kill_tile(&mut w, 10, 20, false).unwrap();
            assert_eq!(kill.kind, 1);
            assert_all_gone(&w, &POT_CELLS);
            assert_eq!(
                kill.events,
                vec![PotEvent {
                    style,
                    x: 10,
                    y: 19
                }],
                "style {style}"
            );
        }
    }

    #[test]
    fn test_DW_2_3_pot_break_sound_by_style() {
        // :56856-56867 — styles 7-9 break with sound 6, 16-24 with 4,
        // everything else (including 100) with 13.
        for (style, sound) in [
            (0i32, 13u16),
            (6, 13),
            (7, 6),
            (9, 6),
            (10, 13),
            (15, 13),
            (16, 4),
            (24, 4),
            (25, 13),
            (100, 13),
        ] {
            assert_eq!(pot_break_sound(style), sound, "style {style}");
        }
    }

    #[test]
    fn test_DW_2_3_pot_ground_solid_tile2() {
        // :56845-56849 — the ground is `SolidTile2` (:70134-70147): solid
        // and slopeless and not actuated — a platform row passes
        // (`Main.tileSolid[19] = true`, `Main.cs:7893`), a half brick or an
        // actuated floor fails.
        let mut w = pot_world(0);
        for x in 10..=11 {
            set_object_cell(&mut w, x, 20, 19, 0, 0); // platforms under both
        }
        set_active(&mut w, 9, 19, 1);
        kill_tile(&mut w, 9, 19, false).unwrap();
        for (x, y) in POT_CELLS {
            assert!(w.tile(x, y).unwrap().active(), "({x}, {y}) on platforms");
        }

        let mut w = pot_world(0);
        let soil = w.index(10, 20);
        w.tiles[soil].flags |= crate::tile::FLAG_HALF_BRICK;
        set_active(&mut w, 9, 19, 1);
        kill_tile(&mut w, 9, 19, false).unwrap();
        assert!(!w.tile(10, 19).unwrap().active(), "half brick ground kills");

        let mut w = pot_world(0);
        let floor = w.index(10, 20);
        w.tiles[floor].flags |= crate::tile::FLAG_INACTIVE;
        set_active(&mut w, 9, 19, 1);
        kill_tile(&mut w, 9, 19, false).unwrap();
        assert!(!w.tile(10, 19).unwrap().active(), "actuated ground kills");
    }

    #[test]
    fn test_DW_2_3_no_gore_items_spawn() {
        // The Gore-shard block (:56878-56953) has no port mechanism — a
        // broken pot spawns no items at all (the shard styles included) and
        // only the loot event leaves; the ANALYSIS entry lands in phase 5.
        for style in [0i32, 25, 31] {
            let mut w = pot_world(style);
            let kill = kill_tile(&mut w, 10, 20, false).unwrap();
            assert_all_gone(&w, &POT_CELLS);
            assert!(kill.object_drops.is_empty(), "style {style}");
            assert_eq!(kill.events.len(), 1, "style {style}");
        }
    }

    /// A 1x1 pile of `style` (`frameX = style * 18`, `frameY = 0`) at
    /// (10, 19) on a `soil` floor with a stone trigger block beside it.
    fn pile_1x1_world(soil: u16, style: i32) -> World {
        let mut w = soil_world(soil);
        set_object_cell(&mut w, 10, 19, 185, (style * 18) as i16, 0);
        set_active(&mut w, 11, 19, 1);
        w
    }

    #[test]
    fn test_DW_2_4_pile_1x1_ground_rules() {
        // `CheckPile` (:46875-46971): `SolidTileAllowBottomSlope` below the
        // cell (:46898-46902), then the style's ground family
        // (:46915-46969) — 36-47 snow/ice (162/224 included), 54-59 and
        // 73-76 sand; the other styles judge nothing.
        for (style, soil) in [(36i32, 147u16), (47, 161), (54, 53), (59, 53), (73, 53)] {
            let mut w = pile_1x1_world(soil, style);
            trigger_at(&mut w, 11, 19);
            assert!(
                w.tile(10, 19).unwrap().active(),
                "style {style} on soil {soil}"
            );
        }
        // Off-family grounds kill.
        for (style, soil) in [(36i32, 2u16), (47, 2), (54, 2), (76, 2)] {
            let mut w = pile_1x1_world(soil, style);
            trigger_at(&mut w, 11, 19);
            assert!(
                !w.tile(10, 19).unwrap().active(),
                "style {style} dies on dirt"
            );
        }
        // Styles outside the judged ranges are free.
        for style in [0i32, 35, 48, 60, 72, 77] {
            let mut w = pile_1x1_world(2, style);
            trigger_at(&mut w, 11, 19);
            assert!(
                w.tile(10, 19).unwrap().active(),
                "style {style} is unjudged"
            );
        }
        // A dead support kills regardless of style (:46898-46902).
        let mut w = pile_1x1_world(2, 0);
        let kill = kill_tile(&mut w, 10, 20, false).unwrap();
        assert_eq!(kill.kind, 2);
        assert!(!w.tile(10, 19).unwrap().active(), "lost support kills");
        assert!(kill.object_drops.is_empty(), "no items for tile 185");
        assert!(kill.events.is_empty());
    }

    /// A 2x1 pile of `style` (`frameY = 18`, `frameX = style*36 + dx*18`)
    /// at (10..11, 19) on a `soil` floor, with a stone trigger block at
    /// (12, 19).
    fn pile_2x1_world(soil: u16, style: i32) -> World {
        let mut w = soil_world(soil);
        set_object_cell(&mut w, 10, 19, 185, (style * 36) as i16, 18);
        set_object_cell(&mut w, 11, 19, 185, (style * 36 + 18) as i16, 18);
        set_active(&mut w, 12, 19, 1);
        w
    }

    #[test]
    fn test_DW_2_4_pile_2x1_support_and_styles() {
        // The `Check2x1` type-185 arm (:47040-47086): both columns need
        // `SolidTileAllowBottomSlope` and no Boulder, then the `frameX / 36`
        // style's family — 25-30 snow/ice, 41-46 and 62-64 sand.
        // Unjudged styles stand anywhere.
        for style in [0i32, 24, 31] {
            let mut w = pile_2x1_world(2, style);
            trigger_at(&mut w, 12, 19);
            for x in [10, 11] {
                assert!(w.tile(x, 19).unwrap().active(), "style {style}");
            }
        }
        // Snow styles on snow hold, on dirt die; sand styles mirror.
        for (style, good, bad) in [(25i32, 147u16, 2u16), (41, 53, 2), (62, 53, 2)] {
            let mut w = pile_2x1_world(good, style);
            trigger_at(&mut w, 12, 19);
            assert!(w.tile(10, 19).unwrap().active(), "style {style} on {good}");
            let mut w = pile_2x1_world(bad, style);
            trigger_at(&mut w, 12, 19);
            assert!(!w.tile(10, 19).unwrap().active(), "style {style} on {bad}");
        }
        // A Boulder under one column refuses the support (:47052-47055).
        let mut w = pile_2x1_world(2, 0);
        set_active(&mut w, 11, 20, 138);
        trigger_at(&mut w, 12, 19);
        assert!(!w.tile(10, 19).unwrap().active(), "boulder refuses");
        // A lost support column kills; no items for tile 185 (the coin-pile
        // rolls are phase-3 RNG).
        let mut w = pile_2x1_world(2, 0);
        let kill = kill_tile(&mut w, 10, 20, false).unwrap();
        assert_eq!(kill.kind, 2);
        assert!(!w.tile(10, 19).unwrap().active());
        assert!(!w.tile(11, 19).unwrap().active());
        assert!(kill.object_drops.is_empty());
        assert!(kill.events.is_empty());
    }

    #[test]
    fn test_DW_2_4_pile_2x1_grass_cover_conversion() {
        // :47041-47048 — the grass-covered variants (`frameX` 1368-1458)
        // revert (`frameX -= 1368`) when neither ground cell is dirt/grass,
        // BEFORE the style is re-judged: over stone the converted pile
        // survives with bare frames; over dirt no conversion happens and
        // the covered pile keeps its frames; over stone the covered snow
        // style (1404 → 39 → 36) converts and the stone ground kills.
        let mut w = pile_2x1_world(1, 38); // frameX 1368 over stone
        trigger_at(&mut w, 12, 19);
        assert!(w.tile(10, 19).unwrap().active(), "converted pile survives");
        assert_eq!(w.tile(10, 19).unwrap().frame_x, 0, "reverted to bare");
        assert_eq!(w.tile(11, 19).unwrap().frame_x, 18);

        let mut w = pile_2x1_world(2, 38); // over dirt: no conversion
        trigger_at(&mut w, 12, 19);
        assert!(w.tile(10, 19).unwrap().active());
        assert_eq!(w.tile(10, 19).unwrap().frame_x, 1368, "frames kept");

        let mut w = pile_2x1_world(1, 39); // 1404 → 36 → num5 = 1 (unjudged)
        trigger_at(&mut w, 12, 19);
        assert!(w.tile(10, 19).unwrap().active(), "converted style 1 passes");
        assert_eq!(w.tile(10, 19).unwrap().frame_x, 36);
    }

    /// A 3x2 pile of `kind`/`style` at (10..12, 18..19) on a `soil` floor,
    /// with a stone trigger block at (13, 19).
    fn pile_3x2_world(kind: u16, soil: u16, style: i32) -> World {
        let mut w = soil_world(soil);
        for dy in 0..2i32 {
            for dx in 0..3i32 {
                set_object_cell(
                    &mut w,
                    10 + dx,
                    18 + dy,
                    kind,
                    (dx * 18 + style * 54) as i16,
                    (dy * 18) as i16,
                );
            }
        }
        set_active(&mut w, 13, 19, 1);
        // Both pile kinds are boot-static frame-important (`Main.cs:7416`/
        // `7417`) — the 187→186 conversion rewrites the kind under the
        // cascade's re-frame, so both bits must be set.
        w.frame_important[186] = true;
        w.frame_important[187] = true;
        w
    }

    #[test]
    fn test_DW_2_4_pile_3x2_support_and_styles() {
        // `Check3x2`'s 186/187 arms (:49300-49373): `SolidTileAllowBottomSlope`
        // plus no Boulder per column, then the style's ground family — 186:
        // 26-31 snow/ice, 32-34 mud/70; 187: 0-5 mud/60/226, 6-8 ash,
        // 29-34 sand. Unjudged styles stand on any solid ground.
        for (kind, style, soil) in [
            (186u16, 0i32, 1u16),
            (186, 26, 147),
            (186, 32, 59),
            (187, 0, 59),
            (187, 6, 57),
            (187, 29, 53),
            (187, 9, 1),
        ] {
            let mut w = pile_3x2_world(kind, soil, style);
            trigger_at(&mut w, 13, 19);
            for x in 10..=12 {
                assert!(
                    w.tile(x, 18).unwrap().active(),
                    "kind {kind} style {style} on {soil}"
                );
            }
        }
        for (kind, style, soil) in [(186u16, 26i32, 2u16), (187, 0, 2), (187, 29, 2)] {
            let mut w = pile_3x2_world(kind, soil, style);
            trigger_at(&mut w, 13, 19);
            for x in 10..=12 {
                assert!(
                    !w.tile(x, 18).unwrap().active(),
                    "kind {kind} style {style} dies on {soil}"
                );
            }
        }
        // A Boulder under one column refuses the whole pile.
        for kind in [186u16, 187] {
            let mut w = pile_3x2_world(kind, 1, 0);
            set_active(&mut w, 11, 20, 138);
            trigger_at(&mut w, 13, 19);
            for x in 10..=12 {
                assert!(
                    !w.tile(x, 18).unwrap().active(),
                    "kind {kind} over a boulder dies"
                );
            }
        }
        // A broken support column kills (:49396-49399 arm); no items for
        // the piles (the empty case group, :65759+).
        let mut w = pile_3x2_world(186, 1, 0);
        let kill = kill_tile(&mut w, 10, 20, false).unwrap();
        assert_eq!(kill.kind, 1);
        for x in 10..=12 {
            for y in 18..=19 {
                assert!(!w.tile(x, y).unwrap().active());
            }
        }
        assert!(kill.object_drops.is_empty());
        assert!(kill.events.is_empty());
    }

    #[test]
    fn test_DW_2_4_187_converts_to_186_frames() {
        // :49379-49393 — an 187 whose origin `frameX` is 756..=900 (the
        // snowy styles 14-16; canonical origins are 756/810/864, the 900
        // range end is only reachable on degenerate frames) over non-dirt
        // ground converts in place: every cell becomes a 186 with
        // `frameX -= 378` and the pile survives — even with a broken cell,
        // where the kill loop's kind filter then matches nothing.
        for style in [14i32, 15, 16] {
            let mut w = pile_3x2_world(187, 1, style);
            trigger_at(&mut w, 13, 19);
            for dy in 0..2i32 {
                for dx in 0..3i32 {
                    let t = w.tile(10 + dx, 18 + dy).unwrap();
                    assert!(t.active() && t.kind == 186, "style {style}");
                    assert_eq!(t.frame_x, (dx * 18 + (style - 7) * 54) as i16);
                    assert_eq!(t.frame_y, (dy * 18) as i16, "frameY untouched");
                }
            }
        }
        // Style 13 (origin 702) is below the range: no conversion.
        let mut w = pile_3x2_world(187, 1, 13);
        trigger_at(&mut w, 13, 19);
        let t = w.tile(10, 18).unwrap();
        assert!(t.active() && t.kind == 187, "below the range: no conversion");
        assert_eq!(t.frame_x, 702);
        // Over dirt (2) the conversion refuses; style 14 has no ground case,
        // so the pile also survives as an 187.
        let mut w = pile_3x2_world(187, 2, 14);
        trigger_at(&mut w, 13, 19);
        let t = w.tile(10, 18).unwrap();
        assert!(t.active() && t.kind == 187 && t.frame_x == 756);
        // A broken cell with the conversion: the rewrite still runs (the
        // cells become 186, so this check's own kind-filtered kill loop
        // matches nothing), but the hole keeps the object broken — the next
        // framing visit re-checks the pile as a 186 and takes the
        // survivors. Nothing drops either way (the piles' case group is
        // empty).
        let mut w = pile_3x2_world(187, 1, 14);
        let gone = w.index(11, 19);
        w.tiles[gone] = Tile::default();
        let kill = trigger_at(&mut w, 13, 19);
        for (x, y) in [(10i32, 18i32), (11, 18), (12, 18), (10, 19), (11, 19), (12, 19)] {
            assert!(!w.tile(x, y).unwrap().active(), "({x}, {y}) gone");
        }
        assert!(kill.object_drops.is_empty());
    }

    #[test]
    fn test_DW_2_5_pile_on_boulder_refuses_support() {
        // `InvalidTileForPilesOrSpeleothems` (:38979-38994): an active
        // Boulder under any column sets the plan even with every other
        // support intact — the pile dies while the Boulder and the
        // neighbouring ground survive (the kill region never includes
        // supports).
        for kind in [186u16, 187] {
            let mut w = pile_3x2_world(kind, 1, 0);
            set_active(&mut w, 11, 20, 138);
            let kill = kill_tile(&mut w, 13, 19, false).unwrap();
            for x in 10..=12 {
                for y in 18..=19 {
                    assert!(!w.tile(x, y).unwrap().active(), "kind {kind}");
                }
            }
            let boulder = w.tile(11, 20).unwrap();
            assert!(boulder.active() && boulder.kind == 138, "boulder stays");
            assert!(w.tile(10, 20).unwrap().active(), "stone stays");
            assert!(kill.object_drops.is_empty());
        }
    }

    #[test]
    fn test_DW_2_5_intact_pile_survives_passing_cascade() {
        // A fully supported pile survives a cascade that merely passes by
        // (a neighbouring block breaks): the check runs and finds nothing.
        let mut w = pile_3x2_world(186, 1, 0);
        let kill = kill_tile(&mut w, 13, 19, false).unwrap();
        assert_eq!(kill.kind, 1);
        for x in 10..=12 {
            for y in 18..=19 {
                let t = w.tile(x, y).unwrap();
                assert!(t.active() && t.kind == 186, "({x}, {y}) survives");
                assert_eq!(t.frame_x, ((x - 10) * 18) as i16, "frames kept");
            }
        }
        assert!(kill.object_drops.is_empty());
        assert!(kill.events.is_empty());
    }

    #[test]
    fn test_DW_2_5_conversion_spares_neighbours() {
        // The 187→186 rewrite touches exactly the converting pile's six
        // cells: an adjacent 186 keeps its kind and frames through the same
        // cascade (the trigger block sits between the two piles).
        let mut w = pile_3x2_world(187, 1, 14);
        for dy in 0..2i32 {
            for dx in 0..3i32 {
                set_object_cell(
                    &mut w,
                    14 + dx,
                    18 + dy,
                    186,
                    (dx * 18) as i16,
                    (dy * 18) as i16,
                );
            }
        }
        trigger_at(&mut w, 13, 19);
        for dy in 0..2i32 {
            for dx in 0..3i32 {
                let t = w.tile(10 + dx, 18 + dy).unwrap();
                assert!(t.active() && t.kind == 186, "converted pile");
                assert_eq!(t.frame_x, (dx * 18 + 378) as i16);
                let n = w.tile(14 + dx, 18 + dy).unwrap();
                assert!(n.active() && n.kind == 186, "neighbour intact");
                assert_eq!(n.frame_x, (dx * 18) as i16, "neighbour frames kept");
            }
        }
    }

    #[test]
    fn style_25_table_spans_a_single_row() {
        // `Check3x2`'s `flag2` (:49223-49229): table style 25 is a 3x1 —
        // the support row is `oy + 1` and the cell region one row.
        let mut w = object_world();
        for dx in 0..3i32 {
            set_object_cell(&mut w, 10 + dx, 18, 14, (dx * 18 + 25 * 54) as i16, 0);
            set_active(&mut w, 10 + dx, 19, 1); // its own support row
        }
        set_active(&mut w, 9, 18, 1);
        kill_tile(&mut w, 9, 18, false).unwrap();
        for dx in 0..3i32 {
            assert!(
                w.tile(10 + dx, 18).unwrap().active(),
                "style-25 table supported"
            );
        }
        // Breaking its support row still kills it.
        let kill = kill_tile(&mut w, 11, 19, false).unwrap();
        assert_eq!(kill.kind, 1);
        for dx in 0..3i32 {
            assert!(!w.tile(10 + dx, 18).unwrap().active());
        }
        assert_eq!(kill.object_drops.len(), 1);
        assert_eq!(kill.object_drops[0].tile_kind, 14);
    }

    // ---- DW-2.6: the clay pot (78) — CheckOnTable1x1 -----------------------

    /// A 1x1 clay pot at (10, 19) on the stone floor — frames (0, 0), the
    /// single-cell origin (the arm reads no frames).
    fn clay_pot_world() -> World {
        let mut w = object_world();
        set_object_cell(&mut w, 10, 19, 78, 0, 0);
        w
    }

    /// Sets a trigger stone at (9, 19) and kills it: the break's
    /// `SquareTileFrame` pass frames the pot's cell (its 3x3 covers
    /// x 8..=10, y 18..=20) without touching the configured support row at
    /// (10, 20).
    fn pot_trigger(w: &mut World) {
        set_active(w, 9, 19, 1);
        trigger_at(w, 9, 19);
    }

    #[test]
    fn test_DW_2_6_pot_support_break_kills() {
        // `CheckOnTable1x1` (`WorldGen.cs:43503-43546`): mining the block
        // under the pot leaves air below — no slope, no half brick, no table
        // anchor and `SolidTileAllowBottomSlope` false (:43534-43540) — the
        // pot dies. The drop record is per-cell (`KillTile_GetItemDrops`
        // case 78 → 222, :64711-64713; resolved through `registry::tile_drop`
        // in phase 3, so `item` is `None`).
        let mut w = clay_pot_world();
        let kill = trigger_at(&mut w, 10, 20);
        assert_eq!(kill.kind, 1, "the floor block is the hit cell");
        assert_all_gone(&w, &[(10, 19)]);
        assert_eq!(
            kill.object_drops,
            vec![ObjectDrop {
                tile_kind: 78,
                x: 10,
                y: 19,
                item: None,
            }]
        );
    }

    #[test]
    fn test_DW_2_6_pot_survives_on_plain_solid_ground() {
        // :43534-43540 — the pot stands on ANY `SolidTileAllowBottomSlope`
        // below (the stone floor): a cascade pass over a neighbouring cell
        // leaves it standing.
        let mut w = clay_pot_world();
        trigger_at(&mut w, 9, 20);
        assert!(
            w.tile(10, 19).unwrap().active(),
            "pot survives on solid ground"
        );
    }

    #[test]
    fn test_DW_2_6_pot_half_brick_below_kills() {
        // :43525-43528 — the half-brick arm runs before the table anchors
        // and the type-78 arm (the C# order: topSlope → halfBrick → anchors
        // → 78).
        let mut w = clay_pot_world();
        let floor = w.index(10, 20);
        w.tiles[floor].flags |= crate::tile::FLAG_HALF_BRICK;
        trigger_at(&mut w, 9, 20);
        assert!(!w.tile(10, 19).unwrap().active(), "half brick below kills");
    }

    #[test]
    fn test_DW_2_6_pot_top_slope_below_kills_unless_platform_corner() {
        // :43510-43524 — a top slope below kills (:43519-43521 for
        // non-platforms) unless the sloped platform has its corner partner:
        // `blockType` 3 (slope 2) needs the active flat platform at `x - 1`,
        // `blockType` 2 (slope 1) at `x + 1` (:43513-43515). A bottom slope
        // is no top slope and falls through to the solid arm — the pot
        // stands (:43534-43540).
        let platform_below = |slope: u8| {
            let mut w = clay_pot_world();
            set_object_cell(&mut w, 10, 20, 19, 0, 0); // platform below the pot
            w.tile_mut(10, 20).unwrap().set_slope(slope);
            w
        };
        // Stone slope 2 — a top slope on a non-platform kills.
        let mut w = clay_pot_world();
        w.tile_mut(10, 20).unwrap().set_slope(2);
        pot_trigger(&mut w);
        assert!(!w.tile(10, 19).unwrap().active(), "top slope kills");

        // Sloped platform without the corner partner (stone at x - 1) kills.
        let mut w = platform_below(2);
        pot_trigger(&mut w);
        assert!(
            !w.tile(10, 19).unwrap().active(),
            "cornerless platform slope kills"
        );

        // Sloped platform with the flat platform at `x - 1` holds the pot.
        let mut w = platform_below(2);
        set_object_cell(&mut w, 9, 20, 19, 0, 0);
        pot_trigger(&mut w);
        assert!(
            w.tile(10, 19).unwrap().active(),
            "platform corner partner holds"
        );

        // A bottom slope (3) is not a top slope — `SolidTileAllowBottomSlope`
        // carries the pot.
        let mut w = clay_pot_world();
        w.tile_mut(10, 20).unwrap().set_slope(3);
        pot_trigger(&mut w);
        assert!(w.tile(10, 19).unwrap().active(), "bottom slope holds");
    }

    #[test]
    fn test_DW_2_6_pot_stands_on_a_platform() {
        // :43534-43540 — a platform is solidTop, so `SolidTileAllowBottomSlope`
        // carries the pot (the `AnchorValid` platform arm :51857-51863 reaches
        // the same verdict for the flat segments).
        let mut w = clay_pot_world();
        set_object_cell(&mut w, 10, 20, 19, 0, 0);
        pot_trigger(&mut w);
        assert!(w.tile(10, 19).unwrap().active(), "pot stands on a platform");
    }

    #[test]
    fn test_DW_2_6_pot_survives_on_a_table_even_actuated() {
        // :43530-43532 — the anchors survive before the type test
        // (`AnchorValid(…, AnchorType.Table)` :51841-51871 or the plain
        // `tile.active() && Main.tileTable[...]` arm). A table tile (87 —
        // `Main.tileTable` `Main.cs:8053`, not in the registry) under the pot
        // holds it, and the actuated twin still does: the plain disjunct
        // reads `active()`, not `nactive()` — the `nactive` gate (:51844)
        // only silences the anchor arms, and the disjunct keeps the pot.
        let mut w = clay_pot_world();
        set_object_cell(&mut w, 10, 20, 87, 0, 0);
        pot_trigger(&mut w);
        assert!(w.tile(10, 19).unwrap().active(), "pot stands on a table");

        let mut w = clay_pot_world();
        set_object_cell(&mut w, 10, 20, 87, 0, 0);
        let below = w.index(10, 20);
        w.tiles[below].flags |= crate::tile::FLAG_INACTIVE;
        pot_trigger(&mut w);
        assert!(
            w.tile(10, 19).unwrap().active(),
            "actuated table still holds the pot"
        );
    }

    #[test]
    fn test_DW_2_6_pot_on_a_table_dies_when_the_table_cascade_kills() {
        // `CheckOnTable1x1` has no `destroyObject` guard (:43503-43546), so
        // when the registry table's `Check3x2` kill loop (:49416-49423) takes
        // the pot's support cell, the re-framed pot is re-checked inside the
        // loop and dies — while the table's own cells stay silent (guarded).
        // Its 222 record (`KillTile_GetItemDrops` case 78, :64711-64713)
        // rides out of the nested kill after the table's own record.
        let mut w = object_world();
        // Style-0 table 3x2 at (10..12, 18..19); the pot stands on the
        // top-middle cell.
        for dy in 0..2i32 {
            for dx in 0..3i32 {
                set_object_cell(&mut w, 10 + dx, 18 + dy, 14, (dx * 18) as i16, (dy * 18) as i16);
            }
        }
        set_object_cell(&mut w, 11, 17, 78, 0, 0);
        // The hit bottom-left table cell is two rows below the pot: the pot
        // survives the first framing pass (its support is still alive) and
        // its verdict comes from inside `Check3x2`'s flag-wrapped loop.
        let kill = kill_tile(&mut w, 10, 19, false).unwrap();
        assert_eq!(kill.kind, 14);
        assert_all_gone(
            &w,
            &[
                (10, 18),
                (10, 19),
                (11, 18),
                (11, 19),
                (12, 18),
                (12, 19),
                (11, 17),
            ],
        );
        assert_eq!(
            kill.object_drops,
            vec![
                ObjectDrop {
                    tile_kind: 14,
                    x: 10,
                    y: 18,
                    item: None,
                },
                ObjectDrop {
                    tile_kind: 78,
                    x: 11,
                    y: 17,
                    item: None,
                },
            ]
        );
    }

    #[test]
    fn test_DW_2_6_pot_on_a_workbench_dies_when_the_bench_cascade_kills() {
        // The review's repro: the bench (`Check2x1`, guarded :46976) holds
        // the pot through `Main.tileTable[18]` (`Main.cs:8034`) while alive;
        // killing the far bench cell re-frames the pot inside `Check2x1`'s
        // kill loop (:47108-47115), where the ungated `CheckOnTable1x1`
        // (:43503-43546) kills it — mid-loop, with the flag still in flight.
        let mut w = object_world();
        // Work bench 2x1 at (10..11, 20); the pot on its left cell.
        set_object_cell(&mut w, 10, 20, 18, 0, 0);
        set_object_cell(&mut w, 11, 20, 18, 18, 0);
        set_object_cell(&mut w, 10, 19, 78, 0, 0);
        // Mine the bench cell that is not the pot's support: the first
        // framing pass anchors the pot (the table arm, :43530-43532), the
        // kill loop's re-frame kills it.
        let kill = kill_tile(&mut w, 11, 20, false).unwrap();
        assert_eq!(kill.kind, 18);
        assert_all_gone(&w, &[(10, 20), (11, 20), (10, 19)]);
        assert_eq!(
            kill.object_drops,
            vec![
                ObjectDrop {
                    tile_kind: 18,
                    x: 10,
                    y: 20,
                    item: None,
                },
                ObjectDrop {
                    tile_kind: 78,
                    x: 10,
                    y: 19,
                    item: None,
                },
            ]
        );
    }

    // ---- Phase 3 (M1.4-TO-4): the herb/pile loot event channels -------------

    #[test]
    fn test_DW_3_2_herb_event_records_the_killed_cell() {
        // `CheckAlch`'s kill records the per-cell herb drop as an event —
        // the kind, the `frameX / 18` style and the cell (`KillTile_DropItems`
        // rolls the app resolves, `WorldGen.cs:65674-65698`).
        let mut w = object_world();
        // Soil swap: dirt → sand (the style-2 Blinkroot set {0, 59}) is not
        // needed — style 0 also accepts 0 via {2,477,109,492}? no: dirt 2 is
        // the style-0 set, so the bare fixture stands.
        set_object_cell(&mut w, 10, 19, 83, 2 * 18, 0);
        let kill = kill_tile(&mut w, 10, 20, false).unwrap();
        assert_all_gone(&w, &[(10, 19)]);
        assert_eq!(
            kill.herb_events,
            vec![HerbEvent {
                kind: 83,
                style: 2,
                x: 10,
                y: 19
            }]
        );
        assert!(kill.object_drops.is_empty(), "no per-cell record");
    }

    #[test]
    fn test_DW_3_2_herb_event_gated_by_no_item() {
        // The herb event stands for a `KillTile_DropItems` roll — the C#
        // `noItem` gate suppresses it like the records (:63860-63864).
        let mut w = object_world();
        set_object_cell(&mut w, 10, 19, 84, 0, 0);
        let kill = kill_tile(&mut w, 10, 20, true).unwrap();
        assert_all_gone(&w, &[(10, 19)]);
        assert!(kill.herb_events.is_empty());
    }

    #[test]
    fn test_DW_3_3_pile_event_records_the_loot_block() {
        // A broken 2x1 coin pile (185, frameX 576 — the copper sub-style)
        // records the loot block with the origin's frameX, read before the
        // kills and after the grass-cover conversion (:47127-47131).
        let mut w = object_world();
        set_object_cell(&mut w, 10, 19, 185, 576, 18);
        set_object_cell(&mut w, 11, 19, 185, 594, 18);
        let kill = kill_tile(&mut w, 10, 20, false).unwrap();
        assert_all_gone(&w, &[(10, 19), (11, 19)]);
        assert_eq!(
            kill.pile_events,
            vec![PileLootEvent {
                frame_x: 576,
                x: 10,
                y: 19
            }]
        );
    }

    #[test]
    fn test_DW_3_3_bench_2x1_carries_no_pile_event() {
        // The loot block keys on `type == 185` (:47130) — the bench's 2x1
        // destruction records only its item drop.
        let mut w = object_world();
        assert!(place_item_tile(&mut w, 10, 19, 18, 0, 1));
        let kill = kill_tile(&mut w, 10, 20, false).unwrap();
        assert_eq!(kill.object_drops.len(), 1);
        assert!(kill.pile_events.is_empty());
    }
}