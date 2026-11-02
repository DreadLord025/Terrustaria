//! Scene sensors and state: `SceneMetrics` (the tile/window sensor), the
//! `Sandstorm` event and `SceneState` (the per-frame fades/intensities).
//!
//! Sources:
//! * `SceneMetrics` — `decompiled/Terraria/SceneMetrics.cs`.
//! * `Sandstorm` — `decompiled/Terraria/GameContent/Events/Sandstorm.cs`.
//! * `SceneState` — `decompiled/Terraria/SceneState.cs`.
//!
//! Data only: no render types cross this module. The tile-window scan that
//! feeds [`SceneMetrics`] lives in `tr-app` (`world_sim::scene_scan`) because
//! the scan is centered on the player, which is a `tr-app` type.

use crate::tile_light::FastRandom;

/// How many tile kinds the window counts cover (`_tileCounts`, `TileID.Count`).
const TILE_COUNT: usize = crate::tiledata::TILE_COUNT;

/// `SceneMetrics` — the M1 sensor subset: the scan window counts around the
/// player, the derived biome counts and the zone flags the screen filters read.
///
/// Fields and thresholds follow `SceneMetrics.cs`; the full type carries many
/// more counters (NPC positions, candles, monoliths) that no M1 filter reads —
/// add them when a reader lands.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneMetrics {
    /// `metrics.Center`: the scan center in world pixels (the player center).
    pub center: [f32; 2],
    /// `metrics.TileCenter`: `Center.ToTileCoordinates().ClampedInWorld()`.
    pub tile_center: (i32, i32),
    /// `metrics.BehindBackwall`: the TileCenter tile has a wall behind it
    /// (`SceneMetrics.cs:671-672`).
    pub behind_backwall: bool,
    /// `_tileCounts[tile.type]` per active tile in the scan window
    /// (`SceneMetrics.cs:353-373`).
    pub tile_counts: Vec<u32>,
    /// `metrics.OceanSandTileCount` (`SceneMetrics.cs:377-379`).
    pub ocean_sand_tile_count: i32,
    /// `metrics.SandTileCount` (`SceneMetrics.cs:613`).
    pub sand_tile_count: i32,
    /// `metrics.HolyTileCount` (`SceneMetrics.cs:589`).
    pub holy_tile_count: i32,
    /// `metrics.SnowTileCount` (`SceneMetrics.cs:590`).
    pub snow_tile_count: i32,
    /// `metrics.GraveyardTileCount` (`SceneMetrics.cs:615-624`).
    pub graveyard_tile_count: i32,
    /// `metrics.DesertSandTileCount` (`SceneMetrics.cs:658`).
    pub desert_sand_tile_count: i32,
    /// `metrics.SurfaceAtmospherics` (`SceneMetrics.cs:698`, `WorldGen.cs:10976`).
    pub surface_atmospherics: bool,
    /// `metrics.ZoneRain` (`SceneMetrics.cs:701`).
    pub zone_rain: bool,
    /// `metrics.ZoneSnow` (`SceneMetrics.cs:675`).
    pub zone_snow: bool,
    /// `metrics.ZoneDesert` (`SceneMetrics.cs:676`).
    pub zone_desert: bool,
    /// `metrics.ZoneSandstorm` (`SceneMetrics.cs:703-707`).
    pub zone_sandstorm: bool,
}

/// `SceneMetrics.AssumedConstantScreenSize` / `ZoneScanPadding` /
/// `ZoneScanSize` (`SceneMetrics.cs:915-919`): 1920/16 + 25*2 - 1 = 169 wide,
/// 1200/16 + 25*2 - 1 = 124 tall.
pub const ZONE_SCAN_SIZE: (i32, i32) = (1920 / 16 + 25 * 2 - 1, 1200 / 16 + 25 * 2 - 1);

/// The static thresholds from `SceneMetrics`'s static ctor (`:922-943`).
pub mod thresholds {
    /// `SnowTileNormalThreshold` (`SceneMetrics.cs:930`).
    pub const SNOW_TILE_NORMAL: i32 = 1500;
    /// `DesertTileNormalThreshold` (`SceneMetrics.cs:934`).
    pub const DESERT_TILE_NORMAL: i32 = 1500;
    /// `GraveyardTileThreshold` (`SceneMetrics.cs:941`).
    pub const GRAVEYARD_TILE: i32 = 28;
    /// `GraveyardTileMin` (`SceneMetrics.cs:940`).
    pub const GRAVEYARD_TILE_MIN: i32 = 16;
    /// `GraveyardTileMax` (`SceneMetrics.cs:939`).
    pub const GRAVEYARD_TILE_MAX: i32 = 36;
}

impl SceneMetrics {
    /// `SceneMetrics.Reset` — zeroed counts and flags.
    pub fn new() -> Self {
        Self {
            center: [0.0; 2],
            tile_center: (0, 0),
            behind_backwall: false,
            tile_counts: vec![0; TILE_COUNT],
            ocean_sand_tile_count: 0,
            sand_tile_count: 0,
            holy_tile_count: 0,
            snow_tile_count: 0,
            graveyard_tile_count: 0,
            desert_sand_tile_count: 0,
            surface_atmospherics: false,
            zone_rain: false,
            zone_snow: false,
            zone_desert: false,
            zone_sandstorm: false,
        }
    }

    /// `metrics.EnoughTilesForSnow` (`SceneMetrics.cs:258`; threshold
    /// `SnowTileNormalThreshold`, `:930` — the Skyblock variant is not ported).
    pub fn enough_tiles_for_snow(&self) -> bool {
        self.snow_tile_count >= thresholds::SNOW_TILE_NORMAL
    }

    /// `metrics.EnoughTilesForDesert` (`SceneMetrics.cs:262`; threshold
    /// `DesertTileNormalThreshold`, `:934`).
    pub fn enough_tiles_for_desert(&self) -> bool {
        self.desert_sand_tile_count >= thresholds::DESERT_TILE_NORMAL
    }

