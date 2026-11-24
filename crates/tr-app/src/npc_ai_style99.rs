//! The aiStyle-99 family — the Solar Goop (`NPC.AI()`'s
//! `else if (this.aiStyle == 99)` arm, `NPC.cs:39927-39990`). W4-d33,
//! research/25 row 99, CODE-ONLY MODE (no compiler invoked; the witness
//! tests below are written, not run).
//!
//! ## Census
//!
//! The family has exactly ONE vanilla member: the SolarGoop 519
//! (`NPCID.SolarGoop`, `SetDefaults` row `NPC.cs:15409-15423`). The
//! census of `this.aiStyle = 99;` across the decompiled tree returns
//! exactly two hits: `NPC.cs:15413` (the `type == 519` branch) and
//! `Projectile.cs:10766` (a projectile-space machine, out of NPC
//! scope). No dynamic reassignment; no second NPC kind rides it.
//!
//! The port's roster has no `NpcKind::SolarGoop` and `update_rng`'s
//! aiStyle axis has no style-99 arm — the module lands UNWIRED (the
//! `npc_ai_style73.rs` precedent). The machine keys on the vanilla
//! `npc_type` until the roster wave.
//!
//! ## The machine
//!
//! A one-shot falling blob. The landing latch (`:39929-39935`):
//! `velocity.Y == 0 && ai[0] == 0` arms the 5-tick dissolve
//! (`ai[0] = 1`, `ai[1] = 0`, `netUpdate`) and RETURNS. The dissolve
//! (`:39936-39947`): velocity zeroed, position snapped back to
//! `oldPosition` every tick, `ai[1]++`; at 5 the `HitEffect(0, 9999.0)`
//! + `active = false` pair fires and the arm RETURNS. Otherwise the
//! fall (`:39948-39953`): `vy += 0.2` capped at 12, rotation =
//! `velocity.ToRotation() - π/2` — the row's `noGravity = true` means
//! this manual gravity is the ONLY fall force (the caller's
//! `UpdateNPC` gravity tail is gated off).
//!
//! The 519-only dust (`:39954-39989`): the first fall tick
//! (`localAI[0] == 0`) latches `localAI[0] = 1` and spawns the 13-dust
//! type-6 burst (`vx*0.5, vy*0.5`, alpha 90, scale 2.5; each
//! `noGravity`, `fadeIn = 1`, velocity ×4, `noLight`). EVERY fall tick
//! then runs the 3-iteration trail: `Main.rand.Next(3) < 2` gates a
//! type-6 dust (same spawn parameters; `noGravity`, velocity ×0.2,
//! `fadeIn = 1`), and `Main.rand.Next(6) == 0` picks the fast variant
//! (velocity ×30, `noGravity = false`, `noLight`) over the redirect
//! (`velocity = DirectionFrom(dust.position) * velocity.Length()` —
//! `Entity.cs`'s `Normalize(Center - Source)`).
//!
//! ## RNG order (by construction)
//!
//! The arm's explicit draws per fall tick: 3 × `Next(3)` gates, each
//! passing gate adding the `NewDust` internals (`Dust.cs:75-256`, the
//! ported `DustPool::new_dust`) then one `Next(6)` pick. The
//! first-tick burst adds 13 `NewDust` internals BEFORE the trail.
//! The dissolve's `HitEffect(0, 9999.0)` runs the type-519 arm
//! (`NPC.cs:83008-83022`): the `oldPos` trail sampled at indices
//! 0/3/6/9 (`oldPos.Length = 10`, `NPC.cs:6119`, step 3), 5 dusts per
//! site, each `Utils.SelectRandom(Main.rand, 6, 259, 158)` (ONE
//! `Next(3)`, `Utils.cs`'s `choices[Next(len)]`) evaluated BEFORE the
//! `NewDust` internals, then `scale += NextFloat()`. The port's `Npc`
//! has no `oldPos` array — [`style99_step`] takes it as the wiring
//! seam parameter (the `projectile.rs:560` `old_pos` precedent); the
//! witnesses pass the zero trail.
//!
//! `HitEffect`'s other gates draw nothing for 519: not a town NPC, no
//! daybreak, none of the type arms at `:81660-82655` match, and the
//! generic else-chain (`:85534-89832`) has no 519 row (the final else
//! returns at `:89813-89815`). The `HitSound`/`DeathSound` are both
//! null (`:15417-15418`) — no sound cue on either path.
//!
//! ## FindFrame
//!
//! `NPC.FindFrame` (`NPC.cs:66684`) has NO case 519 — the frame never
//! advances (`Main.npcFrameCount[519] = 1`, the 510-519 row
//! `1, 1, 1, 1, 1, 1, 4, 1, 10, 1`, `Main.cs`'s `npcFrameCount`
//! array). Nothing to port.
#![allow(dead_code)] // wiring deferred — the kind + dispatch arm land with the roster wave

