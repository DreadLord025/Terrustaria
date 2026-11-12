//! The world clock (plan stage A4): `Main.time`'s day/night wrap, the `TR_DAY`
//! freeze pin and the `Main.Disco*` colour cycle.

use tr_core::{UnifiedRandom, World};

use crate::app::App;

/// `Main.time`: a 15-minute day (`0..54000`) and a 9-minute night (`0..32400`) at 60
/// ticks per second, times the dial fast-forward rate. Returns the `(blood moon,
/// eclipse)` starts so the host broadcasts them (`ChatColors.World`); `rng` is
/// the `_rngs["UpdateTime"]` stream (`Main.cs:17795`), `any_player_life_over_120`
/// the `statLifeMax > 120` gate (`Main.cs:65342-65349`).
pub(crate) fn advance_time(
    world: &mut World,
    dt: f32,
    rng: &mut UnifiedRandom,
    any_player_life_over_120: bool,
) -> (bool, bool) {
    let mut blood_moon_started = false;
    let mut eclipse_started = false;
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
            // `UpdateTime_StartNight`'s blood-moon roll (`Main.cs:65333-65352`):
            // one `Main.rand` draw at the boundary (`Next(maxValue)` in the C#;
            // the port's uniform-threshold form), through the named `UpdateTime`
            // stream the stage runs under.
            let roll = rng.next_double();
            blood_moon_started =
                world.roll_blood_moon(roll, false, any_player_life_over_120, false);
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
        // `UpdateTime_StartDay`'s `Main.AnglerQuestSwap()`
        // (`Main.cs:65373`) — the day's quest re-roll is the FIRST call
        // in the C#'s StartDay; the port arms the request and the app
        // pass rolls it (the clock carries no RNG).
        world.angler_swap_pending = true;
        // `UpdateTime_StartDay`'s invasion roll (`Main.cs:65439-65462`):
        // the smashed orb arms the goblin rolls; the `StartInvasion`
        // request resolves in the app pass (the players' gate).
        // `UpdateTime_StartDay`'s solar-eclipse roll (`Main.cs:65421-65430`),
        // after the `stopEvents` gate — the port has no stop-events system, so
        // the hardmode + mechs roll always applies.
        // `UpdateTime_StartDay`'s solar-eclipse and invasion rolls draw once at the
        // boundary from the same `UpdateTime` stream (the port keeps the C#'s
        // single-roll approximation for the invasion cluster until M4 ports the
        // per-`Next(n)` chain).
        let roll = rng.next_double();
        eclipse_started = world.roll_eclipse(roll, world.hard_mode, world.downed_mech_boss_any());
        world.roll_morning_invasion(roll);
    }
    world.time = t;
    (blood_moon_started, eclipse_started)
}

/// [`advance_time`] plus the two phase-boundary achievement hooks the C#
/// runs inside `UpdateTime_StartNight`/`UpdateTime_StartDay`:
///
/// * dusk (`:65242-65246`): KILL_THE_SUN (progression 3) when the ending day
///   was an eclipse, then `Main.eclipse = false`;
/// * dawn (`:65383-65387`, `:65415`): BLOODBATH (progression 5) when the
///   ending night was a blood moon, then `Main.bloodMoon = false`, then
///   YOU_CAN_DO_IT (progression 1) on every dawn.
///
/// The C# clears `Main.eclipse` on every `StartNight` and `Main.bloodMoon`
/// on every `StartDay` (`stopMoonEvent` does **not** touch the blood moon),
/// so the clears ride the same boundary branches here.
pub(crate) fn advance_time_with_events(
    app: &mut App,
    dt: f32,
    any_player_life_over_120: bool,
) -> (bool, bool) {
    let eclipse_was_up = app.world.eclipse;
    let blood_moon_was_up = app.world.blood_moon;
    let was_day = app.world.day_time;
    let (blood_moon_started, eclipse_started) = {
        let rng = app.rngs.stream(crate::app::RngKey::UpdateTime);
        advance_time(&mut app.world, dt, rng, any_player_life_over_120)
    };
    if was_day && !app.world.day_time {
        // StartNight: the eclipse row, then the clear.
        if eclipse_was_up {
            let _ = app.achievements.notify_progression(3);
        }
        app.world.eclipse = false;
    } else if !was_day && app.world.day_time {
        // StartDay: the blood-moon row, the clear, then the dawn row.
        if blood_moon_was_up {
            let _ = app.achievements.notify_progression(5);
        }
        app.world.blood_moon = false;
        let _ = app.achievements.notify_progression(1);
    }
    (blood_moon_started, eclipse_started)
}

