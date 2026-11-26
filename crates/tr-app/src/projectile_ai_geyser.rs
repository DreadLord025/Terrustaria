//! W4-d13 — `AI()`'s `else if (this.aiStyle == 126)` arm
//! (`Projectile.cs:35834-35997`): the vertical steam/shock column — the
//! Geyser Trap (654, `ProjectileID.GeyserTrap`, `ProjectileID.cs:1862`; the
//! wiring device shot, `tr-core/wiring.rs` `geyser_trap`) and the DD2 Ogre
//! Stomp (670, `ProjectileID.DD2OgreStomp`, `:1894`). The `SetDefaults` rows
//! are `:7024-7037` (654: friendly + hostile + trap) and `:7174-7185` (670:
//! hostile only). Both: 30x30, alpha 255, tileCollide false, ignoreWater,
//! timeLeft 120, penetrate -1.
//!
//! **RNG census (by construction).** Every explicit `Main.rand` draw in the
//! arm stays on the stream in the C#'s order; the `Dust.NewDust` INTERNAL
//! rolls belong to the dust pool and are SKIPPED (the S1-D-4b-7 precedent,
//! `projectile_ai067_b`'s module doc). The dust bodies themselves are
//! skipped (no dust layer). Per tick:
//! * state 0 (`ai[0] == 0`, `:35852-35859`): 3 dust, each TWO `Next(5)`
//!   (scale `:35855`, fadeIn `:35856`) = 6 ranges.
//! * the first-solid burst (`localAI[0] == 0`, `:35912-35925`): 60 dust,
//!   each `SelectRandom(6,259,158)` = ONE `Next(3)` (`Utils.cs:2459-2462`),
//!   ONE `NextFloat` (scale `:35918`), and — only when `num938 == -1` — ONE
//!   `Next(4)` (`:35921`).
//! * the `num938 == 1` plume (`:35930-35943`): 9 dust, same per-dust shape.
//! * the column dust (`:35952-35980`): 6 iterations, each a `Next(3) < 2`
//!   gate; on a pass a `Next(6) < 3` branch pick (the else arm's velocity
//!   math draws nothing).
//! * the top dust (`:35982-35995`): 6 iterations, each a `NextFloat < 0.5`
//!   gate; on a pass `SelectRandom` = ONE `Next(3)`, ONE `NextFloat` (scale),
//!   and — only when `num938 == -1` — ONE `Next(4)`.
//! The stomp's child spawns (`:35889-35904`) draw NOTHING.
//!
//! Deviations (the standing pool precedents, ANALYSIS): the dust bodies and
//! the `SoundID.Item34` cue (`:35926`) — the cue rides `wants_sound` (the
//! pool drains it); the `Collision.SolidCollision`/`WetCollision` probes
//! ride the port's `Player::solid_collision` / `player::wet_collision`
//! (the same `-1/+2` sample expansion). The row shape carries neither
//! friendly nor hostile (the data-row deviation census); the stomp's
//! `owner == Main.myPlayer` child gate folds to the simulated-slot set.

use glam::Vec2;
use tr_core::World;
use tr_core::loot::DropRng;

use crate::projectile::{OGRE_STOMP, Projectile, csign};

/// `Kill()` (`Projectile.cs:70644-70648`) — the sibling-module deactivation
/// (the `projectile_ai_spear` precedent; `Projectile::kill` is private).
fn kill(p: &mut Projectile) {
    p.kill_time_left = p.time_left;
    p.active = false;
    p.time_left = 0;
}

/// One stomp child spawn (`Projectile.cs:35893-35902`): the spawn pixel
/// anchor (the C# `base.Bottom + offset`, `NewProjectile` centres the box)
/// and the child's `localAI[1]` payload. The child is the SAME kind, damage,
/// knockback and owner, velocity `-Vector2.UnitY`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct StompChild {
    pub x: f32,
    pub y: f32,
    pub local_ai1: f32,
}

