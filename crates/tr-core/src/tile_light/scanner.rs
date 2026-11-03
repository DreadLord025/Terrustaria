//! `Terraria.Graphics.Light.TileLightScanner.ApplyTileLight` — the self-lit tile
//! types.
//!
//! The original seeds each visible tile with `GetTileLight`
//! (`TileLightScanner.cs:87`), which combines surface sunlight, the underworld glow,
//! glowing walls, self-lit tile types and liquids. The big `ApplyTileLight` switch
//! (`TileLightScanner.cs:339`) is a 1:1 port here: every case is translated in the
//! order of the C# source, so frame-dependent colours (torch variants, chandeliers,
//! lamps, gemspark, painted lamps) match the original exactly.
//!
//! Tile types whose light is a plain constant are still taken from the generated
//! `tiledata::EMISSIVE` table; this module only owns the cases that the table cannot
//! express (conditions, animation, per-tile random jitter, paint overrides).
//!
//! The original switch is one flat match over ~110 top-level cases. The arms are
//! mutually exclusive tile-kind literals, so this module partitions them into
//! per-theme private matches — `torch_lights`, `cage_lights`, `gemspark_lights`,
//! `paintable_lights`, `misc_lights` — called in a fixed chain. Every kind is
//! handled by exactly one group and each group keeps the arms in the original
//! relative order, so the observable behaviour (including the `FastRandom` draw
//! sequence, which only the firing group consumes) is identical to the flat match.

use super::palette::{
    hsl_to_rgb, paint_color, shimmer_base_color, torch_color, FastRandom, CAGE_FRAMES,
};
use crate::lighting::LightParams;
use crate::tile::*;
use crate::world::World;

/// `TileLightScanner.ApplyTileLight` (`TileLightScanner.cs:339`) — self-lit tile
/// types.
///
/// The C# keeps `R`/`G`/`B` locals and a `flag` that means "the colour came from a
/// lamp, so a paint coat overrides it". Both are reproduced here; the final
/// `max(lightColor, R/G/B)` happens in the caller's accumulator.
pub fn apply_tile_light(
    world: &World,
    tile: &Tile,
    x: i32,
    y: i32,
    rng: &mut FastRandom,
    p: &LightParams,
    c: &mut [f32; 3],
) {
    if (tile.kind as usize) >= crate::tiledata::TILE_COUNT
        || !crate::tiledata::TILE_LIGHTED.has(tile.kind)
    {
        return;
    }
    let fx = tile.frame_x as i32;
    let fy = tile.frame_y as i32;
    let kind = tile.kind as i32;
    let mut l = [0.0f32; 3];
    let mut paint = false;
    // `tile.invisibleBlock()` (`TileLightScanner.cs:350`): an invisible
    // self-lit tile does not emit (torch 658). The bit lives in `World.extras`.
    let invisible = world.invisible_block(x, y);

    // The dispatch chain over the partitioned cases; see the module doc.
    torch_lights(kind, invisible, fx, fy, rng, p, &mut l);
    cage_lights(kind, x, y, fx, fy, p, &mut l);
    gemspark_lights(kind, world, x, y, fx, fy, &mut l);
    paintable_lights(tile, x, y, fx, fy, kind, rng, p, &mut l, &mut paint);
    misc_lights(world, tile, kind, x, y, fx, fy, rng, p, c, &mut l);

    // `if (flag && tile.color() != 0)`: a painted lamp takes the paint colour.
    if paint && tile.paint() != 0 {
        l = paint_color(tile.paint());
    }
    for i in 0..3 {
        if c[i] < l[i] {
            c[i] = l[i];
        }
    }
}

/// The flame sources: the shimmer torch tiles (`TileLightScanner.cs:349`, `:932`),
/// the torch (`:1522`), the campfires (`:1020`, `:1062`), the fireplace (`:1160`)
/// and the furnace (`:2483`). Their colours come from `TorchID.TorchColor` or the
/// shared flicker recipes.
fn torch_lights(
    kind: i32,
    invisible: bool,
    fx: i32,
    fy: i32,
    rng: &mut FastRandom,
    p: &LightParams,
    l: &mut [f32; 3],
) {
    match kind {
        658 => {
            if !invisible {
                *l = torch_color(23, p);
                let k = match fy / 54 {
                    1 => 0.3,
                    2 => 0.1,
                    _ => 0.2,
                };
                *l = [l[0] * k, l[1] * k, l[2] * k];
            }
        }
        660 => *l = torch_color(23, p),
        405 => {
            if fx < 54 {
                let n = rng.next_range(28, 42) as f32 * 0.005
                    + (270.0 - p.mouse_text_color as f32) / 700.0;
                *l = match fx / 54 {
                    1 => [0.7, 1.0, 0.5],
                    2 => {
                        let d = p.demon_torch;
                        [0.5 * d + 1.0 * (1.0 - d), 0.3, 1.0 * d + 0.5 * (1.0 - d)]
                    }
                    3 => [0.45, 0.75, 1.0],
                    4 => [1.15, 1.15, 0.5],
                    5 => [
                        p.disco[0] as f32 / 255.0,
                        p.disco[1] as f32 / 255.0,
                        p.disco[2] as f32 / 255.0,
                    ],
                    _ => [0.9, 0.3, 0.1],
                };
                *l = [l[0] + n, l[1] + n, l[2] + n];
            }
        }
        215 => {
            if fy < 36 {
                let n = rng.next_range(28, 42) as f32 * 0.005
                    + (270.0 - p.mouse_text_color as f32) / 700.0;
                *l = match fx / 54 {
                    1 => [0.7, 1.0, 0.5],
                    2 => {
                        let d = p.demon_torch;
                        [0.5 * d + 1.0 * (1.0 - d), 0.3, 1.0 * d + 0.5 * (1.0 - d)]
                    }
                    3 => [0.45, 0.75, 1.0],
                    4 => [1.15, 1.15, 0.5],
                    5 => [
                        p.disco[0] as f32 / 255.0,
                        p.disco[1] as f32 / 255.0,
                        p.disco[2] as f32 / 255.0,
                    ],
                    6 => [0.75, 1.2825, 1.2],
                    7 => [0.95, 0.65, 1.3],
                    8 => [1.4, 0.85, 0.55],
                    9 => [0.25, 1.3, 0.8],
                    10 => [0.95, 0.4, 1.4],
                    11 => [1.4, 0.7, 0.5],
                    12 => [1.25, 0.6, 1.2],
                    13 => [0.75, 1.45, 0.9],
                    14 => [0.25, 0.65, 1.0],
                    15 => torch_color(23, p),
                    _ => [0.9, 0.3, 0.1],
                };
                *l = [l[0] + n, l[1] + n, l[2] + n];
            }
        }
        592 => {
            if fy > 0 {
                let n = rng.next_range(28, 42) as f32 * 0.005
                    + (270.0 - p.mouse_text_color as f32) / 700.0;
                *l = [1.35 + n, 0.45 + n, 0.15 + n];
            }
        }
        4 => {
            if fx < 66 {
                *l = torch_color(fy / 22, p);
            }
        }
        37 => {
            let n = rng.next_range(95, 106) as f32 * 0.01;
            *l = [0.56 * n, 0.43 * n, 0.15 * n];
        }
        _ => {}
    }
}

