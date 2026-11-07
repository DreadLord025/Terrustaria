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

// --- The transfer buttons (`UI/ChestUI.cs`, M3-STORE-2) ---------------------

/// The coin types (`ItemID.CopperCoin..PlatinumCoin`).
const COIN_TYPES: [u16; 4] = [71, 72, 73, 74];

/// `Item.CanStack` (`Item.cs:49865-49871`): two slots stack only when the
/// type AND the prefix match — the merge gate shared by every transfer
/// (the world-chest paths call it from `chest.rs`/`clicks.rs` too).
pub(crate) fn can_stack(a: &ItemSlot, b: &ItemSlot) -> bool {
    a.kind == b.kind && a.prefix == b.prefix
}

/// `ChestUI.LootAll` (:393-473): the container drains into the player. The
/// coins route into the coin rows (`Player.GetItem`'s isACoin arm,
/// `Player.cs:39330` — the occupied coin slots first, then the empty);
/// everything else rides the pickup stack (hotbar first, then the rows).
/// The overflow STAYS in the container (`GetItem` returns the un-moved
/// remainder) — unless `vault` is `Some`, the world-chest variant's
/// `CanGoIntoVoidVault` delta (`LootAllFromChest`, `GetItemSettings.cs:55-56`;
/// banks ride the default `LootAllFromBank`, :55, and pass `None`): the
/// overflow then fills the void vault as `GetItem`'s per-item last resort
/// (`Player.cs:39395-39398`) via the landed `void_vault_take`.
pub(crate) fn loot_all(
    container: &mut [ItemSlot],
    inventory: &mut [ItemSlot; crate::item::INV],
    coins: &mut [ItemSlot; 4],
    mut vault: Option<&mut BankStore>,
) {
    for slot in container.iter_mut() {
        if slot.kind == 0 || slot.count == 0 {
            continue;
        }
        let mut item = std::mem::replace(
            slot,
            ItemSlot {
                kind: 0,
                name: "",
                count: 0,
                prefix: 0,
            },
        );
        if COIN_TYPES.contains(&item.kind) {
            // `Player.GetItem`'s coin routing: the occupied coin rows first,
            // then the empty ones.
            let coin_cap = crate::registry::max_stack(item.kind);
            for cs in coins.iter_mut() {
                if item.count == 0 {
                    break;
                }
                if cs.count > 0 && can_stack(cs, &item) {
                    let moved = item.count.min(coin_cap.saturating_sub(cs.count));
                    cs.count += moved;
                    item.count -= moved;
                }
            }
            for cs in coins.iter_mut() {
                if item.count == 0 {
                    break;
                }
                if cs.count == 0 {
                    let put = item.count.min(coin_cap);
                    *cs = ItemSlot {
                        kind: item.kind,
                        name: item.name,
                        count: put,
                        prefix: item.prefix,
                    };
                    item.count -= put;
                }
            }
        } else if let Some(i) = crate::item::slot_for_pickup(inventory, item.kind, item.prefix) {
            let cap = crate::registry::max_stack(item.kind).saturating_sub(inventory[i].count);
            let moved = item.count.min(cap);
            inventory[i].count += moved;
            item.count -= moved;
        }
        if item.count > 0 {
            // `Player.cs:39395-39398` — the vault is `GetItem`'s last resort
            // after every inventory pass failed; `CanVoidVaultAccept` gates
            // the kinds (`Player.cs:39425-39433`).
            if let Some(vault) = vault.as_deref_mut() {
                if can_void_vault_accept(item.kind) {
                    let taken = void_vault_take(
                        vault,
                        item.kind,
                        item.prefix,
                        item.count,
                        crate::registry::max_stack(item.kind),
                    );
                    item.count -= taken;
                }
            }
            if item.count > 0 {
                // The remainder rides back home (:400-414's returned item).
                *slot = item;
            }
        }
    }
}

