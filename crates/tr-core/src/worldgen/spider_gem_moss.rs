//! The cave decorations — the spider caves, the gem caves, the moss caves and
//! the neon moss biomes (M5-STRUCT-5).
//!
//! Ports the four vanilla passes that dress the cave air, in the pipeline
//! order `WorldGen.AddPasses` registers them:
//! - `Spider Caves` (`WorldGen.cs:17356-17427`): the pocket search
//!   (`WorldGen.countTiles`/`nextCount` `:9546-9635`) and the flood fill
//!   `Spread.Spider` (`:3682-3789`) with its decor chain (`PlaceTight`
//!   `:38303-38325` → `PlaceUncheckedStalactite` `:38327-38641`, `PlacePot`
//!   `:54066-54114`, `PlaceSmallPile` `:46815-46873`).
//! - `Gem Caves` (`WorldGen.cs:17428-17471`): `gemCave` (`:9719-9735`),
//!   `randGem` (`:9737-9745`), `randGemTile` (`:9747-9762`) and the flood
//!   fill `Spread.Gem` (`:3594-3680`).
//! - `Moss` (`WorldGen.cs:17472-17700`): `randMoss` (`:9764-9781`),
//!   `neonMossBiome` (`:9783-9876`), `setMoss` (`:9878-9884`), `Spread.Moss`
//!   (`:3526-3592`) and the scatter/arm sweeps.
//! - `Webs And Honey` (`WorldGen.cs:19995-20063`): the post-hoc pass that
//!   fills the spider walls with cobweb and the hive walls with honey +
//!   speleothems, keyed purely on the wall id the earlier passes left.
//!
//! Tiles (`TileID.cs`): the stone 1 (`:631`), the dirt 0, the gray brick 38
//! (`:705`), the clay 40, the mud 59 (`:736`), the jungle grass 60 (`:741`),
//! the mushroom grass 70 (`:781`), the snow 147 (`:878`), the ice 161
//! (`:891`), the ebonstone 25 (`:679`), the crimstone 203 (`:1035`), the
//! pearlstone 117 (`:863`), the marble 367 (`:1365`), the granite 368
//! (`:1367`), the hive 225 (`:1081`), the larva 231 (`:1093`), the pots 28
//! (`:516`), the cobweb 51 (`:731`), the stalactite 165 (`:959`), the small
//! piles 185 (`:1001`), the large piles 187 (`:1003`), the exposed gems 178
//! (`:985`), the gems 67/66/63/65/64/68 (`:755-765`) and the moss family
//! 179-183 (`:987-995`) with the specials 381/534/536/539/625 (`:1391`,
//! `:1697`, `:1701`, `:1707`, `:1879`).
//! Walls (`WallID.cs`): the stone 1 (`:107`), the dirt 2 (`:109`), the gem
//! walls 48-53 (`:201-211`), the cave walls 54-58 (`:213-221`), the spider
//! wall 62 (`:229`), the hive wall 86 (`:414`) and the living wood 244
//! (`:556`).
//!
//! Deviations (ANALYSIS "Cave decorations (M5-STRUCT-5)"):
//! - **the wall-0 gates read the port's pre-laid walls as the C#'s 0.** The
//!   port's `fill_terrain` writes a wall under every solid row (dirt 2 /
//!   stone 1 / sandstone 187 — `terrain.rs:284-289`), while the C# leaves
//!   them 0 until `Cave Walls` runs (`WorldGen.cs:17711`, after these
//!   passes). So every `wall != 0` test in `nextCount`, `Spread.Spider`,
//!   `Spread.Gem` and `Spread.Moss` is evaluated against
//!   [`BASELINE_WALLS`] as if the cell were unwalled; only a wall the port
//!   actually painted (jungle/mud/hive/gem/moss/cave walls) counts.
//! - the RNG is the port's per-pass xorshift stream
//!   (`Streams::pass_rng("Spider Caves")` etc. — the port's model of the
//!   vanilla per-pass re-seed `WorldGenerator.cs:497`), not the shared
//!   `UnifiedRandom`; the draw ORDER is faithful, the byte stream is not
//!   (the port's worldgen contract is already xorshift).
//! - `Spread.Spider`'s `AddBuriedChest` arm is the port's `place_chest`
//!   (`structures.rs:39`) — the chest's stocked loot is the M5-CHEST-gen
//!   work, so the tile lands without the C#'s `chestStyle 15` book.
//! - `PlaceTile`-shaped writes use the port's single-tile idiom
//!   (`structures.rs:344`): `frame_x = style * 18`, `frame_y = 0`, instead of
//!   the C#'s 2x2/3x2 `TileObjectData` frames (the port's framer recomputes
//!   the object frames at the pipeline end).
//! - `neonMossBiome`/`SpreadGrass` ride the port's one-shot
//!   `growth::spread_grass` (`growth.rs:601`, `WorldGen.cs:75146-75270`);
//!   the C#'s bounded 3x3 recursion (`grassSpread < 1000`) is not modelled —
//!   the sweeps call it per neighbour instead.
//! - W4-g41 ports the egg-sac validator chain (`CheckStalactite`
//!   `WorldGen.cs:38996-39175` → `UpdateStalagtiteStyle` `:38643-38718` →
//!   `GetStalagtiteStyle`/`GetDesiredStalagtiteStyle` `:38823-38872`/
//!   `:38720-38821` + `InvalidTileForPilesOrSpeleothems` `:38979-38994`)
//!   verbatim, wired through `PlaceTight`'s tail (`:38320-38323`); the
//!   `destroyObject` kills ride `edit::remove_tile` (the generation strip —
//!   the guarded `Check*` re-entries early-return in the C#, so no cascade).
//!   The missing `PlaceUncheckedStalactite` arms (hanging granite/marble,
//!   rising hive), the `tileMoss[627]` member, the `PlacePot` halfBrick
//!   clear + remix centre veto (`:54069-54072`, W4-g41 FINAL), the
//!   `PlaceSmallPile` boulder-seat veto + `anyLava` guard, the
//!   spider pass's `shroomCount > 1` retry veto + `ILied` tail dispatch +
//!   remix y-bands, the webs pass's DST row-50 start + `ILied` honey arm and
//!   `SpiderWallRemover` (`:77279-77344`, WIRED by the W4-g41 follow-up —
//!   its `FinishNotTheBees` host is ported in `not_the_bees.rs`) land with it.
//!   Census: `docs/research/93-spider-cave-census.md`.
//! - M5-SEED-1 ports the for-the-worthy arms of this file against the world's
//!   own flags (`World::seeds`, the `generate` publication of
//!   `GenOptions::flags` — a classic world reads no flag and stays
//!   byte-identical): the spider-cave attempt count (`WorldGen.cs:17378`),
//!   the neon-moss x band (`:17503-17508`) and the neon-moss blob radius and
//!   length (`:9802-9805`). Still unported: the remix, tenth-anniversary,
//!   not-the-bees and dont-starve arms (alternative C# draw schedules, not
//!   mere multipliers) and the secret-seed `noSpiderCaves` pair
//!   (`:269-291`, `:2618`). The classic 0.38-0.62 neon-moss centre re-roll
//!   (`:17510-17516`) is not ported either — the port's classic arm draws
//!   `num4` once, and adding that re-roll would shift a classic world's
//!   draws; the drunk world, whose C# arm *skips* the re-roll, therefore
//!   matches the port's classic path exactly.
//! - no `GenVars.shimmerPosition`/`shimmerSafetyDistance` in the port
//!   (shimmer is not a worldgen liquid yet) — the moss pass's shimmer
//!   exclusion arm (`WorldGen.cs:17585`, `:17604`, `:17614`) is skipped.
//! - the `Webs And Honey` hive arm keys on wall 86 (`WallID.HiveUnsafe`),
//!   which the port's beehive pass does not paint (`structures.rs:404` uses
//!   its own walls), so that arm is a no-op until the beehive wall lands.
//!
//! W4-c29 gem-moss fidelity (ANALYSIS "Gem-moss fidelity (W4-c29)"):
//! - `Spread.Gem`'s `hashSet.Add` return is ignored (`WorldGen.cs:3617`) —
//!   a twice-enqueued cell re-processes and its solid branch re-draws the
//!   `randGemTile` conversions; the former dedup dropped those draws.
//! - the exposed-gem write is `PlaceTile`'s 178 arm (`:59691-59701`): the
//!   `CheckAndAdjustMultiDirectionalTile` anchor gate (`:60314-60366`, the
//!   slope-allowing four-face anchors) + the `Next(3)*18` frame-row draw —
//!   the former `place_framed` write carried neither.
//! - `neonMossBiome`'s classic clamps ride the C#'s edges (`num6` top under
//!   the rock line, `num7` bottom at maxY, `:9843-9858`), SET the drift to
//!   ±5.0, keep the step budget a scaled double and paint the exclusive
//!   loops — the M5-STRUCT-5 draft inverted the clamp edge, nudged ±2.0,
//!   truncated the budget and painted the inclusive rim.
//! - the moss pass's arm A runs the C#'s `num2`/`num3` reject ledger
//!   (`:17554-17566`) and the exclusion scan carries `Main.tileDungeon`;
//!   arm F walks the map live column-major over `Main.tileMoss` (627
//!   included) instead of a row-major snapshot.
//! - `neonMossBiome`'s remix clamp (`Main.remixWorld`, `:9832-9842`) stays
//!   unported with the other remix draw schedules.

use std::collections::HashSet;

use super::structures::place_chest;
use super::{
    Rng, TILE_DIRT, TILE_SAND, TILE_SNOW, TILE_STONE, WALL_DIRT, WALL_SANDSTONE, WALL_STONE,
};
use crate::growth::spread_grass;
use crate::tile::{Tile, FLAG_ACTIVE, FLAG_HALF_BRICK};
use crate::tiledata as td;
use crate::tiledata::{SET_BOULDERS, TILE_MOSS, TILE_SOLID, TILE_SOLID_TOP};
use crate::world::World;

// --- the ids these passes paint ----------------------------------------------

/// `TileID.Clay` (`TileID.cs:640`) — a `Spread.Gemmable` target.
const TILE_CLAY: u16 = 40;
/// `TileID.Mud` (`TileID.cs:736`).
const TILE_MUD: u16 = 59;
/// `TileID.JungleGrass` (`TileID.cs:741`).
const TILE_JUNGLE_GRASS: u16 = 60;
/// `TileID.Pearlstone` (`TileID.cs:863`) — a `Spread.Gemmable` target.
const TILE_PEARLSTONE: u16 = 117;
/// `TileID.Ebonstone` (`TileID.cs:679`).
const TILE_EBONSTONE: u16 = 25;
/// `TileID.Crimstone` (`TileID.cs:1035`).
const TILE_CRIMSTONE: u16 = 203;
/// `TileID.Marble` (`TileID.cs:1365`).
const TILE_MARBLE: u16 = 367;
/// `TileID.Granite` (`TileID.cs:1367`).
const TILE_GRANITE: u16 = 368;
/// `TileID.GrayBrick` (`TileID.cs:705`) — a `Spread.Gemmable` target.
const TILE_GRAY_BRICK: u16 = 38;
/// `TileID.IceBlock` (`TileID.cs:891`).
const TILE_ICE: u16 = 161;
/// `TileID.MushroomGrass` (`TileID.cs:781`).
const TILE_MUSHROOM_GRASS: u16 = 70;
/// `TileID.Hive` (`TileID.cs:1081`).
const TILE_HIVE: u16 = 225;
/// `TileID.Larva` (`TileID.cs:1093`).
const TILE_LARVA: u16 = 231;
/// `TileID.Pots` (`TileID.cs:516`).
const TILE_POT: u16 = 28;
/// `TileID.Cobweb` (`TileID.cs:731`).
const TILE_COBWEB: u16 = 51;
/// `TileID.Stalactite` (`TileID.cs:959`).
const TILE_STALACTITE: u16 = 165;
/// `TileID.SmallPiles` (`TileID.cs:1001`).
const TILE_SMALL_PILES: u16 = 185;
/// `TileID.LargePiles2` (`TileID.cs:1003`).
const TILE_LARGE_PILES: u16 = 187;
/// `TileID.ExposedGems` (`TileID.cs:985`).
const TILE_EXPOSED_GEMS: u16 = 178;
/// `TileID.LavaMoss` (`TileID.cs:1391`).
const TILE_LAVA_MOSS: u16 = 381;
/// `TileID.GreenMoss`..`PurpleMoss` (`TileID.cs:987-995`).
const MOSS_TILES: [u16; 5] = [179, 180, 181, 182, 183];
/// The neon moss list `WorldGen.randMoss` draws from (`WorldGen.cs:9766`):
/// `ArgonMoss`, `XenonMoss`, `KryptonMoss`, `VioletMoss`.
const NEON_MOSS: [u16; 4] = [539, 536, 534, 625];
/// The gem tiles by `randGem()` index (`WorldGen.cs:9747-9762`):
/// amethyst, topaz, sapphire, emerald, ruby, diamond.
const GEM_TILES: [u16; 6] = [67, 66, 63, 65, 64, 68];
/// `WallID.AmethystUnsafe`..`DiamondUnsafe` (`WallID.cs:201-211`).
const WALL_GEM_BASE: u16 = 48;
/// `WallID.CaveUnsafe` (`WallID.cs:213`).
const WALL_MOSS_BASE: u16 = 54;
/// `WallID.HiveUnsafe` (`WallID.cs:414`).
const WALL_HIVE: u16 = 86;
/// `WallID.SpiderUnsafe` (`WallID.cs:229`).
const WALL_SPIDER: u16 = 62;
/// `WallID.LivingWoodUnsafe` (`WallID.cs:556`) — `nextCount` bails on it.
const WALL_LIVING_WOOD: u16 = 244;

/// `Main.tileDungeon` (`Main.cs:7992-7997`) — the six dungeon bricks: the
/// blue/green/pink 41/43/44 plus the ancient 677/678/679. The moss pass's
/// neon-anchor exclusion scan reads it (`WorldGen.cs:17545`).
const TILE_DUNGEON: [u16; 6] = [41, 43, 44, 677, 678, 679];

/// The walls the port's terrain pre-lays under every solid row
/// (`terrain.rs:284-289`) — the deviation counterpart of the C#'s `wall == 0`.
const BASELINE_WALLS: [u16; 4] = [WALL_STONE, WALL_DIRT, WALL_SANDSTONE, 216];

/// The port's adaptation of the C#'s `tile.wall != 0` test: a cell is
/// "walled" only when it carries a wall the port actually painted.
fn has_real_wall(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    !BASELINE_WALLS.contains(&t.wall)
}

/// `WorldGen.SolidTile` (`WorldGen.cs:70111-70132`) — a null tile reads solid;
/// an out-of-range tile reads false (the C#'s `catch`).
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && TILE_SOLID.has(t.kind)
        && !TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && (t.flags & crate::tile::FLAG_INACTIVE) == 0
        // `WorldGen.cs:17360` sets `Main.tileSolid[379] = false` for the whole
        // pass — the bubble tile is not solid for the pocket search.
        && t.kind != 379
}

/// `WorldGen.SolidTile2` — the `SolidTile` without the half-brick / slope
/// condition the C# uses for the pile seats (`WorldGen.cs:46841`).
fn solid_tile2(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active() && TILE_SOLID.has(t.kind) && (t.flags & crate::tile::FLAG_INACTIVE) == 0
}

// --- the pocket measure ------------------------------------------------------

/// `WorldGen.countTiles`'s scratch state (`WorldGen.cs:9546-9557`): the
/// counters the pass's acceptance tests read.
#[derive(Default, Clone, Copy)]
pub(super) struct CaveCounts {
    pub(super) tiles: i32,
    shroom: i32,
    lava: i32,
    ice: i32,
    sand: i32,
    rock: i32,
    /// The C# bails by setting `numTileCount = maxTileCount` (the border, the
    /// living-wood wall, the shimmer liquid, an unwalled cell under the
    /// non-jungle reading, or the lava when `!lavaOk`).
    bailed: bool,
}

