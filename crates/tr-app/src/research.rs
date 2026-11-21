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
    /// `ItemsSacrificedUnlocksTracker.LastEditId` (`:23`): bumped by every
    /// `MarkContentsDirty` (`:222-225`) — a sacrifice registration or a
    /// direct count set. `CheckResearchAchievement` compares it against its
    /// own `_lastResearchVersion` latch to skip unchanged frames.
    last_edit_id: i32,
}

impl ResearchTracker {
    /// `ItemsSacrificedUnlocksTracker.RegisterItemSacrifice`'s count
    /// accumulation (the stack merges into the running total). The C# clamps
    /// at the cap and marks dirty (`:131-134`); the port keeps the running
    /// total (the cap read rides `is_fully_researched`) and bumps the edit id.
    pub fn register_sacrifice(&mut self, item_id: u16, stack: u32) {
        for (eid, count) in self.counts.iter_mut() {
            if *eid == item_id {
                *count += stack;
                self.last_edit_id += 1;
                return;
            }
        }
        self.counts.push((item_id, stack));
        self.last_edit_id += 1;
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

    /// `ItemsSacrificedUnlocksTracker.LastEditId` (`:23`) — the dirty counter
    /// `CheckResearchAchievement` latches against.
    pub fn last_edit_id(&self) -> i32 {
        self.last_edit_id
    }

    /// `ItemsSacrificedUnlocksTracker.CountFullyResearchedItems` (`:60-75`):
    /// walks `ItemID.Count` (6196, `ItemID.cs:14861`); every id with a
    /// sacrifice-cap row counts toward `all_items`, and the ones whose
    /// sacrificed total reaches the cap count toward `fully_researched`.
    /// Returns `(fully_researched, all_items)`.
    pub fn count_fully_researched(&self) -> (u32, u32) {
        let mut fully = 0u32;
        let mut all = 0u32;
        for id in 0..crate::itemdata::ITEMS.len() as u16 {
            if let Some(cap) = sacrifice_cap(id) {
                all += 1;
                if self.sacrificed(id) >= cap {
                    fully += 1;
                }
            }
        }
        (fully, all)
    }

    pub fn reset(&mut self) {
        self.counts.clear();
        // `Reset` calls `MarkContentsDirty` (`:215`).
        self.last_edit_id += 1;
    }
}

/// `AchievementsHelper.CheckResearchAchievement` (`AchievementsHelper.cs:387-399`):
/// when `forced` or the tracker's `LastEditId` moved past the caller's
/// `_lastResearchVersion` latch, count the fully-researched items and fire
/// progression 45 (TRASH_COMPACTOR) once at least half the researchable
/// catalogue is complete (`fullyResearchedItems >= allItems / 2`, integer
/// division). Returns `(fired, new_last_research_version)` — the caller keeps
/// the version latch (the C# static `_lastResearchVersion`, `:17`) and routes
/// `fired` through `notify_progression_net(45)`.
///
/// Wiring: the C# calls this from `Player.UpdateArmorSets`' per-tick tail
/// (`Player.cs:25703`) and `Player.Hooks.EnterWorld` (`:325`, forced). The
/// port's live sacrifice UI is unported (the `ResearchTracker` has no App
/// field or consumer yet), so the tracker never edits in play and the check
/// is inert — the helper + the count are the landed prep, the call sites stay
/// cited until the Journey research UI lands (the census, W4-f30).
pub fn check_research_achievement(
    tracker: &ResearchTracker,
    last_research_version: i32,
    forced: bool,
) -> (bool, i32) {
    let last_edit_id = tracker.last_edit_id();
    if forced || last_research_version != last_edit_id {
        let (fully, all) = tracker.count_fully_researched();
        // `allItems / 2` is C# integer division (`:394`).
        (fully >= all / 2, last_edit_id)
    } else {
        (false, last_research_version)
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

    // ---- W4-f30: the research family (progression 45) ---------------------

    /// `CountFullyResearchedItems` (`ItemsSacrificedUnlocksTracker.cs:60-75`)
    /// counts every cap-row id toward `all_items` and the cap-reached ones
    /// toward `fully_researched`. The Torch (8, cap 100) and the Musket Ball
    /// (97, cap 99) are two known rows.
    #[test]
    fn f30_the_research_count_walks_the_cap_table() {
        let mut t = ResearchTracker::default();
        let (fully0, all) = t.count_fully_researched();
        assert_eq!(fully0, 0, "nothing researched at start");
        assert!(all > 0, "the catalogue has researchable rows");
        // Research the Torch fully; the Musket Ball stays open.
        t.register_sacrifice(8, 100);
        let (fully1, all1) = t.count_fully_researched();
        assert_eq!(all1, all, "the catalogue size is constant");
        assert_eq!(fully1, 1, "the Torch is fully researched");
        assert!(t.is_fully_researched(8));
        // A partial sacrifice does not count.
        t.register_sacrifice(97, 50);
        let (fully2, _) = t.count_fully_researched();
        assert_eq!(fully2, 1, "the half-researched Musket Ball stays open");
    }

    /// `LastEditId` bumps on every registration and reset
    /// (`MarkContentsDirty`, `:222-225`); `CheckResearchAchievement` latches
    /// against it (`AchievementsHelper.cs:387-399`).
    #[test]
    fn f30_the_edit_id_latches_the_research_check() {
        let mut t = ResearchTracker::default();
        assert_eq!(t.last_edit_id(), 0);
        t.register_sacrifice(8, 1);
        assert_eq!(t.last_edit_id(), 1);
        // An unchanged tracker with a matching version skips the check.
        let (fired, ver) = check_research_achievement(&t, 1, false);
        assert!(!fired, "nothing researched half the catalogue");
        assert_eq!(ver, 1, "the version stays put");
        // A new edit moves the version even when the count is still short.
        t.register_sacrifice(8, 1);
        let (fired2, ver2) = check_research_achievement(&t, 1, false);
        assert!(!fired2);
        assert_eq!(ver2, 2, "the latch follows the edit id");
        // `forced` re-checks regardless of the version.
        let (_, ver3) = check_research_achievement(&t, 2, true);
        assert_eq!(ver3, 2);
    }

    /// The half-catalogue threshold fires progression 45
    /// (`fullyResearchedItems >= allItems / 2`, integer division, `:394`).
    /// Researching every cap-row id to its cap crosses it.
    #[test]
    fn f30_half_the_catalogue_fires_45() {
        let mut t = ResearchTracker::default();
        let (_, all) = t.count_fully_researched();
        // Fill every researchable id to its cap.
        for id in 0..crate::itemdata::ITEMS.len() as u16 {
            if let Some(cap) = sacrifice_cap(id) {
                t.register_sacrifice(id, cap);
            }
        }
        let (fully, _) = t.count_fully_researched();
        assert_eq!(fully, all, "every row researched");
        let (fired, _) = check_research_achievement(&t, 0, false);
        assert!(fired, "the whole catalogue is past half");
    }
}
