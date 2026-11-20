//! S1-E-7 wave D (W4-c18) — the THIRD missing block of `Player.UpdateBuffs`
//! (`Player.cs:10462-10976`): Ice Barrier (62) + the 29 summon keep-alive arms
//! (49 … 161). Claim + census: `docs/research/28d-wave-d-claim.md`.
//!
//! Disjoint-wave design (c16 owns `:9914-10048`, c17 `:10318-10461`; the live
//! `player.rs::SUMMON_BUFF_ARMS` path stays authoritative until the waves
//! merge): this module is a PURE effect table — `apply_summon_effect_arm`
//! matches one `buffType[j]` id and folds its writes into a `SummonEffects`
//! accumulator, exactly like the C# arm writes into the `Player` fields. The
//! pool side effects (216's respawn, 335/263's status updaters) are recorded
//! as REQUESTS, never executed here — the wiring step maps them onto
//! `ProjectilePool` exactly as `run_baby_bird_arm` (`player.rs:4076`) does.
//!
//! RNG census (`:10462-10976`): ZERO direct draws. The indirect draws ride
//! the deferred spawn seams (216's `NewProjectile` init at `:10712`; the
//! Abigail/StormTiger status updaters' Kill/spawn pairs, live STUBs at
//! `player.rs:4137/:4142` pending M4-PROJ-8 phases 4/6). No stream moves.

use crate::buffs::{BuffSlots, MAX_BUFFS};

/// The per-tick context the arms read besides `buffType`/`buffTime`.
/// `whoAmI == Main.myPlayer` collapses to true in the single-player port,
/// matching every other ported `UpdateBuffs` arm.
pub struct SummonEffectContext<'a> {
    /// `ownedProjectileCounts` (`Player.cs:12184-12215` recount) — every
    /// keep-alive gate reads it (`:10490` etc.).
    pub owned_projectile_counts: &'a [i32],
    /// `statLife` (`:10464`) — Ice Barrier's half-life gate.
    pub stat_life: i32,
    /// `statLifeMax2` (`:10464`) — the POST-buff cap the C# reads.
    pub stat_life_max2: i32,
    /// `numMinions` (`:10706`) — the baby-bird respawn's cap gate. The live
    /// driver reads the chain-accumulated value (the 110/150 raises run
    /// slot-ordered BEFORE the arms, `:10173-10180`).
    pub num_minions: i32,
    /// `maxMinions` (`:10706`) — same slot-order note as `num_minions`.
    pub max_minions: i32,
    /// `FindItem(4281)` (`:10708`, `Player.cs:55972-55981`) — the first
    /// inventory slot holding the Baby Bird Staff with a stack; `None` when
    /// absent. Supplied by the caller so this table stays inventory-free.
    pub baby_bird_staff: Option<BabyBirdStaff>,
}

/// The `FindItem(4281)` hit's spawn inputs (`:10711-10713`): the staff row's
/// `shoot`/`damage`/`knockBack`. The port's data-row spawn seam writes
/// `originalDamage = item.damage` for minion rows (the live arm's note).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BabyBirdStaff {
    pub shoot: i32,
    pub damage: i32,
    pub knock_back: f32,
}

