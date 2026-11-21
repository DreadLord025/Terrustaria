//! W4-d54 batch 7 — ParticleOrchestrator dust sites 21-30 (research/67):
//! `Spawn_RainbowBoulder4` (`ParticleOrchestrator.cs:1431-1500`, site
//! 1446), `Spawn_RainbowBoulder1` (`:1502-1556`, site 1545),
//! `Spawn_RainbowBoulder3` (`:1585-1646`, site 1628 + clone :1638),
//! `Spawn_MoonLordWhip` (`:1648-1703`, sites 1693/1697),
//! `Spawn_DeadCellsBarrelExplosion` (`:1705-1779`, site 1775),
//! `Spawn_DeadCellsFlint` (`:1781-1889`, site 1886),
//! `SpawnHelper_SpawnSingleLineDust` (`:2016-2026`, site 2018) and
//! `Spawn_ShadowOrbExplosion` (`:2028-2118`, sites 2088/2091).
//!
//! RNG contract: the caller's pass stream (S1-A-10), argument draws
//! before the pool internals. The particle-pool requests are the
//! documented renderer skip; their ARGUMENT draws are consumed in order
//! (the C# evaluates them even when the visual write is dropped — the
//! RainbowBoulder1/3 loops keep their dead first-write draws).
//!
//! WRITTEN NOT RUN (code-only mode).

use crate::bow_launch::{rotated_by, safe_normalize};
use crate::orch_dust::{next_vector2_circular, tile_coords};
use crate::ui::color::hsl_to_rgb;
use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// `Main.hslToRgb(h, s, l, a)` (`Main.cs:47890-47917`) as the dust
/// carrier — the shared banker's-rounding port (`ui/color.rs`).
fn hsl(h: f32, s: f32, l: f32, a: u8) -> Color {
    let [r, g, b, a] = hsl_to_rgb(h as f64, s as f64, l as f64, a);
    Color::new(r, g, b, a)
}

/// `Utils.NextVector2CircularEdge` (`Utils.cs:1271-1274`): the unit draw
/// ONLY (one NextFloat), no radius scale.
fn circular_edge(rng: &mut UnifiedRandom, hw: f32, hh: f32) -> Vec2 {
    let angle = std::f32::consts::PI * 2.0 * rng.next_float();
    let unit = Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32);
    Vec2::new(unit.x * hw, unit.y * hh)
}

/// `Utils.NextVector2FromRectangle` (`Utils.cs:1256-1259`): two NextFloat
/// draws, `X + f·Width` / `Y + f·Height`.
fn from_rectangle(rng: &mut UnifiedRandom, x: f32, y: f32, w: f32, h: f32) -> Vec2 {
    Vec2::new(x + rng.next_float() * w, y + rng.next_float() * h)
}

/// `Utils.CenteredRectangle` (`Utils.cs:947-950`): the truncating int
/// casts of `center − size/2`.
fn centered_rectangle(center: Vec2, size: Vec2) -> (i32, i32, i32, i32) {
    (
        (center.x - size.x / 2.0) as i32,
        (center.y - size.y / 2.0) as i32,
        size.x as i32,
        size.y as i32,
    )
}

