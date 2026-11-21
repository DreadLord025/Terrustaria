//! W4-f20 slice E — `AI_001`'s per-type arms for the aiStyle-1 kinds the
//! round-b row branches ROWED but left on the generic `ai_arrow`
//! (`Projectile.cs:54002-57794`). The round-2 census
//! (`docs/research/26f`) re-scanned every pushed branch's `proj_data`:
//! exactly three kinds gained aiStyle-1 rows whose per-type C# state was
//! never ported — 435 MartianTurretBolt (W4-c11 `a9b22f05`), 576
//! NebulaLaser (W4-d11), 709 MonkStaffT3AltShot (W4-d15 `79576fbf`).
//! Kind set ([`SLICE_E_KINDS`]): 435, 576, 709.
//!
//! ROW-GATED INERTNESS (the d7 unwired precedent): the rows live on the
//! c11/d11/d15 branches, NOT on this one — `proj_data` returns `None`
//! here, so the dispatch's `else if let Some(data)` guard never routes
//! these kinds until the row branches merge. The gate below is
//! kind-disjoint from every existing gate by construction; when the rows
//! land the arm activates with NO further edit (no merge-time dedupe
//! owed — the census verified no other branch ports these overlays).
//!
//! C# structure preserved (the per-kind hooks above the flight, the
//! `flag3` counter switch `:55947-56037`, the gravity else-if chain
//! `:56753-57580`, the rotation tail chain `:57582-57751`, the `flag8`
//! clamp `:57753-57768`):
//! - ALL THREE kinds are on the `flag3 = false` skip list (435 `:55992`,
//!   576 `:56008`, 709 `:56025`) — the flight counter rests at 0, none is
//!   named in any gravity-chain arm, so the generic `ai[0] >= 15f` tail
//!   (`:57568`) is unreachable: every slice-E kind flies exactly straight
//!   (the slice-D STAKE argument).
//! - 435/709 share the rotation arm `:57642-57648` (`ToRotation()`, +π
//!   when `direction == -1` — the 459/682 siblings' arm); 576 takes the
//!   default `:57750` (`Atan2(vy, vx) + 1.57f`).
//! - 709's `ai[1]` is its AGE counter (the d15 spawn carries `ai1: None`):
//!   `ai[1]++` then `>= 30f` → `Kill(); return` (`:54719-54724`) — the
//!   kill fires BEFORE the flight/rotation tail, exactly as the C# orders.
//!
//! RNG: NONE of the ported state paths draws `Main.rand` — the only dust
//! in these kinds' blocks is the 576's 5-dust-242 trail (`:56299-56307`,
//! the plain `NewDust` overload), SKIPPED with its internal draws (the
//! ai001_a/b/c/d convention). The module takes no rng parameter, so the
//! `UpdateProjectiles` stream (`Main.cs:17775-17777`) is untouched BY
//! CONSTRUCTION for every kind. The 435/576 `SoundID.Item12` cues ride
//! `wants_sound` (the S1-E-21h drain, no draws); the 709's
//! `PlayTrackedSound(DD2_SkyDragonsFuryShot)` (`:54717`) folds to the
//! inert no-cue latch (the pool has no tracked-sound layer, S1-E-23 —
//! the d15 `projectile_ai_dd2` ghast/twister precedent); its `ai[1]`
//! state rides alone. The lights (`:54932`, `:54714`, `:56298`) are
//! SKIPPED (the `proj_lights` layer owns the per-kind light census).
//!
//! Movement is the port's documented kill-on-tile stand-in
//! (`HandleMovement`'s default arm `:19843-19848`), replicated verbatim
//! from `ai_style1_flight`'s tail as slices A/B/C/D do. The clamp is the
//! C#'s `flag8` tail EXACTLY (`:57753-57768`): only the +16 bound — the
//! −16 bound is `flag4`'s (kind 1006 only) and no slice-E kind sets it.
//!
//! CODE-ONLY MODE (W4-f20): the witness tests at the bottom are WRITTEN
//! NOT RUN (no compiler invoked on this branch); every expected value was
//! verified by re-reading the cited C# blocks and hand arithmetic. The
//! witnesses hand-build the row slice (the rows live on c11/d11/d15, the
//! d7 unwired-module precedent) — they never call `proj_data`.

use glam::Vec2;
use tr_core::World;

