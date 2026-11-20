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

/// The per-NPC biome preferences (`PersonalityDatabasePopulator.
/// Populate_BiomePreferences`): the port carries the rows for the town NPCs
/// it has — the Guide (22, `:27-30`: likes the Forest, dislikes the Ocean).
/// The table grows with each town NPC (M4-TOWN-3+).
pub fn biome_preferences(kind: NpcKind) -> &'static [BiomePreference] {
    match kind {
        NpcKind::Guide => &[
            BiomePreference {
                level: AffectionLevel::Like,
                biome: BiomeKind::Forest,
            },
            BiomePreference {
                level: AffectionLevel::Dislike,
                biome: BiomeKind::Ocean,
            },
        ],
        // `Register(54)` (`PersonalityDatabasePopulator.cs:122-126`): the
        // Clothier likes the Underground, dislikes the Hallow.
        NpcKind::Clothier => &[
            BiomePreference {
                level: AffectionLevel::Like,
                biome: BiomeKind::Underground,
            },
            BiomePreference {
                level: AffectionLevel::Dislike,
                biome: BiomeKind::Hallow,
            },
        ],
        // `Register(124)` (`PersonalityDatabasePopulator.cs:52-56`):
        // the Mechanic likes the Snow, dislikes the Underground.
        NpcKind::Mechanic => &[
            BiomePreference {
                level: AffectionLevel::Like,
                biome: BiomeKind::Snow,
            },
            BiomePreference {
                level: AffectionLevel::Dislike,
                biome: BiomeKind::Underground,
            },
        ],
        // `Register(353)` (`PersonalityDatabasePopulator.cs:107-111`):
        // the Stylist likes the Ocean, dislikes the Snow.
        NpcKind::Stylist => &[
            BiomePreference {
                level: AffectionLevel::Like,
                biome: BiomeKind::Ocean,
            },
            BiomePreference {
                level: AffectionLevel::Dislike,
                biome: BiomeKind::Snow,
            },
        ],
        // `Register(207)` (`PersonalityDatabasePopulator.cs:67-71`):
        // the Dye Trader likes the Desert, dislikes the Forest.
        NpcKind::DyeTrader => &[
            BiomePreference {
                level: AffectionLevel::Like,
                biome: BiomeKind::Desert,
            },
            BiomePreference {
                level: AffectionLevel::Dislike,
                biome: BiomeKind::Forest,
            },
        ],
        // `Register(208)` (`PersonalityDatabasePopulator.cs:137-141`):
        // the Party Girl likes the Hallow, dislikes the Underground.
        NpcKind::PartyGirl => &[
            BiomePreference {
                level: AffectionLevel::Like,
                biome: BiomeKind::Hallow,
            },
            BiomePreference {
                level: AffectionLevel::Dislike,
                biome: BiomeKind::Underground,
            },
        ],
        // `Register(633)` (`PersonalityDatabasePopulator.cs:42-46`):
        // the Zoologist likes the Forest, dislikes the Desert.
        NpcKind::Zoologist => &[
            BiomePreference {
                level: AffectionLevel::Like,
                biome: BiomeKind::Forest,
            },
            BiomePreference {
                level: AffectionLevel::Dislike,
                biome: BiomeKind::Desert,
            },
        ],
        // `Register(588)` (`PersonalityDatabasePopulator.cs:37-41`):
        NpcKind::Golfer => &[
            BiomePreference {
                level: AffectionLevel::Like,
                biome: BiomeKind::Forest,
            },
            BiomePreference {
                level: AffectionLevel::Dislike,
                biome: BiomeKind::Underground,
            },
        ],
        // `Register(441)` (`PersonalityDatabasePopulator.cs:47-51`):
        // the Tax Collector likes the Snow, dislikes the Hallow.
        NpcKind::TaxCollector => &[
            BiomePreference {
                level: AffectionLevel::Like,
                biome: BiomeKind::Snow,
            },
            BiomePreference {
                level: AffectionLevel::Dislike,
                biome: BiomeKind::Hallow,
            },
        ],
        // `Register(108)` (`PersonalityDatabasePopulator.cs:127-131`):
        NpcKind::Wizard => &[
            BiomePreference {
                level: AffectionLevel::Like,
                biome: BiomeKind::Hallow,
            },
            BiomePreference {
                level: AffectionLevel::Dislike,
                biome: BiomeKind::Ocean,
            },
        ],
        // `Register(107)` (`PersonalityDatabasePopulator.cs:117-121`):
        // the Tinkerer likes the Underground, dislikes the Jungle.
        NpcKind::GoblinTinkerer => &[
            BiomePreference {
                level: AffectionLevel::Like,
                biome: BiomeKind::Underground,
            },
            BiomePreference {
                level: AffectionLevel::Dislike,
                biome: BiomeKind::Jungle,
            },
        ],
        // `Register(228)` (`PersonalityDatabasePopulator.cs:87-91`): the
        // Witch Doctor likes the Jungle, dislikes the Hallow.
        NpcKind::WitchDoctor => &[
            BiomePreference {
                level: AffectionLevel::Like,
                biome: BiomeKind::Jungle,
            },
            BiomePreference {
                level: AffectionLevel::Dislike,
                biome: BiomeKind::Hallow,
            },
        ],
        // `Register(227)` (`PersonalityDatabasePopulator.cs:92-96`): the
        // Painter likes the Jungle, dislikes the Forest.
        NpcKind::Painter => &[
            BiomePreference {
                level: AffectionLevel::Like,
                biome: BiomeKind::Jungle,
            },
            BiomePreference {
                level: AffectionLevel::Dislike,
                biome: BiomeKind::Forest,
            },
        ],
        // `Register(38)` (`PersonalityDatabasePopulator.cs:112-116`): the
        // Demolitionist likes the Underground, dislikes the Ocean.
        NpcKind::Demolitionist => &[
            BiomePreference {
                level: AffectionLevel::Like,
                biome: BiomeKind::Underground,
            },
            BiomePreference {
                level: AffectionLevel::Dislike,
                biome: BiomeKind::Ocean,
            },
        ],
        // `Register(19)` (`PersonalityDatabasePopulator.cs:72-76`): the
        // Arms Dealer likes the Desert, dislikes the Snow.
        NpcKind::ArmsDealer => &[
            BiomePreference {
                level: AffectionLevel::Like,
                biome: BiomeKind::Desert,
            },
            BiomePreference {
                level: AffectionLevel::Dislike,
                biome: BiomeKind::Snow,
            },
        ],
        // `Register(18)` (`PersonalityDatabasePopulator.cs:132-136`): the
        // Nurse likes the Hallow, dislikes the Snow.
        NpcKind::Nurse => &[
            BiomePreference {
                level: AffectionLevel::Like,
                biome: BiomeKind::Hallow,
            },
            BiomePreference {
                level: AffectionLevel::Dislike,
                biome: BiomeKind::Snow,
            },
        ],
        // `Register(20)` (`PersonalityDatabasePopulator.cs:82-86`): the
        // Dryad likes the Jungle, dislikes the Desert.
        NpcKind::Dryad => &[
            BiomePreference {
                level: AffectionLevel::Like,
                biome: BiomeKind::Jungle,
            },
            BiomePreference {
                level: AffectionLevel::Dislike,
                biome: BiomeKind::Desert,
            },
        ],
        // `Register(17)` (`PersonalityDatabasePopulator.cs:32-36`): the
        // Merchant likes the Forest, dislikes the Desert.
        NpcKind::Merchant => &[
            BiomePreference {
                level: AffectionLevel::Like,
                biome: BiomeKind::Forest,
            },
            BiomePreference {
                level: AffectionLevel::Dislike,
                biome: BiomeKind::Desert,
            },
        ],
        // `Register(369)` (`PersonalityDatabasePopulator.cs:97-101`): the
        // Angler likes the Ocean, dislikes the Desert.
        NpcKind::Angler => &[
            BiomePreference {
                level: AffectionLevel::Like,
                biome: BiomeKind::Ocean,
            },
            BiomePreference {
                level: AffectionLevel::Dislike,
                biome: BiomeKind::Desert,
            },
        ],
        // `Register(550)` (`PersonalityDatabasePopulator.cs:142-146`): the
        // Tavernkeep likes the Hallow, dislikes the Snow.
        NpcKind::DD2Bartender => &[
            BiomePreference {
                level: AffectionLevel::Like,
                biome: BiomeKind::Hallow,
            },
            BiomePreference {
                level: AffectionLevel::Dislike,
                biome: BiomeKind::Snow,
            },
        ],
        // The Princess (663) has NO `Register` row — her affection rides the
        // `AllPersonalitiesModifier` random-love picks alone
        // (`AllPersonalitiesModifier.cs:15-36`). The unported kinds' rows
        // (SantaClaus 142 `:62-66`, Truffle 160 `:147`, Steampunker 178
        // `:77-81`, Cyborg 209 `:57-61`, Pirate 229 `:102-106`) land with
        // their kinds.
        _ => &[],
    }
}

