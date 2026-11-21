//! `EquipmentLoadout` (`decompiled/Terraria/EquipmentLoadout.cs`) — the three
//! swappable armor/dye sets behind the loadout buttons (`Main.DrawLoadoutButtons`,
//! `Main.cs:41935-41991`).
//!
//! The port keeps the live equipment arrays on `App.equip` (`inv::EquipSlots`);
//! a loadout stores the *other two* sets plus their share flags. `Swap`
//! exchanges the live arrays with the stored ones exactly as the C# does
//! (`EquipmentLoadout.cs:63-80`), so `TrySwitchingLoadout`'s two-swap shape
//! (`Player.cs:5665-5666`) moves the current set into its slot and the target
//! set out. The per-item `favorited` flag doubles as the loadout-share flag in
//! the equip contexts (`ItemSlot.canLoadoutShareAt`, `ItemSlot.cs:395-399`);
//! the port's grid favorites ride `App.favorite_slots`, so the equip-side
//! flags get their own 30-bit word per loadout here.

use crate::inv::{ARMOR, DYE};
use crate::item::{EMPTY_SLOT, ItemSlot};

/// `EquipmentLoadout.Hide` (`EquipmentLoadout.cs:12`, `new bool[10]` `:18`):
/// the per-accessory visual-hide flags stored with the set.
pub const HIDE: usize = 10;

/// One stored equipment set — `EquipmentLoadout`'s three arrays.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EquipmentLoadout {
    /// `EquipmentLoadout.Armor` — `CreateItemArray(20)` (`:16`).
    pub armor: [ItemSlot; ARMOR],
    /// `EquipmentLoadout.Dye` — `CreateItemArray(10)` (`:17`).
    pub dye: [ItemSlot; DYE],
    /// `EquipmentLoadout.Hide` — `new bool[10]` (`:18`).
    pub hide: [bool; HIDE],
    /// The per-slot `Item.favorited` bits over `armor[0..20]` then
    /// `dye[0..10]` (bit `i` = armor slot `i`, bit `20 + j` = dye slot `j`).
    /// In the equip contexts the flag is the loadout-share toggle
    /// (`ItemSlot.ToggleLoadoutShare`, `ItemSlot.cs:879-889`), read by
    /// `GetEffectiveArmor`/`GetEffectiveDye` (`Player.cs:5690-5764`).
    pub favorited: u32,
}

impl Default for EquipmentLoadout {
    /// The ctor (`EquipmentLoadout.cs:14-19`): 20 + 10 fresh `Item()`s (all
    /// air, `favorited` false) and `Hide[10]` false.
    fn default() -> Self {
        Self {
            armor: [EMPTY_SLOT; ARMOR],
            dye: [EMPTY_SLOT; DYE],
            hide: [false; HIDE],
            favorited: 0,
        }
    }
}

impl EquipmentLoadout {
    /// The share flag of one stored slot: `slot < 20` indexes `Armor`, else
    /// `Dye[slot - 20]` (the `PlayerItemSlotID.Loadout*_Armor_0/Dye_0` split).
    pub fn is_shared(&self, slot: usize) -> bool {
        slot < 30 && self.favorited & (1 << slot) != 0
    }

    /// `Item.favorited = value` on one stored slot (the toggle's write half).
    pub fn set_shared(&mut self, slot: usize, value: bool) {
        if slot >= 30 {
            return;
        }
        if value {
            self.favorited |= 1 << slot;
        } else {
            self.favorited &= !(1 << slot);
        }
    }

    /// The stored item at `slot` (`< 20` armor, else dye) — the
    /// `GetEffectiveArmor`/`GetEffectiveDye` scan reads this.
    pub fn item_at(&self, slot: usize) -> ItemSlot {
        if slot < ARMOR {
            self.armor[slot]
        } else if slot < ARMOR + DYE {
            self.dye[slot - ARMOR]
        } else {
            EMPTY_SLOT
        }
    }

