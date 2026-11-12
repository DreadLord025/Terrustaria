//! The armor set bonuses — `ArmorSetBonuses.Initialize`'s early-metal rows
//! (`ArmorSetBonuses.cs:422-413` region) and the defense consumer
//! (`Player.Hurt`'s `statDefense * 0.5` reduction).

/// `ArmorSetBonuses.Benefits.MetalTier1` (+2): the copper (89/80/76), the
/// ancient-copper (687/688/689), the iron (90/81/77) and the
/// ancient-iron-helmet (954/81/77) sets.
pub const METAL_TIER1: [(i32, i32, i32); 4] = [
    (89, 80, 76),
    (687, 688, 689),
    (90, 81, 77),
    (954, 81, 77),
];
/// `MetalTier2` (+3): the silver (91/82/78), the gold (92/83/79), the
/// ancient-silver (955/83/79) and the ancient-gold (690/691/692,
/// 693/694/695) sets.
pub const METAL_TIER2: [(i32, i32, i32); 5] = [
    (91, 82, 78),
    (92, 83, 79),
    (955, 83, 79),
    (690, 691, 692),
    (693, 694, 695),
];
/// `Benefits.Platinum` (+4): the platinum (696/697/698) set.
pub const PLATINUM: [(i32, i32, i32); 1] = [(696, 697, 698)];
/// `Benefits.Wood` (+1): the wood (727-729 and the variants) sets.
pub const WOOD: [(i32, i32, i32); 7] = [
    (727, 728, 729),
    (733, 734, 735),
    (730, 731, 732),
    (736, 737, 738),
    (924, 925, 926),
    (2509, 2510, 2511),
    (2512, 2513, 2514),
];

/// The set-bonus defense for an equipped `(head, body, legs)` triple — the
/// `statDefense +=` the benefit fns carry (`ArmorSetBonuses.cs:355-371`).
pub fn set_bonus_defense(head: i32, body: i32, legs: i32) -> i32 {
    if head <= 0 || body <= 0 || legs <= 0 {
        return 0;
    }
    let has = |set: &[(i32, i32, i32)]| {
        set.iter()
            .any(|(h, b, l)| *h == head && *b == body && *l == legs)
    };
    if has(&METAL_TIER1) {
        2
    } else if has(&METAL_TIER2) {
        3
    } else if has(&PLATINUM) {
        4
    } else if has(&WOOD) {
        1
    } else {
        0
    }
}

/// `Player.Hurt`'s defense reduction — the classic `damage - defense * 0.5`
/// (the expert formula lands with the difficulty flip), floored at 1.
pub fn reduce_damage(damage: i32, defense: i32) -> i32 {
    (damage as f32 - defense as f32 * 0.5).floor().max(1.0) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_metal_sets_map_to_their_tiers() {
        assert_eq!(set_bonus_defense(89, 80, 76), 2); // copper
        assert_eq!(set_bonus_defense(90, 81, 77), 2); // iron
        assert_eq!(set_bonus_defense(91, 82, 78), 3); // silver
        assert_eq!(set_bonus_defense(92, 83, 79), 3); // gold
        assert_eq!(set_bonus_defense(696, 697, 698), 4); // platinum
        assert_eq!(set_bonus_defense(727, 728, 729), 1); // wood
        // The mixed pieces: no bonus.
        assert_eq!(set_bonus_defense(89, 82, 76), 0);
        assert_eq!(set_bonus_defense(0, 0, 0), 0);
    }

    #[test]
    fn the_defense_reduces_and_floors_at_one() {
        // `damage - defense * 0.5`, the floor 1 (the classic arm).
        assert_eq!(reduce_damage(20, 10), 15);
        assert_eq!(reduce_damage(20, 44), 1);
        assert_eq!(reduce_damage(1, 40), 1);
    }
}

// ---------------------------------------------------------------------------
// The stat halves of the set benefits
// ---------------------------------------------------------------------------

