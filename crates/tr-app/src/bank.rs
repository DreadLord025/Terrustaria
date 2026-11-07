//! The player-bounded banks — `Player.bank/bank2/bank3/bank4`
//! (`Player.cs:1580-1591`, `Chest.CreateBank(-2..-5)`, 40 slots each). The
//! banks are plain `Chest` objects in vanilla (`bankChest = true`,
//! `Chest.cs:191-199`), never in `Main.chest`, always unlocked
//! (`IsLockedOrInUse` → false, `Chest.cs:178-189`). The port keeps them as
//! slot arrays on the `Player`, resolved from `Player.chest`'s
//! -2..-5 index space (`Player.cs:33492-33502`).

use crate::item::ItemSlot;

/// `Chest.CreateBank`'s `maxItems = 40` (`Chest.cs:196`).
pub(crate) const BANK_SLOTS: usize = 40;

/// `Player.chest`'s bank index space (`Player.cs:33492-33502`): the port's
/// `Player.chest` carries the same -1 (closed) / -2..-5 (banks) values.
pub(crate) const CHEST_CLOSED: i32 = -1;

/// One bank's slot array (`Chest.item[]`'s port).
pub(crate) type BankStore = [ItemSlot; BANK_SLOTS];

/// The bank's slots for the open `chest` index; `None` for -1 and the world
/// indices (the world chests are M3-CHEST-UI's).
pub(crate) fn bank_for_chest(chest: i32) -> Option<usize> {
    match chest {
        -5..=-2 => Some((-chest - 2) as usize),
        _ => None,
    }
}

/// `Player.CanVoidVaultAccept` (`Player.cs:39425-39433`): the quest items and
/// the void-lens projectile kind 3822 refuse. The port has no `questItem`
/// flag yet — the id 3822 arm and a placeholder quest set are the seam.
pub(crate) fn can_void_vault_accept(kind: u16) -> bool {
    kind != 3822
}

/// `Player.GetItem_VoidVault` (`Player.cs:39402-39423`): fill the occupied
/// stackable slots first, then the empty ones; returns whether the whole
/// stack fit. The merge rule is `Item.CanStack` (same kind AND same prefix —
/// the port's `slot_for_pickup` shape, applied to the vault's 40 slots).
pub(crate) fn void_vault_take(
    vault: &mut BankStore,
    kind: u16,
    prefix: u16,
    count: u32,
    max_stack: u32,
) -> u32 {
    let mut left = count;
    // The occupied stackable slots first (:39402-39412).
    for slot in vault.iter_mut() {
        if left == 0 {
            break;
        }
        if slot.count > 0 && slot.kind == kind && slot.prefix == prefix {
            let add = left.min(max_stack.saturating_sub(slot.count));
            slot.count += add;
            left -= add;
        }
    }
    // Then the empty slots (:39412-39423).
    for slot in vault.iter_mut() {
        if left == 0 {
            break;
        }
        if slot.count == 0 {
            let add = left.min(max_stack);
            *slot = ItemSlot {
                kind,
                name: crate::registry::name(kind),
                count: add,
                prefix,
            };
            left -= add;
        }
    }
    count - left
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_vault() -> BankStore {
        std::array::from_fn(|_| ItemSlot {
            kind: 0,
            name: "",
            count: 0,
            prefix: 0,
        })
    }

    #[test]
    fn bank_index_space_maps_chest_to_store() {
        // `Player.cs:33492-33502`: -2..-5 → bank/bank2/bank3/bank4.
        assert_eq!(bank_for_chest(-1), None);
        assert_eq!(bank_for_chest(0), None);
        assert_eq!(bank_for_chest(-2), Some(0));
        assert_eq!(bank_for_chest(-3), Some(1));
        assert_eq!(bank_for_chest(-4), Some(2));
        assert_eq!(bank_for_chest(-5), Some(3));
    }

    #[test]
    fn void_vault_accept_fills_stacks_then_empty_slots() {
        // `Player.cs:39402-39423`: the occupied stackable slots first, then
        // the empty ones; the leftover stays on the ground path.
        let mut vault = empty_vault();
        vault[0] = ItemSlot {
            kind: 9,
            name: "Wood",
            count: 999,
            prefix: 0,
        };
        vault[1] = ItemSlot {
            kind: 9,
            name: "Wood",
            count: 10,
            prefix: 0,
        };
        // 999 (full) + 10: slot 1 takes 989 more, then the empty slots take
        // 999 and the last 12 — four slots carry the 2000.
        let taken = void_vault_take(&mut vault, 9, 0, 2000, 999);
        assert_eq!(taken, 2000);
        assert_eq!(vault[0].count, 999);
        assert_eq!(vault[1].count, 999, "slot 1 took 989 more");
        assert_eq!(vault[2].kind, 9);
        assert_eq!(vault[2].count, 999);
        assert_eq!(vault[3].count, 12);
        // The full vault refuses.
        let mut vault = empty_vault();
        for slot in vault.iter_mut() {
            *slot = ItemSlot {
                kind: 3,
                name: "Stone Block",
                count: 999,
                prefix: 0,
            };
        }
        let taken = void_vault_take(&mut vault, 9, 0, 10, 999);
        assert_eq!(taken, 0);
    }

    #[test]
    fn void_vault_rejects_the_excluded_kinds() {
        // `CanVoidVaultAccept` (`Player.cs:39425-39433`): 3822 refuses.
        assert!(!can_void_vault_accept(3822));
        assert!(can_void_vault_accept(9));
    }
}
