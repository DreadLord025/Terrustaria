//! The Old One's Army's wave machinery (`GameContent/Events/DD2Event.cs`) —
//! the wave counter, the per-wave kill budgets, the enemy rosters and the
//! lane spawn rates. The event's state lives on `World` (`dd2_*`); this
//! module ports the pure tables and the driver's tick arithmetic.
//!
//! Scope: `difficulty 1`'s tables verbatim (`Difficulty_1_*`); the tier-2
//! and tier-3 rosters/budgets ride `ReadyForTier2/3` (the downed-mech and
//! downed-golem gates) — their rows land with the difficulty slice.

/// `DD2Event._timeLeftUntilSpawningBegins` — the spawn hold's ticks; its
/// being non-zero is `EnemySpawningIsOnHold` (`:113`).
/// `DD2Event.Ongoing` — the Old One's Army event's live flag (the
/// `Projectile.TurretShouldPersist` gate, `Projectile.cs:670-690`: the
/// O'A sentries persist only while the event runs). The event itself is
/// M4-EVENT-5's scope; until it lands the stub reports false — which makes the
/// O'A sentries wipable like the rest (recorded in ANALYSIS).
pub fn event_ongoing() -> bool {
    false
}

pub const SPAWN_HOLD_WAVE_BREAK: i32 = 1800;
/// `DD2Event.SetEnemySpawningOnHold(300)` at `StartInvasion` (`:288`).
pub const SPAWN_HOLD_START: i32 = 300;

/// `DD2Event.Difficulty_1_GetEnemiesForWave` (`:956-985`) — the wave's
/// roster and lane spawn rate (`LaneSpawnRate`).
pub fn enemies_for_wave(wave: i32) -> (&'static [u16], i32) {
    match wave {
        1 => (&[552], 90),
        2 => (&[552, 555], 60),
        3 => (&[552, 555, 561], 55),
        4 => (&[552, 555, 561, 558], 50),
        5 => (&[552, 555, 561, 558, 564], 40),
        _ => (&[552], 60),
    }
}

/// `DD2Event.Difficulty_1_GetRequiredWaveKills` (`:977-1009`) — the wave's
/// kill budget. Wave 5 caps `currentKillCount` at 139 (`:992-996`, unless
/// the Dark Mage is downed) and wave 6 is the win sentinel (`:397`'s
/// `return` after `StartVictoryScene`).
pub fn required_wave_kills(wave: i32, current_kills: &mut i32, dark_mage_t1: bool) -> i32 {
    match wave {
        0 => 0,
        1 => 60,
        2 => 80,
        3 => 100,
        4 => 120,
        5 => {
            if !dark_mage_t1 && *current_kills > 139 {
                *current_kills = 139;
            }
            140
        }
        6 => 1,
        _ => 10,
    }
}

/// The wave-5's `currentKillCount > 139` clamp's value.
pub const WAVE_5_KILL_CAP: i32 = 139;

/// `DD2Event.GetRequiredWaveKills`'s clamp and the worth rows' latches —
/// `DD2Event._downedDarkMageT1` / `_downedOgreT2` (`:61-63`): the wave
/// bosses' deaths lift their wave's kill cap (`:993`, `:1181`). The
/// invasion's start clears them (`:268-269`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DownedLatches {
    pub dark_mage_t1: bool,
    pub ogre_t2: bool,
}

/// `(uint)(slainMonsterID - 551) <= 14u || (uint)(slainMonsterID - 568) <= 10u`
/// (`:1117`): the two Old One's Army id ranges the worth rows score.
fn in_worth_band(slain_id: u16) -> bool {
    (551..=565).contains(&slain_id) || (568..=578).contains(&slain_id)
}

/// `DD2Event.GetMonsterPointsWorth` (`:546-554`) — the kill's worth: the
/// tier bodies at `:1106-1130` (T1), `:1199-1223` (T2), `:1509-1528` (T3).
///
/// `Main.expertMode` is constants-off (the world carries no game mode), so
/// the band's `!expertMode ? 1 : 2` collapses to 1; the bosses' special
/// rows (`waveKills == 138` T1 `:1119`, `== 218` T2 `:1212`) score the same
/// 1 here and only keep an expert run from overshooting its budget.
pub fn monster_points_worth(
    tier: i32,
    wave: i32,
    wave_kills: f32,
    slain_id: u16,
    latches: &mut DownedLatches,
) -> i32 {
    match tier {
        3 => {
            // `:1511-1518`: wave 7 scores Betsy (551) alone.
            if wave == 7 {
                return i32::from(slain_id == 551);
            }
            i32::from(in_worth_band(slain_id))
        }
        2 => {
            // `:1201-1209`: wave 7's 219+ gate waits for the Ogres.
            if wave == 7 && wave_kills >= 219.0 {
                if slain_id == 576 || slain_id == 577 {
                    latches.ogre_t2 = true;
                    return 1;
                }
                return 0;
            }
            i32::from(in_worth_band(slain_id))
        }
        _ => {
            // `:1108-1116`: wave 5's 139+ gate waits for the Dark Mages.
            if wave == 5 && wave_kills >= 139.0 {
                if slain_id == 564 || slain_id == 565 {
                    latches.dark_mage_t1 = true;
                    return 1;
                }
                return 0;
            }
            i32::from(in_worth_band(slain_id))
        }
    }
}

