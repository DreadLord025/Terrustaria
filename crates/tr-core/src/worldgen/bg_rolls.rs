//! The surface background STYLE rolls — `WorldGen.RandomizeBackgrounds`
//! (`WorldGen.cs:7947-8012`) and `RandomizeTreeStyle` (`:8117-8190`), ported
//! verbatim. **WIRED** into `step_world_tail` by the W4-g71 wiring phase (this
//! branch) on isolated `pass_rng` slots.
//!
//! **KEEP-ONE (research/92 §2.1-FINAL):** `RandomizeCaveBackgrounds`
//! (`:8192-8249`) lives in `worldgen/cave_bg.rs` — g72's canonical driver,
//! adopted verbatim (tip `4a75715e`). The former duplicate here is DROPPED:
//! draw-order-equivalent, but it zeroed the un-rolled style slots instead of
//! carrying the previous `Main.caveBackStyle` values (the C# arm-A/B keep
//! them, `WorldGen.cs:8195-8223`), returned `[i32; 3]` instead of the C#
//! `new int[4]` shape, and g72's landed draw/persist/net trio consumes ITS
//! outcome type. `boot_reroll_backgrounds` below calls THAT driver.
//!
//! Stream contract: every function draws ONLY from its `rng` argument through
//! the [`RollRng`] seam, so the SAME code serves both callers —
//! * the worldgen tail draws on the per-pass `pass_rng("Randomize…")` slots
//!   (the port's documented per-pass-stream deviation, `pipeline.rs`), and
//! * the boot re-roll (`Main.cs:6594-6603`) draws on the bare-`Main.rand`
//!   stand-in ([`crate::unified_random::UnifiedRandom`]).
//!
//! No global state, no side effects: the `World` writes the vanilla does
//! (`setBG`, `Main.treeX`/…) are the caller's job, kept out so the rolls stay
//! witnessable in isolation. The rolls write only style/divider fields —
//! never tiles — so wiring them moves no `world_hash` tile pin (the tail is
//! the last `streams.rng` consumer and these rolls ride isolated `pass_rng`
//! slots); the cave block's shared-stream draw moves the moon/hell VALUES
//! only (stale-by-design, the `a == b` determinism witnesses stay green).

use super::noise::Rng;

/// The draw seam the rolls run on — one `Next(min, max)` per call. Implemented
/// for the worldgen xorshift64 [`Rng`] (the tail's `pass_rng` slots) and for
/// [`crate::unified_random::UnifiedRandom`] (the boot re-roll's bare-`Main.rand`
/// stand-in). Each keeps its OWN range semantics: the xorshift `range` returns
/// `lo` without a draw on an empty band, the Knuth `next_range` draws even then
/// (`UnifiedRandom.cs:120-132`) — the rolls are faithful to whichever stream
/// the caller hands them.
pub trait RollRng {
    /// `Random.Next(minValue, maxValue)` — the only draw the rolls use.
    fn roll_range(&mut self, lo: i32, hi: i32) -> i32;
}

impl RollRng for Rng {
    fn roll_range(&mut self, lo: i32, hi: i32) -> i32 {
        self.range(lo, hi)
    }
}

impl RollRng for crate::unified_random::UnifiedRandom {
    fn roll_range(&mut self, lo: i32, hi: i32) -> i32 {
        self.next_range(lo, hi)
    }
}

/// `WorldGen.RollRandomForestBGStyle` (`WorldGen.cs:8034-8059`), verbatim: the
/// `Next(14)` base, the 1/2 re-roll, the 0 re-roll, then the 3→31 / 5→51 /
/// 7→71..73 variant promotions. The draw ORDER is load-bearing (each guard
/// consumes its `Next` before the next guard reads).
pub fn roll_random_forest_bg_style(rng: &mut impl RollRng) -> i32 {
    let max_value = 14;
    let mut num = rng.roll_range(0, max_value);
    if (num == 1 || num == 2) && rng.roll_range(0, 2) == 0 {
        num = rng.roll_range(0, max_value);
    }
    if num == 0 {
        num = rng.roll_range(0, max_value);
    }
    if num == 3 && rng.roll_range(0, 3) == 0 {
        num = 31;
    }
    if num == 5 && rng.roll_range(0, 2) == 0 {
        num = 51;
    }
    if num == 7 && rng.roll_range(0, 4) == 0 {
        num = rng.roll_range(71, 74);
    }
    num
}

