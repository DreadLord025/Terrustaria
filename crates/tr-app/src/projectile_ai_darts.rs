//! S1-P-i16 — the dart guns' projectile AIs (`Player.ManageShoot`'s
//! `useAmmo == AmmoID.Dart` family in the task's terms; the C# method is
//! `Player.ItemCheck_Shoot`, `Player.cs:47056-49468` — the dart guns 3007
//! Dart Pistol / 3008 Dart Rifle resolve their ammo through `PickAmmo`'s
//! `item.shoot` arm `:53491-53494`, so the per-dart behaviour lives in the
//! projectiles, not the shoot path).
//!
//! Kind set (all rowed in `projectile/data.rs` by this task):
//! - 267 Poison Dart (`Projectile.cs:3529-3537`, aiStyle 1),
//! - 477 Crystal Dart (`:5518-5528`, aiStyle 1, penetrate 5, extraUpdates 1),
//! - 478 Cursed Dart (`:5529-5539`, aiStyle 1, penetrate 3, timeLeft 300),
//! - 479 Ichor Dart (`:5540-5548`, aiStyle 1),
//! - 442 Electrosphere Launcher bolt (`:5126-5136`, aiStyle 1, timeLeft 600),
//! - 443 Electrosphere sphere (`:5137-5149`, aiStyle 77),
//! - 167-170 the Fireworks Launcher's rockets (`:2558-2569`, aiStyle 34).
//! The Cursed Dart's flame drop 480 (`:5549-5560`, aiStyle 14) rides the
//! generic `ai_spear` arm (AI_014 has no 480 flight branch — only the dust
//! block `:25227-25245`, whose draws are censused here).
//!
//! C# sources read WHOLE (the golden rule):
//! - `AI_001`'s dart arm `Projectile.cs:57292-57364` (the shared fade /
//!   20-tick gravity pin / the Ichor split / the Cursed flame drops), the
//!   477 rotation tail `:57684-57695`, the `flag3` counter switch
//!   `:55945-56041` (none of 267/477/478/479/442 is on the skip list — the
//!   counter runs), the clamp `:57753-57768`.
//! - The 442 bolt's `AI_001` block `:54726-54771` (the alpha fade, the
//!   tile-target arrival kill, the 120-tick hover kill).
//! - The 477 crystal bounce `HandleMovement` `:18644-18697` (the penetrate
//!   spend, the reflection, the ×0.85 damage, the post-bounce retarget).
//! - `AI()`'s aiStyle-77 arm `:30671-30761` (the sphere pulse) and
//!   aiStyle-34 arm `:27136-27188` (the fireworks rotation).
//! - The on-hit debuffs `StatusNPC` `:11635-11645` (267), `:11482-11489`
//!   (478/480 → 39, 479 → 69) and the 477 retarget `:13534-13567`.
//! - The Kill arms: the 478 flame drop `:75578-75584`, the 442 sphere
//!   spawn `:73528-73552`, the aiStyle-34 fireworks burst `:76464-76674`.
//!
//! RNG contract (the `UpdateProjectiles` stream, `Main.cs:17775-17777`):
//! every EXPLICIT `Main.rand` draw stays in the C#'s order; the
//! `Dust.NewDust*` internal rolls go with the dust pool (the
//! `projectile_ai_misc` convention). Per kind:
//! - 267: ZERO draws in the AI arm (`:57292-57306` — the fade and the
//!   gravity pin roll nothing).
//! - 477: ZERO AI draws; the bounce retarget spends one `Next(count)`
//!   (`:18689`) and the on-hit retarget one `Next(count)` (`:13559`).
//! - 478: one `Next(5, 20)` per tick (`:57354`).
//! - 479: one `Next(5, 30)` per tick while `ai[1] >= 0` (`:57321`); on the
//!   split one `Next(2, 4)` + one `Next(4)` + per shard a
//!   `Next(-100, 101)` pair (`:57327-57334`).
//! - 442: ZERO AI draws (the fade/arrival/hover are deterministic); the
//!   Kill arm one `Next(3, 7)` dust-count (`:73531`).
//! - 443: the pulse's dust draws — `NextDouble` angle + `NextDouble`
//!   radius per dust (`:30742`), the `Next(2)` clone gate (`:30744`, a
//!   second angle+radius pair when taken), the `Next(4)` gate (`:30752`,
//!   a third pair when taken).
//! - 167-170: ZERO AI draws; the Kill burst's 169 arm spends one
//!   `NextDouble` rotation + `Next(5, 9)` + `Next(12, 17)` + `Next(3, 7)`
//!   (`:76548-76551`); the 170 arm one `Next(3)` per dust (`:76668`).
//! - 480: the AI_014 dust block `:25233` `Next(50)` + the `:25236`
//!   `Next(2)` gate (a second `Next(50)` when taken).
//!
//! Movement is the port's documented kill-on-tile stand-in
//! (`HandleMovement`'s default arm `:19843-19848`, the
//! `projectile_ai001_c` convention) — EXCEPT the 477 crystal bounce, whose
//! per-axis reflection is the C# arm's own (`:18644-18663`), and the 443
//! sphere (`tileCollide = false`, pure integration).
//!
//! CODE-ONLY MODE (S1-P-i16): the witness tests at the bottom are WRITTEN
//! NOT RUN (no compiler invoked on this branch); every expected value is
//! hand-derived from the cited C# lines.