/// `NPC.waveKills` / `totalInvasionPoints`' advance (`:363-370`) — the
/// wave's check runs while the event is on, no wave break is active and
/// nothing is decided yet. Returns the wave number that started (if a
/// wave completed on this kill).
pub fn check_progress(
    ongoing: bool,
    lost: bool,
    won: bool,
    spawn_hold: i32,
    wave: i32,
    wave_kills: f32,
    points: f32,
    worth: f32,
    dark_mage_t1: bool,
) -> Option<i32> {
    if !ongoing || lost || won || spawn_hold != 0 {
        return None;
    }
    let mut current_kills = wave_kills as i32 + worth as i32;
    let mut current_wave = wave;
    let required = required_wave_kills(wave, &mut current_kills, dark_mage_t1);
    if wave_kills + worth >= required as f32 && required != 0 {
        let _ = points;
        Some(current_wave)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wave_budgets_climb() {
        // `Difficulty_1_GetRequiredWaveKills` (`:977-1009`) — the tiers
        // 60/80/100/120/140 with the wave-6 win sentinel.
        let mut kills = 0;
        assert_eq!(required_wave_kills(1, &mut kills, false), 60);
        assert_eq!(required_wave_kills(2, &mut kills, false), 80);
        assert_eq!(required_wave_kills(3, &mut kills, false), 100);
        assert_eq!(required_wave_kills(4, &mut kills, false), 120);
        assert_eq!(required_wave_kills(5, &mut kills, false), 140);
        let mut kills = 139;
        assert_eq!(required_wave_kills(5, &mut kills, false), 140);
        assert_eq!(kills, WAVE_5_KILL_CAP);
    }

    #[test]
    fn the_wave_rosters_grow() {
        // `Difficulty_1_GetEnemiesForWave` (`:956-985`): the roster grows
        // with each wave, the lane spawn rate tightens.
        assert_eq!(enemies_for_wave(1), (&[552][..], 90));
        assert_eq!(enemies_for_wave(2), (&[552, 555][..], 60));
        assert_eq!(enemies_for_wave(3), (&[552, 555, 561][..], 55));
        assert_eq!(enemies_for_wave(4), (&[552, 555, 561, 558][..], 50));
        assert_eq!(enemies_for_wave(5), (&[552, 555, 561, 558, 564][..], 40));
        assert_eq!(enemies_for_wave(6), (&[552][..], 60));
    }

    #[test]
    fn the_kill_progress_advances_the_wave() {
        // `CheckProgress` (`:363-392`): the kills that pass the wave's
        // budget advance it; the hold/lost/won gates refuse.
        let started = check_progress(true, false, false, 0, 1, 59.0, 59.0, 1.0, false);
        assert_eq!(started, Some(1), "the wave completes");
        assert!(check_progress(true, false, false, 0, 1, 59.0, 59.0, 0.5, false).is_none());
        // The spawn hold's refusal.
        assert!(check_progress(true, false, false, 100, 1, 59.0, 59.0, 1.0, false).is_none());
        // The lost/won refusals.
        assert!(check_progress(true, true, false, 0, 1, 59.0, 59.0, 1.0, false).is_none());
        assert!(check_progress(true, false, true, 0, 1, 59.0, 59.0, 1.0, false).is_none());
        // Not ongoing.
        assert!(check_progress(false, false, false, 0, 1, 59.0, 59.0, 1.0, false).is_none());
    }
}

/// `DD2Event.Difficulty_2_GetEnemiesForWave` (`DD2Event.cs:1132-1159`) —
/// the tier-2 rosters (`ReadyForTier2`'s hardmode mech boss gate).
pub fn enemies_for_wave_t2(wave: i32) -> (&'static [u16], i32) {
    match wave {
        1 => (&[553, 562], 90),
        2 => (&[553, 562, 572], 70),
        3 => (&[553, 556, 562, 559, 572], 60),
        4 => (&[553, 559, 570, 572, 562], 55),
        5 => (&[553, 556, 559, 572, 574, 570], 50),
        6 => (&[553, 556, 562, 559, 568, 570, 572, 574], 45),
        7 => (&[553, 556, 572, 559, 568, 574, 570, 576], 42),
        _ => (&[553], 60),
    }
}

/// `DD2Event.Difficulty_3_GetEnemiesForWave` (`DD2Event.cs:1437-1463`) —
/// the tier-3 rosters (`ReadyForTier3`'s hardmode golem gate).
pub fn enemies_for_wave_t3(wave: i32) -> (&'static [u16], i32) {
    match wave {
        1 => (&[554, 557, 563], 85),
        2 => (&[554, 557, 563, 573, 578], 75),
        3 => (&[554, 563, 560, 573, 571], 60),
        4 => (&[554, 560, 571, 573, 563, 575, 565], 60),
        5 => (&[554, 557, 573, 575, 571, 569, 577], 55),
        6 => (&[554, 557, 563, 578, 569, 571, 577, 565], 60),
        7 => (&[554, 557, 563, 569, 571, 551], 90),
        _ => (&[554], 60),
    }
}

/// `DD2Event.Difficulty_2_GetRequiredWaveKills` (`:1162-1197`) — the
/// tier-2 budgets: 60/80/100/120/140/180/220 (the wave-7's 219 cap unless
/// the Ogre is downed, `:1181-1184`), the wave-8 win sentinel.
pub fn required_wave_kills_t2(wave: i32, current_kills: &mut i32, ogre_t2: bool) -> i32 {
    match wave {
        0 => 0,
        1 => 60,
        2 => 80,
        3 => 100,
        4 => 120,
        5 => 140,
        6 => 180,
        7 => {
            if !ogre_t2 && *current_kills > 219 {
                *current_kills = 219;
            }
            220
        }
        8 => 1,
        _ => 10,
    }
}

/// `DD2Event.Difficulty_3_GetRequiredWaveKills` (`:1468-1500`) — the
/// tier-3 budgets: 60/80/100/120/140/180 then the wave-7 Betsy HP bucket
/// (`100 - life/max*100`), the wave-8 win sentinel. `betsy` is the
/// Betsy's live life ratio (`NPC.FindFirstNPC(551)`, `:1487-1494`):
/// `None` — no Betsy on the field — takes the C#'s `return 1` arm.
pub fn required_wave_kills_t3(wave: i32, current_kills: &mut i32, betsy: Option<f32>) -> i32 {
    match wave {
        0 => 0,
        1 => 60,
        2 => 80,
        3 => 100,
        4 => 120,
        5 => 140,
        6 => 180,
        7 => {
            let Some(ratio) = betsy else {
                // `NPC.FindFirstNPC(551) == -1` → the wave's budget is 1.
                return 1;
            };
            *current_kills = (100.0 - ratio * 100.0).max(0.0) as i32;
            100
        }
        8 => 1,
        _ => 10,
    }
}

/// `DD2Event.FindProperDifficulty` (`:358-370`): the tier from the
/// `ReadyForTier2/3` gates (hardmode mech any / golem downed).
pub fn find_tier(hard_mode: bool, downed_mech_any: bool, downed_golem: bool) -> i32 {
    if hard_mode && downed_golem {
        3
    } else if hard_mode && downed_mech_any {
        2
    } else {
        1
    }
}

#[cfg(test)]
mod tier_tests {
    use super::*;

    #[test]
    fn the_tier2_rosters_and_budgets_climb() {
        // `Difficulty_2_*` (`:1132-1197`): the rosters grow, the budgets
        // climb 60→220 with the wave-7 cap.
        assert_eq!(enemies_for_wave_t2(1), (&[553, 562][..], 90));
        assert_eq!(enemies_for_wave_t2(7).0.len(), 8);
        let mut kills = 260;
        assert_eq!(required_wave_kills_t2(1, &mut kills, false), 60);
        // The wave-7 cap reads the over-219 count down.
        assert_eq!(required_wave_kills_t2(7, &mut kills, false), 220);
        assert_eq!(kills, 219);
    }

    #[test]
    fn the_tier3_rosters_peak_at_betsy() {
        // `Difficulty_3_*` (`:1437-1500`): the last wave rosters Betsy
        // (551) and the budget's the HP ratio.
        assert_eq!(enemies_for_wave_t3(7).0.last(), Some(&551));
        let mut kills = 0;
        assert_eq!(required_wave_kills_t3(6, &mut kills, Some(0.4)), 180);
        let mut kills = 0;
        assert_eq!(required_wave_kills_t3(7, &mut kills, Some(0.4)), 100);
        assert_eq!(kills, 60, "Betsy's HP bucket");
        // `NPC.FindFirstNPC(551) == -1` → the wave's budget is 1
        // (`:1489-1492`).
        let mut kills = 0;
        assert_eq!(required_wave_kills_t3(7, &mut kills, None), 1);
    }

    #[test]
    fn the_tier_gates_pick_the_difficulty() {
        // `FindProperDifficulty` (`:358-370`): hardmode alone → tier 1,
        // the mech boss → 2, the golem → 3.
        assert_eq!(find_tier(false, true, true), 1);
        assert_eq!(find_tier(true, false, false), 1);
        assert_eq!(find_tier(true, true, false), 2);
        assert_eq!(find_tier(true, true, true), 3);
    }
}
/// `DD2Event.LaneSpawnRate` (`:1813`, 60) — the gate's spawn interval in
/// frames; the per-wave rates come from `enemies_for_wave`'s second
/// element.
pub const LANE_SPAWN_RATE: i32 = 60;

/// `SpawnMonsterFromGate`'s pick. The C# hands out up to two bodies from a
/// single pass — the `num7`/`num8` (D2) or `num8`/`num9` (D3) pair, where the
/// trailing squire branch spawns the second body alongside the first
/// (`DD2Event.cs:1311-1312`, `:1355-1356`, `:1688-1689`) — and up to THREE
/// in D3 wave 6, whose two independent `if` chains (`:1692-1707`,
/// `:1708-1727`) each spawn, the second chain's squire arm adding its
/// always-spawned `num9` beside the rolled `num8` (B-6e).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LaneSpawn {
    /// The first body — `num7`/`num8`; `None` when the pass picks nothing.
    pub first: Option<u16>,
    /// The second body — `num8`/`num9`, when the branch spawns a pair.
    pub second: Option<u16>,
    /// The third body — only D3 wave 6's second-chain squire arm fills it
    /// (the always-spawned `num9` 554, `DD2Event.cs:1726`); `None` elsewhere.
    pub third: Option<u16>,
}

