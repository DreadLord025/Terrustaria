//! W4-f12 -- the `Player.UpdateBuffs` frame WIRING: one slot-ordered pass over
//! the buff bar in the EXACT C# `else if` chain order (`Player.cs:9865-12045`),
//! driving the gathered wave modules + the canonical g27 orb module and
//! folding their accumulators onto the live `Player` fields.
//!
//! KEEP-ONE (`docs/research/W4-g27-orb-keep-one.md`): the 18 lunar-orb ids
//! (95-100 / 170-181) route EXCLUSIVELY to `buff_effects_orbs.rs::apply_orb_arm`
//! (the d73 arm core + c17 zero-seeding). The duplicate orb dispatches in
//! c17/d73/d74 are NOT called; their non-orb ids stay with their modules.
//!
//! OWNERSHIP (no id is applied twice): the live passes already own many ids --
//!   * the inline wave-A match (`player.rs`, runs in `update_with_grapple`)
//!     owns the potion block + scattered debuff/status/fed arms;
//!   * `update_summon_buff_arms` owns the 29 summon keep-alives + the
//!     110/150/348 cap rows + the 216 respawn;
//!   * `update_accessory_buff_arms` owns the accessory/pet/imbue block
//!     (`:10977-11405` + `:12013-12044`) incl. the pets 40/41/152/91/92/341.
//! This pass dispatches ONLY the genuinely-missing ids at their C# positions:
//!   - `:9935-9946` Sugar Rush 192 / Brain buff 321 (d73 stragglers)
//!   - `:10049-10317` the DEBUFF ids the inline match does NOT own (wave B)
//!   - `:10328-10341` Mana Heat 396 (d73)
//!   - `:10342-10461` the 18 orb ids (canonical g27)
//!   - `:10462-10485` Ice Barrier 62 completion (d73 -- the inline endurance
//!     half is folded out of the wave-A match so it is not applied twice)
//!   - `:11406-12012` the late-pet / status / hunger ids the inline match and
//!     the accessory pass do NOT own (wave E), plus the wave-D misc ids wave E
//!     does not own (the DoT flags 70/397/20/24/137/67/68/39/323/44/324; Wet
//!     103 stays with `refresh_wet_state`, Tipsy 25's COMPLETE arm is
//!     inline-owned — gap-1 keep-one, no second grant here).
//! The frame head/tails (`:9857-9864`, `:12046-12067`) ride
//! `buff_effects_frame.rs::run_buff_frame` around this pass.
//!
//! RNG census: this pass draws NOTHING -- the only draw in the wired blocks is
//! Rabid (148)'s `Main.rand` triple, already owned by the live accessory pass.
//!
//! Code-only worker: witnesses WRITTEN, NOT RUN (no cargo/tests/graphify).

use crate::buff_effects::{apply_debuff_effect_arm, DebuffEffectContext, DebuffEffects};
use crate::buff_effects_boosters::{
    apply_booster_effect_arm, BoosterArmResult, BoosterEffectContext, BoosterEffects, BoosterState,
};
use crate::buff_effects_frame::{
    lava_opacity_ladder, luck_potion_diff_tail, update_hunger_buffs_frame, update_starving_state,
    FrameContext, FrameEffects,
};
use crate::buff_effects_misc::{apply_misc_effect_arm, MiscEffectContext, MiscEffects};
use crate::buff_effects_orbs::{apply_orb_arm, OrbArmResult, OrbContext, OrbEffects};
use crate::buff_effects_status::{apply_status_effect_arm, StatusEffectContext, StatusEffects};
use crate::buffs::MAX_BUFFS;
use crate::player::Player;
use crate::projectile::ProjectilePool;
use tr_core::World;

/// The per-tick inputs the wiring reads besides the `Player` itself.
pub struct BuffFrameInputs<'a> {
    /// `Main.dayTime` -- the Werewolf gate (`Player.cs:11895`).
    pub day_time: bool,
    /// `Main.dontStarveWorld` / `Main.remixWorld` -- the frame's seed gates.
    pub dont_starve: bool,
    pub remix: bool,
    /// `WorldGen.SolidTile3` -- Shimmering's box scan (`:11811`).
    pub solid3: &'a dyn Fn(i32, i32) -> bool,
    /// `HeldItem.type` -- Tipsy's Ale branch (`:11925`).
    pub held_item_type: i32,
}

/// The ids the inline wave-A match already owns (the wiring skips them so no
/// effect is applied twice). Verbatim from the `player.rs` effect match.
const INLINE_OWNED: &[i32] = &[
    1, 3, 4, 5, 6, 7, 8, 9, 10, 12, 13, 14, 15, 17, 18, 21, 22, 23, 25, 26, 29,
    30, 31, 32, 33, 35, 46, 47, 59, 69, 94, 104, 105, 107, 114, 120, 121, 122,
    123, 131, 146, 149, 158, 159, 160, 165, 168, 205, 206, 207, 215, 257, 265,
    279, 305, 308, 311, 312, 314, 343, 365, 383,
];