/// `Main.cs:64995-65003`'s tax pump: while a Tax Collector is alive the
/// player's `taxTimer` counts the dayRate iterations and every
/// `Player.taxRate` (3600) ticks collects the taxes.
pub(crate) fn update_taxes(world: &mut tr_core::World, day_rate: i32) {
    if !world.tax_collector {
        return;
    }
    world.tax_timer += day_rate;
    if world.tax_timer < crate::world_sim::npcs::TAX_RATE {
        return;
    }
    world.tax_timer -= crate::world_sim::npcs::TAX_RATE;
    collect_taxes(world);
}

/// `Player.CollectTaxes` (`Player.cs:24129-24155`): 50 copper per housed,
/// living, non-pet town NPC per collection, capped at 25 gold
/// (`Item.buyPrice(0, 25)`).
pub(crate) fn collect_taxes(world: &mut tr_core::World) {
    if !world.tax_collector || world.tax_money >= crate::world_sim::npcs::TAX_CAP {
        return;
    }
    world.tax_money += 50 * world.tax_payers;
    if world.tax_money > crate::world_sim::npcs::TAX_CAP {
        world.tax_money = crate::world_sim::npcs::TAX_CAP;
    }
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
    #[test]
    fn the_dusk_boundary_rolls_and_reports_the_blood_moon() {
        // The StartNight boundary returns the event so the host broadcasts.
        let mut w = World::new("t".into(), 40, 40);
        w.day_time = true;
        w.time = 53999.0;
        w.sundial_cooldown = 4;
        w.moondial_cooldown = 2;
        // The roll in the 1-in-9 window with a 120+ player.
        let mut rng = stream_with_first_roll(|r| r < 1.0 / 9.0);
        let (blood, eclipse) = advance_time(&mut w, 5.0, &mut rng, true);
        assert!(blood);
        assert!(!eclipse);
        assert!(w.blood_moon);
        assert!(!w.day_time);
        // The blood moon force-resets both dial cooldowns.
        assert_eq!(w.moondial_cooldown, 0);
        assert_eq!(w.sundial_cooldown, 0);
    }

    #[test]
    fn the_dawn_boundary_rolls_and_reports_the_eclipse() {
        let mut w = World::new("t".into(), 40, 40);
        w.day_time = false;
        w.time = 32399.0;
        w.hard_mode = true;
        w.downed_mech_boss1 = true;
        w.sundial_cooldown = 6;
        // The roll in the 1-in-20 window.
        let mut rng = stream_with_first_roll(|r| r < 1.0 / 20.0);
        let (blood, eclipse) = advance_time(&mut w, 5.0, &mut rng, true);
        assert!(!blood);
        assert!(eclipse);
        assert!(w.eclipse);
        assert!(w.day_time);
        assert_eq!(w.sundial_cooldown, 0, "the eclipse force-resets it");
    }

    use super::*;

    /// A stream whose first `NextDouble()` satisfies `pred`. The boundary rolls
    /// draw exactly once (`Main.cs:65333-65352`, `:65421-65430`), so a fresh
    /// `UnifiedRandom` from the found seed reproduces that value.
    fn stream_with_first_roll(pred: impl Fn(f64) -> bool) -> UnifiedRandom {
        for seed in 0..10_000 {
            if pred(UnifiedRandom::new(seed).next_double()) {
                return UnifiedRandom::new(seed);
            }
        }
        panic!("no seed in 0..10_000 has a first roll matching the predicate");
    }

    #[test]
    fn day_night_clock_wraps() {
        let mut w = World::new("t".into(), 8, 8);
        w.time = 53999.0;
        w.day_time = true;
        advance_time(&mut w, 1.0, &mut UnifiedRandom::new(0), false);
        assert!(!w.day_time);
        assert!(w.time < 100.0, "time {}", w.time);
        w.time = 32399.0;
        advance_time(&mut w, 1.0, &mut UnifiedRandom::new(0), false);
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
        advance_time(&mut w, 0.1, &mut UnifiedRandom::new(0), false);
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
        advance_time(&mut w, 0.1, &mut UnifiedRandom::new(0), false);
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
        advance_time(&mut w, 0.1, &mut UnifiedRandom::new(0), false);
        assert!(
            !w.fast_forward_to_dawn,
            "the flag clears on the transition tick"
        );
        assert!(!w.is_fast_forwarding());
        assert_eq!(w.day_rate(), 1);
        // The next add runs at dayRate 1: a 1 s step advances 60 ticks, not 3600.
        advance_time(&mut w, 1.0, &mut UnifiedRandom::new(0), false);
        assert_eq!(w.time, 60.0);
    }

    #[test]
    fn the_all_asleep_flag_runs_the_clock_at_five_times() {
        // `Main.UpdateTimeRate` (`Main.cs:6209-6215`): the all-players-fully-
        // asleep flag multiplies dayRate by 5 — but the dial fast-forward
        // returns first (`:6201-6205`), so 60 outranks 5.
        let mut w = World::new("t".into(), 8, 8);
        assert_eq!(w.day_rate(), 1);
        w.all_players_asleep = true;
        assert_eq!(w.day_rate(), 5);
        w.fast_forward_to_dawn = true;
        assert_eq!(w.day_rate(), 60);
        w.fast_forward_to_dawn = false;
        assert_eq!(w.day_rate(), 5);
        w.all_players_asleep = false;
        assert_eq!(w.day_rate(), 1);
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
        advance_time(&mut w, 0.1, &mut UnifiedRandom::new(0), false);
        assert_eq!(w.sundial_cooldown, 2, "dawn decrements the sundial");
        assert_eq!(w.moondial_cooldown, 5, "StartDay leaves the moondial alone");
        // The following dusk decrements the moondial only.
        w.day_time = true;
        w.time = 53995.0;
        advance_time(&mut w, 0.1, &mut UnifiedRandom::new(0), false);
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
        advance_time(&mut w, 0.1, &mut UnifiedRandom::new(0), false);
        assert_eq!(w.sundial_cooldown, 0);
        w.day_time = true;
        w.time = 53995.0;
        advance_time(&mut w, 0.1, &mut UnifiedRandom::new(0), false);
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
        advance_time(&mut w, 0.1, &mut UnifiedRandom::new(0), false);
        assert!(w.day_time, "exactly one transition fired");
        assert_eq!(w.time, 0.0);
        // From the fresh day start (rate back to 1) the next step stays inside
        // the day half — no dusk this call.
        advance_time(&mut w, 0.1, &mut UnifiedRandom::new(0), false);
        assert!(w.day_time);
        assert!(
            (0.0..54000.0).contains(&w.time),
            "no dusk this call: {}",
            w.time
        );
    }

    /// S1-E-18: the phase-boundary achievements. `UpdateTime_StartNight`
    /// fires KILL_THE_SUN when the ending night overlapped an eclipse and
    /// clears `Main.eclipse` (`Main.cs:65242-65246`); `UpdateTime_StartDay`
    /// fires BLOODBATH on a blood-moon night and YOU_CAN_DO_IT every dawn
    /// (`:65383-65387`, `:65415`).
    #[test]
    fn the_boundary_crossings_fire_the_event_rows_and_clear_the_flags() {
        let done = |app: &crate::app::App, name: &str| {
            app.achievements
                .list
                .iter()
                .any(|a| a.name == name && a.completed)
        };
        // A plain dawn fires the night-survived row alone.
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.world.day_time = false;
        app.world.time = 32399.0;
        let (blood, eclipse) = advance_time_with_events(&mut app, 5.0, false);
        assert!(!blood && !eclipse);
        assert!(done(&app, "YOU_CAN_DO_IT"), "progression 1 at every dawn");
        assert!(!done(&app, "BLOODBATH"));
        assert!(!done(&app, "KILL_THE_SUN"));

        // A blood-moon night ending at dawn fires BLOODBATH and clears the flag.
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.world.day_time = false;
        app.world.time = 32399.0;
        app.world.blood_moon = true;
        advance_time_with_events(&mut app, 5.0, false);
        assert!(app.world.day_time);
        assert!(done(&app, "BLOODBATH"), "progression 5 at the blood-moon dawn");
        assert!(!app.world.blood_moon, "StartDay clears the flag");

        // A day ending with an eclipse up fires KILL_THE_SUN and clears it.
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.world.day_time = true;
        app.world.time = 53999.0;
        app.world.eclipse = true;
        advance_time_with_events(&mut app, 5.0, false);
        assert!(!app.world.day_time);
        assert!(done(&app, "KILL_THE_SUN"), "progression 3 at the eclipse dusk");
        assert!(!app.world.eclipse, "StartNight clears the flag");

        // A plain dusk fires neither row.
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.world.day_time = true;
        app.world.time = 53999.0;
        advance_time_with_events(&mut app, 5.0, false);
        assert!(!done(&app, "KILL_THE_SUN"));
        assert!(!done(&app, "BLOODBATH"));
    }
}
