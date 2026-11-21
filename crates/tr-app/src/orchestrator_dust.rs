//! W4-e4 — the world dust sites' SECOND batch: the
//! `ParticleOrchestrator.cs` `Dust.NewDust*` sites 41-61 (lines 2653-3920,
//! research/66's claim doc). The first 40 orchestrator sites (423-2582)
//! stay W4-d54's lane; WorldGen.cs is W4-g14's and Main.cs is
//! W4-g11/g12/g13's (g12's pushed census). The Mount.cs sites 1-19 live
//! in `mount_dust.rs` (batches 3-4).
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
//! - 3195/3198 — `Spawn_TrueNightsEdge`'s double pair (`:3139-3203`),
//!   both loops step +2 over <3 (the num==1f arms are DEAD, cited).
//! - 3324/3378/3428/3478 — the 267-sparkle tail family: PaladinsHammer
//!   (`:3286-3339`), PrincessWeapon (`:3341-3393`), StardustPunch
//!   (`:3395-3442`), RainbowRodHit (`:3444-3492`); each clone re-rolls a
//!   fresh NewDust. RainbowRodHit's 272-draw contract matches prefix.rs
//!   (S1-P-i45) — the wiring wave folds that count-only drain in here.
//! - 3641 — `Spawn_LightningSmall`'s gated 226/331 dust (`:3599-3653`).
//! - 3723 — `Spawn_FormatC`'s 12 type-213 sparks (`:3683-3743`).
//! - 3882/3920 — `Spawn_FlameWaders` / `Spawn_WallOfFleshGoatMountFlames`
//!   (`:3863-3937`), the Next(16)/Next(8)-gated 4×4 type-6 pair.
//! WRITTEN NOT RUN (code-only mode): the witnesses below are compiled by
//! the merge-verify wave, not executed here.

// The wiring wave landed the drains (`world_sim/projectiles.rs` +
// `world_sim/player.rs` + `world_sim/editing.rs`); the kinds whose callers
// are unported (PaladinsHammer has ZERO C# callers; the AI_186 / useStyle-5
// PrincessWeapon arms wait on their machines) keep the emitters reachable
// through `spawn_orchestra` below.
#![allow(dead_code)]

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

/// Sites 3195/3198 — `Spawn_TrueNightsEdge` (`ParticleOrchestrator.cs:
/// 3139-3203`). `num2 = 0f`; both sparkle loops step `num += 2f` over
/// `< 3f` (iterations 0 and 2 — the `num == 1f` scale arms are DEAD,
/// cited); the second loop spawns TWO 75 pairs per iteration (the inner
/// `i < 2` loop, `:3193`), both scales 1.4, no fadeIn/colour writes.
pub(crate) fn true_nights_edge(
    s: &OrchestraSettings,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let num2 = 0.0f32; // `:3142`
    // `:3143-3168` — the first sparkle loop (0, 2): zero draws.
    let mut num4 = 0.0f32;
    while num4 < 3.0 {
        // `:3169-3192` — the second sparkle loop: zero draws; the vector
        // feeds the dust pairs.
        let vector2 =
            to_rotation_vector2(std::f32::consts::PI / 4.0 + std::f32::consts::PI / 4.0 * num4 + num2) * 4.0;
        for _ in 0..2 {
            // `:3193-3201`
            blade_dust_pair(vector2, 75, 1.4, 1.4, None, None, rng, dust, ctx, world, s.position);
        }
        num4 += 2.0;
    }
}

