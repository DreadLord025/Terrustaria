//! S1-C-12 wave D — the LOW-tier armor sets' FLAG halves
//! (`ArmorSetBonuses.Benefits`, `DataStructures/ArmorSetBonuses.cs:10-384`,
//! the registration rows `:390-493`), kept in a module disjoint from the
//! stat halves (`tr-core/src/armor_sets.rs`) and the accessory families
//! (waves A/B/C).
//!
//! What lands here (the rows the port did not carry yet):
//! * Ash Wood 5279/5280/5281 (`:402`, `Benefits.AshWood` `:380-383`):
//!   `ashWoodBonus` — the lava hit's halves (`Player.cs:27694-27702`);
//! * Gladiator 3187/3188/3189 (`:414`, `:327-330`): `noKnockback` (the
//!   Hurt gate already reads it, `player.rs::hurt_inner`,
//!   `Player.cs:38519-38524`);
//! * Fossil 3374/3375/3376 (`:416`, `:337-340`): `ammoCost80`;
//! * Shadow Scale (`:437`, `:312-315`): `shadowArmor` — the 2×2×2 piece
//!   Cartesian product ({102,956}×{101,957}×{100,958},
//!   `ArmorSetBonus.cs:98-126`) and the movement boost
//!   (`Player.cs:26511-26517`);
//! * Crimson 792/793/794 (`:438`, `:248-251`): `crimsonRegen` — the
//!   `RegenInputs` `extra_time`/`extra_ramp_scale` arms
//!   (`Player.cs:19117-19120`, `:19242-19245`);
//! * Meteor 123/124/125 (`:441`, `:264-267`): `spaceGun` — the mana-pay
//!   skip (`Player.cs:52695-52698`) and the regen-delay skip
//!   (`:50587-50593`);
//! * Frost 684/685/686 (`:442`, `:269-274`): `frostBurn` (the +10%
//!   melee/ranged damage half already rides `tr-core`'s `SET_STATS`);
//! * Molten 231/232/233 (`:444`, `:281-289`): `fireWalk` + the GATED
//!   `buffImmune[24]` (`!vampireBurningInSunlight`, `:285-288` — the
//!   ungated `tr-core` row moved here);
//! * the ranged-head ammo sets: Cobalt 373/374/375 (`:456`, `:174-177`),
//!   Mythril 378/379/380 (`:459`, `:189-192`), Adamantite 402/403/404
//!   (`:462`, `:205-208`);
//! * the mixed-piece completions' table rows for Jungle (`:443`), Snow
//!   (`:445`) and Mining (`:446`) — their STAT/immunity halves ride the
//!   extended `tr-core` tables; the rows here carry the Lang keys and
//!   the tooltip counts.
//!
//! Boundary (docs/ANALYSIS.md "S1-C-12 wave D"): the stat halves
//! (`set_stats`/`set_bonus_defense`/`set_buff_immunity`'s Snow rows) stay
//! in `tr-core/src/armor_sets.rs` (E-7c / M3-EQUIP territory) and are NOT
//! re-applied here; E-7c's `buff_effects_accessory.rs` owns the buff-slot
//! arms (pets/imbues/tongued/…), not these set flags. The ammo-cost flags
//! are granted in `world_sim/player.rs::grant_ammo_mastery` instead of
//! the main seam: that pass clears and re-derives the ammo flags AFTER
//! `update_with_grapple` runs (the call site, `world_sim/player.rs:796`),
//! so a grant here would be wiped the same tick.
//!
//! Deliberately deferred (cited, ANALYSIS): the `frostBurn` PROJECTILE
//! on-hit consumers (`Projectile.cs:11518-11521`/`:11851-11854`/`:17323`
//! — the melee StatusToNPC half landed with S1-P-i10,
//! `melee_on_hit.rs`; `magmaStone`'s melee half likewise), the
//! `lavaRose` halves of the Ash Wood row (`Player.cs:27675`,
//! `:27703-27707` — no `lavaRose` field), the remix-world lava variants
//! (`:27689-27693`, `:27696-27699`), the crimson heal-chunk dust
//! (`:19292-19305`, visual), the empressBrooch/Magiluminescence movement
//! arms (`:26500-26510`, no fields) and the `BuildLookup` group-by
//! overwrite quirk (`ArmorSetBonuses.cs:495-519` — the wave-D rows'
//! piece ids are disjoint, so the Head-then-Body scan of
//! `GetCompleteSet` `:521-540` collapses to a plain complete-row scan).

use crate::inv::ARMOR;
use crate::item::ItemSlot;
use crate::player::Player;
use tr_core::localization::Substitution;

/// `ArmorSetBonus.PartType` (`ArmorSetBonus.cs:10-16`): the piece that
/// decides the set's variant (the head-variant ore sets).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimaryPart {
    None,
    Head,
    Body,
    Legs,
}

/// The flag halves of `ArmorSetBonuses.Benefits` this wave ports. The
/// stat halves (`meleeDamage`, `manaCost`, `pickSpeed`, the Snow
/// immunities) are `tr-core`'s and do not appear here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetFlag {
    /// `Benefits.ShadowScale` (`ArmorSetBonuses.cs:312-315`).
    ShadowArmor,
    /// `Benefits.Crimson` (`:248-251`).
    CrimsonRegen,
    /// `Benefits.Meteor` (`:264-267`).
    SpaceGun,
    /// `Benefits.Gladiator` (`:327-330`).
    NoKnockback,
    /// `Benefits.Molten`'s `fireWalk` (`:284`).
    FireWalk,
    /// `Benefits.Molten`'s gated `buffImmune[24]` (`:285-288`).
    MoltenFireImmune,
    /// `Benefits.AshWood` (`:380-383`).
    AshWoodBonus,
    /// `Benefits.Frost`'s `frostBurn` (`:271`).
    FrostBurn,
    /// `Benefits.Fossil`/`CobaltRanged`/`MythrilRanged` (`:337-340`,
    /// `:174-177`, `:189-192`) — granted in the ammo-mastery seam.
    AmmoCost80,
    /// `Benefits.AdamantiteRanged` (`:205-208`) — ammo-mastery seam.
    AmmoCost75,
}

