//! Rain: `Rain.MakeRain` / `Rain.Update` / `Main.DrawRain`.
//!
//! Weather follows the original's shape: a target intensity (`Main.maxRaining`) that
//! `cloudAlpha` ramps toward at `0.0005 * max(1, dayRate)` per 60 Hz tick, a countdown
//! that ends the event, and one drop per `MakeRain` spawn slot per tick with the
//! original fall velocity (`wind * 18, 14` px/tick). Drops are drawn rotated around
//! their top-left origin with the local light colour, and die on solid or wet tiles
//! (`Collision.SolidCollision`/`WetCollision`, `Rain.cs:117`) with a splash — the
//! gfx-quality-gated `Dust.NewDust(position - velocity, 2, 2, Dust.dustWater())`
//! recipe (`Rain.cs:120-131`, W4-g67) drained into the App's shared `DustPool`
//! (the d50 contract), the water-style splash family (33/98..105/123/288) whose
//! per-tick physics is the `Dust.cs:1834-1874` arm (`water_splash_arm` below).
//! In a snow biome the event renders as snowfall instead: no rain
//! drops, and `Dust` type 76 flakes (vanilla's ambient snow dust) drift down all the
//! time, with the storm intensity increasing the count and the wind.
//!
//! Since M1.4-BG-22 the event lifecycle is the vanilla `UpdateTime` rain section
//! (`Main.cs:64833-64949`) — the `ladyBugRainBoost` drain, the `rainTime` drain with
//! the strength-change roll while raining, and the ×5.75/×4.25/ladybug start rolls
//! with the `86400/dayRate` denominators plus `StartRain`/`ChangeRain`/`StopRain` —
//! driven once per 60 Hz world tick (`world_sim::weather::update_world_tick`); the
//! old session-local schedule is retired. Every roll consumes Rain's own splitmix
//! stream (the WEATHER-2 per-consumer-stream convention).
//!
//! Storms also thicken the cloud deck (`Background_112/113` layered behind the
//! mountains) and, above 0.65 intensity, throw random lightning flashes.
//! `TR_RAIN=<intensity>` pins the weather for screenshots; `TR_NO_WEATHER` disables
//! the scheduler.
//!
//! M5-SEED-2: the two lightning micro seeds (`moreLightning`/`noLightning`,
//! `WorldGen.cs:580-586`) gate the storm-lightning roll that
//! [`Rain::thunder_step`] performs — `noLightning` alone suppresses it,
//! `moreLightning` alone doubles its rate and lets rain (not only a storm)
//! arm it. The gates mirror `WorldGen.SpawnStormLightning`
//! (`WorldGen.cs:71864-71884`), the port's stand-in for the C#'s damaging
//! strike roll. Deferred with it: the thunder-bolt projectile arms
//! (`Projectile.cs:12776`'s `1090` heal/colour pair, `:14823`'s `1091`
//! colour and `:39871`'s `!Main.moreLightningSeed && Main.noLightningSeed`
//! skip) — the port spawns no thunder-bolt projectile, so those three arms
//! have no seam yet.

use glam::Vec2;
use tr_core::World;
use tr_core::lighting::LightMap;
use tr_core::tiledata as td;
use tr_render::sprite::SpriteRenderer;

use crate::assets;

/// `Main.maxRain`.
const MAX_DROPS: usize = 750;
/// Active snowflakes; vanilla caps the snow dust counter near 500 per screen.
const MAX_SNOW: usize = 600;
const TICKS_PER_SEC: f32 = 60.0;
/// `cloudAlpha` change per tick while raining (vanilla `0.0005 * dayRate`).
const RAMP_PER_TICK: f32 = 0.0005;
/// The `cloudBGAlpha` clamp-step per tick — 10× the `cloudAlpha` ramp so the storm
/// deck appears within it (kept from the retired session schedule), scaled by
/// `max(1, dayRate)` like the `cloudBGActive` ramp it stands in for
/// (`Main.cs:13187/13205`; the full `updateCloudLayer` state machine is unported).
const CLOUD_BG_RAMP_PER_TICK: f32 = 0.005;
/// The `numClouds` stand-in for `ChangeRain`'s strength bands (`Main.cs:64730`): the
/// port's static cloud field is pinned at 100 (`render/sky.rs` `make_clouds`'s
/// `num_clouds = 100.0`) — the live cloud-count system (`Main.cs:58952-59036`) is
/// unported. With exactly 100 the `numClouds > 150` and `numClouds > 100` arms are
/// both false, so only the storm gate (`cloudBGActive >= 1f` → `cloud_bg >= 1.0`)
/// reaches the heavy bands; revisit with the weather-completion row.
const NUM_CLOUDS_STAND_IN: i32 = 100;
const DROP_W: f32 = 2.0;
const DROP_H: f32 = 40.0;

#[derive(Clone, Copy, Debug)]
struct Drop {
    pos: Vec2,
    vel: Vec2,
    scale: f32,
    rot: f32,
    /// Rain frame type (0..2 for water style 0).
    kind: u8,
}

/// A `WaterfallManager.TrySparkling` particle: `Dust.NewDust(..., 43, ...)` with
/// the velocity zeroed (`WaterfallManager.cs:1061-1062`), drawn from the `Dust`
/// sheet's frame 43 at scale 0.5. The generic dust fade (`alpha -= 25` per
/// tick) stands in for the type's own AI.
#[derive(Clone, Copy, Debug)]
struct Sparkle {
    pos: Vec2,
    alpha: f32,
}

/// The cross-system inputs of the drop sim + `MakeRain` (W4-g67 residuals):
/// the C# reads them from `Main` statics / `Main.player[Main.myPlayer]`.
/// `Default` is the calm in-world shape the drop-sim tests run under.
#[derive(Clone, Copy, Debug, Default)]
pub struct DropSimInputs {
    /// `Main.gameMenu` — the `Rain.Update` kill-only arm (`Rain.cs:103-109`)
    /// and the `MakeRain` spawn-through-solids arm (`:84`). The port's frame
    /// gates the weather cluster out of menus, so production passes the
    /// front-state flag for shape; residual drops never tick there.
    pub game_menu: bool,
    /// `Main.remixWorld` — the kill-only arm (`Rain.cs:110-116`) and the
    /// spawn-through-solids arm (`:84`). Reads Wind's stand-in field.
    pub remix_world: bool,
    /// `NPC.AnyDanger(quickBossNPCCheck: true)` (`Rain.cs:37`,
    /// `NPC.cs:80243-80288`): the 0.05 spawn throttle. The weather pass
    /// derives it (invasion / moon events / DD2 / Moon Lord countdown /
    /// the boss-flagged NPC scan — the `CurrentFrameFlags.AnyActiveBossNPC`
    /// stand-in, the `weather.rs` cultist-gate convention).
    pub any_danger: bool,
    /// `Main.windSpeedCurrent` — `GetRainFallVelocity` (`Rain.cs:91-94`)
    /// and the `windSpeedCurrent * 15f * 40f` spawn drift (`:56`).
    pub wind_current: f32,
    /// `Main.player[Main.myPlayer].velocity` — the rising-player spawn
    /// window (`Rain.cs:50-53`) and the `velocity.X * 40f` drift (`:57`).
    pub player_vel: Vec2,
}

/// One `DustID.Snowflake` (type 76) particle.
#[derive(Clone, Copy, Debug)]
struct Snowflake {
    pos: Vec2,
    vel: Vec2,
    scale: f32,
    /// Sway phase; each flake drifts horizontally on its own sine.
    phase: f32,
    frame: u8,
}

pub struct Rain {
    /// `Main.cloudAlpha`: current visible intensity, 0..~1.5.
    intensity: f32,
    /// `Main.maxRaining`: intensity the event ramps toward.
    target: f32,
    raining: bool,
    /// `Main.rainTime`: ticks left in the current event (`Main.cs:64874` drains it
    /// by `dayRate` per world tick); 0 outside an event.
    rain_time: i32,
    /// `Main.ladyBugRainBoost` (`Main.cs:64833-64836`): the ladybug-death rain
    /// window, drained by `dayRate` per tick and gating the third start arm
    /// (`Main.cs:64920`). The setter side (M4 ladybug deaths) is not ported — the
    /// field is inert until then.
    lady_bug_rain_boost: i32,
    /// TR_NO_WEATHER: the port's scheduler switch — with it the C#-shaped rolls
    /// never run (the retired session schedule honoured the same switch).
    calm: bool,
    /// Drop/snow drift velocity, set once at seed time. Vanilla reads the live
    /// `Main.windSpeedCurrent` per tick (`Cloud.cs`-style drift is unported);
    /// the retired session schedule was its last writer (`Main.cs:58830-58950`
    /// feeds `Wind` directly now).
    wind: f32,
    /// `Main.cloudBGAlpha`: storm cloud-bank opacity, 0..1.
    cloud_bg: f32,
    /// `Main.lightning`: the flash brightness ramping up via `lightningSpeed`
    /// and decaying via `lightningDecay`, 0..1.
    flash: f32,
    /// `Main.thunderDelay`: ticks until the thunder sound.
    thunder_delay: i32,
    /// `Main.thunderDistance`: the thunder sound's world offset.
    thunder_distance: i32,
    /// `Main.lightningSpeed`: the flash rise rate.
    lightning_speed: f32,
    /// `Main.lightningDecay`: the flash fall rate.
    lightning_decay: f32,
    /// `Main.IsItStorming` (`Player/Wind` sticky flag) for this tick.
    storming: bool,
    drops: Vec<Drop>,
    sparkles: Vec<Sparkle>,
    snow: Vec<Snowflake>,
    /// The camera is in a snow biome: rain is replaced by snowfall.
    snow_mode: bool,
    /// The gfx-quality value the snow cap scales with (S1-B-6) — the
    /// weather pass copies `App.gfx_quality.quality` in.
    pub gfx_quality: f32,
    rng: u64,
    spawn_acc: f32,
    forced: Option<f32>,
    accum: f32,
}

/// The cross-system inputs of the vanilla lightning chain
/// (`Main.UpdateWeather`, `Main.cs:58784-58829`): the vanilla reads them
/// from Main statics.
#[derive(Clone, Copy, Default)]
pub struct ThunderInputs {
    /// `Main.IsItStorming` (the Wind sticky flag).
    pub storming: bool,
    /// `Main.windSpeedTarget` (the tier thresholds 0.5/0.6/0.7).
    pub wind_target: f32,
    /// `Main.maxRaining` (the tier thresholds 0.6/0.7/0.8).
    pub max_raining: f32,
    /// `Main.atmo == 1f` — the atmosphere factor reached the surface band.
    pub atmo: f32,
    /// `SceneMetrics.TileCenter.Y < Main.rockLayer`.
    pub above_rock_layer: bool,
    /// `Main.GraveyardVisualIntensity` (the >= 0.9 graveyard strikes).
    pub graveyard_intensity: f32,
    /// `Main.shimmerAlpha` (mutes `NewLightning`).
    pub shimmer_alpha: f32,
    /// `Main.thunderSkipSound` (`Main.cs:58759`): the strike's cue is
    /// skipped. Only `NewLightning(instant: true, skipSound: true)`
    /// (`ParticleOrchestrator.cs:1043`, unported) sets it, so it stays false
    /// in the port — carried here because the strike gate reads it.
    pub thunder_skip_sound: bool,
    /// `Main.SceneMetrics.Center` (`Main.cs:58752`): the strike's origin.
    pub scene_center: [f32; 2],
    /// `Main.worldSurface * 16` (`Main.cs:58764`'s `num2`): the strike's y
    /// clamp.
    pub world_surface_px: f32,
    /// M5-SEED-2: `Main.IsItRaining` — the `SpawnStormLightning` rain gate
    /// `moreLightning` relaxes (`WorldGen.cs:71872`: `!IsItStorming &&
    /// (!flag || !IsItRaining)` → return). A classic world never reads it.
    pub raining: bool,
    /// M5-SEED-2: `Main.moreLightningSeed` (`WorldGen.cs:580-582`'s
    /// `SecretSeed.moreLightning.Enabled` write) — the storm-lightning gates
    /// `:71866-71881`.
    pub more_lightning: bool,
    /// M5-SEED-2: `Main.noLightningSeed` (`WorldGen.cs:584-586`) — the
    /// suppression arm `:71866-71868`.
    pub no_lightning: bool,
}

/// One thunder strike cue (`Main.cs:58747-58766`), produced by
/// [`Rain::thunder_step`] and played by the weather tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThunderStrike {
    /// The `Sounds/Thunder_<0..5>` cue family (`LegacySoundPlayer.cs:454`).
    pub cue: &'static str,
    /// The picked `SoundInstanceThunder` slot (`LegacySoundPlayer.cs:455`'s
    /// `num12`, re-rolled while playing) — the engine's sink to replace.
    pub slot: u32,
    /// The strike position in world pixels (already the C#'s `(int)` cast).
    pub x: f32,
    pub y: f32,
    /// `Main.rand.Next(-10, 11) * 0.01` (`LegacySoundPlayer.cs:471`): the
    /// instance pitch jitter, in XNA octaves — the engine applies it as a
    /// playback-rate resample (`2^pitch`).
    pub pitch: f32,
}

/// Bridges the Rain stream into [`tr_core::loot::DropRng`] for `Luck.RollLuck`
/// (`Main.cs:64668`): `Random.Next(min,max)` / `NextDouble()` semantics over
/// Rain's `range`/`rand01`.
struct RainLootRng<'a>(&'a mut Rain);

impl tr_core::loot::DropRng for RainLootRng<'_> {
    fn next_range(&mut self, min: i32, max: i32) -> i32 {
        self.0.range(min.max(0) as u32, max.max(0) as u32) as i32
    }
    fn next_float(&mut self) -> f32 {
        self.0.rand01()
    }
}

impl Rain {
    pub fn new(seed: u64) -> Self {
        let forced = crate::env::rain_forced().map(|v| v.clamp(0.0, 1.75));
        let start_calm = crate::env::no_weather();
        Self::with_forced(seed, forced, start_calm)
    }

    /// The injectable constructor (a test hook, not a behaviour — the
    /// world-tick tests pin `max_raining` through it; scene_scan precedent).
    pub(crate) fn with_forced(seed: u64, forced: Option<f32>, start_calm: bool) -> Self {
        let rng = (seed ^ 0x2545_F491_4F6C_DD1D).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        let wind = (((rng >> 40) & 0xFFFF) as f32 / 65535.0 - 0.5) * 0.7;
        Self {
            intensity: 0.0,
            target: 0.0,
            raining: false,
            rain_time: 0,
            lady_bug_rain_boost: 0,
            calm: start_calm,
            wind,
            cloud_bg: 0.0,
            flash: 0.0,
            thunder_delay: 0,
            storming: false,
            thunder_distance: 0,
            lightning_speed: 0.0,
            lightning_decay: 0.008,
            drops: Vec::with_capacity(MAX_DROPS),
            sparkles: Vec::with_capacity(64),
            snow: Vec::with_capacity(MAX_SNOW),
            snow_mode: false,
            gfx_quality: 0.0,
            rng,
            spawn_acc: 0.0,
            forced,
            accum: 0.0,
        }
    }

    /// `Main.maxRaining`: the intensity the event ramps toward (the
    /// WaterDistortion rain term reads it, `SceneState.cs:129`).
    pub fn max_raining(&self) -> f32 {
        self.target
    }

    /// `Main.cloudAlpha` clamped to the range the sky tint uses.
    pub fn visual_alpha(&self) -> f32 {
        self.intensity.clamp(0.0, 1.0)
    }

    /// `Main.raining`: the toggle the tile-light scanner reads for the water
    /// fountain (`tile.frameX == 90`) case.
    pub fn is_raining(&self) -> bool {
        self.raining
    }

    /// `Main.IsItRaining` (`Main.cs:2597`): `cloudAlpha > 0f` — the VISIBLE
    /// intensity, NOT the `Main.raining` event flag. The ambience scheduler
    /// (`AmbienceServer.cs:32/50`), the music rain gates (`Main.cs:12100` etc.)
    /// and the storm-lightning rain gate (`WorldGen.cs:71872`) read this, so
    /// they stay "raining" through the post-`StopRain` cloud ramp-down.
    pub fn is_it_raining(&self) -> bool {
        self.intensity > 0.0
    }

    /// `Rain.ClearRain` (`Rain.cs:22-28`): deactivate every drop. The C# calls
    /// it from `SaveAndQuitCallBack` (`WorldGen.cs:6672`) and `genWorldCallBack`
    /// (`:6747`); the port's save-and-quit funnel calls it, and a freshly
    /// generated world boots a fresh `Rain` (empty drops) so the gen-side call
    /// is a no-op there.
    pub(crate) fn clear_rain(&mut self) {
        self.drops.clear();
    }

    /// `Rain.NewRainForced` (`Rain.cs:135-164`): the Eyebrella vanity drip
    /// (`Player.DoEyebrellaRainEffect`, `Player.cs:14308-14331` — the caller is
    /// unported, so this seam has no production writer yet). Unlike `NewRain`
    /// it ignores `cloudAlpha`/snow/atmo: the budget is `maxRain` cut by the
    /// gfx `(1+q)/2` when q<0.9 (`:139-143`), then the first inactive slot takes
    /// the spawn. Returns the slot index, or `maxRain` when full (`:152-155`).
    #[allow(dead_code)] // the Eyebrella caller (Player.cs:14308) is unported
    pub(crate) fn new_rain_forced(
        &mut self,
        world: &World,
        camera: Vec2,
        position: Vec2,
        velocity: Vec2,
    ) -> usize {
        let mut num2 = MAX_DROPS; // `:138`
        let num3 = (1.0 + self.gfx_quality) / 2.0; // `:139`
        if num3 < 0.9 {
            num2 = (num2 as f32 * num3) as usize; // `:140-143`
        }
        // `:144-155`: the first inactive slot under the budget; the port's
        // packed vec has no holes, so its tail IS that slot.
        if self.drops.len() >= num2 {
            return MAX_DROPS;
        }
        // `:159-162`: scale, velocity, rotation, frame type.
        let scale = 1.0 + self.range_signed(-20, 21) as f32 * 0.01;
        let vel = velocity * scale;
        let water_style = crate::biome::calculate_water_style(world, camera) as u32;
        let kind = (water_style * 3 + self.range(0, 3)) as u8;
        let idx = self.drops.len();
        self.drops.push(Drop {
            pos: position,
            vel,
            scale,
            rot: vel.x.atan2(-vel.y),
            kind,
        });
        idx
    }