/// `WorldGen.RandomizeBackgrounds_Corruption` (`WorldGen.cs:8014-8022`):
/// `Next(6)`, and a 5 promotes to 51/52 on a `Next(2)` coin.
pub fn randomize_backgrounds_corruption(rng: &mut impl RollRng) -> i32 {
    let mut num = rng.roll_range(0, 6);
    if num == 5 {
        num = if rng.roll_range(0, 2) == 0 { 51 } else { 52 };
    }
    num
}

/// `WorldGen.RandomizeBackgrounds_Desert` (`WorldGen.cs:8024-8032`):
/// `Next(6)`, and a 5 promotes to `51 + Next(5)/2` (51/51/52/52/53).
pub fn randomize_backgrounds_desert(rng: &mut impl RollRng) -> i32 {
    let mut num = rng.roll_range(0, 6);
    if num == 5 {
        num = 51 + rng.roll_range(0, 5) / 2;
    }
    num
}

/// The snow style roll — the inline block of `RandomizeBackgrounds`
/// (`WorldGen.cs:7971-8004`, mirrored in `RandomizeBackgroundBasedOnPlayer`
/// `:7856-7893`): `Next(9)`, then the 2→21/22, 3→31/32, 4→41/42 promotions,
/// each gated on a `Next(2)` coin and resolved by a second `Next(2)`.
pub fn roll_snow_bg_style(rng: &mut impl RollRng) -> i32 {
    let mut snow = rng.roll_range(0, 9);
    if snow == 2 && rng.roll_range(0, 2) == 0 {
        snow = if rng.roll_range(0, 2) == 0 { 21 } else { 22 };
    }
    if snow == 3 && rng.roll_range(0, 2) == 0 {
        snow = if rng.roll_range(0, 2) == 0 { 31 } else { 32 };
    }
    if snow == 4 && rng.roll_range(0, 2) == 0 {
        snow = if rng.roll_range(0, 2) == 0 { 41 } else { 42 };
    }
    snow
}

/// The `RandomizeBackgrounds` outcome — the eleven style values the vanilla
/// writes via `setBG(0..12)` (`WorldGen.cs:7965-8011`). The four forest styles
/// are pairwise distinct (the `while` re-rolls, `:7951-7964`); the rest are
/// single draws. `underworld` (slot 9) is included for completeness — g72 owns
/// its draw side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackgroundStyles {
    /// `treeBG1..4` — `setBG(0/10/11/12)`, pairwise distinct.
    pub forest: [i32; 4],
    pub corrupt: i32,
    pub jungle: i32,
    pub snow: i32,
    pub hallow: i32,
    pub crimson: i32,
    pub desert: i32,
    pub ocean: i32,
    pub mushroom: i32,
    pub underworld: i32,
}

/// `WorldGen.RandomizeBackgrounds` (`WorldGen.cs:7947-8012`), verbatim draw
/// order: the four distinct forest rolls, then corrupt / jungle / snow /
/// hallow / crimson / desert / ocean / mushroom / underworld. The `setBG`
/// writes the C# interleaves are pure table expansions (no RNG), so the port
/// returns the styles and leaves the writes to the caller.
pub fn randomize_backgrounds(rng: &mut impl RollRng) -> BackgroundStyles {
    let tree_bg1 = roll_random_forest_bg_style(rng);
    let mut tree_bg2 = roll_random_forest_bg_style(rng);
    while tree_bg2 == tree_bg1 {
        tree_bg2 = roll_random_forest_bg_style(rng);
    }
    let mut tree_bg3 = roll_random_forest_bg_style(rng);
    while tree_bg3 == tree_bg1 || tree_bg3 == tree_bg2 {
        tree_bg3 = roll_random_forest_bg_style(rng);
    }
    let mut tree_bg4 = roll_random_forest_bg_style(rng);
    while tree_bg4 == tree_bg1 || tree_bg4 == tree_bg2 || tree_bg4 == tree_bg3 {
        tree_bg4 = roll_random_forest_bg_style(rng);
    }
    BackgroundStyles {
        forest: [tree_bg1, tree_bg2, tree_bg3, tree_bg4],
        corrupt: randomize_backgrounds_corruption(rng),
        jungle: rng.roll_range(0, 7),
        snow: roll_snow_bg_style(rng),
        hallow: rng.roll_range(0, 6),
        crimson: rng.roll_range(0, 7),
        desert: randomize_backgrounds_desert(rng),
        ocean: rng.roll_range(0, 8),
        mushroom: rng.roll_range(0, 5),
        underworld: rng.roll_range(0, 3),
    }
}