/// One `ArmorSetBonuses.All` entry (`ArmorSetBonus.cs:128-142`): the
/// registration triple, the Lang `textKey` (`Description =
/// Language.GetText(TextKey)`, `:135`), the primary part and the flag
/// halves. The array registrations are expanded into their Cartesian
/// products (`Builder.Set(int[], int[], int[])`, `:98-126`).
pub struct ArmorSetRow {
    pub name: &'static str,
    pub text_key: &'static str,
    pub head: i32,
    pub body: i32,
    pub legs: i32,
    pub primary: PrimaryPart,
    pub flags: &'static [SetFlag],
}

/// The wave-D registration rows, in `ArmorSetBonuses.Initialize` order
/// (`ArmorSetBonuses.cs:390-493`). Jungle/Snow/Mining carry no flags —
/// their halves are `tr-core`'s stat/immunity tables (extended with the
/// same mixed triples).
pub const SETS: &[ArmorSetRow] = &[
    // `:402` — Benefits.AshWood.
    ArmorSetRow {
        name: "AshWood",
        text_key: "ArmorSetBonus.AshWood",
        head: 5279,
        body: 5280,
        legs: 5281,
        primary: PrimaryPart::None,
        flags: &[SetFlag::AshWoodBonus],
    },
    // `:414` — Benefits.Gladiator.
    ArmorSetRow {
        name: "Gladiator",
        text_key: "ArmorSetBonus.Gladiator",
        head: 3187,
        body: 3188,
        legs: 3189,
        primary: PrimaryPart::None,
        flags: &[SetFlag::NoKnockback],
    },
    // `:416` — Benefits.Fossil.
    ArmorSetRow {
        name: "Fossil",
        text_key: "ArmorSetBonus.Fossil",
        head: 3374,
        body: 3375,
        legs: 3376,
        primary: PrimaryPart::None,
        flags: &[SetFlag::AmmoCost80],
    },
    // `:437` — Benefits.ShadowScale: {102,956}×{101,957}×{100,958},
    // head-outer/body-middle/legs-inner (`ArmorSetBonus.cs:112-124`).
    ArmorSetRow { name: "ShadowScale", text_key: "ArmorSetBonus.ShadowScale", head: 102, body: 101, legs: 100, primary: PrimaryPart::None, flags: &[SetFlag::ShadowArmor] },
    ArmorSetRow { name: "ShadowScale", text_key: "ArmorSetBonus.ShadowScale", head: 102, body: 101, legs: 958, primary: PrimaryPart::None, flags: &[SetFlag::ShadowArmor] },
    ArmorSetRow { name: "ShadowScale", text_key: "ArmorSetBonus.ShadowScale", head: 102, body: 957, legs: 100, primary: PrimaryPart::None, flags: &[SetFlag::ShadowArmor] },
    ArmorSetRow { name: "ShadowScale", text_key: "ArmorSetBonus.ShadowScale", head: 102, body: 957, legs: 958, primary: PrimaryPart::None, flags: &[SetFlag::ShadowArmor] },
    ArmorSetRow { name: "ShadowScale", text_key: "ArmorSetBonus.ShadowScale", head: 956, body: 101, legs: 100, primary: PrimaryPart::None, flags: &[SetFlag::ShadowArmor] },
    ArmorSetRow { name: "ShadowScale", text_key: "ArmorSetBonus.ShadowScale", head: 956, body: 101, legs: 958, primary: PrimaryPart::None, flags: &[SetFlag::ShadowArmor] },
    ArmorSetRow { name: "ShadowScale", text_key: "ArmorSetBonus.ShadowScale", head: 956, body: 957, legs: 100, primary: PrimaryPart::None, flags: &[SetFlag::ShadowArmor] },
    ArmorSetRow { name: "ShadowScale", text_key: "ArmorSetBonus.ShadowScale", head: 956, body: 957, legs: 958, primary: PrimaryPart::None, flags: &[SetFlag::ShadowArmor] },
    // `:438` — Benefits.Crimson.
    ArmorSetRow {
        name: "Crimson",
        text_key: "ArmorSetBonus.Crimson",
        head: 792,
        body: 793,
        legs: 794,
        primary: PrimaryPart::None,
        flags: &[SetFlag::CrimsonRegen],
    },
    // `:441` — Benefits.Meteor.
    ArmorSetRow {
        name: "Meteor",
        text_key: "ArmorSetBonus.Meteor",
        head: 123,
        body: 124,
        legs: 125,
        primary: PrimaryPart::None,
        flags: &[SetFlag::SpaceGun],
    },
    // `:442` — Benefits.Frost (the damage half rides tr-core SET_STATS).
    ArmorSetRow {
        name: "Frost",
        text_key: "ArmorSetBonus.Frost",
        head: 684,
        body: 685,
        legs: 686,
        primary: PrimaryPart::None,
        flags: &[SetFlag::FrostBurn],
    },
    // `:443` — Benefits.Jungle: {228,960}×{229,961}×{230,962}. The
    // `manaCost -= 0.16f` half rides tr-core SET_STATS (all 8 triples).
    ArmorSetRow { name: "Jungle", text_key: "ArmorSetBonus.Jungle", head: 228, body: 229, legs: 230, primary: PrimaryPart::None, flags: &[] },
    ArmorSetRow { name: "Jungle", text_key: "ArmorSetBonus.Jungle", head: 228, body: 229, legs: 962, primary: PrimaryPart::None, flags: &[] },
    ArmorSetRow { name: "Jungle", text_key: "ArmorSetBonus.Jungle", head: 228, body: 961, legs: 230, primary: PrimaryPart::None, flags: &[] },
    ArmorSetRow { name: "Jungle", text_key: "ArmorSetBonus.Jungle", head: 228, body: 961, legs: 962, primary: PrimaryPart::None, flags: &[] },
    ArmorSetRow { name: "Jungle", text_key: "ArmorSetBonus.Jungle", head: 960, body: 229, legs: 230, primary: PrimaryPart::None, flags: &[] },
    ArmorSetRow { name: "Jungle", text_key: "ArmorSetBonus.Jungle", head: 960, body: 229, legs: 962, primary: PrimaryPart::None, flags: &[] },
    ArmorSetRow { name: "Jungle", text_key: "ArmorSetBonus.Jungle", head: 960, body: 961, legs: 230, primary: PrimaryPart::None, flags: &[] },
    ArmorSetRow { name: "Jungle", text_key: "ArmorSetBonus.Jungle", head: 960, body: 961, legs: 962, primary: PrimaryPart::None, flags: &[] },
    // `:444` — Benefits.Molten (the meleeDamage half rides tr-core).
    ArmorSetRow {
        name: "Molten",
        text_key: "ArmorSetBonus.Molten",
        head: 231,
        body: 232,
        legs: 233,
        primary: PrimaryPart::None,
        flags: &[SetFlag::FireWalk, SetFlag::MoltenFireImmune],
    },
    // `:445` — Benefits.Snow: {803,978}×{804,979}×{805,980}. The
    // buffImmune[46]/[47] halves ride tr-core set_buff_immunity (all 8).
    ArmorSetRow { name: "Snow", text_key: "ArmorSetBonus.Snow", head: 803, body: 804, legs: 805, primary: PrimaryPart::None, flags: &[] },
    ArmorSetRow { name: "Snow", text_key: "ArmorSetBonus.Snow", head: 803, body: 804, legs: 980, primary: PrimaryPart::None, flags: &[] },
    ArmorSetRow { name: "Snow", text_key: "ArmorSetBonus.Snow", head: 803, body: 979, legs: 805, primary: PrimaryPart::None, flags: &[] },
    ArmorSetRow { name: "Snow", text_key: "ArmorSetBonus.Snow", head: 803, body: 979, legs: 980, primary: PrimaryPart::None, flags: &[] },
    ArmorSetRow { name: "Snow", text_key: "ArmorSetBonus.Snow", head: 978, body: 804, legs: 805, primary: PrimaryPart::None, flags: &[] },
    ArmorSetRow { name: "Snow", text_key: "ArmorSetBonus.Snow", head: 978, body: 804, legs: 980, primary: PrimaryPart::None, flags: &[] },
    ArmorSetRow { name: "Snow", text_key: "ArmorSetBonus.Snow", head: 978, body: 979, legs: 805, primary: PrimaryPart::None, flags: &[] },
    ArmorSetRow { name: "Snow", text_key: "ArmorSetBonus.Snow", head: 978, body: 979, legs: 980, primary: PrimaryPart::None, flags: &[] },
    // `:446` — Benefits.Mining: {88,5588,4008}×{410,5589}×{411,5590}.
    // The pickSpeed half rides tr-core SET_STATS (all 12 triples).
    ArmorSetRow { name: "Mining", text_key: "ArmorSetBonus.Mining", head: 88, body: 410, legs: 411, primary: PrimaryPart::None, flags: &[] },
    ArmorSetRow { name: "Mining", text_key: "ArmorSetBonus.Mining", head: 88, body: 410, legs: 5590, primary: PrimaryPart::None, flags: &[] },
    ArmorSetRow { name: "Mining", text_key: "ArmorSetBonus.Mining", head: 88, body: 5589, legs: 411, primary: PrimaryPart::None, flags: &[] },
    ArmorSetRow { name: "Mining", text_key: "ArmorSetBonus.Mining", head: 88, body: 5589, legs: 5590, primary: PrimaryPart::None, flags: &[] },
    ArmorSetRow { name: "Mining", text_key: "ArmorSetBonus.Mining", head: 5588, body: 410, legs: 411, primary: PrimaryPart::None, flags: &[] },
    ArmorSetRow { name: "Mining", text_key: "ArmorSetBonus.Mining", head: 5588, body: 410, legs: 5590, primary: PrimaryPart::None, flags: &[] },
    ArmorSetRow { name: "Mining", text_key: "ArmorSetBonus.Mining", head: 5588, body: 5589, legs: 411, primary: PrimaryPart::None, flags: &[] },
    ArmorSetRow { name: "Mining", text_key: "ArmorSetBonus.Mining", head: 5588, body: 5589, legs: 5590, primary: PrimaryPart::None, flags: &[] },
    ArmorSetRow { name: "Mining", text_key: "ArmorSetBonus.Mining", head: 4008, body: 410, legs: 411, primary: PrimaryPart::None, flags: &[] },
    ArmorSetRow { name: "Mining", text_key: "ArmorSetBonus.Mining", head: 4008, body: 410, legs: 5590, primary: PrimaryPart::None, flags: &[] },
    ArmorSetRow { name: "Mining", text_key: "ArmorSetBonus.Mining", head: 4008, body: 5589, legs: 411, primary: PrimaryPart::None, flags: &[] },
    ArmorSetRow { name: "Mining", text_key: "ArmorSetBonus.Mining", head: 4008, body: 5589, legs: 5590, primary: PrimaryPart::None, flags: &[] },
    // `:456` — Benefits.CobaltRanged (PartType.Head).
    ArmorSetRow {
        name: "CobaltRanged",
        text_key: "ArmorSetBonus.CobaltRanged",
        head: 373,
        body: 374,
        legs: 375,
        primary: PrimaryPart::Head,
        flags: &[SetFlag::AmmoCost80],
    },
    // `:459` — Benefits.MythrilRanged (PartType.Head).
    ArmorSetRow {
        name: "MythrilRanged",
        text_key: "ArmorSetBonus.MythrilRanged",
        head: 378,
        body: 379,
        legs: 380,
        primary: PrimaryPart::Head,
        flags: &[SetFlag::AmmoCost80],
    },
    // `:462` — Benefits.AdamantiteRanged (PartType.Head).
    ArmorSetRow {
        name: "AdamantiteRanged",
        text_key: "ArmorSetBonus.AdamantiteRanged",
        head: 402,
        body: 403,
        legs: 404,
        primary: PrimaryPart::Head,
        flags: &[SetFlag::AmmoCost75],
    },
];

