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

/// `BannerSystem.NPCtoBanner` (`BannerSystem.cs:403-1154`): the npc net id
/// to banner index map (`0` = the npc drops no banner).
pub fn npc_to_banner(net_id: i32) -> i32 {
    match net_id {
        -10 => 131,
        -9 => 183,
        -8 => 159,
        -7 => 155,
        -6 => 90,
        -5 => 146,
        -4 => 151,
        -3 => 119,
        -2 => 167,
        -1 => 99,
        1 => 69,
        2 => 25,
        3 => 87,
        6 => 27,
        7 => 104,
        8 => 104,
        9 => 104,
        10 => 84,
        11 => 84,
        12 => 84,
        16 => 146,
        21 => 67,
        23 => 55,
        24 => 50,
        25 => 50,
        26 => 40,
        27 => 38,
        28 => 42,
        29 => 39,
        30 => 39,
        31 => 247,
        32 => 68,
        33 => 68,
        34 => 102,
        39 => 13,
        40 => 13,
        41 => 13,
        42 => 47,
        43 => 54,
        44 => 178,
        45 => 177,
        46 => 14,
        47 => 18,
        48 => 44,
        49 => 7,
        51 => 130,
        52 => 106,
        53 => 176,
        55 => 43,
        56 => 168,
        57 => 19,
        58 => 61,
        59 => 135,
        60 => 45,
        61 => 79,
        62 => 24,
        63 => 51,
        64 => 243,
        65 => 66,
        66 => 24,
        67 => 20,
        69 => 4,
        71 => 107,
        73 => 41,
        74 => 8,
        75 => 63,
        77 => 6,
        78 => 57,
        79 => 245,
        80 => 246,
        81 => 99,
        82 => 85,
        83 => 23,
        84 => 28,
        85 => 16,
        86 => 77,
        87 => 86,
        88 => 86,
        89 => 86,
        90 => 86,
        91 => 86,
        92 => 86,
        93 => 114,
        94 => 100,
        95 => 84,
        96 => 84,
        97 => 84,
        98 => 83,
        99 => 83,
        100 => 83,
        101 => 96,
        102 => 1,
        103 => 244,
        104 => 81,
        109 => 17,
        110 => 164,
        111 => 118,
        112 => 100,
        120 => 15,
        121 => 167,
        122 => 37,
        132 => 87,
        133 => 288,
        137 => 128,
        138 => 129,
        140 => 153,
        141 => 75,
        143 => 170,
        144 => 145,
        145 => 169,
        147 => 126,
        148 => 150,
        149 => 150,
        150 => 124,
        151 => 134,
        152 => 116,
        153 => 74,
        154 => 248,
        155 => 82,
        156 => 242,
        157 => 5,
        158 => 78,
        159 => 78,
        161 => 29,
        162 => 34,
        163 => 9,
        164 => 71,
        165 => 71,
        166 => 73,
        167 => 179,
        168 => 98,
        169 => 48,
        170 => 60,
        171 => 60,
        172 => 160,
        173 => 21,
        174 => 46,
        175 => 88,
        176 => 287,
        177 => 26,
        179 => 22,
        180 => 60,
        181 => 30,
        182 => 31,
        183 => 101,
        184 => 171,
        185 => 70,
        186 => 87,
        187 => 87,
        188 => 87,
        189 => 87,
        190 => 25,
        191 => 25,
        192 => 25,
        193 => 25,
        194 => 25,
        195 => 80,
        196 => 80,
        197 => 89,
        198 => 53,
        199 => 53,
        200 => 87,
        201 => 67,
        202 => 67,
        203 => 67,
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
        230 => 43,
        231 => 47,
        232 => 47,
        233 => 47,
        234 => 47,
        235 => 47,
        236 => 52,
        237 => 52,
        238 => 9,
        239 => 12,
        240 => 12,
        241 => 10,
        242 => 11,
        243 => 125,
        244 => 157,
        250 => 2,
        251 => 111,
        252 => 59,
        253 => 65,
        254 => 72,
        255 => 72,
        256 => 36,
        257 => 3,
        258 => 58,
        259 => 35,
        260 => 35,
        261 => 35,
        268 => 127,
        269 => 161,
        270 => 161,
        271 => 161,
        272 => 161,
        273 => 91,
        274 => 91,
        275 => 91,
        276 => 91,
        277 => 121,
        278 => 121,
        279 => 121,
        280 => 121,
        281 => 156,
        282 => 156,
        283 => 147,
        284 => 147,
        285 => 105,
        286 => 105,
        287 => 95,
        288 => 108,
        289 => 115,
        290 => 149,
        291 => 166,
        292 => 175,
        293 => 165,
        294 => 247,
        295 => 247,
        296 => 247,
        297 => 8,
        298 => 8,
        301 => 158,
        302 => 69,
        303 => 14,
        304 => 123,
        305 => 162,
        306 => 162,
        307 => 162,
        308 => 162,
        309 => 162,
        310 => 162,
        311 => 162,
        312 => 162,
        313 => 162,
        314 => 162,
        315 => 120,
        316 => 113,
        317 => 25,
        318 => 25,
        319 => 87,
        320 => 87,
        321 => 87,
        322 => 67,
        323 => 67,
        324 => 67,
        326 => 173,
        329 => 122,
        330 => 152,
        331 => 87,
        332 => 87,
        333 => 69,
        334 => 69,
        335 => 69,
        336 => 69,
        337 => 14,
        338 => 185,
        339 => 185,
        340 => 185,
        341 => 154,
        342 => 117,
        343 => 184,
        347 => 110,
        348 => 148,
        349 => 148,
        350 => 109,
        351 => 132,
        352 => 112,
        378 => 17,
        379 => 92,
        380 => 180,
        381 => 136,
        382 => 142,
        383 => 141,
        384 => 141,
        385 => 140,
        386 => 138,
        387 => 144,
        388 => 137,
        389 => 139,
        390 => 143,
        391 => 163,
        402 => 217,
        403 => 217,
        404 => 217,
        405 => 221,
        406 => 220,
        407 => 218,
        408 => 218,
        409 => 219,
        410 => 219,
        411 => 216,
        412 => 224,
        413 => 224,
        414 => 224,
        415 => 226,
        416 => 225,
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
        430 => 87,
        431 => 29,
        432 => 87,
        433 => 87,
        434 => 87,
        435 => 87,
        436 => 87,
        438 => 93,
        449 => 67,
        450 => 67,
        451 => 67,
        452 => 67,
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
        471 => 186,
        472 => 186,
        473 => 284,
        474 => 285,
        475 => 286,
        477 => 193,
        478 => 193,
        479 => 193,
        480 => 201,
        481 => 202,
        482 => 204,
        483 => 203,
        489 => 205,
        490 => 206,
        494 => 189,
        495 => 189,
        496 => 188,
        497 => 188,
        498 => 187,
        499 => 187,
        500 => 187,
        501 => 187,
        502 => 187,
        503 => 187,
        504 => 187,
        505 => 187,
        506 => 187,
        508 => 210,
        509 => 209,
        510 => 208,
        511 => 208,
        512 => 208,
        513 => 207,
        514 => 207,
        515 => 207,
        516 => 226,
        518 => 225,
        520 => 241,
        524 => 211,
        525 => 211,
        526 => 211,
        527 => 211,
        528 => 212,
        529 => 212,
        530 => 215,
        531 => 215,
        532 => 214,
        533 => 213,
        536 => 267,
        537 => 249,
        540 => 14,
        541 => 251,
        542 => 252,
        543 => 253,
        544 => 254,
        545 => 255,
        546 => 256,
        552 => 258,
        553 => 258,
        554 => 258,
        555 => 257,
        556 => 257,
        557 => 257,
        558 => 264,
        559 => 264,
        560 => 264,
        561 => 265,
        562 => 265,
        563 => 265,
        566 => 259,
        567 => 259,
        568 => 263,
        569 => 263,
        570 => 260,
        571 => 260,
        572 => 262,
        573 => 262,
        574 => 261,
        575 => 261,
        578 => 266,
        580 => 210,
        581 => 209,
        582 => 280,
        586 => 268,
        587 => 269,
        590 => 87,
        591 => 87,
        618 => 273,
        619 => 270,
        620 => 272,
        621 => 271,
        622 => 271,
        623 => 271,
        624 => 275,
        628 => 274,
        629 => 16,
        630 => 277,
        631 => 276,
        632 => 87,
        634 => 279,
        635 => 278,
        665 => 177,
        676 => 289,
        691 => 87,
        692 => 290,
        693 => 291,
        694 => 292,
        _ => 0,
    }
}