/// The ids the live accessory pass owns (`buff_effects_accessory.rs`).
const ACCESSORY_OWNED: &[i32] = &[
    19, 27, 37, 38, 40, 41, 71, 73, 74, 75, 76, 77, 78, 79, 91, 92, 101, 102,
    148, 152, 155, 190, 191, 200, 201, 202, 217, 218, 219, 258, 259, 260, 261,
    262, 264, 266, 267, 268, 274, 284, 285, 286, 287, 288, 289, 290, 291, 292,
    293, 294, 295, 296, 297, 298, 299, 300, 301, 302, 303, 304, 317, 327, 328,
    329, 330, 331, 341, 345, 349, 351, 352, 354, 356, 371, 372, 373, 376, 382,
];

/// The ids the live summon pass owns (`player.rs::SUMMON_BUFF_ARMS` + the cap
/// rows + the 216 respawn).
const SUMMON_OWNED: &[i32] = &[
    49, 64, 83, 110, 125, 126, 133, 134, 135, 139, 140, 150, 161, 182, 187, 188,
    213, 214, 216, 263, 271, 322, 325, 335, 348, 355, 385, 386, 389, 390, 393,
    394,
];

/// The wave-D misc ids this pass dispatches (the genuinely-missing DoT flag
/// arms). Wet (103) is NOT here -- `player.rs::refresh_wet_state` already owns
/// it (the lava delete + the fire immunities). The misc module's OTHER ids
/// (pets/mounts/fed/flasks/Werewolf/Shimmering/Tipsy/Webbed) are owned by the
/// inline match, the accessory pass or wave E -- not dispatched (no double
/// grant). The DoT flags are read off the buff bar by `derive_dot_inputs`
/// (`holding()`); the flag fields are the C# write's faithful record.
const MISC_DISPATCH: &[i32] = &[
    70, 397, 20, 24, 137, 67, 68, 39, 323, 44, 324,
];

/// The wave-B debuff ids this pass dispatches (the block minus the ids the
/// inline match owns and the 110/150/348 cap rows the summon pass owns).
/// W4-f12 sweep gap 2: Stoned 156 (`:10057-10060`) was in NO dispatch set --
/// the module's arm existed but never fired; it joins here (the flag feeds
/// the `CCed`/control-lock readers through `Player.stoned`).
/// 165 (Dryad's Ward) is NOT here: the inline match owns its defense/thorns
/// halves and the equip pass owns its +6 regen (`accessory_life_regen`), so
/// the debuff module's copy is not dispatched (no double grant); the sweep
/// completes its missing `dryadWard` flag half INLINE (`:10307`).
const DEBUFF_DISPATCH: &[i32] = &[
    36, 48, 58, 63, 88, 93, 106, 108, 109, 111, 112, 113, 115, 116, 117, 119,
    124, 144, 156, 306, 366, 398, 399, 400,
];

