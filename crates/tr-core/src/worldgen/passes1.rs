//! W4-d101 — the first slice of the unexecuted vanilla passes
//! (`docs/research/12b-d101-passes-1.md`): Sand Patches, Mount Caves, Dirt
//! Wall Backgrounds, Clay, Small Holes, Dirt Layer Caves, Rock Layer Caves.
//! Each body is the verbatim `WorldGen.AddPasses` registration block on its
//! own `pass_rng("<vanilla name>")` stream; the ice biome (`:12312`) lives in
//! `ice_biome.rs` and Granite already runs in `biomes.rs::pass_granite`.

use super::noise::Rng;
use super::pipeline::GenVars;
use super::terrain::Layers;
use super::tile_runner::{tile_runner, TileRunnerOpts};
use super::{TILE_DIRT, TILE_SAND};
use crate::seeds::SeedFlags;
use crate::tile::FLAG_ACTIVE;
use crate::world::World;

/// The `Sand Patches` pass (`WorldGen.cs:11689-11717`): `maxTilesX * 0.013`
/// sand-53 TileRunner blobs in `[worldSurface, rockLayer)` (remix: a quarter
/// of the count, the band `[rockLayer - 100, maxTilesY - 350)`), with the
/// spawn-middle rejection loop — a blob whose column sits in `(0.46, 0.54)`
/// of the width AND above `worldSurface + 150` re-rolls both coordinates
/// (the re-roll always uses the CLASSIC band, even in remix — the C#'s
/// `:11711-11712` keeps the non-remix `Next` there; kept verbatim).
/// Draw order per blob: x → y → (rejections) → strength `Next(15, 70)` →
/// steps `Next(20, 130)` → the runner's own ledger.
pub(crate) fn pass_sand_patches(world: &mut World, rng: &mut Rng, layers: &Layers) {
    let (width, height) = (world.width, world.height);
    let remix = world.seeds.contains(SeedFlags::REMIX);
    let world_surface = layers.world_surface;
    let rock_layer = layers.rock_layer;
    // `int num = (int)((double)Main.maxTilesX * 0.013)` (`:11695`).
    let mut num = (width as f64 * 0.013) as i32;
    if remix {
        num /= 4;
    }
    // The remix band's upper bound can fall below its lower bound on a
    // sub-vanilla test world (the C# throws there; vanilla never generates
    // below 4200×1200) — the empty band collapses to `lo` without a draw,
    // the `rng.range` empty-range convention (`rng.rs:67-73`).
    let remix_lo = rock_layer as i32 - 100;
    let remix_hi = (height - 350).max(remix_lo);
    for _ in 0..num {
        let mut x = rng.range(0, width);
        let mut y = if remix {
            rng.range(remix_lo, remix_hi)
        } else {
            rng.range(world_surface as i32, rock_layer as i32)
        };
        // The spawn-middle rejection (`:11709-11713`).
        while (x as f64) > width as f64 * 0.46
            && (x as f64) < width as f64 * 0.54
            && (y as f64) < world_surface + 150.0
        {
            x = rng.range(0, width);
            y = rng.range(world_surface as i32, rock_layer as i32);
        }
        // `TileRunner(num2, num3, Next(15, 70), Next(20, 130), 53)` (`:11714`)
        // — named-argument evaluation order: strength before steps.
        let strength = rng.range(15, 70);
        let steps = rng.range(20, 130);
        tile_runner(
            world,
            rng,
            x,
            y,
            strength as f64,
            steps,
            TILE_SAND as i32,
            world_surface,
            &TileRunnerOpts::for_world(world),
        );
    }
}