/// The neighbor-relationship rows (`AllPersonalitiesModifier.
/// ModifyShopPrice_Relationships`, `AllPersonalitiesModifier.cs:41-505`):
/// `(affection, vanilla NPC id)` pairs in the C# case's exact order — the
/// multiply order and the report order both ride it. Ids of unported kinds
/// (Steampunker 178, Cyborg 209, Pirate 229, Truffle 160, SantaClaus 142)
/// stay verbatim; they never match until those kinds land. The Guide DOES
/// carry rows (case 22, `:451-468` — the former "no case 22" note was wrong).
/// The Princess's own case has no switch rows — her picks are the random
/// loves above it (`:15-36`), handled in [`process_mood`].
pub fn neighbor_preferences(kind: NpcKind) -> &'static [(AffectionLevel, i32)] {
    use AffectionLevel::{Dislike, Hate, Like, Love};
    match kind {
        // case 17 (`:43-60`).
        NpcKind::Merchant => &[(Like, 588), (Like, 18), (Dislike, 441), (Hate, 369)],
        // case 18 (`:61-82`).
        NpcKind::Nurse => &[(Love, 19), (Like, 108), (Dislike, 208), (Dislike, 20), (Hate, 633)],
        // case 227 (`:83-100`).
        NpcKind::Painter => &[(Love, 20), (Like, 208), (Dislike, 209), (Dislike, 160)],
        // case 207 (`:101-118`).
        NpcKind::DyeTrader => &[(Like, 19), (Like, 227), (Dislike, 178), (Hate, 229)],
        // case 208 (`:119-140`).
        NpcKind::PartyGirl => &[(Love, 108), (Like, 353), (Dislike, 17), (Hate, 441), (Love, 633)],
        // case 369 (`:141-158`).
        NpcKind::Angler => &[(Like, 208), (Like, 38), (Like, 441), (Hate, 550)],
        // case 353 (`:159-176`).
        NpcKind::Stylist => &[(Love, 207), (Like, 229), (Dislike, 550), (Hate, 107)],
        // case 38 (`:177-194`).
        NpcKind::Demolitionist => &[(Love, 550), (Like, 124), (Dislike, 107), (Dislike, 19)],
        // case 20 (`:195-212`).
        NpcKind::Dryad => &[(Like, 228), (Like, 160), (Dislike, 369), (Hate, 588)],
        // case 550 (`:213-230`).
        NpcKind::DD2Bartender => &[(Love, 38), (Like, 107), (Dislike, 22), (Hate, 207)],
        // case 19 (`:231-248`).
        NpcKind::ArmsDealer => &[(Love, 18), (Like, 178), (Dislike, 588), (Hate, 38)],
        // case 107 (`:249-266`).
        NpcKind::GoblinTinkerer => &[(Love, 124), (Like, 207), (Dislike, 54), (Hate, 353)],
        // case 228 (`:267-284`).
        NpcKind::WitchDoctor => &[(Like, 20), (Like, 22), (Dislike, 18), (Hate, 160)],
        // case 54 (`:285-302`).
        NpcKind::Clothier => &[(Love, 160), (Like, 441), (Dislike, 18), (Hate, 124)],
        // case 124 (`:303-320`).
        NpcKind::Mechanic => &[(Love, 107), (Like, 209), (Dislike, 19), (Hate, 54)],
        // case 441 (`:321-342`).
        NpcKind::TaxCollector => &[(Love, 17), (Like, 208), (Dislike, 38), (Dislike, 124), (Hate, 142)],
        // case 108 (`:361-378`).
        NpcKind::Wizard => &[(Love, 588), (Like, 17), (Dislike, 228), (Hate, 209)],
        // case 588 (`:429-450`).
        NpcKind::Golfer => &[(Like, 227), (Love, 369), (Hate, 17), (Dislike, 229), (Like, 633)],
        // case 22 (`:451-468`).
        NpcKind::Guide => &[(Like, 54), (Dislike, 178), (Hate, 227), (Like, 633)],
        // case 633 (`:487-504`).
        NpcKind::Zoologist => &[(Dislike, 369), (Hate, 19), (Love, 228), (Like, 588)],
        // The Princess (663) has no switch case; the unported cases 229
        // (`:343-360`), 178 (`:379-400`), 209 (`:401-422`), 142 (`:423-428`)
        // and 160 (`:469-486`) land with their kinds.
        _ => &[],
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
    /// `HateLonely` (`ShopHelper.cs:128`) — the Princess alone
    /// (`npcsWithinHouse < 2 && npcsWithinVillage < 2`, `:126`).
    HateLonely,
    DislikeCrowded,
    HateCrowded,
    LoveSpace,
    Content,
    BiomeAffection(AffectionLevel, BiomeKind),
    /// The neighbor affection carries the VANILLA id — the rows reference
    /// unported kinds too (Steampunker 178, Cyborg 209, …) and the Princess's
    /// random loves pick from the whole nearby set.
    NpcAffection(AffectionLevel, i32),
}

