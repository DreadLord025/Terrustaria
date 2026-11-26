//! The world clock (plan stage A4): `Main.time`'s day/night wrap, the `TR_DAY`
//! freeze pin and the `Main.Disco*` colour cycle.

use tr_core::{UnifiedRandom, World};

use crate::app::App;

/// `Main.time`: a 15-minute day (`0..54000`) and a 9-minute night (`0..32400`) at 60
/// ticks per second, times the dial fast-forward rate. Returns the `(blood moon,
/// eclipse)` starts so the host broadcasts them (`ChatColors.World`); `rng` is
/// the `_rngs["UpdateTime"]` stream (`Main.cs:17795`), `any_player_life_over_120`
/// the `statLifeMax > 120` gate (`Main.cs:65342-65349`).
///
/// W4-f39: `date` is the caller-pinned wall-clock `(month, day)` the dawn
/// season checks read (`Main.checkXMas`/`checkHalloween`, `Main.cs:65390-65391`)
/// — the `FixtureClock` seam: the runtime passes `season::now_month_day()`,
/// a witness pins any date. The season re-derivation is draw-free.
pub(crate) fn advance_time(
    world: &mut World,
    dt: f32,
    rng: &mut UnifiedRandom,
    any_player_life_over_120: bool,
    date: (u32, u32),
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
        // `Main.cs:65145-65148` (the `UpdateTime` day branch, beside the
        // blood-moon clear): `stopMoonEvent` runs before the dusk check — the
        // port carried the blood moon's clear to the dawn boundary
        // (`advance_time_with_events`), so the day branch stops the moon
        // event alone. No-op unless a flag is up during the day.
        world.stop_moon_event();
        // `Main.cs:65148`: `if (Main.time > 54000.0)` — STRICT, so a clock
        // resting exactly on 54000.0 is still the day.
        if t > 54000.0 {
            // `Main.cs:65148-65151` -> `UpdateTime_StartNight` (`Main.cs:65208`):
            // the moondial's flag clears and its cooldown ticks down before the
            // clock reset (`:65210-65220`). C# also rolls night events and the
            // blood moon here (`:65342-65351` resets both cooldowns on a blood
            // moon) — event rolls land with M4.
            world.fast_forward_to_dusk = false;
            world.moondial_cooldown = world.moondial_cooldown.saturating_sub(1);
            // `UpdateTime_StartNight`'s `NPC.ResetBadgerHatTime()`
            // (`Main.cs:65221`, W4-f35): the two KilledToday latches
            // clear at dusk (`NPC.cs:78939-78943`).
            // `NPC.ResetBadgerHatTime()` (`Main.cs:65221` → `NPC.cs:78939-
            // 78943`): the daily EoC/WoF death latches clear at dusk (the
            // Badger's Hat needs both in ONE day).
            world.eoc_killed_today = false;
            world.wof_killed_today = false;
            // `Star.NightSetup` (`Main.cs:65223` → `Star.cs:41-60`): the
            // starfall-boost roll runs BEFORE the blood-moon roll in the
            // C# StartNight (W4-d15 final — the growth pass's starfall
            // gate consumes `world.starfall_boost`). 1-3 `UpdateTime`
            // draws per dusk (the gate + the taken band's range roll).
            let tenth = world.seeds.contains(tr_core::seeds::SeedFlags::TENTH_ANNIVERSARY);
            world.roll_starfall_night_setup(tenth, rng);
            // `UpdateTime_StartNight`'s blood-moon roll (`Main.cs:65333-65352`):
            // one `Main.rand` draw at the boundary (`Next(maxValue)` in the C#;
            // the port's uniform-threshold form), through the named `UpdateTime`
            // stream the stage runs under. W4-g70 lane 8: the tenth-anniversary
            // flag (`:65334-65337`) is live now — 1-in-6 on the seed.
            let roll = rng.next_double();
            // W4-f39: `Main.cs:65334-65337` — the anniversary world widens
            // the blood-moon window to 1-in-6 (`maxValue2 = 6`). The roll
            // itself is the same single `NextDouble`; only the threshold
            // moves, so the stream position is unchanged.
            let anniversary = world.seeds.contains(tr_core::seeds::SeedFlags::TENTH_ANNIVERSARY);
            blood_moon_started =
                world.roll_blood_moon(roll, anniversary, any_player_life_over_120, false);
            // `Main.cs:65353-65354`: `Main.time = 0.0; Main.dayTime = false;` —
            // the overshoot past the boundary is discarded, not wrapped.
            t = 0.0;
            world.day_time = false;
        }
    // `Main.cs:65137`: `if (Main.time > 32400.0)` — STRICT the other way too.
    } else if t > 32400.0 {
        // `Main.cs:65137-65140` -> `UpdateTime_StartDay` (`Main.cs:65365`): the
        // sundial's flag clears and its cooldown ticks down (`:65368-65372`,
        // `:65394-65397`). C# rolls the solar eclipse here and force-resets both
        // cooldowns on one (`:65422-65430`) — event rolls land with M4.
        world.fast_forward_to_dawn = false;
        world.sundial_cooldown = world.sundial_cooldown.saturating_sub(1);
        // W4-f39: `UpdateTime_StartDay`'s season trio (`Main.cs:65389-65391`)
        // — `CheckForMoonEventsStartingTemporarySeasons()` + `checkXMas()` +
        // `checkHalloween()`, BEFORE `stopMoonEvent` (`:65392`) clears the
        // moon state the temporary-season check reads. Draw-free; the date
        // is the caller-pinned seam.
        world.refresh_seasons(date.0, date.1);
        // `Main.cs:65382`/`:65389`: `Main.time = 0.0; Main.dayTime = true;`.
        // `UpdateTime_StartDay`'s `Main.stopMoonEvent()` (`Main.cs:65392`):
        // the moon events end at dawn — the flag clears are unconditional,
        // the point/wave resets are single-player (both run here).
        world.stop_moon_event();
        t = 0.0;
        world.day_time = true;
        // `Main.cs:65398-65401`: `moonPhase++` with the wrap at 8 (`:19982`
        // reads it; the port's u8 range is 0..=7).
        world.moon_phase = if world.moon_phase >= 7 {
            0
        } else {
            world.moon_phase + 1
        };
        // `Main.cs:65403-65406`: the drunk world flips the evil flavour at
        // EVERY dawn (`WorldGen.crimson = !WorldGen.crimson`) — W4-g69
        // doc-sweep: `World.crimson` is live now, so the flip lands. No RNG
        // draw; the `netMode != 1` guard is the single-player session.
        if world.seeds.contains(tr_core::seeds::SeedFlags::DRUNK) {
            world.crimson = !world.crimson;
        }
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
    // W4-f39: the runtime wall-clock date for the dawn season checks — read
    // ONCE per call (the C# reads `DateTime.Now` inside each check; the port
    // snapshots it at the boundary, the `FixtureClock` seam's runtime arm).
    let date = tr_core::season::now_month_day();
    let (blood_moon_started, eclipse_started) = {
        let rng = app.rngs.stream(crate::app::RngKey::UpdateTime);
        advance_time(&mut app.world, dt, rng, any_player_life_over_120, date)
    };
    // The eclipse/blood-moon farewell rows ride before the scheduler block:
    // the RNG borrow below (`UpdateTime`'s stream) is exclusive, so the
    // achievement fan-out (which needs `&mut App`) resolves first.
    if was_day && !app.world.day_time && eclipse_was_up {
        let _ = app.notify_progression_net(3);
    }
    // W4-c36 — `UpdateTime_StartNight`'s natural blood-moon start fires
    // progression 4 beside the flag set (`Main.cs:65348-65353`: the
    // NotifyProgressionEvent + the Lang.misc[8] broadcast the caller rides).
    // No shipped row listens on id 4 (`AchievementInitializer` never calls
    // `Create(4)`) — the wire is the faithful no-op, like the C#'s.
    if blood_moon_started {
        let _ = app.notify_progression_net(4);
    }
    if !was_day && app.world.day_time {
        if blood_moon_was_up {
            let _ = app.notify_progression_net(5);
        }
        // `Main.cs:65374`'s dawn row.
        let _ = app.notify_progression_net(1);
        // W4-c36 — `UpdateTime_StartDay`'s natural eclipse start fires
        // progression 2 right after the flag set (`Main.cs:65420-65425`),
        // before the Lang.misc[20] broadcast. No shipped row listens on
        // id 2 either — the faithful no-op wire.
        if eclipse_started {
            let _ = app.notify_progression_net(2);
        }
    }
    // The event schedulers' draws ride the same UpdateTime stream the
    // boundary rolls consumed (S1-A-10: one stream, the C#'s
    // `UpdateWorld_Time` order).
    let rng = app.rngs.stream(crate::app::RngKey::UpdateTime);
    if was_day && !app.world.day_time {
        // StartNight: the eclipse row cleared above, then the world clear.
        app.world.eclipse = false;
        // `Main.cs:65222` — `NPC.freeCake = false` on EVERY dusk (W4-d120):
        // the cake gift only lives for the party day it was earned on; the
        // FreeCakeDialogue's talk arm (`world_sim/npcs.rs`) consumes it
        // before then, else the latch dies here.
        // `Main.cs:65222` — `NPC.freeCake = false` at every dusk (ahead of
        // the event pair, beside `ResetBadgerHatTime`).
        app.world_events.free_cake = false;
        // `Star.NightSetup()` (`Main.cs:65223`, W4-g70 lane 8): the night's
        // starfall boost rolls on the same UpdateTime stream, ahead of the
        // event pair below (the C# order: NightSetup :65223 < setFireFlyChance
        // :65224 < BirthdayParty.CheckNight :65225 < LanternNight :65226).
        let anniversary = app
            .world
            .seeds
            .contains(tr_core::seeds::SeedFlags::TENTH_ANNIVERSARY);
        app.stars.night_setup(rng, anniversary);
        // `Main.cs:65222`: `NPC.freeCake = false` — BEFORE the event pair
        // (`:65225-65226`), so the cake condition dies with the dusk even
        // when no party was up to broadcast the farewell.
        app.world_events.free_cake = false;
        // `UpdateTime_StartNight`'s event pair (`Main.cs:65225-65226`):
        // `BirthdayParty.CheckNight()` then `LanternNight.CheckNight()` —
        // S1-F-14. W4-f41: `NPC.freeCake = false` (`Main.cs:65222`) runs
        // BEFORE the pair — an unclaimed cake gift dies with the day.
        app.world_events.free_cake = false;
        let gates = crate::world_events::LanternGates::from_world(&app.world, &app.npcs);
        let party_girl = app.npcs.iter().any(|n| {
            n.hp > 0 && n.kind == crate::npc::NpcKind::PartyGirl
        });
        crate::world_events::check_night_party(
            &mut app.world_events,
            &mut app.chat,
        );
        crate::world_events::check_night_lanterns(
            &mut app.world_events,
            &gates,
            app.world.downed_moon_lord,
            &mut |lo, hi| rng.next_range(lo, hi),
        );
    } else if !was_day && app.world.day_time {
        // StartDay: the blood-moon row cleared above, then the world clear.
        app.world.blood_moon = false;
        // `UpdateTime_StartDay`'s event pair (`Main.cs:65374-65375`):
        // `BirthdayParty.CheckMorning()` then `LanternNight.CheckMorning()`.
        let party_girl = app.npcs.iter().any(|n| {
            n.hp > 0 && n.kind == crate::npc::NpcKind::PartyGirl
        });
        let partyable: Vec<bool> = app
            .npcs
            .iter()
            .map(crate::world_events::can_npc_party)
            .collect();
        let party_up = crate::world_events::check_morning_party(
            &mut app.world_events,
            party_girl,
            &partyable,
            app.world.seeds.contains(tr_core::seeds::SeedFlags::TENTH_ANNIVERSARY),
            &mut |lo, hi| rng.next_range(lo, hi),
        );
        // `Main.cs:65375` → `BirthdayParty.CheckMorning` → `:141`'s
        // `CheckForAchievement`: a natural party coming up credits
        // THROW_A_PARTY (25).
        if party_up {
            // `BirthdayParty.cs:127-139`: the start broadcast —
            // `Game.BirthdayParty_{n}` over the celebrants'
            // `GetGivenOrTypeNetName`, in `new Color(255, 0, 160)`, BEFORE
            // the `SendData(7)`/`CheckForAchievement` tail.
            let names: Vec<String> = app
                .world_events
                .celebrating_npcs
                .iter()
                .filter_map(|&i| app.npcs.get(i))
                .map(crate::npc_names::given_or_type_name)
                .collect();
            if !names.is_empty() {
                let key = format!("Game.BirthdayParty_{}", names.len());
                let text = crate::lang::text_args(&key, &names);
                crate::chat::broadcast_death_text(&mut app.chat, &text, [255, 0, 160]);
            }
            let _ = app.notify_progression_net(25);
            // `BirthdayParty.cs:127-139` — the `Game.BirthdayParty_1/2/3`
            // broadcast names the celebrants (`GetGivenOrTypeNetName`),
            // `new Color(255, 0, 160)`.
            let names: Vec<String> = app
                .world_events
                .celebrating_npcs
                .iter()
                .filter_map(|&i| app.npcs.get(i))
                .map(|n| {
                    if n.given_name.is_empty() {
                        crate::lang::npc_name(n.kind.net_id())
                    } else {
                        n.given_name.clone()
                    }
                })
                .collect();
            let key = format!("Game.BirthdayParty_{}", names.len());
            crate::chat::broadcast_death_text(
                &mut app.chat,
                &crate::lang::text_args(&key, &names),
                [255, 0, 160],
            );
        }
        crate::world_events::check_morning_lanterns(&mut app.world_events);
    }
    // W4-g52 audit — the boundary `SendData(7)` re-pushes
    // (`UpdateTime_StartNight` `Main.cs:65358-65361`, `UpdateTime_StartDay`
    // `:65406-65409`, both `netMode == 2` only): the header carries the new
    // day flag + the cleared event flags to every client. The send rides
    // AFTER the eclipse/blood-moon clears above — the C# `stopMoonEvent`
    // (`:65246`/`:65392`) runs before its `SendData(7)`, so a stale event
    // flag must never ride the boundary header (the dusk witness pins it).
    if was_day != app.world.day_time {
        app.net_world_repush();
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
/// (`Item.buyPrice(0, 25)`). On the tenth-anniversary world BOTH the
/// per-payer rate and the cap double (`:24133-24137`:
/// `num2 *= 2; num *= 2;` under `Main.tenthAnniversaryWorld` — the port's
/// `SeedFlags::TENTH_ANNIVERSARY`, the `world_sim/editing.rs:756` feed).
/// The `>= cap` early return (`:24138`) reads the DOUBLED cap, and the
/// post-add clamp (`:24151-24153`) is the strict `>` compare.
pub(crate) fn collect_taxes(world: &mut tr_core::World) {
    // `:24131-24137` — the rate/cap pair, doubled on the anniversary seed.
    let (mut rate, mut cap) = (50, crate::world_sim::npcs::TAX_CAP);
    if world
        .seeds
        .contains(tr_core::seeds::SeedFlags::TENTH_ANNIVERSARY)
    {
        cap *= 2;
        rate *= 2;
    }
    if !world.tax_collector || world.tax_money >= cap {
        return;
    }
    world.tax_money += rate * world.tax_payers;
    if world.tax_money > cap {
        world.tax_money = cap;
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
    /// W4-g69 doc-sweep — `Main.cs:65403-65406`: the drunk world flips
    /// `WorldGen.crimson` at every dawn; a classic world never flips.
    #[test]
    fn w4_g69_the_drunk_dawn_flips_the_world_evil() {
        let mut w = World::new("t".into(), 40, 40);
        w.seeds.insert(tr_core::seeds::SeedFlags::DRUNK);
        w.day_time = false;
        w.time = 32399.0;
        w.crimson = false;
        let mut rng = UnifiedRandom::new(1);
        advance_time(&mut w, 5.0, &mut rng, false, (6, 1));
        assert!(w.day_time, "the dawn boundary ran");
        assert!(w.crimson, "the drunk dawn flipped corruption -> crimson");
        // The next dawn flips back.
        w.day_time = false;
        w.time = 32399.0;
        advance_time(&mut w, 5.0, &mut rng, false, (6, 1));
        assert!(!w.crimson, "the second dawn flips back");
        // A classic world keeps its flavour across the same boundary.
        let mut c = World::new("t".into(), 40, 40);
        c.day_time = false;
        c.time = 32399.0;
        c.crimson = true;
        advance_time(&mut c, 5.0, &mut rng, false, (6, 1));
        assert!(c.day_time);
        assert!(c.crimson, "no flip without the drunk seed");
    }

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
        let (blood, eclipse) = advance_time(&mut w, 5.0, &mut rng, true, (6, 1));
        // The roll in the 1-in-9 window with a 120+ player (past the
        // NightSetup prefix — W4-d15 final).
        let mut rng = stream_with_dusk_roll(|r| r < 1.0 / 9.0);
        let (blood, eclipse) = advance_time(&mut w, 5.0, &mut rng, true, (6, 1));
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
        let (blood, eclipse) = advance_time(&mut w, 5.0, &mut rng, true, (6, 1));
        assert!(!blood);
        assert!(eclipse);
        assert!(w.eclipse);
        assert!(w.day_time);
        assert_eq!(w.sundial_cooldown, 0, "the eclipse force-resets it");
    }

    use super::*;

    // --- S1-F-13 Phase 1: the moon events stop at both boundaries ---

    /// DW-1.2: the dawn boundary (`UpdateTime_StartDay`'s `stopMoonEvent`,
    /// `Main.cs:65392`) stops the running event and zeroes the points/waves.
    #[test]
    fn the_dawn_boundary_stops_the_moon_event() {
        let mut w = World::new("t".into(), 40, 40);
        w.day_time = false;
        w.time = 32399.0;
        w.start_snow_moon();
        w.report_frost_moon_kill(344);
        w.wave_number = 5;
        let mut rng = UnifiedRandom::new(1);
        advance_time(&mut w, 5.0, &mut rng, false, (6, 1));
        assert!(w.day_time, "the dawn transition ran");
        assert!(
            !w.snow_moon && !w.pumpkin_moon,
            "Main.cs:65392 stops the event"
        );
        assert_eq!(
            (w.wave_number, w.wave_kills, w.total_invasion_points),
            (0, 0.0, 0.0),
            "the point/wave resets are single-player and both run"
        );
    }

    /// DW-1.2: the dusk side — `Main.cs:65147-65148`'s day-branch
    /// `stopMoonEvent`, which runs beside the blood-moon clear and before the
    /// `time > 54000` boundary that turns night.
    #[test]
    fn the_day_branch_stops_the_moon_event_at_the_dusk_boundary() {
        let mut w = World::new("t".into(), 40, 40);
        w.day_time = true;
        w.time = 53999.0;
        w.start_pumpkin_moon();
        w.wave_kills = 60.0;
        w.total_invasion_points = 210.0;
        w.wave_number = 4;
        let mut rng = UnifiedRandom::new(1);
        advance_time(&mut w, 5.0, &mut rng, false, (6, 1));
        assert!(!w.day_time, "dusk ran");
        assert!(
            !w.pumpkin_moon && !w.snow_moon,
            "Main.cs:65147-65148 stops the event at the dusk boundary"
        );
        assert_eq!(
            (w.wave_number, w.wave_kills, w.total_invasion_points),
            (0, 0.0, 0.0)
        );
        // The stop is a no-op while no event runs: a night that crosses dawn
        // with no event leaves the world untouched by the machinery.
        let mut w = World::new("t".into(), 40, 40);
        w.day_time = false;
        w.time = 32399.0;
        let mut rng = UnifiedRandom::new(1);
        advance_time(&mut w, 5.0, &mut rng, false, (6, 1));
        assert_eq!(
            (w.wave_number, w.wave_kills, w.total_invasion_points),
            (0, 0.0, 0.0)
        );
    }

    /// A stream whose first `NextDouble()` satisfies `pred` — the DAWN
    /// boundary's roll draws exactly once with no prefix
    /// (`Main.cs:65421-65430`), so a fresh `UnifiedRandom` from the
    /// found seed reproduces that value.
    fn stream_with_first_roll(pred: impl Fn(f64) -> bool) -> UnifiedRandom {
        for seed in 0..10_000 {
            if pred(UnifiedRandom::new(seed).next_double()) {
                return UnifiedRandom::new(seed);
            }
        }
        panic!("no seed in 0..10_000 has a first roll matching the predicate");
    }

    /// The DUSK twin: the boundary's blood-moon draw sits AFTER the
    /// `Star.NightSetup` prefix (`Main.cs:65223` — 1-3 draws, W4-d15
    /// final). The probe replays NightSetup to find a seed whose
    /// POST-prefix draw satisfies `pred`; the returned stream is FRESH
    /// (`advance_time` replays the prefix itself — the classic gates,
    /// the fixtures carry no tenth-anniversary seed).
    fn stream_with_dusk_roll(pred: impl Fn(f64) -> bool) -> UnifiedRandom {
        for seed in 0..10_000 {
            let mut r = UnifiedRandom::new(seed);
            let mut w = World::new("probe".into(), 8, 8);
            w.roll_starfall_night_setup(false, &mut r);
            if pred(r.next_double()) {
                return UnifiedRandom::new(seed);
            }
        }
        panic!("no seed in 0..10_000 has a post-NightSetup roll matching the predicate");
    }

    #[test]
    fn day_night_clock_wraps() {
        let mut w = World::new("t".into(), 8, 8);
        w.time = 53999.0;
        w.day_time = true;
        advance_time(&mut w, 1.0, &mut UnifiedRandom::new(0), false, (6, 1));
        assert!(!w.day_time);
        assert!(w.time < 100.0, "time {}", w.time);
        w.time = 32399.0;
        advance_time(&mut w, 1.0, &mut UnifiedRandom::new(0), false, (6, 1));
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
        advance_time(&mut w, 0.1, &mut UnifiedRandom::new(0), false, (6, 1));
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
        advance_time(&mut w, 0.1, &mut UnifiedRandom::new(0), false, (6, 1));
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
        advance_time(&mut w, 0.1, &mut UnifiedRandom::new(0), false, (6, 1));
        assert!(
            !w.fast_forward_to_dawn,
            "the flag clears on the transition tick"
        );
        assert!(!w.is_fast_forwarding());
        assert_eq!(w.day_rate(), 1);
        // The next add runs at dayRate 1: a 1 s step advances 60 ticks, not 3600.
        advance_time(&mut w, 1.0, &mut UnifiedRandom::new(0), false, (6, 1));
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
        advance_time(&mut w, 0.1, &mut UnifiedRandom::new(0), false, (6, 1));
        assert_eq!(w.sundial_cooldown, 2, "dawn decrements the sundial");
        assert_eq!(w.moondial_cooldown, 5, "StartDay leaves the moondial alone");
        // The following dusk decrements the moondial only.
        w.day_time = true;
        w.time = 53995.0;
        advance_time(&mut w, 0.1, &mut UnifiedRandom::new(0), false, (6, 1));
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
        advance_time(&mut w, 0.1, &mut UnifiedRandom::new(0), false, (6, 1));
        assert_eq!(w.sundial_cooldown, 0);
        w.day_time = true;
        w.time = 53995.0;
        advance_time(&mut w, 0.1, &mut UnifiedRandom::new(0), false, (6, 1));
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
        advance_time(&mut w, 0.1, &mut UnifiedRandom::new(0), false, (6, 1));
        assert!(w.day_time, "exactly one transition fired");
        assert_eq!(w.time, 0.0);
        // From the fresh day start (rate back to 1) the next step stays inside
        // the day half — no dusk this call.
        advance_time(&mut w, 0.1, &mut UnifiedRandom::new(0), false, (6, 1));
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

    /// W4-f41 — `UpdateTime_StartNight`'s `NPC.freeCake = false`
    /// (`Main.cs:65222`): an unclaimed cake gift dies at dusk; a mid-day
    /// tick never touches it.
    #[test]
    fn w4_f41_the_dusk_boundary_clears_the_free_cake_gift() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.world.day_time = true;
        app.world.time = 10000.0;
        app.world_events.free_cake = true;
        advance_time_with_events(&mut app, 5.0, false);
        assert!(app.world_events.free_cake, "mid-day keeps the gift");
        app.world.time = 53999.0;
        advance_time_with_events(&mut app, 5.0, false);
        assert!(!app.world.day_time, "the dusk crossed");
        assert!(!app.world_events.free_cake, "StartNight clears the gift");
    }

    /// W4-f38 — the party broadcasts: the dawn `Game.BirthdayParty_{n}`
    /// start row over the celebrants' `GetGivenOrTypeNetName`
    /// (`BirthdayParty.cs:127-139`), the dusk `Lang.misc[99]` farewell
    /// (`:56`) and the `NPC.freeCake` dusk clear (`Main.cs:65222`).
    #[test]
    fn the_party_broadcasts_name_the_celebrants_and_clear_the_cake_at_dusk() {
        let _guard = crate::lang::test_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        crate::lang::init(tr_core::localization::GameCulture::English);
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        // Five partyable towns: the Party Girl plus four Guides (the
        // `CanNPCParty` gates — townNPC, aiStyle != 0, not 37/453/441).
        let mut kinds = [crate::npc::NpcKind::PartyGirl; 5];
        kinds[1] = crate::npc::NpcKind::Guide;
        kinds[2] = crate::npc::NpcKind::Merchant;
        kinds[3] = crate::npc::NpcKind::Nurse;
        kinds[4] = crate::npc::NpcKind::Dryad;
        for (i, kind) in kinds.into_iter().enumerate() {
            let mut n = crate::npc::Npc::new(kind, 32.0, 32.0, 1, 0.0);
            n.hp = n.max_hp;
            n.home = Some((2 + i as i32, 6));
            n.given_name = format!("Name{i}");
            app.npcs.push(n);
        }
        // The party's own draws: Next(10)==0, Next(5,11)=6, the celebrant
        // count roll (1), then one Next() per candidate for the OrderBy.
        let mut seq = [0i32, 6, 1, 4, 3, 2, 1, 0].into_iter();
        app.world.day_time = false;
        app.world.time = 32399.0;
        // Drive the scheduler directly with the pinned party stream — the
        // boundary's own roll rides the named stream (the eclipse/blood-moon
        // gates are off: no hardmode, no 120+ player).
        let party_girl = app
            .npcs
            .iter()
            .any(|n| n.hp > 0 && n.kind == crate::npc::NpcKind::PartyGirl);
        let partyable: Vec<bool> = app
            .npcs
            .iter()
            .map(|n| {
                n.hp > 0 && n.kind.is_town_npc() && n.kind != crate::npc::NpcKind::TaxCollector
            })
            .collect();
        let party_up = crate::world_events::check_morning_party(
            &mut app.world_events,
            party_girl,
            &partyable,
            false,
            &mut |lo, _hi| seq.next().unwrap_or(lo),
        );
        assert!(party_up, "the pinned sequence lands a genuine party");
        assert!(app.world_events.free_cake);
        // The start broadcast — the clock's dawn arm formats it from the
        // celebrants; mirror the same call here (the boundary walk below
        // exercises the dusk side).
        let names: Vec<String> = app
            .world_events
            .celebrating_npcs
            .iter()
            .filter_map(|&i| app.npcs.get(i))
            .map(crate::npc_names::given_or_type_name)
            .collect();
        let key = format!("Game.BirthdayParty_{}", names.len());
        let text = crate::lang::text_args(&key, &names);
        crate::chat::broadcast_death_text(&mut app.chat, &text, [255, 0, 160]);
        assert!(
            app.chat
                .messages
                .iter()
                .any(|m| m.text.starts_with("Looks like Name")
                    && m.text.ends_with("is throwing a party")
                    && m.color == [255, 0, 160]),
            "the Game.BirthdayParty_1 start broadcast: {text}"
        );
        // The dusk boundary: the farewell + the freeCake clear.
        app.world.day_time = true;
        app.world.time = 53999.0;
        app.chat.messages.clear();
        advance_time_with_events(&mut app, 5.0, false);
        assert!(!app.world.day_time, "the dusk transition ran");
        assert!(
            !app.world_events.free_cake,
            "Main.cs:65222 clears NPC.freeCake at dusk"
        );
        assert!(
            app.chat
                .messages
                .iter()
                .any(|m| m.text == "Party time's over!" && m.color == [255, 0, 160]),
            "the Lang.misc[99] farewell"
        );
    }

    #[test]
    fn day_cycle_lengths_and_the_strict_boundaries() {
        // `Main.cs:65137`/`:65148`: the two boundary tests are STRICT —
        // `Main.time > 32400.0` starts the day, `Main.time > 54000.0` starts
        // the night — so a clock resting exactly ON the boundary is still the
        // old half (the naive `>=` flips one tick early).
        // The lengths: day `0..54000`, night `0..32400`, both at
        // `Main.dayRate` 1 (`:64960-64962`), i.e. 900 s and 540 s at 60 Hz.
        let mut w = World::new("t".into(), 8, 8);
        assert_eq!(w.time, 0.0);
        assert!(w.day_time, "a world starts at dawn");
        assert_eq!(54000.0 / 60.0, 900.0, "the 15-minute day");
        assert_eq!(32400.0 / 60.0, 540.0, "the 9-minute night");
        // Exactly on the dusk boundary: still day.
        w.time = 54000.0;
        advance_time(&mut w, 0.0, &mut UnifiedRandom::new(0), false, (6, 1));
        assert!(w.day_time, "54000.0 exactly is still the day");
        assert_eq!(w.time, 54000.0);
        // One step past it: the night, overshoot discarded (`:65151`).
        advance_time(&mut w, 0.05, &mut UnifiedRandom::new(0), false, (6, 1));
        assert!(!w.day_time, "past 54000.0 the night starts");
        assert_eq!(w.time, 0.0);
        // The night boundary is strict in the same way.
        w.time = 32400.0;
        advance_time(&mut w, 0.0, &mut UnifiedRandom::new(0), false, (6, 1));
        assert!(!w.day_time, "32400.0 exactly is still the night");
        advance_time(&mut w, 0.05, &mut UnifiedRandom::new(0), false, (6, 1));
        assert!(w.day_time, "past 32400.0 the day starts");
        // The mid-half pins: noon 27000 (`:5906-5908`), midnight 16200
        // (`:5910-5913`) — both inside their half, so no flip.
        w.time = 27000.0;
        advance_time(&mut w, 0.0, &mut UnifiedRandom::new(0), false, (6, 1));
        assert!(w.day_time, "noon is the middle of the day");
        w.day_time = false;
        w.time = 16200.0;
        advance_time(&mut w, 0.0, &mut UnifiedRandom::new(0), false, (6, 1));
        assert!(!w.day_time, "midnight is the middle of the night");
    }

    /// W4-d120 — `Main.cs:65222`: every dusk clears `NPC.freeCake`, so the
    /// Party Girl's cake gift only lives for the party day it was earned on
    /// (`BirthdayParty.cs:106` sets it at dawn). The dawn boundary must NOT
    /// clear it — the morning party sets it on that same tick.
    #[test]
    fn dusk_clears_the_free_cake_latch() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.world.day_time = true;
        app.world.time = 53999.0;
        app.world_events.free_cake = true;
        advance_time_with_events(&mut app, 5.0, false);
        assert!(!app.world.day_time, "dusk ran");
        assert!(!app.world_events.free_cake, "`Main.cs:65222` clears the latch");
        // The dawn side leaves it alone (the morning party may set it).
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.world.day_time = false;
        app.world.time = 32399.0;
        app.world_events.free_cake = true;
        advance_time_with_events(&mut app, 5.0, false);
        assert!(app.world.day_time, "dawn ran");
        assert!(app.world_events.free_cake, "StartDay keeps the latch");
    }

    /// W4-f37 doc-sweep — `Player.CollectTaxes`'s anniversary pair
    /// W4-f37 doc-sweep - `Player.CollectTaxes`'s anniversary pair
    /// (`Player.cs:24133-24137`): on `Main.tenthAnniversaryWorld` BOTH the
    /// 50-copper rate and the 25-gold cap double (`num2 *= 2; num *= 2;`).
    /// The `>= cap` early return (`:24138`) and the strict `>` clamp
    /// (`:24151-24153`) read the DOUBLED cap.
    #[test]
    fn the_anniversary_world_doubles_the_tax_rate_and_the_cap() {
        // The plain world: 50 per payer, 250000 cap (the standing witness).
        let mut w = World::new("t".into(), 8, 8);
        w.tax_collector = true;
        w.tax_payers = 3;
        collect_taxes(&mut w);
        assert_eq!(w.tax_money, 150, "3 x 50 on the plain world");
        // The anniversary seed: 100 per payer.
        let mut a = World::new("t".into(), 8, 8);
        a.seeds |= tr_core::seeds::SeedFlags::TENTH_ANNIVERSARY;
        a.tax_collector = true;
        a.tax_payers = 3;
        collect_taxes(&mut a);
        assert_eq!(a.tax_money, 300, "3 x 100 on the anniversary world");
        // The doubled cap: a purse at the PLAIN cap still collects (the
        // `>= cap` gate reads 500000), and the clamp lands on 500000.
        a.tax_money = crate::world_sim::npcs::TAX_CAP; // 250000 < 500000
        a.tax_payers = 100;
        collect_taxes(&mut a);
        assert_eq!(a.tax_money, 260000, "the plain cap is not the gate");
        a.tax_money = 500_000 - 10;
        collect_taxes(&mut a);
        assert_eq!(a.tax_money, 500_000, "the doubled 50-gold cap holds");
        let capped = a.tax_money;
        collect_taxes(&mut a);
        assert_eq!(a.tax_money, capped, "no collection past the doubled cap");
        // The plain world's cap is untouched by the sibling's seed.
        w.tax_money = crate::world_sim::npcs::TAX_CAP - 10;
        w.tax_payers = 100;
        collect_taxes(&mut w);
        assert_eq!(w.tax_money, crate::world_sim::npcs::TAX_CAP);
    }


    // ---- W4-f39: the season/date gates at the dawn boundary (pinned dates) ----

    /// A night world one tick below the dawn boundary, ready to cross.
    fn night_at_dawn_edge() -> World {
        let mut w = World::new("t".into(), 40, 40);
        w.day_time = false;
        w.time = 32399.0;
        w
    }

    #[test]
    fn w4_f39_the_dawn_check_opens_the_halloween_window() {
        // `Main.cs:65391` (`checkHalloween`) at the dawn crossing: Oct 10-31
        // + Nov 1 open `Main.halloween`; the edges are inclusive.
        for (date, want) in [
            ((10u32, 9u32), false),
            ((10, 10), true),
            ((10, 31), true),
            ((11, 1), true),
            ((11, 2), false),
            ((6, 15), false),
        ] {
            let mut w = night_at_dawn_edge();
            advance_time(&mut w, 5.0, &mut UnifiedRandom::new(1), false, date);
            assert!(w.day_time, "dawn ran for {date:?}");
            assert_eq!(w.halloween, want, "halloween at {date:?}");
            assert!(!w.xmas, "the xmas window stays shut at {date:?}");
        }
    }

    #[test]
    fn w4_f39_the_dawn_check_opens_the_xmas_window() {
        // `Main.cs:65390` (`checkXMas`): Dec 15-31 open `Main.xMas`.
        for (date, want) in [
            ((12u32, 14u32), false),
            ((12, 15), true),
            ((12, 31), true),
            ((1, 1), false),
        ] {
            let mut w = night_at_dawn_edge();
            advance_time(&mut w, 5.0, &mut UnifiedRandom::new(1), false, date);
            assert_eq!(w.xmas, want, "xmas at {date:?}");
            assert!(!w.halloween, "the halloween window stays shut at {date:?}");
        }
    }

    #[test]
    fn w4_f39_the_endless_seeds_keep_the_season_on_out_of_window() {
        // `forceHalloweenForever`/`forceXMasForever` (the endless micro seeds,
        // `WorldGen.cs:572-578`) hold the flags up at a mid-summer date.
        let mut w = night_at_dawn_edge();
        w.micro_seeds
            .insert(tr_core::seeds::MicroSeedFlags::ENDLESS_HALLOWEEN);
        w.micro_seeds
            .insert(tr_core::seeds::MicroSeedFlags::ENDLESS_CHRISTMAS);
        advance_time(&mut w, 5.0, &mut UnifiedRandom::new(1), false, (7, 4));
        assert!(w.halloween, "the endless halloween seed");
        assert!(w.xmas, "the endless christmas seed");
        // The forever seed suppresses its own today-flag (`Main.cs:10919-10926`).
        assert!(!w.force_halloween_for_today);
        assert!(!w.force_xmas_for_today);
    }

    #[test]
    fn w4_f39_the_moon_wave_15_raises_the_temporary_season_before_the_stop() {
        // `CheckForMoonEventsStartingTemporarySeasons` (`Main.cs:65389`) reads
        // the RUNNING moon event (wave >= 15) BEFORE `stopMoonEvent` (`:65392`)
        // clears it — a pumpkin moon at wave 15 forces halloween for the day
        // even out of window, and the event itself stops at dawn.
        let mut w = night_at_dawn_edge();
        w.start_pumpkin_moon();
        w.wave_number = 15;
        advance_time(&mut w, 5.0, &mut UnifiedRandom::new(1), false, (6, 1));
        assert!(w.force_halloween_for_today, "the wave-15 temporary season");
        assert!(w.halloween, "checkHalloween reads the today-flag");
        assert!(!w.pumpkin_moon, "stopMoonEvent cleared the event at dawn");
        assert_eq!(w.wave_number, 0, "the stop reset the wave");
        // A snow moon at wave 15 forces xmas instead.
        let mut w = night_at_dawn_edge();
        w.start_snow_moon();
        w.wave_number = 20;
        advance_time(&mut w, 5.0, &mut UnifiedRandom::new(1), false, (6, 1));
        assert!(w.force_xmas_for_today);
        assert!(w.xmas);
        assert!(!w.snow_moon);
        // Below wave 15: no temporary season.
        let mut w = night_at_dawn_edge();
        w.start_pumpkin_moon();
        w.wave_number = 14;
        advance_time(&mut w, 5.0, &mut UnifiedRandom::new(1), false, (6, 1));
        assert!(!w.force_halloween_for_today);
        assert!(!w.halloween);
    }

    #[test]
    fn w4_f39_the_season_checks_are_draw_free() {
        // The whole dawn season trio consumes no `UpdateTime` draw beyond the
        // boundary's own eclipse/invasion roll — a same-seed twin with the
        // season flags forced differs only in state, never in stream position.
        let mut plain = night_at_dawn_edge();
        let mut seeded = night_at_dawn_edge();
        seeded
            .micro_seeds
            .insert(tr_core::seeds::MicroSeedFlags::ENDLESS_HALLOWEEN);
        let mut a = UnifiedRandom::new(42);
        let mut b = UnifiedRandom::new(42);
        advance_time(&mut plain, 5.0, &mut a, false, (6, 1));
        advance_time(&mut seeded, 5.0, &mut b, false, (6, 1));
        assert_eq!(a, b, "the season re-derivation draws nothing");
        assert!(seeded.halloween && !plain.halloween, "only the state differs");
    }

    /// W4-d15 final phase: the dusk boundary runs `Star.NightSetup`
    /// (`Main.cs:65223`) BEFORE the blood-moon roll (`:65336`) — the
    /// twin replays the NightSetup draws then the boundary's
    /// `NextDouble`, and the stream equality pins the order. WRITTEN
    /// NOT RUN (code-only round).
    #[test]
    fn the_dusk_boundary_rolls_the_starfall_boost_before_the_blood_moon() {
        let mut w = World::new("t".into(), 8, 8);
        w.day_time = true;
        w.time = 53999.0;
        let mut rng = UnifiedRandom::new(42);
        advance_time(&mut w, 1.0, &mut rng, false, (6, 1));
        assert!(!w.day_time, "dusk ran");
        // The twin: NightSetup's 1-3 draws, then the blood-moon roll.
        let mut twin = UnifiedRandom::new(42);
        let mut probe = World::new("t".into(), 8, 8);
        probe.roll_starfall_night_setup(false, &mut twin);
        assert_eq!(w.starfall_boost, probe.starfall_boost, "the boost landed");
        let _blood_roll = twin.next_double();
        assert_eq!(rng, twin, "NightSetup precedes the blood-moon draw");
        // A dawn crossing does NOT re-roll the boost (StartDay has no
        // NightSetup — the value persists through the day).
        let boost = w.starfall_boost;
        w.time = 32399.0;
        advance_time(&mut w, 1.0, &mut UnifiedRandom::new(7), false, (6, 1));
        assert!(w.day_time);
        assert_eq!(w.starfall_boost, boost, "StartDay leaves the boost");
    }
}
