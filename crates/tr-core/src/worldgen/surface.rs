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

/// `SunflowersPart2` (`WorldGen.cs:19898-19939`): `maxTilesX * 0.002` band
/// attempts — each rolls a centre column `Next(maxTilesX)`, the band
/// `[centre - Next(10) - 7, centre + Next(10) + 7)` clamped to the world,
/// and scans every column from row 1 down to `worldSurface - 1`: the first
/// active cell stops the scan; a grass (2) top with air above gets
/// `PlaceTile(j, k - 1, 27)` — the sunflower arm.
pub(crate) fn pass_sunflowers(world: &mut World, world_surface: f64, rng: &mut Rng) {
    // `:19903-19904` — `double num = maxTilesX * 0.002; for (int i = 0;
    // (double)i < num; i++)`: the loop runs CEIL(num) times (i < num with a
    // fractional bound admits one more integer than truncation).
    let attempts = (world.width as f64 * 0.002).ceil() as i32;
    let last_row = world_surface as i32 - 1;
    for _ in 0..attempts {
        // `:19910-19912` — the three draws, in order.
        let centre = rng.range(0, world.width);
        let mut left = centre - rng.range(0, 10) - 7;
        let mut right = centre + rng.range(0, 10) + 7;
        // `:19913-19920` — the clamps.
        if left < 0 {
            left = 0;
        }
        if right > world.width - 1 {
            right = world.width - 1;
        }
        // `:19923-19936` — the column scans (`k` from 1 to worldSurface - 1).
        for j in left..right {
            for k in 1..last_row {
                let Some(t) = world.tile(j, k) else {
                    break;
                };
                // `:19927-19930` — the grass-top arm (the type test reads
                // the cell even when inactive, the active gate rides it).
                if t.kind == 2 && t.active() && !world.tile(j, k - 1).is_some_and(|a| a.active()) {
                    place_sunflower(world, j, k - 1, world_surface, rng);
                }
                if t.active() {
                    break;
                }
            }
        }
    }
}

/// `WorldGen.PlaceSunflower` (`WorldGen.cs:53636-53686`): the 2×4 sunflower
/// (tile 27) — the surface gate, the eight-cell clear box (no active tile,
/// no wall), the two grass/hallowed-grass floor cells (flat, not half
/// brick), then the frame writes: the top two rows share one `Next(3)`
/// style, the bottom two roll per cell. The `PlaceTile` liquid gate
/// (`:59454-59471`: tile 27 refuses any liquid) rides ahead — the C# runs
/// it before the arm; a liquid cell refuses with NO draw, so the gate's
/// position relative to the style draws is observable only through the
/// refusal (the port keeps it first, the no-draw refusal is identical).
fn place_sunflower(world: &mut World, x: i32, y: i32, world_surface: f64, rng: &mut Rng) {
    // `:53638-53641` — below the surface line the arm refuses (the remix
    // bypass is `Main.remixWorld`, the runtime mirror of the gen flag).
    if y as f64 > world_surface - 1.0 && !world.seeds.contains(crate::seeds::SeedFlags::REMIX) {
        return;
    }
    // `:59454-59471` — the liquid gate (both columns; tile 27 is in the
    // refusal list). No draw on refusal.
    for i in x..x + 2 {
        if world.tile(i, y).is_some_and(|t| t.liquid > 0) {
            return;
        }
    }
    // `:53642-53664` — the clear box + the floor.
    for i in x..x + 2 {
        for j in y - 3..y + 1 {
            match world.tile(i, j) {
                Some(t) if t.active() || t.wall > 0 => return,
                None => return,
                _ => {}
            }
        }
        let Some(floor) = world.tile(i, y + 1) else {
            return;
        };
        if !crate::tileobject::nactive(floor)
            || floor.half_brick()
            || floor.slope() != 0
            || (floor.kind != 2 && floor.kind != 109)
        {
            return;
        }
    }
    // `:53669-53685` — the writes. One shared style for the top two rows,
    // a fresh `Next(3)` per cell for the bottom two; the C# draws in
    // column-major order (k outer, l inner).
    let top_style = rng.range(0, 3);
    for k in 0..2 {
        for l in -3..1 {
            let mut fx = k * 18 + rng.range(0, 3) * 36;
            if l <= -2 {
                fx = k * 18 + top_style * 36;
            }
            let fy = (l + 3) * 18;
            let Some(t) = world.tile_mut(x + k, y + l) else {
                continue;
            };
            t.flags |= FLAG_ACTIVE;
            t.frame_x = fx as i16;
            t.frame_y = fy as i16;
            t.kind = 27;
        }
    }
    // `PlaceTile`'s tail `SquareTileFrame(i, j)` (`:60154`) — the port's
    // worldgen reframe seam; tile 27 is frame-important (boot statics), so
    // the stored frames survive.
    crate::edit::square_tile_frame_gen(world, x, y);
}

