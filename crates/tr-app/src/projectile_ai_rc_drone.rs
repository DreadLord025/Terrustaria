//! W4-d80 — the RC-car (1105, aiStyle 205) and Jim's-drone (1020, aiStyle 195)
//! engine loops: the full AI machines (verbatim from the W4-d21 / W4-d20
//! census ports) plus the tracked-sound ensure/follow the deferred census
//! (research/64) left unwired. The `SoundID.RCCar` / `SoundID.JimsDrone`
//! loops are continuous volume/pitch trackers: the `localAI[0]` steer-age
//! and `localAI[1]` engine-speed lerp feed the envelope, the ensure plays
//! the loop into `localAI[2]` and the per-tick follow re-points it.
//!
//! The sound rides the `ProjTrackedCue::EngineLoop` queue (the
//! `wants_sound` convention): the arm computes the envelope and queues the
//! ensure+follow every tick; the sim drain owns the pool dance. The
//! `ProjectileAudioTracker` condition folds to the shared atomic (the
//! census fold 4, the bomb-fuse precedent) — the slot's death flips it false.
//! The drone's `Kill()` also stops the loop explicitly (`:70863`); the
//! RC car's does not (`:70855-70864` — only the 1020 arm stops).
//!
//! ZERO gameplay draws on the sound path — the variation/pitch picks ride the
//! two audio-only statics. The AI machines' own dust draws stay on the
//! `UpdateProjectiles` stream (the d21/d20 census).
//!
//! CODE-ONLY MODE (W4-d80): the witness tests at the bottom are WRITTEN NOT
//! RUN (no compiler invoked).

use glam::Vec2;
use tr_core::loot::DropRng;
use tr_core::World;

use crate::projectile::{
    remap, rotation_vector, wrap_angle, ProjTrackedCue, Projectile, JIMS_DRONE,
    REMOTE_CONTROL_CAR,
};

