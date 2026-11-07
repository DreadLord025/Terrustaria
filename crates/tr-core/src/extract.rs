//! The Extractinator — `Terraria.GameContent.ExtractinatorHelper`
//! (`ExtractinatorHelper.cs:5-525`), the `ItemID.Sets.ExtractinatorMode` map
//! (`ItemID.cs:1321`) and the `ItemTrader.ChlorophyteExtractinator` table
//! (`ItemTrader.cs:82-108`).
//!
//! The output roll is NOT an `IItemDropRule` chain (vanilla itself rolls the
//! plain `Main.rand` sequence here), so this module ports the `else if` chain
//! verbatim instead of expressing it through [`crate::loot`]: the branch order
//! is the roll order and must never be re-derived from a table.
//!
//! All rolls draw on the injected [`DropRng`] (`Main.rand`), and each draw is
//! spent only when its branch's own gate fires — the stack adders, the
//! hardmode ore re-roll and the `num /= 3` of the fossil mode are part of the
//! same stream order as the C#.

use crate::loot::DropRng;

/// `ExtractinatorHelper.RollExtractinatorDrop` (`ExtractinatorHelper.cs:5-525`).
///
/// `extractionMode` is `ItemID.Sets.ExtractinatorMode[item.type]` (-1 for a
/// non-extractable input — the only caller of the mode -1 arm is the wired
/// path, `Wiring.cs:2554-2584`, which is out of this task's scope), and
/// `extractinator_block_type` is the tile kind (219 / 642). Returns
/// `(item_type, stack)`; `item_type = -1` means "drop nothing" (the C# `out`
/// defaults, `:70-71`, and the caller's `itemType > 0` gate, `Player.cs:42075`).
pub fn roll_extractinator_drop(
    extraction_mode: i32,
    extractinator_block_type: u16,
    hard_mode: bool,
    rng: &mut dyn DropRng,
) -> (i32, i32) {
    // The C# locals (`:7-17`) — kept under their original names so the citation
    // `ExtractinatorHelper.cs:NN` still lines up with the port.
    let mut num = 5000; // amber mosquito: 1-in-num
    let mut num2 = 25; // gems: 1-in-num2
    let mut num3 = 50; // amber: 1-in-num3
    let mut num4 = -1; // fossil ore: 1-in-num4
    let mut num5 = -1; // junk/bait table (mode 2)
    let mut num6 = -1; // moss table (mode 3)
    let mut num7 = 1; // coins: 1-in-num7
    let mut num8 = -1; // dirt fallback (mode 4)
    let mut num9 = -1; // seeds: 1-in-num9
    let mut num10 = -1; // sand fallback (mode 6)
    let mut num11 = -1; // honey fallback (mode 5)
    match extraction_mode {
        -1 => return (-1, 1),
        1 => {
            // Desert Fossil (`:24-29`) — `num /= 3` is C# integer division
            // (5000 / 3 = 1666, the "x3 rate" of atlas section 20), not 1667.
            num /= 3;
            num2 *= 2;
            num3 = 20;
            num4 = 10;
        }
        2 => {
            // Old Shoe / Fishing Seaweed / Tin Can (`:30-37`).
            num = -1;
            num2 = -1;
            num3 = -1;
            num4 = -1;
            num5 = 1;
            num7 = -1;
        }
        3 => {
            // The mosses (`:38-46`).
            num = -1;
            num2 = -1;
            num3 = -1;
            num4 = -1;
            num5 = -1;
            num7 = -1;
            num6 = 1;
        }
        4 => {
            // Poop Block (`:47-54`).
            num = -1;
            num2 = -1;
            num3 = -1;
            num7 = -1;
            num9 = 50;
            num8 = 1;
        }
        5 => {
            // Hive (`:55-61`).
            num = -1;
            num2 = -1;
            num3 = -1;
            num7 = -1;
            num11 = 1;
        }
        6 => {
            // Shell Pile Block / Obsidian (`:62-68`).
            num = -1;
            num2 = -1;
            num3 = -1;
            num7 = -1;
            num10 = 1;
        }
        _ => {}
    }
    let mut item_type = -1i32;
    let mut stack = 1i32;
    if num4 != -1 && rng.next_range(0, num4) == 0 {
        // Fossil Ore (`:72-87`).
        item_type = 3380;
        if rng.next_range(0, 5) == 0 {
            stack += rng.next_range(0, 2);
        }
        if rng.next_range(0, 10) == 0 {
            stack += rng.next_range(0, 3);
        }
        if rng.next_range(0, 15) == 0 {
            stack += rng.next_range(0, 4);
        }
    } else if num7 != -1 && rng.next_range(0, 2) == 0 {
        // Coins, first table (`:88-170`).
        if rng.next_range(0, 12000) == 0 {
            item_type = 74;
            if rng.next_range(0, 14) == 0 {
                stack += rng.next_range(0, 2);
            }
            if rng.next_range(0, 14) == 0 {
                stack += rng.next_range(0, 2);
            }
            if rng.next_range(0, 14) == 0 {
                stack += rng.next_range(0, 2);
            }
        } else if rng.next_range(0, 800) == 0 {
            item_type = 73;
            if rng.next_range(0, 6) == 0 {
                stack += rng.next_range(1, 21);
            }
            if rng.next_range(0, 6) == 0 {
                stack += rng.next_range(1, 21);
            }
            if rng.next_range(0, 6) == 0 {
                stack += rng.next_range(1, 21);
            }
            if rng.next_range(0, 6) == 0 {
                stack += rng.next_range(1, 20);
            }
        } else if rng.next_range(0, 60) == 0 {
            item_type = 72;
            if rng.next_range(0, 4) == 0 {
                stack += rng.next_range(5, 26);
            }
            if rng.next_range(0, 4) == 0 {
                stack += rng.next_range(5, 26);
            }
            if rng.next_range(0, 4) == 0 {
                stack += rng.next_range(5, 26);
            }
            if rng.next_range(0, 4) == 0 {
                stack += rng.next_range(5, 25);
            }
        } else {
            item_type = 71;
            if rng.next_range(0, 3) == 0 {
                stack += rng.next_range(10, 26);
            }
            if rng.next_range(0, 3) == 0 {
                stack += rng.next_range(10, 26);
            }
            if rng.next_range(0, 3) == 0 {
                stack += rng.next_range(10, 26);
            }
            if rng.next_range(0, 3) == 0 {
                stack += rng.next_range(10, 25);
            }
        }
    } else if num != -1 && rng.next_range(0, num) == 0 {
        // Amber Mosquito (`:171-174`).
        item_type = 1242;
    } else if num5 != -1 {
        // The bait table (`:175-193`) — Old Shoe / Fishing Seaweed / Tin Can.
        if rng.next_range(0, 4) != 1 {
            item_type = 2674; // Apprentice Bait
        } else if rng.next_range(0, 3) != 1 {
            item_type = 2006; // Snail
        } else if rng.next_range(0, 3) != 1 {
            item_type = 2002; // Worm
        } else {
            item_type = 2675; // Journeyman Bait
        }
    } else if num6 != -1 && extractinator_block_type == 642 {
        // The mosses on the chlorophyte extractinator (`:194-244`) — the rare
        // set (1-in-10) or the common set.
        if rng.next_range(0, 10) == 1 {
            item_type = match rng.next_range(0, 5) {
                0 => 4354, // Lava Moss
                1 => 4389, // Argon Moss
                2 => 4377, // Krypton Moss
                3 => 5127, // Violet Moss
                _ => 4378, // Xenon Moss
            };
        } else {
            item_type = match rng.next_range(0, 5) {
                0 => 4349, // Green Moss
                1 => 4350, // Brown Moss
                2 => 4351, // Red Moss
                3 => 4352, // Blue Moss
                _ => 4353, // Purple Moss
            };
        }
    } else if num6 != -1 {
        // The mosses on any extractinator (`:245-268`).
        item_type = match rng.next_range(0, 5) {
            0 => 4349, // Green Moss
            1 => 4350, // Brown Moss
            2 => 4351, // Red Moss
            3 => 4352, // Blue Moss
            _ => 4353, // Purple Moss
        };
    } else if num9 != -1 && rng.next_range(0, num9) == 0 {
        // The seed table (`:269-284`) — Poop Block.
        item_type = match rng.next_range(0, 3) {
            0 => 62,  // Grass Seeds
            1 => 195, // Jungle Grass Seeds
            _ => 194, // Mushroom Grass Seeds
        };
    } else if num8 > 0 {
        // Poop Block fallback (`:285-288`).
        item_type = 2; // Dirt Block
    } else if num11 > 0 {
        // Hive (`:289-292`).
        item_type = 1125; // Honey Block
    } else if num10 > 0 {
        // Shell Pile Block / Obsidian (`:293-296`).
        item_type = 169; // Sand Block
    } else if num2 != -1 && rng.next_range(0, num2) == 0 {
        // Gems (`:297-344`).
        item_type = match rng.next_range(0, 6) {
            0 => 181, // Amethyst
            1 => 180, // Topaz
            2 => 177, // Sapphire
            3 => 179, // Emerald
            4 => 178, // Ruby
            _ => 182, // Diamond
        };
        if rng.next_range(0, 20) == 0 {
            stack += rng.next_range(0, 2);
        }
        if rng.next_range(0, 30) == 0 {
            stack += rng.next_range(0, 3);
        }
        if rng.next_range(0, 40) == 0 {
            stack += rng.next_range(0, 4);
        }
        if rng.next_range(0, 50) == 0 {
            stack += rng.next_range(0, 5);
        }
        if rng.next_range(0, 60) == 0 {
            stack += rng.next_range(0, 6);
        }
    } else if num3 != -1 && rng.next_range(0, num3) == 0 {
        // Amber (`:345-368`).
        item_type = 999;
        if rng.next_range(0, 20) == 0 {
            stack += rng.next_range(0, 2);
        }
        if rng.next_range(0, 30) == 0 {
            stack += rng.next_range(0, 3);
        }
        if rng.next_range(0, 40) == 0 {
            stack += rng.next_range(0, 4);
        }
        if rng.next_range(0, 50) == 0 {
            stack += rng.next_range(0, 5);
        }
        if rng.next_range(0, 60) == 0 {
            stack += rng.next_range(0, 6);
        }
    } else if rng.next_range(0, 3) == 0 {
        // Coins, second table (`:369-459`).
        if rng.next_range(0, 5000) == 0 {
            item_type = 74;
            if rng.next_range(0, 10) == 0 {
                stack += rng.next_range(0, 3);
            }
            if rng.next_range(0, 10) == 0 {
                stack += rng.next_range(0, 3);
            }
            if rng.next_range(0, 10) == 0 {
                stack += rng.next_range(0, 3);
            }
            if rng.next_range(0, 10) == 0 {
                stack += rng.next_range(0, 3);
            }
            if rng.next_range(0, 10) == 0 {
                stack += rng.next_range(0, 3);
            }
        } else if rng.next_range(0, 400) == 0 {
            item_type = 73;
            if rng.next_range(0, 5) == 0 {
                stack += rng.next_range(1, 21);
            }
            if rng.next_range(0, 5) == 0 {
                stack += rng.next_range(1, 21);
            }
            if rng.next_range(0, 5) == 0 {
                stack += rng.next_range(1, 21);
            }
            if rng.next_range(0, 5) == 0 {
                stack += rng.next_range(1, 21);
            }
            if rng.next_range(0, 5) == 0 {
                stack += rng.next_range(1, 20);
            }
        } else if rng.next_range(0, 30) == 0 {
            item_type = 72;
            if rng.next_range(0, 3) == 0 {
                stack += rng.next_range(5, 26);
            }
            if rng.next_range(0, 3) == 0 {
                stack += rng.next_range(5, 26);
            }
            if rng.next_range(0, 3) == 0 {
                stack += rng.next_range(5, 26);
            }
            if rng.next_range(0, 3) == 0 {
                stack += rng.next_range(5, 25);
            }
        } else {
            item_type = 71;
            if rng.next_range(0, 2) == 0 {
                stack += rng.next_range(10, 26);
            }
            if rng.next_range(0, 2) == 0 {
                stack += rng.next_range(10, 26);
            }
            if rng.next_range(0, 2) == 0 {
                stack += rng.next_range(10, 26);
            }
            if rng.next_range(0, 2) == 0 {
                stack += rng.next_range(10, 25);
            }
        }
    } else {
        // Ore + stacks (`:460-487`).
        item_type = roll_ore_earlymode(rng);
        if extractinator_block_type == 642 && hard_mode {
            // The hardmode re-roll REPLACES the early result (the early draw
            // already spent a `Next(8)`, then the hard roll spends a
            // `Next(14)` — two draws, `ExtractinatorHelper.cs:462-466`).
            item_type = roll_ore_hardmode(rng);
        }
        if rng.next_range(0, 20) == 0 {
            stack += rng.next_range(0, 2);
        }
        if rng.next_range(0, 30) == 0 {
            stack += rng.next_range(0, 3);
        }
        if rng.next_range(0, 40) == 0 {
            stack += rng.next_range(0, 4);
        }
        if rng.next_range(0, 50) == 0 {
            stack += rng.next_range(0, 5);
        }
        if rng.next_range(0, 60) == 0 {
            stack += rng.next_range(0, 6);
        }
    }
    (item_type, stack)
}

