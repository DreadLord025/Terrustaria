//! W4-f20 slice D — `AI_001`'s per-type arms for the SEVEN rowed aiStyle-1
//! kinds the S1-D-5 slices A/B/C never covered (`Projectile.cs:54002-57798`,
//! the aiStyle-1 inline region). The census (`docs/research/26e`, finding
//! F6) found these kinds rowed by S1-P-i13/i14/i17 AFTER the slices were
//! declared complete; each fell through to the generic `ai_arrow` despite
//! carrying per-type C# state. Kind set ([`SLICE_D_KINDS`]):
//! 323 Stake, 357 PulseBolt, 485 Hellwing, 661 OnyxBlackBolt,
//! 710 DD2BetsyArrow, 719 QueenBeeStinger, 819 BloodArrow.
//!
//! C# structure preserved (the per-kind hooks above the flight, the
//! `flag3` counter switch `:55947-56041`, the gravity else-if chain
//! `:56753-57581`, the rotation tail chain `:57582-57752`, the `flag8`
//! clamp `:57753-57768`):
//! - 323/485/661/710 are on the `flag3 = false` skip list (`:55983`,
//!   `:56006`, `:56021`, `:56026`) — their flight counter rests at 0.
//!   323/661 still reach the generic `ai[0] >= 15f` gravity arm
//!   (`:57568-57579`), which is unreachable with the resting counter —
//!   a stake or a black bolt flies exactly straight, as in vanilla.
//! - 357/719/819 count (`flag3` true). 719/819 reach the generic 15-tick
//!   gravity; **357 does NOT** — the gravity else-if chain is HEADED by
//!   the 357 fade block (`if (this.type == 357)` `:56753`, census F7), so
//!   the Pulse Bolt matches the first arm (fade only) and never falls.
//! - 485's counter rests, but its rotation-tail arm (`:57708-57736`)
//!   blends the velocity toward the (ai[0], ai[1]) launch vector the
//!   Hellwing Bow seeds (`Player.cs:47573-47578`) — the homing bat.
//! - 710 takes its OWN gravity arm (`:57549-57567`, inside the `:57520`
//!   `type != 442/634/635/675` guard): the counter rests (`flag3` false)
//!   but the arm's `ai[0] >= 0f` gate passes at 0, so it increments ONCE
//!   per tick here; from 20 the +0.2 fall / ×0.98 drag / 12 cap fire.
//!
//! RNG: NONE of the ported state paths draws `Main.rand` — every rand in
//! these kinds' C# blocks feeds dust/light only (the 357 trail
//! `:56755-56768`, the 661 rings `:55736-55764`, the 710 Next(4)/Next(2)
//! dust `:55389-55413`, the 719 bursts `:55497-55516`, the 819 burst /
//! trail / Next(2) red dust `:56393-56420`), which the port skips
//! pool-wide (the ai001_a/b/c convention). The module takes no rng
//! parameter, so the `UpdateProjectiles` stream (`Main.cs:17775-17777`)
//! is untouched BY CONSTRUCTION for every kind. The 719/819 `localAI[0]`
//! one-shot latches gate only skipped cues, so their state rides alone.
//! The 710's nested wet `Kill()` (`:55409-55412`) is UNREACHABLE: the row
//! sets `ignoreWater` (`:7529`), so the Update head's wet block
//! (`:16790`) never sets `base.wet` — documented skip, not a behaviour.
//!
//! Movement is the port's documented kill-on-tile stand-in
//! (`HandleMovement`'s default arm `:19843-19848`), replicated verbatim
//! from `ai_style1_flight`'s tail as slices A/B/C do. The clamp is the
//! C#'s `flag8` tail EXACTLY (`:57753-57768`): only the +16 bound — the
//! −16 bound is `flag4`'s (kind 1006 only) and no slice-D kind sets it.
//!
//! CODE-ONLY MODE (W4-f20): the witness tests at the bottom are WRITTEN
//! NOT RUN (no compiler invoked on this branch); every expected value was
//! verified by re-reading the cited C# blocks and hand arithmetic.