/// `ArmorSetBonus.QueryResult` (`ArmorSetBonus.cs:43-50`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct QueryResult {
    pub items_needed: i32,
    pub items_found: i32,
}

impl QueryResult {
    /// `QueryResult.Complete` (`:49`).
    pub fn complete(&self) -> bool {
        self.items_needed == self.items_found
    }
}

/// `ArmorSetBonus.QueryCount` + `TryCounting` (`ArmorSetBonus.cs:174-193`):
/// a `neededItem == 0` piece is a wildcard — it neither counts towards
/// `ItemsNeeded` nor can match (the Wizard/Magic Hat head-only rows).
pub fn query_count(row: &ArmorSetRow, head: i32, body: i32, legs: i32) -> QueryResult {
    let mut result = QueryResult::default();
    let mut try_counting = |tested: i32, needed: i32| {
        // `:183-193`.
        if needed != 0 {
            result.items_needed += 1;
            if tested == needed {
                result.items_found += 1;
            }
        }
    };
    try_counting(head, row.head);
    try_counting(body, row.body);
    try_counting(legs, row.legs);
    result
}

/// `ArmorSetBonuses.GetCompleteSet` (`ArmorSetBonuses.cs:521-540`) over
/// the wave-D table: the first row whose `QueryCount` is complete. The
/// C# scans `SetsContaining[Head]` then `SetsContaining[Body]`; the
/// wave-D piece ids are disjoint across sets, so the plain scan returns
/// the same row (the `BuildLookup` overwrite quirk `:495-519` cannot
/// bite here — see the module doc).
pub fn complete_set(head: i32, body: i32, legs: i32) -> Option<&'static ArmorSetRow> {
    SETS.iter()
        .find(|row| query_count(row, head, body, legs).complete())
}