/// The 267-sparkle dust tail shared by PaladinsHammer / PrincessWeapon /
/// StardustPunch / RainbowRodHit (`:3322-3338` et al.): per dust the
/// colour argument draws first (the caller supplies them), then the
/// `NewDust` internals, then velocity = Circular(2,2), += movement *
/// (0.5 + 0.5*NextFloat) * 1.4, noGravity, the scale write, position +=
/// Circular(16,16), and the final velocity overwrite the hammer/princess
/// arms carry. When the spawn is refused (idx == 6000) the clone is
/// skipped but the post-writes still land on the sentinel. The clone
/// re-rolls a fresh `NewDust` (6 internals), then scale /= 2, fadeIn *=
/// 0.75, colour White.
#[allow(clippy::too_many_arguments)]
fn sparkle_267_tail(
    color: Color,
    scale_write: Option<f32>,
    scale_roll: bool,
    final_velocity: Option<Vec2>,
    s: &OrchestraSettings,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let idx = dust.new_dust(rng, ctx, world, s.position, 0, 0, 267, 0.0, 0.0, 0, color, 1.0);
    {
        let d = dust.slot_mut(idx);
        d.velocity = next_vector2_circular(rng, 2.0, 2.0);
        let f = 0.5 + 0.5 * rng.next_float();
        d.velocity += s.movement * f * 1.4;
        d.no_gravity = true;
        if scale_roll {
            // StardustPunch: scale = 0.6 + NextFloat * 2 (`:3432`).
            d.scale = 0.6 + rng.next_float() * 2.0;
        } else {
            d.scale = scale_write.unwrap_or(0.1);
        }
        d.position += next_vector2_circular(rng, 16.0, 16.0);
        if let Some(v) = final_velocity {
            d.velocity = v;
        }
    }
    if idx != tr_core::particles::DUST_SENTINEL {
        let cidx = dust.clone_dust(rng, ctx, world, idx);
        let cd = dust.slot_mut(cidx);
        cd.scale /= 2.0;
        cd.fade_in *= 0.75;
        cd.color = Color::WHITE;
    }
}

/// Site 3324 — `Spawn_PaladinsHammer` (`:3286-3339`). The head phase
/// draw + the single-iteration sparkle block's 6 draws (scale, size,
/// angle, Circular(12,12)×2 — `:3288-3298`) are consumed though the
/// sparkles are the renderer skip; then 2 iterations of the 267 tail
/// with the fixed colour (1, 0.7, 0.3, 0), scale 0.1 and the final
/// velocity = movement overwrite.
pub(crate) fn paladins_hammer(
    s: &OrchestraSettings,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let _num = rng.next_float() * (std::f32::consts::PI * 2.0); // `:3288`
    // `:3290-3321` — the num3 < 1f sparkle block (one iteration):
    let _num4 = 0.6 + rng.next_float() * 0.35; // `:3292`
    let _size = rng.next_float() * 0.4 + 0.2; // `:3294`
    let _f = rng.next_float() * (std::f32::consts::PI * 2.0); // `:3295`
    let _circ = next_vector2_circular(rng, 12.0, 12.0); // `:3298`
    for _ in 0..2 {
        // `:3324` — newColor = new Color(1f, 0.7f, 0.3f, 0f).
        sparkle_267_tail(color_f(1.0, 0.7, 0.3, 0.0), Some(0.1), false, Some(s.movement), s, rng, dust, ctx, world);
    }
}

/// Site 3378 — `Spawn_PrincessWeapon` (`:3341-3393`). Same head shape as
/// the hammer but the sparkle block draws Circular(8,8) and TWO colour
/// floats (`:3347-3359`); the dust colour is the hsl roll
/// `(0.92 + NextFloat*0.02) % 1, 1, 0.4 + NextFloat*0.25` drawn as the
/// NewDust ARGUMENT (before the internals), scale 0.1, final velocity =
/// movement.
pub(crate) fn princess_weapon(
    s: &OrchestraSettings,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let _num = rng.next_float() * (std::f32::consts::PI * 2.0); // `:3343`
    // `:3345-3375` — the one-iteration sparkle block:
    let _scale = rng.next_float() * 0.35; // `:3347` (0.6 + … factor)
    let _size = rng.next_float() * 0.4 + 0.2; // `:3348`
    let _f = rng.next_float() * (std::f32::consts::PI * 2.0); // `:3349`
    let _circ = next_vector2_circular(rng, 8.0, 8.0); // `:3353`
    let _hue = rng.next_float(); // `:3359` — the sparkle tint's hue
    let _lum = rng.next_float(); // `:3359` — the sparkle tint's luminosity
    for _ in 0..2 {
        // `:3378` — the dust colour argument: two floats, then hsl.
        let h = (0.92 + rng.next_float() * 0.02) % 1.0;
        let l = 0.4 + rng.next_float() * 0.25;
        sparkle_267_tail(hsl(h, l), Some(0.1), false, Some(s.movement), s, rng, dust, ctx, world);
    }
}