use glam::Vec2;
use tr_core::World;

use crate::npc::Npc;
use crate::projectile::{
    Projectile, BETSY_ARROW, BLOOD_ARROW, HELLWING, ONYX_BLACK_BOLT, PULSE_BOLT, QUEEN_BEE_STINGER,
    STAKE,
};

/// The slice-D kind set — kind-disjoint from `projectile_ai001_a/b/c`,
/// the riders, the frostmoon/moonlord/darts/special-arrow/special-bullet
/// gates, the javelin and the Book of Skulls (the seam chains D after C).
pub(crate) const SLICE_D_KINDS: [u16; 7] = [
    STAKE,             // 323
    PULSE_BOLT,        // 357
    HELLWING,          // 485
    ONYX_BLACK_BOLT,   // 661
    BETSY_ARROW,       // 710
    QUEEN_BEE_STINGER, // 719
    BLOOD_ARROW,       // 819
];

/// Whether this module owns the kind's aiStyle-1 arm.
pub(crate) fn covers(kind: u16) -> bool {
    SLICE_D_KINDS.contains(&kind)
}

/// The per-kind gravity arm of the `:56753-57581` else-if chain.
enum Gravity {
    /// No arm fires: 323/485/661 rest their counter (`flag3 = false`), so
    /// the generic `ai[0] >= 15f` tail (`:57568`) is unreachable; 357
    /// matches the chain's FIRST arm (the fade head, `:56753` — census
    /// F7) and takes no gravity at all.
    None,
    /// `:57568-57579` — the generic arrow tail: at `ai[0] >= 15f` pin 15
    /// and `velocity.Y += 0.1f` (`flag4` false — no slice-D kind is 1006).
    /// 719/819 count into it.
    Generic,
    /// `:57549-57567` — the 710 arm: `ai[0] >= 0f` → `ai[0]++`; from 20
    /// `velocity.Y += 0.2f`, then `velocity.Y > 0f` → `velocity.X *=
    /// 0.98f`, then the 12 cap. The counter rests at the `flag3` switch
    /// but THIS arm increments it (the C#'s second write).
    Betsy,
}

