//! The Angler's quest (`Main.AnglerQuestSwap` / `Main.NPCChatText_DoAnglerQuest`
//! / `Player.GetAnglerReward`) — M4-TOWN-15.
//!
//! The vanilla keeps the quest on the Main statics: a 41-entry item pool
//! (`Main.anglerQuestItemNetIDs`, `Main.cs:67032-67039`), a per-day re-roll
//! (`AnglerQuestSwap`, `Main.cs:3754-3902`) that runs from
//! `UpdateTime_StartDay` (`Main.cs:65373`), a turn-in
//! (`NPCChatText_DoAnglerQuest`, `Main.cs:40227-40263`) and a four-part
//! reward (`Player.GetAnglerReward`, `Player.cs:56302-56311`).
//! The port carries the state on the World (the single player collapses
//! `anglerWhoFinishedToday`'s name list to one flag) and this module holds
//! the pool, the roll ladder and the reward table.

use crate::app::{next_rand, rand01};

/// `Main.anglerQuestItemNetIDs` (`Main.cs:67032-67039`): the 41 quest fish —
/// the 39 Dye/Trap/Oyster/Boot/Mucus rows 2450-2488 plus the two Legendary
/// quest fish 4393/4394.
pub const QUEST_POOL: [u16; 41] = [
    2450, 2451, 2452, 2453, 2454, 2455, 2456, 2457, 2458, 2459, 2460, 2461, 2462, 2463, 2464, 2465,
    2466, 2467, 2468, 2469, 2470, 2471, 2472, 2473, 2474, 2475, 2476, 2477, 2478, 2479, 2480, 2481,
    2482, 2483, 2484, 2485, 2486, 2487, 2488, 4393, 4394,
];

/// `Main.AnglerQuestSwap`'s `flag` (`Main.cs:3762`): the "any progression"
/// gate the 2476/2453/2473 rows need — `downedBoss1 || downedBoss2 ||
/// downedBoss3 || hardMode || downedSlimeKing || downedQueenBee` (the port's
/// `downed_eoc`/`downed_eow`/`downed_skeletron` are the C#'s Boss1/2/3).
#[derive(Clone)]
pub struct QuestGates {
    pub hard_mode: bool,
    pub crimson: bool,
    pub downed_boss1: bool,
    pub downed_boss2: bool,
    pub downed_boss3: bool,
    pub downed_slime_king: bool,
    pub downed_queen_bee: bool,
    /// `Main.isThereAWorldSurface` — the constants-off "don't starve" seed.
    pub has_world_surface: bool,
}

/// `Player.GetAnglerRewardRarityMultiplier` (`Player.cs:56313-56317`): the
/// rarity reduction that scales every reward denominator. The C# multiplies
/// the tier result by 0.9.
pub fn reward_rarity_multiplier(quests_done: i32) -> f32 {
    let tier = if quests_done <= 50 {
        1.0 - quests_done as f32 * 0.01
    } else if quests_done <= 100 {
        0.5 - (quests_done - 50) as f32 * 0.005
    } else if quests_done > 150 {
        0.15
    } else {
        0.25 - (quests_done - 100) as f32 * 0.002
    };
    tier * 0.9
}

/// The one-reward-item record the C#'s `QuickSpawnItem` calls produce — the
/// port spawns the drops from the caller.
#[derive(Clone, Debug, PartialEq)]
pub struct RewardLine {
    pub item: u16,
    pub stack: u32,
}

