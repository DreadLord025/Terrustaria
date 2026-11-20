//! W4-d15 — the DD2 weapon/pet AI slice: aiStyles **141-146** + **148**
//! (147 Celebration Mk2 already rides `projectile_ai_misc::ai_celeb2_rocket`;
//! 149/150 golf already ride `projectile_ai_golf` — the census of the
//! `match data.ai_style` dispatch). Seven MISSING styles ported verbatim:
//!
//! - aiStyle 141: `AI_141_MonkStaffT1Explosion` `Projectile.cs:69602-69620`
//!   + `CreateImpactExplosion_SleepyOctopod` `:69680-69731` +
//!   `CreateImpactExplosion2_SleepyOctopod` `:69733-69772` (the 698 row
//!   `:7385-7401`).
//! - aiStyle 142: `AI_142_MonkStaffT2And3` `:69817-69968` (the 699 row
//!   `:7402-7418`, the 708 row `:7491-7507`).
//! - aiStyle 143: `AI_143_MonkStaffT2Ghast` `:69970-70036` (the 700 row
//!   `:7419-7432`).
//! - aiStyle 144: `AI_144_DD2Pet` `:70038-70324` (the 701/702/703 row
//!   `:7433-7446`, the 764 row `:7866-7878`, the 1050 row `:9953-9966`).
//! - aiStyle 145: `AI_145_BookStaffStorm` `:70326-70468` (the 704 row
//!   `:7447-7460`).
//! - aiStyle 146: `AI_146_DD2Victory` `:53960-53962` — an EMPTY C# body
//!   (the 713 row `:7560-7571`); the port keeps the arm a documented no-op.
//! - aiStyle 148: `AI_148_StarSpawner` `:53094-53128` (the 720 row
//!   `:7617-7625`) + `Star.StarFall`/`Star.Fall` (`Star.cs`) + the spawned
//!   FallingStar 12's row (`Projectile.cs:924-938`).
//!
//! RNG contract (the `UpdateProjectiles` stream, `Main.cs:17775-17777`):
//! every EXPLICIT `Main.rand` draw at the call sites stays, in the C#'s
//! order; the `Dust.NewDust*` internal rolls go with the dust pool (the
//! `projectile_ai_misc` precedent). The `Gore.NewGoreDirect` internal kicks
//! DO stay — `Next(10, 31)` on Y first, then `Next(-20, 21)` on X
//! (`Gore.cs:1327-1328`, the LEAD-REVIEW G3 standard). `NextVector2Square`
//! is 2 `NextDouble` draws (`Utils.cs:1251-1254`, the
//! `npc_ai_frost_boss_b` census convention); `Utils.SelectRandom` is one
//! `Next(choices.Length)` (`Utils.cs:2459-2462`).
//!
//! Recorded folds (docs/ANALYSIS.md "The W4-d15 DD2 slice"):
//! - the tracked sounds (`AI_143`'s `DD2_GhastlyGlaiveImpactGhost`
//!   `:69974`, `AI_145`'s `DD2_BookStaffTwisterLoop` `:70332` + the volume
//!   ramp `:70338`) — the pool has no tracked-sound layer (S1-E-23); the
//!   `localAI[1]` slot latch is kept as inert state (`SlotId.Invalid` =
//!   `f32::from_bits(65535)`, `ReLogic.Utilities/SlotId.cs:5`/`:75-78`),
//!   zero draws.
//! - `AI_144`'s `Utils.PlotTileLine` light casts (`:70087-70091`), the
//!   `Opacity` write (`:70155`, no port field — the Last Prism fold) and
//!   the dead `Hitbox.Intersects` read (`:70143-70144`, result discarded).
//! - `AI_144`'s 1050 tree-shake `KillTile_MakeTileDust`/`PlaySounds`/
//!   `DropItems` (`:70217-70229`) — no tile-drop seam in the pool; the
//!   walk + ALL its `Main.rand` draws stay (`Next(100)`, the conditional
//!   `Next(20)`, the conditional `Next(5)`), the effects are the recorded
//!   skip. `KillTile_MakeTileDust`'s own rolls for the shakeable kinds
//!   ride `WorldGen.genRand` (the worldgen stream, not `Main.rand`) or
//!   the dust pool — zero `UpdateProjectiles` draws. `TileID.Sets.
//!   IsShakeable` (`TileID.cs:488`) rides the generated
//!   `tr_core::tiledata::SET_IS_SHAKEABLE`.
//! - `AI_148`'s `Star.StarFall` (`Star.cs:63-84`) picks a BACKGROUND star
//!   and `Star.Fall()` rolls `Next(700,1001)` then `Next(-400,401)` — the
//!   port has no falling-background-star machine (`render/sky.rs`'s
//!   `StarSprite` has no fall state), so the draws stay on the stream as
//!   the census and the visual fall is the recorded skip. The one-shot
//!   gate keeps the C#'s `Main.netMode != 2` form as single-player true.
//! - `AI_142`'s `player.heldProj` write rides the mutable player channel
//!   (the `ai_stardust_guardian` shape); `player.reuseDelay = 2`
//!   (`:69966`) rides [`Dd2Tick::reuse_delay`] back to the world-sim pass
//!   (the `FlailTick.facing` write-back precedent).
//! - `manualDirectionChange = true` on the pet rows (`:7445`) — no port
//!   field; the arms pin `direction`/`spriteDirection` themselves and the
//!   dispatch's free integrate never rewrites facing (the aiStyle-120
//!   precedent).
//! - the 698's `Colliding` overlay (`:15451-15467`, the +70 inflate and
//!   the velocity-shifted second rect) — `damage_hitbox`'s per-type
//!   overlays are the S1-C-8 follow-up, unchanged here.
//!
//! Code-only round (W4-d15): the witness tests below are WRITTEN NOT RUN.

