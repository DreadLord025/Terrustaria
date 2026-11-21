//! W4-d9 aiStyle 81 — the InfluxWaver 451's melee wave (`AI()`'s inline arm
//! `Projectile.cs:31001-31213`). Census: aiStyles 82/83/84/85/88 were armed
//! on main; 81 was a gap (the row + id landed with the W4-d9 census commit).
//!
//! The four-leg state machine (`ai[0]`):
//! * `-1` the launch fade (`:31005-31040`): tileCollide on, the localAI[0]
//!   counter, the speed-scaled alpha ramp to 75 then the hand-off to 0.
//! * `0` the pulse (`:31041-31089`): the alpha oscillation 0↔75 via the
//!   localAI[1] direction latch.
//! * `1..1+penetrate` the homing teleport (`:31090-31153`): tileCollide off,
//!   the alpha ramp + the 0.98/speed velocity decay, the nearest-chaseable
//!   scan, the random-angle teleport at alpha 255 and the target-velocity
//!   ride.
//! * `1+penetrate..1+2·penetrate` the strike decay (`:31154-31210`): the
//!   0.9 scale, the ai[1] counter, the alpha/velocity fade and the kill.
//!
//! The supporting seams (the C# sites outside `AI()`):
//! * the on-hit `ai[0]` ladder — `Damage_PVE_Inner`'s type-451 arm
//!   (`:14051-14065`): the first hit adds penetrate, later hits subtract
//!   penetrate+1, `ai[1]` resets, the walk stops.
//! * the no-damage window — `Damage_CanDealDamage`'s 451 clause (`:12448`).
//! * the tile bounce — `HandleMovement`'s type-451 arm (`:18111-18117`):
//!   a contact arms `ai[0]=1` and halves the velocity.
//!
//! Movement: the AI body carries no `position += velocity` of its own (the
//! C# `HandleMovement` integrates it, `:17015`); the dispatch runs the
//! generic `kill_step` after the arm, with the type-451 bounce override on
//! a tile contact. The homing/strike legs set `tileCollide = false`, so the
//! free-move branch applies; the launch/pulse legs keep it true.
//!
//! RNG contract (the `UpdateProjectiles` stream): the dust bodies are
//! skipped (no dust layer); their EXPLICIT draws are consumed in the C#'s
//! order — the launch/pulse `SelectRandom(226,229)` + `Next(6)` gates
//! (`:31011-31022`/`:31047-31058`), the homing `Next(3)` dust gate
//! (`:31141`) and the strike `SelectRandom` (`:31201`). The ONE
//! state-bearing draw is the teleport angle's `NextDouble` (`:31128`).
//!
//! Code-only round (W4-d9): the witness tests below are WRITTEN NOT RUN.

#![allow(dead_code)] // wiring deferred (W4-d9, code-only mode)

use std::f32::consts::PI;

use glam::Vec2;
use tr_core::loot::DropRng;

use crate::npc::Npc;
use crate::projectile::{chaseable, npc_center, safe_normalize, Projectile, INFLUX_WAVER};

/// `Kill()` (`Projectile.cs:70644-70648`) — the sibling-module deactivation
/// (the `projectile_ai_spear` precedent; `Projectile::kill` is private).
fn kill(p: &mut Projectile) {
    p.kill_time_left = p.time_left;
    p.active = false;
    p.time_left = 0;
}

/// `Entity.Center` for the pool's slots (`pos + size/2`).
fn center(p: &Projectile) -> Vec2 {
    Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5)
}

/// `Utils.SelectRandom<int>(Main.rand, 226, 229)` (`Utils.cs:2459-2462`):
/// `choices[Next(choices.Length)]` — one `Next(2)` draw. The dust colour is
/// a skipped visual; the draw is consumed in order.
fn influx_dust_colour(rng: &mut dyn DropRng) {
    let _ = rng.next_range(0, 2);
}