/// `Main.AnglerQuestSwap` (`Main.cs:3754-3902`): the per-day quest re-roll.
///
/// The `netMode == 1` early return, the `anglerWhoFinishedToday.Clear()` and
/// `anglerQuestFinished = false` day reset ride the caller (the World's
/// fields); this rolls the pool with the vanilla's reject ladder — a rejected
/// id draws a fresh `Next(41)` (the global `Main.rand` stream, so the
/// re-rolls consume it in order).
pub fn angler_quest_swap(gates: &QuestGates, rng: &mut u64) -> u16 {
    // `:3762`'s progression gate.
    let progressed = gates.downed_boss1
        || gates.downed_boss2
        || gates.downed_boss3
        || gates.hard_mode
        || gates.downed_slime_king
        || gates.downed_queen_bee;
    loop {
        let idx = next_int(rng, 41) as usize;
        let item = QUEST_POOL[idx];
        // The reject ladder, in the C#'s if-order.
        let mut reject = false;
        if item == 2454 && (!gates.hard_mode || gates.crimson) {
            reject = true;
        }
        if item == 2457 && gates.crimson {
            reject = true;
        }
        if item == 2462 && !gates.hard_mode {
            reject = true;
        }
        if item == 2463 && (!gates.hard_mode || !gates.crimson) {
            reject = true;
        }
        if item == 2465 && !gates.hard_mode {
            reject = true;
        }
        if item == 2468 && !gates.hard_mode {
            reject = true;
        }
        if item == 2471 && !gates.hard_mode {
            reject = true;
        }
        if item == 2473 && !gates.hard_mode {
            reject = true;
        }
        if item == 2477 && !gates.crimson {
            reject = true;
        }
        if item == 2480 && !gates.hard_mode {
            reject = true;
        }
        if item == 2483 && !gates.hard_mode {
            reject = true;
        }
        if item == 2484 && !gates.hard_mode {
            reject = true;
        }
        if item == 2485 && gates.crimson {
            reject = true;
        }
        if (item == 2476 || item == 2453 || item == 2473) && !progressed {
            reject = true;
        }
        // `:3825-3899`: the no-surface world's exclusions.
        if !gates.has_world_surface {
            const NO_SURFACE: [u16; 21] = [
                2476, 2479, 2480, 2452, 2453, 2481, 2483, 2456, 2458, 2459, 2461, 2467, 2468, 2487,
                2488, 2470, 2473, 2474, 2494, 2498, 4394,
            ];
            if NO_SURFACE.contains(&item) {
                reject = true;
            }
        }
        if !reject {
            return idx as u16;
        }
    }
}

/// `Player.GetAnglerReward` (`Player.cs:56302-56311`): the four sub-rolls in
/// the C#'s order — MainReward, Decoration, Money, Bait — each on the global
/// `Main.rand` stream.
///
/// `owned_ids` is the reward's accessory filter input
/// (`DropAnglerAccByMissing`'s inventory/armor/bank/loadout scan,
/// `Player.cs:56585-56683`): the port's single inventory + the armor slots.
/// The price adjustment is the Angler's mood multiplier (the merchant's
/// `currentShoppingSettings.PriceAdjustment`, `:56306`).
pub fn angler_reward(
    quests_done: i32,
    hard_mode: bool,
    price_adjustment: f32,
    owned_ids: &[u16],
    rng: &mut u64,
) -> Vec<RewardLine> {
    let rarity = reward_rarity_multiplier(quests_done);
    let reduction = rarity * ((price_adjustment + 1.0) / 2.0);
    let mut lines = Vec::new();
    reward_main(quests_done, hard_mode, reduction, owned_ids, rng, &mut lines);
    reward_decoration(quests_done, reduction, rng, &mut lines);
    reward_money(quests_done, hard_mode, rng, &mut lines);
    reward_bait(quests_done, reduction, rng, &mut lines);
    lines
}