    /// `EquipmentLoadout.Swap` (`EquipmentLoadout.cs:63-80`): `Utils.Swap`
    /// every armor slot, every dye slot and every `hideVisibleAccessory` flag
    /// with the player's live arrays. The share flags swap with their items
    /// (the C# `favorited` rides the `Item` instance through the swap).
    pub fn swap(
        &mut self,
        armor: &mut [ItemSlot; ARMOR],
        dye: &mut [ItemSlot; DYE],
        hide: &mut [bool; HIDE],
        live_favorited: &mut u32,
    ) {
        for i in 0..ARMOR {
            std::mem::swap(&mut armor[i], &mut self.armor[i]);
        }
        for j in 0..DYE {
            std::mem::swap(&mut dye[j], &mut self.dye[j]);
        }
        for k in 0..HIDE {
            std::mem::swap(&mut hide[k], &mut self.hide[k]);
        }
        std::mem::swap(live_favorited, &mut self.favorited);
    }
}

/// One item's net-state snapshot — `Player.clientCloneItem`
/// (`Player.cs:54514-54520`): `type`, `stack`, `prefix`, `favorited` (the four
/// fields `Item.IsNetStateDifferent` compares). The port's `ItemSlot` carries
/// kind/count/prefix; the share flag rides the loadout's 30-bit word, so the
/// clone takes it as a parameter (the c28 `item_net_state` convention).
pub type ClonedItem = (u16, u32, u16, bool);

/// `Player.clientCloneItemArray` (`Player.cs:54506-54512`): clone every slot's
/// net state, reading the per-slot `favorited` bit out of `share_word`.
fn clone_item_array(items: &[ItemSlot], share_word: u32, bit_base: usize) -> Vec<ClonedItem> {
    items
        .iter()
        .enumerate()
        .map(|(i, s)| {
            (
                s.kind,
                s.count,
                s.prefix,
                share_word & (1 << (bit_base + i)) != 0,
            )
        })
        .collect()
}

/// The `clientCloneLoadouts` snapshot (`Player.cs:54522-54531`) — the item-array
/// half of `Player.clientClone` (`:54461-54504`, the c28 `ClientClone` carries
/// the scalar half). It clones the LIVE armor + dye arrays and each of the
/// three stored loadouts' Armor + Dye (`clientCloneItemArray`, `:54524-54530`).
/// `Hide`/`CurrentLoadoutIndex` are NOT cloned by the C# (the stored sets'
/// `favorited` rides each item). The `TrySwitchingLoadout` tail calls it on
/// `Main.clientPlayer` (`:5670`) so the next `TrySyncingMyPlayer` array diff
/// (`Main.cs:17557-17575`) sees the swapped sets as already-synced.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LoadoutClone {
    /// `clientCloneItemArray(this.armor, clonePlayer.armor)` (`:54524`).
    pub armor: Vec<ClonedItem>,
    /// `clientCloneItemArray(this.dye, clonePlayer.dye)` (`:54525`).
    pub dye: Vec<ClonedItem>,
    /// `Loadouts[i].Armor` then `Loadouts[i].Dye` per set (`:54526-54530`),
    /// indexed `[loadout]` -> `(armor, dye)`.
    pub loadouts: [(Vec<ClonedItem>, Vec<ClonedItem>); 3],
}

