//! The surface/cave background STYLE rolls — `WorldGen.RandomizeBackgrounds`
//! (`WorldGen.cs:7947-8012`), `RandomizeTreeStyle` (`:8117-8190`) and
//! `RandomizeCaveBackgrounds` (`:8192-8249`), ported verbatim but **UNWIRED**
//! (W4-g71 DOC-SWEEP lane 16, code-only).
//!
//! Why unwired: the vanilla rolls run mid-`WorldGen.Reset` on the shared
//! `genRand` (`WorldGen.cs:11338-11340`), between the `WorldId` draw (`:11337`)
//! and `RandomizeMoonState` (`:11341`) / the dungeon-side + jungle-origin rolls
//! (`:11344-11374`). Inserting them into the port's tail would shift the
//! seed-finish stream and move the frozen `world_hash` pins — the same
//! RNG-shift g72 flagged for the cave rolls. On top of that, a rolled
//! (non-default) surface style needs its `Background_<id>` sheets packed, and
//! the single 8192 px atlas already packs the style-0 set to 7847 px (measured
//! with the `tools/atlas_sim.js` skyline twin): most non-default styles add
//! 336–1503 px and overflow the strict-GPU bound. Wiring needs the S1-G-13
//! background-target family (or an atlas re-budget) AND a tail-stream re-pin;
//! both are out of this lane's scope. The rolls are pure functions over a
//! caller-supplied stream, so a future worldgen lane wires them with its own
//! `pass_rng` slot and re-pins the tail in one move.
//!
//! Every function draws ONLY from its `rng` argument (the port's xorshift64
//! [`Rng`], standing in for the C# `UnifiedRandom` — the same documented
//! deviation as every other pass). No global state, no side effects: the
//! `World` writes the vanilla does (`setBG`, `Main.treeX`/`caveBackX`/…) are
//! the caller's job, kept out so the rolls stay witnessable in isolation.

// UNWIRED (module docs): the rolls are consumed only by their witnesses until
// a worldgen lane wires them into the tail with a `pass_rng` slot + re-pin.
#![allow(dead_code)]

use super::noise::Rng;

/// `WorldGen.RollRandomForestBGStyle` (`WorldGen.cs:8034-8059`), verbatim: the
/// `Next(14)` base, the 1/2 re-roll, the 0 re-roll, then the 3→31 / 5→51 /
/// 7→71..73 variant promotions. The draw ORDER is load-bearing (each guard
/// consumes its `Next` before the next guard reads).
pub(crate) fn roll_random_forest_bg_style(rng: &mut Rng) -> i32 {
    let max_value = 14;
    let mut num = rng.range(0, max_value);
    if (num == 1 || num == 2) && rng.range(0, 2) == 0 {
        num = rng.range(0, max_value);
    }
    if num == 0 {
        num = rng.range(0, max_value);
    }
    if num == 3 && rng.range(0, 3) == 0 {
        num = 31;
    }
    if num == 5 && rng.range(0, 2) == 0 {
        num = 51;
    }
    if num == 7 && rng.range(0, 4) == 0 {
        num = rng.range(71, 74);
    }
    num
}

/// `WorldGen.RandomizeBackgrounds_Corruption` (`WorldGen.cs:8014-8022`):
/// `Next(6)`, and a 5 promotes to 51/52 on a `Next(2)` coin.
pub(crate) fn randomize_backgrounds_corruption(rng: &mut Rng) -> i32 {
    let mut num = rng.range(0, 6);
    if num == 5 {
        num = if rng.range(0, 2) == 0 { 51 } else { 52 };
    }
    num
}

/// `WorldGen.RandomizeBackgrounds_Desert` (`WorldGen.cs:8024-8032`):
/// `Next(6)`, and a 5 promotes to `51 + Next(5)/2` (51/51/52/52/53).
pub(crate) fn randomize_backgrounds_desert(rng: &mut Rng) -> i32 {
    let mut num = rng.range(0, 6);
    if num == 5 {
        num = 51 + rng.range(0, 5) / 2;
    }
    num
}

