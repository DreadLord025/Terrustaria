//! The chest Sort button's engine — a 1:1 port of `Terraria/UI/ItemSorting.cs`
//! (1.4.5.8), the 52-layer sorter behind `ChestUI` button id 4
//! (`ChestUI.cs:227`, `:282-308`).
//!
//! Structure mirrors the source: [`SETUP_LAYERS`] is the fixed layer list
//! `SetupWhiteLists` runs (`ItemSorting.cs:1328-1410`), [`sort_order`] is
//! `SetupSortingPriorities` (`:1437-1490`) with the damage-ranked weapon order
//! (`:1412-1435`), and [`sort`] is `ItemSorting.Sort` (`:1492-1597`).
//!
//! Faithfulness notes (all verified against the source):
//! * `Item.OriginalRarity/Damage/Defense` are `ContentSamples.ItemsByType[type]`
//!   (`Item.cs:365-369`), i.e. the pre-prefix `SetDefaults` values — the
//!   generator's `ItemDef` `rare`/`damage`/`defense`.
//! * `Item.Name` is `Lang.GetItemNameValue(type)` (`lang::item_name`); the port
//!   compares the localized name ordinally, ignoring case
//!   (`string.Compare(..., OrdinalIgnoreCase)`).
//! * Chest slots can never be favorited (`can_favorite_at` excludes ctx 3,
//!   `app/clicks.rs`), so the `!favorited` term of the `Sort` pre-filter is
//!   always true here.
//! * `Validate` (`ItemSortingLayer.Validate`, `:23-29`) keeps only the item
//!   types the layer claimed in the synthetic full-inventory sweep. Because each
//!   type is claimed by exactly one layer, that is a per-type "first claiming
//!   layer" index — [`layer_of_table`] precomputes it once. The C# build asserts
//!   the `LastNotTrash` whitelist is empty (`:1409`); the port's
//!   `setup_whitelists_covers_every_item` test asserts the same invariant.
//! * `withFeedback` only drives `ItemSlot.SetGlow` (`:1584-1587`); the resulting
//!   order is identical without it, and the port has no glow system, so it is
//!   accepted and ignored (docs/ANALYSIS.md).

use std::cmp::Ordering;
use std::sync::OnceLock;

use crate::buffs;
use crate::item::{ItemSlot, EMPTY_SLOT};
use crate::itemdata::ItemDef;
use crate::lang;
use crate::sortdata::{
    IntSet, IS_BASIC_FISH, IS_FISHING_CRATE, IS_FOOD, MOUNT_CART, PROJ_HOOK,
    SORTING_PRIORITY_EXTRACTIBLES, SORTING_PRIORITY_FOOD_BUFFS, SORTING_PRIORITY_MATERIALS,
    SORTING_PRIORITY_MISC_ACORNS, SORTING_PRIORITY_MISC_BOSS_BAGS, SORTING_PRIORITY_MISC_GEMS,
    SORTING_PRIORITY_MISC_HERBS_AND_SEEDS, SORTING_PRIORITY_MISC_IMPORTANTS,
    SORTING_PRIORITY_PAINTING, SORTING_PRIORITY_POTIONS_BUFFS,
    SORTING_PRIORITY_POTIONS_DYE_MATERIAL, SORTING_PRIORITY_ROPES, SORTING_PRIORITY_TERRAFORMING,
    SORTING_PRIORITY_TOOLS_FISHING, SORTING_PRIORITY_TOOLS_GOLF,
    SORTING_PRIORITY_TOOLS_INSTRUMENTS, SORTING_PRIORITY_TOOLS_KEYS, SORTING_PRIORITY_TOOLS_KITES,
    SORTING_PRIORITY_TOOLS_MISC, SORTING_PRIORITY_WEAPONS_RANGED, SORTING_PRIORITY_WIRING,
    TILE_FRAME_IMPORTANT,
};

/// `ItemID.Count` (`ItemID.cs`). The sort tables are indexed by item id.
const ITEM_COUNT: usize = 6196;
/// `BuffID.Count` (`BuffID.cs:45`'s neighbour).
const BUFF_COUNT: i32 = 401;

/// `Item.SetDefaults(type)`'s defaults for `kind` — the `ContentSamples` sample
/// `Item.OriginalRarity/Damage/Defense` and every filter field read.
#[inline]
fn def(kind: u16) -> &'static ItemDef {
    crate::itemdata::ITEMS
        .get(kind as usize)
        .unwrap_or(&crate::itemdata::ITEMS[0])
}

/// `ItemSlot.IsAir` for the port's shape: `type == 0 || stack == 0`.
#[inline]
fn is_empty(s: &ItemSlot) -> bool {
    s.kind == 0 || s.count == 0
}

/// `Item.CanStack` (`Item.cs:49865-49871`): same type AND same prefix.
#[inline]
fn can_stack(a: &ItemSlot, b: &ItemSlot) -> bool {
    a.kind == b.kind && a.prefix == b.prefix
}