/// The `Player` minion-flag fields the block's arms set. One variant per C#
/// field name; the wiring step maps each onto the live `Player` field (the
/// `flag_*` accessors of `player.rs::SUMMON_BUFF_ARMS`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MinionFlag {
    Pygmy,                     // 49, `Player.cs:10492`
    Raven,                     // 83, `:10509`
    Slime,                     // 64, `:10525`
    HornetMinion,              // 125, `:10541`
    ImpMinion,                 // 126, `:10557`
    SpiderMinion,              // 133, `:10573`
    TwinsMinion,               // 134, `:10589`
    PirateMinion,              // 135, `:10605`
    VampireFrog,               // 214, `:10621`
    SharknadoMinion,           // 139, `:10637`
    UfoMinion,                 // 140, `:10653` (`UFOMinion`)
    StardustMinion,            // 182, `:10669`
    BatsOfLight,               // 213, `:10685`
    BabyBird,                  // 216, `:10702`/`:10714`
    DeadCellsMushroomBoiMinion, // 355, `:10733`
    PalworldCattivaMinion,     // 385, `:10749`
    ClayPotMinion,             // 393, `:10765`
    ForbiddenMinion,           // 394, `:10781`
    PalworldFoxsparksMinion,   // 386, `:10797`
    PalworldTrustyCattivaMinion, // 389, `:10813`
    PalworldTrustyFoxsparksMinion, // 390, `:10829`
    FlinxMinion,               // 325, `:10845`
    AbigailMinion,             // 335, `:10861`
    StormTiger,                // 263, `:10881`
    Smolstar,                  // 271, `:10901`
    EmpressBlade,              // 322, `:10917`
    StardustGuardian,          // 187, `:10933`
    StardustDragon,            // 188, `:10949`
    DeadlySphereMinion,        // 161, `:10965`
}

/// One tick's accumulated writes of the block. Flags are OR-ed per slot like
/// the C# chain; the timer refresh / DelBuff ride the arm's RETURN value (the
/// slot writes belong to the driver, [`run_summon_buff_chain`]).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SummonEffects {
    /// The minion flags the kept arms set this tick (in slot order).
    pub flags: u32, // one bit per `MinionFlag` discriminant
    /// `iceBarrier = true` (62, `:10467`) — only below half life.
    pub ice_barrier: bool,
    /// `endurance += 0.25f` (62, `:10468`) — per kept slot.
    pub endurance: f32,
    /// `Lighting.AddLight(Center/16, 0.1, 0.2, 0.45)` (62, `:10466`) — the
    /// RGB request; tile coords come from the caller (the wave-B pattern).
    pub ice_barrier_light: Option<[f32; 3]>,
    /// 216's respawn request (`:10712`): spawn the staff's shoot at the
    /// player's Top with zero velocity, the staff's damage/knockBack. The
    /// wiring maps it onto `spawn_with_data_ai_owned` (the live arm); on a
    /// failed spawn the slot is DelBuff'd instead (the C# `flag2` guard,
    /// `:10717-10722`) — see [`SummonArmOutcome::BabyBirdRespawn`].
    pub baby_bird_respawn: Option<BabyBirdStaff>,
    /// 335's `UpdateAbigailStatus` request (`:10872-10875`) — UNCONDITIONAL
    /// after the arm (the C# runs it even when the buff was DelBuff'd: that
    /// is the orphan-963 cleanup path, `:12277-12287`). The live port only
    /// runs it on keep (`player.rs:4050-4057`) — a deviation this table
    /// records for the merge (unobservable today: the live updater is a
    /// stub, `player.rs:4137`).
    pub abigail_status: bool,
    /// 263's `UpdateStormTigerStatus` request (`:10892-10895`) — same
    /// unconditional shape (`:12318-12327` kills the wrong-rank tigers).
    pub storm_tiger_status: bool,
    /// 62's frame advance (`:10469-10478`): one step per kept slot per tick.
    /// The wiring folds it through [`advance_ice_barrier_frame`].
    pub ice_barrier_frame_steps: u32,
}

impl SummonEffects {
    fn set_flag(&mut self, f: MinionFlag) {
        self.flags |= 1u32 << (f as u32);
    }
    /// Whether the tick's chain set `f` (the wiring's per-flag copy-out).
    pub fn has_flag(&self, f: MinionFlag) -> bool {
        self.flags & (1u32 << (f as u32)) != 0
    }
}

