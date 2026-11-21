//! The biome passes — the deterministic ports of
//! `GameContent/Generation/Biomes/*` and the `GenPassNameID` order
//! (`ID/GenPassNameID.cs`): the jungle (`JunglePass` — faithful since
//! M5-BIOME phase 3, with `MudCavesToJungleGrass` and `DirtToMud` beside
//! it), the glowing mushroom patches (`GlowingMushroomPatches`), the granite
//! and marble caves and the dungeon (`Dungeon`). The evil pass
//! (`CorruptionAndCrimson`) moved to its own module in M5-BIOME phase 4
//! (`super::evil` — the faithful port).
//!
//! Tile ids (`TileID.cs`): the jungle grass 60, the mud 59, the mushroom
//! grass 70, the corrupt grass 23, the ebonstone 25, the ebonsand 112, the
//! crimson grass 199, the crimstone 203, the crimsand 234, the ice 161, the
//! granite 368, the marble 367, the dungeon bricks 41/43/44, the gems
//! 63-68. Wall ids (`WallID.cs`): the jungle 64, the mud 15, the mushroom
//! 74, the granite 180, the marble 178, the ebonstone 3, the crimson 192,
//! the dungeon 7/8/9.
//!
//! Deviations (ANALYSIS "Biome passes (M1.4-WGEN-1)"): the C# passes are
//! shape-driven (`ShapeRunner`/`ShapeFloodFill` chains, `DungeonCrawler`);
//! the port reproduces the vanilla SHAPES with its own deterministic noise
//! and blob machinery — the exact tile-level layout is not 1:1 for the
//! mushroom pass; the jungle shrines, the temple, the pyramids and the
//! chests are the follow-up tasks; the hallow is the hardmode strip
//! (M5-HARDFORCE).

use super::mini_biomes::clear_tile_keep;
use super::ocean::reset_to_type;
use super::spider_gem_moss::{place_tight, place_unchecked_stalactite};
use super::tile_runner::{tile_runner, TileRunnerOpts};
use super::{Rng, TILE_DIRT, TILE_GRASS, TILE_SAND, TILE_SNOW, TILE_STONE, WALL_DIRT, WALL_STONE};
use crate::tile::{LiquidKind, Tile, FLAG_ACTIVE, FLAG_HALF_BRICK};
use crate::tile_light::FastRandom;
use crate::tiledata::{TILE_SOLID, TILE_SOLID_TOP};
use crate::world::World;

pub(crate) const TILE_MUD: u16 = 59;
pub(crate) const TILE_JUNGLE_GRASS: u16 = 60;
pub(crate) const TILE_MUSHROOM_GRASS: u16 = 70;
pub(crate) const TILE_CORRUPT_GRASS: u16 = 23;
pub(crate) const TILE_EBONSTONE: u16 = 25;
pub(crate) const TILE_EBONSAND: u16 = 112;
pub(crate) const TILE_CRIMSON_GRASS: u16 = 199;
pub(crate) const TILE_CRIMSTONE: u16 = 203;
pub(crate) const TILE_CRIMSAND: u16 = 234;
pub(crate) const TILE_GRANITE: u16 = 368;
pub(crate) const TILE_MARBLE: u16 = 367;
pub(crate) const TILE_DUNGEON_BRICK: u16 = 41;

pub(crate) const WALL_JUNGLE: u16 = 64;
pub(crate) const WALL_MUSHROOM: u16 = 74;
/// `WallID.MushroomUnsafe` (`ID/WallID.cs:265`): what `ShroomPatch` converts
/// an existing wall to inside the mud dome (`WorldGen.cs:79920`, `:79933`).
pub(crate) const WALL_MUSHROOM_UNSAFE: u16 = 80;
pub(crate) const WALL_GRANITE: u16 = 180;
pub(crate) const WALL_MARBLE: u16 = 178;
pub(crate) const WALL_EBONSTONE: u16 = 3;
pub(crate) const WALL_CRIMSON: u16 = 192;

/// The biome span decisions shared by the passes (the C# `GenVars` slice):
/// the jungle band on the dungeon-opposite side (re-derived from
/// `GenVars.jungle_origin_x` since M5-BIOME phase 3) and the dungeon at the
/// snow side's surface. The evil band retired with the shape-level
/// `pass_evil` in phase 4 — the faithful pass draws its own centres
/// (`evil::corruption_and_crimson`).
#[derive(Clone)]
pub(crate) struct BiomeSpans {
    pub(crate) jungle_start: i32,
    pub(crate) jungle_end: i32,
    pub(crate) dungeon_x: i32,
}

pub(crate) fn plan_spans(
    width: i32,
    surface: &[i32],
    left_beach_end: i32,
    right_beach_start: i32,
    jungle_origin_x: i32,
) -> BiomeSpans {
    // The faithful jungle band: `JunglePass` (`JunglePass.cs:24-62`) spreads
    // the mud around `jungleOriginX` — the blob walk drifts at most
    // ±(100+250+400)*ws and the body disc adds ±300*ws around the blob mean.
    // The interim band covers that reach (±700*ws) so the shape-level
    // consumers (the shrines, hives, mushroom patches and underground trees)
    // stay on the mud until their own faithful ports (the plan's
    // interim-alignment fallback; re-derived per M5-BIOME phase 3).
    let ws = width as f64 / 4200.0 * 1.5;
    let reach = (700.0 * ws) as i32;
    // The inland clamp rides the vanilla beach anchors (`GenVars.cs:90-92`):
    // the jungle band starts past the left beach and the dungeon keeps the
    // `dungeonBeachPadding` = 50 off the right beach (`GenVars.cs:452`,
    // `WorldGen.cs:11443-11460`).
    let jungle_start = (jungle_origin_x - reach).max(left_beach_end + 40).max(20);
    let jungle_end = (jungle_origin_x + reach)
        .min(width - 60)
        .max(jungle_start + 40);
    // The dungeon digs at the snow side (`DungeonCrawler`'s entrance spawns
    // at the far edge opposite the jungle).
    let dungeon_x = (right_beach_start - 50).max(width / 2);
    let _ = surface;
    BiomeSpans {
        jungle_start,
        jungle_end,
        dungeon_x,
    }
}

fn put(world: &mut World, x: i32, y: i32, kind: u16, wall: u16) {
    if x < 0 || y < 0 || x >= world.width || y >= world.height {
        return;
    }
    let i = world.index(x, y);
    let t = &mut world.tiles[i];
    t.kind = kind;
    t.flags |= FLAG_ACTIVE;
    t.set_slope(0);
    if y > 1 {
        t.wall = wall;
    }
}

fn active_at(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| t.active())
}

// --- the faithful jungle pass family ----------------------------------------
// `GameContent/Biomes/JunglePass.cs` + the two WorldGen lambdas, ported
// verbatim under the golden rule (structure, maths and RNG draw order
// frozen). The start point rides `GenVars.jungle_origin_x` — drawn once in
// the spans step (the port's `Reset` stand-in, `WorldGen.cs:11341-11376`),
// the pass consumes it without redrawing.

/// The jungle pass (`JunglePass.ApplyPass`, `JunglePass.cs:19-63`): three
/// first-pass mud blobs with the gem runners, the giant mud body walking up
/// from below the rock layer (walls 15/64 through the runner's `mudWall`
/// arm), the jungle chasm tunnel, the mud-wall holes and the finishing
/// touches. Deviations recorded for ANALYSIS: the dungeon side is fixed
/// right (the port has no dungeon-side roll — see the phase discovery D2),
/// `GenVars.JungleX` is write-only in 1.4.5.8 and not modelled, and the
/// beach anchors (`GenVars.cs:90-92`) are the port's beach margins.
pub(crate) fn pass_jungle(
    world: &mut World,
    jungle_origin_x: i32,
    world_surface: f64,
    rock_layer: f64,
    water_line: f64,
    lava_line: f64,
    left_beach_end: i32,
    right_beach_start: i32,
    rng: &mut Rng,
) {
    // `this._worldScale = Main.maxTilesX / 4200.0 * 1.5` (`:22`).
    let ws = world.width as f64 / 4200.0 * 1.5;
    // `CreateStartPoint` (`:80-83`): `(jungleOriginX,
    // (int)(maxTilesY + rockLayer) / 2)` — the cast covers the sum, then the
    // integer divide.
    let (mut x, mut y) = (
        jungle_origin_x,
        ((world.height as f64 + rock_layer) as i32) / 2,
    );
    let mut zero = (0i32, 0i32);
    // `ApplyRandomMovement(x, y, 100, 100)` + the first blob pair (`:28-32`).
    (x, y) = apply_random_movement(rng, ws, x, y, 100, 100, rock_layer, world.height);
    zero.0 += x;
    zero.1 += y;
    place_first_pass_mud(
        world,
        rng,
        ws,
        x,
        y,
        3,
        world_surface,
        rock_layer,
        water_line,
        lava_line,
    );
    place_gems_at(world, rng, ws, x, y, 63, 2, world_surface);
    // The second pair (`:34-38`).
    (x, y) = apply_random_movement(rng, ws, x, y, 250, 150, rock_layer, world.height);
    zero.0 += x;
    zero.1 += y;
    place_first_pass_mud(
        world,
        rng,
        ws,
        x,
        y,
        0,
        world_surface,
        rock_layer,
        water_line,
        lava_line,
    );
    place_gems_at(world, rng, ws, x, y, 65, 2, world_surface);
    // `oldX/oldY` saved before the third movement (`:40-41`) — the second
    // blob's centre, the finishing touches' anchor.
    let old_x = x;
    let old_y = y;
    // The third pair (`:42-46`).
    (x, y) = apply_random_movement(rng, ws, x, y, 400, 150, rock_layer, world.height);
    zero.0 += x;
    zero.1 += y;
    place_first_pass_mud(
        world,
        rng,
        ws,
        x,
        y,
        -3,
        world_surface,
        rock_layer,
        water_line,
        lava_line,
    );
    place_gems_at(world, rng, ws, x, y, 67, 2, world_surface);
    // The centre (`:48-52`): the mean of the three blob positions, `num` the
    // body strength and `num2` the beach margin; x clamped off the beaches
    // (`Utils.Clamp` is the min/max ternary, not a panicking clamp).
    let mut cx = zero.0 / 3;
    let cy = zero.1 / 3;
    let num = rng.range((400.0 * ws) as i32, (600.0 * ws) as i32);
    let num2 = (25.0 * ws) as i32;
    let (clamp_lo, clamp_hi) = (
        left_beach_end + num / 2 + num2,
        right_beach_start - num / 2 - num2,
    );
    cx = if cx < clamp_lo {
        clamp_lo
    } else if cx > clamp_hi {
        clamp_hi
    } else {
        cx
    };
    // The main mud body (`:53-54`): `TileRunner(x, y, num, 10000, 59,
    // addTile: false, 0.0, -20.0, noYChange: true)` — with `mudWall` armed,
    // the runner lines the body with walls 15/64 (`WorldGen.cs:76875-76887`).
    tile_runner(
        world,
        rng,
        cx,
        cy,
        num as f64,
        10000,
        59,
        world_surface,
        &TileRunnerOpts {
            mud_wall: true,
            speed_x: 0.0,
            speed_y: -20.0,
            no_y_change: true,
            rock_layer,
            water_line,
            lava_line,
            flags: world.seeds, ..TileRunnerOpts::default()
        },
    );
    // `GenerateTunnelToSurface(x, y)` (`:57`; the extraLiquid seed skips it —
    // out of scope), then `GenVars.mudWall = false` (`:59`).
    generate_tunnel_to_surface(world, rng, cx, cy, world_surface);
    generate_holes_in_mud_walls(world, rng, world_surface);
    generate_finishing_touches(world, rng, ws, old_x, old_y, world_surface);
}

/// `JunglePass.ApplyRandomMovement` (`:85-90`): the +-walk with the
/// `(int)`-truncated scaled ranges and the `[rockLayer, maxTilesY]` y clamp.
fn apply_random_movement(
    rng: &mut Rng,
    ws: f64,
    x: i32,
    y: i32,
    x_range: i32,
    y_range: i32,
    rock_layer: f64,
    max_tiles_y: i32,
) -> (i32, i32) {
    let x = x + rng.range(
        (-x_range as f64 * ws) as i32,
        1 + (x_range as f64 * ws) as i32,
    );
    let y = y + rng.range(
        (-y_range as f64 * ws) as i32,
        1 + (y_range as f64 * ws) as i32,
    );
    // `y = Utils.Clamp(y, (int)Main.rockLayer, Main.maxTilesY)` (`:89`).
    (x, y.clamp(rock_layer as i32, max_tiles_y))
}

/// `JunglePass.PlaceFirstPassMud` (`:73-78`): one mud blob — `TileRunner(x,
/// y, Next((int)(250*ws), (int)(500*ws)), Next(50, 150), 59, addTile: false,
/// speedX: dungeonSide * xSpeedScale)` with the `mudWall` wall arm armed for
/// the call. A zero scale leaves the velocity to the runner's own draws
/// (the `speedX != 0 || speedY != 0` gate, `WorldGen.cs:76801`).
#[allow(clippy::too_many_arguments)]
fn place_first_pass_mud(
    world: &mut World,
    rng: &mut Rng,
    ws: f64,
    x: i32,
    y: i32,
    x_speed_scale: i32,
    world_surface: f64,
    rock_layer: f64,
    water_line: f64,
    lava_line: f64,
) {
    // `DungeonSide.Right` — the port's dungeon sits on the right
    // (`plan_spans`' `dungeon_x`), so the side sign is fixed at +1.
    let dungeon_side = 1i32;
    let strength = rng.range((250.0 * ws) as i32, (500.0 * ws) as i32);
    let steps = rng.range(50, 150);
    tile_runner(
        world,
        rng,
        x,
        y,
        strength as f64,
        steps,
        59,
        world_surface,
        &TileRunnerOpts {
            mud_wall: true,
            speed_x: (dungeon_side * x_speed_scale) as f64,
            rock_layer,
            water_line,
            lava_line,
            flags: world.seeds, ..TileRunnerOpts::default()
        },
    );
}

/// `JunglePass.PlaceGemsAt` (`:65-71`): `6*ws` gem runners — each draws the
/// two +-`125*ws` offsets, the strength `Next(3, 7)`, the steps `Next(3, 8)`
/// and the gem `Next(baseGem, baseGem + gemVariants)` (63 Sapphire/64 Ruby,
/// 65 Emerald/66 Topaz, 67 Amethyst/68 Diamond).
fn place_gems_at(
    world: &mut World,
    rng: &mut Rng,
    ws: f64,
    x: i32,
    y: i32,
    base_gem: i32,
    gem_variants: i32,
    world_surface: f64,
) {
    // `for (int i = 0; (double)i < 6.0 * ws; i++)` — a fractional bound
    // still runs the ceil count.
    for _ in 0..(6.0 * ws).ceil() as i32 {
        let gx = x + rng.range((-125.0 * ws) as i32, (125.0 * ws) as i32);
        let gy = y + rng.range((-125.0 * ws) as i32, (125.0 * ws) as i32);
        let strength = rng.range(3, 7);
        let steps = rng.range(3, 8);
        let kind = rng.range(base_gem, base_gem + gem_variants);
        tile_runner(
            world,
            rng,
            gx,
            gy,
            strength as f64,
            steps,
            kind,
            world_surface,
            &TileRunnerOpts { flags: world.seeds, ..TileRunnerOpts::default() },
        );
    }
}

