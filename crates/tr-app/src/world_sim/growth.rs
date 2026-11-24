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
use glam::Vec2;
use tr_core::UnifiedRandom;
use tr_core::World;

/// `WorldGen.StartHardmode`'s runner half (`WorldGen.cs:31570-31699`): the
/// hardmode switch runs `initializeHardMode` — here on the phase's
/// `UpdateWorldGen` stream (the C#'s `genRand` draws ride whatever
/// `Main.rand` is active at the trigger; the port's forced path runs inside
/// the growth phase). Extracted so tests can pin the stream identity without
/// the `TR_HARDMODE` env pin. W4-g69 retire sweep: the evil flavour is
/// `world.crimson` read inside `initialize_hard_mode` (the C# `GERunner`
/// reads the `WorldGen.crimson` static, `WorldGen.cs:76304`) — the old
/// seed-high-bit `corruption` re-derivation (the `roll_crimson` stand-in)
/// ignored the UI evil pick and the drunk dawn flip.
pub(crate) fn run_hardmode_init(world: &mut World, rng: &mut UnifiedRandom) {
    tr_core::spread::initialize_hard_mode(world, rng);
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
    run_hardmode_init(&mut app.world, app.rngs.stream(RngKey::UpdateWorldGen));
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
        run_hardmode_init(&mut app.world, rng);
    }
    let raining = app.rain.is_raining();
    let day_time = app.world.day_time;
    // W4-d119 — the mid-game relocation driver rides `WorldGen.UpdateWorld`'s
    // head (`WorldGen.cs:71550-71551`), AFTER the liquid pass and BEFORE the
    // growth loops, on the same `UpdateWorldGen` stream. Each relocated NPC
    // fires NO_HOBO (8) once the borrow releases (`SpawnTownNPC:5460`).
    let mut relocated = 0u32;
    app.growth_step.tick(dt.min(0.25), |_| {
        // `WorldGen.UpdateWorld`'s head statics (`WorldGen.cs:71535-71549`,
        // W4-d119 phase 2): the rate-0 early return gates the homeless-spawn
        // timeout decay (once per call while positive) and the
        // `npcSpawnPeriod = 20 * GetWorldUpdateRate()` re-arm. The port runs
        // the growth pass at the day rate 1, so the period is 20.
        if app.creative_powers.time_rate() > 0.0 {
            if app.town_spawn.homeless_spawn_timeout > 0 {
                app.town_spawn.homeless_spawn_timeout -= 1;
            }
            app.town_spawn.npc_spawn_period = 20;
        }
        // `WorldGen.UpdatePrioritizedTownNPC` (`:71627-71641`): the first
        // homeless town NPC with an expired timer becomes the prioritized type.
        crate::town_rehome::update_prioritized_town_npc(&app.npcs, &mut app.town_arrival);
        // `WorldGen.CheckForHousesNearAPlayer` (`:71643-71674`): 300 house-wall
        // picks around the player; the first runs the relocation-only
        // `SpawnTownNPC(x, y, canSpawnNewTownNPC: false)`.
        let player_center = app.player.as_ref().map(|p| p.center());
        if crate::town_rehome::check_for_houses_near_a_player(
            &app.world,
            &mut app.npcs,
            &mut app.town_rooms,
            &mut app.town_arrival,
            player_center,
            false,
            rng,
        )
        .is_some()
        {
            relocated += 1;
        }
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
        // W4-d119 escalation 1 — the `TrySpawningTownNPC` walker seam
        // (`WorldGen.cs:72497`/`:73807`): the growth walkers' inactive-tile
        // arms push `(x, y)` spawn-requests onto the shared queue (the gate
        // + its ONE conditional Next(4) ride this `UpdateWorldGen` stream,
        // inline with the picks exactly like the C#); the drain below runs
        // the full `SpawnTownNPC` per request after the mutable pass.
        let mut walker = tr_core::growth::TownSpawnWalker {
            invasion_type: app.world.invasion_type,
            eclipse: app.world.eclipse,
            npc_spawn_delay: &mut app.town_spawn.npc_spawn_delay,
            npc_spawn_period: app.town_spawn.npc_spawn_period,
            requests: &mut app.town_spawn_requests,
        };
        // The shell-pile arm's weather inputs: the live rain intensity and
        // the wind speed (`Main.maxRaining`/`Main.windSpeedCurrent`).
        tr_core::growth::update_world_ts(
            world,
            day_rate,
            raining,
            freeze,
            allowed_to_spread,
            day_time,
            app.rain.max_raining(),
            app.wind.speed_current,
            rng,
            Some(&mut walker),
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
            tr_core::spread::hard_update_world_gated(
                world,
                sx,
                sy,
                tr_core::growth::hard_mode_world_updates(world),
                world.downed_plant_boss,
                allowed_to_spread,
                rng,
            );
        }
        // W4-d15 — `WorldGen.SpawnFallingObjects` (`WorldGen.cs:71624` →
        // `:71676-71857`), the LAST call of `UpdateWorld`: the natural
        // starfall's 720 spawner loop (`:71828-71856`). The boulder-rain /
        // meteor-shower / coin-rain arms ahead of it are cross-lane state:
        // `_isRainingBoulders` stays unported (the gate folds false), the
        // f45 lane (`1f3a47fa`) owns `meteorShowerCount`/`coinRain`'s
        // .wld round-trip + the StartMeteorShower/StartRain setters (the
        // 1078 shower spawner + the coin drains are its documented
        // unported consumers — the cross-note below); the starfall
        // loop is the aiStyle-148 spawner's ONLY spawn site (no item shoots
        // 720 — the census: Starfury 65 shoots 9, `Item.cs:2158`).
        spawn_natural_starfall(
            world,
            &mut app.projectiles,
            app.player.as_ref(),
            app.local_id as u8,
            rng,
        );
    });
    app.growth_step.cap_backlog();
    // The relocation NO_HOBO notifications after the mutable pass (the borrow
    // seam — `notify_progression_net` needs `&mut App`; one per move-in,
    // `WorldGen.cs:5460`).
    for _ in 0..relocated {
        let _ = app.notify_progression_net(8);
    }
    // W4-d119 escalation 1 — drain the walkers' `TrySpawningTownNPC` queue:
    // each request runs the full `SpawnTownNPC` (`WorldGen.cs:74878`'s inline
    // call, deferred to this `UpdateWorldGen` slot). The NO_HOBO events fire
    // after the borrow releases (the same seam).
    let requests = std::mem::take(&mut app.town_spawn_requests);
    if !requests.is_empty() {
        let player = app
            .player
            .as_ref()
            .map(|p| (p.x, p.y, crate::player::WIDTH, crate::player::HEIGHT));
        let nohobo = crate::town_spawn::drain_town_spawn_requests(
            &requests,
            &mut app.world,
            &mut app.npcs,
            &mut app.town_rooms,
            &mut app.town_arrival,
            &mut app.town_spawn,
            player,
            false,
            &mut app.chat,
        );
        for _ in 0..nohobo {
            let _ = app.notify_progression_net(8);
        }
    }
}

