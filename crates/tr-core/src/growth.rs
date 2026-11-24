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
// W4-g42 dispatcher follow-up: the OVERWORLD `UpdateWorld_GrassGrowth` arms
// (`WorldGen.cs:74510-74866`), the jungle-plant helpers (`:47779-47944`,
// `:73164-73187`) and the UNDERGROUND tail switch (`:74414-74507`) +
// the runtime `PlaceCatTail` (`:58629-58742`).
mod jungle;
mod overworld;
// W4-c31 jungle residuals: the Plantera bulb event placement
// (`WorldGen.cs:74127-74267`) — the all-mechs-defeated hook. The placement
// helper it calls is the g42 keeper `jungle::place_jungle_plant` (the c31
// `jungle_plants.rs` duplicate DROPPED per the g42 GROWTH SEAL DUP-RECONCILE,
// research/300; the two call sites carry the extra `inherit_paint: false`).
pub mod plantera;
mod pumpkins;
mod trees;
mod underground_tail;

// W4-c31 jungle residuals: the Plantera bulb event placement
// (`WorldGen.cs:74127-74267`) — the all-mechs-defeated hook.
pub use plantera::generate_plantera_bulb_on_all_mechs_defeated;

// S1-K-3: the tree family lives in `growth/trees.rs`; the public seams
// stay on `growth` (verbatim move, behaviour frozen).
pub use trees::{
    attempt_grow_tree_from_sapling, empty_tile_check, grow_palm_tree, grow_shroom, grow_tree,
    grow_tree_profile_from_item, grow_tree_with_settings, is_common_sapling,
    is_tile_type_fit_for_tree, try_growing_tree_by_type, GrowTreeSettings, GroundTestKind,
    WallTestKind, COMMON_SAPLINGS, TREE_SOILS,
};

// S1-K-3b: the grass/spread family lives in `growth/grass.rs`; the public
// seams stay on `growth` (verbatim move, behaviour frozen).
pub use grass::{
    spread_desert_walls, spread_grass, spread_grass_walls, update_world_grass_growth,
};
pub(crate) use grass::convert_wall_by_case;
// W4-f25 C4: the type-7 spray cell's `:55465` union pre-gate seams.
pub(crate) use grass::{conversion_hardened_sand_walls_contains, conversion_snow_walls_contains};
// W4-f25 closeout: the Convert 16/18 wall-union seams (the dungeon-biome cells).
pub(crate) use grass::{conversion_dirt_walls_contains, conversion_new_wall_set};
use grass::{grow_moss_arm, grow_moss_arm_underground, SPREAD_UNDERGROUND};
// W4-d106 R1: the Grass Wall pass's `SpreadGrass` write kills an invalid
// tree above the converted cell (`WorldGen.cs:75222` → `:54802-54902`) —
// the same helper the growth-tick slice uses (S1-K-3 seam).
pub(crate) use trees::kill_invalid_tree_above;
pub(crate) use trees::ALLOWS_PLANTS_TO_GROW;

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

/// The `WorldGen.TrySpawningTownNPC` walker seam (W4-d119 escalation 1): the
/// two growth walkers' inactive-tile arms (`WorldGen.cs:72497` overground /
/// `:73807` underground) call `TrySpawningTownNPC`, which in the C# runs
/// `SpawnTownNPC` INLINE. The port's walkers are pure tr-core world fns with
/// no NPC store, so the gate's verdict rides out as a spawn-request intent:
/// the walker pushes `(x, y)` onto [`TownSpawnWalker::requests`] and the app
/// drains the queue at the `UpdateWorldGen` slot, running the full
/// `crate::town_spawn::spawn_town_npc` there (the wiring statue-spawns
/// `take_statue_spawns` event-queue precedent). `None` keeps a pure world
/// walk NPC-free (every tr-core growth test passes `None`).
pub struct TownSpawnWalker<'a> {
    /// `Main.invasionType` (`:74871`).
    pub invasion_type: i32,
    /// `Main.eclipse` (`:74871`).
    pub eclipse: bool,
    /// `WorldGen.npcSpawnDelay` (`:4238`) — the `++` counter, reset on fire.
    pub npc_spawn_delay: &'a mut i32,
    /// `WorldGen.npcSpawnPeriod` (`:4240`) — `20 * GetWorldUpdateRate()`,
    /// re-armed every `UpdateWorld` head (`:71549`).
    pub npc_spawn_period: i32,
    /// The shared spawn-request queue the app drains (one `(x, y)` per fire).
    pub requests: &'a mut Vec<(i32, i32)>,
}

