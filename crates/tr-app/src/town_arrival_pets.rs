//! W4-f43 — the town-spawn census tail: the pet-license, town-pet, town-slime,
//! Santa and Princess arms of `Main.UpdateTime_SpawnTownNPCs`
//! (`Main.cs:65542-66116`) plus their unlock sources. UNWIRED (code-only wave):
//! the core 21-kind machine lives in [`crate::town_arrival`]; this module ports
//! the arms that machine leaves closed, verbatim, as pure functions with the
//! RNG draws injected. The wiring lane splices them at the cited positions.
//!
//! Census correction: the directive's "town pet slimes (701-707)" does not
//! match this decompile — `NPCID.Count = 697` (`ID/NPCID.cs:12581`) and the
//! town slimes are 670/678-684 (`NPCID.cs:12017-12045`,
//! `Sets.IsTownSlime` = {670, 678, 679, 680, 681, 682, 683, 684}
//! `NPCID.cs:10119`). No NPC ids 701-707 exist.
//!
//! RNG slots (the whole census):
//! * The canSpawn table and the priority ladder draw NOTHING for these arms —
//!   the only `Main.rand` draw inside `UpdateTime_SpawnTownNPCs` is the Party
//!   Girl's `Next(40) == 0` (`Main.cs:65786`), already pinned in
//!   `town_arrival.rs` on `RngKey::UpdateTime`.
//! * The pet-license exchange rolls `Main.rand.Next(variants.Length)` —
//!   `Profiles.VariantNPCProfile.RollVariation` (`GameContent/Profiles.cs:214-217`),
//!   up to 100 draws while the index is unchanged (`NPC.cs:80941-80944`).
//! * The pet/slime/princess NAME roll rides `WorldGen.genRand`, NOT `Main.rand`
//!   (`NPC.cs:7311-7322` → `LanguageManager.cs:419`
//!   `value[random.Next(value.Count)]`; the variant profile's
//!   `GetNameForVariant` `Profiles.cs:219-222`).
#![allow(dead_code)] // W4-f43 code-only wave: the wiring lane splices the arms

/// `Main.checkXMas` (`Main.cs:13118-13139`): Christmas is Dec 15-31 on the
/// wall clock; either force flag overrides. The date inputs are injected so
/// the sim stays deterministic (the caller supplies the local date).
pub(crate) fn check_xmas(day: u32, month: u32, force_today: bool, force_forever: bool) -> bool {
    // `:13123-13130` — `day >= 15 && month == 12`.
    let mut x_mas = day >= 15 && month == 12;
    // `:13131-13138` — the two force flags.
    if force_today {
        x_mas = true;
    }
    if force_forever {
        x_mas = true;
    }
    x_mas
}

/// `Player.ItemCheck_UsePetLicenses` (`Player.cs:45352-45366`): the three
/// license items, their town-pet type, the chat key and the net-message data
/// (`-12`/`-13`/`-14`). Order is the C# if-chain order.
pub(crate) const PET_LICENSE_ARMS: [(u16, i32, &str, i32); 3] = [
    (4829, 637, "Misc.LicenseCatUsed", -12),
    (4830, 638, "Misc.LicenseDogUsed", -13),
    (4910, 656, "Misc.LicenseBunnyUsed", -14),
];

/// `Player.LicenseOrExchangePet`'s gate (`Player.cs:45440`):
/// `ItemTimeIsZero && (!petBoughtFlag || NPC.AnyNPCs(npcType))` — a bought
/// license is only re-usable (as an exchange) while the pet is resident.
/// The caller applies `ApplyItemTime` on success (`:45442`).
pub(crate) fn license_gate(item_time_is_zero: bool, pet_bought: bool, any_npcs: bool) -> bool {
    item_time_is_zero && (!pet_bought || any_npcs)
}