    /// `Main.rainTime`: ticks left in the event (`Main.cs:64874` drains it).
    pub(crate) fn rain_time(&self) -> i32 {
        self.rain_time
    }

    /// S1-P-i35 — `Main.ladyBugRainBoost += NPC.ladyBugRainTime` clamped at
    /// `NPC.maximumAmountOfTimesLadyBugRainCanStack` (`NPC.cs:81568-81572`,
    /// the `LadyBugKilled` writer): a consumed ladybug bait (4361/4362) adds
    /// its rain boost. The cap is `crate::luck::LADYBUG_RAIN_CAP`.
    pub(crate) fn add_lady_bug_rain_boost(&mut self, add: i32) {
        self.lady_bug_rain_boost =
            (self.lady_bug_rain_boost + add).min(crate::luck::LADYBUG_RAIN_CAP);
    }

    /// `SetOngoingToTemps` (`WorldFile.cs:159-162`): the load applies the
    /// save-time snapshot — the event state comes back and `Main.cloudAlpha`
    /// := `maxRaining`. The triple rode the `.trwld` v14 block (S1-J-7).
    pub(crate) fn restore_saved(&mut self, raining: bool, rain_time: i32, max_raining: f32) {
        self.raining = raining;
        self.rain_time = rain_time;
        self.target = max_raining;
        self.intensity = max_raining;
    }

    /// A test hook (not a behaviour): sets `Main.raining` for the scene-scan
    /// tests without running the event machine (`Main.cs:64725`'s setter).
    #[cfg(test)]
    pub(crate) fn set_raining_for_test(&mut self, raining: bool) {
        self.raining = raining;
    }

    pub fn drop_count(&self) -> usize {
        self.drops.len()
    }

    pub fn snow_count(&self) -> usize {
        self.snow.len()
    }

    /// Storm cloud-bank opacity (`Main.cloudBGAlpha`).
    pub fn cloud_bg(&self) -> f32 {
        self.cloud_bg
    }

    fn next_u64(&mut self) -> u64 {
        self.rng = self.rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn rand01(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u32 << 24) as f32
    }

    /// `Main.NewLightning` (`Main.cs:58708-58733`): the tiered thunder delay,
    /// the distance copy, and the flash rise/decay rates. `FlashyEffectsWorld`
    /// is true for the non-drunk worlds the port generates; the shimmer mute
    /// reads the live `shimmerAlpha`.
    fn new_lightning(&mut self, shimmer_alpha: f32) {
        if shimmer_alpha > 0.0 {
            return;
        }
        self.thunder_delay = if self.range(0, 3) == 0 {
            self.range(1, 31) as i32
        } else if self.range(0, 2) == 0 {
            self.range(31, 121) as i32
        } else {
            self.range(11, 61) as i32
        };
        self.thunder_distance = self.thunder_delay;
        self.lightning_decay = self.range(0, 1000) as f32 / 1000.0 * 0.05 + 0.008;
        self.lightning_speed = self.range(0, 1000) as f32 / 1000.0 * 0.05 + 0.05;
    }

    fn range(&mut self, lo: u32, hi: u32) -> u32 {
        lo + (self.next_u64() % (hi.saturating_sub(lo).max(1) as u64)) as u32
    }

    /// The signed `Main.rand.Next(min, max)` — the thunder pitch draw
    /// (`Next(-10, 11)`, `LegacySoundPlayer.cs:471`) rides it.
    fn range_signed(&mut self, lo: i32, hi: i32) -> i32 {
        lo + (self.next_u64() % (hi.saturating_sub(lo).max(1)) as u64) as i32
    }

    /// Fixed 60 Hz stepping, independent of frame rate. The drop sim and the
    /// visual ramps only — the event lifecycle moved onto the shared world tick
    /// (`update_time`, driven by `world_sim::weather::update_world_tick`) and the
    /// lightning chain runs there too (`thunder_step`). `day_rate` scales the
    /// ramps (`Main.cs:61902-61906`'s `num4 < 1 → 1` clamp shape).
    ///
    /// `shimmer_alpha`, `atmo`, `snow_tile_count` and `zone_rain` are the
    /// `MakeRain`/`NewRain` gate inputs (`Rain.cs:30-207`, S1-L-2).
    ///
    /// `pool` is the W4-g67 splash drain seam: `Some((DustPool, SpawnCtx))`
    /// from the world pass (the d50 contract — collision splashes spawn
    /// through `DustPool::new_dust` on Rain's own stream); `None` keeps the
    /// gate rolling with the spawn dropped (the drop-sim tests).
    ///
    /// `sim` carries the menu/remix/AnyDanger/wind/player-velocity inputs
    /// (W4-g67 residuals) — `Default::default()` is the calm in-world shape.
    pub fn update(
        &mut self,
        world: &World,
        camera: Vec2,
        screen: Vec2,
        zoom: f32,
        dt: f32,
        snow_biome: bool,
        day_rate: i32,
        shimmer_alpha: f32,
        atmo: f32,
        snow_tile_count: i32,
        zone_rain: bool,
        mut pool: Option<(
            &mut tr_core::particles::DustPool,
            &tr_core::particles::SpawnCtx,
        )>,
        sim: DropSimInputs,
    ) {
        self.snow_mode = snow_biome;
        let step = 1.0 / TICKS_PER_SEC;
        let mut acc = self.accum + dt.min(0.25);
        let mut steps = 0;
        while acc >= step && steps < 12 {
            self.tick(
                world,
                camera,
                screen,
                zoom,
                day_rate,
                shimmer_alpha,
                atmo,
                snow_tile_count,
                zone_rain,
                pool.as_mut().map(|(p, c)| (&mut **p, &**c)),
                sim,
            );
            acc -= step;
            steps += 1;
        }
        self.accum = acc.min(step * 12.0);
    }

    fn tick(
        &mut self,
        world: &World,
        camera: Vec2,
        screen: Vec2,
        zoom: f32,
        day_rate: i32,
        shimmer_alpha: f32,
        atmo: f32,
        snow_tile_count: i32,
        zone_rain: bool,
        pool: Option<(&mut tr_core::particles::DustPool, &tr_core::particles::SpawnCtx)>,
        sim: DropSimInputs,
    ) {
        if let Some(f) = self.forced {
            self.intensity = f;
            self.target = f;
            self.raining = f > 0.01;
        } else {
            // M1.4-BG-22 phase 2: the session schedule (timer rolls) is retired —
            // the vanilla state machine (`update_time`) runs on the shared world
            // tick. Only the `cloudAlpha` ramp remains here, per rain tick
            // (documented deviation vs C#'s per-draw ramp, `Main.cs:61909-61924`),
            // scaled by `max(1, dayRate)` (`:61902-61906`).
            let rate = day_rate.max(1) as f32;
            let ramp = RAMP_PER_TICK * rate;
            let delta = (self.target - self.intensity).clamp(-ramp, ramp);
            self.intensity += delta;
        }

        // `cloudBGAlpha` ramps toward the storm strength (kept at 10× the
        // `cloudAlpha` ramp so the deck appears within it; scaled by
        // `max(1, dayRate)` like the `cloudBGActive` ramp it stands in for,
        // `Main.cs:13187/13205`). Runs under the pin too, as before.
        let rate = day_rate.max(1) as f32;
        let bg_ramp = CLOUD_BG_RAMP_PER_TICK * rate;
        let target = if self.intensity > 0.05 {
            self.intensity.min(1.0)
        } else {
            0.0
        };
        let delta = (target - self.cloud_bg).clamp(-bg_ramp, bg_ramp);
        self.cloud_bg = (self.cloud_bg + delta).clamp(0.0, 1.0);

        // The lightning chain (`Main.UpdateWeather`'s
        // `currentDayRateIteration == 0` arm) is stepped once per world tick —
        // `thunder_step`, called from `world_sim::weather::update_world_tick`
        // — not here with the drop sim.

        // Each tick advances the drops at the original 60 Hz velocity.
        self.update_drops(world, camera, screen, zoom, pool, sim);
        self.update_snow(world, camera, screen, zoom);
        if self.snow_mode {
            // Vanilla suppresses rain drops in a snow biome and lets the ambient
            // snow dust carry the weather instead.
            if rain_zone(world, camera) {
                self.spawn_snow(world, camera, screen, zoom);
            }
        } else if self.intensity > 0.01 {
            self.spawn(
                world,
                camera,
                screen,
                zoom,
                shimmer_alpha,
                atmo,
                snow_tile_count,
                zone_rain,
                sim,
            );
        }
        // The splash particles are `DustPool` slots now (W4-g67) — the shared
        // `update_dust` pass ticks them, nothing to advance here.
        // The generic `Dust.Update` fade (`alpha -= 25` per tick from 254).
        for s in &mut self.sparkles {
            s.alpha -= 25.0 / 255.0;
        }
        self.sparkles.retain(|s| s.alpha > 0.0);
    }

    /// `Main.UpdateWeather`'s lightning chain (`Main.cs:58737-58829`) — the
    /// `currentDayRateIteration == 0` arm vanilla runs ONCE per `Main.Update`
    /// (`Main.cs:17228-17231`), so the world tick calls this once per 60 Hz
    /// step, never per dayRate iteration. The thunder-sound delay counts
    /// down; the flash rises via `lightningSpeed` and decays via
    /// `lightningDecay`; at delay 0 the storming roll re-arms with the
    /// rain/wind tier multipliers. `weather` carries the cross-system inputs
    /// the vanilla reads from Main statics.
    ///
    /// Returns the thunder strike cue when the delay reaches 0 this tick
    /// (`Main.cs:58752-58761`, M6-AUDIO-1) — the position, the focus/skip
    /// gate and the cue-slot/pitch rolls all resolve inside Rain's own
    /// stream like the vanilla's `Main.rand` draws. `is_playing(slot)` is
    /// the engine's `SoundInstanceThunder[slot].State == Playing` — the
    /// case-43 re-roll loop reads it live (`LegacySoundPlayer.cs:456-464`).
    pub(crate) fn thunder_step(
        &mut self,
        weather: ThunderInputs,
        is_playing: &dyn Fn(u32) -> bool,
    ) -> Option<ThunderStrike> {
        let mut strike = None;
        if self.thunder_delay > 0 {
            self.thunder_delay -= 1;
            if self.thunder_delay == 0 {
                // `Main.cs:58751-58768`: the strike — the position
                // (`thunderDistance * 15` off the scene centre, clamped to
                // `worldSurface * 16 - 500`) behind the focus/skip gate, then
                // the case-43 cue family (the re-roll loop + the pitch).
                let flip_left = self.range(0, 2) == 0;
                let (x, y) = crate::ambient_sound::thunder_strike_position(
                    (weather.scene_center[0], weather.scene_center[1]),
                    self.thunder_distance as f32,
                    flip_left,
                    weather.world_surface_px,
                );
                if crate::ambient_sound::thunder_plays_when_focused(
                    true,
                    weather.thunder_skip_sound,
                ) {
                    let mut cue_roll = |lo: i32, hi: i32| self.range_signed(lo, hi);
                    let (cue, slot, pitch) =
                        crate::ambient_sound::thunder_cue_pick(&mut cue_roll, is_playing);
                    strike = Some(ThunderStrike {
                        cue,
                        slot,
                        x,
                        y,
                        pitch,
                    });
                }
                // The vanilla drains `thunderDistance` to 0 with the strike;
                // the re-arm roll below reads it as 0.
                self.thunder_distance = 0;
            }
        }
        if self.lightning_speed > 0.0 {
            // `:58771-58778`: the flash rise, capped at 1.
            self.flash += self.lightning_speed;
            if self.flash >= 1.0 {
                self.flash = 1.0;
                self.lightning_speed = 0.0;
            }
        } else if self.flash > 0.0 {
            // `:58779-58783`: the flash decay.
            self.flash -= self.lightning_decay;
            if self.flash < 0.0 {
                self.flash = 0.0;
            }
        } else if self.thunder_delay <= 0 && weather.above_rock_layer && weather.atmo >= 1.0 {
            // M5-SEED-2: the storm-lightning gates of `WorldGen.SpawnStormLightning`
            // (`WorldGen.cs:71864-71884`), the damaging strike roll the port's
            // thunder chain stands in for (the port has no thunder-bolt
            // projectile yet — see the module note). They mirror the C#'s
            // first three statements:
            //
            // - `:71866-71869` — `if (Main.noLightningSeed && !Main.moreLightningSeed)
            //   return;` suppresses the STORM lightning. The two-set
            //   combination is NOT suppressed (`flag`, `:71870`, is false),
            //   so it behaves exactly like a classic world. The graveyard arm
            //   below is a different chain (`Main.cs:58830-58833`) and carries
            //   no seed gate in vanilla, so it keeps running.
            // - `:71872-71874` — `if (!Main.IsItStorming && (!flag || !Main.IsItRaining))
            //   return;`: the storm arm normally requires `IsItStorming`;
            //   `moreLightning` alone relaxes it to storming OR raining.
            // - `:71875-71881` — the base interval (150) is halved under the
            //   same flag (the port's own chain window, 600/1600, halves the
            //   same way). The C#'s extra `* Main.cloudAlpha` for a rain
            //   without a storm has no port input — `ThunderInputs` carries
            //   no `cloudAlpha`, so the storm case alone is modelled.
            let more_only = weather.more_lightning && !weather.no_lightning;
            let storm_gate = !(weather.no_lightning && !weather.more_lightning)
                && (weather.storming || (more_only && weather.raining));
            // `:58784-58829`: the storming roll. The tier multipliers shrink
            // the delay window with heavier rain and stronger wind targets.
            if storm_gate {
                let mut num3 = 600.0f32;
                let mut num4 = 1600.0f32;
                // `:71875-71881`: `num = (int)((float)num / num2)` with
                // `num2 = 2f` — the moreLightning world strikes twice as often.
                if more_only {
                    num3 *= 0.5;
                    num4 *= 0.5;
                }
                if weather.max_raining > 0.8 {
                    num3 *= 0.6;
                    num4 *= 0.8;
                }
                if weather.max_raining > 0.7 {
                    num3 *= 0.7;
                    num4 *= 0.9;
                }
                if weather.max_raining > 0.6 {
                    num3 *= 0.8;
                    num4 *= 0.95;
                }
                if weather.wind_target.abs() > 0.7 {
                    num3 *= 0.6;
                    num4 *= 0.8;
                }
                if weather.wind_target.abs() > 0.6 {
                    num3 *= 0.7;
                    num4 *= 0.9;
                }
                if weather.wind_target.abs() > 0.5 {
                    num3 *= 0.8;
                    num4 *= 0.95;
                }
                let num5 = self.range(num3 as u32, num4 as u32) as f32
                    * ((1.0 - weather.max_raining + 1.0) / 2.0)
                    * ((1.0 - weather.wind_target + 1.0) / 2.0);
                if self.range(0, num5.max(1.0) as u32) == 0 {
                    self.new_lightning(weather.shimmer_alpha);
                }
            } else if weather.graveyard_intensity >= 0.9 && self.range(0, 120) == 0 {
                // `:58830-58833`: the graveyard lightning.
                self.new_lightning(weather.shimmer_alpha);
            }
        }
        strike
    }

    /// `Main.UpdateTime`'s rain section (`Main.cs:64833-64949`), driven once per
    /// 60 Hz world tick from `world_sim::weather::update_world_tick` — after the
    /// dayRate wind loop, before `Sandstorm.UpdateTime` (`Main.cs:64966`).
    ///
    /// C# order: the `ladyBugRainBoost` drain (`:64833-64836`), the outer
    /// `(netMode != 1 && !gameMenu) && (isThereAWorldSurface || remixWorld)` gate
    /// (`:64846` — netMode 0 and an in-game tick make the first conjunct true),
    /// the raining branch (`:64864-64888`), then the not-raining start arms
    /// (`:64890-64923`). The slime-rain block after them (`:64925-64949`) is M4
    /// events. `FreezeRainPower` does not exist in the port (its gate resolves
    /// false); `start_gates_open` carries the C# `!slimeRain && !LanternsUp &&
    /// !NextNightIsLanternNight` conjunct (`:64890`, W4-g67 census — the weather
    /// tick derives it from `World.slime_rain` + `WorldEvents`). There is no
    /// `Main.dayTime` gate on the start arms (only the slime roll is day-gated).
    pub(crate) fn update_time(
        &mut self,
        day_rate: i32,
        world: &World,
        any_player_120: bool,
        remix_world: bool,
        player_luck: f32,
        start_gates_open: bool,
    ) {
        // TR_RAIN pin / TR_NO_WEATHER: port-only scheduler switches — the pin
        // overrides the state machine exactly as the retired session schedule was
        // skipped under it, and `start_calm` disabled that schedule too.
        if self.forced.is_some() || self.calm {
            return;
        }
        // `Main.cs:64833-64836`.
        if self.lady_bug_rain_boost > 0 {
            self.lady_bug_rain_boost -= day_rate;
        }
        // `Main.cs:2788`: `isThereAWorldSurface => worldSurface > 50.0`; the
        // `remixWorld` half reads Wind's stand-in field.
        let world_surface_ok = world.world_surface > 50.0;
        if !(world_surface_ok || remix_world) {
            return;
        }
        if self.raining {
            // `Main.cs:64864-64888` — the drain and the strength-change roll.
            self.rain_time -= day_rate; // `:64874`
            if day_rate > 0 {
                // `:64875` — always true today (`day_rate()` is 60|1); the gate is
                // the C# shape.
                let num = 86400 / day_rate / 24; // `:64877`
                if self.rain_time <= 0 {
                    self.stop_rain(); // `:64880`
                } else if self.range(0, (num * 2) as u32) == 0 {
                    self.change_rain(); // `:64882-64885`
                }
            }
        } else if start_gates_open {
            // `Main.cs:64890`: `!slimeRain && !LanternsUp &&
            // !NextNightIsLanternNight` — the caller derives the conjunct
            // (W4-g67 census); with it closed no start roll runs at all.
            let num2 = 86400 / if day_rate == 0 { 1 } else { day_rate }; // `:64892`
            // `:64893`: `!FreezeRainPower.Enabled && Main.dayRate != 0` — the
            // freeze power is port-absent.
            let mut flag = day_rate != 0;
            if flag && !any_player_120 {
                // `:64894-64909`: the 255-player `statLifeMax >= 120` scan; the
                // port has one player and the tick computes the bool.
                flag = false;
            }
            if flag {
                // The start arms in C# order — each gate short-circuits before
                // its roll, exactly like the `&&` chains.
                if self.range(0, ((num2 as f64) * 5.75) as u32) == 0 {
                    self.start_rain(world, remix_world, player_luck); // `:64912-64914`
                } else if self.cloud_bg >= 1.0 && self.range(0, ((num2 as f64) * 4.25) as u32) == 0
                {
                    // `:64916-64918`: `cloudBGActive >= 1f` reads the
                    // `cloudBGAlpha` stand-in (`cloud_bg`, documented).
                    self.start_rain(world, remix_world, player_luck);
                } else if self.lady_bug_rain_boost > 0 && self.range(0, num2 as u32) == 0 {
                    self.start_rain(world, remix_world, player_luck); // `:64920-64922`
                }
            }
        }
    }

