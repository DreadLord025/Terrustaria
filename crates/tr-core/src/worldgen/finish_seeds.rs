//! The seed `Finish*` passes — the tail work `GenerateWorld` runs after the
//! whole chain (`WorldGen.cs:22431-22444`): `FinishDrunkGen` (`:26226`),
//! `FinishGetGoodWorld` (`:25799`), `FinishTenthAnniversaryWorld`
//! (`:24237`). Each is gated on its seed flag, so a classic world's
//! pipeline is unchanged (M5-SEED-1's seam — the pipeline hook is
//! `pipeline.rs`'s `step_seed_finish`).
//!
//! What each pass ports, arm by arm:
//!
//! - [`finish_drunk_gen`] — the dungeon palette re-roll (`:26228-26257`),
//!   the sweep over the dungeon bricks/walls with the jungle-grass mud
//!   smoothing (`:26261-26315`) and the `b = 16` temple/Lihzahrd band.
//! - [`finish_get_good_world`] — the cloud-row scan (`:25801-25810`), the
//!   same palette re-roll with its `color() == 14` arm (`:25813-25852`),
//!   the sweep at paint 17 (`:25854-25893`), the ash-lava arm (`:25898`),
//!   the surface-grass conversion with the plant kills (`:25907-25961`).
//! - [`finish_tenth_anniversary_world`] — the six `PaintThe*` sweeps
//!   (dungeon 24/24, living trees 12/12, temple 10/5, clouds 12/12, sand
//!   7/7, pyramids 12/12 — `:24245-24259`) and the 1-in-4 boulder
//!   138 → 665 conversion (`:24261-24277`).
//!
//! Deviations from the C#, all deliberate and pinned here rather than at
//! the call sites:
//!
//! - **No chest inventories in the port.** The chests place as tiles only
//!   (`structures.rs`'s `place_chest`; the stocks ride the storage task),
//!   so the `Main.chest[num3]` loops of `FinishDrunkGen` (`:26317-26346`,
//!   the 5001/678 add-ons) and `FinishGetGoodWorld` (`:25964-25983`, the
//!   678 drop) iterate nothing: no RNG draws, no writes.
//!   `ImproveAllChestContents` (`:24278-24281`, body `:24716-24725` —
//!   `GiveItemGoodPrefixes`) is the same loot-side gap and is deferred for
//!   the same reason.
//! - **`ConvertSkyIslands(2, growTrees: true)` is not ported** (`:24243`).
//!   It is a `WorldGen.Convert(k, l, 2)` Hallow conversion — the grass →
//!   109 / sand → 116 / stone → 117 tile table and the wall table
//!   (`:55098-55224`) — plus a `TryGrowingTreeByType(5, …)` roll and the
//!   plant `KillTile`s, not one of the `PaintThe*` repaints: the port has
//!   no `Convert` helper, and the M5-SEED-1 slice is the paint sweeps plus
//!   the 138 → 665 conversion. The anniversary sweeps therefore repaint
//!   whatever the earlier passes left, in the C#'s own order.
//! - The vanilla guards the port has no state for read as false:
//!   `Main.onlyShimmerOceanWorldsGeneration` (no shimmer-ocean mode) and
//!   `SecretSeed.noSurface.Enabled` (no `noSurface` flag in `SeedFlags`) —
//!   so `PaintTheLivingTrees` (`:24246-24249`), `PaintTheClouds`
//!   (`:24251-24254`) and `PaintThePyramids` (`:24256-24259`) all run.
//!   Every other seed static maps onto the port's own state: `GenOptions
//!   .flags` carries `notTheBees` / `drunkWorldGen` / `remixWorldGen` /
//!   `tenthAnniversaryWorldGen` / `getGoodWorldGen`, and `crimson` is
//!   `GenOptions.crimson`.

use crate::edit::kill_tile;
use crate::seeds::SeedFlags;
use crate::tile::{FLAG_ACTIVE, LiquidKind};
use crate::tiledata as td;
use crate::world::World;
use super::pipeline::Streams;
use super::{
    GenOptions, TILE_ASH, TILE_GRASS, TILE_HARDENED_SAND, TILE_ICE, TILE_SAND, TILE_SANDSTONE,
    TILE_SNOW,
};

// --- the tile/wall ids and sets the sweeps read ---------------------------

/// `Main.tileDungeon` (`Main.cs:7992-7997`) — the six dungeon bricks: the
/// blue/green/pink 41/43/44 (the three named below carry the per-brick
/// colour bands) plus the ancient 677/678/679.
const TILE_DUNGEON: [u16; 6] = [41, 43, 44, 677, 678, 679];
/// `Main.wallDungeon` (`Main.cs:10598-10606`) — the nine dungeon walls.
const WALL_DUNGEON: [u16; 9] = [7, 8, 9, 94, 95, 96, 97, 98, 99];
/// The per-brick colour bands of both Finish* palette re-rolls: the blue
/// 41 (`WorldGen.cs:26251-26254` / `:25843-25847`), the green 43
/// (`:26247-26250` / `:25838-25842`) and the pink 44 (`:26239-26246` /
/// `:25829-25837`).
const TILE_BLUE_BRICK: u16 = 41;
const TILE_GREEN_BRICK: u16 = 43;
const TILE_PINK_BRICK: u16 = 44;