/// `WorldGen.countTiles` (`WorldGen.cs:9546-9635`) over an explicit stack: the
/// C# recursion depth is the pocket size, so the port walks it iteratively
/// (the fill carries no RNG, so the traversal order is free). Shared with
/// the dead-man's-chest pocket gate (`mini_biomes::dead_mans_good_spot`,
/// `WorldGen.cs:9546-9635` — the same `countTiles(jungle: false, lavaOk:
/// true)` reading, the same baseline-wall adaptation).
pub(super) fn count_tiles(world: &World, x: i32, y: i32, lava_ok: bool, max: i32) -> CaveCounts {
    let mut c = CaveCounts::default();
    let mut seen: HashSet<(i32, i32)> = HashSet::new();
    let mut stack = vec![(x, y)];
    while let Some((x, y)) = stack.pop() {
        if c.tiles >= max {
            return c;
        }
        if x <= 1 || y <= 1 || x >= world.width - 1 || y >= world.height - 1 {
            c.bailed = true;
            c.tiles = max;
            return c;
        }
        if !seen.insert((x, y)) {
            continue;
        }
        let Some(t) = world.tile(x, y) else {
            continue;
        };
        // The wall gate (the C# `!jungle` branch — every pass here runs with
        // `jungle: false`). See the module header for the baseline adaptation.
        if has_real_wall(world, x, y) || t.wall == WALL_LIVING_WOOD {
            c.bailed = true;
            c.tiles = max;
            return c;
        }
        let lava_here = t.liquid > 0 && t.liquid_kind() == crate::tile::LiquidKind::Lava;
        let shimmer_here = t.liquid > 0 && t.liquid_kind() == crate::tile::LiquidKind::Shimmer;
        if shimmer_here {
            c.bailed = true;
            c.tiles = max;
            return c;
        }
        if lava_here {
            c.lava += 1;
            if !lava_ok {
                c.bailed = true;
                c.tiles = max;
                return c;
            }
        }
        if t.active() {
            if t.kind == TILE_MUSHROOM_GRASS {
                c.shroom += 1;
            }
            if t.kind == TILE_STONE {
                c.rock += 1;
            }
            if t.kind == TILE_SNOW || t.kind == TILE_ICE {
                c.ice += 1;
            }
            if t.kind == TILE_SAND || t.kind == 396 || t.kind == 397 {
                c.sand += 1;
            }
        }
        if !solid_tile(world, x, y) {
            c.tiles += 1;
            stack.push((x - 1, y));
            stack.push((x + 1, y));
            stack.push((x, y - 1));
            stack.push((x, y + 1));
        }
    }
    c
}

/// `WorldGen.InWorld(i, j, fluff)` (`WorldGen.cs:9688-9695`).
fn in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && y >= fluff && x < world.width - fluff && y < world.height - fluff
}

// --- the gem cave ------------------------------------------------------------

/// `WorldGen.randGem` (`WorldGen.cs:9737-9745`) — the rejection loop until an
/// enabled gem index comes out.
fn rand_gem(rng: &mut Rng, gem: &[bool; 6]) -> usize {
    loop {
        let num = rng.range(0, 6) as usize;
        if gem[num] {
            return num;
        }
    }
}

/// `WorldGen.randGemTile` (`WorldGen.cs:9747-9762`) — 19/20ths of the time the
/// stone stays stone, otherwise the gem for the index.
fn rand_gem_tile(rng: &mut Rng, gem: &[bool; 6]) -> u16 {
    if rng.range(0, 20) != 0 {
        return TILE_STONE;
    }
    GEM_TILES[rand_gem(rng, gem)]
}

/// `Spread.Gemmable` (`WorldGen.cs:3791-3798`).
fn gemmable(kind: u16) -> bool {
    matches!(kind, 0 | 1 | 40 | 59 | 60 | 70 | 147 | 161)
}

/// `Spread.Gem` (`WorldGen.cs:3594-3680`) — the BFS over the same open region
/// `countTiles` measured: the solid/walled cells get gem tiles (and their four
/// neighbours), the open cells get a gem wall and, half the time, an exposed
/// gem.
///
/// W4-c29 gem-fidelity: the C#'s `hashSet.Add(item)` (`:3617`) IGNORES its
/// return — a cell enqueued twice in one wave (two open neighbours both
/// pushing it before either is processed) is processed on EVERY visit, and
/// the solid branch's `randGemTile` conversions re-draw each time. The port's
/// former `if !seen.insert(item) { continue; }` dedup dropped those draws
/// (the g41 census's "gem-fidelity row"). The insert now ignores its result
/// like the C#; a re-visited open cell reads its freshly painted gem wall
/// (`has_real_wall`) and takes the solid branch, exactly as the C#'s
/// `tile.wall != 0` re-read does.
fn spread_gem(world: &mut World, rng: &mut Rng, x: i32, y: i32, gem: &[bool; 6]) {
    if !in_world(world, x, y, 0) {
        return;
    }
    let mut seen: HashSet<(i32, i32)> = HashSet::new();
    let mut next: Vec<(i32, i32)> = vec![(x, y)];
    while !next.is_empty() {
        let cur = std::mem::take(&mut next);
        for item in cur {
            if !in_world(world, item.0, item.1, 1) {
                continue;
            }
            // `hashSet.Add(item)` (`:3617`) — the return is ignored.
            seen.insert(item);
            if solid_tile(world, item.0, item.1) || has_real_wall(world, item.0, item.1) {
                let kind = world.tile(item.0, item.1).map(|t| (t.active(), t.kind));
                if let Some((active, tile_kind)) = kind {
                    if active {
                        if gemmable(tile_kind) {
                            let kind = rand_gem_tile(rng, gem);
                            if let Some(t) = world.tile_mut(item.0, item.1) {
                                t.kind = kind;
                            }
                        }
                        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                            let (nx, ny) = (item.0 + dx, item.1 + dy);
                            let ok = world
                                .tile(nx, ny)
                                .is_some_and(|t| t.active() && gemmable(t.kind));
                            if ok {
                                let kind = rand_gem_tile(rng, gem);
                                if let Some(t) = world.tile_mut(nx, ny) {
                                    t.kind = kind;
                                }
                            }
                        }
                    }
                }
            } else {
                let wall = WALL_GEM_BASE + rand_gem(rng, gem) as u16;
                if let Some(t) = world.tile_mut(item.0, item.1) {
                    t.wall = wall;
                }
                let inactive = world.tile(item.0, item.1).is_some_and(|t| !t.active());
                if inactive && rng.range(0, 2) == 0 {
                    // `PlaceTile(item.X, item.Y, 178, mute: true, forced:
                    // false, -1, randGem())` (`:3655`) — the style draws
                    // BEFORE the placement arm's anchor gate.
                    let style = rand_gem(rng, gem) as i32;
                    place_exposed_gem(world, rng, item.0, item.1, style);
                }
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let cand = (item.0 + dx, item.1 + dy);
                    if !seen.contains(&cand) {
                        next.push(cand);
                    }
                }
            }
        }
    }
}

/// `WorldGen.gemCave` (`WorldGen.cs:9719-9735`).
fn gem_cave(world: &mut World, rng: &mut Rng, x: i32, y: i32) {
    count_tiles(world, x, y, false, 300);
    let mut gem = [false; 6];
    gem[rng.range(0, 6) as usize] = true;
    for j in 0..6usize {
        if rng.range(0, 6) == 0 {
            gem[j] = true;
        }
    }
    spread_gem(world, rng, x, y, &gem);
}

// --- the spider cave ---------------------------------------------------------

/// `WorldGen.PlaceTight` (`WorldGen.cs:38303-38325`) → the stalactite.
/// Generic over the pass stream ([`WorldGenRand`]) so the spider cave's
/// `Rng` and the Stalac pass's `UnifiedRandom` share one port (W4-d106 R4).
/// The `CheckStalactite` tail (`:38320-38323`) is the caller's responsibility:
/// the spider pass's landed port skips it (its seats are pre-validated), the
/// Stalac pass runs it ([`check_stalactite`]) — keeping the spider output
/// byte-identical.
///
/// KEEP-ONE GRAFTED (research-88 §3.2 / research-96 §3, the g41 stalactite
/// 6-point FINAL; research-312 §2.1): g41's chain body wins — the validator
/// runs INSIDE `place_tight`'s tail (the C# `PlaceTight` tail,
/// `WorldGen.cs:38320-38323`), and the four Stalac call sites' explicit
/// `check_stalactite` blocks dropped at the batch-B merge (the
/// double-validation trap — keeping both doubles the `Next(3)` mismatch
/// draws). The signature is d106's generalisation over [`WorldGenRand`] so
/// the spider pass's `Rng` and the Stalac pass's `UnifiedRandom` share one
/// port (matrix-88 §3.2 point 2).
pub(crate) fn place_tight(
    world: &mut World,
    rng: &mut impl super::chest_stock::WorldGenRand,
    x: i32,
    y: i32,
    spiders: bool,
) {
    let blocked = world.tile(x, y).is_some_and(|t| t.any_shimmer());
    let larva = world.tile(x, y).is_some_and(|t| t.active() && t.kind == TILE_LARVA);
    if blocked || larva {
        return;
    }
    let prefer_small = rng.next(2) == 0;
    let variation = rng.next(3);
    place_unchecked_stalactite(world, x, y, prefer_small, variation, spiders);
    // `:38320-38323` — the post-placement support validation (W4-g41): an
    // unsupported sac pops, a mismatched style re-rolls (the chain's only
    // RNG draw, `UpdateStalagtiteStyle` `:38665`).
    if world.tile(x, y).is_some_and(|t| t.active() && t.kind == TILE_STALACTITE) {
        check_stalactite(world, rng, x, y);
    }
}

// --- the egg-sac validator chain (W4-g41) -------------------------------------
//
// `CheckStalactite` → `UpdateStalagtiteStyle` → `GetStalagtiteStyle` /
// `GetDesiredStalagtiteStyle` (`WorldGen.cs:38996-39175` / `:38643-38872`):
// the support check the C# runs after every `PlaceTight`. The kills ride the
// `destroyObject = true` flag (`:39028`/`:39066`/`:39116`) — the guarded
// `Check*` re-entries early-return, so the port's `edit::remove_tile` (the
// generation strip, no cascade) is the faithful stand-in. The C# `KillTile`
// keeps `tile.type` (`:63885-63890` clears active/frames/paint only), so the
// pair's type comparisons read the ORIGINAL kind, captured before the kills.

/// `WorldGen.InvalidTileForPilesOrSpeleothems` (`WorldGen.cs:38979-38994`):
/// an active boulder (`TileID.Sets.Boulders`, `TileID.cs:504`) under the
/// object's seat invalidates it; out-of-world / null reads false.
fn invalid_tile_for_piles_or_speleothems(world: &World, x: i32, y: i32) -> bool {
    if !in_world(world, x, y, 2) {
        return false;
    }
    world.tile(x, y).is_some_and(|t| t.active() && SET_BOULDERS.has(t.kind))
}

/// `WorldGen.GetStalagtiteStyle` (`WorldGen.cs:38823-38872`): the
/// `frameX / 54` → style decode; `None` is the C#'s `fail` (the default arm).
fn get_stalagtite_style(frame_x: i16) -> Option<i32> {
    match frame_x / 54 {
        0 => Some(7),
        1 => Some(0),
        2 => Some(11),
        3 => Some(12),
        4 => Some(1),
        5 => Some(2),
        6 => Some(3),
        7 => Some(4),
        8 => Some(5),
        9 => Some(6),
        10 => Some(8),
        11 => Some(9),
        12 => Some(10),
        _ => None,
    }
}

/// `WorldGen.GetDesiredStalagtiteStyle` (`WorldGen.cs:38720-38821`): the
/// frameY → (support side, height, top row) decode and the support-kind
/// style table. Returns `(desired_style, height, y)`; `None` is `fail`.
fn get_desired_stalagtite_style(world: &World, x: i32, j: i32) -> Option<(i32, i32, i32)> {
    let tile = world.tile(x, j)?;
    let mut y = j;
    let (side, height, support) = if tile.frame_y == 72 {
        // The small hanging sac: the support is the ceiling cell.
        (0, 1, world.tile(x, y - 1).map(|t| t.kind).unwrap_or(0))
    } else if tile.frame_y == 90 {
        // The small rising sac: the support is the floor cell.
        (1, 1, world.tile(x, y + 1).map(|t| t.kind).unwrap_or(0))
    } else if tile.frame_y >= 36 {
        // The tall rising pair (`54` is the bottom cell — step to the top).
        if tile.frame_y == 54 {
            y -= 1;
        }
        (4, 2, world.tile(x, y + 2).map(|t| t.kind).unwrap_or(0))
    } else {
        // The tall hanging pair (`18` is the bottom cell).
        if tile.frame_y == 18 {
            y -= 1;
        }
        (3, 2, world.tile(x, y - 1).map(|t| t.kind).unwrap_or(0))
    };
    // `:38763-38771` — stone/moss → style 0, with the spider override: a
    // tall hanging pair (`side == 3`) over a spider wall is the egg-sac
    // style 11.
    if support == TILE_STONE || TILE_MOSS.has(support) {
        let desired = if side == 3 && tile.wall == WALL_SPIDER { 11 } else { 0 };
        return Some((desired, height, y));
    }
    let desired = match support {
        200 => 10,
        164 => 8,
        163 => 9,
        117 | 402 | 403 => 1,
        25 | 398 | 400 => 2,
        203 | 399 | 401 => 3,
        396 | 397 => 4,
        367 => 6,
        368 => 5,
        147 | 161 => 7,
        // `:38813-38820` — the hive arm only serves the small sacs.
        225 if side == 0 || side == 1 => 12,
        _ => return None,
    };
    Some((desired, height, y))
}

/// `WorldGen.UpdateStalagtiteStyle` (`WorldGen.cs:38643-38718`): the
/// current/desired style pair; on mismatch the `Next(3)*18` variation draw
/// (`:38665`, the chain's ONLY RNG draw) plus the desired-style frame base
/// rewritten over the object's height. The `netMode == 1` early-true
/// (`:38645`) is the client skip — worldgen runs offline (netMode 0).
fn update_stalagtite_style(world: &mut World, rng: &mut impl super::chest_stock::WorldGenRand, x: i32, j: i32) -> bool {
    let Some(tile) = world.tile(x, j) else {
        return true;
    };
    let Some(style) = get_stalagtite_style(tile.frame_x) else {
        return false;
    };
    let Some((desired, height, y)) = get_desired_stalagtite_style(world, x, j) else {
        return false;
    };
    if style != desired {
        let mut num = rng.next(3) * 18;
        // `:38666-38707` — the 13-case desired-style base switch (case 7 is
        // the identity `num = num`).
        num += match desired {
            0 => 54,
            1 => 216,
            2 => 270,
            3 => 324,
            4 => 378,
            5 => 432,
            6 => 486,
            7 => 0,
            8 => 540,
            9 => 594,
            10 => 648,
            11 => 108,
            12 => 162,
            _ => 0,
        };
        for i in y..y + height {
            if let Some(t) = world.tile_mut(x, i) {
                t.frame_x = num as i16;
            }
        }
    }
    true
}

/// `WorldGen.CheckStalactite` (`WorldGen.cs:38996-39175`): the four frameY
/// arms' support validation. A failed check kills the object's cells whose
/// kind still matches the original (`destroyObject` kills — no cascade).
pub(crate) fn check_stalactite(world: &mut World, rng: &mut impl super::chest_stock::WorldGenRand, x: i32, j: i32) {
    let Some(tile) = world.tile(x, j) else {
        return;
    };
    // The C# `KillTile` keeps `tile.type`, so the live `Main.tile[x, j].type`
    // comparisons read the original kind even after the first kill.
    let orig_kind = tile.kind;
    let mut num = j;
    if tile.frame_y == 72 {
        // The small hanging sac (`:39003-39036`).
        let mut flag = !solid_tile(world, x, num - 1);
        if !flag && !update_stalagtite_style(world, rng, x, num) {
            flag = true;
        }
        if flag && world.tile(x, num).is_some_and(|t| t.kind == orig_kind) {
            let _ = crate::edit::remove_tile(world, x, num);
        }
        return;
    }
    if tile.frame_y == 90 {
        // The small rising sac (`:39037-39074`) — the boulder-seat veto.
        let mut flag = !solid_tile(world, x, num + 1);
        if !flag && invalid_tile_for_piles_or_speleothems(world, x, num + 1) {
            flag = true;
        }
        if !flag && !update_stalagtite_style(world, rng, x, num) {
            flag = true;
        }
        if flag && world.tile(x, num).is_some_and(|t| t.kind == orig_kind) {
            let _ = crate::edit::remove_tile(world, x, num);
        }
        return;
    }
    if tile.frame_y >= 36 {
        // The tall rising pair (`:39075-39128`).
        if tile.frame_y == 54 {
            num -= 1;
        }
        let mut flag = !solid_tile(world, x, num + 2);
        let top = world.tile(x, num);
        let bot = world.tile(x, num + 1);
        if !top.is_some_and(|t| t.active())
            || !bot.is_some_and(|t| t.active())
            || top.map(|t| t.kind) != bot.map(|t| t.kind)
        {
            flag = true;
        }
        if top.map(|t| t.frame_x) != bot.map(|t| t.frame_x) {
            flag = true;
        }
        if !flag && invalid_tile_for_piles_or_speleothems(world, x, num + 2) {
            flag = true;
        }
        if !flag && !update_stalagtite_style(world, rng, x, num) {
            flag = true;
        }
        if flag {
            if world.tile(x, num).is_some_and(|t| t.kind == orig_kind) {
                let _ = crate::edit::remove_tile(world, x, num);
            }
            if world.tile(x, num + 1).is_some_and(|t| t.kind == orig_kind) {
                let _ = crate::edit::remove_tile(world, x, num + 1);
            }
        }
        return;
    }
    // The tall hanging pair (`:39129-39174`) — the egg sac's own arm.
    if tile.frame_y == 18 {
        num -= 1;
    }
    let mut flag = !solid_tile(world, x, num - 1);
    let top = world.tile(x, num);
    let bot = world.tile(x, num + 1);
    if !top.is_some_and(|t| t.active())
        || !bot.is_some_and(|t| t.active())
        || top.map(|t| t.kind) != bot.map(|t| t.kind)
    {
        flag = true;
    }
    if top.map(|t| t.frame_x) != bot.map(|t| t.frame_x) {
        flag = true;
    }
    if !flag && !update_stalagtite_style(world, rng, x, num) {
        flag = true;
    }
    if flag {
        if world.tile(x, num).is_some_and(|t| t.kind == orig_kind) {
            let _ = crate::edit::remove_tile(world, x, num);
        }
        if world.tile(x, num + 1).is_some_and(|t| t.kind == orig_kind) {
            let _ = crate::edit::remove_tile(world, x, num + 1);
        }
    }
}

