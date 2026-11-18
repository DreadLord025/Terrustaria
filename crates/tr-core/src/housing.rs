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
    /// `WorldGen.roomTiles` — the tiles the room flood reached. The score
    /// reads it through `Housing_CheckIfInRoom` (`WorldGen.cs:6031-6039`).
    pub room_tiles: std::collections::HashSet<(i32, i32)>,
    /// `WorldGen.roomHasStinkbug` (`WorldGen.cs:4306`): the room holds a
    /// stinkbug cage (tile 630, `CheckRoom:6189-6193`).
    pub has_stinkbug: bool,
    /// `WorldGen.roomHasEchoStinkbug` (`WorldGen.cs:4308`): the room holds an
    /// echo stinkbug cage (tile 631, `CheckRoom:6194-6198`).
    pub has_echo_stinkbug: bool,
}

/// `WorldGen.IsOpenDoorAnchorFrame` (`WorldGen.cs:69171-69184`): an active
/// door (11) whose frame marks an open anchor — `frameX % 72` below 18 or at
/// least 54.
fn is_open_door_anchor_frame(tile: &crate::tile::Tile) -> bool {
    if !tile.active() || tile.kind != 11 {
        return false;
    }
    let num = tile.frame_x % 72;
    if num >= 18 {
        num >= 54
    } else {
        true
    }
}

/// `TileID.Sets.IgnoredInHouseScore` (`TileID.cs:620`): the tile kinds the
/// house score ignores.
const IGNORED_IN_HOUSE_SCORE: &[u16] = &[4, 3, 73, 82, 83, 84, 386];

/// `TileID.Sets.BasicChest` (`TileID.cs:570`): the chest kinds the score
/// counts separately (the −30 penalty each).
const BASIC_CHEST: &[u16] = &[21, 467];

/// `WorldGen.ScoreRoom_CanBeHomeSpot` (`WorldGen.cs:6021-6029`): the tile 379
/// (the "candelabra") is never a home spot.
fn can_be_home_spot(tile: &crate::tile::Tile) -> bool {
    !(tile.active() && tile.kind == 379)
}

/// `WorldGen.Housing_CheckIfInRoom` (`WorldGen.cs:6031-6039`): the room mask.
fn housing_check_if_in_room(scan: &RoomScan, x: i32, y: i32) -> bool {
    scan.room_tiles.contains(&(x, y))
}

/// `Collision.SolidTiles` (`Collision.cs:1862-1878`): any solid tile in the
/// box.
fn solid_tiles_in_box(world: &World, x1: i32, x2: i32, y1: i32, y2: i32) -> bool {
    for x in x1..=x2 {
        for y in y1..=y2 {
            if world
                .tile(x, y)
                .is_some_and(|t| t.active() && TILE_SOLID.has(t.kind))
            {
                return true;
            }
        }
    }
    false
}

/// `WorldGen.Housing_GetTestedRoomBounds` (`WorldGen.cs:5994-6020`): the
/// room's bounds grown by 5/3 + 40 and clipped to the world.
fn tested_room_bounds(world: &World, scan: &RoomScan) -> (i32, i32, i32, i32) {
    let mut start_x = scan.x1 - 5 - 1 - 40;
    let mut end_x = scan.x2 + 5 + 1 + 40;
    let mut start_y = scan.y1 - 3 - 1 - 40;
    let mut end_y = scan.y2 + 3 + 1 + 40;
    if start_x < 5 {
        start_x = 5;
    }
    if end_x >= world.width as i32 - 5 {
        end_x = world.width as i32 - 6;
    }
    if start_y < 5 {
        start_y = 5;
    }
    if end_y >= world.height as i32 - 5 {
        end_y = world.height as i32 - 6;
    }
    (start_x, end_x, start_y, end_y)
}

/// `WorldGen.CountTileTypesInArea` (`WorldGen.cs:5831-5843`): the active
/// tiles' counts per kind.
fn count_tile_types_in_area(
    world: &World,
    start_x: i32,
    end_x: i32,
    start_y: i32,
    end_y: i32,
) -> std::collections::HashMap<u16, i32> {
    let mut counts = std::collections::HashMap::new();
    for x in start_x..=end_x {
        for y in start_y..=end_y {
            if let Some(t) = world.tile(x, y) {
                if t.active() {
                    *counts.entry(t.kind).or_insert(0) += 1;
                }
            }
        }
    }
    counts
}

