//! S1-E-7 wave G — the SIXTH missing block of `Player.UpdateBuffs`
//! (`Player.cs:9855-12069`): the BOOSTER block `Player.cs:10328-10485`
//! (Mana Heat 396, the Beetle/Solar/Nebula tier boosters 95-100/170-181,
//! the Ice Barrier 62 completion) plus the two wave-A stragglers
//! `:9935-9946` (Sugar Rush 192, Brain of Confusion buff 321).
//!
//! Disjoint-wave design (ANALYSIS "The booster buff-effect block (S1-E-7
//! wave G)"): waves A/B edit the `player.rs` effect match, C/D own
//! `buff_effects_accessory.rs`/`buff_effects_misc.rs`. This module is a PURE
//! effect table — `apply_booster_effect_arm` matches one `buffType[j]` id and
//! folds its stat/flag writes into a `BoosterEffects` accumulator, exactly
//! like the C# arm writes into the `Player` fields. The persistent counters
//! (`beetleOrbs`, `solarShields`, the three nebula levels, the ice-barrier
//! frame pair) live in `BoosterState` — the caller keeps them across ticks
//! like the C# `Player` fields. The slot mutations (`buffTime[j] = 5`, the
//! tier `DelBuff` sweeps, the nebula step-down, the ice-barrier delete) run
//! on `BuffSlots` directly, the `update_summon_buff_arms` shape; the
//! `player.rs` scan wiring rides the waves' merge step.
//!
//! Keep-one notes for the merge step:
//! - Ice Barrier 62: the endurance half already sits in the `player.rs`
//!   match (`player.rs:6526`, reading `max_hp`) and the light in
//!   `player_lights.rs:280`. This module ports the WHOLE arm (the C# gate is
//!   `statLifeMax2`, the flag + the 3-tick × 12-frame counter + the
//!   above-half-life `DelBuff(j); j--` are the missing halves); the merge
//!   folds the `player.rs` 62 arm into this one.
//! - Mana Sickness 94 (`:10318-10327`) is already ported
//!   (`potions::apply_mana_sickness_arm`) — NOT re-claimed.
//!
//! RNG census: ZERO draws — `Main.rand`/`Next(`/`Random` appear nowhere in
//! `:10328-10485`, `:9935-9946` or the nebula helper `:12070-12079`. No
//! stream is touched by this wave.

use crate::buffs::{BuffSlots, MAX_BUFFS};

/// The per-tick context the arms read besides `buffType`/`buffTime`.
/// `whoAmI == Main.myPlayer` collapses to true in the single-player port,
/// matching every other ported `UpdateBuffs` arm.
#[derive(Clone, Copy, Debug, Default)]
pub struct BoosterEffectContext {
    /// `statLife` — the Ice Barrier gate (`:10464`).
    pub stat_life: i32,
    /// `statLifeMax2` — the post-equipment life cap the gate reads
    /// (`:10464`, NOT the base `statLifeMax`).
    pub stat_life_max2: i32,
    /// `beetleDefense` — the Beetle Shell/Scale Mail set flag (`:10366`).
    pub beetle_defense: bool,
    /// `beetleOffense` — the Beetle Scale Mail offense set flag (`:10433`).
    pub beetle_offense: bool,
    /// `setSolar` — the Solar Flare set flag (`:10401`).
    pub set_solar: bool,
    /// `latestManaPotionDamageBonus` (`:10330`) — the Mana Heat power.
    pub latest_mana_potion_damage_bonus: f32,
    /// `latestManaPotionDuration` (`:10331`) — the Mana Heat duration in
    /// ticks; the arm's `== 0 → 1` write-back rides the accumulator.
    pub latest_mana_potion_duration: f32,
}

/// The persistent counters the block reads/writes across ticks — the C#
/// `Player` fields `beetleOrbs`/`solarShields`/`nebulaLevel*`/
/// `iceBarrierFrameCounter`/`iceBarrierFrame`. The caller keeps one of these
/// per player; `ResetEffects` does NOT clear them in the C# (they persist
/// while the buff family is absent), so the port does not either.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BoosterState {
    /// `beetleOrbs` (`:10365`/`:10430`) — shared by BOTH beetle families.
    pub beetle_orbs: i32,
    /// `solarShields` (`:10400`).
    pub solar_shields: i32,
    /// `nebulaLevelLife` (`:10450`).
    pub nebula_level_life: i32,
    /// `nebulaLevelMana` (`:10446`).
    pub nebula_level_mana: i32,
    /// `nebulaLevelDamage` (`:10455`).
    pub nebula_level_damage: i32,
    /// `iceBarrierFrameCounter` (`:10469-10472`) — the 3-tick divider.
    pub ice_barrier_frame_counter: i32,
    /// `iceBarrierFrame` (`:10473-10477`) — the 12-frame cycle.
    pub ice_barrier_frame: i32,
}

