//! The world-growth ticks — `WorldGen.UpdateWorld` (`WorldGen.cs:71495-71632`),
//! `GrowAlch` (`:45777-45890`) and the crystal-shard arm of `hardUpdateWorld`
//! (`:69600-69640`).
//!
//! Source map (line numbers into `decompiled/Terraria/`):
//! - the rate (`GetWorldUpdateRate` `:72020-72030`): `min(dayRate, 24)`, 0
//!   when the time freezes; `dayRate` = the time rate ×5 when every player
//!   sleeps (`Main.cs:6210-6220`)
//! - the loops (`:71540-71632`): the overground
//!   `maxX·maxY · 3e-5 · rate` (×1.5 raining), the underground
//!   `maxX·maxY · 1.5e-5 · rate`; each overground pick rolls the alch-seed
//!   gate `Next(num7·100) == 0 → skip` with
//!   `num7 = Lerp(151, 151·2.8, clamp(maxTilesX/4200 − 1, 0, 1))`
//!   (`:71606-71614`)
//! - `UpdateWorld_OvergroundTile` (`:72030+`): the port's slice is the herb
//!   branch (`TileID.Sets.tileAlch` = {82, 83, 84} → `GrowAlch`); the other
//!   arms (the trees, the cacti, the shells, the bamboo) land with their
//!   follow-ups
//! - `GrowAlch` (`:45777-45890`): the herb lifecycle — the mature 84
//!   blooms into 83 by the day/night/rain/depth cue (the frameX stages
//!   0/18/36/72/90/108), the 83 flashes stage frames, the sprout 82 matures;
//!   the liquid kill keeps only the matching stage
//! - the crystal shards (`hardUpdateWorld` `:69603-69640`): the
//!   `CanGrowCrystalShards` stones {117, 116, 164, 402, 403} below the rock
//!   layer roll `Next(5)`, pick a 4-dir neighbour, need it empty, keep the
//!   density `< 2` per radius-6 box and place tile 129 with the style
//!   `Next(18)` (the big shard `18 + Next(6)` at 1/50)
//!
//! Deviations (ANALYSIS "Growth ticks (M7-GROWTH-1)"): the vines/moss and
//! the biome spread = M5-SPREAD-2's follow-up; the notTheBees/remix loop
//! variants are skipped (the port generates no such seeds); the NetMessage
//! sides are MP scope.
//!
//! M5-SPREAD-1 adds the tree + cactus arms: the sapling dispatch (`:72430-
//! 72449` — the kind-20 sapling at `Next(20)`, the default arm → `GrowTree`),
//! `IsTileTypeFitForTree` (`:29695-29710`), `GrowTree`'s gates + the frame
//! table (`:29767-29880+`, the same shape the worldgen `plant_trees` ports),
//! and `GrowCactus` (`:56575-56815`: the water-box abort
//! `cactusWaterWidth=50/Height=25/Limit=25` :86930-86932, the sand arm's
//! ≤3-cactus + >10-sand gates, the tall-arm's 11-height budget + the
//! `Next(11,13)` cap, the three-way fork/lean/highten rolls).

use crate::tile::FLAG_ACTIVE;
use crate::tile_light::FastRandom;
use crate::world::World;

/// `TileID.Sets.tileAlch` (`TileID.cs`) — the herb tiles GrowAlch runs on.
pub const TILE_ALCH: [u16; 3] = [82, 83, 84];

/// `TileID.Sets.CanGrowCrystalShards` (`TileID.cs:585`).
pub const CAN_GROW_CRYSTAL_SHARDS: [u16; 5] = [117, 116, 164, 402, 403];

/// `GetWorldUpdateRate` (`WorldGen.cs:72020-72030`): `min(dayRate, 24)`,
/// 0 while the time freezes.
pub fn world_update_rate(day_rate: i32, freeze_time: bool) -> i32 {
    if freeze_time {
        return 0;
    }
    day_rate.min(24)
}

/// The alch-seed gate (`:71606-71614`): `Next(num7·100) == 0` skips the
/// pick; `num7 = Lerp(151, 151·2.8, clamp(maxTilesX/4200 − 1, 0, 1))`.
pub fn alch_seed_skips(world: &World, rng: &mut FastRandom) -> bool {
    let t = ((world.width as f64 / 4200.0 - 1.0).clamp(0.0, 1.0));
    let num7 = 151.0 + (151.0 * 2.8 - 151.0) * t;
    rng.next((num7 * 100.0) as i32) == 0
}

