//! The town NPC naming layer — `NPC.getNewNPCName` (`NPC.cs:7276-7324`),
//! `NPC.GiveTownUniqueDataToNPCsThatNeedIt` (`NPC.cs:80839-80856`) and the
//! `FullName`/`GetFullNetName` pair (`NPC.cs:6729-6739`/`:7349-7356`).
//! W4-f38: the move-in announcements read the full name, so the arrival
//! writes it through these seams.
//!
//! Streams (the C# reads them verbatim): the name draw is
//! `Language.RandomFromCategory(<Category>, WorldGen.genRand)` — the port's
//! genRand stand-in is `World::frame_rng` (seeded from the world seed,
//! `world.rs:1196-1201`; the `framing.rs` precedent). The pet variation roll
//! is `Main.rand.Next(variants)` inside the `SpawnNPC` SwapRandom wrapper
//! (`NPC.NewNPC`, `NPC.cs:80750-80769`) — the caller passes the stream.

use crate::npc::Npc;
use tr_core::unified_random::UnifiedRandom;

/// `NPC.getNewNPCNameInner` (`NPC.cs:7281-7324`): the type → name category.
/// Every row the C# switch carries; the port ships no Truffle (160),
/// Steampunker (178), Cyborg (209), Pirate (229), Bartender (550), Santa or
/// Old Man kinds, but the table stays whole so a future kind names itself
/// without a second switch. The town pets (637/638/656) and town slimes
/// (670/678-684) answer their *variant 0* category here — the profile arm
/// ([`give_town_unique_data`]) re-rolls through the drawn variation.
pub fn name_category(npc_type: i32) -> Option<&'static str> {
    Some(match npc_type {
        17 => "MerchantNames",
        18 => "NurseNames",
        19 => "ArmsDealerNames",
        20 => "DryadNames",
        22 => "GuideNames",
        38 => "DemolitionistNames",
        54 => "ClothierNames",
        107 => "GoblinTinkererNames",
        108 => "WizardNames",
        124 => "MechanicNames",
        160 => "TruffleNames",
        178 => "SteampunkerNames",
        207 => "DyeTraderNames",
        208 => "PartyGirlNames",
        209 => "CyborgNames",
        227 => "PainterNames",
        228 => "WitchDoctorNames",
        229 => "PirateNames",
        353 => "StylistNames",
        368 => "TravelingMerchantNames",
        369 => "AnglerNames",
        453 => "SkeletonMerchantNames",
        441 => "TaxCollectorNames",
        550 => "BartenderNames",
        588 => "GolferNames",
        633 => "BestiaryGirlNames",
        663 => "PrincessNames",
        637 => "CatNames_Siamese",
        638 => "DogNames_Labrador",
        656 => "BunnyNames_White",
        670 => "SlimeNames_Blue",
        678 => "SlimeNames_Green",
        679 => "SlimeNames_Old",
        680 => "SlimeNames_Purple",
        681 => "SlimeNames_Rainbow",
        682 => "SlimeNames_Red",
        683 => "SlimeNames_Yellow",
        684 => "SlimeNames_Copper",
        _ => return None, // the C#'s `_ => ""` arm
    })
}

/// `NPC.getNewNPCName` (`NPC.cs:7276-7279`) →
/// `Language.RandomFromCategory(category, WorldGen.genRand)`
/// (`LanguageManager.cs:413-420`: one `Next(count)` draw, the category's
/// insertion-ordered keys). `gen_rand` is the caller's genRand stand-in
/// (`World::frame_rng`); a type outside the switch names "" like the C#.
pub fn new_npc_name(npc_type: i32, gen_rand: &mut UnifiedRandom) -> String {
    let Some(category) = name_category(npc_type) else {
        return String::new();
    };
    let count = crate::lang::category_size(category);
    if count == 0 {
        // The C# `RandomFromCategory` on an empty category would throw; the
        // shipped Town.json always carries the rows. A broken load answers
        // the manager's own `Category.RANDOM` placeholder instead.
        return crate::lang::random_from_category(category, 0);
    }
    let index = gen_rand.next_max(count as i32) as usize;
    crate::lang::random_from_category(category, index)
}

