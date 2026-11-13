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

/// `NPCKillsTracker.POSITIVE_KILL_COUNT_CAP` (`NPCKillsTracker.cs:14`).
pub const POSITIVE_KILL_COUNT_CAP: i32 = 999_999_999;

/// `NPCKillsTracker` (`NPCKillsTracker.cs:10-101`): the per-world
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

    /// `RegisterKill` (`NPCKillsTracker.cs:23-33`): increment (the count was clamped on write,
    /// so reading before incrementing cannot overflow the cap).
    pub fn register_kill(&mut self, credit_id: &str) {
        let next = self.get_kill_count(credit_id).saturating_add(1);
        self.set_kill_count_directly(credit_id, next);
    }

    /// `SetKillCountDirectly` (`NPCKillsTracker.cs:40-51`) — clamped to the positive cap
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

    /// `Load` (`:72-81`): the saved pairs overwrite, the rest keep.
    pub fn load_pairs(&mut self, pairs: impl Iterator<Item = (String, i32)>) {
        for (id, count) in pairs {
            self.set_kill_count_directly(&id, count);
        }
    }
}

/// `BestiaryUnlocksTracker` (`BestiaryUnlocksTracker.cs:7-11, 13-39`): the three
/// credit-id sub-trackers one world carries, as one in-memory value. The kills
/// keep their own [`KillTracker`] (its `pairs()` half is the `.trplayer`
/// sidecar's form too, `persist.rs:59`); the sightings and the chats are the
/// two credit-id sets Vanilla keeps beside them.
///
/// Vanilla persists this tracker in **world-file section 10** (1-based)
/// (`WorldFile.SaveBestiary:3396`), so the world-file halves live in
/// `wld/sections/bestiary.rs::{from_wld,to_wld}` — this module stays free of
/// the byte layout.
///
/// The port is **state + IO only** for the three halves (owner decision,
/// 2026-11-08): [`KillTracker::register_kill`] is wired, but nothing records a
/// sighting or a chat yet (`NPCWasNearPlayerTracker.ScanWorldForFinds:102-130`
/// and the chat hook are behaviour, not state), so a world that has never seen
/// an NPC carries an empty sightings/chats pair and reads it back empty. That
/// is the intended state until the recording lands — recorded for
/// `docs/ANALYSIS.md` in Phase 7's docs pass.
///
/// Iteration order: the C# collections are a `Dictionary` and two `HashSet`s
/// whose enumeration order is unspecified; the port stores sorted sets instead
/// (the same documented deviation [`KillTracker::pairs`] already carries), so
/// a save is deterministic and differs from a Vanilla file's ordering only in
/// order, never in content.
#[derive(Clone, Debug, Default)]
pub struct WorldBestiary {
    /// `NPCKillsTracker` (`NPCKillsTracker.cs:16`).
    pub kills: KillTracker,
    /// `NPCWasNearPlayerTracker._wasNearPlayer` (`NPCWasNearPlayerTracker.cs:15`)
    /// — the credit ids the player has seen near an NPC.
    pub sights: std::collections::BTreeSet<String>,
    /// `NPCWasChatWithTracker._chattedWithPlayer`
    /// (`NPCWasChatWithTracker.cs:14`) — the credit ids the player has talked to.
    pub chats: std::collections::BTreeSet<String>,
}

impl WorldBestiary {
    pub fn new() -> Self {
        Self::default()
    }

    /// `BestiaryUnlocksTracker.Reset` (`BestiaryUnlocksTracker.cs:34-39`): all
    /// three sub-trackers clear together.
    pub fn reset(&mut self) {
        self.kills.reset();
        self.sights.clear();
        self.chats.clear();
    }

    /// `NPCWasNearPlayerTracker.SetWasSeenDirectly`
    /// (`NPCWasNearPlayerTracker.cs:43-52`): a credit id already in the set
    /// stays once (the C#'s `HashSet.Add`), so the wiring phase may call this
    /// per frame without changing the state.
    pub fn set_was_seen_directly(&mut self, credit_id: &str) {
        self.sights.insert(credit_id.to_string());
    }

