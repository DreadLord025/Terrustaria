//! The shared xorshift64 core — the worldgen streams and the wind walk.
//!
//! Before `refactor-xorshift-unify` two xorshift64 cores lived apart:
//! [`worldgen::noise`]'s `Rng` (the worldgen contract — raw-XOR seeding, the
//! high-32-bit word draw) and `wind.rs`'s private `Rng` (splitmix64 seeding,
//! the full word with its high half for the modulo). The triple and the word
//! extraction are identical, so one core now carries both draw conventions
//! **byte-exactly** — every generated world's streams and every wind history
//! are unchanged (pinned by `deterministic_for_seed`, the worldgen replay-pair
//! tests and the wind tests).
//!
//! Determinism is a hard invariant: the step, the word extraction and the
//! seeding below are part of the world/session contract — never "improve"
//! them (AGENTS.md, `docs/code-standards.md`). The ported C# stream stays
//! `tile_light::palette::FastRandom` (a 48-bit LCG) and must not be confused
//! with this core.

/// The xorshift64 core: the 13/7/17 triple over a `u64` state.
///
/// The tuple field is the raw state — the worldgen contract seeds each stream
/// as a plain tuple construction (`Rng(seed ^ const)` for the three
/// `pipeline::Streams`, `Rng(seed ^ fnv1a64(name))` per pass), so the
/// constructor stays a tuple and the field stays addressable.
#[derive(Clone, Debug)]
pub struct Xorshift64(pub u64);

impl Xorshift64 {
    /// The splitmix64 seeding (wind's former `Rng::new`): the seed is
    /// finalised so seed 0 is valid, and an unlucky finaliser collision is
    /// mapped off xorshift64's absorbing all-zero state
    /// (`0x2545_F491_4F6C_DD1D`, the xorshift1024*φ constant) — without the
    /// guard a zero state would stay zero forever.
    pub fn from_seed(seed: u32) -> Self {
        let mut z = (seed as u64).wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        if z == 0 {
            z = 0x2545_F491_4F6C_DD1D;
        }
        Self(z)
    }