/// `ExtractinatorHelper.RollOreHardmode` (`ExtractinatorHelper.cs:490-509`) —
/// the 14-entry pool the chlorophyte extractinator switches to in hardmode.
fn roll_ore_hardmode(rng: &mut dyn DropRng) -> i32 {
    match rng.next_range(0, 14) {
        0 => 12,    // Copper Ore
        1 => 11,    // Iron Ore
        2 => 14,    // Silver Ore
        3 => 13,    // Gold Ore
        4 => 699,   // Tin Ore
        5 => 700,   // Lead Ore
        6 => 701,   // Tungsten Ore
        7 => 702,   // Platinum Ore
        8 => 364,   // Cobalt Ore
        9 => 1104,  // Palladium Ore
        10 => 365,  // Mythril Ore
        11 => 1105, // Orichalcum Ore
        12 => 366,  // Adamantite Ore
        _ => 1106,  // Titanium Ore
    }
}

/// `ExtractinatorHelper.RollOreEarlymode` (`ExtractinatorHelper.cs:511-524`) —
/// the 8-entry pre-hardmode pool (both ore alternatives included).
fn roll_ore_earlymode(rng: &mut dyn DropRng) -> i32 {
    match rng.next_range(0, 8) {
        0 => 12,  // Copper Ore
        1 => 11,  // Iron Ore
        2 => 14,  // Silver Ore
        3 => 13,  // Gold Ore
        4 => 699, // Tin Ore
        5 => 700, // Lead Ore
        6 => 701, // Tungsten Ore
        _ => 702, // Platinum Ore
    }
}

