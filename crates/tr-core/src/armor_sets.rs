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
