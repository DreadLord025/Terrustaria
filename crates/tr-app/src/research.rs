//! The Journey research — `CreativeItemSacrificesCatalog.cs` +
//! `ItemsSacrificedUnlocksTracker.cs`: the sacrifice caps (the generated
//! `creative_sacrifices::SACRIFICES` table, the name → the count) and the
//! per-item sacrificed counts (the world-persistent state — the save
//! rides M6-SAVE's world sections).

use crate::creative_sacrifices::SACRIFICES;

/// `TryGetSacrificeCountCapToUnlockInfiniteItems` (`:95-102`): the item
/// id's needed count via the ItemID const name.
pub fn sacrifice_cap(item_id: u16) -> Option<u32> {
    let name = crate::itemdata::NAMES.get(item_id as usize)?;
    let idx = SACRIFICES.binary_search_by(|(n, _)| (*n).cmp(name)).ok()?;
    Some(SACRIFICES[idx].1)
}

/// `ItemsSacrificedUnlocksTracker`: the per-item sacrificed counts.
#[derive(Clone, Debug, Default)]
pub struct ResearchTracker {
    counts: Vec<(u16, u32)>,
    /// `_newlyUnlocked` (`ItemsSacrificedUnlocksTracker.cs:19`): the kinds
    /// whose research just completed, until the duplication grid's hover
    /// clears them (`UICreativeItemGrid.cs:28`). The teammate-credit half
    /// (`_unlockedByTeammate`/`AnyNewUnlocksFromTeammates`) is single-player
    /// inert and stays unported.
    newly: Vec<u16>,
}

impl ResearchTracker {
    /// `ItemsSacrificedUnlocksTracker.RegisterItemSacrifice`
    /// (`ItemsSacrificedUnlocksTracker.cs:116-144`): the count clamps at the
    /// catalog cap (`Math.Min(value3 + amount, amountNeeded)`, `:131`), a
    /// capless kind or an already-full row registers nothing (`:122-130`),
    /// and reaching the cap marks the kind newly-unlocked (`:135-137`).
    pub fn register_sacrifice(&mut self, item_id: u16, stack: u32) {
        let Some(cap) = sacrifice_cap(item_id) else {
            return; // `:122-125` — no catalog row, no registration
        };
        let have = self.sacrificed(item_id);
        if have >= cap {
            return; // `:127-130`
        }
        let total = (have + stack).min(cap); // `:131`
        let mut found = false;
        for (eid, count) in self.counts.iter_mut() {
            if *eid == item_id {
                *count = total;
                found = true;
                break;
            }
        }
        if !found {
            self.counts.push((item_id, total));
        }
        if total >= cap && !self.newly.contains(&item_id) {
            self.newly.push(item_id); // `:135-137`
        }
    }

    /// `IsNewlyResearched` (`ItemsSacrificedUnlocksTracker.cs:101-104`).
    pub fn is_newly_researched(&self, item_id: u16) -> bool {
        self.newly.contains(&item_id)
    }

    /// `ClearNewlyResearchedStatus` (`:106-109`) — the duplication grid's
    /// hover calls it (`UICreativeItemGrid.cs:28`).
    pub fn clear_newly_researched(&mut self, item_id: u16) {
        self.newly.retain(|&id| id != item_id);
    }

    pub fn sacrificed(&self, item_id: u16) -> u32 {
        for (eid, count) in &self.counts {
            if *eid == item_id {
                return *count;
            }
        }
        0
    }

    /// The full research: the sacrificed total reaches the cap (the
    /// items without a cap row stay unresearchable, like the vanilla's
    /// missing dictionary entry).
    pub fn is_fully_researched(&self, item_id: u16) -> bool {
        match sacrifice_cap(item_id) {
            Some(cap) => self.sacrificed(item_id) >= cap,
            None => false,
        }
    }

    /// `ItemsSacrificedUnlocksTracker.TryGetSacrificeNumbers`
    /// (`ItemsSacrificedUnlocksTracker.cs:75-86`): `(have, needed)` when the
    /// catalog carries the item, `None` for the capless kinds (the UI prints
    /// "X" for them, `UICreativeInfiniteItemsDisplay.cs:426-429`).
    pub fn sacrifice_numbers(&self, item_id: u16) -> Option<(u32, u32)> {
        let cap = sacrifice_cap(item_id)?;
        Some((self.sacrificed(item_id), cap))
    }

    /// `ForEachItemWithResearchProgress` (`ItemsSacrificedUnlocksTracker.cs:47-55`):
    /// every item whose sacrificed count is positive — the duplication grid's
    /// roster (`UICreativeInfiniteItemsDisplay.UpdateContents:536-549`).
    pub fn items_with_progress(&self) -> impl Iterator<Item = u16> + '_ {
        self.counts
            .iter()
            .filter(|(_, count)| *count > 0)
            .map(|(id, _)| *id)
    }

    pub fn reset(&mut self) {
        self.counts.clear();
        self.newly.clear(); // `Reset` (`:208-216`)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_caps_read_through_the_names() {
        // The Torch (8): the grade L = 100 (the TSV's Torch row); the
        // Musket Ball 97: the grade from the TSV row.
        assert_eq!(sacrifice_cap(8), Some(100));
        assert_eq!(sacrifice_cap(97), Some(99));
        // The id past the table's range or the nameless row: none.
        assert_eq!(sacrifice_cap(6196), None);
    }

    #[test]
    fn the_tracker_accumulates_and_unlocks() {
        let mut t = ResearchTracker::default();
        assert!(!t.is_fully_researched(8), "the torch starts unresearched");
        t.register_sacrifice(8, 50);
        assert_eq!(t.sacrificed(8), 50);
        assert!(!t.is_fully_researched(8));
        t.register_sacrifice(8, 50);
        assert_eq!(t.sacrificed(8), 100, "the stacks merge");
        assert!(t.is_fully_researched(8), "the cap reached");
        // The reset clears.
        t.reset();
        assert_eq!(t.sacrificed(8), 0);
        // The capless item never researches.
        t.register_sacrifice(6196, 999);
        assert!(!t.is_fully_researched(6196));
        assert_eq!(t.sacrificed(6196), 0, "capless kinds register nothing (:122)");
    }

    /// `RegisterItemSacrifice` clamps at the cap (`:127-131`) and the
    /// newly-unlocked set arms at the completion, cleared by the grid hover
    /// (`:135-137`, `UICreativeItemGrid.cs:28`).
    #[test]
    fn the_counts_clamp_and_the_newly_set_arms_once() {
        let mut t = ResearchTracker::default();
        t.register_sacrifice(8, 500);
        assert_eq!(t.sacrificed(8), 100, "Math.Min(have + amount, cap) (:131)");
        assert!(t.is_newly_researched(8));
        t.register_sacrifice(8, 500); // the full row registers nothing (:127)
        assert_eq!(t.sacrificed(8), 100);
        t.clear_newly_researched(8);
        assert!(!t.is_newly_researched(8));
        // Partial progress never arms the set.
        t.register_sacrifice(97, 5);
        assert_eq!(t.sacrificed(97), 5);
        assert!(!t.is_newly_researched(97));
        assert_eq!(t.items_with_progress().count(), 2);
        assert_eq!(t.sacrifice_numbers(97), Some((5, 99)));
        assert_eq!(t.sacrifice_numbers(6196), None);
    }
}
