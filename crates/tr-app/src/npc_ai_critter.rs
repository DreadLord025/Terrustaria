//! W4-c9 — the gold town-critters' `AI_007_TownEntities` walk subset
//! (`NPC.cs:52965-55563`) for the GoldBunny (443), the GoldFrog (445) and
//! the GoldMouse (447). All three are `NPCID.Sets.TownCritter`
//! (`NPCID.cs:10459`); the GoldMouse is also the `flag8` mouse row
//! (`:53451`) and the GoldFrog the `flag10` frog row (`:53453`). The full
//! town machine (housing, doors, talking, sitting, the danger scan) is the
//! town NPCs' system; the critters ride its wander/hop core, ported here
//! as the documented subset (the W4-d38 `npc_ai_critter` precedent,
//! re-ported on this branch with the C#'s actual idle/walk durations).
//!
//! The state axis (`ai[0]`): 0 idle (the X deadband decay, the `ai[1]`
//! countdown → the walk start), 1 walk (the per-type cap/accel ladder, the
//! ledge hop, the stuck-flip). The C#'s durations draw `Main.rand` — the
//! caller's `UpdateNPCs` stream (`Main.cs:17767`):
//! * idle → walk (`:53741-53753`): `Next(300)` for the 200+ walk duration,
//!   then (TownCritter) `Next(200, 400)` added; the ledge-flip branch
//!   instead draws `Next(120)` for the 60+ flip idle (`:53757-53759`).
//! * walk → idle (`:53817-53831`): `Next(300)` for the 300+ idle, then
//!   (TownCritter) `Next(100)` SUBTRACTED; `localAI[3] = 60`.
//! * the idle wander-flip (`:53782-53787`): `Next(80) == 0` while
//!   `localAI[3] == 0` → flip + `localAI[3] = 200`.
//! * the stuck-flip (`:54226-54242`): `Next(300)` + (TownCritter)
//!   `Next(200, 400)`, the direction + velocity flip.
//!
//! The per-type walk rows (`:53865-53906`): the GoldBunny the default cap
//! 1 / accel 0.07; the GoldMouse (flag8, `:53902-53906`) cap 2 / accel 1;
//! the GoldFrog (flag10) the default cap 1 / accel 0.07 DRY — the wet
//! override (`:53917-53927`) is the documented stand-in below. The ledge
//! hop is `velocity.Y = -6f` (`:54091`), boosted ×1.2 while rising for the
//! TownCritter (`:54191-54194`, the Mouse excluded by `!flag8`).
//!
//! Deliberate stand-ins (recorded in ANALYSIS): the housing/teleport-home,
//! door-bash, talk and sit states are the town system (not the critters'
//! wander); the `GetWalkPrediction` ledge probe (`:55673-55743`) folds to
//! the port's ahead-column tile scan (TownCritter clears `avoidFalling`
//! at `:55700-55703`, so the idle walk-start never ledge-flips — the
//! probe only shapes the walk's ledge-stop, folded to the hop gate); the
//! danger scan (`flag16`/`flag17`, the hostile-NPC proximity) is
//! constants-off (the critters wander undisturbed until the threat system
//! lands); the frog's wet swim override and the drowning escape
//! (`:53725-53736`/`:53975-53992`) fold to the dry walk (the port's
//! `wet` flag feeds the FindFrame swim rows only); `StepUp`/`StepDown`
//! (`:53956-53966`) fold to the tile-collision tail.

use crate::npc::{Npc, NpcKind};
use crate::player::Player;
use tr_core::{UnifiedRandom, World, TILE_PX};

/// The per-type walk row: (cap `num21`, accel `num22`, the TownCritter hop
/// boost applies). Citations are the C# per-type writes (`:53865-53906`).
fn row(kind: NpcKind) -> (f32, f32, bool) {
    match kind {
        // The GoldBunny: the default cap 1 / accel 0.07 (`:53865-53866`),
        // the TownCritter boost (`:54191`).
        NpcKind::GoldBunny => (1.0, 0.07, true),
        // The GoldFrog (flag10): the default cap 1 / accel 0.07 dry
        // (`:53865-53866` — the wet override `:53917` is the stand-in),
        // the TownCritter boost.
        NpcKind::GoldFrog => (1.0, 0.07, true),
        // `:53902-53906` — flag8 (the GoldMouse): cap 2, accel 1; NO hop
        // boost (`:54191`'s `!flag8`).
        NpcKind::GoldMouse => (2.0, 1.0, false),
        _ => unreachable!("the gold town-critters only"),
    }
}

