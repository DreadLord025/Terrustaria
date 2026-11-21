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

/// `DyePlants` (`WorldGen.cs:19981-19994`): one `plantDye` attempt per
/// column (`Next(100, maxTilesX - 100)`, `Next(100, UnderworldLayer)`),
/// then `MatureTheHerbPlants`, `GrowGlowTulips`, `PlaceChilletEggs` — the
/// four tails in the C#'s call order, all on the pass's stream.
pub(crate) fn pass_dye_plants(
    world: &mut World,
    world_surface: f64,
    rock_layer: f64,
    remix: bool,
    rng: &mut Rng,
) {
    // `Main.UnderworldLayer` = `maxTilesY - 200` (`Main.cs:2790`).
    let underworld_layer = world.height - 200;
    // The sub-vanilla guard: both bands must be non-empty (the C# throws;
    // the port skips the loop before any draw — the wave B convention).
    if world.width > 200 && underworld_layer > 100 {
        for _ in 0..world.width {
            let x = rng.range(100, world.width - 100);
            let y = rng.range(100, underworld_layer);
            plant_dye(world, x, y, world_surface, underworld_layer, remix, rng);
        }
    }
    mature_the_herb_plants(world, rock_layer, rng);
    grow_glow_tulips(world, world_surface, remix, rng);
    place_chillet_eggs(world, rock_layer, remix, rng);
}

/// The dye-plant neighbour gate (`plantDye`'s three scans,
/// `WorldGen.cs:71304-71313` / `:71388-71397` / `:71441-71450`): any active
/// tile 227 in the box bails. The classic arm (`exoticPlant` false) only
/// counts the pre-strange styles (`frameX < 272`); the box is half-open
/// `[x - radius, x + radius)` for the first two scans, the third uses the
/// unclamped bounds.
fn dye_nearby(world: &World, x: i32, y: i32, radius: i32, clamp: bool) -> bool {
    let (x0, x1, y0, y1) = if clamp {
        (
            (x - radius).clamp(1, world.width - 2),
            (x + radius).clamp(1, world.width - 2),
            (y - radius).clamp(1, world.height - 2),
            (y + radius).clamp(1, world.height - 2),
        )
    } else {
        (x - radius, x + radius, y - radius, y + radius)
    };
    for i in x0..x1 {
        for j in y0..y1 {
            if world
                .tile(i, j)
                .is_some_and(|t| t.active() && t.kind == 227 && t.frame_x < 272)
            {
                return true;
            }
        }
    }
    false
}

/// The above-cell plant gate (`:71298` / `:71382`): free, or one of the
/// cut plants (3/51/61/73/74/184).
fn dye_above_ok(world: &World, x: i32, y: i32) -> bool {
    match world.tile(x, y) {
        None => false,
        Some(t) => !t.active() || matches!(t.kind, 3 | 51 | 61 | 73 | 74 | 184),
    }
}

