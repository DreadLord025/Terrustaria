//! The bestiary filter/sort engine (W4-e23): `EntryFilterer`/`EntrySorter`
//! (`DataStructures/EntryFilterer.cs:6-100`, `EntrySorter.cs:6-51`), the
//! `Filters` family (`GameContent/Bestiary/Filters.cs:10-184`), the
//! `SortingSteps` family (`SortingSteps.cs:9-233`) and the
//! `ContentSamples.NpcBestiarySortingId` order
//! (`ContentSamples.cs:765-830`/`:1083-1092`) over the port's roster.
//!
//! The data half (tags, priority lists, aliases) is the generated
//! [`crate::bestiary_tags`]; the unlock ladder is `tr_core::bestiary` +
//! [`crate::bestiary_data`] (W4-c37). This module is the working-set
//! machinery `UIBestiaryTest.UpdateBestiaryContents` drives
//! (`UIBestiaryTest.cs:432-444`): filter → sort → fill.
//!
//! Documented deviations (ANALYSIS "The bestiary grid/filters/sort (W4-e23)"):
//! * the sorting-id order runs over the port's `ENTRIES` roster, not the
//!   vanilla full NPC set — the ORDER semantics are the C#'s nine keys, the
//!   absolute numbers differ where unported NPCs would interleave;
//! * `Alphabetical` compares with Rust's ordinal `str` order — the C#
//!   `string.CompareTo` is culture-aware (`SortingSteps.cs:136`); identical
//!   for the English name column;
//! * `BySearch` lowercases both sides (`Filters.cs:30`'s
//!   `ToLower().IndexOf(..., OrdinalIgnoreCase)`); culture case-folds beyond
//!   ASCII are not modelled.

use std::cmp::Ordering;
use std::sync::OnceLock;

use tr_core::bestiary::{UnlockState, WorldBestiary};

use crate::bestiary::{BestiaryEntryDef, ENTRIES};
use crate::bestiary_tags as tags;
use crate::npc::SlimeVariant;

// --- Filters (Filters.cs) -----------------------------------------------------

/// One `IBestiaryEntryFilter` of the database's `AvailableFilters` list —
/// `RegisterFilters` (`BestiaryDatabaseNPCsPopulator.cs:581-591`): the three
/// named filters, then one `ByInfoElement` per common tag (57). The search
/// filter is NOT in this list — `SetSearchFilterObject` keeps it separate
/// (`UIBestiaryTest.cs:55`, `EntryFilterer.cs:80-83`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filter {
    /// `Filters.ByUnlockState` (`Filters.cs:60-84`).
    UnlockState,
    /// `Filters.ByBoss` (`:118-148`).
    Boss,
    /// `Filters.ByRareCreature` (`:86-116`).
    RareCreature,
    /// `Filters.ByInfoElement(tag)` (`:150-183`) — the tag id indexes
    /// [`tags::TAGS`]'s common-filter prefix.
    Tag(u32),
}

/// `AvailableFilters.Count` — 3 + `COMMON_FILTER_COUNT` (`:583-590`).
pub const FILTER_COUNT: usize = 3 + tags::COMMON_FILTER_COUNT;

/// `AvailableFilters[i]` in registration order.
pub fn filter_at(i: usize) -> Filter {
    match i {
        0 => Filter::UnlockState,
        1 => Filter::Boss,
        2 => Filter::RareCreature,
        _ => Filter::Tag((i - 3) as u32),
    }
}

/// The entry carries a `BossBestiaryInfoElement` — the factory's boss rule
/// (`BestiaryEntry.cs:37-41`) minus the Martian Saucer's `RemoveAll`
/// (`BestiaryDatabaseNPCsPopulator.cs:668`).
pub fn is_boss_entry(net_id: i32) -> bool {
    (crate::bestiary_data::set_defaults_boss(net_id)
        || crate::bestiary_data::counted_as_boss_for_bestiary(net_id))
        && net_id != 395
}