use glam::Vec2;
use tr_core::World;
use tr_core::loot::DropRng;

use crate::bow_launch::rotated_by;
use crate::projectile::{
    MONK_STAFF_T2, MONK_STAFF_T3_ALT, MONK_STAFF_T3_ALT_SHOT, Projectile, safe_normalize_or,
};

/// The owner state the 142/144/148 arms read off `Main.player[owner]`
/// each tick, snapshotted by the world-sim pass (the `SpearOwner`
/// pattern — the slot loop cannot borrow the App). The C# reads the live
/// `Player` fields; the port snapshots them once per fixed step.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Dd2Owner {
    /// `player.RotatedRelativePoint(player.MountedCenter)` (`:69820`) —
    /// the T2/T3 swing anchor.
    pub anchor: Vec2,
    /// `player.direction` (`:69821`).
    pub direction: i32,
    /// `player.dead` (`:69824`).
    pub dead: bool,
    /// `player.frozen` (`:69829`) — the frozen owner holds the swing pose.
    pub frozen: bool,
    /// `player.itemAnimation` (`:69844`/`:69907`) — the C# INTEGER
    /// countdown; the pass derives it from `app.swing` (the spear
    /// convention).
    pub item_animation: i32,
    /// `player.itemAnimationMax` (`:69844`).
    pub item_animation_max: i32,
}

/// One queued sub-shot spawn — `(x, y)` are the C# `NewProjectile`
/// CENTER arguments (the dispatch recentres on the row's half-box, the
/// `SpearSubShot` convention).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Dd2Shot {
    pub kind: u16,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
    pub knock_back: f32,
}

/// What the 142/148 arms hand back to the dispatch (the `FlailTick`
/// write-back precedent): the queued sub-shot spawns and the player-side
/// writes.
#[derive(Default)]
pub(crate) struct Dd2Tick {
    /// `NewProjectile(..., 709, this.damage, 0f, this.owner)` (`:69924`)
    /// and the 148's FallingStar 12 (`:53126`).
    pub shots: Vec<Dd2Shot>,
    /// `player.reuseDelay = 2` (`:69966`) — the pass converts to seconds.
    pub reuse_delay: Option<i32>,
    /// `player.heldProj = base.whoAmI` (`:69822`) — the slot index.
    pub held_proj: Option<i32>,
}

/// `Vector2.AngleTo` (`Utils.cs:1924-1927`) — `atan2(dy, dx)`.
fn angle_to(origin: Vec2, target: Vec2) -> f32 {
    (target.y - origin.y).atan2(target.x - origin.x)
}

/// `Vector2.DirectionTo` (`Utils.cs:1955-1958`) — `Vector2.Normalize`
/// with NO safe fallback: a zero delta yields NaN in both engines (the
/// `1/sqrt(0)` infinity times the zero components).
fn direction_to(origin: Vec2, target: Vec2) -> Vec2 {
    let d = target - origin;
    let len = (d.x * d.x + d.y * d.y).sqrt();
    if len > 0.0 { d / len } else { Vec2::splat(f32::NAN) }
}