impl LoadoutClone {
    /// `Player.clientCloneLoadouts` (`Player.cs:54522-54531`): clone the live
    /// armor/dye (the live share word supplies their `favorited` bits) and the
    /// three stored sets (each set's own share word). `live_favorited` is
    /// `App.equip_share_flags`; bit `i` = armor[i], bit `20 + j` = dye[j].
    pub fn clone_from(
        armor: &[ItemSlot; ARMOR],
        dye: &[ItemSlot; DYE],
        live_favorited: u32,
        loadouts: &[EquipmentLoadout; 3],
    ) -> Self {
        Self {
            armor: clone_item_array(armor, live_favorited, 0),
            dye: clone_item_array(dye, live_favorited, ARMOR),
            loadouts: std::array::from_fn(|i| {
                (
                    clone_item_array(&loadouts[i].armor, loadouts[i].favorited, 0),
                    clone_item_array(&loadouts[i].dye, loadouts[i].favorited, ARMOR),
                )
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(kind: u16) -> ItemSlot {
        ItemSlot {
            kind,
            name: crate::registry::name(kind),
            count: 1,
            prefix: 0,
        }
    }

    /// W4-g55 — the ctor is 20 + 10 air items, `Hide[10]` false, no shares
    /// (`EquipmentLoadout.cs:14-19`; the `.plr` fresh-default block is 310
    /// zero bytes per loadout, `player_file.rs`'s golden layout).
    #[test]
    fn w4_g55_the_fresh_loadout_is_all_air() {
        let l = EquipmentLoadout::default();
        assert!(l.armor.iter().all(|s| s.count == 0));
        assert!(l.dye.iter().all(|s| s.count == 0));
        assert_eq!(l.hide, [false; HIDE]);
        assert_eq!(l.favorited, 0);
    }

    /// W4-g55 — `Swap` exchanges all three arrays plus the share word, and a
    /// second swap restores the original state (`Utils.Swap` is symmetric).
    #[test]
    fn w4_g55_swap_exchanges_every_array_and_is_symmetric() {
        let mut live_armor = [EMPTY_SLOT; ARMOR];
        let mut live_dye = [EMPTY_SLOT; DYE];
        let mut live_hide = [false; HIDE];
        let mut live_fav = 0u32;
        live_armor[0] = slot(90); // Iron Helmet (`ItemID.cs:1863`)
        live_dye[0] = slot(1007); // Red Dye (`ItemID.cs:3697`)
        live_hide[3] = true;
        live_fav |= 1 << 0;

        let mut stored = EquipmentLoadout::default();
        stored.armor[1] = slot(954); // Ancient Iron Helmet (`ItemID.cs:3591`)
        stored.set_shared(21, true); // dye slot 1 shared

        let before = (live_armor, live_dye, live_hide, live_fav);
        stored.swap(&mut live_armor, &mut live_dye, &mut live_hide, &mut live_fav);
        assert_eq!(live_armor[0], EMPTY_SLOT, "the live head moved into storage");
        assert_eq!(stored.armor[0].kind, 90);
        assert_eq!(live_armor[1].kind, 954, "the stored head came out");
        assert_eq!(live_dye[0], EMPTY_SLOT);
        assert_eq!(stored.dye[0].kind, 1007);
        assert!(!live_hide[3], "the hide flags swapped");
        assert!(stored.hide[3]);
        assert_eq!(live_fav, 1 << 21, "the share word swapped");
        assert_eq!(stored.favorited, 1 << 0);

        stored.swap(&mut live_armor, &mut live_dye, &mut live_hide, &mut live_fav);
        assert_eq!((live_armor, live_dye, live_hide, live_fav), before);
    }

    /// W4-g55 — `clientCloneLoadouts` (`Player.cs:54522-54531`) clones the
    /// live armor/dye + the three stored sets' Armor/Dye, per-item
    /// type/stack/prefix/favorited (`clientCloneItem`, `:54514-54520`). The
    /// live share word supplies the live arrays' favorited bits; each stored
    /// set's own word supplies its own.
    #[test]
    fn w4_g55_the_loadout_clone_snapshots_every_array() {
        let mut armor = [EMPTY_SLOT; ARMOR];
        let mut dye = [EMPTY_SLOT; DYE];
        armor[0] = slot(90); // Iron Helmet, live + shared
        dye[1] = slot(1007); // Red Dye, live + unshared
        let live_fav = 1u32 << 0; // armor[0] shared, dye[1] not

        let mut loadouts = [EquipmentLoadout::default(); 3];
        loadouts[1].armor[2] = slot(954); // Ancient Iron Helmet, stored
        loadouts[1].set_shared(2, true); // armor[2] shared
        loadouts[2].dye[0] = slot(1007); // stored dye, unshared

        let c = LoadoutClone::clone_from(&armor, &dye, live_fav, &loadouts);
        // The live armor[0]: (90, 1, 0, true).
        assert_eq!(c.armor[0], (90, 1, 0, true));
        assert_eq!(c.armor[1], (0, 0, 0, false), "an air slot clones air");
        // The live dye[1]: (1007, 1, 0, false) — bit 20+1 not set.
        assert_eq!(c.dye[1], (1007, 1, 0, false));
        // Loadout 1's armor[2]: (954, 1, 0, true).
        assert_eq!(c.loadouts[1].0[2], (954, 1, 0, true));
        // Loadout 2's dye[0]: (1007, 1, 0, false).
        assert_eq!(c.loadouts[2].1[0], (1007, 1, 0, false));
        // Loadout 0 is all air.
        assert!(c.loadouts[0].0.iter().all(|&(k, _, _, _)| k == 0));
        // The array lengths match the C# (20 armor + 10 dye per set).
        assert_eq!(c.armor.len(), ARMOR);
        assert_eq!(c.dye.len(), DYE);
        assert_eq!(c.loadouts[1].0.len(), ARMOR);
        assert_eq!(c.loadouts[1].1.len(), DYE);
    }
}
