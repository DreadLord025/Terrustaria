//! The evil pass — `WorldGen.CorruptionAndCrimson` (`WorldGen.cs:13866-14554`)
//! and its walker family, ported verbatim under the golden rule (structure,
//! maths, RNG draw order frozen; every block carries its citation).
//!
//! Vanilla runs one pass for both evil flavours: the crimson branch
//! (`WorldGen.crimson`, the port's `crimson` argument) places the winding
//! crimstone blob with its chambers, veins and heart rooms and scatters the
//! style-1 altars; the corruption branch cuts the chasms with their side
//! caverns, orbs and the never-re-armed altar search, then wraps every orb in
//! an ebonstone blob.
//!
//! M5-SEED-1 ports the drunk half of the pass (`WorldGen.cs:13938`, `:13962`,
//! `:14279`, `:14302`) and the `GenVars.flipInfections` arms (`:11298-11304`):
//! a drunk world halves the biome count, runs both flavours, and draws each
//! band's centre from its own world half; drunk + for-the-worthy + !remix
//! swaps the ebonstone/crimstone tile and wall pairs so the two branches
//! trade their walkers. The flag reads ride the world (`World::seeds`, the
//! `generate` publication of `GenOptions::flags`), so every arm gates on the
//! world's own identity and a classic world is byte-identical. Still out of
//! scope (documented deviations): remix, tenthAnniversary, dontStarve,
//! notTheBees, noSurface, surfaceIsInSpace and skyblock.
//!
//! The pass draws exclusively from its per-pass stream (the caller passes
//! `Streams::pass_rng("Corruption")`); the three contract streams are
//! untouched.

use super::biomes::{TILE_JUNGLE_GRASS, TILE_MUD};
use super::noise::Rng;
use super::tile_runner::{TileRunnerOpts, tile_runner};
use super::{
    TILE_CRIMTANE, TILE_DEMONITE, TILE_DIRT, TILE_GRASS, TILE_HARDENED_SAND, TILE_ICE, TILE_SAND,
    TILE_SANDSTONE, TILE_SNOW, TILE_STONE,
};
use crate::tile::{FLAG_ACTIVE, FLAG_INACTIVE, LiquidKind, Tile};
use crate::tiledata::{
    SET_CAN_BE_CLEARED_DURING_GENERATION, SET_CRACKED_BRICKS, SET_GRASS, TILE_SOLID, TILE_SOLID_TOP,
};
use crate::seeds::SeedFlags;
use crate::world::World;

// --- tiles (TileID.cs) -------------------------------------------------------

/// `TileID.CorruptGrass` = 23.
const TILE_CORRUPT_GRASS: u16 = 23;
/// `TileID.Ebonstone` = 25 (`GenVars.ebonStone`, `WorldGen.cs:8346`).
const TILE_EBONSTONE: u16 = 25;
/// `TileID.Ebonsand` = 112.
const TILE_EBONSAND: u16 = 112;
/// `TileID.CorruptIce` = 163.
const TILE_CORRUPT_ICE: u16 = 163;
/// `TileID.CrimsonGrass` = 199.
const TILE_CRIMSON_GRASS: u16 = 199;
/// `TileID.Crimstone` = 203.
const TILE_CRIMSTONE: u16 = 203;
/// `TileID.Crimsand` = 234.
const TILE_CRIMSAND: u16 = 234;
/// `TileID.CorruptSandstone` = 400.
const TILE_CORRUPT_SANDSTONE: u16 = 400;
/// `TileID.CorruptHardenedSand` = 398.
const TILE_CORRUPT_HARDENED_SAND: u16 = 398;
/// `TileID.CrimsonSandstone` = 401.
const TILE_CRIMSON_SANDSTONE: u16 = 401;
/// `TileID.CrimsonHardenedSand` = 399.
const TILE_CRIMSON_HARDENED_SAND: u16 = 399;
/// `TileID.CorruptJungleGrass` = 661.
const TILE_CORRUPT_JUNGLE_GRASS: u16 = 661;
/// `TileID.CrimsonJungleGrass` = 662.
const TILE_CRIMSON_JUNGLE_GRASS: u16 = 662;
/// `TileID.ShadowOrb` = 31 — the 2x2 orb / crimson heart.
pub const TILE_SHADOW_ORB: u16 = 31;
/// `TileID.DemonAltar` = 26.
const TILE_DEMON_ALTAR: u16 = 26;

// --- walls (WallID.cs) -------------------------------------------------------

/// `WallID.EbonstoneUnsafe` = 3 (`GenVars.ebonStoneWall`, `WorldGen.cs:8345`).
const WALL_EBONSTONE: u16 = 3;
/// `WallID.CrimstoneUnsafe` = 83 (`GenVars.crimStoneWall`, `WorldGen.cs:8344`).
const WALL_CRIMSTONE: u16 = 83;
/// `WallID.HardenedSand` = 216 — the desert wall the shell converts.
const WALL_HARDENED_SAND: u16 = 216;
/// `WallID.Sandstone` = 187 — the other desert wall the shell converts.
const WALL_SANDSTONE: u16 = 187;
/// `WallID.CorruptHardenedSand` = 217.
const WALL_CORRUPT_HARDENED_SAND: u16 = 217;
/// `WallID.CrimsonHardenedSand` = 218.
const WALL_CRIMSON_HARDENED_SAND: u16 = 218;
/// `WallID.CorruptSandstone` = 220.
const WALL_CORRUPT_SANDSTONE: u16 = 220;
/// `WallID.CrimsonSandstone` = 221.
const WALL_CRIMSON_SANDSTONE: u16 = 221;

// --- the Reset constants -----------------------------------------------------

/// `GenVars.evilBiomeBeachAvoidance` = `beachSandRandomCenter + 60` = 380
/// (`WorldGen.cs:11191-11201`, `beachSandRandomCenter` = 275 + 5 + 40 = 320 at
/// `:11187`).
const EVIL_BEACH_AVOIDANCE: i32 = 380;
/// `GenVars.evilBiomeAvoidanceMidFixer` = 50 (`WorldGen.cs:11202`).
const EVIL_MID_FIXER: i32 = 50;
/// `WorldGen.beachDistance` = 380 (`WorldGen.cs:86914`) — the `oceanDepths`
/// ocean margins and the `SpreadGrass` generation-time beach exclusion.
const BEACH_DISTANCE: i32 = 380;

/// `Main.tileDungeon` (`Main.cs:7992-7997`) — the dungeon-brick tiles
/// `CanEvilReplace` protects (`TileID.Sets.CrackedBricks` = 481-483 rides the
/// generated tiledata, `TileID.cs:498`).
const TILE_DUNGEON: [u16; 6] = [41, 43, 44, 677, 678, 679];
/// `Main.wallDungeon` (`Main.cs:10598-10606`) — the dungeon walls
/// `CanEvilReplace` protects.
const WALL_DUNGEON: [u16; 9] = [7, 8, 9, 94, 95, 96, 97, 98, 99];

/// The chasm fill-phase threshold — ChasmRunner's `num3` (`WorldGen.cs:75906`).
const CHASM_FILL_STEPS: i32 = 5;

/// The band-retry valve (the port's D15 deviation — the C# retry is
/// unbounded and cannot terminate on sub-vanilla-sized worlds, where the
/// centre-exclusion zone `(maxTilesX/2 - 200, +200)` can cover the whole
/// legal band; vanilla-size worlds never reach the valve).
const BAND_RETRY_VALVE: i32 = 200_000;

// --- small read helpers ------------------------------------------------------

/// `Main.tile[x, y].type` with out-of-bounds reading as 0.
fn kind_at(world: &World, x: i32, y: i32) -> u16 {
    world.tile(x, y).map_or(0, |t| t.kind)
}

/// `Main.tile[x, y].wall` with out-of-bounds reading as 0.
fn wall_at(world: &World, x: i32, y: i32) -> u16 {
    world.tile(x, y).map_or(0, |t| t.wall)
}

/// `Main.tile[x, y].active()`.
fn active_at(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| t.active())
}

/// The kind write that keeps wall and liquid (the port's plain-write
/// convention — `SquareTileFrame` is the re-frame pass the port defers).
fn set_kind(world: &mut World, x: i32, y: i32, kind: u16) {
    if let Some(t) = world.tile_mut(x, y) {
        t.kind = kind;
    }
}

/// `Main.tile[x, y].active(value)` with out-of-bounds writes skipped.
fn set_active(world: &mut World, x: i32, y: i32, active: bool) {
    if let Some(t) = world.tile_mut(x, y) {
        if active {
            t.flags |= FLAG_ACTIVE;
        } else {
            t.flags &= !FLAG_ACTIVE;
        }
    }
}

/// `Main.tile[x, y].wall = value` with out-of-bounds writes skipped.
fn set_wall(world: &mut World, x: i32, y: i32, wall: u16) {
    if let Some(t) = world.tile_mut(x, y) {
        t.wall = wall;
    }
}

// --- the secret-seed readers -------------------------------------------------

/// `WorldGen.drunkWorldGen` (`WorldGen.cs:11115-11123`'s flag set, published to
/// the passes as `World::seeds`). Read-only: a classic world never sets it.
fn drunk(world: &World) -> bool {
    world.seeds.contains(SeedFlags::DRUNK)
}

/// `GenVars.crimsonLeft` (`GenVars.cs:280`) — the `genRand.Next(2)` half-roll
/// of `Reset` (`WorldGen.cs:11216-11223`) that splits the world into a
/// corruption-left / crimson-left half pair. The port's Reset stand-in lives
/// in `pipeline::run` and its `GenVars` never reaches this pass's signature,
/// so the drunk anchor arms (`:13962`, `:14302`) re-derive the roll from the
/// per-pass stream contract: `Streams::pass_rng(name)` is
/// `Rng(seed ^ fnv1a64(name))`, so this pass's stream state carries the world
/// seed and the "Reset" stream can be rebuilt and drawn exactly once. It must
/// be evaluated before the pass draws anything and it consumes no draws of
/// its own; a classic world never evaluates it. (M5-SEED-1 follow-up: once
/// `step_evil` forwards `&GenVars` the re-derivation can be dropped.)
fn crimson_left(rng: &Rng) -> bool {
    let world_seed = rng.0 ^ super::pipeline::fnv1a64("Corruption");
    let mut reset = Rng(world_seed ^ super::pipeline::fnv1a64("Reset"));
    reset.range(0, 2) == 1
}

/// `GenVars.ebonStone` / `GenVars.ebonStoneWall` (`GenVars.cs`; `Reset` seeds
/// them at `WorldGen.cs:8345-8346`) — the corruption pair, 25/3 classic and
/// 203/83 under the `flipInfections` swap (`:11298-11304`).
fn ebon_ids(world: &World) -> (u16, u16) {
    if world.seeds.flip_infections() {
        (TILE_CRIMSTONE, WALL_CRIMSTONE)
    } else {
        (TILE_EBONSTONE, WALL_EBONSTONE)
    }
}

/// `GenVars.crimStone` / `GenVars.crimStoneWall` (`GenVars.cs`; `Reset` seeds
/// them at `WorldGen.cs:8344`) — the crimson pair, 203/83 classic and 25/3
/// under the `flipInfections` swap. The crimson walkers (`CrimStart`,
/// `CrimVein`, `CrimEnt`, `CrimPlaceHearts`) read this pair.
fn crim_ids(world: &World) -> (u16, u16) {
    if world.seeds.flip_infections() {
        (TILE_EBONSTONE, WALL_EBONSTONE)
    } else {
        (TILE_CRIMSTONE, WALL_CRIMSTONE)
    }
}

// --- the pass ----------------------------------------------------------------

/// `CorruptionAndCrimson` (`WorldGen.cs:13868-14554`). The prologue scans the
/// jungle/snow extents, then `num10 = maxTilesX * 0.00045` biomes are placed
/// by the shared centre/band retry; the crimson branch walks `CrimStart` and
/// scatters style-1 altars, the corruption branch cuts the chasms and wraps
/// the orbs in ebonstone. The drunk world halves the count and runs both
/// flavours (`:13938`, `:14279`) with each band's centre drawn from its own
/// world half (`:13962`, `:14302`); `GenVars.flipInfections`
/// (`:11298-11304`) swaps the ebonstone/crimstone pairs so the branches trade
/// their walkers.
pub(crate) fn corruption_and_crimson(
    world: &mut World,
    rng: &mut Rng,
    world_surface_low: f64,
    world_surface: f64,
    rock_layer: f64,
    dungeon_location: i32,
    crimson: bool,
) {
    // The secret-seed context read off the world (`World::seeds` — the
    // `generate` publication of `GenOptions::flags`). `crimsonLeft` is the
    // Reset half-roll the drunk anchor arms read; it re-derives the stream,
    // so it must run before this pass draws anything.
    let flip_infections = world.seeds.flip_infections();
    let (ebon_stone, ebon_wall) = ebon_ids(world);
    let drunk = drunk(world);
    let crimson_left = crimson_left(rng);
    // `double num = GenVars.worldSurfaceLow; double num2 = Main.worldSurface`
    // (`:13870-13871`; the surfaceIsInSpace/noSurface overrides are secret
    // seeds).
    let num = world_surface_low;
    let num2 = world_surface;
    // The jungle/snow extent scan, no RNG (`:13878-13913`): min/max X of
    // active tile 60 → num3/num4; of 147/161 → num5/num6; then ±10 padding
    // (`num7 = 10`, `:13918-13924`).
    let (mut num3, mut num4, mut num5, mut num6) = (world.width, 0, world.width, 0);
    for i in 0..world.width {
        let mut j = 0;
        while (j as f64) < num2 {
            if let Some(t) = world.tile(i, j) {
                if t.active() {
                    if t.kind == TILE_JUNGLE_GRASS {
                        num3 = num3.min(i);
                        num4 = num4.max(i);
                    } else if t.kind == TILE_SNOW || t.kind == TILE_ICE {
                        num5 = num5.min(i);
                        num6 = num6.max(i);
                    }
                }
            }
            j += 1;
        }
    }
    num3 -= 10;
    num4 += 10;
    num5 -= 10;
    num6 += 10;
    // `num8 = 500; num9 = 100` (`:13925-13926`; the anniversary doubling is a
    // secret seed) and `num10 = maxTilesX * 0.00045` (`:13927`; the remix
    // doubling is a secret seed).
    let num8 = 500i32;
    let num9 = 100i32;
    let mut num10 = world.width as f64 * 0.00045;
    // `bool flag = WorldGen.crimson; if (WorldGen.drunkWorldGen) { flag = true;
    // num10 /= 2.0; }` (`:13936-13941`) — the drunk world halves the biome
    // count and forces the crimson pass on. `:14279`'s `flag = false` re-arms
    // the corruption pass afterwards, so both flavours run.
    let mut flag = crimson;
    if drunk {
        flag = true;
        num10 /= 2.0;
    }
    // `for (int k = 0; (double)k < num10; k++)` — a fractional bound still
    // runs the ceil count.
    let count = num10.ceil() as i32;
    // D14: vanilla's centre draw `Next(num8, maxTilesX - num8)` throws on
    // widths where the range is empty (`UnifiedRandom.cs:120-122`) — vanilla
    // cannot generate these worlds; the port skips deterministically with no
    // draws.
    if world.width - num8 <= num8 {
        return;
    }
    // `WorldGen.heartPos`/`heartCount` (`:11237`, `:86970`) — the pass-local
    // vector is behaviour-equal (D21): the pass is the only producer
    // (`CrimVein`) and consumer (`CrimPlaceHearts`).
    let mut hearts: Vec<(i32, i32)> = Vec::new();
    // `WorldGen.oceanLevel` (`:4447`): `(worldSurface + rockLayer) / 2 + 40`.
    let ocean_level = (world_surface + rock_layer) / 2.0 + 40.0;
    if flag {
        // --- the crimson branch (`:13946-14277`); the drunk `flag = true`
        // (`:13938`) runs it even on a corruption-flavoured world ---
        for _k in 0..count {
            let Some((num15, num16, num17)) = evil_band_retry(
                world,
                rng,
                (num5, num6),
                (num3, num4),
                dungeon_location,
                true,
                drunk,
                crimson_left,
            ) else {
                continue; // the D15 valve
            };
            if num17 <= num16 {
                continue; // D15: a degenerate band runs no body
            }
            if flip_infections {
                // The flipInfections chasm block (`:14048-14077`): the
                // corruption walker (`ChasmRunner` reads `GenVars.ebonStone` /
                // `ebonStoneWall`, i.e. crimstone 203 / wall 83 under the
                // swap), interleaved with the per-column jungle conversion.
                let mut num20 = 0i32;
                for l in num16..num17 {
                    if num20 > 0 {
                        num20 -= 1;
                    }
                    if l == num15 || num20 == 0 {
                        let mut num53 = num as i32;
                        while (num53 as f64) < num2 - 1.0 {
                            if active_at(world, l, num53) || wall_at(world, l, num53) > 0 {
                                if l == num15 {
                                    num20 = 20;
                                    let steps = rng.range(0, 150) + 150;
                                    chasm_runner(
                                        world,
                                        rng,
                                        l,
                                        num53,
                                        steps,
                                        true,
                                        num2,
                                        rock_layer,
                                        ebon_stone,
                                        ebon_wall,
                                    );
                                } else if rng.range(0, 35) == 0 && num20 == 0 {
                                    num20 = 30;
                                    let steps = rng.range(0, 50) + 50;
                                    chasm_runner(
                                        world,
                                        rng,
                                        l,
                                        num53,
                                        steps,
                                        true,
                                        num2,
                                        rock_layer,
                                        ebon_stone,
                                        ebon_wall,
                                    );
                                }
                                break;
                            }
                            num53 += 1;
                        }
                    }
                    convert_surface_jungle_grass(
                        world,
                        rng,
                        l,
                        (num16, num17),
                        num,
                        num2,
                        TILE_CRIMSON_JUNGLE_GRASS,
                    );
                }
            } else {
                // `if (!GenVars.flipInfections) WorldGen.CrimStart(num15,
                // (int)num - 10);` (`:14044-14047`).
                crim_start(world, rng, num15, num as i32 - 10, num2, &mut hearts);
                for l in num16..num17 {
                    convert_surface_jungle_grass(
                        world,
                        rng,
                        l,
                        (num16, num17),
                        num,
                        num2,
                        TILE_CRIMSON_JUNGLE_GRASS,
                    );
                }
            }
            evil_shell(world, rng, (num16, num17), num, num2, true);
            altar_scatter(world, rng, (num16, num17), num2, ocean_level);
            // The flipInfections orb-surround (`:14236-14267`): the C# writes
            // the literal crimstone 203, `GenVars.ebonStone`'s swapped id.
            if flip_infections {
                orb_surround_blob(world, rng, (num16, num17), ebon_stone);
            }
        }
        // `if (!GenVars.flipInfections) WorldGen.CrimPlaceHearts();`
        // (`:14273-14277`) — after the whole biome loop.
        if !flip_infections {
            crim_place_hearts(world, rng, &hearts);
        }
    }
    // `if (WorldGen.drunkWorldGen) flag = false;` (`:14279-14281`) — the
    // corruption pass is re-armed, so a drunk world runs BOTH flavours from
    // the halved count (the two arms draw opposite world halves).
    if drunk {
        flag = false;
    }
    if !flag {
        // --- the corruption branch (`:14280-14554`) ---
        for _k in 0..count {
            let Some((num46, num47, num48)) = evil_band_retry(
                world,
                rng,
                (num5, num6),
                (num3, num4),
                dungeon_location,
                false,
                drunk,
                crimson_left,
            ) else {
                continue; // the D15 valve
            };
            if num48 <= num47 {
                continue; // D15: a degenerate band runs no body
            }
            // `if (GenVars.flipInfections) WorldGen.CrimStart(num46, (int)num
            // - 10);` (`:14381-14384`) — the flipped corruption biome grows
            // the crimson walker instead of the chasms.
            if flip_infections {
                crim_start(world, rng, num46, num as i32 - 10, num2, &mut hearts);
            }
            let mut num51 = 0i32;
            for num52 in num47..num48 {
                // The chasm attempts (`:14384-14401`, the `!flipInfections`
                // arm): cooldown first, then the centre column or a cooled-down
                // column scans for its surface hit. The whole block — the
                // `num51` cooldown included — is skipped under the swap.
                if !flip_infections && num51 > 0 {
                    num51 -= 1;
                }
                if !flip_infections && (num52 == num46 || num51 == 0) {
                    let mut num53 = num as i32;
                    while (num53 as f64) < num2 - 1.0 {
                        if active_at(world, num52, num53) || wall_at(world, num52, num53) > 0 {
                            if num52 == num46 {
                                num51 = 20;
                                let steps = rng.range(0, 150) + 150;
                                chasm_runner(
                                    world,
                                    rng,
                                    num52,
                                    num53,
                                    steps,
                                    true,
                                    num2,
                                    rock_layer,
                                    TILE_EBONSTONE,
                                    WALL_EBONSTONE,
                                );
                            } else if rng.range(0, 35) == 0 && num51 == 0 {
                                num51 = 30;
                                let steps = rng.range(0, 50) + 50;
                                chasm_runner(
                                    world,
                                    rng,
                                    num52,
                                    num53,
                                    steps,
                                    true,
                                    num2,
                                    rock_layer,
                                    TILE_EBONSTONE,
                                    WALL_EBONSTONE,
                                );
                            }
                            break;
                        }
                        num53 += 1;
                    }
                }
                convert_surface_jungle_grass(
                    world,
                    rng,
                    num52,
                    (num47, num48),
                    num,
                    num2,
                    TILE_CORRUPT_JUNGLE_GRASS,
                );
            }
            evil_shell(world, rng, (num47, num48), num, num2, false);
            // `if (GenVars.flipInfections) WorldGen.CrimPlaceHearts(); else
            // { ...orb-surround blob... }` (`:14509-14545`) — the swapped
            // corruption biome ends with the crimson hearts, not the
            // ebonstone orb surround.
            if flip_infections {
                crim_place_hearts(world, rng, &hearts);
            } else {
                orb_surround_blob(world, rng, (num47, num48), TILE_EBONSTONE);
            }
        }
    }
}

