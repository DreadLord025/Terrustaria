//! W4-f42 (sandstorm second half) — the desert debris + family AI machines.
//!
//! The `NPC.cs:3949-4018` sandstorm spawn arm picks these kinds; this module
//! ports their `AI()` arms' velocity/state cores (pure — the caller in
//! `npc.rs` integrates, the `update_cave_bat` convention):
//! * **Vulture (61)** — aiStyle 17 (`NPC.cs:24022-24152`): the perched
//!   takeoff + the bounce-flight chase.
//! * **Antlion (69)** — aiStyle 19 (`NPC.cs:24407-24536`): the buried
//!   sand-ball spit + the surface emergence.
//! * **FlyingAntlion (581)** — aiStyle 44 (`NPC.cs:30861-31124`, the 581
//!   rows `:30987-31019`): the bounce-flight hover.
//! * **Tumbleweed (546)** — aiStyle 26: the shared `AI_026_Unicorns` core
//!   rides `npc_ai_unicorn::unicorn_step` (the 315/329 machine) with the 546
//!   branches woven in there; this module owns only the pre-pass inputs.
//!
//! The arms return [`DesertOutcome`] — the per-tick `noGravity`/`noTileCollide`
//! flags the C# AI writes (`NPC.cs:24024`/`:24510-24516`/`:30863`); the caller
//! picks the integration path off them (the style71 `out.no_gravity`
//! convention). The mummies + WalkingAntlion (aiStyle 3) ride the shared
//! `update_zombie` machine — their speed rows live in `npc_ai3_speeds`.
//!
//! RNG (`Main.rand` rides the `UpdateNPCs` stream): the Vulture/FlyingAntlion
//! draw nothing; the Antlion's burrow-dust arm (`NPC.cs:24517-24528`) draws
//! `Next(2)` + the optional `Next(2)` noGravity/scale gate — the dust bodies
//! are the projectile-arm skip, the explicit draws stay on the stream.

use crate::npc::{HostileShotAi, Npc, NpcKind};
use crate::npc_target::{target_closest, TargetCandidate};
use crate::player::Player;
use tr_core::{UnifiedRandom, World};

/// The per-tick gravity/collision flags the C# AI writes — the caller
/// integrates off them (the style71 `out.no_gravity` convention).
#[derive(Clone, Copy, Debug, Default)]
pub struct DesertOutcome {
    pub no_gravity: bool,
    pub no_tile_collide: bool,
}

/// The session-player candidate (the `npc_ai_bat` convention).
fn cand(player: &Player) -> TargetCandidate {
    TargetCandidate {
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
    }
}

