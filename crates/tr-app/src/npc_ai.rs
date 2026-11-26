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
    /// The jump velocity at a blocked path (`AI_003`'s `velocity.Y = -6f`
    /// for the humanoids, `NPC.cs:60125`; the skeleton family jumps the
    /// 3-tile walls). S1-D-3z: the old `-6.2` cite was wrong — no such
    /// constant exists in the C# arm.
    pub jump: f32,
    /// The aggro radius in pixels (`AI_003`'s 80-tile × 40-tile window for
    /// the zombie; the skeletons see further).
    pub aggro_x: f32,
    pub aggro_y: f32,
}

impl WalkerParams {
    /// The zombie's numbers (`AI_003_Fighters`' defaults, `NPC.cs:10586+`).
    /// S1-D-3z: the jump is the C#'s `-6f` (`NPC.cs:60125`), not -6.2. The
    /// 0.6 speed / 80×40 box are the legacy archetype stand-ins the
    /// test-only `walker_step` reads; the live zombie path rides the
    /// `num107` ladder + the despawn-discouragement gate instead.
    pub const ZOMBIE: Self = Self {
        speed: 0.6,
        jump: -6.0,
        aggro_x: 80.0 * TILE_PX,
        aggro_y: 40.0 * TILE_PX,
    };
    /// The town-NPC wander: the slower pace, no aggro (the direction flips
    /// on the aiTimer).
    pub const WANDER: Self = Self {
        speed: 0.5,
        jump: -6.0,
        aggro_x: 0.0,
        aggro_y: 0.0,
    };
}

/// The flyer tunables — the RETIRED v1 archetype's shape. W4-b2b: the
/// false `AI_002` citation is dead — neither 0.12 nor 3.0 appears in
/// `AI_002_FloatingEye` (`NPC.cs:52274-52731`); the faithful eye machine
/// is `npc_ai_eye::floating_eye_ai`, wired on the aiStyle-2 axis. These
/// fields exist only for the test-only `flyer_step` A/B pin.
#[derive(Clone, Copy, Debug)]
pub struct FlyerParams {
    /// The v1 stand-in's normalized-aim acceleration (INVENTED — not an
    /// `AI_002` constant; the real arm steers 0.1/0.04 per axis).
    pub accel: f32,
    /// The v1 stand-in's symmetric speed clamp (INVENTED — the real arm
    /// clamps ±4 X / ±1.5 Y × `(2 − scale)`).
    pub max_speed: f32,
}

impl FlyerParams {
    /// The RETIRED v1 eye stand-in's numbers (W4-b2b killed the false
    /// `AI_002_FloatingEye` citation — the A-1 audit, research/54 §2).
    /// Test-only: `the_flyer_homes_and_bounces` pins the legacy archetype;
    /// no live caller remains (`update_eye` retired with the wiring).
    #[allow(dead_code)] // test-only A/B pin (W4-b2b) — no live caller
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
        // W4-g4 lane 15 — the Confused debuff (31) reverses the walk
        // direction (`NPC.cs:77772-77775`, the SetTargetTrackingValues
        // tail the fighter core's TargetClosest runs; the generic
        // archetype's ad-hoc facing carries the same flip — the P3
        // buff-31 latch convention: `buffs.find_buff_index(31)`).
        if npc.buffs.find_buff_index(31).is_some() {
            npc.facing *= -1;
        }
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
/// swaps only the velocity write, never the choreography. The frost mob
/// wiring (`npc_ai_frost_mob.rs`, S1-D-3 W-6) reuses it at the AI_003
/// movement-tail position with the -6.2 jump stand-in.
pub(crate) fn walker_integrate(
    npc: &mut Npc,
    world: &World,
    params: WalkerParams,
    gravity: f32,
    max_fall: f32,
) {
    let (w, h) = (npc_size_w(npc), npc_size_h(npc));
    npc.vy = (npc.vy + gravity).min(max_fall);
    // R22 G4 (fix/verifier-w1): the generic `NPC.UpdateNPC` tail's ±0.005
    // X deadband (`NPC.cs:90473-90476`) — the compares are DOUBLE in the
    // C# (`(double)velocity.X < 0.005 && > -0.005`), zeroing the tiny
    // out-of-aggro `vx *= 0.9` decay tail instead of drifting forever.
    if (npc.vx as f64) < 0.005 && (npc.vx as f64) > -0.005 {
        npc.vx = 0.0;
    }

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
        // W4-g4 lane 15 — the Confused (31) walk-direction reversal
        // (`NPC.cs:77772-77775`), as `walker_step`'s aggro arm.
        if npc.buffs.find_buff_index(31).is_some() {
            npc.facing *= -1;
        }
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

/// `NPC.DespawnEncouragement_AIStyle3_Fighters_NotDiscouraged`
/// (`NPC.cs:60300-60330`) — the aiStyle-3 chase gate. The long `&&` chain
/// is the DAY-ABOVE-SURFACE discouragement test: when it holds, the
/// `switch` decides (the pillar/carrier cases and 528-532 stay encouraged,
/// everything else falls to `NPCID.Sets.CritterThatCanTurnOnPlayers`,
/// `NPCID.cs:10436`); when ANY chain term fails (night, underground,
/// statue-spawned, an event that claims the type, ...) the method returns
/// true — every fighter chases.
///
/// Stock stand-ins (documented, ANALYSIS "The shared target selection"):
/// `Main.dontStarveWorld` has no port field — it reads false, so its chain
/// term passes; the `Main.remixWorld` half of `IsItDay`
/// (`Main.cs:62825-62832`) is unported (`world.day_time` stands in).
/// `Main.player[target].ZoneGraveyard` rides the sim's pre-stamp
/// (`Npc::target_zone_graveyard` ← `scene_metrics.zone_graveyard`, W4-g4
/// lane 15, the W4-d22 reconcile fold — the d120 biome pass); false
/// outside the sim (the solo/test convention).
pub fn fighters_not_discouraged(npc: &Npc, world: &World) -> bool {
    let id = npc.kind.cs_id();
    // `Main.player[npcInstance.target].ZoneGraveyard` — the pre-stamp.
    let zone_graveyard = npc.target_zone_graveyard;
    let chain = !world.eclipse
        && world.day_time // `Main.IsItDay()` (`Main.cs:62825-62832`)
        && !npc.spawned_from_statue
        && !((npc.y as f64) > world.world_surface * 16.0)
        && !zone_graveyard
        && (!world.snow_moon || (id != 343 && id != 350))
        && (world.invasion_type != 1 || !matches!(id, 26 | 27 | 28 | 111 | 471))
        // `!Main.dontStarveWorld || (id != 164 && id != 163)` — the flag is
        // unported (false), the term passes.
        && id != 73
        && id != 624
        && (id != 631 || !(npc.ai[2] > 0.0))
        && (world.invasion_type != 3 || !(212..=216).contains(&id))
        && (world.invasion_type != 4
            || !matches!(id, 381 | 382 | 383 | 385 | 386 | 389 | 391 | 520))
        && !matches!(
            id,
            31 | 294
                | 295
                | 296
                | 47
                | 67
                | 77
                | 78
                | 79
                | 80
                | 630
                | 110
                | 120
                | 168
                | 181
                | 185
                | 198
                | 199
                | 206
                | 217
                | 218
                | 219
                | 220
                | 239
                | 243
                | 254
                | 255
                | 257
                | 258
                | 291
                | 292
                | 293
                | 379
                | 380
                | 464
                | 470
                | 424
        )
        && (id != 411 || (!(npc.ai[1] >= 180.0) && !(npc.ai[1] < 90.0)));
    if chain {
        // `:60304-60327` — the switch: the named cases break through to
        // `return true`; the default returns the critter set (528-532 skip
        // the return and break through too).
        match id {
            409 | 415 | 419 | 425 | 427 | 428 | 429 | 508 | 524 | 525 | 526 | 527
            | 580 | 582 | 528 | 529 | 530 | 532 => true,
            _ => crate::npc_ai3_speeds::TURN_CRITTERS.contains(&id),
        }
    } else {
        true
    }
}

/// S1-D-3p (walker migration) — [`walker_step_ladder`] with the C#'s
/// `AI_003_Fighters` target core replacing the invented 80×40-tile aggro
/// box: `TargetClosest()` at the C# position (`NPC.cs:57369`, under the
/// `ai[3] < num57` gate `:57336`/`:57035-57038` and
/// [`fighters_not_discouraged`]), the `:57370-57373` directionY bottom
/// re-check, then the SAME velocity write (the `npc_ai3_speeds` ladder,
/// the direct `facing * speed` fallback) and the SAME [`walker_integrate`]
/// choreography. The discouraged arm keeps the port's `vx *= 0.9` decay
/// (the `update_zombie` convention; the C# wander-flip arm `:57375-57401`
/// is the full AI_003 migration's job).
///
/// Equivalence (BEHAVIOUR FROZEN, tests-as-code below): inside the chase
/// gate, for integer-aligned positions the helper's int-centre facing
/// (`NPC.cs:77751-77755`) equals the old float-centre dx sign bit-for-bit
/// (every batch kind has an even width, the player box is 20×42), the
/// ladder/direct velocity write reads the same inputs, and neither path
/// draws RNG — the A/B twin-seed tests bit-compare facing/sprite_dir/
/// velocity/position and the stream position. The deliberate differences
/// are the C# fidelity fixes: the fractional-position truncation boundary
/// (`:77735`), the dropped distance box (the C# chases gated by the
/// despawn-discouragement arm, not a range window) and the dead-target
/// facing suppression (`:77736-77739`).
pub fn walker_step_fighter(
    npc: &mut Npc,
    world: &World,
    player: &crate::player::Player,
    params: WalkerParams,
    gravity: f32,
    max_fall: f32,
) {
    // The fighter core is `AI_003_Fighters`' — a kind of another style
    // riding this entry as a documented stand-in (the Present Mimic's
    // aiStyle-25 chest machine, `npc.rs`'s dispatch note) keeps the legacy
    // ladder step byte-for-byte until its own machine lands.
    if npc.kind.ai_style() != 3 {
        let (pcx, pcy) = player.center();
        walker_step_ladder(npc, world, pcx, pcy, params, gravity, max_fall, player.dead);
        return;
    }
    // `:57035-57038` — num57 (the 120 teleport arm's 180; no batch kind
    // is 120, the fold keeps the gate verbatim).
    let num57 = if npc.kind.cs_id() == 120 { 180.0 } else { 60.0 };
    // `:57336` — the chase gate.
    if npc.ai[3] < num57 && fighters_not_discouraged(npc, world) {
        // `:57369` — the shared selection (`npc_target.rs`, the
        // `update_zombie` wiring convention: one session player, stock
        // aggro/no-aggro/tank inputs, ticker 0 until the EverLastingTicker
        // plumb).
        let cand = crate::npc_target::session_candidate_for(player, npc.kind);
        crate::npc_target::target_closest(npc, world, &[cand], true, 3, 0);
        // BUG-07's lockstep write, as `walker_step` (`NPC.cs:19867`) and
        // the zombie wiring.
        npc.sprite_dir = npc.facing;
        // `:57370-57373` — `directionY > 0 && player.Center.Y <= Bottom.Y`
        // flips the vertical facing back up.
        let h = npc_size_h(npc);
        if npc.direction_y > 0 && player.center().1 <= npc.y + h {
            npc.direction_y = -1;
        }
        // The velocity write — byte-identical to `walker_step_ladder`'s
        // (the ladder at the C#'s post-target position, the direct-write
        // fallback for the declined kinds).
        let (pcx, pcy) = player.center();
        if !run_ai3_ladder(npc, world, pcx, pcy, gravity, player.dead) {
            npc.vx = npc.facing as f32 * params.speed;
        }
    } else {
        npc.vx *= 0.9;
    }
    walker_integrate(npc, world, params, gravity, max_fall);
}

/// The RETIRED v1 flyer archetype's fly-and-bob step (W4-b2b: was
/// false-cited as `AI_002_FloatingEye`, `NPC.cs:52274+` — the normalized
/// 0.12 aim, the ±3 clamp and the `sin(anim*3)*0.4` bob appear NOWHERE in
/// the C# arm; the faithful machine is `npc_ai_eye::floating_eye_ai`,
/// wired on the aiStyle-2 axis). Test-only: the last live caller
/// (`update_eye`) retired with the wiring; the archetype stays for the
/// A/B pin (`the_flyer_homes_and_bounces`).
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
    /// tiles. W4-d22: this gates the WRAPPER's integration tail (raw
    /// `position += velocity` vs the flyer collision), NOT the bounce arm
    /// — the C# bounce list (`:50656`) includes the noTileCollide kinds
    /// (6/23); their arm is inert because the raw integration never
    /// latches `collideX/Y`. The others bounce with `bounce`.
    pub no_tile_collide: bool,
    /// The tile bounce's restitution (`NPC.cs:50656-50688`): `num14` = 0.7
    /// for the family, 0.4 for the types 6/173.
    pub bounce: f32,
    /// The per-kind rotation arm (`NPC.cs:50580-50655`).
    pub rotation: HoverRotation,
    /// `flag4`'s ±0.023 `ai[0]` bob drift (`NPC.cs:50446-50473`) with the
    /// ALWAYS-true gate (`(num7 > 100f) | flag4`, `:50450` — `flag4` lists
    /// 619 at `:50446`). `true` only for the BloodSquid today; the other
    /// presets keep `false` (their drift is the hover-family wiring wave's
    /// — the bees run theirs kind-locally in `update_bee_hover`).
    pub bob_drift: bool,
    /// The <150px approach boost (`NPC.cs:50474-50478`): `velocity +=
    /// num4/num5 * 0.007` on the normalized aim when the snapped distance
    /// `num7 < 150f` — the C# gate lists {6, 94, 173, 619}; only the
    /// BloodSquid's preset carries it today.
    pub close_boost: bool,
    /// The BloodSquid's −0.3 rise (`NPC.cs:50485-50488`): the ELSE-if of
    /// the dead-target flee — `type == 619 && Center.Y > targetData.Center.Y
    /// - 200f` drops `velocity.Y` by 0.3 before the approach.
    pub hover_rise: bool,
    /// W4-d22: the `num8 | flag4` drift arm (`NPC.cs:50448-50478`). The
    /// Eater of Souls (6) takes it when the raw aim distance > 100; the
    /// Hornet (42) and the bees (210/211) take it every tick (`flag4`).
    pub drift: HoverDrift,
    /// W4-d22: the wet tail (`NPC.cs:50714-50725` for the types
    /// 6/94/173/619, `:50739-50753` for 42/176/231-235): the sinking
    /// ×0.95 damp, the lift and the −2/−4 cap. The Hornet row also
    /// re-picks the target (`:50752`) — the wrapper resolves it after
    /// the wet lift.
    pub wet: HoverWet,
    /// W4-d22: the per-tick dust gate (`NPC.cs:50656-50713`): the
    /// `Main.rand.Next(N) == 0` roll consumed EVERY tick, the dust type
    /// and the colour/scale the spawn carries. `None` for the kinds the
    /// gate excludes (the Hornet, the bees). The spawn's colour is
    /// `default(Color)` — the C# passes `this.color` (the NPC field is
    /// never written on these kinds, `NPC.cs:6489`) or the literal
    /// default; both are the all-zero XNA colour. The scale is the
    /// NPC's `scale`.
    pub dust: Option<HoverDust>,
    /// W4-d22: the day/dead flee exemption (`NPC.cs:50906`): the
    /// `Main.IsItDay()` term is gated OFF for the listed kinds, the dead
    /// term fires for everyone. The servant (5) is NOT on the list — it
    /// flees at dawn (`velocity.Y -= 0.06`); the Meteor Head (23) IS.
    pub dawn_exempt: bool,
}

/// W4-d22: the `num8 | flag4` drift arm (`NPC.cs:50448-50478`) — the
/// `ai[0]` counter's ±0.023 nudges on the velocity, ahead of the approach.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoverDrift {
    /// Outside both sets — no drift (the servant, the Meteor Head).
    None,
    /// `num8` (the Eater of Souls, 6): the arm runs only when the raw aim
    /// distance `num7 > 100` (`:50450`), and the near pass (`num7 < 150`,
    /// `:50474-50478`) blends the velocity toward the aim by ×0.007.
    EaterOfSouls,
    /// `flag4` (the Hornet 42, the bees 210/211): the arm runs every tick
    /// (`(num7 > 100) | flag4` is unconditionally true); no near pass (the
    /// `:50474` type list holds only 6/94/173/619).
    Flag4,
}

