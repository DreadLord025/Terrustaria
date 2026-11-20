//! The `Cave Walls` pass (`GenPassNameID.CaveWallsInEnclosedSpaces`,
//! `WorldGen.cs:17711-17839`): the enclosed-pocket wall retexture — the
//! mushroom/snow/lava/cave wall families over the counted pockets
//! (`countTiles` `:9546-9635`), the remix dirt/grass second sweep, and the
//! jungle-wall → mud-wall `Spread.Wall2` conversion. Plus `Spread.Wall`
//! (`:3356-3414`) / `Spread.Wall2` (`:3416-3524`).
//!
//! Deviations: the pass-head `Main.tileSolid[162] = false` /
//! `[226] = true` / `[232] = false` toggles (`:17713-17715`) are modelled
//! as a local solid predicate for the pass's own reads (the
//! `liquids.rs::solid_waterfalls` window precedent — the vanilla toggles
//! persist globally, the port's boot table stands for later passes);
//! `countTiles`'s `wall != 0` bail reads the port's baseline-wall
//! adaptation (`spider_gem_moss.rs::has_real_wall` — the terrain pre-lays
//! dirt/stone/sandstone walls under every solid row); `Spread.Wall`'s
//! `tile.wall != 0` / `== 0` tests ride the same adaptation. The
//! `dualDungeons` desert-loot state is untouched here (no chest draws).

use std::collections::HashSet;

use super::noise::Rng;
use crate::tile::LiquidKind;
use crate::tiledata::{TILE_SOLID, TILE_SOLID_TOP};
use crate::world::World;

/// The walls the port's terrain pre-lays (`spider_gem_moss.rs`'s
/// `BASELINE_WALLS` slice): stone 1, dirt 2, sandstone 187, hardened 216.
const BASELINE_WALLS: [u16; 4] = [1, 2, 187, 216];

/// `WallID.Sets.CannotBeReplacedByWallSpread` (`WallID.cs:98`):
/// {4, 40, 3, 83, 87, 244, 34}.
const CANNOT_BE_REPLACED: [u16; 7] = [4, 40, 3, 83, 87, 244, 34];

/// `WallID.Sets.WallSpreadStopsAtAir` (`WallID.cs:99`): {63, 62}.
const STOPS_AT_AIR: [u16; 2] = [63, 62];

/// `WorldGen.maxWallOut2` (`WorldGen.cs:4250`, static ctor `:86922`: 5000).
const MAX_WALL_OUT2: i32 = 5000;

/// The pass-head solid window (`:17713-17715`): the breakable ice 162 and
/// the wooden spikes 232 read non-solid, the lihzahrd brick 226 solid.
fn solid_cave_walls(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if !t.active() || t.inactive() || t.half_brick() || t.slope() != 0 {
        return false;
    }
    match t.kind {
        162 | 232 => false,
        226 => true,
        _ => TILE_SOLID.has(t.kind) && !TILE_SOLID_TOP.has(t.kind),
    }
}

/// The baseline-wall adaptation (`spider_gem_moss.rs::has_real_wall`).
fn has_real_wall(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    !BASELINE_WALLS.contains(&t.wall)
}

/// `WorldGen.countTiles`/`nextCount` (`WorldGen.cs:9546-9635`) with the
/// `jungle` flag (the third sweep reads `jungle: true` — the wall/lava
/// bails are skipped). The explicit-stack walk is the
/// `spider_gem_moss.rs::count_tiles` precedent (the fill carries no RNG).
#[derive(Default, Clone, Copy)]
struct CaveCounts {
    tiles: i32,
    shroom: i32,
    lava: i32,
    ice: i32,
    sand: i32,
    rock: i32,
}

