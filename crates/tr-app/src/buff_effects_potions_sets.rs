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

use crate::buffs::{BuffSlots, MAX_BUFFS};
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

/// The slot-ordered driver over the block's ids — the C# `for j` chain
/// (`Player.cs:9865`) restricted to `:10318-10461`. The caller runs the timer
/// tick FIRST (`buffs.rs::update_buffs_tick`, the `:9871-9874` decrement);
/// this driver does NOT decrement. The `DelBuff(j); j--` rescan mirrors
/// `player.rs::update_summon_buff_arms`: after a delete the compaction shifts
/// the later buffs left, so index `j` is re-examined instead of advanced.
///
/// The C# double-delete path (the downgrade branch AND the set-flag-off
/// branch both firing on one slot, driving `j` negative — `DelBuff(-1)`
/// would throw in C#) is unreachable in retail play: `beetleOrbs > 0` at arm
/// entry requires an earlier KEPT slot this tick (the fields reset to 0
/// every frame), a kept slot's own set flag is on, and the two beetle flags
/// are mutually exclusive (one chestpiece, `:16501`/`:16532`). A cross-family
/// downgrade needs a kept Endurance slot (defense on) before a live Might
/// slot (offense off) — but the off-family buff is purged on the FIRST tick
/// after the armor swap (its 5-tick pin only refreshes while its own flag is
/// on), so the two cannot coexist. One rescan per arm is therefore exact.
///
/// The same reasoning covers the lower-tier walk deleting the CURRENT slot
/// (the upgrade branch includes `j` in its range): the walk runs only when a
/// same-family lower tier was KEPT earlier this tick, which needs the same
/// set flag on — the flag-off `DelBuff(j)` then never fires on the emptied
/// slot. (A stale time-0 slot of the family could ride the walk's type-only
/// test, but `AddBuff` never leaves one: it refreshes in place, `:5263-5295`.)
pub fn update_potion_set_buff_arms(
    slots: &mut BuffSlots,
    e: &mut PotionSetEffects,
    ctx: &PotionEffectContext,
    sets: &SetBonusFlags,
) {
    let mut j = 0usize;
    while j < MAX_BUFFS {
        let buff = slots.buff_type[j];
        if buff <= 0 || slots.buff_time[j] <= 0 {
            j += 1; // `:9867-9870`
            continue;
        }
        let rescan = match buff {
            94 | 396 => {
                apply_potion_set_effect_arm(e, buff, slots.buff_time[j], ctx);
                false
            }
            95..=97 => beetle_endurance_arm(slots, e, sets, j), // `:10342`
            170..=172 => solar_shield_arm(slots, e, sets, j),   // `:10377`
            98..=100 => beetle_might_arm(slots, e, sets, j),    // `:10408`
            176..=178 => {
                // `:10444-10447` — Nebula Mana: the level only.
                nebula_buffs(slots, &mut e.nebula_level_mana, 176, j);
                false
            }
            173..=175 => {
                // `:10448-10452` — Nebula Life: level + regen ×6 per level.
                nebula_buffs(slots, &mut e.nebula_level_life, 173, j);
                e.life_regen += 6 * e.nebula_level_life; // `:10451`
                false
            }
            179..=181 => {
                // `:10453-10461` — Nebula Damage: level + 0.15×level to all
                // four classes (one num12, added to each).
                nebula_buffs(slots, &mut e.nebula_level_damage, 179, j);
                let num12 = 0.15f32 * e.nebula_level_damage as f32; // `:10456`
                e.melee_damage += num12; // `:10457`
                e.ranged_damage += num12; // `:10458`
                e.magic_damage += num12; // `:10459`
                e.minion_damage += num12; // `:10460`
                false
            }
            // Not this block's id — the chain leaves the slot alone.
            _ => false,
        };
        if !rescan {
            j += 1;
        }
        // else: the slot was DelBuff'd — rescan index j (`j--` then `j++`).
    }
}

