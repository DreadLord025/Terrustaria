//! W4-f35 — the misc openable-bag OPEN trees: `Player.OpenHerbBag`
//! (`Player.cs:9367-9441`, the Herb Bag 3093), `Player.OpenCanofWorms`
//! (`:9443-9455`, 4345), `Player.OpenOyster` (`:9457-9476`, 4410),
//! `Player.OpenChilletEgg` (`:9491-9500`, the Palworld Chillet Egg
//! 6142), `Player.OpenLockBox` (`:9330-9365`, the Golden Lock Box
//! 3085) + `Player.OpenShadowLockbox` (`:9310-9328`, the Obsidian
//! Lockbox 4879) — the remaining `ItemID.Sets.OpenableBag` kinds
//! (`ItemID.cs:1384`) after the boss bags (`boss_bags.rs`), the
//! fishing crates (`fishing_crates.rs`) and the presents
//! (`presents.rs`). The dispatch order is
//! `ItemSlot.TryOpenContainer_GrantItems` (`UI/ItemSlot.cs:1806-1865`).
//!
//! The two lock boxes carry PRE-GATES in the dispatch (not in the
//! openers): the Lock Box consumes a Golden Key 327
//! (`ConsumeItem(327, reverseOrder: false, includeVoidBag: true)`,
//! `:1838`) and REFUSES the open when none is found; the Shadow
//! Lockbox needs a Shadow Key 329 in the inventory or the open void
//! bag (`HasItemInInventoryOrOpenVoidBag(329)`, `:1846`) — NOT
//! consumed. The wiring owns both gates; this module ports the grant
//! trees only.
//!
//! ## The RNG contract
//!
//! Same as the sibling families: every draw rides `Main.rand` (the
//! wiring feeds `NamedRngs::global`), every `QuickSpawnItem`
//! interleaves its `Prefix(-1)` draws
//! ([`crate::boss_bags::quick_spawn`]).
//!
//! ## The recorded quirks (kept verbatim)
//!
//! - Both lock-box openers are `while (flag) { flag = false; ... }`
//!   (`:9312-9316`, `:9332-9336`) — degenerate ONE-pass loops; the
//!   port keeps the single pass (no behaviour difference).
//! - The Herb Bag's count is `Next(2,5)` + a 1/3 `++` (`:9370-9374`);
//!   each herb's stack is `Next(2,5)` + a 1/3-gated `Next(1,5)` add
//!   (`:9434-9438`) — the add draw only inside the gate.
//! - The Shadow Lockbox's weapon list swaps 112 → 683 on the remix
//!   seed (`:9318-9321`); the pick is `NextFromList` = one
//!   `Next(5)` (`UnifiedRandom.cs:1213-1216`).
//! - The Lock Box's `Next(7)` switch: 0 and the default BOTH map to
//!   164 (the Handgun) — 164 lands 2/7 (`:9338-9347`); case 4 swaps
//!   157 → 2623 on remix; the getGoodWorld gate (`:9356`) draws its
//!   `Next(5)` ONLY when the seed flag is up.
//! - The Oyster's pearl chain is nested INSIDE the 1/4 gate
//!   (`:9460-9474`): 1/10 pink 4414, else 1/3 black 4413, else white
//!   4412; the Shucked Oyster 4411 ALWAYS grants (`:9475`).
//!
//! Census: `docs/research/184-expert-drop-trees-census.md`.

#![allow(dead_code)] // the clicks.rs dispatch lands with the wiring commit

use crate::boss_bags::{BagGrant, quick_spawn, range};
use crate::prefix::PrefixRandom;

/// The misc-bag kinds this module opens (`ItemID.cs`: HerbBag 3093,
/// CanOfWorms 4345, Oyster 4410, PalworldChilletEgg 6142, LockBox
/// 3085, ObsidianLockbox 4879).
pub fn is_misc_bag(kind: u16) -> bool {
    matches!(kind, 3093 | 4345 | 4410 | 6142 | 3085 | 4879)
}

