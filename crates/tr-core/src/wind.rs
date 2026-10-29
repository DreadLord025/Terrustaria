//! Global wind (`Main.windSpeedCurrent`, `Main.windSpeedTarget`, `Main.windCounter`)
//! and the per-tile sway phase (`TileDrawing.GetWindCycle`).
//!
//! Source:
//! * `Main.UpdateWeather` — `decompiled/Terraria/Main.cs:58737-58950`.
//! * `Main.ResetWindCounter` — `Main.cs:58698-58706`.
//! * `Main.PostContentLoadInitialize` — `Main.cs:10864-10881`.
//! * `TileDrawing.Update` — `GameContent/Drawing/TileDrawing.cs:367-386`.
//! * `TileDrawing.GetWindCycle` — `TileDrawing.cs:7676-7700`.
//!
//! Deliberate simplifications:
//! * The original draws randomness from `Main.rand` (`UnifiedRandom`) for the
//!   target walk and from `FastRandom.CreateWithRandomSeed()` in
//!   `ResetWindCounter`. A private xorshift64 replaces both; the *order* of the
//!   draws is the original one, so a seed fully determines the history.
//! * This module is the single-player/server branch. Client sync, the menu
//!   pause, the creative-power freeze and Lantern Night are folded into
//!   [`Wind::frozen`]; when frozen only the current speed still eases towards
//!   the target, exactly as in `UpdateWeather`.
//! * `update(raining)` thresholds `Main.maxRaining` to 1.0/0.0. Use
//!   [`Wind::update_rain`] to pass the real 0..1 intensity.
//! * `world_surface` defaults to +infinity, which is treated as "not
//!   configured yet": the altitude gate in `GetWindCycle` is skipped until the
//!   caller supplies `Main.worldSurface`.
//! * The C# wind counters are `double`; the public counter helper uses `f32` as
//!   specified. This only loses fractional phase after very long sessions.
//!
//! There is no `windSpeedSet` in the 1.4.5.8 decompile; the second persistent
//! counter used by the walk is `Main.extremeWindCounter` (`Main.cs:1269`,
//! 58893-58932), ported here as `extreme_counter`.

/// `6.2831854820251465` — the float-widened 2π literal used by `GetWindCycle`
/// (`TileDrawing.cs:7682`), not `f64::TAU`.
const WIND_TAU: f64 = 6.283_185_482_025_146_5;

/// `TileDrawing._grassWindCounter` / `_treeWindCounter` live in `double`; this is
/// the smallest period of sway (`TileDrawing.cs:371-373`).
const GRASS_PERIOD: f32 = 180.0;
const TREE_PERIOD: f32 = 240.0;

/// Deterministic stand-in for `Main.rand` / `FastRandom`.
///
/// xorshift64 with a splitmix64-derived seed so that seed 0 is valid. The
/// original RNGs are not reproducible from a `u32`, so the exact stream is
/// intentionally different; only the draw order and ranges mirror the C#.
#[derive(Clone, Debug)]
struct Rng(u64);

impl Rng {
    fn new(seed: u32) -> Self {
        let mut z = (seed as u64).wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        if z == 0 {
            z = 0x2545_F491_4F6C_DD1D;
        }
        Self(z)
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// `UnifiedRandom.Next(min, max)`: `min` inclusive, `max` exclusive.
    fn next_int(&mut self, min: i32, max: i32) -> i32 {
        debug_assert!(max > min);
        let range = (max - min) as u64;
        min + (self.next_u64() % range) as i32
    }
}

/// `Main.WindSpeedCurrent` / `Main.WindSpeedTarget` / `Main.windCounter` /
/// `Main.extremeWindCounter` plus the state `GetWindCycle` needs from the world.
#[derive(Clone, Debug)]
pub struct Wind {
    /// `Main.windSpeedCurrent` (also `Main.WindForVisuals`, `Main.cs:2823`).
    pub speed_current: f32,
    /// `Main.windSpeedTarget`.
    pub speed_target: f32,
    /// `Main.windCounter`: ticks until the next target random walk.
    counter: i32,
    /// `Main.extremeWindCounter`: ticks until the next target re-roll.
    extreme_counter: i32,
    rng: Rng,
    /// `Main.worldSurface`, in tile rows. `+inf` disables the altitude gate.
    pub world_surface: f64,
    /// `Main.remixWorld`: flips the altitude gate.
    pub remix_world: bool,
    /// `Main.SettingsEnabled_TilesSwayInWind` (`TileDrawing.cs:7678`).
    pub sway_enabled: bool,
    /// Player with `statLifeMax >= 120` exists (`flag`, `Main.cs:58863-58871`).
    /// While false the walk caps `|speed_target|` at 0.35.
    pub any_player_120_life: bool,
    /// Creative wind freeze, Lantern Night, client/menu: skip the target walk.
    pub frozen: bool,
}

impl Wind {
    /// Deterministic start: `PostContentLoadInitialize` picks
    /// `windSpeedCurrent` in `[-0.8, 0.8]` and copies it to the target
    /// (`Main.cs:10874-10875`); world setup then runs
    /// `ResetWindCounter(resetExtreme: true)` (`WorldGen.cs:6949`,
    /// `Main.cs:58698-58706`).
    pub fn new(seed: u32) -> Self {
        let mut rng = Rng::new(seed);
        let speed_current = rng.next_int(-800, 801) as f32 * 0.001;
        let counter = rng.next_int(900, 2701);
        let extreme_counter = rng.next_int(10, 31);
        Self {
            speed_current,
            speed_target: speed_current,
            counter,
            extreme_counter,
            rng,
            world_surface: f64::INFINITY,
            remix_world: false,
            sway_enabled: true,
            any_player_120_life: true,
            frozen: false,
        }
    }

