//! The NPC movement framework — the movement archetypes extracted from the
//! per-kind `AI_00x` machines (`NPC.cs`'s `AI_001_Slimes`, `AI_002_FloatingEye`,
//! `AI_003_Fighters`, `AI_007_TownNPCs`). Every ported kind's `update_*` composes
//! these steps instead of re-deriving the collision/gravity choreography.
//!
//! The parameters mirror the vanilla per-kind tunables (the acceleration, the
//! max speed, the jump velocity) so a new kind ports as "pick the archetype +
//! the vanilla numbers".

use crate::npc::Npc;
use tr_core::TILE_PX;
use tr_core::World;

/// The archetype tunables — the vanilla per-kind numbers (`NPC.SetDefaults`
/// line groups) a new kind fills in.
#[derive(Clone, Copy, Debug)]
pub struct WalkerParams {
    /// The walk speed toward the player (`npc.velocity.X`'s target).
    pub speed: f32,
    /// The jump velocity at a blocked path (`AI_003`'s `velocity.Y = -6.2`
    /// for the humanoids; the skeleton family jumps the 3-tile walls).
    pub jump: f32,
    /// The aggro radius in pixels (`AI_003`'s 80-tile × 40-tile window for
    /// the zombie; the skeletons see further).
    pub aggro_x: f32,
    pub aggro_y: f32,
}

impl WalkerParams {
    /// The zombie's numbers (`AI_003_Fighters`' defaults, `NPC.cs:10586+`).
    pub const ZOMBIE: Self = Self {
        speed: 0.6,
        jump: -6.2,
        aggro_x: 80.0 * TILE_PX,
        aggro_y: 40.0 * TILE_PX,
    };
    /// The town-NPC wander: the slower pace, no aggro (the direction flips
    /// on the aiTimer).
    pub const WANDER: Self = Self {
        speed: 0.5,
        jump: -6.2,
        aggro_x: 0.0,
        aggro_y: 0.0,
    };
}

/// The flyer tunables.
#[derive(Clone, Copy, Debug)]
pub struct FlyerParams {
    /// The acceleration toward the player (`AI_002`'s 0.12).
    pub accel: f32,
    /// The speed cap (`AI_002`'s 3.0 for the demon eyes).
    pub max_speed: f32,
}

impl FlyerParams {
    /// The demon-eye machine (`AI_002_FloatingEye`, `NPC.cs:52274+`).
    pub const FLOATING_EYE: Self = Self {
        accel: 0.12,
        max_speed: 3.0,
    };
}

/// `AI_003_Fighters`' walk-and-jump step (`NPC.cs:10586+`, the zombie
/// machine): walk toward the player inside the aggro window, jump the
/// one-tile obstacles, integrate the gravity, land on the floor. The caller
/// derives the direction (the chase or the wander) before this runs.
///
/// `facing` and the movement integrate `self.x/y/vx/vy/on_ground`; the
/// function returns after one fixed step.
pub fn walker_step(
    npc: &mut Npc,
    world: &World,
    target_x: f32,
    target_y: f32,
    params: WalkerParams,
    gravity: f32,
    max_fall: f32,
) {
    let (w, h) = (npc_size_w(npc), npc_size_h(npc));
    let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
    let dx = target_x - cx;
    let dy = target_y - cy;
    let aggro = dx.abs() < params.aggro_x && dy.abs() < params.aggro_y;
    if aggro {
        npc.facing = if dx >= 0.0 { 1 } else { -1 };
        npc.vx = npc.facing as f32 * params.speed;
    } else {
        npc.vx *= 0.9;
    }
    npc.vy = (npc.vy + gravity).min(max_fall);

    let nx = npc.x + npc.vx;
    if !crate::player::collides_rect(world, nx, npc.y, w, h) {
        npc.x = nx;
    } else if npc.on_ground {
        npc.vy = params.jump;
        npc.on_ground = false;
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
    } else if npc.vy < 0.0 {
        let ty = (ny / TILE_PX).floor() as i32;
        npc.y = (ty + 1) as f32 * TILE_PX;
        npc.vy = 0.0;
    }
}

/// `AI_002_FloatingEye`'s fly-and-bob step (`NPC.cs:52274+`, the eye
/// machine): accelerate toward the player, bounce off the tiles, the bob
/// rides the anim clock.
pub fn flyer_step(
    npc: &mut Npc,
    world: &World,
    target_x: f32,
    target_y: f32,
    params: FlyerParams,
) {
    let (w, h) = (npc_size_w(npc), npc_size_h(npc));
    let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
    let dx = target_x - cx;
    let dy = target_y - cy;
    let d = (dx * dx + dy * dy).sqrt().max(1.0);
    npc.vx = (npc.vx + dx / d * params.accel).clamp(-params.max_speed, params.max_speed);
    npc.vy = (npc.vy + dy / d * params.accel).clamp(-params.max_speed, params.max_speed);
    let nx = npc.x + npc.vx;
    if !crate::player::collides_rect(world, nx, npc.y, w, h) {
        npc.x = nx;
    } else {
        npc.vx = -npc.vx * 0.5;
    }
    let bob = (npc.anim * 3.0).sin() * 0.4;
    let ny = npc.y + npc.vy + bob;
    if !crate::player::collides_rect(world, npc.x, ny, w, h) {
        npc.y = ny;
    } else {
        npc.vy = -npc.vy * 0.5;
    }
    npc.facing = if dx >= 0.0 { 1 } else { -1 };
    if npc.y > world.height as f32 * TILE_PX + 200.0 {
        npc.hp = 0;
    }
}

