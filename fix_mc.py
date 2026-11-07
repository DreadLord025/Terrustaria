# -*- coding: utf-8 -*-
"""M3-STORE-2: move_coins rewrite (container-direction, the grid pull) +
the 9999-cap test expectations."""
fp = r"C:/Users/DreadLord/Desktop/TR-store2/crates/tr-app/src/bank.rs"
with open(fp, "rb") as f:
    s = f.read().decode("utf-8")
NL = "\r\n" if "\r\n" in s else "\n"
def sub(old, new, cnt=1):
    global s
    o = old.replace("\n", NL); n = new.replace("\n", NL)
    c = s.count(o)
    assert c == cnt, "expected %d, found %d of %r" % (cnt, c, old[:70])
    s = s.replace(o, n)

# 1. the hardcoded 999 in the loot_all coin arm -> max_stack
sub("""            for cs in coins.iter_mut() {
                if item.count == 0 {
                    break;
                }
                if cs.count > 0 && can_stack(cs, &item) {
                    let moved = item.count.min(999 - cs.count);
                    cs.count += moved;
                    item.count -= moved;
                }
            }
            for cs in coins.iter_mut() {
                if item.count == 0 {
                    break;
                }
                if cs.count == 0 {
                    let put = item.count.min(999);""",
"""            let coin_cap = crate::registry::max_stack(item.kind);
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
                    let put = item.count.min(coin_cap);""")

# 2. move_coins: the full rewrite of the body between the signature and restock
start = s.find("pub(crate) fn move_coins(")
end = s.find("/// `ChestUI.DepositAll`")
assert 0 < start < end
NEW_MC = """pub(crate) fn move_coins(
    container: &mut [ItemSlot],
    inventory: &mut [ItemSlot; crate::item::INV],
    coins: &mut [ItemSlot; 4],
    is_favorite: &dyn Fn(usize) -> bool,
) {
    // The vanilla consolidates INTO the container (`MoveCoins(pInv, cInv)`,
    // :997-1185): pass 1 sucks the container's coin stacks into the
    // tallies (:1011-1046), pass 2 pulls the player's un-favorited coin
    // stacks — the whole 0..58 range, the grid rows included (:1052-1080's
    // `j != 58` skip); the carry rolls 100-per-tier (:1082-1089); the
    // container's free slots refill largest-first (:1115-1147); whatever
    // does not fit returns to the player's coin rows (:1090-1208's tail).
    let mut tallies = [0u64; 4];
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
            }
        }
    }
    let coin_cap = |d: usize| crate::registry::max_stack(COIN_TYPES[d]);
    // The player's coin stacks: the grid rows + the coin rows, favorited
    // rows stay.
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
    for i in 0..4 {
        if coins[i].count > 0 && !is_favorite(i) {
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
    // The overflow returns to the player's coin rows (:1090-1208's tail):
    // the previously-held coin rows take their denomination back first.
    for d in 0..4 {
        while tallies[d] > 0 {
            let row = coins
                .iter()
                .position(|sl| sl.count == 0 && !is_favorite(50 + d))
                .unwrap_or(d.min(3));
            if coins[row].count > 0 && coins[row].kind != COIN_TYPES[d] {
                break;
            }
            let put = tallies[d].min(coin_cap(d).saturating_sub(coins[row].count) as u64) as u32;
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

"""
s = s[:start] + NEW_MC + s[end:]

# 3. the call-site signature (quick_stack passes inventory too)
sub("""    move_coins(container, coins, is_favorite);
    // The targets: the container's non-coin occupied stacks.""",
"""    move_coins(container, inventory, coins, is_favorite);
    // The targets: the container's non-coin occupied stacks.""")

# 4. the test expectations (9999 caps)
sub("""    #[test]
    fn loot_all_keeps_the_overflow_in_the_container() {
        // `GetItem`'s returned item (:400-414): a full inventory leaves the
        // remainder in the slot.
        let mut container = [slot(9, 500)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[40] = slot(9, 999); // a same-kind full stack caps the fill
        let mut coins = [slot(0, 0), slot(0, 0), slot(0, 0), slot(0, 0)];
        loot_all(&mut container, &mut inventory, &mut coins);
        assert_eq!(inventory[40].count, 999);
        assert_eq!(container[0].count, 500, "no pickup slot accepted it");
    }""",
"""    #[test]
    fn loot_all_merges_to_the_cap_and_keeps_the_overflow() {
        // `GetItem`'s returned item (:400-414): the stack merges to
        // `maxStack` (9999 for the 1.4.5 common items); a FULL same-kind
        // stack leaves the remainder in the slot.
        let mut container = [slot(9, 500)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[40] = slot(9, 9499); // cap - 500: the whole stack fits
        let mut coins = [slot(0, 0), slot(0, 0), slot(0, 0), slot(0, 0)];
        loot_all(&mut container, &mut inventory, &mut coins);
        assert_eq!(inventory[40].count, 9999, "the merge filled to the cap");
        assert_eq!(container[0].count, 0);
        // The true overflow: a full stack refuses.
        let mut container = [slot(9, 500)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[40] = slot(9, 9999);
        let mut coins = [slot(0, 0), slot(0, 0), slot(0, 0), slot(0, 0)];
        loot_all(&mut container, &mut inventory, &mut coins);
        assert_eq!(inventory[40].count, 9999);
        assert_eq!(container[0].count, 500, "no pickup slot accepted it");
    }""")