/// The engine-loop volume/pitch envelope (`Projectile.cs:38254-38266` /
/// `:38286-38298` / `:40984-40996` — the three sites share the formula):
/// `num = Remap(lai0, 0, 5, 0, 1) * Remap(lai0, 5, 15, 1, 0)` (the steer-age
/// ramp), `vol = Clamp(Max(Remap(lai1, 0, 100, 0, 25), num * 12), 0, 100)`
/// (the engine-speed floor), `pitch = Clamp(Remap(lai1, 0, 100, -1, 1) +
/// num, -1, 1)`. Draw-free.
fn engine_envelope(lai0: f32, lai1: f32) -> (f32, f32) {
    let num = remap(lai0, 0.0, 5.0, 0.0, 1.0) * remap(lai0, 5.0, 15.0, 1.0, 0.0);
    let vol = remap(lai1, 0.0, 100.0, 0.0, 25.0).max(num * 12.0).clamp(0.0, 100.0);
    let pitch = (remap(lai1, 0.0, 100.0, -1.0, 1.0) + num).clamp(-1.0, 1.0);
    (vol, pitch)
}
/// `AI_205_RemoteControlCar` (`Projectile.cs:38076-38461`) — the RC car
/// (1105): the owner's input bits pack into `ai[0]`, the car drives on the
/// ground (the slope-tilt + the left/right accel + the jump) or flies on a
/// wall (the up/down thrust + the left/right turn) while the "fly" bit is
/// set. The `localAI[0]` fly-time counter, the `localAI[1]` engine-speed
/// lerp and the `localAI[2]` tracked-sound slot are the state; the sound
/// bodies ride the `EngineLoop` cue (W4-d80): the envelope + the
/// ensure/follow into `localAI[2]`.
///
/// The single-player port folds `Main.myPlayer == owner` to `owner == 0`;
/// `gravDir == -1` is unreachable (upright-only). The input bits read the
/// live player's control flags (the `LocalInputCache` stand-in).
pub(crate) fn ai_remote_control_car(
    p: &mut Projectile,
    world: &World,
    player: Option<&crate::player::Player>,
    rng: &mut dyn DropRng,
) {
    // `:38078-38083` — the dead-owner kill flag.
    let mut kill = player.is_some_and(|pl| pl.dead);
    // `:38084-38126` — the owner-gated input pack.
    if p.owner == 0 {
        // `:38086-38090` — the space-height kill.
        if p.pos.y - p.h <= 640.0 {
            p.kill_time_left = p.time_left;
            p.active = false;
            p.time_left = 0;
            return;
        }
        // `:38091-38094` — the held-item gate (5738 = the RC Car item). The
        // C# reads `Main.player[owner]` unconditionally; an absent player is
        // the inactive slot (held type 0 → the gate fails → kill).
        let held = player.map_or(0, |pl| pl.held_item_kind);
        if held != 5738 {
            kill = true;
        }
        // `:38095-38124` — the input bit pack. gravDir is 1 (flag4 false).
        // An absent player's control flags are all false (the inactive slot).
        let bits = p.ai[0] as u8;
        let flag2 = bits & (1 << 4) != 0; // the jump latch
        let mut flag3 = bits & (1 << 5) != 0; // the fly mode
        let mut out: u8 = 0;
        let (cl, cr, cu, cd, cj) = match player {
            Some(pl) => (
                pl.control_left,
                pl.control_right,
                pl.control_up,
                pl.control_down,
                pl.control_jump,
            ),
            None => (false, false, false, false, false),
        };
        if cl {
            out |= 1 << 0;
        }
        if cr {
            out |= 1 << 1;
        }
        if cu {
            out |= 1 << 2;
        }
        if cd {
            out |= 1 << 3;
        }
        if cj {
            out |= 1 << 4;
        }
        // `:38106-38117` — the jump press toggles fly (the sound + 2 dust
        // are the skip; no explicit draws — NewDustDirect).
        if !flag2 && cj && p.vel.y != 0.0 {
            flag3 = !flag3;
        }
        if flag3 {
            out |= 1 << 5;
        }
        let num = out as f32;
        if p.ai[0] != num {
            p.ai[0] = num;
            p.net_update = true;
        }
        // `:38125` — DroneCameraTracker.Track is the camera skip.
    }
    // `:38127-38131` — the dead-owner / wrong-item kill.
    if kill {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
        return;
    }
    // `:38132-38140` — unpack the input bits.
    let bits = p.ai[0] as u8;
    p.rot = p.ai[1];
    let flag5 = bits & (1 << 0) != 0; // left
    let flag6 = bits & (1 << 1) != 0; // right
    let flag7 = bits & (1 << 2) != 0; // up
    let flag8 = bits & (1 << 3) != 0; // down
    let flag9 = bits & (1 << 4) != 0; // jump
    let flag10 = bits & (1 << 5) != 0; // fly
    let flag11 = flag5 | flag6 | flag7 | flag8; // any direction
    // `:38141-38145` — the fly-time counter.
    p.local_ai[0] += 1.0;
    if !flag7 && !flag8 {
        p.local_ai[0] = 0.0;
    }
    // `:38146-38148` — the wall probes (oldPosition + centre).
    let old_center = p.old_position + Vec2::new(p.w * 0.5, p.h * 0.5);
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    let wall_at = |v: Vec2| -> bool {
        let tx = (v.x / 16.0) as i32;
        let ty = (v.y / 16.0) as i32;
        world.tile(tx, ty).is_some_and(|t| t.wall > 0)
    };
    let flag12 = wall_at(old_center) & flag10;
    let flag13 = wall_at(center) & flag10;
    let flag14 = !flag13;
    // `:38149-38272` — the wall-flight branch.
    if flag13 {
        let num2 = if flag7 | flag8 { 0.037699115 } else { 0.07539823 };
        let num3 = (flag7 as i32 - flag8 as i32) as f32 * 0.275;
        p.rot += (flag6 as i32 - flag5 as i32) as f32 * num2;
        p.vel += Vec2::new(p.rot.cos(), p.rot.sin()) * num3;
        p.vel *= 0.96;
        if p.frame < 4 {
            p.vel *= 0.5;
            p.frame = 4;
            if p.direction == -1 {
                p.rot += std::f32::consts::PI;
            }
            p.direction = 1;
            p.sprite_direction = 1;
        }
        // `:38170-38215` — the speed-scaled frame machine.
        let num5 = p.vel.length();
        let mut num4 = frame_delay(num5);
        if flag11 {
            num4 = 1;
        }
        p.frame_counter += 1;
        if p.frame_counter >= num4 {
            p.frame_counter = 0;
            p.frame += 1;
            if p.frame >= 7 {
                p.frame = 4;
            }
        }
        p.sprite_direction = 1;
        p.direction = 1;
        // `:38217-38247` — the exhaust dust gates.
        let num6 = if num3 != 0.0 {
            3
        } else if num5 > 0.2 {
            6
        } else if flag11 {
            2
        } else {
            0
        };
        if num6 != 0 {
            if rng.next_range(0, num6) == 0 {
                let _ = next_vector2_circular(rng, 0.3, 0.3);
            }
            if rng.next_range(0, num6) == 0 {
                let _ = next_vector2_circular(rng, 0.3, 0.3);
            }
        }
        // `:38248-38267` — the engine-speed lerp + the tracked sound
        // (W4-d80: the EngineLoop cue below).
        let num9 = if flag11 { 100.0 } else { 0.0 };
        p.local_ai[1] = math_helper_lerp(p.local_ai[1], num9, 0.2);
        // W4-d80 `:38254-38267` / `:38286-38298`: the engine-loop
        // envelope + the tracked ensure/follow (`SoundID.RCCar`).
        let (vol, pitch) = engine_envelope(p.local_ai[0], p.local_ai[1]);
        p.wants_tracked = Some(ProjTrackedCue::EngineLoop { style: "RCCar", vol, pitch });
        // `:38268-38271` — the zero-Y nudge.
        if p.vel.y == 0.0 {
            p.vel.y = 0.0001;
        }
    }
    // `:38273-38452` — the ground branch.
    if flag14 {
        let flag17 = p.vel.y == 0.0; // on the ground
        // `:38275-38299` — the engine-speed lerp + the tracked sound (W4-d80).
        let num12 = if flag11 {
            if flag17 {
                100.0
            } else {
                10.0
            }
        } else {
            0.0
        };
        p.local_ai[1] = math_helper_lerp(p.local_ai[1], num12, 0.2);
        // W4-d80 `:38254-38267` / `:38286-38298`: the engine-loop
        // envelope + the tracked ensure/follow (`SoundID.RCCar`).
        let (vol, pitch) = engine_envelope(p.local_ai[0], p.local_ai[1]);
        p.wants_tracked = Some(ProjTrackedCue::EngineLoop { style: "RCCar", vol, pitch });
        // `:38300-38344` — the ground tilt / air turn.
        let mut num15 = 0.3f32;
        if flag17 {
            // `:38303-38314` — the slope tilt from the two foot probes.
            let mut value3 = 0.0f32;
            let foot_l = p.pos + Vec2::new(0.0, p.h + 0.02);
            let foot_r = p.pos + Vec2::new(p.w, p.h + 0.02);
            let tl = world.tile((foot_l.x / 16.0) as i32, (foot_l.y / 16.0) as i32);
            let tr = world.tile((foot_r.x / 16.0) as i32, (foot_r.y / 16.0) as i32);
            if tl.is_some_and(|t| block_type(t) == 2) {
                value3 = std::f32::consts::FRAC_PI_4;
            }
            if tr.is_some_and(|t| block_type(t) == 3) {
                value3 = -std::f32::consts::FRAC_PI_4;
            }
            p.rot = math_helper_lerp(wrap_angle(p.rot), value3, 0.3);
            p.vel.x *= 0.97;
            // `:38316-38332` — the facing from the input.
            let num16 = if flag5 {
                -1
            } else if flag6 {
                1
            } else {
                0
            };
            if num16 != 0 {
                if p.sprite_direction != num16 {
                    p.local_ai[1] = 0.0;
                }
                p.sprite_direction = num16;
                p.direction = num16;
            }
            // `:38333-38337` — the jump.
            if flag9 {
                p.vel += Vec2::new((p.rot - std::f32::consts::FRAC_PI_2).cos(), (p.rot - std::f32::consts::FRAC_PI_2).sin()) * 6.0;
                num15 = 0.0;
            }
        } else if !flag10 {
            // `:38339-38344` — the air turn.
            let num17 = std::f32::consts::PI / 15.0;
            let num17 = remap(p.local_ai[0], 0.0, 10.0, num17 * 0.1, num17);
            p.rot += (flag7 as i32 - flag8 as i32) as f32 * num17 * p.sprite_direction as f32;
        }
        // `:38345-38349` — the wall-exit boost.
        if flag12 && !flag13 {
            p.vel *= 1.5;
            p.rot = wrap_angle(p.rot);
        }
        // `:38350-38359` — the frame reset on landing.
        let flag18 = (!flag17 && p.frame >= 4) & flag10;
        if (flag17 || !flag10) && p.frame >= 4 {
            if !flag17 && p.vel.x < 0.0 {
                p.direction = -1;
                p.sprite_direction = -1;
                p.rot += std::f32::consts::PI;
            }
            p.frame = 0;
        }
        // `:38360-38405` — the |X|-speed frame machine.
        let num18 = p.vel.x.abs();
        let mut num19 = frame_delay(num18);
        if flag11 {
            num19 = 1;
        }
        if !flag18 {
            p.frame_counter += 1;
            if p.frame_counter >= num19 {
                p.frame_counter = 0;
                p.frame += 1;
                if p.frame >= 4 {
                    p.frame = 0;
                }
            }
        }
        // `:38406-38415` — the gravity + the left/right accel.
        p.vel.y += num15;
        let num20 = if flag17 { 0.3 } else { 0.1 };
        if flag5 {
            p.vel.x -= num20;
        }
        if flag6 {
            p.vel.x += num20;
        }
        // `:38416-38418` — WalkDownSlope moves the position + velocity.
        let wd = walk_down_slope(world, p.pos, p.vel, p.w as i32, p.h as i32, num15);
        p.pos = wd.0;
        p.vel = wd.1;
        // `:38419-38451` — the landing dust (the ground + any-direction
        // gate): the 3-tile drop probe, then 2×(NextFloat gate + the
        // NextVector2CircularEdge draw).
        if flag17 & flag11 {
            let mut num21 = 0.0f32;
            let num23 = 16.0 * 3.0;
            for j in 0..3 {
                let step = Vec2::new(0.0, 16.0);
                let tc = crate::collision::tile_collision(
                    world,
                    [p.pos.x + step.x * j as f32, p.pos.y + step.y * j as f32],
                    [step.x, step.y],
                    p.w,
                    p.h,
                    false,
                    false,
                );
                num21 += tc.vel[1];
                if tc.vel != [step.x, step.y] {
                    break;
                }
            }
            if num21 != num23 {
                for _ in 0..2 {
                    if rng.next_float() < 0.66 {
                        continue;
                    }
                    let _ = next_vector2_circular_edge(rng, 2.5, 1.0);
                }
            }
        }
    }
    // `:38453-38460` — the rotation latch, the 16-speed clamp, the light
    // (skip), the timeLeft refresh and the section check (skip).
    p.ai[1] = p.rot;
    if p.vel.length() > 16.0 {
        p.vel *= 16.0 / p.vel.length();
    }
    p.time_left = 2;
}