/// W4-f42 — `AI_017` (`NPC.cs:24022-24152`), the Vulture's perched takeoff +
/// bounce-flight chase. Returns the noGravity flag (false while perched).
pub fn vulture_ai(npc: &mut Npc, world: &World, player: &Player) -> DesertOutcome {
    let (w, h) = npc.size();
    npc.wet = crate::npc::is_wet(world, npc.x, npc.y, w, h);
    let mut out = DesertOutcome { no_gravity: true, no_tile_collide: false };
    if npc.ai[0] == 0.0 {
        // `:24025-24047` — the perch: noGravity false, TargetClosest, then
        // the takeoff gate (moving, or the player inside the ±100 box, or
        // hurt).
        out.no_gravity = false;
        target_closest(npc, world, &[cand(player)], true, 17, 0);
        if npc.vx != 0.0 || npc.vy < 0.0 || (npc.vy as f64) > 0.3 {
            npc.ai[0] = 1.0;
            npc.net_update = true;
        } else {
            let px = player.x as i32;
            let py = player.y as i32;
            let pw = crate::player::WIDTH as i32;
            let ph = crate::player::HEIGHT as i32;
            let nx = npc.x as i32 - 100;
            let ny = npc.y as i32 - 100;
            let nw = w as i32 + 200;
            let nh = h as i32 + 200;
            let intersects = nx < px + pw && nx + nw > px && ny < py + ph && ny + nh > py;
            if intersects || npc.hp < npc.max_hp {
                npc.ai[0] = 1.0;
                npc.vy -= 6.0;
                npc.net_update = true;
            }
        }
    } else if !player.dead {
        // `:24048-24137` — the flight: the collideX/collideY bounces, the
        // TargetClosest, the ±3 approach, the vertical hover.
        if npc.collide_x {
            npc.vx = npc.old_vx * -0.5;
            if npc.facing == -1 && npc.vx > 0.0 && npc.vx < 2.0 {
                npc.vx = 2.0;
            }
            if npc.facing == 1 && npc.vx < 0.0 && npc.vx > -2.0 {
                npc.vx = -2.0;
            }
        }
        if npc.collide_y {
            npc.vy = npc.old_vy * -0.5;
            if npc.vy > 0.0 && npc.vy < 1.0 {
                npc.vy = 1.0;
            }
            if npc.vy < 0.0 && npc.vy > -1.0 {
                npc.vy = -1.0;
            }
        }
        target_closest(npc, world, &[cand(player)], true, 17, 0);
        if npc.facing == -1 && npc.vx > -3.0 {
            npc.vx -= 0.1;
            if npc.vx > 3.0 {
                npc.vx -= 0.1;
            } else if npc.vx > 0.0 {
                npc.vx -= 0.05;
            }
            if npc.vx < -3.0 {
                npc.vx = -3.0;
            }
        } else if npc.facing == 1 && npc.vx < 3.0 {
            npc.vx += 0.1;
            if npc.vx < -3.0 {
                npc.vx += 0.1;
            } else if npc.vx < 0.0 {
                npc.vx += 0.05;
            }
            if npc.vx > 3.0 {
                npc.vx = 3.0;
            }
        }
        // `:24107-24136` — the hover: num261 the |centre-x| gap, num262 the
        // player's TOP minus half-height (−100 more when the gap > 50).
        let num261 = (npc.x + (w as i32 / 2) as f32
            - (player.x + (crate::player::WIDTH as i32 / 2) as f32))
            .abs();
        let mut num262 = player.y - (h as i32 / 2) as f32;
        if num261 > 50.0 {
            num262 -= 100.0;
        }
        if npc.y < num262 {
            npc.vy += 0.05;
            if npc.vy < 0.0 {
                npc.vy += 0.01;
            }
        } else {
            npc.vy -= 0.05;
            if npc.vy > 0.0 {
                npc.vy -= 0.01;
            }
        }
        npc.vy = npc.vy.clamp(-3.0, 3.0);
    }
    // `:24138-24150` — the wet arm.
    if npc.wet {
        if npc.vy > 0.0 {
            npc.vy *= 0.95;
        }
        npc.vy -= 0.5;
        if npc.vy < -4.0 {
            npc.vy = -4.0;
        }
        target_closest(npc, world, &[cand(player)], true, 17, 0);
    }
    out
}