/// `JunglePass.GenerateTunnelToSurface` (`:92-191`): the jungle chasm — a
/// walker rising to the surface that carves Manhattan diamonds (`KillTile`,
/// `:148` — the worldgen carve convention deactivates the tile and keeps the
/// wall) until six clean tiles stand above the surface, branching side
/// runners past the 10th step. The drunk early-exit is a secret-seed branch.
fn generate_tunnel_to_surface(
    world: &mut World,
    rng: &mut Rng,
    i: i32,
    j: i32,
    world_surface: f64,
) {
    let mut radius = rng.range(5, 11) as f64;
    let mut px = i as f64;
    let mut py = j as f64;
    let mut vel_x = rng.range(-10, 11) as f64 * 0.1;
    let mut vel_y = rng.range(10, 20) as f64 * 0.1;
    let mut steps_since_branch = 0i32;
    loop {
        // The exit check (`:105-123`): above the surface, six consecutive
        // tiles (this row and the five above, clamped into the world) with
        // no wall and no active tile end the tunnel.
        if py < world_surface {
            let value = (px as i32).clamp(10, world.width - 10);
            let value2 = (py as i32).clamp(10, world.height - 10).max(5);
            let clean = |dy: i32| {
                world
                    .tile(value, value2 - dy)
                    .is_some_and(|t| t.wall == 0 && !t.active())
            };
            if clean(0) && clean(1) && clean(2) && clean(3) && clean(4) && clean(5) {
                break;
            }
        }
        // `GenVars.JungleX = (int)vector.X` (`:124`) — write-only in
        // 1.4.5.8 (reset `WorldGen.cs:8332`), not modelled.
        // The radius wobble (`:125-133`).
        radius += rng.range(-20, 21) as f64 * 0.1;
        radius = radius.clamp(5.0, 10.0);
        // The carve rect clamped to `[10, max-10]` (`:134-141`).
        let left = ((px - radius * 0.5) as i32).clamp(10, world.width - 10);
        let right = ((px + radius * 0.5) as i32).clamp(10, world.width - 10);
        let top = ((py - radius * 0.5) as i32).clamp(10, world.height - 10);
        let bottom = ((py + radius * 0.5) as i32).clamp(10, world.height - 10);
        for k in left..right {
            for l in top..bottom {
                let manhattan = (k as f64 - px).abs() + (l as f64 - py).abs();
                if manhattan < radius * 0.5 * (1.0 + rng.range(-10, 11) as f64 * 0.015) {
                    if let Some(tile) = world.tile_mut(k, l) {
                        tile.flags &= !FLAG_ACTIVE;
                    }
                }
            }
        }
        steps_since_branch += 1;
        // The side-branch runner (`:153-162`): `num4` starts at -2 and the
        // `Next(2) == 0` draw maps it to +2.
        if steps_since_branch > 10 && rng.range(0, 50) < steps_since_branch {
            steps_since_branch = 0;
            let branch_speed = if rng.range(0, 2) == 0 { 2.0 } else { -2.0 };
            let strength = rng.range(3, 20);
            let steps = rng.range(10, 100);
            tile_runner(
                world,
                rng,
                px as i32,
                py as i32,
                strength as f64,
                steps,
                -1,
                world_surface,
                &TileRunnerOpts {
                    speed_x: branch_speed,
                    flags: world.seeds, ..TileRunnerOpts::default()
                },
            );
        }
        // The move + velocity updates (`:163-189`): Y rises (clamped
        // `[-2, 0]`), X wanders with the +-200-tile pullback draws.
        px += vel_x;
        py += vel_y;
        vel_y += rng.range(-10, 11) as f64 * 0.01;
        if vel_y > 0.0 {
            vel_y = 0.0;
        }
        if vel_y < -2.0 {
            vel_y = -2.0;
        }
        vel_x += rng.range(-10, 11) as f64 * 0.1;
        if px < i as f64 - 200.0 {
            vel_x += rng.range(5, 21) as f64 * 0.1;
        }
        if px > i as f64 + 200.0 {
            vel_x -= rng.range(5, 21) as f64 * 0.1;
        }
        vel_x = vel_x.clamp(-1.5, 1.5);
    }
}

/// `JunglePass.GenerateHolesInMudWalls` (`:193-206`): `maxTilesX / 4` holes —
/// re-roll a spot until its wall is 64 or 15, then `MudWallRunner`. The
/// re-roll carries a 200k-try valve (see the loop note): vanilla's wall
/// region dwarfs the hole clearing, the port's sub-vanilla test worlds can
/// strip it bare.
fn generate_holes_in_mud_walls(world: &mut World, rng: &mut Rng, world_surface: f64) {
    // `Main.UnderworldLayer` = `maxTilesY - 200` (`Main.cs:2790`).
    let underworld_layer = world.height - 200;
    for _ in 0..world.width / 4 {
        let (mut num, mut num2) = (
            rng.range(20, world.width - 20),
            rng.range(world_surface as i32 + 10, underworld_layer),
        );
        // The re-roll is unbounded in the C# — vanilla's mud walls (the
        // body and blobs line a region hundreds of tiles tall over a
        // 4200-wide world) always leave it a hit. The port's test worlds are
        // sub-vanilla-sized, where the `MudWallRunner` holes can strip the
        // band bare; the valve bounds the search and skips the hole instead
        // of hanging (a recorded deviation — a vanilla-size world never
        // reaches it).
        let mut rolls = 0i32;
        while !matches!(world.tile(num, num2).map_or(0, |t| t.wall), 64 | 15) {
            rolls += 1;
            if rolls > 200_000 {
                break;
            }
            num = rng.range(20, world.width - 20);
            num2 = rng.range(world_surface as i32 + 10, underworld_layer);
        }
        if rolls <= 200_000 {
            mud_wall_runner(world, rng, num, num2, world_surface);
        }
    }
}

/// `WorldGen.MudWallRunner` (`:77212-77274`): the shrinking random walk that
/// clears walls (`wall = 0`) inside its Manhattan discs below the surface.
fn mud_wall_runner(world: &mut World, rng: &mut Rng, i: i32, j: i32, world_surface: f64) {
    let radius = rng.range(8, 21) as f64;
    let steps = rng.range(8, 33) as f64;
    let mut num3 = steps;
    let (mut px, mut py) = (i as f64, j as f64);
    let (mut vel_x, mut vel_y) = (
        rng.range(-10, 11) as f64 * 0.1,
        rng.range(-10, 11) as f64 * 0.1,
    );
    while radius > 0.0 && num3 > 0.0 {
        let disc = radius * (num3 / steps);
        num3 -= 1.0;
        // The rect clamps are `[0, maxTiles]` here (`:77228-77239`).
        let left = ((px - disc * 0.5) as i32).clamp(0, world.width);
        let right = ((px + disc * 0.5) as i32).clamp(0, world.width);
        let top = ((py - disc * 0.5) as i32).clamp(0, world.height);
        let bottom = ((py + disc * 0.5) as i32).clamp(0, world.height);
        for k in left..right {
            for l in top..bottom {
                let manhattan = (k as f64 - px).abs() + (l as f64 - py).abs();
                if manhattan < radius * 0.5 * (1.0 + rng.range(-10, 11) as f64 * 0.015)
                    && l as f64 > world_surface
                {
                    if let Some(tile) = world.tile_mut(k, l) {
                        tile.wall = 0;
                    }
                }
            }
        }
        px += vel_x;
        py += vel_y;
        vel_x += rng.range(-10, 11) as f64 * 0.05;
        vel_x = vel_x.clamp(-1.0, 1.0);
        vel_y += rng.range(-10, 11) as f64 * 0.05;
        vel_y = vel_y.clamp(-1.0, 1.0);
    }
}

/// `JunglePass.GenerateFinishingTouches` (`:208-257`): (a) the `20*ws` big
/// mud runners wandering from the second blob's centre, (b) the `10*ws`
/// pocket walks carving empty/liquid caves (types -1/-2) on mud spots,
/// (c) the `300*ws` dirt pockets with their `Next(4)` gem tails.
fn generate_finishing_touches(
    world: &mut World,
    rng: &mut Rng,
    ws: f64,
    old_x: i32,
    old_y: i32,
    world_surface: f64,
) {
    let (mut num, mut num2) = (old_x, old_y);
    // (a) `for i <= 20*ws` (`:213-219`).
    for _ in 0..(20.0 * ws).floor() as i32 + 1 {
        num += rng.range((-5.0 * ws) as i32, (6.0 * ws) as i32);
        num2 += rng.range((-5.0 * ws) as i32, (6.0 * ws) as i32);
        let strength = rng.range(40, 100);
        let steps = rng.range(300, 500);
        tile_runner(
            world,
            rng,
            num,
            num2,
            strength as f64,
            steps,
            59,
            world_surface,
            &TileRunnerOpts { flags: world.seeds, ..TileRunnerOpts::default() },
        );
    }
    // (b) `for j <= 10*ws` (`:220-241`): re-pick the anchor on a mud 59 tile
    // (the C# reads `.type` — inactive sky mud qualifies), then `8*ws`
    // +-30 walk steps carving pockets.
    for _ in 0..(10.0 * ws).floor() as i32 + 1 {
        (num, num2) = pick_mud_spot(world, rng, ws, old_x, old_y);
        for _ in 0..(8.0 * ws).floor() as i32 + 1 {
            num += rng.range(-30, 31);
            num2 += rng.range(-30, 31);
            let pocket_kind = if rng.range(0, 7) == 0 { -2 } else { -1 };
            let strength = rng.range(10, 20);
            let steps = rng.range(30, 70);
            tile_runner(
                world,
                rng,
                num,
                num2,
                strength as f64,
                steps,
                pocket_kind,
                world_surface,
                &TileRunnerOpts { flags: world.seeds, ..TileRunnerOpts::default() },
            );
        }
    }
    // (c) `for l <= 300*ws` (`:242-256`).
    for _ in 0..(300.0 * ws).floor() as i32 + 1 {
        (num, num2) = pick_mud_spot(world, rng, ws, old_x, old_y);
        let strength = rng.range(4, 10);
        let steps = rng.range(5, 30);
        tile_runner(
            world,
            rng,
            num,
            num2,
            strength as f64,
            steps,
            1,
            world_surface,
            &TileRunnerOpts { flags: world.seeds, ..TileRunnerOpts::default() },
        );
        if rng.range(0, 4) == 0 {
            // `:254`'s named arguments evaluate in source order — the gem
            // `type:` draw comes first, then `i:`, `j:`, `strength:`,
            // `steps:`.
            let g_kind = rng.range(63, 69);
            let gx = num + rng.range(-1, 2);
            let gy = num2 + rng.range(-1, 2);
            let g_strength = rng.range(3, 7);
            let g_steps = rng.range(4, 8);
            tile_runner(
                world,
                rng,
                gx,
                gy,
                g_strength as f64,
                g_steps,
                g_kind,
                world_surface,
                &TileRunnerOpts { flags: world.seeds, ..TileRunnerOpts::default() },
            );
        }
    }
}

/// The (b)/(c) spot picker (`:225-229`, `:247-250`): re-roll within
/// +-`(600*ws, 200*ws)` of the anchor until inside `[1, max-2]` and on a
/// tile of type 59.
fn pick_mud_spot(world: &World, rng: &mut Rng, ws: f64, old_x: i32, old_y: i32) -> (i32, i32) {
    loop {
        let x = old_x + rng.range((-600.0 * ws) as i32, (600.0 * ws) as i32);
        let y = old_y + rng.range((-200.0 * ws) as i32, (200.0 * ws) as i32);
        let in_bounds = x >= 1 && x < world.width - 1 && y >= 1 && y < world.height - 1;
        if in_bounds && world.tile(x, y).is_some_and(|t| t.kind == TILE_MUD) {
            return (x, y);
        }
    }
}

/// `MudCavesToJungleGrass` (`WorldGen.cs:12468-12499`): the full-map sweep
/// spreading jungle grass 60 onto exposed mud 59 (`grassSpread` reset per
/// tile), then the tiny-clump removal scan. RNG-free. The secret-seed
/// branches (`NotTheBees`, `DoSurfaceIsMushrooms`) are out of scope.
pub(crate) fn pass_mud_caves_to_jungle_grass(world: &mut World) {
    for i in 0..world.width {
        for j in 0..world.height {
            if world.tile(i, j).is_some_and(|t| t.active()) {
                // `WorldGen.grassSpread = 0; SpreadGrass(i, j, 59, 60)`
                // (`:12484-12487`).
                let mut grass_spread = 0i32;
                spread_jungle_grass(
                    world,
                    i,
                    j,
                    TILE_MUD,
                    TILE_JUNGLE_GRASS,
                    true,
                    &mut grass_spread,
                );
            }
        }
    }
    for k in 10..world.width - 10 {
        scan_tile_column_and_remove_clumps(world, k);
    }
}

/// `WorldGen.SpreadGrass` (`:75146-75270`), the slice the mud sweep runs:
/// the `InWorld(10)` guard (`:9688`), the kind gate, the 3x3 scan with the
/// C# `flag` semantics (`:75171-75199` — proceed iff some scanned tile is
/// inactive/non-solid; lava re-arms the flag and breaks the inner scan
/// only), the `CanBeClearedDuringGeneration` gate (`:80168` — the tiledata
/// set; 59 is in it), the plain kind write (mud 59 is not in
/// `Conversion.Grass`, `TileID.cs:86` — and no trees exist during
/// generation, so `TryConvertingOrKillingTreesAbove...` is a no-op) and the
/// 3x3 recursion with the `grassSpread < 1000` cap (`:75251-75263`).
/// (`growth.rs::spread_grass` carried an earlier slice whose neighbour gate
/// was inverted vs the C#; fixed 2026-11-09 by M5-SPREAD-1 — both growth
/// consumers now share the C# `flag` semantics.)
fn spread_jungle_grass(
    world: &mut World,
    i: i32,
    j: i32,
    dirt: u16,
    grass: u16,
    repeat: bool,
    grass_spread: &mut i32,
) {
    // `!InWorld(i, j, 10)` (`:75152`).
    if i < 10 || i >= world.width - 10 || j < 10 || j >= world.height - 10 {
        return;
    }
    if !world
        .tile(i, j)
        .is_some_and(|t| t.active() && t.kind == dirt)
    {
        return;
    }
    // The 3x3 scan (`:75171-75199`).
    let mut flag = true;
    for k in (i - 1)..=(i + 1) {
        for l in (j - 1)..=(j + 1) {
            if let Some(t) = world.tile(k, l) {
                if !t.active() || !crate::tiledata::TILE_SOLID.has(t.kind) {
                    flag = false;
                }
                if t.liquid > 0 && t.liquid_kind() == crate::tile::LiquidKind::Lava {
                    flag = true;
                    break;
                }
            }
        }
    }
    if flag {
        return;
    }
    // `!CanBeClearedDuringGeneration(type, i, j, skipDualDungeonCheck: true)`
    // (`:75202`) — the dual-dungeon branch is secret-seed only.
    if !world
        .tile(i, j)
        .is_some_and(|t| crate::tiledata::SET_CAN_BE_CLEARED_DURING_GENERATION.has(t.kind))
    {
        return;
    }
    // The flower-27 exclusions (`:75203-75206`) guard the evil/moss grasses,
    // not jungle grass 60.
    // The write (`:75219-75223`): the type only — walls and liquid kept;
    // `SquareTileFrame` is the re-frame pass the port defers.
    if let Some(tile) = world.tile_mut(i, j) {
        tile.kind = grass;
    }
    // The recursion (`:75234-75251`).
    if !repeat {
        return;
    }
    for m in (i - 1)..=(i + 1) {
        for n in (j - 1)..=(j + 1) {
            if world
                .tile(m, n)
                .is_some_and(|t| t.active() && t.kind == dirt)
                && *grass_spread < 1000
            {
                *grass_spread += 1;
                spread_jungle_grass(world, m, n, dirt, grass, repeat, grass_spread);
                *grass_spread -= 1;
            }
        }
    }
}

/// `WorldGen.ScanTileColumnAndRemoveClumps` (`:10196-10225`): per column,
/// every run of fewer than `tileCounterMax` (20, `:86940`) consecutive solid
/// `CanBeClearedDuringGeneration` tiles whose connected component (the
/// `tileCounter` flood fill, `:10069-10096`, bounded at 20 with the C#'s
/// `[5, max-5]` guard) is also under 20 is deactivated (`tileCounterKill`,
/// `:10098-10107`). RNG-free.
fn scan_tile_column_and_remove_clumps(world: &mut World, x: i32) {
    const TILE_COUNTER_MAX: usize = 20;
    let qualifies = |world: &World, k: i32, l: i32| -> bool {
        world.tile(k, l).is_some_and(|t| {
            t.active()
                && crate::tiledata::TILE_SOLID.has(t.kind)
                && crate::tiledata::SET_CAN_BE_CLEARED_DURING_GENERATION.has(t.kind)
        })
    };
    let mut num = 0i32;
    let mut y = 0i32;
    for i in 10..world.height - 10 {
        if qualifies(world, x, i) {
            if num == 0 {
                y = i;
            }
            num += 1;
            continue;
        }
        if num > 0 && (num as usize) < TILE_COUNTER_MAX {
            // `tileCounter(x, y)` + `tileCounterKill()` (`:10215-10221`).
            let mut component: Vec<(i32, i32)> = Vec::with_capacity(TILE_COUNTER_MAX);
            let mut queue = vec![(x, y)];
            while let Some((k, l)) = queue.pop() {
                // The `tileCounterNext` guard (`:10078-10079`).
                if k < 5 || k > world.width - 5 || l < 5 || l > world.height - 5 {
                    continue;
                }
                if !qualifies(world, k, l) {
                    continue;
                }
                if component.iter().any(|&(a, b)| a == k && b == l) {
                    continue;
                }
                if component.len() >= TILE_COUNTER_MAX {
                    continue; // `tileCounterNum >= tileCounterMax` (`:10080`)
                }
                component.push((k, l));
                queue.push((k - 1, l));
                queue.push((k + 1, l));
                queue.push((k, l - 1));
                queue.push((k, l + 1));
            }
            if component.len() < TILE_COUNTER_MAX {
                for (k, l) in component {
                    if let Some(t) = world.tile_mut(k, l) {
                        t.flags &= !FLAG_ACTIVE;
                    }
                }
            }
        }
        num = 0;
    }
}

