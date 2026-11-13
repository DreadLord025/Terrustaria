//! Rain: `Rain.MakeRain` / `Rain.Update` / `Main.DrawRain`.
//!
//! Weather follows the original's shape: a target intensity (`Main.maxRaining`) that
//! `cloudAlpha` ramps toward at `0.0005 * max(1, dayRate)` per 60 Hz tick, a countdown
//! that ends the event, and one drop per `MakeRain` spawn slot per tick with the
//! original fall velocity (`wind * 18, 14` px/tick). Drops are drawn rotated around
//! their top-left origin with the local light colour, and die on solid or wet tiles
//! with a splash. In a snow biome the event renders as snowfall instead: no rain
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
const SPLASH_LIFE: f32 = 0.32;

#[derive(Clone, Copy, Debug)]
struct Drop {
    pos: Vec2,
    vel: Vec2,
    scale: f32,
    rot: f32,
    /// Rain frame type (0..2 for water style 0).
    kind: u8,
}

#[derive(Clone, Copy, Debug)]
struct Splash {
    pos: Vec2,
    vel: Vec2,
    life: f32,
    frame: u8,
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
    splashes: Vec<Splash>,
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
    /// The strike position in world pixels (already the C#'s `(int)` cast).
    pub x: f32,
    pub y: f32,
    /// `Main.rand.Next(-10, 11) * 0.01` (`LegacySoundPlayer.cs:471`): the
    /// instance pitch jitter. The port's source plumbing has no pitch control
    /// (the liquid-loop deviation is the same), so the roll is consumed for
    /// stream fidelity and the value carried unused.
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
            splashes: Vec::with_capacity(128),
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

