//! The NPC movement framework — the movement archetypes extracted from the
//! per-kind `AI_00x` machines (`NPC.cs`'s `AI_001_Slimes`, `AI_002_FloatingEye`,
//! `AI_003_Fighters`, `AI_007_TownNPCs`). Every ported kind's `update_*` composes
//! these steps instead of re-deriving the collision/gravity choreography.
//!
//! The parameters mirror the vanilla per-kind tunables (the acceleration, the
//! max speed, the jump velocity) so a new kind ports as "pick the archetype +
//! the vanilla numbers".

use crate::npc::{Npc, NpcKind};
use tr_core::UnifiedRandom;
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
    walker_integrate(npc, world, params, gravity, max_fall);
}

/// The gravity + tile-collision integration shared by [`walker_step`] and
/// [`walker_step_ladder`] — verbatim the second half of the walk step
/// (`NPC.cs:10586+`'s position pass). Extracted so the ladder variant
/// swaps only the velocity write, never the choreography.
fn walker_integrate(
    npc: &mut Npc,
    world: &World,
    params: WalkerParams,
    gravity: f32,
    max_fall: f32,
) {
    let (w, h) = (npc_size_w(npc), npc_size_h(npc));
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

/// S1-D-3 (speed table) — [`walker_step`] with the `AI_003_Fighters`
/// per-type velocity ladder (`npc_ai3_speeds.rs`, `NPC.cs:57403-58401`)
/// replacing the direct `vx = facing * speed` write for the aiStyle-3
/// kinds. The facing/sprite writes, the aggro window, the gravity and the
/// integration stay byte-identical to [`walker_step`]; a kind the ladder
/// declines (sibling-owned frost ids, the shooter-block exclusions) or a
/// non-aiStyle-3 kind falls back to the direct write, so every existing
/// caller's behaviour is preserved unless the C# ladder names its type.
pub fn walker_step_ladder(
    npc: &mut Npc,
    world: &World,
    target_x: f32,
    target_y: f32,
    params: WalkerParams,
    gravity: f32,
    max_fall: f32,
    target_dead: bool,
) {
    let (w, h) = (npc_size_w(npc), npc_size_h(npc));
    let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
    let dx = target_x - cx;
    let dy = target_y - cy;
    let aggro = dx.abs() < params.aggro_x && dy.abs() < params.aggro_y;
    if aggro {
        npc.facing = if dx >= 0.0 { 1 } else { -1 };
        // BUG-07's lockstep write, as `walker_step` (`NPC.cs:19867`).
        npc.sprite_dir = npc.facing;
        if !run_ai3_ladder(npc, world, target_x, target_y, gravity, target_dead) {
            npc.vx = npc.facing as f32 * params.speed;
        }
    } else {
        npc.vx *= 0.9;
    }
    walker_integrate(npc, world, params, gravity, max_fall);
}

/// Build the [`crate::npc_ai3_speeds::LadderCtx`] from the NPC + world +
/// target, run the ladder and write its results back. Returns false when
/// the kind is not aiStyle 3 or the ladder declines the type (the caller
/// keeps its current walk). The zombie's own machine (`npc.rs`'s
/// `update_zombie`) calls this at the C#'s ladder position too.
pub fn run_ai3_ladder(
    npc: &mut Npc,
    world: &World,
    target_x: f32,
    target_y: f32,
    gravity: f32,
    target_dead: bool,
) -> bool {
    if npc.kind.ai_style() != 3 {
        return false;
    }
    let (w, h) = (npc_size_w(npc), npc_size_h(npc));
    let (cx, cy) = (npc.x + w * 0.5, npc.y + h * 0.5);
    let mut ctx = crate::npc_ai3_speeds::LadderCtx {
        npc_type: npc.kind.cs_id(),
        vx: npc.vx,
        vy: npc.vy,
        direction: npc.facing,
        sprite_direction: npc.sprite_dir,
        scale: npc.scale,
        life: npc.hp,
        life_max: npc.max_hp,
        ai2: npc.ai[2],
        ai3: npc.ai[3],
        center_x: cx,
        center_y: cy,
        // The port's effective per-tick gravity stands in for the C#'s
        // `NPC.gravity` static (`NPC.cs:90881`'s 0.3f default).
        gravity,
        target_center_x: target_x,
        target_center_y: target_y,
        target_dead,
        day_time: world.day_time,
        below_surface: (npc.y as f64) > world.world_surface * 16.0,
        // `Collision.CanHit(Center, 1, 1, …)` (`:58299`) — the port's
        // line-ray seam (`projectile::can_hit_line`, the `CanHitLine`
        // port) at the same 1×1 centres.
        can_hit_target: crate::projectile::can_hit_line(
            world, cx, cy, 1.0, 1.0, target_x, target_y, 1.0, 1.0,
        ),
        def_damage: npc.kind.contact_damage(),
    };
    let Some(((vx, vy), fx)) = crate::npc_ai3_speeds::apply_ladder(&mut ctx) else {
        return false;
    };
    npc.vx = vx;
    npc.vy = vy;
    if let Some(v) = fx.ai2 {
        npc.ai[2] = v;
    }
    if let Some(v) = fx.ai3 {
        npc.ai[3] = v;
    }
    if let Some(v) = fx.damage {
        // The port's live `NPC.damage` seam is `damage_override` (the
        // contact path reads it at `npc.rs`'s `damage_override.unwrap_or`
        // over the kind's `contact_damage()` — the C# `this.damage` write,
        // `NPC.cs:58266`/`:58308`).
        npc.damage_override = Some(v);
    }
    // `fx.hop_flag` (the `:57847` flag) feeds the C#'s obstacle-jump scan
    // (`:59831`) — the port's collision jump is that scan's stand-in;
    // ANALYSIS deferral.
    true
}

/// `AI_002_FloatingEye`'s fly-and-bob step (`NPC.cs:52274+`, the eye
/// machine): accelerate toward the player, bounce off the tiles, the bob
/// rides the anim clock. The last live caller (the bat stand-in) retired
/// with the S1-D-3 reconcile — the archetype stays for the aiStyle-2
/// slice's A/B pin (`the_flyer_homes_and_bounces`).
#[allow(dead_code)]
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

/// `AI_005`'s per-kind rotation arm (`NPC.cs:50580-50655`) — the four
/// shapes the fly core ends with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoverRotation {
    /// `:50643-50646` — the aim tilt (`atan2(num5, num4) - 1.57`): the
    /// types 6/94/173/619.
    Aim,
    /// `:50580-50592` — the sprite-aim arm (type 23): the sprite direction
    /// follows the aim's x sign and the rotation is `atan2(num5, num4)`
    /// (+π when the aim points left).
    SpriteAim,
    /// `:50647-50651` — the type-42/176/205/231-235 arm: the sprite
    /// direction follows the velocity's x sign and the rotation is
    /// `velocity.X * 0.1`.
    VelocityX,
    /// `:50652-50655` — the else arm: the velocity tilt
    /// (`atan2(velocity.Y, velocity.X) - 1.57`).
    Velocity,
}

