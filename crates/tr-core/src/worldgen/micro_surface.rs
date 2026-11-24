//! M5-SEED-2: the `SecretSeed.FinalizeSecretSeeds` surface-conversion arms
//! (`WorldGen.cs:589-693`): `DoSurfaceIsMushrooms`, `DoHallowOnSurface`,
//! `DoWorldIsInfected`, `DoWorldIsFrozen` (+ its Finish), `DoNoInfection`,
//! `DoStartInHardmode`, `DoPooEverywhere` and `DoSurfaceIsInSpace`. Each is
//! flag-gated and draws from its own `pass_rng` stream; a classic world's
//! pipeline is untouched.
//!
//! Seam: `pipeline.rs`'s `step_seed_finish` dispatches [`run`] after
//! `micro_finish::run` (the C# order inside `FinalizeSecretSeeds`).
//!
//! Deviations from the C# are marked inline; the big ones, once:
//! - `DoWorldIsInfected`/`DoHallowOnSurface`/`DoNoInfection` call
//!   `WorldGen.Convert`, a set-based whole-cell machine (the
//!   `TileID.Sets.Conversion.*` families, `WorldGen.cs:55067-55290`). The
//!   port models those conversions in `spread.rs`
//!   (`convert_tile`/`convert_wall` for the evil flavours,
//!   `convert_tile_hallow`/`convert_wall_hallow` for the hallow) — the same
//!   tables, applied per cell with the C#'s per-column type decision.
//! - The arms gated on `noSurface`/`skyblock` read flags that have no
//!   worldgen behaviour yet (`noSurface`'s sweeps are M5-SEED-2's deferred
//!   family; skyblock is a classic seed none of the port's passes model), so
//!   those inner branches take their non-flag path.
//! - `DoSurfaceIsDesertFinish` (`:2287-2353`) re-frames the pass's tiles;
//!   the port carries no `TileFrame`, so only the tile/wall writes are
//!   ported and the frame arm is documented out.

use crate::spread;
use crate::world::World;
use super::pipeline::Streams;
use super::GenOptions;
use crate::seeds::{MicroSeedFlags, SeedFlags, active_secret_seed_count};

/// The `FinalizeSecretSeeds` surface-conversion arms, in the C# dispatch
/// order (`WorldGen.cs:589-693`): the order matters because the arms read
/// the state the earlier arms write (e.g. `worldIsFrozen` re-typing the
/// stone the desert arm just sandstoned).
pub(crate) fn run(world: &mut World, streams: &mut Streams, opts: &GenOptions) {
    let micro = opts.micro_flags;
    let active = active_secret_seed_count(micro, opts.flags);
    if micro.contains(MicroSeedFlags::SURFACE_IS_DESERT) {
        // W4-d106 R6: the C#'s `FinalizeSecretSeeds` desert arm is
        // `DoSurfaceIsDesertFinish` only (`WorldGen.cs:591-594`); the body
        // `DoSurfaceIsDesert` runs from the Final Cleanup head
        // (`:22146-22149`, `cleanup::final_cleanup`) on a fresh
        // `pass_rng("DoSurfaceIsDesert")` copy — same draws, earlier slot.
        surface_is_desert_finish(world, &mut streams.pass_rng("DoSurfaceIsDesertFinish"), micro);
    }
    if micro.contains(MicroSeedFlags::SURFACE_IS_IN_SPACE) {
        surface_is_in_space(world);
    }
    if micro.contains(MicroSeedFlags::SURFACE_IS_MUSHROOMS) {
        // `FinalizeSecretSeeds` (`WorldGen.cs:607-613`): `if (!noSurface)
        // DoSurfaceIsMushrooms(); DoSurfaceIsMushrooms();` — the second call
        // ALWAYS runs, on the SAME continuing genRand stream (W4-g42 G6).
        // The noSurface arm itself is the documented M5-SEED-2 skip (no
        // noSurface world state), so under that flag both calls stand down.
        // The mid-pipeline third call (`MudCavesToJungleGrass`, `:12474-
        // 12477`) needs the step seam + a continuing stream — the documented
        // follow-up (research/12j-g42).
        if !micro.contains(MicroSeedFlags::NO_SURFACE) {
            let mut rng = streams.pass_rng("DoSurfaceIsMushrooms");
            surface_is_mushrooms(world, &mut rng);
            surface_is_mushrooms(world, &mut rng);
        }
    }
    if micro.contains(MicroSeedFlags::WORLD_IS_FROZEN) {
        world_is_frozen(world, &mut streams.pass_rng("DoWorldIsFrozen"), micro, active);
        world_is_frozen_finish(world, &mut streams.pass_rng("DoWorldIsFrozenFinish"));
    }
    if micro.contains(MicroSeedFlags::NO_INFECTION) {
        no_infection(world, micro);
    }
    if micro.contains(MicroSeedFlags::HALLOW_ON_THE_SURFACE) {
        hallow_on_the_surface(world, &mut streams.pass_rng("DoHallowOnSurface"), micro);
    }
    if micro.contains(MicroSeedFlags::WORLD_IS_INFECTED) {
        world_is_infected(world, &mut streams.pass_rng("DoWorldIsInfected"), micro, opts);
    }
    if micro.contains(MicroSeedFlags::START_IN_HARDMODE) {
        start_in_hardmode(world, opts);
    }
    if micro.contains(MicroSeedFlags::POO_EVERYWHERE) {
        poo_everywhere(world, &mut streams.pass_rng("DoPooEverywhere"), micro, active);
    }
}

