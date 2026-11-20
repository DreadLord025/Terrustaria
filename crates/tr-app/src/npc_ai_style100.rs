//! The aiStyle-100 family — the Ancient Light (`NPC.AI()`'s
//! `else if (this.aiStyle == 100)` arm, `NPC.cs:39991-40066`). W4-d33,
//! research/25 row 100, CODE-ONLY MODE (no compiler invoked; the
//! witness tests below are written, not run).
//!
//! ## Census
//!
//! The family has exactly ONE vanilla member: the AncientLight 522
//! (`NPCID.AncientLight`, `SetDefaults` row `NPC.cs:15780-15794`).
//! The census of `this.aiStyle = 100;` across the decompiled tree
//! returns exactly two hits: `NPC.cs:15784` (the `type == 522`
//! branch) and `Projectile.cs:6098` (projectile space, out of NPC
//! scope). The port's roster has no `NpcKind::AncientLight` — the
//! module lands UNWIRED (the `npc_ai_style73.rs` precedent), keyed on
//! the vanilla `npc_type` until the roster wave.
//!
//! ## The machine
//!
//! The Solar Goop's sibling with a drift tail. The landing latch
//! (`:39993-39999`): `velocity.Y == 0 && ai[0] >= 0` arms the dissolve
//! (`ai[0] = -1`, `ai[1] = 0`, `netUpdate`) and RETURNS. The dissolve
//! (`:40000-40011`): velocity zeroed, `oldPosition` snap, `ai[1]++`;
//! at 5 the `HitEffect(0, 9999.0)` + `active = false` pair, RETURN.
//! The tilt (`:40012`): rotation = `velocity.ToRotation() - π/2`.
//!
//! The 522-only body (`:40013-40065`): the first tick
//! (`localAI[0] == 0`) latches, RESTORES the launch velocity from
//! `ai[2]`/`ai[3]` (the cultist's aim slots) and spawns the 13-dust
//! type-261 burst (alpha 90, scale 2.5, `noGravity`, `fadeIn = 1`,
//! ×4, `noLight`). EVERY tick then runs the 2-iteration trail:
//! `Next(10 - (int)Math.Min(7f, velocity.Length())) < 1` gates a
//! type-261 dust (×0.2, `fadeIn = 0.4`), `Next(6) == 0` picks the ×5
//! `noLight` variant over the `DirectionFrom` redirect. The drift
//! tail (`:40050-40065`, `ai[0] >= 0` only): `ai[0]++`; past 60 the
//! velocity spins by `ai[1]` radians EVERY tick (`RotatedBy`); past
//! 120 it decays ×0.98; below 0.2 length it snaps to zero (which
//! re-arms the landing latch next tick).
//!
//! ## RNG order (by construction)
//!
//! Per body tick: 2 × `Next(10 - min(7, len) as int)` gates, each
//! passing gate adding the `NewDust` internals then one `Next(6)`.
//! The first tick's 13-burst adds 13 `NewDust` internals BEFORE the
//! trail. The dissolve's `HitEffect(0, 9999.0)` runs the type-522 arm
//! (`NPC.cs:83024-83038`): ONE `RotatedByRandom(π)` pair (two
//! `NextDouble`, `Utils.cs:1697-1700`) then 24 ring dusts — each
//! `NewDust(Center, 0, 0, 66, …, Color.Transparent)` internals, then
//! the velocity `(0,-3)-spin.RotatedBy(2π·i/24) × (1.05, 1) ×
//! (0.8 + NextFloat()×0.4) × 2`, `Color.SkyBlue`, `noGravity`,
//! `scale += 0.5 + NextFloat()` — TWO `NextFloat` per dust, in that
//! order. `HitEffect`'s other gates draw nothing for 522 (no town
//! NPC, no daybreak, no matching type arm; the generic else-chain
//! has no 522 row). Both sounds are null (`:15788-15789`).
//!
//! ## FindFrame
//!
//! `NPC.FindFrame` (`NPC.cs:66684`) has NO case 522 — the frame never
//! advances (`Main.npcFrameCount[522] = 1`). Nothing to port.
#![allow(dead_code)] // wiring deferred — the kind + dispatch arm land with the roster wave

