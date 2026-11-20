//! W4-d5 (aiStyle-49) — the Bunny Cannon's ExplosiveBunny (281):
//! `Projectile.AI`'s `else if (this.aiStyle == 49)` block
//! (`Projectile.cs:28243-28322`), its `HandleMovement` collision arm
//! (`:18833-18860`) and its `Kill()` hook (`:75749-75803`), ported 1:1 as
//! PURE machines (code-only mode: no compiler/tests were run on this
//! branch; the witness tests below are written, not executed).
//!
//! ## Census (research/26d row 49, finding F2)
//!
//! ONE vanilla member: ExplosiveBunny 281 (`ProjectileID.cs:1116`,
//! `SetDefaults` `Projectile.cs:3676-3687`: 28×28, friendly, `alpha = 255`,
//! `timeLeft = 600`, the id-static immunity pair (true, 10),
//! `penetrate = -1`). The port carries only the `CANNON_BUNNY` const
//! (`projectile/data.rs:710`, anchors the `BunnyCannonCanFire` scan) — no
//! row, no arm. The `ai_style2` `162 | 281` ladder (projectile.rs:2658)
//! mirrors the C#'s DEAD `AI_002` type-281 branch (`:23321-23365`): 281
//! rides aiStyle 49, so `AI_002` never runs for it in C# either — faithful
//! mirror, kept. This module lands UNWIRED (the `npc_ai_style25`
//! precedent); the wiring round adds the row + the dispatch arm.
//!
//! ## The AI machine (verbatim, `:28243-28322`)
//!
//! * `ai[0] == -2` → `hostile = true`, Kill, return; `ai[0] == -3` → Kill,
//!   return (the released-bunny corpse states).
//! * `soundDelay == 0` → re-arm 3000, play Item14 (the fuse hiss loop).
//! * `ai[0] >= 0` — the flight: direction from the X sign (0 keeps),
//!   spriteDirection = direction, `ai[0]++`,
//!   `rotation += vel.X * 0.05 + direction * 0.05`; at `ai[0] >= 18`
//!   gravity `vy += 0.28`, `vx *= 0.99`; the Y cap is a DOUBLE compare
//!   (`(double)vy > 15.9` → `vy = 15.9f`); `!(ai[0] > 2)` returns; alpha 0;
//!   at `ai[0] == 3` the fuse-spark burst (10× dust 31 + 5×2 dust 6 — no
//!   call-site draws — and ONE gore `Next(61,64)` + its two internal
//!   kicks).
//! * `ai[0] == -1` — the rest: rotation 0, `vx *= 0.95`, `vy += 0.2`.
//!
//! ## The movement arm (verbatim, `:18833-18860`)
//!
//! On a blocked move: `flag15 = |vx| + |vy| < 2 || ai[0] == -1`, forced
//! false at −2/−3. flag15 + local owner → `NPC.ReleaseNPC(Style:
//! (vx > 0 ? 1 : 2) + 2, x: (int)Center.X, y: (int)Bottom.Y − 4, Type: 614)`
//! + `ai[0] = -3` + netUpdate. Else the 0.5× per-axis bounce against
//! `lastVelocity`.
//!
//! ## The Kill hook (verbatim, `:75749-75803`)
//!
//! `ai[0] != -3` only: sound 4 (int overload, style 1), gore 76 + 77 (each
//! TWO call-site `Next(-20,21) * 0.2` velocity draws, X then Y, + the two
//! NewGore internal kicks — Y `Next(10,31)` then X `Next(-20,21)`,
//! `Gore.cs:1327-1328` — then `velocity -= base.velocity * 0.5`), Item14,
//! 20× dust 31 + 10×2 dust 6 (no call-site draws), 4× gore `Next(61,64)`
//! (+ kicks, ×0.4 velocity, the ±1 cardinal nudges); `damage > 0` → the
//! 128×128 centre-preserving resize, `ai[0] == -2` → hostile, `Damage()`.
//!
//! ## Skips (documented)
//!
//! The dust/gore BODIES (no draw layer) — every call-site draw and the
//! NewGore internal kicks STAY (the `projectile_ai_misc` G3 convention).
//! The ChildSafety replacement draws (`Gore.cs:1334-1339`) are the standing
//! port scope-out (ChildSafety off). The `Damage()` walk is the wiring
//! caller's (the resized box rides the slot; [`BunnyKill`] reports the
//! intent). The sounds ride `wants_sound` (single slot — the last cue
//! wins on a multi-cue tick, the boomerang precedent).
#![allow(dead_code)] // UNWIRED (W4-d5): the 281 row + the dispatch arm are the wiring step

