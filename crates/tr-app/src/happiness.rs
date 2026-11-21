//! The town-NPC happiness and the shop prices — `ShopHelper.cs` +
//! `GameContent/Personalities/*`. The vanilla computes the price multiplier
//! when a shop opens (`Main.ShopHelper.OpenShop`): the mood factors (the
//! homelessness, the distance from home, the player's evil biomes, the
//! crowding) and the personality traits (the biome/neighbor affection)
//! multiply together, clamped to 0.75..1.5 and rounded to the cents.

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
/// report/localization keys land with the shop dialog (M4-TOWN-3).
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
        22 => &[bp(L, Forest), bp(D, Ocean)],      // `:27-31` the Guide
        17 => &[bp(L, Forest), bp(D, Desert)],     // `:32-36` the Merchant
        588 => &[bp(L, Forest), bp(D, Underground)], // `:37-41` the Golfer
        633 => &[bp(L, Forest), bp(D, Desert)],    // `:42-46` the Zoologist
        441 => &[bp(L, Snow), bp(D, Hallow)],      // `:47-51` the Tax Collector
        124 => &[bp(L, Snow), bp(D, Underground)], // `:52-56` the Mechanic
        209 => &[bp(L, Snow), bp(D, Jungle)],      // `:57-61` the Cyborg
        142 => &[bp(V, Snow), bp(H, Desert)],      // `:62-66` Santa Claus
        207 => &[bp(L, Desert), bp(D, Forest)],    // `:67-71` the Dye Trader
        19 => &[bp(L, Desert), bp(D, Snow)],       // `:72-76` the Arms Dealer
        178 => &[bp(L, Desert), bp(D, Jungle)],    // `:77-81` the Steampunker
        20 => &[bp(L, Jungle), bp(D, Desert)],     // `:82-86` the Dryad
        228 => &[bp(L, Jungle), bp(D, Hallow)],    // `:87-91` the Witch Doctor
        227 => &[bp(L, Jungle), bp(D, Forest)],    // `:92-96` the Painter
        369 => &[bp(L, Ocean), bp(D, Desert)],     // `:97-101` the Angler
        229 => &[bp(L, Ocean), bp(D, Underground)], // `:102-106` the Pirate
        353 => &[bp(L, Ocean), bp(D, Snow)],       // `:107-111` the Stylist
        38 => &[bp(L, Underground), bp(D, Ocean)], // `:112-116` the Demolitionist
        107 => &[bp(L, Underground), bp(D, Jungle)], // `:117-121` the Tinkerer
        54 => &[bp(L, Underground), bp(D, Hallow)], // `:122-126` the Clothier
        108 => &[bp(L, Hallow), bp(D, Ocean)],     // `:127-131` the Wizard
        18 => &[bp(L, Hallow), bp(D, Snow)],       // `:132-136` the Nurse
        208 => &[bp(L, Hallow), bp(D, Underground)], // `:137-141` the Party Girl
        550 => &[bp(L, Hallow), bp(D, Snow)],      // `:142-146` the Tavernkeep
        160 => &[bp(L, Mushroom)],                 // `:147` the Truffle
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

