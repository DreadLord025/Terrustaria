//! W4-d5 (aiStyle-43) — the CrystalLeafShot (227), `Projectile.AI`'s
//! `else if (this.aiStyle == 43)` block (`Projectile.cs:27860-27892`) plus
//! its `Kill()` hook (`:74524-74533`), ported 1:1 as a PURE machine
//! (code-only mode: no compiler/tests were run on this branch; the witness
//! tests below are written, not executed).
//!
//! ## Census (research/156 row 43 — was 26d, registry §3)
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
//!   157 through [`crystal_leaf_burst_227`] — the W4-e3 STAGED emitter
//!   ACTIVATED here (research/231 row 1: the 8 kept call-site `NextFloat`
//!   draws are REPLACED by the emitter — the g63 body-skip supersede
//!   convention). Per dust: `NewDust` internals FIRST, then `noGravity`,
//!   `velocity *= 3`, `scale = 1.5`, then the POST-spawn
//!   `velocity += base.velocity * NextFloat()` (`:27870-27873`).
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
//! The particle-orchestrator request is render state (no particle layer) —
//! the hue chain is ported as [`leaf_shot_hue`] so the wiring draw can
//! read it. The id-6 sound rides `wants_sound` as `(6, 1, …)`; the pool
//! drain's id-6 façade is a wiring follow-up (today it plays only ids 0/2,
//! `world_sim/projectiles.rs:460-473`) — the intent + the −1 latch are the
//! observable state. The Kill hook's 15× dust-157 bodies + their 15
//! `NextFloat` draws are W4-g65's (`projectile_death_dust.rs`'s 227 drain
//! arm, `:74524-74534`, snapshot→drain on the live stream) — the g65
//! body-lane supersede; `leaf_shot_kill` keeps only the id-6 cue intent.
//!
//! ## RNG contract
//!
//! The AI arm: per dust, the `NewDust` internals then ONE POST-spawn
//! `NextFloat` (`:27873`), ×8, first tick only (the soundDelay latch) —
//! the e3 emitter's draw order, no longer a bare 8-float census. The Kill
//! hook draws NOTHING at the machine (the cue is draw-free); its 15
//! `NextFloat` + internals ride the g65 death drain. Both on the
//! `UpdateProjectiles` stream, in the C#'s order.
#![allow(dead_code)] // UNWIRED (W4-d5): the 227 row + the dispatch arm are the wiring step

use crate::projectile::{Projectile, get_lerp_value};
use tr_core::loot::DropRng;
use tr_core::particles::{Color, DustPool, DustRng, SpawnCtx};
use tr_core::World;

/// `ProjectileID.CrystalLeafShot` (`ProjectileID.cs:1008`).
pub const CRYSTAL_LEAF_SHOT: u16 = 227;

/// The `SetDefaults` row (`Projectile.cs:3124-3136`):
/// (width, height, timeLeft, penetrate, extraUpdates).
pub const SHOT_DEFAULTS: (f32, f32, i32, i32, u32) = (14.0, 14.0, 180, 1, 180);

/// The pool's [`DustRng`] over the pass's [`DropRng`] stream (the W4-e3
/// `ProjDustRng` bundle, `projectile_dust.rs:47-59` — merger keep-one):
/// `Next(max)` is `Next(0, max)` (`UnifiedRandom.cs:136-139`).
struct LeafDustRng<'a>(&'a mut dyn DropRng);

impl DustRng for LeafDustRng<'_> {
    fn next_max(&mut self, max: i32) -> i32 {
        self.0.next_range(0, max)
    }
    fn next_range(&mut self, min: i32, max: i32) -> i32 {
        self.0.next_range(min, max)
    }
    fn next_float(&mut self) -> f32 {
        self.0.next_float()
    }
}