/// `TileID.Mud` (`TileID.cs:747`) — the drunk smoothing's target
/// (`WorldGen.cs:26284`).
const TILE_MUD: u16 = 59;
/// `TileID.JungleGrass` (`:749`) — the drunk smoothing's seed tile and one
/// arm of the temple band in all three sweeps (`WorldGen.cs:26275`,
/// `:26294`, `:25873`, `:24540`).
const TILE_JUNGLE_GRASS: u16 = 60;
/// `TileID.JunglePlants` (`:751`) — the temple band's other arm, the
/// jungle plant standing on the Lihzahrd brick (`WorldGen.cs:24540`).
const TILE_JUNGLE_PLANTS: u16 = 61;
/// `TileID.Plants` (`:635`) and `Plants2` (`:675`) — the FTW grass arm's
/// two deactivate-only cases (`WorldGen.cs:25917-25924`).
const TILE_PLANTS: u16 = 3;
const TILE_PLANTS2: u16 = 73;
/// `TileID.Sunflower` (`:683`), `ImmatureHerbs` (`:793`), `MatureHerbs`
/// (`:795`), `SmallPiles` (`:999`), `LargePiles` (`:1001`), `LargePiles2`
/// (`:1003`), `DyePlants` (`:1083`), `VanityTreeSakura` (`:1821`) and
/// `VanityTreeYellowWillow` (`:1861`) — the FTW grass arm's nine
/// `KillTile` cases (`WorldGen.cs:25925-25961`).
const TILE_SUNFLOWER: u16 = 27;
const TILE_IMMATURE_HERBS: u16 = 82;
const TILE_MATURE_HERBS: u16 = 83;
const TILE_SMALL_PILES: u16 = 185;
const TILE_LARGE_PILES: u16 = 186;
const TILE_LARGE_PILES2: u16 = 187;
const TILE_DYE_PLANTS: u16 = 227;
const TILE_VANITY_SAKURA: u16 = 596;
const TILE_VANITY_WILLOW: u16 = 616;
/// `TileID.Platforms` (`:667`) — the anniversary dungeon sweep's frame
/// band 6-12 (`WorldGen.cs:24607-24614`).
const TILE_PLATFORM: u16 = 19;
/// `TileID.CorruptGrass` (`:675`) / `CrimsonGrass` (`:1027`) — the FTW
/// grass conversion's two targets (`WorldGen.cs:25909-25915`).
const TILE_CORRUPT_GRASS: u16 = 23;
const TILE_CRIMSON_GRASS: u16 = 199;
/// `TileID.Vines` (`:733`) / `VineFlowers` (`:1393`) — the living-tree
/// sweep's vine arm (`WorldGen.cs:24639-24643`).
const TILE_VINES: u16 = 52;
const TILE_VINE_FLOWERS: u16 = 382;
/// `TileID.LivingWood` (`:1011`) / `LeafBlock` (`:1013`) — the living-tree
/// sweep's two painted kinds (`WorldGen.cs:24637`, `:24643`, `:24652`).
const TILE_LIVING_WOOD: u16 = 191;
const TILE_LEAF_BLOCK: u16 = 192;
/// `TileID.Traps` (`:905`) — the framed trap brick, frames 1-4
/// (`WorldGen.cs:26298-26304`, `:25877-25882`, `:24544-24548`).
const TILE_TRAPS: u16 = 137;
/// `TileID.Boulder` (`TileID.cs:905`) — the 1-in-4 conversion's source
/// (`WorldGen.cs:24268`).
const TILE_BOULDER: u16 = 138;
/// `TileID.LifeCrystalBoulder` (`:1959`) — its target (`WorldGen.cs:24270`).
const TILE_LIFE_CRYSTAL_BOULDER: u16 = 665;
/// `TileID.LihzahrdBrick` (`:1081`) — the temple brick the temple band and
/// the wall-87 override hang off (`WorldGen.cs:26290`, `:25869`, `:24540`).
const TILE_LIHZAHRD_BRICK: u16 = 226;
/// `TileID.SandstoneBrick` (`:931`) — `PaintThePyramids`'s tile
/// (`WorldGen.cs:24516`).
const TILE_SANDSTONE_BRICK: u16 = 151;
/// `TileID.Stalactite` (`:959`) — `PaintTheSand`'s neighbour check
/// (`WorldGen.cs:24405-24415`).
const TILE_STALACTITE: u16 = 165;

/// `WallID.LihzahrdBrickUnsafe` (`WallID.cs:279`) — wall 87, repainted to
/// 25 (FTW) or to the band colour (drunk, temple) (`WorldGen.cs:25890`,
/// `:26311`, `:24571`).
const WALL_LIHZAHRD_BRICK: u16 = 87;
/// `WallID.Cloud` (`:251`) — `PaintTheClouds`'s wall (`WorldGen.cs:24586`).
const WALL_CLOUD: u16 = 73;
/// `WallID.LivingWoodUnsafe` (`:593`) — the living-tree sweep's wall
/// (`WorldGen.cs:24639`, `:24663`).
const WALL_LIVING_WOOD: u16 = 244;
/// `WallID.Sandstone` (`:479`) / `HardenedSand` (`:537`) —
/// `PaintTheSand`'s walls, both painted with the TILE colour
/// (`WorldGen.cs:24418-24420`).
const WALL_SANDSTONE: u16 = 187;
const WALL_HARDENED_SAND: u16 = 216;
/// `WallID.SandstoneBrick` (`:173`) — `PaintThePyramids`'s wall
/// (`WorldGen.cs:24407`).
const WALL_SANDSTONE_BRICK: u16 = 34;

// --- the pass bodies ------------------------------------------------------