/// The hover tunables — `AI_005_EaterOfSouls`'s per-kind `num`/`num2`
/// pairs (`NPC.cs:50302-50383`: the max speed / the acceleration) plus the
/// per-kind gates of the shared fly core. The table is pinned by
/// `the_hover_presets_match_the_csharp_table`, so the anchors cannot drift
/// again (the pre-audit HORNET carried type 205's pair).
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
    /// `flag5` (`NPC.cs:50447`) — the approach's extra add on a sign
    /// reversal. It is true for the kinds **outside** {6, 42, 94, 139, 173,
    /// 619} and outside [231, 235]; the Eater of Souls (6) and the Hornet
    /// (42) are in the excluded set and never get the extra add.
    pub double_add: bool,
    /// `noTileCollide` (`SetDefaults`) — the kinds that fly through the
    /// tiles (`AI_005`'s own kind list at `:50656` excludes them from the
    /// bounce arm); the others bounce with `bounce`.
    pub no_tile_collide: bool,
    /// The tile bounce's restitution (`NPC.cs:50656-50688`): `num14` = 0.7
    /// for the family, 0.4 for the types 6/173.
    pub bounce: f32,
    /// The per-kind rotation arm (`NPC.cs:50580-50655`).
    pub rotation: HoverRotation,
}

