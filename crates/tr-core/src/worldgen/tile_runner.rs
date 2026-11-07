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
//! short-circuits without a draw. Ported for the M5-BIOME passes (the wall
//! arm `:76875-76888`, the `type < 0` liquid path `:76889-76913`, the
//! `type == 59` branches `:76963/:76995-76999/:77123-77141`). Unported,
//! waiting for their first caller: the 367/368 wall placements
//! (`:76914-76921`) and the `case 53` underground-desert protection
//! (`:76934` — the port carries no `GenVars.UndergroundDesertLocation`).

use super::noise::Rng;
use super::{TILE_SAND, WALL_DIRT};
use crate::tile::{LiquidKind, Tile, FLAG_ACTIVE, FLAG_LIQUID_LAVA};
use crate::tiledata::{SET_CAN_BE_CLEARED_DURING_GENERATION, SET_ORE, TILE_STONE};
use crate::tileobject::tile_cut;
use crate::world::World;

/// The defaulted `TileRunner` parameters (`WorldGen.cs:76770`): ore calls use
/// every default; the hellstone columns pass `add_tile`/`speed`/`no_y_change`;
/// the M5-BIOME mud runners pass `mud_wall` and the `GenVars` depth anchors.
/// The depth-anchor defaults (0.0) are only inert because no current caller
/// runs `kind == 59` or a negative kind — the depth branches gate on those
/// kinds, and the wall arm gates on `mud_wall`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TileRunnerOpts {
    pub add_tile: bool,
    pub speed_x: f64,
    pub speed_y: f64,
    pub no_y_change: bool,
    pub over_ride: bool,
    pub ignore_tile_type: i32,
    /// `GenVars.mudWall` (`WorldGen.cs:76772`, `:76875`): arms the mud/jungle
    /// wall lining inside the runner disc.
    pub mud_wall: bool,
    /// `Main.rockLayer` — read by the kind-59 downward force (`:77133`).
    pub rock_layer: f64,
    /// `GenVars.waterLine` — read by the kind-59 liquid clear (`:76995`) and
    /// the type -2 water fill (`:76895`).
    pub water_line: f64,
    /// `GenVars.lavaLine` — read by the wall-arm depth split (`:76877`) and
    /// the type -2 lava fill (`:76895`, `:76906`).
    pub lava_line: f64,
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
            mud_wall: false,
            rock_layer: 0.0,
            water_line: 0.0,
            lava_line: 0.0,
        }
    }
}

/// `WallID.MudUnsafe` = 15 (`WorldGen.cs:76881`) — the mud-body lining placed
/// in the deep half of the wall band.
const WALL_MUD_UNSAFE: u16 = 15;
/// `WallID.JungleUnsafe` = 64 (`WorldGen.cs:76886`) — the shallow half. The
/// same id as `biomes.rs::WALL_JUNGLE`; kept local so this leaf mechanic does
/// not depend on the pass modules.
const WALL_JUNGLE_UNSAFE: u16 = 64;

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

/// `WorldGen.PlaceWall` (`WorldGen.cs:75125-75144`): the boundary guard, then
/// a write only onto an empty wall. `SquareWallFrame` is a visual frame pass
/// the port defers — the same convention as every other worldgen wall write.
fn place_wall(world: &mut World, k: i32, l: i32, wall: u16) {
    if k <= 1 || l <= 1 || k >= world.width - 2 || l >= world.height - 2 {
        return;
    }
    if let Some(tile) = world.tile_mut(k, l) {
        if tile.wall == 0 {
            tile.wall = wall;
        }
    }
}

/// `Main.tile[x, y].wall` with out-of-bounds reading as 0. The clamped walker
/// rect (`:76842-76857`) keeps every neighbour read of the wall arm
/// (`:76875-76887`) inside the array; the `Option` is belt-and-braces.
fn wall_at(world: &World, x: i32, y: i32) -> u16 {
    world.tile(x, y).map_or(0, |t| t.wall)
}