/// `:27867-27874` — the 227's first-tick 8× dust-157 burst: the W4-e3
/// STAGED emitter `style43_crystal_leaf_burst_227`
/// (`projectile_dust.rs:1707`, research/231 row 1) ACTIVATED — body
/// verbatim (merger keep-one with e3's copy; this lane's machine is its
/// only caller). Per dust: internals, `noGravity`, vel×3, scale 1.5, then
/// the POST-spawn `vel += base.vel * NextFloat()`. Sentinel behaviour is
/// the pool's (post-writes land on slot 6000, `Dust.cs:79`). The caller
/// owns the soundDelay latch.
pub(crate) fn crystal_leaf_burst_227(
    p: &Projectile,
    world: &World,
    dust: &mut DustPool,
    rng: &mut dyn DropRng,
    ctx: &SpawnCtx,
) {
    for _ in 0..8 {
        let idx = dust.new_dust(
            &mut LeafDustRng(rng),
            ctx,
            world,
            p.pos,
            p.w as i32,
            p.h as i32,
            157,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= 3.0;
        d.scale = 1.5;
        d.velocity += p.vel * rng.next_float();
    }
}

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
/// documented render skip — the latch + the hue stay. The dust burst is
/// the activated e3 emitter ([`crystal_leaf_burst_227`]).
#[allow(clippy::too_many_arguments)]
pub fn leaf_shot_step(
    p: &mut Projectile,
    world: &World,
    dust: &mut DustPool,
    rng: &mut dyn DropRng,
    ctx: &SpawnCtx,
    local_owner: u8,
) {
    // `:27862` — the velocity-facing spin (+π: the sheet faces down-left).
    p.rot = p.vel.y.atan2(p.vel.x) + 3.14;
    // `:27863-27875` — the one-shot cue (the −1 latch never re-arms: the
    // generic decrement only runs while positive, `:16768-16771`).
    if p.sound_delay == 0 && p.kind == CRYSTAL_LEAF_SHOT {
        p.sound_delay = -1;
        // `SoundEngine.PlaySound(6, (int)X, (int)Y)` — the int overload's
        // default style 1 (the drain's id-6 façade is the wiring follow-up).
        p.wants_sound = Some((6, 1, p.pos.x, p.pos.y, 0.0, 1.0));
        // `:27867-27874` — the 8× dust-157 burst, pool internals + the
        // POST-spawn NextFloat blend (the e3 emitter, activated).
        crystal_leaf_burst_227(p, world, dust, rng, ctx);
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
/// play (draw-free). The 15× dust-157 bodies + their POST-spawn
/// `NextFloat` blends (`:74529-74532`) are W4-g65's death drain
/// (`projectile_death_dust.rs`'s 227 arm — snapshot→drain on the live
/// `UpdateProjectiles` stream); the g65 body-lane supersedes this hook's
/// former 15-float census, exactly like the AI burst's e3 supersede.
pub fn leaf_shot_kill(p: &mut Projectile) {
    p.wants_sound = Some((6, 1, p.pos.x, p.pos.y, 0.0, 1.0));
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

    fn ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(1280.0, 720.0),
            max_dust_to_draw: 6000,
        }
    }

    fn world() -> World {
        World::new("test".into(), 64, 64)
    }

    /// `:27862` — the rotation faces the velocity + π.
    #[test]
    fn w4_d5_s43_the_rotation_faces_the_velocity_plus_pi() {
        let (w, c) = (world(), ctx());
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(43);
        let mut p = shot(Vec2::new(12.0, 0.0));
        leaf_shot_step(&mut p, &w, &mut dust, &mut rng, &c, 0);
        assert!((p.rot - 3.14).abs() < 1e-5, "atan2(0,12)=0 → 3.14");
        let mut p = shot(Vec2::new(0.0, -12.0));
        leaf_shot_step(&mut p, &w, &mut dust, &mut rng, &c, 0);
        assert!((p.rot - (3.14 - std::f32::consts::FRAC_PI_2)).abs() < 1e-5);
    }

    /// `:27863-27875` — the first tick latches soundDelay −1, queues the
    /// id-6 cue and emits EXACTLY 8 dust-157 bodies through the pool
    /// (noGravity, scale 1.5); later ticks emit nothing. The draw-mirror
    /// witness below pins the stream position.
    #[test]
    fn w4_d5_s43_the_first_tick_emits_eight_bodies_once() {
        let (w, c) = (world(), ctx());
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(43);
        let mut p = shot(Vec2::new(12.0, 0.0));
        leaf_shot_step(&mut p, &w, &mut dust, &mut rng, &c, 0);
        assert_eq!(p.sound_delay, -1);
        assert_eq!(p.wants_sound, Some((6, 1, p.pos.x, p.pos.y, 0.0, 1.0)));
        let active: Vec<_> = dust.slots().iter().filter(|d| d.active).collect();
        assert_eq!(active.len(), 8);
        for d in &active {
            assert_eq!(d.dust_type, 157);
            assert!(d.no_gravity);
            assert_eq!(d.scale, 1.5);
        }
        for _ in 0..20 {
            leaf_shot_step(&mut p, &w, &mut dust, &mut rng, &c, 0);
        }
        assert_eq!(dust.slots().iter().filter(|d| d.active).count(), 8, "no further bursts");
    }

    /// The activated emitter's draw order (research/231's activation
    /// contract): per dust, the verbatim `NewDust` internals FIRST, then
    /// the post-writes with the POST-spawn `NextFloat` blend — mirrored
    /// body-for-body against a direct `DustPool::new_dust` loop on a
    /// cloned stream; the streams re-align afterwards. WRITTEN NOT RUN.
    #[test]
    fn w4_d5_s43_the_burst_mirrors_the_pool_internals_and_post_float() {
        let (w, c) = (world(), ctx());
        let p = shot(Vec2::new(12.0, -5.0));
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(43);
        crystal_leaf_burst_227(&p, &w, &mut dust, &mut rng, &c);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(43);
        for _ in 0..8 {
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, p.pos, 14, 14, 157,
                0.0, 0.0, 0, Color::default(), 1.0,
            );
            let md = mpool.slot_mut(idx);
            md.no_gravity = true;
            md.velocity *= 3.0;
            md.scale = 1.5;
            md.velocity += p.vel * mirror.next_float();
        }
        for i in 0..8 {
            assert_eq!(dust.slot(i).dust_type, 157);
            assert_eq!(dust.slot(i).position, mpool.slot(i).position);
            assert_eq!(dust.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(dust.slot(i).scale, mpool.slot(i).scale);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000), "stream re-aligned");
    }

    /// `:27879` — the post-increment fires every OTHER tick: counter
    /// 0→1 (no fire), 1→0 (fire), 0→1, …
    #[test]
    fn w4_d5_s43_the_frame_latch_fires_every_other_tick() {
        let (w, c) = (world(), ctx());
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(1);
        let mut p = shot(Vec2::new(1.0, 0.0));
        p.sound_delay = -1; // silence the cue's burst
        let mut fires = 0;
        for tick in 0..10 {
            let old = p.frame_counter;
            leaf_shot_step(&mut p, &w, &mut dust, &mut rng, &c, 0);
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
        // `:27876` — the exact f32 expression (the `+1 %1` step rounds
        // through 1.33f32, so the hue is 0.33000004, NOT the 0.33 literal;
        // the Red remap is therefore 8.05e-8, not 0, and its per-channel
        // truncation drops G 255 -> 254 before the LimeGreen/Gold blend).
        let num320 = (((1.0f32 - 180.0 / 180.0) * -6.0 * 0.85 + 0.33) % 1.0 + 1.0) % 1.0;
        let base = crate::ui::color::hsl_to_rgb(num320 as f64, 1.0, 0.5, 255);
        assert_eq!(base, [5, 255, 0, 255], "hslToRgb(0.33000004, 1, 0.5)");
        let value0 = lerp_rgb(
            [base[0], base[1], base[2]],
            [255, 0, 0],
            get_lerp_value(0.33, 0.7, num320),
        );
        assert_eq!(value0, [5, 254, 0], "the Red lerp's truncation");
        let lg = lerp_rgb([50, 205, 50], [255, 215, 0], 0.3);
        let want = lerp_rgb(value0, lg, value0[0] as f32 / 255.0);
        assert_eq!(value, want);
        // The literal oracle (XNA `Color.Lerp` truncates per channel).
        assert_eq!(value, [7, 253, 0], "the byte-exact spawn colour");
        let (h, _, _) = crate::ui::color::rgb_to_hsl([want[0], want[1], want[2], 255]);
        assert_eq!(piece, (h as f32 * 255.0) as u8);
    }

    /// The Kill hook queues the id-6 cue and draws NOTHING — the 15×
    /// dust-157 bodies + their NextFloat blends ride the g65 death drain
    /// (`projectile_death_dust.rs`'s 227 arm, `:74524-74534`).
    #[test]
    fn w4_d5_s43_the_kill_hook_queues_the_cue_draw_free() {
        let mut rng = UnifiedRandom::new(5);
        let mut oracle = UnifiedRandom::new(5);
        let mut p = shot(Vec2::new(3.0, 4.0));
        leaf_shot_kill(&mut p);
        assert_eq!(p.wants_sound, Some((6, 1, p.pos.x, p.pos.y, 0.0, 1.0)));
        assert_eq!(rng.next_float().to_bits(), oracle.next_float().to_bits(), "zero draws");
        assert_eq!(rng.next_range(0, 999), oracle.next_range(0, 999));
    }
}
