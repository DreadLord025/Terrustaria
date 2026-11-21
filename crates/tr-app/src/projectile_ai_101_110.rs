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
}