/// The lower-tier `DelBuff` walk shared by the Beetle/Solar arms
/// (`Player.cs:10355-10362` / `:10390-10397` / `:10420-10427`): delete every
/// slot whose id sits in `base..=top`, with the `DelBuff(m); m--` rescan.
/// NOTE: the walk reads `buffType` only (no `buffTime` gate) and INCLUDES the
/// current slot `j` (its id is `top` by construction) — after the walk the
/// caller's `j` holds a compacted foreign buff, which the C# chain then
/// skips (`j++`, no rescan in this branch). Reproduced verbatim.
fn del_lower_tiers(slots: &mut BuffSlots, base: i32, top: i32) {
    let mut m = 0usize;
    while m < MAX_BUFFS {
        if slots.buff_type[m] >= base && slots.buff_type[m] <= top {
            slots.del_buff(m); // rescan m (`m--` then `m++`)
        } else {
            m += 1;
        }
    }
}

/// Beetle Endurance, buffs 95-97 (`Player.cs:10342-10376`). Returns true when
/// slot `j` was DelBuff'd by the arm itself (the caller rescans `j`).
fn beetle_endurance_arm(
    slots: &mut BuffSlots,
    e: &mut PotionSetEffects,
    sets: &SetBonusFlags,
    j: usize,
) -> bool {
    slots.buff_time[j] = 5; // `:10344`
    let num8 = 1 + slots.buff_type[j] - 95; // `:10345`
    let mut rescan = false;
    if e.beetle_orbs > 0 && e.beetle_orbs != num8 {
        // `:10346-10364`
        if e.beetle_orbs > num8 {
            slots.del_buff(j); // `:10350-10351`
            rescan = true;
        } else {
            del_lower_tiers(slots, 95, 95 + num8 - 1); // `:10355-10362`
        }
    }
    e.beetle_orbs = num8; // `:10365`
    if !sets.beetle_defense {
        e.beetle_orbs = 0; // `:10368`
        slots.del_buff(j); // `:10369-10370`
        rescan = true;
    } else {
        e.beetle_buff = true; // `:10374`
    }
    rescan
}

/// Solar Shield, buffs 170-172 (`Player.cs:10377-10407`). Same tier shape as
/// Beetle Endurance minus the `beetleBuff` flag; the gate is `setSolar`.
fn solar_shield_arm(
    slots: &mut BuffSlots,
    e: &mut PotionSetEffects,
    sets: &SetBonusFlags,
    j: usize,
) -> bool {
    slots.buff_time[j] = 5; // `:10379`
    let num9 = 1 + slots.buff_type[j] - 170; // `:10380`
    let mut rescan = false;
    if e.solar_shields > 0 && e.solar_shields != num9 {
        // `:10381-10399`
        if e.solar_shields > num9 {
            slots.del_buff(j); // `:10385-10386`
            rescan = true;
        } else {
            del_lower_tiers(slots, 170, 170 + num9 - 1); // `:10390-10397`
        }
    }
    e.solar_shields = num9; // `:10400`
    if !sets.set_solar {
        e.solar_shields = 0; // `:10403`
        slots.del_buff(j); // `:10404-10405`
        rescan = true;
    }
    rescan
}

