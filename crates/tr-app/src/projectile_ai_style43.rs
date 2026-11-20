//! W4-d5 (aiStyle-43) — the CrystalLeafShot (227), `Projectile.AI`'s
//! `else if (this.aiStyle == 43)` block (`Projectile.cs:27860-27892`) plus
//! its `Kill()` hook (`:74524-74533`), ported 1:1 as a PURE machine
//! (code-only mode: no compiler/tests were run on this branch; the witness
//! tests below are written, not executed).
//!
//! ## Census (research/26d row 43)
//!
//! ONE vanilla member: CrystalLeafShot 227 (`ProjectileID.cs:1008`,
//! `SetDefaults` `Projectile.cs:3124-3136`: `netImportant`,
//! `tileCollide = false`, 14×14, friendly, `penetrate = 1`,
//! `timeLeft = 180`, `extraUpdates = 180`, `hide = true`). The port has no
//! `proj_data` row, so this module lands UNWIRED (the `npc_ai_style25`
//! precedent). The 226 leaf's spawn request (`projectile_ai_style42`) is
//! the producer; the wiring round adds the row + the dispatch arm.
//!
//! ## The machine (verbatim)
//!
//! * `:27862` — `rotation = atan2(velocity.Y, velocity.X) + 3.14f`.
//! * `:27863-27875` — the one-shot cue: `soundDelay == 0 && type == 227`
//!   latches `soundDelay = -1`, plays sound 6 (the INT overload
//!   `PlaySound(6, (int)X, (int)Y)` — default style 1) and spawns 8× dust
//!   157, each drawing ONE `Main.rand.NextFloat()` at the call site
//!   (`:27873`, `velocity += base.velocity * NextFloat()`).
//! * `:27876-27878` — the hue chain: `num320 = (((1 − timeLeft/180) * −6 *
//!   0.85 + 0.33) % 1 + 1) % 1`; `Lerp(hslToRgb(num320, 1, 0.5), Red,
//!   Remap(num320, 0.33, 0.7, 0, 1))`, then `Lerp(value, Lerp(LimeGreen,
//!   Gold, 0.3), value.R / 255f)`.
//! * `:27879-27891` — `if (frameCounter++ >= 1)`: reset to 0 and (local
//!   owner only) request the ChlorophyteLeafCrystalShot particle with
//!   `UniqueInfoPiece = (byte)(rgbToHsl(value).X * 255f)`. The post-
//!   increment fires every OTHER tick (old value ≥ 1).
//!
//! ## Skips (documented)
//!
//! The dust BODIES (no dust layer in the pool) — the explicit call-site
//! `NextFloat` draws STAY (the `projectile_ai_misc` convention). The
//! particle-orchestrator request is render state (no particle layer) —
//! the hue chain is ported as [`leaf_shot_hue`] so the wiring draw can
//! read it. The id-6 sound rides `wants_sound` as `(6, 1, …)`; the pool
//! drain's id-6 façade is a wiring follow-up (today it plays only ids 0/2,
//! `world_sim/projectiles.rs:460-473`) — the intent + the −1 latch are the
//! observable state.
//!
//! ## RNG contract
//!
//! The AI arm: 8 `NextFloat` on the first tick only (the soundDelay latch).
//! The Kill hook: 15 `NextFloat` (the 15× dust 157's `oldVelocity` blend,
//! `:74531`) + the sound-6 int play (no draw at the call site). Both on the
//! `UpdateProjectiles` stream, in the C#'s order.
#![allow(dead_code)] // UNWIRED (W4-d5): the 227 row + the dispatch arm are the wiring step

use crate::projectile::{Projectile, get_lerp_value};
use tr_core::loot::DropRng;

/// `ProjectileID.CrystalLeafShot` (`ProjectileID.cs:1008`).
pub const CRYSTAL_LEAF_SHOT: u16 = 227;

/// The `SetDefaults` row (`Projectile.cs:3124-3136`):
/// (width, height, timeLeft, penetrate, extraUpdates).
pub const SHOT_DEFAULTS: (f32, f32, i32, i32, u32) = (14.0, 14.0, 180, 1, 180);

/// XNA `Color.Lerp` (`Color.cs`): per-channel float lerp, `(byte)`
/// truncation — NOT banker's rounding (the `ui::color` module doc).
fn lerp_byte(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t) as u8
}

fn lerp_rgb(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    [lerp_byte(a[0], b[0], t), lerp_byte(a[1], b[1], t), lerp_byte(a[2], b[2], t)]
}