/// `ItemSorting.CompareWithPrioritySet` (`ItemSorting.cs:156-175`). `kind` is
/// always `>= 0`, so the `typeOne/typeTwo < 0` early-out never fires; the set
/// *value* `< 0` branches are the whole behaviour.
fn compare_with_priority_set(set: &IntSet, ka: u16, kb: u16) -> Ordering {
    let va = set.get(ka);
    let vb = set.get(kb);
    if va >= 0 && vb < 0 {
        return Ordering::Less;
    }
    if va < 0 && vb >= 0 {
        return Ordering::Greater;
    }
    if va < 0 && vb < 0 {
        return Ordering::Equal;
    }
    va.cmp(&vb)
}

/// `string.Compare(inv[x].Name, inv[y].Name, OrdinalIgnoreCase)` — ordinal,
/// case-insensitive; item names are ASCII, so the ASCII lowercase compare is
/// exactly the invariant upper-casing `OrdinalIgnoreCase` performs.
fn cmp_name(a: &ItemSlot, b: &ItemSlot) -> Ordering {
    let na = lang::item_name(a.kind).to_ascii_lowercase();
    let nb = lang::item_name(b.kind).to_ascii_lowercase();
    na.cmp(&nb)
}

/// One sorting layer: `ItemSortingLayer` (`ItemSorting.cs:11-35`) — a name, the
/// `Where` filter, and the `SortIndicesStable` comparison.
struct Layer {
    name: &'static str,
    /// `itemsToSort.Where(...)` (`ItemSorting.cs:181` etc.) — the filter, on a
    /// single slot (the sort reads only per-item fields, never the vector).
    filter: fn(&ItemSlot) -> bool,
    /// The `Comparison<int>` passed to `SortIndicesStable` (`:187` etc.). The
    /// stable-sort index tie-break (`:143-154`) is added by [`sort_indices_stable`].
    compare: fn(&ItemSlot, &ItemSlot) -> Ordering,
}

macro_rules! rarity_desc {
    () => {
        |a: &ItemSlot, b: &ItemSlot| def(b.kind).rare.cmp(&def(a.kind).rare)
    };
}

