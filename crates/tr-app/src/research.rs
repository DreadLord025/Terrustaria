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
}

impl ResearchTracker {
    /// `ItemsSacrificedUnlocksTracker.RegisterItemSacrifice`'s count
    /// accumulation (the stack merges into the running total).
    pub fn register_sacrifice(&mut self, item_id: u16, stack: u32) {
        for (eid, count) in self.counts.iter_mut() {
            if *eid == item_id {
                *count += stack;
                return;
            }
        }
        self.counts.push((item_id, stack));
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

    pub fn reset(&mut self) {
        self.counts.clear();
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
    }
}