impl HoverParams {
    /// The Eater of Souls (type 6; `NPC.cs:50304-50307`: num 4, num2 0.02).
    /// `flag5` is false for it (`:50447`) and it flies through the tiles
    /// (`SetDefaults` case 6: `noTileCollide`), so it never bounces; the
    /// rotation is the aim tilt (`:50643-50646`).
    pub const EATER_OF_SOULS: Self = Self {
        max_speed: 4.0,
        accel: 0.02,
        surface_damped: false,
        double_add: false,
        no_tile_collide: true,
        bounce: 0.4,
        rotation: HoverRotation::Aim,
    };
    /// The Hornet (type 42; `NPC.cs:50341-50344`: num 3.5, num2 0.021).
    /// Type 205 is the *other* hover kind (`NPC.cs:50364-50367`: 3.25/0.018) —
    /// this preset must not carry its pair. `flag5` is false for the Hornet
    /// (`:50447`), it **collides** with the tiles (`SetDefaults` case 42 sets
    /// `noGravity` only), so it takes the ×0.7 bounce (`:50658`), and its
    /// rotation is the `velocity.X * 0.1` arm (`:50647-50651`).
    pub const HORNET: Self = Self {
        max_speed: 3.5,
        accel: 0.021,
        surface_damped: true,
        double_add: false,
        no_tile_collide: false,
        bounce: 0.7,
        rotation: HoverRotation::VelocityX,
    };
    /// The Servant of Cthulhu (type 5; `NPC.cs:50379-50382`: num 5, num2
    /// 0.03). The servant has its own arm now (S1-D-3k); the row stays as
    /// the record of the C#'s type-5 pair.
    pub const SERVANT_OF_CTHULHU: Self = Self {
        max_speed: 5.0,
        accel: 0.03,
        surface_damped: false,
        double_add: true,
        no_tile_collide: true,
        bounce: 0.4,
        rotation: HoverRotation::Velocity,
    };
    /// The Meteor Head (type 23; `NPC.cs:50374-50377`: num 1, num2 0.03).
    /// `flag5` is true for it, it flies through the tiles (`SetDefaults`
    /// case 23) and its rotation is the sprite-aim arm (`:50580-50592`).
    pub const METEOR_HEAD: Self = Self {
        max_speed: 1.0,
        accel: 0.03,
        surface_damped: false,
        double_add: true,
        no_tile_collide: true,
        bounce: 0.7,
        rotation: HoverRotation::SpriteAim,
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
    /// The target is dead (`NPC.cs:50299` — `flag = player.dead`): the fly
    /// core flips to the flee vector (`:50480-50484`).
    pub dead: bool,
}

/// The 8-px grid floor of the aim (`NPC.cs:50421-50424`):
/// `(int)(value / 8f) * 8`.
fn snap8(value: f32) -> f32 {
    ((value / 8.0) as i32 * 8) as f32
}

/// `AI_005_EaterOfSouls`'s fly core (`NPC.cs:50418-50579`, ahead of the
/// per-kind dash/attack arms): the desired velocity = the normalized target
/// vector × `num`; the velocity approaches it by `num2` per tick with the
/// `flag5`-gated double-add on the sign flips (`:50547-50578`). The type-42
/// family also takes the `num`/`num2` `2f - scale` factor (`:50350-50351`)
/// and the surface-gated ×0.97 vertical damping (`:50352-50362`), applied to
/// `velocity.Y` before the approach like the source.
///
/// The aim is quantised to the 8-px grid (`:50421-50424`) — the target's
/// centre and the NPC's own centre both floor to 8 px — and the dead-target
/// flee vector (`:50480-50484`) replaces it. The step ends with the
/// per-kind rotation arm (`:50580-50655`) and the tile collision: the
/// `noTileCollide` kinds fly straight through, the others bounce with
/// `num14` (`:50656-50688`).
pub fn hover_step(npc: &mut Npc, world: &World, target: HoverTarget, params: HoverParams) {
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
    // (a climb) or < 80 px below (a dive). Applied before the approach (`:50547`).
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
    // `:50418-50426` — the 8-px grid snap: the target's centre and the
    // NPC's own centre both floor to the 8-px grid, and the aim is the
    // difference of the snapped values.
    let mut aim_x = snap8(target.center_x) - snap8(cx);
    let mut aim_y = snap8(target.center_y) - snap8(cy);
    // `:50427-50444` — the normalized aim at the `num` speed; a zero
    // distance keeps the current velocity (`num4 = velocity.X`).
    let num6 = (aim_x * aim_x + aim_y * aim_y).sqrt();
    if num6 == 0.0 {
        aim_x = npc.vx;
        aim_y = npc.vy;
    } else {
        let scale = max_speed / num6;
        aim_x *= scale;
        aim_y *= scale;
    }
    // `:50480-50484` — a dead target: the NPC flies off along the facing
    // direction (`num`/`2` sideways, `-num`/`2` down).
    if target.dead {
        aim_x = npc.facing as f32 * max_speed / 2.0;
        aim_y = -max_speed / 2.0;
    }
    // `:50547-50578` — the approach (`num2`) with the `flag5` double-add
    // on the sign flips.
    if npc.vx < aim_x {
        npc.vx += accel;
        if params.double_add && npc.vx < 0.0 && aim_x > 0.0 {
            npc.vx += accel;
        }
    } else if npc.vx > aim_x {
        npc.vx -= accel;
        if params.double_add && npc.vx > 0.0 && aim_x < 0.0 {
            npc.vx -= accel;
        }
    }
    if npc.vy < aim_y {
        npc.vy += accel;
        if params.double_add && npc.vy < 0.0 && aim_y > 0.0 {
            npc.vy += accel;
        }
    } else if npc.vy > aim_y {
        npc.vy -= accel;
        if params.double_add && npc.vy > 0.0 && aim_y < 0.0 {
            npc.vy -= accel;
        }
    }
    // `:50580-50655` — the per-kind rotation arm.
    match params.rotation {
        HoverRotation::Aim => npc.rotation = aim_y.atan2(aim_x) - 1.57,
        HoverRotation::SpriteAim => {
            if aim_x > 0.0 {
                npc.sprite_dir = 1;
                npc.rotation = aim_y.atan2(aim_x);
            } else if aim_x < 0.0 {
                npc.sprite_dir = -1;
                npc.rotation = aim_y.atan2(aim_x) + 3.14;
            }
        }
        HoverRotation::VelocityX => {
            npc.sprite_dir = if npc.vx > 0.0 { 1 } else { -1 };
            npc.rotation = npc.vx * 0.1;
        }
        HoverRotation::Velocity => npc.rotation = npc.vy.atan2(npc.vx) - 1.57,
    }
    // `:50656-50688` — the tile collision. The `noTileCollide` kinds (the
    // Eater of Souls, the Meteor Head) fly straight through; the others
    // bounce off the wall with `num14` and the small-velocity clamps.
    let old_vx = npc.vx;
    let old_vy = npc.vy;
    if params.no_tile_collide {
        npc.x += npc.vx;
        npc.y += npc.vy;
    } else {
        let nx = npc.x + npc.vx;
        if !crate::player::collides_rect(world, nx, npc.y, w, h) {
            npc.x = nx;
        } else {
            npc.vx = old_vx * -params.bounce;
            if npc.facing == -1 && npc.vx > 0.0 && npc.vx < 2.0 {
                npc.vx = 2.0;
            }
            if npc.facing == 1 && npc.vx < 0.0 && npc.vx > -2.0 {
                npc.vx = -2.0;
            }
        }
        let ny = npc.y + npc.vy;
        if !crate::player::collides_rect(world, npc.x, ny, w, h) {
            npc.y = ny;
        } else {
            npc.vy = old_vy * -params.bounce;
            if npc.vy > 0.0 && npc.vy < 1.5 {
                npc.vy = 2.0;
            }
            if npc.vy < 0.0 && npc.vy > -1.5 {
                npc.vy = -2.0;
            }
        }
    }
    npc.facing = if target.center_x >= cx { 1 } else { -1 };
}

// S1-D-3: the aiStyle-8 caster family (`CasterParams`/`CasterPhase`/
// `caster_cycle`/`find_teleport_spot`) moved to `npc_ai_bat.rs` —
// `ai_style_8_step` + `attempt_find_teleport_spot`, the faithful
// `NPC.cs:20817-21376` / `:18937-19007` port (the deferred teleport, the
// dungeon-wall gate, the telefrag velocity union, the firing-distance
// rect and the shared `TargetClosest` all landed there).

// S1-D-3 reconcile: the aiStyle-16 swimmer framework (`SwimParams`/
// `swim_step`) is RETIRED — the faithful whole-arm port lives in
// `npc_ai_style16.rs` (`ai_016_step`/`ai_016_arm`, `NPC.cs:23492-24021`),
// wired through `npc.rs::update_swimmer`. The stand-in lacked the head
// `TargetClosest` (`:23494-23497`), the integer feet-slope division
// (`:23709-23736`), the collide-latch bounces (`:23737-23761`), the
// Arapaima/Orca/Dolphin/Pufferfish arms and the beach-avoid `ai[0]` writes
// (`:23932-23956`), and integrated inside the AI (the C# integrates in
// `UpdateCollision` AFTER the arm).

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
    fn s1_d_3_the_ladder_walker_accelerates_at_the_csharp_rate() {
        // S1-D-3 (speed table): the zombie (type 3) rides the num107
        // default arm (`NPC.cs:58322-58401`) — the 0.07 accel, not the
        // old direct `vx = facing * speed` write. Flat ground, no
        // obstacles: the velocity curve is the C#'s own.
        let w = ground();
        let mut npc = Npc::new(NpcKind::Zombie, 90.0 * 16.0, 118.0 * 16.0 - 40.0, 1, 0.0);
        npc.on_ground = true;
        let target = (60.0 * 16.0, 119.0 * 16.0);
        walker_step_ladder(
            &mut npc,
            &w,
            target.0,
            target.1,
            WalkerParams::ZOMBIE,
            0.4,
            10.0,
            false,
        );
        // Tick 1: vx 0 → -0.07 (the target is left), NOT -1.0.
        assert!((npc.vx - (-0.07)).abs() < 1e-6, "vx {}", npc.vx);
        for _ in 0..14 {
            walker_step_ladder(
                &mut npc,
                &w,
                target.0,
                target.1,
                WalkerParams::ZOMBIE,
                0.4,
                10.0,
                false,
            );
        }
        // Tick 15: the cap 1 (`:58324`) — 15 × 0.07 = 1.05 clamps at 1.
        assert!((npc.vx - (-1.0)).abs() < 1e-5, "vx {}", npc.vx);
        assert!(npc.x < 90.0 * 16.0, "the ladder walker still chases");
    }

