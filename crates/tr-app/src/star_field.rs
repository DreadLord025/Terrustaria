//! `Terraria.Star` (`Star.cs`) — the live 200..400-slot star field
//! (W4-g70 DOC-SWEEP lane 8). The static BG-11/16 field is retired: the
//! pool spawns through `Star.SpawnStars` (`:86-158`), ticks through
//! `Star.Update`/`UpdateStars` (`:171-317`), rolls the night's starfall
//! boost at dusk (`NightSetup`, `:41-60`) and converts a sky star into a
//! shooting-star trail when the starfall spawner calls `StarFall`
//! (`:62-84`). The draw side (`Main.DrawStarsInBackground`/`DrawStar`,
//! `Main.cs:63030-63173`) lives in `render::sky`.
//!
//! RNG contract: the C# spawns seed a fresh OS-random `FastRandom` per
//! `SpawnStars` call (`FastRandom.CreateWithRandomSeed`, `Star.cs:88`);
//! the port pins each call's stream to the world seed mixed with a
//! monotonic spawn counter so screenshots stay deterministic (the
//! recorded deviation, ANALYSIS). `Fall`'s two rolls ride `Main.rand` —
//! the caller threads the stage stream (`UpdateProjectiles` for the
//! spawner's call, `Main.cs:17775`).

use tr_core::tile_light::FastRandom;

/// One `Star` (`Star.cs:7-39`). Positions live in the original's
/// 1920x1200 design space (`Main.MaxWorldViewSize`, `Main.cs:66714`).
#[derive(Clone, Copy, Debug)]
pub struct Star {
    pub x: f32,
    pub y: f32,
    pub scale: f32,
    pub rotation: f32,
    /// `Star.type` — the `Star_0..3` sheet index.
    pub tex: usize,
    pub twinkle: f32,
    pub twinkle_speed: f32,
    pub rotation_speed: f32,
    /// `Star.falling` — the shooting-star trail mode.
    pub falling: bool,
    /// `Star.hidden` — retired below y 1500 (`Star.cs:177-180`).
    pub hidden: bool,
    pub fall_speed_x: f32,
    pub fall_speed_y: f32,
    pub fall_time: i32,
    /// `Star.fadeIn` — 1 on a mid-session respawn, decaying at
    /// `6.1728395E-05 * dayRate * 10` per tick (`Star.cs:207-214`).
    pub fade_in: f32,
}

impl Default for Star {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            scale: 0.0,
            rotation: 0.0,
            tex: 0,
            twinkle: 0.0,
            twinkle_speed: 0.0,
            rotation_speed: 0.0,
            falling: false,
            hidden: false,
            fall_speed_x: 0.0,
            fall_speed_y: 0.0,
            fall_time: 0,
            fade_in: 0.0,
        }
    }
}

/// The star field: `Main.star[200]` + `Main.numStars` (`Main.cs:1251`)
/// plus the `Star` statics (`dayCheck`/`starfallBoost`/`starFallCount`,
/// `Star.cs:33-37`).
pub struct StarField {
    pub stars: Vec<Star>,
    /// `Star.dayCheck` (`Star.cs:33`): the decompile never sets it true
    /// (the only writes are the two `false` resets, `:245`/`:321`), so the
    /// noon respawn scan runs on EVERY day tick past 27000 — kept verbatim.
    pub day_check: bool,
    /// `Star.starfallBoost` (`Star.cs:35`): the night's meteor-rate
    /// multiplier (`WorldGen.cs:71834`).
    pub starfall_boost: f32,
    /// `Star.starFallCount` (`Star.cs:37`).
    pub star_fall_count: i32,
    /// The spawn-stream counter: each `SpawnStars` call mixes it into the
    /// world seed (the OS-seed stand-in, module doc).
    spawn_counter: u64,
    /// The world seed the spawn streams pin to.
    seed: u64,
}

