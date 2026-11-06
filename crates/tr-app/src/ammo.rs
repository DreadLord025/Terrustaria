//! The ammo system (`Player.PickAmmo`, `Player.cs:53458-53676`): the pick
//! order, the drain and the not-consume rolls.
//!
//! The pick order is `Player.AmmoSlotOrder_Default` (`Player.cs:57141-57144`):
//! the coin slots 50-53, the ammo slots 54-57, then the main grid 0-53 — the
//! port's `equip.coins` → `equip.ammo` → `App.inventory`. The vanilla's
//! `ammoCyclingMode` defaults to `None` (no cycling), so the first matching
//! slot in the order wins.
//!
//! The drain lives inside `PickAmmo` itself
//! (`Player.cs:53667-53672`: `if (!flag && item.consumable) item.stack--`) —
//! `flag` starts as the `dontConsume` argument (`ItemID.Sets.gunProj` — only
//! 4 special endgame items, `ItemID.cs:1327`) and the not-consume rolls set
//! it. The port's rolls cover the shooters whose items the registry carries.

use crate::inv::EquipSlots;
use crate::item::{ItemSlot, INV};

/// The picked slot's location — the drain mutates through it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AmmoSource {
    Coin(usize),
    Ammo(usize),
    Grid(usize),
}

/// `Player.PickAmmo_PickAmmoItem` + `PickAmmo_IterateRange`
/// (`Player.cs:53727-53753`, `:53685-53726`): the first slot in the order
/// whose item's `ammo` field matches the weapon's `useAmmo` and whose stack
/// is positive. `None` = no ammo — the shot is refused (`canShoot` stays
/// false, `Player.cs:53462-53466`).
pub fn pick(
    inventory: &[ItemSlot; INV],
    equip: &EquipSlots,
    use_ammo: i32,
) -> Option<AmmoSource> {
    if use_ammo <= 0 {
        return None;
    }
    let matches = |slot: &ItemSlot| -> bool {
        slot.count > 0
            && crate::registry::get(slot.kind).is_some_and(|d| d.ammo == use_ammo)
    };
    for (i, slot) in equip.coins.iter().enumerate() {
        if matches(slot) {
            return Some(AmmoSource::Coin(i));
        }
    }
    for (i, slot) in equip.ammo.iter().enumerate() {
        if matches(slot) {
            return Some(AmmoSource::Ammo(i));
        }
    }
    for (i, slot) in inventory.iter().enumerate() {
        if matches(slot) {
            return Some(AmmoSource::Grid(i));
        }
    }
    None
}

/// The slot read through a [`AmmoSource`].
pub fn slot<'a>(
    source: AmmoSource,
    inventory: &'a [ItemSlot; INV],
    equip: &'a EquipSlots,
) -> &'a ItemSlot {
    match source {
        AmmoSource::Coin(i) => &equip.coins[i],
        AmmoSource::Ammo(i) => &equip.ammo[i],
        AmmoSource::Grid(i) => &inventory[i],
    }
}

/// The drain half (`Player.cs:53667-53672`): `stack--`, `TurnToAir` at 0.
/// Returns false when the stack was already empty (the caller re-checks).
pub fn drain(source: AmmoSource, inventory: &mut [ItemSlot; INV], equip: &mut EquipSlots) -> bool {
    let slot = match source {
        AmmoSource::Coin(i) => &mut equip.coins[i],
        AmmoSource::Ammo(i) => &mut equip.ammo[i],
        AmmoSource::Grid(i) => &mut inventory[i],
    };
    if slot.count == 0 {
        return false;
    }
    slot.count -= 1;
    if slot.count == 0 {
        *slot = crate::item::EMPTY_SLOT;
    }
    true
}

/// The not-consume rolls (`Player.cs:53563-53660` + the tail) — the shooters
/// whose items the registry carries, verbatim. `true` = the shot skips the
/// drain. The accessory/buff rolls (`magicQuiver`/`ammoBox`/`ammoPotion`/
/// `chloroAmmoCost80`/`ammoCost80/75`) wait for their systems; the volley
/// specials (the Clockwork's `!ItemAnimationJustStarted`, the Fairy Queen's
/// first-8-ticks) have no port volley state — every shot is the first.
pub fn not_consume(shooter: u16, rng: &mut u64) -> bool {
    match shooter {
        // `Player.cs:53568-53570` — the Minishark: 1/3.
        98 => next(rng) % 3 == 0,
        // `:53596-53598` — the Megashark: 1/2.
        533 => next(rng) % 2 == 0,
        // `:53593-53595` — the Gatligator: 1/2.
        2270 => next(rng) % 2 == 0,
        // `:53599-53601` — the Candy Corn Rifle: 1/3.
        1782 => next(rng) % 3 == 0,
        // `:53602-53604` — the Chain Gun: 2/3 (the code's `Next(3) != 0`).
        1929 => next(rng) % 3 != 0,
        // `:53605-53607` — the SDMG: 2/3.
        1553 => next(rng) % 3 != 0,
        _ => false,
    }
}

