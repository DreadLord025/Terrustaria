//! The bestiary kill-tracking data layer — `Main.BestiaryTracker.Kills`
//! (`NPCKillsTracker.cs`) plus the unlock tiers of
//! `CommonEnemyUICollectionInfoProvider` and the kill counts the banner chain
//! needs (`NPCtoBanner` → `BannerToItem` → `ItemID.Sets.KillsToBanner`).
//!
//! The UI half (`UIBestiaryTest`) is the app crate's `ui/bestiary.rs`.

/// `BestiaryEntryUnlockState` (`BestiaryEntryUnlockState.cs`): the five
/// reveal tiers an entry climbs as kills accumulate.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub enum UnlockState {
    #[default]
    NotKnownAtAll = 0,
    CanShowPortraitOnly = 1,
    CanShowStats = 2,
    CanShowDropsWithoutDropRates = 3,
    CanShowDropsWithDropRates = 4,
}

impl UnlockState {
    /// `Utils.ToInt(value: true)` comparisons (`BestiaryEntryIcon`'s
    /// `GetUnlockState`: `providedInfo.UnlockState > NotKnownAtAll_0`).
    pub fn known(self) -> bool {
        self > UnlockState::NotKnownAtAll
    }
}

/// `CommonEnemyUICollectionInfoProvider.GetUnlockStateByKillCount`
/// (`CommonEnemyUICollectionInfoProvider.cs:55-75`): the full count's ½ and ⅕
/// steps down to the first kill's portrait-only reveal.
pub fn get_unlock_state_by_kill_count(
    kill_count: i32,
    quick_unlock: bool,
    full_kill_count_needed: i32,
) -> UnlockState {
    let half = full_kill_count_needed / 2;
    let fifth = full_kill_count_needed / 5;
    if quick_unlock && kill_count > 0 {
        return UnlockState::CanShowDropsWithDropRates;
    }
    if kill_count >= full_kill_count_needed {
        return UnlockState::CanShowDropsWithDropRates;
    }
    if kill_count >= half {
        return UnlockState::CanShowDropsWithoutDropRates;
    }
    if kill_count >= fifth {
        return UnlockState::CanShowStats;
    }
    if kill_count >= 1 {
        return UnlockState::CanShowPortraitOnly;
    }
    UnlockState::NotKnownAtAll
}

/// `NPCKillsTracker.POSITIVE_KILL_COUNT_CAP` (`NPCKillsTracker.cs:12`).
pub const POSITIVE_KILL_COUNT_CAP: i32 = 999_999_999;

/// `NPCKillsTracker` (`NPCKillsTracker.cs:16-101`): the per-world
/// kill counts keyed by the NPC's bestiary credit id (`NPCID.Search.GetName`
/// of the net id — `ContentSamples.Initialize`, `ContentSamples.cs:862-873`).
/// The port persists the pairs in the `.trplayer` sidecar; vanilla stores the
/// same pairs in world-file section 11 (M6-SAVE-1's scope).
#[derive(Clone, Debug, Default)]
pub struct KillTracker {
    counts: std::collections::BTreeMap<String, i32>,
}

impl KillTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// `RegisterKill` (`:23-32`): increment (the count was clamped on write,
    /// so reading before incrementing cannot overflow the cap).
    pub fn register_kill(&mut self, credit_id: &str) {
        let next = self.get_kill_count(credit_id).saturating_add(1);
        self.set_kill_count_directly(credit_id, next);
    }

    /// `SetKillCountDirectly` (`:44-53`) — clamped to the positive cap
    /// (`Utils.Clamp`), no achievement hook (M6-ACH-1's).
    pub fn set_kill_count_directly(&mut self, credit_id: &str, kill_count: i32) {
        let clamped = kill_count.clamp(0, POSITIVE_KILL_COUNT_CAP);
        self.counts.insert(credit_id.to_string(), clamped);
    }

    /// `GetKillCount(string)` (`:55-59`).
    pub fn get_kill_count(&self, credit_id: &str) -> i32 {
        self.counts.get(credit_id).copied().unwrap_or(0)
    }

    /// `Reset` (`:94-98`).
    pub fn reset(&mut self) {
        self.counts.clear();
    }

    /// The `(credit id, count)` pairs — the save order `Save` walks the
    /// dictionary in (`:76-85`); the port's iteration order is the sorted
    /// map's (documented deviation: the C# dictionary order is unspecified).
    pub fn pairs(&self) -> impl Iterator<Item = (&str, i32)> {
        self.counts.iter().map(|(k, v)| (k.as_str(), *v))
    }

    /// `Load` (`:87-92`): the saved pairs overwrite, the rest keep.
    pub fn load_pairs(&mut self, pairs: impl Iterator<Item = (String, i32)>) {
        for (id, count) in pairs {
            self.set_kill_count_directly(&id, count);
        }
    }
}