/// The mood outcome — the multiplier rides the caller's shop math, the
/// flags feed the report UI.
#[derive(Clone, Debug)]
pub struct Mood {
    pub multiplier: f32,
    pub flags: Vec<MoodFlag>,
}

/// `ShopHelper.ProcessMood` (`ShopHelper.cs:95-174`) + the personality
/// traits (`BiomePreferenceListTrait.cs:38-53`, `AllPersonalitiesModifier.cs:
/// 12-506`). `rng` is the bare `Main.rand` stand-in (`app.rngs.global()`) —
/// the ONLY draw is the Princess's up-to-3 random loves (`:31`). The biome
/// traits judge the PLAYER's zones (`AShoppingBiome.IsInBiome(info.player)`),
/// not the NPC's home scene.
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
    // (the `LimitAndRoundMultiplier` tail never runs on this path). The
    // town pets (`NPCID.Sets.IsTownPet` — 637/638/656/670/678-684) have no
    // ported kinds; the roster grows here.
    let net_id = npc.kind.net_id();
    if world.seeds.contains(tr_core::seeds::SeedFlags::REMIX) || matches!(net_id, 37 | 368 | 453)
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
    if adjustment < 1000.0 && (pm.zone_corrupt || pm.zone_crimson || pm.zone_dungeon) {
        flags.push(MoodFlag::HateBiomeEvil);
        adjustment = 1000.0;
    }

    // `GetNearbyResidentNPCs` (`:288-325`): every active town NPC except the
    // not-really-town trio and the different-housing-category roommates
    // (`TownRoomManager.cs:153-161` — `housingCategory` differs; the pets'
    // category 1 has no ported kinds, so every ported town NPC is category
    // 0). Within 25 tiles of the home anchor = house; 25..120 = village.
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
            && best.is_none_or(|b| affection_rank(b.level) < affection_rank(pref.level))
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
    // its pick), then everyone-but-her likes a nearby Princess (`:37-40`),
    // then the per-kind switch rows (`:41-505`) in source order.
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
            flags.push(MoodFlag::NpcAffection(AffectionLevel::Love, id));
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

