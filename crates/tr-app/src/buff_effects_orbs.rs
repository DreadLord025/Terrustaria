//! W4-g27 тАФ the KEEP-ONE canonical module for the lunar-orb buff arms of
//! `Player.UpdateBuffs` (`Player.cs:10342-10461`): Beetle Endurance 95-97,
//! Solar Shield 170-172, Beetle Might 98-100 and the Nebula trio 173-181
//! (the `UpdateBuffs_NebulaBuffs` helper `:12070-12079` rides along).
//! 18 ids.
//!
//! Five branches implemented this block; the keep-one decision (the full
//! per-branch diff table + the merger contract live in
//! `docs/research/W4-g27-orb-keep-one.md`):
//! - **d73 `buff_effects_boosters.rs`** тАФ MOST FAITHFUL arm core: direct
//!   `BuffSlots` mutation, the adjusted-`j` double-delete quirk, the Might
//!   stats BEFORE the offense gate, the shared sweep/unset helpers. Kept.
//! - **c17 `buff_effects_potions_sets.rs`** тАФ the counter-seeding correction:
//!   `ResetEffects` zeroes `beetleOrbs`/`solarShields`/`nebulaLevel*` every
//!   frame (`Player.cs:18501-18502`/`:18505`/`:18755`), so the scan state
//!   starts at 0 each tick тАФ d73's "persists across ticks" claim is WRONG.
//!   The zeroing is folded into `OrbScanState::reset_effects` below.
//! - **c16 `buff_effects_orbs.rs`** тАФ the request vocabulary + the purge-band
//!   documentation (the walk deletes the CURRENT slot too and the outer `j`
//!   is NOT decremented afterwards тАФ the shifted-in buff skips its arm this
//!   tick). Its `purge_band`/`delete_buff`/rescan requests are mirrored in
//!   `OrbArmRequests` for the accumulator-shaped mergers (the rescan rides
//!   `next_index` there).
//! - **d74 `buff_effects_tail.rs`** тАФ the per-slot request shape (its
//!   `clear_buff_range`/`buff_time_set`/`buff_type_decrement`); same quirk
//!   gap as c16 (the context counters are caller-seeded, the delete tail
//!   uses the unadjusted `j`).
//! - **c18 `buff_effects_summons.rs`** тАФ NO orb arms (it owns 62 + the
//!   summon keep-alives `:10462-10976`); listed for the census only.
//!
//! Scope: the 18 orb ids ONLY. Mana Heat 396 / Sugar Rush 192 / Brain of
//! Confusion 321 / Ice Barrier 62 / Mana Sickness 94 stay with their wave
//! modules (d73/c16/d74/c17/c18 own them); this module's dispatch returns
//! `NotHandled` for those ids so the caller's `else if` chain continues.
//!
//! Wiring contract (the f12/merger step, NOT this module):
//! 1. Run the timer tick FIRST (`buffs.rs::update_buffs_tick`, the
//!    `:9871-9874` decrement тАФ none of the orb ids is in
//!    `TimeLeftDoesNotDecrease`, `BuffID.cs:47`).
//! 2. Call `OrbScanState::reset_effects()` once per frame (the C#
//!    `ResetEffects` writes), then walk the bar in slot order calling
//!    `apply_orb_arm` per active slot (`buffType > 0 && buffTime > 0`,
//!    `:9867-9870`), advancing by the returned `NextSlot` index (clamped
//!    `>= 0` тАФ the -1 case is the guarded double-delete crash path, where
//!    the C# throws `DelBuff(-1)` before the loop ever advances).
//! 3. Fold `OrbEffects` onto the live fields AFTER the scan: stats ADD
//!    (`+=`), `beetle_buff` ORs; the counters/levels already live in
//!    `OrbScanState` (copy them onto the player fields тАФ they are the C#
//!    fields, zeroed again next frame by step 2).
//!
//! RNG census: ZERO draws in `:10342-10461` + `:12070-12079` тАФ arithmetic
//! and slot bookkeeping only; the block is stream-neutral by construction.

use crate::buffs::{BuffSlots, MAX_BUFFS};

