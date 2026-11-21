//! Liquid placement: the forest lake, the ocean shores and lava puddles, and
//! the generation-time settle machinery (`QuickWater`/`WaterCheck`/the
//! quick-settle loop) the `Settle Liquids` and `Settle Liquids Again` passes
//! share (W4-d106 R2).

use super::TILE_DIRT;
use super::noise::Rng;
use super::terrain::{Layers, SurfaceMap};
use crate::liquid::LiquidSystem;
use crate::tile::{FLAG_ACTIVE, FLAG_LIQUID_HONEY, FLAG_LIQUID_LAVA, LiquidKind, Tile};
use crate::tiledata as td;
use crate::world::World;

/// Depth of the lake floor below the water line: 1 tile at the shore, 6 at the
/// centre. Cosine profile, mirroring the shallow bowls `WorldGen.SonOfLakinater` carves.
pub(crate) fn lake_floor_depth(dx: i32, half: i32) -> i32 {
    let t = dx as f32 / half as f32;
    let c = (t * std::f32::consts::FRAC_PI_2).cos();
    1 + (5.0 * c * c).round() as i32
}

/// Carves the lake bowl into the finished terrain, floods it and the ocean shores,
/// then drips lava onto cave floors. Lowers the lake columns of the surface map to
/// the carved floor so the later surface passes dress the bowl, not the old bank.
pub(crate) fn place_liquids(
    world: &mut World,
    rng: &mut Rng,
    map: &mut SurfaceMap,
    layers: &Layers,
    width: i32,
) {
    // --- liquids ------------------------------------------------------------
    let lake_center = map.lake_center;
    let lake_half = map.lake_half;
    let lake_level = map.lake_level;
    let sea_level = map.sea_level;
    let (left_beach_end, right_beach_start) = (map.left_beach_end, map.right_beach_start);
    let avg_surface = layers.avg_surface;
    let hell_top = layers.hell_top;
    let surface = &mut map.heights;
    // Lake: carve a proper bowl into the finished terrain, then fill it to the water
    // line. `WorldGen.SonOfLakinater` clears the tiles below its start row and only
    // places liquid in rows at/below it; the floor here follows `lake_floor_depth`.
    for dx in -lake_half..=lake_half {
        let x = lake_center + dx;
        if x < 1 || x >= width - 1 {
            continue;
        }
        let floor = lake_level + lake_floor_depth(dx, lake_half);
        let top = surface[x as usize];
        if top < floor {
            // Lower the ground: clear every tile from the old surface to the floor.
            // Above the water line the cut is open air, so the old dirt wall goes too
            // (surface lakes in the original have no wall behind the exposed bank).
            for y in top..floor {
                let i = world.index(x, y);
                let wall = if y < lake_level {
                    0
                } else {
                    world.tiles[i].wall
                };
                world.tiles[i] = Tile {
                    wall,
                    ..Default::default()
                };
            }
        } else if top > floor {
            // Natural ground sits below the water line (beach ramp overlap); raise a
            // dirt floor so the bowl still holds water.
            for y in floor..top {
                let i = world.index(x, y);
                let wall = world.tiles[i].wall;
                world.tiles[i] = Tile {
                    kind: TILE_DIRT,
                    wall,
                    flags: FLAG_ACTIVE,
                    ..Default::default()
                };
            }
        }
        // Bowl floor: dirt, solid, level (slopes are applied later from `surface`).
        let i = world.index(x, floor);
        let wall = world.tiles[i].wall;
        world.tiles[i] = Tile {
            kind: TILE_DIRT,
            wall,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
        surface[x as usize] = floor;
        // Water only in air cells at/below the water line; never above `lake_level`.
        for y in lake_level..floor {
            let i = world.index(x, y);
            if !world.tiles[i].active() {
                world.tiles[i].liquid = 255;
                world.tiles[i].flags &= !(FLAG_LIQUID_LAVA | FLAG_LIQUID_HONEY);
            }
        }
    }
    // Ocean water at both shores — retired in M5-OCEAN phase 2: the
    // faithful `Beaches` pass (`WorldGen.cs:14888`) carves the ocean prism
    // and owns the water; the compact flood would double-place it inside
    // the carved columns.
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
}

// --- the generation settle machinery (W4-d106 R2) ---------------------------
//
// `Settle Liquids` (`WorldGen.cs:16132`) and `Settle Liquids Again` (`:21246`)
// share one body: `QuickWater(3)` drops every liquid cell straight down,
// `WaterCheck` re-wakes the world, then ten quick-settle rounds run
// `Liquid.UpdateLiquid` until the active list drains (each round bounded by
// `numLiquid * 5` updates), `WaterCheck` between rounds, and
// `ClearPendingLiquid` at the tail. The port drives the existing
// [`LiquidSystem`] engine with the gen toggles armed (`generating`,
// `quick_settle`, `gen_ignore_water`, `bubble_solid`).
//
// Scope notes (the `finish_seeds.rs` discipline — seed-gated / unported-helper
// arms are documented stand-ins, not invented):
// - `ShimmerRemoveWater` (`:34823`) and `LiquidInteractionsCleanup` (`:23672`)
//   run inside `QuickWater`; both key on `GenVars.shimmerPosition` / the
//   obsidian/shimmer blocks. The port carries no shimmer until d102's Shimmer
//   pass lands, so [`shimmer_remove_water`] is a `None`-guarded stand-in and
//   [`liquid_interactions_cleanup`] ports the obsidian/shimmer-block sweep.
// - `NotTheBees` (`:25107`) is the notTheBees seed's whole-world honey
//   conversion — flag-gated, ported in [`not_the_bees`] (classic worlds skip).
// - `noTrapsWorldGen`'s `SetBoulderSolidity(false)` window (`:11009`) rides
//   the boulder toggle the engine already carries.

/// The layer anchors the settle re-type reads — `GenVars.waterLine`/`lavaLine`
/// plus `Main.rockLayer`/`worldSurface` and the derived `oceanLevel`.
pub(crate) struct SettleAnchors {
    pub water_line: i32,
    pub lava_line: i32,
    pub rock_layer: f64,
    pub world_surface: f64,
}

impl SettleAnchors {
    /// `WorldGen.oceanLevel` (`WorldGen.cs:4447`).
    pub(crate) fn ocean_level(&self) -> f64 {
        (self.world_surface + self.rock_layer) / 2.0 + 40.0
    }
}

/// `WorldGen.ShimmerRemoveWater` (`WorldGen.cs:34823-34851`): clear non-shimmer
/// liquid and deactivate breakable ice 162 inside the shimmer disc. The port
/// has no shimmer position until d102's Shimmer pass writes it, so this is a
/// `None`-guarded stand-in (the C# static is `(0,0)` → the disc is off-map).
/// Shared with `cleanup::shimmer_clean_up` (W4-d106 R6).
pub(crate) fn shimmer_remove_water(world: &mut World, shimmer_position: Option<(i32, i32)>) {
    let Some((sx, sy)) = shimmer_position else {
        return;
    };
    let num = 150;
    for i in (sy - num)..=(sy + num / 2) {
        for j in (sx - num)..=(sx + num) {
            let dx = (j - sx).abs() as f64;
            let dy = (i - sy).abs() as f64;
            if (dx * dx + dy * dy).sqrt() as i32 >= num {
                continue;
            }
            if let Some(t) = world.tile_mut(j, i) {
                // `:34840` — `.shimmer()` is the type bit (amount-independent).
                if t.liquid_kind() != LiquidKind::Shimmer {
                    t.liquid = 0;
                }
                if t.kind == 162 {
                    t.flags &= !FLAG_ACTIVE;
                }
            }
        }
    }
}

/// `WorldGen.LiquidInteractionsCleanup` (`WorldGen.cs:23672-23799`): every
/// obsidian 56 / shimmer block 659 cell drops its liquid and re-fills to 255
/// with the type its open neighbours carry (or its own default when none do).
fn liquid_interactions_cleanup(world: &mut World) {
    let (width, height) = (world.width, world.height);
    for i in 1..width - 2 {
        for j in 1..height - 2 {
            let kind = world.tile(i, j).map_or(0, |t| if t.active() { t.kind } else { 0 });
            if kind != 56 && kind != 659 {
                continue;
            }
            // `:23683-23696` — the four open neighbours' liquid types.
            let neighbour_type = |x: i32, y: i32| -> i32 {
                world
                    .tile(x, y)
                    .filter(|t| !t.active() && t.liquid > 0)
                    .map_or(-1, |t| kind_code_of(t.liquid_kind()))
            };
            let n = [
                neighbour_type(i - 1, j),
                neighbour_type(i + 1, j),
                neighbour_type(i, j - 1),
                neighbour_type(i, j + 1),
            ];
            let mut counts = [0i32; 4];
            for &t in &n {
                if (0..4).contains(&t) {
                    counts[t as usize] += 1;
                }
            }
            // `:23755` — the C# counts the left shimmer neighbour into water
            // (num5) while the other three shimmer arms count into num8; the
            // net effect on the mixed-type veto is unchanged (any two types
            // present → keep the cell), so the port counts shimmer uniformly.
            // `:23761` — two different types present → leave the cell alone.
            let present = counts.iter().filter(|&&c| c > 0).count();
            if present > 1 {
                continue;
            }
            // `:23765-23796` — pick the type: the neighbours' single type, or
            // the block's own default (obsidian → lava, shimmer → shimmer).
            let mut liquid_type = if kind == 56 { 1u8 } else { 3 };
            if present == 0 {
                // keep the default
            } else if counts[0] > 0 {
                liquid_type = 0;
            } else if counts[1] > 0 {
                liquid_type = 1;
            } else if counts[2] > 0 {
                liquid_type = 2;
            } else if counts[3] > 0 {
                liquid_type = 3;
            }
            if let Some(t) = world.tile_mut(i, j) {
                // `Tile.ClearEverything` then re-fill (`:23794-23796`).
                *t = Tile::default();
                t.liquid = 255;
                t.set_liquid_kind(code_to_kind(liquid_type));
            }
        }
    }
}

/// `Liquid.QuickWater` (`Liquid.cs:104-148`) — the generation settle sweep:
/// arm the bubble + the grate/boulder/gen-water toggles, then bottom-up
/// (y from `maxTilesY-3` to 3) and x from 4 to `maxTilesX-4`, drop every
/// liquid cell via `SettleWaterAt`. `verbose`/`minY`/`maxY` ride the C#
/// defaults (the settle passes call `QuickWater(3)` / `QuickWater(-2)`; the
/// verbose arg only drives progress text, no behaviour). W4-d106 R2.
/// Shared with `cleanup::finish_remix_world` (W4-d106 R6's `QuickWater(-2)`
/// calls — the verbose arg is progress-only, the bounds are the defaults).
pub(crate) fn quick_water(
    world: &mut World,
    sys: &mut LiquidSystem,
    anchors: &SettleAnchors,
    shimmer_position: Option<(i32, i32)>,
    no_traps: bool,
) {
    // `:106-113` — the generation-only shimmer strip + noTraps boulder window.
    shimmer_remove_water(world, shimmer_position);
    // `SetBoulderSolidity(false/true)` (`:11011-11019`) for noTraps: the
    // port's engine ties the boulder family to `ignore_solids`, which
    // `settle_water_at` arms per cell anyway; the classic world carries no
    // boulders (the FTW finish converts them), so the window is a documented
    // no-op stand-in.
    let _ = no_traps;
    // `:114` — the bubble is solid for the sweep. (`:115`/`:135`'s
    // `tilesIgnoreWater` wrap is redundant with `settle_water_at`'s own
    // per-cell arm/restore, `Liquid.cs:153`/`:288`.) The pass owns the
    // `worldGenTilesIgnoreWater` toggle (`:21263`/`:21306`) — not this sweep.
    sys.arm_bubble_solid();
    let (width, height) = (world.width, world.height);
    let (min_y, max_y) = (3, height - 3);
    // `:124-134` — bottom-up rows, x 4..width-4.
    let mut num = max_y;
    while num >= min_y {
        for i in 4..width - 4 {
            if world.tile(i, num).is_some_and(|t| t.liquid != 0) {
                sys.settle_water_at(
                    world,
                    i,
                    num,
                    anchors.water_line,
                    anchors.lava_line,
                    anchors.rock_layer,
                    anchors.ocean_level(),
                );
            }
        }
        num -= 1;
    }
    // `:136-147` — shimmer strip again, and the obsidian/shimmer-block
    // cleanup (generation, non-skyblock — the caller gates skyblock).
    shimmer_remove_water(world, shimmer_position);
    liquid_interactions_cleanup(world);
}

/// The shared `Settle Liquids` / `Settle Liquids Again` body
/// (`WorldGen.cs:16156-16208` / `:21263-21307`): `QuickWater(3)`, `WaterCheck`,
/// ten quick-settle rounds, `ClearPendingLiquid`, toggles restored. The
/// `extraLiquid` budget bump (`:16173-16176`) is a secret-seed arm the port's
/// classic path skips (documented). W4-d106 R2.
fn settle_liquids_body(
    world: &mut World,
    anchors: &SettleAnchors,
    shimmer_position: Option<(i32, i32)>,
    remix: bool,
    no_traps: bool,
) {
    let mut sys = LiquidSystem::new(world);
    sys.set_generating(true);
    sys.set_remix_world_gen(remix);
    // `Liquid.worldGenTilesIgnoreWater(ignoreSolids: true)` (`:16156`/`:21263`).
    sys.set_gen_ignore_water(true);
    quick_water(world, &mut sys, anchors, shimmer_position, no_traps);
    sys.water_check(world);
    // `:16159-16204` / `:21266-21303` — the ten-round quick-settle loop.
    sys.set_quick_settle(true);
    let rounds = 10;
    let mut num = 0;
    while num < rounds {
        let mut num3 = sys.num_liquid() + sys.buffered_len();
        num += 1;
        let mut budget = num3 * 5;
        while sys.num_liquid() > 0 {
            budget -= 1;
            if budget < 0 {
                break;
            }
            let now = sys.num_liquid() + sys.buffered_len();
            if now > num3 {
                num3 = now;
            }
            sys.update(world);
        }
        sys.water_check(world);
    }
    // `:16205-16208` / `:21304-21307` — clear pending, restore the toggles.
    sys.clear_pending_liquid();
    sys.set_quick_settle(false);
    sys.set_gen_ignore_water(false);
}

/// `WorldGen.NotTheBees` (`WorldGen.cs:25107-25360`) — the notTheBees seed's
/// whole-world honey conversion. Flag-gated: a classic world never calls it.
/// The port carries the seed flag but the full conversion (the dungeon-side
/// half, the remix bands, the wall-86 honey sweep) is a large seed-only body;
/// this lane ports the gate + the skyblock/no-op guards and defers the
/// conversion arms to the notTheBees lane (documented stand-in — the classic
/// pipeline is untouched, the affected-seed witness rides a==b). Shared with
/// `cleanup::final_cleanup`'s seed dispatch (W4-d106 R6).
pub(crate) fn not_the_bees(world: &mut World, opts: &super::GenOptions) {
    if !opts.flags.contains(crate::seeds::SeedFlags::NOT_THE_BEES) {
        return;
    }
    if opts.flags.contains(crate::seeds::SeedFlags::SKYBLOCK) {
        return;
    }
    // The conversion body (`:25118-25359`) is seed-only; deferred to the
    // notTheBees lane. No classic-world writes.
    let _ = world;
}

/// `Settle Liquids Again` (`GenPassNameID.SettleLiquidsPart2AndNotTheBees`,
/// `WorldGen.cs:21246-21309`) — W4-d106 R2. The second settle pass, after the
/// Micro Biomes family; it re-runs the shared settle body so the liquids the
/// late passes disturbed (the thin ice, the campsites, the mahogany trees)
/// drain to rest. `Skyblock.denyAllGeneration` skips it; the FTW arm sets
/// `tileSolid[56] = true` (obsidian solid for the settle). The pass draws no
/// RNG of its own (the settle engine is deterministic over the tile state).
pub(crate) fn settle_liquids_again(
    world: &mut World,
    gv: &super::pipeline::GenVars,
    opts: &super::GenOptions,
) {
    // `:21248` — `!Skyblock.denyAllGeneration`.
    if opts.flags.contains(crate::seeds::SeedFlags::SKYBLOCK) {
        return;
    }
    // `:21250-21253` — the FTW obsidian-solid window (the engine's solidity
    // reads the stock table; obsidian 56 is solid by default, so the arm is a
    // no-op on the port's table — documented).
    // `:21259-21262` — the notTheBees conversion (flag-gated).
    not_the_bees(world, opts);
    let anchors = SettleAnchors {
        water_line: gv.water_line,
        lava_line: gv.lava_line,
        rock_layer: world.rock_layer,
        world_surface: world.world_surface,
    };
    settle_liquids_body(
        world,
        &anchors,
        gv.shimmer_position,
        opts.flags.contains(crate::seeds::SeedFlags::REMIX),
        opts.flags.contains(crate::seeds::SeedFlags::NO_TRAPS),
    );
}

/// `LiquidKind` → the C# `liquidType()` code (0 water / 1 lava / 2 honey /
/// 3 shimmer).
fn kind_code_of(k: LiquidKind) -> i32 {
    match k {
        LiquidKind::Water => 0,
        LiquidKind::Lava => 1,
        LiquidKind::Honey => 2,
        LiquidKind::Shimmer => 3,
    }
}

/// The C# `liquidType()` code → `LiquidKind`.
fn code_to_kind(code: u8) -> LiquidKind {
    match code {
        1 => LiquidKind::Lava,
        2 => LiquidKind::Honey,
        3 => LiquidKind::Shimmer,
        _ => LiquidKind::Water,
    }
}

#[cfg(test)]
mod tests {
    use super::lake_floor_depth;
    use crate::tiledata as td;
    use crate::world::World;
    use crate::worldgen::{GenOptions, TILE_DIRT, TILE_GRASS, generate};

    /// `lake_center`, `lake_half` and the water line (first row with liquid).
    fn lake_params(w: &World) -> (i32, i32, i32) {
        let center = (w.width / 2 + 150).min(w.width - 80);
        let level = (0..w.height)
            .find(|&y| w.tile(center, y).unwrap().liquid > 0)
            .expect("lake has no water at its centre");
        (center, 32, level)
    }

    /// The ground row below any sky objects (the floating islands' clouds
    /// hang over the terrain — the first solid from the sky is not the
    /// ground anymore; walk down to the first dirt tile).
    fn ground_row(w: &crate::world::World, x: i32) -> i32 {
        let mut y = w.surface_of(x);
        while y < w.height - 2 && w.tile(x, y).unwrap().kind != TILE_DIRT {
            y += 1;
        }
        y
    }

    fn lake_water_stays_at_or_below_the_water_line() {
        let mut w = generate(&GenOptions::default());
        w.compute_column_surface();
        let (center, half, level) = lake_params(&w);
        // The rim must stand above the water line so the lake is not a perched pool.
        let rim_low = [center - half - 1, center + half + 1]
            .iter()
            .map(|&x| w.surface_of(x))
            .min()
            .unwrap();
        assert!(rim_low <= level, "lake rim is below the water line");

        for dx in -(half + 3)..=(half + 3) {
            let x = center + dx;
            // Nothing above the water line, including the dry banks.
            for y in 0..level {
                assert_eq!(
                    w.tile(x, y).unwrap().liquid,
                    0,
                    "liquid above the lake surface at {x},{y}"
                );
            }
            // Inside the bowl: water only in air over the solid floor.
            for y in level..level + 8 {
                let t = w.tile(x, y).unwrap();
                if t.liquid > 0 {
                    assert!(!t.active(), "liquid inside a solid tile at {x},{y}");
                    assert!(
                        y < w.surface_of(x),
                        "liquid floats above the ground surface at {x},{y}"
                    );
                }
            }
        }
    }

    #[test]
    fn lake_keeps_grass_on_the_dry_banks() {
        let mut w = generate(&GenOptions::default());
        w.compute_column_surface();
        let (center, half, level) = lake_params(&w);
        let mut grass_banks = 0;
        for dx in [-(half + 1), half + 1] {
            let x = center + dx;
            // The bank's top: the first non-cloud tile from the surface (the
            // floating islands hang over the terrain).
            let mut top = w.surface_of(x);
            while top < w.height - 2
                && matches!(w.tile(x, top).unwrap().kind, 189 | 196)
            {
                top += 1;
            }
            let t = *w.tile(x, top).unwrap();
            assert!(t.active(), "bank column x={x} has no ground");
            if t.kind == TILE_GRASS {
                assert!(
                    top <= level,
                    "grass bank at x={x} sits below the water line (top {top}, level {level})"
                );
                assert!(
                    (0..w.height).all(|y| w.tile(x, y).unwrap().liquid == 0),
                    "dry bank at x={x} contains liquid"
                );
                grass_banks += 1;
            }
        }
        assert!(grass_banks >= 1, "no grass bank left in the old lake span");
    }

    #[test]
    fn lake_bowl_floor_is_solid_below_the_surface() {
        let mut w = generate(&GenOptions::default());
        w.compute_column_surface();
        let (center, half, level) = lake_params(&w);
        let mut deepest = 0;
        for dx in -half..=half {
            let x = center + dx;
            let top = ground_row(&w, x);
            let t = *w.tile(x, top).unwrap();
            assert!(
                t.active() && td::TILE_SOLID.has(t.kind),
                "bowl floor is not solid at {x},{top}"
            );
            assert_eq!(t.kind, TILE_DIRT, "bowl floor is not dirt at {x},{top}");
            let depth = top - level;
            assert_eq!(
                depth,
                lake_floor_depth(dx, half),
                "unexpected bowl depth at x={x}"
            );
            assert!(
                depth >= 1,
                "bowl floor is not below the water line at x={x}"
            );
            deepest = deepest.max(depth);
            assert!(
                w.tile(x, top + 1).unwrap().active(),
                "bowl floor is only one tile thin at x={x}"
            );
            assert!(
                w.tile(x, level).unwrap().liquid > 0,
                "shore is dry at the water line at x={x}"
            );
        }
        assert_eq!(deepest, 6, "bowl should be 6 deep at its centre");
        assert_eq!(lake_floor_depth(0, half), 6);
        assert_eq!(lake_floor_depth(half, half), 1);
        assert_eq!(lake_floor_depth(-half, half), 1);
    }

    // --- W4-d106 R2 witnesses (written NOT run — code-only branch) ----------

    use super::{SettleAnchors, settle_liquids_again, settle_liquids_body};
    use crate::tile::{FLAG_ACTIVE, LiquidKind, Tile};
    use crate::worldgen::pipeline::GenVars;

    /// A flat stone-floor world with a water column perched above a gap:
    /// the settle sweep must drop it onto the floor and level it.
    fn perched_world() -> World {
        let mut w = World::new("settle".into(), 120, 120);
        // A stone floor at y=100 across the middle.
        for x in 10..110 {
            let i = w.index(x, 100);
            w.tiles[i] = Tile { kind: 1, flags: FLAG_ACTIVE, ..Default::default() };
        }
        // A 255-water cell perched at (60, 40), air below down to the floor.
        let i = w.index(60, 40);
        w.tiles[i].liquid = 255;
        w
    }

    fn anchors() -> SettleAnchors {
        SettleAnchors { water_line: 60, lava_line: 90, rock_layer: 70.0, world_surface: 30.0 }
    }

    /// `QuickWater` drops the perched cell onto the floor (the settle walk,
    /// `Liquid.cs:150-289`) — no liquid stays at the perch row.
    #[test]
    fn r2_quick_water_drops_the_perched_cell() {
        let mut w = perched_world();
        settle_liquids_body(&mut w, &anchors(), None, false, false);
        assert_eq!(w.tile(60, 40).unwrap().liquid, 0, "the perch drained");
        // The water landed on the floor row above the stone (y=99).
        let floor_water: i32 = (10..110).map(|x| w.tile(x, 99).unwrap().liquid as i32).sum();
        assert!(floor_water > 0, "the water settled onto the floor ({floor_water})");
    }

    /// The settle body is deterministic (a == b on the same tile state) and
    /// conserves the total liquid amount (no cell created or destroyed).
    #[test]
    fn r2_the_settle_is_deterministic_and_conserves_liquid() {
        let total = |w: &World| -> i64 {
            w.tiles.iter().map(|t| t.liquid as i64).sum()
        };
        let mut a = perched_world();
        let mut b = perched_world();
        let before = total(&a);
        settle_liquids_body(&mut a, &anchors(), None, false, false);
        settle_liquids_body(&mut b, &anchors(), None, false, false);
        assert_eq!(a.tiles, b.tiles, "a == b");
        assert_eq!(total(&a), before, "the settle conserves liquid");
    }

    /// The pass's skyblock gate (`WorldGen.cs:21248`) skips it tile-for-tile.
    #[test]
    fn r2_skyblock_skips_the_pass() {
        let mut w = perched_world();
        let before = w.tiles.clone();
        let gv = GenVars { water_line: 60, lava_line: 90, ..Default::default() };
        let opts = GenOptions {
            flags: crate::seeds::SeedFlags::from(crate::seeds::SeedFlags::SKYBLOCK),
            ..Default::default()
        };
        settle_liquids_again(&mut w, &gv, &opts);
        assert_eq!(w.tiles, before, "skyblock skips the settle");
    }

    /// A lava cell below `waterLine` re-types to lava as it falls
    /// (`Liquid.cs:184-187`); a water cell above stays water.
    #[test]
    fn r2_the_settle_retypes_by_depth() {
        let mut w = World::new("settle".into(), 120, 120);
        for x in 10..110 {
            let i = w.index(x, 110);
            w.tiles[i] = Tile { kind: 1, flags: FLAG_ACTIVE, ..Default::default() };
        }
        // Water perched high (y=20) falls past waterLine=60 → becomes lava.
        let i = w.index(60, 20);
        w.tiles[i].liquid = 255;
        settle_liquids_body(&mut w, &anchors(), None, false, false);
        let landed = w.tile(60, 109).unwrap();
        assert!(landed.liquid > 0, "the cell landed");
        assert_eq!(landed.liquid_kind(), LiquidKind::Lava, "below waterLine → lava");
    }
}
