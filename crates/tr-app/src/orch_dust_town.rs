//! W4-d54 batch 8 — ParticleOrchestrator dust sites 31-40 (research/67,
//! the e4-reserved first-40 lane COMPLETE):
//! `Spawn_DeadCellsDownDashExplosion` (`ParticleOrchestrator.cs:2120-
//! 2213`, site 2144 + inline CloneDust), `Spawn_ShimmerTownNPC`
//! (`:2254-2315`, site 2293), `Spawn_Digestion` (`:2328-2347`, site
//! 2335), `Spawn_LoadOutChange` (`:2370-2399`, sites 2384/2392),
//! `Spawn_DeadCellsBarnacleShotFiring` (`:2521-2549`, sites 2530/2541),
//! `ElderSlimeEffect` (`:2551-2567`, site 2555), `NerdySlimeEffect`
//! (`:2569-2576`, site 2574) and `CopperSlimeEffect` (`:2578-2590`,
//! site 2582).
//!
//! RNG contract: the caller's pass stream (S1-A-10), argument draws
//! before the pool internals. The particle-pool requests are the
//! documented renderer skip; their ARGUMENT draws are consumed in order.
//! The gore siblings (Digestion :2341, ElderSlime :2565, DownDash :2190)
//! are g16/g17's lane — their argument draws are consumed here in order.
//!
//! WRITTEN NOT RUN (code-only mode).

use crate::bow_launch::{rotated_by, safe_normalize};
use crate::orch_dust::{next_vector2_circular, tile_coords};
use crate::ui::color::hsl_to_rgb;
use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// `Main.hslToRgb` as the dust carrier (the shared banker's-rounding
/// port, `ui/color.rs`).
fn hsl(h: f32, s: f32, l: f32, a: u8) -> Color {
    let [r, g, b, a] = hsl_to_rgb(h as f64, s as f64, l as f64, a);
    Color::new(r, g, b, a)
}

/// XNA `Color.Lerp` — per-channel float lerp, truncating cast.
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let ch = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as i32 as u8;
    Color::new(ch(a.r, b.r), ch(a.g, b.g), ch(a.b, b.b), ch(a.a, b.a))
}

/// `Utils.RotatedByRandom` (`Utils.cs:1697-1700`): two NextDouble draws.
fn rotated_by_random(v: Vec2, max_radians: f64, rng: &mut UnifiedRandom) -> Vec2 {
    let angle = rng.next_double() * max_radians - rng.next_double() * max_radians;
    rotated_by(v, angle)
}

/// `Utils.NextVector2FromRectangle` (`Utils.cs:1256-1259`).
fn from_rectangle(rng: &mut UnifiedRandom, x: f32, y: f32, w: f32, h: f32) -> Vec2 {
    Vec2::new(x + rng.next_float() * w, y + rng.next_float() * h)
}