/// One tick's accumulated writes of the booster block. Flags OR, stats ADD
/// per slot exactly like the C# chain; the `Option` setters are assignments
/// (`latestManaPotionDuration = 1f`). Slot mutations (time pins, deletes,
/// the nebula tier step-down) are NOT here — they run on `BuffSlots`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoosterEffects {
    // Flags.
    pub beetle_buff: bool,   // 95-97/98-100, `:10374`/`:10441`
    pub ice_barrier: bool,   // 62, `:10467`
    // Stat adds (per-slot, summed like the C# chain).
    pub mana_heat: f32,      // 396, `:10340` `manaHeat += num6`
    pub melee_damage: f32,   // 98-100 `:10431`, 179-181 `:10457`, 321 `:9943`
    pub melee_speed: f32,    // 98-100, `:10432`
    pub ranged_damage: f32,  // 179-181, `:10458`
    pub magic_damage: f32,   // 179-181, `:10459`
    pub minion_damage: f32,  // 179-181 `:10460`, 321 `:9946`
    pub life_regen: i32,     // 173-175, `:10451` `lifeRegen += 6 * level`
    pub endurance: f32,      // 62, `:10468` +0.25 at/below half life
    pub melee_crit: i32,     // 321, `:9943` +10
    pub ranged_crit: i32,    // 321, `:9944` +10
    pub magic_crit: i32,     // 321, `:9945` +10
    pub move_speed: f32,     // 192, `:9938` +0.2
    pub pick_speed: f32,     // 192, `:9937` −0.2 (the port's positive-bonus
    // convention: the caller adds this to `pick_speed_bonus`, like the
    // Mining 104 arm's `pickSpeed -= 0.25` → `+0.25`).
    /// `latestManaPotionDuration = 1f` (`:10333`) — the zero-duration
    /// write-back; `None` leaves the field alone.
    pub latest_mana_potion_duration_set: Option<f32>,
}

/// The nebula tier window (`UpdateBuffs_NebulaBuffs`, `:12077`): a tier
/// that hits `buffTime == 2` steps down and re-arms at 480 ticks.
pub const NEBULA_TIER_TIME: i32 = 480; // `:12077`
/// The beetle/solar orb pin (`:10344`/`:10379`): `buffTime[j] = 5` — the
/// armor set re-grants the buff every tick while worn.
pub const ORB_BUFF_TIME: i32 = 5;

/// `Player.UpdateBuffs_NebulaBuffs` (`Player.cs:12070-12079`), verbatim:
/// `nebulaLevel = 1 + buffType[b] - baseBuffId`; at `buffTime[b] == 2` with
/// `level > 1` the tier steps down (`level--`, `buffType[b]--`,
/// `buffTime[b] = 480`). Returns the level the caller's stat adds read.
fn nebula_buffs(slots: &mut BuffSlots, level: &mut i32, base_buff_id: i32, b: usize) -> i32 {
    *level = 1 + slots.buff_type[b] - base_buff_id; // `:12072`
    if slots.buff_time[b] == 2 && *level > 1 {
        // `:12073-12078`
        *level -= 1;
        slots.buff_type[b] -= 1;
        slots.buff_time[b] = NEBULA_TIER_TIME;
    }
    *level
}

/// The Mana Heat arm (`Player.cs:10328-10341`): `manaHeat += bonus *
/// (buffTime / duration)`, the `duration == 0 → 1` write-back included.
/// The `if (0f == 0f)` at `:10336` is decompilation noise around an
/// always-true guard — the multiply always runs.
fn mana_heat_arm(e: &mut BoosterEffects, ctx: &BoosterEffectContext, buff_time: i32) {
    let mut bonus = ctx.latest_mana_potion_damage_bonus; // `:10330` num6
    let mut duration = ctx.latest_mana_potion_duration; // `:10331`
    if duration == 0.0 {
        duration = 1.0; // `:10331-10334`
        e.latest_mana_potion_duration_set = Some(1.0);
    }
    let ratio = buff_time as f32 / duration; // `:10335` num7
    bonus *= ratio; // `:10338`
    e.mana_heat += bonus; // `:10340`
}

