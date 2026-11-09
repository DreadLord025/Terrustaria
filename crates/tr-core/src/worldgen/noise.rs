//! Deterministic value noise for the generation passes.
//!
//! The value-noise output is part of the world contract — changing a constant
//! here changes every generated world, which `deterministic_for_seed` pins. The
//! xorshift RNG the passes draw from moved to [`crate::rng`] (the unified
//! xorshift64 core shared with the wind walk); it stays re-exported here as
//! `Rng` so the passes' imports and seeding contract are unchanged.

/// Deterministic 2D value noise with smooth interpolation.
pub(crate) struct Noise {
    pub(crate) seed: u64,
}

impl Noise {
    #[inline]
    fn hash(&self, x: i32, y: i32) -> u32 {
        let mut h = self.seed
            ^ (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ (y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
        h ^= h >> 30;
        h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        h ^= h >> 27;
        h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
        h ^= h >> 31;
        h as u32
    }

    #[inline]
    fn unit(&self, x: i32, y: i32) -> f32 {
        self.hash(x, y) as f32 / u32::MAX as f32
    }

    #[inline]
    pub(crate) fn value(&self, x: f32, y: f32) -> f32 {
        let x0 = x.floor();
        let y0 = y.floor();
        let fx = x - x0;
        let fy = y - y0;
        let sx = fx * fx * (3.0 - 2.0 * fx);
        let sy = fy * fy * (3.0 - 2.0 * fy);
        let (x0, y0) = (x0 as i32, y0 as i32);
        let a = self.unit(x0, y0);
        let b = self.unit(x0 + 1, y0);
        let c = self.unit(x0, y0 + 1);
        let d = self.unit(x0 + 1, y0 + 1);
        let top = a + (b - a) * sx;
        let bottom = c + (d - c) * sx;
        (top + (bottom - top) * sy) * 2.0 - 1.0
    }

    pub(crate) fn fbm(&self, x: f32, y: f32, octaves: u32, gain: f32) -> f32 {
        let mut amp = 1.0;
        let mut sum = 0.0;
        let mut norm = 0.0;
        let mut px = x;
        let mut py = y;
        for _ in 0..octaves {
            sum += self.value(px, py) * amp;
            norm += amp;
            amp *= gain;
            px *= 2.0;
            py *= 2.0;
        }
        sum / norm
    }
}

/// The xorshift64 core the passes draw from — the unified
/// [`crate::rng::Xorshift64`] (shared with the wind walk, byte-exact),
/// re-exported under its historical name so every pass's import and the
/// worldgen seeding contract (`Rng(seed ^ const)`, the high-32-bit word draw)
/// stay unchanged.
pub(crate) use crate::rng::Xorshift64 as Rng;

#[cfg(test)]
mod tests {
    use super::*;

    /// DW-1.1: the fraction draw is pinned — the word-scaling convention
    /// (exact float bits), the `[0, 1)` bounds and the stream determinism.
    #[test]
    fn next_f64_is_bounded_deterministic_and_word_scaled() {
        // The convention: one word per draw, scaled into [0, 1) — the k-th
        // `next_f64` equals the k-th `next_u32` over 2^32, bit-exact.
        let mut a = Rng(0x0CEA);
        let mut b = Rng(0x0CEA);
        for _ in 0..64 {
            let word = b.next_u32();
            let got = a.next_f64();
            let expected = f64::from(word) / 4_294_967_296.0;
            assert_eq!(got.to_bits(), expected.to_bits());
            assert!((0.0..1.0).contains(&got));
        }
        // Determinism: a fresh same-seed stream replays the sequence.
        let mut c = Rng(0x0CEA);
        let mut d = Rng(0x0CEA);
        for _ in 0..64 {
            assert_eq!(c.next_f64().to_bits(), d.next_f64().to_bits());
        }
        // The draw never returns 1.0.
        let mut r = Rng(u64::MAX);
        for _ in 0..10_000 {
            assert!(r.next_f64() < 1.0);
        }
    }
}