impl StarField {
    /// `Main.Initialize`'s `Star.SpawnStars()` (`Main.cs:6675`): the full
    /// field at boot. `seed` pins the per-call FastRandom streams.
    pub fn new(seed: u64) -> Self {
        let mut f = Self {
            stars: Vec::new(),
            day_check: false,
            starfall_boost: 1.0,
            star_fall_count: 0,
            spawn_counter: 0,
            seed,
        };
        f.spawn_stars(None);
        f
    }

    /// The per-call spawn stream: `FastRandom.CreateWithRandomSeed()`
    /// (`Star.cs:88`) is OS-random; the port mixes the world seed with the
    /// call counter (the make_stars deviation, kept).
    fn spawn_rng(&mut self) -> FastRandom {
        self.spawn_counter = self.spawn_counter.wrapping_add(1);
        FastRandom::new(self.seed ^ self.spawn_counter.wrapping_mul(0xA076_1D64_78BD_642F))
    }

    /// `Star.SpawnStars(int s = -1)` (`Star.cs:86-158`). `None` rolls the
    /// 200..400 count and respawns the whole field; `Some(i)` respawns one
    /// slot with the 10-candidate farthest-from-neighbours X pick
    /// (`:100-125`) and `fadeIn = 1`.
    pub fn spawn_stars(&mut self, s: Option<usize>) {
        let mut rng = self.spawn_rng();
        let (lo, hi) = match s {
            None => {
                let count = rng.next_range(200, 400) as usize;
                self.stars = vec![Star::default(); count];
                (0, count)
            }
            Some(i) => (i, i + 1),
        };
        for i in lo..hi {
            // `:99`: the C# installs the fresh Star into the slot FIRST, so
            // the neighbour scan below sees a default (0,0) star at slot i
            // (excluded by `k != s`) — the port mirrors the install order.
            let mut star = Star::default();
            self.stars[i] = star;
            if s.is_some() {
                star.fade_in = 1.0;
                // `:103-125`: ten candidate X draws; keep the one whose
                // nearest live neighbour is farthest (the `num5 = -2000`
                // start makes the first candidate always win slot 0).
                let mut best_gap = -2000i32;
                for _ in 0..10 {
                    let cand = rng.next_range(0, 1921);
                    let mut nearest = 2000i32;
                    for (k, other) in self.stars.iter().enumerate() {
                        if k != i && !other.hidden && !other.falling {
                            let d = (cand as f32 - other.x).abs() as i32;
                            if d < nearest {
                                nearest = d;
                            }
                        }
                    }
                    if i == 0 || nearest > best_gap {
                        best_gap = nearest;
                        star.x = cand as f32;
                    }
                }
            } else {
                star.x = rng.next_range(0, 1921) as f32;
            }
            // `:131-152`: the shared attribute recipe, in its RNG order.
            star.y = rng.next_range(0, 1201) as f32;
            star.rotation = rng.next_range(0, 628) as f32 * 0.01;
            star.scale = rng.next_range(70, 130) as f32 * 0.006;
            star.tex = rng.next_range(0, 4) as usize;
            star.twinkle = rng.next_range(60, 101) as f32 * 0.01;
            star.twinkle_speed = rng.next_range(30, 110) as f32 * 0.0001;
            if rng.next_range(0, 2) == 0 {
                star.twinkle_speed = -star.twinkle_speed;
            }
            star.rotation_speed = rng.next_range(5, 50) as f32 * 0.0001;
            if rng.next_range(0, 2) == 0 {
                star.rotation_speed = -star.rotation_speed;
            }
            if rng.next_range(0, 40) == 0 {
                star.scale *= 2.0;
                star.twinkle_speed /= 2.0;
                star.rotation_speed /= 2.0;
            }
            self.stars[i] = star;
        }
    }

    /// `Star.NightSetup` (`Star.cs:41-60`), called from
    /// `UpdateTime_StartNight` (`Main.cs:65223`): the boost rolls 1-in-10
    /// (1-in-5 on the anniversary) for 3..5×, else 1-in-3 (1-in-2) for
    /// 1..1.5×; the fall count resets. `rng` is the `UpdateTime` stream.
    pub fn night_setup(&mut self, rng: &mut tr_core::UnifiedRandom, anniversary: bool) {
        self.starfall_boost = 1.0;
        let (max1, max2) = if anniversary { (5, 2) } else { (10, 3) };
        if rng.next_range(0, max1) == 0 {
            self.starfall_boost = rng.next_range(300, 501) as f32 * 0.01;
        } else if rng.next_range(0, max2) == 0 {
            self.starfall_boost = rng.next_range(100, 151) as f32 * 0.01;
        }
        self.star_fall_count = 0;
    }