/// W4-d22: the wet tail rows (`NPC.cs:50714-50753`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoverWet {
    /// No wet arm (the servant, the Meteor Head).
    None,
    /// `:50714-50725` — the Eater of Souls row: `velocity.Y > 0` damps
    /// ×0.95, then −0.3 lift, cap −2.
    EaterOfSouls,
    /// `:50739-50753` — the Hornet row: ×0.95, −0.5 lift, cap −4, and a
    /// `TargetClosest()` re-pick while wet.
    Hornet,
}

/// W4-d22: the per-tick dust gate row (`NPC.cs:50695-50712`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoverDust {
    /// The gate's `Main.rand.Next(N)` bound (20 for the bounce-list kinds,
    /// 40 for the else arm).
    pub gate: i32,
    /// The `Dust.NewDust` type (`num16`/the literal 5): 18 for the Eater
    /// of Souls / Meteor Head, 5 for the servant.
    pub dust_type: i32,
    /// The spawn's scale: the bounce-list arm passes `this.scale`
    /// (`:50702`), the else arm the stock 1.0 (`:50710`).
    pub use_npc_scale: bool,
}

impl HoverParams {
    /// The Eater of Souls (type 6; `NPC.cs:50304-50307`: num 4, num2 0.02).
    /// `flag5` is false for it (`:50447`) and it flies through the tiles
    /// (`SetDefaults` case 6: `noTileCollide`), so it never bounces; the
    /// rotation is the aim tilt (`:50643-50646`). The num8 bob (`:50445`,
    /// the >100px gate) and the type-6 <150px boost (`:50474`) stay OFF
    /// here — the EoS behaviour is frozen; they ride the hover-family
    /// wiring wave (the bee precedent's drift is bee-local).
    pub const EATER_OF_SOULS: Self = Self {
        max_speed: 4.0,
        accel: 0.02,
        surface_damped: false,
        double_add: false,
        no_tile_collide: true,
        bounce: 0.4,
        rotation: HoverRotation::Aim,
        bob_drift: false,
        close_boost: false,
        hover_rise: false,
        // W4-d22: `num8` holds type 6 (`:50445`) — the >100 drift + the
        // <150 near blend; the wet row `:50714`; the `:50695` Next(20)
        // gate spawns type-18 dust at `this.color`/`this.scale`; type 6
        // IS on the dawn-flee exemption list (`:50906`).
        drift: HoverDrift::EaterOfSouls,
        wet: HoverWet::EaterOfSouls,
        dust: Some(HoverDust {
            gate: 20,
            dust_type: 18,
            use_npc_scale: true,
        }),
        dawn_exempt: true,
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
        bob_drift: false,
        close_boost: false,
        hover_rise: false,
        // W4-d22: `flag4` holds type 42 (`:50446`) — the every-tick drift;
        // the wet row `:50741-50753` (the TargetClosest re-pick rides the
        // wrapper); the `:50695` dust gate EXCLUDES type 42 (no dust);
        // type 42 is on the dawn-flee exemption list (`:50906`).
        drift: HoverDrift::Flag4,
        wet: HoverWet::Hornet,
        dust: None,
        dawn_exempt: true,
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
        bob_drift: false,
        close_boost: false,
        hover_rise: false,
        // W4-d22: type 5 is in neither drift set nor any wet row; it is
        // NOT on the `:50656` bounce list, so it takes the ELSE dust arm
        // (`:50708`): Next(40), type-5 dust, default colour/scale. It is
        // NOT on the dawn-flee exemption list (`:50906`) — the servant
        // climbs away at dawn (`velocity.Y -= 0.06`).
        drift: HoverDrift::None,
        wet: HoverWet::None,
        dust: Some(HoverDust {
            gate: 40,
            dust_type: 5,
            use_npc_scale: false,
        }),
        dawn_exempt: false,
    };
    /// W4-d36 — the Crimera (173, `NPC.cs:50304-50307` shares the Eater of
    /// Souls' num 4 / num2 0.02): flag5 false (`:50447` lists 173), the
    /// ×0.4 bounce (`:50659-50661`), the aim-tilt rotation (`:50643`).
    /// FIX vs the in-flight stash: `SetDefaults` case 173 (`:11168-11193`)
    /// writes `noGravity` ONLY — no `noTileCollide` row (the head default
    /// false, `:8522`) — so the Crimera COLLIDES; the ×0.4 restitution
    /// arm exists precisely for the colliding 6/173 pair. (The base
    /// EATER_OF_SOULS row's `no_tile_collide: true` claim predates this
    /// audit — left untouched, recorded for the Lead.)
    pub const CRIMERA: Self = Self {
        max_speed: 4.0,
        accel: 0.02,
        surface_damped: false,
        double_add: false,
        no_tile_collide: false,
        bounce: 0.4,
        rotation: HoverRotation::Aim,
        bob_drift: false,
        close_boost: false,
        hover_rise: false,
        drift: HoverDrift::None,
        wet: HoverWet::None,
        dust: None,
        dawn_exempt: false,
    };
    /// W4-d36 — the Moss Hornet (176, `NPC.cs:50369-50372`: num 4 / num2
    /// 0.017): flag5 TRUE — FIX vs the in-flight stash, which carried
    /// false: the `:50447` exclusion list is {173, 6, 42, 231-235, 94,
    /// 139, 619} and 176 is NOT in it. It collides (`SetDefaults` case
    /// 176 sets `noGravity` only, `:11235`), the ×0.7 family bounce
    /// (`:50658`), the velocity-X rotation (`:50647-50650`).
    pub const MOSS_HORNET: Self = Self {
        max_speed: 4.0,
        accel: 0.017,
        surface_damped: false,
        double_add: true,
        no_tile_collide: false,
        bounce: 0.7,
        rotation: HoverRotation::VelocityX,
        bob_drift: false,
        close_boost: false,
        hover_rise: false,
        drift: HoverDrift::None,
        wet: HoverWet::None,
        dust: None,
        dawn_exempt: false,
    };
    /// W4-d36 — the Moth (205, `NPC.cs:50364-50367`: num 3.25 / num2
    /// 0.018): flag5 true (outside the `:50447` exclusion set), the ×0.7
    /// bounce (`SetDefaults` case 205 sets `noGravity` only, `:11637`),
    /// the velocity-X rotation (`:50647-50650`).
    pub const MOTH: Self = Self {
        max_speed: 3.25,
        accel: 0.018,
        surface_damped: false,
        double_add: true,
        no_tile_collide: false,
        bounce: 0.7,
        rotation: HoverRotation::VelocityX,
        bob_drift: false,
        close_boost: false,
        hover_rise: false,
        drift: HoverDrift::None,
        wet: HoverWet::None,
        dust: None,
        dawn_exempt: false,
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
        bob_drift: false,
        close_boost: false,
        hover_rise: false,
        drift: HoverDrift::None,
        wet: HoverWet::None,
        dust: Some(HoverDust {
            gate: 20,
            dust_type: 18,
            use_npc_scale: true,
        }),
        dawn_exempt: true,
    };
    /// W4-d42 - the Blood Squid (type 619; `NPC.cs:50323-50327`:
    /// num 6, num2 0.1). `flag5` is false for it (`:50447` lists
    /// 619 in the excluded set), it COLLIDES with the tiles
    /// (`SetDefaults` 619 writes noGravity only, `:16945`) so it
    /// takes the x0.7 bounce (`:50658`), and its rotation is the
    /// aim tilt (`:50643-50646` lists 619). W4-d42 FINAL: the flag4
    /// bob (`:50446-50473`, the ALWAYS-true `(num7 > 100f) | flag4`
    /// gate `:50450`), the <150px approach boost (`:50474-50478`,
    /// 619 in the {6, 94, 173, 619} set) and the -0.3 rise
    /// (`:50485-50488`, the dead-flee's else-if) are now modelled —
    /// the ANALYSIS roster-join gap CLOSED.
    pub const BLOOD_SQUID: Self = Self {
        max_speed: 6.0,
        accel: 0.1,
        surface_damped: false,
        double_add: false,
        no_tile_collide: false,
        bounce: 0.7,
        rotation: HoverRotation::Aim,
        bob_drift: true,
        close_boost: true,
        hover_rise: true,
        drift: HoverDrift::None,
        wet: HoverWet::None,
        dust: None,
        dawn_exempt: true,
    };
    /// W4-d37 — the HornetFatty (type 231; `NPC.cs:50345-50348`: num 3,
    /// num2 0.017 — the type-231 override of the 42/231-235 family arm).
    /// The family's `2f - scale` factor + surface damping (`:50350-50362`),
    /// `flag5` false (`:50447` excludes 231-235), tile collision with the
    /// 0.7 bounce (`:50656`), the `velocity.X * 0.1` rotation (`:50647`).
    /// The stinger arm (`:50739+`) is the documented stand-in skip — the
    /// base Hornet's hover precedent.
    pub const HORNET_FATTY: Self = Self {
        max_speed: 3.0,
        accel: 0.017,
        surface_damped: true,
        double_add: false,
        no_tile_collide: false,
        bounce: 0.7,
        rotation: HoverRotation::VelocityX,
        bob_drift: false,
        close_boost: false,
        hover_rise: false,
        drift: HoverDrift::None,
        wet: HoverWet::None,
        dust: None,
        dawn_exempt: true,
    };
    /// W4-d37 — the HornetHoney/Leafy/Spikey/Stingy (types 232-235;
    /// `NPC.cs:50341-50344`: the family base num 3.5, num2 0.021 — no
    /// per-type override). The same family gates as HORNET_FATTY.
    pub const HORNET_VARIANT: Self = Self {
        max_speed: 3.5,
        accel: 0.021,
        surface_damped: true,
        double_add: false,
        no_tile_collide: false,
        bounce: 0.7,
        rotation: HoverRotation::VelocityX,
        bob_drift: false,
        close_boost: false,
        hover_rise: false,
        drift: HoverDrift::None,
        wet: HoverWet::None,
        dust: None,
        dawn_exempt: true,
    };
    /// W4-d37 — the Parrot (type 252; `NPC.cs:50328-50339`): num 6 /
    /// num2 0.1 when the target is hittable, else num 2 / num2 0.01. The
    /// hover stand-in pins the chase pair (the CanHit-gated idle pair is
    /// the documented remainder — the base Hornet's stinger-skip shape).
    /// `flag5` true (`:50447` — 252 is outside the excluded set), tile
    /// collision with the 0.7 bounce (`:50656` — 252 is NOT in the
    /// noTileCollide list), the else-arm velocity tilt (`:50652-50655`).
    pub const PARROT: Self = Self {
        max_speed: 6.0,
        accel: 0.1,
        surface_damped: false,
        double_add: true,
        no_tile_collide: false,
        bounce: 0.7,
        rotation: HoverRotation::Velocity,
        bob_drift: false,
        close_boost: false,
        hover_rise: false,
        // W4-d22: type 23 is in neither drift set nor any wet row; it IS
        // on the `:50656` bounce list, so the `:50695` Next(20) gate
        // spawns type-18 dust at `this.color`/`this.scale`; type 23 is on
        // the dawn-flee exemption list (`:50906`).
        drift: HoverDrift::None,
        wet: HoverWet::None,
        dust: Some(HoverDust {
            gate: 20,
            dust_type: 18,
            use_npc_scale: true,
        }),
        dawn_exempt: true,
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
pub(crate) fn snap8(value: f32) -> f32 {
    ((value / 8.0) as i32 * 8) as f32
}

/// `AI_005_EaterOfSouls`'s fly core (`NPC.cs:50418-50914`) — the AI-only
/// step: the desired velocity = the normalized target vector × `num`;
/// the `num8 | flag4` drift (`:50448-50478`); the velocity approaches the
/// aim by `num2` per tick with the `flag5`-gated double-add on the sign
/// flips (`:50547-50578`); the per-kind rotation (`:50580-50655`); the
/// tile bounce off the PREVIOUS tick's `collideX`/`collideY` latches with
/// `num14` (`:50656-50688`, the `npc_ai_eye` oldVelocity precedent); the
/// wet tail (`:50714-50753`); the day/dead flee (`:50906-50910`) and the
/// velocity-flip netUpdate tail (`:50911-50914`).
///
/// W4-d22: the step no longer integrates — the C# integrates in
/// `UpdateCollision` AFTER `AI()` (`NPC.cs:90490-90498`); the wrapper
/// (`npc.rs::update_hover`) owns the integration tail
/// (`integrate_flyer_collision` / `integrate_no_tile_collide`), the
/// dust/wet half ([`hover_dust_wet`]), the Hornet's stinger ladder
/// (`:50754-50810`) and the flee/netUpdate half ([`hover_flee`]). The
/// bounce reads `npc.old_vx`/`old_vy` (LAST tick's pre-collision velocity,
/// captured by the integration tail at `NPC.cs:92538`) and the
/// `collide_x`/`collide_y` latches it set — NOT an inline probe (the
/// pre-W4-d22 stand-in bounced off a same-tick wall test with a mid-step
/// velocity capture).
///
/// The aim is quantised to the 8-px grid (`:50421-50424`) — the target's
/// centre and the NPC's own centre both floor to 8 px — and the dead-target
/// flee vector (`:50480-50484`) replaces it AFTER the drift (the C# order:
/// normalize `:50427-50444` → drift `:50448` → flee `:50480` → approach).
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
    // distance keeps the current velocity (`num4 = velocity.X`). `num7`
    // (`:50428`) keeps the PRE-normalize snapped distance for the
    // bob/boost gates.
    // distance keeps the current velocity (`num4 = velocity.X`) — read
    // BEFORE the drift, exactly the C# order.
    let num6 = (aim_x * aim_x + aim_y * aim_y).sqrt();
    let num7 = num6;
    if num6 == 0.0 {
        aim_x = npc.vx;
        aim_y = npc.vy;
    } else {
        let scale = max_speed / num6;
        aim_x *= scale;
        aim_y *= scale;
    }
    // `:50445-50478` — the bob + boost block, gated `num8 | flag4`. The
    // only preset carrying it today is the BloodSquid (619, `flag4`
    // `:50446`), so the inner `(num7 > 100f) | flag4` gate (`:50450`) is
    // ALWAYS true — the drift runs every tick. The num8 kinds' >100px
    // gate rides the hover-family wiring wave (the bee precedent's drift
    // is bee-local in `update_bee_hover`).
    if params.bob_drift {
        // `:50452-50472` — ai[0]++ then the ±0.023 drift: Y by the ai[0]
        // sign, X by the ±100 band, the 200 wrap to -200.
        npc.ai[0] += 1.0;
        if npc.ai[0] > 0.0 {
            npc.vy += 0.023;
        } else {
            npc.vy -= 0.023;
        }
        if npc.ai[0] < -100.0 || npc.ai[0] > 100.0 {
            npc.vx += 0.023;
        } else {
            npc.vx -= 0.023;
        }
        if npc.ai[0] > 200.0 {
            npc.ai[0] = -200.0;
        }
        // `:50474-50478` — the <150px approach boost on the NORMALIZED
        // aim ({6, 94, 173, 619}; the BloodSquid's preset carries it).
        if params.close_boost && num7 < 150.0 {
    // `:50445-50478` — the `num8 | flag4` drift: the ai[0] counter's
    // ±0.023 nudges (Y by the counter's sign, X by the ±100 band, the 200
    // wrap), gated on `num7 > 100` for the num8 kinds and unconditional
    // for flag4; then the num8 kinds' <150 near blend (×0.007 of the
    // normalized aim). The bees' old stand-in ran the drift BEFORE the
    // normalize (a zero-distance deviation) — the C# order is restored.
    let drift_runs = match params.drift {
        HoverDrift::None => false,
        HoverDrift::EaterOfSouls => num7 > 100.0,
        HoverDrift::Flag4 => true,
    };
    if params.drift != HoverDrift::None {
        if drift_runs {
            npc.ai[0] += 1.0;
            if npc.ai[0] > 0.0 {
                npc.vy += 0.023;
            } else {
                npc.vy -= 0.023;
            }
            if npc.ai[0] < -100.0 || npc.ai[0] > 100.0 {
                npc.vx += 0.023;
            } else {
                npc.vx -= 0.023;
            }
            if npc.ai[0] > 200.0 {
                npc.ai[0] = -200.0;
            }
        }
        // `:50474-50478` — the near blend (the num8 type list 6/94/173/619).
        if params.drift == HoverDrift::EaterOfSouls && num7 < 150.0 {
            npc.vx += aim_x * 0.007;
            npc.vy += aim_y * 0.007;
        }
    }
    // `:50480-50484` — a dead target: the NPC flies off along the facing
    // direction (`num`/`2` sideways, `-num`/`2` up).
    if target.dead {
        aim_x = npc.facing as f32 * max_speed / 2.0;
        aim_y = -max_speed / 2.0;
    } else if params.hover_rise && cy > target.center_y - 200.0 {
        // `:50485-50488` — the BloodSquid's ELSE-if rise: while its centre
        // is below `targetData.Center.Y - 200f` it sheds 0.3 of upward
        // velocity per tick, BEFORE the approach (the flee's else branch).
        npc.vy -= 0.3;
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
    // `:50656-50688` — the tile bounce, gated on the kind list (the
    // ported kinds: 6/23/42/210/211). It reads the PREVIOUS tick's
    // `collideX`/`collideY` latches and `oldVelocity` (`NPC.cs:92538`'s
    // capture, the port's integration tail) — the `npc_ai_eye` bounce
    // precedent. `num14` = 0.4 for the types 6/173, 0.7 for the rest.
    // The noTileCollide kinds (6/23) never latch, so the arm is inert for
    // them — the C# shape kept verbatim.
    if npc.collide_x {
        npc.net_update = true;
        npc.vx = npc.old_vx * -params.bounce;
        if npc.facing == -1 && npc.vx > 0.0 && npc.vx < 2.0 {
            npc.vx = 2.0;
        }
        if npc.facing == 1 && npc.vx < 0.0 && npc.vx > -2.0 {
            npc.vx = -2.0;
        }
    }
    if npc.collide_y {
        npc.net_update = true;
        npc.vy = npc.old_vy * -params.bounce;
        if npc.vy > 0.0 && npc.vy < 1.5 {
            npc.vy = 2.0;
        }
        if npc.vy < 0.0 && npc.vy > -1.5 {
            npc.vy = -2.0;
        }
    }
}
}
}

/// W4-d22: `AI_005`'s dust gate + wet lift (`NPC.cs:50690-50753`) — the
/// first half of the post-bounce tail. Split from the flee/netUpdate half
/// ([`hover_flee`]) so the wrapper can interleave the Hornet's stinger
/// ladder (`:50754-50810`, between the wet lift and the flee) in the C#
/// order. `dust`/`dust_ctx`/`rng` ride the `UpdateNPCs` stream.
pub fn hover_dust_wet(
    npc: &mut Npc,
    world: &World,
    params: HoverParams,
    rng: &mut UnifiedRandom,
    dust: &mut tr_core::particles::DustPool,
    dust_ctx: &tr_core::particles::SpawnCtx,
) {
    let (w, h) = (npc_size_w(npc), npc_size_h(npc));
    // `:50690-50713` — the per-tick dust gate. The bounce-list kinds
    // (6/23) roll `Next(20)`, the else arm (the servant) `Next(40)`; on 0
    // a dust spawns at (x, y + h*0.25) over width × (int)(h*0.5) at the
    // NPC's velocity (speedY literal 2), alpha 75, then velocity.X *= 0.5
    // / velocity.Y *= 0.1. The colour is `default(Color)` (`this.color`
    // is never written on these kinds); the scale is `this.scale` on the
    // bounce-list arm, the stock 1.0 on the else arm. The netOffset round
    // trip (`:50689`/`:50706`) is zero in single-player — dropped.
    if let Some(row) = params.dust {
        if rng.next_max(row.gate) == 0 {
            let scale = if row.use_npc_scale { npc.scale } else { 1.0 };
            let idx = dust.new_dust(
                rng,
                dust_ctx,
                world,
                glam::Vec2::new(npc.x, npc.y + h * 0.25),
                w as i32,
                (h * 0.5) as i32,
                row.dust_type,
                npc.vx,
                2.0,
                75,
                tr_core::particles::Color::default(),
                scale,
            );
            if idx != tr_core::particles::DUST_SENTINEL {
                let d = dust.slot_mut(idx);
                d.velocity.x *= 0.5;
                d.velocity.y *= 0.1;
            }
        }
    }
    // `:50714-50753` — the wet lift. The Eater of Souls row: the sinking
    // ×0.95 damp, −0.3 lift, −2 cap. The Hornet row: ×0.95, −0.5 lift, −4
    // cap (the TargetClosest re-pick rides the wrapper — the solo fold:
    // one player, `targetData` captured at `:50293` before the re-pick).
    match params.wet {
        HoverWet::None => {}
        HoverWet::EaterOfSouls => {
            if npc.wet {
                if npc.vy > 0.0 {
                    npc.vy *= 0.95;
                }
                npc.vy -= 0.3;
                if npc.vy < -2.0 {
                    npc.vy = -2.0;
                }
            }
        }
        HoverWet::Hornet => {
            if npc.wet {
                if npc.vy > 0.0 {
                    npc.vy *= 0.95;
                }
                npc.vy -= 0.5;
                if npc.vy < -4.0 {
                    npc.vy = -4.0;
                }
            }
        }
    }
}

/// W4-d22: `AI_005`'s flee + netUpdate tail (`NPC.cs:50906-50914`) — the
/// second half of the post-bounce tail, after the dust/wet half
/// ([`hover_dust_wet`]) and (for the Hornet) the stinger ladder.
pub fn hover_flee(npc: &mut Npc, world: &World, target: HoverTarget, params: HoverParams) {
    // `:50906-50910` — the day/dead flee: `(IsItDay() && !exempt) | dead`
    // → `velocity.Y -= num2 * 2` + `EncourageDespawn(10)`. The exemption
    // list holds 6/23/42/210/211 (every ported kind but the servant — the
    // servant climbs away at dawn, `num2 * 2 = 0.06`). `IsItDay()` folds
    // to `day_time` (remixWorld is constants-off, the classic solo env).
    let accel = if params.surface_damped {
        params.accel * (2.0 - npc.scale)
    } else {
        params.accel
    };
    if (world.day_time && !params.dawn_exempt) || target.dead {
        npc.vy -= accel * 2.0;
        npc.encourage_despawn(10);
    }
    // `:50911-50914` — the velocity sign-flip netUpdate tail (vs the
    // previous tick's oldVelocity, gated on !justHit).
    if ((npc.vx > 0.0 && npc.old_vx < 0.0)
        || (npc.vx < 0.0 && npc.old_vx > 0.0)
        || (npc.vy > 0.0 && npc.old_vy < 0.0)
        || (npc.vy < 0.0 && npc.old_vy > 0.0))
        && !npc.just_hit
    {
        npc.net_update = true;
    }
}

/// W4-f4 — `AI_005_EaterOfSouls`'s dust gate (`NPC.cs:50656-50713`,
/// sites 50702/50710). The `:50656` kind list splits the arms:
///
/// * the bounce-list kinds (6/619/23/42/94/139/173/176/205/210/211/
///   231-235) roll `Next(20) == 0` (`:50695`) UNLESS the type is in the
///   exclusion list 42/139/176/205/210/211/252/231-235 — of the ported
///   kinds the Eater of Souls (6) and the Meteor Head (23) draw, the
///   Hornet (42), the Probe (139) and the volley bees (210/211) draw
///   NOTHING (the `&&` short-circuits before `Main.rand`). On 0 a dust
///   spawns at `(x, y + h*0.25)` over `width × (int)(h*0.5)`, type 18
///   (the `num16` default; the 173→5 override has no ported kind), the
///   NPC's `velocity.X` and the literal 2f, alpha 75, `this.color`,
///   `this.scale` (site 50702);
/// * the else arm (the servant 5 — outside the `:50656` list) rolls
///   `Next(40) == 0` (`:50708`, gated `type != 252`): type 5, the same
///   box/velocity pair, the DEFAULT alpha 0 / colour / scale 1f (site
///   50710 — the 7-arg overload, `Dust.cs:75`).
///
/// Both arms post `velocity.X *= 0.5` / `velocity.Y *= 0.1`. The colour
/// census: NO hover kind ever writes `NPC.color` (the `SetDefaults`
/// colour rows are the slimes -3..-10/1/16/537 and the rainbow slime's
/// `AI_001_SetRainbowSlimeColor` only, `NPC.cs:7685-7753`/`:8583`/
/// `:8975`/`:15869`/`:61727`) — `this.color` reads `default(Color)`
/// (`:8526`), so the port passes `Color::default()` and models no
/// colour field (the d22 census, cross-noted).
///
/// The gate draw rides the caller's `UpdateNPCs` stream EVERY tick the
/// kind qualifies; the pool internals roll inside `new_dust` (types
/// 18/5 hit no per-type tweak, `Dust.cs:214-252`). `bounced` = the
/// `:50656` list membership; `excluded` = the `:50695` exclusion (only
/// meaningful when `bounced`). The netOffset round trip
/// (`:50689`/`:50706`) is zero single-player — dropped. The C# runs the
/// gate AFTER the bounce and BEFORE the integration (`:90498`); the
/// port's `hover_step` integrates inside, so the gate reads the
/// post-move box (a one-tick position deviation of the stand-in; the
/// velocity is the fresh post-approach value — exact). The d22
/// fidelity wave owns the AI/integration split (ANALYSIS).
pub fn hover_dust_gate(
    npc: &Npc,
    world: &World,
    bounced: bool,
    excluded: bool,
    rng: &mut UnifiedRandom,
    dust: &mut tr_core::particles::DustPool,
    dust_ctx: &tr_core::particles::SpawnCtx,
) {
    let (w, h) = (npc_size_w(npc), npc_size_h(npc));
    let pos = glam::Vec2::new(npc.x, npc.y + h * 0.25);
    let (bw, bh) = (w as i32, (h * 0.5) as i32);
    let idx = if bounced {
        if excluded {
            return; // `:50695`'s exclusion list — the gate never draws
        }
        // `:50695-50705` — the Next(20) gate, site 50702.
        if rng.next_max(20) != 0 {
            return;
        }
        dust.new_dust(
            rng, dust_ctx, world, pos, bw, bh, 18,
            npc.vx, 2.0, 75,
            tr_core::particles::Color::default(), npc.scale,
        )
    } else {
        // `:50708-50712` — the Next(40) gate, site 50710 (the 7-arg
        // overload: alpha 0, default colour, scale 1f).
        if rng.next_max(40) != 0 {
            return;
        }
        dust.new_dust(
            rng, dust_ctx, world, pos, bw, bh, 5,
            npc.vx, 2.0, 0,
            tr_core::particles::Color::default(), 1.0,
        )
    };
    let dd = dust.slot_mut(idx);
    dd.velocity.x *= 0.5;
    dd.velocity.y *= 0.1;
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
    fn r22_g4_the_walker_deadband_zeroes_the_decay_tail() {
        // R22 G4 (`NPC.cs:90473-90476`): the generic UpdateNPC tail
        // zeroes velocity.X inside the ±0.005 DOUBLE deadband — the
        // out-of-aggro `vx *= 0.9` tail stops dead instead of drifting
        // forever. WRITTEN NOT RUN (code-only).
        let w = ground();
        let mut npc = Npc::new(NpcKind::Zombie, 90.0 * 16.0, 118.0 * 16.0 - 40.0, 1, 0.0);
        npc.on_ground = true;
        // The target outside the aggro window: the decay arm runs.
        let far = (190.0 * 16.0, 119.0 * 16.0);
        npc.vx = 0.005; // the f32 literal widens just BELOW 0.005 in f64
        walker_step(&mut npc, &w, far.0, far.1, WalkerParams::ZOMBIE, 0.4, 10.0);
        // 0.005 * 0.9 = 0.0045 < 0.005 -> zeroed (the C# float->double
        // widen of 0.005f lands below the literal too — same fold).
        assert_eq!(npc.vx, 0.0, "the decay tail dies in the deadband");
        // The negative side mirrors (the compare is `> -0.005`).
        npc.vx = -0.005;
        walker_step(&mut npc, &w, far.0, far.1, WalkerParams::ZOMBIE, 0.4, 10.0);
        assert_eq!(npc.vx, 0.0, "the negative tail dies too");
        // Above the band the velocity survives (0.02 * 0.9 = 0.018).
        npc.vx = 0.02;
        walker_step(&mut npc, &w, far.0, far.1, WalkerParams::ZOMBIE, 0.4, 10.0);
        assert!(npc.vx > 0.005, "outside the band the decay continues");
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

    // ---- S1-D-3p (walker migration) — the A/B equivalence proof ----
    //
    // Code-only mode: tests-as-code, NOT run in this task (no compiler
    // invoked). The old path is `walker_step`/`walker_step_ladder` with
    // the invented 80×40-tile aggro box; the new path is
    // `walker_step_fighter` with `TargetClosest` at the C# position
    // (`NPC.cs:57369`). Twin seeds drive both paths tick-for-tick and
    // BIT-compare facing/sprite_dir/target + the velocity/position trace;
    // a `UnifiedRandom` pair (PartialEq over the full ring state) proves
    // the stream position is untouched — neither path draws (`:57336-
    // 57402`'s core is RNG-free; the sound rolls `:57340-57367` are the
    // unported audio layer).

    /// The night ground: the chase gate (`NPC.cs:57336`,
    /// `:60300-60330`) passes for every fighter at night, so the A/B's
    /// equivalence domain is the whole arena.
    fn night_ground() -> World {
        let mut w = ground();
        w.day_time = false;
        w
    }

    /// A parked session player at (x, y) — the stock S1-D-3p inputs
    /// (aggro 0 / no-aggro false / tank pet none) make the helper's pick
    /// the plain nearest-slot form, slot 0.
    fn parked_player(w: &World, x: f32, y: f32) -> crate::player::Player {
        let mut p = crate::player::Player::spawn(w);
        p.x = x;
        p.y = y;
        p
    }

    /// The bit-compare of one step's outputs (the f32 fields compare by
    /// their bit patterns — the C#-float-op replay convention).
    fn assert_step_eq(old: &Npc, new: &Npc, tick: usize) {
        assert_eq!(old.facing, new.facing, "tick {tick}: facing");
        assert_eq!(old.sprite_dir, new.sprite_dir, "tick {tick}: sprite_dir");
        assert_eq!(old.vx.to_bits(), new.vx.to_bits(), "tick {tick}: vx");
        assert_eq!(old.vy.to_bits(), new.vy.to_bits(), "tick {tick}: vy");
        assert_eq!(old.x.to_bits(), new.x.to_bits(), "tick {tick}: x");
        assert_eq!(old.y.to_bits(), new.y.to_bits(), "tick {tick}: y");
        assert_eq!(old.on_ground, new.on_ground, "tick {tick}: on_ground");
        assert_eq!(old.ai, new.ai, "tick {tick}: ai");
        assert_eq!(
            old.damage_override, new.damage_override,
            "tick {tick}: damage_override"
        );
    }

    /// The twin-seed A/B over one kind: the old ladder step (the 80×40
    /// box, the ad-hoc dx-sign facing) vs the new fighter step, 240
    /// ticks on the flat night ground, the player parked 30 tiles left.
    /// Every kind's width is even, so the helper's int-centre facing
    /// (`NPC.cs:77751-77755`) equals the old float-centre dx sign on the
    /// integer-aligned fixture; the ladder/direct velocity write reads
    /// the same inputs on both paths.
    fn ab_fighter_kind(kind: NpcKind, speed: f32, ladder: bool) {
        let w = night_ground();
        let player = parked_player(&w, 60.0 * 16.0, 118.0 * 16.0 - 42.0);
        let (pcx, pcy) = player.center();
        let params = WalkerParams {
            speed,
            jump: -6.2,
            aggro_x: 80.0 * 16.0,
            aggro_y: 40.0 * 16.0,
        };
        let mut old = Npc::new(kind, 90.0 * 16.0, 118.0 * 16.0 - kind.size().1, 1, 0.0);
        old.on_ground = true;
        let mut new = old.clone();
        // The twin RNG streams — both paths are RNG-free, so the pair
        // must stay equal (the stream-position witness).
        let mut rng_old = UnifiedRandom::new(12345);
        let mut rng_new = UnifiedRandom::new(12345);
        for tick in 0..240 {
            if ladder {
                walker_step_ladder(
                    &mut old, &w, pcx, pcy, params, 0.4, 8.0, player.dead,
                );
            } else {
                walker_step(&mut old, &w, pcx, pcy, params, 0.4, 8.0);
            }
            walker_step_fighter(&mut new, &w, &player, params, 0.4, 8.0);
            assert_step_eq(&old, &new, tick);
            // The new path's selection outputs: the session player is
            // slot 0 and the rect tracks it every tick. (The facing is NOT
            // pinned to a constant sign — a fast carrier reaches and
            // oscillates around the player within 240 ticks; the
            // equivalence claim is `assert_step_eq`'s old == new facing,
            // which holds because the player is parked at integer coords
            // so the helper's int-centre comparison and the old float dx
            // sign read the same boundary.)
            assert_eq!(new.target, 0, "tick {tick}: the slot-0 pick");
            assert_eq!(
                new.target_rect,
                (player.x as i32, player.y as i32, 20, 42),
                "tick {tick}: the rect tracks the player"
            );
        }
        assert_eq!(rng_old, rng_new, "the RNG stream position is untouched");
        assert!(new.x != 90.0 * 16.0, "the walker moved from its spawn");
    }

    #[test]
    fn s1_d_3p_ab_skeleton_matches_the_old_ladder_step_bit_for_bit() {
        // update_skeleton's kind family — the skeleton's num83 row
        // (`NPC.cs:57606`, cap 1.5).
        ab_fighter_kind(NpcKind::Skeleton, 1.5, true);
    }

    #[test]
    fn s1_d_3p_ab_scutlix_matches_the_old_ladder_step_bit_for_bit() {
        // update_scutlix — the 391 carrier row (`NPC.cs:58205-58207`).
        ab_fighter_kind(NpcKind::Scutlix, 5.0, true);
    }

    #[test]
    fn s1_d_3p_ab_solar_carriers_match_the_old_ladder_step_bit_for_bit() {
        // update_solar_drakomire (415, `:58214-58219`),
        // update_solar_selenian (419, `:58220-58225`), the SolarSpearman
        // dispatch arm (518, `:58226-58231`) and the VortexSoldier arm
        // (429, the num83 row `:57681-57684`).
        ab_fighter_kind(NpcKind::SolarDrakomire, 4.0, true);
        ab_fighter_kind(NpcKind::SolarSelenian, 6.0, true);
        ab_fighter_kind(NpcKind::SolarSpearman, 5.0, true);
        ab_fighter_kind(NpcKind::VortexSoldier, 4.0, true);
    }

    #[test]
    fn s1_d_3p_ab_vortex_rifleman_matches_the_old_ladder_step_bit_for_bit() {
        // update_vortex_rifleman — the 425 num83 row (`NPC.cs:57677`).
        ab_fighter_kind(NpcKind::VortexRifleman, 6.0, true);
    }

    #[test]
    fn s1_d_3p_ab_shooter_walkers_match_the_old_direct_step_bit_for_bit() {
        // update_stardust_spider_big (409), update_stardust_soldier
        // (411), update_nebula_predictor (424) — the ladder declines the
        // shooter ids (`:58322`'s exclusion), so the old path is the
        // plain `walker_step` direct write and the new path's fallback
        // is the same write.
        ab_fighter_kind(NpcKind::StardustSpiderBig, 2.0, false);
        ab_fighter_kind(NpcKind::StardustSoldier, 2.0, false);
        ab_fighter_kind(NpcKind::Predictor, 1.0, false);
    }

    #[test]
    fn s1_d_3p_the_day_gate_matches_the_csharp_discouragement() {
        // `NPC.cs:60300-60330`: by day above the surface the chain holds
        // and the switch decides; at night / underground it fails and
        // every fighter chases. The fixture's `world_surface` is pinned
        // ABOVE the NPC's y so "above the surface" is literal (the C#
        // compares `position.Y > worldSurface * 16`).
        let mut w = ground(); // day
        w.world_surface = 130.0; // 2080 px — the NPC at y 1880 is above it
        let mut sk = Npc::new(NpcKind::Skeleton, 1600.0, 1880.0, 1, 0.0);
        assert!(
            !fighters_not_discouraged(&sk, &w),
            "the skeleton is day-discouraged above the surface (:60309)"
        );
        // The pillar switch cases stay encouraged even by day
        // (`:60312-60326`).
        for kind in [
            NpcKind::StardustSpiderBig, // 409
            NpcKind::StardustSoldier,   // 411
            NpcKind::SolarDrakomire,    // 415
            NpcKind::SolarSelenian,     // 419
            NpcKind::VortexRifleman,    // 425
            NpcKind::VortexSoldier,     // 429
        ] {
            let n = Npc::new(kind, 1600.0, 1880.0, 1, 0.0);
            assert!(
                fighters_not_discouraged(&n, &w),
                "{kind:?}'s switch case keeps it encouraged"
            );
        }
        // The Predictor (424) BREAKS the chain (`npcID != 424`,
        // `:60302`) — the lunar mobs are never discouraged, even by day.
        let pr = Npc::new(NpcKind::Predictor, 1600.0, 1880.0, 1, 0.0);
        assert!(fighters_not_discouraged(&pr, &w), "424 breaks the chain");
        // Night: the chain's IsItDay term fails → everyone chases.
        w.day_time = false;
        assert!(fighters_not_discouraged(&sk, &w));
        assert!(fighters_not_discouraged(&pr, &w));
        // Underground by day: the surface term fails → everyone chases.
        w.day_time = true;
        sk.y = 140.0 * 16.0; // 2240 > worldSurface 2080
        assert!(fighters_not_discouraged(&sk, &w));
        // The eclipse suppresses the discouragement (`!Main.eclipse`).
        sk.y = 1880.0;
        w.eclipse = true;
        assert!(fighters_not_discouraged(&sk, &w));
    }

    /// W4-g4 lane 15 — the graveyard term of the chain (`NPC.cs:60302`'s
    /// `!Main.player[target].ZoneGraveyard`): a day-discouraged skeleton
    /// above the surface chases again when the target's zone pre-stamp
    /// is set (the sim feeds `scene_metrics.zone_graveyard`).
    #[test]
    fn w4_g4_the_graveyard_stamp_cancels_the_day_discouragement() {
        let mut w = ground(); // day
        w.world_surface = 130.0;
        let mut sk = Npc::new(NpcKind::Skeleton, 1600.0, 1880.0, 1, 0.0);
        assert!(!fighters_not_discouraged(&sk, &w), "the day gate holds");
        sk.target_zone_graveyard = true;
        assert!(
            fighters_not_discouraged(&sk, &w),
            "the graveyard zone breaks the chain (:60302)"
        );
    }

    /// W4-g4 lane 15 — the confused walk-direction reversal in the
    /// GENERIC walker arms (`NPC.cs:77772-77775`, the
    /// SetTargetTrackingValues tail): the ad-hoc dx-sign facing flips
    /// under Confused (31) BEFORE the sprite lockstep and the velocity
    /// write, so the walker accelerates AWAY from the player. Both
    /// archetypes carry it — `walker_step` (the direct write) and
    /// `walker_step_ladder` (the aiStyle-3 ladder reads the flipped
    /// `direction`). The buff-31 latch convention (P3).
    #[test]
    fn w4_g4_the_confused_generic_walkers_reverse_the_walk() {
        let w = night_ground();
        // The player parked LEFT of the walker: the calm facing is -1.
        let player = parked_player(&w, 60.0 * 16.0, 118.0 * 16.0 - 42.0);
        let (pcx, pcy) = player.center();
        let params = WalkerParams {
            speed: 1.0,
            jump: -6.2,
            aggro_x: 80.0 * 16.0,
            aggro_y: 40.0 * 16.0,
        };
        // walker_step (the direct write — the archer/shooter stand-ins).
        let mut calm = Npc::new(NpcKind::GoblinArcher, 90.0 * 16.0, 118.0 * 16.0 - 38.0, 1, 0.0);
        calm.on_ground = true;
        let mut conf = calm.clone();
        conf.buffs.add_buff(31, 600);
        walker_step(&mut calm, &w, pcx, pcy, params, 0.4, 8.0);
        walker_step(&mut conf, &w, pcx, pcy, params, 0.4, 8.0);
        assert_eq!(calm.facing, -1, "the calm walker faces the player");
        assert_eq!(conf.facing, 1, "the confused flip (:77774)");
        assert_eq!(conf.sprite_dir, 1, "the lockstep rides the flip");
        assert_eq!(conf.vx.to_bits(), 1.0f32.to_bits(), "the walk reverses");
        assert_eq!(calm.vx.to_bits(), (-1.0f32).to_bits());
        // walker_step_ladder (the aiStyle-3 ladder — the VortexSoldier
        // stand-in): the ladder's accel reads the flipped direction.
        // W4-g4 P9: the kind is the Soldier 429 (null immunity row,
        // `NPCID.cs:8561`), NOT the Headcrab 428 — the headcrab is
        // Confused-immune (`NPCID.cs:8555-8559`), so its buff-31 add is
        // rejected and the flip never fires for it in the C#.
        let mut calm3 = Npc::new(NpcKind::VortexSoldier, 90.0 * 16.0, 118.0 * 16.0 - 56.0, 1, 0.0);
        calm3.on_ground = true;
        let mut conf3 = calm3.clone();
        conf3.buffs.add_buff(31, 600);
        walker_step_ladder(&mut calm3, &w, pcx, pcy, params, 0.4, 8.0, false);
        walker_step_ladder(&mut conf3, &w, pcx, pcy, params, 0.4, 8.0, false);
        assert_eq!(calm3.facing, -1);
        assert_eq!(conf3.facing, 1, "the ladder arm flips too");
        assert!(calm3.vx < 0.0, "the calm ladder accelerates left");
        assert!(conf3.vx > 0.0, "the confused ladder accelerates right");
        // The fighter core keeps ONE flip (its TargetClosest already
        // carries :77772 — the generic arms must not double it): the
        // confused skeleton chases away, not back toward the player.
        let mut sk = Npc::new(NpcKind::Skeleton, 90.0 * 16.0, 118.0 * 16.0 - 40.0, 1, 0.0);
        sk.on_ground = true;
        sk.buffs.add_buff(31, 600);
        walker_step_fighter(&mut sk, &w, &player, params, 0.4, 8.0);
        assert_eq!(sk.facing, 1, "one flip only — the core's selection owns it");
        assert!(sk.vx > 0.0, "the confused fighter walks away");
    }

    #[test]
    fn s1_d_3p_the_discouraged_arm_decays_instead_of_chasing() {
        // By day above the surface the skeleton does not chase: the port
        // keeps the zombie convention's 0.9 decay (the C# wander-flip arm
        // `NPC.cs:57375-57401` is the full AI_003 migration's job).
        let mut w = ground(); // day
        w.world_surface = 130.0; // the NPC at y ~1880 is above the surface
        let player = parked_player(&w, 60.0 * 16.0, 118.0 * 16.0 - 42.0);
        let mut n = Npc::new(NpcKind::Skeleton, 90.0 * 16.0, 118.0 * 16.0 - 40.0, 1, 0.0);
        n.on_ground = true;
        n.vx = 1.0;
        let params = WalkerParams {
            speed: 1.5,
            jump: -6.2,
            aggro_x: 0.0,
            aggro_y: 0.0,
        };
        walker_step_fighter(&mut n, &w, &player, params, 0.4, 8.0);
        assert_eq!(n.vx.to_bits(), (1.0f32 * 0.9).to_bits(), "the decay arm");
        assert_eq!(n.target, 255, "no selection ran — the ClearTarget default");
    }

    #[test]
    fn s1_d_3p_the_chase_has_no_distance_box() {
        // The C# chases gated by the despawn-discouragement arm, not a
        // range window: a player 105 tiles away (outside the old 80×40
        // box, still in the world) is still faced and tracked at night.
        let w = night_ground();
        let player = parked_player(&w, 195.0 * 16.0, 118.0 * 16.0 - 42.0);
        let mut n = Npc::new(NpcKind::Skeleton, 90.0 * 16.0, 118.0 * 16.0 - 40.0, -1, 0.0);
        n.on_ground = true;
        let params = WalkerParams {
            speed: 1.5,
            jump: -6.2,
            aggro_x: 0.0,
            aggro_y: 0.0,
        };
        walker_step_fighter(&mut n, &w, &player, params, 0.4, 8.0);
        assert_eq!(n.target, 0, "the far player is still the pick");
        assert_eq!(n.facing, 1, "the NPC turns to face it");
        assert_eq!(n.sprite_dir, 1);
        assert!(n.vx > 0.0, "the ladder accelerates toward it");
    }

    #[test]
    fn s1_d_3p_the_dead_target_is_tracked_not_faced() {
        // `NPC.cs:77736-77739` — a dead picked player suppresses the
        // facing write; the port's velocity fallback keeps the old
        // facing's walk (the documented zombie convention).
        let w = night_ground();
        let mut player = parked_player(&w, 60.0 * 16.0, 118.0 * 16.0 - 42.0);
        player.dead = true;
        let mut n = Npc::new(NpcKind::StardustSoldier, 90.0 * 16.0, 118.0 * 16.0 - 56.0, 1, 0.0);
        n.on_ground = true;
        let params = WalkerParams {
            speed: 2.0,
            jump: -6.2,
            aggro_x: 0.0,
            aggro_y: 0.0,
        };
        walker_step_fighter(&mut n, &w, &player, params, 0.4, 8.0);
        assert_eq!(n.target, 0);
        assert_eq!(n.facing, 1, "the facing write is suppressed (:77736)");
        assert_eq!(
            n.target_rect,
            (player.x as i32, player.y as i32, 20, 42),
            "the corpse is still tracked"
        );
    }

    #[test]
    fn s1_d_3p_the_non_fighter_stand_in_keeps_the_legacy_step() {
        // The Present Mimic (aiStyle 25) rides update_skeleton as a
        // documented stand-in — walker_step_fighter must keep its legacy
        // ladder step (the 80×40 box included) byte-for-byte.
        let w = night_ground();
        let player = parked_player(&w, 60.0 * 16.0, 118.0 * 16.0 - 42.0);
        let (pcx, pcy) = player.center();
        let params = WalkerParams {
            speed: 3.5,
            jump: -6.2,
            aggro_x: 80.0 * 16.0,
            aggro_y: 40.0 * 16.0,
        };
        let mut old = Npc::new(NpcKind::PresentMimic, 90.0 * 16.0, 118.0 * 16.0 - 24.0, 1, 0.0);
        old.on_ground = true;
        let mut new = old.clone();
        for tick in 0..60 {
            walker_step_ladder(&mut old, &w, pcx, pcy, params, 0.4, 8.0, player.dead);
            walker_step_fighter(&mut new, &w, &player, params, 0.4, 8.0);
            assert_step_eq(&old, &new, tick);
        }
        assert_eq!(new.target, 255, "the stand-in never runs the selection");
    }

    #[test]
    fn the_hover_presets_match_the_csharp_table() {
        // The per-kind `num`/`num2` pairs, re-read from `NPC.cs` (1.4.5.8).
        // The pre-audit bug shipped type 205's pair under the Hornet, so pin
        // every row against its own C# branch (`surface_damped` = the
        // type-42/231-235 arm at `NPC.cs:50341-50362`).
        let table: [(HoverParams, f32, f32, bool, &str); 7] = [
            (
                HoverParams::EATER_OF_SOULS,
                4.0,
                0.02,
                false,
                "NPC.cs:50304-50307 (type 6)",
            ),
            // W4-d36 hover batch — the census trio's pairs (the in-flight
            // stash's presets, pinned).
            (
                HoverParams::CRIMERA,
                4.0,
                0.02,
                false,
                "NPC.cs:50304-50307 (type 173 shares the 6 arm)",
            ),
            (
                HoverParams::MOSS_HORNET,
                4.0,
                0.017,
                false,
                "NPC.cs:50369-50372 (type 176)",
            ),
            (
                HoverParams::MOTH,
                3.25,
                0.018,
                false,
                "NPC.cs:50364-50367 (type 205)",
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
        // W4-d36 hover batch — the flag5/bounce/collision gates the
        // in-flight stash got wrong twice, pinned against the C#:
        // `:50447` excludes {173, 6, 42, 231-235, 94, 139, 619} — 176/205
        // are OUTSIDE it (double add TRUE); case 173 writes noGravity
        // only (`:11168-11193`), so the Crimera collides and takes the
        // 6/173 ×0.4 restitution (`:50659-50661`).
        assert!(HoverParams::MOSS_HORNET.double_add, "176 is outside the :50447 exclusion set");
        assert!(HoverParams::MOTH.double_add, "205 is outside the :50447 exclusion set");
        assert!(!HoverParams::CRIMERA.double_add, "173 is IN the :50447 exclusion set");
        assert!(!HoverParams::CRIMERA.no_tile_collide, "case 173 has no noTileCollide row (:11168-11193)");
        assert_eq!(HoverParams::CRIMERA.bounce, 0.4, "the 6/173 restitution (:50659-50661)");
        assert_eq!(HoverParams::MOSS_HORNET.bounce, 0.7, "the family restitution (:50658)");
        assert_eq!(HoverParams::MOTH.bounce, 0.7, "the family restitution (:50658)");
        assert!(matches!(HoverParams::CRIMERA.rotation, HoverRotation::Aim), "the :50643 arm lists 173");
        assert!(matches!(HoverParams::MOSS_HORNET.rotation, HoverRotation::VelocityX), "the :50647 arm lists 176");
        assert!(matches!(HoverParams::MOTH.rotation, HoverRotation::VelocityX), "the :50647 arm lists 205");
    }

    /// W4-d42 rotation census (research/238 §7) — the BloodSquid's (619)
    /// hover step WRITES the Aim tilt (`HoverParams::BLOOD_SQUID`,
    /// `NPC.cs:50643-50646`: `rotation = atan2(num5, num4) - 1.57f`), the
    /// writer `render::entities::npc_rotation_allowed` admits. The aim is
    /// the 8-px-snapped target vector at the num=6 speed (`:50418-50426`).
    /// WRITTEN NOT RUN (code-only mode).
    #[test]
    fn w4_d42_the_blood_squid_hover_writes_the_aim_tilt() {
        let w = World::new("t".into(), 200, 200);
        let mut n = Npc::new(NpcKind::BloodSquid, 160.0, 160.0, 1, 0.0);
        let target = HoverTarget {
            center_x: 400.0,
            center_y: 240.0,
            top_y: 220.0,
            dead: false,
        };
        hover_step(&mut n, &w, target, HoverParams::BLOOD_SQUID);
        // The aim: snap8(400) - snap8(160 + 44/2) = 400 - 176 = 224;
        // snap8(240) - snap8(160 + 44/2) = 240 - 176 = 64 (the 44x44 box
        // — `SetDefaults` 619, `NPC.cs:16937-16938`).
        let (ax, ay) = (224.0f32, 64.0f32);
        let len = (ax * ax + ay * ay).sqrt();
        let want = (ay * 6.0 / len).atan2(ax * 6.0 / len) - 1.57;
        assert!(
            (n.rotation - want).abs() < 1e-5,
            "the Aim tilt: got {} want {want}",
            n.rotation
        );
        assert_ne!(n.rotation, 0.0);
    }

    /// W4-d42 FINAL — the BloodSquid's flag4 bob drift (`NPC.cs:50446-50473`):
    /// the `(num7 > 100f) | flag4` gate (`:50450`) is ALWAYS true for 619
    /// (flag4 lists it at `:50446`), so ai[0]++ + the ±0.023 drift run EVERY
    /// tick — Y by the ai[0] sign, X by the ±100 band, the >200 wrap to
    /// -200. The drift lands BEFORE the approach (`:50547`) and BEFORE the
    /// dead-flee/rise block (`:50480-50488`), exactly the C# order.
    /// WRITTEN NOT RUN (code-only mode).
    #[test]
    fn w4_d42_the_blood_squid_bob_drifts_every_tick_and_wraps() {
        let w = World::new("t".into(), 200, 200);
        // Far target (num7 > 150: no boost) level with the squid's centre
        // (cy == target.center_y > center_y - 200: the rise runs).
        let target = HoverTarget {
            center_x: 1200.0,
            center_y: 182.0,
            top_y: 160.0,
            dead: false,
        };
        let mut n = Npc::new(NpcKind::BloodSquid, 160.0, 160.0, 1, 0.0);
        hover_step(&mut n, &w, target, HoverParams::BLOOD_SQUID);
        // Tick 1: ai[0] 0 -> 1 (> 0: vy += 0.023; in the ±100 band:
        // vx -= 0.023), then the rise (-0.3), then the approach.
        assert_eq!(n.ai[0], 1.0);
        // The snapped aim: x = snap8(1200) - snap8(182) = 1200 - 176 = 1024;
        // y = snap8(182) - snap8(182) = 0. num7 = 1024 (> 150, no boost).
        // aim = (6, 0). vx: -0.023 + 0.1 = 0.077; vy: 0.023 - 0.3 = -0.277
        // (< aim_y 0: + 0.1) = -0.177.
        assert!((n.vx - 0.077).abs() < 1e-6, "bob X then approach: {}", n.vx);
        assert!((n.vy - (-0.177)).abs() < 1e-6, "bob Y + rise then approach: {}", n.vy);
        // The wrap: pre-load ai[0] = 200 -> ++201 (> 100: vx += 0.023;
        // > 0: vy += 0.023) then the > 200 wrap to -200 (`:50469-50472`).
        let mut n = Npc::new(NpcKind::BloodSquid, 160.0, 160.0, 1, 0.0);
        n.ai[0] = 200.0;
        hover_step(&mut n, &w, target, HoverParams::BLOOD_SQUID);
        assert_eq!(n.ai[0], -200.0, "the > 200 wrap");
        // vx: +0.023 (outside band) + 0.1 = 0.123; vy: +0.023 - 0.3 + 0.1.
        assert!((n.vx - 0.123).abs() < 1e-6, "the wrap tick's +X drift: {}", n.vx);
        // The negative side: ai[0] = -101 -> ++-100 (<= 0: vy -= 0.023;
        // -100 is NOT < -100: vx -= 0.023), no wrap.
        let mut n = Npc::new(NpcKind::BloodSquid, 160.0, 160.0, 1, 0.0);
        n.ai[0] = -101.0;
        hover_step(&mut n, &w, target, HoverParams::BLOOD_SQUID);
        assert_eq!(n.ai[0], -100.0);
        assert!((n.vx - (-0.023 + 0.1)).abs() < 1e-6, "the -100 boundary stays in-band: {}", n.vx);
        assert!((n.vy - (-0.023 - 0.3 + 0.1)).abs() < 1e-6, "the <= 0 Y drift: {}", n.vy);
    }

    /// W4-d42 FINAL — the BloodSquid's <150px approach boost
    /// (`NPC.cs:50474-50478`: `velocity += num4/num5 * 0.007` on the
    /// NORMALIZED aim, the snapped num7 < 150 gate) and the -0.3 rise's
    /// 200px gate (`:50485-50488`: `Center.Y > targetData.Center.Y - 200f`,
    /// the dead-flee's ELSE-if). WRITTEN NOT RUN (code-only mode).
    #[test]
    fn w4_d42_the_blood_squid_close_boost_and_rise_gates() {
        let w = World::new("t".into(), 200, 200);
        // Near target: snapped aim (96, 0), num7 = 96 < 150 -> the boost
        // adds aim * 0.007 AFTER the drift, BEFORE the approach.
        let near = HoverTarget {
            center_x: 278.0,
            center_y: 182.0,
            top_y: 160.0,
            dead: false,
        };
        let mut n = Npc::new(NpcKind::BloodSquid, 160.0, 160.0, 1, 0.0);
        hover_step(&mut n, &w, near, HoverParams::BLOOD_SQUID);
        // aim = (6, 0); vx: -0.023 (drift) + 6*0.007 (boost) + 0.1 = 0.119.
        assert!((n.vx - 0.119).abs() < 1e-6, "drift + boost + approach: {}", n.vx);
        // The 150 boundary: the snapped distance is an 8px multiple, so
        // the first step ABOVE 150 is 152 — NOT < 150 (no boost).
        let edge = HoverTarget { center_x: 328.0, center_y: 182.0, top_y: 160.0, dead: false };
        let mut n = Npc::new(NpcKind::BloodSquid, 160.0, 160.0, 1, 0.0);
        hover_step(&mut n, &w, edge, HoverParams::BLOOD_SQUID);
        assert!((n.vx - 0.077).abs() < 1e-6, "the strict < 150 gate: {}", n.vx);
        // The rise gate: the squid's centre 182 vs target centre 400 —
        // 182 > 400 - 200 = 200 is FALSE, no rise. num7 = 218 > 150.
        let high = HoverTarget { center_x: 182.0, center_y: 400.0, top_y: 380.0, dead: false };
        let mut n = Npc::new(NpcKind::BloodSquid, 160.0, 160.0, 1, 0.0);
        hover_step(&mut n, &w, high, HoverParams::BLOOD_SQUID);
        // vy: +0.023 (drift) then the approach towards aim_y = 6: +0.1.
        assert!((n.vy - 0.123).abs() < 1e-6, "the rise stays off above the gate: {}", n.vy);
        // The dead target: the flee REPLACES the aim and the rise's else-if
        // never runs (`:50480-50484` wins). The bob still runs (it precedes
        // the flag block in the C#).
        let dead = HoverTarget { center_x: 278.0, center_y: 182.0, top_y: 160.0, dead: true };
        let mut n = Npc::new(NpcKind::BloodSquid, 160.0, 160.0, 1, 0.0);
        hover_step(&mut n, &w, dead, HoverParams::BLOOD_SQUID);
        // vx: -0.023 + 6*0.007 then the approach towards the flee's
        // facing*3 = +3: +0.1 -> 0.119. vy: +0.023 (drift), NO rise, then
        // the approach towards -3: -0.1 -> -0.077 (flag5 false for 619:
        // no sign-flip double-add).
        assert!((n.vx - 0.119).abs() < 1e-6, "the flee keeps the boost: {}", n.vx);
        assert!((n.vy - (-0.077)).abs() < 1e-6, "the flee skips the rise: {}", n.vy);
    }

    /// W4-d42 FINAL — the bob/boost/rise gates are BloodSquid-ONLY: the
    /// four legacy presets keep them false so every pre-existing hover
    /// kind's behaviour is frozen (the ANALYSIS gap-closure scope).
    /// WRITTEN NOT RUN (code-only mode).
    #[test]
    fn w4_d42_the_hover_bob_gates_are_blood_squid_only() {
        for p in [
            HoverParams::EATER_OF_SOULS,
            HoverParams::HORNET,
            HoverParams::SERVANT_OF_CTHULHU,
            HoverParams::METEOR_HEAD,
        ] {
            assert!(!p.bob_drift, "the legacy preset bobs (frozen behaviour)");
            assert!(!p.close_boost, "the legacy preset boosts (frozen behaviour)");
            assert!(!p.hover_rise, "the legacy preset rises (frozen behaviour)");
        }
        let p = HoverParams::BLOOD_SQUID;
        assert!(p.bob_drift && p.close_boost && p.hover_rise);
    /// W4-d22: the per-kind drift / wet / dust-gate / dawn-exemption rows
    /// re-read from `NPC.cs:50445-50447` (the num8/flag4 sets),
    /// `:50695-50712` (the dust gate), `:50714-50753` (the wet rows) and
    /// `:50906` (the dawn-flee exemption list).
    #[test]
    fn w4_d22_the_hover_drift_wet_dust_dawn_rows_match_the_csharp() {
        // The num8 set (`:50445`) holds type 6; flag4 (`:50446`) holds
        // 42/210/211; the servant (5) and Meteor Head (23) are in neither.
        assert_eq!(HoverParams::EATER_OF_SOULS.drift, HoverDrift::EaterOfSouls);
        assert_eq!(HoverParams::HORNET.drift, HoverDrift::Flag4);
        assert_eq!(HoverParams::SERVANT_OF_CTHULHU.drift, HoverDrift::None);
        assert_eq!(HoverParams::METEOR_HEAD.drift, HoverDrift::None);
        // The wet rows: 6 → the −0.3/−2 row, 42 → the −0.5/−4 row.
        assert_eq!(HoverParams::EATER_OF_SOULS.wet, HoverWet::EaterOfSouls);
        assert_eq!(HoverParams::HORNET.wet, HoverWet::Hornet);
        assert_eq!(HoverParams::SERVANT_OF_CTHULHU.wet, HoverWet::None);
        assert_eq!(HoverParams::METEOR_HEAD.wet, HoverWet::None);
        // The dust gates: 6/23 → Next(20)/type 18/this.scale (the
        // bounce-list arm `:50695-50704`); 5 → Next(40)/type 5/stock 1.0
        // (the else arm `:50708-50712`); 42 → NONE (the `:50695` type list
        // excludes it).
        assert_eq!(
            HoverParams::EATER_OF_SOULS.dust,
            Some(HoverDust { gate: 20, dust_type: 18, use_npc_scale: true })
        );
        assert_eq!(
            HoverParams::METEOR_HEAD.dust,
            Some(HoverDust { gate: 20, dust_type: 18, use_npc_scale: true })
        );
        assert_eq!(
            HoverParams::SERVANT_OF_CTHULHU.dust,
            Some(HoverDust { gate: 40, dust_type: 5, use_npc_scale: false })
        );
        assert_eq!(HoverParams::HORNET.dust, None);
        // The dawn-flee exemption list (`:50906`) holds 6/23/42/210/211 —
        // every ported kind BUT the servant (5), which climbs away at dawn.
        assert!(HoverParams::EATER_OF_SOULS.dawn_exempt);
        assert!(HoverParams::HORNET.dawn_exempt);
        assert!(HoverParams::METEOR_HEAD.dawn_exempt);
        assert!(!HoverParams::SERVANT_OF_CTHULHU.dawn_exempt);
    }

    // ---- W4-f4 witnesses: the hover dust gate (sites 50702/50710) ----
    // WRITTEN NOT RUN (code-only). The mirror replays the gate draw +
    // the pool internals draw-for-draw; the colour census pins
    // `this.color` = default on every hover kind.

    /// W4-f4 — site 50702 (`NPC.cs:50695-50705`): the Eater of Souls /
    /// Meteor Head arm. The Next(20) gate draws EVERY tick; on 0 the
    /// type-18 dust spawns at (x, y+h*0.25) over width×(int)(h*0.5),
    /// velocity (vx, 2), alpha 75, default colour, this.scale, then the
    /// ×0.5/×0.1 velocity posts. Type 18 hits no per-type tweak.
    #[test]
    fn w4_f4_the_hover_bounce_arm_mirrors_site_50702() {
        let w = ground();
        let dctx = crate::npc::permissive_dust_ctx(&w);
        let mut npc = Npc::new(NpcKind::EaterOfSouls, 100.0, 100.0, 1, 0.0);
        npc.vx = 1.5;
        npc.scale = 1.2;
        let (nw, nh) = npc.size();
        let mut rng = UnifiedRandom::new(42);
        let mut witness = rng.clone();
        let mut dust = tr_core::particles::DustPool::new();
        let mut mirror = tr_core::particles::DustPool::new();
        // Find a seed-42 tick where Next(20) == 0 (the gate fires), and
        // mirror the whole run: the gate draw every tick, the internals
        // on the hits.
        let mut fired = 0;
        for _ in 0..60 {
            let g = witness.next_max(20);
            if g == 0 {
                let idx = mirror.new_dust(
                    &mut witness, &dctx, &w,
                    glam::Vec2::new(npc.x, npc.y + nh * 0.25),
                    nw as i32, (nh * 0.5) as i32, 18,
                    npc.vx, 2.0, 75,
                    tr_core::particles::Color::default(), npc.scale,
                );
                let dd = mirror.slot_mut(idx);
                dd.velocity.x *= 0.5;
                dd.velocity.y *= 0.1;
                fired += 1;
            }
            hover_dust_gate(&npc, &w, true, false, &mut rng, &mut dust, &dctx);
            assert_eq!(rng, witness, "the gate ledger tick-by-tick");
        }
        assert!(fired > 0, "seed 42 fires the Next(20) gate within 60 ticks");
        let act: Vec<usize> = (0..6000).filter(|&i| dust.slot(i).active).collect();
        assert_eq!(act.len(), fired);
        for (j, &i) in act.iter().enumerate() {
            let d = dust.slot(i);
            let m = mirror.slot(j);
            assert_eq!(d.dust_type, 18);
            assert_eq!(d.position, m.position, "the y+h*0.25 box");
            assert_eq!(d.velocity, m.velocity, "the ×0.5/×0.1 posts");
            assert_eq!(d.scale, m.scale, "this.scale rides the spawn");
            assert_eq!(d.alpha, 75);
            assert_eq!(d.color, tr_core::particles::Color::default());
        }
    }

    /// W4-f4 — site 50710 (`NPC.cs:50708-50712`): the servant's else arm.
    /// The Next(40) gate, type 5, the DEFAULT alpha 0 / colour / scale 1f
    /// (the 7-arg overload — NOT the bounce arm's 75/this.scale).
    #[test]
    fn w4_f4_the_hover_else_arm_mirrors_site_50710() {
        let w = ground();
        let dctx = crate::npc::permissive_dust_ctx(&w);
        let mut npc = Npc::new(NpcKind::ServantOfCthulhu, 100.0, 100.0, 1, 0.0);
        npc.vx = -2.0;
        npc.scale = 1.4; // the else arm ignores this.scale — the stock 1f
        let (nw, nh) = npc.size();
        let mut rng = UnifiedRandom::new(9);
        let mut witness = rng.clone();
        let mut dust = tr_core::particles::DustPool::new();
        let mut mirror = tr_core::particles::DustPool::new();
        let mut fired = 0;
        for _ in 0..120 {
            let g = witness.next_max(40);
            if g == 0 {
                let idx = mirror.new_dust(
                    &mut witness, &dctx, &w,
                    glam::Vec2::new(npc.x, npc.y + nh * 0.25),
                    nw as i32, (nh * 0.5) as i32, 5,
                    npc.vx, 2.0, 0,
                    tr_core::particles::Color::default(), 1.0,
                );
                let dd = mirror.slot_mut(idx);
                dd.velocity.x *= 0.5;
                dd.velocity.y *= 0.1;
                fired += 1;
            }
            hover_dust_gate(&npc, &w, false, false, &mut rng, &mut dust, &dctx);
            assert_eq!(rng, witness, "the else-arm ledger tick-by-tick");
        }
        assert!(fired > 0, "seed 9 fires the Next(40) gate within 120 ticks");
        let act: Vec<usize> = (0..6000).filter(|&i| dust.slot(i).active).collect();
        assert_eq!(act.len(), fired);
        for (j, &i) in act.iter().enumerate() {
            let d = dust.slot(i);
            let m = mirror.slot(j);
            assert_eq!(d.dust_type, 5);
            assert_eq!(d.position, m.position);
            assert_eq!(d.velocity, m.velocity);
            assert_eq!(d.alpha, 0, "the 7-arg overload's default alpha");
            // The scale is the stock 1f × the pool jitter — NOT ×1.4.
            assert_eq!(d.scale, m.scale);
            assert!(d.scale < 1.25, "the stock scale, not this.scale ({})", d.scale);
        }
    }

    /// W4-f4 — the `:50695` exclusion census: the Hornet (42), the Probe
    /// (139) and the volley bees (210/211) are in the bounce list but
    /// EXCLUDED — the gate draws NOTHING (the `&&` short-circuits before
    /// Main.rand). The colour census: no hover kind writes NPC.color.
    #[test]
    fn w4_f4_the_excluded_hover_kinds_draw_nothing() {
        let w = ground();
        let dctx = crate::npc::permissive_dust_ctx(&w);
        for kind in [NpcKind::Hornet, NpcKind::Probe, NpcKind::Bee, NpcKind::BeeSmall] {
            let npc = Npc::new(kind, 100.0, 100.0, 1, 0.0);
            let mut rng = UnifiedRandom::new(42);
            let before = rng.peek();
            let mut dust = tr_core::particles::DustPool::new();
            for _ in 0..200 {
                hover_dust_gate(&npc, &w, true, true, &mut rng, &mut dust, &dctx);
            }
            assert_eq!(rng.peek(), before, "{kind:?}: the exclusion draws nothing");
            assert!((0..6000).all(|i| !dust.slot(i).active), "{kind:?}: no dust");
        }
    }
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