/// `ItemID.Sets.ExtractinatorMode` (`ItemID.cs:1321`): the held item's
/// extraction mode, -1 when the item is not an extractinator input.
///
/// | item | mode | | item | mode |
/// | --- | --- | --- | --- | --- |
/// | 424 Silt Block | 0 | | 4354/4389/4377/4378/5127/5128 | 3 (mosses) |
/// | 1103 Slush Block | 0 | | 5395 Poop Block | 4 |
/// | 3347 Desert Fossil | 1 | | 1124 Hive | 5 |
/// | 2337/2338/2339 junk trio | 2 | | 4090 Shell Pile, 173 Obsidian | 6 |
pub fn extractinator_mode(item: u16) -> i32 {
    match item {
        424 | 1103 => 0,
        3347 => 1,
        2337 | 2338 | 2339 => 2,
        4354 | 4389 | 4377 | 4378 | 5127 | 5128 => 3,
        5395 => 4,
        1124 => 5,
        4090 | 173 => 6,
        _ => -1,
    }
}

/// `ItemTrader.ChlorophyteExtractinator` (`ItemTrader.cs:82-108`) resolved for
/// `held`: the first option whose `WillTradeFor` accepts (`ItemTrader.cs:17-24`)
/// — the type must equal `TakingItemType` and the stack must cover
/// `TakingItemStack`. Returns `(giving_type, giving_stack)`; every option in the
/// table is 1-for-1, but the stack is read from the table's literal so the
/// citation stays complete.
///
/// The insertion order below IS the match order (`TryGetTradeOption` walks
/// `_options` linearly, `ItemTrader.cs:70-78`) — keep it.
pub fn chlorophyte_trade(held: u16, held_stack: u32) -> Option<(u16, u32)> {
    let taking = |item_type: u16, taking_stack: u32, giving: u16| -> Option<(u16, u32)> {
        (held == item_type && held_stack >= taking_stack).then_some((giving, 1))
    };
    let from_any = |giving: u16, taking_item_types: &[u16]| -> Option<(u16, u32)> {
        taking_item_types
            .iter()
            .find_map(|&t| taking(t, 1, giving))
    };
    // 1. The ore alternatives (`ItemTrader.cs:85-88`).
    if let Some(r) = taking(12, 1, 699).or_else(|| taking(699, 1, 12)) {
        return Some(r);
    }
    if let Some(r) = taking(11, 1, 700).or_else(|| taking(700, 1, 11)) {
        return Some(r);
    }
    if let Some(r) = taking(14, 1, 701).or_else(|| taking(701, 1, 14)) {
        return Some(r);
    }
    if let Some(r) = taking(13, 1, 702).or_else(|| taking(702, 1, 13)) {
        return Some(r);
    }
    // 2. Demonite <-> Crimtane ore (`:89`).
    if let Some(r) = taking(56, 1, 880).or_else(|| taking(880, 1, 56)) {
        return Some(r);
    }
    // 3. The hardmode ore alternatives (`:90-92`).
    if let Some(r) = taking(364, 1, 1104).or_else(|| taking(1104, 1, 364)) {
        return Some(r);
    }
    if let Some(r) = taking(365, 1, 1105).or_else(|| taking(1105, 1, 365)) {
        return Some(r);
    }
    if let Some(r) = taking(366, 1, 1106).or_else(|| taking(1106, 1, 366)) {
        return Some(r);
    }
    // 4. The brick cycle (`:93`).
    if held == 134 {
        return Some((137, 1));
    }
    if held == 137 {
        return Some((139, 1));
    }
    if held == 139 {
        return Some((134, 1));
    }
    // 5. The bar alternatives (`:94-97`).
    if let Some(r) = taking(20, 1, 703).or_else(|| taking(703, 1, 20)) {
        return Some(r);
    }
    if let Some(r) = taking(22, 1, 704).or_else(|| taking(704, 1, 22)) {
        return Some(r);
    }
    if let Some(r) = taking(21, 1, 705).or_else(|| taking(705, 1, 21)) {
        return Some(r);
    }
    if let Some(r) = taking(19, 1, 706).or_else(|| taking(706, 1, 19)) {
        return Some(r);
    }
    // 6. Demonite <-> Crimtane bar (`:98`).
    if let Some(r) = taking(57, 1, 1257).or_else(|| taking(1257, 1, 57)) {
        return Some(r);
    }
    // 7. The hardmode bar alternatives (`:99-101`).
    if let Some(r) = taking(381, 1, 1184).or_else(|| taking(1184, 1, 381)) {
        return Some(r);
    }
    if let Some(r) = taking(382, 1, 1191).or_else(|| taking(1191, 1, 382)) {
        return Some(r);
    }
    if let Some(r) = taking(391, 1, 1198).or_else(|| taking(1198, 1, 391)) {
        return Some(r);
    }
    // 8. Shadow Scale <-> Tissue Sample (`:102`).
    if let Some(r) = taking(86, 1, 1329).or_else(|| taking(1329, 1, 86)) {
        return Some(r);
    }
    // 9-13. The FromAny conversions (`:103-107`).
    if let Some(r) = from_any(3, &[61, 836, 409]) {
        return Some(r);
    }
    if let Some(r) = from_any(169, &[370, 1246, 408]) {
        return Some(r);
    }
    if let Some(r) = from_any(664, &[833, 835, 834]) {
        return Some(r);
    }
    if let Some(r) = from_any(3271, &[3276, 3277, 3339]) {
        return Some(r);
    }
    if let Some(r) = from_any(3272, &[3274, 3275, 3338]) {
        return Some(r);
    }
    None
}