/// The `Mount Caves` pass (`GenPassNameID.MountainCaves`,
/// `WorldGen.cs:11784-11857`): `maxTilesX * 0.001` dirt mountains raised on
/// the surface (remix: ×1.5). Per mountain: the column rolls inside
/// `[0.25, 0.75)·width`, re-rolled out of the ±90 spawn middle (non-remix)
/// and away from every placed anchor (<100 tiles); `maxTilesX / 5` failed
/// spacing tries abandon the mountain. The column then scans from the sky
/// for its first active tile and refuses when any sand 53 / sandstone 151 /
/// gemspark 274 sits in the ±50×±25 window around it. A clean spot calls
/// `Mountinater` and publishes `GenVars.mCaveX/mCaveY[numMCaves]`.
pub(crate) fn pass_mount_caves(world: &mut World, gv: &mut GenVars, rng: &mut Rng, layers: &Layers) {
    let (width, height) = (world.width, world.height);
    let remix = world.seeds.contains(SeedFlags::REMIX);
    let world_surface = layers.world_surface;
    gv.num_m_caves = 0; // `:11788`
    // `int num = (int)((double)Main.maxTilesX * 0.001)` (`:11791-11795`).
    let mut num = (width as f64 * 0.001) as i32;
    if remix {
        num = (num as f64 * 1.5) as i32;
    }
    let (q_lo, q_hi) = ((width as f64 * 0.25) as i32, (width as f64 * 0.75) as i32);
    for _ in 0..num {
        let mut fails = 0;
        let mut abandoned = false;
        let mut spaced = false;
        let mut x = rng.range(q_lo, q_hi);
        while !spaced {
            spaced = true;
            // The spawn-middle exclusion (`:11806-11812`), classic only.
            if !remix {
                while x > width / 2 - 90 && x < width / 2 + 90 {
                    x = rng.range(q_lo, q_hi);
                }
            }
            // The <100 spacing gate against the placed anchors
            // (`:11813-11821`) — the fail counter accumulates across tries.
            for j in 0..gv.num_m_caves as usize {
                if (x - gv.m_cave_x[j]).abs() < 100 {
                    fails += 1;
                    spaced = false;
                    break;
                }
            }
            if fails >= width / 5 {
                abandoned = true;
                break;
            }
        }
        if abandoned {
            continue;
        }
        // The sky scan + the sand/gemstone proximity refusal
        // (`:11830-11853`): the FIRST active tile of the column decides.
        let mut refused = false;
        for k in 0..world_surface as i32 {
            if !world.tile(x, k).is_some_and(|t| t.active()) {
                continue;
            }
            for l in x - 50..x + 50 {
                for m in k - 25..k + 25 {
                    // `WorldGen.InWorld(l, m)` (`:9688-9695`, fluff 0).
                    if l < 0 || l >= width || m < 0 || m >= height {
                        continue;
                    }
                    if world
                        .tile(l, m)
                        .is_some_and(|t| t.active() && matches!(t.kind, 53 | 151 | 274))
                    {
                        refused = true;
                    }
                }
            }
            if !refused {
                mountinater(world, rng, x, k, remix);
                gv.m_cave_x[gv.num_m_caves as usize] = x;
                gv.m_cave_y[gv.num_m_caves as usize] = k;
                gv.num_m_caves += 1;
            }
            break; // the C# breaks on the first active tile either way
        }
    }
}

/// `WorldGen.Mountinater` (`WorldGen.cs:79486-79560`): the dirt-mountain
/// raiser — a shrinking circle (NOT the Manhattan runner) climbs from just
/// below the surface tile, painting dirt 0 into inactive cells only. Draw
/// order: `Next(80,120)` steps → `Next(40,55)` height → the two velocity
/// draws → per iteration `Next(4)` shrink + `Next(80,120)` radius wobble +
/// the two velocity drifts. Remix widens both budgets ×1.5 (`:79491-79495`).
fn mountinater(world: &mut World, rng: &mut Rng, i: i32, j: i32, remix: bool) {
    let mut num = rng.range(80, 120) as f64;
    let mut num3 = rng.range(40, 55) as f64;
    if remix {
        num *= 1.5;
        num3 *= 1.5;
    }
    let (mut cx, mut cy) = (i as f64, j as f64 + num3 / 2.0);
    let mut vx = rng.range(-10, 11) as f64 * 0.1;
    let mut vy = rng.range(-20, -10) as f64 * 0.1;
    while num > 0.0 && num3 > 0.0 {
        num -= rng.range(0, 4) as f64;
        num3 -= 1.0;
        let mut left = (cx - num * 0.5) as i32;
        let mut right = (cx + num * 0.5) as i32;
        let mut top = (cy - num * 0.5) as i32;
        let mut bottom = (cy + num * 0.5) as i32;
        // The clamps are the C#'s (`:79510-79525`): 0 and the FULL extents
        // (the tile getter guards the == edge rows).
        left = left.max(0);
        right = right.min(world.width);
        top = top.max(0);
        bottom = bottom.min(world.height);
        let num2 = num * rng.range(80, 120) as f64 * 0.01;
        for k in left..right {
            for l in top..bottom {
                let dx = (k as f64 - cx).abs();
                let dy = (l as f64 - cy).abs();
                if (dx * dx + dy * dy).sqrt() < num2 * 0.4
                    && world.tile(k, l).is_some_and(|t| !t.active())
                {
                    if let Some(t) = world.tile_mut(k, l) {
                        t.flags |= FLAG_ACTIVE;
                        t.kind = TILE_DIRT;
                    }
                }
            }
        }
        cx += vx;
        cy += vy;
        vx += rng.range(-10, 11) as f64 * 0.05;
        vy += rng.range(-10, 11) as f64 * 0.05;
        vx = vx.clamp(-0.5, 0.5);
        // The Y clamps keep the upward drift (`:79551-79558`).
        if vy > -0.5 {
            vy = -0.5;
        }
        if vy < -1.5 {
            vy = -1.5;
        }
    }
}