/// Site 3428 — `Spawn_StardustPunch` (`:3395-3442`). The sparkle block's
/// scale factor is `0.3 + NextFloat*0.35`, size `NextFloat*0.4 + 0.4`,
/// Circular(8,8), two colour floats; the dust colour hue base is 0.59
/// (NOT the sparkle's 0.6), scale = 0.6 + NextFloat*2 (a POST-spawn roll),
/// NO final velocity overwrite (the Circular+blend velocity stays).
pub(crate) fn stardust_punch(
    s: &OrchestraSettings,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let _num = rng.next_float() * (std::f32::consts::PI * 2.0); // `:3397`
    // `:3399-3425` — the one-iteration sparkle block:
    let _scale = rng.next_float() * 0.35; // `:3401`
    let _size = rng.next_float() * 0.4 + 0.4; // `:3402`
    let _f = rng.next_float() * (std::f32::consts::PI * 2.0); // `:3403`
    let _circ = next_vector2_circular(rng, 8.0, 8.0); // `:3407`
    let _hue = rng.next_float(); // `:3411`
    let _lum = rng.next_float(); // `:3411`
    for _ in 0..2 {
        // `:3428` — the dust colour argument: hue base 0.59.
        let h = (0.59 + rng.next_float() * 0.02) % 1.0;
        let l = 0.4 + rng.next_float() * 0.25;
        sparkle_267_tail(hsl(h, l), None, true, None, s, rng, dust, ctx, world);
    }
}

/// Site 3478 — `Spawn_RainbowRodHit` (`:3444-3492`). Two head floats
/// (`:3446`/`:3448` — num, num3), then SIX sparkle iterations each
/// drawing dir + scale + angle + Circular(8,8)×2 + two colour floats
/// (`:3451-3461`); then 12 dust iterations: colour argument
/// `(num3 + NextFloat*0.12) % 1, 1, 0.4 + NextFloat*0.25`, internals,
/// velocity = Circular(1,1) += movement * NextFloatDirection * 0.5,
/// noGravity, scale = 0.6 + NextFloat*0.9, fadeIn = 0.7 + NextFloat*0.8,
/// clone (scale/2, fadeIn*0.75, White). NOTE (the wiring-wave fold,
/// research/66): this FULL emitter owns every IN-WORLD RainbowRodHit site
/// (the 79 projectile hit `Projectile.cs:14234` + death `:71316`, the
/// Kaleidoscope tag `WhipTagEffect_Kaleidoscope.cs:9`; the 1047
/// RainbowBoulder's `:71244` loop is deferred with its unported machine).
/// The REFORGE tail
/// (`Main.cs:41911-41920`) keeps `prefix.rs`'s count-only 272-draw drain
/// (S1-P-i45) as its ONE draw path — the two are disjoint by call site, so
/// no site double-draws. The count-only form is exact under the documented
/// empty-pool / on-screen assumption (`Dust.cs:97-152` cull + pressure bands
/// draw nothing then); a pool-aware fold would change the frozen reforge
/// stream and re-pin the 602 witness, so it stays count-only (recorded
/// deviation, `prefix.rs::reforge_rainbow_rod_hit_draws`).
pub(crate) fn rainbow_rod_hit(
    s: &OrchestraSettings,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let _num = rng.next_float() * (std::f32::consts::PI * 2.0); // `:3446`
    let num3 = rng.next_float(); // `:3448`
    for _ in 0..6 {
        // `:3449-3475` — the sparkle block (num4 < 1f, step 1/6):
        let _dir = rng.next_float_direction(); // `:3451`
        let _size = rng.next_float() * 0.4 + 0.4; // `:3452`
        let _f = rng.next_float() * (std::f32::consts::PI * 2.0); // `:3453`
        let _circ = next_vector2_circular(rng, 8.0, 8.0); // `:3457`
        let _hue = rng.next_float(); // `:3461`
        let _lum = rng.next_float(); // `:3461`
    }
    for _ in 0..12 {
        // `:3478` — the colour argument draws BEFORE the internals.
        let h = (num3 + rng.next_float() * 0.12) % 1.0;
        let l = 0.4 + rng.next_float() * 0.25;
        let idx = dust.new_dust(rng, ctx, world, s.position, 0, 0, 267, 0.0, 0.0, 0, hsl(h, l), 1.0);
        {
            let d = dust.slot_mut(idx);
            d.velocity = next_vector2_circular(rng, 1.0, 1.0); // `:3479`
            let dir = rng.next_float_direction(); // `:3480`
            d.velocity += s.movement * dir * 0.5;
            d.no_gravity = true; // `:3481`
            d.scale = 0.6 + rng.next_float() * 0.9; // `:3482`
            d.fade_in = 0.7 + rng.next_float() * 0.8; // `:3483`
        }
        if idx != tr_core::particles::DUST_SENTINEL {
            // `:3484-3490`
            let cidx = dust.clone_dust(rng, ctx, world, idx);
            let cd = dust.slot_mut(cidx);
            cd.scale /= 2.0;
            cd.fade_in *= 0.75;
            cd.color = Color::WHITE;
        }
    }
}