/// `AI_142_MonkStaffT2And3` (`Projectile.cs:69817-69968`) — the Monk
/// Staff T2 (699) / T3-alt (708) swing. The head block pins the facing,
/// the owner's `heldProj` and the Centre on the rotated mount anchor
/// (`:69820-69823`); a dead owner kills (`:69824-69828`); a frozen owner
/// holds the pose (the type arms sit under `!player.frozen`, `:69829`).
///
/// RNG: the 699's two trail dusts roll ONE `NextFloat` each (the scale,
/// `:69865`), then the `Next(3)` gate (`:69871`) and the taken branch's
/// TWO floats (scale `:69876` + fadeIn `:69877`). The 708's sub-shot arm
/// (`:69907-69925`) draws `NextVector2Square(-0.2, 0.2)` = 2 `NextDouble`
/// (`Utils.cs:1251-1254`) THEN `Next(4)` (`:69923`) — the switch
/// overwrite (`:69911-69922`) keeps the draws spent; the two dust gates
/// (`k = 0, 2`, `:69926`) roll `Next(6)` each and the taken branch one
/// `NextFloat` (`:69948`) + one `Next(3)` (`:69954`). The dust bodies
/// themselves are the pool skip; `Lighting.AddLight` (`:69885`) is the
/// light skip.
pub(crate) fn ai_monk_staff_t2_and_3(
    p: &mut Projectile,
    who_am_i: usize,
    o: &Dd2Owner,
    local_owner: u8,
    rng: &mut dyn DropRng,
) -> Dd2Tick {
    let mut tick = Dd2Tick::default();
    // `:69819-69823` — the anchor, the facing, the heldProj claim and the
    // Centre pin (all ahead of the dead gate).
    let vector = o.anchor;
    p.direction = o.direction;
    tick.held_proj = Some(who_am_i as i32);
    p.pos = vector - Vec2::new(p.w * 0.5, p.h * 0.5);
    // `:69824-69828` — the dead owner kills and returns.
    if o.dead {
        kill(p);
        return tick;
    }
    // `:69829-69962` — the frozen owner holds the swing pose.
    if !o.frozen {
        if p.kind == MONK_STAFF_T2 {
            swing_t2(p, o, vector, rng);
        } else if p.kind == MONK_STAFF_T3_ALT {
            swing_t3_alt(p, o, vector, local_owner, rng, &mut tick);
        }
    }
    // `:69963-69967` — the local owner's animation end kills and arms the
    // 2-tick reuse delay (the `player.whoAmI == Main.myPlayer` gate folds
    // to the slot-owner comparison — the pool simulates local slots).
    if p.owner == local_owner && o.item_animation <= 2 {
        kill(p);
        tick.reuse_delay = Some(2);
    }
    tick
}

/// The 699 arm (`Projectile.cs:69831-69882`): the alpha fade, the
/// `localAI[0]` countdown, the swing arc position/rotation and the two
/// dust trails' draw census.
fn swing_t2(p: &mut Projectile, o: &Dd2Owner, vector: Vec2, rng: &mut dyn DropRng) {
    // `:69833-69843` — the facing pin, the −127 alpha fade (floor 0) and
    // the localAI[0] countdown.
    p.sprite_direction = o.direction;
    p.direction = o.direction;
    p.alpha -= 127;
    if p.alpha < 0 {
        p.alpha = 0;
    }
    if p.local_ai[0] > 0.0 {
        p.local_ai[0] -= 1.0;
    }
    // `:69844-69850` — the swing arc: the animation fraction, the
    // velocity's rotation/length, the π + num·2π spin scaled by
    // (length, ai[0]), the position kick and the +40 aim point.
    let num = 1.0 - o.item_animation as f32 / o.item_animation_max as f32;
    let num2 = p.vel.y.atan2(p.vel.x);
    let num3 = p.vel.length();
    let num4 = 22.0f32;
    let spin_angle = (std::f32::consts::PI + num * (std::f32::consts::PI * 2.0)) as f64;
    let sp = rotated_by(Vec2::new(1.0, 0.0), spin_angle) * Vec2::new(num3, p.ai[0]);
    let sp_rot = rotated_by(sp, num2 as f64);
    p.pos += sp_rot + rotated_by(Vec2::new(num3 + num4, 0.0), num2 as f64);
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    let target = vector + sp_rot + rotated_by(Vec2::new(num3 + num4 + 40.0, 0.0), num2 as f64);
    // `:69851-69855` — the aim rotation + the π/4 facing offset, the
    // spriteDirection −1 π flip.
    p.rot = angle_to(vector, target) + std::f32::consts::FRAC_PI_4 * o.direction as f32;
    if p.sprite_direction == -1 {
        p.rot += std::f32::consts::PI;
    }
    // `:69856-69858` — the dead `DirectionTo(Center)` discard stays dead;
    // vector3 (the aim direction) and vector4 (the safe velocity
    // direction) feed dust only.
    let _vector3 = direction_to(vector, target);
    let _vector4 = safe_normalize_or(p.vel, Vec2::Y);
    // `:69859-69868` — the two trail dusts: ONE NextFloat each (the
    // scale `:69865`); the position/velocity writes are the dust skip.
    let num5 = 2.0f32;
    let mut i = 0.0f32;
    while i < num5 {
        let _ = rng.next_float(); // `:69865`
        i += 1.0;
    }
    // `:69869-69881` — the one-shot 1-in-3 sparkle: the gate draw, then
    // TWO floats on the taken branch (scale `:69876` + fadeIn `:69877`).
    if rng.next_range(0, 3) == 0 {
        let _ = rng.next_float(); // `:69876`
        let _ = rng.next_float(); // `:69877`
    }
    let _ = center;
}

