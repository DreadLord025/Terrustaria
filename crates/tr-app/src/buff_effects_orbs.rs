//! W4-c16 — the FIRST missing block of `Player.UpdateBuffs`: the SugarRush /
//! BrainOfConfusion stragglers (`Player.cs:9935-9947`) and the ManaHeat /
//! lunar-orb tail (`Player.cs:10328-10461`), 21 ids.
//!
//! Live-set census (docs/research/W4-c16-buff-effects-wave-b.md): every other
//! id in the `:9855-12069` chain already has a live effect (the `player.rs`
//! wave-A match, the summon keep-alives, the wired wave-C accessory pass, the
//! DoT `holding()` derivation, the regen fold or the light feed) or is claimed
//! by the unwired wave-B (`buff_effects.rs`) / wave-D (`buff_effects_misc.rs`)
//! tables. This module owns ONLY the 21 ids no path touches:
//! 192, 321, 396, 95-100 (beetle), 170-172 (solar), 173-181 (nebula).
//!
//! Disjoint-wave design (the B/D pattern): this module is a PURE effect table
//! — `apply_orb_effect_arm` matches one `buffType[j]` id and folds its writes
//! into an `OrbEffects` accumulator, exactly like the C# arm writes into the
//! `Player` fields. The wiring step (the merge, not this wave) copies the
//! accumulator onto the live fields and drains the per-slot requests
//! (`buff_time_set`, `buff_type_decrement`, `delete_buff`/`rescan_slot`,
//! `purge_band`, `mana_potion_duration_reset`) IN SLOT ORDER, so the
//! cross-slot reconciles see the same bar the C# sees.
//!
//! Sequential-state rule (the wave-B `thorns` pattern): the caller SEEDS
//! `e.beetle_orbs` / `e.solar_shields` from the live fields before the scan;
//! the arms read and assign them per slot, matching the C#'s live
//! `this.beetleOrbs` / `this.solarShields` reads inside the chain.
//!
//! RNG census: ZERO `Main.rand` draws in every arm (arithmetic + slot
//! bookkeeping only) — the block is stream-neutral by construction.

/// The per-tick context the arms read besides `buffType`/`buffTime`.
/// `whoAmI == Main.myPlayer` collapses to true in the single-player port,
/// matching every other ported `UpdateBuffs` arm.
#[derive(Clone, Copy, Debug, Default)]
pub struct OrbEffectContext {
    /// `beetleDefense` (`Player.cs:10366`) — the Beetle Shell set bonus
    /// (`ApplySetBonus_BeetleDefense` `:16499-16501`, unported; false until
    /// it lands, so the endurance arms zero + delete exactly like the C#
    /// without the set).
    pub beetle_defense: bool,
    /// `beetleOffense` (`:10433`) — the Beetle Scale Mail set bonus
    /// (`ApplySetBonus_BeetleDamage` `:16529-16532`, unported).
    pub beetle_offense: bool,
    /// `setSolar` (`:10401`) — the Solar Flare set bonus
    /// (`ApplySetBonus_Solar` `:16366-16369`, unported).
    pub set_solar: bool,
    /// `latestManaPotionDamageBonus` (`:10330`) — written by
    /// `TryApplyManaHeat` (`:52008`, ported as `Player::try_apply_mana_heat`).
    pub latest_mana_potion_damage_bonus: f32,
    /// `latestManaPotionDuration` (`:10331-10335`) — the 0 → 1 guard writes
    /// the PLAYER field; the accumulator records the reset request.
    pub latest_mana_potion_duration: f32,
}

/// One tick's accumulated writes of the block. Flags OR, stats ADD per slot
/// exactly like the C# chain; the `Option` setters are assignments (the last
/// live slot wins, like `beetleOrbs = num8` / `nebulaLevel = …`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OrbEffects {
    // Stat adds (per-slot, summed like the C# chain).
    /// 192 `pickSpeed -= 0.2f` (`:9937`) — raw C# sign; the wiring maps it
    /// onto the port's positive `pick_speed_bonus` convention (+0.2).
    pub pick_speed: f32,
    /// 192 `moveSpeed += 0.2f` (`:9938`).
    pub move_speed: f32,
    /// 321 `meleeCrit/rangedCrit/magicCrit += 10` (`:9943-9945`).
    pub melee_crit: i32,
    pub ranged_crit: i32,
    pub magic_crit: i32,
    /// 321 `minionDamage += (float)num / 100f` (`:9946`, num = 10 → 0.1).
    pub minion_damage: f32,
    /// 98-100 `meleeDamage += 0.1f * beetleOrbs` (`:10431`); 179-181 add
    /// `0.15f * nebulaLevelDamage` (`:10456-10457`).
    pub melee_damage: f32,
    /// 179-181 (`:10458`).
    pub ranged_damage: f32,
    /// 179-181 (`:10459`).
    pub magic_damage: f32,
    /// 98-100 `meleeSpeed += 0.1f * beetleOrbs` (`:10432`).
    pub melee_speed: f32,
    /// 173-175 `lifeRegen += 6 * nebulaLevelLife` (`:10451`).
    pub life_regen: i32,
    /// 396 `manaHeat += num6` (`:10340`) — the decayed latest-potion bonus.
    pub mana_heat: f32,
    /// 396's `latestManaPotionDuration = 1f` guard write (`:10333`) — the
    /// wiring applies it BEFORE the next slot so later 396 slots divide by 1.
    pub mana_potion_duration_reset: bool,
    // The orb/shield counters — SEEDED from the live fields by the caller,
    // read + assigned per slot (the wave-B `thorns` sequential pattern).
    /// `beetleOrbs` (`:10365`/`:10430`, zeroed at `:10368`/`:10435`).
    pub beetle_orbs: i32,
    /// `solarShields` (`:10400`, zeroed at `:10403`).
    pub solar_shields: i32,
    /// `beetleBuff = true` (`:10374`/`:10441`) — the set-gated flag.
    pub beetle_buff: bool,
    /// `nebulaLevelLife/Mana/Damage` assignments (`UpdateBuffs_NebulaBuffs`
    /// `:12072`, post-downgrade) — last live slot wins.
    pub nebula_level_life: Option<i32>,
    pub nebula_level_mana: Option<i32>,
    pub nebula_level_damage: Option<i32>,
    // Per-slot requests — the wiring drains them after EACH arm call.
    /// `buffTime[j] = …` — 5 (beetle endurance `:10344`, solar `:10379`),
    /// 480 (the nebula tier-downgrade `:12077`).
    pub buff_time_set: Option<i32>,
    /// The nebula downgrade's `buffType[b]--` (`:12076`).
    pub buff_type_decrement: bool,
    /// `DelBuff(j)` (`:10350`/`:10369`/`:10385`/`:10404`/`:10415`/`:10436`).
    pub delete_buff: bool,
    /// The `j--` rescan after a self-delete (every `DelBuff(j)` site here
    /// pairs with `j--`).
    pub rescan_slot: bool,
    /// The upgrade reconcile's whole-bar purge (`:10355-10362` /
    /// `:10390-10397` / `:10420-10427`): `DelBuff(m); m--` for every slot
    /// whose id lies in the INCLUSIVE band `(base, base + tier - 1)`. The
    /// band's upper end equals the CURRENT slot's id — the C# m-loop deletes
    /// the current slot too, and the outer `j` is NOT decremented afterwards
    /// (the shifted-in buff skips its arm this tick). The wiring replicates
    /// the m-loop verbatim, quirk included.
    pub purge_band: Option<(i32, i32)>,
}

/// `Player.UpdateBuffs_NebulaBuffs` (`Player.cs:12070-12079`): the level is
/// `1 + buffType[b] - baseBuffId`; at `buffTime[b] == 2` with level > 1 the
/// tier downgrades (`level--`, `buffType[b]--`, `buffTime[b] = 480`). Returns
/// the post-downgrade level the caller assigns + reads for its stat add.
fn nebula_level(e: &mut OrbEffects, buff_type: i32, base: i32, buff_time: i32) -> i32 {
    let mut level = 1 + buff_type - base; // `:12072`
    if buff_time == 2 && level > 1 {
        level -= 1; // `:12075`
        e.buff_type_decrement = true; // `:12076`
        e.buff_time_set = Some(480); // `:12077`
    }
    level
}

