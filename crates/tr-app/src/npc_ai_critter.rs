//! W4-d38 — the catchable critters' `AI_007_TownEntities` walk subset
//! (`NPC.cs:52965-55563`) for the Squirrel (299), the Mouse (300), the
//! BunnySlimed (303) and the BunnyXmas (337). All four are
//! `NPCID.Sets.TownCritter` (`NPCID.cs:10459`); the Squirrel is also
//! `CritterThatCanTurnOnPlayers` (`:10436`). The full town machine (housing,
//! doors, talking, sitting, the danger scan) is the town NPCs' system; the
//! critters ride its wander/hop core, ported here as the documented subset
//! (the `update_guide` walk-subset precedent, extended with the C#'s actual
//! idle/walk state machine + the TownCritter hop boost).
//!
//! The state axis (`ai[0]`): 0 idle (the X deadband decay, the `ai[1]`
//! countdown → the walk start), 1 walk (the per-type cap/accel ladder, the
//! ledge hop, the stuck-flip). The C#'s `ai[1]` idle/walk durations draw
//! `Main.rand` — the caller's `UpdateNPCs` stream (`Main.cs:17767`):
//! * idle → walk (`:53741-53760`): `Next(300)` for the 200+ walk duration,
//!   then (TownCritter) `Next(200, 400)` added; the avoid-falling branch
//!   instead draws `Next(120)` for the 60+ flip idle.
//! * the stuck-flip (`:54226-54242`): `Next(300)` + (TownCritter)
//!   `Next(200, 400)`, the direction + velocity flip.
//! * walk → idle (`:54289-54296`): `Next(60)` for the 60+ idle, `Next(60)`
//!   for the 30+ localAI[3].
//!
//! The per-type walk rows (`:53865-53906`): the Squirrel cap 1.5 / accel
//! 0.07 (`:53872-53875`); the Mouse (flag8, `:53451`) cap 2 / accel 1
//! (`:53902-53906`); the bunnies the default cap 1 / accel 0.07. The ledge
//! hop is `velocity.Y = -6f` (`:54091`), boosted ×1.2 while rising for the
//! TownCritter (`:54191-54194`, the Mouse excluded by `!flag8`).
//!
//! Deliberate stand-ins (recorded in ANALYSIS): the housing/teleport-home,
//! door-bash, talk and sit states are the town system (not the critters'
//! wander); the `GetWalkPrediction` ledge probe folds to the port's
//! ahead-column tile scan (the `update_guide` convention); the danger scan
//! (`flag16`/`flag17`, the hostile-NPC proximity) is constants-off (the
//! critters wander undisturbed until the threat system lands); `wet` reads
//! the carried Entity flag.

use crate::npc::{Npc, NpcKind, GRAVITY, MAX_FALL};
use crate::player::Player;
use tr_core::{UnifiedRandom, World, TILE_PX};

/// The per-type walk row: (cap `num21`, accel `num22`, the TownCritter hop
/// boost applies). Citations are the C# per-type writes (`:53865-53906`).
fn row(kind: NpcKind) -> (f32, f32, bool) {
    match kind {
        // `:53872-53875` — the Squirrel cap 1.5, accel 0.07 (the default).
        NpcKind::Squirrel => (1.5, 0.07, true),
        // `:53902-53906` - flag8 (the Mouse): cap 2, accel 1; NO hop boost.
        // W4-c9 merge: the GoldMouse 447 joins flag8 (`:53451`).
        NpcKind::Mouse | NpcKind::GoldMouse => (2.0, 1.0, false),
        // The bunnies: the default cap 1 / accel 0.07, the TownCritter boost.
        // W4-c9 merge: the GoldBunny 443 + the GoldFrog 445 (flag10's land
        // row is the default pair, `:53865-53866`). The GoldBunny row stays
        // spelled out (the c9 canonical arm) — same mechanics as the sibling
        // bunnies, no second implementation.
        NpcKind::GoldBunny => (1.0, 0.07, true),
        NpcKind::BunnySlimed | NpcKind::BunnyXmas | NpcKind::GoldFrog => (1.0, 0.07, true),
        _ => unreachable!("the catchable critters only"),
    }
}