/// The 708 arm (`Projectile.cs:69883-69961`): the same swing arc with the
/// SafeNormalize(UnitX) rotation, the animation-2/6/10 sub-shot volley
/// (709) and the two dust gates.
fn swing_t3_alt(
    p: &mut Projectile,
    o: &Dd2Owner,
    vector: Vec2,
    local_owner: u8,
    rng: &mut dyn DropRng,
    tick: &mut Dd2Tick,
) {
    // `:69885` — Lighting.AddLight: the light skip.
    // `:69886-69891` — the facing pin and the −127 alpha fade.
    p.sprite_direction = o.direction;
    p.direction = o.direction;
    p.alpha -= 127;
    if p.alpha < 0 {
        p.alpha = 0;
    }
    // `:69892-69898` — the swing arc (the 699 shape, num6..num9).
    let num6 = 1.0 - o.item_animation as f32 / o.item_animation_max as f32;
    let num7 = p.vel.y.atan2(p.vel.x);
    let num8 = p.vel.length();
    let num9 = 22.0f32;
    let spin_angle = (std::f32::consts::PI + num6 * (std::f32::consts::PI * 2.0)) as f64;
    let sp2 = rotated_by(Vec2::new(1.0, 0.0), spin_angle) * Vec2::new(num8, p.ai[0]);
    let sp2_rot = rotated_by(sp2, num7 as f64);
    p.pos += sp2_rot + rotated_by(Vec2::new(num8 + num9, 0.0), num7 as f64);
    let vector5 = vector + sp2_rot + rotated_by(Vec2::new(num8 + num9 + 40.0, 0.0), num7 as f64);
    // `:69899-69903` — the SafeNormalize(UnitX) aim rotation.
    p.rot = safe_normalize_or(vector5 - vector, Vec2::X).to_angle()
        + std::f32::consts::FRAC_PI_4 * o.direction as f32;
    if p.sprite_direction == -1 {
        p.rot += std::f32::consts::PI;
    }
    // `:69904-69905` — the two dead SafeNormalize discards stay dead.
    // `:69906` — the safe velocity direction.
    let vector6 = safe_normalize_or(p.vel, Vec2::Y);
    // `:69907-69925` — the animation-2/6/10 volley (local owner only).
    if (o.item_animation == 2 || o.item_animation == 6 || o.item_animation == 10)
        && p.owner == local_owner
    {
        // `:69909-69910` — the jitter draw pair + the ×12 (the switch
        // below overwrites the value; the draws stay spent).
        let jx = (0.4 * rng.next_double() as f32) - 0.2;
        let jy = (0.4 * rng.next_double() as f32) - 0.2;
        let mut vector7 = (vector6 + Vec2::new(jx, jy)) * 12.0;
        // `:69911-69922` — the per-animation heading overwrite.
        match o.item_animation {
            2 => vector7 = rotated_by(vector6, 0.3839724659919739),
            6 => vector7 = rotated_by(vector6, -0.3839724659919739),
            10 => vector7 = rotated_by(vector6, 0.0),
            _ => {}
        }
        // `:69923-69924` — the speed roll and the 709 spawn at Centre.
        vector7 *= 10.0 + rng.next_range(0, 4) as f32;
        let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
        tick.shots.push(Dd2Shot {
            kind: MONK_STAFF_T3_ALT_SHOT,
            x: center.x,
            y: center.y,
            vx: vector7.x,
            vy: vector7.y,
            damage: p.damage,
            knock_back: 0.0,
        });
    }
    // `:69926-69960` — the dust gates: k = 0, 2 (the `k += 2` loop); the
    // k = 1/3 switch arms are unreachable (the C#'s own dead branches).
    // Per gate: Next(6); the taken branch one NextFloat (`:69948`) +
    // one Next(3) (`:69954`).
    for k in [0i32, 2] {
        let (num10, mut num11) = if k == 2 { (0.5f32, 1.25f32) } else { (1.0, 1.0) };
        if rng.next_range(0, 6) != 0 {
            num11 *= 1.2;
            let _ = rng.next_float(); // `:69948` — the velocity scale
            let _ = num10 * num11; // the dust velocity write (the skip)
            if rng.next_range(0, 3) == 0 {
                // `:69956-69957` — the noGravity/fadeIn flip: no draws.
            }
        }
    }
}

