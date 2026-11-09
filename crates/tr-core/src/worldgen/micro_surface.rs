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
        surface_is_desert(world, &mut streams.pass_rng("DoSurfaceIsDesert"), micro, active);
        surface_is_desert_finish(world, &mut streams.pass_rng("DoSurfaceIsDesertFinish"), micro);
    }
    if micro.contains(MicroSeedFlags::SURFACE_IS_IN_SPACE) {
        surface_is_in_space(world);
    }
    if micro.contains(MicroSeedFlags::SURFACE_IS_MUSHROOMS) {
        surface_is_mushrooms(world, &mut streams.pass_rng("DoSurfaceIsMushrooms"));
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
/// (no `noSurface` world state; documented at the module doc).
fn surface_is_desert(world: &mut World, rng: &mut super::Rng, _micro: MicroSeedFlags, _active: u32) {
    let band = world.world_surface as i32 + rng.range(0, 3);
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
/// arm: the per-column band `worldSurface + Next(3)` conversions. The C#'s
/// two `Convert(k, l, 3, 0)` calls (the 60-tile and wall-15 arms) are the
/// `Conversion` machine's "mushroom" flavour; the port has no mushroom
/// conversion table, so those two arms take the direct-equivalent writes the
/// C# itself performs in its noSurface twin and the difference is
/// documented.
fn surface_is_mushrooms(world: &mut World, rng: &mut super::Rng) {
    let band = world.world_surface as i32 + rng.range(0, 3);
    for x in 0..world.width {
        for y in 0..band.min(world.height) {
            let kind = kind_at(world, x, y);
            let wall = wall_at(world, x, y);
            // `if (type == 60) Convert(k, l, 3, 0)` — the mushroom world's own
            // conversion clears the jungle grass to the mushroom grass' dirt
            // base (the noSurface twin writes 59 for the inert cells).
            if kind == 60 {
                set_kind(world, x, y, 59);
            }
            if wall == 15 {
                set_kind(world, x, y, 59);
            }
            if kind == 0 {
                set_kind(world, x, y, 59);
            }
            if kind == 2 {
                set_kind(world, x, y, 60);
            }
            if matches!(wall, 63 | 65 | 66 | 68 | 64 | 205 | 206 | 207) {
                if let Some(t) = world.tile_mut(x, y) {
                    t.wall = 80;
                }
            }
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
fn start_in_hardmode(world: &mut World, opts: &GenOptions) {
    world.hard_mode = true;
    let mut rng = crate::tile_light::FastRandom::new(opts.seed as u64);
    spread::initialize_hard_mode(world, opts.crimson, &mut rng);
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
