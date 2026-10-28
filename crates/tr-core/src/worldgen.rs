//! Terraria-like world generation.
//!
//! This is a compact generator modelled on the shapes of `WorldGen`'s passes (surface
//! noise, dirt/stone layers, cave fields, ore blobs, tree growth) rather than a port of
//! all 45k lines. It is deterministic for a seed and produces a world the existing
//! framer/lighting/renderer can consume unchanged.
//!
//! Layout follows a small Terraria world: 4200x1200, forest surface around row ~360,
//! dirt wall backing, stone below the rock layer and an ash underworld.

use crate::tile::*;
use crate::tiledata as td;
use crate::tree;
use crate::world::World;

#[derive(Clone, Debug)]
pub struct GenOptions {
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub seed: u64,
}

impl Default for GenOptions {
    fn default() -> Self {
        Self {
            name: "Terrustaria".into(),
            width: 4200,
            height: 1200,
            seed: 1,
        }
    }
}

/// Deterministic 2D value noise with smooth interpolation.
struct Noise {
    seed: u64,
}

impl Noise {
    #[inline]
    fn hash(&self, x: i32, y: i32) -> u32 {
        let mut h = self.seed
            ^ (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ (y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
        h ^= h >> 30;
        h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        h ^= h >> 27;
        h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
        h ^= h >> 31;
        h as u32
    }

    #[inline]
    fn unit(&self, x: i32, y: i32) -> f32 {
        self.hash(x, y) as f32 / u32::MAX as f32
    }

    #[inline]
    fn value(&self, x: f32, y: f32) -> f32 {
        let x0 = x.floor();
        let y0 = y.floor();
        let fx = x - x0;
        let fy = y - y0;
        let sx = fx * fx * (3.0 - 2.0 * fx);
        let sy = fy * fy * (3.0 - 2.0 * fy);
        let (x0, y0) = (x0 as i32, y0 as i32);
        let a = self.unit(x0, y0);
        let b = self.unit(x0 + 1, y0);
        let c = self.unit(x0, y0 + 1);
        let d = self.unit(x0 + 1, y0 + 1);
        let top = a + (b - a) * sx;
        let bottom = c + (d - c) * sx;
        (top + (bottom - top) * sy) * 2.0 - 1.0
    }

    fn fbm(&self, x: f32, y: f32, octaves: u32, gain: f32) -> f32 {
        let mut amp = 1.0;
        let mut sum = 0.0;
        let mut norm = 0.0;
        let mut px = x;
        let mut py = y;
        for _ in 0..octaves {
            sum += self.value(px, py) * amp;
            norm += amp;
            amp *= gain;
            px *= 2.0;
            py *= 2.0;
        }
        sum / norm
    }
}

/// Small xorshift RNG, seeded from the world seed.
struct Rng(u64);

impl Rng {
    fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 32) as u32
    }

    fn range(&mut self, lo: i32, hi: i32) -> i32 {
        debug_assert!(hi > lo);
        lo + (self.next_u32() % (hi - lo) as u32) as i32
    }

    fn chance(&mut self, n: u32) -> bool {
        self.next_u32() % n == 0
    }
}

const TILE_DIRT: u16 = 0;
const TILE_STONE: u16 = 1;
const TILE_GRASS: u16 = 2;
const TILE_SNOW: u16 = 147;
const TILE_SAND: u16 = 53;
const TILE_SANDSTONE: u16 = 396;
const TILE_HARDENED_SAND: u16 = 397;
const TILE_IRON: u16 = 6;
const TILE_COPPER: u16 = 7;
const TILE_GOLD: u16 = 8;
const TILE_SILVER: u16 = 9;
const TILE_ASH: u16 = 57;
const TILE_HELLSTONE: u16 = 58;
const WALL_STONE: u16 = 1;
const WALL_DIRT: u16 = 2;
const WALL_SANDSTONE: u16 = 187;