/// The beetle/solar upgrade reconcile shared by the three orb arms
/// (`:10346-10364` / `:10381-10399` / `:10411-10429`): a HIGHER existing
/// count self-deletes this slot; a LOWER one purges the whole-bar band
/// `base..=base + tier - 1` (the m-loop, current slot included).
fn orb_reconcile(e: &mut OrbEffects, current: i32, tier: i32, base: i32) {
    if current > 0 && current != tier {
        if current > tier {
            e.delete_buff = true; // `:10350`/`:10385`/`:10415`
            e.rescan_slot = true; // `:10351`/`:10386`/`:10416`
        } else {
            e.purge_band = Some((base, base + tier - 1)); // `:10357`/`:10392`/`:10422`
        }
    }
}

/// One arm of the block. Returns `true` when the id is handled (the caller's
/// `else if` chain stops), `false` for ids outside this wave's range — the
/// dispatch hook the wiring step runs per active slot, in slot order.
/// `buff_time` is the slot's CURRENT time (after the timer head's decrement,
/// `:9871-9874`).
pub fn apply_orb_effect_arm(
    e: &mut OrbEffects,
    ctx: &OrbEffectContext,
    buff_type: i32,
    buff_time: i32,
) -> bool {
    match buff_type {
        192 => {
            // SugarRush (`Player.cs:9935-9939`).
            e.pick_speed -= 0.2; // `:9937`
            e.move_speed += 0.2; // `:9938`
        }
        321 => {
            // BrainOfConfusionBuff (`:9940-9947`): `int num = 10` — the three
            // crits +10 and `minionDamage += (float)num / 100f` (NOT minion
            // crit; the C# has no minionCrit field).
            e.melee_crit += 10; // `:9943`
            e.ranged_crit += 10; // `:9944`
            e.magic_crit += 10; // `:9945`
            e.minion_damage += 10.0f32 / 100.0f32; // `:9946`
        }
        396 => {
            // ManaHeat (`:10328-10341`): the latest-potion bonus decays with
            // the remaining time. The `if (0f == 0f)` branch (`:10336`) is
            // constants-true — folded.
            let mut num6 = ctx.latest_mana_potion_damage_bonus; // `:10330`
            let duration = if ctx.latest_mana_potion_duration == 0.0 {
                e.mana_potion_duration_reset = true; // `:10333`
                1.0f32
            } else {
                ctx.latest_mana_potion_duration
            };
            let num7 = buff_time as f32 / duration; // `:10335`
            num6 *= num7; // `:10338`
            e.mana_heat += num6; // `:10340`
        }
        95..=97 => {
            // BeetleEndurance 1-3 (`:10342-10376`).
            e.buff_time_set = Some(5); // `:10344`
            let tier = 1 + buff_type - 95; // `:10345` `(byte)(1 + …)`
            orb_reconcile(e, e.beetle_orbs, tier, 95);
            e.beetle_orbs = tier; // `:10365`
            if !ctx.beetle_defense {
                e.beetle_orbs = 0; // `:10368`
                e.delete_buff = true; // `:10369`
                e.rescan_slot = true; // `:10370`
            } else {
                e.beetle_buff = true; // `:10374`
            }
        }
        170..=172 => {
            // SolarShield 1-3 (`:10377-10407`) — no `else` flag (the C# arm
            // has none; the shield count IS the state).
            e.buff_time_set = Some(5); // `:10379`
            let tier = 1 + buff_type - 170; // `:10380`
            orb_reconcile(e, e.solar_shields, tier, 170);
            e.solar_shields = tier; // `:10400`
            if !ctx.set_solar {
                e.solar_shields = 0; // `:10403`
                e.delete_buff = true; // `:10404`
                e.rescan_slot = true; // `:10405`
            }
        }
        98..=100 => {
            // BeetleMight 1-3 (`:10408-10443`) — NO time pin (unlike the
            // endurance arm); the stat adds land BEFORE the offense gate.
            let tier = 1 + buff_type - 98; // `:10410`
            orb_reconcile(e, e.beetle_orbs, tier, 98);
            e.beetle_orbs = tier; // `:10430`
            e.melee_damage += 0.1f32 * e.beetle_orbs as f32; // `:10431`
            e.melee_speed += 0.1f32 * e.beetle_orbs as f32; // `:10432`
            if !ctx.beetle_offense {
                e.beetle_orbs = 0; // `:10435`
                e.delete_buff = true; // `:10436`
                e.rescan_slot = true; // `:10437`
            } else {
                e.beetle_buff = true; // `:10441`
            }
        }
        176..=178 => {
            // NebulaUpMana 1-3 (`:10444-10447`) — the level only; the mana
            // regen consumer (`UpdateManaRegen` `:19567-19583`) reads
            // `nebulaLevelMana` and rides its own lane.
            e.nebula_level_mana = Some(nebula_level(e, buff_type, 176, buff_time));
        }
        173..=175 => {
            // NebulaUpLife 1-3 (`:10448-10452`).
            let level = nebula_level(e, buff_type, 173, buff_time);
            e.nebula_level_life = Some(level);
            e.life_regen += 6 * level; // `:10451`
        }
        179..=181 => {
            // NebulaUpDmg 1-3 (`:10453-10461`).
            let level = nebula_level(e, buff_type, 179, buff_time);
            e.nebula_level_damage = Some(level);
            let num12 = 0.15f32 * level as f32; // `:10456`
            e.melee_damage += num12; // `:10457`
            e.ranged_damage += num12; // `:10458`
            e.magic_damage += num12; // `:10459`
            e.minion_damage += num12; // `:10460`
        }
        _ => return false,
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply(id: i32) -> OrbEffects {
        let mut e = OrbEffects::default();
        assert!(apply_orb_effect_arm(&mut e, &OrbEffectContext::default(), id, 600));
        e
    }

    #[test]
    fn w4_c16_sugar_rush_pick_and_move() {
        let e = apply(192); // `Player.cs:9935-9939`
        assert_eq!(e.pick_speed, -0.2); // `:9937`
        assert_eq!(e.move_speed, 0.2); // `:9938`
    }

    #[test]
    fn w4_c16_brain_of_confusion_crits_and_minion_damage() {
        let e = apply(321); // `:9940-9947`
        assert_eq!((e.melee_crit, e.ranged_crit, e.magic_crit), (10, 10, 10));
        assert_eq!(e.minion_damage, 0.1); // `:9946` `(float)10 / 100f`
    }

    #[test]
    fn w4_c16_mana_heat_decays_with_remaining_time() {
        let mut e = OrbEffects::default();
        let ctx = OrbEffectContext {
            latest_mana_potion_damage_bonus: 0.2,
            latest_mana_potion_duration: 120.0,
            ..Default::default()
        };
        apply_orb_effect_arm(&mut e, &ctx, 396, 60); // `:10335` 60/120 = 0.5
        assert!((e.mana_heat - 0.1).abs() < 1e-6); // `:10340`
        assert!(!e.mana_potion_duration_reset);
    }

    #[test]
    fn w4_c16_mana_heat_zero_duration_guard_writes_one() {
        let mut e = OrbEffects::default();
        let ctx = OrbEffectContext {
            latest_mana_potion_damage_bonus: 0.2,
            latest_mana_potion_duration: 0.0,
            ..Default::default()
        };
        apply_orb_effect_arm(&mut e, &ctx, 396, 60); // `:10331-10334`
        assert!(e.mana_potion_duration_reset); // the field write request
        assert!((e.mana_heat - 12.0).abs() < 1e-4); // 0.2 * (60/1)
    }

    #[test]
    fn w4_c16_beetle_endurance_without_the_set_zeroes_and_deletes() {
        for id in [95, 96, 97] {
            let e = apply(id); // `:10342-10376`, beetleDefense false
            assert_eq!(e.buff_time_set, Some(5)); // `:10344`
            assert_eq!(e.beetle_orbs, 0); // `:10368`
            assert!(e.delete_buff && e.rescan_slot); // `:10369-10370`
            assert!(!e.beetle_buff);
        }
    }

    #[test]
    fn w4_c16_beetle_endurance_with_the_set_keeps_the_tier() {
        let ctx = OrbEffectContext { beetle_defense: true, ..Default::default() };
        for (id, tier) in [(95, 1), (96, 2), (97, 3)] {
            let mut e = OrbEffects::default();
            apply_orb_effect_arm(&mut e, &ctx, id, 600);
            assert_eq!(e.beetle_orbs, tier); // `:10365`
            assert!(e.beetle_buff); // `:10374`
            assert!(!e.delete_buff);
        }
    }

    #[test]
    fn w4_c16_solar_shields_without_the_set_zero_and_delete() {
        for id in [170, 171, 172] {
            let e = apply(id); // `:10377-10407`, setSolar false
            assert_eq!(e.buff_time_set, Some(5)); // `:10379`
            assert_eq!(e.solar_shields, 0); // `:10403`
            assert!(e.delete_buff && e.rescan_slot); // `:10404-10405`
        }
    }

    #[test]
    fn w4_c16_solar_shields_with_the_set_keep_the_tier() {
        let ctx = OrbEffectContext { set_solar: true, ..Default::default() };
        for (id, tier) in [(170, 1), (171, 2), (172, 3)] {
            let mut e = OrbEffects::default();
            apply_orb_effect_arm(&mut e, &ctx, id, 600);
            assert_eq!(e.solar_shields, tier); // `:10400`
            assert!(!e.delete_buff);
        }
    }

    #[test]
    fn w4_c16_beetle_might_adds_stats_before_the_gate() {
        // beetleOffense false: the adds STILL land (`:10431-10432` precede
        // the gate), then the orbs zero + the slot deletes.
        let e = apply(100); // tier 3
        assert!((e.melee_damage - 0.3).abs() < 1e-6); // `:10431`
        assert!((e.melee_speed - 0.3).abs() < 1e-6); // `:10432`
        assert_eq!(e.beetle_orbs, 0); // `:10435`
        assert!(e.delete_buff && e.rescan_slot);
        assert_eq!(e.buff_time_set, None); // NO time pin (`:10408-10443`)
        // With the set: the orbs stay and beetleBuff arms.
        let ctx = OrbEffectContext { beetle_offense: true, ..Default::default() };
        let mut e = OrbEffects::default();
        apply_orb_effect_arm(&mut e, &ctx, 99, 600);
        assert_eq!(e.beetle_orbs, 2);
        assert!(e.beetle_buff); // `:10441`
        assert!((e.melee_damage - 0.2).abs() < 1e-6);
    }

    #[test]
    fn w4_c16_upgrade_purges_the_lower_band_inclusive() {
        // Existing tier 1, new tier 3: the m-loop band is base..=base+2 —
        // the CURRENT id included (the C# quirk, `:10357`).
        let ctx = OrbEffectContext { beetle_defense: true, ..Default::default() };
        let mut e = OrbEffects { beetle_orbs: 1, ..Default::default() };
        apply_orb_effect_arm(&mut e, &ctx, 97, 600);
        assert_eq!(e.purge_band, Some((95, 97))); // `:10355-10362`
        assert!(!e.delete_buff); // the self-delete is the OTHER branch
        assert_eq!(e.beetle_orbs, 3);
        // Solar's band (`:10392`).
        let ctx = OrbEffectContext { set_solar: true, ..Default::default() };
        let mut e = OrbEffects { solar_shields: 1, ..Default::default() };
        apply_orb_effect_arm(&mut e, &ctx, 172, 600);
        assert_eq!(e.purge_band, Some((170, 172)));
        // BeetleMight's band (`:10422`).
        let ctx = OrbEffectContext { beetle_offense: true, ..Default::default() };
        let mut e = OrbEffects { beetle_orbs: 1, ..Default::default() };
        apply_orb_effect_arm(&mut e, &ctx, 100, 600);
        assert_eq!(e.purge_band, Some((98, 100)));
    }

    #[test]
    fn w4_c16_downgrade_self_deletes_the_higher_slot() {
        // Existing tier 3, new tier 1: `beetleOrbs > num8` → DelBuff(j); j--
        // (`:10348-10352`), and the count STILL assigns (`:10365`).
        let ctx = OrbEffectContext { beetle_defense: true, ..Default::default() };
        let mut e = OrbEffects { beetle_orbs: 3, ..Default::default() };
        apply_orb_effect_arm(&mut e, &ctx, 95, 600);
        assert!(e.delete_buff && e.rescan_slot);
        assert_eq!(e.purge_band, None);
        assert_eq!(e.beetle_orbs, 1); // `:10365` runs after the branch
        // Solar (`:10383-10387`) and Might (`:10413-10417`) twins.
        let ctx = OrbEffectContext { set_solar: true, ..Default::default() };
        let mut e = OrbEffects { solar_shields: 3, ..Default::default() };
        apply_orb_effect_arm(&mut e, &ctx, 170, 600);
        assert!(e.delete_buff && e.rescan_slot);
        assert_eq!(e.solar_shields, 1);
    }

    #[test]
    fn w4_c16_equal_tier_skips_the_reconcile() {
        let ctx = OrbEffectContext { beetle_defense: true, ..Default::default() };
        let mut e = OrbEffects { beetle_orbs: 2, ..Default::default() };
        apply_orb_effect_arm(&mut e, &ctx, 96, 600); // `:10346` != gate
        assert!(!e.delete_buff);
        assert_eq!(e.purge_band, None);
        assert_eq!(e.beetle_orbs, 2);
    }

    #[test]
    fn w4_c16_nebula_mana_levels_assign() {
        for (id, level) in [(176, 1), (177, 2), (178, 3)] {
            let e = apply(id); // `:10444-10447`
            assert_eq!(e.nebula_level_mana, Some(level)); // `:12072`
            assert!(!e.buff_type_decrement);
        }
    }

    #[test]
    fn w4_c16_nebula_life_adds_six_per_level() {
        for (id, level) in [(173, 1), (174, 2), (175, 3)] {
            let e = apply(id); // `:10448-10452`
            assert_eq!(e.nebula_level_life, Some(level));
            assert_eq!(e.life_regen, 6 * level); // `:10451`
        }
    }

    #[test]
    fn w4_c16_nebula_damage_adds_fifteen_percent_per_level() {
        for (id, level) in [(179, 1), (180, 2), (181, 3)] {
            let e = apply(id); // `:10453-10461`
            assert_eq!(e.nebula_level_damage, Some(level));
            let want = 0.15 * level as f32; // `:10456`
            assert!((e.melee_damage - want).abs() < 1e-6);
            assert!((e.ranged_damage - want).abs() < 1e-6);
            assert!((e.magic_damage - want).abs() < 1e-6);
            assert!((e.minion_damage - want).abs() < 1e-6);
        }
    }

    #[test]
    fn w4_c16_nebula_downgrades_at_time_two() {
        // `UpdateBuffs_NebulaBuffs` `:12073-12078`: time == 2 && level > 1.
        let mut e = OrbEffects::default();
        apply_orb_effect_arm(&mut e, &OrbEffectContext::default(), 175, 2);
        assert_eq!(e.nebula_level_life, Some(2)); // 3 → 2
        assert!(e.buff_type_decrement); // `:12076`
        assert_eq!(e.buff_time_set, Some(480)); // `:12077`
        assert_eq!(e.life_regen, 12); // 6 * the POST-downgrade level
        // Level 1 never downgrades; time 3 never downgrades.
        let e = {
            let mut e = OrbEffects::default();
            apply_orb_effect_arm(&mut e, &OrbEffectContext::default(), 173, 2);
            e
        };
        assert!(!e.buff_type_decrement);
        assert_eq!(e.nebula_level_life, Some(1));
        let e = {
            let mut e = OrbEffects::default();
            apply_orb_effect_arm(&mut e, &OrbEffectContext::default(), 181, 3);
            e
        };
        assert!(!e.buff_type_decrement);
        assert_eq!(e.nebula_level_damage, Some(3));
    }

    #[test]
    fn w4_c16_stats_stack_across_slots() {
        let mut e = OrbEffects::default();
        let ctx = OrbEffectContext::default();
        apply_orb_effect_arm(&mut e, &ctx, 192, 600);
        apply_orb_effect_arm(&mut e, &ctx, 192, 600);
        assert!((e.move_speed - 0.4).abs() < 1e-6); // per-slot adds
        apply_orb_effect_arm(&mut e, &ctx, 321, 600);
        apply_orb_effect_arm(&mut e, &ctx, 321, 600);
        assert_eq!(e.melee_crit, 20);
        // The nebula LEVEL is an assignment — the last slot wins (`:12072`).
        apply_orb_effect_arm(&mut e, &ctx, 179, 600);
        apply_orb_effect_arm(&mut e, &ctx, 181, 600);
        assert_eq!(e.nebula_level_damage, Some(3));
        assert!((e.melee_damage - (0.15 + 0.45)).abs() < 1e-6); // adds stack
    }

    #[test]
    fn w4_c16_out_of_range_ids_are_not_handled() {
        let mut e = OrbEffects::default();
        let ctx = OrbEffectContext::default();
        // Wave A's potions, wave B's debuffs, the summon keep-alives and the
        // DoT family stay with their owners; 94 (Mana Sickness) is potions'.
        for id in [1, 2, 5, 14, 20, 46, 49, 94, 101, 116, 144, 169, 182, 0, 401] {
            assert!(!apply_orb_effect_arm(&mut e, &ctx, id, 600), "id {id}");
        }
        assert_eq!(e, OrbEffects::default());
    }
}
//! W4-g27 — the KEEP-ONE canonical module for the lunar-orb buff arms of
//! `Player.UpdateBuffs` (`Player.cs:10342-10461`): Beetle Endurance 95-97,
//! Solar Shield 170-172, Beetle Might 98-100 and the Nebula trio 173-181
//! (the `UpdateBuffs_NebulaBuffs` helper `:12070-12079` rides along).
//! 18 ids.
//!
//! Five branches implemented this block; the keep-one decision (the full
//! per-branch diff table + the merger contract live in
//! `docs/research/W4-g27-orb-keep-one.md`):
//! - **d73 `buff_effects_boosters.rs`** — MOST FAITHFUL arm core: direct
//!   `BuffSlots` mutation, the adjusted-`j` double-delete quirk, the Might
//!   stats BEFORE the offense gate, the shared sweep/unset helpers. Kept.
//! - **c17 `buff_effects_potions_sets.rs`** — the counter-seeding correction:
//!   `ResetEffects` zeroes `beetleOrbs`/`solarShields`/`nebulaLevel*` every
//!   frame (`Player.cs:18501-18502`/`:18505`/`:18755`), so the scan state
//!   starts at 0 each tick — d73's "persists across ticks" claim is WRONG.
//!   The zeroing is folded into `OrbScanState::reset_effects` below.
//! - **c16 `buff_effects_orbs.rs`** — the request vocabulary + the purge-band
//!   documentation (the walk deletes the CURRENT slot too and the outer `j`
//!   is NOT decremented afterwards — the shifted-in buff skips its arm this
//!   tick). Its `purge_band`/`delete_buff`/rescan requests are mirrored in
//!   `OrbArmRequests` for the accumulator-shaped mergers (the rescan rides
//!   `next_index` there).
//! - **d74 `buff_effects_tail.rs`** — the per-slot request shape (its
//!   `clear_buff_range`/`buff_time_set`/`buff_type_decrement`); same quirk
//!   gap as c16 (the context counters are caller-seeded, the delete tail
//!   uses the unadjusted `j`).
//! - **c18 `buff_effects_summons.rs`** — NO orb arms (it owns 62 + the
//!   summon keep-alives `:10462-10976`); listed for the census only.
//!
//! Scope: the 18 orb ids ONLY. Mana Heat 396 / Sugar Rush 192 / Brain of
//! Confusion 321 / Ice Barrier 62 / Mana Sickness 94 stay with their wave
//! modules (d73/c16/d74/c17/c18 own them); this module's dispatch returns
//! `NotHandled` for those ids so the caller's `else if` chain continues.
//!
//! Wiring contract (the f12/merger step, NOT this module):
//! 1. Run the timer tick FIRST (`buffs.rs::update_buffs_tick`, the
//!    `:9871-9874` decrement — none of the orb ids is in
//!    `TimeLeftDoesNotDecrease`, `BuffID.cs:47`).
//! 2. Call `OrbScanState::reset_effects()` once per frame (the C#
//!    `ResetEffects` writes), then walk the bar in slot order calling
//!    `apply_orb_arm` per active slot (`buffType > 0 && buffTime > 0`,
//!    `:9867-9870`), advancing by the returned `NextSlot` index (clamped
//!    `>= 0` — the -1 case is the guarded double-delete crash path, where
//!    the C# throws `DelBuff(-1)` before the loop ever advances).
//! 3. Fold `OrbEffects` onto the live fields AFTER the scan: stats ADD
//!    (`+=`), `beetle_buff` ORs; the counters/levels already live in
//!    `OrbScanState` (copy them onto the player fields — they are the C#
//!    fields, zeroed again next frame by step 2).
//!
//! RNG census: ZERO draws in `:10342-10461` + `:12070-12079` — arithmetic
//! and slot bookkeeping only; the block is stream-neutral by construction.

use crate::buffs::{BuffSlots, MAX_BUFFS};

/// The nebula tier window (`UpdateBuffs_NebulaBuffs`, `Player.cs:12077`): a
/// tier that hits `buffTime == 2` steps down and re-arms at 480 ticks.
pub const NEBULA_TIER_TIME: i32 = 480; // `:12077`
/// The beetle/solar orb pin (`Player.cs:10344`/`:10379`): `buffTime[j] = 5`
/// — the armor set re-grants the buff every tick while worn.
pub const ORB_BUFF_TIME: i32 = 5;

/// The per-tick context the arms read besides `buffType`/`buffTime`.
/// `whoAmI == Main.myPlayer` collapses to true in the single-player port,
/// matching every other ported `UpdateBuffs` arm.
#[derive(Clone, Copy, Debug, Default)]
pub struct OrbContext {
    /// `beetleDefense` (`Player.cs:10366`) — `ApplySetBonus_BeetleDefense`
    /// (`:16499-16501`, the Beetle Shell chestpiece; unported until the
    /// armor-set lane lands, so the endurance arms zero + delete exactly
    /// like the C# without the set).
    pub beetle_defense: bool,
    /// `beetleOffense` (`:10433`) — `ApplySetBonus_BeetleDamage`
    /// (`:16529-16532`, the Beetle Scale Mail chestpiece).
    pub beetle_offense: bool,
    /// `setSolar` (`:10401`) — `ApplySetBonus_Solar` (`:16366-16369`).
    pub set_solar: bool,
}

/// The scan's read-write counters — the C# `Player` fields `beetleOrbs` /
/// `solarShields` / `nebulaLevel*`. CORRECTION over d73: `ResetEffects`
/// zeroes ALL of them every frame (`Player.cs:18501` beetleOrbs, `:18505`
/// solarShields, `:18755` the nebula trio), so the scan starts at 0 each
/// tick; within the tick the arms read + assign them sequentially (the
/// wave-B `thorns` pattern). The caller keeps ONE of these per player and
/// calls `reset_effects()` once per frame before the scan.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OrbScanState {
    /// `beetleOrbs` (`:10365`/`:10430`, zeroed `:10368`/`:10435`) — shared
    /// by BOTH beetle families (Endurance and Might).
    pub beetle_orbs: i32,
    /// `solarShields` (`:10400`, zeroed `:10403`).
    pub solar_shields: i32,
    /// `nebulaLevelLife` (`:10450` via `:12072`).
    pub nebula_level_life: i32,
    /// `nebulaLevelMana` (`:10446` via `:12072`).
    pub nebula_level_mana: i32,
    /// `nebulaLevelDamage` (`:10455` via `:12072`).
    pub nebula_level_damage: i32,
}