use crate::npc::Npc;
use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// The type-519 `SetDefaults` row (`NPC.cs:15409-15423`), verbatim,
/// with the head defaults the branch leaves untouched (`:8510`
/// `npcSlots = 1`, `:8533` overridden 0, `:8536` `scale = 1`,
/// `:8544` `value = 0`).
pub struct SolarGoopDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// Both `HitSound` and `DeathSound` are null (`:15417-15418`).
    pub hit_sound: Option<(i32, i32)>,
    pub death_sound: Option<(i32, i32)>,
    pub knock_back_resist: f32,
    pub no_gravity: bool,
    pub no_tile_collide: bool,
    pub alpha: u8,
    pub value: f32,
}

/// `NPC.cs:15409-15423` — `else if (this.type == 519)`.
pub const SOLAR_GOOP: SolarGoopDefaults = SolarGoopDefaults {
    width: 26,
    height: 26,
    ai_style: 99,
    damage: 120,
    defense: 0,
    life_max: 1,
    hit_sound: None,
    death_sound: None,
    knock_back_resist: 0.0,
    no_gravity: true,
    no_tile_collide: false,
    alpha: 0,
    value: 0.0,
};

/// `Main.npcFrameCount[519]` = 1 (the 510-519 row, index 519 = the
/// 10th entry).
pub const SOLAR_GOOP_FRAME_COUNT: u32 = 1;

/// The per-tick environment (the `Style86Env` shape, trimmed — the
/// arm never targets).
pub struct Style99Env<'a> {
    /// The read-only world `NewDust` reads.
    pub world: &'a World,
    /// `Main.dust` — the shared 6001-slot pool (`Dust.cs:75-256`).
    pub dust: &'a mut DustPool,
    /// The `Main` statics `NewDust` reads (`Dust.cs:77-98`).
    pub dust_ctx: &'a SpawnCtx,
    /// The `Main.rand` stream — every draw cited in the module header
    /// rides it.
    pub rng: &'a mut UnifiedRandom,
}

/// The arm's per-tick side effects beyond the NPC state.
#[derive(Default, Debug, PartialEq)]
pub struct Style99Outcome {
    /// The dissolve expiry (`:39943-39944`): the `HitEffect(0, 9999.0)`
    /// already ran INSIDE the step (the oldPos trail dusts); the
    /// caller owes only the `active = false` deactivation (the port's
    /// pool has no `active` field — the style72 pop precedent).
    pub died: bool,
}

/// `(float)Math.Sqrt(x*x + y*y)` — XNA `Vector2.Length()` (the f32
/// sum widened to f64 for the root, the style86 `sqrt_f32` shape).
fn sqrt_f32(x: f32, y: f32) -> f32 {
    ((x * x + y * y) as f64).sqrt() as f32
}

/// XNA `Vector2.Normalize()` — UNGUARDED: a zero vector gives NaN,
/// exactly like the C# (the style86 `normalize_xna` precedent).
fn normalize_xna(x: f32, y: f32) -> (f32, f32) {
    let num = 1.0f32 / ((x * x + y * y) as f64).sqrt() as f32;
    (x * num, y * num)
}