use glam::Vec2;
use tr_core::World;
use tr_core::loot::DropRng;

use crate::npc::Npc;
use crate::projectile::{
    Projectile, can_hit_line, chaseable, safe_normalize,
};

/// The dart/bolt kinds this module owns (aiStyle 1 — the dispatch gates
/// them ahead of the generic `ai_arrow` flight, kind-disjoint from the
/// A/B/C slices and the rider/frostmoon/moonlord sets).
pub(crate) const DART_KINDS: [u16; 5] = [267, 477, 478, 479, 442];

/// Whether this module owns the kind's aiStyle-1 arm.
pub(crate) fn covers(kind: u16) -> bool {
    DART_KINDS.contains(&kind)
}

/// The C# `alpha -= n` with the `< 0 → 0` clamp (the slice-C helper's shape).
fn fade_alpha(p: &mut Projectile, n: i32) {
    p.alpha -= n;
    if p.alpha < 0 {
        p.alpha = 0;
    }
}

/// The port's `Kill()` stand-in for the AI early-kill paths (the slice-C
/// convention — the death hooks run on the pool's `!active` transition).
fn kill(p: &mut Projectile) {
    p.active = false;
    p.time_left = 0;
}

/// `AI_001`'s dart/bolt body for one slot, in the C#'s own block order:
/// the per-type hooks above the counter, the `flag3` counter (`:56038`),
/// the dart gravity arm (`:57292-57364`) or the 442 block (`:54726-54771`),
/// the rotation tail (`:57684-57695` / `:57750`), the clamp (`:57753`) and
/// the movement stand-in.
///
/// `local_owner` is the pool's `Main.myPlayer` (the Ichor/Cursed owner
/// gates `:57307`/`:57351`); the split/drop spawns ride
/// `p.wants_dart_spawns` (the pool resolves them after the sub-steps).
pub(crate) fn step(
    p: &mut Projectile,
    world: &World,
    npcs: &[Npc],
    rng: &mut dyn DropRng,
    local_owner: u8,
    immune: &[i32],
) {
    if p.kind == 442 {
        electrosphere_bolt(p, world);
        return;
    }
    // `:56038-56041` — `flag3` is true for every dart kind (none is on the
    // `:55947-56032` skip list): the counter runs.
    p.ai[0] += 1.0;
    // `:57292-57306` — the shared dart arm: the localAI[0] fade latch and
    // the 20-tick gravity pin (the Crystal Dart 477 never falls).
    p.local_ai[0] += 1.0;
    if p.local_ai[0] > 3.0 {
        p.alpha = 0;
    }
    if p.ai[0] >= 20.0 {
        p.ai[0] = 20.0;
        if p.kind != 477 {
            p.vel.y += 0.075;
        }
    }
    // `:57307-57350` — the Ichor Dart's split machine (owner-gated).
    if p.kind == 479 && p.owner == local_owner {
        ichor_dart(p, rng);
    }
    // `:57351-57363` — the Cursed Dart's flame drops (owner-gated).
    if p.kind == 478 && p.owner == local_owner {
        cursed_dart(p, rng);
    }
    // The rotation tail: the 477's five-tick blend (`:57684-57695`), the
    // others the default `Atan2(vy, vx) + 1.57f` (`:57750`).
    if p.kind == 477 {
        let angle = p.vel.y.atan2(p.vel.x) + 1.57;
        if p.local_ai[1] < 5.0 {
            p.rot = angle;
            p.local_ai[1] += 1.0;
        } else {
            p.rot = (p.rot * 2.0 + angle) / 3.0;
        }
    } else {
        p.rot = p.vel.y.atan2(p.vel.x) + 1.57;
    }
    // `:57753-57768` — the +16 clamp (flag8 true, flag4 false).
    if p.vel.y > 16.0 {
        p.vel.y = 16.0;
    }
    // `Projectile.cs:57769-57793` — the Harpy-charm tail (W4-f20 F7):
    // after the clamp, before the movement. The darts are not arrows
    // (the producer's `arrow` gate never charms them), so the tail is
    // the two-bit no-op; the slice keeps the C# shape.
    crate::projectile_harpy_charm::charm_tail(p, world, npcs, immune);
    // The movement: the 477's bounce arm (`HandleMovement`
    // `:18644-18697`) or the kill-on-tile stand-in.
    if p.kind == 477 {
        crystal_bounce(p, world, npcs, rng, local_owner);
    } else {
        move_kill_on_tile(p, world);
    }
}

