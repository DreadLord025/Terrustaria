//! W4-d7 (aiStyle-63) — the Queen Spider's BabySpider (379),
//! `Projectile.AI`'s `else if (this.aiStyle == 63)` block
//! (`Projectile.cs:29318-29418`), its `HandleMovement` bounce arm
//! (`:18453-18463`), its `StatusNPC` Venom roll (`:11490-11493`), its
//! `Kill()` dust census (`:75522-75532`) and the 378 SpiderEgg's hatch
//! producer (`:77240-77253`), ported 1:1 as PURE machines (code-only mode:
//! no compiler/tests were run on this branch; the witness tests below are
//! written, not executed).
//!
//! ## Census (research/26e row 63)
//!
//! ONE vanilla member: BabySpider 379 (`ProjectileID.cs:1312`,
//! `SetDefaults` `Projectile.cs:4635-4644`: 14×10, friendly,
//! `timeLeft = 300`, `penetrate = 1`, `decidesManualFallThrough = true`).
//! The port has NO `proj_data` row for 379 (the DW-1.1 pin
//! projectile.rs:36585 asserts it rowless; `SENTRY_SHOT` lists it,
//! data.rs:4505, so the whip-tag gate already treats it as a summon shot).
//! This module lands UNWIRED (the d5 / `npc_ai_style25` precedent): pure
//! fns on the slot, reachable and testable before the row + dispatch arm
//! land. 379 is NOT `Main.projPet` (no `projPet[379]` write,
//! `Main.cs:8728-8852`) — `correctSlopeCollision` is false and the
//! fall-through is the arm's own `shouldFallThrough` (`:17583-17586`).
//! `CanCutTiles` excludes 379 (`:15329`).
//!
//! ## The AI machine (verbatim, `:29318-29418`)
//!
//! * `:29320-29324` — no active owner: `active = false`, return (the
//!   direct deactivate, NOT `Kill()` — the pirates' `:49147` shape).
//! * `:29325-29341` — the anchor scan: `center3` starts at
//!   `base.position` (the TOP-LEFT quirk — unused when no target is
//!   found), the closest `CanBeChasedBy` NPC inside 2000 px with a clear
//!   `Collision.CanHit` line wins (the strict `<` running minimum).
//! * `:29342-29346` — no target: `vx *= 0.95`, `shouldFallThrough = false`.
//! * `:29347-29378` — target: `shouldFallThrough = target.Y > Bottom.Y`;
//!   the grounded pair (5 / 0.08) vs the airborne pair (8 / 0.12, any
//!   `vy != 0`); the grounded −6 hop when the target sits more than 50 px
//!   above the centre; the signed X accel + the ±speed clamp.
//! * `:29379-29380` — `Collision.StepUp` with a FRESH LOCAL
//!   `num489 = 0f` stepSpeed (NOT `this.stepSpeed`) and the slot's
//!   `gfxOffY`; gravDir 1, holdsMatching false (the defaults).
//! * `:29381-29407` — the frame machine: airborne (`vy != 0`) frame 3;
//!   grounded the counter ticks only while `|vx| > 0.2`, wraps at 9, and
//!   the bands are ≥6 → 2, ≥3 → 1, else 0.
//! * `:29408-29417` — `direction = Sign(vx)` when `vx != 0`,
//!   `spriteDirection = -direction`, gravity `vy += 0.2` capped 16.
//!
//! Movement: `HandleMovement`'s dry pass runs `TileCollision` with
//! `flag6 = shouldFallThrough` (`decidesManualFallThrough`, `:17583`) and
//! the `!correctSlopeCollision` slope pass (`:17850-17892`); a blocked
//! move reaches the type-379 arm [`spider_bounce`] (the `:18398` gate
//! admits it); `UpdatePosition`'s shared integrate (`:20445`, 63 is not
//! in the `:20435` skip list) is the wiring caller's.
//!
//! ## RNG contract
//!
//! The AI arm draws ZERO `Main.rand`. The kept draws, all on the
//! `UpdateProjectiles` stream (`Main.cs:17775-17777`):
//! - `StatusNPC` (`:11492`): ONE `Next(2,5)` — Venom 70, `60 *` the roll.
//! - `Kill()` (`:75527`): 5× dust 171, each ONE `Next(1,10)` scale draw
//!   (the bodies are the documented dust skip; the 377 SpiderHiver shares
//!   the arm — the wiring caller gates the kind).
//! - The egg hatch (`:77242-77251`): `Next(2,4)` count, `Next(5)` bonus
//!   gate, then per spawn the X jitter `Next(-20,21)` BEFORE the Y jitter
//!   (the C# named-argument list evaluates SpeedX then SpeedY).
#![allow(dead_code)] // UNWIRED (W4-d7): the 379 row + the dispatch arm are the wiring step

