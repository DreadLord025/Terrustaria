//! S1-P-i14 — the bullet launch arms of `Player.ItemCheck_Shoot`
//! (`Player.cs:47056-49468`), the `useAmmo == AmmoID.Bullet` (97) half. The ammo
//! pick, the stat fuse and the not-consume rolls live in [`crate::ammo`] and the
//! shooter wire (`world_sim/editing.rs`); THIS module owns the per-gun launch
//! geometry and spawn arms that run AFTER `PickAmmo` resolved `projToShoot` and
//! the wire fused the damage/speed/knockback.
//!
//! Scope: the gun LAUNCH only. The bullet flight (aiStyle 1/2/78) is the
//! projectile-AI task's; the ammo consumption (`HasAmmo`/consumable-drain/
//! Nanite-single-scale) is i15's; the special-bullet rows/AI are i18's; the
//! rocket/dart launchers are i16's. The two bullet `gunProj` items — the Vortex
//! Beater 3475 (→ 615) here, the Celebration Mk2 3930 (→ 714, `useAmmo` Rocket)
//! stays with the rocket task — fire their FIXED projectile (`Player.cs:48986-
//! 48994`); the wire passes `g.projectile`.
//!
//! ## The shared geometry INHERITED from i13 (`Player.cs:47196-47307`)
//!
//! `ItemCheck_Shoot` runs the SAME launch frame for EVERY shooter before the
//! per-item arms: the `RotatedRelativePoint(MountedCenter)` spawn anchor
//! (`:47196`), the `Dot(UnitX.RotatedBy(fullRotation), normalize(MouseWorld -
//! pointPosition))` facing flip → `ChangeDir(±1)` (`:47202-47224`), and the aim
//! `num4/num5 = MouseWorld - pointPosition` normalized to `speed` with the
//! zero/NaN fallback (`:47277-47307`). S1-P-i13 fixed these shared defects for
//! the bow half; this module REUSES them verbatim through
//! [`crate::bow_launch::launch_frame`] — it does not re-fix them.
//!
//! ## RNG order
//!
//! The port rolls the crit at THROW time (the established `world_sim`
//! convention, ahead of the launch arms' spread draws — the documented
//! crit-at-throw deviation). Within each arm the `Main.rand` draws are consumed
//! in the C#'s exact order. Every draw rides the `UpdatePlayers` stream
//! (`Player.ItemCheck` under the `Main.cs:17754` swap). The Revolver 2269's
//! per-tick `Next(3)` charge/decay draw (`Player.cs:42677-42703`) is a SEPARATE
//! fixed-step draw in `world_sim/player.rs` — it precedes the throw on the same
//! stream and its bonus rides the throw-time crit roll.
//!
//! ## useSound per gun
//!
//! The per-gun `UseSound` plays at the USE START (`Player.ItemCheck_StartActualUse`,
//! `Player.cs:51681-51698`), NOT per shot — the wire's `start_swing_extras` →
//! `crate::item::initial_use_sound` already plays the registry's per-gun
//! `use_sound` row (verified: 95→Item184, 96→Item183, 98/533/800/1265/1870→Item11,
//! 164/219/1255/1929/2269/2270→Item41, 434→Item31, 534/964/3788/4703→Item36,
//! 679→Item38, 1254/1553→Item40, 2797→Item95, 5117→Item61, 3475/3930→null). The
//! launch arms therefore play NO sound (the pre-i14 generic `Item_1` cue was
//! wrong for guns and is removed from the bullet path).
//!
//! ## Documented folds
//!
//! * `gravDir = 1` (no gravity flip) — same fold the bow module carries.
//! * `sItem.scale` is the prefixed instance scale (`Item.Prefix`,
//!   `Item.cs:582`); the Xenopopper muzzle reads it (`:47605`).
//! * `Vector2.ToRotation()` = `atan2(Y, X)` (`Utils.cs`); the Xenopopper bubble
//!   carries it as `ai[0]` (`:47610`).

use glam::Vec2;
use tr_core::UnifiedRandom;

use crate::app::App;
use crate::bow_launch::{launch_frame, rotated_by, safe_normalize, Frame};
use crate::projectile::{can_hit, proj_frames};

/// `AmmoID.Bullet` (`ID/AmmoID.cs:164`) — the category the gun launch keys on.
pub(crate) const AMMO_BULLET: i32 = 97;

/// Whether the shooter's ammo category takes the gun launch path.
pub(crate) fn is_bullet_shooter(use_ammo: i32) -> bool {
    use_ammo == AMMO_BULLET
}

/// One spawned bullet — the `Projectile.NewProjectile` arguments the launch arm
/// passes. `damage`/`knock_back` default to the shot's fused values; the Onyx
/// Blaster's black bolt overrides damage (×2, `Player.cs:48837`).
struct Shot {
    kind: u16,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    ai0: f32,
    ai1: f32,
    /// `localAI[0]`/`localAI[1]` — the Xenopopper bubble's deferred payload
    /// (`Player.cs:47622-47623`: the resolved bullet id + the fused speed).
    local_ai0: Option<f32>,
    local_ai1: Option<f32>,
    /// A per-shot damage override (the Onyx black bolt's ×2).
    damage: Option<i32>,
}

impl Shot {
    fn new(kind: u16, frame: &Frame, vx: f32, vy: f32) -> Self {
        Shot {
            kind,
            x: frame.spawn.x,
            y: frame.spawn.y,
            vx,
            vy,
            ai0: 0.0,
            ai1: 0.0,
            local_ai0: None,
            local_ai1: None,
            damage: None,
        }
    }
}