/// `WorldGen.FinishDrunkGen` (`WorldGen.cs:26226-26348`): the drunk world's
/// whole-map repaint — the dungeon's monochrome re-roll, the jungle-grass
/// mud smoothing, the temple/Lihzahrd band at the fixed colour 16 and the
/// Lihzahrd wall-87 override.
pub(crate) fn finish_drunk_gen(world: &mut World, streams: &mut Streams, opts: &GenOptions) {
    let mut rng = streams.pass_rng("FinishDrunkGen");
    let (width, height) = (world.width, world.height);

    // The palette re-roll (`:26228-26257`): the opener `Next(13, 25)`, then
    // one re-roll per dungeon brick of the full-world column-major scan.
    // The C#'s `flag`/`break` pair (`:26231`, `:26256`) is dead code —
    // `flag` is never set inside this loop — so the scan runs the whole map
    // and `color` leaves it on the LAST brick's roll: the sweep below paints
    // every dungeon brick with that single colour (a monochrome dungeon).
    let mut color = rng.range(13, 25) as u8;
    for x in 0..width {
        for y in 0..height {
            let Some(t) = world.tile(x, y) else {
                continue;
            };
            if !t.active() || !TILE_DUNGEON.contains(&t.kind) {
                continue;
            }
            if t.kind == TILE_PINK_BRICK {
                color = rng.range(13, 15) as u8;
                if rng.range(0, 2) == 0 {
                    color = rng.range(23, 25) as u8;
                }
            }
            if t.kind == TILE_GREEN_BRICK {
                color = rng.range(15, 19) as u8;
            }
            if t.kind == TILE_BLUE_BRICK {
                color = rng.range(19, 23) as u8;
            }
        }
    }

    // The fixed band colour (`:26229`) — the temple/Lihzahrd tiles and the
    // wall-87 override both take it.
    let b = 16u8;
    for k in 10..width - 10 {
        for l in 10..height - 10 {
            let i = world.index(k, l);
            let (kind, active, wall, frame_y) = {
                let t = &world.tiles[i];
                (t.kind, t.active(), t.wall, t.frame_y)
            };
            if active && (TILE_DUNGEON.contains(&kind) || td::SET_CRACKED_BRICKS.has(kind)) {
                world.tiles[i].set_paint(color);
            }
            if WALL_DUNGEON.contains(&wall) {
                world.set_wall_paint(k, l, 25);
            }
            if active {
                // The jungle-grass mud smoothing (`:26275-26286`): the snow
                // 147 and ice 161 cells of the 3x3 around a jungle-grass 60
                // tile become mud 59.
                if kind == TILE_JUNGLE_GRASS {
                    for m in k - 1..=k + 1 {
                        for n in l - 1..=l + 1 {
                            let Some(u) = world.tile(m, n) else {
                                continue;
                            };
                            if u.kind == TILE_SNOW || u.kind == TILE_ICE {
                                let i = world.index(m, n);
                                world.tiles[i].kind = TILE_MUD;
                            }
                        }
                    }
                }
                // The temple band (`:26288-26309`): the Lihzahrd brick 226,
                // the jungle grass 60 standing on it, and the trap frames
                // 1-4, all at the fixed colour — gated on not-the-bees.
                let mut flag2 = false;
                if kind == TILE_LIHZAHRD_BRICK {
                    flag2 = true;
                }
                if kind == TILE_JUNGLE_GRASS
                    && world
                        .tile(k, l + 1)
                        .is_some_and(|u| u.kind == TILE_LIHZAHRD_BRICK)
                {
                    flag2 = true;
                }
                if kind == TILE_TRAPS && (1..=4).contains(&(frame_y as i32 / 18)) {
                    flag2 = true;
                }
                if flag2 && !opts.flags.contains(SeedFlags::NOT_THE_BEES) {
                    world.tiles[i].set_paint(b);
                }
            }
            // The Lihzahrd wall override (`:26311-26314`) sits OUTSIDE the
            // active block — an inactive cell with wall 87 repaints too.
            if wall == WALL_LIHZAHRD_BRICK {
                world.set_wall_paint(k, l, b);
            }
        }
    }
    // The chest add-ons (`:26317-26346`) port to nothing — the module note
    // records why (the port's chests carry no inventories yet).
}