/// `FitsFilter` per filter (`Filters.cs:64-68`/`:90-100`/`:122-132`/
/// `:161-164`). `unlock` is the entry's live tier; the tag arm is the
/// reference-equality membership of [`tags::has_tag`].
pub fn filter_fits(f: Filter, entry: &BestiaryEntryDef, unlock: UnlockState) -> bool {
    match f {
        // `entry.Icon.GetUnlockState` = tier > 0 (`UnlockableNPCEntryIcon.cs:223-226`).
        Filter::UnlockState => unlock > UnlockState::NotKnownAtAll,
        Filter::Boss => is_boss_entry(entry.net_id),
        Filter::RareCreature => crate::bestiary_data::rarity(entry.net_id) != 0,
        Filter::Tag(t) => tags::has_tag(entry.net_id, t),
    }
}

/// `ForcedDisplay` (`Filters.cs:62`/`:88`/`:120`/`:154`): the unlock filter
/// forces itself visible; the rest ride the availability test.
pub fn filter_forced_display(f: Filter) -> Option<bool> {
    match f {
        Filter::UnlockState => Some(true),
        _ => None,
    }
}

/// `GetDisplayNameKey` (`Filters.cs:70-73`/`:102-105`/`:134-137`/
/// `:166-173` → `FilterProviderInfoElement.cs:50-53`).
pub fn filter_display_name_key(f: Filter) -> &'static str {
    match f {
        Filter::UnlockState => "BestiaryInfo.IfUnlocked",
        Filter::Boss => "BestiaryInfo.IsBoss",
        Filter::RareCreature => "BestiaryInfo.IsRare",
        Filter::Tag(t) => tags::TAGS[t as usize].key,
    }
}

/// The filter button's icon: `Icon_Tags_Shadow`'s 16x5 frame index, or
/// `None` for `ByRareCreature`'s whole `Icon_Rank_Light` sheet
/// (`Filters.cs:75-83`/`:107-115`/`:139-147`,
/// `FilterProviderInfoElement.cs:31-39`). ByUnlockState = frame (14, 3),
/// ByBoss = (15, 3).
pub fn filter_icon_frame(f: Filter) -> Option<u32> {
    match f {
        Filter::UnlockState => Some(3 * 16 + 14),
        Filter::Boss => Some(3 * 16 + 15),
        Filter::RareCreature => None,
        Filter::Tag(t) => Some(tags::TAGS[t as usize].frame),
    }
}

// --- the search filter (Filters.BySearch) --------------------------------------

/// `Filters.BySearch.FitsFilter` (`Filters.cs:18-37`): any
/// `IProvideSearchFilterString` in the entry's Info whose string contains
/// the query (case-insensitive). The port's providers: the name plate
/// (`NamePlateInfoElement.cs:33-40` — the localized name, known only), the
/// tag rows (`FilterProviderInfoElement.cs:41-48` — the tag's display text,
/// known only), the rare/boss markers (`RareSpawnBestiaryInfoElement.cs:20-27`,
/// `BossBestiaryInfoElement.cs:13-20` — tier ≥ 1), the search aliases
/// (`SearchAliasInfoElement.cs:14-21`) and the drop item names
/// (`ItemDropBestiaryInfoElement.cs:48-60` — tier ≥ 2, the INTERNAL item
/// name, not the localized tooltip).
pub fn search_fits(entry: &BestiaryEntryDef, unlock: UnlockState, search: &str) -> bool {
    let known = unlock > UnlockState::NotKnownAtAll;
    let needle = search.to_lowercase();
    let hit = |s: &str| s.to_lowercase().contains(&needle);
    if known {
        let name = crate::ui::bestiary::entry_hover_text(entry, unlock);
        if hit(&name) {
            return true;
        }
        for t in tags::entry_tags(entry.net_id) {
            // Wrapped rows are still IProvideSearchFilterString members of
            // Info — the search walks Info, not the filter singletons.
            if hit(&crate::lang::text(tags::TAGS[t.tag as usize].key)) {
                return true;
            }
        }
        if crate::bestiary_data::rarity(entry.net_id) != 0
            && hit(&crate::lang::text("BestiaryInfo.IsRare"))
        {
            return true;
        }
        if is_boss_entry(entry.net_id) && hit(&crate::lang::text("BestiaryInfo.IsBoss")) {
            return true;
        }
        for alias in tags::search_aliases(entry.net_id) {
            if hit(alias) {
                return true;
            }
        }
    }
    if unlock >= UnlockState::CanShowStats {
        for row in crate::loot_db::drop_rows(entry.net_id) {
            if hit(&crate::registry::name(row.item_id as u16)) {
                return true;
            }
        }
    }
    false
}