    /// `NPCWasNearPlayerTracker.GetWasNearbyBefore(string)`
    /// (`NPCWasNearPlayerTracker.cs:59-62`).
    pub fn was_seen_near_player(&self, credit_id: &str) -> bool {
        self.sights.contains(credit_id)
    }

    /// `NPCWasChatWithTracker.SetWasChatWithDirectly`
    /// (`NPCWasChatWithTracker.cs:32-41`).
    pub fn set_was_chat_with_directly(&mut self, credit_id: &str) {
        self.chats.insert(credit_id.to_string());
    }

    /// `NPCWasChatWithTracker.GetWasChatWith(string)`
    /// (`NPCWasChatWithTracker.cs:49-52`).
    pub fn was_chat_with(&self, credit_id: &str) -> bool {
        self.chats.contains(credit_id)
    }

    /// The sighted credit ids in the order `NPCWasNearPlayerTracker.Save`
    /// (`:64-74`, the write loop at `:69-72`) walks the set — the port's
    /// sorted order, not the C#'s unspecified one.
    pub fn sight_ids(&self) -> impl Iterator<Item = &str> {
        self.sights.iter().map(String::as_str)
    }

    /// The chatted-with credit ids in the same deterministic order
    /// (`NPCWasChatWithTracker.Save:54-64`, the write loop at `:59-62`).
    pub fn chat_ids(&self) -> impl Iterator<Item = &str> {
        self.chats.iter().map(String::as_str)
    }

    /// `NPCWasNearPlayerTracker.Load` (`:76-84`): the file's ids add, the rest
    /// keep (`HashSet.Add`).
    pub fn load_sights(&mut self, ids: impl Iterator<Item = String>) {
        self.sights.extend(ids);
    }

    /// `NPCWasChatWithTracker.Load` (`:66-74`).
    pub fn load_chats(&mut self, ids: impl Iterator<Item = String>) {
        self.chats.extend(ids);
    }
}

