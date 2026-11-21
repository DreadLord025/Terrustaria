//! W4-d11 — the aiStyle 101-110 census slice (`Projectile.cs`'s `AI()`
//! if/else chain, `:33165-34109`). Census on this branch's base (a725acb0):
//! - **101** (`:33165-33221`, the 536 MedusaHeadRay + the 591
//!   MinecartMechLaser) — MISSING (no rows, no arm). Ported here.
//! - **102** (`:33222-33447`, the 539/573/574 tower orbs) — the 574-only
//!   `ai_nebula_eye` stand-in is replaced by the verbatim three-kind
//!   machine (the release volley, the dust-gate census, the frames).
//! - **103** (`:33448-33477`, the 540 mark) — already armed
//!   (`ai_stardust_mark`); the row's alpha + the spawn anchor are fixed
//!   to the C# in projectile.rs / data.rs (a separate commit).
//! - **104** (`:33478-33511`, the 565 BrainOfConfusion) — MISSING.
//! - **105** (`:33512-33515` -> `AI_105_SporeSac` `:40043-40192`, the
//!   567/568 spore traps) — MISSING.
//! - **106** (`:33516-33552`, the 569-571 spore gas) — MISSING.
//! - **107** (`:33553-33678`, the 575 NebulaSphere + the 596
//!   DesertDjinnCurse) — MISSING.
//! - **108** (`:33679-34025`, the 578/579/813 ritual markers) — the
//!   578/579 core is armed (`ai_vortex_ritual`); the 813 arm + the
//!   dust-gate census + the cues land in projectile.rs (a separate
//!   commit).
//! - **109** (`:34026-34065`, the 582 MechanicWrench) — MISSING.
//! - **110** (`:34066-34109`, the 584 NurseSyringeHeal) — MISSING.
//!
//! RNG contract (the `UpdateProjectiles` stream, `Main.cs:17775-17777`):
//! every EXPLICIT `Main.rand` draw at the call sites stays, in the C#'s
//! order; the dust BODIES are skipped (no draw layer wired here) and the
//! `Dust.NewDust*` internal rolls go with the dust pool (ANALYSIS "Dust &
//! gore", the `projectile_ai_misc` precedent). The `Lighting.AddLight`
//! calls feed the render light layer only — skipped (the misc precedent).
//!
//! Movement is the dispatch's `kill_step` (the `HandleMovement` default
//! arm's stand-in) run AFTER the AI, exactly as the C# runs
//! `HandleMovement` after `AI()` (`:17015`) — EXCEPT the 582 wrench,
//! whose aiStyle-109 `HandleMovement` arm (`:19512-19571`) bounces off
//! tiles instead of dying: that arm integrates its own movement (the
//! `ai_boomerang` precedent).
//!
//! CODE-ONLY MODE (W4-d11): the witness tests at the bottom are WRITTEN
//! NOT RUN (no compiler invoked on this branch); they were verified by
//! re-reading the cited C# blocks and hand arithmetic.

use glam::Vec2;
use tr_core::World;
use tr_core::loot::DropRng;

use crate::npc::Npc;
use crate::projectile::{Projectile, csign, proj_data, safe_normalize};

/// The port's `Kill()` stand-in for the AI early-kill paths (the
/// `projectile_ai_darts::kill` convention — the death hooks run on the
/// pool's `!active` transition).
fn kill(p: &mut Projectile) {
    p.kill_time_left = p.time_left;
    p.active = false;
    p.time_left = 0;
}

/// `AI()`'s aiStyle-101 block (`Projectile.cs:33165-33221`) — the
/// owner-anchored rays: the 536 MedusaHeadRay hangs off its 535 head
/// (the `ProjectileKey` lookup), the 591 MinecartMechLaser rides the
/// owning player's per-tick position delta. Both fade in over 20 ticks
/// (`alpha = (int)Lerp(0, 255, localAI[0] / 20)`) and die at the cap.
///
/// `parent` is the pool-resolved 535 head centre (the 536 only — the
/// dispatch probes `key_parent(ai[0], 535)`; the 535 row is aiStyle 100,
/// another worker's claim, so the probe reads None until it lands and
/// the ray dies exactly as the C#'s failed `TryGetActive` does).
/// `player` is the local player (the 591's `Main.player[ai[0]]` /
/// `Main.player[owner]` reads fold to it — the single-player port).
///
/// Movement: the arm pins `Center = anchor - velocity`, then the
/// dispatch's `kill_step` adds the velocity back (the generic
/// `UpdatePosition` add, `:20445` — 101 is not on the skip list), so the
/// net position is the anchor. ZERO `Main.rand` draws by construction.
pub(crate) fn ai_medusa_ray(
    p: &mut Projectile,
    parent: Option<Vec2>,
    player: Option<&crate::player::Player>,
    local_owner: u8,
) {
    // `:33167-33169` — the 20-tick fade-in.
    let num782 = 20.0f32;
    p.local_ai[0] += 1.0;
    p.alpha = (255.0 * (p.local_ai[0] / num782)) as i32;
    // `:33170-33181` — the per-type anchor selector.
    let (expected_type, num783) = match p.kind {
        536 => (535u16, 0i32),
        591 => (0, 1),
        _ => (0, -1),
    };
    match num783 {
        1 => {
            // `:33184-33207` — the 591's player-index anchor.
            let num784 = p.ai[0] as i32;
            let pl = player.filter(|_| num784 == local_owner as i32);
            let valid = pl.is_some_and(|pl| !pl.dead);
            if p.local_ai[0] >= num782 || !valid {
                kill(p);
                return;
            }
            let pl = pl.unwrap();
            if p.kind == 591 {
                // `:33194-33201` — ride the owner's position delta, then
                // die when the flight opposes the player's travel.
                p.pos -= p.vel;
                p.pos += Vec2::new(pl.x - pl.old_x, pl.y - pl.old_y);
                p.rot = p.vel.y.atan2(p.vel.x) + std::f32::consts::FRAC_PI_2;
                if csign(p.vel.x) != csign(pl.vx) && pl.vx != 0.0 {
                    kill(p);
                    return;
                }
            } else {
                // `:33205` — the non-591 case-1 anchor (no rowed kind
                // reaches it today; kept verbatim).
                let (cx, cy) = pl.center();
                let c = Vec2::new(cx, cy) - p.vel;
                p.pos = c - Vec2::new(p.w * 0.5, p.h * 0.5);
            }
        }
        0 => {
            // `:33209-33217` — the 536's key-linked head anchor.
            let Some(center) = parent else {
                kill(p);
                return;
            };
            if p.local_ai[0] >= num782 {
                kill(p);
                return;
            }
            let c = center - p.vel;
            p.pos = c - Vec2::new(p.w * 0.5, p.h * 0.5);
        }
        _ => {}
    }
    // `:33220` — the shared rotation tail.
    p.rot = p.vel.y.atan2(p.vel.x) + std::f32::consts::FRAC_PI_2;
    let _ = expected_type;
}

/// `Entity.Center` for a slot (the free-fn shape — a closure would hold
/// an immutable borrow across the arm's mutations).
fn slot_center(p: &Projectile) -> Vec2 {
    Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5)
}

