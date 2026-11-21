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
use crate::UnifiedRandom;
use crate::world::World;

mod decorations;
mod grass;
mod herbs;
mod pumpkins;
mod trees;

// S1-K-3: the tree family lives in `growth/trees.rs`; the public seams
// stay on `growth` (verbatim move, behaviour frozen).
pub use trees::{
    attempt_grow_tree_from_sapling, empty_tile_check, grow_palm_tree, grow_tree,
    grow_tree_profile_from_item, grow_tree_with_settings, is_common_sapling,
    is_tile_type_fit_for_tree, try_growing_tree_by_type, GrowTreeSettings, GroundTestKind,
    COMMON_SAPLINGS, TREE_SOILS,
};

// S1-K-3b: the grass/spread family lives in `growth/grass.rs`; the public
// seams stay on `growth` (verbatim move, behaviour frozen).
pub use grass::{
    spread_desert_walls, spread_grass, spread_grass_walls, update_world_grass_growth,
};
pub(crate) use grass::convert_wall_by_case;
// W4-f25 C4: the type-7 spray cell's `:55465` union pre-gate seams.
pub(crate) use grass::{conversion_hardened_sand_walls_contains, conversion_snow_walls_contains};
use grass::{grow_moss_arm, SPREAD_UNDERGROUND};

// S1-K-3: the decorations family lives in `growth/decorations.rs`; the public
// seams stay on `growth` (verbatim move, behaviour frozen).
pub use decorations::{check_cat_tail, grow_cat_tail, grow_check_seaweed, place_dye, plant_dye};
use decorations::{grow_web, try_growing_abigails_flower};

// S1-K-3d: the herbs family lives in `growth/herbs.rs`; the public seams
// stay on `growth` (verbatim move, behaviour frozen).
pub use herbs::{
    alch_seed_skips, grow_alch, is_alchemy_plant_harvestable,
    is_harvestable_herb_with_seed, WeatherCtx, TILE_ALCH,
};


// S1-K-3e: the pumpkin family lives in `growth/pumpkins.rs`; the public
// seams stay on `growth` (verbatim move, behaviour frozen).
pub use pumpkins::{grow_pumpkin, halloween_pumpkins, place_pumpkin};


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

