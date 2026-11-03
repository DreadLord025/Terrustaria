//! The crafting recipe table (subset-first, M3-CRAFT): `Recipe` data ported
//! from `Recipe.cs` `SetupRecipes` over the items the registry and our item
//! subset actually have. The full ~16k-line `SetupRecipes` extraction is a
//! separate task (`tools/GenRecipes` candidate).
//!
//! Item ids reference [`crate::registry`]; station ids are `TileID` values
//! the player must stand near (`Player.AdjTiles`, `Player.cs:35661`).

use crate::item::{ItemSlot, INV};
use crate::player::{self, Player};
use crate::registry;
use tr_core::World;

/// One crafting recipe: `Recipe.createItem` + `requiredItem[]` +
/// `requiredTile` (`Recipe.cs:13`). Const-constructible (slice fields) so the
/// table is a static.
pub(crate) struct Recipe {
    /// `(item id, stack)` — `createItem`.
    pub(crate) create: (u16, u32),
    /// `(item id, count)` pairs — `requiredItem[]`.
    pub(crate) required: &'static [(u16, u32)],
    /// Station tile ids the player must stand near (`requiredTile`);
    /// empty = craftable by hand.
    pub(crate) stations: &'static [u16],
}

/// The subset recipe table. Every entry cites its `Recipe.cs` registration;
/// group recipes (`RequireGroup(RecipeGroups.Wood)` / `(IronBar)`,
/// `Recipe.cs:610-616` / `:4738`) are expanded to concrete ids (Wood → 9,
/// Iron Bar → 22) — the deviation is recorded in `docs/ANALYSIS.md`.
pub(crate) static RECIPES: &[Recipe] = &[
    // Recipe.cs:610-615 — Torch x3: 1 Gel + 1 Wood, by hand.
    Recipe {
        create: (8, 3),
        required: &[(23, 1), (9, 1)],
        stations: &[],
    },
    // Recipe.cs:4751-4754 — Work Bench: 10 Wood, by hand.
    Recipe {
        create: (36, 1),
        required: &[(9, 10)],
        stations: &[],
    },
    // Recipe.cs:4722-4726 — Wooden Door: 6 Wood, at a work bench.
    Recipe {
        create: (25, 1),
        required: &[(9, 6)],
        stations: &[18],
    },
    // Recipe.cs:4727-4731 — Wooden Chair: 4 Wood, at a work bench.
    Recipe {
        create: (34, 1),
        required: &[(9, 4)],
        stations: &[18],
    },
    // Recipe.cs:4732-4739 — Chest: 8 Wood + 2 Iron Bar, at a work bench
    // (`RequireGroup(RecipeGroups.IronBar)` expanded to Iron Bar 22).
    Recipe {
        create: (48, 1),
        required: &[(9, 8), (22, 2)],
        stations: &[18],
    },
    // Recipe.cs:4746-4750 — Wooden Table: 8 Wood, at a work bench.
    Recipe {
        create: (32, 1),
        required: &[(9, 8)],
        stations: &[18],
    },
    // Recipe.cs:4568-4572 — Stone Wall: 1 Stone, at a work bench.
    Recipe {
        create: (26, 1),
        required: &[(3, 1)],
        stations: &[18],
    },
    // Recipe.cs:4521-4524 — Dirt Wall: 1 Dirt, at a work bench.
    Recipe {
        create: (30, 1),
        required: &[(2, 1)],
        stations: &[18],
    },
    // Recipe.cs:9606-9614 — Furnace: 20 Stone + 4 Wood + 3 Torch, at a work bench.
    Recipe {
        create: (33, 1),
        required: &[(3, 20), (9, 4), (8, 3)],
        stations: &[18],
    },
    // Recipe.cs:9817-9820 — Iron Bar: 3 Iron Ore, at a furnace.
    Recipe {
        create: (22, 1),
        required: &[(11, 3)],
        stations: &[17],
    },
    // Recipe.cs:9822-9826 — Iron Anvil: 5 Iron Bar, at a work bench.
    Recipe {
        create: (35, 1),
        required: &[(22, 5)],
        stations: &[18],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dw_1_1_table_integrity() {
        // Every create/required item id resolves in the registry; counts are
        // positive; no duplicate create ids.
        assert!(!RECIPES.is_empty());
        let mut seen = Vec::new();
        for r in RECIPES {
            assert!(registry::get(r.create.0).is_some(), "create id {}", r.create.0);
            assert!(r.create.1 > 0);
            assert!(!r.required.is_empty(), "recipe {:?}", r.create);
            for (id, count) in r.required {
                assert!(
                    registry::get(*id).is_some(),
                    "required id {id} for create {}",
                    r.create.0
                );
                assert!(*count > 0);
            }
            assert!(!seen.contains(&r.create.0), "duplicate create {}", r.create.0);
            seen.push(r.create.0);
        }
    }

    #[test]
    fn test_dw_1_2_workbench_reference_recipe() {
        // `Recipe.cs:4751-4754`: 10x Wood -> Work Bench, no station.
        let wb = RECIPES
            .iter()
            .find(|r| r.create.0 == 36)
            .expect("work bench recipe");
        assert_eq!(wb.create, (36, 1));
        assert_eq!(wb.required, &[(9, 10)]);
        assert!(wb.stations.is_empty());
    }

    #[test]
    fn torch_recipe_expands_the_wood_group() {
        // `Recipe.cs:610-615`: 1 Gel + 1 Wood -> 3 Torches. The original
        // requires `RecipeGroup(Wood)`; the subset expansion pins Wood (9).
        let torch = RECIPES.iter().find(|r| r.create.0 == 8).unwrap();
        assert_eq!(torch.create, (8, 3));
        assert_eq!(torch.required, &[(23, 1), (9, 1)]);
        assert!(torch.stations.is_empty());
    }

    #[test]
    fn test_dw_2_3_furniture_recipes_match_recipe_cs() {
        // Door :4722-4726, chair :4727-4731, chest :4732-4739, table
        // :4746-4750 — all at a work bench (18). The chest's
        // `RequireGroup(RecipeGroups.IronBar)` expands to Iron Bar (22).
        let door = RECIPES.iter().find(|r| r.create.0 == 25).unwrap();
        assert_eq!(door.required, &[(9, 6)]);
        assert_eq!(door.stations, &[18]);
        let chair = RECIPES.iter().find(|r| r.create.0 == 34).unwrap();
        assert_eq!(chair.required, &[(9, 4)]);
        assert_eq!(chair.stations, &[18]);
        let chest = RECIPES.iter().find(|r| r.create.0 == 48).unwrap();
        assert_eq!(chest.required, &[(9, 8), (22, 2)]);
        assert_eq!(chest.stations, &[18]);
        let table = RECIPES.iter().find(|r| r.create.0 == 32).unwrap();
        assert_eq!(table.required, &[(9, 8)]);
        assert_eq!(table.stations, &[18]);
    }
}

/// `Player.AdjTiles` (`Player.cs:35661`) with `TileReachCheckSettings.Simple`
/// (`TileReachCheckSettings.cs:81-86`: reach X 5 / Y 3, `GetRanges` clamped by
/// `TileReachLimit = 20`; region math `:45-54`): the active tile kinds around
/// the player hitbox. Liquids/water-source flags are not modelled.
pub(crate) fn adjacent_tiles(world: &World, player: &Player) -> Vec<u16> {
    const REACH_X: i32 = 5;
    const REACH_Y: i32 = 3;
    let lx = (player.x / 16.0).floor() as i32 - REACH_X;
    let hx = ((player.x + player::WIDTH as f32) / 16.0).ceil() as i32 - 1 + REACH_X;
    let ly = (player.y / 16.0).floor() as i32 - REACH_Y;
    let hy = ((player.y + player::HEIGHT as f32) / 16.0).ceil() as i32 - 1 + REACH_Y;
    let mut out = Vec::new();
    for i in lx.max(0)..=hx.min(world.width as i32 - 1) {
        for j in ly.max(0)..=hy.min(world.height as i32 - 1) {
            if let Some(t) = world.tile(i, j) {
                if t.active() && !out.contains(&t.kind) {
                    out.push(t.kind);
                }
            }
        }
    }
    out
}

/// Total count of one item kind across the whole inventory.
fn count_kind(inv: &[ItemSlot; INV], kind: u16) -> u32 {
    inv.iter()
        .filter(|s| s.kind == kind)
        .map(|s| s.count)
        .sum()
}

/// `CraftingRequests.ConsumeItemsFrom` (`CraftingRequests.cs:244-290`): one
/// pass in slot-index order — partial decrement when `stack > toConsume`,
/// otherwise the slot is cleared entirely.
fn consume_kind(inv: &mut [ItemSlot; INV], kind: u16, mut to_consume: u32) {
    if to_consume == 0 {
        return;
    }
    for slot in inv.iter_mut() {
        if to_consume == 0 {
            break;
        }
        if slot.kind == kind && slot.count > 0 {
            if slot.count > to_consume {
                slot.count -= to_consume;
                to_consume = 0;
            } else {
                to_consume -= slot.count;
                *slot = crate::item::EMPTY_SLOT;
            }
        }
    }
}

/// All requirements met: every ingredient counted across the inventory and
/// every station tile adjacent to the player.
pub(crate) fn can_craft(inv: &[ItemSlot; INV], recipe: &Recipe, adj: &[u16]) -> bool {
    recipe
        .required
        .iter()
        .all(|(id, count)| count_kind(inv, *id) >= *count)
        && recipe.stations.iter().all(|t| adj.contains(t))
}

/// Consume the ingredients (single pass, `ConsumeItemsFrom` semantics) and
/// return the created `(item id, stack)`. Giving the result to the inventory
/// (or dropping it when full — vanilla drops, `Item.giveItem`) is the
/// caller's duty; the full-inventory case still crafts, like the original.
pub(crate) fn try_craft(
    inv: &mut [ItemSlot; INV],
    recipe: &Recipe,
    adj: &[u16],
) -> Option<(u16, u32)> {
    if !can_craft(inv, recipe, adj) {
        return None;
    }
    for (id, count) in recipe.required {
        consume_kind(inv, *id, *count);
    }
    Some(recipe.create)
}

#[cfg(test)]
mod craft_actions_tests {
    use super::*;
    use crate::item::{slot_for_pickup, ItemSlot, INV};
    use crate::player::Player;
    use tr_core::tile::FLAG_ACTIVE;
    use tr_core::World;

    fn inv_with(entries: &[(u16, u32)]) -> [ItemSlot; INV] {
        let mut inv = [crate::item::EMPTY_SLOT; INV];
        for (id, count) in entries {
            let mut left = *count;
            while left > 0 {
                let i = slot_for_pickup(&mut inv, *id).expect("fixture overflow");
                let add = left.min(999 - inv[i].count);
                inv[i].count += add;
                left -= add;
            }
        }
        inv
    }

    fn recipe_by_create(create: u16) -> &'static Recipe {
        RECIPES.iter().find(|r| r.create.0 == create).unwrap()
    }

    fn world_with_station(x: i32, y: i32) -> World {
        let mut w = World::new("t".into(), 40, 40);
        let i = w.index(x, y);
        w.tiles[i].kind = 18; // WorkBench tile
        w.tiles[i].flags |= FLAG_ACTIVE;
        w
    }

    #[test]
    fn test_dw_2_1_can_craft_cases() {
        // Work Bench (10x Wood, by hand): enough wood -> true.
        let inv = inv_with(&[(9, 10)]);
        let wb = recipe_by_create(36);
        assert!(can_craft(&inv, wb, &[]));
        // Not enough wood -> false.
        let inv = inv_with(&[(9, 9)]);
        assert!(!can_craft(&inv, wb, &[]));
        // Torch needs Gel + Wood: only wood -> false; both -> true (by hand).
        let torch = recipe_by_create(8);
        let inv = inv_with(&[(9, 1)]);
        assert!(!can_craft(&inv, torch, &[]));
        let inv = inv_with(&[(9, 1), (23, 1)]);
        assert!(can_craft(&inv, torch, &[]));
    }

    #[test]
    fn test_dw_2_2_try_craft_consumes_and_returns() {
        // Wood spread across three slots (4 + 4 + 2): a 10x-wood recipe
        // consumes all three, exact amounts.
        let mut inv = inv_with(&[(9, 4)]);
        inv[1] = ItemSlot { kind: 9, name: "Wood", count: 4 };
        inv[2] = ItemSlot { kind: 9, name: "Wood", count: 2 };
        let wb = recipe_by_create(36);
        assert_eq!(try_craft(&mut inv, wb, &[]), Some((36, 1)));
        assert_eq!(count_kind(&inv, 9), 0);
        // The created stack is the caller's to give.
        assert_eq!(wb.create, (36, 1));
    }

    #[test]
    fn test_dw_2_3_adjacent_tiles_semantics() {
        let w = world_with_station(18, 18);
        let mut p = Player::spawn(&w);
        p.x = 20.0 * 16.0; // tile 20: region x 15..=26, y 17..=18+
        p.y = 20.0 * 16.0;
        let adj = adjacent_tiles(&w, &p);
        assert!(adj.contains(&18), "work bench adjacent: {:?}", adj);
        // Far away: the station is out of the region.
        let w2 = world_with_station(35, 30);
        let adj2 = adjacent_tiles(&w2, &p);
        assert!(!adj2.contains(&18), "{:?}", adj2);
    }

    #[test]
    fn test_dw_2_4_station_gate_and_dirty_inputs() {
        // Iron Bar requires a furnace adjacent: without one, no craft even
        // with the ingredient present.
        let inv = inv_with(&[(11, 3)]);
        let bar = recipe_by_create(22);
        assert!(!can_craft(&inv, bar, &[]));
        assert!(can_craft(&inv, bar, &[17]));
        // Empty inventory: cannot craft anything.
        let inv = [crate::item::EMPTY_SLOT; INV];
        assert!(!can_craft(&inv, recipe_by_create(36), &[]));
        // Two different ingredients at once (torch: gel + wood).
        let mut inv = inv_with(&[(23, 1)]);
        let torch = recipe_by_create(8);
        assert!(!can_craft(&inv, torch, &[]));
        slot_for_pickup(&mut inv, 9);
        inv.iter_mut().find(|s| s.kind == 9).unwrap().count = 1;
        assert!(can_craft(&inv, torch, &[]));
    }

    #[test]
    fn test_dw_2_5_full_inventory_still_crafts() {
        // Fill every slot with stone, keep 10 wood in the first slot: the
        // work bench still crafts (the result-drop duty is the caller's, like
        // vanilla `Item.giveItem` dropping on the ground).
        let mut inv = inv_with(&[(9, 10)]);
        for slot in inv.iter_mut().skip(1) {
            *slot = ItemSlot { kind: 3, name: "Stone Block", count: 999 };
        }
        let wb = recipe_by_create(36);
        assert_eq!(try_craft(&mut inv, wb, &[]), Some((36, 1)));
    }

    #[test]
    fn furniture_recipe_crafts_and_consumes() {
        // 16 wood -> work bench (by hand) leaves 6, exactly a door — which
        // then needs the bench standing next to the player.
        let mut inv = inv_with(&[(9, 16)]);
        assert_eq!(
            try_craft(&mut inv, recipe_by_create(36), &[]),
            Some((36, 1))
        );
        assert_eq!(count_kind(&inv, 9), 6);
        let door = recipe_by_create(25);
        assert!(!can_craft(&inv, door, &[]), "no station adjacent yet");
        assert_eq!(try_craft(&mut inv, door, &[18]), Some((25, 1)));
        assert_eq!(count_kind(&inv, 9), 0);
        // The chest needs the iron bar too: wood alone is refused.
        let mut inv = inv_with(&[(9, 8)]);
        assert!(!can_craft(&inv, recipe_by_create(48), &[18]));
        let mut inv = inv_with(&[(9, 8), (22, 2)]);
        assert!(can_craft(&inv, recipe_by_create(48), &[18]));
    }
}

/// The panel's recipe list: every station adjacent and at least one
/// ingredient in the inventory (vanilla shows partial recipes; the craftable
/// subset is what `can_craft` approves).
pub(crate) fn available(inv: &[ItemSlot; INV], adj: &[u16]) -> Vec<usize> {
    RECIPES
        .iter()
        .enumerate()
        .filter(|(_, r)| {
            r.stations.iter().all(|t| adj.contains(t))
                && r.required.iter().any(|(id, _)| count_kind(inv, *id) > 0)
        })
        .map(|(i, _)| i)
        .collect()
}
