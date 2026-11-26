//! The `Wall Variety` pass (`WorldGen.cs:16697-16754`,
//! `GenPassNameID.CaveWallVariety`): the unsafe natural-wall retexture over
//! enclosed cave pockets — a mushroom-grass (60) pocket gains the jungle
//! unsafe walls 204-207, a stone (1) floor with no wall above gains the dirt
//! unsafe 196-199 / lava unsafe 208-211 / rocks unsafe 212-215 families by
//! depth. The shape/action framework the pass rides (`ShapeFloodFill`,
//! `ModShapes.OuterOutline`, `Modifiers.IsNotSolid`/`IsTouching`/`SkipWalls`,
//! `Actions.PlaceWall`/`Blank`/`ContinueWrapper`/`Custom`/`Chain`) is ported
//! inline here, the `desert_entrances.rs` precedent.
//!
//! Deviations: `Actions.PlaceWall`'s `SquareWallFrame` self + 4 neighbours
//! (`Actions.cs:536-547`) is a visual frame pass — deferred per the port's
//! plain-write convention (`tile_runner.rs::place_wall`,
//! `mini_biomes.rs`'s wall-78 interiors). `ShapeData`'s `HashSet<Point16>`
//! iteration order (`ShapeData.cs:65-68` → `ModShapes.OuterOutline`'s
//! `foreach`) is insertion order for an add-only set that `Clear()`s between
//! placements (the .NET entries-array walk); the port keeps a `Vec` in
//! discovery order beside the membership set. The `Main.tile` direct reads
//! fold bounds through the port's `Option` accessors (the
//! `liquids.rs::scan_active` precedent).

use std::collections::{HashSet, VecDeque};

use super::noise::Rng;
use crate::world::World;

/// `WorldGen.shimmerSafetyDistance` (`WorldGen.cs:4156`, static ctor
/// `:86915`: 150).
const SHIMMER_SAFETY_DISTANCE: f64 = 150.0;

/// `ShapeData` (`ShapeData.cs:9-114`): the relative-coordinate cell set.
/// `Add` dedups (`:25-32`); `GetData`'s iteration is insertion order (see
/// the module note); `Clear` resets both halves (`:70-73`).
#[derive(Default)]
struct ShapeData {
    points: Vec<(i32, i32)>,
    set: HashSet<(i32, i32)>,
}

impl ShapeData {
    fn add(&mut self, x: i32, y: i32) {
        if self.set.insert((x, y)) {
            self.points.push((x, y));
        }
    }
    fn contains(&self, x: i32, y: i32) -> bool {
        self.set.contains(&(x, y))
    }
    fn count(&self) -> usize {
        self.set.len()
    }
    fn clear(&mut self) {
        self.points.clear();
        self.set.clear();
    }
}

/// `WorldGen.SolidOrSlopedTile(Tile, includePlatforms: false)`
/// (`WorldGen.cs:69985-70009`): active, `tileSolid`, not `tileSolidTop`,
/// not wired-inactive — slopes and half-bricks pass. (The
/// `mini_biomes.rs::solid_or_sloped` slice.)
fn solid_or_sloped(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| {
        t.active()
            && crate::tiledata::TILE_SOLID.has(t.kind)
            && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
            && (t.flags & crate::tile::FLAG_INACTIVE) == 0
    })
}

/// `Modifiers.IsNotSolid` (`Modifiers.cs:573-583`): the cell passes when it
/// is inactive or not solid-or-sloped.
fn is_not_solid(world: &World, x: i32, y: i32) -> bool {
    !world.tile(x, y).is_some_and(|t| t.active()) || !solid_or_sloped(world, x, y)
}

/// `Modifiers.IsTouching`'s `DIRECTIONS` (`Modifiers.cs:315-319`) — the
/// 8-neighbour order the scan walks (up, right, left, down, then the four
/// diagonals).
const TOUCH_DIRS: [(i32, i32); 8] = [
    (0, -1),
    (1, 0),
    (-1, 0),
    (0, 1),
    (-1, -1),
    (1, -1),
    (-1, 1),
    (1, 1),
];

/// `Modifiers.IsTouching(useDiagonals: true, tileIds)`
/// (`Modifiers.cs:278-321`): any ACTIVE neighbour whose type is in the list
/// passes; the first match short-circuits.
fn is_touching(world: &World, x: i32, y: i32, ids: &[u16]) -> bool {
    for (dx, dy) in TOUCH_DIRS {
        let Some(t) = world.tile(x + dx, y + dy) else {
            continue;
        };
        if !t.active() {
            continue;
        }
        if ids.contains(&t.kind) {
            return true;
        }
    }
    false
}

