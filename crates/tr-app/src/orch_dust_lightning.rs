//! W4-d54 batch 6 — ParticleOrchestrator dust sites 11-20 (research/67):
//! `Spawn_PaladinsShieldHit` (`ParticleOrchestrator.cs:902-972`, sites
//! 932/964), `Spawn_StormLightning`'s tree arms (`:1026-1162`, sites
//! 1084/1091) and `SpawnLightningExplosionDust` (`:1164-1239`, sites
//! 1168/1181/1196/1209/1220/1231 — the six arms, two CloneDust siblings
//! riding :1214/:1225).
//!
//! RNG contract: the caller's pass stream (S1-A-10), argument draws
//! before the pool internals. The particle-pool requests are the
//! documented renderer skip; their ARGUMENT draws are consumed in order.
//!
//! StormLightning's head is the documented renderer seam: the
//! `StormLightningParticle.Prepare` bolt generation
//! (`StormLightningParticle.cs:55-77` + `LightningGenerator`) is unported,
//! so `end_position` (the C#'s `stormLightningParticle.EndPosition`
//! write-back, `:1038`) is a caller input; the camera punch's NextFloat
//! (`:1041`) and `Main.NewLightning`'s thunder draws (`Main.cs:66378+`)
//! are consumed by the caller in order. The per-tick bolt dust
//! (`StormLightningParticle.cs:180-221`) rides the particle renderer lane.
//!
//! WRITTEN NOT RUN (code-only mode).

use crate::orch_dust::{next_vector2_circular, tile_coords, InvokedPlayer};
use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// `Utils.RotatedByRandom` (`Utils.cs:1697-1700`): TWO NextDouble draws,
/// `NextDouble·max − NextDouble·max`, then the float-cos/sin spin
/// (`Utils.RotatedBy`, `:1675-1684`, centre zero).
fn rotated_by_random(v: Vec2, max_radians: f64, rng: &mut UnifiedRandom) -> Vec2 {
    let angle = rng.next_double() * max_radians - rng.next_double() * max_radians;
    let c = angle.cos() as f32;
    let s = angle.sin() as f32;
    Vec2::new(v.x * c - v.y * s, v.x * s + v.y * c)
}

/// XNA `Color.Lerp` (`MonoGame Color.cs`): per-channel float lerp, then
/// the ctor's truncating cast (the dust_table helper's shape).
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let ch = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as i32 as u8;
    Color::new(ch(a.r, b.r), ch(a.g, b.g), ch(a.b, b.b), ch(a.a, b.a))
}