/// The `Player`-scalar effects of a completed set — only the rows of
/// `ArmorSetBonuses.Benefits` (`DataStructures/ArmorSetBonuses.cs:8-383`) whose
/// targets the port models. Flag rows (`shadowArmor`, `fireWalk`, `buffImmune`,
/// `ammoCost80`, `spaceGun`, `onHitDodge`, the Beetle/Solar/Vortex machinery)
/// belong to their own systems and are not represented here.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SetStats {
    pub melee_damage: f32,
    pub ranged_damage: f32,
    pub magic_damage: f32,
    pub minion_damage: f32,
    pub melee_crit: i32,
    pub ranged_crit: i32,
    pub magic_crit: i32,
    pub move_speed: f32,
    pub melee_speed: f32,
    /// `Player.pickSpeed -= 0.1f` in the C# — the port stores the bonus with the
    /// opposite sign (`pick_speed_bonus`, the fed tiers' convention).
    pub pick_speed: f32,
    pub mana_max: i32,
    pub max_minions: i32,
    pub dash_type: i32,
    /// `Player.manaCost -= x` (the casters' set bonuses).
    pub mana_cost: f32,
    /// `Player.endurance += x` (the Turtle/Chlorophyte/Solar rows).
    pub endurance: f32,
    /// `Player.thorns` / `turtleThorns` / `cactusThorns` (the Turtle and
    /// Cactus sets).
    pub thorns: f32,
    pub turtle_thorns: bool,
    pub cactus_thorns: bool,
}

const fn stats() -> SetStats {
    SetStats {
        melee_damage: 0.0,
        ranged_damage: 0.0,
        magic_damage: 0.0,
        minion_damage: 0.0,
        melee_crit: 0,
        ranged_crit: 0,
        magic_crit: 0,
        move_speed: 0.0,
        melee_speed: 0.0,
        pick_speed: 0.0,
        mana_max: 0,
        max_minions: 0,
        dash_type: 0,
        mana_cost: 0.0,
        endurance: 0.0,
        thorns: 0.0,
        turtle_thorns: false,
        cactus_thorns: false,
    }
}