/// The 52 layers in `SetupWhiteLists` order (`ItemSorting.cs:1334-1385`) — the
/// order `Validate`'s whitelists and [`layer_of_table`] are built in. The live
/// `Sort` reorders only the four weapon layers (see [`sort_order`]).
static SETUP_LAYERS: [Layer; 52] = [
    // 0 — Weapons - Melee (`:179-197`)
    Layer {
        name: "Weapons - Melee",
        filter: |s| {
            let d = def(s.kind);
            d.damage > 0
                && !d.consumable
                && d.ammo == 0
                && d.melee
                && d.pick < 1
                && d.hammer < 1
                && d.axe < 1
        },
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            db.rare.cmp(&da.rare).then(db.damage.cmp(&da.damage))
        },
    },
    // 1 — Weapons - Ranged (`:198-220`)
    Layer {
        name: "Weapons - Ranged",
        filter: |s| {
            let d = def(s.kind);
            (d.damage > 0 && !d.consumable && d.ammo == 0 && d.ranged)
                || SORTING_PRIORITY_WEAPONS_RANGED.get(s.kind) > -1
        },
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            compare_with_priority_set(&SORTING_PRIORITY_WEAPONS_RANGED, a.kind, b.kind)
                .then(db.rare.cmp(&da.rare))
                .then(db.damage.cmp(&da.damage))
        },
    },
    // 2 — Weapons - Magic (`:221-239`)
    Layer {
        name: "Weapons - Magic",
        filter: |s| {
            let d = def(s.kind);
            d.damage > 0 && !d.consumable && d.ammo == 0 && d.magic
        },
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            db.rare.cmp(&da.rare).then(db.damage.cmp(&da.damage))
        },
    },
    // 3 — Weapons - Minions (`:240-258`)
    Layer {
        name: "Weapons - Minions",
        filter: |s| {
            let d = def(s.kind);
            d.damage > 0 && !d.consumable && d.summon
        },
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            db.rare.cmp(&da.rare).then(db.damage.cmp(&da.damage))
        },
    },
    // 4 — Weapons - Assorted (`:259-277`)
    Layer {
        name: "Weapons - Assorted",
        filter: |s| {
            let d = def(s.kind);
            d.damage > 0 && d.ammo == 0 && d.pick == 0 && d.axe == 0 && d.hammer == 0
        },
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            db.rare.cmp(&da.rare).then(db.damage.cmp(&da.damage))
        },
    },
    // 5 — Weapons - Ammo (`:278-296`)
    Layer {
        name: "Weapons - Ammo",
        filter: |s| {
            let d = def(s.kind);
            d.ammo > 0 && d.damage > 0
        },
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            db.rare.cmp(&da.rare).then(db.damage.cmp(&da.damage))
        },
    },
    // 6 — Tools - Picksaws (`:297-307`)
    Layer {
        name: "Tools - Picksaws",
        filter: |s| {
            let d = def(s.kind);
            d.pick > 0 && d.axe > 0
        },
        compare: |a, b| def(a.kind).pick.cmp(&def(b.kind).pick),
    },
    // 7 — Tools - Hamaxes (`:308-318`)
    Layer {
        name: "Tools - Hamaxes",
        filter: |s| {
            let d = def(s.kind);
            d.hammer > 0 && d.axe > 0
        },
        compare: |a, b| def(a.kind).axe.cmp(&def(b.kind).axe),
    },
    // 8 — Tools - Pickaxes (`:319-329`)
    Layer {
        name: "Tools - Pickaxes",
        filter: |s| def(s.kind).pick > 0,
        compare: |a, b| def(a.kind).pick.cmp(&def(b.kind).pick),
    },
    // 9 — Tools - Axes (`:330-340`)
    Layer {
        name: "Tools - Axes",
        filter: |s| def(s.kind).axe > 0,
        compare: |a, b| def(a.kind).axe.cmp(&def(b.kind).axe),
    },
    // 10 — Tools - Hammers (`:341-351`)
    Layer {
        name: "Tools - Hammers",
        filter: |s| def(s.kind).hammer > 0,
        compare: |a, b| def(a.kind).hammer.cmp(&def(b.kind).hammer),
    },
    // 11 — Tools - Terraforming (`:352-370`)
    Layer {
        name: "Tools - Terraforming",
        filter: |s| s.kind > 0 && SORTING_PRIORITY_TERRAFORMING.get(s.kind) > -1,
        compare: |a, b| {
            SORTING_PRIORITY_TERRAFORMING
                .get(a.kind)
                .cmp(&SORTING_PRIORITY_TERRAFORMING.get(b.kind))
                .then(def(b.kind).rare.cmp(&def(a.kind).rare))
                .then(b.count.cmp(&a.count))
        },
    },
    // 12 — Tools - Fishing (`:371-416`)
    Layer {
        name: "Tools - Fishing",
        filter: |s| {
            let d = def(s.kind);
            d.fishing_pole > 0
                || d.bait > 0
                || d.quest_item
                || (s.kind > 0
                    && (IS_FISHING_CRATE.get(s.kind)
                        || IS_BASIC_FISH.get(s.kind)
                        || SORTING_PRIORITY_TOOLS_FISHING.get(s.kind) > -1))
        },
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            let mut ord =
                compare_with_priority_set(&SORTING_PRIORITY_TOOLS_FISHING, a.kind, b.kind);
            if ord == Ordering::Equal {
                ord = db.fishing_pole.cmp(&da.fishing_pole);
            }
            if ord == Ordering::Equal {
                ord = db.bait.cmp(&da.bait);
            }
            if ord == Ordering::Equal {
                ord = db.quest_item.cmp(&da.quest_item);
            }
            if ord == Ordering::Equal && b.kind > 0 && a.kind > 0 {
                if ord == Ordering::Equal {
                    ord = IS_FISHING_CRATE
                        .get(b.kind)
                        .cmp(&IS_FISHING_CRATE.get(a.kind));
                }
                if ord == Ordering::Equal {
                    ord = IS_BASIC_FISH.get(b.kind).cmp(&IS_BASIC_FISH.get(a.kind));
                }
            }
            if ord == Ordering::Equal {
                ord = db.rare.cmp(&da.rare);
            }
            if ord == Ordering::Equal {
                ord = b.count.cmp(&a.count);
            }
            ord
        },
    },
    // 13 — Tools - Golf (`:417-439`)
    Layer {
        name: "Tools - Golf",
        filter: |s| s.kind > 0 && SORTING_PRIORITY_TOOLS_GOLF.get(s.kind) > -1,
        compare: |a, b| {
            SORTING_PRIORITY_TOOLS_GOLF
                .get(a.kind)
                .cmp(&SORTING_PRIORITY_TOOLS_GOLF.get(b.kind))
                .then(def(b.kind).rare.cmp(&def(a.kind).rare))
                .then(b.count.cmp(&a.count))
        },
    },
    // 14 — Tools - Instruments (`:440-462`)
    Layer {
        name: "Tools - Instruments",
        filter: |s| s.kind > 0 && SORTING_PRIORITY_TOOLS_INSTRUMENTS.get(s.kind) > -1,
        compare: |a, b| {
            SORTING_PRIORITY_TOOLS_INSTRUMENTS
                .get(a.kind)
                .cmp(&SORTING_PRIORITY_TOOLS_INSTRUMENTS.get(b.kind))
                .then(def(b.kind).rare.cmp(&def(a.kind).rare))
                .then(b.count.cmp(&a.count))
        },
    },
    // 15 — Tools - Keys (`:463-485`)
    Layer {
        name: "Tools - Keys",
        filter: |s| s.kind > 0 && SORTING_PRIORITY_TOOLS_KEYS.get(s.kind) > -1,
        compare: |a, b| {
            SORTING_PRIORITY_TOOLS_KEYS
                .get(a.kind)
                .cmp(&SORTING_PRIORITY_TOOLS_KEYS.get(b.kind))
                .then(def(b.kind).rare.cmp(&def(a.kind).rare))
                .then(b.count.cmp(&a.count))
        },
    },
    // 16 — Tools - Kites (`:486-508`)
    Layer {
        name: "Tools - Kites",
        filter: |s| s.kind > 0 && SORTING_PRIORITY_TOOLS_KITES.get(s.kind) > -1,
        compare: |a, b| {
            SORTING_PRIORITY_TOOLS_KITES
                .get(a.kind)
                .cmp(&SORTING_PRIORITY_TOOLS_KITES.get(b.kind))
                .then(def(b.kind).rare.cmp(&def(a.kind).rare))
                .then(b.count.cmp(&a.count))
        },
    },
    // 17 — Weapons - Ammo Leftovers (`:509-527`)
    Layer {
        name: "Weapons - Ammo Leftovers",
        filter: |s| {
            let d = def(s.kind);
            d.ammo > 0
                && (s.kind as usize) < ITEM_COUNT
                && !IS_FOOD.get(s.kind)
                && SORTING_PRIORITY_MISC_ACORNS.get(s.kind) == -1
        },
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            db.rare.cmp(&da.rare).then(db.damage.cmp(&da.damage))
        },
    },
    // 18 — Tools - Misc (`:528-550`)
    Layer {
        name: "Tools - Misc",
        filter: |s| s.kind > 0 && SORTING_PRIORITY_TOOLS_MISC.get(s.kind) > -1,
        compare: |a, b| {
            SORTING_PRIORITY_TOOLS_MISC
                .get(a.kind)
                .cmp(&SORTING_PRIORITY_TOOLS_MISC.get(b.kind))
                .then(def(b.kind).rare.cmp(&def(a.kind).rare))
                .then(b.count.cmp(&a.count))
        },
    },
    // 19 — Armor - Combat (`:551-573`)
    Layer {
        name: "Armor - Combat",
        filter: |s| {
            let d = def(s.kind);
            (d.body_slot >= 0 || d.head_slot >= 0 || d.leg_slot >= 0) && !d.vanity
        },
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            db.rare
                .cmp(&da.rare)
                .then(db.defense.cmp(&da.defense))
                .then(a.kind.cmp(&b.kind))
        },
    },
    // 20 — Armor - Vanity (`:574-592`)
    Layer {
        name: "Armor - Vanity",
        filter: |s| {
            let d = def(s.kind);
            (d.body_slot >= 0 || d.head_slot >= 0 || d.leg_slot >= 0) && d.vanity
        },
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            db.rare.cmp(&da.rare).then(a.kind.cmp(&b.kind))
        },
    },
    // 21 — Armor - Accessories (`:593-619`)
    Layer {
        name: "Armor - Accessories",
        filter: |s| def(s.kind).accessory,
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            da.vanity
                .cmp(&db.vanity)
                .then(db.rare.cmp(&da.rare))
                .then(db.defense.cmp(&da.defense))
                .then(a.kind.cmp(&b.kind))
        },
    },
    // 22 — Equip - Grapple (`:620-638`)
    Layer {
        name: "Equip - Grapple",
        filter: |s| PROJ_HOOK.get(def(s.kind).projectile.unwrap_or(0)),
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            db.rare.cmp(&da.rare).then(a.kind.cmp(&b.kind))
        },
    },
    // 23 — Equip - Mount (`:639-657`)
    Layer {
        name: "Equip - Mount",
        filter: |s| {
            let d = def(s.kind);
            d.mount_type != -1 && !MOUNT_CART.get(d.mount_type as u16)
        },
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            db.rare.cmp(&da.rare).then(a.kind.cmp(&b.kind))
        },
    },
    // 24 — Equip - Cart (`:658-676`)
    Layer {
        name: "Equip - Cart",
        filter: |s| {
            let d = def(s.kind);
            d.mount_type != -1 && MOUNT_CART.get(d.mount_type as u16)
        },
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            db.rare.cmp(&da.rare).then(a.kind.cmp(&b.kind))
        },
    },
    // 25 — Equip - Light Pet (`:677-695`)
    Layer {
        name: "Equip - Light Pet",
        filter: |s| {
            let d = def(s.kind);
            d.buff_type > 0 && buffs::is_light_pet(d.buff_type)
        },
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            db.rare.cmp(&da.rare).then(a.kind.cmp(&b.kind))
        },
    },
    // 26 — Equip - Vanity Pet (`:696-714`)
    Layer {
        name: "Equip - Vanity Pet",
        filter: |s| {
            let d = def(s.kind);
            d.buff_type > 0 && buffs::is_vanity_pet(d.buff_type)
        },
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            db.rare.cmp(&da.rare).then(a.kind.cmp(&b.kind))
        },
    },
    // 27 — Potions - Dyes (`:837-863`)
    Layer {
        name: "Potions - Dyes",
        filter: |s| {
            let d = def(s.kind);
            d.dye > 0
                || ((s.kind as usize) < ITEM_COUNT
                    && SORTING_PRIORITY_POTIONS_DYE_MATERIAL.get(s.kind) > -1)
        },
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            db.rare
                .cmp(&da.rare)
                .then(db.dye.cmp(&da.dye))
                .then(compare_with_priority_set(
                    &SORTING_PRIORITY_POTIONS_DYE_MATERIAL,
                    a.kind,
                    b.kind,
                ))
                .then(b.count.cmp(&a.count))
        },
    },
    // 28 — Potions - Hair Dyes (`:864-886`)
    Layer {
        name: "Potions - Hair Dyes",
        filter: |s| def(s.kind).hair_dye >= 0,
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            db.rare
                .cmp(&da.rare)
                .then(db.hair_dye.cmp(&da.hair_dye))
                .then(b.count.cmp(&a.count))
        },
    },
    // 29 — Potions - Life (`:715-733`)
    Layer {
        name: "Potions - Life",
        filter: |s| {
            let d = def(s.kind);
            d.consumable && d.heal_life > 0 && d.heal_mana < 1 && s.kind != 5
        },
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            db.heal_life.cmp(&da.heal_life).then(b.count.cmp(&a.count))
        },
    },
    // 30 — Potions - Just The Mushroom (`:734-744`)
    Layer {
        name: "Potions - Just The Mushroom",
        filter: |s| s.kind == 5,
        compare: |a, b| b.count.cmp(&a.count),
    },
    // 31 — Potions - Mana (`:745-763`)
    Layer {
        name: "Potions - Mana",
        filter: |s| {
            let d = def(s.kind);
            d.consumable && d.heal_life < 1 && d.heal_mana > 0
        },
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            db.heal_mana.cmp(&da.heal_mana).then(b.count.cmp(&a.count))
        },
    },
    // 32 — Potions - Elixirs (`:764-782`)
    Layer {
        name: "Potions - Elixirs",
        filter: |s| {
            let d = def(s.kind);
            d.consumable && d.heal_life > 0 && d.heal_mana > 0
        },
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            db.heal_life.cmp(&da.heal_life).then(b.count.cmp(&a.count))
        },
    },
    // 33 — Potions - Buffs (`:783-809`)
    Layer {
        name: "Potions - Buffs",
        filter: |s| {
            let d = def(s.kind);
            (d.consumable
                && d.buff_type > 0
                && (s.kind as usize) < ITEM_COUNT
                && !IS_FOOD.get(s.kind))
                || SORTING_PRIORITY_POTIONS_BUFFS.get(s.kind) > -1
        },
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            compare_with_priority_set(&SORTING_PRIORITY_POTIONS_BUFFS, a.kind, b.kind)
                .then(db.rare.cmp(&da.rare))
                .then(a.kind.cmp(&b.kind))
                .then(b.count.cmp(&a.count))
        },
    },
    // 34 — Potions - Food (`:810-836`)
    Layer {
        name: "Potions - Food",
        filter: |s| {
            let d = def(s.kind);
            d.consumable && d.buff_type > 0 && (s.kind as usize) < ITEM_COUNT && IS_FOOD.get(s.kind)
        },
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            let mut ord = if db.buff_type >= 0 && db.buff_type < BUFF_COUNT {
                SORTING_PRIORITY_FOOD_BUFFS
                    .get(db.buff_type as u16)
                    .cmp(&SORTING_PRIORITY_FOOD_BUFFS.get(da.buff_type as u16))
            } else {
                Ordering::Equal
            };
            if ord == Ordering::Equal {
                ord = db.rare.cmp(&da.rare);
            }
            if ord == Ordering::Equal {
                ord = a.kind.cmp(&b.kind);
            }
            if ord == Ordering::Equal {
                ord = b.count.cmp(&a.count);
            }
            ord
        },
    },
    // 35 — Misc - Importants (`:887-905`)
    Layer {
        name: "Misc - Importants",
        filter: |s| s.kind > 0 && SORTING_PRIORITY_MISC_IMPORTANTS.get(s.kind) > -1,
        compare: |a, b| {
            SORTING_PRIORITY_MISC_IMPORTANTS
                .get(a.kind)
                .cmp(&SORTING_PRIORITY_MISC_IMPORTANTS.get(b.kind))
                .then(b.count.cmp(&a.count))
        },
    },
    // 36 — Misc - Painting (`:982-1008`)
    Layer {
        name: "Misc - Painting",
        filter: |s| {
            let d = def(s.kind);
            (s.kind > 0 && SORTING_PRIORITY_PAINTING.get(s.kind) > -1) || d.paint > 0
        },
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            SORTING_PRIORITY_PAINTING
                .get(b.kind)
                .cmp(&SORTING_PRIORITY_PAINTING.get(a.kind))
                .then(da.paint.cmp(&db.paint))
                .then(da.paint_coating.cmp(&db.paint_coating))
                .then(b.count.cmp(&a.count))
        },
    },
    // 37 — Misc - Wiring (`:906-932`)
    Layer {
        name: "Misc - Wiring",
        filter: |s| {
            let d = def(s.kind);
            (s.kind > 0 && SORTING_PRIORITY_WIRING.get(s.kind) > -1) || d.mech
        },
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            SORTING_PRIORITY_WIRING
                .get(b.kind)
                .cmp(&SORTING_PRIORITY_WIRING.get(a.kind))
                .then(db.rare.cmp(&da.rare))
                .then(b.kind.cmp(&a.kind))
                .then(b.count.cmp(&a.count))
        },
    },
    // 38 — Misc - Materials (`:933-951`)
    Layer {
        name: "Misc - Materials",
        filter: |s| s.kind > 0 && SORTING_PRIORITY_MATERIALS.get(s.kind) > -1,
        compare: |a, b| {
            SORTING_PRIORITY_MATERIALS
                .get(b.kind)
                .cmp(&SORTING_PRIORITY_MATERIALS.get(a.kind))
                .then(b.count.cmp(&a.count))
        },
    },
    // 39 — Misc - Just The Glowing Mushroom (`:952-962`)
    Layer {
        name: "Misc - Just The Glowing Mushroom",
        filter: |s| s.kind == 183,
        compare: |a, b| b.count.cmp(&a.count),
    },
    // 40 — Misc - Ropes (`:1009-1027`)
    Layer {
        name: "Misc - Ropes",
        filter: |s| s.kind > 0 && SORTING_PRIORITY_ROPES.get(s.kind) > -1,
        compare: |a, b| {
            SORTING_PRIORITY_ROPES
                .get(b.kind)
                .cmp(&SORTING_PRIORITY_ROPES.get(a.kind))
                .then(b.count.cmp(&a.count))
        },
    },
    // 41 — Misc - Herbs And Seeds (`:1028-1046`)
    Layer {
        name: "Misc - Herbs And Seeds",
        filter: |s| s.kind > 0 && SORTING_PRIORITY_MISC_HERBS_AND_SEEDS.get(s.kind) > -1,
        compare: |a, b| {
            SORTING_PRIORITY_MISC_HERBS_AND_SEEDS
                .get(b.kind)
                .cmp(&SORTING_PRIORITY_MISC_HERBS_AND_SEEDS.get(a.kind))
                .then(b.count.cmp(&a.count))
        },
    },
    // 42 — Misc - Acorns (`:1066-1084`)
    Layer {
        name: "Misc - Acorns",
        filter: |s| s.kind > 0 && SORTING_PRIORITY_MISC_ACORNS.get(s.kind) > -1,
        compare: |a, b| {
            SORTING_PRIORITY_MISC_ACORNS
                .get(b.kind)
                .cmp(&SORTING_PRIORITY_MISC_ACORNS.get(a.kind))
                .then(b.count.cmp(&a.count))
        },
    },
    // 43 — Misc - Gems (`:1047-1065`)
    Layer {
        name: "Misc - Gems",
        filter: |s| s.kind > 0 && SORTING_PRIORITY_MISC_GEMS.get(s.kind) > -1,
        compare: |a, b| {
            SORTING_PRIORITY_MISC_GEMS
                .get(b.kind)
                .cmp(&SORTING_PRIORITY_MISC_GEMS.get(a.kind))
                .then(b.count.cmp(&a.count))
        },
    },
    // 44 — Misc - Boss Bags (`:1085-1107`)
    Layer {
        name: "Misc - Boss Bags",
        filter: |s| s.kind > 0 && SORTING_PRIORITY_MISC_BOSS_BAGS.get(s.kind) > -1,
        compare: |a, b| {
            SORTING_PRIORITY_MISC_BOSS_BAGS
                .get(a.kind)
                .cmp(&SORTING_PRIORITY_MISC_BOSS_BAGS.get(b.kind))
                .then(def(b.kind).rare.cmp(&def(a.kind).rare))
                .then(b.count.cmp(&a.count))
        },
    },
    // 45 — Misc - Critters (`:1108-1126`)
    Layer {
        name: "Misc - Critters",
        filter: |s| def(s.kind).make_npc > 0,
        compare: |a, b| {
            def(a.kind)
                .make_npc
                .cmp(&def(b.kind).make_npc)
                .then(b.count.cmp(&a.count))
        },
    },
    // 46 — Misc - Extractinator (`:963-981`)
    Layer {
        name: "Misc - Extractinator",
        filter: |s| s.kind > 0 && SORTING_PRIORITY_EXTRACTIBLES.get(s.kind) > -1,
        compare: |a, b| {
            SORTING_PRIORITY_EXTRACTIBLES
                .get(b.kind)
                .cmp(&SORTING_PRIORITY_EXTRACTIBLES.get(a.kind))
                .then(b.count.cmp(&a.count))
        },
    },
    // 47 — Last - Materials (`:1127-1149`)
    Layer {
        name: "Last - Materials",
        filter: |s| {
            let d = def(s.kind);
            d.tile.is_none() && d.wall.map_or(true, |w| w < 1) && d.rare != -1
        },
        compare: |a, b| {
            let (da, db) = (def(a.kind), def(b.kind));
            db.rare
                .cmp(&da.rare)
                .then(db.value.cmp(&da.value))
                .then(b.count.cmp(&a.count))
        },
    },
    // 48 — Last - Tiles (Frame Important) (`:1150-1168`)
    Layer {
        name: "Last - Tiles (Frame Important)",
        filter: |s| {
            def(s.kind)
                .tile
                .is_some_and(|t| TILE_FRAME_IMPORTANT.get(t))
        },
        compare: |a, b| cmp_name(a, b).then(b.count.cmp(&a.count)),
    },
    // 49 — Last - Tiles (Common), Walls (`:1169-1187`)
    Layer {
        name: "Last - Tiles (Common), Walls",
        filter: |s| {
            let d = def(s.kind);
            d.wall.is_some_and(|w| w > 0) || d.tile.is_some()
        },
        compare: |a, b| cmp_name(a, b).then(b.count.cmp(&a.count)),
    },
    // 50 — Last - Not Trash (`:1188-1210`)
    Layer {
        name: "Last - Not Trash",
        filter: |s| def(s.kind).rare >= 0,
        compare: |a, b| {
            def(b.kind)
                .rare
                .cmp(&def(a.kind).rare)
                .then(cmp_name(a, b))
                .then(b.count.cmp(&a.count))
        },
    },
    // 51 — Last - Trash (`:1211-1229`)
    Layer {
        name: "Last - Trash",
        filter: |_s| true,
        compare: |a, b| {
            def(b.kind)
                .value
                .cmp(&def(a.kind).value)
                .then(b.count.cmp(&a.count))
        },
    },
];

