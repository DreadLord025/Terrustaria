//! The NPC movement framework — the movement archetypes extracted from the
//! per-kind `AI_00x` machines (`NPC.cs`'s `AI_001_Slimes`, `AI_002_FloatingEye`,
//! `AI_003_Fighters`, `AI_007_TownNPCs`). Every ported kind's `update_*` composes
//! these steps instead of re-deriving the collision/gravity choreography.
//!
//! The parameters mirror the vanilla per-kind tunables (the acceleration, the
//! max speed, the jump velocity) so a new kind ports as "pick the archetype +
//! the vanilla numbers".

use crate::npc::{Npc, NpcKind};
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
        // `NPC.spriteDirection` is the renderer's flip source
        // (`Main.cs:23124-23128`) and `AI_000_Fighters`' head keeps it in
        // lockstep with the movement direction (`NPC.cs:19867`,
        // `:24893`/`:25607`) — BUG-07: without this write every walker froze
        // at `spriteDirection`'s -1 default once the renderer read it.
        npc.sprite_dir = npc.facing;
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
/// pairs (`NPC.cs:50302-50383`: the max speed / the acceleration). The table
/// is pinned by `the_hover_presets_match_the_csharp_table`, so the anchors
/// cannot drift again (the pre-audit HORNET carried type 205's pair).
#[derive(Clone, Copy, Debug)]
pub struct HoverParams {
    /// `num` — the desired speed (the target vector scaled to it).
    pub max_speed: f32,
    /// `num2` — the velocity approach per tick.
    pub accel: f32,
    /// The type-42/231-235 family arm (`NPC.cs:50341-50362`): its `num`/`num2`
    /// take the `2f - scale` factor and its `velocity.Y` the surface-gated
    /// ×0.97 climb/dive damping. `false` for every other hover kind, which
    /// keeps the plain `num`/`num2` and never damps.
    pub surface_damped: bool,
}

impl HoverParams {
    /// The Eater of Souls (type 6; `NPC.cs:50304-50307`: num 4, num2 0.02).
    pub const EATER_OF_SOULS: Self = Self {
        max_speed: 4.0,
        accel: 0.02,
        surface_damped: false,
    };
    /// The Hornet (type 42; `NPC.cs:50341-50344`: num 3.5, num2 0.021).
    /// Type 205 is the *other* hover kind (`NPC.cs:50364-50367`: 3.25/0.018) —
    /// this preset must not carry its pair.
    pub const HORNET: Self = Self {
        max_speed: 3.5,
        accel: 0.021,
        surface_damped: true,
    };
    /// The Servant of Cthulhu (type 5; `NPC.cs:50379-50382`: num 5, num2 0.03).
    pub const SERVANT_OF_CTHULHU: Self = Self {
        max_speed: 5.0,
        accel: 0.03,
        surface_damped: false,
    };
    /// The Meteor Head (type 23; `NPC.cs:50374-50377`: num 1, num2 0.03).
    pub const METEOR_HEAD: Self = Self {
        max_speed: 1.0,
        accel: 0.03,
        surface_damped: false,
    };
}

/// The hover AI's aim (`NPC.GetTargetData`): the centre drives the desired
/// velocity vector (`NPC.cs:50419-50424`: `position + size/2`), the top edge
/// drives the type-42/231-235 vertical-damping gate (`NPC.cs:50354`/`:50358`:
/// `Main.player[target].position.Y - base.position.Y` — a top-to-top
/// comparison, taken before the 8-px grid floor at `:50421-50424`).
#[derive(Clone, Copy, Debug)]
pub struct HoverTarget {
    /// The target's centre X (`targetData.Position.X + Width/2`).
    pub center_x: f32,
    /// The target's centre Y (`targetData.Position.Y + Height/2`).
    pub center_y: f32,
    /// The target's top edge Y (`targetData.Position.Y`) — the damping gate.
    pub top_y: f32,
}