impl Projectile {
    /// `AI()`'s aiStyle-126 arm (`Projectile.cs:35834-35997`). Returns the
    /// stomp's child spawns (the 654 geyser returns none — its `type == 670`
    /// gates are false). The movement is the dispatch's free integrate
    /// (`tileCollide = false`, `UpdatePosition` `:20445` — 126 is not in the
    /// `:20435` skip list); the arm pins velocity to ±0.001.
    pub(crate) fn ai_geyser_stomp(
        &mut self,
        world: &World,
        rng: &mut dyn DropRng,
        local_owner: u8,
    ) -> Vec<StompChild> {
        let mut children = Vec::new();
        // `:35836-35837` — the column direction and the dust customData tag.
        let num938 = csign(self.vel.y) as i32;
        let num939 = if num938 != -1 { 1 } else { 0 };
        // `:35838-35860` — state 0: the free-fall until the column anchors.
        if self.ai[0] == 0.0 {
            let off_solid = if num938 == -1 { self.h - 48.0 } else { 0.0 };
            let off_wet = if num938 == -1 { self.h - 20.0 } else { 0.0 };
            let solid = crate::player::Player::solid_collision(
                world,
                self.pos.x,
                self.pos.y + off_solid,
                self.w,
                48.0,
            );
            let wet = crate::player::wet_collision(
                world,
                self.pos.x,
                self.pos.y + off_wet,
                self.w,
                20.0,
            );
            if !solid && !wet {
                self.vel = Vec2::new(0.0, csign(self.vel.y) * 0.001);
                self.ai[0] = 1.0;
                self.ai[1] = 0.0;
                self.time_left = 60;
            }
            self.ai[1] += 1.0;
            if self.ai[1] >= 60.0 {
                kill(self);
            }
            // `:35852-35859` — 3 dust 31, each two Next(5) (scale + fadeIn).
            for _ in 0..3 {
                let _ = rng.next_range(0, 5); // scale `:35855`
                let _ = rng.next_range(0, 5); // fadeIn `:35856`
            }
        }
        // `:35861-35864` — only state 1 continues.
        if self.ai[0] != 1.0 {
            return children;
        }
        // `:35865` — the velocity pin.
        self.vel = Vec2::new(0.0, csign(self.vel.y) * 0.001);
        // `:35866-35887` — the column-height scan.
        if num938 != 0 {
            let mut num942 = 16i32;
            let mut num943 = 320i32;
            if self.kind == OGRE_STOMP {
                num943 -= (self.local_ai[1].abs() as i32) * 64;
            }
            while num942 < num943 {
                let off = if num938 == -1 {
                    self.h - num942 as f32 - 16.0
                } else {
                    0.0
                };
                if crate::player::Player::solid_collision(
                    world,
                    self.pos.x,
                    self.pos.y + off,
                    self.w,
                    (num942 + 16) as f32,
                ) {
                    break;
                }
                num942 += 16;
            }
            if num938 == -1 {
                self.pos.y += self.h;
                self.h = num942 as f32;
                self.pos.y -= num942 as f32;
            } else {
                self.h = num942 as f32;
            }
        }
        // `:35888` — the age tick.
        self.ai[1] += 1.0;
        // `:35889-35904` — the stomp's tick-12 child split (local-owner only).
        if self.kind == OGRE_STOMP
            && self.owner == local_owner
            && self.ai[1] == 12.0
            && self.local_ai[1] < 3.0
            && self.local_ai[1] > -3.0
        {
            let bottom = Vec2::new(self.pos.x + self.w * 0.5, self.pos.y + self.h);
            if self.local_ai[1] == 0.0 {
                // `:35893-35896` — the two ±50 children.
                children.push(StompChild {
                    x: bottom.x - 50.0,
                    y: bottom.y - 10.0,
                    local_ai1: self.local_ai[1] - 1.0,
                });
                children.push(StompChild {
                    x: bottom.x + 50.0,
                    y: bottom.y - 10.0,
                    local_ai1: self.local_ai[1] + 1.0,
                });
            } else {
                // `:35900-35902` — the single sign-side child.
                let num945 = csign(self.local_ai[1]);
                children.push(StompChild {
                    x: bottom.x + 50.0 * num945,
                    y: bottom.y - 10.0,
                    local_ai1: self.local_ai[1] + num945,
                });
            }
        }
        // `:35905-35908` — the 60-tick life.
        if self.ai[1] >= 60.0 {
            kill(self);
        }
        // `:35909-35927` — the first-solid burst (once) + the Item34 cue.
        if self.local_ai[0] == 0.0 {
            self.local_ai[0] = 1.0;
            for _ in 0..60 {
                let _ = rng.next_range(0, 3); // SelectRandom(6,259,158) `:35914`
                let _ = rng.next_float(); // scale `:35918`
                if num938 == -1 {
                    let _ = rng.next_range(0, 4); // `:35921`
                }
            }
            // `SoundID.Item34` (`:35926`) at base.position — the pool's cue.
            let (x, y) = (self.pos.x, self.pos.y);
            self.wants_sound = Some((2, 34, x, y, 0.0, 1.0));
        }
        // `:35928-35944` — the downward plume (num938 == 1), 9 dust.
        if num938 == 1 {
            for _ in 0..9 {
                let _ = rng.next_range(0, 3); // SelectRandom `:35932`
                let _ = rng.next_float(); // scale `:35936`
                if num938 == -1 {
                    let _ = rng.next_range(0, 4); // `:35939` (dead: num938 == 1)
                }
            }
        }
        // `:35945-35951` — the column dust span.
        let mut num951 = ((self.ai[1] / 60.0 * self.h) as i32) * 3;
        if num951 > self.h as i32 {
            num951 = self.h as i32;
        }
        let _vector156 = self.pos
            + if num938 == -1 {
                Vec2::new(0.0, self.h - num951 as f32)
            } else {
                Vec2::ZERO
            };
        let _vector157 = self.pos
            + if num938 == -1 {
                Vec2::new(0.0, self.h)
            } else {
                Vec2::ZERO
            };
        // `:35952-35981` — the column dust, 6 iterations.
        for _ in 0..6 {
            if rng.next_range(0, 3) < 2 {
                // `:35956` NewDust internals skipped; the branch pick:
                if rng.next_range(0, 6) < 3 {
                    // `:35963-35973` — the lerp/velocity arm draws nothing.
                } else {
                    // `:35976` — the DirectionFrom velocity arm draws nothing.
                }
            }
        }
        // `:35982-35996` — the top dust, 6 iterations.
        for _ in 0..6 {
            if !(rng.next_float() < 0.5) {
                let _ = rng.next_range(0, 3); // SelectRandom `:35986`
                let _ = rng.next_float(); // scale `:35989`
                if num938 == -1 {
                    let _ = rng.next_range(0, 4); // `:35990`
                }
            }
        }
        let _ = num939; // the dust customData tag (no dust layer)
        children
    }
}