    /// `Main.StartRain` (`Main.cs:64659-64727`). The `instant`/`strengthOverride`
    /// params default to the C# values (the port's callers never pass them) and
    /// `IsRainingForever` has no save field — both cited inline. `player_luck`
    /// is `GetPlayerWithHighestLuck().luck` (`Player.cs:18213-18229` — the
    /// port's single player; a fresh `Player()` with luck 0 when none).
    pub(crate) fn start_rain(&mut self, world: &World, remix_world: bool, player_luck: f32) {
        // `Main.cs:64661`.
        if !remix_world && world.world_surface > 50.0 && !self.raining {
            // `Main.cs:64663-64667`: `range = 25`; the tenth-anniversary 5 needs
            // anniversary worlds (`tenthAnniversaryWorld`) — none is generated.
            let range = 25;
            // `Main.cs:64668`: `GetPlayerWithHighestLuck().RollLuck(range)` —
            // consumed on the Rain stream like every rain roll (WEATHER-2); a
            // hit branch draws 2 more, exactly like the C# branch shape.
            let hit = {
                let mut loot_rng = RainLootRng(self);
                tr_core::loot::roll_luck(&mut loot_rng, player_luck, range) == 0
            };
            if hit {
                // `Main.cs:64670-64673`: the coin-rain value — CONSUMED, not
                // stored: coin projectiles are unported (M2/M4). The would-be
                // `Main.coinRain` is `(int)(Next(75,151) * 100 * 100 *
                // maxTilesX/4200f)`.
                let num = world.width as f32 / 4200.0; // `Main.maxTilesX`
                let coin_rain = self.range(75, 151) as i32 * 100 * 100;
                let _would_be_coin_rain = (coin_rain as f32 * num) as i32;
            }
        }
        // `Main.cs:64676-64701`: the `rainTime` ladder.
        let num3 = 86400 / 24; // `num2`/`num3` at `:64676-64677`
        let mut num4 = self.range((num3 * 8) as u32, 86400) as i64; // `:64678`
        if self.range(0, 3) == 0 {
            num4 += self.range(0, num3 as u32) as i64; // `:64679-64682`
        }
        if self.range(0, 4) == 0 {
            num4 += self.range(0, (num3 * 2) as u32) as i64; // `:64683-64686`
        }
        if self.range(0, 5) == 0 {
            num4 += self.range(0, (num3 * 2) as u32) as i64; // `:64687-64690`
        }
        if self.range(0, 6) == 0 {
            num4 += self.range(0, (num3 * 3) as u32) as i64; // `:64691-64694`
        }
        if self.range(0, 7) == 0 {
            num4 += self.range(0, (num3 * 4) as u32) as i64; // `:64695-64698`
        }
        if self.range(0, 8) == 0 {
            num4 += self.range(0, (num3 * 5) as u32) as i64; // `:64699-64702`
        }
        // `Main.cs:64703-64719`: the strength ladder.
        let mut num5 = 1.0f32;
        if self.range(0, 2) == 0 {
            num5 += 0.05;
        }
        if self.range(0, 3) == 0 {
            num5 += 0.1;
        }
        if self.range(0, 4) == 0 {
            num5 += 0.15;
        }
        if self.range(0, 5) == 0 {
            num5 += 0.2;
        }
        // `Main.cs:64720-64723`: `IsRainingForever` has no save field — the
        // assignment always runs (the C# gate is skipped, cited).
        self.rain_time = (num4 as f32 * num5) as i32; // `:64722` `(int)((float)num4 * num5)`
        self.change_rain(); // `:64724` — `ChangeRain(instant, strengthOverride)` defaults
        self.raining = true; // `:64725`
    }

    /// `Main.ChangeRain` (`Main.cs:64728-64740`). The `instant`/`strengthOverride`
    /// params default to the C# values (the port's callers never pass them);
    /// `numClouds` reads the static cloud-field stand-in and `cloudBGActive >= 1f`
    /// the `cloudBGAlpha` stand-in — the live cloud system (`Main.cs:58952-59036`)
    /// is unported.
    fn change_rain(&mut self) {
        let val = if self.cloud_bg >= 1.0 || NUM_CLOUDS_STAND_IN > 150 {
            // `Main.cs:64730` band 1: `Next(40,91)` / `Next(20,91)` ×0.01.
            if self.range(0, 3) != 0 {
                self.range(40, 91) as f32 * 0.01
            } else {
                self.range(20, 91) as f32 * 0.01
            }
        } else if NUM_CLOUDS_STAND_IN > 100 {
            // Band 2: dead with the 100-cloud stand-in (100 is not > 100).
            if self.range(0, 3) != 0 {
                self.range(20, 61) as f32 * 0.01
            } else {
                self.range(10, 71) as f32 * 0.01
            }
        } else {
            // Band 3: `Next(5,31)` / `Next(5,41)` ×0.01.
            if self.range(0, 3) != 0 {
                self.range(5, 31) as f32 * 0.01
            } else {
                self.range(5, 41) as f32 * 0.01
            }
        };
        // `Main.cs:64731-64734`: the `IsRainingForever` `max(val, 0.1f)` floor —
        // no save field, skipped (cited).
        self.target = val; // `:64735` `Main.maxRaining = val`
        // `:64736-64739`: the `instant` arm snaps `cloudAlpha` — the port's
        // callers never pass it (default false) and the visual ramps own
        // `cloudAlpha`.
    }

    /// `Main.StopRain` (`Main.cs:64644-64658`): the event clears and `cloudAlpha`
    /// decays through the visual ramp (`Main.cs:61928-61937`'s not-raining arm —
    /// `target` 0). The `instant` default false is the ported arm;
    /// `IsRainingForever` has no save field.
    pub(crate) fn stop_rain(&mut self) {
        self.rain_time = 0; // `:64648`
        self.raining = false; // `:64649`
        self.target = 0.0; // `:64650` `Main.maxRaining = 0f`
        // `:64651` `Main.coinRain = 0` — never stored (see `start_rain`).
        // `:64652-64655`: the `instant` arm would snap `cloudAlpha` to 0 — the
        // port's callers use the default and the ramp decays the visible
        // intensity.
    }

    /// Takes a batch of `WaterfallManager.TrySparkling` spawns (world pixels).
    pub fn spawn_sparkles(
        &mut self,
        spawns: impl IntoIterator<Item = tr_core::waterfall::SparkleSpawn>,
    ) {
        for s in spawns {
            if self.sparkles.len() >= 256 {
                break;
            }
            self.sparkles.push(Sparkle {
                pos: Vec2::new(s.x, s.y),
                alpha: 254.0 / 255.0,
            });
        }
    }

    /// `Rain.Update` (`Rain.cs:96-133`): move the drop, then kill it on the
    /// `Collision.SolidCollision(position, 2, 2)` / below-screen /
    /// `WetCollision(position, 2, 2)` chain (`:117`) — the ONE combined
    /// branch rolls the gfx gate and spawns the splash for ALL THREE kills
    /// (`:119-131`), the below-screen one included (the dust's own cull
    /// margin decides whether it survives). The `gameMenu`/`remixWorld`
    /// kill-only arms (`:103-116`) are ported verbatim (W4-g67 residuals):
    /// menu kills below `+2000`, remix below `+100`, NEITHER rolls the gate
    /// nor splashes; the production frame gates the weather cluster out of
    /// menus, so `sim.game_menu` only fires for a residual drop handed a
    /// menu tick (the C# `DrawRain` still calls `Update` there).
    ///
    /// `pool` is the splash drain seam (W4-g67): the world tick hands it the
    /// App's `DustPool` + `SpawnCtx` (the D50 contract — the spawn rides
    /// `DustPool::new_dust` and Rain's own stream); the drop-sim tests pass
    /// `None` (no pool — the gate still rolls, the spawn is dropped).
    fn update_drops(
        &mut self,
        world: &World,
        camera: Vec2,
        screen: Vec2,
        zoom: f32,
        mut pool: Option<(&mut tr_core::particles::DustPool, &tr_core::particles::SpawnCtx)>,
        sim: DropSimInputs,
    ) {
        // `Rain.cs:117`: `screenPosition.Y + screenHeight + 100f` — world px,
        // no zoom term (the port's camera-centred view maps it through half.y).
        let view_bottom = camera.y + screen.y * 0.5 / zoom;
        let bottom = view_bottom + 100.0;
        let mut i = 0;
        while i < self.drops.len() {
            let mut d = self.drops[i];
            // `:102`: `position += velocity` — runs under EVERY arm, the
            // menu/remix kill-only ones included.
            d.pos += d.vel;
            if sim.game_menu {
                // `:103-109`: menu kill-only — below `+2000`, no gate roll,
                // no splash (the C# else-if chain never reaches `:117`).
                if d.pos.y > view_bottom + 2000.0 {
                    self.drops.swap_remove(i);
                } else {
                    self.drops[i] = d;
                    i += 1;
                }
                continue;
            }
            if sim.remix_world {
                // `:110-116`: remix kill-only — below `+100`, same shape.
                if d.pos.y > bottom {
                    self.drops.swap_remove(i);
                } else {
                    self.drops[i] = d;
                    i += 1;
                }
                continue;
            }
            // `:117`: SolidCollision OR below-screen OR WetCollision — the C#
            // `||` short-circuits, so the wet probe runs only when dry+on-screen.
            let solid = crate::player::Player::solid_collision(world, d.pos.x, d.pos.y, 2.0, 2.0);
            let below = d.pos.y > bottom;
            let wet = !solid
                && !below
                && crate::player::wet_collision(world, d.pos.x, d.pos.y, 2.0, 2.0);
            if solid || below || wet {
                self.drops.swap_remove(i);
                // `:120`: the gfx-quality gate rolls Main.rand for EVERY kill
                // of the combined branch (it sits outside NewDust, so it
                // rolls even when the dust would cull or no pool is handed).
                if (self.range(0, 100) as f32) < self.gfx_quality * 100.0 {
                    if let Some((p, ctx)) = pool.as_mut() {
                        self.spawn_splash(p, ctx, world, camera, d.pos, d.vel);
                    }
                }
            } else {
                self.drops[i] = d;
                i += 1;
            }
        }
    }

    /// `Rain.MakeRain` (`Rain.cs:30-89`, S1-L-2) + `Rain.NewRain`
    /// (`:166-207`): the per-tick drop spawn. Gates: `!ZoneRain ||
    /// shimmerAlpha > 0f` suppress spawning (the C# also early-returns on
    /// netMode 2 / paused / menu — the port's frame gates the weather cluster
    /// out of those). The attempt rate is `screenWidth / MaxWorldViewSize.X *
    /// 25 * (0.25 + cloudAlpha)` with the `NPC.AnyDanger(quickBossNPCCheck)`
    /// 0.05 throttle (`:36-45`, floored to 1 attempt when any would remain).
    /// Each attempt draws the X/Y spawn window (the rising-player `num3`
    /// widening `:50-53`, the wind/player-X drift `:56-57`, the world-X clamp
    /// `:58-65`), then spawns only through open sky — or through ANYTHING on
    /// remix/menu (`:84`). `NewRain` caps the live count by `maxRain *
    /// cloudAlpha`, the gfx `(1+q)/2` cut when q<0.9 (`:176-180`), the
    /// snow-tile suppression `(1-(snow/threshold)^2)` (`:181-182`) and
    /// `atmo^9` (0 when `atmo<0.4`, `:183-186`), and sets the
    /// `waterStyle*3 + rand(3)` frame type (`:205`).
    ///
    /// Cadence deviation: the C# loops `for (i=0; i<num; i++)` (ceil(num)
    /// attempts per call); the port accumulates the fractional rate across
    /// ticks (`spawn_acc`) — same average, and the live-count budget caps both.
    fn spawn(
        &mut self,
        world: &World,
        camera: Vec2,
        screen: Vec2,
        zoom: f32,
        shimmer_alpha: f32,
        atmo: f32,
        snow_tile_count: i32,
        zone_rain: bool,
        sim: DropSimInputs,
    ) {
        if !zone_rain || shimmer_alpha > 0.0 {
            return;
        }
        // `Rain.cs:36`: `screenWidth / MaxWorldViewSize.X(1920) * 25 * (0.25 + cloudAlpha)`.
        let mut rate = screen.x / 1920.0 * 25.0 * (0.25 + self.intensity);
        // `Rain.cs:37-45`: `NPC.AnyDanger(quickBossNPCCheck: true)` throttles to
        // 0.05x, floored to 1 attempt when the unthrottled rate would leave any.
        if sim.any_danger {
            let unthrottled = rate;
            rate *= 0.05;
            if unthrottled > 1.0 && rate < 1.0 {
                rate = 1.0;
            }
        }
        self.spawn_acc += rate;
        let count = self.spawn_acc.floor();
        self.spawn_acc -= count;
        if count <= 0.0 {
            return;
        }
        // `Rain.NewRain` (`:166-207`): the active-slot budget.
        let mut num2 = (MAX_DROPS as f32 * self.intensity) as i32; // `:169`
        if num2 > MAX_DROPS as i32 {
            num2 = MAX_DROPS as i32; // `:170-173`
        }
        // `:174-175`: the `Math.Max(...)` depth term is computed and DISCARDED
        // (no assignment) — dead in the C#, omitted.
        // `:176-180`: the gfx-quality cut `(1+q)/2` when q<0.9.
        let gfx = (1.0 + self.gfx_quality) / 2.0;
        if (gfx as f64) < 0.9 {
            num2 = (num2 as f32 * gfx) as i32;
        }
        // `:181-182`: snow suppression. `SceneMetrics.SnowTileThreshold` is
        // skyblock-aware (300 low-tiles / 1500 normal, `SceneMetrics.cs:154-163`);
        // the flag rides `World.skyblock_low_tiles` (W4-g67 — the census
        // `World::scan_skyblock_low_tiles` fills it at load/gen).
        let snow_threshold =
            tr_core::scene::snow_tile_threshold(world.skyblock_low_tiles).max(1) as f32;
        let snow_ratio = (snow_tile_count.max(0) as f32 / snow_threshold).clamp(0.0, 1.0);
        num2 = ((num2 as f32) * (1.0 - snow_ratio * snow_ratio)) as i32;
        num2 = ((num2 as f64) * (atmo as f64).powi(9)) as i32; // `:182` double cast
        if (atmo as f64) < 0.4 {
            num2 = 0; // `:183-186`
        }
        if self.drops.len() >= num2.max(0) as usize {
            return;
        }
        let half = screen * 0.5 / zoom;
        let screen_pos = camera - half; // `Main.screenPosition`
        let max_x = ((world.width - 1) * 16) as f32; // `:62-64`
        for _ in 0..count as usize {
            if self.drops.len() >= num2.max(0) as usize || self.drops.len() >= MAX_DROPS {
                break;
            }
            // `:49-53`: the spawn window widens upward while the player rises.
            let mut num3 = 600;
            if sim.player_vel.y < 0.0 {
                num3 += (sim.player_vel.y.abs() * 30.0) as i32;
            }
            // `:54-55`: X ~ Next(screenX - num3, screenX + screenWidth + num3),
            // Y = screenY - Next(20, 100).
            let lo = screen_pos.x as i32 - num3;
            let hi = screen_pos.x as i32 + screen.x as i32 + num3;
            let mut x = self.range_signed(lo, hi) as f32;
            let y = screen_pos.y - self.range(20, 100) as f32;
            // `:56-57`: the wind + player-X drift.
            x -= sim.wind_current * 15.0 * 40.0;
            x += sim.player_vel.x * 40.0;
            // `:58-65`: clamp X into the world.
            x = x.clamp(0.0, max_x);
            // `:66-83`: the tile probe coords, clamped to the world bounds.
            let tx = ((x as i32) / 16).clamp(0, world.width - 1);
            let ty = ((y as i32) / 16).clamp(0, world.height - 1);
            // `:84`: remix/menu spawn through ANYTHING; else open sky only
            // (`!WorldGen.SolidTile && wall <= 0`).
            let open = sim.remix_world
                || sim.game_menu
                || (!crate::projectile::solid_tile(world, tx, ty)
                    && world.tile(tx, ty).is_some_and(|t| t.wall == 0));
            if !open {
                continue;
            }
            // `Rain.NewRain` `:202-205`: scale, velocity, rotation, frame type.
            let scale = 1.0 + self.range_signed(-20, 21) as f32 * 0.01;
            // `GetRainFallVelocity` (`:91-94`): `(windSpeedCurrent * 18, 14)`.
            let vel = Vec2::new(sim.wind_current * 18.0, 14.0) * scale;
            // `:205`: `type = (byte)(Main.waterStyle * 3 + Main.rand.Next(3))`.
            let water_style = crate::biome::calculate_water_style(world, camera) as u32;
            let kind = (water_style * 3 + self.range(0, 3)) as u8;
            self.drops.push(Drop {
                pos: Vec2::new(x, y),
                vel,
                scale,
                rot: vel.x.atan2(-vel.y),
                kind,
            });
        }
    }

