//! The directional blur that spreads the seeded light, plus the tile-medium
//! classification (`TileLightScanner.GetTileMask`) it decays through.

use rayon::prelude::*;

use super::LightMap;
use crate::tile::*;
use crate::tile_light::light_blocked;
use crate::tiledata as td;
use crate::world::World;

/// Edge rows/columns that are not blurred (`LightingEngine.NON_VISIBLE_PADDING`).
const NON_VISIBLE_PADDING: i32 = 18;

/// Light carried through a tile; matches `LightMaskMode`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Mask {
    Air,
    Solid,
    CrackedBricks,
    Water,
    Honey,
}

impl Mask {
    #[inline]
    fn decay(self) -> [f32; 3] {
        match self {
            Mask::Air => [0.91; 3],
            Mask::Solid => [0.56; 3],
            Mask::CrackedBricks => [0.8; 3],
            // `LightDecayThroughWater` for the default water style.
            Mask::Water => [0.88 * 0.91, 0.96 * 0.91, 1.015 * 0.91],
            Mask::Honey => [0.75 * 0.91, 0.7 * 0.91, 0.6 * 0.91],
        }
    }
}

/// `LightMap.Blur`: two passes over four line sweeps. Columns are swept first
/// (sequential: row-major storage makes them strided) and rows afterwards — in
/// parallel for the large rects that pass `PAR_LIGHT_MIN_CELLS`.
pub(super) fn blur(map: &mut LightMap, parallel: bool) {
    let (w, h, pad) = (map.w, map.h, NON_VISIBLE_PADDING.min(map.w.min(map.h) / 3));
    let wu = w as usize;

    for x in 0..w {
        // top -> bottom, then bottom -> top (stopping at the padding).
        blur_line_v(&mut map.colors, &map.mask, wu, x, 0, 1, h - pad);
        blur_line_v(&mut map.colors, &map.mask, wu, x, h - 1, -1, h - pad);
    }
    let count = w - pad;
    if parallel {
        map.colors
            .par_chunks_mut(wu)
            .zip(map.mask.par_chunks(wu))
            .enumerate()
            .for_each(|(ly, (colors, mask))| {
                let base = ly * wu;
                blur_line_h(colors, mask, base, 0, 1, count);
                blur_line_h(colors, mask, base, w - 1, -1, count);
            });
    } else {
        for (ly, (colors, mask)) in map
            .colors
            .chunks_mut(wu)
            .zip(map.mask.chunks(wu))
            .enumerate()
        {
            let base = ly * wu;
            blur_line_h(colors, mask, base, 0, 1, count);
            blur_line_h(colors, mask, base, w - 1, -1, count);
        }
    }
}

/// One vertical sweep through column `x`, carrying the brightest light seen so far
/// and decaying it by the tile's opacity (`LightMap.BlurLine`).
fn blur_line_v(
    colors: &mut [[f32; 3]],
    mask: &[Mask],
    w: usize,
    x: i32,
    y0: i32,
    dy: i32,
    count: i32,
) {
    let mut carried = [0.0f32; 3];
    let mut done = [false; 3];
    for k in 0..count.max(0) {
        let y = y0 + dy * k;
        let i = y as usize * w + x as usize;
        let m = mask[i];
        let decay = m.decay();
        let jitter = if m == Mask::Water {
            let j = (i as u32).wrapping_mul(2_654_435_761) % 15;
            0.985 + j as f32 * 0.001
        } else {
            1.0
        };
        let color = &mut colors[i];
        for c in 0..3 {
            if carried[c] < color[c] {
                carried[c] = color[c];
                done[c] = false;
            } else if !done[c] {
                if carried[c] < 0.0185 {
                    done[c] = true;
                } else {
                    color[c] = carried[c];
                }
            }
        }
        if done[0] && done[1] && done[2] {
            continue;
        }
        for c in 0..3 {
            if !done[c] {
                carried[c] *= decay[c] * jitter;
            }
        }
    }
}

/// One horizontal sweep through a single row slice. `base` is the global index of
/// `colors[0]` in the full light map, so the water jitter stays bit-identical to the
/// sequential implementation.
fn blur_line_h(colors: &mut [[f32; 3]], mask: &[Mask], base: usize, x0: i32, dx: i32, count: i32) {
    let mut carried = [0.0f32; 3];
    let mut done = [false; 3];
    for k in 0..count.max(0) {
        let lx = (x0 + dx * k).max(0) as usize;
        let i = base + lx;
        let m = mask[lx];
        let decay = m.decay();
        let jitter = if m == Mask::Water {
            let j = (i as u32).wrapping_mul(2_654_435_761) % 15;
            0.985 + j as f32 * 0.001
        } else {
            1.0
        };
        let color = &mut colors[lx];
        for c in 0..3 {
            if carried[c] < color[c] {
                carried[c] = color[c];
                done[c] = false;
            } else if !done[c] {
                if carried[c] < 0.0185 {
                    done[c] = true;
                } else {
                    color[c] = carried[c];
                }
            }
        }
        if done[0] && done[1] && done[2] {
            continue;
        }
        for c in 0..3 {
            if !done[c] {
                carried[c] *= decay[c] * jitter;
            }
        }
    }
}

/// Top of the column's medium: the first solid tile or the first liquid cell,
/// whichever is higher. Seeding stops there, so a lakebed under water is lit by
/// the attenuated light coming through the water instead of a direct sky seed.
pub(super) fn surface_cutoff(world: &World, x: i32) -> i32 {
    if x < 0 || x >= world.width {
        return 0;
    }
    let first_solid = world
        .column_surface
        .get(x as usize)
        .copied()
        .unwrap_or(world.world_surface as i32);
    for y in 0..first_solid.max(0) {
        if world.tile(x, y).is_some_and(|t| t.liquid > 0) {
            return y;
        }
    }
    first_solid
}

/// `TileLightScanner.GetTileMask` — `LightIsBlocked` inside it consumes the
/// tile's `invisibleBlock` bit (`TileLightScanner.cs:66-75,151-162`).
pub(super) fn mask_of(
    tile: &Tile,
    x: i32,
    y: i32,
    draw_invisible: bool,
    world: &World,
) -> Mask {
    let invisible_block = world.invisible_block(x, y);
    if light_blocked(tile, invisible_block, draw_invisible) && tile.kind != 131 && tile.slope() == 0
    {
        if td::SET_CRACKED_BRICKS.has(tile.kind) {
            return Mask::CrackedBricks;
        }
        return Mask::Solid;
    }
    if tile.liquid > 128 && tile.liquid_kind() != LiquidKind::Lava {
        return match tile.liquid_kind() {
            LiquidKind::Honey => Mask::Honey,
            _ => Mask::Water,
        };
    }
    Mask::Air
}