/// `ItemSortingLayer.Validate` (`ItemSorting.cs:23-29`) precomputed: the index
/// (in [`SETUP_LAYERS`] order) of the first layer whose filter claims each item
/// type in the synthetic full-inventory sweep (`SetupWhiteLists`, `:1386-1408`).
/// Because each type is claimed by exactly one layer, `list.Contains(type)` is
/// exactly `layer_of[type] == this_layer_index`.
fn layer_of_table() -> &'static [u8; ITEM_COUNT] {
    static TABLE: OnceLock<[u8; ITEM_COUNT]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut layer_of = [u8::MAX; ITEM_COUNT];
        let mut claimed = [false; ITEM_COUNT];
        for (li, layer) in SETUP_LAYERS.iter().enumerate() {
            for t in 0..ITEM_COUNT {
                if claimed[t] {
                    continue;
                }
                let s = ItemSlot {
                    kind: t as u16,
                    name: "",
                    count: 1,
                    prefix: 0,
                };
                if (layer.filter)(&s) {
                    layer_of[t] = li as u8;
                    claimed[t] = true;
                }
            }
        }
        layer_of
    })
}

/// `ItemSorting.SortIndicesStable` (`ItemSorting.cs:143-154`): a stable sort
/// with the `x.CompareTo(y)` index tie-break — a total order, so the stability
/// is not load-bearing, but the tie-break is.
fn sort_indices_stable(
    idxs: &mut [usize],
    inv: &[ItemSlot],
    compare: fn(&ItemSlot, &ItemSlot) -> Ordering,
) {
    idxs.sort_by(|&x, &y| compare(&inv[x], &inv[y]).then(x.cmp(&y)));
}

