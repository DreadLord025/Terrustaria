//! The full `Player` inventory/equipment model behind `Main.DrawInventory`
//! (`Main.cs:41001`).
//!
//! Vanilla keeps a single 59-slot `Player.inventory` (0..49 the 10x5 grid,
//! 50..53 coins, 54..57 ammo) plus the `armor`/`dye`/`miscEquips`/`miscDyes`
//! arrays. The port keeps the 50-slot main grid in `App::inventory` (so the
//! saved-player format in `persist.rs` is untouched) and the remaining equipment
//! slots here.

use crate::item::{EMPTY_SLOT, ItemSlot};

/// `Player.inventory[50..53]`: the four coin slots (`Main.cs:41753-41764`).
pub const COINS: usize = 4;
/// `Player.inventory[54..57]`: the four ammo slots (`Main.cs:41771-41782`).
pub const AMMO: usize = 4;
/// `Player.armor[0..19]` (`Player.cs:1488-1494`): 3 armour + 7 accessories
/// (+2 hidden), then the matching vanity set.
pub const ARMOR: usize = 20;
/// `Player.dye[0..9]`.
pub const DYE: usize = 10;
/// `Player.miscEquips` / `Player.miscDyes` (`Player.cs:1488`).
pub const MISC: usize = 5;

/// `Player.inventory`'s full length: the 10x5 grid plus the four coin and four
/// ammo slots the C# keeps in the same array (`Player.cs:55340-55362`'s 58
/// per-slot records). The port splits the arrays (`App.inventory` vs
/// `EquipSlots::coins`/`ammo`), so the favorite key space borrows the vanilla
/// numbering — grid `0..50`, coins `50..54`, ammo `54..58`.
pub const INV_SLOTS: usize = crate::item::INV + COINS + AMMO;

/// The vanilla inventory index of a coin cell (`Player.inventory[50..53]`).
pub fn coin_slot_index(k: usize) -> usize {
    crate::item::INV + k
}

/// The vanilla inventory index of an ammo cell (`Player.inventory[54..57]`).
pub fn ammo_slot_index(k: usize) -> usize {
    crate::item::INV + COINS + k
}

/// The inventory slots that are not part of the main 10x5 grid.
#[derive(Clone, Copy)]
pub struct EquipSlots {
    pub coins: [ItemSlot; COINS],
    pub ammo: [ItemSlot; AMMO],
    pub armor: [ItemSlot; ARMOR],
    pub dye: [ItemSlot; DYE],
    pub misc_equips: [ItemSlot; MISC],
    pub misc_dyes: [ItemSlot; MISC],
    pub trash: ItemSlot,
}

impl EquipSlots {
    /// A freshly-created character has every equipment slot empty.
    pub fn empty() -> Self {
        Self {
            coins: [EMPTY_SLOT; COINS],
            ammo: [EMPTY_SLOT; AMMO],
            armor: [EMPTY_SLOT; ARMOR],
            dye: [EMPTY_SLOT; DYE],
            misc_equips: [EMPTY_SLOT; MISC],
            misc_dyes: [EMPTY_SLOT; MISC],
            trash: EMPTY_SLOT,
        }
    }
}

impl Default for EquipSlots {
    fn default() -> Self {
        Self::empty()
    }
}


#[cfg(test)]
mod s1_l_11f_tests {
    use super::*;

    #[test]
    fn the_slot_capacity_matches_the_c_sharp_split() {
        // `Player.inventory = Item[59]` (`Player.cs:1576`): 0..49 the
        // grid/hotbar, 50..53 the coins, 54..57 the ammo and 58 the trash
        // (`inventory[58]`, `Player.cs:5130/:5145`). The port splits the same
        // capacity across `inventory[INV]` + `equip.coins[COINS]` +
        // `equip.ammo[AMMO]` + `equip.trash` — the totals agree.
        assert_eq!(crate::item::INV, 50, "the grid/hotbar slots");
        assert_eq!(COINS + AMMO, 8, "the coin/ammo rows");
        assert_eq!(INV_SLOTS, 58, "the carry slots the UI hits");
        // The 59th slot: the C#'s `inventory[58]` trash stash is the port's
        // `equip.trash` (the BUG-20 shift-click test covers its behavior).
    }
}