/// `WorldGen.RandomizeTreeStyle` (`WorldGen.cs:8117-8190`): the `Main.treeX`
/// dividers + `Main.treeStyle` tree-top variations. Returns `(treeX[0..3],
/// treeStyle[0..4])`; the small (4200) and medium (6400) arms leave the unused
/// slots at 0 (the C# never writes them). The 0→4 promotion (`Next(3) != 0`)
/// runs per active style AFTER the distinctness re-rolls.
pub fn randomize_tree_style(rng: &mut impl RollRng, max_tiles_x: i32) -> ([i32; 3], [i32; 4]) {
    let mut tree_x = [0i32; 3];
    let mut tree_style = [0i32; 4];
    let w = max_tiles_x as f64;
    if max_tiles_x == 4200 {
        tree_x[0] = rng.roll_range((w * 0.5 - w * 0.25) as i32, (w * 0.5 + w * 0.25) as i32);
        tree_style[0] = rng.roll_range(0, 6);
        tree_style[1] = rng.roll_range(0, 6);
        while tree_style[1] == tree_style[0] {
            tree_style[1] = rng.roll_range(0, 6);
        }
        tree_x[1] = max_tiles_x;
        tree_x[2] = max_tiles_x;
        for i in 0..2 {
            if tree_style[i] == 0 && rng.roll_range(0, 3) != 0 {
                tree_style[i] = 4;
            }
        }
        return (tree_x, tree_style);
    }
    if max_tiles_x == 6400 {
        tree_x[0] = rng.roll_range((w * 0.334 - w * 0.2) as i32, (w * 0.334 + w * 0.2) as i32);
        tree_x[1] = rng.roll_range((w * 0.667 - w * 0.2) as i32, (w * 0.667 + w * 0.2) as i32);
        tree_style[0] = rng.roll_range(0, 6);
        tree_style[1] = rng.roll_range(0, 6);
        tree_style[2] = rng.roll_range(0, 6);
        while tree_style[1] == tree_style[0] {
            tree_style[1] = rng.roll_range(0, 6);
        }
        while tree_style[2] == tree_style[0] || tree_style[2] == tree_style[1] {
            tree_style[2] = rng.roll_range(0, 6);
        }
        tree_x[2] = max_tiles_x;
        for j in 0..3 {
            if tree_style[j] == 0 && rng.roll_range(0, 3) != 0 {
                tree_style[j] = 4;
            }
        }
        return (tree_x, tree_style);
    }
    tree_x[0] = rng.roll_range((w * 0.25 - w * 0.15) as i32, (w * 0.25 + w * 0.15) as i32);
    tree_x[1] = rng.roll_range((w * 0.5 - w * 0.15) as i32, (w * 0.5 + w * 0.15) as i32);
    tree_x[2] = rng.roll_range((w * 0.75 - w * 0.15) as i32, (w * 0.75 + w * 0.15) as i32);
    for i in 0..4 {
        tree_style[i] = rng.roll_range(0, 6);
    }
    while tree_style[1] == tree_style[0] {
        tree_style[1] = rng.roll_range(0, 6);
    }
    while tree_style[2] == tree_style[0] || tree_style[2] == tree_style[1] {
        tree_style[2] = rng.roll_range(0, 6);
    }
    while tree_style[3] == tree_style[0]
        || tree_style[3] == tree_style[1]
        || tree_style[3] == tree_style[2]
    {
        tree_style[3] = rng.roll_range(0, 6);
    }
    for k in 0..4 {
        if tree_style[k] == 0 && rng.roll_range(0, 3) != 0 {
            tree_style[k] = 4;
        }
    }
    (tree_x, tree_style)
}