/// `ItemSorting.SetupSortingPriorities` (`ItemSorting.cs:1437-1490`): the
/// damage-ranked weapon order (`AddSortingPrioritiesBasedOnPlayerDamage`,
/// `:1412-1435`) followed by the remaining [`SETUP_LAYERS`] (from index 4).
/// Returns `(layer, setup_index)` pairs.
fn sort_order(melee: f32, ranged: f32, magic: f32, minion: f32) -> Vec<(&'static Layer, usize)> {
    let mut weapons: [(f32, usize); 4] = [(melee, 0), (ranged, 1), (magic, 2), (minion, 3)];
    // `Descending` (`:1427-1435`): multiplier desc, then index asc.
    weapons.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(Ordering::Equal)
            .then(a.1.cmp(&b.1))
    });
    let mut out: Vec<(&'static Layer, usize)> = weapons
        .iter()
        .map(|&(_, li)| (&SETUP_LAYERS[li], li))
        .collect();
    for li in 4..SETUP_LAYERS.len() {
        out.push((&SETUP_LAYERS[li], li));
    }
    out
}

/// One layer pass (`SortingMethod`, `ItemSorting.cs:179` etc.): filter
/// `items_to_sort`, `Validate` against the whitelist, remove the claim from
/// `items_to_sort`, stable-sort, and return the claimed indexes.
fn run_layer(
    layer: &Layer,
    setup_index: usize,
    inv: &[ItemSlot],
    items_to_sort: &mut Vec<usize>,
) -> Vec<usize> {
    let layer_of = layer_of_table();
    let mut idxs: Vec<usize> = items_to_sort
        .iter()
        .copied()
        .filter(|&i| (layer.filter)(&inv[i]))
        .filter(|&i| layer_of[inv[i].kind as usize] == setup_index as u8)
        .collect();
    items_to_sort.retain(|i| !idxs.contains(i));
    sort_indices_stable(&mut idxs, inv, layer.compare);
    idxs
}