    /// `metrics.EnoughTilesForGraveyard` (`SceneMetrics.cs:272`; threshold
    /// `GraveyardTileThreshold`, `:941`).
    pub fn enough_tiles_for_graveyard(&self) -> bool {
        self.graveyard_tile_count >= thresholds::GRAVEYARD_TILE
    }

    /// `SceneMetrics.AggregateTileCounts` (`SceneMetrics.cs:573-660`), M1
    /// subset. The full routine also derives Jungle/Evil/Blood/Mushroom/
    /// Meteor/Dungeon counts and the Holy/Evil/Blood cross-subtraction — none
    /// of which has an M1 reader, so their terms stay 0 here and the
    /// cross-subtraction (`:629-647`) is a no-op for the M1 sums.
    pub fn aggregate_tile_counts(&mut self) {
        // `num = -10` (`:575-578`) only scales `_tileCounts[27]` inside the
        // Evil/Blood sums (×3 for the infected seed) — both are M-later.
        // `SceneMetrics.cs:589`:
        self.holy_tile_count = self.count(109)
            + self.count(492)
            + self.count(110)
            + self.count(113)
            + self.count(117)
            + self.count(116)
            + self.count(164)
            + self.count(403)
            + self.count(402);
        // `SceneMetrics.cs:590`:
        self.snow_tile_count = self.count(147)
            + self.count(148)
            + self.count(161)
            + self.count(162)
            + self.count(164)
            + self.count(163)
            + self.count(200);
        // `SceneMetrics.cs:613`:
        self.sand_tile_count = self.count(53)
            + self.count(112)
            + self.count(116)
            + self.count(234)
            + self.count(397)
            + self.count(398)
            + self.count(402)
            + self.count(399)
            + self.count(396)
            + self.count(400)
            + self.count(403)
            + self.count(401);
        // `SceneMetrics.cs:615-624`: tombstones minus half the sunflowers
        // (integer division), floored at zero.
        self.graveyard_tile_count = self.count(85) - self.count(27) / 2;
        if self.graveyard_tile_count < 0 {
            self.graveyard_tile_count = 0;
        }
        // `SceneMetrics.cs:658`: `Math.Max(0, SandTileCount - OceanSandTileCount)`.
        self.desert_sand_tile_count = (self.sand_tile_count - self.ocean_sand_tile_count).max(0);
    }

    /// `_tileCounts[id]` as an `i32` summand.
    fn count(&self, id: u16) -> i32 {
        self.tile_counts[id as usize] as i32
    }

    /// `SceneMetrics.CalculateZones` (`SceneMetrics.cs:662-707`), M1 subset.
    /// `world_surface` is `Main.worldSurface`; `raining` is `Main.raining`;
    /// `sandstorm_happening` is `Sandstorm.Happening`. The depth flags the
    /// heat/water filters read (`ZoneOverworldHeight`, …) derive straight from
    /// `tile_center` in the phase-3 driver.
    pub fn calculate_zones(
        &mut self,
        world_surface: f64,
        raining: bool,
        sandstorm_happening: bool,
    ) {
        // `SceneMetrics.cs:675-676`:
        self.zone_snow = self.enough_tiles_for_snow();
        self.zone_desert = self.enough_tiles_for_desert();
        // `SceneMetrics.cs:698` via `WorldGen.IsSurfaceForAtmospherics`
        // (`WorldGen.cs:10976-10985`; the remix branch is not ported):
        self.surface_atmospherics = self.tile_center.1 as f64 <= world_surface;
        // `SceneMetrics.cs:701`:
        self.zone_rain = raining && self.surface_atmospherics;
        // `SceneMetrics.cs:703`:
        self.zone_sandstorm = self.zone_desert && self.surface_atmospherics && sandstorm_happening;
        // `SceneMetrics.cs:704-707`: a sandstorm suppresses the rain zone.
        if self.zone_sandstorm {
            self.zone_rain = false;
        }
    }
}

/// `WorldGen.oceanDepths` (`WorldGen.cs:…`): the beach bands at the world's
/// left/right edges, above `oceanLevel` = `(worldSurface + rockLayer)/2 + 40`
/// (`WorldGen.cs:4447`). `WorldGen.beachDistance` = 380 (`WorldGen.cs:86914`).
pub fn ocean_depths(world_surface: f64, rock_layer: f64, x: i32, max_tiles_x: i32, y: i32) -> bool {
    const BEACH_DISTANCE: i32 = 380;
    let ocean_level = (world_surface + rock_layer) / 2.0 + 40.0;
    if y as f64 > ocean_level {
        return false;
    }
    x < BEACH_DISTANCE || x > max_tiles_x - BEACH_DISTANCE
}

/// `TileID.Sets.isDesertBiomeSand` (`TileID.cs:501`): the sands that count as
/// ocean/desert sand.
pub fn is_desert_biome_sand(kind: u16) -> bool {
    matches!(kind, 53 | 397 | 396 | 400 | 403 | 401)
}

/// The `Sandstorm` event (`Events/Sandstorm.cs`).
///
/// The Severity transitions are pure; the start/severity-intention rolls draw
/// from this struct's own `FastRandom` stream — vanilla pulls them from the
/// shared `Main.rand` (`Sandstorm.cs:64,71,83,91,120`), but no UnifiedRandom
/// port exists, and an isolated stream keeps the state machine deterministic.
/// `TR_SANDSTORM` (see [`SceneState::force`]) is the primary determinism lever.
pub struct Sandstorm {
    /// `Sandstorm.Happening` (`Sandstorm.cs:14`).
    pub happening: bool,
    /// `Sandstorm.TimeLeft` (`Sandstorm.cs:16`).
    pub time_left: i32,
    /// `Sandstorm.Severity` (`Sandstorm.cs:18`).
    pub severity: f32,
    /// `Sandstorm.IntendedSeverity` (`Sandstorm.cs:20`).
    pub intended_severity: f32,
    rng: FastRandom,
}

/// `SANDSTORM_DURATION_MINIMUM` / `_MAXIMUM` (`Sandstorm.cs:10-12`).
const SANDSTORM_DURATION_MIN: i32 = 28800;
const SANDSTORM_DURATION_MAX: i32 = 86400;