/// Site 3641 — `Spawn_LightningSmall` (`:3599-3653`). Head: phase float,
/// `Next(1,3)` count, a DISCARDED `Next(1,3)` (`:3603`); the
/// `UniqueInfoPiece != 0` arm re-packs the colour (no draws) and the
/// AddLight is the lights lane's. Per iteration (count = 1 or 2): angle
/// dir, speed float, Circular(6,6)×2, colour-lerp float (the
/// RandomizedFrame particle is the renderer skip), then a `Next(2) == 0`
/// gate: type 226 (331 when info != 0), NewDustPerfect at the particle's
/// position with its velocity, scale 0.8 → velocity = Circular(4,4) →
/// scale *= 1.25 → noLight, colour = the lerp when info != 0.
pub(crate) fn lightning_small(
    s: &OrchestraSettings,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let num = rng.next_float() * (std::f32::consts::PI * 2.0); // `:3601`
    let num2 = rng.next_range(1, 3); // `:3602`
    let _discarded = rng.next_range(1, 3); // `:3603`
    let num3 = 0.7f32; // `:3604`
    let mut color = Color::new(133, 255, 255, 0); // `:3607`
    let mut value = Color::new(15, 100, 155, 0); // `:3608`
    if s.info != 0 {
        // `:3609-3614` — PackedValue = (uint)info, A = 0, value = color*0.75.
        let packed = s.info as u32;
        color = Color::new(
            (packed & 0xFF) as u8,
            ((packed >> 8) & 0xFF) as u8,
            ((packed >> 16) & 0xFF) as u8,
            0,
        );
        value = Color::new(
            (color.r as f32 * 0.75) as u8,
            (color.g as f32 * 0.75) as u8,
            (color.b as f32 * 0.75) as u8,
            0,
        );
    }
    // `:3615` — Lighting.AddLight: the lights lane's (W4-c19/g18-g19).
    let mut num5 = 0.0f32;
    while num5 < 1.0 {
        // `:3618-3621` — the particle argument draws.
        let f = std::f32::consts::PI * 2.0 * num5 + num + rng.next_float_direction() * 0.25;
        let num6 = rng.next_float() * 4.0 + 0.1;
        let vector = next_vector2_circular(rng, 6.0, 6.0) * num3;
        let color2 = lerp_color(color, value, rng.next_float());
        // `:3622-3633` — the RandomizedFrame particle: renderer skip.
        // Its LocalPosition/Velocity feed the dust:
        let local = s.position + vector;
        let vel = to_rotation_vector2(f) * num6 * Vec2::new(1.0, 0.5) * 0.2 + s.movement;
        if rng.next_max(2) == 0 {
            // `:3634-3651`
            let dtype = if s.info != 0 { 331 } else { 226 };
            let idx = dust.new_dust_perfect(rng, ctx, world, local, dtype, Some(vel), 0, Color::default(), 1.0);
            let d = dust.slot_mut(idx);
            d.scale = 0.8; // `:3642`
            d.no_gravity = true; // `:3643`
            d.velocity = next_vector2_circular(rng, 4.0, 4.0); // `:3644`
            d.scale *= 1.25; // `:3645`
            d.no_light = true; // `:3646`
            if s.info != 0 {
                d.color = color2; // `:3647-3650`
            }
        }
        num5 += 1.0 / num2 as f32;
    }
}