/// The jellyfish cages (`TileLightScanner.cs:714`): the light follows the cage's
/// animation frame.
fn cage_lights(kind: i32, x: i32, y: i32, fx: i32, fy: i32, p: &LightParams, l: &mut [f32; 3]) {
    match kind {
        316 | 317 | 318 => {
            // Jellyfish cages: the light follows the cage's animation frame.
            let num18 = x - fx / 18;
            let num19 = y - fy / 18;
            let num20 = (num18 / 3 * (num19 / 3)).rem_euclid(CAGE_FRAMES as i32) as usize;
            let num21 = (kind - 316) as usize;
            let flag6 = p.cage_mode[num21][num20] == 2;
            *l = match (kind, flag6) {
                (316, true) => [0.2, 0.3, 0.8],
                (316, false) => [0.1, 0.2, 0.5],
                (317, true) => [0.2, 0.7, 0.3],
                (317, false) => [0.05, 0.45, 0.1],
                (318, true) => [0.7, 0.2, 0.5],
                (318, false) => [0.4, 0.1, 0.25],
                _ => [0.0; 3],
            };
        }
        _ => {}
    }
}

/// The gemspark family (`TileLightScanner.cs:870`, `:2500`): the block's frame index
/// encodes the four colour bits, and the gem colour lives in the parent tile's
/// packed frameY bits.
fn gemspark_lights(kind: i32, world: &World, x: i32, y: i32, fx: i32, fy: i32, l: &mut [f32; 3]) {
    match kind {
        429 => {
            // Gemspark blocks: the frame index encodes the four colour bits.
            let n = fx / 18;
            let f2 = n % 2 >= 1;
            let f3 = n % 4 >= 2;
            let f4 = n % 8 >= 4;
            let f5 = n % 16 >= 8;
            if f2 {
                l[0] += 0.5;
            }
            if f3 {
                l[1] += 0.5;
            }
            if f4 {
                l[2] += 0.5;
            }
            if f5 {
                l[0] += 0.2;
                l[1] += 0.2;
            }
        }
        171 => {
            // The gem colour lives in the parent tile's packed frameY bits.
            let (tx, ty) = if fx < 10 { (x - fx, y - fy) } else { (x, y) };
            let bits = world
                .tile(tx, ty)
                .map_or(0, |t| (t.frame_y as i32 & 0x3C00) >> 10);
            *l = match bits {
                1 => [0.1, 0.1, 0.1],
                2 => [0.2, 0.0, 0.0],
                3 => [0.0, 0.2, 0.0],
                4 => [0.0, 0.0, 0.2],
                5 => [0.125, 0.125, 0.0],
                6 => [0.2, 0.1, 0.0],
                7 => [0.125, 0.125, 0.0],
                8 => [0.08, 0.175, 0.0],
                9 => [0.0, 0.125, 0.125],
                10 => [0.125, 0.0, 0.125],
                11 => [0.1, 0.1, 0.2],
                _ => [0.0; 3],
            };
            *l = [l[0] * 0.5, l[1] * 0.5, l[2] * 0.5];
        }
        _ => {}
    }
}