/// The outcome of `NPC.UnlockOrExchangePet` (`NPC.cs:80894-80918`). The chat
/// broadcast + `NetMessage.TrySendData(7)` ride the caller's seams; the
/// client arm only requests (`SendData(61, …, netMessageData)` `:80901`).
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PetUseOutcome {
    /// `Main.netMode == 1` and the gate holds (`:80897-80903`): the client
    /// sends the unlock request instead of mutating the flag.
    ClientRequest(i32),
    /// First use (`:80904-80909`): the flag flips true, the license chat key
    /// broadcasts, world flags sync (7).
    Unlocked(&'static str),
    /// Bought + resident + the reroll changed the variation (`:80910-80913`,
    /// `Misc.PetExchangeSuccess`).
    ExchangeSuccess,
    /// Bought + resident + the reroll failed (`:80914-80917`,
    /// `Misc.PetExchangeFail`).
    ExchangeFail,
    /// The client gate failed — nothing happens.
    Nothing,
}

/// `NPC.UnlockOrExchangePet` (`NPC.cs:80894-80918`), single-player + client
/// branches verbatim. `pet_bought` is the C# `ref bool`; `reroll` runs only
/// on the bought-and-resident exchange arm (`RerollVariationForNPCType`,
/// `:80910`). The server broadcast arm (`netMode == 2`) folds into the
/// single-player flag write — the port is single-player.
pub(crate) fn unlock_or_exchange_pet(
    net_mode: i32,
    pet_bought: &mut bool,
    any_npcs: bool,
    text_key: &'static str,
    net_message_data: i32,
    reroll: impl FnOnce() -> bool,
) -> PetUseOutcome {
    if net_mode == 1 {
        // `:80897-80903`.
        if !*pet_bought || any_npcs {
            return PetUseOutcome::ClientRequest(net_message_data);
        }
        return PetUseOutcome::Nothing;
    }
    if !*pet_bought {
        // `:80904-80909`.
        *pet_bought = true;
        return PetUseOutcome::Unlocked(text_key);
    }
    // `:80910-80917`.
    if reroll() {
        PetUseOutcome::ExchangeSuccess
    } else {
        PetUseOutcome::ExchangeFail
    }
}

/// `NPC.RerollVariation`'s roll loop (`NPC.cs:80933-80961`): up to 100 draws
/// of `profile.RollVariation()` while the index is unchanged
/// (`while (num2++ < 100 && townNpcVariationIndex == num)` `:80941`). Returns
/// the new index, or `None` when every draw repeated the old one (the
/// `PetExchangeFail` arm). On success the caller also writes
/// `GivenName = profile.GetNameForVariant` (a `WorldGen.genRand` draw —
/// [`name_category_for`]) and `life = lifeMax` (`:80949-80950`); the
/// PetExchange particle + `TrySendData(56)` are the caller's seams.
///
/// `roll` is `Main.rand.Next(variant_count)`
/// (`Profiles.cs:214-217` — the ONLY `Main.rand` slot in the license chain).
pub(crate) fn reroll_variation(
    current: i32,
    variant_count: i32,
    roll: &mut dyn FnMut(i32) -> i32,
) -> Option<i32> {
    let mut index = current;
    // `num2++ < 100` — at most 100 body rolls; the unchanged check gates each.
    for _ in 0..100 {
        if index != current {
            break;
        }
        index = roll(variant_count);
    }
    // `:80945-80948` — unchanged after the loop = failure.
    if index == current {
        None
    } else {
        Some(index)
    }
}

/// `NPC.RerollVariationForNPCType` (`NPC.cs:80920-80931`): the FIRST active
/// NPC of the type rerolls; no resident = `false`. The caller scans its own
/// roster in slot order (the C# walks `Main.npc[0..maxNPCs]`).
pub(crate) fn reroll_variation_for_npc_type(
    residents: &mut [PetVariationSlot],
    npc_type: i32,
    roll: &mut dyn FnMut(i32) -> i32,
) -> bool {
    for slot in residents.iter_mut() {
        if slot.active && slot.npc_type == npc_type {
            // `:80927` — one resident only, then return.
            if !has_town_profile(npc_type) {
                // `:80935-80938` — no profile = failure.
                return false;
            }
            // The VariantNPCProfile rolls `Main.rand.Next(6)`
            // (`Profiles.cs:216`); every other profile is a
            // Legacy/Stacked/Transformable one whose RollVariation returns
            // the constant 0 (`Profiles.cs:21`/`:85`) — so a legacy-type
            // exchange succeeds only from a non-zero index.
            let mut legacy_roll = |_: i32| 0;
            let roller: &mut dyn FnMut(i32) -> i32 = match variant_count_for(npc_type) {
                Some(_) => roll,
                None => &mut legacy_roll,
            };
            let variant_count = variant_count_for(npc_type).unwrap_or(1);
            let Some(new_index) = reroll_variation(slot.variation_index, variant_count, roller)
            else {
                return false;
            };
            slot.variation_index = new_index;
            // `:80950` — the exchange heals to full.
            slot.life = slot.life_max;
            return true;
        }
    }
    false
}

/// The minimal `NPC` slice `RerollVariationForNPCType` touches — the wiring
/// lane maps it onto `Npc` (`variation_index`, `hp`/`max_hp`, `given_name`).
pub(crate) struct PetVariationSlot {
    pub(crate) active: bool,
    pub(crate) npc_type: i32,
    pub(crate) variation_index: i32,
    pub(crate) life: i32,
    pub(crate) life_max: i32,
}

/// `TownNPCProfiles`' variant rosters (`GameContent/TownNPCProfiles.cs:138-148`):
/// the texture-name order IS the variation-index order.
pub(crate) const CAT_VARIANTS: [&str; 6] = [
    "Siamese",
    "Black",
    "OrangeTabby",
    "RussianBlue",
    "Silver",
    "White",
];
pub(crate) const DOG_VARIANTS: [&str; 6] = [
    "Labrador",
    "PitBull",
    "Beagle",
    "Corgi",
    "Dalmation",
    "Husky",
];
pub(crate) const BUNNY_VARIANTS: [&str; 6] = [
    "White",
    "Angora",
    "Dutch",
    "Flemish",
    "Lop",
    "Silver",
];

/// The head-texture ids per variation (`TownNPCProfiles.cs:211-213`):
/// Cat 27-32, Dog 33-38, Bunny 39-44 (`GetHeadTextureIndex`,
/// `Profiles.cs:238-241`).
pub(crate) fn pet_head_index(npc_type: i32, variation: i32) -> Option<i32> {
    let base = match npc_type {
        637 => 27,
        638 => 33,
        656 => 39,
        _ => return None,
    };
    if (0..6).contains(&variation) {
        Some(base + variation)
    } else {
        None
    }
}

/// The variant count for `RollVariation` — only the three license pets have
/// a `VariantNPCProfile`; everything else (the town slimes'
/// `LegacyNPCProfile`, `Profiles.cs:83-86`) rolls the constant 0.
pub(crate) fn variant_count_for(npc_type: i32) -> Option<i32> {
    match npc_type {
        637 | 638 | 656 => Some(6),
        _ => None,
    }
}

/// `TownNPCProfiles._townNPCProfiles`' key set
/// (`GameContent/TownNPCProfiles.cs:19-181`) — `GetProfile`'s success gate
/// for `RerollVariation` (`NPC.cs:80935-80938`). 40 entries: the 26 town
/// humans + OldMan/SkeletonMerchant/TravelingMerchant + the 3 license pets
/// + the 8 town slimes.
pub(crate) fn has_town_profile(npc_type: i32) -> bool {
    matches!(
        npc_type,
        22 | 20
            | 19
            | 107
            | 160
            | 208
            | 228
            | 550
            | 369
            | 54
            | 209
            | 38
            | 207
            | 588
            | 124
            | 17
            | 18
            | 227
            | 229
            | 142
            | 178
            | 353
            | 441
            | 108
            | 663
            | 633
            | 37
            | 453
            | 368
            | 637
            | 638
            | 656
            | 670
            | 678
            | 679
            | 680
            | 681
            | 682
            | 683
            | 684
    )
}

/// The name category `getNewNPCNameInner` / `GetNameForVariant` draw from
/// (`NPC.cs:7311-7322`, `Profiles.cs:219-222`): the license pets key on the
/// CURRENT variation, the town slimes on their fixed colour, the Princess on
/// the plain category. The draw itself is
/// `WorldGen.genRand.Next(category_count)` (`LanguageManager.cs:419`) — a
/// DIFFERENT stream from the variation roll. Unknown types name `""`
/// (`NPC.cs:7323`).
pub(crate) fn name_category_for(npc_type: i32, variation: i32) -> Option<String> {
    let variants: &[&str] = match npc_type {
        637 => &CAT_VARIANTS,
        638 => &DOG_VARIANTS,
        656 => &BUNNY_VARIANTS,
        663 => return Some("PrincessNames".into()),
        670 => return Some("SlimeNames_Blue".into()),
        678 => return Some("SlimeNames_Green".into()),
        679 => return Some("SlimeNames_Old".into()),
        680 => return Some("SlimeNames_Purple".into()),
        681 => return Some("SlimeNames_Rainbow".into()),
        682 => return Some("SlimeNames_Red".into()),
        683 => return Some("SlimeNames_Yellow".into()),
        684 => return Some("SlimeNames_Copper".into()),
        _ => return None,
    };
    let v = variants.get(variation as usize)?;
    let base = match npc_type {
        637 => "CatNames",
        638 => "DogNames",
        _ => "BunnyNames",
    };
    Some(format!("{base}_{v}"))
}

/// The unlock/season flags the tail arms read. The `bought*` trio is
/// `NPC.boughtCat/Dog/Bunny` (`NPC.cs:6301-6305`); the slime flags are
/// `NPC.unlockedSlime*Spawn`. NOTE (census): the port's world save currently
/// SKIPS the bought trio on load (`wld/sections/flags.rs:608` `r.skip(3)`)
/// and only carries the rainbow/red slime flags (`world.rs:303-307`) — the
/// wiring lane must widen both before these arms can fire in production.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct TailFlags {
    pub(crate) bought_cat: bool,
    pub(crate) bought_dog: bool,
    pub(crate) bought_bunny: bool,
    pub(crate) unlocked_slime_blue: bool,
    pub(crate) unlocked_slime_green: bool,
    pub(crate) unlocked_slime_old: bool,
    pub(crate) unlocked_slime_purple: bool,
    pub(crate) unlocked_slime_rainbow: bool,
    pub(crate) unlocked_slime_red: bool,
    pub(crate) unlocked_slime_yellow: bool,
    pub(crate) unlocked_slime_copper: bool,
    /// `BirthdayParty.GenuineParty` (`Main.cs:65795`) — flag8's first term.
    pub(crate) genuine_party: bool,
    /// `NPC.downedFrost` (`Main.cs:65852`).
    pub(crate) downed_frost: bool,
    /// `Main.xMas` — [`check_xmas`].
    pub(crate) x_mas: bool,
    /// `NPC.unlockedPrincessSpawn` (`Main.cs:65949-65952`).
    pub(crate) unlocked_princess: bool,
    /// `Main.tenthAnniversaryWorld` (`Main.cs:65945`).
    pub(crate) tenth_anniversary: bool,
    /// `Main.getGoodWorld` (`Main.cs:65945`).
    pub(crate) get_good_world: bool,
}

