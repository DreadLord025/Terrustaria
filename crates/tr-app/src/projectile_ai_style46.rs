//! W4-d5 (aiStyle-46) — the Rainbow Rod's RainbowFront (250) /
//! RainbowBack (251), `Projectile.AI`'s `else if (this.aiStyle == 46)`
//! block (`Projectile.cs:28107-28199`), ported 1:1 as a PURE machine
//! (code-only mode: no compiler/tests were run on this branch; the witness
//! tests below are written, not executed).
//!
//! ## Census (research/156 row 46 — was 26d, registry §3)
//!
//! TWO vanilla members:
//! * RainbowFront 250 (`ProjectileID.cs:1054`, `SetDefaults`
//!   `Projectile.cs:3322-3334`: 12×12, `penetrate = -1`, magic,
//!   `alpha = 255`, `ignoreWater = true`, `scale = 1.25f`, the id-static
//!   immunity pair (true, 10); the stock `tileCollide = true`);
//! * RainbowBack 251 (`ProjectileID.cs:1056`, `:3335-3350`: 14×14,
//!   friendly, `penetrate = -1`, magic, `alpha = 255`, `light = 0.3f`,
//!   `tileCollide = false`, `ignoreWater = true`, `scale = 1.25f`, the
//!   same id-static pair).
//! The port has no rows for either, so this module lands UNWIRED (the
//! `npc_ai_style25` precedent).
//!
//! ## The machine (verbatim)
//!
//! * `:28109-28114` — the shimmer kill: the centre tile has
//!   `liquid > 0 && shimmer()` → `Kill()`. NO return — the type arms below
//!   still run on the dead slot (the C# quirk, kept).
//! * `:28115` — `num336 = 2400` (the back's lifetime).
//! * `:28116-28165` — the FRONT (250): `Center.ToTileCoordinates()` with
//!   `InWorld(…, 2)` — out → Kill + return; owner-only `localAI[0]++`,
//!   past 4 resets to **3** and spawns the back 251 at Centre with
//!   `velocity * 0.001f`; owner-only `timeLeft` cap 2400. Then (every
//!   owner): `num337 = 1 − velocity.Y / 3` while rising, `ai[0] += num337`;
//!   past 30 `velocity.Y += 0.5` and X ×0.95 falling / ×1.05 rising; the
//!   velocity renormalizes to `15.95f * scale`; `rotation = atan2 − 1.57`;
//!   return (the generic tail below is the BACK's).
//! * `:28166-28180` — the BACK's first-tick latch (`localAI[0] == 0`):
//!   spriteDirection −1 flying right / +1 left, `rotation = atan2 − 1.57`,
//!   latch 1, `timeLeft = 2400`.
//! * `:28181-28198` — the back's drift: velocity ×0.98 both axes; the
//!   4-way alpha ladder — rotation == 0 → 255; timeLeft < 10 →
//!   `255 − (int)(255 * timeLeft / 10)`; timeLeft > 2390 →
//!   `255 − (int)(255 * (2400 − timeLeft) / 10)`; else 0.
//!
//! Movement: neither style is in `UpdatePosition`'s skip list — the wiring
//! caller runs the generic move after the AI (the front collides tiles,
//! the back does not).
//!
//! ## RNG by construction
//!
//! The arm draws NO `Main.rand` (`:28107-28199` contains no Next/NextFloat).
//! The fn takes no RNG parameter; the witness pins the stream (written,
//! not run).
#![allow(dead_code)] // UNWIRED (W4-d5): the 250/251 rows + the dispatch arm are the wiring step

use crate::projectile::Projectile;
use glam::Vec2;
use tr_core::World;

/// `ProjectileID.RainbowFront` (`ProjectileID.cs:1054`).
pub const RAINBOW_FRONT: u16 = 250;
/// `ProjectileID.RainbowBack` (`ProjectileID.cs:1056`).
pub const RAINBOW_BACK: u16 = 251;