/// The kind-keyed view — the neighbor ids map back through the port's
/// `cs_id` roster; a row for an UNPORTED kind drops out (the vanilla can
/// never have that neighbor either — the kind does not exist).
pub fn neighbor_preferences(kind: NpcKind) -> &'static [(AffectionLevel, NpcKind)] {
    // The reverse `cs_id` table for the town roster — built once, the
    // switch's rows resolve at first read.
    use std::sync::OnceLock;
    static ROWS: OnceLock<Vec<&'static [(AffectionLevel, NpcKind)]>> = OnceLock::new();
    static IDS: &[NpcKind] = &[
        NpcKind::Guide,
        NpcKind::Merchant,
        NpcKind::Nurse,
        NpcKind::ArmsDealer,
        NpcKind::Dryad,
        NpcKind::Demolitionist,
        NpcKind::Clothier,
        NpcKind::GoblinTinkerer,
        NpcKind::Angler,
        NpcKind::Wizard,
        NpcKind::Mechanic,
        NpcKind::Stylist,
        NpcKind::TaxCollector,
        NpcKind::Golfer,
        NpcKind::Zoologist,
        NpcKind::Princess,
        NpcKind::DyeTrader,
        NpcKind::PartyGirl,
        NpcKind::Painter,
        NpcKind::WitchDoctor,
        // The Tavernkeep (550) — `NPCID.DD2Bartender` IS the town Tavernkeep
        // (`NPC.cs:16261-16274`: `townNPC = true`, aiStyle 7). There is NO
        // separate event twin: NPCID 109 is the Clown, and 109 is only the
        // port's wire id (`to_u16`), not his `cs_id` (which resolves 550
        // through the bestiary). His rows + the rows that name him land.
        NpcKind::DD2Bartender,
    ];
    let rows = ROWS.get_or_init(|| {
        IDS.iter()
            .map(|k| {
                let v: Vec<(AffectionLevel, NpcKind)> = neighbor_preferences_by_id(k.cs_id())
                    .iter()
                    .filter_map(|(lv, id)| {
                        IDS.iter()
                            .find(|o| o.cs_id() == *id)
                            .map(|o| (*lv, *o))
                    })
                    .collect();
                // Leaked ONCE at first read — the &'static contract the
                // old hand-table carried.
                &*Box::leak(v.into_boxed_slice())
            })
            .collect()
    });
    let idx = IDS.iter().position(|k| *k == kind);
    match idx {
        Some(i) => rows[i],
        None => &[],
    }
}

/// The resolved mood flags — the report-text keys land with the shop dialog
/// (`AddHappinessReportText`'s categories; the trait rows carry the
/// affection payload for the `LikeBiome`/`LikeNPC` text families).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoodFlag {
    NoHome,
    FarFromHome,
    HateBiomeEvil,
    /// `IsPlayerInEvilBiomes`' named row (`ShopHelper.cs:353-356`) — the
    /// evil biome carries its `BiomeName` substitute; supersedes the
    /// name-less `HateBiomeEvil` marker in the mood pass (the marker stays
    /// for the price-side callers/tests).
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
    NpcAffection(AffectionLevel, NpcKind),
    /// The Princess's random three-loves head (`AllPersonalitiesModifier.cs:
    /// 15-36`): each pick is a Love ×0.88 whose report key is the dedicated
    /// `LoveNPC_{NPCID.Search.GetName(npcType)}` (`ShopHelper.LoveNPCByTypeName`,
    /// `:243-250`), NOT the plain `LoveNPC`. The kind carries the name.
    PrincessLove(NpcKind),
}

/// The mood outcome — the multiplier rides the caller's shop math, the
/// flags feed the report UI.
#[derive(Clone, Debug)]
pub struct Mood {
    pub multiplier: f32,
    pub flags: Vec<MoodFlag>,
}