/// The snow style roll — the inline block of `RandomizeBackgrounds`
/// (`WorldGen.cs:7971-8004`, mirrored in `RandomizeBackgroundBasedOnPlayer`
/// `:7856-7893`): `Next(9)`, then the 2→21/22, 3→31/32, 4→41/42 promotions,
/// each gated on a `Next(2)` coin and resolved by a second `Next(2)`.
pub(crate) fn roll_snow_bg_style(rng: &mut Rng) -> i32 {
    let mut snow = rng.range(0, 9);
    if snow == 2 && rng.range(0, 2) == 0 {
        snow = if rng.range(0, 2) == 0 { 21 } else { 22 };
    }
    if snow == 3 && rng.range(0, 2) == 0 {
        snow = if rng.range(0, 2) == 0 { 31 } else { 32 };
    }
    if snow == 4 && rng.range(0, 2) == 0 {
        snow = if rng.range(0, 2) == 0 { 41 } else { 42 };
    }
    snow
}

/// The `RandomizeBackgrounds` outcome — the eleven style values the vanilla
/// writes via `setBG(0..12)` (`WorldGen.cs:7965-8011`). The four forest styles
/// are pairwise distinct (the `while` re-rolls, `:7951-7964`); the rest are
/// single draws. `underworld` (slot 9) is included for completeness — g72 owns
/// its draw side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BackgroundStyles {
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
pub(crate) fn randomize_backgrounds(rng: &mut Rng) -> BackgroundStyles {
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
        jungle: rng.range(0, 7),
        snow: roll_snow_bg_style(rng),
        hallow: rng.range(0, 6),
        crimson: rng.range(0, 7),
        desert: randomize_backgrounds_desert(rng),
        ocean: rng.range(0, 8),
        mushroom: rng.range(0, 5),
        underworld: rng.range(0, 3),
    }
}

/// `WorldGen.RandomizeTreeStyle` (`WorldGen.cs:8117-8190`): the `Main.treeX`
/// dividers + `Main.treeStyle` tree-top variations. Returns `(treeX[0..3],
/// treeStyle[0..4])`; the small (4200) and medium (6400) arms leave the unused
/// slots at 0 (the C# never writes them). The 0→4 promotion (`Next(3) != 0`)
/// runs per active style AFTER the distinctness re-rolls.
pub(crate) fn randomize_tree_style(rng: &mut Rng, max_tiles_x: i32) -> ([i32; 3], [i32; 4]) {
    let mut tree_x = [0i32; 3];
    let mut tree_style = [0i32; 4];
    let w = max_tiles_x as f64;
    if max_tiles_x == 4200 {
        tree_x[0] = rng.range((w * 0.5 - w * 0.25) as i32, (w * 0.5 + w * 0.25) as i32);
        tree_style[0] = rng.range(0, 6);
        tree_style[1] = rng.range(0, 6);
        while tree_style[1] == tree_style[0] {
            tree_style[1] = rng.range(0, 6);
        }
        tree_x[1] = max_tiles_x;
        tree_x[2] = max_tiles_x;
        for i in 0..2 {
            if tree_style[i] == 0 && rng.range(0, 3) != 0 {
                tree_style[i] = 4;
            }
        }
        return (tree_x, tree_style);
    }
    if max_tiles_x == 6400 {
        tree_x[0] = rng.range((w * 0.334 - w * 0.2) as i32, (w * 0.334 + w * 0.2) as i32);
        tree_x[1] = rng.range((w * 0.667 - w * 0.2) as i32, (w * 0.667 + w * 0.2) as i32);
        tree_style[0] = rng.range(0, 6);
        tree_style[1] = rng.range(0, 6);
        tree_style[2] = rng.range(0, 6);
        while tree_style[1] == tree_style[0] {
            tree_style[1] = rng.range(0, 6);
        }
        while tree_style[2] == tree_style[0] || tree_style[2] == tree_style[1] {
            tree_style[2] = rng.range(0, 6);
        }
        tree_x[2] = max_tiles_x;
        for j in 0..3 {
            if tree_style[j] == 0 && rng.range(0, 3) != 0 {
                tree_style[j] = 4;
            }
        }
        return (tree_x, tree_style);
    }
    tree_x[0] = rng.range((w * 0.25 - w * 0.15) as i32, (w * 0.25 + w * 0.15) as i32);
    tree_x[1] = rng.range((w * 0.5 - w * 0.15) as i32, (w * 0.5 + w * 0.15) as i32);
    tree_x[2] = rng.range((w * 0.75 - w * 0.15) as i32, (w * 0.75 + w * 0.15) as i32);
    for i in 0..4 {
        tree_style[i] = rng.range(0, 6);
    }
    while tree_style[1] == tree_style[0] {
        tree_style[1] = rng.range(0, 6);
    }
    while tree_style[2] == tree_style[0] || tree_style[2] == tree_style[1] {
        tree_style[2] = rng.range(0, 6);
    }
    while tree_style[3] == tree_style[0]
        || tree_style[3] == tree_style[1]
        || tree_style[3] == tree_style[2]
    {
        tree_style[3] = rng.range(0, 6);
    }
    for k in 0..4 {
        if tree_style[k] == 0 && rng.range(0, 3) != 0 {
            tree_style[k] = 4;
        }
    }
    (tree_x, tree_style)
}