/// The nearest chaseable NPC within `range` of the projectile's centre
/// (`:31098-31115` / `:31179-31196`): `(index, centre)`, index −1 if none.
fn nearest_chaseable(p: &Projectile, npcs: &[Npc], range: f32) -> (i32, Vec2) {
    let c = center(p);
    let mut best = -1i32;
    let mut best_center = c;
    let mut best_dist = range;
    for (i, n) in npcs.iter().enumerate() {
        if !chaseable(n) {
            continue;
        }
        let nc = npc_center(n);
        let d = (nc - c).length();
        if d < best_dist {
            best_dist = d;
            best_center = nc;
            best = i as i32;
        }
    }
    (best, best_center)
}

/// `AI()`'s aiStyle-81 arm (`Projectile.cs:31001-31213`). `npcs` is the live
/// NPC list (the homing scan), `rng` the `UpdateProjectiles` stream.
pub(crate) fn ai_influx_waver(p: &mut Projectile, npcs: &[Npc], rng: &mut dyn DropRng) {
    // `:31003-31004` — the penetrate snapshot + the speed scale.
    let num618 = p.penetrate;
    let num619 = (p.vel.length() / 11.0).max(1.0);
    if p.ai[0] == -1.0 {
        // `:31005-31040` — the launch fade.
        p.tile_collide = true;
        p.local_ai[0] += 1.0;
        if p.local_ai[0] > 7.0 {
            // `:31011-31022` — the dust (skipped body, consumed draws).
            influx_dust_colour(rng);
            let _ = rng.next_range(0, 6); // the noGravity gate
        }
        let num624 = 0.01 * num619;
        let num625 = (5.0 * num619) as i32;
        let num626 = 75;
        let num627 = ((7.0 / num619) as i32).min(7);
        if p.local_ai[0] > num627 as f32 && p.local_ai[1] == 0.0 {
            p.scale -= num624;
            p.alpha += num625;
            if p.alpha > num626 {
                p.alpha = num626;
                p.ai[0] = 0.0;
                p.net_update = true;
            }
        }
        p.rot = p.vel.to_angle() + PI / 4.0;
    } else if p.ai[0] == 0.0 {
        // `:31041-31089` — the pulse.
        p.tile_collide = true;
        p.local_ai[0] += 1.0;
        if p.local_ai[0] > 7.0 {
            influx_dust_colour(rng);
            let _ = rng.next_range(0, 6);
        }
        let num632 = 0.01;
        let num633 = 5;
        let num634 = num633 * 15;
        let num635 = 0;
        let num636 = ((7.0 * num619) as i32).min(7);
        if p.local_ai[0] > num636 as f32 {
            if p.local_ai[1] == 0.0 {
                p.scale -= num632;
                p.alpha += num633;
                if p.alpha > num634 {
                    p.alpha = num634;
                    p.local_ai[1] = 1.0;
                }
            } else if p.local_ai[1] == 1.0 {
                p.scale += num632;
                p.alpha -= num633;
                if p.alpha <= num635 {
                    p.alpha = num635;
                    p.local_ai[1] = 0.0;
                }
            }
        }
        p.rot = p.vel.to_angle() + PI / 4.0;
    } else if p.ai[0] >= 1.0 && p.ai[0] < (1 + num618) as f32 {
        // `:31090-31153` — the homing teleport leg.
        p.tile_collide = false;
        let num637 = (15.0 * num619) as i32;
        let num638 = 0.98 / num619;
        p.alpha += num637;
        p.vel *= num638;
        p.local_ai[0] = 0.0;
        // `:31098-31115` — the nearest chaseable NPC within 250 px.
        let (num639, vector66) = nearest_chaseable(p, npcs, 250.0);
        if p.alpha >= 255 {
            p.alpha = 255;
            if p.ai[0] == 1.0 {
                kill(p);
                return;
            }
            if num639 >= 0 {
                p.net_update = true;
                p.ai[0] += num618 as f32;
                // `:31128` — the teleport: a random angle ×100 around the
                // target, re-centred. The ONE state-bearing draw.
                let angle = rng.next_float() as f64 * (PI as f64 * 2.0);
                let offset = Vec2::new(angle.cos() as f32, angle.sin() as f32) * 100.0;
                p.pos = vector66 + offset - Vec2::new(p.w, p.h) / 2.0;
                p.vel = safe_normalize(vector66 - center(p)) * 15.0;
                p.rot = p.vel.to_angle() + PI / 4.0;
            } else {
                kill(p);
            }
        }
        // `:31137-31140` — ride the target's velocity.
        if p.active && num639 >= 0 {
            let n = &npcs[num639 as usize];
            p.pos += Vec2::new(n.vx, n.vy);
        }
        // `:31141-31152` — the 1/3 dust (skipped body, consumed draws).
        if rng.next_range(0, 3) == 0 {
            influx_dust_colour(rng);
        }
    } else if p.ai[0] >= (1 + num618) as f32 && p.ai[0] < (1 + num618 * 2) as f32 {
        // `:31154-31210` — the strike decay leg.
        p.scale = 0.9;
        p.tile_collide = false;
        p.rot = p.vel.to_angle() + PI / 4.0;
        p.ai[1] += 1.0;
        if p.ai[1] >= 15.0 {
            p.alpha += 51;
            p.vel *= 0.8;
            if p.alpha >= 255 {
                kill(p);
            }
        } else {
            p.alpha -= 125;
            if p.alpha < 0 {
                p.alpha = 0;
            }
            p.vel *= 0.98;
        }
        p.local_ai[0] += 1.0;
        // `:31179-31200` — the nearest chaseable + the velocity ride.
        let (num647, _) = nearest_chaseable(p, npcs, 250.0);
        if num647 >= 0 {
            let n = &npcs[num647 as usize];
            p.pos += Vec2::new(n.vx, n.vy);
        }
        // `:31201-31209` — the dust (skipped body, consumed draw).
        influx_dust_colour(rng);
    }
    // `:31211-31212` — the light (skipped: the ProjLight seam owns lights).
}

