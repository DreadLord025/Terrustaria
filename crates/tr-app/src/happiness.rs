//! The town-NPC happiness and the shop prices — `ShopHelper.cs` +
//! `GameContent/Personalities/*`. The vanilla computes the price multiplier
//! at the TALK open (`Player.SetTalkNPC` → `ShopHelper.GetShoppingSettings`,
//! `Player.cs:4383-4390`): the mood factors (the loveStruck buff, the
//! homelessness, the distance from home, the player's evil biomes, the
//! crowding) and the personality traits (the biome/neighbor affection)
//! multiply together, clamped to 0.75..1.5 and rounded to the cents
//! (banker's). batch-L merge (research/68 §2-4 + research/240/241): f37's
//! faithful `process_mood` core + g58's full NPCID-keyed DATA tables, the
//! report builder and the `ShoppingSettings` latch; one capture, two
//! consumers (`shop_mood` mirror + `shopping_settings`), zero re-scans.

use crate::npc::NpcKind;
use tr_core::World;

use crate::world_sim::scene_scan::scan_scene_at;

/// `AffectionLevel` (`GameContent/Personalities/AffectionLevel.cs`) — the
/// affection multipliers are `ShopHelper`'s constants (`:35-42`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AffectionLevel {
    Love,
    Like,
    Dislike,
    Hate,
}

impl AffectionLevel {
    /// The per-level price factor (`likeValue` 0.94, `loveValue` 0.88,
    /// `dislikeValue` 1.06, `hateValue` 1.12 — `ShopHelper.cs:35-42`).
    pub fn multiplier(self) -> f32 {
        match self {
            AffectionLevel::Love => 0.88,
            AffectionLevel::Like => 0.94,
            AffectionLevel::Dislike => 1.06,
            AffectionLevel::Hate => 1.12,
        }
    }

    /// The enum's numeric value (`AffectionLevel.cs:5-8`: Love 100, Like
    /// 50, Dislike -50, Hate -100) — `BiomePreferenceListTrait.
    /// ModifyShopPrice` (`:44`) picks the HIGHEST among the biomes the
    /// judge is in (`biomePreference.Affection < biomePreference2.Affection`),
    /// so the ordering, not the price factor, drives the pick.
    pub fn affection_value(self) -> i32 {
        match self {
            AffectionLevel::Love => 100,
            AffectionLevel::Like => 50,
            AffectionLevel::Dislike => -50,
            AffectionLevel::Hate => -100,
        }
    }
}

/// The shopping biomes (`GameContent/Personalities/*Biome.cs`) — the
/// report/localization keys ride [`biome_name_key`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BiomeKind {
    Forest,
    Ocean,
    Snow,
    Desert,
    Jungle,
    Underground,
    Hallow,
    Mushroom,
    Corruption,
    Crimson,
    Dungeon,
}

/// One biome affection row (`BiomePreferenceListTrait`'s entries).
#[derive(Clone, Copy, Debug)]
pub struct BiomePreference {
    pub level: AffectionLevel,
    pub biome: BiomeKind,
}

/// The row shorthand (`Populate_BiomePreferences`' `{ level, biome }`
/// initializer pairs — `PersonalityDatabasePopulator.cs:27-147`).
#[allow(dead_code)]
const fn bp(level: AffectionLevel, biome: BiomeKind) -> BiomePreference {
    BiomePreference { level, biome }
}
use AffectionLevel::{Dislike as D, Hate as H, Like as L, Love as V};
use BiomeKind::{Desert, Forest, Hallow, Jungle, Mushroom, Ocean, Snow, Underground};

/// The per-NPC biome preferences — the FULL vanilla table
/// (`PersonalityDatabasePopulator.Populate_BiomePreferences`,
/// `PersonalityDatabasePopulator.cs:27-147`), keyed on the C# `NPCID` so the
/// unported kinds (142 Santa, 160 Truffle, 178 Steampunker, 209 Cyborg,
/// 229 Pirate, 550 Tavernkeep — the W4-g58 DATA half) carry their rows too.
/// Row order = the C# initializer order (`BiomePreferenceListTrait.
/// ModifyShopPrice` `:41-48` picks the HIGHEST affection among the biomes
/// the judge is in — the first-match convention is wrong).
pub fn biome_preferences_by_id(id: u16) -> &'static [BiomePreference] {
    match id {
        22 => &[BiomePreference { level: L, biome: Forest }, BiomePreference { level: D, biome: Ocean }],      // `:27-31` the Guide
        17 => &[BiomePreference { level: L, biome: Forest }, BiomePreference { level: D, biome: Desert }],     // `:32-36` the Merchant
        588 => &[BiomePreference { level: L, biome: Forest }, BiomePreference { level: D, biome: Underground }], // `:37-41` the Golfer
        633 => &[BiomePreference { level: L, biome: Forest }, BiomePreference { level: D, biome: Desert }],    // `:42-46` the Zoologist
        441 => &[BiomePreference { level: L, biome: Snow }, BiomePreference { level: D, biome: Hallow }],      // `:47-51` the Tax Collector
        124 => &[BiomePreference { level: L, biome: Snow }, BiomePreference { level: D, biome: Underground }], // `:52-56` the Mechanic
        209 => &[BiomePreference { level: L, biome: Snow }, BiomePreference { level: D, biome: Jungle }],      // `:57-61` the Cyborg
        142 => &[BiomePreference { level: V, biome: Snow }, BiomePreference { level: H, biome: Desert }],      // `:62-66` Santa Claus
        207 => &[BiomePreference { level: L, biome: Desert }, BiomePreference { level: D, biome: Forest }],    // `:67-71` the Dye Trader
        19 => &[BiomePreference { level: L, biome: Desert }, BiomePreference { level: D, biome: Snow }],       // `:72-76` the Arms Dealer
        178 => &[BiomePreference { level: L, biome: Desert }, BiomePreference { level: D, biome: Jungle }],    // `:77-81` the Steampunker
        20 => &[BiomePreference { level: L, biome: Jungle }, BiomePreference { level: D, biome: Desert }],     // `:82-86` the Dryad
        228 => &[BiomePreference { level: L, biome: Jungle }, BiomePreference { level: D, biome: Hallow }],    // `:87-91` the Witch Doctor
        227 => &[BiomePreference { level: L, biome: Jungle }, BiomePreference { level: D, biome: Forest }],    // `:92-96` the Painter
        369 => &[BiomePreference { level: L, biome: Ocean }, BiomePreference { level: D, biome: Desert }],     // `:97-101` the Angler
        229 => &[BiomePreference { level: L, biome: Ocean }, BiomePreference { level: D, biome: Underground }], // `:102-106` the Pirate
        353 => &[BiomePreference { level: L, biome: Ocean }, BiomePreference { level: D, biome: Snow }],       // `:107-111` the Stylist
        38 => &[BiomePreference { level: L, biome: Underground }, BiomePreference { level: D, biome: Ocean }], // `:112-116` the Demolitionist
        107 => &[BiomePreference { level: L, biome: Underground }, BiomePreference { level: D, biome: Jungle }], // `:117-121` the Tinkerer
        54 => &[BiomePreference { level: L, biome: Underground }, BiomePreference { level: D, biome: Hallow }], // `:122-126` the Clothier
        108 => &[BiomePreference { level: L, biome: Hallow }, BiomePreference { level: D, biome: Ocean }],     // `:127-131` the Wizard
        18 => &[BiomePreference { level: L, biome: Hallow }, BiomePreference { level: D, biome: Snow }],       // `:132-136` the Nurse
        208 => &[BiomePreference { level: L, biome: Hallow }, BiomePreference { level: D, biome: Underground }], // `:137-141` the Party Girl
        550 => &[BiomePreference { level: L, biome: Hallow }, BiomePreference { level: D, biome: Snow }],      // `:142-146` the Tavernkeep
        160 => &[BiomePreference { level: L, biome: Mushroom }],                 // `:147` the Truffle
        // The Princess (663) has NO `Register` row — her affection rides the
        // `AllPersonalitiesModifier` random-love picks alone
        // (`AllPersonalitiesModifier.cs:15-36`).
        _ => &[],
    }
}

/// The kind-keyed view (`NpcKind.cs_id` — the M4-TOWN callers/tests).
pub fn biome_preferences(kind: NpcKind) -> &'static [BiomePreference] {
    biome_preferences_by_id(kind.cs_id())
}

/// The `TownNPCMoodBiomes.{NameKey}` localization key
/// (`AShoppingBiome.NameKey` — `OceanBiome.cs:7` … `DungeonBiome.cs:7`;
/// the Underground row's key is `NormalUnderground`, NOT `Underground`).
pub fn biome_name_key(biome: BiomeKind) -> &'static str {
    match biome {
        BiomeKind::Forest => "Forest",
        BiomeKind::Ocean => "Ocean",
        BiomeKind::Snow => "Snow",
        BiomeKind::Desert => "Desert",
        BiomeKind::Jungle => "Jungle",
        BiomeKind::Underground => "NormalUnderground",
        BiomeKind::Hallow => "Hallow",
        BiomeKind::Mushroom => "Mushroom",
        BiomeKind::Corruption => "Corruption",
        BiomeKind::Crimson => "Crimson",
        BiomeKind::Dungeon => "Dungeon",
    }
}

/// The neighbor-relationship rows — the FULL vanilla switch
/// (`AllPersonalitiesModifier.ModifyShopPrice_Relationships`,
/// `AllPersonalitiesModifier.cs:41-505`), keyed on the C# `NPCID`; row
/// order = the C# `if` order (the report text prepends, so the LAST row
/// reads first). The Princess's random three-loves head (`:15-36`) and
/// the universal LikePrincess row (`:37-40`) ride `process_mood` itself.
pub fn neighbor_preferences_by_id(id: u16) -> &'static [(AffectionLevel, u16)] {
    match id {
        // `:43-60` the Merchant: Golfer/Nurse like, TaxCollector dislike,
        // Angler hate.
        17 => &[(L, 588), (L, 18), (D, 441), (H, 369)],
        // `:61-82` the Nurse: ArmsDealer love, Wizard like, PartyGirl +
        // Dryad dislike, Zoologist hate.
        18 => &[(V, 19), (L, 108), (D, 208), (D, 20), (H, 633)],
        // `:83-100` the Painter: Dryad love, PartyGirl like, Cyborg +
        // Truffle dislike.
        227 => &[(V, 20), (L, 208), (D, 209), (D, 160)],
        // `:101-118` the Dye Trader: ArmsDealer/Painter like, Steampunker
        // dislike, Pirate hate.
        207 => &[(L, 19), (L, 227), (D, 178), (H, 229)],
        // `:119-140` the Party Girl: Wizard love, Stylist like, Merchant
        // dislike, TaxCollector hate, Zoologist love (the trailing row).
        208 => &[(V, 108), (L, 353), (D, 17), (H, 441), (V, 633)],
        // `:141-158` the Angler: PartyGirl/Demolitionist/TaxCollector
        // like, Tavernkeep hate.
        369 => &[(L, 208), (L, 38), (L, 441), (H, 550)],
        // `:159-176` the Stylist: DyeTrader love, Pirate like, Tavernkeep
        // dislike, Tinkerer hate.
        353 => &[(V, 207), (L, 229), (D, 550), (H, 107)],
        // `:177-194` the Demolitionist: Tavernkeep love, Mechanic like,
        // Tinkerer + ArmsDealer dislike.
        38 => &[(V, 550), (L, 124), (D, 107), (D, 19)],
        // `:195-212` the Dryad: WitchDoctor/Truffle like, Angler dislike,
        // Golfer hate.
        20 => &[(L, 228), (L, 160), (D, 369), (H, 588)],
        // `:213-230` the Tavernkeep: Demolitionist love, Tinkerer like,
        // Guide dislike, DyeTrader hate.
        550 => &[(V, 38), (L, 107), (D, 22), (H, 207)],
        // `:231-248` the Arms Dealer: Nurse love, Steampunker like,
        // Golfer dislike, Demolitionist hate.
        19 => &[(V, 18), (L, 178), (D, 588), (H, 38)],
        // `:249-266` the Tinkerer: Mechanic love, DyeTrader like,
        // Clothier dislike, Stylist hate.
        107 => &[(V, 124), (L, 207), (D, 54), (H, 353)],
        // `:267-284` the Witch Doctor: Dryad/Guide like, Nurse dislike,
        // Truffle hate.
        228 => &[(L, 20), (L, 22), (D, 18), (H, 160)],
        // `:285-302` the Clothier: Truffle love, TaxCollector like, Nurse
        // dislike, Mechanic hate.
        54 => &[(V, 160), (L, 441), (D, 18), (H, 124)],
        // `:303-320` the Mechanic: Tinkerer love, Cyborg like, ArmsDealer
        // dislike, Clothier hate.
        124 => &[(V, 107), (L, 209), (D, 19), (H, 54)],
        // `:321-342` the Tax Collector: Merchant love, PartyGirl like,
        // Demolitionist + Mechanic dislike, Santa hate.
        441 => &[(V, 17), (L, 208), (D, 38), (D, 124), (H, 142)],
        // `:343-360` the Pirate: Angler love, Tavernkeep like, Stylist
        // dislike, Guide hate.
        229 => &[(V, 369), (L, 550), (D, 353), (H, 22)],
        // `:361-378` the Wizard: Golfer love, Merchant like, WitchDoctor
        // dislike, Cyborg hate.
        108 => &[(V, 588), (L, 17), (D, 228), (H, 209)],
        // `:379-400` the Steampunker: Cyborg love, Painter like,
        // PartyGirl + Wizard + Dryad dislike.
        178 => &[(V, 209), (L, 227), (D, 208), (D, 108), (D, 20)],
        // `:401-422` the Cyborg: Stylist/Pirate/Steampunker like, Wizard
        // hate, Zoologist dislike (the trailing row).
        209 => &[(L, 353), (L, 229), (L, 178), (H, 108), (D, 633)],
        // `:423-428` Santa Claus: TaxCollector hate alone.
        142 => &[(H, 441)],
        // `:429-450` the Golfer: Painter like, Angler love, Merchant
        // hate, Pirate dislike, Zoologist like (the trailing row).
        588 => &[(L, 227), (V, 369), (H, 17), (D, 229), (L, 633)],
        // `:451-468` the Guide: Clothier like, Steampunker dislike,
        // Painter hate, Zoologist like (the trailing row).
        22 => &[(L, 54), (D, 178), (H, 227), (L, 633)],
        // `:469-486` the Truffle: Guide love, Dryad like, Clothier
        // dislike, WitchDoctor hate.
        160 => &[(V, 22), (L, 20), (D, 54), (H, 228)],
        // `:487-504` the Zoologist: Angler dislike, ArmsDealer hate,
        // WitchDoctor love, Golfer like.
        633 => &[(D, 369), (H, 19), (V, 228), (L, 588)],
        _ => &[],
    }
}