use crate::npc::Npc;
use crate::projectile::{can_hit, chaseable, csign, npc_center, Projectile};
use glam::Vec2;
use tr_core::loot::DropRng;
use tr_core::World;

/// `ProjectileID.BabySpider` (`ProjectileID.cs:1312`).
pub const BABY_SPIDER: u16 = 379;
/// `ProjectileID.SpiderEgg` (`ProjectileID.cs:1310`) — the hatch producer.
pub const SPIDER_EGG: u16 = 378;

/// The `SetDefaults` row (`Projectile.cs:4635-4644`):
/// (width, height, timeLeft, penetrate); friendly +
/// `decidesManualFallThrough = true` ride the wiring row.
pub const SPIDER_DEFAULTS: (f32, f32, i32, i32) = (14.0, 10.0, 300, 1);

/// The slot's `Kill()` shape (the pool's `kill` convention).
fn kill(p: &mut Projectile) {
    p.kill_time_left = p.time_left;
    p.active = false;
    p.time_left = 0;
}

/// `AI()`'s aiStyle-63 arm (`Projectile.cs:29318-29418`), verbatim.
/// `owner_active` is `Main.player[this.owner].active` (the port's
/// single-player fold: the dispatch simulates only owner-matched slots,
/// so the wiring caller passes the local player's presence).
pub fn spider_step(p: &mut Projectile, world: &World, npcs: &[Npc], owner_active: bool) {
    // `:29320-29324` — the owner-inactive kill (direct, not Kill()).
    if !owner_active {
        p.active = false;
        return;
    }
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    // `:29325-29327` — the anchor starts at the TOP-LEFT (the quirk).
    let mut center3 = p.pos;
    let mut flag27 = false;
    let mut num484 = 2000.0f32;
    // `:29328-29341` — the closest chaseable NPC with a clear line.
    for n in npcs {
        // `CanBeChasedBy(this)` — the pool's inline-AI gate convention
        // (the aiStyle-70 bubble's `:30314` reads the same helper).
        if !chaseable(n) {
            continue;
        }
        let num486 = center.distance(npc_center(n));
        if num486 >= num484 {
            continue;
        }
        let (nw, nh) = n.size();
        if !can_hit(world, p.pos.x, p.pos.y, p.w, p.h, n.x, n.y, nw, nh) {
            continue;
        }
        num484 = num486;
        center3 = npc_center(n);
        flag27 = true;
    }
    if !flag27 {
        // `:29342-29346` — the no-target drag + fall-through clear.
        p.vel.x *= 0.95;
        p.should_fall_through = false;
    } else {
        // `:29349` — the manual fall-through latch.
        p.should_fall_through = center3.y > p.pos.y + p.h;
        // `:29350-29368` — the grounded/airborne steering pair.
        let mut num487 = 5.0f32;
        let mut num488 = 0.08f32;
        if p.vel.y == 0.0 {
            // `:29354-29362` — the hop when the target sits >50 px above.
            let flag28 = center.y - 50.0 > center3.y;
            if flag28 {
                p.vel.y = -6.0;
            }
        } else {
            num487 = 8.0;
            num488 = 0.12;
        }
        // `:29369-29377` — the signed accel + the ±speed clamp.
        p.vel.x += csign(center3.x - center.x) * num488;
        if p.vel.x < 0.0 - num487 {
            p.vel.x = 0.0 - num487;
        }
        if p.vel.x > num487 {
            p.vel.x = num487;
        }
    }
    // `:29379-29380` — StepUp with a FRESH LOCAL stepSpeed (num489 = 0f),
    // the slot's gfxOffY; gravDir 1 / holdsMatching false (the defaults).
    let mut num489 = 0.0f32;
    let mut pos = [p.pos.x, p.pos.y];
    let mut vel = [p.vel.x, p.vel.y];
    crate::collision::step_up(
        world,
        &mut pos,
        &mut vel,
        p.w,
        p.h,
        &mut num489,
        &mut p.gfx_off_y,
        false,
    );
    p.pos = Vec2::new(pos[0], pos[1]);
    p.vel = Vec2::new(vel[0], vel[1]);
    // `:29381-29407` — the frame machine.
    if p.vel.y != 0.0 {
        p.frame = 3;
    } else {
        if p.vel.x.abs() > 0.2 {
            p.frame_counter += 1;
        }
        if p.frame_counter >= 9 {
            p.frame_counter = 0;
        }
        if p.frame_counter >= 6 {
            p.frame = 2;
        } else if p.frame_counter >= 3 {
            p.frame = 1;
        } else {
            p.frame = 0;
        }
    }
    // `:29408-29412` — the facing pair.
    if p.vel.x != 0.0 {
        p.direction = csign(p.vel.x) as i32;
    }
    p.sprite_direction = -p.direction;
    // `:29413-29417` — the gravity tail.
    p.vel.y += 0.2;
    if p.vel.y > 16.0 {
        p.vel.y = 16.0;
    }
}

