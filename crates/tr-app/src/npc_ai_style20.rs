//! W4-d24 (aiStyle-20) — the Spike Ball, `NPC.AI`'s `if (this.aiStyle ==
//! 20)` block (`NPC.cs:24537-24614`), ported 1:1 as a PURE machine
//! (code-only mode: no compiler/tests were run on this branch; the
//! witness tests below are written, not executed).
//!
//! ## Family census (`SetDefaults` `aiStyle = 20` rows, whole-file grep)
//!
//! ONE vanilla member: SpikeBall 70 (`NPCID.cs` `SpikeBall = 70`,
//! `SetDefaults` `NPC.cs:9680-9696`: npcSlots 0.3, 34×34, dmg 32,
//! def 100, 100 HP, kb 0, NPCHit1/NPCDeath1, noGravity + noTileCollide
//! + dontTakeDamage, scale 1.5). The port has no `NpcKind::SpikeBall`
//! yet — the module lands UNWIRED (the `npc_ai_style17.rs` precedent).
//!
//! ## The machine — the chain-ball launch + bounce
//!
//! * the init (`:24539-24568`, `ai[0] == 0`): `TargetClosest()`, then
//!   BOTH direction axes flip (`direction *= -1`, `directionY *= -1`
//!   — the pick's facing INVERTED: the ball launches AWAY), the body
//!   drops `height / 2 + 8` (INTEGER half), the anchor latches
//!   `ai[1..2]` at the post-drop INTEGER-half centre, the zero-axis
//!   pins (+1), the ONE speed roll `ai[3] = 1 + Next(15) * 0.1`
//!   (1.0..2.4), the launch `vy = directionY * 6 * ai[3]`, `ai[0]++`.
//!   The client fold (`netMode == 1`, `:24562-24566`) only latches the
//!   anchor — single-player never takes it. Either way the init tick
//!   RETURNS (no movement).
//! * the climb (`:24569-24577`): `num287 = 6 * ai[3]` (the speed),
//!   `num288 = 0.2 * ai[3]` (the accel), `num289 = num287/num288/2`
//!   (= 15 exactly in real maths, but the f32 division chain can land
//!   a hair under — the `(int)` truncation is load-bearing, verbatim).
//!   `1 <= ai[0] < (int)num289`: `vy = directionY * num287`, `ai[0]++`.
//! * the turn (`:24578-24585`): `ai[0] >= (int)num289`: `vy = 0`,
//!   `directionY` flips, `vx = num287 * direction`, `ai[0] = -1`.
//! * the bounce (`:24586-24613`, `ai[0] == -1`): the four axis-end
//!   reflections (each flips its axis AND pins the velocity to the
//!   ±num287 end), then the accel `vx += num288 * direction`,
//!   `vy += num288 * directionY`. The arm has NO trailing return — it
//!   is the chain's last fall-through; `AI()` ends at `:41132`.
//!
//! Seam: writes velocity, `ai[0..3]`, `direction`/`directionY`,
//! `position.Y` (the init drop — the arm's ONE position write),
//! `netUpdate`. Does NOT integrate: the ball is noGravity +
//! noTileCollide (`SetDefaults`), so the caller's `UpdateCollision`
//! raw-integrates (`NPC.cs:90494-90498`, the `integrate_no_tile_collide`
//! seam). `Main.netMode != 1` folds true (single-player).
//!
//! ## RNG census
//!
//! The arm's ONLY `Main.rand` draw is the init speed roll `Next(15)`
//! (`:24557`) — exactly ONE, on the `ai[0] == 0` tick, AFTER the
//! direction flips / the drop / the anchor latch / the zero pins and
//! BEFORE the launch write. Every later tick draws NOTHING.
//! `TargetClosest` (`NPC.cs:77652-77781`) draws nothing. The witnesses
//! mirror the roll on a cloned stream and pin the draw-free bounce.
#![allow(dead_code)] // UNWIRED (W4-d24): the SpikeBall kind + the axis claim are the wiring step

use crate::npc::Npc;
use crate::npc_target::{target_closest, TargetCandidate};
use tr_core::{UnifiedRandom, World};