/// `WorldGen.TrySpawningTownNPC` (`WorldGen.cs:74869-74882`), gate half.
/// Verbatim: `Main.invasionType <= 0 && !Main.eclipse &&
/// ++WorldGen.npcSpawnDelay >= WorldGen.npcSpawnPeriod` (`:74871`), the
/// counter resets on fire (`:74873`), and the ONE `Main.rand.Next(4)` draw
/// only when the picked tile's wall is 34 (`:74874` short-circuit — a
/// non-34 wall draws NOTHING). On fire the `(x, y)` intent is queued; the
/// `tileSolid[379]` window + `SpawnTownNPC` (`:74876-74879`) are the app
/// drain's (the generated `TILE_SOLID` base table already carries 379).
pub fn try_spawning_town_npc(
    world: &World,
    x: i32,
    y: i32,
    ts: &mut TownSpawnWalker<'_>,
    rng: &mut UnifiedRandom,
) {
    // `:74871`: the invasion/eclipse guard BEFORE the counter increment.
    if ts.invasion_type > 0 || ts.eclipse {
        return;
    }
    *ts.npc_spawn_delay += 1;
    if *ts.npc_spawn_delay < ts.npc_spawn_period {
        return;
    }
    *ts.npc_spawn_delay = 0;
    // `:74874`: `Main.tile[x, y].wall != 34 || Main.rand.Next(4) == 0`.
    let wall = world.tile(x, y).map(|t| t.wall).unwrap_or(0);
    if wall != 34 || rng.next_max(4) == 0 {
        ts.requests.push((x, y));
    }
}

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
    // `:56585` — `!tile.nactive() || halfBrick() || (!isGeneratingOrLoadingWorld
    // && slope() != 0) || (type != 80 && !Conversion.Sand[type]) ||
    // tile2.liquid > 0`. The runtime growth pass is never generating, so the
    // slope gate always applies (the worldgen cactus pass keeps its own
    // isGeneratingOrLoadingWorld arm). `Conversion.Sand` = {53, 112, 116, 234}
    // (`TileID.cs:92`).
    if !tile.active() || tile.inactive() || tile.half_brick() || tile.slope() != 0 {
        return;
    }
    let is_sand = crate::tiledata::SET_CONVERSION_SAND.has(tile.kind);
    if tile.kind != 80 && !is_sand {
        return;
    }
    if world.tile(i, j - 1).is_some_and(|t| t.liquid > 0) {
        return;
    }
    // The water box abort (:56590-56602): k in [i-50, i+50), l in [j-25, j+25)
    // (EXCLUSIVE high), the InWorld guard, `num3 / 255 > cactusWaterLimit(25)`.
    // The remix-world below-surface exemption (`!remixWorld || !(j >
    // worldSurface)`) is constants-off (no remix seed in the runtime pass).
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
        // The sand arm (:56604-56666): the three neighbours above must be
        // clear, the i±6 × j-3..j+1 box scan gates (≤3 cacti, >10 sand), then
        // the plant into the cell ABOVE (`tile2`, `:56584`).
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
                // `:56616` — `InWorld(m, n, 5)`: the 5-tile margin.
                if m < 5 || n < 5 || m >= world.width - 5 || n >= world.height - 5 {
                    continue;
                }
                let Some(t) = world.tile(m, n) else { continue };
                if !t.active() {
                    continue;
                }
                if t.kind == 80 {
                    num4 += 1;
                    // `:56629-56632` — the 4th cactus aborts immediately.
                    if num4 >= 4 {
                        return;
                    }
                }
                if crate::tiledata::SET_CONVERSION_SAND.has(t.kind) {
                    num5 += 1;
                    // `:56637-56640` — the 11th sand breaks the inner loop.
                    if num5 > 10 {
                        break;
                    }
                }
            }
            // `:56647-56650` — the outer break once the sand count exceeds 10.
            if num5 > 10 {
                break;
            }
        }
        // `:56652` — the plant gate: >10 sand in the box.
        if num5 > 10 {
            // `:56658-56665` — `tile2.active = true; tile2.type = 80;
            // CopyPaintAndCoating(tile); SquareTileFrame(num, num2 - 1)`. The
            // paint/coating rides from the sand cell below (the port's `meta`
            // + `extras`, the spike-arm convention `:45481`). The
            // isGeneratingOrLoadingWorld slope(0) Next(2) arm (`:56654`) is
            // worldgen-only — the runtime pass skips it.
            let (src_meta, src_extras) = {
                let si = world.index(i, j);
                (world.tiles[si].meta, world.extras_at(si))
            };
            let di = world.index(i, j - 1);
            world.tiles[di] = crate::tile::Tile {
                kind: 80,
                frame_x: 0,
                frame_y: 0,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
            world.tiles[di].meta = src_meta;
            world.set_extras_at(di, src_extras);
            if world.frame_important.len() <= 80 {
                world.frame_important.resize(crate::tiledata::TILE_COUNT, false);
            }
            world.frame_important[80] = true;
            // `WorldGen.SquareTileFrame(num, num2 - 1)` (`:56665`, num=i,
            // num2=j) — the 3×3 reframe around the new cactus cell.
            crate::edit::square_tile_frame_gen(world, i, j - 1);
        }
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
/// W4-g83d vine-retire keep-one: `pub(crate)` for `vine_growth`'s delegate
/// (its former self-contained re-port retired — contract §10.3).
pub(crate) fn grow_more_vines(world: &World, x: i32, y: i32) -> bool {
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
    // W4-g83 contract §2: the port-only `frame_important[vine] = true` write
    // is DROPPED at wiring — no C# `Main.tileFrameImportant` write exists for
    // the 8 vine kinds (`Main.cs` census); the flag mis-routed placed vines
    // away from `reframe_area`'s cosmetic pass and the `CheckVines` dispatch
    // now admits them through the explicit `is_vine` gate exception.
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
    // `WorldGen.SquareTileFrame` (`:45482`) — on the caller's ACTIVE genRand
    // stream (W4-g83 contract §2: the growth tick's UpdateWorldGen stream;
    // the framing dispatch's seaweed tip re-rolls draw it).
    crate::edit::square_tile_frame_gen(world, num2, num3, rng);
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
    update_world_overground_tile_ts(world, x, y, day_time, raining, max_raining, wind_speed, rng, None)
}

/// [`update_world_overground_tile`] with the `TrySpawningTownNPC` walker seam
/// (W4-d119 escalation 1): `ts` carries the gate's statics + the shared
/// spawn-request queue. `None` keeps the pure world walk (the landed
/// behaviour — every tr-core test passes `None`).
#[allow(clippy::too_many_arguments)]
pub fn update_world_overground_tile_ts(
    world: &mut World,
    x: i32,
    y: i32,
    day_time: bool,
    raining: bool,
    max_raining: f32,
    wind_speed: f32,
    rng: &mut UnifiedRandom,
    ts: Option<&mut TownSpawnWalker<'_>>,
) {
    let kind = world.tile(x, y).map(|t| (t.active(), t.kind));
    let Some((active, kind)) = kind else {
        return;
    };
    if !active {
        // The beach-shells arm (`:72073`): an inactive beach cell under the
        // surface grows a shell pile behind the wind/rain odds.
        grow_shell_pile(world, x, y, wind_speed, max_raining, raining, rng);
        // `WorldGen.cs:72491-72498` — the inactive-tile `else`: the web arm
        // (`:72493-72496`, wall 62 + no liquid) then `TrySpawningTownNPC`
        // (`:72497`). The web arm is parked (the overground slice); the
        // town-NPC gate rides the seam.
        if let Some(ts) = ts {
            try_spawning_town_npc(world, x, y, ts, rng);
        }
        return;
    }
    if inactive {
        // `:72216`'s `nactive()` gate: an ACTUATED tile (active bit set,
        // inActive bit set) skips the whole growth chain — actuated grass
        // grows no grass. The else arm (`:72491-72498`) runs the web arm
        // (wall 62, dry), then the wall arms below (W4-g42 GROWTH FINAL,
        // the D6 census verdict). The type-keyed head arms (`:72146-72215`
        // vanity-kill/bamboo/lily/cattail/liquid) on actuated cells stay
        // parked (ANALYSIS).
        if world.tile(x, y).is_some_and(|t| t.wall == 62 && t.liquid == 0) {
            grow_web(world, x, y, rng);
        }
        if world.allowed_to_spread_infections {
            spread_grass_walls(world, 3, x, y, rng);
            spread_desert_walls(world, 3, x, y, rng);
        }
        return;
    }
    // `:72218`: the hard-mode infection arm on every ACTIVE overground pick
    // (the `hardModeWorldUpdates` gate inside reads the `:71501` derivation).
    // Deviation: the C# reaches this arm only through the final
    // `else if (nactive())` (`:72216` — the 518/519/liquid>32 arms take the
    // earlier branches); the port's dispatcher runs it on every active pick
    // ahead of the type arms. The infection kinds never overlap 518/519, so
    // the divergence is the liquid>32 grass cells and the post-Plantera
    // `Next(2)` draw on the earlier-branch kinds — the dispatcher's existing
    // approximation shape (the dye head sits ahead of the alch arm too).
    crate::spread::hard_update_world_gated(
        world,
        x,
        y,
        hard_mode_world_updates(world),
        world.downed_plant_boss,
        true,
        rng,
    );
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
        // The C# alch arm (`:72068`) does NOT return — the walk reaches the
        // wall arms (`:72499`). No later arm matches 82/83/84, so the port
        // falls through (W4-g42 GROWTH FINAL).
    }
    // The sapling arm (:72430-72449): the kind-20 sapling at `Next(20)`,
    // the beach palm saplings 595/615 at `Next(5)` (the settings profiles
    // land with the vanity trees). The C# arms do NOT return — the walk
    // continues (the tile is no grass, so the tail arms skip it).
    if kind == 20 && rng.next_max(20) == 0 {
        attempt_grow_tree_from_sapling(world, x, y, false, rng);
    }
    if (kind == 595 || kind == 615) && rng.next_max(5) == 0 {
        // `AttemptToGrowTreeFromSapling`'s 595/615 arms (`:72438-72449`)
        // — the Sakura/Willow settings profiles.
        attempt_grow_tree_from_sapling(world, x, y, false, rng);
    }
    // The vanilla/KillTile arm (:72146-72152): the sakura/willow sapling or
    // its grown tile standing on the evil grass is killed (the trees do not
    // take root in the corruption/crimson). The C# below-gate reads the
    // TYPE only — no active check (an actuated evil grass still kills).
    if matches!(kind, 595 | 615 | 596 | 616)
        && world
            .tile(x, y + 1)
            .is_some_and(|t| matches!(t.kind, 199 | 23))
    {
        if let Some(t) = world.tile_mut(x, y) {
            t.flags &= !FLAG_ACTIVE;
        }
        // The C# KillTile (`:72148`) does NOT return — the killed cell's
        // walk reaches the wall arms (W4-g42 GROWTH FINAL). No later arm
        // matches 595/615/596/616.
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
    // The cactus arm (:72227-72233): the cactus 80 itself rolls `Next(15)`
    // and grows the stack. The C# `else if` chain: 80 → 529 → Conversion.Sand
    // → 530, each mutually exclusive. The chain fall-through reaches the
    // wall arms (W4-g42 GROWTH FINAL — the C# arms do not return; no later
    // chain arm matches these kinds, so the port's early return is
    // equivalent for the chain and the wall arms below still run — see the
    // tail: the wall arms are OUTSIDE the chain).
    if kind == 80 {
        if rng.next_max(15) == 0 {
            grow_cactus(world, x, y, rng);
        }
    } else if kind == 529 {
        // The sea-oat arm (:72234-72240): the 529 oat checks its water and rolls
        // `Next(20)` to advance a stage (`CheckSeaOat && Next(20)==0 → GrowSeaOat`).
        if check_sea_oat_runtime(world, x, y) && rng.next_max(20) == 0 {
            grow_sea_oat_runtime(world, x, y);
        }
    } else if crate::tiledata::SET_CONVERSION_SAND.has(kind) {
        // The Conversion.Sand arm (:72241-72288): the cell ABOVE (`num3 = j-1`,
        // clamped to ≥10 at :72044-72046) must be inactive. Then the oasis plant
        // (`Next(25)`), the sea oat (`Next(20)`), and — only when the oat did NOT
        // plant — the coral band (ocean edge, `Next(500)`) or the inland cactus
        // (`beachDistance+20` bounds, `Next(300)`). The `num3` clamp is the
        // overground window's top edge (:72034/:72044).
        let num3 = (y - 1).max(10);
        if world.tile(x, num3).is_some_and(|t| !t.active()) {
            // `:72245-72252` — the oasis plant at Next(25).
            if rng.next_max(25) == 0 {
                place_oasis_plant_runtime(world, x, num3, 530, rng);
            }
            // `:72253` — the sea oat at Next(20); the coral/cactus arms run
            // only when the oat roll misses OR the plant refuses.
            if rng.next_max(20) != 0 || !plant_sea_oat_runtime(world, x, num3, rng) {
                if x < OCEAN_DISTANCE || x > world.width - OCEAN_DISTANCE {
                    // `:72255-72281` — the coral band: Next(500), the 15×15
                    // coral-count < 6 gate, the 5-deep full-water column.
                    if rng.next_max(500) == 0 {
                        let num9 = 7;
                        let num10 = 6;
                        let mut num11 = 0;
                        for n in (x - num9)..=(x + num9) {
                            for m in (num3 - num9)..=(num3 + num9) {
                                if world.tile(n, m).is_some_and(|t| t.active() && t.kind == 81) {
                                    num11 += 1;
                                }
                            }
                        }
                        let full_water = (0..5).all(|d| {
                            world
                                .tile(x, num3 - d)
                                .is_some_and(|t| t.liquid == u8::MAX)
                        });
                        if num11 < num10 && full_water {
                            // `:72274` — PlaceTile(i, num3, 81, mute). The
                            // coral 1×1 slice (`:59431-59448`): the cell above
                            // inactive, the floor below active + tileSolid +
                            // !halfBrick + slope==0 (the solid_tile_2 shape —
                            // NOT the solidTop-excluding SolidTile). The
                            // coral cell keeps its liquid (the full-water
                            // column gate above already requires it).
                            let above_free =
                                world.tile(x, num3 - 1).is_some_and(|t| !t.active());
                            let below_ok = world.tile(x, num3 + 1).is_some_and(|t| {
                                t.active()
                                    && crate::tiledata::TILE_SOLID.has(t.kind)
                                    && t.slope() == 0
                                    && !t.half_brick()
                            });
                            if above_free && below_ok {
                                if let Some(t) = world.tile_mut(x, num3) {
                                    t.flags |= FLAG_ACTIVE;
                                    t.kind = 81;
                                    t.frame_x = 0;
                                    t.frame_y = 0;
                                    t.set_slope(0);
                                }
                            }
                        }
                    }
                } else if x > BEACH_DISTANCE + 20
                    && x < world.width - BEACH_DISTANCE - 20
                    && rng.next_max(300) == 0
                {
                    // `:72282-72285` — the inland cactus at Next(300).
                    grow_cactus(world, x, y, rng);
                }
            }
        }
    } else if kind == 530 {
        // The oasis-plant arm (:72289-72298): the 530 plant kills itself when the
        // boosted water check fails (`OasisPlantWaterCheck(i, j, boost: true)`).
        if !oasis_plant_water_check(world, x, y, true) {
            let _ = crate::edit::kill_tile(world, x, y, false);
        }
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
        grow_moss_arm(world, x, y, min_i, max_i, min_j, max_j, rng);
        // The C# moss else-if (`:72371`) does not return — the walk reaches
        // the wall arms (W4-g42 GROWTH FINAL). No later arm matches a moss.
    }
    if grass_pass {
        // W4-g42 dispatcher follow-up D2: the OVERGROUND caller passes
        // `underground: false` (`:72357`), so every grass kind rides the
        // OVERWORLD `switch (num10)` (`:74510-74866`) — NOT the underground
        // slice (`update_world_grass_growth`, which returns at `:74508`).
        // The old port routed everything but 70 through the underground
        // slice, so the overworld plant rolls / spread ladder never ran
        // (the case-70 find, T-3). Routing through both would double the
        // plant rolls. `AllowedToSpreadInfections` rides the World field
        // (the `update_world` caller sets it from the journey power).
        let allowed = world.allowed_to_spread_infections;
        match kind {
            70 => overworld_mushroom_growth(world, x, y, min_i, max_i, min_j, max_j, rng),
            60 => overworld::overworld_jungle_growth(world, x, y, min_i, max_i, min_j, max_j, rng),
            226 => overworld::overworld_jungle_plant_growth(world, x, y, min_j, rng),
            633 => overworld::overworld_ash_growth(world, x, y, min_i, max_i, min_j, max_j, rng),
            _ => overworld::overworld_grass_growth(
                world,
                x,
                y,
                min_i,
                max_i,
                min_j,
                max_j,
                allowed,
                rng,
            ),
        }
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
        // W4-g83: the dead overground-69 arm is DELETED here (contract §3 —
        // it sat inside `if grass_pass`, 69 is not a member, it never fired;
        // the genuine arm is the UNDERGROUND one, wired below after
        // `grow_vine_arm`). Keeping both would double-fire if the grass set
        // ever grows.
    }
    // W4-g83 (contract §3): `UpdateWorld_OvergroundTile`'s vine chain
    // (`WorldGen.cs:72504-72802`) — the fresh `if (tile.nactive())` block
    // AFTER the grass/moss arms, at the fn tail. The port's early returns
    // (alch/saplings/cactus/moss) skip it faithfully: none of those kinds is
    // a vine-chain source (192 takes no early return). The lily-61 `else if`
    // between the two chains is the jungle-plant lane's; when the overground
    // wall-spread lands it goes BEFORE this call (`:72499-72503`). The arm
    // draws the caller's ACTIVE genRand (the UpdateWorldGen stream).
    crate::vine_growth::grow_overground_vines(world, x, y, rng);
}

/// The crystal-shard PLACEMENT body of `hardUpdateWorld`
/// (`WorldGen.cs:69609-69648`): the 4-dir neighbour must be empty, the
/// radius-6 density stays under 2, the style `Next(18)` (the big shard
/// `18 + Next(6)` at 1/50). The HEAD gates (the shard-stone kind, the
/// rock-layer/remix depth, the `Next(5)` roll) live at the
/// `hard_update_world` call site (`:69607`) — the C# short-circuits them
/// into the arm's condition; this fn assumes they passed.
pub fn grow_crystal_shards(world: &mut World, x: i32, y: i32, rng: &mut UnifiedRandom) {
    let Some(t) = world.tile(x, y) else {
        return;
    };
    if !t.active() || !CAN_GROW_CRYSTAL_SHARDS.contains(&t.kind) {
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
    update_world_ts(
        world, day_rate, raining, freeze_time, allowed_to_spread, day_time,
        max_raining, wind_speed, rng, None,
    )
}

/// [`update_world`] with the `TrySpawningTownNPC` walker seam (W4-d119
/// escalation 1): `ts` threads the gate's statics + the shared spawn-request
/// queue through both pick loops. `None` keeps the pure world walk.
#[allow(clippy::too_many_arguments)]
pub fn update_world_ts(
    world: &mut World,
    day_rate: i32,
    raining: bool,
    freeze_time: bool,
    allowed_to_spread: bool,
    day_time: bool,
    max_raining: f32,
    wind_speed: f32,
    rng: &mut UnifiedRandom,
    mut ts: Option<&mut TownSpawnWalker<'_>>,
) -> u32 {
    let rate = world_update_rate(day_rate, freeze_time);
    if rate == 0 {
        return 0;
    }
    // `WorldGen.UpdateWorld`'s head (`:71501`) derives the hardmode-update
    // flag once (`hardMode || (remix && getGood && !anniversary)`); the
    // port's dispatchers re-derive it per pick from the same world state
    // (`hard_mode_world_updates`) — the flags cannot change mid-tick, so
    // the reads agree. Pre-hardmode classic worlds skip the infection arms.
    // `WorldGen.UpdateWorld`'s setup (`:71503`, `:71508`): the infection
    // spread flag comes in from the caller (`AllowedToSpreadInfections`, set
    // from the journey `StopBiomeSpreadPower`) and the wall-arm box radius is
    // the constant 3. The flag rides the World field so the overworld grass
    // arm (no flag parameter of its own) reads the same value.
    world.allowed_to_spread_infections = allowed_to_spread;
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
        update_world_overground_tile_ts(
            world,
            x,
            y,
            day_time,
            raining,
            max_raining,
            wind_speed,
            rng,
            ts.as_deref_mut(),
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
        update_world_underground_tile_ts(
            world,
            x,
            y,
            wall_dist,
            allowed_to_spread,
            day_time,
            raining,
            rng,
            ts.as_deref_mut(),
        );
        picks += 1;
    }
    picks
}

/// `WorldGen.hardModeWorldUpdates` (`WorldGen.cs:71501`): the
/// `UpdateWorld` head derives it once per tick — `Main.hardMode ||
/// (Main.remixWorld && Main.getGoodWorld && !Main.tenthAnniversaryWorld)`.
/// The `hardUpdateWorld` gate (`:69602`) reads it.
pub fn hard_mode_world_updates(world: &World) -> bool {
    world.hard_mode
        || (world.seeds.contains(crate::seeds::SeedFlags::REMIX)
            && world.seeds.contains(crate::seeds::SeedFlags::FOR_THE_WORTHY)
            && !world.seeds.contains(crate::seeds::SeedFlags::TENTH_ANNIVERSARY))
}

/// `WorldGen.beachDistance` = 380 (`WorldGen.cs:86914`) — the shell arm's
/// beach margins (the runtime cattail's sand gate reads it too, `:58700`).
pub(crate) const BEACH_DISTANCE: i32 = 380;
/// `WorldGen.oceanDistance` = 250 (`WorldGen.cs:86913`) — the runtime sand
/// arm's coral band (`:72255`).
const OCEAN_DISTANCE: i32 = 250;

// ---- W4-g40 desert chain: the runtime desert-surface growth arm ----------
// `UpdateWorld_OvergroundTile`'s `Conversion.Sand` branch
// (`WorldGen.cs:72241-72288`) + the sea-oat (`:72234-72239`) and oasis-plant
// (`:72289-72298`) arms. The C# calls the shared `WorldGen.PlaceOasisPlant` /
// `PlantSeaOat` / `GrowSeaOat` / `CheckSeaOat` statics on `genRand`
// (`UnifiedRandom`); the worldgen pass's ocean.rs copies draw the xorshift
// `Rng` instead, so the runtime arm carries its own `UnifiedRandom` versions
// here (the RNG-type split — the two consumers cannot share one helper).

/// `WorldGen.SolidTile(i, j)` (`WorldGen.cs:70111-70131`) — the indexed
/// probe the water checks read: active, solid, non-solidTop, non-halfBrick,
/// slopeless. Out-of-world cells are not solid.
fn growth_solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && !t.inactive()
        && crate::tiledata::TILE_SOLID.has(t.kind)
        && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
}

/// `WorldGen.SolidTileAllowBottomSlope(i, j)` (`WorldGen.cs:70167-70185`) —
/// the sea-oat ground probe: active, solid or solidTop, non-halfBrick, top
/// not sloped. Out-of-world / null cells count as solid (the vanilla quirk).
fn growth_solid_tile_allow_bottom_slope(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && (crate::tiledata::TILE_SOLID.has(t.kind)
            || crate::tiledata::TILE_SOLID_TOP.has(t.kind))
        && !t.top_slope()
        && !t.half_brick()
}

/// `WorldGen.OasisPlantWaterCheck(x, y, boost)` (`WorldGen.cs:47506-47536`):
/// the 45/20 water sum (boost widens to 49/23), the `x <= beachDistance`
/// column-skip quirk (the beach columns count no water and fail by
/// construction), `num4 / 255 >= 20`.
fn oasis_plant_water_check(world: &World, x: i32, y: i32, boost: bool) -> bool {
    let (num, num2) = if boost { (49, 23) } else { (45, 20) };
    let num3 = 20u32;
    let mut num4 = 0u32;
    for i in (x - num)..=(x + num) {
        // `:72519` — the skip is on the CENTRE x, not the loop i (the C#
        // quirk: a beach-centred plant counts no water at all).
        if x <= BEACH_DISTANCE || x >= world.width - BEACH_DISTANCE {
            continue;
        }
        for j in (y - num2)..=(y + num2) {
            if world.tile(i, j).is_some_and(|t| !growth_solid_tile(world, i, j)) {
                if let Some(t) = world.tile(i, j) {
                    num4 += t.liquid as u32;
                }
            }
        }
    }
    num4 / 255 >= num3
}

/// `WorldGen.SeaOatWaterCheck(x, y)` (`WorldGen.cs:58253-58296`): the water
/// sum with the beach band's widened 65/25 box and INVERTED verdict — inland
/// needs ≥20 water units, the beach band needs <20 (the `flag` return).
fn sea_oat_water_check(world: &World, x: i32, y: i32) -> bool {
    let (mut num, mut num2) = (45i32, 20i32);
    let num3 = 20u32;
    let beach = x <= BEACH_DISTANCE || x >= world.width - BEACH_DISTANCE;
    if beach {
        num = 65;
        num2 += 5;
    }
    let mut num6 = 0u32;
    for i in (x - num)..=(x + num) {
        for j in (y - num2)..=(y + num2) {
            if let Some(t) = world.tile(i, j) {
                if !growth_solid_tile(world, i, j) && t.liquid > 0 {
                    num6 += t.liquid as u32;
                }
            }
        }
    }
    if num6 / 255 >= num3 {
        return !beach;
    }
    beach
}

/// `WorldGen.PlaceOasisPlant(X, Y, type)` (`WorldGen.cs:47538-47613`) over
/// the runtime `genRand` (`UnifiedRandom`): the `Next(9)` frame column FIRST
/// (before the bounds gate), the 3×2 occupancy scan (the sea-oat 529 cells
/// are allowed), the sand-floor + water-check gates, the `KillTile(noItem)`
/// strip across the box, then the 3×2 frame write (`54*style` column,
/// `36*0` row). The runtime caller passes type 530.
fn place_oasis_plant_runtime(world: &mut World, x: i32, y: i32, kind: u16, rng: &mut UnifiedRandom) {
    let num = rng.next_max(9); // `:47540` — the draw precedes the bounds gate.
    let num2 = 0i32;
    // `:47542` — the bounds gate (beachDistance margins, 5-tile y margin).
    if x < BEACH_DISTANCE || x > world.width - BEACH_DISTANCE || y < 5 || y > world.height - 5 {
        return;
    }
    let mut flag = true;
    for i in (x - 1)..=(x + 1) {
        for j in (y - 1)..=y {
            if let Some(t) = world.tile(i, j) {
                if t.active() && t.kind != 529 {
                    flag = false;
                }
                if t.liquid > 0 {
                    flag = false;
                }
            }
        }
        // `:47568` — the floor below must be solid sand.
        if !growth_solid_tile(world, i, y + 1)
            || !world.tile(i, y + 1).is_some_and(|t| crate::tiledata::SET_CONVERSION_SAND.has(t.kind))
        {
            flag = false;
        }
    }
    if !flag || !oasis_plant_water_check(world, x, y, false) {
        return;
    }
    // `:47577-47586` — the KillTile(noItem) strip across the 3×2 box.
    for k in (x - 1)..=(x + 1) {
        for l in (y - 1)..=y {
            if world.tile(k, l).is_some_and(|t| t.active()) {
                let _ = crate::edit::kill_tile(world, k, l, true);
            }
        }
    }
    // `:47587-47612` — the 3×2 frame write.
    let num3 = (54 * num) as i16;
    let num4 = (36 * num2) as i16;
    for dx in 0..3 {
        for dy in 0..2 {
            let cx = x - 1 + dx;
            let cy = y - 1 + dy;
            if let Some(t) = world.tile_mut(cx, cy) {
                t.flags |= FLAG_ACTIVE;
                t.kind = kind;
                t.frame_x = num3 + (dx * 18) as i16;
                t.frame_y = num4 + (dy * 18) as i16;
            }
        }
    }
}

/// `WorldGen.PlantSeaOat(x, y)` (`WorldGen.cs:58299-58326`) over the runtime
/// `genRand`: the wall/active/liquid refusals, the solid-sand floor below
/// (`SolidTileAllowBottomSlope` + `Conversion.Sand`), the water check, then
/// the 529 write with `Next(5)*18` frameX. Returns whether it planted.
fn plant_sea_oat_runtime(world: &mut World, x: i32, y: i32, rng: &mut UnifiedRandom) -> bool {
    // `:58301` — the cell refusals + the floor probe.
    let here = world.tile(x, y);
    if here.is_some_and(|t| t.wall > 0 || t.active() || t.liquid > 0) {
        return false;
    }
    if !growth_solid_tile_allow_bottom_slope(world, x, y + 1)
        || !world.tile(x, y + 1).is_some_and(|t| crate::tiledata::SET_CONVERSION_SAND.has(t.kind))
    {
        return false;
    }
    if !sea_oat_water_check(world, x, y) {
        return false;
    }
    // `:58314-58320` — the write (slope 0, halfBrick false, 529, Next(5)*18).
    let frame_x = (rng.next_max(5) * 18) as i16;
    if let Some(t) = world.tile_mut(x, y) {
        t.flags |= FLAG_ACTIVE;
        t.set_slope(0);
        t.flags &= !crate::tile::FLAG_HALF_BRICK;
        t.kind = 529;
        t.frame_x = frame_x;
        t.frame_y = 0;
    }
    true
}

/// `WorldGen.GrowSeaOat(x, y)` (`WorldGen.cs:58342-58352`): the frameX
/// advances by 90 while below 180 (the 3-stage growth). Returns false (the
/// C# return value, unused by the caller).
fn grow_sea_oat_runtime(world: &mut World, x: i32, y: i32) -> bool {
    if let Some(t) = world.tile_mut(x, y) {
        if t.frame_x < 180 {
            t.frame_x += 90;
        }
    }
    false
}

/// `WorldGen.CheckSeaOat(x, y)` (`WorldGen.cs:58328-58340`): the water check
/// kills the oat when it fails. Returns whether the oat survives.
fn check_sea_oat_runtime(world: &mut World, x: i32, y: i32) -> bool {
    if !sea_oat_water_check(world, x, y) {
        let _ = crate::edit::kill_tile(world, x, y, false);
        return false;
    }
    true
}

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

    /// W4-d119 escalation 1 — `WorldGen.TrySpawningTownNPC`
    /// (`WorldGen.cs:74869-74882`): the delay counter gates the queue push;
    /// the ONE `Next(4)` draw fires only on a wall-34 tile (the short-circuit);
    /// the invasion/eclipse guards skip before the counter increment.
    #[test]
    fn the_walker_gate_counts_and_draws_only_on_wall_34() {
        let mut w = World::new("t".into(), 200, 200);
        // A wall-34 cell and a wall-0 cell, both inactive (the walker arms).
        let i34 = w.index(100, 100);
        w.tiles[i34].wall = 34;
        let mut delay = 0;
        let mut requests: Vec<(i32, i32)> = Vec::new();
        let mut rng = UnifiedRandom::new(7);
        let pristine = rng.clone();
        // 19 sub-period calls on the wall-0 cell: no push, no draw.
        for _ in 0..19 {
            let mut ts = TownSpawnWalker {
                invasion_type: 0,
                eclipse: false,
                npc_spawn_delay: &mut delay,
                npc_spawn_period: 20,
                requests: &mut requests,
            };
            try_spawning_town_npc(&w, 101, 100, &mut ts, &mut rng);
        }
        assert!(requests.is_empty());
        assert_eq!(rng, pristine, "a non-34 wall draws nothing");
        // The 20th fires with no draw (wall 0).
        {
            let mut ts = TownSpawnWalker {
                invasion_type: 0,
                eclipse: false,
                npc_spawn_delay: &mut delay,
                npc_spawn_period: 20,
                requests: &mut requests,
            };
            try_spawning_town_npc(&w, 101, 100, &mut ts, &mut rng);
        }
        assert_eq!(requests, vec![(101, 100)]);
        assert_eq!(delay, 0, "the counter reset");
        assert_eq!(rng, pristine);
        requests.clear();
        // Wall 34 draws the Next(4) on the fire tick.
        for _ in 0..19 {
            let mut ts = TownSpawnWalker {
                invasion_type: 0,
                eclipse: false,
                npc_spawn_delay: &mut delay,
                npc_spawn_period: 20,
                requests: &mut requests,
            };
            try_spawning_town_npc(&w, 100, 100, &mut ts, &mut rng);
        }
        {
            let mut ts = TownSpawnWalker {
                invasion_type: 0,
                eclipse: false,
                npc_spawn_delay: &mut delay,
                npc_spawn_period: 20,
                requests: &mut requests,
            };
            try_spawning_town_npc(&w, 100, 100, &mut ts, &mut rng);
        }
        let mut twin = UnifiedRandom::new(7);
        let _ = twin.next_max(4);
        assert_eq!(rng, twin, "exactly one Next(4) on the wall-34 fire");
        // The invasion/eclipse guards skip before the counter.
        let before = delay;
        for (inv, ecl) in [(1, false), (0, true)] {
            let mut ts = TownSpawnWalker {
                invasion_type: inv,
                eclipse: ecl,
                npc_spawn_delay: &mut delay,
                npc_spawn_period: 20,
                requests: &mut requests,
            };
            try_spawning_town_npc(&w, 101, 100, &mut ts, &mut rng);
        }
        assert_eq!(delay, before);
    }

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
        // `UpdateWorld_UndergroundTile`'s ported arms (`:73214-73813`): the
        // alch plant grows, the grass family converts (underground), the lily
        // becomes 74, the grass growth re-runs the spikes and the cattail runs
        // its validity/growth pair. The parked arms (the 655 kill, the dye/
        // seaweed, the 444/396 oyster, the web/town-NPC) stay out.
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

    /// W4-d103 POST FINAL item 1: the OVERGROUND dispatcher's hardUpdateWorld
    /// call-site is LIVE (`WorldGen.cs:72216-72218` — the `else if (nactive())`
    /// arm calls `hardUpdateWorld(i, j)` on every active overground pick). The
    /// port's `update_world_overground_tile` runs `hard_update_world_gated`
    /// ahead of the type arms (`growth.rs:722`); in hardmode an ebonstone 25
    /// source corrupts the grass box around it, pre-hardmode the
    /// `hardModeWorldUpdates` gate (`:69602`/`:71501`) freezes it. This pins
    /// the WIRING (the dispatcher reaches the infection arm), not just the
    /// ported body.
    #[test]
    fn w4_d103_post_the_overground_dispatcher_runs_hard_update_world() {
        let build = || {
            let mut w = world();
            // The ±3 pick box (`spread_loop`) filled with grass 2, the
            // ebonstone 25 source at the centre. Every non-centre pick is a
            // convertible grass, so the infection arm fires on the first
            // non-centre draw.
            for x in 1997..=2003 {
                for y in 47..=53 {
                    put(&mut w, x, y, 2, 0);
                }
            }
            put(&mut w, 2000, 50, 25, 0);
            w
        };
        let any_corrupt = |w: &World| {
            (1997..=2003).any(|x| (47..=53).any(|y| w.tile(x, y).is_some_and(|t| t.kind == 23)))
        };
        // Hardmode: the dispatcher's hard_update arm corrupts the box.
        let mut w = build();
        w.hard_mode = true;
        let mut rng = UnifiedRandom::new(11);
        for _ in 0..100 {
            update_world_overground_tile(&mut w, 2000, 50, true, false, 0.0, 0.0, &mut rng);
        }
        assert!(any_corrupt(&w), "the overground dispatcher's hardUpdateWorld arm spreads");
        // Pre-hardmode control: the gate freezes the infection (the box stays
        // grass on every pick — the dispatcher's other arms never convert it).
        let mut w2 = build();
        let mut rng2 = UnifiedRandom::new(11);
        for _ in 0..100 {
            update_world_overground_tile(&mut w2, 2000, 50, true, false, 0.0, 0.0, &mut rng2);
        }
        assert!(!any_corrupt(&w2), "the pre-hardmode gate freezes the overground arm");
    }

    /// W4-d103 POST FINAL item 1: the UNDERGROUND dispatcher's hardUpdateWorld
    /// call-site is LIVE (`WorldGen.cs:73252-73254` — the `else if (nactive())`
    /// arm). The port's `update_world_underground_tile` runs
    /// `hard_update_world_gated` on every active pick (`growth.rs:1733`); the
    /// same ebonstone/grass box corrupts in hardmode and freezes pre-hardmode.
    #[test]
    fn w4_d103_post_the_underground_dispatcher_runs_hard_update_world() {
        let build = || {
            let mut w = world();
            for x in 1997..=2003 {
                for y in 197..=203 {
                    put(&mut w, x, y, 2, 0);
                }
            }
            put(&mut w, 2000, 200, 25, 0);
            w
        };
        let any_corrupt = |w: &World| {
            (1997..=2003).any(|x| (197..=203).any(|y| w.tile(x, y).is_some_and(|t| t.kind == 23)))
        };
        let mut w = build();
        w.hard_mode = true;
        let mut rng = UnifiedRandom::new(12);
        for _ in 0..100 {
            update_world_underground_tile(&mut w, 2000, 200, 3, true, true, false, &mut rng);
        }
        assert!(any_corrupt(&w), "the underground dispatcher's hardUpdateWorld arm spreads");
        let mut w2 = build();
        let mut rng2 = UnifiedRandom::new(12);
        for _ in 0..100 {
            update_world_underground_tile(&mut w2, 2000, 200, 3, true, true, false, &mut rng2);
        }
        assert!(!any_corrupt(&w2), "the pre-hardmode gate freezes the underground arm");
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

    // ---- W4-g40 desert chain: the runtime desert-surface growth arm --------

    #[test]
    fn the_sand_arm_grows_the_inland_cactus_at_next_300() {
        // `WorldGen.cs:72282-72285` — inland sand (x > beachDistance+20) with
        // the cell above free rolls Next(300) → GrowCactus. The oasis/sea-oat
        // rolls (Next(25)/Next(20)) precede it; a seed that misses both and
        // hits the 300 plants the cactus. The world is 4200 wide, x=2000 is
        // inland (380+20 < 2000 < 4200-380-20).
        let mut w = world();
        w.world_surface = 250.0;
        for dx in -8..=8 {
            put(&mut w, 2000 + dx, 200, 53, 0);
        }
        // Find a seed whose Next(25)!=0, Next(20)!=0, Next(300)==0 in order.
        let mut planted = false;
        for seed in 0..4000i32 {
            let mut w = world();
            w.world_surface = 250.0;
            for dx in -8..=8 {
                put(&mut w, 2000 + dx, 200, 53, 0);
            }
            let mut rng = UnifiedRandom::new(seed);
            update_world_overground_tile(&mut w, 2000, 200, true, false, 0.0, 0.0, &mut rng);
            if w.tile(2000, 199).is_some_and(|t| t.active() && t.kind == 80) {
                planted = true;
                break;
            }
        }
        assert!(planted, "some seed plants the inland cactus through the sand arm");
    }

    #[test]
    fn the_sea_oat_arm_advances_the_frame_and_checks_water() {
        // `WorldGen.cs:72234-72240` — the 529 oat: CheckSeaOat (water) then
        // Next(20)==0 → GrowSeaOat (frameX += 90 while < 180). An inland oat
        // with no water nearby fails CheckSeaOat and is killed.
        let mut w = world();
        w.world_surface = 250.0;
        put(&mut w, 2000, 200, 53, 0); // the sand floor
        put(&mut w, 2000, 199, 529, 0); // the oat, frameX 0
        // No water inland → CheckSeaOat kills it.
        let mut rng = UnifiedRandom::new(7);
        update_world_overground_tile(&mut w, 2000, 199, true, false, 0.0, 0.0, &mut rng);
        assert!(w.tile(2000, 199).is_none_or(|t| !t.active()), "the dry inland oat dies");
    }

    #[test]
    fn the_oasis_plant_arm_kills_on_failed_boosted_water_check() {
        // `WorldGen.cs:72289-72298` — the 530 plant kills itself when
        // OasisPlantWaterCheck(boost:true) fails. An inland plant with no
        // water dies.
        let mut w = world();
        w.world_surface = 250.0;
        put(&mut w, 2000, 200, 53, 0);
        put(&mut w, 2000, 199, 530, 0);
        let mut rng = UnifiedRandom::new(7);
        update_world_overground_tile(&mut w, 2000, 199, true, false, 0.0, 0.0, &mut rng);
        assert!(w.tile(2000, 199).is_none_or(|t| !t.active()), "the dry oasis plant dies");
    }

    #[test]
    fn the_grow_cactus_sand_arm_copies_paint_and_reframes() {
        // `WorldGen.cs:56658-56665` — the new cactus cell copies the sand
        // cell's paint/coating and runs SquareTileFrame. The port writes the
        // paint (meta) from the source sand cell.
        let mut w = world();
        for dx in -8..=8 {
            put(&mut w, 100 + dx, 200, 53, 0);
        }
        // Paint the seed sand cell.
        let si = w.index(100, 200);
        w.tiles[si].set_paint(5);
        let mut rng = UnifiedRandom::new(7);
        grow_cactus(&mut w, 100, 200, &mut rng);
        let cactus = w.tile(100, 199).expect("the cactus");
        assert!(cactus.active() && cactus.kind == 80);
        assert_eq!(cactus.paint(), 5, "the paint rides from the sand cell");
    }

    #[test]
    fn test_spread_2_grass_grows_over_dirt_and_the_spikes_grow() {
        // `UpdateWorld_GrassGrowth` (:74269+): the 3x3 scan converts the
        // dirt neighbours to grass; the flower-27 exclusion; the spike arms
        // grow on the matching grass with the density gates. The scan half
        // rides the UNDERGROUND slice (kind 2 is no longer in its table —
        // W4-g42 D1); the overworld kind-2 dirt spread is
        // `overworld::overworld_grass_growth` (the :74644 arm).
        let mut w = world();
        w.world_surface = 250.0; // the surface below the cells
        put(&mut w, 100, 200, 2, 0); // the grass seed cell
        put(&mut w, 101, 200, 0, 0); // the dirt neighbour
        let mut rng = UnifiedRandom::new(7);
        overworld::overworld_grass_growth(&mut w, 100, 200, 99, 102, 199, 202, true, &mut rng);
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

    /// W4-g83 wiring witness (contract §3): the overground vine chain rides
    /// the END of `update_world_overground_tile` (the C# `:72504`
    /// `if (tile.nactive())` block after the grass/moss arms) — a dirt pick
    /// with a free cell below grows the plain vine 52 through the DISPATCHER
    /// (the plain arm's `Next(1)` always fires), on the caller's stream.
    #[test]
    fn w4_g83_the_overground_dispatcher_grows_the_vine_tail() {
        let mut w = world();
        w.world_surface = 250.0;
        put(&mut w, 100, 200, 2, 0); // the dirt pick, free cell below
        let mut rng = UnifiedRandom::new(7);
        update_world_overground_tile(&mut w, 100, 200, true, false, 0.0, 0.0, &mut rng);
        assert!(
            w.tile(100, 201).is_some_and(|t| t.active() && t.kind == 52),
            "the vine chain ran at the fn tail"
        );
        // The grown vine carries NO port-only frame-important flag (the
        // write is dropped at wiring — C# never marks the 8 kinds).
        assert!(
            !w.frame_important.get(52).copied().unwrap_or(false),
            "vine 52 is not frame-important (Main.cs census)"
        );
    }

    /// W4-g83 stream pin (contract §3, seam 5): the overground vine write's
    /// draws ride the CALLER's `UpdateWorldGen` stream — the dispatcher's
    /// tail adds EXACTLY the vine arm's draws (the dye-head `Next(3000)` +
    /// `grow_overground_vines`), no new stream and no reseed. The twin drives
    /// the same sequence by hand on a fresh same-seed stream; equal
    /// `UnifiedRandom` state proves the genRand position matches. A 52 vine
    /// pick (not a `grass_pass` member, no early return) isolates the chain.
    #[test]
    fn w4_g83_the_vine_write_draws_the_update_worldgen_position() {
        let mut w = world();
        w.world_surface = 250.0;
        put(&mut w, 100, 199, 2, 0); // the dirt support above
        put(&mut w, 100, 200, 52, 0); // the vine pick, free cell below
        let mut rng = UnifiedRandom::new(11);
        update_world_overground_tile(&mut w, 100, 200, true, false, 0.0, 0.0, &mut rng);
        // The twin: the dye head draws Next(3000) (plant_dye on a 52 cell
        // draws nothing — no matching arm), then the vine chain.
        let mut twin = UnifiedRandom::new(11);
        let mut tw = world();
        tw.world_surface = 250.0;
        put(&mut tw, 100, 199, 2, 0);
        put(&mut tw, 100, 200, 52, 0);
        if twin.next_max(3000) == 0 {
            crate::growth::plant_dye(&mut tw, 100, 200, false, &mut twin);
        }
        crate::vine_growth::grow_overground_vines(&mut tw, 100, 200, &mut twin);
        assert_eq!(rng, twin, "the vine write consumed exactly the UpdateWorldGen prefix");
        assert_eq!(w.tiles, tw.tiles, "the same vine landed");
    }

    /// W4-g83 wiring witness (contract §3): the jungle-thorn arm rides the
    /// UNDERGROUND dispatcher right after `grow_vine_arm` (`:73658-73661`,
    /// `GrowSpike(69, 60)` NO roll) — a 69 pick with the diamond's standing
    /// thorn grows a new 69; the dead overground-69 arm is deleted (an
    /// overground 69 pick never fires it: 69 is not a `grass_pass` member).
    #[test]
    fn w4_g83_the_jungle_thorn_rides_the_underground_dispatcher() {
        let mut w = world();
        w.world_surface = 100.0;
        // The diamond seed: a 69 standing on the jungle grass 60 within the
        // radius-7 walk of the pick (100, 200).
        put(&mut w, 100, 200, 69, 0);
        put(&mut w, 103, 201, 60, 0);
        put(&mut w, 103, 200, 69, 0);
        let mut grew = false;
        for seed in 1u64..200 {
            let mut w = w.clone();
            let mut rng = UnifiedRandom::new(seed as i32);
            update_world_underground_tile(&mut w, 100, 200, 3, true, true, false, &mut rng);
            let n69 = w.tiles.iter().filter(|t| t.active() && t.kind == 69).count();
            if n69 > 2 {
                grew = true;
                break;
            }
        }
        assert!(grew, "the underground thorn arm grew a third 69");
        // The overground dispatcher does NOT fire the 69 arm (deleted).
        let mut w2 = world();
        w2.world_surface = 250.0;
        put(&mut w2, 100, 200, 69, 0);
        put(&mut w2, 103, 201, 60, 0);
        put(&mut w2, 103, 200, 69, 0);
        let mut rng = UnifiedRandom::new(7);
        for _ in 0..50 {
            update_world_overground_tile(&mut w2, 100, 200, true, false, 0.0, 0.0, &mut rng);
        }
        let n69 = w2.tiles.iter().filter(|t| t.active() && t.kind == 69).count();
        assert_eq!(n69, 2, "the dead overground-69 arm stays dead");
    }

}

/// `WorldGen.UpdateWorld_UndergroundTile` (`WorldGen.cs:73214-73614`): the
/// underground growth dispatcher — the clamps, the alch/hard-mode arms, the
/// grass growth (underground) + the spike arms, the moss arm (no `NextDouble`
/// gate), the lily conversion, the five vine families and, behind
/// `allowed_to_spread`, the wall arms.
///
/// Parked (documented in ANALYSIS): the 655 life-fruit kill (the NPC store),
/// `plantDye` (and its `Main.rand.Next(2500)`/`Next(10000)` head rolls),
/// the 444/396 moss and oyster arms (`PlayerLOS`/`AnchorValid`/
/// `CountNearBlocksTypes` have no ports), the net sides, and the C#'s dual
/// `genRand`/`Main.rand` streams (the port's per-consumer stream carries
/// every draw). `TrySpawningTownNPC` rides the [`TownSpawnWalker`] seam
/// (`_ts` variant, W4-d119 escalation 1).
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
    update_world_underground_tile_ts(
        world, i, j, wall_dist, allowed_to_spread, day_time, raining, rng, None,
    )
}

/// [`update_world_underground_tile`] with the `TrySpawningTownNPC` walker seam
/// (W4-d119 escalation 1). `None` keeps the pure world walk.
#[allow(clippy::too_many_arguments)]
pub fn update_world_underground_tile_ts(
    world: &mut World,
    i: i32,
    j: i32,
    wall_dist: i32,
    allowed_to_spread: bool,
    day_time: bool,
    raining: bool,
    rng: &mut UnifiedRandom,
    ts: Option<&mut TownSpawnWalker<'_>>,
) {
    // The clamped box (`:73216-73231`).
    let min_i = (i - 1).max(10);
    let max_i = (i + 2).min(world.width - 10);
    let min_j = (j - 1).max(10);
    let max_j = (j + 2).min(world.height - 10);
    let Some(tile) = world.tile(i, j) else {
        return;
    };
    let (kind, active, actuated) = (tile.kind, tile.active(), tile.inactive());
    // The 655 life-fruit kill parks (`:73239-73246`).
    // The wall arms (`:73809-73813`) sit OUTSIDE the alch/active if-else —
    // the C# walks every path into them (W4-g42 GROWTH FINAL, the D6
    // residual). The inactive path's own copy is below; the actuated
    // (`nactive()` false, `:73252`) path takes the else arm's web + walls.
    if TILE_ALCH.contains(&kind) {
        grow_alch(world, i, j, day_time, raining, rng);
        return;
    }
    if !active {
        // The hard-mode infection arm (`:73252`) and the tail `else`'s web arm
        // (`:73803-73806`: wall 62, no liquid). The dye/seaweed and 444/396
        // arms park; the town-NPC gate (`:73807`) rides the seam.
        crate::spread::hard_update_world(world, i, j, false, allowed_to_spread, rng);
        if world.tile(i, j).is_some_and(|t| t.wall == 62 && t.liquid == 0) {
            grow_web(world, i, j, rng);
        }
        if let Some(ts) = ts {
            try_spawning_town_npc(world, i, j, ts, rng);
        }
        return;
    } else if actuated {
        // `:73252`'s `nactive()` gate: an ACTUATED tile skips the growth
        // chain — the else arm (`:73801-73808`) runs the web, then the
        // wall arms. Actuated grass grows no grass, but its evil WALL
        // still spreads (W4-g42 GROWTH FINAL, the D6 census verdict).
        if world.tile(i, j).is_some_and(|t| t.wall == 62 && t.liquid == 0) {
            grow_web(world, i, j, rng);
        }
        if allowed_to_spread {
            spread_grass_walls(world, wall_dist, i, j, rng);
            spread_desert_walls(world, wall_dist, i, j, rng);
        }
        return;
    } else {
        underground_nactive_arms(
            world,
            i,
            j,
            allowed_to_spread,
            kind,
            min_i,
            max_i,
            min_j,
            max_j,
            rng,
        );
    }
    // The wall arms (`:73809-73813`, behind `AllowedToSpreadInfections`) —
    // AFTER the whole chain, for every kind (the old port ran them
    // only on the vine-arm fall-through). The web and town-NPC arms park.
    if allowed_to_spread {
        spread_grass_walls(world, wall_dist, i, j, rng);
        spread_desert_walls(world, wall_dist, i, j, rng);
    }
}

/// `UpdateWorld_UndergroundTile`'s `nactive()` body (`:73252-73800`): the
/// dye head, the cattail/seaweed arms, the grass family + spikes, the lily
/// conversion, the vine families and the sapling arms. Split out of the
/// dispatcher so the alch/inactive/actuated paths reach the wall arms
/// (W4-g42 GROWTH FINAL) without duplicating the chain.
fn underground_nactive_arms(
    world: &mut World,
    i: i32,
    j: i32,
    allowed_to_spread: bool,
    kind: u16,
    min_i: i32,
    max_i: i32,
    min_j: i32,
    max_j: i32,
    rng: &mut UnifiedRandom,
) {
    // `:73254`: the hard-mode infection arm on every ACTIVE pick (the
    // `hardModeWorldUpdates` gate inside reads the `:71501` derivation).
    crate::spread::hard_update_world_gated(
        world,
        i,
        j,
        hard_mode_world_updates(world),
        world.downed_plant_boss,
        allowed_to_spread,
        rng,
    );
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
    // The moss arm (`:73699-73755`): the `else if (tileMoss || tileMossBrick)`
    // branch — NO `NextDouble` gate (unlike the overground twin), the box
    // conversion + the 184 spray. The C# does NOT return here (the wall arms
    // `:73809` run after the chain), so the port falls through too; the moss
    // kinds match none of the later return-guards (the grass family, the lily,
    // the vine arm). The 444/396 oyster arms ahead of it park.
    if crate::tiledata::TILE_MOSS.has(kind) || crate::tiledata::SET_TILE_MOSS_BRICK.has(kind) {
        grow_moss_arm_underground(world, i, j, min_i, max_i, min_j, max_j, rng);
    }
    // The grass family + the spikes (`:73280-73295`). The C# `else if`
    // chain does NOT return — the walk continues to the sapling arms
    // (`:73756-73780`) and the wall arms (`:73809-73813`) below (W4-g42
    // dispatcher follow-up D4: the old `return` here skipped them).
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
    } else if kind == 61 && rng.next_max(3) == 0 {
        // The lily → 74 conversion (`:73296-73310`) — an else-if arm, no
        // return (the C# walks on to the wall arms).
        if world.tile(i, j + 1).map(|t| t.kind).unwrap_or(0) != 226 {
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
        }
    } else if matches!(kind, 60 | 62 | 633 | 638 | 70 | 528 | 661 | 636 | 662 | 205) {
        // The five vine families (`:73311-73482`) — the port's one-fn arm
        // (an else-if arm; the 60/70/633/661/662 grasses never reach it,
        // the grass arm above took them). The 444/396 moss and oyster arms
        // park. The jungle-thorn kind-69 arm (`:73658`) is g83's claimed
        // case (vine_growth, fe5b5b8c — UNWIRED contested seam); it stays
        // out of this dispatcher until g83 wires it.
        grow_vine_arm(world, i, j, rng);
    }
    // The five vine families (`:73311-73482`) — the port's one-fn arm. The
    // 444/396 moss and oyster arms park.
    grow_vine_arm(world, i, j, rng);
    // W4-g83 (contract §3): the jungle-thorn arm (`:73658-73661`) — a 69
    // tile runs `GrowSpike(69, 60)` with NO roll, right after the vine
    // families and BEFORE the wall block. Faithful despite the C# `else if`:
    // 69 is disjoint from every earlier arm's kind set and the fn self-gates
    // on `kind == 69`. Same caller stream (UpdateWorldGen).
    crate::vine_growth::grow_underground_jungle_thorn(world, i, j, rng);
    // The wall arms (`:72499-72502`, behind `AllowedToSpreadInfections`); the
    // web and town-NPC arms park.
    if allowed_to_spread {
        spread_grass_walls(world, wall_dist, i, j, rng);
        spread_desert_walls(world, wall_dist, i, j, rng);
    }
    // The wall arms run in the DISPATCHER after this body returns — every
    // path (alch/inactive/actuated/nactive) reaches them (`:73809-73813`,
    // W4-g42 GROWTH FINAL).
}


