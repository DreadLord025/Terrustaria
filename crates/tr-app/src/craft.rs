//! The crafting recipe table (subset-first, M3-CRAFT): `Recipe` data ported
//! from `Recipe.cs` `SetupRecipes` over the items the registry and our item
//! subset actually have. The full ~16k-line `SetupRecipes` extraction is a
//! separate task (`tools/GenRecipes` candidate).
//!
//! Item ids reference [`crate::registry`]; station ids are `TileID` values
//! the player must stand near (`Player.AdjTiles`, `Player.cs:35661`).

use crate::registry;

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
/// wood-group recipes (`RequireGroup(RecipeGroups.Wood)`, `Recipe.cs:610-616`)
/// are expanded to concrete ids (Wood → 9) — the deviation is recorded in
/// `docs/ANALYSIS.md`.
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
}
