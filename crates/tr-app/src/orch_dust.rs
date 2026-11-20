//! W4-d54 batch 3 — ParticleOrchestrator dust sites 1-10 (the e4-reserved
//! first-40 lane, `docs/research/67-dust-orch-sites-claims.md`):
//! `Spawn_ArmletOfRuinRecharge` (`ParticleOrchestrator.cs:418-435`, site
//! 423), `Spawn_PyroclasticStoneRecharge` (`:437-452`, site 442),
//! `Spawn_SnappingStoneRecharge` (`:454-470`, site 459),
//! `Spawn_SnappingStone` (`:593-646`, site 628), `Spawn_SnappingStoneMelee`
//! (`:648-731`, sites 723/726), `Spawn_SnappingStoneSummon` (`:733-813`,
//! sites 791/804) and `Spawn_PaladinsHammerShockwave` (`:815-879`, sites
//! 826/835).
//!
//! RNG contract (S1-A-10): the orchestrator spawns run synchronously inside
//! `RequestParticleSpawn`/`BroadcastParticleSpawn` (`:52-90` →
//! `SpawnParticlesDirect` :162), so each emitter drains on the stream of
//! the pass that invoked it — the caller passes `&mut UnifiedRandom`,
//! never a new stream. Argument draws run BEFORE the pool internals (C#
//! evaluates arguments first). `WorldGen.genRand` rides the port's runtime
//! stand-in (`World::frame_rng`), passed as `gen`.
//!
//! The particle-pool requests (`FadingParticle`/`PrettySparkleParticle`)
//! are the documented renderer skip (research/15: no particle renderer);
//! `ParticlePool.RequestParticle` is RNG-free (`ParticlePool.cs:32`) but
//! their ARGUMENT draws are consumed in order — kept as ordered `next_*`
//! calls with the visual writes dropped (each site cites the range).
//! The gore siblings (`:871` PaladinsHammerShockwave) are g16/g17's lane;
//! their argument draws are consumed here in order (the e2/e4 split).
//!
//! The tile sweeps delegate to g14's landed
//! `tr_core::worldgen_dust_tile::kill_tile_make_tile_dust` (all 29
//! WorldGen sites, a35efec2); the counts come from the ported
//! `collision.rs::tile_dust_amount` (`WorldGen.cs:66597-66685`).
//!
//! WRITTEN NOT RUN (code-only mode).

use crate::bow_launch::rotated_by;
use crate::player::Player;
use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// `Utils.ToTileCoordinates` (`Utils.cs:1236-1239`): `(int)x >> 4` —
/// truncation toward zero THEN the arithmetic shift (negative coords
/// floor to the next tile down).
pub(crate) fn tile_coords(p: Vec2) -> (i32, i32) {
    ((p.x as i32) >> 4, (p.y as i32) >> 4)
}

/// `Utils.NextVector2Unit` (`Utils.cs:1261-1264`): one `NextFloat`, the
/// angle `0 + 2π·f`, then `ToRotationVector2` (`Utils.cs:1665-1668`,
/// double cos/sin cast to f32).
fn unit_vector2(rng: &mut UnifiedRandom) -> Vec2 {
    let angle = std::f32::consts::PI * 2.0 * rng.next_float();
    Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32)
}

/// `Utils.NextVector2Circular` (`Utils.cs:1266-1269`): the unit draw THEN
/// a second `NextFloat` radius scale.
pub(crate) fn next_vector2_circular(rng: &mut UnifiedRandom, hw: f32, hh: f32) -> Vec2 {
    let unit = unit_vector2(rng);
    let r = rng.next_float();
    Vec2::new(unit.x * hw * r, unit.y * hh * r)
}

/// XNA `Vector2.Normalize` — the reciprocal-first order (`num = 1f /
/// sqrt(...)` then the two multiplies; a zero vector yields NaN
/// components, the npc_ai_style75 helper's shape).
fn normalize(v: Vec2) -> Vec2 {
    let inv = 1.0f32 / v.length();
    Vec2::new(v.x * inv, v.y * inv)
}