/// `AI()`'s aiStyle-102 block (`Projectile.cs:33222-33447`) — the three
/// tower orbs: the 539 StardustJellyfishSmall (the 407 host), the 573
/// NebulaBolt (the 424 host) and the 574 NebulaEye (the 420 host, the
/// 576 laser volley). The machine: the host drag, the sign-homing accel
/// under the 6-px clamp, the per-kind dust gates + frames, and the
/// release tick's jittered volley (the 574 fires a 576 laser instead of
/// turning itself).
///
/// `host` is the pool-resolved `ai[1]` NPC slot (the C# indexes
/// `Main.npc[(int)ai[1]]` directly — the dispatch passes `npcs.get(idx)`
/// plus its index; the port's towers spawn the orbs with the host index
/// in `ai[1]`, the `ai_tower_damage_bolt` convention). `target_player`
/// is the host's `target` player centre (the single-player port folds
/// every `Main.player[target]` to the local player).
///
/// RNG census (the dust bodies skip, the gate rolls stay):
/// - 539 hold: `Next(12)` (`:33303`); release: `Next(2)` (`:33418`) then
///   `Next(50)` scale (`:33421`) + `NextFloat` fadeIn (`:33424`) when
///   taken, then `Next(3)` (`:33426`) + `Next(50)` (`:33429`) +
///   `NextFloat` (`:33432`) when taken.
/// - 573 hold: `Next(2)` (`:33322`); release: `Next(4)` (`:33437`).
/// - 574 hold: `Next(10)` (`:33334`).
/// - ALL kinds, the release tick: `Next(-50,51)` ×2 then
///   `Next(80,121)` ×2 (`:33380-33383`).
pub(crate) fn ai_tower_orb(
    p: &mut Projectile,
    world: &World,
    host: Option<(usize, &Npc)>,
    target_player: Option<(f32, f32)>,
    expert: bool,
    local_owner: u8,
    rng: &mut dyn DropRng,
) {
    // `:33234-33264` — the per-kind parameter table.
    // (host type, release tick, accel x/y, speed, !flag49 drag+accel,
    // spawn kind, spawn damage, host-lock flag50)
    let (host_type, release, ax, ay, speed, drag, spawn_kind, spawn_dmg, host_lock) = match p.kind
    {
        539 => (407u16, 210.0f32, 0.15f32, 0.075f32, 16.0f32, true, 0u16, 0i32, false),
        573 => (424, 90.0, 0.0, 0.0, 20.0, false, 0, 0, false),
        574 => (420, 180.0, 0.15, 0.075, 8.0, true, 576, if expert { 50 } else { 65 }, true),
        _ => return,
    };
        // `:33266-33275` — the 574's host lock: die when the host is gone,
    // else pin timeLeft = 2.
    if host_lock {
        let ok = host.is_some_and(|(_, h)| h.kind.net_id() == host_type as i32 && h.hp > 0);
        if !ok {
            kill(p);
            return;
        }
        p.time_left = 2;
    }
    // `:33276-33300` — the counter, the host drag and the homing accel.
    p.ai[0] += 1.0;
    let mut flag51 = true;
    if p.ai[0] < release {
        let host_ok = host.is_some_and(|(_, h)| h.kind.net_id() == host_type as i32 && h.hp > 0);
        if host_ok {
            let (_, h) = host.unwrap();
            // `:33283-33286` — the drag rides oldPos[1] (the two-tick-old
            // position); the port latches `old_x/old_y` once per tick
            // (the recorded one-tick stand-in, the ai_nebula_eye
            // precedent it replaces).
            if drag && (h.old_x != 0.0 || h.old_y != 0.0) {
                p.pos += Vec2::new(h.x - h.old_x, h.y - h.old_y);
            }
        } else {
            p.ai[0] = release;
            flag51 = false;
        }
        if flag51 && drag {
            let (_, h) = host.unwrap();
            let (hx, hy) = h.center();
            let c = slot_center(p);
            p.vel += Vec2::new(csign(hx - c.x), csign(hy - c.y)) * Vec2::new(ax, ay);
            let l = p.vel.length();
            if l > 6.0 {
                p.vel *= 6.0 / l;
            }
        }
        // `:33301-33353` — the per-kind hold-state dust + frames.
        match p.kind {
            539 => {
                if rng.next_range(0, 12) == 0 {
                    // `:33305-33308` — the 180 dust (body skipped).
                }
                p.frame_counter += 1;
                if p.frame_counter >= 4 {
                    p.frame_counter = 0;
                    p.frame += 1;
                    if p.frame >= crate::projectile::proj_frames(p.kind) as i32 {
                        p.frame = 0;
                    }
                }
                p.rot = p.vel.x * 0.1;
            }
            573 => {
                if rng.next_range(0, 2) == 0 {
                    // `:33324-33328` — the 242 dust (body skipped).
                }
                p.alpha = 255;
            }
            574 => {
                if rng.next_range(0, 10) == 0 {
                    // `:33336-33340` — the 242 dust (body skipped).
                }
                if flag51 {
                    // `:33344-33350` — the aim lerp: the host's target
                    // player when the LOS holds, else the velocity.
                    let mut num795 = p.vel.y.atan2(p.vel.x);
                    if let (Some((px, py)), Some((_, h))) = (target_player, host) {
                        let c = slot_center(p);
                        if crate::projectile::can_hit_line(
                            world, c.x, c.y, 0.0, 0.0, px, py, 0.0, 0.0,
                        ) {
                            let d = Vec2::new(px, py) - c;
                            num795 = d.y.atan2(d.x);
                        }
                        let _ = h;
                    }
                    p.rot = crate::npc::angle_lerp(
                        p.rot,
                        num795 + std::f32::consts::FRAC_PI_2,
                        0.2,
                    );
                }
                p.frame = 1;
            }
            _ => {}
        }
    }
    // `:33355-33402` — the release tick: the jittered volley.
    if p.ai[0] == release {
        // `:33357-33378` — the target: the host's target player when the
        // host lives (flag48 false), else FindClosest (the single-player
        // fold: the local player either way).
        let target = target_player.unwrap_or((0.0, 0.0));
        let c = slot_center(p);
        let mut value8 = Vec2::new(target.0, target.1) - c;
        // `:33380-33383` — the four jitter draws, in order.
        value8.x += rng.next_range(-50, 51) as f32;
        value8.y += rng.next_range(-50, 51) as f32;
        value8.x *= rng.next_range(80, 121) as f32 * 0.01;
        value8.y *= rng.next_range(80, 121) as f32 * 0.01;
        let mut vector113 = safe_normalize(value8);
        if vector113.is_nan() || vector113 == Vec2::ZERO {
            vector113 = Vec2::Y;
        }
        if spawn_kind == 0 {
            // `:33391-33392` — the 539/573 turn themselves into the shot.
            p.vel = vector113 * speed;
            p.net_update = true;
        } else {
            // `:33396-33400` — the 574 fires the 576 laser (the LOS gate;
            // netMode != 1 is always true single-player) and resets.
            if crate::projectile::can_hit_line(
                world, c.x, c.y, 0.0, 0.0, target.0, target.1, 0.0, 0.0,
            ) {
                p.wants_dart_spawns.push((
                    spawn_kind,
                    c.x,
                    c.y,
                    vector113.x * speed,
                    vector113.y * speed,
                    spawn_dmg,
                    1.0,
                    0.0,
                    0.0,
                    0,
                ));
            }
            p.ai[0] = 0.0;
        }
    }
    // `:33403-33406` — the pre-release early return.
    if p.ai[0] < release {
        return;
    }
    // `:33407` — the flight-state aim lerp.
    p.rot = crate::npc::angle_lerp(
        p.rot,
        p.vel.y.atan2(p.vel.x) + std::f32::consts::FRAC_PI_2,
        0.4,
    );
    // `:33408-33446` — the flight-state dust + frames.
    match p.kind {
        539 => {
            p.frame_counter += 1;
            if p.frame_counter >= 2 {
                p.frame_counter = 0;
                p.frame += 1;
                if p.frame >= crate::projectile::proj_frames(p.kind) as i32 {
                    p.frame = 0;
                }
            }
            if rng.next_range(0, 2) == 0 {
                let _ = rng.next_range(0, 50); // `:33421` scale bump
                let _ = rng.next_float(); // `:33424` fadeIn
            }
            if rng.next_range(0, 3) == 0 {
                let _ = rng.next_range(0, 50); // `:33429` scale bump
                let _ = rng.next_float(); // `:33432` fadeIn
            }
        }
        573 => {
            if rng.next_range(0, 4) == 0 {
                // `:33439-33443` — the 242 dust (body skipped).
            }
            p.alpha = 0;
        }
        _ => {}
    }
    let _ = local_owner;
}

/// `AI()`'s aiStyle-104 block (`Projectile.cs:33478-33511`) — the 565
/// BrainOfConfusion's confusion burst: the grow/fade-in leg
/// (`scale *= 1.01`, `alpha -= 7`) until alpha crosses 0, then the
/// shrink/fade-out leg (`scale *= 0.995`, `alpha += 3`) to the 250 kill;
/// the 7-tick 4-frame cycle and the upward drift (`velocity.Y -= 0.03`,
/// `velocity.X *= 0.97`). ZERO `Main.rand` draws by construction.
///
/// Movement: 104 is not on `UpdatePosition`'s skip list (`:20435`), so
/// the dispatch's `kill_step` integrates the drifted velocity.
pub(crate) fn ai_brain_of_confusion(p: &mut Projectile) {
    // `:33480-33498` — the two legs.
    if p.ai[0] == 1.0 {
        p.scale *= 0.995;
        p.alpha += 3;
        if p.alpha >= 250 {
            kill(p);
            // The C# does NOT return here — the frame + drift tail
            // still runs on the kill tick (`:33499-33510`).
        }
    } else {
        p.scale *= 1.01;
        p.alpha -= 7;
        if p.alpha < 0 {
            p.alpha = 0;
            p.ai[0] = 1.0;
        }
    }
    // `:33499-33508` — the 7-tick frame cycle (4 rows, `Main.cs:8478`).
    p.frame_counter += 1;
    if p.frame_counter > 6 {
        p.frame_counter = 0;
        p.frame += 1;
        if p.frame > 3 {
            p.frame = 0;
        }
    }
    // `:33509-33510` — the drift.
    p.vel.y -= 0.03;
    p.vel.x *= 0.97;
}