// The `RandomizeCaveBackgrounds` driver + its `CaveBackgrounds` outcome live
// in `worldgen/cave_bg.rs` (g72's canonical copy — the keep-one adjudication,
// research/92 §2.1-FINAL). The former duplicate here is DROPPED.

/// `WorldGen.IsBackgroundConsideredTheSame` (`WorldGen.cs:8061-8095`): the
/// forest re-roll equivalence — 3≡31, 5≡51, 7≡71/72/73, else exact. Used by
/// `RandomizeBackgroundBasedOnPlayer` (`:7910-7941`); ported for completeness
/// (the worldgen `RandomizeBackgrounds` uses exact `==` instead, `:7951`).
pub fn is_background_considered_the_same(old_bg: i32, new_bg: i32) -> bool {
    match old_bg {
        3 | 31 => new_bg == 3 || new_bg == 31,
        5 | 51 => new_bg == 5 || new_bg == 51,
        7 | 71 | 72 | 73 => new_bg == 7 || (71..=73).contains(&new_bg),
        _ => old_bg == new_bg,
    }
}

/// The boot re-roll — `Main.Initialize`'s `WorldGen.RandomizeBackgrounds(Main.rand)`
/// + the two `treeBGSet1[0] == 173` retries + `RandomizeCaveBackgrounds()`
/// (`Main.cs:6594-6603`), applied to a `World` through the caller's stream.
///
/// **Seam contract (W4-g71 wiring phase):** the C# runs this ONCE per process at
/// boot, BEFORE any world loads (`Main.Initialize` is the XNA `Game.Initialize`
/// override, `Main.cs:6447`); `LoadWorldFlags` then overwrites every field it
/// writes with the file's persisted values (`WorldFile.cs:2050-2067`,
/// `:2211-2233`). So the boot roll drives the **menu** background only — a
/// loaded world keeps its saved styles. The port's faithful placement is
/// therefore the menu-world seam, NOT the world-load path: this fn is the
/// canonical roll-and-apply the menu caller invokes on the bare-`Main.rand`
/// stand-in ([`crate::unified_random::UnifiedRandom`], the same `RollRng` the
/// worldgen tail uses through its xorshift `pass_rng` slots). The cave leg
/// calls the CANONICAL `cave_bg::randomize_cave_backgrounds` driver (the
/// keep-one, research/92 §2.1-FINAL) on the same stream — `UnifiedRandom`
/// implements both seams (`RollRng` here, `CaveRoll` there); the previous-
/// style carry passes the world's current array (the C# boot carries
/// `Main.caveBackStyle`; a fresh `World::new` starts at zeros, and g72's
/// `App::new` boot witness pins the width-0 dead-store arm).
///
/// The 173 retry (`Main.cs:6595-6602`): `SetForestBGSet(6)` sets `treeSet[0]=173`
/// (`WorldGen.cs:7691`), so a forest style 6 re-rolls the whole background set,
/// up to two extra times. The port reads the rolled `forest_bg_styles[0]` and
/// re-rolls while it is 6 (the `treeBGSet1[0] == 173` condition), matching the
/// C#'s two `if` guards (at most three total rolls).
pub fn boot_reroll_backgrounds(
    world: &mut crate::world::World,
    rng: &mut crate::unified_random::UnifiedRandom,
) {
    let width = world.width;
    let mut styles = randomize_backgrounds(rng);
    // The two 173 retries (`Main.cs:6595-6602`): style 6 → treeSet[0]=173.
    for _ in 0..2 {
        if styles.forest[0] != 6 {
            break;
        }
        styles = randomize_backgrounds(rng);
    }
    world.set_surface_bg_style(0, styles.forest[0]);
    world.set_surface_bg_style(10, styles.forest[1]);
    world.set_surface_bg_style(11, styles.forest[2]);
    world.set_surface_bg_style(12, styles.forest[3]);
    world.set_surface_bg_style(1, styles.corrupt);
    world.set_surface_bg_style(2, styles.jungle);
    world.set_surface_bg_style(3, styles.snow);
    world.set_surface_bg_style(4, styles.hallow);
    world.set_surface_bg_style(5, styles.crimson);
    world.set_surface_bg_style(6, styles.desert);
    world.set_surface_bg_style(7, styles.ocean);
    world.set_surface_bg_style(8, styles.mushroom);
    world.set_surface_bg_style(9, styles.underworld);
    // `RandomizeCaveBackgrounds()` (`Main.cs:6603`) — the cave arrays + the
    // ice/hell/jungle back styles, on the same stream, through the CANONICAL
    // `cave_bg` driver (keep-one). The previous-style carry passes the world's
    // current array — the C# keeps un-rolled entries at their previous value.
    let cave =
        super::cave_bg::randomize_cave_backgrounds(rng, width, world.cave_back_style);
    world.apply_cave_backgrounds(&cave);
}