/// The `RandomizeCaveBackgrounds` outcome — `Main.caveBackX[0..3]`,
/// `caveBackStyle[0..4]` and the three biome back styles. g72's
/// `render/bg_styles.rs` consumes these (its `BgStylePickInput` /
/// `background_style_sheets`); the rolls are the worldgen lane's to wire.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CaveBackgrounds {
    pub cave_back_x: [i32; 3],
    pub cave_back_style: [i32; 4],
    pub ice_back_style: i32,
    pub hell_back_style: i32,
    pub jungle_back_style: i32,
}

/// `WorldGen.RandomizeCaveBackgrounds` (`WorldGen.cs:8192-8249`), verbatim:
/// the size-banded `caveBackX` dividers + distinct `caveBackStyle` rolls
/// (`Next(8)`), then the three trailing biome draws — `iceBackStyle`
/// `Next(4)` (`:8246`), `hellBackStyle` `Next(3)` (`:8247`), `jungleBackStyle`
/// `Next(2)` (`:8248`). The small/medium arms leave unused style slots at 0.
pub(crate) fn randomize_cave_backgrounds(rng: &mut Rng, max_tiles_x: i32) -> CaveBackgrounds {
    let mut cave_back_x = [0i32; 3];
    let mut cave_back_style = [0i32; 4];
    let w = max_tiles_x as f64;
    if max_tiles_x == 4200 {
        cave_back_x[0] = rng.range((w * 0.5 - w * 0.25) as i32, (w * 0.5 + w * 0.25) as i32);
        cave_back_x[1] = max_tiles_x;
        cave_back_x[2] = max_tiles_x;
        cave_back_style[0] = rng.range(0, 8);
        cave_back_style[1] = rng.range(0, 8);
        while cave_back_style[1] == cave_back_style[0] {
            cave_back_style[1] = rng.range(0, 8);
        }
    } else if max_tiles_x == 6400 {
        cave_back_x[0] = rng.range((w * 0.334 - w * 0.2) as i32, (w * 0.334 + w * 0.2) as i32);
        cave_back_x[1] = rng.range((w * 0.667 - w * 0.2) as i32, (w * 0.667 + w * 0.2) as i32);
        cave_back_x[2] = max_tiles_x;
        cave_back_style[0] = rng.range(0, 8);
        cave_back_style[1] = rng.range(0, 8);
        cave_back_style[2] = rng.range(0, 8);
        while cave_back_style[1] == cave_back_style[0] {
            cave_back_style[1] = rng.range(0, 8);
        }
        while cave_back_style[2] == cave_back_style[0] || cave_back_style[2] == cave_back_style[1] {
            cave_back_style[2] = rng.range(0, 8);
        }
    } else {
        cave_back_x[0] = rng.range((w * 0.25 - w * 0.15) as i32, (w * 0.25 + w * 0.15) as i32);
        cave_back_x[1] = rng.range((w * 0.5 - w * 0.15) as i32, (w * 0.5 + w * 0.15) as i32);
        cave_back_x[2] = rng.range((w * 0.75 - w * 0.15) as i32, (w * 0.75 + w * 0.15) as i32);
        for i in 0..4 {
            cave_back_style[i] = rng.range(0, 8);
        }
        while cave_back_style[1] == cave_back_style[0] {
            cave_back_style[1] = rng.range(0, 8);
        }
        while cave_back_style[2] == cave_back_style[0] || cave_back_style[2] == cave_back_style[1] {
            cave_back_style[2] = rng.range(0, 8);
        }
        while cave_back_style[3] == cave_back_style[0]
            || cave_back_style[3] == cave_back_style[1]
            || cave_back_style[3] == cave_back_style[2]
        {
            cave_back_style[3] = rng.range(0, 8);
        }
    }
    CaveBackgrounds {
        cave_back_x,
        cave_back_style,
        ice_back_style: rng.range(0, 4),
        hell_back_style: rng.range(0, 3),
        jungle_back_style: rng.range(0, 2),
    }
}