/// `GrowAlch` (`WorldGen.cs:45777-45890`): the herb lifecycle. The stages
/// ride `frameX` (0/18/36/72/90/108) and the bloom target 83; the mature 84
/// blooms back; the liquid kill keeps only the matching stage.
pub fn grow_alch(world: &mut World, x: i32, y: i32, day_time: bool, raining: bool, rng: &mut FastRandom) {
    let Some(t) = world.tile(x, y) else {
        return;
    };
    if !t.active() {
        return;
    }
    // `:45784-45796`: the liquid kill — the wrong liquid for the stage
    // destroys the herb (the lava keeps only the stage-5 cactus-stem shape).
    if t.liquid > 0 {
        let stage = t.frame_x / 18;
        let lava = t.flags & crate::tile::FLAG_LIQUID_LAVA != 0;
        let water = t.liquid_kind() == crate::tile::LiquidKind::Water;
        let keep = (lava && stage == 5) || (water && (stage == 1 || stage == 4));
        if !keep {
            crate::edit::kill_tile(world, x, y, false);
            return;
        }
    }
    if t.kind == 82 {
        // The sprout matures: the special cues bloom straight into 83.
        let frame_x = world.tile(x, y).map(|t| t.frame_x).unwrap_or(0);
        let below_surface = y as f64 > world.world_surface;
        let near_underworld = y > world.height - 200;
        let bloom = if rng.next(50) == 0 && (frame_x == 108 || rng.next(2) == 0) {
            true
        } else if day_time && frame_x == 0 && rng.next(50) == 0 {
            true
        } else if !day_time && frame_x == 18 && rng.next(50) == 0 {
            true
        } else if raining && frame_x == 72 && rng.next(50) == 0 {
            true
        } else if below_surface && frame_x == 36 && rng.next(50) == 0 {
            true
        } else if near_underworld && frame_x == 90 && rng.next(50) == 0 {
            true
        } else {
            false
        };
        if bloom {
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = 83;
            }
        }
        return;
    }
    // `:45862-45883`: the stage-3 flash (`frameX == 36`, 2 of 3 rolls skip).
    if world.tile(x, y).is_some_and(|t| t.frame_x == 36) && rng.next(3) != 0 {
        let kind = world.tile(x, y).map(|t| t.kind).unwrap_or(0);
        if kind == 83 {
            if rng.next(2) == 0 {
                if let Some(t) = world.tile_mut(x, y) {
                    t.kind = 84;
                }
            }
        } else if kind == 84 || rng.next(5) == 0 {
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = 83;
            }
        }
    }
}

/// `UpdateWorld_OvergroundTile`'s port slice (`:72030+`): the herb branch.
/// The other arms (the trees, the cacti, the beach shells, the bamboo) land
/// with M7-GROWTH-2.
/// `WorldGen.IsTileTypeFitForTree` (`:29695-29710`).
pub const TREE_SOILS: [u16; 12] = [2, 23, 60, 70, 109, 147, 199, 477, 492, 633, 661, 662];

pub fn is_tile_type_fit_for_tree(kind: u16) -> bool {
    TREE_SOILS.contains(&kind)
}

/// `WorldGen.EmptyTileCheck(startX, endX, startY, endY, ignoreID = 20)`
/// (`:31453-31520`'s shape): every cell in the box is inactive (or the
/// ignored sapling kind — `TileID.Sets.CommonSapling[20]` makes flag=true).
pub fn empty_tile_check(
    world: &World,
    start_x: i32,
    end_x: i32,
    start_y: i32,
    end_y: i32,
    ignore: Option<u16>,
) -> bool {
    if start_x < 0 || end_x >= world.width as i32 || start_y < 0 || end_y >= world.height as i32 {
        return false;
    }
    for x in start_x..=end_x {
        for y in start_y..=end_y {
            let t = world.tile(x, y);
            if let Some(t) = t {
                if t.active() && Some(t.kind) != ignore {
                    return false;
                }
            }
        }
    }
    true
}

/// `WorldGen.AttemptToGrowTreeFromSapling` (:72810-72925)'s port slice: the
/// kind-20 sapling's default arm → `GrowTree` (the gem/ash/vanity saplings
/// 595/615/590+ and the palm-arm styles ride the settings profiles — the
/// port's surface gen spawns the common saplings only).
pub fn attempt_grow_tree_from_sapling(
    world: &mut World,
    x: i32,
    y: i32,
    underground: bool,
    rng: &mut FastRandom,
) -> bool {
    if underground {
        return false;
    }
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if !t.active() || t.kind != 20 {
        return false;
    }
    grow_tree(world, x, y, rng)
}

