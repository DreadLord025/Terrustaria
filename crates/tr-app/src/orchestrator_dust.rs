//! W4-e4 — the world dust sites' SECOND batch, part 1: the
//! `ParticleOrchestrator.cs` `Dust.NewDust*` sites 41-51 (lines 2653-3061,
//! research/66's claim doc). The first 40 orchestrator sites (423-2582)
//! stay W4-d54's lane; WorldGen.cs is W4-g14's and Main.cs is
//! W4-g11/g12/g13's (g12's pushed census).
//!
//! Contract (the S1-E-1 pool, `tr_core::particles`, d50's DustPool):
//! the orchestrator spawns run SYNCHRONOUSLY inside
//! `RequestParticleSpawn`/`BroadcastParticleSpawn`
//! (`ParticleOrchestrator.cs:52-90` → `SpawnParticlesDirect` :162), so
//! every draw rides the caller's pass stream in C# statement order —
//! the explicit `Main.rand` argument draws FIRST (C# evaluates arguments
//! before the call), then the `NewDust` internals roll inside
//! `DustPool::new_dust` (`Dust.cs:75-256`), then the post-spawn writes
//! (which read `Main.dust[6000]` when the pool refuses — the sentinel
//! slot takes the writes exactly like the C#). The `PrettySparkle` /
//! `Flame` / `Fading` / `RandomizedFrame` particle-pool requests are
//! RNG-free (`ParticlePool.RequestParticle`,
//! `Graphics/Renderers/ParticlePool.cs:32`); their ARGUMENT draws are
//! consumed in order, the particle visuals themselves are the documented
//! renderer skip (research/15: no particle renderer exists).
//!
//! Batch ledger (C# `Dust.NewDust*` line = site id):
//! - 2653/2658 — `Spawn_ShimmerArrow`'s i==0 dust pair (`:2592-2667`).
//! - 2775/2778 — `Spawn_TerraBlade`'s pair (`:2702-2783`).
//! - 2837/2840 — `Spawn_Excalibur`'s pair (`:2785-2845`).
//! - 2893 — `Spawn_ClassyCane`'s 7-dust loop (`:2874-2899`); the gore
//!   sibling :2888 stays W4-d55/g17's lane — its ARGUMENT draws are
//!   consumed here in order (the e2 split), the `NewGore` body rolls
//!   (`Gore.cs:1327-1328`, two draws) are the gore pool owner's.
//! - 2997/3000/3058/3061 — `Spawn_TrueExcalibur`'s two gated pairs
//!   (`:2955-3067`), ported WHOLE (the golden rule).
//! WRITTEN NOT RUN (code-only mode): the witnesses below are compiled by
//! the merge-verify wave, not executed here.

#![allow(dead_code)] // unwired until the orchestrator drain lands (code-only mode)

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

use crate::bow_launch::rotated_by;
use crate::ui::color::hsl_to_rgb;

/// `ParticleOrchestraSettings` (`ParticleOrchestraSettings.cs:6-14`):
/// the four fields every `Spawn_*` arm reads. The net carrier
/// (`app/netmodules.rs::Particles`) adds the type byte on top.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct OrchestraSettings {
    pub position: Vec2,
    pub movement: Vec2,
    pub info: i32,
    pub invoker: u8,
}

/// `Utils.ToRotationVector2` (`Utils.cs:1665-1668`): cos/sin in DOUBLE
/// of the widened float, cast back to f32.
pub(crate) fn to_rotation_vector2(f: f32) -> Vec2 {
    Vec2::new((f as f64).cos() as f32, (f as f64).sin() as f32)
}

/// `Utils.ToRotation` (`Utils.cs:1655-1658`): `Math.Atan2(Y, X)` in
/// double of the widened floats, cast back.
pub(crate) fn to_rotation(v: Vec2) -> f32 {
    (v.y as f64).atan2(v.x as f64) as f32
}

/// `Utils.NextVector2Unit` (`Utils.cs:1261-1264`) — one `NextFloat`;
/// the angle is FLOAT arithmetic (`rotationRange * NextFloat()`, the
/// default range `(float)Math.PI * 2f`), then `ToRotationVector2`.
fn next_vector2_unit(rng: &mut UnifiedRandom) -> Vec2 {
    let angle = std::f32::consts::PI * 2.0 * rng.next_float();
    to_rotation_vector2(angle)
}

