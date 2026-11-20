//! W4-d101 — the first slice of the unexecuted vanilla passes
//! (`docs/research/12b-d101-passes-1.md`): Sand Patches, Mount Caves, Dirt
//! Wall Backgrounds, Clay, Small Holes, Dirt Layer Caves, Rock Layer Caves.
//! Each body is the verbatim `WorldGen.AddPasses` registration block on its
//! own `pass_rng("<vanilla name>")` stream; the ice biome (`:12312`) lives in
//! `ice_biome.rs` and Granite already runs in `biomes.rs::pass_granite`.

use super::noise::Rng;
use super::terrain::Layers;
use super::tile_runner::{tile_runner, TileRunnerOpts};
use super::TILE_SAND;
use crate::seeds::SeedFlags;
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
}