/// `(name, (head, body, legs), stats)` — the registration triples of
/// `ArmorSetBonuses.Initialize` (`:396-500`) for the sets whose benefit is a
/// scalar stat. The `PartType.Head` rows keep their concrete triples (the
/// Wizard Hat/Magic Hat pairs carry `0` for the unneeded pieces, which is what
/// the port's empty armor slots hold).
pub const SET_STATS: &[(&str, (i32, i32, i32), SetStats)] = &[
    // `:417` Benefits.Necro — `rangedCrit += 10`.
    (
        "Necro",
        (151, 152, 153),
        SetStats { ranged_crit: 10, ..stats() },
    ),
    (
        "Necro",
        (959, 152, 153),
        SetStats { ranged_crit: 10, ..stats() },
    ),
    // `:413` Benefits.Pumpkin — every damage class `+= 0.1f`.
    (
        "Pumpkin",
        (1731, 1732, 1733),
        SetStats {
            melee_damage: 0.1,
            ranged_damage: 0.1,
            magic_damage: 0.1,
            minion_damage: 0.1,
            ..stats()
        },
    ),
    // `:415` Benefits.Ninja — `moveSpeed += 0.2f`.
    (
        "Ninja",
        (256, 257, 258),
        SetStats { move_speed: 0.2, ..stats() },
    ),
    // `:421-431` Benefits.Wizard (head-only rows).
    (
        "Wizard",
        (238, 1282, 0),
        SetStats { magic_crit: 10, ..stats() },
    ),
    (
        "Wizard",
        (238, 1283, 0),
        SetStats { magic_crit: 10, ..stats() },
    ),
    (
        "Wizard",
        (238, 1284, 0),
        SetStats { magic_crit: 10, ..stats() },
    ),
    (
        "Wizard",
        (238, 1285, 0),
        SetStats { magic_crit: 10, ..stats() },
    ),
    (
        "Wizard",
        (238, 1286, 0),
        SetStats { magic_crit: 10, ..stats() },
    ),
    (
        "Wizard",
        (238, 1287, 0),
        SetStats { magic_crit: 10, ..stats() },
    ),
    (
        "Wizard",
        (238, 2279, 0),
        SetStats { magic_crit: 10, ..stats() },
    ),
    (
        "Wizard",
        (238, 4256, 0),
        SetStats { magic_crit: 10, ..stats() },
    ),
    // `:437-447` Benefits.MagicHat — `statManaMax2 += 60`.
    (
        "MagicHat",
        (2275, 1282, 0),
        SetStats { mana_max: 60, ..stats() },
    ),
    (
        "MagicHat",
        (2275, 1283, 0),
        SetStats { mana_max: 60, ..stats() },
    ),
    (
        "MagicHat",
        (2275, 1284, 0),
        SetStats { mana_max: 60, ..stats() },
    ),
    (
        "MagicHat",
        (2275, 1285, 0),
        SetStats { mana_max: 60, ..stats() },
    ),
    (
        "MagicHat",
        (2275, 1286, 0),
        SetStats { mana_max: 60, ..stats() },
    ),
    (
        "MagicHat",
        (2275, 1287, 0),
        SetStats { mana_max: 60, ..stats() },
    ),
    (
        "MagicHat",
        (2275, 2279, 0),
        SetStats { mana_max: 60, ..stats() },
    ),
    (
        "MagicHat",
        (2275, 4256, 0),
        SetStats { mana_max: 60, ..stats() },
    ),
    // `:451` Benefits.Molten — `meleeDamage += 0.1f` (the `fireWalk` and
    // `buffImmune[24]` rows ride their own systems).
    (
        "Molten",
        (231, 232, 233),
        SetStats { melee_damage: 0.1, ..stats() },
    ),
    // `:454` Benefits.Frost — `meleeDamage += 0.1f; rangedDamage += 0.1f`.
    (
        "Frost",
        (684, 685, 686),
        SetStats { melee_damage: 0.1, ranged_damage: 0.1, ..stats() },
    ),
    // `:459` Benefits.Mining — `pickSpeed -= 0.1f` (the port's bonus is the
    // negation).
    (
        "Mining",
        (88, 410, 411),
        SetStats { pick_speed: 0.1, ..stats() },
    ),
    (
        "Mining",
        (5588, 5589, 5590),
        SetStats { pick_speed: 0.1, ..stats() },
    ),
    (
        "Mining",
        (4008, 410, 411),
        SetStats { pick_speed: 0.1, ..stats() },
    ),
    // `:470` Benefits.CobaltMelee — `meleeSpeed += 0.15f`.
    (
        "CobaltMelee",
        (372, 374, 375),
        SetStats { melee_speed: 0.15, ..stats() },
    ),
    // `:477` Benefits.MythrilMelee — `meleeCrit += 10`.
    (
        "MythrilMelee",
        (377, 379, 380),
        SetStats { melee_crit: 10, ..stats() },
    ),
    // `:483` Benefits.AdamantiteMelee — `meleeSpeed += 0.2f; moveSpeed += 0.2f`.
    (
        "AdamantiteMelee",
        (401, 403, 404),
        SetStats { melee_speed: 0.2, move_speed: 0.2, ..stats() },
    ),
    // `:474` Benefits.CrystalAssassin — every class `+= 0.1f`, every crit
    // `+= 10`, `dashType = 5`.
    (
        "CrystalAssassin",
        (4982, 4983, 4984),
        SetStats {
            melee_damage: 0.1,
            ranged_damage: 0.1,
            magic_damage: 0.1,
            minion_damage: 0.1,
            melee_crit: 10,
            ranged_crit: 10,
            magic_crit: 10,
            dash_type: 5,
            ..stats()
        },
    ),
    // `:475` Benefits.Tiki — `maxMinions++` (the `whipRangeMultiplier` row has
    // no consumer yet).
    (
        "Tiki",
        (1159, 1160, 1161),
        SetStats { max_minions: 1, ..stats() },
    ),
    // `:476` Benefits.Spooky — `minionDamage += 0.25f`.
    (
        "Spooky",
        (1832, 1833, 1834),
        SetStats { minion_damage: 0.25, ..stats() },
    ),
    // `:478` Benefits.Spider — `minionDamage += 0.12f`.
    (
        "Spider",
        (2370, 2371, 2372),
        SetStats { minion_damage: 0.12, ..stats() },
    ),
    // `:158` Benefits.Turtle — `endurance += 0.15f; thorns = 1f;
    // turtleThorns = true`.
    (
        "Turtle",
        (1316, 1317, 1318),
        SetStats {
            endurance: 0.15,
            thorns: 1.0,
            turtle_thorns: true,
            ..stats()
        },
    ),
    // `:452` Benefits.Cactus — `cactusThorns = true`.
    (
        "Cactus",
        (894, 895, 896),
        SetStats { cactus_thorns: true, ..stats() },
    ),
    // `:456` Benefits.Jungle — `manaCost -= 0.16f`.
    (
        "Jungle",
        (228, 229, 230),
        SetStats { mana_cost: -0.16, ..stats() },
    ),
    (
        "Jungle",
        (960, 961, 962),
        SetStats { mana_cost: -0.16, ..stats() },
    ),
    // `:466` Benefits.CobaltCaster — `manaCost -= 0.14f`.
    (
        "CobaltCaster",
        (371, 374, 375),
        SetStats { mana_cost: -0.14, ..stats() },
    ),
    // `:474` Benefits.MythrilCaster — `manaCost -= 0.17f`.
    (
        "MythrilCaster",
        (376, 379, 380),
        SetStats { mana_cost: -0.17, ..stats() },
    ),
    // `:480` Benefits.AdamantiteCaster — `manaCost -= 0.19f`.
    (
        "AdamantiteCaster",
        (400, 403, 404),
        SetStats { mana_cost: -0.19, ..stats() },
    ),
    // `:477` Benefits.Bee — `minionDamage += 0.1f`.
    (
        "Bee",
        (2361, 2362, 2363),
        SetStats { minion_damage: 0.1, ..stats() },
    ),
];