    /// `Star.StarFall(float positionX)` (`Star.cs:62-84`): the nearest
    /// live non-falling star to the spawner's world X (mapped into the
    /// 1920 design space through `rightWorld`, `WorldGen.cs:6554`) starts
    /// falling. `rng` is the caller's stage stream (`Fall`'s two rolls,
    /// `Star.cs:166-167`). `any_secret_seed` gates the Fall
    /// (`Star.cs:162` — the port has no starGame).
    pub fn star_fall(
        &mut self,
        position_x: f32,
        right_world: f32,
        rng: &mut tr_core::UnifiedRandom,
        any_secret_seed: bool,
    ) {
        self.star_fall_count += 1;
        let target = position_x / right_world * 1920.0;
        let mut best: Option<usize> = None;
        let mut best_d = -1.0f32;
        for (i, s) in self.stars.iter().enumerate() {
            if !s.hidden && !s.falling {
                let d = (s.x - target).abs();
                if best_d == -1.0 || d < best_d {
                    best = Some(i);
                    best_d = d;
                }
            }
        }
        if let Some(i) = best {
            // `Star.Fall` (`:160-169`): the secret-seed gate, then the two
            // rolls — Y first (7..10), then X (−4..4).
            if !any_secret_seed {
                let s = &mut self.stars[i];
                s.fall_time = 0;
                s.falling = true;
                s.fall_speed_y = rng.next_range(700, 1001) as f32 * 0.01;
                s.fall_speed_x = rng.next_range(-400, 401) as f32 * 0.01;
            }
        }
    }

    /// `Star.Update` (`Star.cs:171-235`) for one star, at the given day
    /// rate. The falling branch integrates the trail and retires below
    /// y 1500; the idle branch decays `fadeIn` and oscillates the twinkle
    /// (0.6..1, speed sign-flip at the rails) and the rotation (6.28 wrap).
    fn update_star(s: &mut Star, day_rate: i32) {
        if s.falling && !s.hidden {
            s.fall_time += day_rate;
            let k = (day_rate + 99) as f32 / 100.0;
            s.x += s.fall_speed_x * k;
            s.y += s.fall_speed_y * k;
            if s.y > 1500.0 {
                s.hidden = true;
            }
            s.twinkle += s.twinkle_speed * 3.0;
            if s.twinkle > 1.0 {
                s.twinkle = 1.0;
                s.twinkle_speed *= -1.0;
            } else if (s.twinkle as f64) < 0.6 {
                s.twinkle = 0.6;
                s.twinkle_speed *= -1.0;
            }
            s.rotation += 0.5;
            if (s.rotation as f64) > 6.28 {
                s.rotation -= 6.28;
            }
            if s.rotation < 0.0 {
                s.rotation += 6.28;
            }
            return;
        }
        if s.fade_in > 0.0 {
            s.fade_in -= 6.1728395E-05 * day_rate as f32 * 10.0;
            if s.fade_in < 0.0 {
                s.fade_in = 0.0;
            }
        }
        s.twinkle += s.twinkle_speed;
        if s.twinkle > 1.0 {
            s.twinkle = 1.0;
            s.twinkle_speed *= -1.0;
        } else if (s.twinkle as f64) < 0.6 {
            s.twinkle = 0.6;
            s.twinkle_speed *= -1.0;
        }
        s.rotation += s.rotation_speed;
        if (s.rotation as f64) > 6.28 {
            s.rotation -= 6.28;
        }
        if s.rotation < 0.0 {
            s.rotation += 6.28;
        }
    }

