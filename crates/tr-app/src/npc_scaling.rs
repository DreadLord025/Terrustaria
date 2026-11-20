//! Expert/Master mode stat scaling — `NPC.ScaleStats` and its whole
//! family (`NPC.cs:18146-18739`), the difficulty curves
//! (`DataStructures/GameDifficultyData.cs:35-92`) and the invasion-group
//! table (`NPC.cs:78297-78412`). W4-e25, code-only mode.
//!
//! Census: `docs/research/64-expert-master-scaling-census.md`. The port
//! had NO `ScaleStats` port (the i11 deviation note, ANALYSIS "The
//! player→NPC strike mechanics" — the expert kb curve + the 266/267
//! player-count kb loop were unmodelled); this module fills the maths
//! verbatim as pure fns over the [`ScalingStats`] seam, keyed on the RAW
//! C# type id (the `npc_ai_style53.rs` unwired-module precedent).
//!
//! UNWIRED: the live spawns keep the classic SetDefaults values
//! byte-identical — the World carries no game mode (the recurring
//! constants-off note: nurse `Main.cs:40086`, spawner `NPC.cs:853`,
//! dd2 `DD2Event.cs:1123`); the wiring rides the difficulty-flip task.
//!
//! RNG census: the WHOLE family draws ZERO `Main.rand` — pure
//! arithmetic, so no fn takes an rng parameter (determinism by
//! construction, the `npc_ai_style28.rs` witness shape).
//!
//! Rounding: `Math.Round` is banker's (`tr_core::damage::round_bankers`);
//! every `(int)` cast is a truncation. The f32/f64 split is kept exactly:
//! the Tweaks multipliers are f32 products over f64 table constants, the
//! ByPlayerCount final round is an f64 product.

#![allow(dead_code)] // wiring deferred (W4-e25, code-only mode — the World carries no game mode)

/// `GameDifficultyLevel` (`DataStructures/GameDifficultyLevel.cs:17-21`).
pub const JOURNEY: f32 = 0.5;
pub const CLASSIC: f32 = 1.0;
pub const EXPERT: f32 = 2.0;
pub const MASTER: f32 = 3.0;
pub const LEGENDARY: f32 = 4.0;

/// `NPC.CommonMasterBossLifeReduction` (`NPC.cs:6587`, `:95269`) — 0.85.
pub const COMMON_MASTER_BOSS_LIFE_REDUCTION: f64 = 0.85;

/// The NPC scalars `ScaleStats` reads/writes — the C# fields of the same
/// names (`life`/`lifeMax`/`damage`/`defense`/`value`/`knockBackResist`/
/// `scale`/`boss`/`friendly`/`townNPC`/`difficulty`/`defDamage`/
/// `defDefense`/`defLifeMax`/`statsAreScaledForThisManyPlayers`). The
/// live `Npc` struct does not grow the def*/scaled-for fields until the
/// wiring round; the seam carries them here.
#[derive(Clone, Copy, Debug)]
pub struct ScalingStats {
    /// The RAW C# `NPC.type` (not the port's kind/protocol id).
    pub npc_type: i32,
    pub life: i32,
    pub life_max: i32,
    pub damage: i32,
    pub defense: i32,
    /// `NPC.value` — a float field in the C# (the coin copper total).
    pub value: f32,
    pub knockback_resist: f32,
    pub scale: f32,
    pub boss: bool,
    pub friendly: bool,
    pub town_npc: bool,
    /// `NPC.difficulty` — written by `ScaleStats` itself (`:18150`).
    pub difficulty: f32,
    pub def_damage: i32,
    pub def_defense: i32,
    pub def_life_max: i32,
    /// `NPC.statsAreScaledForThisManyPlayers` (`:18562`).
    pub stats_scaled_for_players: i32,
}

/// The world state the family reads — `Main.Difficulty`/`hardMode`/
/// `getGoodWorld`/`NPC.downedPlantBoss`/`Main.netMode` + the `AnyNPCs`
/// scan (the getGood twin arms, `:18190`/`:18199`/`:18208`).
pub struct ScalingEnv<'a> {
    pub difficulty: f32,
    pub hard_mode: bool,
    pub get_good_world: bool,
    pub downed_plant_boss: bool,
    pub net_mode: i32,
    /// `NPC.AnyNPCs(type)` — the caller's live-NPC scan.
    pub any_npcs: &'a dyn Fn(i32) -> bool,
}

// ---------------------------------------------------------------------
// The ID sets (`ID/NPCID.cs`)
// ---------------------------------------------------------------------

/// `NPCID.Sets.NeedsExpertScaling` (`NPCID.cs:10447`).
pub fn needs_expert_scaling(t: i32) -> bool {
    matches!(
        t,
        25 | 30 | 665 | 33 | 112 | 666 | 261 | 265 | 371 | 516 | 519 | 397 | 396 | 398 | 491
    )
}

/// `NPCID.Sets.DontDoHardmodeScaling` (`NPCID.cs:10116`).
pub fn dont_do_hardmode_scaling(t: i32) -> bool {
    matches!(
        t,
        5 | 13 | 14 | 15 | 267 | 113 | 114 | 115 | 116 | 117 | 118 | 119 | 658 | 659 | 660 | 400
            | 522
    )
}

/// `NPCID.Sets.ProjectileNPC` (`NPCID.cs:10448`).
pub fn projectile_npc(t: i32) -> bool {
    matches!(t, 25 | 30 | 665 | 33 | 112 | 666 | 261 | 265 | 371 | 516 | 519)
}

/// `NPCID.Sets.BelongsToInvasionOldOnesArmy` (`NPCID.cs:10432`).
pub fn belongs_to_invasion_old_ones_army(t: i32) -> bool {
    matches!(
        t,
        552 | 553
            | 554
            | 561
            | 562
            | 563
            | 555
            | 556
            | 557
            | 558
            | 559
            | 560
            | 576
            | 577
            | 568
            | 569
            | 566
            | 567
            | 570
            | 571
            | 572
            | 573
            | 548
            | 549
            | 564
            | 565
            | 574
            | 575
            | 551
            | 578
    )
}