/// `ChestUI.MoveCoins` (:997-1185) compacted: the container's coin stacks
/// and the player's (un-favorited) coin stacks consolidate into the
/// container — the denomination carry `>= 100 -> next tier` included. The
/// pull only runs when the CONTAINER held a coin stack (`flag2`, :1044 —
/// `if (!flag2) return 0L;` :1048-1051 makes a coin-less container a total
/// no-op). The port's coins live as stacks inside the 4 coin rows, so the
/// carry walks the tallies directly; the leftovers return to the player's
/// rows (the overflow guard, :1090-1208).
pub(crate) fn move_coins(
    container: &mut [ItemSlot],
    inventory: &mut [ItemSlot; crate::item::INV],
    coins: &mut [ItemSlot; 4],
    is_favorite: &dyn Fn(usize) -> bool,
) {
    // The vanilla consolidates INTO the container (`MoveCoins(pInv, cInv)`,
    // :997-1185): pass 1 sucks the container's coin stacks into the
    // tallies (:1011-1046) and sets `flag2` only then (:1044), pass 2 pulls
    // the player's un-favorited coin stacks — the whole 0..58 range, the
    // grid rows included (:1052-1080's `j != 58` skip); the carry rolls
    // 100-per-tier (:1082-1089); the container's free slots refill
    // largest-first (:1115-1147); whatever does not fit returns to the
    // player's coin rows (:1090-1208's tail).
    let mut tallies = [0u64; 4];
    let mut container_had_coins = false; // `:1044`'s `flag2`
    for slot in container.iter_mut() {
        if slot.count > 0 {
            if let Some(d) = COIN_TYPES.iter().position(|t| *t == slot.kind) {
                tallies[d] += slot.count as u64;
                *slot = ItemSlot {
                    kind: 0,
                    name: "",
                    count: 0,
                    prefix: 0,
                };
                container_had_coins = true;
            }
        }
    }
    // `ChestUI.cs:1048-1051` — `if (!flag2) return 0L;`: a coin-less
    // container is a total no-op, the player's coins stay put.
    if !container_had_coins {
        return;
    }
    let coin_cap = |d: usize| crate::registry::max_stack(COIN_TYPES[d]);
    for i in 0..50 {
        if inventory[i].count > 0 && !is_favorite(i) {
            if let Some(d) = COIN_TYPES
                .iter()
                .position(|t| *t == inventory[i].kind)
            {
                tallies[d] += inventory[i].count as u64;
                inventory[i] = ItemSlot {
                    kind: 0,
                    name: "",
                    count: 0,
                    prefix: 0,
                };
            }
        }
    }
    // The coin rows are pInv 50-53 (`ChestUI.cs:1052-1054` reads each coin
    // item's own flag) — the grid-index space (0-49) must not alias them.
    for i in 0..4 {
        if coins[i].count > 0 && !is_favorite(50 + i) {
            if let Some(d) = COIN_TYPES.iter().position(|t| *t == coins[i].kind) {
                tallies[d] += coins[i].count as u64;
                coins[i] = ItemSlot {
                    kind: 0,
                    name: "",
                    count: 0,
                    prefix: 0,
                };
            }
        }
    }
    // The carry (:1082-1089): 100 of a tier rolls into the next.
    for k in 0..3 {
        tallies[k + 1] += tallies[k] / 100;
        tallies[k] %= 100;
    }
    // The container's free slots refill largest-first (:1115-1147).
    for d in (0..4).rev() {
        while tallies[d] > 0 {
            let Some(free) = container
                .iter()
                .position(|sl| sl.kind == 0 || sl.count == 0)
            else {
                break;
            };
            let put = tallies[d].min(coin_cap(d) as u64) as u32;
            container[free] = ItemSlot {
                kind: COIN_TYPES[d],
                name: crate::registry::name(COIN_TYPES[d]),
                count: put,
                prefix: 0,
            };
            tallies[d] -= put as u64;
        }
    }
    // The overflow returns to the player's coin rows (:1090-1208's tail).
    for d in 0..4 {
        while tallies[d] > 0 {
            let row = coins
                .iter()
                .position(|sl| sl.count == 0 && !is_favorite(50 + d))
                .unwrap_or(d.min(3));
            if coins[row].count > 0 && coins[row].kind != COIN_TYPES[d] {
                break;
            }
            let cap = coin_cap(d).saturating_sub(coins[row].count) as u64;
            let put = tallies[d].min(cap) as u32;
            if put == 0 {
                break;
            }
            coins[row] = ItemSlot {
                kind: COIN_TYPES[d],
                name: crate::registry::name(COIN_TYPES[d]),
                count: coins[row].count + put,
                prefix: 0,
            };
            tallies[d] -= put as u64;
        }
    }
}