/// `AI()`'s aiStyle-105 block (`Projectile.cs:33512-33515`) ->
/// `AI_105_SporeSac` (`:40043-40192`) — the 567/568 Spore Sac traps: the
/// breathing scale (the ±90-tick `localAI[0]` flip, ±0.003/tick), the
/// `IndexForVisuals % 6` drift-direction table, the 240-tick drift cycle
/// (`localAI[1]`: -180..60, the sign flips at -60), the 5400-tick age
/// with the owner-distance penalty ladder (the unequipped-sac +100),
/// and the 340-Manhattan nearest-chaseable homing (the 4/41 blend) or
/// the 0.98 drag above 0.2 speed.
///
/// `index_for_visuals` is the slot index (the C# `key.Index`,
/// `Projectile.cs:552` — the port's slots ARE the identity). `owner` is
/// the owner player's centre; `spore_sac` the equipped flag
/// (`Player.cs:1990`). The `Lighting.AddLight` (`:40046`) feeds the
/// render light layer only — skipped. ZERO `Main.rand` draws.
pub(crate) fn ai_spore_sac(
    p: &mut Projectile,
    npcs: &[Npc],
    owner: Option<(f32, f32)>,
    spore_sac: bool,
    index_for_visuals: usize,
    local_owner: u8,
) {
    let center = slot_center(p);
    // `:40047` — the sac-trap flag (both rowed kinds are traps).
    let flag = p.kind == 567 || p.kind == 568;
    // `:40048-40060` — the breathing scale.
    p.local_ai[0] += 1.0;
    if p.local_ai[0] >= 90.0 {
        p.local_ai[0] *= -1.0;
    }
    if p.local_ai[0] >= 0.0 {
        p.scale += 0.003;
    } else {
        p.scale -= 0.003;
    }
    // `:40061` — the slow spin.
    p.rot += 0.0025 * p.scale;
    // `:40062-40084` — the drift-direction table.
    let mut num2 = 1.0f32;
    let mut num3 = 1.0f32;
    match index_for_visuals % 6 {
        0 => num3 *= -1.0,
        1 => num2 *= -1.0,
        2 => {
            num3 *= -1.0;
            num2 *= -1.0;
        }
        3 => num3 = 0.0,
        4 => num2 = 0.0,
        _ => {}
    }
    // `:40085-40099` — the 240-tick drift cycle.
    p.local_ai[1] += 1.0;
    if p.local_ai[1] > 60.0 {
        p.local_ai[1] = -180.0;
    }
    if p.local_ai[1] >= -60.0 {
        p.vel.x += 0.002 * num3;
        p.vel.y += 0.002 * num2;
    } else {
        p.vel.x -= 0.002 * num3;
        p.vel.y -= 0.002 * num2;
    }
    // `:40100-40162` — the age + the fade legs.
    p.ai[0] += 1.0;
    if p.ai[0] > 5400.0 {
        p.damage = 0;
        p.ai[1] = 1.0;
        if p.alpha < 255 {
            p.alpha += 5;
            if p.alpha > 255 {
                p.alpha = 255;
            }
        } else if p.owner == local_owner {
            kill(p);
            return;
        }
    } else {
        // `:40120-40153` — the owner-distance penalty ladder.
        let (ox, oy) = owner.unwrap_or((center.x, center.y));
        let mut num4 = (center - Vec2::new(ox, oy)).length() / 100.0;
        if num4 > 4.0 {
            num4 *= 1.1;
        }
        if num4 > 5.0 {
            num4 *= 1.2;
        }
        if num4 > 6.0 {
            num4 *= 1.3;
        }
        if num4 > 7.0 {
            num4 *= 1.4;
        }
        if num4 > 8.0 {
            num4 *= 1.5;
        }
        if num4 > 9.0 {
            num4 *= 1.6;
        }
        if num4 > 10.0 {
            num4 *= 1.7;
        }
        if flag && !spore_sac {
            num4 += 100.0;
        }
        p.ai[0] += num4;
        // `:40154-40161` — the fade-in to 50.
        if p.alpha > 50 {
            p.alpha -= 10;
            if p.alpha < 50 {
                p.alpha = 50;
            }
        }
    }
    // `:40163-40191` — the nearest-chaseable homing (the Manhattan 340
    // scan, the `CanBeChasedBy` gate) or the drag.
    let mut flag2 = false;
    let mut vector = Vec2::ZERO;
    let mut num5 = 340.0f32;
    for n in npcs {
        if !crate::projectile::chaseable(n) {
            continue;
        }
        let (nw, nh) = n.size();
        let num6 = n.x + (nw as i32 / 2) as f32;
        let num7 = n.y + (nh as i32 / 2) as f32;
        let num8 = (p.pos.x + (p.w as i32 / 2) as f32 - num6).abs()
            + (p.pos.y + (p.h as i32 / 2) as f32 - num7).abs();
        if num8 < num5 {
            num5 = num8;
            vector = crate::projectile::npc_center(n);
            flag2 = true;
        }
    }
    if flag2 {
        let mut vector2 = vector - center;
        vector2 = safe_normalize(vector2);
        vector2 *= 4.0;
        p.vel = (p.vel * 40.0 + vector2) / 41.0;
    } else if p.vel.length() > 0.2 {
        p.vel *= 0.98;
    }
}

/// `AI()`'s aiStyle-106 block (`Projectile.cs:33516-33552`) — the
/// 569-571 SporeGas clouds (the Spore Sac's released gas): the
/// velocity-driven spin (`rot += vel.X * 0.02`, then ±|vel.Y| * 0.02 by
/// the X sign), the 0.98 drag, and the 60-tick age gate — before it the
/// alpha fades down to 80 (-30/tick), after it up to 255 (+5/tick) then
/// the owner-gated kill. ZERO `Main.rand` draws by construction.
///
/// Movement: 106 is not on `UpdatePosition`'s skip list (`:20435`), so
/// the dispatch's `kill_step` integrates the dragged velocity.
pub(crate) fn ai_spore_gas(p: &mut Projectile, local_owner: u8) {
    // `:33518-33526` — the spin.
    p.rot += p.vel.x * 0.02;
    if p.vel.x < 0.0 {
        p.rot -= p.vel.y.abs() * 0.02;
    } else {
        p.rot += p.vel.y.abs() * 0.02;
    }
    // `:33527` — the drag.
    p.vel *= 0.98;
    // `:33528-33551` — the age gate + the alpha legs.
    p.ai[0] += 1.0;
    if p.ai[0] >= 60.0 {
        if p.alpha < 255 {
            p.alpha += 5;
            if p.alpha > 255 {
                p.alpha = 255;
            }
        } else if p.owner == local_owner {
            kill(p);
        }
    } else if p.alpha > 80 {
        p.alpha -= 30;
        if p.alpha < 80 {
            p.alpha = 80;
        }
    }
}

/// `AI()`'s aiStyle-107 block (`Projectile.cs:33553-33678`) — the 575
/// NebulaSphere (the nebula pillar's seeking sphere) and the 596
/// DesertDjinnCurse (the djinn's curse flame): the player-index homing
/// (`ai[0]` = the target player; the (v*(num-1) + dir*accel)/num blend
/// past the 40-px radius), the orphan cap (timeLeft 30 + ai[0] = -1 +
/// netUpdate), and the per-kind visual legs.
///
/// RNG census (the dust bodies skip, the gate rolls stay):
/// - 596: three `Next(3)` gates per tick (`:33619` — the C# RETURNS on
///   the first non-zero, so the later gates do not roll); at timeLeft < 4
///   ten `SelectRandom(27, 6)` picks = ten `Next(2)` (`:33644`,
///   `Utils.cs:2459-2462`).
/// - 575: ZERO draws (the AddLight feeds the render layer only).
///
/// `player` is the local player (the single-player fold of
/// `Main.player[ai[0]]`); `target_active` gates the homing leg.
pub(crate) fn ai_nebula_sphere(
    p: &mut Projectile,
    world: &World,
    player: Option<(f32, f32)>,
    target_active: bool,
    expert: bool,
    rng: &mut dyn DropRng,
) {
    // `:33555-33557` — the homing parameters.
    let mut num801 = 10.0f32;
    let mut num802 = 5.0f32;
    let num803 = 40.0f32;
        match p.kind {
        575 => {
            // `:33558-33581` — the sphere's fade + frame + light.
            if p.time_left > 30 && p.alpha > 0 {
                p.alpha -= 25;
            }
            if p.time_left > 30
                && p.alpha < 128
                && crate::projectile::solid_collision(world, p.pos, p.w, p.h)
            {
                p.alpha = 128;
            }
            if p.alpha < 0 {
                p.alpha = 0;
            }
            p.frame_counter += 1;
            if p.frame_counter > 4 {
                p.frame_counter = 0;
                p.frame += 1;
                if p.frame >= 4 {
                    p.frame = 0;
                }
            }
            // `:33580` — Lighting.AddLight(0.5, 0.1, 0.3): render-only.
        }
        596 => {
            // `:33582-33652` — the curse flame.
            num801 = 10.0;
            num802 = 7.5;
            if p.time_left > 30 && p.alpha > 0 {
                p.alpha -= 25;
            }
            if p.time_left > 30
                && p.alpha < 128
                && crate::projectile::solid_collision(world, p.pos, p.w, p.h)
            {
                p.alpha = 128;
            }
            if p.alpha < 0 {
                p.alpha = 0;
            }
            p.frame_counter += 1;
            if p.frame_counter > 4 {
                p.frame_counter = 0;
                p.frame += 1;
                if p.frame >= 4 {
                    p.frame = 0;
                }
            }
            // `:33606-33614` — the dust scale ladder (the dust bodies
            // skip; the scale feeds only them).
            p.ai[1] += 1.0;
            // `:33616` — num805 = ai[1]/180 * 2pi: dust-only, skipped.
            // `:33617-33630` — the three dust gates; the C# RETURNS from
            // the whole AI on the first non-zero roll (the homing tail
            // below does not run that tick).
            for _ in 0..3 {
                if rng.next_range(0, 3) != 0 {
                    return;
                }
                // The 27 dust + its ring placement — body skipped.
            }
            // `:33631-33651` — the final flash: the box grows to 60x60
            // around the centre, the damage re-arms (30 / expert 22),
            // ten dust kicks (the SelectRandom picks stay on the stream).
            if p.time_left < 4 {
                let num807 = if expert { 22 } else { 30 };
                let c = slot_center(p);
                p.pos = c;
                p.w = 60.0;
                p.h = 60.0;
                p.pos = c - Vec2::new(p.w * 0.5, p.h * 0.5);
                p.damage = num807;
                for _ in 0..10 {
                    let _ = rng.next_range(0, 2); // SelectRandom(27, 6)
                }
            }
        }
        _ => {}
    }
    // `:33653-33677` — the player-index homing / the orphan cap.
    let num809 = p.ai[0] as i32;
    if num809 >= 0 && target_active {
        if let Some((px, py)) = player {
            let c = slot_center(p);
            let target = Vec2::new(px, py);
            if (c - target).length() > num803 {
                let mut vector114 = safe_normalize(target - c);
                if vector114.is_nan() || vector114 == Vec2::ZERO {
                    vector114 = Vec2::Y;
                }
                p.vel = (p.vel * (num801 - 1.0) + vector114 * num802) / num801;
            }
        }
    } else {
        if p.time_left > 30 {
            p.time_left = 30;
        }
        if p.ai[0] != -1.0 {
            p.ai[0] = -1.0;
            p.net_update = true;
        }
    }
}