/// The Ichor Dart's split machine (`Projectile.cs:57307-57350`): while
/// `ai[1] >= 0` the dart penetrates forever; the first negative tick
/// restores penetrate 1; the counter runs until it beats `Next(5, 30)`,
/// then the dart latches `ai[1] = -1000` and fans 2-4 child darts at 0.9×
/// damage, each `ai[1] = -1000` (the `IchorDartUpdatePenetrate` modifier,
/// `NewProjectileModifiers.cs:15-28`, folds to the same penetrate restore
/// the per-tick arm already runs).
fn ichor_dart(p: &mut Projectile, rng: &mut dyn DropRng) {
    // `:57309-57316` — the penetrate flip.
    if p.ai[1] >= 0.0 {
        p.max_penetrate = -1;
        p.penetrate = -1;
    } else if p.penetrate < 0 {
        p.max_penetrate = 1;
        p.penetrate = 1;
    }
    // `:57317-57320` — the counter only advances while non-negative.
    if p.ai[1] >= 0.0 {
        p.ai[1] += 1.0;
    }
    // `:57321` — the split threshold: ONE `Next(5, 30)` per tick (the C#
    // compares every tick, the -1000 latch included — the draw stays).
    if p.ai[1] > rng.next_range(5, 30) as f32 {
        p.ai[1] = -1000.0;
        let speed = p.vel.length();
        let dir = safe_normalize(p.vel);
        // `:57327-57331` — the shard count: `Next(2, 4)`, +1 on `Next(4) == 0`.
        let mut count = rng.next_range(2, 4);
        if rng.next_range(0, 4) == 0 {
            count += 1;
        }
        let cx = p.pos.x + p.w * 0.5;
        let cy = p.pos.y + p.h * 0.5;
        let damage = (p.damage as f32 * 0.9) as i32;
        for _ in 0..count {
            // `:57334-57338` — the shard direction: a random unit vector
            // + 2× the parent direction, renormalized to the parent speed.
            let mut v = Vec2::new(
                rng.next_range(-100, 101) as f32,
                rng.next_range(-100, 101) as f32,
            );
            v = safe_normalize(v);
            v += dir * 2.0;
            v = safe_normalize(v);
            v *= speed;
            // `:57339` — `NewProjectile(..., base.Center, ..., this.type,
            // (int)(damage * 0.9f), knockBack, owner, 0f, -1000f, ...)`:
            // the C# takes the CENTRE; the port's spawn takes the top-left
            // (the 14×14 dart's half-box subtracts). The dust trail
            // (`:57340-57347`) draws nothing explicit — skipped.
            p.wants_dart_spawns.push((
                479,
                cx - 7.0,
                cy - 7.0,
                v.x,
                v.y,
                damage,
                p.knock_back,
                0.0,
                -1000.0,
                0,
                0.0,
            ));
        }
    }
}

/// The Cursed Dart's flame drops (`Projectile.cs:57351-57363`): `ai[1]`
/// counts; past `Next(5, 20)` the life drops 20 (while > 40), the counter
/// resets and a 480 flame spawns at the centre with 0.8× damage / 0.5×
/// knockback.
fn cursed_dart(p: &mut Projectile, rng: &mut dyn DropRng) {
    p.ai[1] += 1.0;
    if p.ai[1] > rng.next_range(5, 20) as f32 {
        if p.time_left > 40 {
            p.time_left -= 20;
        }
        p.ai[1] = 0.0;
        let cx = p.pos.x + p.w * 0.5;
        let cy = p.pos.y + p.h * 0.5;
        // `:57361` — `NewProjectile(..., base.Center, 0f, 0f, 480,
        // (int)(damage * 0.8), knockBack * 0.5f, owner)` — the 12×12
        // flame's half-box subtracts.
        p.wants_dart_spawns.push((
            480,
            cx - 6.0,
            cy - 6.0,
            0.0,
            0.0,
            (p.damage as f64 * 0.8) as i32,
            p.knock_back * 0.5,
            0.0,
            0.0,
            0,
            0.0,
        ));
    }
}

/// The 442 Electrosphere bolt's `AI_001` block (`Projectile.cs:54726-54771`):
/// the alpha fade (the localAI[0] 4-tick cadence, the <0 → 0 + latch-2
/// snap), the tile-target arrival kill (the ai[0]/ai[1] tile coords the
/// shoot path seeds, `Player.cs:49388-49392`), the 120-tick hover kill.
/// The block sits ABOVE the counter in C# and `return`s on both kills —
/// the counter/rotation/clamp never run for this kind on a kill tick; on
/// the live ticks the generic tail runs with NO gravity (the `:57520`
/// chain guard excludes 442).
fn electrosphere_bolt(p: &mut Projectile, world: &World) {
    // `:54728` — the frame pin.
    p.frame = 0;
    // `:54729-54741` — the fade.
    if p.alpha != 0 {
        p.local_ai[0] += 1.0;
        if p.local_ai[0] >= 4.0 {
            p.alpha -= 90;
            if p.alpha < 0 {
                p.alpha = 0;
                p.local_ai[0] = 2.0;
            }
        }
    }
    // `:54742-54746` — the arrival: within 16 px of the target tile's
    // centre (`(ai[0], ai[1]) * 16 + (8, 8)`).
    let target = Vec2::new(p.ai[0], p.ai[1]) * 16.0 + Vec2::splat(8.0);
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    if center.distance(target) <= 16.0 {
        kill(p);
        return;
    }
    // `:54747-54770` — the hover: once visible, 120 ticks then die. The
    // light (`:54755`) and the 8-dust ring (`:54760-54768`) draw nothing
    // explicit — skipped.
    if p.alpha == 0 {
        p.local_ai[1] += 1.0;
        if p.local_ai[1] >= 120.0 {
            kill(p);
            return;
        }
        p.local_ai[0] += 1.0;
        if p.local_ai[0] == 3.0 {
            p.local_ai[0] = 0.0;
        }
    }
    // The generic tail: 442 IS on the `flag3 = false` skip list
    // (`:55996`) — the counter RESTS (ai[0] keeps the target tile X the
    // shoot path seeded), no gravity (the `:57520` guard excludes 442
    // too), the default rotation (`:57750`), the +16 clamp, the movement.
    p.rot = p.vel.y.atan2(p.vel.x) + 1.57;
    if p.vel.y > 16.0 {
        p.vel.y = 16.0;
    }
    move_kill_on_tile(p, world);
}