/// `ChestUI.DepositAll` (:540-659) after MoveCoins: the storage rows scan
/// 49..=10 descending (the hotbar stays), the favorited rows stay; the
/// stack-merge phase (maxStack > 1, type+prefix equality, :516-524) then the
/// empty-slot phase (the first free slot per source, :579-655).
pub(crate) fn deposit_all(
    container: &mut [ItemSlot],
    inventory: &mut [ItemSlot; crate::item::INV],
    is_favorite: &dyn Fn(usize) -> bool,
) {
    for num in (10..50).rev() {
        if inventory[num].count == 0 || inventory[num].kind == 0 || is_favorite(num) {
            continue;
        }
        let max = crate::registry::max_stack(inventory[num].kind);
        if max > 1 {
            // The merge phase (:567-578).
            for cs in container.iter_mut() {
                if inventory[num].count == 0 {
                    break;
                }
                if cs.count > 0 && cs.count < max && can_stack(cs, &inventory[num]) {
                    let moved = inventory[num].count.min(max - cs.count);
                    cs.count += moved;
                    inventory[num].count -= moved;
                }
            }
        }
        if inventory[num].count > 0 {
            // The empty-slot phase (:579-655): the first free slot.
            if let Some(free) = container.iter().position(|sl| sl.kind == 0 || sl.count == 0) {
                container[free] = std::mem::replace(
                    &mut inventory[num],
                    ItemSlot {
                        kind: 0,
                        name: "",
                        count: 0,
                        prefix: 0,
                    },
                );
            }
        }
    }
}

/// `ChestUI.QuickStack` (:661-825) after its MoveCoins gate: the container's
/// non-coin stacks are the merge targets (the coin exclusion 71-74 rides the
/// TARGET side, :703-714), the source scan = the storage rows 10..50 (+ the
/// coin rows when a bank is open, :715-725), the same-type filter (:726-732),
/// the CanStack merge (:733-760) and the first-source-per-type empty-slot
/// rule (:772-812). The MoveCoins pre-pass is the DISPATCH layer's (M3-CHEST-INT
/// decision): the C# runs it for the bank kinds only (:670-685 — the world
/// chest gets none), so the bank call site does `move_coins` + `quick_stack`
/// and the world call site calls this alone.
pub(crate) fn quick_stack(
    container: &mut [ItemSlot],
    inventory: &mut [ItemSlot; crate::item::INV],
    coins: &mut [ItemSlot; 4],
    is_favorite: &dyn Fn(usize) -> bool,
) {
    // The targets: the container's non-coin occupied stacks.
    let mut types: Vec<u16> = Vec::new();
    for cs in container.iter() {
        if cs.count > 0 && cs.kind > 0 && !COIN_TYPES.contains(&cs.kind) {
            types.push(cs.kind);
        }
    }
    // The merge phase over the storage rows + the coin rows.
    for j in 10..50 {
        if inventory[j].count > 0 && types.contains(&inventory[j].kind) && !is_favorite(j) {
            for cs in container.iter_mut() {
                if inventory[j].count == 0 {
                    break;
                }
                let cap = crate::registry::max_stack(cs.kind);
                if cs.count > 0 && cs.count < cap && can_stack(cs, &inventory[j]) {
                    let moved = inventory[j].count.min(cap - cs.count);
                    cs.count += moved;
                    inventory[j].count -= moved;
                }
            }
        }
    }
    for j in 0..4 {
        if coins[j].count > 0 && types.contains(&coins[j].kind) {
            for cs in container.iter_mut() {
                if coins[j].count == 0 {
                    break;
                }
                let cap = crate::registry::max_stack(cs.kind);
                if cs.count > 0 && cs.count < cap && can_stack(cs, &coins[j]) {
                    let moved = coins[j].count.min(cap - cs.count);
                    cs.count += moved;
                    coins[j].count -= moved;
                }
            }
        }
    }
    // The empty-slot phase: only the first source of a type takes a free
    // slot (:772-812's flag rule).
    let mut taken: Vec<u16> = Vec::new();
    for j in 10..50 {
        if inventory[j].count > 0
            && types.contains(&inventory[j].kind)
            && !is_favorite(j)
            && !taken.contains(&inventory[j].kind)
        {
            if let Some(free) = container.iter().position(|sl| sl.kind == 0 || sl.count == 0) {
                taken.push(inventory[j].kind);
                std::mem::swap(&mut container[free], &mut inventory[j]);
            }
        }
    }
    for j in 0..4 {
        if coins[j].count > 0 && types.contains(&coins[j].kind) && !taken.contains(&coins[j].kind)
        {
            if let Some(free) = container.iter().position(|sl| sl.kind == 0 || sl.count == 0) {
                taken.push(coins[j].kind);
                std::mem::swap(&mut container[free], &mut coins[j]);
            }
        }
    }
}

