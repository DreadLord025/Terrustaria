//! W4-d38 — the hardmode dungeon shooters' `AI_003_Fighters` shooter block
//! (`NPC.cs:59069-59800`) for the Paladin (290), the Skeleton Sniper (291),
//! the Tactical Skeleton (292) and the Skeleton Commando (293). The four are
//! EXCLUDED from the generic AI_003 walker by the `num107` default arm's
//! type chain (`:58322`) — this block IS their walk + fire.
//!
//! The stop-and-fire cycle (the martian-shooter seam's shape, extended with
//! the dungeon per-type rows):
//! * `ai[1]` counts down each tick (`:59086-59089`); a hit rests it at 30
//!   and clears the aim octant (`:59091-59097`, `justHit` → `hurt_cd`).
//! * `num162` the shot cooldown: 30 the Paladin, 200 the Sniper, 120 the
//!   Tactical, 90 the Commando (`:59147-59150`/`:59115-59126`); the fire
//!   lands at `ai[1] == num162 / 2` (`:59184`, `:59208`).
//! * the fire (`:59204-59618`): the aim from the body's centre (the Paladin
//!   10 px up, `:59288-59291`), the gravity lead `|dx| * 0.1` (0 for the
//!   Sniper/Tactical, `:59311-59314`), the per-type jitter, the speed
//!   `num164` (9 Paladin, 4 Commando, 11 default), the projectile + damage
//!   (`:59485-59514`), the spawn at centre + the scaled aim (`:59543-59544`),
//!   the Tactical's four-bolt volley (`:59560-59572`), then the aim octant
//!   on `ai[2]` (`:59595-59617`).
//! * the stop entry (`:59634-59712`): grounded + `ai[1] <= 0` + the player
//!   alive + CanHit + the jittered aim under 700 px → `vx *= 0.5`, the octant
//!   on `ai[2]`, `ai[1] = num162`.
//! * the sleep exit (`:59619-59623`) and the stop's X damp (`:59626`).
//!
//! RNG (the caller's `UpdateNPCs` stream, `Main.cs:17767`): the stop entry
//! draws two `Next(-40, 41)` (X then Y, `:59652-59653`); the fire draws the
//! per-type jitter — the Sniper two `Next(-40, 41) * 0.2` (`:59345-59348`),
//! the Paladin/Commando two `Next(-40, 41)` (`:59392-59396`), the Tactical
//! NONE in the aim but two per bolt ×4 in the volley (`:59567-59568`).
//!
//! Deliberate stand-ins (the `update_martian_shooter` precedent, recorded in
//! ANALYSIS): the player's `stealth`/`itemAnimation` gate (`:59641`) is
//! constants-off (the port's Player carries no swing timer); the walk rides
//! the shared `walker_step_fighter` (the C#'s `num180` 1f / 0.07 / 0.8 ladder
//! folds into the fighter core, the pirate/martian shooters' convention);
//! `confused` (`:59197`) reads the buff engine's Confused (31).

use crate::npc::{Npc, NpcKind, GRAVITY, MAX_FALL};
use crate::player::Player;
use tr_core::{UnifiedRandom, World, TILE_PX};

/// The per-type shooter row: (cooldown `num162`, speed `num164`, projectile
/// `num170`, damage `num169`, the centre-Y offset, the gravity-lead factor).
/// Citations are the C# per-type writes inside the block.
fn row(kind: NpcKind) -> (u32, f32, u16, i32, f32, f32) {
    match kind {
        // `:59147-59150` cd 30, `:59223-59226` speed 9, `:59490-59494`
        // proj 300 dmg 60, `:59288-59291` centre −10, `:59309-59310` lead 0.1.
        NpcKind::Paladin => (30, 9.0, 300, 60, -10.0, 0.1),
        // `:59115-59118` cd 200, speed 11 (default), `:59485-59489` proj 302
        // dmg 100, centre 0, `:59311-59314` lead 0.
        NpcKind::SkeletonSniper => (200, 11.0, 302, 100, 0.0, 0.0),
        // `:59119-59122` cd 120, speed 11, `:59510-59514` proj 180 dmg 50,
        // centre 0, lead 0.
        NpcKind::TacticalSkeleton => (120, 11.0, 180, 50, 0.0, 0.0),
        // `:59123-59126` cd 90, `:59227-59230` speed 4, `:59495-59499`
        // proj 303 dmg 60, centre 0, lead 0.1.
        NpcKind::SkeletonCommando => (90, 4.0, 303, 60, 0.0, 0.1),
        _ => unreachable!("the dungeon shooters only"),
    }
}

