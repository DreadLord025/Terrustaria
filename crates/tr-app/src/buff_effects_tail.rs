//! S1-E-7 wave H — the `Player.UpdateBuffs` post-debuff tail
//! (`Player.cs:10318-10808`, 40 buff ids between wave B's Electrified 144
//! `:10313-10317` and wave C's Trusty Cattiva 389 `:10809`).
//!
//! Disjoint-wave design (A owns `:9875-10048`, B `:10049-10317`, C
//! `:10800-11400`, D `:11400-12069` + the pet head): this module is a PURE
//! effect table — `apply_tail_effect_arm` matches one `buffType[j]` id and
//! folds its MISSING writes into a `TailEffects` accumulator, exactly like
//! the C# arm writes into the `Player` fields. The wiring rides the waves'
//! merge step. Nothing here touches `BuffSlots` (`buffs.rs`).
//!
//! Already-ported ids in the range are WITNESS-ONLY here (the dispatch
//! returns `false` so no effect is applied twice):
//! - 94 Mana Sickness (`:10318-10327`) — `player.rs`'s scan arm →
//!   `potions::apply_mana_sickness_arm` (the `DebugOptions.ManaPotionDelay`
//!   mirror `:10320-10323` stays constants-off).
//! - The 19 summon keep-alives 49…386 (`:10486-10808`) —
//!   `player.rs::SUMMON_BUFF_ARMS` + `run_baby_bird_arm`; the
//!   `TAIL_CENSUS` table below pins the census per id.
//! - 62 Ice Barrier's `endurance += 0.25` half (`:10468`) — the `player.rs`
//!   scan arm; this table adds only the MISSING halves (the `iceBarrier`
//!   flag, the frame counter pair, the light request and the
//!   above-half-life `DelBuff(j); j--`).
//!
//! RNG census: the whole block draws NO `Main.rand` — the orb-tier ladders,
//! the nebula step and the ice frames are deterministic. (The beetle-orb
//! VISUAL drift `UpdateArmorSets_Always_Beetle` `:16466-16467` draws two
//! `Next(-100, 101)` per orb per tick but sits OUTSIDE this range — the
//! unported draw family, recorded in ANALYSIS.)
//!
//! Per-tick lifecycle the wiring must honour (`ResetEffects`): `iceBarrier`
//! `:18446`, `beetleOrbs`/`beetleBuff` `:18501-18502`, `solarShields`
//! `:18505`, `manaHeat = 1f` `:18593`, the nebula trio `:18755` — all clear
//! before `UpdateBuffs` (`Player.cs:25491` → `:25566`), so the tier/level
//! fields here are ASSIGNMENTS (last live slot wins), not accumulations.

/// The per-tick context the arms read besides `buffType`/`buffTime`.
/// `whoAmI == Main.myPlayer` collapses to true in the single-player port,
/// matching every other ported `UpdateBuffs` arm.
#[derive(Clone, Copy, Debug, Default)]
pub struct TailEffectContext {
    /// `statLife` / `statLifeMax2` — the Ice Barrier half-life gate
    /// (`Player.cs:10464`, a `double` compare in the C#).
    pub hp: i32,
    pub max_hp2: i32,
    /// `beetleOrbs` — the tier ladders compare against the count the
    /// earlier slots this tick left (`:10346`, `:10411`).
    pub beetle_orbs: i32,
    /// `beetleDefense` / `beetleOffense` — the Beetle set-bonus gates
    /// (`:10366`, `:10433`; `ApplySetBonus_Beetle*` `:16499-16532`).
    pub beetle_defense: bool,
    pub beetle_offense: bool,
    /// `solarShields` / `setSolar` — the Solar Flare ladder + gate
    /// (`:10381`, `:10401`).
    pub solar_shields: i32,
    pub set_solar: bool,
    /// `iceBarrierFrameCounter` / `iceBarrierFrame` — the 3-tick / 12-frame
    /// animation pair (`:10469-10477`); NOT reset per tick (the barrier
    /// keeps its phase across ticks).
    pub ice_barrier_frame_counter: i32,
    pub ice_barrier_frame: i32,
    /// `latestManaPotionDamageBonus` / `latestManaPotionDuration` — the
    /// Mana Heat decay inputs (`:10330-10335`; the producer is
    /// `Player.TryApplyManaHeat` `:52006-52014`, ported as
    /// `player.rs::try_apply_mana_heat`).
    pub latest_mana_potion_damage_bonus: f32,
    pub latest_mana_potion_duration: f32,
}