/// `WorldGen.plantDye` (`WorldGen.cs:71286-71455`, the classic
/// `exoticPlant = false` arm — the pass never calls the exotic one): the
/// surface band, the underground band and the hanging-dirt tail. The
/// `PlaceTile` liquid gate (`:59454-59471` — 227 is NOT in the refusal
/// list) passes every liquid cell, so the port's `place_dye` carries the
/// per-style liquid rules only.
///
/// This is the GEN-TIME port (the worldgen pass's xorshift `Rng` + the
/// explicit `world_surface`/`underworld_layer` anchors + the remix bypass).
/// The runtime growth twin (`growth::decorations::plant_dye`/`place_dye`)
/// reads `world.world_surface` and draws from `UnifiedRandom` — the
/// `UpdateWorld` stream — so the two cannot share a body without a stream
/// seam; the behaviour (the soil dispatch, the style picks, the density
/// boxes) is the same C# function, ported per caller like the other
/// gen/runtime pairs.
fn plant_dye(
    world: &mut World,
    x: i32,
    y: i32,
    world_surface: f64,
    underworld_layer: i32,
    remix: bool,
    rng: &mut Rng,
) {
    // `:71289-71292` — the active + 95-border gate.
    let Some(kind) = world.tile(x, y).filter(|t| t.active()).map(|t| t.kind) else {
        return;
    };
    if x < 95 || x > world.width - 95 || y < 95 || y > world.height - 95 {
        return;
    }
    let radius = 90;
    // `:71298-71377` — the surface band.
    if ((y as f64) < world_surface || remix) && dye_above_ok(world, x, y - 1) {
        if dye_nearby(world, x, y, radius, true) {
            return;
        }
        if kind == 2 || kind == 109 {
            // `:71329-71336` — the grass pick: 1/4 orange (4), else green (3).
            let style = if rng.range(0, 4) == 0 { 4 } else { 3 };
            place_dye(world, x, y - 1, style);
        } else if kind == 60 {
            // `:71340-71347` — the jungle pick: 1/2 teal (2), else orange (4).
            let style = if rng.range(0, 2) == 0 { 2 } else { 4 };
            place_dye(world, x, y - 1, style);
        } else if kind == 53
            && world.tile(x, y - 1).is_some_and(|t| t.liquid == u8::MAX)
            && world.tile(x, y - 2).is_some_and(|t| t.liquid == u8::MAX)
        {
            // `:71349-71352` — the submerged sand: teal (2).
            place_dye(world, x, y - 1, 2);
        } else if kind == 80
            && !world.tile(x - 1, y - 1).is_some_and(|t| t.active())
            && !world.tile(x + 1, y - 1).is_some_and(|t| t.active())
        {
            // `:71353-71376` — the cactus arm: no evil altar/tube (112/234)
            // in the 11×21 box, then lime (6). The C#'s try/catch folds to
            // the port's bounds-safe reads.
            let mut clear = true;
            for m in x - 5..=x + 5 {
                for n in y - 5..=y + 15 {
                    if world
                        .tile(m, n)
                        .is_some_and(|t| t.active() && (t.kind == 112 || t.kind == 234))
                    {
                        clear = false;
                    }
                }
            }
            if clear {
                place_dye(world, x, y - 1, 6);
            }
        }
    }
    // `:71378-71381` — the underground gate.
    if ((y as f64) < world_surface && !remix) || y >= underworld_layer {
        return;
    }
    // `:71382-71436` — the underground band.
    if dye_above_ok(world, x, y - 1) {
        if dye_nearby(world, x, y, radius, true) {
            return;
        }
        if kind == 60 {
            // `:71408-71419` — the jungle pick: 1/2 teal (2), else 1/2
            // plain (0), else red (1). The second roll draws only when the
            // first fails (the C# else-if).
            if rng.range(0, 2) == 0 {
                place_dye(world, x, y - 1, 2);
            } else if rng.range(0, 2) == 0 {
                place_dye(world, x, y - 1, 0);
            } else {
                place_dye(world, x, y - 1, 1);
            }
        } else if kind == 0 || kind == 1 || kind == 59 {
            // `:71423-71430` — the dirt/stone/mud pick: 1/2 plain (0), else
            // red (1).
            let style = if rng.range(0, 2) == 0 { 0 } else { 1 };
            place_dye(world, x, y - 1, style);
        } else if world.tile(x, y - 1).is_some_and(|t| t.liquid == u8::MAX)
            && world.tile(x, y - 2).is_some_and(|t| t.liquid == u8::MAX)
        {
            // `:71432-71435` — the submerged cell: blue (5).
            place_dye(world, x, y - 1, 5);
        }
    }
    // `:71437-71454` — the hanging-dirt tail.
    if world.tile(x, y + 1).is_some_and(|t| t.active()) {
        return;
    }
    if dye_nearby(world, x, y, radius, false) {
        return;
    }
    if kind == 0 {
        place_dye(world, x, y + 1, 7);
    }
}

