//! The seasonal date gates — `Main.checkXMas` / `Main.checkHalloween` /
//! `Main.isHalloweenDateNow` (`Main.cs:13118-13172`) and
//! `Main.CheckForMoonEventsStartingTemporarySeasons` (`Main.cs:10901-10937`).
//!
//! W4-f39 (the user-flagged season/date gap): the port had NO calendar — the
//! `Main.halloween` / `Main.xMas` statics were stood in by the endless micro
//! seeds alone, so the Oct/Dec windows never opened. This module ports the
//! date decisions as PURE functions over a caller-pinned `(month, day)` (the
//! `FixtureClock` precedent, `tools/WldFixtureHarness/Shims/FixtureClock.cs`:
//! the wall clock is injected, never read inside the decision), so a witness
//! can pin any date. Every function here is draw-free — the C# checks read
//! `DateTime.Now` and compare integers, they consume no `Main.rand`.
//!
//! The force flags: `forceXMasForever` / `forceHalloweenForever` are the
//! `endlessChristmas` / `endlessHalloween` micro seeds (`WorldGen.cs:572-578`
//! → `Main.cs:13135`/`:13168`), which the port already carries as
//! `MicroSeedFlags::ENDLESS_CHRISTMAS` / `ENDLESS_HALLOWEEN`. The `*ForToday`
//! pair is the moon-event temporary season (wave >= 15) — a within-session
//! transient the port keeps runtime-only (the moon event itself is not saved,
//! ANALYSIS "The moon-event wave machinery").

/// `Main.isHalloweenDateNow` (`Main.cs:13141-13155`): the Halloween window is
/// Oct 10-31 plus Nov 1. The C# shape:
/// `if (day < 10 || month != 10) { if (day <= 1) return month == 11; return
/// false; } return true;` — i.e. `(month == 10 && day >= 10) || (month == 11
/// && day <= 1)`. Both edges inclusive (Oct 10 and Nov 1 are in).
pub fn is_halloween_date(month: u32, day: u32) -> bool {
    if day < 10 || month != 10 {
        if day <= 1 {
            return month == 11;
        }
        return false;
    }
    true
}

/// `Main.checkXMas`'s date term (`Main.cs:13120-13130`): `day >= 15 && month
/// == 12` — Dec 15-31, the day edge inclusive.
pub fn is_xmas_date(month: u32, day: u32) -> bool {
    day >= 15 && month == 12
}

/// `Main.checkXMas` (`Main.cs:13118-13139`): the date window OR either force
/// flag. `force_today` is the frost-moon temporary season, `force_forever`
/// the endlessChristmas micro seed. Draw-free.
pub fn check_xmas(month: u32, day: u32, force_today: bool, force_forever: bool) -> bool {
    is_xmas_date(month, day) || force_today || force_forever
}

/// `Main.checkHalloween` (`Main.cs:13157-13172`): the date window OR either
/// force flag. Draw-free.
pub fn check_halloween(month: u32, day: u32, force_today: bool, force_forever: bool) -> bool {
    is_halloween_date(month, day) || force_today || force_forever
}

/// `Main.CheckForMoonEventsStartingTemporarySeasons` (`Main.cs:10901-10937`,
/// the `netMode != 1` arm): re-derive the `*ForToday` pair from the running
/// moon event's wave. The C# clears both first, then raises one per event at
/// `waveNumber >= 15`, then the `*Forever` seed suppresses its own today-flag
/// (the forever flag already keeps the season on, so the broadcast would be
/// redundant). Returns `(force_halloween_for_today, force_xmas_for_today)`.
/// The caller snapshots the moon state BEFORE `stopMoonEvent` clears it (the
/// C# runs this check ahead of `stopMoonEvent`, `Main.cs:65389` vs `:65392`).
/// Draw-free; the change-broadcast is the caller's (UI).
pub fn moon_event_temporary_seasons(
    pumpkin_moon: bool,
    snow_moon: bool,
    wave_number: i32,
    force_halloween_forever: bool,
    force_xmas_forever: bool,
) -> (bool, bool) {
    let mut halloween = false;
    let mut xmas = false;
    if pumpkin_moon && wave_number >= 15 {
        halloween = true;
    }
    if snow_moon && wave_number >= 15 {
        xmas = true;
    }
    if force_halloween_forever {
        halloween = false;
    }
    if force_xmas_forever {
        xmas = false;
    }
    (halloween, xmas)
}