use crate::projectile::Projectile;
use glam::Vec2;
use tr_core::loot::DropRng;

/// `ProjectileID.ExplosiveBunny` (`ProjectileID.cs:1116`).
pub const EXPLOSIVE_BUNNY: u16 = 281;

/// The `SetDefaults` row (`Projectile.cs:3676-3687`):
/// (width, height, timeLeft, penetrate, alpha, idStaticNPCHitCooldown).
pub const BUNNY_DEFAULTS: (f32, f32, i32, i32, i32, i32) = (28.0, 28.0, 600, -1, 255, 10);

/// One `Gore.NewGore` internal-kick pair (`Gore.cs:1327-1328`): Y first
/// (`-= Next(10,31) * 0.1`), then X (`+= Next(-20,21) * 0.1`). The gore
/// body is the documented skip; the kicks stay on the stream.
fn gore_kicks(rng: &mut dyn DropRng) {
    let _ = rng.next_range(10, 31);
    let _ = rng.next_range(-20, 21);
}

/// `AI()`'s aiStyle-49 arm (`Projectile.cs:28243-28322`), verbatim.
pub fn bunny_step(p: &mut Projectile, rng: &mut dyn DropRng) {
    // `:28245-28255` — the corpse states.
    if p.ai[0] == -2.0 {
        p.friendly = false; // `hostile = true` (the port's single flag)
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
        return;
    }
    if p.ai[0] == -3.0 {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
        return;
    }
    // `:28256-28260` — the fuse hiss re-arm.
    if p.sound_delay == 0 {
        p.sound_delay = 3000;
        p.wants_sound = Some((2, 14, p.pos.x, p.pos.y, 0.0));
    }
    if p.ai[0] >= 0.0 {
        // `:28263-28271` — the facing latch.
        if p.vel.x > 0.0 {
            p.direction = 1;
        } else if p.vel.x < 0.0 {
            p.direction = -1;
        }
        p.sprite_direction = p.direction;
        // `:28272-28273` — the counter + the signed spin.
        p.ai[0] += 1.0;
        p.rot += p.vel.x * 0.05 + p.direction as f32 * 0.05;
        // `:28274-28278` — the 18-tick gravity.
        if p.ai[0] >= 18.0 {
            p.vel.y += 0.28;
            p.vel.x *= 0.99;
        }
        // `:28279-28282` — the DOUBLE-compare Y cap (15.9f truncates below
        // the 15.9 double, so a capped vy never re-triggers).
        if p.vel.y as f64 > 15.9 {
            p.vel.y = 15.9;
        }
        // `:28283-28286` — the first two ticks stay hidden.
        if !(p.ai[0] > 2.0) {
            return;
        }
        p.alpha = 0;
        // `:28288-28314` — the tick-3 fuse spark: the dust loops roll
        // nothing; the ONE gore draws Next(61,64) + its kicks.
        if p.ai[0] == 3.0 {
            let _ = rng.next_range(61, 64);
            gore_kicks(rng);
        }
    } else if p.ai[0] == -1.0 {
        // `:28316-28321` — the rest branch.
        p.rot = 0.0;
        p.vel.x *= 0.95;
        p.vel.y += 0.2;
    }
}

/// One `NPC.ReleaseNPC` request (`Projectile.cs:18844`): the wiring sim
/// resolves it into the NPC vec (the `wants_mark_spawn` pattern).
#[derive(Clone, Copy, Debug)]
pub struct BunnyRelease {
    /// `(vx > 0 ? 1 : 2) + 2` — the ReleaseNPC style (3 or 4).
    pub style: i32,
    pub x: i32,
    pub y: i32,
    /// `Type: 614` — the vanilla Bunny.
    pub npc_type: u16,
}