/// `AI()`'s aiStyle-109 block (`Projectile.cs:34026-34065`) — the 582
/// MechanicWrench: the first-tick speed latch (`localAI[1] =
/// velocity.Length()`), the 30-tick outbound wait (`ai[0] == 0`,
/// `localAI[0]++`), then the return leg (`ai[0] == 1`): the Mechanic
/// (124) host validity (die when gone), the `tileCollide = false` drop,
/// the kill inside the latch radius, and the 0.04 lerp toward the host
/// at the latched speed; the shared `rotation += pi/10` spin.
///
/// `host` is the pool-resolved `ai[1]` NPC slot (the C#
/// `Main.npc[(int)ai[1]]`). The `lazyNPCOwnedProjectileSearchArray`
/// write (`:34050`) is the town-NPC owned-projectile latch — unported
/// (the `npc_ai_town.rs:1594-1595` cited skip). ZERO `Main.rand` draws.
///
/// Movement: the 582 rides `HandleMovement`'s aiStyle-109 arm
/// (`:19512-19571`) — the tile bounce (reverse BOTH axes + `ai[0] = 1` +
/// netUpdate + the Dig thud), NOT the generic kill. This fn integrates
/// its own movement (the `ai_boomerang` precedent); the dispatch must
/// NOT call `kill_step` for it.
pub(crate) fn ai_mechanic_wrench(
    p: &mut Projectile,
    world: &World,
    host: Option<&Npc>,
) {
    // `:34028-34031` — the speed latch.
    if p.local_ai[1] == 0.0 {
        p.local_ai[1] = p.vel.length();
    }
    // `:34032-34063` — the two legs.
    if p.ai[0] == 0.0 {
        p.local_ai[0] += 1.0;
        if p.local_ai[0] > 30.0 {
            p.ai[0] = 1.0;
            p.local_ai[0] = 0.0;
            // `:34039`'s return: the spin does NOT run this tick.
            wrench_movement(p, world);
            return;
        }
    } else if p.ai[0] == 1.0 {
        // `:34045-34049` — the host validity (type 124, the Mechanic).
        let ok = host.is_some_and(|h| h.kind.net_id() == 124 && h.hp > 0);
        if !ok {
            kill(p);
            return;
        }
        let h = host.unwrap();
        // `:34050` — the lazyNPCOwnedProjectileSearchArray latch: cited
        // skip (the town-NPC seam is unported).
        let zero3 = crate::projectile::npc_center(h);
        p.tile_collide = false;
        let num820 = p.local_ai[1];
        let center = slot_center(p);
        let mut value9 = zero3 - center;
        if value9.length() < num820 {
            kill(p);
            return;
        }
        value9 = safe_normalize(value9);
        value9 *= num820;
        p.vel = Vec2::lerp(p.vel, value9, 0.04);
    }
    // `:34064` — the shared spin.
    p.rot += std::f32::consts::PI / 10.0;
    wrench_movement(p, world);
}

/// `HandleMovement`'s aiStyle-109 arm (`Projectile.cs:19512-19571`) for
/// the 582: the tile probe reverses BOTH axes from `lastVelocity` and
/// arms the return (`ai[0] = 1`, `:19563-19568`), plays the Dig thud
/// (`PlaySound(0, …)`, `:19571`) and sets netUpdate; the post-bounce
/// velocity integrates (`UpdatePosition`, `:20445`). The port's
/// `collides_rect` stand-in for `Collision.TileCollision` (the
/// `ai_boomerang` recorded deviation).
fn wrench_movement(p: &mut Projectile, world: &World) {
    if p.tile_collide {
        let last = p.vel;
        let nx = p.pos.x + p.vel.x;
        let ny = p.pos.y + p.vel.y;
        let mut hit_x = crate::player::collides_rect(world, nx, p.pos.y, p.w, p.h);
        let mut hit_y = crate::player::collides_rect(world, p.pos.x, ny, p.w, p.h);
        hit_x |= p.pos.x <= 0.0 || p.pos.x + p.w >= world.width as f32 * 16.0;
        hit_y |= p.pos.y <= 0.0 || p.pos.y + p.h >= world.height as f32 * 16.0;
        if hit_x || hit_y {
            // `:19563-19568` — reverse both axes, arm the return.
            p.vel = -last;
            p.ai[0] = 1.0;
            p.net_update = true;
            // `:19571` — the Dig thud (the int-id 0 cue).
            if p.wants_sound.is_none() {
                p.wants_sound = Some((0, 0, p.pos.x, p.pos.y, 0.0));
            }
        }
    }
    p.pos += p.vel;
}

/// `AI()`'s aiStyle-110 block (`Projectile.cs:34066-34109`) — the 584
/// NurseSyringeHeal: the first-tick speed latch, the town-NPC host
/// (`ai[0]` index) seek — die when the host is gone or not town, kill +
/// heal on the latch-radius catch or the hitbox overlap (the heal is
/// `min(lifeMax - life, 20)`, positive only), else the 0.04 lerp toward
/// the host at the latched speed with the Y floor (`value.Y =
/// max(value.Y, vel.Y)` then `++`) and the `vel.X * 0.05` spin.
///
/// `host` is the pool-resolved `ai[0]` NPC slot (the C#
/// `Main.npc[(int)ai[0]]`). The `HealEffect` (`:34089`) is the combat-
/// text visual — the heal itself (`life += num822`) is the state write;
/// the visual is a cited skip (no combat-text layer in the pool). ZERO
/// `Main.rand` draws.
///
/// Movement: 110 is not on `UpdatePosition`'s skip list (`:20435`), so
/// the dispatch's `kill_step` integrates the seek velocity.
pub(crate) fn ai_nurse_syringe(p: &mut Projectile, host: Option<&mut Npc>) {
    // `:34068-34071` — the speed latch.
    if p.local_ai[1] == 0.0 {
        p.local_ai[1] = p.vel.length();
    }
    // `:34073-34108` — the town-host gate.
    let Some(h) = host.filter(|h| h.hp > 0 && h.kind.is_town_npc_like()) else {
        kill(p);
        return;
    };
    let zero4 = crate::projectile::npc_center(h);
    let num821 = p.local_ai[1];
    let center = slot_center(p);
    let mut value10 = zero4 - center;
    // `:34078-34091` — the catch: the latch radius OR the hitbox overlap.
    let (hw, hh) = h.size();
    let overlap = p.pos.x + p.w > h.x
        && p.pos.x < h.x + hw
        && p.pos.y + p.h > h.y
        && p.pos.y < h.y + hh;
    if value10.length() < num821 || overlap {
        kill(p);
        let mut num822 = h.max_hp - h.hp;
        if num822 > 20 {
            num822 = 20;
        }
        if num822 > 0 {
            h.hp += num822;
            // `:34089`'s HealEffect: the combat-text visual, cited skip.
        }
    } else {
        // `:34094-34102` — the seek: normalize to the latched speed, the
        // Y floor + the ++, the 0.04 lerp, the spin.
        value10 = safe_normalize(value10);
        value10 *= num821;
        if value10.y < p.vel.y {
            value10.y = p.vel.y;
        }
        value10.y += 1.0;
        p.vel = Vec2::lerp(p.vel, value10, 0.04);
        p.rot += p.vel.x * 0.05;
    }
}

