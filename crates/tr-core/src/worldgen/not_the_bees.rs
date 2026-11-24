//! The not-the-bees world-conversion machinery (W4-g41 follow-up): the
//! `NotTheBees` hive-cluster conversion (`WorldGen.cs:25107-25360`) and its
//! `FinishNotTheBees` host (`:25388-25797`), dispatched from `Finish()` at
//! `:22435-22439` BETWEEN `FinishDrunkGen` and `FinishGetGoodWorld` — the
//! `pipeline.rs::step_seed_finish` slot this module fills. The host wires the
//! W4-g41 `spider_wall_remover` (`spider_gem_moss.rs`, `:77279-77344`) at its
//! two C# search-loop call sites (`:25420`, `:25433`).
//!
//! Also ported here: `Spread.Wall2` (`:3416-3524`, the DST spider-web flood)
//! and `OverGrownDungeon` (`:23971-24035`, the host's `!remix` tail). Census:
//! `docs/research/94-not-the-bees-host-census.md`.
//!
//! Deviations from the C#, all deliberate and pinned here (the
//! `finish_seeds.rs` / `spider_gem_moss.rs` precedent):
//!
//! - **`SkipOnDualDungeons` reads false** (`:25089-25105`): the port has no
//!   `InAnyPotentialDungeonBounds` model (the `ocean.rs:620` /
//!   `mini_biomes.rs:1902` precedent). The `dualDungeons.Enabled` paint-14
//!   arms (`:25191`, `:25297`) still fire on the micro flag — they need no
//!   bounds.
//! - **`PlaceTile`-shaped writes use the port's single-tile idiom**
//!   (`spider_gem_moss.rs:54`): kind + active + `frame_x/y = -1` (the framer
//!   recomputes at the pipeline end). The C# plant-arm RNG schedule
//!   (`:59507-59620`, `IsFitToPlaceFlowerIn` / `NextFromList`) is NOT modelled
//!   — the port has no plant engine. The larva 231 rides a verbatim
//!   `Place3x3` (`:53534-53634`, draw-free).
//! - **`TileFrame(571)` / `SquareTileFrame` ride the post-pipeline framing
//!   seam** (the M5-STRUCT-5 idiom) — no in-pass framing.
//! - **The `tileSolid[379]` window** (`:25394` false / `:25792` true): the
//!   port's `TILE_SOLID` boot table is immutable, so the in-window solid
//!   tests veto 379 explicitly (`bubble_solid = false`). `OverGrownDungeon`
//!   runs after the restore; its only `SolidTile3` use is the `dualDungeons`
//!   disjunct (`:23996`), which reads false in the port, so no post-restore
//!   solid test is needed.
//! - **The tree-growth bridge**: `growth::{grow_tree, try_growing_tree_by_type}`
//!   take `UnifiedRandom`; the host feeds them ONE `UnifiedRandom` seeded from
//!   its own pass stream (the `micro_surface.rs:419` `start_in_hardmode`
//!   precedent). The C# shares `genRand`; the port's tree engine is
//!   `UnifiedRandom`-only, so the bridge is a documented deviation (draw
//!   ORDER faithful, byte stream not — the worldgen contract).
//! - **The dungeon side is fixed Right** (M5-BIOME, `pipeline.rs:294`): the
//!   `dungeonSide <= Left` arms read false, `>= Right` read true.
//!   `OverGrownDungeon`'s two anchor draws both run (the C# draws the Left
//!   anchor then overwrites it on Right).
//! - **Sub-vanilla band guards**: the remover search bands and the
//!   `OverGrownDungeon` anchors collapse on tiny worlds (the C# throws); the
//!   port skips an inverted band before any draw (the `mini_biomes.rs`
//!   sub-vanilla idiom). Real worlds (4200×1200) never hit them.
//!
//! A classic world reads no flag and never enters either function — both
//! world-hash pins are UNCHANGED (the g41 egg-sac re-pin deferral stands).

use std::collections::{HashSet, VecDeque};

use super::noise::Rng;
use super::pipeline::{GenVars, Streams};
use super::spider_gem_moss::spider_wall_remover;
use super::GenOptions;
use crate::edit::kill_tile;
use crate::growth::{grow_tree, spread_grass, try_growing_tree_by_type};
use crate::seeds::{active_secret_seed_count, MicroSeedFlags, SeedFlags};
use crate::tile::{FLAG_ACTIVE, FLAG_LIQUID_HONEY, FLAG_LIQUID_LAVA};
use crate::tiledata as td;
use crate::world::World;
use crate::UnifiedRandom;

// --- the ids the conversion paints (TileID.cs / WallID.cs) ------------------

const TILE_DIRT: u16 = 0;
const TILE_STONE: u16 = 1;
const TILE_GRASS: u16 = 2;
const TILE_PLANTS: u16 = 3;
const TILE_IRON: u16 = 6;
const TILE_COPPER: u16 = 7;
const TILE_CORRUPT_GRASS: u16 = 23;
const TILE_CORRUPT_PLANTS: u16 = 24;
const TILE_EBONSTONE: u16 = 25;
const TILE_MUD: u16 = 59;
const TILE_JUNGLE_GRASS: u16 = 60;
const TILE_JUNGLE_PLANTS: u16 = 61;
const TILE_JUNGLE_VINES: u16 = 62;
const TILE_JUNGLE_PLANTS2: u16 = 74;
const TILE_HELLSTONE_BRICK: u16 = 76;
const TILE_IMMATURE_HERBS: u16 = 82;
const TILE_SILT: u16 = 123;
const TILE_CLAY: u16 = 40;
const TILE_SPIKES: u16 = 48;
const TILE_COBWEB: u16 = 51;
const TILE_VINES: u16 = 52;
const TILE_SAND: u16 = 53;
const TILE_TIN: u16 = 166;
const TILE_LEAD: u16 = 167;
const TILE_ICE: u16 = 161;
const TILE_SNOW: u16 = 147;
const TILE_CRIMSON_GRASS: u16 = 199;
const TILE_CRIMSON_PLANTS: u16 = 201;
const TILE_CRIMSTONE: u16 = 203;
const TILE_SLUSH: u16 = 224;
const TILE_HIVE: u16 = 225;
const TILE_LIHZAHRD_BRICK: u16 = 226;
const TILE_HONEY_BLOCK: u16 = 229;
const TILE_CRISPY_HONEY: u16 = 230;
const TILE_LARVA: u16 = 231;
const TILE_WOODEN_SPIKES: u16 = 232;
const TILE_PLANT_DETRITUS: u16 = 233;
const TILE_WATER_DRIP: u16 = 373;
const TILE_HONEY_DRIP: u16 = 375;
const TILE_BUBBLE: u16 = 379;
const TILE_VINE_FLOWERS: u16 = 382;
const TILE_LIVING_MAHOGANY: u16 = 383;
const TILE_MAHOGANY_LEAF: u16 = 384;
const TILE_BEE_HIVE: u16 = 444;
const TILE_BAMBOO: u16 = 571;
const TILE_CORRUPT_JUNGLE: u16 = 661;
const TILE_CRIMSON_JUNGLE: u16 = 662;

const WALL_MUD: u16 = 15;
const WALL_GRASS: u16 = 63;
const WALL_JUNGLE: u16 = 64;
const WALL_FLOWER: u16 = 65;
const WALL_HIVE: u16 = 86;
const WALL_LIHZAHRD: u16 = 87;
const WALL_SPIDER: u16 = 62;

/// `WorldGen.beachDistance` (`WorldGen.cs:86914`).
const BEACH_DISTANCE: i32 = 380;
/// `WorldGen.maxWallOut2` (`WorldGen.cs:86922`).
const MAX_WALL_OUT2: i32 = 5000;
/// `Main.tileDungeon` (`Main.cs:7992-7997`) — the six dungeon bricks.
const TILE_DUNGEON: [u16; 6] = [41, 43, 44, 677, 678, 679];
/// `Main.wallDungeon` (`Main.cs:10598-10606`) — the nine dungeon walls.
const WALL_DUNGEON: [u16; 9] = [7, 8, 9, 94, 95, 96, 97, 98, 99];
/// `TileID.Sets.Conversion.Grass` (`TileID.cs:86`) — hand-transcribed (the
/// generated tables carry no Conversion.Grass set; the `trees.rs`
/// `ALLOWS_PLANTS_TO_GROW` precedent).
const CONVERSION_GRASS: [u16; 6] = [2, 23, 199, 109, 477, 492];
/// `WallID.Sets.Glass` (`WallID.cs:91`) — hand-transcribed.
const WALL_GLASS: [u16; 10] = [21, 88, 89, 90, 91, 92, 241, 93, 243, 242];
/// `WallID.Sets.CannotBeReplacedByWallSpread` (`WallID.cs:98`).
const WALL_NO_SPREAD: [u16; 7] = [4, 40, 3, 83, 87, 244, 34];

/// The bamboo 571 `TileFrame` arm (`:25459-25462`) rides the post-pipeline
/// framer, so the id is kept as a real dependency of the const block (the
/// `spider_gem_moss.rs` idiom).
const _: u16 = TILE_BAMBOO;