    fn update_snow(&mut self, world: &World, camera: Vec2, screen: Vec2, zoom: f32) {
        let bottom = camera.y + screen.y * 0.5 / zoom + 40.0 / zoom;
        let px = tr_core::TILE_PX;
        let mut i = 0;
        while i < self.snow.len() {
            let mut s = self.snow[i];
            s.phase += 0.06;
            s.pos.x += s.vel.x + s.phase.sin() * 0.45;
            s.pos.y += s.vel.y;
            let tx = (s.pos.x / px).floor() as i32;
            let ty = (s.pos.y / px).floor() as i32;
            let hit = world.tile(tx, ty).is_some_and(|t| {
                (t.active() && (t.kind as usize) < td::TILE_COUNT && td::TILE_SOLID.has(t.kind))
                    || t.liquid > 0
            });
            if hit || s.pos.y > bottom {
                self.snow.swap_remove(i);
            } else {
                self.snow[i] = s;
                i += 1;
            }
        }
    }

    /// Ambient snowfall: vanilla spawns `500 * screenWidth/1920` dust flakes in a snow
    /// biome, multiplied by `1 + 2*cloudAlpha` during a storm. The spawn cap
    /// carries the gfx-quality budget (`Main.cs:13057`'s
    /// `num2 * (gfxQuality / 2f + 0.5f) + num2 * 0.1f` — S1-B-6), so the
    /// low-quality mode flakes fewer.
    fn spawn_snow(&mut self, world: &World, camera: Vec2, screen: Vec2, zoom: f32) {
        let px = tr_core::TILE_PX;
        let half = screen * 0.5 / zoom;
        let rate = screen.x / 1920.0 * 2.4 * (0.6 + 1.8 * self.intensity);
        let cap = crate::world_sim::gfx_quality::snow_cap(MAX_SNOW, self.gfx_quality);
        if self.snow.len() >= cap {
            return;
        }
        let count = rate.max(0.0).floor() as usize;
        for _ in 0..count {
            if self.snow.len() >= cap {
                break;
            }
            let x = camera.x + (self.rand01() * 2.0 - 1.0) * (half.x + 90.0);
            let y = camera.y - half.y - 10.0 - self.rand01() * 60.0;
            let tx = (x / px).floor() as i32;
            let ty = (y / px).floor() as i32;
            if !world
                .tile(tx, ty)
                .is_some_and(|t| !t.active() && t.wall == 0)
            {
                continue;
            }
            let scale = 0.55 + self.rand01() * 0.45;
            let vy = (2.2 + self.rand01() * 2.4) * scale;
            let vx = self.wind * (5.0 + 9.0 * self.intensity) + (self.rand01() * 2.0 - 1.0) * 0.5;
            let frame = self.range(0, 3) as u8;
            let phase = self.rand01() * std::f32::consts::TAU;
            self.snow.push(Snowflake {
                pos: Vec2::new(x, y),
                vel: Vec2::new(vx, vy),
                scale,
                phase,
                frame,
            });
        }
    }

    /// The `Rain.Update` splash recipe (`Rain.cs:122-130`, W4-g67): one
    /// `Dust.NewDust(position - velocity, 2, 2, Dust.dustWater())` through the
    /// shared pool (the D50 contract — its rolls ride Rain's own stream via
    /// `SplitmixRand`, the WEATHER-2 per-consumer convention), then the eight
    /// slot writes on the returned index — the refused sentinel included
    /// (`Main.dust[6000]`, the sand.rs convention). The caller rolled the
    /// `Next(100) < gfxQuality * 100` gate (`:120`).
    fn spawn_splash(
        &mut self,
        pool: &mut tr_core::particles::DustPool,
        ctx: &tr_core::particles::SpawnCtx,
        world: &World,
        camera: Vec2,
        pos: Vec2,
        vel: Vec2,
    ) {
        // `Dust.dustWater()` (`Dust.cs:402-418`): the per-water-style splash id.
        let water_style = crate::biome::calculate_water_style(world, camera) as i32;
        let idx = pool.new_dust(
            &mut crate::app::SplitmixRand(&mut self.rng),
            ctx,
            world,
            pos - vel,
            2,
            2,
            tr_core::particles::dust_water(water_style),
            0.0,
            0.0,
            0,
            tr_core::particles::Color::default(),
            1.0,
        );
        let d = pool.slot_mut(idx);
        // `:123-130` — the eight writes, in order.
        d.position.x -= 2.0;
        d.position.y += 2.0;
        d.alpha = 38;
        d.velocity *= 0.1;
        d.velocity += -vel * 0.025;
        d.velocity.y -= 2.0;
        d.scale = 0.6;
        d.no_gravity = true;
    }

    /// Draw in the always-rebuilt world-effect range (screen space, under the
    /// map batches and the interface), so the cached world instance range (G6)
    /// is unaffected by the moving drops.
    pub fn draw(
        &self,
        r: &mut SpriteRenderer,
        lookups: &assets::Lookups,
        light: &LightMap,
        camera: Vec2,
        zoom: f32,
        screen: Vec2,
    ) {
        // The splash dust draws through the pool's own `DrawDust` entry
        // (`draw_splashes`, the W4-g67 seam) — the drops/snow/flash still gate
        // this pass.
        if self.drops.is_empty() && self.snow.is_empty() && self.flash <= 0.01 {
            return;
        }
        if self.flash > 0.01 {
            r.push_fx_rect(
                [0.0, 0.0],
                [screen.x, screen.y],
                [1.0, 0.98, 0.9, self.flash * 0.38],
            );
        }
        let to_screen = |p: Vec2| {
            [
                (p.x - camera.x) * zoom + screen.x * 0.5,
                (p.y - camera.y) * zoom + screen.y * 0.5,
            ]
        };
        for d in &self.drops {
            let Some(uv) = lookups.rain_uv(d.kind) else {
                continue;
            };
            let c = light_at(light, d.pos);
            r.push_fx_sprite_rot(
                to_screen(d.pos),
                [DROP_W * d.scale * zoom, DROP_H * d.scale * zoom],
                uv,
                [c[0] * 0.85, c[1] * 0.85, c[2] * 0.85, 1.0],
                d.rot,
            );
        }
        for s in &self.snow {
            let Some(uv) = lookups.dust_uv(760, s.frame as i32 * 10, 8, 8) else {
                continue;
            };
            let c = light_at(light, s.pos);
            let shade = [c[0] * 0.92 + 0.12, c[1] * 0.95 + 0.12, c[2] + 0.14];
            let a = (s.scale * 0.75).clamp(0.35, 0.9);
            r.push_fx_sprite_rot(
                to_screen(s.pos),
                [8.0 * s.scale * zoom, 8.0 * s.scale * zoom],
                uv,
                [shade[0], shade[1], shade[2], a],
                0.0,
            );
        }
        // `WaterfallManager.TrySparkling`'s `Dust` 43 sparkles: scale 0.5, the
        // spawn light colour, the generic dust fade.
        for s in &self.sparkles {
            let Some(uv) = lookups.dust_uv(430, 0, 8, 8) else {
                continue;
            };
            let c = light_at(light, s.pos);
            r.push_fx_sprite_rot(
                to_screen(s.pos),
                [8.0 * 0.5 * zoom, 8.0 * 0.5 * zoom],
                uv,
                [c[0], c[1], c[2], s.alpha],
                0.0,
            );
        }
    }
}

/// Vanilla only rains while `SceneMetrics.ZoneRain` is set; approximate it by
/// requiring the camera to be near that column's surface.
fn rain_zone(world: &World, camera: Vec2) -> bool {
    let tx = (camera.x / tr_core::TILE_PX) as i32;
    let ty = (camera.y / tr_core::TILE_PX) as i32;
    ty < world.surface_of(tx) + 24
}

/// The splash-dust family — every id `Dust.dustWater()` (`Dust.cs:402-418`)
/// can return: the water-style-0 default 33 and the per-style rows
/// 98..=105 / 123 / 288. The shared `UpdateDust` hook ([`water_splash_arm`])
/// and the `DrawDust` entry ([`draw_splashes`]) filter on it.
pub(crate) fn is_splash_type(t: i32) -> bool {
    matches!(t, 33 | 98..=105 | 123 | 288)
}

/// `Dust.UpdateDust`'s splash-family arm (`Dust.cs:1834-1874`, W4-g67): the
/// `308 | 33 | 52 | 266 | 98..=105 | 123 | 288` if-block — a standalone
/// statement of the C#'s FIRST ladder (between the `:477` move and the fresh
/// `if (type >= 130 …)` chain head at `:2130`), so the hook returns `false`
/// and the second chain's final `else if (type != 304 && noGravity)` tail
/// (`:2335`) still runs alongside — the minecart/sand convention. The arm
/// draws NO `Main.rand` (the probes are pure collision reads); the shared
/// loop's ChildSafety arm never fires for these ids (`ChildSafety.cs:32`).
///
/// The 308/52/266 members of the C# list are carried verbatim; today only
/// the `dustWater()` family spawns through this seam.
pub(crate) fn water_splash_arm(d: &mut tr_core::particles::Dust, world: &World) -> bool {
    if !matches!(
        d.dust_type,
        308 | 33 | 52 | 266 | 98..=105 | 123 | 288
    ) {
        return false;
    }
    // `:1836-1844`: the still-in-X arm — solid contact zeroes the scale,
    // then the spin and the nibble run regardless.
    if d.velocity.x == 0.0 {
        if crate::player::Player::solid_collision(world, d.position.x, d.position.y, 2.0, 2.0) {
            d.scale = 0.0;
        }
        d.rotation += 0.5;
        d.scale -= 0.01;
    }
    // `:1845-1849`: the wet probe fades the splash into the surface.
    if crate::player::wet_collision(world, d.position.x, d.position.y, 4.0, 4.0) {
        d.alpha += 20;
        d.scale -= 0.1;
    }
    // `:1850-1855`: the fade pair, then the alpha-255 kill.
    d.alpha += 2;
    d.scale -= 0.005;
    if d.alpha > 255 {
        d.scale = 0.0;
    }
    // `:1856-1859`: the fall-speed clamp.
    if d.velocity.y > 4.0 {
        d.velocity.y = 4.0;
    }
    // `:1860-1873`: the no-gravity spray — the sign-picked spin, the grow,
    // the X stretch and the sink.
    if d.no_gravity {
        if d.velocity.x < 0.0 {
            d.rotation -= 0.2;
        } else {
            d.rotation += 0.2;
        }
        d.scale += 0.03;
        d.velocity.x *= 1.05;
        d.velocity.y += 0.15;
    }
    false
}

/// `Main.DrawDust`'s generic path (`Main.cs:38836-39018`, W4-g67) over the
/// pool's splash-family slots (`dustWater()`, [`is_splash_type`]): the light
/// sample at `(position + 4) / 16` (`:38968`), the `GetAlpha` tint
/// (`:38973`, the default formula for the whole family — `dust_table`), the
/// `(4, 4)` origin rotated draw at `GetVisualScale` (`:39005`), the
/// `GetColor` overlay when the spawn colour is non-zero (`:39006-39012`,
/// never for rain — `default(Color)`), and the full-dark kill (`:39015`).
/// The pool's `maxDustToDraw` scan bound is the update loop's own kill
/// (`Dust.cs:2393-2395`); this entry walks the live family slots.
///
/// The 4×4 on-screen cull rect (`:38832`/`:38847`) is ported verbatim
/// (W4-g67 residuals): `Main.DrawDust`'s live rect is the ±1000/±1050 slack
/// box (`screenPosition + (-1000, -1050, screenWidth+2000, screenHeight+2100)`
/// — `:38832` overwrites `:38831`'s ±500/±50 box; the trail types'
/// `rectangle = rectangle2` restore at `:38845` is a no-op because both hold
/// the same wide rect). A dust whose 4×4 box misses it is DEACTIVATED
/// (`:39020-39022`'s else arm), not merely skipped — the old "documented
/// widening" dropped that kill. The rect is world-px (no zoom term), the
/// port maps it through the camera-centred view's top-left.
pub(crate) fn draw_splashes(
    pool: &mut tr_core::particles::DustPool,
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    light: &LightMap,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
) {
    // `:38832`: the cull rect off `Main.screenPosition` (the view's top-left).
    let screen_pos = camera - screen * 0.5 / zoom;
    let cull = splash_cull_rect(screen_pos, screen);
    for i in 0..tr_core::particles::DUST_SENTINEL {
        if !pool.slot(i).active || !is_splash_type(pool.slot(i).dust_type) {
            continue;
        }
        let d = *pool.slot(i);
        // `:38847` + `:39020-39022`: outside the rect → deactivate, no draw.
        if !dust_in_cull_rect(d.position, cull) {
            pool.slot_mut(i).active = false;
            continue;
        }
        // `:38968`: `Lighting.GetColor((int)((double)pos.X + 4.0) / 16, …)`
        // as XNA bytes (the gore-pass convention, `render/gore.rs`).
        let c = light_at(light, d.position + Vec2::splat(4.0));
        let new_color = tr_core::particles::Color::new(
            ((c[0] * 255.0) as i32).min(255) as u8,
            ((c[1] * 255.0) as i32).min(255) as u8,
            ((c[2] * 255.0) as i32).min(255) as u8,
            255,
        );
        let tint = tr_core::dust_table::get_alpha(&d, new_color);
        // `:38849`/`:39005`: the visual scale and rotation (304-only pins
        // never fire for this family).
        let scale = tr_core::dust_table::get_visual_scale(&d);
        let rot = tr_core::dust_table::get_visual_rotation(&d);
        let Some(uv) = lookups.dust_uv(d.frame_x, d.frame_y, d.frame_w as u32, d.frame_h as u32)
        else {
            continue;
        };
        // The `(4, 4)` origin (`:39005`): XNA rotates about `position` with
        // the origin scaled by scale·zoom — the rotated top-left is
        // `anchor − R(rot)·(origin·scale·zoom)` (the gore `plan_quad`
        // emulation, `render/gore.rs:190-200`).
        let anchor = [
            (d.position.x - camera.x) * zoom + screen.x * 0.5,
            (d.position.y - camera.y) * zoom + screen.y * 0.5,
        ];
        let origin = 4.0 * scale * zoom;
        let (s, co) = rot.sin_cos();
        let pos = [
            anchor[0] - (origin * co - origin * s),
            anchor[1] - (origin * s + origin * co),
        ];
        let size = [d.frame_w as f32 * scale * zoom, d.frame_h as f32 * scale * zoom];
        let tint_f = [
            tint.r as f32 / 255.0,
            tint.g as f32 / 255.0,
            tint.b as f32 / 255.0,
            tint.a as f32 / 255.0,
        ];
        r.push_fx_sprite_rot(pos, size, uv, tint_f, rot);
        // `:39006-39012`: the overlay only when the spawn colour is packed
        // non-zero — rain spawns `default(Color)`, so this arm is dead here
        // but kept verbatim for the family's other future spawners.
        if d.color != tr_core::particles::Color::default() {
            let overlay = tr_core::dust_table::get_color(&d, tint);
            if overlay != tr_core::particles::Color::default() {
                r.push_fx_sprite_rot(
                    pos,
                    size,
                    uv,
                    [
                        overlay.r as f32 / 255.0,
                        overlay.g as f32 / 255.0,
                        overlay.b as f32 / 255.0,
                        overlay.a as f32 / 255.0,
                    ],
                    rot,
                );
            }
        }
        // `:39015-39017`: `newColor == Color.Black` deactivates — XNA's
        // Black is `(0, 0, 0, 255)` (packed 0xFF000000), so only an
        // alpha-0 dust in full dark dies; a faded dust in the dark draws
        // invisibly and survives, exactly like the C#.
        if tint == tr_core::particles::Color::new(0, 0, 0, 255) {
            pool.slot_mut(i).active = false;
        }
    }
}

/// `Main.DrawDust`'s on-screen cull rect (`Main.cs:38832`): the ±1000/±1050
/// slack box off `Main.screenPosition` — `(screenX-1000, screenY-1050,
/// screenWidth+2000, screenHeight+2100)`. `:38831`'s ±500/±50 box is
/// overwritten before the loop; the trail types' `:38845` restore copies the
/// SAME wide rect back, so one rect covers the whole family. World px, no
/// zoom term. Returns `(x, y, w, h)`.
pub(crate) fn splash_cull_rect(screen_pos: Vec2, screen: Vec2) -> (i32, i32, i32, i32) {
    (
        screen_pos.x as i32 - 1000,
        screen_pos.y as i32 - 1050,
        screen.x as i32 + 2000,
        screen.y as i32 + 2100,
    )
}

/// `Main.cs:38847`: `new Rectangle((int)pos.X, (int)pos.Y, 4, 4).Intersects(rect)`
/// — XNA's strict-`<` intersection on both axes (the `Dust.cs:98` spawn-cull
/// convention). A miss means the `:39020-39022` else arm deactivates the dust.
pub(crate) fn dust_in_cull_rect(position: Vec2, rect: (i32, i32, i32, i32)) -> bool {
    let (rx, ry, rw, rh) = rect;
    let sx = position.x as i32;
    let sy = position.y as i32;
    rx < sx + 4 && sx < rx + rw && ry < sy + 4 && sy < ry + rh
}

