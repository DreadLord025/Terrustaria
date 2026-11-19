//! S1-C-12 wave B — the combat/damage accessory family of
//! `Player.ApplyEquipFunctional` (`Player.cs:14368-15604`): the class
//! emblems, the Avenger/Destroyer emblems, the Putrid Scent, the Flesh
//! Knuckles, the melee gloves (Power/Mechanical/Berserker/Fire) and the
//! Celestial Stone bundle. Each per-accessory arm is its own fn carrying
//! the C# citation; [`apply_combat_accessory`] is the seam the
//! `UpdateEquip` grant loop (wave A, `player.rs`) calls per slot —
//! WIRED in wave E (S1-C-12e): `apply_equip_functional` delegates the
//! family kinds here FIRST and the inline combat arms were deleted, so
//! each family kind has exactly one grant path. The dispatch follows
//! the C# source order and leaves every non-family kind to the existing
//! match. The bundle's flag arms are ported in
//! [`set_sky_stone_flag`] and its `UpdateEquips` stat fold in
//! [`apply_sky_stone_effects`] (`Player.cs:12989-13003`).
//!
//! Deviations (also in ANALYSIS): `aggro` has no port field (no
//! multiplayer aggro consumer — same precedent as the Scout Sling row
//! in `player.rs`), and `autoReuseGlove`/`meleeScaleGlove` have no port
//! fields or consumers yet (the Titan Glove row, `player.rs`), so those
//! halves are documented and skipped. No RNG rides these arms.

use crate::player::Player;

/// The Sorcerer Emblem (`Player.cs:15428-15431`): `magicDamage += 0.15f`.
fn sorcerer_emblem(p: &mut Player) {
    p.magic_damage += 0.15;
}

/// The Warrior Emblem (`Player.cs:15432-15435`): `meleeDamage += 0.15f`.
fn warrior_emblem(p: &mut Player) {
    p.melee_damage += 0.15;
}

/// The Ranger Emblem (`Player.cs:15436-15439`): `rangedDamage += 0.15f`.
fn ranger_emblem(p: &mut Player) {
    p.ranged_damage += 0.15;
}

/// The Summoner Emblem (`Player.cs:15440-15443`): `minionDamage += 0.15f`.
fn summoner_emblem(p: &mut Player) {
    p.minion_damage += 0.15;
}

/// The Avenger Emblem (`Player.cs:15444-15450`): +12% to all four damage
/// classes, in the C# order magic → melee → ranged → minion.
fn avenger_emblem(p: &mut Player) {
    p.magic_damage += 0.12;
    p.melee_damage += 0.12;
    p.ranged_damage += 0.12;
    p.minion_damage += 0.12;
}

/// The Destroyer Emblem (`Player.cs:14912-14921`): +8 melee/ranged/magic
/// crit and +10% to all four damage classes. (The port's existing match
/// arm cited "Celestial Stone" for 1301 — 1301 is `ItemID.DestroyerEmblem`;
/// the Celestial Stone is 1865 and rides the sky-stone bundle below.)
fn destroyer_emblem(p: &mut Player) {
    p.melee_crit += 8;
    p.ranged_crit += 8;
    p.magic_crit += 8;
    p.melee_damage += 0.1;
    p.ranged_damage += 0.1;
    p.magic_damage += 0.1;
    p.minion_damage += 0.1;
}

/// The Putrid Scent (`Player.cs:14636-14646`): `aggro -= 400` (no port
/// field — skipped, no aggro consumer), +5 melee/magic/ranged crit and
/// +5% to all four damage classes.
fn putrid_scent(p: &mut Player) {
    p.melee_crit += 5;
    p.magic_crit += 5;
    p.ranged_crit += 5;
    p.melee_damage += 0.05;
    p.magic_damage += 0.05;
    p.ranged_damage += 0.05;
    p.minion_damage += 0.05;
}

/// The Flesh Knuckles (`Player.cs:14647-14650`): `aggro += 400` only.
/// `aggro` has no port field, so the row grants nothing until an aggro
/// consumer lands — kept as an explicit arm so the mapping is complete.
fn flesh_knuckles(_p: &mut Player) {}

/// The Power Glove (`Player.cs:15299-15305`): `kbGlove = true`,
/// `autoReuseGlove = true`, `meleeScaleGlove = true` (the two glove
/// flags have no port fields/consumers yet — skipped), `meleeSpeed +=
/// 0.12f`.
fn power_glove(p: &mut Player) {
    p.kb_glove = true;
    p.melee_speed_bonus += 0.12;
}