/// Beetle Might, buffs 98-100 (`Player.cs:10408-10443`). NO `buffTime = 5`
/// refresh (unlike Endurance); the stat adds land BEFORE the `beetleOffense`
/// gate (`:10431-10432` precede `:10433`) — a dropped buff still adds its
/// damage/speed for this tick, verbatim.
fn beetle_might_arm(
    slots: &mut BuffSlots,
    e: &mut PotionSetEffects,
    sets: &SetBonusFlags,
    j: usize,
) -> bool {
    let num10 = 1 + slots.buff_type[j] - 98; // `:10410`
    let mut rescan = false;
    if e.beetle_orbs > 0 && e.beetle_orbs != num10 {
        // `:10411-10429`
        if e.beetle_orbs > num10 {
            slots.del_buff(j); // `:10415-10416`
            rescan = true;
        } else {
            del_lower_tiers(slots, 98, 98 + num10 - 1); // `:10420-10427`
        }
    }
    e.beetle_orbs = num10; // `:10430`
    e.melee_damage += 0.1f32 * e.beetle_orbs as f32; // `:10431`
    e.melee_speed += 0.1f32 * e.beetle_orbs as f32; // `:10432`
    if !sets.beetle_offense {
        e.beetle_orbs = 0; // `:10435`
        slots.del_buff(j); // `:10436-10437`
        rescan = true;
    } else {
        e.beetle_buff = true; // `:10441`
    }
    rescan
}