// --- small helpers ----------------------------------------------------------

fn in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && y >= fluff && x < world.width - fluff && y < world.height - fluff
}

fn kind_at(world: &World, x: i32, y: i32) -> u16 {
    world.tile(x, y).map_or(0, |t| t.kind)
}

fn wall_at(world: &World, x: i32, y: i32) -> u16 {
    world.tile(x, y).map_or(0, |t| t.wall)
}

fn active_at(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| t.active())
}

/// `WorldGen.SolidTile` (`:70111-70132`) — `bubble_solid` models the
/// `tileSolid[379]` window (false inside `FinishNotTheBees`).
fn solid_tile(world: &World, x: i32, y: i32, bubble_solid: bool) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && td::TILE_SOLID.has(t.kind)
        && !td::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
        && (bubble_solid || t.kind != TILE_BUBBLE)
}

/// `WorldGen.SolidTile2` (`:70134-70151`) — no solidTop veto.
fn solid_tile2(world: &World, x: i32, y: i32, bubble_solid: bool) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && td::TILE_SOLID.has(t.kind)
        && t.slope() == 0
        && !t.half_brick()
        && !t.inactive()
        && (bubble_solid || t.kind != TILE_BUBBLE)
}

/// `WorldGen.SolidOrSlopedTile` (`:69985-70010`, includePlatforms=false) —
/// active, tileSolid, !tileSolidTop, !inActive (no slope/half-brick veto).
/// Used by `NotTheBees` (boot state, 379 solid).
fn solid_or_sloped(world: &World, x: i32, y: i32) -> bool {
    world
        .tile(x, y)
        .is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind) && !td::TILE_SOLID_TOP.has(t.kind) && !t.inactive())
}

/// The single-tile placement idiom (the module note): kind + active, frames
/// left to the framer. Refuses an occupied cell (the C# `EmptyTile` gate).
fn place_single(world: &mut World, x: i32, y: i32, kind: u16) -> bool {
    if !in_world(world, x, y, 0) || active_at(world, x, y) {
        return false;
    }
    if let Some(t) = world.tile_mut(x, y) {
        t.kind = kind;
        t.flags |= FLAG_ACTIVE;
        t.frame_x = -1;
        t.frame_y = -1;
    }
    true
}

fn set_kind(world: &mut World, x: i32, y: i32, kind: u16) {
    if let Some(t) = world.tile_mut(x, y) {
        t.kind = kind;
    }
}

fn deactivate(world: &mut World, x: i32, y: i32) {
    if let Some(t) = world.tile_mut(x, y) {
        t.flags &= !FLAG_ACTIVE;
    }
}

/// `Tile.honey(false)` / `lava(false)` (`Tile.cs:350-377`) — clear the kind
/// bits, keep the amount.
fn clear_honey(world: &mut World, x: i32, y: i32) {
    if let Some(t) = world.tile_mut(x, y) {
        t.flags &= !FLAG_LIQUID_HONEY;
    }
}

fn clear_lava(world: &mut World, x: i32, y: i32) {
    if let Some(t) = world.tile_mut(x, y) {
        t.flags &= !FLAG_LIQUID_LAVA;
    }
}

/// `Tile.honey(true)` (`Tile.cs:367-377`) — set honey, clear lava.
fn set_honey(world: &mut World, x: i32, y: i32) {
    if let Some(t) = world.tile_mut(x, y) {
        t.flags = (t.flags & !FLAG_LIQUID_LAVA) | FLAG_LIQUID_HONEY;
    }
}

/// `SecretSeed.Variations.noSpiderCavesActuallyNoSpiderCaves`
/// (`WorldGen.cs:270-278`) — the short-list veto (the `spider_gem_moss.rs`
/// helper is module-private; re-derived here).
fn no_spider_caves_actually(micro: MicroSeedFlags, flags: SeedFlags) -> bool {
    micro.contains(MicroSeedFlags::NO_SPIDER_CAVES) && active_secret_seed_count(micro, flags) < 4
}

/// The `NotTheBees` solid-tile conversion cascade (`WorldGen.cs:25195-25267`):
/// the living-wood/leaf arms, then the big exclusion list, then the
/// slush/sand → honey-block arms, the beach-band gate, the dungeon paint, and
/// the grass/dirt/deep-hive conversions. `num2` is the deep-hive band line.
/// Returns nothing; the draws ride `rng` in the C# statement order.
#[allow(clippy::too_many_arguments)]
fn convert_solid_cell(
    world: &mut World,
    rng: &mut Rng,
    i: i32,
    j: i32,
    num2: i32,
    lava_line: i32,
    flags: SeedFlags,
) {
    let remix = flags.contains(SeedFlags::REMIX);
    let dst = flags.contains(SeedFlags::DONT_STARVE);
    let kind = kind_at(world, i, j);
    // `:25197-25210` — the living wood / leaf arms (the !remix gate).
    if kind == 191 || kind == TILE_LIVING_MAHOGANY {
        if !remix {
            set_kind(world, i, j, TILE_LIVING_MAHOGANY);
        }
        return;
    }
    if kind == 192 || kind == TILE_MAHOGANY_LEAF {
        if !remix {
            set_kind(world, i, j, TILE_MAHOGANY_LEAF);
        }
        return;
    }
    // `:25211` — the exclusion list (the clouds set + the named bricks).
    if matches!(kind, 151 | 662 | 661 | 120 | 158 | 175 | 45 | 119 | 57 | 76 | 75 | 229 | 230 | 407 | 404)
        || td::SET_CLOUDS.has(kind)
        || (63..=68).contains(&kind)
    {
        return;
    }
    // `:25213-25226` — slush 224 → honey block, sand 53 → honey block on the
    // beach band (both gated on !dst || remix).
    if kind == TILE_SLUSH {
        if !dst || remix {
            set_kind(world, i, j, TILE_HONEY_BLOCK);
        }
        return;
    }
    if kind == TILE_SAND {
        if (!dst || remix)
            && (i < BEACH_DISTANCE + rng.range(0, 3) || i > world.width - BEACH_DISTANCE - rng.range(0, 3))
        {
            set_kind(world, i, j, TILE_HONEY_BLOCK);
        }
        return;
    }
    // `:25227` — the beach-band / sandstone / door / crimstone / corrupt /
    // trap / boulder / explosives gate. The two beach draws run only when the
    // first two disjuncts are false (the C# `||` short-circuit).
    let beach_ok = i <= BEACH_DISTANCE - rng.range(0, 3) || i >= world.width - BEACH_DISTANCE + rng.range(0, 3);
    if !(beach_ok || (kind != 397 && kind != 396))
        || matches!(kind, 10 | 203 | 25 | 137 | 141)
        || td::SET_BOULDERS.has(kind)
    {
        return;
    }
    // `:25229-25232` — the dungeon bricks take paint 14, not a conversion.
    if TILE_DUNGEON.contains(&kind) || td::SET_CRACKED_BRICKS.has(kind) {
        if let Some(t) = world.tile_mut(i, j) {
            t.set_paint(14);
        }
        return;
    }
    // `:25233` — the second exclusion list.
    if matches!(kind, TILE_LIHZAHRD_BRICK | 202 | 70 | 48 | 232) {
        return;
    }
    // `:25235-25245` — the grass conversion (the lava-line band).
    if CONVERSION_GRASS.contains(&kind) || kind == TILE_JUNGLE_GRASS || kind == 70 {
        if j > lava_line + rng.range(-2, 3) + 2 {
            set_kind(world, i, j, 70);
        } else {
            set_kind(world, i, j, TILE_JUNGLE_GRASS);
        }
        return;
    }
    // `:25246-25249` — dirt / mud → mud.
    if kind == TILE_DIRT || kind == TILE_MUD {
        set_kind(world, i, j, TILE_MUD);
        return;
    }
    // `:25250-25263` — the deep-hive / shallow-hive band (the bamboo-block
    // exclusions + the !dst || remix gate).
    if matches!(kind, 633 | 562 | 563) || (dst && !remix) {
        return;
    }
    if j > lava_line + rng.range(-2, 3) + 2 {
        if j <= num2 + rng.range(0, 3) {
            set_kind(world, i, j, TILE_CRISPY_HONEY);
        }
    } else if !remix || j as f64 > world.world_surface + rng.range(-1, 2) as f64 {
        set_kind(world, i, j, TILE_HIVE);
    }
}