/// One located carry slot for [`consume_item`]: the 58-slot inventory
/// (the grid 0..50 then the coin rows 50..54 then the ammo rows
/// 54..58 — the C# `inventory[0..58]` order) or the open void bag's
/// 40 slots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CarryLoc {
    Inventory(usize),
    VoidBag(usize),
}

/// `Player.ConsumeItem(type, reverseOrder: false, includeVoidBag)`
/// (`Player.cs:9269-9308`) over a LOCATED slot list: the first
/// `stack > 0 && type` slot in the C# scan order loses one (at 0 it
/// turns to air — the caller applies that via the returned loc);
/// `None` when nothing matched. The `includeVoidBag` half scans the
/// vault ONLY when `use_void_bag` (the caller resolves
/// `Player.useVoidBag`, `:23171-23181`). The wiring passes the slots
/// in the C# order; the reverseOrder arm has no caller here (the
/// LockBox gate passes false, `ItemSlot.cs:1838`).
pub fn consume_item_loc(
    kind: u16,
    inventory: &[crate::item::ItemSlot],
    void_bag: Option<&crate::bank::BankStore>,
) -> Option<CarryLoc> {
    for (i, s) in inventory.iter().enumerate() {
        if s.count > 0 && s.kind == kind {
            return Some(CarryLoc::Inventory(i));
        }
    }
    if let Some(vault) = void_bag {
        for (i, s) in vault.iter().enumerate() {
            if s.count > 0 && s.kind == kind {
                return Some(CarryLoc::VoidBag(i));
            }
        }
    }
    None
}

/// `Player.OpenHerbBag(int bagType)` (`Player.cs:9367-9441`): 2-4
/// herbs (+1 on a 1/3), each a 14-way pick with a 2-4 stack (+1-4 on
/// a 1/3).
pub fn open_herb_bag<R: PrefixRandom + ?Sized>(rng: &mut R, out: &mut Vec<BagGrant>) {
    let mut num = range(rng, 2, 5); // `:9370`
    if rng.next_max(3) == 0 {
        num += 1; // `:9371-9374`
    }
    for _ in 0..num {
        // `:9377-9433` — the 14 herb kinds (the sequential ifs).
        let num2 = [
            313, 314, 315, 317, 316, 318, 2358, 307, 308, 309, 311, 310, 312, 2357,
        ][rng.next_max(14) as usize];
        let mut num3 = range(rng, 2, 5); // `:9434`
        if rng.next_max(3) == 0 {
            num3 += range(rng, 1, 5); // `:9435-9438`
        }
        quick_spawn(rng, out, num2, num3);
    }
}

/// `Player.OpenCanofWorms(int sourceItemType)` (`Player.cs:9443-9455`):
/// 5-8 worms, a 3/10 enchanted-nightcrawler tail, a 1/20 gold-worm pet.
pub fn open_can_of_worms<R: PrefixRandom + ?Sized>(rng: &mut R, out: &mut Vec<BagGrant>) {
    quick_spawn(rng, out, 2002, range(rng, 5, 9)); // `:9446`
    if rng.next_max(10) < 3 {
        quick_spawn(rng, out, 3191, range(rng, 1, 3)); // `:9447-9450`
    }
    if rng.next_max(20) == 0 {
        quick_spawn(rng, out, 2895, 1); // `:9451-9454`
    }
}

/// `Player.OpenOyster(int sourceItemType)` (`Player.cs:9457-9476`):
/// the 1/4-gated pearl chain + the ALWAYS-granted Shucked Oyster.
pub fn open_oyster<R: PrefixRandom + ?Sized>(rng: &mut R, out: &mut Vec<BagGrant>) {
    if rng.next_max(4) == 0 {
        // `:9460-9474` — the nested pearl chain.
        if rng.next_max(10) == 0 {
            quick_spawn(rng, out, 4414, 1); // Pink Pearl
        } else if rng.next_max(3) == 0 {
            quick_spawn(rng, out, 4413, 1); // Black Pearl
        } else {
            quick_spawn(rng, out, 4412, 1); // White Pearl
        }
    }
    quick_spawn(rng, out, 4411, 1); // `:9475` — no gate
}

