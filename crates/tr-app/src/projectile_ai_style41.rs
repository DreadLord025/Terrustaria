//! W4-d5 (aiStyle-41) — the FlowerPetal (221), `Projectile.AI`'s
//! `else if (this.aiStyle == 41)` block (`Projectile.cs:27784-27792`),
//! ported 1:1 as a PURE machine (code-only mode: no compiler/tests were
//! run on this branch; the witness tests below are written, not executed).
//!
//! ## Census (research/156 row 41 — was 26d, registry §3)
//!
//! ONE vanilla member: FlowerPetal 221 (`ProjectileID.cs:996`,
//! `SetDefaults` `Projectile.cs:3066-3078`: 20×20, friendly,
//! `tileCollide = false`, `ignoreWater = true`, `timeLeft = 120`,
//! `penetrate = -1`, `scale = 1f + Next(30) * 0.01f`, `extraUpdates = 2`).
//! The port has no `proj_data` row for 221, so this module lands UNWIRED
//! (the `npc_ai_style25` pre-wiring precedent): a pure fn on the slot,
//! reachable and testable before the row + dispatch arm land.
//!
//! ## The machine (verbatim)
//!
//! * `:27786-27790` — the one-time latch: `localAI[0] == 0f` sets it to 1
//!   and picks the sheet row `frame = Main.rand.Next(3)`
//!   (`Main.projFrames[221] = 3`, `Main.cs:8601`/`:8604`).
//! * `:27791` — every tick `rotation += velocity.X * 0.01f`.
//!
//! Movement: aiStyle 41 is NOT in `UpdatePosition`'s skip list
//! (`:20435`), so the generic `position += velocity` (`:20445`) runs AFTER
//! the AI — the wiring caller owns it (`tileCollide = false`, the
//! `kill_step` free-move branch).
//!
//! ## RNG contract
//!
//! ONE kept draw per life: the latch's `Next(3)` frame pick, on the
//! `UpdateProjectiles` stream (`Main.cs:17775-17777`). The spawn-time
//! scale roll (`:3076`, `Next(30)`) belongs to `SetDefaults` — carried as
//! [`petal_spawn_scale`] for the wiring spawn site (the frost-row
//! precedent: the roll runs at spawn, not per tick).
#![allow(dead_code)] // UNWIRED (W4-d5): the 221 row + the dispatch arm are the wiring step

use crate::projectile::Projectile;
use tr_core::loot::DropRng;

/// `ProjectileID.FlowerPetal` (`ProjectileID.cs:996`).
pub const FLOWER_PETAL: u16 = 221;

/// The `SetDefaults` row (`Projectile.cs:3066-3078`) — carried as consts
/// until the `proj_data` row lands (the module-doc census).
pub const PETAL_DEFAULTS: (f32, f32, i32, i32, u32) =
    // (width, height, timeLeft, penetrate, extraUpdates)
    (20.0, 20.0, 120, -1, 2);

/// `SetDefaults`' spawn-time scale roll (`:3076`):
/// `this.scale = 1f + (float)Main.rand.Next(30) * 0.01f` — one `Next(30)`
/// on the spawning stream, BEFORE the projectile ticks.
pub fn petal_spawn_scale(rng: &mut dyn DropRng) -> f32 {
    1.0 + rng.next_range(0, 30) as f32 * 0.01
}

/// `AI()`'s aiStyle-41 arm (`Projectile.cs:27784-27792`), verbatim.
pub fn petal_step(p: &mut Projectile, rng: &mut dyn DropRng) {
    // `:27786-27790` — the one-time frame latch.
    if p.local_ai[0] == 0.0 {
        p.local_ai[0] = 1.0;
        p.frame = rng.next_range(0, 3);
    }
    // `:27791` — the spin rides the X velocity every tick.
    p.rot += p.vel.x * 0.01;
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;
    use tr_core::UnifiedRandom;

    fn petal(vel: Vec2) -> Projectile {
        Projectile {
            active: true,
            kind: FLOWER_PETAL,
            owner: 0,
            vel,
            w: PETAL_DEFAULTS.0,
            h: PETAL_DEFAULTS.1,
            time_left: PETAL_DEFAULTS.2,
            penetrate: PETAL_DEFAULTS.3,
            max_penetrate: PETAL_DEFAULTS.3,
            extra_updates: PETAL_DEFAULTS.4,
            tile_collide: false,
            ignore_water: true,
            ..Default::default()
        }
    }

    /// `:27786-27790` — the first tick draws ONE Next(3) and latches; the
    /// frame is the draw's value, localAI[0] becomes 1.
    #[test]
    fn w4_d5_s41_the_first_tick_latches_the_frame() {
        let mut rng = UnifiedRandom::new(41);
        let mut oracle = UnifiedRandom::new(41);
        let mut p = petal(Vec2::new(3.0, 0.0));
        petal_step(&mut p, &mut rng);
        assert_eq!(p.local_ai[0], 1.0);
        assert_eq!(p.frame, oracle.next_range(0, 3));
        assert_eq!(rng.next_range(0, 1), oracle.next_range(0, 1), "one draw only");
    }

    /// The latch fires ONCE: 30 more ticks draw nothing and never rewrite
    /// the frame (`localAI[0] == 1` from tick 2 on).
    #[test]
    fn w4_d5_s41_the_latch_never_refires() {
        let mut rng = UnifiedRandom::new(7);
        let mut p = petal(Vec2::new(2.0, 1.0));
        petal_step(&mut p, &mut rng);
        let frame = p.frame;
        let mut control = UnifiedRandom::new(7);
        let _ = control.next_range(0, 3); // the latch's draw
        for _ in 0..30 {
            petal_step(&mut p, &mut rng);
            assert_eq!(p.frame, frame);
        }
        assert_eq!(rng.next_range(0, 1), control.next_range(0, 1), "zero further draws");
    }

    /// `:27791` — the spin accumulates `vel.X * 0.01` per tick, signed.
    #[test]
    fn w4_d5_s41_the_spin_accumulates_signed() {
        let mut rng = UnifiedRandom::new(1);
        let mut p = petal(Vec2::new(-4.0, 9.0));
        for _ in 0..10 {
            petal_step(&mut p, &mut rng);
        }
        assert!((p.rot - (-0.4)).abs() < 1e-5, "10 × -4 × 0.01 = -0.4");
    }

    /// `:3076` — the spawn scale is `1 + Next(30) * 0.01`, one draw.
    #[test]
    fn w4_d5_s41_the_spawn_scale_rolls_one_next30() {
        let mut rng = UnifiedRandom::new(99);
        let mut oracle = UnifiedRandom::new(99);
        let s = petal_spawn_scale(&mut rng);
        assert_eq!(s, 1.0 + oracle.next_range(0, 30) as f32 * 0.01);
        assert!((1.0..1.30).contains(&s));
        assert_eq!(rng.next_range(0, 1), oracle.next_range(0, 1), "one draw only");
    }
}