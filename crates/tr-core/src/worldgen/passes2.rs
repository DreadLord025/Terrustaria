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
use crate::tile::FLAG_ACTIVE;
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

/// `TileID.Cobweb` = 51 — the Webs pass's runner kind.
const TILE_COBWEB: i32 = 51;

/// The walls the port's terrain pre-lays under every solid row
/// (`terrain.rs:343-351`) — the deviation counterpart of the C#'s `wall > 0`
/// (the `spider_gem_moss.rs::BASELINE_WALLS` precedent): vanilla leaves the
/// underground wall 0 until `Cave Walls` runs, so its surface-gate
/// `wall > 0` only counts walls a pass painted; the port reads its baseline
/// walls as unwalled.
const BASELINE_WALLS: [u16; 4] = [1, 2, 187, 216];

/// `tile.wall > 0` with the port's baseline-wall adaptation (see
/// [`BASELINE_WALLS`]).
fn walled(world: &World, x: i32, y: i32) -> bool {
    world
        .tile(x, y)
        .is_some_and(|t| t.wall > 0 && !BASELINE_WALLS.contains(&t.wall))
}

/// The Webs pass (`WorldGen.cs:13607-13644`): `(int)(area * 0.0006)`
/// ceiling-hugging cobweb runners. Each candidate draws x (`Next(20,
/// maxTilesX - 20)`) then y (`Next(worldSurfaceHigh, maxTilesY - 20)`);
/// the first `GenVars.numMCaves` iterations take the mountain-cave anchors
/// instead (the draws still happen — the C# overwrites after drawing).
/// An inactive candidate above the surface line needs a real wall; the
/// walker climbs to the first active tile above `worldSurfaceLow`, steps
/// back down one, rolls the horizontal direction (`Next(2)`), slides along
/// the ceiling to the first active tile (or the ±10 world margin), steps
/// back one, and runs `TileRunner(j, num, Next(4, 11), Next(2, 4), 51,
/// addTile: true, speedX: num2, speedY: -1)` when the landing cell is
/// below the surface or walled.
pub(crate) fn pass_webs(
    world: &mut World,
    world_surface_high: f64,
    world_surface_low: f64,
    world_surface: f64,
    num_m_caves: i32,
    m_cave_x: &[i32; 30],
    m_cave_y: &[i32; 30],
    rng: &mut Rng,
) {
    let opts = TileRunnerOpts {
        add_tile: true,
        speed_y: -1.0,
        over_ride: false,
        ..TileRunnerOpts::for_world(world)
    };
    // `:13612` — the double multiply, truncated.
    let count = ((world.width * world.height) as f64 * 0.0006) as i32;
    for i in 0..count {
        let mut j = rng.range(20, world.width - 20);
        let mut num = rng.range(world_surface_high as i32, world.height - 20);
        if i < num_m_caves {
            j = m_cave_x[i as usize];
            num = m_cave_y[i as usize];
        }
        let inactive = !world.tile(j, num).is_some_and(|t| t.active());
        // `:13621` — only an INACTIVE candidate enters, and above the
        // surface line it needs a real wall.
        if inactive && ((num as f64) > world_surface || walled(world, j, num)) {
            // The climb (`:13623-13626`): up while inactive, stopping at
            // the worldSurfaceLow floor; then one step back down.
            while !world.tile(j, num).is_some_and(|t| t.active())
                && num > world_surface_low as i32
            {
                num -= 1;
            }
            num += 1;
            let mut num2 = 1;
            if rng.range(0, 2) == 0 {
                num2 = -1;
            }
            // The ceiling slide (`:13633-13636`).
            while !world.tile(j, num).is_some_and(|t| t.active())
                && j > 10
                && j < world.width - 10
            {
                j += num2;
            }
            j -= num2;
            if (num as f64) > world_surface || walled(world, j, num) {
                let strength = rng.range(4, 11);
                let steps = rng.range(2, 4);
                let mut opts = opts;
                opts.speed_x = num2 as f64;
                tile_runner(
                    world,
                    rng,
                    j,
                    num,
                    strength as f64,
                    steps,
                    TILE_COBWEB,
                    world_surface,
                    &opts,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    /// A stone world with an air pocket at rows 120..140 — the webs'
    /// ceiling canvas.
    fn cave_world() -> World {
        let mut w = stone_world(600, 600);
        for x in 0..w.width {
            for y in 120..140 {
                let i = w.index(x, y);
                w.tiles[i].flags &= !FLAG_ACTIVE;
            }
        }
        w
    }

    fn web_count(w: &World) -> usize {
        w.tiles
            .iter()
            .filter(|t| t.active() && t.kind == 51)
            .count()
    }

    #[test]
    fn webs_hug_the_pocket_ceiling_and_replay() {
        let mut a = cave_world();
        let mut b = cave_world();
        let anchors = [0i32; 30];
        pass_webs(&mut a, 100.0, 90.0, 100.0, 0, &anchors, &anchors, &mut Rng(0xEB));
        pass_webs(&mut b, 100.0, 90.0, 100.0, 0, &anchors, &anchors, &mut Rng(0xEB));
        assert_eq!(a.tiles, b.tiles, "a == b");
        assert!(web_count(&a) > 0, "the pass places cobwebs");
        // Every cobweb sits inside the pocket band (the ceiling walk never
        // leaves the carved rows).
        for y in 0..a.height {
            for x in 0..a.width {
                if a.tile(x, y).is_some_and(|t| t.active() && t.kind == 51) {
                    assert!((120..140).contains(&y), "web outside the pocket");
                }
            }
        }
    }

    #[test]
    fn a_solid_world_grows_no_webs() {
        // Every candidate below worldSurfaceHigh is active — the entry
        // gate (`:13621`) rejects them all without a direction draw.
        let mut w = stone_world(600, 600);
        let anchors = [0i32; 30];
        pass_webs(&mut w, 100.0, 90.0, 100.0, 0, &anchors, &anchors, &mut Rng(0xEB));
        assert_eq!(web_count(&w), 0);
    }
}