    #[test]
    fn s1_d_3_the_ladder_walker_keeps_the_direct_write_for_declined_kinds() {
        // A non-aiStyle-3 kind (the DemonEye, style 2) keeps the old
        // direct write byte-for-byte — the ladder only claims style 3.
        let w = ground();
        let mut npc = Npc::new(NpcKind::DemonEye, 90.0 * 16.0, 118.0 * 16.0 - 40.0, 1, 0.0);
        npc.on_ground = true;
        let params = WalkerParams {
            speed: 2.0,
            jump: -6.2,
            aggro_x: 80.0 * 16.0,
            aggro_y: 40.0 * 16.0,
        };
        walker_step_ladder(&mut npc, &w, 60.0 * 16.0, 119.0 * 16.0, params, 0.4, 10.0, false);
        assert_eq!(npc.vx, -2.0, "the declined kind keeps the direct write");
        // The aggro window's outside decay is unchanged too.
        walker_step_ladder(&mut npc, &w, 5000.0, 119.0 * 16.0, params, 0.4, 10.0, false);
        assert_eq!(npc.vx, -2.0 * 0.9);
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


/// `AI_007_TownEntities`'s melee weapon presets (`NPC.cs:55004-55054`).
///
/// The `ai[0] == 15` swing state reads four per-type numbers: `num78` (the
/// swing damage, later scaled by `GetAttackDamage_ForTownNPC`), `num79` (the
/// swing knockback) and `num80`/`num81` (the swing's item width and height,
/// which `NPC.GetSwingStats` takes).
#[derive(Clone, Copy, Debug)]
pub struct TownWeaponParams {
    /// `num78` вЂ” the swing damage before the difficulty multiplier.
    pub damage: i32,
    /// `num79` вЂ” the swing knockback.
    pub knockback: f32,
    /// `num80` вЂ” the swing's item width (`GetSwingStats`'s `itemWidth`).
    pub width: i32,
    /// `num81` вЂ” the swing's item height (`GetSwingStats`'s `itemHeight`).
    pub height: i32,
}

impl TownWeaponParams {
    /// The Dye Trader's exotic scimitar (`NPC.cs:55018-55025`): the damage 11,
    /// the knockback 4.25 and the 32x32 scimitar.
    pub const DYE_TRADER: Self = Self {
        damage: 11,
        knockback: 4.25,
        width: 32,
        height: 32,
    };
    /// The Tax Collector's cane (`NPC.cs:55026-55038`): the damage 9, the
    /// knockback 3.5 and the 28x28 cane.
    pub const TAX_COLLECTOR: Self = Self {
        damage: 9,
        knockback: 3.5,
        width: 28,
        height: 28,
    };
    /// The Stylist's scissors (`NPC.cs:55039-55045`): the damage 10, the
    /// knockback 5 and the 32x32 scissors.
    pub const STYLIST: Self = Self {
        damage: 10,
        knockback: 5.0,
        width: 32,
        height: 32,
    };
    /// The town pets' shared row (`NPC.cs:55047-55053`): the damage 10, the
    /// knockback 3 and the 32x32 claw.
    pub const TOWN_PET: Self = Self {
        damage: 10,
        knockback: 3.0,
        width: 32,
        height: 32,
    };
}