/// `DirtToMud` (`WorldGen.cs:13123-13142`): `(w*h)*0.001` mud runners
/// centred below `rockLayerLow` — scattered underground mud blobs
/// independent of the jungle body — with sand 53 ignored
/// (`ignoreTileType`). Draws x, y, strength, steps in the C# order; the
/// remix branch is secret-seed-only (out of scope).
pub(crate) fn pass_dirt_to_mud(
    world: &mut World,
    rock_layer_low: f64,
    rock_layer: f64,
    water_line: f64,
    lava_line: f64,
    world_surface: f64,
    rng: &mut Rng,
) {
    // `for (int i = 0; (double)i < (maxTilesX * maxTilesY) * 0.001; i++)` —
    // a fractional bound still runs the ceil count.
    let count = (((world.width * world.height) as f64) * 0.001).ceil() as i32;
    for _ in 0..count {
        let x = rng.range(0, world.width);
        let y = rng.range(rock_layer_low as i32, world.height);
        let strength = rng.range(2, 6);
        let steps = rng.range(2, 40);
        tile_runner(
            world,
            rng,
            x,
            y,
            strength as f64,
            steps,
            59,
            world_surface,
            &TileRunnerOpts {
                ignore_tile_type: 53,
                rock_layer,
                water_line,
                lava_line,
                flags: world.seeds, ..TileRunnerOpts::default()
            },
        );
    }
}

/// The glowing mushroom pass (`GlowingMushroomPatches`,
/// `WorldGen.cs:12544-12756`): `maxTilesX / 700` clusters (capped at
/// `GenVars.maxMushroomBiomes = 50`) in the cavern band, each placed after the
/// rejection scans and grown by `ShroomPatch` plus five siblings; a full
/// `SpreadGrass(59, 70)` sweep then converts the mud to mushroom grass and the
/// tidy sweep re-forms the rails.
pub(crate) fn pass_mushroom(
    world: &mut World,
    spans: &BiomeSpans,
    rock_layer: f64,
    world_surface: f64,
    rng: &mut super::Rng,
) {
    let _ = spans;
    let w = world.width;
    let h = world.height;
    // `num = maxTilesX / 700.0`, clamped to maxMushroomBiomes = 50
    // (`:12566-12570`).
    let max_biomes = 50;
    let clusters = ((w as f64 / 700.0) as i32).min(max_biomes);
    let mut positions: Vec<(i32, i32)> = Vec::new();
    for _ in 0..clusters {
        let mut attempts = 0;
        loop {
            // The middle 60% band, widening to 95% after w/4 attempts
            // (`:12578-12582`).
            let x = if attempts > w / 4 {
                rng.range((w as f64 * 0.025) as i32, (w as f64 * 0.975) as i32)
            } else {
                rng.range((w as f64 * 0.2) as i32, (w as f64 * 0.8) as i32)
            };
            // The non-remix band: rockLayer + 50 .. maxTilesY - 300 (`:12586`).
            let lo = rock_layer as i32 + 50;
            let hi = (h - 300).max(lo + 1);
            let y = rng.range(lo, hi);
            attempts += 1;
            let mut reject = shroom_site_rejected(world, x, y);
            if !reject {
                // The 500-tile separation from the accepted patches
                // (`:12636-12645`).
                if positions
                    .iter()
                    .any(|(px, py)| (px - x) * (px - x) + (py - y) * (py - y) < 500 * 500)
                {
                    reject = true;
                }
            }
            if reject || positions.len() >= max_biomes as usize {
                if attempts <= w / 2 {
                    continue;
                }
                break;
            }
            positions.push((x, y));
            shroom_patch(world, x, y, world_surface, rock_layer, rng);
            for _ in 0..5 {
                let dx = x + rng.range(-40, 41);
                let dy = y + rng.range(-40, 41);
                shroom_patch(world, dx, dy, world_surface, rock_layer, rng);
            }
            break;
        }
    }
    // The SpreadGrass sweep over the cavern band (`:12679-12693`): every active
    // tile inside the InWorld(50) band re-seeds `grassSpread` and converts.
    let surface = world_surface as i32;
    for i in 0..w {
        for j in surface.max(0)..h {
            if i < 50 || i >= w - 50 || j < 50 || j >= h - 50 {
                continue;
            }
            if world.tile(i, j).is_some_and(|t| t.active()) {
                let mut grass_spread = 0i32;
                spread_jungle_grass(
                    world,
                    i,
                    j,
                    TILE_MUD,
                    TILE_MUSHROOM_GRASS,
                    false,
                    &mut grass_spread,
                );
            }
        }
    }
    // The tidy sweep (`:12694-12754`): the rail-shaped grass clumps are re-formed
    // (killed or re-mudded) and every fourth grass tile seeds a fresh grass
    // cell in its +/-20x+/-20 neighbourhood.
    for i in 0..w {
        for j in surface.max(0)..h {
            if !world
                .tile(i, j)
                .is_some_and(|t| t.active() && t.kind == TILE_MUSHROOM_GRASS)
            {
                continue;
            }
            shroom_tidy(world, i, j, rng);
        }
    }
}

/// The 100-tile rejection scan (`:12611-12635`): a live patch of ice, the
/// desert sands, the jungle grass, the other caves' tiles or the out-of-world
/// edge vetoes the site.
fn shroom_site_rejected(world: &World, x: i32, y: i32) -> bool {
    let (w, h) = (world.width, world.height);
    let radius = 100;
    let mut l = x - radius;
    while l < x + radius {
        let mut m = y - radius;
        while m < y + radius {
            if l < 0 || m < 0 || l >= w || m >= h {
                return true;
            }
            if let Some(t) = world.tile(l, m) {
                if t.active() && matches!(t.kind, 147 | 161 | 162 | 60 | 368 | 367) {
                    return true;
                }
            }
            m += 3;
        }
        l += 3;
    }
    false
}

/// `WorldGen.ShroomPatch` (`:79862-79982`): the elliptical walk placing mud
/// (59) and hollowing the dome core, converting pre-existing walls to the
/// unsafe mushroom wall, then two mud runners per step.
fn shroom_patch(
    world: &mut World,
    i: i32,
    j: i32,
    world_surface: f64,
    rock_layer: f64,
    rng: &mut super::Rng,
) {
    let mut num = rng.range(80, 100) as f64;
    let mut num2 = rng.range(20, 26) as f64;
    // `num3 *= 1.5` under for-the-worthy && !remix (`WorldGen.cs:79866-79870`) —
    // M5-SEED-1's shroom arm (the world-scale multiplier grows the patch by
    // half). The remix gate has no port field; the seed flags are the nearest
    // stand-in (documented).
    let mut num3 = world.width as f64 / 4200.0;
    if world.seeds.contains(crate::seeds::SeedFlags::FOR_THE_WORTHY)
        && !world.seeds.contains(crate::seeds::SeedFlags::REMIX)
    {
        num3 *= 1.5;
    }
    num *= num3;
    num2 *= num3;
    let num4 = num2 - 1.0;
    let mut num5 = num;
    let (mut px, mut py) = (i as f64, j as f64 - num2 * 0.3);
    let (mut vx, mut vy) = (
        rng.range(-100, 101) as f64 * 0.005,
        rng.range(-200, -99) as f64 * 0.005,
    );
    while num > 0.0 && num2 > 0.0 {
        num -= rng.range(0, 3) as f64;
        num2 -= 1.0;
        let (num6, num7, num8, num9) = (
            (px - num * 0.5) as i32,
            (px + num * 0.5) as i32,
            (py - num * 0.5) as i32,
            (py + num * 0.5) as i32,
        );
        let (num6, num7) = (num6.max(0), num7.min(world.width));
        let (num8, num9) = (num8.max(0), num9.min(world.height));
        num5 = num * rng.range(80, 120) as f64 * 0.01;
        for k in num6..num7 {
            for l in num8..num9 {
                let num10 = (k as f64 - px).abs();
                let num11 = ((l as f64 - py) * 2.3).abs();
                let num12 = (num10 * num10 + num11 * num11).sqrt();
                if num12 < num5 * 0.8
                    && world
                        .tile(k, l)
                        .is_some_and(|t| t.liquid_kind() == crate::tile::LiquidKind::Lava)
                {
                    if let Some(t) = world.tile_mut(k, l) {
                        t.liquid = 0;
                    }
                }
                if num12 < num5 * 0.2 && (l as f64) < py {
                    if let Some(t) = world.tile_mut(k, l) {
                        t.flags &= !FLAG_ACTIVE;
                        if t.wall > 0 {
                            t.wall = WALL_MUSHROOM_UNSAFE;
                        }
                    }
                } else if num12 < num5 * 0.4 * (0.95 + rng.next_f64() * 0.1) {
                    if let Some(t) = world.tile_mut(k, l) {
                        t.kind = TILE_MUD;
                        t.flags |= FLAG_ACTIVE;
                        if t.wall > 0 {
                            t.wall = WALL_MUSHROOM_UNSAFE;
                        }
                    }
                }
            }
        }
        px += vx;
        // The vanilla's `vector2D += vector2D2; vector2D.X += vector2D2.X;`
        // advances X twice per step — keep the quirk.
        px += vx;
        vx += rng.range(-100, 110) as f64 * 0.005;
        vy -= rng.range(0, 110) as f64 * 0.005;
        if vx > -0.5 && vx < 0.5 {
            vx = if vx < 0.0 { -0.5 } else { 0.5 };
        }
        vx = vx.clamp(-0.5, 0.5);
        vy = vy.clamp(-0.5, 0.5);
        for _ in 0..2 {
            // The mud runner re-rolls until it touches an active tile or mud
            // (`:79970-79980`).
            let (mut num13, mut num14);
            loop {
                num13 = px as i32 + rng.range(-20, 20);
                num14 = py as i32 + rng.range(0, 20);
                let hit = world
                    .tile(num13, num14)
                    .is_some_and(|t| t.active() || t.kind == TILE_MUD);
                if hit {
                    break;
                }
            }
            let strength = rng.range(10, 20) as f64;
            let steps = rng.range(10, 20);
            tile_runner(
                world,
                rng,
                num13,
                num14,
                strength,
                steps,
                TILE_MUD as i32,
                world_surface,
                &TileRunnerOpts {
                    add_tile: false,
                    speed_x: 0.0,
                    speed_y: 2.0,
                    no_y_change: true,
                    rock_layer,
                    ..Default::default()
                },
            );
        }
    }
}

/// The tidy sweep's per-tile arm (`:12696-12754`): rail-shaped grass dies, the
/// three-wide gaps re-mud, and every fourth grass tile seeds a fresh grass
/// cell nearby.
fn shroom_tidy(world: &mut World, i: i32, j: i32, rng: &mut super::Rng) {
    let (w, h) = (world.width, world.height);
    let active = |world: &World, x: i32, y: i32| {
        x >= 0 && y >= 0 && x < w && y < h && world.tile(x, y).is_some_and(|t| t.active())
    };
    for k in (i - 1)..=(i + 1) {
        for l in (j - 1)..=(j + 1) {
            if !(0..w).contains(&k) || !(0..h).contains(&l) {
                continue;
            }
            if active(world, k, l) {
                if !active(world, k - 1, l) && !active(world, k + 1, l) {
                    if let Some(t) = world.tile_mut(k, l) {
                        t.flags &= !FLAG_ACTIVE;
                    }
                } else if !active(world, k, l - 1) && !active(world, k, l + 1) {
                    if let Some(t) = world.tile_mut(k, l) {
                        t.flags &= !FLAG_ACTIVE;
                    }
                }
            } else if active(world, k - 1, l) && active(world, k + 1, l) {
                if let Some(t) = world.tile_mut(k, l) {
                    t.kind = TILE_MUD;
                    t.flags |= FLAG_ACTIVE;
                }
                if world
                    .tile(k - 1, j)
                    .is_some_and(|t| t.kind == TILE_MUSHROOM_GRASS)
                {
                    if let Some(t) = world.tile_mut(k - 1, j) {
                        t.kind = TILE_MUD;
                    }
                }
                if world
                    .tile(k + 1, j)
                    .is_some_and(|t| t.kind == TILE_MUSHROOM_GRASS)
                {
                    if let Some(t) = world.tile_mut(k + 1, j) {
                        t.kind = TILE_MUD;
                    }
                }
            } else if active(world, k, l - 1) && active(world, k, l + 1) {
                if let Some(t) = world.tile_mut(k, l) {
                    t.kind = TILE_MUD;
                    t.flags |= FLAG_ACTIVE;
                }
                if world
                    .tile(i, l - 1)
                    .is_some_and(|t| t.kind == TILE_MUSHROOM_GRASS)
                {
                    if let Some(t) = world.tile_mut(i, l - 1) {
                        t.kind = TILE_MUD;
                    }
                }
                if world
                    .tile(i, l + 1)
                    .is_some_and(|t| t.kind == TILE_MUSHROOM_GRASS)
                {
                    if let Some(t) = world.tile_mut(i, l + 1) {
                        t.kind = TILE_MUD;
                    }
                }
            }
        }
    }
    if rng.chance(4) {
        let x = i + rng.range(-20, 21);
        let y = j + rng.range(-20, 21);
        if (0..w).contains(&x)
            && (0..h).contains(&y)
            && world.tile(x, y).is_some_and(|t| t.kind == TILE_MUD)
        {
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = TILE_MUSHROOM_GRASS;
            }
        }
    }
}

/// The marble cave pass (`Marble`, `WorldGen.cs:12757-12838` +
/// `MarbleBiome.cs`): `Count` (4..8 scaled by world area) slab caves in the
/// cavern band, each centered on a rolled point away from the middle band.
pub(crate) fn pass_marble(world: &mut World, rock_layer: f64, rng: &mut super::Rng) {
    // `WorldGenRange.GetRandom`: ScaledMinimum/Maximum = (int)(value * num).
    let area = world.width as f64 * world.height as f64 / 5040000.0;
    let min = (4.0 * area) as i32;
    let max = (8.0 * area) as i32;
    let caves = rng.range(min, max + 1);
    if caves <= 0 {
        return;
    }
    let band = (world.width - 160) / caves;
    let (w, h) = (world.width, world.height);
    let mut placed = 0i32;
    let mut attempts = 0i32;
    while placed < caves {
        let num5 = placed as f64 / caves as f64;
        let mut origin_x = 0;
        let mut origin_y = 0;
        let mut tries = 100;
        loop {
            tries -= 1;
            if tries <= 0 {
                break;
            }
            // RandomRectanglePoint(band x, rockLayer + 20, band, depth)
            // (`WorldGen.cs:12787`).
            origin_x = (num5 * (w as f64 - 200.0)) as i32 + 100 + rng.range(0, band);
            origin_y = rock_layer as i32
                + 20
                + rng.range(
                    0,
                    (h as f64 - rock_layer as f64 - 40.0 - 200.0).max(1.0) as i32,
                );
            // The centre-band re-roll (`WorldGen.cs:12817-12820`).
            while origin_x > (w as f64 * 0.45) as i32 && origin_x < (w as f64 * 0.55) as i32 {
                origin_x = rng.range(BEACH_DISTANCE, w - BEACH_DISTANCE);
            }
            break;
        }
        attempts += 1;
        if marble_place(world, origin_x, origin_y, rng) {
            placed += 1;
            attempts = 0;
        } else if attempts > w * 10 {
            placed = caves;
            attempts = 0;
        }
    }
}

