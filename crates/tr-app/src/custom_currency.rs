//! The custom-currency wallet — `Terraria.GameContent.UI.CustomCurrencyManager`
//! / `CustomCurrencySystem` / `CustomCurrencySingleCoin` (W4-d122 shop
//! currency remainder).
//!
//! Census (1.4.5.8): `CustomCurrencyManager.Initialize`
//! (`CustomCurrencyManager.cs:14-18`) registers EXACTLY ONE currency —
//! `CustomCurrencyID.DefenderMedals = RegisterCurrency(new
//! CustomCurrencySingleCoin(3817, 999L))` — so the only special currency is
//! the Defender Medal, item **3817** (`ItemID.DefenderMedal`), worth 1 per
//! unit, capped at 999 per container AND on the combined total. The Elder/
//! Eternia Crystal **3822** is NOT a currency (it is the sentry ammo,
//! `Player.PayDD2CrystalsBeforeUse` — `world_sim/editing/sentry.rs`); the
//! golf balls are plain coin-priced goods (`Chest.cs:3156-3262`, case 22 —
//! no `shopSpecialCurrency` stamp anywhere outside the Tavernkeep's case 21,
//! `Chest.cs:2987-3150`).
//!
//! The buy chain: `Player.BuyItem(price, customCurrency)`
//! (`Player.cs:35400-35405`) routes a non-(-1) currency to
//! `CustomCurrencyManager.BuyItem` (`CustomCurrencyManager.cs:69-119`):
//! count the player inventory IGNORING slots {58, 57, 56, 55, 54} (the coin
//! slots — `:72`/`:98`) plus bank/bank2/bank3/bank4 (`:73-76`), combine the
//! five capped counts (`:77`), refuse when short, then spend through
//! `CustomCurrencySingleCoin.TryPurchasing` (`CustomCurrencySingleCoin.cs:
//! 26-74`): a full item-cache snapshot, consume min(stack, remaining) per
//! accepted slot in inventory→bank→bank2→bank3→bank4 slot order, and
//! RESTORE the whole cache when the price is not fully covered (`:68-72`).
//!
//! RNG census: NONE of `CountCurrency` / `CombineStacks` / `TryPurchasing` /
//! `BuyItem` draws `Main.rand` — the wallet is pure arithmetic over the slot
//! arrays (the medal DROP side's gravity draws are W4-f36's slice,
//! `DD2Event.cs:802-813` → `Item.cs:49749-49750`, already landed fac5f222).
//!
//! Port shape: the C# passes `List<Item[]>` + `Point` slot lists; the port
//! passes the five fixed slot arrays in the same order (index 0 = the
//! player inventory, 1..4 = the banks — `bank.rs`'s `banks[0..4]` resolver
//! order, `Player.cs:33492-33502`). The base `CustomCurrencySystem.
//! TryPurchasing`'s coin-denomination ladder (`CustomCurrencySystem.cs:
//! 65-216`) is NOT ported: the only registered currency OVERRIDES it with
//! the single-coin walk, and the plain-coin purse rides the recorded
//! aggregate `App.coins` model (ANALYSIS "the purse"). The savings-draw /
//! price-text chrome (`DrawSavings`, `GetPriceText`) is the UI follow-up.

use crate::item::{ItemSlot, EMPTY_SLOT};

/// `CustomCurrencyID.None` (`CustomCurrencyID.cs:5`) — the plain coin purse.
pub(crate) const CURRENCY_NONE: i32 = -1;
/// `CustomCurrencyID.DefenderMedals` after `Initialize`
/// (`CustomCurrencyManager.cs:17` — the FIRST registration, index 0).
pub(crate) const DEFENDER_MEDALS: i32 = 0;
/// The Defender Medal item (`ItemID.DefenderMedal` = 3817,
/// `CustomCurrencyManager.cs:17`'s `CustomCurrencySingleCoin(3817, …)`).
pub(crate) const DEFENDER_MEDAL_ITEM: u16 = 3817;
/// The single coin's cap (`CustomCurrencyManager.cs:17`'s `999L`;
/// `CustomCurrencySystem.cs:12`'s base default 999999999 is overridden).
pub(crate) const MEDAL_CAP: i64 = 999;
/// `CustomCurrencySingleCoin`'s `Include(coinItemID, 1)`
/// (`CustomCurrencySingleCoin.cs:22`) — one medal is worth one unit.
const MEDAL_VALUE: i64 = 1;