/// The aim octant the C# writes on `ai[2]` (`:59595-59617` / `:59689-59711`).
fn octant(ax: f32, ay: f32) -> f32 {
    if ay.abs() > ax.abs() * 2.0 {
        if ay > 0.0 { 1.0 } else { 5.0 }
    } else if ax.abs() > ay.abs() * 2.0 {
        3.0
    } else if ay > 0.0 {
        2.0
    } else {
        4.0
    }
}

/// One fixed step of the dungeon shooters' shooter block. The caller's `rng`
/// is the `UpdateNPCs` stream; the walk rides the shared fighter core after
/// the fire/stop logic (the C#'s walk tail, `:59715+`).
pub fn dungeon_shooter_ai(npc: &mut Npc, world: &World, player: &Player, rng: &mut UnifiedRandom) {
    let (cooldown, speed, proj, dmg, cy_off, lead) = row(npc.kind);
    let (w, h) = npc.size();
    let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
    let (pcx, pcy) = player.center();
    // `:59086-59089` — the cooldown counts down.
    if npc.ai[1] > 0.0 {
        npc.ai[1] -= 1.0;
    }
    // `:59091-59097` — a hit rests the cycle at 30 and clears the octant.
    if npc.hurt_cd > 0.0 {
        npc.ai[1] = 30.0;
        npc.ai[2] = 0.0;
        npc.net_update = true;
    }
    // `:59197-59201` — Confused (31) zeroes the cycle.
    if npc.buffs.find_buff_index(31).is_some() {
        npc.ai[1] = 0.0;
        npc.ai[2] = 0.0;
    }
    let num163 = (cooldown / 2) as f32;
    if npc.ai[2] > 0.0 {
        // `:59204-59207` — flag18 (true for these four): retarget each tick.
        let cand = crate::npc_target::session_candidate_for(player, npc.kind);
        crate::npc_target::target_closest(npc, world, &[cand], true, 3, 0);
        // `:59208` — the fire at the half-cooldown.
        if npc.ai[1] == num163 {
            // `:59279` — the aim origin: the body's centre (the Paladin 10 up).
            let mut sx = cx;
            let mut sy = cy + cy_off;
            // `:59309-59310` — the raw aim + the gravity lead.
            let mut ax = pcx - sx;
            let lead_y = ax.abs() * lead;
            let mut ay = pcy - sy - lead_y;
            // `:59345-59396` — the per-type aim jitter.
            match npc.kind {
                NpcKind::SkeletonSniper => {
                    // `:59345-59348` — ±40 × 0.2 on each axis.
                    ax += rng.next_range(-40, 41) as f32 * 0.2;
                    ay += rng.next_range(-40, 41) as f32 * 0.2;
                }
                NpcKind::TacticalSkeleton => {
                    // `:59392` — the Tactical's aim carries NO jitter (the
                    // volley below jitters each bolt instead).
                }
                _ => {
                    // `:59392-59396` — the Paladin/Commando ±40 on each axis.
                    ax += rng.next_range(-40, 41) as f32;
                    ay += rng.next_range(-40, 41) as f32;
                }
            }
            // `:59397-59401` — the speed scale over the aim length.
            let inv = speed / (ax * ax + ay * ay).sqrt();
            ax *= inv;
            ay *= inv;
            // `:59543-59544` — the spawn point: the origin + the scaled aim.
            sx += ax;
            sy += ay;
            npc.net_update = true;
            // `:59558-59593` — the spawn. The Tactical fires four bolts, each
            // re-aimed from the (already shifted) spawn point with its own
            // ±40 jitter at speed 12 (`:59560-59572`); the others fire one.
            if npc.kind == NpcKind::TacticalSkeleton {
                for _ in 0..4 {
                    let mut bx = pcx - sx;
                    let mut by = pcy - sy;
                    let b_inv = 12.0 / (bx * bx + by * by).sqrt();
                    bx += rng.next_range(-40, 41) as f32;
                    by += rng.next_range(-40, 41) as f32;
                    bx *= b_inv;
                    by *= b_inv;
                    npc.wants_hostile_shot.push((proj, sx, sy, bx, by, dmg));
                }
            } else {
                npc.wants_hostile_shot.push((proj, sx, sy, ax, ay, dmg));
            }
            // `:59595-59617` — the aim octant on ai[2].
            npc.ai[2] = octant(ax, ay);
        }
        // `:59619-59623` — the sleep exit: airborne or the timer expired.
        if npc.vy != 0.0 || npc.ai[1] <= 0.0 {
            npc.ai[2] = 0.0;
            npc.ai[1] = 0.0;
        } else {
            // `:59626-59627` — the stop's X damp + the sprite lockstep.
            npc.vx *= 0.9;
            npc.sprite_dir = npc.facing;
        }
    }
    // `:59634-59712` — the stop entry: grounded, the timer idle, the player
    // alive, CanHit, the jittered aim under 700 px. (The stealth/itemAnimation
    // gate `:59641` is constants-off — the martian-shooter stand-in.)
    if npc.ai[2] <= 0.0 && npc.vy == 0.0 && npc.ai[1] <= 0.0 && !player.dead {
        let can_hit = crate::projectile::can_hit(
            world, npc.x, npc.y, w, h, player.x, player.y,
            crate::player::WIDTH, crate::player::HEIGHT,
        );
        if can_hit {
            let mut ax = pcx - cx;
            let lead_y = ax.abs() * 0.1;
            let mut ay = pcy - cy - lead_y;
            // `:59652-59653` — the two Next(-40, 41) draws (X then Y).
            ax += rng.next_range(-40, 41) as f32;
            ay += rng.next_range(-40, 41) as f32;
            let len = (ax * ax + ay * ay).sqrt();
            // `:59655` — num179 = 700 for these four.
            if len < 700.0 {
                npc.net_update = true;
                npc.vx *= 0.5;
                npc.ai[1] = cooldown as f32;
                npc.ai[2] = octant(ax, ay);
            }
        }
    }
    // The walk tail (`:59715+`): the shared fighter core (the num180 1f /
    // 0.07 / 0.8 ladder folds, the pirate/martian shooters' convention). The
    // stop's X damp above already ran; the core keeps the chase/wander gate.
    crate::npc_ai::walker_step_fighter(
        npc,
        world,
        player,
        crate::npc_ai::WalkerParams {
            speed: npc.kind.ai3_walk_speed().unwrap_or(1.0),
            jump: -6.0,
            aggro_x: 80.0 * TILE_PX,
            aggro_y: 40.0 * TILE_PX,
        },
        GRAVITY,
        MAX_FALL,
    );
}

