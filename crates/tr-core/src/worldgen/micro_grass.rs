//! M5-SEED-2: the `SecretSeed` bodies the C# runs from the **Grass pass**
//! slot (`WorldGen.cs:12423` — `DoDigExtraHoles` `:12427-12430`, the
//! `roundLandmasses` host otherwise): the flag-gated extra-cave body.
//!
//! Seam: `pipeline.rs`'s `step_micro_grass` at the vanilla Grass slot,
//! before `jungle` (`:12466`). A classic world's step draws nothing.

use super::pipeline::Streams;
use super::tile_runner::{tile_runner, TileRunnerOpts};
use crate::seeds::MicroSeedFlags;
use crate::world::World;
use super::GenOptions;

/// `SecretSeed.DoDigExtraHoles` (`WorldGen.cs:2845-2859`): `maxTilesX * 0.1`
/// air `TileRunner` calls scattered over the whole world, each carving an
/// ellipsoidal tunnel through whatever it meets.
///
/// ```text
/// int num = (int)(maxTilesX * 0.1);
/// for (i = 0; i < num; i++) {
///     int i2 = genRand.Next(50, maxTilesX - 50);
///     int j  = genRand.Next(50, maxTilesY - 50);
///     int num2 = genRand.Next(5, 16);
///     if (genRand.Next(3) == 0) num2 += genRand.Next(15);
///     WorldGen.TileRunner(i2, j, num2, genRand.Next(30, 201), -1, addTile: false,
///         (float)genRand.Next(-15, 16) * 0.1f, (float)genRand.Next(10, 26) * 0.1f,
///         noYChange: true);
/// }
/// ```
///
/// Draw order (the named arguments evaluate in source order, after the
/// positional ones): `i2`, `j`, `num2`, the 1-in-3 `Next(3)` (+ its
/// `Next(15)` when it fires), then `steps` `Next(30, 201)`, `speedX`
/// `Next(-15, 16)` and `speedY` `Next(10, 26)`. The port's
/// [`tile_runner`] takes `strength: f64` / `steps: i32`, which is the C#'s
/// `(double)num2` / `Next(30, 201)` pair, and `kind: -1` is the air carve.
///
/// The stream is the pass's own (`pass_rng("DigExtraHoles")` — the port's
/// model of vanilla's per-pass re-seed), so the call shifts no other pass's
/// draws; a classic world takes no draw at all.
pub(crate) fn run(world: &mut World, streams: &mut Streams, opts: &GenOptions) {
    // `WorldGen.cs:12427`: `if (SecretSeed.digExtraHoles.Enabled)`.
    if !opts.micro_flags.contains(MicroSeedFlags::DIG_EXTRA_HOLES) {
        return;
    }
    // `TileRunnerOpts::for_world` copies the world's classic seed flags
    // (vanilla's drunk/remix/for-the-worthy runner modifiers fire on EVERY
    // `TileRunner` call, `WorldGen.cs:76772-76788`), which needs the
    // immutable borrow to end before the `&mut World` walk below.
    let runner_opts = TileRunnerOpts::for_world(world);
    let world_surface = world.world_surface;
    let count = (world.width as f64 * 0.1) as i32;
    let mut rng = streams.pass_rng("DigExtraHoles");
    for _ in 0..count {
        let i = rng.range(50, world.width - 50);
        let j = rng.range(50, world.height - 50);
        let mut strength = rng.range(5, 16);
        if rng.range(0, 3) == 0 {
            strength += rng.range(0, 15);
        }
        let steps = rng.range(30, 201);
        let speed_x = rng.range(-15, 16) as f64 * 0.1;
        let speed_y = rng.range(10, 26) as f64 * 0.1;
        tile_runner(
            world,
            &mut rng,
            i,
            j,
            strength as f64,
            steps,
            -1,
            world_surface,
            &TileRunnerOpts {
                add_tile: false,
                speed_x,
                speed_y,
                no_y_change: true,
                ..runner_opts
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seeds::MicroSeedFlags;
    use crate::worldgen::{generate, GenOptions};

    /// The active-tile census of a generated world, used to tell a carved
    /// world from an untouched one.
    fn active_tiles(world: &World) -> usize {
        world.tiles.iter().filter(|t| t.active()).count()
    }

    /// `DoDigExtraHoles` carves only under the flag: the classic call draws
    /// nothing (byte-identical world) and the flagged call removes tiles.
    #[test]
    fn dig_extra_holes_carves_only_under_the_flag() {
        let base = GenOptions {
            width: 512,
            height: 512,
            seed: 42,
            ..Default::default()
        };
        let classic = generate(&base);
        let flagged = generate(&GenOptions {
            micro_flags: MicroSeedFlags::from(MicroSeedFlags::DIG_EXTRA_HOLES),
            ..base.clone()
        });
        // The classic world is untouched by the Grass slot: same active
        // count as the same seed with the flag off, and the flagged world
        // has strictly fewer active tiles.
        assert_eq!(
            active_tiles(&classic),
            active_tiles(&generate(&GenOptions {
                micro_flags: MicroSeedFlags::none(),
                ..base.clone()
            }))
        );
        assert!(
            active_tiles(&flagged) < active_tiles(&classic),
            "the flagged world must carve: classic {} flagged {}",
            active_tiles(&classic),
            active_tiles(&flagged)
        );
    }

    /// A solid test world the runner can carve (the `run` entry takes a
    /// painted world, not a fresh one).
    fn solid_world(width: i32, height: i32) -> World {
        let mut world = World::new("holes".into(), width, height);
        for x in 0..width {
            for y in 60..height - 60 {
                let i = world.index(x, y);
                let mut tile = crate::tile::Tile::default();
                tile.kind = 1;
                tile.flags |= crate::tile::FLAG_ACTIVE;
                world.tiles[i] = tile;
            }
        }
        world
    }

    /// The classic call draws nothing — the pass stream keyed
    /// `"DigExtraHoles"` is never touched, so a fresh one reproduces the
    /// seeded state, and the world is unchanged tile for tile.
    #[test]
    fn a_classic_call_draws_nothing() {
        let seed = 7u64;
        let opts = GenOptions {
            width: 512,
            height: 512,
            seed,
            ..Default::default()
        };
        let mut classic = solid_world(512, 512);
        let before = classic.tiles.clone();
        let mut streams = super::super::pipeline::Streams::new(seed);
        let untouched = streams.pass_rng("DigExtraHoles");
        run(&mut classic, &mut streams, &opts);
        assert_eq!(classic.tiles, before, "the classic world is untouched");
        // The pass stream advanced not at all: re-deriving it from the seed
        // yields the same state as the stream the untouched call left.
        let after = streams.pass_rng("DigExtraHoles");
        let mut expected = untouched;
        let mut got = after;
        for _ in 0..8 {
            assert_eq!(expected.next_u32(), got.next_u32());
        }
        // And the flagged call on the same world removes tiles.
        let mut flagged = solid_world(512, 512);
        run(
            &mut flagged,
            &mut streams,
            &GenOptions {
                micro_flags: MicroSeedFlags::from(MicroSeedFlags::DIG_EXTRA_HOLES),
                ..opts
            },
        );
        let removed = before
            .iter()
            .zip(flagged.tiles.iter())
            .filter(|(a, b)| a.active() && !b.active())
            .count();
        assert!(removed > 0, "the flagged call must carve air: {removed}");
    }
}
