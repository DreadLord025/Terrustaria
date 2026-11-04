//! Colour tables and the per-tile random source used by the tile-light scanner.
//!
//! These are the leaf pieces of `Terraria.Graphics.Light.TileLightScanner`: the
//! `FastRandom` stream, the torch palette, the HSL/paint helpers and the shimmer
//! wave. `scanner.rs` is the dispatch over tile kinds; everything here is a pure
//! function of its inputs.

use crate::lighting::LightParams;

/// `Main.cageFrames`.
pub const CAGE_FRAMES: usize = 25;

/// `Terraria.Utilities.FastRandom`, the scanner's per-tile random source.
///
/// `TileLightScanner` keeps one `FastRandom` that is re-seeded every frame
/// (`Update()` → `NextSeed()`, `TileLightScanner.cs:58`), then derives a per-tile
/// stream with `WithModifier(x, y)` (`TileLightScanner.cs:91`). That is what makes
/// torch light jitter per tile instead of pulsing uniformly.
#[derive(Clone, Copy, Debug)]
pub struct FastRandom {
    pub seed: u64,
}

impl FastRandom {
    const MULTIPLIER: u64 = 25214903917;
    const ADD: u64 = 11;
    const MASK: u64 = 281474976710655;

    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    #[inline]
    fn next_seed(seed: u64) -> u64 {
        seed.wrapping_mul(Self::MULTIPLIER).wrapping_add(Self::ADD) & Self::MASK
    }

    /// `FastRandom.NextSeed` (`FastRandom.cs:42`).
    pub fn next_seed_in_place(&mut self) {
        self.seed = Self::next_seed(self.seed);
    }

    /// `FastRandom.WithModifier(int x, int y)` (`FastRandom.cs:32`).
    pub fn with_modifier(&self, x: i32, y: i32) -> Self {
        let modifier = (x as i64 as u64)
            .wrapping_add(2654435769)
            .wrapping_add((y as i64 as u64) << 6)
            .wrapping_add((y as i64 as u64) >> 2);
        Self {
            seed: Self::next_seed(modifier) ^ self.seed,
        }
    }

    /// `FastRandom.NextBits` (`FastRandom.cs:47`).
    #[inline]
    fn next_bits(&mut self, bits: u32) -> i32 {
        self.seed = Self::next_seed(self.seed);
        (self.seed >> (48 - bits)) as i32
    }

    /// `FastRandom.NextFloat` (`FastRandom.cs:53`).
    pub fn next_float(&mut self) -> f32 {
        self.next_bits(24) as f32 * 5.9604645e-8
    }

    /// `FastRandom.Next(int max)` (`FastRandom.cs:63-76`): uniform in `0..max`.
    /// `Next(min, max)` is `Next(max - min) + min` (`:78-81`), so this is the
    /// same draw as [`Self::next_range`] with `min = 0`.
    pub fn next(&mut self, max: i32) -> i32 {
        self.next_range(0, max)
    }

    /// `FastRandom.Next(int min, int max)` (`FastRandom.cs:80`, the
    /// `Next(max - min) + min` overload).
    pub fn next_range(&mut self, min: i32, max: i32) -> i32 {
        let range = max - min;
        if range == 0 {
            return min;
        }
        // `if ((max & -max) == max)` — a power of two takes the fast path.
        if range & range.wrapping_neg() == range {
            return (((range as i64) * (self.next_bits(31) as i64)) >> 31) as i32 + min;
        }
        loop {
            let num = self.next_bits(31);
            let num2 = num % range;
            // C#'s unchecked arithmetic: the sum wraps past `int.MaxValue` to
            // negative exactly for the rejected draws, so the `>= 0` check
            // must wrap too (FastRandom.cs:80) — otherwise large ranges
            // (e.g. `Sandstorm.UpdateTime`'s `Next(64800)`) panic in debug.
            if num.wrapping_sub(num2).wrapping_add(range - 1) >= 0 {
                return num2 + min;
            }
        }
    }
}

