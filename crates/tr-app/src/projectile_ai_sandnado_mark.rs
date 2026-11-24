//! W4-d13 — `AI()`'s `else if (this.aiStyle == 128)` arm
//! (`Projectile.cs:36086-36197`): the hostile Sandnado's spawn mark (658,
//! `ProjectileID.SandnadoHostileMark`, `ProjectileID.cs:1870`). The
//! `SetDefaults` row is `:7071-7082` (14x14, penetrate 1 — the stock,
//! timeLeft 900, tileCollide false, ignoreWater, alpha 255, hostile). The
//! mark spins up, fades in, spawns the hostile sandnado (657) at tick 60,
//! fades out and dies at tick 120.
//!
//! **RNG census (by construction).** Every explicit `Main.rand` draw stays
//! on the stream in the C#'s order; the `Dust.NewDust` INTERNAL rolls belong
//! to the dust pool and are SKIPPED (the S1-D-4b-7 precedent); the dust
//! bodies are skipped (no dust layer). Per tick:
//! * the spawn plume (`localAI[1] < 30`, `:36094-36117`): one dust —
//!   `NextFloat` (`:36100`, the lerp amount), `NextFloat` (`:36101`, the X
//!   lerp), then — only when `vector163.X > -1.2` — `NextFloat` (`:36113`,
//!   velocity X), then `NextFloat` (`:36115`, velocity Y).
//! * the two orbit-dust loops (`:36152-36177`): 2 iterations each, a
//!   `Next(10) == 0` gate per iteration (the dust body draws nothing).
//! The tick-60 sandnado spawn (`:36184-36192`) draws NOTHING.
//!
//! Deviations (the standing pool precedents, ANALYSIS): the
//! `SoundID.Item60` cue (`:36092`) rides `wants_sound`; the
//! `Lighting.AddLight` (`:36150`) has no per-frame port layer; the
//! `Main.expertMode` read rides the owner's `buffs.expert_mode` (the
//! `damage_pve` precedent). The `Kill()` dust burst (`Projectile.cs:72374-
//! 72393`) is a separate death-hook census — see [`sandnado_mark_kill_draws`].

use glam::Vec2;
use tr_core::loot::DropRng;

use crate::projectile::{Projectile, opacity_of};

/// `MathHelper.Lerp(value1, value2, amount)` (XNA): `value1 + (value2 -
/// value1) * amount` — NOT `Utils.GetLerpValue` (the port's
/// `get_lerp_value` is the inverse ramp).
fn math_helper_lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// `Kill()` (`Projectile.cs:70644-70648`) — the sibling-module deactivation
/// (the `projectile_ai_spear` precedent; `Projectile::kill` is private).
fn kill(p: &mut Projectile) {
    p.kill_time_left = p.time_left;
    p.active = false;
    p.time_left = 0;
}

/// The tick-60 spawn intent (`Projectile.cs:36184-36192`): the hostile
/// sandnado (657) at the mark's Centre, zero velocity, damage 30 (expert
/// 22), knockback 3, the mark's owner.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MarkSpawn {
    pub x: f32,
    pub y: f32,
    pub damage: i32,
}