/// `WorldGen.FinishGetGoodWorld` (`WorldGen.cs:25799-25983`): the For The
/// Worthy tail — the same dungeon palette re-roll (with its
/// `color() == 14` arm), the sweep at paint 17, the ash-lava arm and the
/// surface-grass conversion with its plant kills.
pub(crate) fn finish_get_good_world(world: &mut World, streams: &mut Streams, opts: &GenOptions) {
    let mut rng = streams.pass_rng("FinishGetGoodWorld");
    let (width, height) = (world.width, world.height);

    // The cloud-row scan (`:25801-25810`): vanilla walks the rows under the
    // surface and keeps the LAST row that holds a cloud (the inner `break`
    // only leaves the column loop), so `num` is the deepest cloud row below
    // the surface — the conversion arm below only rewrites the grass ABOVE
    // that line (`n < num`).
    let mut num = 0i32;
    let mut i = 20i32;
    while (i as f64) < world.world_surface {
        for j in 20..width - 20 {
            if world
                .tile(j, i)
                .is_some_and(|t| t.active() && td::SET_CLOUDS.has(t.kind))
            {
                num = i;
                break;
            }
        }
        i += 1;
    }

    // The palette re-roll (`:25813-25852`): the opener, then the column
    // scan that stops at the first column holding a dungeon brick — the
    // `flag`/`break` pair here is live code, unlike the drunk twin. The
    // `color() == 14` arm pins `b = 14` and skips the type rolls.
    let mut b = rng.range(13, 25) as u8;
    for k in 0..width {
        let mut flag = false;
        for l in 0..height {
            let (kind, active, paint) = {
                let t = &world.tiles[world.index(k, l)];
                (t.kind, t.active(), t.paint())
            };
            if !active || !TILE_DUNGEON.contains(&kind) {
                continue;
            }
            if paint == 14 {
                flag = true;
                b = 14;
                continue;
            }
            if kind == TILE_PINK_BRICK {
                flag = true;
                b = rng.range(13, 15) as u8;
                if rng.range(0, 2) == 0 {
                    b = rng.range(23, 25) as u8;
                }
            }
            if kind == TILE_GREEN_BRICK {
                flag = true;
                b = rng.range(15, 19) as u8;
            }
            if kind == TILE_BLUE_BRICK {
                flag = true;
                b = rng.range(19, 23) as u8;
            }
        }
        if flag {
            break;
        }
    }

    for m in 10..width - 10 {
        for n in 10..height - 10 {
            let i = world.index(m, n);
            let (kind, active, wall, frame_y) = {
                let t = &world.tiles[i];
                (t.kind, t.active(), t.wall, t.frame_y)
            };
            if active && (TILE_DUNGEON.contains(&kind) || td::SET_CRACKED_BRICKS.has(kind)) {
                world.tiles[i].set_paint(b);
            }
            // `:25862-25865` — the dungeon walls keep the drunk pass's own
            // paint (it ran first) and not-the-bees worlds skip them.
            if WALL_DUNGEON.contains(&wall)
                && !opts.flags.contains(SeedFlags::DRUNK)
                && !opts.flags.contains(SeedFlags::NOT_THE_BEES)
            {
                world.set_wall_paint(m, n, b);
            }
            if active {
                // The temple band (`:25866-25889`) — paint 17, the same
                // three tile arms as the drunk pass.
                let mut flag2 = false;
                if kind == TILE_LIHZAHRD_BRICK {
                    flag2 = true;
                }
                if kind == TILE_JUNGLE_GRASS
                    && world
                        .tile(m, n + 1)
                        .is_some_and(|u| u.kind == TILE_LIHZAHRD_BRICK)
                {
                    flag2 = true;
                }
                if kind == TILE_TRAPS && (1..=4).contains(&(frame_y as i32 / 18)) {
                    flag2 = true;
                }
                if flag2 && !opts.flags.contains(SeedFlags::NOT_THE_BEES) {
                    world.tiles[i].set_paint(17);
                }
            }
            if wall == WALL_LIHZAHRD_BRICK {
                world.set_wall_paint(m, n, 25);
            }
            // The two world-shaping arms are active-only (`:25894`).
            if !active {
                continue;
            }
            // The ash-lava arm (`:25898-25906`): an ash tile whose 1-in-15
            // roll fires floods itself with lava — and deactivates itself
            // when the tile ABOVE it is ash too (the C# deactivates
            // `Main.tile[m, n]`, the ash cell itself, not the neighbour).
            if !opts.flags.contains(SeedFlags::REMIX) && kind == TILE_ASH && rng.range(0, 15) == 0 {
                if world.tile(m, n - 1).is_some_and(|u| u.kind == TILE_ASH) {
                    if let Some(t) = world.tile_mut(m, n) {
                        t.flags &= !FLAG_ACTIVE;
                    }
                }
                if let Some(t) = world.tile_mut(m, n) {
                    t.liquid = u8::MAX;
                    t.set_liquid_kind(LiquidKind::Lava);
                }
            }
            // The surface-grass conversion (`:25907-25961`).
            if !opts.flags.contains(SeedFlags::TENTH_ANNIVERSARY)
                && !opts.flags.contains(SeedFlags::NOT_THE_BEES)
                && n < num
                && kind == TILE_GRASS
            {
                let above = world.tile(m, n - 1).map(|u| u.kind).unwrap_or(0);
                if let Some(t) = world.tile_mut(m, n) {
                    t.kind = if opts.crimson {
                        TILE_CRIMSON_GRASS
                    } else {
                        TILE_CORRUPT_GRASS
                    };
                }
                // The two deactivate-only cases and the nine `KillTile`
                // cases — the above tile's kind decides, and `KillTile`
                // keeps it, so the cases stay mutually exclusive.
                if above == TILE_PLANTS || above == TILE_PLANTS2 {
                    if let Some(t) = world.tile_mut(m, n - 1) {
                        t.flags &= !FLAG_ACTIVE;
                    }
                }
                if matches!(
                    above,
                    TILE_SUNFLOWER
                        | TILE_VANITY_SAKURA
                        | TILE_VANITY_WILLOW
                        | TILE_IMMATURE_HERBS
                        | TILE_MATURE_HERBS
                        | TILE_SMALL_PILES
                        | TILE_LARGE_PILES
                        | TILE_LARGE_PILES2
                        | TILE_DYE_PLANTS
                ) {
                    kill_tile(world, m, n - 1, true);
                }
            }
        }
    }
    // The chest sweep (`:25964-25983`) ports to nothing — see the module
    // note on the port's inventory-free chests.
}