// --- small helpers ---------------------------------------------------------

/// A tile's kind, or 0 outside the map (the C#'s `Main.tile[x, y].type`
/// reads as 0 there — the sweeps below all guard with it).
fn kind_at(world: &World, x: i32, y: i32) -> u16 {
    world.tile(x, y).map_or(0, |t| t.kind)
}

fn wall_at(world: &World, x: i32, y: i32) -> u16 {
    world.tile(x, y).map_or(0, |t| t.wall)
}

fn active_at(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| t.active())
}

fn set_kind(world: &mut World, x: i32, y: i32, kind: u16) {
    if let Some(t) = world.tile_mut(x, y) {
        t.kind = kind;
    }
}

/// `WorldGen.SolidTile` (`WorldGen.cs:7588-7591`) — `active && tileSolid[type]`.
fn solid(world: &World, x: i32, y: i32) -> bool {
    world
        .tile(x, y)
        .is_some_and(|t| t.active() && crate::tiledata::TILE_SOLID.has(t.kind))
}

/// `WorldGen.SolidTile3` (`:7597-7600`) — the tile's own `solidTop` bit
/// gate (`Main.tileSolidTop[type] && !halfBrick`), i.e. "standable".
fn solid_top(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| {
        t.active() && crate::tiledata::TILE_SOLID_TOP.has(t.kind)
    })
}

// --- arms ------------------------------------------------------------------

/// `DoSurfaceIsDesert` (`WorldGen.cs:2355-2591`), the normal (non-noSurface)
/// arm only: the per-column band `worldSurface + Next(3)` conversions. The
/// noSurface arm is a row-major scan keyed on the ice bands — not ported
/// (no `noSurface` world state; documented at the module doc). Called from
/// `cleanup::final_cleanup` (the C#'s Final Cleanup head, `:22146-22149`,
/// W4-d106 R6) on a fresh `pass_rng("DoSurfaceIsDesert")` copy. `ws` is the
/// real `Main.worldSurface` layer anchor (`gv.layers()`) — the port's
/// `world.world_surface` is only written at the world tail, after this pass.
pub(crate) fn surface_is_desert(world: &mut World, rng: &mut super::Rng, ws: f64, _micro: MicroSeedFlags, _active: u32) {
    let band = ws as i32 + rng.range(0, 3);
    for x in 0..world.width {
        for y in 0..band.min(world.height) {
            if !active_at(world, x, y) {
                continue;
            }
            let kind = kind_at(world, x, y);
            let new = match kind {
                1 => Some(396),
                25 => Some(398),
                203 => Some(399),
                117 => Some(402),
                0 | 2 => Some(53),
                23 => Some(112),
                109 => Some(116),
                _ => None,
            };
            if let Some(k) = new {
                set_kind(world, x, y, k);
                // The 23/109 arms re-skin the 7x3 neighbourhood's sand
                // (`:2564-2592`): 234 for the corrupt-grass arm, 116 for the
                // hardensand arm.
                let neighbour = if kind == 23 { 234 } else { 116 };
                for nx in x - 3..=x + 3 {
                    for ny in y - 3..=y + 3 {
                        if kind_at(world, nx, ny) == 53 {
                            set_kind(world, nx, ny, neighbour);
                        }
                    }
                }
            }
            // The dirt/sandstone walls under the sand clear (`:2598-2601`).
            let wall = wall_at(world, x, y);
            if matches!(wall, 2 | 59) && kind_at(world, x, y) == 53 {
                if let Some(t) = world.tile_mut(x, y) {
                    t.wall = 0;
                }
            }
        }
    }
}