/// `BannerSystem.NPCtoBanner` (`BannerSystem.cs:403-1150`): the npc type /
/// BannerID to banner index map (0 when no banner).
pub fn npc_to_banner(net_id: i32) -> i32 {
    match net_id {
        -10 => 131,
        -9 => 183,
        -8 => 159,
        -7 => 155,
        -6 => 90,
        -5 | 16 => 146,
        -4 => 151,
        -3 => 119,
        -2 | 121 => 167,
        -1 | 81 => 99,
        1 | 302 | 333 | 334 | 335 | 336 => 69,
        2 | 190 | 191 | 192 | 193 | 194 | 317 | 318 => 25,
        3 | 132 | 186 | 187 | 188 | 189 | 200 | 319 | 320 | 321 | 331 | 332 | 430 | 432 | 433 | 434 | 435 | 436 | 590 | 591 | 632 | 691 => 87,
        6 => 27,
        7 | 8 | 9 => 104,
        10 | 11 | 12 | 95 | 96 | 97 => 84,
        21 | 201 | 202 | 203 | 322 | 323 | 324 | 449 | 450 | 451 | 452 => 67,
        23 => 55,
        24 | 25 => 50,
        26 => 40,
        27 => 38,
        28 => 42,
        29 | 30 => 39,
        31 | 294 | 295 | 296 => 247,
        32 | 33 => 68,
        34 => 102,
        39 | 40 | 41 => 13,
        42 | 231 | 232 | 233 | 234 | 235 => 47,
        43 => 54,
        44 => 178,
        45 | 665 => 177,
        46 | 303 | 337 | 540 => 14,
        47 => 18,
        48 => 44,
        49 => 7,
        51 => 130,
        52 => 106,
        53 => 176,
        55 | 230 => 43,
        56 => 168,
        57 => 19,
        58 => 61,
        59 => 135,
        60 => 45,
        61 => 79,
        62 | 66 => 24,
        63 => 51,
        64 => 243,
        65 => 66,
        67 => 20,
        69 => 4,
        71 => 107,
        73 => 41,
        74 | 297 | 298 => 8,
        75 => 63,
        77 => 6,
        78 => 57,
        79 => 245,
        80 => 246,
        82 => 85,
        83 => 23,
        84 => 28,
        85 | 629 => 16,
        86 => 77,
        87 | 88 | 89 | 90 | 91 | 92 => 86,
        93 => 114,
        94 | 112 => 100,
        98 | 99 | 100 => 83,
        101 => 96,
        102 => 1,
        103 => 244,
        104 => 81,
        109 | 378 => 17,
        110 => 164,
        111 => 118,
        120 => 15,
        122 => 37,
        133 => 288,
        137 => 128,
        138 => 129,
        140 => 153,
        141 => 75,
        143 => 170,
        144 => 145,
        145 => 169,
        147 => 126,
        148 | 149 => 150,
        150 => 124,
        151 => 134,
        152 => 116,
        153 => 74,
        154 => 248,
        155 => 82,
        156 => 242,
        157 => 5,
        158 | 159 => 78,
        161 | 431 => 29,
        162 => 34,
        163 | 238 => 9,
        164 | 165 => 71,
        166 => 73,
        167 => 179,
        168 => 98,
        169 => 48,
        170 | 171 | 180 => 60,
        172 => 160,
        173 => 21,
        174 => 46,
        175 => 88,
        176 => 287,
        177 => 26,
        179 => 22,
        181 => 30,
        182 => 31,
        183 => 101,
        184 => 171,
        185 => 70,
        195 | 196 => 80,
        197 => 89,
        198 | 199 => 53,
        204 => 172,
        205 => 56,
        206 => 49,
        212 => 62,
        213 => 239,
        214 => 238,
        215 => 240,
        216 => 237,
        217 => 97,
        218 => 103,
        219 => 133,
        220 => 250,
        221 => 174,
        223 => 64,
        224 => 32,
        225 => 76,
        226 => 33,
        236 | 237 => 52,
        239 | 240 => 12,
        241 => 10,
        242 => 11,
        243 => 125,
        244 => 157,
        250 => 2,
        251 => 111,
        252 => 59,
        253 => 65,
        254 | 255 => 72,
        256 => 36,
        257 => 3,
        258 => 58,
        259 | 260 | 261 => 35,
        268 => 127,
        269 | 270 | 271 | 272 => 161,
        273 | 274 | 275 | 276 => 91,
        277 | 278 | 279 | 280 => 121,
        281 | 282 => 156,
        283 | 284 => 147,
        285 | 286 => 105,
        287 => 95,
        288 => 108,
        289 => 115,
        290 => 149,
        291 => 166,
        292 => 175,
        293 => 165,
        301 => 158,
        304 => 123,
        305 | 306 | 307 | 308 | 309 | 310 | 311 | 312 | 313 | 314 => 162,
        315 => 120,
        316 => 113,
        326 => 173,
        329 => 122,
        330 => 152,
        338 | 339 | 340 => 185,
        341 => 154,
        342 => 117,
        343 => 184,
        347 => 110,
        348 | 349 => 148,
        350 => 109,
        351 => 132,
        352 => 112,
        379 => 92,
        380 => 180,
        381 => 136,
        382 => 142,
        383 | 384 => 141,
        385 => 140,
        386 => 138,
        387 => 144,
        388 => 137,
        389 => 139,
        390 => 143,
        391 => 163,
        402 | 403 | 404 => 217,
        405 => 221,
        406 => 220,
        407 | 408 => 218,
        409 | 410 => 219,
        411 => 216,
        412 | 413 | 414 => 224,
        415 | 516 => 226,
        416 | 518 => 225,
        417 => 223,
        418 => 222,
        419 => 227,
        420 => 230,
        421 => 229,
        423 => 231,
        424 => 228,
        425 => 236,
        426 => 233,
        427 => 234,
        428 => 232,
        429 => 235,
        438 => 93,
        460 => 196,
        461 => 191,
        462 => 190,
        463 => 199,
        464 => 281,
        465 => 282,
        466 => 197,
        467 => 198,
        468 => 192,
        469 => 195,
        470 => 283,
        471 | 472 => 186,
        473 => 284,
        474 => 285,
        475 => 286,
        477 | 478 | 479 => 193,
        480 => 201,
        481 => 202,
        482 => 204,
        483 => 203,
        489 => 205,
        490 => 206,
        494 | 495 => 189,
        496 | 497 => 188,
        498 | 499 | 500 | 501 | 502 | 503 | 504 | 505 | 506 => 187,
        508 | 580 => 210,
        509 | 581 => 209,
        510 | 511 | 512 => 208,
        513 | 514 | 515 => 207,
        520 => 241,
        524 | 525 | 526 | 527 => 211,
        528 | 529 => 212,
        530 | 531 => 215,
        532 => 214,
        533 => 213,
        536 => 267,
        537 => 249,
        541 => 251,
        542 => 252,
        543 => 253,
        544 => 254,
        545 => 255,
        546 => 256,
        552 | 553 | 554 => 258,
        555 | 556 | 557 => 257,
        558 | 559 | 560 => 264,
        561 | 562 | 563 => 265,
        566 | 567 => 259,
        568 | 569 => 263,
        570 | 571 => 260,
        572 | 573 => 262,
        574 | 575 => 261,
        578 => 266,
        582 => 280,
        586 => 268,
        587 => 269,
        618 => 273,
        619 => 270,
        620 => 272,
        621 | 622 | 623 => 271,
        624 => 275,
        628 => 274,
        630 => 277,
        631 => 276,
        634 => 279,
        635 => 278,
        676 => 289,
        692 => 290,
        693 => 291,
        694 => 292,
        _ => 0,
    }
}