/// `NPC.GetNPCInvasionGroup` (`NPC.cs:78297-78412`) — goblin 1, frost 2,
/// pirate 3, martian 4, frost-moon −1, pumpkin-moon −2, OOA −3, else 0.
pub fn get_npc_invasion_group(npc_id: i32) -> i32 {
    match npc_id {
        26 | 27 | 28 | 29 | 111 | 471 | 472 => 1,
        143 | 144 | 145 => 2,
        212 | 213 | 214 | 215 | 216 | 252 | 491 | 492 | 662 => 3,
        381 | 382 | 383 | 385 | 386 | 387 | 388 | 389 | 390 | 391 | 394 | 395 | 520 => 4,
        338..=352 => -1,
        305..=315 | 325 | 326 | 327 | 329 | 330 => -2,
        547 | 548 | 549 | 551..=578 => -3,
        _ => 0,
    }
}

// ---------------------------------------------------------------------
// The curves + the Utils seams
// ---------------------------------------------------------------------

/// `LinearCurve.Sample` (`GameDifficultyData.cs:35-55`) — the bracket
/// scan, the zero-span guard returns `key.output`, else `(value -
/// key.input) * num2 / num + key.output` (the operation order kept; the
/// `npc_ai_style74.rs:283-299` shape).
pub fn linear_curve_sample(keys: &[(f32, f32)], value: f32) -> f32 {
    let mut key = keys[0];
    let mut key2 = key;
    for k in keys {
        key2 = *k;
        if value <= key2.0 {
            break;
        }
        key = key2;
    }
    let num = key2.0 - key.0;
    let num2 = key2.1 - key.1;
    if num == 0.0 {
        return key.1;
    }
    (value - key.0) * num2 / num + key.1
}

/// `GameDifficultyData.EnemyMaxLifeMultiplier` (`GameDifficultyData.cs:83`):
/// Journey 0.5→0.5, Legendary 4→4 (a single span — Classic 1, Expert 2,
/// Master 3).
pub const ENEMY_MAX_LIFE_KEYS: [(f32, f32); 2] = [(0.5, 0.5), (4.0, 4.0)];

/// `GameDifficultyData.EnemyDamageMultiplier` (`:84`).
pub const ENEMY_DAMAGE_KEYS: [(f32, f32); 3] = [(0.5, 0.5), (3.0, 3.0), (4.0, 5.3333335)];

/// `GameDifficultyData.KnockbackToEnemiesMultiplier` (`:86`) — the i11
/// deviation's curve: Expert samples to 0.8999999761581421 in f32, NOT
/// an exact 0.9.
pub const KNOCKBACK_KEYS: [(f32, f32); 2] = [(1.0, 1.0), (3.0, 0.8)];

/// `GameDifficultyData.EnemyMoneyDropMultiplier` (`:87`) — the expert
/// money "drop bonus": Expert 2.5, Master 2.5, Legendary 3.5.
pub const ENEMY_MONEY_KEYS: [(f32, f32); 4] =
    [(1.0, 1.0), (2.0, 2.5), (3.0, 2.5), (4.0, 3.5)];

/// `Utils.GetLerpValue(from, to, t, clamped)` (`Utils.cs:315-343`) — the
/// clamped branch reads the from<to orientation; the unclamped branch
/// falls straight to the ratio.
pub fn get_lerp_value(from: f32, to: f32, t: f32, clamped: bool) -> f32 {
    if clamped {
        if from < to {
            if t < from {
                return 0.0;
            }
            if t > to {
                return 1.0;
            }
        } else {
            if t < to {
                return 1.0;
            }
            if t > from {
                return 0.0;
            }
        }
    }
    (t - from) / (to - from)
}

/// XNA `MathHelper.Lerp(value1, value2, amount)` — `value1 + (value2 -
/// value1) * amount` (f32).
fn math_helper_lerp(value1: f32, value2: f32, amount: f32) -> f32 {
    value1 + (value2 - value1) * amount
}

/// `Utils.Remap` (float, `Utils.cs:345-348`) — `MathHelper.Lerp(toMin,
/// toMax, GetLerpValue(fromMin, fromMax, fromValue, clamped: true))`.
pub fn remap_f32(from_value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    math_helper_lerp(
        to_min,
        to_max,
        get_lerp_value(from_min, from_max, from_value, true),
    )
}

/// `Utils.Lerp` (double, `Utils.cs:107-110`) — `value1 + (value2 -
/// value1) * amount`.
pub fn lerp_f64(value1: f64, value2: f64, amount: f64) -> f64 {
    value1 + (value2 - value1) * amount
}

/// `NPC.GetKnockbackMultiplier_ScaledByDifficulty` (`NPC.cs:7031-7034`).
pub fn knockback_scaled(normal: f32, difficulty: f32) -> f32 {
    normal * linear_curve_sample(&KNOCKBACK_KEYS, difficulty)
}

/// `NPC.GetAttackDamage_ScaledByDifficulty` (`NPC.cs:7036-7039`) — the
/// `(int)` truncation of the scaled product.
pub fn attack_damage_scaled(normal_damage: f32, difficulty: f32) -> i32 {
    (normal_damage * linear_curve_sample(&ENEMY_DAMAGE_KEYS, difficulty)) as i32
}

// ---------------------------------------------------------------------
// The player-count factors
// ---------------------------------------------------------------------

/// `NPC.GetStatScalingFactors` (`NPC.cs:18722-18739`) — `(balance,
/// boost)`, all f32: per extra player `balance += boost; boost += (1f -
/// boost) / 3f`; the >8 soft cap `(balance*2+8)/3`, the 1000 hard cap.
pub fn get_stat_scaling_factors(num_players: i32) -> (f32, f32) {
    let mut balance = 1.0f32;
    let mut boost = 0.35f32;
    let mut i = 1;
    while i < num_players {
        balance += boost;
        boost += (1.0 - boost) / 3.0;
        i += 1;
    }
    if balance > 8.0 {
        balance = (balance * 2.0 + 8.0) / 3.0;
    }
    if balance > 1000.0 {
        balance = 1000.0;
    }
    (balance, boost)
}

