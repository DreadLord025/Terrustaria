//! The crafting panel's recipe filters — the `EntryFilterer<Item, IItemEntryFilter>`
//! (`DataStructures/EntryFilterer.cs`) `NewCraftingUI` populates with the eleven
//! `ItemFilters` subclasses (`GameContent/Creative/ItemFilters.cs`), and the
//! filter strip's click (`UICreativeItemsInfiniteFilteringOptions.cs:80-100`).
//!
//! Construction order is `NewCraftingUI.cs:282-298` — the ten real filters, then
//! the `MiscFallback` that wraps those ten; `:630-641`'s `FitsFilter` gates every
//! available recipe through the board, and `:604-607` shows only the survivors in
//! the grid.
//!
//! Deliberate deviations (docs/ANALYSIS.md): `Item.tileWand` and the `BySearch`
//! tooltip scan are not carried (see the arms below), and the search box keeps no
//! text because the port has no keyboard text field.

use crate::itemdata::ItemDef;
use crate::sortdata::{PROJ_HOOK, TILE_FRAME_IMPORTANT};

/// The eleven filter slots in `NewCraftingUI`'s construction order
/// (`NewCraftingUI.cs:282-298`); index `i` is the `i`-th tab of the strip
/// (`UICreativeItemsInfiniteFilteringOptions.cs:52-57`), which is also the
/// `ui::crafting::FILTER_ICON_K` order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterKind {
    /// `ItemFilters.Weapon` (`ItemFilters.cs:191-212`).
    Weapon,
    /// `ItemFilters.Armor` (`:226-247`).
    Armor,
    /// `ItemFilters.Vanity` (`:249-270`).
    Vanity,
    /// `ItemFilters.BuildingBlock` (`:71-104`).
    BuildingBlock,
    /// `ItemFilters.Furniture` (`:106-132`).
    Furniture,
    /// `ItemFilters.Accessories` (`:304-325`).
    Accessories,
    /// `ItemFilters.MiscAccessories` (`:327-348`).
    MiscAccessories,
    /// `ItemFilters.Consumables` (`:350-381`).
    Consumables,
    /// `ItemFilters.Tools` (`:134-189`).
    Tools,
    /// `ItemFilters.Materials` (`:443-464`).
    Materials,
    /// `ItemFilters.MiscFallback` (`:406-441`): everything the ten above refuse.
    MiscFallback,
}

/// `NewCraftingUI.cs:282-298` — `list2` (the ten real filters) plus the
/// `MiscFallback` wrapping them.
pub const FILTERS: [FilterKind; 11] = [
    FilterKind::Weapon,
    FilterKind::Armor,
    FilterKind::Vanity,
    FilterKind::BuildingBlock,
    FilterKind::Furniture,
    FilterKind::Accessories,
    FilterKind::MiscAccessories,
    FilterKind::Consumables,
    FilterKind::Tools,
    FilterKind::Materials,
    FilterKind::MiscFallback,
];

/// The ten filters `MiscFallback` checks against (`NewCraftingUI.cs:295-297`
/// passes `list`, the pre-fallback list).
const FALLBACK_SOURCE: [FilterKind; 10] = [
    FilterKind::Weapon,
    FilterKind::Armor,
    FilterKind::Vanity,
    FilterKind::BuildingBlock,
    FilterKind::Furniture,
    FilterKind::Accessories,
    FilterKind::MiscAccessories,
    FilterKind::Consumables,
    FilterKind::Tools,
    FilterKind::Materials,
];

impl FilterKind {
    /// `IItemEntryFilter.GetDisplayNameKey` — the `CreativePowers.Tab*` row the
    /// hover prints (`UICreativeItemsInfiniteFilteringOptions.cs:125-132`).
    pub fn display_name_key(self) -> &'static str {
        match self {
            // `ItemFilters.cs:200`.
            FilterKind::Weapon => "CreativePowers.TabWeapons",
            // `:235`.
            FilterKind::Armor => "CreativePowers.TabArmor",
            // `:258`.
            FilterKind::Vanity => "CreativePowers.TabVanity",
            // `:92`.
            FilterKind::BuildingBlock => "CreativePowers.TabBlocks",
            // `:120`.
            FilterKind::Furniture => "CreativePowers.TabFurniture",
            // `:313`.
            FilterKind::Accessories => "CreativePowers.TabAccessories",
            // `:336`.
            FilterKind::MiscAccessories => "CreativePowers.TabAccessoriesMisc",
            // `:369`.
            FilterKind::Consumables => "CreativePowers.TabConsumables",
            // `:177`.
            FilterKind::Tools => "CreativePowers.TabTools",
            // `:452`.
            FilterKind::Materials => "CreativePowers.TabMaterials",
            // `:429` (the same key `GameplayItems` returns at `:392`).
            FilterKind::MiscFallback => "CreativePowers.TabMisc",
        }
    }
}

