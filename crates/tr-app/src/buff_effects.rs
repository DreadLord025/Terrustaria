//! S1-E-7 wave B — the `Player.UpdateBuffs` DEBUFF block (`Player.cs:10049-10317`).
//!
//! The 47 arms between Chilled (46) and Electrified (144): the control debuffs
//! (chilled/frozen/stoned), the defense debuffs (ichor/broken armor), the
//! set-bonus/aura flags (shadow dodge, palladium, titanium storm, inferno),
//! the whip rows and the potion-station/utility family.
//!
//! Disjoint-wave design (S1-E-7a owns the potion block `:9875-10048` inside
//! `player.rs`): this module is a PURE effect table — `apply_debuff_effect_arm`
//! matches one `buffType[j]` id and folds its writes into a `DebuffEffects`
//! accumulator, exactly like the C# arm writes into the `Player` fields. The
//! `player.rs` `UpdateBuffs` scan wires it later by copying the accumulator
//! onto the live fields (the wiring is wave A's merge step, not this wave's).
//! Nothing here touches `BuffSlots` — the timer engine (`buffs.rs`) is
//! untouched.

/// The per-tick context the arms read besides `buffType`/`buffTime`
/// (`Player.cs:10200` reads `statLifeMax`; `:10218` reads `infernoCounter`).
/// `whoAmI == Main.myPlayer` collapses to true in the single-player port,
/// matching every other ported `UpdateBuffs` arm.
#[derive(Clone, Copy, Debug, Default)]
pub struct DebuffEffectContext {
    /// `statLifeMax` — the base (pre-`statLifeMax2`) life cap for Lifeforce.
    pub stat_life_max: i32,
    /// `infernoCounter` — the Inferno aura's 60-tick damage cadence.
    pub inferno_counter: i32,
}