// --- EntryFilterer (EntryFilterer.cs) -------------------------------------------

/// `EntryFilterer<BestiaryEntry, IBestiaryEntryFilter>`
/// (`EntryFilterer.cs:6-100`). `AlwaysActiveFilters` stays empty (the
/// bestiary never adds one); the search filter object is `BySearch`.
#[derive(Clone, Debug, Default)]
pub struct EntryFilterer {
    /// `ActiveFilters` — indices into the `AvailableFilters` order, in
    /// toggle order (the C# list order; only membership is read).
    pub active: Vec<usize>,
    /// `_searchFilter`'s payload — `None` while null/whitespace
    /// (`SetSearchFilter:85-94`).
    pub search: Option<String>,
}

impl EntryFilterer {
    /// `FitsFilter` (`:30-55`): the search gate first, then the active
    /// filters OR'd (empty active list passes everything).
    pub fn fits(&self, entry: &BestiaryEntryDef, unlock: UnlockState) -> bool {
        if let Some(s) = &self.search {
            if !search_fits(entry, unlock, s) {
                return false; // `:32-35`
            }
        }
        if self.active.is_empty() {
            return true; // `:43-46`
        }
        self.active
            .iter()
            .any(|&i| filter_fits(filter_at(i), entry, unlock)) // `:47-54`
    }

    /// `ToggleFilter` (`:57-68`).
    pub fn toggle(&mut self, i: usize) {
        if let Some(pos) = self.active.iter().position(|&x| x == i) {
            self.active.remove(pos);
        } else {
            self.active.push(i);
        }
    }

    /// `IsFilterActive` (`:70-78`) — the range gate included.
    pub fn is_active(&self, i: usize) -> bool {
        i < FILTER_COUNT && self.active.contains(&i)
    }

    /// `SetSearchFilter` (`:85-94`): whitespace clears the filter object.
    pub fn set_search(&mut self, text: &str) {
        if text.trim().is_empty() {
            self.search = None;
        } else {
            self.search = Some(text.to_string());
        }
    }

    /// `GetDisplayName` (`:96-99`): "Filters ({Count})".
    pub fn display_name(&self) -> String {
        crate::lang::text_args("BestiaryInfo.Filters", &[self.active.len().to_string()])
    }
}

// --- the sorting-id order (ContentSamples.BestiaryHelper) ------------------------

/// `NPC.CountsAsACritter` (`NPC.cs:6887-6897`): `lifeMax <= 5 &&
/// damage == 0 && type != 594 && type != 686`.
fn counts_as_critter(entry: &BestiaryEntryDef) -> bool {
    let stats = crate::npc::entry_stats(entry.kind, entry.variant.unwrap_or(SlimeVariant::Blue));
    stats.hp <= 5 && stats.damage == 0 && entry.net_id != 594 && entry.net_id != 686
}

/// `NPC.isLikeATownNPC` (`NPC.cs:6856-6866`): `type == 453 || townNPC`.
fn is_like_town(entry: &BestiaryEntryDef) -> bool {
    entry.net_id == 453 || entry.kind.is_town_npc()
}

/// `GetBestiaryPowerLevel` (`ContentSamples.cs:797-800`):
/// `damage + defense + lifeMax / 4` — integer division, int sum.
fn power_level(entry: &BestiaryEntryDef) -> i32 {
    let stats = crate::npc::entry_stats(entry.kind, entry.variant.unwrap_or(SlimeVariant::Blue));
    stats.damage + stats.defense + stats.hp / 4
}