pub fn generate(opts: &GenOptions) -> World {
    let width = opts.width.max(512);
    let height = opts.height.max(512);
    let mut world = World::new(opts.name.clone(), width, height);
    world.frame_important = vec![false; td::TILE_COUNT];
    world.frame_important[tree::TREE as usize] = true;
    world.frame_important[3] = true;
    world.frame_important[4] = true;

    let noise = Noise { seed: opts.seed };
    let mut rng = Rng(opts.seed ^ 0xA076_1D64_78BD_642F);

    // --- surface height map -------------------------------------------------
    let avg = height as f32 * 0.30;
    let spawn_center = width / 2;
    // An inland desert west of spawn (small worlds skip it). The band is kept clear
    // of the spawn plateau and the lake.
    let desert_half = 170;
    let desert_center = spawn_center - 380;
    let desert_enabled = width >= 2000;
    let desert_start = if desert_enabled {
        (desert_center - desert_half).max(200)
    } else {
        0
    };
    let desert_end = if desert_enabled {
        (desert_center + desert_half).min(spawn_center - 120)
    } else {
        0
    };
    let in_desert = |x: i32| desert_enabled && x >= desert_start && x < desert_end;
    let mut surface = vec![0i32; width as usize];
    for x in 0..width {
        let fx = x as f32;
        // Large rolling hills plus small roughness.
        let hills = noise.fbm(fx * 0.0028, 11.0, 4, 0.5);
        let detail = noise.fbm(fx * 0.018, 71.0, 3, 0.5);
        let mut h = avg + hills * 42.0 + detail * 5.0;
        // Gentle spawn plateau so the player starts on flat ground.
        let dist = (x - spawn_center).abs() as f32;
        if dist < 45.0 {
            let t = 1.0 - dist / 45.0;
            h = h * (1.0 - t * 0.7) + avg * t * 0.7;
        }
        surface[x as usize] = (h.round() as i32).clamp(80, height - 340);
    }
    // Flatten the desert into smooth dunes so it reads differently from the forest.
    if desert_enabled {
        for x in (desert_start - 30)..(desert_end + 30) {
            if x < 0 || x >= width {
                continue;
            }
            let d = (x - desert_center).abs();
            let blend = if d <= desert_half {
                1.0
            } else {
                1.0 - (d - desert_half) as f32 / 30.0
            };
            let dune =
                avg + (x as f32 * 0.045).sin() * 5.0 + noise.value(x as f32 * 0.02, 9.0) * 4.0;
            surface[x as usize] = (surface[x as usize] as f32 * (1.0 - blend)
                + dune.clamp(80.0, (height - 340) as f32) * blend)
                .round() as i32;
        }
    }
    // Limit adjacent height differences to one tile so the whole surface can be
    // expressed with slope tiles instead of vertical stair-steps.
    for _ in 0..3 {
        for x in 1..width as usize {
            let prev = surface[x - 1];
            surface[x] = surface[x].clamp(prev - 1, prev + 1);
        }
        for x in (0..width as usize - 1).rev() {
            let next = surface[x + 1];
            surface[x] = surface[x].clamp(next - 1, next + 1);
        }
    }

    // A small forest lake near spawn: a smooth bowl with a raised rim so the water
    // stays enclosed, and a water surface a little below the surrounding ground.
    let lake_center = (spawn_center + 150).min(width - 80);
    let lake_half = 32;
    let lake_level = (avg as i32 + 5).clamp(90, height - 300);
    for dx in -(lake_half + 3)..=(lake_half + 3) {
        let x = lake_center + dx;
        if x < 2 || x >= width - 2 {
            continue;
        }
        if dx.abs() <= lake_half {
            let t = dx as f32 / lake_half as f32;
            let depth = (1.0 - t * t) * 8.0;
            surface[x as usize] = lake_level + depth.round() as i32;
        } else {
            surface[x as usize] = surface[x as usize].min(lake_level - 2);
        }
    }

    // Ocean beaches at both edges: a sand ramp down from the sea level into deep water.
    let beach = 140;
    let sea_level = (avg as i32 + 4).clamp(90, height - 300);
    let inner = beach + 30;
    for (x, d) in (0..inner)
        .map(|x| (x, x))
        .chain((width - inner..width).map(|x| (x, width - 1 - x)))
    {
        if x < 0 || x >= width {
            continue;
        }
        let t = 1.0 - d as f32 / inner as f32;
        let ramp = sea_level + (t * t * 30.0) as i32;
        if d < beach {
            surface[x as usize] = ramp;
        } else {
            let k = (inner - d) as f32 / 30.0;
            surface[x as usize] =
                (surface[x as usize] as f32 * (1.0 - k) + ramp as f32 * k).round() as i32;
        }
    }

    // --- terrain fill -------------------------------------------------------
    let avg_surface = surface.iter().sum::<i32>() as f64 / width as f64;
    let rock_layer = (height as f64 * 0.43).max(avg_surface + 90.0);
    let hell_top = (height - 170) as i32;
    // Eastern third of the world is a snow biome.
    let snow_start = width * 2 / 3;

    for x in 0..width {
        let top = surface[x as usize];
        let dirt_depth = 16 + (noise.value(x as f32 * 0.05, 3.0) * 6.0) as i32;
        for y in 0..height {
            let i = world.index(x, y);
            let mut tile = Tile::default();
            let desert = in_desert(x);
            if y >= top {
                if y == top {
                    tile.kind = if x < beach + 24 || x >= width - beach - 24 {
                        TILE_SAND
                    } else if x >= snow_start {
                        TILE_SNOW
                    } else if desert {
                        TILE_SAND
                    } else {
                        TILE_GRASS
                    };
                } else if y < top + dirt_depth {
                    tile.kind = if desert { TILE_SAND } else { TILE_DIRT };
                } else if desert && y < top + dirt_depth + 14 {
                    tile.kind = TILE_HARDENED_SAND;
                } else if desert && y < top + dirt_depth + 30 {
                    tile.kind = TILE_SANDSTONE;
                } else if y < hell_top {
                    tile.kind = TILE_STONE;
                } else {
                    tile.kind = TILE_ASH;
                }
                tile.flags |= FLAG_ACTIVE;
                if y > top + 2 {
                    tile.wall = if desert {
                        WALL_SANDSTONE
                    } else if (y as f64) < rock_layer {
                        WALL_DIRT
                    } else {
                        WALL_STONE
                    };
                }
            }
            world.tiles[i] = tile;
        }
    }

    // --- caves --------------------------------------------------------------
    for x in 0..width {
        let top = surface[x as usize] + 9;
        for y in top..hell_top {
            let fx = x as f32;
            let fy = y as f32;
            let large = noise.fbm(fx * 0.018, fy * 0.030, 3, 0.5);
            let tunnel = noise.fbm(fx * 0.055 + 40.0, fy * 0.045, 3, 0.5);
            let cavern = large > 0.30;
            let worm = tunnel.abs() < 0.035;
            // Keep a crust under the surface and fade caves out near hell.
            let depth = (y - surface[x as usize]) as f32;
            let fade = (depth / 24.0).min(1.0);
            if (cavern || worm) && fade > 0.15 && y < hell_top - 10 {
                let i = world.index(x, y);
                world.tiles[i].flags &= !FLAG_ACTIVE;
            }
        }
    }

    // Dirt pockets inside stone and stone outcrops inside dirt.
    blob_pass(
        &mut world, &mut rng, width, height, TILE_STONE, TILE_DIRT, 900, 10,
    );
    blob_pass(
        &mut world, &mut rng, width, height, TILE_DIRT, TILE_STONE, 500, 6,
    );
    blob_pass(
        &mut world, &mut rng, width, height, TILE_ASH, TILE_STONE, 400, 8,
    );

    // --- ores ---------------------------------------------------------------
    let ores: [(u16, i32, i32, usize, i32); 4] = [
        (TILE_COPPER, 20, 400, 1400, 4),
        (TILE_IRON, 60, 600, 1100, 4),
        (TILE_SILVER, 160, 800, 800, 3),
        (TILE_GOLD, 280, 1000, 600, 3),
    ];
    for (kind, y0, y1, count, size) in ores {
        for _ in 0..count {
            let x = rng.range(10, width - 10);
            let y = surface[x as usize] + rng.range(y0, y1);
            if y >= hell_top {
                continue;
            }
            ore_blob(&mut world, x, y, kind, rng.range(2, size + 3), &mut rng);
        }
    }
    // Hellstone in the ash layer.
    for _ in 0..900 {
        let x = rng.range(10, width - 10);
        let y = rng.range(hell_top, height - 4);
        ore_blob(&mut world, x, y, TILE_HELLSTONE, rng.range(2, 5), &mut rng);
    }

    // --- liquids ------------------------------------------------------------
    // Lake: fill the bowl above the solid floor and turn the floor to dirt.
    for dx in -(lake_half + 1)..=(lake_half + 1) {
        let x = lake_center + dx;
        if x < 1 || x >= width - 1 {
            continue;
        }
        let top = surface[x as usize];
        if top >= 1 && top < height {
            let i = world.index(x, top);
            if world.tiles[i].active() && world.tiles[i].kind == TILE_GRASS {
                world.tiles[i].kind = TILE_DIRT;
            }
        }
        for y in lake_level..top {
            let i = world.index(x, y);
            if !world.tiles[i].active() {
                world.tiles[i].liquid = 255;
                world.tiles[i].flags &= !(FLAG_LIQUID_LAVA | FLAG_LIQUID_HONEY);
            }
        }
    }
    // Ocean water at both shores.
    for shore_x in (0..beach).chain(width - beach..width) {
        if shore_x < 0 || shore_x >= width {
            continue;
        }
        let top = surface[shore_x as usize];
        for y in sea_level..top {
            let i = world.index(shore_x, y);
            if !world.tiles[i].active() {
                world.tiles[i].liquid = 255;
                world.tiles[i].flags &= !(FLAG_LIQUID_LAVA | FLAG_LIQUID_HONEY);
            }
        }
    }
    // Lava puddles on cave floors.
    for _ in 0..700 {
        let x = rng.range(10, width - 10);
        let y = rng.range((avg_surface as i32) + 110, hell_top - 4);
        let floor_solid = world
            .tile(x, y + 1)
            .is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind));
        let air_here = world
            .tile(x, y)
            .is_some_and(|t| !t.active() && t.liquid == 0);
        if !floor_solid || !air_here {
            continue;
        }
        let depth = rng.range(1, 4);
        for d in 0..depth {
            let yy = y - d;
            if yy < 1 {
                break;
            }
            let i = world.index(x, yy);
            if world.tiles[i].active() || world.tiles[i].liquid != 0 {
                break;
            }
            world.tiles[i].liquid = 255;
            world.tiles[i].flags |= FLAG_LIQUID_LAVA;
        }
    }

    // --- surface slopes -----------------------------------------------------
    // With height differences limited to one tile, every height change becomes a
    // slope tile on the higher side instead of a vertical step.
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
            world.tiles[i].slope = 1;
        } else if left_lower {
            world.tiles[i].slope = 2;
        }
    }

    // --- trees --------------------------------------------------------------
    plant_trees(&mut world, &mut rng, &surface);

    // --- surface plants -----------------------------------------------------
    // Tile 3 contains the short grass, flowers and mushrooms the original scatters
    // over forest grass. Frames are explicit, so the tile is flagged important.
    world.frame_important[3] = true;
    for x in 1..width - 1 {
        let top = surface[x as usize];
        let ground = *world.tile(x, top).unwrap_or(&Tile::default());
        if !ground.active() || ground.kind != TILE_GRASS || ground.slope != 0 {
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

    // --- desert cacti -------------------------------------------------------
    // Tile 80 has its own 7x3 frame grid, so it is flagged important and framed by
    // hand: y=0 top, y=18 middle, y=36 base.
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
                .is_some_and(|t| t.active() && t.kind == TILE_SAND && t.slope == 0);
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

    world.world_surface = avg_surface;
    world.rock_layer = rock_layer;
    world.spawn_x = spawn_center;
    world.spawn_y = (surface[spawn_center as usize] - 6).max(1);
    world.time = 27000.0; // noon
    world.day_time = true;
    world
}

fn blob_pass(
    world: &mut World,
    rng: &mut Rng,
    width: i32,
    height: i32,
    from: u16,
    to: u16,
    count: usize,
    radius: i32,
) {
    for _ in 0..count {
        let x = rng.range(4, width - 4);
        let y = rng.range(height / 3, height - 4);
        let r = rng.range(1, radius);
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy > r * r {
                    continue;
                }
                let (tx, ty) = (x + dx, y + dy);
                if !world.in_bounds(tx, ty) {
                    continue;
                }
                let i = world.index(tx, ty);
                let t = &mut world.tiles[i];
                if t.active() && t.kind == from {
                    t.kind = to;
                }
            }
        }
    }
}