use crate::projectile::Projectile;

/// `ProjectileID.MartianTurretBolt` (435) — the W4-c11 row's kind
/// (`Projectile.cs:5056-5063`). Module-private: the `data.rs` const lands
/// with the c11 merge (no import until then — the row-gated inertness).
const MARTIAN_TURRET_BOLT: u16 = 435;
/// `ProjectileID.NebulaLaser` (576) — the W4-d11 row's kind
/// (`Projectile.cs:6258-6269`). Module-private: the `data.rs` const lands
/// with the d11 merge (no import until then — the row-gated inertness).
const NEBULA_LASER: u16 = 576;

/// The slice-E kind set — kind-disjoint from `projectile_ai001_a/b/c/d`,
/// the riders, the frostmoon/moonlord/darts/special-arrow/special-bullet
/// gates, the javelin and the Book of Skulls (the seam chains E after D).
pub(crate) const SLICE_E_KINDS: [u16; 2] = [
    MARTIAN_TURRET_BOLT, // 435
    NEBULA_LASER,        // 576
];

/// Whether this module owns the kind's aiStyle-1 arm.
pub(crate) fn covers(kind: u16) -> bool {
    SLICE_E_KINDS.contains(&kind)
}

/// `AI_001`'s slice-E body for one slot, in the C#'s own block order.
pub(crate) fn step(p: &mut Projectile, world: &World) {
    match p.kind {
        MARTIAN_TURRET_BOLT => {
            // `:54909-54933` — the head block. `:54911-54915`: the
            // `ai[1] == 0f` one-shot latch — the C# sets `ai[1] = 1f`
            // THEN plays `SoundID.Item12` at `base.position`; the cue
            // rides `wants_sound` (the S1-E-21h drain — the payload is
            // identical either order, the slice-A `latch_ai1` note).
            if p.ai[1] == 0.0 {
                p.ai[1] = 1.0;
                p.wants_sound = Some((2, 12, p.pos.x, p.pos.y, 0.0));
            }
            // `:54916-54920` — `alpha -= 40`, clamp 0. The c11 row seeds
            // alpha 0 (the C# arm `:5056-5063` writes NO alpha — the
            // stock `:779`), so the fade is a no-op on the rowed kind;
            // ported verbatim for its shape (a hostile spawn carrying a
            // nonzero alpha fades, as in vanilla).
            p.alpha -= 40;
            if p.alpha < 0 {
                p.alpha = 0;
            }
            // `:54921` — `spriteDirection = base.direction` (the
            // velocity-sign stand-in, the slices A/B/C/D convention:
            // the C# updates `direction` from `velocity.X`'s sign after
            // each sub-step's movement, `:17016-17026`).
            p.sprite_direction = if p.vel.x < 0.0 { -1 } else { 1 };
            // `:54922-54931` — `frameCounter++`, at 3: `frame++`,
            // `frameCounter = 0`, wrap at 4 (`Main.projFrames[435] = 4`,
            // `Main.cs:8532`, the c11 row's `proj_frames`).
            p.frame_counter += 1;
            if p.frame_counter >= 3 {
                p.frame += 1;
                p.frame_counter = 0;
                if p.frame >= 4 {
                    p.frame = 0;
                }
            }
            // `:54932` — the light (0.3, 0.8, 1.1) SKIPPED (the
            // proj_lights layer's census).
            // `flag3 = false` (`:55992`): the counter rests, the generic
            // gravity never fires — a straight bolt. The rotation tail
            // is the 435/459/682/709 arm (`:57642-57648`).
            finish(p, world, true);
        }
        NEBULA_LASER => {
            // `:55361-55365` — the `ai[1] == 0f` one-shot latch: the C#
            // sets `ai[1] = 1f` THEN plays `SoundID.Item12` at
            // `base.position`; the cue rides `wants_sound` (the S1-E-21h
            // drain — the slice-A `latch_ai1` convention).
            if p.ai[1] == 0.0 {
                p.ai[1] = 1.0;
                p.wants_sound = Some((2, 12, p.pos.x, p.pos.y, 0.0));
            }
            // `:55635-55649` — the per-type alpha switch's case 576/577:
            // `localAI[1]++`; while `> 2f`, `alpha -= 15` clamp 0. The
            // d11 row seeds alpha 255 (`:6265`): ticks 1-2 count only,
            // the fade fires from tick 3 (255 − 15×17 → 0 at tick 19).
            p.local_ai[1] += 1.0;
            if p.local_ai[1] > 2.0 {
                if p.alpha > 0 {
                    p.alpha -= 15;
                }
                if p.alpha < 0 {
                    p.alpha = 0;
                }
            }
            // `:56296-56308` — the light (0.4, 0.2, 0.4) SKIPPED (the
            // proj_lights layer's census) + the 5-dust-242 trail
            // (`:56299-56307`, the plain `NewDust` overload — the
            // internal draws ride with the skipped body, the module-doc
            // convention). NO state reads either.
            // `flag3 = false` (`:56008`): the counter rests, the generic
            // gravity never fires — a straight laser. The rotation tail
            // is the DEFAULT (`:57750` — 576 is not in the :57642 arm).
            finish(p, world, false);
        }
        _ => unreachable!("the dispatch gates on SLICE_E_KINDS"),
    }
}