/// W4-f42 — `AI_019` (`NPC.cs:24407-24536`), the Antlion's buried spit +
/// surface emergence. Queues the sand-ball shot into `wants_hostile_shot_ai`;
/// returns the emergence flags. The burrow-dust arm's explicit draws
/// (`Next(2)` + the optional `Next(2)`) ride the caller's stream; the dust
/// bodies are the projectile-arm skip.
pub fn antlion_ai(npc: &mut Npc, world: &World, player: &Player, rng: &mut UnifiedRandom) -> DesertOutcome {
    let (w, h) = npc.size();
    target_closest(npc, world, &[cand(player)], true, 19, 0);
    let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
    // `:24411-24417` — the aim vector scaled to 12.
    let num273 = 12.0f32;
    let mut num274 = player.x + crate::player::WIDTH * 0.5 - cx;
    let mut num275 = player.y - cy;
    let len = (num274 * num274 + num275 * num275).sqrt();
    let num276 = num273 / len;
    num274 *= num276;
    num275 *= num276;
    // `:24418-24444` — the directionY < 0 aim cone.
    let mut flag28 = false;
    if npc.direction_y < 0 {
        npc.rotation = num275.atan2(num274) + 1.57;
        flag28 = !((npc.rotation as f64) < -1.2) && !((npc.rotation as f64) > 1.2);
        if player.dead {
            flag28 = false;
        }
        npc.rotation = npc.rotation.clamp(-0.8, 0.8);
        if npc.vx != 0.0 {
            npc.vx *= 0.9;
            // The C#'s `||` (`:24438`) — verbatim (always true post-×0.9).
            if npc.vx > -0.1 || npc.vx < 0.1 {
                npc.vx = 0.0;
                npc.net_update = true;
            }
        }
    }
    // `:24445-24452` — the cooldown decrement (the Item5 cue at 200).
    if npc.ai[0] > 0.0 {
        if npc.ai[0] == 200.0 {
            npc.wants_sound.push(crate::npc::NpcSoundRequest {
                cue: crate::npc::NpcSoundCue::Item5,
                x: npc.x,
                y: npc.y,
            });
        }
        npc.ai[0] -= 1.0;
    }
    // `:24453-24484` — the spit.
    if npc.ai[0] == 0.0 {
        let num277 = (cx / 16.0) as i32;
        let num278 = ((npc.y + h + 8.0) / 16.0) as i32;
        if flag28 {
            if crate::npc_ai_bat::global_firing_distance_check(npc.center(), player.center())
                && crate::projectile::can_hit_line(world, cx, cy, 1.0, 1.0, player.center().0, player.center().1, 1.0, 1.0)
            {
                npc.ai[0] = 200.0;
                npc.wants_hostile_shot_ai.push(HostileShotAi {
                    kind: crate::projectile::SAND_BALL_FALLING,
                    x: cx,
                    y: cy,
                    vx: num274,
                    vy: num275,
                    damage: 10,
                    ai0: 2.0,
                    ai1: 0.0,
                    scale: None,
                });
                npc.net_update = true;
            }
        } else if world.tile(num277, num278).is_some_and(|t| t.kind == 421 || t.kind == 422) {
            // `:24470-24483` — the hardened-sand spit along the rotation.
            let num280 = npc.rotation - 1.57;
            num274 = num280.cos() * num273;
            num275 = num280.sin() * num273;
            npc.ai[0] = 200.0;
            npc.wants_hostile_shot_ai.push(HostileShotAi {
                kind: crate::projectile::SAND_BALL_FALLING,
                x: cx,
                y: cy,
                vx: num274,
                vy: num275,
                damage: 10,
                ai0: 2.0,
                ai1: 0.0,
                scale: None,
            });
            npc.net_update = true;
        }
    }
    // `:24485-24530` — the floor probe: solid under the feet → emerge
    // (noGravity + noTileCollide, vy −0.2); else burrow (gravity on) + the
    // dust arm.
    let num282 = (npc.x as i32) / 16;
    let num283 = ((npc.x + (w as i32 / 2) as f32) as i32) / 16;
    let num284 = ((npc.x + w) as i32) / 16;
    let num285 = ((npc.y + h) as i32) / 16;
    let solid_under = |tx: i32| {
        world.tile(tx, num285).is_some_and(|t| {
            t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind)
        })
    };
    if solid_under(num282) || solid_under(num283) || solid_under(num284) {
        npc.vy = -0.2;
        return DesertOutcome { no_gravity: true, no_tile_collide: true };
    }
    // `:24517-24528` — the burrow dust: Next(2) gate, then the optional
    // Next(2) noGravity/scale gate. The dust bodies are the skip; the two
    // explicit draws stay on the stream.
    if rng.next_max(2) == 0 {
        let _ = rng.next_max(2);
    }
    DesertOutcome { no_gravity: false, no_tile_collide: false }
}