fn ore_blob(world: &mut World, x: i32, y: i32, kind: u16, size: i32, rng: &mut Rng) {
    let mut cx = x;
    let mut cy = y;
    for _ in 0..size {
        if world.in_bounds(cx, cy) {
            let i = world.index(cx, cy);
            let t = &mut world.tiles[i];
            if t.active() && matches!(t.kind, TILE_STONE | TILE_ASH | TILE_DIRT) {
                t.kind = kind;
            }
        }
        cx += rng.range(-1, 2);
        cy += rng.range(-1, 2);
    }
}

fn plant_trees(world: &mut World, rng: &mut Rng, surface: &[i32]) {
    // Default trunk pieces, then the branch-bearing variants from `GrowTree`.
    const STRAIGHT: [(i16, i16); 3] = [(0, 0), (0, 66), (22, 0)];
    const BRANCHY: [(i16, i16); 8] = [
        (0, 0),
        (0, 66),
        (22, 0),
        (22, 66),
        (44, 66),
        (66, 66),
        (88, 0),
        (110, 66),
    ];
    let width = world.width;
    let mut last = -20;
    let mut x = 12;
    while x < width - 12 {
        // Forest density, like vanilla: a tree every 6-14 columns with gaps.
        x += rng.range(5, 15);
        if x >= width - 12 {
            break;
        }
        if x - last < 5 {
            continue;
        }
        let top = surface[x as usize];
        let ground = *world.tile(x, top).unwrap_or(&Tile::default());
        if !(ground.active() && (ground.kind == TILE_GRASS || ground.kind == TILE_SNOW))
            || ground.slope != 0
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

        // `rows[dy - 1]` is the trunk tile at y = top - dy; the top row is replaced
        // by the canopy marker below.
        let mut rows = vec![(0i16, 0i16); height as usize];
        for (dy, frame) in rows.iter_mut().enumerate() {
            if dy == 0 {
                *frame = STRAIGHT[rng.range(0, 3) as usize];
            } else {
                // Mostly straight wood; the branch-bearing variants are visual noise
                // if used on every row (vanilla draws them on ~3 rows per tree).
                *frame = if rng.chance(10) {
                    BRANCHY[rng.range(0, BRANCHY.len() as i32) as usize]
                } else {
                    STRAIGHT[rng.range(0, 3) as usize]
                };
            }
        }
        let top_y = top - height;
        rows[height as usize - 1] = (22, 198 + rng.range(0, 3) as i16 * 22);

        // Leafy branch markers, each paired with a branch-bearing trunk piece.
        let branches = rng.range(1, 5);
        let mut used = Vec::new();
        for _ in 0..branches {
            let row = rng.range(3, height - 1);
            if used.contains(&row) {
                continue;
            }
            used.push(row);
            let by = top - row;
            rows[(row - 1) as usize] = BRANCHY[rng.range(0, BRANCHY.len() as i32) as usize];
            let sides: &[i32] = if rng.chance(3) { &[-1, 1] } else { &[-1] };
            for &side in sides {
                let bx = x + side;
                if bx < 1 || bx >= width - 1 || by <= top_y || by >= top {
                    continue;
                }
                if world.tile(bx, by).is_some_and(|t| t.active()) {
                    continue;
                }
                let i = world.index(bx, by);
                world.tiles[i] = Tile {
                    kind: tree::TREE,
                    frame_x: if side < 0 { 44 } else { 66 },
                    frame_y: 198 + rng.range(0, 3) as i16 * 22,
                    flags: FLAG_ACTIVE,
                    ..Default::default()
                };
            }
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
        // Forked base trunk pieces (the game adds these next to the lowest rows).
        for side in [-1i32, 1i32] {
            if !rng.chance(3) {
                continue;
            }
            let bx = x + side;
            if bx < 1 || bx >= width - 1 {
                continue;
            }
            let by = top - 1;
            if world.tile(bx, by).is_some_and(|t| t.active()) {
                continue;
            }
            let i = world.index(bx, by);
            world.tiles[i] = Tile {
                kind: tree::TREE,
                frame_x: if side < 0 { 44 } else { 66 },
                frame_y: 132 + rng.range(0, 3) as i16 * 22,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_for_seed() {
        let opts = GenOptions {
            width: 512,
            height: 512,
            seed: 42,
            ..Default::default()
        };
        let a = generate(&opts);
        let b = generate(&opts);
        assert_eq!(a.tiles.len(), b.tiles.len());
        for (x, y) in a.tiles.iter().zip(&b.tiles) {
            assert_eq!(
                (x.kind, x.wall, x.flags, x.frame_x, x.frame_y),
                (y.kind, y.wall, y.flags, y.frame_x, y.frame_y)
            );
        }
    }

    #[test]
    fn desert_band_has_sand_dunes_and_no_trees() {
        let opts = GenOptions::default();
        let mut w = generate(&opts);
        w.compute_column_surface();
        let center = w.width / 2;
        let start = center - 380 - 170 + 20;
        let end = center - 380 + 170 - 20;
        let mut saw_hardened = false;
        let mut saw_sandstone = false;
        for x in start..end {
            let top = w.surface_of(x);
            let t = w.tile(x, top).unwrap();
            assert!(
                t.kind == TILE_SAND,
                "desert surface at x{x} is kind {}",
                t.kind
            );
            for y in top..top + 40 {
                let k = w.tile(x, y).unwrap().kind;
                saw_hardened |= k == TILE_HARDENED_SAND;
                saw_sandstone |= k == TILE_SANDSTONE;
            }
            for y in top..top + 30 {
                assert!(
                    !(w.tile(x, y).unwrap().active() && w.tile(x, y).unwrap().kind == tree::TREE),
                    "tree inside the desert at {x},{y}"
                );
            }
        }
        assert!(saw_hardened && saw_sandstone, "missing desert strata");
        let mut cacti = 0;
        for x in start..end {
            let top = w.surface_of(x);
            for y in top.saturating_sub(5)..top {
                let t = w.tile(x, y).unwrap();
                if t.active() && t.kind == 80 {
                    cacti += 1;
                    assert!(t.frame_y >= 0, "cactus must have a manual frame");
                    assert!(w.frame_important[80]);
                }
            }
        }
        assert!(cacti > 5, "only {cacti} cactus tiles");
    }

    #[test]
    fn has_surface_trees_and_caves() {
        let opts = GenOptions {
            width: 1024,
            height: 600,
            seed: 7,
            ..Default::default()
        };
        let w = generate(&opts);
        assert!(w.tiles.iter().any(|t| t.active() && t.kind == TILE_GRASS));
        assert!(w.tiles.iter().any(|t| t.active() && t.kind == tree::TREE));
        // Some air below the average surface means caves were carved.
        let mut underground_air = 0;
        for y in (w.world_surface as i32)..(w.height - 200) {
            for x in 20..w.width - 20 {
                if !w.tiles[w.index(x, y)].active() {
                    underground_air += 1;
                }
            }
        }
        assert!(underground_air > 1000, "caves {underground_air}");
        assert!(
            w.tiles.iter().any(|t| t.liquid > 0),
            "generator should place a lake and lava pools"
        );
        assert!(
            w.tiles.iter().any(|t| t.active() && t.kind == TILE_SAND),
            "generator should place ocean beaches"
        );
        assert!(
            w.tiles.iter().any(|t| t.active() && t.kind == TILE_SNOW),
            "generator should place a snow biome"
        );
    }
}