/// `BannerSystem.BannerToNPC` (`BannerSystem.cs:1156-1449`): the banner
/// index to the npc it announces (`0` = no npc).
pub fn banner_to_npc(banner: i32) -> i32 {
    match banner {
        1 => 102,
        2 => 250,
        3 => 257,
        4 => 69,
        5 => 157,
        6 => 77,
        7 => 49,
        8 => 74,
        9 => 163,
        10 => 241,
        11 => 242,
        12 => 239,
        13 => 39,
        14 => 46,
        15 => 120,
        16 => 85,
        17 => 109,
        18 => 47,
        19 => 57,
        20 => 67,
        21 => 173,
        22 => 179,
        23 => 83,
        24 => 62,
        25 => 2,
        26 => 177,
        27 => 6,
        28 => 84,
        29 => 161,
        30 => 181,
        31 => 182,
        32 => 224,
        33 => 226,
        34 => 162,
        35 => 259,
        36 => 256,
        37 => 122,
        38 => 27,
        39 => 29,
        40 => 26,
        41 => 73,
        42 => 28,
        43 => 55,
        44 => 48,
        45 => 60,
        46 => 174,
        47 => 42,
        48 => 169,
        49 => 206,
        50 => 24,
        51 => 63,
        52 => 236,
        53 => 199,
        54 => 43,
        55 => 23,
        56 => 205,
        57 => 78,
        58 => 258,
        59 => 252,
        60 => 170,
        61 => 58,
        62 => 212,
        63 => 75,
        64 => 223,
        65 => 253,
        66 => 65,
        67 => 21,
        68 => 32,
        69 => 1,
        70 => 185,
        71 => 164,
        72 => 254,
        73 => 166,
        74 => 153,
        75 => 141,
        76 => 225,
        77 => 86,
        78 => 158,
        79 => 61,
        80 => 196,
        81 => 104,
        82 => 155,
        83 => 98,
        84 => 10,
        85 => 82,
        86 => 87,
        87 => 3,
        88 => 175,
        89 => 197,
        90 => -6,
        91 => 273,
        92 => 379,
        93 => 438,
        95 => 287,
        96 => 101,
        97 => 217,
        98 => 168,
        99 => 81,
        100 => 94,
        101 => 183,
        102 => 34,
        103 => 218,
        104 => 7,
        105 => 285,
        106 => 52,
        107 => 71,
        108 => 288,
        109 => 350,
        110 => 347,
        111 => 251,
        112 => 352,
        113 => 316,
        114 => 93,
        115 => 289,
        116 => 152,
        117 => 342,
        118 => 111,
        119 => -3,
        120 => 315,
        121 => 277,
        122 => 329,
        123 => 304,
        124 => 150,
        125 => 243,
        126 => 147,
        127 => 268,
        128 => 137,
        129 => 138,
        130 => 51,
        131 => -10,
        132 => 351,
        133 => 219,
        134 => 151,
        135 => 59,
        136 => 381,
        137 => 388,
        138 => 386,
        139 => 389,
        140 => 385,
        141 => 383,
        142 => 382,
        143 => 390,
        144 => 387,
        145 => 144,
        146 => 16,
        147 => 283,
        148 => 348,
        149 => 290,
        150 => 148,
        151 => -4,
        152 => 330,
        153 => 140,
        154 => 341,
        155 => -7,
        156 => 281,
        157 => 244,
        158 => 301,
        159 => -8,
        160 => 172,
        161 => 269,
        162 => 305,
        163 => 391,
        164 => 110,
        165 => 293,
        166 => 291,
        167 => 121,
        168 => 56,
        169 => 145,
        170 => 143,
        171 => 184,
        172 => 204,
        173 => 326,
        174 => 221,
        175 => 292,
        176 => 53,
        177 => 45,
        178 => 44,
        179 => 167,
        180 => 380,
        183 => -9,
        184 => 343,
        185 => 338,
        186 => 471,
        187 => 498,
        188 => 496,
        189 => 494,
        190 => 462,
        191 => 461,
        192 => 468,
        193 => 477,
        195 => 469,
        196 => 460,
        197 => 466,
        198 => 467,
        199 => 463,
        201 => 480,
        202 => 481,
        203 => 483,
        204 => 482,
        205 => 489,
        206 => 490,
        207 => 513,
        208 => 510,
        209 => 581,
        210 => 580,
        211 => 524,
        212 => 529,
        213 => 533,
        214 => 532,
        215 => 530,
        216 => 411,
        217 => 402,
        218 => 407,
        219 => 409,
        220 => 406,
        221 => 405,
        222 => 418,
        223 => 417,
        224 => 412,
        225 => 416,
        226 => 415,
        227 => 419,
        228 => 424,
        229 => 421,
        230 => 420,
        231 => 423,
        232 => 428,
        233 => 426,
        234 => 427,
        235 => 429,
        236 => 425,
        237 => 216,
        238 => 214,
        239 => 213,
        240 => 215,
        241 => 520,
        242 => 156,
        243 => 64,
        244 => 103,
        245 => 79,
        246 => 80,
        247 => 31,
        248 => 154,
        249 => 537,
        250 => 220,
        251 => 541,
        252 => 542,
        253 => 543,
        254 => 544,
        255 => 545,
        256 => 546,
        257 => 555,
        258 => 552,
        259 => 566,
        260 => 570,
        261 => 574,
        262 => 572,
        263 => 568,
        264 => 558,
        265 => 561,
        266 => 578,
        267 => 536,
        268 => 586,
        269 => 587,
        270 => 619,
        271 => 621,
        272 => 620,
        273 => 618,
        274 => 628,
        275 => 624,
        276 => 631,
        277 => 630,
        278 => 635,
        279 => 634,
        280 => 582,
        281 => 464,
        282 => 465,
        283 => 470,
        284 => 473,
        285 => 474,
        286 => 475,
        287 => 176,
        288 => 133,
        289 => 676,
        290 => 692,
        291 => 693,
        292 => 694,
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

/// `ItemID.Sets.KillsToBanner` (`ItemID.cs:1304`): the kills a banner
/// item needs before it drops, with
/// `ItemID.Sets.DefaultKillsForBannerNeeded` (50, `ItemID.cs:1303`) for every
/// item the set does not name.
pub fn kills_to_banner_item(item: i32) -> i32 {
    match item {
        1631 => 25,
        1670 => 25,
        1694 => 25,
        2901 => 25,
        2902 => 25,
        2913 => 25,
        2915 => 10,
        2931 => 100,
        2934 => 25,
        2958 => 25,
        2960 => 25,
        2961 => 100,
        2969 => 10,
        2971 => 150,
        2982 => 150,
        2985 => 10,
        2986 => 10,
        2994 => 100,
        3390 => 25,
        3397 => 25,
        3403 => 25,
        3441 => 25,
        3780 => 25,
        3837 => 500,
        3838 => 1000,
        3839 => 150,
        3840 => 100,
        3841 => 100,
        3842 => 200,
        3843 => 50,
        3844 => 200,
        3845 => 200,
        3846 => 50,
        4541 => 10,
        4542 => 25,
        4543 => 25,
        4545 => 25,
        4546 => 25,
        4602 => 10,
        4688 => 25,
        4973 => 25,
        4974 => 25,
        4975 => 25,
        _ => ItemID_Sets_DefaultKillsForBannerNeeded,
    }
}

/// `BannerSystem.BannerToItem` (`BannerSystem.cs:341-401`): the banner index
/// to its banner item id (the banded `+offset` map).
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
        return 3789 + banner - 252;
    }
    if banner == 251 {
        return 3780;
    }
    if banner >= 249 {
        return 3593 + banner - 249;
    }
    if banner >= 186 {
        return 3390 + banner - 186;
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
        // BlueSlime: net 1 -> banner 69 -> item 1683 -> the default 50.
        // `BannerSystem.NPCtoBanner` (`BannerSystem.cs:403-1154`) and
        // `BannerSystem.BannerToItem` (`BannerSystem.cs:341-401`, the `>= 88`
        // band `2897 + banner - 88`).
        assert_eq!(npc_to_banner(1), 69);
        assert_eq!(banner_to_item(69), 1683);
        assert_eq!(kills_needed_for(1), 50);
        // Pinky: net -4 -> banner 151 -> item 2960 -> the explicit 25
        // (`ItemID.Sets.KillsToBanner` override, `ItemID.cs:1304`).
        assert_eq!(npc_to_banner(-4), 151);
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
        assert_eq!(npc_to_banner(50), 0);
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
            "DD2GoblinT1 banner 258 -> item 3838 -> 1000"
        );
        assert_eq!(
            kills_needed_for(555),
            500,
            "DD2GoblinBomberT1 banner 257 -> item 3837 -> 500"
        );
        assert_eq!(
            kills_needed_for(561),
            200,
            "DD2JavelinstT1 banner 265 -> item 3845 -> 200"
        );
        assert_eq!(
            kills_needed_for(566),
            150,
            "DD2SkeletonT1 banner 259 -> item 3839 -> 150"
        );
        assert_eq!(
            kills_needed_for(216),
            25,
            "PirateCaptain banner 237 -> item 3441 -> 25"
        );
        // The unknown persistent-id / absent net-id fallback is the default 50
        // (`GetKillCountNeeded:23-31`).
        assert_eq!(kills_needed_for(999_999), 50);
    }

    #[test]
    fn the_banner_chain_covers_the_whole_c_sharp_table() {
        // `BannerSystem.NPCtoBanner` (`BannerSystem.cs:403-1154`) maps every
        // npc the vanilla gives a banner; the port's table is the C# switch
        // transcribed in full, so the family and the DD2 rows agree.
        assert_eq!(npc_to_banner(1), 69);
        assert_eq!(npc_to_banner(2), 25);
        assert_eq!(npc_to_banner(3), 87);
        assert_eq!(npc_to_banner(7), 104);
        assert_eq!(npc_to_banner(45), 177);
        assert_eq!(npc_to_banner(133), 288);
        assert_eq!(npc_to_banner(471), 186);
        assert_eq!(npc_to_banner(694), 292);
        // An npc the vanilla never banners: `0`.
        assert_eq!(npc_to_banner(50), 0);
        assert_eq!(npc_to_banner(999_999), 0);
        // `BannerSystem.BannerToNPC` (`BannerSystem.cs:1156-1449`) is the
        // banner -> npc direction of the same table.
        assert_eq!(banner_to_npc(69), 1);
        assert_eq!(banner_to_npc(151), -4);
        assert_eq!(banner_to_npc(288), 133);
        assert_eq!(banner_to_npc(292), 694);
        assert_eq!(banner_to_npc(999), 0);
    }

}