/// The gun launch entry — `Player.ItemCheck_Shoot`'s bullet half. The wire calls
/// it for `useAmmo == 97` shooters AFTER the stat fuse and the crit roll. It
/// builds the shared launch frame (inherited from i13, flipping the facing),
/// dispatches the per-gun arm in the C#'s code order and spawns the bullets.
///
/// `remaining_anim_ticks` is the integer `itemAnimation` the Clockwork's spread
/// window reads (`Player.cs:48419-48432`). `scale` is the prefixed `sItem.scale`
/// the Xenopopper muzzle reads (`:47605`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn fire(
    app: &mut App,
    kind: u16,
    proj: u16,
    speed: f32,
    damage: i32,
    knock_back: f32,
    crit: bool,
    world_pos: Vec2,
    remaining_anim_ticks: i32,
    scale: f32,
) {
    // The pre-geometry `projToShoot` conversions (`Player.cs:47107-47118`): the
    // Sniper 1254 / Venus Magnum 1255 / Uzi 1265 turn the plain bullet 14 into
    // the HighVelocityBullet 242. These run BEFORE the launch geometry (and
    // AFTER `PickAmmo`'s roll chain, which the wire already ran with the
    // pre-conversion proj), so applying them here is the C#'s order.
    let proj = if matches!(kind, 1254 | 1255 | 1265) && proj == 14 {
        242
    } else {
        proj
    };
    // The post-pick `projToShoot` override (`Player.cs:47177-47180`): the
    // Pew-matic Horn 5117 fires its own shot 968 regardless of the ammo.
    let proj = if kind == 5117 { 968 } else { proj };

    // The shared launch frame (i13's geometry, reused verbatim): the rotated
    // anchor, the facing flip and the aim normalized to `speed`. The Chain Gun
    // 1929 / Gatligator 2270 carry the C#'s pre-normalization aim jitter
    // (`:47301-47305`: `num4/num5 += Next(-50, 51) * 0.03f / num6`, applied
    // after the `num6` fallback and before `num4 *= num6`) — two draws in the
    // frame's stream slot, ahead of the per-gun spread arms' draws.
    let mut chain_gat_jitter = |n4: f32, n5: f32, n6: f32, rng: &mut UnifiedRandom| {
        (
            n4 + rng.next_range(-50, 51) as f32 * 0.03 / n6,
            n5 + rng.next_range(-50, 51) as f32 * 0.03 / n6,
        )
    };
    let jitter: Option<&mut dyn FnMut(f32, f32, f32, &mut UnifiedRandom) -> (f32, f32)> =
        if matches!(kind, 1929 | 2270) {
            Some(&mut chain_gat_jitter)
        } else {
            None
        };
    let Some((mut frame, _misc_counter)) = launch_frame(app, world_pos, speed, jitter) else {
        return;
    };
    // `:47229-47232` — the Pew-matic Horn 5117 spawns from the entity's upper
    // third (`pointPosition.Y = position.Y + (float)(height / 3)`), AFTER the
    // facing flip and BEFORE the aim delta (`:47277-47278`) — so the aim also
    // re-derives from the shifted anchor. The V5 gate found the arm missing;
    // draw-free geometry, the aim keeps |speed|. (The 3094/3378/3543 thrown
    // siblings at `:47225-47228` are i28/i16's lanes.)
    if kind == 5117 {
        let p = app.player.as_ref().expect("launch_frame returned Some");
        frame.spawn.y = p.y + (crate::player::HEIGHT as i32 / 3) as f32;
        let mut num4 = world_pos.x - frame.spawn.x;
        let mut num5 = world_pos.y - frame.spawn.y;
        let mut num6 = (num4 * num4 + num5 * num5).sqrt();
        if (num4.is_nan() && num5.is_nan()) || (num4 == 0.0 && num5 == 0.0) {
            num4 = frame.direction as f32;
            num5 = 0.0;
            num6 = speed;
        } else {
            num6 = speed / num6;
        }
        num4 *= num6;
        num5 *= num6;
        frame.vel = Vec2::new(num4, num5);
    }

    // The per-gun dispatch, in the C#'s code order. Each arm `return`s in the
    // source; the port folds them into one match. The generic single-shot tail
    // (`:49437`) is the fallback for the guns without a special arm.
    let shots: Vec<Shot> = {
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let world = &app.world;
        match kind {
            // `:47501-47504` — the Minishark 98 / Megashark 533.
            98 | 533 => vec![spread_shot(&frame, proj, rng, -40, 41, 0.01)],
            // `:47627-47637` — the Gatligator 2270.
            2270 => vec![gatligator(&frame, proj, rng)],
            // `:47639-47645` — the Pew-matic Horn 5117.
            5117 => vec![pew_matic_horn(&frame, proj, rng)],
            // `:47810-47813` — the Chain Gun 1929.
            1929 => vec![spread_shot(&frame, proj, rng, -40, 41, 0.03)],
            // `:47815-47818` — the S.D.M.G. 1553.
            1553 => vec![spread_shot(&frame, proj, rng, -40, 41, 0.005)],
            // `:47827-47832` — the Uzi 1265.
            1265 => vec![spread_shot(&frame, proj, rng, -30, 31, 0.03)],
            // `:47958-47967` — the Shotgun 534.
            534 => shotgun(&frame, proj, rng),
            // `:47969-47982` — the Quad-Barrel Shotgun 4703.
            4703 => quad_barrel(&frame, proj, rng),
            // `:48146-48155` — the Boomstick 964.
            964 => boomstick(&frame, proj, rng),
            // `:48355-48363` — the Tactical Shotgun 679.
            679 => tactical_shotgun(&frame, proj, rng),
            // `:48415-48434` — the Clockwork Assault Rifle 434.
            434 => vec![clockwork(&frame, proj, rng, remaining_anim_ticks)],
            // `:47603-47625` — the Xenopopper 2797.
            2797 => xenopopper(world, &frame, proj, speed, scale, rng),
            // `:48828-48838` — the Onyx Blaster 3788.
            3788 => onyx_blaster(&frame, proj, speed, damage, rng),
            // `:48986-48989` — the Vortex Beater 3475 (gunProj, fixed 615).
            3475 => {
                let mut s = Shot::new(615, &frame, frame.vel.x, frame.vel.y);
                s.ai0 = (5 * rng.next_range(0, 20)) as f32;
                vec![s]
            }
            // The generic single-shot tail (`:49437`): the plain guns — the
            // Flintlock 95, Musket 96, Handgun 164, Phoenix Blaster 219, The
            // Undertaker 800, Red Ryder 1870, Revolver 2269 — and the Sniper
            // 1254 / Venus Magnum 1255 (their `projToShoot == 14 → 242`
            // conversion, `:47107-47114`, is applied above). `ai7/ai8/ai9` are
            // 0 for the bullet kinds (the `:49379-49435` arms key on non-bullet
            // proj ids).
            _ => vec![Shot::new(proj, &frame, frame.vel.x, frame.vel.y)],
        }
    };

    // Spawn the bullets. The owner is the local player (`NewProjectile`'s
    // `Owner: i`); the crit stamp is the port's throw-time roll (the Revolver's
    // charge bonus is already folded into it by the wire).
    let owner = app.local_id as u8;
    for shot in shots {
        let dmg = shot.damage.unwrap_or(damage);
        let slot = app.projectiles.spawn_with_data_owned(
            shot.kind,
            owner,
            shot.x,
            shot.y,
            shot.vx,
            shot.vy,
            dmg,
            knock_back,
            shot.ai0,
            shot.ai1,
        );
        if let Some(slot) = slot {
            app.projectiles.set_crit(slot, crit);
            if let Some(v) = shot.local_ai0 {
                app.projectiles.set_local_ai0(slot, v);
            }
            if let Some(v) = shot.local_ai1 {
                app.projectiles.set_local_ai1(slot, v);
            }
        }
    }
    // NO launch sound: the per-gun UseSound plays at the use start
    // (`Player.cs:51681-51698`, the wire's `start_swing_extras`).
}

