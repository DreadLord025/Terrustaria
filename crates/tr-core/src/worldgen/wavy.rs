//! The Don't Starve ("the constant") passes — the seed's own generation
//! work (`WorldGen.cs`): the `WavyCaves` pass (`:12271-12311`, the only
//! seed that runs it) and the `LavaLayerCaverer` the Dungeon pass appends
//! for the Constant (`:14867`, `:73950-74060`).
//!
//! M5-SEED-1 seam: this module is the Constant's pass bodies; the pipeline
//! hooks live in `pipeline.rs` (`step_wavy_caves`, the vanilla "Wavy Caves"
//! slot, and `step_constant_lava_layer`, the vanilla Dungeon-tail call). A
//! classic world never reaches either body and draws nothing — the port's
//! classic draws stay byte-identical.

use crate::world::World;
use super::pipeline::{GenVars, Streams};
use super::GenOptions;

/// `WorldGen.WavyCaves` (`WorldGen.cs:12271-12311`) — the pass body runs
/// only on the Constant; the placeholder below is replaced by the port.
pub(crate) fn wavy_caves(
    _world: &mut World,
    _gv: &mut GenVars,
    _streams: &mut Streams,
    _opts: &GenOptions,
) {
}

/// `WorldGen.LavaLayerCaverer` (`WorldGen.cs:73950-74060`) — the Dungeon
/// tail's horizontal two-row lava caver, Constant only.
pub(crate) fn lava_layer_caverer(_world: &mut World, _streams: &mut Streams, _opts: &GenOptions) {}