    /// One 60 Hz `UpdateWeather` tick. `raining` is `Main.maxRaining != 0`.
    #[inline]
    pub fn update(&mut self, raining: bool) {
        self.update_rain(if raining { 1.0 } else { 0.0 });
    }

    /// One 60 Hz `UpdateWeather` tick with the real `Main.maxRaining` value.
    ///
    /// `Main.cs:58830-58849`:
    /// ```text
    /// step    = 0.0003 + |target*(1 + 5/9*rain) - current| * 0.0015
    /// current = move_towards(current, target*(1 + 5/9*rain), step)
    /// ```
    /// followed by the `windCounter` target walk (`Main.cs:58850-58950`).
    pub fn update_rain(&mut self, max_raining: f32) {
        // `float num6 = 0.8f; float num7 = 0.0003f;`
        const MAX_TARGET: f32 = 0.8;
        let mut step = 0.0003_f32;
        // `float num8 = Main.windSpeedTarget * (1f + 5f / 9f * Main.maxRaining);`
        let effective_target = self.speed_target * (1.0 + 5.0 / 9.0 * max_raining);
        // `num7 += Math.Abs(num8 - Main.windSpeedCurrent) * 0.0015f;`
        step += (effective_target - self.speed_current).abs() * 0.0015;

        if self.speed_current < effective_target {
            self.speed_current += step;
            if self.speed_current > effective_target {
                self.speed_current = effective_target;
            }
        } else if self.speed_current > effective_target {
            self.speed_current -= step;
            if self.speed_current < effective_target {
                self.speed_current = effective_target;
            }
        }

        // `if (Main.netMode == 1 || (Main.netMode != 2 && Main.gameMenu)) return;`
        // `if (!CreativePower...Enabled) { if (LanternNight.LanternsUp) return; ... }`
        if self.frozen {
            return;
        }

        // `Main.windCounter--; if (Main.windCounter <= 0) { ... }`
        self.counter -= 1;
        if self.counter <= 0 {
            // `bool flag` — some active player has `statLifeMax >= 120`.
            let strong_wind = self.any_player_120_life;
            // `float num9 = (windSpeedTarget < 0) ? -1 : 1` (sign before the walk).
            let direction = if self.speed_target < 0.0 { -1.0 } else { 1.0 };

            // `Main.rand.Next(4)`, else `Next(2)`, else the ±100 branch.
            self.speed_target += if self.rng.next_int(0, 4) == 0 {
                self.rng.next_int(-25, 26) as f32 * 0.001
            } else if self.rng.next_int(0, 2) == 0 {
                self.rng.next_int(-50, 51) as f32 * 0.001
            } else {
                self.rng.next_int(-100, 101) as f32 * 0.001
            };
            if !strong_wind && self.speed_target.abs() > 0.35 {
                self.speed_target = 0.35 * self.speed_target.signum();
            }

            // `Main.extremeWindCounter--; if (<= 0) { ResetWindCounter(true); ... }`
            self.extreme_counter -= 1;
            if self.extreme_counter <= 0 {
                self.reset_wind_counter(true);
                if self.rng.next_int(0, 30) < 13 {
                    if self.rng.next_int(0, 2) == 0 {
                        self.speed_target = 0.0;
                        // Direct override after ResetWindCounter.
                        self.counter = self.rng.next_int(7200, 28801);
                    } else {
                        self.speed_target = self.rng.next_int(-200, 201) as f32 * 0.001;
                    }
                } else if self.rng.next_int(0, 20) < 13 {
                    self.speed_target = self.rng.next_int(-400, 401) as f32 * 0.001;
                } else {
                    self.speed_target = self.rng.next_int(-850, 851) as f32 * 0.001;
                }
                if !strong_wind && self.speed_target.abs() > 0.35 {
                    self.speed_target = 0.35 * self.speed_target.signum();
                }
                if self.speed_target.abs() > 0.3 {
                    self.extreme_counter += self.rng.next_int(5, 11);
                }
                if self.speed_target.abs() > 0.5 {
                    self.extreme_counter += self.rng.next_int(10, 21);
                }
                if self.speed_target.abs() > 0.7 {
                    self.extreme_counter += self.rng.next_int(15, 31);
                }
            } else {
                self.reset_wind_counter(false);
            }

            // `if (rand.Next(3) != 0 && direction flipped) target *= -1f;`
            if self.rng.next_int(0, 3) != 0
                && ((direction < 0.0 && self.speed_target > 0.0)
                    || (direction > 0.0 && self.speed_target < 0.0))
            {
                self.speed_target *= -1.0;
            }
        }

        // `if (target > 0.8) target = 0.8; if (target < -0.8) target = -0.8;`
        if self.speed_target > MAX_TARGET {
            self.speed_target = MAX_TARGET;
        }
        if self.speed_target < -MAX_TARGET {
            self.speed_target = -MAX_TARGET;
        }
    }