/// The Mechanical Glove (`Player.cs:15369-15376`): the Power Glove's
/// rows plus `meleeDamage += 0.12f`.
fn mechanical_glove(p: &mut Player) {
    p.kb_glove = true;
    p.melee_speed_bonus += 0.12;
    p.melee_damage += 0.12;
}

/// The Berserker's Glove (`Player.cs:14411-14417`): the POWER Glove's
/// rows (`kbGlove`/`autoReuseGlove`/`meleeScaleGlove`/`meleeSpeed`) plus
/// `aggro += 400` (no port field — skipped). NOTE: unlike the Mechanical
/// Glove/Fire Gauntlet there is NO `meleeDamage` row here — wave B's
/// original arm copied the Mechanical +0.12 in error; wave E's re-read
/// of the switch arm removed it (S1-C-12e dedupe).
fn berserker_glove(p: &mut Player) {
    p.kb_glove = true;
    p.melee_speed_bonus += 0.12;
}

/// The Fire Gauntlet (`Player.cs:15306-15314`): the Mechanical Glove's
/// rows plus `magmaStone = true` (S1-P-i10 — the melee OnFire3 consumer
/// is `melee_on_hit.rs::status_to_npc`, `:6552-6566`; the projectile
/// consumers stay unported). `autoReuseGlove`/`meleeScaleGlove` have no
/// port fields yet — skipped like the Mechanical Glove's arm.
fn fire_gauntlet(p: &mut Player) {
    p.kb_glove = true;
    p.melee_speed_bonus += 0.12;
    p.melee_damage += 0.12;
    p.magma_stone = true;
}

/// The `ApplyEquipFunctional` seam for the combat/damage family: returns
/// `true` when `kind` is one of the family's items and the arm was
/// applied, `false` for everything else (the caller's existing match
/// keeps those). The dispatch order follows the C# source order
/// (`:14411` → `:14636` → `:14647` → `:14912` → `:15299` → `:15306` →
/// `:15369` → `:15428`-`:15450`); each arm touches its own item only,
/// so the order is observationally irrelevant within one slot.
pub(crate) fn apply_combat_accessory(kind: u16, p: &mut Player) -> bool {
    match kind {
        // `Player.cs:14411-14417` (the switch arm).
        3992 => berserker_glove(p),
        // `Player.cs:14636-14650`.
        3015 => putrid_scent(p),
        3016 => flesh_knuckles(p),
        // `Player.cs:14912-14921`.
        1301 => destroyer_emblem(p),
        // `Player.cs:15299-15314`.
        897 => power_glove(p),
        1343 => fire_gauntlet(p),
        // `Player.cs:15369-15376`.
        936 => mechanical_glove(p),
        // `Player.cs:15428-15450`.
        489 => sorcerer_emblem(p),
        490 => warrior_emblem(p),
        491 => ranger_emblem(p),
        2998 => summoner_emblem(p),
        935 => avenger_emblem(p),
        _ => return false,
    }
    true
}

/// The sky-stone flag arms (`Player.cs:15412-15423`): the Celestial
/// Stone (1865) and Celestial Shell (3110) always set `skyStoneEffects`;
/// the Sun Stone (899) only by day, the Moon Stone (900) only at night
/// or during an eclipse. The port folds the whole `equipped[..10]`
/// scan into this call (`player.rs` sets the flag from the same rows).
pub(crate) fn set_sky_stone_flag(equipped: &[u16], day_time: bool, eclipse: bool) -> bool {
    equipped.contains(&1865)
        || equipped.contains(&3110)
        || (equipped.contains(&899) && day_time)
        || (equipped.contains(&900) && (!day_time || eclipse))
}