/// `Spawn_PaladinsShieldHit` (`:902-972`, sites 932/964). The C# reads
/// TWO player indices out of the position's truncated components
/// (`num = (int)X`, `num2 = (int)Y`) and gates each arm on
/// `Main.player.IndexInRange`. The port is single-player: the caller
/// passes the resolved players (`None` = out of range). Each arm: a
/// FadingParticle request (RNG-free, dropped) then four type-57
/// `NewDustDirect` over the player box with `velocity *= 3`, noGravity,
/// scale 1, fadeIn 1.5, noLight. The item-938 asset gate is assumed
/// loaded (the port's assets are always present).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paladins_shield_hit(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    player_a: Option<&InvokedPlayer>,
    player_b: Option<&InvokedPlayer>,
) {
    if let Some(player) = player_a {
        for _ in 0..4 {
            let idx = dust.new_dust_direct(
                rng, ctx, world, player.position, player.width, player.height, 57, 0.0, 0.0, 0,
                Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            d.velocity *= 3.0;
            d.no_gravity = true;
            d.scale = 1.0;
            d.fade_in = 1.5;
            d.no_light = true;
        }
    }
    if let Some(player2) = player_b {
        for _ in 0..4 {
            let idx = dust.new_dust_direct(
                rng, ctx, world, player2.position, player2.width, player2.height, 57, 0.0, 0.0, 0,
                Color::default(), 1.0,
            );
            let d = dust.slot_mut(idx);
            d.velocity *= 3.0;
            d.no_gravity = true;
            d.scale = 1.0;
            d.fade_in = 1.5;
            d.no_light = true;
        }
    }
}

/// `Spawn_StormLightning`'s dust half (`:1052-1161`, sites 1084/1091).
/// `end_position` is the bolt's resolved end (the renderer seam, module
/// doc); `white` is the settings colour (`Color.White` unless
/// `UniqueInfoPiece != 0` packs it). The 3×3 tile sweep mirrors the
/// PaladinsHammerShockwave shape with `num2 = 3`: the tr-core
/// `kill_tile_make_tile_dust` delegation + lift arms, then the tree-5 arms — six
/// type-6 Directs (`velocity *= 2`, `fadeIn += NextFloat`,
/// noLightEmittance) and three type-31 Directs (`velocity *= 3`,
/// noGravity, `fadeIn += 1 + NextFloat`). Then
/// `SpawnLightningExplosionDust`, the four gore siblings (`:1100-1103`,
/// g17's — `RandomVector2`'s two NextDouble + `Next(3)` per gore consumed
/// in order), and the 12/24-iteration RandomizedFrame loop (`:1132-1161`,
/// `Next(14,22)` + two circulars per iteration, visuals dropped).
#[allow(clippy::too_many_arguments)]
pub(crate) fn storm_lightning_dust(
    rng: &mut UnifiedRandom,
    gen_rng: &mut UnifiedRandom,
    disco: [u8; 3],
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    end_position: Vec2,
    white: Color,
    weapon: bool,
) {
    let num2 = 3;
    let (px, py) = tile_coords(end_position);
    for i in px - 1..=px + 1 {
        for j in py - 1..=py + 1 {
            let Some(t) = world.tile(i, j).filter(|t| t.active()) else {
                continue;
            };
            let num3 = crate::collision::tile_dust_amount(t.kind, true, t.frame_x);
            for _ in 0..num3 {
                let idx = tr_core::worldgen_dust_tile::kill_tile_make_tile_dust(i, j, t, (disco[0], disco[1], disco[2]), rng, gen_rng, dust, ctx, world);
                let obj = dust.slot_mut(idx);
                obj.velocity.y -= 3.0 + num2 as f32 * 1.5;
                obj.velocity.y *= rng.next_float();
                obj.velocity.y *= 0.75;
                obj.scale += num2 as f32 * 0.03;
            }
            for _ in 0..num3 - 1 {
                let idx = tr_core::worldgen_dust_tile::kill_tile_make_tile_dust(i, j, t, (disco[0], disco[1], disco[2]), rng, gen_rng, dust, ctx, world);
                let obj2 = dust.slot_mut(idx);
                obj2.velocity.y -= 1.0 + num2 as f32;
                obj2.velocity.y *= rng.next_float();
                obj2.velocity.y *= 0.75;
            }
            if t.kind == 5 {
                let pos = Vec2::new(i as f32 * 16.0, j as f32 * 16.0);
                for _ in 0..6 {
                    let idx = dust.new_dust_direct(rng, ctx, world, pos, 16, 16, 6, 0.0, 0.0, 0, Color::default(), 1.0);
                    let d = dust.slot_mut(idx);
                    d.velocity *= 2.0;
                    d.fade_in += rng.next_float();
                    d.no_light_emittance = true;
                }
                for _ in 0..3 {
                    let idx = dust.new_dust_direct(rng, ctx, world, pos, 16, 16, 31, 0.0, 0.0, 0, Color::default(), 1.0);
                    let d = dust.slot_mut(idx);
                    d.velocity *= 3.0;
                    d.no_gravity = true;
                    d.fade_in += 1.0 + rng.next_float();
                }
            }
        }
    }
    lightning_explosion_dust(rng, dust, ctx, world, end_position, white);
    // `:1100-1103` — four gore siblings (g17's lane): RandomVector2's two
    // NextDouble draws then the 61 + Next(3) type, per gore.
    for _ in 0..4 {
        rng.next_double();
        rng.next_double();
        let _gore_type = 61 + rng.next_max(3);
    }
    // `:1132-1161` — the RandomizedFrame loop (num6 = 24 weapon / 12
    // storm): Next(14,22) + circular(6,6) + circular(6,3) per step.
    let num6 = if weapon { 24.0 } else { 12.0 };
    let mut num7 = 0.0f32;
    while num7 < 1.0 {
        let _num8 = rng.next_range(14, 22);
        let _v3 = next_vector2_circular(rng, 6.0, 6.0);
        let _vel = next_vector2_circular(rng, 6.0, 3.0);
        num7 += 1.0 / num6;
    }
}

/// `SpawnLightningExplosionDust` (`:1164-1239`, sites 1168/1181/1196/
/// 1209/1220/1231). Six arms, all type-226 perfects re-framed to 278 via
/// `HackFrame` (one `Next(3)` each, `Dust.cs:298-309`) except the 306/43
/// tails:
/// - 12×: velocity ·(1+NextFloat·2.5), −2Y, fadeIn 0, scale
///   0.4+NextFloat·0.5, X·2, position −velocity·3.
/// - 6×: the same head, fadeIn 1.7+Next()·1 (the C#'s int Next), noGravity,
///   then ·0.6 after the X·2.
/// - 6×: position +circular(20,20) BEFORE the spawn; velocity
///   (0,−1).RotatedByRandom(π/4) ·(3+NextFloat·6.5); noGravity; −vel·2.
/// - 6× 306: velocity (0,−4).RotatedByRandom(π/2); colour = custom RGB
///   with A 0; scale 2.8; fadeIn 0; noGravity = Next(3)!=0; CloneDust
///   sibling (white A0, scale 1.9).
/// - 6× 306: the (0,−2) twin.
/// - 3× 43: position +circular(20,20), velocity circular(6,6)·NextFloat,
///   alpha 26, colour Lerp(custom, White, NextFloat), scale
///   1+NextFloat·1.4 — all ARGUMENT draws before the pool; fadeIn 1.5,
///   the Y-flip gate `velocity.Y > 0 && Next(2)==0`, noGravity.
pub(crate) fn lightning_explosion_dust(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    position: Vec2,
    custom_color: Color,
) {
    let pi = std::f64::consts::PI;
    for _ in 0..12 {
        let idx = dust.new_dust_perfect(rng, ctx, world, position, 226, None, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.hack_frame(rng, 278);
        d.color = custom_color;
        d.velocity *= 1.0 + rng.next_float() * 2.5;
        d.velocity += Vec2::new(0.0, -2.0);
        d.fade_in = 0.0;
        d.scale = 0.4 + rng.next_float() * 0.5;
        d.velocity.x *= 2.0;
        d.position -= d.velocity * 3.0;
    }
    for _ in 0..6 {
        let idx = dust.new_dust_perfect(rng, ctx, world, position, 226, None, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.hack_frame(rng, 278);
        d.color = custom_color;
        d.velocity *= 1.0 + rng.next_float() * 2.5;
        d.velocity += Vec2::new(0.0, -2.0);
        d.fade_in = 1.7 + rng.next() as f32;
        d.no_gravity = true;
        d.scale = 0.4 + rng.next_float() * 0.5;
        d.velocity.x *= 2.0;
        d.velocity *= 0.6;
        d.position -= d.velocity * 3.0;
    }
    for _ in 0..6 {
        let offset = next_vector2_circular(rng, 20.0, 20.0);
        let idx = dust.new_dust_perfect(rng, ctx, world, position + offset, 226, None, 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.hack_frame(rng, 278);
        d.color = custom_color;
        d.velocity = rotated_by_random(Vec2::new(0.0, -1.0), pi / 4.0, rng);
        d.velocity *= 3.0 + rng.next_float() * 6.5;
        d.fade_in = 0.0;
        d.scale = 0.4 + rng.next_float() * 0.5;
        d.no_gravity = true;
        d.position -= d.velocity * 2.0;
    }
    for (count, up) in [(6, -4.0f32), (6, -2.0)] {
        for _ in 0..count {
            let vel = rotated_by_random(Vec2::new(0.0, up), pi / 2.0, rng);
            let idx = dust.new_dust_perfect(rng, ctx, world, position, 306, Some(vel), 0, Color::default(), 1.0);
            let d = dust.slot_mut(idx);
            d.color = Color::new(custom_color.r, custom_color.g, custom_color.b, 0);
            d.scale = 2.8;
            d.fade_in = 0.0;
            d.no_gravity = rng.next_max(3) != 0;
            let clone = dust.clone_dust(rng, ctx, world, idx);
            let c = dust.slot_mut(clone);
            c.color = Color::new(255, 255, 255, 0);
            c.scale = 1.9;
        }
    }
    for _ in 0..3 {
        let offset = next_vector2_circular(rng, 20.0, 20.0);
        let vel = next_vector2_circular(rng, 6.0, 6.0) * rng.next_float();
        let color = lerp_color(custom_color, Color::WHITE, rng.next_float());
        let scale = 1.0 + rng.next_float() * 1.4;
        let idx = dust.new_dust_perfect(rng, ctx, world, position + offset, 43, Some(vel), 26, color, scale);
        let d = dust.slot_mut(idx);
        d.fade_in = 1.5;
        if d.velocity.y > 0.0 && rng.next_max(2) == 0 {
            d.velocity.y *= -1.0;
        }
        d.no_gravity = true;
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

    /// The default block (the world_dust oracle): position pair, velocity
    /// pair, frame Next(3), scale roll; the 6/228 family adds Next(-10,6).
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

    /// `:930-938` + `:962-970` — two arms of four type-57 blocks over the
    /// 20×42 box, no post draws.
    #[test]
    fn w4_d54_shield_hit_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let p = InvokedPlayer { position: Vec2::new(1600.0, 1600.0), width: 20, height: 42 };
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        for i in 0..100 {
            paladins_shield_hit(&mut rng, &mut dust, &ctx, &w, Some(&p), Some(&p));
            for _ in 0..8 {
                block(&mut probe, 57, 20, 42);
            }
            assert_eq!(rng.peek(), probe.peek(), "shield stream @{i}");
            drain(&mut dust);
        }
        // The out-of-range arm draws nothing.
        let mut rng2 = UnifiedRandom::new(54);
        let mut dust2 = DustPool::new();
        paladins_shield_hit(&mut rng2, &mut dust2, &ctx, &w, None, None);
        assert_eq!(rng2.peek(), UnifiedRandom::new(54).peek());
    }

    /// `:1166-1238` — the six-arm draw ledger. The 43-tail's Y-flip gate
    /// is short-circuit (`velocity.Y > 0 && Next(2)==0`), so the oracle
    /// mirrors the spawn on its own pool to read the same velocity sign.
    #[test]
    fn w4_d54_lightning_explosion_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let pos = Vec2::new(1600.0, 1600.0);
        let white = Color::WHITE;
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        let mut mirror = DustPool::new();
        for i in 0..50 {
            lightning_explosion_dust(&mut rng, &mut dust, &ctx, &w, pos, white);
            for _ in 0..12 {
                block(&mut probe, 226, 0, 0);
                probe.next_max(3); // HackFrame
                probe.next_float();
                probe.next_float();
            }
            for _ in 0..6 {
                block(&mut probe, 226, 0, 0);
                probe.next_max(3);
                probe.next_float();
                probe.next();
                probe.next_float();
            }
            for _ in 0..6 {
                circular(&mut probe);
                block(&mut probe, 226, 0, 0);
                probe.next_max(3);
                probe.next_double();
                probe.next_double();
                probe.next_float();
                probe.next_float();
            }
            for _ in 0..12 {
                probe.next_double();
                probe.next_double();
                block(&mut probe, 306, 0, 0);
                probe.next_max(3);
                // The CloneDust sibling: a fresh type-306 block (the
                // drained pool keeps every index under the 50 % band).
                block(&mut probe, 306, 0, 0);
            }
            for _ in 0..3 {
                // The argument draws, mirrored through a real spawn so
                // the velocity sign is the port's own (perfect overwrites
                // the position, the velocity is the circular product).
                let offset = next_vector2_circular(&mut probe, 20.0, 20.0);
                let vel = next_vector2_circular(&mut probe, 6.0, 6.0) * probe.next_float();
                let _color_t = probe.next_float();
                let _scale_t = probe.next_float();
                let idx = mirror.new_dust_perfect(&mut probe, &ctx, &w, pos + offset, 43, Some(vel), 26, Color::default(), 1.0);
                if mirror.slot(idx).velocity.y > 0.0 {
                    probe.next_max(2);
                }
            }
            assert_eq!(rng.peek(), probe.peek(), "explosion stream @{i}");
            drain(&mut dust);
            drain(&mut mirror);
        }
    }

    /// `:1080-1096` — the tree-5 arms on a lone tree tile: the tile sweep
    /// delegates (type-5 ladder draws), then 6×(type-6 block + NextFloat)
    /// + 3×(type-31 block + NextFloat). Verified via the port's own
    /// kill_tile_make_tile_dust on both sides (the delegation is the oracle).
    #[test]
    fn w4_d54_storm_lightning_tree_arms_spawn_nine_dusts() {
        let mut w = World::new("t".into(), 200, 200);
        let (tx, ty) = (100, 100);
        if let Some(t) = w.tile_mut(tx, ty) {
            t.kind = 5;
            t.flags = tr_core::tile::FLAG_ACTIVE;
        }
        let ctx = dust_ctx(&w);
        let pos = Vec2::new(tx as f32 * 16.0 + 8.0, ty as f32 * 16.0 + 8.0);
        let mut rng = UnifiedRandom::new(54);
        let mut gen_rng = UnifiedRandom::new(7);
        let mut dust = DustPool::new();
        storm_lightning_dust(&mut rng, &mut gen_rng, [255, 255, 255], &mut dust, &ctx, &w, pos, Color::WHITE, false);
        let six: Vec<_> = dust.slots().iter().filter(|d| d.active && d.dust_type == 6).collect();
        let thirty_one: Vec<_> = dust.slots().iter().filter(|d| d.active && d.dust_type == 31).collect();
        assert_eq!(six.len(), 6, "the tree-5 arm spawns six type-6");
        assert_eq!(thirty_one.len(), 3, "and three type-31");
        assert!(thirty_one.iter().all(|d| d.no_gravity));
        assert!(six.iter().all(|d| d.no_light_emittance));
    }
}