/// `ShapeFloodFill(maximumActions)` (`ShapeFloodFill.cs:17-58`) carrying the
/// pass's action chain (`WorldGen.cs:16731-16739`):
/// `IsNotSolid → Blank().Output(shapeData) → ContinueWrapper(IsTouching…
/// → Custom(foundInvalidTile = true))`. A failed `IsNotSolid` returns
/// `Fail()` = false (`GenAction.cs:34-37`), so the cell is NOT visited and
/// NOT expanded (the C# only enqueues neighbours on a successful apply);
/// a success records the relative cell and runs the touching chain for its
/// side effect (`ContinueWrapper` discards the inner result,
/// `Actions.cs:21-25`). Returns `(flag2, foundInvalidTile)` — `flag2` is the
/// C# `queue.Count == 0` after the drain loop (`:48-57`): true only when the
/// flood exhausted the pocket within the budget.
fn flood_fill(
    world: &World,
    origin: (i32, i32),
    maximum_actions: i32,
    mushroom: bool,
    shape: &mut ShapeData,
) -> (bool, bool) {
    let (ox, oy) = origin;
    let mut queue: VecDeque<(i32, i32)> = VecDeque::new();
    let mut visited: HashSet<(i32, i32)> = HashSet::new();
    queue.push_back(origin);
    let mut num = maximum_actions;
    let mut found_invalid = false;
    while !queue.is_empty() && num > 0 {
        let (x, y) = queue.pop_front().unwrap();
        if visited.contains(&(x, y)) || !is_not_solid(world, x, y) {
            continue;
        }
        // The successful apply: Blank.Output records, the touching chain
        // runs for the foundInvalidTile side effect, UnitApply returns true.
        shape.add(x - ox, y - oy);
        // The stone chain's two IsTouching gates (`:16731`): the first
        // includes the mushroom grass 60, the second does not — only a cell
        // touching the second list reaches the Custom. The mushroom chain
        // (`:16735`) carries the second list alone.
        let touches = if mushroom {
            is_touching(world, x, y, &[147, 161, 396, 397, 70, 191])
        } else {
            is_touching(world, x, y, &[60, 147, 161, 396, 397, 70, 191])
                && is_touching(world, x, y, &[147, 161, 396, 397, 70, 191])
        };
        if touches {
            found_invalid = true;
        }
        visited.insert((x, y));
        num -= 1;
        // The neighbour enqueues (`:30-45`) — the 1-tile border guards.
        if x + 1 < world.width - 1 {
            queue.push_back((x + 1, y));
        }
        if x - 1 >= 1 {
            queue.push_back((x - 1, y));
        }
        if y + 1 < world.height - 1 {
            queue.push_back((x, y + 1));
        }
        if y - 1 >= 1 {
            queue.push_back((x, y - 1));
        }
    }
    // The drain loop (`:48-57`): the first unvisited leftover re-queues and
    // breaks — the queue stays non-empty, so the Gen returns false.
    while let Some(item) = queue.pop_front() {
        if !visited.contains(&item) {
            queue.push_front(item);
            break;
        }
    }
    (queue.is_empty(), found_invalid)
}

/// `ModShapes.OuterOutline`'s `POINT_OFFSETS` (`ModShapes.cs:65-69`) — a
/// DIFFERENT order from IsTouching's (right, left, down, up, then the
/// diagonals).
const OUTLINE_OFFSETS: [(i32, i32); 8] = [
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];

/// `Actions.SkipWalls(87, 86, 244) → Actions.PlaceWall(num4)`
/// (`Modifiers.cs:471-491`, `Actions.cs:524-549`): the lihzahrd-unsafe /
/// hive-unsafe / living-wood-unsafe walls are skipped; every other cell
/// takes the wall unconditionally (the plain-write convention — see the
/// module note). Out-of-bounds writes fold away (the C# `Main.tile` array
/// carries the border padding).
fn place_wall_skip(world: &mut World, x: i32, y: i32, wall: u16) {
    if let Some(t) = world.tile_mut(x, y) {
        if t.wall == 87 || t.wall == 86 || t.wall == 244 {
            return;
        }
        t.wall = wall;
    }
}

/// `WorldUtils.Gen(point, ModShapes.OuterOutline(shapeData, useDiagonals:
/// true, useInterior: true), …)` (`WorldGen.cs:16742`, `ModShapes.cs:43-61`):
/// every recorded cell (the interior arm) plus its 8-neighbour ring cells
/// that are NOT in the set, in discovery order. The origin is the FLOOR
/// cell `(x, y)` while the data was recorded relative to `(x, y - 1)` — the
/// whole outline lands one row lower, verbatim.
fn outer_outline_place(world: &mut World, origin: (i32, i32), shape: &ShapeData, wall: u16) {
    let (ox, oy) = origin;
    for &(dx, dy) in &shape.points {
        place_wall_skip(world, ox + dx, oy + dy, wall);
        for (px, py) in OUTLINE_OFFSETS {
            if !shape.contains(dx + px, dy + py) {
                place_wall_skip(world, ox + dx + px, oy + dy + py, wall);
            }
        }
    }
}

