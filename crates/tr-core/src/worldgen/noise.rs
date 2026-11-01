//! Deterministic value noise and the xorshift RNG shared by the generation passes.
//!
//! The output streams of both are part of the world contract — changing a
//! constant here changes every generated world, which `deterministic_for_seed`
//! pins.

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

/// Small xorshift RNG, seeded from the world seed.
pub(crate) struct Rng(pub(crate) u64);

impl Rng {
    pub(crate) fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 32) as u32
    }

    pub(crate) fn range(&mut self, lo: i32, hi: i32) -> i32 {
        debug_assert!(hi > lo);
        lo + (self.next_u32() % (hi - lo) as u32) as i32
    }

    pub(crate) fn chance(&mut self, n: u32) -> bool {
        self.next_u32() % n == 0
    }
}