    /// `Main.rainTime`: ticks left in the event (`Main.cs:64874` drains it).
    pub(crate) fn rain_time(&self) -> i32 {
        self.rain_time
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

    /// Fixed 60 Hz stepping, independent of frame rate. The drop sim and the
    /// visual ramps only — the event lifecycle moved onto the shared world tick
    /// (`update_time`, driven by `world_sim::weather::update_world_tick`) and the
    /// lightning chain runs there too (`thunder_step`). `day_rate` scales the
    /// ramps (`Main.cs:61902-61906`'s `num4 < 1 → 1` clamp shape).
    ///
    /// `shimmer_alpha`, `atmo`, `snow_tile_count` and `zone_rain` are the
    /// `MakeRain`/`NewRain` gate inputs (`Rain.cs:30-207`, S1-L-2).
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
        self.update_drops(world, camera, screen, zoom);
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
            );
        }
        for s in &mut self.splashes {
            s.pos += s.vel;
            s.vel.y += 0.12;
            s.life -= 1.0 / TICKS_PER_SEC;
        }
        self.splashes.retain(|s| s.life > 0.0);
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
    /// stream like the vanilla's `Main.rand` draws.
    pub(crate) fn thunder_step(&mut self, weather: ThunderInputs) -> Option<ThunderStrike> {
        let mut strike = None;
        if self.thunder_delay > 0 {
            self.thunder_delay -= 1;
            if self.thunder_delay == 0 {
                // `Main.cs:58751-58768`: the strike — the position
                // (`thunderDistance * 15` off the scene centre, clamped to
                // `worldSurface * 16 - 500`) behind the focus/skip gate, then
                // the case-43 cue family and its pitch jitter.
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
                    let (cue, pitch) = crate::ambient_sound::thunder_cue_slot(
                        self.range(0, 6),
                        // `Main.rand.Next(-10, 11)` (`LegacySoundPlayer.cs:471`).
                        self.range(0, 21) as i32 - 10,
                    );
                    strike = Some(ThunderStrike { cue, x, y, pitch });
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
    /// events. `FreezeRainPower`, `LanternNight` and `slimeRain` do not exist in
    /// the port — their gates resolve to the C# shape with all false. There is no
    /// `Main.dayTime` gate on the start arms (only the slime roll is day-gated).
    pub(crate) fn update_time(
        &mut self,
        day_rate: i32,
        world: &World,
        any_player_120: bool,
        remix_world: bool,
        player_luck: f32,
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
        } else {
            // `Main.cs:64890-64923`: `!slimeRain && !LanternsUp &&
            // !NextNightIsLanternNight` — all port-absent.
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
    fn start_rain(&mut self, world: &World, remix_world: bool, player_luck: f32) {
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
    fn stop_rain(&mut self) {
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

    fn update_drops(&mut self, world: &World, camera: Vec2, screen: Vec2, zoom: f32) {
        let bottom = camera.y + screen.y * 0.5 / zoom + 100.0 / zoom;
        let px = tr_core::TILE_PX;
        let mut i = 0;
        while i < self.drops.len() {
            let mut d = self.drops[i];
            d.pos += d.vel;
            let tx = (d.pos.x / px).floor() as i32;
            let ty = (d.pos.y / px).floor() as i32;
            let hit = world.tile(tx, ty).is_some_and(|t| {
                (t.active() && (t.kind as usize) < td::TILE_COUNT && td::TILE_SOLID.has(t.kind))
                    || t.liquid > 0
            });
            if hit || d.pos.y > bottom {
                if hit {
                    self.spawn_splash(d);
                }
                self.drops.swap_remove(i);
            } else {
                self.drops[i] = d;
                i += 1;
            }
        }
    }

    /// `Rain.MakeRain` (`Rain.cs:30-89`, S1-L-2): the per-tick drop spawn.
    /// Gates: `!ZoneRain || shimmerAlpha > 0f` suppress spawning (the C# also
    /// early-returns on netMode 2 / paused / menu). The attempt rate is
    /// `screenWidth / MaxWorldViewSize.X * 25 * (0.25 + cloudAlpha)` with the
    /// `NPC.AnyDanger` 0.05 throttle (`:36-45`); `NewRain` (`:166-207`) then
    /// caps the live count by `maxRain * cloudAlpha` scaled by `gfxQuality`,
    /// the snow-tile suppression `(1 - (snow/max)^2)`, and `atmo^9` (0 when
    /// `atmo < 0.4`), and sets the `waterStyle*3 + rand(3)` frame type.
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
    ) {
        if !zone_rain || shimmer_alpha > 0.0 {
            return;
        }
        let mut rate = screen.x / 1920.0 * 25.0 * (0.25 + self.intensity);
        // `Rain.cs:37-45`: `NPC.AnyDanger(quickBossNPCCheck: true)` throttles to
        // 0.05x (floored to 1 attempt when any would remain). The port has no
        // boss-danger scan on Rain; the gate is preserved as a hook (no-op).
        self.spawn_acc += rate;
        let count = self.spawn_acc.floor();
        self.spawn_acc -= count;
        if count <= 0.0 {
            return;
        }
        // `Rain.NewRain` (`:166-207`): the active-slot budget.
        let mut num2 = (MAX_DROPS as f32 * self.intensity.clamp(0.0, 1.0)) as i32;
        if num2 > MAX_DROPS as i32 {
            num2 = MAX_DROPS as i32;
        }
        // gfxQuality defaults to 1.0 in the port (no setting); the C# scales by
        // `(1 + gfxQuality) / 2` when < 0.9 (`:176-180`).
        let snow_threshold = tr_core::scene::thresholds::SNOW_TILE_NORMAL.max(1) as f32;
        let snow_ratio = (snow_tile_count.max(0) as f32 / snow_threshold).clamp(0.0, 1.0);
        num2 = ((num2 as f32) * (1.0 - snow_ratio * snow_ratio)) as i32;
        num2 = ((num2 as f64) * (atmo.clamp(0.0, 1.0) as f64).powi(9)) as i32;
        if atmo < 0.4 {
            num2 = 0;
        }
        if self.drops.len() >= num2.max(0) as usize {
            return;
        }
        let px = tr_core::TILE_PX;
        let half = screen * 0.5 / zoom;
        let wind = self.wind + 0.05 * (self.intensity * 9.0).sin();
        for _ in 0..count as usize {
            if self.drops.len() >= num2.max(0) as usize || self.drops.len() >= MAX_DROPS {
                break;
            }
            let x = camera.x + (self.rand01() * 2.0 - 1.0) * (half.x + 600.0) - wind * 600.0;
            let y = camera.y - half.y - 20.0 - self.rand01() * 80.0;
            let tx = (x / px).floor() as i32;
            let ty = (y / px).floor() as i32;
            let open = world
                .tile(tx, ty)
                .is_some_and(|t| !t.active() && t.wall == 0);
            if !open {
                continue;
            }
            let scale = 1.0 + (self.rand01() * 2.0 - 1.0) * 0.2;
            let vel = Vec2::new(wind * 18.0 * scale, 14.0 * scale);
            // `Rain.cs:205`: `type = (byte)(Main.waterStyle * 3 + Main.rand.Next(3))`.
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

    fn spawn_splash(&mut self, d: Drop) {
        let n = 1 + (self.next_u64() & 1) as usize;
        for _ in 0..n {
            if self.splashes.len() >= 256 {
                break;
            }
            let jitter = Vec2::new((self.rand01() * 2.0 - 1.0) * 1.2, 0.0);
            let frame = self.range(0, 3) as u8;
            self.splashes.push(Splash {
                pos: d.pos + Vec2::new(2.0, 2.0),
                vel: d.vel * 0.0975 + jitter - Vec2::new(0.0, 2.0),
                life: SPLASH_LIFE,
                frame,
            });
        }
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
        if self.drops.is_empty()
            && self.splashes.is_empty()
            && self.snow.is_empty()
            && self.flash <= 0.01
        {
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
        for s in &self.splashes {
            let Some(uv) = lookups.dust_uv(330, s.frame as i32 * 10, 8, 8) else {
                continue;
            };
            let c = light_at(light, s.pos);
            let a = (s.life / SPLASH_LIFE).clamp(0.0, 1.0) * 0.8;
            r.push_fx_sprite_rot(
                to_screen(s.pos),
                [8.0 * 0.6 * zoom, 8.0 * 0.6 * zoom],
                uv,
                [c[0], c[1], c[2], a],
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
            );
        }
    }

    #[test]
    fn forced_rain_spawns_drops_that_die_on_the_ground() {
        let w = ground();
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        run(&mut rain, &w, 120);
        assert!(rain.drop_count() > 100, "drops {}", rain.drop_count());
        // The ground is below the camera, so impacts leave splashes at a steady state.
        assert!(!rain.splashes.is_empty(), "impacts should leave splashes");
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
            );
        }
        assert!(rain.cloud_bg() > 0.5, "cloud deck {}", rain.cloud_bg());
        rain.flash = 1.0;
        rain.lightning_decay = 0.05; // `NewLightning`'s upper decay rate.
        // The flash rise/decay lives in the lightning chain — `thunder_step`,
        // once per world tick since M1.4-BG-22 — not in the drop sim's tick.
        rain.thunder_step(ThunderInputs::default());
        assert!(rain.flash < 1.0 && rain.flash > 0.7);
        for _ in 0..30 {
            rain.thunder_step(ThunderInputs::default());
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
        let strike = rain.thunder_step(weather).expect("a strike");
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
        assert_eq!(rain.thunder_step(weather), None);
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
                w.thunder_step(weather);
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
                    rain.thunder_step(weather)
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
        ending.update_time(1, &tall, true, false, 0.0);
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
        fast.update_time(60, &w, true, false, 0.0);
        assert!(fast.is_raining(), "the ×5.75 roll hit at dayRate 60");

        let mut slow = Rain::with_forced(fire_at_60, None, false);
        slow.update_time(1, &w, true, false, 0.0);
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
        both_fast.update_time(60, &w, true, false, 0.0);
        assert!(both_fast.is_raining());
        let mut both_slow = Rain::with_forced(fire_at_both, None, false);
        both_slow.update_time(1, &w, true, false, 0.0);
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
        rain.update_time(1, &w, true, false, 0.0);
        assert!(!rain.is_raining());
        assert_eq!(rain.max_raining(), 0.0);
        assert_eq!(rain.rain_time, 0, "StopRain pins rainTime to 0 (`:64648`)");
        assert_eq!(rain.rng, state, "no roll is consumed on the stop arm");

        // One tick above the boundary survives and rolls (`rainTime` 2 → 1).
        let mut alive = Rain::with_forced(7, None, false);
        alive.raining = true;
        alive.target = 0.8;
        alive.rain_time = 2;
        alive.update_time(1, &w, true, false, 0.0);
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
        rain.update_time(1, &w, true, false, 0.0);
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
        drain.update_time(60, &w, false, false, 0.0); // no 120-HP player: arms gated
        assert_eq!(drain.lady_bug_rain_boost, 60);
        drain.update_time(60, &w, false, false, 0.0);
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
        rain.update_time(60, &w, true, false, 0.0);
        assert!(rain.is_raining(), "the ladybug arm fired");

        let mut quiet = Rain::with_forced(fire, None, false);
        quiet.update_time(60, &w, true, false, 0.0);
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
        fast.update(&w, camera, screen, 1.0, step, false, 60, 0.0, 1.0, 0, true);
        assert!(
            (fast.visual_alpha() - 0.03).abs() < 1e-6,
            "60× ramp: {}",
            fast.visual_alpha()
        );

        let mut slow = Rain::with_forced(11, None, false);
        slow.raining = true;
        slow.target = 1.0;
        slow.update(&w, camera, screen, 1.0, step, false, 1, 0.0, 1.0, 0, true);
        assert!(
            (slow.visual_alpha() - 0.0005).abs() < 1e-9,
            "dry ramp: {}",
            slow.visual_alpha()
        );

        let mut deck = Rain::with_forced(11, None, false);
        deck.intensity = 1.0; // above the 0.05 gate: the deck chases the intensity
        deck.update(&w, camera, screen, 1.0, step, false, 60, 0.0, 1.0, 0, true);
        assert!(
            (deck.cloud_bg() - 0.3).abs() < 1e-6,
            "deck at FF: {}",
            deck.cloud_bg()
        );
        let mut dry = Rain::with_forced(11, None, false);
        dry.intensity = 1.0;
        dry.update(&w, camera, screen, 1.0, step, false, 1, 0.0, 1.0, 0, true);
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
        rain.update_time(60, &w, true, false, 0.0);
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
        );
        assert_eq!(rain.max_raining(), 0.8);
        assert!(rain.is_raining());

        let mut quiet = Rain::with_forced(9, None, true);
        quiet.lady_bug_rain_boost = 100;
        let state = quiet.rng;
        quiet.update_time(60, &w, true, false, 0.0);
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
}