/// `MarbleBiome.Place` (`MarbleBiome.cs:179-252`): the slab-grid ellipse cave
/// with the `SCALE = 3` slabs, the smoothed states and the `PlaceSlab` walk.
fn marble_place(
    world: &mut World,
    mut origin_x: i32,
    mut origin_y: i32,
    rng: &mut super::Rng,
) -> bool {
    if biome_tile_check(world, origin_x, origin_y) {
        return false;
    }
    let num = rng.range(80, 150) / 3; // Next(80, 150) / 3
    let num2 = rng.range(40, 60) / 3;
    let num3 = (num2 * 3 - rng.range(20, 30)) / 3;
    origin_x -= num * 3 / 2;
    origin_y -= num2 * 3 / 2;
    // `_slabs` — (num + 2) x (num2 + 2) cells of (state, hasWall). The C#
    // keeps a persistent 56x26 array (`:187`); every cell the walk reads is
    // written first, so a fresh grid is equivalent.
    let gw = (num + 2) as usize;
    let gh = (num2 + 2) as usize;
    let mut slabs: Vec<(SlabState, bool)> = vec![(SlabState::Empty, true); gw * gh];
    let slab = |slabs: &[(SlabState, bool)], x: i32, y: i32| -> (SlabState, bool) {
        if x < 0 || y < 0 || x as usize >= gw || y as usize >= gh {
            (SlabState::Empty, false)
        } else {
            slabs[y as usize * gw + x as usize]
        }
    };
    for i in -1..(num + 1) {
        let num4 = (i - num / 2) as f64 / num as f64 + 0.5;
        let num5 = (((0.5 - (num4 - 0.5).abs()) * 5.0) as i32) - 2;
        for j in -1..(num2 + 1) {
            let (mut has_wall, mut flag) = (true, false);
            // `IsGroupSolid(i * 3 + origin.X, j * 3 + origin.Y, 3)`
            // (`MarbleBiome.cs:163-177`): the `SolidOrSlopedTile` census.
            let gx = i * 3 + origin_x;
            let gy = j * 3 + origin_y;
            let mut solid_count = 0;
            for a in 0..3 {
                for b in 0..3 {
                    if solid_or_sloped(world, gx + a, gy + b) {
                        solid_count += 1;
                    }
                }
            }
            let flag2 = solid_count > 3 / 4 * 3; // `num > scale / 4 * 3` (scale 3 → > 0)
            let num6 = (j - num2 / 2).abs() - num3 / 4 + num5;
            if num6 > 3 {
                flag = flag2;
                has_wall = false;
            } else if num6 > 0 {
                flag = (j - num2 / 2 > 0) || flag2;
                has_wall = (j - num2 / 2 < 0) || num6 <= 2;
            } else if num6 == 0 {
                flag = rng.chance(2) && ((j - num2 / 2 > 0) || flag2);
            }
            if (num4 - 0.5).abs() > 0.35 + rng.next_f64() * 0.1 && !flag2 {
                has_wall = false;
                flag = false;
            }
            slabs[(j + 1) as usize * gw + (i + 1) as usize] = (
                if flag { SlabState::Solid } else { SlabState::Empty },
                has_wall,
            );
        }
    }
    // The slope smoothing over the interior.
    for k in 0..num {
        for l in 0..num2 {
            marble_smooth(&mut slabs, k + 1, l + 1, gw as i32, gh as i32);
        }
    }
    // The ellipse arc emission with the lerped vertical offset.
    let num7 = num / 2;
    let num8 = num2 / 2;
    let num9 = (num8 + 1) * (num8 + 1);
    let value = rng.next_f64() * 2.0 - 1.0;
    let num10 = rng.next_f64() * 2.0 - 1.0;
    let value2 = rng.next_f64() * 2.0 - 1.0;
    let mut num11 = 0.0f64;
    for m in 0..=num {
        let num12 = num8 as f64 / num7 as f64 * (m - num7) as f64;
        let num13 = num8.min(((num9 as f64 - num12 * num12).max(0.0)).sqrt() as i32);
        num11 += if m >= num / 2 {
            num10 + (value2 - num10) * ((m as f64 / (num / 2) as f64) - 1.0)
        } else {
            value + (num10 - value) * (m as f64 / (num / 2) as f64)
        };
        for n in (num8 - num13)..=(num8 + num13) {
            let st = slab(&slabs, m + 1, n + 1);
            marble_place_slab(
                world,
                st,
                m * 3 + origin_x,
                n * 3 + origin_y + num11 as i32,
                rng,
            );
        }
    }
    // `structures.AddStructure(..., 8)` — the port has no StructureMap; the
    // cave-house voters read the tiles directly (deviation, ANALYSIS).
    true
}

/// `MarbleBiome.SlabStates` (`MarbleBiome.cs:12-48`): the seven per-cell
/// active patterns the smoothed slab grid carries. `IsSolid` (`:56`) is
/// "not Empty" — every filled pattern counts for the neighbour census.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlabState {
    Empty,
    Solid,
    HalfBrick,
    BottomRightFilled,
    BottomLeftFilled,
    TopRightFilled,
    TopLeftFilled,
}

impl SlabState {
    /// The delegate body (`MarbleBiome.cs:14-47`) at cell `(x, y)` of a
    /// `scale`-wide slab.
    fn active_at(self, x: i32, y: i32, scale: i32) -> bool {
        match self {
            SlabState::Empty => false,
            SlabState::Solid => true,
            SlabState::HalfBrick => y >= scale / 2,
            SlabState::BottomRightFilled => x >= scale - y,
            SlabState::BottomLeftFilled => x < y,
            SlabState::TopRightFilled => x > y,
            SlabState::TopLeftFilled => x < scale - y,
        }
    }
}

/// `MarbleBiome.SmoothSlope` (`MarbleBiome.cs:79-110`): the four-neighbour
/// bitmask maps a solid slab to the corner/half-brick states (the default
/// arm re-solidifies).
fn marble_smooth(slabs: &mut [(SlabState, bool)], x: i32, y: i32, gw: i32, gh: i32) {
    let at = |slabs: &[(SlabState, bool)], x: i32, y: i32| -> bool {
        if x < 0 || y < 0 || x >= gw || y >= gh {
            false
        } else {
            slabs[y as usize * gw as usize + x as usize].0 != SlabState::Empty
        }
    };
    let idx = y as usize * gw as usize + x as usize;
    if slabs[idx].0 == SlabState::Empty {
        return;
    }
    let mask = ((at(slabs, x, y - 1) as i32) << 3)
        | ((at(slabs, x, y + 1) as i32) << 2)
        | ((at(slabs, x - 1, y) as i32) << 1)
        | (at(slabs, x + 1, y) as i32);
    let state = match mask {
        10 => SlabState::TopLeftFilled,
        9 => SlabState::TopRightFilled,
        6 => SlabState::BottomLeftFilled,
        5 => SlabState::BottomRightFilled,
        4 => SlabState::HalfBrick,
        _ => SlabState::Solid,
    };
    slabs[idx].0 = state;
}

/// `MarbleBiome.PlaceSlab` (`MarbleBiome.cs:112-161`): the edge-column skips,
/// the extent jitter, the `ResetToType` ore preservation, the per-cell state
/// pattern, the wall, the per-tile `Tile.SmoothSlope` and the two
/// `Next(4)`-gated `PlaceTight` speleothem arms.
fn marble_place_slab(
    world: &mut World,
    slab: (SlabState, bool),
    origin_x: i32,
    origin_y: i32,
    rng: &mut super::Rng,
) {
    // `:114-120` — the drunk world swaps marble for granite.
    let (num, wall) = if world.seeds.contains(crate::seeds::SeedFlags::DRUNK) {
        (TILE_GRANITE, WALL_GRANITE)
    } else {
        (TILE_MARBLE, WALL_MARBLE)
    };
    let scale = 3;
    let (num2, num3) = (-1i32, scale + 1);
    let (mut num4, mut num5) = (0i32, scale);
    for i in num2..num3 {
        if (i == num2 || i == num3 - 1) && rng.chance(2) {
            continue;
        }
        if rng.chance(2) {
            num4 -= 1;
        }
        if rng.chance(2) {
            num5 += 1;
        }
        for j in num4..num5 {
            let (x, y) = (origin_x + i, origin_y + j);
            let Some(t) = world.tile_mut(x, y) else {
                continue;
            };
            // `ResetToType(TileID.Sets.Ore[tile.type] ? tile.type : num)`
            // (`:142`) — the reset clears liquid/flags/frames, the wall rides.
            let kind = if tile_is_ore(t.kind) { t.kind } else { num };
            reset_to_type(t, kind);
            if slab.0.active_at(i, j, scale) {
                t.flags |= FLAG_ACTIVE;
            } else {
                t.flags &= !FLAG_ACTIVE;
            }
            if slab.1 {
                t.wall = wall;
            }
            // `WorldUtils.TileFrame`/`SquareWallFrame` (`:148-149`) ride the
            // port's deferred end-of-pipeline framing seam (ANALYSIS).
            tile_smooth_slope(world, x, y);
            // `:151-158` — the two speleothem arms: the SolidTile gate draws
            // the Next(4) only when it passes (the C# short-circuit order).
            if solid_tile(world, x, y - 1) && rng.chance(4) {
                place_tight(world, rng, x, y, false);
            }
            if solid_tile(world, x, y) && rng.chance(4) {
                place_tight(world, rng, x, y - 1, false);
            }
        }
    }
}

/// `TileID.Sets.Ore` (`TileID.cs:614`) verbatim — the slab keeps the ores.
fn tile_is_ore(kind: u16) -> bool {
    matches!(
        kind,
        7 | 166 | 6 | 167 | 9 | 168 | 8 | 169 | 22 | 204 | 37 | 58 | 107 | 221 | 108 | 222
            | 111 | 223 | 211
    )
}

/// `WorldGen.SolidTile` (`WorldGen.cs:70111-70131`) — a null tile reads solid
/// (`Main.tile[i, j] == null → true`); the `noDoors` arm never fires here.
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && TILE_SOLID.has(t.kind)
        && !TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

/// `WorldGen.SolidOrSlopedTile` (`WorldGen.cs:69985-70009`,
/// `includePlatforms: false`): active, `tileSolid`, not `tileSolidTop`, not
/// wired-inactive — the slope/half-brick states stay solid.
/// `pub(crate)` since the W4-g40 entrance residual (`Modifiers.IsSolid`,
/// `Modifiers.cs:561-571`).
pub(crate) fn solid_or_sloped(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| {
        t.active() && TILE_SOLID.has(t.kind) && !TILE_SOLID_TOP.has(t.kind) && !t.inactive()
    })
}

/// `WorldGen.TileEmpty` (`WorldGen.cs:69976-69983`): absent or actuated-off
/// reads empty.
fn tile_empty(world: &World, x: i32, y: i32) -> bool {
    !world.tile(x, y).is_some_and(|t| t.active() && !t.inactive())
}

/// `Tile.SmoothSlope(x, y)` (`Tile.cs:792-865`, `applyToNeighbors: true`):
/// the four neighbours smooth first (each with `applyToNeighbors: false`),
/// then the cell itself. No RNG. The `sync` net arm never fires in worldgen.
/// `pub(crate)` since the W4-g40 entrance residual (`Actions.Smooth`,
/// `Actions.cs:608-622`, the anthill tail + the larva mouth).
pub(crate) fn tile_smooth_slope(world: &mut World, x: i32, y: i32) {
    for (nx, ny) in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
        tile_smooth_slope_one(world, nx, ny);
    }
    tile_smooth_slope_one(world, x, y);
}

/// The single-cell `Tile.SmoothSlope` body (`Tile.cs:801-846`): the
/// `CanPoundTile` + `SolidOrSlopedTile` gate, then the neighbour-mask
/// slope/half-brick map. `pub(crate)` since the W4-g40 entrance residual
/// (the anthill's `Smooth()` default `applyToNeighbors: false`).
pub(crate) fn tile_smooth_slope_one(world: &mut World, x: i32, y: i32) {
    if !crate::edit::can_pound_tile(world, x, y, true, |_, _| false)
        || !solid_or_sloped(world, x, y)
    {
        return;
    }
    let flag = !tile_empty(world, x, y - 1);
    let flag2 = !solid_or_sloped(world, x, y - 1) && flag;
    let flag3 = solid_or_sloped(world, x, y + 1);
    let flag4 = solid_or_sloped(world, x - 1, y);
    let flag5 = solid_or_sloped(world, x + 1, y);
    let mask = ((flag as i32) << 3) | ((flag3 as i32) << 2) | ((flag4 as i32) << 1) | (flag5 as i32);
    let Some(t) = world.tile_mut(x, y) else {
        return;
    };
    match mask {
        // `:816-829` — the flag2 veto breaks WITHOUT a write (the default
        // arm does not run).
        10 => {
            if !flag2 {
                t.flags &= !FLAG_HALF_BRICK;
                t.set_slope(3);
            }
        }
        9 => {
            if !flag2 {
                t.flags &= !FLAG_HALF_BRICK;
                t.set_slope(4);
            }
        }
        6 => {
            t.flags &= !FLAG_HALF_BRICK;
            t.set_slope(1);
        }
        5 => {
            t.flags &= !FLAG_HALF_BRICK;
            t.set_slope(2);
        }
        4 => {
            t.set_slope(0);
            t.flags |= FLAG_HALF_BRICK;
        }
        _ => {
            t.flags &= !FLAG_HALF_BRICK;
            t.set_slope(0);
        }
    }
}

/// `WorldGen.BiomeTileCheck` (`WorldGen.cs:10934-10961`): a 50-tile radius box
/// rejects a site that already holds a cave/biome tile or a sandstone /
/// hardened-sand wall.
fn biome_tile_check(world: &World, x: i32, y: i32) -> bool {
    let radius = 50;
    for i in (x - radius)..=(x + radius) {
        for j in (y - radius)..=(y + radius) {
            let Some(t) = world.tile(i, j) else { continue };
            if t.active() && matches!(t.kind, 368 | 367 | 147 | 161 | 162 | 70 | 72 | 396 | 397) {
                return true;
            }
            if matches!(t.wall, 187 | 216) {
                return true;
            }
        }
    }
    false
}

/// `WorldGen.beachDistance` (`WorldGen.cs:86914`) = 380.
const BEACH_DISTANCE: i32 = 380;

/// The granite cave pass (`Granite`, `WorldGen.cs:12839-12926`):
/// `Count` (4..8 scaled by world width) pressure-simulated magma caves. The
/// candidates are collected with the `CanPlace` gate (an inactive origin tile
/// plus `BiomeTileCheck`) and the `WorldGen.cs:12897-12917` retry, then placed
/// in a second phase. `lava_line` is `GenVars.lavaLine` for `ShouldUseLava`.
pub(crate) fn pass_granite(
    world: &mut World,
    rock_layer: f64,
    lava_line: i32,
    rng: &mut super::Rng,
) {
    let width = world.width as f64 / 4200.0;
    let min = (4.0 * width) as i32;
    let max = (8.0 * width) as i32;
    let mut caves = rng.range(min, max + 1);
    if caves <= 0 {
        return;
    }
    let band = (world.width - 200) / caves;
    let (w, h) = (world.width, world.height);
    let mut points: Vec<(i32, i32)> = Vec::new();
    let mut placed = 0i32;
    let mut attempts = 0i32;
    while placed < caves {
        // `:12855` — the C# fraction is a FLOAT here (the marble pass uses
        // double): `(float)num4 / (float)num`, then
        // `(int)(num5 * (float)(maxTilesX - 200))`.
        let num5 = placed as f32 / caves as f32;
        // RandomRectanglePoint(band x, rockLayer + 20, band, depth).
        let mut x = (num5 * (w - 200) as f32) as i32 + 100 + rng.range(0, band);
        let y = rock_layer as i32
            + 20
            + rng.range(
                0,
                (h as f64 - rock_layer as f64 - 40.0 - 200.0).max(1.0) as i32,
            );
        // The centre-band re-roll (`WorldGen.cs:12899-12902`).
        while x > (w as f64 * 0.45) as i32 && x < (w as f64 * 0.55) as i32 {
            x = rng.range(BEACH_DISTANCE, w - BEACH_DISTANCE);
        }
        attempts += 1;
        // `GraniteBiome.CanPlace` (`GraniteBiome.cs:53-60`): the shared
        // BiomeTileCheck, then the inactive origin.
        let in_cave = !world.tile(x, y).is_some_and(|t| t.active());
        if !biome_tile_check(world, x, y) && in_cave {
            points.push((x, y));
            placed += 1;
            // `:12907-12911` — the success arm does NOT reset `num3` (the
            // marble pass does): the give-up budget accumulates across the
            // whole collection phase.
        } else if attempts > w * 10 {
            // `:12912-12917` — `num = num4; num4++` ends the loop.
            caves = placed;
            attempts = 0;
        }
    }
    for (x, y) in points {
        granite_place(world, x, y, lava_line);
    }
}

