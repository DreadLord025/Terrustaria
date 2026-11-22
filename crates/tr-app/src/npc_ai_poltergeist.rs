//! The Poltergeist's aiStyle-22 machine (S1-F-13b2 slice C) — `NPC.AI()`'s
//! `else if (this.aiStyle == 22)` arm (`NPC.cs:24666-25234`) with the
//! type-330 (Poltergeist) branches only: the pumpkin-moon flee gate
//! (`:24670-24673`), the dwell anchor (`ai[0..2]`, the ±16/±40 windows,
//! `:24722-24764`), the 60-tick flip into the −200 dash that creeps back
//! at 0.1/tick (`:24747-24754`, `:24772-24775`), the fall gate
//! (`Bottom.Y > player.Top.Y` → no gravity, `:24971-24976`), the 330 row
//! (alpha 0, speed cap 4, `TargetClosest`/`EncourageDespawn(10)`, the 0.9
//! opposite-velocity damping, `:25148-25168`) and the shared X/Y approach
//! tails (`:25169-25233`). `SetDefaults` (`NPC.cs:13188-13203`): 24x44,
//! aiStyle 22, dmg 90, def 44, 1250 HP, alpha 100, kb 0.4, noGravity +
//! noTileCollide — the identity half landed with S1-F-13b.
//!
//! In 1.4.5.8 the type-330 path carries NO `Projectile.NewProjectile` —
//! the arm's shot blocks belong to types 122/169/268 only; the
//! Poltergeist's offence is its 90-damage contact body. The one RNG draw
//! on the whole path is the flee branch's `Main.rand.Next(-1, 2) * 1.5f`
//! when `velocity.X == 0f` (`:24716-24720`) — it rides the caller's
//! `UpdateNPCs` stream (`Main.cs:17767`), exactly once per such tick.
//!
//! Deliberate stand-ins (house pattern, recorded in ANALYSIS):
//! - `justHit` (`:24682-24685`) folds onto `hurt_cd > 0` (the port's
//!   per-tick hurt cooldown, as in the WoF/Deerclops machines).
//! - `npcTypeNoAggro` (`:24997-25013`) is constant-false in the port (no
//!   aggro table yet) — the scan never runs, as in `npc_target.rs`.
//! - `flag36` (`AI_GlobalFiringDistanceCheck`, `:24795`) is computed but
//!   read only by the 122/268 shot arms — a pure no-op for 330, dropped.
//! - `flag30`'s only setters are the type-316 arm and the 169/268 roof
//!   scan; the `ai[2] >= 30 && num291 == 16` test (`:24743`) is dead C#
//!   (`num291` is always 40 there). For 330 `flag30` is constant-false,
//!   so `:25030-25038` never fires and is not carried.

use crate::npc::{Npc, NpcKind};
use crate::player::Player;
use tr_core::{UnifiedRandom, World};

/// `NPC.TargetClosest()` (`NPC.cs:77652-77666`) through the shared
/// selection (`npc_target.rs`) — the single session player as slot 0,
/// the stock aggro/no-aggro/tank values (the S1-D-3p convention).
fn target_closest(npc: &mut Npc, world: &World, player: &Player) {
    let cand = crate::npc_target::TargetCandidate {
        active: true,
        dead: player.dead,
        ghost: false,
        x: player.x,
        y: player.y,
        width: crate::player::WIDTH as i32,
        height: crate::player::HEIGHT as i32,
        aggro: 0,
        item_animation: player.frame_inputs.swing_anim as i32,
        npc_type_no_aggro: false,
        tank_pet: None,
    };
    // The `EverLastingTicker` stamp's only reader is the unported
    // target-line fade — 0 until the caller migration (S1-D-3p).
    crate::npc_target::target_closest(npc, world, &[cand], true, 22, 0);
}