/// `Spawn_DeadCellsDownDashExplosion` (`:2120-2213`, site 2144). `info`
/// is `settings.UniqueInfoPiece` (1 = the empowered dash). Forty
/// iterations: circular(5,3) velocity (Y flipped, ×1.5 when empowered),
/// the perfect's position argument `position + circular(10,0)`, type
/// 306, hsl(0.01/0.55, 1, 0.5), scale 1.9+0.8·NextFloat — then the
/// INLINE `CloneDust` (the C# clones the just-spawned dust; the clone's
/// scale −0.6 and white colour land on the CLONE, the source keeps its
/// own). Then the tile scan (the Flint shape with the solidTop-frameY
/// gate variant, `num7 = 3` lift arms, the `Next(3)` gore gate +
/// sparkle draws).
#[allow(clippy::too_many_arguments)]
pub(crate) fn dead_cells_down_dash_explosion(
    rng: &mut UnifiedRandom,
    gen_rng: &mut UnifiedRandom,
    disco: [u8; 3],
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    position: Vec2,
    movement: Vec2,
    info: i32,
) {
    let x = movement.x;
    let (ax, ay) = tile_coords(position + Vec2::new(-x, -160.0));
    let (bx, by) = tile_coords(position + Vec2::new(x, 160.0));
    let num = ax / 2 + bx / 2;
    let num2 = x as i32 / 2;
    let flag = info == 1;
    let num3 = if flag { 0.01 } else { 0.55 };
    let _num4 = if flag { 2.0 } else { 1.0 }; // sparkle scale (visuals dropped)
    let _num5 = if flag { 3.0 } else { 2.0 };
    for _ in 0..40 {
        let mut value = next_vector2_circular(rng, 5.0, 3.0);
        if value.y > 0.0 {
            value.y *= -1.0;
        }
        if flag {
            value *= 1.5;
        }
        let offset = next_vector2_circular(rng, 10.0, 0.0);
        let color = hsl(num3, 1.0, 0.5, 255);
        let scale = 1.9 + 0.8 * rng.next_float();
        let idx = dust.new_dust_perfect(rng, ctx, world, position + offset, 306, Some(value), 0, color, scale);
        let clone = dust.clone_dust(rng, ctx, world, idx);
        let c = dust.slot_mut(clone);
        c.scale -= 0.6;
        c.color = Color::WHITE;
    }
    let num6 = 40.0f32;
    let num7 = 3;
    for j in ax..=bx {
        for k in ay..=by {
            if Vec2::distance(position, Vec2::new(j as f32 * 16.0, k as f32 * 16.0)) > num2 as f32 {
                continue;
            }
            let Some(t) = world.tile(j, k) else { continue };
            let solid = tr_core::tiledata::TILE_SOLID.has(t.kind);
            let solid_top = tr_core::tiledata::TILE_SOLID_TOP.has(t.kind);
            let important = world.frame_important.get(t.kind as usize).copied().unwrap_or(false);
            // `:2159` — the DownDash gate variant: solidTop only skips
            // when frameY != 0; frameImportant skips unless solidTop.
            if !t.active() || !solid || (solid_top && t.frame_y != 0) || (important && !solid_top) {
                continue;
            }
            let above_solid = world
                .tile(j, k - 1)
                .is_some_and(|a| a.active() && tr_core::tiledata::TILE_SOLID.has(a.kind) && !tr_core::tiledata::TILE_SOLID_TOP.has(a.kind));
            if above_solid {
                continue;
            }
            let num8 = crate::collision::tile_dust_amount(t.kind, true, t.frame_x);
            for _ in 0..num8 {
                let idx = tr_core::worldgen_dust_tile::kill_tile_make_tile_dust(j, k, t, (disco[0], disco[1], disco[2]), rng, gen_rng, dust, ctx, world);
                let obj = dust.slot_mut(idx);
                obj.velocity.y -= 3.0 + num7 as f32 * 1.5;
                obj.velocity.y *= rng.next_float();
                obj.velocity.y *= 0.75;
                obj.scale += num7 as f32 * 0.03;
            }
            if num7 >= 2 {
                for _ in 0..num8 - 1 {
                    let idx = tr_core::worldgen_dust_tile::kill_tile_make_tile_dust(j, k, t, (disco[0], disco[1], disco[2]), rng, gen_rng, dust, ctx, world);
                    let obj2 = dust.slot_mut(idx);
                    obj2.velocity.y -= 1.0 + num7 as f32;
                    obj2.velocity.y *= rng.next_float();
                    obj2.velocity.y *= 0.75;
                }
            }
            if num8 > 0 && rng.next_max(3) != 0 {
                // `:2189-2209` — the gore sibling + sparkle (g17's).
                let _num9 = (num - j).abs() as f32 / (num6 / 2.0);
                let _gore_type = 61 + rng.next_max(3);
                let _gore_vel = rng.next_float();
                let _num10 = rng.next_range(20, 40);
                let _hue_t = rng.next_float();
                let _sx = rng.next_float();
                let _sy = rng.next_float();
            }
        }
    }
}