/// `WorldGen.FinishTenthAnniversaryWorld` (`WorldGen.cs:24237-24282`): the
/// tenth-anniversary repaint — the six `PaintThe*` sweeps and the 1-in-4
/// boulder 138 → 665 conversion.
pub(crate) fn finish_tenth_anniversary_world(
    world: &mut World,
    streams: &mut Streams,
    opts: &GenOptions,
) {
    let mut rng = streams.pass_rng("FinishTenthAnniversaryWorld");
    let (width, height) = (world.width, world.height);

    // The guard (`:24239`) — the C#'s
    // `(remix || (!notTheBees && !dontStarve)) && !remix` wording reduces
    // to `!remix && !notTheBees && !dontStarve`.
    if !opts.flags.contains(SeedFlags::REMIX)
        && !opts.flags.contains(SeedFlags::NOT_THE_BEES)
        && !opts.flags.contains(SeedFlags::DONT_STARVE)
    {
        // `ConvertSkyIslands(2, growTrees: true)` (`:24241-24244`) is not
        // ported — the module note records why (a `WorldGen.Convert` Hallow
        // sweep, not one of the `PaintThe*` repaints).
        paint_the_dungeon(world, 24, 24); // :24245
        paint_the_living_trees(world, 12, 12); // :24246-24249
        paint_the_temple(world, 10, 5, opts); // :24250
        paint_the_clouds(world, 12, 12); // :24251-24254
        paint_the_sand(world, 7, 7); // :24255
        paint_the_pyramids(world, 12, 12); // :24256-24259
    }

    // The 1-in-4 boulder conversion (`:24261-24277`) — the C#'s `&&` chain
    // draws `Next(4)` first, so the port draws for every cell of the
    // interior whether or not the tile checks pass.
    if !opts.flags.contains(SeedFlags::FOR_THE_WORTHY) {
        for x in 50..width - 50 {
            for y in 50..height - 50 {
                if rng.range(0, 4) != 0 {
                    continue;
                }
                let i = world.index(x, y);
                let (kind, active, frame_x, frame_y) = {
                    let t = &world.tiles[i];
                    (t.kind, t.active(), t.frame_x, t.frame_y)
                };
                if active && kind == TILE_BOULDER && frame_x == 0 && frame_y == 0 {
                    for &(dx, dy) in &[(0i32, 0i32), (1, 0), (0, 1), (1, 1)] {
                        if let Some(t) = world.tile_mut(x + dx, y + dy) {
                            t.kind = TILE_LIFE_CRYSTAL_BOULDER;
                        }
                    }
                }
            }
        }
    }
    // `ImproveAllChestContents` (`:24278-24281`) — deferred: loot-side, and
    // the port has no chest inventories to prefix.
}

// --- the PaintThe* sweeps ------------------------------------------------

/// `WorldGen.PaintTheDungeon` (`WorldGen.cs:24594-24622`): the dungeon
/// bricks + the cracked ones and every dungeon wall, plus the platform
/// frames 6-12 band.
fn paint_the_dungeon(world: &mut World, tile_paint: u8, wall_paint: u8) {
    for x in 0..world.width {
        for y in 0..world.height {
            let i = world.index(x, y);
            let (kind, active, wall, frame_y) = {
                let t = &world.tiles[i];
                (t.kind, t.active(), t.wall, t.frame_y)
            };
            if active {
                if TILE_DUNGEON.contains(&kind) || td::SET_CRACKED_BRICKS.has(kind) {
                    world.tiles[i].set_paint(tile_paint);
                }
                if kind == TILE_PLATFORM && (6..=12).contains(&(frame_y as i32 / 18)) {
                    world.tiles[i].set_paint(tile_paint);
                }
            }
            if WALL_DUNGEON.contains(&wall) {
                world.set_wall_paint(x, y, wall_paint);
            }
        }
    }
}

/// `WorldGen.GetVineTop` (`WorldGen.cs:63421-63434`): the first non-vine
/// cell above a vine column (the walk stops at row 20). The port's
/// out-of-bounds read is air, exactly like `Framing.GetTileSafely`, so a
/// vine column running to the map edge stops at the edge.
fn vine_top(world: &World, x: i32, y: i32) -> (i32, i32) {
    let vx = x;
    let mut vy = y;
    if world
        .tile(vx, vy)
        .is_some_and(|t| td::SET_IS_VINE.has(t.kind))
    {
        while vy > 20
            && world
                .tile(vx, vy)
                .is_some_and(|t| t.active() && td::SET_IS_VINE.has(t.kind))
        {
            vy -= 1;
        }
    }
    (vx, vy)
}

/// `WorldGen.PaintTheLivingTrees` (`WorldGen.cs:24624-24673`): the
/// living-wood wall, the wood 191 and leaf 192, the vines whose top cell is
/// a leaf block and the 187 run walking down to a leaf block.
fn paint_the_living_trees(world: &mut World, tree_paint: u8, wall_paint: u8) {
    for x in 0..world.width {
        for y in 0..world.height {
            let i = world.index(x, y);
            let (kind, active, wall) = {
                let t = &world.tiles[i];
                (t.kind, t.active(), t.wall)
            };
            if active {
                // The chain order is the C#'s (`:24636-24652`) — the wall
                // arm wins over the tile arms.
                if wall == WALL_LIVING_WOOD {
                    world.tiles[i].set_paint(tree_paint);
                } else if kind == TILE_LEAF_BLOCK || kind == TILE_LIVING_WOOD {
                    world.tiles[i].set_paint(tree_paint);
                } else if kind == TILE_VINES || kind == TILE_VINE_FLOWERS {
                    let (tx, ty) = vine_top(world, x, y);
                    if world.tile(tx, ty).is_some_and(|u| u.kind == TILE_LEAF_BLOCK) {
                        world.tiles[i].set_paint(tree_paint);
                    }
                } else if kind == TILE_LARGE_PILES2 {
                    // The walk down the 187 run: the first non-187 cell
                    // below decides (out of bounds reads are air).
                    let mut num = 0i32;
                    let mut below = kind;
                    while below == TILE_LARGE_PILES2 {
                        num += 1;
                        below = world.tile(x, y + num).map(|u| u.kind).unwrap_or(0);
                    }
                    if below == TILE_LEAF_BLOCK {
                        world.tiles[i].set_paint(tree_paint);
                    }
                }
            }
            if wall == WALL_LIVING_WOOD {
                world.set_wall_paint(x, y, wall_paint);
            }
        }
    }
}

