//! S1-E-7 wave C second block (W4-c17) — the MANA-POTION / ARMOR-SET tier arms
//! of `Player.UpdateBuffs` (`Player.cs:10318-10461`): 94 Mana Sickness,
//! 396 Mana Heat, 95-97 Beetle Endurance, 170-172 Solar Shield, 98-100 Beetle
//! Might, 176-178 Nebula Mana, 173-175 Nebula Life, 179-181 Nebula Damage
//! (20 ids; the Nebula downgrade helper `UpdateBuffs_NebulaBuffs`
//! `Player.cs:12070-12079` rides along).
//!
//! Disjoint-wave design (c16 owns `:9914-10048`, c18 `:10486-10972`; the
//! claim + census live in `docs/research/28c-wave-c-claim.md`): the simple
//! arms (94, 396) fold into a `PotionSetEffects` accumulator via
//! `apply_potion_set_effect_arm`, the wave-B `buff_effects.rs` seam pattern.
//! The set-tier arms read AND write the whole buff bar (their lower-tier
//! `DelBuff` walks scan every slot, the Nebula downgrade rewrites
//! `buffType[b]`/`buffTime[b]`), so they ride the slot-ordered
//! `update_potion_set_buff_arms` driver over `&mut BuffSlots` + the
//! accumulator, mirroring the C# `for j` loop with the `DelBuff(j); j--`
//! rescan (`Player.cs:9865`, the `update_summon_buff_arms` convention).
//! Wiring into the live tick is the merge step, NOT this wave: the caller
//! copies `latest_mana_potion_*` into the context, runs the driver after the
//! timer tick (`buffs.rs::update_buffs_tick` — the driver does NOT decrement
//! again), then folds the accumulator onto the live fields (`mana_heat +=`,
//! the stat adds `+=`, the level/orb/shield fields `=` — `ResetEffects`
//! zeroes them every frame, `Player.cs:18501-18505`/`:18755`).
//!
//! Buff 94's live half already runs through `potions::apply_mana_sickness_arm`
//! (`player.rs:6501`); this table MIRRORS it for the merged scan and must not
//! be wired twice. The `DebugOptions.ManaPotionDelay` mirror (`:10320-10323`)
//! is constants-off (false retail static), same as the potions module.
//!
//! RNG census of `:10318-10461` + `:12070-12079`: ZERO draws. No `Main.rand`
//! / `rand.Next` in any arm; the Nebula downgrade is deterministic timer
//! arithmetic (`buffTime == 2 && level > 1` → level−1, buff id−1, time 480).

use crate::potions::MANA_SICK_TIME;

/// The per-tick context the simple arms read besides `buffType`/`buffTime`:
/// the `TryApplyManaHeat` pair (`Player.cs:52006-52014`, written by the
/// mana-star pickup lane `:35071`/`:35087`, live in
/// `player.rs::try_apply_mana_heat`).
#[derive(Clone, Copy, Debug, Default)]
pub struct PotionEffectContext {
    /// `latestManaPotionDamageBonus` (`Player.cs:982`).
    pub latest_mana_potion_damage_bonus: f32,
    /// `latestManaPotionDuration` (`Player.cs:984`) — the 396 arm floors a
    /// zero to 1 IN THE PLAYER FIELD (`:10331-10334`); the accumulator
    /// records the write as `mana_potion_duration_floored`.
    pub latest_mana_potion_duration: f32,
}

/// The armor-set flags the tier arms gate on (`UpdateArmorSets` /
/// `UpdateEquips` write them; `ResetEffects` clears, `Player.cs:25618-25620`).
#[derive(Clone, Copy, Debug, Default)]
pub struct SetBonusFlags {
    /// `beetleDefense` — the Beetle Scale Mail set (`Player.cs:16501`).
    pub beetle_defense: bool,
    /// `beetleOffense` — the Beetle Shell set (`Player.cs:16532`).
    pub beetle_offense: bool,
    /// `setSolar` — the Solar Flare set (`Player.cs:16369`).
    pub set_solar: bool,
}