/// One tick of the `UpdateBuffs` frame + arm chain, in the C# statement order.
/// Runs at the `world_sim/player.rs` post-tick seam AFTER
/// `update_summon_buff_arms` and BEFORE `update_accessory_buff_arms`. The
/// timer head (`buffs.rs::update_buffs_tick`) already ran in
/// `update_with_grapple`; the orb counters were zeroed there too.
pub fn update_buff_frame(
    p: &mut Player,
    pool: &mut ProjectilePool,
    world: &World,
    inputs: &BuffFrameInputs,
) {
    let frame_ctx = FrameContext {
        soul_drain: p.soul_drain,
        dont_starve: inputs.dont_starve,
        remix: inputs.remix,
        luck_potion: p.luck_state.luck_potion,
        lava_vision: p.lava_vision,
        lava_wet: p.lava_wet,
    };
    // The frame HEAD (`Player.cs:9857-9864`), BEFORE the arm chain: the
    // soulDrain 151 grant + the DontStarve 332 re-grant.
    let mut frame_e = FrameEffects::default();
    if frame_ctx.soul_drain > 0 {
        p.buffs.add_buff(151, 2); // `:9859`
    }
    if frame_ctx.dont_starve {
        update_starving_state(
            &mut p.buffs,
            &mut frame_e,
            frame_ctx.dont_starve,
            frame_ctx.remix,
            true,
        ); // `:9863`
    }

    // The set flags stay false until the armor-set lane lands (the g27
    // contract: `ApplySetBonus_*` unported -- the arms zero + delete like C#).
    let orb_ctx = OrbContext::default();
    let mut orb_e = OrbEffects::default();

    let booster_ctx = BoosterEffectContext {
        stat_life: p.hp,
        stat_life_max2: p.stat_life_max2,
        latest_mana_potion_damage_bonus: p.latest_mana_potion_damage_bonus,
        latest_mana_potion_duration: p.latest_mana_potion_duration,
        ..Default::default()
    };
    let mut booster_e = BoosterEffects::default();
    // The ice frame pair persists across ticks -- seed it from the live fields.
    let mut booster_s = BoosterState {
        ice_barrier_frame_counter: p.ice_barrier_frame_counter,
        ice_barrier_frame: p.ice_barrier_frame,
        ..Default::default()
    };

    let debuff_ctx = DebuffEffectContext {
        stat_life_max: p.max_hp,
        inferno_counter: p.inferno_counter,
    };
    let mut debuff_e = DebuffEffects::default();
    debuff_e.thorns = p.thorns; // the Dryad's Ward clamp is per-slot sequential

    let underworld = (world.height - 200) as f32; // Main.UnderworldLayer
    // Snapshot the owned-projectile counts: the contexts borrow them, and the
    // loop below mutates `p` (a live borrow of `p.owned_projectile_counts`
    // would conflict). The recount ran at the seam (`update_projectile_caches`);
    // the pet spawns this pass makes do not change the counts the arms read
    // (the C# reads the same per-tick recount snapshot).
    let owned_counts = p.owned_projectile_counts.to_vec();
    let status_ctx = StatusEffectContext {
        owned_projectile_counts: &owned_counts,
        day_time: inputs.day_time,
        // `wolfAcc` (`Player.cs:2346`): the Moon Charm / Moon Shell /
        // Celestial Shell grant (`:15182-15189`/`:15134-15143`/`:15402-15411`,
        // W4-f12 gap 3). The equip pass arms it before this frame; the
        // `:25605` clear runs at the merman-derivation tail AFTER the buff
        // scan, so the flag is live here (the Werewolf 28 gate reads it).
        wolf_acc: p.wolf_acc,
        merman: p.merman,
        shimmer_wet: p.shimmer_wet,
        x: p.x,
        y: p.y,
        width: p.width as i32,
        height: p.height as i32,
        underworld_layer: underworld,
        solid3: inputs.solid3,
    };

    let misc_ctx = MiscEffectContext {
        owned_projectile_counts: &owned_counts,
        day_time: inputs.day_time,
        // The misc module's 28 arm is NOT dispatched (the status module owns
        // it), so this flag is read by no dispatched arm — kept faithful to
        // the live field for the module's own witnesses.
        wolf_acc: p.wolf_acc,
        merman: p.merman,
        held_item_type: inputs.held_item_type,
        armor_head: armor_slot(p, 0),
        armor_body: armor_slot(p, 1),
        armor_legs: armor_slot(p, 2),
        lava_wet: p.lava_wet,
        shimmer_wet: p.shimmer_wet,
        x: p.x,
        y: p.y,
        width: p.width as i32,
        height: p.height as i32,
        direction: p.facing,
        vy: p.vy,
        underworld_layer: underworld,
        solid3: inputs.solid3,
    };
    // This pass draws nothing (Rabid 148 is accessory-owned and skipped), so a
    // scratch stream is safe -- the misc module's only draw is that arm.
    let mut misc_rng = tr_core::UnifiedRandom::default();

    let mut j = 0usize;
    while j < MAX_BUFFS {
        let buff = p.buffs.buff_type[j];
        if buff <= 0 || p.buffs.buff_time[j] <= 0 {
            j += 1; // `:9867-9870`
            continue;
        }
        let time = p.buffs.buff_time[j];

        // Ids the live passes own -- the chain leaves the slot alone here.
        if INLINE_OWNED.contains(&buff)
            || ACCESSORY_OWNED.contains(&buff)
            || SUMMON_OWNED.contains(&buff)
        {
            j += 1;
            continue;
        }

        let mut next = j as i32 + 1;
        if is_orb_id(buff) {
            // `:10342-10461` -- the canonical g27 orb module (keep-one).
            if let OrbArmResult::NextSlot(n) =
                apply_orb_arm(&mut orb_e, &mut p.orb_scan, &mut p.buffs, &orb_ctx, j)
            {
                next = n;
            }
        } else if matches!(buff, 192 | 321 | 396 | 62) {
            // `:9935-9946` / `:10328-10341` / `:10462-10485` -- d73 non-orb ids.
            if let BoosterArmResult::NextSlot(n) = apply_booster_effect_arm(
                &mut booster_e,
                &mut booster_s,
                &mut p.buffs,
                &booster_ctx,
                j,
            ) {
                next = n;
            }
        } else if DEBUFF_DISPATCH.contains(&buff)
            && apply_debuff_effect_arm(&mut debuff_e, buff, time, &debuff_ctx)
        {
            // `:10049-10317` -- the wave-B debuff ids the inline match lacks.
            // 110/150/348 are NOT here: the live summon pass owns the cap rows
            // (`update_summon_buff_arms`), so the debuff module's copy is not
            // dispatched (no double grant).
        } else {
            // The status/misc blocks: per-slot accumulators (the arms delete /
            // pin the CURRENT slot, so the fold runs immediately).
            let mut status_e = StatusEffects::default();
            if apply_status_effect_arm(&mut status_e, &status_ctx, buff, time) {
                next = apply_status_slot(p, pool, &status_e, j);
            } else if MISC_DISPATCH.contains(&buff) {
                // The wave-D misc ids wave E / the inline match / the accessory
                // pass do NOT own: the DoT flag arms + Wet (103). The misc
                // module owns many other ids (pets/mounts/fed/flasks/Werewolf/
                // Shimmering/Tipsy) that are already live elsewhere -- they are
                // NOT dispatched here (no double grant).
                let mut misc_e = MiscEffects::default();
                if apply_misc_effect_arm(&mut misc_e, &misc_ctx, &mut misc_rng, buff, time) {
                    next = apply_misc_slot(p, pool, &misc_e, j);
                }
            }
        }
        j = next.max(0) as usize;
    }

    // ---- fold the tick-level accumulators onto the live fields -----------
    fold_orb(p, &orb_e);
    fold_booster(p, &booster_e, &booster_s);
    fold_debuff(p, &debuff_e);

    // The frame TAILS (`:12046-12067`), AFTER the arm chain: the per-tick
    // hunger ladder, the luckPotion diff, the lavaOpacity fade. The luck diff
    // reads the POST-chain luckPotion (the `:10285-10302` arm may have set it).
    update_hunger_buffs_frame(&mut p.buffs, &mut frame_e, inputs.dont_starve, inputs.remix);
    let luck_ctx = FrameContext {
        luck_potion: p.luck_state.luck_potion,
        ..frame_ctx
    };
    luck_potion_diff_tail(&luck_ctx, &mut p.frame_state, &mut frame_e);
    lava_opacity_ladder(&frame_ctx, &mut p.frame_state);
    p.pending_frame_emotes.extend(frame_e.emotes.iter().copied());
    p.pending_hungry_sounds += frame_e.hungry_sounds;
    if frame_e.luck_needs_sync {
        p.luck_needs_sync = true;
    }
}