/// `Spawn_RainbowBoulder4` (`:1431-1500`, site 1446). `pet` picks 8/10
/// vs 14/20 counts and 1.2 vs 1.7 scale. Per dust: the hsl colour
/// argument (one NextFloat) BEFORE the 0×0 type-66 spawn (alpha 100),
/// noGravity, velocity ×2 (pet) / ×3, then the position REPLACEMENT
/// `position + circular(30,30)`. The 20/12-iteration sparkle loop draws
/// Next(20,40) + NextFloat(hsl) + 2×from-rectangle + 2×NextFloat(scale)
/// per step (visuals dropped).
#[allow(clippy::too_many_arguments)]
pub(crate) fn rainbow_boulder4(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    position: Vec2,
    movement: Vec2,
    pet: bool,
) {
    let (num, num2) = if pet { (8, 10) } else { (14, 20) };
    for _ in 0..num {
        let color = hsl(rng.next_float(), 1.0, 0.5, 255);
        let idx = dust.new_dust(rng, ctx, world, position, 0, 0, 66, 0.0, 0.0, 100, color, if pet { 1.2 } else { 1.7 });
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        if pet {
            d.velocity *= 2.0;
        } else {
            d.velocity *= 3.0;
        }
        d.position = position + next_vector2_circular(rng, 30.0, 30.0);
    }
    // `:1459-1499` — the sparkle loop (RNG-bearing arguments, visuals dropped).
    let num4 = if pet { 12.0 } else { 20.0 };
    let (rx, ry, rw, rh) = centered_rectangle(position, Vec2::splat(num4));
    let _num5 = movement.y.atan2(movement.x) + std::f32::consts::PI / 2.0;
    for _ in 0..num2 {
        let _num7 = rng.next_range(20, 40);
        let _tint = hsl(rng.next_float(), 1.0, 0.5, 0);
        let _p1 = from_rectangle(rng, rx as f32, ry as f32, rw as f32, rh as f32);
        let _sx = 1.0 + rng.next_float();
        let _sy = 0.7 + rng.next_float() * 0.7;
        let _p2 = from_rectangle(rng, rx as f32, ry as f32, rw as f32, rh as f32);
    }
}

/// `Spawn_RainbowBoulder1` (`:1502-1556`, site 1545). Head: NextFloat
/// (angle), Next(7) (count 6+n), NextFloat (hue), Next(2) (dead draw).
/// The sparkle loop (`num2 = 6+Next(7)` steps) keeps its DEAD first-write
/// draws: NextFloat (vector), NextFloat (num5, overwritten), NextFloat
/// (vector2), Next(20) (num6), circular(16,16) (vector3), circular-edge
/// (vector2 overwrite), then the two hsl colour writes' 2+2 NextFloat.
/// Then 21 type-66 Directs: hsl NextFloat argument, 16×16 alpha-100
/// spawn, noGravity, velocity = circular-edge(1,1)·(8+2·NextFloat),
/// fadeIn = 1+NextFloat, X zeroed, Y flipped positive→negative,
/// position −velocity·5.
pub(crate) fn rainbow_boulder1(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    position: Vec2,
) {
    let _num = rng.next_float() * (std::f32::consts::PI * 2.0);
    let num2 = 6 + rng.next_max(7);
    let _num3 = rng.next_float();
    rng.next_max(2);
    let mut num4 = 0.0f32;
    while num4 < 1.0 {
        let _vector = rng.next_float() * 0.3 + 1.0;
        let _num5_dead = rng.next_float();
        let _vector2_dead = rng.next_float();
        let _num6 = rng.next_max(20) + 180;
        let _vector3 = next_vector2_circular(rng, 16.0, 16.0);
        let _vector2 = circular_edge(rng, 1.5, 1.5);
        // `:1527` draws TWO NextFloat; `:1528`'s overwrite `(num4+num3)%1`
        // draws NONE (the dead first write still consumed its two).
        let _c1a = rng.next_float();
        let _c1b = rng.next_float();
        num4 += 1.0 / num2 as f32;
    }
    for _ in 0..21 {
        let color = hsl(rng.next_float(), 1.0, 0.5, 255);
        let idx = dust.new_dust_direct(rng, ctx, world, position, 16, 16, 66, 0.0, 0.0, 100, color, 1.7);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity = circular_edge(rng, 1.0, 1.0) * (8.0 + 2.0 * rng.next_float());
        d.fade_in = 1.0 + rng.next_float();
        d.velocity.x = 0.0;
        if d.velocity.y > 0.0 {
            d.velocity.y *= -1.0;
        }
        d.position -= d.velocity * 5.0;
    }
}

