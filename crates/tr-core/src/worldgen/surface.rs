//! Surface dressing: slope tiles, trees, small flora and desert cacti.

use std::collections::HashSet;

use super::noise::Rng;
use super::terrain::SurfaceMap;
use super::{GenOptions, TILE_DIRT, TILE_GRASS, TILE_SAND, TILE_SNOW};
use crate::seeds::{MicroSeedFlags, SeedFlags};
use crate::tile::{FLAG_ACTIVE, LiquidKind, Tile};
use crate::tiledata::{SET_CAN_BE_CLEARED_DURING_GENERATION, TILE_SOLID};
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