/// The Sugar Rush arm (`Player.cs:9935-9939`): `pickSpeed -= 0.2f`,
/// `moveSpeed += 0.2f`. Wave-A range, missed by A — claimed here.
fn sugar_rush_arm(e: &mut BoosterEffects) {
    e.pick_speed -= 0.2; // `:9937`
    e.move_speed += 0.2; // `:9938`
}

/// The Brain of Confusion buff arm (`Player.cs:9940-9947`): `num = 10`,
/// +10 melee/ranged/magic crit, `minionDamage += num / 100f`. The dodge
/// GRANT side (`:38046-38055`) is already ported (`player.rs:4233`).
fn brain_of_confusion_arm(e: &mut BoosterEffects) {
    let num = 10; // `:9942`
    e.melee_crit += num; // `:9943`
    e.ranged_crit += num; // `:9944`
    e.magic_crit += num; // `:9945`
    e.minion_damage += num as f32 / 100.0; // `:9946`
}

/// The result of one arm: either the id is outside this wave (the caller's
/// `else if` chain continues) or the caller's NEXT slot index — the C# `j`
/// after the arm plus the `for` loop's `j++`. A plain keep is `j + 1`; a
/// `DelBuff(j); j--` rescan is `j`; the beetle/solar double-delete quirk
/// (below) can be `j - 1`. The caller clamps to `>= 0`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoosterArmResult {
    NotHandled,
    NextSlot(i32),
}