/// `Player.UpdateBuffs_NebulaBuffs` (`Player.cs:12070-12079`): the level is
/// `1 + buffType[b] - baseBuffId`; at `buffTime == 2` with level > 1 the buff
/// downgrades one tier — level−1, `buffType[b]--`, `buffTime[b] = 480`.
/// Deterministic timer arithmetic, no RNG. The downgrade rewrites the slot
/// in place (no DelBuff), so the driver advances normally.
fn nebula_buffs(slots: &mut BuffSlots, nebula_level: &mut i32, base_buff_id: i32, b: usize) {
    *nebula_level = 1 + slots.buff_type[b] - base_buff_id; // `:12072`
    if slots.buff_time[b] == 2 && *nebula_level > 1 {
        // `:12073-12078`
        *nebula_level -= 1;
        slots.buff_type[b] -= 1;
        slots.buff_time[b] = 480;
    }
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

    // --- The slot-ordered driver + the set-tier arms -----------------------

    fn run(slots: &mut BuffSlots, sets: SetBonusFlags) -> PotionSetEffects {
        let mut e = PotionSetEffects::default();
        update_potion_set_buff_arms(slots, &mut e, &PotionEffectContext::default(), &sets);
        e
    }

    #[test]
    fn s1_e_7c17_beetle_endurance_pins_5_and_sets_orbs_and_flag() {
        let mut s = BuffSlots::default();
        s.add_buff(96, 3);
        let sets = SetBonusFlags { beetle_defense: true, ..Default::default() };
        let e = run(&mut s, sets);
        // `:10344-10345`: time pinned to 5, level = 1 + 96 − 95 = 2.
        assert_eq!(s.buff_time[0], 5);
        assert_eq!(e.beetle_orbs, 2); // `:10365`
        assert!(e.beetle_buff); // `:10374`
    }

    #[test]
    fn s1_e_7c17_beetle_endurance_without_the_set_drops_the_buff() {
        let mut s = BuffSlots::default();
        s.add_buff(97, 600);
        let e = run(&mut s, SetBonusFlags::default());
        // `:10366-10371`: orbs reset to 0, DelBuff(j), no beetleBuff.
        assert_eq!(e.beetle_orbs, 0);
        assert!(!e.beetle_buff);
        assert_eq!(s.buff_type[0], 0);
        assert_eq!(s.buff_time[0], 0);
    }

    #[test]
    fn s1_e_7c17_beetle_downgrade_deletes_the_lower_slot_and_rescans() {
        let mut s = BuffSlots::default();
        s.add_buff(97, 600); // slot 0: tier 3
        s.add_buff(95, 600); // slot 1: tier 1 — the downgrade target
        s.add_buff(94, 300); // slot 2: the rescan witness (compacts into 1)
        let sets = SetBonusFlags { beetle_defense: true, ..Default::default() };
        let e = run(&mut s, sets);
        // `:10348-10352`: orbs 3 > 1 → DelBuff(1); j-- rescans the compacted
        // 94, whose arm runs in the SAME pass (`:10318-10327`). The arm's
        // tail STILL runs after the delete (`:10365`): orbs = num8 = 1.
        assert_eq!(s.buff_type[0], 97);
        assert_eq!(s.buff_type[1], 94);
        assert_eq!(e.beetle_orbs, 1, "the arm tail sets num8 after the delete");
        assert!(e.mana_sick, "the rescan ran the shifted-in 94's arm");
        assert_eq!(e.mana_sick_reduction, 0.25);
    }

    #[test]
    fn s1_e_7c17_beetle_upgrade_walk_deletes_the_lower_tiers_including_j() {
        let mut s = BuffSlots::default();
        s.add_buff(95, 600); // slot 0: tier 1, kept
        s.add_buff(97, 600); // slot 1: tier 3 — the upgrade
        s.add_buff(94, 300); // slot 2: compacts into 0 after the walk
        let sets = SetBonusFlags { beetle_defense: true, ..Default::default() };
        let e = run(&mut s, sets);
        // `:10355-10362`: the walk deletes 95..97 — BOTH family slots. The
        // chain then advances (no rescan in this branch), skipping the
        // compacted 94 at slot 0 for this pass — its arm does NOT run.
        assert_eq!(s.buff_type[0], 94);
        assert!(!e.mana_sick, "the upgrade branch skips the compacted slot");
        assert_eq!(e.beetle_orbs, 3); // `:10365` after the walk
        assert!(e.beetle_buff);
    }

    #[test]
    fn s1_e_7c17_solar_shield_pins_5_and_gates_on_set_solar() {
        let mut s = BuffSlots::default();
        s.add_buff(171, 2);
        let e = run(&mut s, SetBonusFlags { set_solar: true, ..Default::default() });
        assert_eq!(s.buff_time[0], 5); // `:10379`
        assert_eq!(e.solar_shields, 2); // `:10380`/`:10400`
        assert!(!e.beetle_buff, "the Solar arm sets no beetleBuff");
        // Without the set: `:10401-10406` — shields 0, DelBuff.
        let mut s = BuffSlots::default();
        s.add_buff(172, 600);
        let e = run(&mut s, SetBonusFlags::default());
        assert_eq!(e.solar_shields, 0);
        assert_eq!(s.buff_type[0], 0);
    }

    #[test]
    fn s1_e_7c17_solar_downgrade_and_upgrade_mirror_the_beetle_shape() {
        // Downgrade: shields 3 → tier-2 slot deleted, rescan.
        let mut s = BuffSlots::default();
        s.add_buff(172, 600);
        s.add_buff(171, 600);
        let sets = SetBonusFlags { set_solar: true, ..Default::default() };
        let e = run(&mut s, sets);
        assert_eq!(s.buff_type[1], 0);
        // The arm tail runs after the delete (`:10400`): shields = num9 = 2.
        assert_eq!(e.solar_shields, 2);
        // Upgrade: the walk takes 170..172 including j (`:10390-10397`).
        let mut s = BuffSlots::default();
        s.add_buff(170, 600);
        s.add_buff(172, 600);
        let e = run(&mut s, sets);
        assert_eq!(s.buff_type[0], 0);
        assert_eq!(e.solar_shields, 3);
    }

    #[test]
    fn s1_e_7c17_beetle_might_adds_damage_and_speed_before_the_gate() {
        let mut s = BuffSlots::default();
        s.add_buff(99, 600);
        let sets = SetBonusFlags { beetle_offense: true, ..Default::default() };
        let e = run(&mut s, sets);
        // `:10430-10432`: level 2 → +0.2 melee damage AND speed. NO time
        // pin (unlike Endurance) — the timer keeps its value.
        assert_eq!(e.beetle_orbs, 2);
        assert!((e.melee_damage - 0.2).abs() < 1e-6);
        assert!((e.melee_speed - 0.2).abs() < 1e-6);
        assert!(e.beetle_buff); // `:10441`
        assert_eq!(s.buff_time[0], 600, "no buffTime = 5 in the Might arm");
        // Gate off (`:10433-10438`): the adds LANDED (`:10431-10432` precede
        // the gate), then orbs 0 + DelBuff, no beetleBuff.
        let mut s = BuffSlots::default();
        s.add_buff(100, 600);
        let e = run(&mut s, SetBonusFlags::default());
        assert!((e.melee_damage - 0.3).abs() < 1e-6, "the add precedes the drop");
        assert_eq!(e.beetle_orbs, 0);
        assert!(!e.beetle_buff);
        assert_eq!(s.buff_type[0], 0);
    }

    #[test]
    fn s1_e_7c17_nebula_levels_and_the_time_2_downgrade() {
        let mut s = BuffSlots::default();
        s.add_buff(178, 100); // Mana tier 3
        s.add_buff(175, 2);   // Life tier 3 at time 2 — downgrades
        s.add_buff(181, 600); // Damage tier 3
        let e = run(&mut s, SetBonusFlags::default());
        assert_eq!(e.nebula_level_mana, 3); // `:10446` → `:12072`
        // `:12073-12078`: level 3→2, buffType 175→174, time 2→480.
        assert_eq!(e.nebula_level_life, 2);
        assert_eq!(s.buff_type[1], 174);
        assert_eq!(s.buff_time[1], 480);
        assert_eq!(e.life_regen, 12); // `:10451`: 6 × the POST-downgrade level
        // `:10456-10460`: 0.15 × 3 = 0.45 to all four classes.
        assert_eq!(e.nebula_level_damage, 3);
        for d in [e.melee_damage, e.ranged_damage, e.magic_damage, e.minion_damage] {
            assert!((d - 0.45).abs() < 1e-6);
        }
    }

    #[test]
    fn s1_e_7c17_nebula_tier_1_at_time_2_does_not_downgrade() {
        let mut s = BuffSlots::default();
        s.add_buff(173, 2); // Life tier 1 — the level > 1 gate holds it
        let e = run(&mut s, SetBonusFlags::default());
        assert_eq!(e.nebula_level_life, 1);
        assert_eq!(s.buff_type[0], 173);
        assert_eq!(s.buff_time[0], 2);
        assert_eq!(e.life_regen, 6);
    }

    #[test]
    fn s1_e_7c17_nebula_downgrade_chains_across_ticks() {
        // The 480 re-pin ticks down to 2 again → another tier drop.
        let mut s = BuffSlots::default();
        s.add_buff(175, 2);
        let sets = SetBonusFlags::default();
        let e = run(&mut s, sets); // 175 → 174 @ 480
        assert_eq!(s.buff_type[0], 174);
        s.buff_time[0] = 2; // simulate the later tick (timer engine's job)
        let e = run(&mut s, sets);
        assert_eq!(e.nebula_level_life, 1);
        assert_eq!(s.buff_type[0], 173);
        assert_eq!(s.buff_time[0], 480);
        // Tier 1 at time 2: no further downgrade.
        s.buff_time[0] = 2;
        let e = run(&mut s, sets);
        assert_eq!(s.buff_type[0], 173);
        assert_eq!(s.buff_time[0], 2);
        assert_eq!(e.life_regen, 6);
    }

    #[test]
    fn s1_e_7c17_the_driver_leaves_foreign_and_dead_slots_alone() {
        let mut s = BuffSlots::default();
        s.add_buff(5, 600);  // Ironskin — wave A's owner
        s.add_buff(62, 600); // Ice Barrier — the live partial, NOT this block
        s.buff_type[2] = 95; // a time-0 beetle slot: the `:9867` guard skips
        s.buff_time[2] = 0;
        let before = s.clone();
        let e = run(&mut s, SetBonusFlags::default());
        assert_eq!(s, before, "no writes outside the block's live ids");
        assert_eq!(e, PotionSetEffects::default());
    }
}