fn count_tiles(world: &World, x: i32, y: i32, jungle: bool, lava_ok: bool, max: i32) -> CaveCounts {
    let mut c = CaveCounts::default();
    let mut seen: HashSet<(i32, i32)> = HashSet::new();
    let mut stack = vec![(x, y)];
    while let Some((x, y)) = stack.pop() {
        if c.tiles >= max {
            return c;
        }
        if x <= 1 || y <= 1 || x >= world.width - 1 || y >= world.height - 1 {
            c.tiles = max;
            return c;
        }
        if !seen.insert((x, y)) {
            continue;
        }
        let Some(t) = world.tile(x, y) else {
            continue;
        };
        // `:9575-9584` — the living-wood wall and shimmer bails.
        if t.wall == 244 {
            c.tiles = max;
            return c;
        }
        if t.liquid > 0 && t.liquid_kind() == LiquidKind::Shimmer {
            c.tiles = max;
            return c;
        }
        // `:9585-9605` — the non-jungle wall/lava bails.
        if !jungle {
            if has_real_wall(world, x, y) {
                c.tiles = max;
                return c;
            }
            let lava_here = t.liquid > 0 && t.liquid_kind() == LiquidKind::Lava;
            if lava_here {
                c.lava += 1;
                if !lava_ok {
                    c.tiles = max;
                    return c;
                }
            }
        }
        // `:9606-9624` — the kind counters.
        if t.active() {
            match t.kind {
                70 => c.shroom += 1,
                1 => c.rock += 1,
                147 | 161 => c.ice += 1,
                53 | 396 | 397 => c.sand += 1,
                _ => {}
            }
        }
        // `:9625-9633` — the open-cell expansion.
        if !solid_cave_walls(world, x, y) {
            c.tiles += 1;
            stack.push((x - 1, y));
            stack.push((x + 1, y));
            stack.push((x, y - 1));
            stack.push((x, y + 1));
        }
    }
    c
}

/// `Spread.Wall` (`WorldGen.cs:3356-3414`): the wave-front BFS — the
/// solid/walled cells take the wall only when active and unwalled (no
/// expansion), the open cells take it unconditionally and expand to the
/// four neighbours. The duplicate-tolerant pop is verbatim (the C# does
/// not re-check the hashSet on pop). The wall tests ride the baseline
/// adaptation (the module note).
fn spread_wall(world: &mut World, x: i32, y: i32, wall: u16) {
    if x < 0 || y < 0 || x >= world.width || y >= world.height {
        return;
    }
    let mut seen: HashSet<(i32, i32)> = HashSet::new();
    let mut next: Vec<(i32, i32)> = vec![(x, y)];
    while !next.is_empty() {
        let mut list = std::mem::take(&mut next);
        while !list.is_empty() {
            let item = list.remove(0);
            let (px, py) = item;
            if px < 1 || py < 1 || px >= world.width - 1 || py >= world.height - 1 {
                continue;
            }
            seen.insert(item);
            let solid = solid_cave_walls(world, px, py);
            let walled = has_real_wall(world, px, py);
            if solid || walled {
                // `:3385-3389` — active + unwalled takes the wall.
                if world.tile(px, py).is_some_and(|t| t.active()) && !walled {
                    if let Some(t) = world.tile_mut(px, py) {
                        t.wall = wall;
                    }
                }
                continue;
            }
            if let Some(t) = world.tile_mut(px, py) {
                t.wall = wall;
            }
            for n in [(px - 1, py), (px + 1, py), (px, py - 1), (px, py + 1)] {
                if !seen.contains(&n) {
                    next.push(n);
                }
            }
        }
    }
}

/// `Spread.Wall2` (`WorldGen.cs:3416-3524`): the budgeted conversion —
/// the same-wall / cannot-be-replaced cells skip, the open cells count
/// against `maxWallOut2` (5000) and expand (the `WallSpreadStopsAtAir`
/// walls also stop at air and expand diagonally +2), the solid active
/// cells take the wall without expanding.
fn spread_wall2(world: &mut World, x: i32, y: i32, wall: u16) {
    if x < 0 || y < 0 || x >= world.width || y >= world.height {
        return;
    }
    let mut num2 = 0;
    let stops_at_air = STOPS_AT_AIR.contains(&wall);
    let mut seen: HashSet<(i32, i32)> = HashSet::new();
    let mut next: Vec<(i32, i32)> = vec![(x, y)];
    while !next.is_empty() {
        let mut list = std::mem::take(&mut next);
        while !list.is_empty() {
            let item = list.remove(0);
            let (px, py) = item;
            if px < 1 || py < 1 || px >= world.width - 1 || py >= world.height - 1 {
                continue;
            }
            seen.insert(item);
            let cur_wall = world.tile(px, py).map_or(0, |t| t.wall);
            // `:3445-3448` — the same-wall / protected skip.
            if cur_wall == wall || CANNOT_BE_REPLACED.contains(&cur_wall) {
                continue;
            }
            if !solid_cave_walls(world, px, py) {
                // `:3451-3456` — the stops-at-air gate.
                if stops_at_air && cur_wall == 0 {
                    continue;
                }
                num2 += 1;
                if num2 >= MAX_WALL_OUT2 {
                    continue;
                }
                if let Some(t) = world.tile_mut(px, py) {
                    t.wall = wall;
                }
                for n in [(px - 1, py), (px + 1, py), (px, py - 1), (px, py + 1)] {
                    if !seen.contains(&n) {
                        next.push(n);
                    }
                }
                // `:3484-3516` — the stops-at-air extra ring.
                if stops_at_air {
                    for n in [
                        (px - 1, py - 1),
                        (px + 1, py - 1),
                        (px - 1, py + 1),
                        (px + 1, py + 1),
                        (px - 2, py),
                        (px + 2, py),
                    ] {
                        if !seen.contains(&n) {
                            next.push(n);
                        }
                    }
                }
            } else if world.tile(px, py).is_some_and(|t| t.active()) {
                // `:3518-3521` — the solid active cell takes the wall.
                if let Some(t) = world.tile_mut(px, py) {
                    t.wall = wall;
                }
            }
        }
    }
}