/// The nine-key sort key of `GetSortedBestiaryEntriesList`
/// (`ContentSamples.cs:768-770`), in `orderby` order.
fn sorting_key(entry: &BestiaryEntryDef) -> (i32, bool, i32, bool, i32, i32, u8, i32, i32) {
    (
        tags::town_priority(entry.net_id),
        !is_like_town(entry),
        tags::gold_critter_priority(entry.net_id),
        !counts_as_critter(entry),
        tags::boss_priority(entry.net_id),
        tags::lowest_biome_group_index(entry.net_id),
        entry.kind.ai_style(),
        power_level(entry),
        crate::bestiary_data::rarity_stars(entry),
    )
}

/// `CreateBestiarySortingIds` (`ContentSamples.cs:1083-1092`): the sorted
/// roster (hidden entries removed, `:771`) numbered from 1. Computed once
/// over the port's `ENTRIES` — the documented roster narrowing.
fn sorting_ids() -> &'static std::collections::HashMap<i32, i32> {
    static IDS: OnceLock<std::collections::HashMap<i32, i32>> = OnceLock::new();
    IDS.get_or_init(|| {
        let mut order: Vec<&BestiaryEntryDef> = ENTRIES
            .iter()
            .filter(|e| {
                // `ShouldHideBestiaryEntry` (`:788-795`).
                !crate::ui::bestiary_draw_offsets::get(e.net_id)
                    .map(|m| m.hide)
                    .unwrap_or(false)
            })
            .collect();
        // LINQ orderby is a stable sort; the roster order breaks ties.
        order.sort_by_key(|e| sorting_key(e));
        order
            .iter()
            .enumerate()
            .map(|(i, e)| (e.net_id, i as i32 + 1))
            .collect()
    })
}

/// `ContentSamples.NpcBestiarySortingId[netId]` — the entry's display index
/// (`NPCNetIdBestiaryInfoElement.cs:10`) and the `ByBestiarySortingId` key.
/// An id outside the roster reads 0 (the C# dictionary would throw; the
/// port's callers only pass roster ids).
pub fn sorting_id(net_id: i32) -> i32 {
    sorting_ids().get(&net_id).copied().unwrap_or(0)
}

// --- SortingSteps (SortingSteps.cs) ----------------------------------------------

/// One `IBestiarySortStep` — `RegisterSortSteps`' registration order
/// (`BestiaryDatabaseNPCsPopulator.cs:562-579`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortStep {
    UnlockState,
    BestiarySortingId,
    Alphabetical,
    NetId,
    Attack,
    Defense,
    Coins,
    Hp,
    BestiaryRarity,
}

/// The registered steps, in order (`:566-574`).
pub const SORT_STEPS: &[SortStep] = &[
    SortStep::UnlockState,
    SortStep::BestiarySortingId,
    SortStep::Alphabetical,
    SortStep::NetId,
    SortStep::Attack,
    SortStep::Defense,
    SortStep::Coins,
    SortStep::Hp,
    SortStep::BestiaryRarity,
];

/// `HiddenFromSortOptions` (`SortingSteps.cs:13`/`:42` true, the rest false).
pub fn step_hidden(step: SortStep) -> bool {
    matches!(step, SortStep::UnlockState | SortStep::NetId)
}

/// `GetDisplayNameKey` per step (`SortingSteps.cs:34-37` etc.).
pub fn step_display_name_key(step: SortStep) -> &'static str {
    match step {
        SortStep::UnlockState => "BestiaryInfo.Sort_Unlocks",
        SortStep::BestiarySortingId => "BestiaryInfo.Sort_BestiaryID",
        SortStep::Alphabetical => "BestiaryInfo.Sort_Alphabetical",
        SortStep::NetId => "BestiaryInfo.Sort_ID",
        SortStep::Attack => "BestiaryInfo.Sort_Attack",
        SortStep::Defense => "BestiaryInfo.Sort_Defense",
        SortStep::Coins => "BestiaryInfo.Sort_Coins",
        SortStep::Hp => "BestiaryInfo.Sort_HitPoints",
        SortStep::BestiaryRarity => "BestiaryInfo.Sort_Rarity",
    }
}