/// `ItemSorting.Sort` (`ItemSorting.cs:1492-1597`): the whole engine — stack
/// merge, then every layer in `SetupSortingPriorities` order, then the items are
/// written back into the lowest free slots in layer order. `ignore` slots are
/// skipped (`params int[] ignoreSlots`, `:1502`/`:1554`). `with_feedback` is
/// accepted for parity but ignored — it only drives `ItemSlot.SetGlow`
/// (`:1584-1587`), which the port has no equivalent for; the resulting order is
/// identical (see the module doc).
pub fn sort(inv: &mut [ItemSlot], ignore: &[usize], with_feedback: bool, mult: [f32; 4]) {
    let _ = with_feedback;
    let layers = sort_order(mult[0], mult[1], mult[2], mult[3]);

    // `_sort_itemsToSort`: non-ignored slots holding a real, non-favorited item
    // (`:1500-1510`). Chest slots are never favorited.
    let mut items_to_sort: Vec<usize> = (0..inv.len())
        .filter(|&i| !ignore.contains(&i))
        .filter(|&i| !is_empty(&inv[i]))
        .collect();

    // The stack-merge pass (`:1511-1550`): pull from later stacks into earlier
    // ones up to `maxStack`; an emptied source slot becomes air and leaves the
    // list (the C# `j--/k--` restarts the same target slot).
    let mut j = 0usize;
    while j < items_to_sort.len() {
        let slot_j = items_to_sort[j];
        let max_j = def(inv[slot_j].kind).max_stack;
        if inv[slot_j].count >= max_j {
            j += 1;
            continue;
        }
        let mut room = max_j - inv[slot_j].count;
        let mut k = j;
        let mut restart = false;
        while k < items_to_sort.len() {
            if k == j {
                k += 1;
                continue;
            }
            let slot_k = items_to_sort[k];
            let max_k = def(inv[slot_k].kind).max_stack;
            if can_stack(&inv[slot_j], &inv[slot_k]) && inv[slot_k].count != max_k {
                let mut take = inv[slot_k].count.min(room);
                if room < take {
                    take = room;
                }
                inv[slot_j].count += take;
                inv[slot_k].count -= take;
                room -= take;
                if inv[slot_k].count == 0 {
                    inv[slot_k] = EMPTY_SLOT;
                    items_to_sort.remove(k);
                    // C# `j--; k--; break;` then the outer `j++` re-reads the
                    // same target slot (now possibly grown).
                    restart = true;
                    break;
                }
                if room == 0 {
                    break;
                }
            }
            k += 1;
        }
        if !restart {
            j += 1;
        }
    }

    // `_sort_availableSortingSlots` (`:1551-1563`): the sorted item slots plus
    // the empty non-ignored ones, ascending.
    let mut available: Vec<usize> = items_to_sort.clone();
    for l in 0..inv.len() {
        if !ignore.contains(&l) && !available.contains(&l) && is_empty(&inv[l]) {
            available.push(l);
        }
    }
    available.sort_unstable();

    // Run every layer; each returns the indexes it sorted (`:1564-1572`).
    let mut sorted_indexes: Vec<usize> = Vec::new();
    for (layer, li) in &layers {
        let claimed = run_layer(layer, *li, inv, &mut items_to_sort);
        sorted_indexes.extend(claimed);
    }
    // Anything left (the C# appends it after the layers, `:1573`); with
    // `Last - Trash` matching everything this is empty.
    sorted_indexes.extend(items_to_sort.iter().copied());

    // Snapshot the sorted items, clear their slots (`:1574-1578`), then write
    // them back into the lowest available slots in layer order (`:1581-1596`).
    let cache: Vec<ItemSlot> = sorted_indexes.iter().map(|&i| inv[i]).collect();
    for &i in &sorted_indexes {
        inv[i] = EMPTY_SLOT;
    }
    for (m, item) in cache.iter().enumerate() {
        if m >= available.len() {
            break;
        }
        inv[available[m]] = *item;
    }
}