/// The hue chain (`Projectile.cs:27876-27878` + `:27888`): the shot's
/// rainbow colour and its `UniqueInfoPiece` byte. `Main.hslToRgb` /
/// `Main.rgbToHsl` are the ported `ui::color` pair (banker's-rounded bytes
/// out, float hue back); `Utils.Remap` is the clamped `GetLerpValue` ramp.
pub fn leaf_shot_hue(time_left: i32) -> (u8, [u8; 3]) {
    // `:27876` — the double-mod wrap keeps the hue in [0, 1).
    let num320 = (((1.0 - time_left as f32 / 180.0) * -6.0 * 0.85 + 0.33) % 1.0 + 1.0) % 1.0;
    // `:27877` — hsl → the Red lerp over the 0.33..0.7 remap.
    let base = crate::ui::color::hsl_to_rgb(num320 as f64, 1.0, 0.5, 255);
    let red_t = get_lerp_value(0.33, 0.7, num320);
    let mut value = lerp_rgb([base[0], base[1], base[2]], [255, 0, 0], red_t);
    // `:27878` — the LimeGreen→Gold 0.3 blend, weighted by value.R / 255.
    let lime_gold = lerp_rgb([50, 205, 50], [255, 215, 0], 0.3);
    value = lerp_rgb(value, lime_gold, value[0] as f32 / 255.0);
    // `:27888` — `(byte)(Main.rgbToHsl(value).X * 255f)`.
    let (h, _, _) = crate::ui::color::rgb_to_hsl([value[0], value[1], value[2], 255]);
    ((h as f32 * 255.0) as u8, value)
}

/// `AI()`'s aiStyle-43 arm (`Projectile.cs:27860-27892`), verbatim.
/// `local_owner` is `Main.myPlayer`; the particle request itself is the
/// documented render skip — the latch + the hue stay.
pub fn leaf_shot_step(p: &mut Projectile, rng: &mut dyn DropRng, local_owner: u8) {
    // `:27862` — the velocity-facing spin (+π: the sheet faces down-left).
    p.rot = p.vel.y.atan2(p.vel.x) + 3.14;
    // `:27863-27875` — the one-shot cue (the −1 latch never re-arms: the
    // generic decrement only runs while positive, `:16768-16771`).
    if p.sound_delay == 0 && p.kind == CRYSTAL_LEAF_SHOT {
        p.sound_delay = -1;
        // `SoundEngine.PlaySound(6, (int)X, (int)Y)` — the int overload's
        // default style 1 (the drain's id-6 façade is the wiring follow-up).
        p.wants_sound = Some((6, 1, p.pos.x, p.pos.y, 0.0));
        for _ in 0..8 {
            // `:27873` — the dust 157's call-site NextFloat; the body is
            // the documented dust skip.
            let _ = rng.next_float();
        }
    }
    // `:27879-27891` — the post-increment latch: fires every other tick.
    let old = p.frame_counter;
    p.frame_counter += 1;
    if old >= 1 {
        p.frame_counter = 0;
        if p.owner == local_owner {
            // The ChlorophyteLeafCrystalShot particle request — render
            // state, skipped; `leaf_shot_hue(p.time_left)` is its payload.
            let _ = leaf_shot_hue(p.time_left);
        }
    }
}