/// `AI_005_EaterOfSouls`'s fly core (`NPC.cs:50418-50579`, ahead of the
/// per-kind dash/attack arms): the desired velocity = the normalized target
/// vector × `num`; the velocity approaches it by `num2` per tick with the
/// double-add on the sign flips (`:50547-50578`). The type-42/231-235 family
/// also takes the `num`/`num2` `2f - scale` factor (`:50350-50351`) and the
/// surface-gated ×0.97 vertical damping (`:50352-50362`), applied to
/// `velocity.Y` before the approach like the source.
pub fn hover_step(
    npc: &mut Npc,
    world: &World,
    target: HoverTarget,
    params: HoverParams,
) {
    let (w, h) = (npc_size_w(npc), npc_size_h(npc));
    // The type-42/231-235 arm (`NPC.cs:50350-50351`): `num`/`num2` scale by
    // `2f - this.scale` (identity for a default-scale Hornet).
    let (max_speed, accel) = if params.surface_damped {
        let f = 2.0 - npc.scale;
        (params.max_speed * f, params.accel * f)
    } else {
        (params.max_speed, params.accel)
    };
    // The same family's vertical damping (`NPC.cs:50352-50362`): only above
    // `worldSurface`, `velocity.Y *= 0.97` when the target is 300+ px below
    // (climb) or < 80 px below (dive). Applied before the approach (`:50547`).
    if params.surface_damped && (npc.y / TILE_PX) < world.world_surface as f32 {
        let top_dy = target.top_y - npc.y;
        if top_dy > 300.0 && npc.vy < 0.0 {
            npc.vy *= 0.97;
        }
        if top_dy < 80.0 && npc.vy > 0.0 {
            npc.vy *= 0.97;
        }
    }
    let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
    let mut dx = target.center_x - cx;
    let mut dy = target.center_y - cy;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist != 0.0 {
        let scale = max_speed / dist;
        dx *= scale;
        dy *= scale;
    } else {
        dx = npc.vx;
        dy = npc.vy;
    }
    // The approach with the double-add on the sign flips (`NPC.cs:50547-50578`).
    if npc.vx < dx {
        npc.vx += accel;
        if dx > 0.0 && npc.vx < 0.0 {
            npc.vx += accel;
        }
    } else if npc.vx > dx {
        npc.vx -= accel;
        if dx < 0.0 && npc.vx > 0.0 {
            npc.vx -= accel;
        }
    }
    if npc.vy < dy {
        npc.vy += accel;
        if dy > 0.0 && npc.vy < 0.0 {
            npc.vy += accel;
        }
    } else if npc.vy > dy {
        npc.vy -= accel;
        if dy < 0.0 && npc.vy > 0.0 {
            npc.vy -= accel;
        }
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
    npc.facing = if target.center_x >= cx { 1 } else { -1 };
}

/// `AI_008`'s per-kind constants (`NPC.cs`'s aiStyle 8 block).
#[derive(Clone, Copy, Debug)]
pub struct CasterParams {
    /// `num88` — the teleport search radius in tiles (20; the Fire Imp 5,
    /// `NPC.cs:21089-21093`).
    pub teleport_radius: i32,
    /// The post-teleport windup (`ai[1] = num88`, `:21101`).
    pub post_teleport_windup: i32,
    /// The shoot tick — the windup countdown fires the sphere when it
    /// reaches it (`num89` = 25 for the standard casters, `:21115`; the
    /// Fire Imp checks `ai[1] == 10`, `:21166`).
    pub shoot_tick: i32,
}

impl CasterParams {
    /// The Goblin Sorcerer/Tim/Dark Caster group (`:21069`'s triggers with
    /// the `num88 = 20` / `num89 = 25` defaults).
    pub const STANDARD: Self = Self {
        teleport_radius: 20,
        post_teleport_windup: 20,
        shoot_tick: 25,
    };
    /// The Fire Imp (`:21090-21093`'s `num88 = 5`, `:21166`'s shoot tick).
    pub const FIRE_IMP: Self = Self {
        teleport_radius: 5,
        post_teleport_windup: 5,
        shoot_tick: 10,
    };
}

/// `AI_008`'s `ai[0]`/`ai[1]` pair folded into the caller's fields
/// (`npc.caster_counter`/`npc.cast_windup`).
pub struct CasterPhase {
    /// `ai[0]` — the cycle counter (the 500 init, the 100/200/300 shot
    /// triggers, the 650 teleport threshold).
    pub counter: i32,
    /// `ai[1]` — the windup countdown (30 from a trigger, `num88` after a
    /// teleport).
    pub windup: i32,
}

/// `AI_AttemptToFindTeleportSpot` (`NPC.cs:18937-19007`): 100 random
/// attempts, each a random x column scanned downward from a random y in the
/// radius window; the floor must be active+solid, the 3×3 body space above
/// it free (`Collision.SolidTiles(x±1, i-4..i-2)` must be false, fluff 1),
/// the tile above not lava (`:19053`), and the ±5-tile telefrag rect clear
/// of the player's hitbox (`:19056-19071`; the port drops the velocity
/// extrapolation). The caster's own ±1 tile never counts (`:18962`'s skip).
/// Deviations: the Dark Caster's dungeon-wall gate (`:19045-19050`) is
/// constants-off (the dungeon sets land with the dungeon work); the
/// velocity-prediction rect is dropped.
fn find_teleport_spot(
    world: &World,
    rng: &mut u64,
    target_tile: (i32, i32),
    self_tile: (i32, i32),
    radius: i32,
    player_x: f32,
    player_y: f32,
) -> Option<(i32, i32)> {
    for _ in 0..100 {
        let x = target_tile.0
            - radius
            + (crate::app::frame::rand01(rng) * (2 * radius + 1) as f32) as i32;
        let y_start = target_tile.1
            - radius
            + (crate::app::frame::rand01(rng) * (2 * radius + 1) as f32) as i32;
        for y in y_start..target_tile.1 + radius {
            if (x - self_tile.0).abs() <= 1 && (y - self_tile.1).abs() <= 1 {
                continue;
            }
            // The floor: active + solid (`:18966`'s `nactive` skip plus
            // `:18968`'s `tileSolid` reject — the `teleportInAir: false`
            // default).
            if !world
                .tile(x, y)
                .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind))
            {
                continue;
            }
            // The lava gate (`:19053`'s `else if (lava())` — the tile above
            // the floor must not be lava).
            if world.tile(x, y - 1).is_some_and(|t| {
                t.liquid > 0 && t.liquid_kind() == tr_core::tile::LiquidKind::Lava
            }) {
                continue;
            }
            // The body space above the floor (`:18971-18975`):
            // `SolidTiles(x - fluff..x + fluff, i - 3 - fluff..i - 1)` must
            // be false — fluff 1: cols x-1..x+1, rows i-4..i-1 (four rows —
            // the 40 px caster body spans three of them plus the settle).
            let mut blocked = false;
            'outer: for bx in (x - 1)..=(x + 1) {
                for by in (y - 4)..=(y - 1) {
                    if world
                        .tile(bx, by)
                        .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind))
                    {
                        blocked = true;
                        break 'outer;
                    }
                }
            }
            if blocked {
                continue;
            }
            // The telefrag gate (`:19056-19071`): the spot's 16×16 rect
            // inflated by 5 tiles (176 px) must not intersect the player's
            // 20x42 hitbox.
            let sx = (x * 16 - 80) as f32;
            let sy = (y * 16 - 80) as f32;
            if sx < player_x + 10.0
                && sx + 176.0 > player_x - 10.0
                && sy < player_y + 21.0
                && sy + 176.0 > player_y - 21.0
            {
                continue;
            }
            return Some((x, y));
        }
    }
    None
}