/// `Main.player[settings.IndexOfPlayerWhoInvokedThis]` — the port is
/// single-player (`Main.player[0]`); the settings' invoker index is the
/// net lane's (`netmodules.rs::Particles.invoker`). The recharge trio
/// reads position/width/height; `width`/`height` are the C# `Entity`
/// ints (the port's `player::WIDTH`/`HEIGHT` as i32).
pub(crate) struct InvokedPlayer {
    pub position: Vec2,
    pub width: i32,
    pub height: i32,
}

impl InvokedPlayer {
    pub(crate) fn of(p: &Player) -> Self {
        Self {
            position: Vec2::new(p.x, p.y),
            width: crate::player::WIDTH as i32,
            height: crate::player::HEIGHT as i32,
        }
    }
}

/// `Spawn_ArmletOfRuinRecharge` (`ParticleOrchestrator.cs:418-435`, site
/// 423): ten type-15 `NewDust` over the player box, alpha 255. The
/// post-arm order is the C#'s: noLight/noGravity/scale/fadeIn, the
/// mid-height Y snap, `velocity *= 0.5` then `velocity.Y *= 3`, alpha 60,
/// `customData = player` (dropped — the port has no customData follower,
/// particles.rs doc) and noLightEmittance. Type 15 adds no per-type tweak.
pub(crate) fn armlet_of_ruin_recharge(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    player: &InvokedPlayer,
) {
    for _ in 0..10 {
        let idx = dust.new_dust(
            rng, ctx, world, player.position, player.width, player.height, 15, 0.0, 0.0, 255,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_light = true;
        d.no_gravity = true;
        d.scale = 0.9;
        d.fade_in = 1.1;
        d.position.y = player.position.y + (player.height / 2) as f32;
        d.velocity *= 0.5;
        d.velocity.y *= 3.0;
        d.alpha = 60;
        d.no_light_emittance = true;
    }
}

/// `Spawn_PyroclasticStoneRecharge` (`:437-452`, site 442): fifteen
/// type-6 puffs, alpha 255; the post-arm REPLACES the velocity with
/// `NextVector2Circular(2, 4)` AFTER the mid-height Y snap (the draw
/// order: the spawn's default block, then the circular's two NextFloat).
/// Type 6's default-block tweak (`Dust.cs:214-219`) runs inside the pool.
pub(crate) fn pyroclastic_stone_recharge(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    player: &InvokedPlayer,
) {
    for _ in 0..15 {
        let idx = dust.new_dust(
            rng, ctx, world, player.position, player.width, player.height, 6, 0.0, 0.0, 255,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_light = true;
        d.no_gravity = true;
        d.scale = 1.5;
        d.fade_in = 1.1;
        d.position.y = player.position.y + (player.height / 2) as f32;
        d.velocity = next_vector2_circular(rng, 2.0, 4.0);
        d.no_light_emittance = true;
    }
}

/// `Spawn_SnappingStoneRecharge` (`:454-470`, site 459): ten type-226
/// puffs — the ArmletOfRuin shape with type 226 (no per-type tweak).
pub(crate) fn snapping_stone_recharge(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    player: &InvokedPlayer,
) {
    for _ in 0..10 {
        let idx = dust.new_dust(
            rng, ctx, world, player.position, player.width, player.height, 226, 0.0, 0.0, 255,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_light = true;
        d.no_gravity = true;
        d.scale = 0.9;
        d.fade_in = 1.1;
        d.position.y = player.position.y + (player.height / 2) as f32;
        d.velocity *= 0.5;
        d.velocity.y *= 3.0;
        d.no_light_emittance = true;
    }
}

/// `Spawn_SnappingStone` (`:593-646`, site 628). `movement_x` is
/// `settings.MovementVector.X`; `num = movement_x / 256` clamped to 0.12
/// AFTER the two FadingParticle requests (`:610-624`, RNG-free — dropped).
/// Per dust: `Next(3) != 0` picks 226 vs 15, the 0×0 `NewDustDirect`,
/// `velocity *= 64·num`, noGravity, scale 1, fadeIn 1.5 then 0 (the C#'s
/// own double write, kept), noLight; the 226 arm halves the velocity,
/// rolls `Next(6) == 0` → gravity + another halving, then noLightEmittance.
/// The `PlaySound(Item72)` and the projectile-1121 asset gate are the
/// sound/asset lanes' (the gate is assumed loaded — the port's assets are
/// always present).
pub(crate) fn snapping_stone(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    position: Vec2,
    movement_x: f32,
) {
    let mut num = movement_x / 256.0;
    if num > 0.12 {
        num = 0.12;
    }
    for _ in 0..20 {
        let flag = rng.next_max(3) != 0;
        let idx = dust.new_dust_direct(
            rng, ctx, world, position, 0, 0, if flag { 226 } else { 15 }, 0.0, 0.0, 0,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 64.0 * num;
        d.no_gravity = true;
        d.scale = 1.0;
        d.fade_in = 1.5;
        d.no_light = true;
        d.fade_in = 0.0;
        if flag {
            d.velocity *= 0.5;
            if rng.next_max(6) == 0 {
                d.no_gravity = false;
                d.velocity *= 0.5;
            }
            d.no_light_emittance = true;
        }
    }
}

/// `Spawn_SnappingStoneMelee` (`:648-731`, sites 723/726). The head draws
/// `NextFloatDirection` once (`:661`, the sparkle angle offset); the two
/// PrettySparkle loops (`:666-720`) are RNG-free requests (dropped). The
/// dust pair runs inside the SECOND sparkle loop (`num6 = 1`, one
/// iteration): `vector4 = ToRotationVector2(π/4 + π/4·1 + num4) · 14`,
/// then per pair `RotatedBy(NextFloatDirection·2π·0.025) · NextFloat`
/// (the ± mirror re-rolls its own direction + radius). Type 6's tweak
/// runs inside the pool; the post-arms set noGravity + scale 1.4.
pub(crate) fn snapping_stone_melee(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    position: Vec2,
) {
    let pi = std::f32::consts::PI;
    let num4 = 0.0 + rng.next_float_direction() * (pi * 2.0) * 0.05;
    // `:698` — the second loop's vector4 (num6 = 1f, num2 = 14f).
    let a = pi / 4.0 + pi / 4.0 * 1.0 + num4;
    let vector4 = Vec2::new((a as f64).cos() as f32, (a as f64).sin() as f32) * 14.0;
    for _ in 0..2 {
        let angle = rng.next_float_direction() * (pi * 2.0) * 0.025;
        let vel = rotated_by(vector4, angle as f64) * rng.next_float();
        let idx = dust.new_dust_perfect(rng, ctx, world, position, 6, Some(vel), 0, Color::default(), 1.0);
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.scale = 1.4;
        let angle2 = rng.next_float_direction() * (pi * 2.0) * 0.025;
        let vel2 = rotated_by(-vector4, angle2 as f64) * rng.next_float();
        let idx2 = dust.new_dust_perfect(rng, ctx, world, position, 6, Some(vel2), 0, Color::default(), 1.0);
        let d2 = dust.slot_mut(idx2);
        d2.no_gravity = true;
        d2.scale = 1.4;
    }
}

/// `Spawn_SnappingStoneSummon` (`:733-813`, sites 791/804). The head's
/// `num = 1f` clamps to 0.12 (`:746-749`); the third FadingParticle draws
/// `NextFloatDirection` then `NextFloat` (`:783-784`, consumed, visuals
/// dropped). Loop 1: fifteen type-172 perfects with
/// `NextVector2Circular(10, 10)` velocity, `scale = max(0.75, 2·(10·num))`,
/// noGravity, then `Next(6) == 0` → gravity + `scale *= 0.25`. Loop 2:
/// the same eight times with type 211.
pub(crate) fn snapping_stone_summon(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    position: Vec2,
) {
    let mut num = 1.0f32;
    if num > 0.12 {
        num = 0.12;
    }
    // `:783-784` — the third FadingParticle's rotation draws (dropped visuals).
    let _rot_vel = rng.next_float_direction() * (std::f32::consts::PI * 2.0) * 0.33 / 60.0;
    let _rot = rng.next_float() * (std::f32::consts::PI * 2.0);
    for (count, dust_type) in [(15, 172), (8, 211)] {
        let num4 = 10.0 * num;
        for _ in 0..count {
            let vel = next_vector2_circular(rng, 10.0, 10.0);
            let idx = dust.new_dust_perfect(rng, ctx, world, position, dust_type, Some(vel), 0, Color::default(), 1.0);
            let d = dust.slot_mut(idx);
            d.scale = 0.75f32.max(2.0 * num4);
            d.no_gravity = true;
            if rng.next_max(6) == 0 {
                d.no_gravity = false;
                d.scale *= 0.25;
            }
        }
    }
}

/// `Spawn_PaladinsHammerShockwave` (`:815-879`, sites 826/835).
/// `movement` is `settings.MovementVector`; the box is its truncated
/// components, `vector = position + (num·0.5, num2·0.5)`. Fifty
/// iterations: `Next(2) == 0` → type-57 puff (alpha 200, scale 1.8) whose
/// velocity becomes the normalized position-offset · 9.5 · NextFloat;
/// `Next(3) == 0` → the type-43 twin (alpha 254, scale 1.3). Then the 7×7
/// tile sweep around the impact tile: `KillTile_GetTileDustAmount(fail)`
/// counts, the g14 `KillTile_MakeTileDust` delegation with the
/// `num5 = 4` lift arms, and the gore sibling (`:868-875`, g17's — its
/// `Next(2)` gate + `Next(3)` type + `NextFloat` velocity draws consumed
/// in order, the gore writes dropped).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paladins_hammer_shockwave(
    rng: &mut UnifiedRandom,
    gen: &mut UnifiedRandom,
    disco: (u8, u8, u8),
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    position: Vec2,
    movement: Vec2,
) {
    let num = movement.x as i32;
    let num2 = movement.y as i32;
    let vector = position + Vec2::new(num as f32 * 0.5, num2 as f32 * 0.5);
    for _ in 0..50 {
        if rng.next_max(2) == 0 {
            let idx = dust.new_dust(rng, ctx, world, position, num, num2, 57, 0.0, 0.0, 200, Color::default(), 1.8);
            let d = dust.slot_mut(idx);
            d.velocity = normalize(d.position - vector) * 9.5;
            d.no_gravity = true;
            d.velocity *= rng.next_float();
        }
        if rng.next_max(3) == 0 {
            let idx = dust.new_dust(rng, ctx, world, position, num, num2, 43, 0.0, 0.0, 254, Color::default(), 1.3);
            let d = dust.slot_mut(idx);
            d.velocity = normalize(d.position - vector) * 9.5;
            d.no_gravity = true;
            d.velocity *= rng.next_float();
        }
    }
    let num5 = 4;
    let (px, py) = tile_coords(position);
    for j in px - 3..=px + 3 {
        for k in py - 3..=py + 3 {
            let Some(t) = world.tile(j, k).filter(|t| t.active()) else {
                continue;
            };
            let num6 = crate::collision::tile_dust_amount(t.kind, true, t.frame_x);
            for _ in 0..num6 {
                let idx = tr_core::worldgen_dust_tile::kill_tile_make_tile_dust(j, k, t, disco, rng, gen, dust, ctx, world);
                let obj = dust.slot_mut(idx);
                obj.velocity.y -= 3.0 + num5 as f32 * 1.5;
                obj.velocity.y *= rng.next_float();
                obj.velocity.y *= 0.75;
                obj.scale += num5 as f32 * 0.03;
            }
            for _ in 0..num6 - 1 {
                let idx = tr_core::worldgen_dust_tile::kill_tile_make_tile_dust(j, k, t, disco, rng, gen, dust, ctx, world);
                let obj2 = dust.slot_mut(idx);
                obj2.velocity.y -= 1.0 + num5 as f32;
                obj2.velocity.y *= rng.next_float();
                obj2.velocity.y *= 0.75;
            }
            if num6 > 0 && rng.next_max(2) != 0 {
                // `:870-874` — the gore sibling (g17's lane): the type
                // draw `61 + Next(3)` and the velocity `NextFloat` are
                // consumed in order; the gore writes are dropped.
                let _num7 = (px - j).abs() / 3;
                let _gore_type = 61 + rng.next_max(3);
                let _gore_vel = rng.next_float();
            }
        }
    }
}

/// `safe_normalize` re-export for the sibling batches (the C#
/// `MovementVector.SafeNormalize(Vector2.Zero)` head draws).
pub(crate) use crate::bow_launch::safe_normalize as movement_normalize;

#[cfg(test)]
mod tests {
    use super::*;

    /// A permissive whole-world ctx (the d52/d54 `dust_ctx` house pattern).
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

    /// One `NewDust` default block on an empty pool under the 50 % band
    /// (`Dust.cs:153-193`): the two position `Next(box-4)` rolls (box
    /// clamps to >= 5), the velocity `Next(-20,21)` pair, the frame
    /// `Next(3)`, the scale `Next(-20,21)`; then the per-type tweak — the
    /// 6/228 family adds one `Next(-10,6)` (`Dust.cs:214-219`).
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

    /// `ParticleOrchestrator.cs:421-434` — ten type-15 blocks over the
    /// 20×42 player box, no post draws.
    #[test]
    fn w4_d54_armlet_recharge_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let p = InvokedPlayer { position: Vec2::new(1600.0, 1600.0), width: 20, height: 42 };
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        for i in 0..100 {
            armlet_of_ruin_recharge(&mut rng, &mut dust, &ctx, &w, &p);
            for _ in 0..10 {
                block(&mut probe, 15, 20, 42);
            }
            assert_eq!(rng.peek(), probe.peek(), "armlet stream @{i}");
            drain(&mut dust);
        }
        // The post-arm census: mid-height Y snap, halved-then-tripled Y.
        let mut rng2 = UnifiedRandom::new(54);
        let mut dust2 = DustPool::new();
        armlet_of_ruin_recharge(&mut rng2, &mut dust2, &ctx, &w, &p);
        let d = dust2.slot(0);
        assert!(d.active && d.dust_type == 15);
        assert_eq!(d.position.y, 1600.0 + 21.0);
        assert_eq!(d.scale, 0.9);
        assert_eq!(d.fade_in, 1.1);
        assert_eq!(d.alpha, 60);
        assert!(d.no_light && d.no_gravity && d.no_light_emittance);
    }

    /// `:440-451` — fifteen type-6 blocks, each followed by the circular
    /// velocity replacement (two NextFloat).
    #[test]
    fn w4_d54_pyroclastic_recharge_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let p = InvokedPlayer { position: Vec2::new(1600.0, 1600.0), width: 20, height: 42 };
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        for i in 0..100 {
            pyroclastic_stone_recharge(&mut rng, &mut dust, &ctx, &w, &p);
            for _ in 0..15 {
                block(&mut probe, 6, 20, 42);
                circular(&mut probe);
            }
            assert_eq!(rng.peek(), probe.peek(), "pyroclastic stream @{i}");
            drain(&mut dust);
        }
    }

    /// `:457-469` — ten type-226 blocks, no post draws (the Armlet shape).
    #[test]
    fn w4_d54_snapping_recharge_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let p = InvokedPlayer { position: Vec2::new(1600.0, 1600.0), width: 20, height: 42 };
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        for i in 0..100 {
            snapping_stone_recharge(&mut rng, &mut dust, &ctx, &w, &p);
            for _ in 0..10 {
                block(&mut probe, 226, 20, 42);
            }
            assert_eq!(rng.peek(), probe.peek(), "snapping-recharge stream @{i}");
            drain(&mut dust);
        }
    }

    /// `:625-645` — per dust: the Next(3) gate, a 0×0 block of the gated
    /// type, then the 226 arm's Next(6). The gate's short-circuit means
    /// the Next(6) draws ONLY on the 226 arm.
    #[test]
    fn w4_d54_snapping_stone_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let pos = Vec2::new(1600.0, 1600.0);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        for i in 0..100 {
            snapping_stone(&mut rng, &mut dust, &ctx, &w, pos, 30.0);
            for _ in 0..20 {
                let flag = probe.next_max(3) != 0;
                block(&mut probe, if flag { 226 } else { 15 }, 0, 0);
                if flag {
                    probe.next_max(6);
                }
            }
            assert_eq!(rng.peek(), probe.peek(), "snapping stream @{i}");
            drain(&mut dust);
        }
    }

    /// `:661` + `:721-729` — one head NextFloatDirection, then two pairs
    /// of (direction, radius, type-6 block) ×2.
    #[test]
    fn w4_d54_snapping_melee_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let pos = Vec2::new(1600.0, 1600.0);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        for i in 0..100 {
            snapping_stone_melee(&mut rng, &mut dust, &ctx, &w, pos);
            probe.next_float_direction();
            for _ in 0..2 {
                for _ in 0..2 {
                    probe.next_float_direction();
                    probe.next_float();
                    block(&mut probe, 6, 0, 0);
                }
            }
            assert_eq!(rng.peek(), probe.peek(), "melee stream @{i}");
            drain(&mut dust);
        }
    }

    /// `:783-784` + `:787-812` — the two rotation draws, then 15+8
    /// (circular, type-172/211 block, Next(6)) triples.
    #[test]
    fn w4_d54_snapping_summon_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let pos = Vec2::new(1600.0, 1600.0);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        for i in 0..100 {
            snapping_stone_summon(&mut rng, &mut dust, &ctx, &w, pos);
            probe.next_float_direction();
            probe.next_float();
            for (count, ty) in [(15, 172), (8, 211)] {
                for _ in 0..count {
                    circular(&mut probe);
                    block(&mut probe, ty, 0, 0);
                    probe.next_max(6);
                }
            }
            assert_eq!(rng.peek(), probe.peek(), "summon stream @{i}");
            drain(&mut dust);
        }
    }

    /// `:822-842` on an EMPTY world (the tile sweep finds no active tile,
    /// so only the 50 gated pairs draw): Next(2) gate → type-57 block +
    /// NextFloat; Next(3) gate → type-43 block + NextFloat.
    #[test]
    fn w4_d54_hammer_shockwave_head_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let pos = Vec2::new(1600.0, 1600.0);
        let mv = Vec2::new(48.0, 32.0);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut gen = UnifiedRandom::new(7);
        let mut dust = DustPool::new();
        for i in 0..100 {
            paladins_hammer_shockwave(&mut rng, &mut gen, (255, 255, 255), &mut dust, &ctx, &w, pos, mv);
            for _ in 0..50 {
                if probe.next_max(2) == 0 {
                    block(&mut probe, 57, 48, 32);
                    probe.next_float();
                }
                if probe.next_max(3) == 0 {
                    block(&mut probe, 43, 48, 32);
                    probe.next_float();
                }
            }
            assert_eq!(rng.peek(), probe.peek(), "shockwave stream @{i}");
            drain(&mut dust);
        }
    }
}