/// The keep-alive rows in the C# chain's verbatim order
/// (`Player.cs:10486-10976`): (buff id, watched projectile kinds, flag).
/// 216/335/263 carry extra halves the dispatcher special-cases.
pub const SUMMON_ARMS: &[(i32, &[i32], MinionFlag)] = &[
    (49, &[191, 192, 193, 194], MinionFlag::Pygmy), // :10486
    (83, &[317], MinionFlag::Raven),                // :10505
    (64, &[266], MinionFlag::Slime),                // :10521
    (125, &[373], MinionFlag::HornetMinion),        // :10537
    (126, &[375], MinionFlag::ImpMinion),           // :10553
    (133, &[390, 391, 392], MinionFlag::SpiderMinion), // :10569
    (134, &[387, 388], MinionFlag::TwinsMinion),    // :10585
    (135, &[393, 394, 395], MinionFlag::PirateMinion), // :10601
    (214, &[758], MinionFlag::VampireFrog),         // :10617
    (139, &[407], MinionFlag::SharknadoMinion),     // :10633
    (140, &[423], MinionFlag::UfoMinion),           // :10649
    (182, &[613], MinionFlag::StardustMinion),      // :10665
    (213, &[755], MinionFlag::BatsOfLight),         // :10681
    (216, &[759], MinionFlag::BabyBird),            // :10697 (+respawn)
    (355, &[1022], MinionFlag::DeadCellsMushroomBoiMinion), // :10729
    (385, &[1093], MinionFlag::PalworldCattivaMinion), // :10745
    (393, &[1118], MinionFlag::ClayPotMinion),      // :10761
    (394, &[1119], MinionFlag::ForbiddenMinion),    // :10777
    (386, &[1094], MinionFlag::PalworldFoxsparksMinion), // :10793
    (389, &[1112], MinionFlag::PalworldTrustyCattivaMinion), // :10809
    (390, &[1113], MinionFlag::PalworldTrustyFoxsparksMinion), // :10825
    (325, &[951], MinionFlag::FlinxMinion),         // :10841
    (335, &[970], MinionFlag::AbigailMinion),       // :10857 (+status)
    (263, &[831], MinionFlag::StormTiger),          // :10877 (+status)
    (271, &[864], MinionFlag::Smolstar),            // :10897
    (322, &[946], MinionFlag::EmpressBlade),        // :10913
    (187, &[623], MinionFlag::StardustGuardian),    // :10929
    (188, &[625], MinionFlag::StardustDragon),      // :10945
    (161, &[533], MinionFlag::DeadlySphereMinion),  // :10961
];

/// The keep-alive refresh every kept summon arm writes (`buffTime[j] =
/// 18000`, `:10502` etc.).
pub const SUMMON_REFRESH: i32 = 18000;

/// One arm's outcome — the slot write the C# arm performs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SummonArmOutcome {
    /// The id is not this block's — the chain leaves the slot alone.
    NotHandled,
    /// Keep the slot: flag set, `buffTime[j] = 18000` (62 keeps its timer).
    Kept,
    /// `DelBuff(j); j--` — the driver deletes and rescans the index.
    Deleted,
    /// 216's respawn path (`:10706-10727`): the spawn REQUEST is recorded
    /// (`baby_bird_respawn`); on a successful spawn the slot is kept +
    /// refreshed, on a failed spawn (pool full / no row) it is DelBuff'd
    /// (the C# `flag2` guard). The pure driver models the success path;
    /// the wiring maps the failure onto a delete (the live arm's
    /// pool-full stand-in, `player.rs:4099-4117`).
    BabyBirdRespawn,
}