/// `ItemSorting.SortChest` (`ItemSorting.cs:1629-1653`) — the chest Sort button.
/// Sorts `slots` (`chest.item`, `chest.maxItems` long) with no ignored slots and
/// feedback on; the net-sync half is a no-op outside multiplayer
/// (`Main.netMode != 1`, `:1669`).
pub fn sort_chest(slots: &mut [ItemSlot], mult: [f32; 4]) {
    sort(slots, &[], true, mult);
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

    /// The C# `SetupWhiteLists` assert (`ItemSorting.cs:1409`): no item type is
    /// claimed by `Last - Not Trash` — every non-trash type has an earlier home.
    #[test]
    fn setup_whitelists_covers_every_item() {
        let layer_of = layer_of_table();
        let last_not_trash = SETUP_LAYERS.len() - 2; // 50
        for (t, &li) in layer_of.iter().enumerate() {
            assert_ne!(
                li as usize,
                last_not_trash,
                "type {t} ({}) fell through to Last - Not Trash",
                crate::itemdata::ITEMS[t].name
            );
        }
    }

    #[test]
    fn compare_with_priority_set_orders_listed_before_unlisted() {
        let set = &SORTING_PRIORITY_MISC_GEMS; // default -1, ids 999/182/...
        assert_eq!(compare_with_priority_set(set, 999, 3), Ordering::Less);
        assert_eq!(compare_with_priority_set(set, 3, 999), Ordering::Greater);
        assert_eq!(compare_with_priority_set(set, 3, 4), Ordering::Equal);
        assert_eq!(compare_with_priority_set(set, 182, 999), Ordering::Greater);
    }

    #[test]
    fn merges_partial_stacks_of_the_same_item() {
        // Two Dirt Blocks (id 2, max stack 9999) merge into one.
        let mut inv = [EMPTY_SLOT; 4];
        inv[0] = slot(2, 100);
        inv[1] = slot(2, 50);
        sort(&mut inv, &[], false, [1.0; 4]);
        assert_eq!(inv[0].kind, 2);
        assert_eq!(inv[0].count, 150);
        assert!(is_empty(&inv[1]), "the drained stack became air");
    }

    #[test]
    fn orders_layers_then_compacts_into_low_slots() {
        // A sword (id 24, melee damage) and a dirt block (id 2). The weapon layer
        // claims the sword first, so it lands in slot 0 and the block in slot 1.
        let mut inv = [EMPTY_SLOT; 4];
        inv[0] = slot(2, 10); // Dirt Block — Last - Tiles (Common)
        inv[1] = slot(24, 1); // Wood Sword? (a melee weapon) — Weapons - Melee
        sort(&mut inv, &[], false, [1.0; 4]);
        assert_eq!(inv[0].kind, 24, "the weapon sorts before the block");
        assert_eq!(inv[1].kind, 2);
        assert!(is_empty(&inv[2]));
    }
}
