//! S1-P-i13 — the bow launch arms of `Player.ItemCheck_Shoot`
//! (`Player.cs:47056-49468`), the `useAmmo == AmmoID.Arrow` (40) /
//! `AmmoID.Stake` (1836) half. The ammo pick, the stat fuse and the
//! not-consume rolls live in [`crate::ammo`] and the shooter wire
//! (`world_sim/editing.rs`); THIS module owns the launch geometry and the
//! per-bow spawn arms that run AFTER `PickAmmo` resolved `projToShoot` and
//! the wire fused the damage/speed/knockback.
//!
//! Scope: the bow LAUNCH only. The arrow flight (aiStyle 1, `AI_001`) is the
//! projectile-AI task's; the ammo consumption is the ammo task's. The two
//! arrow `gunProj` BOWS (the Phantasm 3540 → 630, the DD2 Phoenix Bow 3854 →
//! 705) DO reach here: the wire passes their fixed bolt id (`g.projectile`,
//! the M3-ITEM-6 reset) and they take the generic single tail at the rotated
//! anchor — `ItemCheck_Shoot`'s shared geometry (`:47196-47307`) runs for
//! every shooter, then their arms fire the fixed bolt (`:48996-48999` /
//! `:49014-49017`). The bullet/rocket `gunProj` two (the Vortex Beater 3475,
//! the Celebration Mk2 3930) keep the wire's generic spawn (their launch
//! arms are the bullet/rocket tasks').
//!
//! ## The shared geometry (`Player.cs:47151-47307`)
//!
//! After `PickAmmo` and `GetWeaponKnockback`, the C# runs the post-pick
//! `projToShoot` overrides (`:47157-47176`), then builds the launch frame:
//!
//! * `pointPosition = RotatedRelativePoint(MountedCenter)` (`:47196`) — the
//!   spawn anchor, NOT the body centre + 10 px the pre-i13 wire used.
//! * the facing flip (`:47202-47224`): `Dot(UnitX.RotatedBy(fullRotation),
//!   normalize(MouseWorld - pointPosition))` decides `ChangeDir(±1)`.
//! * `num4/num5 = MouseWorld - pointPosition` (`:47277-47278`), normalized to
//!   `speed` (`:47289-47307`) — the aim velocity every bow arm reads.
//!
//! ## RNG order
//!
//! The port rolls the crit at THROW time (the established `world_sim`
//! convention, a documented deviation from the C#'s hit-time
//! `Projectile.Damage` roll); that one draw precedes the launch arm's draws.
//! Within each arm the `Main.rand` draws are consumed in the C#'s exact
//! order — the spread/count rolls, the per-shot spawn jitter, the
//! `RotatedByRandom` pairs (`Utils.cs:1697-1700`, TWO `NextDouble` each).
//! Every draw rides the `UpdatePlayers` stream (`Player.ItemCheck` under the
//! `Main.cs:17754` swap).
//!
//! ## Documented folds
//!
//! * `gravDir = 1` (no gravity flip in the port) — the `:47285-47288`
//!   reverse-gravity aim arm and the `Directions.Y` halves fold to plain
//!   adds, the same fold `player.rs`'s mount accessors carry.
//! * `ChangeDir`'s pulley reposition (`Player.cs:42097-42110`) and its
//!   `itemRotation`/composite-arm mirrors (`:42112-42115`) are transient in
//!   the port: `item_rotation` is recomputed from `swing_aim` each frame
//!   (`world_sim/player.rs`) and the composite arms are re-set by
//!   `ApplyUseStyle`, so only the persistent `base.direction = dir`
//!   (`:42111`) lands here.
//! * `MountedCenter == center` and `ApplyItemPositionOffsetFromMount` is a
//!   no-op without an active mount (`Player.cs:51387-51390`); the port's
//!   `mounted_center` accessor already folds the height offsets.

use glam::Vec2;
use tr_core::{UnifiedRandom, World};

use crate::app::App;
use crate::projectile::{can_hit, can_hit_line};

/// `AmmoID.Arrow` / `AmmoID.Stake` (`ID/AmmoID.cs:161`, `:174`) — the two
/// categories `AmmoID.Sets.IsArrow` covers (`:115`). The launch arms key on
/// the weapon's `useAmmo`, so the Stake Launcher rides the same generic
/// tail as the bows.
const AMMO_ARROW: i32 = 40;
const AMMO_STAKE: i32 = 1836;

/// Whether the shooter's ammo category takes the bow launch path.
pub(crate) fn is_arrow_shooter(use_ammo: i32) -> bool {
    use_ammo == AMMO_ARROW || use_ammo == AMMO_STAKE
}

/// `Utils.RotatedBy(this Vector2, double radians)` (`Utils.cs:1675-1684`):
/// the float cos/sin of the double angle, the centre-relative spin (the
/// centre is the origin for every launch use).
fn rotated_by(v: Vec2, radians: f64) -> Vec2 {
    let num = radians.cos() as f32;
    let num2 = radians.sin() as f32;
    Vec2::new(v.x * num - v.y * num2, v.x * num2 + v.y * num)
}

/// `Utils.RotatedByRandom(maxRadians)` (`Utils.cs:1697-1700`):
/// `RotatedBy(NextDouble() * max - NextDouble() * max)` — TWO draws, left
/// operand first, the double subtraction, then the float-cos/sin spin.
fn rotated_by_random(v: Vec2, max_radians: f64, rng: &mut UnifiedRandom) -> Vec2 {
    let angle = rng.next_double() * max_radians - rng.next_double() * max_radians;
    rotated_by(v, angle)
}

/// `Vector2.SafeNormalize(defaultValue)` (`Utils.cs:1837-1844`): the zero /
/// NaN vector returns the default, else the unit vector.
fn safe_normalize(v: Vec2, default: Vec2) -> Vec2 {
    if v == Vec2::ZERO || v.x.is_nan() || v.y.is_nan() {
        default
    } else {
        v.normalize()
    }
}

/// `Utils.GetLerpValue(from, to, t, clamped: true)` (`Utils.cs:315-343`) —
/// the Fairy Queen's distance ramp (`:47938`). The `from > to` branch
/// (100 → 40) inverts: `t < to` → 1, `t > from` → 0, else the linear ratio.
fn get_lerp_value_clamped(from: f32, to: f32, t: f32) -> f32 {
    if from < to {
        if t < from {
            return 0.0;
        }
        if t > to {
            return 1.0;
        }
    } else {
        if t < to {
            return 1.0;
        }
        if t > from {
            return 0.0;
        }
    }
    (t - from) / (to - from)
}

