//! Surface dressing: slope tiles, trees, small flora and desert cacti.

use std::collections::HashSet;

use super::noise::Rng;
use super::terrain::{Layers, SurfaceMap};
use super::{GenOptions, TILE_DIRT, TILE_GRASS, TILE_SAND, TILE_SNOW};
use crate::seeds::{MicroSeedFlags, SeedFlags};
use crate::tile::{FLAG_ACTIVE, FLAG_HALF_BRICK, LiquidKind, Tile};
use crate::tiledata::{self as td, SET_CAN_BE_CLEARED_DURING_GENERATION, TILE_SOLID};
use crate::tree;
use crate::world::World;

/// With height differences limited to one tile, every height change becomes a
/// slope tile on the higher side instead of a vertical step.
pub(crate) fn apply_slopes(world: &mut World, width: i32, surface: &[i32]) {
    // --- surface slopes -----------------------------------------------------
    for x in 0..width {
        let h = surface[x as usize];
        if !world.in_bounds(x, h) {
            continue;
        }
        let i = world.index(x, h);
        // Any solid top block gets a slope, not just grass: lake shores and desert
        // dirt stair-step the same way otherwise.
        if !world.tiles[i].active() {
            continue;
        }
        let right_lower = x + 1 < width && surface[(x + 1) as usize] == h + 1;
        let left_lower = x > 0 && surface[(x - 1) as usize] == h + 1;
        if right_lower {
            world.tiles[i].set_slope(1);
        } else if left_lower {
            world.tiles[i].set_slope(2);
        }
    }
}