sub("""        let fav = |_: usize| false;
        deposit_all(&mut container, &mut inventory, &fav);
        assert_eq!(container[0].count, 700);
        assert_eq!(container[1].count, 999);
        assert!(container.iter().any(|sl| sl.kind == 41), "the leftover parked");
        assert_eq!(inventory[20].count, 0);
        assert_eq!(inventory[21].count, 100, "the capped merge left 100");
        assert_eq!(inventory[0].count, 50, "the hotbar untouched");""",
"""        let fav = |_: usize| false;
        deposit_all(&mut container, &mut inventory, &fav);
        assert_eq!(container[0].count, 700);
        assert_eq!(container[1].count, 1099, "the 9999 cap takes the whole 999");
        assert!(container.iter().any(|sl| sl.kind == 41), "the leftover parked");
        assert_eq!(inventory[20].count, 0);
        assert_eq!(inventory[21].count, 0);
        assert_eq!(inventory[0].count, 50, "the hotbar untouched");""")

sub("""        let mut container = [slot(9, 100), slot(71, 30), slot(0, 0)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[20] = slot(9, 150); // merges into the wood target
        inventory[21] = slot(71, 200); // a coin: no target -> parked in a free slot
        let mut coins = [slot(71, 5), slot(0, 0), slot(0, 0), slot(0, 0)];
        let fav = |_: usize| false;
        quick_stack(&mut container, &mut inventory, &mut coins, &fav);
        assert_eq!(container[0].count, 250, "the wood merged");
        assert!(container.iter().any(|sl| sl.kind == 71 && sl.count == 205), "the coins consolidated: MoveCoins + the parked stack");
        assert_eq!(inventory[20].count, 0);""",
"""        let mut container = [slot(9, 100), slot(71, 30), slot(0, 0)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[20] = slot(9, 150); // merges into the wood target
        inventory[21] = slot(71, 200); // a grid coin stack: MoveCoins pulls it
        let mut coins = [slot(71, 5), slot(0, 0), slot(0, 0), slot(0, 0)];
        let fav = |_: usize| false;
        quick_stack(&mut container, &mut inventory, &mut coins, &fav);
        assert_eq!(container[0].count, 250, "the wood merged");
        assert!(container.iter().any(|sl| sl.kind == 71 && sl.count == 235), "the coins consolidated into the container (30+5+200)");
        assert_eq!(inventory[20].count, 0);
        assert_eq!(inventory[21].count, 0, "the grid coin stack drained");""")

sub("""        let mut container = [slot(40, 30),                       // the arrows: demanded (partial in inv)
            ItemSlot { kind: 40, name: "", count: 10, prefix: 3 }, // a prefixed stack: refused
            slot(40, 20),
        ];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[20] = ItemSlot { kind: 40, name: "", count: 50, prefix: 0 }; // partial
        inventory[21] = slot(43, 5); // the demand type
        let moved = restock(&mut container, &mut inventory);
        assert!(moved);
        assert_eq!(inventory[20].count, 100, "the caps: 50 + 30 + 20");
        assert!(container[0].count == 0 && container[2].count == 0);
        assert_eq!(container[1].count, 10, "the prefixed stack refused");
        // The prefixed ammo type is undemanded -> the ammo bonus swaps it into
        // the empty slot (:966-989).
        let mut container = [ItemSlot { kind: 40, name: "", count: 5, prefix: 2 }];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[21] = slot(43, 5);
        let _ = restock(&mut container, &mut inventory);
        assert!(inventory.iter().any(|sl| sl.kind == 40 && sl.prefix == 2), "the bonus moved it in");""",
"""        let mut container = [
            slot(40, 30),   // the arrows: demanded (partial in inv)
            slot(40, 10),
            slot(40, 20),
        ];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[20] = ItemSlot { kind: 40, name: "", count: 50, prefix: 0 }; // partial
        inventory[21] = slot(43, 5); // the demand type
        let moved = restock(&mut container, &mut inventory);
        assert!(moved);
        assert_eq!(inventory[20].count, 100, "the caps: 50 + 30 + 10 + 20");
        assert!(container[0].count == 0 && container[1].count == 0 && container[2].count == 0);
        // The prefixed supply is refused everywhere (:919-924's gate — the
        // ammo bonus rides the same scan's tail, so a prefixed stack never
        // moves via restock).
        let mut container = [ItemSlot { kind: 40, name: "", count: 5, prefix: 2 }];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[21] = slot(43, 5);
        let moved = restock(&mut container, &mut inventory);
        assert!(!moved, "the prefixed stack is not a restock source");
        assert_eq!(container[0].count, 5);
        // The UNPREFIXED undemanded ammo stack: the ammo bonus swaps it into
        // the empty slot (:966-989).
        let mut container = [slot(40, 5)];
        let mut inventory: [ItemSlot; crate::item::INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0, prefix: 0 });
        inventory[21] = slot(43, 5);
        let moved = restock(&mut container, &mut inventory);
        assert!(moved, "the bonus moved it in");
        assert!(inventory.iter().any(|sl| sl.kind == 40 && sl.prefix == 0));""")

with open(fp, "wb") as f:
    f.write(s.encode("utf-8"))
print("fixed")