/// The inventory slots `BuyItem` ignores (`CustomCurrencyManager.cs:72`'s
/// `58, 57, 56, 55, 54` — the coin/ammo/trash tail of the C#'s 59-slot
/// `Player.inventory`, `Player.cs:1576`). The port's grid is the 50-slot
/// hotbar+storage window (`item.rs::INV`), so none of these indices exists
/// here; the ignore list is kept verbatim for the count/consume scans.
const IGNORED_INV_SLOTS: [usize; 5] = [58, 57, 56, 55, 54];

/// `CustomCurrencySystem.Accepts` (`CustomCurrencySystem.cs:218-221`) for
/// the single registered currency: the `_valuePerUnit` dictionary holds
/// exactly the medal kind.
pub(crate) fn accepts(kind: u16) -> bool {
    kind == DEFENDER_MEDAL_ITEM
}

/// `CustomCurrencySystem.CountCurrency` (`CustomCurrencySystem.cs:26-47`):
/// sum `value * stack` over the accepted slots (the ignore list applies to
/// the inventory only — the C# passes it on the `:72` call alone), clamped
/// at the cap with the overflow flag. The clamp fires MID-WALK (`:38-42`),
/// so a container over the cap returns exactly the cap.
fn count_currency(slots: &[ItemSlot], ignore: &[usize]) -> (i64, bool) {
    let mut num = 0i64;
    for (i, slot) in slots.iter().enumerate() {
        if ignore.contains(&i) {
            continue;
        }
        if accepts(slot.kind) {
            num += MEDAL_VALUE * slot.count as i64;
        }
        if num >= MEDAL_CAP {
            return (MEDAL_CAP, true); // `:38-42`
        }
    }
    (num, false)
}

/// `CustomCurrencySystem.CombineStacks` (`CustomCurrencySystem.cs:49-63`):
/// add the per-container counts in order, clamping at the cap the moment
/// the running total reaches it.
fn combine_stacks(counts: &[i64]) -> (i64, bool) {
    let mut num = 0i64;
    for c in counts {
        num += c;
        if num >= MEDAL_CAP {
            return (MEDAL_CAP, true);
        }
    }
    (num, false)
}

/// `CustomCurrencyManager.BuyItem`'s count half (`:72-77`): the inventory
/// (minus the coin slots) + the four banks, each capped, then combined.
/// `inventories[0]` is the player grid, `[1..5]` the banks in the C#'s
/// bank/bank2/bank3/bank4 order.
pub(crate) fn count_wallet(inventories: [&[ItemSlot]; 5]) -> i64 {
    let mut counts = [0i64; 5];
    for (i, slots) in inventories.iter().enumerate() {
        let ignore: &[usize] = if i == 0 { &IGNORED_INV_SLOTS } else { &[] };
        counts[i] = count_currency(slots, ignore).0;
    }
    combine_stacks(&counts).0
}