/// `WorldGen.PlaceUncheckedStalactite` (`WorldGen.cs:38327-38641`) — the
/// support-type frame table. The port keeps the arms the port's world can
/// actually produce (spiders, the ice family, stone/moss, hive, the sandstone
/// pair, granite and marble); the paint copy is the port's `frame_x` write.
pub(crate) fn place_unchecked_stalactite(
    world: &mut World,
    x: i32,
    y: i32,
    prefer_small: bool,
    variation: i32,
    spiders: bool,
) {
    let variation = variation.clamp(0, 2);
    let hanging = solid_tile(world, x, y - 1)
        && !world.tile(x, y).is_some_and(|t| t.active())
        && !world.tile(x, y + 1).is_some_and(|t| t.active());
    if hanging {
        let support = world.tile(x, y - 1).map(|t| t.kind).unwrap_or(0);
        // `Main.tileMoss` (`Main.cs:7007-7047`) — the generated set carries
        // the rainbow moss 627 (`:7031`) beside the five + neon + lava moss.
        let is_moss = TILE_MOSS.has(support);
        if spiders {
            // `:38334-38350` — the spider stalactite, always two tiles tall.
            let frame = 108 + variation * 18;
            write_stalactite(world, x, y, frame, 0);
            write_stalactite(world, x, y + 1, frame, 18);
            return;
        }
        // The independent C# `if` arms (`:38351/:38380/:38409/:38419/
        // :38448/:38477`) — the granite 368 (`:38448-38476`) and marble 367
        // (`:38477-38505`) ceiling arms landed with the marble/granite decor
        // residual (W4-g40); a support kind matches at most one arm, so the
        // else-chain is equivalent.
        let frame = if matches!(support, TILE_SNOW | TILE_ICE | 163 | 164 | 200) {
            variation * 18
        } else if support == TILE_HIVE {
            // `:38409` — the hive stalactite is always the small one.
            write_stalactite(world, x, y, 162 + variation * 18, 72);
            return;
        } else if is_moss
            || matches!(support, TILE_STONE | TILE_PEARLSTONE | TILE_EBONSTONE | TILE_CRIMSTONE)
        {
            54 + variation * 18
        } else if matches!(support, 396 | 397) {
            378 + variation * 18
        } else if support == TILE_GRANITE {
            432 + variation * 18
        } else if support == TILE_MARBLE {
            486 + variation * 18
        } else {
            return;
        };
        if prefer_small {
            write_stalactite(world, x, y, frame, 72);
        } else {
            write_stalactite(world, x, y, frame, 0);
            write_stalactite(world, x, y + 1, frame, 18);
        }
        return;
    }
    // The rising branch (`:38507+`) — the stalactite on the floor.
    let rising = !spiders
        && solid_tile(world, x, y + 1)
        && !world.tile(x, y).is_some_and(|t| t.active())
        && !world.tile(x, y - 1).is_some_and(|t| t.active());
    if rising {
        let support = world.tile(x, y + 1).map(|t| t.kind).unwrap_or(0);
        if support == TILE_HIVE {
            // `:38543-38552` — the rising hive arm is always the small one
            // (frameY 90, no tall variant) (W4-g41).
            write_stalactite(world, x, y, 162 + variation * 18, 90);
            return;
        }
        // `Main.tileMoss` (`Main.cs:7007-7047`) — the generated set carries
        // the rainbow moss 627 (`:7031`) beside the five + neon + lava moss.
        let is_moss = TILE_MOSS.has(support);
        // The independent C# `if` arms (`:38514/:38553/:38582/:38611`) —
        // a support kind matches at most one, so the else-chain is
        // equivalent.
        let frame = if matches!(support, 396 | 397) {
            378 + variation * 18
        } else if support == TILE_GRANITE {
            432 + variation * 18
        } else if support == TILE_MARBLE {
            486 + variation * 18
        } else if is_moss || matches!(support, TILE_STONE | TILE_PEARLSTONE | TILE_EBONSTONE | TILE_CRIMSTONE) {
            54 + variation * 18
        } else {
            return;
        };
        if prefer_small {
            write_stalactite(world, x, y, frame, 90);
        } else {
            write_stalactite(world, x, y - 1, frame, 36);
            write_stalactite(world, x, y, frame, 54);
        }
    }
}

/// One stalactite cell: the C# writes `type`, `active(true)`, `ClearSlope()`
/// and the frame pair (the paint copy is the port's frame write).
fn write_stalactite(world: &mut World, x: i32, y: i32, frame_x: i32, frame_y: i32) {
    if !in_world(world, x, y, 1) {
        return;
    }
    if let Some(t) = world.tile_mut(x, y) {
        t.kind = TILE_STALACTITE;
        t.flags |= FLAG_ACTIVE;
        t.frame_x = frame_x as i16;
        t.frame_y = frame_y as i16;
        t.set_slope(0);
    }
}

// --- the Stalac pass (W4-d106 R4) ---------------------------------------------

/// `WorldGen.oceanDepths` (`WorldGen.cs:10963-10973`) — the ±380 beach
/// margins above the ocean level. Shared with `cleanup::final_cleanup`
/// (W4-d106 R6's sand-fall gate).
pub(crate) fn ocean_depths(world: &World, x: i32, y: i32, ocean_level: f64) -> bool {
    if y as f64 > ocean_level {
        return false;
    }
    x < 380 || x > world.width - 380
}

/// The support-type test the Stalac slope-clear arms read (`WorldGen.cs:22080`
/// / `:22084`): stone 1 / snow 147 / ice 161 / ebonstone 25 / crimstone 203,
/// or any `tileStone` / `tileMoss` kind. Reads the tile TYPE regardless of
/// active state (the C# does).
fn speleothem_support(kind: u16) -> bool {
    matches!(kind, 1 | 147 | 161 | 25 | 203) || td::TILE_STONE.has(kind) || td::TILE_MOSS.has(kind)
}

/// `Stalac` (`GenPassNameID.SpeleothemsAndGemTrees`, `WorldGen.cs:22038-22120`)
/// — W4-d106 R4. The gem-tree scatter + the stalactite/stalagmite placement
/// over the underground, plus the surface snow/ice/evil speleothems. Runs on
/// one name-seeded [`UnifiedRandom`] (the gem-tree grower
/// `growth::try_growing_tree_by_type` and `PlaceTight` share vanilla's single
/// `genRand`). Draws, in C# order per (i, j): the gem-tree gate `Next(5)`
/// (short-circuited by the anniversary/drunk flags) then `Next(7)` on a pass;
/// the stalactite gate `Next(5)` (after the `oceanDepths`/active vetoes); the
/// surface arms' three `Next(5)` rolls.
pub(crate) fn pass_stalac(
    world: &mut World,
    rng: &mut crate::UnifiedRandom,
    world_surface: f64,
    rock_layer: f64,
    opts: &super::GenOptions,
) {
    let anniversary = opts.flags.contains(crate::seeds::SeedFlags::TENTH_ANNIVERSARY);
    let drunk = opts.flags.contains(crate::seeds::SeedFlags::DRUNK);
    let ocean_level = (world_surface + rock_layer) / 2.0 + 40.0;
    let (width, height) = (world.width, world.height);
    let surface = world_surface as i32;
    // `:22043-22118` — i from 20 to maxTilesX-20.
    for i in 20..width - 20 {
        // `:22046-22090` — the underground band, j from worldSurface to
        // maxTilesY-20.
        for j in surface..height - 20 {
            // `:22048-22077` — the gem-tree arm: the anniversary/drunk flags
            // short-circuit the Next(5); the above cell must hold no liquid.
            let above_dry = world.tile(i, j - 1).is_some_and(|t| t.liquid == 0);
            if (anniversary || drunk || rng.next_max(5) == 0) && above_dry {
                let num2 = rng.next_max(7);
                // `:22052-22075` — the gem-tree kind table (583..589).
                let tree_tile_type: u16 = 583 + num2 as u16;
                crate::growth::try_growing_tree_by_type(world, tree_tile_type, i, j, rng);
            }
            // `:22078-22089` — the stalactite arm: not ocean depths, the cell
            // inactive, 1-in-5.
            let cell_inactive = world.tile(i, j).is_some_and(|t| !t.active());
            if !ocean_depths(world, i, j, ocean_level) && cell_inactive && rng.next_max(5) == 0 {
                // `:22080-22083` — flatten the support above when it is a
                // speleothem support and both cells are inactive.
                let above_kind = world.tile(i, j - 1).map_or(0, |t| t.kind);
                if speleothem_support(above_kind)
                    && world.tile(i, j).is_some_and(|t| !t.active())
                    && world.tile(i, j + 1).is_some_and(|t| !t.active())
                {
                    if let Some(t) = world.tile_mut(i, j - 1) {
                        t.set_slope(0);
                    }
                }
                // `:22084-22087` — the same for the support below.
                let below_kind = world.tile(i, j + 1).map_or(0, |t| t.kind);
                if speleothem_support(below_kind)
                    && world.tile(i, j).is_some_and(|t| !t.active())
                    && world.tile(i, j - 1).is_some_and(|t| !t.active())
                {
                    if let Some(t) = world.tile_mut(i, j + 1) {
                        t.set_slope(0);
                    }
                }
                // `:22088` — PlaceTight (the support check rides inside).
                place_tight(world, rng, i, j, false);
            }
        }
        // `:22091-22117` — the surface band, k from 5 to worldSurface.
        for k in 5..surface {
            // `:22093-22100` — the snow/ice ceiling arm.
            let above = world.tile(i, k - 1).copied().unwrap_or_default();
            if above.active() && matches!(above.kind, 147 | 161) && rng.next_max(5) == 0 {
                if world.tile(i, k).is_some_and(|t| !t.active())
                    && world.tile(i, k + 1).is_some_and(|t| !t.active())
                {
                    if let Some(t) = world.tile_mut(i, k - 1) {
                        t.set_slope(0);
                    }
                }
                place_tight(world, rng, i, k, false);
            }
            // `:22101-22108` — the ebonstone/crimstone ceiling arm.
            let above = world.tile(i, k - 1).copied().unwrap_or_default();
            if above.active() && matches!(above.kind, 25 | 203) && rng.next_max(5) == 0 {
                if world.tile(i, k).is_some_and(|t| !t.active())
                    && world.tile(i, k + 1).is_some_and(|t| !t.active())
                {
                    if let Some(t) = world.tile_mut(i, k - 1) {
                        t.set_slope(0);
                    }
                }
                place_tight(world, rng, i, k, false);
            }
            // `:22109-22116` — the ebonstone/crimstone floor arm.
            let below = world.tile(i, k + 1).copied().unwrap_or_default();
            if below.active() && matches!(below.kind, 25 | 203) && rng.next_max(5) == 0 {
                if world.tile(i, k).is_some_and(|t| !t.active())
                    && world.tile(i, k - 1).is_some_and(|t| !t.active())
                {
                    if let Some(t) = world.tile_mut(i, k + 1) {
                        t.set_slope(0);
                    }
                }
                place_tight(world, rng, i, k, false);
            }
        }
    }
}

/// `WorldGen.PlacePot` (`WorldGen.cs:54066-54114`) — the 2x2 seat validation
/// plus the style frame pair. Shared with the pyramid chamber's pot row
/// (W4-c33).
pub(crate) fn place_pot(world: &mut World, rng: &mut Rng, x: i32, y: i32, style: i32) -> bool {
    // `:54069-54072` — the remix centre veto (W4-g41 FINAL): the 0.48-0.52
    // x band below `maxTilesY - 220` refuses the pot. The C# sets
    // `flag = false` and keeps scanning, but a false flag returns before
    // the `Next(3)` variant draw — the early return is write- AND
    // draw-identical (the seat loop allocates null tiles only, a no-op on
    // the port's dense world).
    if world.seeds.contains(crate::seeds::SeedFlags::REMIX)
        && (x as f64) > world.width as f64 * 0.48
        && (x as f64) < world.width as f64 * 0.52
        && y > world.height - 220
    {
        return false;
    }
    let mut ok = true;
    for i in x..x + 2 {
        for j in (y - 1)..(y + 1) {
            if world.tile(i, j).is_some_and(|t| t.active()) {
                ok = false;
            }
        }
        let below = world.tile(i, y + 1);
        if below.is_none_or(|t| {
            !t.active() || t.half_brick() || t.slope() != 0 || !TILE_SOLID.has(t.kind)
        }) {
            ok = false;
        }
    }
    if !ok {
        return false;
    }
    let variant = rng.range(0, 3) * 36;
    for k in 0..2 {
        for l in -1..1 {
            let (px, py) = (x + k, y + l);
            if let Some(t) = world.tile_mut(px, py) {
                t.kind = TILE_POT;
                t.flags |= FLAG_ACTIVE;
                t.frame_x = (k * 18 + variant) as i16;
                t.frame_y = ((l + 1) * 18 + style * 36) as i16;
                // `:54108` — `halfBrick(halfBrick: false)` (W4-g41).
                t.flags &= !FLAG_HALF_BRICK;
            }
        }
    }
    true
}

/// `WorldGen.PlaceSmallPile` (`WorldGen.cs:46815-46873`). Shared with the
/// pyramid chamber's pile scatter (W4-c33).
pub(crate) fn place_small_pile(
    world: &mut World,
    x: i32,
    y: i32,
    pile_style: i32,
    pile_size: i32,
) -> bool {
    let frame_y = (pile_size * 18) as i16;
    let frame_x = (pile_style * 18) as i16;
    // `Tile.anyLava` (`Tile.cs:410-415`) — a positive amount AND the lava
    // kind (W4-g41: the old read tested the kind alone).
    if world
        .tile(x, y)
        .is_some_and(|t| t.liquid > 0 && t.liquid_kind() == crate::tile::LiquidKind::Lava)
    {
        return false;
    }
    if pile_size == 1 {
        let frame_x = (pile_style * 36) as i16;
        let seat = solid_tile2(world, x, y + 1)
            && solid_tile2(world, x + 1, y + 1)
            && !world.tile(x, y).is_some_and(|t| t.active())
            && !world.tile(x + 1, y).is_some_and(|t| t.active());
        if !seat {
            return false;
        }
        // `:46847-46850` — the type-185 boulder-seat veto (W4-g41): a
        // boulder under either seat cell refuses the pile.
        if invalid_tile_for_piles_or_speleothems(world, x, y + 1)
            || invalid_tile_for_piles_or_speleothems(world, x + 1, y + 1)
        {
            return false;
        }
        for (i, fx) in [(x, frame_x), (x + 1, frame_x + 18)] {
            if let Some(t) = world.tile_mut(i, y) {
                t.kind = TILE_SMALL_PILES;
                t.flags |= FLAG_ACTIVE;
                t.frame_x = fx;
                t.frame_y = frame_y;
            }
        }
        true
    } else if solid_tile2(world, x, y + 1) && !world.tile(x, y).is_some_and(|t| t.active()) {
        if let Some(t) = world.tile_mut(x, y) {
            t.kind = TILE_SMALL_PILES;
            t.flags |= FLAG_ACTIVE;
            t.frame_x = frame_x;
            t.frame_y = frame_y;
        }
        true
    } else {
        false
    }
}

