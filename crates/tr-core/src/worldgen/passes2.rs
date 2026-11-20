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
use crate::seeds::{MicroSeedFlags, SeedFlags};
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

/// The Slush pass (`StoneToIceAndSiltPlusMudIntoSlush`,
/// `WorldGen.cs:14697-14752`): the snow-rect retype sweep — silt 123 →
/// slush 224, mud 59 → slush 224 gated on the 7×7 jungle-grass scan (the
/// C#'s inner `break` leaves only the inner loop — the scan keeps walking
/// the outer columns, and a later hit still clears the flag), stone 1 →
/// ice 161. The `surfaceIsDesert && noSurface` micro-seed pair retypes to
/// sand 53 instead of slush (`:14708-14715`, `:14734-14741`). The C#
/// switches on `tile.type` without an active gate for 123/1 — kept
/// verbatim; the mud arm's scan checks `active()` on the neighbours.
/// Draws nothing.
pub(crate) fn pass_slush(
    world: &mut World,
    snow_top: i32,
    snow_bottom: i32,
    snow_min_x: &[i32],
    snow_max_x: &[i32],
) {
    let desert_no_surface = world.micro_seeds.contains(MicroSeedFlags::SURFACE_IS_DESERT)
        && world.micro_seeds.contains(MicroSeedFlags::NO_SURFACE);
    let slush_or_sand: u16 = if desert_no_surface { 53 } else { 224 };
    for i in snow_top..snow_bottom {
        for j in snow_min_x[i as usize]..snow_max_x[i as usize] {
            let Some(t) = world.tile(j, i) else { continue };
            match t.kind {
                123 => {
                    world.tile_mut(j, i).unwrap().kind = slush_or_sand;
                }
                59 => {
                    // The 7×7 jungle-grass scan (`:14719-14731`): 60/70/71/72.
                    let mut flag = true;
                    let num = 3;
                    for k in (j - num)..=(j + num) {
                        for l in (i - num)..=(i + num) {
                            if world.tile(k, l).is_some_and(|t| {
                                t.active() && matches!(t.kind, 60 | 70 | 71 | 72)
                            }) {
                                flag = false;
                                break;
                            }
                        }
                    }
                    if flag {
                        world.tile_mut(j, i).unwrap().kind = slush_or_sand;
                    }
                }
                1 => {
                    world.tile_mut(j, i).unwrap().kind = 161;
                }
                _ => {}
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

    #[test]
    fn slush_retypes_the_snow_rect_and_respects_the_jungle_scan() {
        let mut w = World::new("slush".into(), 100, 100);
        let put = |w: &mut World, x: i32, y: i32, kind: u16| {
            let i = w.index(x, y);
            w.tiles[i].kind = kind;
            w.tiles[i].flags |= FLAG_ACTIVE;
        };
        put(&mut w, 10, 10, 123); // silt → slush
        put(&mut w, 11, 10, 1); // stone → ice
        put(&mut w, 12, 10, 59); // mud near jungle grass → stays
        put(&mut w, 14, 12, 60); // the jungle grass (within 3 of 12,10)
        put(&mut w, 30, 10, 59); // mud far from grass → slush
        put(&mut w, 60, 10, 123); // outside the rect → stays silt
        let min_x = vec![5i32; 100];
        let max_x = vec![50i32; 100];
        pass_slush(&mut w, 8, 20, &min_x, &max_x);
        assert_eq!(w.tile(10, 10).unwrap().kind, 224);
        assert_eq!(w.tile(11, 10).unwrap().kind, 161);
        assert_eq!(w.tile(12, 10).unwrap().kind, 59, "the jungle scan vetoes");
        assert_eq!(w.tile(30, 10).unwrap().kind, 224);
        assert_eq!(w.tile(60, 10).unwrap().kind, 123, "outside the rect");
    }

    #[test]
    fn the_desert_no_surface_pair_retypes_to_sand() {
        let mut w = World::new("slush".into(), 100, 100);
        w.micro_seeds |= MicroSeedFlags::SURFACE_IS_DESERT | MicroSeedFlags::NO_SURFACE;
        let i = w.index(10, 10);
        w.tiles[i].kind = 123;
        w.tiles[i].flags |= FLAG_ACTIVE;
        let min_x = vec![5i32; 100];
        let max_x = vec![50i32; 100];
        pass_slush(&mut w, 8, 20, &min_x, &max_x);
        assert_eq!(w.tile(10, 10).unwrap().kind, 53);
    }

    #[test]
    fn the_empty_snow_rect_is_a_no_op() {
        // The blocked-producer state: snowTop == snowBottom == 0 until
        // Generate Ice Biome (d101) lands — the sweep touches nothing.
        let mut w = stone_world(100, 100);
        let before = w.tiles.clone();
        pass_slush(&mut w, 0, 0, &[], &[]);
        assert_eq!(w.tiles, before);
    }
}