/// One arm of the block (`Player.cs:10462-10976`). Returns the slot outcome;
/// the flag/timer writes ride the accumulator + the driver. The keep-alive
/// gate reads the ACCUMULATED flag as well as the counts — the C# reads the
/// persistent `this.X` field (`if (!this.pygmy)` at `:10495`), so an earlier
/// slot's keep in the same tick keeps a later count-0 slot of the same buff.
pub fn apply_summon_effect_arm(
    e: &mut SummonEffects,
    buff_type: i32,
    ctx: &SummonEffectContext,
) -> SummonArmOutcome {
    if buff_type == 62 {
        // `:10462-10485` — Ice Barrier: below-or-at half of statLifeMax2.
        if (ctx.stat_life as f32) <= (ctx.stat_life_max2 as f32) * 0.5 {
            e.ice_barrier_light = Some([0.1, 0.2, 0.45]); // `:10466`
            e.ice_barrier = true; // `:10467`
            e.endurance += 0.25; // `:10468`
            e.ice_barrier_frame_steps += 1; // `:10469-10478`
            return SummonArmOutcome::Kept; // NO timer refresh in this arm
        }
        return SummonArmOutcome::Deleted; // `:10482-10483`
    }
    let Some(&(_, kinds, flag)) = SUMMON_ARMS.iter().find(|&&(b, _, _)| b == buff_type)
    else {
        return SummonArmOutcome::NotHandled;
    };
    let counted = kinds
        .iter()
        .any(|&k| ctx.owned_projectile_counts[k as usize] > 0);
    if counted {
        e.set_flag(flag);
    }
    if buff_type == 216 {
        // `:10697-10728` — the respawn arm. SP: the myPlayer gate is true.
        // NOTE: unlike the other arms this reads ONLY the count (`:10700`),
        // not the persistent flag — a second count-0 slot still respawns.
        if counted {
            return SummonArmOutcome::Kept; // `:10724-10727` (flag2)
        }
        if ctx.num_minions < ctx.max_minions {
            if let Some(staff) = ctx.baby_bird_staff {
                // `:10712-10714` — the spawn request; the flag arms on a
                // successful spawn (the wiring's decision).
                e.baby_bird_respawn = Some(staff);
                e.set_flag(MinionFlag::BabyBird);
                return SummonArmOutcome::BabyBirdRespawn;
            }
        }
        return SummonArmOutcome::Deleted; // `:10717-10722`
    }
    let outcome = if e.has_flag(flag) {
        SummonArmOutcome::Kept
    } else {
        SummonArmOutcome::Deleted
    };
    // `:10872-10875` / `:10892-10895` — the status updaters run AFTER the
    // keep/delete decision, unconditionally (the orphan-cleanup path).
    match buff_type {
        335 => e.abigail_status = true,
        263 => e.storm_tiger_status = true,
        _ => {}
    }
    outcome
}

/// The slot-ordered driver over the buff bar — the C# `for j` loop with the
/// `DelBuff(j); j--` rescan (`:10482-10483` etc.): after a delete the
/// compaction shifts the later buffs left, so index `j` is re-examined.
/// The timer tick already ran (`buffs.rs::update_buffs_tick`, the `:9869`
/// head); the guard mirrors `:9871-9874`. 62 keeps its own timer; every kept
/// summon slot refreshes to 18000 (`:10502` etc.).
pub fn run_summon_buff_chain(
    slots: &mut BuffSlots,
    e: &mut SummonEffects,
    ctx: &SummonEffectContext,
) {
    let mut j = 0usize;
    while j < MAX_BUFFS {
        let buff = slots.buff_type[j];
        if buff <= 0 || slots.buff_time[j] <= 0 {
            j += 1;
            continue;
        }
        match apply_summon_effect_arm(e, buff, ctx) {
            SummonArmOutcome::NotHandled => j += 1,
            SummonArmOutcome::Kept => {
                if buff != 62 {
                    slots.buff_time[j] = SUMMON_REFRESH;
                }
                j += 1;
            }
            SummonArmOutcome::BabyBirdRespawn => {
                // The success path (see the variant's note).
                slots.buff_time[j] = SUMMON_REFRESH;
                j += 1;
            }
            SummonArmOutcome::Deleted => {
                slots.del_buff(j); // rescan index j
            }
        }
    }
}