    /// The step: one full 64-bit xorshift64 word (the wind walk's draw).
    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// The word draw: the HIGH half of the stepped state — the worldgen
    /// contract's `next_u32`, bit-identical to the pre-refactor
    /// `worldgen::noise::Rng::next_u32`. The high half is the load-bearing
    /// choice: xorshift64's low bits are weak when the stream is sampled only
    /// every few calls (the wind walk), which skewed the extreme-tier rolls
    /// toward the calm branch before the 2026-11-04 fix.
    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    /// The `Random.Next(min, max)` draw. An empty range returns `lo` without a
    /// draw, mirroring `Random.Next` (`minValue == maxValue` → `minValue`;
    /// only `minValue > maxValue` throws) — the degenerate band widths the
    /// sub-vanilla 512 test worlds produce (`GenVars` quartet collapses a
    /// band flat) stay well-formed. The wind walk's former `next_int` is this
    /// function (all its ranges are strictly positive, so its stream is
    /// byte-identical).
    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        debug_assert!(hi >= lo, "range({lo}, {hi})");
        if hi == lo {
            return lo;
        }
        lo + (self.next_u32() % (hi - lo) as u32) as i32
    }

    /// The `Next(n) == 0` roll over the same word draw.
    pub fn chance(&mut self, n: u32) -> bool {
        self.next_u32() % n == 0
    }

    /// The fraction draw — the port's `NextDouble` for the passes that draw
    /// one (the ocean-cave walker's direction/budget/jitter chain
    /// `WorldGen.cs:76462+`, the `AddTrees` beach-palm probability
    /// `:31372-31386`, the oasis density `:21333-21348`). One convention over
    /// the xorshift stream: the next word scaled into `[0, 1)`. The vanilla
    /// `UnifiedRandom.NextDouble` has different numerics — the port's streams
    /// are already xorshift, so only the *fraction* semantics (`0.0` inclusive,
    /// `1.0` exclusive, deterministic order) carry over.
    pub fn next_f64(&mut self) -> f64 {
        f64::from(self.next_u32()) / 4_294_967_296.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pre-refactor worldgen core, verbatim — the byte-exactness reference
    /// for the unified word draw.
    fn old_next_u32(state: &mut u64) -> u32 {
        let mut x = *state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        *state = x;
        (x >> 32) as u32
    }

    /// The pre-refactor wind core, verbatim (the step is shared; `next_int`
    /// took the high half of the full word for the modulo).
    fn old_next_int(state: &mut u64, min: i32, max: i32) -> i32 {
        let mut x = *state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        *state = x;
        min + (((x >> 32) % (max - min) as u64) as i32)
    }

    /// The unified `next_u32` replays the pre-refactor worldgen word draw
    /// bit-exactly over many states and many draws.
    #[test]
    fn next_u32_matches_the_pre_refactor_worldgen_stream() {
        for &seed in &[0u64, 1, 0x0CEA, 0x5EED, u64::MAX, u64::MAX - 1, 42 ^ 0xA076_1D64_78BD_642F] {
            let mut unified = Xorshift64(seed);
            let mut old = seed;
            for _ in 0..512 {
                assert_eq!(unified.next_u32(), old_next_u32(&mut old));
            }
        }
    }

    /// The unified `range` replays the pre-refactor wind `next_int`
    /// bit-exactly (the modulo takes the same high half; all the wind call
    /// sites use strictly positive ranges).
    #[test]
    fn range_matches_the_pre_refactor_wind_next_int() {
        let windows = [
            (-800i32, 801i32),
            (900, 2701),
            (10, 31),
            (0, 4),
            (-25, 26),
            (0, 2),
            (-50, 51),
            (-100, 101),
            (0, 30),
            (7200, 28801),
            (-200, 201),
            (0, 20),
            (-400, 401),
            (-850, 851),
            (5, 11),
            (0, 3),
        ];
        for &seed in &[0u32, 1, 0xDEAD_BEEF, 0x1234_5678, u32::MAX] {
            let mut unified = Xorshift64::from_seed(seed);
            // the pre-refactor wind seeding, verbatim
            let mut z = (seed as u64).wrapping_add(0x9E37_79B9_7F4A_7C15);
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^= z >> 31;
            if z == 0 {
                z = 0x2545_F491_4F6C_DD1D;
            }
            let mut old = z;
            for &(lo, hi) in &windows {
                for _ in 0..64 {
                    assert_eq!(unified.range(lo, hi), old_next_int(&mut old, lo, hi));
                }
            }
        }
    }

    /// The splitmix seeding is byte-exact to wind's former `Rng::new` — same
    /// states, including the all-zero guard path.
    #[test]
    fn from_seed_matches_the_pre_refactor_splitmix_chain() {
        let mix = |seed: u32| -> u64 {
            let mut z = (seed as u64).wrapping_add(0x9E37_79B9_7F4A_7C15);
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^= z >> 31;
            if z == 0 {
                z = 0x2545_F491_4F6C_DD1D;
            }
            z
        };
        for seed in 0u32..4096 {
            assert_eq!(Xorshift64::from_seed(seed).0, mix(seed));
        }
    }

    /// Same state replays identically; the empty range consumes nothing
    /// (the worldgen `range` contract).
    #[test]
    fn determinism_and_the_empty_range_contract() {
        let mut a = Xorshift64(0x0CEA);
        let mut b = Xorshift64(0x0CEA);
        for _ in 0..128 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
        let mut r = Xorshift64(7);
        let before = r.next_u32();
        let mut r2 = Xorshift64(7);
        let _ = r2.next_u32();
        assert_eq!(r2.range(5, 5), 5); // no draw on the empty range
        let _ = before;
        let mut r3 = Xorshift64(7);
        let _ = r3.next_u32();
        assert_eq!(r.range(5, 5), r3.range(5, 5));
        let _ = r.range(9, 9);
        assert_eq!(r.0, r3.0); // the empty range left both states equal
    }
}