/// `NPC.GetActivePlayerCount` (`NPC.cs:18493-18508`) — `netMode == 0 →
/// 1`; else the 255-slot `active && !ghost` scan (the caller supplies
/// the slots' `(active, ghost)` flags).
pub fn get_active_player_count(net_mode: i32, players: &[(bool, bool)]) -> i32 {
    if net_mode == 0 {
        return 1;
    }
    let mut num = 0;
    for &(active, ghost) in players.iter().take(255) {
        if active && !ghost {
            num += 1;
        }
    }
    num
}

/// `NPC.GetBalance` (`NPC.cs:18477-18481`) — the live balance.
pub fn get_balance(net_mode: i32, players: &[(bool, bool)]) -> f32 {
    get_stat_scaling_factors(get_active_player_count(net_mode, players)).0
}

/// `NPC.GetMyBalance` (`NPC.cs:18483-18491`) — the spawn-time balance;
/// `statsAreScaledForThisManyPlayers <= 1 → 1f`.
pub fn get_my_balance(stats: &ScalingStats) -> f32 {
    if stats.stats_scaled_for_players <= 1 {
        return 1.0;
    }
    get_stat_scaling_factors(stats.stats_scaled_for_players).0
}

// ---------------------------------------------------------------------
// ScaleStats_ForExpertHardmode (NPC.cs:18510-18558)
// ---------------------------------------------------------------------

/// The expert-hardmode bump for weak pre-hardmode enemies. The INT
/// `num2 / num` division is the C# quirk (`:18549`): num 41 gives
/// num3 = 1, not 1.95.
pub fn scale_stats_for_expert_hardmode(s: &mut ScalingStats, env: &ScalingEnv) {
    let flag = s.npc_type >= 0 && projectile_npc(s.npc_type);
    let mut flag2 = !dont_do_hardmode_scaling(s.npc_type);
    if env.get_good_world {
        // `:18516-18527` — the twin pairs skip the bump while the
        // partner is up.
        if (s.npc_type == 24 || s.npc_type == 25) && (env.any_npcs)(113) {
            flag2 = false;
        }
        if (s.npc_type == 32 || s.npc_type == 33) && (env.any_npcs)(35) {
            flag2 = false;
        }
        if s.npc_type == 6 && (env.any_npcs)(13) {
            flag2 = false;
        }
    }
    if s.boss || s.life_max >= 1000 {
        flag2 = false;
    }
    if !flag2 {
        return;
    }
    let mut num = s.damage + s.defense + s.life_max / 4;
    if num == 0 {
        num = 1;
    }
    let mut num2 = 80;
    if env.downed_plant_boss {
        num2 += 20;
    }
    if num < num2 {
        let num3 = num2 / num; // INT division (`:18549`)
        s.damage = (s.damage as f32 * num3 as f32 * 0.9) as i32;
        if !flag {
            s.defense = (s.defense as f32 * num3 as f32) as i32;
            s.life_max = ((s.life_max as f32 * num3 as f32) as f64 * 1.1) as i32;
            s.value = ((s.value * num3 as f32) as f64 * 0.8) as i32 as f32;
        }
    }
}

// ---------------------------------------------------------------------
// ScaleStats_ByDifficulty (NPC.cs:18171-18181) + the Tweaks table
// ---------------------------------------------------------------------

/// `NPC.ScaleStats_ByDifficulty` (`NPC.cs:18171-18181`).
pub fn scale_stats_by_difficulty(s: &mut ScalingStats, env: &ScalingEnv) {
    if s.npc_type < 0 || !projectile_npc(s.npc_type) {
        s.life_max =
            (s.life_max as f32 * linear_curve_sample(&ENEMY_MAX_LIFE_KEYS, s.difficulty)) as i32;
        s.value =
            (s.value * linear_curve_sample(&ENEMY_MONEY_KEYS, s.difficulty)) as i32 as f32;
    }
    s.damage = attack_damage_scaled(s.damage as f32, s.difficulty);
    s.knockback_resist = knockback_scaled(s.knockback_resist, s.difficulty);
    scale_stats_by_difficulty_tweaks(s, env);
}