#[cfg(test)]
mod tests {
    //! W4-d38 — tests-as-code, WRITTEN NOT RUN (code-only mode). Every
    //! constant pins to its C# literal (`NPC.cs:59069-59800`).
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

    /// The four shooters' per-type rows (cooldown, speed, projectile, damage).
    #[test]
    fn w4_d38_the_dungeon_shooter_rows() {
        assert_eq!(row(NpcKind::Paladin), (30, 9.0, 300, 60, -10.0, 0.1));
        assert_eq!(row(NpcKind::SkeletonSniper), (200, 11.0, 302, 100, 0.0, 0.0));
        assert_eq!(row(NpcKind::TacticalSkeleton), (120, 11.0, 180, 50, 0.0, 0.0));
        assert_eq!(row(NpcKind::SkeletonCommando), (90, 4.0, 303, 60, 0.0, 0.1));
    }

    /// The Tactical's fire draws EIGHT Next(-40, 41) (two per bolt ×4) and
    /// spawns four 180 bolts at 50 damage (`NPC.cs:59560-59572`).
    #[test]
    fn w4_d38_the_tactical_volley_draws_eight_jitters() {
        let w = ground();
        let mut player = Player::spawn(&w);
        for _ in 0..120 {
            player.update(&w, crate::player::PlayerInput::default());
        }
        let mut n = Npc::new(NpcKind::TacticalSkeleton, player.x - 200.0, 40.0 * TILE_PX - 40.0, 1, 0.0);
        n.on_ground = true;
        // Arm the fire: ai[2] > 0 and ai[1] one ABOVE the half-cooldown —
        // `:59086-59089` decrements FIRST, so ai[1] == num163 (60) on the
        // fire test (`:59208`) needs 61 before the tick.
        n.ai[2] = 3.0;
        n.ai[1] = 61.0;
        let mut rng = UnifiedRandom::new(11);
        let mut witness = UnifiedRandom::new(11);
        for _ in 0..8 {
            let _ = witness.next_range(-40, 41);
        }
        dungeon_shooter_ai(&mut n, &w, &player, &mut rng);
        assert_eq!(rng.peek(), witness.peek(), "the eight volley jitters");
        assert_eq!(n.wants_hostile_shot.len(), 4, "four bolts");
        for &(kind, _, _, _, _, dmg) in &n.wants_hostile_shot {
            assert_eq!(kind, 180);
            assert_eq!(dmg, 50);
        }
    }
}