/// A seeded replay stream for the tests: the port's `FastRandom`
/// (`tile_light/palette.rs`) — not `Main.rand`'s `UnifiedRandom`, which has no
/// port yet; the app rolls against its `item_rng` seam instead. A fixed stream
/// is enough to pin the branch order and the distribution shapes.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile_light::FastRandom;

    struct Seeded(FastRandom);

    impl DropRng for Seeded {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.0.next_range(min, max)
        }
        fn next_float(&mut self) -> f32 {
            self.0.next_float()
        }
    }

    fn mode(item: u16, block: u16, hard_mode: bool, seed: u64) -> (i32, i32) {
        // Small seeds degenerate `FastRandom`'s first draws (the 48-bit state
        // lands in the same top-bit class), so the replay streams are spread.
        let spread = seed
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(0x5DEE_CE66_D1A4_9B5F);
        roll_extractinator_drop(
            extractinator_mode(item),
            block,
            hard_mode,
            &mut Seeded(FastRandom::new(spread)),
        )
    }

    #[test]
    fn mode_minus_one_drops_nothing() {
        // `ExtractinatorHelper.cs:20-23` — the only caller is the wired path.
        assert_eq!(roll_extractinator_drop(-1, 219, false, &mut Seeded(FastRandom::new(7))), (-1, 1));
        assert_eq!(mode(999, 219, false, 7), (-1, 1));
    }

    #[test]
    fn fixed_output_modes_never_roll() {
        // Mode 5 (Hive) and 6 (Shell Pile/Obsidian) are literal outputs —
        // `ExtractinatorHelper.cs:289-296`; the seed must not matter.
        assert_eq!(mode(1124, 219, false, 1), (1125, 1));
        assert_eq!(mode(1124, 219, false, 999), (1125, 1));
        assert_eq!(mode(173, 219, false, 42), (169, 1));
        assert_eq!(mode(4090, 219, false, 42), (169, 1));
    }

    #[test]
    fn poop_falls_back_to_dirt_on_missed_seed_roll() {
        // Mode 4: the 1/50 seed table, else Dirt — `ExtractinatorHelper.cs:269-288`.
        let mut hits = 0;
        let mut dirt = 0;
        for seed in 0..2000u64 {
            let (kind, stack) = mode(5395, 219, false, seed);
            if matches!(kind, 62 | 194 | 195) {
                hits += 1;
                assert_eq!(stack, 1);
            } else {
                assert_eq!((kind, stack), (2, 1));
                dirt += 1;
            }
        }
        assert!(hits > 10 && hits < 90, "seed hits {hits}/2000 (~40 expected)");
        assert_eq!(hits + dirt, 2000);
    }

    #[test]
    fn moss_mode_is_common_set_on_219_rare_set_with_chance_on_642() {
        // `ExtractinatorHelper.cs:194-268` — the six mode-3 mosses (the common
        // mosses 4349-4353 are outputs only, not inputs) roll the common set on
        // 219 and the rare set at 1/10 on 642.
        for seed in 0..500u64 {
            let (kind, _) = mode(4354, 219, false, seed);
            assert!(matches!(kind, 4349..=4353), "219 moss {kind}");
        }
        let mut rare = 0;
        for seed in 0..500u64 {
            let (kind, _) = mode(4354, 642, false, seed);
            if matches!(kind, 4354 | 4389 | 4377 | 5127 | 4378) {
                rare += 1;
            } else {
                assert!(matches!(kind, 4349..=4353), "642 common moss {kind}");
            }
        }
        assert!((25..=75).contains(&rare), "rare moss {rare}/500 (~50 expected)");
    }

    #[test]
    fn junk_mode_never_rolls_ore_coins_or_amber() {
        // Mode 2's fixed bait table — `ExtractinatorHelper.cs:175-193`.
        for seed in 0..1000u64 {
            let (kind, _) = mode(2337, 219, false, seed);
            assert!(matches!(kind, 2674 | 2006 | 2002 | 2675), "bait {kind}");
        }
    }

    #[test]
    fn ore_pools_switch_only_for_642_hardmode() {
        // `ExtractinatorHelper.cs:460-466` — the ore arm is the final `else`, so
        // most rolls land on the coin tables first; only the rolls that reach it
        // matter here. 219 (even in hardmode) stays in the 8-entry pool; 642 +
        // hardmode switches to the 14-entry pool (the re-roll spends a second
        // draw after the early one).
        let early = [12, 11, 14, 13, 699, 700, 701, 702];
        let hard = [12, 11, 14, 13, 699, 700, 701, 702, 364, 1104, 365, 1105, 366, 1106];
        let mut early_hits = 0;
        let mut hard_hits = 0;
        let mut hardmode_only = 0;
        for seed in 0..4000u64 {
            let kind = mode(424, 219, true, seed).0;
            if hard.contains(&kind) || early.contains(&kind) {
                // Only the ore arm reaches these ids (the coin arms mint 71-74).
                assert!(early.contains(&kind), "219 hardmode ore {kind}");
                early_hits += 1;
            }
            let kind = mode(424, 642, true, seed).0;
            if hard.contains(&kind) {
                hard_hits += 1;
                if !early.contains(&kind) {
                    hardmode_only += 1;
                }
            }
        }
        assert!(early_hits > 100, "ore rolls {early_hits}/4000");
        assert!(hard_hits > 100, "ore rolls {hard_hits}/4000");
        // The 14-entry pool replaces the 8-entry one on 642 + hardmode, so its
        // six additions (364/1104/365/1105/366/1106) must appear at all.
        assert!(hardmode_only > 5, "hardmode additions {hardmode_only}/{hard_hits}");
    }

    #[test]
    fn fossil_mode_opens_the_fossil_ore_and_boosted_rears() {
        // Mode 1 is the only one with the fossil-ore arm and the x3 mosquito
        // rate — a distribution check that both arms exist at all.
        let mut fossil = 0;
        let mut mosquitoes = 0;
        for seed in 0..5000u64 {
            let (kind, _) = mode(3347, 219, false, seed);
            match kind {
                3380 => fossil += 1,
                1242 => mosquitoes += 1,
                _ => {}
            }
        }
        // 1-in-10 and 1-in-1666 over 5000 rolls: ~500 and ~3.
        assert!((350..=650).contains(&fossil), "fossil {fossil}/5000");
        assert!(mosquitoes <= 20, "mosquitoes {mosquitoes}/5000 (~3 expected)");
    }

    #[test]
    fn extractinator_mode_map_matches_the_c_sharp_set() {
        assert_eq!(extractinator_mode(424), 0);
        assert_eq!(extractinator_mode(1103), 0);
        assert_eq!(extractinator_mode(3347), 1);
        assert_eq!(extractinator_mode(2337), 2);
        assert_eq!(extractinator_mode(2338), 2);
        assert_eq!(extractinator_mode(2339), 2);
        assert_eq!(extractinator_mode(5128), 3);
        assert_eq!(extractinator_mode(5395), 4);
        assert_eq!(extractinator_mode(1124), 5);
        assert_eq!(extractinator_mode(4090), 6);
        assert_eq!(extractinator_mode(173), 6);
        assert_eq!(extractinator_mode(2), -1);
        assert_eq!(extractinator_mode(0), -1);
    }

    #[test]
    fn trader_matches_one_for_one_and_respects_stack() {
        assert_eq!(chlorophyte_trade(12, 1), Some((699, 1)));
        assert_eq!(chlorophyte_trade(699, 1), Some((12, 1)));
        assert_eq!(chlorophyte_trade(12, 0), None);
        assert_eq!(chlorophyte_trade(86, 1), Some((1329, 1)));
        assert_eq!(chlorophyte_trade(1234, 99), None);
        // The brick cycle and the FromAny groups.
        assert_eq!(chlorophyte_trade(134, 1), Some((137, 1)));
        assert_eq!(chlorophyte_trade(139, 1), Some((134, 1)));
        assert_eq!(chlorophyte_trade(836, 1), Some((3, 1)));
        assert_eq!(chlorophyte_trade(3338, 1), Some((3272, 1)));
    }

    #[test]
    fn seeded_replay_is_deterministic() {
        let a = mode(424, 219, false, 12345);
        let b = mode(424, 219, false, 12345);
        assert_eq!(a, b);
    }

}
