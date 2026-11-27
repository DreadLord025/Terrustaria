//! W4-d38 — the Ghost's (316) aiStyle-22 machine: `NPC.AI()`'s
//! `else if (this.aiStyle == 22)` arm (`NPC.cs:24666-25234`) with the
//! type-316 branches. The Poltergeist (330) rides `npc_ai_poltergeist.rs`,
//! the Drippler (490) `npc_ai_drippler.rs`; this module carries the 316 rows:
//! - the far/dead-target gate (`:24686-24713`): the player dead or >3000 px
//!   away → the `ai[3] == 1` fade (the NPCDeath55 cry once, the Opacity
//!   −0.025/tick → deactivation) OR the retarget + EncourageDespawn(10) +
//!   the flee drift;
//! - the dwell anchor / −200 dash (`:24722-24788` — the generic rows, the
//!   ±16 px X / ±40 px Y window, the 60-tick dash flip);
//! - the floor probe (`:24789-24995` — num294 = 3, the ahead-column scan);
//! - the 0.1 gravity / 3 cap (`:25058-25071`), the flee −0.05 lift
//!   (`:25060-25062`), the −0.1 rise brake / −4 floor (`:25094-25101`);
//! - the num320 = 2 X/Y caps (`:25135`, the 0.1/0.04 accel, the 0.05/0.03
//!   brakes, `:25169-25233`).
//!
//! `SetDefaults` (`NPC.cs:12991-13006`): noGravity + noTileCollide, 24x44,
//! aiStyle 22, dmg 15, def 4, 50 HP, kb 0.5, NPCHit1/NPCDeath6, alpha 100,
//! value 90. The body integrates WITHOUT tile collision (the
//! `integrate_no_tile_collide` shape); the collideX/Y bounces
//! (`:25111-25134`) are dead under noTileCollide (the flags never set).
//!
//! RNG: the flee drift's `Main.rand.Next(-1, 2) * 1.5f` when `velocity.X ==
//! 0f` (`:24716-24720`) — the caller's `UpdateNPCs` stream, exactly once per
//! such tick (the drippler/poltergeist precedent). The dwell/dash/probe/
//! approach tails draw nothing.
//!
//! Deliberate stand-ins (the drippler module's conventions, recorded in
//! ANALYSIS): `justHit` folds onto `hurt_cd > 0`; the Opacity fade folds onto
//! the port's `alpha` (the −0.025/tick ≈ the alpha ramp; the deactivation at
//! Opacity <= 0 is the `hp = 0` convention); `npcTypeNoAggro` constant-false;
//! the target reads ride the single session player.

use crate::npc::{Npc, NpcKind};
use crate::player::Player;
use tr_core::{UnifiedRandom, World, TILE_PX};

/// `NPC.TargetClosest()` through the shared selection (the drippler twin).
fn target_closest(npc: &mut Npc, world: &World, player: &Player) {
    let cand = crate::npc_target::session_candidate_for(player, npc.kind);
    crate::npc_target::target_closest(npc, world, &[cand], true, 22, 0);
}

