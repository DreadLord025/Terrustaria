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