/// `Player.OpenChilletEgg(int itemType)` (`Player.cs:9491-9500`): the
/// 1/2 Chillet / Chillet Ignis pick.
pub fn open_chillet_egg<R: PrefixRandom + ?Sized>(rng: &mut R, out: &mut Vec<BagGrant>) {
    let mut item = 5665; // `:9494`
    if rng.next_max(2) == 0 {
        item = 5666; // `:9495-9498`
    }
    quick_spawn(rng, out, item, 1);
}

/// `Player.OpenLockBox(int lockboxItemType)` (`Player.cs:9330-9365`):
/// the 7-way dungeon-weapon pick (0/default both 164), the 1/3 Shadow
/// Key, the 1/8 Bone Key, the getGoodWorld-gated 1/5 and the 1/4 tails.
/// The Golden Key CONSUME gate lives in the dispatch
/// (`ItemSlot.cs:1838`), not here.
pub fn open_lock_box<R: PrefixRandom + ?Sized>(
    remix: bool,
    get_good_world: bool,
    rng: &mut R,
    out: &mut Vec<BagGrant>,
) {
    // `:9332-9336` — the degenerate one-pass while.
    // `:9338-9347` — the Next(7) switch (case 4 remix-swaps).
    let item = match rng.next_max(7) {
        1 => 3317, // Valor
        2 => 155,  // Muramasa
        3 => 156,  // Cobalt Shield
        4 => {
            if !remix {
                157 // Aqua Scepter
            } else {
                2623 // Bubble Gun — the remix swap (`:9343`)
            }
        }
        5 => 163, // Blue Moon
        6 => 113, // Magic Missile (`:9345`)
        _ => 164, // Handgun (0 AND the default)
    };
    quick_spawn(rng, out, item, 1);
    if rng.next_max(3) == 0 {
        quick_spawn(rng, out, 329, 1); // Shadow Key (`:9348-9351`)
    }
    if rng.next_max(8) == 0 {
        quick_spawn(rng, out, 5465, 1); // Dead Cells Ram Rune (`:9352-9355`)
    }
    // `:9356-9359` — the getGoodWorld gate SHORT-CIRCUITS: no draw
    // when the flag is down.
    if get_good_world && rng.next_max(5) == 0 {
        quick_spawn(rng, out, 5515, 1); // Moon Lord Body (the ftw vanity)
    }
    if rng.next_max(4) == 0 {
        quick_spawn(rng, out, 6156, 1); // Silver Bracer (`:9360-9363`)
    }
}