/// `Kill()`'s type-227 hook (`Projectile.cs:74524-74533`): the sound-6 int
/// play + 15× dust 157, each with ONE call-site `NextFloat`
/// (`velocity += oldVelocity * NextFloat()`). The dust bodies are the
/// documented skip; the draws stay, in order.
pub fn leaf_shot_kill(p: &mut Projectile, rng: &mut dyn DropRng) {
    p.wants_sound = Some((6, 1, p.pos.x, p.pos.y, 0.0));
    for _ in 0..15 {
        let _ = rng.next_float();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;
    use tr_core::UnifiedRandom;

    fn shot(vel: Vec2) -> Projectile {
        Projectile {
            active: true,
            kind: CRYSTAL_LEAF_SHOT,
            owner: 0,
            vel,
            w: SHOT_DEFAULTS.0,
            h: SHOT_DEFAULTS.1,
            time_left: SHOT_DEFAULTS.2,
            penetrate: SHOT_DEFAULTS.3,
            extra_updates: SHOT_DEFAULTS.4,
            tile_collide: false,
            ..Default::default()
        }
    }

    /// `:27862` — the rotation faces the velocity + π.
    #[test]
    fn w4_d5_s43_the_rotation_faces_the_velocity_plus_pi() {
        let mut rng = UnifiedRandom::new(43);
        let mut p = shot(Vec2::new(12.0, 0.0));
        leaf_shot_step(&mut p, &mut rng, 0);
        assert!((p.rot - 3.14).abs() < 1e-5, "atan2(0,12)=0 → 3.14");
        let mut p = shot(Vec2::new(0.0, -12.0));
        leaf_shot_step(&mut p, &mut rng, 0);
        assert!((p.rot - (3.14 - std::f32::consts::FRAC_PI_2)).abs() < 1e-5);
    }

    /// `:27863-27875` — the first tick latches soundDelay −1, queues the
    /// id-6 cue and draws EXACTLY 8 NextFloat; later ticks draw nothing.
    #[test]
    fn w4_d5_s43_the_first_tick_draws_eight_floats_once() {
        let mut rng = UnifiedRandom::new(43);
        let mut oracle = UnifiedRandom::new(43);
        let mut p = shot(Vec2::new(12.0, 0.0));
        leaf_shot_step(&mut p, &mut rng, 0);
        assert_eq!(p.sound_delay, -1);
        assert_eq!(p.wants_sound, Some((6, 1, p.pos.x, p.pos.y, 0.0)));
        for _ in 0..8 {
            assert_eq!(rng.next_float().to_bits(), oracle.next_float().to_bits());
        }
        for _ in 0..20 {
            leaf_shot_step(&mut p, &mut rng, 0);
        }
        assert_eq!(rng.next_float().to_bits(), oracle.next_float().to_bits(), "no further draws");
    }

    /// `:27879` — the post-increment fires every OTHER tick: counter
    /// 0→1 (no fire), 1→0 (fire), 0→1, …
    #[test]
    fn w4_d5_s43_the_frame_latch_fires_every_other_tick() {
        let mut rng = UnifiedRandom::new(1);
        let mut p = shot(Vec2::new(1.0, 0.0));
        p.sound_delay = -1; // silence the cue's draws
        let mut fires = 0;
        for tick in 0..10 {
            let old = p.frame_counter;
            leaf_shot_step(&mut p, &mut rng, 0);
            if old >= 1 {
                fires += 1;
                assert_eq!(p.frame_counter, 0, "fire resets @{tick}");
            } else {
                assert_eq!(p.frame_counter, 1, "no fire increments @{tick}");
            }
        }
        assert_eq!(fires, 5);
    }

    /// `:27876-27878` — the hue at spawn (timeLeft 180): num320 = 0.33,
    /// the Red remap is 0, so the colour is hslToRgb(0.33, 1, 0.5) lerped
    /// toward LimeGreen/Gold by its own R/255. The piece byte round-trips
    /// through rgbToHsl.
    #[test]
    fn w4_d5_s43_the_hue_chain_is_byte_exact_at_spawn() {
        let (piece, value) = leaf_shot_hue(180);
        // num320 = ((1-1)*-5.1+0.33)%1 = 0.33 (f32: 0.33).
        let base = crate::ui::color::hsl_to_rgb(0.33f32 as f64, 1.0, 0.5, 255);
        let lg = lerp_rgb([50, 205, 50], [255, 215, 0], 0.3);
        let want = lerp_rgb([base[0], base[1], base[2]], lg, base[0] as f32 / 255.0);
        assert_eq!(value, want);
        let (h, _, _) = crate::ui::color::rgb_to_hsl([want[0], want[1], want[2], 255]);
        assert_eq!(piece, (h as f32 * 255.0) as u8);
    }

    /// The Kill hook draws EXACTLY 15 NextFloat and queues the id-6 cue.
    #[test]
    fn w4_d5_s43_the_kill_hook_draws_fifteen_floats() {
        let mut rng = UnifiedRandom::new(5);
        let mut oracle = UnifiedRandom::new(5);
        let mut p = shot(Vec2::new(3.0, 4.0));
        leaf_shot_kill(&mut p, &mut rng);
        assert_eq!(p.wants_sound, Some((6, 1, p.pos.x, p.pos.y, 0.0)));
        for _ in 0..15 {
            assert_eq!(rng.next_float().to_bits(), oracle.next_float().to_bits());
        }
        assert_eq!(rng.next_float().to_bits(), oracle.next_float().to_bits(), "15 only");
    }
}