/// The post-`PickAmmo` `projToShoot` overrides (`Player.cs:47157-47176`),
/// the bow half. They run AFTER the resolver fused the ammo's own `shoot`
/// and BEFORE the launch geometry. The 2796/5117 arms are not bows.
///
/// * `:47157-47160` — the Molten Fury 120: the wooden-arrow default 1 → the
///   FireArrow 2 (the Molten Quiver's `:53537` conversion already did this
///   when armed; this is the without-quiver path, gated on `proj == 1`).
/// * `:47161-47164` — the Marrow 682: unconditional → the Bone Arrow 117.
/// * `:47165-47168` — the Ice Bow 725: unconditional → the FrostArrow 120.
/// * `:47173-47176` — the Pulse Bow 2223: unconditional → the PulseBolt 357.
fn post_pick_overrides(kind: u16, proj: u16) -> u16 {
    let mut proj = proj;
    if proj == 1 && kind == 120 {
        proj = 2;
    }
    if kind == 682 {
        proj = 117;
    }
    if kind == 725 {
        proj = 120;
    }
    if kind == 2223 {
        proj = 357;
    }
    proj
}

/// One spawned arrow — the `Projectile.NewProjectile` arguments the launch
/// arm passes, plus the `noDropItem` stamp (`Player.cs:47806` etc).
struct Shot {
    kind: u16,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    ai0: f32,
    ai1: f32,
    no_drop: bool,
}

/// The shared launch frame the bow arms read: the spawn anchor, the aim
/// velocity (`num4/num5`), the player centre and the facing. Built once by
/// [`fire`] from the C#'s `:47196-47307` geometry.
struct Frame {
    /// `pointPosition` (`Player.cs:47196`) — `RotatedRelativePoint(MountedCenter)`.
    spawn: Vec2,
    /// `num4/num5` (`:47306-47307`) — the aim normalized to `speed`.
    vel: Vec2,
    /// `Main.MouseWorld` — the cursor world point the rain-bow spawn X base
    /// and the Fairy Queen's aim read (`:47440`, `:47929`).
    cursor: Vec2,
    /// `base.Center` — the Tsunami/Fairy-Queen pull-back anchor.
    center: Vec2,
    /// `MountedCenter` — the `CanHitLine` anchor (`:47801`).
    mounted_center: Vec2,
    /// `base.direction` after the facing flip.
    direction: i32,
    /// `base.position` + `base.width`/`base.height` — the rain-bow spawn X base.
    pos: Vec2,
    width: f32,
}

/// The bow launch entry — `Player.ItemCheck_Shoot`'s bow half. The wire
/// calls it for arrow/stake shooters AFTER the stat fuse and the crit roll;
/// it applies the post-pick overrides, builds the launch frame (flipping the
/// facing), dispatches the per-bow arm and spawns the arrows. The two arrow
/// `gunProj` BOWS (the Phantasm 3540, the DD2 Phoenix Bow 3854) reach here
/// with their fixed bolt id (`proj = g.projectile`, the M3-ITEM-6 reset):
/// the overrides are non-matching for them and neither has a per-bow arm, so
/// they take the generic single tail at the rotated anchor — their C# launch
/// (`:48996-48999` / `:49014-49017`). The bullet/rocket `gunProj` two keep
/// the wire's generic spawn.
///
/// `anim_max_ticks`/`remaining_anim_ticks` are the integer `itemAnimationMax`
/// / `itemAnimation` the Fairy Queen's fan index reads (`Player.cs:47918`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn fire(
    app: &mut App,
    kind: u16,
    proj: u16,
    speed: f32,
    mut damage: i32,
    knock_back: f32,
    crit: bool,
    world_pos: Vec2,
    anim_max_ticks: i32,
    remaining_anim_ticks: i32,
) {
    let proj = post_pick_overrides(kind, proj);
    // The launch frame needs the player geometry; the facing flip writes it.
    let Some(p) = app.player.as_mut() else {
        return;
    };
    let (mcx, mcy) = p.mounted_center();
    let mounted_center = Vec2::new(mcx, mcy);
    // `pointPosition = RotatedRelativePoint(MountedCenter)` (`:47196`).
    let spawn = p.rotated_relative_point(mounted_center, 0);
    let (cx, cy) = p.center();
    let center = Vec2::new(cx, cy);
    // The facing flip (`:47202-47224`): `value = UnitX.RotatedBy(fullRotation)`,
    // `vector = normalize(MouseWorld - pointPosition)`, `num = Dot(value,
    // vector)`; `num > 0` → ChangeDir(1) else ChangeDir(-1). The `flag`
    // (type 3611) exclusion is not a bow.
    let value = rotated_by(Vec2::X, p.full_rotation as f64);
    let mut vector = world_pos - spawn;
    if vector != Vec2::ZERO {
        vector = vector.normalize();
    }
    let dot = value.dot(vector);
    let new_dir = if dot > 0.0 { 1 } else { -1 };
    // `ChangeDir` (`Player.cs:42091-42116`): the persistent `base.direction`
    // write; the pulley reposition and the itemRotation/composite mirrors are
    // the documented transient folds (module doc).
    p.facing = new_dir;
    let direction = p.facing;
    let pos = Vec2::new(p.x, p.y);
    let width = crate::player::WIDTH;
    let misc_counter = p.misc_counter;
    // The player borrow ends here — the arms below re-borrow only the RNG,
    // the world and the pool through `app`.
    drop(p);

    // `num4/num5 = MouseWorld - pointPosition` (`:47277-47278`), the
    // reverse-gravity arm folded (gravDir = 1). `num6 = sqrt(...)`, the
    // zero/NaN fallback to `(direction, 0)` at `speed` (`:47291-47296`),
    // else `num6 = speed / num6`; `num4 *= num6; num5 *= num6` (`:47306-47307`).
    let mut num4 = world_pos.x - spawn.x;
    let mut num5 = world_pos.y - spawn.y;
    let mut num6 = (num4 * num4 + num5 * num5).sqrt();
    if (num4.is_nan() && num5.is_nan()) || (num4 == 0.0 && num5 == 0.0) {
        num4 = direction as f32;
        num5 = 0.0;
        num6 = speed;
    } else {
        num6 = speed / num6;
    }
    num4 *= num6;
    num5 *= num6;
    let vel = Vec2::new(num4, num5);

    let frame = Frame {
        spawn,
        vel,
        cursor: world_pos,
        center,
        mounted_center,
        direction,
        pos,
        width,
    };

    // The per-bow dispatch, in the C#'s code order. Each arm `return`s in the
    // source; the port folds them into one match. The generic single-shot
    // tail (`:49437`) is the fallback for the bows without a special arm.
    let shots: Vec<Shot> = {
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let world = &app.world;
        match kind {
            // `:47424-47465` — the Daedalus Stormbow.
            3029 => stormbow(&frame, proj, speed, rng),
            // `:47466-47500` — the Blood Rain Bow.
            4381 => blood_rain(&frame, proj, speed, rng),
            // `:47573-47579` — the Hellwing Bow.
            3019 => hellwing(&frame, proj, rng),
            // `:47784-47809` — the Tsunami.
            2624 => tsunami(world, &frame, proj),
            // `:47910-47957` — the Fairy Queen's ranged item.
            4953 => {
                let (shots, dmg) = fairy_queen(
                    world,
                    &frame,
                    proj,
                    speed,
                    damage,
                    anim_max_ticks,
                    remaining_anim_ticks,
                    misc_counter,
                );
                damage = dmg;
                shots
            }
            // `:48278-48308` — the Chlorophyte Shotbow.
            1229 => chlorophyte_shotbow(&frame, proj, rng),
            // `:48748-48760` — the DD2 Betsy Bow.
            3859 => betsy_bow(&frame, proj),
            // The generic single-shot tail (`:49437`): the plain bows and
            // repeaters, the Marrow/Ice Bow/Pulse Bow (their overrides already
            // applied), the Bees' Knees, the Shadowflame Bow, the Stake
            // Launcher, and the two arrow `gunProj` BOWS — the Phantasm 3540
            // (`:48996-48999`, the fixed 630) and the DD2 Phoenix Bow 3854
            // (`:49014-49017`, the fixed 705): their C# arms fire the fixed
            // bolt at `pointPosition` with `num4/num5`, which IS this tail
            // (the wire passes `g.projectile`, the overrides are non-matching
            // for them). `ai7/ai8/ai9` are 0 for the arrow kinds (the
            // `:49379-49435` arms key on non-arrow proj ids).
            _ => vec![Shot {
                kind: proj,
                x: frame.spawn.x,
                y: frame.spawn.y,
                vx: frame.vel.x,
                vy: frame.vel.y,
                ai0: 0.0,
                ai1: 0.0,
                no_drop: false,
            }],
        }
    };

    // Spawn the arrows. The owner is the local player (`NewProjectile`'s
    // `Owner: i`); the crit stamp is the port's throw-time roll.
    let owner = app.local_id as u8;
    for shot in shots {
        let slot = app.projectiles.spawn_with_data_owned(
            shot.kind,
            owner,
            shot.x,
            shot.y,
            shot.vx,
            shot.vy,
            damage,
            knock_back,
            shot.ai0,
            shot.ai1,
        );
        if let Some(slot) = slot {
            app.projectiles.set_crit(slot, crit);
            if shot.no_drop {
                app.projectiles.set_no_drop_item(slot, true);
            }
        }
    }
    let (pcx, pcy) = (center.x, center.y);
    app.audio.play_at("Item_1", pcx, pcy, 1.0);
}