/// `Spawn_RainbowBoulder3` (`:1585-1646`, site 1628 + clone :1638).
/// Head: NextFloat (angle), Next(7) (count 3+n), NextFloat (hue num3),
/// Next(2) (the ±π/2 spin). The sparkle loop keeps its dead draws:
/// NextFloatDirection (vector), NextFloat (vector2), NextFloat (num6
/// dead), NextFloat (vector3), Next(20) (num7), NextFloat (vector4
/// radius), then the two hsl writes' 2+2 NextFloat. Then four type-267
/// 0×0 spawns with the hsl argument `(num3 + NextFloat·0.12) % 1`,
/// luminosity `0.4 + NextFloat·0.25`; post: velocity = circular(1,1) +
/// movement·NextFloatDirection·0.5, noGravity, scale 0.6+NextFloat·0.9,
/// fadeIn 0.7+NextFloat·0.8, noLight + noLightEmittance; the `num8 !=
/// 6000` CloneDust sibling halves the scale, ×0.75 fadeIn, white.
pub(crate) fn rainbow_boulder3(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    position: Vec2,
    movement: Vec2,
) {
    let _num = rng.next_float() * (std::f32::consts::PI * 2.0);
    let num2 = 3 + rng.next_max(7);
    let num3 = rng.next_float();
    let _num4 = std::f32::consts::PI / 2.0 * (rng.next_max(2) * 2 - 1) as f32;
    let mut num5 = 0.0f32;
    while num5 < 1.0 {
        let _vector = movement * rng.next_float_direction() * 0.15;
        let _vector2 = rng.next_float() * 0.2 + 0.3;
        let _num6_dead = rng.next_float();
        let _vector3 = 0.3 + 1.5 * rng.next_float();
        let _num7 = rng.next_max(20) + 70;
        let _radius = 14.0 - num2 as f32 + rng.next_float() * 3.0;
        // `:1604` draws TWO NextFloat; `:1605`'s overwrite draws NONE.
        let _c1a = rng.next_float();
        let _c1b = rng.next_float();
        num5 += 1.0 / num2 as f32;
    }
    for _ in 0..4 {
        let hue = (num3 + rng.next_float() * 0.12) % 1.0;
        let lum = 0.4 + rng.next_float() * 0.25;
        let color = hsl(hue, 1.0, lum, 255);
        let num8 = dust.new_dust(rng, ctx, world, position, 0, 0, 267, 0.0, 0.0, 0, color, 1.0);
        let d = dust.slot_mut(num8);
        d.velocity = next_vector2_circular(rng, 1.0, 1.0);
        d.velocity += movement * rng.next_float_direction() * 0.5;
        d.no_gravity = true;
        d.scale = 0.6 + rng.next_float() * 0.9;
        d.fade_in = 0.7 + rng.next_float() * 0.8;
        d.no_light = true;
        d.no_light_emittance = true;
        if num8 != 6000 {
            let clone = dust.clone_dust(rng, ctx, world, num8);
            let c = dust.slot_mut(clone);
            c.scale /= 2.0;
            c.fade_in *= 0.75;
            c.color = Color::new(255, 255, 255, 255);
            c.no_light = true;
            c.no_light_emittance = true;
        }
    }
}