/// `HandleMovement`'s type-281 collision arm (`Projectile.cs:18833-18860`),
/// verbatim. The wiring caller invokes it when the move was blocked, with
/// the pre-collision `lastVelocity`; the free move stays the generic tail.
pub fn bunny_bounce(
    p: &mut Projectile,
    last_vel: Vec2,
    local_owner: u8,
) -> Option<BunnyRelease> {
    // `:18835-18839` — the slow/rest release gate, forced off at −2/−3.
    let mut flag15 = p.vel.x.abs() + p.vel.y.abs() < 2.0 || p.ai[0] == -1.0;
    if p.ai[0] == -2.0 || p.ai[0] == -3.0 {
        flag15 = false;
    }
    if flag15 {
        if p.owner == local_owner {
            // `:18844-18846` — the release + the corpse latch.
            let release = BunnyRelease {
                style: (if p.vel.x > 0.0 { 1 } else { 2 }) + 2,
                x: (p.pos.x + p.w * 0.5) as i32,
                y: (p.pos.y + p.h) as i32 - 4,
                npc_type: 614,
            };
            p.ai[0] = -3.0;
            p.net_update = true;
            return Some(release);
        }
        None
    } else {
        // `:18851-18858` — the 0.5× per-axis bounce.
        if p.vel.x != last_vel.x {
            p.vel.x = (0.0 - last_vel.x) * 0.5;
        }
        if p.vel.y != last_vel.y {
            p.vel.y = (0.0 - last_vel.y) * 0.5;
        }
        None
    }
}

/// The Kill hook's outcome (`Projectile.cs:75788-75801`): `damage > 0`
/// resized the box to 128×128 (applied to the slot) and ran `Damage()` —
/// the wiring caller owns the damage walk.
#[derive(Clone, Copy, Debug, Default)]
pub struct BunnyKill {
    pub damage: bool,
}