/// `CommonEnemyUICollectionInfoProvider.GetKillCountNeeded`
/// (`CommonEnemyUICollectionInfoProvider.cs:24-37`) narrowed to the port's
/// NPC family: `BannerSystem.NPCtoBanner(netId)` (`BannerSystem.cs:341-427`
/// cases) → `BannerToItem` (`:341-401` bands) →
/// `ItemID.Sets.KillsToBanner[item]` (`ItemID.cs:1303-1304` — the
/// `CreateIntSet` default 50 with the explicit overrides). `0` = the boss
/// entries have no banner; their providers are the quick-unlock kind.
/// `BannerSystem.NPCtoBanner` (`BannerSystem.cs:341+`) narrowed to the
/// port's family — the `(net id, banner id)` pairs (`BannerSystem.cs`: 1→69
/// at `:588`, -3→119 at `:881`, -4→151 at `:802`, -5→146 at `:788`, -6→90
/// at `:879`, -7→155 at `:885`, -8→159 at `:887`, -9→183 at `:889`, 2→86 at
/// `:467`, 3→87 at `:645`, 133→288 at `:1141`). `0` = no banner.
pub fn npc_to_banner(net_id: i32) -> i32 {
    const BANNERS: &[(i32, i32)] = &[
        (1, 69),
        (-3, 119),
        (-4, 151),
        (-5, 146),
        (-6, 90),
        (-7, 155),
        (-8, 159),
        (-9, 183),
        (2, 86),
        (3, 87),
        (133, 288),
    ];
    BANNERS
        .iter()
        .find(|(net, _)| *net == net_id)
        .map(|(_, banner)| *banner)
        .unwrap_or(0)
}

/// `NPC.BannerToNPC`'s family inverse — the net id a banner announces.
pub fn banner_to_npc(banner: i32) -> i32 {
    const PAIRS: &[(i32, i32)] = &[
        (69, 1),
        (119, -3),
        (151, -4),
        (146, -5),
        (90, -6),
        (155, -7),
        (159, -8),
        (183, -9),
        (86, 2),
        (87, 3),
        (288, 133),
    ];
    PAIRS
        .iter()
        .find(|(b, _)| *b == banner)
        .map(|(_, net)| *net)
        .unwrap_or(0)
}

pub fn kills_needed_for(net_id: i32) -> i32 {
    let banner = npc_to_banner(net_id);
    if banner <= 0 {
        return 0;
    }
    let item = banner_to_item(banner);
    // The `KillsToBanner` overrides the family hits (`ItemID.cs:1304`):
    // only Pinky's banner item 2960 carries an explicit 25.
    if item == 2960 {
        25
    } else {
        ItemID_Sets_DefaultKillsForBannerNeeded
    }
}

/// `ItemID.Sets.DefaultKillsForBannerNeeded` (`ItemID.cs:1303`).
pub const ItemID_Sets_DefaultKillsForBannerNeeded: i32 = 50;

/// `BannerSystem.BannerToItem` (`BannerSystem.cs:341-401`): the banded
/// +offset map. Only the bands the family's banners land in are kept.
pub fn banner_to_item(banner: i32) -> i32 {
    if banner >= 276 {
        return 4965 + banner - 276;
    }
    if banner >= 88 {
        return 2897 + banner - 88;
    }
    1615 + banner - 1
}

/// `ItemID.BannerEffect(1f)` sampled at the Classic difficulty — the
/// damage-dealt ×1.5 / damage-received ×0.75 pair every non-DD2 banner
/// carries (`ItemID.cs:11-31`: DamageDealt classic `1 + 1*0.5`, Damage
/// Received classic `(1 + 1/2)/2`; the expert arms 2.0/0.5 land with the
/// expert-mode toggle, M6-CREATIVE-1's difficulty flip).
pub const BANNER_DAMAGE_DEALT: f32 = 1.5;
pub const BANNER_DAMAGE_RECEIVED: f32 = 0.75;