#[cfg(test)]
mod tests {
    //! Witnesses for the background style rolls (W4-g71). WRITTEN NOT RUN
    //! (code-only). Every expected value is derived from an independent Python
    //! reference of the port's xorshift64 stream (`Rng(seed ^ fnv1a64(name))`,
    //! high-32-bit word draw, `range` = `lo + word % (hi-lo)`) fed through the
    //! verbatim C# roll logic — NOT from running this Rust. The stream seeds
    //! match `Streams::pass_rng` — the SAME slots the wired tail draws on.
    use super::*;
    use super::super::pipeline::fnv1a64;

    /// The per-pass stream the wired tail draws on (`Streams::pass_rng`).
    fn stream(seed: u64, name: &str) -> Rng {
        Rng(seed ^ fnv1a64(name))
    }

    #[test]
    fn w4_g71_randomize_backgrounds_seed42_matches_the_reference() {
        // pass_rng("RandomizeBackgrounds"), seed 42. Reference draw order:
        // four distinct forest rolls, corrupt, jungle, snow, hallow, crimson,
        // desert, ocean, mushroom, underworld.
        let s = randomize_backgrounds(&mut stream(42, "RandomizeBackgrounds"));
        assert_eq!(s.forest, [2, 7, 51, 11]);
        assert_eq!(s.corrupt, 51);
        assert_eq!(s.jungle, 5);
        assert_eq!(s.snow, 5);
        assert_eq!(s.hallow, 4);
        assert_eq!(s.crimson, 4);
        assert_eq!(s.desert, 4);
        assert_eq!(s.ocean, 2);
        assert_eq!(s.mushroom, 2);
        assert_eq!(s.underworld, 2);
        // The four forest styles are pairwise distinct (the while re-rolls).
        for i in 0..4 {
            for j in (i + 1)..4 {
                assert_ne!(s.forest[i], s.forest[j], "forest {i} vs {j}");
            }
        }
    }

    #[test]
    fn w4_g71_randomize_backgrounds_is_deterministic() {
        let a = randomize_backgrounds(&mut stream(42, "RandomizeBackgrounds"));
        let b = randomize_backgrounds(&mut stream(42, "RandomizeBackgrounds"));
        assert_eq!(a, b);
        // A different seed rolls a different set (the stream is seed-keyed).
        let c = randomize_backgrounds(&mut stream(43, "RandomizeBackgrounds"));
        assert_ne!(a, c);
    }