impl OrbScanState {
    /// The `ResetEffects` writes (`Player.cs:18501`/`:18505`/`:18755`) —
    /// call once per frame BEFORE the buff scan. (`beetleBuff` `:18502`
    /// rides the accumulator's per-tick default instead.)
    pub fn reset_effects(&mut self) {
        *self = Self::default();
    }
}

/// One tick's accumulated stat/flag writes of the orb block. Stats ADD per
/// slot exactly like the C# chain; `beetle_buff` ORs. Slot mutations (the
/// time pins, the tier sweeps, the nebula step-down) run on `BuffSlots`
/// directly — they are NOT recorded here.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OrbEffects {
    /// `beetleBuff = true` (`:10374`/`:10441`) — the set-gated flag; reset
    /// every frame (`:18502`).
    pub beetle_buff: bool,
    /// 98-100 `meleeDamage += 0.1f * beetleOrbs` (`:10431`) + 179-181
    /// `+= 0.15f * nebulaLevelDamage` (`:10457`).
    pub melee_damage: f32,
    /// 98-100 `meleeSpeed += 0.1f * beetleOrbs` (`:10432`).
    pub melee_speed: f32,
    /// 179-181 (`:10458`).
    pub ranged_damage: f32,
    /// 179-181 (`:10459`).
    pub magic_damage: f32,
    /// 179-181 (`:10460`).
    pub minion_damage: f32,
    /// 173-175 `lifeRegen += 6 * nebulaLevelLife` (`:10451`).
    pub life_regen: i32,
}