/// One tick's accumulated writes of the tail block. Flags OR, damage/regen
/// stats ADD per slot exactly like the C# chain; the `Option` setters are
/// assignments (`beetleOrbs = …`, `buffTime[j] = …`, the nebula levels).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TailEffects {
    /// `DelBuff(j)` (`:10350`/`:10369`/`:10385`/`:10404`/`:10415`/`:10436`/
    /// `:10482`).
    pub delete_buff: bool,
    /// The `j--` rescan after the delete — every delete in this block
    /// rescans (the compaction shifts the next buff into slot `j`).
    pub rescan_slot: bool,
    /// The lower-tier purge loop (`:10355-10362`, `:10390-10397`,
    /// `:10420-10427`): `DelBuff(m); m--` for every slot whose type lies in
    /// the inclusive id range — recorded, the wiring walks the slots.
    pub clear_buff_range: Option<(i32, i32)>,
    /// `buffTime[j] = …` — the 5-pin (`:10344`, `:10379`) and the nebula
    /// step's 480 (`:12077`).
    pub buff_time_set: Option<i32>,
    /// The nebula step's `buffType[b]--` (`:12076`).
    pub buff_type_decrement: bool,
    /// `manaHeat += …` (`:10340`) — per-slot add onto the tick's 1f base.
    pub mana_heat_add: f32,
    /// The `latestManaPotionDuration == 0 → 1` guard ASSIGNS the field
    /// (`:10331-10334`) — a real side effect the wiring must copy back.
    pub latest_mana_potion_duration_fix: bool,
    /// `beetleOrbs = num` (`:10365`, `:10430`; zeroed on the gate failure
    /// `:10368`/`:10435`) — assignment, last live slot wins.
    pub beetle_orbs_set: Option<i32>,
    /// `beetleBuff = true` (`:10374`, `:10441`).
    pub beetle_buff: bool,
    /// `solarShields = num` (`:10400`; zeroed `:10403`).
    pub solar_shields_set: Option<i32>,
    /// `nebulaLevelMana/Life/Damage` (`:12072`, post-step `:12075`) —
    /// assignments per slot (the C# `ref` reassigns; last slot wins).
    pub nebula_level_mana: Option<i32>,
    pub nebula_level_life: Option<i32>,
    pub nebula_level_damage: Option<i32>,
    /// `iceBarrier = true` (`:10467`).
    pub ice_barrier: bool,
    /// The post-increment frame pair (`:10469-10477`) — assignments.
    pub ice_barrier_frame_counter: Option<i32>,
    pub ice_barrier_frame: Option<i32>,
    /// `Lighting.AddLight((int)(Center.X/16), (int)(Center.Y/16), …)`
    /// (`:10466`) — RGB only; the tile coords ride the wiring (the light
    /// itself is already ported in `player_lights.rs`, witness there).
    pub ice_barrier_light: Option<[f32; 3]>,
    // Stat adds (per-slot, summed like the C# chain).
    /// `meleeDamage += 0.1 * orbs` (`:10431`) + the nebula damage tier.
    pub melee_damage: f32,
    /// `meleeSpeed += 0.1 * orbs` (`:10432`).
    pub melee_speed: f32,
    /// `rangedDamage`/`magicDamage`/`minionDamage += 0.15 * level`
    /// (`:10458-10460`).
    pub ranged_damage: f32,
    pub magic_damage: f32,
    pub minion_damage: f32,
    /// `lifeRegen += 6 * nebulaLevelLife` (`:10451`).
    pub life_regen: i32,
}

/// `Player.UpdateBuffs_NebulaBuffs` (`Player.cs:12070-12079`): the level is
/// `1 + buffType[b] - baseBuffId`; at `buffTime == 2` with level > 1 the
/// tier steps DOWN (level--, buffType[b]--, buffTime = 480) instead of
/// expiring. Returns the post-step level; the type/time writes ride the
/// accumulator (`buff_type_decrement`, `buff_time_set`).
fn nebula_level_step(e: &mut TailEffects, buff_type: i32, buff_time: i32, base_buff_id: i32) -> i32 {
    let mut level = 1 + buff_type - base_buff_id; // `:12072`
    if buff_time == 2 && level > 1 {
        // `:12073-12078`
        level -= 1;
        e.buff_type_decrement = true;
        e.buff_time_set = Some(480);
    }
    level
}

/// The shared orb-tier ladder of the Beetle Defense (`:10346-10364`),
/// Solar (`:10381-10399`) and Beetle Might (`:10411-10429`) arms: with a
/// DIFFERENT non-zero count already up, a higher count drops THIS slot
/// (`DelBuff(j); j--`), a lower count purges every slot holding the tiers
/// below it (`DelBuff(m); m--` over `base..=base+num-1`).
fn orb_tier_ladder(e: &mut TailEffects, orbs: i32, num: i32, base: i32) {
    if orbs > 0 && orbs != num {
        if orbs > num {
            e.delete_buff = true;
            e.rescan_slot = true;
        } else {
            e.clear_buff_range = Some((base, base + num - 1));
        }
    }
}

