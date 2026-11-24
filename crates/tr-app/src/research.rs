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

    /// `ItemsSacrificedUnlocksTracker.TryGetSacrificeNumbers`
    /// (`ItemsSacrificedUnlocksTracker.cs:75-86`): `(have, needed)` when the
    /// catalog carries the item, `None` for the capless kinds (the UI prints
    /// "X" for them, `UICreativeInfiniteItemsDisplay.cs:426-429`). The c24
    /// Journey-UI port's accessor — the tooltip's research rows read it
    /// (`Main.cs:21318-21326`, the W4-f28 contract).
    pub fn sacrifice_numbers(&self, item_id: u16) -> Option<(u32, u32)> {
        let cap = sacrifice_cap(item_id)?;
        Some((self.sacrificed(item_id), cap))
    }
}

/// The Journey research tooltip row's context whitelist
/// (`Main.cs:21318`'s seventeen `tooltipContext` compares): the inventory
/// grid/hotbar/coins/ammo (0/1/2/3/4/32), the shop (15), the chest (6),
/// the crafting family (22/35/7/5/41/42/43), the trash (5) and the
/// duplication grid (29/34).
pub fn research_row_context(context: i32) -> bool {
    matches!(
        context,
        0 | 1 | 2 | 3 | 4 | 32 | 15 | 6 | 22 | 35 | 7 | 5 | 29 | 34 | 41 | 42 | 43
    )
}