/// `TorchID.TorchColor` (`TorchID.cs:165`) — base colour per torch id. Ids 7
/// (demon), 14 (disco) and 23 (shimmer) are animated and resolved by
/// [`torch_color`].
pub const TORCH_LIGHT: [[f32; 3]; 24] = [
    [1.0, 0.95, 0.8],
    [0.0, 0.1, 1.3],
    [1.0, 0.1, 0.1],
    [0.0, 1.0, 0.1],
    [0.9, 0.0, 0.9],
    [1.4, 1.4, 1.4],
    [0.9, 0.9, 0.0],
    // Demon torch: `0.5 * demonTorch + (1 - demonTorch)` / `0.3` /
    // `demonTorch + 0.5 * (1 - demonTorch)`.
    [0.8, 0.35, 0.35],
    [1.0, 1.6, 0.5],
    [0.75, 0.85, 1.4],
    [1.0, 0.5, 0.0],
    [1.4, 1.4, 0.7],
    [0.75, 1.35, 1.5],
    [0.95, 0.75, 1.3],
    // Disco torch: `Main.DiscoR/G/B / 255`.
    [0.85, 0.85, 0.85],
    [1.0, 0.0, 1.0],
    [1.4, 0.85, 0.55],
    [0.25, 1.3, 0.8],
    [0.95, 0.4, 1.4],
    [1.4, 0.7, 0.5],
    [1.25, 0.6, 1.2],
    [0.75, 1.45, 0.9],
    [0.3, 0.78, 1.2],
    // Shimmer torch: animated by `mouseTextColor`.
    [0.7, 0.55, 0.9],
];

/// `TorchID.TorchColor(torchID, out R, out G, out B)` (`TorchID.cs:165`), including
/// the animated providers (`DemonTorchLight`, `DiscoTorchLight`, `ShimmerTorchLight`).
pub fn torch_color(id: i32, p: &LightParams) -> [f32; 3] {
    if !(0..TORCH_LIGHT.len() as i32).contains(&id) {
        return [0.0; 3];
    }
    match id {
        7 => [
            0.5 * p.demon_torch + (1.0 - p.demon_torch),
            0.3,
            p.demon_torch + 0.5 * (1.0 - p.demon_torch),
        ],
        14 => [
            p.disco[0] as f32 / 255.0,
            p.disco[1] as f32 / 255.0,
            p.disco[2] as f32 / 255.0,
        ],
        23 => {
            // `ShimmerTorchLight`: base 0.9 with the mouseTextColor deltas.
            let mut num = 0.9 + (270.0 - p.mouse_text_color as f32) / 900.0;
            let mut num2 = 0.9 + (270.0 - p.mouse_text_color as f32) / 125.0;
            num = num.clamp(0.0, 1.0);
            num2 = num2.clamp(0.0, 1.0);
            [num * 0.9, num2 * 0.55, num * 1.2]
        }
        _ => TORCH_LIGHT[id as usize],
    }
}

/// `Main.hue2rgb`.
fn hue2rgb(mut c: f64, t1: f64, t2: f64) -> f64 {
    if c < 0.0 {
        c += 1.0;
    }
    if c > 1.0 {
        c -= 1.0;
    }
    if 6.0 * c < 1.0 {
        return t1 + (t2 - t1) * 6.0 * c;
    }
    if 2.0 * c < 1.0 {
        return t2;
    }
    if 3.0 * c < 2.0 {
        return t1 + (t2 - t1) * (2.0 / 3.0 - c) * 6.0;
    }
    t1
}

/// `Main.hslToRgb(Hue, Saturation, Luminosity)` (`Main.cs:47890`) — byte-quantised
/// like the original (`Math.Round`, which is banker's rounding in .NET; Rust's
/// `round` is half-away from zero, so the .NET `Math.Round(double)` overload is
/// reproduced explicitly).
pub fn hsl_to_rgb(hue: f32, saturation: f32, luminosity: f32) -> [f32; 3] {
    let round = |v: f64| -> f32 {
        // .NET Math.Round(double) = banker's rounding.
        let r = v.round_ties_even();
        (r as u8) as f32 / 255.0
    };
    if saturation == 0.0 {
        let v = round(luminosity as f64 * 255.0);
        return [v, v, v];
    }
    let hue = hue as f64;
    let sat = saturation as f64;
    let lum = luminosity as f64;
    let num2 = if lum < 0.5 {
        lum * (1.0 + sat)
    } else {
        lum + sat - lum * sat
    };
    let t = 2.0 * lum - num2;
    let c = hue + 1.0 / 3.0;
    let c2 = hue;
    let c3 = hue - 1.0 / 3.0;
    [
        round(hue2rgb(c, t, num2) * 255.0),
        round(hue2rgb(c2, t, num2) * 255.0),
        round(hue2rgb(c3, t, num2) * 255.0),
    ]
}