/// The `Dirt Wall Backgrounds` pass (`WorldGen.cs:11858-11895`): the
/// per-column wall sweep from the sky down to `worldSurface + num` where
/// `num` is a clamped `Next(-1, 2)` random walk (0..10). `wall` tracks the
/// last active tile's kind — dirt 2, or snow 40 under a snow tile 147 —
/// and paints every cell once the 2×3-solid `flag` latch has tripped
/// (the cell itself plus its left/right neighbours, both rows). A jungle
/// wall 64 is never overwritten. The write order per cell is the C#'s:
/// kind read → wall paint → latch test (the latch trips only AFTER the
/// current row was painted, so the first fully-solid row keeps its old
/// wall).
pub(crate) fn pass_dirt_wall_backgrounds(world: &mut World, rng: &mut Rng, layers: &Layers) {
    let width = world.width;
    let world_surface = layers.world_surface;
    let mut num = 0;
    for i in 1..width - 1 {
        let mut wall: u16 = 2;
        let mut flag = false;
        // `num += Next(-1, 2)` clamped to [0, 10] (`:11869-11877`).
        num += rng.range(-1, 2);
        num = num.clamp(0, 10);
        let mut j = 0;
        // `j < worldSurface + 10 && !(j > worldSurface + num)` (`:11878`).
        while (j as f64) < world_surface + 10.0 && !((j as f64) > world_surface + num as f64) {
            let active = world.tile(i, j).is_some_and(|t| t.active());
            if active {
                wall = if world.tile(i, j).is_some_and(|t| t.kind != 147) {
                    2
                } else {
                    40
                };
            }
            if flag && world.tile(i, j).is_some_and(|t| t.wall != 64) {
                if let Some(t) = world.tile_mut(i, j) {
                    t.wall = wall;
                }
            }
            // The 2×3-solid latch (`:11888`): (i-1..=i+1) × (j..=j+1).
            let solid = |x: i32, y: i32| world.tile(x, y).is_some_and(|t| t.active());
            if solid(i, j)
                && solid(i - 1, j)
                && solid(i + 1, j)
                && solid(i, j + 1)
                && solid(i - 1, j + 1)
                && solid(i + 1, j + 1)
            {
                flag = true;
            }
            j += 1;
        }
    }
}

/// The `Clay` pass (`WorldGen.cs:11959-12007`): three clay-40 TileRunner
/// bands — `area * 2E-05` blobs above `worldSurfaceLow`, `area * 5E-05` in
/// `[worldSurfaceLow, worldSurfaceHigh + 1)`, `area * 2E-05` in
/// `[worldSurfaceHigh, rockLayerHigh + 1)` (remix replaces the last two
/// with one `area * 7E-05` band `[rockLayer - 25, maxTilesY - 350)`) — then
/// the surface tidy sweep: under every column's first active tile above
/// `worldSurface - 1`, clay in the next five rows reverts to dirt
/// (`:11989-12005`). The runner's own `case 53` protection keeps clay out
/// of sand (`tile_runner.rs`, `WorldGen.cs:76934`). Draw order per blob:
/// x → y → strength → steps.
pub(crate) fn pass_clay(world: &mut World, rng: &mut Rng, layers: &Layers) {
    let (width, height) = (world.width, world.height);
    let remix = world.seeds.contains(SeedFlags::REMIX);
    let world_surface = layers.world_surface;
    let area = (width * height) as f64;
    let surface_low = layers.surface_low as i32;
    let surface_high = layers.surface_high as i32;
    let rock_high = layers.rock_high as i32;
    // (y-band lo/hi, strength lo/hi, steps lo/hi, density) — the C#'s three
    // loops (`:11964-11987`); the hi bounds keep the C#'s +1s verbatim.
    let bands: [(i32, i32, i32, i32, i32, i32, f64); 3] = if remix {
        let lo = layers.rock_layer as i32 - 25;
        [
            (0, surface_low, 4, 14, 10, 50, 2e-5),
            (lo, (height - 350).max(lo), 8, 15, 5, 50, 7e-5),
            (0, 0, 0, 0, 0, 0, 0.0), // the third loop does not exist in remix
        ]
    } else {
        [
            (0, surface_low, 4, 14, 10, 50, 2e-5),
            (surface_low, surface_high + 1, 8, 14, 15, 45, 5e-5),
            (surface_high, rock_high + 1, 8, 15, 5, 50, 2e-5),
        ]
    };
    for &(y0, y1, s0, s1, t0, t1, density) in &bands {
        let count = (area * density) as i32;
        for _ in 0..count {
            let x = rng.range(0, width);
            let y = rng.range(y0, y1);
            let strength = rng.range(s0, s1);
            let steps = rng.range(t0, t1);
            tile_runner(
                world,
                rng,
                x,
                y,
                strength as f64,
                steps,
                40,
                world_surface,
                &TileRunnerOpts::for_world(world),
            );
        }
    }
    // The surface tidy sweep (`:11989-12005`): the first active tile of
    // each column (scanning rows 1..worldSurface-1) clears clay in the
    // five rows starting AT it. No RNG.
    for m in 5..width - 5 {
        for n in 1..world_surface as i32 - 1 {
            if world.tile(m, n).is_some_and(|t| t.active()) {
                for num in n..n + 5 {
                    if let Some(t) = world.tile_mut(m, num) {
                        if t.kind == 40 {
                            t.kind = TILE_DIRT;
                        }
                    }
                }
                break;
            }
        }
    }
}

