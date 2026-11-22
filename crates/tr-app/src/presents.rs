//! W4-f35 — the seasonal present OPEN trees: `Player.OpenPresent`
//! (`Player.cs:7031-7150`, the Christmas Present 1869) +
//! `Player.OpenLegacyPresent` (`:7152-7168`, the Blue/Green/Yellow
//! Presents 599/600/601) + `Player.OpenGoodieBag` (`:9502-9585`, the
//! Halloween Goodie Bag 1774), the grant arms the inventory right-click
//! reaches through `ItemSlot.TryOpenContainer_GrantItems`
//! (`UI/ItemSlot.cs:1828-1831/:1852-1863`).
//!
//! The DROP side (the `IsChristmas`/`XmasPresentDrop`/
//! `HalloweenGoodieBagDrop` loot conditions) already runs through
//! `tr_core::loot` + `loot_db_gen` (S1-P-i52). This module is the OPEN
//! half. The seasonal item tables are verbatim: the Present's
//! hardmode-gated Snow Globe 602, the Coal 1922 / Dog Whistle 1927
//! rares, the Red Ryder 1870 + 30-60 Musket Balls 97, the six 1/150
//! rows (Candy Cane Sword/Pickaxe/Hook, Fruitcake Chakram, Hand
//! Warmer), the 1/15 five-way costume-set pick, the candy-cane block
//! tails; the Goodie Bag's two 1/150 rares (Bat Hook 1800, Unlucky
//! Yarn 1810 — the early RETURNS), the 1/4 Rotten Egg 1809 ×10-40,
//! the 1/10 five-way painting pick `Next(1846, 1851)`, then the
//! 19-way costume switch (the `QuickSpawnItem_CostumeSet`
//! two/three-piece helpers, `:9478-9489`).
//!
//! ## The RNG contract
//!
//! Same as the boss bags / crates: every draw rides `Main.rand` (the
//! wiring feeds `NamedRngs::global`), and every `QuickSpawnItem`
//! interleaves its `Prefix(-1)` natural-roll draws
//! ([`crate::boss_bags::quick_spawn`]).
//!
//! ## The recorded quirks (kept verbatim)
//!
//! - The Present's first gate is `Next(15) == 0 && Main.hardMode`
//!   (`:7034`) — the DRAW happens even in a pre-hardmode world; a hit
//!   with `hardMode` false falls through to the `else if` chain (the
//!   1/30 Coal gate draws NEXT).
//! - The Legacy Present draws ONE `Next(14)` (`:7155`): 0 + hardmode →
//!   602; `num <= 7` (INCLUDING a pre-hardmode 0) → Candy Cane Block
//!   586 ×20-49; else Green Candy Cane Block 591 ×20-49.
//! - The Goodie Bag's costume case 10 grants (1766, 1775, 1776) — the
//!   Witch set, NOT the contiguous 1766-1768 (`:9558`); case 18 grants
//!   (1851, 1852) where 1851 ALSO sits in the 1/10 painting range
//!   (`:9522`).
//!
//! Census: `docs/research/184-expert-drop-trees-census.md`.

#![allow(dead_code)] // the clicks.rs dispatch lands with the wiring commit

use crate::boss_bags::{BagGrant, quick_spawn, range};
use crate::prefix::PrefixRandom;

/// The Christmas Present 1869 (`ItemID.cs:5421`).
pub const PRESENT: u16 = 1869;
/// The legacy Blue/Green/Yellow Presents (`ItemID.cs:2881-2885`).
pub fn is_legacy_present(kind: u16) -> bool {
    matches!(kind, 599 | 600 | 601)
}
/// The Halloween Goodie Bag 1774 (`ItemID.cs:5231`).
pub const GOODIE_BAG: u16 = 1774;