/// One fixed step of the Ghost (type 316) through the aiStyle-22 arm
/// (`NPC.cs:24666-25234`), ending with the noTileCollide integration.
pub fn ghost_ai(npc: &mut Npc, world: &World, player: &Player, rng: &mut UnifiedRandom) {
    debug_assert_eq!(npc.kind, NpcKind::Ghost);
    let (w, h) = npc.size();
    let (cx, cy) = npc.center();
    let (pcx, pcy) = player.center();
    let dist = ((pcx - cx) * (pcx - cx) + (pcy - cy) * (pcy - cy)).sqrt();
    // `:24682-24685` — justHit resets the dwell/dash counter.
    if npc.hurt_cd > 0.0 {
        npc.ai[2] = 0.0;
    }
    // `:24668-24681` — flag30/flag31: the Ghost has no day-flee (the 330/253/
    // 490 rows); flag31 arms only via the far/dead gate below.
    let mut flee = false;
    // `:24686-24713` — the far/dead-target gate.
    if player.dead || dist > 3000.0 {
        if npc.ai[3] == 1.0 {
            // `:24688-24702` — the fade: the NPCDeath55 cry once, the
            // Opacity −0.025/tick → deactivation.
            if npc.local_ai[3] == 0.0 {
                npc.local_ai[3] = 1.0;
                npc.wants_sound.push(crate::npc::NpcSoundRequest {
                    cue: crate::npc::NpcSoundCue::NpcDeath(55),
                    x: cx,
                    y: cy,
                });
            }
            flee = true;
            // The Opacity fade folds onto alpha (the −0.025/tick ramp);
            // Opacity <= 0 deactivates (the hp = 0 convention).
            if npc.alpha <= 6 {
                npc.hp = 0;
                npc.net_update = true;
            } else {
                npc.alpha = npc.alpha.saturating_sub(6);
            }
        } else {
            // `:24703-24712` — retarget; still far/dead → EncourageDespawn(10)
            // + the flee.
            target_closest(npc, world, player);
            let (pcx2, pcy2) = player.center();
            let dist2 = ((pcx2 - cx) * (pcx2 - cx) + (pcy2 - cy) * (pcy2 - cy)).sqrt();
            if player.dead || dist2 > 3000.0 {
                npc.encourage_despawn(10);
                flee = true;
            }
        }
    }
    if flee {
        // `:24714-24721` — the flee drift: one Next(-1, 2) draw ONLY while
        // the body is parked.
        if npc.vx == 0.0 {
            npc.vx = rng.next_range(-1, 2) as f32 * 1.5;
            npc.net_update = true;
        }
    } else if npc.ai[2] >= 0.0 {
        // `:24722-24764` — the dwell anchor (the generic rows).
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
            if npc.ai[2] >= 60.0 {
                // `:24747-24754` — the −200 dash flip.
                npc.ai[2] = -200.0;
                npc.facing *= -1;
                npc.vx *= -1.0;
                npc.collide_x = false;
                npc.net_update = true;
            }
        } else {
            // `:24756-24762` — the anchor re-latch.
            npc.ai[0] = npc.x;
            npc.ai[1] = npc.y;
            npc.ai[2] = 0.0;
            npc.net_update = true;
        }
        target_closest(npc, world, player);
    } else {
        // `:24770-24788` — the dash: the generic +1 creep, the facing away.
        npc.ai[2] += 1.0;
        npc.facing = if pcx > cx { -1 } else { 1 };
    }
    // `:24789-24995` — the floor probe: num294 = 3 (the 316 default), the
    // ahead column at the feet row, the solid/liquid scan.
    let num292 = ((npc.x + w * 0.5) / 16.0) as i32 + npc.facing * 2;
    let num293 = ((npc.y + h) / 16.0) as i32;
    let mut fall = true;
    // `:25039-25071` — flag35 is computed (kept 1:1) but the Ghost's
    // generic rise arm never reads it.
    let mut _near_floor = false;
    let num294 = 3;
    if npc.y + h > player.y {
        for num317 in num293..num293 + num294 {
            let blocked = world.tile(num292, num317).is_some_and(|t| {
                (t.active()
                    && t.flags & tr_core::tile::FLAG_INACTIVE == 0
                    && tr_core::tiledata::TILE_SOLID.has(t.kind))
                    || t.liquid > 0
            });
            if blocked {
                if num317 <= num293 + 1 {
                    _near_floor = true;
                }
                fall = false;
                break;
            }
        }
    }
    // `:25039-25072` — the gravity arm: the 316 default (+0.1, cap 3), the
    // flee −0.05 lift (`:25060-25062`).
    if fall {
        npc.vy += 0.1;
        // `:25058-25071` — the 316 flee arm caps at 6; the plain arm at 3.
        // (The two arms are exclusive in the C#; merging them into one
        // ladder would cap the fleeing body at 3.)
        if npc.kind == NpcKind::Ghost && flee {
            npc.vy -= 0.05;
            if npc.vy > 6.0 {
                npc.vy = 6.0;
            }
        } else if npc.vy > 3.0 {
            npc.vy = 3.0;
        }
    } else {
        // `:25094-25101` — the generic rise brake: −0.1 while rising
        // against the vertical facing, the −4 floor. (flag35 is the
        // 75/169 arms only; the Ghost has no floor term here.)
        if npc.direction_y < 0 && npc.vy > 0.0 {
            npc.vy -= 0.1;
        }
        if npc.vy < -4.0 {
            npc.vy = -4.0;
        }
    }
    // `:25135-25233` — num320 = 2 (the 316 default): the X approach (0.1
    // accel, the 0.1 double-add, the 0.05 brake, the ±2 clamp).
    let max_x = 2.0;
    if npc.facing == -1 && npc.vx > -max_x {
        npc.vx -= 0.1;
        if npc.vx > max_x {
            npc.vx -= 0.1;
        } else if npc.vx > 0.0 {
            npc.vx += 0.05;
        }
        if npc.vx < -max_x {
            npc.vx = -max_x;
        }
    } else if npc.facing == 1 && npc.vx < max_x {
        npc.vx += 0.1;
        if npc.vx < -max_x {
            npc.vx += 0.1;
        } else if npc.vx < 0.0 {
            npc.vx -= 0.05;
        }
        if npc.vx > max_x {
            npc.vx = max_x;
        }
    }
    // The Y approach (`:25222` — num320 = 1.5 for everything but type 490:
    // 0.04 accel, 0.05 double-add, 0.03 brake).
    let max_y = 1.5;
    if npc.direction_y == -1 && npc.vy > -max_y {
        npc.vy -= 0.04;
        if npc.vy > max_y {
            npc.vy -= 0.05;
        } else if npc.vy > 0.0 {
            npc.vy += 0.03;
        }
        if npc.vy < -max_y {
            npc.vy = -max_y;
        }
    } else if npc.direction_y == 1 && npc.vy < max_y {
        npc.vy += 0.04;
        if npc.vy < -max_y {
            npc.vy += 0.05;
        } else if npc.vy < 0.0 {
            npc.vy -= 0.03;
        }
        if npc.vy > max_y {
            npc.vy = max_y;
        }
    }
    // The noTileCollide integration (`NPC.cs:92605-92607` — the Ghost flies
    // through tiles; the collideX/Y bounces are dead under noTileCollide).
    npc.old_vx = npc.vx;
    npc.old_vy = npc.vy;
    npc.collide_x = false;
    npc.collide_y = false;
    npc.x += npc.vx;
    npc.y += npc.vy;
    let _ = TILE_PX;
}