/// The sibling-module deactivation (`Projectile::kill` is private — the
/// `projectile_ai_spear` precedent).
fn kill(p: &mut Projectile) {
    p.kill_time_left = p.time_left;
    p.active = false;
    p.time_left = 0;
}

/// `AI_141_MonkStaffT1Explosion` (`Projectile.cs:69602-69620`) — the
/// Sleepy Octopod's ground blast 698. The scan box is the slot's
/// TopLeft..BottomRight tile span with `scanAreaEnd.Y++` (`:69609`), the
/// explosion range is the WIDTH (`:69608`); the counter kills past 20
/// and the tick-1 arm runs both impact helpers. The `scanAreaStart.X / 2`
/// discards (`:69606-69607`) are C# dead reads — not carried.
pub(crate) fn ai_monk_staff_t1_explosion(p: &mut Projectile, world: &World, rng: &mut dyn DropRng) {
    // `:69604-69609` — the tile span (`ToTileCoordinates` = `(int)(X/16)`).
    let x0 = (p.pos.x / 16.0) as i32;
    let y0 = (p.pos.y / 16.0) as i32;
    let x1 = ((p.pos.x + p.w) / 16.0) as i32;
    let y1 = ((p.pos.y + p.h) / 16.0) as i32 + 1;
    let explosion_range = p.w;
    // `:69610-69614` — the counter and the 20-tick kill.
    p.ai[0] += 1.0;
    if p.ai[0] > 20.0 {
        kill(p);
    } else if p.ai[0] == 1.0 {
        // `:69617` — `base.Bottom` = (x + w/2, y + h) (`Entity.cs:122`).
        let bottom = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h);
        let shocked = impact_explosion_sleepy_octopod(p, world, rng, 8, bottom, x0, y0, x1, y1, explosion_range);
        // `:69618` — the ring origin backs off 10× the velocity.
        impact_explosion2_sleepy_octopod(p, rng, bottom - p.vel * 10.0, shocked);
    }
}

/// `CreateImpactExplosion_SleepyOctopod` (`Projectile.cs:69680-69731`) —
/// the tile walk as the DRAW CENSUS: `num = 3f` (the velocity.X kick
/// scale), `num2 = 4` (the dust layer), `num3 = 0.75f` (the scale pin).
/// Per qualifying tile: the dust amount × multiplier, then that many
/// `KillTile_MakeTileDust` bodies (each one `NextFloat` velocity.Y scale,
/// `:69709`; the X kick `:69710` rolls its own `NextFloat`), then
/// `num2 >= 2`'s second row of amount−1 (one `NextFloat` each, `:69720`;
/// the X kick `:69721` is fixed — no draw). `KillTile_GetTileDustAmount`
/// rides the misc module's port (`WorldGen.cs:66597-66685`).
fn impact_explosion_sleepy_octopod(
    _p: &Projectile,
    world: &World,
    rng: &mut dyn DropRng,
    dust_amount_multiplier: i32,
    origin: Vec2,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    explosion_range: f32,
) -> bool {
    let mut caused_shockwaves = false;
    for i in x0..=x1 {
        for j in y0..=y1 {
            // `:69690-69693` — the radius gate against the tile CORNER.
            if origin.distance(Vec2::new(i as f32 * 16.0, j as f32 * 16.0)) > explosion_range {
                continue;
            }
            // `:69694-69703` — the exposed full-solid gate (the
            // create_ground_explosion shape).
            let Some(t) = world.tile(i, j) else { continue };
            let solid = tr_core::tiledata::TILE_SOLID.has(t.kind);
            let solid_top = tr_core::tiledata::TILE_SOLID_TOP.has(t.kind);
            let important = world.frame_important.get(t.kind as usize).copied().unwrap_or(false);
            if !t.active() || !solid || solid_top || important {
                continue;
            }
            let above_full_solid = world.tile(i, j - 1).is_some_and(|a| {
                a.active()
                    && tr_core::tiledata::TILE_SOLID.has(a.kind)
                    && !tr_core::tiledata::TILE_SOLID_TOP.has(a.kind)
            });
            if above_full_solid {
                continue;
            }
            // `:69704-69713` — the first dust row: TWO NextFloats each
            // (the Y scale `:69709` + the X kick's `0.3+0.7*NextFloat`
            // `:69710`).
            let amount = crate::projectile_ai_misc::kill_tile_dust_amount_fail(t.kind, t.frame_x)
                * dust_amount_multiplier;
            for _ in 0..amount {
                let _ = rng.next_float();
                let _ = rng.next_float();
            }
            // `:69714-69724` — num2 = 4 >= 2: the second row, amount−1
            // dusts with ONE NextFloat each (`:69720`).
            for _ in 0..(amount - 1) {
                let _ = rng.next_float();
            }
            if amount > 0 {
                caused_shockwaves = true;
            }
        }
    }
    caused_shockwaves
}

