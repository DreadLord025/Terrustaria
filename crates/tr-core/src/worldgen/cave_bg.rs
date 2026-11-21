//! `WorldGen.RandomizeCaveBackgrounds` (`WorldGen.cs:8192-8249`) — the cavern
//! background band boundaries (`Main.caveBackX[0..3]`), the per-band styles
//! (`Main.caveBackStyle[0..4]`, pairwise distinct inside the rolled arm) and
//! the ice/hell/jungle variant tail. ONE function, TWO call sites:
//!
//! * the worldgen tail (`WorldGen.cs:11339`, between `RandomizeTreeStyle`
//!   `:11338` and `RandomizeBackgrounds` `:11340`, BEFORE `RandomizeMoonState`
//!   `:11341`'s `Next(9)`) — the port draws it in `step_world_tail` on the
//!   shared xorshift stream (`pipeline.rs`), ahead of the moon draw;
//! * the boot roll (`Main.cs:6603`, `Initialize_AlmostEverything`) — runs ONCE
//!   per process after content load (the splash path `:52572`, dedServ
//!   `:6499`), NOT per world load; at that point no world file is loaded yet,
//!   so `Main.maxTilesX` is still its static 0 (`Main.cs:1011` — the first
//!   writes are `autoCreate` `:4886-4896` / the dedServ size prompt `:5301`,
//!   both after Initialize, and `LoadWorldHeader` `WorldFile.cs:1976`). The
//!   boot roll therefore ALWAYS takes the else arm with `maxTilesX == 0`.
//!   The port fires it in `App::new` (research/64 §2 correction).
//!
//! Stream note (research/64 §3): `WorldGen.genRand => Main.rand`
//! (`WorldGen.cs:4445`) is the C# Knuth `UnifiedRandom`; the port's worldgen
//! contract streams are xorshift64, so the gen-side call draws on
//! `streams.rng` (the established M5-BIOME / M5-ORES re-pin pattern — the
//! `world_hash` pins move by design), while the boot call draws on the exact
//! `UnifiedRandom` stream (the `NamedRngs::global` bare-`Main.rand` stand-in).
//!
//! The draw list, arm bounds and rejection-loop order follow research/64 §1
//! verbatim. Pure helpers + one generic driver: no RNG type is named here, so
//! both streams share the exact draw order.

/// One `Next(maxValue)` draw (`maxValue = 8` for the styles, `WorldGen.cs:8194`).
pub trait CaveRoll {
    fn roll(&mut self, max: i32) -> i32;
    fn roll_range(&mut self, lo: i32, hi: i32) -> i32;
}

impl CaveRoll for crate::rng::Xorshift64 {
    fn roll(&mut self, max: i32) -> i32 {
        // The port's `Next(n)` convention: the high-half word modulo n
        // (`rng.rs` — the same convention every other worldgen draw uses).
        (self.next_u32() % max as u32) as i32
    }
    fn roll_range(&mut self, lo: i32, hi: i32) -> i32 {
        self.range(lo, hi)
    }
}

impl CaveRoll for crate::unified_random::UnifiedRandom {
    fn roll(&mut self, max: i32) -> i32 {
        self.next_max(max)
    }
    fn roll_range(&mut self, lo: i32, hi: i32) -> i32 {
        self.next_range(lo, hi)
    }
}

/// The rolled state — `Main`'s statics, not yet stored on the world.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CaveBackgrounds {
    /// `Main.caveBackX[0..4]` — only `[0..3]` are rolled/persisted; `[3]`
    /// keeps its C# initial 0 (`new int[4]`, `Main.cs:66696`).
    pub cave_back_x: [i32; 4],
    /// `Main.caveBackStyle[0..4]` — entries past the arm's count keep their
    /// previous value in C#; the driver starts from the passed-in array.
    pub cave_back_style: [i32; 4],
    pub ice_back_style: u8,
    pub hell_back_style: u8,
    pub jungle_back_style: u8,
}

/// The width-arm bounds (`WorldGen.cs:8197-8228`), computed in f64 then
/// truncated exactly like the C# `(int)` casts — arm A `(1050, 3150)`, arm B
/// `(857, 3417)` / `(2988, 5548)` (research/64 §1). The third tuple element
/// is how many bands DRAW (A: 1, B: 2, else: 3 — the trailing boundaries are
/// direct `maxTilesX` assignments, `:8198-8199`/`:8211`); the count also sets
/// how many styles roll (A: 2, B: 3, else: 4).
pub fn width_arm(width: i32) -> ([(i32, i32); 3], usize, usize) {
    let w = width as f64;
    let band = |frac: f64, span: f64| ((w * frac - w * span) as i32, (w * frac + w * span) as i32);
    if width == 4200 {
        // :8197 — the single centre band; :8198-8199 assign maxTilesX twice.
        ([(band(0.5, 0.25), (width, width), (width, width)], 2, 1))
    } else if width == 6400 {
        // :8209-8210 — the two third-points ± 0.2; :8211 assigns maxTilesX.
        ([(band(0.334, 0.2), band(0.667, 0.2), (width, width)], 3, 2))
    } else {
        // :8226-8228 — the three quarter-points ± 0.15.
        ([(band(0.25, 0.15), band(0.5, 0.15), band(0.75, 0.15)], 4, 3))
    }
}