/// The result of one arm: either the id is outside the orb block (the
/// caller's `else if` chain continues) or the caller's NEXT slot index —
/// the C# `j` after the arm plus the `for` loop's `j++` (`Player.cs:9865`).
/// A plain keep is `j + 1`; a `DelBuff(j); j--` rescan is `j`; the
/// double-delete quirk can be `j - 1`. The caller clamps to `>= 0` (the
/// -1 case is the guarded crash path — the C# `DelBuff(-1)` throws before
/// the loop ever advances).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrbArmResult {
    NotHandled,
    NextSlot(i32),
}

/// `Player.UpdateBuffs_NebulaBuffs` (`Player.cs:12070-12079`), verbatim:
/// `nebulaLevel = 1 + buffType[b] - baseBuffId`; at `buffTime[b] == 2` with
/// `level > 1` the tier steps down (`level--`, `buffType[b]--`,
/// `buffTime[b] = 480`) instead of expiring. The downgrade rewrites the
/// slot IN PLACE (no DelBuff), so the caller advances normally. Returns
/// the post-step level the caller's stat adds read.
fn nebula_buffs(slots: &mut BuffSlots, level: &mut i32, base_buff_id: i32, b: usize) -> i32 {
    *level = 1 + slots.buff_type[b] - base_buff_id; // `:12072`
    if slots.buff_time[b] == 2 && *level > 1 {
        // `:12073-12078`
        *level -= 1;
        slots.buff_type[b] -= 1; // `:12076`
        slots.buff_time[b] = NEBULA_TIER_TIME; // `:12077`
    }
    *level
}