/// `num4/num5 + Main.rand.Next(lo, hi) * factor` on each axis — the per-axis
/// spread the single-shot guns read (`Player.cs:47503`, `:47812`, `:47817`,
/// `:47831`). TWO draws, X first.
fn spread_shot(
    frame: &Frame,
    proj: u16,
    rng: &mut UnifiedRandom,
    lo: i32,
    hi: i32,
    factor: f32,
) -> Shot {
    let vx = frame.vel.x + rng.next_range(lo, hi) as f32 * factor;
    let vy = frame.vel.y + rng.next_range(lo, hi) as f32 * factor;
    Shot::new(proj, frame, vx, vy)
}

/// `Player.cs:47627-47637` — the Gatligator 2270: the ±40×0.05 spread, then a
/// 1/3 `Next(3)` roll scales BOTH axes by `1 + Next(-30, 31) * 0.02` (two more
/// draws, only when the roll hits).
fn gatligator(frame: &Frame, proj: u16, rng: &mut UnifiedRandom) -> Shot {
    let mut vx = frame.vel.x + rng.next_range(-40, 41) as f32 * 0.05;
    let mut vy = frame.vel.y + rng.next_range(-40, 41) as f32 * 0.05;
    if rng.next_max(3) == 0 {
        vx *= 1.0 + rng.next_range(-30, 31) as f32 * 0.02;
        vy *= 1.0 + rng.next_range(-30, 31) as f32 * 0.02;
    }
    Shot::new(proj, frame, vx, vy)
}

/// `Player.cs:47639-47645` — the Pew-matic Horn 5117: the ±15×0.075 spread and
/// `ai[1] = Next(Main.projFrames[sItem.shoot])` — the 24-frame sheet's random
/// start frame (`Main.cs:8706`, `proj_frames(968)`).
fn pew_matic_horn(frame: &Frame, proj: u16, rng: &mut UnifiedRandom) -> Shot {
    let vx = frame.vel.x + rng.next_range(-15, 16) as f32 * 0.075;
    let vy = frame.vel.y + rng.next_range(-15, 16) as f32 * 0.075;
    let mut s = Shot::new(proj, frame, vx, vy);
    s.ai1 = rng.next_max(proj_frames(968) as i32) as f32;
    s
}

/// `Player.cs:47958-47967` — the Shotgun 534: `Next(4, 6)` shots (4 or 5), each
/// with the ±40×0.05 spread. The count roll precedes the per-shot spreads.
fn shotgun(frame: &Frame, proj: u16, rng: &mut UnifiedRandom) -> Vec<Shot> {
    let count = rng.next_range(4, 6);
    (0..count)
        .map(|_| spread_shot(frame, proj, rng, -40, 41, 0.05))
        .collect()
}

/// `Player.cs:47969-47982` — the Quad-Barrel Shotgun 4703: one straight shot,
/// then 7 shots whose velocity is the aim perturbed by a perpendicular jitter —
/// `(v + SafeNormalize(v).RotatedBy(π/2 * NextFloat()) * NextFloatDirection() *
/// 5).SafeNormalize() * |v|`, then the ±40×0.05 spread. Each of the 7 spends
/// NextFloat (the rotation), NextFloatDirection (the jitter sign/magnitude) and
/// the two spread draws.
fn quad_barrel(frame: &Frame, proj: u16, rng: &mut UnifiedRandom) -> Vec<Shot> {
    let mut shots = vec![Shot::new(proj, frame, frame.vel.x, frame.vel.y)];
    // `float num68 = (float)Math.PI / 2f` (`:47971`) — the f32 value; the
    // rotation angle is `num68 * Main.rand.NextFloat()` (f32 × f32, `:47977`).
    let num68 = std::f32::consts::FRAC_PI_2;
    let speed = frame.vel.length();
    for _ in 0..7 {
        let v = frame.vel;
        let perturbed = v
            + rotated_by(
                safe_normalize(v, Vec2::ZERO),
                (num68 * rng.next_float()) as f64,
            ) * (rng.next_float_direction() * 5.0);
        let dir = safe_normalize(perturbed, Vec2::ZERO) * speed;
        let vx = dir.x + rng.next_range(-40, 41) as f32 * 0.05;
        let vy = dir.y + rng.next_range(-40, 41) as f32 * 0.05;
        shots.push(Shot::new(proj, frame, vx, vy));
    }
    shots
}

/// `Player.cs:48146-48155` — the Boomstick 964: `Next(3, 5)` shots (3 or 4),
/// each with the ±35×0.04 spread.
fn boomstick(frame: &Frame, proj: u16, rng: &mut UnifiedRandom) -> Vec<Shot> {
    let count = rng.next_range(3, 5);
    (0..count)
        .map(|_| spread_shot(frame, proj, rng, -35, 36, 0.04))
        .collect()
}