/// One tick's accumulated writes of the block. Stat fields are ADDED per
/// slot exactly like the C# chain; the level/orb/shield fields are sequential
/// read-write state (the C# `Player` fields, zeroed by `ResetEffects` every
/// frame — `beetleOrbs`/`beetleBuff` `:18501-18502`, `solarShields` `:18505`,
/// the nebula levels `:18755`), so the accumulator starts at 0 each tick and
/// the wiring copies the finals onto the live fields.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PotionSetEffects {
    // 94 Mana Sickness (`Player.cs:10318-10327`) — overwrite, LAST live slot
    // wins (the `potions::apply_mana_sickness_arm` mirror).
    pub mana_sick: bool,
    pub mana_sick_reduction: f32,
    // 396 Mana Heat (`:10328-10341`) — `manaHeat += num6` per slot ON TOP of
    // the reset 1.0 (`:18593`); the wiring ADDS this to the live field.
    pub mana_heat: f32,
    /// The `latestManaPotionDuration == 0f → 1f` write (`:10331-10334`).
    pub mana_potion_duration_floored: bool,
    // The set-tier state (`:10342-10461`).
    pub beetle_orbs: i32,   // `beetleOrbs` (`:10365`/`:10430`)
    pub beetle_buff: bool,  // `beetleBuff` (`:10374`/`:10441`)
    pub solar_shields: i32, // `solarShields` (`:10400`)
    pub nebula_level_mana: i32,   // `nebulaLevelMana` (`:10446`)
    pub nebula_level_life: i32,   // `nebulaLevelLife` (`:10450`)
    pub nebula_level_damage: i32, // `nebulaLevelDamage` (`:10455`)
    // Stat adds (per-slot, summed like the C# chain).
    pub melee_damage: f32,  // 98-100 `:10431`, 179-181 `:10457`
    pub melee_speed: f32,   // 98-100 `:10432` (live `melee_speed_bonus`)
    pub ranged_damage: f32, // 179-181 `:10458`
    pub magic_damage: f32,  // 179-181 `:10459`
    pub minion_damage: f32, // 179-181 `:10460`
    pub life_regen: i32,    // 173-175 `:10451` `lifeRegen += 6 * level`
}