/// `WorldGen.RandomWorldPoint(top, right, bottom, left)`
/// (`WorldGen.cs:27111-27114`): X first (`Next(left, maxTilesX - right)`),
/// then Y (`Next(top, maxTilesY - bottom)`).
fn random_world_point(rng: &mut Rng, world: &World, top: i32, right: i32, bottom: i32, left: i32) -> (i32, i32) {
    (
        rng.range(left, world.width - right),
        rng.range(top, world.height - bottom),
    )
}

/// `Vector2D.Distance` (`ReLogic.Utilities/Vector2D.cs:138-143`).
fn distance(a: (f64, f64), b: (f64, f64)) -> f64 {
    let dx = a.0 - b.0;
    let dy = a.1 - b.1;
    (dx * dx + dy * dy).sqrt()
}

/// The `Wall Variety` pass (`WorldGen.cs:16697-16754`). `num` placements
/// (`300 * worldArea / 5040000`), a 100000-failure budget; each attempt
/// draws a world point below the surface (the shimmer safety re-draw loop
/// `:16710-16713`), picks the wall family from the floor tile (`:16718-16727`
/// — the classic and remix arms verbatim), floods the pocket above
/// (`ShapeFloodFill(1000)`), and places the outline only when the pocket is
/// >50 cells, fully drained and touches no invalid tile (`:16740`). The
/// skyblock gate (`denyAllGeneration`, `:16699`/`:3131`) lives in the step
/// adapter. Sub-vanilla size guard: the point bands must be non-empty
/// (the mining-explosives convention — the C# `Next` throws otherwise).
pub(crate) fn pass_wall_variety(
    world: &mut World,
    world_surface: f64,
    rock_layer: f64,
    lava_line: i32,
    shimmer_position: (f64, f64),
    remix: bool,
    rng: &mut Rng,
) {
    let top = world_surface as i32;
    if world.width <= 4 || world.height - 190 < top {
        return;
    }
    // `:16702-16705` — the placement count and the failure budget.
    let mut num = (300.0 * ((world.width * world.height) as f64 / 5040000.0)) as i32;
    let mut num3 = 100000;
    let mut shape = ShapeData::default();
    // The C# budget semantics are preserved verbatim for normal worlds;
    // only sub-vanilla sizes get the hang guard (their bands can all fail).
    let subvanilla = world.width < 4200 || world.height < 1200;
    let guard_limit = if subvanilla { 1_000_000u64 } else { u64::MAX };
    let mut guard = 0u64;
    while num > 0 && num3 > 0 && guard < guard_limit {
        guard += 1;
        // W4-d103 P21: the C# decrements num3 only inside the placement
        // gate, so an all-fail band can spin; the guard bounds sub-vanilla.
        let mut point = random_world_point(rng, world, top, 2, 190, 2);
        let reroll_limit = if subvanilla { 10_000u32 } else { u32::MAX };
        let mut rerolls = 0u32;
        while distance((point.0 as f64, point.1 as f64), shimmer_position) < SHIMMER_SAFETY_DISTANCE
            && rerolls < reroll_limit
        {
            point = random_world_point(rng, world, top, 2, 190, 2);
            rerolls += 1;
        }
        let (x, y) = point;
        let Some(tile) = world.tile(x, y) else {
            continue;
        };
        if !tile.active() {
            continue;
        }
        let kind = tile.kind;
        let above_wall = world.tile(x, y - 1).map_or(0, |t| t.wall);
        let above_active = world.tile(x, y - 1).is_some_and(|t| t.active());
        // `:16718-16727` — the wall family pick (the draws happen before
        // the `!tile2.active()` gate, verbatim).
        let mushroom = kind == 60;
        let wall: u16 = if mushroom {
            (204 + rng.range(0, 4)) as u16
        } else if kind == 1 && above_wall == 0 {
            if remix {
                if (y as f64) > rock_layer {
                    (196 + rng.range(0, 4)) as u16
                } else if y <= lava_line || rng.range(0, 2) != 0 {
                    (212 + rng.range(0, 4)) as u16
                } else {
                    (208 + rng.range(0, 4)) as u16
                }
            } else if (y as f64) < rock_layer {
                (196 + rng.range(0, 4)) as u16
            } else if y >= lava_line {
                (208 + rng.range(0, 4)) as u16
            } else {
                (212 + rng.range(0, 4)) as u16
            }
        } else {
            0
        };
        if wall != 0 && !above_active {
            // `:16730-16739` — the flood over the pocket above the floor.
            let (complete, found_invalid) = flood_fill(world, (x, y - 1), 1000, mushroom, &mut shape);
            // `:16740-16748` — the >50-cell, drained, clean-pocket gate.
            if shape.count() > 50 && complete && !found_invalid {
                outer_outline_place(world, (x, y), &shape, wall);
                num -= 1;
            } else {
                num3 -= 1;
            }
            shape.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    /// A stone world with a row-71 stone floor and 12-wide × 10-row air
    /// pockets every 20 columns above it (120 cells each — >50, <1000).
    fn pocket_world() -> World {
        let mut world = World::new("wv".into(), 300, 300);
        for y in 0..300 {
            for x in 0..300 {
                let i = world.index(x, y);
                world.tiles[i].kind = 1; // stone (TileID.Stone = 1)
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for x in 0..300 {
            let i = world.index(x, 71);
            world.tiles[i].kind = 1; // stone floor (TileID.Stone = 1)
        }
        for base in (10..290).step_by(20) {
            for y in 61..71 {
                for x in base..base + 12 {
                    let i = world.index(x, y);
                    world.tiles[i].flags &= !FLAG_ACTIVE;
                }
            }
        }
        world
    }

    #[test]
    fn wall_variety_paints_the_dirt_unsafe_family_above_the_pockets() {
        let mut world = pocket_world();
        pass_wall_variety(&mut world, 20.0, 200.0, 250, (0.0, 0.0), false, &mut Rng(42));
        // 300*300/5040000*300 = 5 placements — the pockets qualify, so the
        // 196-199 family lands (y=71 < rockLayer 200).
        let painted = world
            .tiles
            .iter()
            .filter(|t| (196..=199).contains(&t.wall))
            .count();
        assert!(painted > 50, "the outline walls land ({painted})");
    }

    #[test]
    fn wall_variety_is_deterministic_for_a_seed() {
        let base = pocket_world();
        let mut a = base.clone();
        let mut b = base.clone();
        pass_wall_variety(&mut a, 20.0, 200.0, 250, (0.0, 0.0), false, &mut Rng(42));
        pass_wall_variety(&mut b, 20.0, 200.0, 250, (0.0, 0.0), false, &mut Rng(42));
        assert_eq!(a.tiles, b.tiles, "a == b");
    }

    #[test]
    fn wall_variety_size_guard_draws_nothing() {
        let mut world = World::new("tiny".into(), 100, 100);
        let mut rng = Rng(0x5EED);
        // height - 190 < surface → the guard skips before any draw.
        pass_wall_variety(&mut world, 20.0, 50.0, 80, (0.0, 0.0), false, &mut rng);
        assert_eq!(rng.0, 0x5EED, "the size guard draws nothing");
    }

    #[test]
    fn flood_fill_rejects_a_pocket_touching_glowing_mushrooms() {
        let mut world = pocket_world();
        // A glowing-mushroom tile (70) beside the first pocket.
        let i = world.index(9, 65);
        world.tiles[i].kind = 70;
        world.tiles[i].flags |= FLAG_ACTIVE;
        let mut shape = ShapeData::default();
        let (complete, invalid) = flood_fill(&world, (12, 70), 1000, false, &mut shape);
        assert!(complete, "the 120-cell pocket drains");
        assert!(invalid, "the 70 neighbour trips the Custom");
        assert!(shape.count() > 50);
    }

    #[test]
    fn outline_skips_the_unsafe_walls_and_shifts_one_row() {
        let mut world = pocket_world();
        let mut shape = ShapeData::default();
        let (complete, invalid) = flood_fill(&world, (12, 70), 1000, false, &mut shape);
        assert!(complete && !invalid);
        // A lihzahrd-unsafe wall (87) inside the pocket survives.
        let guard = world.index(14, 66);
        world.tiles[guard].wall = 87;
        outer_outline_place(&mut world, (12, 71), &shape, 197);
        assert_eq!(world.tiles[guard].wall, 87, "SkipWalls keeps the 87");
        // The data was recorded relative to (12, 70); the outline origin is
        // (12, 71) — the pocket's top row 61 lands its wall at row 62.
        assert_eq!(world.tile(14, 62).unwrap().wall, 197);
        // The interior cell itself (12, 70) → (12, 71), the stone floor.
        assert_eq!(world.tile(12, 71).unwrap().wall, 197);
    }

    #[test]
    fn mushroom_floor_paints_the_jungle_unsafe_family() {
        let mut world = pocket_world();
        // Turn every floor tile into mushroom grass (60).
        for x in 0..300 {
            let i = world.index(x, 71);
            world.tiles[i].kind = 60;
        }
        pass_wall_variety(&mut world, 20.0, 200.0, 250, (0.0, 0.0), false, &mut Rng(7));
        let painted = world
            .tiles
            .iter()
            .filter(|t| (204..=207).contains(&t.wall))
            .count();
        assert!(painted > 50, "the 204-207 family lands ({painted})");
    }
}
