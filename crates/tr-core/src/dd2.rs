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
/// `DD2Event.SetEnemySpawningOnHold(300)` at `StartInvasion` (`:299`).
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

/// `DD2Event.Difficulty_1_GetRequiredWaveKills` (`:987-1012`) — the wave's
/// kill budget. Wave 5 caps `currentKillCount` at 139 (`:990-992`) and
/// wave 6 is the win sentinel (`:397`'s `return` after `StartVictoryScene`).
pub fn required_wave_kills(wave: i32, current_kills: &mut i32) -> i32 {
    match wave {
        0 => 0,
        1 => 60,
        2 => 80,
        3 => 100,
        4 => 120,
        5 => {
            if *current_kills > 139 {
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

/// `DD2Event.CheckProgress`'s wave-complete test (`:383-391`): the count
/// at or past the wave's budget opens the next wave; the counter's reset
/// and the warp ride the caller.
pub fn wave_budget_passed(wave: i32, kills: &mut f32) -> bool {
    let required = required_wave_kills(wave, &mut (*kills as i32)).max(0) as f32;
    if *kills < 0.0 {
        return false;
    }
    if *kills >= required && required != 0.0 {
        *kills = 0.0;
        true
    } else {
        false
    }
}

/// `DD2Event.GetMonsterPointsWorth` (`:1042-1090`) — the kill's point
/// worth by the slain kind, gated by the event's tier. The port's
/// goblin-family rows first (552-555 the base worth 1, the T2/Bomber
/// rows ride the tier gates).
pub const KILL_WORTH_T1: f32 = 1.0;
pub const KILL_WORTH_T2: f32 = 2.0;
pub const KILL_WORTH_T3: f32 = 4.0;

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
) -> Option<i32> {
    if !ongoing || lost || won || spawn_hold != 0 {
        return None;
    }
    let mut current_kills = wave_kills as i32 + worth as i32;
    let mut current_wave = wave;
    let required = required_wave_kills(wave, &mut current_kills);
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
        // `Difficulty_1_GetRequiredWaveKills` (`:987-1012`) — the tiers
        // 60/80/100/120/140 with the wave-6 win sentinel.
        let mut kills = 0;
        assert_eq!(required_wave_kills(1, &mut kills), 60);
        assert_eq!(required_wave_kills(2, &mut kills), 80);
        assert_eq!(required_wave_kills(3, &mut kills), 100);
        assert_eq!(required_wave_kills(4, &mut kills), 120);
        assert_eq!(required_wave_kills(5, &mut kills), 140);
        let mut kills = 139;
        assert_eq!(required_wave_kills(5, &mut kills), 140);
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
        let started = check_progress(true, false, false, 0, 1, 59.0, 59.0, 1.0);
        assert_eq!(started, Some(1), "the wave completes");
        assert!(check_progress(true, false, false, 0, 1, 59.0, 59.0, 0.5).is_none());
        // The spawn hold's refusal.
        assert!(check_progress(true, false, false, 100, 1, 59.0, 59.0, 1.0).is_none());
        // The lost/won refusals.
        assert!(check_progress(true, true, false, 0, 1, 59.0, 59.0, 1.0).is_none());
        assert!(check_progress(true, false, true, 0, 1, 59.0, 59.0, 1.0).is_none());
        // Not ongoing.
        assert!(check_progress(false, false, false, 0, 1, 59.0, 59.0, 1.0).is_none());
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

/// `DD2Event.Difficulty_2_GetRequiredWaveKills` (`:1162-1195`) — the
/// tier-2 budgets: 60/80/100/120/140/180/220 (the wave-7's 219 cap), the
/// wave-8 win sentinel.
pub fn required_wave_kills_t2(wave: i32, current_kills: &mut i32) -> i32 {
    match wave {
        0 => 0,
        1 => 60,
        2 => 80,
        3 => 100,
        4 => 120,
        5 => 140,
        6 => 180,
        7 => {
            if *current_kills > 219 {
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
/// (`100 - life/max*100`), the wave-8 win sentinel.
pub fn required_wave_kills_t3(wave: i32, current_kills: &mut i32, betsy_hp_ratio: f32) -> i32 {
    match wave {
        0 => 0,
        1 => 60,
        2 => 80,
        3 => 100,
        4 => 120,
        5 => 140,
        6 => 180,
        7 => {
            let mut betsy_found = true;
            let _ = betsy_found;
            *current_kills = (100.0 - betsy_hp_ratio * 100.0).max(0.0) as i32;
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
        // `Difficulty_2_*` (`:1132-1195`): the rosters grow, the budgets
        // climb 60→220 with the wave-7 cap.
        assert_eq!(enemies_for_wave_t2(1), (&[553, 562][..], 90));
        assert_eq!(enemies_for_wave_t2(7).0.len(), 8);
        let mut wave = 1;
        let mut kills = 260;
        assert_eq!(required_wave_kills_t2(1, &mut kills), 60);
        // The wave-7 cap reads the over-219 count down.
        assert_eq!(required_wave_kills_t2(7, &mut kills), 220);
        assert_eq!(kills, 219);
    }

    #[test]
    fn the_tier3_rosters_peak_at_betsy() {
        // `Difficulty_3_*` (`:1437-1500`): the last wave rosters Betsy
        // (551) and the budget's the HP ratio.
        assert_eq!(enemies_for_wave_t3(7).0.last(), Some(&551));
        let mut kills = 0;
        assert_eq!(required_wave_kills_t3(6, &mut kills, 0.4), 180);
        let mut kills = 0;
        assert_eq!(required_wave_kills_t3(7, &mut kills, 0.4), 100);
        assert_eq!(kills, 60, "Betsy's HP bucket");
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
/// (`DD2Event.cs:1311-1312`, `:1355-1356`, `:1688-1689`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LaneSpawn {
    /// The first body — `num7`/`num8`; `None` when the pass picks nothing.
    pub first: Option<u16>,
    /// The second body — `num8`/`num9`, when the branch spawns a pair.
    pub second: Option<u16>,
}

impl LaneSpawn {
    /// A pass that spawns a single body.
    pub fn one(kind: u16) -> Self {
        LaneSpawn {
            first: Some(kind),
            second: None,
        }
    }

    /// A pass that spawns nothing (the C#'s untouched `Main.maxNPCs`).
    pub fn none() -> Self {
        LaneSpawn {
            first: None,
            second: None,
        }
    }

    /// The squire pair: the squire always, the second body on its roll.
    pub fn pair(squire: u16, second: Option<u16>) -> Self {
        LaneSpawn {
            first: Some(squire),
            second,
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
            // `:1077-1098`: the Dark Mage's `if` is its own statement — the
            // chain after it re-assigns `num4` whenever it fires, so the boss
            // only survives when the chain picks nothing.
            LaneSpawn {
                first: chain.or(boss),
                second: None,
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
            // `:1691-1728`: two independent chains — the second (`:1708`)
            // re-assigns `num8` whenever it fires, so it outranks the first.
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
            let second = if rng01() * 7.0 < 1.0 && count(578) < cap_bomb {
                LaneSpawn::one(578)
            } else if rng01() * 7.0 < 1.0 && count(573) + count(575) < cap_mystic {
                // `:1714-1716`: `Next(3) == 0` picks the frost mage's 575.
                LaneSpawn::one(if rng01() * 3.0 < 1.0 { 575 } else { 573 })
            } else if rng01() * 5.0 < 1.0 && count(563) < cap_t3 {
                LaneSpawn::one(563)
            } else if count(554) + count(557) < cap_squire {
                // `:1722-1727`: `Next(3) == 0` drops the 557 beside the 554.
                LaneSpawn::pair(554, roll_second(&rng01, 3.0, 557))
            } else {
                LaneSpawn::none()
            };
            LaneSpawn {
                first: second.first.or(first),
                second: second.second,
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