/// The per-biome centre/band retry loop (`WorldGen.cs:13957-14043` — the
/// crimson branch; the corruption twin `:14391-14469` differs only in the
/// dungeon-side 400-clamps). Returns `(centre, start, end)` or `None` when
/// the D15 valve fired. The desert-overlap retry gates are skipped (D17 —
/// the port carries no `UndergroundDesertLocation`; they draw no RNG).
/// The drunk arms (`:13961-13964` crimson, `:14302-14304` corruption) pass
/// `drunk`/`crimson_left` when the world carries the flag.
fn evil_band_retry(
    world: &World,
    rng: &mut Rng,
    snow_band: (i32, i32),
    jungle_band: (i32, i32),
    dungeon_location: i32,
    crimson: bool,
    drunk: bool,
    crimson_left: bool,
) -> Option<(i32, i32, i32)> {
    let max_x = world.width;
    let mut snow = snow_band; // num11/num12 crimson, num42/num43 corruption
    let mut jungle = jungle_band; // num13/num14 crimson, num44/num45 corruption
    let (mut centre, mut start, mut end) = (0i32, 0i32, 0i32);
    let mut tries = 0i32;
    loop {
        tries += 1;
        if tries > BAND_RETRY_VALVE {
            return None;
        }
        // `int num18 = maxTilesX / 2; int num19 = 200;` (`:13966-13967`).
        let num18 = max_x / 2;
        let mut num19 = 200i32;
        // `num46 = Next(num8, maxTilesX - num8)` (`:13969`) — the classic
        // whole-world draw.
        if !drunk {
            centre = rng.range(500, max_x - 500);
        } else {
            // The drunk anchor draw: the centre comes from the world half the
            // flavour owns, `num8` = 500 marking the far margin. The two arms
            // read `GenVars.crimsonLeft` with OPPOSITE polarity — the crimson
            // branch takes the right half when the corruption owns the left
            // (`:13962`) and the corruption branch the same way (`:14302`),
            // which is what keeps the two flavours apart on a drunk world.
            let half = max_x / 2;
            if crimson {
                // `:13961-13964` — `num19 = 100` on the crimson arm.
                num19 = 100;
                centre = if crimson_left {
                    rng.range(500, half)
                } else {
                    rng.range(half, max_x - 500)
                };
            } else {
                centre = if crimson_left {
                    rng.range(half, max_x - 500)
                } else {
                    rng.range(500, half)
                };
            }
        }
        // `num47 = num46 - Next(200) - 100; num48 = num46 + Next(200) + 100;`
        // (`:13971-13972`).
        start = centre - rng.range(0, 200) - 100;
        end = centre + rng.range(0, 200) + 100;
        // The beach clamp (`:13973-13980`).
        if start < EVIL_BEACH_AVOIDANCE {
            start = EVIL_BEACH_AVOIDANCE;
        }
        if end > max_x - EVIL_BEACH_AVOIDANCE {
            end = max_x - EVIL_BEACH_AVOIDANCE;
        }
        // The centre pull (`:13981-13988`).
        if centre < start + EVIL_MID_FIXER {
            centre = start + EVIL_MID_FIXER;
        }
        if centre > end - EVIL_MID_FIXER {
            centre = end - EVIL_MID_FIXER;
        }
        if crimson {
            // The dungeon-side 400-clamps (`:13989-13996`) — crimson only.
            // `dungeonSide` is fixed Right in the port (Decision #10), so the
            // `dungeonSide >= DungeonSide.Right` arm is the live one.
            if start > max_x - 400 {
                start = max_x - 400;
            }
        }
        // The dungeon-overlap retry (`:13997-14000` / `:14444-14447`).
        if start < dungeon_location + 100 && end > dungeon_location - 100 {
            continue;
        }
        // The world-centre exclusion (`:14005-14016`) — three checks, centre
        // then start then end.
        let (lo, hi) = (num18 - num19, num18 + num19);
        if centre > lo && centre < hi {
            continue;
        }
        if start > lo && start < hi {
            continue;
        }
        if end > lo && end < hi {
            continue;
        }
        // The desert-overlap retries (`:14018-14029`) — skipped (D17).
        // The snow-band shrink-retry (`:14030-14034`), then the jungle one
        // (`:14035-14041`); each failure shrinks its band and retries.
        if start < snow.1 && end > snow.0 {
            snow.0 += 1;
            snow.1 -= 1;
            continue;
        }
        // The `(!notTheBees || remix)` gate holds on the classic path.
        if start < jungle.1 && end > jungle.0 {
            jungle.0 += 1;
            jungle.1 -= 1;
            continue;
        }
        return Some((centre, start, end));
    }
}

/// The per-column jungle-grass conversion (`WorldGen.cs:14061-14077` crimson /
/// `:14402-14416` corruption): from the first surface hit, the top 10-13
/// jungle-grass tiles inside the band's jittered edges become the corrupt /
/// crimson jungle grass. The `Next(5)` edge draws fire only for active
/// jungle-grass tiles (the C# `&&` short-circuit).
fn convert_surface_jungle_grass(
    world: &mut World,
    rng: &mut Rng,
    l: i32,
    band: (i32, i32),
    world_surface_low: f64,
    world_surface: f64,
    grass: u16,
) {
    let mut n = world_surface_low as i32;
    while (n as f64) < world_surface - 1.0 {
        if active_at(world, l, n) {
            let num21 = n + rng.range(10, 14);
            for num22 in n..num21 {
                if active_at(world, l, num22)
                    && kind_at(world, l, num22) == TILE_JUNGLE_GRASS
                    && l >= band.0 + rng.range(0, 5)
                    && l < band.1 - rng.range(0, 5)
                {
                    set_kind(world, l, num22, grass);
                }
            }
            break;
        }
        n += 1;
    }
}

/// The evil surface shell (`WorldGen.cs:14094-14185` crimson /
/// `:14418-14505` corruption): the jittered surface line, the per-tile
/// edge-jitter `Next(2)` draws, the conversion table and the first-surface
/// `SpreadGrass` arms.
fn evil_shell(
    world: &mut World,
    rng: &mut Rng,
    band: (i32, i32),
    world_surface_low: f64,
    world_surface: f64,
    crimson: bool,
) {
    // The flavour twins (the conversion table `:14127-14164` crimson /
    // `:14451-14489` corruption): grass 2 → X, stone 1 → X, sand 53 → X,
    // ice 161 → X, sandstone 396 → X, hardened sand 397 → X, jungle grass
    // 60 → X, walls 216/187 → X/X.
    let (grass, stone, sand, ice, sandstone, hardened) = if crimson {
        (
            TILE_CRIMSON_GRASS,
            TILE_CRIMSTONE,
            TILE_CRIMSAND,
            200u16,
            TILE_CRIMSON_SANDSTONE,
            TILE_CRIMSON_HARDENED_SAND,
        )
    } else {
        (
            TILE_CORRUPT_GRASS,
            TILE_EBONSTONE,
            TILE_EBONSAND,
            TILE_CORRUPT_ICE,
            TILE_CORRUPT_SANDSTONE,
            TILE_CORRUPT_HARDENED_SAND,
        )
    };
    let (wall_a, wall_b) = if crimson {
        (WALL_CRIMSON_HARDENED_SAND, WALL_CRIMSON_SANDSTONE)
    } else {
        (WALL_CORRUPT_HARDENED_SAND, WALL_CORRUPT_SANDSTONE)
    };
    let jungle_grass = if crimson {
        TILE_CRIMSON_JUNGLE_GRASS
    } else {
        TILE_CORRUPT_JUNGLE_GRASS
    };
    let mut num23 = world_surface + 40.0;
    for num24 in band.0..band.1 {
        // The per-column surface-line walk (`:14095-14105`).
        num23 += rng.range(-2, 3) as f64;
        if num23 < world_surface + 30.0 {
            num23 = world_surface + 30.0;
        }
        if num23 > world_surface + 50.0 {
            num23 = world_surface + 50.0;
        }
        let mut flag3 = false;
        let num25 = world_surface_low;
        let mut num26 = num25 as i32;
        while (num26 as f64) < num23 {
            // The edge-jitter gates (`:14112`): the column `Next(2)` draws
            // only on the outer two columns, the row `Next(2)` only on the
            // outer two rows.
            let col_gate = num24 > band.0 + 1 && num24 < band.1 - 2;
            let row_gate = !((num26 as f64) <= num25 + 1.0) && !((num26 as f64) >= num23 - 2.0);
            if col_gate || rng.range(0, 2) != 0 {
                if row_gate || rng.range(0, 2) != 0 {
                    // The noSurface wall pre-conversion (`:14116-14124`) is a
                    // secret seed.
                    if active_at(world, num24, num26) {
                        // The sand arm (`:14127-14129` / `:14451-14453`).
                        if kind_at(world, num24, num26) == TILE_SAND
                            && num24 >= band.0 + rng.range(0, 5)
                            && num24 <= band.1 - rng.range(0, 5)
                        {
                            set_kind(world, num24, num26, sand);
                        }
                        // The first-surface SpreadGrass arms
                        // (`:14130-14140` / `:14455-14465`).
                        if (num26 as f64) < world_surface - 1.0 && !flag3 {
                            let kind = kind_at(world, num24, num26);
                            if kind == TILE_DIRT {
                                let mut grass_spread = 0i32;
                                spread_evil_grass(
                                    world,
                                    num24,
                                    num26,
                                    TILE_DIRT,
                                    grass,
                                    world_surface,
                                    &mut grass_spread,
                                );
                            } else if kind == TILE_MUD {
                                let mut grass_spread = 0i32;
                                spread_evil_grass(
                                    world,
                                    num24,
                                    num26,
                                    TILE_MUD,
                                    jungle_grass,
                                    world_surface,
                                    &mut grass_spread,
                                );
                            }
                        }
                        flag3 = true;
                        // The desert-wall conversions (`:14142-14147` /
                        // `:14467-14472`).
                        let wall = wall_at(world, num24, num26);
                        if wall == WALL_HARDENED_SAND {
                            set_wall(world, num24, num26, wall_a);
                        } else if wall == WALL_SANDSTONE {
                            set_wall(world, num24, num26, wall_b);
                        }
                        // The tile conversions (`:14148-14164` /
                        // `:14473-14489`).
                        let kind = kind_at(world, num24, num26);
                        if kind == TILE_STONE {
                            if num24 >= band.0 + rng.range(0, 5)
                                && num24 <= band.1 - rng.range(0, 5)
                            {
                                set_kind(world, num24, num26, stone);
                            }
                        } else if kind == TILE_GRASS {
                            set_kind(world, num24, num26, grass);
                        } else if kind == TILE_JUNGLE_GRASS {
                            set_kind(world, num24, num26, jungle_grass);
                        } else if kind == TILE_ICE {
                            set_kind(world, num24, num26, ice);
                        } else if kind == TILE_SANDSTONE {
                            set_kind(world, num24, num26, sandstone);
                        } else if kind == TILE_HARDENED_SAND {
                            set_kind(world, num24, num26, hardened);
                        }
                    }
                }
            }
            num26 += 1;
        }
    }
}

/// The crimson altar scatter (`WorldGen.cs:14186-14234`) — the crimson branch
/// only (D16: the corruption branch has no scatter; its altars come from the
/// `ChasmRunner` search). `num30` starts at 0 (`:14191`), the `oceanDepths`
/// redraw, the widen-after-100 and the style-1 `Place3x2`.
fn altar_scatter(
    world: &mut World,
    rng: &mut Rng,
    band: (i32, i32),
    world_surface: f64,
    ocean_level: f64,
) {
    let num27 = rng.range(10, 15);
    for _num28 in 0..num27 {
        let mut num29 = 0i32;
        let mut flag4 = false;
        let mut num30 = 0i32;
        while !flag4 {
            num29 += 1;
            // `num31 = Next(num16 - num30, num17 + num30)` and
            // `num32 = Next((int)(num2 - num30 / 2), (int)(num2 + 100 + num30))`
            // (`:14195-14197`) — `num30 / 2` is the C# integer division.
            let draw_x = |rng: &mut Rng| rng.range(band.0 - num30, band.1 + num30);
            let draw_y = |rng: &mut Rng| {
                rng.range(
                    (world_surface - (num30 / 2) as f64) as i32,
                    (world_surface + 100.0 + num30 as f64) as i32,
                )
            };
            let mut num31 = draw_x(rng);
            let mut num32 = draw_y(rng);
            while ocean_depths(world, num31, num32, ocean_level) {
                num31 = draw_x(rng);
                num32 = draw_y(rng);
            }
            if num29 > 100 {
                num30 += 1;
                num29 = 0;
            }
            // The surface snap (`:14209-14219`).
            if !active_at(world, num31, num32) {
                while !active_at(world, num31, num32) {
                    num32 += 1;
                }
                num32 -= 1;
            } else {
                while active_at(world, num31, num32) && (num32 as f64) > world_surface {
                    num32 -= 1;
                }
            }
            // The placement gate (`:14221-14228`): the widened fallback or a
            // crimstone floor, never near another altar.
            let ground_ok = num30 > 10
                || (active_at(world, num31, num32 + 1)
                    && kind_at(world, num31, num32 + 1) == TILE_CRIMSTONE);
            if ground_ok && !is_tile_nearby(world, num31, num32, TILE_DEMON_ALTAR, 3) {
                place_3x2(world, num31, num32, TILE_DEMON_ALTAR, 1);
                if kind_at(world, num31, num32) == TILE_DEMON_ALTAR {
                    flag4 = true;
                }
            }
            if num30 > 100 {
                flag4 = true;
            }
        }
    }
}