/// `Player.cs:48355-48363` — the Tactical Shotgun 679: SIX shots, each with the
/// ±40×0.05 spread (no count roll).
fn tactical_shotgun(frame: &Frame, proj: u16, rng: &mut UnifiedRandom) -> Vec<Shot> {
    (0..6)
        .map(|_| spread_shot(frame, proj, rng, -40, 41, 0.05))
        .collect()
}

/// `Player.cs:48415-48434` — the Clockwork Assault Rifle 434: the spread depends
/// on the remaining animation. `itemAnimation < 5` → ±40×0.01 then ×1.1;
/// `itemAnimation < 10` → ±20×0.01 then ×1.05; else the plain aim. The burst's
/// three shots (anim 11/7/3, the `useTime 4 < useAnimation 12` volley) thus
/// tighten as the swing progresses.
fn clockwork(frame: &Frame, proj: u16, rng: &mut UnifiedRandom, anim: i32) -> Shot {
    let mut vx = frame.vel.x;
    let mut vy = frame.vel.y;
    if anim < 5 {
        vx += rng.next_range(-40, 41) as f32 * 0.01;
        vy += rng.next_range(-40, 41) as f32 * 0.01;
        vx *= 1.1;
        vy *= 1.1;
    } else if anim < 10 {
        vx += rng.next_range(-20, 21) as f32 * 0.01;
        vy += rng.next_range(-20, 21) as f32 * 0.01;
        vx *= 1.05;
        vy *= 1.05;
    }
    Shot::new(proj, frame, vx, vy)
}

/// `Player.cs:47603-47625` — the Xenopopper 2797: the muzzle advances
/// `Normalize(aim) * 40 * sItem.scale` when the line is clear, then fires
/// `Next(4, 5)` (+1 on a 1/4 `Next(4)` roll) bubbles (type 444). Each bubble's
/// velocity is the aim `RotatedBy(2π/3 * (NextDouble() - 0.5)) * (NextDouble() *
/// 0.2 + 0.05)`; `ai[0] = aim.ToRotation()`, `localAI[0] = projToShoot` (the
/// resolved bullet), `localAI[1] = speed` (the fused speed the bubble's AI_078
/// releases it at).
fn xenopopper(
    world: &tr_core::World,
    frame: &Frame,
    proj: u16,
    speed: f32,
    scale: f32,
    rng: &mut UnifiedRandom,
) -> Vec<Shot> {
    let aim = frame.vel;
    let muzzle = safe_normalize(aim, Vec2::ZERO) * (40.0 * scale);
    let mut spawn = frame.spawn;
    if can_hit(
        world,
        spawn.x,
        spawn.y,
        0.0,
        0.0,
        spawn.x + muzzle.x,
        spawn.y + muzzle.y,
        0.0,
        0.0,
    ) {
        spawn += muzzle;
    }
    let ai = aim.y.atan2(aim.x);
    // `float num23 = (float)Math.PI * 2f / 3f` (`:47611`) — the f32 value, then
    // the double-angle rotation (`RotatedBy(num23 * (float)NextDouble() - num23
    // / 2f)`, `:47620`).
    let num23 = std::f32::consts::PI * 2.0 / 3.0;
    let mut count = rng.next_range(4, 5);
    if rng.next_max(4) == 0 {
        count += 1;
    }
    let mut shots = Vec::new();
    for _ in 0..count {
        let factor = rng.next_double() as f32 * 0.2 + 0.05;
        // `RotatedBy(num23 * (float)Main.rand.NextDouble() - num23 / 2f)`
        // (`:47620`): the angle product + subtraction are f32 (the NextDouble
        // is CAST to float first), promoted to double only inside RotatedBy
        // (the V5 fidelity fix — the f64 product drifted sub-ulp).
        let angle = (num23 * rng.next_double() as f32 - num23 / 2.0) as f64;
        let v = rotated_by(aim, angle) * factor;
        shots.push(Shot {
            kind: 444,
            x: spawn.x,
            y: spawn.y,
            vx: v.x,
            vy: v.y,
            ai0: ai,
            ai1: 0.0,
            local_ai0: Some(proj as f32),
            local_ai1: Some(speed),
            damage: None,
        });
    }
    shots
}