/// The exposed-hive grass skirt (`WorldGen.cs:25268-25296`): an exposed
/// honey/crispy-honey cell on the inland band grows a grass cap and pocks up
/// to 8 mud cells. `from`/`cap` are (225, 60) or (230, 70). The 8-neighbour
/// active/wall tests carry the C#'s own quirk — the last term re-reads
/// `(x+1, y-1)` instead of `(x+1, y+1)` (`:25270`/`:25277`), ported verbatim.
fn hive_skirt(world: &mut World, rng: &mut Rng, i: i32, j: i32, from: u16, cap: u16) {
    let any_open = [(i - 1, j - 1), (i, j - 1), (i + 1, j - 1), (i - 1, j), (i + 1, j), (i - 1, j + 1), (i, j + 1), (i + 1, j - 1)]
        .iter()
        .any(|&(x, y)| !active_at(world, x, y));
    let walls_clear = (j as f64) < world.world_surface
        || [(i - 1, j - 1), (i, j - 1), (i + 1, j - 1), (i - 1, j), (i + 1, j), (i - 1, j + 1), (i, j + 1), (i + 1, j - 1)]
            .iter()
            .all(|&(x, y)| wall_at(world, x, y) == 0);
    if !(active_at(world, i, j) && kind_at(world, i, j) == from && any_open && walls_clear) {
        return;
    }
    set_kind(world, i, j, cap);
    for _ in 0..8 {
        let x = i + rng.range(-2, 3);
        let y = j + rng.range(-2, 3);
        let enclosed = active_at(world, x, y)
            && kind_at(world, x, y) == from
            && [(x - 1, y - 1), (x, y - 1), (x + 1, y - 1), (x - 1, y), (x + 1, y), (x - 1, y + 1), (x, y + 1), (x + 1, y - 1)]
                .iter()
                .all(|&(nx, ny)| active_at(world, nx, ny));
        if enclosed {
            set_kind(world, x, y, TILE_MUD);
        }
    }
}

/// The `NotTheBees` wall-conversion arm (`WorldGen.cs:25301-25338`). The
/// centre-strip gate's two `Next(3)` draws short-circuit exactly as the C#
/// `||` chain; the deep-hive `Next(-1,2)` draw runs only on remix.
fn convert_wall(world: &mut World, rng: &mut Rng, i: i32, j: i32, num2: i32, flags: SeedFlags) {
    let remix = flags.contains(SeedFlags::REMIX);
    let dst = flags.contains(SeedFlags::DONT_STARVE);
    let tenth = flags.contains(SeedFlags::TENTH_ANNIVERSARY);
    let drunk = flags.contains(SeedFlags::DRUNK);
    let wall = wall_at(world, i, j);
    if matches!(wall, 15 | 64 | 204 | 205 | 206 | 207 | WALL_LIHZAHRD) {
        return;
    }
    if WALL_DUNGEON.contains(&wall) {
        world.set_wall_paint(i, j, 14);
        return;
    }
    // `:25307` — the centre-strip / seed gate (the two Next(3) draws are the
    // 3rd/4th disjuncts, reached only when remix and the classic-seed arm are
    // both false).
    let centre_ok = remix
        || (!tenth && !dst && !drunk)
        || !((i as f64) > world.width as f64 * 0.4 - rng.range(0, 3) as f64)
        || !((i as f64) < world.width as f64 * 0.6 + rng.range(0, 3) as f64);
    if !centre_ok
        || (dst && !remix)
        || matches!(wall, 23 | 24 | 42 | 10 | 82 | 187 | 216 | 34 | 244)
        || WALL_GLASS.contains(&wall)
    {
        return;
    }
    // `:25309-25336` — the identity walls, the grass → jungle arm, then the
    // deep-hive wall band.
    if matches!(wall, 2 | 196 | 197 | 198 | 199) {
        return; // the C# re-assigns the same id (no-op)
    }
    if wall == WALL_GRASS {
        if let Some(t) = world.tile_mut(i, j) {
            t.wall = WALL_JUNGLE;
        }
        return;
    }
    if matches!(wall, 3 | 83 | 73 | 62 | 180 | 178 | 13 | 14 | 312 | 313) || wall == 0 {
        return;
    }
    let depth_ok = !remix || (j as f64) > world.world_surface + rng.range(-1, 2) as f64;
    if depth_ok && j <= num2 + rng.range(0, 3) {
        if let Some(t) = world.tile_mut(i, j) {
            t.wall = WALL_HIVE;
        }
    }
}

/// The `NotTheBees` ocean/dungeon-side arm (`WorldGen.cs:25132-25149`): the
/// honey-block → water conversion + the dungeon paint 14, then `continue`.
fn ocean_side_arm(world: &mut World, i: i32, j: i32) {
    if active_at(world, i, j) && kind_at(world, i, j) == TILE_HONEY_BLOCK {
        deactivate(world, i, j);
        if let Some(t) = world.tile_mut(i, j) {
            t.flags &= !FLAG_LIQUID_HONEY; // liquidType(0)
            t.liquid = 255;
        }
    }
    let kind = kind_at(world, i, j);
    if active_at(world, i, j) && (TILE_DUNGEON.contains(&kind) || td::SET_CRACKED_BRICKS.has(kind)) {
        if let Some(t) = world.tile_mut(i, j) {
            t.set_paint(14);
        }
    }
    if WALL_DUNGEON.contains(&wall_at(world, i, j)) {
        world.set_wall_paint(i, j, 14);
    }
}

/// The `NotTheBees` centre-strip arm (`WorldGen.cs:25150-25181`): the
/// drunk/anniversary/DST worlds keep a dirt/mud centre column — the exposed
/// dirt → dirt-2 fill, the mud-wall → flower/grass wall roll, the jungle grass
/// → mud conversion, then `continue`.
fn centre_strip_arm(world: &mut World, rng: &mut Rng, i: i32, j: i32, flags: SeedFlags) {
    let dst = flags.contains(SeedFlags::DONT_STARVE);
    let tenth = flags.contains(SeedFlags::TENTH_ANNIVERSARY);
    if (j as f64) < world.rock_layer - 1.0 {
        // `:25154` — the exposed-dirt fill (the 8-neighbour active test, the
        // C#'s own (x+1, y-1) duplicate quirk).
        if kind_at(world, i, j) == TILE_DIRT {
            let any_open = [(i - 1, j - 1), (i, j - 1), (i + 1, j - 1), (i - 1, j), (i + 1, j), (i - 1, j + 1), (i, j + 1), (i + 1, j - 1)]
                .iter()
                .any(|&(x, y)| !active_at(world, x, y));
            if any_open {
                set_kind(world, i, j, TILE_GRASS);
            }
        }
        // `:25158-25174` — the mud wall → flower/grass wall.
        if wall_at(world, i, j) == WALL_MUD {
            let mut wall = WALL_FLOWER;
            if rng.range(0, 2) == 0 {
                wall = WALL_GRASS;
            }
            if dst {
                wall = WALL_GRASS;
            } else if tenth {
                wall = WALL_FLOWER;
            }
            if let Some(t) = world.tile_mut(i, j) {
                t.wall = wall;
            }
        }
    }
    if kind_at(world, i, j) == TILE_JUNGLE_GRASS {
        set_kind(world, i, j, TILE_MUD);
    }
}

/// `WorldGen.NotTheBees` (`WorldGen.cs:25107-25360`): the hive-cluster world
/// conversion. Draws exclusively on `pass_rng("NotTheBees")`.
pub(crate) fn not_the_bees(world: &mut World, gv: &GenVars, streams: &mut Streams, opts: &GenOptions) {
    let flags = opts.flags;
    if flags.contains(SeedFlags::SKYBLOCK) || !flags.contains(SeedFlags::NOT_THE_BEES) {
        return;
    }
    let mut rng = streams.pass_rng("NotTheBees");
    let (width, height) = (world.width, world.height);
    let lava_line = gv.lava_line;
    let remix = flags.contains(SeedFlags::REMIX);
    let tenth = flags.contains(SeedFlags::TENTH_ANNIVERSARY);
    let dst = flags.contains(SeedFlags::DONT_STARVE);
    let drunk = flags.contains(SeedFlags::DRUNK);
    let dual = opts.micro_flags.contains(MicroSeedFlags::DUAL_DUNGEONS);
    // `:25113-25122` — the side band + the deep-hive line.
    let num = width / 7;
    let num2 = if remix { height - 180 } else { (lava_line + height - 180) / 2 };
    // The port's dungeon side is fixed Right (M5-BIOME): `<= Left` false,
    // `>= Right` true.
    for i in 5..width - 5 {
        for j in 5..height - 180 {
            not_the_bees_cell(world, &mut rng, i, j, num, num2, lava_line, flags, remix, tenth, dst, drunk, dual);
        }
    }
}

