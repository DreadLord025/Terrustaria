//! The Journey research — `CreativeItemSacrificesCatalog.cs` +
//! `ItemsSacrificedUnlocksTracker.cs`: the sacrifice caps (the generated
//! `creative_sacrifices::SACRIFICES` table, the name → the count) and the
//! per-item sacrificed counts. The counts persist through the `.plr`
//! `creativeTracker` block (`Player.cs:54911`/`:55696-55699` →
//! `ItemsSacrificedUnlocksTracker.Save`/`Load`, `:157-196`): the
//! `(persistentId, count)` string pairs the W4-f44 serialization seam
//! carries (`player_file::PlayerFileWrite.research` /
//! `PlayerFile.research`, the same `Vec<(String, i32)>` shape). The
//! persistent id IS the ItemID const name (`ContentSamples.
//! ItemPersistentIdsByNetIds`, the generated `ItemName` table's key) —
//! the port's `itemdata::NAMES` row, the same string the sacrifice caps
//! table keys on.

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
    /// `LastEditId` (`ItemsSacrificedUnlocksTracker.cs:23`,
    /// `MarkContentsDirty` `:222-225`): bumps on every successful
    /// registration / reset — the `CheckResearchAchievement` version gate
    /// (`AchievementsHelper.cs:389-392`). Not persisted (the C# save carries
    /// only the count pairs).
    last_edit_id: u32,
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
        self.last_edit_id = self.last_edit_id.wrapping_add(1); // `MarkContentsDirty` (`:134`)
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
        self.last_edit_id = self.last_edit_id.wrapping_add(1); // `MarkContentsDirty` (`:215`)
    }

    /// `LastEditId` (`ItemsSacrificedUnlocksTracker.cs:23`): the edit version
    /// the research-achievement gate compares (`AchievementsHelper.cs:389`).
    pub fn last_edit_id(&self) -> u32 {
        self.last_edit_id
    }

    /// `CountFullyResearchedItems` (`ItemsSacrificedUnlocksTracker.cs:60-75`):
    /// `(fullyResearchedItems, allItems)` over the whole `ItemID.Count` walk —
    /// every catalog-capped kind counts toward `allItems` (the capless kinds
    /// answer `TryGetSacrificeNumbers` false), the ones at the cap toward the
    /// fully set. The port walks its `NAMES` id space (the same `ItemID.Count`
    /// 6196 rows); the `CreativeResearchItemPersistentIdOverride` alias remap
    /// is the documented unported table (the names map 1:1).
    pub fn count_fully_researched(&self) -> (u32, u32) {
        let mut full = 0u32;
        let mut all = 0u32;
        for id in 0..crate::itemdata::NAMES.len() as u16 {
            if let Some((have, cap)) = self.sacrifice_numbers(id) {
                all += 1;
                if have >= cap {
                    full += 1;
                }
            }
        }
        (full, all)
    }

    /// `ItemsSacrificedUnlocksTracker.Save` (`ItemsSacrificedUnlocksTracker.cs:
    /// 157-169`): the `(persistentId, count)` pairs the `.plr`
    /// `creativeTracker` block carries (`Player.cs:54911`). The persistent
    /// id is the ItemID const name (`ContentSamples.ItemPersistentIdsByNetIds`
    /// — the port's `itemdata::NAMES` row); only the positive counts ride
    /// (the C#'s `_sacrificeCountByItemPersistentId` holds every registered
    /// row, and `RegisterItemSacrifice` never writes a 0). The W4-f44 seam
    /// (`player_file::PlayerFileWrite.research`) owns the byte write: the
    /// literal `false` flag + the i32 count + the (string, i32) pairs.
    pub fn save_pairs(&self) -> Vec<(String, i32)> {
        self.counts
            .iter()
            .filter(|(_, count)| *count > 0)
            .filter_map(|(id, count)| {
                let name = crate::itemdata::NAMES.get(*id as usize)?;
                Some(((*name).to_string(), *count as i32))
            })
            .collect()
    }

    /// `ItemsSacrificedUnlocksTracker.Load` (`:171-196`): the
    /// `(persistentId, count)` pairs land on the net-id cache through the
    /// persistent-id → net-id map (`ContentSamples.ItemNetIdsByPersistentIds`
    /// — the port's `NAMES` reverse lookup). An unknown id keeps its string
    /// row in the C# (the `_sacrificeCountByItemPersistentId` dictionary)
    /// but never reaches the cache; the port's tracker is the cache, so an
    /// unknown name is dropped (the count has no item to attach to — the
    /// documented deviation, ANALYSIS). The `CreativeResearchItemPersistent
    /// IdOverride` remap (`:184-187`) is a generated alias table the port
    /// does not carry — the names map 1:1.
    pub fn load_pairs(&mut self, pairs: &[(String, i32)]) {
        for (name, count) in pairs {
            // The net-id reverse lookup: the NAMES row whose string matches.
            let Some(id) = crate::itemdata::NAMES
                .iter()
                .position(|n| *n == name.as_str())
            else {
                continue; // the unknown-id drop (the C# keeps the string row)
            };
            let count = (*count).max(0) as u32;
            let mut found = false;
            for (eid, c) in self.counts.iter_mut() {
                if *eid == id as u16 {
                    *c = count;
                    found = true;
                    break;
                }
            }
            if !found {
                self.counts.push((id as u16, count));
            }
        }
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

    /// `Save`/`Load` (`ItemsSacrificedUnlocksTracker.cs:157-196`): the
    /// `(persistentId, count)` pairs round-trip through the ItemID const
    /// names — the W4-f44 `.plr` seam's `Vec<(String, i32)>` shape.
    #[test]
    fn the_save_load_pairs_round_trip_through_the_names() {
        let mut t = ResearchTracker::default();
        t.register_sacrifice(8, 100); // Torch, fully
        t.register_sacrifice(2, 37); // DirtBlock, partial
        let pairs = t.save_pairs();
        assert_eq!(
            pairs,
            vec![("Torch".to_string(), 100), ("DirtBlock".to_string(), 37)],
            "the persistent ids are the ItemID const names"
        );
        // The load lands them on a fresh tracker.
        let mut back = ResearchTracker::default();
        back.load_pairs(&pairs);
        assert_eq!(back.sacrificed(8), 100);
        assert_eq!(back.sacrificed(2), 37);
        assert!(back.is_fully_researched(8));
        assert!(!back.is_fully_researched(2));
        // The newly-unlocked set does NOT ride the save (the C#'s
        // `_newlyUnlocked` is not persisted).
        assert!(!back.is_newly_researched(8));
        // An unknown name drops (the C# keeps the string row but the cache
        // never sees it — the port's tracker IS the cache).
        let mut unknown = ResearchTracker::default();
        unknown.load_pairs(&[("NotAnItem".to_string(), 50)]);
        assert_eq!(unknown.items_with_progress().count(), 0);
        // A negative count clamps to 0 (the C#'s Load stores the raw int;
        // the port's u32 cache floors at 0 — the documented deviation).
        let mut neg = ResearchTracker::default();
        neg.load_pairs(&[("Torch".to_string(), -5)]);
        assert_eq!(neg.sacrificed(8), 0);
    }

    /// W4-c24c — `MarkContentsDirty` (`ItemsSacrificedUnlocksTracker.cs:
    /// 222-225`): every successful registration bumps `LastEditId`; the
    /// refused arms (capless `:122-125`, already-full `:127-130`) do not.
    /// `Reset` bumps too (`:215`); the load does NOT (the C# `Load` writes
    /// the dictionaries directly, `:171-196`).
    #[test]
    fn the_edit_version_bumps_only_on_registered_changes() {
        let mut t = ResearchTracker::default();
        assert_eq!(t.last_edit_id(), 0);
        t.register_sacrifice(6196, 5); // capless — no registration, no bump
        assert_eq!(t.last_edit_id(), 0);
        t.register_sacrifice(8, 50);
        assert_eq!(t.last_edit_id(), 1);
        t.register_sacrifice(8, 50); // completes the research
        assert_eq!(t.last_edit_id(), 2);
        t.register_sacrifice(8, 50); // already-full — the early return (:127)
        assert_eq!(t.last_edit_id(), 2);
        t.reset();
        assert_eq!(t.last_edit_id(), 3, "Reset marks dirty (:215)");
        let mut loaded = ResearchTracker::default();
        loaded.load_pairs(&[("Torch".to_string(), 100)]);
        assert_eq!(loaded.last_edit_id(), 0, "Load never marks dirty");
        // The save does not carry the version (only the count pairs).
        assert_eq!(t.save_pairs().len(), 0);
    }

    /// W4-c24c — `CountFullyResearchedItems` (`:60-75`): the denominator is
    /// the catalog-capped id space (the port's generated SACRIFICES rows over
    /// NAMES), the numerator the kinds at their cap. The Torch reaching 100
    /// moves it into the fully set; the partial Dirt does not.
    #[test]
    fn the_fully_researched_count_walks_the_capped_space() {
        let mut t = ResearchTracker::default();
        let (full0, all) = t.count_fully_researched();
        assert_eq!(full0, 0, "a fresh tracker has researched nothing");
        assert!(all > 6000, "the catalog's capped rows: {all}");
        t.register_sacrifice(8, 100); // Torch, fully
        t.register_sacrifice(2, 5); // DirtBlock, partial
        let (full1, all1) = t.count_fully_researched();
        assert_eq!(all1, all, "the denominator is the catalog, not the tracker");
        assert_eq!(full1, 1);
        t.register_sacrifice(2, 100); // DirtBlock cap 100 — completes
        let (full2, _) = t.count_fully_researched();
        assert_eq!(full2, 2);
        t.reset();
        assert_eq!(t.count_fully_researched().0, 0);
    }
}