fn is_orb_id(buff: i32) -> bool {
    matches!(buff, 95..=97 | 170..=172 | 98..=100 | 176..=178 | 173..=175 | 179..=181)
}

/// The armor texture-slot id the fairy override reads (`head`/`body`/`legs`).
fn armor_slot(p: &Player, idx: usize) -> i32 {
    let f: fn(&crate::itemdata::ItemDef) -> i16 = match idx {
        0 => |d| d.head_slot,
        1 => |d| d.body_slot,
        _ => |d| d.leg_slot,
    };
    crate::registry::get(p.equipped[idx]).map_or(-1, f) as i32
}

/// Apply one wave-E status arm's slot writes + fold its accumulator onto the
/// live fields. Returns the next slot index (the C# `j` after the arm + the
/// loop's `j++`): a Werewolf delete rescans (`j--`), a Shimmering delete
/// `continue`s (advance). The pet spawns ride the pool seam.
fn apply_status_slot(p: &mut Player, pool: &mut ProjectilePool, e: &StatusEffects, j: usize) -> i32 {
    if let Some(t) = e.buff_time_set {
        p.buffs.buff_time[j] = t;
    }
    // The pet flags are per-pet-projectile bools (`Player.pet_flags`).
    for &flag in &e.pet_flags {
        if let Some(proj) = status_pet_proj(flag) {
            p.pet_flags[usize::from(proj)] = true;
        }
    }
    // The pet spawn requests (`:11417` etc.) -- only when the owned count is 0
    // (the arm already gated). Spawn at the entity centre, damage 0, kb 0.
    for &proj in &e.pet_spawns {
        let cx = p.x + p.width * 0.5;
        let cy = p.y + p.height * 0.5;
        pool.spawn(proj, 0, glam::Vec2::new(cx, cy), glam::Vec2::ZERO, 0, 0.0, false);
    }
    if e.crystal_leaf_dedupe {
        // `:11663-11673`: kill every owned 226 beyond the first.
        let mut seen = false;
        for i in 0..pool.slots().len() {
            let (active, owner, kind) = {
                let s = &pool.slots()[i];
                (s.active, s.owner, s.kind)
            };
            if active && owner == 0 && kind == 226 {
                if seen {
                    pool.kill(i);
                }
                seen = true;
            }
        }
    }
    if let Some(fs) = e.fall_start_set {
        p.fall_start = fs; // `:11788` Shimmering's fallStart reset
    }
    // The status flags.
    p.blackout |= e.blackout;
    p.dripping_sparkle_slime |= e.dripping_sparkle_slime;
    p.shimmering |= e.shimmering;
    p.frozen |= e.frozen;
    p.headcovered |= e.headcovered;
    p.bleed |= e.bleed;
    p.vortex_debuff |= e.vortex_debuff;
    p.wind_pushed |= e.wind_pushed;
    p.withered_armor |= e.withered_armor;
    p.withered_weapon |= e.withered_weapon;
    p.slow_ogre_spit |= e.slow_ogre_spit;
    p.parry_damage_buff |= e.parry_damage_buff;
    p.moon_leech |= e.moon_leech;
    p.defended_by_paladin |= e.defended_by_paladin;
    p.were_wolf |= e.were_wolf;
    p.hungry |= e.hungry;
    p.starving |= e.starving;
    p.hearty_meal |= e.hearty_meal;
    // The stat adds (per-slot, summed like the C# chain).
    p.stat_defense += e.stat_defense;
    p.melee_crit += e.melee_crit;
    p.ranged_crit += e.ranged_crit;
    p.magic_crit += e.magic_crit;
    p.melee_damage += e.melee_damage;
    p.ranged_damage += e.ranged_damage;
    p.magic_damage += e.magic_damage;
    p.minion_damage += e.minion_damage;
    p.minion_kb += e.minion_kb;
    p.melee_speed_bonus += e.melee_speed;
    p.move_speed += e.move_speed;
    p.pick_speed_bonus -= e.pick_speed; // the positive-bonus convention
    p.buff_life_regen += e.life_regen;
    // The slot delete + rescan semantics.
    if e.delete_buff {
        p.buffs.del_buff(j);
        if e.rescan_slot {
            return j as i32; // Werewolf's `j--` + the loop's `j++` -> rescan
        }
        return j as i32 + 1; // Shimmering's `continue` -> advance
    }
    j as i32 + 1
}

