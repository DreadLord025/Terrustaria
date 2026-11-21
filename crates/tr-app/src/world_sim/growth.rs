//! The world-growth pass (`WorldGen.UpdateWorld`, see `tr_core::growth` for
//! the source map): the 60 Hz fixed step over the rate loops — the herb
//! growth rides the overground picks. The vines/moss and the biome spread
//! are M7-GROWTH-2.
//!
//! The draws ride the `UpdateWorldGen` named stream (`Main.SwapRandom`'s
//! stage key: under the C# swap `WorldGen.genRand` == `Main.rand`,
//! `Main.cs:17802` + `WorldGen.cs:4445`). The stream is continuous across
//! frames exactly like `Main.rand` — the old per-frame
//! `growth_rng`-copy + `next_seed_in_place` reseed ritual is gone with the
//! bridge field.

use crate::app::{App, RngKey};
use tr_core::UnifiedRandom;
use tr_core::World;

/// `WorldGen.StartHardmode`'s runner half (`WorldGen.cs:31701-31720`): the
/// hardmode switch runs `initializeHardMode` — here on the phase's
/// `UpdateWorldGen` stream (the C#'s `genRand` draws ride whatever
/// `Main.rand` is active at the trigger; the port's forced path runs inside
/// the growth phase). Extracted so tests can pin the stream identity without
/// the `TR_HARDMODE` env pin.
pub(crate) fn run_hardmode_init(world: &mut World, corruption: bool, rng: &mut UnifiedRandom) {
    tr_core::spread::initialize_hard_mode(world, corruption, rng);
}

/// `WorldGen.StartHardmode()` (`WorldGen.cs:31701-31719`) — the whole entry,
/// not just the runner half: the `hardMode` early-return (`:31703-31706`, so a
/// repeat call is a no-op), the switch, `initializeHardMode`'s strip walk
/// (`:31570-31591`), then the follow-up broadcast — `Lang.misc[15]` ("The
/// ancient spirits of light and dark have been released.", `:31711`) and
/// progression event 9 (`:31712`). Returns whether it actually ran.
///
/// Deviation: the C# runs the transform on a background thread
/// (`TransformWorldOnBackgroundThread`, `:31721-31742`) with the chat/achievement
/// follow-up queued to the main thread; the port transforms inline, so the
/// follow-up lands in the same call. `TryProtectingSpawnedItems`/`UndoSpawnedItemProtection`
/// (`:31708`/`:31717`) have no ported consumer (no item-protection state).
pub(crate) fn start_hardmode(app: &mut App) -> bool {
    if app.world.hard_mode {
        return false;
    }
    app.world.hard_mode = true;
    let corruption = (app.world.seed >> 56) & 1 == 1;
    run_hardmode_init(
        &mut app.world,
        corruption,
        app.rngs.stream(RngKey::UpdateWorldGen),
    );
    crate::chat::broadcast_death_text(
        &mut app.chat,
        &crate::lang::text("LegacyMisc.15"),
        crate::chat::COLOR_WORLD,
    );
    let _ = app.notify_progression_net(9);
    true
}