/// The 477 Crystal Dart's tile bounce (`HandleMovement`
/// `Projectile.cs:18644-18697`): a blocked axis spends one penetrate (the
/// kill at 0), reflects at −1, re-aims the rotation and decays the damage
/// ×0.85; then (penetrate > 0, owner-local) the retarget scan picks a
/// random chaseable NPC 20-700 px out with a clear line and steers at the
/// current speed.
fn crystal_bounce(
    p: &mut Projectile,
    world: &World,
    npcs: &[Npc],
    rng: &mut dyn DropRng,
    local_owner: u8,
) {
    let last = p.vel;
    // The port's per-axis destination probe (the `ai_style1_flight`
    // stand-in): a blocked axis reads as the C#'s `velocity != last`.
    let nx = p.pos.x + p.vel.x;
    let ny = p.pos.y + p.vel.y;
    let hit_x = crate::player::collides_rect(world, nx, p.pos.y, p.w, p.h);
    let hit_y = crate::player::collides_rect(world, p.pos.x, ny, p.w, p.h);
    if hit_x || hit_y {
        // `:18648-18652` — the penetrate spend + kill.
        p.penetrate -= 1;
        if p.penetrate <= 0 {
            kill(p);
            return;
        }
        // `:18653-18660` — the per-axis reflection.
        if hit_x {
            p.vel.x = -last.x;
        }
        if hit_y {
            p.vel.y = -last.y;
        }
        // `:18661-18662` — the rotation + the ×0.85 damage.
        p.rot = p.vel.y.atan2(p.vel.x) + std::f32::consts::FRAC_PI_2;
        p.damage = (p.damage as f32 * 0.85) as i32;
    } else {
        // S1-P-i57 — the unblocked move rides `:20441`'s wet swap
        // (`wet_velocity`: the `:17612-17655` drag while wet).
        p.pos += crate::projectile_wet::wet_velocity(p, world);
    }
    // `:18664-18697` — the post-bounce retarget (penetrate > 0, owner).
    if p.penetrate > 0 && p.owner == local_owner {
        retarget(p, npcs, world, rng, None);
    }
}

/// The 477's shared retarget scan (`Projectile.cs:18666-18696` /
/// `:13536-13566`): up to 9 chaseable NPCs strictly 20-700 px from the
/// centre with a clear `CanHitLine`, one `Next(count)` pick, the velocity
/// re-aimed at the pick at the current speed, the rotation + π/2.
/// `exclude` is the on-hit variant's victim index (`:13542`).
pub(crate) fn retarget(
    p: &mut Projectile,
    npcs: &[Npc],
    world: &World,
    rng: &mut dyn DropRng,
    exclude: Option<usize>,
) {
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    let mut candidates = [0usize; 10];
    let mut count = 0usize;
    for (idx, n) in npcs.iter().enumerate() {
        if Some(idx) == exclude || !chaseable(n) {
            continue;
        }
        let (nw, nh) = n.size();
        let nc = Vec2::new(n.x + nw * 0.5, n.y + nh * 0.5);
        let d = center.distance(nc);
        if d > 20.0 && d < 700.0 && can_hit_line(world, center.x, center.y, 1.0, 1.0, nc.x, nc.y, 1.0, 1.0) {
            candidates[count] = idx;
            count += 1;
            if count >= 9 {
                break;
            }
        }
    }
    if count > 0 {
        let pick = candidates[rng.next_range(0, count as i32) as usize];
        let (nw, nh) = npcs[pick].size();
        let nc = Vec2::new(npcs[pick].x + nw * 0.5, npcs[pick].y + nh * 0.5);
        let speed = p.vel.length();
        p.vel = safe_normalize(nc - center) * speed;
        p.rot = p.vel.y.atan2(p.vel.x) + std::f32::consts::FRAC_PI_2;
    }
}

/// The kill-on-tile movement stand-in (`HandleMovement`'s default arm
/// `Projectile.cs:19843-19848`, the `projectile_ai001_c::move_only` shape).
fn move_kill_on_tile(p: &mut Projectile, world: &World) {
    if p.tile_collide {
        let nx = p.pos.x + p.vel.x;
        let ny = p.pos.y + p.vel.y;
        let hit_x = crate::player::collides_rect(world, nx, p.pos.y, p.w, p.h);
        let hit_y = crate::player::collides_rect(world, p.pos.x, ny, p.w, p.h);
        if hit_x || hit_y {
            p.active = false;
            p.time_left = 0;
        } else {
            // S1-P-i57 — the `ai_style1_flight` swap (`:20441`'s wet arm via
            // `wet_velocity`; dry byte-identical).
            p.pos += crate::projectile_wet::wet_velocity(p, world);
        }
    } else {
        p.pos += p.vel;
    }
}