/// `AI_008`'s caster cycle (`NPC.cs:20817-21278`). The order is vanilla:
/// the stand snap, the 500 counter init (`:20836-20843`), `ai[0]++`
/// (`:20993`), the shot triggers at 100/200/300 (`:21069-21073` — `flag5`'s
/// firing-distance gate is constant-on in the port, the
/// `Main.MaxWorldViewSize` rectangle lands with the camera work), the
/// teleport attempt at 650 (`:21083-21112`), the generic gravity + floor
/// stop (`:90465-90470`'s `NPC.gravity` block + `UpdateCollision`), then
/// the windup countdown + the shoot (`:21113+`). Returns true on the shoot
/// tick — the caller spawns the sphere through `wants_sphere`.
pub fn caster_cycle(
    npc: &mut Npc,
    world: &World,
    target_x: f32,
    target_y: f32,
    rng: &mut u64,
    params: CasterParams,
    phase: &mut CasterPhase,
) -> bool {
    // The stand: `velocity.X *= 0.93`, snapped to 0 inside ±0.1
    // (`:20831-20835`).
    npc.vx *= 0.93;
    if npc.vx > -0.1 && npc.vx < 0.1 {
        npc.vx = 0.0;
    }
    npc.facing = if target_x >= npc.x + npc.size().0 * 0.5 {
        1
    } else {
        -1
    };
    if phase.counter == 0 {
        phase.counter = 500;
    }
    phase.counter += 1;
    if phase.counter == 100 || phase.counter == 200 || phase.counter == 300 {
        phase.windup = 30;
    }
    if phase.counter >= 650 {
        phase.counter = 1;
        let (w, h) = npc.size();
        let self_tile = (
            ((npc.x + w * 0.5) / TILE_PX) as i32,
            ((npc.y + h * 0.5) / TILE_PX) as i32,
        );
        if let Some((tx, ty)) = find_teleport_spot(
            world,
            rng,
            ((target_x / TILE_PX) as i32, (target_y / TILE_PX) as i32),
            self_tile,
            params.teleport_radius,
            target_x,
            target_y,
        ) {
            phase.windup = params.post_teleport_windup;
            // The teleport placement (`:20924-20930`): the feet land on the
            // spot's floor tile, the X centred on the tile, velocity zeroed.
            npc.x = tx as f32 * TILE_PX - w * 0.5 + 8.0;
            npc.y = ty as f32 * TILE_PX - h;
            npc.vx = 0.0;
            npc.vy = 0.0;
        }
    }
    // The X stand decay meets the tiles (`UpdateCollision`'s X stop).
    let (w, h) = npc.size();
    let nx = npc.x + npc.vx;
    if !crate::player::collides_rect(world, nx, npc.y, w, h) {
        npc.x = nx;
    } else {
        npc.vx = 0.0;
    }
    // The generic gravity + floor stop (`:90465-90470` + `UpdateCollision`).
    npc.vy = (npc.vy + crate::npc::GRAVITY).min(crate::npc::MAX_FALL);
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
    // The windup countdown + the shoot tick (`:21113-21175`).
    if phase.windup > 0 {
        phase.windup -= 1;
        if phase.windup == params.shoot_tick {
            return true;
        }
    }
    false
}

