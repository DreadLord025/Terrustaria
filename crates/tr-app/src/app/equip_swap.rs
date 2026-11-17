//! `ItemSlot`'s right-click equip ladder and the swap helpers it drives —
//! `SwapEquip` (`ItemSlot.cs:3283-3363`), `ArmorSwap` (`:3569-3638`),
//! `DyeSwap` (`:3540-3567`), `EquipSwap` (`:3640-3650`) and
//! `SwapVanityEquip` (`:1867-1899`), over the port's split inventory
//! (`App::equip` = `crate::inv::EquipSlots`).
//!
//! Every function here is pure: the click dispatch in `app::clicks` applies the
//! returned target and owns the sound cue, `Main.EquipPageSelected` and the
//! positional favourite clear. The deviations are the ones already recorded for
//! `CanEquipInArmorSlot` (docs/ANALYSIS.md): `Item.wingSlot` is not carried, so
//! `CanEquipBothAccessories`' two-wings arm is not modelled.

use crate::buffs;
use crate::inv::{ARMOR, DYE, MISC};
use crate::item::ItemSlot;
use crate::registry;
use crate::sortdata::{MOUNT_CART, PROJ_HOOK};

/// `ItemID.Sets.AccessoryIncompatibilityType` (`ItemID.cs:338`) — the only
/// non-default rows: `CreateIntSet(-1, 6168, 6168, 6169, 6168, 6194, 6168,
/// 6193, 6168)`.
fn incompatibility_type(kind: u16) -> i32 {
    match kind {
        6168 | 6169 | 6193 | 6194 => 6168,
        _ => -1,
    }
}

/// `ItemSlot.CanEquipBothAccessories` (`ItemSlot.cs:3451-3472`) without the
/// `wingSlot > 0` arm (the port carries no `Item.wingSlot` — ANALYSIS).
fn can_equip_both_accessories(a: &ItemSlot, b: &ItemSlot) -> bool {
    if a.kind == b.kind {
        return false; // `:3453-3456`
    }
    let (ta, tb) = (incompatibility_type(a.kind), incompatibility_type(b.kind));
    // `:3465-3470`
    !(ta != -1 && ta == tb)
}

/// `ItemSlot.HasSameItemInSlot` (`ItemSlot.cs:3508-3529`): the same type, or
/// the same non-default incompatibility class.
fn has_same_item_in_slot(new_item: &ItemSlot, items: &[ItemSlot]) -> bool {
    if new_item.count == 0 {
        return false; // `:3510-3513` (`IsAir`)
    }
    let t = incompatibility_type(new_item.kind);
    items
        .iter()
        .any(|i| i.kind == new_item.kind || (t != -1 && t == incompatibility_type(i.kind)))
}

/// `ItemSlot.CanEquipAccessoryInSlot` (`ItemSlot.cs:3531-3538`): the only
/// refused placement is the first incompatible accessory's slot, which is
/// allowed when it is the slot being replaced. `slot` is relative to
/// `accessories` (the C# compares absolute `Player.armor` indices).
fn can_equip_accessory_in_slot(item: &ItemSlot, accessories: &[ItemSlot], slot: usize) -> bool {
    match accessories
        .iter()
        .position(|a| !can_equip_both_accessories(a, item))
    {
        Some(collision) => collision == slot,
        None => true,
    }
}

/// `Player.IsItemSlotUnlockedAndUsable` (`Player.cs:12579-12604`) with the
/// port's constants: `extraAccessory` (the Demon Heart) and `Main.masterMode`
/// are not modelled — they are the `inventory::DEMON_HEART_SLOT` /
/// `MASTER_SLOT` `false` pair — so 8/9 and their vanity twins 18/19 are
/// unusable and every other slot is.
fn is_item_slot_unlocked_and_usable(slot: usize) -> bool {
    !matches!(slot, 8 | 9 | 18 | 19)
}

/// `Item.CanBeEquipped` (`Item.cs:1159-1174`).
pub(crate) fn can_be_equipped(item: &ItemSlot) -> bool {
    let Some(d) = registry::get(item.kind) else {
        return false;
    };
    let hook = d.projectile.is_some_and(|p| PROJ_HOOK.get(p));
    if d.head_slot < 0
        && d.body_slot < 0
        && d.leg_slot < 0
        && !d.accessory
        && !hook
        && d.mount_type < 0
    {
        if d.buff_type > 0 {
            return buffs::is_vanity_pet(d.buff_type) || buffs::is_light_pet(d.buff_type);
        }
        return false;
    }
    true
}

/// The destination a `SwapEquip` (`ItemSlot.cs:3283-3363`) moves `item` into.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SwapTarget {
    Armor(usize),
    Dye(usize),
    Misc(usize),
}