/// The hover tunables — `AI_005_EaterOfSouls`'s per-kind `num`/`num2`
/// pairs (`NPC.cs:50294-50330`: the max speed / the acceleration).
#[derive(Clone, Copy, Debug)]
pub struct HoverParams {
    /// `num` — the desired speed (the target vector scaled to it).
    pub max_speed: f32,
    /// `num2` — the velocity approach per tick.
    pub accel: f32,
}

impl HoverParams {
    /// The Eater of Souls (`NPC.cs:50304-50305`: num 4, num2 0.02).
    pub const EATER_OF_SOULS: Self = Self {
        max_speed: 4.0,
        accel: 0.02,
    };
    /// The Hornet (`NPC.cs:50322-50323`: num 3.25, num2 0.018).
    pub const HORNET: Self = Self {
        max_speed: 3.25,
        accel: 0.018,
    };
    /// The Meteor Head (`NPC.cs:50328-50329`: num 1, num2 0.03).
    pub const METEOR_HEAD: Self = Self {
        max_speed: 1.0,
        accel: 0.03,
    };
}

/// `AI_005_EaterOfSouls`'s fly core (`NPC.cs:50392-50370`): the desired
/// velocity = the normalized target vector × max_speed; the velocity
/// approaches it by `accel` per tick with the double-add on the sign flips;
/// the vertical damping (`:50355-50360`) caps the climb/dive when the
/// target is far above/below.
pub fn hover_step(
    npc: &mut Npc,
    world: &World,
    target_x: f32,
    target_y: f32,
    params: HoverParams,
) {
    let (w, h) = (npc_size_w(npc), npc_size_h(npc));
    let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
    let mut dx = target_x - cx;
    let mut dy = target_y - cy;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist != 0.0 {
        let scale = params.max_speed / dist;
        dx *= scale;
        dy *= scale;
    } else {
        dx = npc.vx;
        dy = npc.vy;
    }
    // The approach with the double-add on the sign flips (`:50371-50377`).
    if npc.vx < dx {
        npc.vx += params.accel;
        if dx > 0.0 && npc.vx < 0.0 {
            npc.vx += params.accel;
        }
    } else if npc.vx > dx {
        npc.vx -= params.accel;
        if dx < 0.0 && npc.vx > 0.0 {
            npc.vx -= params.accel;
        }
    }
    if npc.vy < dy {
        npc.vy += params.accel;
        if dy > 0.0 && npc.vy < 0.0 {
            npc.vy += params.accel;
        }
    } else if npc.vy > dy {
        npc.vy -= params.accel;
        if dy < 0.0 && npc.vy > 0.0 {
            npc.vy -= params.accel;
        }
    }
    // The vertical damping (`:50355-50360`): the target 300+ px below
    // damps the climb, 80 px above damps the dive (×0.97).
    if target_y - cy > 300.0 && npc.vy < 0.0 {
        npc.vy *= 0.97;
    }
    if target_y - cy < 80.0 && npc.vy > 0.0 {
        npc.vy *= 0.97;
    }
    let nx = npc.x + npc.vx;
    if !crate::player::collides_rect(world, nx, npc.y, w, h) {
        npc.x = nx;
    } else {
        npc.vx = -npc.vx * 0.5;
    }
    let ny = npc.y + npc.vy;
    if !crate::player::collides_rect(world, npc.x, ny, w, h) {
        npc.y = ny;
    } else {
        npc.vy = -npc.vy * 0.5;
    }
    npc.facing = if target_x >= cx { 1 } else { -1 };
}

fn npc_size_w(npc: &Npc) -> f32 {
    npc.size().0
}

fn npc_size_h(npc: &Npc) -> f32 {
    npc.size().1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::FLAG_ACTIVE;
    use tr_core::Tile;

    fn ground() -> World {
        let mut w = World::new("t".into(), 200, 200);
        for x in 0..200 {
            for y in 120..200 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        // The 2-tall obstacle at x=75: the walker must jump it en route.
        for y in 118..120 {
            let i = w.index(75, y);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w
    }

    #[test]
    fn the_walker_chases_jumps_and_lands() {
        let w = ground();
        let mut npc = Npc::new(NpcKind::Zombie, 90.0 * 16.0, 118.0 * 16.0 - 40.0, 1, 0.0);
        npc.on_ground = true;
        let target = (60.0 * 16.0, 119.0 * 16.0); // to the left, on the ground
        let mut jumped = false;
        for _ in 0..600 {
            walker_step(&mut npc, &w, target.0, target.1, WalkerParams::ZOMBIE, 0.4, 10.0);
            if npc.vy < -1.0 {
                jumped = true;
            }
        }
        assert!(npc.x < 90.0 * 16.0, "the walker approached the target");
        assert!(jumped, "the walker jumped the obstacles");
        assert!(npc.on_ground, "the walker landed");
    }

    #[test]
    fn the_flyer_homes_and_bounces() {
        let w = ground();
        let mut npc = Npc::new(NpcKind::DemonEye, 90.0 * 16.0, 100.0 * 16.0, 1, 0.0);
        let target = (60.0 * 16.0, 100.0 * 16.0);
        for _ in 0..200 {
            flyer_step(&mut npc, &w, target.0, target.1, FlyerParams::FLOATING_EYE);
        }
        let dist = ((npc.x - target.0) / 16.0).abs();
        assert!(dist < 20.0, "the flyer homed (dist {dist} tiles)");
    }
}