/// One entry's live sort inputs — the C# re-reads the provider/stats on
/// every compare (`SortEntries` runs `UpdateBeforeSorting` first,
/// `UIBestiaryTest.cs:320-333`); the port snapshots them per
/// `UpdateBestiaryContents`, the same refresh cadence.
pub struct SortKey<'a> {
    pub entry: &'a BestiaryEntryDef,
    /// `Icon.GetUnlockState` — the known bool, not the tier
    /// (`SortingSteps.cs:44-49`).
    pub known: bool,
    /// The localized name (`Language.GetTextValue(TypeName)`, `:136`).
    pub name: String,
    pub sorting_id: i32,
    pub stars: i32,
    pub damage: i32,
    pub defense: i32,
    pub value: f32,
    pub hp: i32,
}

/// Build the snapshot for one entry.
pub fn sort_key<'a>(entry: &'a BestiaryEntryDef, tracker: &WorldBestiary) -> SortKey<'a> {
    let unlock = crate::bestiary::unlock_state(entry, tracker);
    let stats = crate::npc::entry_stats(entry.kind, entry.variant.unwrap_or(SlimeVariant::Blue));
    SortKey {
        entry,
        known: unlock > UnlockState::NotKnownAtAll,
        // `Alphabetical` compares the localized TypeName with NO unlock
        // gate (`SortingSteps.cs:136`) — unlike the hover text's "???".
        name: {
            let localized = crate::lang::npc_name(entry.net_id);
            if localized.is_empty() {
                entry.name.to_string()
            } else {
                localized
            }
        },
        sorting_id: sorting_id(entry.net_id),
        stars: crate::bestiary_data::rarity_stars(entry),
        damage: stats.damage,
        defense: stats.defense,
        value: stats.value,
        hp: stats.hp,
    }
}

/// One step's `Compare` (`SortingSteps.cs`). Every roster entry carries the
/// net-id/stats elements, so the null-element arms (`:19-30` etc.) never
/// fire — the port compares the values directly.
pub fn compare_step(step: SortStep, x: &SortKey, y: &SortKey) -> Ordering {
    match step {
        // `y.GetUnlockState().CompareTo(x...)` — known first (`:48`).
        SortStep::UnlockState => y.known.cmp(&x.known),
        SortStep::BestiarySortingId => x.sorting_id.cmp(&y.sorting_id), // `:77`
        SortStep::Alphabetical => x.name.cmp(&y.name),                  // `:136`
        SortStep::NetId => x.entry.net_id.cmp(&y.entry.net_id),         // `:31`
        SortStep::Attack => y.damage.cmp(&x.damage),                    // `:186`
        SortStep::Defense => y.defense.cmp(&x.defense),                 // `:199`
        SortStep::Coins => y.value.total_cmp(&x.value),                 // `:212`
        SortStep::Hp => y.hp.cmp(&x.hp),                                // `:225`
        SortStep::BestiaryRarity => y.stars.cmp(&x.stars),              // `:107`
    }
}

// --- EntrySorter (EntrySorter.cs) -------------------------------------------------

/// `EntrySorter<BestiaryEntry, IBestiarySortStep>` (`EntrySorter.cs:6-51`).
#[derive(Clone, Copy, Debug)]
pub struct EntrySorter {
    /// `_prioritizedStep` — the field default 0 (the hidden ByUnlockState).
    pub prioritized: usize,
}

impl Default for EntrySorter {
    fn default() -> Self {
        Self { prioritized: 0 }
    }
}

impl EntrySorter {
    /// `Compare` (`:17-40`): the prioritized step first (unless -1 — the
    /// port's `usize` never is), then every other step in list order.
    pub fn compare(&self, x: &SortKey, y: &SortKey) -> Ordering {
        let first = compare_step(SORT_STEPS[self.prioritized], x, y);
        if first != Ordering::Equal {
            return first; // `:20-27`
        }
        for (i, &step) in SORT_STEPS.iter().enumerate() {
            if i != self.prioritized {
                let c = compare_step(step, x, y);
                if c != Ordering::Equal {
                    return c; // `:28-38`
                }
            }
        }
        Ordering::Equal // `:39`
    }

    /// `SetPrioritizedStepIndex` (`:42-45`).
    pub fn set_prioritized(&mut self, index: usize) {
        self.prioritized = index;
    }

