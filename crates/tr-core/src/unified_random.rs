//! The C# `Main.rand` stream: `Terraria.Utilities.UnifiedRandom`
//! (`decompiled/Terraria/Utilities/UnifiedRandom.cs:1-159`).
//!
//! This is the .NET Framework `System.Random` algorithm — a Knuth subtractive
//! generator over a 56-entry `int` seed array, **not** xoshiro and **not** the
//! port's [`crate::rng::Xorshift64`] worldgen stream. It is kept in its own
//! module precisely so the worldgen contract streams stay untouched
//! (`research/35-rng-utils.md` task seed 1): the migration of the runtime
//! consumers onto named `UnifiedRandom` streams is a later task (S1-A-2+).
//!
//! Every method cites the C# line range it was transcribed from. The stream is
//! pinned byte-for-byte against a real run of the decompiled class by the
//! `golden` tests in this module — those literals came from
//! `dotnet run` of `decompiled/Terraria/Utilities/UnifiedRandom.cs`, so a
//! divergence here means the port, not the oracle, drifted.
//!
//! The `Vector2`/`Vector2D`/`Point`/`Rectangle`/`IntRange` extension helpers in
//! `Utils.cs:1198-1299` are intentionally not ported yet: `tr-core` has no
//! shared geometry types to hang them on, and inventing parallel ones with no
//! consumer would violate the "extend, don't duplicate" rule. The scalar
//! helpers that build on `Sample()` (`NextFloat`, `NextFloatDirection`,
//! `NextFromList`) are ported because they are pure stream draws.

/// `MBIG` (`UnifiedRandom.cs:8`).
const MBIG: i32 = i32::MAX;
/// `MSEED` (`UnifiedRandom.cs:10`).
const MSEED: i32 = 161803398;

/// The `Terraria.Utilities.UnifiedRandom` state: the ring cursor plus the
/// 56-int subtractive seed array (`UnifiedRandom.cs:14-16`).
#[derive(Clone, Debug)]
pub struct UnifiedRandom {
    /// `inext` (`UnifiedRandom.cs:14`).
    inext: u32,
    /// `SeedArray` (`UnifiedRandom.cs:16`).
    seed_array: [i32; 56],
}

impl UnifiedRandom {
    /// `new UnifiedRandom(int Seed)` (`UnifiedRandom.cs:23-26`) → `SetSeed`.
    ///
    /// There is deliberately no `Default`: the C# parameterless ctor seeds from
    /// `Environment.TickCount` (`:18-21`), which a deterministic port must
    /// never do.
    pub fn new(seed: i32) -> Self {
        let mut r = Self {
            inext: 0,
            seed_array: [0; 56],
        };
        r.set_seed(seed);
        r
    }

    /// `SetSeed` (`UnifiedRandom.cs:28-60`): the .NET Framework Knuth warm-up.
    pub fn set_seed(&mut self, seed: i32) {
        // :30-33 — wipe the array.
        for slot in self.seed_array.iter_mut() {
            *slot = 0;
        }
        // :34 — `161803398 - (Seed == int.MinValue ? int.MaxValue : Math.Abs(Seed))`.
        // The `if` keeps `i32::MIN.abs()` (which would overflow) unreachable.
        // The whole class runs in C# `unchecked` context, so every arithmetic
        // step below is wrapping (`wrapping_*`); extreme seeds drive the array
        // out of the non-negative range and the C# silently overflows.
        let mut num = MSEED.wrapping_sub(if seed == i32::MIN { MBIG } else { seed.abs() });
        // :35
        self.seed_array[55] = num;
        // :36
        let mut num2 = 1i32;
        // :37-47
        for j in 1..55 {
            let num3 = (21 * j % 55) as usize;
            self.seed_array[num3] = num2;
            num2 = num.wrapping_sub(num2);
            if num2 < 0 {
                num2 = num2.wrapping_add(MBIG);
            }
            num = self.seed_array[num3];
        }
        // :48-58
        for _k in 1..5 {
            for l in 1..56 {
                self.seed_array[l] =
                    self.seed_array[l].wrapping_sub(self.seed_array[1 + (l + 30) % 55]);
                if self.seed_array[l] < 0 {
                    self.seed_array[l] = self.seed_array[l].wrapping_add(MBIG);
                }
            }
        }
        // :59
        self.inext = 0;
    }