/// The wave-E pet flag name -> pet projectile id (the `STATUS_PET_ARMS` rows).
fn status_pet_proj(flag: &str) -> Option<u16> {
    crate::buff_effects_status::STATUS_PET_ARMS
        .iter()
        .find(|a| a.flag == flag)
        .map(|a| a.proj)
        .or(match flag {
            "crystalLeaf" => Some(226),
            _ => None,
        })
}

/// Apply one wave-D misc arm's slot writes + fold its accumulator. This pass
/// dispatches ONLY the DoT flag arms (`MISC_DISPATCH`), so the fold covers
/// exactly those writes. Returns the next slot index (no delete in this set).
fn apply_misc_slot(p: &mut Player, _pool: &mut ProjectilePool, e: &MiscEffects, j: usize) -> i32 {
    if let Some(t) = e.buff_time_set {
        p.buffs.buff_time[j] = t;
    }
    // The DoT / status flags (`:11707-11783`).
    p.venom |= e.venom;
    p.chlorophyte_spore |= e.chlorophyte_spore;
    p.poisoned |= e.poisoned;
    p.on_fire |= e.on_fire;
    p.dripping |= e.dripping;
    p.dripping_slime |= e.dripping_slime;
    p.burned |= e.burned;
    p.suffocating |= e.suffocating;
    p.on_fire2 |= e.on_fire2;
    p.on_fire3 |= e.on_fire3;
    p.on_frost_burn |= e.on_frost_burn;
    p.on_frost_burn2 |= e.on_frost_burn2;
    if e.delete_buff {
        p.buffs.del_buff(j);
        return j as i32 + 1;
    }
    j as i32 + 1
}

fn fold_orb(p: &mut Player, e: &OrbEffects) {
    p.beetle_buff |= e.beetle_buff;
    p.melee_damage += e.melee_damage;
    p.melee_speed_bonus += e.melee_speed;
    p.ranged_damage += e.ranged_damage;
    p.magic_damage += e.magic_damage;
    p.minion_damage += e.minion_damage;
    p.buff_life_regen += e.life_regen;
}

fn fold_booster(p: &mut Player, e: &BoosterEffects, s: &BoosterState) {
    p.beetle_buff |= e.beetle_buff;
    p.ice_barrier |= e.ice_barrier;
    p.mana_heat += e.mana_heat;
    p.melee_damage += e.melee_damage;
    p.melee_speed_bonus += e.melee_speed;
    p.ranged_damage += e.ranged_damage;
    p.magic_damage += e.magic_damage;
    p.minion_damage += e.minion_damage;
    p.buff_life_regen += e.life_regen;
    p.endurance += e.endurance;
    p.melee_crit += e.melee_crit;
    p.ranged_crit += e.ranged_crit;
    p.magic_crit += e.magic_crit;
    p.move_speed += e.move_speed;
    p.pick_speed_bonus -= e.pick_speed; // the positive-bonus convention
    if let Some(d) = e.latest_mana_potion_duration_set {
        p.latest_mana_potion_duration = d;
    }
    p.ice_barrier_frame_counter = s.ice_barrier_frame_counter;
    p.ice_barrier_frame = s.ice_barrier_frame;
}