/// One fixed step of the Poltergeist (type 330) through the aiStyle-22
/// arm (`NPC.cs:24666-25234`), ending with the `noTileCollide` half of
/// `UpdateCollision` (`:92538-92540`'s `oldVelocity` capture + flag
/// reset, `:92605-92607`'s `position += velocity`). The caller's `rng`
/// is the `UpdateNPCs` stream; the machine draws from it only on a flee
/// tick with `velocity.X == 0f` (`:24718`).
pub fn poltergeist_ai(npc: &mut Npc, world: &World, player: &Player, rng: &mut UnifiedRandom) {
    debug_assert_eq!(npc.kind, NpcKind::Poltergeist);
    // `:24668-24673` — flag31: the Poltergeist flees outside the Pumpkin
    // Moon. (flag30 is constant-false for 330 — see the module note.)
    let flee = !world.pumpkin_moon;
    // `:24682-24685` — `justHit` resets the dwell/dash counter.
    if npc.hurt_cd > 0.0 {
        npc.ai[2] = 0.0;
    }
    if flee {
        // `:24714-24721` — the flee drift: one `Next(-1, 2)` draw ONLY
        // while the body is parked; the C# arms `netUpdate` with it.
        if npc.vx == 0.0 {
            npc.vx = rng.next_range(-1, 2) as f32 * 1.5;
            npc.net_update = true;
        }
    } else if npc.ai[2] >= 0.0 {
        // `:24722-24764` — the dwell anchor: while the body holds the
        // ±16 px X / ±40 px Y window around `ai[0..1]` (or drifts back
        // against its facing), `ai[2]` counts up; at 60 the ghost flips
        // into the −200 dash. Outside the window the anchor re-latches.
        let mut num291 = 16.0;
        let mut near_x = false;
        let mut near_y = false;
        if npc.x > npc.ai[0] - num291 && npc.x < npc.ai[0] + num291 {
            near_x = true;
        } else if (npc.vx < 0.0 && npc.facing > 0) || (npc.vx > 0.0 && npc.facing < 0) {
            near_x = true;
        }
        num291 += 24.0;
        if npc.y > npc.ai[1] - num291 && npc.y < npc.ai[1] + num291 {
            near_y = true;
        }
        if near_x && near_y {
            npc.ai[2] += 1.0;
            // `:24743-24746`'s `ai[2] >= 30 && num291 == 16` is dead C#
            // (num291 is 40 here) — not carried.
            if npc.ai[2] >= 60.0 {
                // `:24747-24754`.
                npc.ai[2] = -200.0;
                npc.facing *= -1;
                npc.vx *= -1.0;
                npc.collide_x = false;
                npc.net_update = true;
            }
        } else {
            // `:24756-24762`.
            npc.ai[0] = npc.x;
            npc.ai[1] = npc.y;
            npc.ai[2] = 0.0;
            npc.net_update = true;
        }
        // `:24763`.
        target_closest(npc, world, player);
    } else {
        // `:24770-24788` — the dash: the 330 row creeps `ai[2]` at 0.1
        // (the others at 1), and the facing points AWAY from the player
        // (`:24780-24787`; the tail's `TargetClosest` re-faces it, exactly
        // as the C# does — the away-facing only steers the tile probe).
        npc.ai[2] += 0.1;
        let (pcx, _) = player.center();
        let (cx, _) = npc.center();
        npc.facing = if pcx > cx { -1 } else { 1 };
    }
    // `:24789-24795` — the tile probe coords + flag36 feed only the
    // unported type arms (122/75/169/268/490) — no-ops for 330.
    // `:24971-24976` — the fall gate: at or below the player's top edge
    // the 330 row skips the floor scan AND the gravity outright.
    let (_, h) = npc.size();
    let fall = !(npc.y + h > player.y);
    // `:25039-25072` — the gravity arm (the generic else group: +0.1,
    // cap 3; the 316-flee special case does not apply).
    if fall {
        npc.vy += 0.1;
        if npc.vy > 3.0 {
            npc.vy = 3.0;
        }
    } else {
        // `:25094-25101` — the rise brake + the −4 floor.
        if npc.direction_y < 0 && npc.vy > 0.0 {
            npc.vy -= 0.1;
        }
        if npc.vy < -4.0 {
            npc.vy = -4.0;
        }
    }
    // `:25111-25134` — the collision bounces. `noTileCollide` keeps both
    // flags false (the tail resets them), so these are faithful no-ops.
    if npc.collide_x {
        npc.vx = npc.old_vx * -0.4;
        if npc.facing == -1 && npc.vx > 0.0 && npc.vx < 1.0 {
            npc.vx = 1.0;
        }
        if npc.facing == 1 && npc.vx < 0.0 && npc.vx > -1.0 {
            npc.vx = -1.0;
        }
    }
    if npc.collide_y {
        npc.vy = npc.old_vy * -0.25;
        if npc.vy > 0.0 && npc.vy < 1.0 {
            npc.vy = 1.0;
        }
        if npc.vy < 0.0 && npc.vy > -1.0 {
            npc.vy = -1.0;
        }
    }
    // `:25148-25168` — the 330 row: the machine zeroes the SetDefaults
    // alpha 100 every tick, the speed cap is 4, the live moon re-picks
    // the target while the off-moon ghost takes the 10-tick despawn
    // encouragement, and the opposite-direction velocity damps at 0.9.
    npc.alpha = 0;
    let max_x = 4.0;
    if !flee {
        target_closest(npc, world, player);
    } else {
        npc.encourage_despawn(10);
    }
    if npc.facing < 0 && npc.vx > 0.0 {
        npc.vx *= 0.9;
    }
    if npc.facing > 0 && npc.vx < 0.0 {
        npc.vx *= 0.9;
    }
    // `:25169-25200` — the X approach (0.1 accel, the 0.1 double-add
    // past the far cap, the 0.05 near-zero brake, the ±4 clamp).
    if npc.facing == -1 && npc.vx > 0.0 - max_x {
        npc.vx -= 0.1;
        if npc.vx > max_x {
            npc.vx -= 0.1;
        } else if npc.vx > 0.0 {
            npc.vx += 0.05;
        }
        if npc.vx < 0.0 - max_x {
            npc.vx = 0.0 - max_x;
        }
    } else if npc.facing == 1 && npc.vx < max_x {
        npc.vx += 0.1;
        if npc.vx < 0.0 - max_x {
            npc.vx += 0.1;
        } else if npc.vx < 0.0 {
            npc.vx -= 0.05;
        }
        if npc.vx > max_x {
            npc.vx = max_x;
        }
    }
    // `:25201-25233` — the Y approach (num320 = 1.5 for every non-490
    // type: 0.04 accel, 0.05 double-add, 0.03 brake, the ±1.5 clamp).
    let max_y = 1.5;
    if npc.direction_y == -1 && npc.vy > 0.0 - max_y {
        npc.vy -= 0.04;
        if npc.vy > max_y {
            npc.vy -= 0.05;
        } else if npc.vy > 0.0 {
            npc.vy += 0.03;
        }
        if npc.vy < 0.0 - max_y {
            npc.vy = 0.0 - max_y;
        }
    } else if npc.direction_y == 1 && npc.vy < max_y {
        npc.vy += 0.04;
        if npc.vy < 0.0 - max_y {
            npc.vy += 0.05;
        } else if npc.vy < 0.0 {
            npc.vy -= 0.03;
        }
        if npc.vy > max_y {
            npc.vy = max_y;
        }
    }
    // `UpdateCollision` (`:92538-92540`) + `Collision_MoveWhileDry`
    // (`:92605-92607`): `oldVelocity` captured, the collide flags reset,
    // and the `noTileCollide` body integrates straight through tiles.
    npc.old_vx = npc.vx;
    npc.old_vy = npc.vy;
    npc.collide_x = false;
    npc.collide_y = false;
    npc.x += npc.vx;
    npc.y += npc.vy;
}