/// `Damage_PVE_Inner`'s type-451 on-hit arm (`Projectile.cs:14051-14065`):
/// the first hit (`ai[0] == 0`) adds penetrate, every later hit subtracts
/// penetrate+1; `ai[1]` resets and the walk stops (`keepIterating = false`).
/// Returns true when the walk must stop.
pub(crate) fn influx_waver_on_hit(p: &mut Projectile) -> bool {
    if p.ai[0] == 0.0 {
        p.ai[0] += p.penetrate as f32;
    } else {
        p.ai[0] -= (p.penetrate + 1) as f32;
    }
    p.ai[1] = 0.0;
    p.net_update = true;
    true
}

/// `Damage_CanDealDamage`'s type-451 clause (`Projectile.cs:12448`): the
/// 451 deals no damage while `((int)(ai[0]-1)/penetrate == 0 || ai[1] < 5)
/// && ai[0] != 0`. Returns true when the hit is refused.
pub(crate) fn influx_waver_refuses_damage(p: &Projectile) -> bool {
    if p.kind != INFLUX_WAVER {
        return false;
    }
    // The C# integer division `(int)(ai[0]-1f)/penetrate` truncates toward
    // zero; penetrate is the live count (3 at spawn, decays per hit). The
    // C# divides by the live penetrate (a 0 would throw; the port guards).
    let q = ((p.ai[0] - 1.0) as i32) / p.penetrate.max(1);
    (q == 0 || p.ai[1] < 5.0) && p.ai[0] != 0.0
}

/// `HandleMovement`'s type-451 blocked-move arm (`Projectile.cs:18111-
/// 18117`): a tile contact arms the homing leg (`ai[0] = 1`, `ai[1] = 0`)
/// and halves the velocity. Called from the dispatch's kill_step override.
pub(crate) fn influx_waver_tile_bounce(p: &mut Projectile, last_velocity: Vec2) {
    p.ai[0] = 1.0;
    p.ai[1] = 0.0;
    p.net_update = true;
    p.vel = last_velocity / 2.0;
}