/// `RandomizeCaveBackgrounds` proper — every draw in C# order: the arm's
/// `caveBackX` bands (the trailing direct assignments draw nothing), the base
/// styles in index order, the distinctness rejection loops in C# order
/// (style[1] vs [0]; style[2] vs [0],[1]; style[3] vs [0],[1],[2]), then the
/// common tail `iceBackStyle = Next(4)` / `hellBackStyle = Next(3)` /
/// `jungleBackStyle = Next(2)` (`:8246-8248`). `styles` carries the PREVIOUS
/// `Main.caveBackStyle` — entries the arm does not roll keep their value.
pub fn randomize_cave_backgrounds<R: CaveRoll>(
    rng: &mut R,
    width: i32,
    styles: [i32; 4],
) -> CaveBackgrounds {
    let (bands, count, drawn) = width_arm(width);
    let mut out = CaveBackgrounds {
        cave_back_x: [0; 4],
        cave_back_style: styles,
        ice_back_style: 0,
        hell_back_style: 0,
        jungle_back_style: 0,
    };
    for (i, &(lo, hi)) in bands.iter().enumerate() {
        // Only the arm's DRAWN bands roll; the trailing ones are the direct
        // `maxTilesX` assignments (`:8198-8199`/`:8211`). The boot roll's
        // width-0 arm draws all three (0, 0) bands — the C# `Next(0, 0)`
        // still draws one Sample() on the Knuth stream
        // (UnifiedRandom.cs:126-130); xorshift's range() draws nothing on an
        // empty band (rng.rs contract) — the documented stream difference,
        // each faithful to its own generator.
        out.cave_back_x[i] = if i < drawn { rng.roll_range(lo, hi) } else { lo };
    }
    for slot in out.cave_back_style.iter_mut().take(count) {
        *slot = rng.roll(8);
    }
    // :8202-8205 / :8215-8218 / :8233-8236 — style[1] vs [0].
    if count >= 2 {
        while out.cave_back_style[1] == out.cave_back_style[0] {
            out.cave_back_style[1] = rng.roll(8);
        }
    }
    // :8219-8222 / :8237-8240 — style[2] vs [0],[1].
    if count >= 3 {
        while out.cave_back_style[2] == out.cave_back_style[0]
            || out.cave_back_style[2] == out.cave_back_style[1]
        {
            out.cave_back_style[2] = rng.roll(8);
        }
    }
    // :8241-8244 — style[3] vs [0],[1],[2].
    if count >= 4 {
        while out.cave_back_style[3] == out.cave_back_style[0]
            || out.cave_back_style[3] == out.cave_back_style[1]
            || out.cave_back_style[3] == out.cave_back_style[2]
        {
            out.cave_back_style[3] = rng.roll(8);
        }
    }
    out.ice_back_style = rng.roll(4) as u8; // :8246
    out.hell_back_style = rng.roll(3) as u8; // :8247
    out.jungle_back_style = rng.roll(2) as u8; // :8248
    out
}