/// `iceBarrierFrameCounter`/`iceBarrierFrame` advance (`Player.cs:10469-
/// 10478`): every 3rd kept tick steps the 12-frame cycle. One call per kept
/// slot per tick, in chain order (the wiring folds `ice_barrier_frame_steps`).
pub fn advance_ice_barrier_frame(counter: &mut i32, frame: &mut i32) {
    *counter += 1;
    if *counter > 2 {
        *counter = 0;
        *frame += 1;
        if *frame >= 12 {
            *frame = 0;
        }
    }
}

/// `GetDesiredStormTigerMinionRank` (`Player.cs:12337-12354`): the 831 gem
/// count's tier — `>0` → 1, `>3` → 2, `>6` → 3 (the sequential ifs, last
/// wins). Deterministic; no draws.
pub fn desired_storm_tiger_rank(gem_count: i32) -> i32 {
    let mut result = 0;
    if gem_count > 0 {
        result = 1;
    }
    if gem_count > 3 {
        result = 2;
    }
    if gem_count > 6 {
        result = 3;
    }
    result
}

/// `ProjectileID.Sets.StormTigerIds` (`ID/ProjectileID.cs:541-542`).
pub const STORM_TIGER_IDS: [i32; 3] = [833, 834, 835];

/// The rank → attack-form map (`Player.cs:12297-12303`): 1→833, 2→834,
/// 3→835, else −1 (the C# sentinel).
pub fn storm_tiger_form(rank: i32) -> i32 {
    match rank {
        1 => 833,
        2 => 834,
        3 => 835,
        _ => -1,
    }
}

/// The status updaters' DETERMINISTIC decision cores — the pool side effects
/// (Kill / NewProjectile) are the wiring's; these mirror the live STUBs'
/// future bodies (`player.rs:4137/:4142`, M4-PROJ-8 phases 4/6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompanionDecision {
    /// No pool change this tick.
    Idle,
    /// Kill every owned projectile of `kind` (Abigail's orphan 963 cleanup,
    /// `:12279-12286`) or every wrong-rank StormTiger form (`:12320-12327`,
    /// `kind == -1` kills ALL forms — the rank-0 sentinel).
    Kill { kind: i32, storm_tiger_family: bool },
    /// Spawn `kind` beside the newest base minion (`FindNewestAI_164Minion`,
    /// `:12260-12272`), zero damage, the base's knockBack; the tiger form
    /// additionally pins `localAI[0] = 60` (`:12333`).
    Spawn { kind: i32, base_minion: i32, tiger_cooldown: bool },
}

/// `UpdateAbigailStatus`' decision (`Player.cs:12274-12293`): no 970 → kill
/// the 963 companions; else no 963 → spawn one beside the newest 970.
pub fn abigail_status_decision(counts: &[i32]) -> CompanionDecision {
    if counts[970] < 1 {
        CompanionDecision::Kill { kind: 963, storm_tiger_family: false }
    } else if counts[963] < 1 {
        CompanionDecision::Spawn { kind: 963, base_minion: 970, tiger_cooldown: false }
    } else {
        CompanionDecision::Idle
    }
}

/// `UpdateStormTigerStatus`' decision (`Player.cs:12295-12335`): the desired
/// form by gem rank; any wrong-form tiger alive (or rank 0) → kill the
/// wrong forms; else none of the desired form → spawn it beside the newest
/// 831 gem with the 60-tick `localAI[0]` pin.
pub fn storm_tiger_status_decision(counts: &[i32]) -> CompanionDecision {
    let form = storm_tiger_form(desired_storm_tiger_rank(counts[831]));
    let wrong_alive = form == -1
        || STORM_TIGER_IDS
            .iter()
            .any(|&k| k != form && counts[k as usize] >= 1);
    if wrong_alive {
        CompanionDecision::Kill { kind: form, storm_tiger_family: true }
    } else if counts[form as usize] < 1 {
        CompanionDecision::Spawn { kind: form, base_minion: 831, tiger_cooldown: true }
    } else {
        CompanionDecision::Idle
    }
}