/// `WorldGen.IsBackgroundConsideredTheSame` (`WorldGen.cs:8061-8095`): the
/// forest re-roll equivalence — 3≡31, 5≡51, 7≡71/72/73, else exact. Used by
/// `RandomizeBackgroundBasedOnPlayer` (`:7910-7941`); ported for completeness
/// (the worldgen `RandomizeBackgrounds` uses exact `==` instead, `:7951`).
pub(crate) fn is_background_considered_the_same(old_bg: i32, new_bg: i32) -> bool {
    match old_bg {
        3 | 31 => new_bg == 3 || new_bg == 31,
        5 | 51 => new_bg == 5 || new_bg == 51,
        7 | 71 | 72 | 73 => new_bg == 7 || (71..=73).contains(&new_bg),
        _ => old_bg == new_bg,
    }
}

#[cfg(test)]
mod tests {
    //! Witnesses for the UNWIRED background style rolls (W4-g71 DOC-SWEEP
    //! lane 16). WRITTEN NOT RUN (code-only). Every expected value is derived
    //! from an independent Python reference of the port's xorshift64 stream
    //! (`Rng(seed ^ fnv1a64(name))`, high-32-bit word draw, `range` =
    //! `lo + word % (hi-lo)`) fed through the verbatim C# roll logic — NOT
    //! from running this Rust. The stream seeds match `Streams::pass_rng`.
    use super::*;
    use super::super::pipeline::fnv1a64;

    /// The per-pass stream a future wiring would use (`Streams::pass_rng`).
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
        // pass_rng("RandomizeCaveBackgrounds"), seed 42, width 8400 (else arm).
        let c = randomize_cave_backgrounds(&mut stream(42, "RandomizeCaveBackgrounds"), 8400);
        assert_eq!(c.cave_back_x, [846, 3324, 5930]);
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
        let c = randomize_cave_backgrounds(&mut stream(42, "RandomizeCaveBackgrounds"), 512);
        assert_eq!(c.cave_back_x, [156, 311, 414]);
        assert_eq!(c.cave_back_style, [1, 4, 3, 0]);
        assert_eq!(c.ice_back_style, 3);
        assert_eq!(c.hell_back_style, 2);
        assert_eq!(c.jungle_back_style, 0);
    }

    #[test]
    fn w4_g71_randomize_cave_backgrounds_small_arm() {
        // width 4200: caveBackX[1]/[2] pin to maxTilesX, two styles roll.
        let c = randomize_cave_backgrounds(&mut stream(42, "RandomizeCaveBackgrounds"), 4200);
        assert_eq!(c.cave_back_x, [1056, 4200, 4200]);
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
}