/// A `PlaceTile(x, y, kind, mute: true, forced: false, style)`-shaped write:
/// the port's single-tile idiom with the object style in `frame_x`.
fn place_framed(world: &mut World, x: i32, y: i32, kind: u16, frame_x: i32) -> bool {
    if !in_world(world, x, y, 1) {
        return false;
    }
    if world.tile(x, y).is_some_and(|t| t.active()) {
        return false;
    }
    if let Some(t) = world.tile_mut(x, y) {
        t.kind = kind;
        t.flags |= FLAG_ACTIVE;
        t.frame_x = frame_x as i16;
        t.frame_y = 0;
    }
    true
}

/// `WorldGen.ValidAnchorForMultiDirectionalTile` (`WorldGen.cs:60297-60312`):
/// the four-direction anchor predicate — a non-boulder, non-door tile whose
/// OPPOSITE face is solid under the slope-allowing reading (direction 0 =
/// the tile ABOVE, its bottom face via `SolidTileAllowTopSlope`; 1 = below,
/// `SolidTileAllowBottomSlope`; 2 = left, `SolidTileAllowLeftSlope`; 3 =
/// right, `SolidTileAllowRightSlope`). The C#'s chained `&&` of negated
/// disjunctions reduces to exactly that per-direction test.
fn valid_anchor_for_multi_directional_tile(world: &World, x: i32, y: i32, direction: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if crate::tiledata::SET_BOULDERS.has(t.kind) || t.kind == 10 {
        return false;
    }
    match direction {
        0 => crate::edit::solid_tile_allow_top_slope(world, x, y),
        1 => crate::edit::solid_tile_allow_bottom_slope(world, x, y),
        2 => crate::edit::solid_tile_allow_left_slope(world, x, y),
        3 => crate::edit::solid_tile_allow_right_slope(world, x, y),
        _ => false,
    }
}

/// `WorldGen.CheckAndAdjustMultiDirectionalTile` (`WorldGen.cs:60314-60366`):
/// the four anchor captures (above → `num2`, below → `num`, left → `num3`,
/// right → `num4`), then the priority pick below(0) → left(2) → right(3) →
/// above(1), each requiring the captured kind to be solid and NOT solid-top.
/// Returns the frame direction, or `None` when no anchor qualifies.
fn check_and_adjust_multi_directional_tile(world: &World, x: i32, y: i32) -> Option<i32> {
    if !in_world(world, x, y, 2) {
        return None;
    }
    let mut below: i32 = -1;
    let mut above: i32 = -1;
    let mut left: i32 = -1;
    let mut right: i32 = -1;
    if valid_anchor_for_multi_directional_tile(world, x, y - 1, 0) {
        above = world.tile(x, y - 1).map(|t| t.kind as i32).unwrap_or(-1);
    }
    if valid_anchor_for_multi_directional_tile(world, x, y + 1, 1) {
        below = world.tile(x, y + 1).map(|t| t.kind as i32).unwrap_or(-1);
    }
    if valid_anchor_for_multi_directional_tile(world, x - 1, y, 2) {
        left = world.tile(x - 1, y).map(|t| t.kind as i32).unwrap_or(-1);
    }
    if valid_anchor_for_multi_directional_tile(world, x + 1, y, 3) {
        right = world.tile(x + 1, y).map(|t| t.kind as i32).unwrap_or(-1);
    }
    let solid_not_top = |kind: i32| {
        kind >= 0
            && TILE_SOLID.has(kind as u16)
            && !TILE_SOLID_TOP.has(kind as u16)
    };
    if solid_not_top(below) {
        return Some(0);
    }
    if solid_not_top(left) {
        return Some(2);
    }
    if solid_not_top(right) {
        return Some(3);
    }
    if solid_not_top(above) {
        return Some(1);
    }
    None
}

/// `WorldGen.PlaceTile`'s 178 arm (`WorldGen.cs:59691-59701`): the
/// multi-directional anchor gate, then `frameX = style * 18`,
/// `frameY = genRand.Next(3) * 18` — the frame-row draw fires ONLY when an
/// anchor qualified (the port's former `place_framed` write carried neither
/// the gate nor the draw). `SquareTileFrame` rides the port's post-pipeline
/// framing seam (the M5-STRUCT-5 idiom).
fn place_exposed_gem(world: &mut World, rng: &mut Rng, x: i32, y: i32, style: i32) -> bool {
    if !in_world(world, x, y, 1) {
        return false;
    }
    if world.tile(x, y).is_some_and(|t| t.active()) {
        return false;
    }
    if check_and_adjust_multi_directional_tile(world, x, y).is_none() {
        return false;
    }
    let row = rng.range(0, 3) * 18;
    if let Some(t) = world.tile_mut(x, y) {
        t.kind = TILE_EXPOSED_GEMS;
        t.flags |= FLAG_ACTIVE;
        t.frame_x = (style * 18) as i16;
        t.frame_y = row as i16;
    }
    true
}

/// `Spread.Spider` (`WorldGen.cs:3682-3789`) — the BFS paints wall 62 over the
/// region and decorates the open cells: the buried chest (1/15 of the floored
/// cells), the pot, the stalactite on a ceiling, the large piles and the
/// small piles on a floor.
fn spread_spider(
    world: &mut World,
    rng: &mut Rng,
    x: i32,
    y: i32,
    hell_top: f64,
    mut stock: Option<&mut super::chest_stock::ChestStockHook<'_>>,
) {
    if !in_world(world, x, y, 0) {
        return;
    }
    let hell_top = hell_top as f64;
    let world_surface = world.world_surface;
    let mut seen: HashSet<(i32, i32)> = HashSet::new();
    let mut next: Vec<(i32, i32)> = vec![(x, y)];
    while !next.is_empty() {
        let cur = std::mem::take(&mut next);
        for item in cur {
            if !in_world(world, item.0, item.1, 1) {
                continue;
            }
            if item.1 as f64 >= hell_top - rng.range(0, 5) as f64 {
                continue;
            }
            if (item.1 as f64) < world_surface + rng.range(0, 5) as f64 {
                continue;
            }
            if !seen.insert(item) {
                continue;
            }
            if solid_tile(world, item.0, item.1) || has_real_wall(world, item.0, item.1) {
                let active = world.tile(item.0, item.1).is_some_and(|t| t.active());
                let wall = world.tile(item.0, item.1).map(|t| t.wall).unwrap_or(1);
                if active && BASELINE_WALLS.contains(&wall) {
                    if let Some(t) = world.tile_mut(item.0, item.1) {
                        t.wall = WALL_SPIDER;
                    }
                }
                continue;
            }
            if let Some(t) = world.tile_mut(item.0, item.1) {
                t.wall = WALL_SPIDER;
            }
            let active = world.tile(item.0, item.1).is_some_and(|t| t.active());
            if !active {
                if let Some(t) = world.tile_mut(item.0, item.1) {
                    t.liquid = 0;
                    t.clear_liquid_kind();
                }
                // The floor decor: the chest at 1/15, otherwise a pot.
                let floored = solid_tile(world, item.0, item.1 + 1);
                if floored && rng.range(0, 3) == 0 {
                    if rng.range(0, 15) == 0 {
                        // `AddBuriedChest(x, y, 939, notNearOtherChests: true,
                        // 15, trySlope: false, 0)` (`:17412-17414`) — the web
                        // chest's main item 939, style 15. S1-A-4: the stock
                        // rides inline.
                        let style = super::structures::resolved_chest_style(
                            15,
                            939,
                            item.1,
                            world.world_surface,
                        );
                        if place_chest(world, item.0, item.1, style) {
                            super::chest_stock::stock_placed_chest(
                                world,
                                stock.as_deref_mut(),
                                rng,
                                item.0,
                                item.1,
                                style,
                                21,
                                939,
                                true,
                            );
                        }
                    } else {
                        let style = rng.range(19, 21);
                        place_pot(world, rng, item.0, item.1, style);
                    }
                }
                let active = world.tile(item.0, item.1).is_some_and(|t| t.active());
                if !active {
                    if solid_tile(world, item.0, item.1 - 1) && rng.range(0, 3) == 0 {
                        place_tight(world, rng, item.0, item.1, true);
                    } else if solid_tile(world, item.0, item.1 + 1) {
                        let style = 9 + rng.range(0, 5);
                        place_framed(world, item.0, item.1, TILE_LARGE_PILES, style * 18);
                        if rng.range(0, 3) == 0 {
                            let active = world.tile(item.0, item.1).is_some_and(|t| t.active());
                            if !active {
                                place_small_pile(world, item.0, item.1, 34 + rng.range(0, 4), 1);
                            }
                            let active = world.tile(item.0, item.1).is_some_and(|t| t.active());
                            if !active {
                                place_small_pile(world, item.0, item.1, 48 + rng.range(0, 6), 0);
                            }
                        }
                    }
                }
            }
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let cand = (item.0 + dx, item.1 + dy);
                if !seen.contains(&cand) {
                    next.push(cand);
                }
            }
        }
    }
}

// --- the moss ----------------------------------------------------------------

/// `WorldGen.randMoss` (`WorldGen.cs:9764-9781`) — the neon pick and the three
/// distinct region moss indices.
fn rand_moss(rng: &mut Rng) -> ([usize; 3], usize) {
    let neon = rng.range(0, 4) as usize;
    let mut moss_type = [0usize; 3];
    moss_type[0] = rng.range(0, 5) as usize;
    moss_type[1] = rng.range(0, 5) as usize;
    while moss_type[1] == moss_type[0] {
        moss_type[1] = rng.range(0, 5) as usize;
    }
    moss_type[2] = rng.range(0, 5) as usize;
    while moss_type[2] == moss_type[0] || moss_type[2] == moss_type[1] {
        moss_type[2] = rng.range(0, 5) as usize;
    }
    (moss_type, neon)
}

/// `WorldGen.setMoss` (`WorldGen.cs:9878-9884`) — the x-banded moss pair the
/// C# parks in `GenVars` (`mossWall`/`mossTile`).
fn set_moss(world: &World, x: i32, moss_type: &[usize; 3]) -> (u16, u16) {
    let region = if (x as f64) < world.width as f64 * 0.334 {
        0
    } else if (x as f64) < world.width as f64 * 0.667 {
        1
    } else {
        2
    };
    let kind = moss_type[region];
    ((WALL_MOSS_BASE + kind as u16), MOSS_TILES[kind])
}

/// `Spread.Moss` (`WorldGen.cs:3526-3592`) — the BFS paints the cave moss wall
/// over the region and converts the stone it can see to the moss tile.
fn spread_moss(world: &mut World, x: i32, y: i32, moss_wall: u16, moss_tile: u16) {
    if !in_world(world, x, y, 0) {
        return;
    }
    let mut seen: HashSet<(i32, i32)> = HashSet::new();
    let mut next: Vec<(i32, i32)> = vec![(x, y)];
    while !next.is_empty() {
        let cur = std::mem::take(&mut next);
        for item in cur {
            if !in_world(world, item.0, item.1, 1) {
                continue;
            }
            if !seen.insert(item) {
                continue;
            }
            if solid_tile(world, item.0, item.1) || has_real_wall(world, item.0, item.1) {
                if let Some(t) = world.tile_mut(item.0, item.1) {
                    if t.active() {
                        if BASELINE_WALLS.contains(&t.wall) {
                            t.wall = moss_wall;
                        }
                        if t.kind == TILE_STONE {
                            t.kind = moss_tile;
                        }
                    }
                }
                continue;
            }
            if let Some(t) = world.tile_mut(item.0, item.1) {
                t.wall = moss_wall;
            }
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let cand = (item.0 + dx, item.1 + dy);
                if !seen.contains(&cand) {
                    next.push(cand);
                }
            }
        }
    }
}

/// The neon-moss blob scale — `WorldGen.cs:9801-9805`'s
/// `num3 = maxTilesX / 4200.0` with the for-the-worthy `*= 1.5`. Both the
/// ellipse radius (`num`, `:9806`) and the step count (`num2`, `:9807`) ride
/// it, so the for-the-worthy blob is wider and longer. Factored out of the
/// walker so the multiplier is testable.
fn neon_moss_scale(world: &World) -> f64 {
    let mut scale = world.width as f64 / 4200.0;
    if world
        .seeds
        .contains(crate::seeds::SeedFlags::FOR_THE_WORTHY)
    {
        scale *= 1.5;
    }
    scale
}

/// `WorldGen.neonMossBiome` (`WorldGen.cs:9783-9876`) — the downward ellipse
/// walk painting the neon moss over the stone it crosses.
///
/// W4-c29 classic-path fidelity fixes over the M5-STRUCT-5 draft:
/// - the rock-line clamp rides `num6` (the ellipse's TOP edge, `:9843-9850`),
///   not the bottom edge — the draft clamped `ymax`, inverting the gate;
/// - the clamp SETS the drift to ±5.0 (`:9848`, `:9856`), the draft nudged
///   ±2.0 — a different walker trajectory after the first clamped step;
/// - the step budget `num2` stays a scaled double (`:9800-9807` — the
///   `while (num2 > 0.0)` runs `ceil` iterations on a sub-4200 world), the
///   draft truncated it before the loop;
/// - the paint loops are EXCLUSIVE on the high edges (`k < num5`, `l < num7`,
///   `:9860-9862`), the draft painted the inclusive rim row/column.
/// The remix arm (`Main.remixWorld`, `:9832-9842`) clamps to the surface
/// instead of the rock line — not ported (the seed-fidelity rows own the
/// alternative draw schedules); a classic world never reads it.
#[allow(clippy::too_many_arguments)]
fn neon_moss_biome(
    world: &mut World,
    rng: &mut Rng,
    i: i32,
    j: i32,
    max_y: i32,
    neon_moss_type: u16,
) {
    let mut pos = (i as f64, j as f64);
    let mut vel = (rng.next_f64() * 4.0 - 2.0, rng.next_f64() * 4.0 - 2.0);
    if vel.0 == 0.0 {
        vel.0 = 1.0;
    }
    while (vel.0 * vel.0 + vel.1 * vel.1).sqrt() < 4.0 {
        vel = (vel.0 * 1.5, vel.1 * 1.5);
    }
    let mut radius = rng.range(60, 80) as f64;
    let mut steps = rng.range(30, 40) as f64;
    // `double num3 = maxTilesX / 4200.0; if (WorldGen.getGoodWorldGen) num3 *=
    // 1.5;` (`:9801-9805`) — the for-the-worthy neon moss blob is 1.5x wider
    // and takes 1.5x as many steps (`num`/`num2` are both scaled by `num3`).
    let scale = neon_moss_scale(world);
    radius *= scale;
    steps *= scale;
    let rock_layer = world.rock_layer;
    while steps > 0.0 {
        radius *= 0.98;
        steps -= 1.0;
        let (x0, y0) = (pos.0 - radius, pos.1 - radius);
        let (x1, y1) = (pos.0 + radius, pos.1 + radius);
        let xmin = (x0 as i32).max(1);
        let xmax = (x1 as i32).min(world.width - 1);
        let mut ymin = (y0 as i32).max(1);
        let ymax = (y1 as i32).min(world.height - 1);
        // `else if ((double)num6 < Main.rockLayer)` (`:9843-9850`) — the TOP
        // edge is forced under the rock line and the drift is SET to 5.0.
        if (ymin as f64) < rock_layer {
            ymin = rock_layer as i32;
            if vel.1 < 5.0 {
                vel.1 = 5.0;
            }
        }
        // `if (num7 > maxY)` (`:9851-9858`) — the BOTTOM edge caps at maxY
        // and the drift is SET to -5.0.
        let mut ymax = ymax;
        if ymax > max_y {
            ymax = max_y;
            if vel.1 > -5.0 {
                vel.1 = -5.0;
            }
        }
        let r = radius * (1.0 + rng.next_f64() * 0.4 - 0.2);
        for k in xmin..xmax {
            for l in ymin..ymax {
                if ((k as f64 - pos.0).hypot(l as f64 - pos.1)) < r * 0.8
                    && world.tile(k, l).is_some_and(|t| t.active() && t.kind == TILE_STONE)
                    && (world.tile(k - 1, l).is_none_or(|t| !t.active())
                        || world.tile(k + 1, l).is_none_or(|t| !t.active())
                        || world.tile(k, l - 1).is_none_or(|t| !t.active())
                        || world.tile(k, l + 1).is_none_or(|t| !t.active()))
                {
                    // `:9866` — the seed is the tile to the LEFT of the
                    // qualifying tile.
                    spread_grass(world, k - 1, l, TILE_STONE, neon_moss_type);
                }
            }
        }
        pos = (pos.0 + vel.0, pos.1 + vel.1);
        vel = (vel.0 + rng.next_f64() * 4.0 - 2.0, vel.1 + rng.next_f64() * 4.0 - 2.0);
        vel = (
            vel.0.clamp(-10.0, 10.0),
            vel.1.clamp(-10.0, 10.0),
        );
    }
}

