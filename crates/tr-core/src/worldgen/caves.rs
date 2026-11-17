//! Cave carving and the dirt/stone/ore blob mixes.

use super::noise::{Noise, Rng};
use super::terrain::Layers;
use super::tile_runner::{tile_runner, TileRunnerOpts};
use super::{
    TILE_ASH, TILE_CRIMTANE, TILE_DEMONITE, TILE_DIRT, TILE_GOLD, TILE_HELLSTONE, TILE_IRON,
    TILE_LEAD, TILE_PLATINUM, TILE_STONE, TILE_TIN, TILE_TUNGSTEN,
};
use crate::ores::SavedOreTiers;
use crate::tile::FLAG_ACTIVE;
use crate::world::World;

/// Carves cave air with a large cavern field plus a narrow worm-tunnel field,
/// keeping a crust under the surface and fading out near hell.
pub(crate) fn carve_caves(
    world: &mut World,
    width: i32,
    noise: &Noise,
    surface: &[i32],
    hell_top: i32,
) {
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
}

/// Converts `count` round blobs of `from` tiles into `to` tiles.
pub(crate) fn blob_pass(
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

/// The gem tile kinds (`TileID.cs`: the amethyst 63 .. the diamond 68) —
/// the small veins in the stone (`WorldGen`'s gem placements).
const GEM_KINDS: [u16; 6] = [63, 64, 65, 66, 67, 68];

/// The tier coin-flips in the C# order copper → iron → silver → gold
/// (`WorldGen.cs:11274-11297`).
///
/// M5-SEED-1: The Constant (`dontStarveWorldGen`) pins iron to tile 6 and gold
/// to tile 8 — the C# gates both flips `(!dontStarveWorldGen ||
/// drunkWorldGen)` (`:11280`, `:11292`), so no lead 167 / platinum 169 ever
/// rolls for a Constant unless drunk is also on. The gate applies to the
/// *assignment* only: the port always consumes its `Next(2)`, where the C#
/// would short-circuit both draws under a non-drunk Constant. That is a
/// documented deviation (the port's stream contract keeps the four draws per
/// world), and it only shifts the stream of Constant worlds — a classic world
/// draws the identical four-draw ledger either way.
///
/// The C# mirrors the flips into the `GenVars.copperBar`-style bar ids — the
/// port's recipes carry both bar families statically, so nothing to roll there.
fn roll_tiers(rng: &mut Rng, flags: crate::seeds::SeedFlags) -> SavedOreTiers {
    // `!dontStarveWorldGen || drunkWorldGen` (`:11280`/`:11292`): the flips are
    // live exactly when this is false.
    let the_constant_pins = flags.contains(crate::seeds::SeedFlags::DONT_STARVE)
        && !flags.contains(crate::seeds::SeedFlags::DRUNK);
    let mut tiers = SavedOreTiers::new();
    if rng.range(0, 2) == 0 {
        tiers.copper = TILE_TIN as i32;
    }
    // The iron flip — the draw is unconditional (see the doc comment); the pin
    // is not.
    let iron_flip = rng.range(0, 2) == 0;
    if iron_flip && !the_constant_pins {
        tiers.iron = TILE_LEAD as i32;
    }
    if rng.range(0, 2) == 0 {
        tiers.silver = TILE_TUNGSTEN as i32;
    }
    let gold_flip = rng.range(0, 2) == 0;
    if gold_flip && !the_constant_pins {
        tiers.gold = TILE_PLATINUM as i32;
    }
    tiers
}

/// The band table (`WorldGen.cs:13399-13573`): (kind, density, y0, y1,
/// strength lo/hi, steps lo/hi) with the GenVars quartet truncated exactly as
/// the C# `(int)` casts truncate. Bands 10-11 sit behind `!noSurface &&
/// !surfaceIsInSpace` (`:13543`) — no such seeds here, they always run. The
/// drunk per-blob tier re-rolls that precede each band's `TileRunner`
/// (`:13194` … `:13562`) are deferred — see `ores_and_shinies`.
fn band_table(
    tiers: &SavedOreTiers,
    layers: &Layers,
    width: i32,
    height: i32,
    crimson: bool,
) -> [(i32, f64, i32, i32, i32, i32, i32, i32); 12] {
    let surface_low = layers.surface_low as i32;
    let surface_high = layers.surface_high as i32;
    let rock_low = layers.rock_low as i32;
    let rock_high = layers.rock_high as i32;
    let rock_layer = layers.rock_layer as i32;
    [
        (tiers.copper, 6e-5, surface_low, surface_high, 3, 6, 2, 6),
        (tiers.copper, 8e-5, surface_high, rock_high, 3, 7, 3, 7),
        (tiers.copper, 2e-4, rock_low, height, 4, 9, 4, 8),
        (tiers.iron, 3e-5, surface_low, surface_high, 3, 7, 2, 5),
        (tiers.iron, 8e-5, surface_high, rock_high, 3, 6, 3, 6),
        (tiers.iron, 2e-4, rock_low, height, 4, 9, 4, 8),
        (tiers.silver, 2.6e-5, surface_high, rock_high, 3, 6, 3, 6),
        (tiers.silver, 1.5e-4, rock_low, height, 4, 9, 4, 8),
        (tiers.gold, 1.2e-4, rock_low, height, 4, 8, 4, 8),
        (tiers.silver, 1.7e-4, 0, surface_low, 4, 9, 4, 8),
        (tiers.gold, 1.2e-4, 0, surface_low - 20, 4, 8, 4, 8),
        // The evil band (`:13589-13602`): demonite in a corruption world,
        // crimtane in a crimson one, from `Main.rockLayer` to the bottom.
        (
            if crimson {
                TILE_CRIMTANE as i32
            } else {
                TILE_DEMONITE as i32
            },
            2.25e-5,
            rock_layer,
            height,
            3,
            6,
            4,
            8,
        ),
    ]
}

/// The `OresAndShinies` generation pass (`WorldGen.cs:13182-13606`, the
/// normal branch `:13397-13603`): the four tier coin-flips in the C# setup
/// order, then twelve `TileRunner` band loops carrying the per-tile density
/// table, closed by the evil band. The gem veins and the hellstone blobs keep
/// their shape-model placement after the bands — vanilla runs them in the
/// `Gems` (`WorldGen.cs:15050`) and `Underworld` (`:13805`) passes
/// (ANALYSIS "OresAndShinies fidelity (M5-ORES-1)").
///
/// M5-SEED-1: the seed flags ride in on `world.seeds` (set once by
/// `worldgen::generate` from `GenOptions::flags`, `mod.rs:118`), so the tier
/// flips and the walkers' own arms see the same secret-seed identity the rest
/// of the pipeline does. The drunk-only pieces of the pass stay deferred:
/// the 22 per-blob tier re-rolls (`if (drunkWorldGen) { Next(2) → GenVars.X }`,
/// at `:13194` … `:13562`) and the drunk evil-band split into two
/// half-density crimtane/demonite loops (`:13370-13378`, `:13577-13586`),
/// which replace the single evil band (`:13589-13602`) in a drunk world. Both
/// are owned by the drunk-world follow-up; a classic world draws neither.
pub(crate) fn ores_and_shinies(
    world: &mut World,
    rng: &mut Rng,
    layers: &Layers,
    width: i32,
    height: i32,
    crimson: bool,
) {
    // The secret-seed flags of the world being generated — a copy, so the
    // mutable `world` borrow stays free for the walkers.
    let flags = world.seeds;
    let tiers = roll_tiers(rng, flags);
    // The setup roll is per-world state in vanilla (`GenerateWorld`'s once
    // per world, `Main.cs:11266-11297`), so it rides the World for the save
    // (M6-SAVE-1f) and the App's runtime copy reads it back.
    world.ore_tiers = tiers;
    let bands = band_table(&tiers, layers, width, height, crimson);
    let area = (width * height) as f64;
    for &(kind, density, y0, y1, s0, s1, t0, t1) in &bands {
        // `(int)((double)(maxTilesX * maxTilesY) * density)` (`:13399` and
        // friends): the C# multiplies the int area first.
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
                kind,
                layers.world_surface,
                &TileRunnerOpts {
                    flags: world.seeds, ..TileRunnerOpts::default()
                },
            );
        }
    }
    // The gem veins in the stone band (the small 1-3 blobs, one kind per
    // vein — the C# `TileRunner` gem placements).
    let hell_top = layers.hell_top;
    for _ in 0..280 {
        let x = rng.range(10, width - 10);
        let y = rng.range(160, hell_top);
        let kind = GEM_KINDS[rng.range(0, 6) as usize];
        ore_blob(world, x, y, kind, rng.range(1, 4), rng);
    }
    // Hellstone in the ash layer.
    for _ in 0..900 {
        let x = rng.range(10, width - 10);
        let y = rng.range(hell_top, height - 4);
        ore_blob(world, x, y, TILE_HELLSTONE, rng.range(2, 5), rng);
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

/// The `SurfaceCaves` pass (`WorldGen.cs:12162-12270`) — groups 1-4, the
/// surface-opening tunnels: each group rolls its column with the C#
/// rejection loop (the spawn middle 0.4-0.6 and the beach margins
/// excluded), scans from the sky down to `GenVars.worldSurfaceHigh` for the
/// first active tile — the runner starts AT the surface, so every passage
/// opens the ground — and walks `TileRunner(type = -1)`, the air dig, with
/// the group's strength/steps/speed table. The draw order is the C#'s:
/// column roll(s) → strength → steps → speed X, then the runner's own
/// ledger. Group 5's `Caverer` call (`:12251-12268`) is deferred (it needs
/// `digTunnel`, `WorldGen.cs:79110+`); the remix `×3` arms are out with the
/// remix seeds themselves.
pub(crate) fn surface_caves(
    world: &mut World,
    rng: &mut Rng,
    layers: &Layers,
    left_beach_end: i32,
    right_beach_start: i32,
) {
    let width = world.width;
    let world_surface = layers.world_surface;
    let scan_to = layers.surface_high;
    // Group 1 (`:12176-12192`): the small diggers, `maxTilesX * 0.002`.
    for _ in 0..((width as f64) * 0.002) as i32 {
        let x = open_column(rng, width, 0.45, 0.55, left_beach_end, right_beach_start);
        if let Some(j) = first_active(world, x, scan_to) {
            let strength = rng.range(3, 6);
            let steps = rng.range(5, 50);
            let vx = (rng.range(-10, 11) as f64) * 0.1;
            dig_shaft(world, rng, x, j, strength, steps, vx, 1.0, false, world_surface);
        }
    }
    // Group 2 (`:12194-12210`): the mid diggers, `maxTilesX * 0.0007`.
    for _ in 0..((width as f64) * 0.0007) as i32 {
        let x = open_column(rng, width, 0.43, 0.57, left_beach_end, right_beach_start);
        if let Some(j) = first_active(world, x, scan_to) {
            let strength = rng.range(10, 15);
            let steps = rng.range(50, 130);
            let vx = (rng.range(-10, 11) as f64) * 0.1;
            dig_shaft(world, rng, x, j, strength, steps, vx, 2.0, false, world_surface);
        }
    }
    // Group 3 (`:12212-12230`): the large three-runner digs,
    // `maxTilesX * 0.0003` — the trio's draw order is the call order.
    for _ in 0..((width as f64) * 0.0003) as i32 {
        let x = open_column(rng, width, 0.4, 0.6, left_beach_end, right_beach_start);
        if let Some(j) = first_active(world, x, scan_to) {
            let (s1, t1) = (rng.range(12, 25), rng.range(150, 500));
            let vx1 = (rng.range(-10, 11) as f64) * 0.1;
            dig_shaft(world, rng, x, j, s1, t1, vx1, 4.0, false, world_surface);
            let (s2, t2) = (rng.range(8, 17), rng.range(60, 200));
            let vx2 = (rng.range(-10, 11) as f64) * 0.1;
            dig_shaft(world, rng, x, j, s2, t2, vx2, 2.0, false, world_surface);
            let (s3, t3) = (rng.range(5, 13), rng.range(40, 170));
            let vx3 = (rng.range(-10, 11) as f64) * 0.1;
            dig_shaft(world, rng, x, j, s3, t3, vx3, 2.0, false, world_surface);
        }
    }
    // Group 4 (`:12232-12249`): the horizontal digs, `maxTilesX * 0.0004` —
    // the C# passes literal `0.0` speed X and `noYChange: true`, so the
    // group's only draws are the column roll, the strength and the steps.
    for _ in 0..((width as f64) * 0.0004) as i32 {
        let x = open_column(rng, width, 0.4, 0.6, left_beach_end, right_beach_start);
        if let Some(j) = first_active(world, x, scan_to) {
            let strength = rng.range(7, 12);
            let steps = rng.range(150, 250);
            dig_shaft(world, rng, x, j, strength, steps, 0.0, 1.0, true, world_surface);
        }
    }
}

/// One `TileRunner(type = -1)` dig from the group tables
/// (`WorldGen.cs:12188`/`:12206`/`:12224-12226`/`:12245`): the air dig from
/// the surface tile the sky scan found.
#[allow(clippy::too_many_arguments)]
fn dig_shaft(
    world: &mut World,
    rng: &mut Rng,
    x: i32,
    j: i32,
    strength: i32,
    steps: i32,
    speed_x: f64,
    speed_y: f64,
    no_y_change: bool,
    world_surface: f64,
) {
    tile_runner(
        world,
        rng,
        x,
        j,
        strength as f64,
        steps,
        -1,
        world_surface,
        &TileRunnerOpts {
            speed_x,
            speed_y,
            no_y_change,
            ..TileRunnerOpts::for_world(world)
        },
    );
}

/// The pass's column roll (`WorldGen.cs:12179-12183` and friends): the C#
/// rejection loop over the spawn middle and the beach margins, verbatim,
/// with one documented small-world guard — vanilla never generates worlds
/// narrower than 4200 tiles (where the beach window plus the middle
/// exclusion always leave rolls); a port test world (512) can make the
/// beach interval empty, and there the gate collapses to the middle
/// exclusion instead of spinning forever.
fn open_column(
    rng: &mut Rng,
    width: i32,
    mid_lo: f64,
    mid_hi: f64,
    left_beach_end: i32,
    right_beach_start: i32,
) -> i32 {
    let beach_gate = left_beach_end + 20 < right_beach_start - 20;
    let mut x = rng.range(0, width);
    while ((x as f64) > width as f64 * mid_lo && (x as f64) < width as f64 * mid_hi)
        || (beach_gate && (x < left_beach_end + 20 || x > right_beach_start - 20))
    {
        x = rng.range(0, width);
    }
    x
}

/// The sky scan (`:12184-12191` and friends): the first active tile down to
/// `GenVars.worldSurfaceHigh` — `None` when the column is open all the way,
/// and the C# skips the runner (and its draws) for that column.
fn first_active(world: &World, x: i32, scan_to: f64) -> Option<i32> {
    let top = (scan_to as i32).min(world.height);
    (0..top).find(|&j| world.tile(x, j).is_some_and(|t| t.active()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seeds::SeedFlags;
    use crate::worldgen::noise::Rng;

    /// DW-4.2: the tier flips consume exactly one `Next(2)` per pair in the
    /// C# order copper → iron → silver → gold (`WorldGen.cs:11274-11297`).
    #[test]
    fn the_tier_flips_draw_in_the_csharp_order() {
        let mut runner = Rng(0x5EED);
        let tiers = roll_tiers(&mut runner, SeedFlags::none());
        let mut replay = Rng(0x5EED);
        for (normal, alt, rolled) in [
            (7i32, TILE_TIN, tiers.copper),
            (6, TILE_LEAD, tiers.iron),
            (9, TILE_TUNGSTEN, tiers.silver),
            (8, TILE_PLATINUM, tiers.gold),
        ] {
            // The C# picks the *alternate* on `Next(2) == 0` (`:11274-11279`) —
            // the pre-M5-ORES-1 pools had the polarity inverted.
            let pick = if replay.range(0, 2) == 0 {
                alt as i32
            } else {
                normal
            };
            assert_eq!(rolled, pick, "the pair {normal}/{alt}");
        }
        // Exactly four draws: the stream state matches the replay's.
        assert_eq!(runner.0, replay.0);
        // The hardmode trio keeps its un-rolled sentinel.
        assert_eq!(
            (tiers.cobalt, tiers.mythril, tiers.adamantite),
            (-1, -1, -1)
        );
    }

    /// M5-SEED-1: The Constant pins iron to tile 6 and gold to tile 8 —
    /// `(!dontStarve || drunk)` (`WorldGen.cs:11280`, `:11292`) gates both
    /// flips, so no lead 167 / platinum 169 rolls. The sweep needs several
    /// streams because a classic world rolls lead/platinum roughly a quarter
    /// of the time each; with the pin on it must never roll either.
    #[test]
    fn the_constant_pins_iron_and_gold() {
        let constant = SeedFlags::from(SeedFlags::DONT_STARVE);
        for seed in 1..=64u64 {
            let mut runner = Rng(seed);
            let tiers = roll_tiers(&mut runner, constant);
            assert_eq!(tiers.iron, TILE_IRON as i32, "seed {seed} iron pinned");
            assert_eq!(tiers.gold, TILE_GOLD as i32, "seed {seed} gold pinned");
        }
        // A classic world rolls the alternates freely (the control half: without
        // it the pinned assertions above would pass by never rolling anything).
        let alternates = (1..=64u64)
            .filter(|&seed| {
                let tiers = roll_tiers(&mut Rng(seed), SeedFlags::none());
                tiers.iron == TILE_LEAD as i32 || tiers.gold == TILE_PLATINUM as i32
            })
            .count();
        assert!(alternates > 0, "a classic world rolls lead/platinum");
    }

    /// M5-SEED-1: drunk lifts the pin — `!dontStarve || drunk` is true, so a
    /// drunken Constant rolls the alternates like any other world.
    #[test]
    fn a_drunken_constant_still_rolls_the_alternates() {
        let flags = SeedFlags::from(SeedFlags::DONT_STARVE | SeedFlags::DRUNK);
        let lead = (1..=512u64)
            .map(|seed| roll_tiers(&mut Rng(seed), flags))
            .filter(|t| t.iron == TILE_LEAD as i32)
            .count();
        assert!(lead > 0, "a drunken Constant rolls lead");
        let platinum = (1..=512u64)
            .map(|seed| roll_tiers(&mut Rng(seed), flags))
            .filter(|t| t.gold == TILE_PLATINUM as i32)
            .count();
        assert!(platinum > 0, "a drunken Constant rolls platinum");
    }

    /// DW-4.3: the band table equals the research §1.2 table
    /// (`WorldGen.cs:13399-13573`, the normal branch).
    #[test]
    fn the_band_table_matches_the_csharp_densities() {
        let layers = Layers {
            avg_surface: 250.0,
            world_surface: 285.0,
            rock_layer: 414.0,
            hell_top: 1030,
            surface_low: 220.0,
            surface_high: 260.0,
            rock_low: 238.0,
            rock_high: 282.0,
        };
        let tiers = SavedOreTiers::new();
        let table = band_table(&tiers, &layers, 4200, 1200, false);
        let expected: [(i32, f64, i32, i32, i32, i32, i32, i32); 12] = [
            (7, 6e-5, 220, 260, 3, 6, 2, 6),
            (7, 8e-5, 260, 282, 3, 7, 3, 7),
            (7, 2e-4, 238, 1200, 4, 9, 4, 8),
            (6, 3e-5, 220, 260, 3, 7, 2, 5),
            (6, 8e-5, 260, 282, 3, 6, 3, 6),
            (6, 2e-4, 238, 1200, 4, 9, 4, 8),
            (9, 2.6e-5, 260, 282, 3, 6, 3, 6),
            (9, 1.5e-4, 238, 1200, 4, 9, 4, 8),
            (8, 1.2e-4, 238, 1200, 4, 8, 4, 8),
            (9, 1.7e-4, 0, 220, 4, 9, 4, 8),
            (8, 1.2e-4, 0, 200, 4, 8, 4, 8),
            (22, 2.25e-5, 414, 1200, 3, 6, 4, 8),
        ];
        assert_eq!(table, expected);
        // The crimson flag swaps the evil band's kind.
        let crimson = band_table(&tiers, &layers, 4200, 1200, true);
        assert_eq!(crimson[11].0, 204);
        // The blob count is the truncated int area times the density.
        let area = (4200 * 1200) as f64;
        assert_eq!((area * 6e-5) as i32, 302);
        assert_eq!((area * 2.25e-5) as i32, 113);
    }
}

/// The `worldgen-caves` slice pins (`task/worldgen-caves`): the world's
/// caves and the copper/iron veins inside the C# depth/size ranges, on the
/// named per-pass streams.
mod worldgen_caves {
    use super::*;
    use crate::ores::ore_runner;
    use crate::UnifiedRandom;
    use crate::worldgen::TILE_COPPER;
    use crate::worldgen::world_hash::world_hash;
    use crate::worldgen::{generate, GenOptions};

    /// A flat synthetic canvas: dirt for the first 20 rows below the
    /// surface, stone below, everything active — the passes' playfield.
    fn flat_world(width: i32, height: i32, surface: i32) -> World {
        let mut w = World::new("worldgen-caves".into(), width, height);
        for x in 0..width {
            for y in surface..height {
                let i = w.index(x, y);
                w.tiles[i].kind = if y < surface + 20 { TILE_DIRT } else { TILE_STONE };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w
    }

    /// A synthetic `Layers` for the canvas: the GenVars quartet brackets the
    /// surface at the same distances a real terrain map produces.
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

    /// t1 — the world contract: the same seed generates byte-identical
    /// tiles (and the generated world carries the copper/iron tier veins).
    #[test]
    fn same_seed_same_tiles() {
        let opts = GenOptions {
            width: 512,
            height: 512,
            seed: 42,
            ..Default::default()
        };
        let a = generate(&opts);
        let b = generate(&opts);
        assert_eq!(a.tiles.len(), b.tiles.len());
        for (ta, tb) in a.tiles.iter().zip(b.tiles.iter()) {
            assert_eq!(ta.kind, tb.kind);
            assert_eq!(ta.wall, tb.wall);
            assert_eq!(ta.flags, tb.flags);
            assert_eq!(ta.liquid, tb.liquid);
        }
        assert_eq!(world_hash(&a), world_hash(&b));
        // The copper/iron tier veins exist in the generated world.
        for tier in [a.ore_tiers.copper as u16, a.ore_tiers.iron as u16] {
            assert!(
                a.tiles.iter().any(|t| t.active() && t.kind == tier),
                "the tier vein {tier} is missing from the generated world"
            );
        }
    }

    /// t2 — the copper veins land inside the C# band depths
    /// (`WorldGen.cs:13399-13444`, the normal branch): the three bands roll
    /// their seed points inside `[worldSurfaceLow, worldSurfaceHigh)` at
    /// `6E-05`, `[worldSurfaceHigh, rockLayerHigh)` at `8E-05` and
    /// `[rockLayerLow, maxTilesY)` at `2E-04`; every placed tile stays
    /// within a walker's reach of its band.
    #[test]
    fn copper_depth_range() {
        let (w, h) = (420, 240);
        let surface = 60;
        let mut world = flat_world(w, h, surface);
        let layers = layers_for(surface, h);
        let mut rng = Rng(0x0CAFE);
        ores_and_shinies(&mut world, &mut rng, &layers, w, h, false);
        let copper = world.ore_tiers.copper as u16;
        let in_band = |lo: f64, hi: f64| -> bool {
            let lo = lo as i32;
            let hi = (hi as i32).min(h);
            (lo.max(0)..hi).any(|y| {
                (0..w).any(|x| world.tile(x, y).is_some_and(|t| t.active() && t.kind == copper))
            })
        };
        // Each band's depth window carries at least one vein tile.
        assert!(
            in_band(layers.surface_low, layers.surface_high),
            "no copper in the surface band"
        );
        assert!(
            in_band(layers.surface_high, layers.rock_high),
            "no copper in the mid band"
        );
        assert!(
            in_band(layers.rock_low, h as f64),
            "no copper in the deep band"
        );
        // And no copper above the surface band minus one walker's reach
        // (the deepest band 1 strength is 5, so the walk climbs ≤ ~8).
        let lo = layers.surface_low as i32;
        let shallowest = (0..w)
            .filter_map(|x| {
                (0..h).find(|&y| {
                    world.tile(x, y).is_some_and(|t| t.active() && t.kind == copper)
                })
            })
            .min();
        assert!(
            shallowest.unwrap_or(lo) >= lo - 10,
            "copper above the C# surface band: row {shallowest:?}"
        );
    }

    /// t3 — the vein size inside the C# walker's bounds: `OreRunner`
    /// (`WorldGen.cs:48831-48900`) lays shrinking Manhattan discs — per
    /// step the radius is `strength · (left/steps) · 0.5 · (1 +
    /// Next(-10, 11) · 0.015)`, so the copper/iron deep band's worst case
    /// (strength 8, steps 8) sums to ≈ 217 placed tiles.
    #[test]
    fn vein_size_bounds() {
        // The worst case: the deep band's upper corners.
        let mut world = flat_world(160, 120, 40);
        let mut rng = UnifiedRandom::new(1234);
        ore_runner(&mut world, 80, 70, 8, 8, TILE_COPPER, &mut rng);
        let placed = world
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == TILE_COPPER)
            .count();
        assert!(placed >= 10, "the max-parameter vein places {placed}");
        assert!(placed <= 220, "the C# walker bound: {placed} tiles");
        // The shallow band's smallest vein stays tiny.
        let mut world = flat_world(160, 120, 40);
        let mut rng = UnifiedRandom::new(1234);
        ore_runner(&mut world, 80, 70, 3, 2, TILE_COPPER, &mut rng);
        let small = world
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == TILE_COPPER)
            .count();
        assert!(small >= 1 && small <= 40, "the min-parameter vein: {small}");
    }

    /// t4 — the SurfaceCaves passages open the surface: the C# scan starts
    /// every runner at the first active tile from the sky
    /// (`WorldGen.cs:12184-12191`), so each passage's mouth is the surface
    /// itself. The canvas is the vanilla small world's 4200-tile width —
    /// the group counts (`maxTilesX * 0.002` and friends) round to zero on
    /// anything narrower. Pinned for the fixed pass seed on the flat
    /// canvas.
    #[test]
    fn caves_connected() {
        let (w, h) = (4200, 400);
        let surface = 60;
        let mut world = flat_world(w, h, surface);
        let layers = layers_for(surface, h);
        let mut rng = Rng(0xCAFE);
        surface_caves(&mut world, &mut rng, &layers, 0, w);
        // An opening: a column whose first active tile moved below the flat
        // surface row; a passage: the mouth's air run keeps descending into
        // the ground (the C# walker carries speed Y 1.0 down from the
        // surface, so the centre columns trench).
        let mut openings = 0;
        let mut passable = 0;
        let mut first: Option<(i32, i32)> = None;
        for x in 0..w {
            let mut y = 0;
            while y < h && world.tile(x, y).is_some_and(|t| !t.active()) {
                y += 1;
            }
            if y <= surface {
                continue;
            }
            openings += 1;
            let air_run = (y..(y + 6).min(h))
                .filter(|&yy| world.tile(x, yy).is_some_and(|t| !t.active()))
                .count();
            if air_run >= 3 {
                passable += 1;
                if first.is_none() {
                    first = Some((x, y));
                }
            }
        }
        assert!(openings >= 1, "no surface openings ({openings})");
        assert!(
            passable >= 1,
            "openings {openings} but none descends into the ground"
        );
        let (ox, oy) = first.unwrap();
        assert!(
            oy > surface,
            "the pinned passage at ({ox},{oy}) is not a surface breach"
        );
    }
}