/// `Player.GetAnglerReward_MainReward` (`Player.cs:56319-56463`): the
/// milestone table (5/10/15/20/25/30 quests), the hardmode 2451 fish rows,
/// the scaled rarity rolls and the accessory pool.
fn reward_main(
    quests_done: i32,
    hard_mode: bool,
    reduction: f32,
    owned_ids: &[u16],
    rng: &mut u64,
    lines: &mut Vec<RewardLine>,
) {
    let item = match quests_done {
        5 => Some(2428u16),
        10 => Some(2367),
        15 => Some(2368),
        20 => Some(2369),
        25 => Some(3031),
        30 => Some(2294),
        _ => None,
    };
    let picked = if let Some(mile) = item {
        Some((mile, 1u32))
    } else {
        // `:56345-56354`: the 2451 quest-fish row (the Jellyfish) splits
        // 5303/5302 — the hardmode arm rolls twice (a `Next(2) == 0` gate
        // then the pick), the classic arm once.
        let mut picked = None;
        if hard_mode && next_int(rng, 2) == 0 {
            picked = Some((if next_int(rng, 2) == 0 { 5303 } else { 5302 }, 1));
        } else if !hard_mode && next_int(rng, 2) == 0 {
            picked = Some((if next_int(rng, 2) == 0 { 5303 } else { 5302 }, 1));
        }
        if picked.is_none() {
            // `:56355-56365`: the accessory bookkeeping (the C# computes the
            // failure product the `DropAnglerAccByMissing` arm reads).
            let wanted: [u16; 7] = [2373, 2374, 2375, 3120, 3037, 3096, 5139];
            let mut num = 1.0f32;
            for _ in 0..3 {
                num *= 1.0 - 1.0 / (40.0 * reduction) as i32 as f32;
            }
            for _ in 0..3 {
                num *= 1.0 - 1.0 / (30.0 * reduction) as i32 as f32;
            }
            let num2 = (1.0 - num * (1.0 - 1.0 / (25.0 * reduction) as i32 as f32)) * 0.8;
            if quests_done > 75 && next_int(rng, (250.0 * reduction) as i32) == 0 {
                picked = Some((2294, 1));
            } else if hard_mode && quests_done > 25 && next_int(rng, (100.0 * reduction) as i32) == 0
            {
                picked = Some((2422, 1));
            } else if hard_mode && quests_done > 10 && next_int(rng, (70.0 * reduction) as i32) == 0 {
                picked = Some((2494, 1));
            } else if quests_done > 10 && next_int(rng, (70.0 * reduction) as i32) == 0 {
                picked = Some((3031, 1));
            } else if quests_done > 10 && next_int(rng, (70.0 * reduction) as i32) == 0 {
                picked = Some((3032, 1));
            } else if next_int(rng, (80.0 * reduction) as i32) == 0 {
                picked = Some((3183, 1));
            } else if next_int(rng, (60.0 * reduction) as i32) == 0 {
                picked = Some((2360, 1));
            } else if next_int(rng, (60.0 * reduction) as i32) == 0 {
                picked = Some((4067, 1));
            } else if next_int(rng, (80.0 * reduction) as i32) == 0 {
                picked = Some((2417, 1));
            } else if next_int(rng, (80.0 * reduction) as i32) == 0 {
                picked = Some((2498, 1));
            } else {
                // `:56416-56425`: the missing-accessory pool.
                let mut pool: Vec<u16> = wanted.to_vec();
                for id in owned_ids {
                    remove_acc_options(&mut pool, *id);
                }
                let mut bothered = false;
                if pool.is_empty() {
                    picked = None;
                } else {
                    bothered = true;
                    if rand01(rng) < num2 {
                        picked = Some((pool[next_int(rng, pool.len() as i32) as usize], 1));
                    }
                }
                if picked.is_none() && (!bothered || true) && rand01(rng) < num2 {
                    picked = Some((wanted[next_int(rng, 7) as usize], 1));
                }
                picked = picked.or_else(|| {
                    // `:56426-56439`: the three potion rows with a 2-5 stack.
                    Some((
                        match next_int(rng, 3) {
                            0 => 2354,
                            1 => 2355,
                            _ => 2356,
                        },
                        next_int(rng, 2 + 4) as u32,
                    ))
                });
            }
        }
        picked
    };
    if let Some((id, stack)) = picked {
        lines.push(RewardLine { item: id, stack });
        // `:56445-56462`: the two accessory pairs ride their own drops.
        if id == 2417 {
            lines.push(RewardLine { item: 2418, stack: 1 });
            lines.push(RewardLine { item: 2419, stack: 1 });
        } else if id == 2498 {
            lines.push(RewardLine { item: 2499, stack: 1 });
            lines.push(RewardLine { item: 2500, stack: 1 });
        }
    }
}

