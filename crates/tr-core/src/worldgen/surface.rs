//! Surface dressing: slope tiles, trees, small flora and desert cacti.

use super::noise::Rng;
use super::terrain::SurfaceMap;
use super::{TILE_GRASS, TILE_SAND, TILE_SNOW};
use crate::tile::{FLAG_ACTIVE, Tile};
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
/// air cell) takes `SpreadGrass(num3, num4, 0, 2, repeat: false)`; the
/// column scan breaks past `worldSurfaceHigh`.
pub(crate) fn pass_spreading_grass(
    world: &mut World,
    world_surface: f64,
    surface_high: f64,
    jungle_min_x: i32,
    jungle_max_x: i32,
) {
    // --- sweep 1: the biome retypes (`:18242-18313`) -----------------------
    for i in 50..(world.width - 50) {
        let mut j = 50;
        while (j as f64) <= world_surface {
            if world.tile(i, j).is_some_and(|t| t.active()) {
                let kind = world.tile(i, j).map(|t| t.kind).unwrap_or(0);
                if kind == 60 {
                    // The jungle-grass spread (`:18249-18265`).
                    for k in (i - 1)..=(i + 1) {
                        for l in (j - 1)..=(j + 1) {
                            if world.tile(k, l).is_some_and(|t| t.active() && t.kind == 0) {
                                let covered = world.tile(k, l - 1).is_some_and(|t| t.active());
                                if let Some(t) = world.tile_mut(k, l) {
                                    t.kind = if covered { 59 } else { 60 };
                                }
                            }
                        }
                    }
                } else if kind == 1 || kind == 40 || crate::tiledata::SET_ORE.has(kind) {
                    // The ±3 flavour scan (`:18266-18289`).
                    let num = 3;
                    let mut flag = false;
                    let mut num2 = 0u16;
                    for m in (i - num)..=(i + num) {
                        for n in (j - num)..=(j + num) {
                            if let Some(t) = world.tile(m, n) {
                                if t.active() {
                                    if t.kind == 53 || num2 == 53 {
                                        num2 = 53;
                                    } else if matches!(t.kind, 59 | 60 | 147 | 161 | 199 | 23) {
                                        num2 = t.kind;
                                    }
                                } else if n < j && t.wall == 0 {
                                    flag = true;
                                }
                            }
                        }
                    }
                    if flag {
                        // The cover vetoes (`:18292-18307`).
                        match num2 {
                            23 | 199 => {
                                if world.tile(i, j - 1).is_some_and(|t| t.active()) {
                                    num2 = 0;
                                }
                            }
                            59 | 60 => {
                                if i >= jungle_min_x && i <= jungle_max_x {
                                    let covered =
                                        world.tile(i, j - 1).is_some_and(|t| t.active());
                                    num2 = if covered { 59 } else { 60 };
                                }
                            }
                            _ => {}
                        }
                        if let Some(t) = world.tile_mut(i, j) {
                            t.kind = num2;
                        }
                    }
                }
            }
            j += 1;
        }
    }
    // --- sweep 2: the first-surface grass (`:18314-18351`) -----------------
    for num3 in 10..(world.width - 10) {
        let mut flag2 = true;
        let mut num4 = 0;
        while (num4 as f64) < world_surface - 1.0 {
            if world.tile(num3, num4).is_some_and(|t| t.active()) {
                if flag2 && world.tile(num3, num4).is_some_and(|t| t.kind == 0) {
                    // `grassSpread = 0; SpreadGrass(num3, num4, 0, 2,
                    // repeat: false)` (`:18334-18338`).
                    let mut grass_spread = 0i32;
                    super::evil::spread_evil_grass(
                        world,
                        num3,
                        num4,
                        0,
                        2,
                        world_surface,
                        &mut grass_spread,
                        false,
                    );
                }
                if (num4 as f64) > surface_high {
                    break;
                }
                flag2 = false;
            } else if world.tile(num3, num4).map(|t| t.wall).unwrap_or(0) == 0 {
                flag2 = true;
            }
            num4 += 1;
        }
    }
}

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
    x: i32,
    y: i32,
    world_surface: f64,
    copper: u16,
    iron: u16,
    rng: &mut Rng,
) -> bool {
    // `:10581-10585`: the tier roll.
    let kind = if rng.range(0, 3) == 0 { iron } else { copper };
    let mut num = x;
    let mut num2 = y;
    // `:10588-10595`: the drop scan.
    while !solid_tile_patch(world, num, num2) {
        num2 += 1;
        if num2 as f64 > world_surface {
            return false;
        }
    }
    // `:10596-10603`: the grass triplet + the wall veto.
    let grass_at = |world: &World, px: i32| {
        world
            .tile(px, num2)
            .is_some_and(|t| crate::tiledata::SET_GRASS.has(t.kind))
    };
    if !grass_at(world, num) || !grass_at(world, num - 1) || !grass_at(world, num + 1) {
        return false;
    }
    if world.tile(num, num2).map(|t| t.wall).unwrap_or(0) > 0 {
        return false;
    }
    // `:10604-10612`: the body scan.
    for i in (num - 10)..=(num + 10) {
        for j in (num2 + 7)..=(num2 + 30) {
            let bad = world.tile(i, j).is_none_or(|t| {
                !t.active()
                    || tile_dungeon(t.kind)
                    || crate::tiledata::SET_CLOUDS.has(t.kind)
                    || crate::tiledata::SET_CONVERSION_SAND.has(t.kind)
                    || t.wall == 0
            });
            if bad {
                return false;
            }
        }
    }
    // `:10613-10616`: the sink + the seed write.
    num2 += rng.range(0, 2);
    if let Some(t) = world.tile_mut(num, num2) {
        t.kind = kind;
        t.flags |= FLAG_ACTIVE;
    }
    ore_helper(world, num, num2);
    // `:10621-10649`: the vein walk — the bound re-rolls EVERY check.
    let num3 = num2;
    while num2 < num3 + rng.range(8, 13) {
        num += rng.range(-1, 2);
        num2 += rng.range(1, 3);
        if rng.range(0, 3) == 0 {
            num2 += 1;
        }
        if let Some(t) = world.tile_mut(num, num2) {
            t.kind = kind;
            t.flags |= FLAG_ACTIVE;
        }
        ore_helper(world, num, num2);
        if rng.range(0, 4) == 0 {
            let num4 = num + rng.range(-2, 3);
            let num5 = num2 + rng.range(0, 2);
            if let Some(t) = world.tile_mut(num4, num5) {
                t.kind = kind;
                t.flags |= FLAG_ACTIVE;
            }
            ore_helper(world, num4, num5);
        }
    }
    // `:10650-10703`: the drifting blob.
    let (mut cx, mut cy) = (num as f64, num2 as f64);
    let mut vel_x = rng.next_f64() * 0.6 - 0.3;
    let mut vel_y = rng.next_f64() * 0.5 + 0.5;
    let mut num6 = rng.range(5, 9) as f64;
    let mut num7 = rng.range(9, 14);
    if rng.range(0, 3) == 0 {
        num6 += rng.range(0, 2) as f64;
    }
    if rng.range(0, 3) == 0 {
        num7 += rng.range(0, 2);
    }
    while num7 > 0 {
        num7 -= 1;
        let reach = (num6 as i32) * 4;
        for k in (num - reach)..=(num + reach) {
            for l in (num2 - reach)..=(num2 + reach) {
                let num8 = num6 * (0.5 + rng.next_f64() * 0.5) * 0.1;
                let mut num9 = num6 * (0.7 + rng.next_f64() * 0.6) * 0.3;
                if rng.range(0, 8) == 0 {
                    num9 *= 2.0;
                }
                let (dx, dy) = (cx - k as f64, cy - l as f64);
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < num8 {
                    if let Some(t) = world.tile_mut(k, l) {
                        t.flags &= !FLAG_ACTIVE;
                    }
                } else if dist < num9 {
                    if let Some(t) = world.tile_mut(k, l) {
                        t.kind = kind;
                        if rng.range(0, 4) == 0 {
                            t.flags |= FLAG_ACTIVE;
                        }
                    }
                    ore_helper(world, k, l);
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
                            world, k, l, rng.range(0, 6), 1, 185,
                        );
                    }
                    super::spider_gem_moss::place_small_pile(
                        world, k, l, rng.range(0, 6), 0, 185,
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
    surface_low: f64,
    world_surface: f64,
    copper: u16,
    iron: u16,
    ore_patch_x: &mut Vec<i32>,
    rng: &mut Rng,
) {
    // The sub-vanilla width guard: the stone scatter's count draw
    // `Next(1, width*7/4200)` throws on a ≤600-wide world (the C# never
    // generates one), and the ore scatter's beach band
    // `Next(380, width - 380)` collapses there too — the port skips the
    // whole pass before any draw (the evil pass's width-gate precedent).
    if world.width * 7 / 4200 <= 1 {
        return;
    }
    // --- the ore patches (`:18452-18483`) ----------------------------------
    let num = rng.range(world.width * 5 / 4200, world.width * 10 / 4200);
    for _ in 0..num {
        let mut num2 = world.width / 420;
        while num2 > 0 {
            num2 -= 1;
            let mut num3 = rng.range(380, world.width - 380);
            while (num3 as f64) >= world.width as f64 * 0.48
                && (num3 as f64) <= world.width as f64 * 0.52
            {
                num3 = rng.range(380, world.width - 380);
            }
            let y = rng.range(surface_low as i32, world_surface as i32);
            let flag = ore_patch_x.iter().any(|px| (num3 - px).abs() < 200);
            if !flag && ore_patch(world, num3, y, world_surface, copper, iron, rng) {
                if ore_patch_x.len() < 50 - 1 {
                    ore_patch_x.push(num3);
                }
                break;
            }
        }
    }
    // --- the stone patches (`:18484-18509`) --------------------------------
    let num = rng.range(1, world.width * 7 / 4200);
    for _ in 0..num {
        let mut num4 = world.width / 420;
        while num4 > 0 {
            num4 -= 1;
            let mut num5 = rng.range(380, world.width - 380);
            while (num5 as f64) >= world.width as f64 * 0.47
                && (num5 as f64) <= world.width as f64 * 0.53
            {
                num5 = rng.range(380, world.width - 380);
            }
            let y2 = rng.range(surface_low as i32, world_surface as i32);
            let flag2 = ore_patch_x.iter().any(|px| (num5 - px).abs() < 100);
            if !flag2 && stone_patch(world, num5, y2, world_surface, rng) {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// W4-d104 pass 31: the first-surface dirt takes grass, the covered
    /// dirt stays, and two runs are byte-equal (a == b — the classic arm
    /// draws no RNG).
    #[test]
    fn w4_d104_31_the_surface_dirt_takes_grass() {
        let build = || {
            let mut w = World::new("sg".into(), 300, 200);
            // A dirt column at x=150: the surface cell at row 60 (open
            // sky above), a buried dirt cell at row 70 under a cover.
            for y in 60..200 {
                let i = w.index(150, y);
                w.tiles[i].kind = 0;
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
            w
        };
        let mut a = build();
        let mut b = build();
        pass_spreading_grass(&mut a, 100.0, 90.0, -1, -1);
        pass_spreading_grass(&mut b, 100.0, 90.0, -1, -1);
        assert_eq!(a.tiles, b.tiles, "a == b: the classic arm replays");
        assert_eq!(a.tile(150, 60).unwrap().kind, 2, "the first dirt takes grass");
        assert_eq!(a.tile(150, 70).unwrap().kind, 0, "the buried dirt stays");
    }

    /// W4-d104 pass 31: the ±3 flavour scan retypes an exposed stone to
    /// the neighbouring snow (147), and the sand latch wins over it.
    #[test]
    fn w4_d104_31_the_stone_takes_the_biome_flavour() {
        let mut w = World::new("sg".into(), 300, 200);
        // An exposed stone at (150, 60) with snow two cells away.
        let i = w.index(150, 60);
        w.tiles[i].kind = 1;
        w.tiles[i].flags |= FLAG_ACTIVE;
        let i = w.index(152, 60);
        w.tiles[i].kind = 147;
        w.tiles[i].flags |= FLAG_ACTIVE;
        pass_spreading_grass(&mut w, 100.0, 90.0, -1, -1);
        assert_eq!(w.tile(150, 60).unwrap().kind, 147, "the snow flavour wins");
        // The sand latch: a sand cell in the box overrides the snow.
        let i = w.index(148, 60);
        w.tiles[i].kind = 53;
        w.tiles[i].flags |= FLAG_ACTIVE;
        let i = w.index(150, 60);
        w.tiles[i].kind = 1;
        pass_spreading_grass(&mut w, 100.0, 90.0, -1, -1);
        assert_eq!(w.tile(150, 60).unwrap().kind, 53, "the sand latch wins");
    }

    /// W4-d104 pass 32: the ore patch drops a vein into the grass floor
    /// and the stone patch retypes the blob — two same-stream runs replay
    /// byte-equal (a == b).
    #[test]
    fn w4_d104_32_the_patches_place_and_replay() {
        let build = || {
            let mut w = World::new("so".into(), 4200, 1200);
            // A grass surface at row 300 with the dirt wall body below
            // (the patch's ±10×+7..+30 scan needs walls).
            for x in 0..w.width {
                for y in 300..1200 {
                    let i = w.index(x, y);
                    w.tiles[i].kind = if y == 300 { 2 } else { 0 };
                    w.tiles[i].flags |= FLAG_ACTIVE;
                    if y > 302 {
                        w.tiles[i].wall = 2;
                    }
                }
            }
            w.world_surface = 325.0;
            w
        };
        let mut a = build();
        let mut b = build();
        let mut pa = Vec::new();
        let mut pb = Vec::new();
        pass_surface_ore_and_stone(&mut a, 300.0, 325.0, 7, 166, &mut pa, &mut crate::worldgen::noise::Rng(42));
        pass_surface_ore_and_stone(&mut b, 300.0, 325.0, 7, 166, &mut pb, &mut crate::worldgen::noise::Rng(42));
        assert_eq!(a.tiles, b.tiles, "a == b: the pass stream replays");
        assert_eq!(pa, pb, "the ore-patch ledger replays");
        assert!(
            a.tiles.iter().any(|t| t.active() && (t.kind == 7 || t.kind == 166)),
            "an ore vein landed"
        );
    }
}