/// `AlchemyHerbs` (`WorldGen.cs:19964-19980`): `maxTilesX * 1.7` attempts
/// (the remix world ×5), each a `PlantAlch` call. The loop bound is the
/// double test `(double)i < num` — ceil semantics, like Sunflowers.
pub(crate) fn pass_herbs(
    world: &mut World,
    world_surface: f64,
    rock_layer: f64,
    remix: bool,
    rng: &mut Rng,
) {
    let mut attempts = world.width as f64 * 1.7;
    if remix {
        attempts *= 5.0;
    }
    let attempts = attempts.ceil() as i32;
    for _ in 0..attempts {
        plant_alch(world, world_surface, rock_layer, remix, rng);
    }
}

/// `WorldGen.PlantAlch` (`WorldGen.cs:45901-45938`): the column roll, the
/// depth-band pick (the 1/40 deep band, the 9/10 surface-to-bottom band,
/// the 1/10 anywhere band — the remix world rolls anywhere directly), the
/// scan down to the first active cell, the floor/air/liquid gate, the
/// scaled herb-density box (15 · width/4200, at most 5 herbs inside), then
/// `PlaceSuitableHerbHere`.
fn plant_alch(world: &mut World, world_surface: f64, rock_layer: f64, remix: bool, rng: &mut Rng) {
    // The sub-vanilla guard: the C# throws on an empty depth band; the port
    // skips the attempt before any draw (the S1-K-2 wave B convention). On
    // a classic world the 9/10 surface band and the 1/10 anywhere band must
    // both be non-empty; the 1/40 deep band's collapse skips after its roll.
    if world.width <= 40 || world.height <= 40 {
        return;
    }
    if !remix && world.height - 20 <= world_surface as i32 {
        return;
    }
    // `:45903` — the column.
    let x = rng.range(20, world.width - 20);
    // `:45905` — the depth pick. The band collapses are guarded the same
    // way (the C# throws; the port skips the attempt, the column draw
    // already consumed).
    let mut y = if remix {
        rng.range(20, world.height - 20)
    } else if rng.range(0, 40) == 0 {
        let lo = ((rock_layer + world.height as f64) / 2.0) as i32;
        let hi = world.height - 20;
        if hi <= lo {
            return;
        }
        rng.range(lo, hi)
    } else if rng.range(0, 10) != 0 {
        rng.range(world_surface as i32, world.height - 20)
    } else {
        rng.range(20, world.height - 20)
    };
    // `:45905-45907` — the scan down to the first active cell.
    while y < world.height - 20 && !world.tile(x, y).is_some_and(|t| t.active()) {
        y += 1;
    }
    // `:45908-45911` — the floor must be nactive, the cell above free and
    // dry. An out-of-world floor reads inactive (the C# null tile).
    let floor_ok = world
        .tile(x, y)
        .is_some_and(|t| crate::tileobject::nactive(t));
    let above = world.tile(x, y - 1);
    if !floor_ok
        || above.is_some_and(|t| t.active())
        || above.is_some_and(|t| t.liquid != 0)
    {
        return;
    }
    // `:45912-45919` — the density box: 15 scaled by width/4200 (truncated),
    // the four clamped bounds.
    let radius = (15.0 * (world.width as f64 / 4200.0)) as i32;
    let x0 = (x - radius).clamp(4, world.width - 4);
    let x1 = (x + radius).clamp(4, world.width - 4);
    let y0 = (y - radius).clamp(4, world.height - 4);
    let y1 = (y + radius).clamp(4, world.height - 4);
    // `:45920-45929` — the herb count (`Main.tileAlch`, the type test reads
    // inactive cells too — the C# tests `.type` with no active gate).
    let mut herbs = 0;
    for i in x0..=x1 {
        for j in y0..=y1 {
            if world
                .tile(i, j)
                .is_some_and(|t| crate::tileobject::tile_alch(t.kind))
            {
                herbs += 1;
            }
        }
    }
    // `:45930-45937` — at most 5 herbs in the box; the netMode send is the
    // server-only arm (no port).
    if herbs < 5 {
        place_suitable_herb_here(world, x, y);
    }
}