/// `WorldGen.PlaceDye` (`WorldGen.cs:53988-54064`): the per-style support
/// and liquid rules, then the write — `type 227, active, halfBrick false,
/// slope 0, frameY 0, frameX 34 * style`. The classic styles only (0-7;
/// the strange 8-11 are the exotic arm's).
fn place_dye(world: &mut World, x: i32, y: i32, style: i32) {
    // `:53991-53994` — both neighbours must exist.
    let (Some(above), Some(below)) = (world.tile(x, y - 1).copied(), world.tile(x, y + 1).copied())
    else {
        return;
    };
    let cell = match world.tile(x, y).copied() {
        Some(t) => t,
        None => return,
    };
    let mut ok = false;
    if style == 7 {
        // `:53995-54005` — the hanging arm: the below cell free or a cut
        // plant, the above cell solid, the below cell inactive.
        if below.active() && !matches!(below.kind, 3 | 51 | 697 | 61 | 703 | 73 | 74 | 184) {
            return;
        }
        if crate::edit::solid_tile(world, x, y - 1) && !below.active() {
            ok = true;
        }
    } else {
        // `:54008-54011` — the above cell free or a cut plant.
        if above.active() && !matches!(above.kind, 3 | 51 | 697 | 61 | 703 | 73 | 74 | 184) {
            return;
        }
        if style == 6 {
            // `:54012-54018` — the cactus seat: a flat cactus below with
            // free sides.
            if crate::tileobject::nactive(&below)
                && below.kind == 80
                && !world.tile(x - 1, y + 1).is_some_and(|t| t.active())
                && !world.tile(x + 1, y + 1).is_some_and(|t| t.active())
            {
                ok = true;
            }
        } else if crate::edit::solid_tile(world, x, y + 1) && !above.active() {
            // `:54019-54053` — the floor seat + the per-style liquid rules.
            match style {
                5 => {
                    // Full water only (no lava/honey/shimmer).
                    if cell.liquid == u8::MAX
                        && cell.liquid_kind() == crate::tile::LiquidKind::Water
                    {
                        ok = true;
                    }
                }
                _ => {
                    if cell.liquid == 0 {
                        if style == 3 || style == 4 {
                            // The wall-less requirement.
                            if cell.wall == 0 {
                                ok = true;
                            }
                        } else {
                            ok = true;
                        }
                    }
                }
            }
        }
    }
    if ok {
        // `:54055-54063` — the write.
        if let Some(t) = world.tile_mut(x, y) {
            t.kind = 227;
            t.flags |= FLAG_ACTIVE;
            t.flags &= !crate::tile::FLAG_HALF_BRICK;
            t.set_slope(0);
            t.frame_y = 0;
            t.frame_x = (34 * style) as i16;
        }
        // `PlaceTile`'s tail `SquareTileFrame` (`:60044`).
        crate::edit::square_tile_frame_gen(world, x, y);
    }
}

/// `WorldGen.MatureTheHerbPlants` (`WorldGen.cs:45748-45775`): the
/// full-world sweep — below the rock layer, a free dry cell above mud/dirt
/// solid floor grows a blinkroot (82, frameX 36) at 1/25; every herb cell
/// (the type test reads inactive cells too) matures 82 → 83 at 1/3, then
/// the blinkroot 36 → 84 at 1/2 and the fireblossom 108 → 84 at 1/3.
fn mature_the_herb_plants(world: &mut World, rock_layer: f64, rng: &mut Rng) {
    for i in 10..world.width - 10 {
        for j in 10..world.height - 10 {
            // `:45754-45760` — the wild blinkroot arm. The floor's type test
            // reads the cell even when inactive; `SolidTile` gates it.
            let floor_kind = world.tile(i, j + 1).map(|t| t.kind);
            if (j as f64) > rock_layer
                && matches!(floor_kind, Some(59) | Some(0))
                && crate::edit::solid_tile(world, i, j + 1)
                && !world.tile(i, j).is_some_and(|t| t.active())
                && world.tile(i, j).is_some_and(|t| t.liquid == 0)
                && rng.range(0, 25) == 0
            {
                if let Some(t) = world.tile_mut(i, j) {
                    t.flags |= FLAG_ACTIVE;
                    t.kind = 82;
                    t.frame_x = 36;
                    t.frame_y = 0;
                }
            }
            // `:45761-45772` — the maturation arms. The C# reads `.type`
            // with no active gate; the port matches it.
            let herb = world.tile(i, j).map(|t| (t.kind, t.frame_x));
            if let Some((82, fx)) = herb {
                if rng.range(0, 3) == 0 {
                    if let Some(t) = world.tile_mut(i, j) {
                        t.kind = 83;
                    }
                    if fx == 36 && rng.range(0, 2) == 0 {
                        if let Some(t) = world.tile_mut(i, j) {
                            t.kind = 84;
                        }
                    }
                    if fx == 108 && rng.range(0, 3) == 0 {
                        if let Some(t) = world.tile_mut(i, j) {
                            t.kind = 84;
                        }
                    }
                }
            }
        }
    }
}