impl LaneSpawn {
    /// A pass that spawns a single body.
    pub fn one(kind: u16) -> Self {
        LaneSpawn {
            first: Some(kind),
            second: None,
            third: None,
        }
    }

    /// A pass that spawns nothing (the C#'s untouched `Main.maxNPCs`).
    pub fn none() -> Self {
        LaneSpawn {
            first: None,
            second: None,
            third: None,
        }
    }

    /// The squire pair: the squire always, the second body on its roll.
    pub fn pair(squire: u16, second: Option<u16>) -> Self {
        LaneSpawn {
            first: Some(squire),
            second,
            third: None,
        }
    }
}

/// `DD2Event.Difficulty_1_SpawnMonsterFromGate` (`DD2Event.cs:1011-1105`)
/// — the wave's spawn pick: the per-wave `switch (waveNumber)` with its
/// concurrent-live caps and the `Main.rand.Next(N)` rolls.
///
/// `count(type)` stands for `NPC.CountNPCS(type)`, `kill`/`required` for
/// `GetInvasionStatus`'s pair, `rng01` for `Main.rand.NextDouble` (the
/// port's rolls read `< 1/N` where the C# reads `Next(N) == 0`). Only the
/// single-player caps carry over: the C#'s `ActivePlayersCount` loop
/// (`:1025-1030`) scales them by 1.3 per extra player.
pub fn lane_spawn_roll_d1(
    wave: i32,
    count: &dyn Fn(u16) -> i32,
    kill: i32,
    required: i32,
    left_gate: bool,
    rng01: impl Fn() -> f32,
) -> LaneSpawn {
    let goblin_cap = 50;
    let jav_cap = if wave > 4 {
        12
    } else if wave > 3 {
        8
    } else {
        6
    };
    let wyv_cap = if wave > 4 { 8 } else { 6 };
    let alive_goblins = count(552) + count(555);
    match wave {
        1 => {
            if alive_goblins < goblin_cap {
                LaneSpawn::one(552)
            } else {
                LaneSpawn::none()
            }
        }
        2 => {
            // `:1046-1050`: the goblin cap gates the branch, and only the
            // survivor's `Next(7)` consumes a roll.
            if alive_goblins < goblin_cap {
                LaneSpawn::one(if rng01() * 7.0 >= 1.0 { 552 } else { 555 })
            } else {
                LaneSpawn::none()
            }
        }
        3 => {
            if rng01() * 6.0 < 1.0 && count(561) < jav_cap {
                LaneSpawn::one(561)
            } else if alive_goblins < goblin_cap {
                LaneSpawn::one(if rng01() * 5.0 >= 1.0 { 552 } else { 555 })
            } else {
                LaneSpawn::none()
            }
        }
        4 => {
            if rng01() * 12.0 < 1.0 && count(558) < wyv_cap {
                LaneSpawn::one(558)
            } else if rng01() * 5.0 < 1.0 && count(561) < jav_cap {
                LaneSpawn::one(561)
            } else if alive_goblins < goblin_cap {
                LaneSpawn::one(if rng01() * 5.0 >= 1.0 { 552 } else { 555 })
            } else {
                LaneSpawn::none()
            }
        }
        5 => {
            let boss = if (!left_gate || rng01() * 2.0 < 1.0)
                && kill > (required as f32 * 0.5) as i32
                && count(564) == 0
            {
                Some(564)
            } else {
                None
            };
            let chain = if rng01() * 10.0 < 1.0 && count(558) < wyv_cap {
                Some(558)
            } else if rng01() * 4.0 < 1.0 && count(561) < jav_cap {
                Some(561)
            } else if alive_goblins < goblin_cap {
                Some(if rng01() * 4.0 >= 1.0 { 552 } else { 555 })
            } else {
                None
            };
            // `:1078-1093`: the Dark Mage's `if` is its own statement and
            // its `NewNPC` RUNS; the chain after it re-assigns `num4` (the
            // net-sync handle) but spawns its body TOO — both NPCs spawn.
            // B-6d: the old `chain.or(boss)` dropped the mage whenever the
            // chain fired.
            LaneSpawn {
                first: boss,
                second: chain,
                third: None,
            }
        }
        _ => LaneSpawn::one(552),
    }
}