/// `Player.OpenShadowLockbox(int boxType)` (`Player.cs:9310-9328`):
/// the 5-way underworld-weapon pick (the remix list swaps 112 → 683),
/// the 1/5 tail. The Shadow Key PRESENCE gate lives in the dispatch
/// (`ItemSlot.cs:1846`), not here.
pub fn open_shadow_lockbox<R: PrefixRandom + ?Sized>(
    remix: bool,
    rng: &mut R,
    out: &mut Vec<BagGrant>,
) {
    // `:9314-9316` — the degenerate one-pass while.
    // `:9317-9321` — NextFromList = one Next(5) over the list.
    let item = if remix {
        [274, 220, 683, 218, 3019][rng.next_max(5) as usize]
    } else {
        [274, 220, 112, 218, 3019][rng.next_max(5) as usize]
    };
    quick_spawn(rng, out, item, 1);
    if rng.next_max(5) == 0 {
        quick_spawn(rng, out, 5010, 1); // Treasure Magnet (`:9323-9326`)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use tr_core::UnifiedRandom;

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
    fn w4_f35_the_misc_bag_set_is_the_six_id_table() {
        for k in [3093u16, 4345, 4410, 6142, 3085, 4879] {
            assert!(is_misc_bag(k), "{k}");
        }
        for k in [0u16, 3086, 1774, 1869, 2334, 3318, 3092, 4344, 4411] {
            assert!(!is_misc_bag(k), "{k} not a misc bag");
        }
    }

    #[test]
    fn w4_f35_herb_bag_zero_script_grants_three_dayblooms() {
        // The count range(2,5)→3 at zero = 2, the 1/3 bump HITS → 3;
        // each herb: Next(14)=0 → 313 (Daybloom), stack range(2,5)=2,
        // the 1/3 add gate HITS → +range(1,5)=1 → 3. Herbs take no
        // prefix (materials).
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        open_herb_bag(&mut rng, &mut out);
        assert_eq!(items(&out), vec![313, 313, 313]);
        assert_eq!(out.iter().map(|g| g.stack).collect::<Vec<_>>(), vec![3, 3, 3]);
        assert_eq!(
            rng.ranges,
            vec![3, 3, 14, 3, 3, 4, 14, 3, 3, 4, 14, 3, 3, 4],
            "the count + bump + three (pick, stack, add-gate, add) rounds"
        );
    }

    #[test]
    fn w4_f35_herb_bag_the_add_draw_only_inside_the_gate() {
        // Script the add gate to MISS (1): no range(1,5) draw follows.
        let mut rng = ScriptRng::new(&[
            2, 1, // count range→4, bump miss → 4 herbs
            0, 0, 1, // herb 1: 313, stack 2, gate miss (no add draw)
            1, 1, 1, // herb 2: 314, stack 3, gate miss
            2, 2, 1, // herb 3: 315, stack 4, gate miss
            3, 0, 1, // herb 4: 317, stack 2, gate miss
        ]);
        let mut out = Vec::new();
        open_herb_bag(&mut rng, &mut out);
        assert_eq!(items(&out), vec![313, 314, 315, 317]);
        assert_eq!(out.iter().map(|g| g.stack).collect::<Vec<_>>(), vec![2, 3, 4, 2]);
        assert_eq!(rng.ranges, vec![3, 3, 14, 3, 3, 14, 3, 3, 14, 3, 3, 14, 3, 3]);
    }

    #[test]
    fn w4_f35_can_of_worms_zero_script_grants_all_three_arms() {
        // 5 worms (range(5,9) min), the Next(10)=0 < 3 → 1 nightcrawler
        // (range(1,3) min), the 1/20 hits → the Gold Worm 2895.
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        open_can_of_worms(&mut rng, &mut out);
        assert_eq!(items(&out), vec![2002, 3191, 2895]);
        assert_eq!(out.iter().map(|g| g.stack).collect::<Vec<_>>(), vec![5, 1, 1]);
        assert_eq!(rng.ranges, vec![4, 10, 2, 20]);
    }

    #[test]
    fn w4_f35_can_of_worms_the_three_in_ten_gate_boundary() {
        // Next(10)=2 < 3 hits; =3 misses (no stack draw).
        let mut hit = ScriptRng::new(&[0, 2]);
        let mut out = Vec::new();
        open_can_of_worms(&mut hit, &mut out);
        assert_eq!(items(&out), vec![2002, 3191]);
        let mut miss = ScriptRng::new(&[0, 3, 1]);
        let mut out2 = Vec::new();
        open_can_of_worms(&mut miss, &mut out2);
        assert_eq!(items(&out2), vec![2002]);
        assert_eq!(miss.ranges, vec![4, 10, 20], "the miss draws no stack");
    }

    #[test]
    fn w4_f35_oyster_zero_script_grants_the_pink_pearl_and_the_shuck() {
        // The 1/4 hits, the 1/10 hits → 4414 (Pink Pearl); the 4411
        // always grants.
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        open_oyster(&mut rng, &mut out);
        assert_eq!(items(&out), vec![4414, 4411]);
        assert_eq!(rng.ranges, vec![4, 10]);
    }

    #[test]
    fn w4_f35_oyster_the_nested_pearl_chain_and_the_miss() {
        // 1/4 hit + 1/10 miss + 1/3 hit → 4413 (Black Pearl);
        // 1/4 hit + both miss → 4412 (White); 1/4 miss → only 4411.
        let mut black = ScriptRng::new(&[0, 1, 0]);
        let mut o1 = Vec::new();
        open_oyster(&mut black, &mut o1);
        assert_eq!(items(&o1), vec![4413, 4411]);
        let mut white = ScriptRng::new(&[0, 1, 1]);
        let mut o2 = Vec::new();
        open_oyster(&mut white, &mut o2);
        assert_eq!(items(&o2), vec![4412, 4411]);
        let mut none = ScriptRng::new(&[1]);
        let mut o3 = Vec::new();
        open_oyster(&mut none, &mut o3);
        assert_eq!(items(&o3), vec![4411]);
        assert_eq!(none.ranges, vec![4], "the miss draws nothing nested");
    }

    #[test]
    fn w4_f35_chillet_egg_the_one_half_pick() {
        // Next(2)=0 → 5666 (Ignis); =1 → 5665 (the default).
        let mut ignis = ScriptRng::new(&[0]);
        let mut o1 = Vec::new();
        open_chillet_egg(&mut ignis, &mut o1);
        assert_eq!(items(&o1), vec![5666]);
        let mut plain = ScriptRng::new(&[1]);
        let mut o2 = Vec::new();
        open_chillet_egg(&mut plain, &mut o2);
        assert_eq!(items(&o2), vec![5665]);
    }

    #[test]
    fn w4_f35_lock_box_zero_script_grants_the_handgun_and_all_tails() {
        // Next(7)=0 → 164 (Handgun — a GUN: one Next(4) prefix gate);
        // the 1/3 Shadow Key hits, the 1/8 Ram Rune hits (an ACCESSORY
        // — one Next(4)), the ftw gate OFF draws nothing, the 1/4
        // Silver Bracer hits (an accessory — one Next(4)).
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        open_lock_box(false, false, &mut rng, &mut out);
        assert_eq!(items(&out), vec![164, 329, 5465, 6156]);
        assert_eq!(rng.ranges, vec![7, 4, 3, 8, 4, 4, 4]);
    }

    #[test]
    fn w4_f35_lock_box_the_ftw_gate_draws_only_when_up() {
        // The ftw flag up: the Next(5) draws + hits at zero → 5515
        // (a vanity item — no prefix draws).
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        open_lock_box(false, true, &mut rng, &mut out);
        assert_eq!(items(&out), vec![164, 329, 5465, 5515, 6156]);
        assert_eq!(rng.ranges, vec![7, 4, 3, 8, 4, 5, 4, 4]);
    }

    #[test]
    fn w4_f35_lock_box_the_seven_way_pick_and_the_remix_swap() {
        // Each Next(7) value maps its weapon; case 4 remix-swaps
        // 157 → 2623. All later gates scripted to miss.
        for (pick, want) in [
            (0i32, 164u16),
            (1, 3317),
            (2, 155),
            (3, 156),
            (4, 157),
            (5, 163),
            (6, 113),
        ] {
            let mut rng = ScriptRng::new(&[pick, 1, 1, 1]);
            let mut out = Vec::new();
            open_lock_box(false, false, &mut rng, &mut out);
            assert_eq!(items(&out), vec![want], "pick {pick}");
        }
        let mut remix = ScriptRng::new(&[4, 1, 1, 1]);
        let mut out = Vec::new();
        open_lock_box(true, false, &mut remix, &mut out);
        assert_eq!(items(&out), vec![2623], "the remix case-4 swap");
    }

    #[test]
    fn w4_f35_shadow_lockbox_zero_script_grants_the_dark_lance_and_magnet() {
        // Next(5)=0 → 274 (Dark Lance — a SPEAR: one Next(4) prefix
        // gate); the 1/5 Treasure Magnet hits (an accessory — one
        // Next(4)).
        let mut rng = ScriptRng::new(&[]);
        let mut out = Vec::new();
        open_shadow_lockbox(false, &mut rng, &mut out);
        assert_eq!(items(&out), vec![274, 5010]);
        assert_eq!(rng.ranges, vec![5, 4, 5, 4]);
    }

    #[test]
    fn w4_f35_shadow_lockbox_the_remix_list_swaps_the_flamelash() {
        // The classic list index 2 = 112 (Flamelash); the remix list
        // index 2 = 683 (Unholy Trident). The other four unchanged.
        for (pick, classic, remix_want) in [
            (0i32, 274u16, 274u16),
            (1, 220, 220),
            (2, 112, 683),
            (3, 218, 218),
            (4, 3019, 3019),
        ] {
            let mut c = ScriptRng::new(&[pick, 1]);
            let mut o1 = Vec::new();
            open_shadow_lockbox(false, &mut c, &mut o1);
            assert_eq!(o1[0].item, classic, "classic pick {pick}");
            let mut m = ScriptRng::new(&[pick, 1]);
            let mut o2 = Vec::new();
            open_shadow_lockbox(true, &mut m, &mut o2);
            assert_eq!(o2[0].item, remix_want, "remix pick {pick}");
        }
    }

    /// The Herb Bag's C# replay (`Player.cs:9367-9441`).
    fn herb_replay(seed: i32) -> (Vec<BagGrant>, UnifiedRandom) {
        let mut r = UnifiedRandom::new(seed);
        let mut w = Vec::new();
        let mut num = r.next_range(2, 5);
        if r.next_max(3) == 0 {
            num += 1;
        }
        for _ in 0..num {
            let n2 = match r.next_max(14) {
                0 => 313,
                1 => 314,
                2 => 315,
                3 => 317,
                4 => 316,
                5 => 318,
                6 => 2358,
                7 => 307,
                8 => 308,
                9 => 309,
                10 => 311,
                11 => 310,
                12 => 312,
                _ => 2357,
            };
            let mut n3 = r.next_range(2, 5);
            if r.next_max(3) == 0 {
                n3 += r.next_range(1, 5);
            }
            spawn_replay(&mut r, &mut w, n2, n3);
        }
        (w, r)
    }

    /// The Lock Box's C# replay (`Player.cs:9330-9365`).
    fn lockbox_replay(seed: i32, remix: bool, ftw: bool) -> (Vec<BagGrant>, UnifiedRandom) {
        let mut r = UnifiedRandom::new(seed);
        let mut w = Vec::new();
        let item = match r.next_max(7) {
            1 => 3317,
            2 => 155,
            3 => 156,
            4 => {
                if !remix {
                    157
                } else {
                    2623
                }
            }
            5 => 163,
            6 => 113,
            _ => 164,
        };
        spawn_replay(&mut r, &mut w, item, 1);
        if r.next_max(3) == 0 {
            spawn_replay(&mut r, &mut w, 329, 1);
        }
        if r.next_max(8) == 0 {
            spawn_replay(&mut r, &mut w, 5465, 1);
        }
        if ftw && r.next_max(5) == 0 {
            spawn_replay(&mut r, &mut w, 5515, 1);
        }
        if r.next_max(4) == 0 {
            spawn_replay(&mut r, &mut w, 6156, 1);
        }
        (w, r)
    }

    /// The Shadow Lockbox's C# replay (`Player.cs:9310-9328`).
    fn shadow_replay(seed: i32, remix: bool) -> (Vec<BagGrant>, UnifiedRandom) {
        let mut r = UnifiedRandom::new(seed);
        let mut w = Vec::new();
        let item = if remix {
            [274, 220, 683, 218, 3019][r.next_max(5) as usize]
        } else {
            [274, 220, 112, 218, 3019][r.next_max(5) as usize]
        };
        spawn_replay(&mut r, &mut w, item, 1);
        if r.next_max(5) == 0 {
            spawn_replay(&mut r, &mut w, 5010, 1);
        }
        (w, r)
    }

    #[test]
    fn w4_f35_the_misc_openers_replay_the_csharp_draw_order() {
        for seed in [1i32, 7, 42, 1234, 99999, 314159] {
            // The herb bag.
            let mut rng = UnifiedRandom::new(seed);
            let mut out = Vec::new();
            open_herb_bag(&mut rng, &mut out);
            let (want, r2) = herb_replay(seed);
            assert_eq!(out, want, "herb seed {seed}");
            assert_eq!(rng, r2, "herb seed {seed} stream");
            // The can of worms.
            let mut rng = UnifiedRandom::new(seed);
            let mut out = Vec::new();
            open_can_of_worms(&mut rng, &mut out);
            let mut r2 = UnifiedRandom::new(seed);
            let mut want = Vec::new();
            spawn_replay(&mut r2, &mut want, 2002, r2.next_range(5, 9));
            if r2.next_max(10) < 3 {
                spawn_replay(&mut r2, &mut want, 3191, r2.next_range(1, 3));
            }
            if r2.next_max(20) == 0 {
                spawn_replay(&mut r2, &mut want, 2895, 1);
            }
            assert_eq!(out, want, "worms seed {seed}");
            assert_eq!(rng, r2, "worms seed {seed} stream");
            // The oyster.
            let mut rng = UnifiedRandom::new(seed);
            let mut out = Vec::new();
            open_oyster(&mut rng, &mut out);
            let mut r2 = UnifiedRandom::new(seed);
            let mut want = Vec::new();
            if r2.next_max(4) == 0 {
                if r2.next_max(10) == 0 {
                    spawn_replay(&mut r2, &mut want, 4414, 1);
                } else if r2.next_max(3) == 0 {
                    spawn_replay(&mut r2, &mut want, 4413, 1);
                } else {
                    spawn_replay(&mut r2, &mut want, 4412, 1);
                }
            }
            spawn_replay(&mut r2, &mut want, 4411, 1);
            assert_eq!(out, want, "oyster seed {seed}");
            assert_eq!(rng, r2, "oyster seed {seed} stream");
            // The chillet egg.
            let mut rng = UnifiedRandom::new(seed);
            let mut out = Vec::new();
            open_chillet_egg(&mut rng, &mut out);
            let mut r2 = UnifiedRandom::new(seed);
            let mut want = Vec::new();
            let mut item = 5665;
            if r2.next_max(2) == 0 {
                item = 5666;
            }
            spawn_replay(&mut r2, &mut want, item, 1);
            assert_eq!(out, want, "chillet seed {seed}");
            assert_eq!(rng, r2, "chillet seed {seed} stream");
            // The two lock boxes × the seed flags.
            for (remix, ftw) in [(false, false), (true, false), (false, true), (true, true)] {
                let mut rng = UnifiedRandom::new(seed);
                let mut out = Vec::new();
                open_lock_box(remix, ftw, &mut rng, &mut out);
                let (want, r2) = lockbox_replay(seed, remix, ftw);
                assert_eq!(out, want, "lockbox seed {seed} remix {remix} ftw {ftw}");
                assert_eq!(rng, r2, "lockbox seed {seed} remix {remix} ftw {ftw} stream");
                let mut rng = UnifiedRandom::new(seed);
                let mut out = Vec::new();
                open_shadow_lockbox(remix, &mut rng, &mut out);
                let (want, r2) = shadow_replay(seed, remix);
                assert_eq!(out, want, "shadow seed {seed} remix {remix}");
                assert_eq!(rng, r2, "shadow seed {seed} remix {remix} stream");
            }
        }
    }
}
