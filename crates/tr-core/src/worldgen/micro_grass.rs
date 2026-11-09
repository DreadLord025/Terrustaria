//! M5-SEED-2: the `SecretSeed` bodies the C# runs from the **Grass pass**
//! slot (`WorldGen.cs:12423` — `DoDigExtraHoles` `:12427-12430`, the
//! `roundLandmasses` host otherwise): the flag-gated extra-cave body.
//!
//! Seam: `pipeline.rs`'s `step_micro_grass` at the vanilla Grass slot,
//! before `jungle` (`:12466`). A classic world's step draws nothing.

use crate::world::World;
use super::pipeline::Streams;
use super::GenOptions;

/// The Grass-slot micro bodies (`DoDigExtraHoles`, `WorldGen.cs:12427-12430`).
pub(crate) fn run(_world: &mut World, _streams: &mut Streams, _opts: &GenOptions) {}