/// `WorldGen.PaintTheTemple` (`WorldGen.cs:24529-24573`): the Lihzahrd
/// brick 226, the jungle plants 61 standing on it and the trap frames 1-4,
/// plus the Lihzahrd wall 87. The C#'s not-the-bees branch paints 0; the
/// whole sweep is already gated on it (`:24239`), so the branch is dead
/// inside the port's pipeline and is kept only for fidelity.
fn paint_the_temple(world: &mut World, tile_paint: u8, wall_paint: u8, opts: &GenOptions) {
    for x in 10..world.width - 10 {
        for y in 10..world.height - 10 {
            let i = world.index(x, y);
            let (kind, active, wall, frame_y) = {
                let t = &world.tiles[i];
                (t.kind, t.active(), t.wall, t.frame_y)
            };
            if active {
                let mut flag = false;
                if kind == TILE_LIHZAHRD_BRICK {
                    flag = true;
                }
                if kind == TILE_JUNGLE_PLANTS
                    && world
                        .tile(x, y + 1)
                        .is_some_and(|u| u.kind == TILE_LIHZAHRD_BRICK)
                {
                    flag = true;
                }
                if kind == TILE_TRAPS && (1..=4).contains(&(frame_y as i32 / 18)) {
                    flag = true;
                }
                if flag {
                    let paint = if opts.flags.contains(SeedFlags::NOT_THE_BEES) {
                        0
                    } else {
                        tile_paint
                    };
                    world.tiles[i].set_paint(paint);
                }
            }
            if wall == WALL_LIHZAHRD_BRICK {
                world.set_wall_paint(x, y, wall_paint);
            }
        }
    }
}

/// `WorldGen.PaintTheClouds` (`WorldGen.cs:24575-24593`): the cloud tiles
/// and the cloud walls.
fn paint_the_clouds(world: &mut World, tile_paint: u8, wall_paint: u8) {
    for x in 0..world.width {
        for y in 0..world.height {
            let i = world.index(x, y);
            let (kind, active, wall) = {
                let t = &world.tiles[i];
                (t.kind, t.active(), t.wall)
            };
            if active && td::SET_CLOUDS.has(kind) {
                world.tiles[i].set_paint(tile_paint);
            }
            if wall == WALL_CLOUD {
                world.set_wall_paint(x, y, wall_paint);
            }
        }
    }
}

/// `WorldGen.PaintTheSand` (`WorldGen.cs:24385-24420`): the sand triple and
/// — below the surface — their stalactite/pile neighbours, plus the
/// sandstone/hardened-sand walls. `_wall_paint` is the C#'s unused
/// parameter: every wall takes the TILE colour.
fn paint_the_sand(world: &mut World, tile_paint: u8, _wall_paint: u8) {
    for x in 20..world.width - 20 {
        for y in 20..world.height - 20 {
            let i = world.index(x, y);
            let (kind, active, wall) = {
                let t = &world.tiles[i];
                (t.kind, t.active(), t.wall)
            };
            if active && (kind == TILE_SAND || kind == TILE_SANDSTONE || kind == TILE_HARDENED_SAND) {
                world.tiles[i].set_paint(tile_paint);
                if (y as f64) > world.world_surface {
                    let up1 = world.tile(x, y - 1).map(|u| u.kind).unwrap_or(0);
                    let up2 = world.tile(x, y - 2).map(|u| u.kind).unwrap_or(0);
                    let down1 = world.tile(x, y + 1).map(|u| u.kind).unwrap_or(0);
                    let down2 = world.tile(x, y + 2).map(|u| u.kind).unwrap_or(0);
                    let (i_up1, i_up2) = (world.index(x, y - 1), world.index(x, y - 2));
                    let (i_dn1, i_dn2) = (world.index(x, y + 1), world.index(x, y + 2));
                    if matches!(up1, 165 | 185 | 186 | 187) {
                        world.tiles[i_up1].set_paint(tile_paint);
                    }
                    if matches!(up2, 165 | 185 | 186 | 187) {
                        world.tiles[i_up2].set_paint(tile_paint);
                    }
                    if down1 == TILE_STALACTITE {
                        world.tiles[i_dn1].set_paint(tile_paint);
                    }
                    if down2 == TILE_STALACTITE {
                        world.tiles[i_dn2].set_paint(tile_paint);
                    }
                }
            }
            if wall == WALL_SANDSTONE || wall == WALL_HARDENED_SAND {
                world.set_wall_paint(x, y, tile_paint);
            }
        }
    }
}