    #[test]
    fn w4_g71_randomize_tree_style_seed42_large_matches_the_reference() {
        // pass_rng("RandomizeTreeStyle"), seed 42, width 8400 (the else arm).
        let (tree_x, tree_style) = randomize_tree_style(&mut stream(42, "RandomizeTreeStyle"), 8400);
        assert_eq!(tree_x, [1546, 5165, 6282]);
        assert_eq!(tree_style, [2, 3, 0, 5]);
        // The dividers sit inside their ±15% bands around the quartiles.
        let w = 8400.0;
        assert!((tree_x[0] as f64) >= w * 0.25 - w * 0.15 && (tree_x[0] as f64) < w * 0.25 + w * 0.15);
        assert!((tree_x[1] as f64) >= w * 0.5 - w * 0.15 && (tree_x[1] as f64) < w * 0.5 + w * 0.15);
        assert!((tree_x[2] as f64) >= w * 0.75 - w * 0.15 && (tree_x[2] as f64) < w * 0.75 + w * 0.15);
    }

    #[test]
    fn w4_g71_randomize_tree_style_small_arm_pins_two_dividers() {
        // width 4200: treeX[1]/[2] pin to maxTilesX, only two styles roll.
        let (tree_x, tree_style) = randomize_tree_style(&mut stream(42, "RandomizeTreeStyle"), 4200);
        assert_eq!(tree_x, [2596, 4200, 4200]);
        assert_eq!(tree_style[0], 5);
        assert_eq!(tree_style[1], 4);
        // The unused slots stay 0 (the C# never writes them).
        assert_eq!(tree_style[2], 0);
        assert_eq!(tree_style[3], 0);
    }

    #[test]
    fn w4_g71_randomize_cave_backgrounds_seed42_large_matches_the_reference() {
        // The canonical cave driver (cave_bg.rs, g72 keep-one) on the former
        // pass_rng("RandomizeCaveBackgrounds") stream, seed 42, width 8400
        // (else arm). Same draw order (bands → base styles → rejections →
        // ice/hell/jungle), so the reference values carry; the outcome is now
        // the [i32; 4] shape ([3] keeps its new int[4] zero, Main.cs:66696).
        let c = super::super::cave_bg::randomize_cave_backgrounds(
            &mut stream(42, "RandomizeCaveBackgrounds"),
            8400,
            [0; 4],
        );
        assert_eq!(c.cave_back_x, [846, 3324, 5930, 0]);
        assert_eq!(c.cave_back_style, [1, 4, 3, 0]);
        assert_eq!(c.ice_back_style, 3);
        assert_eq!(c.hell_back_style, 2);
        assert_eq!(c.jungle_back_style, 0);
        // The four cave styles are pairwise distinct.
        for i in 0..4 {
            for j in (i + 1)..4 {
                assert_ne!(c.cave_back_style[i], c.cave_back_style[j]);
            }
        }
    }

    #[test]
    fn w4_g71_randomize_cave_backgrounds_sub_vanilla_else_arm() {
        // width 512 (the test worlds) takes the same else arm as 8400 — the
        // dividers scale with width, the style draws are identical (same stream).
        let c = super::super::cave_bg::randomize_cave_backgrounds(
            &mut stream(42, "RandomizeCaveBackgrounds"),
            512,
            [0; 4],
        );
        assert_eq!(c.cave_back_x, [156, 311, 414, 0]);
        assert_eq!(c.cave_back_style, [1, 4, 3, 0]);
        assert_eq!(c.ice_back_style, 3);
        assert_eq!(c.hell_back_style, 2);
        assert_eq!(c.jungle_back_style, 0);
    }

    #[test]
    fn w4_g71_randomize_cave_backgrounds_small_arm() {
        // width 4200: caveBackX[1]/[2] pin to maxTilesX, two styles roll. The
        // un-rolled slots KEEP the passed-in previous values (the C# arm never
        // writes them — the canonical driver's carry, g72 keep-one); the
        // previous [0; 4] here reproduces the old witness's zeros.
        let c = super::super::cave_bg::randomize_cave_backgrounds(
            &mut stream(42, "RandomizeCaveBackgrounds"),
            4200,
            [0; 4],
        );
        assert_eq!(c.cave_back_x, [1056, 4200, 4200, 0]);
        assert_eq!(c.cave_back_style[0], 0);
        assert_eq!(c.cave_back_style[1], 2);
        assert_eq!(c.cave_back_style[2], 0);
        assert_eq!(c.cave_back_style[3], 0);
        assert_eq!(c.ice_back_style, 1);
        assert_eq!(c.hell_back_style, 2);
        assert_eq!(c.jungle_back_style, 1);
    }