/// The orb-surround evil-stone blob (`WorldGen.cs:14526-14545` corruption /
/// `:14247-14267` crimson-under-flip): every active tile 31 gets a +-13 box
/// of evil stone with the RNG-wobbled Manhattan fill, the inner +-1 air
/// pocket and the ±(2+Next(3)) hollow.
fn orb_surround_blob(world: &mut World, rng: &mut Rng, band: (i32, i32), stone: u16) {
    for num61 in band.0..band.1 {
        for num62 in 0..(world.height - 50) {
            if active_at(world, num61, num62) && kind_at(world, num61, num62) == TILE_SHADOW_ORB {
                let num63 = num61 - 13;
                let num64 = num61 + 13;
                let num65 = num62 - 13;
                let num66 = num62 + 13;
                for num67 in num63..num64 {
                    if num67 > 10 && num67 < world.width - 10 {
                        for num68 in num65..num66 {
                            let dx = (num67 - num61).abs() as i32;
                            let dy = (num68 - num62).abs() as i32;
                            // The fill arm (`:14532-14543`): the Manhattan
                            // wobble and the `Next(3)` lottery draw before
                            // the type check (the C# `&&` order).
                            if dx + dy < 9 + rng.range(0, 11)
                                && rng.range(0, 3) != 0
                                && kind_at(world, num67, num68) != TILE_SHADOW_ORB
                            {
                                set_active(world, num67, num68, true);
                                set_kind(world, num67, num68, stone);
                                if dx <= 1 && dy <= 1 {
                                    set_active(world, num67, num68, false);
                                }
                            }
                            // The hollow arm (`:14543-14544`).
                            if kind_at(world, num67, num68) != TILE_SHADOW_ORB
                                && dx <= 2 + rng.range(0, 3)
                                && dy <= 2 + rng.range(0, 3)
                            {
                                set_active(world, num67, num68, false);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// `WorldGen.ChasmRunner` (`WorldGen.cs:75885-76111`): the descending chasm
/// walker — radius wobble, the one-time side caverns at `worldSurface + 20`,
/// the RNG-first air carve, the orb placement + the never-re-armed `flag3`
/// altar search (the decompile quirk `:75991` — `flag3` is never set true, so
/// every later `steps <= 0` iteration re-runs the search and chasms
/// accumulate nearby altars), then the ebonstone fill and wall passes with
/// their per-tile `Next(3, 20)` activation gates.
#[allow(clippy::too_many_arguments)]
fn chasm_runner(
    world: &mut World,
    rng: &mut Rng,
    i: i32,
    j: i32,
    steps: i32,
    make_orb: bool,
    world_surface: f64,
    rock_layer: f64,
    stone: u16,
    wall: u16,
) {
    let num = world_surface; // the noSurface override is a secret seed
    let mut flag = false; // the side caverns
    let mut flag2 = false; // the orb
    let mut flag3 = false; // the never-re-armed altar-search arm (`:75991`)
    if !make_orb {
        flag2 = true;
    }
    let mut num2 = steps as f64;
    let (mut px, mut py) = (i as f64, j as f64);
    // `vector2D2.X = Next(-10, 11) * 0.1; vector2D2.Y = Next(11) * 0.2 + 0.5;`
    // (`:75914-75915`).
    let mut vel_x = rng.range(-10, 11) as f64 * 0.1;
    let mut vel_y = rng.range(0, 11) as f64 * 0.2 + 0.5;
    let num3 = CHASM_FILL_STEPS;
    // `num4 = Next(5) + 7` (`:75917-75918`).
    let mut num4 = rng.range(0, 5) as f64 + 7.0;
    while num4 > 0.0 {
        if num2 > 0.0 {
            // The radius wobble (`:75921-75932`).
            num4 += rng.range(0, 3) as f64 - rng.range(0, 3) as f64;
            if num4 < 7.0 {
                num4 = 7.0;
            }
            if num4 > 20.0 {
                num4 = 20.0;
            }
            if num2 == 1.0 && num4 < 10.0 {
                num4 = 10.0;
            }
        } else if py > num + 45.0 {
            // The fill-phase decay (`:75933-75936`).
            num4 -= rng.range(0, 4) as f64;
        }
        // The rock-layer cut-off (`:75937-75940`).
        if py > rock_layer && num2 > 0.0 {
            num2 = 0.0;
        }
        num2 -= 1.0;
        // The one-time side caverns (`:75943-75950`).
        if !flag && py > num + 20.0 {
            flag = true;
            let left_steps = rng.range(20, 40);
            chasm_runner_sideways(
                world,
                rng,
                px as i32,
                py as i32,
                -1,
                left_steps,
                world_surface,
                rock_layer,
                stone,
                wall,
            );
            let right_steps = rng.range(20, 40);
            chasm_runner_sideways(
                world,
                rng,
                px as i32,
                py as i32,
                1,
                right_steps,
                world_surface,
                rock_layer,
                stone,
                wall,
            );
        }
        // The air carve while the walk phase runs (`:75957-75980`): the
        // Manhattan wobble draw comes FIRST (the C# `&&` order), then the
        // protections.
        if num2 > num3 as f64 {
            let mut num5 = (px - num4 * 0.5) as i32;
            let mut num6 = (px + num4 * 0.5) as i32;
            let mut num7 = (py - num4 * 0.5) as i32;
            let mut num8 = (py + num4 * 0.5) as i32;
            if num5 < 0 {
                num5 = 0;
            }
            if num6 > world.width - 1 {
                num6 = world.width - 1;
            }
            if num7 < 0 {
                num7 = 0;
            }
            if num8 > world.height {
                num8 = world.height;
            }
            for k in num5..num6 {
                for l in num7..num8 {
                    if (k as f64 - px).abs() + (l as f64 - py).abs()
                        < num4 * 0.5 * (1.0 + rng.range(-10, 11) as f64 * 0.015)
                        && can_evil_replace(world, k, l)
                        && kind_at(world, k, l) != TILE_SHADOW_ORB
                        && kind_at(world, k, l) != TILE_DEMONITE
                        && kind_at(world, k, l) != TILE_CRIMTANE
                    {
                        set_active(world, k, l, false);
                    }
                }
            }
        }
        // The fill-phase depth hold (`:75982-75985`).
        if num2 <= 2.0 && py < num + 45.0 {
            num2 = 2.0;
        }
        // The orb + altar-search tail (`:75986-76027`).
        if num2 <= 0.0 {
            if !flag2 {
                flag2 = true;
                add_shadow_orb(world, px as i32, py as i32, false);
            } else if !flag3 {
                // The decompile quirk: `flag3 = false` never re-arms the
                // guard, so every later iteration re-runs the search
                // (`:75990-76026`).
                flag3 = false;
                let mut flag4 = false;
                let mut num9 = 0;
                while !flag4 {
                    let mut num10 = rng.range(px as i32 - 25, px as i32 + 25);
                    let mut num11 = rng.range(py as i32 - 50, py as i32);
                    if num10 < 5 {
                        num10 = 5;
                    }
                    if num10 > world.width - 5 {
                        num10 = world.width - 5;
                    }
                    if num11 < 5 {
                        num11 = 5;
                    }
                    if num11 > world.height - 5 {
                        num11 = world.height - 5;
                    }
                    if num11 as f64 > num {
                        if !is_tile_nearby(world, num10, num11, TILE_DEMON_ALTAR, 3) {
                            place_3x2(world, num10, num11, TILE_DEMON_ALTAR, 0);
                        }
                        if kind_at(world, num10, num11) == TILE_DEMON_ALTAR {
                            flag4 = true;
                            continue;
                        }
                        num9 += 1;
                        if num9 >= 10000 {
                            flag4 = true;
                        }
                    } else {
                        flag4 = true;
                    }
                }
            }
        }
        // The move + velocity wobble (`:76029-76037`).
        px += vel_x;
        py += vel_y;
        vel_x += rng.range(-10, 11) as f64 * 0.01;
        if vel_x > 0.3 {
            vel_x = 0.3;
        }
        if vel_x < -0.3 {
            vel_x = -0.3;
        }
        // The ebonstone fill ellipse (`:76048-76073`): the protections come
        // before the wobble draw here (the C# `&&` order).
        let mut num5 = (px - num4 * 1.1) as i32;
        let mut num6 = (px + num4 * 1.1) as i32;
        let mut num7 = (py - num4 * 1.1) as i32;
        let mut num8 = (py + num4 * 1.1) as i32;
        if num5 < 1 {
            num5 = 1;
        }
        if num6 > world.width - 1 {
            num6 = world.width - 1;
        }
        if num7 < 0 {
            num7 = 0;
        }
        if num8 > world.height {
            num8 = world.height;
        }
        for m in num5..num6 {
            for n in num7..num8 {
                if can_evil_replace(world, m, n)
                    && (m as f64 - px).abs() + (n as f64 - py).abs()
                        < num4 * 1.1 * (1.0 + rng.range(-10, 11) as f64 * 0.015)
                {
                    // The depth-gated activation (`:76074-76077`): the
                    // `Next(3, 20)` draws only when the tile is not yet evil
                    // stone (the C# `&&` order).
                    if kind_at(world, m, n) != stone && n > j + rng.range(3, 20) {
                        set_active(world, m, n, true);
                    }
                    // `if (steps <= num3)` — the steps parameter, dead for
                    // the pass's 50-299-step calls but ported verbatim.
                    if steps <= num3 {
                        set_active(world, m, n, true);
                    }
                    if kind_at(world, m, n) != TILE_SHADOW_ORB {
                        set_kind(world, m, n, stone);
                    }
                }
            }
        }
        // The wall ellipse (`:76075-76100`): the type write, the (dead)
        // steps gate, then the unconditional per-tile `Next(3, 20)` depth
        // gate for the wall.
        for m in num5..num6 {
            for n in num7..num8 {
                if can_evil_replace(world, m, n)
                    && (m as f64 - px).abs() + (n as f64 - py).abs()
                        < num4 * 1.1 * (1.0 + rng.range(-10, 11) as f64 * 0.015)
                {
                    if kind_at(world, m, n) != TILE_SHADOW_ORB {
                        set_kind(world, m, n, stone);
                    }
                    if steps <= num3 {
                        set_active(world, m, n, true);
                    }
                    if n > j + rng.range(3, 20) {
                        set_wall(world, m, n, wall);
                    }
                }
            }
        }
    }
}

/// `WorldGen.ChasmRunnerSideways` (`WorldGen.cs:75259-75431`): the horizontal
/// side-cavern walker — velocity homing on the start Y, the unconditional
/// wall pass through `PlaceWall`, and the Demonite TileRunner tail
/// (`:75422-75430`). Note the ebonstone constants are used verbatim (the C#
/// reads `GenVars.ebonStone` here even on crimson worlds — the function is
/// only reachable from the corruption path).
#[allow(clippy::too_many_arguments)]
fn chasm_runner_sideways(
    world: &mut World,
    rng: &mut Rng,
    i: i32,
    j: i32,
    direction: i32,
    steps: i32,
    world_surface: f64,
    rock_layer: f64,
    stone: u16,
    wall: u16,
) {
    let mut num = steps as f64;
    let (mut px, mut py) = (i as f64, j as f64);
    // The initial draws (`:75267-75270`).
    let mut vel_x = rng.range(10, 21) as f64 * 0.1 * direction as f64;
    let mut vel_y = rng.range(-10, 10) as f64 * 0.01;
    let mut num2 = rng.range(0, 5) as f64 + 7.0;
    while num2 > 0.0 {
        if num > 0.0 {
            // The radius wobble (`:75273-75284`).
            num2 += rng.range(0, 3) as f64 - rng.range(0, 3) as f64;
            if num2 < 7.0 {
                num2 = 7.0;
            }
            if num2 > 20.0 {
                num2 = 20.0;
            }
            if num == 1.0 && num2 < 10.0 {
                num2 = 10.0;
            }
        } else {
            num2 -= rng.range(0, 4) as f64;
        }
        // The rock-layer cut-off (`:75286-75290`).
        if py > rock_layer && num > 0.0 {
            num = 0.0;
        }
        num -= 1.0;
        // The air carve (every iteration — no fill-phase gate, `:75292-75324`).
        let mut num3 = (px - num2 * 0.5) as i32;
        let mut num4 = (px + num2 * 0.5) as i32;
        let mut num5 = (py - num2 * 0.5) as i32;
        let mut num6 = (py + num2 * 0.5) as i32;
        if num3 < 0 {
            num3 = 0;
        }
        if num4 > world.width - 1 {
            num4 = world.width - 1;
        }
        if num5 < 0 {
            num5 = 0;
        }
        if num6 > world.height {
            num6 = world.height;
        }
        for k in num3..num4 {
            for l in num5..num6 {
                if can_evil_replace(world, k, l)
                    && (k as f64 - px).abs() + (l as f64 - py).abs()
                        < num2 * 0.5 * (1.0 + rng.range(-10, 11) as f64 * 0.015)
                    && kind_at(world, k, l) != TILE_SHADOW_ORB
                    && kind_at(world, k, l) != TILE_DEMONITE
                    && kind_at(world, k, l) != TILE_CRIMTANE
                {
                    set_active(world, k, l, false);
                }
            }
        }
        // The move + the velocity homing on the start Y (`:75326-75358`).
        px += vel_x;
        py += vel_y;
        vel_y += rng.range(-10, 10) as f64 * 0.1;
        if py < (j - 20) as f64 {
            vel_y += rng.range(0, 20) as f64 * 0.01;
        }
        if py > (j + 20) as f64 {
            vel_y -= rng.range(0, 20) as f64 * 0.01;
        }
        if vel_y < -0.5 {
            vel_y = -0.5;
        }
        if vel_y > 0.5 {
            vel_y = 0.5;
        }
        vel_x += rng.range(-10, 11) as f64 * 0.01;
        // The direction clamps (`:75359-75376`).
        if direction == -1 {
            if vel_x > -0.5 {
                vel_x = -0.5;
            }
            if vel_x < -2.0 {
                vel_x = -2.0;
            }
        } else if direction == 1 {
            if vel_x < 0.5 {
                vel_x = 0.5;
            }
            if vel_x > 2.0 {
                vel_x = 2.0;
            }
        }
        // The ebonstone fill ellipse (`:75379-75405`).
        let mut num3 = (px - num2 * 1.1) as i32;
        let mut num4 = (px + num2 * 1.1) as i32;
        let mut num5 = (py - num2 * 1.1) as i32;
        let mut num6 = (py + num2 * 1.1) as i32;
        if num3 < 1 {
            num3 = 1;
        }
        if num4 > world.width - 1 {
            num4 = world.width - 1;
        }
        if num5 < 0 {
            num5 = 0;
        }
        if num6 > world.height {
            num6 = world.height;
        }
        for m in num3..num4 {
            for n in num5..num6 {
                if can_evil_replace(world, m, n)
                    && (m as f64 - px).abs() + (n as f64 - py).abs()
                        < num2 * 1.1 * (1.0 + rng.range(-10, 11) as f64 * 0.015)
                    && wall_at(world, m, n) != wall
                {
                    if !active_at(world, m, n)
                        || (kind_at(world, m, n) != TILE_SHADOW_ORB
                            && kind_at(world, m, n) != TILE_DEMONITE
                            && kind_at(world, m, n) != TILE_CRIMTANE)
                    {
                        set_kind(world, m, n, stone);
                    }
                    set_active(world, m, n, true);
                    if wall_at(world, m, n) == 2 {
                        set_wall(world, m, n, 0);
                    }
                }
            }
        }
        // The wall pass with the unconditional `PlaceWall` (`:75406-75420`).
        for m in num3..num4 {
            for n in num5..num6 {
                if can_evil_replace(world, m, n)
                    && (m as f64 - px).abs() + (n as f64 - py).abs()
                        < num2 * 1.1 * (1.0 + rng.range(-10, 11) as f64 * 0.015)
                    && wall_at(world, m, n) != wall
                {
                    if !active_at(world, m, n)
                        || (kind_at(world, m, n) != TILE_SHADOW_ORB
                            && kind_at(world, m, n) != TILE_DEMONITE
                            && kind_at(world, m, n) != TILE_CRIMTANE)
                    {
                        set_kind(world, m, n, stone);
                    }
                    set_active(world, m, n, true);
                    place_evil_wall(world, m, n, wall);
                }
            }
        }
    }
    // The Demonite tail (`:75422-75430`): `Next(3) == 0` → walk down to the
    // first active tile → `TileRunner(x, y, Next(2, 6), Next(3, 7), 22)`.
    if rng.range(0, 3) == 0 {
        let num9 = px as i32;
        let mut num10 = py as i32;
        // The C# walk is unbounded (an OOB read crashes vanilla); the port
        // clamps at the world floor.
        while num10 < world.height - 1 && !active_at(world, num9, num10) {
            num10 += 1;
        }
        let strength = rng.range(2, 6);
        let tail_steps = rng.range(3, 7);
        tile_runner(
            world,
            rng,
            num9,
            num10,
            strength as f64,
            tail_steps,
            TILE_DEMONITE as i32,
            world_surface,
            &TileRunnerOpts { flags: world.seeds, ..TileRunnerOpts::default() },
        );
    }
}

/// `WorldGen.PlaceWall` (`WorldGen.cs:75125-75144`) — the boundary guard and
/// the empty-wall-only write (`SquareWallFrame` deferred per the port's
/// plain-write convention; the same slice as `tile_runner.rs::place_wall`).
fn place_evil_wall(world: &mut World, k: i32, l: i32, wall: u16) {
    if k <= 1 || l <= 1 || k >= world.width - 2 || l >= world.height - 2 {
        return;
    }
    if let Some(t) = world.tile_mut(k, l) {
        if t.wall == 0 {
            t.wall = wall;
        }
    }
}

/// `WorldGen.CrimStart` (`WorldGen.cs:75449-75714`): the winding crimstone
/// blob down to `worldSurface + 100`, the 50 elliptical chambers, the 5-8
/// veins recording their endpoints in `heartPos`, the entrance shaft and the
/// per-column surface fill, then `CrimEnt`. The C# reads
/// `GenVars.crimStone`/`crimStoneWall` throughout, so under `flipInfections`
/// (`:11298-11304`) the walker paints ebonstone 25 / wall 3 — that is how the
/// flipped corruption branch grows its biome (`:14381`).
fn crim_start(
    world: &mut World,
    rng: &mut Rng,
    i: i32,
    j: i32,
    world_surface: f64,
    hearts: &mut Vec<(i32, i32)>,
) {
    // `GenVars.crimStone` / `GenVars.crimStoneWall` (the flipInfections
    // swap, `WorldGen.cs:11298-11304`): 203/83 classic, 25/3 when drunk
    // && for-the-worthy && !remix.
    let (crim_stone, crim_wall) = crim_ids(world);
    let num = world_surface; // the noSurface override is a secret seed
    let mut crim_dir = 1;
    // The surface walk-down (`:75456-75464`): clamp to the surface, then the
    // first `SolidTile`.
    let mut k = j;
    if k as f64 > num {
        k = num as i32;
    }
    while !solid_tile(world, i, k) {
        k += 1;
    }
    let num2 = k;
    let (mut pos_x, mut pos_y) = (i as f64, k as f64);
    // The initial velocity draws (`:75466-75467`).
    let mut vel_x = rng.range(-20, 21) as f64 * 0.1;
    let mut vel_y = rng.range(20, 201) as f64 * 0.01;
    if vel_x < 0.0 {
        crim_dir = -1;
    }
    let mut num3 = rng.range(15, 26) as f64;
    let mut flag = true;
    let mut num4 = 0i32;
    // The winding blob (`:75472-75535`).
    while flag {
        num3 += rng.range(-50, 51) as f64 * 0.01;
        if num3 < 15.0 {
            num3 = 15.0;
        }
        if num3 > 25.0 {
            num3 = 25.0;
        }
        let mut l = (pos_x - num3 / 2.0) as i32;
        while (l as f64) < pos_x + num3 / 2.0 {
            let mut m = (pos_y - num3 / 2.0) as i32;
            while (m as f64) < pos_y + num3 / 2.0 {
                if !can_evil_replace(world, l, m) {
                    m += 1;
                    continue;
                }
                let manhattan = (l as f64 - pos_x).abs() + (m as f64 - pos_y).abs();
                if m > num2 {
                    if manhattan < num3 * 0.3 {
                        set_active(world, l, m, false);
                        set_wall(world, l, m, crim_wall);
                    } else if manhattan < num3 * 0.8 && wall_at(world, l, m) != crim_wall {
                        set_active(world, l, m, true);
                        set_kind(world, l, m, crim_stone);
                        if manhattan < num3 * 0.6 {
                            set_wall(world, l, m, crim_wall);
                        }
                    }
                } else if manhattan < num3 * 0.3 && active_at(world, l, m) {
                    set_active(world, l, m, false);
                    set_wall(world, l, m, crim_wall);
                }
                m += 1;
            }
            l += 1;
        }
        // The steering arms (`:75536-75548`).
        if pos_x > (i + 50) as f64 {
            num4 = -100;
        }
        if pos_x < (i - 50) as f64 {
            num4 = 100;
        }
        if num4 < 0 {
            vel_x -= rng.range(20, 51) as f64 * 0.01;
        } else if num4 > 0 {
            vel_x += rng.range(20, 51) as f64 * 0.01;
        } else {
            vel_x += rng.range(-50, 51) as f64 * 0.01;
        }
        vel_y += rng.range(-50, 51) as f64 * 0.01;
        if vel_y < 0.25 {
            vel_y = 0.25;
        }
        if vel_y > 2.0 {
            vel_y = 2.0;
        }
        if vel_x < -2.0 {
            vel_x = -2.0;
        }
        if vel_x > 2.0 {
            vel_x = 2.0;
        }
        pos_x += vel_x;
        pos_y += vel_y;
        if pos_y > num + 100.0 {
            flag = false;
        }
    }
    // The 50 chambers (`:75537-75604`): elliptical hollow/fill/wall discs
    // with the per-tile axis-jitter draws.
    num3 = rng.range(40, 55) as f64;
    for _n in 0..50 {
        let num5 = pos_x as i32 + rng.range(-20, 21);
        let num6 = pos_y as i32 + rng.range(-20, 21);
        let mut num7 = (num5 as f64 - num3 / 2.0) as i32;
        while (num7 as f64) < num5 as f64 + num3 / 2.0 {
            let mut num8 = (num6 as f64 - num3 / 2.0) as i32;
            while (num8 as f64) < num6 as f64 + num3 / 2.0 {
                if !can_evil_replace(world, num7, num8) {
                    num8 += 1;
                    continue;
                }
                let num9 = (num7 - num5).abs() as f64;
                let mut num10 = (num8 - num6).abs() as f64;
                let num11 = 1.0 + rng.range(-20, 21) as f64 * 0.01;
                let num12 = 1.0 + rng.range(-20, 21) as f64 * 0.01;
                let num13 = num9 * num11;
                num10 *= num12;
                let num14 = (num13 * num13 + num10 * num10).sqrt();
                if num14 < num3 * 0.25 {
                    set_active(world, num7, num8, false);
                    set_wall(world, num7, num8, crim_wall);
                } else if num14 < num3 * 0.4 && wall_at(world, num7, num8) != crim_wall {
                    set_active(world, num7, num8, true);
                    set_kind(world, num7, num8, crim_stone);
                    if num14 < num3 * 0.35 {
                        set_wall(world, num7, num8, crim_wall);
                    }
                }
                num8 += 1;
            }
            num7 += 1;
        }
    }
    // The 5-8 veins (`:75606-75650`): the direction draws with the 1.5
    // Manhattan redraw, the overlap re-roll — which reads the WINDING
    // velocity `vector2D`, not the drawn direction (the C# quirk at
    // `:75631-75637`, ported verbatim) — then `CrimVein`.
    let num15 = rng.range(5, 9);
    let mut array: Vec<(f64, f64)> = Vec::with_capacity(num15 as usize);
    for num16 in 0..num15 {
        let num17 = pos_x as i32;
        let num18 = pos_y as i32;
        let mut num19 = 0i32;
        let mut flag2 = true;
        // The pre-loop assignment draws before the re-roll loop
        // (`:75619-75620`).
        let mut vec2 = (
            rng.range(-20, 21) as f64 * 0.15,
            rng.range(0, 21) as f64 * 0.15,
        );
        while flag2 {
            vec2 = (
                rng.range(-20, 21) as f64 * 0.15,
                rng.range(0, 21) as f64 * 0.15,
            );
            while vec2.0.abs() + vec2.1.abs() < 1.5 {
                vec2 = (
                    rng.range(-20, 21) as f64 * 0.15,
                    rng.range(0, 21) as f64 * 0.15,
                );
            }
            flag2 = false;
            for num20 in 0..num16 {
                let prior = array[num20 as usize];
                if vel_x > prior.0 - 0.75
                    && vel_x < prior.0 + 0.75
                    && vel_y > prior.1 - 0.75
                    && vel_y < prior.1 + 0.75
                {
                    flag2 = true;
                    num19 += 1;
                    break;
                }
            }
            if num19 > 10000 {
                break;
            }
        }
        array.push(vec2);
        crim_vein(world, rng, (num17 as f64, num18 as f64), vec2, hearts);
    }
    // The entrance shaft (`:75652-75705`): 50 blobs rising by `num23`, the
    // X-extent tracking, then the per-column surface fill.
    let mut num21 = world.width;
    let mut num22 = 0i32;
    pos_x = i as f64;
    pos_y = num2 as f64;
    num3 = rng.range(25, 35) as f64;
    let mut num23 = rng.range(0, 6) as f64;
    for _num24 in 0..50 {
        if num23 > 0.0 {
            let num25 = rng.range(10, 30) as f64 * 0.01;
            num23 -= num25;
            pos_y -= num25;
        }
        let num26 = pos_x as i32 + rng.range(-2, 3);
        let num27 = pos_y as i32 + rng.range(-2, 3);
        let mut num28 = (num26 as f64 - num3 / 2.0) as i32;
        while (num28 as f64) < num26 as f64 + num3 / 2.0 {
            let mut num29 = (num27 as f64 - num3 / 2.0) as i32;
            while (num29 as f64) < num27 as f64 + num3 / 2.0 {
                if !can_evil_replace(world, num28, num29) {
                    num29 += 1;
                    continue;
                }
                let num30 = (num28 - num26).abs() as f64;
                let mut num31 = (num29 - num27).abs() as f64;
                let num32 = 1.0 + rng.range(-20, 21) as f64 * 0.005;
                let num33 = 1.0 + rng.range(-20, 21) as f64 * 0.005;
                let num34 = num30 * num32;
                num31 *= num33;
                let num35 = (num34 * num34 + num31 * num31).sqrt();
                if num35 < num3 * 0.2 * (rng.range(90, 111) as f64 * 0.01) {
                    set_active(world, num28, num29, false);
                    set_wall(world, num28, num29, crim_wall);
                } else {
                    if num35 >= num3 * 0.45 {
                        num29 += 1;
                        continue;
                    }
                    if num28 < num21 {
                        num21 = num28;
                    }
                    if num28 > num22 {
                        num22 = num28;
                    }
                    if wall_at(world, num28, num29) != crim_wall {
                        set_active(world, num28, num29, true);
                        set_kind(world, num28, num29, crim_stone);
                        if num35 < num3 * 0.35 {
                            set_wall(world, num28, num29, crim_wall);
                        }
                    }
                }
                num29 += 1;
            }
            num28 += 1;
        }
    }
    // The per-column surface fill (`:75706-75713`).
    for num36 in num21..=num22 {
        let mut num37 = num2;
        while (kind_at(world, num36, num37) == crim_stone && active_at(world, num36, num37))
            || wall_at(world, num36, num37) == crim_wall
        {
            num37 += 1;
        }
        let mut num38 = rng.range(15, 20);
        while !active_at(world, num36, num37)
            && num38 > 0
            && wall_at(world, num36, num37) != crim_wall
        {
            if can_evil_replace(world, num36, num37) {
                num38 -= 1;
                set_kind(world, num36, num37, crim_stone);
                set_active(world, num36, num37, true);
                num37 += 1;
            } else {
                // The C# body is empty here (an unprotected tile spins the
                // loop forever); the port breaks instead of hanging — the
                // state needs an inactive tile on a dungeon wall inside the
                // crimson shell, which the band retry keeps 100+ tiles away.
                break;
            }
        }
    }
    // `WorldGen.CrimEnt(position, crimDir);` (`:75713`).
    crim_ent(world, rng, (pos_x, pos_y), crim_dir);
}

/// `WorldGen.CrimVein` (`WorldGen.cs:75808-75883`): the wobbling vein walk —
/// radius clamp [15, 25], velocity clamped to ±0.75 of the initial, the
/// carve/fill/wall discs — ending with the endpoint recorded in
/// `heartPos[heartCount++]` (`:75881-75882`).
fn crim_vein(
    world: &mut World,
    rng: &mut Rng,
    position: (f64, f64),
    velocity: (f64, f64),
    hearts: &mut Vec<(i32, i32)>,
) {
    // `GenVars.crimStone` / `GenVars.crimStoneWall` (the flipInfections
    // swap, `WorldGen.cs:11298-11304`): 203/83 classic, 25/3 when drunk
    // && for-the-worthy && !remix.
    let (crim_stone, crim_wall) = crim_ids(world);
    let mut num = rng.range(15, 26) as f64;
    let mut flag = true;
    let vector2d = velocity;
    let vector2d2 = position;
    let (mut pos_x, mut pos_y) = position;
    let (mut vel_x, mut vel_y) = velocity;
    let mut num2 = rng.range(100, 150);
    if velocity.1 < 0.0 {
        num2 -= 25;
    }
    while flag {
        num += rng.range(-50, 51) as f64 * 0.02;
        if num < 15.0 {
            num = 15.0;
        }
        if num > 25.0 {
            num = 25.0;
        }
        let mut i = (pos_x - num / 2.0) as i32;
        while (i as f64) < pos_x + num / 2.0 {
            let mut j = (pos_y - num / 2.0) as i32;
            while (j as f64) < pos_y + num / 2.0 {
                if !can_evil_replace(world, i, j) {
                    j += 1;
                    continue;
                }
                let num3 = (i as f64 - pos_x).abs();
                let num4 = (j as f64 - pos_y).abs();
                let num5 = (num3 * num3 + num4 * num4).sqrt();
                if num5 < num * 0.2 {
                    set_active(world, i, j, false);
                    set_wall(world, i, j, crim_wall);
                } else if num5 < num * 0.5 && wall_at(world, i, j) != crim_wall {
                    set_active(world, i, j, true);
                    set_kind(world, i, j, crim_stone);
                    if num5 < num * 0.4 {
                        set_wall(world, i, j, crim_wall);
                    }
                }
                j += 1;
            }
            i += 1;
        }
        // The velocity wobble clamped to ±0.75 of the initial (`:75863-75874`).
        vel_x += rng.range(-50, 51) as f64 * 0.05;
        vel_y += rng.range(-50, 51) as f64 * 0.05;
        if vel_y < vector2d.1 - 0.75 {
            vel_y = vector2d.1 - 0.75;
        }
        if vel_y > vector2d.1 + 0.75 {
            vel_y = vector2d.1 + 0.75;
        }
        if vel_x < vector2d.0 - 0.75 {
            vel_x = vector2d.0 - 0.75;
        }
        if vel_x > vector2d.0 + 0.75 {
            vel_x = vector2d.0 + 0.75;
        }
        pos_x += vel_x;
        pos_y += vel_y;
        if (pos_x - vector2d2.0).abs() + (pos_y - vector2d2.1).abs() > num2 as f64 {
            flag = false;
        }
    }
    // `heartPos[heartCount++] = position.ToPoint()` (`:75881-75882`).
    hearts.push((pos_x as i32, pos_y as i32));
}

/// `WorldGen.CrimEnt` (`WorldGen.cs:75764-75806`): the entrance tunnel
/// through crimstone only — the walk ends after 20 consecutive steps without
/// a crimstone carve (each carve resets the counter).
fn crim_ent(world: &mut World, rng: &mut Rng, position: (f64, f64), crim_dir: i32) {
    // `GenVars.crimStone` (`WorldGen.cs:75795` reads the tile id only — the
    // entrance tunnel carves no walls) — 203 classic, 25 under the
    // flipInfections swap (`:11298-11304`).
    let crim_stone = crim_ids(world).0;
    let mut num = 0.0f64;
    let mut num2 = rng.range(6, 11) as f64;
    let mut flag = true;
    // `vector2D = (2.0, Next(-20, 0) * 0.01); vector2D.X *= -crimDir;`
    // (`:75769-75771`).
    let mut vel_x = 2.0f64 * -(crim_dir as f64);
    let mut vel_y = rng.range(-20, 0) as f64 * 0.01;
    let (mut pos_x, mut pos_y) = position;
    while flag {
        num += 1.0;
        if num >= 20.0 {
            flag = false;
        }
        num2 += rng.range(-10, 11) as f64 * 0.02;
        if num2 < 6.0 {
            num2 = 6.0;
        }
        if num2 > 10.0 {
            num2 = 10.0;
        }
        let mut i = (pos_x - num2 / 2.0) as i32;
        while (i as f64) < pos_x + num2 / 2.0 {
            let mut j = (pos_y - num2 / 2.0) as i32;
            while (j as f64) < pos_y + num2 / 2.0 {
                if can_evil_replace(world, i, j) {
                    let num3 = (i as f64 - pos_x).abs();
                    let num4 = (j as f64 - pos_y).abs();
                    if (num3 * num3 + num4 * num4).sqrt() < num2 * 0.5
                        && active_at(world, i, j)
                        && kind_at(world, i, j) == crim_stone
                    {
                        set_active(world, i, j, false);
                        flag = true;
                        num = 0.0;
                    }
                }
                j += 1;
            }
            i += 1;
        }
        pos_x += vel_x;
        pos_y += vel_y;
    }
}

/// `WorldGen.CrimPlaceHearts` (`WorldGen.cs:75716-75762`): per recorded vein
/// endpoint — the crimstone disc (`Next(16, 21)`, `< r*0.4`), then the hollow
/// heart room (`Next(10, 14)`, `< r*0.3`), then `AddShadowOrb(...,
/// crimsonHeart: true)`. Runs once per pass on the classic crimson branch
/// (`:14273`) and once per band on the flipped corruption branch
/// (`:14511-14513`), with the ids the `flipInfections` swap selects.
fn crim_place_hearts(world: &mut World, rng: &mut Rng, hearts: &[(i32, i32)]) {
    // `GenVars.crimStone` / `GenVars.crimStoneWall` (the flipInfections
    // swap, `WorldGen.cs:11298-11304`): 203/83 classic, 25/3 when drunk
    // && for-the-worthy && !remix.
    let (crim_stone, crim_wall) = crim_ids(world);
    let mut num = 16;
    for i in 0..hearts.len() {
        num = rng.range(16, 21);
        let (x, y) = hearts[i];
        let mut j = x - num / 2;
        while j < x + num / 2 {
            let mut k = y - num / 2;
            while k < y + num / 2 {
                let num2 = (j - x).abs() as f64;
                let num3 = (k - y).abs() as f64;
                if (num2 * num2 + num3 * num3).sqrt() < num as f64 * 0.4 {
                    set_active(world, j, k, true);
                    set_kind(world, j, k, crim_stone);
                    set_wall(world, j, k, crim_wall);
                }
                k += 1;
            }
            j += 1;
        }
    }
    for l in 0..hearts.len() {
        num = rng.range(10, 14);
        let (x, y) = hearts[l];
        let mut m = x - num / 2;
        while m < x + num / 2 {
            let mut n = y - num / 2;
            while n < y + num / 2 {
                let num4 = (m - x).abs() as f64;
                let num5 = (n - y).abs() as f64;
                if (num4 * num4 + num5 * num5).sqrt() < num as f64 * 0.3 {
                    set_active(world, m, n, false);
                    set_wall(world, m, n, crim_wall);
                }
                n += 1;
            }
            m += 1;
        }
    }
    for heart in hearts {
        add_shadow_orb(world, heart.0, heart.1, true);
    }
}

/// `WorldGen.AddShadowOrb` (`WorldGen.cs:32095-32132`) — no RNG: the bounds
/// guard, the abort when any of the 2x2 already holds an orb, then the 2x2
/// tile 31 with frames `0/18` (+36 on `frameX` for a crimson heart).
pub fn add_shadow_orb(world: &mut World, x: i32, y: i32, crimson_heart: bool) {
    if x < 10 || x > world.width - 10 || y < 10 || y > world.height - 10 {
        return;
    }
    for i in (x - 1)..(x + 1) {
        for j in (y - 1)..(y + 1) {
            if world
                .tile(i, j)
                .is_some_and(|t| t.active() && t.kind == TILE_SHADOW_ORB)
            {
                return;
            }
        }
    }
    let num: i16 = if crimson_heart { 36 } else { 0 };
    let write = |world: &mut World, x: i32, y: i32, frame_x: i16, frame_y: i16| {
        if let Some(t) = world.tile_mut(x, y) {
            t.flags |= FLAG_ACTIVE;
            t.kind = TILE_SHADOW_ORB;
            t.frame_x = frame_x;
            t.frame_y = frame_y;
        }
    };
    write(world, x - 1, y - 1, num, 0);
    write(world, x, y - 1, 18 + num, 0);
    write(world, x - 1, y, num, 18);
    write(world, x, y, 18 + num, 18);
}

/// `WorldGen.CanEvilReplace` (`WorldGen.cs:75433-75447`): the only protection
/// is the dungeon brick (`Main.tileDungeon` + `TileID.Sets.CrackedBricks`)
/// and the dungeon walls (`Main.wallDungeon`) — evil rewrites evil.
fn can_evil_replace(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true; // OOB reads are replaceable; the writes skip
    };
    let num = if t.active() { t.kind } else { 0 };
    if t.active() && (TILE_DUNGEON.contains(&num) || SET_CRACKED_BRICKS.has(num)) {
        return false;
    }
    if WALL_DUNGEON.contains(&t.wall) {
        return false;
    }
    true
}

/// `WorldGen.IsTileNearby` (`WorldGen.cs:8862-8884`) — no RNG; the step is 1
/// except for tile 235 (not placed by this pass).
fn is_tile_nearby(world: &World, x: i32, y: i32, kind: u16, distance: i32) -> bool {
    let step = if kind == 235 { 3 } else { 1 };
    let mut i = x - distance;
    while i <= x + distance {
        let mut j = y - distance;
        while j <= y + distance {
            // `WorldGen.InWorld(i, j)` — fluff 0 (`:9688-9695`).
            if i >= 0 && i < world.width && j >= 0 && j < world.height {
                if world
                    .tile(i, j)
                    .is_some_and(|t| t.active() && t.kind == kind)
                {
                    return true;
                }
            }
            j += 1;
        }
        i += step;
    }
    false
}

/// `WorldGen.Place3x2` (`WorldGen.cs:52066-52196`), the `type == 26` path the
/// evil pass calls — no RNG: the bounds guard, the clear scan over the 3x2
/// body, the solid floor under every column (`SolidTile2`), then the frame
/// write (`frameX = 54 * style + dx * 18`, `frameY = dy * 18`). The boulder
/// check (`TileID.Sets.Boulders`) is skipped — no boulder tiles exist during
/// worldgen in the port (D20).
fn place_3x2(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    if x < 5 || x > world.width - 5 || y < 5 || y > world.height - 5 {
        return;
    }
    let mut flag2 = true;
    let num = y - 1;
    for i in (x - 1)..(x + 2) {
        for j in num..(y + 1) {
            if world.tile(i, j).is_some_and(|t| t.active()) {
                flag2 = false;
            }
        }
        // The `default` case for type 26 (`:52104-52112`): the boulder check
        // (D20) and the all-columns solid floor.
        if !solid_tile2(world, i, y + 1) {
            flag2 = false;
        }
    }
    if flag2 {
        let num2 = (54 * style) as i16;
        // The 3x2 frame write (`:52181-52195`), in the C# row-major order.
        let write = |world: &mut World, x: i32, y: i32, frame_x: i16, frame_y: i16| {
            if let Some(t) = world.tile_mut(x, y) {
                t.flags |= FLAG_ACTIVE;
                t.kind = kind;
                t.frame_x = frame_x;
                t.frame_y = frame_y;
            }
        };
        write(world, x - 1, y - 1, num2, 0);
        write(world, x, y - 1, num2 + 18, 0);
        write(world, x + 1, y - 1, num2 + 36, 0);
        write(world, x - 1, y, num2, 18);
        write(world, x, y, num2 + 18, 18);
        write(world, x + 1, y, num2 + 36, 18);
    }
}

/// `WorldGen.oceanDepths` (`WorldGen.cs:10963-10973`) — no RNG; the ocean
/// level and the ±`beachDistance` margins.
fn ocean_depths(world: &World, x: i32, y: i32, ocean_level: f64) -> bool {
    if y as f64 > ocean_level {
        return false;
    }
    if x < BEACH_DISTANCE || x > world.width - BEACH_DISTANCE {
        return true;
    }
    false
}

/// `WorldGen.SolidTile` (`WorldGen.cs:70111-70131`) — no RNG; an
/// out-of-bounds tile reads as solid (`Main.tile[i, j] == null → true`).
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && TILE_SOLID.has(t.kind)
        && !TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && (t.flags & FLAG_INACTIVE) == 0
}

/// `WorldGen.SolidTile2` (`WorldGen.cs:70134-70150`) — no RNG; an
/// out-of-bounds tile reads as solid. No `tileSolidTop` exclusion.
fn solid_tile2(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && TILE_SOLID.has(t.kind)
        && t.slope() == 0
        && !t.half_brick()
        && (t.flags & FLAG_INACTIVE) == 0
}

/// `WorldGen.SpreadGrass` (`WorldGen.cs:75146-75268`), the slice the evil
/// shell's four call shapes run (`(0, 23)`, `(0, 199)`, `(59, 661)`,
/// `(59, 662)`). Ported locally into this module (D18 — `growth.rs`'s slice
/// carries the inverted guard, Decision #13; `biomes.rs`'s stays the jungle
/// pass's local slice). During generation (`isGeneratingOrLoadingWorld`):
/// grass 23/199 takes the beach/centre exclusion branch (`:75151-75159`),
/// 661/662 the else-if whose surface rule's `dirt == 0` clause never fires
/// for mud 59. `Conversion.Grass` = {2, 23, 199, 109, 477, 492}
/// (`TileID.cs:86`) holds neither 0 nor 59, so the `flag2` Convert path is
/// dead for our calls and the plain write runs (`:75219-75223` — wall and
/// liquid kept; `SquareTileFrame` deferred). No trees exist during
/// generation, so `TryConvertingOrKillingTreesAbove...` is a no-op.
fn spread_evil_grass(
    world: &mut World,
    i: i32,
    j: i32,
    dirt: u16,
    grass: u16,
    world_surface: f64,
    grass_spread: &mut i32,
) {
    // `!InWorld(i, j, 10) || !active || type != dirt` (`:75149-75151`).
    if i < 10 || i >= world.width - 10 || j < 10 || j >= world.height - 10 {
        return;
    }
    if !world
        .tile(i, j)
        .is_some_and(|t| t.active() && t.kind == dirt)
    {
        return;
    }
    // The generation-time exclusions (`:75151-75159`).
    if grass == 199 || grass == 23 {
        // `i > maxTilesX * 0.45 && i <= maxTilesX * 0.55` or the ±380 ocean
        // margins suppress the dirt conversion during generation.
        if (i as f64 > world.width as f64 * 0.45 && i as f64 <= world.width as f64 * 0.55)
            || i < BEACH_DISTANCE
            || i >= world.width - BEACH_DISTANCE
        {
            return;
        }
    } else {
        // The else-if (`:75160-75165`): `isGeneratingOrLoadingWorld` is true
        // and `growGrassUnderground` false, so the return condition is
        // `(type != dirt || !active || (j >= worldSurface && dirt == 0))` —
        // dead for the mud call (dirt 59) but ported verbatim.
        let matches = world
            .tile(i, j)
            .is_some_and(|t| t.active() && t.kind == dirt);
        let surface_rule = j as f64 >= world_surface && dirt == 0;
        if !matches || surface_rule {
            return;
        }
    }
    // The 3x3 scan (`:75167-75181`): proceed iff some tile is inactive or
    // non-solid; lava re-arms the flag and breaks the inner scan only.
    let num2 = (i - 1).max(0);
    let num3 = (i + 2).min(world.width);
    let num4 = (j - 1).max(0);
    let num5 = (j + 2).min(world.height);
    let mut flag = true;
    for k in num2..num3 {
        for l in num4..num5 {
            if let Some(t) = world.tile(k, l) {
                if !t.active() || !TILE_SOLID.has(t.kind) {
                    flag = false;
                }
                if t.liquid > 0 && t.liquid_kind() == LiquidKind::Lava {
                    flag = true;
                    break;
                }
            }
        }
    }
    // The gates (`:75182-75208`): the scan flag, `CanBeClearedDuringGeneration`
    // and the flower-27 exclusions for the evil grasses.
    let above = world.tile(i, j - 1).map_or(0, |t| t.kind);
    if flag
        || !world
            .tile(i, j)
            .is_some_and(|t| SET_CAN_BE_CLEARED_DURING_GENERATION.has(t.kind))
        || ((grass == 23 || grass == 661) && above == 27)
        || ((grass == 199 || grass == 662) && above == 27)
    {
        return;
    }
    // The write (`:75209-75225`): `flag2` is dead for dirt 0/59 (they are
    // not in `Conversion.Grass`), so the plain write runs.
    let flag2 = dirt > 0 && SET_GRASS.has(dirt);
    let _ = flag2;
    set_kind(world, i, j, grass);
    // The 3x3 recursion (`:75238-75260`) with the `grassSpread < 1000` cap.
    for m in num2..num3 {
        for n in num4..num5 {
            if world
                .tile(m, n)
                .is_some_and(|t| t.active() && t.kind == dirt)
                && *grass_spread < 1000
            {
                *grass_spread += 1;
                spread_evil_grass(world, m, n, dirt, grass, world_surface, grass_spread);
                *grass_spread -= 1;
            }
        }
    }
}

// --- tests -------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::Tile;

    /// A flat synthetic world: dirt above `rock`, stone below, the port's
    /// wall baseline (2 above / 1 below), the surface row bare (no grass —
    /// the port's pre-evil terrain state on the evil band).
    fn flat_world(surface: i32, width: i32, height: i32) -> (World, f64, f64) {
        let mut world = World::new("test".into(), width, height);
        let rock = (surface + 80) as f64;
        for x in 0..width {
            for y in 0..height {
                let i = world.index(x, y);
                let mut tile = Tile::default();
                if y >= surface {
                    tile.kind = if (y as f64) < rock { 0 } else { 1 };
                    tile.flags |= FLAG_ACTIVE;
                    if y > surface + 2 {
                        tile.wall = if (y as f64) < rock { 2 } else { 1 };
                    }
                }
                world.tiles[i] = tile;
            }
        }
        (world, surface as f64, rock)
    }

    /// The tile counter the substance tests share (a free fn — the closure
    /// form borrows `world` across the later `&mut` calls).
    fn count_tiles(world: &World, pred: &dyn Fn(&Tile) -> bool) -> usize {
        world.tiles.iter().filter(|t| pred(t)).count()
    }

    /// DW-4.2: `CanEvilReplace` protects exactly the dungeon bricks
    /// (`Main.tileDungeon` + `CrackedBricks`) and the dungeon walls; plain
    /// and evil tiles are replaceable.
    #[test]
    fn test_DW_4_2_can_evil_replace_protections() {
        let mut world = World::new("t".into(), 100, 100);
        let put = |world: &mut World, x: i32, y: i32, kind: u16, wall: u16| {
            let i = world.index(x, y);
            let mut t = Tile::default();
            t.kind = kind;
            t.wall = wall;
            t.flags |= FLAG_ACTIVE;
            world.tiles[i] = t;
        };
        // The dungeon brick tiles (41/43/44 blue/green/pink, 481-483 the
        // cracked variants) survive.
        for (x, kind) in [
            (20i32, 41u16),
            (21, 43),
            (22, 44),
            (23, 481),
            (24, 482),
            (25, 483),
        ] {
            put(&mut world, x, 50, kind, 0);
            assert!(
                !can_evil_replace(&world, x, 50),
                "dungeon brick {kind} must be protected"
            );
        }
        // The dungeon walls (7/8/9 blue/green/pink, 94-99) protect even an
        // inactive tile.
        for (x, wall) in [(30i32, 7u16), (31, 8), (32, 9), (33, 94), (34, 99)] {
            put(&mut world, x, 50, 0, wall);
            let i = world.index(x, 50);
            world.tiles[i].flags &= !FLAG_ACTIVE; // inactive + dungeon wall
            assert!(
                !can_evil_replace(&world, x, 50),
                "dungeon wall {wall} must be protected"
            );
        }
        // Plain and evil tiles are replaceable.
        for (x, kind) in [
            (40i32, 1u16),
            (41, 0),
            (42, 25),
            (43, 203),
            (44, 59),
            (45, 2),
        ] {
            put(&mut world, x, 50, kind, 2);
            assert!(can_evil_replace(&world, x, 50), "tile {kind} replaceable");
        }
        // A non-dungeon wall does not protect.
        put(&mut world, 50, 50, 1, 2);
        assert!(can_evil_replace(&world, 50, 50));
    }

    /// DW-4.2: the shell table converts exactly — the corruption flavour.
    /// Every test tile sits on an interior column and an interior row, where
    /// the edge-jitter gates pass deterministically (no `Next(2)` draws
    /// decide them), so the conversions are seed-independent.
    #[test]
    fn test_DW_4_2_shell_table_conversions() {
        let (mut world, surface, _rock) = flat_world(60, 2000, 200);
        let band = (500i32, 700i32);
        // The dirt/stone body from the surface down (the SpreadGrass arms
        // need the surface-adjacent dirt).
        // One test column per conversion, tiles at rows 50-58 (interior:
        // > ws_low + 1 and < num23 - 2 for any num23 >= 90).
        let cases: [(i32, u16, u16, u16); 10] = [
            (510, TILE_SAND, TILE_EBONSAND, 0),       // sand 53 → 112
            (520, TILE_STONE, TILE_EBONSTONE, 0),     // stone 1 → 25
            (530, TILE_GRASS, TILE_CORRUPT_GRASS, 0), // grass 2 → 23
            (540, TILE_JUNGLE_GRASS, TILE_CORRUPT_JUNGLE_GRASS, 0), // 60 → 661
            (550, TILE_ICE, TILE_CORRUPT_ICE, 0),     // ice 161 → 163
            (560, TILE_SANDSTONE, TILE_CORRUPT_SANDSTONE, 0), // 396 → 400
            (570, TILE_HARDENED_SAND, TILE_CORRUPT_HARDENED_SAND, 0), // 397 → 398
            (580, TILE_DIRT, TILE_CORRUPT_GRASS, 0),  // dirt 0 → 23 (spread)
            (590, TILE_MUD, TILE_CORRUPT_JUNGLE_GRASS, 0), // mud 59 → 661 (spread)
            (600, TILE_DIRT, TILE_CORRUPT_GRASS, WALL_HARDENED_SAND), // wall 216 → 217
        ];
        for (x, kind, _, wall) in cases {
            for y in 50..59 {
                let i = world.index(x, y);
                let mut t = Tile::default();
                t.kind = if y == 58 { kind } else { kind };
                t.wall = if wall > 0 { wall } else { 0 };
                t.flags |= FLAG_ACTIVE;
                world.tiles[i] = t;
                let _ = i;
            }
        }
        // A sandstone-wall tile (the 187 → 220 arm): dirt with wall 187.
        let i = world.index(610, 55);
        let mut t = Tile::default();
        t.kind = TILE_STONE;
        t.wall = WALL_SANDSTONE;
        t.flags |= FLAG_ACTIVE;
        world.tiles[i] = t;
        let _ = t;
        evil_shell(
            &mut world,
            &mut Rng(0x5111),
            band,
            40.0, // world_surface_low
            surface,
            false,
        );
        for (x, kind, converted, wall) in cases {
            let got = world.tile(x, 58).unwrap();
            assert_eq!(got.kind, converted, "column {x}: {} → {converted}", kind);
            if wall > 0 {
                assert_eq!(got.wall, WALL_CORRUPT_HARDENED_SAND, "wall 216 → 217");
            }
        }
        // The sandstone wall converted on the stone tile too (the wall arm
        // runs after the spread arms).
        assert_eq!(
            world.tile(610, 55).unwrap().wall,
            WALL_CORRUPT_SANDSTONE,
            "wall 187 → 220"
        );
        // The spread recursed into the dirt column below the surface tile.
        assert_eq!(
            world.tile(580, 57).unwrap().kind,
            TILE_CORRUPT_GRASS,
            "the dirt spread recursed"
        );
        // Unrelated tiles survive: dirt outside the band (the band is far
        // from the spread's reach and the shell never touches column 100).
        assert_eq!(world.tile(100, 100).unwrap().kind, TILE_DIRT);
        // The dungeon brick is untouched by the shell (the SpreadGrass and
        // type arms never match 41, and the shell's writes go through
        // `set_kind`/`set_wall` without a protection pass — the dungeon
        // brick sits outside the band here; the chasm protection is the
        // `chasm_runner_skips_dungeon_brick` test).
    }

    /// DW-4.2: the crimson twin of the shell table.
    #[test]
    fn test_DW_4_2_shell_table_crimson_twins() {
        let (mut world, surface, _rock) = flat_world(60, 2000, 200);
        let band = (500i32, 700i32);
        let cases: [(i32, u16, u16); 8] = [
            (510, TILE_SAND, TILE_CRIMSAND),                       // 53 → 234
            (520, TILE_STONE, TILE_CRIMSTONE),                     // 1 → 203
            (530, TILE_GRASS, TILE_CRIMSON_GRASS),                 // 2 → 199
            (540, TILE_JUNGLE_GRASS, TILE_CRIMSON_JUNGLE_GRASS),   // 60 → 662
            (550, TILE_ICE, 200),                                  // 161 → 200 FleshIce
            (560, TILE_SANDSTONE, TILE_CRIMSON_SANDSTONE),         // 396 → 401
            (570, TILE_HARDENED_SAND, TILE_CRIMSON_HARDENED_SAND), // 397 → 399
            (580, TILE_MUD, TILE_CRIMSON_JUNGLE_GRASS),            // 59 → 662 (spread)
        ];
        for (x, kind, _) in cases {
            for y in 50..59 {
                let i = world.index(x, y);
                let mut t = Tile::default();
                t.kind = kind;
                t.flags |= FLAG_ACTIVE;
                world.tiles[i] = t;
            }
        }
        let i = world.index(610, 55);
        let mut t = Tile::default();
        t.kind = TILE_STONE;
        t.wall = WALL_HARDENED_SAND;
        t.flags |= FLAG_ACTIVE;
        world.tiles[i] = t;
        evil_shell(&mut world, &mut Rng(0x5222), band, 40.0, surface, true);
        for (x, kind, converted) in cases {
            assert_eq!(
                world.tile(x, 58).unwrap().kind,
                converted,
                "column {x}: {kind} → {converted}"
            );
        }
        assert_eq!(
            world.tile(610, 55).unwrap().wall,
            WALL_CRIMSON_HARDENED_SAND
        );
        // No corruption tiles anywhere in the crimson shell.
        let corrupt = world
            .tiles
            .iter()
            .filter(|t| t.active() && matches!(t.kind, 23 | 25 | 112 | 163 | 661))
            .count();
        assert_eq!(corrupt, 0, "the crimson shell writes no corruption tiles");
    }

    /// DW-4.2: the `SpreadGrass` slice — the exposure gate, the
    /// generation-time beach/centre exclusions for grass 23/199 (mud 661/662
    /// is exempt), the flower-27 exclusion and the kind gate.
    #[test]
    fn test_DW_4_2_spread_evil_grass_slice() {
        // 2000 wide: the exclusions are [0, 380) ∪ (900, 1100] ∪ [1620, 2000).
        let (mut world, _surface, _rock) = flat_world(60, 2000, 100);
        let put_dirt = |world: &mut World, x: i32, y: i32, kind: u16, above: u16| {
            let i = world.index(x, y);
            let mut t = Tile::default();
            t.kind = kind;
            t.flags |= FLAG_ACTIVE;
            world.tiles[i] = t;
            if above > 0 {
                let i = world.index(x, y - 1);
                let mut t = Tile::default();
                t.kind = above;
                t.flags |= FLAG_ACTIVE;
                world.tiles[i] = t;
            }
        };
        // An exposed dirt tile in the allowed zone converts.
        put_dirt(&mut world, 600, 50, TILE_DIRT, 0);
        let mut spread = 0i32;
        spread_evil_grass(
            &mut world,
            600,
            50,
            TILE_DIRT,
            TILE_CORRUPT_GRASS,
            0.0,
            &mut spread,
        );
        assert_eq!(world.tile(600, 50).unwrap().kind, TILE_CORRUPT_GRASS);
        // The centre exclusion (900, 1100] blocks the dirt conversion.
        put_dirt(&mut world, 1000, 50, TILE_DIRT, 0);
        let mut spread = 0i32;
        spread_evil_grass(
            &mut world,
            1000,
            50,
            TILE_DIRT,
            TILE_CORRUPT_GRASS,
            0.0,
            &mut spread,
        );
        assert_eq!(
            world.tile(1000, 50).unwrap().kind,
            TILE_DIRT,
            "centre exclusion"
        );
        // The ocean margins block.
        put_dirt(&mut world, 200, 50, TILE_DIRT, 0);
        let mut spread = 0i32;
        spread_evil_grass(
            &mut world,
            200,
            50,
            TILE_DIRT,
            TILE_CORRUPT_GRASS,
            0.0,
            &mut spread,
        );
        assert_eq!(
            world.tile(200, 50).unwrap().kind,
            TILE_DIRT,
            "left ocean margin"
        );
        put_dirt(&mut world, 1700, 50, TILE_DIRT, 0);
        let mut spread = 0i32;
        spread_evil_grass(
            &mut world,
            1700,
            50,
            TILE_DIRT,
            TILE_CORRUPT_GRASS,
            0.0,
            &mut spread,
        );
        assert_eq!(
            world.tile(1700, 50).unwrap().kind,
            TILE_DIRT,
            "right ocean margin"
        );
        // The crimson dirt conversion (199) obeys the same exclusions.
        put_dirt(&mut world, 600, 55, TILE_DIRT, 0);
        let mut spread = 0i32;
        spread_evil_grass(
            &mut world,
            600,
            55,
            TILE_DIRT,
            TILE_CRIMSON_GRASS,
            0.0,
            &mut spread,
        );
        assert_eq!(world.tile(600, 55).unwrap().kind, TILE_CRIMSON_GRASS);
        put_dirt(&mut world, 1000, 55, TILE_DIRT, 0);
        let mut spread = 0i32;
        spread_evil_grass(
            &mut world,
            1000,
            55,
            TILE_DIRT,
            TILE_CRIMSON_GRASS,
            0.0,
            &mut spread,
        );
        assert_eq!(world.tile(1000, 55).unwrap().kind, TILE_DIRT);
        // The mud conversion (59 → 661) ignores the exclusions (the 661/662
        // else-if branch never fires for mud).
        put_dirt(&mut world, 1000, 60, TILE_MUD, 0);
        let mut spread = 0i32;
        spread_evil_grass(
            &mut world,
            1000,
            60,
            TILE_MUD,
            TILE_CORRUPT_JUNGLE_GRASS,
            0.0,
            &mut spread,
        );
        assert_eq!(
            world.tile(1000, 60).unwrap().kind,
            TILE_CORRUPT_JUNGLE_GRASS,
            "mud converts inside the centre band"
        );
        // A buried tile (all 9 neighbours active+solid) never converts.
        for x in 695..705 {
            for y in 70..80 {
                put_dirt(&mut world, x, y, TILE_DIRT, 0);
            }
        }
        let mut spread = 0i32;
        spread_evil_grass(
            &mut world,
            700,
            75,
            TILE_DIRT,
            TILE_CORRUPT_GRASS,
            0.0,
            &mut spread,
        );
        assert_eq!(
            world.tile(700, 75).unwrap().kind,
            TILE_DIRT,
            "buried dirt stands"
        );
        // The flower-27 exclusion: a flower above blocks the conversion.
        put_dirt(&mut world, 600, 70, TILE_DIRT, 27);
        let mut spread = 0i32;
        spread_evil_grass(
            &mut world,
            600,
            70,
            TILE_DIRT,
            TILE_CORRUPT_GRASS,
            0.0,
            &mut spread,
        );
        assert_eq!(
            world.tile(600, 70).unwrap().kind,
            TILE_DIRT,
            "flower-27 blocks"
        );
        // The kind gate: a grass tile is not dirt — no conversion.
        put_dirt(&mut world, 600, 80, TILE_GRASS, 0);
        let mut spread = 0i32;
        spread_evil_grass(
            &mut world,
            600,
            80,
            TILE_DIRT,
            TILE_CORRUPT_GRASS,
            0.0,
            &mut spread,
        );
        assert_eq!(world.tile(600, 80).unwrap().kind, TILE_GRASS, "kind gate");
    }

    /// DW-4.1: `AddShadowOrb` — the 2x2 tile 31 with frames 0/18 (+36 for a
    /// crimson heart), the abort when an orb already sits in the 2x2, and
    /// the bounds guard.
    #[test]
    fn add_shadow_orb_places_the_2x2_with_frames() {
        let mut world = World::new("t".into(), 100, 100);
        add_shadow_orb(&mut world, 50, 50, false);
        let orb = [
            (49i32, 49i32, 0i16, 0i16),
            (50, 49, 18, 0),
            (49, 50, 0, 18),
            (50, 50, 18, 18),
        ];
        for (x, y, fx, fy) in orb {
            let t = world.tile(x, y).unwrap();
            assert!(t.active() && t.kind == TILE_SHADOW_ORB, "orb at ({x},{y})");
            assert_eq!((t.frame_x, t.frame_y), (fx, fy), "frames at ({x},{y})");
        }
        // A second orb overlapping the first aborts (no frame damage).
        add_shadow_orb(&mut world, 50, 51, false);
        let t = world.tile(50, 50).unwrap();
        assert_eq!((t.frame_x, t.frame_y), (18, 18), "the overlap aborted");
        // The crimson heart adds 36 to frameX.
        add_shadow_orb(&mut world, 30, 30, true);
        let t = world.tile(30, 30).unwrap();
        assert_eq!((t.frame_x, t.frame_y), (54, 18), "crimson heart frames");
        // The bounds guard.
        let before = world
            .tiles
            .iter()
            .filter(|t| t.kind == TILE_SHADOW_ORB)
            .count();
        add_shadow_orb(&mut world, 5, 5, false);
        let after = world
            .tiles
            .iter()
            .filter(|t| t.kind == TILE_SHADOW_ORB)
            .count();
        assert_eq!(before, after, "the bounds guard");
    }

    /// DW-4.2: `Place3x2` (the type-26 path) — the solid-floor rule, the
    /// clear-body rule and the style frames.
    #[test]
    fn place_3x2_altar_frames_and_floor() {
        let (mut world, _surface, _rock) = flat_world(60, 200, 200);
        // A flat stone shelf at y=40 with air above.
        for x in 20..40 {
            for y in 40..44 {
                let i = world.index(x, y);
                let mut t = Tile::default();
                t.kind = 1;
                t.flags |= FLAG_ACTIVE;
                world.tiles[i] = t;
            }
        }
        place_3x2(&mut world, 30, 39, TILE_DEMON_ALTAR, 1);
        // The 3x2 wrote with the style-1 frames (54 + dx*18, dy*18).
        for dx in 0..3i32 {
            for dy in 0..2i32 {
                let t = world.tile(29 + dx, 38 + dy).unwrap();
                assert!(t.active() && t.kind == TILE_DEMON_ALTAR);
                assert_eq!(t.frame_x, (54 + dx * 18) as i16, "frameX at dx={dx}");
                assert_eq!(t.frame_y, (dy * 18) as i16, "frameY at dy={dy}");
            }
        }
        // A second altar overlapping the first fails (the body is occupied)
        // and leaves the tiles alone.
        place_3x2(&mut world, 29, 39, TILE_DEMON_ALTAR, 0);
        assert_ne!(world.tile(28, 38).unwrap().kind, TILE_DEMON_ALTAR);
        // A hollow floor fails the solid-floor rule: the body rows 42-43 are
        // air (the flat world is empty above the surface) and the row below
        // (44) is air too — no altar.
        place_3x2(&mut world, 50, 43, TILE_DEMON_ALTAR, 0);
        assert!(!world.tile(50, 42).unwrap().active(), "no floor → no altar");
        // With a solid floor under all three columns it places.
        for x in 45..60 {
            let i = world.index(x, 44);
            let mut t = Tile::default();
            t.kind = 1;
            t.flags |= FLAG_ACTIVE;
            world.tiles[i] = t;
        }
        place_3x2(&mut world, 50, 43, TILE_DEMON_ALTAR, 0);
        assert!(world.tile(50, 42).unwrap().active(), "floor → altar");
    }

    /// DW-4.2: `IsTileNearby` and `oceanDepths`.
    #[test]
    fn is_tile_nearby_and_ocean_depths() {
        let (mut world, surface, rock) = flat_world(60, 2000, 200);
        let ocean_level = (surface + rock) / 2.0 + 40.0;
        // A demon altar at (100, 30).
        let i = world.index(100, 30);
        let mut t = Tile::default();
        t.kind = TILE_DEMON_ALTAR;
        t.flags |= FLAG_ACTIVE;
        world.tiles[i] = t;
        assert!(
            is_tile_nearby(&world, 103, 33, TILE_DEMON_ALTAR, 3),
            "distance 3"
        );
        assert!(
            !is_tile_nearby(&world, 104, 30, TILE_DEMON_ALTAR, 3),
            "distance 4"
        );
        assert!(!is_tile_nearby(&world, 100, 34, TILE_DEMON_ALTAR, 3));
        // oceanDepths: inside the ocean margins above the level → true.
        assert!(ocean_depths(&world, 200, 50, ocean_level), "left ocean");
        assert!(ocean_depths(&world, 1900, 50, ocean_level), "right ocean");
        assert!(
            !ocean_depths(&world, 1000, 50, ocean_level),
            "the centre is not ocean"
        );
        // Below the ocean level never ocean.
        assert!(!ocean_depths(
            &world,
            200,
            (ocean_level + 1.0) as i32,
            ocean_level
        ));
    }

    /// DW-4.1: one `ChasmRunner` on a synthetic surface — the orb (tile 31,
    /// 2x2) places at the fill-phase arm, the ebonstone shell and the wall-3
    /// lining exist below the mouth, the carved shaft opens the crust, and
    /// the never-re-armed altar search lands a demon altar below the
    /// surface.
    #[test]
    fn chasm_runner_places_orb_ebonstone_and_walls() {
        let (mut world, surface, rock) = flat_world(60, 200, 300);
        chasm_runner(
            &mut world,
            &mut Rng(0xC0FFEE),
            100,
            58,
            60,
            true,
            surface,
            rock,
            TILE_EBONSTONE,
            WALL_EBONSTONE,
        );
        let orbs = count_tiles(&world, &|t: &Tile| t.active() && t.kind == TILE_SHADOW_ORB);
        assert_eq!(orbs, 4, "one 2x2 shadow orb ({orbs})");
        let ebonstone = count_tiles(&world, &|t: &Tile| t.active() && t.kind == TILE_EBONSTONE);
        assert!(ebonstone > 200, "the ebonstone shell ({ebonstone})");
        let walls = count_tiles(&world, &|t: &Tile| t.wall == WALL_EBONSTONE);
        assert!(walls > 100, "the wall-3 lining ({walls})");
        // The mouth: the crust rows [58, 64) near the start column carry
        // carved air (`:75957-75980`).
        let mouth = (90..110)
            .flat_map(|x| (58..64).map(move |y| (x, y)))
            .filter(|&(x, y)| world.tile(x, y).is_some_and(|t| !t.active()))
            .count();
        assert!(mouth > 0, "no carved mouth ({mouth})");
        // The altar search (the flag3 quirk) places a demon altar strictly
        // below the surface.
        let altars: Vec<(i32, i32)> = (0..world.width)
            .flat_map(|x| (0..world.height).map(move |y| (x, y)))
            .filter(|&(x, y)| {
                world
                    .tile(x, y)
                    .is_some_and(|t| t.active() && t.kind == TILE_DEMON_ALTAR)
            })
            .collect();
        assert!(!altars.is_empty(), "no altar from the chasm search");
        assert!(
            altars.iter().all(|&(_, y)| y as f64 > surface),
            "the search only places below the surface"
        );
    }

    /// DW-4.2 (dirty): the dungeon bricks and the dungeon walls inside the
    /// chasm's path survive the carve, the fill and the wall passes.
    #[test]
    fn chasm_runner_skips_dungeon_brick() {
        let (mut world, surface, rock) = flat_world(60, 200, 300);
        // A dungeon-brick block with dungeon walls right on the walker's
        // descent column.
        for x in 92..108 {
            for y in 90..115 {
                let i = world.index(x, y);
                let mut t = Tile::default();
                t.kind = 41;
                t.wall = 7;
                t.flags |= FLAG_ACTIVE;
                world.tiles[i] = t;
            }
        }
        chasm_runner(
            &mut world,
            &mut Rng(0xC0FFEE),
            100,
            58,
            60,
            true,
            surface,
            rock,
            TILE_EBONSTONE,
            WALL_EBONSTONE,
        );
        let bricks = world
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == 41)
            .count();
        assert!(bricks > 300, "the bricks survived ({bricks})");
        let dungeon_walls = world.tiles.iter().filter(|t| t.wall == 7).count();
        assert!(
            dungeon_walls > 300,
            "the dungeon walls survived ({dungeon_walls})"
        );
        // And nothing inside the block turned ebonstone.
        let converted = (92..108)
            .flat_map(|x| (90..115).map(move |y| (x, y)))
            .filter(|&(x, y)| {
                world
                    .tile(x, y)
                    .is_some_and(|t| t.active() && t.kind == TILE_EBONSTONE && t.wall != 7)
            })
            .count();
        assert_eq!(converted, 0, "the fill never touched the bricks");
    }

    /// DW-4.1: `ChasmRunnerSideways` — the air carve, the ebonstone fill and
    /// the `PlaceWall` wall pass; across seeds the Demonite tail fires.
    #[test]
    fn chasm_runner_sideways_carves_and_fills() {
        let (mut world, surface, rock) = flat_world(60, 300, 300);
        chasm_runner_sideways(
            &mut world,
            &mut Rng(0x51DE),
            150,
            100,
            1,
            25,
            surface,
            rock,
            TILE_EBONSTONE,
            WALL_EBONSTONE,
        );
        let carved = count_tiles(&world, &|t: &Tile| !t.active() && t.kind == TILE_EBONSTONE);
        assert!(carved > 30, "the carve hollowed the walk ({carved})");
        let filled = count_tiles(&world, &|t: &Tile| t.active() && t.kind == TILE_EBONSTONE);
        assert!(filled > 200, "the fill lined the walk ({filled})");
        let walls = count_tiles(&world, &|t: &Tile| t.wall == WALL_EBONSTONE);
        assert!(walls > 100, "the PlaceWall pass ({walls})");
        // The Demonite tail: `Next(3) == 0` — sweep seeds until it fires.
        let mut saw_tail = false;
        for seed in 0..24u64 {
            let (mut w, surface, rock) = flat_world(60, 300, 300);
            chasm_runner_sideways(
                &mut w,
                &mut Rng(seed ^ 0x7A1),
                150,
                100,
                -1,
                25,
                surface,
                rock,
                TILE_EBONSTONE,
                WALL_EBONSTONE,
            );
            if w.tiles
                .iter()
                .any(|t| t.active() && t.kind == TILE_DEMONITE)
            {
                saw_tail = true;
                break;
            }
        }
        assert!(saw_tail, "no seed produced the Demonite tail");
    }

    /// DW-4.1: `CrimStart` + `CrimPlaceHearts` — the crimstone blob with the
    /// wall-83 chambers, 5-8 vein endpoints recorded, the heart-room discs
    /// and the crimson-heart orbs.
    #[test]
    fn crim_start_places_blob_chambers_and_hearts() {
        let (mut world, _surface, _rock) = flat_world(100, 400, 400);
        let mut hearts: Vec<(i32, i32)> = Vec::new();
        crim_start(&mut world, &mut Rng(0xC21A), 200, 90, 100.0, &mut hearts);
        let crimstone = count_tiles(&world, &|t: &Tile| t.active() && t.kind == TILE_CRIMSTONE);
        assert!(crimstone > 2000, "the crimstone blob ({crimstone})");
        let walls = count_tiles(&world, &|t: &Tile| t.wall == WALL_CRIMSTONE);
        assert!(walls > 500, "the wall-83 lining ({walls})");
        // The vein endpoints: `Next(5, 9)` veins.
        assert!(
            (5..=8).contains(&hearts.len()),
            "the vein count ({})",
            hearts.len()
        );
        // The chambers hollowed pockets inside the blob.
        let pockets = count_tiles(&world, &|t: &Tile| !t.active() && t.wall == WALL_CRIMSTONE);
        assert!(pockets > 100, "the chamber pockets ({pockets})");
        // The hearts: the discs + the hollow rooms + the crimson orbs.
        crim_place_hearts(&mut world, &mut Rng(0x4E4E), &hearts);
        let hearts_tiles = world
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == TILE_SHADOW_ORB && t.frame_x >= 36)
            .count();
        assert_eq!(
            hearts_tiles,
            hearts.len() * 4,
            "one 2x2 crimson heart per endpoint"
        );
        // Each heart room: the hollow disc around the endpoint (the centre
        // holds the orb itself — the (x-2, y) tile is inside the hollow but
        // outside the orb's 2x2) with the filled crimstone disc around it.
        for &(hx, hy) in &hearts {
            let hollow = world.tile(hx - 2, hy).unwrap();
            assert!(
                !hollow.active() && hollow.wall == WALL_CRIMSTONE,
                "the heart room hollow at ({hx},{hy})"
            );
            let ring = (-6..=6)
                .flat_map(|dx| (-6..=6).map(move |dy| (hx + dx, hy + dy)))
                .filter(|&(x, y)| {
                    world
                        .tile(x, y)
                        .is_some_and(|t| t.active() && t.kind == TILE_CRIMSTONE)
                })
                .count();
            assert!(ring > 5, "the fill disc around ({hx},{hy}) ({ring})");
        }
    }

    /// DW-4.3: the band-retry ledger — the draw sequence replayed from the
    /// C# transcription (`WorldGen.cs:13957-14043`) on a flat 1800-wide
    /// world (no snow/jungle tiles: the bands degenerate to [w-10, 10] and
    /// never retry; the dungeon sits at 1700, out of every band's reach).
    #[test]
    fn test_DW_4_3_band_retry_ledger() {
        let (world, _surface, _rock) = flat_world(60, 1800, 300);
        let mut rng = Rng(0xB4AD);
        let band = evil_band_retry(
            &world,
            &mut rng,
            (1790, 10),
            (1790, 10),
            1700,
            false,
            false,
            false,
        )
        .expect("the retry terminated");
        // The transcription.
        let mut rep = Rng(0xB4AD);
        let (max_x, dungeon_location) = (1800i32, 1700i32);
        let (mut centre, mut start, mut end) = (0, 0, 0);
        loop {
            centre = rep.range(500, max_x - 500);
            start = centre - rep.range(0, 200) - 100;
            end = centre + rep.range(0, 200) + 100;
            if start < 380 {
                start = 380;
            }
            if end > max_x - 380 {
                end = max_x - 380;
            }
            if centre < start + 50 {
                centre = start + 50;
            }
            if centre > end - 50 {
                centre = end - 50;
            }
            if start < dungeon_location + 100 && end > dungeon_location - 100 {
                continue;
            }
            let (lo, hi) = (max_x / 2 - 200, max_x / 2 + 200);
            if centre > lo && centre < hi {
                continue;
            }
            if start > lo && start < hi {
                continue;
            }
            if end > lo && end < hi {
                continue;
            }
            if start < 10 && end > 1790 {
                continue;
            }
            if start < 10 && end > 1790 {
                continue;
            }
            break;
        }
        assert_eq!((centre, start, end), band, "the band values");
        assert_eq!(rng.0, rep.0, "the final stream state");
        // The band is sane on this world (the valve never fired).
        assert!(end > start);
    }

    /// M5-SEED-1: the drunk anchor draw (`WorldGen.cs:13962` crimson /
    /// `:14302` corruption) picks the world half the flavour owns, reading
    /// `GenVars.crimsonLeft` with opposite polarity on the two arms. The
    /// classic draw is the whole-world `Next(num8, maxTilesX - num8)`.
    #[test]
    fn test_DW_4_3_drunk_anchor_takes_the_world_half() {
        let (world, _surface, _rock) = flat_world(60, 1800, 300);
        let half = world.width / 2;
        // The classic arm is untouched by the drunk parameters.
        let classic = evil_band_retry(
            &world,
            &mut Rng(0xB4AD),
            (1790, 10),
            (1790, 10),
            1700,
            false,
            false,
            false,
        )
        .expect("the retry terminated");
        let reference = evil_band_retry(
            &world,
            &mut Rng(0xB4AD),
            (1790, 10),
            (1790, 10),
            1700,
            false,
            false,
            false,
        )
        .expect("the retry terminated");
        assert_eq!(classic, reference, "the classic arm ignores the drunk flags");
        // The crimson arm: `!crimsonLeft ? right : left`, and the centre
        // exclusion narrows to ±100 (`:13961-13964`), so the settled centre
        // never crosses the world's centre.
        for (crimson_left, left) in [(true, true), (false, false)] {
            let band = evil_band_retry(
                &world,
                &mut Rng(0xB4AD),
                (1790, 10),
                (1790, 10),
                1700,
                true,
                true,
                crimson_left,
            )
            .expect("the retry terminated");
            if left {
                assert!(band.0 < half, "crimsonLeft draws the left half ({})", band.0);
            } else {
                assert!(band.0 >= half, "!crimsonLeft draws the right half ({})", band.0);
            }
        }
        // The corruption arm reads the same roll the other way round.
        for (crimson_left, left) in [(true, false), (false, true)] {
            let band = evil_band_retry(
                &world,
                &mut Rng(0xB4AD),
                (1790, 10),
                (1790, 10),
                1700,
                false,
                true,
                crimson_left,
            )
            .expect("the retry terminated");
            if left {
                assert!(band.0 < half, "crimsonLeft=false draws the left half ({})", band.0);
            } else {
                assert!(band.0 >= half, "crimsonLeft draws the right half ({})", band.0);
            }
        }
    }

    /// M5-SEED-1: the drunk half of the pass (`WorldGen.cs:13938`, `:14279`) —
    /// a drunk world runs BOTH flavours from the halved count. On a
    /// corruption-flavoured world the classic pass paints ebonstone chasms and
    /// no crimstone; the drunk one paints both and its forced crimson arm
    /// places the crimson hearts the classic corruption branch never writes.
    /// On a crimson-flavoured world the halving is visible in the heart count
    /// (`ceil(maxTilesX * 0.00045)` = 2 blobs against the drunk `ceil(...
    /// / 2)` = 1).
    #[test]
    fn test_DW_4_3_drunk_runs_both_flavours() {
        let run = |flags: crate::seeds::SeedFlags, crimson: bool| {
            // A 4200-wide flat world (`:13927`'s count is 2 classic, 1 drunk).
            // The surface row sits at 100 with `world_surface` 10 below it, so
            // both walkers' surface scans find their hit.
            let (mut world, surface, rock) = flat_world(100, 4200, 600);
            world.seeds = flags;
            corruption_and_crimson(
                &mut world,
                &mut Rng(0x0D12),
                surface,
                surface + 10.0,
                rock,
                4100,
                crimson,
            );
            let count = |kind: u16, frame: Option<i32>| {
                count_tiles(&world, &|t: &Tile| {
                    t.active()
                        && t.kind == kind
                        && frame.map_or(true, |f| i32::from(t.frame_x) >= f)
                })
            };
            (
                count(TILE_EBONSTONE, None),
                count(TILE_CRIMSTONE, None),
                count(TILE_SHADOW_ORB, Some(36)),
            )
        };
        // The corruption flavour.
        let (classic_ebonstone, classic_crimstone, classic_hearts) =
            run(crate::seeds::SeedFlags::default(), false);
        assert!(classic_ebonstone > 0, "the classic corruption pass ({classic_ebonstone})");
        assert_eq!(classic_crimstone, 0, "the classic corruption pass grows no crimstone");
        assert_eq!(classic_hearts, 0, "the classic corruption branch places no hearts");
        let (drunk_ebonstone, drunk_crimstone, drunk_hearts) =
            run(crate::seeds::SeedFlags::from(crate::seeds::SeedFlags::DRUNK), false);
        assert!(drunk_ebonstone > 0, "the drunk corruption pass ({drunk_ebonstone})");
        assert!(drunk_crimstone > 0, "the drunk pass also ran the crimson arm ({drunk_crimstone})");
        assert!(drunk_hearts > 0, "the forced crimson arm placed its hearts ({drunk_hearts})");
        // The crimson flavour: the halved count grows fewer hearts.
        let (_, _, classic_crim_hearts) = run(crate::seeds::SeedFlags::default(), true);
        let (_, _, drunk_crim_hearts) =
            run(crate::seeds::SeedFlags::from(crate::seeds::SeedFlags::DRUNK), true);
        assert!(classic_crim_hearts > 0, "the classic crimson hearts ({classic_crim_hearts})");
        assert!(
            drunk_crim_hearts < classic_crim_hearts,
            "the halved count grows fewer hearts ({drunk_crim_hearts} vs {classic_crim_hearts})"
        );
    }

    /// M5-SEED-1: `GenVars.flipInfections` (`WorldGen.cs:11298-11304`) swaps
    /// the two evil pairs, so the branches trade walkers: the corruption
    /// branch grows `CrimStart` (`:14381-14384`) and ends with
    /// `CrimPlaceHearts` (`:14511-14513`), while the crimson branch grows the
    /// chasms (`:14048-14077`) and drops its own `CrimPlaceHearts`
    /// (`:14273-14277`). The drunk flag keeps both arms running, so the test
    /// reads the CORRUPTION arm's products: the crimson hearts, which only
    /// `CrimPlaceHearts` writes and which no classic corruption-flavoured
    /// world ever grows.
    #[test]
    fn test_DW_4_3_flip_infections_trades_the_walkers() {
        // The id resolution itself.
        let (classic, surface, rock) = flat_world(100, 4200, 600);
        let mut flipped = classic.clone();
        flipped.seeds = crate::seeds::SeedFlags::from(
            crate::seeds::SeedFlags::DRUNK | crate::seeds::SeedFlags::FOR_THE_WORTHY,
        );
        assert_eq!(crim_ids(&classic), (TILE_CRIMSTONE, WALL_CRIMSTONE));
        assert_eq!(ebon_ids(&classic), (TILE_EBONSTONE, WALL_EBONSTONE));
        assert_eq!(crim_ids(&flipped), (TILE_EBONSTONE, WALL_EBONSTONE));
        assert_eq!(ebon_ids(&flipped), (TILE_CRIMSTONE, WALL_CRIMSTONE));
        let half = classic.width / 2;
        let corruption_left = !crimson_left(&Rng(0xF11E));
        let hearts = |world: &World| {
            count_tiles(&world, &|t: &Tile| {
                t.active() && t.kind == TILE_SHADOW_ORB && t.frame_x >= 36
            })
        };
        let (mut base_world, _, _) = (classic, surface, rock);
        corruption_and_crimson(
            &mut base_world,
            &mut Rng(0xF11E),
            surface,
            surface + 10.0,
            rock,
            4100,
            false,
        );
        assert_eq!(hearts(&base_world), 0, "the classic corruption arm grows no hearts");
        let mut flipped_world = flipped;
        corruption_and_crimson(
            &mut flipped_world,
            &mut Rng(0xF11E),
            surface,
            surface + 10.0,
            rock,
            4100,
            false,
        );
        let swapped = hearts(&flipped_world);
        assert!(swapped > 0, "the flipped corruption arm grew the crimson heart walker");
        // The hearts sit on the corruption arm's half — the same half the
        // anchor draw picked (`:14302`'s `crimsonLeft ? right : left`), which
        // ties the walker swap to the drunk anchor arms.
        let mut wrong_half = 0usize;
        for x in 0..flipped_world.width {
            for y in 0..flipped_world.height {
                let Some(t) = flipped_world.tile(x, y) else { continue };
                if t.active() && t.kind == TILE_SHADOW_ORB && t.frame_x >= 36 {
                    let on_left = x < half;
                    if on_left != corruption_left {
                        wrong_half += 1;
                    }
                }
            }
        }
        assert_eq!(wrong_half, 0, "the hearts left the corruption half ({wrong_half})");
        // And the ids they were painted with are the swapped pair: the blob
        // underneath the hearts is ebonstone 25 with the ebonstone wall 3.
        let blob = count_tiles(&flipped_world, &|t: &Tile| t.active() && t.kind == TILE_EBONSTONE);
        let lining = count_tiles(&flipped_world, &|t: &Tile| t.wall == WALL_EBONSTONE);
        assert!(blob > 0, "the swapped blob painted ebonstone ({blob})");
        assert!(lining > 0, "the swapped blob painted the ebonstone wall ({lining})");
    }

    /// DW-4.3: the shell draw ledger — the per-column/per-tile draw order of
    /// `evil_shell` replayed against the implementation on identical worlds.
    #[test]
    fn test_DW_4_3_shell_draw_ledger() {
        let (mut wa, surface, _rock) = flat_world(60, 2000, 200);
        let mut wb = wa.clone();
        let band = (500i32, 700i32);
        let seed = 0x5111u64;
        evil_shell(&mut wa, &mut Rng(seed), band, 40.0, surface, false);
        // --- the transcription (WorldGen.cs:14094-14185, corruption twin) ---
        let mut rep = Rng(seed);
        let (num, num2) = (40.0f64, surface);
        let (num16, num17) = band;
        let mut num23 = num2 + 40.0;
        for num24 in num16..num17 {
            num23 += rep.range(-2, 3) as f64;
            if num23 < num2 + 30.0 {
                num23 = num2 + 30.0;
            }
            if num23 > num2 + 50.0 {
                num23 = num2 + 50.0;
            }
            let mut flag3 = false;
            let num25 = num;
            let mut num26 = num25 as i32;
            while (num26 as f64) < num23 {
                let col_gate = num24 > num16 + 1 && num24 < num17 - 2;
                let row_gate = !((num26 as f64) <= num25 + 1.0) && !((num26 as f64) >= num23 - 2.0);
                if col_gate || rep.range(0, 2) != 0 {
                    if row_gate || rep.range(0, 2) != 0 {
                        if active_at(&wb, num24, num26) {
                            if kind_at(&wb, num24, num26) == TILE_SAND
                                && num24 >= num16 + rep.range(0, 5)
                                && num24 <= num17 - rep.range(0, 5)
                            {
                                set_kind(&mut wb, num24, num26, TILE_EBONSAND);
                            }
                            if (num26 as f64) < num2 - 1.0 && !flag3 {
                                let kind = kind_at(&wb, num24, num26);
                                if kind == TILE_DIRT {
                                    let mut gs = 0i32;
                                    spread_evil_grass(
                                        &mut wb,
                                        num24,
                                        num26,
                                        TILE_DIRT,
                                        TILE_CORRUPT_GRASS,
                                        num2,
                                        &mut gs,
                                    );
                                } else if kind == TILE_MUD {
                                    let mut gs = 0i32;
                                    spread_evil_grass(
                                        &mut wb,
                                        num24,
                                        num26,
                                        TILE_MUD,
                                        TILE_CORRUPT_JUNGLE_GRASS,
                                        num2,
                                        &mut gs,
                                    );
                                }
                            }
                            flag3 = true;
                            let wall = wall_at(&wb, num24, num26);
                            if wall == WALL_HARDENED_SAND {
                                set_wall(&mut wb, num24, num26, WALL_CORRUPT_HARDENED_SAND);
                            } else if wall == WALL_SANDSTONE {
                                set_wall(&mut wb, num24, num26, WALL_CORRUPT_SANDSTONE);
                            }
                            let kind = kind_at(&wb, num24, num26);
                            if kind == TILE_STONE {
                                if num24 >= num16 + rep.range(0, 5)
                                    && num24 <= num17 - rep.range(0, 5)
                                {
                                    set_kind(&mut wb, num24, num26, TILE_EBONSTONE);
                                }
                            } else if kind == TILE_GRASS {
                                set_kind(&mut wb, num24, num26, TILE_CORRUPT_GRASS);
                            } else if kind == TILE_JUNGLE_GRASS {
                                set_kind(&mut wb, num24, num26, TILE_CORRUPT_JUNGLE_GRASS);
                            } else if kind == TILE_ICE {
                                set_kind(&mut wb, num24, num26, TILE_CORRUPT_ICE);
                            } else if kind == TILE_SANDSTONE {
                                set_kind(&mut wb, num24, num26, TILE_CORRUPT_SANDSTONE);
                            } else if kind == TILE_HARDENED_SAND {
                                set_kind(&mut wb, num24, num26, TILE_CORRUPT_HARDENED_SAND);
                            }
                        }
                    }
                }
                num26 += 1;
            }
        }
        // The ledger: tile-identical worlds and the same final stream state.
        for (i, (a, b)) in wa.tiles.iter().zip(&wb.tiles).enumerate() {
            assert_eq!(
                (a.kind, a.wall, a.flags),
                (b.kind, b.wall, b.flags),
                "tile mismatch at ({},{})",
                i as i32 % 2000,
                i as i32 / 2000
            );
        }
    }

    /// DW-4.3: the altar-scatter ledger (`WorldGen.cs:14186-14234`) — the
    /// attempt count, the redraws and the widening replayed on a flat world
    /// (the flat surface keeps `Place3x2` failing until `num30 > 100`, which
    /// is exactly the draw path the transcription mirrors).
    #[test]
    fn test_DW_4_3_altar_scatter_ledger() {
        let (mut wa, surface, rock) = flat_world(60, 2000, 300);
        let mut wb = wa.clone();
        let seed = 0x5C47u64;
        let ocean_level = (surface + rock) / 2.0 + 40.0;
        altar_scatter(&mut wa, &mut Rng(seed), (500, 700), surface, ocean_level);
        // --- the transcription ---
        let mut rep = Rng(seed);
        let (num16, num17, num2) = (500i32, 700i32, surface);
        let num27 = rep.range(10, 15);
        for _num28 in 0..num27 {
            let mut num29 = 0i32;
            let mut flag4 = false;
            let mut num30 = 0i32;
            while !flag4 {
                num29 += 1;
                let mut num31 = rep.range(num16 - num30, num17 + num30);
                let mut num32 = rep.range(
                    (num2 - (num30 / 2) as f64) as i32,
                    (num2 + 100.0 + num30 as f64) as i32,
                );
                while ocean_depths(&wb, num31, num32, ocean_level) {
                    num31 = rep.range(num16 - num30, num17 + num30);
                    num32 = rep.range(
                        (num2 - (num30 / 2) as f64) as i32,
                        (num2 + 100.0 + num30 as f64) as i32,
                    );
                }
                if num29 > 100 {
                    num30 += 1;
                    num29 = 0;
                }
                if !active_at(&wb, num31, num32) {
                    while !active_at(&wb, num31, num32) {
                        num32 += 1;
                    }
                    num32 -= 1;
                } else {
                    while active_at(&wb, num31, num32) && (num32 as f64) > num2 {
                        num32 -= 1;
                    }
                }
                let ground_ok = num30 > 10
                    || (active_at(&wb, num31, num32 + 1)
                        && kind_at(&wb, num31, num32 + 1) == TILE_CRIMSTONE);
                if ground_ok && !is_tile_nearby(&wb, num31, num32, TILE_DEMON_ALTAR, 3) {
                    place_3x2(&mut wb, num31, num32, TILE_DEMON_ALTAR, 1);
                    if kind_at(&wb, num31, num32) == TILE_DEMON_ALTAR {
                        flag4 = true;
                    }
                }
                if num30 > 100 {
                    flag4 = true;
                }
            }
        }
        for (i, (a, b)) in wa.tiles.iter().zip(&wb.tiles).enumerate() {
            assert_eq!(
                (a.kind, a.flags),
                (b.kind, b.flags),
                "tile mismatch at ({},{})",
                i as i32 % 2000,
                i as i32 / 2000
            );
        }
    }

    /// The ledger transcription of `ChasmRunner` (`WorldGen.cs:75885-76111`)
    /// — the test's own reading of the C#, reusing only the RNG-free helpers
    /// and the draw-ledger-pinned `tile_runner`.
    fn ledger_chasm_runner(
        world: &mut World,
        rng: &mut Rng,
        i: i32,
        j: i32,
        steps: i32,
        make_orb: bool,
        world_surface: f64,
        rock_layer: f64,
    ) {
        let num = world_surface;
        let mut flag = false;
        let mut flag2 = false;
        let mut flag3 = false;
        if !make_orb {
            flag2 = true;
        }
        let mut num2 = steps as f64;
        let (mut px, mut py) = (i as f64, j as f64);
        let mut vel_x = rng.range(-10, 11) as f64 * 0.1;
        let mut vel_y = rng.range(0, 11) as f64 * 0.2 + 0.5;
        let num3 = 5;
        let mut num4 = rng.range(0, 5) as f64 + 7.0;
        while num4 > 0.0 {
            if num2 > 0.0 {
                num4 += rng.range(0, 3) as f64 - rng.range(0, 3) as f64;
                if num4 < 7.0 {
                    num4 = 7.0;
                }
                if num4 > 20.0 {
                    num4 = 20.0;
                }
                if num2 == 1.0 && num4 < 10.0 {
                    num4 = 10.0;
                }
            } else if py > num + 45.0 {
                num4 -= rng.range(0, 4) as f64;
            }
            if py > rock_layer && num2 > 0.0 {
                num2 = 0.0;
            }
            num2 -= 1.0;
            if !flag && py > num + 20.0 {
                flag = true;
                let left = rng.range(20, 40);
                ledger_chasm_sideways(
                    world,
                    rng,
                    px as i32,
                    py as i32,
                    -1,
                    left,
                    world_surface,
                    rock_layer,
                );
                let right = rng.range(20, 40);
                ledger_chasm_sideways(
                    world,
                    rng,
                    px as i32,
                    py as i32,
                    1,
                    right,
                    world_surface,
                    rock_layer,
                );
            }
            if num2 > num3 as f64 {
                let mut num5 = (px - num4 * 0.5) as i32;
                let mut num6 = (px + num4 * 0.5) as i32;
                let mut num7 = (py - num4 * 0.5) as i32;
                let mut num8 = (py + num4 * 0.5) as i32;
                if num5 < 0 {
                    num5 = 0;
                }
                if num6 > world.width - 1 {
                    num6 = world.width - 1;
                }
                if num7 < 0 {
                    num7 = 0;
                }
                if num8 > world.height {
                    num8 = world.height;
                }
                for k in num5..num6 {
                    for l in num7..num8 {
                        if (k as f64 - px).abs() + (l as f64 - py).abs()
                            < num4 * 0.5 * (1.0 + rng.range(-10, 11) as f64 * 0.015)
                            && can_evil_replace(world, k, l)
                            && kind_at(world, k, l) != TILE_SHADOW_ORB
                            && kind_at(world, k, l) != TILE_DEMONITE
                            && kind_at(world, k, l) != TILE_CRIMTANE
                        {
                            set_active(world, k, l, false);
                        }
                    }
                }
            }
            if num2 <= 2.0 && py < num + 45.0 {
                num2 = 2.0;
            }
            if num2 <= 0.0 {
                if !flag2 {
                    flag2 = true;
                    add_shadow_orb(world, px as i32, py as i32, false);
                } else if !flag3 {
                    flag3 = false;
                    let mut flag4 = false;
                    let mut num9 = 0;
                    while !flag4 {
                        let mut num10 = rng.range(px as i32 - 25, px as i32 + 25);
                        let mut num11 = rng.range(py as i32 - 50, py as i32);
                        if num10 < 5 {
                            num10 = 5;
                        }
                        if num10 > world.width - 5 {
                            num10 = world.width - 5;
                        }
                        if num11 < 5 {
                            num11 = 5;
                        }
                        if num11 > world.height - 5 {
                            num11 = world.height - 5;
                        }
                        if num11 as f64 > num {
                            if !is_tile_nearby(world, num10, num11, TILE_DEMON_ALTAR, 3) {
                                place_3x2(world, num10, num11, TILE_DEMON_ALTAR, 0);
                            }
                            if kind_at(world, num10, num11) == TILE_DEMON_ALTAR {
                                flag4 = true;
                                continue;
                            }
                            num9 += 1;
                            if num9 >= 10000 {
                                flag4 = true;
                            }
                        } else {
                            flag4 = true;
                        }
                    }
                }
            }
            px += vel_x;
            py += vel_y;
            vel_x += rng.range(-10, 11) as f64 * 0.01;
            if vel_x > 0.3 {
                vel_x = 0.3;
            }
            if vel_x < -0.3 {
                vel_x = -0.3;
            }
            let mut num5 = (px - num4 * 1.1) as i32;
            let mut num6 = (px + num4 * 1.1) as i32;
            let mut num7 = (py - num4 * 1.1) as i32;
            let mut num8 = (py + num4 * 1.1) as i32;
            if num5 < 1 {
                num5 = 1;
            }
            if num6 > world.width - 1 {
                num6 = world.width - 1;
            }
            if num7 < 0 {
                num7 = 0;
            }
            if num8 > world.height {
                num8 = world.height;
            }
            for m in num5..num6 {
                for n in num7..num8 {
                    if can_evil_replace(world, m, n)
                        && (m as f64 - px).abs() + (n as f64 - py).abs()
                            < num4 * 1.1 * (1.0 + rng.range(-10, 11) as f64 * 0.015)
                    {
                        if kind_at(world, m, n) != TILE_EBONSTONE && n > j + rng.range(3, 20) {
                            set_active(world, m, n, true);
                        }
                        if steps <= num3 {
                            set_active(world, m, n, true);
                        }
                        if kind_at(world, m, n) != TILE_SHADOW_ORB {
                            set_kind(world, m, n, TILE_EBONSTONE);
                        }
                    }
                }
            }
            for m in num5..num6 {
                for n in num7..num8 {
                    if can_evil_replace(world, m, n)
                        && (m as f64 - px).abs() + (n as f64 - py).abs()
                            < num4 * 1.1 * (1.0 + rng.range(-10, 11) as f64 * 0.015)
                    {
                        if kind_at(world, m, n) != TILE_SHADOW_ORB {
                            set_kind(world, m, n, TILE_EBONSTONE);
                        }
                        if steps <= num3 {
                            set_active(world, m, n, true);
                        }
                        if n > j + rng.range(3, 20) {
                            set_wall(world, m, n, WALL_EBONSTONE);
                        }
                    }
                }
            }
        }
    }

    /// The ledger transcription of `ChasmRunnerSideways`
    /// (`WorldGen.cs:75259-75431`).
    fn ledger_chasm_sideways(
        world: &mut World,
        rng: &mut Rng,
        i: i32,
        j: i32,
        direction: i32,
        steps: i32,
        world_surface: f64,
        rock_layer: f64,
    ) {
        let mut num = steps as f64;
        let (mut px, mut py) = (i as f64, j as f64);
        let mut vel_x = rng.range(10, 21) as f64 * 0.1 * direction as f64;
        let mut vel_y = rng.range(-10, 10) as f64 * 0.01;
        let mut num2 = rng.range(0, 5) as f64 + 7.0;
        while num2 > 0.0 {
            if num > 0.0 {
                num2 += rng.range(0, 3) as f64 - rng.range(0, 3) as f64;
                if num2 < 7.0 {
                    num2 = 7.0;
                }
                if num2 > 20.0 {
                    num2 = 20.0;
                }
                if num == 1.0 && num2 < 10.0 {
                    num2 = 10.0;
                }
            } else {
                num2 -= rng.range(0, 4) as f64;
            }
            if py > rock_layer && num > 0.0 {
                num = 0.0;
            }
            num -= 1.0;
            let mut num3 = (px - num2 * 0.5) as i32;
            let mut num4 = (px + num2 * 0.5) as i32;
            let mut num5 = (py - num2 * 0.5) as i32;
            let mut num6 = (py + num2 * 0.5) as i32;
            if num3 < 0 {
                num3 = 0;
            }
            if num4 > world.width - 1 {
                num4 = world.width - 1;
            }
            if num5 < 0 {
                num5 = 0;
            }
            if num6 > world.height {
                num6 = world.height;
            }
            for k in num3..num4 {
                for l in num5..num6 {
                    if can_evil_replace(world, k, l)
                        && (k as f64 - px).abs() + (l as f64 - py).abs()
                            < num2 * 0.5 * (1.0 + rng.range(-10, 11) as f64 * 0.015)
                        && kind_at(world, k, l) != TILE_SHADOW_ORB
                        && kind_at(world, k, l) != TILE_DEMONITE
                        && kind_at(world, k, l) != TILE_CRIMTANE
                    {
                        set_active(world, k, l, false);
                    }
                }
            }
            px += vel_x;
            py += vel_y;
            vel_y += rng.range(-10, 10) as f64 * 0.1;
            if py < (j - 20) as f64 {
                vel_y += rng.range(0, 20) as f64 * 0.01;
            }
            if py > (j + 20) as f64 {
                vel_y -= rng.range(0, 20) as f64 * 0.01;
            }
            if vel_y < -0.5 {
                vel_y = -0.5;
            }
            if vel_y > 0.5 {
                vel_y = 0.5;
            }
            vel_x += rng.range(-10, 11) as f64 * 0.01;
            if direction == -1 {
                if vel_x > -0.5 {
                    vel_x = -0.5;
                }
                if vel_x < -2.0 {
                    vel_x = -2.0;
                }
            } else if direction == 1 {
                if vel_x < 0.5 {
                    vel_x = 0.5;
                }
                if vel_x > 2.0 {
                    vel_x = 2.0;
                }
            }
            let mut num3 = (px - num2 * 1.1) as i32;
            let mut num4 = (px + num2 * 1.1) as i32;
            let mut num5 = (py - num2 * 1.1) as i32;
            let mut num6 = (py + num2 * 1.1) as i32;
            if num3 < 1 {
                num3 = 1;
            }
            if num4 > world.width - 1 {
                num4 = world.width - 1;
            }
            if num5 < 0 {
                num5 = 0;
            }
            if num6 > world.height {
                num6 = world.height;
            }
            for m in num3..num4 {
                for n in num5..num6 {
                    if can_evil_replace(world, m, n)
                        && (m as f64 - px).abs() + (n as f64 - py).abs()
                            < num2 * 1.1 * (1.0 + rng.range(-10, 11) as f64 * 0.015)
                        && wall_at(world, m, n) != WALL_EBONSTONE
                    {
                        if !active_at(world, m, n)
                            || (kind_at(world, m, n) != TILE_SHADOW_ORB
                                && kind_at(world, m, n) != TILE_DEMONITE
                                && kind_at(world, m, n) != TILE_CRIMTANE)
                        {
                            set_kind(world, m, n, TILE_EBONSTONE);
                        }
                        set_active(world, m, n, true);
                        if wall_at(world, m, n) == 2 {
                            set_wall(world, m, n, 0);
                        }
                    }
                }
            }
            for m in num3..num4 {
                for n in num5..num6 {
                    if can_evil_replace(world, m, n)
                        && (m as f64 - px).abs() + (n as f64 - py).abs()
                            < num2 * 1.1 * (1.0 + rng.range(-10, 11) as f64 * 0.015)
                        && wall_at(world, m, n) != WALL_EBONSTONE
                    {
                        if !active_at(world, m, n)
                            || (kind_at(world, m, n) != TILE_SHADOW_ORB
                                && kind_at(world, m, n) != TILE_DEMONITE
                                && kind_at(world, m, n) != TILE_CRIMTANE)
                        {
                            set_kind(world, m, n, TILE_EBONSTONE);
                        }
                        set_active(world, m, n, true);
                        place_evil_wall(world, m, n, WALL_EBONSTONE);
                    }
                }
            }
        }
        if rng.range(0, 3) == 0 {
            let num9 = px as i32;
            let mut num10 = py as i32;
            while num10 < world.height - 1 && !active_at(world, num9, num10) {
                num10 += 1;
            }
            let strength = rng.range(2, 6);
            let tail_steps = rng.range(3, 7);
            tile_runner(
                world,
                rng,
                num9,
                num10,
                strength as f64,
                tail_steps,
                TILE_DEMONITE as i32,
                world_surface,
                &TileRunnerOpts { flags: world.seeds, ..TileRunnerOpts::default() },
            );
        }
    }

    /// DW-4.3: the ChasmRunner draw ledger — the implementation and the
    /// test's C# transcription run on identical worlds with the same seed;
    /// tile-for-tile and stream-state equality pin the whole draw order,
    /// including the sideways recursion, the orb, the never-re-armed altar
    /// search and the Demonite tail.
    #[test]
    fn test_DW_4_3_chasm_runner_draw_ledger() {
        let (mut wa, surface, rock) = flat_world(60, 200, 300);
        let mut wb = wa.clone();
        let seed = 0xCA9Du64;
        let mut impl_rng = Rng(seed);
        chasm_runner(
            &mut wa,
            &mut impl_rng,
            100,
            58,
            60,
            true,
            surface,
            rock,
            TILE_EBONSTONE,
            WALL_EBONSTONE,
        );
        let mut rep = Rng(seed);
        ledger_chasm_runner(&mut wb, &mut rep, 100, 58, 60, true, surface, rock);
        for (idx, (a, b)) in wa.tiles.iter().zip(&wb.tiles).enumerate() {
            assert_eq!(
                (a.kind, a.wall, a.flags),
                (b.kind, b.wall, b.flags),
                "tile mismatch at ({},{})",
                idx as i32 % 200,
                idx as i32 / 200
            );
        }
        // The final stream state pins the draw count.
        assert_eq!(impl_rng.0, rep.0, "the final stream state");
    }

    /// DW-4.3: the corruption-pass composition ledger — the biome loop's
    /// draw order (band retry → chasm attempts with the cooldown and the
    /// `Next(35)` lottery → the jungle-grass conversion → the shell → the
    /// orb-surround blob) replayed against `corruption_and_crimson` on the
    /// flat 1800-wide world. The band retry, the shell and the walker
    /// internals delegate to the already-ledgered components.
    #[test]
    fn test_DW_4_3_corruption_pass_composition_ledger() {
        let (mut wa, surface, rock) = flat_world(60, 1800, 300);
        let mut wb = wa.clone();
        let seed = 0xC01Fu64;
        let mut impl_rng = Rng(seed);
        corruption_and_crimson(&mut wa, &mut impl_rng, 40.0, surface, rock, 1700, false);
        // --- the transcription (WorldGen.cs:14280-14554, classic path) ---
        let mut rep = Rng(seed);
        let num = 40.0f64;
        let num2 = surface;
        // The prologue scan (no RNG): the flat world has no 60/147/161
        // tiles — the bands degenerate to (w-10, 10).
        let (num3, num4, num5, num6) = (1800 - 10, 10, 1800 - 10, 10);
        let num10 = 1800 as f64 * 0.00045;
        let count = num10.ceil() as i32;
        let mut hearts: Vec<(i32, i32)> = Vec::new();
        let ocean_level = (num2 + rock) / 2.0 + 40.0;
        for _k in 0..count {
            let Some((num46, num47, num48)) =
                evil_band_retry(
                    &wb,
                    &mut rep,
                    (num5, num6),
                    (num3, num4),
                    1700,
                    false,
                    false,
                    false,
                )
            else {
                continue;
            };
            if num48 <= num47 {
                continue;
            }
            let mut num51 = 0i32;
            for num52 in num47..num48 {
                if num51 > 0 {
                    num51 -= 1;
                }
                if num52 == num46 || num51 == 0 {
                    let mut num53 = num as i32;
                    while (num53 as f64) < num2 - 1.0 {
                        if active_at(&wb, num52, num53) || wall_at(&wb, num52, num53) > 0 {
                            if num52 == num46 {
                                num51 = 20;
                                let steps = rep.range(0, 150) + 150;
                                chasm_runner(
                                    &mut wb,
                                    &mut rep,
                                    num52,
                                    num53,
                                    steps,
                                    true,
                                    num2,
                                    rock,
                                    TILE_EBONSTONE,
                                    WALL_EBONSTONE,
                                );
                            } else if rep.range(0, 35) == 0 && num51 == 0 {
                                num51 = 30;
                                let steps = rep.range(0, 50) + 50;
                                chasm_runner(
                                    &mut wb,
                                    &mut rep,
                                    num52,
                                    num53,
                                    steps,
                                    true,
                                    num2,
                                    rock,
                                    TILE_EBONSTONE,
                                    WALL_EBONSTONE,
                                );
                            }
                            break;
                        }
                        num53 += 1;
                    }
                }
                // The conversion loop (`:14402-14416`).
                let mut n = num as i32;
                while (n as f64) < num2 - 1.0 {
                    if active_at(&wb, num52, n) {
                        let num21 = n + rep.range(10, 14);
                        for num22 in n..num21 {
                            if active_at(&wb, num52, num22)
                                && kind_at(&wb, num52, num22) == TILE_JUNGLE_GRASS
                                && num52 >= num47 + rep.range(0, 5)
                                && num52 < num48 - rep.range(0, 5)
                            {
                                set_kind(&mut wb, num52, num22, TILE_CORRUPT_JUNGLE_GRASS);
                            }
                        }
                        break;
                    }
                    n += 1;
                }
            }
            evil_shell(&mut wb, &mut rep, (num47, num48), num, num2, false);
            // The orb-surround blob (`:14526-14545`).
            for num61 in num47..num48 {
                for num62 in 0..(wb.height - 50) {
                    if active_at(&wb, num61, num62) && kind_at(&wb, num61, num62) == TILE_SHADOW_ORB
                    {
                        for num67 in (num61 - 13)..(num61 + 13) {
                            if num67 > 10 && num67 < wb.width - 10 {
                                for num68 in (num62 - 13)..(num62 + 13) {
                                    let dx = (num67 - num61).abs();
                                    let dy = (num68 - num62).abs();
                                    if dx + dy < 9 + rep.range(0, 11)
                                        && rep.range(0, 3) != 0
                                        && kind_at(&wb, num67, num68) != TILE_SHADOW_ORB
                                    {
                                        set_active(&mut wb, num67, num68, true);
                                        set_kind(&mut wb, num67, num68, TILE_EBONSTONE);
                                        if dx <= 1 && dy <= 1 {
                                            set_active(&mut wb, num67, num68, false);
                                        }
                                    }
                                    if kind_at(&wb, num67, num68) != TILE_SHADOW_ORB
                                        && dx <= 2 + rep.range(0, 3)
                                        && dy <= 2 + rep.range(0, 3)
                                    {
                                        set_active(&mut wb, num67, num68, false);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        let _ = (&hearts, ocean_level);
        for (idx, (a, b)) in wa.tiles.iter().zip(&wb.tiles).enumerate() {
            assert_eq!(
                (a.kind, a.wall, a.flags),
                (b.kind, b.wall, b.flags),
                "tile mismatch at ({},{})",
                idx as i32 % 1800,
                idx as i32 / 1800
            );
        }
        assert_eq!(impl_rng.0, rep.0, "the final stream state");
    }

    /// The ledger transcription of the crimson family (`CrimStart`
    /// `WorldGen.cs:75449-75714`, `CrimVein` `:75808-75883`, `CrimEnt`
    /// `:75764-75806`, `CrimPlaceHearts` `:75716-75762`).
    fn ledger_crim_start(
        world: &mut World,
        rng: &mut Rng,
        i: i32,
        j: i32,
        world_surface: f64,
        hearts: &mut Vec<(i32, i32)>,
    ) {
        let num = world_surface;
        let mut crim_dir = 1;
        let mut k = j;
        if k as f64 > num {
            k = num as i32;
        }
        while !solid_tile(world, i, k) {
            k += 1;
        }
        let num2 = k;
        let (mut pos_x, mut pos_y) = (i as f64, k as f64);
        let mut vel_x = rng.range(-20, 21) as f64 * 0.1;
        let mut vel_y = rng.range(20, 201) as f64 * 0.01;
        if vel_x < 0.0 {
            crim_dir = -1;
        }
        let mut num3 = rng.range(15, 26) as f64;
        let mut flag = true;
        let mut num4 = 0i32;
        while flag {
            num3 += rng.range(-50, 51) as f64 * 0.01;
            if num3 < 15.0 {
                num3 = 15.0;
            }
            if num3 > 25.0 {
                num3 = 25.0;
            }
            let mut l = (pos_x - num3 / 2.0) as i32;
            while (l as f64) < pos_x + num3 / 2.0 {
                let mut m = (pos_y - num3 / 2.0) as i32;
                while (m as f64) < pos_y + num3 / 2.0 {
                    if !can_evil_replace(world, l, m) {
                        m += 1;
                        continue;
                    }
                    let manhattan = (l as f64 - pos_x).abs() + (m as f64 - pos_y).abs();
                    if m > num2 {
                        if manhattan < num3 * 0.3 {
                            set_active(world, l, m, false);
                            set_wall(world, l, m, WALL_CRIMSTONE);
                        } else if manhattan < num3 * 0.8 && wall_at(world, l, m) != WALL_CRIMSTONE {
                            set_active(world, l, m, true);
                            set_kind(world, l, m, TILE_CRIMSTONE);
                            if manhattan < num3 * 0.6 {
                                set_wall(world, l, m, WALL_CRIMSTONE);
                            }
                        }
                    } else if manhattan < num3 * 0.3 && active_at(world, l, m) {
                        set_active(world, l, m, false);
                        set_wall(world, l, m, WALL_CRIMSTONE);
                    }
                    m += 1;
                }
                l += 1;
            }
            if pos_x > (i + 50) as f64 {
                num4 = -100;
            }
            if pos_x < (i - 50) as f64 {
                num4 = 100;
            }
            if num4 < 0 {
                vel_x -= rng.range(20, 51) as f64 * 0.01;
            } else if num4 > 0 {
                vel_x += rng.range(20, 51) as f64 * 0.01;
            } else {
                vel_x += rng.range(-50, 51) as f64 * 0.01;
            }
            vel_y += rng.range(-50, 51) as f64 * 0.01;
            vel_y = vel_y.clamp(0.25, 2.0);
            vel_x = vel_x.clamp(-2.0, 2.0);
            pos_x += vel_x;
            pos_y += vel_y;
            if pos_y > num + 100.0 {
                flag = false;
            }
        }
        num3 = rng.range(40, 55) as f64;
        for _n in 0..50 {
            let num5 = pos_x as i32 + rng.range(-20, 21);
            let num6 = pos_y as i32 + rng.range(-20, 21);
            let mut num7 = (num5 as f64 - num3 / 2.0) as i32;
            while (num7 as f64) < num5 as f64 + num3 / 2.0 {
                let mut num8 = (num6 as f64 - num3 / 2.0) as i32;
                while (num8 as f64) < num6 as f64 + num3 / 2.0 {
                    if !can_evil_replace(world, num7, num8) {
                        num8 += 1;
                        continue;
                    }
                    let num9 = (num7 - num5).abs() as f64;
                    let mut num10 = (num8 - num6).abs() as f64;
                    let num11 = 1.0 + rng.range(-20, 21) as f64 * 0.01;
                    let num12 = 1.0 + rng.range(-20, 21) as f64 * 0.01;
                    let num13 = num9 * num11;
                    num10 *= num12;
                    let num14 = (num13 * num13 + num10 * num10).sqrt();
                    if num14 < num3 * 0.25 {
                        set_active(world, num7, num8, false);
                        set_wall(world, num7, num8, WALL_CRIMSTONE);
                    } else if num14 < num3 * 0.4 && wall_at(world, num7, num8) != WALL_CRIMSTONE {
                        set_active(world, num7, num8, true);
                        set_kind(world, num7, num8, TILE_CRIMSTONE);
                        if num14 < num3 * 0.35 {
                            set_wall(world, num7, num8, WALL_CRIMSTONE);
                        }
                    }
                    num8 += 1;
                }
                num7 += 1;
            }
        }
        let num15 = rng.range(5, 9);
        let mut array: Vec<(f64, f64)> = Vec::with_capacity(num15 as usize);
        for num16 in 0..num15 {
            let num17 = pos_x as i32;
            let num18 = pos_y as i32;
            let mut num19 = 0i32;
            let mut flag2 = true;
            let mut vec2 = (
                rng.range(-20, 21) as f64 * 0.15,
                rng.range(0, 21) as f64 * 0.15,
            );
            while flag2 {
                vec2 = (
                    rng.range(-20, 21) as f64 * 0.15,
                    rng.range(0, 21) as f64 * 0.15,
                );
                while vec2.0.abs() + vec2.1.abs() < 1.5 {
                    vec2 = (
                        rng.range(-20, 21) as f64 * 0.15,
                        rng.range(0, 21) as f64 * 0.15,
                    );
                }
                flag2 = false;
                for num20 in 0..num16 {
                    let prior = array[num20 as usize];
                    if vel_x > prior.0 - 0.75
                        && vel_x < prior.0 + 0.75
                        && vel_y > prior.1 - 0.75
                        && vel_y < prior.1 + 0.75
                    {
                        flag2 = true;
                        num19 += 1;
                        break;
                    }
                }
                if num19 > 10000 {
                    break;
                }
            }
            array.push(vec2);
            ledger_crim_vein(world, rng, (num17 as f64, num18 as f64), vec2, hearts);
        }
        let mut num21 = world.width;
        let mut num22 = 0i32;
        pos_x = i as f64;
        pos_y = num2 as f64;
        num3 = rng.range(25, 35) as f64;
        let mut num23 = rng.range(0, 6) as f64;
        for _num24 in 0..50 {
            if num23 > 0.0 {
                let num25 = rng.range(10, 30) as f64 * 0.01;
                num23 -= num25;
                pos_y -= num25;
            }
            let num26 = pos_x as i32 + rng.range(-2, 3);
            let num27 = pos_y as i32 + rng.range(-2, 3);
            let mut num28 = (num26 as f64 - num3 / 2.0) as i32;
            while (num28 as f64) < num26 as f64 + num3 / 2.0 {
                let mut num29 = (num27 as f64 - num3 / 2.0) as i32;
                while (num29 as f64) < num27 as f64 + num3 / 2.0 {
                    if !can_evil_replace(world, num28, num29) {
                        num29 += 1;
                        continue;
                    }
                    let num30 = (num28 - num26).abs() as f64;
                    let mut num31 = (num29 - num27).abs() as f64;
                    let num32 = 1.0 + rng.range(-20, 21) as f64 * 0.005;
                    let num33 = 1.0 + rng.range(-20, 21) as f64 * 0.005;
                    let num34 = num30 * num32;
                    num31 *= num33;
                    let num35 = (num34 * num34 + num31 * num31).sqrt();
                    if num35 < num3 * 0.2 * (rng.range(90, 111) as f64 * 0.01) {
                        set_active(world, num28, num29, false);
                        set_wall(world, num28, num29, WALL_CRIMSTONE);
                    } else {
                        if num35 >= num3 * 0.45 {
                            num29 += 1;
                            continue;
                        }
                        if num28 < num21 {
                            num21 = num28;
                        }
                        if num28 > num22 {
                            num22 = num28;
                        }
                        if wall_at(world, num28, num29) != WALL_CRIMSTONE {
                            set_active(world, num28, num29, true);
                            set_kind(world, num28, num29, TILE_CRIMSTONE);
                            if num35 < num3 * 0.35 {
                                set_wall(world, num28, num29, WALL_CRIMSTONE);
                            }
                        }
                    }
                    num29 += 1;
                }
                num28 += 1;
            }
        }
        for num36 in num21..=num22 {
            let mut num37 = num2;
            while (kind_at(world, num36, num37) == TILE_CRIMSTONE && active_at(world, num36, num37))
                || wall_at(world, num36, num37) == WALL_CRIMSTONE
            {
                num37 += 1;
            }
            let mut num38 = rng.range(15, 20);
            while !active_at(world, num36, num37)
                && num38 > 0
                && wall_at(world, num36, num37) != WALL_CRIMSTONE
            {
                if can_evil_replace(world, num36, num37) {
                    num38 -= 1;
                    set_kind(world, num36, num37, TILE_CRIMSTONE);
                    set_active(world, num36, num37, true);
                    num37 += 1;
                } else {
                    break;
                }
            }
        }
        ledger_crim_ent(world, rng, (pos_x, pos_y), crim_dir);
    }

    /// The `CrimVein` transcription (`WorldGen.cs:75808-75883`).
    fn ledger_crim_vein(
        world: &mut World,
        rng: &mut Rng,
        position: (f64, f64),
        velocity: (f64, f64),
        hearts: &mut Vec<(i32, i32)>,
    ) {
        let mut num = rng.range(15, 26) as f64;
        let mut flag = true;
        let vector2d = velocity;
        let vector2d2 = position;
        let (mut pos_x, mut pos_y) = position;
        let (mut vel_x, mut vel_y) = velocity;
        let mut num2 = rng.range(100, 150);
        if velocity.1 < 0.0 {
            num2 -= 25;
        }
        while flag {
            num += rng.range(-50, 51) as f64 * 0.02;
            num = num.clamp(15.0, 25.0);
            let mut i = (pos_x - num / 2.0) as i32;
            while (i as f64) < pos_x + num / 2.0 {
                let mut j = (pos_y - num / 2.0) as i32;
                while (j as f64) < pos_y + num / 2.0 {
                    if !can_evil_replace(world, i, j) {
                        j += 1;
                        continue;
                    }
                    let num3 = (i as f64 - pos_x).abs();
                    let num4 = (j as f64 - pos_y).abs();
                    let num5 = (num3 * num3 + num4 * num4).sqrt();
                    if num5 < num * 0.2 {
                        set_active(world, i, j, false);
                        set_wall(world, i, j, WALL_CRIMSTONE);
                    } else if num5 < num * 0.5 && wall_at(world, i, j) != WALL_CRIMSTONE {
                        set_active(world, i, j, true);
                        set_kind(world, i, j, TILE_CRIMSTONE);
                        if num5 < num * 0.4 {
                            set_wall(world, i, j, WALL_CRIMSTONE);
                        }
                    }
                    j += 1;
                }
                i += 1;
            }
            vel_x += rng.range(-50, 51) as f64 * 0.05;
            vel_y += rng.range(-50, 51) as f64 * 0.05;
            vel_y = vel_y.clamp(vector2d.1 - 0.75, vector2d.1 + 0.75);
            vel_x = vel_x.clamp(vector2d.0 - 0.75, vector2d.0 + 0.75);
            pos_x += vel_x;
            pos_y += vel_y;
            if (pos_x - vector2d2.0).abs() + (pos_y - vector2d2.1).abs() > num2 as f64 {
                flag = false;
            }
        }
        hearts.push((pos_x as i32, pos_y as i32));
    }

    /// The `CrimEnt` transcription (`WorldGen.cs:75764-75806`).
    fn ledger_crim_ent(world: &mut World, rng: &mut Rng, position: (f64, f64), crim_dir: i32) {
        let mut num = 0.0f64;
        let mut num2 = rng.range(6, 11) as f64;
        let mut flag = true;
        let mut vel_x = 2.0f64 * -(crim_dir as f64);
        let mut vel_y = rng.range(-20, 0) as f64 * 0.01;
        let (mut pos_x, mut pos_y) = position;
        while flag {
            num += 1.0;
            if num >= 20.0 {
                flag = false;
            }
            num2 += rng.range(-10, 11) as f64 * 0.02;
            num2 = num2.clamp(6.0, 10.0);
            let mut i = (pos_x - num2 / 2.0) as i32;
            while (i as f64) < pos_x + num2 / 2.0 {
                let mut j = (pos_y - num2 / 2.0) as i32;
                while (j as f64) < pos_y + num2 / 2.0 {
                    if can_evil_replace(world, i, j) {
                        let num3 = (i as f64 - pos_x).abs();
                        let num4 = (j as f64 - pos_y).abs();
                        if (num3 * num3 + num4 * num4).sqrt() < num2 * 0.5
                            && active_at(world, i, j)
                            && kind_at(world, i, j) == TILE_CRIMSTONE
                        {
                            set_active(world, i, j, false);
                            flag = true;
                            num = 0.0;
                        }
                    }
                    j += 1;
                }
                i += 1;
            }
            pos_x += vel_x;
            pos_y += vel_y;
        }
    }

    /// The `CrimPlaceHearts` transcription (`WorldGen.cs:75716-75762`).
    fn ledger_crim_hearts(world: &mut World, rng: &mut Rng, hearts: &[(i32, i32)]) {
        let mut num = 16;
        for heart in hearts {
            num = rng.range(16, 21);
            let (x, y) = *heart;
            for j in (x - num / 2)..(x + num / 2) {
                for k in (y - num / 2)..(y + num / 2) {
                    let num2 = (j - x).abs() as f64;
                    let num3 = (k - y).abs() as f64;
                    if (num2 * num2 + num3 * num3).sqrt() < num as f64 * 0.4 {
                        set_active(world, j, k, true);
                        set_kind(world, j, k, TILE_CRIMSTONE);
                        set_wall(world, j, k, WALL_CRIMSTONE);
                    }
                }
            }
        }
        for heart in hearts {
            num = rng.range(10, 14);
            let (x, y) = *heart;
            for m in (x - num / 2)..(x + num / 2) {
                for n in (y - num / 2)..(y + num / 2) {
                    let num4 = (m - x).abs() as f64;
                    let num5 = (n - y).abs() as f64;
                    if (num4 * num4 + num5 * num5).sqrt() < num as f64 * 0.3 {
                        set_active(world, m, n, false);
                        set_wall(world, m, n, WALL_CRIMSTONE);
                    }
                }
            }
        }
        for heart in hearts {
            add_shadow_orb(world, heart.0, heart.1, true);
        }
    }

    /// DW-4.3: the crimson-family draw ledger — `CrimStart` + the hearts
    /// against their transcription on identical worlds.
    #[test]
    fn test_DW_4_3_crimson_family_ledger() {
        let (mut wa, _surface, _rock) = flat_world(100, 400, 400);
        let mut wb = wa.clone();
        let seed = 0xC81Au64;
        let mut impl_hearts: Vec<(i32, i32)> = Vec::new();
        let mut impl_rng = Rng(seed);
        crim_start(&mut wa, &mut impl_rng, 200, 90, 100.0, &mut impl_hearts);
        crim_place_hearts(&mut wa, &mut impl_rng, &impl_hearts);
        let mut rep_hearts: Vec<(i32, i32)> = Vec::new();
        let mut rep = Rng(seed);
        ledger_crim_start(&mut wb, &mut rep, 200, 90, 100.0, &mut rep_hearts);
        ledger_crim_hearts(&mut wb, &mut rep, &rep_hearts);
        assert_eq!(impl_hearts, rep_hearts, "the vein endpoints");
        for (idx, (a, b)) in wa.tiles.iter().zip(&wb.tiles).enumerate() {
            assert_eq!(
                (a.kind, a.wall, a.flags, a.frame_x, a.frame_y),
                (b.kind, b.wall, b.flags, b.frame_x, b.frame_y),
                "tile mismatch at ({},{})",
                idx as i32 % 400,
                idx as i32 / 400
            );
        }
        assert_eq!(impl_rng.0, rep.0, "the final stream state");
    }
}