/// `ItemFilters.Tools._itemIdsThatAreAccepted` (`ItemFilters.cs:136-144`).
const TOOLS_EXTRA_IDS: [u16; 58] = [
    213, 5295, 509, 850, 851, 3612, 3625, 3611, 510, 849, 3620, 1071, 1543, 1072, 1544, 1100, 1545,
    50, 3199, 3124, 5358, 5359, 5360, 5361, 5437, 1326, 5335, 3384, 4263, 4819, 4262, 946, 4707,
    205, 206, 207, 1128, 3031, 4820, 5302, 5364, 4460, 4608, 4872, 3032, 5303, 5304, 1991, 4821,
    3183, 779, 5134, 1299, 4711, 4049, 114, 5667, 6174,
];

/// `ItemFilters.AArmor.IsAnArmorThatMatchesSocialState` (`ItemFilters.cs:216-223`).
fn is_armor_matching_social(d: &ItemDef, should_be_social: bool) -> bool {
    if d.body_slot != -1 || d.head_slot != -1 || d.leg_slot != -1 {
        return d.vanity == should_be_social;
    }
    false
}

/// `ItemFilters.AAccessories.IsMiscEquipment` (`ItemFilters.cs:294-301`): a mount,
/// a light pet, a vanity pet or a grapple hook — the `Player.miscEquips` rows.
fn is_misc_equipment(d: &ItemDef) -> bool {
    if d.mount_type == -1
        && (d.buff_type <= 0 || !crate::buffs::is_light_pet(d.buff_type))
        && (d.buff_type <= 0 || !crate::buffs::is_vanity_pet(d.buff_type))
    {
        return d.projectile.is_some_and(|p| PROJ_HOOK.get(p));
    }
    true
}

/// `ItemFilters.AAccessories.IsAnAccessoryOfType` (`ItemFilters.cs:280-292`).
fn is_accessory_of_type(d: &ItemDef, misc: bool) -> bool {
    let m = is_misc_equipment(d);
    if m && misc {
        return true;
    }
    !m && !misc && d.accessory
}

/// One `IItemEntryFilter.FitsFilter`, arm for arm. `tileFrameImportant` is
/// `Main.tileFrameImportant` (`ItemFilters.cs:87`, `:115`).
fn fits_filter(kind: FilterKind, d: &ItemDef) -> bool {
    let frame_important = |t: u16| TILE_FRAME_IMPORTANT.get(t);
    match kind {
        // `:193-196`.
        FilterKind::Weapon => d.damage > 0,

        // `:230` / `:253`.
        FilterKind::Armor => is_armor_matching_social(d, false),
        FilterKind::Vanity => is_armor_matching_social(d, true),

        // `:73-88`: `createWall != -1 || tileWand != -1 || (createTile != -1 &&
        // !tileFrameImportant[createTile])`. `tileWand` is not carried by
        // `ItemDef`, so a wand whose `createTile` is frame-important falls
        // through to Misc here (docs/ANALYSIS.md).
        FilterKind::BuildingBlock => {
            if d.wall.is_some() {
                return true;
            }
            match d.tile {
                None => false,
                Some(t) => !frame_important(t),
            }
        }

        // `:108-116`.
        FilterKind::Furniture => match d.tile {
            None => false,
            Some(t) => frame_important(t),
        },

        // `:308` / `:331`.
        FilterKind::Accessories => is_accessory_of_type(d, false),
        FilterKind::MiscAccessories => is_accessory_of_type(d, true),

        // `:352-365`: the two hard-listed ids, else a consumable that places
        // nothing (`createTile`/`createWall`/`tileWand` all -1).
        FilterKind::Consumables => {
            if d.id == 267 || d.id == 1307 {
                return true;
            }
            d.consumable && d.tile.is_none() && d.wall.is_none()
        }

        // `:146-173`: the four tool stats, the tile wand (not carried), then
        // the hand-listed ids.
        FilterKind::Tools => {
            d.pick > 0
                || d.axe > 0
                || d.hammer > 0
                || d.fishing_pole > 0
                || TOOLS_EXTRA_IDS.contains(&d.id)
        }

        // `:445-448`.
        FilterKind::Materials => d.material,

        // `:417-425`: `!otherFiltersToCheckAgainst.Any(f => f.FitsFilter(entry))`.
        // The C# memoises it per item type; the predicate is already a pure
        // function of the type, so no cache is needed.
        FilterKind::MiscFallback => !FALLBACK_SOURCE.iter().any(|&f| fits_filter(f, d)),
    }
}