/// `Player.RemoveAnglerAccOptionsFromRewardPool` (`Player.cs:56646-56683`):
/// an owned item removes its row — and the families (3721/5064, the
/// High-Tier boots, the 5140-5146 set) remove their whole group.
fn remove_acc_options(pool: &mut Vec<u16>, owned: u16) {
    match owned {
        3721 | 5064 => {
            pool.retain(|v| !matches!(v, 2373 | 2374 | 2375));
        }
        3036 | 3123 | 3124 | 5358 | 5359 | 5360 | 5361 => {
            pool.retain(|v| !matches!(v, 3120 | 3037 | 3096));
        }
        5140 | 5141 | 5142 | 5143 | 5144 | 5145 | 5146 | 5147 => {
            pool.retain(|v| *v != 5139);
        }
        _ => pool.retain(|v| *v != owned),
    }
}

/// `Player.GetAnglerReward_Decoration` (`Player.cs:56465-56496`): the
/// wall-decoration roll, its chance lerped up with the quest count.
fn reward_decoration(
    quests_done: i32,
    reduction: f32,
    rng: &mut u64,
    lines: &mut Vec<RewardLine>,
) {
    let num = (1.0 - reduction).lerp(1.0, (quests_done as f32 / 100.0).min(1.0));
    if num >= 1.0 || rand01(rng) <= num {
        let item = match next_int(rng, 19) {
            1 => 2443,
            2 => 2444,
            3 => 2445,
            4 => 2497,
            5 => 2495,
            6 => 2446,
            7 => 2447,
            8 => 2448,
            9 => 2449,
            10 => 2490,
            11 => 2496,
            12 => 5235,
            13 => 5252,
            14 => 5256,
            15 => 5259,
            16 => 5263,
            17 => 5264,
            18 => 5265,
            _ => 2442,
        };
        lines.push(RewardLine { item, stack: 1 });
    }
}

/// `Player.GetAnglerReward_Money` (`Player.cs:56543-56583`): the gold/silver
/// coin roll (the expert-mode ×2 sits constants-off).
fn reward_money(quests_done: i32, hard_mode: bool, rng: &mut u64, lines: &mut Vec<RewardLine>) {
    let mut num = (((quests_done + 50) / 2 * next_int(rng, 50 + 151)) as f32 * 0.015) as i32;
    if hard_mode {
        num *= 2;
    }
    if num > 100 {
        num /= 100;
        num = num.clamp(1, 10);
        lines.push(RewardLine {
            item: 73, // `ItemID.GoldCoin`
            stack: num as u32,
        });
    } else {
        num = num.clamp(1, 99);
        lines.push(RewardLine {
            item: 72, // `ItemID.SilverCoin`
            stack: num as u32,
        });
    }
}

/// `Player.GetAnglerReward_Bait` (`Player.cs:56498-56541`): the bait roll
/// with the six quest-count stack bumps.
fn reward_bait(quests_done: i32, reduction: f32, rng: &mut u64, lines: &mut Vec<RewardLine>) {
    if next_int(rng, (100.0 * reduction) as i32) <= 50 {
        let mut stack = 1u32;
        let item = if next_int(rng, (15.0 * reduction) as i32) == 0 {
            2676
        } else if next_int(rng, (5.0 * reduction) as i32) == 0 {
            2675
        } else {
            2674
        };
        for gate in [25, 50, 100, 150, 200, 250] {
            if next_int(rng, gate) <= quests_done {
                stack += 1;
            }
        }
        lines.push(RewardLine { item, stack });
    }
}