/// `Profiles.VariantNPCProfile`'s variant rosters (`TownNPCProfiles.cs:139-149`):
/// the six-variant pet families. The town slimes are `LegacyNPCProfile`s
/// (`:151-180`) — no variation roll, the type's own category names them.
fn pet_variants(npc_type: i32) -> Option<&'static [&'static str]> {
    Some(match npc_type {
        637 => &[
            "Siamese",
            "Black",
            "OrangeTabby",
            "RussianBlue",
            "Silver",
            "White",
        ],
        638 => &[
            "Labrador",
            "PitBull",
            "Beagle",
            "Corgi",
            "Dalmation",
            "Husky",
        ],
        656 => &["White", "Angora", "Dutch", "Flemish", "Lop", "Silver"],
        _ => return None,
    })
}

/// `NPC.GiveTownUniqueDataToNPCsThatNeedIt` (`NPC.cs:80839-80856`), the
/// `isLikeATownNPC` arm (`NPC.cs:6856-6864` = `townNPC || type == 453`):
/// `GivenName = getNewNPCName(Type)` first, then — when a profile exists —
/// `townNpcVariationIndex = profile.RollVariation()` and
/// `GivenName = profile.GetNameForVariant(npc)` (a SECOND name draw for the
/// legacy/transformable/variant profiles alike, `Profiles.cs:88-91`/
/// `:144-147`/`:219-222`; the first draw is consumed and discarded). The
/// legacy profiles roll variation 0 (`Profiles.cs:83-86`); the pet variant
/// profiles roll `Main.rand.Next(6)` (`:214-217`) — `main_rand` is the
/// caller's `SpawnNPC`-wrapper stream. Returns the drawn variation
/// (`townNpcVariationIndex`; the port stores no field — the name is the only
/// consumer the arrival announcements read).
///
/// Deviation (ANALYSIS "Town naming (W4-f38)"): the `ShimmeredTownNPCs[Type]`
/// override (`:80850-80853`) only rewrites the variation AFTER the name was
/// drawn from the rolled variant, and the legacy shimmer profiles name
/// through the same category — with no runtime shimmer transform ported the
/// flag can only ride a loaded save, where the name is already restored;
/// the override is inert for the name and skipped.
pub fn give_town_unique_data(
    npc: &mut Npc,
    main_rand: &mut UnifiedRandom,
    gen_rand: &mut UnifiedRandom,
) -> i32 {
    let ty = npc.kind.net_id();
    // `nPC.isLikeATownNPC` — the bound/sleeping forms are not town-like and
    // keep their empty name until the rescue transform re-runs this.
    if !npc.kind.is_town_npc_like() {
        return 0;
    }
    // `:80844`: the first draw, from the type's own (variant-0) category.
    npc.given_name = new_npc_name(ty, gen_rand);
    // Every town-like type the port ships has a profile
    // (`TownNPCProfiles.cs:20-180`); the slimes' legacy profiles roll 0.
    let variation = match pet_variants(ty) {
        Some(variants) => main_rand.next_max(variants.len() as i32),
        None => 0,
    };
    // `:80848` `GetNameForVariant`: the variant profile draws from
    // `<Base>Names_<variant>` (`Profiles.cs:221`), the legacy/transformable
    // profiles re-draw the type's own category (`:90`/`:146`).
    if let Some(variants) = pet_variants(ty) {
        let base = match ty {
            637 => "Cat",
            638 => "Dog",
            _ => "Bunny",
        };
        let category = format!("{base}Names_{}", variants[variation as usize]);
        let count = crate::lang::category_size(&category);
        let index = if count > 0 {
            gen_rand.next_max(count as i32) as usize
        } else {
            0
        };
        npc.given_name = crate::lang::random_from_category(&category, index);
    } else {
        npc.given_name = new_npc_name(ty, gen_rand);
    }
    variation
}

/// `NPC.TypeName` (`NPC.cs:6727`) = `Lang.GetNPCNameValue(netID)`.
pub fn type_name(npc: &Npc) -> String {
    crate::lang::npc_name(npc.kind.net_id())
}

/// `NPC.GetGivenOrTypeNetName` (`NPC.cs:7358-7365`): the given name alone
/// while named ("Andrew"), else the `TypeName` — the party broadcasts read
/// this, NOT the full `FullName` (`BirthdayParty.cs:130-138`).
pub fn given_or_type_name(npc: &Npc) -> String {
    if npc.given_name.is_empty() {
        return type_name(npc);
    }
    npc.given_name.clone()
}

