//! `ItemSlot`'s right-click equip ladder and the swap helpers it drives —
//! `SwapEquip` (`ItemSlot.cs:3283-3363`), `ArmorSwap` (`:3569-3638`),
//! `DyeSwap` (`:3540-3567`), `EquipSwap` (`:3640-3650`) and
//! `SwapVanityEquip` (`:1867-1899`), over the port's split inventory
//! (`App::equip` = `crate::inv::EquipSlots`).
//!
//! Every function here is pure: the click dispatch in `app::clicks` applies the
//! returned target and owns the sound cue, `Main.EquipPageSelected` and the
//! positional favourite clear. The deviations are the ones already recorded for
//! `CanEquipInArmorSlot` (docs/ANALYSIS.md). The two-wings arm of
//! `CanEquipBothAccessories` IS modelled (S1-P-i48) through the port's
//! `Player::wing_slot` table (`player.rs`, the i40 wings census) — the former
//! "`Item.wingSlot` absent" deviation is retired.

use crate::buffs;
use crate::inv::{ARMOR, DYE, MISC};
use crate::item::ItemSlot;
use crate::player::Player;
use crate::registry;
use crate::sortdata::{DUAL_EQUIP_ARMOR, MOUNT_CART, PROJ_HOOK};

/// `ItemID.Sets.AccessoryIncompatibilityType` (`ItemID.cs:338`) — the only
/// non-default rows: `CreateIntSet(-1, 6168, 6168, 6169, 6168, 6194, 6168,
/// 6193, 6168)`.
fn incompatibility_type(kind: u16) -> i32 {
    match kind {
        6168 | 6169 | 6193 | 6194 => 6168,
        _ => -1,
    }
}