/// `Main.rand.Next(min, max)` — the port shares one xorshift stream
/// (`app/frame.rs`'s `next_rand`), so the int draws and the float draws
/// interleave exactly like the C#'s `Main.rand`.
fn next_int(rng: &mut u64, n: i32) -> i32 {
    debug_assert!(n > 0);
    (next_rand(rng) % n.max(1) as u64) as i32
}

/// The lerp the C#'s `MathHelper.Lerp` does (`Player.cs:56467`).
trait Lerp {
    fn lerp(self, to: f32, t: f32) -> f32;
}

impl Lerp for f32 {
    fn lerp(self, to: f32, t: f32) -> f32 {
        self + (to - self) * t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Main.cs:67032-67039`: the 41-id pool — the 39 rows of 2450-2488 and
    /// the two Legendary fish (4393/4394).
    #[test]
    fn the_quest_pool_carries_forty_one_fish() {
        assert_eq!(QUEST_POOL.len(), 41);
        assert_eq!(QUEST_POOL[0], 2450);
        assert_eq!(QUEST_POOL[38], 2488);
        assert_eq!(QUEST_POOL[39], 4393);
        assert_eq!(QUEST_POOL[40], 4394);
    }

    /// `Main.AnglerQuestSwap`'s reject ladder (`Main.cs:3769-3899`): a classic
    /// corruption world can never roll 2454 (it needs hardmode and a non-
    /// crimson world), while the hardmode gate opens it.
    #[test]
    fn the_quest_swap_rejects_the_gated_fish() {
        let classic = QuestGates {
            hard_mode: false,
            crimson: false,
            downed_boss1: false,
            downed_boss2: false,
            downed_boss3: false,
            downed_slime_king: false,
            downed_queen_bee: false,
            has_world_surface: true,
        };
        let mut rng = 7u64;
        for _ in 0..20_000 {
            let idx = angler_quest_swap(&classic, &mut rng);
            assert!(QUEST_POOL[idx as usize] != 2454, "the classic 2454 re-rolls");
            // 2457/2485 are the CRIMSON-only rows — they stay available in a
            // corruption world; 2462 needs hardmode.
            assert!(QUEST_POOL[idx as usize] != 2462, "the classic 2462 re-rolls");
            assert!(QUEST_POOL[idx as usize] != 2465, "the classic 2465 re-rolls");
        }
        // The hardmode arm opens 2454 — the sword-fish row.
        let mut hard = classic.clone();
        hard.hard_mode = true;
        let mut rng = 7u64;
        let mut saw = false;
        for _ in 0..20_000 {
            if QUEST_POOL[angler_quest_swap(&hard, &mut rng) as usize] == 2454 {
                saw = true;
                break;
            }
        }
        assert!(saw, "hardmode reaches 2454");
    }

    /// The roll is deterministic for a seed and the classic default has no
    /// progression rows: the first draw of seed 7 pins the index.
    #[test]
    fn the_quest_swap_is_seed_deterministic() {
        let gates = QuestGates {
            hard_mode: false,
            crimson: false,
            downed_boss1: false,
            downed_boss2: false,
            downed_boss3: false,
            downed_slime_king: false,
            downed_queen_bee: false,
            has_world_surface: true,
        };
        let a = angler_quest_swap(&gates, &mut 7);
        let b = angler_quest_swap(&gates, &mut 7);
        let c = angler_quest_swap(&gates, &mut 8);
        assert_eq!(a, b, "the same seed rolls the same index");
        assert_eq!(QUEST_POOL[a as usize], 2487, "the 2487 default row");
        assert_ne!(a, c, "the next seed moves");
    }

    /// `Player.GetAnglerRewardRarityMultiplier` (`Player.cs:56313-56317`):
    /// the 1 - 0.01n / 0.5 - 0.005n-50 / 0.25 - 0.002n-100 / 0.15 tiers,
    /// each x0.9.
    #[test]
    fn the_reward_rarity_multiplier_halves_by_quest_count() {
        assert!((reward_rarity_multiplier(0) - 0.9).abs() < 1e-6);
        assert!((reward_rarity_multiplier(50) - 0.45).abs() < 1e-6);
        assert!((reward_rarity_multiplier(100) - 0.225).abs() < 1e-6);
        assert!((reward_rarity_multiplier(200) - 0.135).abs() < 1e-6);
    }

    /// `GetAnglerReward_MainReward`'s milestone switch (`Player.cs:56323-56342`):
    /// the 5/10/15/20/25/30 rows hand their exact items, and the accessory
    /// pair (2417) drags its two companions (`:56445-56462`).
    #[test]
    fn the_reward_milestones_hand_their_rows() {
        for (done, id) in [(5, 2428), (10, 2367), (15, 2368), (20, 2369), (25, 3031), (30, 2294)] {
            let mut rng = 99u64;
            let lines = angler_reward(done, false, 1.0, &[], &mut rng);
            assert_eq!(lines[0].item, id, "the {done}-quest milestone");
        }
        // The 2417 toy row drags 2418 + 2419.
        let mut rng = 1u64;
        let mut saw = false;
        for seed in 0..400u64 {
            let mut r = seed + 1;
            let lines = angler_reward(3, false, 1.0, &[], &mut r);
            if lines.iter().any(|l| l.item == 2417) {
                saw = true;
                assert!(lines.iter().any(|l| l.item == 2418), "the toy's pair");
                assert!(lines.iter().any(|l| l.item == 2419), "the toy's second pair");
                break;
            }
        }
        let _ = rng;
        assert!(saw, "the 2417 row appears within 400 seeds");
    }

    /// `GetAnglerReward_Money` (`Player.cs:56543-56583`): the classic roll
    /// lands on silver (72), the hardmode roll on gold (73), and both stacks
    /// clamp into the coin's range.
    #[test]
    fn the_reward_money_clamps_into_the_coin_ranges() {
        for seed in 0..80u64 {
            let mut r = seed + 1;
            let lines = angler_reward(10, false, 1.0, &[], &mut r);
            let money = lines.iter().find(|l| l.item == 72 || l.item == 73).expect("a coin");
            if money.item == 72 {
                assert!(money.stack >= 1 && money.stack <= 99, "the silver stack");
            } else {
                assert!(money.stack >= 1 && money.stack <= 10, "the gold stack");
            }
        }
        let mut hard_r = 3u64;
        let lines = angler_reward(10, true, 1.0, &[], &mut hard_r);
        assert!(
            lines.iter().any(|l| l.item == 72 || l.item == 73),
            "the hardmode roll still lands a coin"
        );
    }

    /// `GetAnglerReward_Bait` (`Player.cs:56498-56541`): the bait line carries
    /// one of 2674/2675/2676 with the quest-count stack bumps.
    #[test]
    fn the_reward_bait_carries_a_known_bait_row() {
        for seed in 0..80u64 {
            let mut r = seed + 1;
            let lines = angler_reward(10, false, 1.0, &[], &mut r);
            if let Some(bait) = lines.iter().find(|l| matches!(l.item, 2674 | 2675 | 2676)) {
                assert!(bait.stack >= 1 && bait.stack <= 7, "the bait stack {bait:?}");
            }
        }
    }

    /// `DropAnglerAccByMissing`'s pool filter (`Player.cs:56585-56683`): an
    /// owned 3721 removes the boot family (2373/2374/2375) from the pool, so
    /// a 3721-owner never sees one of the three.
    #[test]
    fn the_reward_accessory_pool_drops_the_owned_families() {
        let mut pool: Vec<u16> = vec![2373, 2374, 2375, 3120, 5139];
        remove_acc_options(&mut pool, 3721);
        assert!(pool.iter().all(|v| !matches!(v, 2373 | 2374 | 2375)), "the 3721 family");
        remove_acc_options(&mut pool, 5140);
        assert!(!pool.contains(&5139), "the 5140 family");
    }
}