impl Projectile {
    /// `AI()`'s aiStyle-128 arm (`Projectile.cs:36086-36197`). `expert` is
    /// `Main.expertMode` (the tick-60 damage pick); `local_owner` gates the
    /// spawn (`owner == Main.myPlayer`). Returns the tick-60 spawn intent
    /// (the pool resolves it — the arm cannot see the slot array). The
    /// movement is the dispatch's free integrate (`tileCollide = false`);
    /// the arm pins velocity to zero.
    pub(crate) fn ai_sandnado_mark(
        &mut self,
        rng: &mut dyn DropRng,
        expert: bool,
        local_owner: u8,
    ) -> Option<MarkSpawn> {
        use std::f32::consts::TAU;
        let mut spawn = None;
        // `:36089-36093` — the one-shot Item60 cue at the Centre.
        if self.sound_delay == 0 {
            self.sound_delay = -1;
            let (cx, cy) = self.center();
            self.wants_sound = Some((2, 60, cx, cy, 0.0));
        }
        // `:36094-36117` — the spawn plume (localAI[1] < 30), one dust.
        if self.local_ai[1] < 30.0 {
            for _ in 0..1 {
                let value15 = -0.5f32;
                let value16 = 0.9f32;
                let amount4 = rng.next_float(); // `:36100`
                let mut vector163 = Vec2::new(
                    math_helper_lerp(0.1, 1.0, rng.next_float()), // `:36101`
                    math_helper_lerp(value15, value16, amount4),
                );
                vector163.x *= math_helper_lerp(2.2, 0.6, amount4);
                vector163.x *= -1.0;
                // The dust body is skipped; the conditional draws stay.
                if vector163.x > -1.2 {
                    let _ = rng.next_float(); // `:36113` velocity X
                }
                let _ = rng.next_float(); // `:36115` velocity Y
            }
        }
        // `:36118-36124` — the first-tick latch: the 0.8 scale base, the
        // facing, the Centre snap to the tile grid.
        if self.local_ai[0] == 0.0 {
            self.local_ai[0] = 0.8;
            self.direction = 1;
            let center = Vec2::new(self.pos.x + self.w * 0.5, self.pos.y + self.h * 0.5);
            let point9_x = center.x as i32 / 16;
            let point9_y = center.y as i32 / 16;
            let snapped = Vec2::new(point9_x as f32 * 16.0 + 8.0, point9_y as f32 * 16.0 + 8.0);
            self.pos = snapped - Vec2::new(self.w * 0.5, self.h * 0.5);
        }
        // `:36125` — the spin: localAI[1]/40 of a full turn, facing-signed.
        self.rot = self.local_ai[1] / 40.0 * TAU * self.direction as f32;
        // `:36126-36136` — the fade-in (localAI[1] < 33): alpha -8, floor 0.
        if self.local_ai[1] < 33.0 {
            if self.alpha > 0 {
                self.alpha -= 8;
            }
            if self.alpha < 0 {
                self.alpha = 0;
            }
        }
        // `:36137-36147` — the fade-out (localAI[1] > 103): alpha +16, cap 255.
        if self.local_ai[1] > 103.0 {
            if self.alpha < 255 {
                self.alpha += 16;
            }
            if self.alpha > 255 {
                self.alpha = 255;
            }
        }
        // `:36148-36151` — the AddLight at full opacity: no port layer.
        // `:36152-36164` — the first orbit-dust loop (rotation-applied).
        for _ in 0..2 {
            if rng.next_range(0, 10) == 0 {
                // The 267 dust body is skipped (no explicit draw).
            }
        }
        // `:36165-36177` — the second orbit-dust loop (no rotation).
        for _ in 0..2 {
            if rng.next_range(0, 10) == 0 {
                // The 267 dust body is skipped (no explicit draw).
            }
        }
        // `:36178-36181` — the scale ramp outside the hold window.
        if self.local_ai[1] < 33.0 || self.local_ai[1] > 87.0 {
            self.scale = opacity_of(self.alpha) / 2.0 * self.local_ai[0];
        }
        // `:36182-36183` — the velocity pin + the age tick.
        self.vel = Vec2::ZERO;
        self.local_ai[1] += 1.0;
        // `:36184-36192` — the tick-60 sandnado spawn (local-owner only).
        if self.local_ai[1] == 60.0 && self.owner == local_owner {
            let num966 = if expert { 22 } else { 30 };
            let (cx, cy) = self.center();
            spawn = Some(MarkSpawn { x: cx, y: cy, damage: num966 });
        }
        // `:36193-36196` — the 120-tick life.
        if self.local_ai[1] >= 120.0 {
            kill(self);
        }
        spawn
    }
}

/// `Kill()`'s type-658 dust burst (`Projectile.cs:72374-72393`) as the DRAW
/// CENSUS: 10 dust 269, each a `Next(3) == 0` gate; on a 0 TWO
/// `Next(-10, 11)` (fadeIn `:72381` + scale `:72382`), else ONE
/// `Next(-10, 11)` (scale `:72387`). The dust bodies are skipped; the
/// explicit draws stay on the stream in the C#'s order.
pub(crate) fn sandnado_mark_kill_draws(rng: &mut dyn DropRng) {
    for _ in 0..10 {
        if rng.next_range(0, 3) == 0 {
            let _ = rng.next_range(-10, 11); // fadeIn `:72381`
            let _ = rng.next_range(-10, 11); // scale `:72382`
        } else {
            let _ = rng.next_range(-10, 11); // scale `:72387`
        }
    }
}

#[cfg(test)]
mod w4_d13_style128_tests {
    use super::*;
    use crate::projectile::{SANDNADO_MARK, proj_data};

    fn mark(pos: Vec2) -> Projectile {
        let mut p = Projectile::default();
        p.active = true;
        p.kind = SANDNADO_MARK;
        p.owner = 0;
        p.w = 14.0;
        p.h = 14.0;
        p.pos = pos;
        p.alpha = 255;
        p.tile_collide = false;
        p
    }

    struct CensusRng {
        ranges: Vec<(i32, i32)>,
        floats: usize,
    }