/// `SceneMetrics.ScanTiles`' tile-91 arm (`SceneMetrics.cs:416-439`): the
/// banner id from the placed banner's frames — `frameX / 18` plus 111 per
/// full `frameY` row, the blank cells (`frameX < 396 && frameY < 54`, the
/// ids 311/312) skipped.
pub fn banner_num_from_frame(frame_x: i16, frame_y: i16) -> Option<i32> {
    let mut num = frame_x as i32 / 18;
    let mut fy = frame_y;
    while fy >= 54 {
        num += 90 + 21;
        fy -= 54;
    }
    if num >= 311 {
        num -= 1;
    }
    if num >= 312 {
        num -= 1;
    }
    let blank = frame_x < 396 && frame_y < 54;
    if blank || num == 311 || num == 312 {
        return None;
    }
    Some(num)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tier_ladder_is_the_half_and_fifth_steps() {
        // `GetUnlockStateByKillCount` (`:55-75`) with the default 50: 1 kill
        // = portrait, 10 = stats, 25 = drops, 50 = drops+rates.
        let full = 50;
        assert_eq!(
            get_unlock_state_by_kill_count(0, false, full),
            UnlockState::NotKnownAtAll
        );
        assert_eq!(
            get_unlock_state_by_kill_count(1, false, full),
            UnlockState::CanShowPortraitOnly
        );
        assert_eq!(
            get_unlock_state_by_kill_count(9, false, full),
            UnlockState::CanShowPortraitOnly
        );
        assert_eq!(
            get_unlock_state_by_kill_count(10, false, full),
            UnlockState::CanShowStats
        );
        assert_eq!(
            get_unlock_state_by_kill_count(24, false, full),
            UnlockState::CanShowStats
        );
        assert_eq!(
            get_unlock_state_by_kill_count(25, false, full),
            UnlockState::CanShowDropsWithoutDropRates
        );
        assert_eq!(
            get_unlock_state_by_kill_count(49, false, full),
            UnlockState::CanShowDropsWithoutDropRates
        );
        assert_eq!(
            get_unlock_state_by_kill_count(50, false, full),
            UnlockState::CanShowDropsWithDropRates
        );
        assert_eq!(
            get_unlock_state_by_kill_count(1000, false, full),
            UnlockState::CanShowDropsWithDropRates
        );
    }

    #[test]
    fn the_quick_unlock_skips_the_ladder() {
        // The boss providers pass quickUnlock: true — one kill jumps
        // straight to tier 4.
        assert_eq!(
            get_unlock_state_by_kill_count(1, true, 50),
            UnlockState::CanShowDropsWithDropRates
        );
        assert_eq!(
            get_unlock_state_by_kill_count(0, true, 50),
            UnlockState::NotKnownAtAll
        );
    }

    #[test]
    fn the_kill_tracker_caps_and_persists_pairs() {
        let mut tracker = KillTracker::new();
        for _ in 0..3 {
            tracker.register_kill("BlueSlime");
        }
        assert_eq!(tracker.get_kill_count("BlueSlime"), 3);
        assert_eq!(tracker.get_kill_count("Zombie"), 0);
        // The positive cap (`:51`).
        tracker.set_kill_count_directly("Pinky", POSITIVE_KILL_COUNT_CAP + 1);
        assert_eq!(tracker.get_kill_count("Pinky"), POSITIVE_KILL_COUNT_CAP);
        tracker.set_kill_count_directly("Pinky", -5);
        assert_eq!(tracker.get_kill_count("Pinky"), 0);
        // The save/load round-trip.
        let pairs: Vec<(String, i32)> = tracker
            .pairs()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        let mut loaded = KillTracker::new();
        loaded.load_pairs(pairs.into_iter());
        assert_eq!(loaded.get_kill_count("BlueSlime"), 3);
        assert_eq!(loaded.get_kill_count("Pinky"), 0);
        loaded.reset();
        assert_eq!(loaded.get_kill_count("BlueSlime"), 0);
    }

    #[test]
    fn the_family_banner_chain() {
        // BlueSlime: banner 69 → item 1683 → the default 50.
        assert_eq!(banner_to_item(69), 1683);
        assert_eq!(kills_needed_for(1), 50);
        // Pinky: banner 151 → item 2960 → the explicit 25.
        assert_eq!(banner_to_item(151), 2960);
        assert_eq!(kills_needed_for(-4), 25);
        // The rest of the family: the default.
        for net in [-3, -5, -6, -7, -8, -9, 2, 3, 133] {
            assert_eq!(kills_needed_for(net), 50, "net id {net}");
        }
        // The bosses have no banner — the entry's quick-unlock flag governs.
        assert_eq!(kills_needed_for(50), 0);
        // Unknown ids: no banner either.
        assert_eq!(kills_needed_for(17), 0);
    }
}