/// `HandleMovement`'s type-379 blocked-move arm (`Projectile.cs:18453-
/// 18463`), verbatim: X reflects × −0.6 on ANY X-block; Y reflects × −0.6
/// only when the pre-collision `lastVelocity.Y > 2f`. The wiring caller
/// invokes it when the move was blocked (the `:18398` gate admits 379 —
/// `correctSlopeCollision` false), with the pre-collision velocity; the
/// free move stays `UpdatePosition`'s generic tail.
pub fn spider_bounce(p: &mut Projectile, last_vel: Vec2) {
    if p.vel.x != last_vel.x {
        p.vel.x = last_vel.x * -0.6;
    }
    if p.vel.y != last_vel.y && last_vel.y > 2.0 {
        p.vel.y = last_vel.y * -0.6;
    }
}

/// `StatusNPC`'s type-379 arm (`Projectile.cs:11490-11493`): Venom (70)
/// for `60 * Next(2,5)` ticks — ONE kept draw, unconditional.
pub fn spider_status_npc(npc: &mut Npc, rng: &mut dyn DropRng) {
    let t = 60 * rng.next_range(2, 5);
    npc.buffs.add_buff(70, t);
}

/// `Kill()`'s 379/377 dust census (`Projectile.cs:75522-75532`): 5× dust
/// 171, each drawing ONE `Next(1,10)` for its scale (`* 0.1f`). The dust
/// BODIES are the documented skip (no draw layer); the five scale draws
/// stay on the stream. The wiring caller gates the kind pair.
pub fn spider_kill_draws(rng: &mut dyn DropRng) {
    for _ in 0..5 {
        let _ = rng.next_range(1, 10);
    }
}

/// One hatched BabySpider spawn (`Projectile.NewProjectile` `:77251`):
/// the C# spawn arguments minus the stats the wiring caller reads off the
/// egg slot (`Damage: this.damage, KnockBack: this.knockBack, Owner:
/// this.owner`, `Type: 379`); the spawn subtracts the row's half-box
/// (`:10876-10877`, the `RainbowBackSpawn` precedent).
#[derive(Clone, Copy, Debug)]
pub struct SpiderSpawn {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
}