/// The shared orb-tier sweep of the Beetle/Solar families
/// (`Player.cs:10346-10364` / `:10381-10399` / `:10411-10429`): when the
/// stored orb count differs from this slot's tier `num`, a DOWNGRADE
/// (`orbs > num`) deletes just this slot (`DelBuff(j); j--`), an UPGRADE
/// deletes every slot whose tier is `base..=base + num - 1` (the lower
/// tiers, INCLUDING this slot) with the `m--` rescan. The walk reads
/// `buffType` only (no `buffTime` gate). Returns the adjusted `j` (the
/// downgrade's `j--`; the upgrade leaves `j` alone — the outer loop's
/// `j++` then SKIPS the buff compacted into slot `j`, the C# quirk).
fn orb_tier_sweep(slots: &mut BuffSlots, j: usize, orbs: i32, num: i32, base: i32) -> i32 {
    let mut jj = j as i32;
    if orbs > 0 && orbs != num {
        if orbs > num {
            slots.del_buff(jj as usize); // `:10350`/`:10385`/`:10415`
            jj -= 1; // `:10351`/`:10386`/`:10416`
        } else {
            // `:10355-10362` / `:10390-10397` / `:10420-10427` — the
            // `for m … DelBuff(m); m--` rescan over the INCLUSIVE band.
            let mut m = 0usize;
            while m < MAX_BUFFS {
                let bt = slots.buff_type[m];
                if bt >= base && bt <= base + num - 1 {
                    slots.del_buff(m);
                    // m-- then the loop's m++ → stay on the shifted-in slot.
                } else {
                    m += 1;
                }
            }
        }
    }
    jj
}

/// The `!setFlag` tail shared by the three orb families
/// (`Player.cs:10366-10371` / `:10401-10406` / `:10433-10438`): zero the
/// orbs and `DelBuff(j); j--` — using the ALREADY-adjusted `j` from the
/// sweep. C# quirk preserved: after a downgrade delete `j` was decremented,
/// so this second delete hits the slot BEFORE the original. When that `j`
/// is negative the C# `DelBuff(-1)` throws `IndexOutOfRangeException`; the
/// port guards the delete (a documented deviation for that crash path —
/// unreachable in retail play: the counters reset to 0 every frame, so a
/// downgrade needs an earlier KEPT slot whose own set flag was on).
fn orb_unset_tail(slots: &mut BuffSlots, orbs: &mut i32, jj: i32) -> i32 {
    *orbs = 0; // `:10368`/`:10403`/`:10435`
    if jj >= 0 {
        slots.del_buff(jj as usize); // `:10369`/`:10404`/`:10436`
    }
    // The arm's `j--` cancels the loop's `j++` → the next index is `jj`.
    jj
}

/// Beetle Endurance, buffs 95-97 (`Player.cs:10342-10376`): pin
/// `buffTime = 5`, the tier sweep on the shared `beetleOrbs`, then the
/// `beetleDefense` gate (`beetleBuff = true` when set).
fn beetle_endurance_arm(
    e: &mut OrbEffects,
    s: &mut OrbScanState,
    slots: &mut BuffSlots,
    ctx: &OrbContext,
    j: usize,
) -> i32 {
    slots.buff_time[j] = ORB_BUFF_TIME; // `:10344`
    let num = 1 + slots.buff_type[j] - 95; // `:10345` num8 `(byte)(1 + …)`
    let jj = orb_tier_sweep(slots, j, s.beetle_orbs, num, 95);
    s.beetle_orbs = num; // `:10365`
    if !ctx.beetle_defense {
        orb_unset_tail(slots, &mut s.beetle_orbs, jj) // `:10366-10371`
    } else {
        e.beetle_buff = true; // `:10374`
        jj + 1
    }
}

/// Solar Shield, buffs 170-172 (`Player.cs:10377-10407`): the same shape
/// on `solarShields`/`setSolar`; NO `beetleBuff` flag (the C# arm has no
/// `else`), no stat adds — the shield count IS the state.
fn solar_shield_arm(
    s: &mut OrbScanState,
    slots: &mut BuffSlots,
    ctx: &OrbContext,
    j: usize,
) -> i32 {
    slots.buff_time[j] = ORB_BUFF_TIME; // `:10379`
    let num = 1 + slots.buff_type[j] - 170; // `:10380` num9
    let jj = orb_tier_sweep(slots, j, s.solar_shields, num, 170);
    s.solar_shields = num; // `:10400`
    if !ctx.set_solar {
        orb_unset_tail(slots, &mut s.solar_shields, jj) // `:10401-10406`
    } else {
        jj + 1
    }
}

/// Beetle Might, buffs 98-100 (`Player.cs:10408-10443`): NO time pin
/// (unlike Endurance); the tier sweep on the shared `beetleOrbs`, then
/// `meleeDamage`/`meleeSpeed += 0.1 * orbs` — read AFTER the assignment,
/// BEFORE the `beetleOffense` gate (the stats land even on the delete
/// tick), then the gate (`beetleBuff = true` when set).
fn beetle_might_arm(
    e: &mut OrbEffects,
    s: &mut OrbScanState,
    slots: &mut BuffSlots,
    ctx: &OrbContext,
    j: usize,
) -> i32 {
    let num = 1 + slots.buff_type[j] - 98; // `:10410` num10
    let jj = orb_tier_sweep(slots, j, s.beetle_orbs, num, 98);
    s.beetle_orbs = num; // `:10430`
    e.melee_damage += 0.1f32 * s.beetle_orbs as f32; // `:10431`
    e.melee_speed += 0.1f32 * s.beetle_orbs as f32; // `:10432`
    if !ctx.beetle_offense {
        orb_unset_tail(slots, &mut s.beetle_orbs, jj) // `:10433-10438`
    } else {
        e.beetle_buff = true; // `:10441`
        jj + 1
    }
}