/// `ChestUI.Restock` (:871-995): the container -> the player. The demand =
/// the player's stackable types; the supply = the container's UNPREFIXED
/// stacks of demanded types (:919-924's `prefix != 0` + `hashSet.Contains`
/// gate, which governs the whole loop); the capped fill (:925-965) and the
/// ammo-into-empty bonus (:966-989 — a demanded ammo stack that did NOT
/// fully empty into the partials swaps into the first free empty slot).
/// Returns whether anything moved (the single PlaySound(7), :991-994).
pub(crate) fn restock(
    container: &mut [ItemSlot],
    inventory: &mut [ItemSlot; crate::item::INV],
) -> bool {
    let mut moved_any = false;
    let mut demand_types: Vec<u16> = Vec::new();
    let mut partial: Vec<usize> = Vec::new();
    let mut empty_slots: Vec<usize> = Vec::new();
    for i in (0..50).rev() {
        let sl = &inventory[i];
        if sl.count == 0 {
            empty_slots.push(i);
        } else if crate::registry::max_stack(sl.kind) > 1 {
            demand_types.push(sl.kind);
            if sl.count < crate::registry::max_stack(sl.kind) {
                partial.push(i);
            }
        }
    }
    // The supply scan (:919-924): the `hashSet.Contains` gate governs the
    // whole loop — unprefixed demanded types only.
    for i in 0..container.len() {
        let cap_kind = container[i].kind;
        if container[i].count == 0 || container[i].prefix != 0 || !demand_types.contains(&cap_kind)
        {
            continue;
        }
        // The partial fill (:925-965): the capped fill into the demand-side
        // partials; `emptied` is the C#'s `flag2` (:955-958).
        let mut emptied = false;
        for &p in &partial {
            let target = &mut inventory[p];
            if !can_stack(target, &container[i]) {
                continue;
            }
            let cap = crate::registry::max_stack(target.kind) - target.count;
            let moved = container[i].count.min(cap);
            target.count += moved;
            container[i].count -= moved;
            moved_any = moved_any || moved > 0;
            if container[i].count == 0 {
                container[i] = ItemSlot {
                    kind: 0,
                    name: "",
                    count: 0,
                    prefix: 0,
                };
                emptied = true;
                break;
            }
        }
        // The ammo-into-empty bonus (:966-989): a demanded ammo stack that
        // did NOT fully empty into the partials, with an empty slot left,
        // swaps into the first free empty slot (the descending scan's first
        // hit, :970-988 — `list2.RemoveAt(k)` consumes it, and the filled
        // slot joins the demand-side partials, :984-985, so later stacks
        // can pour into it).
        if emptied || empty_slots.is_empty() {
            continue;
        }
        let is_ammo = crate::registry::get(container[i].kind).is_some_and(|d| d.ammo != 0);
        if !is_ammo {
            continue;
        }
        let e = empty_slots.remove(0);
        std::mem::swap(&mut inventory[e], &mut container[i]);
        moved_any = true;
        partial.push(e);
    }
    moved_any
}

#[cfg(test)]
mod tests {
    use super::*;
    fn slot(kind: u16, count: u32) -> ItemSlot {
        ItemSlot {
            kind,
            name: "",
            count,
            prefix: 0,
        }
    }