impl Sandstorm {
    /// A fresh, not-happening storm with its own RNG stream.
    pub fn new(seed: u64) -> Self {
        Self {
            happening: false,
            time_left: 0,
            severity: 0.0,
            intended_severity: 0.0,
            rng: FastRandom::new(seed),
        }
    }

    /// `Sandstorm.HasSufficientWind` (`Sandstorm.cs:22-25`).
    pub fn has_sufficient_wind(wind_speed_current: f32) -> bool {
        wind_speed_current.abs() >= 0.6
    }

    /// `Sandstorm.UpdateTime` (`Sandstorm.cs:32-77`). `day_rate` is
    /// `Main.dayRate` (1 in our port: no sleeping/creative powers yet);
    /// `wind_speed_current` is `Main.windSpeedCurrent`. The `Main.netMode != 1`
    /// gate (`:34`) is always true for a single-player session. The start roll
    /// uses the no-hardMode branch (`:58-59`: `num = 21600 * 3`; hardmode is
    /// not ported yet).
    pub fn update_time(&mut self, day_rate: i32, wind_speed_current: f32) {
        if self.happening {
            // `Sandstorm.cs:38-41`:
            if self.time_left > SANDSTORM_DURATION_MAX {
                self.time_left = 0;
            }
            // `Sandstorm.cs:42-46`: decay, plus the 15×/tick wind penalty.
            self.time_left -= day_rate;
            if !Self::has_sufficient_wind(wind_speed_current) {
                self.time_left -= 15 * day_rate;
            }
            // `Sandstorm.cs:47-50`: dead-calm wind ends the storm at once.
            if wind_speed_current == 0.0 {
                self.time_left = 0;
            }
            // `Sandstorm.cs:51-54`:
            if self.time_left <= 0 {
                self.stop_sandstorm();
            }
        } else {
            // `Sandstorm.cs:58-59`: start-delay denominator.
            let num = 21600 * 3;
            // `Sandstorm.cs:60-69`: one roll per day-rate tick.
            if Self::has_sufficient_wind(wind_speed_current) {
                for _ in 0..day_rate {
                    if self.rng.next_range(0, num) == 0 {
                        self.start_sandstorm();
                    }
                }
            }
        }
        // `Sandstorm.cs:71-74`:
        if self.rng.next_range(0, 18000) == 0 {
            self.change_severity_intentions();
        }
        // `Sandstorm.cs:76`:
        self.update_severity();
    }

    /// `Sandstorm.StartSandstorm` (`Sandstorm.cs:117-122`).
    pub fn start_sandstorm(&mut self) {
        self.happening = true;
        self.time_left = self
            .rng
            .next_range(SANDSTORM_DURATION_MIN, SANDSTORM_DURATION_MAX + 1);
        self.change_severity_intentions();
    }

    /// `Sandstorm.StopSandstorm` (`Sandstorm.cs:124-129`).
    pub fn stop_sandstorm(&mut self) {
        self.happening = false;
        self.time_left = 0;
        self.change_severity_intentions();
    }

    /// `Sandstorm.ChangeSeverityIntentions` (`Sandstorm.cs:79-97`); the
    /// `NetMessage.SendData(7)` tail (`:93-96`) is multiplayer-only.
    fn change_severity_intentions(&mut self) {
        if self.happening {
            // `Sandstorm.cs:83`: 0.4..1.0 while a storm runs.
            self.intended_severity = 0.4 + self.rng.next_float();
        } else if self.rng.next_range(0, 3) == 0 {
            // `Sandstorm.cs:85-88`: a third of the time it wants to stay clear.
            self.intended_severity = 0.0;
        } else {
            // `Sandstorm.cs:89-92`: otherwise a low ambient severity.
            self.intended_severity = self.rng.next_float() * 0.3;
        }
    }

    /// `Sandstorm.UpdateSeverity` (`Sandstorm.cs:99-115`): approach
    /// `intended_severity` at 0.003/tick, snapping on overshoot.
    pub fn update_severity(&mut self) {
        // `Sandstorm.cs:101-108`: NaN guards.
        if self.severity.is_nan() {
            self.severity = 0.0;
        }
        if self.intended_severity.is_nan() {
            self.intended_severity = 0.0;
        }
        // `Sandstorm.cs:109-110`: `Math.Sign` gives 0 on equality.
        let num = match self.intended_severity - self.severity {
            d if d > 0.0 => 1,
            d if d < 0.0 => -1,
            _ => 0,
        };
        self.severity = (self.severity + 0.003 * num as f32).clamp(0.0, 1.0);
        // `Sandstorm.cs:111-114`: crossed the target this tick — snap to it.
        let crossed = match self.intended_severity - self.severity {
            d if d > 0.0 => 1,
            d if d < 0.0 => -1,
            _ => 0,
        };
        if num != crossed {
            self.severity = self.intended_severity;
        }
    }

    /// `TR_SANDSTORM=<0..1>` pin: hold the severity at a fixed value with the
    /// storm running (value > 0). With `severity == intended_severity`,
    /// `update_severity` is a no-op, so the pin survives the state machine.
    pub fn force_severity(&mut self, value: f32) {
        let v = value.clamp(0.0, 1.0);
        self.severity = v;
        self.intended_severity = v;
        self.happening = v > 0.0;
    }

    /// `Sandstorm.ShowSandstormVisuals` (`Sandstorm.cs:131-138`): is the
    /// sandstorm filter/overlay visibly on. The bg inputs are the caller's
    /// (`Main.bgStyle` desert-variant check and `Main.bgDelay`).
    pub fn show_sandstorm_visuals(
        &self,
        zone_sandstorm: bool,
        bg_is_desert_variant: bool,
        bg_delay: i32,
    ) -> bool {
        if self.happening && zone_sandstorm && bg_is_desert_variant {
            return bg_delay < 50;
        }
        false
    }
}

