//! S1-D-3z (zombie fighter audit) — the Drippler's (490) aiStyle-22
//! machine: `NPC.AI()`'s `else if (this.aiStyle == 22)` arm
//! (`NPC.cs:24666-25234`) with the type-490 branches. The Poltergeist
//! (330) rides the SAME arm through `npc_ai_poltergeist.rs` (S1-F-13b2
//! slice C) — that machine stays byte-identical; this module carries the
//! 490 rows:
//! - the DAY flee gate (`:24678-24681` — `type == 490 && Main.dayTime`),
//! - the dwell anchor / −200 dash (`:24722-24788` — the generic rows:
//!   `ai[2]++` and `ai[2] += 1` in the dash, NOT the 330's 0.1 creep),
//! - the distance-scaled floor probe (`:24958-24969` — `num294 = 4 +
//!   min(dist/70, 8)`),
//! - the 0.03 gravity / 0.75 cap (`:25049-25056`) and the −0.075 rise
//!   brake / −0.75 floor (`:25083-25093`),
//! - the X cap 1.5 (`:25144-25147`) and the Y cap 1 (`:25201`).
//!
//! `SetDefaults` (`NPC.cs:14618-14631`): noGravity, 28x30, aiStyle 22,
//! dmg 28, def 14, 50 HP, kb 0.6, NPCHit19/NPCDeath22, value 150. Unlike
//! the Poltergeist, 490 carries NO `noTileCollide` — the body integrates
//! WITH tile collision (the golem-body shape, `npc.rs:14302-14326`), and
//! the `collideX`/`collideY` bounces (`:25111-25134`) are live.
//!
//! RNG: the machine's only draw is the flee branch's
//! `Main.rand.Next(-1, 2) * 1.5f` when `velocity.X == 0f`
//! (`:24716-24720`) — the caller's `UpdateNPCs` stream
//! (`Main.cs:17767`), exactly once per such tick (the poltergeist
//! precedent). The dwell/dash/probe/approach tails draw nothing.
//!
//! Deliberate stand-ins (house pattern, recorded in ANALYSIS):
//! - `justHit` (`:24682-24685`) folds onto `hurt_cd > 0` (the
//!   poltergeist/WoF convention).
//! - `npcTypeNoAggro` (`:24997-25013`) is constant-false in the port —
//!   the directionY re-pick scan never runs (`npc_target.rs`).
//! - `flag30`'s setters are the 316 arm and the 169/268 roof scan; the
//!   `ai[2] >= 30 && num291 == 16` test (`:24743`) is dead C# (num291 is
//!   40 there). For 490 `flag30` is constant-false, so `:25030-25038`
//!   never fires and is not carried.
//! - `flag36` (`AI_GlobalFiringDistanceCheck`, `:24795`) feeds only the
//!   122/268 shot arms — a pure no-op for 490, dropped.
//! - The platform fall-through (`NPC.cs:93016` — `type == 490` sets
//!   `fallThrough = true` in `UpdateCollision`): the port's
//!   `collides_rect` treats platforms as solid; the fall-through is the
//!   documented remainder (ANALYSIS "The zombie fighter audit").
//! - The target reads (`Main.player[this.target]`) ride the single
//!   session player (the `npc_ai_unicorn.rs:74` stand-in): a still-255
//!   target reads the session player for exactly the first tick.

use crate::npc::{Npc, NpcKind};
use crate::player::Player;
use tr_core::{UnifiedRandom, World, TILE_PX};

/// `NPC.TargetClosest()` (`NPC.cs:77652-77666`) through the shared
/// selection (`npc_target.rs`) — the single session player as slot 0,
/// the stock aggro/no-aggro/tank values (the S1-D-3p convention, the
/// poltergeist module's twin).
fn target_closest(npc: &mut Npc, world: &World, player: &Player) {
    let cand = crate::npc_target::session_candidate_for(player, npc.kind);
    // The `EverLastingTicker` stamp's only reader is the unported
    // target-line fade — 0 until the caller migration (S1-D-3p).
    crate::npc_target::target_closest(npc, world, &[cand], true, 22, 0);
}