/// `ArmorSetBonuses.Benefits`' immunity rows: the Snow set's `buffImmune[46]`
/// and `buffImmune[47]` (`ArmorSetBonuses.cs:298-302`) and the Molten set's
/// `buffImmune[24]` (`:288-295`). Returns the debuff ids the completed set
/// makes the player immune to.
pub fn set_buff_immunity(head: i32, body: i32, legs: i32) -> &'static [i32] {
    const SNOW: [(i32, i32, i32); 2] = [(803, 804, 805), (978, 979, 980)];
    const MOLTEN: (i32, i32, i32) = (231, 232, 233);
    let triple = (head, body, legs);
    if SNOW.contains(&triple) {
        &[46, 47]
    } else if triple == MOLTEN {
        &[24]
    } else {
        &[]
    }
}

/// `ArmorSetBonuses.GetCompleteSet` (`:14-20`) over [`SET_STATS`]: the benefit
/// of the worn triple, if the port models one. `head`/`body`/`legs` are the
/// raw item ids (0 = empty slot).
pub fn set_stats(head: i32, body: i32, legs: i32) -> Option<(&'static str, SetStats)> {
    SET_STATS
        .iter()
        .find(|(_, (h, b, l), _)| *h == head && *b == body && *l == legs)
        .map(|(name, _, stats)| (*name, *stats))
}