/// One cell of the `NotTheBees` sweep — the C#'s per-tile body, in order.
#[allow(clippy::too_many_arguments)]
fn not_the_bees_cell(
    world: &mut World,
    rng: &mut Rng,
    i: i32,
    j: i32,
    num: i32,
    num2: i32,
    lava_line: i32,
    flags: SeedFlags,
    remix: bool,
    tenth: bool,
    dst: bool,
    drunk: bool,
    dual: bool,
) {
    let width = world.width;
    let height = world.height;
    // `:25127` — the ocean/beach flag (the two Next(3) draws short-circuit:
    // ocean_depths true skips both).
    let flag = crate::scene::ocean_depths(world.world_surface, world.rock_layer, i, width, j)
        || ((j as f64) < (world.world_surface + world.rock_layer * 2.0) / 3.0 + rng.range(0, 3) as f64
            && (i < BEACH_DISTANCE - 50 - rng.range(0, 3) || i > width - BEACH_DISTANCE + 50 + rng.range(0, 3)));
    // `:25128-25131` — the remix side/depth skip (the three Next(3) draws).
    if remix
        && (i < num + rng.range(0, 3)
            || i >= width - num - rng.range(0, 3)
            || ((j as f64) > (world.world_surface * 2.0 + world.rock_layer) / 3.0 + rng.range(0, 3) as f64
                && j < height - 350 - rng.range(0, 3)))
    {
        return;
    }
    // `:25132-25149` — the ocean/dungeon-side arm (the C# `&` is
    // non-short-circuit, but `flag` is already computed).
    if ((tenth || dst || i > width / 2) && !remix) && flag {
        ocean_side_arm(world, i, j);
        return;
    }
    // `:25150-25181` — the centre-strip arm (the two Next(3) draws).
    if !remix
        && (tenth || dst || drunk)
        && (i as f64) > width as f64 * 0.4 - rng.range(0, 3) as f64
        && (i as f64) < width as f64 * 0.6 + rng.range(0, 3) as f64
    {
        centre_strip_arm(world, rng, i, j, flags);
        return;
    }
    // `:25182-25190` — SkipOnDualDungeons reads false (no bounds model); the
    // vines → hive conversion.
    let kind = kind_at(world, i, j);
    if kind == TILE_VINES || kind == TILE_VINE_FLOWERS {
        set_kind(world, i, j, TILE_JUNGLE_VINES);
    }
    // `:25191-25194` — the dual-dungeon paint (needs no bounds).
    let kind = kind_at(world, i, j);
    if dual && active_at(world, i, j) && (TILE_DUNGEON.contains(&kind) || td::SET_CRACKED_BRICKS.has(kind)) {
        if let Some(t) = world.tile_mut(i, j) {
            t.set_paint(14);
        }
    }
    // `:25195-25267` — the solid-tile conversion cascade (the DST exclusion
    // list + the ore/marble/granite/silt/clay/bubble vetoes).
    let kind = kind_at(world, i, j);
    let dst_ok = !dst
        || remix
        || !matches!(kind, 1 | 147 | 161 | 30 | 321 | 158 | 190 | 162);
    if (solid_or_sloped(world, i, j) || (active_at(world, i, j) && td::SET_CRACKED_BRICKS.has(kind)))
        && dst_ok
        && !td::SET_ORE.has(kind)
        && !matches!(kind, 368 | 367 | 123 | 40 | 379)
    {
        convert_solid_cell(world, rng, i, j, num2, lava_line, flags);
    }
    // `:25268-25296` — the exposed-hive grass skirts (inland band).
    if !remix && j < height - 50 && j > 50 && i > BEACH_DISTANCE + 220 && i < width - BEACH_DISTANCE - 220 {
        hive_skirt(world, rng, i, j, TILE_HIVE, TILE_JUNGLE_GRASS);
        hive_skirt(world, rng, i, j, TILE_CRISPY_HONEY, 70);
    }
    // `:25297-25300` — the dual-dungeon wall paint.
    if dual && WALL_DUNGEON.contains(&wall_at(world, i, j)) {
        world.set_wall_paint(i, j, 14);
    }
    // `:25301-25338` — the wall conversion.
    convert_wall(world, rng, i, j, num2, flags);
    // `:25339-25357` — the liquid honey arm.
    let liquid = world.tile(i, j).map_or(0, |t| t.liquid);
    if (!tenth || remix || !((j as f64) < world.world_surface))
        && (!dst || remix)
        && liquid > 0
        && j <= lava_line + 2
    {
        if (i as f64) > width as f64 * 0.4 && (i as f64) < width as f64 * 0.6 {
            clear_honey(world, i, j);
        } else if (j as f64) > world.rock_layer && (i < BEACH_DISTANCE + 200 || i > width - BEACH_DISTANCE - 200) {
            clear_honey(world, i, j);
        } else if WALL_DUNGEON.contains(&wall_at(world, i, j)) {
            clear_honey(world, i, j);
        } else {
            set_honey(world, i, j);
        }
    }
}

/// The evil-radius conversion (`WorldGen.cs:25468-25519`): around an
/// ebonstone/crimstone (or remix surface corrupt/crimson grass) seed, the
/// jungle grass converts to corrupt/crimson jungle, the bee hive below dies,
/// and the plant above swaps to the evil plant (or dies if detritus/herbs).
/// `radius` is 20 (10 on remix); `crimson` picks the 662/201 pair over 661/24.
fn evil_radius(world: &mut World, m: i32, n: i32, radius: i32, crimson: bool) {
    let (jungle, plant) = if crimson {
        (TILE_CRIMSON_JUNGLE, TILE_CRIMSON_PLANTS)
    } else {
        (TILE_CORRUPT_JUNGLE, TILE_CORRUPT_PLANTS)
    };
    for x in m - radius..=m + radius {
        for y in n - radius..=n + radius {
            if active_at(world, x, y) && kind_at(world, x, y) == TILE_JUNGLE_GRASS {
                if kind_at(world, x, y + 1) == TILE_BEE_HIVE {
                    kill_tile(world, x, y + 1, true);
                }
                set_kind(world, x, y, jungle);
                let above = kind_at(world, x, y - 1);
                if active_at(world, x, y - 1) && (above == TILE_JUNGLE_PLANTS || above == TILE_JUNGLE_PLANTS2) {
                    deactivate(world, x, y - 1);
                    place_single(world, x, y - 1, plant);
                } else if !crimson && (above == TILE_PLANT_DETRITUS || above == TILE_IMMATURE_HERBS) {
                    // `:25487-25490` — the corrupt arm's else-if (the crimson
                    // arm nests it inside the active branch instead).
                    kill_tile(world, x, y - 1, true);
                }
            } else if crimson {
                let above = kind_at(world, x, y - 1);
                if above == TILE_PLANT_DETRITUS || above == TILE_IMMATURE_HERBS {
                    kill_tile(world, x, y - 1, true);
                }
            }
        }
    }
}

/// One cell of the `FinishNotTheBees` main sweep (`WorldGen.cs:25436-25630`).
/// `cloud_row` is the last cloud row (`num`); `bubble_solid` is false (the
/// `tileSolid[379]` window). The larva roll at the tail is the C#'s `continue`
/// gate — the per-cell draws ride `rng` in statement order.
#[allow(clippy::too_many_arguments)]
fn finish_cell(
    world: &mut World,
    rng: &mut Rng,
    m: i32,
    n: i32,
    cloud_row: i32,
    lava_line: i32,
    flags: SeedFlags,
) {
    let remix = flags.contains(SeedFlags::REMIX);
    let dst = flags.contains(SeedFlags::DONT_STARVE);
    let tenth = flags.contains(SeedFlags::TENTH_ANNIVERSARY);
    let ftw = flags.contains(SeedFlags::FOR_THE_WORTHY);
    let no_traps = flags.contains(SeedFlags::NO_TRAPS);
    let width = world.width;
    // `:25444` — num8 = active ? type : -1 (the port's u16::MAX sentinel for
    // the C# -1; none of the compared ids is 0 or MAX, so the arms match).
    let num8 = if active_at(world, m, n) { kind_at(world, m, n) } else { u16::MAX };
    // `:25445-25448` — the FTW altar → crispy honey.
    if ftw && !tenth && num8 == TILE_HELLSTONE_BRICK {
        set_kind(world, m, n, TILE_CRISPY_HONEY);
    }
    // `:25449-25454` — the above-cloud honey block → water.
    if n < cloud_row && num8 == TILE_HONEY_BLOCK {
        deactivate(world, m, n);
        if let Some(t) = world.tile_mut(m, n) {
            t.liquid = 255;
            t.flags &= !FLAG_LIQUID_HONEY; // liquidType(0)
        }
    }
    // `:25455-25458` — the DST/noTraps spikes → wooden spikes.
    if (dst || no_traps) && !tenth && num8 == TILE_SPIKES {
        set_kind(world, m, n, TILE_WOODEN_SPIKES);
    }
    // `:25459-25462` — the bamboo TileFrame rides the post-pipeline framer.
    // `:25463-25493` — the corrupt radius.
    let num9 = if remix { 10 } else { 20 };
    if num8 == TILE_EBONSTONE || (remix && num8 == TILE_CORRUPT_GRASS && (n as f64) < world.world_surface) {
        evil_radius(world, m, n, num9, false);
    } else if num8 == TILE_CRIMSTONE || (remix && num8 == TILE_CRIMSON_GRASS && (n as f64) < world.world_surface) {
        evil_radius(world, m, n, num9, true);
    }
    // `:25520-25523` — the vines on a hive/jungle/mahogany-leaf seat → hive.
    if (num8 == TILE_VINE_FLOWERS || num8 == TILE_VINES)
        && active_at(world, m, n - 1)
        && matches!(kind_at(world, m, n - 1), 62 | 60 | 384)
    {
        set_kind(world, m, n, TILE_JUNGLE_VINES);
    }
    // `:25524-25534` — the grass spread by the lava line (the Next(-2,3)
    // draw runs for EVERY cell — the C# evaluates the `if` condition first).
    if n > lava_line + rng.range(-2, 3) + 2 {
        if !remix {
            spread_grass(world, m, n, TILE_MUD, 70);
        }
    } else {
        spread_grass(world, m, n, TILE_MUD, TILE_JUNGLE_GRASS);
    }
    // `:25535-25537` — the three band lines (the Next(5)/Next(-2,3) draws
    // run for EVERY cell — the C# computes them before the gate).
    let num14 = lava_line + rng.range(0, 5);
    let num15 = world.world_surface as i32 + rng.range(-2, 3) + 35;
    let num16 = (num14 + num15) / 2;
    // `:25538-25585` — the beach-band ice/snow conversion (the two
    // Next(-2,3) beach draws short-circuit on the left band).
    if n > num15
        && n <= num14
        && (m < BEACH_DISTANCE + 200 - 20 - rng.range(-2, 3)
            || m > width - BEACH_DISTANCE - 200 + 20 + rng.range(-2, 3))
    {
        if wall_at(world, m, n) == WALL_HIVE {
            if let Some(t) = world.tile_mut(m, n) {
                t.wall = 0;
            }
        }
        if world.tile(m, n).is_some_and(|t| t.liquid > 0) {
            clear_honey(world, m, n);
            clear_lava(world, m, n);
        }
        if num8 == TILE_MUD {
            let any_jungle = [(m - 1, n - 1), (m, n - 1), (m + 1, n - 1), (m - 1, n), (m, n), (m + 1, n), (m - 1, n + 1), (m, n + 1), (m + 1, n + 1)]
                .iter()
                .any(|&(x, y)| kind_at(world, x, y) == TILE_JUNGLE_GRASS);
            if !any_jungle {
                set_kind(world, m, n, if n < num16 { TILE_ICE } else { TILE_SNOW });
            }
        }
        if num8 == TILE_HIVE || num8 == TILE_CRISPY_HONEY {
            set_kind(world, m, n, if n > num16 { TILE_ICE } else { TILE_SNOW });
        }
    }
    // `:25586-25607` — the deep ore demotion.
    if !remix && !dst {
        finish_ore_demotion(world, rng, m, n, num8, lava_line);
    }
    // `:25608-25629` — the larva roll (the continue gate).
    let honey = world.tile(m, n).is_some_and(|t| t.liquid_kind() == crate::tile::LiquidKind::Honey);
    let liquid = world.tile(m, n).map_or(0, |t| t.liquid);
    if n <= cloud_row
        || m <= 350
        || m >= width - 350
        || (liquid != 0 && !honey)
        || (tenth && !remix)
    {
        return;
    }
    let roll = if dst || remix {
        rng.range(0, 600)
    } else if no_traps {
        rng.range(0, 150)
    } else {
        rng.range(0, 225)
    };
    if roll == 0 {
        place_larva(world, m, n);
    }
}