/// `CommonEnemyUICollectionInfoProvider.GetKillCountNeeded`
/// (`CommonEnemyUICollectionInfoProvider.cs:21-34`): the full kill count is
/// `ItemID.Sets.KillsToBanner[BannerToItem(NPCtoBanner(BannerID()))]`, with
/// `ItemID.Sets.DefaultKillsForBannerNeeded` (50, `ItemID.cs:1303`) for every
/// banner item that has no override (`ItemID.cs:1304`). The `quickUnlock` flag
/// is checked at the entry/unlock-state level, not by zeroing the count.
pub fn kills_needed_for(net_id: i32) -> i32 {
    let banner = npc_to_banner(net_id);
    let item = banner_to_item(banner);
    kills_to_banner_item(item)
}

/// `ItemID.Sets.DefaultKillsForBannerNeeded` (`ItemID.cs:1303`).
pub const ItemID_Sets_DefaultKillsForBannerNeeded: i32 = 50;

/// `ItemID.Sets.KillsToBanner` (`ItemID.cs:1304`): the per-banner-item kill
/// count, defaulting to `ItemID_Sets_DefaultKillsForBannerNeeded` (50). The
/// override pairs are the `CreateIntSet(default, item, count, ...)` arguments.
pub fn kills_to_banner_item(item: i32) -> i32 {
    match item {
        3838 => 1000,
        3845 => 200,
        3837 => 500,
        3844 => 200,
        3839 => 150,
        3840 => 100,
        3842 => 200,
        3841 => 100,
        2971 => 150,
        2982 => 150,
        2931 => 100,
        2961 => 100,
        2994 => 100,
        2985 => 10,
        4541 => 10,
        2969 => 10,
        2986 => 10,
        2915 => 10,
        4602 => 10,
        4542 => 25,
        4543 => 25,
        4546 => 25,
        4545 => 25,
        2901 => 25,
        2902 => 25,
        1631 => 25,
        2913 => 25,
        4688 => 25,
        3390 => 25,
        4973 => 25,
        4974 => 25,
        4975 => 25,
        2934 => 25,
        1670 => 25,
        1694 => 25,
        2958 => 25,
        2960 => 25,
        3441 => 25,
        3780 => 25,
        3397 => 25,
        3403 => 25,
        _ => ItemID_Sets_DefaultKillsForBannerNeeded,
    }
}