/// `GenVars.smallHolesBeachAvoidance` (`WorldGen.cs:11204`):
/// `beachSandRandomCenter + 20` = (275 + 5 + 40) + 20 = 340
/// (`:11191-11192`). Shared by Small Holes and Dirt Layer Caves.
const SMALL_HOLES_BEACH_AVOIDANCE: i32 = 340;

/// The `Small Holes` pass (`WorldGen.cs:12008-12066`): `area * 0.0015`
/// PAIRED diggers — one `Next(5) == 0` lottery per pair picks the type for
/// BOTH runners (-2 water fill on a hit, -1 air otherwise), then the small
/// (2..5, 2..20) and the large (8..15, 7..30) runners each roll their own
/// point through the beach + spawn-middle rejection loop. The
/// tenth-anniversary arm's `while` condition (`:12027`) is the C#'s
/// verbatim `x < 0.2·w && x > 0.8·w` — unsatisfiable, so the loop never
/// re-rolls; kept as the literal condition here (never true). Remix
/// shrinks the runners below the rock line (×0.8/×0.9 small, ×0.7/×0.9
/// large). Draw order per pair: `Next(5)` → x → y → rejections → strength
/// → steps → runner ledger → x → y → rejections → strength → steps →
/// runner ledger.
pub(crate) fn pass_small_holes(
    world: &mut World,
    rng: &mut Rng,
    layers: &Layers,
    gv: &GenVars,
) {
    let (width, height) = (world.width, world.height);
    let flags = world.seeds;
    let remix = flags.contains(SeedFlags::REMIX);
    let anniversary = flags.contains(SeedFlags::TENTH_ANNIVERSARY);
    let world_surface = layers.world_surface;
    let surface_high = layers.surface_high;
    let rock_layer = layers.rock_layer;
    let count = ((width * height) as f64 * 0.0015) as i32;
    for _ in 0..count {
        // The per-pair type lottery (`:12018-12022`).
        let kind = if rng.range(0, 5) == 0 { -2 } else { -1 };
        // --- the small runner (`:12023-12048`) ---
        let mut x = rng.range(0, width);
        let mut y = rng.range(surface_high as i32, height);
        if !remix && anniversary {
            // The C#'s unsatisfiable condition, verbatim (`:12027`).
            while (x as f64) < width as f64 * 0.2
                && (x as f64) > width as f64 * 0.8
                && (y as f64) < world_surface
            {
                x = rng.range(0, width);
                y = rng.range(surface_high as i32, height);
            }
        } else {
            while small_holes_rejected(x, y, width, surface_high, world_surface) {
                x = rng.range(0, width);
                y = rng.range(surface_high as i32, height);
            }
        }
        let mut strength = rng.range(2, 5);
        let mut steps = rng.range(2, 20);
        if remix && (y as f64) > rock_layer {
            strength = (strength as f64 * 0.8) as i32;
            steps = (steps as f64 * 0.9) as i32;
        }
        dig(world, rng, x, y, strength, steps, kind, layers, gv);
        // --- the large runner (`:12049-12063`) ---
        let mut x = rng.range(0, width);
        let mut y = rng.range(surface_high as i32, height);
        while small_holes_rejected(x, y, width, surface_high, world_surface) {
            x = rng.range(0, width);
            y = rng.range(surface_high as i32, height);
        }
        let mut strength = rng.range(8, 15);
        let mut steps = rng.range(7, 30);
        if remix && (y as f64) > rock_layer {
            strength = (strength as f64 * 0.7) as i32;
            steps = (steps as f64 * 0.9) as i32;
        }
        dig(world, rng, x, y, strength, steps, kind, layers, gv);
    }
}

/// The Small Holes rejection gate (`WorldGen.cs:12035`, `:12051`): the
/// beach margins above `worldSurfaceHigh`, or the spawn middle
/// `(0.45, 0.55)` above `worldSurface` (strict inequalities here).
fn small_holes_rejected(x: i32, y: i32, width: i32, surface_high: f64, world_surface: f64) -> bool {
    ((x < SMALL_HOLES_BEACH_AVOIDANCE || x > width - SMALL_HOLES_BEACH_AVOIDANCE)
        && (y as f64) < surface_high)
        || ((x as f64) > width as f64 * 0.45
            && (x as f64) < width as f64 * 0.55
            && (y as f64) < world_surface)
}