/// `NPC.ScaleStats_ByDifficulty_Tweaks` (`NPC.cs:18183-18475`) — the
/// per-type boss table. `num` = the Classic→Expert damage remap target,
/// `num2` = the Classic→Expert life target, `num3` = the Expert→Master
/// life target (f64 table constants, f32 remap products — the C# mix).
pub fn scale_stats_by_difficulty_tweaks(s: &mut ScalingStats, env: &ScalingEnv) {
    let mut num = 1.0f64;
    let mut num2 = 1.0f64;
    let mut num3 = 1.0f64;
    if env.get_good_world {
        // `:18188-18217` — the getGoodWorld twin arms.
        if s.npc_type == 6 && (env.any_npcs)(13) {
            num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
            num2 *= 1.5;
            if s.difficulty >= EXPERT {
                s.defense += 2;
            }
        }
        if s.npc_type == 32 && (env.any_npcs)(35) {
            num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
            num2 *= 1.5;
            if s.difficulty >= EXPERT {
                s.defense += 6;
            }
        }
        if s.npc_type == 24 && (env.any_npcs)(113) {
            num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
            num2 *= 1.5;
            if s.difficulty >= EXPERT {
                s.defense += 10;
            }
        }
    }
    let t = s.npc_type;
    if t == 210 || t == 211 {
        // `:18218-18223` — the bees; the 0.6000000000000001 literal kept.
        num2 = 0.6000000000000001;
        num = 0.6;
        s.defense =
            (s.defense as f32 * remap_f32(s.difficulty, CLASSIC, EXPERT, 1.0, 0.8)) as i32;
    } else if t == 5 {
        num2 = 0.75;
        num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
    } else if t == 4 {
        num2 = 0.65;
        num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
    } else if (13..=15).contains(&t) {
        num2 = 0.7;
        num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
        if t == 13 {
            num *= 1.1;
        }
        if t == 14 {
            num *= 0.8;
        }
        if t == 15 {
            num *= 0.8;
        }
        if s.difficulty >= EXPERT {
            s.scale *= 1.2;
            s.defense += 2;
        }
    } else if t == 266 || t == 267 {
        num2 = 0.85;
        num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
        num *= 0.9;
        if s.difficulty >= EXPERT {
            s.scale *= 1.05;
        }
    } else if t == 50 {
        num2 = 0.7;
        num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
        num *= 0.8;
    } else if t == 471 {
        num2 = 0.85;
    } else if t == 472 {
        num2 = 0.85;
        num = 0.8;
    } else if t == 492 {
        num2 = 0.85;
    } else if t == 393 || t == 394 || t == 395 {
        num2 = 0.85;
    } else if t == 222 {
        num2 = 0.7;
        num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
        num = 0.9;
    } else if t == 35 {
        num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
        num = 1.1;
    } else if t == 36 {
        num2 = 1.3;
        num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
        num = 1.1;
    } else if t == 668 {
        num2 = 0.85;
        num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
    } else if t == 113 || t == 114 {
        num2 = 0.7;
        num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
        num = 1.5;
        if s.difficulty >= EXPERT {
            s.defense += 6;
        }
    } else if t == 115 {
        num2 = 0.7;
        if s.difficulty >= EXPERT {
            s.defense += 6;
        }
    } else if t == 116 {
        num2 = 0.7;
    } else if t == 117 || t == 118 || t == 119 {
        num2 = 0.8;
    } else if t == 657 {
        num2 = 0.8;
        num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
    } else if (658..=660).contains(&t) {
        num2 = 0.75;
        num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
    } else if (134..=136).contains(&t) {
        num2 = 0.75;
        num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
        if t == 134 {
            num *= 2.0;
        }
        if t == 135 {
            num *= 0.85;
        }
        if t == 136 {
            num *= 0.85;
        }
        if s.difficulty >= EXPERT {
            s.scale *= 1.05;
        }
    } else if t == 139 {
        num2 = 0.75;
        num = 0.8;
        if s.difficulty >= EXPERT {
            s.scale *= 1.05;
        }
    } else if (127..=131).contains(&t) {
        num2 = 0.75;
        num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
        num = 0.85;
    } else if (125..=126).contains(&t) {
        num2 = 0.75;
        num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
        num = 0.85;
    } else if t == 262 {
        num2 = 0.7;
        num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
        num = 1.15;
    } else if t == 264 {
        num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
        num = 1.15;
    } else if t == 636 {
        num2 = 0.7;
        num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
    } else if (245..=249).contains(&t) {
        num2 = 0.75;
        num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
        num = 0.8;
    } else if t == 370 {
        num2 = 0.65;
        num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
        num = 0.7;
    } else if t == 371 || t == 372 || t == 373 {
        if t != 371 {
            num2 = 0.75;
        }
        num = 0.75;
    } else if t == 439
        || t == 440
        || (454..=459).contains(&t)
        || t == 522
        || t == 523
    {
        if t != 522 {
            num2 = 0.75;
            num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
        }
        num = 0.75;
    } else if t == 397 || t == 396 || t == 398 {
        num2 = 0.75;
        num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
        num = 0.75;
    } else if t == 551 {
        num2 = 0.75;
        num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
        num = 0.65;
    } else if belongs_to_invasion_old_ones_army(t) {
        num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
    }
    let invasion_group = get_npc_invasion_group(t);
    if invasion_group == -2 || invasion_group == -1 {
        // `:18450-18470` — the moon bosses get the deeper cut.
        match t {
            315 | 325 | 327 | 344 | 345 | 346 => {
                num2 = 0.65;
                num3 *= COMMON_MASTER_BOSS_LIFE_REDUCTION;
                num = 0.75;
            }
            _ => {
                num2 = 0.75;
                num = 0.75;
            }
        }
    }
    // `:18471-18474` — the f32 remap products, the banker's rounds.
    let num4 = 1.0f32
        * remap_f32(s.difficulty, CLASSIC, EXPERT, 1.0, num2 as f32)
        * remap_f32(s.difficulty, EXPERT, MASTER, 1.0, num3 as f32);
    s.life_max = tr_core::damage::round_bankers((s.life_max as f32 * num4) as f64) as i32;
    let num5 = 1.0f32 * remap_f32(s.difficulty, CLASSIC, EXPERT, 1.0, num as f32);
    s.damage = tr_core::damage::round_bankers((s.damage as f32 * num5) as f64) as i32;
}

// ---------------------------------------------------------------------
// ScaleStats_ByPlayerCount (NPC.cs:18560-18720)
// ---------------------------------------------------------------------