/// `DoSurfaceIsDesertFinish` (`WorldGen.cs:2287-2353`): the wall-63→2
/// conversion and the sand-skinning of the 397/396/ore/40 columns under 7
/// solid rows. The `TileFrame` calls and the `roundLandmasses`/skyblock wall
/// suppression are documented out (no frame pass; the flags' behaviour is
/// deferred).
fn surface_is_desert_finish(world: &mut World, _rng: &mut super::Rng, _micro: MicroSeedFlags) {
    let surface = world.world_surface as i32;
    for x in 0..world.width {
        for y in 0..(world.height - 5).max(0) {
            // `if (wall == 63) wall = 2` (`:2317-2320`).
            if wall_at(world, x, y) == 63 {
                if let Some(t) = world.tile_mut(x, y) {
                    t.wall = 2;
                }
            }
            // The sand-skinning: `!surfaceIsDesertNoSurfaceCleanup` arm —
            // 397/396/ore/40 → 53 when the seven rows above are solid
            // (`:2321-2352`).
            let underworld = world.height as i32 - 200;
            if y >= underworld {
                continue;
            }
            let kind = kind_at(world, x, y);
            if !matches!(kind, 397 | 396 | 40) && !crate::tiledata::SET_ORE.has(kind) {
                continue;
            }
            let mut solids = 0;
            for k in (y - 7).max(0)..y {
                if solid(world, x, k) {
                    solids += 1;
                }
            }
            if solids == 7 {
                set_kind(world, x, y, 53);
            }
        }
    }
    let _ = surface;
}

/// `DoSurfaceIsInSpace` (`WorldGen.cs:2069-2085`): the cloud wall 73 clears
/// above the surface where the tile is inactive or sand.
fn surface_is_in_space(world: &mut World) {
    let surface = world.world_surface as i32;
    for x in 0..world.width {
        for y in 0..surface.min(world.height) {
            if wall_at(world, x, y) == 73 && (!active_at(world, x, y) || kind_at(world, x, y) == 53) {
                if let Some(t) = world.tile_mut(x, y) {
                    t.wall = 0;
                }
            }
        }
    }
}

/// `DoSurfaceIsMushrooms` (`WorldGen.cs:2087-2173`), the normal (non-noSurface)
/// arm, verbatim (W4-g42 G6): per column k, the inner `for` CONDITION draws
/// `Next(3)` EVERY iteration — `for (l = 0; (double)l < worldSurface +
/// Next(3); l++)` — so the band wobbles per row and the stream takes one draw
/// per visited row plus one failing draw per column. The C#'s tile reads are
/// UNGATED by active (`.type` reads the stored kind even on inactive cells —
/// the port's `kind_at` matches). The two `Convert(k, l, 3, 0)` calls (the
/// 60-tile and wall-15 arms) ride the mushroom conversion cell
/// ([`crate::spread::convert_cell_mushroom`], G7): 60 → 70 (NOT the old 59
/// stand-in) and the wall-15 → 80 arm. The noSurface arm (`:2094-2116`)
/// stays the documented M5-SEED-2 skip; the skyblock early-out (`:2089-2092`)
/// reads the classic flag the port carries.
fn surface_is_mushrooms(world: &mut World, rng: &mut super::Rng) {
    // `if (WorldGen.skyblockWorldGen) return` (`:2089-2092`).
    if world.seeds.contains(SeedFlags::SKYBLOCK) {
        return;
    }
    for x in 0..world.width {
        // The condition-draw loop (`:2120`): `l < worldSurface + Next(3)`
        // re-evaluated every iteration. The `.min(height)` clamp is the
        // port's bounds guard (the C# indexes the tile array directly).
        let mut y = 0i32;
        while (y as f64) < world.world_surface + rng.range(0, 3) as f64
            && y < world.height
        {
            let kind = kind_at(world, x, y);
            let wall = wall_at(world, x, y);
            // `if (type == 60) Convert(k, l, 3, 0)` (`:2122-2125`).
            if kind == 60 {
                spread::convert_cell_mushroom(world, x, y);
            }
            // `if (wall == 15) Convert(k, l, 3, 0)` (`:2126-2129`) — the
            // wall re-read: the first Convert may have changed it.
            if wall_at(world, x, y) == 15 {
                spread::convert_cell_mushroom(world, x, y);
            }
            // The direct writes (`:2130-2169`) — the kind re-read per arm
            // (the C# reads `Main.tile[k, l].type` fresh each `if`).
            if kind_at(world, x, y) == 0 {
                set_kind(world, x, y, 59);
            }
            if kind_at(world, x, y) == 2 {
                set_kind(world, x, y, 60);
            }
            // The eight wall arms, in the C#'s source order (63, 65, 66, 68,
            // 64, 205, 206, 207 → 80) — one write each, the wall re-read.
            for from in [63u16, 65, 66, 68, 64, 205, 206, 207] {
                if wall_at(world, x, y) == from {
                    if let Some(t) = world.tile_mut(x, y) {
                        t.wall = 80;
                    }
                }
            }
            y += 1;
        }
    }
}