/// `SceneState` (`SceneState.cs:14-62`) — the M1-owned fades and intensities.
/// The filter-parameter driver (`ApplyVisuals`) is phase 3 (`tr-app`); this
/// struct only mutates its own state.
pub struct SceneState {
    /// `SceneState.airLightDecay` (`SceneState.cs:15`).
    pub air_light_decay: f32,
    /// `SceneState.solidLightDecay` (`SceneState.cs:16`).
    pub solid_light_decay: f32,
    /// `SceneState.outsideWeatherEffectIntensity` (`SceneState.cs:20`).
    pub outside_weather_effect_intensity: f32,
    /// `SceneState._outsideWeatherEffectIntensityBackingValue` (`:22`).
    pub outside_weather_effect_intensity_backing: f32,
    /// `Main.GraveyardVisualIntensity` (`Main.cs:1327`): vanilla keeps it on
    /// Main, but `SceneState.UpdateGraveyard` (`:368-371`) is its only writer,
    /// so the port owns it here.
    pub graveyard_visual_intensity: f32,
    /// `SceneState.skipTransitions` (`:35`).
    pub skip_transitions: bool,
    /// `Main.bloodMoon` — the only M1 reader is the phase-3 BloodMoon filter;
    /// set through [`SceneState::force`].
    pub blood_moon: bool,
    /// The `TR_SANDSTORM` pin, mirrored by `Sandstorm::force_severity` at the
    /// `tr-app` glue so the driver can also skip the storm update entirely.
    pub forced_severity: Option<f32>,
    /// The `TR_GRAVEYARD` pin: when set, `update` holds the intensity instead
    /// of fading it toward the tile-derived target.
    pub forced_graveyard: Option<f32>,
}

impl SceneState {
    /// `SceneState.Reset` (`SceneState.cs:44-54`).
    pub fn new() -> Self {
        let mut state = Self {
            air_light_decay: 1.0,
            solid_light_decay: 1.0,
            outside_weather_effect_intensity: 1.0,
            outside_weather_effect_intensity_backing: 1.0,
            graveyard_visual_intensity: 0.0,
            skip_transitions: true,
            blood_moon: false,
            forced_severity: None,
            forced_graveyard: None,
        };
        state.reset();
        state
    }

    /// `SceneState.Reset` (`SceneState.cs:44-54`); the graveyard intensity is
    /// zeroed with the world (`Main.cs:19860`). The TR_ pins clear too — the
    /// per-frame env glue (`scene_scan::apply_scene_forces_from_env`) re-applies
    /// them on the next tick.
    pub fn reset(&mut self) {
        self.air_light_decay = 1.0;
        self.solid_light_decay = 1.0;
        self.outside_weather_effect_intensity = 1.0;
        self.outside_weather_effect_intensity_backing = 1.0;
        self.graveyard_visual_intensity = 0.0;
        self.skip_transitions = true;
        self.blood_moon = false;
        self.forced_severity = None;
        self.forced_graveyard = None;
    }

    /// `SceneState.Update` (`SceneState.cs:56-62`), tr-core subset: the
    /// graveyard intensity fade, the light decays and the outside-weather
    /// intensity backing; then `skip_transitions = false` (`:62`). (The vanilla
    /// body also drives the filter params and the map capture — the filter
    /// driver is phase 3, the map capture already lives in `tr-core/map`.)
    pub fn update(&mut self, metrics: &SceneMetrics) {
        let skip = self.skip_transitions;
        // `SceneState.cs:72` → `UpdateGraveyard` (`:368-371`): fade toward the
        // count-derived 0..1 target (0.02/tick in, 0.1/tick out).
        let target = get_lerp_value(
            thresholds::GRAVEYARD_TILE_MIN as f32,
            thresholds::GRAVEYARD_TILE_MAX as f32,
            metrics.graveyard_tile_count as f32,
            true,
        );
        match self.forced_graveyard {
            // `TR_GRAVEYARD` pin: hold the forced value, skip the fade.
            Some(v) => self.graveyard_visual_intensity = v,
            None => move_towards_impl(
                skip,
                &mut self.graveyard_visual_intensity,
                target,
                0.02,
                0.1,
            ),
        }
        // `SceneState.cs:74` → `UpdateLightDecay` (`:255-281`): the targets are
        // `1 - shimmerAlpha * {0, 0.3}` with the `getGoodWorld` NPC terms —
        // shimmer and FTW seeds are not ported, so both targets are 1.
        move_towards_impl(skip, &mut self.air_light_decay, 1.0, 0.005, 0.005);
        move_towards_impl(skip, &mut self.solid_light_decay, 1.0, 0.005, 0.005);
        // `SceneState.cs:134-135`: the backing value chases -0.1 behind a
        // backwall, 1.1 in the open; the intensity clamps it to 0..1.
        move_towards_impl(
            skip,
            &mut self.outside_weather_effect_intensity_backing,
            if metrics.behind_backwall { -0.1 } else { 1.1 },
            0.005,
            0.005,
        );
        self.outside_weather_effect_intensity = self
            .outside_weather_effect_intensity_backing
            .clamp(0.0, 1.0);
        // `SceneState.cs:62`:
        self.skip_transitions = false;
    }

    /// `SceneState.MoveTowards(ref value, target, amount)` (`:472-477`).
    pub fn move_towards(&self, value: &mut f32, target: f32, amount: f32) {
        move_towards_impl(self.skip_transitions, value, target, amount, amount);
    }

    /// `SceneState.MoveTowards(ref value, target, inc, dec)` (`:479-491`).
    pub fn move_towards_inc_dec(&self, value: &mut f32, target: f32, inc: f32, dec: f32) {
        move_towards_impl(self.skip_transitions, value, target, inc, dec);
    }

    /// The TR_ determinism forces: `SANDSTORM` records the severity pin
    /// (`Sandstorm::force_severity` applies it at the glue), `GRAVEYARD` pins
    /// the visual intensity, `BLOODMOON` sets `blood_moon`. Returns whether
    /// `name` was recognized.
    pub fn force(&mut self, name: &str, value: f32) -> bool {
        match name {
            "SANDSTORM" => self.forced_severity = Some(value.clamp(0.0, 1.0)),
            "GRAVEYARD" => self.forced_graveyard = Some(value.clamp(0.0, 1.0)),
            "BLOODMOON" => self.blood_moon = value != 0.0,
            _ => return false,
        }
        true
    }
}