/// `DD2Event.Difficulty_2_SpawnMonsterFromGate` (`DD2Event.cs:1225-1435`).
///
/// `num`/`num2`…`num6` (`:1227-1260`) are the squire, tier-3, archer,
/// wither, flyer and ogre caps; the tier-2 mystic (`571`) reads `num6`.
pub fn lane_spawn_roll_d2(
    wave: i32,
    count: &dyn Fn(u16) -> i32,
    kill: i32,
    required: i32,
    left_gate: bool,
    rng01: impl Fn() -> f32,
) -> LaneSpawn {
    let cap_squire = 50;
    let cap_t3 = if wave > 5 {
        12
    } else if wave > 3 {
        10
    } else if wave > 1 {
        8
    } else {
        5
    };
    let cap_archer = if wave > 4 { 7 } else { 5 };
    let cap_wither = 2;
    let cap_flyer = if wave > 3 { 12 } else { 8 };
    let cap_ogre = if wave > 5 { 5 } else { 3 };
    match wave {
        1 => {
            if rng01() * 20.0 < 1.0 && count(562) < cap_t3 {
                LaneSpawn::one(562)
            } else if count(553) < cap_squire {
                LaneSpawn::one(553)
            } else {
                LaneSpawn::none()
            }
        }
        2 => {
            if rng01() * 3.0 < 1.0 && count(572) < cap_flyer {
                LaneSpawn::one(572)
            } else if rng01() * 8.0 < 1.0 && count(562) < cap_t3 {
                LaneSpawn::one(562)
            } else if count(553) < cap_squire {
                LaneSpawn::one(553)
            } else {
                LaneSpawn::none()
            }
        }
        3 => {
            if rng01() * 7.0 < 1.0 && count(572) < cap_flyer {
                LaneSpawn::one(572)
            } else if rng01() * 10.0 < 1.0 && count(559) < cap_archer {
                LaneSpawn::one(559)
            } else if rng01() * 8.0 < 1.0 && count(562) < cap_t3 {
                LaneSpawn::one(562)
            } else if count(553) + count(556) < cap_squire {
                // `:1309-1312`: `Next(4) == 0` drops the 556 beside the 553.
                LaneSpawn::pair(553, roll_second(&rng01, 4.0, 556))
            } else {
                LaneSpawn::none()
            }
        }
        4 => {
            if rng01() * 10.0 < 1.0 && count(570) < cap_ogre {
                LaneSpawn::one(570)
            } else if rng01() * 12.0 < 1.0 && count(559) < cap_archer {
                LaneSpawn::one(559)
            } else if rng01() * 6.0 < 1.0 && count(562) < cap_t3 {
                LaneSpawn::one(562)
            } else if rng01() * 3.0 < 1.0 && count(572) < cap_flyer {
                LaneSpawn::one(572)
            } else if count(553) < cap_squire {
                LaneSpawn::one(553)
            } else {
                LaneSpawn::none()
            }
        }
        5 => {
            if rng01() * 7.0 < 1.0 && count(570) < cap_ogre {
                LaneSpawn::one(570)
            } else if rng01() * 10.0 < 1.0 && count(559) < cap_archer {
                LaneSpawn::one(559)
            } else if rng01() * 4.0 < 1.0 && count(572) + count(574) < cap_flyer {
                // `:1347-1349`: `Next(2) != 0` picks the hound's 574.
                LaneSpawn::one(if rng01() * 2.0 >= 1.0 { 574 } else { 572 })
            } else if count(553) + count(556) < cap_squire {
                // `:1353-1356`: `Next(3) == 0` drops the 556 beside the 553.
                LaneSpawn::pair(553, roll_second(&rng01, 3.0, 556))
            } else {
                LaneSpawn::none()
            }
        }
        6 => {
            if rng01() * 7.0 < 1.0 && count(570) < cap_ogre {
                LaneSpawn::one(570)
            } else if rng01() * 17.0 < 1.0 && count(568) < cap_wither {
                LaneSpawn::one(568)
            } else if rng01() * 5.0 < 1.0 && count(572) + count(574) < cap_flyer {
                // `:1369-1371`: `Next(2) == 0` picks the hound's 574.
                LaneSpawn::one(if rng01() * 2.0 < 1.0 { 574 } else { 572 })
            } else if rng01() * 9.0 < 1.0 && count(559) < cap_archer {
                LaneSpawn::one(559)
            } else if rng01() * 3.0 < 1.0 && count(562) < cap_t3 {
                LaneSpawn::one(562)
            } else if count(553) + count(556) < cap_squire {
                // `:1381-1384`: `Next(3) != 0` drops the 556 beside the 553.
                LaneSpawn::pair(553, roll_second(&rng01, 3.0, 556))
            } else {
                LaneSpawn::none()
            }
        }
        7 => {
            // `:1393-1396`: the Ogre rides the chain's head, not a separate
            // `if` — its `!leftGate || Next(2) == 0` and the 10% kill gate.
            if (!left_gate || rng01() * 2.0 < 1.0)
                && kill > (required as f32 * 0.1) as i32
                && count(576) == 0
            {
                LaneSpawn::one(576)
            } else if rng01() * 7.0 < 1.0 && count(570) < cap_ogre {
                LaneSpawn::one(570)
            } else if rng01() * 17.0 < 1.0 && count(568) < cap_wither {
                LaneSpawn::one(568)
            } else if rng01() * 7.0 < 1.0 && count(572) + count(574) < cap_flyer {
                // `:1405-1407`: `Next(3) == 0` picks the hound's 574.
                LaneSpawn::one(if rng01() * 3.0 < 1.0 { 574 } else { 572 })
            } else if rng01() * 11.0 < 1.0 && count(559) < cap_archer {
                LaneSpawn::one(559)
            } else if count(553) + count(556) < cap_squire {
                // `:1413-1418`: `Next(2) == 0` drops the 556 beside the 553.
                LaneSpawn::pair(553, roll_second(&rng01, 2.0, 556))
            } else {
                LaneSpawn::none()
            }
        }
        _ => LaneSpawn::one(553),
    }
}