/// The deep ore demotion (`WorldGen.cs:25586-25607`): iron/copper/tin/lead
/// → dirt below `(lavaLine + rockLayer*2)/3`, silt/clay → stone below
/// `(lavaLine + rockLayer)/2`. The `Next(-2,3)` draw runs only inside the
/// matching case arm (the C# switch).
fn finish_ore_demotion(world: &mut World, rng: &mut Rng, m: i32, n: i32, num8: u16, lava_line: i32) {
    match num8 {
        TILE_IRON | TILE_COPPER | TILE_TIN | TILE_LEAD => {
            if (n as f64) > (lava_line as f64 + world.rock_layer * 2.0) / 3.0 + rng.range(-2, 3) as f64 + 2.0 {
                set_kind(world, m, n, TILE_DIRT);
            }
        }
        TILE_CLAY | TILE_SILT => {
            if (n as f64) > (lava_line as f64 + world.rock_layer) / 2.0 + rng.range(-2, 3) as f64 + 2.0 {
                set_kind(world, m, n, TILE_STONE);
            }
        }
        _ => {}
    }
}

/// `WorldGen.Place3x3` for the larva 231 (`WorldGen.cs:53534-53634`, the
/// `num = -2` arm): the 3×3 block sits with its bottom row at `y`, the two
/// rows above it clear, and a solid floor at `y+1`. Draw-free. The frames
/// ride the framer (the single-tile idiom).
fn place_larva(world: &mut World, x: i32, y: i32) {
    let mut ok = true;
    for i in x - 1..x + 2 {
        for j in y - 2..y + 1 {
            if active_at(world, i, j) {
                ok = false;
            }
        }
    }
    for k in x - 1..x + 2 {
        if !solid_tile2(world, k, y + 1, false) {
            ok = false;
            break;
        }
    }
    if !ok {
        return;
    }
    for dx in -1..=1i32 {
        for dy in -2..=0i32 {
            if let Some(t) = world.tile_mut(x + dx, y + dy) {
                t.kind = TILE_LARVA;
                t.flags |= FLAG_ACTIVE;
                t.frame_x = ((dx + 1) * 18) as i16;
                t.frame_y = ((dy + 2) * 18) as i16;
            }
        }
    }
}

/// `Spread.Wall2` (`WorldGen.cs:3416-3524`): the BFS wall flood. The C#'s
/// two-list wave order (list[0] removal) is a FIFO queue here — the write
/// order is identical (a cell's wall is set before its neighbours enqueue).
/// `maxWallOut2 = 5000` caps the non-solid conversions; wall 62 stops at air
/// and spreads 8-way + the ±2 x arms (`WallSpreadStopsAtAir`, `WallID.cs:99`).
pub(crate) fn spread_wall2(world: &mut World, x: i32, y: i32, wall_type: u16) {
    if !in_world(world, x, y, 0) {
        return;
    }
    let stops_at_air = wall_type == WALL_GRASS || wall_type == WALL_SPIDER;
    let mut count = 0i32;
    let mut seen: HashSet<(i32, i32)> = HashSet::new();
    let mut queue: VecDeque<(i32, i32)> = VecDeque::new();
    queue.push_back((x, y));
    while let Some((cx, cy)) = queue.pop_front() {
        if !in_world(world, cx, cy, 1) {
            continue;
        }
        seen.insert((cx, cy));
        let cur = wall_at(world, cx, cy);
        if cur == wall_type || WALL_NO_SPREAD.contains(&cur) {
            continue;
        }
        if !solid_tile(world, cx, cy, true) {
            if stops_at_air && cur == 0 {
                continue;
            }
            count += 1;
            if count >= MAX_WALL_OUT2 {
                continue;
            }
            if let Some(t) = world.tile_mut(cx, cy) {
                t.wall = wall_type;
            }
            let mut neighbours = vec![(cx - 1, cy), (cx + 1, cy), (cx, cy - 1), (cx, cy + 1)];
            if stops_at_air {
                neighbours.extend([
                    (cx - 1, cy - 1),
                    (cx + 1, cy - 1),
                    (cx - 1, cy + 1),
                    (cx + 1, cy + 1),
                    (cx - 2, cy),
                    (cx + 2, cy),
                ]);
            }
            for (nx, ny) in neighbours {
                if !seen.contains(&(nx, ny)) {
                    queue.push_back((nx, ny));
                }
            }
        } else if active_at(world, cx, cy) {
            if let Some(t) = world.tile_mut(cx, cy) {
                t.wall = wall_type;
            }
        }
    }
}

/// The DST spider-web arm's surface seed scan (`WorldGen.cs:25632-25715`):
/// the jungle-grass/mud cells on a mud/grass wall with an open neighbour seed
/// a `Spread.Wall2(…, 62)` flood. The per-cell `Next(3)` veto (flipped on FTW)
/// and the row-band `Next(3)` draw ride `rng`.
fn dst_surface_seeds(world: &mut World, rng: &mut Rng, flags: SeedFlags) {
    let ftw = flags.contains(SeedFlags::FOR_THE_WORTHY);
    let width = world.width;
    for x in 50..width - 50 {
        let row_band = world.world_surface as i32 + rng.range(0, 3);
        for y in 50..row_band {
            let mut veto = rng.range(0, 3) == 0;
            if ftw {
                veto = !veto;
            }
            if !veto {
                continue;
            }
            let mut open = false;
            let mut seed: (i32, i32) = (-1, -1);
            // `:25650-25676` — the jungle-grass + mud-wall arm.
            if active_at(world, x, y) && kind_at(world, x, y) == TILE_JUNGLE_GRASS && wall_at(world, x, y) == WALL_MUD {
                for nx in x - 1..=x + 1 {
                    for ny in y - 1..=y + 1 {
                        if wall_at(world, nx, ny) == 0 && !solid_tile(world, nx, ny, false) {
                            open = true;
                        }
                    }
                }
                if open {
                    for nx in x - 1..=x + 1 {
                        for ny in y - 1..=y + 1 {
                            if wall_at(world, nx, ny) == WALL_MUD && !solid_tile(world, nx, ny, false) {
                                seed = (nx, ny);
                            }
                        }
                    }
                }
            }
            // `:25677-25703` — the dirt + grass-wall arm (DST only).
            if active_at(world, x, y) && kind_at(world, x, y) == TILE_GRASS && wall_at(world, x, y) == WALL_GRASS {
                for nx in x - 1..=x + 1 {
                    for ny in y - 1..=y + 1 {
                        if wall_at(world, nx, ny) == 0 && !solid_tile(world, nx, ny, false) {
                            open = true;
                        }
                    }
                }
                if open {
                    for nx in x - 1..=x + 1 {
                        for ny in y - 1..=y + 1 {
                            if wall_at(world, nx, ny) == WALL_GRASS && !solid_tile(world, nx, ny, false) {
                                seed = (nx, ny);
                            }
                        }
                    }
                }
            }
            if open && seed.0 > -1 && seed.1 > -1 {
                spread_wall2(world, seed.0, seed.1, WALL_SPIDER);
            }
        }
    }
}