/// `AI_001`'s slice-D body for one slot, in the C#'s own block order.
/// W4-f20 F7: `npcs`/`immune` feed the Harpy-charm tail (`:57769-57793`)
/// in `finish` — the 485/710/819 arrows ARE charm-eligible (the producer's
/// `arrow` gate), the rest are not.
pub(crate) fn step(p: &mut Projectile, world: &World, npcs: &[Npc], immune: &[i32]) {
    match p.kind {
        STAKE => {
            // `:54142-54149` — `alpha -= 50`, clamp 0 (the row seeds
            // alpha 255, `:4168`). No latch, no dust.
            fade_alpha(p, 50);
            // `flag3 = false` (`:55983`): the counter rests, the generic
            // gravity never fires — a straight stake.
            finish(p, world, false, Gravity::None, npcs, immune);
        }
        PULSE_BOLT => {
            // `:56753-56777` — the chain-head fade: the `< 170` trail
            // (`:56755-56768`) is dust-only (SKIPPED); `alpha > 0` →
            // `alpha -= 25`, then the `< 0` clamp. The row seeds 255
            // (`:4480`).
            if p.alpha > 0 {
                p.alpha -= 25;
            }
            if p.alpha < 0 {
                p.alpha = 0;
            }
            // 357 is NOT on the flag3 skip list — the counter counts, but
            // the fade head consumed the gravity chain (census F7): NO
            // gravity, the bolt flies straight until its 600-tick life.
            finish(p, world, true, Gravity::None, npcs, immune);
        }
        HELLWING => {
            // No head hook: the only per-type block is the rotation tail
            // (`:57708-57736`), ported in `finish`'s Hellwing arm.
            // `flag3 = false` (`:56006`): the counter rests.
            finish(p, world, false, Gravity::None, npcs, immune);
        }
        ONYX_BLACK_BOLT => {
            // `:55734-55766` — the `alpha <= 0` dust-240 trio
            // (`:55736-55745`) is SKIPPED (no explicit draws); then
            // `alpha > 0` → `alpha -= 55` AND `scale = 1.3f`; the `< 0`
            // clamp sets alpha 0 and fires the 16-dust-62 ring
            // (`:55752-55764`, SKIPPED — no explicit draws). The row
            // seeds alpha 255 (`:7115`).
            if p.alpha > 0 {
                p.alpha -= 55;
                p.scale = 1.3;
                if p.alpha < 0 {
                    p.alpha = 0;
                }
            }
            // `flag3 = false` (`:56021`): the counter rests, straight.
            finish(p, world, false, Gravity::None, npcs, immune);
        }
        BETSY_ARROW => {
            // `:55382-55414` — `alpha -= 25`, clamp 0 (the row seeds 255,
            // `:7533`); the Next(4) dust block (`:55389-55413`) is
            // SKIPPED with its two draws — the nested wet Kill
            // (`:55409-55412`) is unreachable (ignoreWater, module doc).
            fade_alpha(p, 25);
            // `flag3 = false` (`:56026`): the flag3 counter rests; the
            // 710 gravity arm (`:57549`) does its own increment.
            finish(p, world, false, Gravity::Betsy, npcs, immune);
        }
        QUEEN_BEE_STINGER => {
            // `:55495-55517` — the Next(2) trail dust (`:55497-55500`)
            // and the localAI[0] one-shot's 20+10 dust bursts
            // (`:55501-55516`) are SKIPPED; the latch state rides.
            if p.local_ai[0] == 0.0 {
                p.local_ai[0] = 1.0;
            }
            // 719 is NOT on the flag3 skip list — it counts into the
            // generic 15-tick gravity.
            finish(p, world, true, Gravity::Generic, npcs, immune);
        }
        BLOOD_ARROW => {
            // `:56388-56422` — the localAI[0] one-shot's 8-dust burst
            // (`:56390-56401`, the NextFloatDirection rolls skipped with
            // it), then `alpha -= 20` clamp 0 (the row seeds 255,
            // `:7999`), then the 5-dust trail (`:56407-56414`) + the
            // Next(2) red dust (`:56415-56420`) + the light (`:56421`)
            // SKIPPED. The latch state rides.
            if p.local_ai[0] == 0.0 {
                p.local_ai[0] = 1.0;
            }
            fade_alpha(p, 20);
            // 819 is NOT on the flag3 skip list — it counts into the
            // generic 15-tick gravity (the S1-P-i17 deferral closed).
            finish(p, world, true, Gravity::Generic, npcs, immune);
        }
        _ => unreachable!("the dispatch gates on SLICE_D_KINDS"),
    }
}

/// The C# `alpha -= n` with the `< 0 → 0` clamp (the int alpha cannot
/// underflow the port's i32, so the unconditional subtract is faithful).
fn fade_alpha(p: &mut Projectile, n: i32) {
    p.alpha -= n;
    if p.alpha < 0 {
        p.alpha = 0;
    }
}