impl crate::world::World {
    /// Store a roll onto the world's fields (the `Main` statics' port home).
    pub fn apply_cave_backgrounds(&mut self, roll: &CaveBackgrounds) {
        self.cave_back_x = roll.cave_back_x;
        self.cave_back_style = roll.cave_back_style;
        self.ice_back_style = roll.ice_back_style;
        self.hell_back_style = roll.hell_back_style;
        self.jungle_back_style = roll.jungle_back_style;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Xorshift64;
    use crate::unified_random::UnifiedRandom;

    /// research/64 §1: the arm bounds are the C# f64-truncated constants.
    #[test]
    fn w4_g72_width_arm_bounds_match_the_csharp_casts() {
        let (a, n, d) = width_arm(4200);
        assert_eq!((a[0], n, d), ((1050, 3150), 2, 1));
        assert_eq!((a[1], a[2]), ((4200, 4200), (4200, 4200)));
        let (b, n, d) = width_arm(6400);
        assert_eq!((b[0], b[1], n, d), ((857, 3417), (2988, 5548), 3, 2));
        assert_eq!(b[2], (6400, 6400));
        let (c, n, d) = width_arm(8400);
        assert_eq!((c[0], c[1], c[2], n, d), ((840, 3360), (2940, 5460), (5040, 7560), 4, 3));
        // The boot roll's width-0 arm (Main.cs:1011 static default).
        let (z, n, d) = width_arm(0);
        assert_eq!((z[0], z[1], z[2], n, d), ((0, 0), (0, 0), (0, 0), 4, 3));
    }

    /// The rolled styles are pairwise distinct inside the arm and every
    /// value is in range; the un-rolled entries keep their previous value.
    #[test]
    fn w4_g72_styles_are_distinct_and_ranges_hold() {
        for seed in [1u64, 42, 0xDEAD_BEEF, u64::MAX] {
            for width in [4200, 6400, 8400, 512] {
                let mut rng = Xorshift64(seed);
                let prev = [7, 7, 7, 7];
                let r = randomize_cave_backgrounds(&mut rng, width, prev);
                let count = if width == 4200 { 2 } else if width == 6400 { 3 } else { 4 };
                for i in 0..count {
                    assert!((0..8).contains(&r.cave_back_style[i]));
                    for j in 0..i {
                        assert_ne!(r.cave_back_style[i], r.cave_back_style[j]);
                    }
                }
                for i in count..4 {
                    assert_eq!(r.cave_back_style[i], prev[i]);
                }
                assert!(r.ice_back_style < 4 && r.hell_back_style < 3 && r.jungle_back_style < 2);
                assert_eq!(r.cave_back_x[3], 0); // never rolled (Main.cs:66696)
            }
        }
    }

    /// Arm A/B assign the trailing boundaries without a draw; arm C rolls
    /// all three inside the ±0.15 bands.
    #[test]
    fn w4_g72_boundaries_follow_the_arm() {
        let mut rng = Xorshift64(42);
        let a = randomize_cave_backgrounds(&mut rng, 4200, [0; 4]);
        assert!((1050..3150).contains(&a.cave_back_x[0]));
        assert_eq!((a.cave_back_x[1], a.cave_back_x[2]), (4200, 4200));
        let b = randomize_cave_backgrounds(&mut rng, 6400, a.cave_back_style);
        assert!((857..3417).contains(&b.cave_back_x[0]));
        assert!((2988..5548).contains(&b.cave_back_x[1]));
        assert_eq!(b.cave_back_x[2], 6400);
        let c = randomize_cave_backgrounds(&mut rng, 512, b.cave_back_style);
        // 512*0.25 = 128 ∓ 512*0.15 = 76.8 → (51, 204); 256 ∓ 76.8 → (179, 332);
        // 384 ∓ 76.8 → (307, 460) — the C# `(int)` truncation, exclusive hi.
        assert!((51..204).contains(&c.cave_back_x[0]));
        assert!((179..332).contains(&c.cave_back_x[1]));
        assert!((307..460).contains(&c.cave_back_x[2]));
    }

    /// Same stream state replays the same roll on BOTH generators (the
    /// driver is stream-agnostic; each generator keeps its own semantics).
    #[test]
    fn w4_g72_both_streams_are_deterministic() {
        let x1 = randomize_cave_backgrounds(&mut Xorshift64(7), 8400, [0; 4]);
        let x2 = randomize_cave_backgrounds(&mut Xorshift64(7), 8400, [0; 4]);
        assert_eq!(x1, x2);
        let u1 = randomize_cave_backgrounds(&mut UnifiedRandom::new(7), 0, [0; 4]);
        let u2 = randomize_cave_backgrounds(&mut UnifiedRandom::new(7), 0, [0; 4]);
        assert_eq!(u1, u2);
        // The boot arm rolls four styles on width 0 and zeroes the bands.
        assert_eq!(u1.cave_back_x, [0; 4]);
        for i in 0..4 {
            for j in 0..i {
                assert_ne!(u1.cave_back_style[i], u1.cave_back_style[j]);
            }
        }
    }

    /// The Knuth empty-range draw semantics: `Next(0, 0)` still advances the
    /// stream (UnifiedRandom.cs:126-130), so the boot roll consumes draws
    /// for its three zero bands — the next style draw differs from a
    /// no-band-draw replay.
    #[test]
    fn w4_g72_boot_zero_bands_still_draw_on_knuth() {
        let mut a = UnifiedRandom::new(99);
        let r = randomize_cave_backgrounds(&mut a, 0, [0; 4]);
        let mut b = UnifiedRandom::new(99);
        for _ in 0..3 {
            assert_eq!(b.next_range(0, 0), 0); // the three (0,0) band draws
        }
        assert_eq!(r.cave_back_style[0], b.next_max(8));
    }
}
