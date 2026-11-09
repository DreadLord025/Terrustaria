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

use std::collections::HashSet;

use super::structures::place_chest;
use super::{
    Rng, TILE_DIRT, TILE_SAND, TILE_SNOW, TILE_STONE, WALL_DIRT, WALL_SANDSTONE, WALL_STONE,
};
use crate::growth::spread_grass;
use crate::tile::{Tile, FLAG_ACTIVE};
use crate::tiledata::{TILE_SOLID, TILE_SOLID_TOP};
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
struct CaveCounts {
    tiles: i32,
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
/// (the fill carries no RNG, so the traversal order is free).
fn count_tiles(world: &World, x: i32, y: i32, lava_ok: bool, max: i32) -> CaveCounts {
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
            if !seen.insert(item) {
                continue;
            }
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
                    let style = rand_gem(rng, gem) as i32;
                    place_framed(world, item.0, item.1, TILE_EXPOSED_GEMS, style * 18);
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
fn place_tight(world: &mut World, rng: &mut Rng, x: i32, y: i32, spiders: bool) {
    let blocked = world
        .tile(x, y)
        .is_some_and(|t| t.liquid > 0 && t.liquid_kind() == crate::tile::LiquidKind::Shimmer);
    let larva = world.tile(x, y).is_some_and(|t| t.active() && t.kind == TILE_LARVA);
    if blocked || larva {
        return;
    }
    let prefer_small = rng.range(0, 2) == 0;
    let variation = rng.range(0, 3);
    place_unchecked_stalactite(world, x, y, prefer_small, variation, spiders);
}

/// `WorldGen.PlaceUncheckedStalactite` (`WorldGen.cs:38327-38641`) — the
/// support-type frame table. The port keeps the arms the port's world can
/// actually produce (spiders, the ice family, stone/moss, hive, the sandstone
/// pair, granite and marble); the paint copy is the port's `frame_x` write.
fn place_unchecked_stalactite(
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
        let is_moss = MOSS_TILES.contains(&support) || NEON_MOSS.contains(&support) || support == 381;
        if spiders {
            // `:38334-38350` — the spider stalactite, always two tiles tall.
            let frame = 108 + variation * 18;
            write_stalactite(world, x, y, frame, 0);
            write_stalactite(world, x, y + 1, frame, 18);
            return;
        }
        if matches!(support, TILE_SNOW | TILE_ICE | 163 | 164 | 200) {
            let frame = variation * 18;
            if prefer_small {
                write_stalactite(world, x, y, frame, 72);
            } else {
                write_stalactite(world, x, y, frame, 0);
                write_stalactite(world, x, y + 1, frame, 18);
            }
        } else if support == TILE_HIVE {
            // `:38518` — the hive stalactite is always the small one.
            write_stalactite(world, x, y, 162 + variation * 18, 72);
        } else if is_moss || matches!(support, TILE_STONE | TILE_PEARLSTONE | TILE_EBONSTONE | TILE_CRIMSTONE)
        {
            let frame = 54 + variation * 18;
            if prefer_small {
                write_stalactite(world, x, y, frame, 72);
            } else {
                write_stalactite(world, x, y, frame, 0);
                write_stalactite(world, x, y + 1, frame, 18);
            }
        } else if matches!(support, 396 | 397) {
            let frame = 378 + variation * 18;
            if prefer_small {
                write_stalactite(world, x, y, frame, 72);
            } else {
                write_stalactite(world, x, y, frame, 0);
                write_stalactite(world, x, y + 1, frame, 18);
            }
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
        let is_moss = MOSS_TILES.contains(&support) || NEON_MOSS.contains(&support) || support == 381;
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

/// `WorldGen.PlacePot` (`WorldGen.cs:54066-54114`) — the 2x2 seat validation
/// plus the style frame pair.
fn place_pot(world: &mut World, rng: &mut Rng, x: i32, y: i32, style: i32) -> bool {
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
            }
        }
    }
    true
}

/// `WorldGen.PlaceSmallPile` (`WorldGen.cs:46815-46873`).
fn place_small_pile(world: &mut World, x: i32, y: i32, pile_style: i32, pile_size: i32) -> bool {
    let frame_y = (pile_size * 18) as i16;
    let frame_x = (pile_style * 18) as i16;
    if world.tile(x, y).is_some_and(|t| t.liquid_kind() == crate::tile::LiquidKind::Lava) {
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

/// `Spread.Spider` (`WorldGen.cs:3682-3789`) — the BFS paints wall 62 over the
/// region and decorates the open cells: the buried chest (1/15 of the floored
/// cells), the pot, the stalactite on a ceiling, the large piles and the
/// small piles on a floor.
fn spread_spider(world: &mut World, rng: &mut Rng, x: i32, y: i32, hell_top: f64) {
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
                        // 15, trySlope: false, 0)` — the port places the chest
                        // tile; its stock rides M5-CHEST-gen.
                        let _ = place_chest(world, item.0, item.1);
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
    let mut steps = rng.range(30, 40) as i32;
    // `double num3 = maxTilesX / 4200.0; if (WorldGen.getGoodWorldGen) num3 *=
    // 1.5;` (`:9801-9805`) — the for-the-worthy neon moss blob is 1.5x wider
    // and takes 1.5x as many steps (`num`/`num2` are both scaled by `num3`).
    let scale = neon_moss_scale(world);
    radius *= scale;
    steps = (steps as f64 * scale) as i32;
    let rock_layer = world.rock_layer;
    let world_surface = world.world_surface;
    while steps > 0 {
        radius *= 0.98;
        steps -= 1;
        let (x0, y0) = (pos.0 - radius, pos.1 - radius);
        let (x1, y1) = (pos.0 + radius, pos.1 + radius);
        let xmin = (x0 as i32).max(1);
        let xmax = (x1 as i32).min(world.width - 1);
        let ymin = (y0 as i32).max(1);
        let ymax = (y1 as i32).min(world.height - 1);
        // The remix clamp is the port's rock/surface clamp: the C# forces the
        // ellipse under the rock line (or the surface) and nudges the drift.
        let mut ymax = ymax;
        if (ymax as f64) < rock_layer {
            ymax = rock_layer as i32;
            if vel.1 < 5.0 {
                vel.1 = vel.1 + 2.0;
            }
        }
        if (ymax as f64) > max_y as f64 {
            ymax = max_y;
            if vel.1 > -5.0 {
                vel.1 = vel.1 - 2.0;
            }
        }
        let r = radius * (1.0 + rng.next_f64() * 0.4 - 0.2);
        for k in xmin..=xmax {
            for l in ymin..=ymax {
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
        let _ = world_surface;
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

/// `Spider Caves` (`WorldGen.cs:17356-17427`).
pub(crate) fn pass_spider_caves(
    world: &mut World,
    rng: &mut Rng,
    rock_layer: f64,
    world_surface: f64,
    hell_top: f64,
) -> u32 {
    let mut placed = 0u32;
    let max_spider_count = 3500;
    let retries = world.width / 2;
    let attempts = spider_attempts(world);
    let (min_count, max_count) = (500, max_spider_count);
    for _ in 0..attempts {
        let mut tries = 0i32;
        let mut x = rng.range(200, world.width - 200);
        let mut y = rng.range(
            ((world_surface + rock_layer) / 2.0) as i32,
            world.height - 230,
        );
        let mut c = count_tiles(world, x, y, true, max_spider_count);
        while (c.tiles >= max_count || c.tiles < min_count) && tries < retries {
            tries += 1;
            x = rng.range(200, world.width - 200);
            y = rng.range(rock_layer as i32 + 30, world.height - 230);
            c = count_tiles(world, x, y, true, max_spider_count);
        }
        if tries < retries {
            spread_spider(world, rng, x, y, hell_top);
            placed += 1;
        }
    }
    placed
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
    // Arm A — the neon moss biomes.
    let mut wanted = (world.width / 2100) as i32;
    let mut placed_count = 0i32;
    while placed_count < wanted {
        let mut x = neon_moss_x(rng, world.width, world.seeds);
        let y = rng.range(rock_layer as i32 + 40, lava_line as i32 - 40);
        // The exclusion scan (`:17508-17573`).
        let mut rejected = false;
        for i in (x - 50)..=(x + 50) {
            for j in (y - 50)..=(y + 50) {
                if world.tile(i, j).is_some_and(|t| t.active())
                    && matches!(
                        world.tile(i, j).map(|t| t.kind).unwrap_or(0),
                        70 | 60 | TILE_MARBLE | TILE_GRANITE | TILE_ICE | TILE_SNOW | 396 | 397
                    )
                {
                    rejected = true;
                    break;
                }
            }
            if rejected {
                break;
            }
        }
        if rejected {
            placed_count += 1;
        } else {
            neon_moss_biome(world, rng, x, y, lava_line as i32, neon_moss_type);
            placed_count += 1;
        }
        let _ = &mut x;
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
    // Arm F — the closing spread sweep (`:17659-17698`).
    let moss_tiles: Vec<(i32, i32)> = world
        .tiles
        .iter()
        .enumerate()
        .filter(|(_, t)| t.active() && (MOSS_TILES.contains(&t.kind) || NEON_MOSS.contains(&t.kind) || t.kind == 381))
        .map(|(idx, _)| {
            let x = (idx as i32) % world.width;
            let y = (idx as i32) / world.width;
            (x, y)
        })
        .collect();
    for (x, y) in moss_tiles {
        let kind = world.tile(x, y).map(|t| t.kind).unwrap_or(0);
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            spread_grass(world, x + dx, y + dy, TILE_STONE, kind);
        }
    }
    caves
}

/// `Webs And Honey` (`WorldGen.cs:19995-20063`) — the pass that hangs cobweb
/// in the spider caves and converts the hive liquid to honey + speleothems.
pub(crate) fn pass_webs_and_honey(world: &mut World, rng: &mut Rng, _world_surface: f64) -> u32 {
    let mut webs = 0u32;
    for i in 100..world.width - 100 {
        for j in (world.world_surface as i32)..world.height - 100 {
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
                if let Some(t) = world.tile_mut(i, j) {
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

/// The dirt/stone ids the scatter arms read (kept for the const block to stay
/// a real dependency — the arms compare against `TILE_STONE` directly).
const _: (u16, u16, u16, u16) = (TILE_DIRT, TILE_STONE, TILE_SAND, WALL_SANDSTONE);

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
        spread_spider(&mut w, &mut rng, px + 10, py + 10, 180.0);
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
        let placed = pass_spider_caves(&mut w, &mut rng, rock, surface, 780.0);
        assert!(placed >= 1, "the pass placed a spider cave");
        let spider = w.tiles.iter().filter(|t| t.wall == WALL_SPIDER).count();
        assert!(spider >= 500, "the spider wall covers the cave ({spider})");
    }

    #[test]
    fn the_spider_decor_places_the_seat_families() {
        let (px, py, size) = pocket();
        // Sweep seeds until the decor families land, then pin their frames.
        for seed in 0..60u64 {
            let mut w = flat_world(600, 300, 40);
            carve_pocket(&mut w, px, py, size, size);
            spread_spider(&mut w, &mut Rng(seed), px + 10, py + 10, 180.0);
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
        // `Spread.Gem` exposes half the open cells (`Next(2) == 0`).
        let exposed = w.tiles.iter().filter(|t| t.kind == TILE_EXPOSED_GEMS).count();
        let cells = (size * size) as usize;
        assert!(exposed > cells / 4, "the exposed gems are the Next(2) arm ({exposed})");
        assert!(exposed <= cells, "never more than the pocket ({exposed})");
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
}