/// Site 3723 — `Spawn_FormatC` (`:3683-3743`). Head: `num` = movement
/// rotation + π/2 + 0.9424779 * dir (`:3685`); the one-iteration Fading
/// block draws dir + dir + Circular(8,8)×2 (`:3690-3695`, the particles
/// are the renderer skip); then 12 type-213 dusts: the velocity ARGUMENTS
/// are `Next(-2,3)` X then Y (drawn before the internals), noGravity,
/// fadeIn = scale + 1 + 0.01*Next(0,51), noGravity again, velocity *=
/// Next(15,51)*0.01, X *= Next(25,101)*0.01, Y -= Next(15,31)*0.1,
/// position.Y -= 4, velocity = RotatedBy(num ± π/2 by the X sign),
/// position -= velocity*5, then Next(3): != 0 → noGravity false, else
/// scale *= 0.6.
pub(crate) fn format_c(
    s: &OrchestraSettings,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let num = to_rotation(s.movement) + std::f32::consts::PI / 2.0
        + 0.9424779 * rng.next_float_direction(); // `:3685`
    // `:3688-3720` — the one-iteration Fading block:
    let _dir1 = rng.next_float_direction(); // `:3690`
    let _dir2 = rng.next_float_direction(); // `:3691`
    let _circ = next_vector2_circular(rng, 8.0, 8.0); // `:3695`
    for _ in 0..12 {
        // `:3723` — the speed arguments draw X then Y.
        let sx = rng.next_range(-2, 3) as f32;
        let sy = rng.next_range(-2, 3) as f32;
        let idx = dust.new_dust(rng, ctx, world, s.position, 1, 1, 213, sx, sy, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.no_gravity = true; // `:3724`
        d.fade_in = d.scale + 1.0 + 0.01 * rng.next_range(0, 51) as f32; // `:3725`
        d.no_gravity = true; // `:3726`
        d.velocity *= rng.next_range(15, 51) as f32 * 0.01; // `:3727`
        d.velocity.x *= rng.next_range(25, 101) as f32 * 0.01; // `:3728`
        d.velocity.y -= rng.next_range(15, 31) as f32 * 0.1; // `:3729`
        d.position.y -= 4.0; // `:3730`
        // `:3731` — the spin angle picks ±π/2 by the velocity's X sign.
        let half = if d.velocity.x > 0.0 { std::f32::consts::PI / 2.0 } else { -(std::f32::consts::PI / 2.0) };
        d.velocity = rotated_by(d.velocity, (num + half) as f64);
        d.position -= d.velocity * 5.0; // `:3732`
        if rng.next_max(3) != 0 {
            d.no_gravity = false; // `:3733-3736`
        } else {
            d.scale *= 0.6; // `:3737-3740`
        }
    }
}

/// The FlameWaders / WoFGoatMountFlames shared shape
/// (`:3863-3899` / `:3901-3937`): three iterations i = -1..1, each draws
/// `NextFromList([326,327,328])` (one Next(3)), a scale float, then a
/// `Next(16)` / `Next(8)` gate; a passed gate spawns a 4×4 type-6
/// NewDustDirect alpha 100 at position + (i*5.3333335, 0), then
/// `Next(2) == 0` → noGravity + fadeIn 1.15 else scale 0.6; velocity *=
/// 0.6, Y -= 1.2, noLight, position.Y -= 4. The FlameParticle + the
/// `dust.shader` write are renderer skips (the port has no shader chain).
fn flame_mount_arm(
    gate_max: i32,
    s: &OrchestraSettings,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for i in -1..=1 {
        let _proj = rng.next_max(3); // `:3868`/`:3906` — NextFromList
        let _scale = rng.next_float() * 0.9 + 0.1; // `:3871`/`:3909`
        let vector = s.position + Vec2::new(i as f32 * 5.3333335, 0.0);
        // `:3873-3879`/`:3911-3917` — the FlameParticle: renderer skip.
        if rng.next_max(gate_max) == 0 {
            // `:3880`/`:3918`
            let idx = dust.new_dust_direct(rng, ctx, world, vector, 4, 4, 6, 0.0, 0.0, 100, Color::default(), 1.0);
            let d = dust.slot_mut(idx);
            if rng.next_max(2) == 0 {
                d.no_gravity = true;
                d.fade_in = 1.15;
            } else {
                d.scale = 0.6;
            }
            d.velocity *= 0.6;
            d.velocity.y -= 1.2;
            d.no_light = true;
            d.position.y -= 4.0;
            // `dust.shader = …` (`:3896`/`:3934`): dropped field, cited.
        }
    }
}

/// Site 3882 — `Spawn_FlameWaders` (`:3863-3899`): the Next(16) gate.
pub(crate) fn flame_waders(
    s: &OrchestraSettings,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    flame_mount_arm(16, s, rng, dust, ctx, world);
}

/// Site 3920 — `Spawn_WallOfFleshGoatMountFlames` (`:3901-3937`): the
/// Next(8) gate; the body is otherwise identical (the shader source is
/// cMount vs cFlameWaker — a dropped field either way).
pub(crate) fn wof_goat_mount_flames(
    s: &OrchestraSettings,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    flame_mount_arm(8, s, rng, dust, ctx, world);
}

/// `Utils.NextVector2FromRectangle` (`Utils.cs:1256-1259`): TWO `NextFloat`
/// draws, X FIRST — `(float)rect.X + NextFloat() * (float)rect.Width`, then
/// Y. The rect is the C#'s integer `Rectangle` (the NPC `Hitbox` / the
/// whip-tag `npcHit.Hitbox`); the caller truncates the entity box to ints
/// exactly like `Entity.getRect()`.
pub(crate) fn next_vector2_from_rectangle(
    rng: &mut UnifiedRandom,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) -> Vec2 {
    let px = x as f32 + rng.next_float() * w as f32;
    let py = y as f32 + rng.next_float() * h as f32;
    Vec2::new(px, py)
}

/// `ParticleOrchestraType` (`GameContent/Drawing/ParticleOrchestraType.cs:3-88`)
/// — the enum's byte values, in declaration order. Only the kinds this lane
/// owns emitters for are named; the dispatch below routes them.
pub(crate) mod orchestra_kind {
    pub const FLAME_WADERS: u8 = 1;
    pub const WOF_GOAT_MOUNT_FLAMES: u8 = 3;
    pub const RAINBOW_ROD_HIT: u8 = 5;
    pub const STARDUST_PUNCH: u8 = 7;
    pub const PRINCESS_WEAPON: u8 = 8;
    pub const PALADINS_HAMMER: u8 = 9;
    pub const TRUE_NIGHTS_EDGE: u8 = 12;
    pub const EXCALIBUR: u8 = 13;
    pub const TRUE_EXCALIBUR: u8 = 14;
    pub const TERRA_BLADE: u8 = 15;
    pub const SHIMMER_ARROW: u8 = 25;
    pub const BLUE_LIGHTNING_SMALL: u8 = 37;
    pub const CLASSY_CANE: u8 = 55;
    pub const FORMAT_C: u8 = 72;
    pub const RED_LIGHTNING_SMALL: u8 = 77;
}

/// `ParticleOrchestrator.SpawnParticlesDirect` (`ParticleOrchestrator.cs:162-
/// 420`) — the synchronous dispatch the `RequestParticleSpawn` head (`:52-59`)
/// calls. The port routes ONLY the kinds this lane owns (sites 41-61); every
/// other enum value is a d54 / unported `Spawn_*` body and drops here (the
/// `netmodules.rs` Particles carrier's documented drop, extended to the local
/// path). `BlueLightningSmall` (37) and `RedLightningSmall` (77) BOTH route to
/// `Spawn_LightningSmall` (`:291-292`/`:399-400`); the red caller packs
/// `GetLightningColor(1122).PackedValue` into `settings.info` (the emitter's
/// `info != 0` arm re-types 226→331 and re-colours). The draws ride the
/// CALLER's pass stream in C# statement order (the module contract).
pub(crate) fn spawn_orchestra(
    kind: u8,
    s: &OrchestraSettings,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    use orchestra_kind::*;
    match kind {
        FLAME_WADERS => flame_waders(s, rng, dust, ctx, world),
        WOF_GOAT_MOUNT_FLAMES => wof_goat_mount_flames(s, rng, dust, ctx, world),
        RAINBOW_ROD_HIT => rainbow_rod_hit(s, rng, dust, ctx, world),
        STARDUST_PUNCH => stardust_punch(s, rng, dust, ctx, world),
        PRINCESS_WEAPON => princess_weapon(s, rng, dust, ctx, world),
        PALADINS_HAMMER => paladins_hammer(s, rng, dust, ctx, world),
        TRUE_NIGHTS_EDGE => true_nights_edge(s, rng, dust, ctx, world),
        EXCALIBUR => excalibur(s, rng, dust, ctx, world),
        TRUE_EXCALIBUR => true_excalibur(s, rng, dust, ctx, world),
        TERRA_BLADE => terra_blade(s, rng, dust, ctx, world),
        SHIMMER_ARROW => shimmer_arrow(s, rng, dust, ctx, world),
        BLUE_LIGHTNING_SMALL | RED_LIGHTNING_SMALL => lightning_small(s, rng, dust, ctx, world),
        CLASSY_CANE => classy_cane(s, rng, dust, ctx, world),
        FORMAT_C => format_c(s, rng, dust, ctx, world),
        // Every other kind: a d54-reserved or unported Spawn_* body — no
        // emitter here, no draws (the C# would run its own body; the port's
        // lane boundary drops it, research/66).
        _ => {}
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
        // TerraBlade: type 107, 6 internals → 8 × (2 + 6) = 64 draws.
        {
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(37);
            terra_blade(&s, &mut rng, &mut pool, &c, &w);
            let slots = active(&pool);
            assert_eq!(slots.len(), 8);
            for &i in &slots {
                assert_eq!(pool.slot(i).dust_type, 107);
                assert!(pool.slot(i).no_gravity);
                assert_eq!(pool.slot(i).position, s.position);
            }
            let mut bare = UnifiedRandom::new(37);
            for _ in 0..8 * (2 + 6) {
                bare.next();
            }
            assert_eq!(rng.next_range(0, 1000), bare.next_range(0, 1000));
        }
        // Excalibur: type 169, 7 internals → 8 × (2 + 7) = 72 draws.
        {
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(37);
            excalibur(&s, &mut rng, &mut pool, &c, &w);
            let slots = active(&pool);
            assert_eq!(slots.len(), 8);
            for &i in &slots {
                assert_eq!(pool.slot(i).dust_type, 169);
                assert!(pool.slot(i).no_gravity);
                assert_eq!(pool.slot(i).position, s.position);
            }
            let mut bare = UnifiedRandom::new(37);
            for _ in 0..8 * (2 + 7) {
                bare.next();
            }
            assert_eq!(rng.next_range(0, 1000), bare.next_range(0, 1000));
        }
    }

    /// W4-e4 — sites 3195/3198: both loops step +2 over <3 (iterations 0
    /// and 2), the second spawns 2 pairs each → 4 pairs = 8 dusts, type
    /// 75 (tweak chain → 7 internals), 2 argument draws each = 9 × 8 = 72.
    #[test]
    fn w4_e4_true_nights_edge_steps_by_two_and_pairs_twice() {
        let w = world();
        let c = ctx();
        let s = settings();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(43);
        true_nights_edge(&s, &mut rng, &mut pool, &c, &w);
        let slots = active(&pool);
        assert_eq!(slots.len(), 8);
        for &i in &slots {
            assert_eq!(pool.slot(i).dust_type, 75);
            assert!(pool.slot(i).no_gravity);
            assert_eq!(pool.slot(i).scale, 1.4);
        }
        let mut bare = UnifiedRandom::new(43);
        for _ in 0..72 {
            bare.next();
        }
        assert_eq!(rng.next_range(0, 1000), bare.next_range(0, 1000));
    }

    /// W4-e4 — site 3478: the FULL RainbowRodHit body draws exactly 272
    /// (2 head + 6×7 sparkle + 12×19 dust), the same contract `prefix.rs`
    /// (S1-P-i45) consumes count-only — the wiring wave folds that drain
    /// into this emitter. 12 dusts + 12 clones = 24 live slots, type 267.
    #[test]
    fn w4_e4_rainbow_rod_hit_matches_the_prefix_272_contract() {
        let w = world();
        let c = ctx();
        let s = settings();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(53);
        rainbow_rod_hit(&s, &mut rng, &mut pool, &c, &w);
        let slots = active(&pool);
        assert_eq!(slots.len(), 24, "12 dusts + 12 clones");
        for &i in &slots {
            assert_eq!(pool.slot(i).dust_type, 267);
            assert!(pool.slot(i).no_gravity);
        }
        let mut bare = UnifiedRandom::new(53);
        for _ in 0..272 {
            bare.next();
        }
        assert_eq!(rng.next_range(0, 1000), bare.next_range(0, 1000));
    }

    /// W4-e4 — site 3324: 6 head draws (phase + the 5 sparkle-block:
    /// scale, size, angle, Circular(12,12)×2) + 2 × 17 tail (6 internals
    /// + 2 circular + 1 blend float + 2 circular + 6 clone internals) =
    /// 40; 2 dusts + 2 clones live, the final velocity overwrite pins
    /// movement, the clone halves scale.
    #[test]
    fn w4_e4_paladins_hammer_tail_overwrites_velocity_and_clones() {
        let w = world();
        let c = ctx();
        let s = settings();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(47);
        paladins_hammer(&s, &mut rng, &mut pool, &c, &w);
        let slots = active(&pool);
        assert_eq!(slots.len(), 4, "2 dusts + 2 clones");
        // The parent dusts (even slots) carry the movement velocity and
        // scale 0.1; the clones (odd slots) halve it and go White.
        for (k, &i) in slots.iter().enumerate() {
            let d = pool.slot(i);
            assert_eq!(d.dust_type, 267);
            assert!(d.no_gravity);
            if k % 2 == 0 {
                assert_eq!(d.velocity, s.movement);
                assert_eq!(d.scale, 0.1);
            } else {
                assert_eq!(d.scale, 0.05);
                assert_eq!(d.color, Color::WHITE);
            }
        }
        let mut bare = UnifiedRandom::new(47);
        for _ in 0..40 {
            bare.next();
        }
        assert_eq!(rng.next_range(0, 1000), bare.next_range(0, 1000));
    }

    /// W4-e4 — site 3723: 5 head draws (1 num dir + 2 block dirs + 2
    /// circular) + 12 × 13 (2 speed args + 6 internals + 5 post rolls) =
    /// 161; 12 type-213 dusts, the Next(3) tail splits noGravity/scale.
    #[test]
    fn w4_e4_format_c_spins_the_minecart_sparks() {
        let w = world();
        let c = ctx();
        let s = settings();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(61);
        format_c(&s, &mut rng, &mut pool, &c, &w);
        let slots = active(&pool);
        assert_eq!(slots.len(), 12);
        for &i in &slots {
            assert_eq!(pool.slot(i).dust_type, 213);
        }
        let mut bare = UnifiedRandom::new(61);
        for _ in 0..161 {
            bare.next();
        }
        assert_eq!(rng.next_range(0, 1000), bare.next_range(0, 1000));
    }

    /// W4-e4 — sites 3882/3920: three iterations each draw Next(3) + a
    /// scale float + the gate; a passed gate spawns one 4×4 type-6
    /// NewDustDirect (alpha 100) with the Next(2) noGravity/scale split.
    /// Twin-pool parity over both gate widths.
    #[test]
    fn w4_e4_flame_arms_gate_per_iteration() {
        let w = world();
        let c = ctx();
        let s = settings();
        for gate in [16i32, 8i32] {
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(71);
            let mut mpool = DustPool::new();
            let mut mirror = UnifiedRandom::new(71);
            if gate == 16 {
                flame_waders(&s, &mut rng, &mut pool, &c, &w);
            } else {
                wof_goat_mount_flames(&s, &mut rng, &mut pool, &c, &w);
            }
            let mut expect = 0usize;
            for i in -1..=1 {
                let _proj = mirror.next_max(3);
                let _scale = mirror.next_float() * 0.9 + 0.1;
                let vector = s.position + Vec2::new(i as f32 * 5.3333335, 0.0);
                if mirror.next_max(gate) == 0 {
                    let idx = mpool.new_dust_direct(&mut mirror, &c, &w, vector, 4, 4, 6, 0.0, 0.0, 100, Color::default(), 1.0);
                    let gate2 = mirror.next_max(2) == 0;
                    let d = pool.slot(expect);
                    assert_eq!(d.dust_type, 6);
                    assert_eq!(d.alpha, 100);
                    assert_eq!(d.no_gravity, gate2);
                    assert!(d.no_light);
                    // The emitter nudges position.Y -= 4 after the spawn.
                    assert_eq!(d.position, mpool.slot(idx).position - Vec2::new(0.0, 4.0));
                    expect += 1;
                }
            }
            assert_eq!(active(&pool).len(), expect);
            assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
        }
    }
}