/// `WorldGen.PaintThePyramids` (`WorldGen.cs:24510-24528`): the sandstone
/// brick 151 and the sandstone-brick wall 34.
fn paint_the_pyramids(world: &mut World, tile_paint: u8, wall_paint: u8) {
    for x in 0..world.width {
        for y in 0..world.height {
            let i = world.index(x, y);
            let (kind, active, wall) = {
                let t = &world.tiles[i];
                (t.kind, t.active(), t.wall)
            };
            if active && kind == TILE_SANDSTONE_BRICK {
                world.tiles[i].set_paint(tile_paint);
            }
            if wall == WALL_SANDSTONE_BRICK {
                world.set_wall_paint(x, y, wall_paint);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seeds::SeedFlags;
    use crate::tile::FLAG_ACTIVE;
    use crate::world::World;

    /// A small all-air test world with the surface row pinned.
    fn air(width: i32, height: i32, surface: f64) -> World {
        let mut world = World::new("t".into(), width, height);
        world.world_surface = surface;
        world
    }

    /// Places an active tile of `kind` with the given frame at `(x, y)`.
    fn put(world: &mut World, x: i32, y: i32, kind: u16) {
        let i = world.index(x, y);
        world.tiles[i].kind = kind;
        world.tiles[i].flags |= FLAG_ACTIVE;
    }

    /// Puts the wall `wall` at `(x, y)` (a wall exists on an inactive cell).
    fn wall(world: &mut World, x: i32, y: i32, wall: u16) {
        let i = world.index(x, y);
        world.tiles[i].wall = wall;
    }

    fn opts(flags: u16, crimson: bool) -> GenOptions {
        GenOptions {
            flags: SeedFlags::from(flags),
            crimson,
            ..Default::default()
        }
    }

    fn streams(seed: u64) -> Streams<'static> {
        Streams::new(seed)
    }

    /// The drunk world's dungeon is monochrome: the sweep paints every
    /// dungeon brick with the LAST re-roll's colour, whichever brick rolled
    /// it, and the fixed band colour takes over the temple/Lihzahrd side.
    #[test]
    fn drunk_dungeon_bricks_and_walls_carry_the_band_paint() {
        let mut world = air(200, 140, 100.0);
        put(&mut world, 100, 60, TILE_BLUE_BRICK);
        put(&mut world, 120, 70, TILE_PINK_BRICK);
        wall(&mut world, 120, 70, 7); // a dungeon wall rides the pink brick
        wall(&mut world, 130, 80, WALL_LIHZAHRD_BRICK); // inactive cell + wall 87
        put(&mut world, 140, 90, TILE_TRAPS);
        let i = world.index(140, 90);
        world.tiles[i].frame_y = 18; // frame 1
        put(&mut world, 150, 100, TILE_JUNGLE_GRASS);
        put(&mut world, 151, 100, TILE_SNOW);
        let flags = opts(SeedFlags::DRUNK, false);
        let streams = &mut streams(5);
        finish_drunk_gen(&mut world, streams, &flags);

        let blue = world.tiles[world.index(100, 60)].paint();
        let pink = world.tiles[world.index(120, 70)].paint();
        assert_eq!(blue, pink, "the drunk dungeon is one colour");
        assert!(
            (13..=24).contains(&blue),
            "dungeon paint {blue} is outside the 13..25 band"
        );
        assert_eq!(world.wall_paint(120, 70), 25, "wallDungeon paints 25");
        assert_eq!(
            world.wall_paint(130, 80),
            16,
            "the Lihzahrd wall 87 paints the band colour"
        );
        assert_eq!(
            world.tiles[world.index(140, 90)].paint(),
            16,
            "the trap frame-1 band paints 16"
        );
        assert_eq!(
            world.tiles[world.index(151, 100)].kind,
            TILE_MUD,
            "the snow neighbour of a jungle-grass tile turns to mud"
        );
    }

    /// The FTW surface conversion: grass above the cloud row becomes the
    /// evil grass (23 corruption / 199 crimson), its plant tiles above go
    /// with it, and grass below the cloud row is untouched.
    #[test]
    fn ftw_surface_grass_converts_above_the_cloud_row() {
        for (crimson, want) in [(false, TILE_CORRUPT_GRASS), (true, TILE_CRIMSON_GRASS)] {
            let mut world = air(200, 140, 70.0);
            put(&mut world, 100, 55, 189); // a cloud tile: `num` lands on 55
            put(&mut world, 50, 40, TILE_GRASS);
            put(&mut world, 50, 39, TILE_PLANTS);
            put(&mut world, 60, 40, TILE_GRASS);
            put(&mut world, 60, 39, TILE_SUNFLOWER);
            put(&mut world, 70, 65, TILE_GRASS); // below `num`: untouched
            let flags = opts(SeedFlags::FOR_THE_WORTHY, crimson);
            let streams = &mut streams(9);
            finish_get_good_world(&mut world, streams, &flags);

            assert_eq!(world.tiles[world.index(50, 40)].kind, want);
            assert!(
                !world.tiles[world.index(50, 39)].active(),
                "the plant above the converted grass deactivates"
            );
            assert_eq!(
                world.tiles[world.index(60, 40)].kind,
                want,
                "the sunflower arm converts the grass too"
            );
            assert!(
                !world.tiles[world.index(60, 39)].active(),
                "KillTile removed the sunflower above the converted grass"
            );
            assert_eq!(
                world.tiles[world.index(70, 65)].kind,
                TILE_GRASS,
                "grass below the cloud row stays grass"
            );
        }
    }

    /// The ash-lava arm: an ash tile whose 1-in-15 roll fires floods itself
    /// with lava (and stays ash); a bulk patch guarantees the roll fires at
    /// least once.
    #[test]
    fn ftw_ash_tiles_flood_with_lava() {
        let mut world = air(200, 140, 70.0);
        for x in 20..180 {
            for y in 20..60 {
                put(&mut world, x, y, TILE_ASH);
            }
        }
        let flags = opts(SeedFlags::FOR_THE_WORTHY, false);
        let streams = &mut streams(11);
        finish_get_good_world(&mut world, streams, &flags);

        let lava = world
            .tiles
            .iter()
            .filter(|t| t.liquid == u8::MAX)
            .count();
        assert!(lava > 0, "no ash tile took the 1-in-15 lava roll");
        for (idx, t) in world.tiles.iter().enumerate() {
            if t.liquid == u8::MAX {
                let x = (idx % world.width as usize) as i32;
                let y = (idx / world.width as usize) as i32;
                assert_eq!(t.kind, TILE_ASH, "lava flooded a non-ash tile at {x},{y}");
                assert_eq!(t.liquid_kind(), LiquidKind::Lava);
                // The ash under ash deactivates; the top row (air above)
                // stays active.
                let ash_above = world.tile(x, y - 1).is_some_and(|u| u.kind == TILE_ASH);
                assert_eq!(
                    t.active(),
                    !ash_above,
                    "the ash/lava activation at {x},{y} does not match the C# arm"
                );
            }
        }
    }

    /// The tenth-anniversary sweeps repaint each family with its own pair:
    /// the dungeon 24/24, the temple 10/5, the clouds 12/12, the sand 7/7
    /// and the pyramids 12/12 (tiles and walls alike).
    #[test]
    fn anniversary_sweeps_paint_each_family() {
        let mut world = air(200, 140, 100.0);
        put(&mut world, 100, 60, TILE_BLUE_BRICK);
        wall(&mut world, 100, 60, 7);
        put(&mut world, 110, 80, TILE_LIHZAHRD_BRICK);
        wall(&mut world, 110, 80, WALL_LIHZAHRD_BRICK);
        put(&mut world, 120, 100, TILE_SAND);
        wall(&mut world, 120, 100, WALL_SANDSTONE);
        put(&mut world, 130, 50, 189); // cloud tile
        wall(&mut world, 130, 50, WALL_CLOUD);
        put(&mut world, 140, 110, TILE_SANDSTONE_BRICK);
        wall(&mut world, 140, 110, WALL_SANDSTONE_BRICK);
        put(&mut world, 150, 60, TILE_LEAF_BLOCK);
        wall(&mut world, 150, 60, WALL_LIVING_WOOD);
        put(&mut world, 150, 61, TILE_VINES);
        put(&mut world, 150, 62, TILE_VINES);
        let flags = opts(SeedFlags::TENTH_ANNIVERSARY, false);
        let streams = &mut streams(13);
        finish_tenth_anniversary_world(&mut world, streams, &flags);

        assert_eq!(world.tiles[world.index(100, 60)].paint(), 24);
        assert_eq!(world.wall_paint(100, 60), 24);
        assert_eq!(world.tiles[world.index(110, 80)].paint(), 10);
        assert_eq!(world.wall_paint(110, 80), 5);
        assert_eq!(world.tiles[world.index(120, 100)].paint(), 7);
        assert_eq!(world.wall_paint(120, 100), 7);
        assert_eq!(world.tiles[world.index(130, 50)].paint(), 12);
        assert_eq!(world.wall_paint(130, 50), 12);
        assert_eq!(world.tiles[world.index(140, 110)].paint(), 12);
        assert_eq!(world.wall_paint(140, 110), 12);
        assert_eq!(world.tiles[world.index(150, 60)].paint(), 12);
        assert_eq!(world.wall_paint(150, 60), 12);
        // The vine arm: both vine cells walk up to the leaf block and take
        // the living-tree paint.
        assert_eq!(world.tiles[world.index(150, 61)].paint(), 12);
        assert_eq!(world.tiles[world.index(150, 62)].paint(), 12);
    }

    /// The 1-in-4 boulder conversion: every anchored 2x2 group either stays
    /// 138 or turns 665 wholesale, and a struck anchor never leaves its
    /// group half-converted.
    #[test]
    fn anniversary_converts_boulders_in_whole_2x2_groups() {
        let mut world = air(400, 200, 100.0);
        for bx in (60..340).step_by(4) {
            for by in (80..160).step_by(4) {
                for &(dx, dy) in &[(0i32, 0i32), (1, 0), (0, 1), (1, 1)] {
                    let i = world.index(bx + dx, by + dy);
                    world.tiles[i].kind = TILE_BOULDER;
                    world.tiles[i].flags |= FLAG_ACTIVE;
                    world.tiles[i].frame_x = (dx * 18) as i16;
                    world.tiles[i].frame_y = (dy * 18) as i16;
                }
            }
        }
        let flags = opts(SeedFlags::TENTH_ANNIVERSARY, false);
        let streams = &mut streams(17);
        finish_tenth_anniversary_world(&mut world, streams, &flags);

        let mut converted = 0usize;
        for bx in (60..340).step_by(4) {
            for by in (80..160).step_by(4) {
                let struck = world.tiles[world.index(bx, by)].kind == TILE_LIFE_CRYSTAL_BOULDER;
                if struck {
                    converted += 1;
                }
                // A group is all-or-nothing: the struck anchors take their
                // four cells, the untouched ones leave all four as boulders.
                for &(dx, dy) in &[(0i32, 0i32), (1, 0), (0, 1), (1, 1)] {
                    let kind = world.tiles[world.index(bx + dx, by + dy)].kind;
                    let want = if struck {
                        TILE_LIFE_CRYSTAL_BOULDER
                    } else {
                        TILE_BOULDER
                    };
                    assert_eq!(
                        kind, want,
                        "the group at {bx},{by} did not convert wholesale"
                    );
                }
            }
        }
        assert!(converted > 0, "no boulder group took the 1-in-4 roll");
    }
}