/// The shared tail: the `flag3` counter (`:56038-56041`), the per-kind
/// gravity arm, the rotation tail (`:57582-57752`), the `flag8` clamp
/// (`:57753-57768` — the exact C# +16 bound, module doc), then the
/// movement stand-in. W4-f20 F7: the Harpy-charm tail (`:57769-57793`)
/// runs after the clamp, before the movement.
fn finish(
    p: &mut Projectile,
    world: &World,
    count_flight: bool,
    gravity: Gravity,
    npcs: &[Npc],
    immune: &[i32],
) {
    // `Projectile.cs:56038-56041` — `if (flag3) ai[0]++`.
    if count_flight {
        p.ai[0] += 1.0;
    }
    match gravity {
        Gravity::None => {}
        // `Projectile.cs:57568-57579` — the generic arrow gravity.
        Gravity::Generic => {
            if p.ai[0] >= 15.0 {
                p.ai[0] = 15.0;
                p.vel.y += 0.1;
            }
        }
        // `Projectile.cs:57549-57567` — the 710 arm (the C#'s order: the
        // increment, the >= 20 fall, the > 0 drag, the 12 cap).
        Gravity::Betsy => {
            if p.ai[0] >= 0.0 {
                p.ai[0] += 1.0;
                if p.ai[0] >= 20.0 {
                    p.vel.y += 0.2;
                    if p.vel.y > 0.0 {
                        p.vel.x *= 0.98;
                    }
                    if p.vel.y > 12.0 {
                        p.vel.y = 12.0;
                    }
                }
            }
        }
    }
    // The rotation tail chain, in the C#'s order BEFORE the clamp.
    if p.kind == HELLWING {
        // `Projectile.cs:57708-57736` — the 485 arm. The velocity blend
        // toward the (ai[0], ai[1]) launch vector (`:57710-57711`):
        // `velocity = (velocity * 39f + vector45) / 40f`.
        let launch = Vec2::new(p.ai[0], p.ai[1]);
        p.vel = (p.vel * 39.0 + launch) / 40.0;
        // `:57712-57715` — the dust-6 spawn is SKIPPED (no explicit draws).
        // `:57716-57725` — the 2-tick frame machine, wrap at 5
        // (`Main.projFrames[485] = 5`, `Main.cs:8482`, data.rs).
        p.frame_counter += 1;
        if p.frame_counter >= 2 {
            p.frame_counter = 0;
            p.frame += 1;
            if p.frame >= 5 {
                p.frame = 0;
            }
        }
        // `:57726-57735` — the direction-split atan2 (the velocity-sign
        // stand-in for `spriteDirection`, slices A/B/C's convention).
        if p.vel.x < 0.0 {
            p.sprite_direction = -1;
            p.rot = (0.0 - p.vel.y).atan2(0.0 - p.vel.x);
        } else {
            p.sprite_direction = 1;
            p.rot = p.vel.y.atan2(p.vel.x);
        }
    } else {
        // `Projectile.cs:57750` — the default tail (the literal 1.57).
        p.rot = p.vel.y.atan2(p.vel.x) + 1.57;
    }
    // `Projectile.cs:57753-57768` — `flag8` is true for every slice-D
    // kind (only 577 opts out); `flag4` is false, so just the +16 bound.
    if p.vel.y > 16.0 {
        p.vel.y = 16.0;
    }
    // `Projectile.cs:57769-57793` — the Harpy-charm tail (W4-f20 F7).
    crate::projectile_harpy_charm::charm_tail(p, world, npcs, immune);
    move_only(p, world);
}