/// The tile scan groups `WorldGen.GetTileTypeCountByCategory` reads
/// (`WorldGen.cs:5845-5856`).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TileScanGroup {
    None,
    Corruption,
    Crimson,
    Hallow,
    TotalGoodEvil,
}

/// `WorldGen.GetTileTypeCountByCategory` (`WorldGen.cs:5845-5856`): the
/// category sums over the tile counts. The port carries the three groups the
/// house score needs (the `TotalGoodEvil` roll).
fn tile_type_count_by_category(
    counts: &std::collections::HashMap<u16, i32>,
    group: TileScanGroup,
) -> i32 {
    let count = |kind: u16| counts.get(&kind).copied().unwrap_or(0);
    match group {
        TileScanGroup::Corruption => {
            count(23) + count(24) + count(25) + count(32) + count(112) + count(163)
                + count(400) + count(398)
                - 5 * count(27)
        }
        TileScanGroup::Crimson => {
            count(199) + count(203) + count(200) + count(401) + count(399) + count(234)
                + count(352)
                - 5 * count(27)
        }
        TileScanGroup::Hallow => {
            count(109) + count(110) + count(113) + count(117) + count(116) + count(164)
                + count(403) + count(402)
        }
        TileScanGroup::TotalGoodEvil => {
            tile_type_count_by_category(counts, TileScanGroup::Hallow)
                - (tile_type_count_by_category(counts, TileScanGroup::Corruption)
                    + tile_type_count_by_category(counts, TileScanGroup::Crimson)
                    + 5 * count(27))
        }
        TileScanGroup::None => 0,
    }
}

/// `WorldGen.ScoreRoom`'s globals (`WorldGen.cs:5858-5992`): the score, the
/// best home spot and the three failure flags `MoveTownNPC`'s feedback text
/// reads (`:4803-4823`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoomScoreReport {
    /// `WorldGen.hiScore`: −1 occupied, ≤ −250 evil-rejected, 0 no
    /// candidate scored, > 0 the best spot's score.
    pub hi_score: i32,
    /// `WorldGen.bestX/bestY` — the last spot that beat the high score.
    pub best: Option<(i32, i32)>,
    /// `WorldGen.roomOccupied` (`:4300`, set at `:5870`).
    pub room_occupied: bool,
    /// `WorldGen.roomEvil` (`:4302`, set at `:5888`).
    pub room_evil: bool,
    /// `WorldGen.roomHasStandingSpace` (`:4304`, set at `:5934`).
    pub room_has_standing_space: bool,
}