/// One arm of the `UpdateBuffs` post-debuff tail (`Player.cs:10318-10808`).
/// Returns `true` when the id is handled HERE (the caller's `else if` chain
/// stops), `false` for ids outside this wave's missing set — including the
/// witness-only ids (94, the 19 summon keep-alives) whose live effect path
/// already exists. The dispatch hook the scan wires per active slot, in
/// slot order.
pub fn apply_tail_effect_arm(
    e: &mut TailEffects,
    buff_type: i32,
    buff_time: i32,
    ctx: &TailEffectContext,
) -> bool {
    match buff_type {
        // `Player.cs:10328-10341` — Mana Heat (396): the latest-potion
        // bonus decays linearly with the buff's remaining time.
        396 => {
            let mut bonus = ctx.latest_mana_potion_damage_bonus; // `:10330`
            let duration = if ctx.latest_mana_potion_duration == 0.0 {
                e.latest_mana_potion_duration_fix = true; // `:10331-10334`
                1.0
            } else {
                ctx.latest_mana_potion_duration
            };
            let ratio = buff_time as f32 / duration; // `:10335`
            bonus *= ratio; // `:10336-10339` (the `0f == 0f` arm is always taken)
            e.mana_heat_add += bonus; // `:10340`
        }
        // `:10342-10376` — Beetle Defense tiers (95-97): the 5-tick pin,
        // the ladder, then the set gate.
        95..=97 => {
            e.buff_time_set = Some(5); // `:10344`
            let num = 1 + buff_type - 95; // `:10345`
            orb_tier_ladder(e, ctx.beetle_orbs, num, 95);
            e.beetle_orbs_set = Some(num); // `:10365`
            if !ctx.beetle_defense {
                // `:10366-10371` — no set bonus: zero the count, drop.
                e.beetle_orbs_set = Some(0);
                e.delete_buff = true;
                e.rescan_slot = true;
            } else {
                e.beetle_buff = true; // `:10374`
            }
        }
        // `:10377-10407` — Solar Flare shields (170-172): the same ladder
        // with NO beetleBuff flag on the keep path.
        170..=172 => {
            e.buff_time_set = Some(5); // `:10379`
            let num = 1 + buff_type - 170; // `:10380`
            orb_tier_ladder(e, ctx.solar_shields, num, 170);
            e.solar_shields_set = Some(num); // `:10400`
            if !ctx.set_solar {
                e.solar_shields_set = Some(0); // `:10403`
                e.delete_buff = true;
                e.rescan_slot = true;
            }
        }
        // `:10408-10443` — Beetle Might tiers (98-100): NO time pin; the
        // damage/speed adds ride the post-ladder count.
        98..=100 => {
            let num = 1 + buff_type - 98; // `:10410`
            orb_tier_ladder(e, ctx.beetle_orbs, num, 98);
            e.beetle_orbs_set = Some(num); // `:10430`
            e.melee_damage += 0.1 * num as f32; // `:10431`
            e.melee_speed += 0.1 * num as f32; // `:10432`
            if !ctx.beetle_offense {
                e.beetle_orbs_set = Some(0); // `:10435`
                e.delete_buff = true;
                e.rescan_slot = true;
            } else {
                e.beetle_buff = true; // `:10441`
            }
        }
        // `:10444-10447` — Nebula Mana tiers (176-178): level only.
        176..=178 => {
            let level = nebula_level_step(e, buff_type, buff_time, 176);
            e.nebula_level_mana = Some(level);
        }
        // `:10448-10452` — Nebula Life tiers (173-175): +6 regen per level.
        173..=175 => {
            let level = nebula_level_step(e, buff_type, buff_time, 173);
            e.nebula_level_life = Some(level);
            e.life_regen += 6 * level; // `:10451`
        }
        // `:10453-10461` — Nebula Damage tiers (179-181): +15% per level
        // to all four classes.
        179..=181 => {
            let level = nebula_level_step(e, buff_type, buff_time, 179);
            e.nebula_level_damage = Some(level);
            let add = 0.15 * level as f32; // `:10456`
            e.melee_damage += add; // `:10457`
            e.ranged_damage += add; // `:10458`
            e.magic_damage += add; // `:10459`
            e.minion_damage += add; // `:10460`
        }
        // `:10462-10485` — Ice Barrier (62): at or below half life the
        // flag + light + frame pair (the endurance +0.25 half already
        // rides the `player.rs` scan arm — NOT duplicated here); above
        // half life the buff dies and the slot rescans.
        62 => {
            if (ctx.hp as f64) <= (ctx.max_hp2 as f64) * 0.5 {
                // `:10464` — the C# casts both to double.
                e.ice_barrier_light = Some([0.1, 0.2, 0.45]); // `:10466`
                e.ice_barrier = true; // `:10467`
                let mut counter = ctx.ice_barrier_frame_counter + 1; // `:10469`
                let mut frame = ctx.ice_barrier_frame;
                if counter > 2 {
                    // `:10470-10478`
                    counter = 0;
                    frame += 1;
                    if frame >= 12 {
                        frame = 0;
                    }
                }
                e.ice_barrier_frame_counter = Some(counter);
                e.ice_barrier_frame = Some(frame);
            } else {
                e.delete_buff = true; // `:10482-10483`
                e.rescan_slot = true;
            }
        }
        // 94 + the 19 summon keep-alives: witness-only, handled elsewhere.
        _ => return false,
    }
    true
}