/// `Player.cs:47424-47465` — the Daedalus Stormbow 3029: a rain of 2-4 arrows
/// from above. The count is 3, −1 when the resolved arrow is one of
/// {91, 4, 5, 41} and `Next(3) == 0`, else +1 when `Next(3) == 0`
/// (`:47426-47437`). Each shot spawns 600 px above the mounted centre,
/// jittered toward the cursor, and falls at the aim normalized to `speed`
/// with the per-axis spread and the 0.75-1.5× horizontal scale.
fn stormbow(frame: &Frame, proj: u16, speed: f32, rng: &mut UnifiedRandom) -> Vec<Shot> {
    let mut count = 3;
    if proj == 91 || proj == 4 || proj == 5 || proj == 41 {
        if rng.next_max(3) == 0 {
            count -= 1;
        }
    } else if rng.next_max(3) == 0 {
        count += 1;
    }
    let mut shots = Vec::new();
    for k in 0..count {
        // `:47440`: spawn X = pos.X + width/2 + Next(201)*-direction +
        // (cursorX - pos.X); Y = MountedCenter.Y - 600.
        let mut sx = frame.pos.x + frame.width * 0.5
            + (rng.next_max(201) * -frame.direction) as f32
            + (frame.cursor.x - frame.pos.x);
        // `:47441`: X = (X*10 + Center.X)/11 + Next(-100,101).
        sx = (sx * 10.0 + frame.center.x) / 11.0 + rng.next_range(-100, 101) as f32;
        // `:47442`: Y -= 150*k.
        let sy = frame.mounted_center.y - 600.0 - 150.0 * k as f32;
        // `:47443-47452`: num4 = cursorX - sx, num5 = cursorY - sy, abs(num5),
        // min 20.
        let mut n4 = frame.cursor.x - sx;
        let mut n5 = frame.cursor.y - sy;
        if n5 < 0.0 {
            n5 *= -1.0;
        }
        if n5 < 20.0 {
            n5 = 20.0;
        }
        // `:47453-47456`: num6 = speed/len; num4 *= num6; num5 *= num6.
        let n6 = speed / (n4 * n4 + n5 * n5).sqrt();
        n4 *= n6;
        n5 *= n6;
        // `:47457-47459`: vx = num4 + Next(-40,41)*0.03, vy = num5 +
        // Next(-40,41)*0.03, vx *= Next(75,150)*0.01.
        let mut vx = n4 + rng.next_range(-40, 41) as f32 * 0.03;
        let vy = n5 + rng.next_range(-40, 41) as f32 * 0.03;
        vx *= rng.next_range(75, 150) as f32 * 0.01;
        // `:47460`: spawnX += Next(-50,51).
        sx += rng.next_range(-50, 51) as f32;
        shots.push(Shot {
            kind: proj,
            x: sx,
            y: sy,
            vx,
            vy,
            ai0: 0.0,
            ai1: 0.0,
            no_drop: true,
        });
    }
    shots
}