/// One `TileRunner` call. `strength`/`steps` are the C# blob size and step
/// count, `world_surface` is `Main.worldSurface` (read by the sand protection,
/// the `no_y_change` dirt-wall lining, the mud-wall arm and the kind-59 stone
/// protection); the `GenVars` depth anchors ride in `opts`.
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
            // The beach `num3` reset (`:76860-76863`): `num3` is 0 on normal
            // worlds either way, so the reset only matters for the type -2
            // `liquidType(num3)` — always water here.
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
                // The `GenVars.mudWall` wall arm (`WorldGen.cs:76875-76888`):
                // the mud body's inner lining. Every gate short-circuits in
                // order — `Next(3)` draws only past the surface with a
                // non-dirt wall above, the 0.45-disc wobble only past that,
                // and `Next(0, 4)` only inside the 0.45 disc.
                if opts.mud_wall
                    && (l as f64) > world_surface
                    && wall_at(world, k, l - 1) != WALL_DIRT
                    && l < world.height - 210 - rng.range(0, 3)
                    && manhattan < strength * 0.45 * (1.0 + rng.range(-10, 11) as f64 * 0.01)
                {
                    if (l as f64) > opts.lava_line - rng.range(0, 4) as f64 - 50.0 {
                        if wall_at(world, k, l - 1) != WALL_JUNGLE_UNSAFE
                            && wall_at(world, k, l + 1) != WALL_JUNGLE_UNSAFE
                            && wall_at(world, k - 1, l) != WALL_JUNGLE_UNSAFE
                            && wall_at(world, k + 1, l) != WALL_JUNGLE_UNSAFE
                        {
                            place_wall(world, k, l, WALL_MUD_UNSAFE);
                        }
                    } else if wall_at(world, k, l - 1) != WALL_MUD_UNSAFE
                        && wall_at(world, k, l + 1) != WALL_MUD_UNSAFE
                        && wall_at(world, k - 1, l) != WALL_MUD_UNSAFE
                        && wall_at(world, k + 1, l) != WALL_MUD_UNSAFE
                    {
                        place_wall(world, k, l, WALL_JUNGLE_UNSAFE);
                    }
                }
                // The `type < 0` liquid-carving path (`WorldGen.cs:76889-76913`):
                // -1 hollows empty pockets, -2 fills them; sand survives both.
                if kind < 0 {
                    if was_active && existing == TILE_SAND {
                        continue;
                    }
                    if kind == -2
                        && was_active
                        && ((l as f64) < opts.water_line || (l as f64) > opts.lava_line)
                    {
                        if let Some(tile) = world.tile_mut(k, l) {
                            tile.liquid = u8::MAX;
                            // `liquidType(num3)` with the normal-world
                            // `num3` = 0 → water (`:76898`).
                            tile.set_liquid_kind(LiquidKind::Water);
                            if (l as f64) > opts.lava_line {
                                tile.set_liquid_kind(LiquidKind::Lava);
                            }
                        }
                    }
                    if let Some(tile) = world.tile_mut(k, l) {
                        tile.flags &= !FLAG_ACTIVE;
                    }
                    continue;
                }
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
                        // Stone: the kind-59 case draws `Next(-50, 50)` for
                        // every active stone candidate and protects the band
                        // just under the surface (`WorldGen.cs:76962-76967`).
                        1 => {
                            if kind == 59 {
                                let stone_wobble = rng.range(-50, 50) as f64;
                                if (l as f64) < world_surface + stone_wobble {
                                    protected = true;
                                }
                            }
                        }
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
                // The kind-59 liquid clear (`WorldGen.cs:76995-76999`): mud
                // displaces every liquid at or below `GenVars.waterLine` —
                // even on a protected tile, since it runs after the flag4
                // chain.
                if kind == 59 && (l as f64) > opts.water_line && tile.liquid > 0 {
                    tile.flags &= !FLAG_LIQUID_LAVA;
                    tile.liquid = 0;
                }
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
        // The kind-59 depth clamps (`WorldGen.cs:77123-77141`): the mud walk
        // drifts at most half a tile per step, dives straight down while
        // above `Main.rockLayer + 100` and climbs straight up near the world
        // bottom. The forces override the clamp after it.
        if kind == 59 && !opts.no_y_change {
            if vel_y > 0.5 {
                vel_y = 0.5;
            }
            if vel_y < -0.5 {
                vel_y = -0.5;
            }
            if y < opts.rock_layer + 100.0 {
                vel_y = 1.0;
            }
            if y > (world.height - 300) as f64 {
                vel_y = -1.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stone_world_sized(width: i32, height: i32) -> World {
        let mut world = World::new("test".into(), width, height);
        for i in 0..world.tiles.len() {
            let mut tile = Tile::default();
            tile.kind = 1;
            tile.flags |= FLAG_ACTIVE;
            world.tiles[i] = tile;
        }
        world
    }

    fn stone_world() -> World {
        stone_world_sized(200, 200)
    }

    /// Overwrites one tile wholesale (kind, wall, liquid, active).
    fn put(world: &mut World, x: i32, y: i32, kind: u16, wall: u16) {
        let idx = world.index(x, y);
        let mut tile = Tile::default();
        tile.kind = kind;
        tile.wall = wall;
        tile.flags |= FLAG_ACTIVE;
        world.tiles[idx] = tile;
    }

    fn add_liquid(world: &mut World, x: i32, y: i32, amount: u8, lava: bool) {
        let idx = world.index(x, y);
        world.tiles[idx].liquid = amount;
        if lava {
            world.tiles[idx].flags |= FLAG_LIQUID_LAVA;
        }
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

    /// The defaults keep every new branch inert: the wall arm gates on
    /// `mud_wall`, the depth branches on kinds no current caller uses (the
    /// only call site is `caves.rs:180`, positive ore kinds).
    #[test]
    fn the_default_opts_keep_every_new_branch_inert() {
        let o = TileRunnerOpts::default();
        assert!(!o.mud_wall);
        assert_eq!((o.rock_layer, o.water_line, o.lava_line), (0.0, 0.0, 0.0));
    }

    /// DW-2.1: the `mudWall` wall arm ledger. A strength-4 single-step runner
    /// at (50, 50) covers the rect k,l ∈ 48..52 (16 candidates). With every
    /// tile passing the surface/wall-above gates, the replay per candidate is
    /// the 0.5-disc wobble (`:76871`), then `Next(3)` (`:76875`), the
    /// 0.45-disc wobble (`:76875`), and `Next(0, 4)` exactly when the
    /// candidate sits inside the 0.45 disc (`:76877`) — Manhattan ≤ 1 always
    /// passes it (threshold spans [1.62, 1.98]), Manhattan ≥ 2 never does.
    #[test]
    fn mud_wall_arm_replays_the_csharp_draws() {
        let mut world = stone_world_sized(200, 400);
        let mut runner = Rng(0xDEAD_BEEF);
        tile_runner(
            &mut world,
            &mut runner,
            50,
            50,
            4.0,
            1,
            7,
            0.0,
            &TileRunnerOpts {
                mud_wall: true,
                lava_line: 0.0,
                ..TileRunnerOpts::default()
            },
        );
        let mut replay = Rng(0xDEAD_BEEF);
        replay.range(-10, 11); // initial velocity x
        replay.range(-10, 11); // initial velocity y
        replay.range(0, 4); // the num3 lottery
        for k in 48i32..52 {
            for l in 48i32..52 {
                let w1 = replay.range(-10, 11) as f64;
                let manhattan = ((k - 50).abs() + (l - 50).abs()) as f64;
                if manhattan >= 4.0 * 0.5 * (1.0 + w1 * 0.015) {
                    continue; // :76871
                }
                replay.range(0, 3); // :76875 — the band draw
                let w2 = replay.range(-10, 11) as f64; // :76875 — the 0.45 wobble
                if manhattan < 4.0 * 0.45 * (1.0 + w2 * 0.01) {
                    replay.range(0, 4); // :76877 — the lava-line split draw
                }
            }
        }
        replay.range(-10, 11); // vx drift
        replay.range(-10, 11); // vy drift
        assert_eq!(runner.0, replay.0, "mud wall arm ledger");
    }

    /// The wall arm places 15 below the lava-line pocket and 64 above it
    /// (`:76877-76887`), only through `PlaceWall`'s empty-wall rule
    /// (`:75125-75144`), and never onto a tile whose neighbour already carries
    /// the other wall. The Manhattan-≤1 tiles always qualify; Manhattan-3/4
    /// tiles never pass the 0.5 disc.
    #[test]
    fn mud_wall_places_15_or_64_by_the_lava_line() {
        let always: &[(i32, i32)] = &[(50, 50), (49, 50), (51, 50), (50, 49), (50, 51)];
        // (a) lava_line 0: every candidate is deep → wall 15.
        let mut world = stone_world_sized(200, 400);
        tile_runner(
            &mut world,
            &mut Rng(0xDEAD_BEEF),
            50,
            50,
            4.0,
            1,
            7,
            0.0,
            &TileRunnerOpts {
                mud_wall: true,
                lava_line: 0.0,
                ..TileRunnerOpts::default()
            },
        );
        for &(x, y) in always {
            assert_eq!(world.tile(x, y).unwrap().wall, 15, "({x},{y}) mud wall");
        }
        for &(x, y) in &[(48, 48), (48, 49), (49, 48), (48, 51), (51, 48)] {
            assert_eq!(world.tile(x, y).unwrap().wall, 0, "({x},{y}) outside disc");
        }
        assert!(
            !world.tiles.iter().any(|t| t.wall == 64),
            "no jungle wall below the lava pocket"
        );
        // (b) lava_line 200: every candidate is shallow → wall 64.
        let mut world = stone_world_sized(200, 400);
        tile_runner(
            &mut world,
            &mut Rng(0xDEAD_BEEF),
            50,
            50,
            4.0,
            1,
            7,
            0.0,
            &TileRunnerOpts {
                mud_wall: true,
                lava_line: 200.0,
                ..TileRunnerOpts::default()
            },
        );
        for &(x, y) in always {
            assert_eq!(world.tile(x, y).unwrap().wall, 64, "({x},{y}) jungle wall");
        }
        assert!(
            !world.tiles.iter().any(|t| t.wall == 15),
            "no mud wall above the lava pocket"
        );
        // (c) a neighbour already carrying 64 blocks the 15 write on the tile
        // next to it (`:76879`), and PlaceWall never overwrites a wall
        // (`:75135`).
        let mut world = stone_world_sized(200, 400);
        let idx = world.index(49, 50);
        world.tiles[idx].wall = 64;
        let idx = world.index(50, 50);
        world.tiles[idx].wall = 2;
        tile_runner(
            &mut world,
            &mut Rng(0xDEAD_BEEF),
            50,
            50,
            4.0,
            1,
            7,
            0.0,
            &TileRunnerOpts {
                mud_wall: true,
                lava_line: 0.0,
                ..TileRunnerOpts::default()
            },
        );
        assert_eq!(world.tile(49, 50).unwrap().wall, 64, "64 stands");
        assert_eq!(world.tile(50, 50).unwrap().wall, 2, "dirt wall stands");
        assert_eq!(
            world.tile(50, 49).unwrap().wall,
            15,
            "unblocked neighbour still lined"
        );
        assert_eq!(
            world.tile(50, 51).unwrap().wall,
            0,
            "dirt wall above kills the arm for the tile below it"
        );
    }

    /// The arm's gates short-circuit before their draws: a dirt wall above
    /// (`:76875`) or a candidate at/above the surface leaves the ledger at
    /// the plain walk — no `Next(3)`, no 0.45 wobble, no `Next(0, 4)`.
    #[test]
    fn mud_wall_arm_short_circuits_without_draws() {
        // (a) every wall-above read sees dirt wall 2.
        let mut world = stone_world_sized(200, 400);
        for tile in world.tiles.iter_mut() {
            tile.wall = 2;
        }
        let mut runner = Rng(0xDEAD_BEEF);
        tile_runner(
            &mut world,
            &mut runner,
            50,
            50,
            4.0,
            1,
            7,
            0.0,
            &TileRunnerOpts {
                mud_wall: true,
                ..TileRunnerOpts::default()
            },
        );
        let mut replay = Rng(0xDEAD_BEEF);
        replay.range(-10, 11);
        replay.range(-10, 11);
        replay.range(0, 4);
        for _ in 0..16 {
            replay.range(-10, 11); // the 0.5 wobble per candidate
        }
        replay.range(-10, 11);
        replay.range(-10, 11);
        assert_eq!(runner.0, replay.0, "dirt wall above short-circuits");
        // (b) the whole rect sits at/above the surface.
        let mut world = stone_world_sized(200, 400);
        let mut runner = Rng(0xDEAD_BEEF);
        tile_runner(
            &mut world,
            &mut runner,
            50,
            50,
            4.0,
            1,
            7,
            60.0,
            &TileRunnerOpts {
                mud_wall: true,
                ..TileRunnerOpts::default()
            },
        );
        let mut replay = Rng(0xDEAD_BEEF);
        replay.range(-10, 11);
        replay.range(-10, 11);
        replay.range(0, 4);
        for _ in 0..16 {
            replay.range(-10, 11);
        }
        replay.range(-10, 11);
        replay.range(-10, 11);
        assert_eq!(runner.0, replay.0, "surface gate short-circuits");
    }

    /// DW-2.1: a negative kind walks and draws the full C# sequence — the old
    /// port early-returned before any draw, the C# never does
    /// (`:76889-76913` sits inside the tile loop). No per-tile draws beyond
    /// the 0.5 wobble exist on this path.
    #[test]
    fn negative_kind_draws_the_full_csharp_walk() {
        let mut world = stone_world();
        let mut runner = Rng(0xDEAD_BEEF);
        tile_runner(
            &mut world,
            &mut runner,
            50,
            50,
            4.0,
            1,
            -1,
            0.0,
            &TileRunnerOpts::default(),
        );
        let mut replay = Rng(0xDEAD_BEEF);
        replay.range(-10, 11);
        replay.range(-10, 11);
        replay.range(0, 4);
        for _ in 0..16 {
            replay.range(-10, 11);
        }
        replay.range(-10, 11);
        replay.range(-10, 11);
        assert_eq!(runner.0, replay.0, "negative-kind ledger");
        assert!(
            !world.tile(50, 50).unwrap().active(),
            "the pocket was carved"
        );
    }

    /// The `type < 0` path (`:76889-76913`): -1 hollows pockets, -2 fills
    /// them with water above `waterLine` / lava below `lavaLine`, sand
    /// survives both, and every non-sand tile is deactivated.
    #[test]
    fn negative_kind_carves_pockets_and_skips_sand() {
        // -1: stone deactivated, sand untouched.
        let mut world = stone_world();
        put(&mut world, 50, 42, 53, 0);
        tile_runner(
            &mut world,
            &mut Rng(11),
            50,
            42,
            4.0,
            1,
            -1,
            0.0,
            &TileRunnerOpts::default(),
        );
        let sand = world.tile(50, 42).unwrap();
        assert!(sand.active() && sand.kind == 53, "sand survives the carve");
        assert!(!world.tile(49, 42).unwrap().active(), "stone hollowed");
        // -2 water: active tiles above the water line fill with water.
        let mut world = stone_world();
        tile_runner(
            &mut world,
            &mut Rng(11),
            50,
            42,
            4.0,
            1,
            -2,
            0.0,
            &TileRunnerOpts {
                water_line: 43.0,
                lava_line: 100.0,
                ..TileRunnerOpts::default()
            },
        );
        let filled = world.tile(50, 42).unwrap();
        assert_eq!(filled.liquid, 255, "pocket filled");
        assert_eq!(
            filled.flags & FLAG_LIQUID_LAVA,
            0,
            "water above the water line"
        );
        assert!(!filled.active(), "filled tile deactivated");
        assert_eq!(
            world.tile(50, 43).unwrap().liquid,
            0,
            "between the lines: carved dry"
        );
        // -2 lava: tiles below the lava line fill with lava.
        let mut world = stone_world();
        tile_runner(
            &mut world,
            &mut Rng(11),
            50,
            42,
            4.0,
            1,
            -2,
            0.0,
            &TileRunnerOpts {
                water_line: 0.0,
                lava_line: 41.5,
                ..TileRunnerOpts::default()
            },
        );
        let lava = world.tile(50, 42).unwrap();
        assert_eq!(lava.liquid, 255, "deep pocket filled");
        assert_eq!(
            lava.flags & FLAG_LIQUID_LAVA,
            FLAG_LIQUID_LAVA,
            "lava below the lava line"
        );
        assert_eq!(
            world.tile(50, 41).unwrap().liquid,
            0,
            "between the lines: carved dry"
        );
        // -2 skips sand entirely.
        let mut world = stone_world();
        put(&mut world, 50, 42, 53, 0);
        tile_runner(
            &mut world,
            &mut Rng(11),
            50,
            42,
            4.0,
            1,
            -2,
            0.0,
            &TileRunnerOpts {
                water_line: 43.0,
                ..TileRunnerOpts::default()
            },
        );
        let sand = world.tile(50, 42).unwrap();
        assert!(
            sand.active() && sand.kind == 53 && sand.liquid == 0,
            "sand survives the fill"
        );
    }

    /// DW-2.1: the kind-59 ledger — the stone protection `case 1` draws
    /// `Next(-50, 50)` for every active stone candidate that passes the 0.5
    /// disc (`:76962-76967`, the draw precedes the comparison), and the depth
    /// clamps draw nothing. Below a surface of 0 the protection can never
    /// fire, so the disc centre turns to mud.
    #[test]
    fn kind_59_ledger_includes_the_stone_draws() {
        let mut world = stone_world();
        let mut runner = Rng(0xDEAD_BEEF);
        tile_runner(
            &mut world,
            &mut runner,
            50,
            50,
            4.0,
            1,
            59,
            0.0,
            &TileRunnerOpts::default(),
        );
        let mut replay = Rng(0xDEAD_BEEF);
        replay.range(-10, 11);
        replay.range(-10, 11);
        replay.range(0, 4);
        for k in 48i32..52 {
            for l in 48i32..52 {
                let w1 = replay.range(-10, 11) as f64;
                let manhattan = ((k - 50).abs() + (l - 50).abs()) as f64;
                if manhattan >= 4.0 * 0.5 * (1.0 + w1 * 0.015) {
                    continue; // :76871
                }
                replay.range(-50, 50); // :76963 — the stone wobble
            }
        }
        replay.range(-10, 11);
        replay.range(-10, 11);
        assert_eq!(runner.0, replay.0, "kind-59 ledger");
        assert_eq!(world.tile(50, 50).unwrap().kind, 59, "centre mudded");
    }

    /// The kind-59 liquid clear (`:76995-76999`): every liquid at/below the
    /// water line inside the disc is emptied and de-lava'd — even on a
    /// protected tile, because the clear runs after the flag4 chain. Tiles
    /// outside the disc keep their liquid.
    #[test]
    fn kind_59_clears_liquid_below_the_water_line() {
        let mut world = stone_world();
        add_liquid(&mut world, 50, 50, 255, true);
        add_liquid(&mut world, 49, 50, 128, false);
        put(&mut world, 50, 51, 41, 0); // silver brick: generation-protected
        add_liquid(&mut world, 50, 51, 255, true);
        add_liquid(&mut world, 52, 52, 255, true); // outside the disc
        tile_runner(
            &mut world,
            &mut Rng(0xDEAD_BEEF),
            50,
            50,
            4.0,
            1,
            59,
            0.0,
            &TileRunnerOpts {
                water_line: 45.0,
                ..TileRunnerOpts::default()
            },
        );
        let centre = world.tile(50, 50).unwrap();
        assert_eq!(centre.kind, 59, "stone mudded");
        assert_eq!(centre.liquid, 0, "liquid displaced");
        assert_eq!(centre.flags & FLAG_LIQUID_LAVA, 0, "lava flag cleared");
        assert_eq!(world.tile(49, 50).unwrap().liquid, 0, "water displaced");
        let brick = world.tile(50, 51).unwrap();
        assert_eq!(brick.kind, 41, "brick protected");
        assert_eq!(brick.liquid, 0, "but its liquid still displaced");
        let outside = world.tile(52, 52).unwrap();
        assert_eq!(outside.liquid, 255, "outside the disc: untouched");
        assert_eq!(
            outside.flags & FLAG_LIQUID_LAVA,
            FLAG_LIQUID_LAVA,
            "outside the disc: lava intact"
        );
    }

    /// The kind-59 depth clamps (`:77123-77141`): above `rockLayer + 100` the
    /// walker dives at exactly one tile per step; in the free band it drifts
    /// at most half a tile per step. Neither clamp draws.
    #[test]
    fn kind_59_velocity_clamps_read_the_depth_anchors() {
        // (a) the downward force: rock_layer 0 → every y < 100 dives. Over 30
        // steps the forced walker reaches row ~67 (40 + v0y + 28), while a
        // walker stuck at the ±0.5 clamp could only reach ~55 — the deepest
        // placed row discriminates the force.
        let mut world = stone_world_sized(512, 512);
        tile_runner(
            &mut world,
            &mut Rng(7),
            50,
            40,
            3.0,
            30,
            59,
            0.0,
            &TileRunnerOpts {
                rock_layer: 0.0,
                ..TileRunnerOpts::default()
            },
        );
        let mud: Vec<(i32, i32)> = (0..world.width)
            .flat_map(|x| (0..world.height).map(move |y| (x, y)))
            .filter(|&(x, y)| world.tile(x, y).map_or(false, |t| t.kind == 59))
            .collect();
        assert!(mud.len() >= 8, "the walk placed mud: {:?}", mud.len());
        let min_y = mud.iter().map(|&(_, y)| y).min().unwrap();
        let max_y = mud.iter().map(|&(_, y)| y).max().unwrap();
        assert!(min_y >= 38, "no climb past the first draw: {min_y}");
        assert!(max_y >= 56, "one tile of descent per step: {max_y}");
        // (b) the free band: between rockLayer + 100 and the world bottom the
        // velocity is clamped to ±0.5, so 80 steps move the walker at most
        // ~40 tiles — the guaranteed range of a correct clamp.
        let mut world = stone_world_sized(512, 600);
        tile_runner(
            &mut world,
            &mut Rng(7),
            256,
            250,
            3.0,
            80,
            59,
            0.0,
            &TileRunnerOpts {
                rock_layer: 100.0,
                ..TileRunnerOpts::default()
            },
        );
        let mud: Vec<(i32, i32)> = (0..world.width)
            .flat_map(|x| (0..world.height).map(move |y| (x, y)))
            .filter(|&(x, y)| world.tile(x, y).map_or(false, |t| t.kind == 59))
            .collect();
        assert!(!mud.is_empty(), "the walk placed mud");
        let min_y = mud.iter().map(|&(_, y)| y).min().unwrap();
        let max_y = mud.iter().map(|&(_, y)| y).max().unwrap();
        assert!(min_y >= 208, "half-tile drift down: {min_y}");
        assert!(max_y <= 292, "half-tile drift up: {max_y}");
    }
}