/// `AI_195_JimsDrone` (`Projectile.cs:40923-41067`) — Jim's Drone (1020),
/// the RC drone the player steers with the run/up/down keys. The owner block
/// packs the input bits into `ai[0]` (the grav-flip XOR on left/right), the
/// altitude/held-item kill gates, the tilt machine (`ai[1]` = rotation), the
/// thrust/drag/cap physics, the `timeLeft = 2` keep-alive and the ground-probe
/// dust.
///
/// The port keeps `gravDir = 1` (no gravity-flip system — the `mounted_top`
/// precedent), so `flag2` is false and the input XOR folds to the raw keys.
/// `Main.DroneCameraTracker.Track` (`:40955`) and
/// `CheckSectionsInCaseOwnerIsWatching` (`:41066`) are camera/section
/// bookkeeping — skipped. The tracked-loop sound (`SoundID.JimsDrone`,
/// `:40986-40997`) rides the `EngineLoop` cue (W4-d80): the volume/pitch
/// envelope from the `localAI[0]`/`[1]` engine-speed state, the ensure into
/// `localAI[2]` and the per-tick follow.
///
/// RNG: the ground-probe dust (`:41050-41063`) runs only when airborne
/// (`!flag6`) and the 3-step probe found a floor within 48 px (`num7 != 48`).
/// TWO iterations, each a `NextFloat` gate; on the taken branch
/// (`!(NextFloat < 0.66)`) a `NextVector2CircularEdge(2.5, 1)` (one float,
/// `Utils.cs:1271-1273`) then a `Next(2)` dust-type pick. The dust body skips.
///
/// The row keeps stock `tileCollide = true`; the blocked-move arm
/// (`:18057-18067`) reverses the blocked axis at ×0.25 when |last| > 1 —
/// [`drone_move`] ports that (the `ai_music_note` per-axis probe stand-in).
pub(crate) fn ai_jims_drone(
    p: &mut Projectile,
    world: &World,
    player: Option<&crate::player::Player>,
    local_owner: u8,
    rng: &mut dyn DropRng,
) {
    // `:40925-40930` — the dead-owner kill flag.
    let mut flag = player.map(|pl| pl.dead).unwrap_or(false);
    // `:40931-40956` — the owner-only block: the altitude kill, the held-item
    // gate and the input-bit pack. The port's single player rides slot 0.
    if p.owner == local_owner {
        // `position.Y - height <= 640` → kill (the drone flew too high).
        if p.pos.y - p.h <= 640.0 {
            p.kill_time_left = p.time_left;
            p.active = false;
            p.time_left = 0;
            return;
        }
        // `HeldItem.type != 5451` (the RC transmitter) → the kill flag.
        if player.map(|pl| pl.held_item_kind).unwrap_or(0) != 5451 {
            flag = true;
        }
        // `gravDir == -1` folds false (the port keeps gravDir = 1).
        let flag2 = false;
        let (cl, cr, cu, cd) = player
            .map(|pl| (pl.control_left, pl.control_right, pl.control_up, pl.control_down))
            .unwrap_or((false, false, false, false));
        // BitsByte: bit0 = left ^ flag2, bit1 = right ^ flag2, bit2 = up,
        // bit3 = down.
        let mut bits = 0u8;
        if cl ^ flag2 {
            bits |= 1;
        }
        if cr ^ flag2 {
            bits |= 2;
        }
        if cu {
            bits |= 4;
        }
        if cd {
            bits |= 8;
        }
        let num = bits as f32;
        if p.ai[0] != num {
            p.ai[0] = num;
            p.net_update = true;
        }
        // Main.DroneCameraTracker.Track — skipped (camera bookkeeping).
    }
    // `:40957-40961` — the kill flag (dead owner / wrong held item).
    if flag {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
        return;
    }
    // `:40962-40967` — unpack the input bits; rotation = ai[1].
    let bits = p.ai[0] as u8;
    p.rot = p.ai[1];
    let flag3 = bits & 1 != 0; // left
    let flag4 = bits & 2 != 0; // right
    let flag5 = bits & 4 != 0; // up (thrust)
    // `:40968-40972` — the steer-age counter (resets when neither left/right).
    p.local_ai[0] += 1.0;
    if !flag3 && !flag4 {
        p.local_ai[0] = 0.0;
    }
    // `:40973-40983` — the throttle target: 0 grounded, 10 airborne, 100
    // thrusting; localAI[1] lerps toward it at 0.2.
    let flag6 = p.vel.y == 0.0;
    let mut num2 = if flag6 { 0.0f32 } else { 10.0 };
    if flag5 {
        num2 = 100.0;
    }
    p.local_ai[1] += (num2 - p.local_ai[1]) * 0.2;
    // W4-d80 `:40984-40997`: the engine-loop envelope + the tracked
    // ensure/follow (`SoundID.JimsDrone`).
    let (vol, pitch) = engine_envelope(p.local_ai[0], p.local_ai[1]);
    p.wants_tracked = Some(ProjTrackedCue::EngineLoop { style: "JimsDrone", vol, pitch });
    // `:40984-40997` — the sound volume/pitch envelope feeds the EngineLoop
    // cue (W4-d80, draw-free).
    // `:40998-41020` — the tilt/thrust machine.
    let mut num5 = 0.15f32;
    if flag6 {
        // Grounded: damp the tilt ×0.7, drag X ×0.97; thrust adds the
        // rotation-vector kick at 0.5 and lowers gravity to 0.05.
        p.rot = wrap_angle(p.rot) * 0.7;
        p.vel.x *= 0.97;
        if flag5 {
            p.vel += rotation_vector(p.rot - std::f32::consts::FRAC_PI_2) * 0.5;
            num5 = 0.05;
        }
    } else {
        // Airborne: the turn rate ramps 0.1×→1× over the first 10 steer ticks.
        let mut num6 = std::f32::consts::PI / 15.0;
        num6 = remap(p.local_ai[0], 0.0, 10.0, num6 * 0.1, num6);
        p.rot += (flag4 as i32 - flag3 as i32) as f32 * num6;
        if flag5 {
            let vector = rotation_vector(p.rot - std::f32::consts::FRAC_PI_2);
            p.vel.x *= 0.97;
            p.vel += vector * 0.5;
        }
    }
    // `:41021-41030` — gravity, the ai[1] rotation latch, the fall drag and
    // the 16 speed cap.
    p.vel.y += num5;
    p.ai[1] = p.rot;
    if !flag5 && p.vel.y > 12.0 {
        p.vel.y *= 0.95;
    }
    let len = p.vel.length();
    if len > 16.0 {
        p.vel *= 16.0 / len;
    }
    // `:41031-41032` — AddLight skipped; the timeLeft = 2 keep-alive.
    p.time_left = 2;
    // `:41033-41065` — the airborne ground-probe dust. The 3-step (16 px)
    // TileCollision probe accumulates the free-fall distance; a floor within
    // 48 px opens the two-dust window.
    if !flag6 {
        let vector2 = p.pos;
        let mut num7 = 0.0f32;
        let num8 = 3;
        let num9 = 16 * num8;
        for i in 0..num8 {
            let vector3 = Vec2::new(0.0, 16.0);
            let probe_pos = vector2 + vector3 * i as f32;
            let tc = crate::collision::tile_collision(
                world,
                [probe_pos.x, probe_pos.y],
                [vector3.x, vector3.y],
                p.w,
                p.h,
                false,
                false,
            );
            num7 += tc.vel[1];
            if tc.vel != [vector3.x, vector3.y] {
                break;
            }
        }
        if num7 != num9 as f32 {
            for _ in 0..2 {
                if !(rng.next_float() < 0.66) {
                    let _ = rng.next_float(); // NextVector2CircularEdge(2.5, 1)
                    let _ = rng.next_range(0, 2); // the dust-type pick (31 / 16)
                }
            }
        }
    }
    // CheckSectionsInCaseOwnerIsWatching — skipped (section bookkeeping).
}