/// `Player.cs:47466-47500` — the Blood Rain Bow 4381: 1-3 blood arrows from
/// above. The count is `Next(1, 3)` (1 or 2), +1 when `Next(3) == 0`
/// (`:47468-47472`). Each shot spawns 600 px above, jittered, with the
/// 0.55-0.8× horizontal scale and the per-axis spread.
fn blood_rain(frame: &Frame, proj: u16, speed: f32, rng: &mut UnifiedRandom) -> Vec<Shot> {
    let mut count = rng.next_range(1, 3);
    if rng.next_max(3) == 0 {
        count += 1;
    }
    let mut shots = Vec::new();
    for _ in 0..count {
        // `:47475`: spawn X = pos.X + width/2 + Next(61)*-direction +
        // (cursorX - pos.X); Y = MountedCenter.Y - 600.
        let mut sx = frame.pos.x + frame.width * 0.5
            + (rng.next_max(61) * -frame.direction) as f32
            + (frame.cursor.x - frame.pos.x);
        // `:47476`: X = (X*10 + Center.X)/11 + Next(-30,31).
        sx = (sx * 10.0 + frame.center.x) / 11.0 + rng.next_range(-30, 31) as f32;
        // `:47477`: Y -= 150*NextFloat().
        let sy = frame.mounted_center.y - 600.0 - 150.0 * rng.next_float();
        // `:47478-47487`: num4 = cursorX - sx, num5 = cursorY - sy, abs, min 20.
        let mut n4 = frame.cursor.x - sx;
        let mut n5 = frame.cursor.y - sy;
        if n5 < 0.0 {
            n5 *= -1.0;
        }
        if n5 < 20.0 {
            n5 = 20.0;
        }
        let n6 = speed / (n4 * n4 + n5 * n5).sqrt();
        n4 *= n6;
        n5 *= n6;
        // `:47492-47494`: vx = num4 + Next(-20,21)*0.03, vy = num5 +
        // Next(-40,41)*0.03, vx *= Next(55,80)*0.01.
        let mut vx = n4 + rng.next_range(-20, 21) as f32 * 0.03;
        let vy = n5 + rng.next_range(-40, 41) as f32 * 0.03;
        vx *= rng.next_range(55, 80) as f32 * 0.01;
        // `:47495`: spawnX += Next(-50,51).
        sx += rng.next_range(-50, 51) as f32;
        shots.push(Shot {
            kind: proj,
            x: sx,
            y: sy,
            vx,
            vy,
            ai0: 0.0,
            ai1: 0.0,
            no_drop: true,
        });
    }
    shots
}

/// `Player.cs:47573-47579` — the Hellwing Bow 3019: a single arrow whose
/// velocity is the aim spun by `RotatedByRandom(π/16)` and whose `ai[0]/ai[1]`
/// carry the wider `RotatedByRandom(π/8)` spin for the homing flight
/// (`AI_001`'s 485 arm, `Projectile.cs:57708-57735`). FOUR `NextDouble`
/// draws: the ai-spin pair first (`:47575`), then the velocity pair
/// (`:47576`).
fn hellwing(frame: &Frame, proj: u16, rng: &mut UnifiedRandom) -> Vec<Shot> {
    // `vector14 = (num4,num5).RotatedByRandom(0.39269909262657166)` (π/8).
    let ai_spin = rotated_by_random(frame.vel, 0.39269909262657166, rng);
    // `vector15 = (num4,num5).RotatedByRandom(0.19634954631328583)` (π/16).
    let vel_spin = rotated_by_random(frame.vel, 0.19634954631328583, rng);
    vec![Shot {
        kind: proj,
        x: frame.spawn.x,
        y: frame.spawn.y,
        vx: vel_spin.x,
        vy: vel_spin.y,
        ai0: ai_spin.x,
        ai1: ai_spin.y,
        no_drop: false,
    }]
}

/// `Player.cs:47784-47809` — the Tsunami 2624: five parallel arrows in a
/// π/10 fan. The spawn offset is `normalize(num4,num5)*40` (`:47788-47790`),
/// each shot's offset rotated by `π/10*(k - 2)` (`:47794`); if the 40-px
/// probe is blocked the offset collapses to the spawn (`:47795-47798`), and
/// if the line from the mounted centre is blocked the spawn pulls back 15 px
/// along `(spawn - Center).SafeNormalize` (`:47800-47804`). The velocity is
/// the SAME `num4/num5` for all five (`:47805`); every arrow stamps
/// `noDropItem` (`:47806`). NO RNG.
fn tsunami(world: &World, frame: &Frame, proj: u16) -> Vec<Shot> {
    let angle = std::f32::consts::PI / 10.0;
    let count = 5;
    let offset = safe_normalize(frame.vel, Vec2::ZERO) * 40.0;
    let probe = frame.spawn + offset;
    let can = can_hit(world, frame.spawn.x, frame.spawn.y, 0.0, 0.0, probe.x, probe.y, 0.0, 0.0);
    let mut shots = Vec::new();
    for k in 0..count {
        // `vector23 = vector22.RotatedBy(angle*(k - (count-1)/2))`.
        let mut v23 = rotated_by(offset, (angle * (k as f32 - (count as f32 - 1.0) / 2.0)) as f64);
        if !can {
            v23 -= offset;
        }
        let mut v24 = frame.spawn + v23;
        let v25 = safe_normalize(v24 - frame.center, Vec2::ZERO);
        // `CanHitLine(MountedCenter, 4, 4, vector24 - (num4,num5), 0, 0)`.
        let line_target = v24 - frame.vel;
        if !can_hit_line(
            world,
            frame.mounted_center.x,
            frame.mounted_center.y,
            4.0,
            4.0,
            line_target.x,
            line_target.y,
            0.0,
            0.0,
        ) {
            v24 -= v25 * 15.0;
        }
        shots.push(Shot {
            kind: proj,
            x: v24.x,
            y: v24.y,
            vx: frame.vel.x,
            vy: frame.vel.y,
            ai0: 0.0,
            ai1: 0.0,
            no_drop: true,
        });
    }
    shots
}