/// `BannerSystem.BannerToItem` (`BannerSystem.cs:341-401`): the banner-id to
/// placeable banner item map.
pub fn banner_to_item(banner: i32) -> i32 {
    if banner == 292 {
        return 5673;
    }
    if banner == 291 {
        return 5672;
    }
    if banner == 290 {
        return 5651;
    }
    if banner == 289 {
        return 5352;
    }
    if banner >= 276 {
        return 4965 + banner - 276;
    }
    if banner >= 274 {
        return 4687 + banner - 274;
    }
    if banner == 273 {
        return 4602;
    }
    if banner >= 267 {
        return 4541 + banner - 267;
    }
    if banner >= 257 {
        return 3837 + banner - 257;
    }
    if banner >= 252 {
        return 3794 + banner - 252;
    }
    if banner >= 244 {
        return 3398 + banner - 244;
    }
    if banner >= 241 {
        return 3390 + banner - 241;
    }
    if banner >= 88 {
        return 2897 + banner - 88;
    }
    if banner >= 24 {
        return 1683 + banner - 24;
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
        let pairs: Vec<(String, i32)> = tracker.pairs().map(|(k, v)| (k.to_string(), v)).collect();
        let mut loaded = KillTracker::new();
        loaded.load_pairs(pairs.into_iter());
        assert_eq!(loaded.get_kill_count("BlueSlime"), 3);
        assert_eq!(loaded.get_kill_count("Pinky"), 0);
        loaded.reset();
        assert_eq!(loaded.get_kill_count("BlueSlime"), 0);
    }

    #[test]
    fn the_world_bestiary_holds_the_three_credit_id_trackers() {
        // `BestiaryUnlocksTracker`'s three halves: the kills keep their
        // `KillTracker`, the sightings and chats are the credit-id sets.
        let mut b = WorldBestiary::new();
        b.kills.register_kill("BlueSlime");
        b.kills.register_kill("Pinky");
        assert_eq!(b.kills.pairs().count(), 2);
        assert!(!b.was_seen_near_player("BlueSlime"));
        assert!(!b.was_chat_with("BlueSlime"));
        b.set_was_seen_directly("BlueSlime");
        b.set_was_chat_with_directly("BlueSlime");
        assert!(b.was_seen_near_player("BlueSlime"));
        assert!(b.was_chat_with("BlueSlime"));
        // The set semantics: a second registration of the same id is a no-op
        // (`HashSet.Add`), an unknown id reads false.
        b.set_was_seen_directly("BlueSlime");
        assert_eq!(b.sight_ids().count(), 1);
        assert!(!b.was_seen_near_player("Zombie"));
        assert!(!b.was_chat_with("Zombie"));
    }

    #[test]
    fn the_world_bestiary_resets_all_three_trackers() {
        // `BestiaryUnlocksTracker.Reset:34-39`.
        let mut b = WorldBestiary::new();
        b.kills.set_kill_count_directly("BlueSlime", 7);
        b.set_was_seen_directly("GreenSlime");
        b.set_was_chat_with_directly("Pinky");
        b.reset();
        assert_eq!(b.kills.get_kill_count("BlueSlime"), 0);
        assert_eq!(b.sight_ids().count(), 0);
        assert_eq!(b.chat_ids().count(), 0);
        assert_eq!(b.kills.pairs().count(), 0);
    }

    #[test]
    fn the_world_bestiary_iterates_deterministically() {
        // The save order is the sorted order (the C# collection order is
        // unspecified), so a save is reproducible byte for byte.
        let mut b = WorldBestiary::new();
        for id in ["Pinky", "BlueSlime", "GreenSlime"] {
            b.kills.set_kill_count_directly(id, 1);
            b.set_was_seen_directly(id);
            b.set_was_chat_with_directly(id);
        }
        let kills: Vec<(String, i32)> = b.kills.pairs().map(|(k, v)| (k.into(), v)).collect();
        let sights: Vec<&str> = b.sight_ids().collect();
        let chats: Vec<&str> = b.chat_ids().collect();
        assert_eq!(
            kills.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>(),
            ["BlueSlime", "GreenSlime", "Pinky"],
            "sorted, whatever the insertion order was"
        );
        assert_eq!(sights, ["BlueSlime", "GreenSlime", "Pinky"]);
        assert_eq!(chats, ["BlueSlime", "GreenSlime", "Pinky"]);
        // …and the load halves add rather than replace.
        b.load_sights(["Zombie".to_string()].into_iter());
        b.load_chats(["Zombie".to_string()].into_iter());
        assert!(b.was_seen_near_player("Zombie") && b.was_chat_with("Zombie"));
        assert_eq!(b.sight_ids().count(), 4);
        assert_eq!(
            b.kills.pairs().count(),
            3,
            "the kills load half is separate"
        );
    }

    #[test]
    fn the_family_banner_chain() {
        // BlueSlime: banner 69 -> item 1728 -> the default 50.
        // `BannerSystem.BannerToItem` (`BannerSystem.cs:341-401`, the `>= 24`
        // band `1683 + banner - 24`).
        assert_eq!(banner_to_item(69), 1728);
        assert_eq!(kills_needed_for(1), 50);
        // Pinky: banner 151 -> item 2960 -> the explicit 25
        // (`ItemID.Sets.KillsToBanner` override, `ItemID.cs:1304`).
        assert_eq!(banner_to_item(151), 2960);
        assert_eq!(kills_needed_for(-4), 25);
        // The rest of the family: the default.
        for net in [-3, -5, -6, -7, -8, -9, 2, 3, 133] {
            assert_eq!(kills_needed_for(net), 50, "net id {net}");
        }
        // No banner -> `BannerToItem(0)` = 1614, whose `KillsToBanner` entry is
        // the default 50 (`CommonEnemyUICollectionInfoProvider.GetKillCountNeeded`
        // `:21-34`, `BannerSystem.cs:341-401`, `ItemID.cs:1303-1304`). Quick-unlock
        // is a separate flag, not a zero full-kill count.
        assert_eq!(banner_to_item(0), 1614);
        assert_eq!(kills_needed_for(50), 50);
        assert_eq!(kills_needed_for(17), 50);
    }

    #[test]
    fn kills_needed_for_honors_kills_to_banner_overrides() {
        // `ItemID.Sets.KillsToBanner` (`ItemID.cs:1304`) pairs item ids with a
        // non-default kill count; every other banner item keeps
        // `DefaultKillsForBannerNeeded` (50, `ItemID.cs:1303`). The full count
        // comes from `CommonEnemyUICollectionInfoProvider.GetKillCountNeeded`
        // (`CommonEnemyUICollectionInfoProvider.cs:21-34`):
        // `BannerToItem(NPCtoBanner(BannerID()))` -> `KillsToBanner[item]`.
        assert_eq!(kills_needed_for(45), 10, "Tim banner 177 -> item 2986 -> 10");
        assert_eq!(
            kills_needed_for(552),
            1000,
            "DD2GoblinT1 banner 257 -> item 3837 -> 1000"
        );
        assert_eq!(
            kills_needed_for(520),
            25,
            "MartianWalker banner 205 -> item 3014 -> 25"
        );
        // The unknown persistent-id / absent net-id fallback is the default 50
        // (`GetKillCountNeeded:23-31`).
        assert_eq!(kills_needed_for(999_999), 50);
    }

}