/// The 1020 drone's blocked-move arm (`HandleMovement`
/// `Projectile.cs:18057-18067`): the blocked axis reverses at ×0.25 of the
/// pre-collision velocity, but ONLY when |last axis| > 1 (the speed gate the
/// 1116 petal lacks). The port's per-axis destination-box probe stands in for
/// the swept `TileCollision` (the `ai_music_note` precedent).
pub(crate) fn drone_move(p: &mut Projectile, world: &World) {
    let last = p.vel;
    let nx = p.pos.x + p.vel.x;
    let ny = p.pos.y + p.vel.y;
    let hit_x = crate::player::collides_rect(world, nx, p.pos.y, p.w, p.h);
    let hit_y = crate::player::collides_rect(world, p.pos.x, ny, p.w, p.h);
    if hit_x && last.x.abs() > 1.0 {
        p.vel.x = -last.x * 0.25;
    }
    if hit_y && last.y.abs() > 1.0 {
        p.vel.y = -last.y * 0.25;
    }
    p.pos += p.vel;
}

/// `Utils.NextVector2Circular(hw, hh)` (`Utils.cs:1266-1269`):
/// `NextVector2Unit() * (hw, hh) * NextFloat()` — the unit-angle float
/// (`:1261-1264`, one `NextFloat` → `rotationRange * NextFloat()` →
/// `ToRotationVector2`) then the radius float. TWO draws, in order.
fn next_vector2_circular(rng: &mut dyn DropRng, hw: f32, hh: f32) -> Vec2 {
    let angle = std::f32::consts::TAU * rng.next_float();
    let unit = Vec2::new(angle.cos(), angle.sin());
    unit * Vec2::new(hw, hh) * rng.next_float()
}