#[cfg(test)]
mod tests {
    //! W4-d38 — tests-as-code, WRITTEN NOT RUN (code-only mode). Every
    //! constant pins to its C# literal (`NPC.cs:24666-25234`, the 316 rows).
    use super::*;
    use crate::npc::Npc;

    fn air() -> World {
        World::new("t".into(), 200, 120)
    }

    /// The far-target flee drift draws one Next(-1, 2) when parked
    /// (`NPC.cs:24716-24720`); the dwell/dash tails draw nothing.
    #[test]
    fn w4_d38_the_ghost_flee_draws_one() {
        let w = air();
        let mut player = Player::spawn(&w);
        player.x = 5000.0; // >3000 px away
        let mut n = Npc::new(NpcKind::Ghost, 100.0, 100.0, 1, 0.0);
        n.vx = 0.0;
        let mut rng = UnifiedRandom::new(3);
        let mut witness = UnifiedRandom::new(3);
        let drift = witness.next_range(-1, 2) as f32 * 1.5;
        ghost_ai(&mut n, &w, &player, &mut rng);
        assert_eq!(rng.peek(), witness.peek(), "the one flee draw");
        // `:24714-24721` drifts ±1.5; the same tick's X approach
        // (`:25135-25222`, facing 1 toward the far player) then adds
        // +0.1 and, when still negative, the 0.05 brake. The drift does
        // NOT survive verbatim as the final velocity.
        assert_eq!(n.facing, 1, "TargetClosest faces the player to the right");
        let want = if drift > 0.0 {
            drift + 0.1
        } else if drift < 0.0 {
            // The exact f32 sequence of the two C# writes.
            (drift + 0.1) - 0.05
        } else {
            0.1
        };
        assert_eq!(n.vx, want, "the ±1.5 drift then the X approach");
    }

    /// The Ghost's FindFrame (`NPC.cs:74834-74885`): the vx-sign flip +
    /// vx*0.1 tilt, the 6-tick flap over the 4-row strip.
    #[test]
    fn w4_d38_the_ghost_frame() {
        let mut n = Npc::new(NpcKind::Ghost, 100.0, 100.0, 1, 0.0);
        n.vx = 2.0;
        n.frame_counter = 0.0;
        for _ in 0..6 {
            n.find_frame();
        }
        assert_eq!(n.sprite_dir, 1, "the vx-sign flip");
        assert_eq!(n.rotation, 0.2, "the vx*0.1 tilt");
        assert_eq!(n.frame, 1, "the 6-tick flap advances one row");
    }
}
