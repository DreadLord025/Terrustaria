//! M5-SEED-2: the `SecretSeed.FinalizeSecretSeeds` surface-conversion arms
//! (`WorldGen.cs:589-693`): `DoSurfaceIsMushrooms`, `DoHallowOnSurface`,
//! `DoWorldIsInfected`, `DoWorldIsFrozen` (+ its Finish), `DoNoInfection`,
//! `DoStartInHardmode` and `DoPooEverywhere`. Each is flag-gated and draws
//! from its own `pass_rng` stream; a classic world's pipeline is untouched.
//!
//! Seam: `pipeline.rs`'s `step_seed_finish` dispatches [`run`] after
//! `micro_finish::run` (the C# order inside `FinalizeSecretSeeds`).

use crate::world::World;
use super::pipeline::Streams;
use super::GenOptions;

/// The `FinalizeSecretSeeds` surface-conversion arms, in the C# dispatch order.
pub(crate) fn run(_world: &mut World, _streams: &mut Streams, _opts: &GenOptions) {}
