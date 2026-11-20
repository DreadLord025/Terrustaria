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
//!   `SUMMON_TAIL_WITNESS` table below pins the census per id.
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