/// `Player.OpenPresent(int itemType)` (`Player.cs:7031-7150`): the
/// else-if chance chain — exactly ONE arm grants (the first hit
/// wins), the final else is the candy-cane food pick.
pub fn open_present<R: PrefixRandom + ?Sized>(
    hard_mode: bool,
    rng: &mut R,
    out: &mut Vec<BagGrant>,
) {
    // `:7034-7037` — the draw ALWAYS happens; the hardmode gate only
    // decides whether a hit grants (a pre-hardmode hit falls through).
    if rng.next_max(15) == 0 && hard_mode {
        quick_spawn(rng, out, 602, 1); // Snow Globe
    } else if rng.next_max(30) == 0 {
        quick_spawn(rng, out, 1922, 1); // Coal (`:7038-7041`)
    } else if rng.next_max(400) == 0 {
        quick_spawn(rng, out, 1927, 1); // Dog Whistle (`:7042-7045`)
    } else if rng.next_max(150) == 0 {
        // `:7046-7050` — the Red Ryder + 30-60 Musket Balls.
        quick_spawn(rng, out, 1870, 1);
        quick_spawn(rng, out, 97, range(rng, 30, 61));
    } else if rng.next_max(150) == 0 {
        quick_spawn(rng, out, 1909, 1); // Candy Cane Sword (`:7051-7054`)
    } else if rng.next_max(150) == 0 {
        quick_spawn(rng, out, 1917, 1); // Candy Cane Pickaxe (`:7055-7058`)
    } else if rng.next_max(150) == 0 {
        quick_spawn(rng, out, 1915, 1); // Candy Cane Hook (`:7059-7062`)
    } else if rng.next_max(150) == 0 {
        quick_spawn(rng, out, 1918, 1); // Fruitcake Chakram (`:7063-7066`)
    } else if rng.next_max(150) == 0 {
        quick_spawn(rng, out, 1921, 1); // Hand Warmer (`:7067-7070`)
    } else if rng.next_max(300) == 0 {
        quick_spawn(rng, out, 1923, 1); // Toolbox (`:7071-7074`)
    } else if rng.next_max(40) == 0 {
        quick_spawn(rng, out, 1907, 1); // Reindeer Antlers (`:7075-7078`)
    } else if rng.next_max(10) == 0 {
        quick_spawn(rng, out, 1908, 1); // Holly (`:7079-7082`)
    } else if rng.next_max(15) == 0 {
        // `:7083-7109` — the five-way costume-set pick.
        match rng.next_max(5) {
            0 => {
                quick_spawn(rng, out, 1932, 1);
                quick_spawn(rng, out, 1933, 1);
                quick_spawn(rng, out, 1934, 1);
            }
            1 => {
                quick_spawn(rng, out, 1935, 1);
                quick_spawn(rng, out, 1936, 1);
                quick_spawn(rng, out, 1937, 1);
            }
            2 => {
                quick_spawn(rng, out, 1940, 1);
                quick_spawn(rng, out, 1941, 1);
                quick_spawn(rng, out, 1942, 1);
            }
            3 => quick_spawn(rng, out, 1938, 1),
            _ => quick_spawn(rng, out, 1939, 1),
        }
    } else if rng.next_max(7) == 0 {
        // `:7110-7126` — the three-way food pick (the sequential ifs).
        let mut num = rng.next_max(3);
        if num == 0 {
            num = 1911; // Christmas Pudding
        }
        if num == 1 {
            num = 1919; // Sugar Cookie
        }
        if num == 2 {
            num = 1920; // Gingerbread Cookie
        }
        quick_spawn(rng, out, num, 1);
    } else if rng.next_max(8) == 0 {
        quick_spawn(rng, out, 1912, range(rng, 1, 4)); // Eggnog (`:7127-7130`)
    } else if rng.next_max(9) == 0 {
        quick_spawn(rng, out, 1913, range(rng, 20, 41)); // Star Anise (`:7131-7134`)
    } else {
        // `:7135-7149` — the candy-cane block tail: 20-49 each.
        match rng.next_max(3) {
            0 => quick_spawn(rng, out, 1872, range(rng, 20, 50)), // Pine Tree Block
            1 => quick_spawn(rng, out, 586, range(rng, 20, 50)),  // Candy Cane Block
            _ => quick_spawn(rng, out, 591, range(rng, 20, 50)),  // Green Candy Cane Block
        }
    }
}

/// `Player.OpenLegacyPresent(int itemType)` (`Player.cs:7152-7168`):
/// ONE `Next(14)` — 0 + hardmode → the Snow Globe; `<= 7` (including a
/// pre-hardmode 0) → the Candy Cane Block; else the Green Candy Cane
/// Block.
pub fn open_legacy_present<R: PrefixRandom + ?Sized>(
    hard_mode: bool,
    rng: &mut R,
    out: &mut Vec<BagGrant>,
) {
    let num = rng.next_max(14); // `:7155`
    if num == 0 && hard_mode {
        quick_spawn(rng, out, 602, 1);
    } else if num <= 7 {
        quick_spawn(rng, out, 586, range(rng, 20, 50));
    } else {
        quick_spawn(rng, out, 591, range(rng, 20, 50));
    }
}