pub(crate) fn update_growth(app: &mut App, dt: f32) {
    // The hardmode switch (`WorldGen.StartHardmode`, `WorldGen.cs:31701-
    // 31719`): the dev trigger `TR_HARDMODE=1` runs the runner half only —
    // `start_hardmode`'s chat/achievement follow-up stays off so a benched
    // run does not paint the chat log.
    let rng = app.rngs.stream(RngKey::UpdateWorldGen);
    if !app.world.hard_mode && crate::env::hardmode_forced() {
        app.world.hard_mode = true;
        let corruption = (app.world.seed >> 56) & 1 == 1;
        run_hardmode_init(&mut app.world, corruption, rng);
    }
    let raining = app.rain.is_raining();
    let day_time = app.world.day_time;
    app.growth_step.tick(dt.min(0.25), |_| {
        let world = &mut app.world;
        // W4-c24c — `GetWorldUpdateRate` (`WorldGen.cs:72020-72028`):
        // `min(desiredWorldTilesUpdateRate, 24)`, 0 while FreezeTime is on.
        // The desired rate is `UpdateTimeRate`'s num (`Main.cs:6210-6220`):
        // the ModifyTimeRate target ×5 while every player sleeps, 0 frozen;
        // the dial fast-forward keeps the tiles at 1 (`:6204`). The port's
        // `World::day_rate` carries the 60/5/1 shape — the 60 arm maps to the
        // desired 1, the 5 arm to target×5, the 1 arm to the target.
        let freeze = app.creative_powers.toggle(crate::creative::PowerId::FreezeTime);
        let target = app.creative_powers.slider(crate::creative::PowerId::TimeRate) as i32;
        let day_rate = match world.day_rate() {
            60 => 1, // the dial arm (`:6204`)
            5 => target * 5, // the all-asleep ×5 (`:6213`)
            _ => target,
        };
        // `WorldGen.UpdateWorld`'s infection flag (`WorldGen.cs:71502-71507`):
        // the journey `StopBiomeSpreadPower` freezes the evil/hallow spread.
        let allowed_to_spread = app.creative_powers.allowed_to_spread_infections();
        // The shell-pile arm's weather inputs: the live rain intensity and
        // the wind speed (`Main.maxRaining`/`Main.windSpeedCurrent`).
        tr_core::growth::update_world(
            world,
            day_rate,
            raining,
            freeze,
            allowed_to_spread,
            day_time,
            app.rain.max_raining(),
            app.wind.speed_current,
            rng,
        );
        // `:71536-71540`: rate 0 returns BEFORE every loop — the port's
        // hardUpdateWorld surrogate picks ride the same gate (the C# calls
        // it from inside those loops).
        if tr_core::growth::world_update_rate(day_rate, freeze) == 0 {
            return;
        }
        // `hardUpdateWorld`'s spread arms ride the same picks (the port
        // runs them unconditionally — the hardmode gate waits for
        // M5-HARDFORCE; `downedPlantBoss` false = the full rate).
        //
        // The bands are the ones the C#'s own loops pick from
        // (`WorldGen.genRand.Next(10, Main.maxTilesX - 10)`,
        // `Next((int)Main.worldSurface - 1, Main.maxTilesY - 20)`,
        // `WorldGen.cs:71574-71575`/`:71621`). A world too small to hold them
        // — the headless fixtures run 8x8 and 220x220 — would ask
        // `UnifiedRandom::next_range` for a reversed band, which is exactly
        // the `Next(min, max)` guard the C# would trip too. The pick is a
        // port-only surrogate (`hardUpdateWorld` is called from *inside* those
        // loops, on cells the loops already chose), so on a degenerate world
        // the faithful reading of "the pick found no cell" is to skip it.
        let (sx_lo, sx_hi) = (10, world.width - 10);
        let (sy_lo, sy_hi) = (world.rock_layer as i32, world.height - 40);
        if sx_lo < sx_hi && sy_lo < sy_hi {
            let sx = rng.next_range(sx_lo, sx_hi);
            let sy = rng.next_range(sy_lo, sy_hi);
            tr_core::spread::hard_update_world(world, sx, sy, false, allowed_to_spread, rng);
        }
    });
    app.growth_step.cap_backlog();
}

#[cfg(test)]
mod tests {
    use super::{run_hardmode_init, update_growth};
    use crate::app::RngKey;
    use crate::app::flow::front_flow_tests::flow_app;
    use tr_core::{UnifiedRandom, World};

    /// A growth-sized world: `WorldGen.UpdateWorld`'s pick counts
    /// (`maxX·maxY·3e-5·rate`) are 0 on `flow_app`'s 8×8 and the
    /// `Next(10, maxTilesX - 10)` bands need the production minimums.
    fn big_world() -> World {
        World::new("growth".into(), 400, 300)
    }