/// The simple arms of the block (`Player.cs:10318-10341`): 94 and 396.
/// Returns `true` when the id is handled, `false` otherwise — the wave-B
/// dispatch shape. The set-tier range heads ride the slot-ordered driver
/// below (they touch `BuffSlots`).
pub fn apply_potion_set_effect_arm(
    e: &mut PotionSetEffects,
    buff_type: i32,
    buff_time: i32,
    ctx: &PotionEffectContext,
) -> bool {
    match buff_type {
        94 => {
            // `:10318-10327` — the DebugOptions.ManaPotionDelay mirror is
            // constants-off; the reduction overwrites (last slot wins).
            e.mana_sick = true; // `:10325`
            e.mana_sick_reduction =
                0.25f32 * (buff_time as f32 / MANA_SICK_TIME as f32); // `:10324-10326`
        }
        396 => {
            // `:10328-10341` — Mana Heat: bonus × remaining-time fraction,
            // added per slot. The zero-duration floor writes the PLAYER
            // field; the accumulator records it for the wiring.
            let mut num6 = ctx.latest_mana_potion_damage_bonus; // `:10330`
            let duration = if ctx.latest_mana_potion_duration == 0.0 {
                e.mana_potion_duration_floored = true; // `:10331-10334`
                1.0
            } else {
                ctx.latest_mana_potion_duration
            };
            let num7 = buff_time as f32 / duration; // `:10335`
            num6 *= num7; // `:10336-10339` — the `0f == 0f` branch, always taken
            e.mana_heat += num6; // `:10340`
        }
        _ => return false,
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s1_e_7c17_buff_94_mirrors_the_live_sickness_arm() {
        let mut e = PotionSetEffects::default();
        let ctx = PotionEffectContext::default();
        assert!(apply_potion_set_effect_arm(&mut e, 94, 150, &ctx));
        assert!(e.mana_sick); // `Player.cs:10325`
        // `:10324-10326`: 0.25 * 150/300 = 0.125.
        assert_eq!(e.mana_sick_reduction, 0.125);
        // The live twin agrees (`potions::apply_mana_sickness_arm`).
        let w = crate::test_world_helpers::ground();
        let mut p = crate::player::Player::spawn(&w);
        crate::potions::apply_mana_sickness_arm(&mut p, 150);
        assert_eq!(p.mana_sick_reduction, e.mana_sick_reduction);
    }

    #[test]
    fn s1_e_7c17_buff_94_last_slot_wins() {
        let mut e = PotionSetEffects::default();
        let ctx = PotionEffectContext::default();
        apply_potion_set_effect_arm(&mut e, 94, 300, &ctx);
        apply_potion_set_effect_arm(&mut e, 94, 60, &ctx);
        // Overwrite, NOT add — the C# `=` at `:10326`.
        assert_eq!(e.mana_sick_reduction, 0.25 * 60.0 / 300.0);
    }

    #[test]
    fn s1_e_7c17_buff_396_scales_the_bonus_by_remaining_time() {
        let mut e = PotionSetEffects::default();
        // The star pickup's pair: power 0.2, time 2s → duration 120
        // (`TryApplyManaHeat`, `Player.cs:52008-52009`).
        let ctx = PotionEffectContext {
            latest_mana_potion_damage_bonus: 0.2,
            latest_mana_potion_duration: 120.0,
        };
        assert!(apply_potion_set_effect_arm(&mut e, 396, 60, &ctx));
        // `:10335-10340`: 0.2 * 60/120 = 0.1.
        assert!((e.mana_heat - 0.1).abs() < 1e-6);
        assert!(!e.mana_potion_duration_floored);
    }

    #[test]
    fn s1_e_7c17_buff_396_floors_a_zero_duration_and_records_the_write() {
        let mut e = PotionSetEffects::default();
        let ctx = PotionEffectContext {
            latest_mana_potion_damage_bonus: 0.2,
            latest_mana_potion_duration: 0.0,
        };
        apply_potion_set_effect_arm(&mut e, 396, 60, &ctx);
        // `:10331-10334`: duration 0 → 1, so the fraction is 60/1.
        assert!(e.mana_potion_duration_floored);
        assert!((e.mana_heat - 12.0).abs() < 1e-4);
    }

    #[test]
    fn s1_e_7c17_buff_396_stacks_across_slots() {
        let mut e = PotionSetEffects::default();
        let ctx = PotionEffectContext {
            latest_mana_potion_damage_bonus: 0.2,
            latest_mana_potion_duration: 120.0,
        };
        apply_potion_set_effect_arm(&mut e, 396, 60, &ctx);
        apply_potion_set_effect_arm(&mut e, 396, 120, &ctx);
        // `manaHeat += num6` per slot (`:10340`): 0.1 + 0.2.
        assert!((e.mana_heat - 0.3).abs() < 1e-6);
    }

    #[test]
    fn s1_e_7c17_simple_arm_declines_foreign_ids() {
        let mut e = PotionSetEffects::default();
        let ctx = PotionEffectContext::default();
        // The set-tier ids ride the driver, not this hook; neighbours stay
        // with their owners (93 ammo box = wave B, 62 ice barrier = live).
        for id in [0, 93, 95, 98, 170, 173, 176, 179, 62, 401] {
            assert!(!apply_potion_set_effect_arm(&mut e, id, 600, &ctx), "id {id}");
        }
        assert_eq!(e, PotionSetEffects::default());
    }
}