/// The shared body of both `SceneState.MoveTowards` overloads
/// (`SceneState.cs:479-491`).
fn move_towards_impl(skip_transitions: bool, value: &mut f32, target: f32, inc: f32, dec: f32) {
    if skip_transitions {
        *value = target;
    } else if *value < target {
        *value = (*value + inc).min(target);
    } else if *value > target {
        *value = (*value - dec).max(target);
    }
}

/// Parses one `TR_<NAME>` / `TR_<NAME>=<value>` environment entry into the
/// `(name, value)` pair `SceneState::force` eats. A bare name means 1.0;
/// numeric values are clamped to 0..1; anything unparsable is `None`.
pub fn parse_env_force(raw: &str) -> Option<(&str, f32)> {
    let rest = raw.strip_prefix("TR_")?;
    let (name, value) = match rest.split_once('=') {
        Some((name, value)) => (name, value),
        None => (rest, ""),
    };
    if name.is_empty() {
        return None;
    }
    let v = if value.is_empty() {
        1.0
    } else {
        value.trim().parse::<f32>().ok()?
    };
    if v.is_nan() {
        return None;
    }
    Some((name, v.clamp(0.0, 1.0)))
}

/// `Utils.GetLerpValue` (`Utils.cs:315-343`): the clamped inverse lerp the
/// graveyard intensity target uses. Same body as the local copy in `wind.rs`
/// (`wind.rs:361`); shared here once a second caller appeared.
pub fn get_lerp_value(from: f32, to: f32, t: f32, clamped: bool) -> f32 {
    if clamped {
        if from < to {
            if t < from {
                return 0.0;
            }
            if t > to {
                return 1.0;
            }
        } else {
            if t < to {
                return 1.0;
            }
            if t > from {
                return 0.0;
            }
        }
    }
    (t - from) / (to - from)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A metrics with only the given window counts set.
    fn metrics_with_counts(counts: &[(u16, u32)]) -> SceneMetrics {
        let mut m = SceneMetrics::new();
        for &(id, n) in counts {
            m.tile_counts[id as usize] = n;
        }
        m
    }

    // --- AggregateTileCounts (SceneMetrics.cs:573-660) ---

    #[test]
    fn holy_and_snow_sums_match_source_id_lists() {
        // `SceneMetrics.cs:589-590`: 164 belongs to BOTH sums.
        let mut m = metrics_with_counts(&[
            (109, 10),
            (492, 1),
            (110, 2),
            (113, 3),
            (117, 4),
            (116, 5),
            (164, 6),
            (403, 7),
            (402, 8),
            (147, 100),
            (161, 200),
            (163, 300),
            (200, 400),
        ]);
        m.aggregate_tile_counts();
        assert_eq!(m.holy_tile_count, 10 + 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8);
        assert_eq!(m.snow_tile_count, 6 + 100 + 200 + 300 + 400);
    }

    #[test]
    fn sand_sums_match_source_id_list() {
        // `SceneMetrics.cs:613`.
        let mut m = metrics_with_counts(&[
            (53, 1),
            (112, 2),
            (116, 4),
            (234, 8),
            (397, 16),
            (398, 32),
            (402, 64),
            (399, 128),
            (396, 256),
            (400, 512),
            (403, 1024),
            (401, 2048),
        ]);
        m.aggregate_tile_counts();
        let total: i32 = 1 + 2 + 4 + 8 + 16 + 32 + 64 + 128 + 256 + 512 + 1024 + 2048;
        assert_eq!(m.sand_tile_count, total);
        // No ocean sand seen in the window: desert sand = sand.
        assert_eq!(m.desert_sand_tile_count, total);
    }

    #[test]
    fn graveyard_is_tombstones_minus_half_sunflowers_floored_at_zero() {
        // `SceneMetrics.cs:615-624`: `counts[85] - counts[27]/2` (integer
        // division), floored at zero.
        let mut m = metrics_with_counts(&[(85, 30), (27, 10)]);
        m.aggregate_tile_counts();
        assert_eq!(m.graveyard_tile_count, 25);

        let mut m = metrics_with_counts(&[(85, 4), (27, 9)]);
        m.aggregate_tile_counts();
        assert_eq!(m.graveyard_tile_count, 0); // 4 - 4 = 0

        let mut m = metrics_with_counts(&[(85, 0), (27, 20)]);
        m.aggregate_tile_counts();
        assert_eq!(m.graveyard_tile_count, 0); // -10 -> 0
    }

    #[test]
    fn desert_sand_subtracts_ocean_sand_floored_at_zero() {
        // `SceneMetrics.cs:658`. The sand sum is re-derived from the window
        // counts; the ocean count is filled by the scan (`:377-379`).
        let mut m = metrics_with_counts(&[(53, 100)]);
        m.ocean_sand_tile_count = 40;
        m.aggregate_tile_counts();
        assert_eq!(m.sand_tile_count, 100);
        assert_eq!(m.desert_sand_tile_count, 60);

        let mut m = metrics_with_counts(&[(53, 10)]);
        m.ocean_sand_tile_count = 40;
        m.aggregate_tile_counts();
        assert_eq!(m.desert_sand_tile_count, 0);
    }

    // --- CalculateZones (SceneMetrics.cs:662-707) ---

    #[test]
    fn zones_at_exact_thresholds() {
        // `SceneMetrics.cs:258,262`: `>=` comparisons.
        let mut m = SceneMetrics::new();
        m.snow_tile_count = thresholds::SNOW_TILE_NORMAL;
        m.desert_sand_tile_count = thresholds::DESERT_TILE_NORMAL;
        m.calculate_zones(100.0, false, false);
        assert!(m.zone_snow);
        assert!(m.zone_desert);

        let mut m = SceneMetrics::new();
        m.snow_tile_count = thresholds::SNOW_TILE_NORMAL - 1;
        m.desert_sand_tile_count = thresholds::DESERT_TILE_NORMAL - 1;
        m.calculate_zones(100.0, false, false);
        assert!(!m.zone_snow);
        assert!(!m.zone_desert);
    }

    #[test]
    fn zone_rain_needs_raining_and_surface() {
        let mut m = SceneMetrics::new();
        m.tile_center = (100, 100);
        m.calculate_zones(150.0, true, false); // y=100 <= surface=150
        assert!(m.zone_rain && m.surface_atmospherics);

        let mut m = SceneMetrics::new();
        m.tile_center = (100, 200);
        m.calculate_zones(150.0, true, false); // below the surface
        assert!(!m.zone_rain && !m.surface_atmospherics);

        let mut m = SceneMetrics::new();
        m.tile_center = (100, 100);
        m.calculate_zones(150.0, false, false);
        assert!(!m.zone_rain);
    }

    #[test]
    fn zone_rain_and_zone_snow_coexist() {
        // The blizzard gate is `ZoneRain && ZoneSnow` (`SceneState.cs:120`) —
        // both flags must survive together.
        let mut m = SceneMetrics::new();
        m.snow_tile_count = thresholds::SNOW_TILE_NORMAL;
        m.tile_center = (0, 10);
        m.calculate_zones(100.0, true, false);
        assert!(m.zone_rain);
        assert!(m.zone_snow);
    }

    #[test]
    fn zone_sandstorm_suppresses_zone_rain() {
        // `SceneMetrics.cs:703-707`.
        let mut m = SceneMetrics::new();
        m.desert_sand_tile_count = thresholds::DESERT_TILE_NORMAL;
        m.tile_center = (0, 10);
        m.calculate_zones(100.0, true, true);
        assert!(m.zone_sandstorm);
        assert!(!m.zone_rain);

        // Without the happening storm the rain zone stays.
        let mut m = SceneMetrics::new();
        m.desert_sand_tile_count = thresholds::DESERT_TILE_NORMAL;
        m.tile_center = (0, 10);
        m.calculate_zones(100.0, true, false);
        assert!(!m.zone_sandstorm);
        assert!(m.zone_rain);

        // A sandstorm underground does not count (needs the surface).
        let mut m = SceneMetrics::new();
        m.desert_sand_tile_count = thresholds::DESERT_TILE_NORMAL;
        m.tile_center = (0, 500);
        m.calculate_zones(100.0, false, true);
        assert!(!m.zone_sandstorm);
    }

    // --- oceanDepths / isDesertBiomeSand (WorldGen.cs:4447, TileID.cs:501) ---

    #[test]
    fn ocean_depths_beach_bands() {
        // worldSurface=100, rockLayer=180 -> oceanLevel = 180.
        // Left band: x < 380; right band: x > W - 380.
        assert!(ocean_depths(100.0, 180.0, 379, 1000, 100));
        assert!(ocean_depths(100.0, 180.0, 621, 1000, 100));
        assert!(!ocean_depths(100.0, 180.0, 380, 1000, 100));
        assert!(!ocean_depths(100.0, 180.0, 620, 1000, 100));
        // Below oceanLevel the bands do not count.
        assert!(!ocean_depths(100.0, 180.0, 100, 1000, 181));
        // Exactly at oceanLevel still counts (`>` comparison).
        assert!(ocean_depths(100.0, 180.0, 100, 1000, 180));
    }

    #[test]
    fn desert_biome_sand_set_matches_tile_ids() {
        // `TileID.cs:501`: CreateBoolSet(53, 397, 396, 400, 403, 401).
        for id in [53u16, 397, 396, 400, 403, 401] {
            assert!(is_desert_biome_sand(id));
        }
        for id in [112u16, 234, 398, 399, 402, 147] {
            assert!(!is_desert_biome_sand(id));
        }
    }

    // --- Sandstorm (Events/Sandstorm.cs) - DW-1.2 ---

    #[test]
    fn test_DW_1_2_severity_approaches_at_0_003() {
        // `Sandstorm.cs:109-110`: +0.003 per tick toward the intention.
        let mut s = Sandstorm::new(1);
        s.severity = 0.0;
        s.intended_severity = 1.0;
        s.update_severity();
        assert!((s.severity - 0.003).abs() < 1e-6);
        for _ in 0..99 {
            s.update_severity();
        }
        assert!((s.severity - 0.3).abs() < 1e-4);
    }

    #[test]
    fn test_DW_1_2_severity_snaps_on_overshoot() {
        // `Sandstorm.cs:111-114`: crossing the target snaps to it.
        let mut s = Sandstorm::new(1);
        s.severity = 0.0;
        s.intended_severity = 0.002;
        s.update_severity();
        assert_eq!(s.severity, 0.002);

        // Downward as well.
        let mut s = Sandstorm::new(1);
        s.severity = 0.001;
        s.intended_severity = 0.0;
        s.update_severity();
        assert_eq!(s.severity, 0.0);
    }

    #[test]
    fn test_DW_1_2_severity_clamped_0_1() {
        // `Sandstorm.cs:110`: MathHelper.Clamp(..., 0, 1).
        let mut s = Sandstorm::new(1);
        s.severity = 0.999;
        s.intended_severity = 2.0;
        s.update_severity();
        assert_eq!(s.severity, 1.0);

        let mut s = Sandstorm::new(1);
        s.severity = 0.001;
        s.intended_severity = -1.0;
        s.update_severity();
        assert_eq!(s.severity, 0.0);
    }

    #[test]
    fn test_DW_1_2_nan_severity_resets() {
        // `Sandstorm.cs:101-108`.
        let mut s = Sandstorm::new(1);
        s.severity = f32::NAN;
        s.intended_severity = 0.5;
        s.update_severity();
        assert!((s.severity - 0.003).abs() < 1e-6);

        let mut s = Sandstorm::new(1);
        s.severity = 0.5;
        s.intended_severity = f32::NAN;
        s.update_severity();
        // Intended reset to 0: severity walks down from 0.5 by 0.003.
        assert!((s.severity - (0.5 - 0.003)).abs() < 1e-5);
    }

    #[test]
    fn test_DW_1_2_time_left_decay_and_stop() {
        // `Sandstorm.cs:42-46`: `-dayRate`, plus `-15*dayRate` when the wind
        // is too weak (0 < |wind| < 0.6).
        let mut s = Sandstorm::new(1);
        s.happening = true;
        s.time_left = 100;
        s.update_time(1, 0.5);
        assert_eq!(s.time_left, 84); // 100 - 1 - 15

        let mut s = Sandstorm::new(1);
        s.happening = true;
        s.time_left = 100;
        s.update_time(1, 0.7);
        assert_eq!(s.time_left, 99);

        // `Sandstorm.cs:47-54`: wind exactly 0 zeroes TimeLeft (`:47-50`) and
        // the `<= 0` check stops the storm.
        let mut s = Sandstorm::new(1);
        s.happening = true;
        s.time_left = 100;
        s.update_time(1, 0.0);
        assert!(!s.happening);
        assert_eq!(s.time_left, 0);
    }

    #[test]
    fn test_DW_1_2_time_left_over_maximum_resets_first() {
        // `Sandstorm.cs:38-41`: a TimeLeft above the maximum is zeroed at
        // entry, so the next tick stops the storm.
        let mut s = Sandstorm::new(1);
        s.happening = true;
        s.time_left = 86401;
        s.update_time(1, 0.7);
        assert!(!s.happening);
        assert_eq!(s.time_left, 0);
    }

    #[test]
    fn test_DW_1_2_start_sets_duration_range_and_intent() {
        // `Sandstorm.cs:117-122`: duration 28800..=86400, intent 0.4..1.4.
        for seed in 0..8u64 {
            let mut s = Sandstorm::new(seed);
            s.start_sandstorm();
            assert!(s.happening);
            assert!((SANDSTORM_DURATION_MIN..=SANDSTORM_DURATION_MAX).contains(&s.time_left));
            assert!((0.4..1.4).contains(&s.intended_severity));
        }
    }

    #[test]
    fn test_DW_1_2_has_sufficient_wind_0_6() {
        // `Sandstorm.cs:22-25`: `|wind| >= 0.6`.
        assert!(Sandstorm::has_sufficient_wind(0.6));
        assert!(Sandstorm::has_sufficient_wind(-0.6));
        assert!(Sandstorm::has_sufficient_wind(0.8));
        assert!(!Sandstorm::has_sufficient_wind(0.599));
        assert!(!Sandstorm::has_sufficient_wind(0.0));
    }

    #[test]
    fn storm_starts_from_the_roll_branch_deterministically() {
        // `Sandstorm.cs:56-69`: with wind, one `Next(64800)` roll per
        // day-rate tick; a huge day rate starts the storm.
        let mut s = Sandstorm::new(42);
        s.update_time(70000, 0.8);
        assert!(s.happening);
        assert!((SANDSTORM_DURATION_MIN..=SANDSTORM_DURATION_MAX).contains(&s.time_left));

        // Same seed -> same outcome.
        let mut s2 = Sandstorm::new(42);
        s2.update_time(70000, 0.8);
        assert_eq!(s2.happening, s.happening);
        assert_eq!(s2.time_left, s.time_left);
        assert_eq!(s2.severity.to_bits(), s.severity.to_bits());
    }

    #[test]
    fn show_sandstorm_visuals_gate() {
        // `Sandstorm.cs:131-138`.
        let mut s = Sandstorm::new(1);
        s.happening = true;
        assert!(s.show_sandstorm_visuals(true, true, 49));
        assert!(!s.show_sandstorm_visuals(true, true, 50)); // bgDelay < 50
        assert!(!s.show_sandstorm_visuals(true, false, 0)); // not a desert bg
        assert!(!s.show_sandstorm_visuals(false, true, 0)); // not happening
    }

    // --- SceneState (SceneState.cs) - DW-1.3 ---

    #[test]
    fn test_DW_1_3_move_towards_inc_dec() {
        // `SceneState.cs:479-491`.
        let state = SceneState::new();
        let mut v = 0.0;
        // `new()` resets with skipTransitions = true: the first call snaps.
        state.move_towards(&mut v, 1.0, 0.005);
        assert_eq!(v, 1.0);

        // After the first update the transitions are live.
        let mut state = SceneState::new();
        let metrics = SceneMetrics::new();
        state.update(&metrics);
        let mut v = 0.0;
        state.move_towards(&mut v, 1.0, 0.005);
        assert!((v - 0.005).abs() < 1e-7);
        state.move_towards(&mut v, 1.0, 0.005);
        assert!((v - 0.01).abs() < 1e-7);
        // Approaching from below never overshoots the target.
        let mut v = 0.999;
        state.move_towards(&mut v, 1.0, 0.005);
        assert_eq!(v, 1.0);
        // Separate inc/dec overload.
        let mut v = 2.0;
        state.move_towards_inc_dec(&mut v, 0.0, 0.02, 0.1);
        assert!((v - 1.9).abs() < 1e-6);
        // Decrement never overshoots the target.
        let mut v = 0.999;
        state.move_towards_inc_dec(&mut v, 1.0, 0.02, 0.1);
        assert_eq!(v, 1.0);
    }

    #[test]
    fn test_DW_1_3_skip_transitions_snaps_then_lives() {
        // `SceneState.cs:53` sets skipTransitions on Reset, `:62` clears it
        // after the first Update.
        let mut state = SceneState::new();
        let mut metrics = SceneMetrics::new();
        metrics.behind_backwall = true;
        state.update(&metrics);
        // The first update ran with skip=true: the backing snapped to -0.1.
        assert_eq!(state.outside_weather_effect_intensity_backing, -0.1);
        assert_eq!(state.outside_weather_effect_intensity, 0.0);
        assert!(!state.skip_transitions);
        // The second update fades at 0.005/tick from where it snapped.
        state.outside_weather_effect_intensity_backing = 0.0;
        state.update(&metrics);
        assert!((state.outside_weather_effect_intensity_backing - (-0.005)).abs() < 1e-6);
        assert!((state.outside_weather_effect_intensity - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_DW_1_3_outside_weather_backwall_fade() {
        // `SceneState.cs:134-135`.
        let mut state = SceneState::new();
        let mut metrics = SceneMetrics::new();
        state.update(&metrics); // clear skipTransitions
        metrics.behind_backwall = true;
        for _ in 0..240 {
            state.update(&metrics);
        }
        // Backing settles at -0.1; the intensity clamps to 0.
        assert!((state.outside_weather_effect_intensity_backing - (-0.1)).abs() < 1e-6);
        assert_eq!(state.outside_weather_effect_intensity, 0.0);
        metrics.behind_backwall = false;
        for _ in 0..300 {
            state.update(&metrics);
        }
        // Backing settles at 1.1; the intensity clamps to 1.
        assert!((state.outside_weather_effect_intensity_backing - 1.1).abs() < 1e-5);
        assert!((state.outside_weather_effect_intensity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_DW_1_3_graveyard_intensity_lerp_rates() {
        // `SceneState.cs:368-371`: target = GetLerpValue(16, 36, count),
        // in at 0.02/tick, out at 0.1/tick.
        let mut state = SceneState::new();
        let mut metrics = SceneMetrics::new();
        state.update(&metrics); // clear skipTransitions

        metrics.graveyard_tile_count = thresholds::GRAVEYARD_TILE_MAX;
        state.update(&metrics);
        assert!((state.graveyard_visual_intensity - 0.02).abs() < 1e-6);

        for _ in 0..49 {
            state.update(&metrics);
        }
        assert!((state.graveyard_visual_intensity - 1.0).abs() < 1e-5);

        // Halfway target at count 26.
        metrics.graveyard_tile_count = 26;
        for _ in 0..200 {
            state.update(&metrics);
        }
        assert!((state.graveyard_visual_intensity - 0.5).abs() < 1e-5);

        // Zero graves: fades back out at 0.1/tick.
        metrics.graveyard_tile_count = 0;
        state.update(&metrics);
        assert!((state.graveyard_visual_intensity - 0.4).abs() < 1e-5);
    }

    #[test]
    fn light_decay_fades_to_one() {
        // `SceneState.cs:255-281`: targets 1/1 (shimmer and FTW terms are 0).
        let mut state = SceneState::new();
        state.air_light_decay = 0.9;
        state.solid_light_decay = 0.5;
        let metrics = SceneMetrics::new();
        state.update(&metrics); // skip=true: snap to target
        assert_eq!(state.air_light_decay, 1.0);
        assert_eq!(state.solid_light_decay, 1.0);
    }

    // --- TR_ forces - DW-1.4 ---

    #[test]
    fn test_DW_1_4_parse_env_force() {
        assert_eq!(
            parse_env_force("TR_SANDSTORM=0.75"),
            Some(("SANDSTORM", 0.75))
        );
        assert_eq!(parse_env_force("TR_BLOODMOON"), Some(("BLOODMOON", 1.0)));
        assert_eq!(parse_env_force("TR_BLOODMOON="), Some(("BLOODMOON", 1.0)));
        assert_eq!(parse_env_force("TR_GRAVEYARD=1"), Some(("GRAVEYARD", 1.0)));
        // Clamped to 0..1.
        assert_eq!(parse_env_force("TR_SANDSTORM=5"), Some(("SANDSTORM", 1.0)));
        assert_eq!(parse_env_force("TR_SANDSTORM=-3"), Some(("SANDSTORM", 0.0)));
        // Junk is rejected, not defaulted.
        assert_eq!(parse_env_force("TR_SANDSTORM=abc"), None);
        assert_eq!(parse_env_force("TR_SANDSTORM=nan"), None);
        assert_eq!(parse_env_force("WRONG_X=1"), None);
        assert_eq!(parse_env_force("TR_=1"), None);
    }

    #[test]
    fn test_DW_1_4_force_pins_severity_and_graveyard() {
        // SANDSTORM: the pin lands on the storm via force_severity.
        let mut state = SceneState::new();
        assert!(state.force("SANDSTORM", 0.75));
        assert_eq!(state.forced_severity, Some(0.75));
        let mut storm = Sandstorm::new(1);
        storm.force_severity(state.forced_severity.unwrap());
        assert!(storm.happening);
        for _ in 0..50 {
            storm.update_severity();
        }
        assert_eq!(storm.severity, 0.75);

        // Zero severity pins "no storm".
        let mut storm = Sandstorm::new(1);
        storm.force_severity(0.0);
        assert!(!storm.happening);

        // GRAVEYARD: update() holds the forced value against the fade.
        let mut state = SceneState::new();
        assert!(state.force("GRAVEYARD", 0.8));
        let metrics = SceneMetrics::new();
        state.update(&metrics);
        assert!((state.graveyard_visual_intensity - 0.8).abs() < 1e-6);
        for _ in 0..10 {
            state.update(&metrics);
        }
        assert!((state.graveyard_visual_intensity - 0.8).abs() < 1e-6);

        // BLOODMOON: boolean flag.
        let mut state = SceneState::new();
        assert!(state.force("BLOODMOON", 1.0));
        assert!(state.blood_moon);
        assert!(state.force("BLOODMOON", 0.0));
        assert!(!state.blood_moon);

        // Unknown names are refused.
        let mut state = SceneState::new();
        assert!(!state.force("SOMETHINGELSE", 1.0));
    }

    #[test]
    fn reset_restores_vanilla_defaults() {
        // `SceneState.cs:44-54`.
        let state = SceneState::new();
        assert_eq!(state.air_light_decay, 1.0);
        assert_eq!(state.solid_light_decay, 1.0);
        assert_eq!(state.outside_weather_effect_intensity, 1.0);
        assert_eq!(state.outside_weather_effect_intensity_backing, 1.0);
        assert_eq!(state.graveyard_visual_intensity, 0.0);
        assert!(state.skip_transitions);
        assert!(!state.blood_moon);
    }

    #[test]
    fn get_lerp_value_matches_utils() {
        // `Utils.cs:315-343` clamped form.
        assert_eq!(get_lerp_value(16.0, 36.0, 16.0, true), 0.0);
        assert_eq!(get_lerp_value(16.0, 36.0, 36.0, true), 1.0);
        assert_eq!(get_lerp_value(16.0, 36.0, 26.0, true), 0.5);
        assert_eq!(get_lerp_value(16.0, 36.0, 0.0, true), 0.0);
        assert_eq!(get_lerp_value(16.0, 36.0, 100.0, true), 1.0);
    }
}