/// The shared orb-tier sweep of the Beetle/Solar families
/// (`Player.cs:10346-10364` / `:10381-10399` / `:10411-10429`): when the
/// stored orb count differs from this slot's tier `num`, a DOWNGRADE
/// (`orbs > num`) deletes just this slot (`DelBuff(j); j--`), an UPGRADE
/// deletes every slot whose tier is `base..=base + num - 1` (the lower
/// tiers, INCLUDING this slot) with the `m--` rescan. Returns the adjusted
/// `j` (the downgrade's `j--`; the upgrade leaves `j` alone).
fn orb_tier_sweep(slots: &mut BuffSlots, j: usize, orbs: i32, num: i32, base: i32) -> i32 {
    let mut jj = j as i32;
    if orbs > 0 && orbs != num {
        if orbs > num {
            slots.del_buff(jj as usize); // `:10350`
            jj -= 1; // `:10351`
        } else {
            // `:10355-10362` — the `for m … DelBuff(m); m--` rescan.
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
/// unreachable in practice, the buff is only granted while the set is worn).
fn orb_unset_tail(slots: &mut BuffSlots, orbs: &mut i32, jj: i32) -> i32 {
    *orbs = 0;
    if jj >= 0 {
        slots.del_buff(jj as usize);
    }
    // The arm's `j--` cancels the loop's `j++` → the next index is `jj`.
    jj
}

/// The Beetle Endurance arm (`Player.cs:10342-10376`, ids 95-97): pin
/// `buffTime = 5`, the tier sweep on the shared `beetleOrbs`, then the
/// `beetleDefense` gate (`beetleBuff = true` when set).
fn beetle_endurance_arm(
    e: &mut BoosterEffects,
    s: &mut BoosterState,
    slots: &mut BuffSlots,
    ctx: &BoosterEffectContext,
    j: usize,
) -> i32 {
    slots.buff_time[j] = ORB_BUFF_TIME; // `:10344`
    let num = 1 + slots.buff_type[j] - 95; // `:10345` num8
    let jj = orb_tier_sweep(slots, j, s.beetle_orbs, num, 95);
    s.beetle_orbs = num; // `:10365`
    if !ctx.beetle_defense {
        orb_unset_tail(slots, &mut s.beetle_orbs, jj) // `:10366-10371`
    } else {
        e.beetle_buff = true; // `:10374`
        jj + 1
    }
}

/// The Solar Shield arm (`Player.cs:10377-10407`, ids 170-172): the same
/// shape on `solarShields`/`setSolar`; NO `beetleBuff` flag, no stat adds.
fn solar_shield_arm(
    s: &mut BoosterState,
    slots: &mut BuffSlots,
    ctx: &BoosterEffectContext,
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

/// The Beetle Might arm (`Player.cs:10408-10443`, ids 98-100): NO time pin;
/// the tier sweep on the shared `beetleOrbs`, then `meleeDamage`/`meleeSpeed
/// += 0.1 * orbs` (read AFTER the assignment, BEFORE the offense gate — the
/// stats land even on the delete tick), then the `beetleOffense` gate.
fn beetle_might_arm(
    e: &mut BoosterEffects,
    s: &mut BoosterState,
    slots: &mut BuffSlots,
    ctx: &BoosterEffectContext,
    j: usize,
) -> i32 {
    let num = 1 + slots.buff_type[j] - 98; // `:10410` num10
    let jj = orb_tier_sweep(slots, j, s.beetle_orbs, num, 98);
    s.beetle_orbs = num; // `:10430`
    e.melee_damage += 0.1 * s.beetle_orbs as f32; // `:10431`
    e.melee_speed += 0.1 * s.beetle_orbs as f32; // `:10432`
    if !ctx.beetle_offense {
        orb_unset_tail(slots, &mut s.beetle_orbs, jj) // `:10433-10438`
    } else {
        e.beetle_buff = true; // `:10441`
        jj + 1
    }
}

/// The Ice Barrier arm (`Player.cs:10462-10485`, id 62): at/below half life
/// (`statLife <= statLifeMax2 * 0.5`, the f64 compare) set the flag, +0.25
/// endurance and advance the 3-tick × 12-frame counter; above half life
/// `DelBuff(j); j--`. The `Lighting.AddLight` half (`:10466`) already rides
/// `player_lights.rs:280` (the frame feed) — not re-emitted here.
fn ice_barrier_arm(
    e: &mut BoosterEffects,
    s: &mut BoosterState,
    slots: &mut BuffSlots,
    ctx: &BoosterEffectContext,
    j: usize,
) -> i32 {
    if (ctx.stat_life as f64) <= (ctx.stat_life_max2 as f64) * 0.5 {
        // `:10464`
        e.ice_barrier = true; // `:10467`
        e.endurance += 0.25; // `:10468`
        s.ice_barrier_frame_counter += 1; // `:10469`
        if s.ice_barrier_frame_counter > 2 {
            // `:10470-10478`
            s.ice_barrier_frame_counter = 0;
            s.ice_barrier_frame += 1;
            if s.ice_barrier_frame >= 12 {
                s.ice_barrier_frame = 0;
            }
        }
        j as i32 + 1
    } else {
        slots.del_buff(j); // `:10482`
        j as i32 // `:10483` j-- (+ the loop's j++ → stay).
    }
}

/// One arm of the booster block. `j` is the live slot index; `buff_type` /
/// `buff_time` are read from `slots[j]` by the caller's chain. Returns
/// `NotHandled` for ids outside this wave (the caller's `else if` continues),
/// else the next slot index. Stat/flag writes fold into `e`, the persistent
/// counters into `s`, the slot mutations run on `slots`.
pub fn apply_booster_effect_arm(
    e: &mut BoosterEffects,
    s: &mut BoosterState,
    slots: &mut BuffSlots,
    ctx: &BoosterEffectContext,
    j: usize,
) -> BoosterArmResult {
    let buff_type = slots.buff_type[j];
    let buff_time = slots.buff_time[j];
    let next = match buff_type {
        396 => {
            mana_heat_arm(e, ctx, buff_time); // `:10328-10341`
            j as i32 + 1
        }
        95..=97 => beetle_endurance_arm(e, s, slots, ctx, j), // `:10342`
        170..=172 => solar_shield_arm(s, slots, ctx, j),      // `:10377`
        98..=100 => beetle_might_arm(e, s, slots, ctx, j),    // `:10408`
        176..=178 => {
            // `:10444-10447` — mana tier, no stat add (the mana-regen
            // consumer reads `nebulaLevelMana` elsewhere).
            nebula_buffs(slots, &mut s.nebula_level_mana, 176, j);
            j as i32 + 1
        }
        173..=175 => {
            // `:10448-10452`
            let level = nebula_buffs(slots, &mut s.nebula_level_life, 173, j);
            e.life_regen += 6 * level; // `:10451`
            j as i32 + 1
        }
        179..=181 => {
            // `:10453-10461`
            let level = nebula_buffs(slots, &mut s.nebula_level_damage, 179, j);
            let num12 = 0.15 * level as f32; // `:10456`
            e.melee_damage += num12; // `:10457`
            e.ranged_damage += num12; // `:10458`
            e.magic_damage += num12; // `:10459`
            e.minion_damage += num12; // `:10460`
            j as i32 + 1
        }
        62 => ice_barrier_arm(e, s, slots, ctx, j), // `:10462`
        192 => {
            sugar_rush_arm(e); // `:9935-9939`
            j as i32 + 1
        }
        321 => {
            brain_of_confusion_arm(e); // `:9940-9947`
            j as i32 + 1
        }
        _ => return BoosterArmResult::NotHandled,
    };
    BoosterArmResult::NextSlot(next)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mid-scan slot state, set directly (the C# chain reads the arrays
    /// after the timer head's decrement; `add_buff`'s 1.2x quirk is not
    /// this block's concern).
    fn slots(rows: &[(i32, i32)]) -> BuffSlots {
        let mut s = BuffSlots::default();
        for (k, &(bt, time)) in rows.iter().enumerate() {
            s.buff_type[k] = bt;
            s.buff_time[k] = time;
        }
        s
    }

    fn run(
        s: &mut BoosterState,
        sl: &mut BuffSlots,
        ctx: &BoosterEffectContext,
        j: usize,
    ) -> (BoosterEffects, BoosterArmResult) {
        let mut e = BoosterEffects::default();
        let r = apply_booster_effect_arm(&mut e, s, sl, ctx, j);
        (e, r)
    }

    #[test]
    fn s1_e_7g_396_mana_heat_decays_the_bonus_by_the_time_ratio() {
        // `:10330-10340`: bonus 0.2, duration 120, time 60 -> +0.1.
        let ctx = BoosterEffectContext {
            latest_mana_potion_damage_bonus: 0.2,
            latest_mana_potion_duration: 120.0,
            ..Default::default()
        };
        let mut sl = slots(&[(396, 60)]);
        let (e, r) = run(&mut BoosterState::default(), &mut sl, &ctx, 0);
        assert!((e.mana_heat - 0.1).abs() < 1e-6);
        assert_eq!(e.latest_mana_potion_duration_set, None);
        assert_eq!(r, BoosterArmResult::NextSlot(1));
    }

    #[test]
    fn s1_e_7g_396_zero_duration_writes_back_one_and_divides_by_it() {
        // `:10331-10334`: duration == 0 -> 1f (the field write-back), the
        // ratio then reads time / 1.
        let ctx = BoosterEffectContext {
            latest_mana_potion_damage_bonus: 0.25,
            latest_mana_potion_duration: 0.0,
            ..Default::default()
        };
        let mut sl = slots(&[(396, 3)]);
        let (e, _) = run(&mut BoosterState::default(), &mut sl, &ctx, 0);
        assert_eq!(e.latest_mana_potion_duration_set, Some(1.0));
        assert!((e.mana_heat - 0.75).abs() < 1e-6);
    }

    #[test]
    fn s1_e_7g_192_sugar_rush_pick_and_move() {
        // `:9937-9938`: pickSpeed -= 0.2, moveSpeed += 0.2.
        let mut sl = slots(&[(192, 100)]);
        let (e, r) = run(
            &mut BoosterState::default(),
            &mut sl,
            &BoosterEffectContext::default(),
            0,
        );
        assert!((e.pick_speed + 0.2).abs() < 1e-6);
        assert!((e.move_speed - 0.2).abs() < 1e-6);
        assert_eq!(r, BoosterArmResult::NextSlot(1));
    }

    #[test]
    fn s1_e_7g_321_brain_buff_crits_and_minion_damage() {
        // `:9942-9946`: +10/+10/+10 crit, minionDamage += 0.1.
        let mut sl = slots(&[(321, 240)]);
        let (e, _) = run(
            &mut BoosterState::default(),
            &mut sl,
            &BoosterEffectContext::default(),
            0,
        );
        assert_eq!((e.melee_crit, e.ranged_crit, e.magic_crit), (10, 10, 10));
        assert!((e.minion_damage - 0.1).abs() < 1e-6);
    }

    #[test]
    fn s1_e_7g_95_beetle_endurance_pins_five_and_sets_the_flag() {
        // `:10344-10374`: tier 1, no stored orbs, defense worn.
        let ctx = BoosterEffectContext { beetle_defense: true, ..Default::default() };
        let mut sl = slots(&[(95, 2)]);
        let mut st = BoosterState::default();
        let (e, r) = run(&mut st, &mut sl, &ctx, 0);
        assert_eq!(sl.buff_time[0], 5, "the orb pin (:10344)");
        assert_eq!(st.beetle_orbs, 1);
        assert!(e.beetle_buff);
        assert_eq!(r, BoosterArmResult::NextSlot(1));
    }

    #[test]
    fn s1_e_7g_97_upgrade_sweep_deletes_the_lower_tiers_including_self() {
        // `:10355-10362`: stored orbs 1, this slot tier 3 (97) -> every
        // slot in 95..=97 is DelBuff'd (the m-- rescan), orbs = 3.
        let ctx = BoosterEffectContext { beetle_defense: true, ..Default::default() };
        let mut sl = slots(&[(95, 5), (96, 5), (97, 5), (2, 100)]);
        let mut st = BoosterState { beetle_orbs: 1, ..Default::default() };
        let (e, r) = run(&mut st, &mut sl, &ctx, 2);
        assert_eq!(st.beetle_orbs, 3);
        assert!(e.beetle_buff);
        // The three beetle slots are gone; Regen (2) compacted to slot 0.
        assert_eq!(sl.buff_type[0], 2);
        assert_eq!(sl.buff_type[1], 0);
        assert_eq!(r, BoosterArmResult::NextSlot(3), "upgrade leaves j alone");
    }

    #[test]
    fn s1_e_7g_95_downgrade_deletes_self_and_rescans() {
        // `:10348-10352`: stored orbs 3, this slot tier 1 -> DelBuff(j);
        // j-- (the next index stays j), orbs = 1, flag still set.
        let ctx = BoosterEffectContext { beetle_defense: true, ..Default::default() };
        let mut sl = slots(&[(2, 100), (95, 5), (3, 100)]);
        let mut st = BoosterState { beetle_orbs: 3, ..Default::default() };
        let (e, r) = run(&mut st, &mut sl, &ctx, 1);
        assert_eq!(st.beetle_orbs, 1);
        assert!(e.beetle_buff);
        assert_eq!(sl.buff_type[1], 3, "Swiftness compacted into slot 1");
        assert_eq!(r, BoosterArmResult::NextSlot(1), "the j-- rescan");
    }

    #[test]
    fn s1_e_7g_95_defense_off_zeroes_orbs_and_deletes() {
        // `:10366-10371`: no set -> orbs = 0, DelBuff(j); j--, no flag.
        let mut sl = slots(&[(96, 5), (2, 100)]);
        let mut st = BoosterState::default();
        let (e, r) = run(&mut st, &mut sl, &BoosterEffectContext::default(), 0);
        assert_eq!(st.beetle_orbs, 0);
        assert!(!e.beetle_buff);
        assert_eq!(sl.buff_type[0], 2);
        assert_eq!(r, BoosterArmResult::NextSlot(0));
    }

    #[test]
    fn s1_e_7g_95_double_delete_quirk_hits_the_previous_slot() {
        // The C# quirk: downgrade at j=1 (j-- -> 0) THEN !beetleDefense
        // deletes the ALREADY-decremented j (slot 0). Both slots die.
        let mut sl = slots(&[(2, 100), (95, 5), (3, 100)]);
        let mut st = BoosterState { beetle_orbs: 3, ..Default::default() };
        let (_, r) = run(&mut st, &mut sl, &BoosterEffectContext::default(), 1);
        assert_eq!(st.beetle_orbs, 0);
        assert_eq!(sl.buff_type[0], 3, "slots 0 AND 1 deleted, 3 compacts down");
        assert_eq!(sl.buff_type[1], 0);
        assert_eq!(r, BoosterArmResult::NextSlot(0));
    }

    #[test]
    fn s1_e_7g_95_double_delete_at_slot_zero_guards_the_negative_index() {
        // Same quirk at j=0: the second delete's index is -1 — the C#
        // throws; the port guards (documented deviation). No panic.
        let mut sl = slots(&[(95, 5), (2, 100)]);
        let mut st = BoosterState { beetle_orbs: 2, ..Default::default() };
        let (_, r) = run(&mut st, &mut sl, &BoosterEffectContext::default(), 0);
        assert_eq!(st.beetle_orbs, 0);
        assert_eq!(sl.buff_type[0], 2, "only the first delete ran");
        assert_eq!(r, BoosterArmResult::NextSlot(-1), "the caller clamps");
    }

    #[test]
    fn s1_e_7g_98_beetle_might_no_pin_stats_before_the_gate() {
        // `:10408-10442`: NO buffTime pin; tier 2 -> +0.2 melee damage and
        // speed; offense worn -> flag.
        let ctx = BoosterEffectContext { beetle_offense: true, ..Default::default() };
        let mut sl = slots(&[(99, 3)]);
        let mut st = BoosterState::default();
        let (e, r) = run(&mut st, &mut sl, &ctx, 0);
        assert_eq!(sl.buff_time[0], 3, "no time pin in the Might arm");
        assert_eq!(st.beetle_orbs, 2);
        assert!((e.melee_damage - 0.2).abs() < 1e-6);
        assert!((e.melee_speed - 0.2).abs() < 1e-6);
        assert!(e.beetle_buff);
        assert_eq!(r, BoosterArmResult::NextSlot(1));
    }

    #[test]
    fn s1_e_7g_98_might_offense_off_still_adds_the_stats_then_deletes() {
        // `:10431-10432` run BEFORE the `:10433` gate — the stats land on
        // the delete tick too.
        let mut sl = slots(&[(100, 3)]);
        let mut st = BoosterState::default();
        let (e, r) = run(&mut st, &mut sl, &BoosterEffectContext::default(), 0);
        assert!((e.melee_damage - 0.3).abs() < 1e-6, "0.1 * 3 before the gate");
        assert_eq!(st.beetle_orbs, 0);
        assert!(!e.beetle_buff);
        assert_eq!(sl.buff_type[0], 0);
        assert_eq!(r, BoosterArmResult::NextSlot(0));
    }

    #[test]
    fn s1_e_7g_170_solar_shield_pins_and_gates_on_set_solar() {
        // `:10377-10406`: tier 3 pin 5, shields = 3, set worn -> keep; no
        // beetle_buff flag, no stat adds.
        let ctx = BoosterEffectContext { set_solar: true, ..Default::default() };
        let mut sl = slots(&[(172, 2)]);
        let mut st = BoosterState::default();
        let (e, r) = run(&mut st, &mut sl, &ctx, 0);
        assert_eq!(sl.buff_time[0], 5);
        assert_eq!(st.solar_shields, 3);
        assert!(!e.beetle_buff, "the Solar arm sets no beetleBuff");
        assert_eq!(e.melee_damage, 0.0);
        assert_eq!(r, BoosterArmResult::NextSlot(1));
        // Set absent -> zero + delete.
        let mut sl2 = slots(&[(170, 5)]);
        let mut st2 = BoosterState { solar_shields: 2, ..Default::default() };
        let (_, r2) = run(&mut st2, &mut sl2, &BoosterEffectContext::default(), 0);
        assert_eq!(st2.solar_shields, 0);
        assert_eq!(sl2.buff_type[0], 0);
        assert_eq!(r2, BoosterArmResult::NextSlot(0));
    }

    #[test]
    fn s1_e_7g_173_nebula_life_tier_steps_down_at_time_two() {
        // `:12072-12078` via `:10448-10452`: buff 174 (tier 2) at time 2
        // -> level 1, buffType 173, time 480; lifeRegen += 6 * 1.
        let mut sl = slots(&[(174, 2)]);
        let mut st = BoosterState::default();
        let (e, r) = run(&mut st, &mut sl, &BoosterEffectContext::default(), 0);
        assert_eq!(st.nebula_level_life, 1);
        assert_eq!(sl.buff_type[0], 173, "the tier step-down");
        assert_eq!(sl.buff_time[0], 480);
        assert_eq!(e.life_regen, 6);
        assert_eq!(r, BoosterArmResult::NextSlot(1));
    }

    #[test]
    fn s1_e_7g_175_nebula_life_tier_three_regen_eighteen() {
        // No step-down above time 2: level 3 -> lifeRegen += 18.
        let mut sl = slots(&[(175, 100)]);
        let mut st = BoosterState::default();
        let (e, _) = run(&mut st, &mut sl, &BoosterEffectContext::default(), 0);
        assert_eq!(st.nebula_level_life, 3);
        assert_eq!(sl.buff_time[0], 100, "no re-arm above time 2");
        assert_eq!(e.life_regen, 18);
    }

    #[test]
    fn s1_e_7g_176_nebula_mana_sets_the_level_only() {
        // `:10444-10447`: the mana tier writes nebulaLevelMana, no stats.
        let mut sl = slots(&[(177, 2)]);
        let mut st = BoosterState::default();
        let (e, _) = run(&mut st, &mut sl, &BoosterEffectContext::default(), 0);
        assert_eq!(st.nebula_level_mana, 1, "stepped down from tier 2");
        assert_eq!(sl.buff_type[0], 176);
        assert_eq!(sl.buff_time[0], 480);
        assert_eq!(e.life_regen, 0);
        assert_eq!(e.melee_damage, 0.0);
    }

    #[test]
    fn s1_e_7g_179_nebula_damage_quarter_percent_per_level() {
        // `:10453-10461`: tier 3 -> 0.15 * 3 = 0.45 on all four classes.
        let mut sl = slots(&[(181, 50)]);
        let mut st = BoosterState::default();
        let (e, _) = run(&mut st, &mut sl, &BoosterEffectContext::default(), 0);
        assert_eq!(st.nebula_level_damage, 3);
        for v in [e.melee_damage, e.ranged_damage, e.magic_damage, e.minion_damage] {
            assert!((v - 0.45).abs() < 1e-5, "0.15 * 3, got {v}");
        }
    }

    #[test]
    fn s1_e_7g_62_ice_barrier_at_half_life_flags_and_counts_frames() {
        // `:10464-10478`: exactly half -> active; the counter divides by
        // 3, the frame wraps at 12.
        let ctx = BoosterEffectContext { stat_life: 100, stat_life_max2: 200, ..Default::default() };
        let mut sl = slots(&[(62, 100)]);
        let mut st = BoosterState::default();
        let mut frames = Vec::new();
        for _ in 0..36 {
            let (e, r) = run(&mut st, &mut sl, &ctx, 0);
            assert!(e.ice_barrier);
            assert!((e.endurance - 0.25).abs() < 1e-6);
            assert_eq!(r, BoosterArmResult::NextSlot(1));
            frames.push(st.ice_barrier_frame);
        }
        // 3 ticks per frame: frames 0,0,0,1,1,1,...,11,11,11 then wrap 0.
        assert_eq!(frames[2], 0);
        assert_eq!(frames[3], 1);
        assert_eq!(frames[35], 11);
        let (e, _) = run(&mut st, &mut sl, &ctx, 0);
        assert!(e.ice_barrier);
        assert_eq!(st.ice_barrier_frame, 0, "the 12-frame wrap (:10474-10477)");
    }

    #[test]
    fn s1_e_7g_62_above_half_life_deletes_and_rescans() {
        // `:10480-10484`: DelBuff(j); j-- — no flag, no endurance.
        let ctx = BoosterEffectContext { stat_life: 101, stat_life_max2: 200, ..Default::default() };
        let mut sl = slots(&[(62, 100), (2, 100)]);
        let (e, r) = run(&mut BoosterState::default(), &mut sl, &ctx, 0);
        assert!(!e.ice_barrier);
        assert_eq!(e.endurance, 0.0);
        assert_eq!(sl.buff_type[0], 2, "Regen compacted into slot 0");
        assert_eq!(r, BoosterArmResult::NextSlot(0));
    }

    #[test]
    fn s1_e_7g_unclaimed_ids_pass_through() {
        // 94 (Mana Sickness, potions.rs) and 49 (Pygmy, SUMMON_BUFF_ARMS)
        // are NOT this wave's — the chain continues.
        for bt in [94, 49, 2, 144] {
            let mut sl = slots(&[(bt, 100)]);
            let (_, r) = run(
                &mut BoosterState::default(),
                &mut sl,
                &BoosterEffectContext::default(),
                0,
            );
            assert_eq!(r, BoosterArmResult::NotHandled, "buff {bt}");
            assert_eq!(sl.buff_type[0], bt, "the slot is untouched");
        }
    }
}

