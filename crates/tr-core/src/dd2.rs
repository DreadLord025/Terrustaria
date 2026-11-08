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