/// `DD2Event.Difficulty_3_SpawnMonsterFromGate` (`DD2Event.cs:1530-1762`).
///
/// `num`/`num2`…`num7` (`:1532-1574`) are the squire, tier-3, archer, drake,
/// mystic, tier-2 mystic and kobold-glider caps.
pub fn lane_spawn_roll_d3(
    wave: i32,
    count: &dyn Fn(u16) -> i32,
    rng01: impl Fn() -> f32,
) -> LaneSpawn {
    let cap_squire = 60;
    let cap_t3 = if wave > 5 {
        15
    } else if wave > 3 {
        12
    } else if wave > 1 {
        9
    } else {
        7
    };
    let cap_archer = if wave > 4 { 10 } else { 7 };
    let cap_drake = if wave > 5 { 3 } else { 2 };
    let cap_mystic = if wave > 3 { 18 } else { 12 };
    let cap_mystic_t2 = if wave > 5 { 6 } else { 4 };
    let cap_bomb = 4;
    match wave {
        1 => {
            if rng01() * 18.0 < 1.0 && count(563) < cap_t3 {
                LaneSpawn::one(563)
            } else if count(554) < cap_squire {
                // `:1586-1592`: `Next(7) == 0` drops the 557 beside the 554.
                LaneSpawn::pair(554, roll_second(&rng01, 7.0, 557))
            } else {
                LaneSpawn::none()
            }
        }
        2 => {
            if rng01() * 3.0 < 1.0 && count(578) < cap_bomb {
                LaneSpawn::one(578)
            } else if rng01() * 7.0 < 1.0 && count(563) < cap_t3 {
                LaneSpawn::one(563)
            } else if rng01() * 3.0 < 1.0 && count(573) < cap_mystic {
                LaneSpawn::one(573)
            } else if count(554) < cap_squire {
                // `:1608-1614`: `Next(4) == 0` drops the 557 beside the 554.
                LaneSpawn::pair(554, roll_second(&rng01, 4.0, 557))
            } else {
                LaneSpawn::none()
            }
        }
        3 => {
            if rng01() * 13.0 < 1.0 && count(571) < cap_mystic_t2 {
                LaneSpawn::one(571)
            } else if rng01() * 7.0 < 1.0 && count(573) < cap_mystic {
                LaneSpawn::one(573)
            } else if rng01() * 10.0 < 1.0 && count(560) < cap_archer {
                LaneSpawn::one(560)
            } else if rng01() * 8.0 < 1.0 && count(563) < cap_t3 {
                LaneSpawn::one(563)
            } else if count(554) < cap_squire {
                LaneSpawn::one(554)
            } else {
                LaneSpawn::none()
            }
        }
        4 => {
            if rng01() * 24.0 < 1.0 && count(565) == 0 {
                LaneSpawn::one(565)
            } else if rng01() * 12.0 < 1.0 && count(571) < cap_mystic_t2 {
                LaneSpawn::one(571)
            } else if rng01() * 15.0 < 1.0 && count(560) < cap_archer {
                LaneSpawn::one(560)
            } else if rng01() * 7.0 < 1.0 && count(563) < cap_t3 {
                LaneSpawn::one(563)
            } else if rng01() * 5.0 < 1.0 && count(573) + count(575) < cap_mystic {
                // `:1656-1658`: `Next(3) == 0` picks the frost mage's 575.
                LaneSpawn::one(if rng01() * 3.0 < 1.0 { 575 } else { 573 })
            } else if count(554) < cap_squire {
                LaneSpawn::one(554)
            } else {
                LaneSpawn::none()
            }
        }
        5 => {
            if rng01() * 20.0 < 1.0 && count(577) == 0 {
                LaneSpawn::one(577)
            } else if rng01() * 17.0 < 1.0 && count(569) < cap_drake {
                LaneSpawn::one(569)
            } else if rng01() * 8.0 < 1.0 && count(571) < cap_mystic_t2 {
                LaneSpawn::one(571)
            } else if rng01() * 7.0 < 1.0 && count(573) + count(575) < cap_mystic {
                // `:1678-1680`: `Next(4) == 0` picks the frost mage's 575.
                LaneSpawn::one(if rng01() * 4.0 < 1.0 { 575 } else { 573 })
            } else if count(554) + count(557) < cap_squire {
                // `:1682-1688`: `Next(3) == 0` drops the 557 beside the 554.
                LaneSpawn::pair(554, roll_second(&rng01, 3.0, 557))
            } else {
                LaneSpawn::none()
            }
        }
        6 => {
            // `:1691-1728`: TWO INDEPENDENT `if` chains — the first
            // (`:1692-1707`, the bosses/drake) runs its `NewNPC`, then the
            // second (`:1708-1727`) runs its OWN; re-assigning `num8` is
            // only the net-sync handle (`:1755-1762`), it does not un-spawn
            // the first chain's body. B-6e: the old `second.first.or(first)`
            // dropped the first chain's body whenever the second fired.
            let first = if rng01() * 20.0 < 1.0 && count(577) == 0 {
                Some(577)
            } else if rng01() * 20.0 < 1.0 && count(565) == 0 {
                Some(565)
            } else if rng01() * 12.0 < 1.0 && count(571) < cap_mystic_t2 {
                Some(571)
            } else if rng01() * 25.0 < 1.0 && count(569) < cap_drake {
                Some(569)
            } else {
                None
            };
            // The second chain: its single-body arms fill `first` (the
            // `num8` slot); its squire arm rolls the 557 into `num8` and
            // ALWAYS spawns the 554 into `num9` (`:1722-1726`) — carried in
            // `third` so the final pick keeps all three bodies in order.
            let second = if rng01() * 7.0 < 1.0 && count(578) < cap_bomb {
                LaneSpawn::one(578)
            } else if rng01() * 7.0 < 1.0 && count(573) + count(575) < cap_mystic {
                // `:1714`: `Next(3) == 0` picks the frost mage's 575.
                LaneSpawn::one(if rng01() * 3.0 < 1.0 { 575 } else { 573 })
            } else if rng01() * 5.0 < 1.0 && count(563) < cap_t3 {
                LaneSpawn::one(563)
            } else if count(554) + count(557) < cap_squire {
                // `:1722-1726`: the `Next(3)` 557 spawns FIRST when it hits,
                // then the 554 squire spawns unconditionally (`num9`).
                LaneSpawn {
                    first: roll_second(&rng01, 3.0, 557),
                    second: None,
                    third: Some(554),
                }
            } else {
                LaneSpawn::none()
            };
            LaneSpawn {
                first,
                second: second.first,
                third: second.third,
            }
        }
        7 => {
            if rng01() * 20.0 < 1.0 && count(571) < cap_mystic_t2 {
                LaneSpawn::one(571)
            } else if rng01() * 17.0 < 1.0 && count(569) < cap_drake {
                LaneSpawn::one(569)
            } else if rng01() * 10.0 < 1.0 && count(563) < cap_t3 {
                LaneSpawn::one(563)
            } else if count(554) + count(557) < cap_squire {
                // `:1742-1747`: `Next(5) == 0` drops the 557 beside the 554.
                LaneSpawn::pair(554, roll_second(&rng01, 5.0, 557))
            } else {
                LaneSpawn::none()
            }
        }
        _ => LaneSpawn::one(554),
    }
}