/// The nebula tier window (`UpdateBuffs_NebulaBuffs`, `Player.cs:12077`): a
/// tier that hits `buffTime == 2` steps down and re-arms at 480 ticks.
pub const NEBULA_TIER_TIME: i32 = 480; // `:12077`
/// The beetle/solar orb pin (`Player.cs:10344`/`:10379`): `buffTime[j] = 5`
/// тАФ the armor set re-grants the buff every tick while worn.
pub const ORB_BUFF_TIME: i32 = 5;

/// The per-tick context the arms read besides `buffType`/`buffTime`.
/// `whoAmI == Main.myPlayer` collapses to true in the single-player port,
/// matching every other ported `UpdateBuffs` arm.
#[derive(Clone, Copy, Debug, Default)]
pub struct OrbContext {
    /// `beetleDefense` (`Player.cs:10366`) тАФ `ApplySetBonus_BeetleDefense`
    /// (`:16499-16501`, the Beetle Shell chestpiece; unported until the
    /// armor-set lane lands, so the endurance arms zero + delete exactly
    /// like the C# without the set).
    pub beetle_defense: bool,
    /// `beetleOffense` (`:10433`) тАФ `ApplySetBonus_BeetleDamage`
    /// (`:16529-16532`, the Beetle Scale Mail chestpiece).
    pub beetle_offense: bool,
    /// `setSolar` (`:10401`) тАФ `ApplySetBonus_Solar` (`:16366-16369`).
    pub set_solar: bool,
}

/// The scan's read-write counters тАФ the C# `Player` fields `beetleOrbs` /
/// `solarShields` / `nebulaLevel*`. CORRECTION over d73: `ResetEffects`
/// zeroes ALL of them every frame (`Player.cs:18501` beetleOrbs, `:18505`
/// solarShields, `:18755` the nebula trio), so the scan starts at 0 each
/// tick; within the tick the arms read + assign them sequentially (the
/// wave-B `thorns` pattern). The caller keeps ONE of these per player and
/// calls `reset_effects()` once per frame before the scan.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OrbScanState {
    /// `beetleOrbs` (`:10365`/`:10430`, zeroed `:10368`/`:10435`) тАФ shared
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
    /// The `ResetEffects` writes (`Player.cs:18501`/`:18505`/`:18755`) тАФ
    /// call once per frame BEFORE the buff scan. (`beetleBuff` `:18502`
    /// rides the accumulator's per-tick default instead.)
    pub fn reset_effects(&mut self) {
        *self = Self::default();
    }
}