/// One tick's accumulated writes of the debuff block. Every field mirrors the
/// C# `Player` field the arm writes; flags are OR-ed, stats are ADDED per slot
/// exactly like the C# chain (multiple slots of the same buff stack — e.g.
/// Weak's `statDefense -= 4` per slot is the model at `:11915`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DebuffEffects {
    // Control debuffs.
    pub chilled: bool,          // 46, `Player.cs:10049-10052`
    pub frozen: bool,           // 47, `:10053-10056`
    pub stoned: bool,           // 156, `:10057-10060`
    pub ichor: bool,            // 69, `:10061-10065`
    pub broken_armor: bool,     // 36, `:10066-10069`
    pub honey: bool,            // 48, `:10070-10073`
    pub shadow_dodge: bool,     // 59, `:10074-10077`
    pub ammo_box: bool,         // 93, `:10078-10081`
    pub palladium_regen: bool,  // 58, `:10082-10085`
    pub has_titanium_storm_buff: bool, // 306, `:10086-10089`
    pub chaos_state: bool,      // 88, `:10090-10093`
    pub accelerate_poisons: bool, // 398, `:10094-10097`
    pub blue_lightning: bool,   // 399, `:10098-10101`
    pub red_lightning: bool,    // 400, `:10102-10105`
    pub cool_whip_buff: bool,   // 312, `:10122-10125`
    pub cob_whip_buff: bool,    // 365, `:10126-10129`
    pub life_magnet: bool,      // 105, `:10138-10141`
    pub calmed: bool,           // 106, `:10142-10145`
    pub sonar_potion: bool,     // 122, `:10150-10153`
    pub crate_potion: bool,     // 123, `:10154-10157`
    pub kb_buff: bool,          // 108, `:10164-10167`
    pub ignore_water: bool,     // 109, `:10168-10172`
    pub acc_flipper: bool,      // 109, `:10168-10172`
    pub dead_cells_potion_station: bool, // 366, `:10185-10188`
    pub danger_sense: bool,     // 111, `:10189-10192`
    pub ammo_potion: bool,      // 112, `:10193-10196`
    pub life_force: bool,       // 113, `:10197-10201`
    pub inferno: bool,          // 116, `:10212-10264`
    pub love_struck: bool,      // 119, `:10272-10275`
    pub stinky: bool,           // 120, `:10276-10280`
    pub resist_cold: bool,      // 124, `:10281-10284`
    pub dryad_ward: bool,       // 165, `:10303-10312`
    pub electrified: bool,      // 144, `:10313-10317`
    // Stat adds (per-slot, summed like the C# chain).
    pub stat_defense: i32,      // 69 −15, 215 +5, 165 +8
    pub stat_life_max2: i32,    // 113, `:10200`
    pub summoner_weapon_speed_bonus: f32, // 311/308/314
    pub move_speed: i32,        // 63 `moveSpeed++`, `:10130-10133`
    pub pick_speed: f32,        // 104 −0.25, `:10134-10137`
    pub fishing_skill: i32,     // 121 +15, `:10146-10149`
    pub tile_speed: f32,        // 107 +0.25, `:10158-10163`
    pub wall_speed: f32,        // 107 +0.25, `:10158-10163`
    pub block_range: i32,       // 107 ++, `:10158-10163`
    pub max_minions: i32,       // 110/150 ++, `:10173-10180`
    pub max_turrets: i32,       // 348 ++, `:10181-10184`
    pub endurance: f32,         // 114 +0.1, `:10202-10205`
    pub melee_crit: i32,        // 115 +10, `:10206-10211`; 321 +10, `:9940-9947`
    pub ranged_crit: i32,       // 115 +10; 321 +10
    pub magic_crit: i32,        // 115 +10; 321 +10
    pub melee_damage: f32,      // 117 +0.1, `:10265-10271`
    pub ranged_damage: f32,     // 117 +0.1
    pub magic_damage: f32,      // 117 +0.1
    pub minion_damage: f32,     // 117 +0.1; 321 +0.1, `:9946`
    pub life_regen: i32,        // 165 +6, `:10305`
    pub luck_potion: i32,       // 257 tier, `:10285-10302`
    /// `talkNPC = -1` — Stinky breaks the open dialogue (`:10278`).
    pub talk_npc_reset: bool,
    /// Dryad's Ward thorns add (`:10308-10311`): +0.5 only while `thorns < 1`.
    /// The accumulator carries the CURRENT thorns so the clamp is per-slot,
    /// matching the chain's sequential writes.
    pub thorns: f32,
    /// `Lighting.AddLight` requests the arms make (tile coords come from the
    /// caller — this table only records the RGB, `:10215` / `:10316`).
    pub inferno_light: Option<[f32; 3]>,
    pub electrified_light: Option<[f32; 3]>,
    /// The Inferno aura's damage tick: `infernoCounter % 60 == 0`
    /// (`:10218`). The NPC/PvP scan itself needs the world and rides with
    /// the wiring step (recorded in ANALYSIS as deferred).
    pub inferno_damage_tick: bool,
}

/// The Inferno aura constants (`Player.cs:10216-10219`): the debuff it spreads
/// (323 Hellfire), the 200-px radius, the 20 damage on the 60-tick cadence.
pub const INFERNO_DEBUFF_ID: i32 = 323; // `:10216` `int num3 = 323`
pub const INFERNO_RADIUS: f32 = 200.0; // `:10217` `float num4 = 200f`
pub const INFERNO_DAMAGE: i32 = 20; // `:10219` `int damage = 20`
pub const INFERNO_DEBUFF_TIME: i32 = 120; // `:10231` `AddBuff(num3, 120)`