/// The kind-keyed view — batch-L keep-one (research/68 §3 item 4): a
/// DIRECT wrapper over the id table returning the VERBATIM rows (the
/// unported-neighbor ids stay — they never match until the kind lands,
/// and the Princess's whole-roster picks need them). g58's
/// `OnceLock`/`Box::leak` filtered kind view DROPPED; the payload is f37's
/// `i32` vanilla id (the merged `MoodFlag::NpcAffection` contract).
pub fn neighbor_preferences(kind: NpcKind) -> &'static [(AffectionLevel, i32)] {
    use std::sync::OnceLock;
    static ROWS: OnceLock<Vec<&'static [(AffectionLevel, i32)]>> = OnceLock::new();
    // The full C# switch-key set (the 25 cases) — the id-keyed rows widen
    // to i32 once at first read (the &'static contract).
    static IDS: &[u16] = &[
        17, 18, 227, 207, 208, 369, 353, 38, 20, 550, 19, 107, 228, 54, 124, 441, 229, 108,
        178, 209, 142, 588, 22, 160, 633,
    ];
    let rows = ROWS.get_or_init(|| {
        IDS.iter()
            .map(|id| {
                let v: Vec<(AffectionLevel, i32)> = neighbor_preferences_by_id(*id)
                    .iter()
                    .map(|(lv, oid)| (*lv, *oid as i32))
                    .collect();
                &*Box::leak(v.into_boxed_slice())
            })
            .collect()
    });
    let id = kind.cs_id();
    match IDS.iter().position(|o| *o == id) {
        Some(i) => rows[i],
        None => &[],
    }
}

/// The resolved mood flags — the report-text keys ride
/// [`happiness_report`] (`AddHappinessReportText`'s categories; the trait
/// rows carry the affection payload for the `LikeBiome`/`LikeNPC` text
/// families). batch-L merged enum (research/68 §2/§3): f37's i32-carrying
/// `NpcAffection` + g58's `HateBiomeNamed`/`PrincessLove` report payloads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoodFlag {
    NoHome,
    FarFromHome,
    /// The name-less legacy marker — kept for the price-side callers/tests;
    /// the mood pass emits [`MoodFlag::HateBiomeNamed`] instead.
    HateBiomeEvil,
    /// `IsPlayerInEvilBiomes`' named row (`ShopHelper.cs:353-356`) — the
    /// evil biome carries its `BiomeName` substitute (g58's report half;
    /// f37's name-less push replaced per research/68 §3 item 7).
    HateBiomeNamed(BiomeKind),
    DislikeCrowded,
    HateCrowded,
    LoveSpace,
    /// The Princess's loneliness sentinel (`ShopHelper.cs:126-130`): fewer
    /// than 2 within-house AND fewer than 2 within-village neighbors →
    /// `HateLonely` + the 1000 sentinel.
    HateLonely,
    Content,
    BiomeAffection(AffectionLevel, BiomeKind),
    /// The neighbor affection carries the VANILLA id (f37's contract) —
    /// the rows reference unported kinds too (Steampunker 178, Cyborg
    /// 209, …); the report resolves the name through `lang_tables::NPC_ID`.
    NpcAffection(AffectionLevel, i32),
    /// The Princess's random three-loves head (`AllPersonalitiesModifier.cs:
    /// 15-36`): each pick is a Love ×0.88 whose report key is the dedicated
    /// `LoveNPC_{NPCID.Search.GetName(npcType)}` (`ShopHelper.LoveNPCByTypeName`,
    /// `:243-250`), NOT the plain `LoveNPC`. The i32 vanilla id carries the
    /// name (the merged contract — g58's NpcKind payload widens).
    PrincessLove(i32),
}

/// The mood outcome — the multiplier rides the caller's shop math, the
/// flags feed the report UI.
#[derive(Clone, Debug)]
pub struct Mood {
    pub multiplier: f32,
    pub flags: Vec<MoodFlag>,
}

/// `ShopHelper.GetNearbyResidentNPCs`'s housingCategory gate
/// (`ShopHelper.cs:305` + `TownRoomManager.CanNPCsLiveWithEachOther_ShopHelper`,
/// `TownRoomManager.cs:153-161`): the C# admits a crowd neighbor when
/// `!CanNPCsLiveWithEachOther(npc, nPC)` — CanLive compares the two
/// `housingCategory` fields for INEQUALITY (`:155`), so the NEGATED gate
/// counts only SAME-category neighbors: the town pets (category 1,
/// `NPC.SetDefaults` `NPC.cs:17224`/`:17308`/`:17494`/`:17607`) crowd each
/// other, never the humans (category 0), and vice versa. The port's
/// `town::can_npcs_live_with_each_other` carries the same comparison (the
/// housing lane's port); the g10 pet kinds are LIVE on this tree, so the
/// filter is armed (g58's latch-round half folded into the f37 core).
fn crowd_neighbor_counts(judge_net_id: i32, other_net_id: i32) -> bool {
    !crate::town::can_npcs_live_with_each_other(judge_net_id, other_net_id)
}

/// `ShopHelper.ProcessMood` (`ShopHelper.cs:95-174`) + the personality
/// traits (`BiomePreferenceListTrait.cs:38-53`, `AllPersonalitiesModifier.cs:
/// 12-506`) — the f37 faithful core (research/68 §2 KEEP): the loveStruck
/// head BEFORE the exclusions, the remix return, the 37/368/453 exclusion
/// set, the Tavernkeep-counting crowd roster + the housingCategory filter,
/// the Princess RNG loves + HateLonely, the ungated traits, the player-zone
/// judge + single best-affection row, the banker's-cents tail. `rng` is the
/// bare `Main.rand` stand-in (`app.rngs.global()`) — the ONLY draw is the
/// Princess's up-to-3 random loves (`:31`). The biome traits judge the
/// PLAYER's zones (`AShoppingBiome.IsInBiome(info.player)`), not the NPC's
/// home scene.
pub fn process_mood(
    world: &World,
    all_npcs: &[crate::npc::Npc],
    npc: &crate::npc::Npc,
    player_center: (f32, f32),
    raining: bool,
    rng: &mut tr_core::UnifiedRandom,
) -> Mood {
    let mut flags: Vec<MoodFlag> = Vec::new();
    // `:97-98`: the reset.
    let mut adjustment = 1.0f32;
    // `:99-102`: the loveStruck ×0.9 — BEFORE the exclusions, so even the
    // travelling sellers keep it. `NPC.loveStruck` is buff 119's flag
    // (`NPC.cs:92057-92060`); the port derives it from the live buff slots.
    if npc.buffs.has_buff(119) {
        adjustment *= 0.9;
    }
    // `:103-106`: the exclusions — the remix world, the two travelling
    // sellers (368/453), the town pets and `IsNotReallyTownNPC` (37/368/453,
    // `:363-371`) return before any mood work, with the UNROUNDED adjustment
    // (the `LimitAndRoundMultiplier` tail never runs on this path).
    let net_id = npc.kind.net_id();
    if world.seeds.contains(tr_core::seeds::SeedFlags::REMIX)
        || matches!(net_id, 37 | 368 | 453)
        || npc.kind.is_town_pet()
    {
        return Mood {
            multiplier: adjustment,
            flags,
        };
    }

    // The home anchor (`:293-297`): the homeless judge from their centre.
    let (home_x, home_y, homeless) = match npc.home {
        Some((hx, hy)) => (hx as f32, hy as f32, false),
        None => (
            (npc.x + npc.size().0 * 0.5) / 16.0,
            (npc.y + npc.size().1 * 0.5) / 16.0,
            true,
        ),
    };

    // `:107-114`: the homelessness and the 120-tile distance both push the
    // sentinel 1000 (the clamp floor at `:178` lands it on the 1.5
    // ceiling — the vanilla's "crazy" prices ARE the ceiling).
    if homeless {
        flags.push(MoodFlag::NoHome);
        adjustment = 1000.0;
    } else {
        let (cx, cy) = npc.center();
        let dist = ((home_x - cx / 16.0).powi(2) + (home_y - cy / 16.0).powi(2)).sqrt();
        if dist > 120.0 {
            flags.push(MoodFlag::FarFromHome);
            adjustment = 1000.0;
        }
    }

    // The player's scene — scanned once for BOTH the evil-biome sentinel and
    // the biome traits (the C# reads the live `player.Zone*` flags).
    let pm = scan_scene_at(
        world,
        [player_center.0, player_center.1],
        raining,
        false,
        false,
    );

    // `:115-118`: the player in the evil biomes (`:28-34`'s three, in the
    // array order Corruption → Crimson → Dungeon) pushes the sentinel too.
    // g58's named row (research/68 §3 item 7): the first hit names the
    // report's `BiomeName` substitute (`:353-356`).
    if adjustment < 1000.0 && (pm.zone_corrupt || pm.zone_crimson || pm.zone_dungeon) {
        let evil = if pm.zone_corrupt {
            BiomeKind::Corruption
        } else if pm.zone_crimson {
            BiomeKind::Crimson
        } else {
            BiomeKind::Dungeon
        };
        flags.push(MoodFlag::HateBiomeNamed(evil));
        adjustment = 1000.0;
    }

    // `GetNearbyResidentNPCs` (`:288-325`): every active town NPC except the
    // not-really-town trio and the different-housing-category roommates
    // (`TownRoomManager.cs:153-161` — `housingCategory` differs; the g10
    // pets are category 1, the humans category 0). Within 25 tiles of the
    // home anchor = house; 25..120 = village.
    let mut within_house = 0u32;
    let mut within_village = 0u32;
    let mut nearby_ids: Vec<i32> = Vec::new();
    for other in all_npcs {
        if other.hp <= 0 || std::ptr::eq(other, npc) {
            continue;
        }
        // `nPC.townNPC` (`:305`) — the port's town-like set plus the
        // Tavernkeep (550, `NPC.cs`'s `townNPC = true` arm; the shared
        // `is_town_npc_like` omits him, the mood scan must not).
        if !other.kind.is_town_npc_like() && other.kind != NpcKind::DD2Bartender {
            continue;
        }
        let oid = other.kind.net_id();
        if matches!(oid, 37 | 368 | 453) {
            continue; // IsNotReallyTownNPC (`:363-371`)
        }
        // `:305`'s last gate — `!WorldGen.TownManager.
        // CanNPCsLiveWithEachOther_ShopHelper(npc, nPC)`
        // (`TownRoomManager.cs:153-161`): CanLive = the housingCategories
        // DIFFER, so the mood pass admits only SAME-category neighbors
        // (pets count toward pets, humans toward humans).
        if !crowd_neighbor_counts(net_id, oid) {
            continue;
        }
        let (ox, oy) = match other.home {
            Some((hx, hy)) => (hx as f32, hy as f32),
            None => (
                (other.x + other.size().0 * 0.5) / 16.0,
                (other.y + other.size().1 * 0.5) / 16.0,
            ),
        };
        let dist = ((home_x - ox).powi(2) + (home_y - oy).powi(2)).sqrt();
        if dist < 25.0 {
            nearby_ids.push(oid);
            within_house += 1;
        } else if dist < 120.0 {
            within_village += 1;
        }
    }
    // `:120-131`: the Princess (663) — `flag = false` kills her crowd
    // multiply AND her LoveSpace; alone (< 2 house AND < 2 village) she
    // hates the loneliness at the sentinel.
    let princess = net_id == 663;
    let crowd_step = if princess { 1.0 } else { 1.05 };
    if princess && within_house < 2 && within_village < 2 {
        flags.push(MoodFlag::HateLonely);
        adjustment = 1000.0;
    }
    // `:132-146`: the crowd — each resident past the third multiplies (the
    // sentinel does NOT stop it; only the final clamp tames it).
    if within_house > 3 {
        for _ in 3..within_house {
            adjustment *= crowd_step;
        }
        flags.push(if within_house > 6 {
            MoodFlag::HateCrowded
        } else {
            MoodFlag::DislikeCrowded
        });
    }
    // `:147-151`: LoveSpace — `flag && npcsWithinHouse <= 2 &&
    // npcsWithinVillage < 4` (the Princess's flag is false).
    if !princess && within_house <= 2 && within_village < 4 {
        flags.push(MoodFlag::LoveSpace);
        adjustment *= 0.95;
    }

    // `:152-168`: the personality traits — ALWAYS applied (no sentinel
    // gate); the clamp at the tail tames whatever they multiply into 1000.
    // The biome trait (`BiomePreferenceListTrait.ModifyShopPrice`,
    // `:38-53`): among the rows whose biome the PLAYER is in, the single
    // HIGHEST-affection row wins (`AffectionLevel` Love 100 > Like 50 >
    // Dislike -50 > Hate -100, `AffectionLevel.cs:5-8`) — one multiply, not
    // one per matching row.
    let zones = shopping_zones(&pm, player_center, world);
    let mut best: Option<&BiomePreference> = None;
    for pref in biome_preferences(npc.kind) {
        if biome_in(&zones, pref.biome)
            && best.is_none_or(|b| b.level.affection_value() < pref.level.affection_value())
        {
            best = Some(pref);
        }
    }
    if let Some(pref) = best {
        flags.push(MoodFlag::BiomeAffection(pref.level, pref.biome));
        adjustment *= pref.level.multiplier();
    }
    // `AllPersonalitiesModifier` (`:12-506`): the Princess's random loves
    // first (`:15-36` — up to 3 draws off the nearby-type list, each removing
    // its pick; the report key is the dedicated `LoveNPC_{name}`,
    // `LoveNPCByTypeName` `:243-250` — the `PrincessLove` flag), then
    // everyone-but-her likes a nearby Princess (`:37-40`, the dedicated
    // `LikeNPC_Princess` key), then the per-kind switch rows (`:41-505`) in
    // source order.
    if princess {
        let mut picks: Vec<i32> = nearby_ids.clone();
        picks.sort_unstable(); // the bool[NPCID.Count] scan is id order
        picks.dedup();
        for _ in 0..3 {
            if picks.is_empty() {
                break;
            }
            let index = rng.next_max(picks.len() as i32) as usize;
            let id = picks.remove(index);
            flags.push(MoodFlag::PrincessLove(id));
            adjustment *= AffectionLevel::Love.multiplier();
        }
    } else if nearby_ids.contains(&663) {
        flags.push(MoodFlag::NpcAffection(AffectionLevel::Like, 663));
        adjustment *= AffectionLevel::Like.multiplier();
    }
    for (level, id) in neighbor_preferences(npc.kind) {
        if nearby_ids.contains(id) {
            flags.push(MoodFlag::NpcAffection(*level, *id));
            adjustment *= level.multiplier();
        }
    }

    if flags.is_empty() {
        flags.push(MoodFlag::Content);
    }
    // `LimitAndRoundMultiplier` (`:176-181`): the clamp 0.75..1.5, then
    // `Math.Round(x * 100) / 100` — the BANKER'S round (the shop.rs helper).
    let clamped = adjustment.clamp(0.75, 1.5);
    let multiplier = crate::shop::round_half_to_even(clamped * 100.0) as f32 / 100.0;
    Mood { multiplier, flags }
}