/// `Kill()`'s type-281 hook (`Projectile.cs:75749-75803`), verbatim. The
/// 20-draw gore census: 76 (2 call-site + 2 kicks), 77 (same), then 4×
/// (Next(61,64) + 2 kicks) — in that order.
pub fn bunny_kill(p: &mut Projectile, rng: &mut dyn DropRng) -> BunnyKill {
    let mut out = BunnyKill::default();
    if p.ai[0] != -3.0 {
        // `:75753` — the int-overload NPCHit thud (style 1 default).
        p.wants_sound = Some((4, 1, p.pos.x, p.pos.y, 0.0));
        // `:75754-75757` — gore 76 + 77: the call-site velocity pair
        // (X then Y, `Next(-20,21) * 0.2`), then the internal kicks.
        for _ in 0..2 {
            let _ = rng.next_range(-20, 21);
            let _ = rng.next_range(-20, 21);
            gore_kicks(rng);
        }
        // `:75758` — the explosion blast (the last cue wins the slot).
        p.wants_sound = Some((2, 14, p.pos.x, p.pos.y, 0.0));
        // `:75759-75771` — the dust loops roll nothing (skipped bodies).
        // `:75772-75787` — the 4 cardinal gibs.
        for _ in 0..4 {
            let _ = rng.next_range(61, 64);
            gore_kicks(rng);
        }
        // `:75788-75801` — the resize + the Damage intent.
        if p.damage > 0 {
            let cx = p.pos.x + p.w * 0.5;
            let cy = p.pos.y + p.h * 0.5;
            p.w = 128.0;
            p.h = 128.0;
            p.pos = Vec2::new(cx - 64.0, cy - 64.0);
            if p.ai[0] == -2.0 {
                p.friendly = false; // `hostile = true`
            }
            out.damage = true;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::UnifiedRandom;

    fn bunny() -> Projectile {
        Projectile {
            active: true,
            kind: EXPLOSIVE_BUNNY,
            owner: 0,
            pos: Vec2::new(1600.0, 800.0),
            vel: Vec2::new(6.0, -4.0),
            w: BUNNY_DEFAULTS.0,
            h: BUNNY_DEFAULTS.1,
            time_left: BUNNY_DEFAULTS.2,
            penetrate: BUNNY_DEFAULTS.3,
            max_penetrate: BUNNY_DEFAULTS.3,
            alpha: BUNNY_DEFAULTS.4,
            damage: 60,
            uses_id_static_immunity: true,
            id_static_hit_cooldown: BUNNY_DEFAULTS.5,
            ..Default::default()
        }
    }

    /// `:28245-28255` — −2 flips hostile and kills; −3 kills silently.
    #[test]
    fn w4_d5_s49_the_corpse_states_kill() {
        let mut rng = UnifiedRandom::new(49);
        let mut p = bunny();
        p.ai[0] = -2.0;
        bunny_step(&mut p, &mut rng);
        assert!(!p.active);
        assert!(!p.friendly, "−2 flips hostile");
        let mut p = bunny();
        p.ai[0] = -3.0;
        bunny_step(&mut p, &mut rng);
        assert!(!p.active);
        assert!(p.friendly, "−3 keeps friendly");
    }

    /// `:28256-28286` — the fuse re-arms 3000 once; the first two ticks
    /// keep alpha 255; tick 3 clears alpha and draws EXACTLY 3 (the gore
    /// pick + its two kicks).
    #[test]
    fn w4_d5_s49_the_fuse_spark_draws_three_on_tick_3() {
        let mut rng = UnifiedRandom::new(49);
        let mut oracle = UnifiedRandom::new(49);
        let mut p = bunny();
        for tick in 1..=3 {
            bunny_step(&mut p, &mut rng);
            assert_eq!(p.ai[0], tick as f32);
            if tick < 3 {
                assert_eq!(p.alpha, 255, "hidden @{tick}");
            }
        }
        assert_eq!(p.alpha, 0);
        assert_eq!(p.sound_delay, 3000);
        assert_eq!(rng.next_range(61, 64), oracle.next_range(61, 64));
        assert_eq!(rng.next_range(10, 31), oracle.next_range(10, 31));
        assert_eq!(rng.next_range(-20, 21), oracle.next_range(-20, 21));
        // Ticks 4+ draw nothing.
        for _ in 0..10 {
            bunny_step(&mut p, &mut rng);
        }
        assert_eq!(rng.next_range(0, 999), oracle.next_range(0, 999));
    }

    /// `:28274-28282` — the 18-tick gravity and the DOUBLE-compare cap:
    /// vy 15.9f (15.899999618…) does NOT exceed the 15.9 double, so a
    /// capped vy stays; 15.95 caps to 15.9f.
    #[test]
    fn w4_d5_s49_the_gravity_and_the_double_cap() {
        let mut rng = UnifiedRandom::new(1);
        let mut p = bunny();
        p.ai[0] = 17.0; // becomes 18 → gravity fires
        p.vel = Vec2::new(10.0, 0.0);
        bunny_step(&mut p, &mut rng);
        assert!((p.vel.y - 0.28).abs() < 1e-6);
        assert!((p.vel.x - 9.9).abs() < 1e-5);
        p.ai[0] = 20.0;
        p.vel.y = 15.9;
        bunny_step(&mut p, &mut rng);
        // 15.9f + 0.28 = 16.18 > 15.9 → capped to 15.9f.
        assert_eq!(p.vel.y, 15.9);
        p.vel.y = 15.9; // the f32 truncation is BELOW the double 15.9
        p.ai[0] = 10.0; // no gravity this tick
        bunny_step(&mut p, &mut rng);
        assert_eq!(p.vel.y, 15.9, "the capped value never re-triggers");
    }

    /// `:28316-28321` — the −1 rest: rotation 0, vx ×0.95, vy +0.2.
    #[test]
    fn w4_d5_s49_the_rest_branch_damps() {
        let mut rng = UnifiedRandom::new(2);
        let mut p = bunny();
        p.ai[0] = -1.0;
        p.rot = 2.0;
        p.vel = Vec2::new(4.0, 1.0);
        bunny_step(&mut p, &mut rng);
        assert_eq!(p.rot, 0.0);
        assert!((p.vel.x - 3.8).abs() < 1e-6);
        assert!((p.vel.y - 1.2).abs() < 1e-6);
    }

    /// `:18835-18846` — a slow bunny releases NPC 614 (style 3 right /
    /// 4 left) at (int Centre.X, int Bottom.Y − 4) and latches −3.
    #[test]
    fn w4_d5_s49_the_slow_bunny_releases() {
        let mut p = bunny();
        p.vel = Vec2::new(1.0, 0.5); // |1| + |0.5| < 2
        let r = bunny_bounce(&mut p, Vec2::new(6.0, -4.0), 0).expect("releases");
        assert_eq!(r.style, 3, "vx > 0 → 1 + 2");
        assert_eq!(r.npc_type, 614);
        assert_eq!((r.x, r.y), (1614, 824));
        assert_eq!(p.ai[0], -3.0);
        assert!(p.net_update);
        let mut p = bunny();
        p.vel = Vec2::new(-1.0, 0.5);
        let r = bunny_bounce(&mut p, Vec2::new(-6.0, -4.0), 0).expect("releases");
        assert_eq!(r.style, 4, "vx <= 0 → 2 + 2");
    }

    /// `:18849-18858` — a fast bunny bounces at 0.5× per changed axis;
    /// the −2/−3 states force the bounce even when slow.
    #[test]
    fn w4_d5_s49_the_fast_bunny_bounces_half() {
        let mut p = bunny(); // |6| + |−4| = 10 ≥ 2
        let last = Vec2::new(6.0, -4.0);
        p.vel = Vec2::new(0.0, -4.0); // the X axis was blocked
        assert!(bunny_bounce(&mut p, last, 0).is_none());
        assert_eq!(p.vel.x, -3.0, "−lastX × 0.5");
        assert_eq!(p.vel.y, -4.0, "the unchanged Y keeps");
        let mut p = bunny();
        p.ai[0] = -3.0;
        p.vel = Vec2::new(0.5, 0.5); // slow, but −3 forces the bounce
        p.vel = Vec2::new(0.0, 0.5);
        assert!(bunny_bounce(&mut p, Vec2::new(0.5, 0.5), 0).is_none());
        assert_eq!(p.vel.x, -0.25);
    }

    /// `:75749-75801` — the Kill hook draws EXACTLY 20 Next calls in
    /// order (76: 2+2, 77: 2+2, 4× gib: 1+2), resizes to 128×128 about
    /// the centre and reports the Damage intent; −3 draws NOTHING.
    #[test]
    fn w4_d5_s49_the_kill_hook_draws_twenty_and_resizes() {
        let mut rng = UnifiedRandom::new(7);
        let mut oracle = UnifiedRandom::new(7);
        let mut p = bunny();
        let out = bunny_kill(&mut p, &mut rng);
        assert!(out.damage);
        assert_eq!((p.w, p.h), (128.0, 128.0));
        assert_eq!(p.pos, Vec2::new(1614.0 - 64.0, 814.0 - 64.0));
        let want: [(i32, i32); 20] = [
            (-20, 21), (-20, 21), (10, 31), (-20, 21), // gore 76
            (-20, 21), (-20, 21), (10, 31), (-20, 21), // gore 77
            (61, 64), (10, 31), (-20, 21), // gib 1
            (61, 64), (10, 31), (-20, 21), // gib 2
            (61, 64), (10, 31), (-20, 21), // gib 3
            (61, 64), (10, 31), (-20, 21), // gib 4
        ];
        for (lo, hi) in want {
            assert_eq!(rng.next_range(lo, hi), oracle.next_range(lo, hi));
        }
        assert_eq!(rng.next_range(0, 999), oracle.next_range(0, 999), "20 only");
        // The released corpse (−3) is silent.
        let mut p = bunny();
        p.ai[0] = -3.0;
        let out = bunny_kill(&mut p, &mut rng);
        assert!(!out.damage);
        assert_eq!((p.w, p.h), (28.0, 28.0));
        assert_eq!(rng.next_range(0, 999), oracle.next_range(0, 999));
    }
}