/// `ShopHelper.ProcessMood` (`ShopHelper.cs:95-174`) + the personality
/// traits. `raining`/`sandstorm_happening` ride the scene scan's zone
/// calculation like the player-side scans. The loveStruck ×0.9 rides the
/// buff pass (the port's buff 87 hook). `roll` stands in for the bare
/// `Main.rand.Next(n)` the Princess's random three-loves head draws
/// (`AllPersonalitiesModifier.cs:31`) — the call chain runs outside every
/// `SwapRandom` stage (`SetTalkNPC` ← `HoverOverNPCs` ← `DrawMouseOver` ←
/// `DrawInterface_39_MouseOver`, `Main.cs:43081`/`:42939`/`:45150`), so the
/// caller threads `NamedRngs::global()`. Non-Princess judges never draw.
pub fn process_mood(
    world: &World,
    all_npcs: &[crate::npc::Npc],
    npc: &crate::npc::Npc,
    player_center: (f32, f32),
    raining: bool,
    roll: &mut dyn FnMut(i32) -> i32,
) -> Mood {
    let mut flags: Vec<MoodFlag> = Vec::new();
    // `:97-98`: the reset.
    let mut adjustment = 1.0f32;
    // `:103-106`: the exclusions — the remix world, the two travelling
    // sellers and the town pets return before any mood work. The
    // Travelling Merchant is the port's 368 (`npc.type == 368`), so his
    // prices ride the plain `PriceAdjustment = 1f` with an empty report
    // — no homelessness sentinel, no crowd, no biome prefs.
    if npc.kind == crate::npc::NpcKind::TravellingMerchant {
        return Mood {
            multiplier: 1.0,
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

    // `:115-118`: the player in the evil biomes (`:28-34`'s three) pushes
    // the sentinel too.
    if adjustment < 1000.0 {
        let pm = scan_scene_at(
            world,
            [player_center.0, player_center.1],
            raining,
            false,
            false,
        );
        if pm.zone_corrupt || pm.zone_crimson || pm.zone_dungeon {
            // `_dangerousBiomes` order (`:28-34`): Corruption first, then
            // Crimson, then Dungeon — the first hit names the report row.
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
    }

    // `:119-156`: the crowd — the resident neighbors within 25 tiles of the
    // home anchor (the 120-tile village ring counts for LoveSpace only).
    let mut within_house = 0u32;
    let mut within_village = 0u32;
    let mut nearby_types: Vec<NpcKind> = Vec::new();
    for other in all_npcs {
        if other.hp <= 0 || std::ptr::eq(other, npc) {
            continue;
        }
        // `:305` — `nPC.active && nPC.townNPC && !IsNotReallyTownNPC(nPC)`
        // (`:363-371`: 37 OldMan / 368 Travelling Merchant / 453 Skeleton
        // Merchant). The port's `is_town_resident` carries the `townNPC`
        // flag but omits the Tavernkeep and includes the Travelling
        // Merchant, so the gate re-derives the C# pair: admit the
        // Tavernkeep (550 — `NPCID.DD2Bartender` IS the town Tavernkeep,
        // `NPC.cs:16261-16274`; no separate event twin exists, NPCID 109
        // is the Clown / the port's wire id) and drop the Travelling
        // Merchant (the only ported `IsNotReallyTownNPC` kind). The pets'
        // housingCategory filter (`CanNPCsLiveWithEachOther_ShopHelper`,
        // `TownRoomManager.cs:153-161`) is inert — no pets ported.
        let town_npc = other.kind.is_town_resident() || other.kind == NpcKind::DD2Bartender;
        if !town_npc || other.kind == NpcKind::TravellingMerchant {
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
            nearby_types.push(other.kind);
            within_house += 1;
        } else if dist < 120.0 {
            within_village += 1;
        }
    }
    // `:120-131`: the Princess's crowd overrides — `flag = false` (no
    // LoveSpace), `num = 1f` (the crowd multiply is inert), and the
    // HateLonely sentinel when fewer than 2 within-house AND fewer than 2
    // within-village neighbors.
    let is_princess = npc.kind == NpcKind::Princess;
    let love_space_ok = !is_princess;
    let crowd_mult = if is_princess { 1.0 } else { 1.05 };
    if is_princess && within_house < 2 && within_village < 2 {
        flags.push(MoodFlag::HateLonely);
        adjustment = 1000.0;
    }
    if within_house > 3 {
        for _ in 3..within_house {
            adjustment *= crowd_mult;
        }
        flags.push(if within_house > 6 {
            MoodFlag::HateCrowded
        } else {
            MoodFlag::DislikeCrowded
        });
    }
    if love_space_ok && within_house <= 2 && within_village < 4 {
        flags.push(MoodFlag::LoveSpace);
        adjustment *= 0.95;
    }

    // `:152-168`: the personality traits run UNCONDITIONALLY (the C# has
    // no `< 1000` gate here — the sentinel rides the clamp at `:178`, but
    // the report rows + the Princess's RNG draws still land).
    // `BiomePreferenceListTrait.ModifyShopPrice` (`:38-53`) tests the
    // PLAYER's zones (`info.player`, `:44`) and picks the HIGHEST affection
    // among the matching biomes, applying ONE row (`ApplyPreference`,
    // `:55-73`) — NOT the home tile and NOT every matching row.
    {
        let pm = scan_scene_at(
            world,
            [player_center.0, player_center.1],
            raining,
            false,
            false,
        );
        // `:41-48` — the highest-affection pick among the biomes the player
        // is in; the first row wins a tie (the `<` compare keeps the
        // earlier row on equal affection).
        let mut best: Option<&BiomePreference> = None;
        for pref in biome_preferences(npc.kind) {
            if biome_is_in(pref.biome, &pm)
                && best.is_none_or(|b| b.level.affection_value() < pref.level.affection_value())
            {
                best = Some(pref);
            }
        }
        if let Some(pref) = best {
            flags.push(MoodFlag::BiomeAffection(pref.level, pref.biome));
            adjustment *= pref.level.multiplier();
        }
        // `AllPersonalitiesModifier.ModifyShopPrice_Relationships` (`:12-
        // 505`): the Princess's random three-loves head FIRST (`:15-36`),
        // then the universal LikePrincess row (`:37-40`), then the static
        // switch (`:41-505`).
        //
        // The Princess head (`:15-36`): the list is the nearby resident
        // TYPES in ascending NPCID order (`nearbyNPCsByType` is a
        // `bool[NPCID.Count]`, `:152-156`, so the `for i` walk is ascending
        // and deduped by type); three picks, each `Main.rand.Next(list.Count)`
        // then `RemoveAt` (no replacement), each a Love ×0.88 whose report
        // key is `LoveNPC_{name}` (`LoveNPCByTypeName`, `:243-250`).
        if is_princess {
            let mut list: Vec<NpcKind> = nearby_types.clone();
            list.sort_by_key(|k| k.cs_id());
            list.dedup();
            for _ in 0..3 {
                if list.is_empty() {
                    break;
                }
                let index = roll(list.len() as i32) as usize;
                let kind = list.remove(index);
                flags.push(MoodFlag::PrincessLove(kind));
                adjustment *= AffectionLevel::Love.multiplier();
            }
        }
        // The universal LikePrincess row (`:37-40`): every NON-Princess
        // town NPC likes a nearby Princess (×0.94).
        if !is_princess && nearby_types.contains(&NpcKind::Princess) {
            flags.push(MoodFlag::NpcAffection(AffectionLevel::Like, NpcKind::Princess));
            adjustment *= AffectionLevel::Like.multiplier();
        }
        // The static switch (`:41-505`): the NpcAffection flags feed the
        // report UI (the row order is the C# `if` order). The Princess has
        // no static rows (`neighbor_preferences_by_id(663)` is empty).
        for (level, other_kind) in neighbor_preferences(npc.kind) {
            if nearby_types.iter().any(|t| t == other_kind) {
                flags.push(MoodFlag::NpcAffection(*level, *other_kind));
                adjustment *= level.multiplier();
            }
        }
    }

    if flags.is_empty() {
        flags.push(MoodFlag::Content);
    }
    // `:173-181`: the clamp 0.75..1.5 and the cents rounding.
    let multiplier = (adjustment.clamp(0.75, 1.5) * 100.0).round() / 100.0;
    Mood { multiplier, flags }
}

/// `AShoppingBiome.IsInBiome(player)` per biome — each reads one player
/// zone flag (`Player.Zone*` = `SceneMetrics.Zone*`, `Player.cs:16705-
/// 16713`), except the two shopping-zone properties:
/// * Forest (`ForestBiome.cs:10-13`): `ShoppingZone_Forest` =
///   `!ShoppingZone_AnyBiome && !ShoppingZone_BelowSurface`
///   (`Player.cs:3837-3847`); `ShoppingZone_AnyBiome` (`:3823-3833`) is
///   the nine-zone OR (Dungeon/Corrupt/Crimson/Glowshroom/Hallow/Jungle/
///   Snow/Beach/Desert).
/// * Underground (`UndergroundBiome.cs:10-13`): `ShoppingZone_BelowSurface`
///   = `Center.ToTileCoordinates().Y > Main.worldSurface`
///   (`Player.cs:3835`) — the scan's `below_surface`.
/// * Mushroom (`MushroomBiome.cs:10-13`): `ZoneGlowshroom` =
///   `EnoughTilesForGlowingMushroom` (`SceneMetrics.cs:677`).
fn biome_is_in(biome: BiomeKind, pm: &tr_core::scene::SceneMetrics) -> bool {
    match biome {
        BiomeKind::Forest => {
            let any_biome = pm.zone_dungeon
                || pm.zone_corrupt
                || pm.zone_crimson
                || pm.enough_tiles_for_glowing_mushroom()
                || pm.zone_hallow
                || pm.zone_jungle
                || pm.zone_snow
                || pm.zone_beach
                || pm.zone_desert;
            !any_biome && !pm.below_surface
        }
        BiomeKind::Ocean => pm.zone_beach,
        BiomeKind::Snow => pm.zone_snow,
        BiomeKind::Desert => pm.zone_desert,
        BiomeKind::Jungle => pm.zone_jungle,
        BiomeKind::Underground => pm.below_surface,
        BiomeKind::Hallow => pm.zone_hallow,
        BiomeKind::Mushroom => pm.enough_tiles_for_glowing_mushroom(),
        BiomeKind::Corruption => pm.zone_corrupt,
        BiomeKind::Crimson => pm.zone_crimson,
        BiomeKind::Dungeon => pm.zone_dungeon,
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

/// `NPC.CanShowHomelessText` (`NPC.cs:94828-94841`): the homeless NPC
/// that is not the Old Man (37) / Travelling Merchant (368) / Skeleton
/// Merchant (453). The port's homeless stand-in is `home == None` (the
/// `Npc.homeless` field); the Old Man / Skeleton Merchant kinds are
/// unported, the Travelling Merchant is.
pub fn can_show_homeless_text(npc: Option<&crate::npc::Npc>) -> bool {
    match npc {
        Some(n) => n.home.is_none() && n.kind != NpcKind::TravellingMerchant,
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
/// (`AllPersonalitiesModifier.cs:37-40` → `ShopHelper.cs:252-259`).
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
                let (key, name_kind) =
                    if *level == AffectionLevel::Like && *other == NpcKind::Princess {
                        ("LikeNPC_Princess", *other)
                    } else {
                        (
                            match level {
                                AffectionLevel::Love => "LoveNPC",
                                AffectionLevel::Like => "LikeNPC",
                                AffectionLevel::Dislike => "DislikeNPC",
                                AffectionLevel::Hate => "HateNPC",
                            },
                            *other,
                        )
                    };
                prepend_report(
                    &mut report,
                    kind,
                    transformed,
                    key,
                    &[("NPCName", npc_full_name(all_npcs, name_kind))],
                );
            }
            MoodFlag::PrincessLove(other) => {
                // `LoveNPCByTypeName` (`ShopHelper.cs:243-250`): the key is
                // `LoveNPC_{NPCID.Search.GetName(npcType)}` with the picked
                // neighbor's full name. The category is the Princess's own
                // (`kind`), the key names the picked neighbor.
                let Some(name) = crate::lang_tables::NPC_ID
                    .iter()
                    .find(|(id, _)| *id == other.cs_id() as i32)
                    .map(|(_, n)| *n)
                else {
                    continue;
                };
                prepend_report(
                    &mut report,
                    kind,
                    transformed,
                    &format!("LoveNPC_{name}"),
                    &[("NPCName", npc_full_name(all_npcs, *other))],
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
/// writes `altTexture = 2` on — `NPC.cs:90225-90227`). `roll` is the bare
/// `Main.rand` stand-in the Princess head draws (`NamedRngs::global()`).
pub fn shopping_settings(
    world: &World,
    all_npcs: &[crate::npc::Npc],
    npc: &crate::npc::Npc,
    player_center: (f32, f32),
    raining: bool,
    roll: &mut dyn FnMut(i32) -> i32,
) -> ShoppingSettings {
    let mood = process_mood(world, all_npcs, npc, player_center, raining, roll);
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

    /// A no-draw `Main.rand` stand-in for the non-Princess judges — their
    /// random three-loves head never runs, so the closure is never called.
    fn no_roll() -> impl FnMut(i32) -> i32 {
        |_n: i32| 0
    }

    /// The forest test ground: dirt from y = 120 on a 1000-wide world —
    /// no special zones. The width exceeds 2×`BEACH_DISTANCE` (380) so the
    /// middle columns (x≈500) are NOT ocean (`WorldGen.oceanDepths`,
    /// `WorldGen.cs:4447`); `worldSurface` 130 sits below the player row
    /// (y≈110) so `ShoppingZone_BelowSurface` is false and the Forest
    /// predicate (`!ShoppingZone_AnyBiome && !ShoppingZone_BelowSurface`,
    /// `Player.cs:3837-3847`) holds. The action centres on x = 500.
    fn forest_world() -> World {
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
        w.world_surface = 130.0;
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
            &mut no_roll(),
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
        let guide = housed_guide(&w, (500, 120));
        let mood = process_mood(
            &w,
            std::slice::from_ref(&guide),
            &guide,
            (500.0 * 16.0, 110.0 * 16.0),
            false,
            &mut no_roll(),
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
            &mut no_roll(),
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
            &mut no_roll(),
        );
        assert!((mood.multiplier - 1.5).abs() < 0.001);
        assert!(mood.flags.contains(&MoodFlag::HateBiomeNamed(BiomeKind::Corruption)));
    }

    #[test]
    fn the_crowd_multiplies_past_three() {
        // `:132-146`: the residents within 25 tiles past the third push
        // ×1.05 each; the LoveSpace gate closes (within_house > 2).
        let w = forest_world();
        let guide = housed_guide(&w, (500, 120));
        let mut all = vec![guide.clone()];
        for (dx, dy) in [(4, 0), (-4, 0), (0, -6), (7, 3)] {
            let mut other = Npc::spawn_guide(&w, 500 + dx).expect("the neighbor spawns");
            other.home = Some((500 + dx, 120 + dy));
            other.hp = other.max_hp;
            all.push(other);
        }
        let mood = process_mood(&w, &all, &all[0], (500.0 * 16.0, 110.0 * 16.0), false, &mut no_roll());
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
        let guide = housed_guide(&w, (500, 120));
        let mut all = vec![guide.clone()];
        for dx in [30, 60, 90, 110] {
            let mut other = Npc::spawn_guide(&w, 500 + dx).expect("the villager spawns");
            other.home = Some((500 + dx, 120));
            other.hp = other.max_hp;
            all.push(other);
        }
        let mood = process_mood(&w, &all, &all[0], (500.0 * 16.0, 110.0 * 16.0), false, &mut no_roll());
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

        /// The kind-keyed view resolves through `cs_id` and drops the
        /// unported neighbor kinds (the vanilla can never house them).
        #[test]
        fn the_kind_view_filters_the_unported_neighbors() {
            // The Guide (`:451-468`): Clothier like, Steampunker dislike
            // (UNPORTED — drops), Painter hate, Zoologist like.
            let guide = neighbor_preferences(NpcKind::Guide);
            assert_eq!(
                guide,
                &[
                    (L, NpcKind::Clothier),
                    (H, NpcKind::Painter),
                    (L, NpcKind::Zoologist),
                ]
            );
            // The Merchant: the Angler hate row survives (369 ports).
            let merchant = neighbor_preferences(NpcKind::Merchant);
            assert_eq!(merchant.len(), 4);
            assert_eq!(merchant[3], (H, NpcKind::Angler));
            // The Tax Collector's Santa hate row drops (142 unported).
            let tax = neighbor_preferences(NpcKind::TaxCollector);
            assert_eq!(tax.len(), 4);
            assert!(!tax.iter().any(|(_, k)| k.cs_id() == 142));
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
                &[
                    (V, NpcKind::Demolitionist),
                    (L, NpcKind::GoblinTinkerer),
                    (D, NpcKind::Guide),
                    (H, NpcKind::DyeTrader),
                ]
            );
            // The rows that NAME him survive: the Angler hates him (`:141-
            // 158`), the Demolitionist loves him (`:177-194`).
            let angler = neighbor_preferences(NpcKind::Angler);
            assert!(angler.contains(&(H, NpcKind::DD2Bartender)));
            let demo = neighbor_preferences(NpcKind::Demolitionist);
            assert!(demo.contains(&(V, NpcKind::DD2Bartender)));
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
            let mut pm = tr_core::scene::SceneMetrics::new();
            pm.zone_snow = true;
            pm.zone_desert = true;
            let mut best: Option<&BiomePreference> = None;
            for pref in santa {
                if biome_is_in(pref.biome, &pm)
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

        /// A scripted `Main.rand.Next(n)` stand-in: each call pops the next
        /// pre-seeded index (the C# `Main.rand.Next(list.Count)` draw) and
        /// asserts the index is in range. The draw-count witness uses a
        /// separate counting closure.
        fn scripted_roll(indices: &[i32]) -> impl FnMut(i32) -> i32 + '_ {
            let mut cursor = 0usize;
            move |n: i32| {
                let idx = indices.get(cursor).copied().unwrap_or(0);
                cursor += 1;
                assert!(idx < n, "the draw index {idx} must be < list.Count {n}");
                idx
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
            let mut roll = |_n: i32| 0;
            let mood = process_mood(
                &w,
                std::slice::from_ref(&princess),
                &princess,
                (500.0 * 16.0, 110.0 * 16.0),
                false,
                &mut roll,
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
            // within_house = 3 → no HateLonely (needs < 2). Draws [0,0,0]:
            // pick list[0]=17 (Merchant), then list[0]=18 (Nurse), then
            // list[0]=22 (Guide).
            let draws = [0, 0, 0];
            let mut roll = scripted_roll(&draws);
            let mood = process_mood(
                &w,
                &all,
                &all[0],
                (500.0 * 16.0, 110.0 * 16.0),
                false,
                &mut roll,
            );
            let loves: Vec<NpcKind> = mood
                .flags
                .iter()
                .filter_map(|f| match f {
                    MoodFlag::PrincessLove(k) => Some(*k),
                    _ => None,
                })
                .collect();
            assert_eq!(
                loves,
                vec![NpcKind::Merchant, NpcKind::Nurse, NpcKind::Guide],
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
            let mut draws = 0i32;
            let mut roll = |_n: i32| {
                draws += 1;
                0
            };
            let mood = process_mood(
                &w,
                &all,
                &all[0],
                (500.0 * 16.0, 110.0 * 16.0),
                false,
                &mut roll,
            );
            assert_eq!(draws, 1, "one neighbor → one draw, then the break");
            let loves: Vec<NpcKind> = mood
                .flags
                .iter()
                .filter_map(|f| match f {
                    MoodFlag::PrincessLove(k) => Some(*k),
                    _ => None,
                })
                .collect();
            assert_eq!(loves, vec![NpcKind::Nurse]);
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
            let flags = [MoodFlag::PrincessLove(NpcKind::Guide)];
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
            let flags = [MoodFlag::NpcAffection(AffectionLevel::Like, NpcKind::Clothier)];
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
            let flags = [MoodFlag::NpcAffection(AffectionLevel::Like, NpcKind::Princess)];
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