/// One fixed step of the critter wander/hop core. The caller's `rng` is the
/// `UpdateNPCs` stream; the idle/walk durations draw from it (module docs).
pub fn critter_ai(npc: &mut Npc, world: &World, _player: &Player, rng: &mut UnifiedRandom) {
    let (cap, accel, hop_boost) = row(npc.kind);
    let (w, h) = npc.size();
    if npc.ai[0] == 0.0 {
        // `:53632-53635` - the idle localAI[3] countdown (W4-c9 merge).
        if npc.local_ai[3] > 0.0 {
            npc.local_ai[3] -= 1.0;
        }
        // `:53697-53700` - flag8 (the GoldMouse) halves the idle X first
        // (W4-c9 merge).
        if npc.kind == NpcKind::GoldMouse {
            npc.vx *= 0.5;
        }
        // `:53695-53712` - the idle X deadband decay (±0.1 → 0).
        if npc.vx > 0.1 {
            npc.vx -= 0.1;
        } else if npc.vx < -0.1 {
            npc.vx += 0.1;
        } else {
            npc.vx = 0.0;
        }
        // `:53719-53722` — the idle countdown.
        if npc.ai[1] > 0.0 {
            npc.ai[1] -= 1.0;
        }
        // `:53741-53760` — idle → walk (the ledge probe folds to the ahead
        // scan; the critters have no home, so avoidFalling reads the ledge).
        if npc.ai[1] <= 0.0 {
            if !ledge_ahead(world, npc, w, h) {
                // `:53745-53753` — the walk start: 200 + Next(300), then
                // (TownCritter) + Next(200, 400).
                npc.ai[0] = 1.0;
                npc.ai[1] = 200.0 + rng.next_max(300) as f32;
                npc.ai[1] += rng.next_range(200, 400) as f32;
                npc.ai[2] = 0.0;
                npc.local_ai[3] = 0.0;
                npc.net_update = true;
            } else {
                // `:53757-53759` — the flip idle: 60 + Next(120).
                npc.facing *= -1;
                npc.ai[1] = 60.0 + rng.next_max(120) as f32;
                npc.net_update = true;
            }
        }
    } else if npc.ai[0] == 1.0 {
        // `:53815` - the walk countdown (W4-c9 merge: the C#'s own order -
        // the countdown + the exit run BEFORE the ladder/hop/stuck-flip;
        // the drowning/homing skips are the stand-ins, the homeless
        // critter always decrements).
        npc.ai[1] -= 1.0;
        // `:53817-53831` - walk -> idle: 300 + Next(300), then
        // (TownCritter) - Next(100); localAI[3] = 60 (the c9 faithful
        // draw order - the :53824 subtraction; the d38 module's :54289
        // 60+Next(60) was the sit/talk-state exit, a DIFFERENT AI_007 arm).
        if npc.ai[1] <= 0.0 {
            npc.ai[0] = 0.0;
            npc.ai[1] = 300.0 + rng.next_max(300) as f32;
            npc.ai[1] -= rng.next_max(100) as f32;
            npc.ai[2] = 0.0;
            npc.local_ai[3] = 60.0;
            npc.net_update = true;
        }
        // W4-c10 — `:53917-53927`: flag10 (the frogs 361/445/687) wet REPLACES
        // the walk ladder: still (|vx| < 0.05 && |vy| < 0.05) kicks
        // `vx += cap * 10 * direction`, else brakes ×0.9.
        if npc.kind == NpcKind::GoldFrog && npc.wet {
            if npc.vx.abs() < 0.05 && npc.vy.abs() < 0.05 {
                npc.vx += cap * 10.0 * npc.facing as f32;
            } else {
                npc.vx *= 0.9;
            }
        }
        // `:53928-53950` — the walk ladder: over-cap the grounded brake
        // (×0.8 both axes), under-cap accelerate toward direction, clamped.
        else if npc.vx < -cap || npc.vx > cap {
            if npc.vy == 0.0 {
                npc.vx *= 0.8;
                npc.vy *= 0.8;
            }
        } else if npc.vx < cap && npc.facing == 1 {
            npc.vx += accel;
            if npc.vx > cap {
                npc.vx = cap;
            }
        } else if npc.vx > -cap && npc.facing == -1 {
            npc.vx -= accel;
            if npc.vx < -cap {
                npc.vx = -cap;
            }
        }
        npc.sprite_dir = npc.facing;
        // `:54083-54093` — the ledge hop: moving into a solid ahead cell
        // (with headroom) hops -6f. The TownCritter boost ×1.2 while rising
        // (`:54191-54194`, the Mouse excluded).
        if npc.vy == 0.0 && npc.vx != 0.0 && wall_ahead(world, npc, w, h) {
            npc.vy = -6.0;
            npc.net_update = true;
        }
        if npc.vy < 0.0 && hop_boost {
            npc.vy *= 1.2;
        }
        // `:54226-54242` - the stuck-flip: parked 30 ticks, flip + re-walk.
        if npc.vx == 0.0 && npc.vy == 0.0 {
            npc.local_ai[2] += 1.0;
            if npc.local_ai[2] >= 30.0 {
                npc.local_ai[2] = 0.0;
                npc.ai[0] = 1.0;
                npc.ai[1] = 200.0 + rng.next_max(300) as f32;
                npc.ai[1] += rng.next_range(200, 400) as f32;
                npc.ai[2] = 0.0;
                npc.facing *= -1;
                npc.vx *= -1.0;
                npc.net_update = true;
            }
        } else {
            npc.local_ai[2] = 0.0;
        }
        // W4-c9 merge: the old :54257 tail countdown dropped - the
        // faithful :53815-53831 countdown + exit moved to the block head
        // above (the C#'s own order).
    }
    // W4-c10 — `:55158-55197`: flag13 (the turtles + the frogs) wet tail —
    // the GoldFrog 445 is the port's only member. The ai[1] = 50 pin, the
    // wall/world-edge bounce (×−0.25), the waterline hover (−0.8 with the
    // −4 cap + the line-crossing clamp; above the line the vy min-clamp)
    // and the no-water −0.2 drift.
    if npc.kind == NpcKind::GoldFrog && npc.wet {
        npc.ai[1] = 50.0; // `:55160-55163` — flag10's pin.
        let (cx, cy) = npc.center();
        let col = (cx / 16.0) as i32;
        // `:55166-55172` — collideX or the 5-tile world margins.
        if npc.collide_x
            || (col < 5 && npc.facing == -1)
            || (col > world.width - 5 && npc.facing == 1)
        {
            npc.facing *= -1;
            npc.vx *= -0.25;
            npc.net_update = true;
        }
        // `:55173-55196` — GetWaterLine at the truncated centre tile
        // (`Center.ToTileCoordinates()`, the >>4 floor — the port's
        // positive-coordinate world folds it onto the /16 truncation).
        if let Some(line) = crate::collision::get_water_line(world, col, (cy as i32) >> 4) {
            let num90 = cy + 1.0;
            if cy > line {
                npc.vy -= 0.8;
                if npc.vy < -4.0 {
                    npc.vy = -4.0;
                }
                if num90 + npc.vy < line {
                    npc.vy = line - num90;
                }
            } else {
                npc.vy = npc.vy.min(line - num90);
            }
        } else {
            npc.vy -= 0.2;
        }
    }
    // The generic gravity + tile-collision integration (`NPC.UpdateNPC`'s
    // tail, `:90465-90493` — the critters have no noTileCollide/noGravity).
    npc.vy = (npc.vy + GRAVITY).min(MAX_FALL);
    let nx = npc.x + npc.vx;
    if !crate::player::collides_rect(world, nx, npc.y, w, h) {
        npc.x = nx;
    } else {
        npc.vx = 0.0;
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
    } else {
        npc.vy = 0.0;
    }
}