/// `Tile.blockType()` (`Tile.cs:202-214`): 0 = full block, 1 = half brick,
/// 2-5 = the four slopes. The 205's ground-tilt reads it.
fn block_type(t: &tr_core::tile::Tile) -> u8 {
    if t.half_brick() {
        return 1;
    }
    let s = t.slope();
    if s > 0 {
        s + 1
    } else {
        0
    }
}

/// The 205's speed→frame-delay ladder (`:38170-38207` / `:38360-38397`):
/// 999999 below 0.02, then 16/12/10/8/6/3/2/1 at the cited thresholds.
fn frame_delay(speed: f32) -> i32 {
    let mut d = 999999;
    if speed >= 0.02 {
        d = 16;
    }
    if speed >= 0.2 {
        d = 12;
    }
    if speed >= 0.5 {
        d = 10;
    }
    if speed >= 0.8 {
        d = 8;
    }
    if speed >= 1.5 {
        d = 6;
    }
    if speed >= 3.0 {
        d = 3;
    }
    if speed >= 6.0 {
        d = 2;
    }
    if speed >= 9.0 {
        d = 1;
    }
    d
}

/// `MathHelper.Lerp` (XNA): `value1 + (value2 - value1) * amount`. The
/// 205's engine-speed lerp + the ground tilt read it (NOT `Utils.GetLerpValue`,
/// which is the clamped 0..1 ramp the `remap` helper wraps).
fn math_helper_lerp(value1: f32, value2: f32, amount: f32) -> f32 {
    value1 + (value2 - value1) * amount
}