/// `Spawn_MoonLordWhip` (`:1648-1703`, sites 1693/1697). Head: one
/// NextFloat (the dead num2 write, overwritten by the movement
/// rotation). `vector2 = ToRotationVector2(rotation)·4`; the dust pair
/// mirrors the SnappingStoneMelee shape with type 229, scale 1.2 and
/// `position += velocity·4` on BOTH.
pub(crate) fn moon_lord_whip(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    position: Vec2,
    movement: Vec2,
) {
    let pi = std::f32::consts::PI;
    let _num2_dead = pi * 2.0 * rng.next_float();
    let norm = safe_normalize(movement, Vec2::ZERO);
    let num2 = norm.y.atan2(norm.x);
    let vector2 = Vec2::new((num2 as f64).cos() as f32, (num2 as f64).sin() as f32) * 4.0;
    for _ in 0..2 {
        let angle = rng.next_float_direction() * (pi * 2.0) * 0.025;
        let vel = rotated_by(vector2, angle as f64) * rng.next_float();
        let idx = dust.new_dust_perfect(rng, ctx, world, position, 229, Some(vel), 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.scale = 1.2;
        d.position += d.velocity * 4.0;
        let angle2 = rng.next_float_direction() * (pi * 2.0) * 0.025;
        let vel2 = rotated_by(-vector2, angle2 as f64) * rng.next_float();
        let idx2 = dust.new_dust_perfect(rng, ctx, world, position, 229, Some(vel2), 0, Color::default(), 1.0);
        let d2 = dust.slot_mut(idx2);
        d2.no_gravity = true;
        d2.scale = 1.2;
        d2.position += d2.velocity * 4.0;
    }
}

/// `Spawn_DeadCellsBarrelExplosion` (`:1705-1779`, site 1775). The four
/// head FadingParticles draw NextFloat + NextFloatDirection (`:1718-
/// 1719`); the 25-step spark loop (`num2 += 0.04`) draws 3 NextFloat per
/// step (`:1762` ×2, `:1770`). Then seven type-267 perfects (velocity
/// Zero, colour (100,200,255,127), scale 2): noGravity, velocity =
/// (0,12).RotatedBy(2π·num5)·NextFloat.
pub(crate) fn dead_cells_barrel_explosion(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    position: Vec2,
) {
    let _rot = rng.next_float();
    let _rot_vel = rng.next_float_direction();
    let mut num2 = 0.0f32;
    while num2 < 1.0 {
        let _v = rng.next_float();
        let _p = rng.next_float();
        let _s = rng.next_float();
        num2 += 0.04;
    }
    let mut num5 = 0.0f32;
    while num5 < 1.0 {
        let idx = dust.new_dust_perfect(
            rng, ctx, world, position, 267, Some(Vec2::ZERO), 0, Color::new(100, 200, 255, 127), 2.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        let angle = (std::f32::consts::PI * 2.0 * num5) as f64;
        d.velocity = rotated_by(Vec2::new(0.0, 12.0), angle) * rng.next_float();
        num5 += 1.0 / 7.0;
    }
}

/// `Spawn_DeadCellsFlint` (`:1781-1889`, site 1886). `movement` is
/// `settings.MovementVector` (X = the half-width seed, Y = the gore
/// direction scalar, 0 → 1). The tile scan walks the ±160px box expanded
/// by 3 tiles, gated on distance ≤ `(int)x/2 + 24`, solid-not-top-not-
/// frame-important tiles with a non-solid cell above; `flag` marks any
/// hit. The per-tile dust arms mirror the StormLightning shape with
/// `num8 = 3` (the second loop gated `num8 >= 2`); the gore sibling
/// (`:1847-1870`) draws Next(2) gate + Next(3) type + NextFloat velocity
/// + Next(20,40) + 2 NextFloat scale (g17's, visuals dropped). When no
/// tile hit, the C# returns BEFORE the 30-dust tail. The tail: per dust
/// circular(5,3) (Y flipped), then the perfect's position argument
/// `position + circular(7,0)`, type 228, scale 0.7+0.8·NextFloat.
#[allow(clippy::too_many_arguments)]
pub(crate) fn dead_cells_flint(
    rng: &mut UnifiedRandom,
    gen: &mut UnifiedRandom,
    disco: [u8; 3],
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    position: Vec2,
    movement: Vec2,
) {
    let mut num = movement.y;
    if num == 0.0 {
        num = 1.0;
    }
    let x = movement.x;
    let mut flag = false;
    let (mut ax, mut ay) = tile_coords(position + Vec2::new(-x, -160.0));
    let (mut bx, mut by) = tile_coords(position + Vec2::new(x, 160.0));
    let num2 = 3;
    ax -= num2;
    ay -= num2;
    bx += num2;
    by += num2;
    let num3 = ax / 2 + bx / 2;
    let num4 = x as i32 / 2 + 8 * num2;
    let num7 = 40.0f32;
    let num8 = 3;
    for i in ax..=bx {
        for j in ay..=by {
            if Vec2::distance(position, Vec2::new(i as f32 * 16.0, j as f32 * 16.0)) > num4 as f32 {
                continue;
            }
            let Some(t) = world.tile(i, j) else { continue };
            let solid = tr_core::tiledata::TILE_SOLID.has(t.kind);
            let solid_top = tr_core::tiledata::TILE_SOLID_TOP.has(t.kind);
            let important = world.frame_important.get(t.kind as usize).copied().unwrap_or(false);
            if !t.active() || !solid || solid_top || important {
                continue;
            }
            let above_solid = world
                .tile(i, j - 1)
                .is_some_and(|a| a.active() && tr_core::tiledata::TILE_SOLID.has(a.kind) && !tr_core::tiledata::TILE_SOLID_TOP.has(a.kind));
            if above_solid {
                continue;
            }
            flag = true;
            if i <= ax + num2 && i >= ax - num2 && j <= ay + num2 && j >= by - num2 {
                continue;
            }
            let num9 = crate::world_dust_tile::tile_dust_amount(true, t.kind, t.frame_x);
            for _ in 0..num9 {
                let idx = crate::world_dust_tile::make_tile_dust(rng, gen, disco, dust, ctx, world, i, j, t);
                let obj = dust.slot_mut(idx);
                obj.velocity.y -= 3.0 + num8 as f32 * 1.5;
                obj.velocity.y *= rng.next_float();
                obj.velocity.y *= 0.75;
                obj.scale += num8 as f32 * 0.03;
            }
            if num8 >= 2 {
                for _ in 0..num9 - 1 {
                    let idx = crate::world_dust_tile::make_tile_dust(rng, gen, disco, dust, ctx, world, i, j, t);
                    let obj2 = dust.slot_mut(idx);
                    obj2.velocity.y -= 1.0 + num8 as f32;
                    obj2.velocity.y *= rng.next_float();
                    obj2.velocity.y *= 0.75;
                }
            }
            if num9 > 0 && rng.next_max(2) != 0 {
                // `:1849-1870` — the gore sibling + its sparkle (g17's).
                let _num10 = (num3 - i).abs() as f32 / (num7 / 2.0);
                let _gore_type = 61 + rng.next_max(3);
                let _gore_vel = rng.next_float();
                let _num11 = rng.next_range(20, 40);
                let _sx = rng.next_float();
                let _sy = rng.next_float();
            }
        }
    }
    if !flag {
        return;
    }
    for _ in 0..30 {
        let mut value = next_vector2_circular(rng, 5.0, 3.0);
        if value.y > 0.0 {
            value.y *= -1.0;
        }
        let offset = next_vector2_circular(rng, 7.0, 0.0);
        let scale = 0.7 + 0.8 * rng.next_float();
        dust.new_dust_perfect(rng, ctx, world, position + offset, 228, Some(value), 0, Color::default(), scale);
    }
}

/// `SpawnHelper_SpawnSingleLineDust` (`:2016-2026`, site 2018): a 1×1
/// `NewDust`, the position snapped back to `dustPos`, scale =
/// `Next(70,110)·0.013`, velocity ×0.2, the colour write only when
/// non-default. The four line emitters (UFOLaser/MagnetSphereBolt/
/// HeatRay/Shadowbeam, `:1950-1994`) call it per `SpawnHelper_SpawnInLine`
/// step (`:1996-2014`): the step walk is `num3 = 0; num3 < num; num3 +=
/// num2` with `num = |position − movement|`, `num2 = max(4, |arg|)`,
/// `arg = SafeNormalize(position − movement)·(UniqueInfoPiece/1000)`.
pub(crate) fn single_line_dust(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    dust_id: i32,
    dust_pos: Vec2,
    color: Color,
) {
    let idx = dust.new_dust(rng, ctx, world, dust_pos, 1, 1, dust_id, 0.0, 0.0, 0, Color::default(), 1.0);
    let d = dust.slot_mut(idx);
    d.position = dust_pos;
    d.scale = rng.next_range(70, 110) as f32 * 0.013;
    d.velocity *= 0.2;
    if color != Color::default() {
        d.color = color;
    }
}

/// `SpawnHelper_SpawnInLine` (`:1996-2014`) — the step walk, with the
/// per-step callback the four line emitters pass. `info` is
/// `settings.UniqueInfoPiece`.
pub(crate) fn spawn_in_line(
    position: Vec2,
    movement: Vec2,
    info: i32,
    mut effects_per_step: impl FnMut(Vec2, Vec2),
) {
    let delta = position - movement;
    let arg = safe_normalize(delta, Vec2::ZERO) * (info as f32 / 1000.0);
    let num = delta.length();
    let mut num2 = arg.length();
    if num2 < 4.0 {
        num2 = 4.0;
    }
    let mut num3 = 0.0f32;
    while num3 < num {
        let t = num3 / num;
        effects_per_step(Vec2::lerp(movement, position, t), arg);
        num3 += num2;
    }
}

/// `Spawn_ShadowOrbExplosion` (`:2028-2118`, sites 2088/2091). Head:
/// NextFloat (num), NextFloat (num3), NextFloat (hsl hue), NextFloat
/// (the colour scale). The first sparkle loop draws circular(16,16) per
/// step; the second draws circular(16,16) THEN the type-6 dust pair
/// (the SnappingStoneMelee shape, scale 1.4, no position offset). The
/// `j < 0` gore loop (`:2099-2117`) is dead — zero iterations, no draws.
pub(crate) fn shadow_orb_explosion(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    position: Vec2,
) {
    let pi = std::f32::consts::PI;
    let _num = 20.0 + 10.0 * rng.next_float();
    let num2 = -(pi / 4.0);
    let _num3 = 0.2 + 0.4 * rng.next_float();
    let _hue = rng.next_float() * 0.05 + 0.75;
    let _scale_t = rng.next_float();
    for num4 in [0.0f32, 1.0] {
        let _offset = next_vector2_circular(rng, 16.0, 16.0);
        let _ = num4;
    }
    for num5 in [0.0f32, 1.0] {
        let _offset = next_vector2_circular(rng, 16.0, 16.0);
        let a = pi / 4.0 + pi * num5 + num2;
        let vector2 = Vec2::new((a as f64).cos() as f32, (a as f64).sin() as f32) * 4.0;
        for _ in 0..1 {
            let angle = rng.next_float_direction() * (pi * 2.0) * 0.025;
            let vel = rotated_by(vector2, angle as f64) * rng.next_float();
            let idx = dust.new_dust_perfect(rng, ctx, world, position, 6, Some(vel), 0, Color::default(), 1.0);
            let d = dust.slot_mut(idx);
            d.no_gravity = true;
            d.scale = 1.4;
            let angle2 = rng.next_float_direction() * (pi * 2.0) * 0.025;
            let vel2 = rotated_by(-vector2, angle2 as f64) * rng.next_float();
            let idx2 = dust.new_dust_perfect(rng, ctx, world, position, 6, Some(vel2), 0, Color::default(), 1.0);
            let d2 = dust.slot_mut(idx2);
            d2.no_gravity = true;
            d2.scale = 1.4;
        }
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

    /// `:1444-1458` + the sparkle loop — 14×(hsl NextFloat + type-66
    /// block + circular) then 20×(Next(20,40) + 5 NextFloat).
    #[test]
    fn w4_d54_boulder4_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let pos = Vec2::new(1600.0, 1600.0);
        let mv = Vec2::new(3.0, -4.0);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        for i in 0..50 {
            rainbow_boulder4(&mut rng, &mut dust, &ctx, &w, pos, mv, false);
            for _ in 0..14 {
                probe.next_float();
                block(&mut probe, 66, 0, 0);
                circular(&mut probe);
            }
            for _ in 0..20 {
                probe.next_range(20, 40);
                for _ in 0..5 {
                    probe.next_float();
                }
            }
            assert_eq!(rng.peek(), probe.peek(), "boulder4 stream @{i}");
            drain(&mut dust);
        }
    }

    /// `:1504-1555` — the head 4 draws, the dead-write loop, then
    /// 21×(hsl + type-66 16×16 block + edge NextFloat + 2 NextFloat).
    #[test]
    fn w4_d54_boulder1_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let pos = Vec2::new(1600.0, 1600.0);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        for i in 0..50 {
            rainbow_boulder1(&mut rng, &mut dust, &ctx, &w, pos);
            probe.next_float();
            let num2 = 6 + probe.next_max(7);
            probe.next_float();
            probe.next_max(2);
            let mut num4 = 0.0f32;
            while num4 < 1.0 {
                probe.next_float();
                probe.next_float();
                probe.next_float();
                probe.next_max(20);
                circular(&mut probe);
                probe.next_float(); // edge unit
                probe.next_float();
                probe.next_float();
                num4 += 1.0 / num2 as f32;
            }
            for _ in 0..21 {
                probe.next_float();
                block(&mut probe, 66, 16, 16);
                probe.next_float(); // edge unit
                probe.next_float();
                probe.next_float();
            }
            assert_eq!(rng.peek(), probe.peek(), "boulder1 stream @{i}");
            drain(&mut dust);
        }
    }

    /// `:1626-1645` — four type-267 spawns with the hsl pair, the
    /// circular + direction + 2 NextFloat post draws, then the clone's
    /// fresh type-267 block (the drained pool keeps indices low).
    #[test]
    fn w4_d54_boulder3_tail_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let pos = Vec2::new(1600.0, 1600.0);
        let mv = Vec2::new(2.0, 1.0);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        for i in 0..50 {
            rainbow_boulder3(&mut rng, &mut dust, &ctx, &w, pos, mv);
            probe.next_float();
            let num2 = 3 + probe.next_max(7);
            probe.next_float();
            probe.next_max(2);
            let mut num5 = 0.0f32;
            while num5 < 1.0 {
                probe.next_float_direction();
                probe.next_float();
                probe.next_float();
                probe.next_float();
                probe.next_max(20);
                probe.next_float();
                probe.next_float();
                probe.next_float();
                num5 += 1.0 / num2 as f32;
            }
            for _ in 0..4 {
                probe.next_float();
                probe.next_float();
                block(&mut probe, 267, 0, 0);
                circular(&mut probe);
                probe.next_float_direction();
                probe.next_float();
                probe.next_float();
                block(&mut probe, 267, 0, 0); // the clone
            }
            assert_eq!(rng.peek(), probe.peek(), "boulder3 stream @{i}");
            drain(&mut dust);
        }
    }

    /// `:2018-2025` — one 1×1 block + Next(70,110); the position snaps
    /// back to the exact dustPos.
    #[test]
    fn w4_d54_single_line_dust_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let pos = Vec2::new(1600.0, 1600.0);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        for i in 0..100 {
            single_line_dust(&mut rng, &mut dust, &ctx, &w, 160, pos, Color::default());
            block(&mut probe, 160, 1, 1);
            probe.next_range(70, 110);
            assert_eq!(rng.peek(), probe.peek(), "line stream @{i}");
            drain(&mut dust);
        }
        let mut rng2 = UnifiedRandom::new(54);
        let mut dust2 = DustPool::new();
        single_line_dust(&mut rng2, &mut dust2, &ctx, &w, 160, pos, Color::default());
        assert_eq!(dust2.slot(0).position, pos);
    }

    /// `:2030-2094` — head 4 NextFloat, 2 circulars, then 2×(circular +
    /// 2×(direction + NextFloat + type-6 block)).
    #[test]
    fn w4_d54_shadow_orb_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let pos = Vec2::new(1600.0, 1600.0);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        for i in 0..50 {
            shadow_orb_explosion(&mut rng, &mut dust, &ctx, &w, pos);
            for _ in 0..4 {
                probe.next_float();
            }
            circular(&mut probe);
            circular(&mut probe);
            for _ in 0..2 {
                circular(&mut probe);
                for _ in 0..2 {
                    probe.next_float_direction();
                    probe.next_float();
                    block(&mut probe, 6, 0, 0);
                }
            }
            assert_eq!(rng.peek(), probe.peek(), "shadow-orb stream @{i}");
            drain(&mut dust);
        }
    }
}