fn fold_debuff(p: &mut Player, e: &DebuffEffects) {
    p.chilled |= e.chilled;
    p.frozen |= e.frozen;
    p.stoned |= e.stoned;
    p.ichor |= e.ichor;
    p.broken_armor |= e.broken_armor;
    p.honey |= e.honey;
    p.shadow_dodge |= e.shadow_dodge;
    p.ammo_box |= e.ammo_box;
    p.palladium_regen |= e.palladium_regen;
    p.has_titanium_storm_buff |= e.has_titanium_storm_buff;
    p.chaos_state |= e.chaos_state;
    p.accelerate_poisons |= e.accelerate_poisons;
    p.blue_lightning |= e.blue_lightning;
    p.red_lightning |= e.red_lightning;
    p.cool_whip_buff |= e.cool_whip_buff;
    p.cob_whip_buff |= e.cob_whip_buff;
    p.life_magnet |= e.life_magnet;
    p.calmed |= e.calmed;
    p.sonar_potion |= e.sonar_potion;
    p.crate_potion |= e.crate_potion;
    p.kb_buff |= e.kb_buff;
    p.ignore_water |= e.ignore_water;
    p.acc_flipper |= e.acc_flipper;
    p.buffs.dead_cells_potion_station |= e.dead_cells_potion_station;
    p.danger_sense |= e.danger_sense;
    p.ammo_potion |= e.ammo_potion;
    p.life_force |= e.life_force;
    p.inferno |= e.inferno;
    p.love_struck |= e.love_struck;
    p.luck_state.stinky |= e.stinky;
    p.buffs.resist_cold |= e.resist_cold;
    p.dryad_ward |= e.dryad_ward;
    p.electrified |= e.electrified;
    // The stat adds.
    p.stat_defense += e.stat_defense;
    p.stat_life_max2 += e.stat_life_max2;
    p.summoner_weapon_speed_bonus += e.summoner_weapon_speed_bonus;
    p.move_speed += e.move_speed as f32;
    p.pick_speed_bonus -= e.pick_speed;
    p.fishing_skill += e.fishing_skill;
    p.tile_speed_bonus += e.tile_speed;
    p.wall_speed_bonus += e.wall_speed;
    p.block_range += e.block_range;
    p.max_minions += e.max_minions;
    p.max_turrets += e.max_turrets;
    p.endurance += e.endurance;
    p.melee_crit += e.melee_crit;
    p.ranged_crit += e.ranged_crit;
    p.magic_crit += e.magic_crit;
    p.melee_damage += e.melee_damage;
    p.ranged_damage += e.ranged_damage;
    p.magic_damage += e.magic_damage;
    p.minion_damage += e.minion_damage;
    p.buff_life_regen += e.life_regen;
    p.thorns = e.thorns;
    if e.luck_potion > 0 {
        p.luck_state.luck_potion = e.luck_potion as u8;
    }
    // The Inferno/Electrified lights ride the entity-light feed (player_lights
    // already reads the buffs); the aura damage scan is deferred (world roster).
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::World;

    fn world() -> World {
        World::new("t".into(), 64, 800)
    }

    fn player() -> Player {
        Player::spawn(&world())
    }

    fn inputs() -> BuffFrameInputs<'static> {
        static NO_SOLID: fn(i32, i32) -> bool = |_, _| false;
        BuffFrameInputs {
            day_time: true,
            dont_starve: false,
            remix: false,
            solid3: &NO_SOLID,
            held_item_type: 0,
        }
    }

    fn run(p: &mut Player, pool: &mut ProjectilePool) {
        let w = world();
        update_buff_frame(p, pool, &w, &inputs());
    }

    // ---- the keep-one orb routing ----------------------------------------

    #[test]
    fn w4_f12_orb_ids_route_to_the_canonical_module() {
        // The 18 orb ids are dispatched to `apply_orb_arm` (keep-one): with
        // the set flags false (the armor-set lane unported) the arms zero +
        // delete exactly like the C# without the set (`:10366-10371`).
        let mut p = player();
        let mut pool = ProjectilePool::default();
        p.buffs.add_buff(95, 5);
        run(&mut p, &mut pool);
        assert_eq!(p.orb_scan.beetle_orbs, 0, "the gate failure zeroes the orbs");
        assert!(p.buffs.find_buff_index(95).is_none(), "the buff is deleted");
        assert!(!p.beetle_buff, "no set -> no beetleBuff flag");
    }

    #[test]
    fn w4_f12_final_g27_the_orb_set_is_exclusive_and_canonical() {
        // The W4-f12 final g27 cross-verify (origin/task/W4-g27-orb-keep-one-prep
        // tip 1cd7ac367): `buff_effects_orbs.rs` is BYTE-IDENTICAL between
        // the two branches (`git diff HEAD origin/... -- buff_effects_orbs.rs`
        // empty), so `apply_orb_arm`'s shape is still canonical — the d73
        // arm core (adjusted-j double-delete quirk, Might stats before the
        // gate) + the c17 ResetEffects zero-seeding + the c16/d74 request
        // view. This witness pins the ROUTING exclusivity on our side: the
        // 18 orb ids are exactly the six 3-id bands, none of them sits in
        // an ownership set (so the chain reaches `is_orb_id`), and no
        // non-orb id sneaks into the bands.
        let orb_ids: Vec<i32> = (95..=100).chain(170..=181).collect();
        assert_eq!(orb_ids.len(), 18);
        for &b in &orb_ids {
            assert!(is_orb_id(b), "orb {b} routes to apply_orb_arm");
            assert!(!INLINE_OWNED.contains(&b), "orb {b} not inline-owned");
            assert!(!ACCESSORY_OWNED.contains(&b), "orb {b} not accessory-owned");
            assert!(!SUMMON_OWNED.contains(&b), "orb {b} not summon-owned");
            assert!(!DEBUFF_DISPATCH.contains(&b), "orb {b} not debuff-dispatched");
            assert!(!MISC_DISPATCH.contains(&b), "orb {b} not misc-dispatched");
            // The d73 non-orb lane (192/321/396/62) never overlaps the orbs.
            assert!(!matches!(b, 192 | 321 | 396 | 62));
        }
        // The bands' boundaries: the ids just outside are NOT orbs.
        for b in [94, 101, 169, 182] {
            assert!(!is_orb_id(b), "{b} is outside the orb bands");
        }
        // The nebula-twin contract (research/W4-f12-sweep-census.md): the
        // ONE writer is `nebula_buffs` inside `apply_orb_arm` writing
        // `OrbScanState.nebula_level_mana` (base 176); the f49 regen read
        // side binds to that field at the merge — no second dispatch here.
        let mut p = player();
        let mut pool = ProjectilePool::default();
        p.buffs.buff_type[0] = 177; // Nebula Mana tier 2
        p.buffs.buff_time[0] = 3600;
        run(&mut p, &mut pool);
        assert_eq!(p.orb_scan.nebula_level_mana, 2, "level = 1 + 177 - 176");
        assert_eq!(p.buffs.buff_type[0], 177, "no step-down above time 2");
    }

    #[test]
    fn w4_f12_nebula_life_steps_down_and_adds_regen() {
        // `:10448-10452` + `:12070-12079`: tier 2 at time 2 steps to tier 1,
        // re-arms at 480, +6 regen (the canonical module's arm core).
        let mut p = player();
        let mut pool = ProjectilePool::default();
        p.buffs.buff_type[0] = 174;
        p.buffs.buff_time[0] = 2;
        run(&mut p, &mut pool);
        assert_eq!(p.buffs.buff_type[0], 173, "the tier step-down");
        assert_eq!(p.buffs.buff_time[0], 480);
        assert_eq!(p.orb_scan.nebula_level_life, 1);
        assert_eq!(p.buff_life_regen, 6, "the Nebula Life regen add");
    }

    // ---- the booster stragglers / Mana Heat / Ice Barrier ----------------

    #[test]
    fn w4_f12_sugar_rush_pick_and_move() {
        // `:9935-9939`: pickSpeed -= 0.2 (the positive-bonus convention +0.2),
        // moveSpeed += 0.2.
        let mut p = player();
        let mut pool = ProjectilePool::default();
        let base_move = p.move_speed;
        p.buffs.add_buff(192, 100);
        run(&mut p, &mut pool);
        assert!((p.pick_speed_bonus - 0.2).abs() < 1e-6);
        assert!((p.move_speed - (base_move + 0.2)).abs() < 1e-6);
    }

    #[test]
    fn w4_f12_brain_buff_crits_and_minion_damage() {
        // `:9940-9947`: +10 melee/ranged/magic crit, minionDamage += 0.1.
        let mut p = player();
        let mut pool = ProjectilePool::default();
        let base_minion = p.minion_damage;
        p.buffs.add_buff(321, 240);
        run(&mut p, &mut pool);
        assert_eq!(p.melee_crit, 4 + 10);
        assert_eq!(p.ranged_crit, 4 + 10);
        assert_eq!(p.magic_crit, 4 + 10);
        assert!((p.minion_damage - (base_minion + 0.1)).abs() < 1e-6);
    }

    #[test]
    fn w4_f12_mana_heat_decays_the_bonus() {
        // `:10328-10341`: bonus 0.2, duration 120, time 60 -> manaHeat += 0.1.
        let mut p = player();
        let mut pool = ProjectilePool::default();
        p.latest_mana_potion_damage_bonus = 0.2;
        p.latest_mana_potion_duration = 120.0;
        p.buffs.buff_type[0] = 396;
        p.buffs.buff_time[0] = 60;
        run(&mut p, &mut pool);
        assert!((p.mana_heat - 1.1).abs() < 1e-6, "the 1f base + 0.1");
    }

    #[test]
    fn w4_f12_ice_barrier_whole_arm_below_half_life() {
        // `:10462-10485`: at/below half life the flag + endurance +0.25 + the
        // frame advance; the inline endurance half is folded out (keep-one).
        let mut p = player();
        let mut pool = ProjectilePool::default();
        p.stat_life_max2 = p.max_hp; // the ResetEffects rebase (the direct call skips it)
        p.hp = p.max_hp / 2;
        p.buffs.add_buff(62, 600);
        run(&mut p, &mut pool);
        assert!(p.ice_barrier, "the flag arms");
        assert!((p.endurance - 0.25).abs() < 1e-6, "the endurance half, ONCE");
        assert_eq!(p.ice_barrier_frame_counter, 1, "the frame counter advances");
    }

    #[test]
    fn w4_f12_ice_barrier_above_half_life_deletes_and_rescans() {
        // `:10482-10483`: above half life DelBuff(j); j-- (the slot rescans).
        let mut p = player();
        let mut pool = ProjectilePool::default();
        p.stat_life_max2 = p.max_hp; // the ResetEffects rebase (the direct call skips it)
        p.hp = p.max_hp;
        p.buffs.add_buff(62, 600);
        run(&mut p, &mut pool);
        assert!(p.buffs.find_buff_index(62).is_none(), "the buff dies");
        assert!(!p.ice_barrier);
    }

    // ---- the debuff block -------------------------------------------------

    #[test]
    fn w4_f12_lifeforce_raises_stat_life_max2() {
        // `:10197-10201`: statLifeMax2 += statLifeMax / 5 / 20 * 20.
        let mut p = player();
        let mut pool = ProjectilePool::default();
        let base = p.stat_life_max2;
        p.buffs.add_buff(113, 600);
        run(&mut p, &mut pool);
        assert!(p.life_force);
        assert_eq!(p.stat_life_max2, base + p.max_hp / 5 / 20 * 20);
    }

    #[test]
    fn w4_f12_rage_adds_crit_to_three_classes() {
        // `:10206-10211`: +10 melee/ranged/magic crit (NOT minion).
        let mut p = player();
        let mut pool = ProjectilePool::default();
        p.buffs.add_buff(115, 600);
        run(&mut p, &mut pool);
        assert_eq!(p.melee_crit, 4 + 10);
        assert_eq!(p.ranged_crit, 4 + 10);
        assert_eq!(p.magic_crit, 4 + 10);
    }

    /// W4-f12 sweep gap 2 — Stoned (156, `:10057-10060`) joins
    /// DEBUFF_DISPATCH: the module's arm existed but no dispatch set carried
    /// the id, so `Player.stoned` never armed (the `CCed` readers only saw
    /// the raw slot). One grant: the inline match has no 156 arm.
    #[test]
    fn w4_f12_sweep_stoned_dispatches_through_the_debuff_lane() {
        let mut p = player();
        let mut pool = ProjectilePool::default();
        p.buffs.add_buff(156, 600);
        run(&mut p, &mut pool);
        assert!(p.stoned, "the flag arms through the wave-B fold");
        assert!(p.buffs.find_buff_index(156).is_some(), "the buff stays");
    }

    // ---- the status block -------------------------------------------------

    #[test]
    fn w4_f12_hungry_subtracts_the_fed_block() {
        // `:11979-11993`: Hungry (333) subtracts the fed-tier stat block.
        let mut p = player();
        let mut pool = ProjectilePool::default();
        p.buffs.add_buff(333, 600);
        run(&mut p, &mut pool);
        assert!(p.hungry);
        assert_eq!(p.stat_defense, -2);
        assert_eq!(p.melee_crit, 4 - 2);
    }

    #[test]
    fn w4_f12_werewolf_day_deletes_and_rescans() {
        // `:11893-11910`: dayTime -> the buff dies AND the slot rescans.
        let mut p = player();
        let mut pool = ProjectilePool::default();
        p.buffs.add_buff(28, 60);
        run(&mut p, &mut pool); // inputs().day_time = true
        assert!(p.buffs.find_buff_index(28).is_none(), "the day gate deletes");
        assert!(!p.were_wolf);
    }

    #[test]
    fn w4_f12_late_pet_pins_and_spawns() {
        // `:11406-11419`: Black Cat (84) pins 18000, sets the flag, spawns 319
        // when the owned count is 0.
        let mut p = player();
        let mut pool = ProjectilePool::default();
        p.buffs.add_buff(84, 600);
        run(&mut p, &mut pool);
        let i = p.buffs.find_buff_index(84).unwrap();
        assert_eq!(p.buffs.buff_time[i], 18000, "the pet pin");
        assert!(p.pet_flags[319], "the blackCat flag");
        assert!(pool.slots().iter().any(|s| s.active && s.kind == 319));
    }

    // ---- the DoT flags ----------------------------------------------------

    #[test]
    fn w4_f12_dot_flags_arm() {
        // `:11707-11783`: the DoT flag arms set the C# bools.
        let mut p = player();
        let mut pool = ProjectilePool::default();
        p.buffs.add_buff(20, 600); // Poisoned
        p.buffs.add_buff(70, 600); // Venom
        run(&mut p, &mut pool);
        assert!(p.poisoned);
        assert!(p.venom);
    }

    // ---- the frame --------------------------------------------------------

    #[test]
    fn w4_f12_dont_starve_grants_well_fed() {
        // `:9861-9864` + `:12152-12182`: the DontStarve head re-grants 332.
        let mut p = player();
        let mut pool = ProjectilePool::default();
        let w = world();
        let mut inp = inputs();
        inp.dont_starve = true;
        update_buff_frame(&mut p, &mut pool, &w, &inp);
        assert!(p.buffs.find_buff_index(332).is_some(), "the 332 re-grant");
        assert!(p.pending_frame_emotes.contains(&146), "the starving emote");
    }

    #[test]
    fn w4_f12_no_double_grant_on_inline_owned_ids() {
        // The inline wave-A match owns Swiftness (3): the wiring skips it, so
        // the moveSpeed add is NOT applied twice (the wiring alone adds 0).
        let mut p = player();
        let mut pool = ProjectilePool::default();
        let base_move = p.move_speed;
        p.buffs.add_buff(3, 600);
        run(&mut p, &mut pool);
        assert!((p.move_speed - base_move).abs() < 1e-6, "the wiring skips id 3");
    }
}