/// `Main.rand.Next(n)` against the stream (the splitmix mix of `rain.rs`).
fn next(rng: &mut u64) -> u32 {
    *rng = rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *rng;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    (z ^ (z >> 31)) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inv::EquipSlots;

    fn inv_with(entries: &[(usize, u16, u32)]) -> [ItemSlot; INV] {
        let mut inv = [crate::item::EMPTY_SLOT; INV];
        for &(i, kind, count) in entries {
            inv[i] = ItemSlot {
                kind,
                name: crate::registry::name(kind),
                count,
            };
        }
        inv
    }

    fn equip_with_ammo(entries: &[(usize, u16, u32)]) -> EquipSlots {
        let mut e = EquipSlots::empty();
        for &(i, kind, count) in entries {
            e.ammo[i] = ItemSlot {
                kind,
                name: crate::registry::name(kind),
                count,
            };
        }
        e
    }

    #[test]
    fn the_pick_walks_coins_then_ammo_then_the_grid() {
        // `AmmoSlotOrder_Default` (`Player.cs:57141`): the coins 50-53, the
        // ammo 54-57, the grid 0-53 — the first `ammo == useAmmo` match wins.
        let inv = inv_with(&[(5, 40, 10), (10, 40, 99)]);
        let equip = equip_with_ammo(&[(0, 40, 3)]);
        let pick = super::pick(&inv, &equip, 40).expect("ammo exists");
        assert_eq!(pick, AmmoSource::Ammo(0), "the ammo slot precedes the grid");

        // No ammo-slot entry: the grid's first match wins.
        let equip = EquipSlots::empty();
        let pick = super::pick(&inv, &equip, 40).expect("grid ammo");
        assert_eq!(pick, AmmoSource::Grid(5), "the grid order is slot order");

        // The coin gun's coins: useAmmo 71 matches the coins slots first.
        let coin_inv = inv_with(&[]);
        let mut equip = EquipSlots::empty();
        equip.coins[1] = ItemSlot { kind: 71, name: "Copper Coin", count: 30 };
        let pick = super::pick(&coin_inv, &equip, 71).expect("coins");
        assert_eq!(pick, AmmoSource::Coin(1));
    }

    #[test]
    fn the_pick_refuses_the_wrong_ammo_and_the_empty_stacks() {
        // The Musket (96) wants Bullet (97); the arrows (ammo 40) don't match.
        let inv = inv_with(&[(3, 40, 99)]);
        let equip = EquipSlots::empty();
        assert_eq!(super::pick(&inv, &equip, 97), None);
        // An empty stack doesn't match (`item.stack > 0`, `Player.cs:53699`).
        let inv = inv_with(&[(3, 97, 0)]);
        assert_eq!(super::pick(&inv, &equip, 97), None);
        // useAmmo <= 0 refuses outright (the melee weapons never pick).
        assert_eq!(super::pick(&inv, &equip, 0), None);
    }

    #[test]
    fn the_drain_decrements_and_turns_to_air() {
        // `Player.cs:53667-53672`: `stack--`, `TurnToAir` at 0.
        let mut inv = inv_with(&[(3, 97, 2)]);
        let mut equip = EquipSlots::empty();
        let src = pick(&inv, &equip, 97).unwrap();
        assert!(drain(src, &mut inv, &mut equip));
        assert_eq!(inv[3].count, 1);
        assert!(drain(src, &mut inv, &mut equip));
        assert_eq!(inv[3].count, 0);
        assert_eq!(inv[3].kind, 0, "TurnToAir");
    }

    #[test]
    fn the_drain_hits_the_equip_slots() {
        let mut inv = inv_with(&[]);
        let mut equip = equip_with_ammo(&[(2, 40, 5)]);
        let src = pick(&inv, &equip, 40).unwrap();
        assert_eq!(src, AmmoSource::Ammo(2));
        assert!(drain(src, &mut inv, &mut equip));
        assert_eq!(equip.ammo[2].count, 4);
    }

    #[test]
    fn the_not_consume_rolls_are_seed_deterministic() {
        // The shooters' rolls (`Player.cs:53568-53607`) — the same seed gives
        // the same save pattern.
        let mut a = 42u64;
        let mut b = 42u64;
        let a_seq: Vec<bool> = (0..30).map(|_| not_consume(98, &mut a)).collect();
        let b_seq: Vec<bool> = (0..30).map(|_| not_consume(98, &mut b)).collect();
        assert_eq!(a_seq, b_seq, "byte-equal patterns");
        assert!(a_seq.contains(&true), "the Minishark saves sometimes");
        // The shooters without rolls never save.
        let mut r = 7u64;
        assert!((0..50).all(|_| !not_consume(39, &mut r)), "the wooden bow");
    }
}