    /// `Main.ResetWindCounter`: `windCounter = rand(900, 2701)` and optionally
    /// `extremeWindCounter = rand(10, 31)` (`Main.cs:58698-58706`).
    fn reset_wind_counter(&mut self, reset_extreme: bool) {
        self.counter = self.rng.next_int(900, 2701);
        if reset_extreme {
            self.extreme_counter = self.rng.next_int(10, 31);
        }
    }

    /// `TileDrawing.GetWindCycle` (`TileDrawing.cs:7676-7700`).
    ///
    /// `counter` is `_grassWindCounter` for tiles and `_treeWindCounter` for
    /// canopies (advance with [`advance_counter`] / [`advance_tree_counter`]).
    /// Returns 0 outside the surface band or when sway is disabled.
    ///
    /// The raw result is `(0.5*cos(...) + wind) * lerp`, so at
    /// `|wind| = 0.8` it can reach ±1.3. That is the C# range; only
    /// `speed_target` is clamped to ±0.8.
    pub fn cycle(&self, x: i32, y: i32, counter: f32) -> f32 {
        // `if (!Main.SettingsEnabled_TilesSwayInWind) return 0f;`
        if !self.sway_enabled {
            return 0.0;
        }

        // The original computes the cosine first, then applies the altitude gate.
        let offset = x as f32 * 0.5 + (y / 100) as f32 * 0.5;
        let mut num = (counter as f64 * WIND_TAU + offset as f64).cos() as f32 * 0.5;

        if self.remix_world {
            // `if (!(y > Main.worldSurface)) return 0f;`
            if self.world_surface.is_finite() && !((y as f64) > self.world_surface) {
                return 0.0;
            }
        } else {
            // `if (!(y < Main.worldSurface)) return 0f;`
            if self.world_surface.is_finite() && !((y as f64) < self.world_surface) {
                return 0.0;
            }
        }

        // `num += Main.WindForVisuals;`
        num += self.speed_current;

        // `return num * Utils.GetLerpValue(0.08f, 0.18f, |wind|, clamped: true);`
        num * get_lerp_value(0.08, 0.18, self.speed_current.abs(), true)
    }

    /// `Main.windCounter` (ticks until the next random walk).
    pub fn wind_counter(&self) -> i32 {
        self.counter
    }

