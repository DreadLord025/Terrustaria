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

use super::tile_runner::{tile_runner, TileRunnerOpts};
use super::{Rng, TILE_DIRT, TILE_GRASS, TILE_SAND, TILE_SNOW, TILE_STONE, WALL_DIRT, WALL_STONE};
use crate::tile::{Tile, FLAG_ACTIVE};
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
            ..TileRunnerOpts::default()
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
            ..TileRunnerOpts::default()
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
            &TileRunnerOpts::default(),
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
                    ..TileRunnerOpts::default()
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
            &TileRunnerOpts::default(),
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
                &TileRunnerOpts::default(),
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
            &TileRunnerOpts::default(),
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
                &TileRunnerOpts::default(),
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
/// (`growth.rs::spread_grass` carries an earlier slice whose neighbour gate
/// is inverted vs the C# — see the phase discovery; its consumers are
/// outside this phase's file scope.)
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
                ..TileRunnerOpts::default()
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
    let num3 = world.width as f64 / 4200.0;
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
    // `_slabs` — (num + 2) x (num2 + 2) cells of (solid, hasWall).
    let gw = (num + 2) as usize;
    let gh = (num2 + 2) as usize;
    let mut slabs: Vec<(bool, bool)> = vec![(false, true); gw * gh];
    let slab = |slabs: &[(bool, bool)], x: i32, y: i32| -> (bool, bool) {
        if x < 0 || y < 0 || x as usize >= gw || y as usize >= gh {
            (false, false)
        } else {
            slabs[y as usize * gw + x as usize]
        }
    };
    for i in -1..(num + 1) {
        let num4 = (i - num / 2) as f64 / num as f64 + 0.5;
        let num5 = (((0.5 - (num4 - 0.5).abs()) * 5.0) as i32) - 2;
        for j in -1..(num2 + 1) {
            let (mut has_wall, mut flag) = (true, false);
            // `IsGroupSolid(i * 3 + origin.X, j * 3 + origin.Y, 3)`.
            let gx = i * 3 + origin_x;
            let gy = j * 3 + origin_y;
            let mut solid_count = 0;
            for a in 0..3 {
                for b in 0..3 {
                    if world
                        .tile(gx + a, gy + b)
                        .is_some_and(|t| t.active() && (t.slope() != 0 || !t.half_brick()))
                    {
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
            slabs[(j + 1) as usize * gw + (i + 1) as usize] = (flag, has_wall);
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

/// `MarbleBiome.SmoothSlope` (`MarbleBiome.cs:79-110`): the four-neighbour
/// bitmask maps a solid slab to the corner/half-brick states.
fn marble_smooth(slabs: &mut [(bool, bool)], x: i32, y: i32, gw: i32, gh: i32) {
    let at = |slabs: &[(bool, bool)], x: i32, y: i32| -> bool {
        if x < 0 || y < 0 || x >= gw || y >= gh {
            false
        } else {
            slabs[y as usize * gw as usize + x as usize].0
        }
    };
    if !at(slabs, x, y) {
        return;
    }
    let mask = ((at(slabs, x, y - 1) as i32) << 3)
        | ((at(slabs, x, y + 1) as i32) << 2)
        | ((at(slabs, x - 1, y) as i32) << 1)
        | (at(slabs, x + 1, y) as i32);
    // The port keeps the state as the active pattern: the corner states carry
    // no C#-equivalent tile flag, so the shape stays solid/empty here and the
    // visual slope comes from the per-tile `Tile.SmoothSlope` inside
    // `PlaceSlab` (documented in ANALYSIS).
    let _ = mask;
}

/// `MarbleBiome.PlaceSlab` (`MarbleBiome.cs:112-161`): the edge-column skips,
/// the extent jitter, the ore preservation, the wall and the per-tile
/// `Tile.SmoothSlope`.
fn marble_place_slab(
    world: &mut World,
    slab: (bool, bool),
    origin_x: i32,
    origin_y: i32,
    rng: &mut super::Rng,
) {
    let (mut num4, mut num5) = (-1i32, 0i32);
    let (scale_lo, scale_hi) = (-1, 4);
    for i in scale_lo..scale_hi {
        if (i == -1 || i == scale_hi - 1) && rng.chance(2) {
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
            // `ResetToType` keeps the ore tiles (`TileID.Sets.Ore`).
            if !tile_is_ore(t.kind) {
                t.kind = TILE_MARBLE;
                t.frame_x = -1;
                t.frame_y = -1;
            }
            t.flags = if slab.0 {
                t.flags | FLAG_ACTIVE
            } else {
                t.flags & !FLAG_ACTIVE
            };
            if slab.1 {
                t.wall = WALL_MARBLE;
            }
            // The vanilla's pots (`PlaceTight`) have no port tile yet.
        }
    }
}

/// The C#'s `TileID.Sets.Ore` check (`TileID.cs`) — the slab keeps the ores.
fn tile_is_ore(kind: u16) -> bool {
    matches!(
        kind,
        7 | 6
            | 8
            | 9
            | 166
            | 167
            | 168
            | 169
            | 170
            | 171
            | 172
            | 174
            | 176
            | 408
            | 221
            | 222
            | 223
            | 224
            | 403
    )
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
/// plus `BiomeTileCheck`) and the `WorldGen.cs:12897-12915` retry, then placed
/// in a second phase.
pub(crate) fn pass_granite(world: &mut World, rock_layer: f64, rng: &mut super::Rng) {
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
        let num5 = placed as f64 / caves as f64;
        // RandomRectanglePoint(band x, rockLayer + 20, band, depth).
        let mut x = (num5 * (w as f64 - 200.0)) as i32 + 100 + rng.range(0, band);
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
        // `GraniteBiome.CanPlace` (`GraniteBiome.cs:53-60`): inactive origin
        // plus the shared BiomeTileCheck.
        let in_cave = !world.tile(x, y).is_some_and(|t| t.active());
        if in_cave && !biome_tile_check(world, x, y) {
            points.push((x, y));
            placed += 1;
            attempts = 0;
        } else if attempts > w * 10 {
            caves = placed;
            attempts = 0;
        }
    }
    for (x, y) in points {
        granite_place(world, x, y);
    }
}

/// `GraniteBiome.Place` (`GraniteBiome.cs:62-77`): the 200x200 magma map, the
/// pressure simulation, the granite placement and the cleanup. The magma
/// map's first index is the world X and the second the world Y
/// (`BuildMagmaMap`'s `i2 = i + tileOrigin.X`).
fn granite_place(world: &mut World, origin_x: i32, origin_y: i32) -> bool {
    // `GraniteBiome.CanPlace` (`GraniteBiome.cs:53-60`): the origin tile must
    // be inactive — the cave the granite chamber grows out of — plus the
    // shared `BiomeTileCheck`. The port's pipeline now runs granite after the
    // cave carving (the vanilla order), so the gate holds.
    if world.tile(origin_x, origin_y).is_some_and(|t| t.active())
        || biome_tile_check(world, origin_x, origin_y)
    {
        return false;
    }
    // The origin is offset by half the map (`Place`'s centring).
    let (ox, oy) = (origin_x - 100, origin_y - 100);
    // `BuildMagmaMap` (`GraniteBiome.cs:79-93`): resistance 1 for non-solid,
    // 4 for solid.
    let side = 200usize;
    let mut source: Vec<MagmaCell> = vec![MagmaCell::empty(1.0); side * side];
    for mx in 0..side {
        for my in 0..side {
            let solid = world
                .tile(ox + mx as i32, oy + my as i32)
                .is_some_and(|t| t.active());
            source[my * side + mx] = MagmaCell::empty(if solid { 4.0 } else { 1.0 });
        }
    }
    let mut target = source.clone();
    let vecs = normalised_vectors();
    // The active box starts 0x0 at the centre and grows (`SimulatePressure`).
    let (mut mx0, mut mx1, mut my0, mut my1) = (side / 2, side / 2, side / 2, side / 2);
    for iter in 0..300 {
        for my in my0..=my1 {
            for mx in mx0..=mx1 {
                let cell = source[my * side + mx];
                if !cell.active {
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
                    let flow = (sum - gradient - cell.pressure).max(0.0)
                        + gradient
                        + cell.pressure * 0.875
                        - cell.resistance;
                    target[my * side + mx] =
                        MagmaCell::flow(flow.max(0.0), (cell.resistance - flow * 0.02).max(0.0));
                }
            }
        }
        if iter < 2 {
            target[(side / 2) * side + side / 2] = MagmaCell::flow(25.0, 0.0);
        }
        std::mem::swap(&mut source, &mut target);
    }
    let area = (mx0, mx1, my0, my1);
    granite_place_tiles(world, ox, oy, &source, area);
    granite_cleanup(world, ox, oy, &source, area);
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

/// `GraniteBiome.PlaceGranite` (`GraniteBiome.cs:171-245`): the sine-gated
/// placement, the ore preservation and the lava flag.
fn granite_place_tiles(
    world: &mut World,
    ox: i32,
    oy: i32,
    map: &[MagmaCell],
    area: (usize, usize, usize, usize),
) {
    let side = 200usize;
    let (mx0, mx1, my0, my1) = area;
    for my in my0..=my1 {
        for mx in mx0..=mx1 {
            let cell = map[my * side + mx];
            if !cell.active {
                continue;
            }
            let (x, y) = (ox + mx as i32, oy + my as i32);
            if !world.in_bounds(x, y) {
                continue;
            }
            // The C# `ShouldUseLava` slice: the lava-line test plus the tile
            // family scan. The port's lava line is not tracked yet, so the
            // lava arm rides off (deviation, ANALYSIS).
            let pressure = (cell.pressure - cell.resistance).max(0.0).sqrt();
            let gate =
                1.0 - ((y as f64 * 0.4).sin() * 0.7 + 1.2) * (0.2 + 0.5 / pressure.max(1e-12));
            let threshold = 0.35
                + if world.tile(x, y).is_some_and(|t| t.active()) {
                    0.0
                } else {
                    0.5
                };
            if gate.max(cell.pressure / 15.0) > threshold {
                let kind_now = world.tile(x, y).map(|t| t.kind).unwrap_or(0);
                if !tile_is_ore(kind_now) {
                    if let Some(t) = world.tile_mut(x, y) {
                        t.kind = TILE_GRANITE;
                        t.frame_x = -1;
                        t.frame_y = -1;
                        t.wall = WALL_GRANITE;
                    }
                }
            }
        }
    }
}

/// `GraniteBiome.CleanupTiles` (`GraniteBiome.cs:247-294`): solid granite with
/// fewer than 3 solid neighbours is cleared.
fn granite_cleanup(
    world: &mut World,
    ox: i32,
    oy: i32,
    map: &[MagmaCell],
    area: (usize, usize, usize, usize),
) {
    let side = 200usize;
    let (mx0, mx1, my0, my1) = area;
    let mut clear: Vec<(i32, i32)> = Vec::new();
    for my in my0..=my1 {
        for mx in mx0..=mx1 {
            if !map[my * side + mx].active {
                continue;
            }
            let (x, y) = (ox + mx as i32, oy + my as i32);
            if !world.in_bounds(x, y) {
                continue;
            }
            let mut solid = 0;
            for k in -1..=1i32 {
                for l in -1..=1i32 {
                    if world.tile(x + k, y + l).is_some_and(|t| t.active()) {
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
        if let Some(t) = world.tile_mut(x, y) {
            t.flags &= !FLAG_ACTIVE;
            t.wall = WALL_GRANITE;
        }
    }
}

/// The dungeon pass (`Dungeon` + `DungeonCrawler`): the surface entrance box
/// and the descending shaft, then 3 corridor levels of the dungeon brick
/// with the dungeon walls, the corridors carved inward. The brick colour
/// rides the world's dungeon roll (41 blue / 43 green / 44 pink).
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
    let top = surface[x0 as usize];
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
                    ..TileRunnerOpts::default()
                },
            );
        };
        let mut gems = |wb: &mut World, rep: &mut Rng, x: i32, y: i32, lo: i32| {
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
                    &TileRunnerOpts::default(),
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
                ..TileRunnerOpts::default()
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
                        ..TileRunnerOpts::default()
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
                &TileRunnerOpts::default(),
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
                    &TileRunnerOpts::default(),
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
                &TileRunnerOpts::default(),
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
                    &TileRunnerOpts::default(),
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
}