    #[test]
    fn loot_all_drains_into_the_player_and_routes_the_coins() {
        // `ChestUI.LootAll` (:393-473): the container drains; the coins ride
        // `GetItem`'s coin rows, the stacks ride the pickup; the overflow
        // stays in the container.
        let mut container = [
            slot(9, 5),      // the wood stack -> the pickup
            slot(71, 50),    // the copper coins -> the coin rows
            slot(1, 1),      // the pickaxe (stack 1) -> the pickup
            ItemSlot { kind: 0, name: "", count: 0, prefix: 0 },
        ];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        let mut coins = [slot(71, 10), slot(0, 0), slot(0, 0), slot(0, 0)];
        loot_all(&mut container, &mut inventory, &mut coins, None);
        assert_eq!(coins[0].count, 60, "the coins merged into row 0");
        assert!(inventory.iter().any(|sl| sl.kind == 9 && sl.count == 5));
        assert!(inventory.iter().any(|sl| sl.kind == 1 && sl.count == 1));
        assert!(container[0].count == 0 && container[1].count == 0);
    }

    #[test]
    fn loot_all_merges_to_the_cap_and_keeps_the_overflow() {
        // `GetItem`'s returned item (:400-414): the stack merges to
        // `maxStack` (9999 for the 1.4.5 common items); a FULL same-kind
        // stack leaves the remainder in the slot.
        let mut container = [slot(9, 500)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[40] = slot(9, 9499); // cap - 500: the whole stack fits
        let mut coins = [slot(0, 0), slot(0, 0), slot(0, 0), slot(0, 0)];
        loot_all(&mut container, &mut inventory, &mut coins, None);
        assert_eq!(inventory[40].count, 9999, "the merge filled to the cap");
        assert_eq!(container[0].count, 0);
        // The true overflow: a full stack refuses.
        let mut container = [slot(9, 500)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[40] = slot(9, 9999);
        let mut coins = [slot(0, 0), slot(0, 0), slot(0, 0), slot(0, 0)];
        loot_all(&mut container, &mut inventory, &mut coins, None);
        assert_eq!(inventory[40].count, 9999);
        assert_eq!(container[0].count, 500, "no pickup slot accepted it");
    }

    #[test]
    fn loot_all_routes_the_overflow_into_the_void_vault_when_allowed() {
        // `Player.cs:39395-39398` — `LootAllFromChest`'s `CanGoIntoVoidVault`
        // (`GetItemSettings.cs:55-56`) makes the vault `GetItem`'s per-item
        // last resort: the inventory passes fail, the overflow fills the
        // vault (occupied stackable slots then empty ones), the coins still
        // ride the coin rows first. The bank variant (`LootAllFromBank`,
        // :55) passes `None` and keeps the overflow in the chest.
        let mut container = [slot(9, 5), slot(71, 50), slot(3, 500)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        for s in inventory.iter_mut() {
            *s = slot(3, 9999); // stone everywhere: nothing else fits
        }
        let mut coins = [slot(0, 0), slot(0, 0), slot(0, 0), slot(0, 0)];
        let mut vault = empty_vault();
        loot_all(&mut container, &mut inventory, &mut coins, Some(&mut vault));
        assert_eq!(coins[0].count, 50, "the coins ride the coin rows first");
        assert_eq!(container[0].count, 0, "the vault took the whole wood stack");
        assert_eq!(container[2].count, 0, "the vault took the whole stone stack");
        assert_eq!(vault[0].kind, 9);
        assert_eq!(vault[0].count, 5);
        assert_eq!(vault[1].kind, 3);
        assert_eq!(vault[1].count, 500);
        // Without the vault (`LootAllFromBank`, `GetItemSettings.cs:55`):
        // the overflow stays in the chest.
        let mut container = [slot(9, 5)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        for s in inventory.iter_mut() {
            *s = slot(3, 9999);
        }
        let mut coins = [slot(0, 0), slot(0, 0), slot(0, 0), slot(0, 0)];
        loot_all(&mut container, &mut inventory, &mut coins, None);
        assert_eq!(container[0].count, 5, "the overflow stays put");
        // A kind the vault refuses (`CanVoidVaultAccept`, `Player.cs:
        // 39425-39433` — the dirtiest-block projectile 3822) stays too.
        let mut container = [slot(3822, 5)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        for s in inventory.iter_mut() {
            *s = slot(3, 9999);
        }
        let mut coins = [slot(0, 0), slot(0, 0), slot(0, 0), slot(0, 0)];
        let mut vault = empty_vault();
        loot_all(&mut container, &mut inventory, &mut coins, Some(&mut vault));
        assert_eq!(container[0].count, 5, "CanVoidVaultAccept refused");
        assert_eq!(vault[0].count, 0);
    }

    #[test]
    fn deposit_all_merges_then_parks_the_rest() {
        // `ChestUI.DepositAll` (:540-659): the rows scan 49..=10 (the hotbar
        // stays), the merge phase then the first free slot; the favorited
        // rows stay.
        let mut container = [slot(9, 500), slot(3, 100), slot(0, 0), slot(0, 0)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[0] = slot(9, 50); // the hotbar: untouched
        inventory[20] = slot(9, 200); // merges 500 -> 700
        inventory[21] = slot(3, 999); // merges 100 -> 999, 100 left
        inventory[22] = slot(41, 1); // a non-merge kind -> the first free slot
        let fav = |_: usize| false;
        deposit_all(&mut container, &mut inventory, &fav);
        assert_eq!(container[0].count, 700);
        assert_eq!(container[1].count, 1099, "the 9999 cap takes the whole 999");
        assert!(container.iter().any(|sl| sl.kind == 41), "the leftover parked in the free slot");
        assert_eq!(inventory[20].count, 0);
        assert_eq!(inventory[21].count, 0);
        assert_eq!(inventory[0].count, 50, "the hotbar untouched");
    }

    #[test]
    fn quick_stack_matches_types_and_excludes_coins_on_the_container_side() {
        // `ChestUI.QuickStack` (:661-825) after the M3-CHEST-INT hoist: the
        // container's non-coin stacks are the targets; the coins never match
        // (the exclusion is container-side, :703-714); NO MoveCoins runs
        // here — the pre-pass is the dispatch layer's (the C# bank arm
        // :670-685 vs the world chest :687+; the dispatch-level bank
        // regression rides the clicks.rs tests).
        let mut container = [slot(9, 100), slot(71, 30), slot(0, 0)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[20] = slot(9, 150); // merges into the wood target
        inventory[21] = slot(71, 200); // a grid coin stack: no target, stays
        let mut coins = [slot(71, 5), slot(0, 0), slot(0, 0), slot(0, 0)];
        let fav = |_: usize| false;
        quick_stack(&mut container, &mut inventory, &mut coins, &fav);
        assert_eq!(container[0].count, 250, "the wood merged");
        assert_eq!(container[1].count, 30, "the chest's coin stack is not a target");
        assert_eq!(inventory[20].count, 0);
        assert_eq!(inventory[21].count, 200, "no MoveCoins ran (the dispatch owns it)");
        assert_eq!(coins[0].count, 5, "the coin rows untouched");
    }

    #[test]
    fn move_coins_is_a_noop_when_the_container_holds_no_coins() {
        // `ChestUI.cs:1044`/`:1048-1051` — `flag2` is set only when the
        // CONTAINER held a coin stack and `if (!flag2) return 0L;` follows:
        // a coin-less container never pulls the player's coins.
        let mut container = [slot(9, 100), slot(0, 0), slot(0, 0)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[20] = slot(71, 200); // a grid coin stack
        let mut coins = [slot(71, 60), slot(0, 0), slot(0, 0), slot(0, 0)];
        let fav = |_: usize| false;
        move_coins(&mut container, &mut inventory, &mut coins, &fav);
        assert_eq!(container[0].count, 100, "the container untouched");
        assert_eq!(inventory[20].count, 200, "the grid coins stay put");
        assert_eq!(coins[0].count, 60, "the coin rows stay put");
    }

    #[test]
    fn move_coins_pulls_the_player_coins_and_checks_the_pinv_favorite_space() {
        // `ChestUI.cs:1052-1081` — with a coin in the container the pull
        // runs; the favorited check reads each coin item's own flag over the
        // pInv indices — the coin rows are pInv 50-53 (:1054), so a GRID
        // favorite (hotbar slot 0) must not alias onto coin row 0.
        let mut container = [slot(71, 10)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[0] = slot(71, 20); // the hotbar slot 0: favorited
        inventory[20] = slot(71, 30);
        let mut coins = [slot(71, 5), slot(0, 0), slot(0, 0), slot(0, 0)];
        let fav = |i: usize| i == 0; // favorite_slots = [0] — a grid index
        move_coins(&mut container, &mut inventory, &mut coins, &fav);
        // The favorited grid stack stays (:1054's own-flag check); the
        // unfavored grid stack and the coin row pull (10 + 30 + 5 = 45 — no
        // carry at 45).
        assert_eq!(inventory[0].count, 20, "the favorited grid stack stays");
        assert_eq!(inventory[20].count, 0);
        assert_eq!(coins[0].count, 0, "coin row 0 pulled — pInv 50, not grid 0");
        assert_eq!(container[0].kind, 71);
        assert_eq!(container[0].count, 45, "the tallies consolidated into the container");
    }

    #[test]
    fn restock_fills_from_unprefixed_supply_only() {
        // `ChestUI.Restock` (:871-995): the container -> the player; the
        // supply must be UNPREFIXED demanded types (:919-924 — the
        // `hashSet.Contains` gate governs the whole loop, the ammo bonus
        // included); the capped fill.
        let mut container = [
            slot(40, 30), // the arrows: demanded (partial in inv)
            slot(40, 10),
            slot(40, 20),
        ];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[20] = ItemSlot { kind: 40, name: "", count: 50, prefix: 0 }; // partial
        inventory[21] = slot(43, 5); // the demand type
        let moved = restock(&mut container, &mut inventory);
        assert!(moved);
        assert_eq!(inventory[20].count, 110, "the 9999 cap takes all 60");
        assert!(container[0].count == 0 && container[1].count == 0 && container[2].count == 0);
        // The prefixed supply is refused everywhere (:919-924's gate — the
        // ammo bonus rides the same gated scan, so a prefixed stack never
        // moves via restock).
        let mut container = [ItemSlot { kind: 40, name: "", count: 5, prefix: 2 }];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[21] = slot(43, 5);
        let moved = restock(&mut container, &mut inventory);
        assert!(!moved, "the prefixed stack is not a restock source");
        assert_eq!(container[0].count, 5);
        // The UNPREFIXED undemanded ammo stack: :921's gate skips it before
        // the :966-989 bonus can see it — nothing moves.
        let mut container = [slot(40, 5)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[21] = slot(43, 5);
        let moved = restock(&mut container, &mut inventory);
        assert!(!moved, "an undemanded stack is not a restock source");
        assert_eq!(container[0].count, 5, "the chest stack never moved");
        assert!(inventory.iter().all(|sl| sl.kind != 40), "the inventory untouched");
    }

    #[test]
    fn restock_bonus_swaps_a_demanded_ammo_remainder_into_an_empty_slot() {
        // `ChestUI.cs:966-989` — the bonus: a demanded ammo stack that did
        // NOT fully empty into the partials (`flag2 == false`, :955-964)
        // swaps into the first free empty slot (the descending scan's first
        // hit, :970-988 — `list2.RemoveAt(k)` consumes it, and the filled
        // slot joins the partials, :984-985). A stack that fully emptied,
        // a non-ammo kind, or no empty slot skips (:966).
        let mut container = [slot(40, 20)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[20] = slot(40, 9990); // partial: room 9 — the fill leaves 11
        let moved = restock(&mut container, &mut inventory);
        assert!(moved);
        assert_eq!(inventory[20].count, 9999, "the partial filled to the cap");
        assert_eq!(inventory[49].kind, 40, "the remainder swapped into slot 49");
        assert_eq!(inventory[49].count, 11);
        assert_eq!(container[0].count, 0, "the swap emptied the container slot");
        // A stack that fully emptied into the partials skips the bonus
        // (:955-964's `flag2` → :966's `continue`).
        let mut container = [slot(40, 5)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[20] = slot(40, 9990); // room 9 takes the whole 5
        let moved = restock(&mut container, &mut inventory);
        assert!(moved);
        assert_eq!(inventory[20].count, 9995);
        assert_eq!(inventory[49].count, 0, "no bonus after a full empty");
        assert_eq!(container[0].count, 0);
        // A non-ammo remainder never swaps (:966's `item.ammo == 0`).
        let mut container = [slot(9, 20)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[20] = slot(9, 9990); // room 9 leaves 11
        let moved = restock(&mut container, &mut inventory);
        assert!(moved, "the partial fill moved");
        assert_eq!(inventory[20].count, 9999);
        assert_eq!(container[0].count, 11, "the wood remainder stays in the chest");
        assert_eq!(inventory[49].count, 0, "no bonus for a non-ammo kind");
    }



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