/// `ItemSlot.SwapEquip`'s arm ladder (`:3291-3358`), in the C# order: dye,
/// grapple, mount, minecart, light pet, vanity pet, then the armour rows.
/// Returns `None` when every arm refuses the item.
pub(crate) fn swap_equip_target(
    item: &ItemSlot,
    armor: &[ItemSlot; ARMOR],
    dye: &[ItemSlot; DYE],
    dye_counter: &mut usize,
) -> Option<SwapTarget> {
    let d = registry::get(item.kind)?;
    if d.dye > 0 {
        return dye_swap_target(item, dye, dye_counter).map(SwapTarget::Dye);
    }
    // `:3301` — `Main.projHook[inv[slot].shoot]` -> `miscEquips[4]`.
    if d.projectile.is_some_and(|p| PROJ_HOOK.get(p)) {
        return Some(SwapTarget::Misc(4));
    }
    // `:3311-3329` — the mount / minecart split.
    if d.mount_type != -1 {
        let slot = if MOUNT_CART.get(d.mount_type as u16) {
            2
        } else {
            3
        };
        return Some(SwapTarget::Misc(slot));
    }
    // `:3330-3347` — light pet then vanity pet.
    if d.buff_type > 0 && buffs::is_light_pet(d.buff_type) {
        return Some(SwapTarget::Misc(1));
    }
    if d.buff_type > 0 && buffs::is_vanity_pet(d.buff_type) {
        return Some(SwapTarget::Misc(0));
    }
    armor_swap_target(item, armor).map(SwapTarget::Armor)
}

/// `ItemSlot.ArmorSwap` (`ItemSlot.cs:3569-3638`) — the `Player.armor` slot the
/// item wants, or `None` for the C#'s "return the item unchanged".
pub(crate) fn armor_swap_target(item: &ItemSlot, armor: &[ItemSlot; ARMOR]) -> Option<usize> {
    if item.count < 1 {
        return None; // `:3573-3576`
    }
    let d = registry::get(item.kind)?;
    let (head, body, leg) = (d.head_slot != -1, d.body_slot != -1, d.leg_slot != -1);
    if !head && !body && !leg && !d.accessory {
        return None; // `:3577-3580`
    }
    if head || body || leg {
        // `:3582-3588` — the real AND the vanity set are both checked.
        if has_same_item_in_slot(item, &armor[0..3]) || has_same_item_in_slot(item, &armor[10..13])
        {
            return None;
        }
    } else if d.accessory {
        // `:3589-3592`.
        let worn = if d.vanity {
            &armor[3..10]
        } else {
            &armor[13..20]
        };
        if has_same_item_in_slot(item, worn) {
            return None;
        }
    }
    let offset = if d.vanity { 10 } else { 0 }; // `:3593`
    if head {
        return Some(offset); // `:3594-3597`
    }
    if body {
        return Some(offset + 1);
    }
    if leg {
        return Some(offset + 2);
    }
    // `:3606-3629` — the accessory scan over `armor[3+offset .. 3+offset+7]`.
    let start = 3 + offset;
    let row = &armor[start..start + 7];
    if let Some(collision) = row
        .iter()
        .position(|a| !can_equip_both_accessories(a, item))
    {
        let slot = start + collision;
        return is_item_slot_unlocked_and_usable(slot).then_some(slot); // `:3611-3614`
    }
    let mut target = start; // `:3618`
    for i in 0..7 {
        let slot = start + i;
        if is_item_slot_unlocked_and_usable(slot) && row[i].count == 0 {
            target = slot;
            break;
        }
    }
    Some(target)
}

/// `ItemSlot.DyeSwap` (`ItemSlot.cs:3540-3567`) — the `Player.dye` index, and
/// the round-robin counter advance (`ItemSlot.dyeSwapCounter`, `:3549-3564`).
pub(crate) fn dye_swap_target(
    item: &ItemSlot,
    dye: &[ItemSlot; DYE],
    counter: &mut usize,
) -> Option<usize> {
    let d = registry::get(item.kind)?;
    if d.dye <= 0 {
        return None; // `:3544-3547`
    }
    while !is_item_slot_unlocked_and_usable(*counter) {
        // `:3549-3552`
        *counter = (*counter + 1) % DYE;
    }
    for (i, cell) in dye.iter().enumerate() {
        // `:3553-3560`
        if is_item_slot_unlocked_and_usable(i) && cell.count == 0 {
            *counter = i;
            break;
        }
    }
    let target = *counter;
    *counter = (*counter + 1) % DYE; // `:3564`
    Some(target)
}

