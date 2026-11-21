//! W4-c39 — `AI_166_Dove` (`Projectile.cs:47540-47610`), the aiStyle-166
//! machine the released dove (851) and the lantern-night floating lantern
//! (855) share. The lantern arm (`flag = type == 855`) bobs on a sin of its
//! remaining life, drifts with the wind, casts a warm light and fades out
//! over its last 90 ticks; the dove arm rises at a capped −3 and clamps X.
//!
//! The bounce is `HandleMovement`'s per-type arm (`Projectile.cs:18347-18361`
//! for the lantern, `:18336-18346` for the dove): the blocked axes reverse,
//! the lantern's upward bounce damped ×0.25. The port's generic movement
//! integrates the velocity; the bounce rides [`Projectile::dove_bounce`].
//!
//! RNG: the one-shot `frame = Main.rand.Next(4)` latch (`:47546`) on the
//! `UpdateProjectiles` stream; the rest of the arm draws nothing. The
//! `Lighting.AddLight` (`:47569`) is the frame's entity-light feed
//! (`proj_lights`), skipped here (the pool has no light layer — ANALYSIS).

use tr_core::loot::DropRng;

use crate::projectile::{Projectile, RELEASE_DOVES, RELEASE_LANTERN};

impl Projectile {
    /// `AI_166_Dove` (`Projectile.cs:47540-47610`). `wind` is
    /// `Main.WindForVisuals` (= `windSpeedCurrent`, `Main.cs:2823`).
    pub(crate) fn ai_166_dove(&mut self, rng: &mut dyn DropRng, wind: f32) {
        let lantern = self.kind == RELEASE_LANTERN;
        // `:47543-47547` — the one-shot frame latch.
        if self.local_ai[0] == 0.0 {
            self.local_ai[0] = 1.0;
            self.frame = rng.next_range(0, 4);
        }
        // `:47548-47560` — the frame cadence: 6 ticks for the lantern, 4 for
        // the dove; the frame wraps at 4.
        let num = if lantern { 6 } else { 4 };
        self.frame_counter += 1;
        if self.frame_counter > num {
            self.frame_counter = 0;
            self.frame += 1;
            if self.frame > 3 {
                self.frame = 0;
            }
        }
        // `:47561-47565` — the facing follows the X velocity.
        self.sprite_direction = -1;
        if self.vel.x > 0.0 {
            self.sprite_direction = 1;
        }
        if lantern {
            // `:47566-47591` — the lantern arm. The light cast (`:47569`,
            // `(1, 0.6, 0.1) * (1 - alpha/255)`) is the frame's entity-light
            // feed — skipped here (no pool light layer).
            self.rot = self.vel.x * 0.2;
            if self.vel.y < -1.0 {
                self.vel.y *= 0.99;
            } else {
                self.vel.y +=
                    ((self.time_left % 200) as f32 / 200.0 * std::f32::consts::TAU).sin() * 0.25;
                if self.vel.y > 0.5 {
                    self.vel.y *= 0.8;
                }
                self.vel.y = self.vel.y.clamp(-1.5, 1.5);
            }
            let mut w = wind;
            if w == 0.0 {
                w = 0.1;
            }
            self.vel.x += 0.0525 * w;
            self.vel.x = self.vel.x.clamp(-2.0, 2.0);
        } else {
            // `:47592-47601` — the dove arm: the capped rise.
            self.rot = self.vel.x * 0.15;
            self.vel.y -= 0.025;
            if self.vel.y < -3.0 {
                self.vel.y = -3.0;
            }
            self.vel.x = self.vel.x.clamp(-5.0, 5.0);
        }
        // `:47602-47609` — the last-90-tick fade and the alpha kill.
        if self.time_left < 90 {
            self.alpha += 3;
        }
        if self.alpha >= 255 {
            self.kill();
        }
    }

    /// `HandleMovement`'s lantern bounce (`Projectile.cs:18347-18361`): the
    /// blocked X reverses outright; the blocked Y reverses, damped ×0.25 when
    /// the pre-bounce Y was upward (negative). `last` is the pre-collision
    /// velocity; `hit_x`/`hit_y` the blocked axes (the TileCollision resolution
    /// zeroes them — the port's per-axis probe).
    pub(crate) fn lantern_bounce(&mut self, last: glam::Vec2, hit_x: bool, hit_y: bool) {
        if hit_x {
            self.vel.x = -last.x;
        }
        if hit_y {
            self.vel.y = -last.y;
            if last.y < 0.0 {
                self.vel.y *= 0.25;
            }
        }
    }

    /// `HandleMovement`'s dove bounce (`Projectile.cs:18336-18346`): the
    /// blocked X reverses ×0.8, the blocked Y ×0.5.
    pub(crate) fn dove_bounce(&mut self, last: glam::Vec2, hit_x: bool, hit_y: bool) {
        if hit_x {
            self.vel.x = -last.x * 0.8;
        }
        if hit_y {
            self.vel.y = -last.y * 0.5;
        }
    }