/// The paintable fixtures — lamp (`TileLightScanner.cs:1202`), candle (`:1544`),
/// chandelier (`:2151`), hanging lamp (`:1846`), lantern (`:2566`) and the glowing
/// mushroom family (`:2903`, `:2919`, `:2942`). Their cases set the C# `flag`, so a
/// paint coat overrides the frame colour.
#[allow(clippy::too_many_arguments)]
fn paintable_lights(
    tile: &Tile,
    x: i32,
    y: i32,
    fx: i32,
    fy: i32,
    kind: i32,
    rng: &mut FastRandom,
    p: &LightParams,
    l: &mut [f32; 3],
    paint: &mut bool,
) {
    match kind {
        93 => {
            if fx == 0 {
                *l = match fy / 54 {
                    1 => [0.95, 0.95, 0.5],
                    2 => [0.85, 0.6, 1.0],
                    3 => [0.75, 1.0, 0.6],
                    4 => [0.75, 0.85, 1.0],
                    5 => [0.75, 0.85, 1.0],
                    6 => [1.0, 0.95, 0.65],
                    7 => [1.0, 0.97, 0.85],
                    9 => [1.0, 1.0, 0.7],
                    10 => [1.0, 0.95, 0.65],
                    12 => [1.0, 0.95, 0.65],
                    13 => [1.0, 1.0, 0.6],
                    14 => [1.0, 0.95, 0.65],
                    18 => [1.0, 0.95, 0.65],
                    19 => {
                        if tile.paint() == 0 {
                            [0.37, 0.8, 1.0]
                        } else {
                            *paint = true;
                            [0.0; 3]
                        }
                    }
                    20 => [0.0, 0.9, 1.0],
                    21 => [0.25, 0.7, 1.0],
                    23 => torch_color(7, p),
                    24 => [0.35, 0.5, 0.3],
                    25 => [0.34, 0.4, 0.31],
                    26 => [0.25, 0.32, 0.5],
                    29 => [0.9, 0.75, 1.0],
                    30 => [1.0, 0.95, 0.65],
                    31 => {
                        let v = hsl_to_rgb(p.demon_torch * 0.12 + 0.69, 1.0, 0.75);
                        [v[0] * 1.2, v[1] * 1.2, v[2] * 1.2]
                    }
                    32 => [1.0, 0.97, 0.85],
                    33 => [0.55, 0.45, 0.95],
                    34 => [1.0, 0.6, 0.1],
                    35 => [0.3, 0.75, 0.55],
                    36 => [0.9, 0.55, 0.7],
                    37 => [0.55, 0.85, 1.0],
                    38 => [1.0, 0.95, 0.65],
                    39 => [1.0, 0.95, 0.65],
                    40 => [0.4, 0.8, 0.9],
                    41 => [1.0, 1.0, 1.0],
                    42 => [0.95, 0.5, 0.4],
                    43 => {
                        let v = shimmer_base_color(x, y, p.time_for_visuals);
                        [
                            (v[0] * 1.5).clamp(0.0, 1.0),
                            (v[1] * 1.5).clamp(0.0, 1.0),
                            (v[2] * 1.5).clamp(0.0, 1.0),
                        ]
                    }
                    44 => [1.0, 0.95, 0.65],
                    45 => [1.0, 0.66666667, 0.77647059],
                    46 => [1.0, 0.95, 0.65],
                    47 => [0.95294118, 0.90588235, 0.36078431],
                    48 => [0.63529412, 0.50196078, 1.0],
                    49 => [1.0, 0.39215686, 0.39215686],
                    50 => [0.74509804, 0.74509804, 1.0],
                    51 => [0.66666667, 0.70588235, 1.0],
                    52 => [1.0, 0.95, 0.65],
                    53 => [1.0, 0.95, 0.75],
                    54 => [1.0, 0.85499996, 0.585],
                    55 => {
                        *paint = true;
                        [0.5, 0.9, 1.0]
                    }
                    56 => [1.0, 0.9, 0.9],
                    57 => [0.70588235, 0.90196078, 1.0],
                    58 => [0.58823529, 0.92156863, 0.96078431],
                    59 => [0.66666667, 0.96078431, 1.0],
                    60 => [1.0, 0.95, 0.65],
                    61 => [1.0, 0.95, 0.65],
                    62 => [0.92156863, 0.41176471, 1.0],
                    63 => [0.74509804, 0.74509804, 1.0],
                    64 => [0.84313725, 0.68627451, 0.96078431],
                    _ => [1.0, 0.97, 0.85],
                };
            }
        }

        33 => {
            if fx == 0 {
                *l = match fy / 22 {
                    0 => [1.0, 0.95, 0.65],
                    1 => [0.55, 0.85, 0.35],
                    2 => [0.65, 0.95, 0.5],
                    3 => [0.2, 0.75, 1.0],
                    5 => [0.85, 0.6, 1.0],
                    7 => [0.75, 0.85, 1.0],
                    8 => [0.75, 0.85, 1.0],
                    9 => [1.0, 0.95, 0.65],
                    10 => [1.0, 0.97, 0.85],
                    14 => [1.0, 1.0, 0.6],
                    15 => [1.0, 0.95, 0.65],
                    18 => [1.0, 0.95, 0.65],
                    19 => {
                        if tile.paint() == 0 {
                            [0.37, 0.8, 1.0]
                        } else {
                            *paint = true;
                            [0.0; 3]
                        }
                    }
                    20 => [0.0, 0.9, 1.0],
                    21 => [0.25, 0.7, 1.0],
                    23 => [1.0, 0.95, 0.65],
                    24 => [1.0, 0.95, 0.65],
                    25 => torch_color(7, p),
                    28 => [0.9, 0.75, 1.0],
                    29 => [1.0, 0.95, 0.65],
                    30 => {
                        let v = hsl_to_rgb(p.demon_torch * 0.12 + 0.69, 1.0, 0.75);
                        [v[0] * 1.2, v[1] * 1.2, v[2] * 1.2]
                    }
                    31 => [1.0, 0.97, 0.85],
                    32 => [0.55, 0.45, 0.95],
                    33 => [1.0, 0.6, 0.1],
                    34 => [0.3, 0.75, 0.55],
                    35 => [0.9, 0.55, 0.7],
                    36 => [0.55, 0.85, 1.0],
                    37 => [1.0, 0.95, 0.65],
                    38 => [1.0, 0.95, 0.65],
                    39 => [0.4, 0.8, 0.9],
                    40 => [1.0, 1.0, 1.0],
                    41 => [0.95, 0.5, 0.4],
                    42 => {
                        let v = shimmer_base_color(x, y, p.time_for_visuals);
                        [
                            (v[0] * 1.5).clamp(0.0, 1.0),
                            (v[1] * 1.5).clamp(0.0, 1.0),
                            (v[2] * 1.5).clamp(0.0, 1.0),
                        ]
                    }
                    43 => [1.0, 0.95, 0.65],
                    44 => [1.0, 0.66666667, 0.77647059],
                    45 => [1.0, 0.95, 0.65],
                    46 => [0.95294118, 0.90588235, 0.36078431],
                    47 => [0.63529412, 0.50196078, 1.0],
                    48 => [1.0, 0.39215686, 0.39215686],
                    49 => [0.74509804, 0.74509804, 1.0],
                    50 => [0.66666667, 0.70588235, 1.0],
                    51 => [1.0, 0.95, 0.65],
                    52 => [1.0, 0.95, 0.75],
                    53 => [1.0, 0.85499996, 0.585],
                    54 => {
                        *paint = true;
                        [0.5, 0.9, 1.0]
                    }
                    55 => [1.0, 0.9, 0.9],
                    56 => [0.70588235, 0.90196078, 1.0],
                    57 => [0.58823529, 0.92156863, 0.96078431],
                    58 => [0.66666667, 0.96078431, 1.0],
                    59 => [1.0, 0.95, 0.65],
                    60 => [1.0, 0.95, 0.65],
                    61 => [0.92156863, 0.41176471, 1.0],
                    62 => [0.74509804, 0.74509804, 1.0],
                    63 => [0.84313725, 0.68627451, 0.96078431],
                    _ => [1.0, 0.95, 0.65],
                };
            }
        }

        34 => {
            if fx % 108 < 54 {
                *l = match fy / 54 + 37 * (fx / 108) {
                    7 => [0.95, 0.95, 0.5],
                    8 => [0.85, 0.6, 1.0],
                    9 => [1.0, 0.6, 0.6],
                    12 => [1.0, 0.95, 0.65],
                    13 => [1.0, 0.97, 0.85],
                    11 => [0.75, 0.85, 1.0],
                    17 => [0.75, 0.85, 1.0],
                    15 => [1.0, 1.0, 0.7],
                    16 => [1.0, 0.95, 0.65],
                    18 => [1.0, 1.0, 0.6],
                    19 => [1.0, 0.95, 0.65],
                    23 => [1.0, 0.95, 0.65],
                    24 => {
                        if tile.paint() == 0 {
                            [0.37, 0.8, 1.0]
                        } else {
                            *paint = true;
                            [0.0; 3]
                        }
                    }
                    25 => [0.0, 0.9, 1.0],
                    26 => [0.25, 0.7, 1.0],
                    27 => [0.55, 0.85, 0.35],
                    28 => [0.65, 0.95, 0.5],
                    29 => [0.2, 0.75, 1.0],
                    30 => [1.0, 0.95, 0.65],
                    32 => torch_color(7, p),
                    35 => [0.9, 0.75, 1.0],
                    36 => [1.0, 0.95, 0.65],
                    37 => {
                        let v = hsl_to_rgb(p.demon_torch * 0.12 + 0.69, 1.0, 0.75);
                        [v[0] * 1.2, v[1] * 1.2, v[2] * 1.2]
                    }
                    38 => [1.0, 0.97, 0.85],
                    39 => [0.55, 0.45, 0.95],
                    40 => [1.0, 0.6, 0.1],
                    41 => [0.3, 0.75, 0.55],
                    42 => [0.9, 0.55, 0.7],
                    43 => [0.55, 0.85, 1.0],
                    44 => [1.0, 0.95, 0.65],
                    45 => [1.0, 0.95, 0.65],
                    46 => [0.4, 0.8, 0.9],
                    47 => [1.0, 1.0, 1.0],
                    48 => [0.95, 0.5, 0.4],
                    49 => {
                        let v = shimmer_base_color(x, y, p.time_for_visuals);
                        [
                            (v[0] * 1.5).clamp(0.0, 1.0),
                            (v[1] * 1.5).clamp(0.0, 1.0),
                            (v[2] * 1.5).clamp(0.0, 1.0),
                        ]
                    }
                    50 => [1.0, 0.95, 0.65],
                    51 => [1.0, 0.66666667, 0.77647059],
                    52 => [1.0, 0.95, 0.65],
                    53 => [0.95294118, 0.90588235, 0.36078431],
                    54 => [0.63529412, 0.50196078, 1.0],
                    55 => [1.0, 0.39215686, 0.39215686],
                    56 => [0.74509804, 0.74509804, 1.0],
                    57 => [0.66666667, 0.70588235, 1.0],
                    58 => [1.0, 0.95, 0.65],
                    59 => [1.0, 0.95, 0.75],
                    60 => [1.0, 0.85499996, 0.585],
                    61 => {
                        *paint = true;
                        [0.5, 0.9, 1.0]
                    }
                    62 => [1.0, 0.9, 0.9],
                    63 => [0.70588235, 0.90196078, 1.0],
                    64 => [0.58823529, 0.92156863, 0.96078431],
                    65 => [0.66666667, 0.96078431, 1.0],
                    66 => [1.0, 0.95, 0.65],
                    67 => [1.0, 0.95, 0.65],
                    68 => [0.92156863, 0.41176471, 1.0],
                    69 => [0.74509804, 0.74509804, 1.0],
                    70 => [0.84313725, 0.68627451, 0.96078431],
                    _ => [1.0, 0.95, 0.8],
                };
            }
        }

        100 | 173 => {
            if fx < 36 {
                *l = match fy / 36 {
                    1 => [0.95, 0.95, 0.5],
                    2 => [0.85, 0.6, 1.0],
                    3 => [1.0, 0.6, 0.6],
                    5 => [1.0, 0.95, 0.65],
                    6 => [1.0, 0.95, 0.65],
                    7 => [1.0, 0.95, 0.65],
                    8 => [1.0, 0.97, 0.85],
                    9 => [0.75, 0.85, 1.0],
                    11 => [1.0, 1.0, 0.7],
                    12 => [1.0, 0.95, 0.65],
                    13 => [1.0, 1.0, 0.6],
                    14 => [1.0, 0.95, 0.65],
                    18 => [1.0, 0.95, 0.65],
                    19 => {
                        if tile.paint() == 0 {
                            [0.37, 0.8, 1.0]
                        } else {
                            *paint = true;
                            [0.0; 3]
                        }
                    }
                    20 => [0.0, 0.9, 1.0],
                    21 => [0.25, 0.7, 1.0],
                    25 => torch_color(7, p),
                    22 => [0.35, 0.5, 0.3],
                    23 => [0.34, 0.4, 0.31],
                    24 => [0.25, 0.32, 0.5],
                    29 => [0.9, 0.75, 1.0],
                    30 => [1.0, 0.95, 0.65],
                    31 => {
                        let v = hsl_to_rgb(p.demon_torch * 0.12 + 0.69, 1.0, 0.75);
                        [v[0] * 1.2, v[1] * 1.2, v[2] * 1.2]
                    }
                    32 => [1.0, 0.97, 0.85],
                    33 => [0.55, 0.45, 0.95],
                    34 => [1.0, 0.6, 0.1],
                    35 => [0.3, 0.75, 0.55],
                    36 => [0.9, 0.55, 0.7],
                    37 => [0.55, 0.85, 1.0],
                    38 => [1.0, 0.95, 0.65],
                    39 => [1.0, 0.95, 0.65],
                    40 => [0.4, 0.8, 0.9],
                    41 => [1.0, 1.0, 1.0],
                    42 => [0.95, 0.5, 0.4],
                    43 => {
                        let v = shimmer_base_color(x, y, p.time_for_visuals);
                        [
                            (v[0] * 1.5).clamp(0.0, 1.0),
                            (v[1] * 1.5).clamp(0.0, 1.0),
                            (v[2] * 1.5).clamp(0.0, 1.0),
                        ]
                    }
                    44 => [1.0, 0.95, 0.65],
                    45 => [1.0, 0.66666667, 0.77647059],
                    46 => [1.0, 0.95, 0.65],
                    47 => [0.95294118, 0.90588235, 0.36078431],
                    48 => [0.63529412, 0.50196078, 1.0],
                    49 => [1.0, 0.39215686, 0.39215686],
                    50 => [0.74509804, 0.74509804, 1.0],
                    51 => [0.66666667, 0.70588235, 1.0],
                    52 => [1.0, 0.95, 0.65],
                    53 => [1.0, 0.95, 0.75],
                    54 => [1.0, 0.85499996, 0.585],
                    55 => {
                        *paint = true;
                        [0.5, 0.9, 1.0]
                    }
                    56 => [1.0, 0.9, 0.9],
                    57 => [0.70588235, 0.90196078, 1.0],
                    58 => [0.58823529, 0.92156863, 0.96078431],
                    59 => [0.66666667, 0.96078431, 1.0],
                    60 => [1.0, 0.95, 0.65],
                    61 => [1.0, 0.95, 0.65],
                    62 => [0.92156863, 0.41176471, 1.0],
                    63 => [0.74509804, 0.74509804, 1.0],
                    64 => [0.84313725, 0.68627451, 0.96078431],
                    _ => [1.0, 0.95, 0.65],
                };
            }
        }

        42 => {
            if fx == 0 {
                *l = match fy / 36 {
                    0 => [0.7, 0.65, 0.55],
                    1 => [0.9, 0.75, 0.6],
                    2 => [0.8, 0.6, 0.6],
                    3 => [0.65, 0.5, 0.2],
                    4 => [0.5, 0.7, 0.4],
                    5 => [0.9, 0.4, 0.2],
                    6 => [0.7, 0.75, 0.3],
                    7 => torch_color(7, p),
                    8 => [0.75, 0.6, 0.3],
                    9 => torch_color(7, p),
                    11 => [0.85, 0.6, 1.0],
                    14 => [1.0, 0.95, 0.65],
                    15 => [1.0, 0.95, 0.65],
                    16 => [1.0, 0.95, 0.65],
                    17 => [1.0, 0.97, 0.85],
                    18 => [0.75, 0.85, 1.0],
                    21 => [1.0, 0.95, 0.65],
                    22 => [1.0, 1.0, 0.6],
                    23 => [1.0, 0.95, 0.65],
                    27 => [1.0, 0.95, 0.65],
                    28 => {
                        if tile.paint() == 0 {
                            [0.37, 0.8, 1.0]
                        } else {
                            *paint = true;
                            [0.0; 3]
                        }
                    }
                    29 => [0.0, 0.9, 1.0],
                    30 => [0.25, 0.7, 1.0],
                    32 => torch_color(7, p),
                    35 => [0.7, 0.6, 0.9],
                    36 => [1.0, 0.95, 0.65],
                    37 => {
                        let v = hsl_to_rgb(p.demon_torch * 0.12 + 0.69, 1.0, 0.75);
                        [v[0] * 1.2, v[1] * 1.2, v[2] * 1.2]
                    }
                    38 => [1.0, 0.97, 0.85],
                    39 => [0.55, 0.45, 0.95],
                    40 => [1.0, 0.6, 0.1],
                    41 => [0.3, 0.75, 0.55],
                    42 => [0.9, 0.55, 0.7],
                    43 => [0.55, 0.85, 1.0],
                    44 => [1.0, 0.95, 0.65],
                    45 => [1.0, 0.95, 0.65],
                    46 => [0.4, 0.8, 0.9],
                    47 => [1.0, 1.0, 1.0],
                    48 => [0.95, 0.5, 0.4],
                    49 => {
                        let v = shimmer_base_color(x, y, p.time_for_visuals);
                        [
                            (v[0] * 1.5).clamp(0.0, 1.0),
                            (v[1] * 1.5).clamp(0.0, 1.0),
                            (v[2] * 1.5).clamp(0.0, 1.0),
                        ]
                    }
                    50 => [1.0, 0.95, 0.65],
                    51 => [1.0, 0.66666667, 0.77647059],
                    52 => [1.0, 0.95, 0.65],
                    53 => [0.95294118, 0.90588235, 0.36078431],
                    54 => [0.63529412, 0.50196078, 1.0],
                    55 => [1.0, 0.39215686, 0.39215686],
                    56 => [0.74509804, 0.74509804, 1.0],
                    57 => [0.66666667, 0.70588235, 1.0],
                    58 => [1.0, 0.95, 0.65],
                    59 => [1.0, 0.95, 0.75],
                    60 => [1.0, 0.85499996, 0.585],
                    61 => {
                        *paint = true;
                        [0.5, 0.9, 1.0]
                    }
                    62 => [1.0, 0.9, 0.9],
                    63 => [0.70588235, 0.90196078, 1.0],
                    64 => [0.58823529, 0.92156863, 0.96078431],
                    65 => [0.66666667, 0.96078431, 1.0],
                    66 => [1.0, 0.95, 0.65],
                    67 => [1.0, 0.95, 0.65],
                    68 => [0.92156863, 0.41176471, 1.0],
                    69 => [0.74509804, 0.74509804, 1.0],
                    70 => [0.84313725, 0.68627451, 0.96078431],
                    _ => [1.0, 1.0, 1.0],
                };
            }
        }

        519 => {
            if fy == 90 {
                if tile.paint() == 0 {
                    let n = rng.next_range(28, 42) as f32 * 0.005
                        + (270.0 - p.mouse_text_color as f32) / 1000.0;
                    *l = [0.1, 0.2 + n / 2.0, 0.7 + n];
                } else {
                    *paint = true;
                }
            }
        }
        70 | 71 | 72 | 190 | 348 | 349 | 528 | 578 => {
            if kind != 349 || fx >= 36 {
                let n = rng.next_range(28, 42) as f32 * 0.005
                    + (270.0 - p.mouse_text_color as f32) / 1000.0;
                if tile.paint() == 0 {
                    *l = [0.0, 0.2 + n / 2.0, 1.0];
                } else {
                    *paint = true;
                }
            }
        }
        739 => {
            *l = [0.35, 0.63, 0.7];
            *paint = true;
        }
        _ => {}
    }
}

