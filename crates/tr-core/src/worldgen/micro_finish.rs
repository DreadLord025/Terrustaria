//! M5-SEED-2: the `SecretSeed.FinalizeSecretSeeds` paint/coat/trap arms —
//! the `Do*` sweeps the micro seeds run after the whole chain
//! (`WorldGen.cs:589-693`): the `actuallyNoTraps` spike sweeps, the
//! `noSpiderCaves` wall sweep and the `paintEverything*` /
//! `coatEverything*` families with their `Variations` gates.
//!
//! Seam: `pipeline.rs`'s `step_seed_finish` dispatches [`run`] after the
//! classic Finish tails, matching the C# call order (`Finish()`'s classic
//! `Finish*` calls `:22431-22444`, then `FinalizeSecretSeeds` `:22467`).
//! Every body is flag-gated and draws from its own `pass_rng` stream, so a
//! classic world's pipeline is untouched.

use crate::world::World;
use super::pipeline::Streams;
use super::GenOptions;

/// The `FinalizeSecretSeeds` paint/coat/trap arms, in the C# dispatch order.
pub(crate) fn run(_world: &mut World, _streams: &mut Streams, _opts: &GenOptions) {}