/// One arm of the orb block. `j` is the live slot index; the arm reads
/// `buffType[j]`/`buffTime[j]` itself (the timer head already decremented).
/// Returns `NotHandled` for ids outside the 18-id orb set (the caller's
/// `else if` continues), else the next slot index. Stat/flag writes fold
/// into `e`, the counters into `s`, the slot mutations run on `slots`.
pub fn apply_orb_arm(
    e: &mut OrbEffects,
    s: &mut OrbScanState,
    slots: &mut BuffSlots,
    ctx: &OrbContext,
    j: usize,
) -> OrbArmResult {
    let buff_type = slots.buff_type[j];
    let next = match buff_type {
        95..=97 => beetle_endurance_arm(e, s, slots, ctx, j), // `:10342`
        170..=172 => solar_shield_arm(s, slots, ctx, j), // `:10377`
        98..=100 => beetle_might_arm(e, s, slots, ctx, j), // `:10408`
        176..=178 => {
            // `:10444-10447` — Nebula Mana: the level only (the mana-regen
            // consumer reads `nebulaLevelMana` in its own lane).
            nebula_buffs(slots, &mut s.nebula_level_mana, 176, j);
            j as i32 + 1
        }
        173..=175 => {
            // `:10448-10452` — Nebula Life: level + regen ×6 per level.
            let level = nebula_buffs(slots, &mut s.nebula_level_life, 173, j);
            e.life_regen += 6 * level; // `:10451`
            j as i32 + 1
        }
        179..=181 => {
            // `:10453-10461` — Nebula Damage: one num12 = 0.15 × level,
            // added to all four classes.
            let level = nebula_buffs(slots, &mut s.nebula_level_damage, 179, j);
            let num12 = 0.15f32 * level as f32; // `:10456`
            e.melee_damage += num12; // `:10457`
            e.ranged_damage += num12; // `:10458`
            e.magic_damage += num12; // `:10459`
            e.minion_damage += num12; // `:10460`
            j as i32 + 1
        }
        _ => return OrbArmResult::NotHandled,
    };
    OrbArmResult::NextSlot(next)
}

/// The per-slot request view of one arm — the c16/d74 accumulator shape,
/// derived from the SAME arm core (no second implementation): run the arm
/// on a one-slot scratch bar and diff the before/after. For mergers that
/// keep the request-drain wiring instead of the direct driver.
///
/// Quirk notes the drain MUST honour (c16's documentation, verified here):
/// - `purge_band` is INCLUSIVE of the current slot's id; after draining it
///   the outer `j` is NOT decremented (the shifted-in buff skips its arm).
/// - `delete_buff` uses the arm's ADJUSTED index: after a downgrade sweep
///   the gate-failure delete hits `j - 1` (the slot before the original);
///   `next_index` already encodes the rescan — prefer `next_index` over
///   replaying `delete_buff` at the original `j`.
/// - `buff_time_set`/`buff_type_decrement` are the nebula step-down's
///   in-place writes (no delete, advance normally).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OrbArmRequests {
    /// `buffTime[j] = …` — the 5-pin (`:10344`/`:10379`) or the nebula
    /// step's 480 (`:12077`).
    pub buff_time_set: Option<i32>,
    /// The nebula step's `buffType[b]--` (`:12076`).
    pub buff_type_decrement: bool,
    /// A `DelBuff` ran (the sweep's self-delete or the gate tail).
    pub delete_buff: bool,
    /// The upgrade sweep's whole-bar band (`:10357`/`:10392`/`:10422`).
    pub purge_band: Option<(i32, i32)>,
    /// The caller's next slot index (the authoritative quirk encoding).
    pub next_index: i32,
}