/// The wave-D hook: `Player.UpdateArmorSets` (`Player.cs:16304-16306` →
/// `ArmorSetBonuses.GetCompleteSet(...)?.Effect(this)`, called at
/// `:25685` after `UpdateEquips` `:25633`). Grants the completed set's
/// flag halves — everything except the ammo-cost flags, which ride
/// [`grant_set_ammo_flags`] in the ammo-mastery seam (its per-tick clear
/// runs after this update). Called from `player.rs` right after
/// `accessory_defense::apply_defense_accessories`, i.e. AFTER the
/// equipment-flag clears (the C#'s `ResetEffects` `:25491` precedes
/// `UpdateArmorSets`).
pub fn apply_armor_set_bonuses(p: &mut Player) {
    let (head, body, legs) = (
        p.equipped[0] as i32,
        p.equipped[1] as i32,
        p.equipped[2] as i32,
    );
    let Some(row) = complete_set(head, body, legs) else {
        return;
    };
    for flag in row.flags {
        match flag {
            // `ArmorSetBonuses.cs:312-315` — Benefits.ShadowScale.
            SetFlag::ShadowArmor => p.shadow_armor = true,
            // `:248-251` — Benefits.Crimson.
            SetFlag::CrimsonRegen => p.crimson_regen = true,
            // `:264-267` — Benefits.Meteor.
            SetFlag::SpaceGun => p.space_gun = true,
            // `:327-330` — Benefits.Gladiator.
            SetFlag::NoKnockback => p.no_knockback = true,
            // `:284` — Benefits.Molten's fireWalk.
            SetFlag::FireWalk => p.fire_walk = true,
            // `:285-288` — Benefits.Molten's gated buffImmune[24]. The
            // vampire sun-scorch granter landed with W4-f30
            // (`sun_scorch.rs::update_sun_scorch`, `Player.cs:28709`): the
            // burn's `Array.Clear` (`:28653`) strips this immunity on the
            // burning ticks, so the gate holds while the flag stands.
            SetFlag::MoltenFireImmune => {
                if !p.vampire_burning_in_sunlight {
                    if let Some(slot) = p.buffs.buff_immune.get_mut(24) {
                        *slot = true;
                    }
                }
            }
            // `:380-383` — Benefits.AshWood.
            SetFlag::AshWoodBonus => p.ash_wood_bonus = true,
            // `:271` — Benefits.Frost's frostBurn.
            SetFlag::FrostBurn => p.frost_burn = true,
            // The ammo-cost flags belong to the ammo-mastery seam
            // (`world_sim/player.rs::grant_ammo_mastery` clears them
            // after this update) — granted in `grant_set_ammo_flags`.
            SetFlag::AmmoCost80 | SetFlag::AmmoCost75 => {}
        }
    }
}