/// The DST spider-web arm's cobweb placement (`WorldGen.cs:25716-25752`):
/// the wall-62 cells above the rock layer clear their liquid, and 1-in-10
/// (the `Next(10)` veto) place a cobweb if a solid sits in the `Next(2,5)`
/// box. The `TileFrame` rides the framer.
fn dst_cobwebs(world: &mut World, rng: &mut Rng) {
    let width = world.width;
    for x in 200..width - 200 {
        for y in 50..world.rock_layer as i32 {
            if wall_at(world, x, y) == WALL_SPIDER {
                if let Some(t) = world.tile_mut(x, y) {
                    t.liquid = 0;
                    t.flags &= !FLAG_LIQUID_LAVA;
                }
            }
            if wall_at(world, x, y) != WALL_SPIDER || active_at(world, x, y) || rng.range(0, 10) == 0 {
                continue;
            }
            let r = rng.range(2, 5);
            let mut solid_near = false;
            'scan: for nx in x - r..=x + r {
                for ny in y - r..=y + r {
                    if solid_tile(world, nx, ny, false) {
                        solid_near = true;
                        break 'scan;
                    }
                }
            }
            if solid_near {
                place_single(world, x, y, TILE_COBWEB);
            }
        }
    }
}

/// The surface strip (`WorldGen.cs:25754-25791`): above the last cloud row,
/// clear honey + convert the honey drip, then (non-remix) jungle grass → dirt
/// with the vanity-tree roll + grass placement, mud → dirt; remix grows a
/// forest tree instead. `tree_rng` is the host's `UnifiedRandom` bridge.
fn surface_strip(world: &mut World, rng: &mut Rng, tree_rng: &mut UnifiedRandom, cloud_row: i32, remix: bool) {
    let width = world.width;
    for y in 20..cloud_row {
        for x in 20..=width - 20 {
            clear_honey(world, x, y);
            if kind_at(world, x, y) == TILE_HONEY_DRIP {
                set_kind(world, x, y, TILE_WATER_DRIP);
            }
            if !remix {
                if kind_at(world, x, y) == TILE_JUNGLE_GRASS {
                    set_kind(world, x, y, TILE_GRASS);
                    if rng.range(0, 2) == 0 {
                        try_growing_tree_by_type(world, 616, x, y, tree_rng);
                    } else {
                        try_growing_tree_by_type(world, 596, x, y, tree_rng);
                    }
                    if !active_at(world, x, y - 1) {
                        place_single(world, x, y - 1, TILE_PLANTS);
                    }
                }
                if kind_at(world, x, y) == TILE_MUD {
                    set_kind(world, x, y, TILE_DIRT);
                }
            } else {
                // `:25788` — TryGrowingTreeByType(5) dispatches to GrowTree
                // (`:30348-30350`); the port's settings dispatch has no kind-5
                // arm, so call grow_tree directly (same target).
                grow_tree(world, x, y, tree_rng);
            }
        }
    }
}

/// `WorldGen.OverGrownDungeon` (`WorldGen.cs:23971-24035`): per-column, a
/// random dungeon-wall anchor grows jungle grass over the enclosed bricks,
/// with a tree/jungle-plant above and a 1-in-3 vine cascade below. The port's
/// dungeon side is fixed Right, so the Left anchor draw is overwritten (both
/// draws run, the C# order). Runs after the `tileSolid[379]` restore.
fn over_grown_dungeon(world: &mut World, rng: &mut Rng, tree_rng: &mut UnifiedRandom) {
    let (width, height) = (world.width, world.height);
    for _ in 0..width {
        // `:23976-23980` — the Left anchor draw, then the Right overwrite.
        let mut num = rng.range(50, width / 2);
        num = rng.range(width / 2, width - 50);
        let num2 = rng.range(50, height - 50);
        if !WALL_DUNGEON.contains(&wall_at(world, num, num2)) {
            continue;
        }
        let num3 = rng.range(5, 16);
        for j in num - num3..=num + num3 {
            for k in num2 - num3..=num2 + num3 {
                if !active_at(world, j, k) || !TILE_DUNGEON.contains(&kind_at(world, j, k)) {
                    continue;
                }
                let flag = !active_at(world, j - 1, k - 1) || !active_at(world, j, k - 1) || !active_at(world, j + 1, k - 1);
                // `:23996` — the enclosed-brick test (dualDungeons reads
                // false, so the second disjunct never fires).
                let enclosed = !flag
                    && active_at(world, j - 1, k)
                    && active_at(world, j + 1, k)
                    && active_at(world, j - 1, k + 1)
                    && active_at(world, j, k + 1)
                    && active_at(world, j + 1, k - 1);
                if !enclosed {
                    continue;
                }
                set_kind(world, j, k, TILE_JUNGLE_GRASS);
                if let Some(t) = world.tile_mut(j, k) {
                    t.set_paint(0);
                }
                if !active_at(world, j, k - 1) && !grow_tree(world, j, k, tree_rng) {
                    place_single(world, j, k - 1, TILE_JUNGLE_PLANTS);
                }
                if active_at(world, j, k + 1) || rng.range(0, 3) == 0 {
                    continue;
                }
                place_single(world, j, k + 1, TILE_JUNGLE_VINES);
                if active_at(world, j, k + 2) {
                    continue;
                }
                place_single(world, j, k + 2, TILE_JUNGLE_VINES);
                if !active_at(world, j, k + 3) {
                    place_single(world, j, k + 3, TILE_JUNGLE_VINES);
                    if !active_at(world, j, k + 4) && rng.range(0, 4) != 0 {
                        place_single(world, j, k + 4, TILE_JUNGLE_VINES);
                    }
                    if !active_at(world, j, k + 5) && rng.range(0, 3) != 0 {
                        place_single(world, j, k + 5, TILE_JUNGLE_VINES);
                    }
                    if !active_at(world, j, k + 6) && rng.range(0, 2) != 0 {
                        place_single(world, j, k + 6, TILE_JUNGLE_VINES);
                    }
                }
            }
        }
    }
}

/// The FTW+NTB `SpiderWallRemover` search loops (`WorldGen.cs:25407-25435`):
/// `maxTilesX/4` deep-band rolls + `maxTilesX/3` mid-band rolls, each a
/// ≤10000-draw wall-62 search then `SpiderWallRemover(x, y)` — called even
/// when the search fails (the C# falls through with the last roll). The
/// sub-vanilla guard skips an inverted band before any draw (the C# throws).
fn spider_wall_search(world: &mut World, rng: &mut Rng) {
    let (width, height) = (world.width, world.height);
    let underworld = height - 200; // Main.UnderworldLayer (Main.cs:2790)
    let surface = world.world_surface as i32;
    // `:25409-25421` — the deep band (UnderworldLayer+10 .. maxTilesY-20).
    let (deep_lo, deep_hi) = (underworld + 10, height - 20);
    if deep_hi > deep_lo {
        for _ in 0..width / 4 {
            let mut x = rng.range(20, width - 20);
            let mut y = rng.range(deep_lo, deep_hi);
            let mut budget = 10000;
            while wall_at(world, x, y) != WALL_SPIDER && budget > 0 {
                budget -= 1;
                x = rng.range(20, width - 20);
                y = rng.range(deep_lo, deep_hi);
            }
            spider_wall_remover(world, rng, x, y);
        }
    }
    // `:25422-25434` — the mid band (worldSurface .. UnderworldLayer-30).
    let (mid_lo, mid_hi) = (surface, underworld - 30);
    if mid_hi > mid_lo {
        for _ in 0..width / 3 {
            let mut x = rng.range(20, width - 20);
            let mut y = rng.range(mid_lo, mid_hi);
            let mut budget = 10000;
            while wall_at(world, x, y) != WALL_SPIDER && budget > 0 {
                budget -= 1;
                x = rng.range(20, width - 20);
                y = rng.range(mid_lo, mid_hi);
            }
            spider_wall_remover(world, rng, x, y);
        }
    }
}