/// Derive the request view for slot `j` WITHOUT touching the caller's bar:
/// the arm runs on a scratch copy holding only that slot, seeded with the
/// same state/context. `s`/`e` receive the same writes the direct arm
/// would make. (The scratch diff cannot see cross-slot sweep deletes —
/// those are the `purge_band` request's job; the direct driver is the
/// faithful path for the live bar.)
pub fn orb_arm_requests(
    e: &mut OrbEffects,
    s: &mut OrbScanState,
    slots: &BuffSlots,
    ctx: &OrbContext,
    j: usize,
) -> Option<OrbArmRequests> {
    let bt = slots.buff_type[j];
    let time = slots.buff_time[j];
    let mut scratch = BuffSlots::default();
    scratch.buff_type[0] = bt;
    scratch.buff_time[0] = time;
    let before_orbs = (s.beetle_orbs, s.solar_shields);
    let next = match apply_orb_arm(e, s, &mut scratch, ctx, 0) {
        OrbArmResult::NextSlot(n) => n,
        OrbArmResult::NotHandled => return None,
    };
    let mut req = OrbArmRequests {
        // The scratch slot sits at 0; the arm's next index is an offset
        // from it (keep +1, rescan 0, double-delete -1) — rebase onto j.
        next_index: next + j as i32,
        ..Default::default()
    };
    // The pin runs BEFORE the sweep (`:10344`/`:10379`), so it is recorded
    // even when the sweep/gate later deletes the scratch slot (a time diff
    // would read the post-delete 0 instead of the 5).
    if (95..=97).contains(&bt) || (170..=172).contains(&bt) {
        req.buff_time_set = Some(ORB_BUFF_TIME);
    }
    if scratch.buff_type[0] == bt - 1 && (173..=181).contains(&bt) {
        req.buff_type_decrement = true; // the nebula step-down
        req.buff_time_set = Some(NEBULA_TIER_TIME); // `:12077`
    }
    // The upgrade band fires when the sweep took the purge branch: the
    // stored count was lower than this tier (and non-zero).
    let (orbs, base) = match bt {
        95..=97 => (before_orbs.0, 95),
        98..=100 => (before_orbs.0, 98),
        170..=172 => (before_orbs.1, 170),
        _ => (0, 0),
    };
    if orbs > 0 {
        let tier = 1 + bt - base;
        if orbs < tier {
            req.purge_band = Some((base, base + tier - 1));
        }
    }
    if scratch.buff_type[0] == 0 && req.purge_band.is_none() {
        // The scratch slot died via the arm's OWN DelBuff (the downgrade
        // self-delete or the gate tail) — not via the purge band (whose
        // self-inclusion is the band's quirk, recorded separately).
        req.delete_buff = true;
    }
    Some(req)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mid-scan slot state, set directly (the C# chain reads the arrays
    /// after the timer head's decrement).
    fn slots(rows: &[(i32, i32)]) -> BuffSlots {
        let mut s = BuffSlots::default();
        for (k, &(bt, time)) in rows.iter().enumerate() {
            s.buff_type[k] = bt;
            s.buff_time[k] = time;
        }
        s
    }

    fn run(
        s: &mut OrbScanState,
        sl: &mut BuffSlots,
        ctx: &OrbContext,
        j: usize,
    ) -> (OrbEffects, OrbArmResult) {
        let mut e = OrbEffects::default();
        let r = apply_orb_arm(&mut e, s, sl, ctx, j);
        (e, r)
    }

    #[test]
    fn w4_g27_reset_effects_zeroes_the_counters_every_frame() {
        // The c17 correction over d73: `Player.cs:18501`/`:18505`/`:18755`
        // — ResetEffects zeroes beetleOrbs/solarShields/the nebula trio.
        let mut s = OrbScanState {
            beetle_orbs: 3,
            solar_shields: 2,
            nebula_level_life: 3,
            nebula_level_mana: 2,
            nebula_level_damage: 1,
        };
        s.reset_effects();
        assert_eq!(s, OrbScanState::default());
    }

    #[test]
    fn w4_g27_95_beetle_endurance_pins_five_and_sets_the_flag() {
        // `:10344-10374`: tier 1, no stored orbs, defense worn.
        let ctx = OrbContext { beetle_defense: true, ..Default::default() };
        let mut sl = slots(&[(95, 2)]);
        let mut st = OrbScanState::default();
        let (e, r) = run(&mut st, &mut sl, &ctx, 0);
        assert_eq!(sl.buff_time[0], 5, "the orb pin (:10344)");
        assert_eq!(st.beetle_orbs, 1);
        assert!(e.beetle_buff);
        assert_eq!(r, OrbArmResult::NextSlot(1));
    }

    #[test]
    fn w4_g27_95_97_endurance_tiers_with_the_set() {
        // c16's per-tier witness: each id keeps its tier with defense on.
        let ctx = OrbContext { beetle_defense: true, ..Default::default() };
        for (id, tier) in [(95, 1), (96, 2), (97, 3)] {
            let mut sl = slots(&[(id, 2)]);
            let mut st = OrbScanState::default();
            let (e, _) = run(&mut st, &mut sl, &ctx, 0);
            assert_eq!(st.beetle_orbs, tier);
            assert_eq!(sl.buff_time[0], 5);
            assert!(e.beetle_buff);
        }
    }

    #[test]
    fn w4_g27_97_upgrade_sweep_deletes_the_lower_tiers_including_self() {
        // `:10355-10362`: stored orbs 1, this slot tier 3 (97) -> every
        // slot in 95..=97 is DelBuff'd (the m-- rescan), orbs = 3. The
        // outer j is NOT decremented: the shifted-in Regen skips its arm.
        let ctx = OrbContext { beetle_defense: true, ..Default::default() };
        let mut sl = slots(&[(95, 5), (96, 5), (97, 5), (2, 100)]);
        let mut st = OrbScanState { beetle_orbs: 1, ..Default::default() };
        let (e, r) = run(&mut st, &mut sl, &ctx, 2);
        assert_eq!(st.beetle_orbs, 3);
        assert!(e.beetle_buff);
        assert_eq!(sl.buff_type[0], 2, "Regen compacted to slot 0");
        assert_eq!(sl.buff_type[1], 0);
        assert_eq!(r, OrbArmResult::NextSlot(3), "upgrade leaves j alone");
    }

    #[test]
    fn w4_g27_95_downgrade_deletes_self_and_rescans() {
        // `:10348-10352`: stored orbs 3, this slot tier 1 -> DelBuff(j);
        // j-- (the next index stays j), orbs = 1, flag still set.
        let ctx = OrbContext { beetle_defense: true, ..Default::default() };
        let mut sl = slots(&[(2, 100), (95, 5), (3, 100)]);
        let mut st = OrbScanState { beetle_orbs: 3, ..Default::default() };
        let (e, r) = run(&mut st, &mut sl, &ctx, 1);
        assert_eq!(st.beetle_orbs, 1);
        assert!(e.beetle_buff);
        assert_eq!(sl.buff_type[1], 3, "Swiftness compacted into slot 1");
        assert_eq!(r, OrbArmResult::NextSlot(1), "the j-- rescan");
    }

    #[test]
    fn w4_g27_95_defense_off_zeroes_orbs_and_deletes() {
        // `:10366-10371`: no set -> orbs = 0, DelBuff(j); j--, no flag.
        let mut sl = slots(&[(96, 5), (2, 100)]);
        let mut st = OrbScanState::default();
        let (e, r) = run(&mut st, &mut sl, &OrbContext::default(), 0);
        assert_eq!(st.beetle_orbs, 0);
        assert!(!e.beetle_buff);
        assert_eq!(sl.buff_type[0], 2);
        assert_eq!(r, OrbArmResult::NextSlot(0));
    }

    #[test]
    fn w4_g27_95_double_delete_quirk_hits_the_previous_slot() {
        // The d73 quirk (c17 collapses it to one rescan — WRONG): downgrade
        // at j=1 (j-- -> 0) THEN !beetleDefense deletes the ALREADY-
        // decremented j (slot 0). Both slots die.
        let mut sl = slots(&[(2, 100), (95, 5), (3, 100)]);
        let mut st = OrbScanState { beetle_orbs: 3, ..Default::default() };
        let (_, r) = run(&mut st, &mut sl, &OrbContext::default(), 1);
        assert_eq!(st.beetle_orbs, 0);
        assert_eq!(sl.buff_type[0], 3, "slots 0 AND 1 deleted, 3 compacts down");
        assert_eq!(sl.buff_type[1], 0);
        assert_eq!(r, OrbArmResult::NextSlot(0));
    }

    #[test]
    fn w4_g27_95_double_delete_at_slot_zero_guards_the_negative_index() {
        // Same quirk at j=0: the second delete's index is -1 — the C#
        // throws; the port guards (documented deviation). No panic.
        let mut sl = slots(&[(95, 5), (2, 100)]);
        let mut st = OrbScanState { beetle_orbs: 2, ..Default::default() };
        let (_, r) = run(&mut st, &mut sl, &OrbContext::default(), 0);
        assert_eq!(st.beetle_orbs, 0);
        assert_eq!(sl.buff_type[0], 2, "only the first delete ran");
        assert_eq!(r, OrbArmResult::NextSlot(-1), "the caller clamps");
    }

    #[test]
    fn w4_g27_98_beetle_might_no_pin_stats_before_the_gate() {
        // `:10408-10442`: NO buffTime pin; tier 2 -> +0.2 melee damage and
        // speed; offense worn -> flag.
        let ctx = OrbContext { beetle_offense: true, ..Default::default() };
        let mut sl = slots(&[(99, 3)]);
        let mut st = OrbScanState::default();
        let (e, r) = run(&mut st, &mut sl, &ctx, 0);
        assert_eq!(sl.buff_time[0], 3, "no time pin in the Might arm");
        assert_eq!(st.beetle_orbs, 2);
        assert!((e.melee_damage - 0.2).abs() < 1e-6);
        assert!((e.melee_speed - 0.2).abs() < 1e-6);
        assert!(e.beetle_buff);
        assert_eq!(r, OrbArmResult::NextSlot(1));
    }

    #[test]
    fn w4_g27_98_might_offense_off_still_adds_the_stats_then_deletes() {
        // `:10431-10432` run BEFORE the `:10433` gate — the stats land on
        // the delete tick too (c16/d73/d74 agree; pinned here).
        let mut sl = slots(&[(100, 3)]);
        let mut st = OrbScanState::default();
        let (e, r) = run(&mut st, &mut sl, &OrbContext::default(), 0);
        assert!((e.melee_damage - 0.3).abs() < 1e-6, "0.1 * 3 before the gate");
        assert_eq!(st.beetle_orbs, 0);
        assert!(!e.beetle_buff);
        assert_eq!(sl.buff_type[0], 0);
        assert_eq!(r, OrbArmResult::NextSlot(0));
    }

    #[test]
    fn w4_g27_98_might_sweep_shares_the_beetle_orbs_counter() {
        // The Might family sweeps on the SAME beetleOrbs the Endurance
        // family writes (`:10411` reads what `:10365` left): stored 1 from
        // an Endurance slot, Might tier 3 upgrades -> band 98..=100.
        let ctx = OrbContext { beetle_offense: true, ..Default::default() };
        let mut sl = slots(&[(98, 5), (100, 5), (2, 100)]);
        let mut st = OrbScanState { beetle_orbs: 1, ..Default::default() };
        let (_, r) = run(&mut st, &mut sl, &ctx, 1);
        assert_eq!(st.beetle_orbs, 3);
        assert_eq!(sl.buff_type[0], 2, "98 and 100 purged, Regen compacted");
        assert_eq!(r, OrbArmResult::NextSlot(2));
    }

    #[test]
    fn w4_g27_170_solar_shield_pins_and_gates_on_set_solar() {
        // `:10377-10406`: tier 3 pin 5, shields = 3, set worn -> keep; no
        // beetle_buff flag, no stat adds.
        let ctx = OrbContext { set_solar: true, ..Default::default() };
        let mut sl = slots(&[(172, 2)]);
        let mut st = OrbScanState::default();
        let (e, r) = run(&mut st, &mut sl, &ctx, 0);
        assert_eq!(sl.buff_time[0], 5);
        assert_eq!(st.solar_shields, 3);
        assert!(!e.beetle_buff, "the Solar arm sets no beetleBuff");
        assert_eq!(e.melee_damage, 0.0);
        assert_eq!(r, OrbArmResult::NextSlot(1));
    }

    #[test]
    fn w4_g27_170_solar_set_absent_equal_tier_deletes_at_j() {
        // d73's re-derived witness: set absent with the EQUAL tier (no
        // sweep, no j--) -> zero + delete at j itself (a downgrade here
        // would j-- to -1; the equal-tier case is the retail shape).
        let mut sl = slots(&[(170, 5)]);
        let mut st = OrbScanState { solar_shields: 1, ..Default::default() };
        let (_, r) = run(&mut st, &mut sl, &OrbContext::default(), 0);
        assert_eq!(st.solar_shields, 0);
        assert_eq!(sl.buff_type[0], 0);
        assert_eq!(r, OrbArmResult::NextSlot(0));
    }

    #[test]
    fn w4_g27_170_solar_upgrade_band_is_inclusive() {
        // c16's band witness on the Solar family (`:10392`).
        let ctx = OrbContext { set_solar: true, ..Default::default() };
        let mut sl = slots(&[(170, 5), (172, 5), (2, 100)]);
        let mut st = OrbScanState { solar_shields: 1, ..Default::default() };
        let (_, r) = run(&mut st, &mut sl, &ctx, 1);
        assert_eq!(st.solar_shields, 3);
        assert_eq!(sl.buff_type[0], 2);
        assert_eq!(r, OrbArmResult::NextSlot(2));
    }

    #[test]
    fn w4_g27_173_nebula_life_tier_steps_down_at_time_two() {
        // `:12072-12078` via `:10448-10452`: buff 174 (tier 2) at time 2
        // -> level 1, buffType 173, time 480; lifeRegen += 6 * 1.
        let mut sl = slots(&[(174, 2)]);
        let mut st = OrbScanState::default();
        let (e, r) = run(&mut st, &mut sl, &OrbContext::default(), 0);
        assert_eq!(st.nebula_level_life, 1);
        assert_eq!(sl.buff_type[0], 173, "the tier step-down");
        assert_eq!(sl.buff_time[0], 480);
        assert_eq!(e.life_regen, 6);
        assert_eq!(r, OrbArmResult::NextSlot(1));
    }

    #[test]
    fn w4_g27_175_nebula_life_tier_three_regen_eighteen() {
        // No step-down above time 2: level 3 -> lifeRegen += 18.
        let mut sl = slots(&[(175, 100)]);
        let mut st = OrbScanState::default();
        let (e, _) = run(&mut st, &mut sl, &OrbContext::default(), 0);
        assert_eq!(st.nebula_level_life, 3);
        assert_eq!(sl.buff_time[0], 100, "no re-arm above time 2");
        assert_eq!(e.life_regen, 18);
    }

    #[test]
    fn w4_g27_173_nebula_tier_one_at_time_two_does_not_step() {
        // The `level > 1` guard (`:12073`): tier 1 at time 2 stays put.
        let mut sl = slots(&[(173, 2)]);
        let mut st = OrbScanState::default();
        let (e, _) = run(&mut st, &mut sl, &OrbContext::default(), 0);
        assert_eq!(st.nebula_level_life, 1);
        assert_eq!(sl.buff_type[0], 173);
        assert_eq!(sl.buff_time[0], 2, "no re-arm at tier 1");
        assert_eq!(e.life_regen, 6);
    }

    #[test]
    fn w4_g27_176_nebula_mana_sets_the_level_only() {
        // `:10444-10447`: the mana tier writes nebulaLevelMana, no stats.
        let mut sl = slots(&[(177, 2)]);
        let mut st = OrbScanState::default();
        let (e, _) = run(&mut st, &mut sl, &OrbContext::default(), 0);
        assert_eq!(st.nebula_level_mana, 1, "stepped down from tier 2");
        assert_eq!(sl.buff_type[0], 176);
        assert_eq!(sl.buff_time[0], 480);
        assert_eq!(e.life_regen, 0);
        assert_eq!(e.melee_damage, 0.0);
    }

    #[test]
    fn w4_g27_179_nebula_damage_fifteen_percent_per_level() {
        // `:10453-10461`: tier 3 -> 0.15 * 3 = 0.45 on all four classes.
        let mut sl = slots(&[(181, 50)]);
        let mut st = OrbScanState::default();
        let (e, _) = run(&mut st, &mut sl, &OrbContext::default(), 0);
        assert_eq!(st.nebula_level_damage, 3);
        for v in [e.melee_damage, e.ranged_damage, e.magic_damage, e.minion_damage] {
            assert!((v - 0.45).abs() < 1e-5, "0.15 * 3, got {v}");
        }
    }

    #[test]
    fn w4_g27_nebula_levels_are_last_slot_wins_within_the_tick() {
        // The C# `ref` reassigns per slot: two Life slots -> the LAST
        // slot's level wins, but the regen adds STACK (`:10451` per slot).
        let mut sl = slots(&[(173, 100), (175, 100)]);
        let mut st = OrbScanState::default();
        let mut e = OrbEffects::default();
        assert_eq!(apply_orb_arm(&mut e, &mut st, &mut sl, &OrbContext::default(), 0),
            OrbArmResult::NextSlot(1));
        assert_eq!(apply_orb_arm(&mut e, &mut st, &mut sl, &OrbContext::default(), 1),
            OrbArmResult::NextSlot(2));
        assert_eq!(st.nebula_level_life, 3, "last slot wins");
        assert_eq!(e.life_regen, 6 + 18, "the adds stack");
    }

    #[test]
    fn w4_g27_scan_state_starts_zero_each_tick() {
        // The c17 correction end-to-end: a kept tier-3 Endurance slot on
        // tick N leaves beetle_orbs = 3; reset_effects() before tick N+1
        // means the same slot re-arms from 0 (no phantom downgrade).
        let ctx = OrbContext { beetle_defense: true, ..Default::default() };
        let mut st = OrbScanState::default();
        for _ in 0..3 {
            st.reset_effects();
            let mut sl = slots(&[(97, 5)]);
            let (e, r) = run(&mut st, &mut sl, &ctx, 0);
            assert_eq!(st.beetle_orbs, 3);
            assert!(e.beetle_buff);
            assert_eq!(sl.buff_type[0], 97, "never self-deletes across ticks");
            assert_eq!(r, OrbArmResult::NextSlot(1));
        }
    }

    #[test]
    fn w4_g27_requests_view_matches_the_direct_arm() {
        // The c16/d74 request vocabulary derived from the same core: the
        // pin, the band, the nebula step, the gate delete.
        let ctx = OrbContext { beetle_defense: true, ..Default::default() };
        let sl = slots(&[(97, 2)]);
        let mut st = OrbScanState { beetle_orbs: 1, ..Default::default() };
        let mut e = OrbEffects::default();
        let req = orb_arm_requests(&mut e, &mut st, &sl, &ctx, 0).unwrap();
        assert_eq!(req.buff_time_set, Some(5));
        assert_eq!(req.purge_band, Some((95, 97)));
        assert!(!req.delete_buff);
        assert_eq!(req.next_index, 1, "upgrade leaves j alone");
        assert_eq!(st.beetle_orbs, 3);
        // The nebula step-down rides the same view.
        let sl2 = slots(&[(174, 2)]);
        let mut st2 = OrbScanState::default();
        let mut e2 = OrbEffects::default();
        let req2 = orb_arm_requests(&mut e2, &mut st2, &sl2, &OrbContext::default(), 0).unwrap();
        assert_eq!(req2.buff_time_set, Some(480));
        assert!(req2.buff_type_decrement);
        assert!(!req2.delete_buff);
        assert_eq!(req2.next_index, 1);
        // The gate failure: delete + rescan, no band.
        let sl3 = slots(&[(96, 5)]);
        let mut st3 = OrbScanState::default();
        let mut e3 = OrbEffects::default();
        let req3 = orb_arm_requests(&mut e3, &mut st3, &sl3, &OrbContext::default(), 0).unwrap();
        assert!(req3.delete_buff);
        assert_eq!(req3.purge_band, None);
        assert_eq!(req3.next_index, 0);
        // Out-of-block ids derive nothing.
        let sl4 = slots(&[(396, 60)]);
        assert!(orb_arm_requests(&mut e3, &mut st3, &sl4, &OrbContext::default(), 0).is_none());
    }

    #[test]
    fn w4_g27_out_of_block_ids_pass_through() {
        // 396/192/321/62/94 belong to the other wave modules; 2/144 are
        // other waves entirely — the chain continues, the slot untouched.
        for bt in [396, 192, 321, 62, 94, 2, 144] {
            let mut sl = slots(&[(bt, 100)]);
            let (_, r) = run(
                &mut OrbScanState::default(),
                &mut sl,
                &OrbContext::default(),
                0,
            );
            assert_eq!(r, OrbArmResult::NotHandled, "buff {bt}");
            assert_eq!(sl.buff_type[0], bt, "the slot is untouched");
        }
    }
}