/// `AI()`'s aiStyle-77 arm (`Projectile.cs:30671-30761`) — the
/// Electrosphere sphere 443: the ai[1]-latched fade-out kill, the 120-tick
/// pulse cycle (the 80·(0.75..1.0) box breathing), the 3-tick frame
/// machine and the dust ring's draws. The tracked-sound volume decay
/// (`:30673-30686`) has no sound engine — skipped (no draws). S1-P-i57: the
/// integrate rides `wet_velocity` (the row's `tileCollide = false`, the
/// `:17476` gate keeps it un-scaled).
pub(crate) fn sphere_pulse(p: &mut Projectile, rng: &mut dyn DropRng, world: &World) {
    // `:30687-30711` — the fade: ai[1] == 1 (the 442's Kill pulled it in)
    // ramps alpha +51 to the 255 kill; otherwise the −50 fade-in.
    if p.ai[1] == 1.0 {
        p.friendly = false;
        if p.alpha < 255 {
            p.alpha += 51;
        }
        if p.alpha >= 255 {
            p.alpha = 255;
            kill(p);
            return;
        }
    } else {
        fade_alpha(p, 50);
    }
    // `:30712-30723` — the pulse: ai[0] counts to 120 then wraps; the box
    // breathes 80 × max(0.75 + sin·0.25, 0.8 − sin·0.2).
    let period = 30.0f32;
    p.ai[0] += 1.0;
    if p.ai[0] > period * 4.0 {
        p.ai[0] = 0.0;
    }
    let ring = -Vec2::Y.rotate(Vec2::from_angle(std::f32::consts::TAU * p.ai[0] / period));
    let breathe = (0.75 + ring.y * 0.25).max(0.8 - ring.y * 0.2);
    let cx = p.pos.x + p.w * 0.5;
    let cy = p.pos.y + p.h * 0.5;
    let size = (80.0 * breathe) as i32 as f32;
    p.w = size;
    p.h = size;
    p.pos = Vec2::new(cx - size * 0.5, cy - size * 0.5);
    // `:30724-30733` — the 3-tick frame machine (4 frames).
    p.frame_counter += 1;
    if p.frame_counter >= 3 {
        p.frame_counter = 0;
        p.frame += 1;
        if p.frame >= 4 {
            p.frame = 0;
        }
    }
    // `:30734-30760` — the dust ring's draws: one angle + one radius
    // (`NextDouble` pair, `:30742`), the `Next(2)` clone gate (a second
    // pair, `:30749`), the `Next(4)` gate (a third pair, `:30757`).
    let _ = rng.next_float();
    let _ = rng.next_float();
    if rng.next_range(0, 2) == 0 {
        let _ = rng.next_float();
        let _ = rng.next_float();
    }
    if rng.next_range(0, 4) == 0 {
        let _ = rng.next_float();
        let _ = rng.next_float();
    }
    // The movement: tileCollide false — pure integration. S1-P-i57:
    // `wet_velocity` (the `:17476` free-move gate keeps it un-scaled).
    p.pos += crate::projectile_wet::wet_velocity(p, world);
}

/// `AI()`'s aiStyle-34 arm (`Projectile.cs:27136-27188`) — the fireworks
/// rockets 167-170: the velocity-pointing rotation; the ai[1] == 1 launch
/// counter (the wire's `ai1: 1f`, `Player.cs:49028`) counts and its dust
/// gates draw nothing explicit; the plain wiring rockets (ai[1] == 0)
/// take the else arm's single dust (no draws). ZERO `Main.rand` draws.
pub(crate) fn firework_flight(p: &mut Projectile) {
    p.rot = p.vel.y.atan2(p.vel.x) + std::f32::consts::FRAC_PI_2;
    if p.ai[1] == 1.0 {
        // `:27139-27169` — the launch counter (the dust at ai[0] == 1 and
        // ai[0] > 2 draws nothing explicit).
        p.ai[0] += 1.0;
    } else if matches!(p.kind, 415..=418) {
        // `:27170-27180` — the fireworks-box variants (unrowed — kept for
        // the arm's shape).
        p.ai[0] += 1.0;
    }
    // `:27181-27187` — the else arm's single dust: no draws.
    // The movement: the kill-on-tile stand-in runs at the dispatch.
}

/// The 480 flame drop's AI_014 dust block (`Projectile.cs:25227-25245`)
/// as the draw census: the `alpha = 255` pin, the first dust's `Next(50)`
/// scale and the `Next(2)` gate (a second `Next(50)` when taken). The
/// generic AI_014 flight runs at the dispatch's `ai_spear`.
pub(crate) fn flame_drop_dust(p: &mut Projectile, rng: &mut dyn DropRng) {
    p.alpha = 255;
    let _ = rng.next_range(0, 50);
    if rng.next_range(0, 2) == 0 {
        let _ = rng.next_range(0, 50);
    }
}