/// `AI_009`'s sphere core (`NPC.cs:21404-21429`): the velocity points at
/// the player ONCE (`target == 255`'s launch arm — `TargetClosest` + the
/// normalized delta × `num120`), then the ball flies straight
/// (`noGravity` + `noTileCollide`: the generic `position += velocity`
/// integration, `:90495-90498`). The port launches while the velocity is
/// still zero. `num120` is 6, the Burning Sphere 5 (`:21395-21399`).
pub fn sphere_step(npc: &mut Npc, target_x: f32, target_y: f32, world: &World) {
    let (w, h) = npc.size();
    if npc.vx == 0.0 && npc.vy == 0.0 {
        let dx = target_x - (npc.x + w * 0.5);
        let dy = target_y - (npc.y + h * 0.5);
        let d = (dx * dx + dy * dy).sqrt().max(1.0);
        let speed = match npc.kind {
            NpcKind::BurningSphere => 5.0,
            _ => 6.0,
        };
        npc.vx = dx / d * speed;
        npc.vy = dy / d * speed;
    }
    npc.x += npc.vx;
    npc.y += npc.vy;
    npc.facing = if npc.vx >= 0.0 { 1 } else { -1 };
    // The off-world guard (the flyer_step shape): the balls fly through
    // everything, so the only stop is the world edge.
    if npc.y > world.height as f32 * TILE_PX + 200.0 {
        npc.hp = 0;
    }
}

/// The swim tunables — `AI_016`'s per-kind constants (`NPC.cs`'s aiStyle 16
/// block: the chase accel/caps and the aggro gate).
#[derive(Clone, Copy, Debug)]
pub struct SwimParams {
    /// The chase acceleration per axis (`:23805`'s 0.15 for the Shark,
    /// `:23826`'s 0.1 for the rest).
    pub chase_accel: f32,
    /// The chase X cap (`:23807`'s 5 for the Shark, `:23828`'s 3).
    pub chase_max_x: f32,
    /// The chase Y cap (`:23815`'s 3 for the Shark, `:23836`'s 2).
    pub chase_max_y: f32,
    /// The idle X cap (`:23906`'s num258 = 1).
    pub idle_max_x: f32,
    /// The aggro gate — `flag25`'s kind exclusion (`:23701`): the Goldfish
    /// never chases (55 sits in the exclusion list).
    pub aggro: bool,
}