/// W4-e9 — the Ghost's (type 316) flee cry (`NPC.cs:24686-24701`, the
/// same aiStyle-22 arm's 316 branch): when the target is dead or farther
/// than 3000 px and `ai[3] == 1` (the fade state), the FIRST such tick
/// (`localAI[3] == 0`) latches `localAI[3] = 1` and plays
/// `SoundID.NPCDeath55` at `(int)base.Center.X/Y` — the style façade
/// (`SoundEngine.cs:93-99`, the NextFloat rides the audio-only stream),
/// case 4's one pitch draw on the `UpdateNPCs` stream. Queues on
/// `wants_sound`; RNG-free (the push draws nothing). Returns whether the
/// latch fired (the caller runs the `Opacity -= 0.025` fade + the
/// `active = false` drop — the port's `Npc` has no Opacity field yet, the
/// 316 kind's wiring owns it). The `ai[3] != 1` else-arm's
/// `TargetClosest`/`EncourageDespawn(10)` is the 316 machine's, not this
/// sound seam's.
#[allow(dead_code)] // the Ghost 316 kind's wiring consumes the cry
pub fn ghost_flee_cry(npc: &mut Npc, target_dead: bool, target_distance: f32) -> bool {
    // `:24686` — the outer gate: dead target OR distance > 3000.
    if !(target_dead || target_distance > 3000.0) {
        return false;
    }
    // `:24688-24694` — the fade state + the one-shot latch.
    if npc.ai[3] != 1.0 || npc.local_ai[3] != 0.0 {
        return false;
    }
    npc.local_ai[3] = 1.0;
    let (cx, cy) = npc.center();
    npc.wants_sound.push(crate::npc::NpcSoundRequest {
        cue: crate::npc::NpcSoundCue::NpcDeath55,
        x: cx,
        y: cy,
    });
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::TILE_PX;

    /// A flat test world (the S1-F-13b shape): solid ground rows
    /// 100..120, the session player at (120, 95) tiles.
    fn world() -> World {
        let mut w = World::new("t".into(), 200, 120);
        for x in 0..200 {
            for y in 100..120 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::tile::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.pumpkin_moon = true;
        w
    }

    fn player(w: &World) -> Player {
        let mut p = Player::spawn(w);
        p.x = 120.0 * TILE_PX;
        p.y = 95.0 * TILE_PX;
        p
    }

    fn polt(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::Poltergeist, x, y, 1, 0.0);
        n.hp = n.max_hp;
        n
    }

    /// The first tick re-latches the dwell anchor (`:24756-24762`):
    /// `ai[0..1]` take the position, `ai[2]` rests at 0, `netUpdate`
    /// arms — and the 330 row zeroes the SetDefaults alpha 100.
    #[test]
    fn s1_f_13b2c_the_first_tick_latches_the_dwell_anchor() {
        let w = world();
        let p = player(&w);
        let mut n = polt(60.0 * TILE_PX, 60.0 * TILE_PX);
        assert_eq!(n.alpha, 100, "the SetDefaults row spawns at 100");
        let mut rng = UnifiedRandom::new(7);
        poltergeist_ai(&mut n, &w, &p, &mut rng);
        // The anchor latched at the PRE-integration position.
        assert_eq!(n.ai[0], 60.0 * TILE_PX);
        assert_eq!(n.ai[1], 60.0 * TILE_PX);
        assert_eq!(n.ai[2], 0.0);
        assert!(n.net_update);
        assert_eq!(n.alpha, 0, "the machine zeroes the alpha (:25150)");
        assert_eq!(n.facing, 1, "the player is to the right");
    }

    /// While the body holds the ±16/±40 window the counter climbs 1 per
    /// tick (`:24742`); the X approach drifts 0.1/tick so the first few
    /// ticks stay inside the window.
    #[test]
    fn s1_f_13b2c_the_dwell_counter_climbs_inside_the_window() {
        let w = world();
        let p = player(&w);
        let mut n = polt(60.0 * TILE_PX, 60.0 * TILE_PX);
        let mut rng = UnifiedRandom::new(7);
        poltergeist_ai(&mut n, &w, &p, &mut rng); // latch, ai[2] = 0
        for tick in 1..=5 {
            poltergeist_ai(&mut n, &w, &p, &mut rng);
            assert_eq!(n.ai[2], tick as f32, "tick {tick}");
        }
    }

    /// At `ai[2] == 60` the ghost flips into the −200 dash
    /// (`:24747-24754`): the counter rests at −200, the facing and vx
    /// flip, `collideX` clears, `netUpdate` arms. The next tick creeps
    /// the counter at exactly 0.1 (`:24774`).
    #[test]
    fn s1_f_13b2c_the_sixty_tick_flip_arms_the_dash() {
        let w = world();
        let p = player(&w);
        let mut n = polt(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.ai = [n.x, n.y, 59.0, 0.0];
        n.vx = 2.0;
        n.facing = 1;
        let mut rng = UnifiedRandom::new(7);
        poltergeist_ai(&mut n, &w, &p, &mut rng);
        assert_eq!(n.ai[2], -200.0, "the 60th dwell tick flips the dash");
        assert!(n.net_update);
        assert!(!n.collide_x);
        // The tail's TargetClosest re-faces the player (the C# order).
        assert_eq!(n.facing, 1, "the player is right; the flip is re-faced");
        let before = n.ai[2];
        poltergeist_ai(&mut n, &w, &p, &mut rng);
        assert_eq!(n.ai[2], before + 0.1, "the dash creeps at 0.1/tick");
    }

    /// The dash facing points AWAY from the player before the tail's
    /// `TargetClosest` re-faces it — observable through the 0.9 damping
    /// order: a body moving toward the player is never damped.
    #[test]
    fn s1_f_13b2c_the_dash_creeps_and_the_cap_is_four() {
        let w = world();
        let p = player(&w);
        let mut n = polt(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.ai = [0.0, 0.0, -100.0, 0.0];
        n.vx = 3.95;
        let mut rng = UnifiedRandom::new(7);
        poltergeist_ai(&mut n, &w, &p, &mut rng);
        assert_eq!(n.ai[2], -100.0f32 + 0.1);
        // 3.95 + 0.1 = 4.05 → the ±4 clamp (`:25196-25199`).
        assert_eq!(n.vx, 4.0);
        // Above the cap the approach does not run — no clamp-down.
        n.vx = 10.0;
        poltergeist_ai(&mut n, &w, &p, &mut rng);
        assert_eq!(n.vx, 10.0, "the C# clamps only inside the approach");
    }

    /// W4-e9 — `NPC.cs:24686-24694`: the Ghost's flee cry latches ONCE
    /// (`localAI[3]` 0→1), queues the NPCDeath55 façade at the CENTRE,
    /// and draws nothing; a live close target, the non-fade state and the
    /// already-latched tick all queue nothing.
    #[test]
    fn w4_e9_the_ghost_flee_cry_latches_once_at_the_centre() {
        let w = world();
        let mut n = polt(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.ai[3] = 1.0; // the fade state
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        // A live, close target: no cry.
        assert!(!ghost_flee_cry(&mut n, false, 100.0));
        assert!(n.wants_sound.is_empty());
        // Dead target: the latch fires, ONE queue entry at the centre.
        assert!(ghost_flee_cry(&mut n, true, 100.0));
        assert_eq!(n.local_ai[3], 1.0);
        assert_eq!(n.wants_sound.len(), 1);
        let req = n.wants_sound[0];
        assert_eq!(req.cue, crate::npc::NpcSoundCue::NpcDeath55);
        let (cx, cy) = n.center();
        assert_eq!((req.x, req.y), (cx, cy), "the C# plays at Center");
        // The latched tick never re-queues (the fade continues silently).
        assert!(!ghost_flee_cry(&mut n, true, 100.0));
        assert_eq!(n.wants_sound.len(), 1);
        // The distance arm (> 3000) fires the same latch on a fresh body.
        let mut m = polt(60.0 * TILE_PX, 60.0 * TILE_PX);
        m.ai[3] = 1.0;
        assert!(ghost_flee_cry(&mut m, false, 3000.5));
        // ai[3] != 1 (not yet fading): nothing.
        let mut k = polt(60.0 * TILE_PX, 60.0 * TILE_PX);
        assert!(!ghost_flee_cry(&mut k, true, 5000.0));
        assert!(k.wants_sound.is_empty());
        assert_eq!(rng.peek(), peek, "the latch + push draw nothing");
    }

    /// The left-facing mirror: −3.95 → −4.0, and the opposite-direction
    /// 0.9 damping + the 0.1/0.05 approach steps (`:25160-25199`).
    #[test]
    fn s1_f_13b2c_the_left_approach_and_the_opposite_damping() {
        let w = world();
        let mut p = player(&w);
        p.x = 10.0 * TILE_PX; // the player is LEFT of the ghost
        let mut n = polt(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.vx = -3.95;
        let mut rng = UnifiedRandom::new(7);
        poltergeist_ai(&mut n, &w, &p, &mut rng);
        assert_eq!(n.facing, -1);
        assert_eq!(n.vx, -4.0);
        // Facing left while moving right: ×0.9, then −0.1, then the
        // >0 brake +0.05 (`:25160-25163`, `:25169-25179`).
        n.vx = 2.0;
        poltergeist_ai(&mut n, &w, &p, &mut rng);
        let mut expect = 2.0f32 * 0.9;
        expect -= 0.1;
        expect += 0.05;
        assert_eq!(n.vx, expect);
    }

    /// Below the player's top edge the 330 row skips gravity
    /// (`:24973-24975`) and the Y approach runs at 0.04 with the ±1.5
    /// clamp (`:25201-25217`).
    #[test]
    fn s1_f_13b2c_the_fall_gate_and_the_y_approach() {
        let w = world();
        let mut p = player(&w);
        // The player's top edge ABOVE the ghost's bottom, centre BELOW
        // the ghost's centre → fall = false, directionY = −1.
        p.x = 60.0 * TILE_PX;
        p.y = 60.0 * TILE_PX - 10.0;
        let mut n = polt(60.0 * TILE_PX, 60.0 * TILE_PX);
        let mut rng = UnifiedRandom::new(7);
        poltergeist_ai(&mut n, &w, &p, &mut rng);
        assert_eq!(n.direction_y, -1);
        assert_eq!(
            n.vy, -0.04,
            "no gravity below the top edge; the approach steps"
        );
        // −1.47 − 0.04 = −1.51 → the −1.5 clamp.
        n.vy = -1.47;
        poltergeist_ai(&mut n, &w, &p, &mut rng);
        assert_eq!(n.vy, -1.5);
        // Above the player (fall = true): the generic +0.1 / cap-3 arm.
        p.y = 95.0 * TILE_PX;
        n.vy = 2.95;
        poltergeist_ai(&mut n, &w, &p, &mut rng);
        assert_eq!(n.vy, 3.0);
    }

    /// RNG witness: the flee branch draws exactly one `Next(-1, 2)`
    /// when parked (`:24716-24720`) and arms the 10-tick despawn
    /// encouragement (`:25158`); the stream advances by exactly that
    /// one draw.
    #[test]
    fn s1_f_13b2c_the_flee_drift_draws_one_next_on_the_caller_stream() {
        let mut w = world();
        w.pumpkin_moon = false;
        let p = player(&w);
        let mut n = polt(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.vx = 0.0;
        let mut rng = UnifiedRandom::new(42);
        let mut witness = rng.clone();
        let draw = witness.next_range(-1, 2);
        poltergeist_ai(&mut n, &w, &p, &mut rng);
        assert_eq!(rng.peek(), witness.peek(), "exactly one draw, in order");
        // The drawn drift feeds the approach tail; re-derive it.
        let mut expect = draw as f32 * 1.5;
        let facing = n.facing; // the flee keeps the spawn facing (1)
        if facing > 0 && expect < 0.0 {
            expect *= 0.9;
        }
        if facing == 1 && expect < 4.0 {
            expect += 0.1;
            if expect < -4.0 {
                expect += 0.1;
            } else if expect < 0.0 {
                expect -= 0.05;
            }
            if expect > 4.0 {
                expect = 4.0;
            }
        }
        assert_eq!(n.vx, expect);
        assert_eq!(n.despawn_timer, 10, "EncourageDespawn(10) (:25158)");
        assert_eq!(n.alpha, 0);
    }

    /// A moving flee body draws NOTHING (`:24716`'s `velocity.X == 0f`
    /// gate) — the stream is untouched across the tick.
    #[test]
    fn s1_f_13b2c_the_moving_flee_body_draws_nothing() {
        let mut w = world();
        w.pumpkin_moon = false;
        let p = player(&w);
        let mut n = polt(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.vx = 2.0;
        let mut rng = UnifiedRandom::new(42);
        let witness = rng.clone();
        for _ in 0..30 {
            poltergeist_ai(&mut n, &w, &p, &mut rng);
        }
        assert_eq!(
            rng.peek(),
            witness.peek(),
            "the flee path is draw-free while moving"
        );
    }

    /// The active (pumpkin-moon) machine is draw-free over a long run —
    /// dwell, flip, dash and the approach tails never touch `Main.rand`.
    /// This pins the UpdateNPCs stream order for every mixed roster.
    #[test]
    fn s1_f_13b2c_the_active_machine_is_draw_free() {
        let w = world();
        let p = player(&w);
        let mut n = polt(60.0 * TILE_PX, 60.0 * TILE_PX);
        let mut rng = UnifiedRandom::new(123);
        let witness = rng.clone();
        for _ in 0..600 {
            poltergeist_ai(&mut n, &w, &p, &mut rng);
        }
        assert_eq!(rng.peek(), witness.peek(), "600 active ticks, zero draws");
        assert!(n.vx.abs() <= 4.0 + 0.001, "the 330 cap holds");
    }

    /// `justHit` (the `hurt_cd` stand-in) resets the dwell/dash counter
    /// (`:24682-24685`) — even mid-dash.
    #[test]
    fn s1_f_13b2c_the_just_hit_resets_the_counter() {
        let w = world();
        let p = player(&w);
        let mut n = polt(60.0 * TILE_PX, 60.0 * TILE_PX);
        n.ai = [0.0, 0.0, -150.0, 0.0];
        n.hurt_cd = 0.18;
        let mut rng = UnifiedRandom::new(7);
        poltergeist_ai(&mut n, &w, &p, &mut rng);
        // Reset to 0 → the dwell branch runs (not the dash's +0.1).
        assert_eq!(n.ai[2], 0.0);
    }

    /// `noTileCollide` (`SetDefaults :13191`): the body integrates
    /// straight through solid tiles, the collide flags stay false and
    /// `oldVelocity` carries the pre-integration velocity.
    #[test]
    fn s1_f_13b2c_the_body_flies_through_tiles() {
        let w = world();
        let p = player(&w);
        // Inside the solid ground band (rows 100..120).
        let mut n = polt(60.0 * TILE_PX, 105.0 * TILE_PX);
        n.vx = 1.0;
        let mut rng = UnifiedRandom::new(7);
        let (x0, y0) = (n.x, n.y);
        poltergeist_ai(&mut n, &w, &p, &mut rng);
        assert_eq!(n.x, x0 + n.vx);
        assert_eq!(n.y, y0 + n.vy);
        assert!(!n.collide_x && !n.collide_y);
        assert_eq!(n.old_vx, n.vx);
        assert_eq!(n.old_vy, n.vy);
    }
}