/// `Player.cs:47910-47957` — the Fairy Queen's ranged item 4953: a single
/// aimed arrow per fire tick (the volley is the useTime 2 < useAnimation 30
/// cadence). The fan index `num61 = (animMax - anim)/2` (`:47918`), mirrored
/// to `4 - num61` when facing right (`:47920-47923`), picks one of the five
/// π/10 fan offsets (`:47924`); the spawn pulls back like the Tsunami. The
/// aim is `DirectionTo(MouseWorld)` from the offset spawn, lerped toward the
/// centre-aim by `GetLerpValue(100, 40, distance)` (`:47936-47942`). At
/// `num61 == 2` the shot becomes the 932 prism at double damage and double
/// speed with `ai1 = miscCounterNormalized*12 % 1` (`:47944-47952`); else the
/// resolved arrow at `v3` with `noDropItem` (`:47954-47955`). NO RNG.
/// Returns the shots and the (possibly doubled) damage.
#[allow(clippy::too_many_arguments)]
fn fairy_queen(
    world: &World,
    frame: &Frame,
    proj: u16,
    speed: f32,
    mut damage: i32,
    anim_max_ticks: i32,
    remaining_anim_ticks: i32,
    misc_counter: u32,
) -> (Vec<Shot>, i32) {
    let angle = std::f32::consts::PI / 10.0;
    let count = 5;
    let offset = safe_normalize(frame.vel, Vec2::ZERO) * 40.0;
    let probe = frame.spawn + offset;
    let can = can_hit(world, frame.spawn.x, frame.spawn.y, 0.0, 0.0, probe.x, probe.y, 0.0, 0.0);
    // `num61 = (itemAnimationMax - itemAnimation) / 2` (integer division).
    let num61 = (anim_max_ticks - remaining_anim_ticks) / 2;
    let num62 = if frame.direction == 1 { 4 - num61 } else { num61 };
    let mut v28 = rotated_by(offset, (angle * (num62 as f32 - (count as f32 - 1.0) / 2.0)) as f64);
    if !can {
        v28 -= offset;
    }
    let mut v29 = frame.spawn + v28;
    let v30 = safe_normalize(v29 - frame.center, Vec2::ZERO);
    let line_target = v29 - frame.vel;
    if !can_hit_line(
        world,
        frame.mounted_center.x,
        frame.mounted_center.y,
        4.0,
        4.0,
        line_target.x,
        line_target.y,
        0.0,
        0.0,
    ) {
        v29 -= v30 * 15.0;
    }
    // `vector31 = vector29.DirectionTo(MouseWorld).SafeNormalize(-UnitY)`.
    let mouse_world = frame.cursor;
    let mut v31 = safe_normalize(mouse_world - v29, -Vec2::Y);
    // `value2 = Center.DirectionTo(Center + (num4,num5)).SafeNormalize(-UnitY)`.
    let value2 = safe_normalize(frame.vel, -Vec2::Y);
    let dist = (mouse_world - frame.center).length();
    let lerp = get_lerp_value_clamped(100.0, 40.0, dist);
    if lerp > 0.0 {
        let fallback = safe_normalize(frame.vel, -Vec2::Y);
        v31 = safe_normalize(v31.lerp(value2, lerp), fallback);
    }
    let v3 = v31 * speed;
    let mut proj = proj;
    if num61 == 2 {
        proj = 932;
        damage *= 2;
    }
    if proj == 932 {
        // `velocity = v3.SafeNormalize(Zero) * (speed*2)`, `ai1 =
        // miscCounterNormalized*12 % 1` (`miscCounterNormalized =
        // miscCounter/300`, `Player.cs:3389`).
        let misc_norm = misc_counter as f32 / 300.0;
        let ai1 = (misc_norm * 12.0) % 1.0;
        let v = safe_normalize(v3, Vec2::ZERO) * (speed * 2.0);
        return (
            vec![Shot {
                kind: proj,
                x: v29.x,
                y: v29.y,
                vx: v.x,
                vy: v.y,
                ai0: 0.0,
                ai1,
                no_drop: false,
            }],
            damage,
        );
    }
    (
        vec![Shot {
            kind: proj,
            x: v29.x,
            y: v29.y,
            vx: v3.x,
            vy: v3.y,
            ai0: 0.0,
            ai1: 0.0,
            no_drop: true,
        }],
        damage,
    )
}

/// `Player.cs:48278-48308` — the Chlorophyte Shotbow 1229: 2-3 arrows with a
/// cumulative per-axis spread. The count is 2, +1 when `Next(3) == 0`
/// (`:48280-48284`). Shot k adds `Next(-35,36)*0.04` to BOTH axes once for
/// each of `k > 0`, `k > 1`, `k > 2` (`:48289-48303`) — so shot 0 is the plain
/// aim, shot 1 one pair, shot 2 two pairs, shot 3 three pairs. The velocity
/// is NOT renormalized; every arrow stamps `noDropItem` (`:48305`).
fn chlorophyte_shotbow(frame: &Frame, proj: u16, rng: &mut UnifiedRandom) -> Vec<Shot> {
    let mut count = 2;
    if rng.next_max(3) == 0 {
        count += 1;
    }
    let mut shots = Vec::new();
    for k in 0..count {
        let mut vx = frame.vel.x;
        let mut vy = frame.vel.y;
        if k > 0 {
            vx += rng.next_range(-35, 36) as f32 * 0.04;
            vy += rng.next_range(-35, 36) as f32 * 0.04;
        }
        if k > 1 {
            vx += rng.next_range(-35, 36) as f32 * 0.04;
            vy += rng.next_range(-35, 36) as f32 * 0.04;
        }
        if k > 2 {
            vx += rng.next_range(-35, 36) as f32 * 0.04;
            vy += rng.next_range(-35, 36) as f32 * 0.04;
        }
        shots.push(Shot {
            kind: proj,
            x: frame.spawn.x,
            y: frame.spawn.y,
            vx,
            vy,
            ai0: 0.0,
            ai1: 0.0,
            no_drop: true,
        });
    }
    shots
}