    /// `Star.UpdateStars` (`Star.cs:237-317`), called once per tick from
    /// `Main.cs:17253` (the surface gate is the caller's). The noon
    /// respawn scan (`:247-256`) runs while `dayCheck` is false — which
    /// the decompile never sets true, so every day tick past 27000
    /// respawns the hidden slots (the verbatim quirk). The secret-seed
    /// menu gravity block (`:261-316`) needs `gameMenu && generatingWorld`
    /// — unreachable in the port's frame loop, documented out.
    pub fn update_stars(&mut self, day_time: bool, time: f64, day_rate: i32) {
        if !day_time {
            self.day_check = false;
        } else if !self.day_check && time >= 27000.0 {
            for i in 0..self.stars.len() {
                if self.stars[i].hidden {
                    self.spawn_stars(Some(i));
                }
            }
        }
        for s in &mut self.stars {
            Self::update_star(s, day_rate);
        }
    }
}

// ---------------------------------------------------------------------------
// Witnesses — WRITTEN NOT RUN (code-only lane). Expectations hand-derived
// from the cited C#; run with `cargo test -p tr-app star_field`.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_count_and_recipe_are_reproducible() {
        // `Star.cs:89`: 200..400 stars; the port's pinned streams make two
        // fields off one seed identical.
        let a = StarField::new(0xDEAD_BEEF);
        let b = StarField::new(0xDEAD_BEEF);
        assert!((200..400).contains(&a.stars.len()));
        assert_eq!(a.stars.len(), b.stars.len());
        for (sa, sb) in a.stars.iter().zip(b.stars.iter()) {
            assert_eq!(sa.x, sb.x);
            assert_eq!(sa.tex, sb.tex);
            assert!((0.42..1.56).contains(&sa.scale)); // 70..129 × 0.006, ×2 rare
            assert!((0.6..=1.0).contains(&sa.twinkle));
        }
    }

    #[test]
    fn twinkle_bounces_at_the_rails() {
        // `Star.cs:215-225`: +speed per tick; >1 clamps to 1 and flips the
        // speed; <0.6 clamps to 0.6 and flips.
        let mut s = Star { twinkle: 0.99, twinkle_speed: 0.05, ..Default::default() };
        StarField::update_star(&mut s, 1);
        assert_eq!(s.twinkle, 1.0);
        assert_eq!(s.twinkle_speed, -0.05);
        StarField::update_star(&mut s, 1);
        assert!((s.twinkle - 0.95).abs() < 1e-6);
        s.twinkle = 0.61;
        StarField::update_star(&mut s, 1);
        assert_eq!(s.twinkle, 0.6);
        assert_eq!(s.twinkle_speed, 0.05);
    }

    #[test]
    fn falling_star_integrates_and_retires_below_1500() {
        // `Star.cs:173-180`: position += fallSpeed * (dayRate + 99) / 100;
        // y > 1500 hides. At dayRate 1 the factor is exactly 1.
        let mut s = Star {
            falling: true,
            y: 1499.0,
            fall_speed_y: 7.0,
            fall_speed_x: -2.0,
            twinkle: 0.8,
            twinkle_speed: 0.01,
            ..Default::default()
        };
        StarField::update_star(&mut s, 1);
        assert_eq!(s.y, 1506.0);
        assert!(s.hidden);
        assert_eq!(s.fall_time, 1);
        // The falling twinkle runs at 3× speed (`:185`).
        assert!((s.twinkle - 0.83).abs() < 1e-6);
        // The rotation advances 0.5 per tick (`:196`), not rotationSpeed.
        assert_eq!(s.rotation, 0.5);
    }

    #[test]
    fn fade_in_decays_at_the_cadence() {
        // `Star.cs:207-214`: −6.1728395E-05 * dayRate * 10 per tick.
        let mut s = Star { fade_in: 1.0, twinkle: 0.8, ..Default::default() };
        StarField::update_star(&mut s, 1);
        assert!((s.fade_in - (1.0 - 6.1728395E-05 * 10.0)).abs() < 1e-9);
    }

    #[test]
    fn star_fall_picks_the_nearest_live_star() {
        // `Star.cs:62-84`: the design-space X = positionX / rightWorld *
        // 1920; the nearest non-hidden non-falling star falls.
        let mut f = StarField::new(7);
        f.stars = vec![
            Star { x: 100.0, ..Default::default() },
            Star { x: 900.0, ..Default::default() },
            Star { x: 950.0, hidden: true, ..Default::default() },
        ];
        // rightWorld 19200 → target = 9600/19200*1920 = 960 → slot 1
        // (the hidden 950 is skipped).
        let mut rng = tr_core::UnifiedRandom::new(3);
        f.star_fall(9600.0, 19200.0, &mut rng, false);
        assert_eq!(f.star_fall_count, 1);
        assert!(!f.stars[0].falling);
        assert!(f.stars[1].falling);
        assert!(!f.stars[2].falling);
        // Fall's rolls: Y in 7..=10, X in −4..=4 (`Star.cs:166-167`,
        // Next(700,1001)×0.01 / Next(-400,401)×0.01).
        assert!((7.0..=10.0).contains(&f.stars[1].fall_speed_y));
        assert!((-4.0..=4.0).contains(&f.stars[1].fall_speed_x));
    }

    #[test]
    fn secret_seeds_block_the_fall_but_count_it() {
        // `Star.cs:162`: `!anySecretSeedIsActive || starGame` — the port
        // has no starGame, so a secret seed skips the Fall body; the
        // count still bumps (`:64`).
        let mut f = StarField::new(7);
        f.stars = vec![Star { x: 100.0, ..Default::default() }];
        let mut rng = tr_core::UnifiedRandom::new(3);
        f.star_fall(1000.0, 19200.0, &mut rng, true);
        assert_eq!(f.star_fall_count, 1);
        assert!(!f.stars[0].falling);
    }

    #[test]
    fn night_setup_boost_windows() {
        // `Star.cs:41-60`: the 1-in-10 → 3..5× window, else 1-in-3 →
        // 1..1.5×. A stream pinned to hit the first window: Next(10) == 0
        // then Next(300,501).
        let mut f = StarField::new(1);
        // Scan seeds for one whose first Next(10) is 0 (the boost path).
        let mut hit = None;
        for seed in 0..1000 {
            let mut rng = tr_core::UnifiedRandom::new(seed);
            if rng.next_range(0, 10) == 0 {
                hit = Some(seed);
                break;
            }
        }
        let seed = hit.expect("a 1-in-10 seed exists below 1000");
        let mut rng = tr_core::UnifiedRandom::new(seed);
        f.night_setup(&mut rng, false);
        assert!((3.0..=5.0).contains(&f.starfall_boost));
        assert_eq!(f.star_fall_count, 0);
        // The anniversary narrows the first window to 1-in-5.
        let mut hit5 = None;
        for seed in 0..1000 {
            let mut rng = tr_core::UnifiedRandom::new(seed);
            if rng.next_range(0, 5) == 0 {
                hit5 = Some(seed);
                break;
            }
        }
        let mut rng = tr_core::UnifiedRandom::new(hit5.unwrap());
        f.night_setup(&mut rng, true);
        assert!((3.0..=5.0).contains(&f.starfall_boost));
    }

    #[test]
    fn noon_respawn_scan_replaces_hidden_slots() {
        // `Star.cs:247-256`: dayCheck never sets true in the decompile, so
        // every day tick past 27000 respawns hidden slots with fadeIn 1.
        let mut f = StarField::new(11);
        let n = f.stars.len();
        f.stars[3].hidden = true;
        f.update_stars(true, 27000.0, 1);
        assert!(!f.stars[3].hidden);
        // The respawn sets fadeIn 1, then the SAME call's twinkle walk decays
        // it one tick (`Star.cs:207-214`): 1 - 6.1728395E-05*1*10.
        assert!((f.stars[3].fade_in - (1.0 - 6.1728395E-05 * 10.0)).abs() < 1e-9);
        assert_eq!(f.stars.len(), n);
        // Night resets dayCheck and runs no respawn.
        f.stars[4].hidden = true;
        f.update_stars(false, 10000.0, 1);
        assert!(f.stars[4].hidden);
    }
}