/// `TileID.Sets.Conversion.Grass` (`TileID.cs:86`): 2/23/199/109/477/492.
/// The generated tables carry no `SET_CONVERSION_GRASS` (the grass family
/// is the `SET_GRASS`/`SET_GRASS_SPECIAL` pair plus the golf grasses); the
/// literal set is the chillet/tulip ground gate's read.
const CONVERSION_GRASS: [u16; 6] = [2, 23, 199, 109, 477, 492];

/// `WorldGen.HasValidGroundForGlowTulipBelowSpot` /
/// `HasValidGroundForChilletEggBelowSpot` (`WorldGen.cs:73096-73117` /
/// `:45725-45746` — the two bodies are identical): the floor cell must be
/// active, one of the dirt/mushroom/ash/mud/hive kinds, the conversion
/// grass/stone families or a moss, and `SolidTileAllowBottomSlope`.
fn valid_ground_below(world: &World, x: i32, y: i32) -> bool {
    // `InWorld(x, y, 2)`.
    if x < 2 || y < 2 || x >= world.width - 2 || y >= world.height - 2 {
        return false;
    }
    let Some(floor) = world.tile(x, y + 1) else {
        return false;
    };
    if !floor.active() {
        return false;
    }
    let kind = floor.kind;
    if kind != 0
        && kind != 70
        && kind != 633
        && kind != 59
        && kind != 225
        && !CONVERSION_GRASS.contains(&kind)
        && !crate::tiledata::SET_CONVERSION_STONE.has(kind)
        && !crate::tiledata::TILE_MOSS.has(kind)
    {
        return false;
    }
    crate::edit::solid_tile_allow_bottom_slope(world, x, y + 1)
}

/// `WorldGen.NoNearbyGlowTulips` (`WorldGen.cs:73144-73162`): no active
/// 656 in the ±120 box (clamped to the 10-border).
fn no_nearby_glow_tulips(world: &World, x: i32, y: i32) -> bool {
    let x0 = (x - 120).clamp(10, world.width - 11);
    let x1 = (x + 120).clamp(10, world.width - 11);
    let y0 = (y - 120).clamp(10, world.height - 11);
    let y1 = (y + 120).clamp(10, world.height - 11);
    for i in x0..=x1 {
        for j in y0..=y1 {
            if world.tile(i, j).is_some_and(|t| t.active() && t.kind == 656) {
                return false;
            }
        }
    }
    true
}

/// `WorldGen.NoNearbyChilletEggs` (`WorldGen.cs:45680-45698`): no active
/// 752 in the ±20 box (clamped to the 10-border).
fn no_nearby_chillet_eggs(world: &World, x: i32, y: i32) -> bool {
    let x0 = (x - 20).clamp(10, world.width - 11);
    let x1 = (x + 20).clamp(10, world.width - 11);
    let y0 = (y - 20).clamp(10, world.height - 11);
    let y1 = (y + 20).clamp(10, world.height - 11);
    for i in x0..=x1 {
        for j in y0..=y1 {
            if world.tile(i, j).is_some_and(|t| t.active() && t.kind == 752) {
                return false;
            }
        }
    }
    true
}

