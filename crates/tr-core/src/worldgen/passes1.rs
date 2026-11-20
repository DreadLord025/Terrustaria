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
}