    /// `InternalSample` (`UnifiedRandom.cs:67-88`): one subtractive step.
    fn internal_sample(&mut self) -> i32 {
        // :69-73
        let mut num = self.inext + 1;
        if num > 55 {
            num = 1;
        }
        // :74-78
        let mut num2 = num + 21;
        if num2 > 55 {
            num2 -= 55;
        }
        // :79-85
        let (idx, idx2) = (num as usize, num2 as usize);
        let mut num3 = self.seed_array[idx].wrapping_sub(self.seed_array[idx2]);
        if num3 == MBIG {
            num3 -= 1;
        }
        num3 = num3.wrapping_add((num3 >> 31) & 0x7FFF_FFFF);
        self.seed_array[idx] = num3;
        // :86-87
        self.inext = num;
        num3
    }

    /// `Sample` (`UnifiedRandom.cs:62-65`): `InternalSample() * 2^-31`.
    fn sample(&mut self) -> f64 {
        self.internal_sample() as f64 * 4.656612875245797e-10
    }

    /// `Peek` (`UnifiedRandom.cs:90-103`): the next raw difference, no advance.
    pub fn peek(&self) -> i32 {
        // :92-101
        let mut num = self.inext + 1;
        if num > 55 {
            num = 1;
        }
        let mut num2 = num + 21;
        if num2 > 55 {
            num2 -= 55;
        }
        // :102 — the raw difference, WITHOUT the `InternalSample` correction.
        self.seed_array[num as usize].wrapping_sub(self.seed_array[num2 as usize])
    }

    /// `Next()` (`UnifiedRandom.cs:105-108`).
    pub fn next(&mut self) -> i32 {
        self.internal_sample()
    }

    /// `GetSampleForLargeRange` (`UnifiedRandom.cs:110-118`).
    fn get_sample_for_large_range(&mut self) -> f64 {
        // :112
        let mut num = self.internal_sample();
        // :113-116 — the second draw decides the sign.
        if self.internal_sample() % 2 == 0 {
            num = -num;
        }
        // :117
        (num as f64 + 2147483646.0) / 4294967293.0
    }

    /// `Next(int minValue, int maxValue)` (`UnifiedRandom.cs:120-132`).
    ///
    /// Faithful detail: the C# draws from `Sample()` even for an empty range
    /// (`min == max` → `Sample() * 0 + min`), so this stream advances there.
    pub fn next_range(&mut self, min: i32, max: i32) -> i32 {
        // :122-125
        assert!(min <= max, "minValue must be less than maxValue");
        // :126
        let num = max as i64 - min as i64;
        // :127-130
        if num <= i32::MAX as i64 {
            (self.sample() * num as f64) as i32 + min
        } else {
            // :131
            ((self.get_sample_for_large_range() * num as f64) as i64 + min as i64) as i32
        }
    }

    /// `Next(int maxValue)` (`UnifiedRandom.cs:134-141`).
    pub fn next_max(&mut self, max: i32) -> i32 {
        // :136-139
        assert!(max >= 0, "maxValue must be positive.");
        // :140
        (self.sample() * max as f64) as i32
    }

    /// `NextDouble()` (`UnifiedRandom.cs:143-146`).
    pub fn next_double(&mut self) -> f64 {
        self.sample()
    }

    /// `NextBytes(byte[])` (`UnifiedRandom.cs:148-158`).
    pub fn next_bytes(&mut self, buffer: &mut [u8]) {
        // :154-157
        for slot in buffer.iter_mut() {
            *slot = (self.internal_sample() % 256) as u8;
        }
    }

    /// `Utils.NextFloat(this UnifiedRandom)` (`Utils.cs:1198-1201`).
    pub fn next_float(&mut self) -> f32 {
        self.next_double() as f32
    }

    /// `Utils.NextFloatDirection` (`Utils.cs:1203-1206`).
    pub fn next_float_direction(&mut self) -> f32 {
        self.next_double() as f32 * 2.0 - 1.0
    }