/// `Player.cs:48748-48760` — the DD2 Betsy Bow 3859: six arrows in a fan.
/// `vector53 = (num4,num5)*0.8`, `projToShoot = 710` forced (`:48751`),
/// `vector54 = vector53.SafeNormalize(-UnitY)`, `num163 = π/180 * -direction`
/// (`:48754`). The loop `num164 = -2.5; num164 < 3; num164++` runs six times
/// (−2.5, −1.5, −0.5, 0.5, 1.5, 2.5); each velocity is
/// `(vector53 + vector54*num164*0.5).RotatedBy(num164*num163)` (`:48757`).
/// NO RNG, NO noDropItem.
fn betsy_bow(frame: &Frame, _proj: u16) -> Vec<Shot> {
    let v53 = frame.vel * 0.8;
    let v54 = safe_normalize(v53, -Vec2::Y);
    let num163 = (std::f32::consts::PI / 180.0) * -(frame.direction as f32);
    let mut shots = Vec::new();
    let mut num164 = -2.5f32;
    while num164 < 3.0 {
        let v = rotated_by(v53 + v54 * num164 * 0.5, (num164 * num163) as f64);
        shots.push(Shot {
            kind: 710,
            x: frame.spawn.x,
            y: frame.spawn.y,
            vx: v.x,
            vy: v.y,
            ai0: 0.0,
            ai1: 0.0,
            no_drop: false,
        });
        num164 += 1.0;
    }
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
    /// `seed`. The cursor world point is passed to [`fire`] directly, so the
    /// camera/mouse mapping is irrelevant here.
    fn rig(seed: i32) -> App {
        let mut w = World::new("bow".into(), 64, 800);
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
        app.rngs
            .stream(crate::app::RngKey::UpdatePlayers)
            .clone()
    }

    // ---- the post-PickAmmo overrides (`Player.cs:47157-47176`) -------------

    #[test]
    fn s1_p_i13_the_post_pick_overrides_map_the_four_bows() {
        // `:47157-47160` — the Molten Fury 120 converts ONLY the wooden-arrow
        // default 1 → the FireArrow 2 (the Molten Quiver's `:53537` already
        // did it when armed; this is the without-quiver path).
        assert_eq!(post_pick_overrides(120, 1), 2);
        assert_eq!(post_pick_overrides(120, 2), 2, "already 2 → unchanged");
        assert_eq!(post_pick_overrides(120, 4), 4, "not 1 → no conversion");
        // `:47161-47164` — the Marrow 682: unconditional → 117.
        assert_eq!(post_pick_overrides(682, 1), 117);
        assert_eq!(post_pick_overrides(682, 4), 117, "no proj gate");
        // `:47165-47168` — the Ice Bow 725: unconditional → 120.
        assert_eq!(post_pick_overrides(725, 1), 120);
        assert_eq!(post_pick_overrides(725, 2), 120);
        // `:47173-47176` — the Pulse Bow 2223: unconditional → 357.
        assert_eq!(post_pick_overrides(2223, 1), 357);
        assert_eq!(post_pick_overrides(2223, 5), 357);
        // A plain bow (the Wooden Bow 39) keeps the resolved arrow.
        assert_eq!(post_pick_overrides(39, 1), 1);
        assert_eq!(post_pick_overrides(39, 4), 4);
    }

    // ---- the generic single-shot tail (`Player.cs:49437`) -----------------

    #[test]
    fn s1_p_i13_the_wooden_bow_fires_one_arrow_at_the_aim() {
        // The generic tail: one arrow at the spawn anchor, the aim normalized
        // to `speed`, no spread, no noDropItem. The Wooden Bow 39 + the wooden
        // arrow's resolved 1.
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        fire(&mut app, 39, 1, 9.1, 9, 2.0, false, cursor, 30, 29);
        let shots = live(&app);
        assert_eq!(shots.len(), 1, "one arrow");
        assert_eq!(shots[0].kind, 1, "the wooden arrow");
        assert!(shots[0].vel.x > 0.0, "flies at the cursor");
        assert!(
            (shots[0].vel.length() - 9.1).abs() < 1e-3,
            "the aim normalized to shootSpeed + ammo speed"
        );
        assert!(!shots[0].no_drop_item, "the generic tail drops its arrow");
        assert_eq!(shots[0].damage, 9);
        assert_eq!(shots[0].knock_back, 2.0);
    }

    #[test]
    fn s1_p_i13_the_marrow_fires_the_bone_arrow() {
        // The Marrow 682's override → 117, then the generic tail.
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        fire(&mut app, 682, 1, 14.0, 48, 4.7, false, cursor, 19, 18);
        let shots = live(&app);
        assert_eq!(shots.len(), 1);
        assert_eq!(shots[0].kind, 117, "the Bone Arrow override (`:47163`)");
    }

    #[test]
    fn s1_p_i13_the_ice_bow_fires_the_frost_arrow() {
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        fire(&mut app, 725, 1, 13.0, 46, 4.5, false, cursor, 14, 13);
        let shots = live(&app);
        assert_eq!(shots.len(), 1);
        assert_eq!(shots[0].kind, 120, "the FrostArrow override (`:47167`)");
    }

    #[test]
    fn s1_p_i13_the_pulse_bow_fires_the_pulse_bolt() {
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        fire(&mut app, 2223, 1, 10.75, 85, 3.0, false, cursor, 20, 19);
        let shots = live(&app);
        assert_eq!(shots.len(), 1);
        assert_eq!(shots[0].kind, 357, "the PulseBolt override (`:47175`)");
    }

    #[test]
    fn s1_p_i13_the_molten_fury_fires_the_fire_arrow_without_the_quiver() {
        // The without-quiver path: proj 1 → 2 (`:47157-47160`).
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        fire(&mut app, 120, 1, 11.0, 36, 2.0, false, cursor, 22, 21);
        let shots = live(&app);
        assert_eq!(shots.len(), 1);
        assert_eq!(shots[0].kind, 2, "the FireArrow (`:47159`)");
    }

    // ---- the facing flip (`Player.cs:47202-47224`) ------------------------

    #[test]
    fn s1_p_i13_the_launch_flips_the_facing_toward_the_cursor() {
        // The cursor to the LEFT: the Dot product is negative → ChangeDir(-1).
        let mut app = rig(0x5EED);
        let (cx, cy) = app.player.as_ref().unwrap().center();
        let cursor = Vec2::new(cx - 80.0, cy);
        fire(&mut app, 39, 1, 9.1, 9, 2.0, false, cursor, 30, 29);
        assert_eq!(app.player.as_ref().unwrap().facing, -1, "faces left");
        let shots = live(&app);
        assert!(shots[0].vel.x < 0.0, "the arrow flies left");
    }

    // ---- the Tsunami (`Player.cs:47784-47809`) ----------------------------

    #[test]
    fn s1_p_i13_the_tsunami_fires_five_parallel_arrows() {
        // Five arrows, the SAME velocity (num4/num5), the fan offsets only move
        // the spawn. Every arrow stamps noDropItem. NO RNG.
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        let seed_stream = stream_of(&mut app);
        fire(&mut app, 2624, 1, 13.0, 58, 2.0, false, cursor, 24, 23);
        let shots = live(&app);
        assert_eq!(shots.len(), 5, "the Tsunami's five arrows");
        for s in &shots {
            assert_eq!(s.kind, 1);
            assert!(s.no_drop_item, "noDropItem (`:47806`)");
            assert!(
                (s.vel.x - shots[0].vel.x).abs() < 1e-4
                    && (s.vel.y - shots[0].vel.y).abs() < 1e-4,
                "the velocity is the SAME for all five (`:47805`)"
            );
        }
        // The fan spreads the spawn Y across the five offsets.
        let ys: Vec<f32> = shots.iter().map(|s| s.pos.y).collect();
        assert!(ys.iter().any(|&y| (y - ys[0]).abs() > 1.0), "the fan offsets differ");
        // NO RNG: the stream is untouched.
        assert_eq!(stream_of(&mut app), seed_stream, "the Tsunami draws nothing");
    }

    // ---- the Chlorophyte Shotbow (`Player.cs:48278-48308`) ----------------

    #[test]
    fn s1_p_i13_the_chlorophyte_shotbow_spreads_two_or_three_arrows() {
        // The count is 2, +1 when Next(3)==0. Shot 0 is the plain aim, shot 1
        // adds one ±0.04 pair, shot 2 two pairs. Every arrow noDropItem.
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        fire(&mut app, 1229, 1, 14.5, 39, 2.75, false, cursor, 19, 18);
        let shots = live(&app);
        assert!(shots.len() == 2 || shots.len() == 3, "2 or 3 arrows");
        for s in &shots {
            assert_eq!(s.kind, 1);
            assert!(s.no_drop_item, "noDropItem (`:48305`)");
        }
        // Shot 0 is the plain aim (no spread).
        assert!(
            (shots[0].vel.length() - 14.5).abs() < 1e-3,
            "shot 0 keeps the full speed"
        );
        // The spread shots deviate from the aim.
        if shots.len() > 1 {
            assert!(
                (shots[1].vel.x - shots[0].vel.x).abs() > 1e-4
                    || (shots[1].vel.y - shots[0].vel.y).abs() > 1e-4,
                "shot 1 carries the spread"
            );
        }
    }

    #[test]
    fn s1_p_i13_the_chlorophyte_count_roll_spends_one_draw() {
        // The count roll is ONE Next(3); the per-shot spread draws follow. For
        // a 2-shot volley (Next(3)!=0): 1 count draw + 2 spread draws (shot 1's
        // pair) = 3. For 3-shot: 1 + 2 + 2 = 5. The crit is rolled by the wire
        // BEFORE fire, so fire's own draws start at the count roll.
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        // Replay the count roll to know the shot count.
        let mut probe = UnifiedRandom::new(0x5EED);
        let count = if probe.next_max(3) == 0 { 3 } else { 2 };
        fire(&mut app, 1229, 1, 14.5, 39, 2.75, false, cursor, 19, 18);
        // Shot k spends 2 draws per true condition among {k>0, k>1, k>2} =
        // 2*min(k,3); the count is 2 or 3, so the spread total is
        // 2*(0+1+…+(count-1)) = count*(count-1).
        let expected_draws = 1 + count * (count - 1);
        assert_eq!(
            stream_of(&mut app),
            draws_after(0x5EED, expected_draws as usize),
            "the count roll + the per-shot spread pairs, in order"
        );
    }

    // ---- the DD2 Betsy Bow (`Player.cs:48748-48760`) ----------------------

    #[test]
    fn s1_p_i13_the_betsy_bow_fires_six_fanned_arrows() {
        // Six arrows (the -2.5..<3 loop), kind 710 forced, the fan rotation
        // π/180*-direction. NO RNG, NO noDropItem.
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        let seed_stream = stream_of(&mut app);
        fire(&mut app, 3859, 1, 14.0, 43, 4.5, false, cursor, 30, 29);
        let shots = live(&app);
        assert_eq!(shots.len(), 6, "the six-iteration fan loop");
        for s in &shots {
            assert_eq!(s.kind, 710, "projToShoot forced to 710 (`:48751`)");
            assert!(!s.no_drop_item, "the Betsy fan drops its arrows");
        }
        // The fan spreads the velocity direction.
        assert!(
            (shots[0].vel.y - shots[5].vel.y).abs() > 1e-3,
            "the fan rotates the velocities"
        );
        assert_eq!(stream_of(&mut app), seed_stream, "the Betsy fan draws nothing");
    }

    // ---- the Hellwing Bow (`Player.cs:47573-47579`) -----------------------

    #[test]
    fn s1_p_i13_the_hellwing_spends_four_rotation_draws() {
        // RotatedByRandom(π/8) for the ai-spin (2 NextDouble), then
        // RotatedByRandom(π/16) for the velocity (2 NextDouble) = 4 draws.
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        fire(&mut app, 3019, 485, 9.0, 27, 5.5, false, cursor, 13, 12);
        let shots = live(&app);
        assert_eq!(shots.len(), 1);
        assert_eq!(shots[0].kind, 485);
        assert_eq!(
            stream_of(&mut app),
            draws_after(0x5EED, 4),
            "the ai-spin pair then the velocity pair (`:47575-47576`)"
        );
        // The ai[0]/ai[1] carry the wider spin, distinct from the velocity.
        assert!(
            (shots[0].ai[0] - shots[0].vel.x).abs() > 1e-4
                || (shots[0].ai[1] - shots[0].vel.y).abs() > 1e-4,
            "the ai-spin (π/8) differs from the velocity spin (π/16)"
        );
    }

    #[test]
    fn s1_p_i13_the_hellwing_rotation_is_seed_deterministic() {
        let run = || {
            let mut app = rig(0xB0A);
            let cursor = cursor_right(&app);
            fire(&mut app, 3019, 485, 9.0, 27, 5.5, false, cursor, 13, 12);
            let s = live(&app)[0];
            (s.vel, s.ai[0], s.ai[1])
        };
        assert_eq!(run(), run(), "the dual spin is seed-stable");
    }

    // ---- the Daedalus Stormbow (`Player.cs:47424-47465`) ------------------

    #[test]
    fn s1_p_i13_the_stormbow_rains_arrows_from_above() {
        // The rain spawns 600 px above the mounted centre; every arrow
        // noDropItem. The count is 2-4.
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        let (mcx, mcy) = app.player.as_ref().unwrap().mounted_center();
        fire(&mut app, 3029, 1, 15.5, 43, 2.25, false, cursor, 19, 18);
        let shots = live(&app);
        assert!(shots.len() >= 2 && shots.len() <= 4, "2-4 rain arrows");
        for s in &shots {
            assert_eq!(s.kind, 1);
            assert!(s.no_drop_item, "noDropItem (`:47462`)");
            assert!(
                s.pos.y < mcy - 500.0,
                "the rain spawns above the player (y={})",
                s.pos.y
            );
            assert!(s.vy > 0.0, "the rain falls downward");
        }
        let _ = mcx;
    }

    #[test]
    fn s1_p_i13_the_stormbow_draw_count_matches_the_shot_count() {
        // The count roll is ONE Next(3); each shot spends SIX draws
        // (Next(201), Next(-100,101), Next(-40,41) vx, Next(-40,41) vy,
        // Next(75,150), Next(-50,51)).
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        let mut probe = UnifiedRandom::new(0x5EED);
        // The wooden arrow (proj 1) is NOT in {91,4,5,41}, so the else branch:
        // count = 3 + (Next(3)==0 ? 1 : 0).
        let count = if probe.next_max(3) == 0 { 4 } else { 3 };
        fire(&mut app, 3029, 1, 15.5, 43, 2.25, false, cursor, 19, 18);
        assert_eq!(live(&app).len(), count as usize);
        assert_eq!(
            stream_of(&mut app),
            draws_after(0x5EED, (1 + 6 * count) as usize),
            "the count roll + six draws per shot, in the C#'s order"
        );
    }

    // ---- the Blood Rain Bow (`Player.cs:47466-47500`) ---------------------

    #[test]
    fn s1_p_i13_the_blood_rain_rains_one_to_three_arrows() {
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        let (_, mcy) = app.player.as_ref().unwrap().mounted_center();
        fire(&mut app, 4381, 819, 12.0, 19, 3.0, false, cursor, 19, 18);
        let shots = live(&app);
        assert!(shots.len() >= 1 && shots.len() <= 3, "1-3 blood arrows");
        for s in &shots {
            assert_eq!(s.kind, 819);
            assert!(s.no_drop_item, "noDropItem (`:47497`)");
            assert!(s.pos.y < mcy - 500.0, "the rain spawns above");
        }
    }

    #[test]
    fn s1_p_i13_the_blood_rain_draw_count_matches_the_shot_count() {
        // The count rolls: Next(1,3) then Next(3). Each shot spends SEVEN draws
        // (Next(61), Next(-30,31), NextFloat, Next(-20,21), Next(-40,41),
        // Next(55,80), Next(-50,51)).
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        let mut probe = UnifiedRandom::new(0x5EED);
        let mut count = probe.next_range(1, 3);
        if probe.next_max(3) == 0 {
            count += 1;
        }
        fire(&mut app, 4381, 819, 12.0, 19, 3.0, false, cursor, 19, 18);
        assert_eq!(live(&app).len(), count as usize);
        assert_eq!(
            stream_of(&mut app),
            draws_after(0x5EED, (2 + 7 * count) as usize),
            "the two count rolls + seven draws per shot"
        );
    }

    // ---- the Fairy Queen's ranged item (`Player.cs:47910-47957`) ----------

    #[test]
    fn s1_p_i13_the_fairy_queen_fires_one_aimed_arrow_per_tick() {
        // A single shot per fire tick (the volley is the cadence). At num61 != 2
        // the resolved arrow flies at v3 with noDropItem.
        let mut app = rig(0x5EED);
        let cursor = cursor_right(&app);
        // anim 30, remaining 29 → num61 = 0.
        fire(&mut app, 4953, 2, 13.0, 55, 2.0, false, cursor, 30, 29);
        let shots = live(&app);
        assert_eq!(shots.len(), 1, "one shot per fire tick");
        assert_eq!(shots[0].kind, 2, "the resolved flaming arrow (num61 != 2)");
        assert!(shots[0].no_drop_item, "noDropItem (`:47955`)");
    }

    #[test]
    fn s1_p_i13_the_fairy_queen_mid_volley_becomes_the_prism() {
        // At num61 == 2 (anim 30, remaining 25 or 26) the shot becomes 932 at
        // double damage and double speed, ai1 = miscCounterNormalized*12 % 1.
        let mut app = rig(0x5EED);
        app.player.as_mut().unwrap().misc_counter = 150; // miscNorm = 0.5
        let cursor = cursor_right(&app);
        fire(&mut app, 4953, 2, 13.0, 55, 2.0, false, cursor, 30, 25);
        let shots = live(&app);
        assert_eq!(shots.len(), 1);
        assert_eq!(shots[0].kind, 932, "the prism (`:47946`)");
        assert_eq!(shots[0].damage, 110, "double damage (`:47947`)");
        assert!(
            (shots[0].vel.length() - 26.0).abs() < 1e-2,
            "double speed (`:47951`)"
        );
        // ai1 = (150/300)*12 % 1 = 6 % 1 = 0.
        assert!(shots[0].ai[1].abs() < 1e-4, "ai1 = miscNorm*12 % 1");
        assert!(!shots[0].no_drop_item, "the 932 prism drops");
    }

    #[test]
    fn s1_p_i13_the_fairy_queen_fan_index_mirrors_with_the_facing() {
        // num62 = num61 when facing left, 4 - num61 when facing right
        // (`:47920-47923`). The spawn offset differs with the facing.
        let mut left = rig(0x5EED);
        left.player.as_mut().unwrap().facing = -1;
        let (cx, cy) = left.player.as_ref().unwrap().center();
        let cursor = Vec2::new(cx - 80.0, cy);
        fire(&mut left, 4953, 2, 13.0, 55, 2.0, false, cursor, 30, 27);
        let mut right = rig(0x5EED);
        right.player.as_mut().unwrap().facing = 1;
        let (cx2, cy2) = right.player.as_ref().unwrap().center();
        let cursor2 = Vec2::new(cx2 + 80.0, cy2);
        fire(&mut right, 4953, 2, 13.0, 55, 2.0, false, cursor2, 30, 27);
        // num61 = (30-27)/2 = 1; left num62 = 1, right num62 = 3. The fan
        // offsets differ, so the spawn Y differs.
        let ly = live(&left)[0].pos.y;
        let ry = live(&right)[0].pos.y;
        assert!((ly - ry).abs() > 1e-3, "the mirrored fan index offsets differ");
    }
}