/// The `Cave Walls` pass (`WorldGen.cs:17711-17839`). The skyblock gate
/// (`denyAllGeneration`, `:17716`) lives in the step adapter. Sub-vanilla
/// size guard: the x bands need > 400 wide, the first y band needs
/// height − 220 above its floor, the remix bands need their own floors,
/// and the jungle sweep needs lavaLine above the surface — the C# `Next`
/// throws otherwise; the port skips before any draw.
pub(crate) fn pass_cave_walls(
    world: &mut World,
    world_surface: f64,
    rock_layer: f64,
    lava_line: i32,
    remix: bool,
    rng: &mut Rng,
) {
    let band1 = ((world_surface + rock_layer) as i32) / 2;
    if world.width <= 400
        || world.height - 220 <= band1
        || lava_line <= world_surface as i32
        || (remix
            && (rock_layer as i32 <= world_surface as i32 + 25
                || world.height - 350 <= rock_layer as i32))
    {
        return;
    }
    let max_tile_count = 1500;
    // `:17720-17786` — sweep 1: the family pick over the counted pockets.
    let attempts = (world.width as f64 * 0.04) as i32;
    for _ in 0..attempts {
        let mut num = 0;
        let mut x = rng.range(200, world.width - 200);
        let mut y = rng.range(band1, world.height - 220);
        if remix {
            y = rng.range(world_surface as i32 + 25, rock_layer as i32);
        }
        let mut c = count_tiles(world, x, y, false, true, max_tile_count);
        while (c.tiles >= max_tile_count || c.tiles < 10) && num < 500 {
            num += 1;
            x = rng.range(200, world.width - 200);
            y = rng.range(band1, world.height - 220);
            if remix {
                y = rng.range(world_surface as i32 + 25, rock_layer as i32);
            }
            c = count_tiles(world, x, y, false, true, max_tile_count);
        }
        if num < 500 {
            // `:17744-17783` — the family ladder over the LAST count.
            let mut num3 = rng.range(0, 2);
            if c.shroom as f64 > c.rock as f64 * 0.75 {
                num3 = 80;
            } else if c.ice > 0 {
                num3 = match num3 {
                    0 => 40,
                    _ => 71,
                };
            } else if c.lava > 0 {
                num3 = 79;
            } else {
                num3 = match rng.range(0, 4) {
                    0 => 59,
                    1 => 61,
                    2 => 170,
                    _ => 171,
                };
            }
            spread_wall(world, x, y, num3 as u16);
        }
    }
    // `:17787-17809` — sweep 2 (remix only): the dirt/grass walls over the
    // clean deep pockets.
    if remix {
        for _ in 0..attempts {
            let mut num4 = 0;
            let mut x2 = rng.range(200, world.width - 200);
            let mut y2 = rng.range(rock_layer as i32, world.height - 350);
            let mut c = count_tiles(world, x2, y2, false, true, max_tile_count);
            while (c.tiles >= max_tile_count || c.tiles < 10) && num4 < 500 {
                num4 += 1;
                x2 = rng.range(200, world.width - 200);
                y2 = rng.range(rock_layer as i32, world.height - 350);
                c = count_tiles(world, x2, y2, false, true, max_tile_count);
            }
            if num4 < 500 && c.ice == 0 && c.lava == 0 && c.sand == 0 {
                let wall = if rng.range(0, 2) != 0 { 63 } else { 2 };
                spread_wall(world, x2, y2, wall);
            }
        }
    }
    // `:17810-17837` — sweep 3: the jungle-wall pockets take the mud wall
    // through Spread.Wall2 (the wallHouse/244 re-draw gate).
    let num6 = world.width as f64 * 0.02;
    let mut k = 0;
    while (k as f64) < num6 {
        k += 1;
        let mut num7 = 0;
        let mut num8 = rng.range(200, world.width - 200);
        let mut num9 = rng.range(world_surface as i32, lava_line);
        let mut num10 = 0;
        if world.tile(num8, num9).is_some_and(|t| t.wall == 64) {
            num10 = count_tiles(world, num8, num9, true, false, max_tile_count).tiles;
        }
        while (num10 >= max_tile_count || num10 < 10) && num7 < 1000 {
            num7 += 1;
            num8 = rng.range(200, world.width - 200);
            num9 = rng.range(world_surface as i32, lava_line);
            let wall = world.tile(num8, num9).map_or(0, |t| t.wall);
            // `:17828-17831` — the wallHouse/244 gate keeps num10 as-is.
            if !crate::edit::wall_house(wall) && wall != 244 {
                num10 = if wall == 64 {
                    count_tiles(world, num8, num9, true, false, max_tile_count).tiles
                } else {
                    0
                };
            }
        }
        if num7 < 1000 {
            spread_wall2(world, num8, num9, 15);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    /// A stone world with a 12×10 air pocket (rows 61-70, cols 10-21)
    /// with NO walls — the pocket counts clean.
    fn pocket_world() -> World {
        let mut world = World::new("cw".into(), 600, 500);
        for y in 0..500 {
            for x in 0..600 {
                let i = world.index(x, y);
                world.tiles[i].kind = 1;
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for y in 61..71 {
            for x in 10..22 {
                let i = world.index(x, y);
                world.tiles[i].flags &= !FLAG_ACTIVE;
                world.tiles[i].wall = 0;
            }
        }
        world
    }

    #[test]
    fn count_tiles_measures_the_clean_pocket() {
        let world = pocket_world();
        let c = count_tiles(&world, 15, 65, false, true, 1500);
        assert_eq!(c.tiles, 120, "the 12×10 pocket");
        assert!(c.rock > 0, "the stone rim counts");
    }

    #[test]
    fn count_tiles_bails_on_a_real_wall() {
        let mut world = pocket_world();
        let i = world.index(15, 65);
        world.tiles[i].wall = 62; // the spider wall — a real wall
        let c = count_tiles(&world, 15, 65, false, true, 1500);
        assert_eq!(c.tiles, 1500, "the wall bails the count");
        // The jungle reading skips the wall bail.
        let cj = count_tiles(&world, 15, 65, true, false, 1500);
        assert!(cj.tiles < 1500, "jungle: true ignores the wall");
    }

    #[test]
    fn spread_wall_paints_the_pocket_and_the_active_rim() {
        let mut world = pocket_world();
        spread_wall(&mut world, 15, 65, 59);
        // The open cells take the wall.
        assert_eq!(world.tile(15, 65).unwrap().wall, 59);
        // The active unwalled rim cells take it too (the pocket rim has
        // wall 0 in this fixture).
        assert_eq!(world.tile(9, 65).unwrap().wall, 59);
        // A walled solid cell keeps its wall.
        let i = world.index(30, 65);
        world.tiles[i].wall = 1; // the baseline stone wall
        spread_wall(&mut world, 15, 65, 61);
        assert_eq!(world.tiles[i].wall, 1, "the baseline wall survives");
    }

    #[test]
    fn spread_wall2_converts_the_jungle_wall() {
        let mut world = pocket_world();
        for y in 61..71 {
            for x in 10..22 {
                let i = world.index(x, y);
                world.tiles[i].wall = 64;
            }
        }
        spread_wall2(&mut world, 15, 65, 15);
        assert_eq!(world.tile(15, 65).unwrap().wall, 15);
        assert_eq!(world.tile(10, 61).unwrap().wall, 15);
    }

    #[test]
    fn spread_wall2_protects_the_unreplaceable_walls() {
        let mut world = pocket_world();
        let i = world.index(15, 65);
        world.tiles[i].wall = 87; // the lihzahrd wall — protected
        spread_wall2(&mut world, 15, 65, 15);
        assert_eq!(world.tiles[i].wall, 87, "the protected wall survives");
    }

    #[test]
    fn cave_walls_pass_is_deterministic_for_a_seed() {
        let base = pocket_world();
        let mut a = base.clone();
        let mut b = base.clone();
        pass_cave_walls(&mut a, 20.0, 200.0, 300, false, &mut Rng(42));
        pass_cave_walls(&mut b, 20.0, 200.0, 300, false, &mut Rng(42));
        assert_eq!(a.tiles, b.tiles, "a == b");
    }

    #[test]
    fn cave_walls_size_guard_draws_nothing() {
        let mut world = World::new("tiny".into(), 100, 100);
        let mut rng = Rng(0x5EED);
        pass_cave_walls(&mut world, 20.0, 50.0, 80, false, &mut rng);
        assert_eq!(rng.0, 0x5EED, "the size guard draws nothing");
    }
}
