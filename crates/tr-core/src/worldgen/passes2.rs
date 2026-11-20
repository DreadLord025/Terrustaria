//! W4-d102 — the second slice of unexecuted worldgen passes, in C# anchor
//! order (the lane's range declaration: `docs/research/12a-d102-passes-2.md`):
//! Silt (`WorldGen.cs:13143`), Webs (`:13607`), Slush (`:14697`), Mountain
//! Caves (`:14873`), Gems (`:15050`), Clean Up Dirt (`:15246`), Dirt Rock
//! Wall Runner (`:15461`) and Wood Tree Walls (`:15717`). The Shimmer pass
//! (`:15195`) lives in its own module (`worldgen/shimmer.rs`).
//!
//! Every pass draws from its own `pass_rng("<vanilla name>")` stream (the
//! port's per-pass re-seed model, `pipeline.rs`'s `Streams::pass_rng`), so
//! inserting a step shifts no existing step's draw ledger. The secret-seed
//! arms are ported where they are pure multipliers/band swaps behind
//! `World::seeds` / `World::micro_seeds`; a classic world reads no flag.

use super::noise::Rng;
use super::tile_runner::{tile_runner, TileRunnerOpts};
use crate::seeds::SeedFlags;
use crate::world::World;

/// `TileID.SiltBlock` (`TileID.cs`) — the Silt pass's runner kind.
const TILE_SILT: i32 = 123;
/// `WallID.Sandstone` = 187 / `WallID.HardenedSand` = 216 — the desert walls
/// the Silt candidates skip (`WorldGen.cs:13159`, `:13174`).
const WALL_SANDSTONE: u16 = 187;
const WALL_HARDENED_SAND: u16 = 216;

/// The Silt pass (`WorldGen.cs:13143-13181`): two TileRunner loops over the
/// rock layer, kind 123. The first runs `(int)((float)area * 0.0001f)` big
/// blobs (`Next(5, 12)` strength, `Next(15, 50)` steps), the second
/// `(int)((float)area * 0.0005f)` small ones (`Next(2, 5)` × `Next(2, 5)`).
/// Each candidate draws x then y; the remix band swap draws y AGAIN from
/// `worldSurface..rockLayer` (the C# overwrites `num3`/`num5` after the
/// first draw — both draws ride the stream). Candidates on the desert
/// walls 187/216 are skipped without the strength/steps draws.
pub(crate) fn pass_silt(
    world: &mut World,
    rock_layer_high: f64,
    world_surface: f64,
    rock_layer: f64,
    rng: &mut Rng,
) {
    let remix = world.seeds.contains(SeedFlags::REMIX);
    let opts = TileRunnerOpts::for_world(world);
    // `:13149` — the float multiply is the C#'s `(float)` cast, kept in f32.
    let area = (world.width * world.height) as f32;
    let num = (area * 0.0001f32) as i32;
    for _ in 0..num {
        let x = rng.range(0, world.width);
        let mut y = rng.range(rock_layer_high as i32, world.height);
        if remix {
            y = rng.range(world_surface as i32, rock_layer as i32);
        }
        let wall = world.tile(x, y).map_or(0, |t| t.wall);
        if wall != WALL_SANDSTONE && wall != WALL_HARDENED_SAND {
            let strength = rng.range(5, 12);
            let steps = rng.range(15, 50);
            tile_runner(
                world,
                rng,
                x,
                y,
                strength as f64,
                steps,
                TILE_SILT,
                world_surface,
                &opts,
            );
        }
    }
    // `:13164` — the small-blob loop.
    let num = (area * 0.0005f32) as i32;
    for _ in 0..num {
        let x = rng.range(0, world.width);
        let mut y = rng.range(rock_layer_high as i32, world.height);
        if remix {
            y = rng.range(world_surface as i32, rock_layer as i32);
        }
        let wall = world.tile(x, y).map_or(0, |t| t.wall);
        if wall != WALL_SANDSTONE && wall != WALL_HARDENED_SAND {
            let strength = rng.range(2, 5);
            let steps = rng.range(2, 5);
            tile_runner(
                world,
                rng,
                x,
                y,
                strength as f64,
                steps,
                TILE_SILT,
                world_surface,
                &opts,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    /// A stone world below row 100, walls 0 — the Silt band's canvas.
    fn stone_world(width: i32, height: i32) -> World {
        let mut w = World::new("silt".into(), width, height);
        for x in 0..width {
            for y in 100..height {
                let i = w.index(x, y);
                w.tiles[i].kind = 1;
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.world_surface = 100.0;
        w.rock_layer = 120.0;
        w
    }

    fn silt_count(w: &World) -> usize {
        w.tiles.iter().filter(|t| t.kind == 123).count()
    }

    #[test]
    fn silt_blobs_land_below_the_rock_line_and_replay() {
        let mut a = stone_world(600, 600);
        let mut b = stone_world(600, 600);
        pass_silt(&mut a, 120.0, 100.0, 120.0, &mut Rng(0x5117));
        pass_silt(&mut b, 120.0, 100.0, 120.0, &mut Rng(0x5117));
        assert_eq!(a.tiles, b.tiles, "a == b");
        assert!(silt_count(&a) > 0, "the pass places silt");
        // Every silt cell sits at or below the rock-line band's floor.
        for y in 0..100 {
            for x in 0..a.width {
                assert_ne!(a.tile(x, y).unwrap().kind, 123, "silt above the band");
            }
        }
    }

    #[test]
    fn the_desert_walls_skip_every_candidate() {
        // `:13159`/`:13174`: wall 187/216 candidates skip the runner — a
        // fully sandstone-walled band places nothing and draws only the
        // x/y pairs.
        let mut w = stone_world(600, 600);
        for x in 0..w.width {
            for y in 100..w.height {
                let i = w.index(x, y);
                w.tiles[i].wall = if x % 2 == 0 { 187 } else { 216 };
            }
        }
        pass_silt(&mut w, 120.0, 100.0, 120.0, &mut Rng(0x5117));
        assert_eq!(silt_count(&w), 0);
    }
}