/// The movement stand-in — a verbatim replication of `ai_style1_flight`'s
/// integration (the `HandleMovement` default arm `Projectile.cs:19843-
/// 19848`, slices A/B/C's `move_only`).
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
// =====================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::{
        proj_data, FlailInputs, ProjectilePool, YoyoInputs, MUSKET_BALL, WOODEN_ARROW,
    };
    use tr_core::{World, TILE_PX};

    /// A tile-free world: the flight witnesses run in open air.
    fn world() -> World {
        World::new("t".into(), 512, 64)
    }

    /// A slot carrying its `SetDefaults` row slice, spawned at (160, 160)
    /// with the given velocity. Field-by-field on `Default` — the struct
    /// has private bookkeeping fields (E0451 outside projectile.rs).
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
        p.tile_collide = d.tile_collide;
        p.alpha = d.alpha;
        p.scale = d.scale;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.extra_updates = d.extra_updates;
        p
    }

    fn step_pool(pool: &mut ProjectilePool, w: &mut World) {
        let mut rng = tr_core::tile_light::FastRandom::new(7);
        let mut npcs = Vec::new();
        pool.update(
            w,
            &mut npcs,
            None,
            None,
            None,
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut rng,
            0,
            false,
        );
    }

    /// The gate is disjoint from A/B/C's kind lists, the riders, the
    /// Book of Skulls and the escape-clause kinds — `covers` is the
    /// seam's whole contract.
    #[test]
    fn w4_f20d_the_gate_is_disjoint_from_a_b_and_c() {
        for k in crate::projectile_ai001_a::SLICE_A_KINDS {
            assert!(!covers(k), "slice A owns {k}");
        }
        for k in crate::projectile_ai001_b::SLICE_B_KINDS {
            assert!(!covers(k), "slice B owns {k}");
        }
        for k in crate::projectile_ai001_c::SLICE_C_KINDS {
            assert!(!covers(k), "slice C owns {k}");
        }
        assert!(!covers(WOODEN_ARROW)); // the generic arrow
        assert!(!covers(2)); // the flame arrow
        assert!(!covers(MUSKET_BALL)); // slice A
        assert!(!covers(20)); // the riders stay on ai_style1_rider
        assert!(!covers(88));
        assert!(!covers(355));
        assert!(!covers(660));
        assert!(!covers(837)); // the book of skulls keeps its own arm
        assert!(!covers(507)); // the javelin keeps projectile_ai_thrown
        assert!(!covers(344)); // the North Pole flake keeps its arm
        assert!(!covers(469)); // the special arrows keep their gate
        assert!(!covers(242)); // the special bullets keep their gate
        assert!(!covers(687)); // the aiStyle-136 data deviation (f19 F5)
        for k in SLICE_D_KINDS {
            assert!(covers(k));
        }
    }

    /// 323 (`:54142-54149` fade 50, `:55983` flag3 = false): 30 ticks at
    /// (2, 0) → vel stays (2, 0), ai[0] = 0, alpha 255 − 50×6 → 0 (6
    /// ticks to clear), rot = atan2(0, 2) + 1.57 = 1.57 every tick.
    #[test]
    fn w4_f20d_the_stake_flies_straight_and_fades_at_fifty() {
        let w = world();
        let mut p = proj(STAKE, 2.0, 0.0);
        for _ in 0..30 {
            step(&mut p, &w, &[], &[]);
        }
        assert_eq!(p.ai[0], 0.0, "flag3 = false (`:55983`)");
        assert_eq!(p.alpha, 0, "255 - 50/tick clamps at 0 (`:54144`)");
        assert_eq!(p.vel.y, 0.0, "no gravity — the counter rests");
        assert!((p.rot - 1.57).abs() < 1e-6);
        assert!((p.pos.x - (160.0 + 60.0)).abs() < 1e-4, "2 px/tick × 30");
    }

    /// 357 (`:56753-56777` fade 25, the chain head — census F7): 30 ticks
    /// at (2, 0) → ai[0] = 30 (counts, NO pin), vel.y = 0 (NO gravity —
    /// the fade head consumed the chain), alpha 255 − 25×11 → 0 (11
    /// ticks: 255→230→…→5→0 at the 11th), rot = 1.57.
    #[test]
    fn w4_f20d_the_pulse_bolt_counts_but_never_falls() {
        let w = world();
        let mut p = proj(PULSE_BOLT, 2.0, 0.0);
        for _ in 0..30 {
            step(&mut p, &w, &[], &[]);
        }
        assert_eq!(p.ai[0], 30.0, "357 is off the flag3 skip list, no pin");
        assert_eq!(p.vel.y, 0.0, "the fade head (`:56753`) takes no gravity");
        assert_eq!(p.alpha, 0, "255 - 25/tick clamps at 0 (`:56771`)");
        assert!((p.rot - 1.57).abs() < 1e-6);
        assert!((p.pos.x - (160.0 + 60.0)).abs() < 1e-4, "2 px/tick × 30");
    }

    /// 485 (`:57708-57736`, `:56006` flag3 = false): the (ai0, ai1) =
    /// (0, −4) launch vector blends the velocity at the weight 40. From
    /// (8, 0): tick 1 vel = (8×39/40, −4/40) = (7.8, −0.1); the frame
    /// advances every 2 ticks (30 ticks → 15 advances → frame 0 mod 5);
    /// spriteDirection = 1 (vx > 0), rot = atan2(vy, vx).
    #[test]
    fn w4_f20d_the_hellwing_blends_toward_the_launch_vector() {
        let w = world();
        let mut p = proj(HELLWING, 8.0, 0.0);
        p.ai[0] = 0.0;
        p.ai[1] = -4.0;
        step(&mut p, &w, &[], &[]);
        assert!((p.vel.x - 7.8).abs() < 1e-5, "8 × 39/40 (`:57711`)");
        assert!((p.vel.y - (-0.1)).abs() < 1e-6, "-4/40 (`:57711`)");
        assert_eq!(p.ai[0], 0.0, "flag3 = false — the blend reads, never writes ai");
        assert_eq!(p.sprite_direction, 1, "vx > 0 (`:57733`)");
        assert!((p.rot - p.vel.y.atan2(p.vel.x)).abs() < 1e-6, "(`:57734`)");
        for _ in 0..29 {
            step(&mut p, &w, &[], &[]);
        }
        assert_eq!(p.frame, 0, "30 ticks / 2 = 15 advances mod 5 (`:57721`)");
        assert_eq!(p.frame_counter, 0, "the 2-tick cadence (`:57717`)");
        // The blend converges toward the launch (0, −4): vx decays by
        // ×39/40 each tick (the launch X is 0), so after 30 ticks
        // vx = 8 × 0.975^30; vy approaches −4 from above.
        assert!((p.vel.x - 8.0 * 0.975f32.powi(30)).abs() < 1e-4, "vx {}", p.vel.x);
        assert!(p.vel.y < 0.0 && p.vel.y > -4.0, "vy converges to −4: {}", p.vel.y);
    }

    /// 485's left-facing tail (`:57726-57730`): from (−8, 0) with launch
    /// (0, −4) → vx < 0 → spriteDirection = −1, rot = atan2(−vy, −vx).
    #[test]
    fn w4_f20d_the_hellwing_flips_the_rotation_when_left_facing() {
        let w = world();
        let mut p = proj(HELLWING, -8.0, 0.0);
        p.ai[1] = -4.0;
        step(&mut p, &w, &[], &[]);
        assert_eq!(p.sprite_direction, -1, "vx < 0 (`:57728`)");
        let want = (0.0 - p.vel.y).atan2(0.0 - p.vel.x);
        assert!((p.rot - want).abs() < 1e-6, "(`:57729`)");
    }

    /// 661 (`:55734-55766` fade 55 + scale 1.3, `:56021` flag3 = false):
    /// 30 ticks at (2, 0) → alpha 255 − 55×5 → 0 (5 ticks: 255→200→145→
    /// 90→35→0), scale 1.3 (set on every fade tick), vel.y = 0, ai[0] = 0.
    #[test]
    fn w4_f20d_the_black_bolt_fades_at_fiftyfive_and_scales() {
        let w = world();
        let mut p = proj(ONYX_BLACK_BOLT, 2.0, 0.0);
        for _ in 0..30 {
            step(&mut p, &w, &[], &[]);
        }
        assert_eq!(p.alpha, 0, "255 - 55/tick clamps at 0 (`:55748`)");
        assert!((p.scale - 1.3).abs() < 1e-6, "the scale write (`:55749`)");
        assert_eq!(p.ai[0], 0.0, "flag3 = false (`:56021`)");
        assert_eq!(p.vel.y, 0.0, "no gravity — the counter rests");
    }

    /// 710 (`:55382-55414` fade 25 + `:57549-57567` gravity, `:56026`
    /// flag3 = false): the arm increments ai[0] from 0; 25 ticks at
    /// (10, 0) → ai[0] = 25, the fall fired on ticks 20..25 = 6 times →
    /// vel.y = 6 × 0.2 = 1.2, vx = 10 × 0.98^6 (the drag fires with the
    /// fall while vy > 0), alpha 255 − 25×11 → 0.
    #[test]
    fn w4_f20d_the_betsy_arrow_falls_after_twenty_at_02() {
        let w = world();
        let mut p = proj(BETSY_ARROW, 10.0, 0.0);
        for _ in 0..25 {
            step(&mut p, &w, &[], &[]);
        }
        assert_eq!(p.ai[0], 25.0, "the arm's own increment (`:57553`)");
        assert!((p.vel.y - 1.2).abs() < 1e-5, "6 fall ticks × 0.2 (`:57556`)");
        let want_vx = 10.0 * 0.98f32.powi(6);
        assert!((p.vel.x - want_vx).abs() < 1e-4, "vx {}", p.vel.x);
        assert_eq!(p.alpha, 0, "255 - 25/tick (`:55384`)");
    }

    /// 710's 12 cap (`:57561-57564`): 80 ticks → the fall fires 61 times
    /// (ticks 20..80), vel.y would be 12.2 but the cap pins 12 from the
    /// 61st fall tick (ai[0] = 80).
    #[test]
    fn w4_f20d_the_betsy_arrow_caps_the_fall_at_twelve() {
        let w = world();
        let mut p = proj(BETSY_ARROW, 10.0, 0.0);
        for _ in 0..80 {
            step(&mut p, &w, &[], &[]);
        }
        assert_eq!(p.ai[0], 80.0);
        assert!((p.vel.y - 12.0).abs() < 1e-5, "the 12 cap (`:57563`)");
    }

    /// 719 (`:55495-55517` latch, generic gravity): 20 ticks at (2, 0) →
    /// localAI[0] = 1 (the one-shot), ai[0] pinned at 15, the fall fired
    /// on ticks 15..20 = 6 times → vel.y = 0.6.
    #[test]
    fn w4_f20d_the_stinger_latches_and_falls_at_fifteen() {
        let w = world();
        let mut p = proj(QUEEN_BEE_STINGER, 2.0, 0.0);
        for _ in 0..20 {
            step(&mut p, &w, &[], &[]);
        }
        assert_eq!(p.local_ai[0], 1.0, "the one-shot (`:55503`)");
        assert_eq!(p.ai[0], 15.0, "the generic pin (`:57570`)");
        assert!((p.vel.y - 0.6).abs() < 1e-5, "6 gravity ticks × 0.1");
    }

    /// 819 (`:56388-56422` latch + fade 20, generic gravity): 20 ticks at
    /// (2, 0) → localAI[0] = 1, alpha 255 − 20×13 → 0 (13 ticks to
    /// clear), ai[0] pinned 15, vel.y = 0.6 (the i17 deferral closed).
    #[test]
    fn w4_f20d_the_blood_arrow_latches_fades_and_falls() {
        let w = world();
        let mut p = proj(BLOOD_ARROW, 2.0, 0.0);
        for _ in 0..20 {
            step(&mut p, &w, &[], &[]);
        }
        assert_eq!(p.local_ai[0], 1.0, "the one-shot (`:56392`)");
        assert_eq!(p.alpha, 0, "255 - 20/tick (`:56402`)");
        assert_eq!(p.ai[0], 15.0, "the generic pin (`:57570`)");
        assert!((p.vel.y - 0.6).abs() < 1e-5, "6 gravity ticks × 0.1");
    }

    /// The pool seam: a spawned 357 rides the slice-D arm (no gravity),
    /// NOT the generic `ai_arrow` (which would fall after 15 ticks). The
    /// row's extraUpdates 2 (`:4483`) means each `update` runs 3 sub-steps
    /// (`:16597`), so 10 updates = 30 AI ticks: vel.y stays 0 and ai[0]
    /// counts to 30 — the census F7 behaviour through the live dispatch.
    #[test]
    fn w4_f20d_the_seam_routes_the_pulse_bolt_to_slice_d() {
        let mut w = world();
        let mut pool = ProjectilePool::new();
        let slot = pool
            .spawn_with_data_ai_owned(PULSE_BOLT, 0, 160.0, 160.0, 2.0, 0.0, 10, 0.0, 0.0, 0.0)
            .expect("the 357 row");
        for _ in 0..10 {
            step_pool(&mut pool, &mut w);
        }
        let p = &pool.slots()[slot];
        assert!(p.active);
        assert_eq!(p.vel.y, 0.0, "slice D: no gravity (the generic arm would fall)");
        assert_eq!(p.ai[0], 30.0, "10 updates × 3 sub-steps, no pin");
    }
}