/// One arm of the `UpdateBuffs` debuff block (`Player.cs:10049-10317`).
/// Returns `true` when the id is handled (the caller's `else if` chain stops),
/// `false` for ids outside this wave's range — the dispatch hook the
/// `player.rs` scan wires per active slot, in slot order.
pub fn apply_debuff_effect_arm(
    e: &mut DebuffEffects,
    buff_type: i32,
    buff_time: i32,
    ctx: &DebuffEffectContext,
) -> bool {
    match buff_type {
        46 => e.chilled = true, // `Player.cs:10049-10052`
        47 => e.frozen = true,  // `:10053-10056`
        156 => e.stoned = true, // `:10057-10060`
        69 => {
            // `:10061-10065` — Ichor: the flag AND −15 defense.
            e.ichor = true;
            e.stat_defense -= 15;
        }
        36 => e.broken_armor = true, // `:10066-10069`
        48 => e.honey = true,        // `:10070-10073`
        59 => e.shadow_dodge = true, // `:10074-10077`
        93 => e.ammo_box = true,     // `:10078-10081`
        58 => e.palladium_regen = true, // `:10082-10085`
        306 => e.has_titanium_storm_buff = true, // `:10086-10089`
        88 => e.chaos_state = true, // `:10090-10093`
        398 => e.accelerate_poisons = true, // `:10094-10097`
        399 => e.blue_lightning = true, // `:10098-10101`
        400 => e.red_lightning = true, // `:10102-10105`
        215 => e.stat_defense += 5, // `:10106-10109` (Bast's defense)
        311 => e.summoner_weapon_speed_bonus += 0.35, // `:10110-10113`
        308 => e.summoner_weapon_speed_bonus += 0.25, // `:10114-10117`
        314 => e.summoner_weapon_speed_bonus += 0.12, // `:10118-10121`
        312 => e.cool_whip_buff = true, // `:10122-10125`
        365 => e.cob_whip_buff = true,  // `:10126-10129`
        63 => e.move_speed += 1, // `:10130-10133` (Well Fed's neighbour: `moveSpeed++`)
        104 => e.pick_speed -= 0.25, // `:10134-10137` (Mining: pickSpeed is a delay)
        105 => e.life_magnet = true, // `:10138-10141`
        106 => e.calmed = true,      // `:10142-10145`
        121 => e.fishing_skill += 15, // `:10146-10149` (Fishing)
        122 => e.sonar_potion = true, // `:10150-10153`
        123 => e.crate_potion = true, // `:10154-10157`
        107 => {
            // `:10158-10163` — Builder: tile/wall speed +0.25, range +1.
            e.tile_speed += 0.25;
            e.wall_speed += 0.25;
            e.block_range += 1;
        }
        108 => e.kb_buff = true, // `:10164-10167` (Titan)
        109 => {
            // `:10168-10172` — Flipper: BOTH flags.
            e.ignore_water = true;
            e.acc_flipper = true;
        }
        110 | 150 => e.max_minions += 1, // `:10173-10180` (Summoning / Bewitched)
        348 => e.max_turrets += 1, // `:10181-10184` (War Table)
        366 => e.dead_cells_potion_station = true, // `:10185-10188`
        111 => e.danger_sense = true, // `:10189-10192`
        112 => e.ammo_potion = true,  // `:10193-10196` (Ammo Reservation)
        113 => {
            // `:10197-10201` — Lifeforce: `statLifeMax2 += statLifeMax / 5 / 20 * 20`
            // (integer division twice, then ×20 — rounds DOWN to a 20-block).
            e.life_force = true;
            e.stat_life_max2 += ctx.stat_life_max / 5 / 20 * 20;
        }
        114 => e.endurance += 0.1, // `:10202-10205`
        115 => {
            // `:10206-10211` — Rage: +10 crit to melee/ranged/magic (NOT minion).
            e.melee_crit += 10;
            e.ranged_crit += 10;
            e.magic_crit += 10;
        }
        116 => {
            // `:10212-10264` — Inferno: the flag, the light and the 60-tick
            // cadence. The NPC aura scan (`:10224-10238`, AddBuff 323/120 +
            // 20 damage on the cadence) and the PvP twin (`:10243-10263`)
            // need the world roster — deferred to the wiring step; the
            // constants ride above so the scan ports verbatim.
            e.inferno = true;
            e.inferno_light = Some([0.65, 0.4, 0.1]); // `:10215`
            e.inferno_damage_tick = ctx.inferno_counter % 60 == 0; // `:10218`
        }
        117 => {
            // `:10265-10271` — Wrath: +10% damage to all four classes.
            e.melee_damage += 0.1;
            e.ranged_damage += 0.1;
            e.magic_damage += 0.1;
            e.minion_damage += 0.1;
        }
        // W4-d115 lane 19 — `:9940-9947`: buff 321 (the brain dodge's
        // BrainOfConfusionBuff cooldown) grants +10 crit to the three
        // non-summon classes + 0.10 minion damage. The arm sits in the
        // C#'s pre-debuff chain (between 158 and 2); the table matches
        // by id so the position is free. The live scan's copy rides
        // `player.rs` (the 59-arm neighbour).
        321 => {
            e.melee_crit += 10;
            e.ranged_crit += 10;
            e.magic_crit += 10;
            e.minion_damage += 0.1;
        }
        119 => e.love_struck = true, // `:10272-10275`
        120 => {
            // `:10276-10280` — Stinky: closes the dialogue AND the luck flag.
            e.talk_npc_reset = true;
            e.stinky = true;
        }
        124 => e.resist_cold = true, // `:10281-10284` (Warmth)
        257 => {
            // `:10285-10302` — Lucky: the tier by REMAINING time
            // (>36000 → 3, >18000 → 2, else 1). SP: the myPlayer gate is true.
            e.luck_potion = if buff_time > 36_000 {
                3
            } else if buff_time > 18_000 {
                2
            } else {
                1
            };
        }
        165 => {
            // `:10303-10312` — Dryad's Ward: regen +6, defense +8, the flag,
            // thorns +0.5 only while under 1 (per-slot, sequential).
            e.life_regen += 6;
            e.stat_defense += 8;
            e.dryad_ward = true;
            if e.thorns < 1.0 {
                e.thorns += 0.5;
            }
        }
        144 => {
            // `:10313-10317` — Electrified: the flag + the light. (The C#
            // passes `(int)Center.X / 16` — int-then-divide, unlike Inferno's
            // `(int)(Center.X / 16f)`; the caller's tile coords differ when
            // Center is negative — noted for the wiring step.)
            e.electrified = true;
            e.electrified_light = Some([0.3, 0.8, 1.1]); // `:10316`
        }
        _ => return false,
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply(id: i32) -> DebuffEffects {
        let mut e = DebuffEffects::default();
        assert!(apply_debuff_effect_arm(&mut e, id, 600, &DebuffEffectContext::default()));
        e
    }

    #[test]
    fn s1_e_7b_control_debuffs_set_their_flags() {
        assert!(apply(46).chilled);
        assert!(apply(47).frozen);
        assert!(apply(156).stoned);
        assert!(apply(36).broken_armor);
    }

    #[test]
    fn s1_e_7b_ichor_flags_and_drops_defense_15() {
        let e = apply(69);
        assert!(e.ichor);
        assert_eq!(e.stat_defense, -15); // `Player.cs:10064`
    }

    #[test]
    fn s1_e_7b_set_bonus_and_whip_flags() {
        assert!(apply(59).shadow_dodge);
        assert!(apply(58).palladium_regen);
        assert!(apply(306).has_titanium_storm_buff);
        assert!(apply(312).cool_whip_buff);
        assert!(apply(365).cob_whip_buff);
        assert!(apply(88).chaos_state);
        assert!(apply(398).accelerate_poisons);
        assert!(apply(399).blue_lightning);
        assert!(apply(400).red_lightning);
    }

    #[test]
    fn s1_e_7b_honey_and_ammo_box_flags() {
        assert!(apply(48).honey);
        assert!(apply(93).ammo_box);
    }

    #[test]
    fn s1_e_7b_bast_defense_and_whip_speed_rows() {
        assert_eq!(apply(215).stat_defense, 5); // `:10108`
        assert_eq!(apply(311).summoner_weapon_speed_bonus, 0.35); // `:10112`
        assert_eq!(apply(308).summoner_weapon_speed_bonus, 0.25); // `:10116`
        assert_eq!(apply(314).summoner_weapon_speed_bonus, 0.12); // `:10120`
    }

    #[test]
    fn s1_e_7b_whip_speed_stacks_across_slots() {
        let mut e = DebuffEffects::default();
        let ctx = DebuffEffectContext::default();
        apply_debuff_effect_arm(&mut e, 311, 600, &ctx);
        apply_debuff_effect_arm(&mut e, 308, 600, &ctx);
        assert!((e.summoner_weapon_speed_bonus - 0.60).abs() < 1e-6);
    }

    #[test]
    fn s1_e_7b_utility_potion_flags() {
        assert!(apply(105).life_magnet);
        assert!(apply(106).calmed);
        assert!(apply(122).sonar_potion);
        assert!(apply(123).crate_potion);
        assert!(apply(108).kb_buff);
        assert!(apply(111).danger_sense);
        assert!(apply(112).ammo_potion);
        assert!(apply(119).love_struck);
        assert!(apply(124).resist_cold);
        assert!(apply(366).dead_cells_potion_station);
    }

    #[test]
    fn s1_e_7b_flipper_sets_both_flags() {
        let e = apply(109); // `:10168-10172`
        assert!(e.ignore_water);
        assert!(e.acc_flipper);
    }

    #[test]
    fn s1_e_7b_builder_speeds_and_range() {
        let e = apply(107); // `:10158-10163`
        assert_eq!(e.tile_speed, 0.25);
        assert_eq!(e.wall_speed, 0.25);
        assert_eq!(e.block_range, 1);
    }

    #[test]
    fn s1_e_7b_move_pick_and_fishing_adds() {
        assert_eq!(apply(63).move_speed, 1); // `:10132` `moveSpeed++`
        assert_eq!(apply(104).pick_speed, -0.25); // `:10136`
        assert_eq!(apply(121).fishing_skill, 15); // `:10148`
    }

    #[test]
    fn s1_e_7b_minion_and_turret_caps() {
        assert_eq!(apply(110).max_minions, 1); // `:10175`
        assert_eq!(apply(150).max_minions, 1); // `:10179`
        assert_eq!(apply(348).max_turrets, 1); // `:10183`
        let mut e = DebuffEffects::default();
        let ctx = DebuffEffectContext::default();
        apply_debuff_effect_arm(&mut e, 110, 600, &ctx);
        apply_debuff_effect_arm(&mut e, 150, 600, &ctx);
        assert_eq!(e.max_minions, 2); // both stack, `:10173-10180`
    }

    #[test]
    fn s1_e_7b_lifeforce_rounds_down_to_20_blocks() {
        let mut e = DebuffEffects::default();
        // `:10200`: statLifeMax / 5 / 20 * 20 — 400 → 400/5=80 → 80/20=4 → 80.
        let ctx = DebuffEffectContext { stat_life_max: 400, ..Default::default() };
        apply_debuff_effect_arm(&mut e, 113, 600, &ctx);
        assert!(e.life_force);
        assert_eq!(e.stat_life_max2, 80);
        // 499 → 99 → 4 → 80 (the double integer division truncates).
        let mut e = DebuffEffects::default();
        let ctx = DebuffEffectContext { stat_life_max: 499, ..Default::default() };
        apply_debuff_effect_arm(&mut e, 113, 600, &ctx);
        assert_eq!(e.stat_life_max2, 80);
        // 500 → 100 → 5 → 100.
        let mut e = DebuffEffects::default();
        let ctx = DebuffEffectContext { stat_life_max: 500, ..Default::default() };
        apply_debuff_effect_arm(&mut e, 113, 600, &ctx);
        assert_eq!(e.stat_life_max2, 100);
    }

    #[test]
    fn s1_e_7b_endurance_rage_wrath_adds() {
        assert_eq!(apply(114).endurance, 0.1); // `:10204`
        let e = apply(115); // `:10208-10210`
        assert_eq!((e.melee_crit, e.ranged_crit, e.magic_crit), (10, 10, 10));
        let e = apply(117); // `:10267-10270`
        assert_eq!(e.melee_damage, 0.1);
        assert_eq!(e.ranged_damage, 0.1);
        assert_eq!(e.magic_damage, 0.1);
        assert_eq!(e.minion_damage, 0.1);
    }

    #[test]
    fn s1_e_7b_inferno_light_cadence_and_constants() {
        let mut e = DebuffEffects::default();
        let ctx = DebuffEffectContext { inferno_counter: 120, ..Default::default() };
        apply_debuff_effect_arm(&mut e, 116, 600, &ctx);
        assert!(e.inferno);
        assert_eq!(e.inferno_light, Some([0.65, 0.4, 0.1])); // `:10215`
        assert!(e.inferno_damage_tick); // 120 % 60 == 0, `:10218`
        let mut e = DebuffEffects::default();
        let ctx = DebuffEffectContext { inferno_counter: 121, ..Default::default() };
        apply_debuff_effect_arm(&mut e, 116, 600, &ctx);
        assert!(!e.inferno_damage_tick);
        // The aura constants (`:10216-10219`, `:10231`).
        assert_eq!(INFERNO_DEBUFF_ID, 323);
        assert_eq!(INFERNO_RADIUS, 200.0);
        assert_eq!(INFERNO_DAMAGE, 20);
        assert_eq!(INFERNO_DEBUFF_TIME, 120);
    }

    #[test]
    fn s1_e_7b_stinky_resets_the_dialogue() {
        let e = apply(120); // `:10278-10279`
        assert!(e.stinky);
        assert!(e.talk_npc_reset);
    }

    #[test]
    fn s1_e_7b_lucky_tiers_follow_remaining_time() {
        let ctx = DebuffEffectContext::default();
        for (t, tier) in [(36_001, 3), (36_000, 2), (18_001, 2), (18_000, 1), (1, 1)] {
            let mut e = DebuffEffects::default();
            apply_debuff_effect_arm(&mut e, 257, t, &ctx);
            assert_eq!(e.luck_potion, tier, "time {t}"); // `:10289-10300`
        }
    }

    #[test]
    fn s1_e_7b_dryads_ward_regen_defense_and_thorns_clamp() {
        let mut e = DebuffEffects::default();
        let ctx = DebuffEffectContext::default();
        apply_debuff_effect_arm(&mut e, 165, 600, &ctx);
        assert!(e.dryad_ward);
        assert_eq!(e.life_regen, 6); // `:10305`
        assert_eq!(e.stat_defense, 8); // `:10306`
        assert_eq!(e.thorns, 0.5); // `:10308-10311`
        // A second slot adds another 0.5 (0.5 < 1), then the clamp holds.
        apply_debuff_effect_arm(&mut e, 165, 600, &ctx);
        assert_eq!(e.thorns, 1.0);
        apply_debuff_effect_arm(&mut e, 165, 600, &ctx);
        assert_eq!(e.thorns, 1.0);
        assert_eq!(e.stat_defense, 24); // the defense keeps stacking
    }

    #[test]
    fn s1_e_7b_electrified_flag_and_light() {
        let e = apply(144); // `:10313-10317`
        assert!(e.electrified);
        assert_eq!(e.electrified_light, Some([0.3, 0.8, 1.1])); // `:10316`
    }

    /// W4-d115 lane 19 — buff 321 (BrainOfConfusionBuff): +10 crit ×3
    /// + 0.10 minion damage (`Player.cs:9940-9947`), per-slot adds.
    #[test]
    fn w4_d115_lane19_the_brain_confusion_buff_grants_crit_and_minion_damage() {
        let e = apply(321);
        assert_eq!(e.melee_crit, 10);
        assert_eq!(e.ranged_crit, 10);
        assert_eq!(e.magic_crit, 10);
        assert!((e.minion_damage - 0.1).abs() < 1e-6);
        // Per-slot stacking like the C# chain (two live slots add twice).
        let mut e2 = DebuffEffects::default();
        let ctx = DebuffEffectContext::default();
        apply_debuff_effect_arm(&mut e2, 321, 240, &ctx);
        apply_debuff_effect_arm(&mut e2, 321, 240, &ctx);
        assert_eq!(e2.melee_crit, 20);
        assert!((e2.minion_damage - 0.2).abs() < 1e-6);
    }

    #[test]
    fn s1_e_7b_out_of_range_ids_are_not_handled() {
        let mut e = DebuffEffects::default();
        let ctx = DebuffEffectContext::default();
        // Wave A's potion ids and the DoT family stay with their owners.
        for id in [1, 2, 3, 5, 14, 20, 24, 39, 94, 0, 401] {
            assert!(!apply_debuff_effect_arm(&mut e, id, 600, &ctx), "id {id}");
        }
        assert_eq!(e, DebuffEffects::default());
    }
}