/// The `UpdateEquips` sky-stone bundle (`Player.cs:12989-13003`):
/// `lifeRegen += 2` (the port's `accessory_life_regen` fold runs it in
/// `player.rs` — this fn applies only the stat halves), `statDefense
/// += 4`, `meleeSpeed += 0.1f`, +10% melee/ranged/magic/minion damage,
/// +2 melee/ranged/magic crit, `pickSpeed -= 0.15f` (the port's bonus
/// field is the negation) and `minionKB += 0.5f`.
pub(crate) fn apply_sky_stone_effects(p: &mut Player) {
    p.stat_defense += 4;
    p.melee_speed_bonus += 0.1;
    p.melee_damage += 0.1;
    p.melee_crit += 2;
    p.ranged_damage += 0.1;
    p.ranged_crit += 2;
    p.magic_damage += 0.1;
    p.magic_crit += 2;
    // `pickSpeed -= 0.15f` — the port's bonus field is the negation.
    p.pick_speed_bonus += 0.15;
    p.minion_damage += 0.1;
    p.minion_kb += 0.5;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::Player;
    use crate::test_world_helpers;

    /// A spawned player (baseline: damage classes 1.0, crits 4, speed
    /// bonus 0 — `Player::spawn`, `player.rs:1608-1621`).
    fn fresh() -> Player {
        Player::spawn(&test_world_helpers::ground())
    }

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn the_class_emblems_raise_only_their_class_by_15_percent() {
        // `Player.cs:15428-15443`.
        let mut p = fresh();
        assert!(apply_combat_accessory(489, &mut p));
        assert!(near(p.magic_damage, 1.15));
        assert!(near(p.melee_damage, 1.0));
        assert!(near(p.ranged_damage, 1.0));
        assert!(near(p.minion_damage, 1.0));

        let mut p = fresh();
        assert!(apply_combat_accessory(490, &mut p));
        assert!(near(p.melee_damage, 1.15));
        assert!(near(p.magic_damage, 1.0));

        let mut p = fresh();
        assert!(apply_combat_accessory(491, &mut p));
        assert!(near(p.ranged_damage, 1.15));
        assert!(near(p.melee_damage, 1.0));

        let mut p = fresh();
        assert!(apply_combat_accessory(2998, &mut p));
        assert!(near(p.minion_damage, 1.15));
        assert!(near(p.ranged_damage, 1.0));
    }

    #[test]
    fn the_avenger_emblem_raises_all_four_classes_by_12_percent() {
        // `Player.cs:15444-15450`.
        let mut p = fresh();
        assert!(apply_combat_accessory(935, &mut p));
        assert!(near(p.melee_damage, 1.12));
        assert!(near(p.ranged_damage, 1.12));
        assert!(near(p.magic_damage, 1.12));
        assert!(near(p.minion_damage, 1.12));
        // No crit or speed rows.
        assert_eq!(p.melee_crit, 4);
        assert!(near(p.melee_speed_bonus, 0.0));
    }

    #[test]
    fn the_destroyer_emblem_adds_8_crit_and_10_percent_damage() {
        // `Player.cs:14912-14921` — the three non-minion crits +8 (the
        // spawn baseline is 4), all four damage classes +10%.
        let mut p = fresh();
        assert!(apply_combat_accessory(1301, &mut p));
        assert_eq!(p.melee_crit, 12);
        assert_eq!(p.ranged_crit, 12);
        assert_eq!(p.magic_crit, 12);
        assert!(near(p.melee_damage, 1.1));
        assert!(near(p.ranged_damage, 1.1));
        assert!(near(p.magic_damage, 1.1));
        assert!(near(p.minion_damage, 1.1));
    }

    #[test]
    fn the_putrid_scent_adds_5_crit_and_5_percent_damage() {
        // `Player.cs:14636-14646` (the `aggro -= 400` half has no port
        // field).
        let mut p = fresh();
        assert!(apply_combat_accessory(3015, &mut p));
        assert_eq!(p.melee_crit, 9);
        assert_eq!(p.ranged_crit, 9);
        assert_eq!(p.magic_crit, 9);
        assert!(near(p.melee_damage, 1.05));
        assert!(near(p.ranged_damage, 1.05));
        assert!(near(p.magic_damage, 1.05));
        assert!(near(p.minion_damage, 1.05));
    }

    #[test]
    fn the_flesh_knuckles_grant_only_the_unported_aggro() {
        // `Player.cs:14647-14650` — the arm is claimed (returns true)
        // but every stat is untouched until an aggro field lands.
        let mut p = fresh();
        assert!(apply_combat_accessory(3016, &mut p));
        assert!(near(p.melee_damage, 1.0));
        assert!(near(p.minion_damage, 1.0));
        assert_eq!(p.melee_crit, 4);
        assert!(near(p.melee_speed_bonus, 0.0));
        assert!(!p.kb_glove);
    }

    #[test]
    fn the_glove_family_layers_speed_damage_and_knockback() {
        // Power Glove (`Player.cs:15299-15305`): speed + knockback only.
        let mut p = fresh();
        assert!(apply_combat_accessory(897, &mut p));
        assert!(p.kb_glove);
        assert!(near(p.melee_speed_bonus, 0.12));
        assert!(near(p.melee_damage, 1.0));

        // Mechanical Glove (`Player.cs:15369-15376`): + melee damage.
        let mut p = fresh();
        assert!(apply_combat_accessory(936, &mut p));
        assert!(p.kb_glove);
        assert!(near(p.melee_speed_bonus, 0.12));
        assert!(near(p.melee_damage, 1.12));

        // Berserker's Glove (`Player.cs:14411-14417`): the POWER Glove
        // rows + the unported aggro — the C# arm has NO meleeDamage row
        // (wave B's +0.12 copy of the Mechanical row was removed in
        // wave E's re-read).
        let mut p = fresh();
        assert!(apply_combat_accessory(3992, &mut p));
        assert!(p.kb_glove);
        assert!(near(p.melee_speed_bonus, 0.12));
        assert!(near(p.melee_damage, 1.0), "no meleeDamage row at :14411");

        // Fire Gauntlet (`Player.cs:15306-15314`): the Mechanical rows
        // + the magma stone flag (S1-P-i10 — the melee OnFire3 consumer
        // is `melee_on_hit.rs`).
        let mut p = fresh();
        assert!(apply_combat_accessory(1343, &mut p));
        assert!(p.kb_glove);
        assert!(near(p.melee_speed_bonus, 0.12));
        assert!(near(p.melee_damage, 1.12));
        assert!(p.magma_stone, "the gauntlet arms the magma stone (:15313)");
    }

    #[test]
    fn the_family_stacks_across_slots() {
        // Two emblems in two slots: the loop applies each arm once.
        let mut p = fresh();
        assert!(apply_combat_accessory(490, &mut p));
        assert!(apply_combat_accessory(935, &mut p));
        assert!(near(p.melee_damage, 1.27));
        assert!(near(p.ranged_damage, 1.12));
    }

    #[test]
    fn non_family_kinds_fall_through() {
        let mut p = fresh();
        for kind in [0u16, 53, 54, 405, 1865, 3110, 899, 900, 1858, 2220, 3224] {
            assert!(!apply_combat_accessory(kind, &mut p), "kind {kind}");
        }
        assert!(near(p.melee_damage, 1.0));
        assert_eq!(p.melee_crit, 4);
    }

    #[test]
    fn the_sky_stone_flag_follows_the_day_night_gates() {
        // `Player.cs:15412-15423`.
        let stone = [1865u16];
        let shell = [3110u16];
        let sun = [899u16];
        let moon = [900u16];
        assert!(set_sky_stone_flag(&stone, true, false));
        assert!(set_sky_stone_flag(&stone, false, false));
        assert!(set_sky_stone_flag(&shell, false, false));
        assert!(set_sky_stone_flag(&sun, true, false));
        assert!(!set_sky_stone_flag(&sun, false, false));
        // An eclipse keeps the Sun Stone off (the C# gate is dayTime
        // only) and the Moon Stone on.
        assert!(!set_sky_stone_flag(&sun, false, true));
        assert!(!set_sky_stone_flag(&moon, true, false));
        assert!(set_sky_stone_flag(&moon, false, false));
        assert!(set_sky_stone_flag(&moon, true, true));
        assert!(!set_sky_stone_flag(&[0u16], false, false));
    }

    #[test]
    fn the_sky_stone_bundle_folds_the_update_equips_stats() {
        // `Player.cs:12989-13003` (the `lifeRegen += 2` half rides the
        // port's `accessory_life_regen` fold in `player.rs`).
        let mut p = fresh();
        apply_sky_stone_effects(&mut p);
        assert_eq!(p.stat_defense, 4);
        assert!(near(p.melee_speed_bonus, 0.1));
        assert!(near(p.melee_damage, 1.1));
        assert!(near(p.ranged_damage, 1.1));
        assert!(near(p.magic_damage, 1.1));
        assert!(near(p.minion_damage, 1.1));
        assert_eq!(p.melee_crit, 6);
        assert_eq!(p.ranged_crit, 6);
        assert_eq!(p.magic_crit, 6);
        assert!(near(p.pick_speed_bonus, 0.15));
        assert!(near(p.minion_kb, 0.5));
    }
}