// ---------------------------------------------------------------------------
// Witnesses (W4-d11 code-only round — WRITTEN NOT RUN).

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::ProjectilePool;

    /// A slot carrying its `SetDefaults` row slice (the
    /// `projectile_ai_misc` fixture shape).
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
        p.extra_updates = d.extra_updates;
        p
    }

    fn world() -> World {
        World::new("t".into(), 300, 200)
    }

    /// The two rows keep their census styles + boxes (`Projectile.cs`
    /// `:6107-6119` the 536, `:6397-6408` the 591).
    #[test]
    fn w4_d11_101_the_rows_carry_their_styles() {
        let d = proj_data(536).expect("the 536 row");
        assert_eq!(d.ai_style, 101);
        assert_eq!((d.w, d.h), (10.0, 10.0), ":6109-6110");
        assert!(!d.tile_collide, ":6114");
        let d = proj_data(591).expect("the 591 row");
        assert_eq!(d.ai_style, 101);
        assert_eq!((d.w, d.h), (8.0, 8.0), ":6399-6400");
        assert!(!d.tile_collide, ":6404");
    }

    /// `:33168-33169` — the alpha ramp is `(int)(255 * t / 20)`: tick 1
    /// reads 12, tick 10 reads 127, tick 20 reads 255 (then the cap kill).
    #[test]
    fn w4_d11_101_the_alpha_ramp_truncates_the_lerp() {
        let mut p = proj(536);
        p.vel = Vec2::new(4.0, 0.0);
        ai_medusa_ray(&mut p, Some(Vec2::new(800.0, 400.0)), None, 0);
        assert_eq!(p.alpha, 12, "tick 1: 255/20 = 12.75 -> 12");
        for _ in 0..9 {
            ai_medusa_ray(&mut p, Some(Vec2::new(800.0, 400.0)), None, 0);
        }
        assert_eq!(p.alpha, 127, "tick 10: 127.5 -> 127");
        assert!(p.active);
    }

    /// `:33211-33214` — the 536 dies when the key probe finds no live
    /// 535 head (the failed `TryGetActive`), and at the 20-tick cap.
    #[test]
    fn w4_d11_101_the_ray_dies_without_its_head() {
        let mut p = proj(536);
        ai_medusa_ray(&mut p, None, None, 0);
        assert!(!p.active, "no head -> Kill");
        let mut p = proj(536);
        for _ in 0..19 {
            ai_medusa_ray(&mut p, Some(Vec2::new(800.0, 400.0)), None, 0);
        }
        assert!(p.active, "tick 19 still flies");
        ai_medusa_ray(&mut p, Some(Vec2::new(800.0, 400.0)), None, 0);
        assert!(!p.active, "tick 20: localAI[0] >= 20 -> Kill");
    }

    /// `:33216` — the 536 pins `Center = head.Center - velocity`; the
    /// dispatch's `kill_step` adds the velocity back, so the net centre
    /// is the head's.
    #[test]
    fn w4_d11_101_the_ray_pins_to_the_head_minus_velocity() {
        let mut p = proj(536);
        p.vel = Vec2::new(4.0, -2.0);
        let head = Vec2::new(800.0, 400.0);
        ai_medusa_ray(&mut p, Some(head), None, 0);
        let want = head - p.vel - Vec2::new(p.w * 0.5, p.h * 0.5);
        assert_eq!(p.pos, want, "Center = head - velocity");
        // The rotation tail: velocity.ToRotation() + pi/2.
        let want_rot = (-2.0f32).atan2(4.0) + std::f32::consts::FRAC_PI_2;
        assert!((p.rot - want_rot).abs() < 1e-5);
    }

    /// `:33194-33201` — the 591 rides the owner's position delta and
    /// dies when its flight opposes the player's non-zero travel.
    #[test]
    fn w4_d11_101_the_mech_laser_rides_the_player_delta() {
        let w = world();
        let mut pl = crate::player::Player::spawn(&w);
        pl.x = 500.0;
        pl.old_x = 496.0; // a +4 px tick delta
        pl.y = 300.0;
        pl.old_y = 300.0;
        pl.vx = 4.0;
        let mut p = proj(591);
        p.ai[0] = 0.0; // the local player's index
        p.vel = Vec2::new(6.0, 0.0);
        let before = p.pos;
        ai_medusa_ray(&mut p, None, Some(&pl), 0);
        assert!(p.active);
        // pos -= vel, then += the player delta (the kill_step adds vel
        // back, netting the delta).
        assert_eq!(p.pos, before - p.vel + Vec2::new(4.0, 0.0));
        // Opposing flight against a moving player -> Kill.
        pl.vx = -4.0;
        ai_medusa_ray(&mut p, None, Some(&pl), 0);
        assert!(!p.active, "sign(vel.X) != sign(player.vx) -> Kill");
    }

    /// `:33187` — a dead player (or a foreign index) kills the 591.
    #[test]
    fn w4_d11_101_the_mech_laser_dies_with_its_player() {
        let w = world();
        let mut pl = crate::player::Player::spawn(&w);
        pl.dead = true;
        let mut p = proj(591);
        p.ai[0] = 0.0;
        ai_medusa_ray(&mut p, None, Some(&pl), 0);
        assert!(!p.active, "dead player -> Kill");
        let mut p = proj(591);
        p.ai[0] = 3.0; // not the local index
        ai_medusa_ray(&mut p, None, None, 0);
        assert!(!p.active, "no resolved player -> Kill");
        let _ = ProjectilePool::new();
    }

    /// A draw-recording `Main.rand` stand-in (the `projectile_ai_misc`
    /// CensusRng shape): `next_range` returns `min`, `next_float` 0.
    struct CensusRng {
        ranges: Vec<(i32, i32)>,
        floats: usize,
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
    }

    fn census() -> CensusRng {
        CensusRng { ranges: Vec::new(), floats: 0 }
    }

    fn host_npc(kind: crate::npc::NpcKind, x: f32, y: f32) -> Npc {
        let mut n = Npc::new(kind, x, y, 1, 0.0);
        n.max_hp = 400;
        n.hp = 400;
        n
    }

    /// The three orb rows + the 576 laser row (`Projectile.cs:6142-6150`,
    /// `:6230-6238`, `:6239-6247`, `:6258-6269`).
    #[test]
    fn w4_d11_102_the_orb_rows_carry_their_styles() {
        let d = proj_data(539).unwrap();
        assert_eq!((d.ai_style, d.w, d.h, d.penetrate, d.time_left), (102, 18.0, 30.0, -1, 600));
        let d = proj_data(573).unwrap();
        assert_eq!((d.ai_style, d.w, d.h, d.penetrate, d.time_left), (102, 18.0, 30.0, -1, 600));
        let d = proj_data(574).unwrap();
        assert_eq!((d.ai_style, d.w, d.h, d.penetrate, d.time_left), (102, 18.0, 18.0, 1, 600));
        assert!(!d.tile_collide, ":6246");
        let d = proj_data(576).unwrap();
        assert_eq!(d.ai_style, 1);
        assert_eq!((d.w, d.h, d.extra_updates, d.alpha), (4.0, 4.0, 2, 255));
        assert_eq!(d.scale, 1.2, ":6267");
        assert_eq!(crate::projectile::proj_frames(539), 4, "Main.cs:8549");
        assert_eq!(crate::projectile::proj_frames(574), 2, "Main.cs:8551");
    }

    /// `:33266-33275` — the 574's host lock: no live 420 host -> Kill;
    /// a live host pins timeLeft = 2.
    #[test]
    fn w4_d11_102_the_eye_dies_without_its_host() {
        let w = world();
        let mut rng = census();
        let mut p = proj(574);
        ai_tower_orb(&mut p, &w, None, Some((900.0, 400.0)), false, 0, &mut rng);
        assert!(!p.active, "no host -> Kill");
        let h = host_npc(crate::npc::NpcKind::NebulaFloater, 800.0, 400.0);
        let mut p = proj(574);
        p.time_left = 600;
        ai_tower_orb(&mut p, &w, Some((0, &h)), Some((900.0, 400.0)), false, 0, &mut rng);
        assert!(p.active);
        assert_eq!(p.time_left, 2, ":33274");
        assert_eq!(p.ai[0], 1.0, ":33276");
    }

    /// `:33293-33299` — the 539's sign-homing accel (0.15, 0.075) under
    /// the 6-px clamp; the host drag rides oldPos (`:33285`).
    #[test]
    fn w4_d11_102_the_jellyfish_homes_with_the_sign_accel() {
        let w = world();
        let mut rng = census();
        let mut h = host_npc(crate::npc::NpcKind::StardustJellyfishBig, 800.0, 400.0);
        h.old_x = 796.0; // a +4 px host delta
        h.old_y = 400.0;
        let mut p = proj(539);
        p.pos = Vec2::new(600.0, 400.0);
        let before = p.pos;
        ai_tower_orb(&mut p, &w, Some((0, &h)), Some((900.0, 400.0)), false, 0, &mut rng);
        // The drag: pos += host delta (+4, 0).
        assert_eq!(p.pos, before + Vec2::new(4.0, 0.0));
        // The accel: sign(host - center) = (+1, ~0) -> (+0.15, +0.075)
        // (the y sign rides the centre delta, not zero).
        assert!(p.vel.x > 0.0 && p.vel.x <= 0.15 + 1e-5);
        assert!(p.vel.length() <= 6.0 + 1e-5, "the 6-px clamp");
        // The hold-state dust gate rolled Next(12).
        assert_eq!(rng.ranges, vec![(0, 12)], ":33303");
    }

    /// `:33355-33401` — the 574's release tick: the four jitter draws in
    /// order, the 576 laser spawn request (LOS open), the ai[0] reset.
    #[test]
    fn w4_d11_102_the_eye_fires_the_576_at_release() {
        let w = world();
        let mut rng = census();
        let h = host_npc(crate::npc::NpcKind::NebulaFloater, 800.0, 400.0);
        let mut p = proj(574);
        p.ai[0] = 179.0; // one tick under the release
        ai_tower_orb(&mut p, &w, Some((0, &h)), Some((900.0, 400.0)), false, 0, &mut rng);
        assert_eq!(p.ai[0], 0.0, ":33400's reset");
        assert_eq!(p.wants_dart_spawns.len(), 1, "the 576 fires");
        let (kind, _x, _y, _vx, _vy, dmg, kb, ..) = p.wants_dart_spawns[0];
        assert_eq!(kind, 576);
        assert_eq!(dmg, 65, ":33258 (non-expert)");
        assert_eq!(kb, 1.0, ":33398");
        // The jitter census: the hold block is skipped (the counter
        // reaches the release tick), so only the four release draws.
        assert_eq!(
            rng.ranges,
            vec![(-50, 51), (-50, 51), (80, 121), (80, 121)],
            ":33380-33383"
        );
        // Expert damage 50.
        let mut rng = census();
        let mut p = proj(574);
        p.ai[0] = 179.0;
        ai_tower_orb(&mut p, &w, Some((0, &h)), Some((900.0, 400.0)), true, 0, &mut rng);
        assert_eq!(p.wants_dart_spawns[0].5, 50, ":33261");
    }

    /// `:33389-33392` — the 539/573 turn THEMSELVES into the shot at
    /// release (velocity = normalized jitter * speed, netUpdate).
    #[test]
    fn w4_d11_102_the_jellyfish_turns_itself_at_release() {
        let w = world();
        let mut rng = census();
        let h = host_npc(crate::npc::NpcKind::StardustJellyfishBig, 800.0, 400.0);
        let mut p = proj(539);
        p.ai[0] = 209.0;
        ai_tower_orb(&mut p, &w, Some((0, &h)), Some((900.0, 400.0)), false, 0, &mut rng);
        assert!(p.net_update, ":33392");
        assert!(p.wants_dart_spawns.is_empty(), "no child spawn");
        // CensusRng returns min: jitter (-50,-50) * 0.8 -> the shot
        // flies up-left at 16 px; the flight-state dust gates follow.
        assert!(p.vel.x < 0.0 && p.vel.y < 0.0);
        assert!((p.vel.length() - 16.0).abs() < 1e-3, ":33391 speed 16");
        assert_eq!(
            rng.ranges,
            vec![(-50, 51), (-50, 51), (80, 121), (80, 121), (0, 2), (0, 50), (0, 3), (0, 50)],
            "the release jitter + the flight dust census (the hold block is skipped)"
        );
        assert_eq!(rng.floats, 2, "the two fadeIn draws");
    }

    /// `:33290-33291` — a dead host jumps ai[0] to the release tick (the
    /// orb fires immediately, no drag).
    #[test]
    fn w4_d11_102_a_dead_host_jumps_to_release() {
        let w = world();
        let mut rng = census();
        let mut p = proj(539);
        ai_tower_orb(&mut p, &w, None, Some((900.0, 400.0)), false, 0, &mut rng);
        assert!(p.active, "the 539 has no host lock");
        // ai[0] jumped to 210 then the release block ran (no reset for
        // the 539 — it keeps flying).
        assert_eq!(p.ai[0], 210.0);
        assert!(p.net_update, "the release fired");
    }

    /// The 565 row (`Projectile.cs:6161-6171`) + the 4-frame sheet
    /// (`Main.cs:8478`).
    #[test]
    fn w4_d11_104_the_brain_row_carries_its_style() {
        let d = proj_data(565).unwrap();
        assert_eq!(d.ai_style, 104);
        assert_eq!((d.w, d.h, d.penetrate), (28.0, 28.0, -1));
        assert!(!d.tile_collide, ":6167");
        assert_eq!((d.alpha, d.scale), (255, 0.8), ":6169-6170");
        assert_eq!(crate::projectile::proj_frames(565), 4);
    }

    /// `:33491-33497` — the fade-in leg: scale *= 1.01, alpha -= 7; at
    /// alpha < 0 the clamp arms ai[0] = 1. From 255: 37 ticks reach -4
    /// -> 0 (255 - 36*7 = 3 still positive; tick 37 crosses).
    #[test]
    fn w4_d11_104_the_fade_in_arms_the_shrink_leg() {
        let mut p = proj(565);
        for _ in 0..36 {
            ai_brain_of_confusion(&mut p);
        }
        assert_eq!(p.alpha, 3, "255 - 36*7");
        assert_eq!(p.ai[0], 0.0, "still fading in");
        let s36 = 0.8f32 * 1.01f32.powi(36);
        assert!((p.scale - s36).abs() < 1e-4, "the 1.01 growth");
        ai_brain_of_confusion(&mut p);
        assert_eq!(p.alpha, 0, "the clamp");
        assert_eq!(p.ai[0], 1.0, ":33496");
        // The drift ran every tick: vy = -0.03 * 37.
        assert!((p.vel.y + 0.03 * 37.0).abs() < 1e-4, ":33509");
    }

    /// `:33482-33487` — the shrink leg: scale *= 0.995, alpha += 3; the
    /// kill at alpha >= 250 (84 ticks from 0: 83*3 = 249, tick 84 = 252).
    /// The frame + drift tail runs on the kill tick too.
    #[test]
    fn w4_d11_104_the_shrink_leg_kills_at_250() {
        let mut p = proj(565);
        p.ai[0] = 1.0;
        p.alpha = 0;
        for _ in 0..83 {
            ai_brain_of_confusion(&mut p);
        }
        assert_eq!(p.alpha, 249);
        assert!(p.active);
        ai_brain_of_confusion(&mut p);
        assert_eq!(p.alpha, 252);
        assert!(!p.active, "alpha >= 250 -> Kill");
        // The tail ran on the kill tick: the drift accumulated 84 ticks.
        assert!((p.vel.y + 0.03 * 84.0).abs() < 1e-4);
    }

    /// `:33499-33508` — the frame cycle: frameCounter > 6 advances the
    /// frame, wrapping 3 -> 0 (the 7-tick cadence, NOT >= 6).
    #[test]
    fn w4_d11_104_the_frame_cycle_wraps_at_three() {
        let mut p = proj(565);
        for _ in 0..6 {
            ai_brain_of_confusion(&mut p);
        }
        assert_eq!(p.frame, 0, "six ticks: counter 6, no advance");
        ai_brain_of_confusion(&mut p);
        assert_eq!(p.frame, 1, "tick 7: counter 7 > 6 -> frame 1");
        assert_eq!(p.frame_counter, 0);
        for _ in 0..21 {
            ai_brain_of_confusion(&mut p);
        }
        assert_eq!(p.frame, 0, "three more cycles wrap 3 -> 0");
    }

    /// The 567/568 rows (`Projectile.cs:6185-6205`) + the shared 567
    /// immunity identity (`UseGroupStaticImmunity(567)`, `:6204`).
    #[test]
    fn w4_d11_105_the_trap_rows_carry_their_styles() {
        let d = proj_data(567).unwrap();
        assert_eq!((d.ai_style, d.w, d.h, d.penetrate), (105, 14.0, 14.0, 1));
        assert_eq!((d.alpha, d.time_left), (255, 3600));
        assert!(!d.tile_collide, ":6202");
        assert!(d.uses_id_static_immunity, ":6204");
        assert_eq!(d.id_static_hit_cooldown, 10, ":10679");
        assert_eq!(d.immunity_identity, 567, "the shared group");
        let d = proj_data(568).unwrap();
        assert_eq!((d.ai_style, d.w, d.h), (105, 16.0, 16.0));
        assert_eq!(d.immunity_identity, 567, "both traps share 567");
    }

    /// `:40048-40061` — the breathing scale: +0.003 while localAI[0] >= 0,
    /// the flip at 90 (localAI[0] = -90), then -0.003; the spin rides the
    /// CURRENT scale.
    #[test]
    fn w4_d11_105_the_scale_breathes_over_180_ticks() {
        let mut p = proj(567);
        let npcs: Vec<Npc> = Vec::new();
        for _ in 0..89 {
            ai_spore_sac(&mut p, &npcs, Some(p.pos), true, 5, 0);
        }
        assert_eq!(p.local_ai[0], 89.0);
        assert!((p.scale - (1.0 + 0.003 * 89.0)).abs() < 1e-4);
        ai_spore_sac(&mut p, &npcs, Some(p.pos), true, 5, 0);
        assert_eq!(p.local_ai[0], -90.0, "the 90-tick flip");
        assert!((p.scale - (1.0 + 0.003 * 89.0 - 0.003)).abs() < 1e-4, "the shrink leg");
    }

    /// `:40062-40099` — the IndexForVisuals % 6 drift table + the
    /// localAI[1] cycle: index 3 zeroes num3 (no X drift), index 4 zeroes
    /// num2 (no Y drift); the cycle wraps 61 -> -180.
    #[test]
    fn w4_d11_105_the_drift_table_and_cycle() {
        let npcs: Vec<Npc> = Vec::new();
        let mut p = proj(567);
        ai_spore_sac(&mut p, &npcs, Some(p.pos), true, 3, 0);
        assert_eq!(p.vel.x, 0.0, "index 3: num3 = 0");
        assert!((p.vel.y - 0.002).abs() < 1e-6, "num2 = 1");
        let mut p = proj(567);
        ai_spore_sac(&mut p, &npcs, Some(p.pos), true, 4, 0);
        assert!((p.vel.x - 0.002).abs() < 1e-6);
        assert_eq!(p.vel.y, 0.0, "index 4: num2 = 0");
        // The cycle: 60 ticks reach 60, tick 61 wraps to -180 (the
        // negative leg subtracts).
        let mut p = proj(567);
        for _ in 0..61 {
            ai_spore_sac(&mut p, &npcs, Some(p.pos), true, 5, 0);
        }
        assert_eq!(p.local_ai[1], -180.0, ":40088");
    }

    /// `:40149-40153` — the unequipped sac's +100 age penalty; the
    /// equipped sac ages by 1 + the distance ladder only.
    #[test]
    fn w4_d11_105_the_unequipped_sac_ages_100_per_tick() {
        let npcs: Vec<Npc> = Vec::new();
        let mut p = proj(567);
        let c = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
        ai_spore_sac(&mut p, &npcs, Some((c.x, c.y)), false, 5, 0);
        assert_eq!(p.ai[0], 101.0, "1 + 0 distance + 100 penalty");
        let mut p = proj(567);
        ai_spore_sac(&mut p, &npcs, Some((c.x, c.y)), true, 5, 0);
        assert_eq!(p.ai[0], 1.0, "equipped: no penalty");
        // The fade-in: alpha 255 -> 245 on the first tick.
        assert_eq!(p.alpha, 245, ":40156");
    }

    /// `:40100-40117` — past 5400 the trap zeroes damage, latches ai[1],
    /// fades +5/tick and dies at 255 (the owner gate).
    #[test]
    fn w4_d11_105_the_old_trap_fades_and_dies() {
        let npcs: Vec<Npc> = Vec::new();
        let mut p = proj(567);
        p.ai[0] = 5400.0;
        p.alpha = 250;
        p.damage = 30;
        ai_spore_sac(&mut p, &npcs, Some(p.pos), true, 5, 0);
        assert_eq!(p.ai[0], 5401.0);
        assert_eq!(p.damage, 0, ":40103");
        assert_eq!(p.ai[1], 1.0, ":40104");
        assert_eq!(p.alpha, 255, "the +5 clamp");
        assert!(p.active, "the fade leg does not kill");
        ai_spore_sac(&mut p, &npcs, Some(p.pos), true, 5, 0);
        assert!(!p.active, "alpha 255 + owner -> Kill");
    }

    /// `:40163-40191` — the nearest-chaseable homing: the Manhattan 340
    /// scan picks the closest, the velocity blends (v*40 + dir*4)/41;
    /// with no target the 0.98 drag runs above 0.2 speed.
    #[test]
    fn w4_d11_105_the_trap_homes_the_nearest_chaseable() {
        let mut p = proj(567);
        p.pos = Vec2::new(1000.0, 1000.0);
        p.vel = Vec2::ZERO;
        let mut n = host_npc(crate::npc::NpcKind::Zombie, 1100.0, 1000.0);
        n.max_hp = 40;
        n.hp = 40;
        let npcs = vec![n];
        // The expected direction: the zombie's centre minus the trap's
        // PRE-drift centre (the C# reads base.Center before the blend),
        // normalized to 4. The drift added (0.002, 0.002) first.
        let (nw, nh) = npcs[0].size();
        let npc_c = Vec2::new(1100.0 + nw * 0.5, 1000.0 + nh * 0.5);
        let trap_c = Vec2::new(1000.0 + 7.0, 1000.0 + 7.0);
        let dir = (npc_c - trap_c).normalize() * 4.0;
        ai_spore_sac(&mut p, &npcs, Some((1000.0, 1000.0)), true, 5, 0);
        let want = (Vec2::new(0.002, 0.002) * 40.0 + dir) / 41.0;
        assert!((p.vel - want).length() < 1e-4, "the 4/41 blend: {:?}", p.vel);
        // No chaseable: the drag.
        let mut p = proj(567);
        p.vel = Vec2::new(1.0, 0.0);
        ai_spore_sac(&mut p, &[], Some(p.pos), true, 5, 0);
        // The drift adds 0.002 then the 0.98 drag: (1.002)*0.98.
        assert!((p.vel.x - 1.002 * 0.98).abs() < 1e-5, ":40190");
    }

    /// The 569-571 rows (`Projectile.cs:6206-6218`) + the shared 569
    /// immunity identity (`UseGroupStaticImmunity(569)`, `:6217`).
    #[test]
    fn w4_d11_106_the_gas_rows_carry_their_styles() {
        for kind in [569u16, 570, 571] {
            let d = proj_data(kind).unwrap();
            assert_eq!((d.ai_style, d.w, d.h, d.penetrate), (106, 32.0, 32.0, -1));
            assert_eq!((d.alpha, d.time_left), (255, 3600));
            assert!(!d.tile_collide, ":6215");
            assert!(d.uses_id_static_immunity, ":6217");
            assert_eq!(d.immunity_identity, 569, "the shared group");
        }
    }

    /// `:33518-33527` — the spin: rot += vel.X*0.02 then ±|vel.Y|*0.02
    /// by the X sign; the 0.98 drag runs after.
    #[test]
    fn w4_d11_106_the_spin_reads_the_velocity_sign() {
        let mut p = proj(569);
        p.vel = Vec2::new(2.0, -3.0);
        ai_spore_gas(&mut p, 0);
        // +X: rot = 2*0.02 + 3*0.02 = 0.1.
        assert!((p.rot - 0.1).abs() < 1e-6);
        assert_eq!(p.vel, Vec2::new(2.0, -3.0) * 0.98, "the drag");
        let mut p = proj(569);
        p.vel = Vec2::new(-2.0, -3.0);
        ai_spore_gas(&mut p, 0);
        // -X: rot = -2*0.02 - 3*0.02 = -0.1.
        assert!((p.rot + 0.1).abs() < 1e-6);
    }

    /// `:33544-33551` — before 60 ticks the alpha fades -30/tick to the
    /// 80 floor (255 -> 225 -> 195 -> 165 -> 135 -> 105 -> 80 clamp).
    #[test]
    fn w4_d11_106_the_alpha_fades_to_eighty() {
        let mut p = proj(569);
        for _ in 0..5 {
            ai_spore_gas(&mut p, 0);
        }
        assert_eq!(p.alpha, 105, "255 - 5*30");
        ai_spore_gas(&mut p, 0);
        assert_eq!(p.alpha, 80, "the clamp (75 -> 80)");
        ai_spore_gas(&mut p, 0);
        assert_eq!(p.alpha, 80, "the floor holds");
    }

    /// `:33529-33542` — at 60 ticks the fade flips: +5/tick to 255, then
    /// the owner-gated kill (a foreign owner's cloud lingers at 255).
    #[test]
    fn w4_d11_106_the_old_cloud_fades_up_and_dies() {
        let mut p = proj(569);
        p.ai[0] = 59.0;
        p.alpha = 80;
        ai_spore_gas(&mut p, 0);
        assert_eq!(p.ai[0], 60.0);
        assert_eq!(p.alpha, 85, "the +5 leg");
        for _ in 0..34 {
            ai_spore_gas(&mut p, 0);
        }
        assert_eq!(p.alpha, 255, "85 + 34*5");
        assert!(p.active, "the fade leg does not kill");
        ai_spore_gas(&mut p, 0);
        assert!(!p.active, "alpha 255 + owner -> Kill");
        // A foreign owner's cloud never kills on the alpha gate.
        let mut p = proj(569);
        p.owner = 3;
        p.ai[0] = 60.0;
        p.alpha = 255;
        ai_spore_gas(&mut p, 0);
        assert!(p.active, ":33539's owner gate");
    }

    /// The 575/596 rows (`Projectile.cs:6219-6229`, `:6450-6460`) + the
    /// 4-frame sheets (`Main.cs:8550`/`:8491`).
    #[test]
    fn w4_d11_107_the_sphere_rows_carry_their_styles() {
        let d = proj_data(575).unwrap();
        assert_eq!((d.ai_style, d.w, d.h, d.penetrate, d.time_left), (107, 24.0, 24.0, -1, 420));
        assert_eq!(d.alpha, 255, ":6228");
        assert!(!d.tile_collide, ":6226");
        let d = proj_data(596).unwrap();
        assert_eq!((d.ai_style, d.w, d.h, d.penetrate, d.time_left), (107, 8.0, 8.0, 1, 180));
        assert_eq!(d.alpha, 255, ":6456");
        assert_eq!(crate::projectile::proj_frames(575), 4);
        assert_eq!(crate::projectile::proj_frames(596), 4);
    }

    /// `:33653-33664` — the homing blend: past the 40-px radius the
    /// velocity lerps (v*9 + dir*5)/10 toward the player; inside the
    /// radius it holds.
    #[test]
    fn w4_d11_107_the_sphere_homes_past_forty_pixels() {
        let w = world();
        let mut rng = census();
        let mut p = proj(575);
        p.pos = Vec2::new(1000.0, 1000.0);
        p.ai[0] = 0.0; // the local player's index
        p.vel = Vec2::ZERO;
        ai_nebula_sphere(&mut p, &w, Some((1200.0, 1000.0)), true, false, &mut rng);
        // dir = (1, 0); vel = (0*9 + (1,0)*5)/10 = (0.5, 0).
        assert!((p.vel - Vec2::new(0.5, 0.0)).length() < 1e-5);
        assert!(rng.ranges.is_empty(), "the 575 draws nothing");
        // Inside the radius: no blend.
        let mut p = proj(575);
        p.pos = Vec2::new(1000.0, 1000.0);
        p.ai[0] = 0.0;
        p.vel = Vec2::new(3.0, 0.0);
        ai_nebula_sphere(&mut p, &w, Some((1020.0, 1000.0)), true, false, &mut rng);
        assert_eq!(p.vel, Vec2::new(3.0, 0.0), "inside 40 px: no steer");
    }

    /// `:33666-33676` — the orphan cap: a dead/absent target caps
    /// timeLeft at 30 and latches ai[0] = -1 + netUpdate (once).
    #[test]
    fn w4_d11_107_the_orphan_caps_and_latches() {
        let w = world();
        let mut rng = census();
        let mut p = proj(575);
        p.time_left = 420;
        p.ai[0] = 0.0;
        ai_nebula_sphere(&mut p, &w, None, false, false, &mut rng);
        assert_eq!(p.time_left, 30, ":33670");
        assert_eq!(p.ai[0], -1.0, ":33674");
        assert!(p.net_update, ":33675");
        p.net_update = false;
        ai_nebula_sphere(&mut p, &w, None, false, false, &mut rng);
        assert!(!p.net_update, "the latch holds (ai[0] already -1)");
        assert_eq!(p.time_left, 30, "the cap does not re-raise");
    }

    /// `:33617-33630` — the 596's three Next(3) dust gates: the C#
    /// RETURNS on the first non-zero roll (the homing tail skips that
    /// tick). CensusRng returns min (0) so all three pass.
    #[test]
    fn w4_d11_107_the_curse_dust_gates_census() {
        let w = world();
        let mut rng = census();
        let mut p = proj(596);
        p.ai[0] = 0.0;
        ai_nebula_sphere(&mut p, &w, Some((1200.0, 1000.0)), true, false, &mut rng);
        assert_eq!(rng.ranges, vec![(0, 3), (0, 3), (0, 3)], "the three gates");
        assert_eq!(p.ai[1], 1.0, ":33615");
        // A non-zero first gate returns early: no homing, no later gates.
        struct FirstNonZero;
        impl DropRng for FirstNonZero {
            fn next_range(&mut self, _min: i32, _max: i32) -> i32 {
                1
            }
            fn next_float(&mut self) -> f32 {
                0.0
            }
        }
        let mut p = proj(596);
        p.pos = Vec2::new(1000.0, 1000.0);
        p.ai[0] = 0.0;
        p.vel = Vec2::ZERO;
        ai_nebula_sphere(&mut p, &w, Some((1200.0, 1000.0)), true, false, &mut FirstNonZero);
        assert_eq!(p.vel, Vec2::ZERO, "the early return skips the homing");
    }

    /// `:33631-33651` — the final flash at timeLeft < 4: the box grows
    /// to 60x60 around the centre, the damage re-arms (30 / expert 22),
    /// ten SelectRandom(27,6) = Next(2) picks stay on the stream.
    #[test]
    fn w4_d11_107_the_curse_flashes_at_the_end() {
        let w = world();
        let mut rng = census();
        let mut p = proj(596);
        p.pos = Vec2::new(1000.0, 1000.0);
        p.time_left = 3;
        p.ai[0] = -1.0; // orphan: no homing interference
        ai_nebula_sphere(&mut p, &w, None, false, false, &mut rng);
        assert_eq!((p.w, p.h), (60.0, 60.0), ":33639");
        // The centre is preserved: (1004, 1004) -> pos (974, 974).
        assert_eq!(p.pos, Vec2::new(974.0, 974.0));
        assert_eq!(p.damage, 30, ":33641 (non-expert)");
        // Three gates + ten SelectRandom picks.
        assert_eq!(rng.ranges.len(), 13);
        assert_eq!(rng.ranges[3..], [(0, 2); 10], "the ten Next(2) picks");
        let mut rng = census();
        let mut p = proj(596);
        p.time_left = 3;
        p.ai[0] = -1.0;
        ai_nebula_sphere(&mut p, &w, None, false, true, &mut rng);
        assert_eq!(p.damage, 22, ":33636 (expert)");
    }

    /// The 582 row (`Projectile.cs:6316-6324`).
    #[test]
    fn w4_d11_109_the_wrench_row_carries_its_style() {
        let d = proj_data(582).unwrap();
        assert_eq!((d.ai_style, d.w, d.h, d.penetrate), (109, 20.0, 20.0, -1));
        assert_eq!(d.extra_updates, 1, "MaxUpdates 2");
        assert!(d.tile_collide, "the stock true");
    }

    /// `:34028-34040` — the speed latch + the 30-tick outbound wait: the
    /// wrench flies straight, spinning pi/10/tick; at localAI[0] > 30 the
    /// return arms (ai[0] = 1) and the spin skips that tick.
    #[test]
    fn w4_d11_109_the_wrench_waits_then_arms_the_return() {
        let w = world();
        let mut p = proj(582);
        p.vel = Vec2::new(6.0, 0.0);
        ai_mechanic_wrench(&mut p, &w, None);
        assert_eq!(p.local_ai[1], 6.0, "the speed latch");
        assert_eq!(p.local_ai[0], 1.0);
        assert!((p.rot - std::f32::consts::PI / 10.0).abs() < 1e-6, "the spin");
        for _ in 0..29 {
            ai_mechanic_wrench(&mut p, &w, None);
        }
        assert_eq!(p.local_ai[0], 30.0);
        assert_eq!(p.ai[0], 0.0, "still outbound at 30");
        let rot_before = p.rot;
        ai_mechanic_wrench(&mut p, &w, None);
        assert_eq!(p.ai[0], 1.0, "localAI[0] > 30 arms the return");
        assert_eq!(p.local_ai[0], 0.0, "the counter reset");
        assert_eq!(p.rot, rot_before, ":34039's return skips the spin");
    }

    /// `:34045-34062` — the return leg: no live Mechanic (124) -> Kill;
    /// a live host drops tileCollide, lerps the velocity toward it at the
    /// latched speed (0.04), and kills inside the latch radius.
    #[test]
    fn w4_d11_109_the_return_homes_the_mechanic() {
        let w = world();
        let mut p = proj(582);
        p.ai[0] = 1.0;
        p.local_ai[1] = 6.0;
        ai_mechanic_wrench(&mut p, &w, None);
        assert!(!p.active, "no host -> Kill");
        let h = host_npc(crate::npc::NpcKind::Mechanic, 1200.0, 1000.0);
        let mut p = proj(582);
        p.pos = Vec2::new(1000.0, 1000.0);
        p.ai[0] = 1.0;
        p.local_ai[1] = 6.0;
        p.vel = Vec2::new(6.0, 0.0);
        ai_mechanic_wrench(&mut p, &w, Some(&h));
        assert!(p.active);
        assert!(!p.tile_collide, ":34052");
        // The lerp: vel = lerp((6,0), (6,0), 0.04) = (6,0) (already aimed).
        assert!((p.vel.x - 6.0).abs() < 1e-4);
        // Inside the latch radius -> Kill.
        let h = host_npc(crate::npc::NpcKind::Mechanic, 1002.0, 1000.0);
        let mut p = proj(582);
        p.pos = Vec2::new(1000.0, 1000.0);
        p.ai[0] = 1.0;
        p.local_ai[1] = 6.0;
        ai_mechanic_wrench(&mut p, &w, Some(&h));
        assert!(!p.active, "inside the latch radius -> Kill");
    }

    /// `HandleMovement` `:19563-19571` — the tile bounce reverses both
    /// axes, arms the return, sets netUpdate + the Dig thud.
    #[test]
    fn w4_d11_109_the_wrench_bounces_off_tiles() {
        let mut w = world();
        // A solid wall at tile (110, 50).
        let i = w.index(110, 50);
        w.tiles[i] = tr_core::tile::Tile { kind: 1, ..Default::default() };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut p = proj(582);
        p.pos = Vec2::new(109.0 * 16.0 - 20.0, 50.0 * 16.0);
        p.vel = Vec2::new(6.0, 0.0);
        ai_mechanic_wrench(&mut p, &w, None);
        assert_eq!(p.vel, Vec2::new(-6.0, 0.0), "the X reversal");
        assert_eq!(p.ai[0], 1.0, "the return arms");
        assert!(p.net_update, ":19570");
        assert!(p.wants_sound.is_some(), "the Dig thud");
    }

    /// The 584 row (`Projectile.cs:6340-6348`).
    #[test]
    fn w4_d11_110_the_syringe_row_carries_its_style() {
        let d = proj_data(584).unwrap();
        assert_eq!((d.ai_style, d.w, d.h, d.penetrate), (110, 10.0, 10.0, 3));
        assert_eq!(d.scale, 0.8, ":6346");
    }

    /// `:34073-34108` — a non-town / absent host kills the syringe; the
    /// speed latch holds the launch speed.
    #[test]
    fn w4_d11_110_the_syringe_dies_without_a_town_host() {
        let mut p = proj(584);
        p.vel = Vec2::new(5.0, 0.0);
        ai_nurse_syringe(&mut p, None);
        assert!(!p.active, "no host -> Kill");
        assert_eq!(p.local_ai[1], 5.0, "the speed latch ran first");
        let mut zombie = host_npc(crate::npc::NpcKind::Zombie, 1200.0, 1000.0);
        let mut p = proj(584);
        ai_nurse_syringe(&mut p, Some(&mut zombie));
        assert!(!p.active, "a non-town host -> Kill");
    }

    /// `:34078-34090` — the catch heals min(lifeMax - life, 20), positive
    /// only; a full-health host heals nothing.
    #[test]
    fn w4_d11_110_the_catch_heals_up_to_twenty() {
        let mut nurse = host_npc(crate::npc::NpcKind::Nurse, 1002.0, 1000.0);
        nurse.max_hp = 250;
        nurse.hp = 200; // 50 missing -> capped at 20
        let mut p = proj(584);
        p.pos = Vec2::new(1000.0, 1000.0);
        p.local_ai[1] = 6.0;
        ai_nurse_syringe(&mut p, Some(&mut nurse));
        assert!(!p.active, "the catch kills the syringe");
        assert_eq!(nurse.hp, 220, "min(50, 20) = 20");
        // A full-health host: no heal.
        let mut nurse = host_npc(crate::npc::NpcKind::Nurse, 1002.0, 1000.0);
        nurse.max_hp = 250;
        nurse.hp = 250;
        let mut p = proj(584);
        p.pos = Vec2::new(1000.0, 1000.0);
        p.local_ai[1] = 6.0;
        ai_nurse_syringe(&mut p, Some(&mut nurse));
        assert_eq!(nurse.hp, 250, "no missing life -> no heal");
        // A small wound heals exactly the wound.
        let mut nurse = host_npc(crate::npc::NpcKind::Nurse, 1002.0, 1000.0);
        nurse.max_hp = 250;
        nurse.hp = 243; // 7 missing
        let mut p = proj(584);
        p.pos = Vec2::new(1000.0, 1000.0);
        p.local_ai[1] = 6.0;
        ai_nurse_syringe(&mut p, Some(&mut nurse));
        assert_eq!(nurse.hp, 250, "min(7, 20) = 7");
    }

    /// `:34094-34102` — the seek: the Y floor (value.Y = max(value.Y,
    /// vel.Y)) then ++, the 0.04 lerp, the vel.X*0.05 spin.
    #[test]
    fn w4_d11_110_the_seek_floors_y_and_spins() {
        let mut nurse = host_npc(crate::npc::NpcKind::Nurse, 1200.0, 1000.0);
        let mut p = proj(584);
        p.pos = Vec2::new(1000.0, 1000.0);
        p.local_ai[1] = 6.0;
        p.vel = Vec2::new(6.0, 2.0); // a downward vel.Y floor
        ai_nurse_syringe(&mut p, Some(&mut nurse));
        assert!(p.active, "outside the latch radius");
        // dir = (1, 0) * 6 = (6, 0); Y floor: max(0, 2) = 2, then ++ = 3.
        // vel = lerp((6,2), (6,3), 0.04) = (6, 2.04).
        assert!((p.vel.x - 6.0).abs() < 1e-4);
        assert!((p.vel.y - 2.04).abs() < 1e-4, "the Y floor + ++ + lerp");
        assert!((p.rot - 6.0 * 0.05).abs() < 1e-5, "the spin");
    }
}