/// `WorldGen.SpawnFallingObjects`'s natural-starfall loop
/// (`WorldGen.cs:71828-71856`): at night (or on remix) the dayRate loop
///
/// land-MD merge note: W4-g70 lane 8 also carried a simpler `spawn_starfall`
/// roll (the live `Star.cs` pool's `starfall_boost` snapshot); it is
/// SUPERSEDED here by W4-d15's fuller `spawn_natural_starfall` (the surface
/// + AFK gates, the owned spawn with `local_id`, and its witnesses). One
/// roll only — keeping both call sites would double the `UpdateWorldGen`
/// stream draws.
/// rolls `Next(8000) < 10 * (maxTilesX / 4200 * starfallBoost)`; a hit
/// picks a sky point, rolls the closest player's luck (the remix world
/// rolls BAD luck) for the player-targeted branch, refuses a solid
/// spawn box and spawns the 720 spawner (damage 0, kb 0, owner
/// `Main.myPlayer`, ai1 = the targeted player index or −1). The 148 arm
/// (`AI_148_StarSpawner`, `Projectile.cs:53094-53128`) ages it to the
/// FallingStar 12.
///
/// Folds (recorded, docs/ANALYSIS.md "The W4-d15 DD2 slice"):
/// - `Star.starfallBoost` (`Star.cs:35`): LANDED (final phase) —
///   `World.starfall_boost`, re-rolled every dusk by
///   `roll_starfall_night_setup` (`Star.NightSetup`, `Main.cs:65223`,
///   the clock's StartNight arm ahead of the blood-moon roll). The
///   meteor-shower interplay: the C# shower arm (`:71744-71779`) does
///   NOT touch `starfallBoost` — the shower spawns 1078 MeteorOre and
///   decrements `meteorShowerCount` (the f45 lane's field, `1f3a47fa`);
///   the boost's writers are ONLY `NightSetup` + the static ctor
///   (`Star.cs:322`). The two systems share `SpawnFallingObjects`, not
///   state — the shower-arm spawner stays the f45 lane's documented
///   unported consumer (the 1078 row is unported; wiring it here would
///   need `Main.masterMode`/`expertMode` damage tiers + the
///   `FindClosest` dead-gate + the clamp — its own slice).
/// - `Player.FindClosest` (`Player.cs:4942-4967`) folds to the port's one
///   player: a live player is index 0, no player reads the inactive
///   slot's default box (position (0,0), luck 0, `afkCounter` 0 — the
///   `ai_star_spawner` inactive-slot fold). A DEAD player keeps its
///   position/luck (the first-active fallback) — the `:71842` gate has
///   no dead term.
/// - `afkCounter < AFKTimeNeededForNoLuckyStars` (`:71842`, 10800 —
///   `Player.cs:57103`): LANDED (final phase) — `Player.afk_counter`
///   (the `:25497-25506` standing-still counter + the `Select` reset),
///   the inactive slot folds 0. The other readers (`NPC.cs:411`'s
///   `noGroundWorms >= 300`, `WorldGen.cs:71905`'s storm-lightning
///   gate) stay their lanes' folds; `afkCounterForKiting` + the
///   spectating clamp (`:17086`) are unported (no kite/spectate state).
/// - `Main.isThereAWorldSurface` (`Main.cs:2788`: `worldSurface > 50`)
///   gates the WHOLE `SpawnFallingObjects` (`:71678-71681`) — the
///   skyblock/no-surface worlds skip the starfall too.
fn spawn_natural_starfall(
    world: &World,
    projectiles: &mut crate::projectile::ProjectilePool,
    player: Option<&crate::player::Player>,
    local_id: u8,
    rng: &mut UnifiedRandom,
) {
    // `:71678-71681` — the surface gate (the whole SpawnFallingObjects).
    if world.world_surface <= 50.0 {
        return;
    }
    // `:71828-71831` — daylight kills the loop (the remix world keeps it).
    let remix = world.seeds.contains(tr_core::seeds::SeedFlags::REMIX);
    if world.day_time && !remix {
        return;
    }
    // The closest player's snapshot (`Player.FindClosest` — the one
    // player): position + luck + afkCounter. No player reads the
    // inactive slot (position (0,0), luck 0, afk 0). A DEAD player keeps
    // its position/luck — `FindClosest`'s first-active fallback
    // (`Player.cs:4944-4952`) returns it and the `:71842` gate carries
    // NO dead term (the meteor-shower arm's `!dead` at `:71755` is a
    // different branch).
    let (player_x, player_y, player_luck, player_afk) = player
        .map(|p| (p.x, p.y, p.luck(), p.afk_counter))
        .unwrap_or((0.0, 0.0, 0.0, 0));
    // `:71832` — `for (int i = 0; i < Main.dayRate; i++)`.
    for _ in 0..world.day_rate() {
        // `:71834-71835` — the boost-scaled gate roll. `starfallBoost`
        // is the dusk-rolled `World.starfall_boost` (the NightSetup
        // chain — the fold above, LANDED).
        let num16 = world.width as f64 / 4200.0 * world.starfall_boost as f64;
        if !((rng.next_max(8000) as f64) < 10.0 * num16) {
            continue;
        }
        // `:71838` — the sky point: X in `(Next(maxTilesX - 50) + 100) *
        // 16`, Y in `Next(maxTilesY * 0.05) * 16`.
        let mut pos = Vec2::new(
            (rng.next_max(world.width - 50) + 100) as f32 * 16.0,
            rng.next_max((world.height as f64 * 0.05) as i32) as f32 * 16.0,
        );
        // `:71839-71847` — the luck roll for the player-targeted branch:
        // `RollLuck(15)` (the remix world `RollBadLuck(15)`), the surface
        // gate and the afk gate. `num18` stays −1 unless all three pass.
        let mut num18 = -1i32;
        let luck_roll = if remix {
            tr_core::loot::roll_bad_luck(rng, player_luck, 15)
        } else {
            tr_core::loot::roll_luck(rng, player_luck, 15)
        };
        if luck_roll == 0
            && (player_y as f64) < world.world_surface * 16.0
            && player_afk < crate::player::AFK_TIME_NEEDED_FOR_NO_LUCKY_STARS
        {
            // `:71844-71846` — the lateral offset roll: `Next(1, 640)`
            // then `Next(-num20, num20 + 1)`.
            let num20 = rng.next_range(1, 640);
            pos.x = player_x + rng.next_range(-num20, num20 + 1) as f32;
            num18 = 0; // `Player.FindClosest` — the port's one player
        }
        // `:71848-71854` — the solid-box refusal, then the velocity
        // rolls (`Next(-100, 101)`, `Next(200) + 100`) normalized to the
        // FallingStar's speed 12, and the 720 spawn (damage 0, kb 0,
        // ai0 0, ai1 = num18).
        if crate::projectile::solid_collision(world, pos, 16.0, 16.0) {
            continue;
        }
        let num21 = rng.next_range(-100, 101) as f32;
        let num22 = (rng.next_max(200) + 100) as f32;
        let num23 = 12.0 / (num21 * num21 + num22 * num22).sqrt();
        // `NewProjectile` centres the box (`Projectile.cs:10876-10877`);
        // the pool seam takes the top-left — the 720 row is 16×16.
        let _ = projectiles.spawn_with_data_ai_owned(
            crate::projectile::FALLING_STAR_SPAWNER,
            local_id,
            pos.x - 8.0,
            pos.y - 8.0,
            num21 * num23,
            num22 * num23,
            0,
            0.0,
            0.0,
            num18 as f32,
        );
    }
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
        // The arm's own order (update_growth): the flag flips first, the
        // runner half draws the phase stream. The evil flavour rides
        // `world.crimson` (the ctor default false = the corruption world).
        app.world.hard_mode = true;
        {
            let rng = app.rngs.stream(RngKey::UpdateWorldGen);
            run_hardmode_init(&mut app.world, rng);
        }
        assert!(app.world.hard_mode, "the init ran");
        // Twin: a fresh seed-seeded stream through the same call leaves the
        // identical state — the init consumed exactly this stream and
        // nothing else drew from it.
        let mut twin = UnifiedRandom::new(app.world.seed as i32);
        let mut twin_world = big_world();
        tr_core::spread::initialize_hard_mode(&mut twin_world, &mut twin);
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
        tr_core::spread::hard_update_world_gated(
            &mut twin.world,
            sx,
            sy,
            tr_core::growth::hard_mode_world_updates(&twin.world),
            twin.world.downed_plant_boss,
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

    /// W4-d119 escalation 1 — the walker seam rides the phase's
    /// `UpdateWorldGen` stream: with the period armed to 1 (and FreezeTime on
    /// so the head's `npcSpawnPeriod = 20` re-arm is skipped, the rate-0 gate),
    /// every inactive-tile pick runs the `TrySpawningTownNPC` gate (the wall-0
    /// cells fire with no draw), the requests queue fills, and the drain runs
    /// the full `SpawnTownNPC` per request. The twin drives the same prefix by
    /// hand through `update_world_ts` + the gate, proving the draw counts match.
    #[test]
    fn the_walker_seam_queues_and_drains_on_the_phase_stream() {
        let freeze = |app: &mut crate::app::App| {
            app.creative_powers.set(
                crate::creative::PowerId::FreezeTime,
                crate::creative::PowerState::Toggle(true),
            );
            app.town_spawn.npc_spawn_period = 1;
        };
        let mut app = fresh_app();
        freeze(&mut app);
        update_growth(&mut app, 1.0 / 60.0);
        // The gate fired on the inactive picks: the queue drained (empty now)
        // and the counter reset per fire.
        assert!(app.town_spawn_requests.is_empty(), "the drain emptied the queue");
        assert_eq!(app.town_spawn.npc_spawn_delay, 0);

        // The twin: the same picks through the seam, counting the pushes.
        let mut twin = fresh_app();
        freeze(&mut twin);
        let raining = twin.rain.is_raining();
        let day_time = twin.world.day_time;
        let allowed = twin.creative_powers.allowed_to_spread_infections();
        let mut stream = UnifiedRandom::new(twin.world.seed as i32);
        let mut delay = 0;
        let mut requests: Vec<(i32, i32)> = Vec::new();
        {
            let mut walker = tr_core::growth::TownSpawnWalker {
                invasion_type: twin.world.invasion_type,
                eclipse: twin.world.eclipse,
                npc_spawn_delay: &mut delay,
                npc_spawn_period: 1,
                requests: &mut requests,
            };
            tr_core::growth::update_world_ts(
                &mut twin.world,
                1,
                raining,
                false,
                allowed,
                day_time,
                twin.rain.max_raining(),
                twin.wind.speed_current,
                &mut stream,
                Some(&mut walker),
            );
        }
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
            "the walker seam consumed exactly this prefix of the stream"
        );
        assert_worlds_equal(&app.world, &twin.world);
    /// W4-g69 retire sweep — the hardmode V's evil flavour reads the LIVE
    /// `World.crimson` (the C# `GERunner` reads the `WorldGen.crimson` static
    /// at `WorldGen.cs:76304`), NOT the seed's high bit: a UI-picked crimson
    /// world on a corruption-rolled seed carves the crimson V, and the drunk
    /// dawn flip (`Main.cs:65403-65406`) flips the NEXT hardmode init's
    /// flavour with it. The old `(seed >> 56) & 1` stand-in ignored both.
    #[test]
    fn w4_g69_the_hardmode_v_reads_the_live_world_evil() {
        use tr_core::tile::{FLAG_ACTIVE, Tile};
        // A seed whose high bit rolls CORRUPTION (`roll_crimson` false).
        let seed: u64 = 0x00AB_CDEF_0123_4567;
        assert!(!tr_core::worldgen::roll_crimson(seed));
        let grass_world = |crimson: bool| {
            let mut w = World::new("evil".into(), 400, 300);
            w.seed = seed;
            w.crimson = crimson;
            for x in 0..400 {
                for y in 100..300 {
                    let i = w.index(x, y);
                    w.tiles[i] = Tile {
                        kind: if y == 100 { 2 } else { 1 },
                        ..Default::default()
                    };
                    w.tiles[i].flags |= FLAG_ACTIVE;
                }
            }
            w
        };
        let count = |w: &World, kinds: &[u16]| {
            w.tiles
                .iter()
                .filter(|t| t.active() && kinds.contains(&t.kind))
                .count()
        };
        // The UI-picked crimson world: the evil V is crimson (199/203),
        // even though the seed's high bit says corruption.
        let mut w = grass_world(true);
        let mut rng = UnifiedRandom::new(seed as i32);
        run_hardmode_init(&mut w, &mut rng);
        assert!(count(&w, &[199, 203]) > 200, "the crimson V");
        assert_eq!(count(&w, &[23, 25]), 0, "no corrupt tiles");
        // The corruption default: the mirror image on the SAME seed.
        let mut w2 = grass_world(false);
        let mut rng2 = UnifiedRandom::new(seed as i32);
        run_hardmode_init(&mut w2, &mut rng2);
        assert!(count(&w2, &[23, 25]) > 200, "the corrupt V");
        assert_eq!(count(&w2, &[199, 203]), 0, "no crimson tiles");
        // The RNG draw order is flavour-independent (the C# reads the static
        // INSIDE the convert arm, after the wobble roll) — both runs consume
        // the identical stream prefix.
        assert_eq!(rng, rng2, "the flavour changes no draw");
    }
}

/// W4-d15 seam witnesses — the natural starfall's 720 spawn loop
/// (`WorldGen.SpawnFallingObjects`, `WorldGen.cs:71828-71856`), the
/// aiStyle-148 spawner's ONLY spawn site (no item shoots 720). WRITTEN
/// NOT RUN (code-only round).
#[cfg(test)]
mod w4_d15_starfall_tests {
    use super::{spawn_natural_starfall, update_growth};
    use crate::app::RngKey;
    use crate::app::flow::front_flow_tests::flow_app;
    use tr_core::{UnifiedRandom, World};

    /// A starfall-sized world: the sky band needs `maxTilesX - 50 > 0`
    /// and the surface gate needs `worldSurface > 50`.
    fn starfall_app() -> crate::app::App {
        let mut app = flow_app(None, vec![], vec![]);
        let mut w = World::new("starfall".into(), 400, 300);
        w.world_surface = 75.0;
        w.day_time = false;
        app.world = w;
        app.player = Some(crate::player::Player::spawn(&app.world));
        app
    }

    /// The daylight gate (`:71828-71831`): a classic day draws NOTHING
    /// and spawns nothing — the loop returns before the first roll.
    #[test]
    fn the_daylight_gate_draws_nothing() {
        let mut app = starfall_app();
        app.world.day_time = true;
        let pristine = UnifiedRandom::new(app.world.seed as i32);
        let player = app.player.clone();
        let rng = app.rngs.stream(RngKey::UpdateWorldGen);
        spawn_natural_starfall(&app.world, &mut app.projectiles, player.as_ref(), 0, rng);
        assert_eq!(*app.rngs.stream(RngKey::UpdateWorldGen), pristine, "no draws");
        assert!(app.projectiles.slots().iter().all(|p| !p.active), "no spawn");
    }

    /// The surface gate (`:71678-71681`, `Main.cs:2788`): a no-surface
    /// world (`worldSurface <= 50`) skips the whole SpawnFallingObjects.
    #[test]
    fn the_no_surface_world_skips_the_starfall() {
        let mut app = starfall_app();
        app.world.world_surface = 50.0;
        let pristine = UnifiedRandom::new(app.world.seed as i32);
        let player = app.player.clone();
        let rng = app.rngs.stream(RngKey::UpdateWorldGen);
        spawn_natural_starfall(&app.world, &mut app.projectiles, player.as_ref(), 0, rng);
        assert_eq!(*app.rngs.stream(RngKey::UpdateWorldGen), pristine, "no draws");
    }

    /// The night loop's draw census: the gate roll `Next(8000)` runs
    /// once per dayRate iteration; a miss spends exactly that one draw.
    /// The twin replays the same stream by hand — equal state proves the
    /// loop's draw order (the gate first, then the point/luck/velocity
    /// chain only on a hit).
    #[test]
    fn the_night_loop_consumes_the_gate_rolls_in_order() {
        let mut app = starfall_app();
        let player = app.player.clone();
        let rng = app.rngs.stream(RngKey::UpdateWorldGen);
        spawn_natural_starfall(&app.world, &mut app.projectiles, player.as_ref(), 0, rng);
        // The twin: replay the C# loop's draws on a fresh stream. The
        // 400-wide world's threshold is 10 * 400/4200 ≈ 0.95, so a gate
        // roll < 1 hits — the replay mirrors every branch the arm took.
        let mut twin = UnifiedRandom::new(app.world.seed as i32);
        let mut twin_app = starfall_app();
        let world = &twin_app.world;
        let (px, py, luck, afk) = twin_app
            .player
            .as_ref()
            .map(|p| (p.x, p.y, p.luck(), p.afk_counter))
            .unwrap_or((0.0, 0.0, 0.0, 0));
        for _ in 0..world.day_rate() {
            let num16 = world.width as f64 / 4200.0 * world.starfall_boost as f64;
            if !((twin.next_max(8000) as f64) < 10.0 * num16) {
                continue;
            }
            let mut pos = glam::Vec2::new(
                (twin.next_max(world.width - 50) + 100) as f32 * 16.0,
                twin.next_max((world.height as f64 * 0.05) as i32) as f32 * 16.0,
            );
            let roll = tr_core::loot::roll_luck(&mut twin, luck, 15);
            if roll == 0
                && (py as f64) < world.world_surface * 16.0
                && afk < crate::player::AFK_TIME_NEEDED_FOR_NO_LUCKY_STARS
            {
                let num20 = twin.next_range(1, 640);
                pos.x = px + twin.next_range(-num20, num20 + 1) as f32;
            }
            if crate::projectile::solid_collision(world, pos, 16.0, 16.0) {
                continue;
            }
            let _ = twin.next_range(-100, 101);
            let _ = twin.next_max(200);
        }
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateWorldGen),
            twin,
            "the starfall loop consumed exactly this draw prefix"
        );
    }

    /// A forced gate hit spawns the 720 spawner with the C#'s payload:
    /// damage 0, kb 0, ai0 0, ai1 the targeted-player index (−1 when the
    /// luck roll misses), velocity normalized to 12. The seed search
    /// finds a stream whose first gate roll hits (the 400-wide world's
    /// threshold ≈ 0.95 → a roll of 0).
    #[test]
    fn a_gate_hit_spawns_the_720_with_the_census_payload() {
        // Find a seed whose first Next(8000) is 0 (a guaranteed hit).
        let seed = (0..100_000)
            .find(|&s| UnifiedRandom::new(s).next_max(8000) == 0)
            .expect("a gate-hit seed exists");
        let mut app = starfall_app();
        app.world.seed = seed as u64;
        app.rngs.reset(seed);
        let player = app.player.clone();
        let rng = app.rngs.stream(RngKey::UpdateWorldGen);
        spawn_natural_starfall(&app.world, &mut app.projectiles, player.as_ref(), 0, rng);
        let fired: Vec<_> = app.projectiles.slots().iter().filter(|p| p.active).collect();
        assert!(!fired.is_empty(), "the gate hit spawns");
        let p = fired[0];
        assert_eq!(p.kind, 720, ":71853 — the FallingStarSpawner");
        assert_eq!(p.damage, 0, ":71853 — damage 0");
        assert_eq!(p.knock_back, 0.0, ":71853 — kb 0");
        assert_eq!(p.ai[0], 0.0, ":71853 — ai0 0");
        assert!(p.ai[1] == -1.0 || p.ai[1] == 0.0, ":71839/:71846 — the target index");
        assert!(
            (p.vel.length() - 12.0).abs() < 0.01,
            ":71852 — the speed-12 normalization"
        );
    }

    /// The growth phase wires the loop AFTER the growth picks: a night
    /// world's phase draws more than the day twin's prefix (the starfall
    /// gate rolls join the stream), and the day twin's pin still holds.
    #[test]
    fn the_growth_phase_runs_the_starfall_after_the_picks() {
        let mut night = starfall_app();
        let mut day = starfall_app();
        day.world.day_time = true;
        update_growth(&mut night, 1.0 / 60.0);
        update_growth(&mut day, 1.0 / 60.0);
        // Both phases ran the same growth picks; the night loop adds the
        // starfall gate draws on top (the day loop returns draw-free).
        let night_state = *night.rngs.stream(RngKey::UpdateWorldGen);
        let day_state = *day.rngs.stream(RngKey::UpdateWorldGen);
        assert_ne!(night_state, day_state, "the night starfall drew past the day prefix");
    }

    /// W4-d15 final phase: the gate threshold reads `starfall_boost`
    /// (`WorldGen.cs:71834` — the NightSetup-rolled field, no longer
    /// pinned 1). A boosted world's threshold widens: a seed whose
    /// gate roll misses at boost 1 but hits at boost 5 spawns ONLY on
    /// the boosted twin. WRITTEN NOT RUN (code-only round).
    #[test]
    fn the_gate_threshold_scales_with_the_night_setup_boost() {
        // The 400-wide world: threshold = 10 * 400/4200 * boost ≈
        // 0.952 * boost. A first Next(8000) of 1..4 misses at boost 1
        // (needs 0) and hits at boost 5 (threshold ≈ 4.76).
        let seed = (0..100_000i32)
            .find(|&s| (1..5).contains(&UnifiedRandom::new(s).next_max(8000)))
            .expect("a band seed");
        let mut plain = starfall_app();
        plain.world.seed = seed as u64;
        plain.rngs.reset(seed);
        let mut boosted = starfall_app();
        boosted.world.seed = seed as u64;
        boosted.world.starfall_boost = 5.0;
        boosted.rngs.reset(seed);
        let p1 = plain.player.clone();
        let rng = plain.rngs.stream(RngKey::UpdateWorldGen);
        spawn_natural_starfall(&plain.world, &mut plain.projectiles, p1.as_ref(), 0, rng);
        let p2 = boosted.player.clone();
        let rng = boosted.rngs.stream(RngKey::UpdateWorldGen);
        spawn_natural_starfall(&boosted.world, &mut boosted.projectiles, p2.as_ref(), 0, rng);
        assert!(
            plain.projectiles.slots().iter().all(|p| !p.active),
            "boost 1: the roll misses the ≈0.95 threshold"
        );
        assert!(
            boosted.projectiles.slots().iter().any(|p| p.active),
            "boost 5: the roll hits the ≈4.76 threshold"
        );
    }

    /// W4-d15 final phase: the afk gate (`WorldGen.cs:71842`,
    /// `afkCounter < 10800`) — a player at the ceiling keeps the luck
    /// roll spent but the targeted branch REFUSED (ai1 stays −1, no
    /// offset draws); the fresh twin targets (ai1 = 0). The seed forces
    /// the gate hit AND the luck-roll 0 (the zero-luck `Next(15)`).
    /// WRITTEN NOT RUN (code-only round).
    #[test]
    fn the_afk_ceiling_refuses_the_targeted_branch() {
        // Gate hit (Next(8000) == 0), sky point (2 draws), luck roll
        // Next(15) == 0 — scan the 4-draw prefix. The joint probability
        // is ~1/120000 per seed, so the scan runs wide (1e6 → <0.03%
        // miss chance).
        let seed = (0..1_000_000i32)
            .find(|&s| {
                let mut r = UnifiedRandom::new(s);
                if r.next_max(8000) != 0 {
                    return false;
                }
                let _x = r.next_max(350);
                let _y = r.next_max(15);
                r.next_max(15) == 0
            })
            .expect("a gate+luck hit seed");
        let mut afk = starfall_app();
        afk.world.seed = seed as u64;
        afk.rngs.reset(seed);
        // Lift both players above the surface gate (`player_y <
        // world_surface*16` = 1200; the stock spawn sits exactly ON it)
        // so the afk term is the ONLY difference between the twins.
        afk.player.as_mut().unwrap().y = 100.0;
        afk.player.as_mut().unwrap().afk_counter =
            crate::player::AFK_TIME_NEEDED_FOR_NO_LUCKY_STARS;
        let mut fresh = starfall_app();
        fresh.world.seed = seed as u64;
        fresh.rngs.reset(seed);
        fresh.player.as_mut().unwrap().y = 100.0;
        let p1 = afk.player.clone();
        let rng = afk.rngs.stream(RngKey::UpdateWorldGen);
        spawn_natural_starfall(&afk.world, &mut afk.projectiles, p1.as_ref(), 0, rng);
        let p2 = fresh.player.clone();
        let rng = fresh.rngs.stream(RngKey::UpdateWorldGen);
        spawn_natural_starfall(&fresh.world, &mut fresh.projectiles, p2.as_ref(), 0, rng);
        let a = afk.projectiles.slots().iter().find(|p| p.active).expect("the afk spawn");
        let f = fresh.projectiles.slots().iter().find(|p| p.active).expect("the fresh spawn");
        assert_eq!(a.ai[1], -1.0, ":71842 — the afk gate refuses the target");
        assert_eq!(f.ai[1], 0.0, "the fresh player targets (FindClosest = 0)");
        // The afk arm skipped the two offset draws: its first-spawn X
        // stays the sky roll, the fresh twin's rides the player offset.
        let mut twin = UnifiedRandom::new(seed);
        let _gate = twin.next_max(8000);
        let sky_x = (twin.next_max(350) + 100) as f32 * 16.0;
        let _y = twin.next_max(15);
        let _luck = twin.next_max(15);
        assert_eq!(a.pos.x + 8.0, sky_x, "the afk spawn keeps the sky X");
        let n = twin.next_range(1, 640);
        let off = twin.next_range(-n, n + 1);
        let px = fresh.player.as_ref().unwrap().x;
        assert_eq!(f.pos.x + 8.0, px + off as f32, "the fresh spawn rides the offset");
    }
}