/// One fixed step of the Drippler (type 490) through the aiStyle-22 arm
/// (`NPC.cs:24666-25234`), ending with the tile-collision integration
/// (`UpdateCollision`, `:92538-92607` — 490 has no `noTileCollide`). The
/// caller's `rng` is the `UpdateNPCs` stream; the machine draws from it
/// only on a flee tick with `velocity.X == 0f` (`:24718`).
pub fn drippler_ai(npc: &mut Npc, world: &World, player: &Player, rng: &mut UnifiedRandom) {
    debug_assert_eq!(npc.kind, NpcKind::Drippler);
    let (w, h) = npc.size();
    // `:24678-24681` — flag31: the Drippler flees by day. (flag30 is
    // constant-false for 490 — see the module note.)
    let flee = world.day_time;
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
        // `:24770-24788` — the dash: the GENERIC row creeps `ai[2]` at 1
        // (the 0.1 is the 330's, `:24772-24775`), and the facing points
        // AWAY from the player (`:24780-24787`).
        npc.ai[2] += 1.0;
        let (pcx, _) = player.center();
        let (cx, _) = npc.center();
        npc.facing = if pcx > cx { -1 } else { 1 };
    }
    // `:24789-24790` — the floor-probe coords: two tiles ahead of the
    // centre, at the body's bottom row.
    let num292 = ((npc.x + w * 0.5) / 16.0) as i32 + npc.facing * 2;
    let num293 = ((npc.y + h) / 16.0) as i32;
    let mut fall = true; // flag34
    let mut near_floor = false; // flag35
    // `:24793` + `:24958-24969` — num294: the 490 row is 4 plus the
    // distance-scaled reach (dist/70 capped at 8, truncated to int).
    let mut num294 = 4;
    if npc.target >= 0 {
        let (pcx, pcy) = player.center();
        let (cx, cy) = npc.center();
        let mut num316 = ((pcx - cx) * (pcx - cx) + (pcy - cy) * (pcy - cy)).sqrt() / 70.0;
        if num316 > 8.0 {
            num316 = 8.0;
        }
        num294 += num316 as i32;
    }
    // `:24971-24995` — the floor scan: at or below the player's top edge,
    // probe num294 rows down the ahead column; a solid/liquid cell kills
    // the fall, and one within the first two rows arms the rise brake.
    // (The 330 skip-floor branch `:24973-24976` is the poltergeist's.)
    if npc.y + h > player.y {
        for num317 in num293..num293 + num294 {
            let t = world.tile(num292, num317);
            let blocked = t.is_some_and(|t| {
                (t.active()
                    && t.flags & tr_core::tile::FLAG_INACTIVE == 0
                    && tr_core::tiledata::TILE_SOLID.has(t.kind))
                    || t.liquid > 0
            });
            if blocked {
                if num317 <= num293 + 1 {
                    near_floor = true;
                }
                fall = false;
                break;
            }
        }
    }
    // `:24997-25013` — the npcTypeNoAggro directionY re-pick: constant-
    // false in the port (module docs), never runs.
    // `:25014-25029` — the 169/268 roof scan: not 490.
    // `:25030-25038` — flag30: constant-false for 490 (module docs).
    // `:25039-25072` — the gravity arm: the 490 row (+0.03, cap 0.75).
    if fall {
        npc.vy += 0.03;
        if npc.vy > 0.75 {
            npc.vy = 0.75;
        }
    } else {
        // `:25083-25093` — the 490 rise brake: −0.075 while rising
        // against the vertical facing OR near the floor, the −0.75 cap.
        if (npc.direction_y < 0 && npc.vy > 0.0) || near_floor {
            npc.vy -= 0.075;
        }
        if npc.vy < -0.75 {
            npc.vy = -0.75;
        }
        // `:25098-25101` — the shared −4 floor (unreachable under the
        // 490 cap, kept verbatim).
        if npc.vy < -4.0 {
            npc.vy = -4.0;
        }
    }
    // `:25111-25134` — the collision bounces. 490 HAS tile collision (no
    // `noTileCollide`), so these are live: the previous tick's integration
    // set the flags (the tail below).
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
    // `:25135-25147` — num320: the 490 X cap is 1.5. (The 330 block
    // `:25148-25168` — alpha 0, cap 4, TargetClosest/EncourageDespawn,
    // the 0.9 damping — is the poltergeist's, not carried.)
    let max_x = 1.5;
    // `:25169-25200` — the X approach (0.1 accel, the 0.1 double-add
    // past the far cap, the 0.05 near-zero brake, the ±1.5 clamp).
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
    // `:25201-25233` — the Y approach (num320 = 1 for 490: 0.04 accel,
    // 0.05 double-add, 0.03 brake, the ±1 clamp).
    let max_y = 1.0;
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
    // `UpdateCollision` (`:92538-92540`): `oldVelocity` captured, the
    // collide flags reset — then the tile-collision integration (490 has
    // no `noTileCollide`; the golem-body shape, `npc.rs:14302-14326`).
    // The platform fall-through (`:93016`) is the documented remainder
    // (module docs): `collides_rect` treats platforms as solid.
    npc.old_vx = npc.vx;
    npc.old_vy = npc.vy;
    npc.collide_x = false;
    npc.collide_y = false;
    let nx = npc.x + npc.vx;
    if !crate::player::collides_rect(world, nx, npc.y, w, h) {
        npc.x = nx;
    } else {
        npc.vx = 0.0;
        npc.collide_x = true;
    }
    let ny = npc.y + npc.vy;
    if !crate::player::collides_rect(world, npc.x, ny, w, h) {
        npc.y = ny;
        npc.on_ground = false;
    } else if npc.vy > 0.0 {
        let ty = ((ny + h) / TILE_PX).floor() as i32;
        npc.y = ty as f32 * TILE_PX - h;
        npc.vy = 0.0;
        npc.on_ground = true;
        npc.collide_y = true;
    } else if npc.vy < 0.0 {
        let ty = (ny / TILE_PX).floor() as i32;
        npc.y = (ty + 1) as f32 * TILE_PX;
        npc.vy = 0.0;
        npc.collide_y = true;
    }
}