use crate::npc::Npc;
use crate::npc_ai_style37::rotated_by;
use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// The type-522 `SetDefaults` row (`NPC.cs:15780-15794`), verbatim.
pub struct AncientLightDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// Both `HitSound` and `DeathSound` are null (`:15788-15789`).
    pub hit_sound: Option<(i32, i32)>,
    pub death_sound: Option<(i32, i32)>,
    pub knock_back_resist: f32,
    pub no_gravity: bool,
    pub no_tile_collide: bool,
    pub alpha: u8,
}

/// `NPC.cs:15780-15794` — `else if (this.type == 522)`.
pub const ANCIENT_LIGHT: AncientLightDefaults = AncientLightDefaults {
    width: 26,
    height: 26,
    ai_style: 100,
    damage: 120,
    defense: 0,
    life_max: 400,
    hit_sound: None,
    death_sound: None,
    knock_back_resist: 0.0,
    no_gravity: true,
    no_tile_collide: true,
    alpha: 0,
};

/// `Main.npcFrameCount[522]` = 1 (the 520-529 row).
pub const ANCIENT_LIGHT_FRAME_COUNT: u32 = 1;

/// `Color.SkyBlue` (XNA) — `(135, 206, 235, 255)`, the HitEffect
/// ring's colour (`NPC.cs:83034`).
const SKY_BLUE: Color = Color { r: 135, g: 206, b: 235, a: 255 };

/// The per-tick environment (the `Style99Env` shape).
pub struct Style100Env<'a> {
    pub world: &'a World,
    pub dust: &'a mut DustPool,
    pub dust_ctx: &'a SpawnCtx,
    pub rng: &'a mut UnifiedRandom,
}

/// The arm's per-tick side effects beyond the NPC state.
#[derive(Default, Debug, PartialEq)]
pub struct Style100Outcome {
    /// The dissolve expiry (`:40007-40008`): the `HitEffect(0, 9999.0)`
    /// already ran INSIDE the step (the 24-dust ring); the caller owes
    /// only the `active = false` deactivation.
    pub died: bool,
}

/// `(float)Math.Sqrt(x*x + y*y)` — XNA `Vector2.Length()` (the f32
/// sum widened to f64 for the root, the style86 `sqrt_f32` shape).
fn sqrt_f32(x: f32, y: f32) -> f32 {
    ((x * x + y * y) as f64).sqrt() as f32
}

/// XNA `Vector2.Normalize()` — UNGUARDED (the style86 precedent).
fn normalize_xna(x: f32, y: f32) -> (f32, f32) {
    let num = 1.0f32 / ((x * x + y * y) as f64).sqrt() as f32;
    (x * num, y * num)
}

/// `Utils.RotatedByRandom(maxRadians)` (`Utils.cs:1697-1700`): TWO
/// `NextDouble` draws, `+first×max − second×max` (the bow_launch.rs
/// port's order).
fn rotated_by_random(v: Vec2, max_radians: f64, rng: &mut UnifiedRandom) -> Vec2 {
    let a = rng.next_double() * max_radians;
    let b = rng.next_double() * max_radians;
    let (x, y) = rotated_by(v.x, v.y, a - b, 0.0, 0.0);
    Vec2::new(x, y)
}

/// The `HitEffect(0, 9999.0)` type-522 arm (`NPC.cs:83024-83038`),
/// verbatim: the one `RotatedByRandom(π)` spin of `(0, -3)`, then the
/// 24-dust type-66 ring at the centre — velocity
/// `spin.RotatedBy(2π·i/24) × (1.05, 1) × (0.8 + NextFloat()×0.4) × 2`,
/// SkyBlue, `noGravity`, `scale += 0.5 + NextFloat()`.
pub fn light_hit_effect(npc: &Npc, env: &mut Style100Env<'_>) {
    // `:83026` — the spin pair (two NextDouble).
    let spin = rotated_by_random(Vec2::new(0.0, -3.0), std::f64::consts::PI, env.rng);
    let (cx, cy) = npc.center();
    // `:83029-83037` — the 24-iteration ring.
    for i in 0..24 {
        let idx = env.dust.new_dust(
            env.rng,
            env.dust_ctx,
            env.world,
            Vec2::new(cx, cy),
            0,
            0,
            66,
            0.0,
            0.0,
            0,
            Color::TRANSPARENT,
            1.0,
        );
        {
            let d = env.dust.slot_mut(idx);
            d.position = Vec2::new(cx, cy);
            // `:83033` — the C# computes the angle in FLOAT
            // (`(float)Math.PI * 2f * num127 / num126`) then widens.
            let angle = (std::f32::consts::PI * 2.0 * i as f32 / 24.0) as f64;
            let (rx, ry) = rotated_by(spin.x, spin.y, angle, 0.0, 0.0);
            let speed = (0.8 + env.rng.next_float() * 0.4) * 2.0;
            d.velocity = Vec2::new(rx * 1.05 * speed, ry * 1.0 * speed);
            d.color = SKY_BLUE;
            d.no_gravity = true;
            d.scale += 0.5 + env.rng.next_float();
        }
    }
}

