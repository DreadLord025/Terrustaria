//! The world clock (plan stage A4): `Main.time`'s day/night wrap, the `TR_DAY`
//! freeze pin and the `Main.Disco*` colour cycle.

use std::sync::OnceLock;

use tr_core::World;

use crate::app::App;

/// `TR_DAY` pins the sky for screenshots; time-of-day advancement pauses.
pub(crate) fn time_frozen() -> bool {
    static FROZEN: OnceLock<bool> = OnceLock::new();
    *FROZEN.get_or_init(|| std::env::var_os("TR_DAY").is_some())
}

/// `Main.time`: a 15-minute day (`0..54000`) and a 9-minute night (`0..32400`) at 60
/// ticks per second, times the dial fast-forward rate.
pub(crate) fn advance_time(world: &mut World, dt: f32) {
    // `Main.cs:64962-64965`: `UpdateTimeRate()` picks the rate, then the full
    // `dt * 60 * dayRate` add lands before the boundary checks read the
    // post-add value. The 0.1 s dt clamp (`frame.rs`) caps the add at 360 —
    // far below the 32400 night length, so one call crosses at most one boundary.
    let rate = f64::from(world.day_rate());
    let mut t = world.time + dt as f64 * 60.0 * rate;
    if world.day_time {
        if t >= 54000.0 {
            // `Main.cs:65148-65151` -> `UpdateTime_StartNight` (`Main.cs:65208`):
            // the moondial's flag clears and its cooldown ticks down before the
            // clock reset (`:65210-65220`). C# also rolls night events and the
            // blood moon here (`:65342-65351` resets both cooldowns on a blood
            // moon) — event rolls land with M4.
            world.fast_forward_to_dusk = false;
            world.moondial_cooldown = world.moondial_cooldown.saturating_sub(1);
            // `Main.cs:65353-65354`: `Main.time = 0.0; Main.dayTime = false;` —
            // the overshoot past the boundary is discarded, not wrapped.
            t = 0.0;
            world.day_time = false;
        }
    } else if t >= 32400.0 {
        // `Main.cs:65137-65140` -> `UpdateTime_StartDay` (`Main.cs:65365`): the
        // sundial's flag clears and its cooldown ticks down (`:65368-65372`,
        // `:65394-65397`). C# rolls the solar eclipse here and force-resets both
        // cooldowns on one (`:65422-65430`) — event rolls land with M4.
        world.fast_forward_to_dawn = false;
        world.sundial_cooldown = world.sundial_cooldown.saturating_sub(1);
        // `Main.cs:65382`/`:65389`: `Main.time = 0.0; Main.dayTime = true;`.
        t = 0.0;
        world.day_time = true;
        // `Main.cs:65398-65401`: `moonPhase++` with the wrap at 8 (`:19982`
        // reads it; the port's u8 range is 0..=7).
        world.moon_phase = if world.moon_phase >= 7 {
            0
        } else {
            world.moon_phase + 1
        };
        // `UpdateTime_StartDay`'s invasion roll (`Main.cs:65439-65462`):
        // the smashed orb arms the goblin rolls; the `StartInvasion`
        // request resolves in the app pass (the players' gate).
        world.roll_morning_invasion((t / 32400.0).fract().abs() as f64 * 0.9999);
    }
    world.time = t;
}