    #[test]
    fn w4_g71_forest_roll_promotions_and_bounds() {
        // Every roll lands in the legal style set (the SetForestBGSet arms +
        // the 31/51/71/72/73 promotions). Sweep a batch of seeds.
        let legal = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 31, 51, 71, 72, 73];
        for seed in 0..64u64 {
            let mut r = stream(seed, "RandomizeBackgrounds");
            for _ in 0..32 {
                let s = roll_random_forest_bg_style(&mut r);
                assert!(legal.contains(&s), "seed {seed} rolled illegal forest style {s}");
            }
        }
    }

    #[test]
    fn w4_g71_corruption_desert_snow_promotion_bounds() {
        for seed in 0..64u64 {
            let mut r = stream(seed, "RandomizeBackgrounds");
            for _ in 0..32 {
                let c = randomize_backgrounds_corruption(&mut r);
                assert!((0..=5).contains(&c) || c == 51 || c == 52, "corrupt {c}");
                let d = randomize_backgrounds_desert(&mut r);
                assert!((0..=5).contains(&d) || (51..=53).contains(&d), "desert {d}");
                let s = roll_snow_bg_style(&mut r);
                assert!(
                    (0..=8).contains(&s) || [21, 22, 31, 32, 41, 42].contains(&s),
                    "snow {s}"
                );
            }
        }
    }

    #[test]
    fn w4_g71_is_background_considered_the_same_equivalence_classes() {
        // WorldGen.cs:8061-8095: 3≡31, 5≡51, 7≡71/72/73, else exact.
        assert!(is_background_considered_the_same(3, 31));
        assert!(is_background_considered_the_same(31, 3));
        assert!(is_background_considered_the_same(3, 3));
        assert!(!is_background_considered_the_same(3, 5));
        assert!(is_background_considered_the_same(5, 51));
        assert!(is_background_considered_the_same(51, 5));
        assert!(!is_background_considered_the_same(5, 3));
        for a in [7, 71, 72, 73] {
            for b in [7, 71, 72, 73] {
                assert!(is_background_considered_the_same(a, b), "{a} vs {b}");
            }
        }
        assert!(!is_background_considered_the_same(7, 74));
        assert!(!is_background_considered_the_same(7, 5));
        // The default arm is exact equality.
        assert!(is_background_considered_the_same(11, 11));
        assert!(!is_background_considered_the_same(11, 12));
        assert!(!is_background_considered_the_same(0, 1));
    }

    // ---- W4-g71 wiring phase: the boot re-roll seam (Main.cs:6594-6603) ----

    #[test]
    fn w4_g71_boot_reroll_seed42_matches_the_unified_random_reference() {
        // The boot roll runs on the bare-`Main.rand` stand-in (UnifiedRandom,
        // seed 42). Expectations from an independent Python port of
        // UnifiedRandom.cs (golden Next() vector verified) fed through the
        // verbatim C# roll logic — NOT from running this Rust. 512-wide world.
        let mut w = crate::world::World::new("boot".into(), 512, 512);
        let mut rng = crate::unified_random::UnifiedRandom::new(42);
        boot_reroll_backgrounds(&mut w, &mut rng);
        assert_eq!(w.forest_bg_styles, [9, 71, 10, 73]);
        assert_eq!(w.corrupt_bg_style, 1);
        assert_eq!(w.jungle_bg_style, 1);
        assert_eq!(w.snow_bg_style, 41);
        assert_eq!(w.hallow_bg_style, 1);
        assert_eq!(w.crimson_bg_style, 3);
        assert_eq!(w.desert_bg_style, 0);
        assert_eq!(w.ocean_bg_style, 6);
        assert_eq!(w.mushroom_bg_style, 2);
        assert_eq!(w.underworld_bg_style, 1);
        assert_eq!(w.cave_back_x, [74, 192, 414, 0]);
        assert_eq!(w.cave_back_style, [6, 4, 0, 5]);
        assert_eq!(w.ice_back_style, 0);
        assert_eq!(w.hell_back_style, 2);
        assert_eq!(w.jungle_back_style, 1);
    }

    #[test]
    fn w4_g71_boot_reroll_is_deterministic_and_seed_keyed() {
        let mut a = crate::world::World::new("boot".into(), 512, 512);
        let mut b = crate::world::World::new("boot".into(), 512, 512);
        boot_reroll_backgrounds(&mut a, &mut crate::unified_random::UnifiedRandom::new(42));
        boot_reroll_backgrounds(&mut b, &mut crate::unified_random::UnifiedRandom::new(42));
        assert_eq!(a.forest_bg_styles, b.forest_bg_styles);
        assert_eq!(a.cave_back_style, b.cave_back_style);
        assert_eq!(a.hell_back_style, b.hell_back_style);
        // A different seed rolls a different set.
        let mut c = crate::world::World::new("boot".into(), 512, 512);
        boot_reroll_backgrounds(&mut c, &mut crate::unified_random::UnifiedRandom::new(43));
        assert_eq!(c.forest_bg_styles, [11, 10, 8, 4]);
        assert_eq!(c.cave_back_x, [179, 326, 388, 0]);
        assert_ne!(a.forest_bg_styles, c.forest_bg_styles);
    }

    #[test]
    fn w4_g71_boot_reroll_small_world_pins_two_cave_dividers() {
        // width 4200 (the default world): caveBackX[1]/[2] pin to maxTilesX,
        // only two cave styles roll. Same seed-42 stream as the 512 case up to
        // the cave block (the surface rolls are width-independent).
        let mut w = crate::world::World::new("boot".into(), 4200, 1200);
        boot_reroll_backgrounds(&mut w, &mut crate::unified_random::UnifiedRandom::new(42));
        assert_eq!(w.forest_bg_styles, [9, 71, 10, 73]);
        assert_eq!(w.cave_back_x, [1369, 4200, 4200, 0]);
        assert_eq!(w.cave_back_style[0], 0);
        assert_eq!(w.cave_back_style[1], 5);
        assert_eq!(w.cave_back_style[2], 0);
        assert_eq!(w.cave_back_style[3], 0);
        assert_eq!(w.ice_back_style, 3);
        assert_eq!(w.hell_back_style, 1);
        assert_eq!(w.jungle_back_style, 0);
    }

    #[test]
    fn w4_g71_boot_reroll_173_retry_rerolls_style_six() {
        // `Main.cs:6595-6602`: a forest style 6 (treeSet[0]=173, WorldGen.cs:7691)
        // re-rolls the whole set, up to two extra times. Seed 11's FIRST roll
        // lands forest[0]=6 (Python reference); the retry moves it to 3.
        let mut probe = crate::unified_random::UnifiedRandom::new(11);
        let first = randomize_backgrounds(&mut probe);
        assert_eq!(first.forest[0], 6, "seed 11 first roll is the 173 style");
        let mut w = crate::world::World::new("boot".into(), 512, 512);
        boot_reroll_backgrounds(&mut w, &mut crate::unified_random::UnifiedRandom::new(11));
        assert_eq!(w.forest_bg_styles, [3, 11, 12, 10], "the retry re-rolled off 6");
        // Bounds: every boot roll lands legal styles across a seed sweep.
        for seed in 0..64i32 {
            let mut w = crate::world::World::new("boot".into(), 512, 512);
            boot_reroll_backgrounds(&mut w, &mut crate::unified_random::UnifiedRandom::new(seed));
            assert!(w.hell_back_style < 3);
            assert!(w.ice_back_style < 4);
            assert!(w.jungle_back_style < 2);
            for i in 0..4 {
                for j in (i + 1)..4 {
                    assert_ne!(w.cave_back_style[i], w.cave_back_style[j]);
                }
            }
        }
    }
}