/// `EntryFilterer<Item, IItemEntryFilter>`'s active-filter set
/// (`EntryFilterer.cs:6-99`), holding the strip index rather than filter objects.
///
/// The C# keeps a `List<U> ActiveFilters`; the board holds one index because
/// `singleFilterButtonClick` clears the list before toggling
/// (`UICreativeItemsInfiniteFilteringOptions.cs:89-93`) — at most one filter is
/// ever active.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FilterBoard {
    /// `EntryFilterer.ActiveFilters` as a strip index; `None` = the empty list,
    /// which passes everything (`EntryFilterer.cs:43-45`).
    pub active: Option<usize>,
}

impl FilterBoard {
    /// `singleFilterButtonClick` (`UICreativeItemsInfiniteFilteringOptions.cs:80-100`):
    /// clicking an inactive tab clears the active set first, so the strip is
    /// exclusive; clicking the active one only toggles it back off.
    pub fn click(&mut self, index: usize) {
        if index >= FILTERS.len() {
            return;
        }
        if self.active != Some(index) {
            self.active = None; // `:91` `ActiveFilters.Clear()`
        }
        self.active = if self.active == Some(index) {
            None // `ToggleFilter`'s remove arm (`EntryFilterer.cs:57-67`)
        } else {
            Some(index)
        };
    }

    /// `EntryFilterer.IsFilterActive` (`EntryFilterer.cs:70-78`) as
    /// `UpdateVisuals` reads it (`UICreativeItemsInfiniteFilteringOptions.cs:104`).
    pub fn is_active(&self, index: usize) -> bool {
        self.active == Some(index)
    }

    /// The `active` flags `ui::crafting::draw_filter_strip` wants.
    pub fn active_flags(&self) -> [bool; 11] {
        let mut flags = [false; 11];
        if let Some(i) = self.active {
            flags[i] = true;
        }
        flags
    }

    /// `EntryFilterer.FitsFilter` (`EntryFilterer.cs:30-55`) for one item type.
    /// The always-active list is empty in the crafting UI.
    pub fn fits(&self, kind: u16) -> bool {
        match self.active {
            None => true,
            Some(i) => crate::registry::item(kind).is_some_and(|d| fits_filter(FILTERS[i], d)),
        }
    }

    /// `NewCraftingUI.UpdateContents`'s grid pass (`NewCraftingUI.cs:596-611`):
    /// the available recipes whose product passes `FitsFilter`. Only the *shown*
    /// list narrows — `Main.focusRecipe` stays an `availableRecipe` index
    /// (`:610`, `:622`), which is what the port's `focus_recipe` already holds.
    pub fn shown(&self, available: Vec<usize>) -> Vec<usize> {
        if self.active.is_none() {
            return available;
        }
        available
            .into_iter()
            .filter(|&r| self.fits(crate::craft::RECIPES[r].create.0))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def(kind: u16) -> &'static ItemDef {
        crate::registry::item(kind).expect("the test item exists")
    }

    /// The construction order is `NewCraftingUI.cs:282-298`, index for index with
    /// the strip's icon table (`ui::crafting::FILTER_ICON_K`).
    #[test]
    fn the_filter_order_matches_the_c_sharp_construction() {
        assert_eq!(FILTERS.len(), 11);
        assert_eq!(FILTERS[0], FilterKind::Weapon);
        assert_eq!(FILTERS[1], FilterKind::Armor);
        assert_eq!(FILTERS[2], FilterKind::Vanity);
        assert_eq!(FILTERS[3], FilterKind::BuildingBlock);
        assert_eq!(FILTERS[4], FilterKind::Furniture);
        assert_eq!(FILTERS[5], FilterKind::Accessories);
        assert_eq!(FILTERS[6], FilterKind::MiscAccessories);
        assert_eq!(FILTERS[7], FilterKind::Consumables);
        assert_eq!(FILTERS[8], FilterKind::Tools);
        assert_eq!(FILTERS[9], FilterKind::Materials);
        assert_eq!(FILTERS[10], FilterKind::MiscFallback);
        assert_eq!(TOOLS_EXTRA_IDS.len(), 58, "`ItemFilters.cs:136-144`");
        assert_eq!(
            FilterKind::Weapon.display_name_key(),
            "CreativePowers.TabWeapons"
        );
        assert_eq!(
            FilterKind::MiscFallback.display_name_key(),
            "CreativePowers.TabMisc"
        );
    }