/// The ammo-set half of the wave-D hook, called at the tail of
/// `world_sim/player.rs::grant_ammo_mastery` (after its clear, before
/// the shoot path reads the flags): `Benefits.Fossil`
/// (`ArmorSetBonuses.cs:337-340`), `CobaltRanged` (`:174-177`),
/// `MythrilRanged` (`:189-192`) → `ammoCost80`; `AdamantiteRanged`
/// (`:205-208`) → `ammoCost75`. The C# grants these inside
/// `UpdateArmorSets` (`Player.cs:25685`); the port's ammo flags are
/// cleared/re-derived later in the tick, so the grant lives with them.
pub fn grant_set_ammo_flags(p: &mut Player, armor: &[ItemSlot; ARMOR]) {
    let (head, body, legs) = (
        armor[0].kind as i32,
        armor[1].kind as i32,
        armor[2].kind as i32,
    );
    let Some(row) = complete_set(head, body, legs) else {
        return;
    };
    for flag in row.flags {
        match flag {
            SetFlag::AmmoCost80 => p.ammo_cost80 = true,
            SetFlag::AmmoCost75 => p.ammo_cost75 = true,
            _ => {}
        }
    }
}

/// `Player.ItemCheck_PayMana_ShouldSkipManaUse`'s spaceGun arm
/// (`Player.cs:52695-52698`): the Meteor set makes the Space Gun (127),
/// the Gray/Orange Zapinators (4347/4348) and the Laser Rifle (514)
/// cost no mana. The same four ids skip the mana-regen delay
/// (`ItemCheck_ApplyManaRegenDelay`, `:50587-50593`) — the port folds
/// both into the pay path's skip (no pay, no delay). The 3269 arm
/// (`:52691-52694`) and the turret/minion-targeting alt-fire arms
/// (`:52699-52706`) have no port items/seam yet and stay out.
pub fn space_gun_skips_mana(space_gun: bool, item_kind: u16) -> bool {
    space_gun && matches!(item_kind, 127 | 4347 | 4348 | 514)
}

// ---------------------------------------------------------------------------
// The set-bonus tooltip text
// ---------------------------------------------------------------------------

/// The `UI.ItemSetBonusDecidedBy*` row for a primary part
/// (`ArmorSetBonus.cs:232-238`).
fn decided_by_key(part: PrimaryPart) -> &'static str {
    match part {
        PrimaryPart::Head => "UI.ItemSetBonusDecidedByHead",
        PrimaryPart::Body => "UI.ItemSetBonusDecidedByBody",
        PrimaryPart::Legs => "UI.ItemSetBonusDecidedByLegs",
        PrimaryPart::None => "",
    }
}

/// `ArmorSetBonus.GetPart` (`ArmorSetBonus.cs:163-172`).
fn row_part(row: &ArmorSetRow, part: PrimaryPart) -> i32 {
    match part {
        PrimaryPart::Head => row.head,
        PrimaryPart::Body => row.body,
        PrimaryPart::Legs => row.legs,
        PrimaryPart::None => 0,
    }
}

/// `ArmorSetBonus.GetTooltipForSinglePiece` (`ArmorSetBonus.cs:195-206`):
/// the unworn piece's grey "Set Bonus: …" row. When the set has a
/// primary part and this piece is not it, the description collapses to
/// the "Changes with … piece" row (`:198-201`).
pub fn tooltip_for_single_piece(row: &ArmorSetRow, item_type: i32) -> String {
    let description = if row.primary != PrimaryPart::None && row_part(row, row.primary) != item_type
    {
        crate::lang::text(decided_by_key(row.primary))
    } else {
        crate::lang::text(row.text_key)
    };
    // `ItemSetBonusGeneral.FormatWith({ Description })` (`:202-205`,
    // the template `UI.ItemSetBonusGeneral`, `:231`).
    crate::lang::text_format_with(
        "UI.ItemSetBonusGeneral",
        &[("Description", Substitution::Text(description))],
    )
}

/// `ArmorSetBonus.GetTooltipForWornArmor` (`ArmorSetBonus.cs:208-221`):
/// the worn piece's "Set Bonus (found/needed)" row; the primary-part
/// mismatch compares the CONTEXT's piece against the row's (`:211`).
pub fn tooltip_for_worn_armor(
    row: &ArmorSetRow,
    head: i32,
    body: i32,
    legs: i32,
    result: QueryResult,
) -> String {
    let context_part = match row.primary {
        PrimaryPart::Head => head,
        PrimaryPart::Body => body,
        PrimaryPart::Legs => legs,
        PrimaryPart::None => 0,
    };
    let description =
        if row.primary != PrimaryPart::None && context_part != row_part(row, row.primary) {
            crate::lang::text(decided_by_key(row.primary))
        } else {
            crate::lang::text(row.text_key)
        };
    // `ItemSetBonusEquipped.FormatWith({ Description, Numerator,
    // Denominator })` (`:215-220`, the template `:230`).
    crate::lang::text_format_with(
        "UI.ItemSetBonusEquipped",
        &[
            ("Description", Substitution::Text(description)),
            ("Numerator", Substitution::Int(result.items_found as i64)),
            ("Denominator", Substitution::Int(result.items_needed as i64)),
        ],
    )
}

