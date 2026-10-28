//! Tile frame resolution.
//!
//! In the original game `WorldGen.TileFrame` (~3.4k lines of `TileFrameCosmetic`)
//! computes frames for every non-"important" tile, including per-type merge
//! groups. This module is the v0.1 subset: important tiles keep their stored
//! frames, everything else is drawn with a per-sheet "plain" frame chosen at
//! atlas build time. The full port is tracked in `docs/ROADMAP.md`.

use crate::tile::Tile;

#[inline]
pub fn resolve(tile: &Tile, plain: (i16, i16)) -> (i16, i16) {
    if tile.frame_x >= 0 && tile.frame_y >= 0 {
        (tile.frame_x, tile.frame_y)
    } else {
        plain
    }
}