    /// `Weapon` is `damage > 0` (`ItemFilters.cs:195`).
    #[test]
    fn weapon_is_any_damage() {
        assert!(fits_filter(FilterKind::Weapon, def(4)), "Iron Broadsword");
        assert!(!fits_filter(FilterKind::Weapon, def(2)), "Dirt Block");
    }

    /// `Armor`/`Vanity` split on `vanity` (`ItemFilters.cs:216-223`).
    #[test]
    fn armor_and_vanity_split_on_the_social_flag() {
        assert!(fits_filter(FilterKind::Armor, def(89)), "Copper Helmet");
        assert!(!fits_filter(FilterKind::Vanity, def(89)));
        // Boss Mask Betsy: head_slot 207 with `vanity`.
        assert!(fits_filter(FilterKind::Vanity, def(3863)));
        assert!(!fits_filter(FilterKind::Armor, def(3863)));
        assert!(!fits_filter(FilterKind::Armor, def(4)), "not armour at all");
    }

    /// `BuildingBlock` vs `Furniture` split on `tileFrameImportant`
    /// (`ItemFilters.cs:87`/`:115`).
    #[test]
    fn blocks_and_furniture_split_on_frame_important() {
        assert!(fits_filter(FilterKind::BuildingBlock, def(2)), "Dirt Block");
        assert!(!fits_filter(FilterKind::Furniture, def(2)));
        // The Work Bench places tile 18, frame-important.
        assert!(fits_filter(FilterKind::Furniture, def(36)));
        assert!(!fits_filter(FilterKind::BuildingBlock, def(36)));
        // A wall-only item is still a building block.
        assert!(fits_filter(FilterKind::BuildingBlock, def(30)), "Dirt Wall");
    }

    /// `Consumables` excludes anything that places a tile/wall and hard-lists
    /// 267/1307 (`ItemFilters.cs:352-365`).
    #[test]
    fn consumables_exclude_placeables() {
        assert!(
            fits_filter(FilterKind::Consumables, def(267)),
            "Voodoo Doll"
        );
        assert!(fits_filter(FilterKind::Consumables, def(1307)));
        assert!(
            !fits_filter(FilterKind::Consumables, def(2)),
            "places tile 0"
        );
        assert!(
            fits_filter(FilterKind::Consumables, def(28)),
            "Healing Potion"
        );
    }

    /// `Tools` reads the four tool stats plus the hand-listed ids
    /// (`ItemFilters.cs:146-173`).
    #[test]
    fn tools_read_the_stats_and_the_extra_ids() {
        assert!(
            fits_filter(FilterKind::Tools, def(1)),
            "Iron Pickaxe, pick 40"
        );
        assert!(fits_filter(FilterKind::Tools, def(213)), "a listed id");
        assert!(!fits_filter(FilterKind::Tools, def(4)), "Iron Broadsword");
    }

    /// `Materials` is `entry.material` (`ItemFilters.cs:447`).
    #[test]
    fn materials_is_the_flag() {
        assert!(fits_filter(FilterKind::Materials, def(2)), "Dirt Block");
        assert!(
            !fits_filter(FilterKind::Materials, def(4)),
            "Iron Broadsword"
        );
    }

    /// `MiscFallback` is the negation of the other ten (`ItemFilters.cs:417-425`).
    #[test]
    fn misc_fallback_claims_what_the_others_refuse() {
        for kind in [1u16, 2, 4, 28, 30, 36, 267, 3863] {
            let d = def(kind);
            let claimed = FALLBACK_SOURCE.iter().any(|&f| fits_filter(f, d));
            assert_eq!(
                fits_filter(FilterKind::MiscFallback, d),
                !claimed,
                "item {kind} ({}) must be the exact complement",
                d.name
            );
        }
    }

    /// `singleFilterButtonClick` clears before toggling, and re-clicking the
    /// active tab turns it off (`UICreativeItemsInfiniteFilteringOptions.cs:89-93`).
    #[test]
    fn the_filter_strip_is_exclusive() {
        let mut b = FilterBoard::default();
        assert!(b.fits(4), "no filter = everything passes");
        b.click(0);
        assert!(b.is_active(0));
        assert!(b.fits(4), "Weapon takes the sword");
        assert!(!b.fits(2), "…and refuses the Dirt Block");
        b.click(1);
        assert!(!b.is_active(0), "a new tab clears the previous one");
        assert!(b.is_active(1));
        assert!(b.fits(89), "Armor takes the Copper Helmet");
        assert!(!b.fits(4));
        b.click(1);
        assert_eq!(b.active, None, "re-clicking the active tab clears it");
        assert!(b.fits(4));
        assert_eq!(b.active_flags(), [false; 11]);
        // An out-of-range index is a no-op rather than a panic.
        b.click(11);
        assert_eq!(b.active, None);
    }
}