/// The SpikeBall's `SetDefaults` row (`NPC.cs:9680-9696`), verbatim.
pub struct SpikeBallDefaults {
    pub npc_slots: f32,
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    pub knock_back_resist: f32,
    pub hit_sound: (i32, i32),
    pub death_sound: (i32, i32),
    pub no_gravity: bool,
    pub no_tile_collide: bool,
    pub dont_take_damage: bool,
    pub scale: f32,
}

/// `NPC.cs:9680-9696` — `else if (this.type == 70)`.
pub const SPIKE_BALL: SpikeBallDefaults = SpikeBallDefaults {
    npc_slots: 0.3,
    width: 34,
    height: 34,
    ai_style: 20,
    damage: 32,
    defense: 100,
    life_max: 100,
    knock_back_resist: 0.0,
    hit_sound: (3, 1),
    death_sound: (4, 1),
    no_gravity: true,
    no_tile_collide: true,
    dont_take_damage: true,
    scale: 1.5,
};

/// The aiStyle-20 arm (`NPC.cs:24537-24614`). One fixed step; the ONE
/// `Next(15)` speed roll rides the caller's stream on the init tick.
pub fn ai_style20(
    npc: &mut Npc,
    world: &World,
    players: &[TargetCandidate],
    ticker: u64,
    rng: &mut UnifiedRandom,
) {
    let (w, h) = npc.size();
    if npc.ai[0] == 0.0 {
        // `:24541-24561` — the host init (netMode != 1 folds true).
        target_closest(npc, world, players, true, 20, ticker);
        // `:24544-24545` — BOTH axes flip (the away-launch).
        npc.facing *= -1;
        npc.direction_y *= -1;
        // `:24546` — the INTEGER-half drop.
        npc.y += (h as i32 / 2 + 8) as f32;
        // `:24547-24548` — the anchor at the post-drop centre.
        npc.ai[1] = npc.x + (w as i32 / 2) as f32;
        npc.ai[2] = npc.y + (h as i32 / 2) as f32;
        // `:24549-24556` — the zero-axis pins.
        if npc.facing == 0 {
            npc.facing = 1;
        }
        if npc.direction_y == 0 {
            npc.direction_y = 1;
        }
        // `:24557` — the ONE speed roll: 1.0 .. 2.4.
        npc.ai[3] = 1.0 + rng.next_max(15) as f32 * 0.1;
        // `:24558-24560` — the launch + the arm.
        npc.vy = (npc.direction_y * 6) as f32 * npc.ai[3];
        npc.ai[0] += 1.0;
        npc.net_update = true;
        // `:24567` — the init tick returns (no movement).
        return;
    }
    // `:24569-24571` — the speed/accel/half-period. The f32 division
    // chain + the `(int)` truncation are verbatim (num289 is 15 in real
    // maths; the float chain may land 14.999999 → 14).
    let num287 = 6.0 * npc.ai[3];
    let num288 = 0.2 * npc.ai[3];
    let num289 = num287 / num288 / 2.0;
    if npc.ai[0] >= 1.0 && npc.ai[0] < num289 as i32 as f32 {
        // `:24572-24577` — the climb.
        npc.vy = npc.direction_y as f32 * num287;
        npc.ai[0] += 1.0;
        return;
    }
    if npc.ai[0] >= num289 as i32 as f32 {
        // `:24578-24585` — the turn: vy zero, the Y flip, the X launch.
        npc.vy = 0.0;
        npc.direction_y *= -1;
        npc.vx = num287 * npc.facing as f32;
        npc.ai[0] = -1.0;
        return;
    }
    // `:24586-24598` — the Y axis-end reflection.
    if npc.direction_y > 0 {
        if npc.vy >= num287 {
            npc.direction_y *= -1;
            npc.vy = num287;
        }
    } else if npc.direction_y < 0 && npc.vy <= -num287 {
        npc.direction_y *= -1;
        npc.vy = -num287;
    }
    // `:24599-24611` — the X axis-end reflection.
    if npc.facing > 0 {
        if npc.vx >= num287 {
            npc.facing *= -1;
            npc.vx = num287;
        }
    } else if npc.facing < 0 && npc.vx <= -num287 {
        npc.facing *= -1;
        npc.vx = -num287;
    }
    // `:24612-24613` — the accel tail (no return — the chain's end).
    npc.vx += num288 * npc.facing as f32;
    npc.vy += num288 * npc.direction_y as f32;
}