/// `Spawn_ShimmerTownNPC` (`:2254-2315`, site 2293). The 20-step sparkle
/// loop draws Next(20,40) + NextFloat(hsl) + 2×from-rectangle + 2×
/// NextFloat(scale) per step (visuals dropped). Then twenty type-308
/// spawns over the centered 30×60 rectangle: velocity.Y −8, X ×0.5,
/// scale 0.8, noGravity, then the `Next(6)` colour switch (0 →
/// (255,255,210), 1 → (190,245,255), 2 → (255,150,255), default →
/// (190,175,255)). Type 308's default-block tweak (`Dust.cs:225-229`:
/// velocity ×0.5, Y++) runs inside the pool BEFORE these post-arms.
pub(crate) fn shimmer_town_npc(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    position: Vec2,
) {
    let rx = (position.x - 15.0) as i32;
    let ry = (position.y - 30.0) as i32;
    for _ in 0..20 {
        let _num2 = rng.next_range(20, 40);
        let _tint = rng.next_float();
        let _p1 = from_rectangle(rng, rx as f32, ry as f32, 30.0, 60.0);
        let _sx = rng.next_float();
        let _sy = rng.next_float();
        let _p2 = from_rectangle(rng, rx as f32, ry as f32, 30.0, 60.0);
    }
    let top_left = Vec2::new(rx as f32, ry as f32);
    for _ in 0..20 {
        let idx = dust.new_dust(rng, ctx, world, top_left, 30, 60, 308, 0.0, 0.0, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.velocity.y -= 8.0;
        d.velocity.x *= 0.5;
        d.scale = 0.8;
        d.no_gravity = true;
        d.color = match rng.next_max(6) {
            0 => Color::new(255, 255, 210, 255),
            1 => Color::new(190, 245, 255, 255),
            2 => Color::new(255, 150, 255, 255),
            _ => Color::new(190, 175, 255, 255),
        };
    }
}

/// `Spawn_Digestion` (`:2328-2347`, site 2335). `num = movement.X < 0 ?
/// 1 : -1`; `Next(4)` picks 3..6 puffs at `position + (−num·8, 0) −
/// (5,5) + (0,8)`, 3×6 box, type 216, SpeedX = −num, SpeedY = 1; post:
/// velocity /2, scale 0.8. The `Next(30) == 0` gore arm draws Next(580,
/// 583) (g17's, consumed). Type 216 adds no per-type tweak.
pub(crate) fn digestion(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    position: Vec2,
    movement_x: f32,
) {
    let num = if movement_x < 0.0 { 1 } else { -1 };
    let num2 = rng.next_max(4);
    let pos = position + Vec2::new(-num as f32 * 8.0, 0.0) - Vec2::splat(5.0) + Vec2::new(0.0, 8.0);
    for _ in 0..3 + num2 {
        let idx = dust.new_dust(rng, ctx, world, pos, 3, 6, 216, -num as f32, 1.0, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.velocity /= 2.0;
        d.scale = 0.8;
    }
    if rng.next_max(30) == 0 {
        let _gore_type = rng.next_range(580, 583);
    }
}

/// The LoadOutChange player inputs: the hitbox (the port's 20×42 box at
/// (x,y)), `gravDir` and the velocity. The C# shrinks the hitbox height
/// by 6 and shifts Y +6 when gravDir == 1.
pub(crate) struct LoadOutPlayer {
    pub hitbox_x: f32,
    pub hitbox_y: f32,
    pub hitbox_w: f32,
    pub hitbox_h: f32,
    pub grav_dir: f32,
    pub velocity: Vec2,
}

/// `Spawn_LoadOutChange` (`:2370-2399`, sites 2384/2392). Forty type-16
/// perfects: position = from-rectangle(hitbox) (2 NextFloat), velocity
/// null, alpha 120, scale NextFloat·0.8+0.8; post: velocity =
/// (0, −height·NextFloat·0.04).RotatedBy(NextFloatDirection·2π·0.1) +
/// player.velocity·2·NextFloat, noGravity, noLight + noLightEmittance.
/// Then five type-43 perfects with the hsl(NextFloat, 0.3, 0.8) colour
/// argument (drawn BEFORE the scale argument) and alpha 254.
pub(crate) fn load_out_change(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    player: &LoadOutPlayer,
) {
    let pi = std::f32::consts::PI;
    let mut hy = player.hitbox_y;
    let hw = player.hitbox_w;
    let mut hh = player.hitbox_h;
    hh -= 6.0;
    if player.grav_dir == 1.0 {
        hy += 6.0;
    }
    let hx = player.hitbox_x;
    for _ in 0..40 {
        let pos = from_rectangle(rng, hx, hy, hw, hh);
        let scale = rng.next_float() * 0.8 + 0.8;
        let idx = dust.new_dust_perfect(rng, ctx, world, pos, 16, None, 120, Color::default(), scale);
        let d = dust.slot_mut(idx);
        let up = -hh * rng.next_float() * 0.04;
        let angle = rng.next_float_direction() * (pi * 2.0) * 0.1;
        d.velocity = rotated_by(Vec2::new(0.0, up), angle as f64);
        d.velocity += player.velocity * 2.0 * rng.next_float();
        d.no_gravity = true;
        d.no_light = true;
        d.no_light_emittance = true;
    }
    for _ in 0..5 {
        let pos = from_rectangle(rng, hx, hy, hw, hh);
        let color = hsl(rng.next_float(), 0.3, 0.8, 255);
        let scale = rng.next_float() * 0.8 + 0.8;
        let idx = dust.new_dust_perfect(rng, ctx, world, pos, 43, None, 254, color, scale);
        let d = dust.slot_mut(idx);
        let up = -hh * rng.next_float() * 0.04;
        let angle = rng.next_float_direction() * (pi * 2.0) * 0.1;
        d.velocity = rotated_by(Vec2::new(0.0, up), angle as f64);
        d.velocity += player.velocity * 2.0 * rng.next_float();
        d.no_gravity = true;
        d.no_light = true;
        d.no_light_emittance = true;
    }
}

/// `Spawn_DeadCellsBarnacleShotFiring` (`:2521-2549`, sites 2530/2541).
/// Fifteen type-2 perfects: position + (Next(-4,4), Next(-4,4)),
/// velocity = SafeNormalize(movement).RotatedByRandom(0.19634954…) ·
/// (i >= 8 ? 3+NextFloat·2 : 7+NextFloat·2); post: alpha 50, scale
/// 1.2+Next(-5,5)·0.01, fadeIn 0.5, noGravity, velocity re-write (the
/// C#'s own duplicate). Then five type-267 perfects: the 0.09817477…
/// spin, radius NextFloat·8, alpha 127, colour Lerp(DarkOliveGreen
/// (85,107,47), White, NextFloat·0.25), fadeIn 0.6+NextFloat·0.6,
/// scale 0.5, noGravity, velocity re-write.
pub(crate) fn barnacle_shot_firing(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    position: Vec2,
    movement: Vec2,
) {
    let norm = safe_normalize(movement, Vec2::ZERO);
    for i in 0..15 {
        let pos = position + Vec2::new(rng.next_range(-4, 4) as f32, rng.next_range(-4, 4) as f32);
        let spun = rotated_by_random(norm, 0.19634954631328583, rng);
        let radius = if i >= 8 { 3.0 + rng.next_float() * 2.0 } else { 7.0 + rng.next_float() * 2.0 };
        let vector = spun * radius;
        let idx = dust.new_dust_perfect(rng, ctx, world, pos, 2, Some(vector), 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.alpha = 50;
        d.scale = 1.2 + rng.next_range(-5, 5) as f32 * 0.01;
        d.fade_in = 0.5;
        d.no_gravity = true;
        d.velocity = vector;
    }
    for _ in 0..5 {
        let pos = position + Vec2::new(rng.next_range(-4, 4) as f32, rng.next_range(-4, 4) as f32);
        let spun = rotated_by_random(norm, 0.09817477315664291, rng);
        let vector2 = spun * (0.0 + rng.next_float() * 8.0);
        let idx = dust.new_dust_perfect(rng, ctx, world, pos, 267, Some(vector2), 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.alpha = 127;
        d.color = lerp_color(Color::new(85, 107, 47, 255), Color::WHITE, rng.next_float() * 0.25);
        d.fade_in = 0.6 + rng.next_float() * 0.6;
        d.scale = 0.5;
        d.no_gravity = true;
        d.velocity = vector2;
    }
}

/// `Main.OurFavoriteColor` (`Main.cs:66748`): (255, 231, 69).
const OUR_FAVORITE_COLOR: Color = Color::new(255, 231, 69, 255);

/// The slime-transform trio's shared perfect shape (`:2555`/`:2574`/
/// `:2582`): position (+circular(20,20) for Elder/Copper), velocity =
/// (movement·0.75 + circular(6,6))·NextFloat, alpha, Lerp colour, scale
/// 1+NextFloat·1.4 (Nerdy: 0.6+…). Elder/Copper add fadeIn 1.5 + the
/// short-circuit Y-flip gate + noGravity; Nerdy has no post-arms.
fn slime_perfect(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    position: Vec2,
    movement: Vec2,
    dust_type: i32,
    alpha: i32,
    base_color: Color,
    lerp_scale: f32,
    scale_base: f32,
    offset_box: bool,
    post_arms: bool,
) {
    let pos = if offset_box {
        position + next_vector2_circular(rng, 20.0, 20.0)
    } else {
        position
    };
    let vel = (movement * 0.75 + next_vector2_circular(rng, 6.0, 6.0)) * rng.next_float();
    let color = lerp_color(base_color, Color::WHITE, rng.next_float() * lerp_scale);
    let scale = scale_base + rng.next_float() * 1.4;
    let idx = dust.new_dust_perfect(rng, ctx, world, pos, dust_type, Some(vel), alpha, color, scale);
    if post_arms {
        let d = dust.slot_mut(idx);
        d.fade_in = 1.5;
        if d.velocity.y > 0.0 && rng.next_max(2) == 0 {
            d.velocity.y *= -1.0;
        }
        d.no_gravity = true;
    }
}

/// `ElderSlimeEffect` (`:2551-2567`, site 2555): thirty type-43 perfects
/// (alpha 26, Lerp(OurFavoriteColor, White, NextFloat)), then eight gore
/// siblings (`:2563-2566`, g17's — RandomVector2's two NextDouble +
/// Next(3) per gore, consumed in order).
pub(crate) fn elder_slime_effect(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    position: Vec2,
    movement: Vec2,
) {
    for _ in 0..30 {
        slime_perfect(rng, dust, ctx, world, position, movement, 43, 26, OUR_FAVORITE_COLOR, 1.0, 1.0, true, true);
    }
    for _ in 0..8 {
        rng.next_double();
        rng.next_double();
        let _gore_type = 61 + rng.next_max(3);
    }
}

/// `NerdySlimeEffect` (`:2569-2576`, site 2574): sixty type-4 perfects,
/// alpha 175, colour (0,80,255,100) — the C# passes `newColor` directly
/// (no Lerp; the helper's lerp_scale 0 keeps the base colour, but the
/// C# draws NO NextFloat for the colour here, so the shared helper is
/// NOT used — the body is inline).
pub(crate) fn nerdy_slime_effect(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    position: Vec2,
    movement: Vec2,
) {
    let new_color = Color::new(0, 80, 255, 100);
    for _ in 0..60 {
        let vel = (movement * 0.75 + next_vector2_circular(rng, 6.0, 6.0)) * rng.next_float();
        let scale = 0.6 + rng.next_float() * 1.4;
        dust.new_dust_perfect(rng, ctx, world, position, 4, Some(vel), 175, new_color, scale);
    }
}

/// `CopperSlimeEffect` (`:2578-2590`, site 2582): forty type-43 perfects,
/// alpha 26, Lerp((183,88,25), White, NextFloat·0.5).
pub(crate) fn copper_slime_effect(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    position: Vec2,
    movement: Vec2,
) {
    for _ in 0..40 {
        slime_perfect(rng, dust, ctx, world, position, movement, 43, 26, Color::new(183, 88, 25, 255), 0.5, 1.0, true, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dust_ctx(w: &World) -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(w.width as f32 * 16.0, w.height as f32 * 16.0),
            max_dust_to_draw: 6000,
        }
    }

    fn block(r: &mut UnifiedRandom, dust_type: i32, box_w: i32, box_h: i32) {
        r.next_max(box_w.max(5) - 4);
        r.next_max(box_h.max(5) - 4);
        r.next_range(-20, 21);
        r.next_range(-20, 21);
        r.next_max(3);
        r.next_range(-20, 21);
        if matches!(dust_type, 6 | 228) {
            r.next_range(-10, 6);
        }
    }

    fn circular(r: &mut UnifiedRandom) {
        r.next_float();
        r.next_float();
    }

    fn drain(dust: &mut DustPool) {
        for s in dust.slots_mut() {
            s.active = false;
        }
    }

    /// `:2291-2313` — the sparkle loop (20×6 draws) then 20×(type-308
    /// 30×60 block + Next(6) colour switch).
    #[test]
    fn w4_d54_shimmer_town_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let pos = Vec2::new(1600.0, 1600.0);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        for i in 0..50 {
            shimmer_town_npc(&mut rng, &mut dust, &ctx, &w, pos);
            for _ in 0..20 {
                probe.next_range(20, 40);
                // hsl(1) + from-rect(2) + scale(2) + from-rect(2) = 7.
                for _ in 0..7 {
                    probe.next_float();
                }
            }
            for _ in 0..20 {
                block(&mut probe, 308, 30, 60);
                probe.next_max(6);
            }
            assert_eq!(rng.peek(), probe.peek(), "shimmer-town stream @{i}");
            drain(&mut dust);
        }
    }

    /// `:2332-2345` — Next(4) count, (3+n)×(3×6 type-216 block), the
    /// Next(30) gore gate (+ Next(580,583) when it fires).
    #[test]
    fn w4_d54_digestion_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let pos = Vec2::new(1600.0, 1600.0);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        for i in 0..100 {
            digestion(&mut rng, &mut dust, &ctx, &w, pos, -3.0);
            let n = probe.next_max(4);
            for _ in 0..3 + n {
                block(&mut probe, 216, 3, 6);
            }
            if probe.next_max(30) == 0 {
                probe.next_range(580, 583);
            }
            assert_eq!(rng.peek(), probe.peek(), "digestion stream @{i}");
            drain(&mut dust);
        }
    }

    /// `:2382-2397` — 40×(2 from-rect + scale + type-16 block + up +
    /// direction + velocity NextFloat) then 5×(2 from-rect + hsl +
    /// scale + type-43 block + up + direction + velocity).
    #[test]
    fn w4_d54_load_out_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let p = LoadOutPlayer {
            hitbox_x: 1600.0, hitbox_y: 1600.0, hitbox_w: 20.0, hitbox_h: 42.0,
            grav_dir: 1.0, velocity: Vec2::new(1.0, 0.0),
        };
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        for i in 0..50 {
            load_out_change(&mut rng, &mut dust, &ctx, &w, &p);
            for _ in 0..40 {
                probe.next_float();
                probe.next_float();
                probe.next_float();
                block(&mut probe, 16, 0, 0);
                probe.next_float();
                probe.next_float_direction();
                probe.next_float();
            }
            for _ in 0..5 {
                probe.next_float();
                probe.next_float();
                probe.next_float();
                probe.next_float();
                block(&mut probe, 43, 0, 0);
                probe.next_float();
                probe.next_float_direction();
                probe.next_float();
            }
            assert_eq!(rng.peek(), probe.peek(), "load-out stream @{i}");
            drain(&mut dust);
        }
    }

    /// `:2526-2548` — 15×(2 Next(-4,4) + 2 NextDouble + radius NextFloat
    /// + type-2 block + Next(-5,5)) then 5×(2 Next(-4,4) + 2 NextDouble
    /// + radius + type-267 block + lerp NextFloat + fadeIn NextFloat).
    #[test]
    fn w4_d54_barnacle_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let pos = Vec2::new(1600.0, 1600.0);
        let mv = Vec2::new(6.0, -2.0);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        for i in 0..50 {
            barnacle_shot_firing(&mut rng, &mut dust, &ctx, &w, pos, mv);
            for _ in 0..15 {
                probe.next_range(-4, 4);
                probe.next_range(-4, 4);
                probe.next_double();
                probe.next_double();
                probe.next_float();
                block(&mut probe, 2, 0, 0);
                probe.next_range(-5, 5);
            }
            for _ in 0..5 {
                probe.next_range(-4, 4);
                probe.next_range(-4, 4);
                probe.next_double();
                probe.next_double();
                probe.next_float();
                block(&mut probe, 267, 0, 0);
                probe.next_float();
                probe.next_float();
            }
            assert_eq!(rng.peek(), probe.peek(), "barnacle stream @{i}");
            drain(&mut dust);
        }
    }

    /// `:2572-2575` — sixty (circular + NextFloat velocity + NextFloat
    /// scale + type-4 block) quadruples, NO colour draw (the C# passes
    /// the constant newColor).
    #[test]
    fn w4_d54_nerdy_slime_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let pos = Vec2::new(1600.0, 1600.0);
        let mv = Vec2::new(2.0, 0.0);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        for i in 0..50 {
            nerdy_slime_effect(&mut rng, &mut dust, &ctx, &w, pos, mv);
            for _ in 0..60 {
                circular(&mut probe);
                probe.next_float();
                probe.next_float();
                block(&mut probe, 4, 0, 0);
            }
            assert_eq!(rng.peek(), probe.peek(), "nerdy stream @{i}");
            drain(&mut dust);
        }
    }

    /// `:2580-2589` — forty (offset circular + velocity circular +
    /// NextFloat + lerp NextFloat + scale NextFloat + type-43 block +
    /// gate Next(2) when Y>0) — the gate mirrored through a real spawn.
    #[test]
    fn w4_d54_copper_slime_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let pos = Vec2::new(1600.0, 1600.0);
        let mv = Vec2::new(2.0, 0.0);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        let mut mirror = DustPool::new();
        for i in 0..50 {
            copper_slime_effect(&mut rng, &mut dust, &ctx, &w, pos, mv);
            for _ in 0..40 {
                let offset = next_vector2_circular(&mut probe, 20.0, 20.0);
                let vel = (mv * 0.75 + next_vector2_circular(&mut probe, 6.0, 6.0)) * probe.next_float();
                let _t = probe.next_float();
                let _s = probe.next_float();
                let idx = mirror.new_dust_perfect(&mut probe, &ctx, &w, pos + offset, 43, Some(vel), 26, Color::default(), 1.0);
                if mirror.slot(idx).velocity.y > 0.0 {
                    probe.next_max(2);
                }
            }
            assert_eq!(rng.peek(), probe.peek(), "copper stream @{i}");
            drain(&mut dust);
            drain(&mut mirror);
        }
    }
}