    /// `Utils.NextFromList<T>` (`Utils.cs:1213-1216`): `objs[Next(len)]`.
    pub fn next_from_slice<T: Copy>(&mut self, objs: &[T]) -> T {
        objs[self.next_max(objs.len() as i32) as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The first 64 `Next()` (= `InternalSample()`) values for a seed, captured
    /// from the decompiled C# class under .NET 10.
    fn assert_next_stream(seed: i32, expected: &[i32; 64]) {
        let mut r = UnifiedRandom::new(seed);
        for (i, &want) in expected.iter().enumerate() {
            let got = r.next();
            assert_eq!(got, want, "seed {seed} draw {i}");
        }
    }

    /// Seeds 0 / 1 / 42 / -1 are the audit's required vector
    /// (`research/35-rng-utils.md` task seed 2); `i32::MIN`/`i32::MAX` pin the
    /// `Seed == int.MinValue` branch (`UnifiedRandom.cs:34`) and the
    /// `Math.Abs(seed)` magnitude path.
    const GOLDEN_0: [i32; 64] = [
        1559595546, 1755192844, 1649316166, 1198642031, 442452829, 1200195957, 1945678308,
        949569752, 2099272109, 587775847, 626863973, 1003550677, 1358625013, 1008269081,
        2109153755, 65212616, 1851925803, 2137491580, 1454235444, 675580731, 1754296375,
        1821177336, 2130093701, 70062080, 1503113964, 1130186590, 2005789796, 1476653312,
        1174277203, 174182291, 401846963, 973512717, 638171722, 2122881600, 1380182313, 1638451829,
        65271247, 818200948, 736891500, 2056119311, 1084756724, 1537539262, 255459778, 587232589,
        1947978014, 1706746116, 724046315, 981848395, 315304373, 475269784, 880625662, 1543454120,
        1331075398, 1047903413, 418573418, 1885901857, 1772582790, 1579254086, 1843011714,
        1459749886, 1341889808, 469024996, 1922776196, 1925089818,
    ];
    const GOLDEN_1: [i32; 64] = [
        534011718, 237820880, 1002897798, 1657007234, 1412011072, 929393559, 760389092, 2026928803,
        217468053, 1379662799, 61497087, 532638534, 687431273, 2125508764, 1464848243, 1406361028,
        607156385, 1321446349, 1512368656, 1507096884, 2039633683, 200550235, 345545550, 820276643,
        1713579631, 363927940, 1704637349, 659767626, 1767440095, 1903678842, 1194650126,
        1539174560, 1501726364, 29236707, 2130037697, 1731519511, 1844137836, 134224868,
        1104652174, 1127983490, 585887566, 2134161410, 1484135457, 830250430, 932725007,
        1450422331, 1454666067, 1015490648, 525100016, 759101347, 1130118952, 363842328, 744291659,
        1485403625, 848563345, 333461483, 2039758977, 182621155, 2090911250, 1048083132,
        1372239857, 100621466, 259488708, 461272858,
    ];
    const GOLDEN_42: [i32; 64] = [
        1434747710, 302596119, 269548474, 1122627734, 361709742, 563913476, 1555655117, 1101493307,
        372913049, 1634773126, 503774878, 552593494, 1085775344, 687695533, 818126015, 558871098,
        1111217775, 75846583, 1748346701, 1239520805, 853561429, 326832151, 193515357, 1514302668,
        1752737390, 1159577995, 94805257, 1527192852, 317314903, 1946077082, 1487325104,
        1108990006, 400244687, 89141974, 661853736, 1252327179, 1763223987, 8492999, 1150454279,
        1729120475, 1607088558, 825865714, 320230768, 56563676, 109541013, 1678565381, 1345304841,
        247339374, 536786791, 1658777195, 621925607, 1391882737, 308478477, 95559494, 1298281176,
        1107915559, 109080762, 902729453, 1517373991, 1349615394, 469108219, 28462265, 784178404,
        574319614,
    ];
    /// `Math.Abs(-1) == 1` (and the `int.MinValue` special case) make these two
    /// seeds replay `GOLDEN_1`.
    const GOLDEN_MINUS_ONE: [i32; 64] = GOLDEN_1;
    const GOLDEN_I32_MAX: [i32; 64] = [
        1559595546, 1755192844, 1649316172, 1198642031, 442452829, 1200195955, 1945678308,
        949569752, 2099272109, 587775835, 626863973, 1003550677, 1358624999, 1008269081,
        2109153755, 65212616, 1851925783, 2137491580, 1454235444, 675580701, 1754296375,
        1821177336, 2130093701, 70062072, 1503113964, 1130186590, 2005789778, 1476653312,
        1174277203, 174182293, 401846963, 973512717, 638171722, 2122881612, 1380182313, 1638451829,
        65271241, 818200948, 736891500, 2056119313, 1084756754, 1537539262, 255459778, 587232601,
        1947978014, 1706746116, 724046329, 981848415, 315304373, 475269784, 880625680, 1543454120,
        1331075398, 1047903443, 418573420, 1885901857, 1772582790, 1579254100, 1843011714,
        1459749886, 1341889824, 469024996, 1922776196, 1925089816,
    ];
    /// `int.MinValue` takes the `Seed == int.MinValue → int.MaxValue` branch
    /// (`UnifiedRandom.cs:34`), so it replays `GOLDEN_I32_MAX`.
    const GOLDEN_I32_MIN: [i32; 64] = GOLDEN_I32_MAX;

    #[test]
    fn golden_next_stream_for_the_audit_seed_vector() {
        assert_next_stream(0, &GOLDEN_0);
        assert_next_stream(1, &GOLDEN_1);
        assert_next_stream(42, &GOLDEN_42);
        assert_next_stream(-1, &GOLDEN_MINUS_ONE);
        assert_next_stream(i32::MAX, &GOLDEN_I32_MAX);
        assert_next_stream(i32::MIN, &GOLDEN_I32_MIN);
    }

    /// `NextDouble()` is `Sample()`, compared by exact bit pattern so the
    /// `4.656612875245797e-10` scale cannot drift.
    #[test]
    fn golden_next_double_bits() {
        let expected: [u64; 16] = [
            0x3fe56120cfaac242,
            0x3fc2094017241280,
            0x3fc010fbba2021f7,
            0x3fe0ba7c25a174f8,
            0x3fc58f40ae2b1e81,
            0x3fd0ce5182219ca3,
            0x3fe72e5a736e5cb5,
            0x3fe069dd0ee0d3ba,
            0x3fc63a33992c7467,
            0x3fe85c2a21b0b854,
            0x3fce06fe9e3c0dfd,
            0x3fd077f42b20efe8,
            0x3fe02de77c205bcf,
            0x3fd47eb356a8fd67,
            0x3fd861ce5fb0c39d,
            0x3fd0a7d91d214fb2,
        ];
        let mut r = UnifiedRandom::new(42);
        for (i, &want) in expected.iter().enumerate() {
            let got = r.next_double().to_bits();
            assert_eq!(got, want, "seed 42 NextDouble draw {i}");
        }
    }

    /// `Next(min, max)` truncation (`UnifiedRandom.cs:129`).
    #[test]
    fn golden_next_range() {
        let expected: [i32; 32] = [
            16, 11, 11, 15, 11, 12, 17, 15, 11, 17, 12, 12, 15, 13, 13, 12, 15, 10, 18, 15, 13, 11,
            10, 17, 18, 15, 10, 17, 11, 19, 16, 15,
        ];
        let mut r = UnifiedRandom::new(42);
        for (i, &want) in expected.iter().enumerate() {
            assert_eq!(r.next_range(10, 20), want, "seed 42 Next(10,20) draw {i}");
        }
    }

    /// `Next(max)` (`UnifiedRandom.cs:140`).
    #[test]
    fn golden_next_max() {
        let expected: [i32; 32] = [
            66, 14, 12, 52, 16, 26, 72, 51, 17, 76, 23, 25, 50, 32, 38, 26, 51, 3, 81, 57, 39, 15,
            9, 70, 81, 53, 4, 71, 14, 90, 69, 51,
        ];
        let mut r = UnifiedRandom::new(42);
        for (i, &want) in expected.iter().enumerate() {
            assert_eq!(r.next_max(100), want, "seed 42 Next(100) draw {i}");
        }
    }

    /// `NextBytes` (`UnifiedRandom.cs:154-157`).
    #[test]
    fn golden_next_bytes() {
        let expected = [
            62u8, 23, 186, 150, 174, 4, 205, 59, 153, 134, 158, 86, 240, 173, 191, 58,
        ];
        let mut r = UnifiedRandom::new(42);
        let mut buf = [0u8; 16];
        r.next_bytes(&mut buf);
        assert_eq!(buf, expected);
    }

    /// `Peek` returns the next raw difference without advancing
    /// (`UnifiedRandom.cs:90-103`).
    #[test]
    fn golden_peek() {
        let mut r = UnifiedRandom::new(42);
        assert_eq!(r.peek(), -712735937);
        assert_eq!(r.next(), 1434747710);
    }

    /// `Utils.NextFromList` (`Utils.cs:1213-1216`) draws `Next(len)`.
    #[test]
    fn next_from_slice_matches_next_max_indexing() {
        let objs = [10, 20, 30, 40, 50];
        let mut a = UnifiedRandom::new(42);
        let mut b = UnifiedRandom::new(42);
        for _ in 0..64 {
            let idx = b.next_max(objs.len() as i32) as usize;
            assert_eq!(a.next_from_slice(&objs), objs[idx]);
        }
    }

    /// The `max - min > int.MaxValue` branch (`UnifiedRandom.cs:131`) that goes
    /// through `GetSampleForLargeRange` (`:110-118`).
    #[test]
    fn golden_next_large_range() {
        let expected: [i32; 16] = [
            1434747709,
            -269548476,
            -361709744,
            1555655116,
            -372913051,
            -503774880,
            1085775343,
            -818126017,
            1111217774,
            1748346700,
            853561428,
            -193515359,
            1752737389,
            -94805259,
            -317314905,
            -1487325106,
        ];
        let mut r = UnifiedRandom::new(42);
        for (i, &want) in expected.iter().enumerate() {
            assert_eq!(
                r.next_range(i32::MIN, i32::MAX),
                want,
                "seed 42 Next(int.MinValue, int.MaxValue) draw {i}"
            );
        }
    }

    /// `Utils.NextFloat` = `(float)NextDouble()` (`Utils.cs:1200`), pinned by
    /// f32 bit pattern so the C# float cast rounding is exact.
    #[test]
    fn golden_next_float_bits() {
        let expected: [u32; 16] = [
            0x3f2b0906, 0x3e104a01, 0x3e0087de, 0x3f05d3e1, 0x3e2c7a05, 0x3e86728c, 0x3f3972d4,
            0x3f034ee8, 0x3e31d19d, 0x3f42e151, 0x3e7037f5, 0x3e83bfa1, 0x3f016f3c, 0x3ea3f59b,
            0x3ec30e73, 0x3e853ec9,
        ];
        let mut r = UnifiedRandom::new(42);
        for (i, &want) in expected.iter().enumerate() {
            assert_eq!(r.next_float().to_bits(), want, "seed 42 NextFloat draw {i}");
        }
    }

    /// `Utils.NextFloatDirection` = `(float)NextDouble() * 2f - 1f`
    /// (`Utils.cs:1205`).
    #[test]
    fn golden_next_float_direction_bits() {
        let expected: [u32; 16] = [
            0x3eac2418, 0xbf37db00, 0xbf3fbc11, 0x3d3a7c20, 0xbf29c2fe, 0xbef31ae8, 0x3ee5cb50,
            0x3cd3ba00, 0xbf271732, 0x3f05c2a2, 0xbf07e406, 0xbef880be, 0x3c379e00, 0xbeb814ca,
            0xbe73c634, 0xbef5826e,
        ];
        let mut r = UnifiedRandom::new(42);
        for (i, &want) in expected.iter().enumerate() {
            assert_eq!(
                r.next_float_direction().to_bits(),
                want,
                "seed 42 NextFloatDirection draw {i}"
            );
        }
    }

    /// Faithful C# detail: `Next(min, max)` draws from `Sample()` even when the
    /// range is empty (`min == max` → `Sample() * 0 + min`,
    /// `UnifiedRandom.cs:126-129`), so the next `Next()` is the *second* raw
    /// draw, not the first.
    #[test]
    fn golden_empty_range_still_draws() {
        let expected: [(i32, i32); 8] = [
            (5, 302596119),
            (5, 1122627734),
            (5, 563913476),
            (5, 1101493307),
            (5, 1634773126),
            (5, 552593494),
            (5, 687695533),
            (5, 558871098),
        ];
        let mut r = UnifiedRandom::new(42);
        for (i, &(empty, next)) in expected.iter().enumerate() {
            assert_eq!(r.next_range(5, 5), empty, "empty-range draw {i}");
            assert_eq!(r.next(), next, "draw-after-empty {i}");
        }
    }
}
