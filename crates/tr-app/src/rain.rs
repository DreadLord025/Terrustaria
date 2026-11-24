//! Rain: `Rain.MakeRain` / `Rain.Update` / `Main.DrawRain`.
//!
//! Weather follows the original's shape: a target intensity (`Main.maxRaining`) that
//! `cloudAlpha` ramps toward at `0.0005 * max(1, dayRate)` per 60 Hz tick, a countdown
//! that ends the event, and one drop per `MakeRain` spawn slot per tick with the
//! original fall velocity (`wind * 18, 14` px/tick). Drops are drawn rotated around
//! their top-left origin with the local light colour, and die on solid or wet tiles
//! (`Collision.SolidCollision`/`WetCollision`, `Rain.cs:117`) with a splash — the
//! gfx-quality-gated `Dust.NewDust(position - velocity, 2, 2, Dust.dustWater())`
//! recipe (`Rain.cs:120-131`, g67 `40128e5d` transplanted onto this branch's
//! verbatim core, `docs/research/83-weather-reconcile.md` §3) drained into the
//! App's shared `DustPool` (the d50 contract), the water-style splash family
//! (33/98..105/123/288) whose per-tick physics is the `Dust.cs:1834-1874` arm
//! (`water_splash_arm` below). The ambient snowfall is NOT this rig's: vanilla's `Main.snowing()`
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
/// The `cloudBGAlpha` ramp step (`Main.cs:59360/59368`) — the C# moves it by
/// `0.0005f * dayRate` per DRAW inside `DrawSurfaceBG`'s surface gate, toward
/// 1 while `cloudBGActive > 0f` and toward 0 otherwise; the port runs it in
/// [`Rain::cloud_bg_draw_step`] at the draw head (W4-g69 deviation fix — the
/// retired session schedule's 10×-per-tick intensity chase is gone).
const CLOUD_BG_RAMP_PER_DRAW: f32 = 0.0005;
const DROP_W: f32 = 2.0;
const DROP_H: f32 = 40.0;

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
    /// `FreezeRainPower.Enabled` (`CreativePowers.cs:1507-1538`): the world
    /// tick copies the power in per step (`Main.cs:64866`/`:64893` read the
    /// manager live; the port snapshots like the other cross-system inputs).
    freeze_rain: bool,
    /// `Main.ladyBugRainBoost` (`Main.cs:64833-64836`): the ladybug-death rain
    /// window, drained by `dayRate` per tick and gating the third start arm
    /// (`Main.cs:64920`). The setter side (M4 ladybug deaths) is not ported — the
    /// field is inert until then.
    lady_bug_rain_boost: i32,
    /// TR_NO_WEATHER: the port's scheduler switch — with it the C#-shaped rolls
    /// never run (the retired session schedule honoured the same switch).
    calm: bool,
    /// `Main.cloudBGAlpha`: storm cloud-bank opacity, 0..1 - ramped per draw
    /// by [Rain::cloud_bg_draw_step] (`Main.cs:59358-59373`, W4-g69).
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
    /// `WorldGen.Skyblock.lowTiles` — the `SnowTileThreshold` skyblock arm
    /// (`SceneMetrics.cs:154-163`: 300 low-tiles / 1500 normal). The port
    /// gathers `World.skyblock_low_tiles` here (g67 `f8c4b659` fold,
    /// research/134 R5 — adapted to the `RainInputs` seam).
    pub skyblock_low_tiles: bool,
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
    /// `Main.gameMenu` (`Rain.cs:84/103-109`, g67b `d2dd8989` fold): the
    /// `Rain.Update` kill-only arm (+2000, no gate roll, no splash) and the
    /// `MakeRain` spawn-through-solids disjunct. The port's frame gates the
    /// weather cluster out of menus, so production passes the front-state
    /// flag for shape — residual drops never tick there (g67's note).
    pub game_menu: bool,
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
            freeze_rain: false,
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

    /// `Main.cloudAlpha` RAW - the cloud spawn/kill gates read the unclamped
    /// intensity (`Cloud.cs:116`, `Main.cs:58980-59001`; W4-g69); the
    /// snowfall's intensity/budget reads it too (`Main.cs:13044-13046`).
    pub fn raw_alpha(&self) -> f32 {
        self.intensity
    }

    /// The d130-name alias of [Self::raw_alpha] (batch-K compose: the
    /// snow/scene feeds keep the d130 accessor name).
    pub fn cloud_alpha(&self) -> f32 {
        self.intensity
    /// `Main.lightning` (`Main.cs:1207`): the flash brightness the sky tint's
    /// lerp-to-white reads (`SetBackColor`, `:63905-63916`). W4-g70 lane 8.
    pub fn flash(&self) -> f32 {
        self.flash
    }

    /// `Main.raining`: the toggle the tile-light scanner reads for the water
    /// fountain (`tile.frameX == 90`) case.
    pub fn is_raining(&self) -> bool {
        self.raining
    }

    /// `Main.IsItRaining` (`Main.cs:2597`: `cloudAlpha > 0f`) — g67b
    /// `112d1b20`'s census fix: the ambience scheduler's sunny/calm gates
    /// (`AmbienceServer.cs:32/:50`), the ambient-sky entity aborts
    /// (`AmbientSky.cs:216` passim), the music rain branches
    /// (`Main.cs:12859/:12880/:12932/:12973`), the town quiet-sky gate
    /// (`:12670/:12674`) and the storm-lightning relaxation
    /// (`WorldGen.cs:71871`) all read the CLOUD, not the event flag — a
    /// post-`StopRain` cloud ramp still counts as raining for them.
    pub fn is_it_raining(&self) -> bool {
        self.intensity > 0.0
    }

    /// `Rain.ClearRain` (`Rain.cs:22-28`, g67b `5c9398ea` fold): deactivate
    /// every drop. The C# calls it from `SaveAndQuitCallBack`
    /// (`WorldGen.cs:6672`) and `genWorldCallBack` (`:6747`); the port's
    /// save-and-quit funnel calls it, and a freshly generated world boots a
    /// fresh `Rain` (empty drops) so the gen-side call is a no-op there.
    pub(crate) fn clear_rain(&mut self) {
        for d in &mut self.drops {
            d.active = false;
        }
    }

    /// `Main.rainTime`: ticks left in the event (`Main.cs:64874` drains it).
    pub(crate) fn rain_time(&self) -> i32 {
        self.rain_time
    }

    /// `Main.cloudAlpha` — the RAW visible intensity (the rain slider's
    /// `GetSliderValueInner` reads it verbatim, `CreativePowers.cs:1332-1335`;
    /// [`Self::visual_alpha`] is the sky-tint clamp, a different consumer).
    pub fn cloud_alpha(&self) -> f32 {
        self.intensity
    }

    /// The world tick's per-step copy of `FreezeRainPower.Enabled`
    /// (`Main.cs:64866`/`:64893` read the manager live).
    pub(crate) fn set_freeze_rain(&mut self, frozen: bool) {
        self.freeze_rain = frozen;
    }

    /// `ModifyRainPower.UpdateInfoFromSliderValueCache`
    /// (`CreativePowers.cs:1320-1330`): cache 0 → `StopRain(instant: true)`,
    /// else `StartRain(instant: true, strengthOverride: cache)`. The full
    /// `StartRain` ladder still rolls (`Main.cs:64676-64723`) — the override
    /// only replaces `ChangeRain`'s strength draw; `instant` snaps
    /// `cloudAlpha := maxRaining` (`:64736-64739`).
    pub(crate) fn set_rain_power(&mut self, cache: f32, world: &World, remix_world: bool, player_luck: f32) {
        if cache == 0.0 {
            self.stop_rain_instant(); // `:1324`
        } else {
            self.start_rain_with(world, remix_world, player_luck, true, Some(cache)); // `:1328`
        }
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

    /// A test hook (not a behaviour): the Rain stream's raw state, for the
    /// W4-f39 gate witnesses' draw-count pins (the stream is private).
    #[cfg(test)]
    pub(crate) fn rng_state_for_test(&self) -> u64 {
        self.rng
    }

    /// A test hook (not a behaviour): arms a running rain event for the
    /// W4-f39 ladder/lantern witnesses (`Main.raining`/`rainTime`/
    /// `maxRaining` are private; the C# state is set by `StartRain`).
    #[cfg(test)]
    pub(crate) fn arm_raining_for_test(&mut self, rain_time: i32, target: f32) {
        self.raining = true;
        self.rain_time = rain_time;
        self.target = target;
    }

    /// A test hook (not a behaviour): `Main.rainTime` for the W4-f39
    /// lantern-stop witness (the field is private).
    #[cfg(test)]
    pub(crate) fn rain_time_for_test(&self) -> i32 {
        self.rain_time
    }

    /// Live drops (`Rain.active` slots) - the debug readout.
    pub fn drop_count(&self) -> usize {
        self.drops.iter().filter(|d| d.active).count()
    }

    /// Storm cloud-bank opacity (`Main.cloudBGAlpha`).
    pub fn cloud_bg(&self) -> f32 {
        self.cloud_bg
    }

    /// `DrawSurfaceBG`'s `cloudBGAlpha` ramp (`Main.cs:59358-59373`), verbatim:
    /// `cloudBGActive > 0f` → `+= 0.0005f * dayRate` capped at 1, else
    /// `-= 0.0005f * dayRate` floored at 0. Runs once per DRAW at the draw
    /// head (`app/frame/draw.rs`), inside the C#'s own gate chain —
    /// `!drunkWorldGen && ShouldDrawSurfaceBackground() && screenPosition.Y/16
    /// <= worldSurface + 10` (`:59354`) AND `BackgroundEnabled` (`:59356`);
    /// the caller evaluates the gates. `dayRate` is the RAW value (no
    /// `max(1, …)` — the C# multiplies the unclamped static, so dayRate 0
    /// freezes the deck). W4-g69 deviation fix: the retired session schedule
    /// chased the rain intensity at 10× per tick instead.
    pub(crate) fn cloud_bg_draw_step(&mut self, cloud_bg_active: f32, day_rate: i32) {
        // `Main.cs:59358-59365`.
        if cloud_bg_active > 0.0 {
            self.cloud_bg += CLOUD_BG_RAMP_PER_DRAW * day_rate as f32;
            if self.cloud_bg > 1.0 {
                self.cloud_bg = 1.0;
            }
        } else {
            // `Main.cs:59366-59373`.
            self.cloud_bg -= CLOUD_BG_RAMP_PER_DRAW * day_rate as f32;
            if self.cloud_bg < 0.0 {
                self.cloud_bg = 0.0;
            }
        }
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
    /// `pool` is the splash drain seam (g67 `40128e5d`, transplanted per
    /// `docs/research/83-weather-reconcile.md` §3): `Some((DustPool, SpawnCtx))`
    /// from the world pass (the d50 contract — collision splashes spawn
    /// through `DustPool::new_dust` on Rain's own stream); `None` keeps the
    /// gate rolling with the spawn dropped (the drop-sim tests).
    pub fn update(
        &mut self,
        world: &World,
        inputs: &RainInputs,
        dt: f32,
        mut pool: Option<(
            &mut tr_core::particles::DustPool,
            &tr_core::particles::SpawnCtx,
        )>,
    ) {
        let step = 1.0 / TICKS_PER_SEC;
        let mut acc = self.accum + dt.min(0.25);
        let mut steps = 0;
        while acc >= step && steps < 12 {
            self.tick(
                world,
                inputs,
                pool.as_mut().map(|(p, c)| (&mut **p, &**c)),
            );
            acc -= step;
            steps += 1;
        }
        self.accum = acc.min(step * 12.0);
    }

    fn tick(
        &mut self,
        world: &World,
        i: &RainInputs,
        pool: Option<(&mut tr_core::particles::DustPool, &tr_core::particles::SpawnCtx)>,
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
            // scaled by `max(1, dayRate)` (`:61902-61906`). The `gameMenu` ×20
            // arm (`:61907-61910`) folds — the port's rain rig never ticks in
            // the menus (the front end runs `update_menu_music`, not this).
            let rate = i.day_rate.max(1) as f32;
            let ramp = RAMP_PER_TICK * rate;
            // `Main.cs:61911-61937` — the TWO-arm shape (W4-d130 final):
            // while raining, `cloudAlpha` chases `maxRaining` in BOTH
            // directions with the overshoot snap; while NOT raining it
            // decays toward 0 UNCONDITIONALLY — `maxRaining` is not read
            // (the earlier single clamp-toward-target was observation-equal
            // only because `StopRain` zeroes the target; the C# arm is
            // independent of it).
            if self.raining {
                // `:61913-61928`.
                if self.intensity > self.target {
                    self.intensity -= ramp;
                    if self.intensity < self.target {
                        self.intensity = self.target;
                    }
                } else if self.intensity < self.target {
                    self.intensity += ramp;
                    if self.intensity > self.target {
                        self.intensity = self.target;
                    }
                }
            } else {
                // `:61930-61937`: the dry decay, floor 0.
                self.intensity -= ramp;
                if self.intensity < 0.0 {
                    self.intensity = 0.0;
                }
            }
        }

        // `cloudBGAlpha` no longer ramps here: the C# walks it per DRAW inside
        // `DrawSurfaceBG` (`Main.cs:59358-59373`), ported as
        // `cloud_bg_draw_step` at the draw head (W4-g69 deviation fix).

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
        self.update_drops(world, i, pool);
        // The splash particles are `DustPool` slots now (g67's lane,
        // transplanted) — the shared `update_dust` pass ticks them
        // (`water_splash_arm`), nothing to advance here.
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
    /// C# order: the lanterns-up storm-deck head clamp (`:64825-64831`,
    /// W4-c39 graft), the `ladyBugRainBoost` drain (`:64833-64836`), the outer
    /// `(netMode != 1 && !gameMenu) && (isThereAWorldSurface || remixWorld)` gate
    /// (`:64846` — netMode 0 and an in-game tick make the first conjunct true),
    /// the raining branch (`:64864-64888`), then the not-raining start arms
    /// (`:64890-64923`). The slime-rain block after them (`:64925-64949`) is
    /// the weather tick's (W4-f39). `FreezeRainPower` does not exist in the
    /// port — its gates resolve to the C# shape with false. There is no
    /// `Main.dayTime` gate on the start arms (only the slime roll is day-gated).
    ///
    /// W4-f39 (folded, final phase): `slime_rain`/`lanterns_up`/
    /// `next_night_lanterns` are `Main.slimeRain` / `LanternNight.LanternsUp` /
    /// `LanternNight.NextNightIsLanternNight` sampled at the branch entry —
    /// the `:64868-64871` lanterns-up `StopRain` arm inside the raining
    /// branch, and the `:64890` `!slimeRain && !LanternsUp &&
    /// !NextNightIsLanternNight` gate on the start arms.
    ///
    /// `cloud_bg_active`/`num_clouds` are the live `CloudField` statics
    /// (`Main.cloudBGActive`/`Main.numClouds`) the ×4.25 gate and `ChangeRain`'s
    /// band ladder read (W4-g69 deviation fix — the stand-ins are retired).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn update_time(
        &mut self,
        day_rate: i32,
        world: &mut World,
        any_player_120: bool,
        remix_world: bool,
        player_luck: f32,
        slime_rain: bool,
        lanterns_up: bool,
        next_night_lanterns: bool,
        cloud_bg_active: f32,
        num_clouds: i32,
    ) {
        // TR_RAIN pin / TR_NO_WEATHER: port-only scheduler switches — the pin
        // overrides the state machine exactly as the retired session schedule was
        // skipped under it, and `start_calm` disabled that schedule too.
        if self.forced.is_some() || self.calm {
            return;
        }
        // W4-c39 (graft, research/114 §13): `Main.cs:64825-64831` —
        // `UpdateTime`'s head, BEFORE the ladybug drain: while the lanterns
        // are up `cloudBGActive = 0f` and `numClouds` caps at 30. The port's
        // `numClouds` stand-in is pinned at 100 (`NUM_CLOUDS_STAND_IN`) and
        // both `> 150`/`> 100` band arms are dead at 100 AND at 30, so the
        // cap changes no outcome (recorded); the storm-deck clear is
        // observable — the `:64916` second start arm reads `cloud_bg`.
        if lanterns_up {
            self.cloud_bg = 0.0;
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
            // `Main.cs:64864-64888` — the drain and the strength-change roll,
            // the whole block behind `!FreezeRainPower.Enabled` (`:64866`).
            if !self.freeze_rain {
                self.rain_time -= day_rate; // `:64874`
                if day_rate > 0 {
                    // `:64875` — always true today (`day_rate()` is 60|1); the gate is
                    // the C# shape.
            // W4-f39: `Main.cs:64868-64871` — a lantern night ends the rain
            // at once (no drain, no strength roll); the else arm is the
            // `rainTime` drain + the strength-change roll (`:64872-64887`).
            if lanterns_up {
                self.stop_rain(); // `:64870`
            } else {
                // `Main.cs:64874` — the drain.
                self.rain_time -= day_rate;
                if day_rate > 0 {
                    // `:64875` — always true today (`day_rate()` is 60|1); the
                    // gate is the C# shape.
                    let num = 86400 / day_rate / 24; // `:64877`
                    if self.rain_time <= 0 {
                        self.stop_rain(); // `:64880`
                    } else if self.range(0, (num * 2) as u32) == 0 {
                        self.change_rain(); // `:64882-64885`
                        self.change_rain(cloud_bg_active, num_clouds); // `:64882-64885`
                    }
            // `Main.cs:64864-64888` — the drain and the strength-change roll.
            self.rain_time -= day_rate; // `:64874`
            if day_rate > 0 {
                // `:64875` — always true today (`day_rate()` is 60|1); the gate is
                // the C# shape.
                let num = 86400 / day_rate / 24; // `:64877`
                if self.rain_time <= 0 {
                    self.stop_rain(world); // `:64880`
                } else if self.range(0, (num * 2) as u32) == 0 {
                    self.change_rain(); // `:64882-64885`
                }
            }
        } else if !slime_rain && !lanterns_up && !next_night_lanterns {
            // `Main.cs:64890`: `!Main.slimeRain && !LanternNight.LanternsUp &&
            // !LanternNight.NextNightIsLanternNight` (W4-f39 — the live event
            // flags, sampled at the branch entry by the caller).
            let num2 = 86400 / if day_rate == 0 { 1 } else { day_rate }; // `:64892`
            // `:64893`: `!FreezeRainPower.Enabled && Main.dayRate != 0`.
            let mut flag = !self.freeze_rain && day_rate != 0;
            if flag && !any_player_120 {
                // `:64894-64909`: the 255-player `statLifeMax >= 120` scan; the
                // port has one player and the tick computes the bool.
                flag = false;
            }
            if flag {
                // The start arms in C# order — each gate short-circuits before
                // its roll, exactly like the `&&` chains.
                if self.range(0, ((num2 as f64) * 5.75) as u32) == 0 {
                    self.start_rain(world, remix_world, player_luck, cloud_bg_active, num_clouds); // `:64912-64914`
                } else if cloud_bg_active >= 1.0
                    && self.range(0, ((num2 as f64) * 4.25) as u32) == 0
                {
                    // `:64916-64918`: `cloudBGActive >= 1f` — the live
                    // overcast state machine's value (`CloudField`, W4-g69).
                    self.start_rain(world, remix_world, player_luck, cloud_bg_active, num_clouds);
                } else if self.lady_bug_rain_boost > 0 && self.range(0, num2 as u32) == 0 {
                    self.start_rain(world, remix_world, player_luck, cloud_bg_active, num_clouds); // `:64920-64922`
                }
            }
        }
    }

    /// `Main.StartRain` (`Main.cs:64659-64727`). The `instant`/`strengthOverride`
    /// params default to the C# values (the scheduler's callers never pass
    /// them; the rain power's slider does — `CreativePowers.cs:1328`) and
    /// `IsRainingForever` has no save field — both cited inline. `player_luck`
    /// is `GetPlayerWithHighestLuck().luck` (`Player.cs:18213-18229` — the
    /// port's single player; a fresh `Player()` with luck 0 when none).
    pub(crate) fn start_rain(&mut self, world: &World, remix_world: bool, player_luck: f32) {
        self.start_rain_with(world, remix_world, player_luck, false, None);
    }

    /// The parameterised `StartRain(instant, strengthOverride)` — the rain
    /// power's `UpdateInfoFromSliderValueCache` arm (`CreativePowers.cs:1328`).
    pub(crate) fn start_rain_with(
    /// `cloud_bg_active`/`num_clouds` feed the `ChangeRain` tail (the live
    /// `Main.cloudBGActive`/`Main.numClouds` statics, W4-g69).
    pub(crate) fn start_rain(
        &mut self,
        world: &World,
        remix_world: bool,
        player_luck: f32,
        instant: bool,
        strength_override: Option<f32>,
        cloud_bg_active: f32,
        num_clouds: i32,
    ) {
    pub(crate) fn start_rain(&mut self, world: &mut World, remix_world: bool, player_luck: f32) {
        // `Main.cs:64661`.
        if !remix_world && world.world_surface > 50.0 && !self.raining {
            // `Main.cs:64663-64667`: `range = 25`; the tenth-anniversary
            // world (not getGood) narrows it to 5 (W4-f39 Phase B compose —
            // the port HAS the anniversary seed now). The roll's DRAW COUNT
            // is unchanged at luck 0 (one `Next(range)`); only the
            // denominator moves.
            let anniversary = world.seeds.contains(tr_core::seeds::SeedFlags::TENTH_ANNIVERSARY)
                && !world.seeds.contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY);
            let range = if anniversary { 5 } else { 25 };
            // `Main.cs:64668`: `GetPlayerWithHighestLuck().RollLuck(range)` —
            // consumed on the Rain stream like every rain roll (WEATHER-2); a
            // hit branch draws 2 more, exactly like the C# branch shape.
            let hit = {
                let mut loot_rng = RainLootRng(self);
                tr_core::loot::roll_luck(&mut loot_rng, player_luck, range) == 0
            };
            if hit {
                // `Main.cs:64670-64673`: the coin-rain value — W4-f45 stores it
                // into `World.coin_rain` (the `.wld` flags block carries it,
                // `:1423`); the coin projectiles themselves are unported
                // (M2/M4), so the value round-trips without a spawn.
                let num = world.width as f32 / 4200.0; // `Main.maxTilesX`
                let coin_rain = self.range(75, 151) as i32 * 100 * 100;
                world.coin_rain = (coin_rain as f32 * num) as i32;
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
        self.change_rain_with(instant, strength_override); // `:64724`
        self.change_rain(cloud_bg_active, num_clouds); // `:64724` — `ChangeRain(instant, strengthOverride)` defaults
        self.raining = true; // `:64725`
    }

    /// `Main.ChangeRain` (`Main.cs:64728-64740`). The `instant`/`strengthOverride`
    /// params default to the C# values (the scheduler's callers never pass
    /// them); `numClouds` reads the static cloud-field stand-in and
    /// `cloudBGActive >= 1f` the `cloudBGAlpha` stand-in — the live cloud
    /// system (`Main.cs:58952-59036`) is unported.
    fn change_rain(&mut self) {
        self.change_rain_with(false, None);
    }

    /// The parameterised `ChangeRain(instant, strengthOverride)` — the rain
    /// power's slider passes its cache as the override (`CreativePowers.cs:1328`
    /// → `Main.cs:64730`'s `strengthOverride.HasValue` arm, which consumes NO
    /// strength draws).
    fn change_rain_with(&mut self, instant: bool, strength_override: Option<f32>) {
        let val = if let Some(v) = strength_override {
            v // `:64730`: the override replaces the whole band ladder
        } else if self.cloud_bg >= 1.0 || NUM_CLOUDS_STAND_IN > 150 {
    /// params default to the C# values (the port's callers never pass them).
    /// `cloud_bg_active` is `Main.cloudBGActive` and `num_clouds` is
    /// `Main.numClouds` — the LIVE `CloudField` statics (W4-g69 deviation fix:
    /// the band ladder read the pinned `NUM_CLOUDS_STAND_IN`/`cloud_bg`
    /// stand-ins until the pool went live). The compares are the C#'s
    /// `(double)numClouds > 150.0` / `> 100.0`.
    fn change_rain(&mut self, cloud_bg_active: f32, num_clouds: i32) {
        let val = if cloud_bg_active >= 1.0 || (num_clouds as f64) > 150.0 {
            // `Main.cs:64730` band 1: `Next(40,91)` / `Next(20,91)` ×0.01.
            if self.range(0, 3) != 0 {
                self.range(40, 91) as f32 * 0.01
            } else {
                self.range(20, 91) as f32 * 0.01
            }
        } else if (num_clouds as f64) > 100.0 {
            // Band 2: `Next(20,61)` / `Next(10,71)` ×0.01 — live now that
            // numClouds walks past 100 on overcast/rain days.
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
        // `:64736-64739`: the `instant` arm snaps `cloudAlpha := maxRaining`
        // (the rain power's slider passes it; the scheduler's callers use the
        // default false and the visual ramps own `cloudAlpha`).
        if instant {
            self.intensity = self.target;
        }
    }

    /// `Main.StopRain` (`Main.cs:64644-64658`): the event clears and `cloudAlpha`
    /// decays through the visual ramp (`Main.cs:61928-61937`'s not-raining arm —
    /// `target` 0). The `instant` default false is the scheduler's arm;
    /// `IsRainingForever` has no save field.
    pub(crate) fn stop_rain(&mut self) {
        self.stop_rain_with(false);
    }

    /// `StopRain(instant: true)` — the rain power's cache-0 arm
    /// (`CreativePowers.cs:1324`): `cloudAlpha := maxRaining` (0) at once.
    pub(crate) fn stop_rain_instant(&mut self) {
        self.stop_rain_with(true);
    }

    fn stop_rain_with(&mut self, instant: bool) {
        self.rain_time = 0; // `:64648`
        self.raining = false; // `:64649`
        self.target = 0.0; // `:64650` `Main.maxRaining = 0f`
        // `:64651` `Main.coinRain = 0` — never stored (see `start_rain`).
        // `:64652-64655`: the `instant` arm snaps `cloudAlpha` to `maxRaining`.
        if instant {
            self.intensity = self.target;
        }
    pub(crate) fn stop_rain(&mut self, world: &mut World) {
        self.rain_time = 0; // `:64648`
        self.raining = false; // `:64649`
        self.target = 0.0; // `:64650` `Main.maxRaining = 0f`
        world.coin_rain = 0; // `:64651` `Main.coinRain = 0` (W4-f45)
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
    ///
    /// `pool` is the splash drain seam (g67): the world tick hands it the
    /// App's `DustPool` + `SpawnCtx` (the d50 contract — the spawn rides
    /// `DustPool::new_dust` and Rain's own stream); the drop-sim tests pass
    /// `None` (no pool — the gate still rolls, the spawn is dropped).
    fn update_drops(
        &mut self,
        world: &World,
        i: &RainInputs,
        mut pool: Option<(&mut tr_core::particles::DustPool, &tr_core::particles::SpawnCtx)>,
    ) {
        // `:112/:117`: `screenPosition.Y + screenHeight + 100f` — the port's
        // camera is the view CENTRE; the world-space view bottom plus the
        // 100f margin as RAW world px (the C# adds it unscaled — g67's shape;
        // the earlier 100/zoom fold was a deviation, re-pinned W4-d130
        // reconcile, docs/research/83).
        let view_bottom = i.camera.y + i.screen.y * 0.5 / i.zoom;
        let bottom = view_bottom + 100.0;
        for slot in 0..MAX_DROPS {
            let mut d = self.drops[slot];
            if !d.active {
                continue;
            }
            // `:102`: `position += velocity` — runs under EVERY arm, the
            // menu/remix kill-only ones included.
            d.pos += d.vel;
            // `:103-116`: the gameMenu (+2000f) / remixWorld (+100f) kill-only
            // arms — `active = false` and NOTHING else: the :120 impact roll
            // sits inside the :117 else-if, so these arms never roll and
            // never probe collisions. The menu arm is g67b `d2dd8989`'s
            // (the earlier "gameMenu never holds in-world" fold dropped it).
            let die = if i.game_menu {
                d.pos.y > view_bottom + 2000.0
            } else if i.remix_world {
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
                    // and is fixed here (docs/research/83). The gate rolls
                    // even when the dust would cull or no pool is handed;
                    // the spawn is g67's real DustPool recipe (§3 union).
                    if self.range(0, 100) as f32 < i.gfx_quality * 100.0 {
                        if let Some((p, ctx)) = pool.as_mut() {
                            self.spawn_splash(p, ctx, world, i.camera, d.pos, d.vel);
                        }
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
            // `:84`: remixWorld OR gameMenu spawns anywhere (the menu
            // disjunct is g67b `5c9398ea`'s); else open sky only —
            // `!SolidTile && wall <= 0`.
            let open = i.remix_world || i.game_menu || {
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
        // `SceneMetrics.SnowTileThreshold` is skyblock-aware (300 low-tiles /
        // 1500 normal, `SceneMetrics.cs:154-163`); the flag rides
        // `RainInputs.skyblock_low_tiles` ← `World.skyblock_low_tiles`
        // (g67 `f8c4b659` fold, research/134 R5 — the census
        // `World::scan_skyblock_low_tiles` fills it at load/gen).
        let snow_threshold = tr_core::scene::snow_tile_threshold(i.skyblock_low_tiles) as f32;
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

    /// `Rain.NewRainForced` (`Rain.cs:135-164`, g67b `5c9398ea` fold): the
    /// Eyebrella vanity drip (`Player.DoEyebrellaRainEffect`,
    /// `Player.cs:14308-14331` — the `UpdateEquips` arm `:13083-13087`,
    /// g67 `356a1ac0` fold, drained in `world_sim/player.rs`). Unlike
    /// `NewRain` it ignores cloudAlpha/snow/atmo: the budget is `maxRain` cut
    /// by the gfx `(1+q)/2` when q<0.9 (`:139-143`), then the first inactive
    /// slot takes the spawn. Returns the slot index, or `maxRain` when full
    /// (`:152-155`). RNG order: scale roll then frame roll (`:159-162`).
    pub(crate) fn new_rain_forced(&mut self, i: &RainInputs, position: Vec2, velocity: Vec2) -> usize {
        let mut num2 = MAX_DROPS as i32; // `:138`
        let num3 = (1.0 + i.gfx_quality) / 2.0; // `:139`
        if num3 < 0.9 {
            num2 = (num2 as f32 * num3) as i32; // `:140-143`
        }
        // `:144-155`: the first inactive slot under the budget (the fixed
        // 750-slot pool keeps its holes — the same scan `new_rain` runs).
        let mut slot = None;
        for (s, d) in self.drops.iter().enumerate().take(num2.max(0) as usize) {
            if !d.active {
                slot = Some(s);
                break;
            }
        }
        let Some(s) = slot else {
            return MAX_DROPS;
        };
        // `:159-162`: scale, velocity, rotation, frame type.
        let scale = 1.0 + self.range_signed(-20, 21) as f32 * 0.01;
        let vel = velocity * scale;
        let kind = (i.water_style * 3 + self.range(0, 3)) as u8;
        self.drops[s] = Drop {
            active: true,
            pos: position,
            vel,
            scale,
            rot: vel.x.atan2(-vel.y),
            kind,
        };
        s
    }

    /// The `Rain.Update` splash recipe (`Rain.cs:122-130`, g67 `40128e5d`
    /// transplanted per docs/research/83 §3): one
    /// `Dust.NewDust(position - velocity, 2, 2, Dust.dustWater())` through the
    /// shared pool (the d50 contract — its rolls ride Rain's own stream via
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
        // The splash dust draws through the pool's own `DrawDust` entry
        // (`draw_splashes`, the g67 seam) — the drops/flash still gate
        // this pass.
        if self.drop_count() == 0 && self.flash <= 0.01 {
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
        // `:38813`: `Lighting.GetColor((int)(pos.X + 4f) >> 4, ...)` — the
        // tile containing `pos + 4`, NOT the dust convention's bare `pos/16`
        // (g67d `7aaf66f0`'s census re-pin: the two differ in each cell's
        // 12..15 px band).
        let light_tile = |p: Vec2| {
            let (lx, ly) = rain_light_tile(p);
            light.get(lx, ly)
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

/// `Main.DrawRain`'s light-sample tile (`Main.cs:38813`):
/// `Lighting.GetColor((int)(rain.position.X + 4f) >> 4, (int)(…Y + 4f) >> 4)` —
/// the tile containing `pos + 4`, arithmetic-shifted. Differs from the dust
/// convention (`:38968`'s `(pos+4)/16`, [`light_at`]) in the 12..15 px band of
/// each cell, where the `+4` crosses the tile edge. (g67d `7aaf66f0`'s
/// `rain_light_tile`, extracted so the draw closure and the witness share it —
/// the d130 retire compose; the semantics were already in the inline closure.)
pub(crate) fn rain_light_tile(p: Vec2) -> (i32, i32) {
    (((p.x + 4.0) as i32) >> 4, ((p.y + 4.0) as i32) >> 4)
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

/// The splash-dust family — every id `Dust.dustWater()` (`Dust.cs:402-418`)
/// can return: the water-style-0 default 33 and the per-style rows
/// 98..=105 / 123 / 288. The shared `UpdateDust` hook ([`water_splash_arm`])
/// and the `DrawDust` entry ([`draw_splashes`]) filter on it.
pub(crate) fn is_splash_type(t: i32) -> bool {
    matches!(t, 33 | 98..=105 | 123 | 288)
}

/// `Dust.UpdateDust`'s splash-family arm (`Dust.cs:1834-1874`, g67 `40128e5d`
/// transplanted per docs/research/83 §3): the
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
    if !matches!(d.dust_type, 308 | 33 | 52 | 266 | 98..=105 | 123 | 288) {
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

/// `Main.DrawDust`'s generic path (`Main.cs:38836-39018`, g67 `40128e5d`
/// transplanted) over the pool's splash-family slots (`dustWater()`,
/// [`is_splash_type`]): the light sample at `(position + 4) / 16`
/// (`:38968`), the `GetAlpha` tint (`:38973`, the default formula for the
/// whole family — `dust_table`), the `(4, 4)` origin rotated draw at
/// `GetVisualScale` (`:39005`), the `GetColor` overlay when the spawn colour
/// is non-zero (`:39006-39012`, never for rain — `default(Color)`), and the
/// full-dark kill (`:39015`). The pool's `maxDustToDraw` scan bound is the
/// update loop's own kill (`Dust.cs:2393-2395`); this entry walks the live
/// family slots.
///
/// The 4×4 on-screen cull rect (`:38832`/`:38847`) is ported verbatim
/// (g67b `2c452eddb` fold — the earlier "documented widening" was WRONG):
/// `Main.DrawDust`'s live rect is the ±1000/±1050 slack box
/// (`screenPosition + (-1000, -1050, screenWidth+2000, screenHeight+2100)`
/// — `:38832` overwrites `:38831`'s ±500/±50 box; the trail types'
/// `rectangle = rectangle2` restore at `:38845` is a no-op because both
/// hold the same wide rect). A dust whose 4×4 box misses it is DEACTIVATED
/// (`:39020-39022`'s else arm), not merely skipped. The rect is world-px
/// (no zoom term), mapped through the camera-centred view's top-left.
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
        let size = [
            d.frame_w as f32 * scale * zoom,
            d.frame_h as f32 * scale * zoom,
        ];
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

/// `Main.DrawDust`'s on-screen cull rect (`Main.cs:38832`, g67b `2c452eddb`
/// fold): the ±1000/±1050 slack box off `Main.screenPosition` —
/// `(screenX-1000, screenY-1050, screenWidth+2000, screenHeight+2100)`.
/// `:38831`'s ±500/±50 box is overwritten before the loop; the trail
/// types' `:38845` restore copies the SAME wide rect back, so one rect
/// covers the whole family. World px, no zoom term. Returns `(x, y, w, h)`.
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
            skyblock_low_tiles: false,
            zone_rain: true,
            water_style: 0,
            any_danger: false,
            gfx_quality: 1.0,
            remix_world: false,
            game_menu: false,
        }
    }

    fn run(rain: &mut Rain, w: &World, n: usize) {
        let i = inputs();
        for _ in 0..n {
            rain.update(w, &i, 1.0 / 60.0, None);
        }
    }

    /// The pool + ctx the splash drain needs — the camera-centred view the
    /// `run` helper uses, full dust budget (g67's `splash_ctx`, adapted).
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
        let mut pool = tr_core::particles::DustPool::new();
        let ctx = splash_ctx();
        let i = inputs();
        for _ in 0..120 {
            rain.update(&w, &i, 1.0 / 60.0, Some((&mut pool, &ctx)));
        }
        assert!(rain.drop_count() > 100, "drops {}", rain.drop_count());
        // `ground()` is 64 tall — SolidCollision/WetCollision clamp their
        // scan at `height - 40` (row 24), so the row-30 floor is invisible
        // and the drops die BELOW-SCREEN. The :120 gate rolls for that kill
        // too (the combined :117 branch — W4-d130 reconcile), so at quality
        // 1 the impacts still leave splash dust at a steady state — the
        // spawn's own off-screen cull (margin 400 at dCount 0) keeps the
        // just-below-view dust (`Rain.cs:122`'s `dustWater()` = 33 here).
        assert!(splash_count(&pool) > 0, "impacts should leave splashes");
        assert!(
            pool.slots()[..tr_core::particles::DUST_SENTINEL]
                .iter()
                .any(|d| d.active && d.dust_type == 33),
            "the water-style-0 splash type"
        );
    }


    /// W4-g73 retire phase (re-grafted at batch-K onto d130's RainInputs
    /// shape) — `Rain.NewRain` reads the STORED `Main.waterStyle`
    /// (`Rain.cs:205`: `type = waterStyle*3 + Next(3)`), not a spawn-time
    /// camera re-derivation: the fountain head arm's colour
    /// (`Main.cs:57390-57393`, latched by `update_rain` into
    /// `RainInputs.water_style`) reaches the drop frame types. The fixture
    /// camera derives OCEAN (13) from the 64-wide world's edge rule — a
    /// flowing underground-desert fountain (colour 12) must override it:
    /// every drop kind lands in 36..=38, never 39..=41.
    #[test]
    fn the_stored_fountain_water_style_feeds_the_drop_frame_types() {
        let w = ground();
        let camera = Vec2::new(32.0 * 16.0, 25.0 * 16.0);
        assert_eq!(crate::biome::calculate_water_style(&w, camera), 13);
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        // The `update_rain` latch with a flowing colour-12 fountain.
        let style =
            crate::biome::calculate_water_style_with_fountain(&w, camera, 12) as u32;
        assert_eq!(style, 12);
        let i = RainInputs { water_style: style, ..inputs() };
        for _ in 0..120 {
            rain.update(&w, &i, 1.0 / 60.0, None);
        }
        assert!(
            rain.drops.iter().all(|d| (36..=38).contains(&d.kind)),
            "every drop rides the fountain style 12, not the biome's 13"
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
        rain.update_drops(&w, &i, None);
        assert!(!rain.drops[3].active, "the below-screen kill fired");
        assert_eq!(rain.rng, r.0, "exactly ONE draw: the gate");
        // (b) quality 0 never splashes but STILL rolls — the positive arm:
        // at quality 1 the same below-screen kill splashes (the real
        // DustPool recipe consumes its six default-block draws behind the
        // gate — g67's lane, transplanted).
        let mut zero = Rain::with_forced(7, Some(0.0), false);
        let mut zi = inputs();
        zi.gfx_quality = 0.0;
        pin_drop(&mut zero, 0, Vec2::new(500.0, 591.0), Vec2::new(0.0, 14.0));
        let mut pool = tr_core::particles::DustPool::new();
        let ctx = splash_ctx();
        let state = zero.rng;
        zero.update_drops(&w, &zi, Some((&mut pool, &ctx)));
        assert_ne!(zero.rng, state, "the gate rolls even at quality 0");
        assert_eq!(splash_count(&pool), 0, "quality 0: no splash");
        let mut one = Rain::with_forced(7, Some(0.0), false);
        pin_drop(&mut one, 0, Vec2::new(500.0, 591.0), Vec2::new(0.0, 14.0));
        one.update_drops(&w, &inputs(), Some((&mut pool, &ctx))); // quality 1
        assert!(
            splash_count(&pool) > 0,
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
        solid_rain.update_drops(&tall, &si, None);
        assert!(!solid_rain.drops[1].active, "the solid kill fired");
        assert_eq!(solid_rain.rng, r.0, "solid kill: ONE gate draw");
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
        wet_rain.update_drops(&wet_w, &si, None);
        assert!(!wet_rain.drops[2].active, "the wet kill fired");
        assert_eq!(wet_rain.rng, r.0, "wet kill: ONE gate draw");
        // The quality-0 no-splash arms: one pool shared across (b)'s zero
        // run and these — `None` above spawns nothing, the pool still holds
        // only (b)'s single quality-1 dust, never a quality-0 one.
        assert_eq!(splash_count(&pool), 1, "only the quality-1 kill spawned");
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
        let mut pool = tr_core::particles::DustPool::new();
        let ctx = splash_ctx();
        let state = rain.rng;
        rain.update_drops(&w, &i, Some((&mut pool, &ctx)));
        assert!(!rain.drops[5].active, "the remix bottom kill fired");
        assert_eq!(rain.rng, state, "the remix arm consumes NO roll");
        assert_eq!(splash_count(&pool), 0, "the remix arm never splashes");
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
            full.update(&w, &i, 1.0 / 60.0, None);
        }
        assert_eq!(full.drop_count(), 0, "the snow suppression zeroes the budget");
        assert!(full.is_raining(), "the event itself keeps running");

        let mut half = Rain::with_forced(17, Some(1.0), false);
        i.snow_tile_count = tr_core::scene::thresholds::SNOW_TILE_NORMAL / 2;
        for _ in 0..120 {
            half.update(&w, &i, 1.0 / 60.0, None);
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
            low.update(&w, &i, 1.0 / 60.0, None);
        }
        assert_eq!(low.drop_count(), 0, "below 0.4 nothing spawns");
        let mut mid = Rain::with_forced(19, Some(1.0), false);
        i.atmo = 0.8;
        for _ in 0..60 {
            mid.update(&w, &i, 1.0 / 60.0, None);
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
        rain.update(&w, &i, 1.0 / 60.0, None);
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
        rain.update(&w, &i, 1.0 / 60.0, None);
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
        open.update(&w, &i, 1.0 / 60.0, None);
        assert_eq!(open.drop_count(), 27, "the open rate");
        let mut rain = Rain::with_forced(29, Some(1.0), false);
        i.any_danger = true;
        rain.update(&w, &i, 1.0 / 60.0, None);
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
            rain.update(&w, &i, 1.0 / 60.0, None);
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
            full.update(&w, &i, 1.0 / 60.0, None);
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
        // The deck ramps per DRAW now (`Main.cs:59358-59373` via the
        // draw head, W4-g69) - drive the step alongside the drop-sim tick
        // with the overcast state active, exactly like the wired frame.
        for _ in 0..600 {
            run(&mut rain, &w, 1);
            rain.cloud_bg_draw_step(1.0, 1);
        }
        // 600 draws x 0.0005 = 0.3 at dayRate 1 (the C# rate).
        assert!(
            (rain.cloud_bg() - 0.3).abs() < 1e-4,
            "cloud deck {}",
            rain.cloud_bg()
        );
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
        ending.update_time(1, &tall, true, false, 0.0, false, false, false, 0.0, 100);
        ending.update_time(1, &mut tall, true, false, 0.0);
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

    /// W4-d130 final — the ramp's TWO arms (`Main.cs:61911-61937`): the
    /// raining arm chases `maxRaining` in BOTH directions with the
    /// overshoot snap; the NOT-raining arm decays toward 0 ignoring
    /// `maxRaining` entirely.
    #[test]
    fn the_ramp_arms_chase_while_raining_and_decay_to_zero_when_dry() {
        let w = ground();
        let i = inputs(); // dayRate 1 → the 0.0005 step
        let step = 1.0 / 60.0;
        // The rain-start ramp: 0 → target 0.8, one step per tick.
        let mut start = Rain::with_forced(11, None, false);
        start.raining = true;
        start.target = 0.8;
        start.update(&w, &i, step, None);
        assert!((start.intensity - 0.0005).abs() < 1e-9);
        // The ChangeRain DOWN-shift while still raining (`:61913-61919`):
        // intensity 0.8, new target 0.3 — the ramp walks DOWN and snaps
        // at the target (no overshoot below it).
        let mut down = Rain::with_forced(11, None, false);
        down.raining = true;
        down.target = 0.3;
        down.intensity = 0.3002;
        down.update(&w, &i, step, None);
        assert_eq!(down.intensity, 0.3, "the overshoot snap (`:61916-61919`)");
        // The up-shift snap (`:61924-61927`).
        let mut up = Rain::with_forced(11, None, false);
        up.raining = true;
        up.target = 0.3;
        up.intensity = 0.2998;
        up.update(&w, &i, step, None);
        assert_eq!(up.intensity, 0.3, "the up overshoot snap");
        // The rain-STOP decay (`:61930-61937`): NOT raining with a stale
        // NON-ZERO maxRaining — the C# ignores the target and decays to 0.
        // (StopRain zeroes the target in the live flow; this pins the arm
        // itself, the shape the earlier clamp-toward-target lacked.)
        let mut dry = Rain::with_forced(11, None, false);
        dry.raining = false;
        dry.target = 0.8; // stale — the dry arm must NOT chase it
        dry.intensity = 0.001;
        dry.update(&w, &i, step, None);
        assert!((dry.intensity - 0.0005).abs() < 1e-9, "decays DOWN, not up");
        dry.update(&w, &i, step, None);
        assert_eq!(dry.intensity, 0.0, "the 0 floor (`:61933-61936`)");
        // The floor holds — no negative drift on further dry ticks.
        dry.update(&w, &i, step, None);
        assert_eq!(dry.intensity, 0.0);
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

    /// W4-d130 final — the sandstorm interplay's downstream half
    /// (`SceneMetrics.cs:698-703` → `Rain.cs:32`): a happening desert
    /// sandstorm clears `ZoneRain`, so `MakeRain`'s early-out stops the
    /// drop spawns — while `Main.raining` itself stays up (the event
    /// keeps draining through the storm) and the intensity keeps ramping.
    /// The zone gate is ONE-WAY: the rain never blocks the sandstorm.
    #[test]
    fn a_suppressed_rain_zone_spawns_no_drops_but_the_event_lives() {
        let w = ground();
        let mut i = inputs();
        i.zone_rain = false; // the sandstorm suppression's output
        let mut rain = Rain::with_forced(5, None, false);
        rain.raining = true;
        rain.target = 1.0;
        rain.rain_time = 50_000;
        for _ in 0..120 {
            rain.update(&w, &i, 1.0 / 60.0, None);
        }
        assert_eq!(rain.drop_count(), 0, "the !ZoneRain early-out (`Rain.cs:32`)");
        assert!(rain.is_raining(), "the event flag survives the storm zone");
        assert!(rain.cloud_alpha() > 0.0, "the intensity ramp is zone-blind");
        // The zone returns (the storm ended) — the same event resumes
        // spawning without a restart.
        i.zone_rain = true;
        for _ in 0..120 {
            rain.update(&w, &i, 1.0 / 60.0, None);
        }
        assert!(rain.drop_count() > 0, "the rain resumes when the zone returns");
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
    /// `ChangeRain`'s lowest band (`:64730` — the replay's callers pass
    /// numClouds 100 / cloudBGActive 0, so `numClouds > 150`/`> 100` are
    /// false). Returns the
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
        let mut w = rain_world();
        for seed in [42u64, 1337, 90_210, 271_828] {
            let mut rain = Rain::with_forced(seed, None, false);
            rain.start_rain(&w, false, 0.0, 0.0, 100);
            rain.start_rain(&mut w, false, 0.0);
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
    fn test_DW_2_1_change_rain_band_gates_pick_by_live_cloud_statics() {
        // `Main.cs:64730`: the bands gate on `cloudBGActive >= 1f ||
        // (double)numClouds > 150.0`, then `(double)numClouds > 100.0` —
        // W4-g69: both inputs are the LIVE CloudField statics now, so all
        // three bands are reachable (the stand-in kept the middle dead).
        // Each case replays its band on an independent stream: values AND
        // exact stream position.
        for seed in [5u64, 77, 1234] {
            // Band 3: clear sky (active 0, numClouds 100 — not > 100).
            let mut calm = Rain::with_forced(seed, None, false);
            calm.change_rain(0.0, 100);
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
            assert_eq!(calm.rng, r.0, "seed {seed}: band-3 stream position");

            // Band 2: numClouds 101..150 with no overcast (101 > 100, not
            // > 150) — live only since the count walk feeds the ladder.
            let mut mid = Rain::with_forced(seed, None, false);
            mid.change_rain(0.0, 120);
            assert!(
                (0.10..0.71).contains(&mid.target),
                "middle band, seed {seed}: {}",
                mid.target
            );
            let mut r = StreamReplay::new(seed);
            let want_mid = if r.next(0, 3) != 0 {
                r.next(20, 61) as f32 * 0.01
            } else {
                r.next(10, 71) as f32 * 0.01
            };
            assert_eq!(mid.target, want_mid, "seed {seed}");
            assert_eq!(mid.rng, r.0, "seed {seed}: band-2 stream position");

            // Band 1 via the count arm: numClouds > 150 alone (active 0).
            let mut heavy = Rain::with_forced(seed, None, false);
            heavy.change_rain(0.0, 151);
            let mut r = StreamReplay::new(seed);
            let want_storm = if r.next(0, 3) != 0 {
                r.next(40, 91) as f32 * 0.01
            } else {
                r.next(20, 91) as f32 * 0.01
            };
            assert_eq!(heavy.target, want_storm, "seed {seed}: count arm");
            assert!(
                (0.20..0.91).contains(&heavy.target),
                "storm band, seed {seed}: {}",
                heavy.target
            );

            // Band 1 via the overcast arm: cloudBGActive >= 1 with a LOW
            // count — the C# `||` shape (the old stand-in read cloudBGAlpha).
            let mut storm = Rain::with_forced(seed, None, false);
            storm.change_rain(1.0, 0);
            assert_eq!(storm.target, want_storm, "seed {seed}: overcast arm");
            assert_eq!(storm.rng, r.0, "seed {seed}: band-1 stream position");
        }
        // The exact edges: 100 is band 3, 150 is band 2 (the double
        // compares are strict `>`).
        let mut edge = Rain::with_forced(5, None, false);
        edge.change_rain(0.0, 150);
        let mut r = StreamReplay::new(5);
        let want_mid = if r.next(0, 3) != 0 {
            r.next(20, 61) as f32 * 0.01
        } else {
            r.next(10, 71) as f32 * 0.01
        };
        assert_eq!(edge.target, want_mid, "150 stays in band 2");
    }

    #[test]
    fn test_DW_2_1_stop_rain_clears_the_event_state() {
        let mut w = rain_world();
        let mut rain = Rain::with_forced(7, None, false);
        rain.raining = true;
        rain.target = 0.8;
        rain.rain_time = 5_000;
        rain.stop_rain(&mut w);
        assert!(!rain.is_raining());
        assert_eq!(rain.max_raining(), 0.0);
        assert_eq!(rain.rain_time, 0);
        // `Main.cs:64652-64655`: without `instant` the visible `cloudAlpha`
        // decays through the ramp — `StopRain` leaves the field alone.
        rain.intensity = 0.5;
        rain.stop_rain(&mut w);
        assert_eq!(rain.intensity, 0.5, "the ramp owns the decay, not StopRain");
    }

    #[test]
    fn test_DW_2_2_start_denominator_8280_at_day_rate_60_vs_496800_at_1() {
        // `Main.cs:64892` + `:64912`: `num2 = 86400 / max(1, dayRate)`; the
        // first start arm rolls `Next((int)(num2 * 5.75))` — 1440·5.75 = 8280
        // at dayRate 60, 86400·5.75 = 496800 at dayRate 1. A first draw
        // ≡ 0 mod 8280 but not mod 496800 starts the rain only at dayRate 60.
        let mut w = rain_world();
        let fire_at_60 = (1..2_000_000u64)
            .find(|&s| {
                let d = StreamReplay::new(s).next_u64();
                d % 8280 == 0 && d % 496_800 != 0
            })
            .expect("a 8280-only seed exists");
        let mut fast = Rain::with_forced(fire_at_60, None, false);
        fast.update_time(60, &w, true, false, 0.0, false, false, false, 0.0, 100);
        assert!(fast.is_raining(), "the ×5.75 roll hit at dayRate 60");

        let mut slow = Rain::with_forced(fire_at_60, None, false);
        slow.update_time(1, &w, true, false, 0.0, false, false, false, 0.0, 100);
        fast.update_time(60, &mut w, true, false, 0.0);
        assert!(fast.is_raining(), "the ×5.75 roll hit at dayRate 60");

        let mut slow = Rain::with_forced(fire_at_60, None, false);
        slow.update_time(1, &mut w, true, false, 0.0);
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
        both_fast.update_time(60, &w, true, false, 0.0, false, false, false, 0.0, 100);
        assert!(both_fast.is_raining());
        let mut both_slow = Rain::with_forced(fire_at_both, None, false);
        both_slow.update_time(1, &w, true, false, 0.0, false, false, false, 0.0, 100);
        both_fast.update_time(60, &mut w, true, false, 0.0);
        assert!(both_fast.is_raining());
        let mut both_slow = Rain::with_forced(fire_at_both, None, false);
        both_slow.update_time(1, &mut w, true, false, 0.0);
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
        let mut w = rain_world();
        let mut rain = Rain::with_forced(7, None, false);
        rain.raining = true;
        rain.target = 0.8;
        rain.rain_time = 1;
        let state = rain.rng;
        rain.update_time(1, &w, true, false, 0.0, false, false, false, 0.0, 100);
        rain.update_time(1, &mut w, true, false, 0.0);
        assert!(!rain.is_raining());
        assert_eq!(rain.max_raining(), 0.0);
        assert_eq!(rain.rain_time, 0, "StopRain pins rainTime to 0 (`:64648`)");
        assert_eq!(rain.rng, state, "no roll is consumed on the stop arm");

        // One tick above the boundary survives and rolls (`rainTime` 2 → 1).
        let mut alive = Rain::with_forced(7, None, false);
        alive.raining = true;
        alive.target = 0.8;
        alive.rain_time = 2;
        alive.update_time(1, &w, true, false, 0.0, false, false, false, 0.0, 100);
        alive.update_time(1, &mut w, true, false, 0.0);
        assert!(alive.is_raining(), "rainTime 1 is still above the boundary");
        assert_eq!(alive.rain_time, 1);
    }

    /// W4-f39 (folded, final phase) — the `:64868-64871` lanterns-up arm:
    /// a lantern night STOPS the rain at once — no `rainTime` drain, no
    /// strength-change roll, the stream untouched.
    #[test]
    fn the_lanterns_up_arm_stops_the_rain_without_draining_or_rolling() {
        let w = rain_world();
        let mut rain = Rain::with_forced(7, None, false);
        rain.raining = true;
        rain.target = 0.8;
        rain.rain_time = 50_000;
        let state = rain.rng;
        rain.update_time(1, &w, true, false, 0.0, false, true, false, 0.0, 100);
        assert!(!rain.is_raining(), "LanternsUp → StopRain (`:64870`)");
        assert_eq!(rain.max_raining(), 0.0);
        assert_eq!(rain.rain_time, 0, "StopRain pins rainTime, not the drain");
        assert_eq!(rain.rng, state, "no strength roll on the lantern arm");
    }

    /// W4-c39 (graft, research/114 §13) — the `:64825-64831` head clamp:
    /// `UpdateTime`'s head clears `cloudBGActive` while the lanterns are up
    /// — BEFORE the ladybug drain and the rain branches, draw-free. The
    /// `numClouds` cap at 30 is outcome-neutral (the 100 stand-in's
    /// `> 150`/`> 100` arms are dead at 100 AND at 30 — recorded at the
    /// clamp). c39's witness body ports unchanged (same rig names).
    #[test]
    fn w4_c39_the_lantern_head_clamps_the_storm_deck_before_the_drain() {
        let w = rain_world();
        let mut rain = Rain::with_forced(7, None, false);
        rain.cloud_bg = 0.8;
        rain.lady_bug_rain_boost = 100;
        let state = rain.rng;
        rain.update_time(1, &w, true, false, 0.0, false, true, false, 0.0, 100);
        assert_eq!(rain.cloud_bg(), 0.0, "the lantern head clears the deck");
        assert_eq!(rain.lady_bug_rain_boost, 99, "the drain still runs");
        assert_eq!(rain.rng, state, "the clamp draws nothing");
        // The contrast arm: no lanterns, the deck keeps its value.
        let mut plain = Rain::with_forced(7, None, false);
        plain.cloud_bg = 0.8;
        plain.update_time(1, &w, true, false, 0.0, false, false, false, 0.0, 100);
        assert_eq!(plain.cloud_bg(), 0.8);
    }

    /// W4-f39 (folded, final phase) — the `:64890` gate on the start arms:
    /// `!slimeRain && !LanternsUp && !NextNightIsLanternNight`. Each flag
    /// closes the arms BEFORE any roll — a fire-seeded stream stays put.
    #[test]
    fn the_64890_gate_flags_close_the_start_arms_before_any_roll() {
        let w = rain_world();
        let fire = (1..8_000_000u64)
            .find(|&s| StreamReplay::new(s).next_u64() % 496_800 == 0)
            .expect("a 496800 seed exists");
        // The control: all flags down, the same seed starts the rain.
        let mut open = Rain::with_forced(fire, None, false);
        open.update_time(1, &w, true, false, 0.0, false, false, false, 0.0, 100);
        assert!(open.is_raining(), "the control fires");
        // Each gate flag alone closes the arms, draw-free.
        for (slime, lantern, next) in [(true, false, false), (false, true, false), (false, false, true)]
        {
            let mut gated = Rain::with_forced(fire, None, false);
            let state = gated.rng;
            gated.update_time(1, &w, true, false, 0.0, slime, lantern, next, 0.0, 100);
            assert!(!gated.is_raining(), "the gate ({slime},{lantern},{next}) closes");
            assert_eq!(gated.rng, state, "no roll under the gate");
        }
    }

    #[test]
    fn test_update_time_night_start_roll_fires_without_a_day_time_gate() {
        // `Main.cs:64912-64923`: the start arms carry no `Main.dayTime` gate —
        // only the slime-rain roll at `:64925` is day-gated, and that block is
        // M4. A hit on the ×5.75 bound starts rain regardless of time of day.
        let mut w = rain_world();
        let fire = (1..8_000_000u64)
            .find(|&s| StreamReplay::new(s).next_u64() % 496_800 == 0)
            .expect("a 496800 seed exists");
        let mut rain = Rain::with_forced(fire, None, false);
        rain.update_time(1, &w, true, false, 0.0, false, false, false, 0.0, 100);
        rain.update_time(1, &mut w, true, false, 0.0);
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
        let mut w = rain_world();
        let mut drain = Rain::with_forced(7, None, false);
        drain.lady_bug_rain_boost = 120;
        drain.update_time(60, &w, false, false, 0.0, false, false, false, 0.0, 100); // no 120-HP player: arms gated
        assert_eq!(drain.lady_bug_rain_boost, 60);
        drain.update_time(60, &w, false, false, 0.0, false, false, false, 0.0, 100);
        drain.update_time(60, &mut w, false, false, 0.0); // no 120-HP player: arms gated
        assert_eq!(drain.lady_bug_rain_boost, 60);
        drain.update_time(60, &mut w, false, false, 0.0);
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
        rain.update_time(60, &w, true, false, 0.0, false, false, false, 0.0, 100);
        assert!(rain.is_raining(), "the ladybug arm fired");

        let mut quiet = Rain::with_forced(fire, None, false);
        quiet.update_time(60, &w, true, false, 0.0, false, false, false, 0.0, 100);
        rain.update_time(60, &mut w, true, false, 0.0);
        assert!(rain.is_raining(), "the ladybug arm fired");

        let mut quiet = Rain::with_forced(fire, None, false);
        quiet.update_time(60, &mut w, true, false, 0.0);
        assert!(
            !quiet.is_raining(),
            "boost 0 gates the third arm before its roll"
        );
    }

    #[test]
    fn test_DW_2_3_cloud_alpha_ramp_moves_60x_at_day_rate_60() {
        // `Main.cs:61902-61924`: `num5 = 0.0005 * max(1, dayRate)` per step
        // toward `maxRaining` — one visual tick at dayRate 60 moves 60× the
        // dry constant. The storm deck's own ramp is per-draw now
        // (`cloud_bg_draw_step`, `Main.cs:59358-59373`) — witnessed below.
        let w = ground();
        let step = 1.0 / 60.0;
        let mut i = inputs();
        i.day_rate = 60;

        let mut fast = Rain::with_forced(11, None, false);
        fast.raining = true;
        fast.target = 1.0;
        fast.update(&w, &i, step, None);
        assert!(
            (fast.visual_alpha() - 0.03).abs() < 1e-6,
            "60× ramp: {}",
            fast.visual_alpha()
        );

        let mut slow = Rain::with_forced(11, None, false);
        slow.raining = true;
        slow.target = 1.0;
        i.day_rate = 1;
        slow.update(&w, &i, step, None);
        assert!(
            (slow.visual_alpha() - 0.0005).abs() < 1e-9,
            "dry ramp: {}",
            slow.visual_alpha()
        );

        // The storm deck (`cloudBGAlpha`) no longer rides the drop-sim tick:
        // `cloud_bg_draw_step` runs per DRAW at the C# rate —
        // `0.0005 * dayRate` RAW (`Main.cs:59360/59368`), toward 1 while
        // cloudBGActive > 0 and toward 0 otherwise (W4-g69 deviation fix).
        let mut deck = Rain::with_forced(11, None, false);
        i.day_rate = 60;
        deck.update(&w, &i, step, None);
        assert_eq!(deck.cloud_bg(), 0.0, "the tick no longer moves the deck");
        deck.cloud_bg_draw_step(1.0, 60);
        assert!(
            (deck.cloud_bg() - 0.03).abs() < 1e-6,
            "deck at FF: {}",
            deck.cloud_bg()
        );
        let mut dry = Rain::with_forced(11, None, false);
        dry.cloud_bg_draw_step(1.0, 1);
        assert!(
            (dry.cloud_bg() - 0.0005).abs() < 1e-9,
            "deck dry: {}",
            dry.cloud_bg()
        );
    }

    #[test]
    fn w4_g69_cloud_bg_draw_step_replays_the_csharp_ramp() {
        // `Main.cs:59358-59373` verbatim: active > 0 climbs 0.0005*dayRate
        // capped at 1; active <= 0 falls the same step floored at 0; the
        // RAW dayRate 0 freezes it (no max(1, …) — the C# multiplies the
        // unclamped static).
        let mut r = Rain::with_forced(11, None, false);
        // Climb: 100 draws at dayRate 1 = 0.05.
        for _ in 0..100 {
            r.cloud_bg_draw_step(0.5, 1);
        }
        assert!((r.cloud_bg() - 0.05).abs() < 1e-6, "{}", r.cloud_bg());
        // The cap at 1 (2100 draws × 0.0005 = 1.05 -> clamped).
        for _ in 0..2100 {
            r.cloud_bg_draw_step(1.0, 1);
        }
        assert_eq!(r.cloud_bg(), 1.0, "the ramp caps at 1");
        // The fall arm: active <= 0 (the cooldown climb is negative).
        for _ in 0..100 {
            r.cloud_bg_draw_step(-500.0, 1);
        }
        assert!((r.cloud_bg() - 0.95).abs() < 1e-5, "{}", r.cloud_bg());
        // The floor at 0.
        for _ in 0..2100 {
            r.cloud_bg_draw_step(0.0, 1);
        }
        assert_eq!(r.cloud_bg(), 0.0, "the ramp floors at 0");
        // dayRate 0 freezes both arms (the C# multiplies the raw static).
        r.cloud_bg_draw_step(1.0, 0);
        assert_eq!(r.cloud_bg(), 0.0);
        r.cloud_bg = 0.5;
        r.cloud_bg_draw_step(-1.0, 0);
        assert_eq!(r.cloud_bg(), 0.5);
        // The boundary: active exactly 0 takes the FALL arm (`> 0f` strict).
        r.cloud_bg_draw_step(0.0, 60);
        assert!((r.cloud_bg() - 0.47).abs() < 1e-6, "{}", r.cloud_bg());
    }

    #[test]
    fn test_DW_2_4_forced_pin_precedence_over_update_time() {
        // The TR_RAIN pin is the scheduler: `update_time` never runs under it
        // (the retired session schedule skipped the same way) — a dead
        // `rainTime` cannot stop a pinned storm and no roll consumes the
        // stream. TR_NO_WEATHER (`calm`) disables the rolls the same way.
        let mut w = rain_world();
        let mut rain = Rain::with_forced(7, Some(0.8), false);
        rain.rain_time = 0;
        rain.lady_bug_rain_boost = 100;
        let state = rain.rng;
        rain.update_time(60, &w, true, false, 0.0, false, false, false, 0.0, 100);
        rain.update_time(60, &mut w, true, false, 0.0);
        assert_eq!(rain.rain_time, 0, "no drain under the pin");
        assert_eq!(rain.lady_bug_rain_boost, 100, "no drain under the pin");
        assert_eq!(rain.rng, state, "no rolls under the pin");
        // The pin still owns the visual state through the drop-sim tick.
        let mut i = inputs();
        i.camera = Vec2::new(512.0, 400.0);
        i.day_rate = 60;
        rain.update(&w, &i, 1.0 / 60.0, None);
        assert_eq!(rain.max_raining(), 0.8);
        assert!(rain.is_raining());

        let mut quiet = Rain::with_forced(9, None, true);
        quiet.lady_bug_rain_boost = 100;
        let state = quiet.rng;
        quiet.update_time(60, &w, true, false, 0.0, false, false, false, 0.0, 100);
        quiet.update_time(60, &mut w, true, false, 0.0);
        assert_eq!(
            quiet.lady_bug_rain_boost, 100,
            "no drain under TR_NO_WEATHER"
        );
        assert_eq!(quiet.rng, state);
    }

    /// W4-f39 Phase B compose (d130 retire) — the anniversary coin-rain range
    /// (`Main.cs:64663-64667`): `tenthAnniversaryWorld && !getGoodWorld`
    /// narrows `RollLuck(range)` from 25 to 5. The DRAW COUNT at luck 0 is
    /// unchanged (one `Next(range)`, plus the coin draw on a hit) — only the
    /// denominator moves. WRITTEN NOT RUN (code-only).
    #[test]
    fn w4_f39_compose_the_anniversary_range_is_5_not_25() {
        let mut anniv = rain_world();
        anniv.seeds.insert(tr_core::seeds::SeedFlags::TENTH_ANNIVERSARY);
        // Find a seed whose first draw hits Next(5)==0 but misses Next(25)!=0
        // — the coin arm fires ONLY on the anniversary world.
        let seed = (1..100_000u64)
            .find(|&s| {
                let mut r = StreamReplay::new(s);
                let d = r.next_u64();
                d % 5 == 0 && d % 25 != 0
            })
            .expect("a discriminating seed exists");
        let mut a = Rain::with_forced(seed, None, false);
        a.start_rain(&anniv, false, 0.0);
        let mut r = StreamReplay::new(seed);
        assert_eq!(r.next(0, 5), 0, "the anniversary roll hits");
        let _coin = r.next(75, 151); // the coin draw was consumed
        let (want_time, want_target, want_state) = replay_start_rain_tail(&mut r);
        assert_eq!(a.rng, want_state, "the anniversary stream position");
        assert_eq!(a.rain_time, want_time);
        assert_eq!(a.target, want_target);
        // The same seed on a classic world misses Next(25) — no coin draw.
        let classic = rain_world();
        let mut c = Rain::with_forced(seed, None, false);
        c.start_rain(&classic, false, 0.0);
        let mut r = StreamReplay::new(seed);
        assert_ne!(r.next(0, 25), 0, "the classic roll misses");
        let (want_time, want_target, want_state) = replay_start_rain_tail(&mut r);
        assert_eq!(c.rng, want_state, "the classic stream skips the coin draw");
        assert_eq!(c.rain_time, want_time);
        assert_eq!(c.target, want_target);
        // getGood (FOR_THE_WORTHY) cancels the narrowing (`!getGoodWorld`).
        let mut ftw = rain_world();
        ftw.seeds.insert(tr_core::seeds::SeedFlags::TENTH_ANNIVERSARY);
        ftw.seeds.insert(tr_core::seeds::SeedFlags::FOR_THE_WORTHY);
        let mut f = Rain::with_forced(seed, None, false);
        f.start_rain(&ftw, false, 0.0);
        assert_eq!(f.rng, c.rng, "anniversary+getGood keeps the 25 range");
    }

    #[test]
    fn test_DW_2_4_coin_rain_roll_is_consumed_without_a_spawn() {
        // `Main.cs:64668-64673`: on a `RollLuck(25)` hit the coin value
        // (`Next(75,151) * 100 * 100 * maxTilesX/4200f`) is stored into
        // `World.coin_rain` (W4-f45) but never spawned — coin projectiles are
        // unported (M2/M4). The stream position proves both draws happened; the
        // event still starts with the full ladder behind them.
        let hit = (1..10_000u64)
            .find(|&s| StreamReplay::new(s).next_u64() % 25 == 0)
            .expect("a RollLuck hit seed exists");
        let mut w = rain_world();
        let mut rain = Rain::with_forced(hit, None, false);
        rain.start_rain(&w, false, 0.0, 0.0, 100);
        rain.start_rain(&mut w, false, 0.0);
        assert!(rain.is_raining());
        let (want_time, want_target, want_state) = replay_start_rain(hit);
        assert_eq!(rain.rng, want_state, "the coin draw was consumed");
        assert_eq!(rain.rain_time, want_time);
        assert_eq!(rain.target, want_target);
        // Sanity: the pinned seed really took the coin arm — and the value
        // landed in the world mirror (the `.wld` flags block carries it).
        let mut r = StreamReplay::new(hit);
        assert_eq!(r.next(0, 25), 0, "the seed hits RollLuck(25)");
        let coin = r.next(75, 151);
        let num = 64.0f32 / 4200.0; // `ground()` is 64 tiles wide
        let want_coin_rain = ((coin as i32 * 100 * 100) as f32 * num) as i32;
        assert_eq!(w.coin_rain, want_coin_rain, "coinRain stored (W4-f45)");
        // `StopRain` zeroes it (`Main.cs:64651`).
        rain.stop_rain(&mut w);
        assert_eq!(w.coin_rain, 0, "StopRain clears coinRain");
    }

    #[test]
    fn test_DW_3_4_coin_rain_luck_selects_the_branch_without_reordering() {
        // DW-3.4 — `Main.cs:64668` rides the real player luck: at luck 1.0
        // the hit branch always fires and consumes its 3 draws (float gate +
        // inner Next(12, 25) + outer Next(inner, 25)) where luck 0 consumes
        // the single plain Next(25) — the same stream, branch-selected, no
        // reordering; the ladder tail behind the gate is unchanged.
        let mut w = rain_world();
        for seed in [42u64, 1337, 90_210, 271_828] {
            let mut rain = Rain::with_forced(seed, None, false);
            rain.start_rain(&w, false, 1.0, 0.0, 100);
            rain.start_rain(&mut w, false, 1.0);
            assert!(rain.is_raining(), "seed {seed}");
            let mut r = StreamReplay::new(seed);
            replay_start_rain_head(&mut r, 1.0);
            let (want_time, want_target, want_state) = replay_start_rain_tail(&mut r);
            assert_eq!(rain.rng, want_state, "seed {seed}: exact stream position");
            assert_eq!(rain.rain_time, want_time, "seed {seed}: rainTime ladder");
            assert_eq!(rain.target, want_target, "seed {seed}: ChangeRain band");
            // And the luck-0 run from the same seed sits elsewhere.
            let mut plain = Rain::with_forced(seed, None, false);
            plain.start_rain(&w, false, 0.0, 0.0, 100);
            plain.start_rain(&mut w, false, 0.0);
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

    // ---- g67 transplanted: the rain collision splash → DustPool lane -------
    // g67 `40128e5d`'s seven witnesses, adapted to this branch's fixed
    // 750-slot pool (`pin_drop` instead of the compacted-Vec push, quality
    // through `RainInputs` instead of the `Rain.gfx_quality` field) per
    // docs/research/83 §3. Derived by hand from the C# (`Rain.cs:96-133`,
    // `Dust.cs:75-256`'s default block, `Dust.cs:1834-1874`) over the port's
    // splitmix seam (`app/frame.rs`'s `SplitmixRand`): one `next_rand` step
    // per draw, the 53-bit sample, the C# `Next` conversions.

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

    /// g67 witness — `Rain.cs:117-131`'s solid-kill splash: the gate roll,
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
        let i = inputs(); // gfx_quality 1: the gate always passes
        // Impact: pos (500, 470) + vel (0, 14) → (500, 484) — inside the
        // row-30 solid (480..496). `position - velocity` = (500, 470).
        let pos = Vec2::new(500.0, 470.0);
        let vel = Vec2::new(0.0, 14.0);
        pin_drop(&mut rain, 0, pos, vel);
        let mut pool = tr_core::particles::DustPool::new();
        let ctx = splash_ctx();
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
        let sc = 1.0 + sr.next_range(-20, 21) as f32 * 0.01;
        drop(sr);
        // Type 33's spawn tweak (`Dust.cs:230-235`): alpha 170, vel ×0.5,
        // vel.Y += 1 — BEFORE the recipe writes.
        let tweak_v = Vec2::new(vx, vy) * 0.5 + Vec2::new(0.0, 1.0);
        // The eight recipe writes (`Rain.cs:123-130`).
        let want_pos = Vec2::new(500.0 + jx as f32 + 4.0 - 2.0, 470.0 + jy as f32 + 4.0 + 2.0);
        let want_vel = tweak_v * 0.1 + -vel * 0.025 - Vec2::new(0.0, 2.0);
        rain.update_drops(&w, &i, Some((&mut pool, &ctx)));
        assert_eq!(rain.drop_count(), 0, "the solid kill fired");
        assert_eq!(rain.rng, twin, "gate + 6 default-block draws, in order");
        let live: Vec<_> = pool.slots()[..tr_core::particles::DUST_SENTINEL]
            .iter()
            .filter(|d| d.active)
            .collect();
        assert_eq!(live.len(), 1, "one splash dust");
        let d = live[0];
        assert_eq!(
            d.dust_type, 33,
            "dustWater(): the small-world beach style 13 maps to the default 33"
        );
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

    /// g67 witness — the gfx gate (`Rain.cs:120`): quality 0 rolls
    /// `Next(100)` and NEVER spawns (the roll still consumes the stream);
    /// the refused sentinel takes the eight writes without a live slot
    /// when the ctx is paused (`Dust.cs:85`).
    #[test]
    fn w4_g67_the_gfx_gate_rolls_even_when_it_refuses() {
        let w = tall_ground();
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        let mut i = inputs();
        i.gfx_quality = 0.0;
        pin_drop(&mut rain, 0, Vec2::new(500.0, 470.0), Vec2::new(0.0, 14.0));
        let mut pool = tr_core::particles::DustPool::new();
        let ctx = splash_ctx();
        let mut twin = rain.rng;
        let _gate = crate::app::next_rand(&mut twin) % 100;
        rain.update_drops(&w, &i, Some((&mut pool, &ctx)));
        assert_eq!(rain.rng, twin, "exactly ONE draw: the gate");
        assert_eq!(splash_count(&pool), 0, "quality 0: no splash");
        // The paused ctx refuses INSIDE NewDust (`Dust.cs:85`) — the gate
        // passed, the sentinel took the writes, nothing lives.
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        pin_drop(&mut rain, 0, Vec2::new(500.0, 470.0), Vec2::new(0.0, 14.0));
        let paused = tr_core::particles::SpawnCtx {
            game_paused: true,
            ..splash_ctx()
        };
        let mut twin = rain.rng;
        let _gate = crate::app::next_rand(&mut twin) % 100;
        rain.update_drops(&w, &inputs(), Some((&mut pool, &paused)));
        assert_eq!(rain.rng, twin, "the guard precedes every default draw");
        assert_eq!(splash_count(&pool), 0, "the paused guard refused");
        assert!(!pool.slot(tr_core::particles::DUST_SENTINEL).active);
        assert_eq!(
            pool.slot(tr_core::particles::DUST_SENTINEL).alpha,
            38,
            "the sentinel took the recipe writes (Main.dust[6000])"
        );
    }

    /// g67 witness — the below-screen kill ALSO spawns (`Rain.cs:117`'s
    /// combined branch): a drop past `screenPosition.Y + screenHeight + 100`
    /// rolls the gate and calls NewDust; the off-screen cull
    /// (`Dust.cs:97-101`, margin 400 at dCount 0) is what drops it.
    #[test]
    fn w4_g67_below_screen_kill_rolls_the_gate_and_culls_far_dust() {
        let w = ground();
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        let i = inputs();
        // bottom = 400 + 100 + 100 = 600. The drop lands at y 605 — below,
        // dry (the ground rows are solid but the y-clamp `height-40`=24
        // keeps SolidCollision off them) and inside the dust cull box
        // (view y 300..500 + margin 400 → -100..900; 605-14=591 is INSIDE,
        // so this one SURVIVES the cull — the C# spawns it too).
        pin_drop(&mut rain, 0, Vec2::new(500.0, 591.0), Vec2::new(0.0, 14.0));
        let mut pool = tr_core::particles::DustPool::new();
        let ctx = splash_ctx();
        rain.update_drops(&w, &i, Some((&mut pool, &ctx)));
        assert_eq!(rain.drop_count(), 0, "below-screen kill");
        assert_eq!(splash_count(&pool), 1, "the combined branch spawns it");
        // Far below (y 1205 → dust at 1191 > 900): the cull refuses.
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        pin_drop(&mut rain, 0, Vec2::new(500.0, 1205.0), Vec2::new(0.0, 14.0));
        let mut pool = tr_core::particles::DustPool::new();
        let mut twin = rain.rng;
        let _gate = crate::app::next_rand(&mut twin) % 100;
        rain.update_drops(&w, &i, Some((&mut pool, &ctx)));
        assert_eq!(rain.rng, twin, "the cull precedes the default draws");
        assert_eq!(splash_count(&pool), 0, "off-screen dust culled");
    }

    /// g67 witness — the wet-kill arm (`Rain.cs:117`'s WetCollision):
    /// a drop entering a liquid tile dies and splashes; the probe is the
    /// 2×2 inset rect (`Collision.cs:1576-1587`), the surface at
    /// `tileY + (256-liquid)/32*2`.
    #[test]
    fn w4_g67_wet_surface_impact_splashes() {
        let mut w = World::new("wet".into(), 64, 128);
        // A full water tile at (31, 29): surface y = 464, box 464..480.
        let idx = w.index(31, 29);
        w.tiles[idx].liquid = 255;
        w.compute_column_surface();
        w.compute_chunks();
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        // The drop's next step lands at (500, 466): the 2×2 probe's inset
        // rect (500..502, 466.5..467.5) overlaps the water surface.
        pin_drop(&mut rain, 0, Vec2::new(500.0, 452.0), Vec2::new(0.0, 14.0));
        let mut pool = tr_core::particles::DustPool::new();
        let ctx = splash_ctx();
        rain.update_drops(&w, &inputs(), Some((&mut pool, &ctx)));
        assert_eq!(rain.drop_count(), 0, "the wet kill fired");
        assert_eq!(splash_count(&pool), 1, "wet impacts splash too");
    }

    /// g67 witness — `water_splash_arm` (`Dust.cs:1834-1874`), the
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
        assert!(
            (d.velocity.y - 4.15).abs() < 1e-6,
            "the >4 clamp then the +0.15 sink"
        );
        // Wet contact: +20 alpha and −0.1 scale on top of the fade pair.
        let mut ww = World::new("wet".into(), 64, 64);
        let idx = ww.index(31, 18);
        ww.tiles[idx].liquid = 255; // surface y 288..304 — the 4×4 probe at 300 hits
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

    /// g67 witness — the splash family rides the SHARED `update_dust`
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

    /// g67 witness — the RNG census: N solid impacts consume exactly
    /// N × (1 gate + 6 default-block) draws on Rain's stream, in slot order
    /// (the fixed pool's walk order — the compacted Vec's kill order in
    /// g67); the pool's pressure bands never roll at slot 0..N (dCount 0,
    /// the low band resets it). A second Rain from the same seed replays
    /// the identical stream.
    #[test]
    fn w4_g67_the_splash_draws_ride_rains_stream_in_kill_order() {
        let w = tall_ground();
        let i = inputs();
        let mut a = Rain::with_forced(7, Some(1.0), false);
        let mut b = Rain::with_forced(7, Some(1.0), false);
        for r in [&mut a, &mut b] {
            for k in 0..5 {
                pin_drop(
                    r,
                    k,
                    Vec2::new(400.0 + k as f32 * 32.0, 470.0),
                    Vec2::new(0.0, 14.0),
                );
            }
        }
        let mut pa = tr_core::particles::DustPool::new();
        let mut pb = tr_core::particles::DustPool::new();
        let ctx = splash_ctx();
        a.update_drops(&w, &i, Some((&mut pa, &ctx)));
        b.update_drops(&w, &i, Some((&mut pb, &ctx)));
        assert_eq!(a.rng, b.rng, "the same seed replays the same draws");
        assert_eq!(splash_count(&pa), 5);
        for idx in 0..tr_core::particles::DUST_SENTINEL {
            assert_eq!(pa.slot(idx), pb.slot(idx), "slot {idx} byte-equal");
        }
        // The census: 5 kills × 7 draws from the seed state.
        let mut twin = Rain::with_forced(7, Some(1.0), false).rng;
        for _ in 0..(5 * 7) {
            crate::app::next_rand(&mut twin);
        }
        assert_eq!(a.rng, twin, "5 × (gate + 6 defaults), no extra draws");
    }

    // --- g67b residual folds (d2dd8989 / 5c9398ea / 2c452eddb / 112d1b20),
    // re-pinned to the d130 fixed-slot pool + RainInputs shape.

    /// g67b witness (d2dd8989) — the `gameMenu` kill-only arm
    /// (`Rain.cs:103-109`): the drop still MOVES (`:102` runs first), dies
    /// only below `screenPosition.Y + screenHeight + 2000`, and NEITHER the
    /// gfx gate NOR a splash fires (the else-if chain never reaches `:117`).
    /// view bottom = 400 + 100 = 500 → menu bound 2500.
    #[test]
    fn w4_g67_menu_arm_kills_only_below_plus_2000_without_a_roll() {
        // The one-row floor at row 80 (the (c) rig of the impact-gate
        // witness): the probes clamp at height-40 = 88, so row 80 is solid.
        let mut w = World::new("rain-tall".into(), 64, 128);
        for x in 0..64 {
            let idx = w.index(x, 80);
            w.tiles[idx] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[idx].flags |= FLAG_ACTIVE;
        }
        w.compute_column_surface();
        w.compute_chunks();
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        let mut si = inputs();
        si.game_menu = true;
        si.camera = Vec2::new(512.0, 1300.0); // view bottom 1400 → bound 3400
        // A drop INSIDE the solid floor survives the menu arm — the C#
        // menu branch never probes collisions (steps to 1282, under 3400).
        pin_drop(&mut rain, 0, Vec2::new(500.0, 1268.0), Vec2::new(0.0, 14.0));
        let mut pool = tr_core::particles::DustPool::new();
        let ctx = splash_ctx();
        let state = rain.rng;
        rain.update_drops(&w, &si, Some((&mut pool, &ctx)));
        assert!(rain.drops[0].active, "no collision kill in menu mode");
        assert_eq!(rain.drops[0].pos, Vec2::new(500.0, 1282.0), ":102 moved it");
        assert_eq!(rain.rng, state, "the gate never rolls");
        assert_eq!(splash_count(&pool), 0);
        // Below the +2000 bound: kill-only, still no roll, no splash.
        pin_drop(&mut rain, 0, Vec2::new(500.0, 3390.0), Vec2::new(0.0, 14.0));
        rain.update_drops(&w, &si, Some((&mut pool, &ctx)));
        assert!(!rain.drops[0].active, "3404 > 3400");
        assert_eq!(rain.rng, state, "the menu kill rolls nothing");
        assert_eq!(splash_count(&pool), 0);
    }

    /// g67b witness (5c9398ea) — `NewRainForced` (`Rain.cs:135-164`): the
    /// budget is `maxRain` cut by `(1+q)/2` when q<0.9, ignoring
    /// cloudAlpha/snow/atmo. At q 0 the cap is 375; the 376th spawn refuses
    /// with the `maxRain` sentinel. The spawn takes the `:159-162` shape.
    #[test]
    fn w4_g67_new_rain_forced_budget_and_shape() {
        let mut rain = Rain::with_forced(7, None, false); // intensity 0 — forced ignores it
        let mut i = inputs();
        i.gfx_quality = 0.0; // (1+0)/2 = 0.5 < 0.9 → cap 750*0.5 = 375
        let pos = Vec2::new(500.0, 300.0);
        let vel = Vec2::new(2.0, 9.0);
        let idx = rain.new_rain_forced(&i, pos, vel);
        assert_eq!(idx, 0);
        let d = rain.drops[0];
        assert_eq!(d.pos, pos);
        assert!((d.scale - 1.0).abs() <= 0.2 + 1e-6);
        assert_eq!(d.vel, vel * d.scale, "`:160` velocity * scale");
        assert_eq!(d.rot, d.vel.x.atan2(-d.vel.y), "`:161`");
        // Fill to the cap, then the next refuses.
        let mut n = 1;
        while n < 375 {
            assert_eq!(rain.new_rain_forced(&i, pos, vel), n);
            n += 1;
        }
        assert_eq!(rain.new_rain_forced(&i, pos, vel), MAX_DROPS, "full → maxRain");
        assert_eq!(rain.drop_count(), 375);
    }

    /// g67b witness (112d1b20) — `ClearRain` (`Rain.cs:22-28`) empties the
    /// field; `IsItRaining` (`Main.cs:2597`) follows `cloudAlpha`, NOT the
    /// event flag.
    #[test]
    fn w4_g67_clear_rain_and_is_it_raining_semantics() {
        let w = ground();
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        let i = inputs();
        for _ in 0..60 {
            rain.update(&w, &i, 1.0 / 60.0, None);
        }
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

    /// g67b witness (2c452eddb) — the `DrawDust` cull rect (`Main.cs:38832`
    /// / `:38847` / `:39020-39022`): the ±1000/±1050 slack box off
    /// `screenPosition`, the 4×4 strict-`<` intersection, and the else-arm
    /// DEACTIVATION (the old widening dropped the kill). Pure geometry —
    /// both draw loops (splashes + snow 76) apply exactly this test before
    /// drawing each family slot.
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
        assert!(
            !dust_in_cull_rect(Vec2::new(800.0, 450.0), shifted),
            "the old view is now outside"
        );
    }

    /// W4-g67 witness (folded `f8c4b659`, research/134 R5) — the skyblock
    /// snow-threshold arm (`Rain.cs:181` via `SceneMetrics.SnowTileThreshold`,
    /// `SceneMetrics.cs:154-163`): 300 snow tiles fully suppress the budget on
    /// a low-tiles skyblock world (ratio 1 → budget 0) but barely suppress on
    /// a normal world (300/1500 = 0.2 → budget 720 > the 26-attempt spawn).
    /// Adapted to the `RainInputs.skyblock_low_tiles` seam. WRITTEN NOT RUN.
    #[test]
    fn w4_g67_skyblock_low_tiles_lowers_the_snow_threshold() {
        let w = World::new("air".into(), 64, 64);
        let mut i = inputs();
        // The threshold getters the spawn reads.
        assert_eq!(tr_core::scene::snow_tile_threshold(false), 1500);
        assert_eq!(tr_core::scene::snow_tile_threshold(true), 300);
        // A skyblock world with 300+ snow tiles fully suppresses (ratio 1 →
        // budget 0): no drop spawns at all.
        let mut sky = Rain::with_forced(7, Some(1.0), false);
        i.skyblock_low_tiles = true;
        i.snow_tile_count = 300;
        for _ in 0..60 {
            sky.update(&w, &i, 1.0 / 60.0, None);
        }
        assert_eq!(sky.drop_count(), 0, "300 snow tiles fully suppress on skyblock");
        // The same 300 tiles on a normal world: ratio 0.2 → budget 720 > the
        // per-tick attempts, so drops still spawn.
        let mut normal = Rain::with_forced(7, Some(1.0), false);
        i.skyblock_low_tiles = false;
        for _ in 0..60 {
            normal.update(&w, &i, 1.0 / 60.0, None);
        }
        assert!(normal.drop_count() > 0, "300 tiles barely suppress on a normal world");
    }

    /// W4-g67d witness (final phase 3, `7aaf66f0`) — the DrawRain light-sample
    /// tile is `(int)(pos + 4f) >> 4` (`Main.cs:38813`), NOT the dust
    /// convention's bare `pos/16`: in the 12..15 px band of a cell the `+4`
    /// crosses into the NEXT tile. The d130 draw carried the semantics inline
    /// since `ea4863d9`; this pins the extracted helper both share.
    /// WRITTEN NOT RUN (code-only).
    #[test]
    fn w4_g67d_draw_rain_light_tile_is_pos_plus_four_shifted() {
        // Cell interior: pos 16*3+2 = 50 → (50+4)>>4 = 3 (same tile).
        assert_eq!(rain_light_tile(Vec2::new(50.0, 50.0)), (3, 3));
        // The 12..15 band: pos 16*3+12 = 60 → (60+4)>>4 = 4 (NEXT tile —
        // a bare pos/16 read tile 3 here, one tile early).
        assert_eq!(rain_light_tile(Vec2::new(60.0, 60.0)), (4, 4));
        // The exact edge: pos 63.9 → (int)67.9 = 67 >> 4 = 4.
        assert_eq!(rain_light_tile(Vec2::new(63.9, 63.9)), (4, 4));
        // Negative coords truncate toward zero then arithmetic-shift:
        // pos -1 → (int)3 = 3 >> 4 = 0; pos -5 → (int)(-1) = -1 >> 4 = -1.
        assert_eq!(rain_light_tile(Vec2::new(-1.0, -1.0)), (0, 0));
        assert_eq!(rain_light_tile(Vec2::new(-5.0, -5.0)), (-1, -1));
    }
}