    impl DropRng for CensusRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.ranges.push((min, max));
            min
        }
        fn next_float(&mut self) -> f32 {
            self.floats += 1;
            0.0
        }
    }

    /// The row's `SetDefaults` slice (`Projectile.cs:7071-7082`).
    #[test]
    fn w4_d13_the_mark_row_carries_the_csharp_set_defaults() {
        let d = proj_data(SANDNADO_MARK).unwrap();
        assert_eq!((d.w, d.h), (14.0, 14.0));
        assert_eq!(d.ai_style, 128);
        assert_eq!(d.penetrate, 1, "the stock penetrate");
        assert_eq!(d.time_left, 900);
        assert_eq!(d.alpha, 255);
        assert!(!d.tile_collide);
        assert!(d.ignore_water);
    }

    /// `:36118-36125` — the first-tick latch: the 0.8 scale base, the
    /// facing, the tile-grid Centre snap, the spin off localAI[1].
    #[test]
    fn w4_d13_the_first_tick_snaps_to_the_tile_grid() {
        let mut p = mark(Vec2::new(100.0, 100.0)); // centre (107, 107)
        let mut rng = CensusRng { ranges: Vec::new(), floats: 0 };
        let _ = p.ai_sandnado_mark(&mut rng, false, 0);
        assert_eq!(p.local_ai[0], 0.8, "the scale base");
        assert_eq!(p.direction, 1);
        // centre tile (6, 6) → snap (104, 104); pos = (97, 97).
        assert_eq!(p.pos, Vec2::new(97.0, 97.0), "the tile-grid snap");
        assert_eq!(p.local_ai[1], 1.0, "the age tick");
        assert_eq!(p.sound_delay, -1, "the Item60 latch");
        assert_eq!(p.wants_sound.map(|r| (r.0, r.1)), Some((2, 60)));
    }

    /// `:36184-36192` — the tick-60 spawn: damage 30 (expert 22) at the
    /// Centre, local-owner only.
    #[test]
    fn w4_d13_the_tick_60_spawn_picks_the_expert_damage() {
        let mut p = mark(Vec2::new(104.0 - 7.0, 104.0 - 7.0));
        p.local_ai[0] = 0.8; // skip the latch
        p.local_ai[1] = 59.0; // ++ → 60
        p.sound_delay = -1;
        let mut rng = CensusRng { ranges: Vec::new(), floats: 0 };
        let spawn = p.ai_sandnado_mark(&mut rng, false, 0).expect("the spawn");
        assert_eq!(spawn.damage, 30, "the normal damage");
        assert_eq!((spawn.x, spawn.y), (104.0, 104.0), "the Centre anchor");
        // Expert: 22.
        let mut q = mark(Vec2::new(97.0, 97.0));
        q.local_ai[0] = 0.8;
        q.local_ai[1] = 59.0;
        q.sound_delay = -1;
        let spawn = q.ai_sandnado_mark(&mut rng, true, 0).expect("the spawn");
        assert_eq!(spawn.damage, 22, "the expert damage");
        // A foreign owner spawns nothing.
        let mut r = mark(Vec2::new(97.0, 97.0));
        r.owner = 255;
        r.local_ai[0] = 0.8;
        r.local_ai[1] = 59.0;
        r.sound_delay = -1;
        assert!(r.ai_sandnado_mark(&mut rng, false, 0).is_none());
    }

    /// `:36193-36196` — the 120-tick life kill.
    #[test]
    fn w4_d13_the_mark_dies_at_120() {
        let mut p = mark(Vec2::new(97.0, 97.0));
        p.local_ai[0] = 0.8;
        p.local_ai[1] = 119.0; // ++ → 120
        p.sound_delay = -1;
        let mut rng = CensusRng { ranges: Vec::new(), floats: 0 };
        let _ = p.ai_sandnado_mark(&mut rng, false, 0);
        assert!(!p.active, "the 120 kill");
        assert_eq!(p.time_left, 0);
    }

    /// The Kill census (`:72374-72393`): 10 Next(3) gates; a 0 draws two
    /// Next(-10,11), else one. With the census rng returning min (0), every
    /// gate passes → 10 + 20 = 30 range draws.
    #[test]
    fn w4_d13_the_kill_census_draws_the_gates_and_scales() {
        let mut rng = CensusRng { ranges: Vec::new(), floats: 0 };
        sandnado_mark_kill_draws(&mut rng);
        assert_eq!(rng.ranges.len(), 30, "10 gates + 2×10 fadeIn/scale");
        assert_eq!(rng.ranges[0], (0, 3), "the gate");
        assert_eq!(rng.ranges[1], (-10, 11), "fadeIn");
        assert_eq!(rng.ranges[2], (-10, 11), "scale");
        assert_eq!(rng.floats, 0, "no float draws in the kill burst");
    }
}