/// The Journey research tooltip rows (`Main.cs:21318-21333`): on a
/// difficulty-3 (Journey) character, a whitelisted context and a catalog
/// item print `CommonItemTooltip.CreativeSacrificeNeeded` with the
/// remaining count while unresearched, in `Colors.JourneyMode`
/// (`Color.Lerp(HotPink, White, 0.1f)` = (255, 120, 187) — the byte Lerp
/// truncates per channel: G = 105 + (byte)(150 × 0.1f) = 120, B = 180 +
/// (byte)(75 × 0.1f) = 187, `ID/Colors.cs:113`); the fully-researched
/// duplication-grid hover duplication-grid hover
/// (context 29) prints `CommonItemTooltip.ItemUnlockedByTeammate` when a
/// teammate credit exists — the credit tracker
/// (`_unlockedByTeammate`/`TryGetTeammateUnlockCredit`) is multiplayer
/// state the c24 port documents as unported, so `teammate_credit` stays
/// `None` (the W4-f28 contract, docs/ANALYSIS.md).
pub fn research_tooltip_rows(
    context: i32,
    difficulty: u8,
    tracker: &ResearchTracker,
    kind: u16,
    teammate_credit: Option<&str>,
    lines: &mut Vec<(String, [f32; 4])>,
) {
    if difficulty != 3 || !research_row_context(context) {
        return;
    }
    let Some((have, needed)) = tracker.sacrifice_numbers(kind) else {
        return;
    };
    // `Colors.JourneyMode` = Lerp(HotPink (255,105,180), White, 0.1) —
    // the BYTE Lerp: `(byte)(start + (byte)((end - start) * amount))`
    // per channel (XNA `Color.Lerp`), G = 105 + 15 = 120, B = 180 + 7 = 187.
    let journey = [1.0, 120.0 / 255.0, 187.0 / 255.0, 1.0];
    if have < needed {
        lines.push((
            crate::lang::text_args(
                "CommonItemTooltip.CreativeSacrificeNeeded",
                &[(needed - have).to_string()],
            ),
            journey,
        ));
    } else if context == 29 {
        if let Some(name) = teammate_credit {
            lines.push((
                crate::lang::text_args(
                    "CommonItemTooltip.ItemUnlockedByTeammate",
                    &[name.to_string()],
                ),
                journey,
            ));
        }
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
/// Wiring (W4-f30): the C# calls this from `Player.UpdateArmorSets`' per-tick
/// tail (`Player.cs:25703`) and `Player.Hooks.EnterWorld` (`:325`, forced).
/// Both call sites are wired — the per-tick check rides `world_sim/player.rs`'s
/// armor tail, the forced check rides `App::enter_loaded_world`. The tracker
/// is `App.research` (the `LastEditId` latch is `App.last_research_version`).
/// The live sacrifice UI is unported, so the tracker never edits in play and
/// the check is inert until the Journey research UI lands — the helper, the
/// count, the App field and both call sites are the landed prep.
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

    /// W4-f28 — the Journey research tooltip rows (`Main.cs:21318-21333`):
    /// the difficulty-3 gate, the seventeen-context whitelist, the
    /// remaining-count row in Colors.JourneyMode, and the teammate-credit
    /// arm's context-29 restriction. WRITTEN NOT RUN (code-only mode).
    #[test]
    fn w4_f28_the_research_rows_gate_and_print_the_remaining_count() {
        let _g = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
        crate::lang::test_register_source(
            "Terraria.Localization.Content.en-US.Items.json",
            r#"{ "CommonItemTooltip": {
                    "CreativeSacrificeNeeded": "Research {0} more to unlock duplication",
                    "ItemUnlockedByTeammate": "Research completed by {0}" } }"#,
        );
        let mut t = ResearchTracker::default();
        t.register_sacrifice(8, 40); // the Torch: cap 100, 60 remaining
        let journey = [1.0, 120.0 / 255.0, 187.0 / 255.0, 1.0];
        // The whitelist: all seventeen contexts pass, the others refuse.
        for c in [0, 1, 2, 3, 4, 32, 15, 6, 22, 35, 7, 5, 29, 34, 41, 42, 43] {
            assert!(research_row_context(c), "context {c}");
        }
        for c in [8, 9, 10, 11, 12, 13, 21, 30, -1] {
            assert!(!research_row_context(c), "context {c} must refuse");
        }
        // The remaining-count row on a Journey character.
        let mut lines = Vec::new();
        research_tooltip_rows(0, 3, &t, 8, None, &mut lines);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].0, "Research 60 more to unlock duplication");
        assert_eq!(lines[0].1, journey);
        // Classic (0) / mediumcore (1) / hardcore (2) characters: no row.
        for d in [0u8, 1, 2] {
            let mut lines = Vec::new();
            research_tooltip_rows(0, d, &t, 8, None, &mut lines);
            assert!(lines.is_empty(), "difficulty {d}");
        }
        // A non-whitelisted context: no row even on Journey.
        let mut lines = Vec::new();
        research_tooltip_rows(8, 3, &t, 8, None, &mut lines);
        assert!(lines.is_empty());
        // The capless item: TryGetSacrificeNumbers fails, no row.
        let mut lines = Vec::new();
        research_tooltip_rows(0, 3, &t, 6196, None, &mut lines);
        assert!(lines.is_empty());
        // Fully researched: the teammate arm fires ONLY on context 29.
        t.register_sacrifice(8, 60);
        let mut lines = Vec::new();
        research_tooltip_rows(0, 3, &t, 8, Some("Bob"), &mut lines);
        assert!(lines.is_empty(), "context 0 never prints the credit");
        research_tooltip_rows(29, 3, &t, 8, Some("Bob"), &mut lines);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].0, "Research completed by Bob");
        assert_eq!(lines[0].1, journey);
        // No credit (the single-player port): no row.
        let mut lines = Vec::new();
        research_tooltip_rows(29, 3, &t, 8, None, &mut lines);
        assert!(lines.is_empty());
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

    /// W4-f30 — the EnterWorld forced check (`Player.cs:325` →
    /// `CheckResearchAchievement(forced: true)`): a world whose tracker is
    /// already past half fires 45 on entry; an empty tracker does not. The
    /// `enter_loaded_world` call site reads `App.research`. WRITTEN NOT RUN.
    #[test]
    fn f30_the_enter_world_forced_check_fires_45() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        // An empty tracker: the forced check does not fire.
        app.enter_loaded_world();
        let row = app.achievements.list.iter().find(|a| a.name == "TRASH_COMPACTOR").unwrap();
        assert!(!row.completed, "an empty catalogue never fires 45");
        // Fill the tracker past half, then re-enter: the forced check fires.
        let (_, all) = app.research.count_fully_researched();
        let mut filled = 0u32;
        for id in 0..crate::itemdata::ITEMS.len() as u16 {
            if let Some(cap) = sacrifice_cap(id) {
                app.research.register_sacrifice(id, cap);
                filled += 1;
                if filled >= all / 2 + 1 {
                    break;
                }
            }
        }
        app.enter_loaded_world();
        let row = app.achievements.list.iter().find(|a| a.name == "TRASH_COMPACTOR").unwrap();
        assert!(row.completed, "the past-half catalogue fires 45 on entry");
    }
}