#[cfg(test)]
mod w4_d13_style126_tests {
    use super::*;
    use crate::projectile::{GEYSER_TRAP, proj_data};

    fn world() -> World {
        World::new("t".into(), 64, 64)
    }

    fn column(kind: u16, pos: Vec2, vy: f32) -> Projectile {
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.w = 30.0;
        p.h = 30.0;
        p.pos = pos;
        p.vel = Vec2::new(0.0, vy);
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

    /// The two rows' `SetDefaults` slice (`Projectile.cs:7024-7037`,
    /// `:7174-7185`).
    #[test]
    fn w4_d13_the_two_rows_carry_the_csharp_set_defaults() {
        for kind in [GEYSER_TRAP, OGRE_STOMP] {
            let d = proj_data(kind).unwrap_or_else(|| panic!("kind {kind} rowed"));
            assert_eq!((d.w, d.h), (30.0, 30.0), "kind {kind}");
            assert_eq!(d.ai_style, 126, "kind {kind}");
            assert_eq!(d.penetrate, -1, "kind {kind}");
            assert_eq!(d.time_left, 120, "kind {kind}");
            assert_eq!(d.alpha, 255, "kind {kind}");
            assert!(!d.tile_collide, "kind {kind}");
            assert!(d.ignore_water, "kind {kind}");
        }
        assert!(proj_data(GEYSER_TRAP).unwrap().trap, "the geyser is a trap");
        assert!(!proj_data(OGRE_STOMP).unwrap().trap, "the stomp is not");
    }

    /// `:35838-35846` — state 0 in free air anchors the column: ai[0] → 1,
    /// ai[1] reset then ticked to 1, timeLeft 60, the velocity pin.
    #[test]
    fn w4_d13_state0_anchors_in_free_air() {
        let w = world();
        let mut p = column(GEYSER_TRAP, Vec2::new(200.0, 200.0), -8.0);
        let mut rng = CensusRng { ranges: Vec::new(), floats: 0 };
        let kids = p.ai_geyser_stomp(&w, &mut rng, 0);
        assert!(kids.is_empty(), "the geyser spawns no children");
        assert_eq!(p.ai[0], 1.0, "the anchor latch");
        assert_eq!(p.ai[1], 2.0, "reset to 0 then state-0 ++ then state-1 ++");
        assert_eq!(p.time_left, 60);
        assert_eq!(p.vel, Vec2::new(0.0, -0.001), "the ±0.001 pin");
        // state-0's 3 dust × 2 Next(5), then the state-1 census follows.
        assert_eq!(rng.ranges[0], (0, 5));
        assert_eq!(rng.ranges[1], (0, 5));
    }

    /// `:35889-35904` — the stomp's tick-12 split: localAI[1] == 0 spawns
    /// the two ±50 children with localAI[1] ∓1.
    #[test]
    fn w4_d13_the_stomp_splits_at_tick_12() {
        let w = world();
        let mut p = column(OGRE_STOMP, Vec2::new(200.0, 200.0), 8.0);
        p.ai[0] = 1.0;
        p.ai[1] = 11.0; // the ++ makes it 12
        p.local_ai[0] = 1.0; // skip the first-solid burst
        p.local_ai[1] = 0.0;
        let mut rng = CensusRng { ranges: Vec::new(), floats: 0 };
        let kids = p.ai_geyser_stomp(&w, &mut rng, 0);
        assert_eq!(kids.len(), 2, "the two ±50 children");
        let bottom = Vec2::new(215.0, 200.0 + p.h);
        assert_eq!(kids[0].x, bottom.x - 50.0);
        assert_eq!(kids[0].local_ai1, -1.0);
        assert_eq!(kids[1].x, bottom.x + 50.0);
        assert_eq!(kids[1].local_ai1, 1.0);
        assert_eq!(kids[0].y, bottom.y - 10.0);
    }

    /// `:35870-35873` — the stomp's column cap shrinks by |localAI[1]|·64;
    /// a ±2 child caps at 320 − 128 = 192.
    #[test]
    fn w4_d13_the_stomp_child_cap_shrinks() {
        let w = world();
        let mut p = column(OGRE_STOMP, Vec2::new(200.0, 100.0), 8.0);
        p.ai[0] = 1.0;
        p.ai[1] = 1.0;
        p.local_ai[0] = 1.0;
        p.local_ai[1] = 2.0;
        let mut rng = CensusRng { ranges: Vec::new(), floats: 0 };
        let _ = p.ai_geyser_stomp(&w, &mut rng, 0);
        // num943 = 320 − 2·64 = 192; the scan steps 16..192 → height 192.
        assert_eq!(p.h, 192.0, "the |localAI[1]|·64 cap");
    }
}