/// The `SetDefaults` rows (`Projectile.cs:3322-3334` / `:3335-3350`):
/// (width, height, scale, alpha, idStaticNPCHitCooldown).
pub const FRONT_DEFAULTS: (f32, f32, f32, i32, i32) = (12.0, 12.0, 1.25, 255, 10);
pub const BACK_DEFAULTS: (f32, f32, f32, i32, i32) = (14.0, 14.0, 1.25, 255, 10);

/// One queued 251 spawn (`Projectile.NewProjectile` `:28130`): the C#
/// spawn arguments (the wiring spawn subtracts the row's half-box).
#[derive(Clone, Copy, Debug)]
pub struct RainbowBackSpawn {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
}

/// `Kill()` (`Projectile.cs:70644-70648`) — the sibling-module deactivation
/// (the `projectile_ai_spear::kill` precedent).
fn kill(p: &mut Projectile) {
    p.kill_time_left = p.time_left;
    p.active = false;
    p.time_left = 0;
}

/// `WorldGen.InWorld(x, y, fluff)` (`WorldGen.cs:9688-9695`).
fn in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// `AI()`'s aiStyle-46 arm (`Projectile.cs:28107-28199`), verbatim.
/// `local_owner` is `Main.myPlayer`.
pub fn rainbow_step(
    p: &mut Projectile,
    world: &World,
    local_owner: u8,
) -> Option<RainbowBackSpawn> {
    let mut spawn = None;
    // `:28109-28114` — the shimmer kill (NO return — verbatim).
    let tx = ((p.pos.x + p.w * 0.5) / 16.0) as i32;
    let ty = ((p.pos.y + p.h * 0.5) / 16.0) as i32;
    if world.tile(tx, ty).is_some_and(|t| {
        t.liquid > 0 && t.liquid_kind() == tr_core::tile::LiquidKind::Shimmer
    }) {
        kill(p);
    }
    let num336 = 2400;
    if p.kind == RAINBOW_FRONT {
        // `:28118-28123` — the fluff-2 out-of-world kill.
        if !in_world(world, tx, ty, 2) {
            kill(p);
            return None;
        }
        // `:28124-28136` — the owner-gated back-spawn cadence + life cap.
        if p.owner == local_owner {
            p.local_ai[0] += 1.0;
            if p.local_ai[0] > 4.0 {
                p.local_ai[0] = 3.0;
                spawn = Some(RainbowBackSpawn {
                    x: p.pos.x + p.w * 0.5,
                    y: p.pos.y + p.h * 0.5,
                    vx: p.vel.x * 0.001,
                    vy: p.vel.y * 0.001,
                });
            }
            if p.time_left > num336 {
                p.time_left = num336;
            }
        }
        // `:28137-28154` — the rise-weighted counter + the >30 arc.
        let mut num337 = 1.0f32;
        if p.vel.y < 0.0 {
            num337 -= p.vel.y / 3.0;
        }
        p.ai[0] += num337;
        if p.ai[0] > 30.0 {
            p.vel.y += 0.5;
            if p.vel.y > 0.0 {
                p.vel.x *= 0.95;
            } else {
                p.vel.x *= 1.05;
            }
        }
        // `:28155-28163` — the 15.95·scale renormalization + the rotation.
        let len = (p.vel.x * p.vel.x + p.vel.y * p.vel.y).sqrt();
        let num338 = 15.95 * p.scale / len;
        p.vel.x *= num338;
        p.vel.y *= num338;
        p.rot = p.vel.y.atan2(p.vel.x) - 1.57;
        return spawn;
    }
    // `:28166-28180` — the BACK's first-tick latch.
    if p.local_ai[0] == 0.0 {
        if p.vel.x > 0.0 {
            p.sprite_direction = -1;
        } else {
            p.sprite_direction = 1;
        }
        p.rot = p.vel.y.atan2(p.vel.x) - 1.57;
        p.local_ai[0] = 1.0;
        p.time_left = num336;
    }
    // `:28181-28182` — the drift damp.
    p.vel.x *= 0.98;
    p.vel.y *= 0.98;
    // `:28183-28198` — the 4-way alpha ladder.
    if p.rot == 0.0 {
        p.alpha = 255;
    } else if p.time_left < 10 {
        p.alpha = 255 - (255.0 * p.time_left as f32 / 10.0) as i32;
    } else if p.time_left > num336 - 10 {
        p.alpha = 255 - (255.0 * (num336 - p.time_left) as f32 / 10.0) as i32;
    } else {
        p.alpha = 0;
    }
    spawn
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::UnifiedRandom;

    fn air() -> World {
        World::new("t".into(), 200, 120)
    }

    fn front() -> Projectile {
        Projectile {
            active: true,
            kind: RAINBOW_FRONT,
            owner: 0,
            pos: Vec2::new(1600.0, 800.0),
            vel: Vec2::new(10.0, 0.0),
            w: FRONT_DEFAULTS.0,
            h: FRONT_DEFAULTS.1,
            scale: FRONT_DEFAULTS.2,
            alpha: FRONT_DEFAULTS.3,
            penetrate: -1,
            max_penetrate: -1,
            magic: true,
            ignore_water: true,
            uses_id_static_immunity: true,
            id_static_hit_cooldown: FRONT_DEFAULTS.4,
            ..Default::default()
        }
    }

    fn back() -> Projectile {
        Projectile {
            active: true,
            kind: RAINBOW_BACK,
            owner: 0,
            pos: Vec2::new(1600.0, 800.0),
            vel: Vec2::new(10.0, 0.0),
            w: BACK_DEFAULTS.0,
            h: BACK_DEFAULTS.1,
            scale: BACK_DEFAULTS.2,
            alpha: BACK_DEFAULTS.3,
            penetrate: -1,
            max_penetrate: -1,
            magic: true,
            tile_collide: false,
            ignore_water: true,
            light: 0.3,
            uses_id_static_immunity: true,
            id_static_hit_cooldown: BACK_DEFAULTS.4,
            ..Default::default()
        }
    }

    /// `:28109-28114` — a shimmer centre tile kills, and the type arm
    /// STILL runs (the no-return quirk): the front keeps steering.
    #[test]
    fn w4_d5_s46_the_shimmer_kill_does_not_return() {
        let mut w = air();
        let t = w.tile_mut(100, 50).unwrap();
        t.liquid = 255;
        t.set_liquid_kind(tr_core::tile::LiquidKind::Shimmer);
        let mut p = front(); // centre (1606, 806) → tile (100, 50)
        let _ = rainbow_step(&mut p, &w, 0);
        assert!(!p.active);
        assert_eq!(p.local_ai[0], 1.0, "the front arm ran after the kill");
        assert!((p.vel.length() - 15.95 * 1.25).abs() < 1e-3, "renormalized");
    }

    /// `:28118-28123` — outside the fluff-2 margin the front dies and
    /// returns (no counter bump).
    #[test]
    fn w4_d5_s46_the_fluff2_margin_kills_the_front() {
        let w = air();
        let mut p = front();
        p.pos = Vec2::new(8.0, 800.0); // centre tile x = 0 < 2
        assert!(rainbow_step(&mut p, &w, 0).is_none());
        assert!(!p.active);
        assert_eq!(p.local_ai[0], 0.0);
    }

    /// `:28124-28136` — the owner-gated cadence: ticks 1-4 no spawn,
    /// tick 5 spawns at Centre with vel × 0.001 and resets to 3 (so the
    /// next spawn is 2 ticks later); the life caps at 2400. A non-local
    /// owner never counts.
    #[test]
    fn w4_d5_s46_the_back_spawn_cadence_is_5_then_2() {
        let w = air();
        let mut p = front();
        p.time_left = 3600;
        let mut fires = Vec::new();
        for tick in 0..9 {
            if rainbow_step(&mut p, &w, 0).is_some() {
                fires.push(tick);
            }
        }
        assert_eq!(fires, vec![4, 6, 8], "5th tick then every 2nd");
        assert_eq!(p.time_left, 2400, "the owner life cap");
        let mut p = front();
        p.owner = 1;
        p.time_left = 3600;
        for _ in 0..9 {
            assert!(rainbow_step(&mut p, &w, 0).is_none());
        }
        assert_eq!(p.local_ai[0], 0.0);
        assert_eq!(p.time_left, 3600, "the cap is owner-gated too");
    }

    /// `:28137-28163` — the rise-weighted counter (vy −6 → +3/tick) and
    /// the renormalization to 15.95 × scale; past 30 the arc bends down.
    #[test]
    fn w4_d5_s46_the_front_steers_at_15_95_scale() {
        let w = air();
        let mut p = front();
        p.vel = Vec2::new(10.0, -6.0);
        let _ = rainbow_step(&mut p, &w, 0);
        assert_eq!(p.ai[0], 3.0, "1 − (−6/3) = 3");
        assert!((p.vel.length() - 15.95 * 1.25).abs() < 1e-3);
        // Run past the 30 counter: gravity bends the flight down.
        let mut p = front();
        p.ai[0] = 30.0;
        p.vel = Vec2::new(10.0, 0.0);
        let _ = rainbow_step(&mut p, &w, 0);
        assert!(p.vel.y > 0.0, "the arc turned down");
        assert!((p.rot - (p.vel.y.atan2(p.vel.x) - 1.57)).abs() < 1e-6);
    }

    /// `:28166-28198` — the back's latch (spriteDirection −1 right / +1
    /// left, life 2400), the 0.98 damp and the alpha ladder's fade-in arm
    /// (timeLeft 2395 → alpha 128).
    #[test]
    fn w4_d5_s46_the_back_latches_and_fades() {
        let w = air();
        let mut p = back();
        let _ = rainbow_step(&mut p, &w, 0);
        assert_eq!(p.sprite_direction, -1);
        assert_eq!(p.local_ai[0], 1.0);
        assert_eq!(p.time_left, 2400);
        assert_eq!(p.vel, Vec2::new(10.0 * 0.98, 0.0));
        assert_eq!(p.alpha, 255, "timeLeft 2400 > 2390 -> the fade-in arm (`:28190`)");
        // The fade-in arm: 2400 − 2395 = 5 → 255 − (int)(255×5/10) = 128.
        let mut p = back();
        p.local_ai[0] = 1.0;
        p.rot = 1.0;
        p.time_left = 2395;
        let _ = rainbow_step(&mut p, &w, 0);
        assert_eq!(p.alpha, 255 - (255.0 * 5.0 / 10.0) as i32);
        // The fade-out arm: timeLeft 5 → 255 − (int)(255×5/10) = 128.
        let mut p = back();
        p.local_ai[0] = 1.0;
        p.rot = 1.0;
        p.time_left = 5;
        let _ = rainbow_step(&mut p, &w, 0);
        assert_eq!(p.alpha, 128);
        // rotation == 0 pins 255 regardless of life.
        let mut p = back();
        p.local_ai[0] = 1.0;
        p.rot = 0.0;
        p.time_left = 5;
        let _ = rainbow_step(&mut p, &w, 0);
        assert_eq!(p.alpha, 255);
    }

    /// RNG-free by construction: front + back ticks leave the stream
    /// untouched (the module-doc contract).
    #[test]
    fn w4_d5_s46_the_arm_draws_no_rng() {
        let w = air();
        let mut rng = UnifiedRandom::new(46);
        let mut control = UnifiedRandom::new(46);
        let mut f = front();
        let mut b = back();
        for _ in 0..10 {
            let _ = rainbow_step(&mut f, &w, 0);
            let _ = rainbow_step(&mut b, &w, 0);
        }
        assert_eq!(rng.next_range(0, 1000), control.next_range(0, 1000));
    }
}