/// `Main.DoUpdate_AnimateDiscoRGB`: step the shared disco-ball colour cycle.
///
/// The original runs its six `if`s sequentially, so a style that completes on
/// this tick also advances the next style in the same tick; that cascade is kept.
pub(crate) fn advance_disco(app: &mut App) {
    const NUM: i32 = 7;
    if app.disco_style == 0 {
        app.disco[1] = (app.disco[1] as i32 + NUM).min(255) as u8;
        if app.disco[1] >= 255 {
            app.disco[1] = 255;
            app.disco_style += 1;
        }
    }
    if app.disco_style == 1 {
        app.disco[0] = (app.disco[0] as i32 - NUM).max(0) as u8;
        if app.disco[0] <= 0 {
            app.disco[0] = 0;
            app.disco_style += 1;
        }
    }
    if app.disco_style == 2 {
        app.disco[2] = (app.disco[2] as i32 + NUM).min(255) as u8;
        if app.disco[2] >= 255 {
            app.disco[2] = 255;
            app.disco_style += 1;
        }
    }
    if app.disco_style == 3 {
        app.disco[1] = (app.disco[1] as i32 - NUM).max(0) as u8;
        if app.disco[1] <= 0 {
            app.disco[1] = 0;
            app.disco_style += 1;
        }
    }
    if app.disco_style == 4 {
        app.disco[0] = (app.disco[0] as i32 + NUM).min(255) as u8;
        if app.disco[0] >= 255 {
            app.disco[0] = 255;
            app.disco_style += 1;
        }
    }
    if app.disco_style == 5 {
        app.disco[2] = (app.disco[2] as i32 - NUM).max(0) as u8;
        if app.disco[2] <= 0 {
            app.disco[2] = 0;
            app.disco_style = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_night_clock_wraps() {
        let mut w = World::new("t".into(), 8, 8);
        w.time = 53999.0;
        w.day_time = true;
        advance_time(&mut w, 1.0);
        assert!(!w.day_time);
        assert!(w.time < 100.0, "time {}", w.time);
        w.time = 32399.0;
        advance_time(&mut w, 1.0);
        assert!(w.day_time);
        assert!(w.time < 100.0, "time {}", w.time);
    }

    #[test]
    fn dw_1_1_ff_to_dawn_crosses_day_boundary() {
        // DW-1.1 (`Main.cs:64962-64965`, `:65137-65140`): armed 360 below the
        // night boundary, the max one-call add (dt 0.1 × 60 × dayRate 60) lands
        // past it, so StartDay runs on this very tick.
        let mut w = World::new("t".into(), 8, 8);
        w.day_time = false;
        w.time = 32040.0;
        w.moon_phase = 3;
        w.fast_forward_to_dawn = true;
        advance_time(&mut w, 0.1);
        assert!(w.day_time, "StartDay ran on the crossing tick");
        // `Main.cs:65382`/`:65389`: `Main.time = 0.0; Main.dayTime = true;` —
        // the overshoot is discarded, not wrapped into the new half.
        assert_eq!(w.time, 0.0);
        // `Main.cs:65398-65401`: the moon phase advances on the dawn tick.
        assert_eq!(w.moon_phase, 4);
    }

    #[test]
    fn dw_1_1_ff_to_dusk_crosses_dusk_boundary() {
        // DW-1.1 mirror (`Main.cs:65148-65151` -> `UpdateTime_StartNight`).
        let mut w = World::new("t".into(), 8, 8);
        w.day_time = true;
        w.time = 53640.0;
        w.moon_phase = 2;
        w.fast_forward_to_dusk = true;
        advance_time(&mut w, 0.1);
        assert!(!w.day_time, "StartNight ran on the crossing tick");
        // `Main.cs:65353-65354`: `Main.time = 0.0; Main.dayTime = false;`.
        assert_eq!(w.time, 0.0);
        // StartNight does not touch the moon phase (only StartDay does).
        assert_eq!(w.moon_phase, 2);
    }

    #[test]
    fn dw_1_2_flag_clears_at_boundary_and_day_rate_restores() {
        // DW-1.2 (`Main.cs:6199-6206`, `:65368-65372`): dayRate is 60 while the
        // flag is up, back to 1 from the transition tick on — the flag is
        // re-read on every `day_rate()` call.
        let mut w = World::new("t".into(), 8, 8);
        w.day_time = false;
        w.time = 32390.0;
        w.fast_forward_to_dawn = true;
        assert!(w.is_fast_forwarding());
        assert_eq!(w.day_rate(), 60);
        advance_time(&mut w, 0.1);
        assert!(
            !w.fast_forward_to_dawn,
            "the flag clears on the transition tick"
        );
        assert!(!w.is_fast_forwarding());
        assert_eq!(w.day_rate(), 1);
        // The next add runs at dayRate 1: a 1 s step advances 60 ticks, not 3600.
        advance_time(&mut w, 1.0);
        assert_eq!(w.time, 60.0);
    }

    #[test]
    fn dw_1_3_cooldowns_decrement_once_per_transition() {
        // DW-1.3 (`Main.cs:65394-65397`, `:65217-65220`): each transition
        // decrements only its own dial's cooldown, exactly once — here on a
        // plain 1× crossing (no fast-forward armed).
        let mut w = World::new("t".into(), 8, 8);
        w.day_time = false;
        w.time = 32395.0;
        w.sundial_cooldown = 3;
        w.moondial_cooldown = 5;
        advance_time(&mut w, 0.1);
        assert_eq!(w.sundial_cooldown, 2, "dawn decrements the sundial");
        assert_eq!(w.moondial_cooldown, 5, "StartDay leaves the moondial alone");
        // The following dusk decrements the moondial only.
        w.day_time = true;
        w.time = 53995.0;
        advance_time(&mut w, 0.1);
        assert_eq!(w.moondial_cooldown, 4, "dusk decrements the moondial");
        assert_eq!(w.sundial_cooldown, 2, "StartNight leaves the sundial alone");
    }

    #[test]
    fn dw_1_3_cooldowns_saturate_at_zero() {
        // DW-1.3: the C# `> 0` guard (`Main.cs:65394-65397`) — a cooldown at 0
        // stays 0 (a bare u8 decrement would wrap to 255).
        let mut w = World::new("t".into(), 8, 8);
        w.day_time = false;
        w.time = 32395.0;
        advance_time(&mut w, 0.1);
        assert_eq!(w.sundial_cooldown, 0);
        w.day_time = true;
        w.time = 53995.0;
        advance_time(&mut w, 0.1);
        assert_eq!(w.moondial_cooldown, 0);
    }

    #[test]
    fn boundary_max_add_crosses_at_most_one_boundary() {
        // Test Plan [Boundary]: the dt clamp is 0.1 s (`frame.rs`), so the max
        // one-call add is 0.1 × 60 × dayRate 60 = 360 — pinned below the
        // 32400 night length (the fast-forward must be armed to reach it).
        let max_add = 0.1_f64 * 60.0 * 60.0;
        assert!(max_add < 32400.0, "one call cannot cross two boundaries");
        // 360 below the night boundary the max add crosses exactly one
        // boundary: the day flips once (a second flip would return to night).
        let mut w = World::new("t".into(), 8, 8);
        w.day_time = false;
        w.time = 32400.0 - 360.0;
        w.fast_forward_to_dawn = true;
        advance_time(&mut w, 0.1);
        assert!(w.day_time, "exactly one transition fired");
        assert_eq!(w.time, 0.0);
        // From the fresh day start (rate back to 1) the next step stays inside
        // the day half — no dusk this call.
        advance_time(&mut w, 0.1);
        assert!(w.day_time);
        assert!(
            (0.0..54000.0).contains(&w.time),
            "no dusk this call: {}",
            w.time
        );
    }
}