    fn fresh_app() -> crate::app::App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = big_world();
        app
    }

    /// The full tile grid + the metadata the growth pass reads or writes.
    fn assert_worlds_equal(a: &World, b: &World) {
        assert_eq!(a.tiles, b.tiles, "tile grids diverged");
        assert_eq!((a.width, a.height), (b.width, b.height));
        assert_eq!((a.world_surface, a.rock_layer), (b.world_surface, b.rock_layer));
        assert_eq!(a.hard_mode, b.hard_mode);
    }

    /// DW-1.1: the growth phase draws the `UpdateWorldGen` stream — the
    /// stream advanced past its pristine seed state, and the phase touched
    /// no other stage's stream.
    #[test]
    fn test_dw_1_1_growth_phase_draws_the_update_worldgen_stream() {
        let mut app = fresh_app();
        let pristine = UnifiedRandom::new(app.world.seed as i32);
        update_growth(&mut app, 1.0 / 60.0);
        assert_ne!(
            *app.rngs.stream(RngKey::UpdateWorldGen),
            pristine,
            "the growth group drew from the UpdateWorldGen stream"
        );
        for key in [
            RngKey::UpdatePlayers,
            RngKey::UpdateTime,
            RngKey::UpdateNpcs,
            RngKey::UpdateItems,
        ] {
            assert_eq!(
                *app.rngs.stream(key),
                pristine,
                "{:?} must be untouched by the growth phase",
                key
            );
        }
    }

    /// DW-1.1: the hardmode-force init consumes exactly the phase's
    /// `UpdateWorldGen` stream (the same bound `rng` local — the env-free
    /// form of the `TR_HARDMODE=1` arm).
    #[test]
    fn test_dw_1_1_hardmode_init_draws_the_update_worldgen_stream() {
        let mut app = fresh_app();
        let corruption = (app.world.seed >> 56) & 1 == 1;
        // The arm's own order (update_growth): the flag flips first, the
        // runner half draws the phase stream.
        app.world.hard_mode = true;
        {
            let rng = app.rngs.stream(RngKey::UpdateWorldGen);
            run_hardmode_init(&mut app.world, corruption, rng);
        }
        assert!(app.world.hard_mode, "the init ran");
        // Twin: a fresh seed-seeded stream through the same call leaves the
        // identical state — the init consumed exactly this stream and
        // nothing else drew from it.
        let mut twin = UnifiedRandom::new(app.world.seed as i32);
        let mut twin_world = big_world();
        tr_core::spread::initialize_hard_mode(&mut twin_world, corruption, &mut twin);
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateWorldGen),
            twin,
            "hardmode init = exactly the UpdateWorldGen stream's prefix"
        );
        assert_eq!(app.world.tiles, twin_world.tiles, "same ore seeded");
    }

    /// DW-1.4 (determinism pair): two fresh same-seed runs of the growth
    /// phase produce identical worlds.
    #[test]
    fn test_dw_1_4_growth_group_is_deterministic_per_seed() {
        let mut a = fresh_app();
        let mut b = fresh_app();
        for _ in 0..10 {
            update_growth(&mut a, 1.0 / 60.0);
            update_growth(&mut b, 1.0 / 60.0);
        }
        assert_worlds_equal(&a.world, &b.world);
    }

    /// DW-1.4 (draw-count pin): one growth step consumes EXACTLY the group's
    /// draws — `update_world`, the two `hardUpdateWorld` picks and
    /// `hard_update_world` — from the `UpdateWorldGen` stream and nothing
    /// else. The twin drives a fresh seed-seeded stream by hand; equal
    /// `UnifiedRandom` state proves the draw counts match (no more, no
    /// fewer) and equal tile grids prove the same values landed.
    #[test]
    fn test_dw_1_4_growth_consumes_exactly_its_stream_prefix() {
        let mut app = fresh_app();
        update_growth(&mut app, 1.0 / 60.0);

        let mut twin = fresh_app();
        let raining = twin.rain.is_raining();
        let day_time = twin.world.day_time;
        let allowed = twin.creative_powers.allowed_to_spread_infections();
        let mut stream = UnifiedRandom::new(twin.world.seed as i32);
        tr_core::growth::update_world(
            &mut twin.world,
            1,
            raining,
            false,
            allowed,
            day_time,
            twin.rain.max_raining(),
            twin.wind.speed_current,
            &mut stream,
        );
        let sx = stream.next_range(10, twin.world.width - 10);
        let sy = stream.next_range(twin.world.rock_layer as i32, twin.world.height - 40);
        tr_core::spread::hard_update_world(
            &mut twin.world,
            sx,
            sy,
            false,
            allowed,
            &mut stream,
        );

        assert_eq!(
            *app.rngs.stream(RngKey::UpdateWorldGen),
            stream,
            "the growth group consumed exactly this prefix of the stream"
        );
        assert_worlds_equal(&app.world, &twin.world);
    }

    /// W4-c24c — `GetWorldUpdateRate` (`WorldGen.cs:72020-72028`): FreezeTime
    /// zeroes the rate, and `UpdateWorld`'s `:71536-71540` early return skips
    /// EVERY loop — the growth phase draws nothing from the stream (the
    /// hardUpdateWorld surrogate picks ride the same gate). The unfrozen
    /// twin at the same seed draws the normal prefix.
    #[test]
    fn w4_c24c_freeze_time_stops_the_growth_loops() {
        let mut frozen = fresh_app();
        frozen.creative_powers.set(
            crate::creative::PowerId::FreezeTime,
            crate::creative::PowerState::Toggle(true),
        );
        let pristine = UnifiedRandom::new(frozen.world.seed as i32);
        update_growth(&mut frozen, 1.0 / 60.0);
        assert_eq!(
            *frozen.rngs.stream(RngKey::UpdateWorldGen),
            pristine,
            "rate 0 returns before every loop (:71537-71540)"
        );
        // The ModifyTimeRate slider scales the loops (`:6210`'s num): x24
        // draws more than x1 on the same seed (the pick counts scale).
        let mut fast = fresh_app();
        fast.creative_powers.set(
            crate::creative::PowerId::TimeRate,
            crate::creative::PowerState::Slider(24.0),
        );
        let before = *fast.rngs.stream(RngKey::UpdateWorldGen);
        update_growth(&mut fast, 1.0 / 60.0);
        assert_ne!(
            *fast.rngs.stream(RngKey::UpdateWorldGen),
            before,
            "the unfrozen x24 loops draw"
        );
        // The frozen world's tiles never moved.
        let untouched = fresh_app();
        assert_worlds_equal(&frozen.world, &untouched.world);
    }
}
