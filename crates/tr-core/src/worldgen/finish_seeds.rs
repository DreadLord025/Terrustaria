//! The seed `Finish*` passes — the tail work `GenerateWorld` runs after the
//! whole chain (`WorldGen.cs:22431-22444`): `FinishDrunkGen` (`:26226`),
//! `FinishGetGoodWorld` (`:25799`), `FinishTenthAnniversaryWorld`
//! (`:24237`). Each is gated on its seed flag, so a classic world's
//! pipeline is unchanged (M5-SEED-1's seam — the pipeline hook is
//! `pipeline.rs`'s `step_seed_finish`).

use crate::world::World;
use super::pipeline::Streams;
use super::GenOptions;

/// `WorldGen.FinishDrunkGen` (`WorldGen.cs:26226-26348`).
pub(crate) fn finish_drunk_gen(_world: &mut World, _streams: &mut Streams, _opts: &GenOptions) {}

/// `WorldGen.FinishGetGoodWorld` (`WorldGen.cs:25799-25983`).
pub(crate) fn finish_get_good_world(
    _world: &mut World,
    _streams: &mut Streams,
    _opts: &GenOptions,
) {
}

/// `WorldGen.FinishTenthAnniversaryWorld` (`WorldGen.cs:24237-24282`).
pub(crate) fn finish_tenth_anniversary_world(
    _world: &mut World,
    _streams: &mut Streams,
    _opts: &GenOptions,
) {
}