/// Everything else: the constant emitters, the frame tables and the one-off recipes
/// (`TileLightScanner.cs:373` … `:3103`).
#[allow(clippy::too_many_arguments)]
fn misc_lights(
    world: &World,
    tile: &Tile,
    kind: i32,
    x: i32,
    y: i32,
    fx: i32,
    fy: i32,
    rng: &mut FastRandom,
    p: &LightParams,
    c: &mut [f32; 3],
    l: &mut [f32; 3],
) {
    match kind {
        356 => {
            if p.sundial_cooldown == 0 {
                *l = [0.45, 0.25, 0.0];
            }
        }
        663 => {
            if p.moondial_cooldown == 0 {
                *l = [0.0, 0.25, 0.45];
            }
        }
        656 => *l = [0.2, 0.55, 0.5],
        20 => {
            let n = fx / 18;
            if (30..=32).contains(&n) {
                *l = [0.325, 0.15, 0.05];
            }
        }
        634 => *l = [0.65, 0.3, 0.1],
        633 | 637 | 638 => *l = [0.325, 0.15, 0.05],
        463 => *l = [0.2, 0.4, 0.8],
        491 => *l = [0.5, 0.4, 0.7],
        209 => {
            // `PortalHelper.GetPortalColor(Main.myPlayer, 0|1)` in single player:
            // the two fixed portal hues, scaled by 0.65.
            if fx == 234 || fx == 252 {
                let v = hsl_to_rgb(0.12, 1.0, 0.5);
                *l = [v[0] * 0.65, v[1] * 0.65, v[2] * 0.65];
            } else if fx == 306 || fx == 324 {
                let v = hsl_to_rgb(0.52, 1.0, 0.6);
                *l = [v[0] * 0.65, v[1] * 0.65, v[2] * 0.65];
            }
        }
        415 => *l = [0.7, 0.5, 0.1],
        500 => *l = [0.525, 0.375, 0.075],
        416 => *l = [0.0, 0.6, 0.7],
        501 => *l = [0.0, 0.45, 0.525],
        417 => *l = [0.6, 0.2, 0.6],
        502 => *l = [0.45, 0.15, 0.45],
        418 => *l = [0.6, 0.6, 0.9],
        503 => *l = [0.45, 0.45, 0.675],
        390 => *l = [0.4, 0.2, 0.1],
        597 => {
            *l = match fx / 54 {
                0 => [0.05, 0.8, 0.3],
                1 => [0.7, 0.8, 0.05],
                2 => [0.7, 0.5, 0.9],
                3 => [0.6, 0.6, 0.8],
                4 => [0.4, 0.4, 1.15],
                5 => [0.85, 0.45, 0.1],
                6 => [0.8, 0.8, 1.0],
                7 => [0.5, 0.8, 1.2],
                _ => [0.0; 3],
            };
            *l = [l[0] * 0.75, l[1] * 0.75, l[2] * 0.75];
        }
        564 => {
            if fx < 36 {
                *l = [0.05, 0.3, 0.55];
            }
        }
        568 => *l = [1.0, 0.61, 0.65],
        569 => *l = [0.12, 1.0, 0.66],
        570 => *l = [0.57, 0.57, 1.0],
        580 => *l = [0.7, 0.3, 0.2],
        391 => *l = [0.3, 0.1, 0.25],
        381 | 517 | 687 => *l = [0.25, 0.1, 0.0],
        534 | 535 | 689 => *l = [0.0, 0.25, 0.0],
        536 | 537 | 690 => *l = [0.0, 0.16, 0.34],
        539 | 540 | 688 => *l = [0.3, 0.0, 0.17],
        625 | 626 | 691 => *l = [0.3, 0.0, 0.35],
        627 | 628 | 692 => {
            *l = [
                p.disco[0] as f32 / 255.0 * 0.25,
                p.disco[1] as f32 / 255.0 * 0.25,
                p.disco[2] as f32 / 255.0 * 0.25,
            ]
        }
        184 => {
            if fx == 110 {
                *l = [0.25, 0.1, 0.0];
            }
            if fx == 132 {
                *l = [0.0, 0.25, 0.0];
            }
            if fx == 154 {
                *l = [0.0, 0.16, 0.34];
            }
            if fx == 176 {
                *l = [0.3, 0.0, 0.17];
            }
            if fx == 198 {
                *l = [0.3, 0.0, 0.35];
            }
            if fx == 220 {
                *l = [
                    p.disco[0] as f32 / 255.0 * 0.25,
                    p.disco[1] as f32 / 255.0 * 0.25,
                    p.disco[2] as f32 / 255.0 * 0.25,
                ];
            }
        }
        370 => *l = [0.32, 0.16, 0.12],
        659 | 667 | 708 => *l = shimmer_base_color(x, y, p.time_for_visuals),
        711 => *l = [0.01, 0.01, 0.01],
        27 => {
            if fy < 36 {
                *l = [0.3, 0.27, 0.0];
            }
        }
        336 => *l = [0.85, 0.5, 0.3],
        340 => *l = [0.45, 1.0, 0.45],
        341 => {
            *l = [
                0.4 * p.demon_torch + 0.6 * (1.0 - p.demon_torch),
                0.35,
                1.0 * p.demon_torch + 0.6 * (1.0 - p.demon_torch),
            ]
        }
        342 => *l = [0.5, 0.5, 1.1],
        343 => *l = [0.85, 0.85, 0.3],
        344 => *l = [0.6, 1.026, 0.960_000_04],
        327 => {
            // `0.5 + (270 - mouseTextColor) / 1500 + random * 0.0005`.
            let n = 0.5
                + (270.0 - p.mouse_text_color as f32) / 1500.0
                + rng.next_range(0, 50) as f32 * 0.0005;
            *l = [n, 0.5 * n, 0.1 * n];
        }
        719 => {
            // Rainbow brick: a 14-step hue wheel indexed by position and time.
            let n = (x + y + (p.global_time * 15.0) as i32) % 14;
            let (r, g, b) = match n {
                0 => (255, 171, 183),
                1 => (255, 170, 220),
                2 => (252, 171, 255),
                3 => (224, 171, 255),
                4 => (192, 171, 255),
                5 => (174, 178, 255),
                6 => (168, 195, 255),
                7 => (167, 224, 255),
                8 => (168, 255, 252),
                9 => (162, 255, 233),
                10 => (158, 255, 198),
                11 => (207, 255, 173),
                12 => (255, 213, 186),
                13 => (255, 192, 182),
                _ => (0, 0, 0),
            };
            *l = [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0];
        }
        718 => {
            // Night-only dim glow, only when nothing solid sits on top.
            if !p.day_time && !solid_tile3(world, x, y - 1) {
                *l = [
                    rng.next_float() * 0.04 + 0.1 + p.disco[0] as f32 / 800.0,
                    rng.next_float() * 0.04 + 0.1 + p.disco[1] as f32 / 800.0,
                    rng.next_float() * 0.04 + 0.1 + p.disco[2] as f32 / 800.0,
                ];
            }
        }
        717 => {
            let n = (0.55 + (270.0 - p.mouse_text_color as f32) / 800.0 + rng.next_float() * 0.03)
                * 0.5;
            *l = [n * 1.1, n * 0.4, n * 0.1];
        }
        286 | 619 => *l = [0.1, 0.2, 0.7],
        620 => {
            // `new Color(230,230,230,0).MultiplyRGBA(hsl) * 0.4f` — XNA multiplies
            // the bytes with `>> 8`, then the float scale converts back to a byte.
            let h = hsl_to_rgb((p.global_time * 0.5) % 1.0, 1.0, 0.5);
            let q = |a: f32, b: f32| -> f32 {
                let byte = ((a * 255.0) as i32 * (b * 255.0) as i32) >> 8;
                let scaled = (byte as f32 * 0.4) as i32;
                scaled as f32 / 255.0
            };
            *l = [
                q(230.0 / 255.0, h[0]),
                q(230.0 / 255.0, h[1]),
                q(230.0 / 255.0, h[2]),
            ];
        }
        582 | 598 => *l = [0.7, 0.2, 0.1],
        270 => *l = [0.73, 1.0, 0.41],
        271 => *l = [0.45, 0.95, 1.0],
        581 => *l = [1.0, 0.75, 0.5],
        572 => {
            *l = match fy / 36 {
                0 => [0.9, 0.5, 0.7],
                1 => [0.7, 0.55, 0.96],
                2 => [0.45, 0.96, 0.95],
                3 => [0.5, 0.96, 0.62],
                4 => [0.47, 0.69, 0.95],
                5 => [0.92, 0.57, 0.51],
                _ => [0.0; 3],
            }
        }
        262 => *l = [0.75, 0.0, 0.75],
        263 => *l = [0.75, 0.75, 0.0],
        264 => *l = [0.0, 0.0, 0.75],
        265 => *l = [0.0, 0.75, 0.0],
        266 => *l = [0.75, 0.0, 0.0],
        267 => *l = [0.75, 0.75, 0.75],
        268 => *l = [0.75, 0.375, 0.0],
        237 => *l = [0.1, 0.1, 0.0],
        238 => {
            // These two write the accumulated colour directly instead of R/G/B.
            if c[0] < 0.5 {
                c[0] = 0.5;
            }
            if c[2] < 0.5 {
                c[2] = 0.5;
            }
        }
        235 => {
            if c[0] < 0.6 {
                c[0] = 0.6;
            }
            if c[1] < 0.6 {
                c[1] = 0.6;
            }
        }
        92 => {
            if fy <= 18 && fx == 0 {
                *l = [1.0, 1.0, 1.0];
            }
        }
        593 => {
            if fx < 18 {
                *l = [0.8, 0.3, 0.1];
            }
        }
        594 => {
            if fx < 36 {
                *l = [0.8, 0.3, 0.1];
            }
        }
        548 => {
            if fx / 54 >= 7 {
                *l = [0.7, 0.3, 0.2];
            }
        }
        613 | 614 => *l = [0.7, 0.3, 0.2],
        96 => {
            if fx >= 36 {
                *l = [0.5, 0.35, 0.1];
            }
        }
        98 => {
            if fy == 0 {
                *l = [1.0, 0.97, 0.85];
            }
        }
        372 => {
            if fx == 0 {
                *l = [0.9, 0.1, 0.75];
            }
        }
        646 => {
            if fx == 0 {
                *l = [0.2, 0.3, 0.32];
            }
        }
        174 => {
            if fx == 0 {
                *l = [1.0, 0.95, 0.65];
            }
        }
        35 => {
            if fx < 36 {
                *l = [0.75, 0.6, 0.3];
            }
        }
        95 => {
            if fx < 36 {
                *l = [1.0, 0.95, 0.8];
            }
        }
        17 | 133 | 302 => *l = [0.83, 0.6, 0.5],
        77 => *l = [0.75, 0.45, 0.25],
        22 | 140 => {
            if tile.paint() != 27 && tile.paint() != 26 {
                l[0] = 0.12;
            }
            l[1] = 0.07;
            l[2] = 0.32;
        }
        204 | 347 => {
            if tile.paint() != 27 && tile.paint() != 26 {
                l[0] = 0.35;
            }
        }
        49 => {
            if fx == 0 {
                *l = [0.0, 0.35, 0.8];
            }
        }
        350 => {
            // `(int)(t / 6.283) % 3 == 1 ? t : 0` inside a cosine pulse.
            let n = p.time_for_visuals as f64 * 0.08;
            let phase = if (n / 6.283) as i32 % 3 == 1 { n } else { 0.0 };
            let v = (0.0 - phase.cos() * 0.1 + 0.1) as f32;
            *l = [v, v, v];
        }
        61 | 703 => {
            if fx == 144 {
                let a = 1.0 + (270.0 - p.mouse_text_color as f32) / 400.0;
                let b = 0.8 - (270.0 - p.mouse_text_color as f32) / 400.0;
                *l = [0.42 * b, 0.81 * a, 0.52 * b];
            }
        }
        26 | 31 | 695 | 696 => {
            let is_lamp = ((kind == 31 || kind == 696) && fx >= 36)
                || ((kind == 26 || kind == 695) && fx >= 54);
            if is_lamp {
                let n = rng.next_range(-5, 6) as f32 * 0.0025;
                *l = [0.5 + n * 2.0, 0.2 + n, 0.1];
            } else {
                let n = rng.next_range(-5, 6) as f32 * 0.0025;
                *l = [0.31 + n, 0.1, 0.44 + n * 2.0];
            }
        }
        699 => *l = [0.4, 0.2, 0.15],
        84 => match fx / 18 {
            2 => {
                let n = ((270.0 - p.mouse_text_color as f32) / 400.0).clamp(0.0, 1.0);
                *l = [n * 1.4, n * 1.2, n / 2.0];
            }
            5 => *l = [0.9, 0.72, 0.18],
            6 => *l = [0.0, 0.08 * 0.8, 0.08],
            _ => {}
        },
        83 => {
            if fx == 18 && !p.day_time {
                *l = [0.1, 0.4, 0.6];
            }
            if fx == 90 && !p.raining && p.time > 40500.0 {
                *l = [0.9, 0.72, 0.18];
            }
        }
        126 => {
            if fx < 36 {
                *l = [
                    p.disco[0] as f32 / 255.0,
                    p.disco[1] as f32 / 255.0,
                    p.disco[2] as f32 / 255.0,
                ];
            }
        }
        125 => {
            let n =
                rng.next_range(28, 42) as f32 * 0.01 + (270.0 - p.mouse_text_color as f32) / 800.0;
            l[1] = 0.3 * n;
            l[2] = 0.6 * n;
            // The C# assigns `lightColor.Y/Z` directly here, not through R/G/B.
            c[1] = l[1];
            c[2] = l[2];
        }
        129 => {
            *l = match (fx / 18) % 3 {
                0 => [0.0, 0.05, 0.25],
                1 => [0.2, 0.0, 0.15],
                2 => [0.1, 0.0, 0.2],
                _ => [0.0; 3],
            }
        }
        149 => {
            if fx <= 36 {
                *l = match fx / 18 {
                    0 => [0.1, 0.2, 0.5],
                    1 => [0.5, 0.1, 0.1],
                    2 => [0.2, 0.5, 0.1],
                    _ => [0.0; 3],
                };
                // Three independent draws, in channel order.
                let a = rng.next_range(970, 1031) as f32 * 0.001;
                let b = rng.next_range(970, 1031) as f32 * 0.001;
                let d = rng.next_range(970, 1031) as f32 * 0.001;
                *l = [l[0] * a, l[1] * b, l[2] * d];
            }
        }
        160 => {
            *l = [
                p.disco[0] as f32 / 255.0 * 0.25,
                p.disco[1] as f32 / 255.0 * 0.25,
                p.disco[2] as f32 / 255.0 * 0.25,
            ]
        }
        354 => *l = [0.65, 0.35, 0.15],
        _ => {}
    }
}