/// One fixed step of the critter wander/hop core. The caller's `rng` is the
/// `UpdateNPCs` stream; the idle/walk durations draw from it (module docs).
pub fn critter_ai(npc: &mut Npc, world: &World, _player: &Player, rng: &mut UnifiedRandom) {
    let (cap, accel, hop_boost) = row(npc.kind);
    let (w, h) = npc.size();
    if npc.ai[0] == 0.0 {
        // `:53632-53635` — the idle localAI[3] countdown.
        if npc.local_ai[3] > 0.0 {
            npc.local_ai[3] -= 1.0;
        }
        // `:53697-53700` — flag8 (the GoldMouse) halves the idle X first.
        if npc.kind == NpcKind::GoldMouse {
            npc.vx *= 0.5;
        }
        // `:53701-53712` — the idle X deadband decay (±0.1 → 0).
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
        // `:53741-53760` — idle → walk. TownCritter clears avoidFalling
        // (`:55700-55703`), so the walk-start always takes the first
        // branch (the ledge-flip is the non-TownCritter town NPC's).
        if npc.ai[1] <= 0.0 {
            // `:53745-53753` — the walk start: 200 + Next(300), then
            // (TownCritter) + Next(200, 400).
            npc.ai[0] = 1.0;
            npc.ai[1] = 200.0 + rng.next_max(300) as f32;
            npc.ai[1] += rng.next_range(200, 400) as f32;
            npc.ai[2] = 0.0;
            npc.local_ai[3] = 0.0;
            npc.net_update = true;
        }
        // `:53782-53787` — the idle wander-flip: Next(80) == 0 while
        // localAI[3] == 0 → flip + localAI[3] = 200. (The home-distance
        // branch `:53766-53781` is the housed town NPC's — the homeless
        // critter takes this else.)
        if rng.next_max(80) == 0 && npc.local_ai[3] == 0.0 {
            npc.local_ai[3] = 200.0;
            npc.facing *= -1;
            npc.net_update = true;
        }
    } else if npc.ai[0] == 1.0 {
        // `:53815` — the walk countdown (the drowning/homing skips are the
        // stand-ins; the homeless critter always decrements).
        npc.ai[1] -= 1.0;
        // `:53817-53831` — walk → idle: 300 + Next(300), then
        // (TownCritter) − Next(100); localAI[3] = 60.
        if npc.ai[1] <= 0.0 {
            npc.ai[0] = 0.0;
            npc.ai[1] = 300.0 + rng.next_max(300) as f32;
            npc.ai[1] -= rng.next_max(100) as f32;
            npc.ai[2] = 0.0;
            npc.local_ai[3] = 60.0;
            npc.net_update = true;
        }
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
            // `:53946-53949` — the C#'s own `> num21` clamp on the
            // negative-accel branch (dead on the facing side; kept
            // verbatim, the style24 steering precedent).
            if npc.vx > cap {
                npc.vx = cap;
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
    }
    // The generic gravity + tile-collision integration (`NPC.UpdateNPC`'s
    // tail, `:90465-90493`). The critters have no noTileCollide/noGravity;
    // the gravity is the `UpdateNPC_UpdateGravity` dry row (`:90880-90936`:
    // base 0.3 depth-scaled, maxFallSpeed 10 — none of the per-type rows
    // matches a gold critter; the aiStyle-7 `ai[0]==25` shimmer-town row
    // `:90922` never fires for the wander states). The wet merges
    // (`:90937-90953`) are the documented dry stand-in.
    let gravity = crate::npc_ai_golem::body_gravity(npc.y, world.width, world.world_surface);
    npc.vy = (npc.vy + gravity).min(10.0);
    // `:90473-90476` — the dead-band X snap (double compare).
    if (npc.vx as f64) < 0.005 && (npc.vx as f64) > -0.005 {
        npc.vx = 0.0;
    }
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

/// The wall probe (the `:54087` hop gate): a solid cell ahead at the body
/// row with headroom above → the hop. The ahead column is the C#'s `num23`
/// (`:53969`: centre + 15·direction, int-truncated), the body row `num24`
/// (`:53970`: feet − 16).
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
    //! W4-c9 — tests-as-code, WRITTEN NOT RUN (code-only mode). Every
    //! constant pins to its C# literal (`NPC.cs:53865-54242`).
    use super::*;
    use crate::npc::Npc;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    fn ground() -> World {
        let mut w = World::new("t".into(), 200, 80);
        w.world_surface = 30.0;
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
    fn w4_c9_the_gold_critter_walk_rows() {
        assert_eq!(row(NpcKind::GoldBunny), (1.0, 0.07, true));
        assert_eq!(row(NpcKind::GoldFrog), (1.0, 0.07, true));
        assert_eq!(row(NpcKind::GoldMouse), (2.0, 1.0, false));
    }

    /// The idle → walk start draws Next(300) then Next(200, 400) (the
    /// TownCritter duration), setting ai[0] = 1 (`NPC.cs:53745-53753`).
    #[test]
    fn w4_c9_the_gold_critter_walk_start_draws_two() {
        let w = ground();
        let player = Player::spawn(&w);
        let mut n = Npc::new(NpcKind::GoldBunny, 100.0 * TILE_PX, 40.0 * TILE_PX - 20.0, 1, 0.0);
        n.on_ground = true;
        n.ai[0] = 0.0;
        n.ai[1] = 0.0;
        let mut rng = UnifiedRandom::new(5);
        let mut witness = UnifiedRandom::new(5);
        let d1 = witness.next_max(300);
        let d2 = witness.next_range(200, 400);
        // The idle wander-flip's Next(80) follows the two duration draws.
        let _d3 = witness.next_max(80);
        critter_ai(&mut n, &w, &player, &mut rng);
        assert_eq!(rng.peek(), witness.peek(), "the duration draws + the flip roll");
        assert_eq!(n.ai[0], 1.0, "the walk starts");
        assert_eq!(n.ai[1], 200.0 + d1 as f32 + d2 as f32, "the walk duration");
    }

    /// The walk → idle draws Next(300) then SUBTRACTS Next(100) (the
    /// TownCritter idle, `NPC.cs:53817-53831`), localAI[3] = 60.
    #[test]
    fn w4_c9_the_gold_critter_walk_to_idle_subtracts() {
        let w = ground();
        let player = Player::spawn(&w);
        let mut n = Npc::new(NpcKind::GoldBunny, 100.0 * TILE_PX, 40.0 * TILE_PX - 20.0, 1, 0.0);
        n.on_ground = true;
        n.ai[0] = 1.0;
        n.ai[1] = 1.0; // decrements to 0 → the idle transition
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