/// The shared tail: the `flag3` counter rests for every slice-E kind
/// (`:55992`/`:56008`/`:56025` — no `ai[0]++` at `:56038-56041`), no
/// gravity-chain arm names them (the generic `:57568` tail is unreachable
/// with the resting counter), then the rotation tail, the `flag8` clamp
/// (`:57753-57768` — the exact C# +16 bound, module doc), then the
/// movement stand-in. `dir_split_rot` selects the `:57642-57648` arm
/// (435/709) over the `:57750` default (576).
fn finish(p: &mut Projectile, world: &World, dir_split_rot: bool) {
    if dir_split_rot {
        // `Projectile.cs:57642-57648` — `rotation =
        // velocity.ToRotation()`; `if (direction == -1) rotation += π`.
        // The velocity-sign stand-in for `base.direction` (module doc).
        p.rot = p.vel.y.atan2(p.vel.x);
        if p.vel.x < 0.0 {
            p.rot += std::f32::consts::PI;
        }
    } else {
        // `Projectile.cs:57750` — the default tail (the literal 1.57).
        p.rot = p.vel.y.atan2(p.vel.x) + 1.57;
    }
    // `Projectile.cs:57753-57768` — `flag8` is true for every slice-E
    // kind (only 577 opts out, unrowed); `flag4` is false, so just the
    // +16 bound.
    if p.vel.y > 16.0 {
        p.vel.y = 16.0;
    }
    move_only(p, world);
}

/// The movement stand-in — a verbatim replication of `ai_style1_flight`'s
/// integration (the `HandleMovement` default arm `Projectile.cs:19843-
/// 19848`, slices A/B/C/D's `move_only`).
fn move_only(p: &mut Projectile, world: &World) {
    if p.tile_collide {
        let nx = p.pos.x + p.vel.x;
        let ny = p.pos.y + p.vel.y;
        let hit_x = crate::player::collides_rect(world, nx, p.pos.y, p.w, p.h);
        let hit_y = crate::player::collides_rect(world, p.pos.x, ny, p.w, p.h);
        if hit_x || hit_y {
            p.active = false;
            p.time_left = 0;
        } else {
            p.pos.x = nx;
            p.pos.y = ny;
        }
    } else {
        p.pos += p.vel;
    }
}