#[cfg(test)]
mod tests {
    //! Witness tests — WRITTEN, NOT RUN (code-only mode). Each pins one
    //! C# branch against its citation (`NPC.cs:24537-24614`).
    use super::*;
    use crate::npc::NpcKind;

    fn air() -> World {
        World::new("t".into(), 200, 120)
    }

    fn cand(x: f32, y: f32) -> TargetCandidate {
        TargetCandidate {
            active: true,
            dead: false,
            ghost: false,
            x,
            y,
            width: 20,
            height: 42,
            aggro: 0,
            item_animation: 0,
            npc_type_no_aggro: false,
            tank_pet: None,
        }
    }

    /// A stand-in body with the SpikeBall's 34×34 box (the kind is
    /// unported; the machine is the family's only member).
    fn ball(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::DemonEye, x, y, 1, 0.0);
        n.size_override = Some((34.0, 34.0));
        n.hp = 100;
        n.max_hp = 100;
        n
    }

    /// The `SetDefaults` row (`NPC.cs:9680-9696`).
    #[test]
    fn the_defaults_row_matches_the_census() {
        assert_eq!(SPIKE_BALL.npc_slots, 0.3);
        assert_eq!((SPIKE_BALL.width, SPIKE_BALL.height), (34, 34));
        assert_eq!(SPIKE_BALL.ai_style, 20);
        assert_eq!((SPIKE_BALL.damage, SPIKE_BALL.defense, SPIKE_BALL.life_max), (32, 100, 100));
        assert_eq!(SPIKE_BALL.knock_back_resist, 0.0);
        assert_eq!(SPIKE_BALL.hit_sound, (3, 1));
        assert_eq!(SPIKE_BALL.death_sound, (4, 1));
        assert!(SPIKE_BALL.no_gravity && SPIKE_BALL.no_tile_collide && SPIKE_BALL.dont_take_damage);
        assert_eq!(SPIKE_BALL.scale, 1.5);
    }

    /// `:24539-24567` — the init: the pick's facing INVERTS, the body
    /// drops 25 (34/2 + 8), the anchor latches at the post-drop centre,
    /// the ONE Next(15) roll sets ai[3] and the launch vy.
    #[test]
    fn the_init_flips_drops_and_rolls_once() {
        let w = air();
        let mut n = ball(1000.0, 1000.0);
        // The player below-right: the pick faces (+1, +1).
        let p = cand(1100.0, 1100.0);
        let mut rng = UnifiedRandom::new(3);
        let mut mirror = UnifiedRandom::new(3);
        let roll = mirror.next_max(15);
        ai_style20(&mut n, &w, &[p], 7, &mut rng);
        assert_eq!(n.facing, -1, "the pick's +1 inverted");
        assert_eq!(n.direction_y, -1, "the pick's +1 inverted");
        assert_eq!(n.y, 1025.0, "the 17 + 8 drop");
        assert_eq!(n.ai[1], 1017.0, "the post-drop centre X");
        assert_eq!(n.ai[2], 1042.0, "the post-drop centre Y");
        let speed = 1.0 + roll as f32 * 0.1;
        assert_eq!(n.ai[3], speed);
        assert_eq!(n.vy, -6.0 * speed, "the away-launch");
        assert_eq!(n.ai[0], 1.0);
        assert!(n.net_update);
        assert_eq!(rng, mirror, "exactly the one Next(15)");
    }

    /// `:24549-24556` — the zero-axis pins: a 0 facing/directionY
    /// (the flip of 0 is 0) pins to +1 AFTER the anchor latch.
    #[test]
    fn the_zero_axes_pin_to_one() {
        let w = air();
        let mut n = ball(1000.0, 1000.0);
        // A fresh NPC enters the AI with both axes at the C# field
        // default 0 (type 70's SetDefaults writes neither direction nor
        // directionY); the fixture starts there so the picks stay 0.
        n.facing = 0;
        n.direction_y = 0;
        // A dead player: the pick leaves the axes at 0.
        let mut p = cand(1100.0, 1100.0);
        p.dead = true;
        let mut rng = UnifiedRandom::new(3);
        ai_style20(&mut n, &w, &[p], 0, &mut rng);
        assert_eq!(n.facing, 1);
        assert_eq!(n.direction_y, 1);
        assert!(n.vy > 0.0, "the pinned +1 launches down");
    }

    /// `:24569-24585` — the climb counts to (int)num289, then the turn:
    /// vy 0, the Y flip, vx = speed * direction, ai[0] = −1.
    #[test]
    fn the_climb_turns_at_the_truncated_half_period() {
        let w = air();
        let mut n = ball(1000.0, 1000.0);
        n.ai[0] = 1.0;
        n.ai[3] = 1.0; // num287 = 6, num288 = 0.2, num289 = 15
        n.direction_y = -1;
        n.facing = 1;
        let mut rng = UnifiedRandom::new(3);
        let witness = rng.clone();
        let period = (6.0f32 / 0.2 / 2.0) as i32;
        for i in 1..period {
            ai_style20(&mut n, &w, &[cand(1100.0, 1100.0)], 0, &mut rng);
            assert_eq!(n.ai[0], (i + 1) as f32, "the climb beat {i}");
            assert_eq!(n.vy, -6.0, "the constant climb speed");
        }
        // The turn tick.
        ai_style20(&mut n, &w, &[cand(1100.0, 1100.0)], 0, &mut rng);
        assert_eq!(n.ai[0], -1.0);
        assert_eq!(n.vy, 0.0);
        assert_eq!(n.direction_y, 1, "the Y flip");
        assert_eq!(n.vx, 6.0, "the X launch");
        assert_eq!(rng, witness, "the climb + turn draw nothing");
    }

    /// `:24586-24613` — the bounce: the axis-end reflections flip + pin,
    /// then the accel tail adds on the NEW axes.
    #[test]
    fn the_bounce_reflects_pins_and_accelerates() {
        let w = air();
        let mut n = ball(1000.0, 1000.0);
        n.ai[0] = -1.0;
        n.ai[3] = 1.0; // speed 6, accel 0.2
        n.direction_y = 1;
        n.facing = 1;
        n.vx = 5.9;
        n.vy = 6.0; // at the +Y end → flip + pin
        let mut rng = UnifiedRandom::new(3);
        let witness = rng.clone();
        ai_style20(&mut n, &w, &[cand(1100.0, 1100.0)], 0, &mut rng);
        assert_eq!(n.direction_y, -1, "the Y end flips");
        // vy: pinned 6, then the accel on the NEW −1 axis: 6 − 0.2.
        assert!((n.vy - 5.8).abs() < 1e-5, "the pin + the new-axis accel");
        // vx: 5.9 < 6 (no flip), + 0.2 = 6.1.
        assert!((n.vx - 6.1).abs() < 1e-5);
        // The next tick: vx 6.1 >= 6 → the X flip + pin, then −0.2.
        ai_style20(&mut n, &w, &[cand(1100.0, 1100.0)], 0, &mut rng);
        assert_eq!(n.facing, -1);
        assert!((n.vx - 5.8).abs() < 1e-5, "the X pin + the new-axis accel");
        assert_eq!(rng, witness, "the bounce draws nothing");
    }

    /// The RNG census: after the init, 500 mixed climb/turn/bounce
    /// ticks draw NOTHING — the stream state is pinned.
    #[test]
    fn the_post_init_census_draws_no_rng() {
        let w = air();
        let mut n = ball(1000.0, 1000.0);
        let mut rng = UnifiedRandom::new(17);
        // The init consumes the one roll.
        ai_style20(&mut n, &w, &[cand(1100.0, 1100.0)], 0, &mut rng);
        let witness = rng.clone();
        for _ in 0..500 {
            ai_style20(&mut n, &w, &[cand(1100.0, 1100.0)], 0, &mut rng);
        }
        assert_eq!(rng, witness, "500 post-init ticks draw nothing");
        assert_eq!(n.ai[0], -1.0, "the ball is in the bounce phase");
    }
}