/// The player's shopping-zone snapshot — the ten `AShoppingBiome.IsInBiome`
/// inputs. Eight are plain `player.Zone*` flags; the Forest and Underground
/// rows ride the special getters (`Player.cs:3823-3847`):
/// `ShoppingZone_BelowSurface = Center tile Y > Main.worldSurface`,
/// `ShoppingZone_Forest = !AnyBiome && !BelowSurface` where `AnyBiome` is
/// any of the eight named zones (Desert included).
struct ShoppingZones {
    corrupt: bool,
    crimson: bool,
    dungeon: bool,
    glowshroom: bool,
    hallow: bool,
    jungle: bool,
    snow: bool,
    beach: bool,
    desert: bool,
    forest: bool,
    below_surface: bool,
}

fn shopping_zones(
    pm: &tr_core::scene::SceneMetrics,
    player_center: (f32, f32),
    world: &World,
) -> ShoppingZones {
    let z = pm.zones();
    let any_biome = z.dungeon
        || z.corrupt
        || z.crimson
        || z.glowshroom
        || z.hallow
        || z.jungle
        || z.snow
        || pm.zone_beach
        || z.desert;
    // `(int)Center.Y >> 4` (`Utils.cs:1809`) vs `Main.worldSurface` (double).
    let tile_y = (player_center.1 as i32) >> 4;
    let below_surface = f64::from(tile_y) > world.world_surface;
    ShoppingZones {
        corrupt: z.corrupt,
        crimson: z.crimson,
        dungeon: z.dungeon,
        glowshroom: z.glowshroom,
        hallow: z.hallow,
        jungle: z.jungle,
        snow: z.snow,
        beach: pm.zone_beach,
        desert: z.desert,
        forest: !any_biome && !below_surface,
        below_surface,
    }
}

/// One `AShoppingBiome.IsInBiome` row over the snapshot (the ten
/// `GameContent/Personalities/*Biome.cs` classes).
fn biome_in(z: &ShoppingZones, biome: BiomeKind) -> bool {
    match biome {
        BiomeKind::Forest => z.forest,
        BiomeKind::Ocean => z.beach,
        BiomeKind::Snow => z.snow,
        BiomeKind::Desert => z.desert,
        BiomeKind::Jungle => z.jungle,
        BiomeKind::Underground => z.below_surface,
        BiomeKind::Hallow => z.hallow,
        BiomeKind::Mushroom => z.glowshroom,
        BiomeKind::Corruption => z.corrupt,
        BiomeKind::Crimson => z.crimson,
        BiomeKind::Dungeon => z.dungeon,
    }
}

/// `ShoppingSettings` (`ShoppingSettings.cs:3-14`) — the pair
/// `Player.SetTalkNPC` latches (`Player.cs:4383-4390`): `NotInShop` is
/// `{ 1f, "" }`.
#[derive(Clone, Debug)]
pub struct ShoppingSettings {
    pub price_adjustment: f32,
    pub happiness_report: String,
}

impl Default for ShoppingSettings {
    /// `ShoppingSettings.NotInShop` (`ShoppingSettings.cs:9-13`).
    fn default() -> Self {
        ShoppingSettings {
            price_adjustment: 1.0,
            happiness_report: String::new(),
        }
    }
}

/// `NPC.GetFullnameByID` (`NPC.cs:7010-7020`): the first live NPC of the
/// type answers `FullName` (the given name through `Game.NPCTitle`,
/// `NPC.cs:6980-6990`); no resident → `Lang.GetNPCNameValue`.
pub fn npc_full_name(all_npcs: &[crate::npc::Npc], kind: NpcKind) -> String {
    if let Some(n) = all_npcs.iter().find(|n| n.hp > 0 && n.kind == kind) {
        let type_name = crate::lang::npc_name(kind.cs_id() as i32);
        if n.given_name.is_empty() {
            return type_name; // `TypeName` (`NPC.cs:6978`)
        }
        // `Language.GetTextValue("Game.NPCTitle", given, typeName)`
        // (`NPC.cs:6988`) — the "{0} the {1}" row.
        return crate::lang::text_args("Game.NPCTitle", &[n.given_name.clone(), type_name]);
    }
    crate::lang::npc_name(kind.cs_id() as i32)
}

/// The i32-id twin (the merged `NpcAffection`/`PrincessLove` payload): a
/// ported id resolves through `kind_from_net_id`; an unported id answers
/// the plain `Lang.GetNPCNameValue` type name (the no-resident arm).
fn npc_full_name_by_id(all_npcs: &[crate::npc::Npc], id: i32) -> String {
    match crate::npc::kind_from_net_id(id) {
        Some((kind, _)) => npc_full_name(all_npcs, kind),
        None => crate::lang::npc_name(id),
    }
}

/// `NPC.CanShowHomelessText` (`NPC.cs:94828-94841`): the homeless NPC
/// that is not the Old Man (37) / Travelling Merchant (368) / Skeleton
/// Merchant (453). The port's homeless stand-in is `home == None` (the
/// `Npc.homeless` field); the exclusion set rides the vanilla net ids.
pub fn can_show_homeless_text(npc: Option<&crate::npc::Npc>) -> bool {
    match npc {
        Some(n) => n.home.is_none() && !matches!(n.kind.net_id(), 37 | 368 | 453),
        None => false,
    }
}

/// `NPC.ShouldBestiaryGirlBeLycantrope` (`NPC.cs:93753-93764`): the
/// Zoologist's shapeshift form — a blood-moon night, or a full-moon
/// (`moonPhase == 0`) night. `UpdateAltTexture` (`NPC.cs:90225-90227`)
/// writes `altTexture = 2` exactly on this predicate, so the report's
/// `Transformed` suffix (`ShopHelper.cs:191-194`) reads it directly — no
/// alt-texture field needed. The `npc_chat.rs` Zoologist arm inlines the
/// same gate (`:822-826`); this is the shared derivation.
pub fn should_bestiary_girl_be_lycantrope(blood_moon: bool, day_time: bool, moon_phase: u8) -> bool {
    if !blood_moon || day_time {
        moon_phase == 0 && !day_time
    } else {
        true
    }
}

/// `ShopHelper.AddHappinessReportText` (`ShopHelper.cs:188-196`): the
/// category is `TownNPCMood_{NPCID.Search.GetName(netID)}`, plus the
/// `Transformed` suffix when the talked NPC is the Zoologist (633) in her
/// shapeshift form (`:191-194`: `type == 633 && altTexture == 2` — the
/// caller derives the form through [`should_bestiary_girl_be_lycantrope`],
/// the predicate `UpdateAltTexture` writes `altTexture = 2` on,
/// `NPC.cs:90225-90227`); each row PREPENDS
/// `Language.GetTextValueWith(category.key, substitutes) + " "`.
fn prepend_report(
    report: &mut String,
    kind: NpcKind,
    transformed: bool,
    key: &str,
    args: &[(&str, String)],
) {
    let Some(name) = crate::lang_tables::NPC_ID
        .iter()
        .find(|(id, _)| *id == kind.cs_id() as i32)
        .map(|(_, n)| *n)
    else {
        return;
    };
    // `:191-194` — the Zoologist's shapeshift category suffix.
    let suffix = if kind == NpcKind::Zoologist && transformed {
        "Transformed"
    } else {
        ""
    };
    let subs: Vec<(&str, tr_core::localization::Substitution)> = args
        .iter()
        .map(|(k, v)| (*k, tr_core::localization::Substitution::Text(v.clone())))
        .collect();
    let line = crate::lang::text_format_with(&format!("TownNPCMood_{name}{suffix}.{key}"), &subs);
    // `string.Concat(str1: text, str0: report, str2: " ")` — text FIRST.
    *report = format!("{line}{report} ");
}

/// `ShopHelper.BiomeNameByKey` (`ShopHelper.cs:183-186`).
fn biome_name(key: &str) -> String {
    crate::lang::text(&format!("TownNPCMoodBiomes.{key}"))
}