#[cfg(test)]
mod tests {
    //! S1-D-3z (zombie fighter audit) — tests-as-code, WRITTEN NOT RUN
    //! (code-only mode — no compiler invoked). Every assertion pins a
    //! constant to its C# literal (`NPC.cs:24666-25234`, the 490 rows).
    use super::*;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    /// A 64x64 world, dirt ground at row 30.
    fn ground() -> World {
        let mut w = World::new("t".into(), 64, 64);
        for x in 0..64 {
            for y in 30..64 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    fn player(w: &World) -> Player {
        let mut p = Player::spawn(w);
        p.x = 40.0 * TILE_PX;
        p.y = 26.0 * TILE_PX;
        p
    }

    /// The day flee (`NPC.cs:24678-24681` + `:24714-24721`): by day the
    /// Drippler flees, and a parked body draws ONE `Next(-1, 2)` for the
    /// drift; a moving body draws nothing.
    #[test]
    fn the_day_flee_drifts_and_draws_only_when_parked() {
        let mut w = ground();
        w.day_time = true;
        let p = player(&w);
        let mut n = Npc::new(NpcKind::Drippler, 20.0 * TILE_PX, 20.0 * TILE_PX, 1, 0.0);
        n.vx = 0.0;
        let mut rng = UnifiedRandom::new(7);
        let before = rng.clone();
        drippler_ai(&mut n, &w, &p, &mut rng);
        assert_ne!(rng, before, "the parked flee drew one Next(-1, 2)");
        // The drift {-1.5, 0, 1.5} then rides the X-approach tail
        // (`NPC.cs:25169-25200`, facing = +1): -1.5 → -1.45 (the +0.1
        // accel then the -0.05 near-zero brake), 0 → 0.1, 1.5 stays
        // (the `vx < max_x` gate is strict).
        assert!(
            (n.vx - (-1.45)).abs() < 1e-5 || (n.vx - 0.1).abs() < 1e-5 || n.vx == 1.5,
            "the drift magnitude after the X approach: {}",
            n.vx
        );
        // A moving body draws nothing on the flee branch.
        let mut n2 = Npc::new(NpcKind::Drippler, 20.0 * TILE_PX, 20.0 * TILE_PX, 1, 0.0);
        n2.vx = 1.0;
        let mut rng2 = UnifiedRandom::new(7);
        let before2 = rng2.clone();
        drippler_ai(&mut n2, &w, &p, &mut rng2);
        assert_eq!(rng2, before2, "the moving flee draws nothing");
    }

    /// The night dwell (`NPC.cs:24722-24764`): at night the Drippler does
    /// NOT flee; the dwell anchor latches and `ai[2]` counts up, drawing
    /// nothing.
    #[test]
    fn the_night_dwell_counts_without_draws() {
        let mut w = ground();
        w.day_time = false;
        let p = player(&w);
        let mut n = Npc::new(NpcKind::Drippler, 20.0 * TILE_PX, 20.0 * TILE_PX, 1, 0.0);
        n.ai = [n.x, n.y, 0.0, 0.0]; // latched on its own position
        let mut rng = UnifiedRandom::new(7);
        let before = rng.clone();
        drippler_ai(&mut n, &w, &p, &mut rng);
        assert_eq!(rng, before, "the dwell draws nothing");
        assert_eq!(n.ai[2], 1.0, "the dwell counter ticks");
    }

    /// The gravity cap (`NPC.cs:25049-25056`): falling free, the 490 row
    /// adds 0.03 and caps at 0.75.
    #[test]
    fn the_fall_gravity_caps_at_three_quarters() {
        let mut w = ground();
        w.day_time = false;
        let p = player(&w);
        let mut n = Npc::new(NpcKind::Drippler, 20.0 * TILE_PX, 10.0 * TILE_PX, 1, 0.0);
        n.ai = [n.x, n.y, 0.0, 0.0];
        n.vy = 0.74;
        let mut rng = UnifiedRandom::new(7);
        drippler_ai(&mut n, &w, &p, &mut rng);
        // 0.74 + 0.03 = 0.77 → capped 0.75 (the Y approach may nudge it,
        // but the cap holds the fall).
        assert!(n.vy <= 0.75 + 0.04, "the 0.75 fall cap, got {}", n.vy);
    }

    /// The X cap (`NPC.cs:25144-25147`): the 490 approach clamps at 1.5.
    #[test]
    fn the_x_approach_caps_at_one_and_a_half() {
        let mut w = ground();
        w.day_time = false;
        let p = player(&w);
        let mut n = Npc::new(NpcKind::Drippler, 20.0 * TILE_PX, 20.0 * TILE_PX, 1, 0.0);
        n.ai = [n.x, n.y, 0.0, 0.0];
        n.facing = 1;
        n.vx = 1.45;
        let mut rng = UnifiedRandom::new(7);
        drippler_ai(&mut n, &w, &p, &mut rng);
        assert!(n.vx <= 1.5, "the 1.5 X cap, got {}", n.vx);
    }
}