/// `ItemSlot.CanEquipBothAccessories` (`ItemSlot.cs:3451-3472`): the same
/// type never doubles, two wings never stack (the `vanity && 3580` Yoraiz0r
/// exception aside), and the `AccessoryIncompatibilityType` classes collide.
/// The wings test reads the port's `Player::wing_slot` table (the i40 census,
/// `player.rs` — the same 47 `Item.wingSlot` assignments).
pub(crate) fn can_equip_both_accessories(a: &ItemSlot, b: &ItemSlot, vanity: bool) -> bool {
    if a.kind == b.kind {
        return false; // `:3453-3456`
    }
    // `:3457-3464` — the two-wings arm, with the 3580 vanity exception.
    if Player::wing_slot(a.kind as i32) > 0 && Player::wing_slot(b.kind as i32) > 0 {
        if vanity && (a.kind == 3580 || b.kind == 3580) {
            return true;
        }
        return false;
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

/// `ItemSlot.HasIncompatibleAccessory` (`ItemSlot.cs:3474-3486`): the FIRST
/// slot whose worn accessory refuses `new_acc`, or `None`.
fn has_incompatible_accessory(
    new_acc: &ItemSlot,
    vanity: bool,
    accessories: &[ItemSlot],
) -> Option<usize> {
    accessories
        .iter()
        .position(|a| !can_equip_both_accessories(a, new_acc, vanity))
}

/// `ItemSlot.CanEquipAccessoryInSlot` (`ItemSlot.cs:3531-3538`): with a
/// collision, only the collision slot itself may take the item; without one,
/// every slot may. `slot` is relative to `accessories` (the C# compares
/// absolute `Player.armor` indices on both sides, so the relative compare is
/// the same test).
fn can_equip_accessory_in_slot(
    item: &ItemSlot,
    vanity: bool,
    accessories: &[ItemSlot],
    slot: usize,
) -> bool {
    match has_incompatible_accessory(item, vanity, accessories) {
        Some(collision) => collision == slot,
        None => true,
    }
}

/// `Player.IsItemSlotUnlockedAndUsable` (`Player.cs:12579-12604`) over the
/// player's live state: slot 8/18 need the Demon Heart's `extraAccessory`
/// (`Player.cs:45194-45198`) AND expert mode; slot 9/19 need master mode —
/// the port has no master difficulty, so they stay locked (ANALYSIS "The
/// UpdateEquip seam"). The `Main.gameMenu` escape arms read false in play.
/// This is the SAME gate `Player::is_item_slot_unlocked_and_usable`
/// (`player.rs`) applies to the grant loop — the equip machinery must not
/// place items the sim would refuse (S1-P-i48).
fn is_item_slot_unlocked_and_usable(slot: usize, p: &Player) -> bool {
    match slot {
        8 | 18 => p.extra_accessory && p.buffs.expert_mode,
        9 | 19 => false, // `Main.masterMode` — unported difficulty
        _ => true,
    }
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

/// `ItemSlot.CanEquipInArmorSlot` (`ItemSlot.cs:1424-1482`) — the left-click
/// placement gate for the armour/vanity rows (`PickItemMovementAction`
/// `:1530-1538`). `slot` is the absolute `Player.armor` index (the C#'s
/// `slot % 10` picks the row, `slot < 10` the real column). The duplicate
/// half and the `DualEquipArmor` bypass ARE modelled (S1-P-i48): a real
/// armour piece refuses when the VANITY set already wears it (and vice
/// versa), unless the kind is one of the three dual-equip heads
/// (`ItemID.cs:336`); an accessory refuses on a collision outside its own
/// slot (`CanEquipAccessoryInSlot`) or when the OPPOSITE column already
/// wears the same kind (`HasSameItemInSlot` — the real column checks the
/// vanity accessories and vice versa, so one copy for show + one for stats
/// is legal, two in the same column is not).
pub(crate) fn can_equip_in_armor_slot(check: &ItemSlot, slot: usize, p: &Player, armor: &[ItemSlot; ARMOR]) -> bool {
    let Some(d) = registry::get(check.kind) else {
        return false;
    };
    let row = slot % 10;
    let real_column = slot < 10; // `:1427` `flag2`
    if row < 3 {
        // `:1430-1450` — the head/body/leg rows demand their own slot id.
        let fits = match row {
            0 => d.head_slot >= 0,
            1 => d.body_slot >= 0,
            _ => d.leg_slot >= 0,
        };
        if !fits {
            return false;
        }
        // `:1451-1463` — the DualEquipArmor bypass, then the opposite set's
        // duplicate check (real -> VanityArmorSlots, vanity -> ArmorSlots).
        if DUAL_EQUIP_ARMOR.get(check.kind) {
            return true;
        }
        let opposite = if real_column { &armor[10..13] } else { &armor[0..3] };
        return !has_same_item_in_slot(check, opposite);
    }
    if !d.accessory {
        return false; // `:1465-1468`
    }
    if real_column {
        // `:1469-1476` — AccSlots collision (vanity: false), then the vanity
        // column's duplicate check.
        if can_equip_accessory_in_slot(check, false, &armor[3..10], row - 3) {
            return !has_same_item_in_slot(check, &armor[13..20]);
        }
        return false;
    }
    // `:1477-1481` — VanityAccSlots collision (vanity: true), then the real
    // column's duplicate check.
    if can_equip_accessory_in_slot(check, true, &armor[13..20], row - 3) {
        return !has_same_item_in_slot(check, &armor[3..10]);
    }
    false
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
    p: &Player,
) -> Option<SwapTarget> {
    let d = registry::get(item.kind)?;
    if d.dye > 0 {
        return dye_swap_target(item, dye, dye_counter, p).map(SwapTarget::Dye);
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
    armor_swap_target(item, armor, p).map(SwapTarget::Armor)
}

/// `ItemSlot.ArmorSwap` (`ItemSlot.cs:3569-3638`) — the `Player.armor` slot the
/// item wants, or `None` for the C#'s "return the item unchanged".
pub(crate) fn armor_swap_target(item: &ItemSlot, armor: &[ItemSlot; ARMOR], p: &Player) -> Option<usize> {
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
    // `:3609` — `HasIncompatibleAccessory(item, item.vanity, ...)`.
    if let Some(collision) = has_incompatible_accessory(item, d.vanity, row) {
        let slot = start + collision;
        return is_item_slot_unlocked_and_usable(slot, p).then_some(slot); // `:3611-3614`
    }
    let mut target = start; // `:3618`
    for i in 0..7 {
        let slot = start + i;
        if is_item_slot_unlocked_and_usable(slot, p) && row[i].count == 0 {
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
    p: &Player,
) -> Option<usize> {
    let d = registry::get(item.kind)?;
    if d.dye <= 0 {
        return None; // `:3544-3547`
    }
    while !is_item_slot_unlocked_and_usable(*counter, p) {
        // `:3549-3552`
        *counter = (*counter + 1) % DYE;
    }
    for (i, cell) in dye.iter().enumerate() {
        // `:3553-3560`
        if is_item_slot_unlocked_and_usable(i, p) && cell.count == 0 {
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
        // slot the one being replaced (relative index `slot - 13`). The
        // vanity item checks AccSlots with `vanity: false`, the real item
        // checks VanityAccSlots with `vanity: true` — the flags ride the
        // DESTINATION column, exactly like the C# call sites.
        let rel = slot.saturating_sub(13);
        if item.count > 0 && !can_equip_accessory_in_slot(item, false, &armor[3..10], rel) {
            return false;
        }
        if real.count > 0 && !can_equip_accessory_in_slot(real, true, &armor[13..20], rel) {
            return false;
        }
    }
    true
}

/// `ItemSlot.TryItemSwap` (`ItemSlot.cs:1901-1995`, S1-P-i48) — the
/// right-click variant cycle for the `ItemID.Sets.HasItemSwap` kinds
/// (`ItemID.cs:1385`). Returns the new kind, or `None` for a kind outside the
/// switch. `Item.ChangeItemType` (`Item.cs`) keeps the stack and re-applies
/// the prefix — the port's `ItemSlot` carries `prefix` verbatim, so only the
/// kind and name change (`SetDefaults(to)` + `Prefix(prefixWeWant)`).
pub(crate) fn try_item_swap(item: &mut ItemSlot) -> Option<u16> {
    let new_kind = match item.kind {
        4131 | 5325 => if item.kind == 5325 { 4131 } else { 5325 }, // `:1906-1910`
        5059 | 5060 => if item.kind == 5059 { 5060 } else { 5059 }, // `:1911-1915`
        5324 => 5329, // `:1916-1919`
        5329 => 5330, // `:1920-1923`
        5330 => 5324, // `:1924-1927`
        4346 | 5391 => if item.kind == 4346 { 5391 } else { 4346 }, // `:1928-1932`
        6190 | 6195 => if item.kind == 6190 { 6195 } else { 6190 }, // `:1933-1937`
        5323 | 5455 => if item.kind == 5323 { 5455 } else { 5323 }, // `:1938-1942`
        4767 | 5453 => if item.kind == 4767 { 5453 } else { 4767 }, // `:1943-1947`
        5309 | 5454 => if item.kind == 5309 { 5454 } else { 5309 }, // `:1948-1952`
        5358 => 5360, // `:1953-1956`
        5360 => 5361, // `:1957-1960`
        5361 => 5359, // `:1961-1964`
        5359 => 5358, // `:1965-1968`
        5437 => 5358, // `:1969-1972`
        6168 => 6169, // `:1973-1976` — the Ammo Cycler cycle
        6169 => 6193, // `:1977-1980`
        6193 => 6194, // `:1981-1984`
        6194 => 6168, // `:1985-1988`
        2611 | 5526 => if item.kind == 2611 { 5526 } else { 2611 }, // `:1989-1993`
        _ => return None,
    };
    item.kind = new_kind;
    item.name = registry::name(new_kind);
    Some(new_kind)
}

/// `ItemSlot.AfterItemSwap`'s cue choice (`ItemSlot.cs:1997-2009`): the
/// fifteen listed new kinds play `SoundEngine.PlaySound(22)` (Unlock), every
/// other swap `PlaySound(7)` (Grab). The `Main.stackSplit = 30` /
/// `mouseRightRelease = false` tail is the caller's (the click dispatch).
pub(crate) fn after_item_swap_cue(new_kind: u16) -> &'static str {
    match new_kind {
        5324 | 5329 | 5330 | 4346 | 5391 | 5358 | 5361 | 5360 | 5359 | 2611 | 5526 | 6168
        | 6169 | 6193 | 6194 => "Unlock",
        _ => "Grab",
    }
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

    /// A classic-mode player with no Demon Heart — the default unlock state
    /// (`Player.cs:12579-12604` reads `extraAccessory` + `Main.expertMode`).
    fn player() -> Player {
        let w = tr_core::world::World::new("t".into(), 64, 800);
        Player::spawn(&w)
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
        let (a, p) = (armor(), player());
        assert_eq!(
            armor_swap_target(&slot(89, 1), &a, &p),
            Some(0),
            "Copper Helmet"
        );
        assert_eq!(
            armor_swap_target(&slot(80, 1), &a, &p),
            Some(1),
            "Copper Chainmail"
        );
        assert_eq!(
            armor_swap_target(&slot(76, 1), &a, &p),
            Some(2),
            "Copper Greaves"
        );
        // 237 Sunglasses is `vanity` + `headSlot 12` -> `armor[10]` (`:3593`).
        assert_eq!(armor_swap_target(&slot(237, 1), &a, &p), Some(10));
        // A non-wearable and an air stack both answer `None` (`:3573-3580`).
        assert_eq!(armor_swap_target(&slot(4, 1), &a, &p), None);
        assert_eq!(armor_swap_target(&EMPTY_SLOT, &a, &p), None);
    }

    /// `ArmorSwap`'s duplicate guards (`:3582-3592`): the real and the vanity
    /// set are both checked for armour, and an accessory checks the C#'s
    /// `item.vanity ? AccSlots : VanityAccSlots` — a *real* accessory is
    /// compared against the vanity column and vice versa, so the same
    /// accessory may be worn once for show and once for stats.
    #[test]
    fn armor_swap_refuses_a_duplicate_piece() {
        let p = player();
        let mut a = armor();
        a[0] = slot(89, 1);
        assert_eq!(
            armor_swap_target(&slot(89, 1), &a, &p),
            None,
            "`:3584` real set"
        );
        // The vanity helmet lives in `armor[10]` and blocks too (`:3584`).
        let mut a = armor();
        a[10] = slot(89, 1);
        assert_eq!(
            armor_swap_target(&slot(89, 1), &a, &p),
            None,
            "`:3584` vanity set"
        );
        // A real accessory checks `VanityAccSlots` = `armor[13..20]` (`:3589`).
        let mut a = armor();
        a[13] = slot(15, 1);
        assert_eq!(armor_swap_target(&slot(15, 1), &a, &p), None);
        // The real column does not block it — the collision scan lands on the
        // occupied slot itself and the slot is usable (`:3611`/`:3622`).
        let mut a = armor();
        a[3] = slot(15, 1);
        assert_eq!(armor_swap_target(&slot(15, 1), &a, &p), Some(3));
        // A vanity accessory (4822) reads `AccSlots` (`:3589`).
        let mut a = armor();
        a[3] = slot(4822, 1);
        assert_eq!(armor_swap_target(&slot(4822, 1), &a, &p), None);
        // ... and targets `armor[13..20]` (`:3593`).
        let mut a = armor();
        assert_eq!(armor_swap_target(&slot(4822, 1), &a, &p), Some(13));
    }

    /// `ArmorSwap`'s accessory scan (`:3606-3629`): the first unlocked empty
    /// row from `armor[3]`, and 8/9 skipped by `IsItemSlotUnlockedAndUsable`.
    #[test]
    fn armor_swap_scans_the_accessory_rows() {
        let p = player();
        // An occupied first row sends it to the next empty one.
        let mut a = armor();
        a[3] = slot(16, 1);
        assert_eq!(armor_swap_target(&slot(15, 1), &a, &p), Some(4));
        // Rows 3..=5 occupied -> row 6.
        for i in 3..6 {
            a[i] = slot(16 + i as u16, 1);
        }
        assert_eq!(armor_swap_target(&slot(15, 1), &a, &p), Some(6));
        // Rows 3..=7 full: the `targetSlot = accessories.Offset` fallback
        // (`:3618`) keeps the first row — matching the C#'s full-row case.
        let mut a = armor();
        for i in 3..8 {
            a[i] = slot(16 + i as u16, 1);
        }
        assert_eq!(armor_swap_target(&slot(15, 1), &a, &p), Some(3));
    }

    /// S1-P-i48 — `IsItemSlotUnlockedAndUsable` (`Player.cs:12579-12604`)
    /// reads the player's live state: the Demon Heart + expert mode unlock
    /// slot 8 (and its vanity twin 18), master mode (unported) keeps 9/19
    /// locked. The equip scan must agree with the sim's grant gate
    /// (`Player::is_item_slot_unlocked_and_usable`).
    #[test]
    fn armor_swap_respects_the_demon_heart_unlock() {
        let mut p = player();
        let mut a = armor();
        // Rows 3..=7 full and no Demon Heart: the fallback is row 3 (`:3618`),
        // never the locked row 8.
        for i in 3..8 {
            a[i] = slot(16 + i as u16, 1);
        }
        a[3] = slot(15, 1); // the collision target: same kind -> slot 3 itself
        assert_eq!(armor_swap_target(&slot(15, 1), &a, &p), Some(3));
        // A NEW kind with rows 3..=7 full falls back to row 3 too — row 8
        // stays out of the scan while locked (`:3622`).
        assert_eq!(armor_swap_target(&slot(212, 1), &a, &p), Some(3));
        // Demon Heart + expert: row 8 joins the empty-slot scan.
        p.extra_accessory = true;
        p.buffs.expert_mode = true;
        assert_eq!(armor_swap_target(&slot(212, 1), &a, &p), Some(8));
        // Expert without the heart keeps it locked (`:12587-12595`).
        p.extra_accessory = false;
        assert_eq!(armor_swap_target(&slot(212, 1), &a, &p), Some(3));
        // Slot 9 stays locked in every state (no master mode).
        p.extra_accessory = true;
        a[8] = slot(212, 1);
        assert_eq!(armor_swap_target(&slot(285, 1), &a, &p), Some(3));
    }

    /// S1-P-i48 — `CanEquipBothAccessories`' two-wings arm
    /// (`ItemSlot.cs:3457-3464`): two different wings refuse each other in
    /// the real column; the vanity column's 3580 (Yoraiz0r Wings) exception
    /// lets ANY wing pair through when one side is 3580.
    #[test]
    fn the_two_wings_arm_refuses_a_second_pair() {
        // 492 Demon Wings / 493 Angel Wings — both `wingSlot > 0`.
        assert!(!can_equip_both_accessories(&slot(492, 1), &slot(493, 1), false));
        assert!(!can_equip_both_accessories(&slot(492, 1), &slot(493, 1), true));
        // The 3580 exception is vanity-only (`:3459-3462`).
        assert!(can_equip_both_accessories(&slot(3580, 1), &slot(493, 1), true));
        assert!(can_equip_both_accessories(&slot(492, 1), &slot(3580, 1), true));
        assert!(!can_equip_both_accessories(&slot(3580, 1), &slot(493, 1), false));
        // A wing and a non-wing still coexist.
        assert!(can_equip_both_accessories(&slot(492, 1), &slot(15, 1), false));
        // The scan refuses the second wings pair: the collision lands on the
        // worn wing's own slot (`ArmorSwap` `:3609-3614`).
        let p = player();
        let mut a = armor();
        a[3] = slot(492, 1);
        assert_eq!(armor_swap_target(&slot(493, 1), &a, &p), Some(3));
        // A fresh wing with the first row occupied by a NON-wing goes to 4.
        let mut a = armor();
        a[3] = slot(15, 1);
        assert_eq!(armor_swap_target(&slot(493, 1), &a, &p), Some(4));
    }

    /// S1-P-i48 — `CanEquipInArmorSlot` (`ItemSlot.cs:1424-1482`), the
    /// left-click gate: the row kinds, the opposite-set duplicate refusal,
    /// the `DualEquipArmor` bypass and the accessory collision/duplicate
    /// halves.
    #[test]
    fn can_equip_in_armor_slot_gates_the_left_click() {
        let p = player();
        let a = armor();
        // The row kinds (`:1430-1450`).
        assert!(can_equip_in_armor_slot(&slot(89, 1), 0, &p, &a));
        assert!(!can_equip_in_armor_slot(&slot(80, 1), 0, &p, &a));
        assert!(can_equip_in_armor_slot(&slot(15, 1), 3, &p, &a));
        assert!(!can_equip_in_armor_slot(&slot(89, 1), 3, &p, &a));
        // A real helmet refuses when the VANITY set wears it (`:1455`).
        let mut a = armor();
        a[10] = slot(89, 1);
        assert!(!can_equip_in_armor_slot(&slot(89, 1), 0, &p, &a));
        // ... and a vanity helmet refuses when the REAL set wears it (`:1461`).
        let mut a = armor();
        a[0] = slot(89, 1);
        assert!(!can_equip_in_armor_slot(&slot(89, 1), 10, &p, &a));
        // The DualEquipArmor bypass (`ItemID.cs:336` — 205 Empty Bucket).
        let mut a = armor();
        a[10] = slot(205, 1);
        assert!(can_equip_in_armor_slot(&slot(205, 1), 0, &p, &a));
        // A real accessory refuses when the vanity column wears the same
        // kind (`:1473`), but its OWN slot's collision is allowed (`:1471`).
        let mut a = armor();
        a[13] = slot(15, 1);
        assert!(!can_equip_in_armor_slot(&slot(15, 1), 3, &p, &a));
        let mut a = armor();
        a[3] = slot(15, 1);
        assert!(can_equip_in_armor_slot(&slot(15, 1), 3, &p, &a));
        assert!(!can_equip_in_armor_slot(&slot(15, 1), 4, &p, &a));
        // The vanity accessory mirrors it (`:1477-1480`).
        let mut a = armor();
        a[3] = slot(15, 1);
        assert!(!can_equip_in_armor_slot(&slot(15, 1), 13, &p, &a));
        let mut a = armor();
        a[13] = slot(15, 1);
        assert!(can_equip_in_armor_slot(&slot(15, 1), 13, &p, &a));
        assert!(!can_equip_in_armor_slot(&slot(15, 1), 14, &p, &a));
        // The wings collision rides the same gate (`:3457-3464` through
        // `CanEquipAccessoryInSlot`).
        let mut a = armor();
        a[3] = slot(492, 1);
        assert!(!can_equip_in_armor_slot(&slot(493, 1), 4, &p, &a));
        assert!(can_equip_in_armor_slot(&slot(493, 1), 3, &p, &a));
    }

    /// `DyeSwap` (`ItemSlot.cs:3553-3564`): the counter's slot is reused when
    /// it is empty, else the first empty row wins and the counter advances.
    #[test]
    fn dye_swap_round_robins() {
        let p = player();
        let mut dye = [EMPTY_SLOT; DYE];
        let mut counter = 0usize;
        assert_eq!(dye_swap_target(&slot(1007, 1), &dye, &mut counter, &p), Some(0));
        assert_eq!(counter, 1, "`:3564` advances past the used slot");
        // The scan restarts from the counter, so a filled slot 0 sends the next
        // dye to slot 1.
        dye[0] = slot(1007, 1);
        assert_eq!(dye_swap_target(&slot(2871, 1), &dye, &mut counter, &p), Some(1));
        // A full row keeps the counter's slot (`:3553-3560` finds nothing).
        let full = [slot(1007, 1); DYE];
        let mut counter = 4usize;
        assert_eq!(
            dye_swap_target(&slot(1007, 1), &full, &mut counter, &p),
            Some(4)
        );
        assert_eq!(counter, 5);
        // A non-dye refuses (`:3544-3547`).
        let mut counter = 0usize;
        assert_eq!(dye_swap_target(&slot(4, 1), &dye, &mut counter, &p), None);
    }

    /// `SwapEquip`'s ladder (`ItemSlot.cs:3291-3347`): the misc rows.
    #[test]
    fn swap_equip_ladder_picks_the_misc_row() {
        let (a, dye, p) = (armor(), [EMPTY_SLOT; DYE], player());
        let mut counter = 0usize;
        // 1007 is a dye -> the dye column.
        assert_eq!(
            swap_equip_target(&slot(1007, 1), &a, &dye, &mut counter, &p),
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

    /// S1-P-i48 — `SwapVanityEquip`'s ctx-11 accessory legs
    /// (`ItemSlot.cs:1874`): a wings collision in ANOTHER real slot refuses
    /// the vanity swap; the collision slot itself (the row's own real
    /// counterpart) passes — `CanEquipAccessoryInSlot` compares the absolute
    /// slot against the collision (`:3535`).
    #[test]
    fn vanity_swap_checks_the_wings_collision() {
        // Real wings in armor[4]; the vanity slot 13 (rel 0) is NOT the
        // collision slot (rel 1) -> the first leg refuses.
        let mut a = armor();
        a[4] = slot(492, 1);
        assert!(!vanity_swap_allowed(&slot(493, 1), &EMPTY_SLOT, 11, 13, &a));
        // Vanity slot 14 (rel 1) IS the collision slot -> allowed.
        assert!(vanity_swap_allowed(&slot(493, 1), &EMPTY_SLOT, 11, 14, &a));
        // The second leg mirrors it: a real item moving up collides with a
        // vanity wings pair outside its own slot.
        let mut a = armor();
        a[14] = slot(492, 1);
        assert!(!vanity_swap_allowed(&EMPTY_SLOT, &slot(493, 1), 11, 13, &a));
        assert!(vanity_swap_allowed(&EMPTY_SLOT, &slot(493, 1), 11, 14, &a));
    }

    /// `CanEquipBothAccessories` (`:3451-3472`): the same item never doubles,
    /// and the 6168 incompatibility class collides.
    #[test]
    fn accessory_compatibility_pairs() {
        assert!(!can_equip_both_accessories(&slot(15, 1), &slot(15, 1), false));
        assert!(can_equip_both_accessories(&slot(15, 1), &slot(16, 1), false));
        assert!(!can_equip_both_accessories(&slot(6168, 1), &slot(6169, 1), false));
        assert!(can_equip_both_accessories(&slot(6168, 1), &slot(16, 1), false));
    }

    /// `CanEquipAccessoryInSlot` allows the collision slot itself (`:3531-3538`).
    #[test]
    fn accessory_collision_allows_the_replaced_slot() {
        let mut acc = [EMPTY_SLOT; 7];
        acc[2] = slot(15, 1);
        assert!(can_equip_accessory_in_slot(&slot(15, 1), false, &acc, 2));
        assert!(!can_equip_accessory_in_slot(&slot(15, 1), false, &acc, 3));
        assert!(can_equip_accessory_in_slot(&slot(16, 1), false, &acc, 3));
    }

    /// S1-P-i48 — `TryItemSwap` (`ItemSlot.cs:1901-1995`): the Ammo Cycler
    /// cycle 6168 -> 6169 -> 6193 -> 6194 -> 6168, the three-step 5324 cycle,
    /// the two-way flips, and the stack/prefix carry (`ChangeItemType`).
    #[test]
    fn try_item_swap_cycles_the_variants() {
        let mut s = slot(6168, 1);
        s.prefix = 65;
        assert_eq!(try_item_swap(&mut s), Some(6169));
        assert_eq!(s.prefix, 65, "`ChangeItemType` keeps the prefix");
        assert_eq!(s.count, 1, "... and the stack");
        assert_eq!(try_item_swap(&mut s), Some(6193));
        assert_eq!(try_item_swap(&mut s), Some(6194));
        assert_eq!(try_item_swap(&mut s), Some(6168));
        // The three-step cycle (`:1916-1927`).
        let mut s = slot(5324, 1);
        assert_eq!(try_item_swap(&mut s), Some(5329));
        assert_eq!(try_item_swap(&mut s), Some(5330));
        assert_eq!(try_item_swap(&mut s), Some(5324));
        // A two-way flip (`:1906-1910`).
        let mut s = slot(4131, 1);
        assert_eq!(try_item_swap(&mut s), Some(5325));
        assert_eq!(try_item_swap(&mut s), Some(4131));
        // 5437 joins the 5358 cycle at its head (`:1969-1972`).
        let mut s = slot(5437, 1);
        assert_eq!(try_item_swap(&mut s), Some(5358));
        // A kind outside the switch answers None and stays put.
        let mut s = slot(15, 1);
        assert_eq!(try_item_swap(&mut s), None);
        assert_eq!(s.kind, 15);
    }

    /// S1-P-i48 — `AfterItemSwap`'s cue split (`ItemSlot.cs:1999-2006`): the
    /// fifteen listed kinds play 22 (Unlock), the rest 7 (Grab).
    #[test]
    fn after_item_swap_splits_the_cues() {
        assert_eq!(after_item_swap_cue(6169), "Unlock");
        assert_eq!(after_item_swap_cue(5329), "Unlock");
        assert_eq!(after_item_swap_cue(5526), "Unlock");
        assert_eq!(after_item_swap_cue(5325), "Grab", "not in the :1999 list");
        assert_eq!(after_item_swap_cue(4131), "Grab");
        assert_eq!(after_item_swap_cue(6195), "Grab");
    }
}