impl SwimParams {
    /// The Shark (`:23803-23823`'s 65/102/692 group).
    pub const SHARK: Self = Self {
        chase_accel: 0.15,
        chase_max_x: 5.0,
        chase_max_y: 3.0,
        idle_max_x: 1.0,
        aggro: true,
    };
    /// The Piranha (`:23824-23844`'s else group).
    pub const PIRANHA: Self = Self {
        chase_accel: 0.1,
        chase_max_x: 3.0,
        chase_max_y: 2.0,
        idle_max_x: 1.0,
        aggro: true,
    };
    /// The Goldfish — the flag25 exclusion (`:23701`).
    pub const GOLDFISH: Self = Self {
        chase_accel: 0.1,
        chase_max_x: 3.0,
        chase_max_y: 2.0,
        idle_max_x: 1.0,
        aggro: false,
    };
}

/// `Collision.CanHit`'s stand-in: the segment sampled every half tile
/// rejects solid tiles. The vanilla's exact shape walk lands with the
/// projectile line-of-sight work.
pub fn line_clear(world: &World, from: (f32, f32), to: (f32, f32)) -> bool {
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    let steps = ((dx * dx + dy * dy).sqrt() / (TILE_PX * 0.5)).ceil().max(1.0) as i32;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let x = ((from.0 + dx * t) / TILE_PX) as i32;
        let y = ((from.1 + dy * t) / TILE_PX) as i32;
        if world
            .tile(x, y)
            .is_some_and(|tl| tl.active() && tr_core::tiledata::TILE_SOLID.has(tl.kind))
        {
            return false;
        }
    }
    true
}

