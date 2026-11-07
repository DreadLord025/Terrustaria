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
/// is not `noItem`-gated. `kind` disambiguates the two pile loot blocks:
/// 185's 2x1 coin/ore table (`Check2x1`, `WorldGen.cs:47130-47245`) and
/// 187's sword-in-stone roll (`Check3x2`'s `case 187`,
/// `WorldGen.cs:49828-49839`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct PileLootEvent {
    pub kind: u16,
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

// The ring walk's re-entrancy cap (`square_tile_frame` → the check → the
// sweep → the ring → `square_tile_frame` — the vanilla terminates through
// the objects dying; the cap guards the port's pathologies).
thread_local! {
    static RING_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
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
            // `noItem` gate; the pile loot blocks are `Check2x1`/`Check3x2`
            // -level `Item.NewItem` calls and are not gated (:47130-47245,
            // :49828-49839).
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
/// `CheckDoorOpen` (:86339), 13/33/49/78 → `CheckOnTable1x1` (:43503, the
/// 13/33/49/50/78/174/372/646/707 group at :85604-85614), 14/87/88/186/187 →
/// `Check3x2` (:49187, dispatch :85980-85989), 15 → `Check1x2` (:43335),
/// 18 → `Check2x1` (:46973), 21 → `CheckChest` (:57923, via
/// `TileID.Sets.BasicChest` :85616-85619), 27 → `CheckSunflower` (:85639),
/// 28 → `CheckPot` (:85643), 34 → `Check3x3` (:53059), 42 → `Check1x2Top`
/// (:46706), 79/90 → `Check4x2` (:47408), 82/83/84 → `CheckAlch` (the
/// `Main.tileAlch` default arm, :85830-85832), 93 → `Check1xX` (:37788),
/// 100/172 → `Check2x2`'s default (:48397), 101 → `Check3x4` (:50076),
/// 104 → `Check2xX` (:37993), 185 → `CheckPile` (:85275).
fn check_object(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let kind = world.tile(x, y)?.kind;
    match kind {
        10 => check_door_closed(world, x, y),
        11 => check_door_open(world, x, y),
        13 | 33 | 49 | 78 => check_on_table_1x1(world, x, y),
        29 => check_piggy(world, x, y),
        14 | 87 | 88 | 186 | 187 => check_3x2(world, x, y, kind),
        15 => check_chair(world, x, y),
        18 => check_bench(world, x, y),
        21 => check_chest(world, x, y),
        27 => check_sunflower(world, x, y),
        28 => check_pot(world, x, y),
        34 => check_chandelier(world, x, y),
        42 => check_lantern(world, x, y),
        79 | 90 => check_bed_bathtub(world, x, y),
        82 | 83 | 84 => check_alch(world, x, y),
        93 => check_lamp(world, x, y),
        100 | 172 => check_candelabra_sink(world, x, y),
        101 => check_bookcase(world, x, y),
        104 => check_clock(world, x, y),
        185 => check_pile(world, x, y),
        _ => None,
    }
}

/// `WorldGen.Check2x1`'s case-29 arm (`WorldGen.cs:47003-47013`) for the
/// piggy bank: the 2x1 pair with the second cell's `frameX == first + 18`
/// (:47000-47008), and BOTH bottom cells must hold an active table tile
/// (`Main.tileTable[type]`) without halfBrick/topSlope — the case-29 arm
/// checks `Main.tileTable` directly, no `AnchorValid` fallback (unlike the
/// case-103 arm :46988-46998). The port's placement anchor is broader
/// (`AnchorType.Table` accepts platforms, `TileObject.cs:392-409`) — the
/// C# kill is stricter than its own placement, ported verbatim.
fn check_piggy(world: &mut World, x: i32, y: i32) -> Option<CheckPlan> {
    let t = world.tile(x, y)?;
    let frame_x = t.frame_x as i32;
    let ox = x - frame_x / 18 % 2;
    let left = world.tile(ox, y);
    let right = world.tile(ox + 1, y);
    let left_x = left.map(|l| l.frame_x as i32).unwrap_or(i32::MIN);
    // :47000-47008: the pair intact = the right cell's frameX is first+18
    // and both cells are the same active kind.
    let intact = right.is_some_and(|c| c.frame_x as i32 == left_x + 18)
        && left.is_some_and(|c| c.active() && c.kind == 29)
        && right.is_some_and(|c| c.active() && c.kind == 29)
        // :47003-47013: both bottom cells on a table.
        && world
            .tile(ox, y + 1)
            .is_some_and(|b| b.active() && crate::tileobject::is_table(b.kind) && !b.half_brick() && !b.top_slope())
        && world
            .tile(ox + 1, y + 1)
            .is_some_and(|b| b.active() && crate::tileobject::is_table(b.kind) && !b.half_brick() && !b.top_slope());
    if intact {
        return None;
    }
    let region = [(ox, y), (ox + 1, y)];
    Some(CheckPlan {
        drop: Some(ObjectDrop {
            tile_kind: 29,
            x: ox,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, 29),
        pot_event: None,
        herb_event: None,
        pile_event: None,
        ring: None,
         // floats until a later break (`WorldGen.cs`'s Check2x1 tail).
    })
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
        drop: Some(ObjectDrop {
            tile_kind: kind,
            x,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, kind),
        pot_event: None,
        herb_event: None,
        pile_event: None,
        ring: None,
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
        drop: Some(ObjectDrop {
            tile_kind: kind,
            x,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, kind),
        pot_event: None,
        herb_event: None,
        pile_event: None,
        ring: None,
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
        drop: Some(ObjectDrop {
            tile_kind: kind,
            x,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, kind),
        pot_event: None,
        herb_event: None,
        pile_event: None,
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
        drop: Some(ObjectDrop {
            tile_kind: kind,
            x,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, kind),
        pot_event: None,
        herb_event: None,
        pile_event: None,
        ring: None,
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
        drop: Some(ObjectDrop {
            tile_kind: kind,
            x,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, kind),
        pot_event: None,
        herb_event: None,
        pile_event: None,
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
        drop: Some(ObjectDrop {
            tile_kind: kind,
            x,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, kind),
        pot_event: None,
        herb_event: None,
        pile_event: None,
        ring: None,
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
        drop: Some(ObjectDrop {
            tile_kind: kind,
            x,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, kind),
        pot_event: None,
        herb_event: None,
        pile_event: None,
        ring: None,
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
        ring: None,
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
        ring: None,
         // floats until a later break (`WorldGen.cs`'s check_door_open).
    })
}

/// `WorldGen.Check3x2` (`WorldGen.cs:49187-49430`) for the table family —
/// 14 tables, 87 pianos, 88 dressers — and the large piles (186/187 —
/// dispatch `WorldGen.cs:85980-85989`): origin from `frameY % 36 / 18` and
/// `frameX / 18` mod 3 (:49199-49209 — the hardcoded 36 is the `Style3x2`
/// full height; the family's styles are horizontal with `StyleWrapLimit = 0`,
/// so `frameY` only ever encodes the row), every cell must be active of
/// `kind` with the expected frames (:49219-49226), and the per-column support
/// judge branches on the kind (:49280-49373): 186/187 refuse
/// `SolidTileAllowBottomSlope` failures, Boulders
/// (`InvalidTileForPilesOrSpeleothems` :38979-38994, set `TileID.cs:504`) and
/// each style's ground family; the table keeps the default
/// `SolidTileAllowBottomSlope` arm (the trailing `else if`, :49396-49399).
/// Table style 25 spans a single row (`flag2`, :49223-49229).
/// The 187→186 conversion (:49379-49393) runs before the early return: an
/// intact snowy 187 over non-dirt ground becomes a 186 (`frameX -= 378` on
/// all six cells) and survives — the kill loop below then matches zero cells
/// of `kind`. Drop: only the table family carries one —
/// `GetItemDrop_Tables(style)` for 14 (:49434, 32x32 at the trigger cell;
/// style 0 → 32 :42194), `GetItemDrop_Pianos/Dressers(style)` for 87/88
/// (:49693-49696); the piles' case group is empty (:65759+).
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
    // The drop is the table family's alone (`GetItemDrop_Tables(style)` for
    // 14, `GetItemDrop_Pianos/Dressers(style)` for 87/88 — the slices cover
    // style 0 → 32); the piles' per-cell
    // case group is empty (:65759-65815) and `Check3x2`'s destroy switch
    // carries the pile specials — for 187 the sword-in-stone styles roll
    // the Enchanted Sword (:49828-49839), resolved app-side like the 2x1
    // coin table.
    let pile_event = if kind == 187 {
        // `int frameX = Main.tile[i, j].frameX;` (:49443) — the CHECKED
        // cell's frameX (a survivor of the pile; the dispatch never runs on
        // the dead cell). The 54-wide style stride puts every cell of a
        // style in one 54 bucket, so the 918..=970 gate selects exactly the
        // sword-in-stone style whichever cell was checked.
        Some(PileLootEvent {
            kind: 187,
            frame_x,
            x: ox,
            y: oy,
        })
    } else {
        None
    };
    Some(CheckPlan {
        drop: (kind == 14 || kind == 87 || kind == 88).then(|| ObjectDrop {
            tile_kind: kind,
            x,
            y,
            item: None,
        }),
        cells: killable_cells(world, &region, kind),
        pot_event: None,
        herb_event: None,
        pile_event,
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
        ring: None,
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
        ring: None,
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
        ring: None,
         // floats until a later break (`WorldGen.cs`'s check_chest).
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
        ring: None,
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
        ring: None,
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
            ring: None,
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
        ring: None,
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
            kind: 185,
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
        ring: None,
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

/// `WorldGen.CheckOnTable1x1` (`WorldGen.cs:43503-43546`) — the dispatch
/// group 13/33/49/50/78/174/372/646/707 (:85604-85614); the registry carries
/// 13 the placed bottle, 33/49 the candles and 78 the clay pot. The support
/// cell below decides:
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
        // :43534-43543 — the group's unconditional kill, with the type-78
        // exception: any `SolidTileAllowBottomSlope` support carries the pot.
        !(kind == 78 && solid_tile_allow_bottom_slope(world, x, y + 1))
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
        ring: None,
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
        ring: None,
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

/// `WorldGen.CanKillTile` (`WorldGen.cs:62219-62326`) — the three break
/// refusals. The mining chain zeroes the pick damage on a refusal
/// (`Player.PickTile_DetermineDamage`, `Player.cs:53830-53833`) and the
/// `KillTile` entry duplicates the same cases through
/// `CheckTileBreakability` (the 1/2 flavours, `:62377-62461`), so every
/// break source refuses, not only the pick. The C#'s
/// `out bool blockDamaged` collapses here — the 2-arg overload
/// (`:62213-62217`) discards it and no caller reads it.
///
/// Unported arms of the same C# function (M1.4-TO-6 ports the three named
/// refusals only): the tree-trunk / palm / cactus-80 anchors above
/// (`:62247-62257`, `:62271-62281`), the boulder `CheckBoulderChest` scan
/// (`:62283-62287`) and the teleporter 235 (`:62297-62310`).
///
/// `chest_holds_items` stands in for `Chest.CanDestroyChest`'s registry scan
/// (`Chest.cs:609-623`: no entry → destroyable; any slot with
/// `type > 0 && stack > 0` → refused) keyed by the object's top-left cell.
/// The app answers from its in-memory chest store; sources without one pass
/// `|_, _| false` (the store stays empty until the M3-CHEST-UI seam fills
/// it, so the refusal passes there today — same as the actuation slice in
/// `wiring.rs`).
pub fn can_kill_tile(
    world: &World,
    x: i32,
    y: i32,
    chest_holds_items: impl Fn(i32, i32) -> bool,
) -> bool {
    // `:62222-62235` — the bounds and the null/inactive guards; the port's
    // dense array folds them into the `Option`.
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if !t.active() {
        return false;
    }
    // `:62236-62239` — the dungeon echo wall refuses every break.
    if t.wall == WALL_UNBREAKABLE {
        return false;
    }
    // `:62240-62281` — the anchor above (`j >= 1` guard kept); only the
    // container-family arm is ported.
    if y >= 1 {
        if let Some(above) = world.tile(x, y - 1) {
            if above.active() {
                let kind = above.kind;
                match kind {
                    // `:62259-62270` — 21 Containers, 26 DemonAltar,
                    // 72 MushroomTrees, 77 Hellforge, 88 Dressers,
                    // 467 Containers2, 488 FallenLog (TileID.cs:671/:681/
                    // :773/:783/:805/:1563/:1605). A different kind under the
                    // object refuses: the object stands on this cell.
                    21 | 26 | 72 | 77 | 88 | 467 | 488 => {
                        if t.kind != kind {
                            return false;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    // `:62288-62296` — the locked golden door (`is_locked_door`,
    // `:69191-69197`); `OpenDoor` refuses the same frames (`:37654-37657`).
    if t.kind == 10 && is_locked_door(t) {
        return false;
    }
    // `:62311-62323` — a chest 21/467 (2 cells wide) or dresser 88 (3 wide)
    // holding items refuses; the anchor is the object's top-left cell, the
    // `Chest._chestsByCoords` key (`Chest.cs:92`).
    let wide: i32 = match t.kind {
        21 | 467 => 2,
        88 => 3,
        _ => return true,
    };
    let anchor_x = x - i32::from(t.frame_x) / 18 % wide;
    let anchor_y = y - i32::from(t.frame_y) / 18;
    !chest_holds_items(anchor_x, anchor_y)
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
    /// The bank open ran (`Player.cs:33122-33138`'s num73 switch): `bank`
    /// is the chest index -2..-5. No sound on the open path in the port's
    /// cue set (`OpenChest`'s SoundID 10/11/12 keys are absent).
    BankOpen {
        bank: i32,
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
    // The bank tiles (tiles 29/97/463/491, `Player.cs:33056-33138`): the
    // num73 switch maps 29 -> -2, 97 -> -3, 463 -> -4, 491 -> -5. The
    // vanilla opens via `OpenChest(num74, num75, num76)`; the port's banks
    // are player-bounded (`Player.bank[1..4]`, `Player.cs:1580-1591`), so
    // only the chest index moves. The vault's `disableVoidBag` gate
    // (:33127-33133) has no port mechanic yet.
    if matches!(t.kind, 29 | 97 | 463 | 491) {
        let bank = match t.kind {
            29 => -2,
            97 => -3,
            463 => -4,
            _ => -5,
        };
        return TileInteraction::BankOpen { bank };
    }
    // The port routes on the registry def's `interaction` tag (Decision #3)
    // where the C# chain tests `Main.tile[myX, myY].type` literals
    // (`Player.cs:32835`/:32857 for 10/11).
    let Some(def) = crate::tileobject::def_for_tile(t.kind) else {
        return TileInteraction::None;
    };
    match def.interaction {
        crate::tileobject::Interaction::Bank(bank) => {
            return TileInteraction::BankOpen { bank };
        }
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
                && !crate::tileobject::is_drip_tile(t.kind)
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
                && (crate::tileobject::tile_cut(t.kind)
                    || t.kind == 165
                    || crate::tileobject::is_drip_tile(t.kind))
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

    /// M1.4-TO-6: the container family above refuses the cell below
    /// (`WorldGen.cs:62259-62270`); the same kind passes (`tile.type != type`).
    #[test]
    fn the_container_family_above_refuses_can_kill_tile() {
        let mut w = ground_world();
        for (n, kind) in [21u16, 26, 72, 77, 88, 467, 488].into_iter().enumerate() {
            let x = 10 + n as i32;
            set_object_cell(&mut w, x, 19, kind, 0, 0);
            assert!(!can_kill_tile(&w, x, 20, |_, _| false), "{kind} refuses below");
        }
        // The same kind under itself does not refuse (`tile.type != type`).
        set_object_cell(&mut w, 20, 19, 1, 0, 0);
        assert!(can_kill_tile(&w, 20, 20, |_, _| false));
    }

    /// M1.4-TO-6: the locked golden door refuses (`:62288-62296` through
    /// `IsLockedDoor` `:69191-69197`); the wooden frames and the wide
    /// `frameX` column pass.
    #[test]
    fn the_locked_door_refuses_can_kill_tile() {
        let mut w = ground_world();
        set_object_cell(&mut w, 10, 20, 10, 0, 594);
        assert!(!can_kill_tile(&w, 10, 20, |_, _| false), "lock row opens at 594");
        set_object_cell(&mut w, 10, 20, 10, 0, 646);
        assert!(!can_kill_tile(&w, 10, 20, |_, _| false), "lock row ends at 646");
        set_object_cell(&mut w, 10, 20, 10, 0, 0);
        assert!(can_kill_tile(&w, 10, 20, |_, _| false), "wooden frames pass");
        set_object_cell(&mut w, 10, 20, 10, 54, 594);
        assert!(can_kill_tile(&w, 10, 20, |_, _| false), "frameX >= 54 is not the lock");
    }

    /// M1.4-TO-6: a chest 21/467 (2 cells wide) or dresser 88 (3 wide)
    /// holding items refuses through the anchor scan (`:62311-62323`,
    /// `Chest.cs:609-623`); the anchor is the top-left cell from
    /// `frameX / 18 % wide`.
    #[test]
    fn the_stocked_chest_and_dresser_refuse_can_kill_tile() {
        let mut w = ground_world();
        // The chest's right cell maps back to the top-left anchor.
        set_object_cell(&mut w, 10, 18, 21, 0, 0);
        set_object_cell(&mut w, 11, 18, 21, 18, 0);
        let holds = |x: i32, y: i32| x == 10 && y == 18;
        assert!(!can_kill_tile(&w, 10, 18, holds));
        assert!(!can_kill_tile(&w, 11, 18, holds), "the right cell maps to the anchor");
        assert!(can_kill_tile(&w, 10, 18, |_, _| false), "no registry entry passes");
        // Containers2 anchors like the chest.
        set_object_cell(&mut w, 12, 18, 467, 18, 0);
        let holds_467 = |x: i32, y: i32| x == 11 && y == 18;
        assert!(!can_kill_tile(&w, 12, 18, holds_467));
        // The dresser is 3 wide: `frameX / 18 % 3`.
        set_object_cell(&mut w, 14, 18, 88, 36, 0);
        let holds_dresser = |x: i32, y: i32| x == 12 && y == 18;
        assert!(!can_kill_tile(&w, 14, 18, holds_dresser));
        assert!(can_kill_tile(&w, 14, 18, |_, _| false));
    }

    /// M1.4-TO-6: the `CanKillTile` contract head — air, out-of-bounds,
    /// inactive and the dungeon echo wall refuse (`:62222-62239`).
    #[test]
    fn can_kill_tile_contract_head_refuses_air_inactive_and_wall_350() {
        let w = ground_world();
        assert!(!can_kill_tile(&w, 10, 5, |_, _| false), "air refuses");
        assert!(!can_kill_tile(&w, -1, 20, |_, _| false), "out of bounds refuses");
        let mut w = ground_world();
        let i = w.index(10, 20);
        w.tiles[i].flags &= !FLAG_ACTIVE;
        assert!(!can_kill_tile(&w, 10, 20, |_, _| false), "inactive refuses");
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.tiles[i].wall = WALL_UNBREAKABLE;
        assert!(!can_kill_tile(&w, 10, 20, |_, _| false), "wall 350 refuses");
    }

    /// M1.4-TO-6: the refusals gate the outer break sources only —
    /// `kill_tile`'s body (and the cascade's re-entries) stay ungated, the
    /// same split the C# draws between `CanKillTile` and the `KillTile`
    /// entry (`CheckTileBreakability`). Recorded in ANALYSIS as the port's
    /// single-choked-gate deviation.
    #[test]
    fn kill_tile_itself_stays_ungated() {
        let mut w = ground_world();
        set_object_cell(&mut w, 10, 19, 21, 0, 0);
        assert!(!can_kill_tile(&w, 10, 20, |_, _| false));
        assert_eq!(kill_tile(&mut w, 10, 20, true).map(|k| k.kind), Some(1));
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
    fn test_store_1_piggy_survives_on_a_table_and_dies_off_it() {
        // `Check2x1`'s case-29 arm (`WorldGen.cs:47003-47013`): both bottom
        // cells must hold an active table tile. A stone kill ABOVE the piggy
        // frames its cells (the cascade's 3x3 walk) without touching them.
        let piggy_world = |table: bool| -> World {
            let mut w = soil_world(2);
            if table {
                set_object_cell(&mut w, 10, 21, 14, 0, 0);
                set_object_cell(&mut w, 11, 21, 14, 18, 0);
                set_object_cell(&mut w, 12, 21, 14, 36, 0);
            }
            set_object_cell(&mut w, 10, 20, 29, 0, 0);
            set_object_cell(&mut w, 11, 20, 29, 18, 0);
            set_active(&mut w, 10, 19, 1); // the cascade trigger block
            w
        };
        // On the table: the pair survives the pass-through framing.
        let mut w = piggy_world(true);
        trigger_at(&mut w, 10, 19);
        let t = w.tile(10, 20).unwrap();
        assert!(t.active() && t.kind == 29, "the piggy sits on the table");
        // Off the table: the pass-through framing kills the pair through the
        // case-29 arm (the piggy drops item 87).
        let mut w = piggy_world(false);
        let kill = trigger_at(&mut w, 10, 19);
        assert!(
            !w.tile(10, 20).unwrap().active(),
            "no table below — the case-29 arm kills"
        );
        assert!(!kill.object_drops.is_empty(), "the piggy drops item 87");
        // The direct hit kills through the piggy's own check.
        let mut w = piggy_world(true);
        let kill = kill_tile(&mut w, 10, 20, false).unwrap();
        assert_eq!(kill.kind, 29);
        assert!(!w.tile(10, 20).unwrap().active(), "the direct hit kills");
    }

    #[test]
    fn test_store_1_bank_tiles_resolve_the_interaction() {
        // `Player.cs:33056-33138`'s num73 switch: 29 -> -2, 97 -> -3,
        // 463 -> -4, 491 -> -5. The port's interact_at returns BankOpen for
        // each literal (the defs carry Interaction::Bank too).
        for (kind, chest) in [(29, -2), (97, -3), (463, -4), (491, -5)] {
            let mut w = soil_world(2);
            set_object_cell(&mut w, 10, 20, kind, 0, 0);
            let hit = interact_at(&mut w, 10, 20, 1, &|_, _| false);
            assert!(
                matches!(hit, TileInteraction::BankOpen { bank } if bank == chest),
                "tile {kind} opens chest {chest}"
            );
        }
        // An ordinary tile: no interaction.
        let mut w = soil_world(2);
        let hit = interact_at(&mut w, 10, 20, 1, &|_, _| false);
        assert!(matches!(hit, TileInteraction::None));
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
        // `tile.active() && Main.tileTable[...]` arm). An intact registry
        // table (14, style 0) under the pot holds it, and the actuated twin
        // still does: the plain disjunct reads `active()`, not `nactive()` —
        // the `nactive` gate (:51844) only silences the anchor arms, and the
        // disjunct keeps the pot. (The pre-registry fixture — a lone tile-87
        // cell — died with the furniture registry work: 87/88 are registered
        // and `TileFrameImportant` (:85980-85989) routes them to `Check3x2`,
        // so a lone piano cell collapses under the pot exactly like the C#.)
        let table_under_pot = |w: &mut World| {
            for dy in 0..2i32 {
                for dx in 0..3i32 {
                    set_object_cell(
                        w,
                        9 + dx,
                        20 + dy,
                        14,
                        (dx * 18) as i16,
                        (dy * 18) as i16,
                    );
                }
            }
            for dx in 0..3i32 {
                set_active(w, 9 + dx, 22, 1); // the table's support row
            }
        };
        let mut w = clay_pot_world();
        table_under_pot(&mut w);
        pot_trigger(&mut w);
        assert!(w.tile(10, 19).unwrap().active(), "pot stands on a table");

        let mut w = clay_pot_world();
        table_under_pot(&mut w);
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
                kind: 185,
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