/// `WorldGen.SolidTile3`: a solid, non-platform block that is not actuated.
fn solid_tile3(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && t.flags & FLAG_INACTIVE == 0
        && crate::tiledata::TILE_SOLID.has(t.kind)
        && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    fn tile(kind: u16, fx: i16, fy: i16) -> Tile {
        let mut t = Tile::default();
        t.kind = kind;
        t.frame_x = fx;
        t.frame_y = fy;
        t.flags |= FLAG_ACTIVE;
        t
    }

    /// Light produced by one tile, before the blur.
    fn lit(t: &Tile, x: i32, y: i32, p: &LightParams) -> [f32; 3] {
        let w = World::new("t".into(), 64, 64);
        let mut c = [0.0f32; 3];
        let mut rng = FastRandom::new(p.rng_seed).with_modifier(x, y);
        apply_tile_light(&w, t, x, y, &mut rng, p, &mut c);
        c
    }

    fn close(a: [f32; 3], b: [f32; 3]) {
        for i in 0..3 {
            assert!((a[i] - b[i]).abs() < 1e-5, "channel {i}: {a:?} != {b:?}");
        }
    }

    #[test]
    fn torch_variants_use_the_frame_row() {
        let p = LightParams::default();
        // `TorchID.Torch` (frameY 0) and `TorchID.Blue` (frameY 22).
        close(lit(&tile(4, 0, 0), 5, 5, &p), [1.0, 0.95, 0.8]);
        close(lit(&tile(4, 0, 22), 5, 5, &p), [0.0, 0.1, 1.3]);
        // frameX >= 66 is the tall torch sprite and emits nothing.
        close(lit(&tile(4, 66, 0), 5, 5, &p), [0.0; 3]);
    }

    #[test]
    fn demon_and_disco_torches_are_animated() {
        let mut p = LightParams::default();
        p.demon_torch = 1.0;
        close(lit(&tile(4, 0, 7 * 22), 5, 5, &p), [0.5, 0.3, 1.0]);
        p.demon_torch = 0.0;
        close(lit(&tile(4, 0, 7 * 22), 5, 5, &p), [1.0, 0.3, 0.5]);
        // Disco torch: the light is exactly `Main.DiscoR/G/B / 255`.
        p.disco = [255, 128, 0];
        close(
            lit(&tile(4, 0, 14 * 22), 5, 5, &p),
            [1.0, 128.0 / 255.0, 0.0],
        );
    }

    #[test]
    fn candles_and_lamps_follow_their_frame_tables() {
        let p = LightParams::default();
        // Candle, green (frameY / 22 == 1).
        close(lit(&tile(33, 0, 22), 5, 5, &p), [0.55, 0.85, 0.35]);
        // Lamp on (frameY / 54 == 1), unpainted.
        close(lit(&tile(93, 0, 54), 5, 5, &p), [0.95, 0.95, 0.5]);
        // Blue lamp (case 19) with no paint.
        close(lit(&tile(93, 0, 54 * 19), 5, 5, &p), [0.37, 0.8, 1.0]);
        // A painted lamp takes the paint colour instead of the frame colour.
        let mut painted = tile(93, 0, 54 * 19);
        painted.set_paint(1);
        close(lit(&painted, 5, 5, &p), [1.0, 0.0, 0.0]);
    }

    #[test]
    fn gemspark_encodes_colour_in_the_frame_column() {
        let p = LightParams::default();
        // frameX / 18 == 15: red + green + blue bits and the extra red/green pair.
        close(lit(&tile(429, 18 * 15, 0), 5, 5, &p), [0.7, 0.7, 0.5]);
        close(lit(&tile(429, 0, 0), 5, 5, &p), [0.0; 3]);
    }

    #[test]
    fn lantern_frame_sets_the_colour() {
        let p = LightParams::default();
        // Lantern (42), frameY / 36 == 1.
        close(lit(&tile(42, 0, 36), 5, 5, &p), [0.9, 0.75, 0.6]);
        // frameX != 0 is a multi-tile part that must not glow on its own.
        close(lit(&tile(42, 18, 36), 5, 5, &p), [0.0; 3]);
    }

    #[test]
    fn campfire_only_lights_its_lit_frames() {
        let p = LightParams::default();
        // Default frame: `0.9, 0.3, 0.1` plus the 28..42 random flicker term.
        let c = lit(&tile(215, 0, 0), 5, 5, &p);
        assert!(c[0] > 0.9 && c[1] > 0.3 && c[1] < 0.6, "campfire {c:?}");
        // Green campfire (frameX / 54 == 1).
        let c = lit(&tile(215, 54, 0), 5, 5, &p);
        assert!(c[1] > c[0] && c[2] < c[1], "green campfire {c:?}");
        close(lit(&tile(215, 0, 36), 5, 5, &p), [0.0; 3]);
    }
}