/// One `TileRunner(type)` dig with the depth anchors the -2 water fill
/// reads (`WorldGen.cs:76895-76909`: `GenVars.waterLine`/`lavaLine`) — the
/// lane's shared runner call for the negative-kind diggers.
#[allow(clippy::too_many_arguments)]
fn dig(
    world: &mut World,
    rng: &mut Rng,
    x: i32,
    y: i32,
    strength: i32,
    steps: i32,
    kind: i32,
    layers: &Layers,
    gv: &GenVars,
) {
    tile_runner(
        world,
        rng,
        x,
        y,
        strength as f64,
        steps,
        kind,
        layers.world_surface,
        &TileRunnerOpts {
            water_line: gv.water_line as f64,
            lava_line: gv.lava_line as f64,
            ..TileRunnerOpts::for_world(world)
        },
    );
}

/// The `Dirt Layer Caves` pass (`WorldGen.cs:12067-12106`): `area * 3E-05`
/// big worms (remix ×2 count) through the dirt band
/// `[worldSurfaceLow, rockLayerHigh + 1)`, strength `Next(5, 15)`, steps
/// `Next(30, 200)` (remix ×1.1 / ×1.9), the `Next(6) == 0` water lottery.
/// The rejection loop mirrors Small Holes' but the spawn middle is
/// INCLUSIVE here (`>= 0.45·w && <= 0.55·w`, `:12090`) and the depth
/// comparison uses `Main.worldSurface`. The whole loop body is gated on
/// `rockLayerHigh <= maxTilesY` (`:12081`) — true on every real world.
/// Draw order per worm: `Next(6)` → x → y → rejections → strength → steps.
pub(crate) fn pass_dirt_layer_caves(
    world: &mut World,
    rng: &mut Rng,
    layers: &Layers,
    gv: &GenVars,
) {
    let (width, height) = (world.width, world.height);
    let remix = world.seeds.contains(SeedFlags::REMIX);
    let world_surface = layers.world_surface;
    let surface_high = layers.surface_high;
    let rock_high = layers.rock_high;
    let mut count = ((width * height) as f64 * 3e-5) as i32;
    if remix {
        count *= 2;
    }
    for _ in 0..count {
        if rock_high > height as f64 {
            continue; // the C# `if` wraps the whole body incl. its draws
        }
        let kind = if rng.range(0, 6) == 0 { -2 } else { -1 };
        let mut x = rng.range(0, width);
        let mut y = rng.range(layers.surface_low as i32, rock_high as i32 + 1);
        // The rejection gate (`:12090`) — the middle terms are inclusive.
        let rejected = |x: i32, y: i32| {
            ((x < SMALL_HOLES_BEACH_AVOIDANCE || x > width - SMALL_HOLES_BEACH_AVOIDANCE)
                && (y as f64) < surface_high)
                || ((x as f64) >= width as f64 * 0.45
                    && (x as f64) <= width as f64 * 0.55
                    && (y as f64) < world_surface)
        };
        while rejected(x, y) {
            x = rng.range(0, width);
            y = rng.range(layers.surface_low as i32, rock_high as i32 + 1);
        }
        let mut strength = rng.range(5, 15);
        let mut steps = rng.range(30, 200);
        if remix {
            strength = (strength as f64 * 1.1) as i32;
            steps = (steps as f64 * 1.9) as i32;
        }
        dig(world, rng, x, y, strength, steps, kind, layers, gv);
    }
}