/// The multiplayer boss-HP scaling. The if/else structure is the C#'s
/// exactly (the standalone ifs vs the else-if chains differ in which
/// arms can stack — e.g. 35/36 are an else-if pair, 668 is standalone).
pub fn scale_stats_by_player_count(s: &mut ScalingStats, num_players: i32) {
    s.stats_scaled_for_players = num_players;
    let (balance, boost) = get_stat_scaling_factors(num_players);
    let mut num = 1.0f64;
    let t = s.npc_type;
    if t == 4 {
        num *= balance as f64;
    }
    if (13..=15).contains(&t) {
        num *= balance as f64;
    }
    if t == 266 || t == 267 {
        num *= balance as f64;
        // `:18576-18584` — the kb loop: ×0.8 per 0.34 balance step,
        // floored to 0 below 0.1 (the i11 deviation's second half).
        let mut num2 = 1.0f32;
        while num2 < balance {
            if (s.knockback_resist as f64) < 0.1 {
                s.knockback_resist = 0.0;
                break;
            }
            s.knockback_resist *= 0.8;
            num2 += 0.34;
        }
    }
    if t == 50 {
        num *= balance as f64;
    }
    if t == 471 {
        num *= lerp_f64(1.0, balance as f64, 2.0 / 3.0);
    }
    if t == 472 {
        num *= lerp_f64(1.0, balance as f64, 0.5);
    }
    if t == 222 {
        num *= balance as f64;
    }
    if t == 35 {
        num *= balance as f64;
    } else if t == 36 {
        num *= balance as f64;
    }
    if t == 668 {
        num *= balance as f64;
    }
    if t == 113 || t == 114 {
        num *= balance as f64;
    } else if t == 115 {
        num *= balance as f64;
        if num_players > 4 {
            s.knockback_resist = 0.0;
        } else if num_players > 1 {
            s.knockback_resist *= 1.0 - boost;
        }
    } else if t == 116 {
        num *= balance as f64;
        if num_players > 4 {
            s.knockback_resist = 0.0;
        } else if num_players > 1 {
            s.knockback_resist *= 1.0 - boost;
        }
    }
    if t == 657 {
        num *= balance as f64;
    }
    if (658..=660).contains(&t) {
        num *= balance as f64;
    }
    if (134..=136).contains(&t) {
        num *= balance as f64;
    } else if t == 139 {
        num *= lerp_f64(1.0, balance as f64, 2.0 / 3.0);
    }
    if (127..=131).contains(&t) {
        num *= balance as f64;
    }
    if (125..=126).contains(&t) {
        num *= balance as f64;
    }
    if t == 262 {
        num *= balance as f64;
    } else if t == 264 {
        num *= balance as f64;
    }
    if t == 636 {
        num *= balance as f64;
    }
    if (245..=249).contains(&t) {
        num *= balance as f64;
    }
    if t == 370 {
        num *= balance as f64;
    }
    // `:18686` — 522 is EXCLUDED here (unlike the Tweaks arm).
    if t == 439 || t == 440 || (454..=459).contains(&t) || t == 523 {
        num *= balance as f64;
    }
    if t == 397 || t == 396 || t == 398 {
        num *= balance as f64;
    }
    if t == 551 {
        num *= balance as f64;
    } else if belongs_to_invasion_old_ones_army(t) {
        // `:18700` — the f32 6/7 literal widened to double.
        num *= lerp_f64(1.0, balance as f64, 0.8571428656578064);
    }
    let invasion_group = get_npc_invasion_group(t);
    if invasion_group == -2 || invasion_group == -1 {
        // `:18702-18718` — the invasion mobs scale linearly; the six
        // moon-boss ids skip.
        match t {
            315 | 325 | 327 | 344 | 345 | 346 => {}
            _ => {
                num *= 1.0 + (num_players - 1) as f64 * 0.2;
            }
        }
    }
    // `:18719` — the f64 product, banker's round.
    s.life_max = tr_core::damage::round_bankers(s.life_max as f64 * num) as i32;
}

// ---------------------------------------------------------------------
// ScaleStats (NPC.cs:18146-18169) — the entry
// ---------------------------------------------------------------------

/// `NPC.ScaleStats(activePlayersCount, strengthOverride)` — the whole
/// pipeline in the C# order: the gate, the difficulty write, the
/// expert-hardmode bump, the difficulty curves + tweaks, the
/// player-count scale, the life floor + the def* snaps.
pub fn scale_stats(
    s: &mut ScalingStats,
    env: &ScalingEnv,
    active_players_count: Option<i32>,
    strength_override: Option<f32>,
    players: &[(bool, bool)],
) {
    if !(needs_expert_scaling(s.npc_type)
        || (s.life_max > 5 && s.damage != 0 && !s.friendly && !s.town_npc))
    {
        return;
    }
    s.difficulty = strength_override.unwrap_or(env.difficulty);
    if s.difficulty >= EXPERT && env.hard_mode {
        scale_stats_for_expert_hardmode(s, env);
    }
    scale_stats_by_difficulty(s, env);
    if s.difficulty >= EXPERT {
        let count = active_players_count
            .unwrap_or_else(|| get_active_player_count(env.net_mode, players));
        scale_stats_by_player_count(s, count);
    }
    if (s.npc_type < 0 || !projectile_npc(s.npc_type)) && s.life_max < 6 {
        s.life_max = 6;
    }
    s.life = s.life_max;
    s.def_damage = s.damage;
    s.def_defense = s.defense;
    s.def_life_max = s.life_max;
}

#[cfg(test)]
mod w4_e25_scaling_tests {
    //! W4-e25 witnesses — the scaling pins (WRITTEN NOT RUN, code-only
    //! mode). Every expected value re-derived from the C# maths with
    //! exact f32/f64 semantics (the census doc §1 carries the tables).
    use super::*;

