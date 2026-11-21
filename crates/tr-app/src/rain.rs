//! Rain: `Rain.MakeRain` / `Rain.Update` / `Main.DrawRain`.
//!
//! Weather follows the original's shape: a target intensity (`Main.maxRaining`) that
//! `cloudAlpha` ramps toward at `0.0005 * max(1, dayRate)` per 60 Hz tick, a countdown
//! that ends the event, and one drop per `MakeRain` spawn slot per tick with the
//! original fall velocity (`wind * 18, 14` px/tick). Drops are drawn rotated around
//! their top-left origin with the local light colour, and die on solid or wet tiles
//! with a splash. The ambient snowfall is NOT this rig's: vanilla's `Main.snowing()`
//! (`Main.cs:13032-13116`) spawns `Dust` type 76 into the shared pool wherever the
//! scene metrics count snow tiles at the surface — `crate::snow` owns that spawn,
//! AI and draw (W4-d130); the old biome-gated flake approximation is retired.
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

/// `Main.maxRain` (`Main.cs:66801`).
const MAX_DROPS: usize = 750;
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

#[derive(Clone, Copy, Debug, Default)]
struct Drop {
    active: bool,
    pos: Vec2,
    vel: Vec2,
    scale: f32,
    rot: f32,
    /// Rain frame type (`waterStyle * 3 + rand(3)`, `Rain.cs:205`).
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
    /// `Main.rain` — the 750-slot pool (`Main.cs:66801`): slot order IS the
    /// `DrawRain`/`Update` walk order and `NewRain` takes the first inactive
    /// slot (`Rain.cs:187-194`), so the pool keeps fixed slots, never compacts.
    drops: Vec<Drop>,
    splashes: Vec<Splash>,
    sparkles: Vec<Sparkle>,
    rng: u64,
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

/// The per-tick `Main` statics `MakeRain`/`NewRain`/`Rain.Update` read
/// (`Rain.cs:30-133`): the port gathers them once per frame in
/// `world_sim::weather::update_rain` — vanilla reads them live inside the
/// spawn/draw, the values are constant across one frame.
#[derive(Clone, Copy, Debug)]
pub struct RainInputs {
    /// `Main.screenPosition` — the view's world-px CENTRE (the port's camera).
    pub camera: Vec2,
    /// `Main.screenWidth`/`screenHeight` — the UI size.
    pub screen: Vec2,
    /// The view zoom (the port draws at `zoom` inside the same pixels).
    pub zoom: f32,
    /// `Main.dayRate` — the visual-ramp scale.
    pub day_rate: i32,
    /// `Main.windSpeedCurrent` — the live wind (`Rain.cs:56/93`).
    pub wind_speed_current: f32,
    /// `Main.player[Main.myPlayer].velocity` — the spawn-window shaping
    /// (`Rain.cs:50-57`); zero when no player exists.
    pub player_velocity: Vec2,
    /// `Main.shimmerAlpha` (`Rain.cs:32`).
    pub shimmer_alpha: f32,
    /// `Main.atmo` (`Rain.cs:182-186`).
    pub atmo: f32,
    /// `Main.SceneMetrics.SnowTileCount` (`Rain.cs:181`).
    pub snow_tile_count: i32,
    /// `Main.SceneMetrics.ZoneRain` (`Rain.cs:32`).
    pub zone_rain: bool,
    /// `Main.waterStyle` (`Rain.cs:162/205` — the frame type).
    pub water_style: u32,
    /// `NPC.AnyDanger(quickBossNPCCheck: true)` (`Rain.cs:37-45`).
    pub any_danger: bool,
    /// `Main.gfxQuality` (`Rain.cs:120/176-180`).
    pub gfx_quality: f32,
    /// `Main.remixWorld` (`Rain.cs:84/110-116`).
    pub remix_world: bool,
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
        Self {
            intensity: 0.0,
            target: 0.0,
            raining: false,
            rain_time: 0,
            lady_bug_rain_boost: 0,
            calm: start_calm,
            cloud_bg: 0.0,
            flash: 0.0,
            thunder_delay: 0,
            storming: false,
            thunder_distance: 0,
            lightning_speed: 0.0,
            lightning_decay: 0.008,
            drops: vec![Drop::default(); MAX_DROPS],
            splashes: Vec::with_capacity(128),
            sparkles: Vec::with_capacity(64),
            rng,
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

    /// `Main.cloudAlpha` unclamped — the snowfall's intensity/budget reads
    /// the raw value (`Main.cs:13044-13046`), not the sky-tint clamp.
    pub fn cloud_alpha(&self) -> f32 {
        self.intensity
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

    /// Live drops (`Rain.active` slots) — the debug readout.
    pub fn drop_count(&self) -> usize {
        self.drops.iter().filter(|d| d.active).count()
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
    pub fn update(&mut self, world: &World, inputs: &RainInputs, dt: f32) {
        let step = 1.0 / TICKS_PER_SEC;
        let mut acc = self.accum + dt.min(0.25);
        let mut steps = 0;
        while acc >= step && steps < 12 {
            self.tick(world, inputs);
            acc -= step;
            steps += 1;
        }
        self.accum = acc.min(step * 12.0);
    }

    fn tick(&mut self, world: &World, i: &RainInputs) {
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
            let rate = i.day_rate.max(1) as f32;
            let ramp = RAMP_PER_TICK * rate;
            let delta = (self.target - self.intensity).clamp(-ramp, ramp);
            self.intensity += delta;
        }

        // `cloudBGAlpha` ramps toward the storm strength (kept at 10× the
        // `cloudAlpha` ramp so the deck appears within it; scaled by
        // `max(1, dayRate)` like the `cloudBGActive` ramp it stands in for,
        // `Main.cs:13187/13205`). Runs under the pin too, as before.
        let rate = i.day_rate.max(1) as f32;
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

        // `Main.cs:17220-17223`: `if (FocusHelper.AllowRain && cloudAlpha > 0f)
        // Rain.MakeRain();` — the spawn runs BEFORE the drops' `Update` (which
        // the C# runs inside `DrawRain`, `Main.cs:38819-38822`). The port keeps
        // both on this 60 Hz accumulator, spawn-then-update per tick.
        if self.intensity > 0.0 {
            self.spawn(world, i);
        }
        self.update_drops(world, i);
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

    /// `Rain.Update` (`Rain.cs:96-133`) over the fixed pool, in slot order —
    /// the C# runs it inside `DrawRain` per active slot (`Main.cs:38819-38822`);
    /// the port's 60 Hz accumulator owns the cadence (the draw stays pure).
    /// `gamePaused` is folded out (the weather cluster is pause-gated);
    /// `gameMenu` never holds in-world. The death probes are the real
    /// `Collision.SolidCollision`/`WetCollision` 2x2 boxes (`:117`); the
    /// `:120` impact roll fires for ALL THREE `:117` kills (below-screen
    /// included) and NEVER for the remix kill-only arm (`:110-116`).
    fn update_drops(&mut self, world: &World, i: &RainInputs) {
        // `:112/:117`: `screenPosition.Y + screenHeight + 100f` — the port's
        // camera is the view CENTRE; the world-space view bottom plus the
        // 100f margin as RAW world px (the C# adds it unscaled — g67's shape;
        // the earlier 100/zoom fold was a deviation, re-pinned W4-d130
        // reconcile, docs/research/83).
        let bottom = i.camera.y + i.screen.y * 0.5 / i.zoom + 100.0;
        for slot in 0..MAX_DROPS {
            let mut d = self.drops[slot];
            if !d.active {
                continue;
            }
            // `:102`: `position += velocity`.
            d.pos += d.vel;
            // `:103-116`: the gameMenu (+2000f) / remixWorld (+100f) kill-only
            // arms — `active = false` and NOTHING else: the :120 impact roll
            // sits inside the :117 else-if, so these arms never roll.
            // `gameMenu` never holds in-world (folded); the remix arm stays.
            let die = if i.remix_world {
                d.pos.y > bottom
            } else {
                // `:117`: solid OR bottom OR wet — the C# `||` short-circuits,
                // so the wet probe runs only when dry AND on-screen.
                let solid =
                    crate::player::Player::solid_collision(world, d.pos.x, d.pos.y, 2.0, 2.0);
                let below = d.pos.y > bottom;
                let wet = !solid
                    && !below
                    && crate::player::wet_collision(world, d.pos.x, d.pos.y, 2.0, 2.0);
                if solid || below || wet {
                    // `:119` first: the drop dies BEFORE the roll/spawn.
                    d.active = false;
                    // `:120-131`: ONE combined branch — the gate rolls
                    // `Main.rand.Next(100)` for ALL THREE kills, the
                    // below-screen one included (`:120` sits outside the
                    // NewDust call). The g67 reading is correct (006bc04d /
                    // 40128e5d); the earlier collision-only gate was WRONG
                    // and is fixed here (docs/research/83). The stand-in
                    // splash keeps the roll; g67 owns the real
                    // `Dust.NewDust(..., dustWater())` spawn + AI/draw.
                    if self.range(0, 100) as f32 < i.gfx_quality * 100.0 {
                        self.spawn_splash(d);
                    }
                }
                solid || below || wet
            };
            if die {
                d.active = false;
            }
            self.drops[slot] = d;
        }
    }

    /// `Rain.MakeRain` (`Rain.cs:30-89`) + `Rain.NewRain` (`:166-207`),
    /// verbatim: the attempt rate `screenWidth / MaxWorldViewSize.X * 25 *
    /// (0.25 + cloudAlpha)` with the `NPC.AnyDanger` 0.05 throttle, the
    /// player-velocity spawn window, the wind lead (`windSpeedCurrent * 15 *
    /// 40`) and the open-sky gate; per attempt `NewRain` budgets the FIRST
    /// inactive slot within `maxRain * cloudAlpha` scaled by gfxQuality, the
    /// snow-tile suppression `(1 - (snow/threshold)^2)` and `atmo^9`.
    /// RNG order per attempt: x-window roll, y roll, then (on an open tile)
    /// the scale roll and the frame roll — exactly `Rain.cs:54-55` + `:202-205`.
    fn spawn(&mut self, world: &World, i: &RainInputs) {
        // `:32`: netMode 2 / gamePaused / gameMenu fold out (the pause gate
        // owns the cluster); ZoneRain + shimmer gate verbatim.
        if !i.zone_rain || i.shimmer_alpha > 0.0 {
            return;
        }
        // `:36`: `screenWidth / MaxWorldViewSize.X * 25f * (0.25f + cloudAlpha)`
        // — MaxWorldViewSize.X = 1920 (`Main.cs:66714`).
        let mut num = i.screen.x / 1920.0 * 25.0 * (0.25 + 1.0 * self.intensity);
        // `:37-45`: the AnyDanger 0.05 cut with the >1 → <1 floor.
        if i.any_danger {
            let num2 = num;
            num *= 0.05;
            if num2 > 1.0 && num < 1.0 {
                num = 1.0;
            }
        }
        // `Main.screenPosition` — the view's world-px top-left.
        let view_min = i.camera - i.screen * 0.5 / i.zoom;
        let mut attempt = 0.0_f32;
        while attempt < num {
            attempt += 1.0;
            // `:49-53`: the window widens while the player rises.
            let mut num3 = 600;
            if i.player_velocity.y < 0.0 {
                num3 += (i.player_velocity.y.abs() * 30.0) as i32;
            }
            // `:54-55`: the two window rolls (X then Y — the C# order).
            let mut x = self.range_signed(
                view_min.x as i32 - num3,
                view_min.x as i32 + i.screen.x as i32 + num3,
            ) as f32;
            let y = view_min.y - self.range(20, 100) as f32;
            // `:56-57`: the wind lead and the player-velocity lead.
            x -= i.wind_speed_current * 15.0 * 40.0;
            x += i.player_velocity.x * 40.0;
            // `:58-65`: the world-edge clamps.
            if x < 0.0 {
                x = 0.0;
            }
            let max_x = ((world.width - 1) * 16) as f32;
            if x > max_x {
                x = max_x;
            }
            // `:66-83`: the tile indices, clamped to the world.
            let num4 = ((x as i32) / 16).clamp(0, world.width - 1);
            let num5 = ((y as i32) / 16).clamp(0, world.height - 1);
            // `:84`: remixWorld spawns anywhere; else open sky only —
            // `!SolidTile && wall <= 0`.
            let open = i.remix_world || {
                let t = world.tile(num4, num5);
                let solid = t.is_some_and(|t| {
                    t.active()
                        && !t.inactive()
                        && td::TILE_SOLID.has(t.kind)
                        && !td::TILE_SOLID_TOP.has(t.kind)
                        && !t.half_brick()
                        && t.slope() == 0
                });
                !solid && t.is_some_and(|t| t.wall == 0)
            };
            if !open {
                continue;
            }
            // `:86`: `NewRain(GetRainFallVelocity(), vector)` — the budget
            // rolls no RNG; the scale + frame rolls fire only on a taken slot.
            self.new_rain(Vec2::new(x, y), i);
        }
    }

    /// `Rain.NewRain` (`Rain.cs:166-207`): the slot budget then the drop
    /// fields. `GetRainFallVelocity` (`:91-94`) is `(windSpeedCurrent * 18, 14)`.
    fn new_rain(&mut self, position: Vec2, i: &RainInputs) {
        // `:169-173`: `num2 = (int)(maxRain * cloudAlpha)`, capped at maxRain.
        let mut num2 = (MAX_DROPS as f32 * self.intensity) as i32;
        if num2 > MAX_DROPS as i32 {
            num2 = MAX_DROPS as i32;
        }
        // `:174-175`: the depth factor is COMPUTED AND DISCARDED in the C#
        // (a bare `Math.Max(...)` statement) — no effect, kept as a note.
        // `:176-180`: the gfxQuality scale below 0.9.
        let num4 = (1.0 + i.gfx_quality) / 2.0;
        if (num4 as f64) < 0.9 {
            num2 = (num2 as f32 * num4) as i32;
        }
        // `:181-182`: the snow-tile suppression, then `atmo^9`.
        let snow_threshold = tr_core::scene::thresholds::SNOW_TILE_NORMAL as f32;
        let num5 = (i.snow_tile_count as f32 / snow_threshold).clamp(0.0, 1.0);
        num2 = (((num2 as f32) * (1.0 - num5 * num5)) as i32 as f64
            * (i.atmo as f64).powf(9.0)) as i32;
        // `:183-186`: below 0.4 atmo nothing spawns.
        if (i.atmo as f64) < 0.4 {
            num2 = 0;
        }
        // `:187-198`: the first inactive slot within the budget.
        let mut slot = None;
        for (s, d) in self.drops.iter().enumerate().take(num2.max(0) as usize) {
            if !d.active {
                slot = Some(s);
                break;
            }
        }
        let Some(s) = slot else {
            return;
        };
        // `:199-205`: the drop fields — scale roll then frame roll.
        let scale = 1.0 + self.range_signed(-20, 21) as f32 * 0.01;
        // `:93`: `GetRainFallVelocity` — the LIVE wind, no sine term.
        let vel = Vec2::new(i.wind_speed_current * 18.0, 14.0) * scale;
        let kind = (i.water_style * 3 + self.range(0, 3)) as u8;
        self.drops[s] = Drop {
            active: true,
            pos: position,
            vel,
            scale,
            // `:204`: `atan2(velocity.X, 0f - velocity.Y)`.
            rot: vel.x.atan2(-vel.y),
            kind,
        };
    }

    /// The impact splash stand-in — the port's own approximation of the C#'s
    /// `Dust.NewDust(pos - velocity, 2, 2, dustWater())` puff (`Rain.cs:122-130`:
    /// alpha 38, velocity `*0.1 - vel*0.025`, `Y -= 2`, scale 0.6, noGravity).
    /// The verbatim pool spawn + the splash dust's AI/draw are g67's scope
    /// (weather-rain-splash); this keeps the visible impact until it lands.
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

    /// `Main.DrawRain` (`Main.cs:38800-38826`) — the drop pass, verbatim
    /// colour math: the 2x40 frame at `type * 4`, the light sampled at
    /// `(int)(position + 4) >> 4` per axis (`:38813`), the `* 0.85` tint and
    /// the shimmer `* (1 - shimmerAlpha)` (`:38814-38817`), drawn rotated
    /// about the top-left origin (`:38818`, `origin = Vector2.Zero`).
    /// Drawn in the always-rebuilt world-effect range (screen space, under
    /// the map batches and the interface), so the cached world instance
    /// range (G6) is unaffected by the moving drops. The C#'s in-draw
    /// `rain.Update()` (`:38819-38822`) rides the 60 Hz accumulator instead
    /// (`update_drops`) — the recorded cadence deviation.
    pub fn draw(
        &self,
        r: &mut SpriteRenderer,
        lookups: &assets::Lookups,
        light: &LightMap,
        camera: Vec2,
        zoom: f32,
        screen: Vec2,
        shimmer_alpha: f32,
    ) {
        if self.drop_count() == 0
            && self.splashes.is_empty()
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
        // `:38813`: `Lighting.GetColor((int)(pos.X + 4f) >> 4, ...)`.
        let light_tile = |p: Vec2| {
            light.get(
                ((p.x + 4.0) as i32) >> 4,
                ((p.y + 4.0) as i32) >> 4,
            )
        };
        for d in &self.drops {
            if !d.active {
                continue;
            }
            let Some(uv) = lookups.rain_uv(d.kind) else {
                continue;
            };
            let c = light_tile(d.pos);
            // `:38813-38817`: the XNA byte chain (GetColor → *0.85 → shimmer).
            let tint = rain_tint(c, shimmer_alpha);
            r.push_fx_sprite_rot(
                to_screen(d.pos),
                [DROP_W * d.scale * zoom, DROP_H * d.scale * zoom],
                uv,
                tint,
                d.rot,
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

/// `Main.DrawRain`'s colour chain (`Main.cs:38813-38817`) as XNA bytes:
/// `Lighting.GetColor` truncates `light * 255` per channel and clamps at 255
/// (`Lighting.cs:224-240`, alpha 255), `* 0.85f` byte-casts every channel,
/// then `shimmerAlpha > 0` multiplies by `1 - shimmerAlpha` with another byte
/// cast. Returns the normalised `[f32; 4]` the sprite pipeline consumes.
pub(crate) fn rain_tint(light: [f32; 3], shimmer_alpha: f32) -> [f32; 4] {
    let byte = |v: f32| (v.min(255.0) as i32) as u8;
    let c = [
        byte(light[0] * 255.0),
        byte(light[1] * 255.0),
        byte(light[2] * 255.0),
        255u8,
    ];
    let mut out = [0.0; 4];
    for k in 0..4 {
        let mut b = (c[k] as f32 * 0.85) as i32 as u8;
        if shimmer_alpha > 0.0 {
            b = (b as f32 * (1.0 - shimmer_alpha)) as i32 as u8;
        }
        out[k] = b as f32 / 255.0;
    }
    out
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

    /// The per-tick inputs the tests run under: zoom 1, dry wind, no player,
    /// open ZoneRain, full quality — the `MakeRain` gate passes.
    fn inputs() -> RainInputs {
        RainInputs {
            camera: Vec2::new(32.0 * 16.0, 25.0 * 16.0),
            screen: Vec2::new(1600.0, 200.0),
            zoom: 1.0,
            day_rate: 1,
            wind_speed_current: 0.0,
            player_velocity: Vec2::ZERO,
            shimmer_alpha: 0.0,
            atmo: 1.0,
            snow_tile_count: 0,
            zone_rain: true,
            water_style: 0,
            any_danger: false,
            gfx_quality: 1.0,
            remix_world: false,
        }
    }

    fn run(rain: &mut Rain, w: &World, n: usize) {
        let i = inputs();
        for _ in 0..n {
            rain.update(w, &i, 1.0 / 60.0);
        }
    }

    #[test]
    fn forced_rain_spawns_drops_that_die_on_the_ground() {
        let w = ground();
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        run(&mut rain, &w, 120);
        assert!(rain.drop_count() > 100, "drops {}", rain.drop_count());
        // `ground()` is 64 tall — SolidCollision/WetCollision clamp their
        // scan at `height - 40` (row 24), so the row-30 floor is invisible
        // and the drops die BELOW-SCREEN. The :120 gate rolls for that kill
        // too (the combined :117 branch — W4-d130 reconcile), so at quality
        // 1 the impacts still leave splashes at a steady state.
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

    /// One drop pinned into a fixed slot at an exact position, so the NEXT
    /// `update_drops` step moves it to `pos + vel` and the kill arm is
    /// deterministic. `update_drops` is called directly — no spawn, no ramp.
    fn pin_drop(rain: &mut Rain, slot: usize, pos: Vec2, vel: Vec2) {
        rain.drops[slot] = Drop {
            active: true,
            pos,
            vel,
            scale: 1.0,
            rot: 0.0,
            kind: 0,
        };
    }

    #[test]
    fn the_impact_gate_rolls_for_all_three_kills() {
        // W4-d130 reconcile witness — `Rain.cs:117-131`: the ONE combined
        // branch rolls `Next(100) < gfxQuality*100` for the solid kill, the
        // below-screen kill AND the wet kill. Hand-derived on Rain's stream:
        // each kill consumes exactly ONE gate draw at quality 0 (the gate
        // fails, no splash); g67's 40128e5d reading, re-verified against
        // Rain.cs:96-133 (docs/research/83).
        // (a) below-screen: an all-air world, camera bottom + 100 = 600;
        // the drop steps to y 605 > 600, dry and on-screen-solid-free.
        let w = World::new("air".into(), 64, 64);
        let mut rain = Rain::with_forced(7, Some(0.0), false);
        let mut i = inputs();
        i.gfx_quality = 0.0; // the gate FAILS after its one draw — no splash
        pin_drop(&mut rain, 3, Vec2::new(500.0, 591.0), Vec2::new(0.0, 14.0));
        let mut r = StreamReplay(rain.rng);
        let _gate = r.next(0, 100);
        rain.update_drops(&w, &i);
        assert!(!rain.drops[3].active, "the below-screen kill fired");
        assert_eq!(rain.rng, r.0, "exactly ONE draw: the gate");
        // (b) quality 0 never splashes but STILL rolls — the positive arm:
        // at quality 1 the same below-screen kill splashes (the stand-in
        // consumes its own draws behind the gate).
        let mut zero = Rain::with_forced(7, Some(0.0), false);
        let mut zi = inputs();
        zi.gfx_quality = 0.0;
        pin_drop(&mut zero, 0, Vec2::new(500.0, 591.0), Vec2::new(0.0, 14.0));
        let state = zero.rng;
        zero.update_drops(&w, &zi);
        assert_ne!(zero.rng, state, "the gate rolls even at quality 0");
        assert!(zero.splashes.is_empty(), "quality 0: no splash");
        let mut one = Rain::with_forced(7, Some(0.0), false);
        pin_drop(&mut one, 0, Vec2::new(500.0, 591.0), Vec2::new(0.0, 14.0));
        one.update_drops(&w, &inputs()); // quality 1
        assert!(
            !one.splashes.is_empty(),
            "the below-screen kill SPLASHES at quality 1"
        );
        // (c) the solid kill: a TALL world — the probes clamp their scan at
        // `height - 40` (`Collision.cs:2708`/`:1594`), so the floor must sit
        // UNDER row 88 (128 - 40). Floor row 80 (y 1280); the drop steps to
        // y 1282 inside it, well above the view bottom (1500).
        let mut tall = World::new("rain-tall".into(), 64, 128);
        for x in 0..64 {
            let idx = tall.index(x, 80);
            tall.tiles[idx] = Tile {
                kind: 1,
                ..Default::default()
            };
            tall.tiles[idx].flags |= FLAG_ACTIVE;
        }
        tall.compute_column_surface();
        tall.compute_chunks();
        let mut solid_rain = Rain::with_forced(7, Some(0.0), false);
        let mut si = inputs();
        si.camera = Vec2::new(512.0, 1300.0);
        si.gfx_quality = 0.0;
        pin_drop(&mut solid_rain, 1, Vec2::new(500.0, 1268.0), Vec2::new(0.0, 14.0));
        let mut r = StreamReplay(solid_rain.rng);
        let _gate = r.next(0, 100);
        solid_rain.update_drops(&tall, &si);
        assert!(!solid_rain.drops[1].active, "the solid kill fired");
        assert_eq!(solid_rain.rng, r.0, "solid kill: ONE gate draw");
        assert!(solid_rain.splashes.is_empty(), "quality 0: no splash");
        // (d) the wet kill: the same floor as a full liquid tile — the wet
        // probe's surface is `tileY + (256-liquid)/32*2` (`Collision.cs`
        // arm, `player.rs::wet_collision`); the drop steps into it dry-solid-
        // free and dies with the SAME single gate draw.
        let mut wet_w = World::new("rain-wet".into(), 64, 128);
        for x in 0..64 {
            let idx = wet_w.index(x, 80);
            wet_w.tiles[idx].liquid = 255;
        }
        wet_w.compute_column_surface();
        wet_w.compute_chunks();
        let mut wet_rain = Rain::with_forced(7, Some(0.0), false);
        pin_drop(&mut wet_rain, 2, Vec2::new(500.0, 1268.0), Vec2::new(0.0, 14.0));
        let mut r = StreamReplay(wet_rain.rng);
        let _gate = r.next(0, 100);
        wet_rain.update_drops(&wet_w, &si);
        assert!(!wet_rain.drops[2].active, "the wet kill fired");
        assert_eq!(wet_rain.rng, r.0, "wet kill: ONE gate draw");
        assert!(wet_rain.splashes.is_empty(), "quality 0: no splash");
    }

    #[test]
    fn the_remix_kill_arm_never_rolls() {
        // `Rain.cs:110-116`: the remixWorld arm is `active = false` ONLY —
        // the :120 gate sits inside the :117 else-if, which never runs when
        // remixWorld holds. No draw, no splash (g67 folded this arm out;
        // the d130 core keeps it — docs/research/83).
        let w = World::new("air".into(), 64, 64);
        let mut rain = Rain::with_forced(7, Some(0.0), false);
        let mut i = inputs();
        i.remix_world = true;
        pin_drop(&mut rain, 5, Vec2::new(500.0, 591.0), Vec2::new(0.0, 14.0));
        let state = rain.rng;
        rain.update_drops(&w, &i);
        assert!(!rain.drops[5].active, "the remix bottom kill fired");
        assert_eq!(rain.rng, state, "the remix arm consumes NO roll");
        assert!(rain.splashes.is_empty(), "the remix arm never splashes");
    }

    #[test]
    fn snow_tiles_suppress_the_rain_budget_not_the_event() {
        // `Rain.cs:181-182`: `num2 *= 1 - (SnowTileCount/1500)^2` — at the
        // threshold the budget is 0 and NO drops spawn even with ZoneRain up
        // (vanilla's snowfall is `Main.snowing()`'s dust, `crate::snow`, not
        // a rain-rig mode). Half the threshold keeps 3/4 of the budget.
        let w = ground();
        let mut full = Rain::with_forced(17, Some(1.0), false);
        let mut i = inputs();
        i.snow_tile_count = tr_core::scene::thresholds::SNOW_TILE_NORMAL;
        for _ in 0..120 {
            full.update(&w, &i, 1.0 / 60.0);
        }
        assert_eq!(full.drop_count(), 0, "the snow suppression zeroes the budget");
        assert!(full.is_raining(), "the event itself keeps running");

        let mut half = Rain::with_forced(17, Some(1.0), false);
        i.snow_tile_count = tr_core::scene::thresholds::SNOW_TILE_NORMAL / 2;
        for _ in 0..120 {
            half.update(&w, &i, 1.0 / 60.0);
        }
        assert!(half.drop_count() > 0, "3/4 of the budget still spawns");
    }

    #[test]
    fn the_atmo_gate_and_power_nine_shape_the_budget() {
        // `Rain.cs:182-186`: `num2 = (int)(num2 * atmo^9)`, and `atmo < 0.4`
        // zeroes it outright. At atmo 0.8 the budget is 750 * 0.8^9 ≈ 100.
        let w = ground();
        let mut low = Rain::with_forced(19, Some(1.0), false);
        let mut i = inputs();
        i.atmo = 0.39;
        for _ in 0..60 {
            low.update(&w, &i, 1.0 / 60.0);
        }
        assert_eq!(low.drop_count(), 0, "below 0.4 nothing spawns");
        let mut mid = Rain::with_forced(19, Some(1.0), false);
        i.atmo = 0.8;
        for _ in 0..60 {
            mid.update(&w, &i, 1.0 / 60.0);
        }
        let budget = (750.0_f64 * 0.8_f64.powf(9.0)) as usize;
        assert!(mid.drop_count() > 0);
        assert!(
            mid.drop_count() <= budget,
            "drops {} exceed the atmo^9 budget {}",
            mid.drop_count(),
            budget
        );
    }

    #[test]
    fn the_fall_velocity_reads_the_live_wind() {
        // `Rain.cs:91-94` + `:203-204`: `velocity = (windSpeedCurrent * 18,
        // 14) * scale`, `rotation = atan2(v.X, -v.Y)` — the frozen seed-time
        // wind snapshot + sine term are retired (W4-d130).
        let w = ground();
        let mut rain = Rain::with_forced(23, Some(1.0), false);
        let mut i = inputs();
        i.wind_speed_current = 0.5;
        rain.update(&w, &i, 1.0 / 60.0);
        assert!(rain.drop_count() > 0);
        for d in rain.drops.iter().filter(|d| d.active) {
            assert!(
                (d.vel.x - 0.5 * 18.0 * d.scale).abs() < 1e-4,
                "vx {} != wind*18*scale {}",
                d.vel.x,
                0.5 * 18.0 * d.scale
            );
            assert!((d.vel.y - 14.0 * d.scale).abs() < 1e-4);
            assert!((d.rot - d.vel.x.atan2(-d.vel.y)).abs() < 1e-6);
            assert!((0.8..=1.2).contains(&d.scale), "scale {}", d.scale);
            assert!(d.kind < 3, "waterStyle 0 frames 0..2, got {}", d.kind);
        }
    }

    #[test]
    fn test_DW_d130_make_rain_replay_matches_the_csharp_draw_order() {
        // One pinned tick of `MakeRain`/`NewRain` replayed straight from the
        // C# on the Rain stream: 27 attempts at the 1600-px rate
        // (`1600/1920 * 25 * 1.25 = 26.04`, `i < num` admits i = 0..26), each
        // rolling X then Y (`Rain.cs:54-55`), then — every tile open in the
        // fixture — the scale and frame rolls (`:202-205`). Values AND the
        // exact stream position must match.
        let w = ground();
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        let i = inputs();
        rain.update(&w, &i, 1.0 / 60.0);
        let mut r = StreamReplay::new(7);
        let view_min_x = (i.camera.x - i.screen.x * 0.5) as i32; // -288
        let view_min_y = i.camera.y - i.screen.y * 0.5; // 300
        let mut want = Vec::new();
        let mut attempt = 0.0_f32;
        let num = 1600.0 / 1920.0 * 25.0 * (0.25 + 1.0);
        while attempt < num {
            attempt += 1.0;
            let mut x = (view_min_x - 600
                + (r.next_u64() % ((view_min_x + 1600 + 600) - (view_min_x - 600)) as u64)
                    as i32) as f32;
            let y = view_min_y - (20 + r.next_u64() % 80) as f32;
            x -= 0.0; // wind 0: the `:56-57` leads are zero
            x = x.clamp(0.0, ((w.width - 1) * 16) as f32);
            // Every (x, y) lands in open sky above the row-30 ground.
            let scale = 1.0 + (-20 + (r.next_u64() % 41) as i32) as f32 * 0.01;
            let vel = Vec2::new(0.0, 14.0) * scale;
            let kind = (r.next_u64() % 3) as u8;
            want.push((Vec2::new(x, y), vel, scale, kind));
        }
        assert_eq!(want.len(), 27, "the float attempt loop admits 27");
        assert_eq!(rain.drop_count(), 27);
        // The drops moved one tick (`Update` runs after the spawn): compare
        // against pos + vel, and the slot order is the spawn order.
        let live: Vec<_> = rain.drops.iter().filter(|d| d.active).collect();
        for (d, (pos, vel, scale, kind)) in live.iter().zip(want.iter()) {
            assert_eq!(d.pos, *pos + *vel, "position after one Update tick");
            assert_eq!(d.vel, *vel);
            assert_eq!(d.scale, *scale);
            assert_eq!(d.kind, *kind);
        }
        assert_eq!(rain.rng, r.0, "exact stream position after the tick");
    }

    #[test]
    fn the_any_danger_throttle_cuts_the_attempt_rate() {
        // `Rain.cs:37-45`: `num *= 0.05` under AnyDanger. At the 1600-px
        // storm rate 26.04 → 1.302 → the float loop admits i = 0,1 → 2
        // attempts (vs 27 open). The `num2 > 1 && num < 1 → num = 1` floor
        // is count-invisible (the loop admits i=0 for any num > 0, floored
        // or not) — the observable is the 27 → 2 cut.
        let w = ground();
        let mut open = Rain::with_forced(29, Some(1.0), false);
        let mut i = inputs();
        open.update(&w, &i, 1.0 / 60.0);
        assert_eq!(open.drop_count(), 27, "the open rate");
        let mut rain = Rain::with_forced(29, Some(1.0), false);
        i.any_danger = true;
        rain.update(&w, &i, 1.0 / 60.0);
        let throttled = 1600.0 / 1920.0 * 25.0 * 1.25 * 0.05;
        let mut n = 0;
        let mut k = 0.0_f32;
        while k < throttled {
            k += 1.0;
            n += 1;
        }
        assert_eq!(rain.drop_count(), n, "the throttled attempt count");
        assert_eq!(n, 2);
        assert!(n < 27, "throttled below the open rate");
    }

    #[test]
    fn the_gfx_quality_scale_bites_below_09() {
        // `Rain.cs:176-180`: `num4 = (1 + gfxQuality)/2`; below 0.9 the
        // budget scales by it. At quality 0.5 num4 = 0.75 → 750 * 0.75 = 562.
        // An all-air world keeps the drops alive to the bottom kill, so the
        // steady-state population (~675 at the 27/tick rate) reaches past the
        // scaled budget and the cap is observable.
        let w = World::new("air".into(), 64, 64);
        let mut rain = Rain::with_forced(31, Some(1.0), false);
        let mut i = inputs();
        i.gfx_quality = 0.5;
        for _ in 0..240 {
            rain.update(&w, &i, 1.0 / 60.0);
        }
        assert!(
            rain.drop_count() <= 562,
            "drops {} exceed the 0.75-scaled budget",
            rain.drop_count()
        );
        // Quality 1.0 leaves num4 = 1.0 — the `< 0.9` gate never fires.
        let mut full = Rain::with_forced(31, Some(1.0), false);
        i.gfx_quality = 1.0;
        for _ in 0..240 {
            full.update(&w, &i, 1.0 / 60.0);
        }
        assert!(full.drop_count() > 562, "the full budget fills higher");
    }

    #[test]
    fn the_draw_tint_is_the_xna_byte_chain() {
        // `Main.cs:38813-38817`: GetColor bytes * 0.85 (byte cast), then
        // * (1 - shimmerAlpha) (another byte cast) — two truncations, alpha
        // included (255 * 0.85 = 216).
        let white = [1.0, 1.0, 1.0];
        let t = rain_tint(white, 0.0);
        assert_eq!(t, [216.0 / 255.0; 4]);
        let t = rain_tint(white, 0.5);
        // 216 * 0.5 = 108 exactly.
        assert_eq!(t, [108.0 / 255.0; 4]);
        // A mid light: (int)(0.5*255) = 127; 127*0.85 = 107.95 → 107;
        // shimmer 0.25: 107*0.75 = 80.25 → 80. Alpha 216*0.75 = 162.
        let t = rain_tint([0.5, 0.5, 0.5], 0.25);
        assert_eq!(t[0], 80.0 / 255.0);
        assert_eq!(t[3], 162.0 / 255.0);
        // Overbright light clamps at 255 before the scale (`Lighting.cs:228`).
        let t = rain_tint([1.4, 1.4, 1.4], 0.0);
        assert_eq!(t, [216.0 / 255.0; 4]);
        // Zero shimmer alpha skips the second multiply entirely (`> 0f`).
        assert_eq!(rain_tint(white, 0.0), rain_tint(white, -0.1));
    }

    #[test]
    fn storms_ramp_the_cloud_deck_and_lightning_decays() {
        let w = ground();
        let mut rain = Rain::with_forced(23, Some(1.0), false);
        run(&mut rain, &w, 600);
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
        let mut i = inputs();
        i.day_rate = 60;

        let mut fast = Rain::with_forced(11, None, false);
        fast.raining = true;
        fast.target = 1.0;
        fast.update(&w, &i, step);
        assert!(
            (fast.visual_alpha() - 0.03).abs() < 1e-6,
            "60× ramp: {}",
            fast.visual_alpha()
        );

        let mut slow = Rain::with_forced(11, None, false);
        slow.raining = true;
        slow.target = 1.0;
        i.day_rate = 1;
        slow.update(&w, &i, step);
        assert!(
            (slow.visual_alpha() - 0.0005).abs() < 1e-9,
            "dry ramp: {}",
            slow.visual_alpha()
        );

        let mut deck = Rain::with_forced(11, None, false);
        deck.intensity = 1.0; // above the 0.05 gate: the deck chases the intensity
        i.day_rate = 60;
        deck.update(&w, &i, step);
        assert!(
            (deck.cloud_bg() - 0.3).abs() < 1e-6,
            "deck at FF: {}",
            deck.cloud_bg()
        );
        let mut dry = Rain::with_forced(11, None, false);
        dry.intensity = 1.0;
        i.day_rate = 1;
        dry.update(&w, &i, step);
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
        let mut i = inputs();
        i.camera = Vec2::new(512.0, 400.0);
        i.day_rate = 60;
        rain.update(&w, &i, 1.0 / 60.0);
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