/// `WorldGen.paintColor(color)` (`WorldGen.cs:43921`) — the paint colour used when a
/// lamp is painted.
pub fn paint_color(color: u8) -> [f32; 3] {
    let (r, g, b) = match color {
        1 | 13 => (255, 0, 0),
        2 | 14 => (255, 127, 0),
        3 | 15 => (255, 255, 0),
        4 | 16 => (127, 255, 0),
        5 | 17 => (0, 255, 0),
        6 | 18 => (0, 255, 127),
        7 | 19 => (0, 255, 255),
        8 | 20 => (0, 127, 255),
        9 | 21 => (0, 0, 255),
        10 | 22 => (127, 0, 255),
        11 | 23 => (255, 0, 255),
        12 | 24 => (255, 0, 127),
        25 => (75, 75, 75),
        26 => (255, 255, 255),
        27 => (175, 175, 175),
        28 => (255, 178, 125),
        29 => (25, 25, 25),
        30 => (200, 200, 200),
        _ => (255, 255, 255),
    };
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0]
}

/// `LiquidRenderer.GetShimmerWave`.
fn shimmer_wave(x: f32, y: f32, time: f32) -> f32 {
    (((x + y / 6.0) / 10.0) - time / 360.0)
        .mul_add(std::f32::consts::TAU, 0.0)
        .sin()
}

/// `LiquidRenderer.GetShimmerBaseColor(x, y)`: a lerp between two purples driven by
/// the shimmer wave, sampled at tile coordinates.
pub fn shimmer_base_color(x: i32, y: i32, time: f32) -> [f32; 3] {
    let w = shimmer_wave(x as f32, y as f32, time);
    let amount = 0.1 + w * 0.4;
    let a = [165.0 / 255.0, 130.0 / 255.0, 14.0 / 15.0];
    let b = [205.0 / 255.0, 205.0 / 255.0, 1.0];
    [
        a[0] + (b[0] - a[0]) * amount,
        a[1] + (b[1] - a[1]) * amount,
        a[2] + (b[2] - a[2]) * amount,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: [f32; 3], b: [f32; 3]) {
        for i in 0..3 {
            assert!((a[i] - b[i]).abs() < 1e-5, "channel {i}: {a:?} != {b:?}");
        }
    }

    #[test]
    fn fast_random_matches_the_original_stream() {
        // `FastRandom(0)` -> `NextBits` values from the LCG
        // (seed * 25214903917 + 11) & 0xFFFFFFFFFFFF.
        let mut r = FastRandom::new(0);
        let first = r.next_range(0, 100);
        let mut r2 = FastRandom::new(0);
        assert_eq!(first, r2.next_range(0, 100));
        assert!((0..100).contains(&first));
        // WithModifier is a pure function of the seed and the position.
        let a = FastRandom::new(7).with_modifier(11, 13);
        let b = FastRandom::new(7).with_modifier(11, 13);
        assert_eq!(a.seed, b.seed);
        assert_ne!(a.seed, FastRandom::new(7).with_modifier(11, 14).seed);
    }

    #[test]
    fn hsl_to_rgb_matches_primary_colours() {
        close(hsl_to_rgb(0.0, 1.0, 0.5), [1.0, 0.0, 0.0]);
        close(hsl_to_rgb(1.0 / 3.0, 1.0, 0.5), [0.0, 1.0, 0.0]);
        close(hsl_to_rgb(2.0 / 3.0, 1.0, 0.5), [0.0, 0.0, 1.0]);
        // `Math.Round(127.5)` is banker's rounding -> 128.
        close(hsl_to_rgb(0.0, 0.0, 0.5), [128.0 / 255.0; 3]);
    }
    #[test]
    fn fast_random_next_n_matches_the_next_zero_min_range() {
        // `Next(max)` is the same draw as `Next(0, max)` (`FastRandom.cs:78-81`).
        let mut a = FastRandom::new(0xDEAD_BEEF);
        let mut b = FastRandom::new(0xDEAD_BEEF);
        for n in [1, 2, 3, 5, 8, 121] {
            let v = a.next(n);
            assert_eq!(v, b.next_range(0, n), "n = {n}");
            assert!((0..n).contains(&v), "n = {n}, v = {v}");
        }
        // `Next(1)` is always 0 (the only value in 0..1).
        let mut one = FastRandom::new(7);
        for _ in 0..16 {
            assert_eq!(one.next(1), 0);
        }
    }
}