/// The 378 SpiderEgg's `Kill()` hatch producer (`Projectile.cs:77240-
/// 77253`), verbatim: the local-owner gate, the `Next(2,4)` count + the
/// `Next(5) == 0` bonus, then per spawn the egg's velocity jittered by
/// `1 + Next(-20,21) * 0.01` per axis — the X draw BEFORE the Y draw (the
/// named-argument evaluation order: SpeedX then SpeedY). The wiring
/// caller invokes it on the egg's death hook.
pub fn egg_hatch(p: &Projectile, local_owner: u8, rng: &mut dyn DropRng) -> Vec<SpiderSpawn> {
    let mut out = Vec::new();
    // `:77240` — the kind + owner gate.
    if p.kind != SPIDER_EGG || p.owner != local_owner {
        return out;
    }
    // `:77242-77246` — the count roll + the bonus.
    let mut num981 = rng.next_range(2, 4);
    if rng.next_range(0, 5) == 0 {
        num981 += 1;
    }
    let cx = p.pos.x + p.w * 0.5;
    let cy = p.pos.y + p.h * 0.5;
    // `:77247-77252` — the per-spawn jitter pair, X then Y.
    for _ in 0..num981 {
        let x2 = p.vel.x;
        let y8 = p.vel.y;
        let jx = rng.next_range(-20, 21) as f32 * 0.01;
        let jy = rng.next_range(-20, 21) as f32 * 0.01;
        out.push(SpiderSpawn {
            x: cx,
            y: cy,
            vx: x2 * (1.0 + jx),
            vy: y8 * (1.0 + jy),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::UnifiedRandom;

    /// A scripted `DropRng` recording every `(min,max)` draw — the
    /// draw-order contract mirror (the `projectile_ai_thrown` pattern).
    struct ScriptRng {
        rolls: Vec<i32>,
        draws: Vec<(i32, i32)>,
    }
    impl ScriptRng {
        fn with_rolls(rolls: &[i32]) -> Self {
            Self { rolls: rolls.to_vec(), draws: Vec::new() }
        }
    }
    impl DropRng for ScriptRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.draws.push((min, max));
            self.rolls.remove(0)
        }
        fn next_float(&mut self) -> f32 {
            0.0
        }
    }

    fn world() -> World {
        World::new("t".into(), 64, 800)
    }

    fn spider(pos: Vec2, vel: Vec2) -> Projectile {
        Projectile {
            active: true,
            kind: BABY_SPIDER,
            owner: 0,
            pos,
            vel,
            w: SPIDER_DEFAULTS.0,
            h: SPIDER_DEFAULTS.1,
            time_left: SPIDER_DEFAULTS.2,
            penetrate: SPIDER_DEFAULTS.3,
            max_penetrate: SPIDER_DEFAULTS.3,
            friendly: true,
            ..Default::default()
        }
    }

    fn npc_at(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.max_hp = 40;
        n.hp = 40;
        n
    }

    /// `:29320-29324` — no active owner: the direct deactivate, no draws,
    /// no state advance (the velocity untouched).
    #[test]
    fn w4_d7_s63_the_owner_inactive_kill_is_direct() {
        let w = world();
        let mut p = spider(Vec2::new(100.0, 100.0), Vec2::new(4.0, 1.0));
        spider_step(&mut p, &w, &[], false);
        assert!(!p.active);
        assert_eq!(p.vel, Vec2::new(4.0, 1.0), "the early return skips the machine");
        assert_eq!(p.time_left, SPIDER_DEFAULTS.2, "not Kill() — timeLeft untouched");
    }

    /// `:29342-29346` + `:29413` — no target: the 0.95 X drag, the
    /// fall-through clear, and the +0.2 gravity tail; the AI draws nothing.
    #[test]
    fn w4_d7_s63_no_target_drags_and_clears_fall_through() {
        let w = world();
        let mut rng = UnifiedRandom::new(63);
        let mut control = UnifiedRandom::new(63);
        let mut p = spider(Vec2::new(100.0, 100.0), Vec2::new(4.0, 0.0));
        p.should_fall_through = true;
        spider_step(&mut p, &w, &[], true);
        assert!((p.vel.x - 3.8).abs() < 1e-6, "4 × 0.95");
        assert!(!p.should_fall_through);
        assert!((p.vel.y - 0.2).abs() < 1e-6, "the gravity tail");
        assert_eq!(p.direction, 1);
        assert_eq!(p.sprite_direction, -1);
        assert_eq!(rng.next_range(0, 1), control.next_range(0, 1), "zero draws");
    }

    /// `:29349-29377` — a target BELOW the bottom latches the fall-through
    /// and steers with the grounded 5/0.08 pair (vy == 0 at the check).
    #[test]
    fn w4_d7_s63_the_grounded_pair_steers_toward_the_target() {
        let w = world();
        // Spider centre (107, 105), bottom 110; the zombie's centre sits
        // right and below (the 18×40 zombie box at (145, 121) → (154, 141)).
        let mut p = spider(Vec2::new(100.0, 100.0), Vec2::ZERO);
        let npcs = vec![npc_at(145.0, 121.0)];
        spider_step(&mut p, &w, &npcs, true);
        assert!(p.should_fall_through, "141 > 110");
        assert!((p.vel.x - 0.08).abs() < 1e-6, "the grounded accel");
        // No hop: centre.Y − 50 = 55 is NOT > 141.
        assert!((p.vel.y - 0.2).abs() < 1e-6, "no hop, just gravity");
    }

    /// `:29352-29362` — a grounded spider hops −6 when the target sits
    /// more than 50 px above its centre.
    #[test]
    fn w4_d7_s63_the_grounded_hop_fires_above_50px() {
        let w = world();
        let mut p = spider(Vec2::new(100.0, 200.0), Vec2::ZERO);
        // Centre (107, 205); the target centre (154, 141): 205 − 50 = 155 > 141.
        let npcs = vec![npc_at(145.0, 121.0)];
        spider_step(&mut p, &w, &npcs, true);
        // The hop sets −6 BEFORE the frame check, so the airborne frame 3
        // runs and the gravity tail lands −5.8.
        assert!((p.vel.y - (-5.8)).abs() < 1e-5);
        assert_eq!(p.frame, 3, "vy != 0 at the frame check");
        assert!(!p.should_fall_through, "141 < 210 bottom");
    }

    /// `:29364-29368` + the clamp — any nonzero vy selects the airborne
    /// 8/0.12 pair; an over-speed X clamps to ±8.
    #[test]
    fn w4_d7_s63_the_airborne_pair_and_the_clamp() {
        let w = world();
        let mut p = spider(Vec2::new(100.0, 100.0), Vec2::new(10.0, 1.0));
        let npcs = vec![npc_at(145.0, 121.0)];
        spider_step(&mut p, &w, &npcs, true);
        assert!((p.vel.x - 8.0).abs() < 1e-6, "10 + 0.12 clamps to 8");
        let mut q = spider(Vec2::new(100.0, 100.0), Vec2::new(0.0, -1.0));
        spider_step(&mut q, &w, &npcs, true);
        assert!((q.vel.x - 0.12).abs() < 1e-6, "the airborne accel");
    }

    /// `:18453-18463` — the bounce: X × −0.6 on any X-block; Y × −0.6 only
    /// when lastVy > 2 (a shallow fall keeps its zeroed Y).
    #[test]
    fn w4_d7_s63_the_bounce_ladder() {
        let mut p = spider(Vec2::new(100.0, 100.0), Vec2::new(0.0, 4.0));
        spider_bounce(&mut p, Vec2::new(6.0, 4.0));
        assert!((p.vel.x - (-3.6)).abs() < 1e-6, "X reflects");
        assert_eq!(p.vel.y, 4.0, "Y unchanged — no Y-block");
        let mut q = spider(Vec2::new(100.0, 100.0), Vec2::new(6.0, 0.0));
        spider_bounce(&mut q, Vec2::new(6.0, 4.0));
        assert!((q.vel.y - (-2.4)).abs() < 1e-6, "4 > 2 reflects");
        let mut r = spider(Vec2::new(100.0, 100.0), Vec2::new(6.0, 0.0));
        spider_bounce(&mut r, Vec2::new(6.0, 1.5));
        assert_eq!(r.vel.y, 0.0, "1.5 <= 2 keeps the zeroed Y");
    }

    /// `:11492` — the Venom roll: ONE Next(2,5), buff 70 at 60× the roll.
    #[test]
    fn w4_d7_s63_the_venom_roll_draws_once() {
        let mut n = npc_at(0.0, 0.0);
        let mut rng = ScriptRng::with_rolls(&[3]);
        spider_status_npc(&mut n, &mut rng);
        assert_eq!(rng.draws, vec![(2, 5)]);
        assert_eq!(n.buffs.buff_time_of(70), 180, "60 × 3");
    }

    /// `:75527` — the Kill census: exactly five Next(1,10) scale draws.
    #[test]
    fn w4_d7_s63_the_kill_census_draws_five_scales() {
        let mut rng = ScriptRng::with_rolls(&[1, 2, 3, 4, 5]);
        spider_kill_draws(&mut rng);
        assert_eq!(rng.draws, vec![(1, 10); 5]);
    }

    /// `:77242-77251` — the hatch: Next(2,4) count, Next(5) bonus, then
    /// per spawn the X jitter BEFORE the Y jitter; the spawn rides the
    /// egg's centre with the jittered velocity.
    #[test]
    fn w4_d7_s63_the_egg_hatch_draw_order() {
        let mut egg = spider(Vec2::new(100.0, 100.0), Vec2::new(6.0, -4.0));
        egg.kind = SPIDER_EGG;
        egg.w = 16.0;
        egg.h = 16.0;
        // count 2, bonus hit (0) → 3 spawns; jitters +10/−10, +0/+5, −20/+20.
        let mut rng = ScriptRng::with_rolls(&[2, 0, 10, -10, 0, 5, -20, 20]);
        let spawns = egg_hatch(&egg, 0, &mut rng);
        assert_eq!(
            rng.draws,
            vec![(2, 4), (0, 5), (-20, 21), (-20, 21), (-20, 21), (-20, 21), (-20, 21), (-20, 21)]
        );
        assert_eq!(spawns.len(), 3);
        assert!((spawns[0].x - 108.0).abs() < 1e-6 && (spawns[0].y - 108.0).abs() < 1e-6);
        assert!((spawns[0].vx - 6.0 * 1.10).abs() < 1e-5);
        assert!((spawns[0].vy - (-4.0) * 0.90).abs() < 1e-5);
        assert!((spawns[2].vx - 6.0 * 0.80).abs() < 1e-5);
        assert!((spawns[2].vy - (-4.0) * 1.20).abs() < 1e-5);
    }

    /// `:77240` — a foreign-owned egg (or a non-egg slot) hatches nothing
    /// and draws nothing.
    #[test]
    fn w4_d7_s63_the_hatch_gate_is_kind_and_owner() {
        let mut egg = spider(Vec2::new(100.0, 100.0), Vec2::new(6.0, -4.0));
        egg.kind = SPIDER_EGG;
        egg.owner = 1;
        let mut rng = ScriptRng::with_rolls(&[]);
        assert!(egg_hatch(&egg, 0, &mut rng).is_empty());
        assert!(rng.draws.is_empty());
        egg.owner = 0;
        egg.kind = BABY_SPIDER;
        assert!(egg_hatch(&egg, 0, &mut rng).is_empty());
        assert!(rng.draws.is_empty());
    }
}