/// `WorldGen.ScoreRoom` (`WorldGen.cs:5858-5992`): the room's score, its
/// best home spot and the failure flags, as one report.
///
/// The occupancy half (`ScoreRoom_IsThisRoomOccupiedBySomeone` `:5814-5829`)
/// needs the NPC list and lives with the caller (`town.rs`); its verdict
/// arrives as `occupied`, and `shared_room_x` is the C#'s
/// `WorldGen.sharedRoomX` — the home of a NPC the room may be shared with —
/// which drives the `:5936-5939` adjustment.
pub fn score_room_report(
    world: &World,
    scan: &RoomScan,
    occupied: bool,
    shared_room_x: Option<i32>,
) -> RoomScoreReport {
    let mut report = RoomScoreReport {
        hi_score: 0,
        best: None,
        room_occupied: false,
        room_evil: false,
        room_has_standing_space: false,
    };
    // `:5868-5873`: an occupied room scores −1 and returns at once.
    if occupied {
        report.room_occupied = true;
        report.hi_score = -1;
        return report;
    }
    // `:5877-5883`: the evil count over the tested area lowers the base.
    let (tx1, tx2, ty1, ty2) = tested_room_bounds(world, scan);
    let counts = count_tile_types_in_area(world, tx1 + 1, tx2 - 1, ty1 + 2, ty2 + 1);
    let mut evil = -tile_type_count_by_category(&counts, TileScanGroup::TotalGoodEvil);
    if evil < 50 {
        evil = 0;
    }
    let base = 50 - evil;
    // `:5886-5889`: the clamped evil count flags the room even when the
    // base rejects it outright below.
    if evil > 0 {
        report.room_evil = true;
    }
    if base <= -250 {
        // `:5890-5894`: the room is rejected outright, hiScore = the base.
        report.hi_score = base;
        return report;
    }
    let mut hi_score = 0;
    // `:5897-5899`: the room's own area.
    for i in (scan.x1 + 1)..scan.x2 {
        for j in (scan.y1 + 2)..(scan.y2 + 2) {
            // `:5901-5904`: the candidate must be an active, solid, framed
            // tile with solid neighbours.
            let Some(tile) = world.tile(i, j) else {
                continue;
            };
            if !tile.active()
                || !can_be_home_spot(&tile)
                || !TILE_SOLID.has(tile.kind)
                || solid_tiles_in_box(world, i - 1, i + 1, j - 3, j - 1)
            {
                continue;
            }
            let framed = |x: i32, y: i32| {
                world
                    .tile(x, y)
                    .is_some_and(|t| t.active() && TILE_SOLID.has(t.kind))
            };
            if !framed(i - 1, j) || !framed(i + 1, j) {
                continue;
            }
            let mut num = base;
            let mut on_column = 0;
            let mut chests = 0;
            // `:5910-5930`: the furniture around the spot scores the room.
            for k in (i - 2)..(i + 3) {
                for l in (j - 4)..j {
                    let Some(t) = world.tile(k, l) else {
                        continue;
                    };
                    if !t.active() || IGNORED_IN_HOUSE_SCORE.contains(&t.kind) {
                        continue;
                    }
                    if t.kind == 11 && !is_open_door_anchor_frame(&t) {
                        continue;
                    }
                    if k == i {
                        on_column += 1;
                    } else if BASIC_CHEST.contains(&t.kind) {
                        chests += 1;
                    } else if t.kind == 10 || t.kind == 388 {
                        num -= 20;
                    } else if !is_open_door_anchor_frame(&t) && t.kind != 389 {
                        if TILE_SOLID.has(t.kind) {
                            num -= 5;
                        } else {
                            num += 5;
                        }
                    } else {
                        num -= 20;
                    }
                }
            }
            // `:5932-5935`: a positive raw score means the spot can be
            // stood on — before the shared-room/chest/column adjustments.
            if num > 0 {
                report.room_has_standing_space = true;
            }
            // `:5936-5939`: a spot next to the shared room scores 1.
            if let Some(shared) = shared_room_x {
                if num >= 1 && (shared - i).abs() < 3 {
                    num = 1;
                }
            }
            // `:5940-5951`: the chest and column penalties.
            if num > 0 && chests > 0 {
                num = (num - 30 * chests).max(1);
            }
            if num > 0 && on_column > 0 {
                num = (num - 15 * on_column).max(1);
            }
            if num <= hi_score {
                continue;
            }
            // `:5953-5972`: the three cells above the spot must be in the
            // room and the room must hold the spot itself.
            let mut in_room = housing_check_if_in_room(scan, i, j);
            let mut above = [false; 3];
            for m in 1..=3 {
                let solid = world
                    .tile(i, j - m)
                    .is_some_and(|t| t.active() && TILE_SOLID.has(t.kind));
                if !solid {
                    above[(m - 1) as usize] = true;
                }
                if !housing_check_if_in_room(scan, i, j - m) {
                    above[(m - 1) as usize] = false;
                }
            }
            if above.iter().any(|v| !*v) {
                in_room = false;
            }
            if in_room && housing_check_if_in_room(scan, i, j - 1) {
                hi_score = num;
                report.best = Some((i, j));
            }
        }
    }
    report.hi_score = hi_score;
    report
}

/// `WorldGen.ScoreRoom` (`WorldGen.cs:5858-5992`) through the
/// [`score_room_report`] seam: the best home spot and its score when one
/// beat the high score, `None` otherwise. The `QuickFindHome` chain's
/// shape (`hiScore > 0` is the caller's gate).
pub fn score_room(
    world: &World,
    scan: &RoomScan,
    shared_room_x: Option<i32>,
) -> Option<(i32, i32, i32)> {
    let report = score_room_report(world, scan, false, shared_room_x);
    report.best.map(|(x, y)| (x, y, report.hi_score))
}

/// `WorldGen.RoomNeeds`' four furniture flags (`WorldGen.cs:4292-4298`:
/// `roomTorch`/`roomDoor`/`roomChair`/`roomTable`), which `MoveTownNPC`
/// reads to name what the house is missing (`:4729-4762`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RoomFurniture {
    pub chair: bool,
    pub table: bool,
    pub torch: bool,
    pub door: bool,
}

impl RoomFurniture {
    /// The `RoomNeeds` verdict (`:5721-5729`): all four present.
    pub fn complete(&self) -> bool {
        self.chair && self.table && self.torch && self.door
    }
}