/// `NPC.FullName` (`NPC.cs:6729-6739`): the `TypeName` while unnamed, else
/// `Language.GetTextValue("Game.NPCTitle", _givenName, TypeName)` —
/// "{0} the {1}" ("Andrew the Guide"). `GetFullNetName` (`:7349-7356`) is
/// the same text as a NetworkText; the port's chat carries flat strings, so
/// one function serves both readers.
pub fn full_name(npc: &Npc) -> String {
    if npc.given_name.is_empty() {
        return type_name(npc);
    }
    crate::lang::text_args("Game.NPCTitle", &[npc.given_name.clone(), type_name(npc)])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_english<T>(f: impl FnOnce() -> T) -> T {
        let _guard = crate::lang::test_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        f()
    }

    #[test]
    fn the_category_table_matches_the_csharp_switch() {
        // `NPC.cs:7285-7322`, spot rows across the ladder.
        assert_eq!(name_category(22), Some("GuideNames"));
        assert_eq!(name_category(368), Some("TravelingMerchantNames"));
        assert_eq!(name_category(633), Some("BestiaryGirlNames"));
        assert_eq!(name_category(637), Some("CatNames_Siamese"));
        assert_eq!(name_category(684), Some("SlimeNames_Copper"));
        assert_eq!(name_category(4), None, "the Eye of Cthulhu names \"\"");
    }

    #[test]
    fn a_new_name_draws_from_the_category_stream() {
        with_english(|| {
            let count = crate::lang::category_size("GuideNames");
            assert!(count > 0, "the shipped Town.json carries GuideNames");
            let mut rng = UnifiedRandom::new(1234);
            let mut mirror = UnifiedRandom::new(1234);
            let name = new_npc_name(22, &mut rng);
            let index = mirror.next_max(count as i32) as usize;
            assert_eq!(name, crate::lang::random_from_category("GuideNames", index));
            assert!(!name.is_empty());
            // An unnamed type answers "" without drawing.
            let mut rng2 = UnifiedRandom::new(1234);
            let fresh = UnifiedRandom::new(1234);
            assert_eq!(new_npc_name(4, &mut rng2), "");
            assert_eq!(rng2, fresh, "no draw consumed");
        });
    }

    #[test]
    fn the_full_name_reads_given_then_title() {
        with_english(|| {
            let mut guide = Npc::new(crate::npc::NpcKind::Guide, 0.0, 0.0, 1, 0.0);
            // Unnamed: the TypeName (`NPC.cs:6733-6736`).
            assert_eq!(full_name(&guide), "Guide");
            guide.given_name = "Andrew".into();
            // `Game.NPCTitle` = "{0} the {1}" (`en-US.json:805`).
            assert_eq!(full_name(&guide), "Andrew the Guide");
        });
    }

    #[test]
    fn the_town_unique_data_draws_twice_for_a_legacy_profile() {
        with_english(|| {
            let mut guide = Npc::new(crate::npc::NpcKind::Guide, 0.0, 0.0, 1, 0.0);
            let mut main_rand = UnifiedRandom::new(7);
            let mut rng = UnifiedRandom::new(7);
            let mut mirror = UnifiedRandom::new(7);
            let variation = give_town_unique_data(&mut guide, &mut main_rand, &mut rng);
            assert_eq!(variation, 0, "the legacy profile rolls 0");
            // Two draws: the first is discarded (`:80844` then `:80848`).
            let count = crate::lang::category_size("GuideNames");
            let _first = mirror.next_max(count as i32);
            let second = mirror.next_max(count as i32) as usize;
            assert_eq!(
                guide.given_name,
                crate::lang::random_from_category("GuideNames", second)
            );
            // A non-town NPC is untouched (`isLikeATownNPC` false).
            let mut eye = Npc::new(crate::npc::NpcKind::DemonEye, 0.0, 0.0, 1, 0.0);
            let before = eye.given_name.clone();
            give_town_unique_data(&mut eye, &mut main_rand, &mut rng);
            assert_eq!(eye.given_name, before);
        });
    }
}