/// `WorldGen.TryGrowingGlowTulip` (`WorldGen.cs:73119-73142`): five jitter
/// attempts (`Next(max(10, x-10), min(width-10, x+10))` per axis), each
/// gated on the ground + the neighbour box, then `PlaceTile(656)` — the
/// glow-tulip arm (`:59495-59506`): a free/cut/breakable cell above valid
/// ground writes active 656, frames 0, slope 0, no half brick. The liquid
/// gate (`:59468`) refuses any liquid cell before the arm.
fn try_growing_glow_tulip(world: &mut World, x: i32, y: i32, rng: &mut Rng) -> bool {
    for _ in 0..5 {
        let nx = rng.range((x - 10).max(10), (x + 10).min(world.width - 10));
        let ny = rng.range((y - 10).max(10), (y + 10).min(world.height - 10));
        if !valid_ground_below(world, nx, ny) || !no_nearby_glow_tulips(world, nx, ny) {
            continue;
        }
        // The PlaceTile liquid gate + the 656 arm.
        let Some(cell) = world.tile(nx, ny).copied() else {
            continue;
        };
        if cell.liquid > 0 {
            continue;
        }
        if (!cell.active()
            || crate::tileobject::tile_cut(cell.kind)
            || crate::tileobject::breakable_when_placing(cell.kind))
            && valid_ground_below(world, nx, ny)
        {
            if let Some(t) = world.tile_mut(nx, ny) {
                t.flags |= FLAG_ACTIVE;
                t.kind = 656;
                t.flags &= !crate::tile::FLAG_HALF_BRICK;
                t.set_slope(0);
                t.frame_x = 0;
                t.frame_y = 0;
            }
        }
        if world.tile(nx, ny).is_some_and(|t| t.active() && t.kind == 656) {
            return true;
        }
    }
    false
}

/// `WorldGen.GrowGlowTulips` (`WorldGen.cs:45622-45650`): 2/4/6 tulips by
/// world size, the 10000-attempt budget — the second half of the quota
/// prefers the right band (`width-300..width-100`) once 75% of the budget
/// is spent or a 1/2 roll passes; the y band is `worldSurface..height-200`
/// (remix `..height-350`).
fn grow_glow_tulips(world: &mut World, world_surface: f64, remix: bool, rng: &mut Rng) {
    // `GetWorldSize` (`:6559-6570`): 0 ≤ 4200, 1 ≤ 6400, 2 above.
    let size = if world.width <= 4200 {
        0
    } else if world.width <= 6400 {
        1
    } else {
        2
    };
    let quota = match size {
        1 => 4,
        2 => 6,
        _ => 2,
    };
    let mut placed = 0;
    let mut budget = 10000;
    let budget_floor = (10000.0 * 0.75) as i32;
    let y_hi = world.height - if remix { 350 } else { 200 };
    while placed < quota {
        budget -= 1;
        if budget <= 0 {
            break;
        }
        // The x-band pick: the ternary's condition draws `Next(2)` only
        // when the first half is already placed AND the budget is above
        // the floor (the C# `||` short-circuits).
        let right = placed >= quota / 2 && (budget <= budget_floor || rng.range(0, 2) != 0);
        // The sub-vanilla band collapse: the C# throws AFTER the x draw;
        // the port draws then stops (the consumed draws stay consumed).
        if world.width <= 400 {
            break;
        }
        let x = if right {
            rng.range(world.width - 300, world.width - 100)
        } else {
            rng.range(100, 300)
        };
        let y_lo = world_surface as i32;
        if y_hi <= y_lo {
            break;
        }
        let y = rng.range(y_lo, y_hi);
        if try_growing_glow_tulip(world, x, y, rng) {
            placed += 1;
        }
    }
}

