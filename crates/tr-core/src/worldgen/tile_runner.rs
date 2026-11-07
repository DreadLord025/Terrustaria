//! The `WorldGen.TileRunner` blob walker (`WorldGen.cs:76770-77142`).
//!
//! The random-walk routine vanilla uses for every ore vein, chasm, mud patch
//! and hellstone column: a point walks with a clamped velocity while carving a
//! shrinking Manhattan disc, re-rolling the disc wobble per candidate tile.
//!
//! The port covers the **normal-world** path (no drunk / remix / getGood /
//! notTheBees secret seeds exist here): their strength/steps modifiers
//! (`:76772-76788`) and jitter draws (`:76823-76827`, `:77088-77091`) never
//! fire, and the per-blob ladder condition `(!drunk || Next(3) != 0)`
//! short-circuits without a draw. Unported, waiting for their first caller
//! (see ANALYSIS "OresAndShinies fidelity (M5-ORES-1)"): the `mudWall` branch
//! (`:76875-76888`), the `type < 0` liquid path (`:76889-76913`), the
//! 367/368 wall placements (`:76914-76921`), the `type == 59` branches that
//! read `Main.rockLayer`/`GenVars.waterLine` (`:76995-76999`, `:77123-77141`).

use super::noise::Rng;
use crate::tile::{Tile, FLAG_ACTIVE, FLAG_LIQUID_LAVA};
use crate::tiledata::{SET_CAN_BE_CLEARED_DURING_GENERATION, SET_ORE, TILE_STONE};
use crate::tileobject::tile_cut;
use crate::world::World;

/// The defaulted `TileRunner` parameters (`WorldGen.cs:76770`): ore calls use
/// every default; the hellstone columns pass `add_tile`/`speed`/`no_y_change`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TileRunnerOpts {
    pub add_tile: bool,
    pub speed_x: f64,
    pub speed_y: f64,
    pub no_y_change: bool,
    pub over_ride: bool,
    pub ignore_tile_type: i32,
}

impl Default for TileRunnerOpts {
    fn default() -> Self {
        Self {
            add_tile: false,
            speed_x: 0.0,
            speed_y: 0.0,
            no_y_change: false,
            over_ride: true,
            ignore_tile_type: -1,
        }
    }
}

/// The speed-up ladder thresholds (`WorldGen.cs:77003-77086`): the C# nests one
/// `if (num > t)` level per threshold, each level advancing the walker an extra
/// step and re-rolling both velocity components. The flat table walks the same
/// thresholds in the same order, so the draw sequence is identical.
const LADDER_THRESHOLDS: [f64; 12] = [
    50.0, 100.0, 150.0, 200.0, 250.0, 300.0, 400.0, 500.0, 600.0, 700.0, 800.0, 900.0,
];

/// The ebonstone-family protection (`WorldGen.cs:76947-76957`): the runner
/// never replaces these existing tiles.
fn protected_stone_family(existing: u16) -> bool {
    matches!(existing, 45 | 147 | 189 | 190 | 196 | 460 | 717 | 718 | 719)
}

