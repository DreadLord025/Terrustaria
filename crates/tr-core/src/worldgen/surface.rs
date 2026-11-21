//! Surface dressing: slope tiles, trees, small flora and desert cacti.

use super::noise::Rng;
use super::terrain::{Layers, SurfaceMap};
use super::{GenOptions, TILE_GRASS, TILE_SAND, TILE_SNOW};
use crate::seeds::{MicroSeedFlags, SeedFlags};
use crate::tile::{FLAG_ACTIVE, FLAG_HALF_BRICK, Tile};
use crate::tiledata::{self as td, TILE_SOLID};
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

// --- W4-g38 C1: the Place Fallen Log pass ------------------------------------

/// `Skyblock.denySomeGeneration` (`WorldGen.cs:3133-3147`): skyblock with
/// none of the eight surface-rebuilding micro seeds and no extra living
/// trees denies the pass; every other world enters.
fn deny_some_generation(opts: &GenOptions) -> bool {
    if !opts.flags.contains(SeedFlags::SKYBLOCK) {
        return false;
    }
    let rebuilding = MicroSeedFlags::WORLD_IS_FROZEN
        | MicroSeedFlags::SURFACE_IS_DESERT
        | MicroSeedFlags::SURFACE_IS_MUSHROOMS
        | MicroSeedFlags::WORLD_IS_INFECTED
        | MicroSeedFlags::HALLOW_ON_THE_SURFACE
        | MicroSeedFlags::NO_INFECTION
        | MicroSeedFlags::EXTRA_FLOATING_ISLANDS
        | MicroSeedFlags::EXTRA_LIQUID;
    if opts.micro_flags.intersects(rebuilding) {
        return false;
    }
    !opts.micro_flags.contains(MicroSeedFlags::EXTRA_LIVING_TREES)
}

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
    // `!Skyblock.denySomeGeneration` (`:18515`).
    if deny_some_generation(opts) {
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
    if deny_some_generation(opts) {
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
        assert!(deny_some_generation(&sky));
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
        assert!(!deny_some_generation(&sky));
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