/// Days since 1970-01-01 -> `(year, month, day)` (Howard Hinnant's
/// `civil_from_days`, the same conversion `ui/world_select.rs` uses for the
/// created-date row). Kept here so the runtime date read is self-contained.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (y + (m <= 2) as i64, m as u32, d as u32)
}

/// The runtime wall-clock date (`DateTime.Now`, local time in the C#). The
/// port reads UTC (`SystemTime`) — the date boundary is the same calendar day
/// for the season windows (a few hours' offset never crosses a month edge
/// mid-window). This is the ONLY wall-clock read; every decision above takes
/// the pinned `(month, day)` so tests never touch it. Returns `(month, day)`;
/// an unreadable clock falls back to the Unix epoch (1970-01-01, no season).
pub fn now_month_day() -> (u32, u32) {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let (_, month, day) = civil_from_days(secs / 86_400);
    (month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_halloween_window_edges_are_inclusive() {
        // Oct 10-31 + Nov 1 (`Main.cs:13141-13155`). Oct 9 / Nov 2 are out.
        assert!(!is_halloween_date(10, 9));
        assert!(is_halloween_date(10, 10));
        assert!(is_halloween_date(10, 31));
        assert!(is_halloween_date(11, 1));
        assert!(!is_halloween_date(11, 2));
        // Other months never open, whatever the day.
        assert!(!is_halloween_date(9, 15));
        assert!(!is_halloween_date(12, 1));
        assert!(!is_halloween_date(1, 1));
    }

    #[test]
    fn the_xmas_window_is_dec_15_to_31() {
        // `day >= 15 && month == 12` (`Main.cs:13123`).
        assert!(!is_xmas_date(12, 14));
        assert!(is_xmas_date(12, 15));
        assert!(is_xmas_date(12, 31));
        assert!(!is_xmas_date(1, 1));
        assert!(!is_xmas_date(11, 20));
    }

    #[test]
    fn the_force_flags_override_the_date() {
        // A mid-summer date opens under either force flag.
        assert!(check_halloween(7, 4, true, false));
        assert!(check_halloween(7, 4, false, true));
        assert!(!check_halloween(7, 4, false, false));
        assert!(check_xmas(7, 4, true, false));
        assert!(check_xmas(7, 4, false, true));
        assert!(!check_xmas(7, 4, false, false));
        // The date alone opens without any force.
        assert!(check_halloween(10, 20, false, false));
        assert!(check_xmas(12, 20, false, false));
    }

    #[test]
    fn the_moon_temporary_season_needs_wave_15() {
        // `waveNumber >= 15` (`Main.cs:10911`/`:10915`).
        assert_eq!(moon_event_temporary_seasons(true, false, 14, false, false), (false, false));
        assert_eq!(moon_event_temporary_seasons(true, false, 15, false, false), (true, false));
        assert_eq!(moon_event_temporary_seasons(false, true, 20, false, false), (false, true));
        // No event up: nothing, whatever the wave.
        assert_eq!(moon_event_temporary_seasons(false, false, 20, false, false), (false, false));
        // The forever seed suppresses its own today-flag (`:10919-10926`).
        assert_eq!(moon_event_temporary_seasons(true, false, 15, true, false), (false, false));
        assert_eq!(moon_event_temporary_seasons(false, true, 15, false, true), (false, false));
    }

    #[test]
    fn the_civil_conversion_pins_known_dates() {
        // 2026-10-31 and 2026-12-15 (the window edges) round-trip.
        let days = |y: i64, m: u32, d: u32| -> i64 {
            // days_from_civil (Hinnant inverse) for the fixture pins.
            let y = if m <= 2 { y - 1 } else { y };
            let era = if y >= 0 { y } else { y - 399 } / 400;
            let yoe = y - era * 400;
            let mp = if m > 2 { m - 3 } else { m + 9 } as i64;
            let doy = (153 * mp + 2) / 5 + d as i64 - 1;
            let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
            era * 146_097 + doe - 719_468
        };
        assert_eq!(civil_from_days(days(2026, 10, 31)), (2026, 10, 31));
        assert_eq!(civil_from_days(days(2026, 12, 15)), (2026, 12, 15));
        assert_eq!(civil_from_days(days(2026, 11, 1)), (2026, 11, 1));
        assert_eq!(civil_from_days(0), (1970, 1, 1));
    }
}