    fn env(difficulty: f32) -> ScalingEnv<'static> {
        ScalingEnv {
            difficulty,
            hard_mode: false,
            get_good_world: false,
            downed_plant_boss: false,
            net_mode: 0,
            any_npcs: &|_| false,
        }
    }

    fn stats(npc_type: i32, life_max: i32, damage: i32) -> ScalingStats {
        ScalingStats {
            npc_type,
            life: life_max,
            life_max,
            damage,
            defense: 0,
            value: 0.0,
            knockback_resist: 0.0,
            scale: 1.0,
            boss: false,
            friendly: false,
            town_npc: false,
            difficulty: CLASSIC,
            def_damage: 0,
            def_defense: 0,
            def_life_max: 0,
            stats_scaled_for_players: 0,
        }
    }

    #[test]
    fn w4_e25_the_stat_scaling_factors_ladder() {
        // NPC.cs:18722-18739 — the f32 ladder, exact bits.
        assert_eq!(get_stat_scaling_factors(1), (1.0, 0.35));
        let (b2, bo2) = get_stat_scaling_factors(2);
        assert_eq!(b2, 1.350000023841858f32);
        assert_eq!(bo2, 0.5666666626930237f32);
        assert_eq!(get_stat_scaling_factors(3).0, 1.9166667461395264f32);
        // 10 players crosses the >8 soft cap: (b*2+8)/3.
        assert_eq!(get_stat_scaling_factors(10).0, 8.06714916229248f32);
        assert_eq!(get_stat_scaling_factors(30).0, 21.36667823791504f32);
        // The 1000 hard cap.
        assert_eq!(get_stat_scaling_factors(3000).0, 1000.0f32);
    }

    #[test]
    fn w4_e25_the_difficulty_curves_sample_the_c_sharp_keys() {
        // GameDifficultyData.cs:83-87 — Expert 2 / Master 3.
        assert_eq!(linear_curve_sample(&ENEMY_MAX_LIFE_KEYS, EXPERT), 2.0);
        assert_eq!(linear_curve_sample(&ENEMY_MAX_LIFE_KEYS, MASTER), 3.0);
        assert_eq!(linear_curve_sample(&ENEMY_MONEY_KEYS, EXPERT), 2.5);
        assert_eq!(linear_curve_sample(&ENEMY_MONEY_KEYS, MASTER), 2.5);
        assert_eq!(linear_curve_sample(&ENEMY_MONEY_KEYS, LEGENDARY), 3.5);
        assert_eq!(linear_curve_sample(&ENEMY_DAMAGE_KEYS, MASTER), 3.0);
        // The i11 kb curve: expert is NOT an exact 0.9 in f32.
        assert_eq!(
            linear_curve_sample(&KNOCKBACK_KEYS, EXPERT),
            0.8999999761581421f32
        );
        assert_eq!(
            linear_curve_sample(&KNOCKBACK_KEYS, MASTER),
            0.800000011920929f32
        );
        // Journey clamps to the first key (the bracket scan).
        assert_eq!(linear_curve_sample(&ENEMY_MAX_LIFE_KEYS, JOURNEY), 0.5);
        assert_eq!(linear_curve_sample(&ENEMY_MONEY_KEYS, JOURNEY), 1.0);
    }

    #[test]
    fn w4_e25_the_attack_damage_scale_truncates() {
        // NPC.cs:7036-7039 — (int) truncation, not a round.
        assert_eq!(attack_damage_scaled(7.0, EXPERT), 14);
        assert_eq!(attack_damage_scaled(7.0, MASTER), 21);
        // 5 * 5.3333335 = 26.666667 → 26 (legendary).
        assert_eq!(attack_damage_scaled(5.0, LEGENDARY), 26);
        assert_eq!(knockback_scaled(0.8, EXPERT), 0.7199999690055847f32);
    }

    #[test]
    fn w4_e25_the_expert_hardmode_bump_uses_int_division() {
        // NPC.cs:18537-18557 — num = damage + defense + lifeMax/4.
        let mut s = stats(1, 100, 10);
        s.defense = 20; // num = 10 + 20 + 25 = 55 < 80 → num3 = 80/55 = 1
        s.value = 500.0;
        scale_stats_for_expert_hardmode(&mut s, &env(EXPERT));
        assert_eq!(s.damage, 9, "(int)(10 * 1 * 0.9)");
        assert_eq!(s.defense, 20);
        assert_eq!(s.life_max, 110, "(int)((double)(100*1) * 1.1)");
        assert_eq!(s.value, 400.0, "(int)((double)(500*1) * 0.8)");
        // num = 40 → num3 = 2 exactly.
        let mut s = stats(1, 100, 10);
        s.defense = 5; // 10 + 5 + 25 = 40
        s.value = 500.0;
        scale_stats_for_expert_hardmode(&mut s, &env(EXPERT));
        assert_eq!(s.damage, 18, "(int)(10 * 2 * 0.9)");
        assert_eq!(s.defense, 10);
        assert_eq!(s.life_max, 220);
        assert_eq!(s.value, 800.0);
        // The INT-division quirk: num 41 → num3 = 1 (not 1.95).
        let mut s = stats(1, 104, 10);
        s.defense = 5; // 10 + 5 + 26 = 41
        scale_stats_for_expert_hardmode(&mut s, &env(EXPERT));
        assert_eq!(s.damage, 9);
        // Post-Plantera the bar is 100: num 90 now bumps.
        let mut e = env(EXPERT);
        e.downed_plant_boss = true;
        let mut s = stats(1, 100, 10);
        s.defense = 55; // 10 + 55 + 25 = 90 < 100 → num3 = 1
        scale_stats_for_expert_hardmode(&mut s, &e);
        assert_eq!(s.damage, 9);
        // Bosses and lifeMax >= 1000 skip; the DontDoHardmodeScaling
        // set skips (type 5); ProjectileNPC keeps only the damage arm.
        let mut s = stats(1, 1000, 10);
        scale_stats_for_expert_hardmode(&mut s, &env(EXPERT));
        assert_eq!(s.life_max, 1000);
        let mut s = stats(5, 100, 10);
        scale_stats_for_expert_hardmode(&mut s, &env(EXPERT));
        assert_eq!(s.damage, 10, "type 5 is DontDoHardmodeScaling");
        let mut s = stats(25, 100, 10); // ProjectileNPC
        s.defense = 5;
        scale_stats_for_expert_hardmode(&mut s, &env(EXPERT));
        assert_eq!(s.damage, 18);
        assert_eq!(s.life_max, 100, "projectile: no life/def/value bump");
    }

    #[test]
    fn w4_e25_the_tweaks_bankers_round_the_boss_life() {
        // NPC.cs:18471-18474 — EoC (4): num2 0.65, num3 0.85.
        let mut s = stats(4, 1000, 100);
        s.difficulty = MASTER;
        scale_stats_by_difficulty_tweaks(&mut s, &env(MASTER));
        // num4 = 0.65f * 0.85f = 0.5525000095367432; f32(1000 * num4)
        // lands on 552.5 EXACTLY → the banker's midpoint rounds to 552.
        assert_eq!(s.life_max, 552, "the .5 midpoint rounds to even");
        assert_eq!(s.damage, 100, "EoC num = 1 → no damage remap");
        let mut s = stats(4, 1000, 100);
        s.difficulty = EXPERT;
        scale_stats_by_difficulty_tweaks(&mut s, &env(EXPERT));
        assert_eq!(s.life_max, 650, "1000 * 0.65");
        // EoW head (13): expert scale ×1.2, defense +2, damage ×1.1.
        let mut s = stats(13, 100, 50);
        s.difficulty = EXPERT;
        scale_stats_by_difficulty_tweaks(&mut s, &env(EXPERT));
        assert_eq!(s.damage, 55, "50 * 1.1 = 55.000001 → 55");
        assert_eq!(s.scale, 1.2f32);
        assert_eq!(s.defense, 2);
        assert_eq!(s.life_max, 70);
        // The bees (210): the defense remap ×0.8 at expert.
        let mut s = stats(210, 100, 10);
        s.defense = 10;
        s.difficulty = EXPERT;
        scale_stats_by_difficulty_tweaks(&mut s, &env(EXPERT));
        assert_eq!(s.defense, 8, "(int)(10 * 0.8)");
        assert_eq!(s.damage, 6, "10 * 0.6");
        // Skeletron hand (36): master num2 1.3 × 0.85 = 1.105.
        let mut s = stats(36, 1000, 100);
        s.difficulty = MASTER;
        scale_stats_by_difficulty_tweaks(&mut s, &env(MASTER));
        assert_eq!(s.life_max, 1105);
    }

    #[test]
    fn w4_e25_the_player_count_scale_rounds_f64_bankers() {
        // NPC.cs:18719 — the f64 product, banker's round.
        let mut s = stats(4, 2800, 15);
        scale_stats_by_player_count(&mut s, 2);
        assert_eq!(s.life_max, 3780, "2800 * 1.3500000238 = 3780.00007");
        assert_eq!(s.stats_scaled_for_players, 2);
        let mut s = stats(4, 2800, 15);
        scale_stats_by_player_count(&mut s, 3);
        assert_eq!(s.life_max, 5367, "2800 * 1.9166667 = 5366.6667");
        // 1 player: balance 1 → no change, but the carrier is stamped.
        let mut s = stats(4, 2800, 15);
        scale_stats_by_player_count(&mut s, 1);
        assert_eq!(s.life_max, 2800);
        assert_eq!(s.stats_scaled_for_players, 1);
        // The invasion default: 1 + (n-1)*0.2 linear (type 305, group
        // −2, not one of the six skipped moon bosses).
        let mut s = stats(305, 100, 10);
        scale_stats_by_player_count(&mut s, 2);
        assert_eq!(s.life_max, 120);
        // The six moon-boss ids skip the linear arm (315).
        let mut s = stats(315, 100, 10);
        scale_stats_by_player_count(&mut s, 2);
        assert_eq!(s.life_max, 100);
        // OOA lerp 6/7 (type 552): 1000 * (1 + 0.35 * 0.8571428656) =
        // 1300.0000234 → 1300.
        let mut s = stats(552, 1000, 10);
        scale_stats_by_player_count(&mut s, 2);
        assert_eq!(s.life_max, 1300);
        // 471's 2/3 lerp: 1000 * 1.2333333492 = 1233.33 → 1233.
        let mut s = stats(471, 1000, 10);
        scale_stats_by_player_count(&mut s, 2);
        assert_eq!(s.life_max, 1233);
        // 522 is EXCLUDED from the balance arm (:18686 vs :18425).
        let mut s = stats(522, 1000, 10);
        scale_stats_by_player_count(&mut s, 2);
        assert_eq!(s.life_max, 1000);
        // A non-boss non-invasion kind (1) never scales.
        let mut s = stats(1, 100, 10);
        scale_stats_by_player_count(&mut s, 5);
        assert_eq!(s.life_max, 100);
    }

    #[test]
    fn w4_e25_the_brain_creeper_kb_loop_and_the_hungry_arms() {
        // NPC.cs:18576-18584 — the i11 deviation's kb loop: ×0.8 per
        // 0.34 balance step, floored to 0 below 0.1.
        let mut s = stats(267, 100, 10);
        s.knockback_resist = 0.8;
        scale_stats_by_player_count(&mut s, 2);
        // balance 1.35: steps 1.0 and 1.34 both < 1.35 → two ×0.8.
        assert_eq!(s.knockback_resist, 0.5120000243186951f32);
        let mut s = stats(266, 100, 10);
        s.knockback_resist = 0.8;
        scale_stats_by_player_count(&mut s, 3);
        // balance 1.9167: steps 1.0/1.34/1.68 → three ×0.8.
        assert_eq!(s.knockback_resist, 0.40960001945495605f32);
        // The <0.1 floor snaps to 0 mid-loop.
        let mut s = stats(267, 100, 10);
        s.knockback_resist = 0.12;
        scale_stats_by_player_count(&mut s, 3);
        // 0.12 → 0.096 (<0.1) → the next step floors to 0.
        assert_eq!(s.knockback_resist, 0.0);
        // 115/116: >4 players → kb 0; 2 players → kb *= 1 - boost.
        let mut s = stats(115, 100, 10);
        s.knockback_resist = 0.8;
        scale_stats_by_player_count(&mut s, 5);
        assert_eq!(s.knockback_resist, 0.0);
        let mut s = stats(116, 100, 10);
        s.knockback_resist = 0.8;
        scale_stats_by_player_count(&mut s, 2);
        // boost(2) = 0.5666666626930237 → 0.8 * 0.43333334.
        assert_eq!(s.knockback_resist, 0.3466666638851166f32);
    }

    #[test]
    fn w4_e25_the_full_eoc_pipeline_pins() {
        // The whole ScaleStats walk for the Eye of Cthulhu (4), the
        // SetDefaults stats lifeMax 2800 / damage 15 / value 10000.
        let mut s = stats(4, 2800, 15);
        s.boss = true;
        s.value = 10000.0;
        scale_stats(&mut s, &env(CLASSIC), None, None, &[]);
        assert_eq!((s.life_max, s.damage, s.value), (2800, 15, 10000.0));
        assert_eq!(s.life, 2800);
        assert_eq!(s.def_life_max, 2800);
        let mut s = stats(4, 2800, 15);
        s.boss = true;
        s.value = 10000.0;
        scale_stats(&mut s, &env(EXPERT), None, None, &[]);
        // life: 2800*2 = 5600 → ×0.65 = 3640; dmg 15*2 = 30; value
        // 10000*2.5 = 25000; 1 player → balance 1.
        assert_eq!((s.life_max, s.damage, s.value), (3640, 30, 25000.0));
        assert_eq!(s.def_damage, 30);
        let mut s = stats(4, 2800, 15);
        s.boss = true;
        s.value = 10000.0;
        scale_stats(&mut s, &env(MASTER), None, None, &[]);
        // life: 2800*3 = 8400 → ×0.5525 (f32 4641.00008) → 4641.
        assert_eq!((s.life_max, s.damage), (4641, 45));
        // 2-player expert: 3640 * 1.3500000238 = 4914.00009 → 4914.
        let mut s = stats(4, 2800, 15);
        s.boss = true;
        scale_stats(&mut s, &env(EXPERT), Some(2), None, &[]);
        assert_eq!(s.life_max, 4914);
        assert_eq!(s.stats_scaled_for_players, 2);
        // 3-player master: 4641 * 1.9166667 = 8895.25 → 8895.
        let mut s = stats(4, 2800, 15);
        s.boss = true;
        scale_stats(&mut s, &env(MASTER), Some(3), None, &[]);
        assert_eq!(s.life_max, 8895);
        // The strength override wins over the env difficulty.
        let mut s = stats(4, 2800, 15);
        s.boss = true;
        scale_stats(&mut s, &env(CLASSIC), None, Some(MASTER), &[]);
        assert_eq!(s.difficulty, MASTER);
        assert_eq!(s.life_max, 4641);
    }

    #[test]
    fn w4_e25_the_gate_and_the_life_floor() {
        // NPC.cs:18148 — friendly/townNPC/zero-damage/lifeMax<=5 skip
        // unless NeedsExpertScaling carries the type.
        let mut s = stats(1, 25, 7);
        s.friendly = true;
        scale_stats(&mut s, &env(EXPERT), None, None, &[]);
        assert_eq!(s.life_max, 25, "friendly skips the whole pipeline");
        let mut s = stats(1, 25, 7);
        s.town_npc = true;
        scale_stats(&mut s, &env(EXPERT), None, None, &[]);
        assert_eq!(s.life_max, 25);
        let mut s = stats(1, 5, 7);
        scale_stats(&mut s, &env(EXPERT), None, None, &[]);
        assert_eq!(s.life_max, 5, "lifeMax <= 5 skips");
        // NeedsExpertScaling member 261 (FungiSpore, ProjectileNPC):
        // the gate passes on the set; the life/value scale skips; the
        // life floor skips (projectile) — life 1 stays 1.
        let mut s = stats(261, 1, 10);
        scale_stats(&mut s, &env(EXPERT), None, None, &[]);
        assert_eq!(s.life_max, 1);
        assert_eq!(s.damage, 20, "the damage scale still applies");
        // A non-projectile scaled below 6 floors to 6 (:18160-18163).
        let mut s = stats(210, 10, 10);
        s.difficulty = CLASSIC;
        scale_stats(&mut s, &env(JOURNEY), None, None, &[]);
        // journey life ×0.5 = 5, tweaks ×0.6 → 3 → floor 6.
        assert_eq!(s.life_max, 6);
    }

    #[test]
    fn w4_e25_the_zombie_expert_walk_and_the_rng_census() {
        // A plain enemy (zombie, type 3): no Tweaks arm, no
        // ByPlayerCount arm — only the curves. The whole family takes
        // NO rng parameter: determinism by construction (the census's
        // zero-Main.rand claim, pinned by the signatures themselves).
        let mut s = stats(3, 45, 12);
        s.defense = 2;
        s.knockback_resist = 0.5;
        s.value = 100.0;
        scale_stats(&mut s, &env(EXPERT), None, None, &[]);
        assert_eq!(s.life_max, 90);
        assert_eq!(s.damage, 24);
        assert_eq!(s.value, 250.0);
        assert_eq!(s.knockback_resist, 0.44999998807907104f32);
        assert_eq!(s.stats_scaled_for_players, 1, "expert stamps 1p");
        // GetActivePlayerCount: single-player → 1 regardless of slots.
        assert_eq!(get_active_player_count(0, &[(true, false); 8]), 1);
        // netMode 1: the active && !ghost scan.
        let slots = [(true, false), (true, true), (false, false), (true, false)];
        assert_eq!(get_active_player_count(1, &slots), 2);
        assert_eq!(get_balance(1, &slots), 1.350000023841858f32);
        // GetMyBalance reads the spawn-time carrier.
        let mut s = stats(4, 100, 10);
        assert_eq!(get_my_balance(&s), 1.0, "unscaled → 1");
        s.stats_scaled_for_players = 2;
        assert_eq!(get_my_balance(&s), 1.350000023841858f32);
    }

    #[test]
    fn w4_e25_the_invasion_group_table() {
        // NPC.cs:78297-78412.
        assert_eq!(get_npc_invasion_group(26), 1);
        assert_eq!(get_npc_invasion_group(472), 1);
        assert_eq!(get_npc_invasion_group(143), 2);
        assert_eq!(get_npc_invasion_group(216), 3);
        assert_eq!(get_npc_invasion_group(662), 3);
        assert_eq!(get_npc_invasion_group(390), 4);
        assert_eq!(get_npc_invasion_group(338), -1);
        assert_eq!(get_npc_invasion_group(352), -1);
        assert_eq!(get_npc_invasion_group(305), -2);
        assert_eq!(get_npc_invasion_group(330), -2);
        assert_eq!(get_npc_invasion_group(551), -3);
        assert_eq!(get_npc_invasion_group(578), -3);
        assert_eq!(get_npc_invasion_group(1), 0);
        // The ID sets.
        assert!(needs_expert_scaling(261));
        assert!(needs_expert_scaling(398));
        assert!(!needs_expert_scaling(4));
        assert!(dont_do_hardmode_scaling(5));
        assert!(dont_do_hardmode_scaling(522));
        assert!(projectile_npc(25));
        assert!(!projectile_npc(398));
        assert!(belongs_to_invasion_old_ones_army(551));
        assert!(belongs_to_invasion_old_ones_army(578));
        assert!(!belongs_to_invasion_old_ones_army(550));
    }
}
