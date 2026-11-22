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
        // `:53902-53906` — flag8 (the Mouse): cap 2, accel 1; NO hop boost.
        NpcKind::Mouse => (2.0, 1.0, false),
        // The bunnies: the default cap 1 / accel 0.07, the TownCritter boost.
        NpcKind::BunnySlimed | NpcKind::BunnyXmas => (1.0, 0.07, true),
        _ => unreachable!("the catchable critters only"),
    }
}

/// One fixed step of the critter wander/hop core. The caller's `rng` is the
/// `UpdateNPCs` stream; the idle/walk durations draw from it (module docs).
pub fn critter_ai(npc: &mut Npc, world: &World, _player: &Player, rng: &mut UnifiedRandom) {
    let (cap, accel, hop_boost) = row(npc.kind);
    let (w, h) = npc.size();
    if npc.ai[0] == 0.0 {
        // `:53695-53712` — the idle X deadband decay (±0.1 → 0).
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
        // `:53928-53950` — the walk ladder: over-cap the grounded brake
        // (×0.8 both axes), under-cap accelerate toward direction, clamped.
        if npc.vx < -cap || npc.vx > cap {
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
        // `:54226-54242` — the stuck-flip: parked 30 ticks, flip + re-walk.
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
        // `:54257-54265` — the walk countdown → idle (the C#'s ai[0]==2/11
        // rest arm; the walk's own duration ends via the idle transition).
        npc.ai[1] -= 1.0;
        if npc.ai[1] <= 0.0 {
            npc.ai[0] = 0.0;
            npc.ai[1] = 60.0 + rng.next_max(60) as f32;
            npc.net_update = true;
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
}