/// `Utils.NextVector2CircularEdge(hw, hh)` (`Utils.cs:1271-1274`):
/// `NextVector2Unit() * (hw, hh)` — ONE float (the unit angle), no radius.
fn next_vector2_circular_edge(rng: &mut dyn DropRng, hw: f32, hh: f32) -> Vec2 {
    let angle = std::f32::consts::TAU * rng.next_float();
    Vec2::new(angle.cos(), angle.sin()) * Vec2::new(hw, hh)
}

/// `Collision.WalkDownSlope` (`Collision.cs:1686-1789`): when the velocity's
/// Y equals the gravity, scan the two rows below the box for the nearest
/// slope tile and add |X| to Y when the box rides it (the down-slope walk).
/// Returns the (position, velocity) pair — the position is unchanged (the
/// C# returns `Position` verbatim); only Y of the velocity may grow.
fn walk_down_slope(
    world: &World,
    position: Vec2,
    velocity: Vec2,
    width: i32,
    height: i32,
    gravity: f32,
) -> (Vec2, Vec2) {
    if velocity.y != gravity {
        return (position, velocity);
    }
    let mut vel = velocity;
    let value = (position.x / 16.0) as i32;
    let value2 = ((position.x + width as f32) / 16.0) as i32;
    let value3 = ((position.y + height as f32 + 4.0) / 16.0) as i32;
    let value = value.clamp(0, world.width - 1);
    let value2 = value2.clamp(0, world.width - 1);
    let value3 = value3.clamp(0, (world.height - 2 - 40).max(0));
    let mut num = (value3 + 3) as f32 * 16.0;
    let mut num2 = -1i32;
    let mut num3 = -1i32;
    let num4 = if velocity.x < 0.0 { 2 } else { 1 };
    for i in value..=value2 {
        for j in value3..=value3 + 1 {
            let Some(t) = world.tile(i, j) else { continue };
            if !t.active()
                || (!tr_core::tiledata::TILE_SOLID.has(t.kind)
                    && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind))
            {
                continue;
            }
            let mut num5 = j as f32 * 16.0;
            if t.half_brick() {
                num5 += 8.0;
            }
            // The 16×16 probe rect at (i*16, j*16-17) vs the box.
            let probe = tr_core::whip::Rect::new(i * 16, j * 16 - 17, 16, 16);
            let box_r = tr_core::whip::Rect::new(
                position.x as i32,
                position.y as i32,
                width,
                height,
            );
            if !probe.intersects(box_r) || !(num5 <= num) {
                continue;
            }
            if num == num5 {
                if t.slope() == 0 {
                    continue;
                }
                if num2 != -1 && num3 != -1 {
                    if let Some(prev) = world.tile(num2, num3) {
                        if prev.slope() != 0 {
                            if t.slope() == num4 {
                                num = num5;
                                num2 = i;
                                num3 = j;
                            }
                            continue;
                        }
                    }
                }
                num = num5;
                num2 = i;
                num3 = j;
            } else {
                num = num5;
                num2 = i;
                num3 = j;
            }
        }
    }
    if num2 != -1 && num3 != -1 {
        if let Some(t) = world.tile(num2, num3) {
            let num8 = t.slope();
            if num8 > 0 {
                let vx = num2 as f32 * 16.0;
                let vy = num3 as f32 * 16.0;
                match num8 {
                    2 => {
                        let num9 = vx + 16.0 - (position.x + width as f32);
                        if position.y + height as f32 >= vy + num9 && velocity.x < 0.0 {
                            vel.y += velocity.x.abs();
                        }
                    }
                    1 => {
                        let num9 = position.x - vx;
                        if position.y + height as f32 >= vy + num9 && velocity.x > 0.0 {
                            vel.y += velocity.x.abs();
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    (position, vel)
}

// ---------------------------------------------------------------------------
// Witness tests — WRITTEN NOT RUN (code-only mode, W4-d80). The engine-loop
// envelope, the sound cue, the kill gates and the RNG contract.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::proj_data;
    use tr_core::tile_light::FastRandom;

    fn world() -> World {
        World::new("t".into(), 300, 200)
    }

    fn proj(kind: u16) -> Projectile {
        let d = proj_data(kind).expect("the rowed kind");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(100.0 * 16.0, 80.0 * 16.0);
        p.w = d.w;
        p.h = d.h;
        p.tile_collide = d.tile_collide;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p
    }

    fn player(w: &World, held: u16) -> crate::player::Player {
        let mut pl = crate::player::Player::spawn(w);
        pl.held_item_kind = held;
        pl
    }

    /// The rows carry the census slices (`Projectile.cs:10294-10302` /
    /// `:9659-9666`).
    #[test]
    fn w4_d80_the_rc_drone_rows() {
        let d = proj_data(REMOTE_CONTROL_CAR).unwrap();
        assert_eq!(d.ai_style, 205);
        assert_eq!((d.w, d.h), (26.0, 26.0), "`:10296-10297`");
        assert!(d.tile_collide, "`:10300`");
        let d = proj_data(JIMS_DRONE).unwrap();
        assert_eq!(d.ai_style, 195);
        assert_eq!((d.w, d.h), (12.0, 12.0), "`:9661-9662`");
        assert_eq!(d.penetrate, -1, "`:9665`");
    }

    /// The engine envelope (`:38254-38266`): the steer-age ramp × the
    /// engine-speed floor, the pitch remap. Hand-derived fixtures.
    #[test]
    fn w4_d80_the_engine_envelope() {
        // lai0 = 0 (no steer): num = remap(0,0,5,0,1)=0 * remap(0,5,15,1,0)=1
        // → 0. vol = max(remap(lai1,0,100,0,25), 0). pitch = remap(lai1,...).
        let (vol, pitch) = engine_envelope(0.0, 0.0);
        assert_eq!(vol, 0.0);
        assert_eq!(pitch, -1.0, "remap(0,0,100,-1,1) = -1");
        let (vol, pitch) = engine_envelope(0.0, 100.0);
        assert_eq!(vol, 25.0, "remap(100,0,100,0,25) = 25");
        assert_eq!(pitch, 1.0);
        // lai0 = 5 (peak steer): num = remap(5,0,5,0,1)=1 * remap(5,5,15,1,0)=1
        // → 1. vol = max(remap(lai1,...), 12). At lai1=0: max(0,12)=12.
        let (vol, pitch) = engine_envelope(5.0, 0.0);
        assert_eq!(vol, 12.0, "the num*12 floor");
        assert_eq!(pitch, 0.0, "remap(0,...)=-1 + num 1 = 0");
        // lai0 = 15 (steer ended): num = remap(15,0,5,0,1)=1 * remap(15,5,15,1,0)=0
        // → 0. vol = remap(lai1,...).
        let (vol, _) = engine_envelope(15.0, 50.0);
        assert_eq!(vol, 12.5, "remap(50,0,100,0,25) = 12.5");
        // The clamps: vol caps at 100, pitch at ±1.
        let (vol, pitch) = engine_envelope(5.0, 100.0);
        assert_eq!(vol, 25.0, "max(25, 12) = 25");
        assert_eq!(pitch, 1.0, "remap(100,...)=1 + 1 = 2 → clamp 1");
    }

    /// The RC car queues the engine-loop cue every tick (the wall-flight or
    /// ground branch). The dead-owner / wrong-item kill gates fire first.
    #[test]
    fn w4_d80_the_rc_car_queues_the_engine_loop() {
        let w = world();
        let pl = player(&w, 5738); // the RC Car item
        let mut p = proj(REMOTE_CONTROL_CAR);
        p.pos = Vec2::new(1600.0, 2000.0); // below the 640 altitude kill
        let mut rng = FastRandom::new(7);
        ai_remote_control_car(&mut p, &w, Some(&pl), &mut rng);
        assert!(p.active, "the car lives");
        assert!(
            matches!(p.wants_tracked, Some(ProjTrackedCue::EngineLoop { style: "RCCar", .. })),
            "the engine-loop cue queued"
        );
        assert_eq!(p.time_left, 2, "the keep-alive (`:38459`)");
    }

    /// The RC car altitude kill (`:38086-38090`): position.Y - height <= 640.
    #[test]
    fn w4_d80_the_rc_car_altitude_kill() {
        let w = world();
        let pl = player(&w, 5738);
        let mut p = proj(REMOTE_CONTROL_CAR);
        p.pos = Vec2::new(1600.0, 600.0); // 600 - 26 = 574 <= 640
        let mut rng = FastRandom::new(7);
        ai_remote_control_car(&mut p, &w, Some(&pl), &mut rng);
        assert!(!p.active, "the altitude kill");
    }

    /// The RC car wrong-held-item kill (`:38091-38094`).
    #[test]
    fn w4_d80_the_rc_car_wrong_item_kills() {
        let w = world();
        let pl = player(&w, 0); // not the RC Car item
        let mut p = proj(REMOTE_CONTROL_CAR);
        p.pos = Vec2::new(1600.0, 2000.0);
        let mut rng = FastRandom::new(7);
        ai_remote_control_car(&mut p, &w, Some(&pl), &mut rng);
        assert!(!p.active, "the wrong-item kill");
    }

    /// The drone queues the engine-loop cue; the altitude / held-item kill
    /// gates fire first (`:40933-40941`).
    #[test]
    fn w4_d80_the_drone_queues_the_engine_loop() {
        let w = world();
        let pl = player(&w, 5451); // the Jim's Drone item
        let mut p = proj(JIMS_DRONE);
        p.pos = Vec2::new(1600.0, 2000.0);
        let mut rng = FastRandom::new(7);
        ai_jims_drone(&mut p, &w, Some(&pl), 0, &mut rng);
        assert!(p.active);
        assert!(
            matches!(p.wants_tracked, Some(ProjTrackedCue::EngineLoop { style: "JimsDrone", .. })),
            "the engine-loop cue queued"
        );
        assert_eq!(p.time_left, 2, "the keep-alive (`:41032`)");
    }

    /// The drone altitude kill (`:40933-40937`).
    #[test]
    fn w4_d80_the_drone_altitude_kill() {
        let w = world();
        let pl = player(&w, 5451);
        let mut p = proj(JIMS_DRONE);
        p.pos = Vec2::new(1600.0, 600.0); // 600 - 12 = 588 <= 640
        let mut rng = FastRandom::new(7);
        ai_jims_drone(&mut p, &w, Some(&pl), 0, &mut rng);
        assert!(!p.active, "the altitude kill");
    }

    /// The drone wrong-held-item kill (`:40938-40941`).
    #[test]
    fn w4_d80_the_drone_wrong_item_kills() {
        let w = world();
        let pl = player(&w, 0);
        let mut p = proj(JIMS_DRONE);
        p.pos = Vec2::new(1600.0, 2000.0);
        let mut rng = FastRandom::new(7);
        ai_jims_drone(&mut p, &w, Some(&pl), 0, &mut rng);
        assert!(!p.active, "the wrong-item kill");
    }

    /// The RNG contract: the engine-loop cue path draws nothing on the
    /// gameplay stream (the envelope is pure math; the tracked play's
    /// variation/pitch picks ride the audio-only statics at the drain).
    /// The AI machines' dust draws DO ride the stream (the d21/d20 census) —
    /// this witness pins the envelope alone.
    #[test]
    fn w4_d80_the_engine_envelope_is_draw_free() {
        let mut rng = FastRandom::new(7);
        let peek = rng.seed;
        let _ = engine_envelope(5.0, 50.0);
        let _ = engine_envelope(0.0, 0.0);
        assert_eq!(rng.seed, peek, "the envelope draws nothing");
    }
}