/// The Princess's flag9 roster (`Main.cs:65944`): EVERY one of the 24
/// non-pet, non-Santa, non-OldMan town types resident — `num2..num27` minus
/// `num7` (OldMan 37) and `num13` (Santa 142). The type order below is the
/// C# `num` order, not the id order.
pub(crate) const PRINCESS_ROSTER: [i32; 24] = [
    17, 18, 20, 19, 22, 38, 54, 108, 107, 124, 160, 178, 207, 208, 209, 227, 228, 229, 353, 369,
    441, 550, 588, 633,
];

/// `flag9` (`Main.cs:65944-65952`): all 24 residents, OR the tenth-anniversary
/// world (unless getGoodWorld), OR the debug unlock. Zero RNG.
pub(crate) fn princess_flag9(
    flags: &TailFlags,
    count: &dyn Fn(i32) -> i32,
) -> bool {
    // `:65944` — the 24-term conjunction.
    let mut flag9 = PRINCESS_ROSTER.iter().all(|t| count(*t) > 0);
    // `:65945-65948`.
    if flags.tenth_anniversary && !flags.get_good_world {
        flag9 = true;
    }
    // `:65949-65952`.
    if flags.unlocked_princess {
        flag9 = true;
    }
    flag9
}

/// The unclaimed `townNPCCanSpawn` arms, in the C# write order: Santa
/// (`Main.cs:65852-65855`), the three license pets (`:65900-65911`), the
/// eight town slimes (`:65912-65943`) and the Princess (`:65953-65956`).
/// Every arm is `condition && numN < 1`; `count` is the per-type live
/// resident count (`num28..num39`, `:65719-65766`). Zero RNG.
pub(crate) fn tail_can_spawn(
    flags: &TailFlags,
    count: &dyn Fn(i32) -> i32,
) -> [(i32, bool); 13] {
    // `:65795-65799` — flag8 = GenuineParty || unlockedSlimeGreenSpawn.
    let flag8 = flags.genuine_party || flags.unlocked_slime_green;
    let flag9 = princess_flag9(flags, count);
    [
        // `:65852-65855` — downedFrost && num13 < 1 && Main.xMas.
        (142, flags.downed_frost && count(142) < 1 && flags.x_mas),
        // `:65900-65903`.
        (637, flags.bought_cat && count(637) < 1),
        // `:65904-65907`.
        (638, flags.bought_dog && count(638) < 1),
        // `:65908-65911`.
        (656, flags.bought_bunny && count(656) < 1),
        // `:65912-65915`.
        (670, flags.unlocked_slime_blue && count(670) < 1),
        // `:65916-65919` — flag8.
        (678, flag8 && count(678) < 1),
        // `:65920-65923`.
        (679, flags.unlocked_slime_old && count(679) < 1),
        // `:65924-65927`.
        (680, flags.unlocked_slime_purple && count(680) < 1),
        // `:65928-65931`.
        (681, flags.unlocked_slime_rainbow && count(681) < 1),
        // `:65932-65935`.
        (682, flags.unlocked_slime_red && count(682) < 1),
        // `:65936-65939`.
        (683, flags.unlocked_slime_yellow && count(683) < 1),
        // `:65940-65943`.
        (684, flags.unlocked_slime_copper && count(684) < 1),
        // `:65953-65956` — flag9 && num39 < 1.
        (663, flag9 && count(663) < 1),
    ]
}