// --- the pass drivers --------------------------------------------------------

/// The spider-cave attempt count — `WorldGen.cs:17365`'s
/// `num2 = (int)(maxTilesX * 0.005)` with the for-the-worthy tripling
/// (`:17378`). Factored out of the pass so the multiplier is testable.
fn spider_attempts(world: &World) -> i32 {
    let mut attempts = (world.width as f64 * 0.005) as i32;
    // `else if (WorldGen.getGoodWorldGen) num2 *= 3;` (`:17378`) — the last
    // arm of the C#'s if/else-if chain; the not-the-bees + for-the-worthy
    // combo takes the FIRST branch (`num2 *= 2` with the 1000/5000 bounds,
    // not ported), hence the guard. A classic world reads no flag and keeps
    // the `maxTilesX * 0.005` count.
    if !world.seeds.not_the_bees_and_for_the_worthy_no_celebration()
        && world
            .seeds
            .contains(crate::seeds::SeedFlags::FOR_THE_WORTHY)
    {
        attempts *= 3;
    }
    attempts
}

/// The neon-moss anchor `num4` (`WorldGen.cs:17494-17516`): the C#'s
/// `remix → tenthAnniversary → getGoodWorldGen → !drunk` x-band chain. Only
/// the for-the-worthy arm is ported — it re-rolls until `num4` leaves the
/// 0.42-0.48 band, so the biome avoids the world's centre strip. The remix
/// and anniversary arms are alternative draw schedules (out of scope), and
/// the classic `!drunk` 0.38-0.62 re-roll is deliberately NOT ported: the
/// port's classic arm draws `num4` once (adding the re-roll would shift a
/// classic world's draws), which is exactly what the C# drunk arm does — so a
/// drunk world needs no extra draw.
fn neon_moss_x(rng: &mut Rng, width: i32, flags: crate::seeds::SeedFlags) -> i32 {
    let mut x = rng.range(100, width - 100);
    if !flags.contains(crate::seeds::SeedFlags::REMIX)
        && !flags.contains(crate::seeds::SeedFlags::TENTH_ANNIVERSARY)
        && flags.contains(crate::seeds::SeedFlags::FOR_THE_WORTHY)
    {
        while (x as f64) > width as f64 * 0.42 && (x as f64) < width as f64 * 0.48 {
            x = rng.range(100, width - 100);
        }
    }
    x
}

/// M5-SEED-2 `SecretSeed.Variations.noSpiderCavesActuallyNoSpiderCaves`
/// (`WorldGen.cs:270-278`): the `noSpiderCaves` world actually loses its
/// spider caves when the secret-seed list is short:
///
/// ```text
/// if (SecretSeed.noSpiderCaves.Enabled) return SecretSeed.activeSecretSeedCount < 4;
/// return false;
/// ```
///
/// The pass body is gated on the negation (`WorldGen.cs:17358`: `if
/// (!Skyblock.denyAllGeneration && !SecretSeed.Variations.
/// noSpiderCavesActuallyNoSpiderCaves)`), so a long list (or no
/// `noSpiderCaves` at all — the other agent's `ILied` wall sweep,
/// `:280-289`) keeps the pass. `activeSecretSeedCount` counts both registries,
/// hence the classic `SeedFlags` fold through
/// [`crate::seeds::active_secret_seed_count`].
fn no_spider_caves_actually_no_spider_caves(world: &World) -> bool {
    if !world
        .micro_seeds
        .contains(crate::seeds::MicroSeedFlags::NO_SPIDER_CAVES)
    {
        return false;
    }
    crate::seeds::active_secret_seed_count(world.micro_seeds, world.seeds) < 4
}

/// `Spider Caves` (`WorldGen.cs:17356-17427`).
pub(crate) fn pass_spider_caves(
    world: &mut World,
    rng: &mut Rng,
    rock_layer: f64,
    world_surface: f64,
    hell_top: f64,
    mut stock: Option<&mut super::chest_stock::ChestStockHook<'_>>,
) -> u32 {
    // `WorldGen.cs:17358`: the pass is skipped whole under
    // `Variations.noSpiderCavesActuallyNoSpiderCaves` — the M5-SEED-2 seed
    // was a lie for short secret-seed lists. A classic world reads no micro
    // flag and runs exactly as before.
    if no_spider_caves_actually_no_spider_caves(world) {
        return 0;
    }
    let mut placed = 0u32;
    let max_spider_count = 3500;
    let retries = world.width / 2;
    let attempts = spider_attempts(world);
    let (min_count, max_count) = (500, max_spider_count);
    let remix = world.seeds.contains(crate::seeds::SeedFlags::REMIX);
    for _ in 0..attempts {
        let mut tries = 0i32;
        let mut x = rng.range(200, world.width - 200);
        let mut y = if remix {
            // `:17392-17395` — the remix first draw rides the surface→rock
            // band (W4-g41; a classic world reads no flag and keeps the
            // deep band).
            rng.range(world_surface as i32, rock_layer as i32)
        } else {
            rng.range(
                ((world_surface + rock_layer) / 2.0) as i32,
                world.height - 230,
            )
        };
        let mut c = count_tiles(world, x, y, true, max_spider_count);
        while (c.tiles >= max_count || c.tiles < min_count) && tries < retries {
            tries += 1;
            x = rng.range(200, world.width - 200);
            y = if remix {
                // `:17402-17405` — the remix retry band.
                rng.range(world_surface as i32, rock_layer as i32)
            } else {
                rng.range(rock_layer as i32 + 30, world.height - 230)
            };
            c = count_tiles(world, x, y, true, max_spider_count);
            // `:17407-17410` — a shroom-bearing pocket (count > 1) forces
            // another retry: the C# zeroes the count INSIDE the loop, so
            // the acceptance test re-fails (W4-g41).
            if c.shroom > 1 {
                c.tiles = 0;
            }
        }
        if tries < retries {
            spread_spider(world, rng, x, y, hell_top, stock.as_deref_mut());
            placed += 1;
        }
    }
    // `:17422-17425` — the `ILied` tail dispatch (W4-g41): a `noSpiderCaves`
    // world with ≥ 4 secrets converts the evil/hive/jungle walls to spider
    // walls instead of losing the caves. The sweep itself is
    // `micro_finish::no_spider_caves_i_lied` (`WorldGen.cs:2618-2630`).
    if no_spider_caves_i_lied_more_spider_caves(world) {
        super::micro_finish::no_spider_caves_i_lied(world);
    }
    placed
}

/// M5-SEED-2 `SecretSeed.Variations.noSpiderCavesILiedMoreSpiderCaves`
/// (`WorldGen.cs:281-291`): the `noSpiderCaves` seed with ≥ 4 active secrets
/// keeps its spider caves and converts the other biome walls instead.
fn no_spider_caves_i_lied_more_spider_caves(world: &World) -> bool {
    if !world
        .micro_seeds
        .contains(crate::seeds::MicroSeedFlags::NO_SPIDER_CAVES)
    {
        return false;
    }
    crate::seeds::active_secret_seed_count(world.micro_seeds, world.seeds) >= 4
}

/// `Gem Caves` (`WorldGen.cs:17428-17471`).
pub(crate) fn pass_gem_caves(
    world: &mut World,
    rng: &mut Rng,
    rock_layer: f64,
    _world_surface: f64,
) -> u32 {
    let mut placed = 0u32;
    let attempts = (world.width as f64 * 0.003) as i32;
    for _ in 0..attempts {
        let mut tries = 0i32;
        let mut x = rng.range(200, world.width - 200);
        let mut y = rng.range(rock_layer as i32 + 30, world.height - 230);
        let mut c = count_tiles(world, x, y, false, 300);
        while (c.tiles >= 300
            || c.tiles < 50
            || c.lava > 0
            || c.ice > 0
            || c.rock == 0)
            && tries < 1000
        {
            tries += 1;
            x = rng.range(200, world.width - 200);
            y = rng.range(rock_layer as i32 + 30, world.height - 230);
            c = count_tiles(world, x, y, false, 300);
        }
        if tries < 1000 {
            gem_cave(world, rng, x, y);
            placed += 1;
        }
    }
    placed
}

/// `Moss` (`WorldGen.cs:17472-17700`) — the neon-moss biomes, the moss caves,
/// the scatters, the lava moss and the closing spread sweep.
pub(crate) fn pass_moss(
    world: &mut World,
    rng: &mut Rng,
    rock_layer: f64,
    water_line: f64,
    lava_line: f64,
    hell_top: i32,
) -> u32 {
    let (moss_type, neon) = rand_moss(rng);
    let neon_moss_type = NEON_MOSS[neon];
    let mut caves = 0u32;
    // Arm A — the neon moss biomes. W4-c29: the C#'s `num2`/`num3` ledger
    // (`:17554-17566`) — a rejected anchor bumps `num2` (the consecutive-reject
    // streak) and only advances `num3` (the placed count) once the streak
    // passes `maxTilesX`; an accepted anchor RESETS `num2` and advances `num3`
    // at once. The draft advanced the placed count on every attempt, so a
    // reject-heavy world silently placed fewer biomes than the C#.
    let wanted = (world.width / 2100) as i32;
    let mut reject_streak = 0i32;
    let mut placed_count = 0i32;
    while placed_count < wanted {
        let x = neon_moss_x(rng, world.width, world.seeds);
        let y = rng.range(rock_layer as i32 + 40, lava_line as i32 - 40);
        // The exclusion scan (`:17518-17553`) — the classic arm's set
        // (`:17545`): 70/60/367/368/161/147/396/397 plus `Main.tileDungeon`
        // (the dungeon bricks, `Main.cs:7992-7997` — the draft omitted them).
        let mut rejected = false;
        'scan: for i in (x - 50)..=(x + 50) {
            for j in (y - 50)..=(y + 50) {
                if let Some(t) = world.tile(i, j) {
                    if t.active()
                        && (matches!(
                            t.kind,
                            70 | 60 | TILE_MARBLE | TILE_GRANITE | TILE_ICE | TILE_SNOW | 396 | 397
                        ) || TILE_DUNGEON.contains(&t.kind))
                    {
                        rejected = true;
                        break 'scan;
                    }
                }
            }
        }
        if rejected {
            reject_streak += 1;
            if reject_streak > world.width {
                placed_count += 1;
            }
        } else {
            reject_streak = 0;
            placed_count += 1;
            neon_moss_biome(world, rng, x, y, lava_line as i32, neon_moss_type);
        }
    }
    // Arm B — the moss caves.
    let attempts = (world.width as f64 * 0.01) as i32;
    for _ in 0..attempts {
        let mut tries = 0i32;
        let mut x = rng.range(200, world.width - 200);
        let mut y = rng.range(
            ((world.world_surface + rock_layer) / 2.0) as i32,
            water_line as i32,
        );
        let mut c = count_tiles(world, x, y, false, 2500);
        while (c.tiles >= 2500
            || c.tiles < 10
            || c.lava > 0
            || c.ice > 0
            || c.rock == 0
            || c.shroom > 0)
            && tries < 1000
        {
            tries += 1;
            x = rng.range(200, world.width - 200);
            y = rng.range(rock_layer as i32 + 30, world.height - 230);
            c = count_tiles(world, x, y, false, 2500);
        }
        if tries < 1000 {
            let (moss_wall, moss_tile) = set_moss(world, x, &moss_type);
            spread_moss(world, x, y, moss_wall, moss_tile);
            caves += 1;
        }
    }
    // Arm C — the random dirt-to-moss scatter.
    for _ in 0..world.width {
        let x = rng.range(50, world.width - 50);
        let y = rng.range(
            ((world.world_surface + rock_layer) / 2.0) as i32,
            lava_line as i32,
        );
        if world.tile(x, y).is_some_and(|t| t.active() && t.kind == TILE_STONE) {
            let (_, moss_tile) = set_moss(world, x, &moss_type);
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = moss_tile;
            }
        }
    }
    // Arm D — the exposed-surface moss. The C# loop (`WorldGen.cs:17612-
    // 17623`) decrements only on a hit, so it relies on the vanilla cave field
    // always offering another exposed stone cell; the port caps the attempts
    // (`6 * maxTilesX`, the M5-STRUCT-5 termination guard) so a degenerate
    // fixture cannot spin.
    let mut targets = world.width as f64 * 0.05;
    let mut arm_d_attempts = 0i32;
    let arm_d_cap = world.width * 6;
    while targets > 0.0 && arm_d_attempts < arm_d_cap {
        arm_d_attempts += 1;
        let x = rng.range(50, world.width - 50);
        let y = rng.range(
            ((world.world_surface + rock_layer) / 2.0) as i32,
            lava_line as i32,
        );
        let exposed = world.tile(x, y).is_some_and(|t| t.active() && t.kind == TILE_STONE)
            && (world.tile(x - 1, y).is_none_or(|t| !t.active())
                || world.tile(x + 1, y).is_none_or(|t| !t.active())
                || world.tile(x, y - 1).is_none_or(|t| !t.active())
                || world.tile(x, y + 1).is_none_or(|t| !t.active()));
        if exposed {
            let (_, moss_tile) = set_moss(world, x, &moss_type);
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = moss_tile;
            }
            targets -= 1.0;
        }
    }
    // Arm E — the lava moss (`:17624-17658`).
    let mut budget = world.width as f64 * 0.065;
    while budget > 0.0 {
        let x = rng.range(50, world.width - 50);
        let y = rng.range(water_line as i32, hell_top as i32);
        if world.tile(x, y).is_some_and(|t| t.active() && t.kind == TILE_STONE)
            && (world.tile(x - 1, y).is_none_or(|t| !t.active())
                || world.tile(x + 1, y).is_none_or(|t| !t.active())
                || world.tile(x, y - 1).is_none_or(|t| !t.active())
                || world.tile(x, y + 1).is_none_or(|t| !t.active()))
        {
            let mut lava_here = 0i32;
            for m in (x - 25)..(x + 25) {
                for n in (y - 25)..(y + 25) {
                    if world
                        .tile(m, n)
                        .is_some_and(|t| t.liquid > 0 && t.liquid_kind() == crate::tile::LiquidKind::Lava)
                    {
                        lava_here += 1;
                    }
                }
            }
            if lava_here > 20 {
                if let Some(t) = world.tile_mut(x, y) {
                    t.kind = TILE_LAVA_MOSS;
                }
                budget -= 1.0;
            } else {
                budget -= 0.002;
            }
        }
        budget -= 0.001;
    }
    // Arm F — the closing spread sweep (`:17659-17698`). W4-c29: the C# walks
    // the map LIVE, column-major (`num20` = x outer, `num21` = y inner), and
    // reads `Main.tileMoss` — the generated `TILE_MOSS` set carries the
    // rainbow moss 627 (`Main.cs:7031`) beside the five + neon + lava moss.
    // The draft snapshotted a ROW-major list without 627: a different visit
    // order (the sweep's conversions feed later cells) and a missed family.
    for x in 0..world.width {
        for y in 0..world.height {
            let kind = world.tile(x, y).map(|t| (t.active(), t.kind));
            let Some((active, kind)) = kind else {
                continue;
            };
            if active && crate::tiledata::TILE_MOSS.has(kind) {
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    spread_grass(world, x + dx, y + dy, TILE_STONE, kind);
                }
            }
        }
    }
    caves
}