/// W4-f42 — `AI_044` (`NPC.cs:30861-31124`), the FlyingAntlion's bounce-flight
/// hover (the 581 rows `:30987-31019`). Always noGravity.
pub fn flying_antlion_ai(npc: &mut Npc, world: &World, player: &Player) -> DesertOutcome {
    let (w, h) = npc.size();
    npc.wet = crate::npc::is_wet(world, npc.x, npc.y, w, h);
    // `:30864-30887` — the collideX/collideY direction flips.
    if npc.collide_x {
        npc.facing = if npc.old_vx > 0.0 { -1 } else { 1 };
        npc.vx = npc.facing as f32;
    }
    if npc.collide_y {
        npc.direction_y = if npc.old_vy > 0.0 { -1 } else { 1 };
        npc.vy = npc.direction_y as f32;
    }
    // `:30930-30945` — the target refresh: the 90-tick cooldown while the
    // player is wet/dead/out-of-line, else the decrement.
    let num663 = npc.target;
    let num664 = npc.facing;
    let can_hit = crate::projectile::can_hit_line(
        world,
        npc.x + w * 0.5,
        npc.y + h * 0.5,
        1.0,
        1.0,
        player.center().0,
        player.center().1,
        1.0,
        1.0,
    );
    if npc.target == 255 || player.wet || player.dead || can_hit {
        npc.ai[0] = 90.0;
        target_closest(npc, world, &[cand(player)], true, 44, 0);
    } else if npc.ai[0] > 0.0 {
        npc.ai[0] -= 1.0;
        target_closest(npc, world, &[cand(player)], true, 44, 0);
    }
    if npc.net_update && num663 == npc.target && num664 == npc.facing {
        npc.net_update = false;
    }
    // `:30946-30951` — the 581 rows: accel 0.06, vertical 0.02, cap 4,
    // vertical cap 2, dead-zone 40, far 150.
    let mut num665 = 0.06f32;
    let mut num666 = 0.02f32;
    let mut num667 = 4.0f32;
    let num668 = 2.0f32;
    let num669 = 40.0f32;
    let num670 = 150.0f32;
    let num671 = (npc.x + (w as i32 / 2) as f32
        - (player.x + (crate::player::WIDTH as i32 / 2) as f32))
        .abs();
    let mut num672 = player.center().1 - (h as i32 / 2) as f32;
    // `:30996` — the rotation + the same-type separation (the siblings
    // pre-pass, the Tumbleweed convention).
    npc.rotation = npc.vx * 0.1;
    for &(sx, sy) in &npc.tumbleweed_siblings {
        if (npc.x - sx).abs() + (npc.y - sy).abs() < w {
            if npc.x < sx {
                npc.vx -= 0.05;
            } else {
                npc.vx += 0.05;
            }
            if npc.y < sy {
                npc.vy -= 0.05;
            } else {
                npc.vy += 0.05;
            }
        }
    }
    // `:31035-31048` — the idle drift.
    if npc.ai[0] <= 0.0 {
        num667 *= 0.8;
        num665 *= 0.7;
        num672 = npc.y + h * 0.5 + (npc.direction_y * 1000) as f32;
        if npc.vx < 0.0 {
            npc.facing = -1;
        } else if npc.vx > 0.0 || npc.facing == 0 {
            npc.facing = 1;
        }
    }
    // `:31049-31083` — the horizontal approach.
    if num671 > num669 {
        if npc.facing == -1 && npc.vx > 0.0 - num667 {
            npc.vx -= num665;
            if npc.vx > num667 {
                npc.vx -= num665;
            } else if npc.vx > 0.0 {
                npc.vx -= num665 / 2.0;
            }
            if npc.vx < 0.0 - num667 {
                npc.vx = 0.0 - num667;
            }
        } else if npc.facing == 1 && npc.vx < num667 {
            npc.vx += num665;
            if npc.vx < 0.0 - num667 {
                npc.vx += num665;
            } else if npc.vx < 0.0 {
                npc.vx += num665 / 2.0;
            }
            if npc.vx > num667 {
                npc.vx = num667;
            }
        }
    }
    // `:31084-31111` — the vertical hover.
    if num671 > num670 {
        num672 -= num670 / 2.0;
    }
    if npc.y < num672 {
        npc.vy += num666;
        if npc.vy < 0.0 {
            npc.vy += num666;
        }
    } else {
        npc.vy -= num666;
        if npc.vy > 0.0 {
            npc.vy -= num666;
        }
    }
    npc.vy = npc.vy.clamp(0.0 - num668, num668);
    // `:31112-31123` — the wet arm.
    if npc.wet {
        if npc.vy > 0.0 {
            npc.vy *= 0.95;
        }
        npc.vy -= 0.5;
        if npc.vy < -4.0 {
            npc.vy = -4.0;
        }
    }
    DesertOutcome { no_gravity: true, no_tile_collide: false }
}