/// `Lighting.GetColor` for a world-space point, clamped to the computed light rect.
pub(crate) fn light_at(light: &LightMap, p: Vec2) -> [f32; 3] {
    if light.w <= 0 || light.h <= 0 {
        return [1.0; 3];
    }
    let tx = ((p.x / tr_core::TILE_PX) as i32).clamp(light.x0, light.x0 + light.w - 1);
    let ty = ((p.y / tr_core::TILE_PX) as i32).clamp(light.y0, light.y0 + light.h - 1);
    light.get(tx, ty)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    fn ground() -> World {
        let mut w = World::new("rain".into(), 64, 64);
        for x in 0..64 {
            for y in 30..64 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    fn run(rain: &mut Rain, w: &World, n: usize) {
        let camera = Vec2::new(32.0 * 16.0, 25.0 * 16.0);
        for _ in 0..n {
            rain.update(
                w,
                camera,
                Vec2::new(1600.0, 200.0),
                1.0,
                1.0 / 60.0,
                false,
                1,
                0.0,
                1.0,
                0,
                true,
                None,
                DropSimInputs::default(),
            );
        }
    }

    /// The gfx quality the splash gate reads (`Rain.cs:120`): the drop-sim
    /// tests that assert splashes pin it at 1 (`Next(100) < 100` always hits).
    fn full_quality(rain: &mut Rain) {
        rain.gfx_quality = 1.0;
    }

    /// The pool + ctx the W4-g67 splash drain needs — the camera-centred
    /// view the `run` helper uses, full dust budget.
    fn splash_ctx() -> tr_core::particles::SpawnCtx {
        crate::app::spawn_ctx_from(
            false,
            false,
            Vec2::new(32.0 * 16.0, 25.0 * 16.0),
            Vec2::new(1600.0, 200.0),
            1.0,
            6000,
        )
    }

    /// The live splash-dust population (`dustWater()`'s family,
    /// `Dust.cs:402-418`).
    fn splash_count(pool: &tr_core::particles::DustPool) -> usize {
        pool.slots()[..tr_core::particles::DUST_SENTINEL]
            .iter()
            .filter(|d| d.active && is_splash_type(d.dust_type))
            .count()
    }

    #[test]
    fn forced_rain_spawns_drops_that_die_on_the_ground() {
        let w = ground();
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        full_quality(&mut rain);
        let mut pool = tr_core::particles::DustPool::new();
        let ctx = splash_ctx();
        let camera = Vec2::new(32.0 * 16.0, 25.0 * 16.0);
        for _ in 0..120 {
            rain.update(
                &w,
                camera,
                Vec2::new(1600.0, 200.0),
                1.0,
                1.0 / 60.0,
                false,
                1,
                0.0,
                1.0,
                0,
                true,
                Some((&mut pool, &ctx)),
                DropSimInputs::default(),
            );
        }
        assert!(rain.drop_count() > 100, "drops {}", rain.drop_count());
        // The ground is below the camera, so impacts leave splash dust at a
        // steady state (`Rain.cs:122`'s `dustWater()` = 33 on this world).
        assert!(splash_count(&pool) > 0, "impacts should leave splashes");
        assert!(
            pool.slots()[..tr_core::particles::DUST_SENTINEL]
                .iter()
                .any(|d| d.active && d.dust_type == 33),
            "the water-style-0 splash type"
        );
    }

    #[test]
    fn calm_weather_has_no_drops() {
        let w = ground();
        let mut rain = Rain::with_forced(9, None, true);
        run(&mut rain, &w, 240);
        assert_eq!(rain.drop_count(), 0);
        assert_eq!(rain.visual_alpha(), 0.0);
    }

    #[test]
    fn snow_replaces_rain_in_a_snow_biome() {
        let w = ground();
        let mut rain = Rain::with_forced(17, Some(1.0), false);
        let camera = Vec2::new(32.0 * 16.0, 25.0 * 16.0);
        for _ in 0..120 {
            rain.update(
                &w,
                camera,
                Vec2::new(1600.0, 200.0),
                1.0,
                1.0 / 60.0,
                true,
                1,
                0.0,
                1.0,
                0,
                true,
                None,
                DropSimInputs::default(),
            );
        }
        assert_eq!(rain.drop_count(), 0, "no rain drops while snowing");
        assert!(rain.snow_count() > 50, "snow {}", rain.snow_count());
    }

    #[test]
    fn snow_settles_back_into_rain_outside_the_snow_biome() {
        let w = ground();
        let mut rain = Rain::with_forced(19, Some(1.0), false);
        let camera = Vec2::new(32.0 * 16.0, 25.0 * 16.0);
        let screen = Vec2::new(1600.0, 200.0);
        for _ in 0..60 {
            rain.update(
                &w,
                camera,
                screen,
                1.0,
                1.0 / 60.0,
                true,
                1,
                0.0,
                1.0,
                0,
                true,
                None,
                DropSimInputs::default(),
            );
        }
        let snow = rain.snow_count();
        assert!(snow > 0);
        for _ in 0..600 {
            rain.update(
                &w,
                camera,
                screen,
                1.0,
                1.0 / 60.0,
                false,
                1,
                0.0,
                1.0,
                0,
                true,
                None,
                DropSimInputs::default(),
            );
        }
        assert_eq!(rain.snow_count(), 0, "flakes should leave the view");
        assert!(rain.drop_count() > 0);
    }

    #[test]
    fn storms_ramp_the_cloud_deck_and_lightning_decays() {
        let w = ground();
        let mut rain = Rain::with_forced(23, Some(1.0), false);
        let camera = Vec2::new(32.0 * 16.0, 25.0 * 16.0);
        let screen = Vec2::new(1600.0, 200.0);
        for _ in 0..600 {
            rain.update(
                &w,
                camera,
                screen,
                1.0,
                1.0 / 60.0,
                false,
                1,
                0.0,
                1.0,
                0,
                true,
                None,
                DropSimInputs::default(),
            );
        }
        assert!(rain.cloud_bg() > 0.5, "cloud deck {}", rain.cloud_bg());
        rain.flash = 1.0;
        rain.lightning_decay = 0.05; // `NewLightning`'s upper decay rate.
        // The flash rise/decay lives in the lightning chain — `thunder_step`,
        // once per world tick since M1.4-BG-22 — not in the drop sim's tick.
        rain.thunder_step(ThunderInputs::default(), &|_| false);
        assert!(rain.flash < 1.0 && rain.flash > 0.7);
        for _ in 0..30 {
            rain.thunder_step(ThunderInputs::default(), &|_| false);
        }
        assert_eq!(rain.flash, 0.0, "flash must decay to zero");
    }

    #[test]
    fn thunder_step_emits_one_strike_when_the_delay_runs_out() {
        // M6-AUDIO-1: `Main.cs:58751-58766`'s strike — a delay of 1 fires its
        // cue on the next tick and the strike consumes Rain's own stream
        // (the flip, the slot, the pitch — three draws per strike).
        let mut rain = Rain::with_forced(31, None, false);
        rain.thunder_delay = 1;
        rain.thunder_distance = 4;
        let weather = ThunderInputs {
            scene_center: [800.0, 600.0],
            world_surface_px: 4000.0,
            ..Default::default()
        };
        // Delay 1 → the strike fires with distance 4 → ±60 px off the centre,
        // the y untouched above the surface clamp.
        let strike = rain.thunder_step(weather, &|_| false).expect("a strike");
        assert!(
            (strike.x == 740.0) || (strike.x == 860.0),
            "x = {}",
            strike.x
        );
        assert_eq!(strike.y, 600.0);
        assert!(strike.cue.starts_with("Thunder_"), "cue {}", strike.cue);
        assert!(
            (-0.10..=0.10).contains(&strike.pitch),
            "pitch {}",
            strike.pitch
        );
        assert_eq!(rain.thunder_delay, 0);
        assert_eq!(
            rain.thunder_distance, 0,
            "the vanilla drains it with the strike"
        );
        // No delay: no strike, and the stream is untouched.
        assert_eq!(rain.thunder_step(weather, &|_| false), None);
    }

    // M5-SEED-2: the `moreLightning` / `noLightning` micro seeds
    // (`WorldGen.cs:580-586`) gate the storm-lightning roll — the port's
    // `WorldGen.SpawnStormLightning` stand-in (`WorldGen.cs:71864-71884`):
    // `noLightning` alone suppresses it, `moreLightning` alone doubles its
    // rate and lets rain (not only a storm) arm it.
    #[test]
    fn the_lightning_micro_seeds_gate_the_storm_roll() {
        // The chain's arming inputs: `Main.atmo == 1` (the surface band) and
        // a tile centre above the rock layer, with no pending thunder delay.
        // Every case is counted, not sampled: one arm = the flash's rising
        // edge after `new_lightning` (`Main.cs:58708-58733`).
        let storm = ThunderInputs {
            storming: true,
            atmo: 1.0,
            above_rock_layer: true,
            graveyard_intensity: 0.0,
            ..Default::default()
        };
        let arms = |seed: u64, weather: ThunderInputs| {
            let mut w = Rain::with_forced(seed, None, false);
            let mut n = 0u32;
            let mut up = false;
            for _ in 0..20_000 {
                w.thunder_step(weather, &|_| false);
                if w.flash > 0.0 && !up {
                    n += 1;
                }
                up = w.flash > 0.0;
            }
            n
        };
        // The classic storm arms (the base rate), `noLightning` alone never
        // does, and the two-set combination replays the classic exactly.
        let base = arms(41, storm);
        assert!(base > 0, "the classic storm roll armed");
        assert_eq!(
            arms(
                41,
                ThunderInputs {
                    no_lightning: true,
                    ..storm
                }
            ),
            0,
            "noLightning suppresses the storm"
        );
        assert_eq!(
            arms(
                41,
                ThunderInputs {
                    no_lightning: true,
                    more_lightning: true,
                    ..storm
                }
            ),
            base,
            "both-set replays the classic"
        );
        // `moreLightning` alone relaxes the gate to storming OR raining, and
        // halves the window (`:71875-71881`): the rain-only input a classic
        // world ignores now arms, and the storm arms roughly twice as often.
        let rain_only = ThunderInputs {
            storming: false,
            raining: true,
            atmo: 1.0,
            above_rock_layer: true,
            graveyard_intensity: 0.0,
            ..Default::default()
        };
        assert_eq!(arms(41, rain_only), 0, "rain alone never storms");
        let boosted = arms(
            41,
            ThunderInputs {
                more_lightning: true,
                ..rain_only
            },
        );
        assert!(boosted > 0, "moreLightning storms in the rain");
        let boosted_storm = arms(
            41,
            ThunderInputs {
                more_lightning: true,
                ..storm
            },
        );
        assert!(
            boosted_storm > base,
            "the halved window strikes more often: base {base}, storm {boosted_storm}"
        );
        // The graveyard arm is a different chain (`Main.cs:58830-58833`) and
        // carries no seed gate in vanilla, so `noLightning` must leave it
        // armed — the suppression applies to the storm roll only.
        let graveyard = ThunderInputs {
            storming: false,
            raining: true,
            atmo: 1.0,
            above_rock_layer: true,
            graveyard_intensity: 1.0,
            ..Default::default()
        };
        assert!(arms(41, graveyard) > 0, "the graveyard chain arms");
        assert!(
            arms(
                41,
                ThunderInputs {
                    no_lightning: true,
                    ..graveyard
                }
            ) > 0,
            "noLightning leaves the graveyard chain alone"
        );
    }

    #[test]
    fn thunder_step_is_deterministic_per_seed() {
        // The same seed must produce the same strike stream (the
        // per-consumer-stream convention, `rain.rs`'s module doc).
        let play = || {
            let mut rain = Rain::with_forced(37, None, false);
            rain.thunder_delay = 1;
            let weather = ThunderInputs {
                scene_center: [800.0, 600.0],
                world_surface_px: 4000.0,
                ..Default::default()
            };
            (0..3)
                .filter_map(|_| {
                    rain.thunder_delay = 1;
                    rain.thunder_distance = 4;
                    rain.thunder_step(weather, &|_| false)
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(play(), play());
    }

    #[test]
    fn max_raining_is_the_ramp_target_not_the_visible_intensity() {
        // `Main.maxRaining` (`Main.cs:64735`, set by `ChangeRain`) is the target
        // the visible `cloudAlpha` ramps toward (`Main.cs:61913-61926`); the
        // WaterDistortion rain term reads the target (`SceneState.cs:129`).
        let w = ground();
        let mut calm = Rain::with_forced(9, None, true);
        run(&mut calm, &w, 10);
        assert_eq!(calm.max_raining(), 0.0, "no event: maxRaining 0");

        let mut ramping = Rain::with_forced(11, None, false);
        ramping.raining = true;
        ramping.target = 0.8;
        run(&mut ramping, &w, 10);
        assert_eq!(ramping.max_raining(), 0.8);
        assert!(ramping.visual_alpha() < 0.8, "intensity still ramping");

        // A forced storm pins both (`TR_RAIN`), and the event end resets it to 0
        // (`Main.cs:64650`, via `StopRain` from `update_time`'s drain).
        let mut forced = Rain::with_forced(13, Some(1.5), false);
        run(&mut forced, &w, 2);
        assert_eq!(forced.max_raining(), 1.5);
        let mut ending = Rain::with_forced(15, None, false);
        ending.raining = true;
        ending.target = 0.8;
        ending.rain_time = 0;
        let mut tall = ground();
        tall.world_surface = 200.0; // `isThereAWorldSurface` (`Main.cs:2788`)
        ending.update_time(1, &tall, true, false, 0.0, true);
        assert_eq!(ending.max_raining(), 0.0);
        assert!(!ending.is_raining());
    }

    #[test]
    fn intensity_ramps_toward_the_target() {
        let w = ground();
        let mut rain = Rain::with_forced(11, None, false);
        rain.raining = true;
        rain.target = 1.0;
        run(&mut rain, &w, 600);
        let a = rain.visual_alpha();
        assert!(a > 0.1 && a < 0.5, "ramp {a}");
    }

    #[test]
    fn drops_only_spawn_into_open_sky() {
        let mut w = ground();
        // A wall behind the air blocks rain spawns even with open sky above.
        for x in 0..64 {
            for y in 5..30 {
                let i = w.index(x, y);
                w.tiles[i].wall = 2;
            }
        }
        let mut rain = Rain::with_forced(5, Some(1.0), false);
        run(&mut rain, &w, 120);
        assert_eq!(rain.drop_count(), 0);
    }

    #[test]
    fn test_DW_1_1_thunder_arm_runs_once_per_tick_not_per_day_rate_iteration() {
        // `Main.UpdateWeather`'s lightning chain sits in the
        // `currentDayRateIteration == 0` arm (`Main.cs:58739`), so a dial
        // fast-forward (dayRate 60) must advance it once per world tick —
        // here 3 → 2, not 3 → 0 with a re-arm.
        use crate::app::flow::front_flow_tests::flow_app;
        let mut app = flow_app(None, vec![], vec![]);
        app.world.fast_forward_to_dawn = true; // `day_rate()` = 60
        app.rain.thunder_delay = 3;
        crate::world_sim::weather::update_world_tick(&mut app, 1.0 / 60.0);
        assert_eq!(app.rain.thunder_delay, 2, "one decrement per world tick");
        // The flash decay rides the same once-per-tick cadence.
        app.rain.lightning_decay = 0.05;
        app.rain.flash = 1.0;
        crate::world_sim::weather::update_world_tick(&mut app, 1.0 / 60.0);
        assert!(app.rain.flash < 1.0, "the decay ran exactly once per tick");
    }

    /// The exact `Rain` stream replay for pinned-seed tests: `with_forced`'s
    /// seed mix plus the splitmix of [`Rain::next_u64`], drawn with the C#
    /// `Random.Next(min, max)` semantics `Rain::range` mirrors.
    struct StreamReplay(u64);

    impl StreamReplay {
        fn new(seed: u64) -> Self {
            Self((seed ^ 0x2545_F491_4F6C_DD1D).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
        }
        fn next_u64(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        }
        fn next(&mut self, lo: u64, hi: u64) -> u64 {
            lo + self.next_u64() % (hi - lo).max(1)
        }
    }

    /// A `ground()` world above the `isThereAWorldSurface` gate
    /// (`Main.worldSurface > 50.0`, `Main.cs:2788`) so `update_time` runs.
    fn rain_world() -> World {
        let mut w = ground();
        w.world_surface = 200.0;
        w
    }

    /// Mirrors `Main.StartRain`'s full draw sequence (`Main.cs:64659-64738`)
    /// on the replay stream: the coin arm (`RollLuck(25)` at luck 0 — one
    /// draw — plus the `Next(75,151)` roll on a hit), the `rainTime` ladder
    /// (`:64676-64701`), the strength ladder (`:64703-64719`), then
    /// `ChangeRain`'s lowest band (`:64730` — the 100-cloud stand-in keeps
    /// `numClouds > 150`/`> 100` false and `cloud_bg` is 0). Returns the
    /// expected `(rainTime, maxRaining, final stream state)`.
    /// The coin-rain gate's head (`Main.cs:64666-64673`) as the C# branch
    /// shape: luck 0 takes the plain `Next(25)` (one draw), luck 1.0 always
    /// takes the hit branch (float gate + inner `Next(12, 25)` + outer
    /// `Next(inner, 25)`); the coin value is consumed-but-not-spawned on a
    /// hit either way.
    fn replay_start_rain_head(r: &mut StreamReplay, player_luck: f32) {
        if player_luck > 0.0 {
            let _gate = r.next_u64(); // `NextFloat() < luck` — always true at 1.0
            let inner = r.next(12, 25); // `Next(range / 2, range)`
            let outer = r.next(inner, 25); // `Next(inner, range)`
            if outer == 0 {
                let _coin = r.next(75, 151); // `Main.cs:64672` — consumed
            }
        } else if r.next(0, 25) == 0 {
            let _coin = r.next(75, 151); // `Main.cs:64672` — consumed
        }
    }

    /// The post-gate rainTime ladder + ChangeRain band (`Main.cs:64678+`),
    /// shared by the luck-0 and luck-1 replays.
    fn replay_start_rain_tail(r: &mut StreamReplay) -> (i32, f32, u64) {
        let num3 = 86400 / 24;
        let mut num4 = r.next((num3 * 8) as u64, 86400) as i64; // `:64678`
        if r.next(0, 3) == 0 {
            num4 += r.next(0, num3 as u64) as i64;
        }
        if r.next(0, 4) == 0 {
            num4 += r.next(0, (num3 * 2) as u64) as i64;
        }
        if r.next(0, 5) == 0 {
            num4 += r.next(0, (num3 * 2) as u64) as i64;
        }
        if r.next(0, 6) == 0 {
            num4 += r.next(0, (num3 * 3) as u64) as i64;
        }
        if r.next(0, 7) == 0 {
            num4 += r.next(0, (num3 * 4) as u64) as i64;
        }
        if r.next(0, 8) == 0 {
            num4 += r.next(0, (num3 * 5) as u64) as i64;
        }
        let mut num5 = 1.0f32;
        if r.next(0, 2) == 0 {
            num5 += 0.05;
        }
        if r.next(0, 3) == 0 {
            num5 += 0.1;
        }
        if r.next(0, 4) == 0 {
            num5 += 0.15;
        }
        if r.next(0, 5) == 0 {
            num5 += 0.2;
        }
        // `Main.cs:64730`, band 3.
        let val = if r.next(0, 3) != 0 {
            r.next(5, 31) as f32 * 0.01
        } else {
            r.next(5, 41) as f32 * 0.01
        };
        ((num4 as f32 * num5) as i32, val, r.0)
    }

    fn replay_start_rain(seed: u64) -> (i32, f32, u64) {
        let mut r = StreamReplay::new(seed);
        replay_start_rain_head(&mut r, 0.0);
        replay_start_rain_tail(&mut r)
    }

    #[test]
    fn test_DW_2_1_start_rain_ladder_matches_the_csharp_replay() {
        // The ladder of `Main.StartRain` (`Main.cs:64659-64727`) replayed
        // straight from the C# — values AND exact stream position must match
        // at pinned seeds covering both coin-arm outcomes and fired/skipped
        // bonus gates.
        let w = rain_world();
        for seed in [42u64, 1337, 90_210, 271_828] {
            let mut rain = Rain::with_forced(seed, None, false);
            rain.start_rain(&w, false, 0.0);
            assert!(rain.is_raining(), "seed {seed}");
            assert!(rain.rain_time > 0, "seed {seed}");
            let (want_time, want_target, want_state) = replay_start_rain(seed);
            assert_eq!(rain.rain_time, want_time, "seed {seed}: rainTime ladder");
            assert_eq!(rain.target, want_target, "seed {seed}: ChangeRain band");
            assert_eq!(rain.rng, want_state, "seed {seed}: exact stream position");
            assert!(
                (0.05..0.41).contains(&rain.target),
                "lowest band, seed {seed}"
            );
        }
    }

    #[test]
    fn test_DW_2_1_change_rain_band_gates_pick_by_cloud_bg_stand_in() {
        // `Main.cs:64730`: the bands gate on `cloudBGActive >= 1f ||
        // numClouds > 150`, then `numClouds > 100`. The stand-in pins numClouds
        // at 100 (`sky.rs` `make_clouds`), so only the `cloud_bg >= 1.0` storm
        // gate reaches the heavy band; the middle band is dead by construction.
        for seed in [5u64, 77, 1234] {
            let mut calm = Rain::with_forced(seed, None, false);
            calm.change_rain();
            assert!(
                (0.05..0.41).contains(&calm.target),
                "calm band, seed {seed}: {}",
                calm.target
            );
            let mut r = StreamReplay::new(seed);
            let want_calm = if r.next(0, 3) != 0 {
                r.next(5, 31) as f32 * 0.01
            } else {
                r.next(5, 41) as f32 * 0.01
            };
            assert_eq!(calm.target, want_calm, "seed {seed}");

            let mut storm = Rain::with_forced(seed, None, false);
            storm.cloud_bg = 1.0; // the `cloudBGActive >= 1f` stand-in
            storm.change_rain();
            assert!(
                (0.40..0.91).contains(&storm.target),
                "storm band, seed {seed}: {}",
                storm.target
            );
            let mut r = StreamReplay::new(seed);
            let want_storm = if r.next(0, 3) != 0 {
                r.next(40, 91) as f32 * 0.01
            } else {
                r.next(20, 91) as f32 * 0.01
            };
            assert_eq!(storm.target, want_storm, "seed {seed}");
        }
    }

    #[test]
    fn test_DW_2_1_stop_rain_clears_the_event_state() {
        let mut rain = Rain::with_forced(7, None, false);
        rain.raining = true;
        rain.target = 0.8;
        rain.rain_time = 5_000;
        rain.stop_rain();
        assert!(!rain.is_raining());
        assert_eq!(rain.max_raining(), 0.0);
        assert_eq!(rain.rain_time, 0);
        // `Main.cs:64652-64655`: without `instant` the visible `cloudAlpha`
        // decays through the ramp — `StopRain` leaves the field alone.
        rain.intensity = 0.5;
        rain.stop_rain();
        assert_eq!(rain.intensity, 0.5, "the ramp owns the decay, not StopRain");
    }

    #[test]
    fn test_DW_2_2_start_denominator_8280_at_day_rate_60_vs_496800_at_1() {
        // `Main.cs:64892` + `:64912`: `num2 = 86400 / max(1, dayRate)`; the
        // first start arm rolls `Next((int)(num2 * 5.75))` — 1440·5.75 = 8280
        // at dayRate 60, 86400·5.75 = 496800 at dayRate 1. A first draw
        // ≡ 0 mod 8280 but not mod 496800 starts the rain only at dayRate 60.
        let w = rain_world();
        let fire_at_60 = (1..2_000_000u64)
            .find(|&s| {
                let d = StreamReplay::new(s).next_u64();
                d % 8280 == 0 && d % 496_800 != 0
            })
            .expect("a 8280-only seed exists");
        let mut fast = Rain::with_forced(fire_at_60, None, false);
        fast.update_time(60, &w, true, false, 0.0, true);
        assert!(fast.is_raining(), "the ×5.75 roll hit at dayRate 60");

        let mut slow = Rain::with_forced(fire_at_60, None, false);
        slow.update_time(1, &w, true, false, 0.0, true);
        assert!(
            !slow.is_raining(),
            "dayRate 1 rolls the plain 86400·5.75 denominator"
        );

        // A draw ≡ 0 mod 496800 hits BOTH denominators (496800 = 8280 × 60) —
        // dayRate 1 is the plain 86400 shape.
        let fire_at_both = (1..8_000_000u64)
            .find(|&s| StreamReplay::new(s).next_u64() % 496_800 == 0)
            .expect("a 496800 seed exists");
        let mut both_fast = Rain::with_forced(fire_at_both, None, false);
        both_fast.update_time(60, &w, true, false, 0.0, true);
        assert!(both_fast.is_raining());
        let mut both_slow = Rain::with_forced(fire_at_both, None, false);
        both_slow.update_time(1, &w, true, false, 0.0, true);
        assert!(
            both_slow.is_raining(),
            "the plain 86400 shape fired at dayRate 1"
        );
    }

    #[test]
    fn test_DW_2_2_rain_time_zero_boundary_stops_rain_without_rolling() {
        // `Main.cs:64874-64886`: the drain hits `rainTime <= 0` (exactly 0 is
        // the boundary) → `StopRain`, and the `else if` roll is never
        // evaluated — the stream stays put.
        let w = rain_world();
        let mut rain = Rain::with_forced(7, None, false);
        rain.raining = true;
        rain.target = 0.8;
        rain.rain_time = 1;
        let state = rain.rng;
        rain.update_time(1, &w, true, false, 0.0, true);
        assert!(!rain.is_raining());
        assert_eq!(rain.max_raining(), 0.0);
        assert_eq!(rain.rain_time, 0, "StopRain pins rainTime to 0 (`:64648`)");
        assert_eq!(rain.rng, state, "no roll is consumed on the stop arm");

        // One tick above the boundary survives and rolls (`rainTime` 2 → 1).
        let mut alive = Rain::with_forced(7, None, false);
        alive.raining = true;
        alive.target = 0.8;
        alive.rain_time = 2;
        alive.update_time(1, &w, true, false, 0.0, true);
        assert!(alive.is_raining(), "rainTime 1 is still above the boundary");
        assert_eq!(alive.rain_time, 1);
    }

    #[test]
    fn test_update_time_night_start_roll_fires_without_a_day_time_gate() {
        // `Main.cs:64912-64923`: the start arms carry no `Main.dayTime` gate —
        // only the slime-rain roll at `:64925` is day-gated, and that block is
        // M4. A hit on the ×5.75 bound starts rain regardless of time of day.
        let w = rain_world();
        let fire = (1..8_000_000u64)
            .find(|&s| StreamReplay::new(s).next_u64() % 496_800 == 0)
            .expect("a 496800 seed exists");
        let mut rain = Rain::with_forced(fire, None, false);
        rain.update_time(1, &w, true, false, 0.0, true);
        assert!(rain.is_raining());
        assert!(rain.rain_time > 0);
    }

    #[test]
    fn test_lady_bug_boost_drains_by_day_rate_and_arms_the_third_start_roll() {
        // `Main.cs:64833-64836`: the boost drains by dayRate per tick (the
        // setter side — M4 ladybug deaths — is inert); `:64920-64922`: the
        // third start arm only rolls while the boost is up, after the first
        // arm missed and with the storm gate down (`cloud_bg` 0 skips the
        // second arm's roll entirely).
        let w = rain_world();
        let mut drain = Rain::with_forced(7, None, false);
        drain.lady_bug_rain_boost = 120;
        drain.update_time(60, &w, false, false, 0.0, true); // no 120-HP player: arms gated
        assert_eq!(drain.lady_bug_rain_boost, 60);
        drain.update_time(60, &w, false, false, 0.0, true);
        assert_eq!(drain.lady_bug_rain_boost, 0, "the `> 0` gate clamps at 0");

        let fire = (1..200_000u64)
            .find(|&s| {
                // The first draw is arm 1's miss; the third arm's roll is the
                // SECOND stream draw.
                let mut r = StreamReplay::new(s);
                let d1 = r.next_u64();
                let d2 = r.next_u64();
                d1 % 8280 != 0 && d2 % 1440 == 0
            })
            .expect("a ladybug-arm seed exists");
        let mut rain = Rain::with_forced(fire, None, false);
        rain.lady_bug_rain_boost = 100;
        rain.update_time(60, &w, true, false, 0.0, true);
        assert!(rain.is_raining(), "the ladybug arm fired");

        let mut quiet = Rain::with_forced(fire, None, false);
        quiet.update_time(60, &w, true, false, 0.0, true);
        assert!(
            !quiet.is_raining(),
            "boost 0 gates the third arm before its roll"
        );
    }

    #[test]
    fn test_DW_2_3_cloud_alpha_ramp_moves_60x_at_day_rate_60() {
        // `Main.cs:61902-61924`: `num5 = 0.0005 * max(1, dayRate)` per step
        // toward `maxRaining` — one visual tick at dayRate 60 moves 60× the
        // dry constant. The storm deck ramp scales the same way (the
        // `Main.cs:13187/13205` shape at the port's 0.005 stand-in rate).
        let w = ground();
        let step = 1.0 / 60.0;
        let camera = Vec2::new(32.0 * 16.0, 25.0 * 16.0);
        let screen = Vec2::new(1600.0, 200.0);

        let mut fast = Rain::with_forced(11, None, false);
        fast.raining = true;
        fast.target = 1.0;
        fast.update(&w, camera, screen, 1.0, step, false, 60, 0.0, 1.0, 0, true, None, DropSimInputs::default());
        assert!(
            (fast.visual_alpha() - 0.03).abs() < 1e-6,
            "60× ramp: {}",
            fast.visual_alpha()
        );

        let mut slow = Rain::with_forced(11, None, false);
        slow.raining = true;
        slow.target = 1.0;
        slow.update(&w, camera, screen, 1.0, step, false, 1, 0.0, 1.0, 0, true, None, DropSimInputs::default());
        assert!(
            (slow.visual_alpha() - 0.0005).abs() < 1e-9,
            "dry ramp: {}",
            slow.visual_alpha()
        );

        let mut deck = Rain::with_forced(11, None, false);
        deck.intensity = 1.0; // above the 0.05 gate: the deck chases the intensity
        deck.update(&w, camera, screen, 1.0, step, false, 60, 0.0, 1.0, 0, true, None, DropSimInputs::default());
        assert!(
            (deck.cloud_bg() - 0.3).abs() < 1e-6,
            "deck at FF: {}",
            deck.cloud_bg()
        );
        let mut dry = Rain::with_forced(11, None, false);
        dry.intensity = 1.0;
        dry.update(&w, camera, screen, 1.0, step, false, 1, 0.0, 1.0, 0, true, None, DropSimInputs::default());
        assert!(
            (dry.cloud_bg() - 0.005).abs() < 1e-9,
            "deck dry: {}",
            dry.cloud_bg()
        );
    }

    #[test]
    fn test_DW_2_4_forced_pin_precedence_over_update_time() {
        // The TR_RAIN pin is the scheduler: `update_time` never runs under it
        // (the retired session schedule skipped the same way) — a dead
        // `rainTime` cannot stop a pinned storm and no roll consumes the
        // stream. TR_NO_WEATHER (`calm`) disables the rolls the same way.
        let w = rain_world();
        let mut rain = Rain::with_forced(7, Some(0.8), false);
        rain.rain_time = 0;
        rain.lady_bug_rain_boost = 100;
        let state = rain.rng;
        rain.update_time(60, &w, true, false, 0.0, true);
        assert_eq!(rain.rain_time, 0, "no drain under the pin");
        assert_eq!(rain.lady_bug_rain_boost, 100, "no drain under the pin");
        assert_eq!(rain.rng, state, "no rolls under the pin");
        // The pin still owns the visual state through the drop-sim tick.
        rain.update(
            &w,
            Vec2::new(512.0, 400.0),
            Vec2::new(1600.0, 200.0),
            1.0,
            1.0 / 60.0,
            false,
            60,
            0.0,
            1.0,
            0,
            true,
            None,
            DropSimInputs::default(),
        );
        assert_eq!(rain.max_raining(), 0.8);
        assert!(rain.is_raining());

        let mut quiet = Rain::with_forced(9, None, true);
        quiet.lady_bug_rain_boost = 100;
        let state = quiet.rng;
        quiet.update_time(60, &w, true, false, 0.0, true);
        assert_eq!(
            quiet.lady_bug_rain_boost, 100,
            "no drain under TR_NO_WEATHER"
        );
        assert_eq!(quiet.rng, state);
    }

    #[test]
    fn test_DW_2_4_coin_rain_roll_is_consumed_without_a_spawn() {
        // `Main.cs:64668-64673`: on a `RollLuck(25)` hit the coin value
        // (`Next(75,151) * 100 * 100 * maxTilesX/4200f`) is CONSUMED but never
        // spawned — coin projectiles are unported (M2/M4), the port stores no
        // `coinRain`. The stream position proves both draws happened; the
        // event still starts with the full ladder behind them.
        let hit = (1..10_000u64)
            .find(|&s| StreamReplay::new(s).next_u64() % 25 == 0)
            .expect("a RollLuck hit seed exists");
        let w = rain_world();
        let mut rain = Rain::with_forced(hit, None, false);
        rain.start_rain(&w, false, 0.0);
        assert!(rain.is_raining());
        let (want_time, want_target, want_state) = replay_start_rain(hit);
        assert_eq!(rain.rng, want_state, "the coin draw was consumed");
        assert_eq!(rain.rain_time, want_time);
        assert_eq!(rain.target, want_target);
        // Sanity: the pinned seed really took the coin arm.
        let mut r = StreamReplay::new(hit);
        assert_eq!(r.next(0, 25), 0, "the seed hits RollLuck(25)");
        let coin = r.next(75, 151);
        let num = 64.0f32 / 4200.0; // `ground()` is 64 tiles wide
        let _would_be_coin_rain = ((coin as i32 * 100 * 100) as f32 * num) as i32;
    }

    #[test]
    fn test_DW_3_4_coin_rain_luck_selects_the_branch_without_reordering() {
        // DW-3.4 — `Main.cs:64668` rides the real player luck: at luck 1.0
        // the hit branch always fires and consumes its 3 draws (float gate +
        // inner Next(12, 25) + outer Next(inner, 25)) where luck 0 consumes
        // the single plain Next(25) — the same stream, branch-selected, no
        // reordering; the ladder tail behind the gate is unchanged.
        let w = rain_world();
        for seed in [42u64, 1337, 90_210, 271_828] {
            let mut rain = Rain::with_forced(seed, None, false);
            rain.start_rain(&w, false, 1.0);
            assert!(rain.is_raining(), "seed {seed}");
            let mut r = StreamReplay::new(seed);
            replay_start_rain_head(&mut r, 1.0);
            let (want_time, want_target, want_state) = replay_start_rain_tail(&mut r);
            assert_eq!(rain.rng, want_state, "seed {seed}: exact stream position");
            assert_eq!(rain.rain_time, want_time, "seed {seed}: rainTime ladder");
            assert_eq!(rain.target, want_target, "seed {seed}: ChangeRain band");
            // And the luck-0 run from the same seed sits elsewhere.
            let mut plain = Rain::with_forced(seed, None, false);
            plain.start_rain(&w, false, 0.0);
            assert_ne!(plain.rng, rain.rng, "seed {seed}: the branch differs");
        }
    }

    #[test]
    fn test_DW_2_2_update_time_runs_inside_the_world_tick() {
        // The phase-2 slot: `update_time` runs inside `update_world_tick`
        // after the wind loop and before the sandstorm arm — a dead storm
        // stops in one tick and the sandstorm still decays behind it.
        use crate::app::flow::front_flow_tests::flow_app;
        let mut app = flow_app(None, vec![], vec![]);
        app.rain = Rain::with_forced(7, None, false);
        app.rain.raining = true;
        app.rain.target = 0.8;
        app.rain.rain_time = 0;
        app.world.world_surface = 200.0; // `isThereAWorldSurface`
        app.sandstorm.happening = true;
        app.sandstorm.time_left = 5_000;
        crate::world_sim::weather::update_world_tick(&mut app, 1.0 / 60.0);
        assert!(!app.rain.is_raining(), "StopRain fired inside the tick");
        assert_eq!(app.rain.max_raining(), 0.0);
        assert!(
            app.sandstorm.time_left < 5_000,
            "the sandstorm arm ran after the rain section"
        );
    }

    // ---- W4-g67: the rain collision splash → DustPool lane ----------------
    // Witnesses are derived by hand from the C# (`Rain.cs:96-133`,
    // `Dust.cs:75-256`'s default block, `Dust.cs:1834-1874`) over the port's
    // splitmix seam (`app/frame.rs`'s `SplitmixRand`): one `next_rand` step
    // per draw, the 53-bit sample, the C# `Next` conversions.

    /// One drop pinned into the field at the ground row (y=30 → 480 px) so
    /// the NEXT `update_drops` step moves it into the solid and kills it.
    fn one_falling_drop(rain: &mut Rain, pos: Vec2, vel: Vec2) {
        rain.drops.clear();
        rain.drops.push(Drop {
            pos,
            vel,
            scale: 1.0,
            rot: 0.0,
            kind: 0,
        });
    }

    /// A TALL ground world: `Collision.SolidCollision`/`WetCollision` clamp
    /// their scan at `maxTilesY - 40` (`Collision.cs:2708`/`:1594`), so the
    /// 64-tall `ground()`'s row-30 floor is invisible to the real probes —
    /// the collision witnesses need rows under `height - 40` (128 − 40 = 88).
    fn tall_ground() -> World {
        let mut w = World::new("rain-tall".into(), 64, 128);
        for x in 0..64 {
            for y in 30..128 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    /// A ctx whose view contains the impact point (`ground()`'s camera).
    fn impact_ctx() -> tr_core::particles::SpawnCtx {
        splash_ctx()
    }

    /// W4-g67 witness — `Rain.cs:117-131`'s solid-kill splash: the gate roll,
    /// the `NewDust(position - velocity, 2, 2, 33)` defaults and the eight
    /// recipe writes. Hand-derived on seed 7's stream: the gate is
    /// `next_rand % 100`; the default block's six draws are
    /// `Next(1)`×2 (the min-5 box), `Next(-20,21)`×2 (velocity), `Next(3)`
    /// (frame row), `Next(-20,21)` (scale). Every expected value below is
    /// pinned through the SAME seam the spawn uses, so the test replays the
    /// stream draw-for-draw rather than trusting a snapshot.
    #[test]
    fn w4_g67_solid_impact_spawns_the_recipe_dust_on_rains_stream() {
        let w = tall_ground();
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        full_quality(&mut rain);
        // Impact: pos (500, 470) + vel (0, 14) → (500, 484) — inside the
        // row-30 solid (480..496). `position - velocity` = (500, 470).
        let pos = Vec2::new(500.0, 470.0);
        let vel = Vec2::new(0.0, 14.0);
        one_falling_drop(&mut rain, pos, vel);
        let mut pool = tr_core::particles::DustPool::new();
        let ctx = impact_ctx();
        let camera = Vec2::new(32.0 * 16.0, 25.0 * 16.0);
        // Replay the expected stream on a twin of Rain's seeded state.
        let mut twin = rain.rng;
        let gate = (crate::app::next_rand(&mut twin) % 100) as f32;
        assert!(gate < 100.0, "full quality: the gate always passes");
        // The default block, `Dust.cs:153-193` through SplitmixRand.
        let mut sr = crate::app::SplitmixRand(&mut twin);
        use tr_core::particles::DustRng;
        let jx = sr.next_max(1); // width 2 → min-5 box → Next(1) = 0
        let jy = sr.next_max(1);
        let vx = sr.next_range(-20, 21) as f32 * 0.1;
        let vy = sr.next_range(-20, 21) as f32 * 0.1;
        let row = sr.next_max(3);
        let sc = (1.0 + sr.next_range(-20, 21) as f32 * 0.01) * 1.0;
        drop(sr);
        // Type 33's spawn tweak (`Dust.cs:230-235`): alpha 170, vel ×0.5,
        // vel.Y += 1 — BEFORE the recipe writes.
        let tweak_v = Vec2::new(vx, vy) * 0.5 + Vec2::new(0.0, 1.0);
        // The eight recipe writes (`Rain.cs:123-130`).
        let want_pos = Vec2::new(500.0 + jx as f32 + 4.0 - 2.0, 470.0 + jy as f32 + 4.0 + 2.0);
        let want_vel = tweak_v * 0.1 + -vel * 0.025 - Vec2::new(0.0, 2.0);
        rain.update_drops(&w, camera, Vec2::new(1600.0, 200.0), 1.0, Some((&mut pool, &ctx)), DropSimInputs::default());
        assert_eq!(rain.drops.len(), 0, "the solid kill fired");
        assert_eq!(rain.rng, twin, "gate + 6 default-block draws, in order");
        let live: Vec<_> = pool.slots()[..tr_core::particles::DUST_SENTINEL]
            .iter()
            .filter(|d| d.active)
            .collect();
        assert_eq!(live.len(), 1, "one splash dust");
        let d = live[0];
        assert_eq!(d.dust_type, 33, "dustWater(): the small-world beach style 13 maps to the default 33");
        assert_eq!(d.position, want_pos);
        assert_eq!(d.velocity, want_vel);
        assert_eq!(d.alpha, 38, "`:125` overwrites the tweak's 170");
        assert_eq!(d.scale, 0.6, "`:129` overwrites the rolled scale (twin: {sc})");
        assert!(d.no_gravity, "`:130`");
        assert_eq!(d.frame_x, 330);
        assert_eq!(d.frame_y, 10 * row);
        assert_eq!((d.frame_w, d.frame_h), (8, 8));
        assert_eq!(d.color, tr_core::particles::Color::default());
    }

    /// W4-g67 witness — the gfx gate (`Rain.cs:120`): quality 0 rolls
    /// `Next(100)` and NEVER spawns (the roll still consumes the stream);
    /// the refused sentinel takes the eight writes without a live slot
    /// when the ctx is paused (`Dust.cs:85`).
    #[test]
    fn w4_g67_the_gfx_gate_rolls_even_when_it_refuses() {
        let w = tall_ground();
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        rain.gfx_quality = 0.0;
        one_falling_drop(&mut rain, Vec2::new(500.0, 470.0), Vec2::new(0.0, 14.0));
        let mut pool = tr_core::particles::DustPool::new();
        let ctx = impact_ctx();
        let camera = Vec2::new(32.0 * 16.0, 25.0 * 16.0);
        let mut twin = rain.rng;
        let _gate = crate::app::next_rand(&mut twin) % 100;
        rain.update_drops(&w, camera, Vec2::new(1600.0, 200.0), 1.0, Some((&mut pool, &ctx)), DropSimInputs::default());
        assert_eq!(rain.rng, twin, "exactly ONE draw: the gate");
        assert_eq!(splash_count(&pool), 0, "quality 0: no splash");
        // The paused ctx refuses INSIDE NewDust (`Dust.cs:85`) — the gate
        // passed, the sentinel took the writes, nothing lives.
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        full_quality(&mut rain);
        one_falling_drop(&mut rain, Vec2::new(500.0, 470.0), Vec2::new(0.0, 14.0));
        let paused = tr_core::particles::SpawnCtx {
            game_paused: true,
            ..impact_ctx()
        };
        let mut twin = rain.rng;
        let _gate = crate::app::next_rand(&mut twin) % 100;
        rain.update_drops(&w, camera, Vec2::new(1600.0, 200.0), 1.0, Some((&mut pool, &paused)), DropSimInputs::default());
        assert_eq!(rain.rng, twin, "the guard precedes every default draw");
        assert_eq!(splash_count(&pool), 0, "the paused guard refused");
        assert!(!pool.slot(tr_core::particles::DUST_SENTINEL).active);
        assert_eq!(pool.slot(tr_core::particles::DUST_SENTINEL).alpha, 38,
            "the sentinel took the recipe writes (Main.dust[6000])");
    }

    /// W4-g67 witness — the below-screen kill ALSO spawns (`Rain.cs:117`'s
    /// combined branch): a drop past `screenPosition.Y + screenHeight + 100`
    /// rolls the gate and calls NewDust; the off-screen cull
    /// (`Dust.cs:97-101`, margin 400 at dCount 0) is what drops it.
    #[test]
    fn w4_g67_below_screen_kill_rolls_the_gate_and_culls_far_dust() {
        let w = ground();
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        full_quality(&mut rain);
        let camera = Vec2::new(32.0 * 16.0, 25.0 * 16.0);
        let screen = Vec2::new(1600.0, 200.0);
        // bottom = 400 + 100 + 100 = 600. The drop lands at y 605 — below,
        // dry (the ground rows are solid but the y-clamp `height-40`=24
        // keeps SolidCollision off them) and outside the dust cull box
        // (view y 300..500 + margin 400 → -100..900; 605-14=591 is INSIDE,
        // so this one SURVIVES the cull — the C# spawns it too).
        one_falling_drop(&mut rain, Vec2::new(500.0, 591.0), Vec2::new(0.0, 14.0));
        let mut pool = tr_core::particles::DustPool::new();
        let ctx = impact_ctx();
        rain.update_drops(&w, camera, screen, 1.0, Some((&mut pool, &ctx)), DropSimInputs::default());
        assert_eq!(rain.drops.len(), 0, "below-screen kill");
        assert_eq!(splash_count(&pool), 1, "the combined branch spawns it");
        // Far below (y 1205 → dust at 1191 > 900): the cull refuses.
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        full_quality(&mut rain);
        one_falling_drop(&mut rain, Vec2::new(500.0, 1205.0), Vec2::new(0.0, 14.0));
        let mut pool = tr_core::particles::DustPool::new();
        let mut twin = rain.rng;
        let _gate = crate::app::next_rand(&mut twin) % 100;
        rain.update_drops(&w, camera, screen, 1.0, Some((&mut pool, &ctx)), DropSimInputs::default());
        assert_eq!(rain.rng, twin, "the cull precedes the default draws");
        assert_eq!(splash_count(&pool), 0, "off-screen dust culled");
    }

    /// W4-g67 witness — the wet-kill arm (`Rain.cs:117`'s WetCollision):
    /// a drop entering a liquid tile dies and splashes; the probe is the
    /// 2×2 inset rect (`Collision.cs:1576-1587`), the surface at
    /// `tileY + (256-liquid)/32*2`.
    #[test]
    fn w4_g67_wet_surface_impact_splashes() {
        let mut w = World::new("wet".into(), 64, 128);
        // A full water tile at (31, 29): surface y = 464, box 464..480.
        let i = w.index(31, 29);
        w.tiles[i].liquid = 255;
        w.compute_column_surface();
        w.compute_chunks();
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        full_quality(&mut rain);
        // The drop's next step lands at (500, 466): the 2×2 probe's inset
        // rect (500..502, 466.5..467.5) overlaps the water surface.
        one_falling_drop(&mut rain, Vec2::new(500.0, 452.0), Vec2::new(0.0, 14.0));
        let mut pool = tr_core::particles::DustPool::new();
        let ctx = impact_ctx();
        let camera = Vec2::new(32.0 * 16.0, 25.0 * 16.0);
        rain.update_drops(&w, camera, Vec2::new(1600.0, 200.0), 1.0, Some((&mut pool, &ctx)), DropSimInputs::default());
        assert_eq!(rain.drops.len(), 0, "the wet kill fired");
        assert_eq!(splash_count(&pool), 1, "wet impacts splash too");
    }

    /// W4-g67 witness — `water_splash_arm` (`Dust.cs:1834-1874`), the
    /// no-gravity spray on a dry, still-X splash: the solid probe zeroes
    /// nothing (open air), the spin +0.5 then the −0.01 nibble, the fade
    /// pair (+2 alpha, −0.005 scale), the sign-picked +0.2 spin, the +0.03
    /// grow, the ×1.05 X stretch and the +0.15 sink. No RNG draws.
    #[test]
    fn w4_g67_the_splash_arm_runs_the_1834_chain() {
        let w = World::new("air".into(), 64, 64);
        let mut d = tr_core::particles::Dust {
            dust_type: 33,
            position: Vec2::new(500.0, 300.0),
            velocity: Vec2::new(0.0, -1.0),
            alpha: 38,
            scale: 0.6,
            rotation: 0.0,
            no_gravity: true,
            active: true,
            ..Default::default()
        };
        let matched = water_splash_arm(&mut d, &w);
        assert!(!matched, "FIRST-ladder arm: the gravity tail still runs");
        // Still X: rotation +0.5, scale −0.01; fade +2/−0.005; noGravity:
        // X ≥ 0 → +0.2 spin, +0.03 scale, ×1.05 X, +0.15 Y.
        assert_eq!(d.alpha, 40);
        assert!((d.scale - (0.6 - 0.01 - 0.005 + 0.03)).abs() < 1e-6);
        assert!((d.rotation - 0.7).abs() < 1e-6);
        assert_eq!(d.velocity.x, 0.0);
        assert!((d.velocity.y - (-1.0 + 0.15)).abs() < 1e-6);
        // Moving X < 0: the −0.2 spin arm, no still-X block.
        let mut d = tr_core::particles::Dust {
            dust_type: 33,
            velocity: Vec2::new(-2.0, 5.0),
            alpha: 38,
            scale: 0.6,
            no_gravity: true,
            active: true,
            ..Default::default()
        };
        water_splash_arm(&mut d, &w);
        assert!((d.rotation - -0.2).abs() < 1e-6, "the negative-X spin");
        assert!((d.velocity.x - -2.1).abs() < 1e-6, "×1.05");
        assert!((d.velocity.y - 4.15).abs() < 1e-6, "the >4 clamp then the +0.15 sink");
        // Wet contact: +20 alpha and −0.1 scale on top of the fade pair.
        let mut ww = World::new("wet".into(), 64, 64);
        let i = ww.index(31, 18);
        ww.tiles[i].liquid = 255; // surface y 288..304 — the 4×4 probe at 300 hits
        let mut d = tr_core::particles::Dust {
            dust_type: 33,
            position: Vec2::new(500.0, 300.0),
            velocity: Vec2::new(1.0, 0.0),
            alpha: 38,
            scale: 0.6,
            no_gravity: true,
            active: true,
            ..Default::default()
        };
        water_splash_arm(&mut d, &ww);
        assert_eq!(d.alpha, 60, "38 + 20 (wet) + 2 (fade)");
        assert!((d.scale - (0.6 - 0.1 - 0.005 + 0.03)).abs() < 1e-6);
        // The alpha-255 kill zeroes the scale (`:1852-1855`).
        let mut d = tr_core::particles::Dust {
            dust_type: 33,
            velocity: Vec2::new(1.0, 0.0),
            alpha: 254,
            scale: 0.6,
            no_gravity: true,
            active: true,
            ..Default::default()
        };
        water_splash_arm(&mut d, &w);
        assert_eq!(d.alpha, 256);
        assert_eq!(d.scale, 0.03, "zeroed then the +0.03 grow");
        // A non-family type is a no-op (the hook falls through).
        let mut d = tr_core::particles::Dust {
            dust_type: 213,
            scale: 1.0,
            alpha: 10,
            active: true,
            ..Default::default()
        };
        let before = d;
        assert!(!water_splash_arm(&mut d, &w));
        assert_eq!(d, before);
    }

    /// W4-g67 witness — the splash family rides the SHARED `update_dust`
    /// loop (the minecart::tick seam): the head move precedes the arm, the
    /// generic no-gravity tail (`Dust.cs:2335`) runs alongside (the arm
    /// returns false), and the scale-floor kill fires below 0.1.
    #[test]
    fn w4_g67_splashes_tick_through_the_shared_pool_loop() {
        let w = World::new("air".into(), 64, 64);
        let mut pool = tr_core::particles::DustPool::new();
        {
            let d = pool.slot_mut(0);
            d.active = true;
            d.dust_type = 33;
            d.position = Vec2::new(500.0, 300.0);
            d.velocity = Vec2::new(0.0, -1.0);
            d.alpha = 38;
            d.scale = 0.6;
            d.no_gravity = true;
            d.frame_x = 330;
            d.frame_w = 8;
            d.frame_h = 8;
        }
        let mut rng = tr_core::UnifiedRandom::new(7);
        crate::minecart::tick(
            &mut pool,
            &mut rng,
            6000,
            Vec2::new(500.0, 300.0),
            Vec2::new(1600.0, 900.0),
            1.0,
            &w,
            0.0,
            false,
        );
        let d = pool.slot(0);
        assert!(d.active);
        assert_eq!(d.position, Vec2::new(500.0, 299.0), "the :477 move first");
        // Arm: alpha 40, scale 0.615, rot 0.7, vel (0, -0.85); then the
        // tail's ×0.92 damp and fadeIn-gated −0.04.
        assert_eq!(d.alpha, 40);
        assert!((d.scale - (0.615 - 0.04)).abs() < 1e-6, "arm + tail shrink");
        assert!((d.velocity.y - (-0.85 * 0.92)).abs() < 1e-6, "the tail damp");
        // The scale floor: a 0.11 splash with moving X shrinks under 0.1 in
        // one tick (arm −0.005 + 0.03 = +0.025, tail −0.04 → net −0.015).
        let d = pool.slot_mut(0);
        d.scale = 0.11;
        d.velocity = Vec2::new(1.0, 0.0);
        crate::minecart::tick(
            &mut pool,
            &mut rng,
            6000,
            Vec2::new(500.0, 300.0),
            Vec2::new(1600.0, 900.0),
            1.0,
            &w,
            0.0,
            false,
        );
        assert!(!pool.slot(0).active, "0.11 − 0.015 < the 0.1 floor");
    }

    /// W4-g67 witness — the RNG census: N solid impacts consume exactly
    /// N × (1 gate + 6 default-block) draws on Rain's stream, in drop order;
    /// the pool's pressure bands never roll at slot 0..N (dCount 0, the
    /// low band resets it). A second Rain from the same seed replays the
    /// identical stream.
    #[test]
    fn w4_g67_the_splash_draws_ride_rains_stream_in_kill_order() {
        let w = tall_ground();
        let camera = Vec2::new(32.0 * 16.0, 25.0 * 16.0);
        let screen = Vec2::new(1600.0, 200.0);
        let mut a = Rain::with_forced(7, Some(1.0), false);
        let mut b = Rain::with_forced(7, Some(1.0), false);
        full_quality(&mut a);
        full_quality(&mut b);
        for r in [&mut a, &mut b] {
            r.drops.clear();
            for k in 0..5 {
                r.drops.push(Drop {
                    pos: Vec2::new(400.0 + k as f32 * 32.0, 470.0),
                    vel: Vec2::new(0.0, 14.0),
                    scale: 1.0,
                    rot: 0.0,
                    kind: 0,
                });
            }
        }
        let mut pa = tr_core::particles::DustPool::new();
        let mut pb = tr_core::particles::DustPool::new();
        let ctx = impact_ctx();
        a.update_drops(&w, camera, screen, 1.0, Some((&mut pa, &ctx)), DropSimInputs::default());
        b.update_drops(&w, camera, screen, 1.0, Some((&mut pb, &ctx)), DropSimInputs::default());
        assert_eq!(a.rng, b.rng, "the same seed replays the same draws");
        assert_eq!(splash_count(&pa), 5);
        for i in 0..tr_core::particles::DUST_SENTINEL {
            assert_eq!(pa.slot(i), pb.slot(i), "slot {i} byte-equal");
        }
        // The census: 5 kills × 7 draws from the seed state.
        let mut twin = Rain::with_forced(7, Some(1.0), false).rng;
        for _ in 0..(5 * 7) {
            crate::app::next_rand(&mut twin);
        }
        assert_eq!(a.rng, twin, "5 × (gate + 6 defaults), no extra draws");
    }

    /// W4-g67 residual witness — the `gameMenu` kill-only arm
    /// (`Rain.cs:103-109`): the drop still MOVES (`:102` runs first), dies
    /// only below `screenPosition.Y + screenHeight + 2000`, and NEITHER the
    /// gfx gate NOR a splash fires (the else-if chain never reaches `:117`).
    /// view bottom = 400 + 100 = 500 → menu bound 2500.
    #[test]
    fn w4_g67_menu_arm_kills_only_below_plus_2000_without_a_roll() {
        let w = tall_ground();
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        full_quality(&mut rain);
        let camera = Vec2::new(32.0 * 16.0, 25.0 * 16.0);
        let screen = Vec2::new(1600.0, 200.0);
        let sim = DropSimInputs { game_menu: true, ..Default::default() };
        // A drop INSIDE the solid row-30 floor (y 484) survives the menu
        // arm — the C# menu branch never probes collisions.
        one_falling_drop(&mut rain, Vec2::new(500.0, 470.0), Vec2::new(0.0, 14.0));
        let mut pool = tr_core::particles::DustPool::new();
        let ctx = impact_ctx();
        let state = rain.rng;
        rain.update_drops(&w, camera, screen, 1.0, Some((&mut pool, &ctx)), sim);
        assert_eq!(rain.drops.len(), 1, "no collision kill in menu mode");
        assert_eq!(rain.drops[0].pos, Vec2::new(500.0, 484.0), ":102 moved it");
        assert_eq!(rain.rng, state, "the gate never rolls");
        assert_eq!(splash_count(&pool), 0);
        // Below 2500: kill-only, still no roll, no splash.
        one_falling_drop(&mut rain, Vec2::new(500.0, 2490.0), Vec2::new(0.0, 14.0));
        rain.update_drops(&w, camera, screen, 1.0, Some((&mut pool, &ctx)), sim);
        assert_eq!(rain.drops.len(), 0, "2504 > 2500");
        assert_eq!(rain.rng, state, "the menu kill rolls nothing");
        assert_eq!(splash_count(&pool), 0);
    }

    /// W4-g67 residual witness — the `remixWorld` kill-only arm
    /// (`Rain.cs:110-116`): below `+100` (the SAME bound as the in-world
    /// combined branch) but WITHOUT the gate roll / splash / collision
    /// probes. view bottom 500 → bound 600.
    #[test]
    fn w4_g67_remix_arm_kills_below_plus_100_without_a_roll() {
        let w = tall_ground();
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        full_quality(&mut rain);
        let camera = Vec2::new(32.0 * 16.0, 25.0 * 16.0);
        let screen = Vec2::new(1600.0, 200.0);
        let sim = DropSimInputs { remix_world: true, ..Default::default() };
        // Inside the solid floor: survives (no SolidCollision probe).
        one_falling_drop(&mut rain, Vec2::new(500.0, 470.0), Vec2::new(0.0, 14.0));
        let mut pool = tr_core::particles::DustPool::new();
        let ctx = impact_ctx();
        let state = rain.rng;
        rain.update_drops(&w, camera, screen, 1.0, Some((&mut pool, &ctx)), sim);
        assert_eq!(rain.drops.len(), 1, "remix skips the collision probe");
        assert_eq!(rain.rng, state, "no gate roll");
        assert_eq!(splash_count(&pool), 0);
        // Below the +100 bound: kill-only.
        one_falling_drop(&mut rain, Vec2::new(500.0, 591.0), Vec2::new(0.0, 14.0));
        rain.update_drops(&w, camera, screen, 1.0, Some((&mut pool, &ctx)), sim);
        assert_eq!(rain.drops.len(), 0, "605 > 600");
        assert_eq!(rain.rng, state, "the remix kill rolls nothing");
        assert_eq!(splash_count(&pool), 0, "no splash on the remix kill");
    }

    /// W4-g67 census witness — `MakeRain`'s `AnyDanger` throttle
    /// (`Rain.cs:37-45`): the attempt rate drops to 0.05x. At intensity 1 /
    /// gfx 1 the calm rate is `1600/1920*25*1.25 = 26.04` (26 attempts the
    /// first tick); throttled it is `1.302` (1 attempt). Open-air world so
    /// every attempt spawns.
    #[test]
    fn w4_g67_any_danger_throttles_the_spawn_rate_to_005() {
        let w = World::new("air".into(), 64, 64);
        let camera = Vec2::new(32.0 * 16.0, 25.0 * 16.0);
        let screen = Vec2::new(1600.0, 200.0);
        let mut calm = Rain::with_forced(7, Some(1.0), false);
        calm.gfx_quality = 1.0;
        calm.spawn(&w, camera, screen, 1.0, 0.0, 1.0, 0, true, DropSimInputs::default());
        assert_eq!(calm.drops.len(), 26, "floor(26.04) calm attempts");
        let mut danger = Rain::with_forced(7, Some(1.0), false);
        danger.gfx_quality = 1.0;
        let sim = DropSimInputs { any_danger: true, ..Default::default() };
        danger.spawn(&w, camera, screen, 1.0, 0.0, 1.0, 0, true, sim);
        assert_eq!(danger.drops.len(), 1, "floor(26.04*0.05)=1 throttled attempt");
    }

    /// W4-g67 census witness — the `MakeRain`/`NewRain` RNG census: one open
    /// spawn attempt draws exactly 4 on Rain's stream — X `Next(lo,hi)`, Y
    /// `Next(20,100)`, scale `Next(-20,21)`, frame `Next(3)` (`Rain.cs:54-55`,
    /// `:202-205`). 26 calm attempts → 104 draws, replayed draw-for-draw.
    #[test]
    fn w4_g67_spawn_attempts_draw_four_each_in_order() {
        let w = World::new("air".into(), 64, 64);
        let camera = Vec2::new(32.0 * 16.0, 25.0 * 16.0);
        let screen = Vec2::new(1600.0, 200.0);
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        rain.gfx_quality = 1.0;
        let mut twin = rain.rng;
        rain.spawn(&w, camera, screen, 1.0, 0.0, 1.0, 0, true, DropSimInputs::default());
        assert_eq!(rain.drops.len(), 26);
        for _ in 0..(26 * 4) {
            crate::app::next_rand(&mut twin);
        }
        assert_eq!(rain.rng, twin, "26 attempts × (X, Y, scale, frame)");
        // The frame type is the water-style row: `waterStyle*3 + Next(3)`.
        let style = crate::biome::calculate_water_style(&w, camera) as u8;
        for d in &rain.drops {
            assert!(d.kind >= style * 3 && d.kind < style * 3 + 3, "kind {}", d.kind);
            assert!((d.scale - 1.0).abs() <= 0.2 + 1e-6, "scale in [0.8,1.2]");
        }
    }

    /// W4-g67 census witness — `NewRainForced` (`Rain.cs:135-164`): the budget
    /// is `maxRain` cut by `(1+q)/2` when q<0.9, ignoring cloudAlpha/snow/atmo.
    /// At q 0 the cap is 375; the 376th spawn refuses with the `maxRain`
    /// sentinel. The spawn itself takes the `:159-162` shape.
    #[test]
    fn w4_g67_new_rain_forced_budget_and_shape() {
        let w = World::new("air".into(), 64, 64);
        let camera = Vec2::new(32.0 * 16.0, 25.0 * 16.0);
        let mut rain = Rain::with_forced(7, None, false); // intensity 0 — forced ignores it
        rain.gfx_quality = 0.0; // (1+0)/2 = 0.5 < 0.9 → cap 750*0.5 = 375
        let pos = Vec2::new(500.0, 300.0);
        let vel = Vec2::new(2.0, 9.0);
        let idx = rain.new_rain_forced(&w, camera, pos, vel);
        assert_eq!(idx, 0);
        let d = rain.drops[0];
        assert_eq!(d.pos, pos);
        assert!((d.scale - 1.0).abs() <= 0.2 + 1e-6);
        assert_eq!(d.vel, vel * d.scale, "`:160` velocity * scale");
        assert_eq!(d.rot, d.vel.x.atan2(-d.vel.y), "`:161`");
        // Fill to the cap, then the next refuses.
        while rain.drops.len() < 375 {
            rain.new_rain_forced(&w, camera, pos, vel);
        }
        assert_eq!(rain.new_rain_forced(&w, camera, pos, vel), MAX_DROPS, "full → maxRain");
        assert_eq!(rain.drops.len(), 375);
    }

    /// W4-g67 census witness — `ClearRain` (`Rain.cs:22-28`) empties the field;
    /// `IsItRaining` (`Main.cs:2597`) follows `cloudAlpha`, NOT the event flag.
    #[test]
    fn w4_g67_clear_rain_and_is_it_raining_semantics() {
        let w = World::new("air".into(), 64, 64);
        let camera = Vec2::new(32.0 * 16.0, 25.0 * 16.0);
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        rain.gfx_quality = 1.0;
        rain.spawn(&w, camera, Vec2::new(1600.0, 200.0), 1.0, 0.0, 1.0, 0, true, DropSimInputs::default());
        assert!(rain.drop_count() > 0);
        rain.clear_rain();
        assert_eq!(rain.drop_count(), 0, "ClearRain deactivates every drop");
        // The event flag is off but the clouds still linger → IsItRaining true.
        rain.raining = false;
        rain.intensity = 0.5;
        assert!(!rain.is_raining(), "Main.raining is the event flag");
        assert!(rain.is_it_raining(), "cloudAlpha 0.5 > 0 → IsItRaining");
        rain.intensity = 0.0;
        assert!(!rain.is_it_raining(), "cloudAlpha 0 → not raining");
    }

    /// W4-g67 census witness — the `UpdateTime` start gate
    /// (`Main.cs:64890`): with `start_gates_open` false (slime rain / lantern
    /// night) NO start roll runs — the stream is untouched and the event never
    /// begins; with it open the same seed's ladder fires.
    #[test]
    fn w4_g67_start_gates_close_suppresses_every_start_roll() {
        let w = rain_world();
        let mut closed = Rain::with_forced(42, None, false);
        let state = closed.rng;
        closed.update_time(60, &w, true, false, 0.0, false);
        assert_eq!(closed.rng, state, "no roll while the gates are closed");
        assert!(!closed.is_raining());
        // The open twin at the same seed consumes the ladder.
        let mut open = Rain::with_forced(42, None, false);
        open.update_time(60, &w, true, false, 0.0, true);
        assert_ne!(open.rng, state, "the open gate rolls the start arms");
    }

    /// W4-g67 residual witness — the `DrawDust` cull rect (`Main.cs:38832` /
    /// `:38847` / `:39020-39022`): the ±1000/±1050 slack box off
    /// `screenPosition`, the 4×4 strict-`<` intersection, and the else-arm
    /// DEACTIVATION (the old widening dropped the kill). Pure geometry — the
    /// draw loop applies exactly this test before drawing each family slot.
    #[test]
    fn w4_g67_drawdust_cull_rect_is_the_wide_box_with_an_else_kill() {
        // screenPosition (0,0), screen 1600×900 → rect (-1000,-1050,3600,3000).
        let rect = splash_cull_rect(Vec2::ZERO, Vec2::new(1600.0, 900.0));
        assert_eq!(rect, (-1000, -1050, 3600, 3000), ":38832's wide box");
        // Inside: a splash near the view centre.
        assert!(dust_in_cull_rect(Vec2::new(800.0, 450.0), rect));
        // The slack margins: just inside the left/top edges still draw.
        assert!(dust_in_cull_rect(Vec2::new(-999.0, -1049.0), rect));
        // Outside → the else arm kills. Past the right edge (x ≥ 2600).
        assert!(!dust_in_cull_rect(Vec2::new(2600.0, 450.0), rect));
        // Past the bottom edge (y ≥ 1950).
        assert!(!dust_in_cull_rect(Vec2::new(800.0, 1950.0), rect));
        // Far left (x + 4 ≤ -1000 → x ≤ -1004).
        assert!(!dust_in_cull_rect(Vec2::new(-1004.0, 450.0), rect));
        // The 4×4 box: a dust at x = -1003 still intersects (−1003+4 > −1000).
        assert!(dust_in_cull_rect(Vec2::new(-1003.0, 450.0), rect));
        // Non-zero screenPosition shifts the box (the camera-centred mapping).
        let shifted = splash_cull_rect(Vec2::new(5000.0, 3000.0), Vec2::new(1600.0, 900.0));
        assert_eq!(shifted, (4000, 1950, 3600, 3000));
        assert!(dust_in_cull_rect(Vec2::new(5000.0, 3000.0), shifted));
        assert!(!dust_in_cull_rect(Vec2::new(800.0, 450.0), shifted), "the old view is now outside");
    }

    /// W4-g67 witness — the skyblock snow-threshold arm (`Rain.cs:181` via
    /// `SceneMetrics.SnowTileThreshold`, `SceneMetrics.cs:154-163`): 150 snow
    /// tiles suppress the live-drop budget to 562 on a low-tiles skyblock world
    /// (150/300 = 0.5 → 1−0.25 = 0.75 → 750·0.75 = 562) but leave it at the full
    /// 750 on a normal world (150/1500 = 0.1 → 0.99 → 742, above the 26-attempt
    /// spawn). The 26 calm attempts fill whichever budget is lower.
    /// WRITTEN NOT RUN (code-only).
    #[test]
    fn w4_g67_skyblock_low_tiles_lowers_the_snow_threshold() {
        let camera = Vec2::new(32.0 * 16.0, 25.0 * 16.0);
        let screen = Vec2::new(1600.0, 200.0);
        // Normal world: threshold 1500, budget 742 > 26 attempts → 26 drops.
        let normal = World::new("air".into(), 64, 64);
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        rain.gfx_quality = 1.0;
        rain.spawn(&normal, camera, screen, 1.0, 0.0, 1.0, 150, true, DropSimInputs::default());
        assert_eq!(rain.drop_count(), 26, "normal threshold barely suppresses");
        // Low-tiles skyblock world: threshold 300, budget 562 — still > 26, so
        // the attempt count is unchanged; the BUDGET is what the arm moves. Pin
        // it directly through the threshold getter the spawn reads.
        assert_eq!(tr_core::scene::snow_tile_threshold(false), 1500);
        assert_eq!(tr_core::scene::snow_tile_threshold(true), 300);
        // A skyblock world with 300+ snow tiles fully suppresses (ratio 1 →
        // budget 0): no drop spawns at all.
        let mut sky = World::new("sky".into(), 64, 64);
        sky.skyblock_low_tiles = true;
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        rain.gfx_quality = 1.0;
        rain.spawn(&sky, camera, screen, 1.0, 0.0, 1.0, 300, true, DropSimInputs::default());
        assert_eq!(rain.drop_count(), 0, "300 snow tiles fully suppress on skyblock");
        // The same 300 tiles on a normal world: ratio 0.2 → budget 720 > 26.
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        rain.gfx_quality = 1.0;
        rain.spawn(&normal, camera, screen, 1.0, 0.0, 1.0, 300, true, DropSimInputs::default());
        assert_eq!(rain.drop_count(), 26, "300 tiles barely suppress on a normal world");
    }
}