/// `AffectionLevel`'s enum values (`AffectionLevel.cs:5-8`) — the biome
/// trait's best-row comparison rides them, not the multiplier magnitudes.
fn affection_rank(level: AffectionLevel) -> i32 {
    match level {
        AffectionLevel::Love => 100,
        AffectionLevel::Like => 50,
        AffectionLevel::Dislike => -50,
        AffectionLevel::Hate => -100,
    }
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
        let mut w = World::new("t".into(), 200, 200);
        for x in 0..200 {
            for y in 120..200 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        // W4-f37: the surface line sits AT the dirt top — the biome traits
        // judge the PLAYER (`ShoppingZone_BelowSurface`, `Player.cs:3835`),
        // so a player standing on the dirt (tile y 110 < 120) is above the
        // surface and the Forest row holds.
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
        assert!(mood.flags.contains(&MoodFlag::HateBiomeEvil));
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
                .filter(|f| matches!(f, MoodFlag::NpcAffection(AffectionLevel::Love, _)))
                .count(),
            1
        );
        assert!(mood.flags.contains(&MoodFlag::NpcAffection(AffectionLevel::Love, 22)));
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
        assert!(mood.flags.contains(&MoodFlag::NpcAffection(AffectionLevel::Love, pick1)));
        assert!(mood.flags.contains(&MoodFlag::NpcAffection(AffectionLevel::Love, pick2)));
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
}