/// W4-f42 — the aiStyle-3 desert walkers (the mummies + WalkingAntlion) ride
/// the shared `update_zombie` machine (the C#'s `AI_003_Fighters` is one arm
/// per style; the per-type speed rows live in `npc_ai3_speeds`).
pub fn is_desert_walker(kind: NpcKind) -> bool {
    matches!(
        kind,
        NpcKind::Mummy
            | NpcKind::DarkMummy
            | NpcKind::BloodMummy
            | NpcKind::LightMummy
            | NpcKind::WalkingAntlion
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    fn ground_air() -> World {
        let mut w = World::new("desert".into(), 200, 200);
        for x in 0..200 {
            for y in 120..200 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 53, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    fn player_at(x: f32, y: f32) -> Player {
        let w = ground_air();
        let mut p = Player::spawn(&w);
        p.x = x;
        p.y = y;
        p
    }

    /// `NPC.cs:24025-24047` — the perched Vulture takes off when the player
    /// enters the ±100 box, gaining the −6 vy lift.
    #[test]
    fn the_vulture_takes_off_when_the_player_near() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let mut n = Npc::new(NpcKind::Vulture, 96.0 * 16.0, 120.0 * 16.0 - 36.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[0] = 0.0;
        let out = vulture_ai(&mut n, &w, &p);
        assert_eq!(n.ai[0], 1.0, "the takeoff fired");
        assert!(n.vy < 0.0, "the −6 lift");
        assert!(!out.no_gravity, "the perch tick keeps gravity");
    }

    /// `NPC.cs:24504-24513` — the Antlion emerges (noGravity + noTileCollide,
    /// vy −0.2) when solid sand sits under its feet.
    #[test]
    fn the_antlion_emerges_over_solid_ground() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let mut n = Npc::new(NpcKind::Antlion, 96.0 * 16.0, 120.0 * 16.0 - 24.0, 1, 0.0);
        n.hp = n.max_hp;
        n.direction_y = -1;
        n.ai[0] = 0.0;
        let mut rng = UnifiedRandom::new(7);
        let out = antlion_ai(&mut n, &w, &p, &mut rng);
        assert!(out.no_gravity && out.no_tile_collide, "the emergence flags");
        assert_eq!(n.vy, -0.2);
    }

    /// `NPC.cs:30996` — the FlyingAntlion's rotation follows vx * 0.1.
    #[test]
    fn the_flying_antlion_rotates_with_velocity() {
        let w = ground_air();
        let p = player_at(100.0 * 16.0, 116.0 * 16.0);
        let mut n = Npc::new(NpcKind::FlyingAntlion, 96.0 * 16.0, 100.0 * 16.0, 1, 0.0);
        n.hp = n.max_hp;
        n.vx = 2.0;
        let out = flying_antlion_ai(&mut n, &w, &p);
        assert!((n.rotation - 0.2).abs() < 1e-6, "rotation = vx * 0.1");
        assert!(out.no_gravity);
    }
}