/// `num9`'s `if (Main.rand.Next(N) == 0)` companion body: the roll the C#'s
/// squire pair spends before it always spawns the squire.
fn roll_second(rng01: &impl Fn() -> f32, n: f32, kind: u16) -> Option<u16> {
    if rng01() * n < 1.0 {
        Some(kind)
    } else {
        None
    }
}

/// `DD2Event.ReportLoss`'s `SetEnemySpawningOnHold(30)` (`DD2Event.cs:514`)
/// — the loss's short spawn hold; its expiry broadcasts the loss message
/// (`UpdateTime`, `:233-236`).
pub const SPAWN_HOLD_LOSS: i32 = 30;

/// `DD2Event.INFO_FAILURE_INVASION_COLOR` (`DD2Event.cs:1812`) —
/// `new Color(255, 0, 0)`: the loss broadcast's red.
pub const INFO_FAILURE_INVASION_COLOR: [u8; 3] = [255, 0, 0];

/// `DD2Event.INFO_START_INVASION_COLOR` (`DD2Event.cs:1811`) —
/// `ChatColors.World` = `new Color(50, 255, 130)` (`ChatColors.cs:24`):
/// the win broadcast's green (`WinInvasionInternal`, `:341`).
pub const INFO_START_INVASION_COLOR: [u8; 3] = [50, 255, 130];