/// `WorldGen.RoomNeeds` (`:5679-5730`): the house-tile table against the
/// four furniture sets, in the C#'s order — chair, table, torch, door —
/// each flag set by the first matching kind (`:5685-5720`).
pub fn room_needs_flags(house_tiles: &[u16]) -> RoomFurniture {
    let has = |types: &[u16]| house_tiles.iter().any(|k| types.contains(k));
    RoomFurniture {
        chair: has(CHAIR_TYPES),
        table: has(TABLE_TYPES),
        torch: has(TORCH_TYPES),
        door: has(DOOR_TYPES),
    }
}

/// `WorldGen.RoomNeeds` (`:5679-5730`): the house-tile table against the
/// four furniture sets — all present means a valid home.
pub fn room_needs(house_tiles: &[u16]) -> bool {
    room_needs_flags(house_tiles).complete()
}

/// `SceneMetrics.MushroomTileThreshold` (`SceneMetrics.cs:935`).
pub const MUSHROOM_TILE_THRESHOLD: i32 = 100;

/// `WorldGen.CheckSpecialTownNPCSpawningConditions` (`WorldGen.cs:4970-4998`):
/// every type but the Truffle (160) passes. The Truffle needs either its
/// unlock or a room above the surface of a world that has one
/// (`Main.NoFunctionalSurface` is `worldSurface <= 30.0`, `Main.cs:2636`),
/// and at least [`MUSHROOM_TILE_THRESHOLD`] mushroom tiles (70/71/72/528)
/// over the tested bounds.
pub fn check_special_town_npc_spawning_conditions(
    world: &World,
    scan: &RoomScan,
    net_id: i32,
    unlocked_truffle_spawn: bool,
) -> bool {
    if net_id != 160 {
        return true;
    }
    // `:4974`: `!unlockedTruffleSpawn && roomY2 > worldSurface &&
    // !NoFunctionalSurface` refuses.
    let no_functional_surface = world.world_surface <= 30.0;
    if !unlocked_truffle_spawn
        && scan.y2 as f64 > world.world_surface
        && !no_functional_surface
    {
        return false;
    }
    // `:4978-4990`: the mushroom count over the tested bounds, the same
    // window the score's evil count reads.
    let (start_x, end_x, start_y, end_y) = tested_room_bounds(world, scan);
    let mut num = 0;
    for i in (start_x + 1)..end_x {
        for j in (start_y + 2)..(end_y + 2) {
            if let Some(t) = world.tile(i, j) {
                if t.active() && matches!(t.kind, 70 | 71 | 72 | 528) {
                    num += 1;
                }
            }
        }
    }
    num >= MUSHROOM_TILE_THRESHOLD
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
        room_tiles: std::collections::HashSet::new(),
        // `StartRoomCheck:6047-6048` clears both stinkbug flags per scan.
        has_stinkbug: false,
        has_echo_stinkbug: false,
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
    // `WorldGen.roomTiles` — the flood's reached tiles (`StartRoomCheck`
    // `:6041-6117` marks every tile it scans).
    scan.room_tiles = visited;
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
            // `CheckRoom:6189-6198`: the stinkbug cages flag the room. They
            // are not solid, so the fill keeps running past them — only the
            // flags are written (the C#'s `feedback.Stinkbug`/`EchoStinkbug`
            // report but do not stop the scan).
            if t.kind == 630 {
                scan.has_stinkbug = true;
            }
            if t.kind == 631 {
                scan.has_echo_stinkbug = true;
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

/// `StartRoomCheck(x, y, NoRoomCheckFeedback.WithoutText)` the way
/// `WorldFile.FixAgainstExploits:1884`/`:1889` calls it: the room scan alone,
/// without the `RoomNeeds` furniture gate that belongs to the move-in
/// evaluation (`town::quick_find_home`'s `scan_spot`).
pub fn start_room_check(world: &World, x: i32, y: i32) -> bool {
    check_room(world, x, y).is_ok()
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
    fn the_room_score_picks_the_best_home_spot() {
        // `WorldGen.ScoreRoom` (`WorldGen.cs:5858-5992`): the best spot
        // inside the room — not the scan's centre.
        let mut w = ground();
        carve_room(&mut w, 50, 100, 12, 9, &[15, 14, 4, 10]);
        let scan = check_room(&w, 55, 105).expect("the room scans");
        let (bx, by, score) = score_room(&w, &scan, None).expect("the room scores");
        assert!(score > 0, "the score: {score}");
        assert!(bx >= scan.x1 && bx <= scan.x2, "the x is inside the room");
        assert!(by >= scan.y1 && by <= scan.y2, "the y is inside the room");
        // The spot's three cells above are inside the room (`:5953-5972`).
        for m in 1..=3 {
            assert!(
                scan.room_tiles.contains(&(bx, by - m)),
                "the cell above the spot is in the room"
            );
        }
    }

    #[test]
    fn the_stinkbug_cages_flag_the_room() {
        // `CheckRoom:6189-6198`: tile 630 sets `roomHasStinkbug`, 631 sets
        // `roomHasEchoStinkbug`; `StartRoomCheck:6047-6048` clears both per
        // scan, so a cage-free room reads false again.
        let mut w = ground();
        carve_room(&mut w, 50, 100, 12, 9, &[15, 14, 4, 10]);
        let clean = check_room(&w, 55, 105).expect("the room scans");
        assert!(!clean.has_stinkbug && !clean.has_echo_stinkbug);
        // A stinkbug cage on the floor beside the furniture.
        let i = w.index(56, 107);
        w.tiles[i] = Tile { kind: 630, frame_x: 0, frame_y: 0, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let scan = check_room(&w, 55, 105).expect("the cage does not stop the scan");
        assert!(scan.has_stinkbug, "tile 630 flags the room");
        assert!(!scan.has_echo_stinkbug);
        // Swap to the echo cage: only the echo flag.
        w.tiles[i].kind = 631;
        let scan = check_room(&w, 55, 105).expect("the echo cage scans");
        assert!(!scan.has_stinkbug, "the flag is per-scan, not sticky");
        assert!(scan.has_echo_stinkbug, "tile 631 flags the echo room");
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
    fn start_room_check_is_the_scan_without_the_furniture_gate() {
        // `FixAgainstExploits:1884` calls `StartRoomCheck` alone: an
        // enclosed but unfurnished room still passes (the furniture gate is
        // `RoomNeeds`, the move-in evaluation's concern).
        let mut w = ground();
        carve_room(&mut w, 50, 100, 12, 9, &[15, 14, 4, 10]);
        assert!(start_room_check(&w, 55, 105));
        assert!(room_needs(&check_room(&w, 55, 105).unwrap().house_tiles));
        let mut bare = ground();
        carve_room(&mut bare, 50, 100, 12, 9, &[]);
        assert!(start_room_check(&bare, 55, 105), "no furniture needed");
        assert!(!room_needs(&check_room(&bare, 55, 105).unwrap().house_tiles));
        // The failures the re-validation reacts to.
        assert!(!start_room_check(&w, 5, 105), "too close to the world edge");
        assert!(!start_room_check(&w, 30, 130), "a hole in the wall");
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

    #[test]
    fn the_score_report_flags_the_failure_reasons() {
        // `ScoreRoom`'s globals (`WorldGen.cs:5864-5894`, `:5932-5935`):
        // a good room reports standing space and no failure flags; an
        // occupied room scores −1 with `roomOccupied`; an evil-flooded
        // room flags `roomEvil` and the ≤ −250 base rejects outright.
        let mut w = ground();
        carve_room(&mut w, 50, 100, 12, 9, &[15, 14, 4, 10]);
        let scan = check_room(&w, 55, 105).unwrap();
        let good = score_room_report(&w, &scan, false, None);
        assert!(good.hi_score > 0);
        assert!(good.best.is_some());
        assert!(good.room_has_standing_space);
        assert!(!good.room_occupied && !good.room_evil);
        // The occupied arm returns before any tile is read (`:5868-5873`).
        let occupied = score_room_report(&w, &scan, true, None);
        assert_eq!(occupied.hi_score, -1);
        assert!(occupied.room_occupied);
        assert!(occupied.best.is_none());
        assert!(!occupied.room_has_standing_space);
        // Flood the tested area with corruption: evil ≥ 300 rejects the
        // base outright (`:5890-5894`) and flags the room (`:5886-5889`).
        let (tx1, tx2, ty1, ty2) = tested_room_bounds(&w, &scan);
        let mut placed = 0;
        for x in (tx1 + 1)..tx2 {
            for y in (ty1 + 2)..(ty2 + 1) {
                if placed >= 320 {
                    break;
                }
                let i = w.index(x, y);
                if !w.tiles[i].active() {
                    w.tiles[i] = Tile { kind: 23, ..Default::default() };
                    w.tiles[i].flags |= FLAG_ACTIVE;
                    placed += 1;
                }
            }
        }
        let evil = score_room_report(&w, &scan, false, None);
        assert!(evil.room_evil, "the clamped evil count flags the room");
        assert!(evil.hi_score <= -250, "hiScore carries the base: {}", evil.hi_score);
        assert_eq!(evil.hi_score, 50 - 320);
        assert!(evil.best.is_none());
    }

    #[test]
    fn the_room_needs_flags_name_the_missing_pieces() {
        // `RoomNeeds` (`WorldGen.cs:5679-5730`): each flag is its own set's
        // verdict; `MoveTownNPC` (`:4732-4753`) lists the missing ones in
        // the torch/door/table/chair order.
        let flags = room_needs_flags(&[15, 14, 4, 10]);
        assert!(flags.complete());
        assert_eq!(
            flags,
            RoomFurniture { chair: true, table: true, torch: true, door: true }
        );
        let missing = room_needs_flags(&[14, 10]); // chair + torch gone
        assert!(!missing.complete());
        assert!(!missing.chair && !missing.torch);
        assert!(missing.table && missing.door);
        assert_eq!(room_needs_flags(&[]), RoomFurniture::default());
    }

    #[test]
    fn the_special_conditions_gate_only_the_truffle() {
        // `CheckSpecialTownNPCSpawningConditions` (`WorldGen.cs:4970-4998`):
        // every non-160 type passes; the Truffle needs the mushroom count.
        let mut w = ground();
        w.world_surface = 60.0;
        carve_room(&mut w, 50, 40, 12, 9, &[15, 14, 4, 10]);
        let scan = check_room(&w, 55, 45).unwrap();
        assert!(check_special_town_npc_spawning_conditions(&w, &scan, 22, false));
        // Above the surface, no unlock, no mushrooms: refused (`:4974`).
        assert!(!check_special_town_npc_spawning_conditions(&w, &scan, 160, false));
        // The unlock lifts the surface gate but not the mushroom count.
        assert!(!check_special_town_npc_spawning_conditions(&w, &scan, 160, true));
        // 100 glowing mushrooms in the tested area open the gate
        // (`SceneMetrics.MushroomTileThreshold`, `:4991-4995`).
        let (tx1, tx2, ty1, ty2) = tested_room_bounds(&w, &scan);
        let mut placed = 0;
        'outer: for x in (tx1 + 1)..tx2 {
            for y in (ty1 + 2)..(ty2 + 2) {
                let i = w.index(x, y);
                if !w.tiles[i].active() {
                    w.tiles[i] = Tile { kind: 71, ..Default::default() };
                    w.tiles[i].flags |= FLAG_ACTIVE;
                    placed += 1;
                    if placed == MUSHROOM_TILE_THRESHOLD {
                        break 'outer;
                    }
                }
            }
        }
        assert!(check_special_town_npc_spawning_conditions(&w, &scan, 160, true));
        // Below the surface the unlock is not needed (`roomY2 <= surface`).
        let mut deep = ground();
        deep.world_surface = 60.0;
        carve_room(&mut deep, 50, 100, 12, 9, &[15, 14, 4, 10]);
        let deep_scan = check_room(&deep, 55, 105).unwrap();
        assert!(!check_special_town_npc_spawning_conditions(&deep, &deep_scan, 160, false));
        // A world with no functional surface (worldSurface <= 30,
        // `Main.cs:2636`) skips the surface gate entirely: with the
        // mushrooms present the Truffle passes without the unlock.
        let mut tiny = ground();
        tiny.world_surface = 30.0;
        carve_room(&mut tiny, 50, 100, 12, 9, &[15, 14, 4, 10]);
        let tiny_scan = check_room(&tiny, 55, 105).unwrap();
        let (tx1, tx2, ty1, ty2) = tested_room_bounds(&tiny, &tiny_scan);
        let mut placed = 0;
        'tiny: for x in (tx1 + 1)..tx2 {
            for y in (ty1 + 2)..(ty2 + 2) {
                let i = tiny.index(x, y);
                if !tiny.tiles[i].active() {
                    tiny.tiles[i] = Tile { kind: 71, ..Default::default() };
                    tiny.tiles[i].flags |= FLAG_ACTIVE;
                    placed += 1;
                    if placed == MUSHROOM_TILE_THRESHOLD {
                        break 'tiny;
                    }
                }
            }
        }
        assert!(check_special_town_npc_spawning_conditions(&tiny, &tiny_scan, 160, false));
    }
}