/// `CustomCurrencySingleCoin.TryPurchasing` (`CustomCurrencySingleCoin.cs:
/// 26-74`): snapshot EVERY slot of every container (`ItemCacheCreate`,
/// `CustomCurrencySystem.cs:244-255`), then walk the accepted slots in
/// container order / slot order (the C#'s `slotCoins` list is built by
/// `BuyItem`'s `:99-108` scan in exactly that order), consuming
/// `min(stack, remaining)` per slot; a drained slot turns to air
/// (`SetDefaults(0)`, `:40-62`'s empty-slot bookkeeping — the port's
/// `EMPTY_SLOT`). When the price is not fully covered, RESTORE the whole
/// cache (`ItemCacheRestore`, `CustomCurrencySystem.cs:257-263`) and
/// return false — nothing moves on a failed buy.
fn try_purchasing(price: i64, inventories: [&mut [ItemSlot]; 5]) -> bool {
    let mut num = price;
    if num == 0 {
        return true; // `:73` — a zero price covers before the walk
    }
    // `:28` — the full-cache snapshot (the port's slots are Copy).
    let cache: [Vec<ItemSlot>; 5] =
        std::array::from_fn(|i| inventories[i].to_vec());
    for (i, slots) in inventories.iter_mut().enumerate() {
        for (k, slot) in slots.iter_mut().enumerate() {
            // `BuyItem`'s `:103` Accepts scan + the `:98` ignore list.
            if i == 0 && IGNORED_INV_SLOTS.contains(&k) {
                continue;
            }
            if !accepts(slot.kind) {
                continue;
            }
            // `:33-39` — num2 = min(stack, remaining); consume it.
            let num2 = (slot.count as i64).min(num);
            num -= num2;
            slot.count -= num2 as u32;
            if slot.count == 0 {
                *slot = EMPTY_SLOT; // `:40-62`'s SetDefaults(0)
            }
            if num == 0 {
                return true; // `:63-66`'s break → `:73`'s true
            }
        }
    }
    // `:68-72` — the price was not covered: restore every slot.
    for (i, slots) in inventories.iter_mut().enumerate() {
        for (k, slot) in slots.iter_mut().enumerate() {
            *slot = cache[i][k];
        }
    }
    false
}