/// The `HitEffect(0, 9999.0)` type-519 arm (`NPC.cs:83008-83022`),
/// verbatim: the `oldPos` trail at indices 0/3/6/9, 5 dusts per site
/// (`SelectRandom(6, 259, 158)` — one `Next(3)` BEFORE the `NewDust`
/// internals — at the trail point, box-sized, speed `(0, -2.5)` for
/// `hitDirection = 0`, alpha 200, velocity ×2.4, `scale +=
/// NextFloat()`). `hit_direction` is the C# parameter (the dissolve
/// passes 0).
pub fn goop_hit_effect(
    npc: &Npc,
    old_pos: &[Vec2; 10],
    hit_direction: i32,
    env: &mut Style99Env<'_>,
) {
    let (w, h) = npc.size();
    // `:83011` — `for (num123 = 0; num123 < oldPos.Length; num123 += 3)`.
    let mut i = 0;
    while i < old_pos.len() {
        let site = old_pos[i];
        for _ in 0..5 {
            // `:83017` — the SelectRandom argument draw precedes NewDust.
            let dust_type = [6, 259, 158][env.rng.next_max(3) as usize];
            let idx = env.dust.new_dust(
                env.rng,
                env.dust_ctx,
                env.world,
                site,
                w as i32,
                h as i32,
                dust_type,
                2.5 * hit_direction as f32,
                -2.5,
                0,
                Color::default(),
                1.0,
            );
            {
                let d = env.dust.slot_mut(idx);
                d.alpha = 200;
                d.velocity *= 2.4;
                d.scale += env.rng.next_float();
            }
        }
        i += 3;
    }
}

