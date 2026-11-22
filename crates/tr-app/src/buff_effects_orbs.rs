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