/// `Utils.NextVector2Circular` (`Utils.cs:1266-1269`) — the unit draw
/// THEN a second `NextFloat` radius scale; the half-extents multiply
/// per axis before the radius.
pub(crate) fn next_vector2_circular(rng: &mut UnifiedRandom, half_w: f32, half_h: f32) -> Vec2 {
    let unit = next_vector2_unit(rng);
    let r = rng.next_float();
    Vec2::new(unit.x * half_w * r, unit.y * half_h * r)
}

/// `Utils.NextVector2Square` (`Utils.cs:1251-1254`) — TWO `NextDouble`
/// draws (X first), each `(max - min) * (float)NextDouble() + min`.
pub(crate) fn next_vector2_square(rng: &mut UnifiedRandom, min: f32, max: f32) -> Vec2 {
    let x = (max - min) * rng.next_double() as f32 + min;
    let y = (max - min) * rng.next_double() as f32 + min;
    Vec2::new(x, y)
}

/// XNA `Color.Lerp` (`MonoGame Color.cs`): `MathHelper.Lerp` per channel
/// in f32, then the ctor's truncating `(int)` cast (the `dust_table.rs`
/// convention, kept local — that helper is private to the table).
pub(crate) fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let ch = |x: u8, y: u8| ((x as f32 + (y as f32 - x as f32) * t) as i32) as u8;
    Color::new(ch(a.r, b.r), ch(a.g, b.g), ch(a.b, b.b), ch(a.a, b.a))
}

/// XNA `new Color(float, float, float, float)`: every channel `* 255f`
/// then the truncating byte cast.
fn color_f(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color::new(
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
        (a * 255.0) as u8,
    )
}

/// `Main.hslToRgb(Hue, 1f, Luminosity)` with the default `a = 255`
/// (`Main.cs:47890`) — the port's byte-exact banker's-rounding walk
/// (`ui::color::hsl_to_rgb`), the float args widened on the way in.
fn hsl(h: f32, l: f32) -> Color {
    let [r, g, b, a] = hsl_to_rgb(h as f64, 1.0, l as f64, 255);
    Color::new(r, g, b, a)
}