/// One fixed step of the aiStyle-99 arm (`NPC.cs:39927-39990`), in C#
/// order. `npc_type` is the vanilla id (519) until the roster kind
/// lands; `old_pos` is `NPC.oldPos` (the wiring seam — module docs).
/// The caller owns the raw movement integration after `AI()` (the
/// row's `noGravity = true` gates the gravity tail off) and drains
/// [`Style99Outcome`].
pub fn style99_step(
    npc: &mut Npc,
    npc_type: i32,
    old_pos: &[Vec2; 10],
    env: &mut Style99Env<'_>,
) -> Style99Outcome {
    let mut out = Style99Outcome::default();
    // `:39929-39935` — the landing latch.
    if npc.vy == 0.0 && npc.ai[0] == 0.0 {
        npc.ai[0] = 1.0;
        npc.ai[1] = 0.0;
        npc.net_update = true;
        return out;
    }
    // `:39936-39947` — the 5-tick dissolve.
    if npc.ai[0] == 1.0 {
        npc.vx = 0.0;
        npc.vy = 0.0;
        npc.x = npc.old_x;
        npc.y = npc.old_y;
        npc.ai[1] += 1.0;
        if npc.ai[1] >= 5.0 {
            goop_hit_effect(npc, old_pos, 0, env);
            out.died = true;
        }
        return out;
    }
    // `:39948-39953` — the manual fall + the velocity-aligned tilt.
    npc.vy += 0.2;
    if npc.vy > 12.0 {
        npc.vy = 12.0;
    }
    npc.rotation = npc.vy.atan2(npc.vx) - std::f32::consts::FRAC_PI_2;
    // `:39954-39957` — only the 519 draws.
    if npc_type != 519 {
        return out;
    }
    let (w, h) = npc.size();
    // `:39958-39969` — the first-tick 13-dust burst.
    if npc.local_ai[0] == 0.0 {
        npc.local_ai[0] = 1.0;
        for _ in 0..13 {
            let idx = env.dust.new_dust(
                env.rng,
                env.dust_ctx,
                env.world,
                Vec2::new(npc.x, npc.y),
                w as i32,
                h as i32,
                6,
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
    // `:39970-39989` — the every-tick 3-iteration trail.
    for _ in 0..3 {
        if env.rng.next_max(3) < 2 {
            let idx = env.dust.new_dust(
                env.rng,
                env.dust_ctx,
                env.world,
                Vec2::new(npc.x, npc.y),
                w as i32,
                h as i32,
                6,
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
                d.fade_in = 1.0;
                if env.rng.next_max(6) == 0 {
                    // `:39980-39982` — the fast variant.
                    d.velocity *= 30.0;
                    d.no_gravity = false;
                    d.no_light = true;
                } else {
                    // `:39986` — DirectionFrom(dust.position) * Length().
                    let len = sqrt_f32(d.velocity.x, d.velocity.y);
                    let (cx, cy) = npc.center();
                    let (nx, ny) = normalize_xna(cx - d.position.x, cy - d.position.y);
                    d.velocity = Vec2::new(nx * len, ny * len);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// A 519-shaped NPC: no kind yet, so the fixture rides a
    /// placeholder with the C# box through `size_override` (the
    /// style73 turret convention).
    fn goop(x: f32, y: f32) -> Npc {
        let d = &SOLAR_GOOP;
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

    /// The menu ctx: `NewDust` refuses BEFORE any draw (`Dust.cs:77`)
    /// — isolates the arm's EXPLICIT Next draws (the style86 fixture).
    fn menu_ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: true,
            ..spawn_ctx()
        }
    }

    /// `NPC.cs:15409-15423` — the SetDefaults row, field by field.
    #[test]
    fn w4_d33_s99_the_defaults_row() {
        let d = &SOLAR_GOOP;
        assert_eq!((d.width, d.height), (26, 26));
        assert_eq!(d.ai_style, 99);
        assert_eq!((d.damage, d.defense, d.life_max), (120, 0, 1));
        assert_eq!(d.hit_sound, None);
        assert_eq!(d.death_sound, None);
        assert_eq!(d.knock_back_resist, 0.0);
        assert!(d.no_gravity);
        assert!(!d.no_tile_collide);
        assert_eq!(d.alpha, 0);
        assert_eq!(SOLAR_GOOP_FRAME_COUNT, 1);
    }

    /// `:39929-39935` — the landing latch arms the dissolve and
    /// returns without touching the velocity.
    #[test]
    fn w4_d33_s99_the_landing_latch_arms_the_dissolve() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(99);
        let mut n = goop(100.0, 100.0);
        n.vy = 0.0;
        let trail = [Vec2::ZERO; 10];
        let mut env = Style99Env { world: &w, dust: &mut pool, dust_ctx: &c, rng: &mut rng };
        let out = style99_step(&mut n, 519, &trail, &mut env);
        assert_eq!(n.ai[0], 1.0);
        assert_eq!(n.ai[1], 0.0);
        assert!(n.net_update);
        assert!(!out.died);
        assert_eq!(n.vy, 0.0, "the latch tick never falls");
        assert_eq!(rng.peek(), UnifiedRandom::new(99).peek(), "draw-free");
    }

    /// `:39936-39947` — the dissolve freezes at oldPosition, counts 5
    /// ticks, then fires the HitEffect + death seam. Under the menu
    /// ctx the trail dusts refuse before drawing, so the whole
    /// dissolve is explicit-draw-free.
    #[test]
    fn w4_d33_s99_the_dissolve_snaps_back_and_dies_at_five() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(99);
        let mut n = goop(100.0, 100.0);
        n.old_x = 90.0;
        n.old_y = 80.0;
        n.ai[0] = 1.0;
        n.vx = 3.0;
        n.vy = 4.0;
        let trail = [Vec2::ZERO; 10];
        let mut env = Style99Env { world: &w, dust: &mut pool, dust_ctx: &c, rng: &mut rng };
        for tick in 1..=5 {
            let out = style99_step(&mut n, 519, &trail, &mut env);
            assert_eq!((n.x, n.y), (90.0, 80.0), "the oldPosition snap @{tick}");
            assert_eq!((n.vx, n.vy), (0.0, 0.0));
            assert_eq!(n.ai[1], tick as f32);
            assert_eq!(out.died, tick == 5, "the death fires at 5 only");
        }
        assert_eq!(rng.peek(), UnifiedRandom::new(99).peek(), "menu ctx refuses all");
    }

    /// `:39948-39953` — the fall accelerates 0.2/tick, caps at 12,
    /// and the rotation tracks the velocity minus π/2.
    #[test]
    fn w4_d33_s99_the_fall_caps_at_twelve() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(99);
        let mut n = goop(100.0, 100.0);
        n.ai[0] = 2.0; // past both state arms — the fall tail only
        let trail = [Vec2::ZERO; 10];
        let mut env = Style99Env { world: &w, dust: &mut pool, dust_ctx: &c, rng: &mut rng };
        let _ = style99_step(&mut n, 516, &trail, &mut env); // non-519: no dust
        assert_eq!(n.vy, 0.2);
        assert_eq!(n.rotation, (0.2f32).atan2(0.0) - std::f32::consts::FRAC_PI_2);
        for _ in 0..100 {
            let _ = style99_step(&mut n, 516, &trail, &mut env);
        }
        assert_eq!(n.vy, 12.0, "the cap holds");
        assert_eq!(rng.peek(), UnifiedRandom::new(99).peek(), "516 draws nothing");
    }

    /// `:39958-39989` — the 519 FIRST fall tick's explicit draw census
    /// under the menu ctx: the 13-burst is NewDust-internals-only (no
    /// explicit draws), then 3 Next(3) gates, each passing gate adding
    /// one Next(6). The mirrored oracle replays the exact sequence.
    #[test]
    fn w4_d33_s99_the_first_tick_trail_draws_in_order() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(99);
        let mut oracle = UnifiedRandom::new(99);
        let mut n = goop(100.0, 100.0);
        n.ai[0] = 2.0;
        n.vy = 1.0;
        let trail = [Vec2::ZERO; 10];
        let mut env = Style99Env { world: &w, dust: &mut pool, dust_ctx: &c, rng: &mut rng };
        let _ = style99_step(&mut n, 519, &trail, &mut env);
        assert_eq!(n.local_ai[0], 1.0, "the burst latch");
        for _ in 0..3 {
            if oracle.next_max(3) < 2 {
                oracle.next_max(6);
            }
        }
        assert_eq!(rng.peek(), oracle.peek(), "the trail draw order");
    }

    /// `:83008-83022` — the HitEffect trail census: 4 sites × (one
    /// Next(3) SelectRandom + one NextFloat) = 8 explicit draws under
    /// the menu ctx, in site order.
    #[test]
    fn w4_d33_s99_the_hit_effect_trail_draws_eight() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(519);
        let mut oracle = UnifiedRandom::new(519);
        let n = goop(100.0, 100.0);
        let trail = [Vec2::ZERO; 10];
        let mut env = Style99Env { world: &w, dust: &mut pool, dust_ctx: &c, rng: &mut rng };
        goop_hit_effect(&n, &trail, 0, &mut env);
        for _ in 0..4 {
            for _ in 0..5 {
                oracle.next_max(3);
                oracle.next_float();
            }
        }
        assert_eq!(rng.peek(), oracle.peek(), "4 sites x 5 dusts x 2 draws");
    }

    /// The full-pool stream parity: the first fall tick on the REAL
    /// dust ctx (the NewDust internals interleave) must match a
    /// cloned twin step bit-for-bit (the style86 twin-pool precedent).
    #[test]
    fn w4_d33_s99_the_full_pool_first_tick_is_twin_stable() {
        let w = air();
        let c = spawn_ctx();
        let trail = [Vec2::ZERO; 10];
        let mut a = goop(600.0, 600.0);
        a.ai[0] = 2.0;
        a.vy = 1.0;
        let mut b = a.clone();
        let mut pool_a = DustPool::new();
        let mut pool_b = DustPool::new();
        let mut rng_a = UnifiedRandom::new(519);
        let mut rng_b = UnifiedRandom::new(519);
        for tick in 0..30 {
            let mut env_a = Style99Env { world: &w, dust: &mut pool_a, dust_ctx: &c, rng: &mut rng_a };
            let _ = style99_step(&mut a, 519, &trail, &mut env_a);
            let mut env_b = Style99Env { world: &w, dust: &mut pool_b, dust_ctx: &c, rng: &mut rng_b };
            let _ = style99_step(&mut b, 519, &trail, &mut env_b);
            assert_eq!(rng_a.peek(), rng_b.peek(), "stream @{tick}");
            assert_eq!(a.vy.to_bits(), b.vy.to_bits(), "vy @{tick}");
            assert_eq!(a.local_ai[0].to_bits(), b.local_ai[0].to_bits(), "latch @{tick}");
        }
    }
}