/// The worn-arm tooltip's set pick (`Main.cs:21280-21306`): among the
/// rows containing the hovered piece, keep the first, then prefer the
/// row with MORE pieces needed, breaking ties by MORE pieces found
/// (`:21292-21304`). Returns the picked row and its count.
pub fn best_worn_set(
    item_type: i32,
    head: i32,
    body: i32,
    legs: i32,
) -> Option<(&'static ArmorSetRow, QueryResult)> {
    let mut best: Option<&'static ArmorSetRow> = None;
    let mut result = QueryResult::default();
    for row in SETS
        .iter()
        .filter(|r| r.head == item_type || r.body == item_type || r.legs == item_type)
    {
        if best.is_none() {
            best = Some(row);
        }
        let query = query_count(row, head, body, legs);
        let better = result.items_needed < query.items_needed
            || (result.items_needed == query.items_needed
                && result.items_found < query.items_found);
        if better {
            result = query;
            best = Some(row);
        }
    }
    best.map(|row| (row, result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::PlayerInput;
    use tr_core::world::World;

    fn rig() -> (World, Player) {
        let w = World::new("t".into(), 64, 800);
        let p = Player::spawn(&w);
        (w, p)
    }

    fn wear(w: &World, p: &mut Player, head: u16, body: u16, legs: u16) {
        p.equipped[0] = head;
        p.equipped[1] = body;
        p.equipped[2] = legs;
        p.update(w, PlayerInput::default());
    }

    /// `ArmorSetBonuses.cs:437` + `ArmorSetBonus.cs:98-126` — the Shadow
    /// Scale registration expands to all 8 ancient/pure combinations.
    #[test]
    fn the_shadow_scale_table_expands_the_eight_piece_combinations() {
        let rows: Vec<_> = SETS.iter().filter(|r| r.name == "ShadowScale").collect();
        assert_eq!(rows.len(), 8);
        for (h, b, l) in [
            (102, 101, 100),
            (102, 101, 958),
            (102, 957, 100),
            (102, 957, 958),
            (956, 101, 100),
            (956, 101, 958),
            (956, 957, 100),
            (956, 957, 958),
        ] {
            let row = complete_set(h, b, l).expect("the combination completes");
            assert_eq!(row.name, "ShadowScale");
        }
        // A repeated piece is not a set.
        assert!(complete_set(102, 101, 101).is_none());
        assert!(complete_set(0, 0, 0).is_none());
    }

    /// `:443`/`:445`/`:446` — the Jungle/Snow/Mining Cartesian products.
    #[test]
    fn the_mixed_set_tables_match_the_builder_cartesian_products() {
        assert_eq!(SETS.iter().filter(|r| r.name == "Jungle").count(), 8);
        assert_eq!(SETS.iter().filter(|r| r.name == "Snow").count(), 8);
        assert_eq!(SETS.iter().filter(|r| r.name == "Mining").count(), 12);
        // The mixed completions resolve (the stat halves ride tr-core).
        assert_eq!(complete_set(228, 961, 962).unwrap().name, "Jungle");
        assert_eq!(complete_set(978, 979, 805).unwrap().name, "Snow");
        assert_eq!(complete_set(5588, 410, 5590).unwrap().name, "Mining");
        // Their flag lists are empty — no double-apply with tr-core.
        assert!(complete_set(228, 961, 962).unwrap().flags.is_empty());
        assert!(complete_set(978, 979, 805).unwrap().flags.is_empty());
        assert!(complete_set(5588, 410, 5590).unwrap().flags.is_empty());
    }

    /// `:414` + `:327-330` — the Gladiator set's knockback immunity;
    /// the Hurt gate reads the flag (`Player.cs:38519-38524`).
    #[test]
    fn the_gladiator_set_grants_knockback_immunity() {
        let (w, mut p) = rig();
        wear(&w, &mut p, 3187, 3188, 3189);
        assert!(p.no_knockback);
        // Taking the helm off clears it (ResetEffects, `:18356`).
        p.equipped[0] = 0;
        p.update(&w, PlayerInput::default());
        assert!(!p.no_knockback);
    }

    /// `:441` + `:264-267` — the Meteor set's spaceGun flag.
    #[test]
    fn the_meteor_set_grants_the_space_gun_flag() {
        let (w, mut p) = rig();
        wear(&w, &mut p, 123, 124, 125);
        assert!(p.space_gun);
        p.equipped[0] = 0;
        p.update(&w, PlayerInput::default());
        assert!(!p.space_gun, "ResetEffects clears spaceGun (`:18364`)");
    }

    /// `Player.cs:52695-52698` — the four exempt weapons.
    #[test]
    fn the_space_gun_skip_covers_the_four_laser_weapons() {
        for kind in [127u16, 4347, 4348, 514] {
            assert!(space_gun_skips_mana(true, kind), "kind {kind}");
            assert!(!space_gun_skips_mana(false, kind));
        }
        assert!(!space_gun_skips_mana(true, 123));
        assert!(!space_gun_skips_mana(true, 0));
    }

    /// `:444` + `:281-289` — Molten: fireWalk + the gated fire immunity;
    /// the +10% melee damage stays tr-core's (no double-apply).
    #[test]
    fn the_molten_set_grants_fire_walk_and_the_gated_fire_immunity() {
        let (w, mut p) = rig();
        wear(&w, &mut p, 231, 232, 233);
        assert!(p.fire_walk);
        assert!(p.buffs.buff_immune[24]);
        // The piece 232's +7% (ARMOR_STATS) + the set's +10% — once each.
        assert!((p.melee_damage - 1.17).abs() < 1e-4);
        // The gate (`ArmorSetBonuses.cs:285-288`): while the vampire
        // sun-scorch flag stands, the immunity is withheld.
        p.buffs.buff_immune[24] = false;
        p.vampire_burning_in_sunlight = true;
        apply_armor_set_bonuses(&mut p);
        assert!(!p.buffs.buff_immune[24]);
        assert!(p.fire_walk, "fireWalk is not gated");
        p.vampire_burning_in_sunlight = false;
        apply_armor_set_bonuses(&mut p);
        assert!(p.buffs.buff_immune[24]);
    }

    /// `:438` + `:248-251` — Crimson: the flag and the regen-time add
    /// (`Player.cs:19117-19120` → `RegenInputs.extra_time`).
    #[test]
    fn the_crimson_set_grants_the_regen_flag_and_time_add() {
        let (w, mut p) = rig();
        wear(&w, &mut p, 792, 793, 794);
        assert!(p.crimson_regen);
        // One survival tick: the base +1 (`:19112`) plus the crimson +1.
        p.regen.life_regen_time = 0.0;
        p.update_survival(&w, &mut tr_core::UnifiedRandom::default());
        assert!((p.regen.life_regen_time - 2.0).abs() < 1e-4);
        // Without the set the same tick adds only the base 1.
        p.crimson_regen = false;
        p.regen.life_regen_time = 0.0;
        p.update_survival(&w, &mut tr_core::UnifiedRandom::default());
        assert!((p.regen.life_regen_time - 1.0).abs() < 1e-4);
    }

    /// `Player.cs:19242-19245` — the crimson ramp ×1.5 rides
    /// `RegenInputs.extra_ramp_scale` in the engine's own slot (after
    /// the movement multiply, before the campfire).
    #[test]
    fn the_crimson_ramp_scale_multiplies_the_natural_regen() {
        let mut state = crate::regen::RegenState::default();
        state.life_regen_time = 300.0; // one ramp step armed
        let mut life = 50;
        let base = crate::regen::RegenInputs {
            max_life: 100,
            standing_still: true,
            ..crate::regen::RegenInputs::default()
        };
        crate::regen::update_life_regen(&mut state, &base, &mut life);
        let plain = state.life_regen;
        state.life_regen_time = 300.0;
        let crimson = crate::regen::RegenInputs {
            extra_ramp_scale: 1.5,
            ..base
        };
        crate::regen::update_life_regen(&mut state, &crimson, &mut life);
        // 1 step × 1.25 (still) × 0.3625 (100 max life) rounds to 0;
        // × 1.5 lifts it to 1 (`:19262`'s banker's round).
        assert_eq!(plain, 0);
        assert_eq!(state.life_regen, 1);
    }

    /// `:442` + `:269-274` — Frost: the flag; the damage halves stay
    /// tr-core's (piece 684's +16% + the set's +10%, once each).
    #[test]
    fn the_frost_set_grants_the_frost_burn_flag_and_keeps_the_stat_row() {
        let (w, mut p) = rig();
        wear(&w, &mut p, 684, 685, 686);
        assert!(p.frost_burn);
        assert!((p.melee_damage - 1.26).abs() < 1e-4);
        assert!((p.ranged_damage - 1.26).abs() < 1e-4);
        p.equipped[0] = 0;
        p.update(&w, PlayerInput::default());
        assert!(!p.frost_burn, "ResetEffects clears frostBurn (`:18686`)");
    }

    /// `:402` + `:380-383` — Ash Wood: the flag and the lava halves
    /// (`Player.cs:27694-27702`: 80/2 = 40 damage, 420−210 = 210 burn).
    /// The set's own 7 defense (2+3+2) floors the reduction at 3
    /// (`reduce_damage`: floor(40 − 7×0.5) = 36), so hp lands at 64.
    #[test]
    fn the_ash_wood_set_halves_the_lava_hit() {
        let (w, mut p) = rig();
        wear(&w, &mut p, 5279, 5280, 5281);
        assert!(p.ash_wood_bonus);
        // The lava hit itself (the `update_survival` sub-step, the
        // existing lava test's pattern): budget empty, cooldown clear.
        p.lava_time = 0;
        p.hp = 100;
        p.hurt_cd = 0.0;
        p.in_lava = true;
        p.update_survival(&w, &mut tr_core::UnifiedRandom::default());
        assert_eq!(p.hp, 64, "the halved 40-damage lava hit, floor(40−3.5)=36");
        let idx = p.buffs.find_buff_index(24).expect("On Fire");
        assert_eq!(p.buffs.buff_time[idx], 210, "the 420-210 burn");
        // Without the set the same hit is the full 80 (floor(80−3.5)=76).
        // The first hit armed the General window — clear both halves
        // (`hurt_inner` gates on `immune` AND `hurt_cd`, `:38210-38216`).
        p.ash_wood_bonus = false;
        p.hp = 100;
        p.immune = false;
        p.hurt_cd = 0.0;
        p.buffs.clear_buff(24);
        p.in_lava = true;
        p.update_survival(&w, &mut tr_core::UnifiedRandom::default());
        assert_eq!(p.hp, 24, "the unhalved 80-damage lava hit");
    }

    /// `:416`/`:456`/`:459`/`:462` — the ammo sets grant their cost
    /// flags in the ammo-mastery seam, NOT in the main flag pass.
    #[test]
    fn the_ammo_sets_grant_their_cost_flags_in_the_ammo_seam() {
        let (_, mut p) = rig();
        let mut armor = [crate::item::EMPTY_SLOT; ARMOR];
        let misc = [crate::item::EMPTY_SLOT; crate::inv::MISC];
        for (head, body, legs, cost80, cost75) in [
            (3374u16, 3375u16, 3376u16, true, false), // Fossil
            (373, 374, 375, true, false),             // CobaltRanged
            (378, 379, 380, true, false),             // MythrilRanged
            (402, 403, 404, false, true),             // AdamantiteRanged
        ] {
            armor[0].kind = head;
            armor[1].kind = body;
            armor[2].kind = legs;
            p.ammo_cost80 = false;
            p.ammo_cost75 = false;
            crate::world_sim::player::grant_ammo_mastery(&mut p, &armor, &misc);
            assert_eq!(p.ammo_cost80, cost80, "head {head}");
            assert_eq!(p.ammo_cost75, cost75, "head {head}");
            // The main seam must NOT grant them (the ammo pass clears
            // after the update — a grant there would be wiped).
            p.ammo_cost80 = false;
            p.ammo_cost75 = false;
            p.equipped[0] = head;
            p.equipped[1] = body;
            p.equipped[2] = legs;
            apply_armor_set_bonuses(&mut p);
            assert!(!p.ammo_cost80 && !p.ammo_cost75);
        }
    }

    /// `Player.cs:26511-26517` — the Shadow Scale movement boost:
    /// acceleration ×1.75, max/acc run speed ×1.15, slowdown ×1.75.
    #[test]
    fn the_shadow_set_boosts_the_run_stats() {
        let (w, mut p) = rig();
        p.update(&w, PlayerInput::default());
        let base = (
            p.max_run_speed_value(),
            p.acc_run_speed_value(),
            p.run_acceleration_value(),
            p.run_slowdown_value(),
        );
        wear(&w, &mut p, 956, 957, 958);
        assert!(p.shadow_armor);
        let boosted = (
            p.max_run_speed_value(),
            p.acc_run_speed_value(),
            p.run_acceleration_value(),
            p.run_slowdown_value(),
        );
        assert!((boosted.0 - base.0 * 1.15).abs() < 1e-4);
        assert!((boosted.1 - base.1 * 1.15).abs() < 1e-4);
        assert!((boosted.2 - base.2 * 1.75).abs() < 1e-4);
        assert!((boosted.3 - base.3 * 1.75).abs() < 1e-4);
        // The `!tongued` half of the `:26494` gate: the flag (E-7c's arm,
        // granted by the world-sim buff pass, not `Player::update`) pins
        // the boost out. Set it directly — the seam only reads it.
        p.tongued = true;
        p.update(&w, PlayerInput::default());
        assert!((p.max_run_speed_value() - base.0).abs() < 1e-4);
        assert!((p.run_acceleration_value() - base.2).abs() < 1e-4);
        p.tongued = false;
    }

    /// `ArmorSetBonus.cs:174-193` — the wildcard count: a 0-needed piece
    /// neither counts nor matches (the head-only Wizard rows' shape).
    #[test]
    fn the_query_count_wildcard_skips_zero_pieces() {
        let row = ArmorSetRow {
            name: "Wizard",
            text_key: "ArmorSetBonus.Wizard",
            head: 238,
            body: 1282,
            legs: 0,
            primary: PrimaryPart::Head,
            flags: &[],
        };
        let q = query_count(&row, 238, 1282, 999);
        assert_eq!(q.items_needed, 2);
        assert_eq!(q.items_found, 2);
        assert!(q.complete(), "the legs wildcard cannot fail the set");
        let q = query_count(&row, 238, 0, 0);
        assert_eq!((q.items_needed, q.items_found), (2, 1));
        assert!(!q.complete());
    }

    /// `Main.cs:21280-21306` — the worn tooltip picks the row with the
    /// most pieces needed, then the most found.
    #[test]
    fn the_worn_set_pick_prefers_the_deeper_count() {
        // Hovering the Cobalt breastplate while wearing the caster head:
        // only the CobaltRanged row contains 374, count 2/3.
        let (row, result) = best_worn_set(374, 371, 374, 375).expect("a row");
        assert_eq!(row.name, "CobaltRanged");
        assert_eq!((result.items_found, result.items_needed), (2, 3));
        // The complete triple counts 3/3.
        let (_, result) = best_worn_set(374, 373, 374, 375).unwrap();
        assert_eq!((result.items_found, result.items_needed), (3, 3));
        assert!(result.complete());
        // An unrelated piece has no row.
        assert!(best_worn_set(727, 0, 0, 0).is_none());
    }

    /// `ArmorSetBonus.cs:195-221` + the `UI.ItemSetBonus*` templates
    /// (`en-US.json:368-372`) — the tooltip text, English culture.
    #[test]
    fn the_set_bonus_tooltips_format_the_lang_rows() {
        let _guard = crate::lang::test_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let shadow = complete_set(102, 101, 100).unwrap();
        // The unworn piece: the general template.
        assert_eq!(
            tooltip_for_single_piece(shadow, 102),
            "Set Bonus: Increased movement speed and acceleration"
        );
        // Worn, partial: the equipped template with the count.
        let q = query_count(shadow, 102, 101, 0);
        assert_eq!(
            tooltip_for_worn_armor(shadow, 102, 101, 0, q),
            "Set Bonus (2/3)\nIncreased movement speed and acceleration"
        );
        // Worn, complete.
        let q = query_count(shadow, 102, 101, 100);
        assert_eq!(
            tooltip_for_worn_armor(shadow, 102, 101, 100, q),
            "Set Bonus (3/3)\nIncreased movement speed and acceleration"
        );
        // A head-variant set's non-head piece collapses to the
        // "Changes with head piece" row (`:198-201`).
        let cobalt = complete_set(373, 374, 375).unwrap();
        assert_eq!(
            tooltip_for_single_piece(cobalt, 374),
            "Set Bonus: Changes with head piece"
        );
        assert_eq!(
            tooltip_for_single_piece(cobalt, 373),
            "Set Bonus: 20% chance to save ammo"
        );
        // Worn with the wrong head: the context mismatch (`:211`).
        let q = query_count(cobalt, 371, 374, 375);
        assert_eq!(
            tooltip_for_worn_armor(cobalt, 371, 374, 375, q),
            "Set Bonus (2/3)\nChanges with head piece"
        );
    }
}