/// The sword-sparkle dust pair every blade arm shares
/// (`:2653-2662` / `:2775-2780` / `:2837-2842` / `:2997-3002` /
/// `:3058-3063` / `:3195-3200`): per dust the velocity argument draws
/// `NextFloatDirection` THEN `NextFloat` (the `RotatedBy` angle is
/// `dir * ((float)Math.PI * 2f) * 0.025f`, all f32, widened for the
/// double `RotatedBy`), then the `NewDustPerfect` internals, then the
/// post-writes. The second dust spins the NEGATED base vector.
#[allow(clippy::too_many_arguments)]
fn blade_dust_pair(
    base: Vec2,
    dust_type: i32,
    scale_a: f32,
    scale_b: f32,
    fade: Option<f32>,
    color: Option<Color>,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    pos: Vec2,
) {
    for (sign, scale) in [(1.0f32, scale_a), (-1.0, scale_b)] {
        let dir = rng.next_float_direction();
        let angle = dir * (std::f32::consts::PI * 2.0) * 0.025;
        let mag = rng.next_float();
        let vel = rotated_by(base * sign, angle as f64) * mag;
        let idx = dust.new_dust_perfect(rng, ctx, world, pos, dust_type, Some(vel), 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.scale = scale;
        if let Some(f) = fade {
            d.fade_in = f;
        }
        if let Some(c) = color {
            d.color = c;
        }
    }
}

/// Sites 2653/2658 — `Spawn_ShimmerArrow` (`ParticleOrchestrator.cs:
/// 2592-2667`). Per outer iteration: the spin `num2` draw + the hue draw
/// (`:2597-2598`), `color.A /= 2` is INTEGER division (255 → 127), the
/// `value` copy keeps A=255 and lerps to White 0.5. The two 4-iteration
/// PrettySparkle loops are RNG-free (the visuals are the renderer skip);
/// ONLY the i==0 pass spawns dust — one 306 pair per second-loop
/// iteration, colour = the half-alpha `color`, fadeIn 1.2, scale 1.4.
pub(crate) fn shimmer_arrow(
    s: &OrchestraSettings,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for i in 0..2 {
        // `:2597` — num2 = 2π * NextFloatDirection() * 0.05f (f32 chain).
        let num2 = std::f32::consts::PI * 2.0 * rng.next_float_direction() * 0.05;
        // `:2598-2602` — the hue draw, then the byte surgery.
        let mut color = hsl(rng.next_float(), 0.5);
        color.a /= 2; // `:2599` — integer division.
        let mut value = color;
        value.a = 255;
        let _value = lerp_color(value, Color::WHITE, 0.5); // sparkle tint only
        // `:2603-2626` — the first sparkle loop: zero draws (skip cited).
        for num4 in 0..4 {
            // `:2627-2648` — the second sparkle loop's vector feeds the
            // dust; the sparkle fields themselves draw nothing.
            let vector2 = to_rotation_vector2(std::f32::consts::PI / 2.0 * num4 as f32 + num2) * 4.0;
            if i == 0 {
                // `:2651-2663` — the j<1 loop: one pair per iteration.
                blade_dust_pair(vector2, 306, 1.4, 1.4, Some(1.2), Some(color), rng, dust, ctx, world, s.position);
            }
        }
    }
}

/// Sites 2775/2778 — `Spawn_TerraBlade` (`:2702-2783`). `num2` = the
/// movement's rotation + π/2 (`:2705`, NO draw); the three sparkle loops
/// (4 + 2 + 4 iterations) are RNG-free; the final 4-iteration loop spawns
/// one 107 pair each — scales 0.8 then 1.4, no fadeIn write, no colour.
pub(crate) fn terra_blade(
    s: &OrchestraSettings,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let num2 = to_rotation(s.movement) + std::f32::consts::PI / 2.0; // `:2705`
    // `:2707-2750` — the 4 + 2 sparkle loops: zero draws (skip cited).
    for num5 in 0..4 {
        // `:2751-2772` — the third sparkle loop: zero draws.
        let vector3 = to_rotation_vector2(std::f32::consts::PI / 2.0 * num5 as f32 + num2) * 4.0;
        // `:2773-2781` — the i<1 loop: one pair per iteration.
        blade_dust_pair(vector3, 107, 0.8, 1.4, None, None, rng, dust, ctx, world, s.position);
    }
}

/// Sites 2837/2840 — `Spawn_Excalibur` (`:2785-2845`). `num2 = 0f`
/// (`:2788`); the first 4-iteration sparkle loop is RNG-free; the second
/// spawns one 169 pair per iteration, both scales 1.4.
pub(crate) fn excalibur(
    s: &OrchestraSettings,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let num2 = 0.0f32; // `:2788`
    // `:2789-2812` — the first sparkle loop: zero draws (skip cited).
    for num4 in 0..4 {
        let vector2 = to_rotation_vector2(std::f32::consts::PI / 2.0 * num4 as f32 + num2) * 4.0;
        // `:2835-2843` — the i<1 loop: one pair per iteration.
        blade_dust_pair(vector2, 169, 1.4, 1.4, None, None, rng, dust, ctx, world, s.position);
    }
}

/// Site 2893 — `Spawn_ClassyCane` (`:2874-2899`). The box recentres by
/// INTEGER halves (`num2 / 2`, `num3 / 2`, `:2883-2884`); `num4 =
/// Next(0, 2)` gates the gore loop. Gore sibling :2888 is W4-d55/g17's:
/// its argument draws (X `Next(num2)`, Y `Next(num3)`, then the velocity
/// overwrite's `Next(-10, 11)` + `Next(6)`) are consumed HERE in C#
/// order; the `NewGore` body's two internal draws (`Gore.cs:1327-1328`,
/// type 1218 has SpecialAI 0 and is not DangerousGore — no extra rolls)
/// stay the gore pool owner's. Then 7 type-10 dusts, alpha 80, scale
/// 1.1, speedY -1, each with a `Next(3) != 0` noGravity gate.
pub(crate) fn classy_cane(
    s: &OrchestraSettings,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let num2 = 20; // `:2881`
    let num3 = 20; // `:2882`
    let pos = Vec2::new(s.position.x - (num2 / 2) as f32, s.position.y - (num3 / 2) as f32);
    let num4 = rng.next_range(0, 2); // `:2885`
    for _ in 0..num4 {
        // `:2888` — the gore ARGUMENT draws (the body rolls are d55/g17's).
        let _gx = rng.next_max(num2);
        let _gy = rng.next_max(num3);
        // `:2889` — the velocity overwrite's argument draws.
        let _vx = rng.next_range(-10, 11) as f32 * 0.4;
        let _vy = 0.0 - (3.0 + rng.next_max(6) as f32 * 0.3);
    }
    for _ in 0..7 {
        // `:2893` — NewDust(pos, 20, 20, 10, 0, -1, 80, default, 1.1).
        let idx = dust.new_dust(rng, ctx, world, pos, num2, num3, 10, 0.0, -1.0, 80, Color::default(), 1.1);
        if rng.next_max(3) != 0 {
            // `:2894-2897`
            dust.slot_mut(idx).no_gravity = true;
        }
    }
}

/// Sites 2997/3000/3058/3061 — `Spawn_TrueExcalibur` (`:2955-3067`),
/// ported WHOLE. `num2 = π/4` (`:2958`); the first two sparkle loops
/// (2 + 2 iterations) are RNG-free, but the SECOND loop's dust pair is
/// gated per iteration by `Next(2) != 0` (`:2995`) — type 242, both
/// scales 1.4. Then `num = 30f; num2 = 0f` (`:3006-3007`); the third
/// sparkle loop (4 iterations) is RNG-free; the fourth gates its 169
/// pair the same way (`:3056`). A FAILED gate still spends its draw.
pub(crate) fn true_excalibur(
    s: &OrchestraSettings,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let num2 = std::f32::consts::PI / 4.0; // `:2958`
    // `:2959-2976` — the first sparkle loop: zero draws (skip cited).
    for num4 in 0..2 {
        // `:2977-2992` — the second sparkle loop: zero draws.
        let vector = to_rotation_vector2(std::f32::consts::PI / 2.0 * num4 as f32 + num2) * 4.0;
        if rng.next_max(2) != 0 {
            // `:2995-3003`
            blade_dust_pair(vector, 242, 1.4, 1.4, None, None, rng, dust, ctx, world, s.position);
        }
    }
    let num2 = 0.0f32; // `:3007`
    // `:3008-3031` — the third sparkle loop: zero draws (skip cited).
    for num6 in 0..4 {
        let vector3 = to_rotation_vector2(std::f32::consts::PI / 2.0 * num6 as f32 + num2) * 4.0;
        if rng.next_max(2) != 0 {
            // `:3056-3064`
            blade_dust_pair(vector3, 169, 1.4, 1.4, None, None, rng, dust, ctx, world, s.position);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> World {
        World::new("e4".into(), 200, 200)
    }

    /// A permissive ctx: the whole world on-screen, the full budget —
    /// the pool never culls (the e2 `ctx()` convention).
    fn ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(3200.0, 3200.0),
            max_dust_to_draw: 6000,
        }
    }

    fn active(pool: &DustPool) -> Vec<usize> {
        (0..6000).filter(|&i| pool.slot(i).active).collect()
    }

    fn settings() -> OrchestraSettings {
        OrchestraSettings {
            position: Vec2::new(900.0, 900.0),
            movement: Vec2::new(3.0, -4.0),
            info: 0,
            invoker: 0,
        }
    }

    /// W4-e4 — sites 2653/2658: 2 head draws per outer iteration, then
    /// ONLY i==0 spawns — 4 pairs × 2 dusts × (2 argument draws + 6
    /// internals; 306 is outside the tweak chain). The mirror walks the
    /// same sequence; the colour's A halves by INTEGER division.
    #[test]
    fn w4_e4_shimmer_arrow_spends_the_head_draws_and_the_i0_pairs() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(17);
        let s = settings();
        shimmer_arrow(&s, &mut rng, &mut pool, &c, &w);
        // 8 dusts, all type 306, fadeIn 1.2, scale 1.4, noGravity, the
        // half-alpha colour (255 / 2 = 127 integer); position pinned by
        // NewDustPerfect.
        let slots = active(&pool);
        assert_eq!(slots.len(), 8);
        for &i in &slots {
            let d = pool.slot(i);
            assert_eq!(d.dust_type, 306);
            assert!(d.no_gravity);
            assert_eq!(d.scale, 1.4);
            assert_eq!(d.fade_in, 1.2);
            assert_eq!(d.color.a, 127);
            assert_eq!(d.position, s.position);
        }
        // Twin-pool parity: a second identical run reproduces every slot.
        let mut rng2 = UnifiedRandom::new(17);
        let mut pool2 = DustPool::new();
        shimmer_arrow(&s, &mut rng2, &mut pool2, &c, &w);
        for &i in &slots {
            assert_eq!(pool.slot(i).velocity, pool2.slot(i).velocity);
            assert_eq!(pool.slot(i).color, pool2.slot(i).color);
        }
        // The draw-count contract against a bare stream: 2 head draws per
        // outer iteration (2×2) + i==0's 4 pairs × 2 dusts × (2 argument +
        // 6 internals; 306 is outside the tweak chain) = 4 + 64 = 68.
        let mut bare = UnifiedRandom::new(17);
        for _ in 0..68 {
            bare.next();
        }
        assert_eq!(rng.next_range(0, 1000), bare.next_range(0, 1000));
    }

    /// W4-e4 — site 2893: the Next(0,2) gate, the gore ARGUMENT pairs
    /// (the body rolls stay d55/g17's), then 7 dusts each with a Next(3)
    /// noGravity gate. The mirror replays the exact draw sequence.
    #[test]
    fn w4_e4_classy_cane_keeps_the_gore_argument_split() {
        let w = world();
        let c = ctx();
        let s = settings();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(23);
        let mut mirror = UnifiedRandom::new(23);
        let mut mpool = DustPool::new();
        classy_cane(&s, &mut rng, &mut pool, &c, &w);
        let pos = Vec2::new(s.position.x - 10.0, s.position.y - 10.0);
        let num4 = mirror.next_range(0, 2) as usize;
        for _ in 0..num4 {
            let _gx = mirror.next_max(20);
            let _gy = mirror.next_max(20);
            let _vx = mirror.next_range(-10, 11);
            let _vy = mirror.next_max(6);
        }
        let mut expect = 0usize;
        for _ in 0..7 {
            let idx = mpool.new_dust(&mut mirror, &c, &w, pos, 20, 20, 10, 0.0, -1.0, 80, Color::default(), 1.1);
            let gate = mirror.next_max(3) != 0;
            let d = pool.slot(expect);
            assert_eq!(d.dust_type, 10);
            assert_eq!(d.alpha, 80);
            assert_eq!(d.no_gravity, gate);
            assert_eq!(d.position, mpool.slot(idx).position);
            assert_eq!(d.velocity, mpool.slot(idx).velocity);
            expect += 1;
        }
        assert_eq!(active(&pool).len(), expect);
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e4 — sites 2997/3000/3058/3061: six Next(2) gates (2 + 4),
    /// each PASSED gate spawns a pair (2 argument draws + 7 internals —
    /// 242 and 169 both ride the tweak chain's extra Next(-10,6)); a
    /// failed gate spends only its one draw. Twin-pool parity over the
    /// whole arm.
    #[test]
    fn w4_e4_true_excalibur_gates_both_pairs() {
        let w = world();
        let c = ctx();
        let s = settings();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(29);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(29);
        true_excalibur(&s, &mut rng, &mut pool, &c, &w);
        let mut expect = 0usize;
        for (count, dtype) in [(2, 242), (4, 169)] {
            for _ in 0..count {
                if mirror.next_max(2) != 0 {
                    for _ in 0..2 {
                        let _dir = mirror.next_float_direction();
                        let _mag = mirror.next_float();
                        let idx = mpool.new_dust_perfect(&mut mirror, &c, &w, s.position, dtype, Some(Vec2::ZERO), 0, Color::default(), 1.0);
                        let d = pool.slot(expect);
                        assert_eq!(d.dust_type, dtype);
                        assert!(d.no_gravity);
                        assert_eq!(d.scale, 1.4);
                        // The internals' positions match (the Perfect
                        // overwrite pins position; velocity is the
                        // argument, checked via the stream parity below).
                        assert_eq!(d.position, mpool.slot(idx).position);
                        expect += 1;
                    }
                }
            }
        }
        assert_eq!(active(&pool).len(), expect);
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e4 — sites 2775/2778 + 2837/2840: the blade arms draw NO head
    /// floats — 4 pairs × 2 dusts × (2 + internals) each; 107 is outside
    /// the tweak chain (6), 169 inside (7). The terra blade's angle rides
    /// the movement's atan2, the excalibur's is flat 0.
    #[test]
    fn w4_e4_terra_blade_and_excalibur_draw_only_the_pairs() {
        let w = world();
        let c = ctx();
        let s = settings();
        for (f, dtype, internals): (fn(&OrchestraSettings, &mut UnifiedRandom, &mut DustPool, &SpawnCtx, &World), i32, usize) in
            [(terra_blade, 107i32, 6usize), (excalibur, 169i32, 7usize)]
        {
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(37);
            f(&s, &mut rng, &mut pool, &c, &w);
            let slots = active(&pool);
            assert_eq!(slots.len(), 8);
            for &i in &slots {
                assert_eq!(pool.slot(i).dust_type, dtype);
                assert!(pool.slot(i).no_gravity);
                assert_eq!(pool.slot(i).position, s.position);
            }
            // 8 × (2 argument + internals) draws, no head floats.
            let mut bare = UnifiedRandom::new(37);
            for _ in 0..8 * (2 + internals) {
                bare.next();
            }
            assert_eq!(rng.next_range(0, 1000), bare.next_range(0, 1000));
        }
    }
}