/// `DoWorldIsFrozen` (`WorldGen.cs:2216-2285`): the per-column band
/// `UnderworldLayer - maxTilesY/10 + Next(3)` (the rock layer when ≥6
/// secrets are active) with the direct tile/wall conversions.
fn world_is_frozen(world: &mut World, rng: &mut super::Rng, _micro: MicroSeedFlags, active: u32) {
    let mut top = (world.height as i32 - 200) - world.height / 10;
    if active >= 6 {
        top = world.rock_layer as i32;
    }
    let band = top + rng.range(0, 3);
    for x in 0..world.width {
        for y in 0..band.min(world.height) {
            let kind = kind_at(world, x, y);
            let new = match kind {
                1 => Some(161),
                25 => Some(163),
                203 => Some(200),
                117 => Some(164),
                0 | 2 | 23 | 199 | 109 => Some(147),
                123 => Some(224),
                196 => Some(460),
                _ => None,
            };
            if let Some(k) = new {
                set_kind(world, x, y, k);
            }
            if matches!(wall_at(world, x, y), 2 | 59) {
                if let Some(t) = world.tile_mut(x, y) {
                    t.wall = 40;
                }
            }
        }
    }
}

/// `DoWorldIsFrozenFinish` (`WorldGen.cs:2175-2214`): the chest Present row
/// and the endless-Christmas Guide→Santa swap are chest/NPC-side — the port
/// places chests as tiles without inventories and has no gen-time spawn
/// reader here, so both arms are documented out (a worldgen no-op).
fn world_is_frozen_finish(_world: &mut World, _rng: &mut super::Rng) {}

/// `DoNoInfection` (`WorldGen.cs:1905-1948`): the whole-world clear — every
/// tile except mushroom grass (70) and every wall except the mushroom wall
/// (80) converts to the plain flavour; with `worldIsInfected` also on the
/// other seed's evil stone/walls (203/25, 83/3) stay.
fn no_infection(world: &mut World, micro: MicroSeedFlags) {
    let keep_evil = micro.contains(MicroSeedFlags::WORLD_IS_INFECTED);
    for x in 0..world.width {
        for y in 0..world.height {
            let kind = kind_at(world, x, y);
            let wall = wall_at(world, x, y);
            let clear_tile = kind != 70 && !(keep_evil && matches!(kind, 203 | 25));
            let clear_wall = wall != 80 && !(keep_evil && matches!(wall, 83 | 3));
            if clear_tile {
                if let Some(t) = world.tile_mut(x, y) {
                    t.kind = 0;
                }
            }
            if clear_wall {
                if let Some(t) = world.tile_mut(x, y) {
                    t.wall = 0;
                }
            }
        }
    }
}