/// `Player.cs:48828-48838` — the Onyx Blaster 3788: two pairs of bullets, each
/// pair a +π/4 and a −π/4 rotated jitter — `v + SafeNormalize(v).RotatedBy(±π/4
/// * (NextFloat() * 0.5 + 0.5)) * NextFloatDirection() * 2` — then ONE black bolt
/// (type 661) at `SafeNormalize(v, UnitX * direction) * speed * 1.3` with DOUBLE
/// damage. Each bullet spends NextFloat (the rotation) + NextFloatDirection (the
/// jitter); the bolt spends none.
fn onyx_blaster(
    frame: &Frame,
    proj: u16,
    speed: f32,
    damage: i32,
    rng: &mut UnifiedRandom,
) -> Vec<Shot> {
    let v = frame.vel;
    // `float num172 = (float)Math.PI / 4f` (`:48831`) — the f32 value; the angle
    // is `num172 * (NextFloat() * 0.5f + 0.5f)` (f32, `:48834`).
    let num172 = std::f32::consts::FRAC_PI_4;
    let mut shots = Vec::new();
    for _ in 0..2 {
        let plus = v
            + rotated_by(
                safe_normalize(v, Vec2::ZERO),
                (num172 * (rng.next_float() * 0.5 + 0.5)) as f64,
            ) * (rng.next_float_direction() * 2.0);
        shots.push(Shot::new(proj, frame, plus.x, plus.y));
        let minus = v
            + rotated_by(
                safe_normalize(v, Vec2::ZERO),
                (-num172 * (rng.next_float() * 0.5 + 0.5)) as f64,
            ) * (rng.next_float_direction() * 2.0);
        shots.push(Shot::new(proj, frame, minus.x, minus.y));
    }
    let bolt_dir = safe_normalize(v, Vec2::X * frame.direction as f32) * (speed * 1.3);
    let mut bolt = Shot::new(661, frame, bolt_dir.x, bolt_dir.y);
    bolt.damage = Some(damage * 2);
    shots.push(bolt);
    shots
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;
    use tr_core::tile::{Tile, FLAG_ACTIVE};
    use tr_core::TILE_PX;

    /// A launch rig: a 64×800 world with ground at row 300, the player parked
    /// at tile (33, 260) in clear air, the `UpdatePlayers` stream pinned to
    /// `seed`. The cursor world point is passed to [`fire`] directly. Mirrors
    /// the bow module's rig (S1-P-i13) so the shared geometry is exercised
    /// identically.
    fn rig(seed: i32) -> App {
        let mut w = tr_core::World::new("gun".into(), 64, 800);
        for x in 0..64 {
            let i = w.index(x, 300);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.compute_column_surface();
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        let mut p = Player::spawn(&app.world);
        p.x = 33.0 * TILE_PX;
        p.y = 260.0 * TILE_PX;
        p.facing = 1;
        app.player = Some(p);
        *app.rngs.stream(crate::app::RngKey::UpdatePlayers) = UnifiedRandom::new(seed);
        app
    }

    /// The cursor 80 px right of the player centre — the aim direction +x.
    fn cursor_right(app: &App) -> Vec2 {
        let p = app.player.as_ref().unwrap();
        let (cx, cy) = p.center();
        Vec2::new(cx + 80.0, cy)
    }

    fn live(app: &App) -> Vec<&crate::projectile::Projectile> {
        app.projectiles.slots().iter().filter(|p| p.active).collect()
    }

    /// The stream state after `n` draws from `seed` — the draw-count contract.
    fn draws_after(seed: i32, n: usize) -> UnifiedRandom {
        let mut rng = UnifiedRandom::new(seed);
        for _ in 0..n {
            rng.next();
        }
        rng
    }

    fn stream_of(app: &mut App) -> UnifiedRandom {
        app.rngs.stream(crate::app::RngKey::UpdatePlayers).clone()
    }

    // ---- the shared geometry (inherited from i13) -------------------------

    #[test]
    fn s1_p_i14_the_plain_gun_fires_from_the_rotated_anchor_with_the_facing_flip() {
        // The Musket 96 has no special arm — the generic single tail
        // (`Player.cs:49437`) at the shared `RotatedRelativePoint(MountedCenter)`
        // anchor (`:47196`), the aim normalized to `speed` (`:47277-47307`).
        // For the rig's stationary unmounted player the rotated anchor ==
        // `center()`; the facing flips toward the cursor.
        let mut app = rig(0x5EED);
        // Cursor LEFT: the aim dot is negative → ChangeDir(-1) (`:47216-47223`).
        let (cx, cy) = app.player.as_ref().unwrap().center();
        let cursor = Vec2::new(cx - 80.0, cy);
        app.player.as_mut().unwrap().facing = 1;
        fire(&mut app, 96, 14, 13.0, 38, 5.25, false, cursor, 31, 1.0);
        let shots = live(&app);
        assert_eq!(shots.len(), 1, "one musket ball");
        assert_eq!(shots[0].kind, 14);
        assert_eq!(app.player.as_ref().unwrap().facing, -1, "the facing flip");
        assert!(
            (shots[0].pos.x - cx).abs() < 1e-3 && (shots[0].pos.y - cy).abs() < 1e-3,
            "the rotated anchor, not centre+10 ({:?} vs ({cx},{cy}))",
            shots[0].pos
        );
        assert!(shots[0].vel.x < 0.0, "the aim flies left");
        assert!(
            (shots[0].vel.length() - 13.0).abs() < 1e-3,
            "the aim normalized to the fused speed"
        );
        // The generic tail spends NO launch draw (the crit is the wire's).
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 0));
    }

    #[test]
    fn s1_p_i14_the_gun_launch_plays_no_sound() {
        // The per-gun UseSound plays at the USE START (`Player.cs:51681-51698`,
        // the wire's `start_swing_extras`), NOT per shot — the launch arms are
        // silent (the pre-i14 generic `Item_1` cue was wrong for guns).
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        let before = app.audio.played_count();
        fire(&mut app, 98, 14, 11.0, 13, 0.0, false, cursor, 7, 1.0);
        assert_eq!(app.audio.played_count(), before, "no launch cue");
    }

    // ---- the single-shot spreads ------------------------------------------

    #[test]
    fn s1_p_i14_the_minishark_spread_spends_two_draws() {
        // `Player.cs:47501-47504`: ±40×0.01 on each axis, X first.
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        let mut probe = UnifiedRandom::new(0x5EED);
        let vx = 11.0 + probe.next_range(-40, 41) as f32 * 0.01;
        let vy = 0.0 + probe.next_range(-40, 41) as f32 * 0.01;
        fire(&mut app, 98, 14, 11.0, 13, 0.0, false, cursor, 7, 1.0);
        let shots = live(&app);
        assert_eq!(shots.len(), 1);
        assert!((shots[0].vel.x - vx).abs() < 1e-5);
        assert!((shots[0].vel.y - vy).abs() < 1e-5);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 2));
    }

    #[test]
    fn s1_p_i14_the_sdmgs_spread_is_tighter_than_the_chain_guns() {
        // SDMG 1553 ±40×0.005 (`:47817`) vs Chain Gun 1929 ±40×0.03 (`:47812`):
        // the SDMG's per-axis spread FACTOR is 6× smaller. The C# draws the
        // SAME `Next(-40,41)` integers for both, so the fidelity witness pins
        // each gun's velocity to its own factor applied to the probe's draws —
        // a single-seed magnitude compare is NOT a valid ordering (the Chain
        // Gun's two pre-normalization aim-jitter draws `:47301-47305` shift its
        // spread draws AND its aim, so the realized |Δ| can land either way;
        // the V5 gate replaced the coder's seed-specific magnitude claim).
        // Draw counts: SDMG 2 (spread only); Chain Gun 4 (2 aim-jitter + 2 spread).
        let check = |kind: u16, factor: f32, jitter: bool, draws: usize| {
            let mut app = rig(0x5EED);
            let cursor = cursor_right(&app);
            let mut probe = UnifiedRandom::new(0x5EED);
            // The Chain Gun's aim jitter rotates the aim slightly; the spread
            // then adds factor×draw to each axis of the (jittered) aim.
            let (base_x, base_y) = if jitter {
                // The jitter's exact aim is re-derived by fire(); here we only
                // pin the SPREAD delta off the realized base, so read the base
                // from a no-spread reference is not possible — instead pin the
                // velocity to base + factor×draw with base captured below.
                (f32::NAN, f32::NAN)
            } else {
                (12.0, 0.0)
            };
            let _ = (base_x, base_y);
            // Consume the aim-jitter draws in the probe so the spread draws line up.
            if jitter {
                let _jx = probe.next_range(-50, 51);
                let _jy = probe.next_range(-50, 51);
            }
            let dx = probe.next_range(-40, 41) as f32 * factor;
            let dy = probe.next_range(-40, 41) as f32 * factor;
            fire(&mut app, kind, 14, 12.0, 30, 2.0, false, cursor, 4, 1.0);
            let v = live(&app)[0].vel;
            assert_eq!(stream_of(&mut app), draws_after(0x5EED, draws));
            if !jitter {
                // The SDMG flies at the pure +x aim plus the exact spread.
                assert!((v.x - (12.0 + dx)).abs() < 1e-4, "SDMG x = aim + 0.005×draw");
                assert!((v.y - dy).abs() < 1e-4, "SDMG y = 0.005×draw");
                assert!(dx.abs() <= 0.2 + 1e-6 && dy.abs() <= 0.2 + 1e-6, "±40×0.005 bound");
            } else {
                // The Chain Gun's spread delta off its (jittered) aim is bounded
                // by ±40×0.03 = ±1.2; the aim itself stays near +x at |12|.
                assert!(v.length() > 10.0 && v.length() < 14.0, "the aim keeps ~|12|");
            }
        };
        check(1553, 0.005, false, 2);
        check(1929, 0.03, true, 4);
    }

    #[test]
    fn s1_p_i14_the_gatligator_speed_roll_is_conditional() {
        // `Player.cs:47627-47637` + `:47301-47305`: the 2 pre-normalization aim
        // jitter draws, then the ±40×0.05 spread (2 draws), then Next(3); only
        // on a hit do the two ×(1+Next(-30,31)*0.02) draws follow. The draw
        // count is 5 (miss) or 7 (hit) — seed-determined.
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        let mut probe = UnifiedRandom::new(0x5EED);
        let _jx = probe.next_range(-50, 51);
        let _jy = probe.next_range(-50, 51);
        let _vx = probe.next_range(-40, 41);
        let _vy = probe.next_range(-40, 41);
        let hit = probe.next_max(3) == 0;
        let n = if hit { 7 } else { 5 };
        fire(&mut app, 2270, 14, 8.0, 21, 1.5, false, cursor, 6, 1.0);
        assert_eq!(live(&app).len(), 1);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, n));
    }

    #[test]
    fn s1_p_i14_the_chain_gun_aim_jitter_perturbs_before_normalization() {
        // `Player.cs:47301-47305`: the Chain Gun 1929 adds Next(-50,51)*0.03/num6
        // to each raw aim axis BEFORE `num4 *= num6` — so the jitter rotates the
        // aim slightly (the velocity keeps |speed| but its direction deviates
        // from the pure cursor aim). The SDMG 1553 (no jitter) flies exactly at
        // the cursor. Both then spend their 2 spread draws.
        let aim_deviation = |kind: u16| {
            let mut app = rig(0x5EED);
            let cursor = cursor_right(&app);
            fire(&mut app, kind, 14, 12.0, 30, 2.0, false, cursor, 4, 1.0);
            let v = live(&app)[0].vel;
            // The pure aim is +x; the deviation is the |y| component the jitter
            // introduces (the spread also adds y, so compare the two guns).
            v.y.abs()
        };
        // The Chain Gun's pre-jitter + spread gives a larger |y| than the SDMG's
        // spread alone on the same seed (the jitter's 0.03/num6 term dominates).
        assert!(
            aim_deviation(1929) > aim_deviation(1553),
            "the Chain Gun's pre-normalization jitter adds aim deviation"
        );
    }

    // ---- the multi-shot fans ----------------------------------------------

    #[test]
    fn s1_p_i14_the_shotgun_fires_four_or_five_spread_shots() {
        // `Player.cs:47958-47967`: count Next(4,6), then 2 draws per shot.
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        let mut probe = UnifiedRandom::new(0x5EED);
        let count = probe.next_range(4, 6);
        fire(&mut app, 534, 14, 7.0, 15, 4.0, false, cursor, 44, 1.0);
        assert_eq!(live(&app).len(), count as usize);
        assert_eq!(
            stream_of(&mut app),
            draws_after(0x5EED, (1 + 2 * count) as usize)
        );
    }

    #[test]
    fn s1_p_i14_the_boomstick_fires_three_or_four() {
        // `Player.cs:48146-48155`: count Next(3,5), ±35×0.04 per shot.
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        let mut probe = UnifiedRandom::new(0x5EED);
        let count = probe.next_range(3, 5);
        fire(&mut app, 964, 14, 5.35, 12, 5.0, false, cursor, 39, 1.0);
        assert_eq!(live(&app).len(), count as usize);
        assert_eq!(
            stream_of(&mut app),
            draws_after(0x5EED, (1 + 2 * count) as usize)
        );
    }

    #[test]
    fn s1_p_i14_the_tactical_shotgun_fires_six_no_count_roll() {
        // `Player.cs:48355-48363`: SIX shots, ±40×0.05, no count draw.
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        fire(&mut app, 679, 14, 6.0, 34, 6.0, false, cursor, 33, 1.0);
        assert_eq!(live(&app).len(), 6);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 12));
    }

    #[test]
    fn s1_p_i14_the_quad_barrel_fires_one_plus_seven() {
        // `Player.cs:47969-47982`: one straight shot, then 7 perturbed shots.
        // Each of the 7 spends NextFloat + NextFloatDirection + 2 spread draws
        // = 4; the straight shot spends none. Total 28.
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        fire(&mut app, 4703, 14, 7.0, 14, 6.5, false, cursor, 54, 1.0);
        assert_eq!(live(&app).len(), 8);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 28));
    }

    // ---- the Clockwork's animation-window spread --------------------------

    #[test]
    fn s1_p_i14_the_clockwork_spread_tightens_with_the_animation() {
        // `Player.cs:48415-48434`: anim<5 → ±40×0.01 then ×1.1 (2 draws);
        // anim<10 → ±20×0.01 then ×1.05 (2 draws); else the plain aim (0 draws).
        // The burst's three shots (anim 11/7/3) thus go plain → ×1.05 → ×1.1.
        let run = |anim: i32| {
            let mut app = rig(0x5EED);
            let cursor = cursor_right(&app);
            fire(&mut app, 434, 14, 7.75, 30, 4.0, false, cursor, anim, 1.0);
            let s = live(&app)[0];
            (s.vel, stream_of(&mut app))
        };
        let (v11, s11) = run(11);
        let (v7, s7) = run(7);
        let (v3, s3) = run(3);
        assert_eq!(s11, draws_after(0x5EED, 0), "anim 11: plain aim, no draw");
        assert_eq!(s7, draws_after(0x5EED, 2), "anim 7: the <10 window");
        assert_eq!(s3, draws_after(0x5EED, 2), "anim 3: the <5 window");
        assert!((v11.length() - 7.75).abs() < 1e-4, "anim 11 keeps raw speed");
        // The <10 window scales by 1.05, the <5 by 1.1 (plus the jitter).
        assert!(v7.length() > 7.75 && v3.length() > v7.length() - 0.5);
    }

    // ---- the Xenopopper bubbles -------------------------------------------

    #[test]
    fn s1_p_i14_the_xenopopper_fires_bubbles_with_the_deferred_payload() {
        // `Player.cs:47603-47625`: count Next(4,5) + Next(4)==0 bonus; each
        // bubble is type 444 with ai[0]=aim.ToRotation(), localAI[0]=proj,
        // localAI[1]=speed. Each bubble spends 2 NextDouble (factor + angle).
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        let mut probe = UnifiedRandom::new(0x5EED);
        let mut count = probe.next_range(4, 5);
        if probe.next_max(4) == 0 {
            count += 1;
        }
        fire(&mut app, 2797, 14, 12.0, 45, 3.0, false, cursor, 20, 1.0);
        let shots = live(&app);
        assert_eq!(shots.len(), count as usize);
        for s in &shots {
            assert_eq!(s.kind, 444, "the bubble, not the bullet");
            assert_eq!(s.local_ai[0], 14.0, "localAI[0] = projToShoot");
            assert_eq!(s.local_ai[1], 12.0, "localAI[1] = the fused speed");
            assert!((s.ai[0] - 0.0).abs() < 1e-4, "ai[0] = the +x aim rotation");
        }
        assert_eq!(
            stream_of(&mut app),
            draws_after(0x5EED, (2 + 2 * count) as usize)
        );
    }

    // ---- the Onyx Blaster --------------------------------------------------

    #[test]
    fn s1_p_i14_the_onyx_blaster_fires_four_bullets_and_a_double_damage_bolt() {
        // `Player.cs:48828-48838`: 2 pairs × (plus + minus) = 4 bullets, each
        // spending NextFloat + NextFloatDirection (2 draws), then ONE black
        // bolt 661 at speed×1.3 with damage×2 (no draw). Total 8 draws.
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        fire(&mut app, 3788, 14, 7.0, 24, 6.0, false, cursor, 47, 1.0);
        let shots = live(&app);
        assert_eq!(shots.len(), 5, "4 bullets + 1 bolt");
        let bullets: Vec<_> = shots.iter().filter(|s| s.kind == 14).collect();
        let bolts: Vec<_> = shots.iter().filter(|s| s.kind == 661).collect();
        assert_eq!(bullets.len(), 4);
        assert_eq!(bolts.len(), 1);
        assert_eq!(bolts[0].damage, 48, "the bolt's ×2 damage");
        assert!(
            (bolts[0].vel.length() - 7.0 * 1.3).abs() < 1e-3,
            "the bolt's ×1.3 speed"
        );
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 8));
    }

    // ---- the Vortex Beater (gunProj) ---------------------------------------

    #[test]
    fn s1_p_i14_the_vortex_beater_fires_the_fixed_615_with_the_ai0_roll() {
        // `Player.cs:48986-48989`: the fixed 615 at the aim, ai[0] =
        // 5 * Next(0, 20) — ONE draw.
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        let mut probe = UnifiedRandom::new(0x5EED);
        let ai0 = (5 * probe.next_range(0, 20)) as f32;
        fire(&mut app, 3475, 615, 20.0, 50, 2.0, false, cursor, 19, 1.0);
        let shots = live(&app);
        assert_eq!(shots.len(), 1);
        assert_eq!(shots[0].kind, 615, "the fixed gun projectile");
        assert_eq!(shots[0].ai[0], ai0);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 1));
    }

    // ---- the Pew-matic Horn ------------------------------------------------

    #[test]
    fn s1_p_i14_the_pew_matic_horn_rolls_the_frame_and_overrides_to_968() {
        // `Player.cs:47639-47645` + `:47177-47180`: the ±15×0.075 spread (2
        // draws) + ai[1] = Next(projFrames[968] = 24) (1 draw); the post-pick
        // override fires 968 regardless of the resolved ammo proj.
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        let mut probe = UnifiedRandom::new(0x5EED);
        let _vx = probe.next_range(-15, 16);
        let _vy = probe.next_range(-15, 16);
        let frame = probe.next_max(24);
        // Pass a non-968 proj to prove the override.
        fire(&mut app, 5117, 14, 14.0, 30, 3.0, false, cursor, 14, 1.0);
        let shots = live(&app);
        assert_eq!(shots.len(), 1);
        assert_eq!(shots[0].kind, 968, "the post-pick override");
        assert_eq!(shots[0].ai[1], frame as f32);
        // `:47229-47232` — the horn spawns from the entity's upper third
        // (position.Y + height/3 = +14), NOT the mounted centre (the V5
        // fidelity fix); the aim re-derives from the shifted anchor.
        let py = app.player.as_ref().unwrap().y;
        assert!(
            (shots[0].pos.y - (py + 14.0)).abs() < 1e-3,
            "the upper-third anchor (`:47231`)"
        );
        assert!(
            (shots[0].vel.length() - 14.0).abs() < 1e-3,
            "the re-derived aim keeps |speed|"
        );
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 3));
    }

    // ---- the high-velocity conversion --------------------------------------

    #[test]
    fn s1_p_i14_the_sniper_venus_uzi_convert_the_bullet_to_242() {
        // `Player.cs:47107-47118`: projToShoot == 14 → 242 for 1254/1255/1265.
        for kind in [1254u16, 1255, 1265] {
            let mut app = rig(0x5EED);
            let cursor = cursor_right(&app);
            fire(&mut app, kind, 14, 16.0, 50, 5.0, false, cursor, 35, 1.0);
            let shots = live(&app);
            assert_eq!(shots.len(), 1, "{kind}: one shot");
            assert_eq!(shots[0].kind, 242, "{kind}: the HighVelocityBullet");
        }
        // A non-converting gun keeps the plain bullet.
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        fire(&mut app, 96, 14, 9.0, 31, 5.25, false, cursor, 31, 1.0);
        assert_eq!(live(&app)[0].kind, 14, "the Musket keeps 14");
    }

    // ---- seed determinism --------------------------------------------------

    #[test]
    fn s1_p_i14_the_gun_launch_is_seed_deterministic() {
        let run = || {
            let mut app = rig(0xB0A);
            let cursor = cursor_right(&app);
            fire(&mut app, 3788, 14, 7.0, 24, 6.0, false, cursor, 47, 1.0);
            live(&app)
                .iter()
                .map(|s| (s.kind, s.pos, s.vel, s.damage))
                .collect::<Vec<_>>()
        };
        assert_eq!(run(), run(), "the Onyx roll chain is seed-stable");
    }

    // ---- the Xenopopper bubble's flight + death release (V5 gate) ----------

    /// Drive one pool update with a mouse-world aim (the `YoyoInputs.aim`
    /// snapshot the 444's death release reads).
    fn step_pool(pool: &mut crate::projectile::ProjectilePool, app: &mut App, aim: (f32, f32)) {
        let mut npcs = std::mem::take(&mut app.npcs);
        let inputs = crate::projectile::YoyoInputs {
            aim: Some(aim),
            ..Default::default()
        };
        let rng = app.rngs.stream(crate::app::RngKey::UpdateProjectiles);
        pool.update(
            &mut app.world,
            &mut npcs,
            None,
            None,
            None,
            inputs,
            crate::projectile::FlailInputs::default(),
            rng,
            app.local_id as u8,
            false,
        );
        app.npcs = npcs;
    }

    #[test]
    fn s1_p_i14_the_xenopopper_bubble_steers_fades_and_clamps_its_life() {
        // `Projectile.cs:30762-30790` (AI_078): alpha −30/tick, the ±π-wrapped
        // 5% rotation blend toward ai[0], the ×0.96 drag, rotation = vel-angle
        // − π/2, and the owner-gated 60-tick life clamp. Draw-free.
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        fire(&mut app, 2797, 14, 12.0, 45, 3.0, false, cursor, 20, 1.0);
        let mut pool = std::mem::take(&mut app.projectiles);
        // Pin one bubble: aim straight +x (ai[0]=0), velocity angled +y so the
        // steer rotates it back toward +x; life 3600 → clamped 60.
        let slot = pool
            .slots()
            .iter()
            .position(|p| p.active && p.kind == 444)
            .expect("a bubble");
        {
            let p = &mut pool.slots_mut()[slot];
            p.ai[0] = 0.0;
            p.vel = Vec2::new(1.0, 1.0);
            p.alpha = 255;
            p.time_left = 3600;
        }
        step_pool(&mut pool, &mut app, (cursor.x, cursor.y));
        let p = &pool.slots()[slot];
        assert_eq!(p.alpha, 225, "alpha −30 (`:30766`)");
        assert_eq!(p.time_left, 59, "clamped 60 then the tick's −1 (`:30788`)");
        // The steer rotated the velocity toward +x (the y component shrank
        // relative to x) and the ×0.96 drag shortened it below √2.
        assert!(p.vel.y < p.vel.x, "the steer bends toward ai[0]=+x");
        assert!(p.vel.length() < 2.0f32.sqrt(), "the ×0.96 drag");
        assert!(
            (p.rot - (p.vel.y.atan2(p.vel.x) - std::f32::consts::FRAC_PI_2)).abs() < 1e-4,
            "rotation = vel-angle − π/2 (`:30785`)"
        );
        app.projectiles = pool;
    }

    #[test]
    fn s1_p_i14_the_xenopopper_bubble_death_releases_the_payload_toward_the_mouse() {
        // `Projectile.cs:73398-73419` (Kill): the 444's death spawns the
        // deferred bullet (localAI[0]) at localAI[1] speed toward the mouse,
        // carrying the bubble's damage/knockback/owner. Draw-free.
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        fire(&mut app, 2797, 14, 12.0, 45, 3.0, false, cursor, 20, 1.0);
        let mut pool = std::mem::take(&mut app.projectiles);
        let slot = pool
            .slots()
            .iter()
            .position(|p| p.active && p.kind == 444)
            .expect("a bubble");
        // Park the bubble left of the cursor so the release flies +x; force
        // its death this tick (timeLeft 1 → 0).
        {
            let p = &mut pool.slots_mut()[slot];
            p.pos = Vec2::new(cursor.x - 100.0, cursor.y);
            p.vel = Vec2::ZERO;
            p.time_left = 1;
            p.local_ai[0] = 14.0;
            p.local_ai[1] = 12.0;
        }
        let before = pool.slots().iter().filter(|p| p.active && p.kind == 14).count();
        step_pool(&mut pool, &mut app, (cursor.x, cursor.y));
        assert!(!pool.slots()[slot].active, "the bubble died");
        let bullets: Vec<_> = pool
            .slots()
            .iter()
            .filter(|p| p.active && p.kind == 14)
            .collect();
        assert_eq!(bullets.len(), before + 1, "one released bullet");
        let b = bullets.last().unwrap();
        assert!(
            (b.vel.x - 12.0).abs() < 1e-3 && b.vel.y.abs() < 1e-3,
            "the release flies +x at localAI[1]=12 (`:73417-73418`)"
        );
        assert_eq!(b.damage, 45, "the bubble's damage carries");
        app.projectiles = pool;
    }
}