    /// The aiStyle-166 movement: the per-axis tile probe, the bounce, then the
    /// integrate (`UpdatePosition`, `:20445`). The lantern and dove keep
    /// `tileCollide = true` (the stock), so a blocked move bounces rather than
    /// kills.
    pub(crate) fn dove_movement(&mut self, world: &tr_core::World) {
        if !self.tile_collide {
            self.pos += self.vel;
            return;
        }
        let last = self.vel;
        let hit_x = crate::player::collides_rect(
            world,
            self.pos.x + last.x,
            self.pos.y,
            self.w,
            self.h,
        );
        let hit_y = crate::player::collides_rect(
            world,
            self.pos.x,
            self.pos.y + last.y,
            self.w,
            self.h,
        );
        if hit_x {
            self.vel.x = 0.0;
        }
        if hit_y {
            self.vel.y = 0.0;
        }
        if self.kind == RELEASE_LANTERN {
            self.lantern_bounce(last, hit_x, hit_y);
        } else if self.kind == RELEASE_DOVES {
            self.dove_bounce(last, hit_x, hit_y);
        }
        self.pos += self.vel;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::proj_data;

    /// A counting rng: the frame latch draws exactly one `Next(4)`.
    struct CountRng {
        draws: Vec<(i32, i32)>,
    }
    impl DropRng for CountRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.draws.push((min, max));
            min
        }
        fn next_float(&mut self) -> f32 {
            0.0
        }
    }

    fn lantern() -> Projectile {
        let mut p = Projectile::default();
        p.kind = RELEASE_LANTERN;
        p.active = true;
        let d = proj_data(RELEASE_LANTERN).unwrap();
        p.w = d.w;
        p.h = d.h;
        p.time_left = d.time_left;
        p.tile_collide = d.tile_collide;
        p
    }

    /// W4-c39 — the one-shot frame latch draws one `Next(4)` (`:47546`), then
    /// the lantern's frame cadence is 6 ticks (`:47548-47552`).
    #[test]
    fn w4_c39_the_lantern_frame_latch_draws_once() {
        let mut p = lantern();
        let mut rng = CountRng { draws: Vec::new() };
        p.ai_166_dove(&mut rng, 0.0);
        assert_eq!(rng.draws, vec![(0, 4)], "one Next(4) on the first tick");
        assert_eq!(p.local_ai[0], 1.0);
        // The next ticks draw nothing (the latch is set).
        for _ in 0..6 {
            p.ai_166_dove(&mut rng, 0.0);
        }
        assert_eq!(rng.draws.len(), 1, "no further draws");
        // After 7 ticks the frame counter wrapped once (cadence 6: the 7th
        // tick bumps the frame).
        assert_eq!(p.frame_counter, 0);
        assert_eq!(p.frame, 1);
    }

    /// W4-c39 — the lantern's last-90-tick fade (`:47602-47609`): alpha +3
    /// per tick, the kill at 255.
    #[test]
    fn w4_c39_the_lantern_fades_and_dies_at_alpha_255() {
        let mut p = lantern();
        p.local_ai[0] = 1.0; // skip the latch draw
        p.time_left = 89;
        p.alpha = 252;
        let mut rng = CountRng { draws: Vec::new() };
        p.ai_166_dove(&mut rng, 0.0);
        assert_eq!(p.alpha, 255);
        assert!(!p.active, "alpha >= 255 kills");
    }

    /// W4-c39 — the lantern bounce (`:18347-18361`): X reverses outright, Y
    /// reverses damped ×0.25 when the pre-bounce Y was upward.
    #[test]
    fn w4_c39_the_lantern_bounce_damps_the_upward_y() {
        let mut p = lantern();
        // Upward Y (negative): the bounce damps ×0.25.
        p.lantern_bounce(glam::Vec2::new(2.0, -4.0), true, true);
        assert!((p.vel.x - (-2.0)).abs() < 1e-5);
        assert!((p.vel.y - (4.0 * 0.25)).abs() < 1e-5, "upward Y damped: {}", p.vel.y);
        // Downward Y (positive): the bounce reverses outright.
        p.lantern_bounce(glam::Vec2::new(2.0, 4.0), false, true);
        assert!((p.vel.y - (-4.0)).abs() < 1e-5);
    }

    /// W4-c39 — the dove bounce (`:18336-18346`): X ×0.8, Y ×0.5.
    #[test]
    fn w4_c39_the_dove_bounce_scales_the_axes() {
        let mut p = Projectile::default();
        p.kind = RELEASE_DOVES;
        p.dove_bounce(glam::Vec2::new(4.0, -6.0), true, true);
        assert!((p.vel.x - (-4.0 * 0.8)).abs() < 1e-5);
        assert!((p.vel.y - (6.0 * 0.5)).abs() < 1e-5);
    }
}
