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
    let center = || Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
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
            let c = center();
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
                        let c = center();
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
        let c = center();
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
}
