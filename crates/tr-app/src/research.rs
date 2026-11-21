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
/// (`Color.Lerp(HotPink, White, 0.1f)` = (255, 117, 181),
/// `ID/Colors.cs:113`); the fully-researched duplication-grid hover
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
    // `Colors.JourneyMode` = Lerp(HotPink (255,105,180), White, 0.1).
    let journey = [1.0, 117.0 / 255.0, 181.0 / 255.0, 1.0];
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
        let journey = [1.0, 117.0 / 255.0, 181.0 / 255.0, 1.0];
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
}