/// Grows the forest/snow trees: a port of `WorldGen.GrowTree`'s frame tables.
///
/// The beach palm arm is the `AddTrees` beach-band roll (`WorldGen.cs:31372-31386`):
/// per column within `beachDistance` (380) of either edge, each row up to the
/// surface rolls `NextDouble() < 0.1 + NextDouble() * 0.35` with a liquid-0
/// gate, and a hit grows a palm on tile 323. The compact forest model keeps
/// its own density (the full `AddTrees` per-tile forest arms belong to the
/// faithful Trees pass — M5-STRUCT's follow-up).
pub(crate) fn plant_trees(world: &mut World, rng: &mut Rng, surface: &[i32], world_surface: i32) {
    let width = world.width;
    let mut last = -20;
    let mut x = 12;
    // The two beach densities, drawn once before the column loop
    // (`WorldGen.cs:31373-31374`).
    let (palm_left, palm_right) = (
        0.1 + rng.next_f64() * 0.35,
        0.1 + rng.next_f64() * 0.35,
    );
    while x < width - 12 {
        // Forest density, like vanilla: a tree every 6-14 columns with gaps.
        x += rng.range(5, 15);
        if x >= width - 12 {
            break;
        }
        if x - last < 5 {
            continue;
        }
        // The AddTrees beach palm arm (`WorldGen.cs:31372-31386`): the
        // per-column, per-row roll inside the beach bands.
        if x < super::ocean::ZONE_MARGIN || x >= width - super::ocean::ZONE_MARGIN {
            let prob = if x < super::ocean::ZONE_MARGIN {
                palm_left
            } else {
                palm_right
            };
            for j in 20..world_surface {
                let idx = world.index(x, j);
                if rng.next_f64() < prob && world.tiles[idx].liquid == 0 {
                    super::ocean::grow_palm_tree(world, x, j, 0, false, rng);
                }
            }
        }
        let top = surface[x as usize];
        let ground = *world.tile(x, top).unwrap_or(&Tile::default());
        if !(ground.active() && super::biomes::tree_soil(ground.kind))
            || ground.slope() != 0
        {
            continue;
        }
        // Require adjacent ground too (vanilla's IsTileTypeFitForTree checks).
        let flat = surface[(x - 1) as usize] - top <= 1 && surface[(x + 1) as usize] - top <= 1;
        if !flat {
            continue;
        }
        let height = rng.range(6, 17);
        if top - height < 12 {
            continue;
        }
        let mut clear = true;
        for dy in 1..=height {
            if world.tile(x, top - dy).is_some_and(|t| t.active()) {
                clear = false;
                break;
            }
        }
        if !clear {
            continue;
        }
        last = x;

        // Port of `WorldGen.GrowTree`'s frame table. `num6` picks the trunk piece
        // (0..4 and 8/9 are straight, 5/6/7 carry a left/right/both branch); the
        // per-row `num5` style selects one of the three frame-Y variants. The bottom
        // and top rows force num6 = 0, which is what gives the tree its solid base.
        let mut rows = vec![(0i16, 0i16); height as usize];
        let mut markers: Vec<(i32, i32, i16, i16)> = Vec::new();
        let mut left_last = false;
        let mut right_last = false;
        for (i, row) in rows.iter_mut().enumerate() {
            let mut num6 = if i == 0 || i + 1 == height as usize {
                0
            } else {
                rng.range(0, 10)
            };
            while ((num6 == 5 || num6 == 7) && left_last)
                || ((num6 == 6 || num6 == 7) && right_last)
            {
                num6 = rng.range(0, 10);
            }
            let style = rng.range(0, 3) as i16;
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
            let by = top - 1 - i as i32;
            if left_last {
                markers.push((x - 1, by, 44, 198 + rng.range(0, 3) as i16 * 22));
            }
            if right_last {
                markers.push((x + 1, by, 66, 198 + rng.range(0, 3) as i16 * 22));
            }
        }
        let top_y = top - height;
        rows[height as usize - 1] = (22, 198 + rng.range(0, 3) as i16 * 22);

        // `GrowTree`'s base forks: the lowest trunk tile gets a root frame and the
        // grass on either side may get a matching fork tile. The fork tiles are
        // ordinary trunk pieces (frameY 132..176), *not* branch markers (198+), and
        // the right fork uses frameX 22, the left one 44.
        let grass_at = |dx: i32| {
            world.tile(x + dx, top).is_some_and(|t| {
                t.active()
                    && matches!(t.kind, 2 | 23 | 60 | 109)
                    && t.slope() == 0
                    && !t.half_brick()
            })
        };
        let left_grass = grass_at(-1);
        let right_grass = grass_at(1);
        let mut base_style = rng.range(0, 3);
        if !left_grass {
            if base_style == 0 {
                base_style = 2;
            }
            if base_style == 1 {
                base_style = 3;
            }
        }
        if !right_grass {
            if base_style == 0 {
                base_style = 1;
            }
            if base_style == 2 {
                base_style = 3;
            }
        }
        if left_grass && !right_grass {
            base_style = 2;
        }
        if right_grass && !left_grass {
            base_style = 1;
        }
        if base_style < 3 {
            let fy = 132 + rng.range(0, 3) as i16 * 22;
            let fx = match base_style {
                0 => 88,
                1 => 0,
                _ => 66,
            };
            rows[0] = (fx, fy);
            let mut add_fork = |side: i32, fx: i16| {
                let bx = x + side;
                if bx < 1 || bx >= width - 1 {
                    return;
                }
                if world.tile(bx, top - 1).is_some_and(|t| t.active()) {
                    return;
                }
                let i = world.index(bx, top - 1);
                let fy = 132 + rng.range(0, 3) as i16 * 22;
                world.tiles[i] = Tile {
                    kind: tree::TREE,
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

        // Branch markers are separate transparent tiles that `DrawTrees` turns into
        // the canopy sprites; they must sit exactly beside the branch trunk piece.
        for (bx, by, fx, fy) in markers {
            if bx < 1 || bx >= width - 1 || by <= top_y || by >= top {
                continue;
            }
            if world.tile(bx, by).is_some_and(|t| t.active()) {
                continue;
            }
            let i = world.index(bx, by);
            world.tiles[i] = Tile {
                kind: tree::TREE,
                frame_x: fx,
                frame_y: fy,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        for (dy, &(fx, fy)) in rows.iter().enumerate() {
            let i = world.index(x, top - 1 - dy as i32);
            world.tiles[i] = Tile {
                kind: tree::TREE,
                frame_x: fx,
                frame_y: fy,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
    }
}

/// Scatters the short grass, flowers and mushrooms of tile 3 over flat forest grass.
pub(crate) fn plant_surface_flora(world: &mut World, rng: &mut Rng, width: i32, surface: &[i32]) {
    // --- surface plants -----------------------------------------------------
    // Tile 3 contains the short grass, flowers and mushrooms the original scatters
    // over forest grass. Frames are explicit, so the tile is flagged important.
    world.frame_important[3] = true;
    for x in 1..width - 1 {
        let top = surface[x as usize];
        let ground = *world.tile(x, top).unwrap_or(&Tile::default());
        if !ground.active() || ground.kind != TILE_GRASS || ground.slope() != 0 {
            continue;
        }
        if !rng.chance(4) {
            continue;
        }
        let y = top - 1;
        if y < 1 || world.tile(x, y).is_some_and(|t| t.active()) {
            continue;
        }
        // 0..8 are grass tufts; the rest are flowers and small mushrooms.
        let frame = if rng.chance(4) {
            rng.range(9, 30)
        } else {
            rng.range(0, 9)
        };
        let i = world.index(x, y);
        world.tiles[i] = Tile {
            kind: 3,
            frame_x: (frame * 18) as i16,
            frame_y: 0,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
    }
}

/// Plants cacti (tile 80) on flat desert sand inside the desert band.
///
/// Retired with M5-OCEAN phase 5: the faithful `Cactus, Palm Trees, & Coral`
/// pass (`WorldGen.cs:21310`) — its middle zone's `PlantCactus` covers the
/// desert's cacti with the vanilla density — replaced this compact scatter.
#[allow(dead_code)]
pub(crate) fn plant_cacti(world: &mut World, rng: &mut Rng, map: &SurfaceMap, width: i32) {
    // --- desert cacti -------------------------------------------------------
    // Tile 80 has its own 7x3 frame grid, so it is flagged important and framed by
    // hand: y=0 top, y=18 middle, y=36 base.
    let desert_enabled = map.desert_enabled;
    let desert_start = map.desert_start;
    let desert_end = map.desert_end;
    let surface = &map.heights;
    if desert_enabled {
        world.frame_important[80] = true;
        let mut x = desert_start + 6;
        while x < desert_end - 6 {
            x += rng.range(6, 18);
            if x < 4 || x >= width - 4 {
                continue;
            }
            let top = surface[x as usize];
            if top < 8 {
                continue;
            }
            let flat = surface[(x - 1) as usize] == top && surface[(x + 1) as usize] == top;
            let ground_ok = world
                .tile(x, top)
                .is_some_and(|t| t.active() && t.kind == TILE_SAND && t.slope() == 0);
            if !flat || !ground_ok {
                continue;
            }
            let h = rng.range(2, 5);
            if top - h < 2 {
                continue;
            }
            let mut clear = true;
            for dy in 1..=h {
                if world.tile(x, top - dy).is_some_and(|t| t.active()) {
                    clear = false;
                    break;
                }
            }
            if !clear {
                continue;
            }
            for dy in 1..=h {
                let frame_y = if dy == h {
                    0
                } else if dy == 1 {
                    36
                } else {
                    18
                };
                let i = world.index(x, top - dy);
                world.tiles[i] = Tile {
                    kind: 80,
                    frame_x: 0,
                    frame_y,
                    flags: FLAG_ACTIVE,
                    ..Default::default()
                };
            }
        }
    }
}

// --- Grass Wall (W4-d106 R1) --------------------------------------------------

/// Wall ids the pass reads/writes (`ID/WallID.cs`): DirtUnsafe 2 (`:109`),
/// MudUnsafe 15 (`:135`), Cave6Unsafe 59 (`:223`), SpiderUnsafe 62 (`:229`),
/// GrassUnsafe 63 (`:231`), FlowerUnsafe 65 (`:235`).
const WALL_MUD_UNSAFE: u16 = 15;
const WALL_CAVE6_UNSAFE: u16 = 59;
const WALL_SPIDER_UNSAFE: u16 = 62;
const WALL_GRASS_UNSAFE: u16 = 63;
const WALL_FLOWER_UNSAFE: u16 = 65;

/// `WorldGen.maxTileCount` (`WorldGen.cs:86921` static ctor / `:19593` —
/// the pass re-sets the same 3500) and `maxWallOut2` (`:86922`, never
/// rewritten — 5000 for the whole generation).
const MAX_TILE_COUNT: i32 = 3500;
const MAX_WALL_OUT: i32 = 5000;

/// `WorldGen.InWorld(x, y, fluff)` (`WorldGen.cs:9688-9695`).
fn in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// `WallID.Sets.CannotBeReplacedByWallSpread` (`WallID.cs:98` —
/// Wood 4, SnowWallUnsafe 40, EbonstoneUnsafe 3, CrimstoneUnsafe 83,
/// LihzahrdBrickUnsafe 87, LivingWoodUnsafe 244, SandstoneBrick 34).
fn cannot_be_replaced_by_wall_spread(wall: u16) -> bool {
    matches!(wall, 4 | 40 | 3 | 83 | 87 | 244 | 34)
}

/// `Spread.Wall2` (`WorldGen.cs:3416-3524`): the wave-front wall flood.
/// The two-list + visited-set structure is verbatim (`list` = this wave,
/// `list2` = the next; every popped point joins `hashSet` before its tile
/// tests). A non-solid cell converts and fans out 4-way — plus the
/// diagonals and the ±2 columns when `WallSpreadStopsAtAir[wallType]`
/// (`WallID.cs:99` — the set is {63, 62}, true for both walls this pass
/// spreads) — under the 5000-conversion budget; the budget hit skips the
/// write but still fans nothing (the C# `continue`s before the write).
/// A solid active cell takes the wall without fanning out (`:3518-3521`).
/// RNG-free.
fn spread_wall(world: &mut World, x: i32, y: i32, wall_type: u16) {
    // `!InWorld(x, y)` (`:3418`).
    if !in_world(world, x, y, 0) {
        return;
    }
    let stops_at_air = matches!(wall_type, WALL_GRASS_UNSAFE | WALL_SPIDER_UNSAFE);
    let mut wave: Vec<(i32, i32)> = vec![(x, y)];
    let mut visited: HashSet<(i32, i32)> = HashSet::new();
    let mut conversions = 0;
    while !wave.is_empty() {
        let mut next: Vec<(i32, i32)> = Vec::new();
        for (px, py) in wave.drain(..) {
            // `!InWorld(item.X, item.Y, 1)` (`:3437`).
            if !in_world(world, px, py, 1) {
                continue;
            }
            visited.insert((px, py));
            let t = world.tiles[world.index(px, py)];
            // `tile.wall == num || CannotBeReplacedByWallSpread[wall]` (`:3445`).
            if t.wall == wall_type || cannot_be_replaced_by_wall_spread(t.wall) {
                continue;
            }
            if !super::biome_detail::solid_tile(world, px, py) {
                // `WallSpreadStopsAtAir[num] && tile.wall == 0` (`:3451-3456`).
                if stops_at_air && t.wall == 0 {
                    continue;
                }
                conversions += 1;
                if conversions >= MAX_WALL_OUT {
                    continue;
                }
                world.tiles[world.index(px, py)].wall = wall_type;
                // The 4-way fan (`:3464-3483`).
                let mut fan = [
                    (px - 1, py),
                    (px + 1, py),
                    (px, py - 1),
                    (px, py + 1),
                ]
                .to_vec();
                if stops_at_air {
                    // The diagonals + the ±2 columns (`:3484-3516`).
                    fan.extend([
                        (px - 1, py - 1),
                        (px + 1, py - 1),
                        (px - 1, py + 1),
                        (px + 1, py + 1),
                        (px - 2, py),
                        (px + 2, py),
                    ]);
                }
                for p in fan {
                    if !visited.contains(&p) {
                        next.push(p);
                    }
                }
            } else if t.active() {
                // The solid-active arm (`:3518-3521`): the wall only.
                world.tiles[world.index(px, py)].wall = wall_type;
            }
        }
        wave = next;
    }
}

/// `WorldGen.countDirtTiles` + `nextDirtCount` (`WorldGen.cs:9637-9681`):
/// the connected dirt-wall census the spread gate reads. The C# recurses
/// depth-first in a fixed 10-neighbour order; the port walks an explicit
/// LIFO stack with the neighbours pushed in reverse order so the pop order
/// is the C#'s call order exactly (the count saturates at `max`, so only
/// the traversal *set* matters, but the order is kept verbatim anyway).
/// The border touch (`x <= 1 || ...`, `:9651`) and the snow/ice tiles 147/
/// 161 or the structure walls 244/83/3/187/216 (`:9657-9664`) saturate the
/// count at `max`; only non-solid cells with wall 2/59 count (`:9665`).
/// RNG-free.
fn count_dirt_tiles(world: &World, x: i32, y: i32, max: i32) -> i32 {
    let mut count = 0;
    let mut counted: HashSet<(i32, i32)> = HashSet::new();
    let mut stack = vec![(x, y)];
    while let Some((cx, cy)) = stack.pop() {
        // `numTileCount >= maxTileCount` (`:9647`).
        if count >= max {
            return count;
        }
        if cx <= 1 || cx >= world.width - 1 || cy <= 1 || cy >= world.height - 1 {
            count = max;
            continue;
        }
        if counted.contains(&(cx, cy)) {
            continue;
        }
        let t = world.tiles[world.index(cx, cy)];
        if t.active() && matches!(t.kind, super::TILE_SNOW | super::TILE_ICE) {
            count = max;
            continue;
        }
        if matches!(t.wall, 244 | 83 | 3 | 187 | 216) {
            count = max;
            continue;
        }
        if !super::biome_detail::solid_tile(world, cx, cy)
            && matches!(t.wall, super::WALL_DIRT | WALL_CAVE6_UNSAFE)
        {
            counted.insert((cx, cy));
            count += 1;
            // The C#'s call order (`:9669-9678`) — pushed reversed for the
            // LIFO pop.
            for p in [
                (cx + 2, cy),
                (cx - 2, cy),
                (cx + 1, cy + 1),
                (cx + 1, cy - 1),
                (cx - 1, cy + 1),
                (cx - 1, cy - 1),
                (cx, cy + 1),
                (cx, cy - 1),
                (cx + 1, cy),
                (cx - 1, cy),
            ] {
                stack.push(p);
            }
        }
    }
    count
}

/// `WorldGen.SpreadGrass(i, j)` — the default-args call shape (`dirt = 0`,
/// `grass = 2`, `repeat = true`, `WorldGen.cs:75146-75257`), the slice this
/// pass runs. The generation-time surface rule (`:75162`: `j >=
/// worldSurface && dirt == 0` refuses) and the 3×3 enclosure scan with the
/// lava exception (`:75186-75201`) are verbatim; the `CanBeClearedDuring-
/// Generation` gate (`:75202`, skipDualDungeonCheck) reads the tiledata
/// set; the flower-27 exclusions guard grasses 23/661/199/662/109, not
/// grass 2. The write (`:75220-75225`) kills an invalid tree above
/// (`:54802-54902` — the shared growth helper), then converts the kind
/// only (`SquareTileFrame` is the post-pipeline framing seam, the port's
/// documented deferral). The recursion (`:75231-75252`) carries the
/// `grassSpread < 1000` cap. RNG-free.
fn spread_dirt_grass(world: &mut World, i: i32, j: i32, world_surface: f64, grass_spread: &mut i32) {
    // `!InWorld(i, j, 10) || !active || type != dirt` (`:75150`).
    if !in_world(world, i, j, 10) {
        return;
    }
    if !world.tile(i, j).is_some_and(|t| t.active() && t.kind == TILE_DIRT) {
        return;
    }
    // The generation-time else-if (`:75162-75165`): grass 2 is not 199/23,
    // `isGeneratingOrLoadingWorld` is true and `growGrassUnderground` false
    // — only the surface rule can fire for dirt 0.
    if j as f64 >= world_surface {
        return;
    }
    // The clamped 3×3 scan (`:75166-75201`).
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
    if flag
        || !world
            .tile(i, j)
            .is_some_and(|t| SET_CAN_BE_CLEARED_DURING_GENERATION.has(t.kind))
    {
        return;
    }
    // `flag2` (`:75206`) is dead for dirt 0 (`Conversion.Grass` holds
    // 2/23/199/109/477/492, `TileID.cs:86`) — the plain write runs.
    crate::growth::kill_invalid_tree_above(world, i, j, TILE_GRASS);
    if let Some(t) = world.tile_mut(i, j) {
        t.kind = TILE_GRASS;
    }
    // The 3×3 recursion with the cap (`:75231-75252`).
    for m in num2..num3 {
        for n in num4..num5 {
            if world.tile(m, n).is_some_and(|t| t.active() && t.kind == TILE_DIRT)
                && *grass_spread < 1000
            {
                *grass_spread += 1;
                spread_dirt_grass(world, m, n, world_surface, grass_spread);
                *grass_spread -= 1;
            }
        }
    }
}

/// The `Grass Wall` pass (`GenPassNameID.SurfaceDirtWallsToGrassWalls`,
/// `WorldGen.cs:19589-19684`): the surface dirt walls turn to grass walls
/// (63, the Don't Starve spider-wall 62 arm) behind exposed grass tiles,
/// then the flower-wall speckle (63 → 65 at 1/10) and the dirt-to-grass
/// fringe sweep. `Skyblock.denyAllGeneration` (`:19591` → `:3131` — the
/// skyblock flag) skips the whole pass.
///
/// Draws, in C# order on the pass's own stream: one `Next(4)` per (i, j)
/// in `[50, maxTilesX − 50) × [0, worldSurface − 10)` (`:19595-19599`), the
/// Don't Starve `Next(maxValue)` only when a spread actually fires
/// (`:19641`, short-circuit: no draw when the seed is off), then one
/// `Next(10)` per (i, j) with wall 63 in `[5, maxTilesX − 5) × [10,
/// worldSurface − 1)` (`:19654-19658`, short-circuit: no draw on other
/// walls). `grassSpread` resets to 0 per conversion site — the C# static
/// is 0 here because no earlier executed pass recursed `SpreadGrass`
/// (the jungle/evil sweeps zero it per site, `:12484`/`:14137`).
pub(crate) fn grass_wall(world: &mut World, world_surface: f64, rng: &mut Rng, opts: &GenOptions) {
    // `!Skyblock.denyAllGeneration` (`:19591`).
    if opts.flags.contains(SeedFlags::SKYBLOCK) {
        return;
    }
    // `maxTileCount = 3500` (`:19593`) — the census cap, [`MAX_TILE_COUNT`].
    let dont_starve = opts.flags.contains(SeedFlags::DONT_STARVE);
    // `SecretSeed.Variations.noSpiderCavesActuallyNoSpiderCaves`
    // (`WorldGen.cs:269-279`): noSpiderCaves && activeSecretSeedCount < 4.
    let no_spider_actually = opts.micro_flags.contains(MicroSeedFlags::NO_SPIDER_CAVES)
        && crate::seeds::active_secret_seed_count(opts.micro_flags, opts.flags) < 4;
    // The wall-spread sweep (`:19595-19653`) — i-outer, j-inner, the
    // `Next(4)` gate first (`:19599`). The row count is the C# loop bound
    // `(double)j < worldSurface - 10.0` evaluated once (the house
    // convention, `biome_detail::pass_muds_walls_in_jungle`).
    let spread_rows = {
        let mut rows = 0i32;
        let mut j = 0i32;
        while (j as f64) < world_surface - 10.0 {
            rows += 1;
            j += 1;
        }
        rows
    };
    for i in 50..world.width - 50 {
        for j in 0..spread_rows {
            if rng.range(0, 4) != 0 {
                continue;
            }
            let mut flag = false;
            let mut num = -1;
            let mut num2 = -1;
            let seed_cell = world
                .tile(i, j)
                .is_some_and(|t| t.active() && t.kind == TILE_GRASS && matches!(t.wall, super::WALL_DIRT | WALL_GRASS_UNSAFE));
            if seed_cell {
                // The 3×3 exposure scan (`:19606-19615`): a wall-0 non-solid
                // neighbour exposes the pocket. Out-of-world reads would
                // throw in C#; unreachable here (the seed cell is terrain,
                // far below row 0), so the port reads them as no-expose.
                for k in (i - 1)..=(i + 1) {
                    for l in (j - 1)..=(j + 1) {
                        let exposes = match world.tile(k, l) {
                            Some(t) => t.wall == 0 && !super::biome_detail::solid_tile(world, k, l),
                            None => false,
                        };
                        if exposes {
                            flag = true;
                        }
                    }
                }
                if flag {
                    // The 3×3 target pick (`:19618-19628`): the LAST wall-2/
                    // 15 non-solid cell in scan order wins.
                    for m in (i - 1)..=(i + 1) {
                        for n in (j - 1)..=(j + 1) {
                            let candidate = match world.tile(m, n) {
                                Some(t) => {
                                    matches!(t.wall, super::WALL_DIRT | WALL_MUD_UNSAFE)
                                        && !super::biome_detail::solid_tile(world, m, n)
                                }
                                None => false,
                            };
                            if candidate {
                                num = m;
                                num2 = n;
                            }
                        }
                    }
                }
            }
            // The spread gate (`:19631-19650`).
            if flag && num > -1 && num2 > -1 && count_dirt_tiles(world, num, num2, MAX_TILE_COUNT) < MAX_TILE_COUNT {
                // `maxValue = 3`, for-the-worthy 10 (`:19635-19639`).
                let max_value = if opts.flags.contains(SeedFlags::FOR_THE_WORTHY) { 10 } else { 3 };
                let mut wall_type = WALL_GRASS_UNSAFE;
                // The Don't Starve spider-wall arm (`:19641-19644`) — the
                // `Next(maxValue)` draw short-circuits behind dontStarve and
                // ahead of the noSpiderCaves check.
                if dont_starve && rng.range(0, max_value) != 0 && !no_spider_actually {
                    wall_type = WALL_SPIDER_UNSAFE;
                }
                spread_wall(world, num, num2, wall_type);
            }
        }
    }
    // The flower speckle + grass fringe sweep (`:19654-19682`) — rows
    // `[10, worldSurface − 1)`.
    let fringe_rows = {
        let mut rows = 0i32;
        let mut j = 0i32;
        while (j as f64) < world_surface - 1.0 {
            rows += 1;
            j += 1;
        }
        rows
    };
    for num3 in 5..world.width - 5 {
        for num4 in 10..fringe_rows {
            // The wall-63 speckle (`:19658-19661`) — the `Next(10)` draw
            // short-circuits behind the wall test.
            if world.tile(num3, num4).is_some_and(|t| t.wall == WALL_GRASS_UNSAFE)
                && rng.range(0, 10) == 0
            {
                world.tiles[world.index(num3, num4)].wall = WALL_FLOWER_UNSAFE;
            }
            // The dirt fringe (`:19662-19680`): any wall-63/65 cell in the
            // 3×3 converts the dirt tile via SpreadGrass. The C# `break`
            // exits the inner loop only, but `flag2` is only ever assigned
            // true (`:19671`) and the scan draws nothing — breaking both
            // loops on the first hit is outcome-identical.
            if world.tile(num3, num4).is_some_and(|t| t.active() && t.kind == TILE_DIRT) {
                let mut flag2 = false;
                'scan: for num5 in (num3 - 1)..=(num3 + 1) {
                    for num6 in (num4 - 1)..=(num4 + 1) {
                        if world
                            .tile(num5, num6)
                            .is_some_and(|t| matches!(t.wall, WALL_GRASS_UNSAFE | WALL_FLOWER_UNSAFE))
                        {
                            flag2 = true;
                            break 'scan;
                        }
                    }
                }
                if flag2 {
                    // `grassSpread` is the C# static — 0 at this site (no
                    // earlier executed pass left it non-zero).
                    let mut grass_spread = 0;
                    spread_dirt_grass(world, num3, num4, world_surface, &mut grass_spread);
                }
            }
        }
    }
}

#[cfg(test)]
mod grass_wall_tests {
    use super::*;

    /// A synthetic surface world: air above row 50, a grass row with wall
    /// 2 at row 50 across `[50, width − 50)`, dirt with wall 2 below — plus
    /// an exposed wall-2 air pocket at row 49 (the overhang the C# pass
    /// spreads into: the seed cell needs a non-solid wall-2/15 target).
    fn wall_world(width: i32, height: i32) -> World {
        let mut world = World::new("gw".into(), width, height);
        for x in 50..width - 50 {
            for y in 50..height - 10 {
                let i = world.index(x, y);
                let mut t = Tile::default();
                t.kind = if y == 50 { TILE_GRASS } else { TILE_DIRT };
                t.wall = super::super::WALL_DIRT;
                t.flags |= FLAG_ACTIVE;
                world.tiles[i] = t;
            }
        }
        // The pocket: inactive cells with wall 2 one row above the grass,
        // every third column left wall-0 air so the seed cells' 3×3
        // exposure scan (`:19610`, wall == 0 && !SolidTile) always finds an
        // exposer while the target scan still finds wall-2 candidates.
        for x in 55..width - 55 {
            if x % 3 != 0 {
                world.tiles[world.index(x, 49)].wall = super::super::WALL_DIRT;
            }
        }
        world
    }

    /// W4-d106 R1: `Spread.Wall2` converts the exposed pocket and takes the
    /// wall behind solid active cells (`WorldGen.cs:3449-3521`), while
    /// wall-0 air stops the spread (`WallSpreadStopsAtAir`, `:3451-3456`)
    /// and the protected walls (`WallID.cs:98`) are never replaced.
    #[test]
    fn r1_wall2_converts_the_pocket_and_stops_at_air_and_protected_walls() {
        let mut w = World::new("gw".into(), 60, 60);
        // A dirt-wall pocket: (30,30) and (31,30) carry wall 2, air.
        for (x, y) in [(30, 30), (31, 30), (30, 31)] {
            w.tiles[w.index(x, y)].wall = super::super::WALL_DIRT;
        }
        // A solid grass cell beside the pocket and a protected wood wall.
        let i = w.index(29, 30);
        w.tiles[i].kind = TILE_GRASS;
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.tiles[w.index(32, 30)].wall = 4; // Wood — CannotBeReplacedByWallSpread.
        spread_wall(&mut w, 30, 30, WALL_GRASS_UNSAFE);
        assert_eq!(w.tile(30, 30).unwrap().wall, WALL_GRASS_UNSAFE);
        assert_eq!(w.tile(31, 30).unwrap().wall, WALL_GRASS_UNSAFE);
        assert_eq!(w.tile(30, 31).unwrap().wall, WALL_GRASS_UNSAFE);
        // The solid-active arm (`:3518-3521`): the wall lands behind it.
        assert_eq!(w.tile(29, 30).unwrap().wall, WALL_GRASS_UNSAFE);
        // The protected wall survives; wall-0 air never converts.
        assert_eq!(w.tile(32, 30).unwrap().wall, 4);
        assert_eq!(w.tile(33, 30).unwrap().wall, 0);
    }

    /// W4-d106 R1: `countDirtTiles` counts the connected wall-2/59 pocket
    /// and saturates at the structure walls (`WorldGen.cs:9661-9664`) and
    /// the snow/ice tiles (`:9657-9660`).
    #[test]
    fn r1_count_dirt_tiles_counts_and_saturates() {
        let mut w = World::new("gw".into(), 60, 60);
        for (x, y) in [(30, 30), (31, 30), (30, 31)] {
            w.tiles[w.index(x, y)].wall = super::super::WALL_DIRT;
        }
        assert_eq!(count_dirt_tiles(&w, 30, 30, MAX_TILE_COUNT), 3);
        // A crimstone wall (83) in reach saturates the census.
        w.tiles[w.index(32, 30)].wall = 83;
        assert_eq!(count_dirt_tiles(&w, 30, 30, MAX_TILE_COUNT), MAX_TILE_COUNT);
        // An ice tile (161) saturates too.
        w.tiles[w.index(32, 30)].wall = 0;
        let i = w.index(32, 30);
        w.tiles[i].kind = super::super::TILE_ICE;
        w.tiles[i].flags |= FLAG_ACTIVE;
        assert_eq!(count_dirt_tiles(&w, 30, 30, MAX_TILE_COUNT), MAX_TILE_COUNT);
    }

    /// W4-d106 R1: the `SpreadGrass(0, 2)` slice converts an exposed dirt
    /// cell above the surface line and refuses below it (`:75162`) or when
    /// enclosed (`:75186-75201`).
    #[test]
    fn r1_spread_dirt_grass_converts_exposed_dirt_above_the_surface() {
        let mut w = World::new("gw".into(), 60, 60);
        let i = w.index(30, 30);
        w.tiles[i].kind = TILE_DIRT;
        w.tiles[i].flags |= FLAG_ACTIVE;
        let mut spread = 0;
        spread_dirt_grass(&mut w, 30, 30, 100.0, &mut spread);
        assert_eq!(w.tile(30, 30).unwrap().kind, TILE_GRASS);
        // Below the surface line the generation-time rule refuses.
        let j = w.index(30, 40);
        w.tiles[j].kind = TILE_DIRT;
        w.tiles[j].flags |= FLAG_ACTIVE;
        spread_dirt_grass(&mut w, 30, 40, 35.0, &mut spread);
        assert_eq!(w.tile(30, 40).unwrap().kind, TILE_DIRT);
    }

    /// W4-d106 R1: the pass is deterministic (a == b on the same seed) and
    /// the skyblock gate (`:19591`) skips it tile-for-tile.
    #[test]
    fn r1_the_pass_is_deterministic_and_skyblock_skips_it() {
        let opts = GenOptions {
            width: 512,
            height: 512,
            seed: 42,
            ..Default::default()
        };
        let mut a = wall_world(200, 120);
        let mut b = wall_world(200, 120);
        grass_wall(&mut a, 100.0, &mut Rng(42), &opts);
        grass_wall(&mut b, 100.0, &mut Rng(42), &opts);
        assert_eq!(a.tiles, b.tiles, "a == b");
        // The classic pass converts at least one pocket wall to 63 (the
        // 100-column grass band rolls ~100 Next(4) chances at row 50).
        assert!(
            a.tiles.iter().any(|t| t.wall == WALL_GRASS_UNSAFE),
            "the classic pass must spread grass walls"
        );
        let mut sky = wall_world(200, 120);
        let before = sky.tiles.clone();
        let sky_opts = GenOptions {
            flags: SeedFlags::from(SeedFlags::SKYBLOCK),
            ..opts.clone()
        };
        grass_wall(&mut sky, 100.0, &mut Rng(42), &sky_opts);
        assert_eq!(sky.tiles, before, "skyblock skips the pass");
    }
}

// --- W4-g38 C1: the Place Fallen Log pass ------------------------------------

/// `WorldGen.SolidTile2(int, int)` (`WorldGen.cs:70492-70509`) — the
/// `Place3x2` floor test: active + `tileSolid` + (platform half-brick/top-
/// slope exemption OR slope 0) + not half-brick + not inactive. Null reads
/// solid; out-of-bounds reads false (the C# catch).
fn solid_tile2(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && TILE_SOLID.has(t.kind)
        && ((td::SET_PLATFORMS.has(t.kind) && (t.half_brick() || t.top_slope()))
            || t.slope() == 0)
        && !t.half_brick()
        && !t.inactive()
}

/// `WorldGen.Place3x2` (`WorldGen.cs:52066-52196`), the type-488 path the
/// fallen-log placement runs — no RNG: the 5-tile bounds guard, the 3×2
/// clear scan (`y-1..y+1` — the `14/25` picnic one-row arm never fires for
/// 488), the campfire liquid veto (dead for 488), the default-case floor
/// (`SolidTile2` under every column; the 186/187 piles veto and the 26
/// boulder veto are dead kinds here), then the row-major frame write
/// (`frameX = 54*style + dx*18`, `frameY = dy*18`). The 88 chest ledger and
/// the 582/619 solid-top arm are other kinds' paths.
fn place_3x2(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    if x < 5 || x > world.width - 5 || y < 5 || y > world.height - 5 {
        return;
    }
    let mut ok = true;
    for i in (x - 1)..(x + 2) {
        for j in (y - 1)..(y + 1) {
            if let Some(t) = world.tile(i, j) {
                if t.active() {
                    ok = false;
                }
                // `TileID.Sets.Campfires[488]` is false — the liquid veto
                // never fires for this kind (`:52096-52099`).
            }
        }
        if !solid_tile2(world, i, y + 1) {
            ok = false;
        }
    }
    if !ok {
        return;
    }
    let num2 = (54 * style) as i16;
    let write = |world: &mut World, x: i32, y: i32, frame_x: i16, frame_y: i16| {
        if let Some(t) = world.tile_mut(x, y) {
            t.flags |= FLAG_ACTIVE;
            t.kind = kind;
            t.frame_x = frame_x;
            t.frame_y = frame_y;
        }
    };
    // The 3×2 write (`:52171-52194`), C# row-major order.
    write(world, x - 1, y - 1, num2, 0);
    write(world, x, y - 1, num2 + 18, 0);
    write(world, x + 1, y - 1, num2 + 36, 0);
    write(world, x - 1, y, num2, 18);
    write(world, x, y, num2 + 18, 18);
    write(world, x + 1, y, num2 + 36, 18);
}

/// `WorldGen.PlaceTile(i, j, 488)` (`WorldGen.cs:59342-60295`), the slice
/// the fallen-log pass runs: the gen-time existing-log veto (`:59345`),
/// the empty/non-solid entry (`:59373` — `Main.tileSolid[488]` is false,
/// so the entry always passes), the inactive-cell `Clear(Tile|TilePaint|
/// Slope)` (`:59473-59476`), the `Place3x2` dispatch (`:59888-59891`), and
/// the post-dispatch success flag (`:60255-60262`). The `SquareTileFrame`
/// re-frame rides the port's post-pipeline seam (the d106 convention).
/// Returns the C# `result` — whether the centre cell ended active.
fn place_tile_fallen_log(world: &mut World, x: i32, y: i32) -> bool {
    // `:59345`: during generation an existing fallen log blocks placement.
    if world
        .tile(x, y)
        .is_some_and(|t| t.active() && t.kind == 488)
    {
        return false;
    }
    let Some(t) = world.tile(x, y).copied() else {
        return false;
    };
    // `:59373` entry: `Collision.EmptyTile || !tileSolid[488]` — 488 is not
    // in `Main.tileSolid`, so the second disjunct always admits. The
    // torch/flower/moss/Abigail special arms are other kinds' paths.
    // `:59473-59476`: an inactive cell clears tile + paint/coating + slope
    // (wall, liquid and wiring survive).
    if !t.active() {
        if let Some(c) = world.tile_mut(x, y) {
            c.kind = 0;
            c.flags &= !FLAG_ACTIVE;
            c.frame_x = 0;
            c.frame_y = 0;
            c.set_paint(0);
            c.flags &= !crate::tile::FLAG_FULLBRIGHT;
            c.set_slope(0);
            c.flags &= !FLAG_HALF_BRICK;
        }
        world.set_invisible_block(x, y, false);
    }
    place_3x2(world, x, y, 488, 0);
    // `:60255-60262`: `result = tile.active()` — `TruncatesWalls[488]` is
    // false, the sound/dust arms are mute-irrelevant at gen time.
    world.tile(x, y).is_some_and(|t| t.active())
}

/// `Main.tileDungeon` (`Main.cs:7992-7997`) — the six dungeon brick kinds
/// the fallen-log neighbourhood veto reads.
const TILE_DUNGEON: [u16; 6] = [41, 43, 44, 677, 678, 679];

/// The `Place Fallen Log` pass (`GenPassNameID.FallenLogsAndWaterFeatures`,
/// `WorldGen.cs:18513-18656`): `maxTilesX / 2100` (+ `Next(-1, 2)`, remix
/// `Next(0, 2)`) fallen logs (tile 488, a 3×2 object), each found by a
/// 30000-attempt surface scan with a 5000-attempt desperation tail.
///
/// Draws, in C# order on the pass's own stream: one count roll, then per
/// attempt the (x, y) pair, the centre-band x re-rolls while attempts
/// remain above half the desperation threshold, and on a placed log one
/// `Next(2)` for the `GenVars.logX/logY` flower anchor (`:18643-18647`,
/// consumed by the Flowers pass `:20468`/`:20532`).
///
/// Deferred seed arms (documented, flag-gated — a classic world never
/// enters them): the `extraLiquid && !roundLandmasses` water-features head
/// (`:18517-18521`, `SecretSeed.DoExtraLiquidAddBubbleBlocks` `:1648-1807`
/// — the QuickWater pre-settle + bubble-square speckle belongs to the
/// settle-liquids/extra-liquid lane); the remix surface-layer y band
/// (`:18543-18546`/`:18562-18567`, `GenVars.remixSurfaceLayerLow/High` —
/// `TerrainPass.cs:239-240`'s rock-low..height-350 band, not yet a port
/// GenVars field). The skyblock budgets (`:18533-18537`) and the
/// notTheBees/frozen/desert soil kinds (`:18637`) are ported.
pub(crate) fn place_fallen_log(
    world: &mut World,
    layers: &Layers,
    rng: &mut Rng,
    opts: &GenOptions,
    log_anchor: &mut (i32, i32),
) {
    // `:18515`: !Skyblock.denySomeGeneration. matrix-88 §3.5: the gate
    // rides f24 A1's canonical table (the g38 C1 local copy dropped).
    if super::skyblock::deny_some_generation(opts.flags, opts.micro_flags) {
        return;
    }
    // The extraLiquid water-features head (`:18517-18521`) — deferred (see
    // the doc comment); no draws on the classic path.
    // `!SecretSeed.noSurface.Enabled` (`:18522`).
    if opts.micro_flags.contains(MicroSeedFlags::NO_SURFACE) {
        return;
    }
    let remix = opts.flags.contains(SeedFlags::REMIX);
    let skyblock = opts.flags.contains(SeedFlags::SKYBLOCK);
    let bees = opts.flags.contains(SeedFlags::NOT_THE_BEES);
    let frozen = opts.micro_flags.contains(MicroSeedFlags::WORLD_IS_FROZEN);
    let desert_surface = opts.micro_flags.contains(MicroSeedFlags::SURFACE_IS_DESERT);
    // `num = maxTilesX / 2100` then the count roll (`:18525-18526`).
    let mut count = world.width / 2100;
    count += if remix { rng.range(0, 2) } else { rng.range(-1, 2) };
    // The x band must be non-empty (the sub-vanilla 512 test world
    // collapses it — the C# `Next` would throw; the port skips the whole
    // loop after the count roll, drawing nothing more, like the
    // mining-explosives width guard). `num2 = beachDistance + 20`
    // (`:18530`, `:86914`'s 380).
    let margin = super::ocean::ZONE_MARGIN + 20;
    if margin >= world.width - margin {
        return;
    }
    let world_surface = layers.world_surface;
    for _ in 0..count {
        let (mut attempts, mut desperation) = (30000i32, 5000i32);
        if skyblock {
            attempts /= 10;
            desperation /= 10;
        }
        while attempts > 0 {
            attempts -= 1;
            let mut x = rng.range(margin, world.width - margin);
            let mut y = rng.range(10, world_surface as i32);
            if remix {
                // Deferred remix band (see the doc comment): the port's
                // nearest published anchors — rock_low..height-350
                // (`TerrainPass.cs:239-240`). Drawn in the C#'s spot.
                y = rng.range(layers.rock_low as i32, world.height - 350);
            }
            let desperate = attempts < desperation;
            if attempts > desperation / 2 {
                // The centre-band exclusion re-roll (`:18552-18558`).
                while (x as f64) > (world.width as f64) * 0.4
                    && (x as f64) < (world.width as f64) * 0.6
                {
                    x = rng.range(margin, world.width - margin);
                }
            }
            let start_clear = world
                .tile(x, y)
                .is_some_and(|t| !t.active() && t.wall == 0);
            if !start_clear {
                continue;
            }
            // The fall-to-ground scan (`:18562-18573`): the classic arm
            // drops while air + wall 0 and `j <= worldSurface`.
            let mut ok = true;
            if remix {
                let high = world.height - 350;
                while world
                    .tile(x, y)
                    .is_some_and(|t| !t.active() && t.wall == 0)
                    && y <= high
                {
                    y += 1;
                }
            } else {
                while world
                    .tile(x, y)
                    .is_some_and(|t| !t.active() && t.wall == 0)
                    && (y as f64) <= world_surface
                {
                    y += 1;
                }
                if (y as f64) > world_surface - 10.0 {
                    ok = false;
                }
            }
            if ok && !desperate {
                // The 100×100 neighbourhood veto (`:18578-18616`): clouds
                // 189, sand 53, dungeon bricks, crimson and corrupt kinds
                // disqualify the site. The scan bounds mirror the C#
                // (>10 / <max-10); reads outside the world are skipped.
                'hood: for k in (x - 50)..(x + 50) {
                    if k <= 10 || k >= world.width - 10 {
                        continue;
                    }
                    for l in (y - 50)..(y + 50) {
                        if l <= 10 || l >= world.height - 10 {
                            continue;
                        }
                        let Some(t) = world.tile(k, l) else { continue };
                        let kind = t.kind;
                        if kind == 189
                            || kind == 53
                            || TILE_DUNGEON.contains(&kind)
                            || td::SET_CRIMSON.has(kind)
                            || td::SET_CORRUPT.has(kind)
                        {
                            ok = false;
                            break 'hood;
                        }
                    }
                }
                if ok {
                    // The 20×9 headroom veto (`:18617-18635`): no solid
                    // active tile and no wall in the columns above the
                    // seat. The C# reads unguarded; the port's bounds
                    // check only skips cells the scan can never reach on
                    // a real world (x ≥ 400, y ≥ 10).
                    for m in (x - 10)..(x + 10) {
                        for n in (y - 10)..(y - 1) {
                            if let Some(t) = world.tile(m, n) {
                                if t.active() && TILE_SOLID.has(t.kind) {
                                    ok = false;
                                }
                                if t.wall != 0 {
                                    ok = false;
                                }
                            }
                        }
                    }
                }
            }
            if !ok {
                continue;
            }
            // The seat test (`:18637`): liquid 0 (or the desperation tail
            // below a fifth), dirt 2 under the centre and both neighbours
            // (bees mud 60 / frozen snow 147 / desert sand 53 arms).
            let soil = |dx: i32| {
                world.tile(x + dx, y).is_some_and(|t| {
                    t.kind == 2
                        || (bees && t.kind == 60)
                        || (frozen && t.kind == 147)
                        || (desert_surface && t.kind == 53)
                })
            };
            let liquid_ok = world.tile(x, y - 1).is_some_and(|t| t.liquid == 0)
                || attempts < desperation / 5;
            if !(liquid_ok && soil(0) && soil(-1) && soil(1)) {
                continue;
            }
            y -= 1;
            place_tile_fallen_log(world, x, y);
            if world
                .tile(x, y)
                .is_some_and(|t| t.active() && t.kind == 488)
            {
                // The Flowers-pass anchor roll (`:18643-18647`).
                if rng.range(0, 2) == 0 {
                    *log_anchor = (x, y);
                }
                attempts = -1;
            }
        }
    }
}

// --- W4-g38 C6: the Sunflowers pass ------------------------------------------
//
// `GenPassNameID.SunflowersPart2` (`WorldGen.cs:19898-19939`): `maxTilesX *
// 0.002` attempts (the double ceil loop), each rolling a centre column
// `Next(maxTilesX)` and a ±(Next(10)+7) span, then scanning every column
// top-down for the first grass-2 seat with air above and planting a
// sunflower there (`PlaceTile(j, k-1, 27, mute: true)` → `PlaceSunflower`).
//
// `PlaceSunflower` (`:53636-53686`) draws: the depth gate (`y >
// worldSurface - 1 && !remix` refuses, no draw), the 2×4 clear scan + the
// grass/hallowed-grass floor test (no draw), then ONE `Next(3)` head-variant
// roll + a `Next(3)` per bottom-row cell (the two head rows reuse the
// variant) — 5 draws per placed flower. The 2×4 body writes at
// `(x+k, y+l)`, k 0..2, l -3..1: `frameX = k*18 + variant*36`,
// `frameY = (l+3)*18`.
//
// Sub-vanilla note: `PlaceSunflower` has no InWorld guard (the C# would
// throw on an edge column); the port's bounds-checked reads/writes skip
// out-of-world cells (the pass's span clamp keeps x+1 in-world, and the
// negative head rows only occur on a grass seat above row 4 — never on a
// generated surface).

/// `WorldGen.PlaceSunflower(x, y, 27)` (`WorldGen.cs:53636-53686`) — the
/// depth gate, the 2×4 clear + wall veto, the two-column grass floor, the
/// head-variant roll and the 2×4 frame write. Returns whether it wrote.
fn place_sunflower(
    world: &mut World,
    x: i32,
    y: i32,
    world_surface: f64,
    remix: bool,
    rng: &mut Rng,
) -> bool {
    // `:53638-53641`: below the surface line refuses (remix exempt).
    if (y as f64) > world_surface - 1.0 && !remix {
        return false;
    }
    let mut ok = true;
    for i in x..(x + 2) {
        for j in (y - 3)..(y + 1) {
            if world
                .tile(i, j)
                .is_some_and(|t| t.active() || t.wall > 0)
            {
                ok = false;
            }
        }
        // `:53660`: nactive + not half-brick + slope 0 + grass 2 / hallowed
        // grass 109 under both columns.
        let floor_ok = world.tile(i, y + 1).is_some_and(|t| {
            t.active()
                && !t.inactive()
                && !t.half_brick()
                && t.slope() == 0
                && matches!(t.kind, 2 | 109)
        });
        if !floor_ok {
            ok = false;
        }
    }
    if !ok {
        return false;
    }
    // `:53669`: the head variant.
    let variant = rng.range(0, 3);
    for k in 0..2 {
        for l in -3..1 {
            // `:53674-53678`: the bottom two rows roll per cell, the head
            // rows reuse the variant.
            let frame_x = if l <= -2 {
                k * 18 + variant * 36
            } else {
                k * 18 + rng.range(0, 3) * 36
            };
            let frame_y = (l + 3) * 18;
            if let Some(t) = world.tile_mut(x + k, y + l) {
                t.flags |= FLAG_ACTIVE;
                t.frame_x = frame_x as i16;
                t.frame_y = frame_y as i16;
                t.kind = 27;
            }
        }
    }
    true
}

/// `WorldGen.PlaceTile(i, j, 27, mute: true)` (`WorldGen.cs:59342-60295`),
/// the slice the Sunflowers pass runs: the entry always admits
/// (`tileSolid[27]` is false, `:59373`), the seat is inactive (the pass
/// checked) so the clear runs (`:59473-59476`), the dispatch is
/// `PlaceSunflower` (`:60152-60155`); the `SquareTileFrame` re-frame rides
/// the post-pipeline seam (the d106 convention).
fn place_tile_sunflower(
    world: &mut World,
    x: i32,
    y: i32,
    world_surface: f64,
    remix: bool,
    rng: &mut Rng,
) {
    if world.tile(x, y).is_none() {
        return;
    }
    if !world.tile(x, y).is_some_and(|t| t.active()) {
        if let Some(c) = world.tile_mut(x, y) {
            c.kind = 0;
            c.flags &= !FLAG_ACTIVE;
            c.frame_x = 0;
            c.frame_y = 0;
            c.set_paint(0);
            c.flags &= !crate::tile::FLAG_FULLBRIGHT;
            c.set_slope(0);
            c.flags &= !FLAG_HALF_BRICK;
        }
        world.set_invisible_block(x, y, false);
    }
    place_sunflower(world, x, y, world_surface, remix, rng);
}

/// The Sunflowers pass (`GenPassNameID.SunflowersPart2`,
/// `WorldGen.cs:19898-19939`). Draws, in C# order on the pass's own stream:
/// per attempt the centre roll + the two span rolls, then per placed
/// flower the 5 `PlaceSunflower` draws.
pub(crate) fn pass_sunflowers(
    world: &mut World,
    world_surface: f64,
    rng: &mut Rng,
    opts: &GenOptions,
) {
    // `:19900`: !Skyblock.denySomeGeneration.
    // matrix-88 §3.5: the denySomeGeneration gate rides f24 A1's canonical
    // table (the g38 C1 local copy dropped with the pass).
    if super::skyblock::deny_some_generation(opts.flags, opts.micro_flags) {
        return;
    }
    let remix = opts.flags.contains(SeedFlags::REMIX);
    // `:19903-19904`: the double ceil loop.
    let count = ((world.width as f64) * 0.002).ceil() as i32;
    let num6 = world_surface as i32 - 1;
    for _ in 0..count {
        // `:19910-19920`: the centre + span rolls, the clamps.
        let centre = rng.range(0, world.width);
        let lo = (centre - rng.range(0, 10) - 7).max(0);
        let hi = (centre + rng.range(0, 10) + 7).min(world.width - 1);
        // `:19923-19936`: the per-column top-down scan.
        for j in lo..hi {
            for k in 1..num6 {
                let seat = world.tile(j, k).is_some_and(|t| t.active() && t.kind == 2);
                let air_above = !world.tile(j, k - 1).is_some_and(|t| t.active());
                if seat && air_above {
                    place_tile_sunflower(world, j, k - 1, world_surface, remix, rng);
                }
                if world.tile(j, k).is_some_and(|t| t.active()) {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod sunflower_tests {
    use super::*;
    use crate::tiledata::TILE_COUNT;

    /// A grass-2 surface row at `ground` with air above.
    fn grass_world(ground: i32) -> World {
        let mut w = World::new("sunflowers".into(), 120, 120);
        w.frame_important = vec![false; TILE_COUNT];
        for x in 0..120 {
            for y in ground..120 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: if y == ground { 2 } else { 1 },
                    flags: FLAG_ACTIVE,
                    ..Default::default()
                };
            }
        }
        w
    }

    /// W4-g38 C6: `place_sunflower` writes the 2×4 body — the head rows
    /// share the variant, the bottom rows roll per cell; 5 draws total.
    #[test]
    fn w4_g38_c6_sunflower_frames_and_draw_count() {
        let mut w = grass_world(40);
        let mut rng = Rng(7);
        let mut probe = Rng(7);
        assert!(place_sunflower(&mut w, 60, 39, 60.0, false, &mut rng));
        // The head variant is the first draw.
        let variant = probe.range(0, 3);
        for k in 0..2 {
            for l in -3..1 {
                let t = w.tile(60 + k, 39 + l).unwrap();
                assert!(t.active());
                assert_eq!(t.kind, 27);
                assert_eq!(t.frame_y, ((l + 3) * 18) as i16);
                let v = if l <= -2 { variant } else { probe.range(0, 3) };
                assert_eq!(t.frame_x, (k * 18 + v * 36) as i16);
            }
        }
        // Exactly 5 draws consumed.
        assert_eq!(rng.next_u32(), probe.next_u32());
    }

    /// W4-g38 C6: the vetoes — a wall in the body, a non-grass floor and
    /// the below-surface depth gate all refuse without draws.
    #[test]
    fn w4_g38_c6_sunflower_vetoes_draw_nothing() {
        // Depth gate.
        let mut w = grass_world(40);
        let mut rng = Rng(7);
        assert!(!place_sunflower(&mut w, 60, 39, 30.0, false, &mut rng));
        assert_eq!(rng.next_u32(), Rng(7).next_u32(), "no draw");
        // A wall in the body.
        let mut w = grass_world(40);
        let i = w.index(61, 38);
        w.tiles[i].wall = 2;
        let mut rng = Rng(7);
        assert!(!place_sunflower(&mut w, 60, 39, 60.0, false, &mut rng));
        assert_eq!(rng.next_u32(), Rng(7).next_u32(), "no draw");
        // A stone floor under one column.
        let mut w = grass_world(40);
        let i = w.index(61, 40);
        w.tiles[i].kind = 1;
        let mut rng = Rng(7);
        assert!(!place_sunflower(&mut w, 60, 39, 60.0, false, &mut rng));
        assert_eq!(rng.next_u32(), Rng(7).next_u32(), "no draw");
    }

    /// W4-g38 C6: the pass plants on the grass seats and the deny gate
    /// skips without draws.
    #[test]
    fn w4_g38_c6_pass_plants_and_gate_skips() {
        let mut w = grass_world(40);
        pass_sunflowers(&mut w, 60.0, &mut Rng(3), &GenOptions::default());
        let flowers = w.tiles.iter().filter(|t| t.active() && t.kind == 27).count();
        assert!(flowers > 0, "the 120-wide world plants sunflowers");
        // The skyblock deny gate.
        let mut w = grass_world(40);
        let before = w.tiles.clone();
        let mut opts = GenOptions::default();
        opts.flags.insert(SeedFlags::SKYBLOCK);
        let mut rng = Rng(3);
        pass_sunflowers(&mut w, 60.0, &mut rng, &opts);
        assert_eq!(w.tiles, before);
        assert_eq!(rng.next_u32(), Rng(3).next_u32(), "no draw");
    }
}

// --- W4-g38 C7: the Herbs pass -----------------------------------------------
//
// The vanilla `AlchemyHerbs` pass (`WorldGen.cs:19964-19980`): gated only by
// `Skyblock.denyAllGeneration` (= `skyblockWorldGen`, `:3131` — even the
// surface-rebuilding micro seeds do NOT admit it), it runs a `w * 1.7`
// double-ceil ladder of `PlantAlch` attempts (remix ×5), each planting a
// stage-0 herb (tile 82) on a suitable soil when fewer than 5 herbs already
// sit in its ±num3 box.
//
// The gen-local `PlaceAlch` slice below duplicates the soil/liquid/write
// core of `crate::edit::place_alch` (`edit/place.rs:269`) by design: the
// runtime edit path carries `touch`/`after_edit` side effects (the revision
// bump + the re-frame area) which generation must not run — the re-frame
// rides the post-pipeline seam (the d106 convention), and the 82-84
// frame-important boot bits are already set (`boot_statics`).

/// The per-style soil families of `WorldGen.PlaceAlch`
/// (`WorldGen.cs:45395-45466`) — the same table as `crate::edit::ALCH_SOILS`
/// (`edit.rs:47-55`, private to the runtime edit path); every style also
/// accepts the clay pot 78 and the planter box 380.
const HERB_SOILS: [&[u16]; 7] = [
    &[2, 477, 492, 78, 380, 109],
    &[60, 78, 380],
    &[0, 59, 78, 380],
    &[661, 662, 199, 203, 23, 25, 78, 380],
    &[53, 78, 380, 116],
    &[57, 633, 78, 380],
    &[78, 380, 147, 161, 163, 164, 200],
];

/// `WorldGen.PlaceAlch(x, y, style)` (`WorldGen.cs:45380-45479`), the
/// generation slice: the cell-empty + `nactive` flat-soil gate
/// (`:45392`), the per-style soil sets + liquid rules (`:45395-45466` —
/// styles 0-3 refuse any liquid, 4/6 refuse lava only, 5 refuses
/// everything but lava), then the write `active + 82 + frameX = 18*style +
/// frameY = 0` and `CopyPaintAndCoating(soil)` (`:45470-45474`,
/// `Tile.cs:867-872` — paint + invisible block + fullbright). The C#
/// null-creates the cells (`:45382-45388`); the port's bounds-checked
/// reads refuse out-of-world coordinates instead. No draws.
fn place_alch_gen(world: &mut World, x: i32, y: i32, style: i32) -> bool {
    let Some(herb) = world.tile(x, y).copied() else {
        return false;
    };
    let Some(soil) = world.tile(x, y + 1).copied() else {
        return false;
    };
    // `:45392`: !active && nactive(below) && !halfBrick && slope 0.
    if herb.active()
        || !crate::tileobject::nactive(&soil)
        || soil.half_brick()
        || soil.slope() != 0
    {
        return false;
    }
    // `:45395-45466`: the soil set + the liquid rule per style.
    let soil_ok = match style {
        0..=6 => HERB_SOILS[style as usize].contains(&soil.kind),
        // No case matches: the C# switch falls through with the reject
        // flag still false — an out-of-range style plants unchecked.
        _ => true,
    };
    let herb_lava = herb.liquid_kind() == crate::tile::LiquidKind::Lava;
    let liquid_ok = match style {
        0..=3 => herb.liquid == 0,
        4 | 6 => !(herb.liquid > 0 && herb_lava),
        5 => !(herb.liquid > 0 && !herb_lava),
        _ => true,
    };
    if !soil_ok || !liquid_ok {
        return false;
    }
    // `:45470-45474`: the write + `CopyPaintAndCoating(tile2)`.
    if let Some(cell) = world.tile_mut(x, y) {
        cell.flags |= FLAG_ACTIVE;
        cell.kind = 82;
        cell.frame_x = (18 * style) as i16;
        cell.frame_y = 0;
        cell.set_paint(soil.paint());
        cell.flags = (cell.flags & !crate::tile::FLAG_FULLBRIGHT)
            | (soil.flags & crate::tile::FLAG_FULLBRIGHT);
    }
    world.set_invisible_block(x, y, world.invisible_block(x, y + 1));
    true
}

/// `WorldGen.PlaceSuitableHerbHere(x, y)` (`WorldGen.cs:45940-45970`):
/// the seven sequential soil-kind ifs — NOT else-if, one soil kind can
/// fire several arms (each arm targets a different style, so at most one
/// write lands: the first `PlaceAlch` fills the cell, the rest refuse on
/// the active gate). The sand/pearlsand arm (style 4) carries the
/// `beachDistance` band (`WorldGen.beachDistance = 380`, `:86914`).
fn place_suitable_herb_here(world: &mut World, x: i32, y: i32) {
    let Some(kind) = world.tile(x, y).map(|t| t.kind) else {
        return;
    };
    if kind == 2 || kind == 109 {
        place_alch_gen(world, x, y - 1, 0);
    }
    if kind == 60 {
        place_alch_gen(world, x, y - 1, 1);
    }
    if kind == 0 || kind == 59 {
        place_alch_gen(world, x, y - 1, 2);
    }
    if matches!(kind, 23 | 661 | 25 | 203 | 199 | 662) {
        place_alch_gen(world, x, y - 1, 3);
    }
    if (kind == 53 || kind == 116) && x >= 380 && x <= world.width - 380 {
        place_alch_gen(world, x, y - 1, 4);
    }
    if kind == 57 || kind == 633 {
        place_alch_gen(world, x, y - 1, 5);
    }
    if matches!(kind, 147 | 163 | 164 | 161 | 200) {
        place_alch_gen(world, x, y - 1, 6);
    }
}

/// `WorldGen.PlantAlch()` (`WorldGen.cs:45901-45938`). Draws, in C# order
/// on the pass stream: the x roll, the y-ladder rolls (remix: one full
/// band roll; classic: `Next(40)` then — on 0 — the deep band roll, else
/// `Next(10)` then the surface/deep band roll). The fall-to-ground scan,
/// the `nactive`/above-active/above-liquid refuse and the ±num3 herb-count
/// box are draw-free. `num4 = 5` herbs veto a fresh plant.
fn plant_alch(
    world: &mut World,
    rng: &mut Rng,
    world_surface: f64,
    rock_layer: f64,
    remix: bool,
) {
    let (w, h) = (world.width, world.height);
    // `:45903-45905`: the x roll + the y ladder.
    let x = rng.range(20, w - 20);
    let mut y = if remix {
        rng.range(20, h - 20)
    } else if rng.chance(40) {
        // Next(40) == 0 → the deep band, from `(int)(rockLayer + h) / 2`
        // (the C# casts the double sum to int, then divides).
        rng.range((rock_layer + h as f64) as i32 / 2, h - 20)
    } else if !rng.chance(10) {
        // Next(10) != 0 → the surface band.
        rng.range(world_surface as i32, h - 20)
    } else {
        rng.range(20, h - 20)
    };
    // `:45905`: fall while the cell is inactive, stop at h - 20.
    while y < h - 20 && !world.tile(x, y).is_some_and(|t| t.active()) {
        y += 1;
    }
    // `:45908`: the landing refuse — not nactive, or the cell above is
    // active or carries liquid.
    let landed = world
        .tile(x, y)
        .is_some_and(|t| crate::tileobject::nactive(t));
    let above_blocked = world
        .tile(x, y - 1)
        .is_some_and(|t| t.active() || t.liquid != 0);
    if !landed || above_blocked {
        return;
    }
    // `:45912-45919`: the ±num3 count box, clamped into [4, w-4]/[4, h-4].
    let num3 = (15.0 * (w as f64 / 4200.0)) as i32;
    let x_lo = (x - num3).clamp(4, w - 4);
    let x_hi = (x + num3).clamp(4, w - 4);
    let y_lo = (y - num3).clamp(4, h - 4);
    let y_hi = (y + num3).clamp(4, h - 4);
    let mut herbs = 0;
    for i in x_lo..=x_hi {
        for j in y_lo..=y_hi {
            if world
                .tile(i, j)
                .is_some_and(|t| crate::tileobject::tile_alch(t.kind))
            {
                herbs += 1;
            }
        }
    }
    // `:45930-45932`: fewer than 5 herbs in the box → plant. (The C#'s
    // netMode-2 SendTileSquare tail is a server broadcast, not state.)
    if herbs < 5 {
        place_suitable_herb_here(world, x, y);
    }
}

/// The Herbs pass (`GenPassNameID.AlchemyHerbs`, `WorldGen.cs:19964-19980`).
/// The `w * 1.7` double-ceil ladder (remix ×5) of `PlantAlch` attempts on
/// the pass's own stream; gated by `Skyblock.denyAllGeneration` only.
pub(crate) fn pass_herbs(
    world: &mut World,
    world_surface: f64,
    rock_layer: f64,
    rng: &mut Rng,
    opts: &GenOptions,
) {
    // `:19966`: !Skyblock.denyAllGeneration (= skyblockWorldGen, :3131).
    if opts.flags.contains(SeedFlags::SKYBLOCK) {
        return;
    }
    let remix = opts.flags.contains(SeedFlags::REMIX);
    // `:19968-19974`: the double count, the ceil loop (`(double)i < num`).
    let mut count = world.width as f64 * 1.7;
    if remix {
        count *= 5.0;
    }
    let count = count.ceil() as i32;
    for _ in 0..count {
        plant_alch(world, rng, world_surface, rock_layer, remix);
    }
}

#[cfg(test)]
mod herb_tests {
    use super::*;
    use crate::tiledata::TILE_COUNT;

    /// A grass-2 surface row at `ground` with air above.
    fn grass_world(ground: i32) -> World {
        let mut w = World::new("herbs".into(), 120, 120);
        w.frame_important = vec![false; TILE_COUNT];
        for x in 0..120 {
            for y in ground..120 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: if y == ground { 2 } else { 1 },
                    flags: FLAG_ACTIVE,
                    ..Default::default()
                };
            }
        }
        w
    }

    /// W4-g38 C7: `place_alch_gen` writes the stage-0 herb on its soil —
    /// the 18*style frameX, the paint/coating copy — and refuses the wrong
    /// soil, the liquid rules and an occupied cell. No draws anywhere.
    #[test]
    fn w4_g38_c7_place_alch_gen_soil_liquid_and_paint() {
        let mut w = grass_world(40);
        // Style 0 on grass 2: the write + the coating copy.
        let i = w.index(60, 40);
        w.tiles[i].set_paint(7);
        w.tiles[i].flags |= crate::tile::FLAG_FULLBRIGHT;
        w.set_invisible_block(60, 40, true);
        assert!(place_alch_gen(&mut w, 60, 39, 0));
        let t = w.tile(60, 39).unwrap();
        assert!(t.active());
        assert_eq!(t.kind, 82);
        assert_eq!(t.frame_x, 0);
        assert_eq!(t.frame_y, 0);
        assert_eq!(t.paint(), 7);
        assert!(t.flags & crate::tile::FLAG_FULLBRIGHT != 0);
        assert!(w.invisible_block(60, 39));
        // The occupied cell refuses (the second sequential arm).
        assert!(!place_alch_gen(&mut w, 60, 39, 0));
        // Style 1 refuses grass (mushroom-grass soil only).
        assert!(!place_alch_gen(&mut w, 61, 39, 1));
        // Style 2 on mud 59.
        let i = w.index(62, 40);
        w.tiles[i].kind = 59;
        assert!(place_alch_gen(&mut w, 62, 39, 2));
        assert_eq!(w.tile(62, 39).unwrap().frame_x, 36);
        // Style 5 (ash) accepts lava, refuses water; style 0 refuses both.
        let i = w.index(63, 40);
        w.tiles[i].kind = 57;
        let i = w.index(63, 39);
        w.tiles[i].liquid = 255;
        w.tiles[i].flags |= crate::tile::FLAG_LIQUID_LAVA;
        assert!(place_alch_gen(&mut w, 63, 39, 5));
        let i = w.index(64, 40);
        w.tiles[i].kind = 57;
        let i = w.index(64, 39);
        w.tiles[i].liquid = 255; // water
        assert!(!place_alch_gen(&mut w, 64, 39, 5));
        let i = w.index(65, 39);
        w.tiles[i].liquid = 255;
        assert!(!place_alch_gen(&mut w, 65, 39, 0));
    }

    /// W4-g38 C7: `plant_alch`'s draw schedule — the x roll, the Next(40)
    /// / Next(10) ladder, the band roll (3 draws on the deep arm, 4 on
    /// the others) — then the fall + plant. The stream stays in sync,
    /// and the landing follows the C# fall + refuse rules.
    #[test]
    fn w4_g38_c7_plant_alch_draw_schedule_and_fall() {
        let mut w = grass_world(40);
        let before = w.tiles.clone();
        let mut rng = Rng(11);
        let mut probe = Rng(11);
        // The C# ladder rolls, in order (`:45903-45905`).
        let x = probe.range(20, 100);
        let y0 = if probe.chance(40) {
            probe.range((60.0 + 120.0) as i32 / 2, 100)
        } else if !probe.chance(10) {
            probe.range(30, 100)
        } else {
            probe.range(20, 100)
        };
        plant_alch(&mut w, &mut rng, 30.0, 60.0, false);
        assert_eq!(rng.next_u32(), probe.next_u32(), "the streams stay in sync");
        // The fall stops at the first active cell; the plant only lands
        // when that cell is the surface row (a deeper landing has an
        // active dirt cell above → the `:45908` refuse).
        if y0 <= 40 {
            let t = w.tile(x, 39).unwrap();
            assert!(t.active(), "a herb at ({x},39)");
            assert_eq!(t.kind, 82);
            assert_eq!(t.frame_x, 0, "grass soil → style 0");
        } else {
            assert_eq!(w.tiles, before, "the above-active refuse");
        }
    }

    /// W4-g38 C7: the five-herb box veto and the pass gates — the
    /// skyblock denyAllGeneration gate skips without draws, the classic
    /// pass plants herbs.
    #[test]
    fn w4_g38_c7_pass_plants_and_gates() {
        // An 840-wide world → num3 = 3 (`:45915`); herbs buried in row 41
        // put 7 tileAlch kinds in every landing box (rows 37..43) — the
        // `< 5` count gate (`:45930`) vetoes every fresh plant while the
        // above-cell check stays clear.
        let mut w = World::new("herbs".into(), 840, 120);
        w.frame_important = vec![false; TILE_COUNT];
        for x in 0..840 {
            for y in 40..120 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: if y == 40 {
                        2
                    } else if y == 41 {
                        82
                    } else {
                        1
                    },
                    flags: FLAG_ACTIVE,
                    ..Default::default()
                };
            }
        }
        let before = w.tiles.clone();
        pass_herbs(&mut w, 30.0, 60.0, &mut Rng(5), &GenOptions::default());
        assert_eq!(w.tiles, before, "the five-herb box veto");
        // The classic pass plants.
        let mut w = grass_world(40);
        pass_herbs(&mut w, 30.0, 60.0, &mut Rng(5), &GenOptions::default());
        let herbs = w.tiles.iter().filter(|t| t.active() && t.kind == 82).count();
        assert!(herbs > 0, "the 120-wide world plants herbs");
        // The skyblock gate: no tiles, no draws (denyAllGeneration admits
        // no micro seed — unlike denySomeGeneration).
        let mut w = grass_world(40);
        let before = w.tiles.clone();
        let mut opts = GenOptions::default();
        opts.flags.insert(SeedFlags::SKYBLOCK);
        opts.micro_flags.insert(MicroSeedFlags::SURFACE_IS_DESERT);
        let mut rng = Rng(5);
        pass_herbs(&mut w, 30.0, 60.0, &mut rng, &opts);
        assert_eq!(w.tiles, before);
        assert_eq!(rng.next_u32(), Rng(5).next_u32(), "no draw");
    }
}

// --- W4-g38 C8: the Dye Plants pass ------------------------------------------
//
// The vanilla `DyePlants` pass (`WorldGen.cs:19981-19994`): gated only by
// `Skyblock.denyAllGeneration` (`:3131`), it runs `w` `plantDye` attempts
// (`Next(100, w-100)` × `Next(100, UnderworldLayer)`, the `exoticPlant =
// false` path only), then `MatureTheHerbPlants` (the stage rolls over every
// cell), `GrowGlowTulips` (the 2/4/6 edge-band budget loop) and
// `PlaceChilletEggs` (the 6/9/12 cave budget loop). All draws ride the
// pass's own stream, in C# call order.
//
// Sub-vanilla note: `plantDye` reads neighbour cells without an InWorld
// guard beyond its own 95 margin (`:71289`) — on a generated world the
// margin keeps every read in-world, but a sub-vanilla small world (the
// 512×512 pin world) can roll `j > h-95`, where the C# throws; the port's
// bounds-checked reads treat out-of-world cells as inert and the pass's
// width/height guards skip the degenerate bands (the C2/C3 convention).

/// The kinds that count as "clear above/below" for a dye plant — the
/// grass/flower family the `plantDye` above-check (`:71298`/`:71382`) and
/// the `PlaceDye` refuse lists (`:53997`/`:54008`) exempt. The two lists
/// differ: `PlaceDye` also exempts 697/703.
const DYE_CLEAR_ABOVE: &[u16] = &[3, 51, 61, 73, 74, 184];

/// `WorldGen.SolidTile(int, int)` (`WorldGen.cs:70111-70131`): active +
/// solid + !solidTop + !halfBrick + slope 0 + !inactive. Null reads solid
/// (unreachable on the port's dense grid); out-of-bounds reads false (the
/// C# catch).
fn dye_solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && TILE_SOLID.has(t.kind)
        && !td::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

/// `WorldGen.PlatformProperTopFrame` (`WorldGen.cs:70153-70165`) — the
/// flat platform frame bands (the same slice as structures.rs's
/// `trap_platform_proper_top_frame`).
fn dye_platform_proper_top_frame(frame_x: i16) -> bool {
    let num = i32::from(frame_x) / 18;
    (0..=7).contains(&num) || (12..=16).contains(&num) || (25..=26).contains(&num)
}

/// `WorldGen.SolidTileAllowBottomSlope(int, int)`
/// (`WorldGen.cs:70167-70189`): out-of-world reads solid (the InWorld
/// quirk); active + (solid || solidTop) + (!topSlope || platform-proper-
/// frame) + !halfBrick + !inactive.
fn dye_solid_allow_bottom_slope(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && (TILE_SOLID.has(t.kind) || td::TILE_SOLID_TOP.has(t.kind))
        && (!t.top_slope()
            || (td::SET_PLATFORMS.has(t.kind) && dye_platform_proper_top_frame(t.frame_x)))
        && !t.half_brick()
        && !t.inactive()
}

/// `WorldGen.HasValidGroundForGlowTulipBelowSpot` (`WorldGen.cs:73096-73117`)
/// — `HasValidGroundForChilletEggBelowSpot` (`:45725-45746`) is the SAME
/// body: InWorld(x, y, 2), the cell below active, its kind in the ground
/// family (0/70/633/59/225 + Conversion.Grass + Conversion.Stone +
/// tileMoss), then SolidTileAllowBottomSlope.
fn dye_valid_ground(world: &World, x: i32, y: i32) -> bool {
    if !super::temple::in_world(world, x, y, 2) {
        return false;
    }
    let Some(below) = world.tile(x, y + 1) else {
        return false;
    };
    if !below.active() {
        return false;
    }
    let kind = below.kind;
    if !matches!(kind, 0 | 70 | 633 | 59 | 225)
        && !td::SET_GRASS.has(kind)
        && !td::SET_CONVERSION_STONE.has(kind)
        && !td::TILE_MOSS.has(kind)
    {
        return false;
    }
    dye_solid_allow_bottom_slope(world, x, y + 1)
}

/// `WorldGen.PlaceDye(x, y, style)` (`WorldGen.cs:53988-54064`): the
/// style-7 ceiling arm (the below cell must be inactive-or-flora, the
/// seat above solid) vs the floor arms (the above cell inactive-or-flora;
/// style 6 the cactus seat, style 5 full water, styles 8-11 unchecked, the
/// default liquid-0 + the 3/4 wall-0 rule), then the write `227 + active +
/// !halfBrick + slope 0 + frameY 0 + frameX = 34*style`. No draws.
fn place_dye(world: &mut World, x: i32, y: i32, style: i32) {
    // `:53991-53994`: both neighbours must exist.
    let (Some(below), Some(above)) = (world.tile(x, y + 1).copied(), world.tile(x, y - 1).copied())
    else {
        return;
    };
    let Some(cell) = world.tile(x, y).copied() else {
        return;
    };
    let mut ok = false;
    if style == 7 {
        // `:53997`: an active below cell refuses unless it is flora.
        if below.active()
            && !matches!(below.kind, 3 | 51 | 697 | 61 | 703 | 73 | 74 | 184)
        {
            return;
        }
        // `:54001`: solid above + inactive below.
        if dye_solid_tile(world, x, y - 1) && !below.active() {
            ok = true;
        }
    } else {
        // `:54008`: an active above cell refuses unless it is flora.
        if above.active()
            && !matches!(above.kind, 3 | 51 | 697 | 61 | 703 | 73 | 74 | 184)
        {
            return;
        }
        if style == 6 {
            // `:54014`: the cactus seat — nactive cactus 80 below with
            // both side cells inactive.
            let sides_clear = !world.tile(x - 1, y + 1).is_some_and(|t| t.active())
                && !world.tile(x + 1, y + 1).is_some_and(|t| t.active());
            if crate::tileobject::nactive(&below) && below.kind == 80 && sides_clear {
                ok = true;
            }
        } else if dye_solid_tile(world, x, y + 1) && !above.active() {
            // `:54019-54052`: the per-style liquid/wall rules.
            ok = match style {
                5 => {
                    cell.liquid == 255
                        && cell.liquid_kind() == crate::tile::LiquidKind::Water
                }
                8..=11 => true,
                _ => {
                    cell.liquid == 0
                        && ((style != 3 && style != 4) || cell.wall == 0)
                }
            };
        }
    }
    if ok {
        // `:54055-54062`: the write.
        if let Some(t) = world.tile_mut(x, y) {
            t.kind = 227;
            t.flags |= FLAG_ACTIVE;
            t.flags &= !FLAG_HALF_BRICK;
            t.set_slope(0);
            t.frame_y = 0;
            t.frame_x = (34 * style) as i16;
        }
    }
}

/// `WorldGen.PlaceTile(i, j, 227, mute, forced, -1, style)`
/// (`WorldGen.cs:59342-60295`), the slice the Dye Plants pass runs: the
/// entry admits (`tileSolid[227]` is false, `:59373`), the inactive-cell
/// clear (`:59473-59476`), the dispatch `PlaceDye` (`:60042-60045`); the
/// `SquareTileFrame` re-frame rides the post-pipeline seam.
fn dye_place_tile_227(world: &mut World, x: i32, y: i32, style: i32) {
    // `:59345`: the gen-time fallen-log guard — an active 488 refuses.
    if world
        .tile(x, y)
        .is_some_and(|t| t.active() && t.kind == 488)
    {
        return;
    }
    if world.tile(x, y).is_none() {
        return;
    }
    if !world.tile(x, y).is_some_and(|t| t.active()) {
        if let Some(c) = world.tile_mut(x, y) {
            c.kind = 0;
            c.flags &= !FLAG_ACTIVE;
            c.frame_x = 0;
            c.frame_y = 0;
            c.set_paint(0);
            c.flags &= !crate::tile::FLAG_FULLBRIGHT;
            c.set_slope(0);
            c.flags &= !FLAG_HALF_BRICK;
        }
        world.set_invisible_block(x, y, false);
    }
    place_dye(world, x, y, style);
}

/// The ±90 dye-density scan of `plantDye`'s surface/underground arms
/// (`:71300-71313`/`:71384-71397`): clamped box `[i-90, i+90)` ×
/// `[j-90, j+90)` (bounds 1..w-2 / 1..h-2), any active 227 with
/// `frameX < 272` (the classic styles) refuses. No draws.
fn dye_nearby_classic(world: &World, i: i32, j: i32) -> bool {
    let (w, h) = (world.width, world.height);
    let x_lo = (i - 90).clamp(1, w - 2);
    let x_hi = (i + 90).clamp(1, w - 2);
    let y_lo = (j - 90).clamp(1, h - 2);
    let y_hi = (j + 90).clamp(1, h - 2);
    for k in x_lo..x_hi {
        for l in y_lo..y_hi {
            if world
                .tile(k, l)
                .is_some_and(|t| t.active() && t.kind == 227 && t.frame_x < 272)
            {
                return true;
            }
        }
    }
    false
}

/// `WorldGen.plantDye(i, j)` (`WorldGen.cs:71286-71455`), the
/// `exoticPlant = false` path only (the pass's call, `:19988`). Draws: the
/// surface arm's grass/mushroom style rolls, the underground arm's
/// mushroom/dirt rolls — in C# arm order. The arms are sequential ifs,
/// NOT else-if: on a classic world the surface and underground gates are
/// mutually exclusive, but on remix BOTH run (the surface gate admits
/// every j, the underground gate only refuses j ≥ UnderworldLayer); the
/// ceiling arm runs after both.
fn plant_dye(
    world: &mut World,
    rng: &mut Rng,
    i: i32,
    j: i32,
    world_surface: f64,
    remix: bool,
) {
    let (w, h) = (world.width, world.height);
    // `:71289`: the entry refuse — inactive seat or outside the 95 margin.
    if !world.tile(i, j).is_some_and(|t| t.active())
        || i < 95
        || i > w - 95
        || j < 95
        || j > h - 95
    {
        return;
    }
    let seat_kind = world.tile(i, j).map(|t| t.kind).unwrap_or(0);
    let above_clear = |world: &World| {
        world
            .tile(i, j - 1)
            .is_none_or(|t| !t.active() || DYE_CLEAR_ABOVE.contains(&t.kind))
    };
    // `:71298-71377`: the surface arm.
    if ((j as f64) < world_surface || remix) && above_clear(world) {
        if dye_nearby_classic(world, i, j) {
            return;
        }
        if seat_kind == 2 || seat_kind == 109 {
            // `:71327-71337`: Next(4) == 0 → style 4, else 3.
            let style = if rng.chance(4) { 4 } else { 3 };
            dye_place_tile_227(world, i, j - 1, style);
        } else if seat_kind == 60 {
            // `:71338-71348`: Next(2) == 0 → style 2, else 4.
            let style = if rng.chance(2) { 2 } else { 4 };
            dye_place_tile_227(world, i, j - 1, style);
        } else if seat_kind == 53
            && world.tile(i, j - 1).is_some_and(|t| t.liquid == 255)
            && world.tile(i, j - 2).is_some_and(|t| t.liquid == 255)
        {
            // `:71349-71352`: the submerged sand — style 2, no draw.
            dye_place_tile_227(world, i, j - 1, 2);
        } else if seat_kind == 80
            && !world.tile(i - 1, j - 1).is_some_and(|t| t.active())
            && !world.tile(i + 1, j - 1).is_some_and(|t| t.active())
        {
            // `:71353-71376`: the cactus — no ebonstone/pearlstone
            // (112/234) in the ±5 × (j-5..j+15) box → style 6.
            let mut clear = true;
            for m in (i - 5)..=(i + 5) {
                for n in (j - 5)..=(j + 15) {
                    if world
                        .tile(m, n)
                        .is_some_and(|t| t.active() && (t.kind == 112 || t.kind == 234))
                    {
                        clear = false;
                    }
                }
            }
            if clear {
                dye_place_tile_227(world, i, j - 1, 6);
            }
        }
    }
    // `:71378-71381`: the underground gate.
    if ((j as f64) < world_surface && !remix) || j >= h - 200 {
        return;
    }
    // `:71382-71436`: the underground arm.
    if above_clear(world) {
        if dye_nearby_classic(world, i, j) {
            return;
        }
        if seat_kind == 60 {
            // `:71406-71420`: Next(2) == 0 → 2, elif Next(2) == 0 → 0,
            // else 1.
            if rng.chance(2) {
                dye_place_tile_227(world, i, j - 1, 2);
            } else if rng.chance(2) {
                dye_place_tile_227(world, i, j - 1, 0);
            } else {
                dye_place_tile_227(world, i, j - 1, 1);
            }
        } else if matches!(seat_kind, 0 | 1 | 59) {
            // `:71421-71431`: Next(2) == 0 → 0, else 1.
            let style = if rng.chance(2) { 0 } else { 1 };
            dye_place_tile_227(world, i, j - 1, style);
        } else if world.tile(i, j - 1).is_some_and(|t| t.liquid == 255)
            && world.tile(i, j - 2).is_some_and(|t| t.liquid == 255)
        {
            // `:71432-71435`: the submerged arm — style 5, no draw.
            dye_place_tile_227(world, i, j - 1, 5);
        }
    }
    // `:71437-71454`: the ceiling arm — an active below cell refuses, the
    // UNCLAMPED ±90 scan counts ANY active 227, then dirt seats style 7.
    if world.tile(i, j + 1).is_some_and(|t| t.active()) {
        return;
    }
    for k in (i - 90)..(i + 90) {
        for l in (j - 90)..(j + 90) {
            if world
                .tile(k, l)
                .is_some_and(|t| t.active() && t.kind == 227)
            {
                return;
            }
        }
    }
    if seat_kind == 0 {
        dye_place_tile_227(world, i, j + 1, 7);
    }
}

/// `WorldGen.MatureTheHerbPlants()` (`WorldGen.cs:45748-45775`): the
/// 10-margin column-major sweep. The deep-mud plant (`j > rockLayer`,
/// mud/dirt below solid, cell inactive + liquid 0, `Next(25) == 0`) writes
/// a stage-1 herb (82, frameX 36); then EVERY cell whose kind reads 82
/// rolls `Next(3)` → 83, and inside that the frameX-36 `Next(2)` / the
/// frameX-108 `Next(3)` → 84. The kind tests read the cell's stored type
/// even when inactive (the C# quirk, kept 1:1); the draws are conditional
/// in exactly the C# order.
fn mature_the_herb_plants(world: &mut World, rng: &mut Rng, rock_layer: f64) {
    let (w, h) = (world.width, world.height);
    for i in 10..(w - 10) {
        for j in 10..(h - 10) {
            // `:45754-45760`: the deep plant.
            let plant_ok = (j as f64) > rock_layer
                && world
                    .tile(i, j + 1)
                    .is_some_and(|t| t.kind == 59 || t.kind == 0)
                && dye_solid_tile(world, i, j + 1)
                && world.tile(i, j).is_some_and(|t| !t.active() && t.liquid == 0);
            if plant_ok && rng.chance(25) {
                if let Some(t) = world.tile_mut(i, j) {
                    t.flags |= FLAG_ACTIVE;
                    t.kind = 82;
                    t.frame_x = 36;
                    t.frame_y = 0;
                }
            }
            // `:45761-45772`: the stage rolls.
            if world.tile(i, j).is_some_and(|t| t.kind == 82) && rng.chance(3) {
                if let Some(t) = world.tile_mut(i, j) {
                    t.kind = 83;
                }
                let frame_x = world.tile(i, j).map(|t| t.frame_x).unwrap_or(0);
                if frame_x == 36 && rng.chance(2) {
                    if let Some(t) = world.tile_mut(i, j) {
                        t.kind = 84;
                    }
                }
                if frame_x == 108 && rng.chance(3) {
                    if let Some(t) = world.tile_mut(i, j) {
                        t.kind = 84;
                    }
                }
            }
        }
    }
}

/// `WorldGen.NoNearbyGlowTulips(i, j)` (`WorldGen.cs:73144-73162`): the
/// ±120 box clamped into [10, w-11] × [10, h-11], any active 656 refuses.
fn no_nearby_glow_tulips(world: &World, i: i32, j: i32) -> bool {
    let (w, h) = (world.width, world.height);
    for k in (i - 120).clamp(10, w - 11)..=(i + 120).clamp(10, w - 11) {
        for l in (j - 120).clamp(10, h - 11)..=(j + 120).clamp(10, h - 11) {
            if world
                .tile(k, l)
                .is_some_and(|t| t.active() && t.kind == 656)
            {
                return false;
            }
        }
    }
    true
}

/// `WorldGen.PlaceTile(i, j, 656, mute: true)` (`WorldGen.cs:59342-60295`),
/// the glow-tulip slice: the entry admits (`tileSolid[656]` false), the
/// liquid refuse (`:59468` — 656 is in the no-liquid list), the
/// inactive-cell clear (`:59473-59476`), then the 656/701 arm
/// (`:59495-59504`): `(!active || tileCut || BreakableWhenPlacing) &&
/// HasValidGroundForGlowTulipBelowSpot` → the flat frame-0 write.
fn dye_place_tile_656(world: &mut World, x: i32, y: i32) {
    let Some(cell) = world.tile(x, y).copied() else {
        return;
    };
    // `:59345`: the gen-time fallen-log guard — an active 488 refuses.
    if cell.active() && cell.kind == 488 {
        return;
    }
    // `:59454/:59468`: any liquid refuses.
    if cell.liquid > 0 {
        return;
    }
    let replaceable = !cell.active()
        || crate::tileobject::tile_cut(cell.kind)
        || td::SET_BREAKABLE_WHEN_PLACING.has(cell.kind);
    if !replaceable || !dye_valid_ground(world, x, y) {
        return;
    }
    if !cell.active() {
        if let Some(c) = world.tile_mut(x, y) {
            c.kind = 0;
            c.flags &= !FLAG_ACTIVE;
            c.frame_x = 0;
            c.frame_y = 0;
            c.set_paint(0);
            c.flags &= !crate::tile::FLAG_FULLBRIGHT;
            c.set_slope(0);
            c.flags &= !FLAG_HALF_BRICK;
        }
        world.set_invisible_block(x, y, false);
    }
    if let Some(t) = world.tile_mut(x, y) {
        t.flags |= FLAG_ACTIVE;
        t.kind = 656;
        t.flags &= !FLAG_HALF_BRICK;
        t.set_slope(0);
        t.frame_x = 0;
        t.frame_y = 0;
    }
}

/// `WorldGen.TryGrowingGlowTulip(i, j)` (`WorldGen.cs:73119-73142`): five
/// attempts, each rolling the ±10 jittered x AND y (both clamped into the
/// 10 margin), the ground + nearby gates, then the PlaceTile(656) write;
/// success = the rolled cell ended active-656.
fn try_growing_glow_tulip(world: &mut World, rng: &mut Rng, i: i32, j: i32) -> bool {
    let (w, h) = (world.width, world.height);
    for _ in 0..5 {
        let x = rng.range((i - 10).max(10), (i + 10).min(w - 10));
        let y = rng.range((j - 10).max(10), (j + 10).min(h - 10));
        if !dye_valid_ground(world, x, y) || !no_nearby_glow_tulips(world, x, y) {
            continue;
        }
        dye_place_tile_656(world, x, y);
        if world
            .tile(x, y)
            .is_some_and(|t| t.active() && t.kind == 656)
        {
            return true;
        }
    }
    false
}

/// `WorldGen.GrowGlowTulips()` (`WorldGen.cs:45622-45650`): the target
/// 2/4/6 by width (>4200 → 4, >6400 → 6), the 10000-attempt budget with
/// the ≤7500 desperation tail. Per attempt: once half the target sits,
/// the right-edge band takes over (in desperation always, else on
/// `Next(2) != 0`); the y band is `[worldSurface, h-200)` (remix
/// `h-350`).
fn grow_glow_tulips(world: &mut World, rng: &mut Rng, world_surface: f64, remix: bool) {
    let (w, h) = (world.width, world.height);
    let target = if w > 6400 {
        6
    } else if w > 4200 {
        4
    } else {
        2
    };
    let mut placed = 0;
    let mut budget = 10000;
    let desperate = (10000.0 * 0.75) as i32;
    while placed < target {
        budget -= 1;
        if budget <= 0 {
            break;
        }
        // `:45642`: the x band switch + the y band.
        let x = if placed >= target / 2 && (budget <= desperate || !rng.chance(2)) {
            rng.range(w - 300, w - 100)
        } else {
            rng.range(100, 300)
        };
        let y_max = if remix { h - 350 } else { h - 200 };
        let y = rng.range(world_surface as i32, y_max);
        if try_growing_glow_tulip(world, rng, x, y) {
            placed += 1;
        }
    }
}

/// `WorldGen.NoNearbyChilletEggs(i, j)` (`WorldGen.cs:45680-45698`): the
/// ±20 box clamped into [10, w-11] × [10, h-11], any active 752 refuses.
fn no_nearby_chillet_eggs(world: &World, i: i32, j: i32) -> bool {
    let (w, h) = (world.width, world.height);
    for k in (i - 20).clamp(10, w - 11)..=(i + 20).clamp(10, w - 11) {
        for l in (j - 20).clamp(10, h - 11)..=(j + 20).clamp(10, h - 11) {
            if world
                .tile(k, l)
                .is_some_and(|t| t.active() && t.kind == 752)
            {
                return false;
            }
        }
    }
    true
}

/// `WorldGen.Place2x2(x, y, 752, 0)` (`WorldGen.cs:51330-51423`), the
/// style-0 chillet-egg path — the same slice as structures.rs's
/// `trap_place_2x2`: the 5-tile bounds gate, the 2×2 `(x-1..x, y-1..y)`
/// body empty, both floor columns `nactive && (SolidTile2(Tile) ||
/// tileTable)`, then the 2×2 write `frameX = k*18`, `frameY = l*18`.
fn dye_place_2x2_752(world: &mut World, x: i32, y: i32) {
    if x < 5 || x > world.width - 5 || y < 5 || y > world.height - 5 {
        return;
    }
    for i in (x - 1)..x + 1 {
        for j in (y - 1)..y + 1 {
            if world.tile(i, j).is_some_and(|t| t.active()) {
                return;
            }
        }
        let Some(below) = world.tile(i, y + 1) else {
            return;
        };
        // `SolidTile2(Tile)` (`:70134-70151`): active + solid + slope 0 +
        // !halfBrick + !inactive (no platform arm on the Tile overload).
        let solid2 = below.active()
            && TILE_SOLID.has(below.kind)
            && below.slope() == 0
            && !below.half_brick()
            && !below.inactive();
        if !crate::tileobject::nactive(below)
            || (!solid2 && !crate::tileobject::is_tile_table(below.kind))
        {
            return;
        }
    }
    for k in 0..2 {
        for l in 0..2 {
            if let Some(t) = world.tile_mut(x - 1 + k, y - 1 + l) {
                t.flags |= FLAG_ACTIVE;
                t.frame_x = (k * 18) as i16;
                t.frame_y = (l * 18) as i16;
                t.kind = 752;
            }
        }
    }
}

/// `WorldGen.PlaceTile(i, j, 752, mute: true)` (`WorldGen.cs:59342-60295`),
/// the chillet-egg slice: the entry admits (`tileSolid[752]` false), the
/// inactive-cell clear, the dispatch `Place2x2` style 0 (`:60196-60198`).
fn dye_place_tile_752(world: &mut World, x: i32, y: i32) {
    // `:59345`: the gen-time fallen-log guard — an active 488 refuses.
    if world
        .tile(x, y)
        .is_some_and(|t| t.active() && t.kind == 488)
    {
        return;
    }
    if world.tile(x, y).is_none() {
        return;
    }
    if !world.tile(x, y).is_some_and(|t| t.active()) {
        if let Some(c) = world.tile_mut(x, y) {
            c.kind = 0;
            c.flags &= !FLAG_ACTIVE;
            c.frame_x = 0;
            c.frame_y = 0;
            c.set_paint(0);
            c.flags &= !crate::tile::FLAG_FULLBRIGHT;
            c.set_slope(0);
            c.flags &= !FLAG_HALF_BRICK;
        }
        world.set_invisible_block(x, y, false);
    }
    dye_place_2x2_752(world, x, y);
}

/// `WorldGen.TryPlacingChilletEgg(i, j)` (`WorldGen.cs:45700-45723`): the
/// same five-attempt ±10 jitter shape as the glow tulip, the shared
/// ground check + the ±20 egg scan, then PlaceTile(752); success = the
/// rolled cell ended active-752.
fn try_placing_chillet_egg(world: &mut World, rng: &mut Rng, i: i32, j: i32) -> bool {
    let (w, h) = (world.width, world.height);
    for _ in 0..5 {
        let x = rng.range((i - 10).max(10), (i + 10).min(w - 10));
        let y = rng.range((j - 10).max(10), (j + 10).min(h - 10));
        if !dye_valid_ground(world, x, y) || !no_nearby_chillet_eggs(world, x, y) {
            continue;
        }
        dye_place_tile_752(world, x, y);
        if world
            .tile(x, y)
            .is_some_and(|t| t.active() && t.kind == 752)
        {
            return true;
        }
    }
    false
}

/// `WorldGen.PlaceChilletEggs()` (`WorldGen.cs:45652-45678`): the target
/// 6/9/12 by `GetWorldSize` (`:6559-6570` — w ≤ 4200 → 0, ≤ 6400 → 1,
/// else 2), the 10000-attempt budget. Per attempt: x `Next(100, w-100)`,
/// y `[rockLayer, h-200)` (remix `h-350`).
fn place_chillet_eggs(world: &mut World, rng: &mut Rng, rock_layer: f64, remix: bool) {
    let (w, h) = (world.width, world.height);
    let target = if w > 6400 {
        12
    } else if w > 4200 {
        9
    } else {
        6
    };
    let mut placed = 0;
    let mut budget = 10000;
    while placed < target {
        budget -= 1;
        if budget <= 0 {
            break;
        }
        let x = rng.range(100, w - 100);
        let y_max = if remix { h - 350 } else { h - 200 };
        let y = rng.range(rock_layer as i32, y_max);
        if try_placing_chillet_egg(world, rng, x, y) {
            placed += 1;
        }
    }
}

/// The Dye Plants pass (`GenPassNameID.DyePlants`,
/// `WorldGen.cs:19981-19994`). The `w` plantDye attempts (the x/y rolls
/// interleave with the style rolls on the pass's own stream), then the
/// maturity sweep, the glow-tulip budget loop and the chillet-egg budget
/// loop — all in C# call order. Gated by `Skyblock.denyAllGeneration`
/// only. No progress message (the C# only `Set(1.0)`).
///
/// Sub-vanilla width/height guards (the C2/C3 convention): the pass's
/// `Next(100, w-100)` / `Next(100, UnderworldLayer)` rolls need
/// `w > 200` and `h > 300`; below that the C# throws and the port skips
/// the whole pass (no draws) — the 512×512 pin world clears both guards
/// but its plantDye margin (`j ≤ h-95 = 417` vs `j ≥ 100`) and the
/// `h-200` bands stay live, so the pin moves by design.
pub(crate) fn pass_dye_plants(
    world: &mut World,
    world_surface: f64,
    rock_layer: f64,
    rng: &mut Rng,
    opts: &GenOptions,
) {
    // `:19983`: !Skyblock.denyAllGeneration (= skyblockWorldGen, :3131).
    if opts.flags.contains(SeedFlags::SKYBLOCK) {
        return;
    }
    let (w, h) = (world.width, world.height);
    if w <= 200 || h <= 300 {
        return;
    }
    let remix = opts.flags.contains(SeedFlags::REMIX);
    // `:19986-19989`: the w plantDye attempts.
    for _ in 0..w {
        let i = rng.range(100, w - 100);
        let j = rng.range(100, h - 200);
        plant_dye(world, rng, i, j, world_surface, remix);
    }
    // `:19990-19992`: the three sweeps, in order.
    mature_the_herb_plants(world, rng, rock_layer);
    grow_glow_tulips(world, rng, world_surface, remix);
    place_chillet_eggs(world, rng, rock_layer, remix);
}

#[cfg(test)]
mod dye_tests {
    use super::*;
    use crate::tiledata::TILE_COUNT;

    /// A grass-2 surface row at `ground` with air above, dirt below.
    fn grass_world(width: i32, height: i32, ground: i32) -> World {
        let mut w = World::new("dye".into(), width, height);
        w.frame_important = vec![false; TILE_COUNT];
        for x in 0..width {
            for y in ground..height {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: if y == ground { 2 } else { 1 },
                    flags: FLAG_ACTIVE,
                    ..Default::default()
                };
            }
        }
        w
    }

    /// W4-g38 C8: `place_dye`'s style rules — the floor write (34*style
    /// frameX, flat), the wall-0 rule for styles 3/4, the style-5 full
    /// water rule, the style-7 ceiling arm and the flora-exempt refuse.
    #[test]
    fn w4_g38_c8_place_dye_style_rules() {
        let mut w = grass_world(400, 400, 200);
        // Style 0 on the grass surface: the write.
        place_dye(&mut w, 200, 199, 0);
        let t = w.tile(200, 199).unwrap();
        assert!(t.active());
        assert_eq!(t.kind, 227);
        assert_eq!(t.frame_x, 0);
        assert_eq!(t.frame_y, 0);
        // Style 3 refuses a wall behind the seat, accepts wall 0.
        let i = w.index(201, 199);
        w.tiles[i].wall = 2;
        place_dye(&mut w, 201, 199, 3);
        assert!(!w.tile(201, 199).unwrap().active());
        let i = w.index(201, 199);
        w.tiles[i].wall = 0;
        place_dye(&mut w, 201, 199, 3);
        assert_eq!(w.tile(201, 199).unwrap().frame_x, 102);
        // Style 5 needs full water; style 1 refuses any liquid.
        let i = w.index(202, 199);
        w.tiles[i].liquid = 255;
        place_dye(&mut w, 202, 199, 5);
        assert_eq!(w.tile(202, 199).unwrap().frame_x, 170);
        let i = w.index(203, 199);
        w.tiles[i].liquid = 100;
        place_dye(&mut w, 203, 199, 1);
        assert!(!w.tile(203, 199).unwrap().active());
        // Style 7 hangs under a solid dirt ceiling with air below.
        let i = w.index(204, 198);
        w.tiles[i] = Tile {
            kind: 1,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
        place_dye(&mut w, 204, 199, 7);
        assert_eq!(w.tile(204, 199).unwrap().frame_x, 238);
        // An active non-flora above cell refuses a floor style.
        let i = w.index(205, 198);
        w.tiles[i] = Tile {
            kind: 1,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
        place_dye(&mut w, 205, 199, 0);
        assert!(!w.tile(205, 199).unwrap().active());
    }

    /// W4-g38 C8: `mature_the_herb_plants` — the deep-mud plant writes the
    /// frameX-36 stage-1 herb, the stage rolls advance 82 → 83 → 84 in the
    /// C# conditional-draw order, and the above-rockLayer cells draw
    /// nothing.
    #[test]
    fn w4_g38_c8_mature_sweep_stages_and_draws() {
        let mut w = World::new("mature".into(), 60, 60);
        w.frame_important = vec![false; TILE_COUNT];
        // A mud floor at row 50 (below rockLayer 40), air above.
        for x in 0..60 {
            let i = w.index(x, 50);
            w.tiles[i] = Tile {
                kind: 59,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        // A pre-planted stage-0 herb (frameX 0) at (20, 30) — above the
        // rock layer: the plant arm skips it, the stage arm still rolls.
        let i = w.index(20, 30);
        w.tiles[i] = Tile {
            kind: 82,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
        let mut rng = Rng(9);
        mature_the_herb_plants(&mut w, &mut rng, 40.0);
        // Every deep air cell over the mud rolled Next(25); the hits are
        // 82/83/84 with frameX 36 (the stage rolls may advance them).
        let mut deep_herbs = 0;
        for x in 10..50 {
            let t = w.tile(x, 49).unwrap();
            if t.active() {
                assert!(matches!(t.kind, 82 | 83 | 84));
                assert_eq!(t.frame_x, 36);
                deep_herbs += 1;
            }
        }
        assert!(deep_herbs > 0, "the 1/25 plant lands on the 40-column band");
        // The pre-planted herb advanced past stage 0 or stayed (the
        // Next(3) roll) — but never carries a wrong kind.
        let t = w.tile(20, 30).unwrap();
        assert!(matches!(t.kind, 82 | 83 | 84));
    }

    /// W4-g38 C8: the pass plants dye on the surface band, the budget
    /// loops run without panic on a small live world, and the
    /// denyAllGeneration gate + the degenerate-size guard skip without
    /// draws.
    #[test]
    fn w4_g38_c8_pass_plants_and_gates() {
        // 400×400 clears the size guards; worldSurface 150 puts the
        // plantDye y band [100, 200) across the grass row 160.
        let mut w = grass_world(400, 400, 160);
        pass_dye_plants(&mut w, 150.0, 220.0, &mut Rng(4), &GenOptions::default());
        let dye = w.tiles.iter().filter(|t| t.active() && t.kind == 227).count();
        assert!(dye > 0, "the 400-wide world plants dye");
        // The skyblock gate: no tiles, no draws.
        let mut w = grass_world(400, 400, 160);
        let before = w.tiles.clone();
        let mut opts = GenOptions::default();
        opts.flags.insert(SeedFlags::SKYBLOCK);
        let mut rng = Rng(4);
        pass_dye_plants(&mut w, 150.0, 220.0, &mut rng, &opts);
        assert_eq!(w.tiles, before);
        assert_eq!(rng.next_u32(), Rng(4).next_u32(), "no draw");
        // The degenerate-size guard: a 120-wide world skips draw-free.
        let mut w = grass_world(120, 120, 60);
        let before = w.tiles.clone();
        let mut rng = Rng(4);
        pass_dye_plants(&mut w, 50.0, 70.0, &mut rng, &GenOptions::default());
        assert_eq!(w.tiles, before);
        assert_eq!(rng.next_u32(), Rng(4).next_u32(), "no draw");
    }
}

#[cfg(test)]
mod fallen_log_tests {
    use super::*;
    use crate::tiledata::TILE_COUNT;

    /// A flat dirt-2 ground row with air above — the fallen-log seat the
    /// C# scan finds (`:18637`'s three-column dirt test).
    fn seat_world() -> World {
        let mut w = World::new("log".into(), 64, 64);
        w.frame_important = vec![false; TILE_COUNT];
        w.frame_important[488] = true;
        for x in 0..64 {
            let i = w.index(x, 30);
            w.tiles[i] = Tile {
                kind: 2,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        w
    }

    /// W4-g38 C1: `Place3x2`'s type-488 path — the 3×2 frame write
    /// (`WorldGen.cs:52171-52194`): frameX = dx*18, frameY = dy*18, the
    /// floor `SolidTile2` under every column, the clear-body veto.
    #[test]
    fn w4_g38_c1_place_3x2_writes_the_log_frames_on_a_solid_floor() {
        let mut w = seat_world();
        place_3x2(&mut w, 30, 29, 488, 0);
        // The six cells: rows y-1/y, columns x-1..x+1.
        for (dx, dy) in [(-1i32, -1i32), (0, -1), (1, -1), (-1, 0), (0, 0), (1, 0)] {
            let t = w.tile(30 + dx, 29 + dy).unwrap();
            assert!(t.active(), "cell ({dx},{dy})");
            assert_eq!(t.kind, 488);
            assert_eq!(t.frame_x, ((dx + 1) * 18) as i16);
            assert_eq!(t.frame_y, ((dy + 1) * 18) as i16);
        }
        // An occupied body cell vetoes the whole write.
        let mut w2 = seat_world();
        let i = w2.index(30, 28);
        w2.tiles[i] = Tile { kind: 1, flags: FLAG_ACTIVE, ..Default::default() };
        place_3x2(&mut w2, 30, 29, 488, 0);
        assert!(!w2.tile(30, 29).unwrap().active());
        // A missing floor column vetoes too (`SolidTile2`, `:52118`).
        let mut w3 = seat_world();
        let i = w3.index(31, 30);
        w3.tiles[i] = Tile::default();
        place_3x2(&mut w3, 30, 29, 488, 0);
        assert!(!w3.tile(30, 29).unwrap().active());
        // The 5-tile bounds guard (`:52068`).
        let mut w4 = seat_world();
        place_3x2(&mut w4, 3, 29, 488, 0);
        assert!(!w4.tile(3, 29).unwrap().active());
    }

    /// W4-g38 C1: `PlaceTile(488)`'s gen-time slice — the existing-log veto
    /// (`WorldGen.cs:59345`) and the success flag (`:60255-60262`).
    #[test]
    fn w4_g38_c1_place_tile_fallen_log_vetoes_an_existing_log() {
        let mut w = seat_world();
        assert!(place_tile_fallen_log(&mut w, 30, 29));
        assert!(w.tile(30, 29).is_some_and(|t| t.active() && t.kind == 488));
        // A second placement on the same centre is refused, world unchanged.
        let before = w.tiles.clone();
        assert!(!place_tile_fallen_log(&mut w, 30, 29));
        assert_eq!(w.tiles, before);
    }

    /// W4-g38 C1: the pass gates — skyblock's `denySomeGeneration`
    /// (`WorldGen.cs:3133-3147`) and the noSurface skip (`:18522`); the
    /// sub-vanilla width guard draws only the count roll.
    #[test]
    fn w4_g38_c1_the_pass_gates_skip_without_tile_writes() {
        let layers = Layers {
            avg_surface: 30.0,
            world_surface: 30.0,
            rock_layer: 50.0,
            hell_top: 60,
            surface_low: 30.0,
            surface_high: 30.0,
            rock_low: 46.0,
            rock_high: 52.0,
        };
        // Plain skyblock denies; the extra-living-trees micro seed admits.
        let mut sky = GenOptions::default();
        sky.flags.insert(SeedFlags::SKYBLOCK);
        assert!(super::super::skyblock::deny_some_generation(sky.flags, sky.micro_flags));
        let mut w = seat_world();
        let before = w.tiles.clone();
        let mut anchor = (-1, -1);
        // The denied pass draws nothing at all (`:18515` outer gate).
        let mut rng_denied = Rng(7);
        place_fallen_log(&mut w, &layers, &mut rng_denied, &sky, &mut anchor);
        assert_eq!(w.tiles, before);
        assert_eq!(anchor, (-1, -1));
        assert_eq!(rng_denied.next_u32(), Rng(7).next_u32(), "no draw");
        sky.micro_flags.insert(MicroSeedFlags::EXTRA_LIVING_TREES);
        assert!(!super::super::skyblock::deny_some_generation(sky.flags, sky.micro_flags));
        // The admitted skyblock world enters, but the 64-wide band guard
        // stops it after the count roll (see the classic arm below).
        place_fallen_log(&mut w, &layers, &mut Rng(7), &sky, &mut anchor);
        assert_eq!(w.tiles, before);
        // noSurface skips the log loop (`:18522`).
        let mut nos = GenOptions::default();
        nos.micro_flags.insert(MicroSeedFlags::NO_SURFACE);
        place_fallen_log(&mut w, &layers, &mut Rng(7), &nos, &mut anchor);
        assert_eq!(w.tiles, before);
        // The 64-wide world collapses the x band (margin 400): the count
        // roll runs, the scan draws nothing, no tile moves.
        let classic = GenOptions::default();
        let mut rng_a = Rng(7);
        let mut rng_b = Rng(7);
        let _count_roll = rng_a.range(-1, 2);
        place_fallen_log(&mut w, &layers, &mut rng_b, &classic, &mut anchor);
        assert_eq!(w.tiles, before);
        assert_eq!(rng_a.next_u32(), rng_b.next_u32(), "only the count roll drew");
    }
}


/// The `Spreading Grass` pass's classic arm
/// (`WorldGen.cs:18237-18351`, W4-d104 pass 31 — the full pass name is
/// `SpreadingGrassOnSurfaceSunflowersEvilsOnSurfaceAndLavaCleanup`): the
/// two surface sweeps. The remix arm (`:18352-18443` — the surface-layer
/// grass, the sunflowers, the evil Convert band, the hive/lava cleanup)
/// is the remix-seed lane's (d107). The classic arm draws NO RNG — the
/// step still takes its own `pass_rng("Spreading Grass")` stream slot so
/// the remix arm lands without a ledger shift.
///
/// Sweep 1 (`:18242-18313`): over the ±50-margin columns down to
/// `worldSurface`, the jungle-grass 60 spreads to adjacent dirt (grass
/// over open air, mud under a cover), and the stone/snow/ore cells take
/// the ±3-box's biome flavour (the sand 53 latch, the 59/60/147/161/
/// 199/23 picks, the open-air-above `flag` gate, the evil cover veto and
/// the jungle-band grass/mud split).
///
/// Sweep 2 (`:18314-18351`): over the ±10-margin columns, the FIRST
/// surface dirt cell (the `flag2` open-sky latch, reset by any wall-less
/// air cell) takes the LIVE `SpreadGrass(num3, num4)` call (`:18334`) —
/// the defaults `dirt: 0, grass: 2, repeat: true` (`:75146`), so the
/// bounded 3×3 recursion fires. The `repeat: false` call at `:18339` is
/// ONLY the unreachable `catch` fallback (SpreadGrass carries its own
/// try/catch `:75148/:75254` and never throws) — research/86 §6.1. The
/// column scan breaks past `worldSurfaceHigh`.
pub(crate) fn pass_spreading_grass(
    world: &mut World,


// --- Surface Ore and Stone (W4-d104 pass 32) --------------------------------

/// `WorldGen.SolidTile(i, j)` (`WorldGen.cs:70111-70132`) — the patch
/// scans' floor predicate; a null tile reads solid.
fn solid_tile_patch(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && crate::tiledata::TILE_SOLID.has(t.kind)
        && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}


/// `Main.tileDungeon` (`Main.cs:7992-7997`) — the patch scans' dungeon
/// veto.
fn tile_dungeon(kind: u16) -> bool {
    matches!(kind, 41 | 43 | 44 | 677 | 678 | 679)
}


// --- Place Fallen Log (W4-d104 pass 33) --------------------------------------

/// `Skyblock.denySomeGeneration` (`WorldGen.cs:3133-3147`): a skyblock
/// world denies the pass unless one of the surface-shape seeds is on
/// (then it allows), else `!extraLivingTrees`.
pub(crate) fn deny_some_generation(world: &World) -> bool {
    if !world.seeds.contains(crate::seeds::SeedFlags::SKYBLOCK) {
        return false;
    }
    let m = world.micro_seeds;
    let surface_shape = m.contains(crate::seeds::MicroSeedFlags::WORLD_IS_FROZEN)
        || m.contains(crate::seeds::MicroSeedFlags::SURFACE_IS_DESERT)
        || m.contains(crate::seeds::MicroSeedFlags::SURFACE_IS_MUSHROOMS)
        || m.contains(crate::seeds::MicroSeedFlags::WORLD_IS_INFECTED)
        || m.contains(crate::seeds::MicroSeedFlags::HALLOW_ON_THE_SURFACE)
        || m.contains(crate::seeds::MicroSeedFlags::NO_INFECTION)
        || m.contains(crate::seeds::MicroSeedFlags::EXTRA_FLOATING_ISLANDS)
        || m.contains(crate::seeds::MicroSeedFlags::EXTRA_LIQUID);
    if surface_shape {
        return false;
    }
    !m.contains(crate::seeds::MicroSeedFlags::EXTRA_LIVING_TREES)
}


/// `WorldGen.OreHelper` (`WorldGen.cs:10228-10240`): the ±1 box's stone
/// 1 / snow 40 cells retype to dirt around every ore write.
fn ore_helper(world: &mut World, x: i32, y: i32) {
    for i in (x - 1)..=(x + 1) {
        for j in (y - 1)..=(y + 1) {
            if let Some(t) = world.tile_mut(i, j) {
                if t.kind == 1 || t.kind == 40 {
                    t.kind = 0;
                }
            }
        }
    }
}


/// `WorldGen.OrePatch` (`WorldGen.cs:10579-10705`): the surface ore vein
/// — the copper/iron tier roll, the drop scan, the grass-triplet + wall
/// vetoes, the ±10×+7..+30 body scan, the `Next(2)` sink, the seed write,
/// the `Next(8, 13)`-bounded vein walk (the per-step x/y drift + the
/// 1-in-3 extra sink + the 1-in-4 side pocket), then the drifting-blob
/// carve/ore pass (`num7` rounds of the ±4·num6 box, the per-cell
/// `num8`/`num9` radii with the 1-in-8 double).
fn ore_patch(
    world: &mut World,


/// `WorldGen.StonePatch` (`WorldGen.cs:10242-10324`): the surface stone
/// blob — the same drop/grass/wall/body gates (the body scan adds the
/// 199/23 evil veto), then the drifting-blob retype (the solid cells →
/// stone 1) with the pile decor on the stone-floor air cells (the 186
/// large pile `Next(7, 13)` style + the two 185 small piles).
fn stone_patch(world: &mut World, x: i32, y: i32, world_surface: f64, rng: &mut Rng) -> bool {
    let mut num = y;
    while !solid_tile_patch(world, x, num) {
        num += 1;
        if num as f64 > world_surface {
            return false;
        }
    }
    let grass_at = |world: &World, px: i32| {
        world
            .tile(px, num)
            .is_some_and(|t| crate::tiledata::SET_GRASS.has(t.kind))
    };
    if !grass_at(world, x) || !grass_at(world, x - 1) || !grass_at(world, x + 1) {
        return false;
    }
    if world.tile(x, num).map(|t| t.wall).unwrap_or(0) > 0 {
        return false;
    }
    for i in (x - 10)..=(x + 10) {
        for j in (num + 7)..=(num + 30) {
            let bad = world.tile(i, j).is_none_or(|t| {
                !t.active()
                    || tile_dungeon(t.kind)
                    || crate::tiledata::SET_CLOUDS.has(t.kind)
                    || crate::tiledata::SET_CONVERSION_SAND.has(t.kind)
                    || t.wall == 0
                    || t.kind == 199
                    || t.kind == 23
            });
            if bad {
                return false;
            }
        }
    }
    let (mut cx, mut cy) = (x as f64, num as f64);
    let mut vel_x = rng.next_f64() * 0.6 - 0.3;
    let mut vel_y = rng.next_f64() * 0.5 + 0.5;
    let mut num2 = rng.range(13, 18) as f64;
    let mut num3 = rng.range(13, 19);
    if rng.range(0, 3) == 0 {
        num2 += rng.range(0, 3) as f64;
    }
    if rng.range(0, 3) == 0 {
        num3 += rng.range(0, 3);
    }
    while num3 > 0 {
        num3 -= 1;
        let reach = (num2 as i32) * 4;
        for k in (x - reach)..=(x + reach) {
            for l in (num - reach)..=(num + reach) {
                let mut num4 = num2 * (0.7 + rng.next_f64() * 0.6) * 0.3;
                if rng.range(0, 8) == 0 {
                    num4 *= 2.0;
                }
                let (dx, dy) = (cx - k as f64, cy - l as f64);
                let dist = (dx * dx + dy * dy).sqrt();
                // The pile decor (`:10294-10306`): the air cell over a
                // stone floor with solid flanks.
                let pile_seat = dist < num4 * 2.0
                    && !world.tile(k, l).is_some_and(|t| t.active())
                    && world.tile(k, l + 1).is_some_and(|t| t.active() && t.kind == 1)
                    && rng.range(0, 7) == 0
                    && solid_tile_patch(world, k - 1, l + 1)
                    && solid_tile_patch(world, k + 1, l + 1);
                if pile_seat {
                    if rng.range(0, 3) != 0 {
                        // `PlaceTile(k, l, 186, style: Next(7, 13))` —
                        // the `Place3x2` route (`:59993-60002`).
                        super::evil::place_3x2(world, k, l, 186, rng.range(7, 13));
                    }
                    if rng.range(0, 3) != 0 {
                        super::spider_gem_moss::place_small_pile(
                            world, k, l, rng.range(0, 6), 1,
                        );
                    }
                    super::spider_gem_moss::place_small_pile(
                        world, k, l, rng.range(0, 6), 0,
                    );
                }
                if dist < num4 {
                    if let Some(t) = world.tile_mut(k, l) {
                        if crate::tiledata::TILE_SOLID.has(t.kind) {
                            t.kind = 1;
                        }
                    }
                }
            }
        }
        cx += vel_x;
        cy += vel_y;
        vel_x += rng.next_f64() * 0.2 - 0.1;
        vel_y += rng.next_f64() * 0.2 - 0.1;
        vel_x = vel_x.clamp(-0.3, 0.3);
        vel_y = vel_y.clamp(0.5, 1.0);
    }
    true
}


/// The `Surface Ore and Stone` pass (`WorldGen.cs:18447-18511`, W4-d104
/// pass 32): the ore-patch scatter (`Next(width*5/4200, width*10/4200)`
/// attempts, each a `width/420`-budget retry over the beach-excluded,
/// centre-excluded [0.48, 0.52] column band, the `worldSurfaceLow..
/// worldSurface` row, the ±200 spacing from the recorded ore patches)
/// then the stone-patch scatter (`Next(1, width*7/4200)` attempts, the
/// [0.47, 0.53] centre band, the ±100 spacing). The `GenVars.orePatchX/
/// numOrePatch` ledger (`GenVars.cs:240-244`, `maxOrePatch = 50` `:461`)
/// rides the caller's slice — the record only fires below `maxOrePatch -
/// 1` (`:18475`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn pass_surface_ore_and_stone(
    world: &mut World,