/// `Webs And Honey` (`WorldGen.cs:19995-20063`) — the pass that hangs cobweb
/// in the spider caves and converts the hive liquid to honey + speleothems.
pub(crate) fn pass_webs_and_honey(world: &mut World, rng: &mut Rng, _world_surface: f64) -> u32 {
    let mut webs = 0u32;
    // `:20002-20006` — the Don't Starve world starts the sweep at row 50
    // instead of the surface line (W4-g41).
    let start_row = if world.seeds.contains(crate::seeds::SeedFlags::DONT_STARVE) {
        50
    } else {
        world.world_surface as i32
    };
    let i_lied = no_spider_caves_i_lied_more_spider_caves(world);
    for i in 100..world.width - 100 {
        for j in start_row..world.height - 100 {
            let wall = world.tile(i, j).map(|t| t.wall).unwrap_or(0);
            if wall == WALL_HIVE {
                let has_liquid = world.tile(i, j).is_some_and(|t| t.liquid > 0);
                if has_liquid {
                    if let Some(t) = world.tile_mut(i, j) {
                        t.set_liquid_kind(crate::tile::LiquidKind::Honey);
                    }
                }
                if rng.range(0, 3) == 0 {
                    place_tight(world, rng, i, j, false);
                }
            }
            if wall == WALL_SPIDER {
                if i_lied {
                    // `:20022-20028` — the `ILied` arm honey-fills the
                    // spider-wall liquid instead of clearing it (W4-g41).
                    let has_liquid = world.tile(i, j).is_some_and(|t| t.liquid > 0);
                    if has_liquid {
                        if let Some(t) = world.tile_mut(i, j) {
                            t.set_liquid_kind(crate::tile::LiquidKind::Honey);
                        }
                    }
                } else if let Some(t) = world.tile_mut(i, j) {
                    t.liquid = 0;
                    t.clear_liquid_kind();
                }
                let active = world.tile(i, j).is_some_and(|t| t.active());
                if !active && rng.range(0, 10) != 0 {
                    let spread = rng.range(2, 5);
                    let mut nearby_solid = false;
                    'scan: for k in (i - spread)..=(i + spread) {
                        for l in (j - spread)..=(j + spread) {
                            if solid_tile(world, k, l) {
                                nearby_solid = true;
                                break 'scan;
                            }
                        }
                    }
                    if nearby_solid {
                        if place_framed(world, i, j, TILE_COBWEB, 0) {
                            webs += 1;
                        }
                    }
                }
            }
        }
    }
    webs
}

// --- the spider-wall eraser (W4-g41) ------------------------------------------

/// `WorldGen.SpiderWallRemover` (`WorldGen.cs:77279-77344`) — the drifting
/// diamond eraser: a `Next(8,21)`-wide / `Next(8,33)`-step blob walks a
/// `Next(-10,11)*0.1` velocity, clearing wall 62 → 0 inside the shrinking
/// `|dx|+|dy| < radius*0.5*(1+Next(-10,11)*0.015)` diamond each step, with
/// the `Next(-10,11)*0.05` velocity jitter clamped to ±1.
///
/// WIRED (W4-g41 follow-up): the C#'s only callers are `FinishNotTheBees`'
/// two 10000-draw search loops (`:25409-25434`, the for-the-worthy +
/// not-the-bees combo), ported in `not_the_bees.rs::spider_wall_search` and
/// dispatched from `step_seed_finish`'s NTB slot. A classic world reads no
/// flag and never reaches it.
pub(crate) fn spider_wall_remover(world: &mut World, rng: &mut Rng, i: i32, j: i32) {
    // `:77281` — the width budget; the C# never decrements `num` (only the
    // `num3` life counter steps down), so the loop runs exactly `num2` times.
    let num = rng.range(8, 21) as f64;
    let num2 = rng.range(8, 33) as f64;
    let mut num3 = num2;
    let mut pos = (i as f64, j as f64);
    let mut vel = (
        rng.range(-10, 11) as f64 * 0.1,
        rng.range(-10, 11) as f64 * 0.1,
    );
    while num > 0.0 && num3 > 0.0 {
        let num4 = num * (num3 / num2);
        num3 -= 1.0;
        // `:77294-77313` — the diamond's bounding box, clamped to the world
        // (the C# clamps to `maxTilesX`/`maxTilesY` exclusive).
        let mut xmin = (pos.0 - num4 * 0.5) as i32;
        let mut xmax = (pos.0 + num4 * 0.5) as i32;
        let mut ymin = (pos.1 - num4 * 0.5) as i32;
        let mut ymax = (pos.1 + num4 * 0.5) as i32;
        if xmin < 0 {
            xmin = 0;
        }
        if xmax > world.width {
            xmax = world.width;
        }
        if ymin < 0 {
            ymin = 0;
        }
        if ymax > world.height {
            ymax = world.height;
        }
        for k in xmin..xmax {
            for l in ymin..ymax {
                // `:77318` — the jittered diamond test: the C# `&&` evaluates
                // the distance FIRST, so the `Next(-10,11)` draw runs only
                // for cells already inside the un-jittered radius (the port
                // mirrors the short-circuit by scaling the base radius).
                let dist = (k as f64 - pos.0).abs() + (l as f64 - pos.1).abs();
                if dist < num * 0.5 * (1.0 + rng.range(-10, 11) as f64 * 0.015)
                    && world.tile(k, l).is_some_and(|t| t.wall == WALL_SPIDER)
                {
                    if let Some(t) = world.tile_mut(k, l) {
                        t.wall = 0;
                    }
                }
            }
        }
        pos = (pos.0 + vel.0, pos.1 + vel.1);
        vel.0 += rng.range(-10, 11) as f64 * 0.05;
        if vel.0 > 1.0 {
            vel.0 = 1.0;
        }
        if vel.0 < -1.0 {
            vel.0 = -1.0;
        }
        vel.1 += rng.range(-10, 11) as f64 * 0.05;
        if vel.1 > 1.0 {
            vel.1 = 1.0;
        }
        if vel.1 < -1.0 {
            vel.1 = -1.0;
        }
    }
}

/// The dirt/stone ids the scatter arms read (kept for the const block to stay
/// a real dependency — the arms compare against `TILE_STONE` directly).
const _: (u16, u16, u16, u16) = (TILE_DIRT, TILE_STONE, TILE_SAND, WALL_SANDSTONE);

/// `Main.wallDungeon` (`Main.cs:10598-10606`): the nine dungeon wall ids.
const WALL_DUNGEON: [u16; 9] = [7, 8, 9, 94, 95, 96, 97, 98, 99];

/// `ExposedGemsUnderground` (`WorldGen.cs:20729-20768`): two `maxTilesX`
/// loops. The first scatters single gems (178) on the free underground
/// cells (the rock-layer band, no lava, no dungeon/planked wall) with the
/// weighted `Next(12)` style; the second fills gem patches (style 6, the
/// diamond) on the free cells beside the marble/granite walls (216/187) in
/// the `Next(1,4)`-half-sized rectangles.
pub(crate) fn pass_random_gems(
    world: &mut World,
    rng: &mut Rng,
    world_surface: f64,
    rock_layer: f64,
) {
    // The sub-vanilla guard: both y bands must be non-empty (the C# throws
    // on `Next(rockLayer, height-300)` collapsing; the port skips the loops
    // before any draw — the wave B convention).
    let y_hi = world.height - 300;
    if world.width <= 40 || y_hi <= rock_layer as i32 || y_hi <= world_surface as i32 {
        return;
    }
    // `:20734-20744` — the single-gem scatter.
    for _ in 0..world.width {
        let x = rng.range(20, world.width - 20);
        let y = rng.range(rock_layer as i32, y_hi);
        let Some(t) = world.tile(x, y) else {
            continue;
        };
        let any_lava = t.liquid > 0 && t.liquid_kind() == crate::tile::LiquidKind::Lava;
        if !t.active() && !any_lava && !WALL_DUNGEON.contains(&t.wall) && t.wall != 27 {
            // `:20740-20742` — the weighted style (`Next(12)`: <3 → 0,
            // <6 → 1, <8 → 2, <10 → 3, ≥11 → 5, else 4).
            let roll = rng.range(0, 12);
            let style = if roll < 3 {
                0
            } else if roll < 6 {
                1
            } else if roll < 8 {
                2
            } else if roll < 10 {
                3
            } else if roll >= 11 {
                5
            } else {
                4
            };
            // batch-O keep-one: the c29-fidelity `place_exposed_gem` (the
            // :59691-59701 arm above — the anchor gate + the Next(3) row,
            // rng-first signature) serves BOTH callers; the d105 local copy
            // retired (its square_tile_frame_gen rides the post-pipeline
            // framing seam, the M5-STRUCT-5 idiom the c29 form documents).
            place_exposed_gem(world, rng, x, y, style);
        }
    }
    // `:20745-20766` — the marble/granite gem patches.
    for _ in 0..world.width {
        let x = rng.range(20, world.width - 20);
        let y = rng.range(world_surface as i32, y_hi);
        let Some(t) = world.tile(x, y) else {
            continue;
        };
        let any_lava = t.liquid > 0 && t.liquid_kind() == crate::tile::LiquidKind::Lava;
        if !t.active() && !any_lava && (t.wall == 216 || t.wall == 187) {
            let left = rng.range(1, 4);
            let right = rng.range(1, 4);
            let up = rng.range(1, 4);
            let down = rng.range(1, 4);
            for k in x - left..x + right {
                for l in y - up..y + down {
                    if world.tile(k, l).is_some_and(|t| !t.active()) {
                        place_exposed_gem(world, rng, k, l, 6);
                    }
                }
            }
        }
    }
}

// batch-O keep-one (c29 × d105): the d105 local `place_exposed_gem` (x/y-first
// signature) + its `check_and_adjust_multi_directional`/`valid_anchor`/three
// slope-helper copies RETIRED — the c29-fidelity forms above serve both
// callers (`spread_gem` :403 + `pass_random_gems`), the anchor predicates ride
// the canonical `crate::edit::solid_tile_allow_*_slope` seams (kill.rs), and
// `SquareTileFrame` rides the post-pipeline framing seam (the M5-STRUCT-5
// idiom; the d105 immediate `square_tile_frame_gen` call retired with the dup
// — the C# :59700 call is covered by the pipeline's final framing pass).

/// `LongMoss` (`WorldGen.cs:20769-20812`): the full-world sweep — every
/// active moss tile (`Main.tileMoss`) grows the long moss (184) on each of
/// its four free neighbours via `PlaceTile(184)`. The sweep draws no RNG
/// of its own beyond the `PlaceTile`-184 arm's `Next(3)` frame variants.
pub(crate) fn pass_moss_grass(world: &mut World, rng: &mut Rng) {
    let last_x = world.width - 5;
    for i in 5..last_x {
        for j in 5..world.height - 5 {
            // `:20781` — the active moss gate.
            if !world
                .tile(i, j)
                .is_some_and(|t| t.active() && crate::tiledata::TILE_MOSS.has(t.kind))
            {
                continue;
            }
            // `:20783-20806` — the four neighbours (left, right, up, down).
            for k in 0..4 {
                let (nx, ny) = match k {
                    0 => (i - 1, j),
                    1 => (i + 1, j),
                    2 => (i, j - 1),
                    _ => (i, j + 1),
                };
                if world.tile(nx, ny).is_some_and(|t| !t.active()) {
                    place_long_moss(world, nx, ny, rng);
                }
            }
        }
    }
}