/// `Player.OpenGoodieBag(int itemType)` (`Player.cs:9502-9585`): the
/// two 1/150 rare early-returns, the 1/4 Rotten Egg, the 1/10 painting
/// pick, then the 19-way costume switch.
pub fn open_goodie_bag<R: PrefixRandom + ?Sized>(rng: &mut R, out: &mut Vec<BagGrant>) {
    // `:9505-9509` — the 1/150 Unlucky Yarn (the black-cat pet license).
    if rng.next_max(150) == 0 {
        quick_spawn(rng, out, 1810, 1);
        return;
    }
    // `:9510-9514` — the 1/150 Bat Hook.
    if rng.next_max(150) == 0 {
        quick_spawn(rng, out, 1800, 1);
        return;
    }
    // `:9515-9519` — the 1/4 Rotten Egg ×10-40.
    if rng.next_max(4) == 0 {
        quick_spawn(rng, out, 1809, range(rng, 10, 41));
        return;
    }
    // `:9520-9524` — the 1/10 five-way painting pick.
    if rng.next_max(10) == 0 {
        quick_spawn(rng, out, range(rng, 1846, 1851), 1);
        return;
    }
    // `:9525-9584` — the 19-way costume switch (the CostumeSet helpers
    // spawn each piece in argument order, `:9478-9489`).
    let pieces: &[i32] = match rng.next_max(19) {
        0 => &[1749, 1750, 1751],
        1 => &[1746, 1747, 1748],
        2 => &[1752, 1753],
        3 => &[1767, 1768, 1769],
        4 => &[1770, 1771],
        5 => &[1772, 1773],
        6 => &[1754, 1755, 1756],
        7 => &[1757, 1758, 1759],
        8 => &[1760, 1761, 1762],
        9 => &[1763, 1764, 1765],
        10 => &[1766, 1775, 1776], // `:9558` — the Witch set, NOT 1766-1768
        11 => &[1777, 1778],
        12 => &[1779, 1780, 1781],
        13 => &[1819, 1820],
        14 => &[1821, 1822, 1823],
        15 => &[1824],
        16 => &[1838, 1839, 1840],
        17 => &[1841, 1842, 1843],
        _ => &[1851, 1852], // `:9582` — 1851 also sits in the paint range
    };
    for &p in pieces {
        quick_spawn(rng, out, p, 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use tr_core::UnifiedRandom;

    /// The boss_bags ScriptRng shape (forced draws, exhausted → range
    /// minimum, every requested range recorded).
    struct ScriptRng {
        draws: VecDeque<i32>,
        ranges: Vec<i32>,
    }
    impl ScriptRng {
        fn new(draws: &[i32]) -> Self {
            Self {
                draws: VecDeque::from(draws.to_vec()),
                ranges: Vec::new(),
            }
        }
    }
    impl PrefixRandom for ScriptRng {
        fn next_max(&mut self, max: i32) -> i32 {
            self.ranges.push(max);
            let v = self.draws.pop_front().unwrap_or(0);
            assert!((0..max).contains(&v), "draw {v} outside [0, {max})");
            v
        }
    }

    fn items(out: &[BagGrant]) -> Vec<u16> {
        out.iter().map(|g| g.item).collect()
    }

    fn spawn_replay(r: &mut UnifiedRandom, want: &mut Vec<BagGrant>, item: i32, stack: i32) {
        if stack <= 0 {
            return;
        }
        let prefix = crate::prefix::roll_prefix(item as u16, -1, r);
        want.push(BagGrant {
            item: item as u16,
            stack: stack as u32,
            prefix,
        });
    }

    #[test]
    fn w4_f35_present_hardmode_zero_script_grants_the_snow_globe() {
        // The first Next(15) hits at zero + hardmode → 602, ONE draw.
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        open_present(true, &mut rng, &mut out);
        assert_eq!(items(&out), vec![602]);
        assert_eq!(rng.ranges, vec![15], "the chain short-circuits");
    }

    #[test]
    fn w4_f35_present_prehardmode_zero_falls_through_to_the_coal() {
        // `:7034` — the Next(15) DRAWS and hits, but hardMode false
        // falls to the else-if: the Next(30) hits at zero → Coal 1922.
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        open_present(false, &mut rng, &mut out);
        assert_eq!(items(&out), vec![1922]);
        assert_eq!(rng.ranges, vec![15, 30], "the hit-but-gated draw still consumed");
    }

    #[test]
    fn w4_f35_present_all_miss_lands_the_pine_tree_block_tail() {
        // Every gate misses (scripted 1s), the final Next(3)=0 → the
        // Pine Tree Block 1872 ×20 (the range minimum).
        let mut rng = ScriptRng::new(&[1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0]);
        let mut out = Vec::new();
        open_present(true, &mut rng, &mut out);
        assert_eq!(items(&out), vec![1872]);
        assert_eq!(out[0].stack, 20);
        // The 16 gates + the Next(3) pick + the range(20,50)→30.
        assert_eq!(
            rng.ranges,
            vec![15, 30, 400, 150, 150, 150, 150, 150, 150, 300, 40, 10, 15, 7, 8, 9, 3, 30]
        );
    }

    #[test]
    fn w4_f35_present_costume_arm_grants_the_three_pieces_in_order() {
        // The 1/15 costume gate hits (the 13th gate), Next(5)=0 → the
        // Mrs. Claus set 1932/1933/1934 (all vanity — no prefix draws).
        let mut rng = ScriptRng::new(&[1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0]);
        let mut out = Vec::new();
        open_present(false, &mut rng, &mut out);
        assert_eq!(items(&out), vec![1932, 1933, 1934]);
    }

    #[test]
    fn w4_f35_present_red_ryder_arm_grants_the_gun_then_the_balls() {
        // The 4th gate (1/150) hits → 1870 (a GUN — one Next(4) prefix
        // gate) then 97 ×30 (the range(30,61) minimum).
        let mut rng = ScriptRng::new(&[1, 1, 1, 0]);
        let mut out = Vec::new();
        open_present(false, &mut rng, &mut out);
        assert_eq!(items(&out), vec![1870, 97]);
        assert_eq!(out[1].stack, 30);
        assert_eq!(rng.ranges, vec![15, 30, 400, 150, 4, 31]);
    }

    #[test]
    fn w4_f35_legacy_present_one_draw_three_arms() {
        // 0 + hardmode → 602; 0 pre-hardmode → 586 (the <=7 arm);
        // 7 → 586; 8 → 591. Each is ONE Next(14) + the stack range.
        for (draw, hard, want) in [
            (0i32, true, 602u16),
            (0, false, 586),
            (7, true, 586),
            (8, false, 591),
            (13, true, 591),
        ] {
            let mut rng = ScriptRng::new(&[draw]);
            let mut out = Vec::new();
            open_legacy_present(hard, &mut rng, &mut out);
            assert_eq!(items(&out), vec![want], "draw {draw} hard {hard}");
            if want != 602 {
                assert_eq!(out[0].stack, 20, "the range(20,50) minimum");
                assert_eq!(rng.ranges, vec![14, 30]);
            } else {
                assert_eq!(rng.ranges, vec![14]);
            }
        }
        assert!(is_legacy_present(599) && is_legacy_present(600) && is_legacy_present(601));
        assert!(!is_legacy_present(1869) && !is_legacy_present(598) && !is_legacy_present(602));
    }

    #[test]
    fn w4_f35_goodie_bag_zero_script_grants_the_unlucky_yarn_and_returns() {
        // The first 1/150 hits at zero → 1810, ONE draw (the early
        // return skips every later gate).
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        open_goodie_bag(&mut rng, &mut out);
        assert_eq!(items(&out), vec![1810]);
        assert_eq!(rng.ranges, vec![150]);
    }

    #[test]
    fn w4_f35_goodie_bag_bat_hook_and_rotten_egg_arms() {
        // The second 1/150 hits → 1800 (the Bat Hook — a hook, no
        // prefix set); the 1/4 hits → 1809 ×10.
        let mut rng = ScriptRng::new(&[1, 0]);
        let mut out = Vec::new();
        open_goodie_bag(&mut rng, &mut out);
        assert_eq!(items(&out), vec![1800]);
        assert_eq!(rng.ranges, vec![150, 150]);
        let mut rng2 = ScriptRng::new(&[1, 1, 0]);
        let mut out2 = Vec::new();
        open_goodie_bag(&mut rng2, &mut out2);
        assert_eq!(items(&out2), vec![1809]);
        assert_eq!(out2[0].stack, 10);
        assert_eq!(rng2.ranges, vec![150, 150, 4, 31]);
    }

    #[test]
    fn w4_f35_goodie_bag_costume_switch_all_19_rows() {
        // The four gates miss, the Next(19) picks each row: the full
        // costume table (the case-10 Witch set + the case-18
        // Treasure Hunter pair).
        let rows: &[(i32, &[u16])] = &[
            (0, &[1749, 1750, 1751]),
            (1, &[1746, 1747, 1748]),
            (2, &[1752, 1753]),
            (3, &[1767, 1768, 1769]),
            (4, &[1770, 1771]),
            (5, &[1772, 1773]),
            (6, &[1754, 1755, 1756]),
            (7, &[1757, 1758, 1759]),
            (8, &[1760, 1761, 1762]),
            (9, &[1763, 1764, 1765]),
            (10, &[1766, 1775, 1776]),
            (11, &[1777, 1778]),
            (12, &[1779, 1780, 1781]),
            (13, &[1819, 1820]),
            (14, &[1821, 1822, 1823]),
            (15, &[1824]),
            (16, &[1838, 1839, 1840]),
            (17, &[1841, 1842, 1843]),
            (18, &[1851, 1852]),
        ];
        for &(pick, want) in rows {
            let mut rng = ScriptRng::new(&[1, 1, 1, 1, pick]);
            let mut out = Vec::new();
            open_goodie_bag(&mut rng, &mut out);
            assert_eq!(items(&out), want, "costume row {pick}");
            assert_eq!(rng.ranges, vec![150, 150, 4, 10, 19], "row {pick} census");
        }
    }

    #[test]
    fn w4_f35_goodie_bag_painting_arm_picks_the_range() {
        // The 1/10 hits, the range(1846,1851)→5 at 2 → 1848.
        let mut rng = ScriptRng::new(&[1, 1, 1, 0, 2]);
        let mut out = Vec::new();
        open_goodie_bag(&mut rng, &mut out);
        assert_eq!(items(&out), vec![1848]);
        assert_eq!(rng.ranges, vec![150, 150, 4, 10, 5]);
    }

    /// The Present's C# replay (`Player.cs:7031-7150`).
    fn present_replay(seed: i32, hard: bool) -> (Vec<BagGrant>, UnifiedRandom) {
        let mut r = UnifiedRandom::new(seed);
        let mut w = Vec::new();
        if r.next_max(15) == 0 && hard {
            spawn_replay(&mut r, &mut w, 602, 1);
        } else if r.next_max(30) == 0 {
            spawn_replay(&mut r, &mut w, 1922, 1);
        } else if r.next_max(400) == 0 {
            spawn_replay(&mut r, &mut w, 1927, 1);
        } else if r.next_max(150) == 0 {
            spawn_replay(&mut r, &mut w, 1870, 1);
            spawn_replay(&mut r, &mut w, 97, r.next_range(30, 61));
        } else if r.next_max(150) == 0 {
            spawn_replay(&mut r, &mut w, 1909, 1);
        } else if r.next_max(150) == 0 {
            spawn_replay(&mut r, &mut w, 1917, 1);
        } else if r.next_max(150) == 0 {
            spawn_replay(&mut r, &mut w, 1915, 1);
        } else if r.next_max(150) == 0 {
            spawn_replay(&mut r, &mut w, 1918, 1);
        } else if r.next_max(150) == 0 {
            spawn_replay(&mut r, &mut w, 1921, 1);
        } else if r.next_max(300) == 0 {
            spawn_replay(&mut r, &mut w, 1923, 1);
        } else if r.next_max(40) == 0 {
            spawn_replay(&mut r, &mut w, 1907, 1);
        } else if r.next_max(10) == 0 {
            spawn_replay(&mut r, &mut w, 1908, 1);
        } else if r.next_max(15) == 0 {
            match r.next_max(5) {
                0 => {
                    spawn_replay(&mut r, &mut w, 1932, 1);
                    spawn_replay(&mut r, &mut w, 1933, 1);
                    spawn_replay(&mut r, &mut w, 1934, 1);
                }
                1 => {
                    spawn_replay(&mut r, &mut w, 1935, 1);
                    spawn_replay(&mut r, &mut w, 1936, 1);
                    spawn_replay(&mut r, &mut w, 1937, 1);
                }
                2 => {
                    spawn_replay(&mut r, &mut w, 1940, 1);
                    spawn_replay(&mut r, &mut w, 1941, 1);
                    spawn_replay(&mut r, &mut w, 1942, 1);
                }
                3 => spawn_replay(&mut r, &mut w, 1938, 1),
                _ => spawn_replay(&mut r, &mut w, 1939, 1),
            }
        } else if r.next_max(7) == 0 {
            let mut num = r.next_max(3);
            if num == 0 {
                num = 1911;
            }
            if num == 1 {
                num = 1919;
            }
            if num == 2 {
                num = 1920;
            }
            spawn_replay(&mut r, &mut w, num, 1);
        } else if r.next_max(8) == 0 {
            spawn_replay(&mut r, &mut w, 1912, r.next_range(1, 4));
        } else if r.next_max(9) == 0 {
            spawn_replay(&mut r, &mut w, 1913, r.next_range(20, 41));
        } else {
            match r.next_max(3) {
                0 => spawn_replay(&mut r, &mut w, 1872, r.next_range(20, 50)),
                1 => spawn_replay(&mut r, &mut w, 586, r.next_range(20, 50)),
                _ => spawn_replay(&mut r, &mut w, 591, r.next_range(20, 50)),
            }
        }
        (w, r)
    }

    /// The Goodie Bag's C# replay (`Player.cs:9502-9585`).
    fn goodie_replay(seed: i32) -> (Vec<BagGrant>, UnifiedRandom) {
        let mut r = UnifiedRandom::new(seed);
        let mut w = Vec::new();
        if r.next_max(150) == 0 {
            spawn_replay(&mut r, &mut w, 1810, 1);
            return (w, r);
        }
        if r.next_max(150) == 0 {
            spawn_replay(&mut r, &mut w, 1800, 1);
            return (w, r);
        }
        if r.next_max(4) == 0 {
            spawn_replay(&mut r, &mut w, 1809, r.next_range(10, 41));
            return (w, r);
        }
        if r.next_max(10) == 0 {
            spawn_replay(&mut r, &mut w, r.next_range(1846, 1851), 1);
            return (w, r);
        }
        let pieces: &[i32] = match r.next_max(19) {
            0 => &[1749, 1750, 1751],
            1 => &[1746, 1747, 1748],
            2 => &[1752, 1753],
            3 => &[1767, 1768, 1769],
            4 => &[1770, 1771],
            5 => &[1772, 1773],
            6 => &[1754, 1755, 1756],
            7 => &[1757, 1758, 1759],
            8 => &[1760, 1761, 1762],
            9 => &[1763, 1764, 1765],
            10 => &[1766, 1775, 1776],
            11 => &[1777, 1778],
            12 => &[1779, 1780, 1781],
            13 => &[1819, 1820],
            14 => &[1821, 1822, 1823],
            15 => &[1824],
            16 => &[1838, 1839, 1840],
            17 => &[1841, 1842, 1843],
            _ => &[1851, 1852],
        };
        for &p in pieces {
            spawn_replay(&mut r, &mut w, p, 1);
        }
        (w, r)
    }

    #[test]
    fn w4_f35_the_present_openers_replay_the_csharp_draw_order() {
        for seed in [1i32, 7, 42, 1234, 99999, 314159] {
            for hard in [false, true] {
                let mut rng = UnifiedRandom::new(seed);
                let mut out = Vec::new();
                open_present(hard, &mut rng, &mut out);
                let (want, r2) = present_replay(seed, hard);
                assert_eq!(out, want, "present seed {seed} hard {hard}");
                assert_eq!(rng, r2, "present seed {seed} hard {hard} stream");
                // The legacy present on the same seed.
                let mut lr = UnifiedRandom::new(seed);
                let mut lout = Vec::new();
                open_legacy_present(hard, &mut lr, &mut lout);
                let mut lr2 = UnifiedRandom::new(seed);
                let mut lwant = Vec::new();
                let num = lr2.next_max(14);
                if num == 0 && hard {
                    spawn_replay(&mut lr2, &mut lwant, 602, 1);
                } else if num <= 7 {
                    spawn_replay(&mut lr2, &mut lwant, 586, lr2.next_range(20, 50));
                } else {
                    spawn_replay(&mut lr2, &mut lwant, 591, lr2.next_range(20, 50));
                }
                assert_eq!(lout, lwant, "legacy seed {seed} hard {hard}");
                assert_eq!(lr, lr2, "legacy seed {seed} hard {hard} stream");
            }
            let mut rng = UnifiedRandom::new(seed);
            let mut out = Vec::new();
            open_goodie_bag(&mut rng, &mut out);
            let (want, r2) = goodie_replay(seed);
            assert_eq!(out, want, "goodie seed {seed}");
            assert_eq!(rng, r2, "goodie seed {seed} stream");
        }
    }
}
