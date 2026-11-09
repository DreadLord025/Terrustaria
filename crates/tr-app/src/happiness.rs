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
        // `Register(108)` (`PersonalityDatabasePopulator.cs:127-131`):
        // the Wizard likes the Hallow, dislikes the Ocean.
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
        _ => &[],
    }
}

/// The neighbor-relationship rows (`AllPersonalitiesModifier.
/// ModifyShopPrice_Relationships`): the Guide carries none (the switch has
/// no case 22 — the Guide judges the biome alone). The table grows with the
/// town NPCs.
pub fn neighbor_preferences(kind: NpcKind) -> &'static [(AffectionLevel, NpcKind)] {
    let _ = kind;
    &[]
}

/// The resolved mood flags — the report-text keys land with the shop dialog
/// (`AddHappinessReportText`'s categories; the trait rows carry the
/// affection payload for the `LikeBiome`/`LikeNPC` text families).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoodFlag {
    NoHome,
    FarFromHome,
    HateBiomeEvil,
    DislikeCrowded,
    HateCrowded,
    LoveSpace,
    Content,
    BiomeAffection(AffectionLevel, BiomeKind),
    NpcAffection(AffectionLevel, NpcKind),
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
/// buff pass (the port's buff 87 hook); the port's town set (the Guide)
/// skips the 663 Princess branches.
pub fn process_mood(
    world: &World,
    all_npcs: &[crate::npc::Npc],
    npc: &crate::npc::Npc,
    player_center: (f32, f32),
    raining: bool,
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
            flags.push(MoodFlag::HateBiomeEvil);
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
        if !matches!(other.kind, NpcKind::Guide) {
            // `IsNotReallyTownNPC` + the townNPC gate — the port's town set
            // grows here.
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
    if within_house > 3 {
        for _ in 3..within_house {
            adjustment *= 1.05;
        }
        flags.push(if within_house > 6 {
            MoodFlag::HateCrowded
        } else {
            MoodFlag::DislikeCrowded
        });
    }
    if within_house <= 2 && within_village < 4 {
        flags.push(MoodFlag::LoveSpace);
        adjustment *= 0.95;
    }

    // `:152-168`: the personality traits — the biome prefs judge the home
    // anchor's scene (`ShopHelper` reads `info.npc`'s surroundings through
    // the HelperInfo rows; the biome traits test the NPC's position).
    if adjustment < 1000.0 {
        let home_center = [home_x * 16.0, home_y * 16.0];
        let pm = scan_scene_at(world, home_center, raining, false, false);
        let home_biome = classify_biome(&pm);
        for pref in biome_preferences(npc.kind) {
            if pref.biome == home_biome {
                flags.push(MoodFlag::BiomeAffection(pref.level, pref.biome));
                adjustment *= pref.level.multiplier();
            }
        }
        // The neighbor prefs (`AllPersonalitiesModifier`): the Guide's rows
        // are empty; the table grows with the town NPCs.
        for (level, other_kind) in neighbor_preferences(npc.kind) {
            if nearby_types.iter().any(|t| t == other_kind) {
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

/// The shopping-biome classification at a scene centre — the vanilla
/// biome traits test the player/NPC zone flags (`AShoppingBiome.IsInBiome`):
/// the Forest row is the fallthrough (`ForestBiome.cs`'s
/// `IsInBiome = !any other zone`). The first matching non-Forest zone wins;
/// the scan's zone flags carry the truth.
fn classify_biome(pm: &tr_core::scene::SceneMetrics) -> BiomeKind {
    if pm.zone_corrupt {
        BiomeKind::Corruption
    } else if pm.zone_crimson {
        BiomeKind::Crimson
    } else if pm.zone_dungeon {
        BiomeKind::Dungeon
    } else if pm.zone_beach {
        BiomeKind::Ocean
    } else if pm.zone_snow {
        BiomeKind::Snow
    } else if pm.zone_desert {
        BiomeKind::Desert
    } else if pm.zone_jungle {
        BiomeKind::Jungle
    } else if pm.zone_hallow {
        BiomeKind::Hallow
    } else {
        BiomeKind::Forest
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
        w.world_surface = 30.0;
        w.rock_layer = 40.0;
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
        let mood = process_mood(&w, &all, &all[0], (60.0 * 16.0, 110.0 * 16.0), false);
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
        let mood = process_mood(&w, &all, &all[0], (60.0 * 16.0, 110.0 * 16.0), false);
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
}