/// `AI_016`'s wet machine (`NPC.cs:23698-23961`). `aggro_ok` is `flag25` —
/// the caller folds the kind gate (`:23701`'s exclusion), the player's
/// `wet`/alive state and `CanHit`. The order is vanilla: the slope flips at
/// the feet (`:23709-23736`), the idle bob (`:23848-23960`: the X accel to
/// the 1-px cap, the ±0.3 vertical bob through `ai[0]`, the beach-avoid,
/// the 0.95 damping past 0.4) or the chase (`:23766-23845`: `ai[0] = 0`,
/// the accel to the kind caps), then the tilt (`:24008-24019`) and the
/// tile integration (the X flip on the wall — `:23739-23744`; the Y bounce
/// through `directionY`/`ai[0]` — `:23745-23760`).
pub fn swim_step(
    npc: &mut Npc,
    world: &World,
    target_x: f32,
    target_y: f32,
    aggro_ok: bool,
    params: SwimParams,
) {
    let (w, h) = npc.size();
    let cx = npc.x + w * 0.5;
    // The slope flips at the feet (`:23709-23736`): the tile under the
    // centre (and one below) with a top slope pushes the fish downhill —
    // `topSlope()` = slope 1|2, `leftSlope()` = slope 2|4 (`Tile.cs:290+`).
    let feet_y = ((npc.y + h) / TILE_PX) as i32;
    let feet_x = (cx / TILE_PX) as i32;
    for dy in 0..=1 {
        let slope = world.tile(feet_x, feet_y + dy).map(|t| t.slope());
        if let Some(sl) = slope {
            if matches!(sl, 1 | 2) {
                if sl == 2 || sl == 4 {
                    npc.facing = -1;
                    npc.vx = -npc.vx.abs();
                } else {
                    npc.facing = 1;
                    npc.vx = npc.vx.abs();
                }
                break;
            }
        }
    }
    if aggro_ok {
        // The chase (`:23766-23845`): `ai[0] = 0`, the target direction,
        // the accel to the kind caps.
        if npc.swim_phase != 0.0 {
            npc.swim_phase = 0.0;
        }
        npc.facing = if target_x >= cx { 1 } else { -1 };
        let dir_y = if target_y >= npc.y + h * 0.5 { 1 } else { -1 };
        npc.vx += npc.facing as f32 * params.chase_accel;
        npc.vy += dir_y as f32 * params.chase_accel;
        if npc.vx > params.chase_max_x {
            npc.vx = params.chase_max_x;
        }
        if npc.vx < -params.chase_max_x {
            npc.vx = -params.chase_max_x;
        }
        if npc.vy > params.chase_max_y {
            npc.vy = params.chase_max_y;
        }
        if npc.vy < -params.chase_max_y {
            npc.vy = -params.chase_max_y;
        }
    } else {
        // The idle bob (`:23848-23960`).
        if npc.swim_phase == 0.0 {
            npc.swim_phase = 1.0;
        }
        npc.vx += npc.facing as f32 * 0.1;
        if npc.vx < -params.idle_max_x || npc.vx > params.idle_max_x {
            npc.vx *= 0.95;
        }
        if npc.swim_phase == -1.0 {
            npc.vy -= 0.01;
            if npc.vy < -0.3 {
                npc.swim_phase = 1.0;
            }
        } else {
            npc.vy += 0.01;
            if npc.vy > 0.3 {
                npc.swim_phase = -1.0;
            }
        }
        // The beach-avoid (`:23946-23956`): water above the centre and a
        // tile one or two below it active — rise.
        let ccx = (cx / TILE_PX) as i32;
        let ccy = ((npc.y + h * 0.5) / TILE_PX) as i32;
        let water_above = world
            .tile(ccx, ccy - 1)
            .is_some_and(|t| t.liquid > 128);
        let floor_below = world.tile(ccx, ccy + 1).is_some_and(|t| t.active())
            || world.tile(ccx, ccy + 2).is_some_and(|t| t.active());
        if water_above && floor_below {
            npc.swim_phase = -1.0;
        }
        if npc.vy > 0.4 || npc.vy < -0.4 {
            npc.vy *= 0.95;
        }
    }
    // The tilt (`:24008-24019`): `rotation = vy * direction * 0.1` clamped
    // to ±0.2.
    npc.rotation = (npc.vy * npc.facing as f32 * 0.1).clamp(-0.2, 0.2);
    // The X integration + the wall flip (`:23739-23744`).
    let nx = npc.x + npc.vx;
    if !crate::player::collides_rect(world, nx, npc.y, w, h) {
        npc.x = nx;
    } else {
        npc.vx = -npc.vx;
        npc.facing = -npc.facing;
    }
    // The Y integration + the bounce (`:23745-23760`): the sign flips and
    // `ai[0]` rides the new vertical direction.
    let ny = npc.y + npc.vy;
    if !crate::player::collides_rect(world, npc.x, ny, w, h) {
        npc.y = ny;
    } else if npc.vy > 0.0 {
        npc.vy = -npc.vy.abs();
        npc.swim_phase = -1.0;
    } else if npc.vy < 0.0 {
        npc.vy = npc.vy.abs();
        npc.swim_phase = 1.0;
    } else {
        npc.vy = 0.0;
    }
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

    #[test]
    fn the_hover_presets_match_the_csharp_table() {
        // The per-kind `num`/`num2` pairs, re-read from `NPC.cs` (1.4.5.8).
        // The pre-audit bug shipped type 205's pair under the Hornet, so pin
        // every row against its own C# branch (`surface_damped` = the
        // type-42/231-235 arm at `NPC.cs:50341-50362`).
        let table: [(HoverParams, f32, f32, bool, &str); 4] = [
            (
                HoverParams::EATER_OF_SOULS,
                4.0,
                0.02,
                false,
                "NPC.cs:50304-50307 (type 6)",
            ),
            (
                HoverParams::HORNET,
                3.5,
                0.021,
                true,
                "NPC.cs:50341-50344 (type 42)",
            ),
            (
                HoverParams::SERVANT_OF_CTHULHU,
                5.0,
                0.03,
                false,
                "NPC.cs:50379-50382 (type 5)",
            ),
            (
                HoverParams::METEOR_HEAD,
                1.0,
                0.03,
                false,
                "NPC.cs:50374-50377 (type 23)",
            ),
        ];
        for (p, num, num2, flap, src) in table {
            assert_eq!(p.max_speed, num, "{src}: num");
            assert_eq!(p.accel, num2, "{src}: num2");
            assert_eq!(p.surface_damped, flap, "{src}: the family arm");
        }
        // The regression the DONE-row re-verification caught: the Hornet
        // carried type 205's pair (3.25/0.018, `NPC.cs:50364-50367`).
        assert_ne!(
            (HoverParams::HORNET.max_speed, HoverParams::HORNET.accel),
            (3.25, 0.018),
            "the Hornet must not ship type 205's constants"
        );
    }
}