/// The 451's `HandleMovement` half — the port's `kill_step` destination-box
/// probe stand-in (`Projectile.cs:19851-19855`), with the type-451 bounce
/// (`:18111-18117`) replacing the default kill on a tile contact. The
/// free-move branch (`tileCollide = false`, the homing/strike legs)
/// integrates the velocity; the launch/pulse legs keep `tileCollide` true
/// and bounce. The exact bounce pixel may differ by the swept span (the
/// recorded `kill_step` deviation, ANALYSIS).
pub(crate) fn influx_waver_move(p: &mut Projectile, world: &tr_core::World) {
    if !p.tile_collide {
        p.pos += p.vel;
        return;
    }
    let last_vel = p.vel;
    let nx = p.pos.x + p.vel.x;
    let ny = p.pos.y + p.vel.y;
    let hit_x = crate::player::collides_rect(world, nx, p.pos.y, p.w, p.h);
    let hit_y = crate::player::collides_rect(world, p.pos.x, ny, p.w, p.h);
    p.pos += p.vel;
    if hit_x || hit_y {
        influx_waver_tile_bounce(p, last_vel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::proj_data;
    use tr_core::TILE_PX;

    struct Recorder {
        draws: Vec<(i32, i32)>,
        floats: usize,
        next: i32,
    }
    impl Recorder {
        fn new(next: i32) -> Self {
            Self { draws: Vec::new(), floats: 0, next }
        }
    }
    impl DropRng for Recorder {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.draws.push((min, max));
            self.next
        }
        fn next_float(&mut self) -> f32 {
            self.floats += 1;
            0.5
        }
    }

    fn proj(vx: f32, vy: f32) -> Projectile {
        let d = proj_data(INFLUX_WAVER).expect("the 451 row");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = INFLUX_WAVER;
        p.owner = 0;
        p.pos = Vec2::new(10.0 * TILE_PX, 10.0 * TILE_PX);
        p.vel = Vec2::new(vx, vy);
        p.w = d.w;
        p.h = d.h;
        p.damage = 40;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.max_penetrate = d.penetrate;
        p.alpha = d.alpha;
        p.scale = d.scale;
        p.tile_collide = d.tile_collide;
        p.melee = d.melee;
        p
    }

    /// The row matches `SetDefaults` verbatim (`Projectile.cs:5229-5239`).
    #[test]
    fn w4_d9_the_influx_row_matches_set_defaults() {
        let d = proj_data(INFLUX_WAVER).unwrap();
        assert_eq!((d.w, d.h), (16.0, 16.0));
        assert_eq!(d.ai_style, 81);
        assert!(d.melee);
        assert_eq!(d.penetrate, 3);
        assert_eq!(d.alpha, 255);
        assert!((d.light - 0.2).abs() < 1e-6);
        assert!(d.tile_collide);
        assert_eq!(d.time_left, 3600);
    }

    /// The launch fade hands off to ai[0]=0 at alpha 75.
    #[test]
    fn w4_d9_the_influx_launch_fade_hands_off() {
        let mut p = proj(11.0, 0.0);
        p.ai[0] = -1.0;
        p.alpha = 255;
        let mut rng = Recorder::new(1);
        // num619 = max(1, 11/11) = 1; num627 = min(7, 7/1) = 7. The fade
        // starts at localAI[0] > 7; alpha 255 + 5 > 75 clamps to 75 and
        // hands off on the first fade tick (the 8th).
        for _ in 0..8 {
            ai_influx_waver(&mut p, &[], &mut rng);
        }
        assert_eq!(p.ai[0], 0.0, "the launch hands off at alpha 75");
        assert_eq!(p.alpha, 75);
        assert!(p.net_update);
    }

    /// The pulse oscillates alpha 0↔75 via localAI[1].
    #[test]
    fn w4_d9_the_influx_pulse_oscillates() {
        let mut p = proj(11.0, 0.0);
        p.ai[0] = 0.0;
        p.alpha = 0;
        p.local_ai[0] = 8.0; // past the num636 = min(7, 7*1) = 7 threshold
        let mut rng = Recorder::new(1);
        // 15 ticks of +5 reach alpha 75; the flip fires at alpha > 75, so
        // the 16th tick clamps back to 75 and sets localAI[1] = 1.
        for _ in 0..16 {
            ai_influx_waver(&mut p, &[], &mut rng);
        }
        assert_eq!(p.alpha, 75);
        assert_eq!(p.local_ai[1], 1.0);
        for _ in 0..15 {
            ai_influx_waver(&mut p, &[], &mut rng);
        }
        assert_eq!(p.alpha, 0);
        assert_eq!(p.local_ai[1], 0.0);
    }

    /// The on-hit ladder: first hit adds penetrate, later hits subtract
    /// penetrate+1; ai[1] resets and the walk stops.
    #[test]
    fn w4_d9_the_influx_on_hit_ladder() {
        let mut p = proj(11.0, 0.0);
        p.penetrate = 3;
        p.ai[0] = 0.0;
        p.ai[1] = 9.0;
        assert!(influx_waver_on_hit(&mut p));
        assert_eq!(p.ai[0], 3.0, "the first hit adds penetrate");
        assert_eq!(p.ai[1], 0.0);
        assert!(influx_waver_on_hit(&mut p));
        assert_eq!(p.ai[0], -1.0, "the second hit subtracts penetrate+1");
    }

    /// The no-damage window: ai[0]=0 always deals; the q==0 band and the
    /// ai[1]<5 window refuse.
    #[test]
    fn w4_d9_the_influx_damage_window() {
        let mut p = proj(11.0, 0.0);
        p.penetrate = 3;
        p.ai[0] = 0.0;
        assert!(!influx_waver_refuses_damage(&p), "ai[0]=0 always deals");
        p.ai[0] = 1.0; // (1-1)/3 = 0 → refuse
        p.ai[1] = 9.0;
        assert!(influx_waver_refuses_damage(&p));
        p.ai[0] = 4.0; // (4-1)/3 = 1, ai[1]=9 ≥ 5 → deal
        assert!(!influx_waver_refuses_damage(&p));
        p.ai[1] = 4.0; // ai[1] < 5 → refuse
        assert!(influx_waver_refuses_damage(&p));
    }

    /// The tile bounce arms ai[0]=1 and halves the velocity.
    #[test]
    fn w4_d9_the_influx_tile_bounce() {
        let mut p = proj(11.0, 0.0);
        influx_waver_tile_bounce(&mut p, Vec2::new(10.0, 4.0));
        assert_eq!(p.ai[0], 1.0);
        assert_eq!(p.ai[1], 0.0);
        assert_eq!(p.vel, Vec2::new(5.0, 2.0));
        assert!(p.net_update);
    }

    /// The homing teleport: at alpha 255 with a target, ai[0] jumps by
    /// penetrate and the position teleports around the target.
    #[test]
    fn w4_d9_the_influx_homing_teleport() {
        let mut p = proj(11.0, 0.0);
        p.penetrate = 3;
        p.ai[0] = 2.0; // the homing band (1..4), not the first-hit kill
        p.alpha = 250;
        let mut n = Npc::new(crate::npc::NpcKind::Zombie, 300.0, 160.0, 1, 0.0);
        n.hp = n.max_hp;
        let npcs = vec![n];
        let mut rng = Recorder::new(1);
        ai_influx_waver(&mut p, &npcs, &mut rng);
        assert_eq!(p.ai[0], 5.0, "the teleport jumps ai[0] by penetrate");
        assert!(p.net_update);
        assert_eq!(rng.floats, 1, "the teleport's one NextDouble");
        // The position is 100 px from the target's centre.
        let nc = npc_center(&npcs[0]);
        let d = (center(&p) - nc).length();
        assert!((d - 100.0).abs() < 1.0, "the teleport lands 100 px out: {d}");
    }
}