/// `WorldGen.TryPlacingChilletEgg` (`WorldGen.cs:45700-45723`): five jitter
/// attempts, each gated on the ground + the neighbour box, then
/// `PlaceTile(752)` — the `Place2x2` arm (`:60196-60197` →
/// `:51330-51423`): the 2×2 clear box on a solid/table floor, the frames
/// `k*18` / `style*36 + l*18`.
fn try_placing_chillet_egg(world: &mut World, x: i32, y: i32, rng: &mut Rng) -> bool {
    for _ in 0..5 {
        let nx = rng.range((x - 10).max(10), (x + 10).min(world.width - 10));
        let ny = rng.range((y - 10).max(10), (y + 10).min(world.height - 10));
        if !valid_ground_below(world, nx, ny) || !no_nearby_chillet_eggs(world, nx, ny) {
            continue;
        }
        place_2x2(world, nx, ny, 752, 0);
        if world.tile(nx, ny).is_some_and(|t| t.active() && t.kind == 752) {
            return true;
        }
    }
    false
}

/// `WorldGen.Place2x2` (`WorldGen.cs:51330-51423`, the default floor arm —
/// the 752 chillet egg's route): the 5-border gate, the 2×2 clear box
/// (`x-1..x+1`, `y-1..y+1`), the two floor cells `nactive` and
/// (`SolidTile2` or `tileTable`), then the four writes at `(x-1+k, y-1+l)`
/// with `frameX k*18`, `frameY style*36 + l*18`.
fn place_2x2(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    // `:51336-51339` — the border gate.
    if x < 5 || x > world.width - 5 || y < 5 || y > world.height - 5 {
        return;
    }
    // `:51349-51383` — the clear box + the floor seats.
    for i in x - 1..x + 1 {
        for j in y - 1..y + 1 {
            if world.tile(i, j).is_some_and(|t| t.active()) {
                return;
            }
        }
        let Some(floor) = world.tile(i, y + 1) else {
            return;
        };
        if !crate::tileobject::nactive(floor)
            || (!crate::edit::solid_tile2(world, i, y + 1)
                && !crate::tileobject::is_tile_table(floor.kind))
        {
            return;
        }
    }
    // `:51409-51422` — the writes.
    for k in 0..2 {
        for l in 0..2 {
            if let Some(t) = world.tile_mut(x - 1 + k, y - 1 + l) {
                t.flags |= FLAG_ACTIVE;
                t.frame_x = (k * 18) as i16;
                t.frame_y = (style * 36 + l * 18) as i16;
                t.kind = kind;
            }
        }
    }
}