/// `WorldGen.FinishNotTheBees` (`WorldGen.cs:25388-25797`): the not-the-bees
/// host — the cloud-row scan, the FTW+NTB spider-wall remover search loops
/// (wiring the W4-g41 `spider_wall_remover`), the main conversion sweep, the
/// DST spider-web arm, the surface strip and `OverGrownDungeon`. Draws
/// exclusively on `pass_rng("FinishNotTheBees")`; the tree-growth bridge
/// seeds one `UnifiedRandom` from that stream (the module note).
pub(crate) fn finish_not_the_bees(world: &mut World, gv: &GenVars, streams: &mut Streams, opts: &GenOptions) {
    let flags = opts.flags;
    // `:25390-25393` — the seed/skyblock gate.
    if !flags.contains(SeedFlags::NOT_THE_BEES) || flags.contains(SeedFlags::SKYBLOCK) {
        return;
    }
    let mut rng = streams.pass_rng("FinishNotTheBees");
    // The tree-growth bridge (the module note): one UnifiedRandom seeded from
    // the host stream, shared by the surface strip + OverGrownDungeon.
    let mut tree_rng = UnifiedRandom::new(rng.next_u32() as i32);
    let (width, height) = (world.width, world.height);
    let lava_line = gv.lava_line;
    let remix = flags.contains(SeedFlags::REMIX);
    let dst = flags.contains(SeedFlags::DONT_STARVE);
    let ftw = flags.contains(SeedFlags::FOR_THE_WORTHY);
    let tenth = flags.contains(SeedFlags::TENTH_ANNIVERSARY);
    // `:25394` — tileSolid[379] = false (the in-window solid tests veto 379;
    // the boot table is immutable, so nothing is restored after).
    // `:25396-25406` — the cloud-row scan (keeps the LAST cloud row).
    let mut cloud_row = 0i32;
    let mut i = 20i32;
    while (i as f64) < world.world_surface {
        for j in 20..width - 20 {
            if world.tile(j, i).is_some_and(|t| t.active() && td::SET_CLOUDS.has(t.kind)) {
                cloud_row = i;
                break;
            }
        }
        i += 1;
    }
    // `:25407-25435` — the FTW+NTB spider-wall remover search loops.
    if ftw && !tenth {
        spider_wall_search(world, &mut rng);
    }
    // `:25436-25630` — the main conversion sweep.
    for m in 25..width - 25 {
        for n in 25..height - 25 {
            finish_cell(world, &mut rng, m, n, cloud_row, lava_line, flags);
        }
    }
    // `:25632-25752` — the DST spider-web arm (the noSpiderCaves short-list
    // veto reads the micro flags).
    if dst && !remix && !no_spider_caves_actually(opts.micro_flags, flags) {
        dst_surface_seeds(world, &mut rng, flags);
        dst_cobwebs(world, &mut rng);
    }
    // `:25754-25791` — the surface strip.
    surface_strip(world, &mut rng, &mut tree_rng, cloud_row, remix);
    // `:25792` — tileSolid[379] = true (the boot table already carries it).
    // `:25793-25796` — OverGrownDungeon (!remix).
    if !remix {
        over_grown_dungeon(world, &mut rng, &mut tree_rng);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::LiquidKind;

    /// A small all-air test world with the surface/rock lines pinned.
    fn air(width: i32, height: i32, surface: f64, rock: f64) -> World {
        let mut world = World::new("ntb".into(), width, height);
        world.world_surface = surface;
        world.rock_layer = rock;
        world
    }

    fn put(world: &mut World, x: i32, y: i32, kind: u16) {
        let i = world.index(x, y);
        world.tiles[i].kind = kind;
        world.tiles[i].flags |= FLAG_ACTIVE;
    }

    fn wall(world: &mut World, x: i32, y: i32, wall: u16) {
        let i = world.index(x, y);
        world.tiles[i].wall = wall;
    }

    fn opts(flags: u16) -> GenOptions {
        GenOptions {
            flags: SeedFlags::from(flags),
            ..Default::default()
        }
    }

    fn gv(lava_line: i32) -> GenVars {
        GenVars {
            lava_line,
            ..Default::default()
        }
    }

    /// The seed gate: a classic world never enters either function — the
    /// tiles are byte-identical and no stream draw lands.
    #[test]
    fn classic_world_is_untouched() {
        let mut w = air(200, 160, 60.0, 90.0);
        put(&mut w, 100, 70, TILE_JUNGLE_GRASS);
        wall(&mut w, 100, 70, WALL_MUD);
        let before = w.tiles.clone();
        let o = opts(0);
        let s = &mut Streams::new(7);
        not_the_bees(&mut w, &gv(100), s, &o);
        finish_not_the_bees(&mut w, &gv(100), s, &o);
        assert_eq!(w.tiles, before, "a classic world reads no NTB flag");
    }

    /// The skyblock gate: both functions return before any draw.
    #[test]
    fn skyblock_skips_the_host() {
        let mut w = air(200, 160, 60.0, 90.0);
        put(&mut w, 100, 70, TILE_JUNGLE_GRASS);
        let before = w.tiles.clone();
        let o = opts(SeedFlags::NOT_THE_BEES | SeedFlags::SKYBLOCK);
        let s = &mut Streams::new(7);
        not_the_bees(&mut w, &gv(100), s, &o);
        finish_not_the_bees(&mut w, &gv(100), s, &o);
        assert_eq!(w.tiles, before, "skyblock skips the conversion");
    }

    /// The vines → hive conversion (`:25187-25190`): a vine 52 / vine-flower
    /// 382 cell becomes jungle vine 62 regardless of solidity.
    #[test]
    fn vines_convert_to_hive() {
        let mut w = air(900, 400, 100.0, 180.0);
        put(&mut w, 450, 120, TILE_VINES);
        put(&mut w, 451, 120, TILE_VINE_FLOWERS);
        let o = opts(SeedFlags::NOT_THE_BEES);
        not_the_bees(&mut w, &gv(300), &mut Streams::new(3), &o);
        assert_eq!(w.tiles[w.index(450, 120)].kind, TILE_JUNGLE_VINES);
        assert_eq!(w.tiles[w.index(451, 120)].kind, TILE_JUNGLE_VINES);
    }

    /// The slush → honey-block arm (`:25213-25218`): a solid slush 224 cell
    /// converts on a plain NTB world (!dst).
    #[test]
    fn slush_converts_to_honey_block() {
        let mut w = air(900, 400, 100.0, 180.0);
        put(&mut w, 450, 120, TILE_SLUSH);
        let o = opts(SeedFlags::NOT_THE_BEES);
        not_the_bees(&mut w, &gv(300), &mut Streams::new(3), &o);
        assert_eq!(w.tiles[w.index(450, 120)].kind, TILE_HONEY_BLOCK);
    }

    /// The dirt → mud arm (`:25246-25249`): a solid dirt cell inland converts
    /// to mud 59.
    #[test]
    fn dirt_converts_to_mud() {
        let mut w = air(900, 400, 100.0, 180.0);
        put(&mut w, 450, 120, TILE_DIRT);
        let o = opts(SeedFlags::NOT_THE_BEES);
        not_the_bees(&mut w, &gv(300), &mut Streams::new(3), &o);
        assert_eq!(w.tiles[w.index(450, 120)].kind, TILE_MUD);
    }

    /// The grass → jungle/mushroom band (`:25235-25245`): a grass cell above
    /// the lava line becomes jungle grass 60, below becomes mushroom 70.
    #[test]
    fn grass_converts_by_the_lava_line() {
        let mut w = air(900, 400, 100.0, 180.0);
        put(&mut w, 450, 120, TILE_GRASS); // above lava line 300
        put(&mut w, 450, 350, TILE_GRASS); // below lava line 300
        let o = opts(SeedFlags::NOT_THE_BEES);
        not_the_bees(&mut w, &gv(300), &mut Streams::new(3), &o);
        assert_eq!(w.tiles[w.index(450, 120)].kind, TILE_JUNGLE_GRASS);
        assert_eq!(w.tiles[w.index(450, 350)].kind, 70);
    }

    /// The wall conversion (`:25329-25336`): a grass wall 63 becomes jungle
    /// wall 64; a plain stone wall 1 below the deep-hive line becomes hive 86.
    #[test]
    fn walls_convert_to_jungle_and_hive() {
        let mut w = air(900, 400, 100.0, 180.0);
        wall(&mut w, 450, 120, WALL_GRASS);
        wall(&mut w, 450, 130, 1); // stone wall, above num2 = (300+400-180)/2 = 260
        let o = opts(SeedFlags::NOT_THE_BEES);
        not_the_bees(&mut w, &gv(300), &mut Streams::new(3), &o);
        assert_eq!(w.tiles[w.index(450, 120)].wall, WALL_JUNGLE);
        assert_eq!(w.tiles[w.index(450, 130)].wall, WALL_HIVE);
    }

    /// The liquid honey arm (`:25339-25357`): an inland water cell above the
    /// lava line turns to honey; the centre strip keeps water.
    #[test]
    fn liquid_turns_to_honey_inland() {
        let mut w = air(900, 400, 100.0, 180.0);
        let i = w.index(700, 120);
        w.tiles[i].liquid = 100; // inland (x > 0.6*900 = 540), above lava line
        let c = w.index(450, 120);
        w.tiles[c].liquid = 100; // centre strip (0.4*900 < 450 < 0.6*900)
        let o = opts(SeedFlags::NOT_THE_BEES);
        not_the_bees(&mut w, &gv(300), &mut Streams::new(3), &o);
        assert_eq!(w.tiles[i].liquid_kind(), LiquidKind::Honey);
        assert_eq!(w.tiles[c].liquid_kind(), LiquidKind::Water);
    }

    /// The FTW altar → crispy-honey arm (`:25445-25448`) and the DST/noTraps
    /// spikes → wooden-spikes arm (`:25455-25458`) of the main sweep.
    #[test]
    fn finish_altar_and_spikes_arms() {
        let mut w = air(900, 400, 100.0, 180.0);
        put(&mut w, 450, 120, TILE_HELLSTONE_BRICK);
        put(&mut w, 450, 130, TILE_SPIKES);
        let o = opts(SeedFlags::NOT_THE_BEES | SeedFlags::FOR_THE_WORTHY | SeedFlags::NO_TRAPS);
        finish_not_the_bees(&mut w, &gv(300), &mut Streams::new(3), &o);
        assert_eq!(w.tiles[w.index(450, 120)].kind, TILE_CRISPY_HONEY);
        assert_eq!(w.tiles[w.index(450, 130)].kind, TILE_WOODEN_SPIKES);
    }

    /// The deep ore demotion (`:25586-25607`): an iron cell deep below the
    /// `(lavaLine + rockLayer*2)/3` line becomes dirt.
    #[test]
    fn deep_ore_demotes_to_dirt() {
        let mut w = air(900, 400, 100.0, 180.0);
        // lava_line 300, rock 180: the iron line is (300+360)/3 = 220; a cell
        // at y=350 is well below it (the ±2 jitter cannot reach).
        put(&mut w, 450, 350, TILE_IRON);
        let o = opts(SeedFlags::NOT_THE_BEES);
        finish_not_the_bees(&mut w, &gv(300), &mut Streams::new(3), &o);
        assert_eq!(w.tiles[w.index(450, 350)].kind, TILE_DIRT);
    }

    /// The FTW+NTB spider-wall remover wiring (`:25407-25435`): a spider-wall
    /// block in the mid band is eroded by the search loops + the remover. The
    /// host is deterministic for a seed (a == b).
    #[test]
    fn the_host_wires_the_spider_wall_remover() {
        let build = |seed: u64| {
            let mut w = air(900, 400, 100.0, 180.0);
            // A spider-wall block in the mid band (worldSurface 100 ..
            // UnderworldLayer-30 = 170): the search rolls land inside it.
            for px in 300..600 {
                for py in 110..160 {
                    let i = w.index(px, py);
                    w.tiles[i].wall = WALL_SPIDER;
                }
            }
            let o = opts(SeedFlags::NOT_THE_BEES | SeedFlags::FOR_THE_WORTHY);
            finish_not_the_bees(&mut w, &gv(300), &mut Streams::new(seed), &o);
            w
        };
        let a = build(0xBEE);
        let b = build(0xBEE);
        assert_eq!(a.tiles, b.tiles, "the host is deterministic for a seed");
        let spider_left = a.tiles.iter().filter(|t| t.wall == WALL_SPIDER).count();
        assert!(spider_left < 300 * 50, "the remover cleared some spider wall ({spider_left})");
    }

    /// The FTW gate: without FOR_THE_WORTHY the search loops never run, so a
    /// spider-wall block survives the host (the main sweep does not touch
    /// wall 62).
    #[test]
    fn the_remover_search_needs_for_the_worthy() {
        let mut w = air(900, 400, 100.0, 180.0);
        for px in 300..600 {
            for py in 110..160 {
                let i = w.index(px, py);
                w.tiles[i].wall = WALL_SPIDER;
            }
        }
        let o = opts(SeedFlags::NOT_THE_BEES);
        finish_not_the_bees(&mut w, &gv(300), &mut Streams::new(0xBEE), &o);
        let spider_left = w.tiles.iter().filter(|t| t.wall == WALL_SPIDER).count();
        assert_eq!(spider_left, 300 * 50, "no FTW flag, no remover search");
    }

    /// `Spread.Wall2` (`:3416-3524`): the wall-62 flood stops at air (the
    /// `WallSpreadStopsAtAir` set) and converts walled non-solid cells; a
    /// solid cell takes the wall without spreading past it.
    #[test]
    fn spread_wall2_floods_walled_cells_and_stops_at_air() {
        let mut w = air(120, 120, 40.0, 60.0);
        // A walled corridor (wall 1) surrounded by air (wall 0).
        for px in 40..80 {
            wall(&mut w, px, 60, 1);
        }
        spread_wall2(&mut w, 50, 60, WALL_SPIDER);
        assert_eq!(w.tiles[w.index(50, 60)].wall, WALL_SPIDER);
        assert_eq!(w.tiles[w.index(79, 60)].wall, WALL_SPIDER);
        // The air beyond the corridor stays wall 0 (the flood stops at air).
        assert_eq!(w.tiles[w.index(85, 60)].wall, 0);
        assert_eq!(w.tiles[w.index(50, 55)].wall, 0);
    }

    /// `Spread.Wall2`'s solid-cell arm (`:3518-3521`): a solid cell takes the
    /// wall but does not spread — the cell beyond it keeps its own wall.
    #[test]
    fn spread_wall2_solid_cell_takes_the_wall_without_spreading() {
        let mut w = air(120, 120, 40.0, 60.0);
        wall(&mut w, 50, 60, 1);
        put(&mut w, 51, 60, TILE_STONE); // solid blocker
        wall(&mut w, 51, 60, 1);
        wall(&mut w, 52, 60, 1);
        spread_wall2(&mut w, 50, 60, WALL_SPIDER);
        assert_eq!(w.tiles[w.index(50, 60)].wall, WALL_SPIDER);
        assert_eq!(w.tiles[w.index(51, 60)].wall, WALL_SPIDER, "the solid cell takes the wall");
        assert_eq!(w.tiles[w.index(52, 60)].wall, 1, "the flood does not spread past the solid");
    }

    /// The larva `Place3x3` (`:53534-53634`): a 3×3 larva lands on a solid
    /// floor with the two rows above clear; an occupied roof refuses it.
    #[test]
    fn place_larva_needs_a_clear_roof_and_solid_floor() {
        let mut w = air(120, 120, 40.0, 60.0);
        for px in 49..52 {
            put(&mut w, px, 63, TILE_STONE); // the floor at y+1
        }
        place_larva(&mut w, 50, 62);
        assert_eq!(w.tiles[w.index(50, 62)].kind, TILE_LARVA);
        assert_eq!(w.tiles[w.index(49, 60)].kind, TILE_LARVA);
        // An occupied roof cell refuses the placement.
        let mut blocked = air(120, 120, 40.0, 60.0);
        for px in 49..52 {
            put(&mut blocked, px, 63, TILE_STONE);
        }
        put(&mut blocked, 50, 61, TILE_STONE); // roof blocker
        place_larva(&mut blocked, 50, 62);
        assert_ne!(blocked.tiles[blocked.index(50, 62)].kind, TILE_LARVA);
    }

    /// The `no_spider_caves_actually` veto (`:270-278`): a short secret-seed
    /// list with noSpiderCaves reads true; a long list (≥4) reads false.
    #[test]
    fn the_no_spider_caves_veto_reads_the_secret_count() {
        let micro = MicroSeedFlags::from(MicroSeedFlags::NO_SPIDER_CAVES);
        assert!(no_spider_caves_actually(micro, SeedFlags::none()));
        let long = MicroSeedFlags::from(
            MicroSeedFlags::NO_SPIDER_CAVES
                | MicroSeedFlags::PAINT_EVERYTHING_GRAY
                | MicroSeedFlags::PAINT_EVERYTHING_NEGATIVE
                | MicroSeedFlags::COAT_EVERYTHING_ECHO,
        );
        assert!(!no_spider_caves_actually(long, SeedFlags::none()));
        assert!(!no_spider_caves_actually(MicroSeedFlags::from(0), SeedFlags::none()));
    }

    /// The full-pipeline a == b determinism witness: an NTB+FTW world generates
    /// byte-identically across two runs (the host's draws are deterministic),
    /// and differs from a classic world (the conversion actually runs). Written
    /// NOT run (code-only).
    #[test]
    fn the_ntb_world_is_reproducible_and_distinct_from_classic() {
        let ntb = GenOptions {
            width: 512,
            height: 512,
            seed: 42,
            flags: SeedFlags::from(SeedFlags::NOT_THE_BEES | SeedFlags::FOR_THE_WORTHY),
            ..Default::default()
        };
        let a = crate::worldgen::generate(&ntb);
        let b = crate::worldgen::generate(&ntb);
        assert_eq!(a.tiles, b.tiles, "the NTB+FTW world is deterministic");
        let classic = GenOptions {
            width: 512,
            height: 512,
            seed: 42,
            ..Default::default()
        };
        let c = crate::worldgen::generate(&classic);
        assert_ne!(a.tiles, c.tiles, "the NTB conversion changes the world");
    }
}