/// One `TileRunner` call. `strength`/`steps` are the C# blob size and step
/// count, `world_surface` is `Main.worldSurface` (read by the sand protection
/// and the `no_y_change` dirt-wall lining).
pub(crate) fn tile_runner(
    world: &mut World,
    rng: &mut Rng,
    i: i32,
    j: i32,
    strength: f64,
    steps: i32,
    kind: i32,
    world_surface: f64,
    opts: &TileRunnerOpts,
) {
    // The `type < 0` liquid-carving path (`WorldGen.cs:76889-76913`) is not
    // ported; it lands with its first caller.
    if kind < 0 {
        return;
    }
    let mut num = strength;
    let mut num2 = steps as f64;
    let (mut x, mut y) = (i as f64, j as f64);
    // The initial velocity always draws, even when speedX/speedY override it
    // right after (`WorldGen.cs:76795-76801`).
    let mut vel_x = rng.range(-10, 11) as f64 * 0.1;
    let mut vel_y = rng.range(-10, 11) as f64 * 0.1;
    if opts.speed_x != 0.0 || opts.speed_y != 0.0 {
        vel_x = opts.speed_x;
        vel_y = opts.speed_y;
    }
    // The runner kind keeps its slope cleared: `flag = type >= 0 &&
    // !TileID.Sets.SaveSlopes[type]` (`:76802`) and `SaveSlopes` is empty
    // (`TileID.cs:468`).
    let clears_slope = kind >= 0;

    // The `num3` liquid-type lottery (`:76806-76820`) draws `Next(4)` on every
    // call even though every branch is secret-seed-only and the result is a
    // no-op for solid kinds — the draw is part of the stream contract.
    let _liquid_pick = rng.range(0, 4);

    while num > 0.0 && num2 > 0.0 {
        // The drunk jitter (`:76823-76827`) and the tenth-anniversary `num3`
        // reset (`:76828-76831`) are secret-seed-only.
        // The mud early exit (`:76832-76835`).
        if kind == 59 && y < 0.0 && num2 > 0.0 {
            num2 = 0.0;
        }
        num = strength * (num2 / steps as f64);
        num2 -= 1.0;
        let (mut left, mut right) = ((x - num * 0.5) as i32, (x + num * 0.5) as i32);
        let (mut top, mut bottom) = ((y - num * 0.5) as i32, (y + num * 0.5) as i32);
        if left < 1 {
            left = 1;
        }
        if right > world.width - 1 {
            right = world.width - 1;
        }
        if top < 1 {
            top = 1;
        }
        if bottom > world.height - 1 {
            bottom = world.height - 1;
        }
        for k in left..right {
            // The beach `num3` reset (`:76860-76863`) only feeds the unported
            // liquid path.
            for l in top..bottom {
                let Some(tile) = world.tile(k, l) else {
                    continue;
                };
                // The frame-important guard (`:76871`) skips without a draw;
                // `Main.tileFrameImportant` defaults to false for out-of-range
                // ids just like the port's `get`.
                let frame_guard = tile.active()
                    && world
                        .frame_important
                        .get(tile.kind as usize)
                        .copied()
                        .unwrap_or(false)
                    && !tile_cut(tile.kind);
                let ignored = opts.ignore_tile_type >= 0
                    && tile.active()
                    && tile.kind as i32 == opts.ignore_tile_type;
                if frame_guard || ignored {
                    continue;
                }
                // The Manhattan wobble draws per candidate tile and tests
                // against the *original* strength, not the shrinking `num`
                // (`:76871`).
                let wobble = rng.range(-10, 11) as f64;
                let manhattan = (k as f64 - x).abs() + (l as f64 - y).abs();
                if manhattan >= strength * 0.5 * (1.0 + wobble * 0.015) {
                    continue;
                }
                let (existing, was_active) = (tile.kind, tile.active());
                // The `flag4` protection chain (`:76924-76975`).
                let mut protected = false;
                if opts.over_ride && was_active {
                    // `Main.tileStone[type] && tile.type != 1` (`:76926`) —
                    // `TILE_STONE` (the `Main.tileStone` array) holds only the
                    // gems and mosses, never an ore kind, so the first half is
                    // false for every current runner.
                    protected = TILE_STONE.has(kind as u16) && existing != 1;
                    if !SET_CAN_BE_CLEARED_DURING_GENERATION.has(existing) {
                        protected = true;
                    }
                    match existing {
                        // Sand: the underground-desert containment (`:76934`)
                        // needs `GenVars.UndergroundDesertLocation`; the
                        // portable halves are the clay runner and the surface
                        // sand protection.
                        53 => {
                            if kind == 40 {
                                protected = true;
                            }
                            if (l as f64) < world_surface && kind != 59 {
                                protected = true;
                            }
                        }
                        45 | 147 | 189 | 190 | 196 | 460 | 717 | 718 | 719 => {
                            protected = true;
                        }
                        // Sandstone/hardened sand yield to ore runners only
                        // (`:76958-76961`).
                        396 | 397 => protected = !SET_ORE.has(kind as u16),
                        // Stone: the mud case draws `Next(-50, 50)`
                        // (`:76962-76967`) — unported with the mud path.
                        1 => {}
                        // `:76968-76974`: the 367/368 shells yield to mud only.
                        367 | 368 => protected = kind == 59,
                        _ => {}
                    }
                }
                let Some(tile) = world.tile_mut(k, l) else {
                    continue;
                };
                if !protected {
                    tile.kind = kind as u16;
                    if clears_slope {
                        tile.set_slope(0);
                    }
                }
                if opts.add_tile {
                    tile.flags |= FLAG_ACTIVE;
                    tile.liquid = 0;
                    tile.flags &= !FLAG_LIQUID_LAVA;
                }
                // The `no_y_change` dirt-wall lining (`:76991-76994`).
                if opts.no_y_change && (l as f64) < world_surface && kind != 59 {
                    tile.wall = 2;
                }
                // The mud liquid clear (`:76995-76999`) reads
                // `GenVars.waterLine` — unported with the mud path.
            }
        }
        x += vel_x;
        y += vel_y;
        for threshold in LADDER_THRESHOLDS {
            if num <= threshold {
                break;
            }
            x += vel_x;
            y += vel_y;
            num2 -= 1.0;
            vel_y += rng.range(-10, 11) as f64 * 0.05;
            vel_x += rng.range(-10, 11) as f64 * 0.05;
        }
        // The velocity drift (`:77087-77110`): X always, Y when the walker is
        // allowed to change depth; both clamp to ±1.
        vel_x += rng.range(-10, 11) as f64 * 0.05;
        vel_x = vel_x.clamp(-1.0, 1.0);
        if !opts.no_y_change {
            vel_y += rng.range(-10, 11) as f64 * 0.05;
            vel_y = vel_y.clamp(-1.0, 1.0);
        } else if kind != 59 && num < 3.0 {
            vel_y = vel_y.clamp(-1.0, 1.0);
        }
        // The mud velocity clamps (`:77123-77141`) read `Main.rockLayer` —
        // unported with the mud path.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stone_world() -> World {
        let mut world = World::new("test".into(), 200, 200);
        for i in 0..world.tiles.len() {
            let mut tile = Tile::default();
            tile.kind = 1;
            tile.flags |= FLAG_ACTIVE;
            world.tiles[i] = tile;
        }
        world
    }

    /// DW-3.1: the draw ledger. On an all-stone world every candidate tile
    /// draws its wobble, so the replay below is the full C# sequence for an
    /// ore call: 2 velocity draws, 1 `Next(4)` lottery, then per iteration the
    /// wobbles of the `strength`-sized rect plus 2 drift draws. With `steps =
    /// 1` the walker has not moved yet, so the rect is the exact integer-start
    /// disc `(int)(i ± strength/2)` — no velocity dependence.
    #[test]
    fn draw_ledger_matches_the_csharp_replay() {
        let cases: &[(f64, usize)] = &[
            // strength 1.0 → rect (int)(50±0.5) = 1×1 tile → 1 wobble.
            (1.0, 1),
            // strength 3.5 → (int)(50±1.75) = 3×3 → 9 wobbles.
            (3.5, 9),
            // strength 4.0 → (int)(50±2) = 4×4 → 16 wobbles.
            (4.0, 16),
        ];
        for &(strength, wobbles) in cases {
            let mut world = stone_world();
            let mut runner = Rng(0xDEAD_BEEF);
            tile_runner(
                &mut world,
                &mut runner,
                50,
                50,
                strength,
                1,
                7,
                200.0,
                &TileRunnerOpts::default(),
            );
            let mut replay = Rng(0xDEAD_BEEF);
            for _ in 0..2 {
                replay.range(-10, 11); // initial velocity
            }
            replay.range(0, 4); // the num3 lottery
            for _ in 0..wobbles {
                replay.range(-10, 11); // the per-tile wobble
            }
            replay.range(-10, 11); // vx drift
            replay.range(-10, 11); // vy drift
            assert_eq!(runner.0, replay.0, "strength {strength} ledger");
        }
    }

    /// DW-3.2: a strength-1 walker touches at most one tile per step and
    /// inactive tiles carry the kind without being activated.
    #[test]
    fn strength_one_touches_at_most_one_tile_per_step() {
        let mut world = World::new("test".into(), 200, 200);
        let mut rng = Rng(7);
        tile_runner(
            &mut world,
            &mut rng,
            100,
            100,
            1.0,
            3,
            7,
            200.0,
            &TileRunnerOpts::default(),
        );
        let touched: Vec<usize> = world
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| t.kind == 7)
            .map(|(i, _)| i)
            .collect();
        assert!(
            touched.len() <= 3,
            "one tile per step, got {}",
            touched.len()
        );
        for i in touched {
            assert_eq!(
                world.tiles[i].flags & FLAG_ACTIVE,
                0,
                "inactive stays inactive"
            );
        }
    }

    /// DW-3.3: the protection chain — frame-important and generation-protected
    /// tiles are untouched, sand above the surface stands, sandstone yields to
    /// ores, stone is replaced.
    #[test]
    fn protection_rules_match_the_csharp_switch() {
        let mut world = World::new("test".into(), 200, 200);
        for i in 0..world.tiles.len() {
            let mut tile = Tile::default();
            tile.kind = 1; // stone: clearable, replaced
            tile.flags |= FLAG_ACTIVE;
            world.tiles[i] = tile;
        }
        // (100, 100) silver brick 41 — generation-protected (TileID.cs:573).
        let mut tile = Tile::default();
        tile.kind = 41;
        tile.flags |= FLAG_ACTIVE;
        let idx = world.index(100, 100);
        world.tiles[idx] = tile;
        // (101, 100) a frame-important tile: the guard skips it before any draw.
        let mut tile = Tile::default();
        tile.kind = 10;
        tile.flags |= FLAG_ACTIVE;
        let idx = world.index(101, 100);
        world.tiles[idx] = tile;
        world.frame_important = vec![false; 754];
        world.frame_important[10] = true;
        // (102, 100) surface sand above world_surface.
        let mut tile = Tile::default();
        tile.kind = 53;
        tile.flags |= FLAG_ACTIVE;
        let idx = world.index(102, 100);
        world.tiles[idx] = tile;
        // (103, 100) sandstone 396 — ore runners replace it.
        let mut tile = Tile::default();
        tile.kind = 396;
        tile.flags |= FLAG_ACTIVE;
        let idx = world.index(103, 100);
        world.tiles[idx] = tile;

        let mut rng = Rng(9);
        // world_surface 150: rows above it (y < 150) count as "above the
        // surface" for the sand protection.
        tile_runner(
            &mut world,
            &mut rng,
            100,
            100,
            60.0,
            4,
            7,
            150.0,
            &TileRunnerOpts::default(),
        );

        assert_eq!(
            world.tile(100, 100).unwrap().kind,
            41,
            "silver brick stands"
        );
        assert_eq!(
            world.tile(101, 100).unwrap().kind,
            10,
            "frame-important stands"
        );
        assert_eq!(
            world.tile(102, 100).unwrap().kind,
            53,
            "surface sand stands"
        );
        assert_eq!(
            world.tile(103, 100).unwrap().kind,
            7,
            "sandstone yields to ore"
        );
        assert_eq!(world.tile(95, 100).unwrap().kind, 7, "stone is replaced");
    }
}