/// `WorldGen.PlaceChilletEggs` (`WorldGen.cs:45652-45678`): 6/9/12 eggs by
/// world size (the small world's 6 — the C# switch's `_` arm), the
/// 10000-attempt budget, the x band `100..width-100`, the y band
/// `rockLayer..height-200` (remix `..height-350`).
fn place_chillet_eggs(world: &mut World, rock_layer: f64, remix: bool, rng: &mut Rng) {
    let size = if world.width <= 4200 {
        0
    } else if world.width <= 6400 {
        1
    } else {
        2
    };
    let quota = match size {
        1 => 9,
        2 => 12,
        _ => 6,
    };
    let mut placed = 0;
    let mut budget = 10000;
    let y_hi = world.height - if remix { 350 } else { 200 };
    while placed < quota {
        budget -= 1;
        if budget <= 0 {
            break;
        }
        let y_lo = rock_layer as i32;
        if world.width <= 200 {
            break;
        }
        let x = rng.range(100, world.width - 100);
        if y_hi <= y_lo {
            break;
        }
        let y = rng.range(y_lo, y_hi);
        if try_placing_chillet_egg(world, x, y, rng) {
            placed += 1;
        }
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

    /// `PlaceDye` (`WorldGen.cs:53988-54064`): the grass style 3 needs a
    /// wall-less cell; the write is frameX 34*style, frameY 0.
    #[test]
    fn place_dye_writes_the_style_frame() {
        let mut w = grass_world(200, 200, 100);
        // The cell above the grass at (100, 99): free, no wall, dry.
        place_dye(&mut w, 100, 99, 3);
        let t = w.tile(100, 99).unwrap();
        assert!(t.active() && t.kind == 227);
        assert_eq!(t.frame_x, 34 * 3);
        assert_eq!(t.frame_y, 0);
        // A walled cell refuses style 3 (no draw, no write).
        let i = w.index(110, 99);
        w.tiles[i].wall = 2;
        place_dye(&mut w, 110, 99, 3);
        assert!(!w.tile(110, 99).unwrap().active());
        // Style 0 accepts the wall.
        place_dye(&mut w, 110, 99, 0);
        assert!(w.tile(110, 99).is_some_and(|t| t.active() && t.kind == 227));
    }

    /// The hanging style 7 (`:53995-54005`): a solid ceiling above, free
    /// below — the write lands under the dirt overhang.
    #[test]
    fn place_dye_style_7_hangs_from_a_ceiling() {
        let mut w = grass_world(200, 200, 100);
        // A dirt ceiling at (100, 50), air below.
        let i = w.index(100, 50);
        w.tiles[i].kind = 0;
        w.tiles[i].flags |= FLAG_ACTIVE;
        place_dye(&mut w, 100, 51, 7);
        let t = w.tile(100, 51).unwrap();
        assert!(t.active() && t.kind == 227 && t.frame_x == 34 * 7);
    }

    /// `MatureTheHerbPlants` (`:45748-45775`): the 82 → 83 maturation and
    /// the blinkroot 36 → 84 bloom fire on their rolls; a fireblossom 108
    /// blooms at 1/3.
    #[test]
    fn mature_the_herb_plants_blooms_the_herbs() {
        let mut w = grass_world(100, 100, 50);
        // An 82 blinkroot (frameX 36) at (50, 40) — above the rock layer.
        let i = w.index(50, 40);
        w.tiles[i].kind = 82;
        w.tiles[i].frame_x = 36;
        w.tiles[i].flags |= FLAG_ACTIVE;
        // Sweep seeds until the 1/3 + 1/2 both hit (the bloom).
        let mut bloomed = false;
        for seed in 0..40u32 {
            let mut w2 = w.clone();
            let mut rng = Rng::from_seed(seed);
            mature_the_herb_plants(&mut w2, 10.0, &mut rng);
            if w2.tile(50, 40).is_some_and(|t| t.kind == 84) {
                bloomed = true;
                break;
            }
        }
        assert!(bloomed, "a seed blooms the blinkroot");
    }

    /// `Place2x2` (`:51330-51423`): the chillet egg's 2×2 write on a solid
    /// floor — frames k*18 / l*18, the clear-box gate.
    #[test]
    fn place_2x2_writes_the_chillet_egg() {
        let mut w = grass_world(200, 200, 100);
        place_2x2(&mut w, 100, 99, 752, 0);
        for k in 0..2 {
            for l in 0..2 {
                let t = w.tile(99 + k, 98 + l).unwrap();
                assert!(t.active() && t.kind == 752);
                assert_eq!(t.frame_x, (k * 18) as i16);
                assert_eq!(t.frame_y, (l * 18) as i16);
            }
        }
        // An occupied box refuses.
        let i = w.index(120, 98);
        w.tiles[i].kind = 1;
        w.tiles[i].flags |= FLAG_ACTIVE;
        place_2x2(&mut w, 120, 99, 752, 0);
        assert!(!w.tile(119, 98).is_some_and(|t| t.kind == 752));
    }

    /// The full pass (`:19981-19994`): dye plants land on the grass, the
    /// herb maturation runs, and the glow tulips / chillet eggs place on
    /// the valid ground.
    #[test]
    fn the_dye_plants_pass_dresses_the_world() {
        let mut w = grass_world(4200, 1200, 300);
        let mut rng = Rng::from_seed(11);
        pass_dye_plants(&mut w, 350.0, 600.0, false, &mut rng);
        let dye = w
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == 227)
            .count();
        assert!(dye > 0, "the dye plants planted ({dye})");
        // Every dye frame is a 34-multiple (the style columns).
        for t in w.tiles.iter().filter(|t| t.active() && t.kind == 227) {
            assert_eq!(t.frame_x % 34, 0, "the dye style frame");
            assert_eq!(t.frame_y, 0);
        }
    }
}