/// `ItemSlot.SwapVanityEquip`'s guard chain (`ItemSlot.cs:1874-1877`) for the
/// vanity row `slot` (`10..19`) against its real counterpart `slot - 10`.
pub(crate) fn vanity_swap_allowed(
    item: &ItemSlot,
    real: &ItemSlot,
    context: i32,
    slot: usize,
    armor: &[ItemSlot; ARMOR],
) -> bool {
    if item.count == 0 && real.count == 0 {
        return false;
    }
    if context == 11 {
        // `:1874` — the two accessory-compatibility legs, with the compared
        // slot the one being replaced (relative index `slot - 13`).
        let rel = slot.saturating_sub(13);
        if item.count > 0 && !can_equip_accessory_in_slot(item, &armor[3..10], rel) {
            return false;
        }
        if real.count > 0 && !can_equip_accessory_in_slot(real, &armor[13..20], rel) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::EMPTY_SLOT;

    fn slot(kind: u16, count: u32) -> ItemSlot {
        ItemSlot {
            kind,
            name: registry::name(kind),
            count,
            prefix: 0,
        }
    }

    fn armor() -> [ItemSlot; ARMOR] {
        [EMPTY_SLOT; ARMOR]
    }

    /// `CanBeEquipped` (`Item.cs:1159-1174`): the armour/pet/hook/mount rows.
    #[test]
    fn can_be_equipped_reads_the_flag_groups() {
        assert!(can_be_equipped(&slot(89, 1)), "Copper Helmet");
        assert!(can_be_equipped(&slot(80, 1)), "Copper Chainmail");
        assert!(can_be_equipped(&slot(15, 1)), "Copper Watch (accessory)");
        assert!(!can_be_equipped(&slot(4, 1)), "Iron Broadsword");
        assert!(!can_be_equipped(&slot(2, 999)), "Dirt Block");
        assert!(!can_be_equipped(&EMPTY_SLOT));
    }

    /// `ArmorSwap` (`ItemSlot.cs:3593-3605`): the three armour rows land on
    /// their own slot, a vanity set offset by 10.
    #[test]
    fn armor_swap_targets_the_matching_armor_row() {
        let a = armor();
        assert_eq!(
            armor_swap_target(&slot(89, 1), &a),
            Some(0),
            "Copper Helmet"
        );
        assert_eq!(
            armor_swap_target(&slot(80, 1), &a),
            Some(1),
            "Copper Chainmail"
        );
        assert_eq!(
            armor_swap_target(&slot(76, 1), &a),
            Some(2),
            "Copper Greaves"
        );
        // 237 Sunglasses is `vanity` + `headSlot 12` -> `armor[10]` (`:3593`).
        assert_eq!(armor_swap_target(&slot(237, 1), &a), Some(10));
        // A non-wearable and an air stack both answer `None` (`:3573-3580`).
        assert_eq!(armor_swap_target(&slot(4, 1), &a), None);
        assert_eq!(armor_swap_target(&EMPTY_SLOT, &a), None);
    }

    /// `ArmorSwap`'s duplicate guards (`:3582-3592`): the real and the vanity
    /// set are both checked for armour, and an accessory checks the C#'s
    /// `item.vanity ? AccSlots : VanityAccSlots` — a *real* accessory is
    /// compared against the vanity column and vice versa, so the same
    /// accessory may be worn once for show and once for stats.
    #[test]
    fn armor_swap_refuses_a_duplicate_piece() {
        let mut a = armor();
        a[0] = slot(89, 1);
        assert_eq!(
            armor_swap_target(&slot(89, 1), &a),
            None,
            "`:3584` real set"
        );
        // The vanity helmet lives in `armor[10]` and blocks too (`:3584`).
        let mut a = armor();
        a[10] = slot(89, 1);
        assert_eq!(
            armor_swap_target(&slot(89, 1), &a),
            None,
            "`:3584` vanity set"
        );
        // A real accessory checks `VanityAccSlots` = `armor[13..20]` (`:3589`).
        let mut a = armor();
        a[13] = slot(15, 1);
        assert_eq!(armor_swap_target(&slot(15, 1), &a), None);
        // The real column does not block it — the collision scan lands on the
        // occupied slot itself and the slot is usable (`:3611`/`:3622`).
        let mut a = armor();
        a[3] = slot(15, 1);
        assert_eq!(armor_swap_target(&slot(15, 1), &a), Some(3));
        // A vanity accessory (4822) reads `AccSlots` (`:3589`).
        let mut a = armor();
        a[3] = slot(4822, 1);
        assert_eq!(armor_swap_target(&slot(4822, 1), &a), None);
        // ... and targets `armor[13..20]` (`:3593`).
        let mut a = armor();
        assert_eq!(armor_swap_target(&slot(4822, 1), &a), Some(13));
    }

    /// `ArmorSwap`'s accessory scan (`:3606-3629`): the first unlocked empty
    /// row from `armor[3]`, and 8/9 skipped by `IsItemSlotUnlockedAndUsable`.
    #[test]
    fn armor_swap_scans_the_accessory_rows() {
        // An occupied first row sends it to the next empty one.
        let mut a = armor();
        a[3] = slot(16, 1);
        assert_eq!(armor_swap_target(&slot(15, 1), &a), Some(4));
        // Rows 3..=5 occupied -> row 6.
        for i in 3..6 {
            a[i] = slot(16 + i as u16, 1);
        }
        assert_eq!(armor_swap_target(&slot(15, 1), &a), Some(6));
        // Rows 3..=7 full: the `targetSlot = accessories.Offset` fallback
        // (`:3618`) keeps the first row — matching the C#'s full-row case.
        let mut a = armor();
        for i in 3..8 {
            a[i] = slot(16 + i as u16, 1);
        }
        assert_eq!(armor_swap_target(&slot(15, 1), &a), Some(3));
    }

    /// `DyeSwap` (`ItemSlot.cs:3553-3564`): the counter's slot is reused when
    /// it is empty, else the first empty row wins and the counter advances.
    #[test]
    fn dye_swap_round_robins() {
        let mut dye = [EMPTY_SLOT; DYE];
        let mut counter = 0usize;
        assert_eq!(dye_swap_target(&slot(1007, 1), &dye, &mut counter), Some(0));
        assert_eq!(counter, 1, "`:3564` advances past the used slot");
        // The scan restarts from the counter, so a filled slot 0 sends the next
        // dye to slot 1.
        dye[0] = slot(1007, 1);
        assert_eq!(dye_swap_target(&slot(2871, 1), &dye, &mut counter), Some(1));
        // A full row keeps the counter's slot (`:3553-3560` finds nothing).
        let full = [slot(1007, 1); DYE];
        let mut counter = 4usize;
        assert_eq!(
            dye_swap_target(&slot(1007, 1), &full, &mut counter),
            Some(4)
        );
        assert_eq!(counter, 5);
        // A non-dye refuses (`:3544-3547`).
        let mut counter = 0usize;
        assert_eq!(dye_swap_target(&slot(4, 1), &dye, &mut counter), None);
    }

    /// `SwapEquip`'s ladder (`ItemSlot.cs:3291-3347`): the misc rows.
    #[test]
    fn swap_equip_ladder_picks_the_misc_row() {
        let (a, dye) = (armor(), [EMPTY_SLOT; DYE]);
        let mut counter = 0usize;
        // 1007 is a dye -> the dye column.
        assert_eq!(
            swap_equip_target(&slot(1007, 1), &a, &dye, &mut counter),
            Some(SwapTarget::Dye(0))
        );
    }

    /// `SwapVanityEquip` (`ItemSlot.cs:1874-1877`): both air refuses, one side
    /// filled swaps.
    #[test]
    fn vanity_swap_needs_an_item_on_either_side() {
        let a = armor();
        assert!(!vanity_swap_allowed(&EMPTY_SLOT, &EMPTY_SLOT, 9, 10, &a));
        assert!(vanity_swap_allowed(&slot(89, 1), &EMPTY_SLOT, 9, 10, &a));
        assert!(vanity_swap_allowed(&EMPTY_SLOT, &slot(89, 1), 9, 10, &a));
    }

    /// `CanEquipBothAccessories` (`:3451-3472`): the same item never doubles,
    /// and the 6168 incompatibility class collides.
    #[test]
    fn accessory_compatibility_pairs() {
        assert!(!can_equip_both_accessories(&slot(15, 1), &slot(15, 1)));
        assert!(can_equip_both_accessories(&slot(15, 1), &slot(16, 1)));
        assert!(!can_equip_both_accessories(&slot(6168, 1), &slot(6169, 1)));
        assert!(can_equip_both_accessories(&slot(6168, 1), &slot(16, 1)));
    }

    /// `CanEquipAccessoryInSlot` allows the collision slot itself (`:3531-3538`).
    #[test]
    fn accessory_collision_allows_the_replaced_slot() {
        let mut acc = [EMPTY_SLOT; 7];
        acc[2] = slot(15, 1);
        assert!(can_equip_accessory_in_slot(&slot(15, 1), &acc, 2));
        assert!(!can_equip_accessory_in_slot(&slot(15, 1), &acc, 3));
        assert!(can_equip_accessory_in_slot(&slot(16, 1), &acc, 3));
    }
}