/// `DD2Event.INFO_NEW_WAVE_COLOR` (`DD2Event.cs:1810`) —
/// `new Color(175, 55, 255)`: the wave-complete broadcast's violet
/// (`CheckProgress`, `:403`).
pub const INFO_NEW_WAVE_COLOR: [u8; 3] = [175, 55, 255];

/// `DD2Event.DropMedals`'s item id (`DD2Event.cs:809`) — the Defender
/// Medal (3817), one `numberOfMedals` stack per live Eternia Crystal.
pub const MEDAL_ITEM: u16 = 3817;

/// `DD2Event.ClearAllDD2EnergyCrystalsInChests`'s item id (`:682`) — the
/// Elder Crystal (3822) the wipe turns to air in the open chests.
pub const ENERGY_CRYSTAL_ITEM: u16 = 3822;

/// `NPCID.Sets.BelongsToInvasionOldOnesArmy` (`NPCID.cs:10432`) — the 30
/// ids `ClearAllDD2HostilesInGame` wipes (`DD2Event.cs:651-664`): the
/// crystal (548) and the portals (549) plus the contiguous 551-578 monster
/// band. The bartender (550) is NOT in the set.
pub fn belongs_to_ooa(id: u16) -> bool {
    id == 548 || id == 549 || (551..=578).contains(&id)
}

/// `ProjectileID.Sets.IsADD2Turret` (`ProjectileID.cs:526`) — the 12
/// sentry projectile ids `ClearAllTowersInGame` kills (`DD2Event.cs:640-649`).
pub fn is_dd2_turret(id: u16) -> bool {
    matches!(
        id,
        663 | 665 | 667 | 677 | 678 | 679 | 688 | 689 | 690 | 691 | 692 | 693
    )
}

/// `DD2Event.ReportLoss` (`DD2Event.cs:507-515`) — the crystal's loss
/// scene entry (`NPC.cs:40882`): `LostThisRun = true` and the 30-tick
/// spawn hold. The `_damageTracker.Stop(won: false)` half (`:509-512`)
/// is unported (the port carries no damage tracker — ANALYSIS).
pub fn report_loss(lost: &mut bool, spawn_hold: &mut i32) {
    *lost = true;
    *spawn_hold = SPAWN_HOLD_LOSS;
}

/// `NPC.checkDead`'s crystal intercept (`NPC.cs:78087-78095`):
/// `type == 548 && ai[1] != 1` — the crystal's death does NOT kill it; it
/// flips into the loss scene (`ai[1] = 1`, `ai[0] = 0`, `life = lifeMax`,
/// `dontTakeDamageFromHostiles = true`) and `checkDead` returns. Once the
/// scene runs (`ai[1] == 1`) the guard no longer fires — the 600-tick end
/// (`NPC.cs:40954-40967`) kills it for real.
pub fn crystal_death_starts_loss_scene(ai1: f32) -> bool {
    ai1 != 1.0
}

/// `DD2Event.StopInvasion(win: false)` (`DD2Event.cs:293-312`) — the loss
/// form: only an ongoing event stops; the counters clear
/// (`totalInvasionPoints = 0`, `waveKills = 0`, `waveNumber = 0`,
/// `:305-307`) and `WipeEntities` (`:308`) runs. Returns whether the stop
/// happened (the caller runs the wipe). The `win` arm
/// (`WinInvasionInternal`, `:314-342` — the downed-tier flags + medals)
/// is the victory slice's scope.
pub fn stop_invasion_loss(ongoing: &mut bool, wave: &mut i32, wave_kills: &mut f32) -> bool {
    if !*ongoing {
        return false;
    }
    *ongoing = false;
    *wave_kills = 0.0;
    *wave = 0;
    true
}

#[cfg(test)]
mod worth_tests {
    use super::*;

    #[test]
    fn the_worth_band_is_the_two_family_ranges() {
        // `(uint)(id - 551) <= 14u || (uint)(id - 568) <= 10u` (`:1117`) —
        // 551-565 and 568-578; the skeletons (566/567) sit between them.
        for id in [551, 565, 568, 578] {
            assert!(in_worth_band(id), "{id}");
        }
        for id in [0, 550, 566, 567, 579, 650] {
            assert!(!in_worth_band(id), "{id}");
        }
    }

    #[test]
    fn tier1_scores_the_band_and_latches_the_dark_mage() {
        // `Difficulty_1_GetMonsterPointsWorth` (`:1106-1130`).
        let mut latches = DownedLatches::default();
        assert_eq!(monster_points_worth(1, 1, 0.0, 552, &mut latches), 1);
        assert_eq!(
            monster_points_worth(1, 1, 0.0, 566, &mut latches),
            0,
            "the skeletons are out of band"
        );
        assert_eq!(
            monster_points_worth(1, 4, 200.0, 552, &mut latches),
            1,
            "only wave 5 reads the count"
        );
        assert_eq!(monster_points_worth(1, 5, 137.0, 552, &mut latches), 1);
        assert_eq!(
            monster_points_worth(1, 5, 138.0, 552, &mut latches),
            1,
            "the 138 special case refuses the overshoot"
        );
        assert_eq!(
            monster_points_worth(1, 5, 139.0, 552, &mut latches),
            0,
            "the wave waits for the Dark Mage"
        );
        assert!(!latches.dark_mage_t1);
        assert_eq!(monster_points_worth(1, 5, 139.0, 558, &mut latches), 0);
        assert_eq!(monster_points_worth(1, 5, 139.0, 564, &mut latches), 1);
        assert!(latches.dark_mage_t1, "the Dark Mage's death latches");
        // 565 is the tier-3 Dark Mage; it latches too (`:1110`).
        let mut latches = DownedLatches::default();
        assert_eq!(monster_points_worth(1, 5, 139.0, 565, &mut latches), 1);
        assert!(latches.dark_mage_t1);
    }