/// `CustomCurrencyManager.BuyItem` (`CustomCurrencyManager.cs:69-119`):
/// the combined wallet must cover the price (`:77-80`'s refusal), then the
/// single-coin spend (`:114-117`). Returns false without touching any slot
/// when short OR when the spend cannot cover (the restore arm).
pub(crate) fn buy_with_currency(price: i64, mut inventories: [&mut [ItemSlot]; 5]) -> bool {
    // `:72-77` — the count half over immutable views of the same arrays.
    let total = count_wallet(std::array::from_fn(|i| &*inventories[i]));
    if total < price {
        return false;
    }
    try_purchasing(price, inventories)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bank::BANK_SLOTS;
    use crate::item::INV;

    fn medal(count: u32) -> ItemSlot {
        ItemSlot {
            kind: DEFENDER_MEDAL_ITEM,
            name: "Defender Medal",
            count,
            prefix: 0,
        }
    }

    fn empty_wallet() -> [Vec<ItemSlot>; 5] {
        std::array::from_fn(|i| {
            vec![EMPTY_SLOT; if i == 0 { INV } else { BANK_SLOTS }]
        })
    }

    fn views(w: &[Vec<ItemSlot>; 5]) -> [&[ItemSlot]; 5] {
        std::array::from_fn(|i| w[i].as_slice())
    }

    fn mut_views(w: &mut [Vec<ItemSlot>; 5]) -> [&mut [ItemSlot]; 5] {
        std::array::from_fn(|i| w[i].as_mut_slice())
    }

    /// The census pins: the currency item is 3817 (NOT the 3822 crystal),
    /// the id is 0, the cap is 999 (`CustomCurrencyManager.cs:17`).
    #[test]
    fn the_medal_currency_constants_match_the_registration() {
        assert_eq!(DEFENDER_MEDAL_ITEM, 3817);
        assert_eq!(DEFENDER_MEDALS, 0);
        assert_eq!(CURRENCY_NONE, -1);
        assert_eq!(MEDAL_CAP, 999);
        assert!(accepts(3817));
        assert!(!accepts(3822), "the Eternia Crystal is sentry ammo, not money");
        assert!(!accepts(71) && !accepts(74), "the coins are the plain purse");
    }

    /// `CountCurrency` (`:26-47`): the accepted stacks sum, the cap clamps
    /// mid-walk, and the ignore list skips the coin slots.
    #[test]
    fn the_count_sums_clamps_and_ignores_the_coin_slots() {
        let mut inv = vec![EMPTY_SLOT; 59];
        inv[0] = medal(3);
        inv[10] = medal(4);
        inv[54] = medal(500); // an ignored slot — never counted
        assert_eq!(count_currency(&inv, &IGNORED_INV_SLOTS), (7, false));
        // Without the ignore list the 500 stack counts (the banks' call).
        assert_eq!(count_currency(&inv, &[]), (507, false));
        // The cap clamp: 999 exactly trips the overflow flag (`>=`).
        let mut big = vec![EMPTY_SLOT; 59];
        big[0] = medal(999);
        assert_eq!(count_currency(&big, &[]), (999, true));
        big[0] = medal(1500);
        assert_eq!(count_currency(&big, &[]), (999, true));
        // A non-medal kind contributes nothing (`Accepts`).
        let mut other = vec![EMPTY_SLOT; 59];
        other[0] = ItemSlot { kind: 3822, count: 99, ..medal(0) };
        assert_eq!(count_currency(&other, &[]), (0, false));
    }

    /// `CombineStacks` (`:49-63`): the running total clamps at the cap.
    #[test]
    fn the_combine_clamps_the_running_total() {
        assert_eq!(combine_stacks(&[100, 200, 300, 300, 99]), (999, true));
        assert_eq!(combine_stacks(&[100, 200, 300, 300, 98]), (998, false));
        assert_eq!(combine_stacks(&[999, 999]), (999, true));
    }

    /// `BuyItem`'s count half (`:72-77`): inventory + all four banks.
    #[test]
    fn the_wallet_counts_the_inventory_and_every_bank() {
        let mut w = empty_wallet();
        w[0][0] = medal(5);
        w[1][0] = medal(15);
        w[2][39] = medal(60);
        w[3][7] = medal(50);
        w[4][12] = medal(1);
        assert_eq!(count_wallet(views(&w)), 131);
        // Each container caps at 999 before the combine.
        w[1][0] = medal(1500);
        assert_eq!(count_wallet(views(&w)), 999, "the combine clamps too");
    }

    /// The spend walks inventory→bank→bank2→bank3→bank4 in slot order and
    /// drains the touched slots to air (`SingleCoin.TryPurchasing`).
    #[test]
    fn the_spend_consumes_in_container_and_slot_order() {
        let mut w = empty_wallet();
        w[0][3] = medal(2);
        w[0][7] = medal(10);
        w[1][0] = medal(5);
        assert!(buy_with_currency(9, mut_views(&mut w)));
        assert_eq!(w[0][3], EMPTY_SLOT, "the first stack drained to air");
        assert_eq!(w[0][7].count, 3, "7 taken from the second stack");
        assert_eq!(w[1][0].count, 5, "the bank untouched");
        // A cross-container spend: the inventory empties, the bank pays.
        assert!(buy_with_currency(6, mut_views(&mut w)));
        assert_eq!(w[0][7], EMPTY_SLOT);
        assert_eq!(w[1][0].count, 2);
    }

    /// The refusal arms: a short wallet changes NOTHING (`:77-80`), and a
    /// spend that cannot cover restores the full cache (`:68-72`).
    #[test]
    fn the_failed_buy_restores_every_slot() {
        let mut w = empty_wallet();
        w[0][0] = medal(3);
        w[2][5] = medal(4);
        let before = w.clone();
        assert!(!buy_with_currency(8, mut_views(&mut w)), "7 < 8 refuses");
        assert_eq!(w, before, "the short-wallet refusal moves nothing");
        // The exact-price buy succeeds and drains both stacks.
        assert!(buy_with_currency(7, mut_views(&mut w)));
        assert_eq!(w[0][0], EMPTY_SLOT);
        assert_eq!(w[2][5], EMPTY_SLOT);
        // Zero price: nothing consumed, success (`num == 0` before the walk).
        let mut w2 = empty_wallet();
        w2[0][0] = medal(1);
        assert!(buy_with_currency(0, mut_views(&mut w2)));
        assert_eq!(w2[0][0].count, 1);
    }

    /// The cap interaction: 999+ medals across containers still buy at the
    /// COMBINED cap (the C# clamps the count, then spends the real stacks).
    #[test]
    fn the_capped_wallet_buys_at_the_cap() {
        let mut w = empty_wallet();
        w[0][0] = medal(999);
        w[1][0] = medal(999);
        assert_eq!(count_wallet(views(&w)), 999);
        assert!(!buy_with_currency(1000, mut_views(&mut w)));
        assert!(buy_with_currency(999, mut_views(&mut w)));
        assert_eq!(w[0][0], EMPTY_SLOT);
        assert_eq!(w[1][0].count, 999, "the second container untouched");
    }
}