/// The ledge probe (the `GetWalkPrediction` fold): the ahead column at the
/// feet row has no floor within two tiles → the critter avoids walking off.
fn ledge_ahead(world: &World, npc: &Npc, w: f32, h: f32) -> bool {
    let col = ((npc.x + w * 0.5 + 15.0 * npc.facing as f32) / 16.0) as i32;
    let row = ((npc.y + h - 16.0) / 16.0) as i32;
    for dy in 1..=2 {
        if world
            .tile(col, row + dy)
            .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind))
        {
            return false;
        }
    }
    true
}

/// The wall probe (the `:54087` hop gate): a solid cell ahead at the body
/// row with headroom above → the hop.
fn wall_ahead(world: &World, npc: &Npc, w: f32, h: f32) -> bool {
    let col = ((npc.x + w * 0.5 + 15.0 * npc.facing as f32) / 16.0) as i32;
    let row = ((npc.y + h - 16.0) / 16.0) as i32;
    let solid = |c: i32, r: i32| {
        world
            .tile(c, r)
            .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind))
    };
    solid(col, row) && !solid(col, row - 2)
}

#[cfg(test)]
mod tests {
    //! W4-d38 — tests-as-code, WRITTEN NOT RUN (code-only mode). Every
    //! constant pins to its C# literal (`NPC.cs:53865-54265`).
    use super::*;
    use crate::npc::Npc;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    fn ground() -> World {
        let mut w = World::new("t".into(), 200, 80);
        for x in 0..200 {
            for y in 40..80 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    /// The per-type walk rows (cap, accel, hop boost).
    #[test]
    fn w4_d38_the_critter_walk_rows() {
        assert_eq!(row(NpcKind::Squirrel), (1.5, 0.07, true));
        assert_eq!(row(NpcKind::Mouse), (2.0, 1.0, false));
        assert_eq!(row(NpcKind::BunnySlimed), (1.0, 0.07, true));
        assert_eq!(row(NpcKind::BunnyXmas), (1.0, 0.07, true));
    }

    /// W4-c10 — the gold critters' rows: the GoldBunny/GoldFrog the default
    /// pair + boost (`NPC.cs:53865-53866`, TownCritter `NPCID.cs:10459`),
    /// the GoldMouse flag8's cap 2 / accel 1 / no boost (`:53451`,
    /// `:53902-53906`, `:54191`). WRITTEN NOT RUN (code-only).
    #[test]
    fn w4_c10_the_gold_critter_walk_rows() {
        assert_eq!(row(NpcKind::GoldBunny), (1.0, 0.07, true));
        assert_eq!(row(NpcKind::GoldFrog), (1.0, 0.07, true));
        assert_eq!(row(NpcKind::GoldMouse), (2.0, 1.0, false));
    }

    /// W4-c10 — the GoldFrog's wet swim (`NPC.cs:53917-53927` +
    /// `:55158-55197`): still kicks vx += cap*10*direction, moving brakes
    /// ×0.9; the tail pins ai[1] = 50, bounces off collideX (×−0.25 + the
    /// facing flip) and drifts −0.2 with no waterline. The walk-duration
    /// draws stay untouched on the swim tick (the countdown decrements).
    /// WRITTEN NOT RUN (code-only).
    #[test]
    fn w4_c10_the_gold_frog_swims_when_wet() {
        let w = ground();
        let player = Player::spawn(&w);
        let mut n = Npc::new(NpcKind::GoldFrog, 100.0 * TILE_PX, 30.0 * TILE_PX, 1, 0.0);
        n.ai[0] = 1.0;
        n.ai[1] = 500.0;
        n.wet = true;
        n.facing = 1;
        n.vx = 0.0;
        n.vy = 0.0;
        let mut rng = UnifiedRandom::new(3);
        let witness = rng.clone();
        critter_ai(&mut n, &w, &player, &mut rng);
        // The still kick: 1.0 * 10 * 1 = 10 (`:53920`), no wet-tail draws.
        assert!(n.vx > 0.0, "the still kick fired: {}", n.vx);
        assert_eq!(n.ai[1], 50.0, "the flag10 pin (:55162)");
        assert_eq!(rng, witness, "the swim tick draws nothing");
        // Moving: the ×0.9 brake (`:53925`).
        let vx0 = n.vx;
        n.vy = 0.0;
        critter_ai(&mut n, &w, &player, &mut rng);
        assert!((n.vx - vx0 * 0.9).abs() < 1e-4, "the wet brake");
        // The wall bounce (`:55166-55172`): collideX flips + ×−0.25.
        n.collide_x = true;
        let (f0, v0) = (n.facing, n.vx);
        critter_ai(&mut n, &w, &player, &mut rng);
        assert_eq!(n.facing, -f0, "the bounce flips");
        assert!((n.vx - v0 * 0.9 * -0.25).abs() < 1e-4, "brake then bounce");
        assert!(n.net_update);
    }

    /// The idle → walk start draws Next(300) then Next(200, 400) (the
    /// TownCritter duration), setting ai[0] = 1 (`NPC.cs:53745-53753`).
    #[test]
    fn w4_d38_the_critter_walk_start_draws_two() {
        let w = ground();
        let player = Player::spawn(&w);
        let mut n = Npc::new(NpcKind::BunnySlimed, 100.0 * TILE_PX, 40.0 * TILE_PX - 20.0, 1, 0.0);
        n.on_ground = true;
        n.ai[0] = 0.0;
        n.ai[1] = 0.0;
        let mut rng = UnifiedRandom::new(5);
        let mut witness = UnifiedRandom::new(5);
        let d1 = witness.next_max(300);
        let d2 = witness.next_range(200, 400);
        critter_ai(&mut n, &w, &player, &mut rng);
        assert_eq!(rng.peek(), witness.peek(), "the two duration draws");
        assert_eq!(n.ai[0], 1.0, "the walk starts");
        assert_eq!(n.ai[1], 200.0 + d1 as f32 + d2 as f32, "the walk duration");
    }

    /// W4-c9 merge (349 s6b keeper): the walk -> idle exit draws
    /// 300 + Next(300) then (TownCritter) - Next(100), localAI[3] = 60
    /// (`NPC.cs:53817-53831` - the :53824 subtraction; the d38 module's
    /// old :54289 60+Next(60) was the sit/talk-state exit, a DIFFERENT
    /// AI_007 arm). The GoldBunny rides the merged module. WRITTEN, NOT
    /// RUN (code-only mode).
    #[test]
    fn w4_c9_the_gold_critter_walk_to_idle_subtracts() {
        let w = ground();
        let player = Player::spawn(&w);
        let mut n = Npc::new(NpcKind::GoldBunny, 100.0 * TILE_PX, 40.0 * TILE_PX - 20.0, 1, 0.0);
        n.on_ground = true;
        n.ai[0] = 1.0;
        n.ai[1] = 1.0; // decrements to 0 -> the idle transition
        let mut rng = UnifiedRandom::new(9);
        let mut witness = UnifiedRandom::new(9);
        let d1 = witness.next_max(300);
        let d2 = witness.next_max(100);
        critter_ai(&mut n, &w, &player, &mut rng);
        assert_eq!(n.ai[0], 0.0, "the idle starts");
        assert_eq!(n.ai[1], 300.0 + d1 as f32 - d2 as f32, "the TownCritter idle");
        assert_eq!(n.local_ai[3], 60.0);
    }
}