    /// `GetDisplayName` (`:47-50`).
    pub fn display_name(&self) -> String {
        crate::lang::text(step_display_name_key(SORT_STEPS[self.prioritized]))
    }
}

// --- the working set (UIBestiaryTest.FilterEntries/SortEntries) --------------------

/// `FilterEntries` + `SortEntries` (`UIBestiaryTest.cs:314-333`): the
/// original roster filtered, then sorted — returned as indices into
/// [`ENTRIES`] (the grid's working set). The C# `List.Sort` is unstable,
/// but the chain ends in the unique net ids, a total order — the stable
/// sort produces the identical permutation.
pub fn working_set(
    tracker: &WorldBestiary,
    filterer: &EntryFilterer,
    sorter: &EntrySorter,
) -> Vec<usize> {
    let mut keys: Vec<(usize, SortKey)> = ENTRIES
        .iter()
        .enumerate()
        .filter(|(_, e)| filterer.fits(e, crate::bestiary::unlock_state(e, tracker)))
        .map(|(i, e)| (i, sort_key(e, tracker)))
        .collect();
    keys.sort_by(|(_, a), (_, b)| sorter.compare(a, b));
    keys.into_iter().map(|(i, _)| i).collect()
}

/// `GetIsFilterAvailableForEntries` (`UIBestiaryFilteringOptionsGrid.cs:192-207`)
/// over the whole original roster: forced display wins, else any entry that
/// fits AND is known. The availability test lists are built once from the
/// ORIGINAL roster (`SetupAvailabilityTest:62-78`) — the fits half is
/// tracker-independent for every filter but `ByUnlockState` (forced anyway).
pub fn filter_available(i: usize, tracker: &WorldBestiary) -> bool {
    let f = filter_at(i);
    if let Some(forced) = filter_forced_display(f) {
        return forced;
    }
    ENTRIES.iter().any(|e| {
        let unlock = crate::bestiary::unlock_state(e, tracker);
        filter_fits(f, e, unlock) && unlock > UnlockState::NotKnownAtAll
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(net: i32) -> &'static BestiaryEntryDef {
        ENTRIES
            .iter()
            .find(|e| e.net_id == net)
            .unwrap_or_else(|| panic!("roster carries net {net}"))
    }

    #[test]
    fn the_filter_list_is_three_named_plus_the_common_tags() {
        // RegisterFilters (:581-591): 3 + 57 = 60.
        assert_eq!(FILTER_COUNT, 60);
        assert_eq!(filter_at(0), Filter::UnlockState);
        assert_eq!(filter_at(1), Filter::Boss);
        assert_eq!(filter_at(2), Filter::RareCreature);
        assert_eq!(filter_at(3), Filter::Tag(0)); // Biomes.Surface
        assert_eq!(filter_at(59), Filter::Tag(56)); // Biomes.StardustPillar
        // The icon frames: (14,3)/(15,3) on the 16x5 sheet, the rare
        // filter's whole Icon_Rank_Light, Surface frame 0.
        assert_eq!(filter_icon_frame(Filter::UnlockState), Some(62));
        assert_eq!(filter_icon_frame(Filter::Boss), Some(63));
        assert_eq!(filter_icon_frame(Filter::RareCreature), None);
        assert_eq!(filter_icon_frame(Filter::Tag(0)), Some(0));
        // ForcedDisplay: only the unlock filter.
        assert_eq!(filter_forced_display(Filter::UnlockState), Some(true));
        assert_eq!(filter_forced_display(Filter::Boss), None);
    }

    #[test]
    fn the_active_filters_or_and_the_search_gates_first() {
        // EntryFilterer.FitsFilter (:30-55): empty active passes all;
        // two active filters OR; the search ANDs on top.
        let mut f = EntryFilterer::default();
        let zombie = entry(3);
        assert!(f.fits(zombie, UnlockState::CanShowDropsWithDropRates));
        f.toggle(1); // Boss
        f.toggle(2); // Rare
        assert!(!f.fits(zombie, UnlockState::CanShowDropsWithDropRates));
        assert!(f.fits(entry(4), UnlockState::CanShowDropsWithDropRates)); // EoC boss
        assert!(f.fits(entry(45), UnlockState::CanShowDropsWithDropRates)); // Tim rare
        // Toggle off removes.
        f.toggle(1);
        assert!(!f.is_active(1));
        assert!(f.is_active(2));
        assert!(!f.is_active(FILTER_COUNT)); // the range gate (:72-75)
        // Whitespace search clears the object (:87-91).
        f.set_search("   ");
        assert!(f.search.is_none());
        f.set_search("tim");
        assert_eq!(f.search.as_deref(), Some("tim"));
    }

    #[test]
    fn the_boss_filter_drops_the_martian_saucer() {
        // ModifyEntriesThatNeedIt (:668): 395's BossBestiaryInfoElement is
        // removed — the saucer is a SetDefaults boss but not a boss ENTRY.
        assert!(crate::bestiary_data::set_defaults_boss(395));
        assert!(!is_boss_entry(395));
        assert!(is_boss_entry(4));
        // The pillars count without the SetDefaults flag (NPCID.cs:10481).
        assert!(!crate::bestiary_data::set_defaults_boss(517));
        assert!(is_boss_entry(517));
    }

    #[test]
    fn the_wrapped_tags_do_not_count_for_filter_membership() {
        // 175: the automated pass adds UndergroundJungle (:1338), the
        // manual pass removes it (:1798) then re-adds Jungle +
        // UndergroundJungle plainly (:1799-1803); ModifyEntriesThatNeedIt
        // wraps a SECOND UndergroundJungle (:618). The plain row survives,
        // so the filter still fits — but the wrap itself never matches.
        let tags175 = tags::entry_tags(175);
        assert!(tags175.iter().any(|t| t.tag == 21 && t.preferred));
        assert!(tags175.iter().any(|t| t.tag == 21 && !t.preferred));
        assert!(tags::has_tag(175, 21)); // the plain row carries membership
        // 52 keeps its plain Jungle tag (:620) alongside the wrap (:620's
        // preference row) — both present, membership via the plain one.
        assert!(tags::has_tag(52, 20));
        assert!(tags::entry_tags(52).iter().any(|t| t.tag == 20 && t.preferred));
        // The sorting-id biome index reads the plain rows: 175's highest
        // common-filter tag is UndergroundJungle (21), not the wrap.
        assert_eq!(tags::lowest_biome_group_index(175), 21);
        // A tag the entry carries ONLY as a wrap would not fit — the
        // helper's `!preferred` gate is the reference-equality rule
        // (`Filters.ByInfoElement.FitsFilter`, Filters.cs:161-164).
        assert!(!tags::entry_tags(175).iter().any(|t| t.preferred && t.tag == 20));
    }

    #[test]
    fn the_sort_chain_runs_the_prioritized_step_first() {
        let _guard = crate::lang::test_lock().lock().unwrap();
        crate::lang::init(tr_core::localization::GameCulture::English);
        // EntrySorter.Compare (:17-40): default prioritized 0 = the hidden
        // ByUnlockState — known entries sort before unknown ones.
        let tracker = WorldBestiary::new();
        let sorter = EntrySorter::default();
        let mut known = sort_key(entry(3), &tracker);
        let unknown = sort_key(entry(3), &tracker);
        known.known = true;
        assert_eq!(sorter.compare(&known, &unknown), Ordering::Less);
        // Both known: the chain falls to ByBestiarySortingId (step 1).
        let mut a = sort_key(entry(3), &tracker);
        let mut b = sort_key(entry(2), &tracker);
        a.known = true;
        b.known = true;
        assert_eq!(
            sorter.compare(&a, &b),
            a.sorting_id.cmp(&b.sorting_id)
        );
        // Prioritizing Attack flips to damage-descending.
        let mut s = EntrySorter::default();
        s.set_prioritized(4); // ByAttack
        a.damage = 10;
        b.damage = 20;
        assert_eq!(s.compare(&a, &b), Ordering::Greater);
    }

    #[test]
    fn the_sort_options_hide_the_two_machine_steps() {
        // HiddenFromSortOptions: ByUnlockState + ByNetId (7 visible).
        let visible: Vec<_> = SORT_STEPS.iter().filter(|s| !step_hidden(**s)).collect();
        assert_eq!(visible.len(), 7);
        assert_eq!(visible[0], &SortStep::BestiarySortingId);
        assert_eq!(visible[6], &SortStep::BestiaryRarity);
        // The re-click fallback target is the field default 0.
        assert_eq!(EntrySorter::default().prioritized, 0);
    }

    #[test]
    fn the_sorting_id_puts_the_town_roster_first() {
        let _guard = crate::lang::test_lock().lock().unwrap();
        crate::lang::init(tr_core::localization::GameCulture::English);
        // GetSortedBestiaryEntriesList's first key is the town priority
        // list (NPCID.cs:10106-10113): 22 Merchant, 17 Guide, 18 Nurse...
        // — only the ported town ids interleave.
        assert_eq!(sorting_id(22), 1);
        assert_eq!(sorting_id(17), 2);
        assert_eq!(sorting_id(18), 3);
        // Every roster entry carries a unique 1..=N number.
        let mut ids: Vec<i32> = ENTRIES.iter().map(|e| sorting_id(e.net_id)).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), ENTRIES.len());
        assert_eq!(ids[0], 1);
        // The boss priority list ranks the EoC (4) before the EoW (13)
        // among the non-town non-critter entries (NPCID.cs:10096-10105).
        assert!(sorting_id(4) < sorting_id(13));
    }

    #[test]
    fn the_search_matches_names_aliases_and_known_tags_only() {
        let _guard = crate::lang::test_lock().lock().unwrap();
        crate::lang::init(tr_core::localization::GameCulture::English);
        let eoc = entry(4);
        // Unknown entries match nothing (every provider gates on tier).
        assert!(!search_fits(eoc, UnlockState::NotKnownAtAll, "eye"));
        assert!(!search_fits(eoc, UnlockState::NotKnownAtAll, "eoc"));
        let known = UnlockState::CanShowDropsWithDropRates;
        // The alias row (:731) is a RAW string — no localization needed.
        assert!(search_fits(eoc, known, "eoc"));
        assert!(search_fits(eoc, known, "EOC")); // case-insensitive
        assert!(!search_fits(eoc, known, "zzzznope"));
        // The localized providers (name / boss marker / tag text) need the
        // resource files; without them the manager returns the key itself,
        // so pin the gate against `lang::text` like the flavor-text test.
        let boss_text = crate::lang::text("BestiaryInfo.IsBoss");
        assert_eq!(
            search_fits(eoc, known, &boss_text.to_lowercase()),
            true,
            "the boss marker's own display text always matches"
        );
        // The Zombie 3 carries NightTime (:981); its tag text matches when
        // the resources are loaded (else the key string is the haystack).
        let night = crate::lang::text("Bestiary_Times.NightTime");
        assert!(search_fits(entry(3), known, &night.to_lowercase()));
    }

    #[test]
    fn the_working_set_filters_then_sorts() {
        let _guard = crate::lang::test_lock().lock().unwrap();
        crate::lang::init(tr_core::localization::GameCulture::English);
        let tracker = WorldBestiary::new();
        let mut filterer = EntryFilterer::default();
        let sorter = EntrySorter::default();
        // No filters: the whole roster in sorting-id order (all unknown,
        // step 0 ties, step 1 = the sorting id).
        let all = working_set(&tracker, &filterer, &sorter);
        assert_eq!(all.len(), ENTRIES.len());
        assert_eq!(ENTRIES[all[0]].net_id, 22);
        // Boss-only: every survivor is a boss entry.
        filterer.toggle(1);
        let bosses = working_set(&tracker, &filterer, &sorter);
        assert!(!bosses.is_empty());
        assert!(bosses.iter().all(|&i| is_boss_entry(ENTRIES[i].net_id)));
        assert!(bosses.len() < all.len());
    }

    #[test]
    fn the_availability_test_forces_unlock_and_gates_the_rest() {
        // A fresh world knows nothing: only the forced unlock filter shows.
        let tracker = WorldBestiary::new();
        assert!(filter_available(0, &tracker));
        assert!(!filter_available(1, &tracker)); // Boss — nothing known
        assert!(!filter_available(3, &tracker)); // Surface tag
    }
}