/// The Santa ladder arm (`Main.cs:66042-66045`): splices AFTER the Party Girl
/// arm (`:66038-66041`) and BEFORE the Bartender arm (`:66046-66049`) of the
/// existing ladder. `current` is `num42` — a non-zero pick passes through.
pub(crate) fn santa_ladder_arm(
    current: i32,
    flags: &TailFlags,
    count: &dyn Fn(i32) -> i32,
) -> i32 {
    if current == 0 && flags.downed_frost && count(142) < 1 && flags.x_mas {
        return 142;
    }
    current
}

/// The pet/slime/Princess ladder tail (`Main.cs:66066-66113`): splices AFTER
/// the Zoologist arm (`:66062-66065`) — the last arm before
/// `WorldGen.prioritizedTownNPCType = num42` (`:66114`). The order is the C#
/// one and is NOT the canSpawn order: Princess, Copper, Blue, Green, Old,
/// Purple, RED, YELLOW, RAINBOW, Bunny, Cat, Dog. Zero RNG.
pub(crate) fn pet_slime_princess_ladder_tail(
    current: i32,
    flags: &TailFlags,
    count: &dyn Fn(i32) -> i32,
) -> i32 {
    let mut num42 = current;
    let flag8 = flags.genuine_party || flags.unlocked_slime_green;
    let flag9 = princess_flag9(flags, count);
    // `:66066-66069` — the Princess leads the tail.
    if num42 == 0 && flag9 && count(663) < 1 {
        num42 = 663;
    }
    // `:66070-66073`.
    if num42 == 0 && flags.unlocked_slime_copper && count(684) < 1 {
        num42 = 684;
    }
    // `:66074-66077`.
    if num42 == 0 && flags.unlocked_slime_blue && count(670) < 1 {
        num42 = 670;
    }
    // `:66078-66081` — flag8.
    if num42 == 0 && flag8 && count(678) < 1 {
        num42 = 678;
    }
    // `:66082-66085`.
    if num42 == 0 && flags.unlocked_slime_old && count(679) < 1 {
        num42 = 679;
    }
    // `:66086-66089`.
    if num42 == 0 && flags.unlocked_slime_purple && count(680) < 1 {
        num42 = 680;
    }
    // `:66090-66093` — Red BEFORE Yellow and Rainbow (the canSpawn table
    // writes Rainbow first; the ladder does not).
    if num42 == 0 && flags.unlocked_slime_red && count(682) < 1 {
        num42 = 682;
    }
    // `:66094-66097`.
    if num42 == 0 && flags.unlocked_slime_yellow && count(683) < 1 {
        num42 = 683;
    }
    // `:66098-66101`.
    if num42 == 0 && flags.unlocked_slime_rainbow && count(681) < 1 {
        num42 = 681;
    }
    // `:66102-66105`.
    if num42 == 0 && flags.bought_bunny && count(656) < 1 {
        num42 = 656;
    }
    // `:66106-66109`.
    if num42 == 0 && flags.bought_cat && count(637) < 1 {
        num42 = 637;
    }
    // `:66110-66113`.
    if num42 == 0 && flags.bought_dog && count(638) < 1 {
        num42 = 638;
    }
    num42
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_residents(_: i32) -> i32 {
        0
    }

    /// `Main.cs:13123-13138`: Dec 15-31 only; the force flags override any
    /// date. RNG-free.
    #[test]
    fn the_xmas_window_is_december_fifteen_through_thirty_one() {
        assert!(!check_xmas(14, 12, false, false));
        assert!(check_xmas(15, 12, false, false));
        assert!(check_xmas(31, 12, false, false));
        assert!(!check_xmas(1, 1, false, false));
        assert!(!check_xmas(20, 11, false, false));
        assert!(check_xmas(1, 6, true, false), "forceXMasForToday");
        assert!(check_xmas(1, 6, false, true), "forceXMasForever");
    }

    /// `Player.cs:45440`: the license gate — item time zero AND (unbought OR
    /// the pet is resident).
    #[test]
    fn the_license_gate_wants_zero_item_time_and_a_resident_for_exchanges() {
        assert!(license_gate(true, false, false), "first use, no pet yet");
        assert!(!license_gate(false, false, false), "item time blocks");
        assert!(!license_gate(true, true, false), "bought, pet gone");
        assert!(license_gate(true, true, true), "bought + resident = exchange");
    }

    /// `NPC.cs:80894-80918`: first use flips the flag and broadcasts the
    /// license key; the exchange arms ride the reroll result; the client
    /// only requests.
    #[test]
    fn the_unlock_or_exchange_branches_follow_the_csharp() {
        let mut bought = false;
        assert_eq!(
            unlock_or_exchange_pet(0, &mut bought, false, "Misc.LicenseCatUsed", -12, || false),
            PetUseOutcome::Unlocked("Misc.LicenseCatUsed")
        );
        assert!(bought, "the ref flag flipped");
        assert_eq!(
            unlock_or_exchange_pet(0, &mut bought, true, "Misc.LicenseCatUsed", -12, || true),
            PetUseOutcome::ExchangeSuccess
        );
        assert_eq!(
            unlock_or_exchange_pet(0, &mut bought, true, "Misc.LicenseCatUsed", -12, || false),
            PetUseOutcome::ExchangeFail
        );
        // The client arm never mutates the flag (`:80897-80903`).
        let mut client_flag = false;
        assert_eq!(
            unlock_or_exchange_pet(1, &mut client_flag, false, "k", -13, || true),
            PetUseOutcome::ClientRequest(-13)
        );
        assert!(!client_flag);
        assert_eq!(
            unlock_or_exchange_pet(1, &mut client_flag, false, "k", -13, || true),
            PetUseOutcome::ClientRequest(-13)
        );
        let mut bought_gone = true;
        assert_eq!(
            unlock_or_exchange_pet(1, &mut bought_gone, false, "k", -13, || true),
            PetUseOutcome::Nothing
        );
    }

    /// `NPC.cs:80941-80948`: the reroll draws at most 100 times and fails
    /// when every draw repeats the old index. The draw is
    /// `Main.rand.Next(6)` (`Profiles.cs:216`) — the injected slot.
    #[test]
    fn the_reroll_loop_caps_at_one_hundred_draws() {
        let mut draws = 0;
        let got = reroll_variation(2, 6, &mut |n| {
            draws += 1;
            assert_eq!(n, 6, "Next(variants.Length)");
            2
        });
        assert_eq!(got, None, "a constant roll never changes the index");
        assert_eq!(draws, 100, "the num2++ < 100 cap");
        // An immediate change draws exactly once.
        let mut draws = 0;
        let got = reroll_variation(2, 6, &mut |_| {
            draws += 1;
            4
        });
        assert_eq!(got, Some(4));
        assert_eq!(draws, 1);
        // A change on the 100th draw still succeeds.
        let mut draws = 0;
        let got = reroll_variation(0, 6, &mut |_| {
            draws += 1;
            if draws == 100 {
                1
            } else {
                0
            }
        });
        assert_eq!(got, Some(1));
        assert_eq!(draws, 100);
    }

    /// `TownNPCProfiles.cs:138-148` + `:211-213`: six variants per license
    /// pet, head ids 27-32 / 33-38 / 39-44 in variant order.
    #[test]
    fn the_variant_rosters_and_head_ids_match_the_profiles() {
        assert_eq!(CAT_VARIANTS[0], "Siamese");
        assert_eq!(DOG_VARIANTS[4], "Dalmation");
        assert_eq!(BUNNY_VARIANTS[5], "Silver");
        assert_eq!(pet_head_index(637, 0), Some(27));
        assert_eq!(pet_head_index(637, 5), Some(32));
        assert_eq!(pet_head_index(638, 0), Some(33));
        assert_eq!(pet_head_index(656, 5), Some(44));
        assert_eq!(pet_head_index(656, 6), None);
        assert_eq!(pet_head_index(670, 0), None, "slimes have no variant heads");
        assert_eq!(variant_count_for(637), Some(6));
        assert_eq!(variant_count_for(670), None, "LegacyNPCProfile rolls 0");
    }

    /// `NPC.cs:7311-7322` + `Profiles.cs:219-222`: the name category keys on
    /// the variation for the pets, on the colour for the slimes.
    #[test]
    fn the_name_categories_key_on_the_variation() {
        assert_eq!(
            name_category_for(637, 2).as_deref(),
            Some("CatNames_OrangeTabby")
        );
        assert_eq!(name_category_for(638, 0).as_deref(), Some("DogNames_Labrador"));
        assert_eq!(name_category_for(656, 1).as_deref(), Some("BunnyNames_Angora"));
        assert_eq!(name_category_for(670, 0).as_deref(), Some("SlimeNames_Blue"));
        assert_eq!(name_category_for(684, 0).as_deref(), Some("SlimeNames_Copper"));
        assert_eq!(name_category_for(663, 0).as_deref(), Some("PrincessNames"));
        assert_eq!(name_category_for(637, 6), None);
        assert_eq!(name_category_for(22, 0), None, "the Guide names elsewhere");
    }

    /// `NPC.cs:80920-80931`: the FIRST resident of the type rerolls and
    /// heals to full; a second resident is untouched; no resident = false.
    #[test]
    fn the_exchange_rerolls_the_first_resident_only() {
        let mut roster = [
            PetVariationSlot { active: true, npc_type: 637, variation_index: 1, life: 10, life_max: 100 },
            PetVariationSlot { active: true, npc_type: 637, variation_index: 1, life: 20, life_max: 100 },
        ];
        assert!(reroll_variation_for_npc_type(&mut roster, 637, &mut |_| 3));
        assert_eq!(roster[0].variation_index, 3);
        assert_eq!(roster[0].life, 100, "life = lifeMax (:80950)");
        assert_eq!(roster[1].variation_index, 1, "the second copy waits");
        assert_eq!(roster[1].life, 20);
        assert!(!reroll_variation_for_npc_type(&mut roster, 638, &mut |_| 3));
        // A town slime has a LegacyNPCProfile (constant-0 roll): from index
        // 0 the exchange always fails, from a non-zero index it resets to 0.
        let mut slimes = [PetVariationSlot {
            active: true, npc_type: 670, variation_index: 0, life: 5, life_max: 50,
        }];
        assert!(!reroll_variation_for_npc_type(&mut slimes, 670, &mut |_| 1));
        slimes[0].variation_index = 1;
        assert!(reroll_variation_for_npc_type(&mut slimes, 670, &mut |_| 99));
        assert_eq!(slimes[0].variation_index, 0, "LegacyNPCProfile rolls 0");
        // A type with no profile at all fails before any roll.
        let mut strangers = [PetVariationSlot {
            active: true, npc_type: 1, variation_index: 1, life: 5, life_max: 50,
        }];
        assert!(!reroll_variation_for_npc_type(&mut strangers, 1, &mut |_| {
            panic!("no profile = no roll")
        }));
    }

    /// `Main.cs:65900-65911`: the bought flags open the pet arms; a resident
    /// closes its own arm. Zero RNG.
    #[test]
    fn the_pet_arms_open_on_the_bought_flags() {
        let flags = TailFlags { bought_cat: true, bought_dog: true, bought_bunny: true, ..Default::default() };
        let table = tail_can_spawn(&flags, &no_residents);
        assert!(table.iter().find(|(t, _)| *t == 637).unwrap().1);
        assert!(table.iter().find(|(t, _)| *t == 638).unwrap().1);
        assert!(table.iter().find(|(t, _)| *t == 656).unwrap().1);
        // One resident cat closes only the cat arm.
        let table = tail_can_spawn(&flags, &|t| if t == 637 { 1 } else { 0 });
        assert!(!table.iter().find(|(t, _)| *t == 637).unwrap().1);
        assert!(table.iter().find(|(t, _)| *t == 638).unwrap().1);
        // Unbought = closed.
        let table = tail_can_spawn(&TailFlags::default(), &no_residents);
        assert!(table.iter().all(|(_, open)| !open));
    }

    /// `Main.cs:65795-65799` + `:65916-65919`: the green slime opens on a
    /// genuine party OR its own unlock; the other slimes need their flag.
    #[test]
    fn the_green_slime_rides_the_party_or_its_unlock() {
        let party = TailFlags { genuine_party: true, ..Default::default() };
        let table = tail_can_spawn(&party, &no_residents);
        assert!(table.iter().find(|(t, _)| *t == 678).unwrap().1);
        assert!(!table.iter().find(|(t, _)| *t == 670).unwrap().1);
        let unlock = TailFlags { unlocked_slime_green: true, ..Default::default() };
        let table = tail_can_spawn(&unlock, &no_residents);
        assert!(table.iter().find(|(t, _)| *t == 678).unwrap().1);
        // All eight slime flags open all eight arms.
        let all = TailFlags {
            unlocked_slime_blue: true, unlocked_slime_green: true, unlocked_slime_old: true,
            unlocked_slime_purple: true, unlocked_slime_rainbow: true, unlocked_slime_red: true,
            unlocked_slime_yellow: true, unlocked_slime_copper: true, ..Default::default()
        };
        let table = tail_can_spawn(&all, &no_residents);
        for t in [670, 678, 679, 680, 681, 682, 683, 684] {
            assert!(table.iter().find(|(id, _)| *id == t).unwrap().1, "slime {t}");
        }
    }

    /// `Main.cs:65852-65855`: Santa needs downedFrost AND xMas AND no
    /// resident — all three terms.
    #[test]
    fn santa_needs_the_frost_legion_and_the_season() {
        let base = TailFlags { downed_frost: true, ..Default::default() };
        let table = tail_can_spawn(&base, &no_residents);
        assert!(!table.iter().find(|(t, _)| *t == 142).unwrap().1, "no xMas");
        let xmas = TailFlags { x_mas: true, ..base };
        let table = tail_can_spawn(&xmas, &no_residents);
        assert!(table.iter().find(|(t, _)| *t == 142).unwrap().1);
        let table = tail_can_spawn(&xmas, &|t| if t == 142 { 1 } else { 0 });
        assert!(!table.iter().find(|(t, _)| *t == 142).unwrap().1, "resident Santa");
        let no_frost = TailFlags { x_mas: true, ..Default::default() };
        let table = tail_can_spawn(&no_frost, &no_residents);
        assert!(!table.iter().find(|(t, _)| *t == 142).unwrap().1);
    }

    /// `Main.cs:65944-65956`: flag9 wants ALL 24 roster types resident; the
    /// anniversary and unlock overrides force it; getGoodWorld cancels the
    /// anniversary override but not the unlock.
    #[test]
    fn the_princess_wants_every_town_type_or_an_override() {
        let flags = TailFlags::default();
        assert!(!princess_flag9(&flags, &no_residents));
        // 23 of 24 is not enough.
        let almost = |t: i32| if t == 633 { 0 } else { 1 };
        assert!(!princess_flag9(&flags, &almost));
        let all = |_: i32| 1;
        assert!(princess_flag9(&flags, &all));
        assert!(princess_flag9(&TailFlags { tenth_anniversary: true, ..flags }, &no_residents));
        assert!(!princess_flag9(
            &TailFlags { tenth_anniversary: true, get_good_world: true, ..flags },
            &no_residents
        ));
        assert!(princess_flag9(&TailFlags { unlocked_princess: true, ..flags }, &no_residents));
        // The Princess arm itself also wants no resident Princess.
        let unlock = TailFlags { unlocked_princess: true, ..flags };
        let table = tail_can_spawn(&unlock, &no_residents);
        assert!(table.iter().find(|(t, _)| *t == 663).unwrap().1);
        let table = tail_can_spawn(&unlock, &|t| if t == 663 { 1 } else { 0 });
        assert!(!table.iter().find(|(t, _)| *t == 663).unwrap().1);
        // The roster is the 24 C# num terms — no OldMan 37, no Santa 142,
        // no pets.
        assert_eq!(PRINCESS_ROSTER.len(), 24);
        assert!(!PRINCESS_ROSTER.contains(&37));
        assert!(!PRINCESS_ROSTER.contains(&142));
        assert!(!PRINCESS_ROSTER.contains(&637));
    }

    /// `Main.cs:66042-66045`: the Santa ladder arm sits between the Party
    /// Girl and the Bartender; a non-zero pick passes through untouched.
    #[test]
    fn the_santa_ladder_arm_passes_a_live_pick_through() {
        let xmas = TailFlags { downed_frost: true, x_mas: true, ..Default::default() };
        assert_eq!(santa_ladder_arm(0, &xmas, &no_residents), 142);
        assert_eq!(santa_ladder_arm(208, &xmas, &no_residents), 208);
        assert_eq!(santa_ladder_arm(0, &TailFlags::default(), &no_residents), 0);
    }

    /// `Main.cs:66066-66113`: the tail order is Princess, Copper, Blue,
    /// Green, Old, Purple, Red, Yellow, Rainbow, Bunny, Cat, Dog — Red
    /// BEFORE Rainbow even though the canSpawn table writes Rainbow first.
    #[test]
    fn the_ladder_tail_order_is_the_csharp_one() {
        let all = TailFlags {
            bought_cat: true, bought_dog: true, bought_bunny: true,
            unlocked_slime_blue: true, unlocked_slime_green: true, unlocked_slime_old: true,
            unlocked_slime_purple: true, unlocked_slime_rainbow: true, unlocked_slime_red: true,
            unlocked_slime_yellow: true, unlocked_slime_copper: true,
            unlocked_princess: true, ..Default::default()
        };
        // Everything eligible: the Princess leads.
        assert_eq!(pet_slime_princess_ladder_tail(0, &all, &no_residents), 663);
        // Princess resident: Copper next.
        let no_princess = TailFlags { unlocked_princess: false, ..all };
        assert_eq!(pet_slime_princess_ladder_tail(0, &no_princess, &no_residents), 684);
        // Copper + Blue resident: Green (flag8 via its unlock).
        let count = |t: i32| if t == 684 || t == 670 { 1 } else { 0 };
        assert_eq!(pet_slime_princess_ladder_tail(0, &no_princess, &count), 678);
        // Only Red + Rainbow unlocked: RED wins (the ladder order quirk).
        let red_rainbow = TailFlags {
            unlocked_slime_red: true, unlocked_slime_rainbow: true, ..Default::default()
        };
        assert_eq!(pet_slime_princess_ladder_tail(0, &red_rainbow, &no_residents), 682);
        // Only the three licenses bought: Bunny, then Cat, then Dog.
        let pets = TailFlags { bought_cat: true, bought_dog: true, bought_bunny: true, ..Default::default() };
        assert_eq!(pet_slime_princess_ladder_tail(0, &pets, &no_residents), 656);
        assert_eq!(pet_slime_princess_ladder_tail(0, &pets, &|t| if t == 656 { 1 } else { 0 }), 637);
        assert_eq!(
            pet_slime_princess_ladder_tail(0, &pets, &|t| if t == 656 || t == 637 { 1 } else { 0 }),
            638
        );
        // A live pick passes through.
        assert_eq!(pet_slime_princess_ladder_tail(22, &all, &no_residents), 22);
    }
}