/// One tick's accumulated stat/flag writes of the orb block. Stats ADD per
/// slot exactly like the C# chain; `beetle_buff` ORs. Slot mutations (the
/// time pins, the tier sweeps, the nebula step-down) run on `BuffSlots`
/// directly тАФ they are NOT recorded here.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OrbEffects {
    /// `beetleBuff = true` (`:10374`/`:10441`) тАФ the set-gated flag; reset
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
/// caller's `else if` chain continues) or the caller's NEXT slot index тАФ
/// the C# `j` after the arm plus the `for` loop's `j++` (`Player.cs:9865`).
/// A plain keep is `j + 1`; a `DelBuff(j); j--` rescan is `j`; the
/// double-delete quirk can be `j - 1`. The caller clamps to `>= 0` (the
/// -1 case is the guarded crash path тАФ the C# `DelBuff(-1)` throws before
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
/// downgrade's `j--`; the upgrade leaves `j` alone тАФ the outer loop's
/// `j++` then SKIPS the buff compacted into slot `j`, the C# quirk).
fn orb_tier_sweep(slots: &mut BuffSlots, j: usize, orbs: i32, num: i32, base: i32) -> i32 {
    let mut jj = j as i32;
    if orbs > 0 && orbs != num {
        if orbs > num {
            slots.del_buff(jj as usize); // `:10350`/`:10385`/`:10415`
            jj -= 1; // `:10351`/`:10386`/`:10416`
        } else {
            // `:10355-10362` / `:10390-10397` / `:10420-10427` тАФ the
            // `for m тАж DelBuff(m); m--` rescan over the INCLUSIVE band.
            let mut m = 0usize;
            while m < MAX_BUFFS {
                let bt = slots.buff_type[m];
                if bt >= base && bt <= base + num - 1 {
                    slots.del_buff(m);
                    // m-- then the loop's m++ тЖТ stay on the shifted-in slot.
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
/// orbs and `DelBuff(j); j--` тАФ using the ALREADY-adjusted `j` from the
/// sweep. C# quirk preserved: after a downgrade delete `j` was decremented,
/// so this second delete hits the slot BEFORE the original. When that `j`
/// is negative the C# `DelBuff(-1)` throws `IndexOutOfRangeException`; the
/// port guards the delete (a documented deviation for that crash path тАФ
/// unreachable in retail play: the counters reset to 0 every frame, so a
/// downgrade needs an earlier KEPT slot whose own set flag was on).
fn orb_unset_tail(slots: &mut BuffSlots, orbs: &mut i32, jj: i32) -> i32 {
    *orbs = 0; // `:10368`/`:10403`/`:10435`
    if jj >= 0 {
        slots.del_buff(jj as usize); // `:10369`/`:10404`/`:10436`
    }
    // The arm's `j--` cancels the loop's `j++` тЖТ the next index is `jj`.
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
    let num = 1 + slots.buff_type[j] - 95; // `:10345` num8 `(byte)(1 + тАж)`
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
/// `else`), no stat adds тАФ the shield count IS the state.
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
/// `meleeDamage`/`meleeSpeed += 0.1 * orbs` тАФ read AFTER the assignment,
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
            // `:10444-10447` тАФ Nebula Mana: the level only (the mana-regen
            // consumer reads `nebulaLevelMana` in its own lane).
            nebula_buffs(slots, &mut s.nebula_level_mana, 176, j);
            j as i32 + 1
        }
        173..=175 => {
            // `:10448-10452` тАФ Nebula Life: level + regen ├Ч6 per level.
            let level = nebula_buffs(slots, &mut s.nebula_level_life, 173, j);
            e.life_regen += 6 * level; // `:10451`
            j as i32 + 1
        }
        179..=181 => {
            // `:10453-10461` тАФ Nebula Damage: one num12 = 0.15 ├Ч level,
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

/// The per-slot request view of one arm тАФ the c16/d74 accumulator shape,
/// derived from the SAME arm core (no second implementation): run the arm
/// on a one-slot scratch bar and diff the before/after. For mergers that
/// keep the request-drain wiring instead of the direct driver.
///
/// Quirk notes the drain MUST honour (c16's documentation, verified here):
/// - `purge_band` is INCLUSIVE of the current slot's id; after draining it
///   the outer `j` is NOT decremented (the shifted-in buff skips its arm).
/// - `delete_buff` uses the arm's ADJUSTED index: after a downgrade sweep
///   the gate-failure delete hits `j - 1` (the slot before the original);
///   `next_index` already encodes the rescan тАФ prefer `next_index` over
///   replaying `delete_buff` at the original `j`.
/// - `buff_time_set`/`buff_type_decrement` are the nebula step-down's
///   in-place writes (no delete, advance normally).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OrbArmRequests {
    /// `buffTime[j] = тАж` тАФ the 5-pin (`:10344`/`:10379`) or the nebula
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
/// would make. (The scratch diff cannot see cross-slot sweep deletes тАФ
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
        // from it (keep +1, rescan 0, double-delete -1) тАФ rebase onto j.
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
        // self-delete or the gate tail) тАФ not via the purge band (whose
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
        // тАФ ResetEffects zeroes beetleOrbs/solarShields/the nebula trio.
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
        // The d73 quirk (c17 collapses it to one rescan тАФ WRONG): downgrade
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
        // Same quirk at j=0: the second delete's index is -1 тАФ the C#
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
        // `:10431-10432` run BEFORE the `:10433` gate тАФ the stats land on
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
        // other waves entirely тАФ the chain continues, the slot untouched.
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