// =====================================================================
// Witness tests — WRITTEN, NOT RUN (code-only mode: no cargo/rustc).
// Every expected value is hand-derived from the cited C# lines; the RNG
// order is asserted against a recording stream.
// =====================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::{proj_data, ProjectilePool, FlailInputs, YoyoInputs};
    use tr_core::{World, TILE_PX};

    /// A recording stream: every draw lands in `draws` in order.
    struct Recorder {
        draws: Vec<(i32, i32)>,
        floats: usize,
        next: i32,
    }
    impl Recorder {
        fn new(next: i32) -> Self {
            Self { draws: Vec::new(), floats: 0, next }
        }
    }
    impl DropRng for Recorder {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.draws.push((min, max));
            self.next
        }
        fn next_float(&mut self) -> f32 {
            self.floats += 1;
            0.5
        }
    }

    fn world() -> World {
        World::new("t".into(), 512, 64)
    }

    /// A slot carrying its `SetDefaults` row slice (the slice-C fixture
    /// shape — field-by-field on `Default`).
    fn proj(kind: u16, vx: f32, vy: f32) -> Projectile {
        let d = proj_data(kind).expect("the rowed kind");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(10.0 * TILE_PX, 10.0 * TILE_PX);
        p.vel = Vec2::new(vx, vy);
        p.w = d.w;
        p.h = d.h;
        p.damage = 40;
        p.knock_back = 3.0;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.max_penetrate = d.penetrate;
        p.alpha = d.alpha;
        p.extra_updates = d.extra_updates;
        p.tile_collide = d.tile_collide;
        p
    }

    /// The rows match `SetDefaults` verbatim (the census's data half).
    #[test]
    fn s1_p_i16_the_dart_rows_match_set_defaults() {
        // 267 (`Projectile.cs:3529-3537`): 14×14, aiStyle 1, alpha 255,
        // the stock penetrate 1 / timeLeft 3600.
        let d = proj_data(267).unwrap();
        assert_eq!((d.w, d.h), (14.0, 14.0));
        assert_eq!(d.ai_style, 1);
        assert_eq!(d.alpha, 255);
        assert_eq!(d.penetrate, 1);
        assert_eq!(d.time_left, 3600);
        // 477 (`:5518-5528`): penetrate 5, extraUpdates 1.
        let d = proj_data(477).unwrap();
        assert_eq!(d.penetrate, 5);
        assert_eq!(d.extra_updates, 1);
        assert_eq!(d.alpha, 255);
        // 478 (`:5529-5539`): penetrate 3, timeLeft 300.
        let d = proj_data(478).unwrap();
        assert_eq!(d.penetrate, 3);
        assert_eq!(d.time_left, 300);
        // 479 (`:5540-5548`): the stock penetrate 1.
        let d = proj_data(479).unwrap();
        assert_eq!(d.penetrate, 1);
        assert_eq!(d.alpha, 255);
        // 480 (`:5549-5560`): 12×12, aiStyle 14, penetrate 1, timeLeft 180.
        let d = proj_data(480).unwrap();
        assert_eq!((d.w, d.h), (12.0, 12.0));
        assert_eq!(d.ai_style, 14);
        assert_eq!(d.penetrate, 1);
        assert_eq!(d.time_left, 180);
        // 442 (`:5126-5136`): 14×14, aiStyle 1, timeLeft 600, alpha 255.
        let d = proj_data(442).unwrap();
        assert_eq!((d.w, d.h), (14.0, 14.0));
        assert_eq!(d.ai_style, 1);
        assert_eq!(d.time_left, 600);
        assert_eq!(d.alpha, 255);
        // 443 (`:5137-5149`): 80×80, aiStyle 77, penetrate −1,
        // tileCollide false, the 300-tick NewProjectileSetup seed.
        let d = proj_data(443).unwrap();
        assert_eq!((d.w, d.h), (80.0, 80.0));
        assert_eq!(d.ai_style, 77);
        assert_eq!(d.penetrate, -1);
        assert!(!d.tile_collide);
        assert_eq!(d.time_left, 300);
        // 167-170 (`:2558-2569`): 14×14, aiStyle 34, timeLeft 45, the
        // idStatic-1 pair.
        for k in 167..=170u16 {
            let d = proj_data(k).unwrap();
            assert_eq!((d.w, d.h), (14.0, 14.0));
            assert_eq!(d.ai_style, 34);
            assert_eq!(d.time_left, 45);
            assert!(d.uses_id_static_immunity);
            assert_eq!(d.id_static_hit_cooldown, 1);
        }
        // The cluster shrapnel rows (779/783/862/863): 14×14, aiStyle 16,
        // the 60-tick NewProjectileSetup seed (`:11053-11064`).
        for k in [779u16, 783, 862, 863] {
            let d = proj_data(k).unwrap();
            assert_eq!(d.ai_style, 16);
            assert_eq!(d.time_left, 60);
            assert_eq!(d.penetrate, -1);
        }
    }

    /// The shared dart arm (`:57292-57306`): the localAI[0] fade latch
    /// clears alpha at tick 4; the gravity pin waits ai[0] >= 20 and the
    /// Crystal Dart never falls.
    #[test]
    fn s1_p_i16_the_dart_fade_and_gravity_pin() {
        let w = world();
        let mut rng = Recorder::new(100); // no split/drop thresholds beat
        // 267: four ticks fade, the 20th adds 0.075 gravity.
        let mut p = proj(267, 6.0, 0.0);
        for _ in 0..3 {
            step(&mut p, &w, &[], &mut rng, 0, &[]);
        }
        assert_eq!(p.alpha, 255, "the fade waits localAI[0] > 3");
        step(&mut p, &w, &[], &mut rng, 0, &[]);
        assert_eq!(p.alpha, 0, ":57295-57298 — the fourth tick clears");
        assert_eq!(p.vel.y, 0.0, "no gravity before ai[0] >= 20");
        for _ in 0..16 {
            step(&mut p, &w, &[], &mut rng, 0, &[]);
        }
        // ai[0] is 20 now: the pin fires and the gravity adds 0.075.
        assert_eq!(p.ai[0], 20.0, ":57301 — the pin");
        assert!((p.vel.y - 0.075).abs() < 1e-6, ":57304 — the 0.075 gravity");
        // 477: the same 20 ticks, NO gravity (`:57302`).
        let mut c = proj(477, 6.0, 0.0);
        for _ in 0..20 {
            step(&mut c, &w, &[], &mut rng, 0, &[]);
        }
        assert_eq!(c.vel.y, 0.0, ":57302 — the crystal dart never falls");
        assert!(rng.draws.is_empty(), "267/477 spend no AI draws");
    }

    /// The Ichor split (`:57307-57350`): the penetrate flip, the counter,
    /// the Next(5,30) threshold, the Next(2,4)+Next(4) count and the
    /// per-shard direction pair — in that order.
    #[test]
    fn s1_p_i16_the_ichor_split_draw_order_and_fan() {
        let w = world();
        let mut p = proj(479, 6.0, 0.0);
        p.ai[1] = 5.0; // one more tick beats a Next(5,30) returning 5
        struct Seq(Vec<i32>, usize);
        impl DropRng for Seq {
            fn next_range(&mut self, _min: i32, _max: i32) -> i32 {
                let v = self.0[self.1];
                self.1 += 1;
                v
            }
            fn next_float(&mut self) -> f32 { 0.5 }
        }
        // threshold 5, count base 2, no +1 (Next(4) = 1), then two
        // direction pairs (0, 100) and (100, 0).
        let mut seq = Seq(vec![5, 2, 1, 0, 100, 100, 0], 0);
        step(&mut p, &w, &[], &mut seq, 0, &[]);
        assert_eq!(p.ai[1], -1000.0, ":57323 — the split latch");
        assert_eq!(p.penetrate, -1, ":57311 — the pre-split penetrate");
        assert_eq!(p.wants_dart_spawns.len(), 2, "Next(2,4) = 2, no bonus");
        // The first shard: dir (0,100) → (0,1) + 2×(1,0) = (2,1),
        // normalized × 6.
        let s0 = p.wants_dart_spawns[0];
        assert_eq!(s0.0, 479);
        assert_eq!(s0.5, 36, "(int)(40 * 0.9) = 36");
        assert_eq!(s0.8, -1000.0, "the child ai[1] latch");
        let expect0 = Vec2::new(2.0, 1.0).normalize() * 6.0;
        assert!((s0.3 - expect0.x).abs() < 1e-4 && (s0.4 - expect0.y).abs() < 1e-4);
        // The second tick: ai[1] = -1000 < 0 → penetrate restores to 1
        // (`:57313-57316`), the threshold draw still spends (the C#
        // compares every tick).
        let mut seq2 = Seq(vec![5], 0);
        step(&mut p, &w, &[], &mut seq2, 0, &[]);
        assert_eq!(p.penetrate, 1, ":57315 — the restore");
        assert_eq!(p.max_penetrate, 1);
        assert_eq!(seq2.1, 1, "the threshold draw stays on the stream");
    }

    /// The Cursed flame drops (`:57351-57363`): the Next(5,20) threshold,
    /// the −20 life, the counter reset and the 480 spawn at 0.8×/0.5×.
    #[test]
    fn s1_p_i16_the_cursed_dart_drops_flames() {
        let w = world();
        let mut p = proj(478, 6.0, 0.0);
        p.ai[1] = 5.0;
        p.time_left = 300;
        struct Seq(Vec<i32>, usize);
        impl DropRng for Seq {
            fn next_range(&mut self, _min: i32, _max: i32) -> i32 {
                let v = self.0[self.1];
                self.1 += 1;
                v
            }
            fn next_float(&mut self) -> f32 { 0.5 }
        }
        let mut seq = Seq(vec![5], 0);
        step(&mut p, &w, &[], &mut seq, 0, &[]);
        assert_eq!(p.ai[1], 0.0, ":57360 — the reset");
        // The step itself does not decrement timeLeft (the pool does);
        // the drop took 20 off 300 → 280 (`:57356-57359`).
        assert_eq!(p.time_left, 280);
        assert_eq!(p.wants_dart_spawns.len(), 1);
        let s = p.wants_dart_spawns[0];
        assert_eq!(s.0, 480);
        assert_eq!(s.5, 32, "(int)(40 * 0.8) = 32");
        assert_eq!(s.6, 1.5, "knockBack * 0.5");
        assert_eq!((s.3, s.4), (0.0, 0.0), "the flame spawns at rest");
    }

    /// The 442 bolt (`:54726-54771`): the fade cadence, the arrival kill
    /// and the 120-tick hover kill.
    #[test]
    fn s1_p_i16_the_electrosphere_bolt_fades_arrives_and_hovers() {
        let w = world();
        let mut p = proj(442, 6.0, 0.0);
        p.ai[0] = 100.0; // the target tile x
        p.ai[1] = 10.0; // the target tile y
        let mut rng = Recorder::new(0);
        // Four ticks: alpha 255 → 165 at the fourth (localAI[0] hits 4).
        for _ in 0..4 {
            step(&mut p, &w, &[], &mut rng, 0, &[]);
        }
        assert_eq!(p.alpha, 165, ":54734 — the 90 fade at localAI[0] >= 4");
        // The arrival: place the bolt at the target centre.
        p.pos = Vec2::new(100.0 * 16.0 + 8.0 - 7.0, 10.0 * 16.0 + 8.0 - 7.0);
        step(&mut p, &w, &[], &mut rng, 0, &[]);
        assert!(!p.active, ":54742-54745 — the arrival kill");
        // The hover: a visible bolt (alpha 0) dies at localAI[1] 120.
        let mut h = proj(442, 0.0, 0.0);
        h.alpha = 0;
        h.ai[0] = 1000.0; // far target — no arrival
        h.ai[1] = 1000.0;
        h.local_ai[1] = 119.0;
        step(&mut h, &w, &[], &mut rng, 0, &[]);
        assert!(!h.active, ":57350-57353 — the 120-tick hover kill");
        assert!(rng.draws.is_empty(), "the bolt spends no draws");
    }

    /// The 477 bounce (`:18644-18663`): the penetrate spend, the
    /// reflection and the ×0.85 damage.
    #[test]
    fn s1_p_i16_the_crystal_dart_bounces_and_decays() {
        // A wall one tile to the right of the dart.
        let mut w = World::new("t".into(), 64, 64);
        let i = w.index(12, 10);
        w.tiles[i] = tr_core::Tile { kind: 1, ..Default::default() };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let mut p = proj(477, 6.0, 0.0);
        p.pos = Vec2::new(11.0 * TILE_PX + 2.0, 10.0 * TILE_PX);
        p.damage = 40;
        let mut rng = Recorder::new(0);
        step(&mut p, &w, &[], &mut rng, 0, &[]);
        assert_eq!(p.penetrate, 4, ":18648 — one bounce spends one");
        assert_eq!(p.vel.x, -6.0, ":18655 — the reflection");
        assert_eq!(p.damage, 34, "(int)(40 * 0.85) = 34 (:18662)");
        assert!(p.active, "penetrate 4 > 0 — alive");
    }

    /// The sphere pulse (`:30671-30761`): the fade-in, the box breathing
    /// and the dust draws' order.
    #[test]
    fn s1_p_i16_the_sphere_pulse_breathes_and_fades() {
        let mut p = proj(443, 0.0, 0.0);
        p.alpha = 255;
        let w = world();
        let mut rng = Recorder::new(1); // the Next(2)/Next(4) gates miss
        sphere_pulse(&mut p, &mut rng, &w);
        assert_eq!(p.alpha, 205, ":30705 — the −50 fade-in");
        assert_eq!(p.ai[0], 1.0, ":30714 — the counter");
        // The dust draws: angle+radius, then the two gates (no clones).
        assert_eq!(rng.floats, 2, "one angle + one radius");
        assert_eq!(rng.draws, vec![(0, 2), (0, 4)], "the clone gates");
        // The pull-in latch: ai[1] == 1 ramps +51 to the 255 kill.
        let mut q = proj(443, 0.0, 0.0);
        q.ai[1] = 1.0;
        q.alpha = 204;
        let mut rng2 = Recorder::new(0);
        sphere_pulse(&mut q, &mut rng2, &w);
        assert!(!q.active, ":30694-30698 — the 255 kill");
        assert_eq!(q.alpha, 255);
        assert!(!q.friendly, ":30689 — the pull-in flips friendly");
    }

    /// The pool resolves the dart spawn requests after the sub-steps (the
    /// wants_dart_spawns drain) — the Ichor fan reaches the pool.
    #[test]
    fn s1_p_i16_the_pool_resolves_the_dart_fan() {
        let mut pool = ProjectilePool::new();
        let mut w = world();
        let slot = pool
            .spawn_with_data_ai(479, 160.0, 160.0, 6.0, 0.0, 40, 3.0, 0.0, 5.0)
            .unwrap();
        struct Seq(Vec<i32>, usize);
        impl DropRng for Seq {
            fn next_range(&mut self, _min: i32, _max: i32) -> i32 {
                let v = self.0[self.1.min(self.0.len() - 1)];
                self.1 += 1;
                v
            }
            fn next_float(&mut self) -> f32 { 0.5 }
        }
        let mut seq = Seq(vec![5, 2, 1, 0, 100, 100, 0], 0);
        pool.update(
            &mut w,
            &mut [],
            None,
            None,
            None,
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut seq,
            0,
            false,
            &mut tr_core::UnifiedRandom::new(0),
        );
        let children: Vec<_> = pool
            .slots()
            .iter()
            .enumerate()
            .filter(|(i, p)| p.active && p.kind == 479 && *i != slot)
            .collect();
        assert_eq!(children.len(), 2, "the fan's two shards spawned");
        for (_, c) in children {
            assert_eq!(c.damage, 36, "the 0.9× damage");
            assert_eq!(c.ai[1], -1000.0, "the child latch");
            // The child's FIRST tick already ran (the drain sits ahead of
            // the next slot's update): ai[1] < 0 restored penetrate 1
            // (`:57313-57316`, the IchorDartUpdatePenetrate fold).
            assert_eq!(c.penetrate, 1, "the child's penetrate restore");
        }
    }
}