    #[test]
    fn tier2_scores_the_band_and_latches_the_ogre() {
        // `Difficulty_2_GetMonsterPointsWorth` (`:1199-1223`).
        let mut latches = DownedLatches::default();
        assert_eq!(monster_points_worth(2, 7, 218.0, 553, &mut latches), 1);
        assert_eq!(
            monster_points_worth(2, 7, 219.0, 553, &mut latches),
            0,
            "the wave waits for the Ogre"
        );
        assert!(!latches.ogre_t2 && !latches.dark_mage_t1);
        assert_eq!(monster_points_worth(2, 7, 219.0, 576, &mut latches), 1);
        assert!(latches.ogre_t2, "the Ogre's death latches");
        let mut latches = DownedLatches::default();
        assert_eq!(monster_points_worth(2, 7, 219.0, 577, &mut latches), 1);
        assert!(latches.ogre_t2);
    }

    #[test]
    fn tier3_pins_the_seventh_wave_to_betsy() {
        // `Difficulty_3_GetMonsterPointsWorth` (`:1509-1528`): at wave 7 only
        // Betsy (551) scores; earlier waves read the shared band.
        let mut latches = DownedLatches::default();
        assert_eq!(monster_points_worth(3, 7, 0.0, 551, &mut latches), 1);
        assert_eq!(monster_points_worth(3, 7, 0.0, 552, &mut latches), 0);
        assert_eq!(monster_points_worth(3, 6, 219.0, 552, &mut latches), 1);
        assert_eq!(monster_points_worth(3, 6, 0.0, 566, &mut latches), 0);
        assert!(
            !latches.ogre_t2 && !latches.dark_mage_t1,
            "tier 3 latches nothing"
        );
    }

    #[test]
    fn the_boss_latches_lift_the_caps() {
        // `:993` / `:1181`: the wave-5/-7 clamps read the latches.
        let mut kills = 200;
        assert_eq!(required_wave_kills(5, &mut kills, false), 140);
        assert_eq!(kills, 139);
        let mut kills = 200;
        assert_eq!(required_wave_kills(5, &mut kills, true), 140);
        assert_eq!(kills, 200, "the latch lifts the cap");
        let mut kills = 260;
        assert_eq!(required_wave_kills_t2(7, &mut kills, false), 220);
        assert_eq!(kills, 219);
        let mut kills = 260;
        assert_eq!(required_wave_kills_t2(7, &mut kills, true), 220);
        assert_eq!(kills, 260);
    }
}

#[cfg(test)]
mod loss_tests {
    use super::*;

    #[test]
    fn report_loss_latches_and_holds_the_spawns_30() {
        // `ReportLoss` (`DD2Event.cs:507-515`): LostThisRun = true,
        // SetEnemySpawningOnHold(30).
        let (mut lost, mut hold) = (false, 1800);
        report_loss(&mut lost, &mut hold);
        assert!(lost);
        assert_eq!(hold, SPAWN_HOLD_LOSS);
        assert_eq!(hold, 30);
    }

    #[test]
    fn the_crystal_death_flips_into_the_loss_scene_once() {
        // `NPC.checkDead`'s intercept (`NPC.cs:78087-78095`): ai[1] != 1
        // → the loss scene; ai[1] == 1 (the scene already running) → the
        // real death passes.
        assert!(crystal_death_starts_loss_scene(0.0));
        assert!(crystal_death_starts_loss_scene(2.0));
        assert!(!crystal_death_starts_loss_scene(1.0));
    }

    #[test]
    fn the_loss_stop_clears_the_counters_only_while_ongoing() {
        // `StopInvasion(win: false)` (`DD2Event.cs:293-312`).
        let (mut ongoing, mut wave, mut kills) = (true, 3, 45.0);
        assert!(stop_invasion_loss(&mut ongoing, &mut wave, &mut kills));
        assert!(!ongoing);
        assert_eq!(wave, 0);
        assert_eq!(kills, 0.0);
        // The second call refuses — `if (DD2Event.Ongoing)` (`:295`).
        let (mut wave, mut kills) = (7, 12.0);
        assert!(!stop_invasion_loss(&mut ongoing, &mut wave, &mut kills));
        assert_eq!(wave, 7, "a stopped event's counters stay");
        assert_eq!(kills, 12.0);
    }

    #[test]
    fn the_wipe_sets_match_the_vanilla_factories() {
        // `NPCID.Sets.BelongsToInvasionOldOnesArmy` (`NPCID.cs:10432`):
        // 548/549 + 551-578 — the bartender 550 is out.
        for id in [548, 549, 551, 564, 566, 567, 578] {
            assert!(belongs_to_ooa(id), "{id}");
        }
        for id in [0, 547, 550, 579, 650] {
            assert!(!belongs_to_ooa(id), "{id}");
        }
        // `ProjectileID.Sets.IsADD2Turret` (`ProjectileID.cs:526`).
        for id in [663, 665, 667, 677, 678, 679, 688, 689, 690, 691, 692, 693] {
            assert!(is_dd2_turret(id), "{id}");
        }
        for id in [662, 664, 666, 668, 672, 676, 680, 687, 694, 713] {
            assert!(!is_dd2_turret(id), "{id}");
        }
        // `ClearAllDD2EnergyCrystalsInChests`'s item (`:682`).
        assert_eq!(ENERGY_CRYSTAL_ITEM, 3822);
        // `INFO_FAILURE_INVASION_COLOR` (`:1812`) — pure red.
        assert_eq!(INFO_FAILURE_INVASION_COLOR, [255, 0, 0]);
    }

    #[test]
    fn the_lost_flag_refuses_the_kill_progress() {
        // `CheckProgress`'s gate (`DD2Event.cs:364`): LostThisRun refuses
        // the whole kill — no worth, no wave advance during the loss scene.
        assert!(check_progress(true, true, false, 0, 1, 59.0, 59.0, 1.0, false).is_none());
    }
}