/// `GraniteBiome.Place` (`GraniteBiome.cs:62-77`): the 200x200 magma map, the
/// pressure simulation, the granite placement, the cleanup and the
/// decorations. The magma map's first index is the world X and the second
/// the world Y (`BuildMagmaMap`'s `i2 = i + tileOrigin.X`).
fn granite_place(world: &mut World, origin_x: i32, origin_y: i32, lava_line: i32) -> bool {
    // `:64-67` — Place re-checks ONLY the origin's active bit (the
    // `BiomeTileCheck` ran once at collection time in `CanPlace`; a chamber
    // painted by an earlier placement does not veto a later one).
    if world.tile(origin_x, origin_y).is_some_and(|t| t.active()) {
        return false;
    }
    // The origin is offset by half the map (`Place`'s centring).
    let (ox, oy) = (origin_x - 100, origin_y - 100);
    // `BuildMagmaMap` (`GraniteBiome.cs:79-93`): resistance 1 for non-solid,
    // 4 for solid — `WorldGen.SolidTile`, so half-bricks/slopes read
    // non-solid and a null tile reads solid.
    let side = 200usize;
    let mut source: Vec<MagmaCell> = vec![MagmaCell::empty(1.0); side * side];
    for mx in 0..side {
        for my in 0..side {
            let solid = solid_tile(world, ox + mx as i32, oy + my as i32);
            source[my * side + mx] = MagmaCell::empty(if solid { 4.0 } else { 1.0 });
        }
    }
    let mut target = source.clone();
    let vecs = normalised_vectors();
    // The active box starts 0x0 at the centre and grows (`SimulatePressure`,
    // `GraniteBiome.cs:95-169`). The C# outer loop is the FIRST index (world
    // X, `j`), the inner the second (world Y, `k`) — the mid-iteration box
    // growth reads the same order.
    let (mut mx0, mut mx1, mut my0, mut my1) = (side / 2, side / 2, side / 2, side / 2);
    for iter in 0..300 {
        // The C# `for` conditions re-read the growing box bounds every
        // iteration (`j <= num4`, `k <= num6`) — a Rust range-for would
        // capture them once, so the walks are while-loops.
        let mut mx = mx0;
        while mx <= mx1 {
            let mut my = my0;
            while my <= my1 {
                let cell = source[my * side + mx];
                if !cell.active {
                    my += 1;
                    continue;
                }
                let mut sum = 0.0f64;
                let (mut grad_x, mut grad_y) = (0.0f64, 0.0f64);
                for l in -1..=1i32 {
                    for m in -1..=1i32 {
                        if l == 0 && m == 0 {
                            continue;
                        }
                        let (nmx, nmy) = (mx as i32 + l, my as i32 + m);
                        if nmx < 0 || nmy < 0 || nmx as usize >= side || nmy as usize >= side {
                            continue;
                        }
                        let v = vecs[((l + 1) * 3 + (m + 1)) as usize];
                        let neighbour = source[nmy as usize * side + nmx as usize];
                        if cell.pressure > 0.01 && !neighbour.active {
                            if l == -1 {
                                mx0 = mx0.min(nmx as usize).max(1);
                            } else {
                                mx1 = mx1.max(nmx as usize).min(side - 2);
                            }
                            if m == -1 {
                                my0 = my0.min(nmy as usize).max(1);
                            } else {
                                my1 = my1.max(nmy as usize).min(side - 2);
                            }
                            target[nmy as usize * side + nmx as usize] = neighbour.to_flow();
                        }
                        sum += neighbour.pressure;
                        grad_x += neighbour.pressure * v.0;
                        grad_y += neighbour.pressure * v.1;
                    }
                }
                sum /= 8.0;
                if sum > cell.resistance {
                    let gradient = (grad_x * grad_x + grad_y * grad_y).sqrt() / 8.0;
                    // `:157` — num9 = Max(0, ...) BEFORE the resistance
                    // decay reads it (`:158`).
                    let flow = ((sum - gradient - cell.pressure).max(0.0)
                        + gradient
                        + cell.pressure * 0.875
                        - cell.resistance)
                        .max(0.0);
                    target[my * side + mx] =
                        MagmaCell::flow(flow, (cell.resistance - flow * 0.02).max(0.0));
                }
                my += 1;
            }
            mx += 1;
        }
        if iter < 2 {
            target[(side / 2) * side + side / 2] = MagmaCell::flow(25.0, 0.0);
        }
        std::mem::swap(&mut source, &mut target);
    }
    let area = (mx0, mx1, my0, my1);
    granite_place_tiles(world, ox, oy, &source, area, lava_line);
    granite_cleanup(world, ox, oy, &source, area);
    granite_decorations(world, ox, oy, &source, area);
    // `structures.AddStructure(effectedMapArea.Modified(...), 8)` — no
    // StructureMap in the port (deviation, ANALYSIS).
    true
}

/// The magma cell: `MagmaCell` mirrors `GraniteBiome.Magma`'s
/// `(pressure, resistance, active)`.
#[derive(Clone, Copy)]
struct MagmaCell {
    pressure: f64,
    resistance: f64,
    active: bool,
}

impl MagmaCell {
    fn empty(resistance: f64) -> Self {
        Self {
            pressure: 0.0,
            resistance,
            active: false,
        }
    }
    fn flow(pressure: f64, resistance: f64) -> Self {
        Self {
            pressure,
            resistance,
            active: true,
        }
    }
    fn to_flow(&self) -> Self {
        Self {
            pressure: self.pressure,
            resistance: self.resistance,
            active: true,
        }
    }
}

/// The static 8-direction normalisation (`GraniteBiome.cs:332-347`).
fn normalised_vectors() -> Vec<(f64, f64)> {
    let mut v = Vec::with_capacity(9);
    for (x, y) in [
        (-1.0f64, -1.0f64),
        (-1.0, 0.0),
        (-1.0, 1.0),
        (0.0, -1.0),
        (0.0, 0.0),
        (0.0, 1.0),
        (1.0, -1.0),
        (1.0, 0.0),
        (1.0, 1.0),
    ] {
        let len = (x * x + y * y).sqrt();
        v.push((x / len.max(1e-12), y / len.max(1e-12)));
    }
    v
}

/// `GraniteBiome.ShouldUseLava` (`GraniteBiome.cs:171-196`): the chamber
/// centre within 30 of `GenVars.lavaLine`, and no snow/ice (147/161-163) or
/// ice-furniture 200 in the 100x100 scan around it.
fn granite_should_use_lava(world: &World, ox: i32, oy: i32, lava_line: i32) -> bool {
    if oy + 100 <= lava_line - 30 {
        return false;
    }
    for i in -50..50 {
        for j in -50..50 {
            let (x, y) = (ox + 100 + i, oy + 100 + j);
            if !in_world(world, x, y, 10) {
                continue;
            }
            if let Some(t) = world.tile(x, y) {
                if t.active() && (t.kind == 147 || (161..=163).contains(&t.kind) || t.kind == 200)
                {
                    return false;
                }
            }
        }
    }
    true
}

/// `WorldGen.InWorld` (`WorldGen.cs:9688-9695`) with the fluff margin.
fn in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// `GraniteBiome.PlaceGranite` (`GraniteBiome.cs:198-245`): the sine-gated
/// placement, the ore preservation (`ResetToType` both arms — the ore arm
/// keeps the kind but still resets the tile and repaints the wall), the
/// resistance-0 clear arm and the lava retype.
fn granite_place_tiles(
    world: &mut World,
    ox: i32,
    oy: i32,
    map: &[MagmaCell],
    area: (usize, usize, usize, usize),
    lava_line: i32,
) {
    // `:200-207` — the drunk world swaps granite for marble.
    let use_lava = granite_should_use_lava(world, ox, oy, lava_line);
    let (kind, wall) = if world.seeds.contains(crate::seeds::SeedFlags::DRUNK) {
        (TILE_MARBLE, WALL_MARBLE)
    } else {
        (TILE_GRANITE, WALL_GRANITE)
    };
    let side = 200usize;
    let (mx0, mx1, my0, my1) = area;
    // The C# loops are X-major (`i` over Left..Right, `j` over Top..Bottom).
    for mx in mx0..=mx1 {
        for my in my0..=my1 {
            let (x, y) = (ox + mx as i32, oy + my as i32);
            if !in_world(world, x, y, 10) {
                continue;
            }
            let cell = map[my * side + mx];
            if !cell.active {
                continue;
            }
            // `:222` — the sine gate; the sqrt argument is clamped at 0, so
            // a zero excess gives 0.5/0 = +Infinity (the C# double divide),
            // which drives the gate to -Infinity and the pressure term wins.
            let pressure = (cell.pressure - cell.resistance).max(0.0).sqrt();
            let gate = 1.0 - ((y as f64 * 0.4).sin() * 0.7 + 1.2) * (0.2 + 0.5 / pressure);
            let threshold = 0.35 + if solid_tile(world, x, y) { 0.0 } else { 0.5 };
            if gate.max(cell.pressure / 15.0) > threshold {
                let kind_now = world.tile(x, y).map(|t| t.kind).unwrap_or(0);
                let target_kind = if tile_is_ore(kind_now) { kind_now } else { kind };
                if let Some(t) = world.tile_mut(x, y) {
                    reset_to_type(t, target_kind);
                    t.wall = wall;
                }
            } else if cell.resistance < 0.01 {
                // `:234-238` — the fully-flowed cell clears to air.
                clear_tile_keep(world, x, y);
                if let Some(t) = world.tile_mut(x, y) {
                    t.wall = wall;
                }
            }
            // `:239-242` — the lava retype: `(liquid > 0) & flag`.
            if let Some(t) = world.tile_mut(x, y) {
                if t.liquid > 0 && use_lava {
                    t.set_liquid_kind(LiquidKind::Lava);
                }
            }
        }
    }
}

/// `GraniteBiome.CleanupTiles` (`GraniteBiome.cs:247-294`): solid cells with
/// fewer than 3 solid neighbours in the 3x3 are cleared (the wall painted
/// after). The census is `WorldGen.SolidTile`, not the active bit.
fn granite_cleanup(
    world: &mut World,
    ox: i32,
    oy: i32,
    map: &[MagmaCell],
    area: (usize, usize, usize, usize),
) {
    let wall = if world.seeds.contains(crate::seeds::SeedFlags::DRUNK) {
        WALL_MARBLE
    } else {
        WALL_GRANITE
    };
    let side = 200usize;
    let (mx0, mx1, my0, my1) = area;
    let mut clear: Vec<(i32, i32)> = Vec::new();
    for mx in mx0..=mx1 {
        for my in my0..=my1 {
            if !map[my * side + mx].active {
                continue;
            }
            let (x, y) = (ox + mx as i32, oy + my as i32);
            if !in_world(world, x, y, 10) || !solid_tile(world, x, y) {
                continue;
            }
            let mut solid = 0;
            for k in -1..=1i32 {
                for l in -1..=1i32 {
                    if solid_tile(world, x + k, y + l) {
                        solid += 1;
                    }
                }
            }
            if solid < 3 {
                clear.push((x, y));
            }
        }
    }
    for (x, y) in clear {
        // `WorldUtils.ClearTile(x, y, frameNeighbors: true)` — the reframe
        // rides the deferred framing seam (ANALYSIS).
        clear_tile_keep(world, x, y);
        if let Some(t) = world.tile_mut(x, y) {
            t.wall = wall;
        }
    }
}

/// `GraniteBiome.PlaceDecorations` (`GraniteBiome.cs:296-330`): the
/// world-seed-keyed `FastRandom` stalactite pass — `WithModifier(65440)`
/// once per chamber, then a per-cell `WithModifier(x, y)` copy: `Next(8)`
/// gates the active-cell stalactite pair (below first, then above — each
/// with its own `Next(2)` small roll and `Next(3)` variation), then the
/// unconditional `Next(2)` slope-smoothing roll. No worldgen-stream draw.
fn granite_decorations(
    world: &mut World,
    ox: i32,
    oy: i32,
    map: &[MagmaCell],
    area: (usize, usize, usize, usize),
) {
    // `Main.ActiveWorldFileData.Seed` is an int — the `FastRandom(int)`
    // constructor (`FastRandom.cs:21-25`), the hive precedent's cast.
    let fast = FastRandom::new(world.seed as i32 as u64).with_modifier_u64(65440);
    let side = 200usize;
    let (mx0, mx1, my0, my1) = area;
    for mx in mx0..=mx1 {
        for my in my0..=my1 {
            let cell = map[my * side + mx];
            let (x, y) = (ox + mx as i32, oy + my as i32);
            if !in_world(world, x, y, 10) || !cell.active {
                continue;
            }
            // `WorldUtils.TileFrame`/`SquareWallFrame` (`:310-311`) ride the
            // deferred framing seam (ANALYSIS).
            let mut fast2 = fast.with_modifier(x, y);
            if fast2.next(8) == 0 && world.tile(x, y).is_some_and(|t| t.active()) {
                if !world.tile(x, y + 1).is_some_and(|t| t.active()) {
                    let small = fast2.next(2) == 0;
                    let variation = fast2.next(3);
                    place_unchecked_stalactite(world, x, y + 1, small, variation, false);
                }
                if !world.tile(x, y - 1).is_some_and(|t| t.active()) {
                    let small = fast2.next(2) == 0;
                    let variation = fast2.next(3);
                    place_unchecked_stalactite(world, x, y - 1, small, variation, false);
                }
            }
            if fast2.next(2) == 0 {
                tile_smooth_slope(world, x, y);
            }
        }
    }
}

/// The dungeon pass (`Dungeon` + `DungeonCrawler`): the surface entrance box
/// and the descending shaft, then 3 corridor levels of the dungeon brick
/// with the dungeon walls, the corridors carved inward. The brick colour
/// rides the world's dungeon roll (41 blue / 43 green / 44 pink).
///
/// Retired from the pipeline 2026-11-09 (M5-DUNGEON): the faithful crawler
/// (`worldgen/dungeon.rs::generate_dungeon`) took the step's slot — kept for
/// its tests and as the reconciliation reference.
#[allow(dead_code)]
pub(crate) fn pass_dungeon(
    world: &mut World,
    surface: &[i32],
    spans: &BiomeSpans,
    rng: &mut super::Rng,
) {
    // The brick colour rides the world roll (`Main.cs`'s dungeon roll): the
    // blue 41 / the green 43 / the pink 44 — the walls follow (7/8/9).
    let brick_i = rng.range(0, 3) as usize;
    let brick = [41u16, 43, 44][brick_i];
    let dungeon_wall = [7u16, 8, 9][brick_i];
    let x0 = spans.dungeon_x;
    // The drunk start row (`WorldGen.cs:14804-14807`): `(int)Main.worldSurface
    // + 70` instead of the searched depth — M5-SEED-1. The vanilla arm comes
    // after the search and the `Next(-200,200)` offset, so no draw moves.
    let top = if world.seeds.contains(crate::seeds::SeedFlags::DRUNK) {
        world.world_surface as i32 + 70
    } else {
        surface[x0 as usize]
    };
    let bottom = (world.height - 220).min(top + (world.rock_layer as i32) * 2);
    // The entrance tower: a 5-wide brick shell down from the surface.
    let mut y = top - 8;
    while y < top + 30 {
        for dx in -3..=3 {
            put(world, x0 + dx, y, brick, dungeon_wall);
        }
        y += 1;
    }
    // The shaft: carved through the brick column down to the corridors.
    for yy in (top - 7)..(top + 30) {
        for dx in -1..=1 {
            let i = world.index(x0 + dx, yy);
            world.tiles[i] = Tile::default();
            world.tiles[i].wall = dungeon_wall;
        }
    }
    // The corridor levels: a stair-stepping crawl to the bottom, each level
    // a carved tunnel in the brick-filled rock with the dungeon walls.
    let mut cx = x0;
    let mut cy = top + 40;
    let level_gap = ((bottom - cy) / 4).max(30);
    while cy < bottom {
        let end_x = if cx < world.width / 2 {
            cx + rng.range(60, 140)
        } else {
            cx - rng.range(60, 140)
        };
        let step = if end_x > cx { 1 } else { -1 };
        let mut x = cx;
        while x != end_x {
            for dy in 0..4 {
                for dx in 0..2 {
                    let i = world.index(x + dx, cy + dy);
                    if x + dx > 2 && x + dx < world.width - 2 && cy + dy < world.height - 2 {
                        world.tiles[i] = Tile::default();
                        world.tiles[i].wall = dungeon_wall;
                    }
                }
            }
            // The brick lining above and below the tunnel.
            for dy in [-1i32, 4] {
                for dx in 0..2 {
                    put(world, x + dx, cy + dy, brick, dungeon_wall);
                }
            }
            x += step;
        }
        // The vertical drop to the next level.
        let next_cy = (cy + level_gap).min(bottom);
        while cy < next_cy {
            for dx in 0..2 {
                let i = world.index(cx, cy);
                if cx > 2 && cx < world.width - 2 && cy < world.height - 2 {
                    world.tiles[i] = Tile::default();
                    world.tiles[i].wall = dungeon_wall;
                }
                let _ = dx;
                cy += 1;
                break;
            }
        }
        cx = end_x;
    }
}

/// The solid biome-ground tile test for the island clearance scans (the
/// stone/dirt/ice/sand family the islands must not overlap).
pub(crate) fn is_solid_biome_tile(kind: u16) -> bool {
    matches!(
        kind,
        0 | 1 | 53 | 59 | 112 | 147 | 161 | 199 | 203 | 234 | 367 | 368 | 41
    )
}