/// Where a tail id's LIVE effect path lives (the census the wiring reads).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TailOwner {
    /// This module's accumulator carries the id's missing writes.
    Tail,
    /// `player.rs`'s inline buff scan (94 → `potions::apply_mana_sickness_arm`).
    PlayerScan,
    /// `player.rs::SUMMON_BUFF_ARMS` (+ `run_baby_bird_arm` for 216).
    SummonArms,
    /// Split: the endurance half inline, the rest here (Ice Barrier 62).
    SplitIceBarrier,
}

/// The full 40-id census of `Player.cs:10318-10808`, in the C# `else if`
/// chain's verbatim order. One row per id — the per-id witness map: the
/// `Tail`/`SplitIceBarrier` rows are handled by `apply_tail_effect_arm`,
/// the rest MUST return `false` there (their live path already exists —
/// a second grant would double-apply).
pub const TAIL_CENSUS: [(i32, TailOwner); 40] = [
    (94, TailOwner::PlayerScan),   // `:10318` Mana Sickness
    (396, TailOwner::Tail),        // `:10328` Mana Heat
    (95, TailOwner::Tail),         // `:10342` Beetle Defense I
    (96, TailOwner::Tail),         //          Beetle Defense II
    (97, TailOwner::Tail),         //          Beetle Defense III
    (170, TailOwner::Tail),        // `:10377` Solar Shield I
    (171, TailOwner::Tail),        //          Solar Shield II
    (172, TailOwner::Tail),        //          Solar Shield III
    (98, TailOwner::Tail),         // `:10408` Beetle Might I
    (99, TailOwner::Tail),         //          Beetle Might II
    (100, TailOwner::Tail),        //          Beetle Might III
    (176, TailOwner::Tail),        // `:10444` Nebula Mana I
    (177, TailOwner::Tail),        //          Nebula Mana II
    (178, TailOwner::Tail),        //          Nebula Mana III
    (173, TailOwner::Tail),        // `:10448` Nebula Life I
    (174, TailOwner::Tail),        //          Nebula Life II
    (175, TailOwner::Tail),        //          Nebula Life III
    (179, TailOwner::Tail),        // `:10453` Nebula Damage I
    (180, TailOwner::Tail),        //          Nebula Damage II
    (181, TailOwner::Tail),        //          Nebula Damage III
    (62, TailOwner::SplitIceBarrier), // `:10462` Ice Barrier
    (49, TailOwner::SummonArms),   // `:10486` Pygmy
    (83, TailOwner::SummonArms),   // `:10505` Raven
    (64, TailOwner::SummonArms),   // `:10521` Baby Slime
    (125, TailOwner::SummonArms),  // `:10537` Hornet
    (126, TailOwner::SummonArms),  // `:10553` Imp
    (133, TailOwner::SummonArms),  // `:10569` Spider
    (134, TailOwner::SummonArms),  // `:10585` Twins
    (135, TailOwner::SummonArms),  // `:10601` Pirate
    (214, TailOwner::SummonArms),  // `:10617` Vampire Frog
    (139, TailOwner::SummonArms),  // `:10633` Sharknado
    (140, TailOwner::SummonArms),  // `:10649` UFO
    (182, TailOwner::SummonArms),  // `:10665` Stardust Cell
    (213, TailOwner::SummonArms),  // `:10681` Bats of Light (Rainbow Crystal)
    (216, TailOwner::SummonArms),  // `:10697` Baby Bird (+respawn arm)
    (355, TailOwner::SummonArms),  // `:10729` Mushroom Boi
    (385, TailOwner::SummonArms),  // `:10745` Cattiva
    (393, TailOwner::SummonArms),  // `:10761` Clay Pot
    (394, TailOwner::SummonArms),  // `:10777` Forbidden
    (386, TailOwner::SummonArms),  // `:10793` Foxsparks
];

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> TailEffectContext {
        TailEffectContext {
            hp: 100,
            max_hp2: 200,
            beetle_defense: true,
            beetle_offense: true,
            set_solar: true,
            latest_mana_potion_damage_bonus: 0.2,
            latest_mana_potion_duration: 120.0,
            ..Default::default()
        }
    }

    /// The census witness: all 40 ids, one row per C# arm head — the
    /// Tail/Split rows dispatch here, the PlayerScan/SummonArms rows MUST
    /// decline (their live path exists; a true would double-apply).
    #[test]
    fn s1_e_7h_census_40_ids_dispatch_matches_ownership() {
        assert_eq!(TAIL_CENSUS.len(), 40);
        for (id, owner) in TAIL_CENSUS {
            let mut e = TailEffects::default();
            let handled = apply_tail_effect_arm(&mut e, id, 100, &ctx());
            let expected = matches!(owner, TailOwner::Tail | TailOwner::SplitIceBarrier);
            assert_eq!(handled, expected, "buff {id} ({owner:?})");
        }
        // Ids outside the block decline too (B's 144, C's 389).
        for id in [144, 389, 1, 0, 999] {
            assert!(!apply_tail_effect_arm(&mut TailEffects::default(), id, 100, &ctx()));
        }
    }

    /// 396 Mana Heat (`:10328-10341`): bonus × time/duration, per-slot add.
    #[test]
    fn s1_e_7h_buff_396_mana_heat_decays_linearly() {
        let mut e = TailEffects::default();
        assert!(apply_tail_effect_arm(&mut e, 396, 60, &ctx()));
        assert!((e.mana_heat_add - 0.2 * 60.0 / 120.0).abs() < 1e-6);
        assert!(!e.latest_mana_potion_duration_fix);
        // Full time → the whole bonus; two slots add like the chain.
        let mut e2 = TailEffects::default();
        apply_tail_effect_arm(&mut e2, 396, 120, &ctx());
        apply_tail_effect_arm(&mut e2, 396, 120, &ctx());
        assert!((e2.mana_heat_add - 0.4).abs() < 1e-6);
    }

    /// 396's zero-duration guard (`:10331-10334`): the field is ASSIGNED 1
    /// (the fix flag) and the ratio divides by 1.
    #[test]
    fn s1_e_7h_buff_396_zero_duration_guard_assigns_one() {
        let c = TailEffectContext {
            latest_mana_potion_duration: 0.0,
            ..ctx()
        };
        let mut e = TailEffects::default();
        apply_tail_effect_arm(&mut e, 396, 60, &c);
        assert!(e.latest_mana_potion_duration_fix);
        assert!((e.mana_heat_add - 0.2 * 60.0).abs() < 1e-4);
        // A zero bonus adds nothing but still runs the guard.
        let c0 = TailEffectContext {
            latest_mana_potion_damage_bonus: 0.0,
            latest_mana_potion_duration: 0.0,
            ..ctx()
        };
        let mut e0 = TailEffects::default();
        apply_tail_effect_arm(&mut e0, 396, 60, &c0);
        assert!(e0.latest_mana_potion_duration_fix);
        assert_eq!(e0.mana_heat_add, 0.0);
    }

    /// 95-97 Beetle Defense (`:10342-10376`): the 5-pin + tier assignment
    /// + beetleBuff per tier, with the set bonus up and no prior orbs.
    #[test]
    fn s1_e_7h_buffs_95_97_beetle_defense_tiers() {
        for (id, tier) in [(95, 1), (96, 2), (97, 3)] {
            let mut e = TailEffects::default();
            assert!(apply_tail_effect_arm(&mut e, id, 1, &ctx()));
            assert_eq!(e.buff_time_set, Some(5), "buff {id} time pin");
            assert_eq!(e.beetle_orbs_set, Some(tier));
            assert!(e.beetle_buff);
            assert!(!e.delete_buff && e.clear_buff_range.is_none());
            // The defense arm grants NO stat adds.
            assert_eq!((e.melee_damage, e.melee_speed), (0.0, 0.0));
        }
    }

    /// The ladder (`:10346-10364`): higher count up → drop THIS slot;
    /// lower count up → purge the tiers below; same count → no ladder.
    #[test]
    fn s1_e_7h_beetle_defense_ladder_directions() {
        // 3 orbs up, a tier-1 slot arrives: DelBuff(j) + rescan.
        let c = TailEffectContext { beetle_orbs: 3, ..ctx() };
        let mut e = TailEffects::default();
        apply_tail_effect_arm(&mut e, 95, 1, &c);
        assert!(e.delete_buff && e.rescan_slot);
        assert_eq!(e.beetle_orbs_set, Some(1), "the assignment still runs");
        assert!(e.beetle_buff, "the gate passed before the drop order");
        // 1 orb up, a tier-3 slot arrives: purge 95..=97.
        let c1 = TailEffectContext { beetle_orbs: 1, ..ctx() };
        let mut e1 = TailEffects::default();
        apply_tail_effect_arm(&mut e1, 97, 1, &c1);
        assert!(!e1.delete_buff);
        assert_eq!(e1.clear_buff_range, Some((95, 97)));
        assert_eq!(e1.beetle_orbs_set, Some(3));
        // Same count: no ladder writes at all.
        let c2 = TailEffectContext { beetle_orbs: 2, ..ctx() };
        let mut e2 = TailEffects::default();
        apply_tail_effect_arm(&mut e2, 96, 1, &c2);
        assert!(!e2.delete_buff && e2.clear_buff_range.is_none());
        // Zero orbs (first slot of the tick): no ladder.
        let mut e3 = TailEffects::default();
        apply_tail_effect_arm(&mut e3, 96, 1, &ctx());
        assert!(!e3.delete_buff && e3.clear_buff_range.is_none());
    }

    /// The set gate (`:10366-10371`): without `beetleDefense` the count
    /// zeroes, the buff drops and NO beetleBuff — even when the ladder
    /// already wrote a drop (the C# runs both).
    #[test]
    fn s1_e_7h_beetle_defense_gate_failure_zeroes_and_drops() {
        let c = TailEffectContext { beetle_defense: false, ..ctx() };
        let mut e = TailEffects::default();
        apply_tail_effect_arm(&mut e, 96, 1, &c);
        assert_eq!(e.beetle_orbs_set, Some(0));
        assert!(e.delete_buff && e.rescan_slot);
        assert!(!e.beetle_buff);
        assert_eq!(e.buff_time_set, Some(5), "the pin ran before the gate");
    }

    /// 170-172 Solar (`:10377-10407`): the same ladder, no beetleBuff.
    #[test]
    fn s1_e_7h_buffs_170_172_solar_shield_tiers() {
        for (id, tier) in [(170, 1), (171, 2), (172, 3)] {
            let mut e = TailEffects::default();
            assert!(apply_tail_effect_arm(&mut e, id, 1, &ctx()));
            assert_eq!(e.buff_time_set, Some(5));
            assert_eq!(e.solar_shields_set, Some(tier));
            assert!(!e.beetle_buff, "Solar has no beetleBuff write");
            assert!(e.beetle_orbs_set.is_none(), "Solar never touches beetleOrbs");
        }
        // Ladder up: 2 shields, tier-1 slot drops; tier-3 purges 170..=172.
        let c = TailEffectContext { solar_shields: 2, ..ctx() };
        let mut e = TailEffects::default();
        apply_tail_effect_arm(&mut e, 170, 1, &c);
        assert!(e.delete_buff && e.rescan_slot);
        let mut e3 = TailEffects::default();
        apply_tail_effect_arm(&mut e3, 172, 1, &c);
        assert_eq!(e3.clear_buff_range, Some((170, 172)));
        // Gate failure (`:10401-10406`).
        let cg = TailEffectContext { set_solar: false, ..ctx() };
        let mut eg = TailEffects::default();
        apply_tail_effect_arm(&mut eg, 171, 1, &cg);
        assert_eq!(eg.solar_shields_set, Some(0));
        assert!(eg.delete_buff && eg.rescan_slot);
    }

    /// 98-100 Beetle Might (`:10408-10443`): NO time pin; +0.1·orbs melee
    /// damage AND speed per slot; the offense gate mirrors the defense.
    #[test]
    fn s1_e_7h_buffs_98_100_beetle_might_tiers() {
        for (id, tier) in [(98, 1), (99, 2), (100, 3)] {
            let mut e = TailEffects::default();
            assert!(apply_tail_effect_arm(&mut e, id, 300, &ctx()));
            assert_eq!(e.buff_time_set, None, "buff {id}: no 5-pin here");
            assert_eq!(e.beetle_orbs_set, Some(tier));
            assert!((e.melee_damage - 0.1 * tier as f32).abs() < 1e-6);
            assert!((e.melee_speed - 0.1 * tier as f32).abs() < 1e-6);
            assert!(e.beetle_buff);
        }
        // Two slots add per-slot like the chain (the C# re-reads beetleOrbs
        // each arm; the accumulator sums the two writes).
        let mut e2 = TailEffects::default();
        let c1 = TailEffectContext { beetle_orbs: 0, ..ctx() };
        apply_tail_effect_arm(&mut e2, 100, 300, &c1);
        let c3 = TailEffectContext { beetle_orbs: 3, ..ctx() };
        apply_tail_effect_arm(&mut e2, 100, 300, &c3);
        assert!((e2.melee_damage - 0.6).abs() < 1e-6);
        // Gate failure (`:10433-10438`): the stat adds STILL ran (the C#
        // adds before the gate), the count zeroes, the buff drops.
        let cg = TailEffectContext { beetle_offense: false, ..ctx() };
        let mut eg = TailEffects::default();
        apply_tail_effect_arm(&mut eg, 99, 300, &cg);
        assert_eq!(eg.beetle_orbs_set, Some(0));
        assert!(eg.delete_buff && eg.rescan_slot);
        assert!(!eg.beetle_buff);
        assert!((eg.melee_damage - 0.2).abs() < 1e-6, "adds precede the gate");
        // The Might ladder purges its OWN triple, not the defense's.
        let cl = TailEffectContext { beetle_orbs: 1, ..ctx() };
        let mut el = TailEffects::default();
        apply_tail_effect_arm(&mut el, 100, 300, &cl);
        assert_eq!(el.clear_buff_range, Some((98, 100)));
    }

    /// 176-178 Nebula Mana (`:10444-10447` + `:12070-12079`): level only,
    /// no stat writes; the tier-2/3 step-down at buffTime == 2.
    #[test]
    fn s1_e_7h_buffs_176_178_nebula_mana_levels_and_step() {
        for (id, level) in [(176, 1), (177, 2), (178, 3)] {
            let mut e = TailEffects::default();
            assert!(apply_tail_effect_arm(&mut e, id, 480, &ctx()));
            assert_eq!(e.nebula_level_mana, Some(level));
            assert!(!e.buff_type_decrement && e.buff_time_set.is_none());
            assert_eq!(e.life_regen, 0);
            assert_eq!((e.melee_damage, e.ranged_damage), (0.0, 0.0));
        }
        // buffTime == 2, tier 3: level 2, type-- (178→177), time = 480.
        let mut e = TailEffects::default();
        apply_tail_effect_arm(&mut e, 178, 2, &ctx());
        assert_eq!(e.nebula_level_mana, Some(2));
        assert!(e.buff_type_decrement);
        assert_eq!(e.buff_time_set, Some(480));
        // Tier 1 at time 2 does NOT step (level > 1 gate).
        let mut e1 = TailEffects::default();
        apply_tail_effect_arm(&mut e1, 176, 2, &ctx());
        assert_eq!(e1.nebula_level_mana, Some(1));
        assert!(!e1.buff_type_decrement && e1.buff_time_set.is_none());
        // Tier 2 at time 3 does NOT step (time == 2 gate).
        let mut e3 = TailEffects::default();
        apply_tail_effect_arm(&mut e3, 177, 3, &ctx());
        assert_eq!(e3.nebula_level_mana, Some(2));
        assert!(!e3.buff_type_decrement);
    }

    /// 173-175 Nebula Life (`:10448-10452`): +6 lifeRegen per level; the
    /// step-down lowers BOTH the level and the regen add.
    #[test]
    fn s1_e_7h_buffs_173_175_nebula_life_regen() {
        for (id, level) in [(173, 1), (174, 2), (175, 3)] {
            let mut e = TailEffects::default();
            apply_tail_effect_arm(&mut e, id, 480, &ctx());
            assert_eq!(e.nebula_level_life, Some(level));
            assert_eq!(e.life_regen, 6 * level);
        }
        // Tier 3 stepping down at time 2 grants 6*2, not 6*3.
        let mut e = TailEffects::default();
        apply_tail_effect_arm(&mut e, 175, 2, &ctx());
        assert_eq!(e.nebula_level_life, Some(2));
        assert_eq!(e.life_regen, 12);
        assert!(e.buff_type_decrement);
        // Per-slot adds: two tier-1 slots = +12.
        let mut e2 = TailEffects::default();
        apply_tail_effect_arm(&mut e2, 173, 480, &ctx());
        apply_tail_effect_arm(&mut e2, 173, 480, &ctx());
        assert_eq!(e2.life_regen, 12);
        assert_eq!(e2.nebula_level_life, Some(1), "assignment, last wins");
    }

    /// 179-181 Nebula Damage (`:10453-10461`): +0.15·level to all four
    /// classes, per-slot; the step-down lowers the add.
    #[test]
    fn s1_e_7h_buffs_179_181_nebula_damage_all_classes() {
        for (id, level) in [(179, 1), (180, 2), (181, 3)] {
            let mut e = TailEffects::default();
            apply_tail_effect_arm(&mut e, id, 480, &ctx());
            assert_eq!(e.nebula_level_damage, Some(level));
            let want = 0.15 * level as f32;
            assert!((e.melee_damage - want).abs() < 1e-6);
            assert!((e.ranged_damage - want).abs() < 1e-6);
            assert!((e.magic_damage - want).abs() < 1e-6);
            assert!((e.minion_damage - want).abs() < 1e-6);
            assert_eq!(e.melee_speed, 0.0, "no speed add on this arm");
        }
        let mut e = TailEffects::default();
        apply_tail_effect_arm(&mut e, 181, 2, &ctx());
        assert_eq!(e.nebula_level_damage, Some(2));
        assert!((e.magic_damage - 0.30).abs() < 1e-6);
        assert_eq!(e.buff_time_set, Some(480));
    }

    /// 62 Ice Barrier (`:10462-10485`): the half-life `<=` gate, the flag,
    /// the light RGB and the 3-tick / 12-frame counter pair. The endurance
    /// half is NOT here (the inline scan arm owns it — no double grant).
    #[test]
    fn s1_e_7h_buff_62_ice_barrier_flag_light_and_frames() {
        // Exactly half life gates ON (the C# `<=`).
        let mut e = TailEffects::default();
        assert!(apply_tail_effect_arm(&mut e, 62, 600, &ctx()));
        assert!(e.ice_barrier);
        assert_eq!(e.ice_barrier_light, Some([0.1, 0.2, 0.45]));
        assert_eq!(e.ice_barrier_frame_counter, Some(1));
        assert_eq!(e.ice_barrier_frame, Some(0));
        assert!(!e.delete_buff);
        // One above half gates OFF: DelBuff + rescan, no writes.
        let c = TailEffectContext { hp: 101, ..ctx() };
        let mut eo = TailEffects::default();
        apply_tail_effect_arm(&mut eo, 62, 600, &c);
        assert!(eo.delete_buff && eo.rescan_slot);
        assert!(!eo.ice_barrier && eo.ice_barrier_light.is_none());
        assert!(eo.ice_barrier_frame.is_none());
        // The counter rolls every 3 ticks: 2→0 with frame++, 11→0 wrap.
        let c2 = TailEffectContext { ice_barrier_frame_counter: 2, ice_barrier_frame: 5, ..ctx() };
        let mut e2 = TailEffects::default();
        apply_tail_effect_arm(&mut e2, 62, 600, &c2);
        assert_eq!(e2.ice_barrier_frame_counter, Some(0));
        assert_eq!(e2.ice_barrier_frame, Some(6));
        let cw = TailEffectContext { ice_barrier_frame_counter: 2, ice_barrier_frame: 11, ..ctx() };
        let mut ew = TailEffects::default();
        apply_tail_effect_arm(&mut ew, 62, 600, &cw);
        assert_eq!(ew.ice_barrier_frame_counter, Some(0));
        assert_eq!(ew.ice_barrier_frame, Some(0), "12 wraps to 0");
        // Counter 1→2 keeps the frame (the > 2 gate).
        let c1 = TailEffectContext { ice_barrier_frame_counter: 1, ice_barrier_frame: 7, ..ctx() };
        let mut e1 = TailEffects::default();
        apply_tail_effect_arm(&mut e1, 62, 600, &c1);
        assert_eq!(e1.ice_barrier_frame_counter, Some(2));
        assert_eq!(e1.ice_barrier_frame, Some(7));
    }

    /// 94 witness (`:10318-10327`): the dispatch DECLINES — the live arm
    /// is `player.rs` → `potions::apply_mana_sickness_arm`; the accumulator
    /// stays untouched (a grant here would double the reduction).
    #[test]
    fn s1_e_7h_buff_94_is_witness_only() {
        let mut e = TailEffects::default();
        assert!(!apply_tail_effect_arm(&mut e, 94, 300, &ctx()));
        assert_eq!(e, TailEffects::default());
    }

    /// The 19 summon keep-alive witnesses (`:10486-10808`): every id
    /// declines here with NO accumulator write — the live path is
    /// `player.rs::SUMMON_BUFF_ARMS` (+216's respawn arm).
    #[test]
    fn s1_e_7h_summon_keep_alives_are_witness_only() {
        let summon_ids = [
            49, 83, 64, 125, 126, 133, 134, 135, 214, 139, 140, 182, 213, 216, 355, 385, 393,
            394, 386,
        ];
        assert_eq!(summon_ids.len(), 19);
        for id in summon_ids {
            let mut e = TailEffects::default();
            assert!(!apply_tail_effect_arm(&mut e, id, 100, &ctx()), "buff {id}");
            assert_eq!(e, TailEffects::default(), "buff {id} wrote nothing");
            assert_eq!(
                TAIL_CENSUS.iter().find(|(b, _)| *b == id).unwrap().1,
                TailOwner::SummonArms
            );
        }
    }

    /// RNG census witness: no arm takes an RNG parameter — the whole block
    /// is deterministic (the type signature IS the census; this test pins
    /// that a full 40-id sweep with fixed inputs is reproducible).
    #[test]
    fn s1_e_7h_block_is_rng_free_and_reproducible() {
        let run = || {
            let mut out = Vec::new();
            for (id, _) in TAIL_CENSUS {
                let mut e = TailEffects::default();
                apply_tail_effect_arm(&mut e, id, 2, &ctx());
                out.push(e);
            }
            out
        };
        assert_eq!(run(), run());
    }
}