/// One fixed step of the aiStyle-100 arm (`NPC.cs:39991-40066`), in
/// C# order. `npc_type` is the vanilla id (522) until the roster kind
/// lands. The caller owns the raw movement integration after `AI()`
/// (the row's `noGravity`/`noTileCollide` are both true) and drains
/// [`Style100Outcome`].
pub fn style100_step(
    npc: &mut Npc,
    npc_type: i32,
    env: &mut Style100Env<'_>,
) -> Style100Outcome {
    let mut out = Style100Outcome::default();
    // `:39993-39999` — the landing latch (ai[0] >= 0, NOT == 0).
    if npc.vy == 0.0 && npc.ai[0] >= 0.0 {
        npc.ai[0] = -1.0;
        npc.ai[1] = 0.0;
        npc.net_update = true;
        return out;
    }
    // `:40000-40011` — the 5-tick dissolve.
    if npc.ai[0] == -1.0 {
        npc.vx = 0.0;
        npc.vy = 0.0;
        npc.x = npc.old_x;
        npc.y = npc.old_y;
        npc.ai[1] += 1.0;
        if npc.ai[1] >= 5.0 {
            light_hit_effect(npc, env);
            out.died = true;
        }
        return out;
    }
    // `:40012` — the velocity-aligned tilt.
    npc.rotation = npc.vy.atan2(npc.vx) - std::f32::consts::FRAC_PI_2;
    // `:40013-40016` — only the 522 body.
    if npc_type != 522 {
        return out;
    }
    let (w, h) = npc.size();
    // `:40017-40030` — the first-tick launch restore + 13-dust burst.
    if npc.local_ai[0] == 0.0 {
        npc.local_ai[0] = 1.0;
        npc.vx = npc.ai[2];
        npc.vy = npc.ai[3];
        for _ in 0..13 {
            let idx = env.dust.new_dust(
                env.rng,
                env.dust_ctx,
                env.world,
                Vec2::new(npc.x, npc.y),
                w as i32,
                h as i32,
                261,
                npc.vx * 0.5,
                npc.vy * 0.5,
                90,
                Color::default(),
                2.5,
            );
            {
                let d = env.dust.slot_mut(idx);
                d.no_gravity = true;
                d.fade_in = 1.0;
                d.velocity *= 4.0;
                d.no_light = true;
            }
        }
    }
    // `:40031-40049` — the every-tick 2-iteration trail. The gate's
    // bound reads the LIVE velocity (post-restore on the first tick).
    for _ in 0..2 {
        let bound = 10 - 7.0f32.min(sqrt_f32(npc.vx, npc.vy)) as i32;
        if env.rng.next_max(bound) < 1 {
            let idx = env.dust.new_dust(
                env.rng,
                env.dust_ctx,
                env.world,
                Vec2::new(npc.x, npc.y),
                w as i32,
                h as i32,
                261,
                npc.vx * 0.5,
                npc.vy * 0.5,
                90,
                Color::default(),
                2.5,
            );
            {
                let d = env.dust.slot_mut(idx);
                d.no_gravity = true;
                d.velocity *= 0.2;
                d.fade_in = 0.4;
                if env.rng.next_max(6) == 0 {
                    // `:40041-40042` — the fast variant.
                    d.velocity *= 5.0;
                    d.no_light = true;
                } else {
                    // `:40046` — DirectionFrom(dust.position) * Length().
                    let len = sqrt_f32(d.velocity.x, d.velocity.y);
                    let (cx, cy) = npc.center();
                    let (nx, ny) = normalize_xna(cx - d.position.x, cy - d.position.y);
                    d.velocity = Vec2::new(nx * len, ny * len);
                }
            }
        }
    }
    // `:40050-40065` — the drift tail (ai[0] >= 0 only).
    if npc.ai[0] >= 0.0 {
        npc.ai[0] += 1.0;
        if npc.ai[0] > 60.0 {
            let (rx, ry) = rotated_by(npc.vx, npc.vy, npc.ai[1] as f64, 0.0, 0.0);
            npc.vx = rx;
            npc.vy = ry;
        }
        if npc.ai[0] > 120.0 {
            npc.vx *= 0.98;
            npc.vy *= 0.98;
        }
        if sqrt_f32(npc.vx, npc.vy) < 0.2 {
            npc.vx = 0.0;
            npc.vy = 0.0;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// A 522-shaped NPC (the style73 placeholder convention).
    fn light(x: f32, y: f32) -> Npc {
        let d = &ANCIENT_LIGHT;
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
        n
    }

    fn air() -> World {
        World::new("t".into(), 400, 300)
    }

    fn spawn_ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(1920.0, 1080.0),
            max_dust_to_draw: 6000,
        }
    }

    fn menu_ctx() -> SpawnCtx {
        SpawnCtx { game_menu: true, ..spawn_ctx() }
    }

    /// `NPC.cs:15780-15794` — the SetDefaults row, field by field.
    #[test]
    fn w4_d33_s100_the_defaults_row() {
        let d = &ANCIENT_LIGHT;
        assert_eq!((d.width, d.height), (26, 26));
        assert_eq!(d.ai_style, 100);
        assert_eq!((d.damage, d.defense, d.life_max), (120, 0, 400));
        assert_eq!(d.hit_sound, None);
        assert_eq!(d.death_sound, None);
        assert!(d.no_gravity);
        assert!(d.no_tile_collide, "the 522 ghosts through tiles");
        assert_eq!(ANCIENT_LIGHT_FRAME_COUNT, 1);
    }

    /// `:39993-39999` — the latch fires on ai[0] >= 0 (a POSITIVE
    /// drift counter also latches, unlike the 99's == 0).
    #[test]
    fn w4_d33_s100_the_latch_accepts_positive_counters() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(100);
        let mut n = light(100.0, 100.0);
        n.ai[0] = 40.0; // mid-drift
        n.vy = 0.0;
        let mut env = Style100Env { world: &w, dust: &mut pool, dust_ctx: &c, rng: &mut rng };
        let out = style100_step(&mut n, 522, &mut env);
        assert_eq!(n.ai[0], -1.0);
        assert_eq!(n.ai[1], 0.0);
        assert!(n.net_update);
        assert!(!out.died);
    }

    /// `:40000-40011` — the dissolve snaps to oldPosition and dies at
    /// 5; under the menu ctx the ring refuses before drawing, but the
    /// spin pair is EXPLICIT — two NextDouble always fire.
    #[test]
    fn w4_d33_s100_the_dissolve_dies_at_five_with_the_spin_pair() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(100);
        let mut oracle = UnifiedRandom::new(100);
        let mut n = light(100.0, 100.0);
        n.old_x = 90.0;
        n.old_y = 80.0;
        n.ai[0] = -1.0;
        let mut env = Style100Env { world: &w, dust: &mut pool, dust_ctx: &c, rng: &mut rng };
        for tick in 1..=5 {
            let out = style100_step(&mut n, 522, &mut env);
            assert_eq!((n.x, n.y), (90.0, 80.0));
            assert_eq!(out.died, tick == 5);
        }
        // The death tick: the spin pair + 24 x (NextFloat, NextFloat).
        oracle.next_double();
        oracle.next_double();
        for _ in 0..24 {
            oracle.next_float();
            oracle.next_float();
        }
        assert_eq!(rng.peek(), oracle.peek(), "the ring's explicit draws");
    }

    /// `:40017-40021` — the first body tick restores the launch
    /// velocity from ai[2]/ai[3] BEFORE the trail gates read it.
    #[test]
    fn w4_d33_s100_the_first_tick_restores_the_launch_velocity() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(100);
        let mut n = light(100.0, 100.0);
        n.ai[0] = -2.0; // past the latch, not the dissolve
        n.ai[2] = 3.0;
        n.ai[3] = -4.0;
        let mut env = Style100Env { world: &w, dust: &mut pool, dust_ctx: &c, rng: &mut rng };
        let _ = style100_step(&mut n, 522, &mut env);
        assert_eq!(n.local_ai[0], 1.0);
        assert_eq!((n.vx, n.vy), (3.0, -4.0), "the ai[2]/ai[3] restore");
        // len = 5 -> bound 10 - 5 = 5; two gates, each passing adds Next(6).
        let mut oracle = UnifiedRandom::new(100);
        for _ in 0..2 {
            if oracle.next_max(5) < 1 {
                oracle.next_max(6);
            }
        }
        assert_eq!(rng.peek(), oracle.peek(), "the trail reads the restored velocity");
    }

    /// `:40050-40065` — the drift tail: the counter bumps, the spin
    /// starts past 60, the decay past 120, the 0.2 snap re-arms the
    /// latch.
    #[test]
    fn w4_d33_s100_the_drift_tail_spins_decays_and_snaps() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(100);
        let mut n = light(100.0, 100.0);
        n.ai[0] = 60.0;
        n.ai[1] = std::f32::consts::FRAC_PI_2;
        n.local_ai[0] = 1.0; // burst done
        // vy must be NON-zero: vy == 0 with ai[0] >= 0 is the landing
        // latch (`:39993`), which would fire before the drift tail.
        n.vx = 0.0;
        n.vy = 4.0;
        let mut env = Style100Env { world: &w, dust: &mut pool, dust_ctx: &c, rng: &mut rng };
        let _ = style100_step(&mut n, 522, &mut env);
        assert_eq!(n.ai[0], 61.0);
        // RotatedBy(pi/2): (0,4) -> (-4,0) (the f64 cos(pi/2) ~ 6e-17).
        assert!((n.vx + 4.0).abs() < 1e-5);
        assert!(n.vy.abs() < 1e-5);
        // The decay: ai[0] = 120 -> 121 fires x0.98.
        n.ai[0] = 120.0;
        n.ai[1] = 0.0;
        n.vx = 1.0;
        n.vy = 0.5;
        let _ = style100_step(&mut n, 522, &mut env);
        assert_eq!(n.vx, 0.98);
        // The snap: len < 0.2 zeroes both, re-arming the latch.
        n.vx = 0.1;
        n.vy = 0.1;
        let _ = style100_step(&mut n, 522, &mut env);
        assert_eq!((n.vx, n.vy), (0.0, 0.0));
        let _ = style100_step(&mut n, 522, &mut env);
        assert_eq!(n.ai[0], -1.0, "the zero velocity re-latches");
    }

    /// The full-pool stream parity: the first body tick on the REAL
    /// dust ctx must match a cloned twin step bit-for-bit (the
    /// style86 twin-pool precedent).
    #[test]
    fn w4_d33_s100_the_full_pool_first_tick_is_twin_stable() {
        let w = air();
        let c = spawn_ctx();
        let mut a = light(600.0, 600.0);
        a.ai[0] = -2.0;
        a.ai[2] = 2.0;
        a.ai[3] = -3.0;
        let mut b = a.clone();
        let mut pool_a = DustPool::new();
        let mut pool_b = DustPool::new();
        let mut rng_a = UnifiedRandom::new(522);
        let mut rng_b = UnifiedRandom::new(522);
        for tick in 0..30 {
            let mut env_a = Style100Env { world: &w, dust: &mut pool_a, dust_ctx: &c, rng: &mut rng_a };
            let _ = style100_step(&mut a, 522, &mut env_a);
            let mut env_b = Style100Env { world: &w, dust: &mut pool_b, dust_ctx: &c, rng: &mut rng_b };
            let _ = style100_step(&mut b, 522, &mut env_b);
            assert_eq!(rng_a.peek(), rng_b.peek(), "stream @{tick}");
            assert_eq!(a.vx.to_bits(), b.vx.to_bits(), "vx @{tick}");
            assert_eq!(a.ai[0].to_bits(), b.ai[0].to_bits(), "ai0 @{tick}");
        }
    }
}