/// The tree pass extension: the jungle and the evil grass grow trees like
/// the forest grass (`UndergroundJungleTrees` for the surface jungle — the
/// surface jungle uses the regular tree placement on the jungle grass).
pub(crate) fn tree_soil(kind: u16) -> bool {
    matches!(
        kind,
        TILE_GRASS
            | 147
            | TILE_JUNGLE_GRASS
            | TILE_CORRUPT_GRASS
            | TILE_CRIMSON_GRASS
            | TILE_MUSHROOM_GRASS
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worldgen::noise::Noise;
    use crate::worldgen::terrain;

    /// A flat synthetic terrain with the port's wall baseline (dirt 2 above
    /// `rock_layer`, stone 1 below) — the wall model the faithful pass reads.
    fn flat_world(surface: i32, width: i32, height: i32) -> (World, f64, f64) {
        let mut world = World::new("test".into(), width, height);
        let rock_layer = (surface + 80) as f64;
        for x in 0..width {
            for y in 0..height {
                let i = world.index(x, y);
                let mut tile = Tile::default();
                if y >= surface {
                    tile.kind = if (y as f64) < rock_layer { 0 } else { 1 };
                    tile.flags |= FLAG_ACTIVE;
                    if y > surface + 2 {
                        tile.wall = if (y as f64) < rock_layer { 2 } else { 1 };
                    }
                }
                world.tiles[i] = tile;
            }
        }
        (world, surface as f64, rock_layer)
    }

    /// DW-3.1: the full-pass ledger. The implementation runs against the
    /// test's own transcription of `JunglePass.cs:19-257` — the stage order,
    /// arg draws and clamps transcribed from the C#, the walker internals
    /// delegated to `tile_runner`/`mud_wall_runner` (both draw-ledger
    /// pinned). Tile-for-tile and stream-state equality pin the whole draw
    /// sequence: any stage reorder, missed draw or wrong range desyncs every
    /// later write.
    #[test]
    fn test_DW_3_1_jungle_pass_stage_ledger() {
        let (mut wa, ws_surf, rock) = flat_world(200, 512, 512);
        let mut wb = wa.clone();
        // The seed flags ride their own copy — the runner calls below borrow
        // `&mut wb` and read the flags in the same expression.
        let flags = wb.seeds;
        let (water, lava, origin, seed) = (300.0, 340.0, 102, 0x4A55_4E47u64);
        // The beach anchors: the fixed margins the ledger replays (left 140,
        // right 512−140 — the pre-beach-pass values).
        let (left_beach_end, right_beach_start) = (140, 512 - 140);
        let mut jr = Rng(seed);
        pass_jungle(
            &mut wa,
            origin,
            ws_surf,
            rock,
            water,
            lava,
            left_beach_end,
            right_beach_start,
            &mut jr,
        );
        // --- the transcription ---
        let mut rep = Rng(seed);
        let ws = 512 as f64 / 4200.0 * 1.5;
        let scale = |v: f64| (v * ws) as i32;
        let mut x = origin;
        let mut y = ((512 as f64 + rock) as i32) / 2;
        let (mut zx, mut zy) = (0, 0);
        let mut mud = |wb: &mut World, rep: &mut Rng, x: i32, y: i32, sx: f64| {
            let flags = wb.seeds;
            let s = rep.range(scale(250.0), scale(500.0));
            let st = rep.range(50, 150);
            tile_runner(
                wb,
                rep,
                x,
                y,
                s as f64,
                st,
                59,
                ws_surf,
                &TileRunnerOpts {
                    mud_wall: true,
                    speed_x: sx,
                    rock_layer: rock,
                    water_line: water,
                    lava_line: lava,
                    flags, ..TileRunnerOpts::default()
                },
            );
        };
        let mut gems = |wb: &mut World, rep: &mut Rng, x: i32, y: i32, lo: i32| {
            let flags = wb.seeds;
            for _ in 0..(6.0 * ws).ceil() as i32 {
                let gx = x + rep.range(scale(-125.0), scale(125.0));
                let gy = y + rep.range(scale(-125.0), scale(125.0));
                let gs = rep.range(3, 7);
                let gst = rep.range(3, 8);
                let gk = rep.range(lo, lo + 2);
                tile_runner(
                    wb,
                    rep,
                    gx,
                    gy,
                    gs as f64,
                    gst,
                    gk,
                    ws_surf,
                    &TileRunnerOpts { flags, ..TileRunnerOpts::default() },
                );
            }
        };
        let move_by = |rep: &mut Rng, x: i32, y: i32, xr: f64, yr: f64| -> (i32, i32) {
            let x = x + rep.range(((-xr) * ws) as i32, 1 + (xr * ws) as i32);
            let y = y + rep.range(((-yr) * ws) as i32, 1 + (yr * ws) as i32);
            (x, y.clamp(rock as i32, 512))
        };
        // The first pair (`:28-32`).
        (x, y) = move_by(&mut rep, x, y, 100.0, 100.0);
        zx += x;
        zy += y;
        mud(&mut wb, &mut rep, x, y, 3.0);
        gems(&mut wb, &mut rep, x, y, 63);
        // The second pair (`:34-38`); `oldX/oldY` saved before movement 3.
        (x, y) = move_by(&mut rep, x, y, 250.0, 150.0);
        let (old_x, old_y) = (x, y);
        zx += x;
        zy += y;
        mud(&mut wb, &mut rep, x, y, 0.0);
        gems(&mut wb, &mut rep, x, y, 65);
        // The third pair (`:40-46`).
        (x, y) = move_by(&mut rep, x, y, 400.0, 150.0);
        zx += x;
        zy += y;
        mud(&mut wb, &mut rep, x, y, -3.0);
        gems(&mut wb, &mut rep, x, y, 67);
        // The centre + the body (`:48-54`).
        let mut cx = zx / 3;
        let cy = zy / 3;
        let num = rep.range(scale(400.0), scale(600.0));
        let num2 = scale(25.0);
        let lo = left_beach_end + num / 2 + num2;
        let hi = right_beach_start - num / 2 - num2;
        cx = if cx < lo {
            lo
        } else if cx > hi {
            hi
        } else {
            cx
        };
        tile_runner(
            &mut wb,
            &mut rep,
            cx,
            cy,
            num as f64,
            10000,
            59,
            ws_surf,
            &TileRunnerOpts {
                mud_wall: true,
                speed_y: -20.0,
                no_y_change: true,
                rock_layer: rock,
                water_line: water,
                lava_line: lava,
                flags, ..TileRunnerOpts::default()
            },
        );
        // The tunnel (`:92-191`).
        let mut radius = rep.range(5, 11) as f64;
        let (mut px, mut py) = (cx as f64, cy as f64);
        let mut vel_x = rep.range(-10, 11) as f64 * 0.1;
        let mut vel_y = rep.range(10, 20) as f64 * 0.1;
        let mut branch_steps = 0i32;
        loop {
            if py < ws_surf {
                let value = (px as i32).clamp(10, 502);
                let value2 = (py as i32).clamp(10, 502).max(5);
                let clean = |dy: i32| {
                    wb.tile(value, value2 - dy)
                        .is_some_and(|t| t.wall == 0 && !t.active())
                };
                if clean(0) && clean(1) && clean(2) && clean(3) && clean(4) && clean(5) {
                    break;
                }
            }
            radius += rep.range(-20, 21) as f64 * 0.1;
            radius = radius.clamp(5.0, 10.0);
            let left = ((px - radius * 0.5) as i32).clamp(10, 502);
            let right = ((px + radius * 0.5) as i32).clamp(10, 502);
            let top = ((py - radius * 0.5) as i32).clamp(10, 502);
            let bottom = ((py + radius * 0.5) as i32).clamp(10, 502);
            for k in left..right {
                for l in top..bottom {
                    let manhattan = (k as f64 - px).abs() + (l as f64 - py).abs();
                    if manhattan < radius * 0.5 * (1.0 + rep.range(-10, 11) as f64 * 0.015) {
                        if let Some(tile) = wb.tile_mut(k, l) {
                            tile.flags &= !FLAG_ACTIVE;
                        }
                    }
                }
            }
            branch_steps += 1;
            if branch_steps > 10 && rep.range(0, 50) < branch_steps {
                branch_steps = 0;
                let dir = if rep.range(0, 2) == 0 { 2.0 } else { -2.0 };
                let s = rep.range(3, 20);
                let st = rep.range(10, 100);
                tile_runner(
                    &mut wb,
                    &mut rep,
                    px as i32,
                    py as i32,
                    s as f64,
                    st,
                    -1,
                    ws_surf,
                    &TileRunnerOpts {
                        speed_x: dir,
                        flags, ..TileRunnerOpts::default()
                    },
                );
            }
            px += vel_x;
            py += vel_y;
            vel_y += rep.range(-10, 11) as f64 * 0.01;
            if vel_y > 0.0 {
                vel_y = 0.0;
            }
            if vel_y < -2.0 {
                vel_y = -2.0;
            }
            vel_x += rep.range(-10, 11) as f64 * 0.1;
            if px < cx as f64 - 200.0 {
                vel_x += rep.range(5, 21) as f64 * 0.1;
            }
            if px > cx as f64 + 200.0 {
                vel_x -= rep.range(5, 21) as f64 * 0.1;
            }
            vel_x = vel_x.clamp(-1.5, 1.5);
        }
        // The holes (`:193-206`, with the port's 200k valve).
        for _ in 0..512 / 4 {
            let (mut num, mut num2) = (rep.range(20, 492), rep.range(ws_surf as i32 + 10, 312));
            let mut rolls = 0i32;
            while !matches!(wb.tile(num, num2).map_or(0, |t| t.wall), 64 | 15) {
                rolls += 1;
                if rolls > 200_000 {
                    break;
                }
                num = rep.range(20, 492);
                num2 = rep.range(ws_surf as i32 + 10, 312);
            }
            if rolls <= 200_000 {
                mud_wall_runner(&mut wb, &mut rep, num, num2, ws_surf);
            }
        }
        // The finishing touches (`:208-257`).
        let (mut num, mut num2) = (old_x, old_y);
        for _ in 0..(20.0 * ws).floor() as i32 + 1 {
            num += rep.range(scale(-5.0), scale(6.0));
            num2 += rep.range(scale(-5.0), scale(6.0));
            let s = rep.range(40, 100);
            let st = rep.range(300, 500);
            tile_runner(
                &mut wb,
                &mut rep,
                num,
                num2,
                s as f64,
                st,
                59,
                ws_surf,
                &TileRunnerOpts { flags, ..TileRunnerOpts::default() },
            );
        }
        let pick = |wb: &World, rep: &mut Rng| -> (i32, i32) {
            loop {
                let x = old_x + rep.range(scale(-600.0), scale(600.0));
                let y = old_y + rep.range(scale(-200.0), scale(200.0));
                if x >= 1
                    && x < 511
                    && y >= 1
                    && y < 511
                    && wb.tile(x, y).is_some_and(|t| t.kind == TILE_MUD)
                {
                    return (x, y);
                }
            }
        };
        for _ in 0..(10.0 * ws).floor() as i32 + 1 {
            (num, num2) = pick(&wb, &mut rep);
            for _ in 0..(8.0 * ws).floor() as i32 + 1 {
                num += rep.range(-30, 31);
                num2 += rep.range(-30, 31);
                let kind = if rep.range(0, 7) == 0 { -2 } else { -1 };
                let s = rep.range(10, 20);
                let st = rep.range(30, 70);
                tile_runner(
                    &mut wb,
                    &mut rep,
                    num,
                    num2,
                    s as f64,
                    st,
                    kind,
                    ws_surf,
                    &TileRunnerOpts { flags, ..TileRunnerOpts::default() },
                );
            }
        }
        for _ in 0..(300.0 * ws).floor() as i32 + 1 {
            (num, num2) = pick(&wb, &mut rep);
            let s = rep.range(4, 10);
            let st = rep.range(5, 30);
            tile_runner(
                &mut wb,
                &mut rep,
                num,
                num2,
                s as f64,
                st,
                1,
                ws_surf,
                &TileRunnerOpts { flags, ..TileRunnerOpts::default() },
            );
            if rep.range(0, 4) == 0 {
                // `:254`'s named-argument order — the gem draw first.
                let gk = rep.range(63, 69);
                let gx = num + rep.range(-1, 2);
                let gy = num2 + rep.range(-1, 2);
                let gs = rep.range(3, 7);
                let gst = rep.range(4, 8);
                tile_runner(
                    &mut wb,
                    &mut rep,
                    gx,
                    gy,
                    gs as f64,
                    gst,
                    gk,
                    ws_surf,
                    &TileRunnerOpts { flags, ..TileRunnerOpts::default() },
                );
            }
        }
        // The ledger: tile-identical worlds and the same final stream state.
        for (i, (a, b)) in wa.tiles.iter().zip(&wb.tiles).enumerate() {
            assert_eq!(
                (a.kind, a.wall, a.flags, a.liquid),
                (b.kind, b.wall, b.flags, b.liquid),
                "tile mismatch at ({},{})",
                i as i32 % 512,
                i as i32 / 512
            );
        }
        assert_eq!(jr.0, rep.0, "the final stream state");
    }

    /// DW-3.1: the `MudWallRunner` ledger (`WorldGen.cs:77212-77274`) — the
    /// radius/steps/velocity draws, then per iteration the shrinking rect's
    /// wobble draws and the two +-0.05 drift draws; the cleared diamonds sit
    /// below the surface only.
    #[test]
    fn test_DW_3_1_mud_wall_runner_ledger() {
        let mut world = World::new("t".into(), 200, 400);
        for i in 0..world.tiles.len() {
            let mut tile = Tile::default();
            tile.kind = 1;
            tile.flags |= FLAG_ACTIVE;
            tile.wall = 64;
            world.tiles[i] = tile;
        }
        let mut runner = Rng(0xBEEF);
        mud_wall_runner(&mut world, &mut runner, 100, 200, 100.0);
        let mut rep = Rng(0xBEEF);
        let radius = rep.range(8, 21) as f64;
        let steps = rep.range(8, 33) as f64;
        let mut num3 = steps;
        let (mut px, mut py) = (100.0, 200.0);
        let mut vel_x = rep.range(-10, 11) as f64 * 0.1;
        let mut vel_y = rep.range(-10, 11) as f64 * 0.1;
        while radius > 0.0 && num3 > 0.0 {
            let disc = radius * (num3 / steps);
            num3 -= 1.0;
            let left = ((px - disc * 0.5) as i32).clamp(0, 200);
            let right = ((px + disc * 0.5) as i32).clamp(0, 200);
            let top = ((py - disc * 0.5) as i32).clamp(0, 400);
            let bottom = ((py + disc * 0.5) as i32).clamp(0, 400);
            for k in left..right {
                for l in top..bottom {
                    let manhattan = (k as f64 - px).abs() + (l as f64 - py).abs();
                    if manhattan < radius * 0.5 * (1.0 + rep.range(-10, 11) as f64 * 0.015)
                        && l as f64 > 100.0
                    {
                        // The replay only mirrors the draws — the writes are
                        // asserted on the implementation's world below.
                    }
                }
            }
            px += vel_x;
            py += vel_y;
            vel_x += rep.range(-10, 11) as f64 * 0.05;
            vel_x = vel_x.clamp(-1.0, 1.0);
            vel_y += rep.range(-10, 11) as f64 * 0.05;
            vel_y = vel_y.clamp(-1.0, 1.0);
        }
        assert_eq!(runner.0, rep.0, "the mud-wall-runner ledger");
        assert_eq!(
            world.tile(100, 204).unwrap().wall,
            0,
            "the disc cleared walls below the surface"
        );
        assert_eq!(
            world.tile(100, 50).unwrap().wall,
            64,
            "above the surface the walls stand"
        );
    }

    /// DW-3.1: the vanilla substance on a fresh world — the mud body lines
    /// the stone band with both wall halves (the env's lava line splits at
    /// 340 - Next(0,4) - 50 = 286), the tunnel mouth opens the surface
    /// crust, the holes leave wall-0 gaps inside the lining, and the
    /// finishing dirt pockets and gems land inside the mud.
    #[test]
    fn jungle_pass_places_the_vanilla_substance() {
        let (mut world, ws_surf, rock) = flat_world(200, 512, 512);
        pass_jungle(
            &mut world,
            102,
            ws_surf,
            rock,
            300.0,
            340.0,
            140,
            512 - 140,
            &mut Rng(0x5EED),
        );
        let count = |pred: &dyn Fn(&Tile) -> bool| world.tiles.iter().filter(|t| pred(t)).count();
        let mud = count(&|t: &Tile| t.active() && t.kind == TILE_MUD);
        assert!(mud > 2000, "the mud body ({mud})");
        // (The 15/64 wall split is asserted by
        // `the_mud_body_splits_walls_at_the_lava_line` on the body stage
        // alone — the full pass's hole clearing strips the lining on
        // sub-vanilla worlds, see that test's note.)
        // The tunnel mouth: on this cave-free world the only carver that
        // reaches the crust rows is the tunnel (the pockets walk ±200*ws of
        // `oldY`, well below them) — inactive crust tiles in [200, 212) are
        // its mark (`JunglePass.cs:105-151`).
        let mouths = (90..310)
            .flat_map(|x| (200..212).map(move |y| (x, y)))
            .filter(|&(x, y)| world.tile(x, y).is_some_and(|t| !t.active()))
            .count();
        assert!(mouths > 0, "no tunnel mouth through the crust ({mouths})");
        // The holes: wall-0 gaps on the lined mud tiles (the MudWallRunner
        // clears walls only — the mud kind stays).
        let holes = count(&|t: &Tile| t.wall == 0 && t.active() && t.kind == TILE_MUD);
        assert!(holes > 20, "the mud-wall holes ({holes})");
        // The finishing dirt pockets inside the body's depth range.
        let dirt_in_body = (0..world.width)
            .flat_map(|x| (0..world.height).map(move |y| (x, y)))
            .filter(|&(x, y)| {
                (200..340).contains(&y)
                    && world.tile(x, y).is_some_and(|t| t.active() && t.kind == 1)
            })
            .count();
        assert!(dirt_in_body > 50, "the dirt pockets ({dirt_in_body})");
        let gems: usize = (63..=68)
            .map(|k| {
                world
                    .tiles
                    .iter()
                    .filter(|t| t.active() && t.kind == k)
                    .count()
            })
            .sum();
        assert!(gems > 4, "the gem runners ({gems})");
    }
    /// The mud body's wall split (`JunglePass.cs:53-54` +
    /// `WorldGen.cs:76875-76887`): the body runner lines the stone band with
    /// 64 above the lava-line pocket and 15 below it. Run on the body stage
    /// alone — the full pass's `MudWallRunner` holes strip the lining on
    /// sub-vanilla-sized worlds (a recorded small-world deviation), so the
    /// split is only assertable before the holes run.
    #[test]
    fn the_mud_body_splits_walls_at_the_lava_line() {
        let (mut world, ws_surf, rock) = flat_world(200, 512, 512);
        let flags = world.seeds;
        tile_runner(
            &mut world,
            &mut Rng(0x5EED),
            250,
            396,
            108.0,
            10000,
            59,
            ws_surf,
            &TileRunnerOpts {
                mud_wall: true,
                speed_x: 0.0,
                speed_y: -20.0,
                no_y_change: true,
                rock_layer: rock,
                water_line: 300.0,
                lava_line: 340.0,
                flags,
                ..TileRunnerOpts::default()
            },
        );
        // The band rows [281, 302) (the wall-above gate reads the port's
        // stone baseline below rock_layer 280); the split sits at
        // 340 - Next(0,4) - 50 = 286..290.
        let (mut w15, mut w64) = (0usize, 0usize);
        for x in 0..512 {
            for y in 281..302 {
                match world.tile(x, y).map_or(0, |t| t.wall) {
                    15 => w15 += 1,
                    64 => w64 += 1,
                    _ => {}
                }
            }
        }
        assert!(w15 > 50, "the deep half lined with 15 ({w15})");
        assert!(w64 > 50, "the shallow half lined with 64 ({w64})");
        // The 15s sit strictly below the 64s.
        let min15 = (0..512)
            .flat_map(|x| (281..302).map(move |y| (x, y)))
            .filter(|&(x, y)| world.tile(x, y).map_or(false, |t| t.wall == 15))
            .map(|(_, y)| y)
            .min()
            .unwrap();
        let max64 = (0..512)
            .flat_map(|x| (281..302).map(move |y| (x, y)))
            .filter(|&(x, y)| world.tile(x, y).map_or(false, |t| t.wall == 64))
            .map(|(_, y)| y)
            .max()
            .unwrap();
        assert!(
            min15 > max64 - 5,
            "15 below, 64 above (15@{min15}, 64@{max64})"
        );
    }

    /// DW-3.2: the sweep spreads jungle grass 60 onto exposed mud, leaves
    /// fully-enclosed mud alone (all 9 active+solid), and the clump scan
    /// removes tiny (< 20) isolated solid `CanBeClearedDuringGeneration`
    /// components. The clump fixtures sit in an air pocket — a stone-backed
    /// clump joins the stone mass's component and correctly survives.
    #[test]
    fn test_DW_3_2_mud_caves_spread_and_clumps() {
        let mut world = World::new("t".into(), 200, 300);
        let mut put = |world: &mut World, x: i32, y: i32, kind: u16| {
            let i = world.index(x, y);
            let mut tile = Tile::default();
            tile.kind = kind;
            tile.flags |= FLAG_ACTIVE;
            world.tiles[i] = tile;
        };
        // Stone everywhere below 40 (the backdrop).
        for x in 0..200 {
            for y in 40..300 {
                put(&mut world, x, y, 1);
            }
        }
        // The big mud slab with an air pocket above its top row.
        for x in 40..140 {
            for y in 100..160 {
                put(&mut world, x, y, TILE_MUD);
            }
        }
        for x in 40..140 {
            for y in 98..100 {
                let i = world.index(x, y);
                world.tiles[i] = Tile::default();
            }
        }
        // A tiny exposed mud clump (2x2) inside a 2-tile-deep air pocket at
        // (20, 200) — no clearable tile 4-adjacent to it.
        for dx in -3..=4 {
            for dy in -3..=3 {
                let i = world.index(20 + dx, 200 + dy);
                world.tiles[i] = Tile::default();
            }
        }
        for (x, y) in [(20i32, 200i32), (21, 200), (20, 201), (21, 201)] {
            put(&mut world, x, y, TILE_MUD);
        }
        // A fully-enclosed mud pocket: stone on all 8 neighbours.
        for (x, y) in [(170i32, 200i32), (171, 200), (170, 201), (171, 201)] {
            put(&mut world, x, y, TILE_MUD);
        }
        pass_mud_caves_to_jungle_grass(&mut world);
        // The slab's exposed top spread; the buried interior stayed mud.
        assert_eq!(
            world.tile(80, 100).unwrap().kind,
            TILE_JUNGLE_GRASS,
            "the exposed top spread"
        );
        assert_eq!(
            world.tile(80, 130).unwrap().kind,
            TILE_MUD,
            "the buried interior stayed mud"
        );
        // The exposed clump: spread then removed by the clump scan.
        for (x, y) in [(20i32, 200i32), (21, 200), (20, 201), (21, 201)] {
            assert!(
                !world.tile(x, y).unwrap().active(),
                "the isolated clump at ({x},{y}) was eliminated"
            );
        }
        // The enclosed pocket: no air in its 3x3 (the stone counts solid),
        // so it never spreads — and it stays, being part of the stone
        // mass's oversized component.
        assert_eq!(
            world.tile(170, 200).unwrap().kind,
            TILE_MUD,
            "the enclosed pocket never spread"
        );
        assert!(world.tile(170, 200).unwrap().active());
        // The slab's shell (a > 20 component) survives the scan.
        assert!(world.tile(80, 100).unwrap().active());
    }

    /// The `SpreadGrass` gate slice (`WorldGen.cs:75171-75199`): a fully
    /// solid 3x3 (no non-solid neighbour, no lava) never spreads; mud with
    /// air above does.
    #[test]
    fn spread_jungle_grass_gates_match_the_csharp() {
        let mut world = World::new("t".into(), 100, 100);
        let mut put = |world: &mut World, x: i32, y: i32, kind: u16| {
            let i = world.index(x, y);
            let mut tile = Tile::default();
            tile.kind = kind;
            tile.flags |= FLAG_ACTIVE;
            world.tiles[i] = tile;
        };
        // A 5x5 mud block: the centre is fully buried (no spread).
        for x in 40..45 {
            for y in 40..45 {
                put(&mut world, x, y, TILE_MUD);
            }
        }
        let mut spread = 0i32;
        spread_jungle_grass(
            &mut world,
            42,
            42,
            TILE_MUD,
            TILE_JUNGLE_GRASS,
            true,
            &mut spread,
        );
        assert_eq!(
            world.tile(42, 42).unwrap().kind,
            TILE_MUD,
            "buried mud stands"
        );
        // Open the tile above; the exposed tile now spreads.
        let i = world.index(42, 40);
        world.tiles[i] = Tile::default();
        let mut spread = 0i32;
        spread_jungle_grass(
            &mut world,
            42,
            41,
            TILE_MUD,
            TILE_JUNGLE_GRASS,
            true,
            &mut spread,
        );
        assert_eq!(
            world.tile(42, 41).unwrap().kind,
            TILE_JUNGLE_GRASS,
            "exposed mud spread"
        );
    }

    /// DW-3.2: `DirtToMud` places mud below `rockLayerLow` (the kind-59
    /// divers never climb back above it) and never writes on sand 53
    /// (`ignoreTileType`).
    #[test]
    fn test_DW_3_2_dirt_to_mud_below_rock_low_and_never_sand() {
        let mut world = World::new("t".into(), 200, 300);
        for x in 0..200 {
            for y in 0..300 {
                let i = world.index(x, y);
                let mut tile = Tile::default();
                if y >= 100 {
                    tile.kind = 0;
                    tile.flags |= FLAG_ACTIVE;
                    tile.wall = 1;
                }
                world.tiles[i] = tile;
            }
        }
        // A sand pocket deep in the dirt.
        for x in 80..90 {
            for y in 120..140 {
                let i = world.index(x, y);
                let mut tile = Tile::default();
                tile.kind = 53;
                tile.flags |= FLAG_ACTIVE;
                world.tiles[i] = tile;
            }
        }
        let sand_before = world
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == 53)
            .count();
        let mut runner = Rng(7);
        pass_dirt_to_mud(&mut world, 100.0, 120.0, 250.0, 280.0, 100.0, &mut runner);
        let mud: Vec<(i32, i32)> = (0..world.width)
            .flat_map(|x| (0..world.height).map(move |y| (x, y)))
            .filter(|&(x, y)| world.tile(x, y).is_some_and(|t| t.active() && t.kind == 59))
            .collect();
        assert!(!mud.is_empty(), "the runners placed mud");
        let min_y = mud.iter().map(|&(_, y)| y).min().unwrap();
        assert!(min_y >= 96, "no mud above rock_low - disc margin: {min_y}");
        let sand_after = world
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == 53)
            .count();
        assert_eq!(sand_after, sand_before, "sand ignored by every runner");
    }

    /// The runner count is the area formula's ceil (`WorldGen.cs:13127`:
    /// `(maxTilesX * maxTilesY) * 0.001` with `(double)i < num`): a 101x101
    /// world runs 11 runners, each drawing x, y, strength, steps plus the
    /// runner's own 2 velocity + 1 lottery + 2*steps draws (the sand world
    /// ignores every candidate before the wobble, and the ladder never
    /// fires below strength 50).
    #[test]
    fn dirt_to_mud_runner_count_matches_the_area_formula() {
        let mut world = World::new("t".into(), 101, 101);
        for i in 0..world.tiles.len() {
            let mut tile = Tile::default();
            tile.kind = 53;
            tile.flags |= FLAG_ACTIVE;
            world.tiles[i] = tile;
        }
        let mut runner = Rng(9);
        pass_dirt_to_mud(&mut world, 50.0, 60.0, 80.0, 90.0, 0.0, &mut runner);
        let mut rep = Rng(9);
        for _ in 0..11 {
            let _x = rep.range(0, 101);
            let _y = rep.range(50, 101);
            let _s = rep.range(2, 6);
            let st = rep.range(2, 40);
            rep.range(-10, 11); // initial velocity x
            rep.range(-10, 11); // initial velocity y
            rep.range(0, 4); // the num3 lottery
            for _ in 0..st {
                rep.range(-10, 11); // vx drift
                rep.range(-10, 11); // vy drift
            }
        }
        assert_eq!(runner.0, rep.0, "11 runners replayed the stream");
    }

    /// DW-3.4: the four interim span consumers place inside the re-derived
    /// band — the test re-derives `jungle_origin_x` from a fresh
    /// `pass_rng("Jungle")` (the same identity the spans step uses) and the
    /// band from `plan_spans`' formula, then checks the shrine brick, the
    /// hive blocks, the mushroom grass and the underground tree trunks.
    #[test]
    fn test_DW_3_4_interim_consumers_place_in_the_rederived_band() {
        let opts = crate::worldgen::GenOptions {
            seed: 77,
            ..Default::default()
        };
        let world = crate::worldgen::generate(&opts);
        // The origin replay (the spans step's draw, `WorldGen.cs:11371-11375`).
        let mut origin_rng = crate::worldgen::pipeline::Streams::new(opts.seed).pass_rng("Jungle");
        let percent = origin_rng.range(15, 30);
        let origin = (world.width as f64 * (percent as f64 * 0.01)) as i32;
        // The band (the `plan_spans` formula).
        let ws = world.width as f64 / 4200.0 * 1.5;
        let reach = (700.0 * ws) as i32;
        // The band (the `plan_spans` formula) over the Reset beach draws
        // (`WorldGen.cs:11415-11440` — the terrain step's stand-in, replayed).
        let mut brng = crate::worldgen::pipeline::Streams::new(opts.seed).pass_rng("Beaches");
        let left_beach_end = brng.range(300, 340) + 40;
        let _right_beach_start = world.width - (brng.range(300, 340) + 20);
        let jungle_start = (origin - reach).max(left_beach_end + 40).max(20);
        let jungle_end = (origin + reach)
            .min(world.width - 60)
            .max(jungle_start + 40);
        let in_band = |x: i32| x >= jungle_start && x < jungle_end;
        let (mut bad, mut shrine, mut hive, mut shroom, mut trees) = (0usize, 0, 0, 0, 0);
        let width = world.width;
        for (i, t) in world.tiles.iter().enumerate() {
            let x = i as i32 % width;
            let y = i as i32 / width;
            if !t.active() {
                continue;
            }
            let underground_tree = t.kind == 5 && y as f64 > world.world_surface;
            match t.kind {
                119 => {
                    shrine += 1;
                    bad += !in_band(x) as usize;
                }
                225 => {
                    hive += 1;
                    bad += !in_band(x) as usize;
                }
                70 => {
                    shroom += 1;
                    // M5-BIOME-2: the faithful `GlowingMushroomPatches` is NOT
                    // band-limited — the cluster roll is the middle 60% and
                    // the `ShroomPatch` walk drifts globally
                    // (`WorldGen.cs:12578-12582`, `:79962-79968`). The
                    // shape-level pass pinned cx to the jungle band; the
                    // vanilla's mushroom domains are global, so the bucket
                    // only counts, it no longer feeds `bad`.
                }
                _ if underground_tree => {
                    trees += 1;
                    bad += !in_band(x) as usize;
                }
                _ => {}
            }
        }
        assert!(shrine > 0, "the shrines placed");
        assert!(hive > 0, "the hives placed");
        assert!(shroom > 0, "the mushroom patches placed");
        assert!(trees > 0, "the underground trees placed");
        assert_eq!(bad, 0, "{bad} consumer tiles outside the band");
        // The band's substance: the faithful mud lives inside it.
        let band_mud = (0..world.width)
            .flat_map(|x| (0..world.height).map(move |y| (x, y)))
            .filter(|&(x, y)| {
                in_band(x)
                    && world
                        .tile(x, y)
                        .is_some_and(|t| t.active() && (t.kind == 59 || t.kind == 60))
            })
            .count();
        assert!(band_mud > 1000, "the band's mud ({band_mud})");
    }

    // --- W4-g40 marble/granite residual witnesses (WRITTEN NOT RUN —
    // code-only mode; the merge gate runs them) ------------------------------

    /// `MarbleBiome.SlabStates` (`MarbleBiome.cs:14-47`): every pattern's
    /// per-cell predicate transcribed independently over the 3x3 slab grid.
    #[test]
    fn w4_g40_slab_state_predicates_match_the_csharp_delegates() {
        for x in 0..3 {
            for y in 0..3 {
                assert!(!SlabState::Empty.active_at(x, y, 3));
                assert!(SlabState::Solid.active_at(x, y, 3));
                assert_eq!(SlabState::HalfBrick.active_at(x, y, 3), y >= 3 / 2);
                assert_eq!(SlabState::BottomRightFilled.active_at(x, y, 3), x >= 3 - y);
                assert_eq!(SlabState::BottomLeftFilled.active_at(x, y, 3), x < y);
                assert_eq!(SlabState::TopRightFilled.active_at(x, y, 3), x > y);
                assert_eq!(SlabState::TopLeftFilled.active_at(x, y, 3), x < 3 - y);
            }
        }
    }

    /// `MarbleBiome.SmoothSlope`'s mask map (`MarbleBiome.cs:88-108`): the
    /// five named cases plus the default re-solidify, over a 3x3 slab grid
    /// with the centre solid and the four neighbours toggled per mask.
    #[test]
    fn w4_g40_marble_smooth_maps_the_five_named_masks() {
        let cases = [
            (10u8, SlabState::TopLeftFilled),
            (9, SlabState::TopRightFilled),
            (6, SlabState::BottomLeftFilled),
            (5, SlabState::BottomRightFilled),
            (4, SlabState::HalfBrick),
            (0, SlabState::Solid),
            (15, SlabState::Solid),
            (3, SlabState::Solid),
        ];
        for (mask, want) in cases {
            // gw = gh = 3, centre (1, 1); bit 3 = above, bit 2 = below,
            // bit 1 = left, bit 0 = right (`:88`).
            let mut slabs = vec![(SlabState::Empty, true); 9];
            slabs[4].0 = SlabState::Solid;
            if mask & 8 != 0 {
                slabs[1].0 = SlabState::Solid;
            }
            if mask & 4 != 0 {
                slabs[7].0 = SlabState::Solid;
            }
            if mask & 2 != 0 {
                slabs[3].0 = SlabState::Solid;
            }
            if mask & 1 != 0 {
                slabs[5].0 = SlabState::Solid;
            }
            marble_smooth(&mut slabs, 1, 1, 3, 3);
            assert_eq!(slabs[4].0, want, "mask {mask}");
            assert!(slabs[4].1, "WithState keeps HasWall (`:64-67`)");
        }
        // An empty centre is untouched (`:82`'s IsSolid gate).
        let mut slabs = vec![(SlabState::Empty, false); 9];
        marble_smooth(&mut slabs, 1, 1, 3, 3);
        assert_eq!(slabs[4], (SlabState::Empty, false));
    }

    /// `TileID.Sets.Ore` (`TileID.cs:614`) — the literal set transcribed
    /// from the decompile, both directions (the old port list carried
    /// 170/171/172/174/176/408/224/403, none of which are Ore).
    #[test]
    fn w4_g40_tile_is_ore_is_the_tileid_sets_ore_literal() {
        let ore = [
            7u16, 166, 6, 167, 9, 168, 8, 169, 22, 204, 37, 58, 107, 221, 108, 222, 111, 223,
            211,
        ];
        for kind in 0..700u16 {
            assert_eq!(tile_is_ore(kind), ore.contains(&kind), "kind {kind}");
        }
    }

    /// `MarbleBiome.PlaceSlab` (`MarbleBiome.cs:112-161`) with an EMPTY slab
    /// (every cell inactive): the per-column draw ledger replays exactly —
    /// the edge skip roll, the two extent rolls, and NO `Next(4)` draws (the
    /// written cells are inactive, so both `SolidTile` gates fail and the
    /// short-circuit spends nothing). The writes are `ResetToType(367)` +
    /// active(false) + wall 178; `Tile.SmoothSlope` refuses inactive cells.
    #[test]
    fn w4_g40_marble_place_slab_empty_ledger_and_writes() {
        let (mut world, _, _) = flat_world(20, 60, 60);
        world.seed = 5;
        let (ox, oy) = (25, 35);
        // Carve the slab's reach (plus the SmoothSlope/PlaceTight neighbour
        // halo) to air so every `SolidTile` gate fails and the ledger is the
        // column rolls alone (no Next(4) draws).
        for x in (ox - 2)..=(ox + 4) {
            for y in (oy - 6)..=(oy + 9) {
                let i = world.index(x, y);
                world.tiles[i].flags &= !FLAG_ACTIVE;
            }
        }
        let mut world_b = world.clone();
        let mut rng = Rng(0x9A11);
        marble_place_slab(&mut world, (SlabState::Empty, true), ox, oy, &mut rng);
        // The independent draw replay (`:125-138`).
        let mut rep = Rng(0x9A11);
        let (mut num4, mut num5) = (0i32, 3i32);
        let mut touched: Vec<(i32, i32)> = Vec::new();
        for i in -1..4 {
            if (i == -1 || i == 3) && rep.chance(2) {
                continue;
            }
            if rep.chance(2) {
                num4 -= 1;
            }
            if rep.chance(2) {
                num5 += 1;
            }
            for j in num4..num5 {
                touched.push((ox + i, oy + j));
            }
        }
        assert_eq!(rng.0, rep.0, "the column rolls replay the stream");
        assert!(!touched.is_empty(), "the witness is not vacuous");
        for (x, y) in touched {
            let t = world.tile(x, y).unwrap();
            assert_eq!(t.kind, TILE_MARBLE, "ResetToType(367) at {x},{y}");
            assert!(!t.active(), "the Empty state writes active(false)");
            assert_eq!(t.wall, WALL_MARBLE, "HasWall paints 178");
            assert_eq!((t.frame_x, t.frame_y, t.liquid, t.meta), (0, 0, 0, 0));
        }
        // a == b: a fresh same-seed run over a clone is tile- and stream-equal.
        let mut rng_b = Rng(0x9A11);
        marble_place_slab(&mut world_b, (SlabState::Empty, true), ox, oy, &mut rng_b);
        assert_eq!(world.tiles, world_b.tiles);
        assert_eq!(rng.0, rng_b.0);
    }

    /// The drunk swap (`MarbleBiome.cs:116-120`): the slab paints granite
    /// 368 / wall 180 instead; and the ore arm (`:142`) keeps an ore kind
    /// through `ResetToType`.
    #[test]
    fn w4_g40_marble_place_slab_drunk_swap_and_ore_preservation() {
        let (mut world, _, _) = flat_world(20, 60, 60);
        world.seed = 5;
        world.seeds |= crate::seeds::SeedFlags::DRUNK;
        // An ore field across the slab's reach plus the neighbour halo
        // (kind 7 = Copper, `Sets.Ore`), INACTIVE so the PlaceTight
        // SolidTile gates fail and no stalactite overwrites the paint —
        // `ResetToType` reads the kind regardless of the active bit.
        for x in 18..36 {
            for y in 28..46 {
                let i = world.index(x, y);
                world.tiles[i].kind = 7;
                world.tiles[i].flags &= !FLAG_ACTIVE;
            }
        }
        let mut rng = Rng(0x9A11);
        marble_place_slab(&mut world, (SlabState::Empty, true), 25, 35, &mut rng);
        let painted: Vec<_> = world
            .tiles
            .iter()
            .filter(|t| t.wall == WALL_GRANITE)
            .collect();
        assert!(!painted.is_empty(), "the drunk slab painted wall 180");
        for t in painted {
            assert_eq!(t.kind, 7, "the ore kind survives ResetToType");
            assert!(!t.active());
        }
        assert!(
            !world.tiles.iter().any(|t| t.wall == WALL_MARBLE),
            "no marble wall in the drunk world"
        );
    }

    /// `Tile.SmoothSlope`'s single-cell mask map (`Tile.cs:814-845`): the
    /// five named arms over a stone cell with toggled neighbours. The
    /// `flag2` veto (an active-but-not-solid tile above) blocks the two
    /// top-corner arms (`:817`/`:824`).
    #[test]
    fn w4_g40_tile_smooth_slope_one_maps_the_mask_arms() {
        // Neighbour layout: above (10,9), below (10,11), left (9,10),
        // right (11,10). mask = above<<3 | below<<2 | left<<1 | right.
        let run = |above: Option<u16>, below: bool, left: bool, right: bool| -> (u8, bool) {
            let mut w = World::new("t".into(), 20, 20);
            let set = |w: &mut World, x: i32, y: i32, kind: u16| {
                let i = w.index(x, y);
                w.tiles[i].kind = kind;
                w.tiles[i].flags |= FLAG_ACTIVE;
            };
            set(&mut w, 10, 10, TILE_STONE);
            // A pre-existing slope: the veto arms must NOT clear it (the
            // C# breaks without a write), the default arm must.
            let c = w.index(10, 10);
            w.tiles[c].set_slope(2);
            if let Some(kind) = above {
                set(&mut w, 10, 9, kind);
            }
            if below {
                set(&mut w, 10, 11, TILE_STONE);
            }
            if left {
                set(&mut w, 9, 10, TILE_STONE);
            }
            if right {
                set(&mut w, 11, 10, TILE_STONE);
            }
            tile_smooth_slope_one(&mut w, 10, 10);
            let t = w.tile(10, 10).unwrap();
            (t.slope(), t.half_brick())
        };
        // mask 6 (below+left, above empty) → slope 1.
        assert_eq!(run(None, true, true, false), (1, false));
        // mask 5 (below+right) → slope 2.
        assert_eq!(run(None, true, false, true), (2, false));
        // mask 4 (below only) → half-brick.
        assert_eq!(run(None, true, false, false), (0, true));
        // mask 10 (above+left) → slope 3; mask 9 (above+right) → slope 4.
        assert_eq!(run(Some(TILE_STONE), false, true, false), (3, false));
        assert_eq!(run(Some(TILE_STONE), false, false, true), (4, false));
        // The flag2 veto: a platform above (solidTop — active, not
        // SolidOrSloped) blocks both top-corner arms → NO write (the
        // pre-existing slope 2 survives; `:817`/`:824` break).
        assert_eq!(run(Some(19), false, true, false), (2, false));
        assert_eq!(run(Some(19), false, false, true), (2, false));
        // Default (all four solid) → flat.
        assert_eq!(run(Some(TILE_STONE), true, true, true), (0, false));
    }

    /// `GraniteBiome.ShouldUseLava` (`GraniteBiome.cs:171-196`): the
    /// lava-line gate (`tileOrigin.Y + 100 <= lavaLine - 30` refuses) and
    /// the ice-family veto inside the 100x100 scan.
    #[test]
    fn w4_g40_granite_should_use_lava_gate_and_ice_veto() {
        let mut w = World::new("t".into(), 400, 400);
        // Chamber origin (100, 100): centre y = 200. lavaLine 229 → the
        // gate `200 <= 229 - 30 = 199` is false → the scan runs (empty → true).
        assert!(granite_should_use_lava(&w, 100, 100, 229));
        // lavaLine 230 → `200 <= 200` → refused before any scan.
        assert!(!granite_should_use_lava(&w, 100, 100, 230));
        // The veto kinds 147/161/162/163/200 (`:188`), at the scan's far
        // corner (centre + 49, + 49).
        for kind in [147u16, 161, 162, 163, 200] {
            let mut v = w.clone();
            let i = v.index(249, 249);
            v.tiles[i].kind = kind;
            v.tiles[i].flags |= FLAG_ACTIVE;
            assert!(!granite_should_use_lava(&v, 100, 100, 229), "kind {kind}");
        }
        // A stone tile does not veto; an INACTIVE ice tile does not either.
        let i = w.index(249, 249);
        w.tiles[i].kind = TILE_STONE;
        w.tiles[i].flags |= FLAG_ACTIVE;
        assert!(granite_should_use_lava(&w, 100, 100, 229));
        w.tiles[i].kind = 161;
        w.tiles[i].flags &= !FLAG_ACTIVE;
        assert!(granite_should_use_lava(&w, 100, 100, 229));
    }

    /// `GraniteBiome.CleanupTiles` (`GraniteBiome.cs:247-294`): the census
    /// is `SolidTile` (half-bricks/slopes do not count), the lone solid
    /// clears to air with the granite wall painted after, and the drunk
    /// world paints 178.
    #[test]
    fn w4_g40_granite_cleanup_census_and_clear() {
        let mut w = World::new("t".into(), 40, 40);
        // One active magma cell at map (10, 10) → world (10, 10) (ox = oy = 0).
        let side = 200usize;
        let mut map = vec![MagmaCell::empty(1.0); side * side];
        map[10 * side + 10] = MagmaCell::flow(5.0, 0.0);
        let i = w.index(10, 10);
        w.tiles[i].kind = TILE_GRANITE;
        w.tiles[i].flags |= FLAG_ACTIVE;
        // A half-brick neighbour: active but NOT SolidTile (`:274`).
        let j = w.index(11, 10);
        w.tiles[j].kind = TILE_STONE;
        w.tiles[j].flags |= FLAG_ACTIVE | FLAG_HALF_BRICK;
        granite_cleanup(&mut w, 0, 0, &map, (10, 10, 10, 10));
        let t = w.tile(10, 10).unwrap();
        assert!(!t.active(), "1 solid neighbour < 3 → cleared");
        assert_eq!(t.wall, WALL_GRANITE, "the wall rides after the clear");
        assert_eq!(t.kind, TILE_GRANITE, "ClearTile keeps the kind");
        // Three solid neighbours (self + two) survive.
        let mut w2 = World::new("t".into(), 40, 40);
        for (x, y) in [(10, 10), (9, 10), (10, 9)] {
            let i = w2.index(x, y);
            w2.tiles[i].kind = TILE_GRANITE;
            w2.tiles[i].flags |= FLAG_ACTIVE;
        }
        granite_cleanup(&mut w2, 0, 0, &map, (10, 10, 10, 10));
        assert!(w2.tile(10, 10).unwrap().active(), "3 solid neighbours keep it");
        // The drunk wall.
        let mut w3 = World::new("t".into(), 40, 40);
        w3.seeds |= crate::seeds::SeedFlags::DRUNK;
        let i = w3.index(10, 10);
        w3.tiles[i].kind = TILE_MARBLE;
        w3.tiles[i].flags |= FLAG_ACTIVE;
        granite_cleanup(&mut w3, 0, 0, &map, (10, 10, 10, 10));
        assert_eq!(w3.tile(10, 10).unwrap().wall, WALL_MARBLE);
    }

    /// `GraniteBiome.PlaceDecorations`' FastRandom ledger
    /// (`GraniteBiome.cs:296-330`): over an all-air world every active map
    /// cell spends `Next(8)` then (the active-bit gate fails → no stalactite
    /// draws) the unconditional `Next(2)`. The per-cell stream is
    /// `FastRandom(worldSeed).WithModifier(65440).WithModifier(x, y)` —
    /// replayed here independently. No worldgen-stream draw exists (the
    /// function takes no `Rng`).
    #[test]
    fn w4_g40_granite_decorations_fastrandom_ledger() {
        let mut w = World::new("t".into(), 40, 40);
        w.seed = 0x1234_5678;
        let side = 200usize;
        let mut map = vec![MagmaCell::empty(1.0); side * side];
        for my in 10..=12 {
            for mx in 10..=12 {
                map[my * side + mx] = MagmaCell::flow(5.0, 0.0);
            }
        }
        let before = w.tiles.clone();
        granite_decorations(&mut w, 0, 0, &map, (10, 12, 10, 12));
        // All cells inactive: the smoothing rolls fire but write nothing.
        assert_eq!(w.tiles, before, "no writes on an all-air chamber");
        // The ledger: the C# walk is X-major; each cell's draws come from
        // its own WithModifier copy, so the replay is per-cell.
        let fast = FastRandom::new(w.seed as i32 as u64).with_modifier_u64(65440);
        for mx in 10..=12i32 {
            for my in 10..=12i32 {
                let mut f = fast.with_modifier(mx, my);
                let eight = f.next(8);
                assert!((0..8).contains(&eight));
                // The active-bit gate fails (all air) → straight to Next(2).
                let _two = f.next(2);
            }
        }
        // a == b: the pass is a pure function of (world, map) — a second
        // run over a clone is tile-equal.
        let mut w2 = World::new("t".into(), 40, 40);
        w2.seed = 0x1234_5678;
        granite_decorations(&mut w2, 0, 0, &map, (10, 12, 10, 12));
        assert_eq!(w.tiles, w2.tiles);
    }

    /// The decorations' firing arm, pinned to a pre-computed seed: with
    /// world seed 10368 the cell (10,10) FastRandom copy rolls
    /// `Next(8) == 0`, then below-small `Next(2) == 0`, variation
    /// `Next(3) == 0`, above-small `Next(2) == 1`, variation `Next(3) == 2`,
    /// smoothing `Next(2) == 1` (the LCG replayed independently in Python
    /// from `FastRandom.cs:27-35/:63-81`). A lone granite cell at (10,10)
    /// with air above/below therefore grows: the below hanging SMALL
    /// granite stalactite (frame 432, frame_y 72) at (10,11), then the
    /// above rising TALL pair (frame 432+2*18 = 468, frame_y 36/54) at
    /// (10,8)/(10,9) — `PlaceUncheckedStalactite`'s granite arms
    /// (`WorldGen.cs:38448-38476` hanging, `:38582-38610` rising). The
    /// smoothing roll is 1 → no slope write.
    #[test]
    fn w4_g40_granite_decorations_firing_arm_writes_the_stalactite_pair() {
        let mut w = World::new("t".into(), 40, 40);
        w.seed = 10368;
        let i = w.index(10, 10);
        w.tiles[i].kind = TILE_GRANITE;
        w.tiles[i].flags |= FLAG_ACTIVE;
        let side = 200usize;
        let mut map = vec![MagmaCell::empty(1.0); side * side];
        map[10 * side + 10] = MagmaCell::flow(5.0, 0.0);
        granite_decorations(&mut w, 0, 0, &map, (10, 10, 10, 10));
        let below = w.tile(10, 11).unwrap();
        assert_eq!(below.kind, 165, "the hanging stalactite");
        assert!(below.active());
        assert_eq!((below.frame_x, below.frame_y), (432, 72), "small granite");
        let top = w.tile(10, 8).unwrap();
        assert_eq!(top.kind, 165);
        assert_eq!((top.frame_x, top.frame_y), (468, 36), "tall rising upper");
        let mid = w.tile(10, 9).unwrap();
        assert_eq!(mid.kind, 165);
        assert_eq!((mid.frame_x, mid.frame_y), (468, 54), "tall rising lower");
        // The support cell survives (the rising pair writes y-1/y of the
        // call site 9, i.e. cells 8/9); the smoothing roll was 1.
        let support = w.tile(10, 10).unwrap();
        assert_eq!(support.kind, TILE_GRANITE);
        assert_eq!(support.slope(), 0);
        assert!(!support.half_brick());
    }

    /// `MarbleBiome.Place` end-to-end a == b on the flat synthetic terrain:
    /// two same-seed runs are tile- and stream-equal (the slab grid, the
    /// smoothing, the arc emission, the PlaceTight arms and the
    /// SmoothSlope cascade all ride the one pass stream).
    #[test]
    fn w4_g40_marble_place_is_reproducible_for_a_seed() {
        let (mut wa, _, rock) = flat_world(120, 512, 512);
        wa.seed = 42;
        let mut wb = wa.clone();
        let (mut ra, mut rb) = (Rng(0x0CEA), Rng(0x0CEA));
        let ok_a = marble_place(&mut wa, 256, rock as i32 + 100, &mut ra);
        let ok_b = marble_place(&mut wb, 256, rock as i32 + 100, &mut rb);
        assert_eq!(ok_a, ok_b);
        assert!(ok_a, "the flat stone site passes BiomeTileCheck");
        assert_eq!(wa.tiles, wb.tiles, "tile-for-tile");
        assert_eq!(ra.0, rb.0, "the draw ledger");
        // The cave substance: marble tiles and walls landed.
        assert!(wa.tiles.iter().any(|t| t.kind == TILE_MARBLE && t.active()));
        assert!(wa.tiles.iter().any(|t| t.wall == WALL_MARBLE));
    }
}