/// `WorldGen.PlaceTile`'s long-moss arm (`WorldGen.cs:59703-59720`, type
/// 184): two independent gates — a solid `tileMoss` neighbour, then a solid
/// `tileMossBrick` neighbour — each writing active 184 with `frameX
/// style*18` (the pass passes style 0) and a fresh `frameY Next(3)*18`.
/// Both gates can fire on one cell (two draws); the liquid gate
/// (`:59454-59471` — 184 is NOT in the refusal list) passes every liquid
/// cell.
fn place_long_moss(world: &mut World, x: i32, y: i32, rng: &mut Rng) {
    // The four neighbour reads (the type test reads inactive cells too —
    // the C# tests `.type` with no active gate; `SolidTile` gates it).
    let neighbours = [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)];
    let moss_side = neighbours.iter().any(|&(nx, ny)| {
        world.tile(nx, ny).is_some_and(|t| crate::tiledata::TILE_MOSS.has(t.kind))
            && solid_tile(world, nx, ny)
    });
    if moss_side {
        let frame_y = rng.range(0, 3) * 18;
        if let Some(t) = world.tile_mut(x, y) {
            t.flags |= FLAG_ACTIVE;
            t.kind = 184;
            t.frame_x = 0;
            t.frame_y = frame_y as i16;
        }
        crate::edit::square_tile_frame_gen_legacy(world, x, y);
    }
    let brick_side = neighbours.iter().any(|&(nx, ny)| {
        world
            .tile(nx, ny)
            .is_some_and(|t| crate::tiledata::SET_CONVERSION_MOSS_BRICK.has(t.kind))
            && solid_tile(world, nx, ny)
    });
    if brick_side {
        let frame_y = rng.range(0, 3) * 18;
        if let Some(t) = world.tile_mut(x, y) {
            t.flags |= FLAG_ACTIVE;
            t.kind = 184;
            t.frame_x = 0;
            t.frame_y = frame_y as i16;
        }
        crate::edit::square_tile_frame_gen_legacy(world, x, y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A flat stone world carrying the port's wall baseline, so the cave-decor
    /// passes see the same wall state the pipeline hands them.
    fn flat_world(width: i32, height: i32, surface: i32) -> World {
        let mut w = World::new("sgm".into(), width, height);
        for x in 0..width {
            for y in 0..height {
                let i = w.index(x, y);
                let mut t = Tile::default();
                if y >= surface {
                    t.kind = TILE_STONE;
                    t.flags |= FLAG_ACTIVE;
                    t.wall = if y > surface + 40 { WALL_STONE } else { WALL_DIRT };
                }
                w.tiles[i] = t;
            }
        }
        w.world_surface = surface as f64;
        w.rock_layer = (surface + (height - surface) / 2) as f64;
        w
    }

    /// A single air pocket with a stone floor and ceiling.
    fn carve_pocket(world: &mut World, x: i32, y: i32, w: i32, h: i32) {
        for px in x..x + w {
            for py in y..y + h {
                let i = world.index(px, py);
                let t = &mut world.tiles[i];
                t.flags &= !FLAG_ACTIVE;
            }
        }
    }

    /// The mid-world pocket the vanilla-width passes can find.
    fn pocket() -> (i32, i32, i32) {
        (300, 120, 28)
    }

    #[test]
    fn count_tiles_measures_an_open_pocket_and_bails_on_a_wall() {
        let (px, py, size) = pocket();
        let mut w = flat_world(600, 300, 40);
        carve_pocket(&mut w, px, py, size, size);
        let c = count_tiles(&w, px + 10, py + 10, false, 3500);
        let cells = size * size;
        assert!(c.tiles >= cells, "the pocket measured {} of {cells}", c.tiles);
        assert!(!c.bailed);
        // The stone rim the fill touches counts as rock.
        assert!(c.rock > 50, "the stone rim counts as rock ({})", c.rock);
        assert!(c.sand == 0 && c.ice == 0);
        // A painted wall (the gem/moss/cave walls) bails the count.
        let i = w.index(px + 10, py + 10);
        w.tiles[i].wall = 50;
        let c2 = count_tiles(&w, px + 10, py + 10, false, 3500);
        assert!(c2.bailed, "a real wall bails the count");
        assert_eq!(c2.tiles, 3500);
        // The world border bails too.
        let c3 = count_tiles(&w, 1, 1, false, 3500);
        assert!(c3.bailed);
    }

    #[test]
    fn spread_spider_paints_the_spider_wall_over_the_pocket() {
        let (px, py, size) = pocket();
        let mut w = flat_world(600, 300, 40);
        carve_pocket(&mut w, px, py, size, size);
        let mut rng = Rng(0x51DE_C0DE);
        spread_spider(&mut w, &mut rng, px + 10, py + 10, 180.0, None);
        let spider = w.tiles.iter().filter(|t| t.wall == WALL_SPIDER).count();
        assert!(spider >= (size * size) as usize, "the spider wall spread ({spider})");
        // The liquid inside the cave is cleared.
        let liquid = w.tiles.iter().filter(|t| t.liquid > 0).count();
        assert_eq!(liquid, 0);
    }

    #[test]
    fn the_spider_pass_finds_a_deep_pocket() {
        let mut w = flat_world(900, 900, 60);
        // The pass draws y in [(worldSurface + rockLayer) / 2, maxTilesY - 230)
        // and its retries in [rockLayer + 30, maxTilesY - 230) (:17391, :17401);
        // the acceptance gate is `500 <= countTiles < 3500` (:17365, :17397),
        // so the pocket sits in the retry band and stays under the cap.
        carve_pocket(&mut w, 300, 520, 300, 10);
        let mut rng = Rng(0xABCD);
        let rock = w.rock_layer;
        let surface = w.world_surface;
        let placed = pass_spider_caves(&mut w, &mut rng, rock, surface, 780.0, None);
        assert!(placed >= 1, "the pass placed a spider cave");
        let spider = w.tiles.iter().filter(|t| t.wall == WALL_SPIDER).count();
        assert!(spider >= 500, "the spider wall covers the cave ({spider})");
    }

    #[test]
    fn the_no_spider_caves_variation_skips_the_whole_pass() {
        // `WorldGen.cs:17358` + `Variations.noSpiderCavesActuallyNoSpiderCaves`
        // (`:270-278`): a lone `noSpiderCaves` world
        // (`activeSecretSeedCount = 1 < 4`) runs no spider caves at all.
        let mut w = flat_world(900, 900, 60);
        carve_pocket(&mut w, 300, 520, 300, 10);
        w.micro_seeds |= crate::seeds::MicroSeedFlags::NO_SPIDER_CAVES;
        let mut rng = Rng(0xABCD);
        let (rock, surface) = (w.rock_layer, w.world_surface);
        let placed = pass_spider_caves(&mut w, &mut rng, rock, surface, 780.0, None);
        assert_eq!(placed, 0, "the pass is skipped whole");
        assert_eq!(w.tiles.iter().filter(|t| t.wall == WALL_SPIDER).count(), 0);
        // A crowded secret-seed list (count >= 4) keeps the pass running — the
        // other agent's `ILied` arm territory. The three extra bits beside
        // `noSpiderCaves` reach the threshold.
        let mut crowded = flat_world(900, 900, 60);
        carve_pocket(&mut crowded, 300, 520, 300, 10);
        crowded.micro_seeds |= crate::seeds::MicroSeedFlags::NO_SPIDER_CAVES
            | crate::seeds::MicroSeedFlags::ERROR_WORLD
            | crate::seeds::MicroSeedFlags::RAINS_FOR_A_YEAR
            | crate::seeds::MicroSeedFlags::RANDOM_SPAWN;
        assert!(crowded.micro_seeds.count() >= 4);
        let mut rng2 = Rng(0xABCD);
        let (rock2, surface2) = (crowded.rock_layer, crowded.world_surface);
        let ran = pass_spider_caves(&mut crowded, &mut rng2, rock2, surface2, 780.0, None);
        assert!(ran >= 1, "the crowded world still runs the pass");
        assert!(crowded.tiles.iter().filter(|t| t.wall == WALL_SPIDER).count() >= 500);
    }

    #[test]
    fn the_spider_decor_places_the_seat_families() {
        let (px, py, size) = pocket();
        // Sweep seeds until the decor families land, then pin their frames.
        for seed in 0..60u64 {
            let mut w = flat_world(600, 300, 40);
            carve_pocket(&mut w, px, py, size, size);
            spread_spider(&mut w, &mut Rng(seed), px + 10, py + 10, 180.0, None);
            let pots = w.tiles.iter().filter(|t| t.kind == TILE_POT).count();
            let stal = w.tiles.iter().filter(|t| t.kind == TILE_STALACTITE).count();
            if pots >= 2 && stal >= 2 {
                // The pot's frame pair comes from `PlacePot`'s style math.
                let pot = w.tiles.iter().find(|t| t.kind == TILE_POT).unwrap();
                assert!(pot.frame_y >= 36, "the pot row frame {}", pot.frame_y);
                // The spider stalactite uses the 108 + variation*18 family.
                let stal = w.tiles.iter().find(|t| t.kind == TILE_STALACTITE).unwrap();
                assert!(
                    (108..162).contains(&stal.frame_x),
                    "the spider stalactite frame {}",
                    stal.frame_x
                );
                return;
            }
        }
        panic!("no seed produced both a pot and a stalactite");
    }

    #[test]
    fn gem_caves_paint_gem_walls_and_convert_the_stone_rim() {
        let (px, py, size) = pocket();
        let mut w = flat_world(600, 300, 40);
        carve_pocket(&mut w, px, py, size, size);
        let mut rng = Rng(0xC0FFEE);
        gem_cave(&mut w, &mut rng, px + 10, py + 10);
        let gem_walls = w
            .tiles
            .iter()
            .filter(|t| (WALL_GEM_BASE..WALL_GEM_BASE + 6).contains(&t.wall))
            .count();
        assert!(gem_walls >= (size * size) as usize, "the gem walls ({gem_walls})");
        // W4-c29: the exposed gems ride `PlaceTile`'s 178 arm — the
        // multi-directional anchor gate means only the pocket's rim-adjacent
        // open cells qualify (the deep interior has no solid neighbour), and
        // each placement draws the `Next(3)*18` frame row.
        let exposed: Vec<_> = w
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == TILE_EXPOSED_GEMS)
            .collect();
        let cells = (size * size) as usize;
        assert!(!exposed.is_empty(), "the rim-adjacent cells expose gems");
        assert!(exposed.len() <= cells, "never more than the pocket ({})", exposed.len());
        for t in &exposed {
            assert!(t.frame_x % 18 == 0 && (0..6).contains(&(t.frame_x / 18)),
                "the gem style column {}", t.frame_x);
            assert!(matches!(t.frame_y, 0 | 18 | 36),
                "the Next(3) frame row {}", t.frame_y);
        }
    }

    #[test]
    fn the_gem_pass_finds_a_pocket_that_passes_the_vanilla_gate() {
        let mut w = flat_world(900, 600, 60);
        carve_pocket(&mut w, 300, 200, 24, 24);
        let mut rng = Rng(0x5A5A);
        let rock = w.rock_layer;
        let surface = w.world_surface;
        let placed = pass_gem_caves(&mut w, &mut rng, rock, surface);
        // The pocket is tiny for the `50..300` gate, so a 0 count is a
        // legitimate outcome; the walls must not appear without a cave.
        let gem_walls = w
            .tiles
            .iter()
            .filter(|t| (WALL_GEM_BASE..WALL_GEM_BASE + 6).contains(&t.wall))
            .count();
        if placed > 0 {
            assert!(gem_walls > 0);
        } else {
            assert_eq!(gem_walls, 0, "no cave, no gem walls");
        }
    }

    #[test]
    fn spread_moss_converts_stone_and_paints_the_cave_wall() {
        let (px, py, size) = pocket();
        let mut w = flat_world(600, 300, 40);
        carve_pocket(&mut w, px, py, size, size);
        spread_moss(&mut w, px + 10, py + 10, WALL_MOSS_BASE, MOSS_TILES[0]);
        let moss_walls = w
            .tiles
            .iter()
            .filter(|t| (WALL_MOSS_BASE..WALL_MOSS_BASE + 5).contains(&t.wall))
            .count();
        assert!(moss_walls >= (size * size) as usize, "the cave moss wall ({moss_walls})");
        let moss_tiles = w.tiles.iter().filter(|t| t.kind == MOSS_TILES[0]).count();
        assert!(moss_tiles > 50, "the stone rim became moss ({moss_tiles})");
    }

    #[test]
    fn rand_moss_picks_three_distinct_region_indices() {
        let mut rng = Rng(7);
        for _ in 0..50 {
            let (moss, neon) = rand_moss(&mut rng);
            assert!(neon < 4);
            assert_ne!(moss[0], moss[1]);
            assert_ne!(moss[0], moss[2]);
            assert_ne!(moss[1], moss[2]);
            for m in moss {
                assert!(m < 5);
            }
        }
    }

    #[test]
    fn set_moss_bins_the_region_by_the_x_band() {
        let w = flat_world(600, 300, 40);
        let moss_type = [0usize, 1, 2];
        assert_eq!(set_moss(&w, 10, &moss_type), (WALL_MOSS_BASE, MOSS_TILES[0]));
        assert_eq!(
            set_moss(&w, 350, &moss_type),
            (WALL_MOSS_BASE + 1, MOSS_TILES[1])
        );
        assert_eq!(
            set_moss(&w, 560, &moss_type),
            (WALL_MOSS_BASE + 2, MOSS_TILES[2])
        );
    }

    #[test]
    fn the_moss_pass_is_deterministic_for_a_seed() {
        let build = |seed: u64| {
            let mut w = flat_world(600, 700, 40);
            carve_pocket(&mut w, 300, 520, 28, 28);
            let mut rng = Rng(seed);
            let rock = w.rock_layer;
            pass_moss(&mut w, &mut rng, rock, 420.0, 530.0, 480);
            w.tiles.clone()
        };
        assert_eq!(build(11), build(11));
    }

    #[test]
    fn the_moss_pass_paints_the_cave_wall_over_the_pocket() {
        let mut w = flat_world(900, 600, 60);
        // Arm B's retry band is [rockLayer + 30, maxTilesY - 230) (:17601) and
        // its gate is `10 <= countTiles < 2500` (:17601-17604) — the pocket sits
        // in the band and under the cap.
        carve_pocket(&mut w, 300, 355, 200, 10);
        let mut rng = Rng(0x5EED);
        let rock = w.rock_layer;
        pass_moss(&mut w, &mut rng, rock, 420.0, 530.0, 480);
        let moss_walls = w
            .tiles
            .iter()
            .filter(|t| (WALL_MOSS_BASE..WALL_MOSS_BASE + 5).contains(&t.wall))
            .count();
        assert!(moss_walls > 100, "the moss cave wall landed ({moss_walls})");
    }

    #[test]
    fn the_webs_pass_fills_the_spider_walls_with_cobweb() {
        let mut w = flat_world(600, 300, 40);
        // Paint the spider wall over a pocket, then run the webs pass.
        carve_pocket(&mut w, 300, 120, 40, 30);
        for px in 300..340 {
            for py in 120..150 {
                let i = w.index(px, py);
                if !w.tiles[i].active() {
                    w.tiles[i].wall = WALL_SPIDER;
                }
            }
        }
        let mut rng = Rng(0x5EED);
        let surface = w.world_surface;
        let webs = pass_webs_and_honey(&mut w, &mut rng, surface);
        let cobweb = w.tiles.iter().filter(|t| t.kind == TILE_COBWEB).count();
        assert!(cobweb > 0, "the cobweb landed ({cobweb})");
        assert!(webs >= 1);
    }

    #[test]
    fn the_stalactite_frame_table_matches_the_support_kind() {
        // Stone support → the 54 + variation*18 family.
        let mut w = flat_world(300, 200, 40);
        carve_pocket(&mut w, 100, 100, 40, 20);
        place_unchecked_stalactite(&mut w, 110, 100, false, 0, false);
        let t = w.tile(110, 100).unwrap();
        assert_eq!(t.kind, TILE_STALACTITE);
        assert_eq!(t.frame_x, 54);
        assert_eq!(t.frame_y, 0);
        assert_eq!(w.tile(110, 101).unwrap().frame_y, 18);
        // The spider family.
        let mut w2 = flat_world(300, 200, 40);
        carve_pocket(&mut w2, 100, 100, 40, 20);
        place_unchecked_stalactite(&mut w2, 110, 100, false, 1, true);
        assert_eq!(w2.tile(110, 100).unwrap().frame_x, 108 + 18);
        assert_eq!(w2.tile(110, 100).unwrap().frame_y, 0);
        // The sandstone support.
        let mut w3 = flat_world(300, 200, 40);
        carve_pocket(&mut w3, 100, 100, 40, 20);
        let seat = w3.index(110, 99);
        w3.tiles[seat].kind = 396;
        place_unchecked_stalactite(&mut w3, 110, 100, false, 0, false);
        assert_eq!(w3.tile(110, 100).unwrap().frame_x, 378);
    }

    #[test]
    fn the_pot_needs_an_empty_seat_and_a_solid_floor() {
        let mut w = flat_world(300, 200, 40);
        carve_pocket(&mut w, 100, 100, 40, 20);
        let mut rng = Rng(3);
        // The seat sits on the pocket floor: the row above it must be air and
        // the row below it solid.
        assert!(place_pot(&mut w, &mut rng, 110, 119, 19));
        let t = w.tile(110, 119).unwrap();
        assert_eq!(t.kind, TILE_POT);
        assert!(t.frame_y >= 36, "the pot row frame {}", t.frame_y);
        // A blocked seat refuses.
        let seat = w.index(110, 119);
        w.tiles[seat].flags |= FLAG_ACTIVE;
        assert!(!place_pot(&mut w, &mut rng, 110, 119, 19));
    }

    /// M5-SEED-1: `WorldGen.cs:17365`/`:17378` — the attempt count is
    /// `maxTilesX * 0.005`, tripled for the worthy. A classic world reads no
    /// flag and keeps the vanilla count.
    #[test]
    fn the_for_the_worthy_world_triples_the_spider_cave_attempts() {
        let mut world = flat_world(2000, 600, 60);
        let base = spider_attempts(&world);
        assert_eq!(base, 10, "maxTilesX * 0.005");
        world.seeds = crate::seeds::SeedFlags::from(crate::seeds::SeedFlags::FOR_THE_WORTHY);
        assert_eq!(spider_attempts(&world), base * 3, "the for-the-worthy 3x");
        // The C#'s FIRST branch is the not-the-bees + for-the-worthy combo
        // (`num2 *= 2` with the 1000/5000 count bounds), which the port does
        // not carry, so the 3x does not apply there either.
        world.seeds = crate::seeds::SeedFlags::from(
            crate::seeds::SeedFlags::NOT_THE_BEES | crate::seeds::SeedFlags::FOR_THE_WORTHY,
        );
        assert_eq!(
            spider_attempts(&world),
            base,
            "the bees combo takes branch 1"
        );
    }

    /// M5-SEED-1: `WorldGen.cs:17503-17508` — the for-the-worthy neon-moss
    /// anchor is re-rolled out of the 0.42-0.48 band, while the classic (and
    /// drunk) arm draws `num4` exactly once.
    #[test]
    fn the_for_the_worthy_neon_moss_anchor_avoids_the_centre_band() {
        let width = 2000i32;
        let (lo, hi) = (width as f64 * 0.42, width as f64 * 0.48);
        let classic = crate::seeds::SeedFlags::default();
        let worthy = crate::seeds::SeedFlags::from(crate::seeds::SeedFlags::FOR_THE_WORTHY);
        let drunk = crate::seeds::SeedFlags::from(crate::seeds::SeedFlags::DRUNK);
        let mut classic_saw_band = false;
        let mut worthy_rerolled = false;
        for seed in 0..512u64 {
            // The seed is mixed multiplicatively: a run of consecutive seeds
            // differing only in their low bits makes xorshift's linear step
            // collapse (the 512 draws all land in one 100-tile window), so the
            // sweep would never sample the centre band.
            let mixed = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x4A11;
            // The classic arm: one draw, no re-roll — the reference stream is
            // a single `Next(100, maxTilesX - 100)`.
            let mut plain = Rng(mixed);
            let drawn = plain.range(100, width - 100);
            let mut classic_rng = Rng(mixed);
            let classic = neon_moss_x(&mut classic_rng, width, classic);
            assert_eq!(classic, drawn, "the classic arm draws once (seed {seed})");
            assert_eq!(classic_rng.0, plain.0, "and leaves the stream there");
            if (drawn as f64) > lo && (drawn as f64) < hi {
                classic_saw_band = true;
            }
            // The drunk arm skips the re-roll entirely, so it matches the
            // classic path draw for draw.
            let mut drunk_rng = Rng(mixed);
            assert_eq!(neon_moss_x(&mut drunk_rng, width, drunk), drawn);
            assert_eq!(drunk_rng.0, plain.0, "the drunk arm re-rolls nothing");
            // The for-the-worthy arm: never inside the band, and the sweep
            // catches at least one seed whose first draw was (so the re-roll
            // is genuinely exercised).
            let mut worthy_rng = Rng(mixed);
            let anchored = neon_moss_x(&mut worthy_rng, width, worthy);
            assert!(
                !((anchored as f64) > lo && (anchored as f64) < hi),
                "the worthy anchor left the 0.42-0.48 band (seed {seed})"
            );
            if worthy_rng.0 != plain.0 {
                worthy_rerolled = true;
            }
        }
        assert!(classic_saw_band, "the sweep never drew inside the band");
        assert!(worthy_rerolled, "the re-roll never fired");
    }

    /// M5-SEED-1: `WorldGen.cs:9801-9805` — the neon-moss blob scale is
    /// `maxTilesX / 4200.0`, 1.5x for the worthy (the radius `num` and the
    /// step count `num2` both ride it).
    #[test]
    fn the_for_the_worthy_neon_moss_blob_scale() {
        let mut world = flat_world(4200, 600, 60);
        assert_eq!(neon_moss_scale(&world), 1.0, "the vanilla 4200 baseline");
        world.seeds = crate::seeds::SeedFlags::from(crate::seeds::SeedFlags::FOR_THE_WORTHY);
        assert_eq!(neon_moss_scale(&world), 1.5, "the for-the-worthy 1.5x");
        // A sub-4200 world scales down before the for-the-worthy factor.
        let mut small = flat_world(2100, 600, 60);
        small.seeds = crate::seeds::SeedFlags::from(crate::seeds::SeedFlags::FOR_THE_WORTHY);
        assert_eq!(neon_moss_scale(&small), 0.75);
    }

    /// M5-SEED-1: the for-the-worthy blob covers strictly more stone than the
    /// classic one on identical worlds and seeds — `WorldGen.cs:9806-9807`
    /// scales both `num` and `num2`.
    #[test]
    fn the_for_the_worthy_neon_moss_blob_covers_more_stone() {
        let (mut classic, mut worthy) = {
            let mut c = flat_world(2000, 600, 60);
            carve_pocket(&mut c, 900, 200, 220, 140);
            (c.clone(), c)
        };
        worthy.seeds = crate::seeds::SeedFlags::from(crate::seeds::SeedFlags::FOR_THE_WORTHY);
        neon_moss_biome(&mut classic, &mut Rng(0x4E0A), 1000, 260, 520, NEON_MOSS[0]);
        neon_moss_biome(&mut worthy, &mut Rng(0x4E0A), 1000, 260, 520, NEON_MOSS[0]);
        let painted =
            |w: &World| w.tiles.iter().filter(|t| t.kind == NEON_MOSS[0]).count();
        let (base, big) = (painted(&classic), painted(&worthy));
        assert!(base > 0, "the classic blob painted nothing ({base})");
        assert!(
            big > base,
            "the worthy blob covered no more stone ({big} vs {base})"
        );
    }

    // --- W4-c29: the gem-moss fidelity witnesses (written NOT run) ----------

    /// W4-c29: `Spread.Gem`'s duplicate-visit redraw (`WorldGen.cs:3617`) —
    /// the C#'s `hashSet.Add` ignores its return, so a cell two open
    /// neighbours enqueue in one wave is processed twice and the second
    /// visit's solid branch re-draws the `randGemTile` conversions. The a==b
    /// replay pins the tile field AND the stream state; the exposed gems
    /// land only on rim-adjacent cells (the 178 anchor gate), never in the
    /// deep interior the former ungated write filled.
    #[test]
    fn w4_c29_spread_gem_duplicate_visits_redraw_deterministically() {
        let build = |seed: u64| {
            let mut w = flat_world(600, 300, 40);
            carve_pocket(&mut w, 290, 110, 30, 30);
            let mut rng = Rng(seed);
            let gem = [true, false, true, false, false, false];
            spread_gem(&mut w, &mut rng, 305, 125, &gem);
            (w, rng.0)
        };
        let (wa, state_a) = build(0x6E6);
        let (wb, state_b) = build(0x6E6);
        assert_eq!(wa.tiles, wb.tiles, "the tile field replays");
        assert_eq!(state_a, state_b, "the stream state replays");
        // Every exposed gem sits next to a solid cell (the anchor gate); the
        // deep interior (305, 125) carries none.
        assert!(wa.tile(305, 125).is_some_and(|t| t.kind != TILE_EXPOSED_GEMS),
            "the deep interior exposes no gem");
        for y in 111..139 {
            for x in 291..319 {
                if wa.tile(x, y).is_some_and(|t| t.active() && t.kind == TILE_EXPOSED_GEMS) {
                    let anchored = [(-1, 0), (1, 0), (0, -1), (0, 1)].iter().any(|&(dx, dy)| {
                        wa.tile(x + dx, y + dy)
                            .is_some_and(|t| t.active() && TILE_SOLID.has(t.kind))
                    });
                    assert!(anchored, "the gem at ({x},{y}) has no solid anchor");
                }
            }
        }
    }

    /// W4-c29: the 178 `PlaceTile` arm (`WorldGen.cs:59691-59701`) — the
    /// multi-directional anchor gate refuses a deep-interior cell (no solid
    /// neighbour) and accepts a rim-adjacent one; the accepted write draws
    /// the `Next(3)*18` frame row. The gate's priority pick is below(0) →
    /// left(2) → right(3) → above(1) (`:60345-60364`).
    #[test]
    fn w4_c29_the_exposed_gem_anchor_gate_and_frame_row() {
        let mut w = flat_world(600, 300, 40);
        carve_pocket(&mut w, 290, 110, 30, 30);
        // Deep interior: no solid neighbour within the four faces.
        let mut rng = Rng(7);
        let before = rng.0;
        assert!(!place_exposed_gem(&mut w, &mut rng, 305, 125, 2),
            "the deep interior has no anchor");
        assert_eq!(rng.0, before, "the refused gate draws nothing");
        // Rim-adjacent: the cell just inside the pocket's floor has the
        // solid floor below (direction 1 → frame direction 0).
        assert!(place_exposed_gem(&mut w, &mut rng, 305, 139, 2),
            "the floor-adjacent cell anchors");
        let t = w.tile(305, 139).unwrap();
        assert_eq!(t.kind, TILE_EXPOSED_GEMS);
        assert_eq!(t.frame_x, 36, "style 2 → frameX 36");
        assert!(matches!(t.frame_y, 0 | 18 | 36), "the Next(3) row {}", t.frame_y);
        assert_ne!(rng.0, before, "the accepted write drew the frame row");
        // The anchor priority: a cell with BOTH a floor below and a ceiling
        // above takes the below anchor (frame direction 0) — the gate's
        // first pick. A one-tall corridor cell qualifies on both faces.
        let mut c = flat_world(600, 300, 40);
        carve_pocket(&mut c, 290, 125, 30, 1);
        assert_eq!(
            check_and_adjust_multi_directional_tile(&c, 305, 125),
            Some(0),
            "the below anchor wins the priority pick"
        );
        // A boulder neighbour is not an anchor (`Sets.Boulders`, `:60299`).
        let mut b = flat_world(600, 300, 40);
        carve_pocket(&mut b, 290, 125, 30, 1);
        let seat = b.index(305, 126);
        b.tiles[seat].kind = 138; // the boulder
        assert_eq!(
            check_and_adjust_multi_directional_tile(&b, 305, 125),
            Some(1),
            "the boulder floor is refused, the ceiling above anchors"
        );
    }

    /// W4-c29: `neonMossBiome`'s classic clamps (`WorldGen.cs:9843-9858`) —
    /// the rock-line clamp rides the ellipse's TOP edge and SETS the drift
    /// to +5.0; the maxY clamp rides the BOTTOM edge and SETS -5.0. A blob
    /// anchored above the rock line is pushed DOWN (the top-edge clamp), so
    /// it paints stone below the rock line, never above it.
    #[test]
    fn w4_c29_the_neon_moss_blob_stays_under_the_rock_line() {
        let mut w = flat_world(2000, 600, 60);
        // The rock line sits at row 300 (flat_world: surface 60, half the
        // remaining height); anchor the blob well ABOVE it.
        let rock = w.rock_layer as i32;
        carve_pocket(&mut w, 900, rock - 80, 220, 200);
        neon_moss_biome(&mut w, &mut Rng(0x4E0A), 1000, rock - 60, 520, NEON_MOSS[0]);
        let painted_above = (0..rock).any(|y| {
            (0..w.width).any(|x| {
                w.tile(x, y).is_some_and(|t| t.active() && t.kind == NEON_MOSS[0])
            })
        });
        assert!(!painted_above, "no neon moss above the rock line");
        let painted_below = (rock..w.height).any(|y| {
            (0..w.width).any(|x| {
                w.tile(x, y).is_some_and(|t| t.active() && t.kind == NEON_MOSS[0])
            })
        });
        assert!(painted_below, "the blob painted under the rock line");
    }

    /// W4-c29: the moss pass's arm A reject ledger (`WorldGen.cs:17554-
    /// 17566`) — a rejected anchor bumps the consecutive-reject streak and
    /// only advances the placed count once the streak passes `maxTilesX`;
    /// an accepted anchor resets the streak. On a pocket-free flat world
    /// every anchor is accepted (no exclusion kind in the scan box), so the
    /// pass places exactly `maxTilesX / 2100` biomes and the a==b replay
    /// holds. The dungeon-brick exclusion (`Main.tileDungeon`) rejects an
    /// anchor whose scan box carries a brick.
    #[test]
    fn w4_c29_the_moss_arm_a_ledger_and_dungeon_exclusion() {
        // The exclusion scan rejects a dungeon brick in the 101x101 box.
        let mut w = flat_world(2000, 600, 60);
        let rock = w.rock_layer as i32;
        let lava = 520i32;
        // Plant a blue dungeon brick at the centre of the only anchor band.
        let (ax, ay) = (1000, (rock + 40 + lava - 40) / 2);
        let idx = w.index(ax, ay);
        w.tiles[idx].kind = 41;
        w.tiles[idx].flags |= FLAG_ACTIVE;
        // The scan box around (ax, ay) carries the brick → rejected.
        let rejected = ((ax - 50)..=(ax + 50)).any(|i| {
            ((ay - 50)..=(ay + 50)).any(|j| {
                w.tile(i, j).is_some_and(|t| {
                    t.active() && TILE_DUNGEON.contains(&t.kind)
                })
            })
        });
        assert!(rejected, "the dungeon brick rejects the anchor");
        // The a==b replay of the whole moss pass on a classic world.
        let build = |seed: u64| {
            let mut w = flat_world(600, 700, 40);
            carve_pocket(&mut w, 300, 520, 28, 28);
            let mut rng = Rng(seed);
            let rock = w.rock_layer;
            pass_moss(&mut w, &mut rng, rock, 420.0, 530.0, 480);
            (w.tiles.clone(), rng.0)
        };
        let (ta, sa) = build(11);
        let (tb, sb) = build(11);
        assert_eq!(ta, tb, "the moss pass tile field replays");
        assert_eq!(sa, sb, "the moss pass stream state replays");
    }

    /// W4-c29: arm F walks the map LIVE column-major over `Main.tileMoss`
    /// (`WorldGen.cs:17659-17698`, the set carries 627 `Main.cs:7031`) — a
    /// rainbow-moss 627 tile spreads onto its stone neighbours exactly like
    /// the five classic mosses. The fixture gives the target's 3x3 box one
    /// air cell (the `spread_grass` enclosure gate refuses a fully-solid
    /// box, `:75186-75201`).
    #[test]
    fn w4_c29_the_moss_arm_f_sweeps_the_rainbow_moss_live() {
        let mut w = flat_world(600, 300, 40);
        // The rainbow-moss seed in the stone field.
        let idx = w.index(300, 150);
        w.tiles[idx].kind = 627;
        w.tiles[idx].flags |= FLAG_ACTIVE;
        // One air cell inside the left neighbour's 3x3 box.
        let air = w.index(298, 149);
        w.tiles[air].flags &= !FLAG_ACTIVE;
        // Arm F's sweep: the live column-major walk over the moss set.
        for x in 0..w.width {
            for y in 0..w.height {
                let k = w.tile(x, y).map(|t| (t.active(), t.kind));
                if let Some((true, kind)) = k {
                    if crate::tiledata::TILE_MOSS.has(kind) {
                        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                            spread_grass(&mut w, x + dx, y + dy, TILE_STONE, kind);
                        }
                    }
                }
            }
        }
        assert!(w.tile(299, 150).is_some_and(|t| t.active() && t.kind == 627),
            "the rainbow moss spread onto the stone neighbour");
        assert_eq!(w.tile(300, 150).unwrap().kind, 627, "the seed survives");
    }

    /// `place_exposed_gem` (`WorldGen.cs:59691-59701`): a free cell with a
    /// solid floor below takes the gem — frameX style*18, frameY a
    /// Next(3)*18 variant. (d105 witness re-pinned to the c29-fidelity
    /// rng-first signature at the batch-O keep-one.)
    #[test]
    fn place_exposed_gem_writes_the_style_and_variant() {
        let mut w = flat_world(200, 600, 60);
        // A free cell at (100, 200) with the stone floor at (100, 201).
        carve_pocket(&mut w, 100, 200, 1, 1);
        let mut rng = Rng(0x1234);
        place_exposed_gem(&mut w, &mut rng, 100, 200, 3);
        let t = w.tile(100, 200).unwrap();
        assert!(t.active() && t.kind == TILE_EXPOSED_GEMS);
        assert_eq!(t.frame_x, 3 * 18);
        assert!(t.frame_y == 0 || t.frame_y == 18 || t.frame_y == 36);
    }

    /// The multi-directional anchor (`:60314-60366`): a free cell with no
    /// solid neighbour refuses (no write, no frame_y draw).
    #[test]
    fn place_exposed_gem_refuses_without_an_anchor() {
        let mut w = flat_world(200, 600, 60);
        // A 3x3 air pocket — the centre cell has no solid neighbour.
        carve_pocket(&mut w, 99, 199, 3, 3);
        let mut rng = Rng(0x1234);
        let mut replay = Rng(0x1234);
        place_exposed_gem(&mut w, &mut rng, 100, 200, 0);
        assert!(!w.tile(100, 200).unwrap().active());
        assert_eq!(rng.next_u32(), replay.next_u32(), "no draw on refusal");
    }

    /// The full pass (`:20729-20768`): the scatter plants gems on the free
    /// underground cells; the weighted styles spread across the bands.
    #[test]
    fn the_random_gems_pass_scatters_the_gems() {
        let mut w = flat_world(2000, 600, 60);
        // Carve a large cavern so the scatter has free cells with anchors.
        carve_pocket(&mut w, 400, 250, 1200, 200);
        let mut rng = Rng(0x77);
        pass_random_gems(&mut w, &mut rng, 60.0, 200.0);
        let gems = w
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == TILE_EXPOSED_GEMS)
            .count();
        assert!(gems > 0, "the gems scattered ({gems})");
        // Every gem frame is a style multiple of 18 with a Next(3) variant.
        for t in w.tiles.iter().filter(|t| t.active() && t.kind == TILE_EXPOSED_GEMS) {
            assert_eq!(t.frame_x % 18, 0);
            assert!(t.frame_y == 0 || t.frame_y == 18 || t.frame_y == 36);
        }
    }

    /// `place_long_moss` (`WorldGen.cs:59703-59720`): a free cell beside a
    /// solid moss tile takes the long moss (184), frameX 0, frameY a
    /// Next(3)*18 variant.
    #[test]
    fn place_long_moss_grows_beside_the_moss() {
        let mut w = flat_world(200, 600, 60);
        // A moss tile at (100, 200), a free cell at (101, 200).
        let i = w.index(100, 200);
        w.tiles[i].kind = MOSS_TILES[0];
        w.tiles[i].flags |= FLAG_ACTIVE;
        carve_pocket(&mut w, 101, 200, 1, 1);
        let mut rng = Rng(0x42);
        place_long_moss(&mut w, 101, 200, &mut rng);
        let t = w.tile(101, 200).unwrap();
        assert!(t.active() && t.kind == 184);
        assert_eq!(t.frame_x, 0);
        assert!(t.frame_y == 0 || t.frame_y == 18 || t.frame_y == 36);
    }

    /// The no-moss refusal: a free cell with only stone neighbours stays
    /// empty (no write, no draw).
    #[test]
    fn place_long_moss_refuses_without_moss() {
        let mut w = flat_world(200, 600, 60);
        carve_pocket(&mut w, 100, 200, 1, 1);
        let mut rng = Rng(0x42);
        let mut replay = Rng(0x42);
        place_long_moss(&mut w, 100, 200, &mut rng);
        assert!(!w.tile(100, 200).unwrap().active());
        assert_eq!(rng.next_u32(), replay.next_u32(), "no draw on refusal");
    }

    /// The full pass (`:20769-20812`): the long moss grows on the free
    /// neighbours of the moss tiles.
    #[test]
    fn the_moss_grass_pass_grows_the_long_moss() {
        let mut w = flat_world(400, 600, 60);
        // A moss patch with air around it.
        carve_pocket(&mut w, 100, 200, 60, 40);
        for x in 110..150 {
            for y in 210..230 {
                let i = w.index(x, y);
                w.tiles[i].kind = MOSS_TILES[(x + y) as usize % 5];
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let mut rng = Rng(0x99);
        pass_moss_grass(&mut w, &mut rng);
        let long_moss = w
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == 184)
            .count();
        assert!(long_moss > 0, "the long moss grew ({long_moss})");
    }
}