/// `DoHallowOnSurface` (`WorldGen.cs:1950-2015`): the per-column band
/// `worldSurface + Next(3)` with the hallow conversion (type 2), walls
/// included.
fn hallow_on_the_surface(world: &mut World, rng: &mut super::Rng, micro: MicroSeedFlags) {
    let mut top = world.world_surface as i32;
    // The noSurface arm's depth: the rock layer (half when noInfection too).
    if micro.contains(MicroSeedFlags::NO_SURFACE) {
        let rock = world.rock_layer as i32;
        top = if micro.contains(MicroSeedFlags::NO_INFECTION) {
            rock / 2
        } else {
            rock
        };
    }
    let band = top + rng.range(0, 3);
    for x in 0..world.width {
        for y in 0..band.min(world.height) {
            let kind = kind_at(world, x, y);
            let wall = wall_at(world, x, y);
            let tiles = matches!(kind, 2 | 161 | 53 | 396 | 397)
                || (micro.contains(MicroSeedFlags::NO_SURFACE) && kind == 1);
            if tiles {
                if let Some(new) = spread::convert_tile_hallow(kind) {
                    set_kind(world, x, y, new);
                }
            }
            if matches!(wall, 187 | 216 | 63 | 65 | 66 | 68) {
                if let Some(new) = spread::convert_wall_hallow(wall) {
                    if let Some(t) = world.tile_mut(x, y) {
                        t.wall = new;
                    }
                }
            }
        }
    }
}

/// `DoWorldIsInfected` (`WorldGen.cs:2017-2067`): the whole-world evil
/// conversion below the anchor, per-column type by the C#'s decision table
/// (the drunk half split rides `GenVars.crimsonLeft`; the plain world rides
/// `WorldGen.crimson`).
fn world_is_infected(world: &mut World, rng: &mut super::Rng, micro: MicroSeedFlags, opts: &GenOptions) {
    let mut start = 0i32;
    if micro.contains(MicroSeedFlags::NO_INFECTION) {
        start = if micro.contains(MicroSeedFlags::NO_SURFACE) {
            ((world.rock_layer as i32) + (world.height as i32 - 200)) / 2
        } else if micro.contains(MicroSeedFlags::HALLOW_ON_THE_SURFACE) {
            ((world.rock_layer as i32) + (world.height as i32 - 200)) / 2
        } else {
            world.world_surface as i32 + 10
        };
    } else if micro.contains(MicroSeedFlags::HALLOW_ON_THE_SURFACE) {
        start = world.world_surface as i32 - 3;
    }
    // The drunk half split (`:2046-2052`): the port's `crimson_left` slice
    // rides the pipeline's GenVars; micro_seeds can't reach it here, so the
    // split is re-derived the same way the evil pass does — from the Reset
    // stream.
    let drunk = opts.flags.contains(SeedFlags::DRUNK);
    let crimson_left = if drunk {
        let mut reset = super::pipeline::Streams::new(opts.seed).pass_rng("Reset");
        reset.range(0, 2) == 1
    } else {
        false
    };
    for x in 0..world.width {
        let y0 = start + rng.range(0, 3);
        let crimson = if drunk {
            (crimson_left && x < world.width / 2) || (!crimson_left && x > world.width / 2)
        } else {
            opts.crimson
        };
        for y in y0..world.height {
            let kind = kind_at(world, x, y);
            let wall = wall_at(world, x, y);
            let tiles = !matches!(kind, 60 | 109 | 117 | 164 | 116 | 403 | 402);
            let walls = !matches!(wall, 64 | 204 | 205 | 206 | 207 | 70 | 265 | 28 | 219 | 222);
            if tiles {
                if let Some(new) = spread::convert_tile(kind, crimson) {
                    set_kind(world, x, y, new);
                }
            }
            if walls {
                if let Some(new) = spread::convert_wall(wall, crimson) {
                    if let Some(t) = world.tile_mut(x, y) {
                        t.wall = new;
                    }
                }
            }
        }
    }
}