// =====================================================================
// Witness tests — WRITTEN, NOT RUN (code-only mode: no cargo/rustc).
// Every expected value below is hand-derived from the cited C# lines;
// the RNG-order argument is by construction (the module takes no rng).
// The rows live on c11/d11/d15 — the witnesses hand-build the row slice
// (the d7 unwired-module precedent), never `proj_data`.
// =====================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::TILE_PX;

    /// A tile-free world: the flight witnesses run in open air.
    fn world() -> World {
        World::new("t".into(), 512, 64)
    }

    /// A slot carrying the 435 row slice BY HAND (`Projectile.cs:5056-
    /// 5063` — the c11 row `a9b22f05`): 10x10, hostile, ignoreWater, the
    /// stock penetrate 1 (`:774`) / tileCollide true (`:775`) / timeLeft
    /// 3600 (`:787`) / alpha 0 (`:779`) / scale 1 / extraUpdates 0.
    /// Field-by-field on `Default` — the struct has private bookkeeping
    /// fields (E0451 outside projectile.rs).
    fn bolt(vx: f32, vy: f32) -> Projectile {
        let mut p = Projectile::default();
        p.active = true;
        p.kind = MARTIAN_TURRET_BOLT;
        p.owner = 255; // the hostile shot (the c11 spawn seam)
        p.pos = Vec2::new(10.0 * TILE_PX, 10.0 * TILE_PX);
        p.vel = Vec2::new(vx, vy);
        p.w = 10.0;
        p.h = 10.0;
        p.tile_collide = true;
        p.alpha = 0;
        p.scale = 1.0;
        p.time_left = 3600;
        p.penetrate = 1;
        p.extra_updates = 0;
        p
    }

    /// A slot carrying the 576 row slice BY HAND (`Projectile.cs:6258-
    /// 6269` — the d11 row): 4x4, hostile, penetrate −1, alpha 255,
    /// extraUpdates 2, scale 1.2, timeLeft 600, the stock tileCollide
    /// true (`:775`).
    fn laser(vx: f32, vy: f32) -> Projectile {
        let mut p = Projectile::default();
        p.active = true;
        p.kind = NEBULA_LASER;
        p.owner = 255; // the hostile shot (the 574 eye's release)
        p.pos = Vec2::new(10.0 * TILE_PX, 10.0 * TILE_PX);
        p.vel = Vec2::new(vx, vy);
        p.w = 4.0;
        p.h = 4.0;
        p.tile_collide = true;
        p.alpha = 255;
        p.scale = 1.2;
        p.time_left = 600;
        p.penetrate = -1;
        p.extra_updates = 2;
        p
    }

    /// The gate is disjoint from A/B/C/D's kind lists and the generic
    /// arrows — `covers` is the seam's whole contract.
    #[test]
    fn w4_f20e_the_gate_is_disjoint_from_a_b_c_and_d() {
        for k in crate::projectile_ai001_a::SLICE_A_KINDS {
            assert!(!covers(k), "slice A owns {k}");
        }
        for k in crate::projectile_ai001_b::SLICE_B_KINDS {
            assert!(!covers(k), "slice B owns {k}");
        }
        for k in crate::projectile_ai001_c::SLICE_C_KINDS {
            assert!(!covers(k), "slice C owns {k}");
        }
        for k in crate::projectile_ai001_d::SLICE_D_KINDS {
            assert!(!covers(k), "slice D owns {k}");
        }
        assert!(!covers(1)); // the generic arrow
        assert!(!covers(2)); // the flame arrow
        assert!(!covers(436)); // the Scrambler keeps slice A
        assert!(!covers(459)); // the Charged Blaster keeps its rider
        assert!(!covers(577)); // the 577 sibling stays unrowed (d11)
        for k in SLICE_E_KINDS {
            assert!(covers(k));
        }
    }

    /// 435 (`:54909-54933`, `:55992` flag3 = false): 30 ticks at (2, 0)
    /// → ai[1] = 1 (the tick-1 latch), the Item12 cue queued ONCE, alpha
    /// stays 0 (the row seeds 0; −40 clamps), ai[0] = 0 (rests), vel.y =
    /// 0 (no gravity), the 3-tick frame machine advances 10 times →
    /// frame 10 mod 4 = 2, spriteDirection 1, rot = atan2(0, 2) = 0.
    #[test]
    fn w4_f20e_the_turret_bolt_latches_frames_and_flies_straight() {
        let w = world();
        let mut p = bolt(2.0, 0.0);
        step(&mut p, &w);
        assert_eq!(p.ai[1], 1.0, "the one-shot latch (`:54913`)");
        assert_eq!(
            p.wants_sound,
            Some((2, 12, 160.0, 160.0, 0.0)),
            "the Item12 cue at base.position (`:54914`)"
        );
        p.wants_sound = None; // the drain consumed it (the pass tail)
        for _ in 0..29 {
            step(&mut p, &w);
        }
        assert_eq!(p.ai[1], 1.0, "the latch never re-arms");
        assert_eq!(p.wants_sound, None, "one cue per life");
        assert_eq!(p.alpha, 0, "the row seeds 0; −40 clamps (`:54916`)");
        assert_eq!(p.ai[0], 0.0, "flag3 = false (`:55992`)");
        assert_eq!(p.vel.y, 0.0, "no gravity — the counter rests");
        assert_eq!(p.frame, 2, "30 ticks / 3 = 10 advances mod 4 (`:54927`)");
        assert_eq!(p.frame_counter, 0, "the 3-tick cadence (`:54923`)");
        assert_eq!(p.sprite_direction, 1, "vx > 0 (`:54921`)");
        assert!(p.rot.abs() < 1e-6, "atan2(0, 2) = 0 (`:57644`)");
        assert!((p.pos.x - (160.0 + 60.0)).abs() < 1e-4, "2 px/tick × 30");
    }

    /// 435's left-facing tail (`:57645-57647`): from (−2, 0) →
    /// spriteDirection −1, rot = atan2(0, −2) + π = 2π.
    #[test]
    fn w4_f20e_the_turret_bolt_adds_pi_when_left_facing() {
        let w = world();
        let mut p = bolt(-2.0, 0.0);
        step(&mut p, &w);
        assert_eq!(p.sprite_direction, -1, "vx < 0 (`:54921`)");
        let want = p.vel.y.atan2(p.vel.x) + std::f32::consts::PI;
        assert!((p.rot - want).abs() < 1e-6, "(`:57646`)");
    }

    /// 435's fade shape (`:54916-54920`): a spawn carrying alpha 255
    /// (the hostile-seam override) fades −40/tick → 0 after 7 ticks
    /// (255→215→175→135→95→55→15→0), then stays 0.
    #[test]
    fn w4_f20e_the_turret_bolt_fades_a_nonzero_alpha_at_forty() {
        let w = world();
        let mut p = bolt(2.0, 0.0);
        p.alpha = 255;
        for _ in 0..7 {
            step(&mut p, &w);
        }
        assert_eq!(p.alpha, 0, "255 - 40×7 clamps at 0 (`:54918`)");
        step(&mut p, &w);
        assert_eq!(p.alpha, 0, "the clamp holds");
    }

    /// 576 (`:55361-55365` latch, `:55635-55649` fade, `:56008` flag3 =
    /// false): the latch fires on tick 1 with ONE Item12 cue; localAI[1]
    /// counts every tick; the fade starts at tick 3 (localAI[1] > 2).
    /// After 2 ticks: alpha still 255, localAI[1] = 2. After 19 ticks:
    /// 17 fade ticks × 15 = 255 → alpha 0, localAI[1] = 19.
    #[test]
    fn w4_f20e_the_nebula_laser_latches_and_fades_after_two() {
        let w = world();
        let mut p = laser(3.0, 0.0);
        step(&mut p, &w);
        assert_eq!(p.ai[1], 1.0, "the one-shot latch (`:55363`)");
        assert_eq!(
            p.wants_sound,
            Some((2, 12, 160.0, 160.0, 0.0)),
            "the Item12 cue at base.position (`:55364`)"
        );
        p.wants_sound = None; // the drain consumed it (the pass tail)
        step(&mut p, &w);
        assert_eq!(p.local_ai[1], 2.0, "the counter (`:55637`)");
        assert_eq!(p.alpha, 255, "the fade gate is > 2 (`:55638`)");
        assert_eq!(p.wants_sound, None, "one cue per life");
        for _ in 0..17 {
            step(&mut p, &w);
        }
        assert_eq!(p.local_ai[1], 19.0);
        assert_eq!(p.alpha, 0, "255 - 15×17 clamps at 0 (`:55642`)");
        assert_eq!(p.ai[0], 0.0, "flag3 = false (`:56008`)");
        assert_eq!(p.vel.y, 0.0, "no gravity — the counter rests");
        // The DEFAULT rotation tail (`:57750`): atan2(0, 3) + 1.57.
        assert!((p.rot - 1.57).abs() < 1e-6);
        assert!((p.pos.x - (160.0 + 57.0)).abs() < 1e-4, "3 px/tick × 19");
    }

    /// 576's left-facing default tail (`:57750` — NOT the :57642 arm):
    /// from (−3, 0) → rot = atan2(0, −3) + 1.57 = π + 1.57, NO extra π.
    #[test]
    fn w4_f20e_the_nebula_laser_keeps_the_default_rotation() {
        let w = world();
        let mut p = laser(-3.0, 0.0);
        step(&mut p, &w);
        let want = p.vel.y.atan2(p.vel.x) + 1.57;
        assert!((p.rot - want).abs() < 1e-6, "(`:57750`)");
    }
}