/// `CreateImpactExplosion2_SleepyOctopod` (`Projectile.cs:69733-69772`) —
/// the 25-point ring census: each particle rolls TWO `NextFloat`s (the
/// X kick `:69752` + the fadeIn `:69753`); the `!causedShockwaves` second
/// ring adds 25 more with ONE each (the fadeIn `:69768` — the X kick
/// `:69767` is fixed). The rotations/colour are draw-only — skipped.
fn impact_explosion2_sleepy_octopod(p: &Projectile, rng: &mut dyn DropRng, _origin: Vec2, shocked: bool) {
    let _ = p;
    for _ in 0..25 {
        let _ = rng.next_float(); // `:69752`
        let _ = rng.next_float(); // `:69753`
    }
    if !shocked {
        for _ in 0..25 {
            let _ = rng.next_float(); // `:69768`
        }
    }
}

// Witnesses (W4-d15 code-only round — WRITTEN NOT RUN).

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::{MONK_STAFF_T1_EXPLOSION, proj_data};
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// A draw-counting `Main.rand` stand-in (the `projectile_ai_misc`
    /// CensusRng shape): `next_range` returns `min`, `next_float` 0,
    /// `next_double` 0 (counted separately — the NextVector2Square pair).
    struct CensusRng {
        ranges: Vec<(i32, i32)>,
        floats: usize,
        doubles: usize,
    }
    impl DropRng for CensusRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.ranges.push((min, max));
            min
        }
        fn next_float(&mut self) -> f32 {
            self.floats += 1;
            0.0
        }
        fn next_double(&mut self) -> f64 {
            self.doubles += 1;
            0.0
        }
    }
    fn census() -> CensusRng {
        CensusRng { ranges: Vec::new(), floats: 0, doubles: 0 }
    }

    fn world() -> World {
        World::new("t".into(), 300, 200)
    }

    fn proj(kind: u16) -> Projectile {
        let d = proj_data(kind).expect("the rowed kind");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(100.0 * 16.0, 50.0 * 16.0);
        p.w = d.w;
        p.h = d.h;
        p.tile_collide = d.tile_collide;
        p.alpha = d.alpha;
        p.scale = d.scale;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.max_penetrate = d.penetrate;
        p
    }

    fn put(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile { kind, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    /// The 698 row is verbatim `SetDefaults` (`Projectile.cs:7385-7401`).
    #[test]
    fn the_698_row_is_verbatim() {
        let d = proj_data(MONK_STAFF_T1_EXPLOSION).expect("the 698 row");
        assert_eq!((d.w, d.h), (75.0, 200.0), ":7387-7388");
        assert_eq!(d.ai_style, 141, ":7389");
        assert_eq!(d.alpha, 255, ":7390");
        assert!(!d.tile_collide, ":7391");
        assert!(d.ignore_water, ":7392");
        assert_eq!(d.time_left, 5, ":7393");
        assert_eq!(d.penetrate, -1, ":7395");
        assert!(d.uses_local_immunity, ":7396");
        assert_eq!(d.local_hit_cooldown, -1, ":7397");
        assert!(d.melee, ":7398");
    }

    /// Tick 1 over ONE exposed dirt tile: amount 3 × 8 = 24 → the first
    /// row 24×2 floats + the second row 23×1 = 71, plus the shocked ring
    /// 25×2 = 50 → 121 floats, zero ranges. The counter advances to 1.
    #[test]
    fn tick_one_over_one_tile_is_the_121_float_census() {
        let mut w = world();
        // The blast Bottom sits at (100*16 + 37.5, 52*16); the tile at
        // (102, 62) is inside the 75-px radius and exposed.
        put(&mut w, 102, 62, 1); // dirt — amount 3 (the fail table default)
        let mut p = proj(MONK_STAFF_T1_EXPLOSION);
        let mut rng = census();
        ai_monk_staff_t1_explosion(&mut p, &w, &mut rng);
        assert_eq!(p.ai[0], 1.0, ":69610");
        assert!(p.active);
        assert_eq!(rng.floats, 24 * 2 + 23 + 25 * 2, ":69705-69724 + :69739-69755");
        assert!(rng.ranges.is_empty());
    }

    /// No qualifying tile → `causedShockwaves = false` → BOTH rings:
    /// 50 + 25 = 75 floats.
    #[test]
    fn tick_one_in_open_air_draws_both_rings() {
        let w = world();
        let mut p = proj(MONK_STAFF_T1_EXPLOSION);
        let mut rng = census();
        ai_monk_staff_t1_explosion(&mut p, &w, &mut rng);
        assert_eq!(rng.floats, 75, ":69739-69770 — the unshocked double ring");
    }

    /// The counter kills past 20 (`:69611-69613`); ticks 2..20 draw
    /// nothing (the impact helpers run on tick 1 only).
    #[test]
    fn the_counter_kills_past_twenty_and_the_middle_ticks_are_draw_free() {
        let w = world();
        let mut p = proj(MONK_STAFF_T1_EXPLOSION);
        p.ai[0] = 19.0;
        let mut rng = census();
        ai_monk_staff_t1_explosion(&mut p, &w, &mut rng);
        assert!(p.active, "tick 20 stays live");
        assert_eq!(rng.floats, 0);
        ai_monk_staff_t1_explosion(&mut p, &w, &mut rng);
        assert!(!p.active, "tick 21 kills");
        assert_eq!(p.time_left, 0);
    }

    fn owner(anim: i32, anim_max: i32) -> Dd2Owner {
        Dd2Owner {
            anchor: Vec2::new(1600.0, 800.0),
            direction: 1,
            dead: false,
            frozen: false,
            item_animation: anim,
            item_animation_max: anim_max,
        }
    }

    /// The 699/708/709 rows are verbatim `SetDefaults`
    /// (`Projectile.cs:7402-7418`/`:7491-7507`/`:7508-7520`).
    #[test]
    fn the_monk_staff_rows_are_verbatim() {
        let d = proj_data(MONK_STAFF_T2).expect("the 699 row");
        assert_eq!((d.w, d.h, d.ai_style), (24.0, 24.0, 142), ":7404-7406");
        assert_eq!(d.penetrate, -1, ":7409");
        assert_eq!(d.alpha, 255, ":7410");
        assert!(!d.tile_collide && d.ignore_water, ":7413-7414");
        assert!(d.uses_id_static_immunity && !d.uses_local_immunity, ":7415");
        assert_eq!(d.id_static_hit_cooldown, 10, ":7416");
        assert!(d.melee, ":7408");
        let d = proj_data(MONK_STAFF_T3_ALT).expect("the 708 row");
        assert_eq!((d.w, d.h, d.ai_style), (24.0, 24.0, 142), ":7493-7495");
        assert!(d.uses_local_immunity && !d.uses_id_static_immunity, ":7504");
        assert_eq!(d.local_hit_cooldown, 6, ":7505");
        let d = proj_data(MONK_STAFF_T3_ALT_SHOT).expect("the 709 row");
        assert_eq!((d.w, d.h, d.ai_style), (12.0, 12.0, 1), ":7510-7512");
        assert_eq!(d.alpha, 255, ":7515");
        assert_eq!(d.extra_updates, 1, ":7517");
        assert_eq!(d.scale, 0.6, ":7519");
        assert!(d.melee, ":7514");
        assert!(d.ignore_water, ":7516");
    }

    /// The head block: the Centre pins on the anchor, the facing rides
    /// the owner, heldProj claims the slot; a dead owner kills ahead of
    /// the type arms (`:69819-69828`).
    #[test]
    fn the_head_block_pins_the_centre_and_the_dead_owner_kills() {
        let o = owner(20, 20);
        let mut p = proj(MONK_STAFF_T2);
        p.vel = Vec2::new(12.0, 0.0);
        let mut rng = census();
        let tick = ai_monk_staff_t2_and_3(&mut p, 7, &o, 0, &mut rng);
        assert_eq!(tick.held_proj, Some(7), ":69822");
        let c = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
        // The swing arc moves the position AFTER the pin; with ai[0] = 0
        // and anim == max (num = 0) the spin is π·(1,0)·(len,0) rotated
        // by 0 → (−12, 0), plus (len+22, 0) → (+34, 0): net +22 X.
        assert!((c.x - (o.anchor.x + 22.0)).abs() < 1e-3, "the arc kick");
        assert!((c.y - o.anchor.y).abs() < 1e-3);
        assert_eq!(p.direction, 1, ":69821");
        let mut dead_o = o;
        dead_o.dead = true;
        let mut p2 = proj(MONK_STAFF_T2);
        let mut rng2 = census();
        let tick2 = ai_monk_staff_t2_and_3(&mut p2, 3, &dead_o, 0, &mut rng2);
        assert!(!p2.active, ":69826");
        assert_eq!(tick2.held_proj, Some(3), "the claim precedes the kill");
        assert_eq!(rng2.floats, 0, "the dead arm draws nothing");
    }

    /// The 699 census: 2 trail floats + the Next(3) gate; the min-return
    /// census takes the gate → 2 more floats (scale + fadeIn). The
    /// animation end (<= 2, local owner) kills and arms reuseDelay 2.
    #[test]
    fn the_699_draw_census_and_the_animation_end_kill() {
        let o = owner(20, 20);
        let mut p = proj(MONK_STAFF_T2);
        p.alpha = 255;
        p.local_ai[0] = 5.0;
        p.vel = Vec2::new(12.0, 0.0);
        let mut rng = census();
        let tick = ai_monk_staff_t2_and_3(&mut p, 0, &o, 0, &mut rng);
        assert_eq!(rng.floats, 4, ":69865 ×2 + :69876-69877");
        assert_eq!(rng.ranges, vec![(0, 3)], ":69871");
        assert_eq!(p.alpha, 128, ":69835-69839");
        assert_eq!(p.local_ai[0], 4.0, ":69840-69843");
        assert!(p.active);
        assert!(tick.reuse_delay.is_none());
        // The animation-end tick.
        let end_o = owner(2, 20);
        let tick = ai_monk_staff_t2_and_3(&mut p, 0, &end_o, 0, &mut rng);
        assert!(!p.active, ":69965");
        assert_eq!(tick.reuse_delay, Some(2), ":69966");
    }

    /// The frozen owner holds the pose: no type-arm writes, no draws,
    /// but the head block + the animation-end kill still run.
    #[test]
    fn the_frozen_owner_holds_the_swing_pose() {
        let mut o = owner(20, 20);
        o.frozen = true;
        let mut p = proj(MONK_STAFF_T2);
        p.alpha = 255;
        let mut rng = census();
        ai_monk_staff_t2_and_3(&mut p, 0, &o, 0, &mut rng);
        assert_eq!(p.alpha, 255, "the fade is inside !frozen");
        assert_eq!(rng.floats, 0);
        assert!(rng.ranges.is_empty());
    }

    /// The 708 volley at animation 2: NextVector2Square = 2 doubles,
    /// Next(4) speed, the 709 spawn at Centre with the rotated heading
    /// (×10 at the census's min return), then the two Next(6) dust gates
    /// (min ≠ 0 → both taken: 1 float + 1 Next(3) each).
    #[test]
    fn the_708_volley_draw_order_and_spawn() {
        let o = owner(2, 20);
        let mut p = proj(MONK_STAFF_T3_ALT);
        p.damage = 42;
        p.vel = Vec2::new(12.0, 0.0);
        let mut rng = census();
        let tick = ai_monk_staff_t2_and_3(&mut p, 0, &o, 0, &mut rng);
        assert_eq!(rng.doubles, 2, ":69909 — NextVector2Square(-0.2, 0.2)");
        assert_eq!(
            rng.ranges,
            vec![(0, 4), (0, 6), (0, 3), (0, 6), (0, 3)],
            ":69923 then the k=0/2 gates :69944/:69954"
        );
        assert_eq!(rng.floats, 2, ":69948 ×2");
        assert_eq!(tick.shots.len(), 1, ":69924");
        let s = tick.shots[0];
        assert_eq!(s.kind, MONK_STAFF_T3_ALT_SHOT);
        assert_eq!(s.damage, 42);
        assert_eq!(s.knock_back, 0.0);
        // vector6 = (1, 0) rotated by +0.3839724659919739, ×10.
        let a = 0.3839724659919739f64;
        assert!((s.vx - 10.0 * a.cos() as f32).abs() < 1e-3);
        assert!((s.vy - 10.0 * a.sin() as f32).abs() < 1e-3);
        // The animation-end kill fires on the same tick (anim 2 <= 2).
        assert!(!p.active);
        assert_eq!(tick.reuse_delay, Some(2));
    }

    /// A foreign-owned 708 skips the volley (the `owner == Main.myPlayer`
    /// gate, `:69907`) but keeps the dust gates; no reuseDelay write.
    #[test]
    fn the_foreign_owner_skips_the_volley_and_the_reuse_write() {
        let o = owner(2, 20);
        let mut p = proj(MONK_STAFF_T3_ALT);
        p.owner = 3;
        p.vel = Vec2::new(12.0, 0.0);
        let mut rng = census();
        let tick = ai_monk_staff_t2_and_3(&mut p, 0, &o, 0, &mut rng);
        assert!(tick.shots.is_empty(), ":69907");
        assert_eq!(rng.doubles, 0);
        assert_eq!(rng.ranges, vec![(0, 6), (0, 3), (0, 6), (0, 3)]);
        assert!(p.active, ":69963 — the local-owner gate");
        assert!(tick.reuse_delay.is_none());
    }
}