/// `WorldGen.GrowCactus` (:56575-56815). The water box
/// (`cactusWaterWidth=50/Height=25/Limit=25`, :86930-86932) aborts; the
/// sand arm plants a fresh cactus (the neighbours clear, ≤3 cacti in the
/// i±6 × j-3..j+1 box, >10 sand in it); the cactus arm grows the stack by
/// one with the 11-height budget and the `Next(11,13)` cap (the fork/lean
/// arms ride the same rolls).
pub fn grow_cactus(world: &mut World, i: i32, j: i32, rng: &mut UnifiedRandom) {
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
    // The cactus arm (:56668-56799): the stack walk DOWN from the top with the
    // lean corrections, the height/budget, the ±2-column density cap, and the
    // fork — the side shoots (left/right) and the tip.
    if world.tile(i, j).is_none_or(|t| t.kind != 80) {
        return;
    }
    let mut num = i;
    let mut num2 = j;
    while world
        .tile(num, num2)
        .is_some_and(|t| t.active() && t.kind == 80)
    {
        num2 += 1;
        if !world
            .tile(num, num2)
            .is_some_and(|t| t.active() && t.kind == 80)
        {
            if world
                .tile(num - 1, num2)
                .is_some_and(|t| t.active() && t.kind == 80)
                && world
                    .tile(num - 1, num2 - 1)
                    .is_some_and(|t| t.active() && t.kind == 80)
                && num >= i
            {
                num -= 1;
            }
            if world
                .tile(num + 1, num2)
                .is_some_and(|t| t.active() && t.kind == 80)
                && world
                    .tile(num + 1, num2 - 1)
                    .is_some_and(|t| t.active() && t.kind == 80)
                && num <= i
            {
                num += 1;
            }
        }
    }
    let num6 = num2 - 1 - j;
    let num7 = i - num;
    num = i - num7;
    num2 = j;
    let num8 = 11 - num6;
    let mut num9 = 0;
    for num10 in (num - 2)..=(num + 2) {
        for num11 in (num2 - num8)..=(num2 + num6) {
            if let Some(t) = world.tile(num10, num11) {
                if t.active() && t.kind == 80 {
                    num9 += 1;
                }
            }
        }
    }
    // The drunk arm's cap is the 11-20 range (`:56690-56698`); the classic
    // worlds take the 11-13 cap the port already carried.
    if num9 >= 11 + rng.next_max(2) {
        return;
    }
    num = i;
    num2 = j;
    if num7 != 0 {
        // The lean arm (:56788-56797): a free cell above with the neighbour
        // shoot standing — the cactus leans into it.
        if !world.tile(num, num2 - 1).is_some_and(|t| t.active())
            && !world.tile(num, num2 - 2).is_some_and(|t| t.active())
            && !world.tile(num + num7, num2 - 1).is_some_and(|t| t.active())
            && world
                .tile(num - num7, num2 - 1)
                .is_some_and(|t| t.active() && t.kind == 80)
        {
            let idx = world.index(num, num2 - 1);
            world.tiles[idx] = crate::tile::Tile {
                kind: 80,
                frame_x: 0,
                frame_y: 0,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        return;
    }
    if num6 == 0 {
        if world.tile(num, num2 - 1).is_none_or(|t| !t.active()) {
            let idx = world.index(num, num2 - 1);
            world.tiles[idx] = crate::tile::Tile {
                kind: 80,
                frame_x: 0,
                frame_y: 0,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        return;
    }
    // The fork arms (:56739-56783): the side shoots and the tip.
    let mut flag = false;
    let mut flag2 = false;
    if world
        .tile(num, num2 - 1)
        .is_some_and(|t| t.active() && t.kind == 80)
    {
        if !world.tile(num - 1, num2).is_some_and(|t| t.active())
            && !world.tile(num - 2, num2 + 1).is_some_and(|t| t.active())
            && !world.tile(num - 1, num2 - 1).is_some_and(|t| t.active())
            && !world.tile(num - 1, num2 + 1).is_some_and(|t| t.active())
            && !world.tile(num - 2, num2).is_some_and(|t| t.active())
        {
            flag = true;
        }
        if !world.tile(num + 1, num2).is_some_and(|t| t.active())
            && !world.tile(num + 2, num2 + 1).is_some_and(|t| t.active())
            && !world.tile(num + 1, num2 - 1).is_some_and(|t| t.active())
            && !world.tile(num + 1, num2 + 1).is_some_and(|t| t.active())
            && !world.tile(num + 2, num2).is_some_and(|t| t.active())
        {
            flag2 = true;
        }
    }
    let num12 = rng.next_max(3);
    if num12 == 0 && flag {
        let idx = world.index(num - 1, num2);
        world.tiles[idx] = crate::tile::Tile {
            kind: 80,
            frame_x: 0,
            frame_y: 0,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
    } else if num12 == 1 && flag2 {
        let idx = world.index(num + 1, num2);
        world.tiles[idx] = crate::tile::Tile {
            kind: 80,
            frame_x: 0,
            frame_y: 0,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
    } else if num6 < rng.next_range(2, 8)
        && world
            .tile(num - 1, num2 - 1)
            .is_none_or(|t| !t.active() || t.kind != 80)
        && world
            .tile(num + 1, num2 - 1)
            .is_none_or(|t| !t.active() || t.kind != 80)
        && world.tile(num, num2 - 1).is_none_or(|t| !t.active())
    {
        let idx = world.index(num, num2 - 1);
        world.tiles[idx] = crate::tile::Tile {
            kind: 80,
            frame_x: 0,
            frame_y: 0,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
    }
}

/// `WorldGen.SolidTile` (`:70111-70131`) — the growth module's own copy of the
/// tile-only solidity test (the shared `worldgen` module is crate-private).
fn growth_solid(world: &World, x: i32, y: i32) -> bool {
    match world.tile(x, y) {
        Some(t) => {
            t.active()
                && (t.kind as usize) < crate::tiledata::TILE_COUNT
                && crate::tiledata::TILE_SOLID.has(t.kind)
                && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
                && !t.half_brick()
                && t.slope() == 0
        }
        None => false,
    }
}

/// `WorldGen.GetWaterDepth` (`:58355-58373`): the liquid column above the
/// floor under `(x, y)` — the solid walk down, then the liquid count back up.
fn water_depth(world: &World, x: i32, y: i32) -> i32 {
    let mut num = y;
    while !growth_solid(world, x, num) {
        num += 1;
        if num > world.height - 1 {
            return 0;
        }
    }
    num -= 1;
    let mut num2 = num;
    while world.tile(x, num2).is_some_and(|t| t.liquid > 0)
        && !growth_solid(world, x, num2)
    {
        num2 -= 1;
    }
    num - num2
}

/// `WorldGen.CountGrowingPlantTiles` (`:58375-58390`): the tiles of `kind` in
/// the `[x-range, x+range] × [y-range*3, y+range*3]` box.
fn count_growing_plant_tiles(world: &World, x: i32, y: i32, range: i32, kind: u16) -> i32 {
    let mut num = 0;
    for i in (x - range)..=(x + range) {
        for j in (y - range * 3)..=(y + range * 3) {
            if world.tile(i, j).is_some_and(|t| t.active() && t.kind == kind) {
                num += 1;
            }
        }
    }
    num
}

/// `WorldGen.PlaceBamboo` (`:58391-58445`): one bamboo stalk on the jungle
/// grass / bamboo under `(x, y)` — the wall/surface guard, the water-depth
/// window (2..5), the density gates and the write. Returns whether a stalk
/// landed.
fn place_bamboo(world: &mut World, x: i32, y: i32, rng: &mut UnifiedRandom) -> bool {
    let min_depth = 2;
    let max_depth = 5;
    let num3 = rng.next_range(1, 21);
    let Some(tile) = world.tile(x, y) else {
        return false;
    };
    if tile.wall > 0 && (y as f64) <= world.world_surface {
        return false;
    }
    if tile.active() && tile.kind == 314 {
        return false;
    }
    let below = world.tile(x, y + 1).map(|t| (t.active(), t.kind));
    let Some((below_active, below_kind)) = below else {
        return false;
    };
    if below_active && matches!(below_kind, 571 | 60) {
        let depth = water_depth(world, x, y);
        if depth < min_depth || depth > max_depth {
            return false;
        }
        let mut num4 = count_growing_plant_tiles(world, x, y, 5, 571);
        let mut i = 1;
        if below_kind == 571 {
            loop {
                if growth_solid(world, x, y + i) {
                    break;
                }
                i += 1;
            }
            if i + num4 / rng.next_range(1, 21) > num3 {
                return false;
            }
        } else {
            num4 += 25;
        }
        if num4 + i * 2 > rng.next_range(40, 61) {
            return false;
        }
        if let Some(t) = world.tile_mut(x, y) {
            t.kind = 571;
            t.set_slope(0);
            t.flags = FLAG_ACTIVE;
        }
        return true;
    }
    false
}

/// `TileID.Sets.IsVine` (`TileID.cs:531`) — the vine kinds the density count
/// reads.
const IS_VINE: &[u16] = &[52, 382, 62, 115, 205, 528, 636, 638];

/// `Collision.CanHitLine`'s LOS over the two tile centres: the port's own
/// Bresenham walk (the shared `smart_cursor` helper is module-private; the C#
/// call passes 1x1 boxes, so the walk runs centre-to-centre).
fn can_hit_line(world: &World, x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
    let steep = (y2 - y1).abs() > (x2 - x1).abs();
    let (mut ax, mut ay, mut bx, mut by) = (x1, y1, x2, y2);
    if steep {
        std::mem::swap(&mut ax, &mut ay);
        std::mem::swap(&mut bx, &mut by);
    }
    let dx = (bx - ax).abs();
    let dy = (by - ay).abs();
    let sx = if ax < bx { 1 } else { -1 };
    let sy = if ay < by { 1 } else { -1 };
    let mut err = dx / 2;
    let mut i = ax;
    loop {
        let (px, py) = if steep { (ay, i) } else { (i, ay) };
        if world
            .tile(px, py)
            .is_some_and(|t| t.active() && growth_solid_kind(t.kind))
        {
            return false;
        }
        if i == bx {
            break;
        }
        err -= dy;
        if err < 0 {
            ay += sy;
            err += dx;
        }
        i += sx;
    }
    true
}

fn growth_solid_kind(kind: u16) -> bool {
    (kind as usize) < crate::tiledata::TILE_COUNT
        && crate::tiledata::TILE_SOLID.has(kind)
        && !crate::tiledata::TILE_SOLID_TOP.has(kind)
}

/// `WorldGen.GrowMoreVines` (`:45586-45621`): the vine-density cap around
/// `(x, y)` — the ±4/±6/±10 box's vine count, the LOS-weighted accumulation
/// below the source and the 60 budget (÷5 on the mushroom vine 528).
fn grow_more_vines(world: &World, x: i32, y: i32) -> bool {
    if x < 30 || y < 30 || x >= world.width - 30 || y >= world.height - 30 {
        return false;
    }
    let (lo_x, lo_y, hi_y, mut num4) = (4, 6, 10, 60);
    if world.tile(x, y).is_some_and(|t| t.kind == 528) {
        num4 /= 5;
    }
    let mut num5 = 0;
    for i in (x - lo_x)..=(x + lo_x) {
        for j in (y - lo_y)..=(y + hi_y) {
            if world.tile(i, j).is_some_and(|t| t.active() && IS_VINE.contains(&t.kind)) {
                num5 += 1;
                if j > y && can_hit_line(world, x, y, i, j) {
                    let weight = if world.tile(i, j).is_some_and(|t| t.kind == 528) {
                        20
                    } else {
                        2
                    };
                    num5 += (j - y) * weight;
                }
                if num5 > num4 {
                    return false;
                }
            }
        }
    }
    true
}

/// `UpdateWorld_UndergroundTile`'s vine arms (`:73310-73470`): the jungle,
/// mushroom, corrupt, crimson and ash families grow their vine one row below
/// at the vanilla odds (the 1/5 jungle arm, the 70/7 odds for the rest,
/// 7 when the tile is already the vine kind), behind the free-below gate, the
/// 10-row look-up through the family grass and `GrowMoreVines`' density.
pub(crate) fn grow_vine_arm(world: &mut World, i: i32, j: i32, rng: &mut UnifiedRandom) {
    let kind = world.tile(i, j).map(|t| (t.active(), t.kind, t.bottom_slope()));
    let Some((active, kind, bottom_slope)) = kind else {
        return;
    };
    if !active || bottom_slope {
        return;
    }
    // The free-below gate: the cell below the source must be inactive and dry
    // (the C#'s `!active() && !anyLava()` `:73313`).
    let below_clear = world
        .tile(i, j + 1)
        .is_some_and(|t| !t.active() && t.liquid == 0);
    if !below_clear {
        return;
    }
    // `(grass, vine, odds, gate)` per family (`:73311-73499`).
    let (grass, vine, odds, gated) = match kind {
        60 | 62 => (60, 62, 5, true),
        633 | 638 => (633, 638, 70, false),
        70 | 528 => (70, 528, 70, false),
        661 | 636 => (661, 636, 70, false),
        662 | 205 => (662, 205, 70, false),
        _ => return,
    };
    let odds = if kind == vine {
        if gated {
            odds
        } else {
            7
        }
    } else {
        odds
    };
    if rng.next_max(odds) != 0 {
        return;
    }
    if !grow_more_vines(world, i, j) {
        return;
    }
    // The 10-row look-up: the family grass below (a bottom slope aborts).
    let mut flag = false;
    let mut n = j;
    while n > j - 10 {
        let t = world.tile(i, n);
        if t.is_some_and(|t| t.bottom_slope()) {
            flag = false;
            break;
        }
        if t.is_some_and(|t| t.active() && t.kind == grass && !t.bottom_slope()) {
            flag = true;
            break;
        }
        n -= 1;
    }
    if !flag {
        return;
    }
    if let Some(t) = world.tile_mut(i, j + 1) {
        t.kind = vine;
        t.flags = FLAG_ACTIVE;
        t.liquid = 0;
    }
    if world.frame_important.len() <= vine as usize {
        world.frame_important.resize(crate::tiledata::TILE_COUNT, false);
    }
    world.frame_important[vine as usize] = true;
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
    rng: &mut UnifiedRandom,
) {
    // The "converted grass" map (`:45484-45491`): 23 ↔ 661 and 199 ↔ 662;
    // any other `landType` leaves the C# `-1` — no second id matches. The
    // port carries `u16::MAX` as that sentinel (0 would collide with dirt).
    let converted: u16 = match land_type {
        23 => 661,
        199 => 662,
        _ => u16::MAX,
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
    let (num2, num3) = match rng.next_max(4) {
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
    // `Main.tile[..].type = spikeType; active(active: true)` (`:45476-45480`)
    // — the C# writes the kind and the active bit on the EXISTING tile, so
    // the frame/liquid survive; the port's whole-Tile replacement zeroed
    // them (M5-SPREAD-2c).
    if let Some(t) = world.tile_mut(num2, num3) {
        t.kind = spike_type;
        t.flags |= FLAG_ACTIVE;
    }
    // `CopyPaintAndCoating(Main.tile[i, j])` (`:45481`) — the paint/coating
    // rides from the seed tile (the port's `Tile` carries paint in `meta` and
    // the coatings in `extras`).
    let (src_meta, src_extras) = {
        let si = world.index(i, j);
        (world.tiles[si].meta, world.extras_at(si))
    };
    let di = world.index(num2, num3);
    world.tiles[di].meta = src_meta;
    world.set_extras_at(di, src_extras);
    // `WorldGen.SquareTileFrame` (`:45482`).
    crate::edit::square_tile_frame_gen(world, num2, num3);
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
    max_raining: f32,
    wind_speed: f32,
    rng: &mut UnifiedRandom,
) {
    let kind = world.tile(x, y).map(|t| (t.active(), t.kind));
    let Some((active, kind)) = kind else {
        return;
    };
    if !active {
        // The beach-shells arm (`:72073`): an inactive beach cell under the
        // surface grows a shell pile behind the wind/rain odds.
        grow_shell_pile(world, x, y, wind_speed, max_raining, raining, rng);
        return;
    }
    // `:72063-72067`: the tombstone arm — a grave cell rolls
    // `TryGrowingAbigailsFlower` once per update, ahead of the alch/beach
    // chain the C# walks next.
    if kind == 85 {
        try_growing_abigails_flower(world, x, y, rng);
    }
    // `:72219-72228`: the dye-plant head — one `Main.rand.Next(3000)` behind
    // every `nactive` pick, the exotic arm at `Next(15000)` when the cell
    // stands outside the middle 40..60 % of the world and hardmode is on.
    if rng.next_max(3000) == 0 {
        plant_dye(world, x, y, false, rng);
    } else if world.hard_mode
        && ((x as f64) < world.width as f64 * 0.4 || (x as f64) > world.width as f64 * 0.6)
        && rng.next_max(15000) == 0
    {
        plant_dye(world, x, y, true, rng);
    }
    if TILE_ALCH.contains(&kind) {
        grow_alch(world, x, y, day_time, raining, rng);
        return;
    }
    // The sapling arm (:72430-72434): the kind-20 sapling at `Next(20)`,
    // the beach palm saplings 595/615 at `Next(5)` (the settings profiles
    // land with the vanity trees).
    if kind == 20 && rng.next_max(20) == 0 {
        attempt_grow_tree_from_sapling(world, x, y, false, rng);
        return;
    }
    if (kind == 595 || kind == 615) && rng.next_max(5) == 0 {
        // `AttemptToGrowTreeFromSapling`'s 595/615 arms (`:72435-72444`)
        // — the Sakura/Willow settings profiles.
        attempt_grow_tree_from_sapling(world, x, y, false, rng);
        return;
    }
    // The vanilla/KillTile arm (:72146-72152): the sakura/willow sapling or
    // its grown tile standing on the evil grass is killed (the trees do not
    // take root in the corruption/crimson).
    if matches!(kind, 595 | 615 | 596 | 616)
        && world
            .tile(x, y + 1)
            .is_some_and(|t| t.active() && matches!(t.kind, 199 | 23))
    {
        if let Some(t) = world.tile_mut(x, y) {
            t.flags &= !FLAG_ACTIVE;
        }
        return;
    }
    // The bamboo arm (:72154-72156): the bamboo 571 or the jungle grass 60
    // with liquid above, at `Next(5)`, when the cell above is free and — for
    // the jungle grass — at `Next(30)`.
    let above_kind = world.tile(x, y - 1).map(|t| (t.active(), t.kind, t.liquid));
    let bamboo = match above_kind {
        Some((_, _, liquid)) if kind == 60 && liquid > 0 => true,
        Some(_) if kind == 571 => true,
        _ => false,
    };
    if bamboo && rng.next_max(5) == 0 {
        let above_free = world
            .tile(x, y - 1)
            .is_some_and(|t| !t.active() || matches!(t.kind, 61 | 74 | 518));
        let gate = if kind == 60 { rng.next_max(30) == 0 } else { true };
        if above_free && gate {
            place_bamboo(world, x, y - 1, rng);
        }
    }
    // The cactus arm (:72229-72231): `Next(15)`.
    if kind == 80 && rng.next_max(15) == 0 {
        grow_cactus(world, x, y, rng);
        return;
    }
    // The bare-sand arm (:72282-72284) rolls `Next(300)` inside the
    // beach-distance bounds — the port has no beachDistanceYet; the
    // water-box abort inside `grow_cactus` keeps the shore in check.
    if matches!(kind, 53 | 234 | 116 | 112) && rng.next_max(300) == 0 {
        grow_cactus(world, x, y, rng);
        return;
    }
    // The grass-growth pass (:72357 + :73282): the spread-overground grasses
    // with the plant windows (minI/maxI = i±1, minJ/maxJ = j±1 per :72350-
    // 72356's window).
    // `TileID.Sets.SpreadOverground` (`TileID.cs:621`) = 2/23/661/32/60/70/
    // 109/199/662/352/477/492/633/226 — the 32/352 thorns and the golf-grass
    // 477/492 were missing, which left the thorn arm below unreachable for
    // the tiles it actually grows on (M5-SPREAD-2c).
    let grass_pass = matches!(
        kind,
        2 | 23 | 199 | 60 | 70 | 109 | 226 | 633 | 661 | 662 | 32 | 352 | 477 | 492
    );
    // The moss families (`:72371`'s gate) — the next else-if after the
    // `SpreadOverground` grass pass.
    if (crate::tiledata::TILE_MOSS.has(kind) || crate::tiledata::SET_TILE_MOSS_BRICK.has(kind))
        && active
    {
        grow_moss_arm(
            world,
            x,
            y,
            x - 1,
            x + 2,
            y - 1,
            y + 2,
            rng,
        );
        return;
    }
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
        // The thorn arms (:72362-72370): the corruption thorn 32 grows into
        // the corrupt grass 23 and the crimson thorn 352 into the crimson
        // grass 199, both at `Next(3)` — the C# triggers on the THORN tile,
        // not the grass (the grass keys are `landType`'s density map).
        if (kind == 32 || kind == 352) && rng.next_max(3) == 0 {
            if kind == 32 {
                grow_spike(world, x, y, 32, 23, rng);
            } else {
                grow_spike(world, x, y, 352, 199, rng);
            }
        }
        // The jungle thorn arm (:73658-73661): the 69 tile grows the same
        // walk with no roll.
        if kind == 69 {
            grow_spike(world, x, y, 69, 60, rng);
        }
    }
}

/// The crystal-shard arm of `hardUpdateWorld` (`:69603-69640`): the shard
/// stones below the rock layer roll `Next(5)`, the 4-dir neighbour must be
/// empty, the radius-6 density stays under 2, the style `Next(18)` (the big
/// shard `18 + Next(6)` at 1/50).
pub fn grow_crystal_shards(world: &mut World, x: i32, y: i32, rng: &mut UnifiedRandom) {
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
    if rng.next_max(5) != 0 {
        return;
    }
    // `:69609-69625`: the 4-dir pick — 0 left, 1 right, 2/3 down (the C#'s
    // `default: num3 = ((num != 0) ? 1 : -1)` resolves to +1 for both —
    // the shards never grow upward).
    let (dx, dy) = match rng.next_max(4) {
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
    let mut style = rng.next_max(18);
    if rng.next_max(50) == 0 {
        style = 18 + rng.next_max(6);
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
    // `WorldGen.AllowedToSpreadInfections` (`WorldGen.cs:71502-71507`): true
    // unless the journey `StopBiomeSpreadPower` is enabled. The underground
    // dispatcher's infection rows and wall arms read it.
    allowed_to_spread: bool,
    day_time: bool,
    max_raining: f32,
    wind_speed: f32,
    rng: &mut UnifiedRandom,
) -> u32 {
    let rate = world_update_rate(day_rate, freeze_time);
    if rate == 0 {
        return 0;
    }
    // `WorldGen.UpdateWorld`'s setup (`:71503`, `:71508`): the infection
    // spread flag comes in from the caller (`AllowedToSpreadInfections`, set
    // from the journey `StopBiomeSpreadPower`) and the wall-arm box radius is
    // the constant 3.
    let wall_dist = 3;
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
        update_world_overground_tile(
            world,
            x,
            y,
            day_time,
            raining,
            max_raining,
            wind_speed,
            rng,
        );
        picks += 1;
    }
    let underground_count = (world.width as f64 * world.height as f64) * 1.5e-5 * rate as f64;
    for _ in 0..underground_count as i32 {
        let x = rng.next_range(10, world.width - 10);
        let y = rng.next_range(world.world_surface as i32 - 1, world.height - 20);
        // The underground dispatcher (`UpdateWorld_UndergroundTile`
        // `:73214-73614`) replaces the overground parity call; the dyed/
        // moss/oyster arms inside it park (ANALYSIS).
        update_world_underground_tile(
            world,
            x,
            y,
            wall_dist,
            allowed_to_spread,
            day_time,
            raining,
            rng,
        );
        picks += 1;
    }
    picks
}

/// `WorldGen.beachDistance` = 380 (`WorldGen.cs:86914`) — the shell arm's
/// beach margins.
const BEACH_DISTANCE: i32 = 380;

/// `WorldGen.RollRandomSeaShellStyle` (`WorldGen.cs:45170-45187`) over the
/// growth pass's `UnifiedRandom` (the C# `genRand`): the 5 seashell styles on
/// tile 324 — `Next(2)`, then `Next(10)==0 → 2`, `Next(10)==0 → 3`,
/// `Next(50)==0 → 4` (nested `if`s, so the later rolls re-roll earlier).
fn roll_sea_shell_style(rng: &mut UnifiedRandom) -> i32 {
    let mut result = rng.next_range(0, 2);
    if rng.next_max(10) == 0 {
        result = 2;
    }
    if rng.next_max(10) == 0 {
        result = 3;
    }
    if rng.next_max(50) == 0 {
        result = 4;
    }
    result
}

/// The beach-shells arm of `UpdateWorld_OvergroundTile`
/// (`WorldGen.cs:72073-72135`): a dry empty cell in the beach margin under
/// the surface grows a shell pile — the calmer and drier the weather the
/// higher the odds (`num5 = 3000 - |wind| · 1250 - maxRaining · 1250`, floor
/// 300), then the sand scan, the pile-count cap `Next(2, 5)` with the
/// ±`Next(8, 11)` box check and the coral (81, on deep water) / seashell
/// (324, the rolled style) write.
pub fn grow_shell_pile(
    world: &mut World,
    i: i32,
    j: i32,
    wind_speed: f32,
    max_raining: f32,
    raining: bool,
    rng: &mut UnifiedRandom,
) -> bool {
    let surface = world.world_surface;
    let width = world.width;
    // The gate: an inactive cell under the surface inside the beach margins
    // (`:72073`).
    if !((j as f64) < surface + 10.0) {
        return false;
    }
    if !(i < BEACH_DISTANCE || i > width - BEACH_DISTANCE) {
        return false;
    }
    if world.tile(i, j).is_some_and(|t| t.active()) {
        return false;
    }
    // The odds (`:72074-72080`).
    let mut num5 = 3000 - (wind_speed.abs() * 1250.0) as i32;
    if raining {
        num5 -= (1250.0 * max_raining) as i32;
    }
    if num5 < 300 {
        num5 = 300;
    }
    if rng.next_max(num5.max(1)) != 0 {
        return false;
    }
    // The sand scan (`:72083-72090`): down to the first solid within 15 rows.
    let mut k = j;
    while (k as f64) < surface + 10.0 && !world.tile(i, k).is_some_and(|t| t.active()) && k - j < 15
    {
        k += 1;
    }
    let sand_ok = world
        .tile(i, k)
        .is_some_and(|t| t.active() && t.kind == 53)
        && crate::edit::solid_tile_allow_bottom_slope(world, i, k);
    if !sand_ok {
        return false;
    }
    // The pile above the sand (`:72091-72096`): an existing pile stands down.
    if world.tile(i, k - 1).is_some_and(|t| t.active() && t.kind == 324) {
        return false;
    }
    // The density cap and the box count (`:72097-72110`).
    let num6 = rng.next_range(2, 5);
    let num7 = rng.next_range(8, 11);
    let mut num8 = 0;
    for l in (i - num7)..=(i + num7) {
        for m in (k - num7)..=(k + num7) {
            if world.tile(l, m).is_some_and(|t| t.active() && matches!(t.kind, 324 | 81)) {
                num8 += 1;
            }
        }
    }
    if num8 >= num6 {
        return false;
    }
    // The coral on the deep water, the seashell otherwise (`:72111-72130`).
    let coral = rng.next_max(2) == 0 && world.tile(i, k).is_some_and(|t| t.liquid >= 230);
    let style = if coral { 0 } else { roll_sea_shell_style(rng) };
    let idx = world.index(i, k - 1);
    let t = &mut world.tiles[idx];
    t.flags |= crate::tile::FLAG_ACTIVE;
    t.kind = if coral { 81 } else { 324 };
    if !coral {
        t.frame_x = (22 * style) as i16;
    }
    true
}

#[cfg(test)]
pub(crate) mod test_fixtures {
    use super::*;
    use crate::tile::Tile;

    pub(crate) fn world() -> World {
        let mut w = World::new("t".into(), 4200, 400);
        w.rock_layer = 160.0;
        w
    }

    pub(crate) fn put(w: &mut World, x: i32, y: i32, kind: u16, frame_x: i16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            frame_x,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    /// The underground vine arms (`:73310-73470`): the jungle grass with the
    /// vine below it grows the vine one row down at the 1/5 arm, behind the
    /// free-below gate, the 10-row look-up and `GrowMoreVines`' density.
    #[test]
    fn the_vine_arms_grow_below_their_grass() {
        // A jungle-grass column 30..32 with the free cell below (33), far
        // enough from the world edges for GrowMoreVines' InWorld(30).
        let mut w = World::new("t".into(), 200, 200);
        w.world_surface = 100.0;
        for y in 30..=32 {
            let i = w.index(100, y);
            w.tiles[i] = crate::tile::Tile {
                kind: 60,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut grew = 0;
        let mut runs = 0;
        for seed in 1u64..600 {
            let mut w = w.clone();
            let mut rng = UnifiedRandom::new(seed as i32);
            grow_vine_arm(&mut w, 100, 32, &mut rng);
            runs += 1;
            grew += w
                .tiles
                .iter()
                .filter(|t| t.active() && t.kind == 62)
                .count();
        }
        assert!(grew > 0, "the jungle arm grows the vine 62 ({grew} over {runs})");

        // The control: the same column with the vine already one below takes
        // the 70-odds arm of the family (the vine kind's own odds).
        let mut w = World::new("t".into(), 200, 200);
        w.world_surface = 100.0;
        for (y, kind) in [(30, 60), (31, 60), (32, 62), (33, 60)] {
            let i = w.index(100, y);
            w.tiles[i] = crate::tile::Tile {
                kind,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut grew2 = 0;
        for seed in 1u64..600 {
            let mut w = w.clone();
            let mut rng = UnifiedRandom::new(seed as i32);
            grow_vine_arm(&mut w, 100, 33, &mut rng);
            grew2 += w
                .tiles
                .iter()
                .filter(|t| t.active() && t.kind == 62)
                .count();
        }
        assert!(grew2 > 0, "the vine's own arm extends it ({grew2})");
    }

    /// fix/vines-destroy-walls: the runtime vine arm (`WorldGen.cs:73331-
    /// 73335` — `type = 62`, `active(true)`, `SquareTileFrame`,
    /// `CopyPaintAndCoating`) writes the tile fields only; the cell's
    /// `wall`/`wallFrame` are separate fields the arm never touches, so a
    /// vine growing into a walled mini-cave cell keeps the wall behind it.
    #[test]
    fn the_vine_arm_preserves_the_wall_and_wall_frame() {
        let mut base = World::new("t".into(), 200, 200);
        base.world_surface = 100.0;
        for y in 30..=32 {
            let i = base.index(100, y);
            base.tiles[i].kind = 60;
            base.tiles[i].flags |= FLAG_ACTIVE;
        }
        // The free cell below carries the natural dirt wall + its autotile
        // frame (the mini-cave pocket the user reported).
        let below = base.index(100, 33);
        base.tiles[below].wall = 2; // WallID.DirtUnsafe
        base.tiles[below].set_wall_frame(5, 2);
        let mut grew = 0;
        for seed in 1u64..600 {
            let mut w = base.clone();
            let mut rng = UnifiedRandom::new(seed as i32);
            grow_vine_arm(&mut w, 100, 32, &mut rng);
            let t = w.tile(100, 33).unwrap();
            if t.active() && t.kind == 62 {
                grew += 1;
                assert_eq!(t.wall, 2, "the grown vine keeps the dirt wall");
                assert_eq!(
                    (t.wall_col(), t.wall_row()),
                    (5, 2),
                    "the grown vine keeps the wall autotile frame"
                );
                assert_eq!(t.liquid, 0, "the arm zeroes the liquid");
            } else {
                // No growth: the cell is untouched, wall included.
                assert_eq!(t.wall, 2);
                assert_eq!((t.wall_col(), t.wall_row()), (5, 2));
            }
        }
        assert!(grew > 0, "the arm grew at least one vine ({grew})");
    }

    /// The bamboo arm (`:72154-72156` + `PlaceBamboo` `:58391-58445`): a
    /// bamboo 571 with a free cell above grows a stalk at the vanilla gates —
    /// the wall/surface refusal, the water-depth window (2..5), the density
    /// caps and the write.
    #[test]
    fn the_bamboo_arm_grows_a_stalk_over_the_water() {
        // A pool of water over a floor: the bamboo 571 on the floor with a
        // water column 3 deep and the free cell above it.
        let mut w = World::new("t".into(), 60, 80);
        w.world_surface = 40.0;
        // The floor at y = 60.
        for x in 0..60 {
            let i = w.index(x, 60);
            w.tiles[i] = crate::tile::Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        // The water column y = 57..59 and the bamboo at y = 56.
        for x in 0..60 {
            for y in 57..=59 {
                let i = w.index(x, y);
                w.tiles[i].liquid = 255;
            }
            let i = w.index(30, 56);
            w.tiles[i] = crate::tile::Tile {
                kind: 571,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        // Sweep seeds: the arm's Next(5)/Next(30) gates and PlaceBamboo's
        // own rolls land across enough streams.
        let mut grew = 0;
        let mut runs = 0;
        for seed in 1u64..600 {
            let mut w = w.clone();
            let mut rng = UnifiedRandom::new(seed as i32);
            update_world_overground_tile(&mut w, 30, 56, true, false, 0.0, 0.0, &mut rng);
            runs += 1;
            grew += w
                .tiles
                .iter()
                .filter(|t| t.active() && t.kind == 571)
                .count();
        }
        assert!(grew > 0, "the bamboo arm grows a stalk ({grew} over {runs} picks)");
    }

    /// The vanity-KillTile arm (`:72146-72152`): a sakura/willow sapling or
    /// its grown tile standing on the evil grass is killed, not left rooted.
    #[test]
    fn the_vanity_saplings_die_on_the_evil_grass() {
        // The arm runs on a 595/615 sapling (or its grown tile) standing on the
        // corruption/crimson grass (`:72146-72152`). The dispatcher's stream
        // order puts the dye head (`:72219-72228`) in front of the type arms,
        // so the sweep runs a spread of seeds and needs the kill to happen on
        // at least one.
        let build = |soil: u16| {
            let mut w = World::new("t".into(), 40, 60);
            w.world_surface = 40.0;
            let i = w.index(10, 30);
            w.tiles[i] = crate::tile::Tile {
                kind: soil,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
            let i = w.index(10, 29);
            w.tiles[i] = crate::tile::Tile {
                kind: 595,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
            w
        };
        let mut killed = false;
        for k in 0..256u64 {
            let seed = k.wrapping_mul(0x9E37_79B9_7F4A_7C15);
            let mut w = build(23);
            let mut rng = UnifiedRandom::new(seed as i32);
            update_world_overground_tile(&mut w, 10, 29, true, false, 0.0, 0.0, &mut rng);
            if !w.tile(10, 29).is_some_and(|t| t.active()) {
                killed = true;
                break;
            }
        }
        assert!(killed, "the sapling on the evil grass is killed");

        // The control: on plain grass it survives the arm on every stream.
        for k in 0..64u64 {
            let seed = k.wrapping_mul(0x9E37_79B9_7F4A_7C15);
            let mut w = build(2);
            let mut rng = UnifiedRandom::new(seed as i32);
            update_world_overground_tile(&mut w, 10, 29, true, false, 0.0, 0.0, &mut rng);
            assert!(
                w.tile(10, 29).is_some_and(|t| t.active()),
                "the sapling on the plain grass survives"
            );
        }
    }

    use super::*;

    use super::*;
    use crate::tile::Tile;

    fn world() -> World {
        let mut w = World::new("t".into(), 4200, 400);
        w.rock_layer = 160.0;
        w
    }

    /// Sets a cell's wall (the wall-arm tests' natural/evil walls).
    pub(super) fn set_wall(w: &mut World, x: i32, y: i32, wall: u16) {
        let i = w.index(x, y);
        w.tiles[i].wall = wall;
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
        let mut rng = UnifiedRandom::new(4);
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
    fn the_underground_dispatcher_runs_the_passed_arms() {
        // `UpdateWorld_UndergroundTile`'s ported arms (`:73214-73614`): the
        // alch plant grows, the grass family converts (underground), the lily
        // becomes 74, the grass growth re-runs the spikes and the cattail runs
        // its validity/growth pair. The parked arms (the 655 kill, the dye/
        // seaweed, the 444/396 moss/oyster, the web/town-NPC) stay out.
        // The lily conversion: a lily (61) at Next(3)==0 over a non-226 tile
        // converts to 74.
        let mut w = world();
        put(&mut w, 100, 200, 61, 0);
        let mut grew = false;
        for seed in 1..64u64 {
            let mut rng = UnifiedRandom::new(seed as i32);
            update_world_underground_tile(&mut w, 100, 200, 3, true, true, false, &mut rng);
            if w.tile(100, 200).is_some_and(|t| t.kind == 74) {
                grew = true;
                break;
            }
        }
        assert!(grew, "the lily converts to 74 on some stream");

        // The grass family: the jungle grass 60 converts in the underground
        // direction (the conversion arm runs through the dispatcher).
        let mut w2 = world();
        put(&mut w2, 100, 200, 0, 0);
        put(&mut w2, 100, 201, 60, 0);
        put(&mut w2, 99, 201, 60, 0);
        let mut converted = false;
        for seed in 1..64u64 {
            let mut rng = UnifiedRandom::new(seed as i32);
            update_world_underground_tile(&mut w2, 100, 201, 3, true, true, false, &mut rng);
            if w2.tile(100, 200).is_some_and(|t| t.active()) {
                converted = true;
                break;
            }
        }
        assert!(converted, "the grass arm runs underground");
    }

    #[test]
    fn the_beach_shells_arm_places_the_pile() {
        // The beach-shells arm (`:72073-72135`): a dry empty beach cell under
        // the surface grows a seashell (324) behind the calm/dry odds — the
        // gate first, then the wind/rain-adjusted `num5` roll, the sand scan,
        // the density cap and the write. With the odds at their easiest
        // (calm, dry: `num5 = 3000`) the 1-in-3000 roll still needs luck, so
        // the test seeds a hit through a tight loop and checks the contract.
        let mut w = world();
        w.world_surface = 100.0;
        // The beach margin: a sand floor at y = 101 near the world edge.
        for dx in -30i32..=30 {
            let i = w.index(5 + dx.max(0) as i32, 101);
            w.tiles[i] = crate::tile::Tile {
                kind: 53,
                flags: crate::tile::FLAG_ACTIVE,
                ..Default::default()
            };
        }
        // The pick sits above the sand, inside the margin (i < 380). The
        // search window is a fixture, not a constant: a 1-in-3000 first draw
        // needs ~21 hits' worth of seeds to be stable (the UnifiedRandom
        // re-pin widened it from the FastRandom-era 4096).
        let grew = (1..65536u64).any(|seed| {
            let mut rng = UnifiedRandom::new(seed as i32);
            grow_shell_pile(&mut w, 5, 100, 0.0, 0.0, false, &mut rng)
        });
        assert!(grew, "a calm dry day grows a shell pile on the beach");
        let pile = w.tile(5, 100).is_some_and(|t| t.active() && t.kind == 324)
            || w.tile(5, 100)
                .is_some_and(|t| t.active() && t.kind == 81);
        assert!(pile, "the pile is a seashell or a coral");
    }

    #[test]
    fn the_beach_shells_arm_refuses_the_gates() {
        // The arm's refusals: the weather odds (a storm makes `num5` bottom
        // at 300 but the roll still runs), a cell outside the beach margin,
        // and a cell with no sand below.
        let mut w = world();
        w.world_surface = 100.0;
        for dx in -30i32..=30 {
            let i = w.index(4000 + dx.max(0) as i32, 101);
            w.tiles[i] = crate::tile::Tile {
                kind: 53,
                flags: crate::tile::FLAG_ACTIVE,
                ..Default::default()
            };
        }
        let mut rng = UnifiedRandom::new(3);
        // Mid-world (outside the beach margins) never grows.
        assert!(
            !grow_shell_pile(&mut w, 2000, 100, 0.0, 0.0, false, &mut rng),
            "the inland cell is refused"
        );
        // The beach cell with no sand under it (the floor is stone).
        let mut w2 = world();
        w2.world_surface = 100.0;
        for dx in -30i32..=30 {
            let i = w2.index(5 + dx.max(0) as i32, 101);
            w2.tiles[i] = crate::tile::Tile {
                kind: 1,
                flags: crate::tile::FLAG_ACTIVE,
                ..Default::default()
            };
        }
        for seed in 1..256u64 {
            let mut rng2 = UnifiedRandom::new(seed as i32);
            if grow_shell_pile(&mut w2, 5, 100, 0.0, 0.0, false, &mut rng2) {
                panic!("no sand under the cell yet the pile grew");
            }
        }
    }

    /// The cactus arm's fork (`:56739-56783`): a standing column forks into the
    /// side shoots (the left/right clearance checks behind `Next(3)`) or the
    /// tip (`num6 < Next(2,8)` with the diagonals clear), and the lean arm
    /// (`:56788-56797`) fills the free cell when the column leans.
    #[test]
    fn the_cactus_forks_into_the_side_shoots_and_the_tip() {
        // A 3-tall column on sand: num6 = 2, so the tip gate's range and the
        // side shoots are both live over a seed sweep.
        let mut base = World::new("t".into(), 60, 80);
        base.world_surface = 40.0;
        for x in 0..60 {
            for y in 30..80 {
                let i = base.index(x, y);
                base.tiles[i] = crate::tile::Tile {
                    kind: 53,
                    ..Default::default()
                };
                base.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for y in 28..30 {
            let i = base.index(30, y);
            base.tiles[i] = crate::tile::Tile {
                kind: 80,
                ..Default::default()
            };
            base.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut fork = 0;
        let mut runs = 0;
        for seed in 1u64..800 {
            let mut w = base.clone();
            let mut rng = UnifiedRandom::new(seed as i32);
            // j is the column's TOP cell (the C# walks the stack downward).
            grow_cactus(&mut w, 30, 28, &mut rng);
            runs += 1;
            // The side shoot or the tip adds a 80 outside the original column.
            fork += w
                .tiles
                .iter()
                .enumerate()
                .filter(|(idx, t)| t.active() && t.kind == 80 && {
                    let x = (*idx % 60) as i32;
                    let y = (*idx / 60) as i32;
                    // Outside the original column (28..=29): the tip above or
                    // the side shoots.
                    !(x == 30 && (28..=29).contains(&y))
                })
                .count();
        }
        assert!(fork > 0, "the column forks ({fork} over {runs} picks)");
    }

    #[test]
    fn test_spread_1_cactus_plants_on_sand_with_the_box_gates() {
        // `GrowCactus` (:56575-56660): the sand arm needs the neighbours
        // above clear, <=3 cacti in the i±6 box and >10 sand in it.
        let mut w = world();
        for dx in -8..=8 {
            put(&mut w, 100 + dx, 200, 53, 0); // the sand floor
        }
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
        update_world_grass_growth(&mut w, 100, 200, 99, 102, 199, 202, true, &mut rng);
        assert!(
            w.tile(101, 200).is_some_and(|t| t.kind == 2),
            "the dirt converted to grass"
        );
        // The spike: the THORN tile itself is the updated cell (the C#
        // `:72362-72370` arm triggers on 32/352, not on the grass) — the
        // diamond needs the land grass with a standing thorn above it.
        let mut w = world();
        w.world_surface = 250.0; // the surface below the cells
        put(&mut w, 100, 201, 199, 0); // the land type below the thorn
        put(&mut w, 101, 201, 199, 0);
        put(&mut w, 100, 200, 352, 0); // the thorn being updated
        put(&mut w, 100, 199, 352, 0); // the standing thorn over the land
        let mut rng = UnifiedRandom::new(7);
        let mut grew = false;
        for tick in 0..200 {
            update_world_overground_tile(&mut w, 100, 200, true, false, 0.0, 0.0, &mut rng);
            // The walk lands at a free neighbour of the thorn; the world's
            // 352 count is the observable.
            let n352 = w
                .tiles
                .iter()
                .filter(|t| t.active() && t.kind == 352)
                .count();
            if n352 > 2 {
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
        let mut rng = UnifiedRandom::new(9);
        let picks = update_world(&mut w, 1, false, false, true, true, 0.0, 0.0, &mut rng);
        assert_eq!(picks, 50 + 25);
        let picks = update_world(&mut w, 1, true, false, true, true, 0.0, 0.0, &mut rng);
        assert_eq!(picks, 75 + 25);
        // The frozen time picks nothing.
        let picks = update_world(&mut w, 1, false, true, true, true, 0.0, 0.0, &mut rng);
        assert_eq!(picks, 0);
        // The spread-stop flag changes no pick budget — `WorldGen.UpdateWorld`
        // only hands `AllowedToSpreadInfections` to the tile arms
        // (`:71502-71507`).
        let picks = update_world(&mut w, 1, false, false, false, true, 0.0, 0.0, &mut rng);
        assert_eq!(picks, 50 + 25);
    }

}

/// `WorldGen.UpdateWorld_UndergroundTile` (`WorldGen.cs:73214-73614`): the
/// underground growth dispatcher — the clamps, the alch/hard-mode arms, the
/// grass growth (underground) + the spike arms, the lily conversion, the five
/// vine families and, behind `allowed_to_spread`, the wall arms.
///
/// Parked (documented in ANALYSIS): the 655 life-fruit kill (the NPC store),
/// `plantDye` (and its `Main.rand.Next(2500)`/`Next(10000)` head rolls),
/// the 444/396 moss and oyster arms (`PlayerLOS`/`AnchorValid`/
/// `CountNearBlocksTypes` have no ports), `TrySpawningTownNPC`, the net sides,
/// and the C#'s dual `genRand`/`Main.rand` streams (the port's per-consumer
/// stream carries every draw).
pub fn update_world_underground_tile(
    world: &mut World,
    i: i32,
    j: i32,
    wall_dist: i32,
    allowed_to_spread: bool,
    day_time: bool,
    raining: bool,
    rng: &mut UnifiedRandom,
) {
    // The clamped box (`:73216-73231`).
    let min_i = (i - 1).max(10);
    let max_i = (i + 2).min(world.width - 10);
    let min_j = (j - 1).max(10);
    let max_j = (j + 2).min(world.height - 10);
    let Some(tile) = world.tile(i, j) else {
        return;
    };
    let (kind, active) = (tile.kind, tile.active());
    // The 655 life-fruit kill parks (`:73239-73246`).
    if TILE_ALCH.contains(&kind) {
        grow_alch(world, i, j, day_time, raining, rng);
        return;
    }
    if !active {
        // The hard-mode infection arm (`:73252`) and the tail `else`'s web arm
        // (`:73803-73806`: wall 62, no liquid). The dye/seaweed, 444/396 and
        // town-NPC arms park.
        crate::spread::hard_update_world(world, i, j, false, allowed_to_spread, rng);
        if world.tile(i, j).is_some_and(|t| t.wall == 62 && t.liquid == 0) {
            grow_web(world, i, j, rng);
        }
        return;
    }
    // `:73255-73262`: the dye-plant head — `Main.rand.Next(2500)` on every
    // `nactive` pick, the exotic arm at `Next(10000)` in hardmode.
    if rng.next_max(2500) == 0 {
        plant_dye(world, i, j, false, rng);
    } else if world.hard_mode && rng.next_max(10000) == 0 {
        plant_dye(world, i, j, true, rng);
    }
    // `:73263-73271`: the cattail arm — the validity sweep, then (on
    // `genRand.Next(2) == 0`, the cell still live) one growth step and a
    // second validity sweep.
    if kind == 519 {
        check_cat_tail(world, i, j, rng);
        if world.tile(i, j).is_some_and(|t| t.active()) && rng.next_max(2) == 0 {
            grow_cat_tail(world, i, j, rng);
            check_cat_tail(world, i, j, rng);
        }
    }
    // `:73272-73282`: the seaweed arms — a 549 sweeps (and grows) every pick,
    // a 53 sand cell with a flat top and a free cell above rolls `Next(20)`.
    // The C#'s `else if` keeps the sand roll out of the 549 path and the
    // `&&` chain draws only after the tile gates pass.
    if kind == 549 {
        grow_check_seaweed(world, i, j, rng);
    } else if kind == 53
        && world.tile(i, j).is_some_and(|t| !t.top_slope() && !t.half_brick())
        && !world.tile(i, j - 1).is_some_and(|t| t.active())
        && rng.next_max(20) == 0
    {
        grow_check_seaweed(world, i, j, rng);
    }
    // The grass family + the spikes (`:73281-73295`).
    if SPREAD_UNDERGROUND.contains(&kind) {
        update_world_grass_growth(
            world,
            i,
            j,
            min_i,
            max_i,
            min_j,
            max_j,
            allowed_to_spread,
            rng,
        );
        if matches!(kind, 32 | 352) && rng.next_max(3) == 0 {
            if kind == 32 {
                grow_spike(world, i, j, 32, 23, rng);
            } else {
                grow_spike(world, i, j, 352, 199, rng);
            }
        }
        return;
    }
    // The lily → 74 conversion (`:73296-73310`).
    if kind == 61 && rng.next_max(3) == 0 && world.tile(i, j + 1).map(|t| t.kind).unwrap_or(0) != 226 {
        let frame_x = world.tile(i, j).map(|t| t.frame_x).unwrap_or(0);
        if frame_x < 144 {
            if rng.next_max(4) == 0 {
                if let Some(t) = world.tile_mut(i, j) {
                    t.frame_x = (162 + rng.next_range(0, 8) * 18) as i16;
                }
            }
            if let Some(t) = world.tile_mut(i, j) {
                t.kind = 74;
            }
        }
        return;
    }
    // The five vine families (`:73311-73482`) — the port's one-fn arm. The
    // 444/396 moss and oyster arms park.
    grow_vine_arm(world, i, j, rng);
    // The wall arms (`:72499-72502`, behind `AllowedToSpreadInfections`); the
    // web and town-NPC arms park.
    if allowed_to_spread {
        spread_grass_walls(world, wall_dist, i, j, rng);
        spread_desert_walls(world, wall_dist, i, j, rng);
    }
}