/// The `Rock Layer Caves` pass (`WorldGen.cs:12107-12161`):
/// `area * 0.00013` deep worms (remix ×1.1 count) below `rockLayerHigh`,
/// strength `Next(6, 20)`, steps `Next(50, 300)` (remix ×0.7 both), the
/// `Next(10) == 0` water lottery. NOTE the draw order: the C# rolls
/// strength and steps BEFORE the runner's x/y (`:12127-12134` — they are
/// computed into locals first, the coordinates are inline arguments).
/// The remix-only second loop (`:12137-12159`, the paired horizontal
/// `noYChange` diggers) ports with its verbatim five-draw head: type
/// lottery → strength `Next(7, 26)` → steps `Next(50, 200)` → speedX
/// `Next(100, 221) * 0.1` → speedY `Next(-10, 11) * 0.02` → x → y, then
/// the mirrored runner pair.
pub(crate) fn pass_rock_layer_caves(
    world: &mut World,
    rng: &mut Rng,
    layers: &Layers,
    gv: &GenVars,
) {
    let (width, height) = (world.width, world.height);
    let remix = world.seeds.contains(SeedFlags::REMIX);
    let rock_high = layers.rock_high;
    let mut count = ((width * height) as f64 * 0.00013) as i32;
    if remix {
        count = (count as f64 * 1.1) as i32;
    }
    for _ in 0..count {
        if rock_high > height as f64 {
            continue; // `:12120` — the gate wraps the draws too
        }
        let kind = if rng.range(0, 10) == 0 { -2 } else { -1 };
        // Strength/steps BEFORE the coordinates (`:12127-12134`).
        let mut strength = rng.range(6, 20);
        let mut steps = rng.range(50, 300);
        if remix {
            strength = (strength as f64 * 0.7) as i32;
            steps = (steps as f64 * 0.7) as i32;
        }
        let x = rng.range(0, width);
        let y = rng.range(rock_high as i32, height);
        dig(world, rng, x, y, strength, steps, kind, layers, gv);
    }
    if remix {
        // The paired horizontal diggers (`:12137-12159`).
        let count2 = ((width * height) as f64 * 0.00013 * 0.4) as i32;
        for _ in 0..count2 {
            if rock_high > height as f64 {
                continue;
            }
            let kind = if rng.range(0, 10) == 0 { -2 } else { -1 };
            let strength = rng.range(7, 26);
            let steps = rng.range(50, 200);
            let speed_x = rng.range(100, 221) as f64 * 0.1;
            let speed_y = rng.range(-10, 11) as f64 * 0.02;
            let x = rng.range(0, width);
            let y = rng.range(rock_high as i32, height);
            for (sx, sy) in [(speed_x, speed_y), (-speed_x, -speed_y)] {
                tile_runner(
                    world,
                    rng,
                    x,
                    y,
                    strength as f64,
                    steps,
                    kind,
                    layers.world_surface,
                    &TileRunnerOpts {
                        speed_x: sx,
                        speed_y: sy,
                        no_y_change: true,
                        water_line: gv.water_line as f64,
                        lava_line: gv.lava_line as f64,
                        ..TileRunnerOpts::for_world(world)
                    },
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::FLAG_ACTIVE;
    use crate::worldgen::{TILE_DIRT, TILE_STONE};

    /// A flat synthetic canvas (the `caves.rs` test convention): dirt for 20
    /// rows below the surface, stone below, everything active.
    fn flat_world(width: i32, height: i32, surface: i32) -> World {
        let mut w = World::new("passes1".into(), width, height);
        for x in 0..width {
            for y in surface..height {
                let i = w.index(x, y);
                w.tiles[i].kind = if y < surface + 20 { TILE_DIRT } else { TILE_STONE };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w
    }

    fn layers_for(surface: i32, height: i32) -> Layers {
        Layers {
            avg_surface: surface as f64,
            world_surface: surface as f64 + 5.0,
            rock_layer: (surface + 40) as f64,
            hell_top: height - 60,
            surface_low: (surface + 2) as f64,
            surface_high: (surface + 22) as f64,
            rock_low: (surface + 45) as f64,
            rock_high: (surface + 80) as f64,
        }
    }

    /// The blobs land in the C# band: seeds roll in
    /// `[worldSurface, rockLayer)` (`WorldGen.cs:11704`), so no sand sits
    /// more than one walker's reach (strength ≤ 69 → radius ≤ 34, plus the
    /// ≤ 1-tile-per-step drift) above the world-surface line.
    #[test]
    fn sand_patches_land_in_the_surface_to_rock_band() {
        let (w, h) = (1000, 300);
        let surface = 60;
        let mut world = flat_world(w, h, surface);
        let layers = layers_for(surface, h);
        let mut rng = Rng(0x5A17);
        pass_sand_patches(&mut world, &mut rng, &layers);
        let sand: Vec<(i32, i32)> = (0..w)
            .flat_map(|x| (0..h).map(move |y| (x, y)))
            .filter(|&(x, y)| world.tile(x, y).is_some_and(|t| t.active() && t.kind == TILE_SAND))
            .collect();
        assert!(!sand.is_empty(), "13 blobs place no sand");
        let top = layers.world_surface as i32 - 40;
        assert!(
            sand.iter().all(|&(_, y)| y >= top),
            "sand above the band's walker reach"
        );
    }

    /// The pass is deterministic for a seed (the a == b contract).
    #[test]
    fn sand_patches_are_deterministic_for_a_seed() {
        let run = || {
            let mut world = flat_world(600, 240, 50);
            let layers = layers_for(50, 240);
            pass_sand_patches(&mut world, &mut Rng(0x5A17), &layers);
            world.tiles
        };
        assert_eq!(run(), run());
    }

    /// Mount Caves: on the vanilla-width canvas the pass raises dirt above
    /// the flat surface and publishes its anchors (`WorldGen.cs:11847-11849`).
    /// The anchors sit ≥100 apart (the spacing gate `:11815`) and outside
    /// the ±90 spawn middle (`:11808`).
    #[test]
    fn mount_caves_raise_dirt_and_publish_spaced_anchors() {
        let (w, h) = (4200, 400);
        let surface = 100;
        let mut world = flat_world(w, h, surface);
        let layers = layers_for(surface, h);
        let mut gv = GenVars::default();
        pass_mount_caves(&mut world, &mut gv, &mut Rng(0x3000), &layers);
        assert!(gv.num_m_caves > 0, "4 mountains roll on a 4200-wide world");
        for j in 0..gv.num_m_caves as usize {
            let x = gv.m_cave_x[j];
            assert!(x <= w / 2 - 90 || x >= w / 2 + 90, "anchor {x} in the middle");
            for k in 0..j {
                assert!((x - gv.m_cave_x[k]).abs() >= 100, "anchors too close");
            }
            // The mountain climbs ABOVE the flat surface row.
            assert!(
                (0..surface).any(|y| world
                    .tile(x, y)
                    .is_some_and(|t| t.active() && t.kind == TILE_DIRT)),
                "no dirt raised above the surface at anchor {x}"
            );
        }
    }

    /// The sand proximity refusal (`WorldGen.cs:11838`): a sand tile inside
    /// the ±50×±25 window of the column's first active tile refuses the
    /// mountain. A world paved with sand at the surface raises nothing.
    #[test]
    fn mount_caves_refuse_sand_neighbourhoods() {
        let (w, h) = (4200, 400);
        let surface = 100;
        let mut world = flat_world(w, h, surface);
        for x in 0..w {
            let i = world.index(x, surface);
            world.tiles[i].kind = TILE_SAND;
        }
        let layers = layers_for(surface, h);
        let mut gv = GenVars::default();
        pass_mount_caves(&mut world, &mut gv, &mut Rng(0x3000), &layers);
        assert_eq!(gv.num_m_caves, 0, "sand world raises no mountains");
    }

    /// Dirt Wall Backgrounds: the wall paints start one row BELOW the
    /// first fully-solid 2×3 block (the latch trips after the row's paint,
    /// `WorldGen.cs:11884-11891`) and stop at `worldSurface + num ≤
    /// worldSurface + 10`; a snow column carries wall 40, a jungle wall 64
    /// cell keeps its wall.
    #[test]
    fn dirt_walls_paint_below_the_solid_latch_and_stop_at_the_band() {
        let (w, h) = (64, 200);
        let surface = 40;
        let mut world = flat_world(w, h, surface);
        // A snow patch over columns 10..20 and a pre-existing jungle wall
        // at (30, surface + 5).
        for x in 10..20 {
            for y in surface..h {
                let i = world.index(x, y);
                world.tiles[i].kind = 147;
            }
        }
        let ji = world.index(30, surface + 5);
        world.tiles[ji].wall = 64;
        let layers = layers_for(surface, h); // world_surface = 45
        pass_dirt_wall_backgrounds(&mut world, &mut Rng(0xD1A7), &layers);
        // The loop bound `j < worldSurface + 10` caps the paint at row
        // worldSurface + 9; the per-column `num` walk can stop it earlier.
        let cap = layers.world_surface as i32 + 10;
        let mut painted = 0;
        for x in 1..w - 1 {
            for y in 0..h {
                let t = world.tile(x, y).unwrap();
                if y <= surface || y >= cap {
                    if !(x == 30 && y == surface + 5) {
                        assert_eq!(t.wall, 0, "wall at ({x},{y}) outside the paint band");
                    }
                    continue;
                }
                if x == 30 && y == surface + 5 {
                    assert_eq!(t.wall, 64, "the jungle wall survives");
                } else if t.wall != 0 {
                    painted += 1;
                    let expected = if (10..20).contains(&x) { 40 } else { 2 };
                    assert_eq!(t.wall, expected, "wall at ({x},{y})");
                }
            }
        }
        assert!(painted > 0, "no walls painted");
    }

    /// Clay: the blobs land (the 4200×1200 canvas gives 100 + 252 + 100
    /// runners) and the tidy sweep leaves no clay in the five rows under
    /// any column's first active tile (`WorldGen.cs:11989-12005`).
    #[test]
    fn clay_blobs_land_and_the_surface_sweep_clears_the_top_rows() {
        let (w, h) = (1000, 400);
        let surface = 100;
        let mut world = flat_world(w, h, surface);
        let layers = layers_for(surface, h);
        pass_clay(&mut world, &mut Rng(0xC1A0), &layers);
        assert!(
            world.tiles.iter().any(|t| t.active() && t.kind == 40),
            "no clay placed"
        );
        // The sweep's own postcondition: rows n..n+5 under the first
        // active tile carry no clay (columns 5..w-5, n < worldSurface-1).
        for m in 5..w - 5 {
            if let Some(n) = (1..layers.world_surface as i32 - 1)
                .find(|&n| world.tile(m, n).is_some_and(|t| t.active()))
            {
                for num in n..n + 5 {
                    assert!(
                        !world.tile(m, num).is_some_and(|t| t.kind == 40),
                        "clay survives the sweep at ({m},{num})"
                    );
                }
            }
        }
    }

    /// Clay is deterministic for a seed (the a == b contract).
    #[test]
    fn clay_is_deterministic_for_a_seed() {
        let run = || {
            let mut world = flat_world(600, 240, 50);
            pass_clay(&mut world, &mut Rng(0xC1A0), &layers_for(50, 240));
            world.tiles
        };
        assert_eq!(run(), run());
    }

    /// Small Holes: the diggers open air below `worldSurfaceHigh` and the
    /// 1-in-5 lottery fills some pockets with liquid OUTSIDE the
    /// `[waterLine, lavaLine]` band (`WorldGen.cs:12019-12021`; the
    /// runner's -2 arm gates on `l < waterLine || l > lavaLine`, `:76895`).
    #[test]
    fn small_holes_dig_air_and_fill_water_pockets() {
        let (w, h) = (1000, 400);
        let surface = 100;
        let mut world = flat_world(w, h, surface);
        let layers = layers_for(surface, h);
        let mut gv = GenVars::default();
        gv.water_line = 200;
        gv.lava_line = 300;
        pass_small_holes(&mut world, &mut Rng(0x5301), &layers, &gv);
        // Air opened below the surface-high line.
        let air = (layers.surface_high as i32..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .filter(|&(x, y)| world.tile(x, y).is_some_and(|t| !t.active()))
            .count();
        assert!(air > 0, "no holes dug");
        // The -2 arm fills water above the water line or below the lava
        // line (`:76895`: `l < GenVars.waterLine || l > GenVars.lavaLine`).
        let water = (layers.surface_high as i32..200)
            .chain(301..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .any(|(x, y)| world.tile(x, y).is_some_and(|t| t.liquid > 0));
        assert!(water, "no water pocket from the Next(5) lottery");
    }

    /// Small Holes is deterministic for a seed (the a == b contract).
    #[test]
    fn small_holes_are_deterministic_for_a_seed() {
        let run = || {
            let mut world = flat_world(600, 300, 60);
            let mut gv = GenVars::default();
            gv.water_line = 150;
            gv.lava_line = 220;
            pass_small_holes(&mut world, &mut Rng(0x5301), &layers_for(60, 300), &gv);
            world.tiles
        };
        assert_eq!(run(), run());
    }

    /// Dirt Layer Caves: the long worms open air through the dirt band and
    /// are deterministic for a seed (a == b). The 1000×400 canvas gives
    /// `400000 * 3E-05 = 12` worms of 30..199 steps.
    #[test]
    fn dirt_layer_caves_open_long_passages_deterministically() {
        let run = || {
            let mut world = flat_world(1000, 400, 100);
            let mut gv = GenVars::default();
            gv.water_line = 200;
            gv.lava_line = 300;
            pass_dirt_layer_caves(&mut world, &mut Rng(0xD12C), &layers_for(100, 400), &gv);
            world.tiles
        };
        let tiles = run();
        assert_eq!(tiles, run());
        let air = tiles.iter().filter(|t| !t.active()).count();
        assert!(air > 100, "12 long worms open {air} cells");
    }

    /// Rock Layer Caves: the deep worms SEED below `rockLayerHigh`
    /// (`WorldGen.cs:12134`) and are deterministic (a == b). The worms may
    /// drift up to ±1 tile/step, so the assertion is on the deep air, not
    /// an absolute ceiling. The 1200×600 canvas gives 93 worms.
    #[test]
    fn rock_layer_caves_open_deep_passages_deterministically() {
        let run = || {
            let mut world = flat_world(1200, 600, 100);
            let mut gv = GenVars::default();
            gv.water_line = 350;
            gv.lava_line = 450;
            pass_rock_layer_caves(&mut world, &mut Rng(0x20C7), &layers_for(100, 600), &gv);
            world.tiles
        };
        let tiles = run();
        assert_eq!(tiles, run());
        // layers_for(100, 600): rock_high = 180 — deep air below it.
        let w = 1200usize;
        let deep_air = tiles
            .iter()
            .enumerate()
            .filter(|(i, t)| !t.active() && (i / w) >= 180)
            .count();
        assert!(deep_air > 100, "the deep worms open {deep_air} cells");
    }
}