/// The per-piece stat rows of `GrantArmorBenefits`' `switch (armorPiece.type)`
/// (`Player.cs:13126-13950`) — the damage/crit/speed scalars every ported armor
/// piece adds while worn. The `manaCost` rows are in `Player::update`'s piece
/// match (they predate this table) and the `maxMinions` rows belong to
/// `grant_summon_caps`, so neither appears here.
pub const ARMOR_STATS: &[(u16, SetStats)] = &[
    (123, SetStats { magic_damage: 0.09, ..stats() }),
    (151, SetStats { ranged_damage: 0.05, ..stats() }),
    (229, SetStats { magic_damage: 0.06, ..stats() }),
    (232, SetStats { melee_damage: 0.07, ..stats() }),
    (233, SetStats { melee_speed: 0.07, ..stats() }),
    (238, SetStats { magic_damage: 0.05, ..stats() }),
    (371, SetStats { magic_damage: 0.1, ..stats() }),
    (372, SetStats { move_speed: 0.1, melee_damage: 0.15, ..stats() }),
    (373, SetStats { ranged_damage: 0.1, ..stats() }),
    (375, SetStats { ranged_damage: 0.03, melee_damage: 0.03, magic_damage: 0.03, minion_damage: 0.03, move_speed: 0.1, ..stats() }),
    (376, SetStats { magic_damage: 0.15, ..stats() }),
    (377, SetStats { melee_damage: 0.1, ..stats() }),
    (378, SetStats { ranged_damage: 0.12, ..stats() }),
    (379, SetStats { ranged_damage: 0.07, melee_damage: 0.07, magic_damage: 0.07, minion_damage: 0.07, ..stats() }),
    (400, SetStats { magic_damage: 0.12, ..stats() }),
    (401, SetStats { melee_damage: 0.14, ..stats() }),
    (402, SetStats { ranged_damage: 0.14, ..stats() }),
    (403, SetStats { ranged_damage: 0.08, melee_damage: 0.08, magic_damage: 0.08, minion_damage: 0.08, ..stats() }),
    (404, SetStats { move_speed: 0.05, ..stats() }),
    (552, SetStats { ranged_damage: 0.07, melee_damage: 0.07, magic_damage: 0.07, minion_damage: 0.07, move_speed: 0.08, ..stats() }),
    (553, SetStats { ranged_damage: 0.15, ..stats() }),
    (558, SetStats { magic_damage: 0.12, ..stats() }),
    (559, SetStats { melee_damage: 0.1, melee_speed: 0.1, ..stats() }),
    (684, SetStats { ranged_damage: 0.16, melee_damage: 0.16, ..stats() }),
    (686, SetStats { move_speed: 0.08, melee_speed: 0.1, ..stats() }),
    (792, SetStats { melee_damage: 0.03, ranged_damage: 0.03, magic_damage: 0.03, minion_damage: 0.03, ..stats() }),
    (1001, SetStats { melee_damage: 0.16, ..stats() }),
    (1002, SetStats { ranged_damage: 0.16, ..stats() }),
    (1003, SetStats { magic_damage: 0.16, ..stats() }),
    (1004, SetStats { melee_damage: 0.05, magic_damage: 0.05, ranged_damage: 0.05, minion_damage: 0.05, ..stats() }),
    (1005, SetStats { move_speed: 0.05, ..stats() }),
    (1159, SetStats { minion_damage: 0.15, ..stats() }),
    (1205, SetStats { melee_damage: 0.12, melee_speed: 0.12, ..stats() }),
    (1206, SetStats { ranged_damage: 0.09, ..stats() }),
    (1207, SetStats { magic_damage: 0.09, ..stats() }),
    (1208, SetStats { melee_damage: 0.03, ranged_damage: 0.03, magic_damage: 0.03, minion_damage: 0.03, ..stats() }),
    (1209, SetStats { melee_damage: 0.02, ranged_damage: 0.02, magic_damage: 0.02, minion_damage: 0.02, ..stats() }),
    (1210, SetStats { melee_damage: 0.11, melee_speed: 0.11, move_speed: 0.07, ..stats() }),
    (1211, SetStats { move_speed: 0.08, ..stats() }),
    (1214, SetStats { move_speed: 0.11, melee_damage: 0.08, ranged_damage: 0.08, magic_damage: 0.08, minion_damage: 0.08, ..stats() }),
    (1215, SetStats { melee_damage: 0.09, melee_speed: 0.09, ..stats() }),
    (1216, SetStats { ranged_damage: 0.16, ..stats() }),
    (1217, SetStats { magic_damage: 0.16, ..stats() }),
    (1218, SetStats { melee_damage: 0.04, ranged_damage: 0.04, magic_damage: 0.04, minion_damage: 0.04, ..stats() }),
    (1219, SetStats { melee_damage: 0.03, ranged_damage: 0.03, magic_damage: 0.03, minion_damage: 0.03, move_speed: 0.06, ..stats() }),
    (1316, SetStats { melee_damage: 0.06, ..stats() }),
    (1317, SetStats { melee_damage: 0.08, ..stats() }),
    (1504, SetStats { magic_damage: 0.07, ..stats() }),
    (1505, SetStats { magic_damage: 0.08, move_speed: 0.08, ..stats() }),
    (1549, SetStats { ranged_damage: 0.13, ..stats() }),
    (1550, SetStats { move_speed: 0.12, ..stats() }),
    (1832, SetStats { minion_damage: 0.11, ..stats() }),
    (1833, SetStats { minion_damage: 0.11, ..stats() }),
    (1834, SetStats { move_speed: 0.2, minion_damage: 0.11, ..stats() }),
    (2189, SetStats { magic_damage: 0.1, ..stats() }),
    (2199, SetStats { melee_damage: 0.06, ..stats() }),
    (2200, SetStats { melee_damage: 0.08, melee_speed: 0.06, move_speed: 0.06, ..stats() }),
    (2201, SetStats { melee_damage: 0.05, ..stats() }),
    (2202, SetStats { melee_speed: 0.06, move_speed: 0.06, ..stats() }),
    (2275, SetStats { magic_damage: 0.06, ..stats() }),
    (2277, SetStats { magic_damage: 0.05, melee_damage: 0.05, ranged_damage: 0.05, minion_damage: 0.05, melee_speed: 0.1, move_speed: 0.1, ..stats() }),
    (2279, SetStats { magic_damage: 0.06, ..stats() }),
    (2361, SetStats { minion_damage: 0.04, ..stats() }),
    (2362, SetStats { minion_damage: 0.04, ..stats() }),
    (2363, SetStats { minion_damage: 0.05, ..stats() }),
    (2372, SetStats { minion_damage: 0.06, ..stats() }),
    (2757, SetStats { ranged_damage: 0.16, ..stats() }),
    (2758, SetStats { ranged_damage: 0.12, ..stats() }),
    (2759, SetStats { ranged_damage: 0.08, move_speed: 0.1, ..stats() }),
    (2760, SetStats { magic_damage: 0.07, ..stats() }),
    (2761, SetStats { magic_damage: 0.09, ..stats() }),
    (2762, SetStats { move_speed: 0.1, magic_damage: 0.1, ..stats() }),
    (2764, SetStats { melee_damage: 0.29, ..stats() }),
    (2765, SetStats { melee_speed: 0.15, move_speed: 0.15, ..stats() }),
    (3266, SetStats { minion_damage: 0.08, ..stats() }),
    (3268, SetStats { minion_damage: 0.08, ..stats() }),
    (3375, SetStats { ranged_damage: 0.05, ..stats() }),
    (3381, SetStats { minion_damage: 0.22, ..stats() }),
    (3382, SetStats { minion_damage: 0.22, ..stats() }),
    (3776, SetStats { magic_damage: 0.15, minion_damage: 0.15, ..stats() }),
    (3777, SetStats { minion_damage: 0.1, ..stats() }),
    (3778, SetStats { magic_damage: 0.1, ..stats() }),
    (3797, SetStats { magic_damage: 0.1, ..stats() }),
    (3798, SetStats { magic_damage: 0.1, minion_damage: 0.2, ..stats() }),
    (3799, SetStats { minion_damage: 0.1, move_speed: 0.2, ..stats() }),
    (3801, SetStats { melee_damage: 0.15, minion_damage: 0.15, ..stats() }),
    (3802, SetStats { minion_damage: 0.15, move_speed: 0.15, ..stats() }),
    (3804, SetStats { ranged_damage: 0.2, minion_damage: 0.2, ..stats() }),
    (3805, SetStats { minion_damage: 0.1, move_speed: 0.2, ..stats() }),
    (3806, SetStats { melee_speed: 0.2, ..stats() }),
    (3807, SetStats { melee_damage: 0.2, minion_damage: 0.2, ..stats() }),
    (3808, SetStats { minion_damage: 0.1, move_speed: 0.2, ..stats() }),
    (3871, SetStats { melee_damage: 0.1, minion_damage: 0.1, ..stats() }),
    (3872, SetStats { minion_damage: 0.3, ..stats() }),
    (3873, SetStats { minion_damage: 0.2, move_speed: 0.2, ..stats() }),
    (3874, SetStats { magic_damage: 0.15, minion_damage: 0.15, ..stats() }),
    (3875, SetStats { minion_damage: 0.25, magic_damage: 0.1, ..stats() }),
    (3876, SetStats { minion_damage: 0.2, move_speed: 0.2, ..stats() }),
    (3877, SetStats { minion_damage: 0.1, ..stats() }),
    (3878, SetStats { minion_damage: 0.25, ranged_damage: 0.25, ..stats() }),
    (3879, SetStats { minion_damage: 0.25, move_speed: 0.2, ..stats() }),
    (3880, SetStats { minion_damage: 0.2, melee_damage: 0.2, ..stats() }),
    (3881, SetStats { melee_speed: 0.2, minion_damage: 0.2, ..stats() }),
    (3882, SetStats { minion_damage: 0.2, move_speed: 0.3, ..stats() }),
    (4873, SetStats { minion_damage: 0.1, ..stats() }),
    (4983, SetStats { ranged_damage: 0.05, melee_damage: 0.05, magic_damage: 0.05, minion_damage: 0.05, ..stats() }),
    (4984, SetStats { melee_speed: 0.1, move_speed: 0.2, ..stats() }),
    (5068, SetStats { minion_damage: 0.05, ..stats() }),
    (5524, SetStats { minion_damage: 0.16, ..stats() }),
];

/// The worn piece's [`ARMOR_STATS`] row, if any.
pub fn armor_stats(item: u16) -> Option<SetStats> {
    ARMOR_STATS
        .iter()
        .find(|(id, _)| *id == item)
        .map(|(_, stats)| *stats)
}