    /// `Main.extremeWindCounter` (ticks until the next full re-roll).
    pub fn extreme_wind_counter(&self) -> i32 {
        self.extreme_counter
    }
}

/// `TileDrawing.Update`, grass/plant counter (`TileDrawing.cs:371-373`):
/// `counter += 1/180 + 1/180 * num * 4`, where
/// `num = Utils.GetLerpValue(0.08, 1.2, |Main.WindForVisuals|, clamped)`.
pub fn advance_counter(counter: &mut f32, wind_speed: f32) {
    let num = visual_wind_amount(wind_speed);
    *counter += 1.0 / GRASS_PERIOD + 1.0 / GRASS_PERIOD * num * 4.0;
}

/// `TileDrawing.Update`, tree-canopy counter (`TileDrawing.cs:372`):
/// `counter += 1/240 + 1/240 * num * 2`, same `num` as [`advance_counter`].
pub fn advance_tree_counter(counter: &mut f32, wind_speed: f32) {
    let num = visual_wind_amount(wind_speed);
    *counter += 1.0 / TREE_PERIOD + 1.0 / TREE_PERIOD * num * 2.0;
}

/// `Utils.GetLerpValue(0.08f, 1.2f, |Main.WindForVisuals|, clamped: true)`.
///
/// This is the `num` in `TileDrawing.Update` (`TileDrawing.cs:371`), not the
/// raw wind speed: it is 0 below 0.08, rises linearly and saturates at 1 above
/// 1.2. Full storms (`speed_current ≈ 1.24`) therefore use the same counter
/// rate as `|wind| = 1.2`.
#[inline]
pub fn visual_wind_amount(wind_speed: f32) -> f32 {
    get_lerp_value(0.08, 1.2, wind_speed.abs(), true)
}

/// `Utils.GetLerpValue` (`Utils.cs:315-343`), both branches of the clamped form.
#[inline]
fn get_lerp_value(from: f32, to: f32, t: f32, clamped: bool) -> f32 {
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

    /// `0.8 * (1 + 5/9)` — `windSpeedCurrent` follows the rain-scaled target,
    /// which is not clamped to ±0.8 in the C# (`Main.cs:58832-58849`).
    const MAX_RAIN_CURRENT: f32 = 0.8 * (1.0 + 5.0 / 9.0);

    #[test]
    fn new_is_deterministic_and_in_source_ranges() {
        let a = Wind::new(0x1234_5678);
        let b = Wind::new(0x1234_5678);
        assert_eq!(a.speed_current.to_bits(), b.speed_current.to_bits());
        assert_eq!(a.speed_target.to_bits(), b.speed_target.to_bits());
        assert_eq!(a.wind_counter(), b.wind_counter());
        assert_eq!(a.extreme_wind_counter(), b.extreme_wind_counter());
        assert!(a.speed_current.abs() <= 0.8);
        assert!((900..=2700).contains(&a.wind_counter()));
        assert!((10..=30).contains(&a.extreme_wind_counter()));
    }

    #[test]
    fn dry_wind_stays_within_08() {
        let mut wind = Wind::new(0x5EED);
        for _ in 0..200_000 {
            wind.update(false);
            assert!(
                wind.speed_current.abs() <= 0.8 + 1e-5,
                "current {}",
                wind.speed_current
            );
            assert!(
                wind.speed_target.abs() <= 0.8 + 1e-5,
                "target {}",
                wind.speed_target
            );
        }
    }

    #[test]
    fn target_and_rain_scaled_current_stay_bounded() {
        let mut wind = Wind::new(0xBADF00D);
        for tick in 0..200_000 {
            let raining = tick % 2400 < 600;
            wind.update(raining);
            assert!(wind.speed_target.abs() <= 0.8 + 1e-5);
            assert!(
                wind.speed_current.abs() <= MAX_RAIN_CURRENT + 1e-4,
                "current {} at tick {tick}",
                wind.speed_current
            );
        }
    }

    #[test]
    fn rain_increases_abs_current() {
        let mut dry = Wind::new(7);
        let mut wet = Wind::new(7);
        let mut wet_negative = Wind::new(7);
        for w in [&mut dry, &mut wet, &mut wet_negative] {
            w.speed_target = 0.4;
            w.speed_current = 0.0;
            // Freeze the target walk so only the rain factor differs.
            w.frozen = true;
        }
        wet_negative.speed_target = -0.4;
        for _ in 0..20_000 {
            dry.update(false);
            wet.update(true);
            wet_negative.update(true);
        }
        let expected = 0.4 * (1.0 + 5.0 / 9.0);
        assert!((dry.speed_current - 0.4).abs() < 1e-5);
        assert!((wet.speed_current - expected).abs() < 1e-5);
        assert!((wet_negative.speed_current + expected).abs() < 1e-5);
        assert!(wet.speed_current.abs() > dry.speed_current.abs());
        assert!(wet_negative.speed_current.abs() > dry.speed_current.abs());
    }

    #[test]
    fn advance_counter_matches_grass_formula() {
        // (wind speed, expected `num` after the 0.08..1.2 clamped lerp).
        let cases = [
            (0.0_f32, 0.0_f32),
            (0.08, 0.0),
            (0.6, (0.6 - 0.08) / 1.12),
            (1.2, 1.0),
            (2.0, 1.0),
        ];
        for (wind_speed, num) in cases {
            let mut counter = 0.0_f32;
            advance_counter(&mut counter, wind_speed);
            let expected = 1.0 / 180.0 + 1.0 / 180.0 * num * 4.0;
            assert!(
                (counter - expected).abs() < 1e-7,
                "speed {wind_speed}: {counter} != {expected}"
            );
        }

        // 180 ticks at full sway add 180 * (5/180) = 5.
        let mut counter = 0.25_f32;
        for _ in 0..180 {
            advance_counter(&mut counter, 1.2);
        }
        assert!((counter - 5.25).abs() < 1e-4);
    }

    #[test]
    fn advance_tree_counter_matches_tree_formula() {
        let mut counter = 0.0_f32;
        for _ in 0..240 {
            advance_tree_counter(&mut counter, 1.2);
        }
        // 240 * (3/240) = 3.
        assert!((counter - 3.0).abs() < 1e-4);
    }

    #[test]
    fn cycle_stays_within_source_bounds() {
        let mut wind = Wind::new(11);
        for speed in [-0.8_f32, -0.5, -0.2, 0.0, 0.2, 0.5, 0.8] {
            wind.speed_current = speed;
            for x in [-123, -1, 0, 1, 57] {
                for y in [0, 100, 250] {
                    for k in 0..16 {
                        let counter = k as f32 * 0.125;
                        let v = wind.cycle(x, y, counter);
                        // cos term is ±0.5; the lerp saturates at 1.
                        assert!(
                            v.abs() <= speed.abs() + 0.5 + 1e-5,
                            "speed {speed} x {x} y {y} c {counter}: {v}"
                        );
                        // `0.5*cos + wind` stays inside [-1, 1] up to |wind|=0.5,
                        // which covers calm/normal weather.
                        if speed.abs() <= 0.5 {
                            assert!(v.abs() <= 1.0 + 1e-5, "speed {speed}: {v}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn cycle_varies_with_x_y_and_counter() {
        let mut wind = Wind::new(3);
        wind.speed_current = 0.3;
        let base = wind.cycle(0, 0, 0.0);
        assert!((wind.cycle(1, 0, 0.0) - base).abs() > 1e-3);
        // `y / 100` is integer division, so the phase changes every 100 rows.
        assert!((wind.cycle(0, 100, 0.0) - base).abs() > 1e-3);
        assert!((wind.cycle(0, 0, 0.25) - base).abs() > 1e-3);
        assert_eq!(wind.cycle(0, 42, 0.0), wind.cycle(0, 0, 0.0));
    }

    #[test]
    fn cycle_respects_surface_and_sway_setting() {
        let mut wind = Wind::new(5);
        wind.speed_current = 0.5;
        wind.world_surface = 100.0;
        assert_eq!(wind.cycle(0, 100, 0.0), 0.0);
        assert_eq!(wind.cycle(0, 150, 0.0), 0.0);
        assert_ne!(wind.cycle(0, 99, 0.0), 0.0);

        wind.remix_world = true;
        assert_eq!(wind.cycle(0, 99, 0.0), 0.0);
        assert_ne!(wind.cycle(0, 101, 0.0), 0.0);

        wind.sway_enabled = false;
        assert_eq!(wind.cycle(0, 101, 0.0), 0.0);
    }

    #[test]
    fn same_seed_same_history() {
        let mut a = Wind::new(0xDEAD_BEEF);
        let mut b = Wind::new(0xDEAD_BEEF);
        for tick in 0..50_000 {
            let raining = (tick / 300) % 5 == 0;
            a.update(raining);
            b.update(raining);
            assert_eq!(a.speed_current.to_bits(), b.speed_current.to_bits());
            assert_eq!(a.speed_target.to_bits(), b.speed_target.to_bits());
            assert_eq!(a.wind_counter(), b.wind_counter());
            assert_eq!(a.extreme_wind_counter(), b.extreme_wind_counter());
        }
    }
}