/// `WorldGen.PlaceSuitableHerbHere` (`WorldGen.cs:45940-45970`): the soil
/// family picks the herb style — grass/hallowed 0, jungle 1, dirt/mud 2,
/// the evil family 3, sand/pearlsand (inland of the beaches) 4, ash 5, the
/// snow/ice family 6. The C# runs every `if` (no else), but the soil sets
/// are disjoint, so at most one fires.
fn place_suitable_herb_here(world: &mut World, x: i32, y: i32) {
    let Some(kind) = world.tile(x, y).map(|t| t.kind) else {
        return;
    };
    // `WorldGen.beachDistance` = 380 (`WorldGen.cs:86914`).
    const BEACH_DISTANCE: i32 = 380;
    if kind == 2 || kind == 109 {
        crate::edit::place_alch(world, x, y - 1, 0);
    }
    if kind == 60 {
        crate::edit::place_alch(world, x, y - 1, 1);
    }
    if kind == 0 || kind == 59 {
        crate::edit::place_alch(world, x, y - 1, 2);
    }
    if matches!(kind, 23 | 661 | 25 | 203 | 199 | 662) {
        crate::edit::place_alch(world, x, y - 1, 3);
    }
    if (kind == 53 || kind == 116) && x >= BEACH_DISTANCE && x <= world.width - BEACH_DISTANCE {
        crate::edit::place_alch(world, x, y - 1, 4);
    }
    if kind == 57 || kind == 633 {
        crate::edit::place_alch(world, x, y - 1, 5);
    }
    if matches!(kind, 147 | 163 | 164 | 161 | 200) {
        crate::edit::place_alch(world, x, y - 1, 6);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A flat grass world: grass row `surface`, dirt below, air above;
    /// `frame_important` sized so the reframe seam keeps stored frames.
    fn grass_world(width: i32, height: i32, surface: i32) -> World {
        let mut w = World::new("sf".into(), width, height);
        w.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        w.frame_important[27] = true;
        for x in 0..width {
            for y in surface..height {
                let i = w.index(x, y);
                w.tiles[i].kind = if y == surface { TILE_GRASS } else { 0 };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w
    }

    /// `PlaceSunflower` (`WorldGen.cs:53636-53686`): the 2×4 write on a
    /// grass floor — the top two rows share the style, the bottom two roll
    /// per cell; eight cells, all type 27, frameY 0/18/36/54.
    #[test]
    fn place_sunflower_writes_the_2x4_frames() {
        let mut w = grass_world(100, 100, 50);
        let mut rng = Rng::from_seed(3);
        place_sunflower(&mut w, 40, 49, 80.0, &mut rng);
        let count = w
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == 27)
            .count();
        assert_eq!(count, 8, "the 2x4 sunflower");
        // The top two rows (y 46/47) share one style column pair; the
        // frameX values are k*18 + style*36.
        let top_left = w.tile(40, 46).unwrap().frame_x;
        let top_right = w.tile(41, 46).unwrap().frame_x;
        assert_eq!(top_right - top_left, 18, "the column offset");
        assert_eq!(w.tile(40, 47).unwrap().frame_x, top_left);
        assert_eq!(w.tile(41, 47).unwrap().frame_x, top_right);
        assert!(top_left % 36 == 0, "the style multiple");
        for (dy, fy) in [(0, 0), (1, 18), (2, 36), (3, 54)] {
            assert_eq!(w.tile(40, 46 + dy).unwrap().frame_y, fy);
        }
    }

    /// The floor gate (`:53660-53663`): a stone floor refuses — no write,
    /// no draw.
    #[test]
    fn place_sunflower_refuses_a_stone_floor() {
        let mut w = grass_world(100, 100, 50);
        let i = w.index(40, 50);
        w.tiles[i].kind = 1;
        let mut rng = Rng::from_seed(3);
        let mut replay = Rng::from_seed(3);
        place_sunflower(&mut w, 40, 49, 80.0, &mut rng);
        assert!(!w.tiles.iter().any(|t| t.active() && t.kind == 27));
        assert_eq!(rng.next_u32(), replay.next_u32(), "no draw on refusal");
    }

    /// The wall gate (`:53651-53654`): a wall in the clear box refuses.
    #[test]
    fn place_sunflower_refuses_a_walled_box() {
        let mut w = grass_world(100, 100, 50);
        let i = w.index(41, 47);
        w.tiles[i].wall = 2;
        let mut rng = Rng::from_seed(3);
        place_sunflower(&mut w, 40, 49, 80.0, &mut rng);
        assert!(!w.tiles.iter().any(|t| t.active() && t.kind == 27));
    }

    /// The surface gate (`:53638-53641`): below `worldSurface - 1` refuses
    /// on a classic world.
    #[test]
    fn place_sunflower_refuses_below_the_surface_line() {
        let mut w = grass_world(100, 100, 50);
        let mut rng = Rng::from_seed(3);
        place_sunflower(&mut w, 40, 49, 40.0, &mut rng);
        assert!(!w.tiles.iter().any(|t| t.active() && t.kind == 27));
    }

    /// The pass (`:19898-19939`): the band attempts plant sunflowers on the
    /// grass tops; the count is `width * 0.002` attempts, each drawing
    /// exactly three band rolls before any placement draw.
    #[test]
    fn the_pass_plants_sunflowers_on_the_grass() {
        let mut w = grass_world(4200, 400, 200);
        let mut rng = Rng::from_seed(9);
        pass_sunflowers(&mut w, 250.0, &mut rng);
        let flowers = w
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == 27)
            .count();
        assert!(flowers > 0, "the sunflowers planted ({flowers})");
        assert_eq!(flowers % 8, 0, "whole 2x4 flowers only");
        // Every flower sits above a grass/hallowed floor.
        for x in 0..w.width {
            for y in 0..w.height {
                if w.tile(x, y).is_some_and(|t| t.active() && t.kind == 27 && t.frame_y == 54) {
                    let floor = w.tile(x, y + 1).unwrap();
                    assert!(floor.active() && (floor.kind == 2 || floor.kind == 109));
                }
            }
        }
    }

    /// `PlantAlch` (`WorldGen.cs:45901-45938`) + `PlaceSuitableHerbHere`
    /// (`:45940-45970`): the herbs land on the grass tops as tile 82 with
    /// the style-0 frame (frameX 0), and the density cap keeps at most 5
    /// herbs in any scaled box.
    #[test]
    fn the_herbs_pass_plants_daybloom_on_the_grass() {
        let mut w = grass_world(4200, 1200, 300);
        let mut rng = Rng::from_seed(5);
        pass_herbs(&mut w, 350.0, 600.0, false, &mut rng);
        let herbs: Vec<_> = w
            .tiles
            .iter()
            .filter(|t| t.active() && crate::tileobject::tile_alch(t.kind))
            .collect();
        assert!(!herbs.is_empty(), "the herbs planted");
        // Every herb sits directly above a solid floor cell.
        for x in 0..w.width {
            for y in 1..w.height {
                if w.tile(x, y).is_some_and(|t| t.active() && t.kind == 82) {
                    let floor = w.tile(x, y + 1).unwrap();
                    assert!(floor.active(), "the herb at {x},{y} has a floor");
                }
            }
        }
    }

    /// The soil families (`:45940-45970`): jungle grass → style 1 (frameX
    /// 18), mud → style 2 (frameX 36), ash → style 5 (frameX 90), snow →
    /// style 6 (frameX 108).
    #[test]
    fn place_suitable_herb_picks_the_soil_style() {
        for (soil, style) in [(60u16, 1i16), (59, 2), (57, 5), (147, 6)] {
            let mut w = grass_world(100, 100, 50);
            let i = w.index(50, 50);
            w.tiles[i].kind = soil;
            place_suitable_herb_here(&mut w, 50, 50);
            let herb = w.tile(50, 49).unwrap();
            assert!(herb.active() && herb.kind == 82, "soil {soil} plants");
            assert_eq!(herb.frame_x, style * 18, "soil {soil} → style {style}");
        }
    }

    /// The sand style's beach gate (`:45958-45961`): sand inside 380 of
    /// either edge refuses; inland sand plants style 4.
    #[test]
    fn place_suitable_herb_sand_needs_the_inland_band() {
        let mut w = grass_world(1000, 100, 50);
        for x in [100, 500] {
            let i = w.index(x, 50);
            w.tiles[i].kind = 53;
        }
        place_suitable_herb_here(&mut w, 100, 50);
        place_suitable_herb_here(&mut w, 500, 50);
        assert!(!w.tile(100, 49).unwrap().active(), "the beach sand refuses");
        let inland = w.tile(500, 49).unwrap();
        assert!(inland.active() && inland.kind == 82 && inland.frame_x == 72);
    }
}