/// The mood flags → the report string (`ShopHelper.ProcessMood`'s
/// `AddHappinessReportText` call order — the flags carry the order the
/// factors landed, and every call PREPENDS). The evil-biome row names the
/// biome (`IsPlayerInEvilBiomes` `:353-356` passes `BiomeName`); the
/// LikePrincess row uses the `LikeNPC_Princess` key
/// (`AllPersonalitiesModifier.cs:37-40` → `ShopHelper.cs:252-259`); the
/// Princess's picks use `LoveNPC_{name}` (`LoveNPCByTypeName` `:243-250`).
/// batch-L: the i32-id payloads resolve through `lang_tables::NPC_ID` /
/// `kind_from_net_id` (research/68 §4's report-builder adaptation).
pub fn happiness_report(
    kind: NpcKind,
    transformed: bool,
    flags: &[MoodFlag],
    all_npcs: &[crate::npc::Npc],
) -> String {
    let mut report = String::new();
    for flag in flags {
        match flag {
            MoodFlag::NoHome => prepend_report(&mut report, kind, transformed, "NoHome", &[]),
            MoodFlag::FarFromHome => {
                prepend_report(&mut report, kind, transformed, "FarFromHome", &[])
            }
            MoodFlag::HateBiomeEvil => {
                // The name-less legacy marker — no BiomeName substitute
                // (the mood pass emits `HateBiomeNamed` instead).
                prepend_report(&mut report, kind, transformed, "HateBiome", &[]);
            }
            MoodFlag::HateBiomeNamed(biome) => prepend_report(
                &mut report,
                kind,
                transformed,
                "HateBiome",
                &[("BiomeName", biome_name(biome_name_key(*biome)))],
            ),
            MoodFlag::DislikeCrowded => {
                prepend_report(&mut report, kind, transformed, "DislikeCrowded", &[])
            }
            MoodFlag::HateCrowded => {
                prepend_report(&mut report, kind, transformed, "HateCrowded", &[])
            }
            MoodFlag::LoveSpace => prepend_report(&mut report, kind, transformed, "LoveSpace", &[]),
            MoodFlag::HateLonely => prepend_report(&mut report, kind, transformed, "HateLonely", &[]),
            MoodFlag::Content => prepend_report(&mut report, kind, transformed, "Content", &[]),
            MoodFlag::BiomeAffection(level, biome) => {
                let key = match level {
                    AffectionLevel::Love => "LoveBiome",
                    AffectionLevel::Like => "LikeBiome",
                    AffectionLevel::Dislike => "DislikeBiome",
                    AffectionLevel::Hate => "HateBiome",
                };
                prepend_report(
                    &mut report,
                    kind,
                    transformed,
                    key,
                    &[("BiomeName", biome_name(biome_name_key(*biome)))],
                );
            }
            MoodFlag::NpcAffection(level, other) => {
                // The LikePrincess row's dedicated key (`ShopHelper.cs:
                // 252-259` — `LikeNPC_Princess` with the Princess's name).
                let key = if *level == AffectionLevel::Like && *other == 663 {
                    "LikeNPC_Princess"
                } else {
                    match level {
                        AffectionLevel::Love => "LoveNPC",
                        AffectionLevel::Like => "LikeNPC",
                        AffectionLevel::Dislike => "DislikeNPC",
                        AffectionLevel::Hate => "HateNPC",
                    }
                };
                prepend_report(
                    &mut report,
                    kind,
                    transformed,
                    key,
                    &[("NPCName", npc_full_name_by_id(all_npcs, *other))],
                );
            }
            MoodFlag::PrincessLove(other) => {
                // `LoveNPCByTypeName` (`ShopHelper.cs:243-250`): the key is
                // `LoveNPC_{NPCID.Search.GetName(npcType)}` with the picked
                // neighbor's full name. The category is the Princess's own
                // (`kind`), the key names the picked neighbor.
                let Some(name) = crate::lang_tables::NPC_ID
                    .iter()
                    .find(|(id, _)| *id == *other)
                    .map(|(_, n)| *n)
                else {
                    continue;
                };
                prepend_report(
                    &mut report,
                    kind,
                    transformed,
                    &format!("LoveNPC_{name}"),
                    &[("NPCName", npc_full_name_by_id(all_npcs, *other))],
                );
            }
        }
    }
    report
}