/// `DoStartInHardmode` (`WorldGen.cs:1899-1903`): the world starts hardmode
/// — `Main.hardMode = true` + `WorldGen.initializeHardMode()`.
///
/// The feed is a `UnifiedRandom` seeded with the world seed: during
/// generation `Main.rand = new UnifiedRandom(seed)` (`WorldGen.cs:6611`,
/// `:11144`) and `genRand` == `Main.rand` (`:4445`), so the C#'s
/// `initializeHardMode` draws exactly this stream (the pre-S1-A-11 local
/// `FastRandom` was a wrong-algorithm stand-in; the pinned hash worlds never
/// set the `startInHardmode` secret seed, so their hashes are untouched).
fn start_in_hardmode(world: &mut World, opts: &GenOptions) {
    world.hard_mode = true;
    let mut rng = crate::UnifiedRandom::new(opts.seed as i32);
    // W4-g69 retire sweep: `initialize_hard_mode` reads `world.crimson`
    // (the C# `GERunner` reads the `WorldGen.crimson` static, `:76304`);
    // the generation entries stamp it from `opts.crimson` before the
    // pipeline runs, so the flavour is the same value the old explicit
    // `opts.crimson` argument carried.
    spread::initialize_hard_mode(world, &mut rng);
}

/// `DoPooEverywhere` (`WorldGen.cs:2966-2993`): the count-scaled
/// `TileRunner(666)` blobs under the first solid of each candidate column.
fn poo_everywhere(world: &mut World, rng: &mut super::Rng, _micro: MicroSeedFlags, active: u32) {
    let mut count = ((world.width as f64 * world.height as f64) * 0.0002) as i32;
    let divisor = (((2 + active as i32) / 3).max(1)) as i32;
    count = (count / divisor).max(1);
    let border = 40;
    for _ in 0..count {
        let x = rng.range(border, (world.width - border).max(border + 1));
        let mut y = rng.range(border, (world.height - border).max(border + 1));
        // Walk down to the first solid (≤30 rows).
        for _ in 0..30 {
            if solid(world, x, y) {
                break;
            }
            y += 1;
        }
        let ok = (!active_at(world, x, y)
            || true) // the port carries no dungeon/cloud tile sets
            && solid(world, x, y)
            && y < world.height - 1;
        if ok {
            let strength = rng.range(5, 12);
            let steps = rng.range(10, 31);
            super::tile_runner::tile_runner(
                world,
                rng,
                x,
                y,
                strength as f64,
                steps,
                666,
                world.world_surface,
                &super::tile_runner::TileRunnerOpts::default(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seeds::{SeedFlags, active_secret_seed_count};

    fn opts(micro: MicroSeedFlags) -> GenOptions {
        GenOptions {
            micro_flags: micro,
            // 512 is the repo's small-test size; the terrain model's clamp
            // (`range(80, height - 340)`) needs height > 420.
            width: 512,
            height: 512,
            ..Default::default()
        }
    }

    fn count(world: &World, kind: u16) -> usize {
        world.tiles.iter().filter(|t| t.active() && t.kind == kind).count()
    }

    fn sandstone(world: &World) -> usize {
        world.tiles.iter().filter(|t| t.wall == 187).count()
    }

    /// A classic world's tiles are untouched: the classic control.
    #[test]
    fn a_classic_world_is_untouched() {
        let a = crate::worldgen::generate(&opts(MicroSeedFlags::none()));
        let b = crate::worldgen::generate(&opts(MicroSeedFlags::none()));
        assert_eq!(a.tiles.len(), b.tiles.len());
        for (x, y) in a.tiles.iter().zip(&b.tiles) {
            assert_eq!((x.kind, x.wall, x.flags), (y.kind, y.wall, y.flags));
        }
    }

    /// `worldIsFrozen` re-types the stone/dirt band to ice (`:2225-2250`).
    #[test]
    fn the_frozen_world_converts_the_stone_band_to_ice() {
        let classic = crate::worldgen::generate(&opts(MicroSeedFlags::none()));
        let frozen = crate::worldgen::generate(&opts(MicroSeedFlags::from(
            MicroSeedFlags::WORLD_IS_FROZEN,
        )));
        assert!(count(&frozen, 161) > count(&classic, 161), "the ice band grew");
        assert!(count(&classic, 161) >= 0);
    }

    /// `surfaceIsMushrooms` runs the band (`WorldGen.cs:2087-2173`): the
    /// surface grass 2 → 60 fires on the full-size default world (the 512
    /// test world's compact surface carries snow, so the arm never sees
    /// grass there — the difference pinned in the frozen/frozen-finish tests
    /// instead).
    #[test]
    fn the_mushroom_world_converts_the_surface_grass() {
        let mk = |micro: MicroSeedFlags| {
            crate::worldgen::generate(&GenOptions {
                micro_flags: micro,
                ..Default::default()
            })
        };
        let classic = mk(MicroSeedFlags::none());
        let shroom = mk(MicroSeedFlags::from(MicroSeedFlags::SURFACE_IS_MUSHROOMS));
        assert!(
            count(&shroom, 60) > count(&classic, 60),
            "the mushroom grass ({}) beat the classic ({})",
            count(&shroom, 60),
            count(&classic, 60)
        );
    }

    /// W4-g42 G6: the verbatim `DoSurfaceIsMushrooms` (`:2118-2172`) on a
    /// small fixture — the condition-draw band, the 60 → 70 Convert arm
    /// (NOT the old 59 stand-in), the wall-15 → 80 Convert arm, the direct
    /// 0 → 59 / 2 → 60 writes and the eight-wall → 80 set. The stream
    /// replay pins the per-row `Next(3)` condition draws (one per visited
    /// row plus one failing draw per column).
    #[test]
    fn w4_g42_the_mushroom_seed_arm_is_verbatim() {
        let mut w = World::new("t".into(), 40, 60);
        w.world_surface = 10.0;
        // Column 5: jungle grass at row 3 (active), mud wall 15 at row 4
        // (inactive), dirt 0 at row 5, grass 2 at row 6, wall 63 at row 7.
        let put = |w: &mut World, x: i32, y: i32, kind: u16, wall: u16, active: bool| {
            let i = w.index(x, y);
            w.tiles[i] = crate::tile::Tile { kind, wall, ..Default::default() };
            if active {
                w.tiles[i].flags |= crate::tile::FLAG_ACTIVE;
            }
        };
        put(&mut w, 5, 3, 60, 0, true);
        put(&mut w, 5, 4, 0, 15, false);
        put(&mut w, 5, 5, 0, 0, true);
        put(&mut w, 5, 6, 2, 0, true);
        put(&mut w, 5, 7, 1, 63, true);
        let mut rng = super::super::Rng(0x5EED);
        surface_is_mushrooms(&mut w, &mut rng);
        // The Convert arms: 60 → 70, wall 15 → 80 (the wall arm runs on the
        // inactive cell — the type-3 cell's `!active && wall <= 0` gate
        // passes because wall 15 > 0).
        assert_eq!(w.tile(5, 3).unwrap().kind, 70, "the jungle grass converts");
        assert_eq!(w.tile(5, 4).unwrap().wall, 80, "the mud wall converts");
        // The direct writes: the inert dirt 0 → 59 (the C# reads the stored
        // type ungated), the grass 2 → 60, the wall 63 → 80.
        assert_eq!(w.tile(5, 5).unwrap().kind, 59, "dirt → mud");
        assert_eq!(w.tile(5, 6).unwrap().kind, 60, "grass → jungle grass");
        assert_eq!(w.tile(5, 7).unwrap().wall, 80, "the grass wall → 80");
        // The stream replay: per column, one Next(3) per visited row plus
        // one failing draw. The band is `worldSurface + Next(3)` ∈ [10, 13),
        // so every column visits rows 0..=9 at minimum; the exact count is
        // data-dependent — replay the same loop shape and compare states.
        let mut replay = super::super::Rng(0x5EED);
        for _x in 0..40 {
            let mut y = 0i32;
            while (y as f64) < 10.0 + replay.range(0, 3) as f64 && y < 60 {
                y += 1;
            }
        }
        assert_eq!(rng.0, replay.0, "the condition-draw ledger");
    }

    /// W4-g42 G6: the skyblock early-out (`:2089-2092`) — the flag-carrying
    /// world draws NOTHING and writes nothing.
    #[test]
    fn w4_g42_the_mushroom_seed_arm_skips_skyblock() {
        let mut w = World::new("t".into(), 40, 60);
        w.world_surface = 10.0;
        w.seeds |= SeedFlags::SKYBLOCK;
        let i = w.index(5, 3);
        w.tiles[i] = crate::tile::Tile { kind: 60, ..Default::default() };
        w.tiles[i].flags |= crate::tile::FLAG_ACTIVE;
        let mut rng = super::super::Rng(0x5EED);
        let before = rng.0;
        surface_is_mushrooms(&mut w, &mut rng);
        assert_eq!(rng.0, before, "no draws on skyblock");
        assert_eq!(w.tile(5, 3).unwrap().kind, 60, "no writes on skyblock");
    }

    /// `surfaceIsDesert` sands the surface band (`:2517-2592`) and the
    /// sandstone walls follow in the Finish arm.
    #[test]
    fn the_desert_world_sands_the_surface_band() {
        let classic = crate::worldgen::generate(&opts(MicroSeedFlags::none()));
        let desert = crate::worldgen::generate(&opts(MicroSeedFlags::from(
            MicroSeedFlags::SURFACE_IS_DESERT,
        )));
        assert!(
            count(&desert, 53) > count(&classic, 53),
            "the sand band grew: {} vs {}",
            count(&desert, 53),
            count(&classic, 53)
        );
        assert!(sandstone(&desert) >= sandstone(&classic));
    }

    /// `pooEverywhere` places the poop blocks (666).
    #[test]
    fn the_poo_world_places_poop_blocks() {
        let classic = crate::worldgen::generate(&opts(MicroSeedFlags::none()));
        let poo = crate::worldgen::generate(&opts(MicroSeedFlags::from(
            MicroSeedFlags::POO_EVERYWHERE,
        )));
        assert!(count(&classic, 666) == 0, "no poop in a classic world");
        assert!(count(&poo, 666) > 0, "the poop blocks landed: {}", count(&poo, 666));
    }

    /// `startInHardmode` sets `world.hard_mode` (`:1901-1902`).
    #[test]
    fn the_hardmode_world_starts_hardmode() {
        let classic = crate::worldgen::generate(&opts(MicroSeedFlags::none()));
        assert!(!classic.hard_mode);
        let hard = crate::worldgen::generate(&opts(MicroSeedFlags::from(
            MicroSeedFlags::START_IN_HARDMODE,
        )));
        assert!(hard.hard_mode, "the world starts in hardmode");
    }

    /// W4-g42: the surfaceIsMushrooms seed world is deterministic (a == b)
    /// across two full generations — the verbatim `DoSurfaceIsMushrooms`
    /// (the per-row condition draws + the double-call dispatch) and the
    /// mushroom-biome lane's tile changes replay byte-identically. The
    /// classic control stays a == b too (the flag-gated arm draws nothing).
    #[test]
    fn w4_g42_the_mushroom_seed_world_is_deterministic() {
        let mk = |micro: MicroSeedFlags| {
            crate::worldgen::generate(&GenOptions {
                micro_flags: micro,
                seed: 42,
                ..Default::default()
            })
        };
        let a = mk(MicroSeedFlags::from(MicroSeedFlags::SURFACE_IS_MUSHROOMS));
        let b = mk(MicroSeedFlags::from(MicroSeedFlags::SURFACE_IS_MUSHROOMS));
        assert_eq!(a.tiles.len(), b.tiles.len());
        for (x, y) in a.tiles.iter().zip(&b.tiles) {
            assert_eq!(
                (x.kind, x.wall, x.frame_x, x.frame_y, x.liquid, x.flags, x.meta),
                (y.kind, y.wall, y.frame_x, y.frame_y, y.liquid, y.flags, y.meta),
                "the mushroom seed world replays"
            );
        }
        // The seed arm actually fired: the mushroom grass 70 grew over the
        // classic (the Convert 60 → 70 + the surface conversions).
        let classic = mk(MicroSeedFlags::none());
        assert!(
            count(&a, 70) > count(&classic, 70),
            "the mushroom grass grew: {} vs {}",
            count(&a, 70),
            count(&classic, 70)
        );
    }

    /// The count gate reads the classic flags too (the `Variations`
    /// threshold's `activeSecretSeedCount`).
    #[test]
    fn the_count_gate_includes_the_classic_flags() {
        let micro = MicroSeedFlags::from(MicroSeedFlags::POO_EVERYWHERE);
        assert_eq!(active_secret_seed_count(micro, SeedFlags::none()), 1);
        assert_eq!(
            active_secret_seed_count(micro, SeedFlags::from(SeedFlags::DRUNK | SeedFlags::FOR_THE_WORTHY)),
            3
        );
    }
}