/// `WorldGen.GrowTree` (`:29767-29980`): walk through the sapling stack,
/// the liquid gate, the ground checks, the clear-box roll, then plant the
/// trunk + the branches + the forks (the frame table the worldgen
/// `plant_trees` shares — the same vanilla tables). Returns the success.
pub fn grow_tree(world: &mut World, i: i32, y: i32, rng: &mut FastRandom) -> bool {
    // The sapling stack walk (:29769).
    let mut j = y;
    while world
        .tile(i, j)
        .is_some_and(|t| t.active() && t.kind == 20)
    {
        j += 1;
    }
    // The liquid gate (:29772-29776).
    for dx in [-1i32, 0, 1] {
        if world
            .tile(i + dx, j - 1)
            .is_some_and(|t| t.liquid > 0)
        {
            return false;
        }
    }
    // The ground tile checks (:29778-29789): nactive, flat, the soil fits,
    // the wall arm (`wall == 0` — the port's surface trees stand on bare
    // ground; the wall-sets arm rides the M5-SPREAD-2 continuation), and a
    // tree-fitting soil neighbour.
    let Some(ground) = world.tile(i, j) else {
        return false;
    };
    if !ground.active() || ground.half_brick() || ground.slope() != 0 {
        return false;
    }
    if !is_tile_type_fit_for_tree(ground.kind) {
        return false;
    }
    if world.tile(i, j - 1).is_some_and(|t| t.wall != 0) {
        return false;
    }
    let neighbour_soil = [-1i32, 1].iter().any(|&dx| {
        world
            .tile(i + dx, j)
            .is_some_and(|t| t.active() && is_tile_type_fit_for_tree(t.kind))
    });
    if !neighbour_soil {
        return false;
    }
    // The height + the clearance (:29796-29816): `Next(5,17)`, the jungle
    // grass adds 5, the clear box `EmptyTileCheck(i±2, j-num3..j-1, 20)`.
    let num2 = 5 + rng.next(12);
    let mut num3 = num2 + 4;
    if ground.kind == 60 {
        num3 += 5;
    }
    if !empty_tile_check(world, i - 2, i + 2, j - num3, j - 1, Some(20)) {
        return false;
    }
    // The frame table — the same vanilla shape `plant_trees` ports
    // (surface.rs): the trunk rows num6/num5, the branch markers, the forks.
    let height = num2 as usize;
    let top = j; // the ground row; the trunk runs j-num2..j-1
    let mut rows: Vec<(i16, i16)> = vec![(0, 0); height];
    let mut markers: Vec<(i32, i32, i16, i16)> = Vec::new();
    let mut left_last = false;
    let mut right_last = false;
    for (row_i, row) in rows.iter_mut().enumerate() {
        let mut num6 = if row_i == 0 || row_i + 1 == height {
            0
        } else {
            rng.next(10) as i16
        };
        while ((num6 == 5 || num6 == 7) && left_last)
            || ((num6 == 6 || num6 == 7) && right_last)
        {
            num6 = rng.next(10) as i16;
        }
        let style = rng.next(3) as i16;
        let dy = style * 22;
        *row = match num6 {
            1 => (0, 66 + dy),
            2 => (22, dy),
            3 => (44, 66 + dy),
            4 => (22, 66 + dy),
            5 => (88, dy),
            6 => (66, 66 + dy),
            7 => (110, 66 + dy),
            _ => (0, dy),
        };
        left_last = num6 == 5 || num6 == 7;
        right_last = num6 == 6 || num6 == 7;
        let by = top - 1 - row_i as i32;
        if left_last {
            markers.push((i - 1, by, 44, 198 + rng.next(3) as i16 * 22));
        }
        if right_last {
            markers.push((i + 1, by, 66, 198 + rng.next(3) as i16 * 22));
        }
    }
    rows[height - 1] = (22, 198 + rng.next(3) as i16 * 22);
    // The forks: the base style follows the grass neighbours (:29980+).
    let grass_at = |dx: i32| {
        world
            .tile(i + dx, top)
            .is_some_and(|t| t.active() && matches!(t.kind, 2 | 23 | 60 | 109))
    };
    let left_grass = grass_at(-1);
    let right_grass = grass_at(1);
    let mut base_style = rng.next(3);
    if !left_grass && base_style == 0 {
        base_style = 2;
    }
    if !left_grass && base_style == 1 {
        base_style = 3;
    }
    if !right_grass && base_style == 0 {
        base_style = 1;
    }
    if !right_grass && base_style == 2 {
        base_style = 3;
    }
    if left_grass && !right_grass {
        base_style = 2;
    }
    if right_grass && !left_grass {
        base_style = 1;
    }
    if base_style < 3 {
        let fy = 132 + rng.next(3) as i16 * 22;
        rows[0] = match base_style {
            0 => (88, fy),
            1 => (0, fy),
            _ => (66, fy),
        };
        let mut add_fork = |side: i32, fx: i16| {
            let bx = i + side;
            if bx < 1 || bx >= world.width - 1 {
                return;
            }
            if world.tile(bx, top - 1).is_some_and(|t| t.active()) {
                return;
            }
            let fy = 132 + rng.next(3) as i16 * 22;
            let idx = world.index(bx, top - 1);
            world.tiles[idx] = crate::tile::Tile {
                kind: 5,
                frame_x: fx,
                frame_y: fy,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        };
        if base_style == 0 || base_style == 1 {
            add_fork(1, 22);
        }
        if base_style == 0 || base_style == 2 {
            add_fork(-1, 44);
        }
    }
    // Write the trunk.
    for (row_i, (fx, fy)) in rows.iter().enumerate() {
        let ty = top - 1 - row_i as i32;
        let idx = world.index(i, ty);
        world.tiles[idx] = crate::tile::Tile {
            kind: 5,
            frame_x: *fx,
            frame_y: *fy,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
        if world.frame_important.len() <= 5 {
            world.frame_important.resize(crate::tiledata::TILE_COUNT, false);
        }
        world.frame_important[5] = true;
    }
    // The branch markers sit beside the branch pieces.
    for (bx, by, fx, fy) in markers {
        if bx < 1 || bx >= world.width - 1 || by <= top - height as i32 || by >= top {
            continue;
        }
        if world.tile(bx, by).is_some_and(|t| t.active()) {
            continue;
        }
        let idx = world.index(bx, by);
        world.tiles[idx] = crate::tile::Tile {
            kind: 5,
            frame_x: fx,
            frame_y: fy,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
    }
    true
}

/// `WorldGen.GrowCactus` (:56575-56815). The water box
/// (`cactusWaterWidth=50/Height=25/Limit=25`, :86930-86932) aborts; the
/// sand arm plants a fresh cactus (the neighbours clear, ≤3 cacti in the
/// i±6 × j-3..j+1 box, >10 sand in it); the cactus arm grows the stack by
/// one with the 11-height budget and the `Next(11,13)` cap (the fork/lean
/// arms ride the same rolls).
pub fn grow_cactus(world: &mut World, i: i32, j: i32, rng: &mut FastRandom) {
    if j <= 0 {
        return;
    }
    let Some(tile) = world.tile(i, j) else {
        return;
    };
    if !tile.active() || tile.half_brick() || tile.slope() != 0 {
        return;
    }
    let is_sand = matches!(tile.kind, 53 | 234 | 116 | 112)
        || tile.kind == 234
        || tile.kind == 116
        || tile.kind == 112;
    if tile.kind != 80 && !is_sand {
        return;
    }
    if world.tile(i, j - 1).is_some_and(|t| t.liquid > 0) {
        return;
    }
    // The water box abort (:56588-56595).
    let mut num3 = 0u32;
    for k in (i - 50)..(i + 50) {
        for l in (j - 25)..(j + 25) {
            if let Some(t) = world.tile(k, l) {
                num3 += t.liquid as u32;
            }
        }
    }
    if num3 / 255 > 25 {
        return;
    }
    if is_sand {
        // The sand arm (:56598-56660): the neighbours above must be clear,
        // the box scan gates, then the plant.
        if world.tile(i, j - 1).is_some_and(|t| t.active())
            || world.tile(i - 1, j - 1).is_some_and(|t| t.active())
            || world.tile(i + 1, j - 1).is_some_and(|t| t.active())
        {
            return;
        }
        let mut num4 = 0;
        let mut num5 = 0;
        for m in (i - 6)..=(i + 6) {
            for n in (j - 3)..=(j + 1) {
                if let Some(t) = world.tile(m, n) {
                    if t.active() {
                        if t.kind == 80 {
                            num4 += 1;
                        }
                        if matches!(t.kind, 53 | 234 | 116 | 112)
                            || matches!(t.kind, 234 | 116 | 112)
                        {
                            num5 += 1;
                        }
                    }
                }
            }
        }
        if num4 >= 4 || num5 <= 10 {
            return;
        }
        let idx = world.index(i, j - 1);
        world.tiles[idx] = crate::tile::Tile {
            kind: 80,
            frame_x: 0,
            frame_y: 0,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
        if world.frame_important.len() <= 80 {
            world.frame_important.resize(crate::tiledata::TILE_COUNT, false);
        }
        world.frame_important[80] = true;
        return;
    }
    // The cactus arm (:56666-56815): walk the stack, the budget, the cap,
    // then grow by one (the fork/lean rolls land with their brushes).
    let mut num2 = j;
    while world
        .tile(i, num2)
        .is_some_and(|t| t.active() && t.kind == 80)
    {
        num2 -= 1;
        if !world
            .tile(i, num2)
            .is_some_and(|t| t.active() && t.kind == 80)
        {
            break;
        }
    }
    let num6 = j - num2; // the stack height above the soil row
    let num8 = 11 - num6;
    let mut num9 = 0;
    for num10 in (i - 2)..=(i + 2) {
        for num11 in (num2 - num8)..=(j + num6) {
            if let Some(t) = world.tile(num10, num11) {
                if t.active() && t.kind == 80 {
                    num9 += 1;
                }
            }
        }
    }
    if num9 >= 11 + rng.next(2) {
        return;
    }
    if world.tile(i, j - 1).is_none_or(|t| !t.active()) {
        let idx = world.index(i, j - 1);
        world.tiles[idx] = crate::tile::Tile {
            kind: 80,
            frame_x: 0,
            frame_y: 0,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
    }
}

/// `WorldGen.UpdateWorld_GrassGrowth` (:74269-74424), the slice: the
/// per-type table (:74284-74356 - the infection rows gated by
/// `AllowedToSpreadInfections`, the jungle/mushroom/plant rows maxValue 10),
/// the plant-above roll (:74368-74371), the 3x3 soil scan converting the
/// neighbours via [`spread_grass`] (:74381-74404).
pub fn update_world_grass_growth(
    world: &mut World,
    i: i32,
    j: i32,
    min_i: i32,
    max_i: i32,
    min_j: i32,
    max_j: i32,
    allowed_to_spread: bool,
    rng: &mut FastRandom,
) {
    if i < 10 || j < 10 || i >= world.width as i32 - 10 || j >= world.height as i32 - 10 {
        return;
    }
    let Some(t) = world.tile(i, j) else {
        return;
    };
    let (soil, soil2, grass, grass2, plant, max_value): (i16, i16, i16, i16, i16, i32) = match t.kind {
        2 => (0, -1, 2, -1, -1, 1),
        23 => (0, 59, 23, 661, 24, 2),
        199 => (0, 59, 199, 662, 201, 2),
        661 => (59, 0, 661, 23, 24, 2),
        662 => (59, 0, 662, 199, 201, 2),
        60 => (59, -1, 60, -1, 61, 10),
        70 => (59, -1, 70, -1, 71, 10),
        633 => (57, -1, 633, -1, 637, 2),
        226 => (-1, -1, 226, -1, 61, 10),
        _ => return,
    };
    if matches!(t.kind, 23 | 199 | 661 | 662) && !allowed_to_spread {
        return;
    }
    // The plant-above roll (:74368-74371).
    if plant != -1
        && min_j < world.height as i32
        && world.tile(i, min_j).is_none_or(|t| !t.active())
        && rng.next(max_value) == 0
    {
        let idx = world.index(i, min_j);
        world.tiles[idx] = crate::tile::Tile {
            kind: plant as u16,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
        if world.frame_important.len() <= plant as usize {
            world.frame_important.resize(crate::tiledata::TILE_COUNT, false);
        }
        world.frame_important[plant as usize] = true;
    }
    // The 3x3 soil scan (:74381-74404).
    if soil != -1 {
        for k in min_i..max_i {
            for l in min_j..max_j {
                if k < 10
                    || l < 10
                    || k >= world.width as i32 - 10
                    || l >= world.height as i32 - 10
                {
                    continue;
                }
                if k == i && l == j {
                    continue;
                }
                let Some(n) = world.tile(k, l) else {
                    continue;
                };
                if !n.active() {
                    continue;
                }
                if n.kind == soil as u16 {
                    spread_grass(world, k, l, soil as u16, grass as u16);
                } else if soil2 > -1 && grass2 > -1 && n.kind == soil2 as u16 {
                    spread_grass(world, k, l, soil2 as u16, grass2 as u16);
                }
            }
        }
    }
}

/// `WorldGen.SpreadGrass` (:75146-75270), the slice: the guards (the dirt
/// type match, the surface rule `j >= worldSurface && dirt == 0`, the +-1
/// solid-neighbour scan with the lava exception, the flower-27 exclusion)
/// + the write. The infection Convert()/wall arms ride the next slice.
pub fn spread_grass(world: &mut World, i: i32, j: i32, dirt: u16, grass: u16) {
    let Some(t) = world.tile(i, j) else {
        return;
    };
    if !t.active() || t.kind != dirt {
        return;
    }
    // The surface rule (:75166-75169).
    if dirt == 0 && j as f64 >= world.world_surface {
        return;
    }
    // The +-1 solid-neighbour scan with the lava exception (:75183-75197).
    let mut all_empty = true;
    let mut lava = false;
    for k in (i - 1)..=(i + 1) {
        for l in (j - 1)..=(j + 1) {
            let Some(n) = world.tile(k, l) else {
                continue;
            };
            if n.active() && crate::tiledata::TILE_SOLID.has(n.kind) {
                all_empty = false;
            }
            if n.liquid > 0 && n.liquid_kind() == crate::tile::LiquidKind::Lava {
                lava = true;
            }
        }
    }
    if all_empty && !lava {
        return;
    }
    // The flower-27 exclusions (:75204-75206).
    if matches!(grass, 23 | 661 | 199 | 662 | 109)
        && world.tile(i, j - 1).is_some_and(|t| t.kind == 27)
    {
        return;
    }
    let idx = world.index(i, j);
    world.tiles[idx] = crate::tile::Tile {
        kind: grass,
        flags: FLAG_ACTIVE,
        ..Default::default()
    };
    if world.frame_important.len() <= grass as usize {
        world.frame_important.resize(crate::tiledata::TILE_COUNT, false);
    }
    world.frame_important[grass as usize] = true;
}

/// `WorldGen.GrowSpike` (:45481-45570): the thorn bushes on the evil/jungle
/// grass. The 4-neighbour density gate, the Next(4) directional walk, the
/// landing cell empty with <=2 spike neighbours and a spike within the
/// diamond radius 7 standing on the land type, then the plant.
pub fn grow_spike(
    world: &mut World,
    i: i32,
    j: i32,
    spike_type: u16,
    land_type: u16,
    rng: &mut FastRandom,
) {
    let converted = match land_type {
        23 => 661,
        199 => 662,
        _ => 0,
    };
    let spike_neighbours = |x: i32, y: i32| -> i32 {
        [(1i32, 0), (-1, 0), (0, 1), (0, -1)]
            .iter()
            .filter(|&&(dx, dy)| {
                world
                    .tile(x + dx, y + dy)
                    .is_some_and(|t| t.active() && t.kind == spike_type)
            })
            .count() as i32
    };
    let Some(t) = world.tile(i, j) else {
        return;
    };
    let num4 = spike_neighbours(i, j);
    if num4 >= 3 && t.kind != land_type && t.kind != converted {
        return;
    }
    let (num2, num3) = match rng.next(4) {
        0 => (i, j - 1),
        1 => (i, j + 1),
        2 => (i - 1, j),
        _ => (i + 1, j),
    };
    if world.tile(num2, num3).is_some_and(|t| t.active()) {
        return;
    }
    if spike_neighbours(num2, num3) >= 2 {
        return;
    }
    let num5 = 7;
    let mut flag = false;
    'outer: for k in (num2 - num5)..(num2 + num5) {
        for l in (num3 - num5)..(num3 + num5) {
            if (k - num2).abs() * 2 + (l - num3).abs() < 9
                && world.tile(k, l).is_some_and(|t| {
                    t.active() && (t.kind == land_type || t.kind == converted)
                })
                && world
                    .tile(k, l - 1)
                    .is_some_and(|t| t.active() && t.kind == spike_type && t.liquid == 0)
            {
                flag = true;
                break 'outer;
            }
        }
    }
    if !flag {
        return;
    }
    let idx = world.index(num2, num3);
    world.tiles[idx] = crate::tile::Tile {
        kind: spike_type,
        flags: FLAG_ACTIVE,
        ..Default::default()
    };
    if world.frame_important.len() <= spike_type as usize {
        world.frame_important.resize(crate::tiledata::TILE_COUNT, false);
    }
    world.frame_important[spike_type as usize] = true;
}

pub fn update_world_overground_tile(
    world: &mut World,
    x: i32,
    y: i32,
    day_time: bool,
    raining: bool,
    rng: &mut FastRandom,
) {
    let kind = world.tile(x, y).map(|t| (t.active(), t.kind));
    let Some((active, kind)) = kind else {
        return;
    };
    if !active {
        return;
    }
    if TILE_ALCH.contains(&kind) {
        grow_alch(world, x, y, day_time, raining, rng);
        return;
    }
    // The sapling arm (:72430-72434): the kind-20 sapling at `Next(20)`,
    // the beach palm saplings 595/615 at `Next(5)` (the settings profiles
    // land with the vanity trees).
    if kind == 20 && rng.next(20) == 0 {
        attempt_grow_tree_from_sapling(world, x, y, false, rng);
        return;
    }
    if (kind == 595 || kind == 615) && rng.next(5) == 0 {
        // `AttemptToGrowTreeFromSapling`'s 595/615 arms — the settings
        // profiles (Sakura/Willow) land with the vanity trees.
        return;
    }
    // The cactus arm (:72229-72231): `Next(15)`.
    if kind == 80 && rng.next(15) == 0 {
        grow_cactus(world, x, y, rng);
        return;
    }
    // The bare-sand arm (:72282-72284) rolls `Next(300)` inside the
    // beach-distance bounds — the port has no beachDistanceYet; the
    // water-box abort inside `grow_cactus` keeps the shore in check.
    if matches!(kind, 53 | 234 | 116 | 112) && rng.next(300) == 0 {
        grow_cactus(world, x, y, rng);
        return;
    }
    // The grass-growth pass (:72357 + :73282): the spread-overground grasses
    // with the plant windows (minI/maxI = i±1, minJ/maxJ = j±1 per :72350-
    // 72356's window).
    let grass_pass = matches!(
        kind,
        2 | 23 | 199 | 60 | 70 | 109 | 226 | 633 | 661 | 662
    );
    if grass_pass {
        update_world_grass_growth(
            world,
            x,
            y,
            x - 1,
            x + 2,
            y - 1,
            y + 2,
            true,
            rng,
        );
        // The thorn arms (:72362-72370): the jungle 32 at Next(3), the evil
        // 352 at Next(3), on their grass types.
        if (kind == 23 && rng.next(3) == 0) || (kind == 60 && rng.next(3) == 0) {
            grow_spike(world, x, y, 32, 23, rng);
        }
        if kind == 199 && rng.next(3) == 0 {
            grow_spike(world, x, y, 352, 199, rng);
        }
    }
}

/// The crystal-shard arm of `hardUpdateWorld` (`:69603-69640`): the shard
/// stones below the rock layer roll `Next(5)`, the 4-dir neighbour must be
/// empty, the radius-6 density stays under 2, the style `Next(18)` (the big
/// shard `18 + Next(6)` at 1/50).
pub fn grow_crystal_shards(world: &mut World, x: i32, y: i32, rng: &mut FastRandom) {
    let Some(t) = world.tile(x, y) else {
        return;
    };
    if !t.active() || !CAN_GROW_CRYSTAL_SHARDS.contains(&t.kind) {
        return;
    }
    // `((double)j > Main.rockLayer || remixWorld)`.
    if (y as f64) <= world.rock_layer {
        return;
    }
    if rng.next(5) != 0 {
        return;
    }
    // `:69609-69625`: the 4-dir pick — 0 left, 1 right, 2/3 down (the C#'s
    // `default: num3 = ((num != 0) ? 1 : -1)` resolves to +1 for both —
    // the shards never grow upward).
    let (dx, dy) = match rng.next(4) {
        0 => (-1, 0),
        1 => (1, 0),
        _ => (0, 1),
    };
    let (nx, ny) = (x + dx, y + dy);
    if world.tile(nx, ny).is_some_and(|t| t.active()) {
        return;
    }
    // `:69629-69638`: the radius-6 density scan for the existing shards
    // (tile 129).
    let mut count = 0;
    for k in (x - 6)..=(x + 6) {
        for l in (y - 6)..=(y + 6) {
            if world
                .tile(k, l)
                .is_some_and(|t| t.active() && t.kind == 129)
            {
                count += 1;
            }
        }
    }
    if count >= 2 {
        return;
    }
    // `:69639-69643`: the style — the big shards 18..=23 at 1/50.
    let mut style = rng.next(18);
    if rng.next(50) == 0 {
        style = 18 + rng.next(6);
    }
    if let Some(t) = world.tile_mut(nx, ny) {
        t.kind = 129;
        t.frame_x = (style * 18) as i16;
        t.flags |= FLAG_ACTIVE;
    }
}

/// `UpdateWorld`'s growth loops (`:71540-71632`): the overground
/// `3e-5 · rate` picks (×1.5 raining) with the alch-seed gate, then the
/// underground `1.5e-5 · rate` picks (the port's slice runs the same herb
/// pass — the underground arms are GROWTH-2). Returns the picks made (the
/// diagnostics).
pub fn update_world(
    world: &mut World,
    day_rate: i32,
    raining: bool,
    freeze_time: bool,
    day_time: bool,
    rng: &mut FastRandom,
) -> u32 {
    let rate = world_update_rate(day_rate, freeze_time);
    if rate == 0 {
        return 0;
    }
    let mut picks = 0u32;
    let mut overground_count =
        (world.width as f64 * world.height as f64) * 3e-5 * rate as f64;
    if raining {
        overground_count *= 1.5;
    }
    for _ in 0..overground_count as i32 {
        if alch_seed_skips(world, rng) {
            continue;
        }
        let x = rng.next_range(10, world.width - 10);
        let y = rng.next_range(10, world.world_surface as i32 - 1);
        update_world_overground_tile(world, x, y, day_time, raining, rng);
        picks += 1;
    }
    let underground_count = (world.width as f64 * world.height as f64) * 1.5e-5 * rate as f64;
    for _ in 0..underground_count as i32 {
        let x = rng.next_range(10, world.width - 10);
        let y = rng.next_range(world.world_surface as i32 - 1, world.height - 20);
        // The underground arms (the vines, the moss, the spread) are
        // GROWTH-2; the herb branch keeps the parity here.
        update_world_overground_tile(world, x, y, day_time, raining, rng);
        picks += 1;
    }
    picks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::Tile;

    fn world() -> World {
        let mut w = World::new("t".into(), 4200, 400);
        w.rock_layer = 160.0;
        w
    }

    fn put(w: &mut World, x: i32, y: i32, kind: u16, frame_x: i16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            frame_x,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    #[test]
    fn the_rate_caps_and_freezes() {
        // `:72020-72030`: `min(dayRate, 24)`, the freeze → 0.
        assert_eq!(world_update_rate(1, false), 1);
        assert_eq!(world_update_rate(30, false), 24);
        assert_eq!(world_update_rate(30, true), 0);
    }

    #[test]
    fn the_alch_gate_uses_the_world_size_lerp() {
        // `:71606-71614`: `num7 = Lerp(151, 422.8, maxTilesX/4200 − 1)` —
        // the 4200-wide world sits at t = 0 → 151; a 8400-wide world at
        // t = 1 → 422.8. The gate skips on `Next(num7·100) == 0`.
        let w = world();
        let mut rng = FastRandom::new(5);
        let skips = (0..1000).filter(|_| alch_seed_skips(&w, &mut rng)).count();
        // The skip chance = 1/(151·100) ≈ 1/15100 → ~0.07/1000 rolls; the
        // 4200-world roll is nearly never skipped.
        assert!(skips <= 2, "skips {skips}");
    }

    #[test]
    fn the_herb_lifecycle_blooms_by_the_cues() {
        // The sprout 82 (the stage-0 frame) blooms in the daylight.
        let mut w = world();
        put(&mut w, 30, 90, 82, 0);
        let mut rng = FastRandom::new(1);
        let mut bloomed = false;
        for _ in 0..4000 {
            grow_alch(&mut w, 30, 90, true, false, &mut rng);
            if w.tile(30, 90).unwrap().kind == 83 {
                bloomed = true;
                break;
            }
        }
        assert!(bloomed, "the daylight cue blooms the stage-0 sprout");
        // The rain cue drives the stage-72 frame.
        let mut w = world();
        put(&mut w, 30, 90, 82, 72);
        let mut rng = FastRandom::new(2);
        let mut bloomed = false;
        for _ in 0..4000 {
            grow_alch(&mut w, 30, 90, false, true, &mut rng);
            if w.tile(30, 90).unwrap().kind == 83 {
                bloomed = true;
                break;
            }
        }
        assert!(bloomed, "the rain cue blooms the stage-72 frame");
        // The generic arm (`:45785-45800`): every frame except 108 blooms on
        // `Next(50) == 0` regardless of the cue — the cue arms are the
        // chain's fallbacks.
        let mut w = world();
        put(&mut w, 30, 90, 82, 0);
        let mut rng = FastRandom::new(3);
        let mut bloomed = false;
        for _ in 0..400 {
            grow_alch(&mut w, 30, 90, false, false, &mut rng);
            if w.tile(30, 90).unwrap().kind == 83 {
                bloomed = true;
                break;
            }
        }
        assert!(bloomed, "the generic 1/50 arm blooms any frame");
    }

    #[test]
    fn the_wrong_liquid_kills_the_herb() {
        let mut w = world();
        put(&mut w, 30, 90, 82, 0);
        let i = w.index(30, 90);
        w.tiles[i].liquid = 200;
        let mut rng = FastRandom::new(1);
        grow_alch(&mut w, 30, 90, true, false, &mut rng);
        assert!(
            w.tile(30, 90).unwrap().flags & FLAG_ACTIVE == 0,
            "the water on the stage-0 herb kills it"
        );
    }

    #[test]
    fn the_shards_grow_sparse_and_big_rarely() {
        let mut w = world();
        // The stone below the rock layer with an air pocket BELOW (the
        // shards grow down/sideways only).
        for x in 100..120 {
            for y in 200..210 {
                if !(x == 110 && y == 206) {
                    put(&mut w, x, y, 117, 0);
                }
            }
        }
        let mut rng = FastRandom::new(4);
        let mut grew = 0;
        for _ in 0..200 {
            let before = w
                .tiles
                .iter()
                .filter(|t| t.active() && t.kind == 129)
                .count();
            grow_crystal_shards(&mut w, 110, 205, &mut rng);
            let after = w
                .tiles
                .iter()
                .filter(|t| t.active() && t.kind == 129)
                .count();
            grew += (after - before) as i32;
        }
        assert!(grew > 0, "the shards grow on the stones");
        // The density cap: no 6-radius box holds 3 shards.
        let mut max_density = 0;
        for x in 100..120 {
            for y in 200..210 {
                let mut count = 0;
                for k in (x - 6)..=(x + 6) {
                    for l in (y - 6)..=(y + 6) {
                        if w.tile(k, l).is_some_and(|t| t.active() && t.kind == 129) {
                            count += 1;
                        }
                    }
                }
                max_density = max_density.max(count);
            }
        }
        assert!(max_density <= 2, "the density cap holds (max {max_density})");
    }

    #[test]
    fn test_spread_1_sapling_grows_a_tree_with_the_gates() {
        // `GrowTree` (:29767-29980): the sapling stack walk, the liquid
        // gate, the soil + neighbour checks, the clear box, then the trunk
        // plants over the sapling.
        let mut w = world();
        put(&mut w, 100, 200, 2, 0); // the grass
        put(&mut w, 101, 200, 2, 0); // the neighbour soil
        put(&mut w, 100, 199, 20, 0); // the sapling
        let mut rng = FastRandom::new(7);
        assert!(attempt_grow_tree_from_sapling(&mut w, 100, 199, false, &mut rng));
        assert!(w.tile(100, 199).is_some_and(|t| t.kind == 5), "the trunk over the sapling");
        let trunk = (0..20).filter(|&dy| {
            w.tile(100, 199 - dy).is_some_and(|t| t.active() && t.kind == 5)
        }).count();
        assert!(trunk >= 5, "the trunk height in the Next(5,17) band: {trunk}");
        // The liquid gate: a flooded neighbour refuses.
        let mut w = world();
        put(&mut w, 100, 200, 2, 0);
        put(&mut w, 101, 200, 2, 0);
        put(&mut w, 100, 199, 20, 0);
        let i = w.index(101, 199);
        w.tiles[i].liquid = 255;
        w.tiles[i].flags |= FLAG_ACTIVE;
        let mut rng = FastRandom::new(7);
        assert!(!attempt_grow_tree_from_sapling(&mut w, 100, 199, false, &mut rng));
    }

    #[test]
    fn test_spread_1_cactus_plants_on_sand_with_the_box_gates() {
        // `GrowCactus` (:56575-56660): the sand arm needs the neighbours
        // above clear, <=3 cacti in the i±6 box and >10 sand in it.
        let mut w = world();
        for dx in -8..=8 {
            put(&mut w, 100 + dx, 200, 53, 0); // the sand floor
        }
        let mut rng = FastRandom::new(7);
        grow_cactus(&mut w, 100, 200, &mut rng);
        assert!(w.tile(100, 199).is_some_and(|t| t.active() && t.kind == 80), "the cactus planted");
        // A cactus crowding (4 in the box) refuses.
        let mut w = world();
        for dx in -8..=8 {
            put(&mut w, 100 + dx, 200, 53, 0);
        }
        for dx in [-3i32, 3, -5, 5] {
            put(&mut w, 100 + dx, 199, 80, 0); // 4 existing cacti
        }
        let mut rng = FastRandom::new(7);
        grow_cactus(&mut w, 100, 200, &mut rng);
        assert!(w.tile(100, 199).is_none_or(|t| !t.active() || t.kind != 80), "the crowd gate");
    }

    #[test]
    fn test_spread_2_grass_grows_over_dirt_and_the_spikes_grow() {
        // `UpdateWorld_GrassGrowth` (:74269+): the 3x3 scan converts the
        // dirt neighbours to grass; the flower-27 exclusion; the spike arms
        // grow on the matching grass with the density gates.
        let mut w = world();
        w.world_surface = 250.0; // the surface below the cells
        put(&mut w, 100, 200, 2, 0); // the grass seed cell
        put(&mut w, 101, 200, 0, 0); // the dirt neighbour
        let mut rng = FastRandom::new(7);
        update_world_grass_growth(&mut w, 100, 200, 99, 102, 199, 202, true, &mut rng);
        assert!(
            w.tile(101, 200).is_some_and(|t| t.kind == 2),
            "the dirt converted to grass"
        );
        // The spike: the evil grass grows a 352 thorn onto the empty cell.
        let mut w = world();
        w.world_surface = 250.0; // the surface below the cells
        put(&mut w, 100, 200, 199, 0);
        put(&mut w, 101, 200, 199, 0);
        put(&mut w, 100, 199, 352, 0); // a spike on the land type
        let mut rng = FastRandom::new(7);
        let mut grew = false;
        for tick in 0..200 {
            update_world_overground_tile(&mut w, 100, 200, true, false, &mut rng);
            let grew_cell = [
                (99i32, 199),
                (100, 198),
                (101, 199),
                (100, 201), // the down walk lands too (the diamond covers it)
            ]
            .iter()
            .any(|&(cx, cy)| {
                w.tile(cx, cy).is_some_and(|t| t.active() && t.kind == 352)
            });
            if grew_cell {
                grew = true;
                break;
            }
        }
        assert!(grew, "the thorn grew eventually");
    }

    #[test]
    fn the_driver_picks_at_the_verbatim_rates() {
        // 4200×400 tiles at rate 1: the overground 3e-5·tiles = 50 picks,
        // ×1.5 raining = 75; the underground 1.5e-5·tiles = 25.
        let mut w = world();
        let mut rng = FastRandom::new(9);
        let picks = update_world(&mut w, 1, false, false, true, &mut rng);
        assert_eq!(picks, 50 + 25);
        let picks = update_world(&mut w, 1, true, false, true, &mut rng);
        assert_eq!(picks, 75 + 25);
        // The frozen time picks nothing.
        let picks = update_world(&mut w, 1, false, true, true, &mut rng);
        assert_eq!(picks, 0);
    }
}