/// `ShopHelper.GetShoppingSettings` (`ShopHelper.cs:49-62`) — the
/// `Player.SetTalkNPC` companion (`Player.cs:4389`): the mood pass plus the
/// report string. `process_mood`'s flags carry the call order; the report
/// folds them through `AddHappinessReportText`'s prepend. The Zoologist's
/// `Transformed` category suffix reads her shapeshift form off the world
/// clock (`ShouldBestiaryGirlBeLycantrope`, the predicate `UpdateAltTexture`
/// writes `altTexture = 2` on — `NPC.cs:90225-90227`). `rng` is the bare
/// `Main.rand` stand-in the Princess head draws (`NamedRngs::global()`).
/// batch-L (research/68 §3 item 8): this is the struct-building helper the
/// capture sites call WITH the rng — `set_talk_npc` + the gated shop-open
/// are the ONLY callers.
pub fn shopping_settings(
    world: &World,
    all_npcs: &[crate::npc::Npc],
    npc: &crate::npc::Npc,
    player_center: (f32, f32),
    raining: bool,
    rng: &mut tr_core::UnifiedRandom,
) -> ShoppingSettings {
    let mood = process_mood(world, all_npcs, npc, player_center, raining, rng);
    let transformed =
        should_bestiary_girl_be_lycantrope(world.blood_moon, world.day_time, world.moon_phase);
    ShoppingSettings {
        price_adjustment: mood.multiplier,
        happiness_report: happiness_report(npc.kind, transformed, &mood.flags, all_npcs),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::{Npc, NpcKind};
    use tr_core::World;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// The forest test ground: dirt from y = 120 — no special zones (the
    /// home rows sit below the beach's ocean level, so the Forest
    /// fallthrough holds).
    fn forest_world() -> World {
        // batch-L merged fixture: g58's 1000-wide ground (the width exceeds
        // 2×BEACH_DISTANCE so the x≈500 columns are NOT ocean — its hallow/
        // judge witnesses need the band) + f37's `world_surface = 120.0`
        // (research/68 §4: the player-zone judge needs the player above the
        // surface for the Forest row; f37's x≈60 rigs stay non-beach too).
        let mut w = World::new("t".into(), 1000, 200);
        for x in 0..1000 {
            for y in 120..200 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.world_surface = 120.0;
        w.rock_layer = 140.0;
        w.compute_column_surface();
        w
    }

    fn housed_guide(w: &World, home: (i32, i32)) -> Npc {
        let mut g = Npc::spawn_guide(w, home.0).expect("the guide spawns");
        g.home = Some(home);
        g.hp = g.max_hp;
        g
    }

    /// A no-draw `Main.rand` stand-in for the non-Princess judges — their
    /// random three-loves head never runs (batch-L: the merged core takes
    /// `&mut UnifiedRandom`; the seed-0 stream stands in for g58's no_roll
    /// closure, never drawn).
    fn no_roll() -> tr_core::UnifiedRandom {
        tr_core::UnifiedRandom::new(0)
    }


    /// A hallow patch around (cx, cy) on the forest ground — pearlstone
    /// (109) counts toward `HolyTileCount` (`SceneMetrics.cs:589`); 125+
    /// tiles in the scan window raise `ZoneHallow` (`:673`, threshold
    /// `HallowTileThreshold` = 125). The patch sits ABOVE the surface so
    /// `below_surface` stays false (the Forest/Underground rows stay out).
    fn hallow_patch(w: &mut World, cx: i32, cy: i32) {
        for x in (cx - 8)..(cx + 8) {
            for y in (cy - 8)..(cy + 8) {
                if x < 0 || y < 0 || x >= w.width || y >= w.height {
                    continue;
                }
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 109,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
    }
    #[test]
    fn the_travelling_merchant_ignores_the_mood() {
        // `ShopHelper.cs:103-106`'s early return: `npc.type == 368` (and the
        // 453/37 pair, `IsNotReallyTownNPC` at `:363-371`) drops out before
        // the homelessness sentinel — even though `AI_007` forces him
        // `homeless = true` every tick (`NPC.cs:53360`), his prices ride the
        // plain `PriceAdjustment = 1f` with an empty report.
        let w = forest_world();
        let mut m = Npc::new(NpcKind::TravellingMerchant, 60.0 * 16.0, 120.0 * 16.0, 1, 0.0);
        m.home = None; // homeless, like the vanilla's forced flag
        let mood = process_mood(
            &w,
            std::slice::from_ref(&m),
            &m,
            (60.0 * 16.0, 110.0 * 16.0),
            false,
            &mut tr_core::UnifiedRandom::new(0),
        );
        assert_eq!(mood.multiplier, 1.0, "the plain multiplier");
        assert!(mood.flags.is_empty(), "no happiness report");
        // The distance formula `ShopHelper.GetTravelingMerchantPrices`
        // (`ShopHelper.cs:90-93`) is NEVER called in 1.4.5.8 (repo-wide grep:
        // no caller) — the port keeps the math out of the price path and
        // records it here: `(2 + (1.5 - d / (maxTilesX / 2))) / 3`.
        let formula = |dist_tiles: f32, max_tiles_x: i32| {
            (2.0 + (1.5 - dist_tiles / (max_tiles_x as f32 / 2.0))) / 3.0
        };
        assert!((formula(0.0, 400) - 1.166_666_6).abs() < 1e-5, "at the spawn");
        assert!(
            (formula(200.0, 400) - 0.833_333_3).abs() < 1e-5,
            "200 tiles from the spawn on a 400-wide world"
        );
    }

    #[test]
    fn the_forest_guide_alone_is_content() {
        // `ProcessMood` (`ShopHelper.cs:95-174`): the housed Guide alone in
        // the forest — the LoveSpace ×0.95 (`:147-151`) times the Forest
        // Like ×0.94 (`:27-30`'s Register(22)) = 0.893 → the cents round
        // 0.89 (`:176-181`).
        let w = forest_world();
        let guide = housed_guide(&w, (60, 120));
        let mood = process_mood(
            &w,
            std::slice::from_ref(&guide),
            &guide,
            (60.0 * 16.0, 110.0 * 16.0),
            false,
            &mut tr_core::UnifiedRandom::new(0),
        );
        assert!(
            (mood.multiplier - 0.89).abs() < 0.001,
            "the multiplier {}",
            mood.multiplier
        );
        assert!(mood.flags.contains(&MoodFlag::LoveSpace));
        assert!(mood.flags.contains(&MoodFlag::BiomeAffection(
            AffectionLevel::Like,
            BiomeKind::Forest
        )));
    }

    #[test]
    fn the_homeless_guide_prices_at_the_ceiling() {
        // `RuinMoodIfHomeless` (`:107-110`): the sentinel 1000 rides the
        // clamp's 1.5 ceiling (`:178`) — the vanilla's crazy prices ARE the
        // ceiling.
        let w = forest_world();
        let mut guide = Npc::spawn_guide(&w, 60).expect("the guide spawns");
        guide.home = None;
        guide.hp = guide.max_hp;
        let mood = process_mood(
            &w,
            std::slice::from_ref(&guide),
            &guide,
            (60.0 * 16.0, 110.0 * 16.0),
            false,
            &mut tr_core::UnifiedRandom::new(0),
        );
        assert!((mood.multiplier - 1.5).abs() < 0.001);
        assert!(mood.flags.contains(&MoodFlag::NoHome));
    }

    #[test]
    fn the_evil_biome_prices_at_the_ceiling() {
        // `IsPlayerInEvilBiomes` (`:115-118`, the three dangerous biomes
        // `:28-34`): the corruption under the player pushes the sentinel.
        let mut w = forest_world();
        // The ebonstone patch around the player centre (the scan counts
        // kind 23; the 300-tile threshold).
        for x in 45..70 {
            for y in 100..125 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 23,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        // The Guide stands above the patch (the column-surface cache predates
        // the carve — the placement is manual).
        let mut guide = Npc::new(NpcKind::Guide, 60.0 * 16.0, 90.0 * 16.0, 1, 0.0);
        guide.home = Some((60, 120));
        guide.hp = guide.max_hp;
        let mood = process_mood(
            &w,
            std::slice::from_ref(&guide),
            &guide,
            (57.0 * 16.0, 112.0 * 16.0),
            false,
            &mut tr_core::UnifiedRandom::new(0),
        );
        assert!((mood.multiplier - 1.5).abs() < 0.001);
        // batch-L re-pin (research/68 §3 item 7): the evil arm pushes the
        // NAMED row (g58's report payload), not the name-less marker.
        assert!(mood.flags.contains(&MoodFlag::HateBiomeNamed(BiomeKind::Corruption)));
    }

    #[test]
    fn the_crowd_multiplies_past_three() {
        // `:132-146`: the residents within 25 tiles past the third push
        // ×1.05 each; the LoveSpace gate closes (within_house > 2).
        let w = forest_world();
        let guide = housed_guide(&w, (60, 120));
        let mut all = vec![guide.clone()];
        for (dx, dy) in [(4, 0), (-4, 0), (0, -6), (7, 3)] {
            let mut other = Npc::spawn_guide(&w, 60 + dx).expect("the neighbor spawns");
            other.home = Some((60 + dx, 120 + dy));
            other.hp = other.max_hp;
            all.push(other);
        }
        let mood = process_mood(
            &w,
            &all,
            &all[0],
            (60.0 * 16.0, 110.0 * 16.0),
            false,
            &mut tr_core::UnifiedRandom::new(0),
        );
        // 4 neighbors → 1 × 1.05; the forest Like ×0.94 → 0.987 → 0.99.
        assert!(
            (mood.multiplier - 0.99).abs() < 0.001,
            "the multiplier {}",
            mood.multiplier
        );
        assert!(mood.flags.contains(&MoodFlag::DislikeCrowded));
        assert!(!mood.flags.contains(&MoodFlag::LoveSpace));
    }

    #[test]
    fn the_love_space_gate_needs_the_village_room() {
        // `:147-151`: within_house ≤ 2 AND within_village < 4 — a packed
        // village (the neighbors 25..120 tiles out) keeps LoveSpace off.
        let w = forest_world();
        let guide = housed_guide(&w, (60, 120));
        let mut all = vec![guide.clone()];
        for dx in [30, 60, 90, 110] {
            let mut other = Npc::spawn_guide(&w, 60 + dx).expect("the villager spawns");
            other.home = Some((60 + dx, 120));
            other.hp = other.max_hp;
            all.push(other);
        }
        let mood = process_mood(
            &w,
            &all,
            &all[0],
            (60.0 * 16.0, 110.0 * 16.0),
            false,
            &mut tr_core::UnifiedRandom::new(0),
        );
        // within_house = 0, within_village = 4 → no LoveSpace; the forest
        // Like alone → 0.94.
        assert!(
            (mood.multiplier - 0.94).abs() < 0.001,
            "the multiplier {}",
            mood.multiplier
        );
        assert!(!mood.flags.contains(&MoodFlag::LoveSpace));
    }
    #[test]
    fn the_dryad_judges_the_jungle_and_the_desert() {
        // `Register(20)` (`PersonalityDatabasePopulator.cs:82-86`): the
        // Like Jungle / Dislike Desert rows ride the home anchor's biome.
        assert_eq!(biome_preferences(NpcKind::Dryad).len(), 2);
        assert_eq!(
            biome_preferences(NpcKind::Dryad)[0].biome,
            BiomeKind::Jungle
        );
        assert_eq!(
            biome_preferences(NpcKind::Dryad)[1].biome,
            BiomeKind::Desert
        );
    }

    #[test]
    fn the_tinkerer_judges_the_underground_and_the_jungle() {
        // `Register(107)` (`PersonalityDatabasePopulator.cs:117-121`): the
        // Like Underground / Dislike Jungle rows.
        assert_eq!(biome_preferences(NpcKind::GoblinTinkerer).len(), 2);
        assert_eq!(
            biome_preferences(NpcKind::GoblinTinkerer)[0].level,
            AffectionLevel::Like
        );
        assert_eq!(
            biome_preferences(NpcKind::GoblinTinkerer)[0].biome,
            BiomeKind::Underground
        );
        assert_eq!(
            biome_preferences(NpcKind::GoblinTinkerer)[1].level,
            AffectionLevel::Dislike
        );
        assert_eq!(
            biome_preferences(NpcKind::GoblinTinkerer)[1].biome,
            BiomeKind::Jungle
        );
    }

    /// W4-f37 — `loveStruck` (buff 119, `NPC.cs:92057-92060`) multiplies ×0.9
    /// BEFORE the exclusions (`ShopHelper.cs:99-106`): the travelling seller
    /// keeps the 0.9 (UNROUNDED — the early return skips the cents tail),
    /// the forest Guide rides 0.893 × 0.9 = 0.8037 → 0.80.
    #[test]
    fn the_love_struck_multiply_precedes_the_exclusions() {
        let w = forest_world();
        let mut m = Npc::new(NpcKind::TravellingMerchant, 60.0 * 16.0, 120.0 * 16.0, 1, 0.0);
        m.home = None;
        m.buffs.add_buff(119, 600);
        let mood = process_mood(
            &w,
            std::slice::from_ref(&m),
            &m,
            (60.0 * 16.0, 110.0 * 16.0),
            false,
            &mut tr_core::UnifiedRandom::new(0),
        );
        assert!(
            (mood.multiplier - 0.9).abs() < 1e-5,
            "the exclusion keeps the raw 0.9, got {}",
            mood.multiplier
        );
        assert!(mood.flags.is_empty());
        let mut guide = housed_guide(&w, (60, 120));
        guide.buffs.add_buff(119, 600);
        let mood = process_mood(
            &w,
            std::slice::from_ref(&guide),
            &guide,
            (60.0 * 16.0, 110.0 * 16.0),
            false,
            &mut tr_core::UnifiedRandom::new(0),
        );
        assert!(
            (mood.multiplier - 0.80).abs() < 0.001,
            "0.95 × 0.94 × 0.9 → 0.80, got {}",
            mood.multiplier
        );
    }

    /// W4-f37 — `Main.remixWorld` (`ShopHelper.cs:103`): EVERY town NPC drops
    /// out before the mood work — even the homeless Guide keeps 1.0 (the
    /// sentinel never runs), with no report.
    #[test]
    fn the_remix_world_skips_every_mood() {
        let mut w = forest_world();
        w.seeds.insert(tr_core::seeds::SeedFlags::REMIX);
        let mut guide = Npc::spawn_guide(&w, 60).expect("the guide spawns");
        guide.home = None; // homeless — would be the 1.5 ceiling on classic
        guide.hp = guide.max_hp;
        let mood = process_mood(
            &w,
            std::slice::from_ref(&guide),
            &guide,
            (60.0 * 16.0, 110.0 * 16.0),
            false,
            &mut tr_core::UnifiedRandom::new(0),
        );
        assert_eq!(mood.multiplier, 1.0);
        assert!(mood.flags.is_empty());
    }

    /// W4-f37 — the Princess (663, `ShopHelper.cs:122-131` +
    /// `AllPersonalitiesModifier.cs:15-36`): alone she hates the loneliness
    /// (the sentinel → 1.5); with neighbors she draws up to 3 random loves
    /// off `Main.rand` (each removing its pick) and NEVER rides the crowd
    /// ×1.05 or LoveSpace (flag = false).
    #[test]
    fn the_princess_hates_loneliness_and_skips_the_crowd_rows() {
        let w = forest_world();
        let mut princess = Npc::new(NpcKind::Princess, 60.0 * 16.0, 118.0 * 16.0, 1, 0.0);
        princess.home = Some((60, 120));
        princess.hp = princess.max_hp;
        // Alone: HateLonely at the sentinel (the clamp ceiling).
        let mood = process_mood(
            &w,
            std::slice::from_ref(&princess),
            &princess,
            (60.0 * 16.0, 110.0 * 16.0),
            false,
            &mut tr_core::UnifiedRandom::new(0),
        );
        assert!((mood.multiplier - 1.5).abs() < 0.001);
        assert!(mood.flags.contains(&MoodFlag::HateLonely));
        assert!(!mood.flags.contains(&MoodFlag::LoveSpace));
        // Four neighbors (past the crowd gate): the Princess's num = 1f
        // kills the ×1.05s, but the REPORT still lands (`:138-145` is not
        // flag-gated); LoveSpace stays off (flag = false); her loves alone
        // (one deduped Guide type → one ×0.88 → 0.88).
        let mut all = vec![princess.clone()];
        for (dx, dy) in [(4, 0), (-4, 0), (0, -6), (7, 3)] {
            let mut other = Npc::spawn_guide(&w, 60 + dx).expect("the neighbor spawns");
            other.home = Some((60 + dx, 120 + dy));
            other.hp = other.max_hp;
            all.push(other);
        }
        let mood = process_mood(
            &w,
            &all,
            &all[0],
            (60.0 * 16.0, 110.0 * 16.0),
            false,
            &mut tr_core::UnifiedRandom::new(0),
        );
        assert!(
            (mood.multiplier - 0.88).abs() < 0.001,
            "one deduped love, no crowd multiply, got {}",
            mood.multiplier
        );
        assert!(mood.flags.contains(&MoodFlag::DislikeCrowded), "the report still lands");
        assert!(!mood.flags.contains(&MoodFlag::LoveSpace));
        assert!(!mood.flags.contains(&MoodFlag::HateLonely));
    }

    /// W4-f37 — the Princess's picks dedup by TYPE (the `bool[NPCID.Count]`
    /// scan, `AllPersonalitiesModifier.cs:14-24`): two Guides nearby are ONE
    /// entry — one love draw ×0.88 → 0.88. A Guide + a Merchant are two
    /// entries — two draws, and the RNG replay pins WHICH order.
    #[test]
    fn the_princess_picks_dedup_by_type_and_ride_the_global_stream() {
        let w = forest_world();
        let mut princess = Npc::new(NpcKind::Princess, 60.0 * 16.0, 118.0 * 16.0, 1, 0.0);
        princess.home = Some((60, 120));
        princess.hp = princess.max_hp;
        let mut all = vec![princess.clone()];
        for dx in [4, -4] {
            let mut other = Npc::spawn_guide(&w, 60 + dx).expect("the neighbor spawns");
            other.home = Some((60 + dx, 120));
            other.hp = other.max_hp;
            all.push(other);
        }
        let mut rng = tr_core::UnifiedRandom::new(0);
        let mood = process_mood(&w, &all, &all[0], (60.0 * 16.0, 110.0 * 16.0), false, &mut rng);
        assert!(
            (mood.multiplier - 0.88).abs() < 0.001,
            "one deduped type → one love, got {}",
            mood.multiplier
        );
        assert_eq!(
            mood.flags
                .iter()
                .filter(|f| matches!(f, MoodFlag::PrincessLove(_)))
                .count(),
            1
        );
        // batch-L re-pin: the picks ride the dedicated `PrincessLove` flag
        // (g58's LoveNPC_{name} report key), not the plain NpcAffection.
        assert!(mood.flags.contains(&MoodFlag::PrincessLove(22)));
        // Two TYPES: the draws replay off the same seed — first
        // `Next(2)` picks the index, `RemoveAt` shrinks the list.
        let mut merchant = Npc::new(NpcKind::Merchant, 64.0 * 16.0, 118.0 * 16.0, 1, 0.0);
        merchant.home = Some((64, 120));
        merchant.hp = merchant.max_hp;
        all[2] = merchant;
        let mut rng = tr_core::UnifiedRandom::new(0);
        let mut expect = tr_core::UnifiedRandom::new(0);
        let first = expect.next_max(2) as usize;
        let mut picks = vec![17i32, 22];
        let pick1 = picks.remove(first);
        // The second draw is `Next(1)` — it still consumes the stream.
        let _ = expect.next_max(1);
        let pick2 = picks.remove(0);
        let mood = process_mood(&w, &all, &all[0], (60.0 * 16.0, 110.0 * 16.0), false, &mut rng);
        assert_eq!(rng, expect, "the picks ride the passed stream verbatim");
        assert!(mood.flags.contains(&MoodFlag::PrincessLove(pick1)));
        assert!(mood.flags.contains(&MoodFlag::PrincessLove(pick2)));
        assert!(
            (mood.multiplier - 0.77).abs() < 0.001,
            "0.88² → 0.77, got {}",
            mood.multiplier
        );
    }

    /// W4-f37 — everyone BUT the Princess likes a nearby Princess
    /// (`AllPersonalitiesModifier.cs:37-40`), and the Guide DOES carry
    /// switch rows (case 22, `:451-468`): a nearby Clothier (54) is a Like.
    /// Forest Like ×0.94 × LikePrincess ×0.94 × LikeNPC(54) ×0.94 with
    /// LoveSpace ×0.95 (2 in house) → 0.95 × 0.94³ = 0.789… → 0.79.
    #[test]
    fn the_guide_likes_a_nearby_princess_and_clothier() {
        let w = forest_world();
        let guide = housed_guide(&w, (60, 120));
        let mut all = vec![guide.clone()];
        let mut princess = Npc::new(NpcKind::Princess, 64.0 * 16.0, 118.0 * 16.0, 1, 0.0);
        princess.home = Some((64, 120));
        princess.hp = princess.max_hp;
        all.push(princess);
        let mut clothier = Npc::new(NpcKind::Clothier, 56.0 * 16.0, 118.0 * 16.0, 1, 0.0);
        clothier.home = Some((56, 120));
        clothier.hp = clothier.max_hp;
        all.push(clothier);
        let mood = process_mood(
            &w,
            &all,
            &all[0],
            (60.0 * 16.0, 110.0 * 16.0),
            false,
            &mut tr_core::UnifiedRandom::new(0),
        );
        assert!(mood.flags.contains(&MoodFlag::NpcAffection(AffectionLevel::Like, 663)));
        assert!(mood.flags.contains(&MoodFlag::NpcAffection(AffectionLevel::Like, 54)));
        assert!(
            (mood.multiplier - 0.79).abs() < 0.001,
            "0.95 × 0.94³ = 0.789 → 0.79, got {}",
            mood.multiplier
        );
        // The report order: LoveSpace, the biome Like, LikePrincess, then
        // the switch rows (`ShopHelper.cs:164-168` — the traits before the
        // AllPersonalitiesModifier? NO: the database traits run FIRST, then
        // AllPersonalities — the biome row precedes the princess like).
        let idx = |f: MoodFlag| mood.flags.iter().position(|x| *x == f).unwrap();
        assert!(idx(MoodFlag::BiomeAffection(AffectionLevel::Like, BiomeKind::Forest))
            < idx(MoodFlag::NpcAffection(AffectionLevel::Like, 663)));
        assert!(idx(MoodFlag::NpcAffection(AffectionLevel::Like, 663))
            < idx(MoodFlag::NpcAffection(AffectionLevel::Like, 54)));
    }

    /// W4-f37 — the biome trait judges the PLAYER's zones, not the NPC's
    /// home scene (`BiomePreferenceListTrait.cs:44` —
    /// `IsInBiome(info.player)`): the Guide's home sits in the forest, the
    /// PLAYER stands in the underground (below `Main.worldSurface`) — no
    /// Forest row, no Underground row (the Guide has none), so the mood is
    /// LoveSpace alone ×0.95. The old home-anchor port gave 0.89 here.
    #[test]
    fn the_biome_traits_judge_the_player_not_the_npc_home() {
        let w = forest_world();
        let guide = housed_guide(&w, (60, 120));
        // The player BELOW the surface line (tile y 130 > 120) — the dirt
        // there carries no special zone, so only ShoppingZone_BelowSurface.
        let mood = process_mood(
            &w,
            std::slice::from_ref(&guide),
            &guide,
            (60.0 * 16.0, 130.0 * 16.0),
            false,
            &mut tr_core::UnifiedRandom::new(0),
        );
        assert!(
            (mood.multiplier - 0.95).abs() < 0.001,
            "LoveSpace alone, got {}",
            mood.multiplier
        );
        assert!(!mood.flags.iter().any(|f| matches!(f, MoodFlag::BiomeAffection(..))));
    }

    /// W4-f37 — the single BEST-affection row wins
    /// (`BiomePreferenceListTrait.cs:44`'s `Affection <` compare,
    /// `AffectionLevel.cs:5-8`): the Tinkerer underground AND in the jungle
    /// rides the Like Underground ×0.94 ALONE (not ×1.06 for the jungle
    /// Dislike too). The rig: the player below the surface inside a jungle
    /// patch (the scan's 140-tile threshold).
    #[test]
    fn the_best_affection_row_wins_over_the_lower_ones() {
        let mut w = forest_world();
        // Jungle grass (kind 60) under the player — enough for the zone.
        for x in 40..80 {
            for y in 125..160 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 60,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let mut tink = Npc::new(NpcKind::GoblinTinkerer, 60.0 * 16.0, 118.0 * 16.0, 1, 0.0);
        tink.home = Some((60, 120));
        tink.hp = tink.max_hp;
        let mood = process_mood(
            &w,
            std::slice::from_ref(&tink),
            &tink,
            (60.0 * 16.0, 130.0 * 16.0),
            false,
            &mut tr_core::UnifiedRandom::new(0),
        );
        assert!(mood.flags.contains(&MoodFlag::BiomeAffection(
            AffectionLevel::Like,
            BiomeKind::Underground
        )));
        assert!(!mood.flags.contains(&MoodFlag::BiomeAffection(
            AffectionLevel::Dislike,
            BiomeKind::Jungle
        )));
        // LoveSpace ×0.95 × Like ×0.94 = 0.893 → 0.89.
        assert!(
            (mood.multiplier - 0.89).abs() < 0.001,
            "got {}",
            mood.multiplier
        );
    }

    /// W4-f37 — the cents round is the BANKER'S round
    /// (`ShopHelper.cs:179` — `Math.Round(x * 100) / 100`): 0.925 → 92.5 →
    /// 92 (even), not 93. The rig: LoveSpace ×0.95 × Dislike ×1.06 =
    /// 1.007 → 1.01 is not a half case; the direct helper pins the tie.
    #[test]
    fn the_cents_round_is_bankers() {
        // The helper the tail rides (shop.rs's Math.Round port).
        assert_eq!(crate::shop::round_half_to_even(92.5), 92);
        assert_eq!(crate::shop::round_half_to_even(93.5), 94);
        assert_eq!(crate::shop::round_half_to_even(88.5), 88);
        // A full-chain tie: 0.95 (LoveSpace) × 0.94 (Like) × 0.94
        // (LikeNPC) = 0.83942 → 83.942 → 84 — no tie; the half-case needs
        // the raw product to land exactly on .5, which the f32 chain
        // rarely does — the helper pins the CONTRACT, the chain pins the
        // plumbing.
        let w = forest_world();
        let guide = housed_guide(&w, (60, 120));
        let mut clothier = Npc::new(NpcKind::Clothier, 64.0 * 16.0, 118.0 * 16.0, 1, 0.0);
        clothier.home = Some((64, 120));
        clothier.hp = clothier.max_hp;
        let all = vec![guide.clone(), clothier];
        let mood = process_mood(
            &w,
            &all,
            &all[0],
            (60.0 * 16.0, 110.0 * 16.0),
            false,
            &mut tr_core::UnifiedRandom::new(0),
        );
        // 0.95 × 0.94 × 0.94 = 0.83942 → 0.84.
        assert!(
            (mood.multiplier - 0.84).abs() < 0.001,
            "got {}",
            mood.multiplier
        );
    }

    /// W4-f37 — the traits run even at the sentinel
    /// (`ShopHelper.cs:152-168` has NO `< 1000` gate): the homeless Guide
    /// still collects LoveSpace + the Forest Like into the report; the
    /// clamp lands the product on the 1.5 ceiling either way.
    #[test]
    fn the_traits_run_at_the_sentinel_too() {
        let w = forest_world();
        let mut guide = Npc::spawn_guide(&w, 60).expect("the guide spawns");
        guide.home = None;
        guide.hp = guide.max_hp;
        let mood = process_mood(
            &w,
            std::slice::from_ref(&guide),
            &guide,
            (60.0 * 16.0, 110.0 * 16.0),
            false,
            &mut tr_core::UnifiedRandom::new(0),
        );
        assert!((mood.multiplier - 1.5).abs() < 0.001);
        assert!(mood.flags.contains(&MoodFlag::NoHome));
        assert!(mood.flags.contains(&MoodFlag::LoveSpace));
        assert!(mood.flags.contains(&MoodFlag::BiomeAffection(
            AffectionLevel::Like,
            BiomeKind::Forest
        )));
    }

    /// W4-g58 witnesses — the DATA tables against the C# source rows.
    mod w4_g58_tests {
        use super::*;

        /// `PersonalityDatabasePopulator.cs:27-147` — all 25 Register rows:
        /// the id, the row count and the (level, biome) pairs in the
        /// initializer order.
        #[test]
        fn the_biome_table_matches_the_populator_rows() {
            let cases: &[(u16, &[(AffectionLevel, BiomeKind)])] = &[
                (22, &[(L, Forest), (D, Ocean)]),
                (17, &[(L, Forest), (D, Desert)]),
                (588, &[(L, Forest), (D, Underground)]),
                (633, &[(L, Forest), (D, Desert)]),
                (441, &[(L, Snow), (D, Hallow)]),
                (124, &[(L, Snow), (D, Underground)]),
                (209, &[(L, Snow), (D, Jungle)]),
                (142, &[(V, Snow), (H, Desert)]),
                (207, &[(L, Desert), (D, Forest)]),
                (19, &[(L, Desert), (D, Snow)]),
                (178, &[(L, Desert), (D, Jungle)]),
                (20, &[(L, Jungle), (D, Desert)]),
                (228, &[(L, Jungle), (D, Hallow)]),
                (227, &[(L, Jungle), (D, Forest)]),
                (369, &[(L, Ocean), (D, Desert)]),
                (229, &[(L, Ocean), (D, Underground)]),
                (353, &[(L, Ocean), (D, Snow)]),
                (38, &[(L, Underground), (D, Ocean)]),
                (107, &[(L, Underground), (D, Jungle)]),
                (54, &[(L, Underground), (D, Hallow)]),
                (108, &[(L, Hallow), (D, Ocean)]),
                (18, &[(L, Hallow), (D, Snow)]),
                (208, &[(L, Hallow), (D, Underground)]),
                (550, &[(L, Hallow), (D, Snow)]),
                (160, &[(L, Mushroom)]),
            ];
            for (id, rows) in cases {
                let got = biome_preferences_by_id(*id);
                assert_eq!(got.len(), rows.len(), "id {id} row count");
                for (i, (lv, b)) in rows.iter().enumerate() {
                    assert_eq!(got[i].level, *lv, "id {id} row {i} level");
                    assert_eq!(got[i].biome, *b, "id {id} row {i} biome");
                }
            }
            // The unregistered ids carry no rows (the Princess 663 has no
            // Register row — `Populate_BiomePreferences` skips her).
            assert!(biome_preferences_by_id(663).is_empty());
            assert!(biome_preferences_by_id(368).is_empty());
        }

        /// `AllPersonalitiesModifier.cs:41-505` — every switch case's rows
        /// in the C# `if` order (the report prepends, so the order is the
        /// read order).
        #[test]
        fn the_neighbor_table_matches_the_switch() {
            let cases: &[(u16, &[(AffectionLevel, u16)])] = &[
                (17, &[(L, 588), (L, 18), (D, 441), (H, 369)]),
                (18, &[(V, 19), (L, 108), (D, 208), (D, 20), (H, 633)]),
                (227, &[(V, 20), (L, 208), (D, 209), (D, 160)]),
                (207, &[(L, 19), (L, 227), (D, 178), (H, 229)]),
                (208, &[(V, 108), (L, 353), (D, 17), (H, 441), (V, 633)]),
                (369, &[(L, 208), (L, 38), (L, 441), (H, 550)]),
                (353, &[(V, 207), (L, 229), (D, 550), (H, 107)]),
                (38, &[(V, 550), (L, 124), (D, 107), (D, 19)]),
                (20, &[(L, 228), (L, 160), (D, 369), (H, 588)]),
                (550, &[(V, 38), (L, 107), (D, 22), (H, 207)]),
                (19, &[(V, 18), (L, 178), (D, 588), (H, 38)]),
                (107, &[(V, 124), (L, 207), (D, 54), (H, 353)]),
                (228, &[(L, 20), (L, 22), (D, 18), (H, 160)]),
                (54, &[(V, 160), (L, 441), (D, 18), (H, 124)]),
                (124, &[(V, 107), (L, 209), (D, 19), (H, 54)]),
                (441, &[(V, 17), (L, 208), (D, 38), (D, 124), (H, 142)]),
                (229, &[(V, 369), (L, 550), (D, 353), (H, 22)]),
                (108, &[(V, 588), (L, 17), (D, 228), (H, 209)]),
                (178, &[(V, 209), (L, 227), (D, 208), (D, 108), (D, 20)]),
                (209, &[(L, 353), (L, 229), (L, 178), (H, 108), (D, 633)]),
                (142, &[(H, 441)]),
                (588, &[(L, 227), (V, 369), (H, 17), (D, 229), (L, 633)]),
                (22, &[(L, 54), (D, 178), (H, 227), (L, 633)]),
                (160, &[(V, 22), (L, 20), (D, 54), (H, 228)]),
                (633, &[(D, 369), (H, 19), (V, 228), (L, 588)]),
            ];
            for (id, rows) in cases {
                assert_eq!(neighbor_preferences_by_id(*id), *rows, "id {id}");
            }
            // The Princess's own case is the random three-loves head —
            // no static rows (`:15-36`).
            assert!(neighbor_preferences_by_id(663).is_empty());
        }

        /// The kind-keyed view — batch-L RE-PIN (research/68 §3 item 4):
        /// the wrapper returns the VERBATIM id rows (no unported-neighbor
        /// drop — the ids never match until the kind lands, and the
        /// Princess's whole-roster picks need them).
        #[test]
        fn the_kind_view_filters_the_unported_neighbors() {
            // The Guide (`:451-468`): Clothier like, Steampunker dislike
            // (178 unported — the row STAYS verbatim), Painter hate,
            // Zoologist like.
            let guide = neighbor_preferences(NpcKind::Guide);
            assert_eq!(
                guide,
                &[(L, 54), (D, 178), (H, 227), (L, 633)]
            );
            // The Merchant: the Angler hate row survives (369 ports).
            let merchant = neighbor_preferences(NpcKind::Merchant);
            assert_eq!(merchant.len(), 4);
            assert_eq!(merchant[3], (H, 369));
            // The Tax Collector's Santa hate row STAYS (142 unported).
            let tax = neighbor_preferences(NpcKind::TaxCollector);
            assert_eq!(tax.len(), 5);
            assert!(tax.iter().any(|(_, id)| *id == 142));
        }

        /// The Tavernkeep (550) is a town NPC — `NPCID.DD2Bartender` IS the
        /// town Tavernkeep (`NPC.cs:16261-16274`: `townNPC = true`, aiStyle
        /// 7); NPCID 109 is the Clown (the port's wire id), NOT a second
        /// event twin. His own rows + the rows that name him resolve, and
        /// he counts toward the crowd (`GetNearbyResidentNPCs`, `:305`).
        #[test]
        fn the_tavernkeep_is_a_town_neighbor() {
            // His `cs_id` resolves 550 through the bestiary (not the wire 109).
            assert_eq!(NpcKind::DD2Bartender.cs_id(), 550);
            // His own switch (`:213-230`): Demolitionist love, Tinkerer
            // like, Guide dislike, DyeTrader hate — all four port.
            let tk = neighbor_preferences(NpcKind::DD2Bartender);
            assert_eq!(
                tk,
                &[(V, 38), (L, 107), (D, 22), (H, 207)]
            );
            // The rows that NAME him survive: the Angler hates him (`:141-
            // 158`), the Demolitionist loves him (`:177-194`).
            let angler = neighbor_preferences(NpcKind::Angler);
            assert!(angler.contains(&(H, 550)));
            let demo = neighbor_preferences(NpcKind::Demolitionist);
            assert!(demo.contains(&(V, 550)));
            // The crowd gate admits him: a housed Tavernkeep beside a
            // housed Guide is one within-house neighbor (the Guide alone
            // would be LoveSpace; two housed NPCs still ≤2 → LoveSpace,
            // but the roster now counts the Tavernkeep). The multiplier
            // witness: the Guide + the Tavernkeep within 25 tiles, the
            // forest Like ×0.94 × LoveSpace 0.95 × the Guide's LikeNPC
            // (the Tavernkeep is not in the Guide's rows, so no neighbor
            // factor) — the point is the Tavernkeep is COUNTED, not dropped.
            let w = forest_world();
            let guide = housed_guide(&w, (500, 120));
            let mut tk_npc = Npc::new(NpcKind::DD2Bartender, 504.0 * 16.0, 118.0 * 16.0, 1, 0.0);
            tk_npc.home = Some((504, 120));
            tk_npc.hp = tk_npc.max_hp;
            let all = vec![guide.clone(), tk_npc];
            let mood = process_mood(&w, &all, &all[0], (500.0 * 16.0, 110.0 * 16.0), false, &mut no_roll());
            // within_house = 1 (the Tavernkeep) → LoveSpace holds; the
            // forest Like ×0.94 × 0.95 = 0.893 → 0.89.
            assert!(mood.flags.contains(&MoodFlag::LoveSpace));
            assert!(
                (mood.multiplier - 0.89).abs() < 0.001,
                "the multiplier {}",
                mood.multiplier
            );
        }

        /// `ShopHelper.cs:305` + `TownRoomManager.cs:153-161` — the crowd
        /// gate admits only SAME-housingCategory neighbors (the C# negates
        /// CanLive, which is category INEQUALITY). Humans count humans;
        /// a pet never crowds a human and vice versa; pets crowd pets.
        #[test]
        fn the_crowd_gate_filters_by_housing_category() {
            // Human × human (Guide 22, Nurse 18 — both category 0).
            assert!(crowd_neighbor_counts(22, 18));
            // Human × pet (TownCat 637, category 1 — `NPC.cs:17224`):
            // CanLive = true → the negated mood gate DROPS the pet.
            assert!(!crowd_neighbor_counts(22, 637));
            assert!(!crowd_neighbor_counts(637, 22));
            // Pet × pet (TownCat 637, TownDog 638): same category → counts.
            assert!(crowd_neighbor_counts(637, 638));
            // Every ported town kind is category 0, so the live roster is
            // unaffected until a pet kind ships (the filter is armed).
            assert!(crowd_neighbor_counts(22, 550));
        }

        /// `BiomePreferenceListTrait.ModifyShopPrice` (`:38-53`) tests the
        /// PLAYER's zones (`info.player`, `:44`), NOT the NPC's home tile.
        /// A Guide housed in the forest but with the PLAYER standing in the
        /// hallow gets no Forest Like (the player is not in the forest) —
        /// the Guide's rows are Like Forest / Dislike Ocean, neither matches
        /// the hallow, so no biome factor lands.
        #[test]
        fn the_biome_judge_reads_the_player_not_the_home() {
            let mut w = forest_world();
            // The hallow around the PLAYER's centre (400, 110), above the
            // surface and in the non-beach band (x∈[380,620]); the Guide's
            // home (500, 120) stays forest dirt.
            hallow_patch(&mut w, 400, 110);
            let guide = housed_guide(&w, (500, 120));
            let mood = process_mood(
                &w,
                std::slice::from_ref(&guide),
                &guide,
                (400.0 * 16.0, 110.0 * 16.0),
                false,
                &mut no_roll(),
            );
            // The player is in the hallow → no Forest Like, no Ocean
            // Dislike; the Guide has no hallow row. LoveSpace ×0.95 alone.
            assert!(
                !mood
                    .flags
                    .iter()
                    .any(|f| matches!(f, MoodFlag::BiomeAffection(..))),
                "no biome row matches the hallow: {:?}",
                mood.flags
            );
            assert!(mood.flags.contains(&MoodFlag::LoveSpace));
            assert!(
                (mood.multiplier - 0.95).abs() < 0.001,
                "the multiplier {}",
                mood.multiplier
            );
        }

        /// The same Guide, player back in the forest: the Forest Like lands
        /// (the judge follows the player, not the home — the home never
        /// moved). The companion to the hallow witness above.
        #[test]
        fn the_biome_judge_forest_when_the_player_is_forest() {
            let mut w = forest_world();
            hallow_patch(&mut w, 400, 110);
            let guide = housed_guide(&w, (500, 120));
            // The player stands far from the hallow patch (x=600, still in
            // the non-beach band), in plain forest.
            let mood = process_mood(
                &w,
                std::slice::from_ref(&guide),
                &guide,
                (600.0 * 16.0, 110.0 * 16.0),
                false,
                &mut no_roll(),
            );
            assert!(mood.flags.contains(&MoodFlag::BiomeAffection(
                AffectionLevel::Like,
                BiomeKind::Forest
            )));
            // Forest Like ×0.94 × LoveSpace 0.95 = 0.893 → 0.89.
            assert!(
                (mood.multiplier - 0.89).abs() < 0.001,
                "the multiplier {}",
                mood.multiplier
            );
        }

        /// `BiomePreferenceListTrait.ModifyShopPrice` (`:41-48`) picks the
        /// HIGHEST affection among the matching biomes and applies ONE row
        /// (`ApplyPreference`). Santa's rows are Love Snow / Hate Desert
        /// (`PersonalityDatabasePopulator.cs:62-66`); a player in BOTH the
        /// snow and the desert takes the Love Snow alone (×0.88), NOT
        /// 0.88 × 1.12. The `AffectionLevel` ordering (Love 100 > Hate -100)
        /// drives the pick.
        #[test]
        fn the_biome_pick_is_the_highest_affection_single_row() {
            // The ordering values (`AffectionLevel.cs:5-8`).
            assert!(AffectionLevel::Love.affection_value() > AffectionLevel::Like.affection_value());
            assert!(AffectionLevel::Like.affection_value() > AffectionLevel::Dislike.affection_value());
            assert!(AffectionLevel::Dislike.affection_value() > AffectionLevel::Hate.affection_value());
            // Santa's two rows: Love Snow, Hate Desert.
            let santa = biome_preferences_by_id(142);
            assert_eq!(santa.len(), 2);
            assert_eq!(santa[0].level, AffectionLevel::Love);
            assert_eq!(santa[0].biome, BiomeKind::Snow);
            assert_eq!(santa[1].level, AffectionLevel::Hate);
            assert_eq!(santa[1].biome, BiomeKind::Desert);
            // A scene that is BOTH snow and desert: the pick is Love Snow.
            // (batch-L re-pin: the merged judge is f37's `shopping_zones`/
            // `biome_in` snapshot — the g58 `biome_is_in(pm)` twin dropped.)
            let mut pm = tr_core::scene::SceneMetrics::new();
            pm.zone_snow = true;
            pm.zone_desert = true;
            let w = forest_world();
            let zones = shopping_zones(&pm, (500.0 * 16.0, 110.0 * 16.0), &w);
            let mut best: Option<&BiomePreference> = None;
            for pref in santa {
                if biome_in(&zones, pref.biome)
                    && best.is_none_or(|b| b.level.affection_value() < pref.level.affection_value())
                {
                    best = Some(pref);
                }
            }
            let pick = best.expect("a row matches");
            assert_eq!(pick.level, AffectionLevel::Love);
            assert_eq!(pick.biome, BiomeKind::Snow);
            assert_eq!(pick.level.multiplier(), 0.88, "ONE row, not 0.88×1.12");
        }

        /// A scripted `Main.rand` stand-in (batch-L re-pin): a
        /// `CountingRandom` wrapper is not available on the merged core's
        /// `&mut UnifiedRandom` signature, so the scripted-draw witnesses
        /// ride f37's stream-replay shape —
        /// `the_princess_picks_dedup_by_type_and_ride_the_global_stream`
        /// above pins the draw order/count against a mirrored
        /// `UnifiedRandom`; this g58 twin pins the ASCENDING-list +
        /// no-replacement semantics with a seed whose first three
        /// `Next(3)/Next(2)/Next(1)` draws are all 0 (the [0,0,0] script).
        fn zero_head_seed() -> i32 {
            let mut seed = 0;
            loop {
                let mut r = tr_core::UnifiedRandom::new(seed);
                if r.next_max(3) == 0 && r.next_max(2) == 0 && r.next_max(1) == 0 {
                    return seed;
                }
                seed += 1;
            }
        }

        /// `ShopHelper.cs:122-130` — the Princess's HateLonely gate: fewer
        /// than 2 within-house AND fewer than 2 within-village neighbors →
        /// the 1000 sentinel (the 1.5 ceiling) + the HateLonely row. She
        /// also gets NO LoveSpace (`flag = false`, `:124`) and the crowd
        /// multiply is ×1 (`num = 1f`, `:125`).
        #[test]
        fn the_princess_hates_lonely() {
            let w = forest_world();
            // A housed Princess alone — 0 within-house, 0 within-village.
            let mut princess = Npc::new(NpcKind::Princess, 500.0 * 16.0, 118.0 * 16.0, 1, 0.0);
            princess.home = Some((500, 120));
            princess.hp = princess.max_hp;
            let mood = process_mood(
                &w,
                std::slice::from_ref(&princess),
                &princess,
                (500.0 * 16.0, 110.0 * 16.0),
                false,
                &mut no_roll(),
            );
            assert!(mood.flags.contains(&MoodFlag::HateLonely));
            assert!(!mood.flags.contains(&MoodFlag::LoveSpace), "flag = false (`:124`)");
            assert!((mood.multiplier - 1.5).abs() < 0.001, "the sentinel ceiling");
        }

        /// `AllPersonalitiesModifier.cs:15-36` — the Princess's random
        /// three-loves head: the list is the nearby resident TYPES in
        /// ascending NPCID order (deduped), three picks each
        /// `Main.rand.Next(list.Count)` + `RemoveAt` (no replacement), each
        /// a Love ×0.88. With 3 neighbors and draws [0,0,0] the picks are
        /// the ascending list's first, then first-of-remainder, etc.
        #[test]
        fn the_princess_three_loves_head_draws_the_global_rng() {
            let w = forest_world();
            let mut princess = Npc::new(NpcKind::Princess, 500.0 * 16.0, 118.0 * 16.0, 1, 0.0);
            princess.home = Some((500, 120));
            princess.hp = princess.max_hp;
            // Three housed neighbors within 25 tiles: the Nurse (18), the
            // Merchant (17), the Guide (22). Ascending NPCID: 17, 18, 22.
            let mut all = vec![princess.clone()];
            for (kind, dx) in [
                (NpcKind::Nurse, 4),
                (NpcKind::Merchant, -4),
                (NpcKind::Guide, 0),
            ] {
                let mut n = Npc::new(kind, (500 + dx) as f32 * 16.0, 118.0 * 16.0, 1, 0.0);
                n.home = Some((500 + dx, 120));
                n.hp = n.max_hp;
                all.push(n);
            }
            // within_house = 3 → no HateLonely (needs < 2). The zero-head
            // seed draws [0,0,0]: pick list[0]=17 (Merchant), then
            // list[0]=18 (Nurse), then list[0]=22 (Guide).
            let mut rng = tr_core::UnifiedRandom::new(zero_head_seed());
            let mood = process_mood(
                &w,
                &all,
                &all[0],
                (500.0 * 16.0, 110.0 * 16.0),
                false,
                &mut rng,
            );
            let loves: Vec<i32> = mood
                .flags
                .iter()
                .filter_map(|f| match f {
                    MoodFlag::PrincessLove(k) => Some(*k),
                    _ => None,
                })
                .collect();
            assert_eq!(
                loves,
                vec![17, 18, 22],
                "ascending NPCID list, no replacement"
            );
            // 3 Love ×0.88 = 0.681472; the Princess has no biome row and no
            // LoveSpace; the clamp floor 0.75 binds → 0.75.
            assert!(
                (mood.multiplier - 0.75).abs() < 0.001,
                "the multiplier {}",
                mood.multiplier
            );
            // The crowd is ×1 for her: 3 within-house is NOT > 3, so no
            // crowd row; even past 3 the ×1 keeps the price flat.
            assert!(!mood.flags.contains(&MoodFlag::LoveSpace));
        }

        /// The head stops at the list's end (`:27-30` `list.Count <= 0`
        /// break): a Princess with ONE neighbor draws once, not three times.
        #[test]
        fn the_princess_head_stops_at_the_list_end() {
            let w = forest_world();
            let mut princess = Npc::new(NpcKind::Princess, 500.0 * 16.0, 118.0 * 16.0, 1, 0.0);
            princess.home = Some((500, 120));
            princess.hp = princess.max_hp;
            let mut nurse = Npc::new(NpcKind::Nurse, 504.0 * 16.0, 118.0 * 16.0, 1, 0.0);
            nurse.home = Some((504, 120));
            nurse.hp = nurse.max_hp;
            let all = vec![princess.clone(), nurse];
            // within_house = 1 < 2 AND within_village = 0 < 2 → HateLonely
            // sentinel, but the head STILL draws (the traits run
            // unconditionally, `:164-168`). One neighbor → one pick.
            // batch-L re-pin: the draw count rides the stream-replay twin
            // (a `Next(1)` still consumes the stream — one draw, then the
            // list-empty break).
            let mut rng = tr_core::UnifiedRandom::new(0);
            let mut expect = tr_core::UnifiedRandom::new(0);
            let _ = expect.next_max(1);
            let mood = process_mood(
                &w,
                &all,
                &all[0],
                (500.0 * 16.0, 110.0 * 16.0),
                false,
                &mut rng,
            );
            assert_eq!(rng, expect, "one neighbor → one draw, then the break");
            let loves: Vec<i32> = mood
                .flags
                .iter()
                .filter_map(|f| match f {
                    MoodFlag::PrincessLove(k) => Some(*k),
                    _ => None,
                })
                .collect();
            assert_eq!(loves, vec![18]);
            assert!(mood.flags.contains(&MoodFlag::HateLonely));
        }

        /// `LoveNPCByTypeName` (`ShopHelper.cs:243-250`): the Princess's
        /// pick report key is `LoveNPC_{NPCID.Search.GetName(npcType)}` —
        /// e.g. `TownNPCMood_Princess.LoveNPC_Guide` — NOT the plain
        /// `LoveNPC`. The picked neighbor's full name substitutes.
        #[test]
        fn the_princess_love_report_uses_the_named_key() {
            let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
            crate::lang::init(tr_core::localization::GameCulture::English);
            let flags = [MoodFlag::PrincessLove(22)];
            let report = happiness_report(NpcKind::Princess, false, &flags, &[]);
            let expected = crate::lang::text_format_with(
                "TownNPCMood_Princess.LoveNPC_Guide",
                &[(
                    "NPCName",
                    tr_core::localization::Substitution::Text(crate::lang::npc_name(22)),
                )],
            );
            assert_eq!(report, format!("{expected} "));
            // The plain LoveNPC row is a DIFFERENT string ("I love {NPCName}.")
            // — the named key is the dedicated "{NPCName} is a really nice guy!".
            let plain = crate::lang::text_format_with(
                "TownNPCMood_Princess.LoveNPC",
                &[(
                    "NPCName",
                    tr_core::localization::Substitution::Text(crate::lang::npc_name(22)),
                )],
            );
            assert_ne!(report, format!("{plain} "));
        }

        /// `AffectionLevel.cs:5-8` + `ShopHelper.cs:35-42` — the four
        /// multipliers.
        #[test]
        fn the_affection_multipliers() {
            assert_eq!(AffectionLevel::Love.multiplier(), 0.88);
            assert_eq!(AffectionLevel::Like.multiplier(), 0.94);
            assert_eq!(AffectionLevel::Dislike.multiplier(), 1.06);
            assert_eq!(AffectionLevel::Hate.multiplier(), 1.12);
        }

        /// The Underground row's key is `NormalUnderground`
        /// (`UndergroundBiome.cs:7`), not the enum's name.
        #[test]
        fn the_biome_name_keys() {
            assert_eq!(biome_name_key(BiomeKind::Underground), "NormalUnderground");
            assert_eq!(biome_name_key(BiomeKind::Mushroom), "Mushroom");
            assert_eq!(biome_name_key(BiomeKind::Forest), "Forest");
        }

        /// `AddHappinessReportText` (`ShopHelper.cs:188-196`): each row
        /// PREPENDS `text + report + " "` — the LAST flag reads first, and
        /// the substitutes land (`{BiomeName}`/`{NPCName}`).
        #[test]
        fn the_report_prepends_in_flag_order() {
            let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
            crate::lang::init(tr_core::localization::GameCulture::English);
            // The Guide's rows (`TownNPCMood_Guide`, en-US.Town.json
            // `:1854-1870`): LoveSpace then the Forest Like — the Like
            // reads FIRST (it was added last).
            let flags = [
                MoodFlag::LoveSpace,
                MoodFlag::BiomeAffection(AffectionLevel::Like, BiomeKind::Forest),
            ];
            let report = happiness_report(NpcKind::Guide, false, &flags, &[]);
            let like = crate::lang::text_format_with(
                "TownNPCMood_Guide.LikeBiome",
                &[(
                    "BiomeName",
                    tr_core::localization::Substitution::Text("the Forest".into()),
                )],
            );
            let space = crate::lang::text("TownNPCMood_Guide.LoveSpace");
            assert_eq!(report, format!("{like}{space} "));
            assert!(report.ends_with(' '), "the trailing space (`:195`)");
        }

        /// The neighbor row substitutes `NPC.GetFullnameByID` (`NPC.cs:
        /// 7010-7020`): a live resident answers `Game.NPCTitle` with the
        /// given name; nobody → the plain type name.
        #[test]
        fn the_report_names_the_neighbor() {
            let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
            crate::lang::init(tr_core::localization::GameCulture::English);
            let mut clothier =
                crate::npc::Npc::new(NpcKind::Clothier, 0.0, 0.0, 1, 0.0);
            clothier.hp = clothier.max_hp;
            clothier.given_name = "Isaac".into();
            let flags = [MoodFlag::NpcAffection(AffectionLevel::Like, 54)];
            let report = happiness_report(NpcKind::Guide, false, &flags, &[clothier]);
            let full = crate::lang::text_args(
                "Game.NPCTitle",
                &["Isaac".to_string(), crate::lang::npc_name(54)],
            );
            assert!(report.contains(&full), "the given-name title: {report}");
            // No resident → the type name alone.
            let bare = happiness_report(NpcKind::Guide, false, &flags, &[]);
            assert!(bare.contains(&crate::lang::npc_name(54)));
            assert!(!bare.contains("Isaac"));
        }

        /// The LikePrincess row's dedicated key (`ShopHelper.cs:252-259`,
        /// `TownNPCMood_Guide.LikeNPC_Princess` — en-US.Town.json `:1869`).
        #[test]
        fn the_princess_row_uses_its_own_key() {
            let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
            crate::lang::init(tr_core::localization::GameCulture::English);
            let flags = [MoodFlag::NpcAffection(AffectionLevel::Like, 663)];
            let report = happiness_report(NpcKind::Guide, false, &flags, &[]);
            let expected = crate::lang::text_format_with(
                "TownNPCMood_Guide.LikeNPC_Princess",
                &[(
                    "NPCName",
                    tr_core::localization::Substitution::Text(crate::lang::npc_name(663)),
                )],
            );
            assert_eq!(report, format!("{expected} "));
        }

        /// `CanShowHomelessText` (`NPC.cs:94828-94841`): homeless + not
        /// 37/368/453; a housed NPC never shows it.
        #[test]
        fn the_homeless_text_gate() {
            let mut g = crate::npc::Npc::new(NpcKind::Guide, 0.0, 0.0, 1, 0.0);
            g.home = None;
            assert!(can_show_homeless_text(Some(&g)));
            g.home = Some((10, 10));
            assert!(!can_show_homeless_text(Some(&g)));
            let mut tm = crate::npc::Npc::new(NpcKind::TravellingMerchant, 0.0, 0.0, 1, 0.0);
            tm.home = None;
            assert!(!can_show_homeless_text(Some(&tm)), "368 excluded (`:94836`)");
            assert!(!can_show_homeless_text(None), "npcIndex < 0 (`:94831`)");
        }

        /// `ShouldBestiaryGirlBeLycantrope` (`NPC.cs:93753-93764`): the
        /// shapeshift form — a blood-moon night, or a full-moon night; the
        /// day and the other moon phases are human. `UpdateAltTexture`
        /// (`:90225-90227`) writes `altTexture = 2` exactly here.
        #[test]
        fn the_lycantrope_predicate() {
            // A blood-moon night transforms regardless of the moon phase.
            assert!(should_bestiary_girl_be_lycantrope(true, false, 3));
            // A blood moon by day does NOT (the `|| dayTime` arm).
            assert!(!should_bestiary_girl_be_lycantrope(true, true, 0));
            // A full-moon night (phase 0) transforms.
            assert!(should_bestiary_girl_be_lycantrope(false, false, 0));
            // The other phases by night stay human.
            assert!(!should_bestiary_girl_be_lycantrope(false, false, 4));
            // Full moon by day stays human.
            assert!(!should_bestiary_girl_be_lycantrope(false, true, 0));
        }

        /// `AddHappinessReportText`'s `Transformed` suffix (`ShopHelper.cs:
        /// 191-194`): the Zoologist (633) in her shapeshift form reads the
        /// `TownNPCMood_BestiaryGirlTransformed` category; human reads
        /// `TownNPCMood_BestiaryGirl`. Other kinds never take the suffix.
        #[test]
        fn the_zoologist_transformed_suffix() {
            let _guard = crate::lang::test_lock().lock().unwrap_or_else(|e| e.into_inner());
            crate::lang::init(tr_core::localization::GameCulture::English);
            let flags = [MoodFlag::Content];
            // Human form — the plain category.
            let human = happiness_report(NpcKind::Zoologist, false, &flags, &[]);
            assert_eq!(
                human,
                format!("{} ", crate::lang::text("TownNPCMood_BestiaryGirl.Content"))
            );
            // Shapeshift form — the Transformed category ("Rrrr... feel fine.").
            let wolf = happiness_report(NpcKind::Zoologist, true, &flags, &[]);
            assert_eq!(
                wolf,
                format!(
                    "{} ",
                    crate::lang::text("TownNPCMood_BestiaryGirlTransformed.Content")
                )
            );
            assert_ne!(human, wolf, "the two forms read different rows");
            // A non-Zoologist never takes the suffix even when the flag is set.
            let guide = happiness_report(NpcKind::Guide, true, &flags, &[]);
            assert_eq!(
                guide,
                format!("{} ", crate::lang::text("TownNPCMood_Guide.Content"))
            );
        }
    }
}
