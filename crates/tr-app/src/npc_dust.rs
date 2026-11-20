//! W4-e2 — the NPC.cs `Dust.NewDust` sites' SECOND batch (the first ~40
//! are W4-d52's: the Eye of Cthulhu, the aiStyle-8 casters, the golem
//! body/fists and the sphere family). This module carries the shared
//! emitters for the sites whose machines live INLINE in `npc.rs`
//! (`update_skeletron`, `update_brain`, `update_flying_dutchman`,
//! `update_stardust_cell`, `update_moon_leech_blob`, `update_golem_rng`'s
//! free head) plus the twins' gate/burst pair shared by
//! `npc_ai_style30`/`npc_ai_style31`.
//!
//! Contract (the S1-E-1 pool, `tr_core::particles`, d50's DustPool):
//! every draw rides the caller's `UpdateNPCs` stream in C# statement
//! order — the explicit `Main.rand` argument draws FIRST (C# evaluates
//! arguments before the call), then the `NewDust` internals roll inside
//! `DustPool::new_dust` (`Dust.cs:75-256`), then the post-spawn writes
//! (which read `Main.dust[6000]` when the pool refuses — the sentinel
//! slot exists and takes the writes exactly like the C#). The
//! `position ± netOffset` round trips net to zero in the single-player
//! port (the field is never written) and fold away with this note.
//! Gore siblings stay W4-d55's lane: where a gore call sits between
//! dust draws its ARGUMENT draws are consumed here in order (the
//! `NewGore` body rolls are d55's — the same split d52's spin hand-off
//! used), and the site comments cite it.
//!
//! Batch ledger (C# `Dust.NewDust*` line = site id):
//! - 22285/22292 — Skeletron's outside-spin blood trail (`:22283-22297`).
//! - 26462/26755/26760 — Retinazer gate/burst/tick (`npc_ai_style30`).
//! - 27076/27369/27374 — Spazmatism gate/burst/tick (`npc_ai_style31`).
//! - 31576/31579 — the free Golem head's two 228 emitters
//!   (`:31572-31585`, `npc_ai_golem`).
//! - 32416 — the Brain's retreat-entry 20-dust burst (`:32406-32418`).
//! - 36741/36750 — the Moon Leech Clot's 6+3 ring (`:36738-36755`).
//! - 38958 — the Flying Dutchman's 2-iteration 228 trail
//!   (`:38953-38966`).
//! - 39421 — the Stardust Cell's growth-ring emitter (`:39407-39428`).
//! WRITTEN NOT RUN (code-only mode): the witnesses below are compiled
//! by the merge-verify wave, not executed here.

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// Sites 22285/22292 — Skeletron's blood trail (`NPC.cs:22283-22297`),
/// AFTER the state match and its velocity writes, BEFORE the facing
/// latch: `ai[1] != 2 && ai[1] != 3 && type != 68 && (num168 != 0 ||
/// !expertMode)` — `num168` is the hands-alive count (`:21965`'s scan;
/// the port's `world.skeletron_hands_alive`), expert rides the caller's
/// flag. The C# carries NO rand draws outside `NewDust` here (the
/// velocities are fixed expressions) — only the pool internals roll.
pub fn skeletron_trail(
    x: f32,
    y: f32,
    width: i32,
    height: f32,
    vx: f32,
    vy: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:22285` — the foot puff: box (30, 10) at
    // (x + width/2 - 15 - vx*5, y + height - 2), type 5, speeds
    // ((0 - vx)*0.2, 3), scale 2. `width / 2` is INTEGER division.
    let idx = dust.new_dust(
        rng,
        ctx,
        world,
        Vec2::new(x + (width / 2) as f32 - 15.0 - vx * 5.0, y + height - 2.0),
        30,
        10,
        5,
        (0.0 - vx) * 0.2,
        3.0,
        0,
        Color::default(),
        2.0,
    );
    let d = dust.slot_mut(idx);
    d.no_gravity = true; // `:22286`
    d.velocity.x *= 1.3; // `:22287`
    d.velocity.x += vx * 0.4; // `:22288`
    d.velocity.y += 2.0 + vy; // `:22289`
    // `:22290-22296` — two box dusts at (x, y + 120), (width, 60),
    // speeds (vx, vy), scale 2; noGravity, velocity -= (vx, vy), then
    // velocity.Y += 5.
    for _ in 0..2 {
        let idx = dust.new_dust(
            rng, ctx, world, Vec2::new(x, y + 120.0), width, 60, 5, vx, vy, 0,
            Color::default(), 2.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity -= Vec2::new(vx, vy);
        d.velocity.y += 5.0;
    }
}

/// Sites 26462/27076 — the twins' ambient drip (`NPC.cs:26460-26465` /
/// `:27074-27079`, identical bodies): the `Next(5) == 0` gate EVERY
/// tick (the caller consumes it before the day gate), then one type-5
/// `NewDust` at `(x, y + height*0.25)`, box `(width, (int)(height*0.5))`,
/// speeds `(vx, 2f)`; post-spawn `velocity.X *= 0.5`, `velocity.Y *= 0.1`.
/// `gate` is the already-drawn `Next(5) == 0` result — the gate draw
/// stays at the machine's C# position even on the dust-free entries.
pub fn twin_drip(
    gate: bool,
    x: f32,
    y: f32,
    width: i32,
    height: f32,
    vx: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    if !gate {
        return;
    }
    let idx = dust.new_dust(
        rng,
        ctx,
        world,
        Vec2::new(x, y + height * 0.25),
        width,
        (height * 0.5) as i32,
        5,
        vx,
        2.0,
        0,
        Color::default(),
        1.0,
    );
    let d = dust.slot_mut(idx);
    d.velocity.x *= 0.5;
    d.velocity.y *= 0.1;
}

/// Sites 26755/27369 — the twins' spin hand-off burst
/// (`NPC.cs:26746-26757` / `:27360-27371`): 2 × (3 gores × 2 draws
/// `Next(-30, 31) * 0.2`) then 20 type-5 dusts × the same pair. The
/// machine keeps the six gore ARGUMENT pairs at its C# position (the
/// `NewGore` bodies are d55's); this fn draws each dust's velocity pair
/// then rolls the spawn — 20 × (2 argument draws + internals), the
/// sounds (3, 15) stay the machine's queue.
pub fn twin_spin_burst(
    x: f32,
    y: f32,
    width: i32,
    height: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for _ in 0..20 {
        let dx = rng.next_range(-30, 31) as f32 * 0.2;
        let dy = rng.next_range(-30, 31) as f32 * 0.2;
        dust.new_dust(
            rng, ctx, world, Vec2::new(x, y), width, height, 5, dx, dy, 0,
            Color::default(), 1.0,
        );
    }
}

/// Sites 26760/27374 — the twins' per-tick spin dust
/// (`NPC.cs:26760` / `:27374`): ONE type-5 box dust every spin tick,
/// also on the hand-off tick AFTER the burst, before the 0.98 damp.
pub fn twin_spin_tick(
    x: f32,
    y: f32,
    width: i32,
    height: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let dx = rng.next_range(-30, 31) as f32 * 0.2;
    let dy = rng.next_range(-30, 31) as f32 * 0.2;
    dust.new_dust(
        rng, ctx, world, Vec2::new(x, y), width, height, 5, dx, dy, 0,
        Color::default(), 1.0,
    );
}

/// Site 32416 — the Brain of Cthulhu's retreat-entry burst
/// (`NPC.cs:32406-32418`): inside the `localAI[2]` latch, AFTER the
/// `PlaySound(3)` and the four gore calls (392/393/394/395 — their
/// eight ARGUMENT draws are consumed here in order, the `NewGore`
/// bodies are d55's), 20 type-5 dusts × `Next(-30, 31) * 0.2` pairs,
/// then the case-15 roar queues (the caller's order).
pub fn brain_retreat_burst(
    x: f32,
    y: f32,
    width: i32,
    height: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    // `:32410-32413` — the four gores' argument pairs (d55 bodies).
    for _ in 0..4 {
        let _gx = rng.next_range(-30, 31) as f32 * 0.2;
        let _gy = rng.next_range(-30, 31) as f32 * 0.2;
    }
    // `:32414-32417` — the 20-dust loop (site 32416).
    for _ in 0..20 {
        let dx = rng.next_range(-30, 31) as f32 * 0.2;
        let dy = rng.next_range(-30, 31) as f32 * 0.2;
        dust.new_dust(
            rng, ctx, world, Vec2::new(x, y), width, height, 5, dx, dy, 0,
            Color::default(), 1.0,
        );
    }
}

/// Site 38958 — the Flying Dutchman's smoke trail
/// (`NPC.cs:38953-38966`): two iterations, each a `Next(2) != 0` gate;
/// on a pass the offset draws `(NextFloat() - 0.5) * (width - 70)` and
/// `(NextFloat() - 0.5) * 20 + height/2 + 10` (INTEGER `height / 2`)
/// rotate by `rotation`, then ONE type-228 `NewDust(Center, 0, 0)`
/// whose post-writes pin position = Center + offset, velocity zero,
/// noGravity + noLight, fadeIn 1.5, scale 0.5. Runs AFTER the
/// `rotation = velocity.X * 0.025` write (`:38951`), before the arm's
/// end. `RotatedBy` is the double-precision cos/sin form
/// (`Utils.cs:1675-1684`, the pillar port's convention).
pub fn dutchman_trail(
    cx: f32,
    cy: f32,
    width: i32,
    height: i32,
    rotation: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for _ in 0..2 {
        if rng.next_max(2) == 0 {
            continue;
        }
        let ox = (rng.next_float() - 0.5) * (width - 70) as f32;
        let oy = (rng.next_float() - 0.5) * 20.0 + (height / 2) as f32 + 10.0;
        let r = rotation as f64;
        let (cos, sin) = (r.cos() as f32, r.sin() as f32);
        let rx = ox * cos - oy * sin;
        let ry = ox * sin + oy * cos;
        let idx = dust.new_dust(
            rng, ctx, world, Vec2::new(cx, cy), 0, 0, 228, 0.0, 0.0, 0,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.position = Vec2::new(cx + rx, cy + ry);
        d.velocity = Vec2::ZERO;
        d.no_gravity = true;
        d.no_light = true;
        d.fade_in = 1.5;
        d.scale = 0.5;
    }
}

/// Site 39421 — the Stardust Cell (402)'s growth ring
/// (`NPC.cs:39407-39428`): `num1319 = (int)(ai[0] / (300/2f))` (the
/// style's `num1318 = 300`), then `num1319 + 1` iterations, each a
/// `Next(2) != 0` gate; on a pass the type is 226 with scale 0.4
/// (odd iterations 0.65 — the C# writes 226 in both arms), the ring
/// point is `Center + (NextDouble() * 2π).ToRotationVector2() *
/// (12 - num1319*2)`, the spawn box is (24, 24) at point - (12, 12)
/// with speeds (vx/2, vy/2); post-writes: position -= (2, 2),
/// velocity = Normalize(Center - point) * 1.5 * (10 - num1319*2) / 10,
/// noGravity, scale. `customData = this` drops with the field (the
/// S1-E-1 record note). Runs at the arm's END — after the 300-tick
/// Transform early-return (`:39394-39401`), so a transform tick draws
/// nothing. `Vector2.Normalize` of a zero vector stays zero (XNA).
pub fn stardust_cell_ring(
    cx: f32,
    cy: f32,
    vx: f32,
    vy: f32,
    ai0: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let num1319 = (ai0 / (300.0 / 2.0)) as i32;
    for i in 0..num1319 + 1 {
        if rng.next_max(2) == 0 {
            continue;
        }
        let scale = if i % 2 == 1 { 0.65 } else { 0.4 };
        let ang = rng.next_double() * (std::f32::consts::PI as f64 * 2.0);
        let radius = 12.0 - (num1319 * 2) as f32;
        let px = cx + ang.cos() as f32 * radius;
        let py = cy + ang.sin() as f32 * radius;
        let idx = dust.new_dust(
            rng, ctx, world, Vec2::new(px - 12.0, py - 12.0), 24, 24, 226,
            vx / 2.0, vy / 2.0, 0, Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.position -= Vec2::new(2.0, 2.0);
        let (mut nx, mut ny) = (cx - px, cy - py);
        let len = (nx * nx + ny * ny).sqrt();
        if len > 0.0 {
            nx /= len;
            ny /= len;
        }
        let k = 1.5 * (10.0 - (num1319 * 2) as f32) / 10.0;
        d.velocity = Vec2::new(nx * k, ny * k);
        d.no_gravity = true;
        d.scale = scale;
    }
}

/// Sites 36741/36750 — the Moon Leech Clot (401)'s attached rings
/// (`NPC.cs:36738-36755`): six type-229 dusts on the
/// `-UnitY * height/2` spinner at `i * 2π/6` (velocity = -UnitY,
/// noGravity, scale 0.7), then three type-229 dusts on the
/// `-UnitY * height/6` spinner at `i * 2π/6` (speeds (0, -2),
/// noGravity, scale 1.5). Both spawn boxes are 0×0 at
/// `Center - (4, 4) + spun`. `customData = this` drops with the field.
/// The C#'s lerp-to-the-leech-anchor (`:36733-36737`) needs the
/// sibling/proj pools — the port's stand-in keeps the blob at its own
/// position (the `update_moon_leech_blob` deviation note); the rings
/// read that Centre. NO rand draws outside the pool internals.
pub fn moon_leech_rings(
    cx: f32,
    cy: f32,
    height: f32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let step = std::f32::consts::PI * 2.0 / 6.0;
    for i in 0..6 {
        // (0, -h/2).RotatedBy(i*step) — the double cos/sin form.
        let a = (i as f32 * step) as f64;
        let (cos, sin) = (a.cos() as f32, a.sin() as f32);
        let (sx, sy) = ((-height / 2.0) * -sin, (-height / 2.0) * cos);
        let idx = dust.new_dust(
            rng, ctx, world, Vec2::new(cx - 4.0 + sx, cy - 4.0 + sy), 0, 0, 229,
            0.0, 0.0, 0, Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.velocity = Vec2::new(0.0, -1.0);
        d.no_gravity = true;
        d.scale = 0.7;
    }
    for i in 0..3 {
        let a = (i as f32 * step) as f64;
        let (cos, sin) = (a.cos() as f32, a.sin() as f32);
        let (sx, sy) = ((-height / 6.0) * -sin, (-height / 6.0) * cos);
        let idx = dust.new_dust(
            rng, ctx, world, Vec2::new(cx - 4.0 + sx, cy - 4.0 + sy), 0, 0, 229,
            0.0, -2.0, 0, Color::default(), 1.0,
        );
        let d = dust.slot_mut(idx);
        d.no_gravity = true;
        d.scale = 1.5;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> World {
        World::new("e2".into(), 200, 200)
    }

    /// A permissive ctx: the whole world on-screen, the full budget —
    /// the pool never culls (the `update_rng` scratch convention).
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

    /// W4-e2 — sites 22285/22292: three dusts, fixed velocities, no
    /// explicit draws. The foot puff's post-chain lands
    /// vx' = ((-vx*0.2) + jitter)*1.3 + vx*0.4 — re-derived against a
    /// mirrored stream through the same pool call.
    #[test]
    fn w4_e2_skeletron_trail_spawns_three_and_chains_the_foot_puff() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(11);
        let mut mirror = UnifiedRandom::new(11);
        let mut mpool = DustPool::new();
        skeletron_trail(1000.0, 900.0, 100, 100.0, 2.0, -1.0, &mut rng, &mut pool, &c, &w);
        let idx = mpool.new_dust(
            &mut mirror, &c, &w,
            Vec2::new(1000.0 + 50.0 - 15.0 - 10.0, 900.0 + 100.0 - 2.0),
            30, 10, 5, -0.4, 3.0, 0, Color::default(), 2.0,
        );
        let md = mpool.slot_mut(idx);
        md.no_gravity = true;
        md.velocity.x *= 1.3;
        md.velocity.x += 0.8;
        md.velocity.y += 1.0;
        assert_eq!(active(&pool).len(), 3, "1 foot puff + 2 box dusts");
        assert_eq!(pool.slot(0).position, mpool.slot(0).position);
        assert_eq!(pool.slot(0).velocity, mpool.slot(0).velocity);
        assert!(pool.slot(0).no_gravity);
        assert_eq!(pool.slot(0).scale, 2.0);
        // The box pair: velocity -= (vx, vy) then +5 Y.
        for i in 1..3 {
            assert!(pool.slot(i).no_gravity);
            assert_eq!(pool.slot(i).scale, 2.0);
        }
    }

    /// W4-e2 — sites 26462/27076: a failed gate draws NOTHING (the
    /// machine already consumed the Next(5)); a passed gate spawns one
    /// type-5 with the quarter-height anchor and the damp chain.
    #[test]
    fn w4_e2_twin_drip_gates_and_damps() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(3);
        twin_drip(false, 500.0, 500.0, 100, 110.0, 4.0, &mut rng, &mut pool, &c, &w);
        assert!(active(&pool).is_empty(), "the failed gate spawns nothing");
        twin_drip(true, 500.0, 500.0, 100, 110.0, 4.0, &mut rng, &mut pool, &c, &w);
        assert_eq!(active(&pool).len(), 1);
        let d = pool.slot(0);
        assert_eq!(d.dust_type, 5);
        assert!(d.position.y >= 527.5 && d.position.y < 527.5 + 55.0);
        // vx' = (4 + jitter) * 0.5 — the jitter is in [-1.2, 1.2]
        // (width 100 box, Dust.cs:158-163), so vx' in [1.4, 2.6].
        assert!(d.velocity.x >= 1.4 && d.velocity.x <= 2.6);
    }

    /// W4-e2 — sites 26755/27369 + 26760/27374: the burst is 20 dusts,
    /// the tick one; both consume the argument pair BEFORE the pool
    /// internals — a mirrored stream through new_dust with the same
    /// pre-draws lands identical records.
    #[test]
    fn w4_e2_twin_burst_and_tick_mirror_the_argument_order() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(21);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(21);
        twin_spin_burst(300.0, 300.0, 100, 110, &mut rng, &mut pool, &c, &w);
        twin_spin_tick(300.0, 300.0, 100, 110, &mut rng, &mut pool, &c, &w);
        for _ in 0..21 {
            let dx = mirror.next_range(-30, 31) as f32 * 0.2;
            let dy = mirror.next_range(-30, 31) as f32 * 0.2;
            mpool.new_dust(
                &mut mirror, &c, &w, Vec2::new(300.0, 300.0), 100, 110, 5,
                dx, dy, 0, Color::default(), 1.0,
            );
        }
        assert_eq!(active(&pool).len(), 21);
        for i in 0..21 {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
        }
    }

    /// W4-e2 — site 32416: the four gore argument pairs (8 draws) run
    /// BEFORE the 20 dusts; the stream after the burst equals a mirror
    /// that drew 8 + 20×2 range pairs plus the same pool internals.
    #[test]
    fn w4_e2_brain_burst_consumes_the_gore_argument_pairs_first() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(31);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(31);
        brain_retreat_burst(700.0, 700.0, 176, 216, &mut rng, &mut pool, &c, &w);
        for _ in 0..4 {
            let _gx = mirror.next_range(-30, 31);
            let _gy = mirror.next_range(-30, 31);
        }
        for _ in 0..20 {
            let dx = mirror.next_range(-30, 31) as f32 * 0.2;
            let dy = mirror.next_range(-30, 31) as f32 * 0.2;
            mpool.new_dust(
                &mut mirror, &c, &w, Vec2::new(700.0, 700.0), 176, 216, 5,
                dx, dy, 0, Color::default(), 1.0,
            );
        }
        assert_eq!(active(&pool).len(), 20);
        for i in 0..20 {
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }

    /// W4-e2 — site 38958: each passed gate draws exactly two floats
    /// then the 228 internals; the post-writes pin the rotated offset,
    /// zero velocity, noLight, fadeIn 1.5, scale 0.5.
    #[test]
    fn w4_e2_dutchman_trail_pins_the_rotated_offset_and_shape() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(41);
        let mut mirror = UnifiedRandom::new(41);
        dutchman_trail(1600.0, 800.0, 350, 120, 0.3, &mut rng, &mut pool, &c, &w);
        // Re-derive the gates/offsets on the mirror; every accepted
        // dust must match the pinned shape.
        let mut expect = 0usize;
        for _ in 0..2 {
            if mirror.next_max(2) == 0 {
                continue;
            }
            let ox = (mirror.next_float() - 0.5) * 280.0;
            let oy = (mirror.next_float() - 0.5) * 20.0 + 60.0 + 10.0;
            let r = 0.3f64;
            let rx = ox * r.cos() as f32 - oy * r.sin() as f32;
            let ry = ox * r.sin() as f32 + oy * r.cos() as f32;
            // The pool internals roll after the two offset floats; the
            // pinned post-writes overwrite position/velocity, so the
            // record is deterministic given the gate/offset draws.
            let d = pool.slot(expect);
            assert_eq!(d.dust_type, 228);
            assert!((d.position.x - (1600.0 + rx)).abs() < 1e-3);
            assert!((d.position.y - (800.0 + ry)).abs() < 1e-3);
            assert_eq!(d.velocity, Vec2::ZERO);
            assert!(d.no_gravity && d.no_light);
            assert_eq!(d.fade_in, 1.5);
            assert_eq!(d.scale, 0.5);
            expect += 1;
        }
        assert_eq!(active(&pool).len(), expect);
    }

    /// W4-e2 — site 39421: at ai[0] = 200, num1319 = 1 → two iterations
    /// (scales 0.4 / 0.65 on passes), radius 10, velocity factor
    /// 1.5 * 8 / 10 = 1.2 toward the centre; the odd/even scale rows
    /// and the -2 position nudge pin.
    #[test]
    fn w4_e2_stardust_ring_scales_radius_and_inward_velocity() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(51);
        stardust_cell_ring(1000.0, 1000.0, 1.0, -1.0, 200.0, &mut rng, &mut pool, &c, &w);
        let act = active(&pool);
        assert!(act.len() <= 2, "num1319+1 = 2 iterations");
        for (k, &i) in act.iter().enumerate() {
            let d = pool.slot(i);
            assert_eq!(d.dust_type, 226);
            assert_eq!(d.scale, if k % 2 == 1 { 0.65 } else { 0.4 });
            assert!(d.no_gravity);
            // |velocity| = 1.2 (the normalized inward vector).
            let v = d.velocity.length();
            assert!((v - 1.2).abs() < 1e-4, "inward speed 1.2, got {v}");
        }
    }

    /// W4-e2 — sites 36741/36750: nine dusts (6 + 3), all type 229,
    /// noGravity; the six-ring scales 0.7 with velocity (0, -1), the
    /// three-ring scales 1.5. Zero explicit draws — the stream advances
    /// by the pool internals only (mirrored).
    #[test]
    fn w4_e2_moon_leech_rings_spawn_six_plus_three() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(61);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(61);
        moon_leech_rings(900.0, 900.0, 34.0, &mut rng, &mut pool, &c, &w);
        let step = std::f32::consts::PI * 2.0 / 6.0;
        for i in 0..6 {
            let a = (i as f32 * step) as f64;
            let (sx, sy) = (-17.0 * -(a.sin() as f32), -17.0 * a.cos() as f32);
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, Vec2::new(896.0 + sx, 896.0 + sy), 0, 0,
                229, 0.0, 0.0, 0, Color::default(), 1.0,
            );
            let d = mpool.slot_mut(idx);
            d.velocity = Vec2::new(0.0, -1.0);
            d.no_gravity = true;
            d.scale = 0.7;
        }
        for i in 0..3 {
            let a = (i as f32 * step) as f64;
            let (sx, sy) = (-(34.0 / 6.0) * -(a.sin() as f32), -(34.0 / 6.0) * a.cos() as f32);
            let idx = mpool.new_dust(
                &mut mirror, &c, &w, Vec2::new(896.0 + sx, 896.0 + sy), 0, 0,
                229, 0.0, -2.0, 0, Color::default(), 1.0,
            );
            let d = mpool.slot_mut(idx);
            d.no_gravity = true;
            d.scale = 1.5;
        }
        assert_eq!(active(&pool).len(), 9);
        for i in 0..9 {
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).scale, mpool.slot(i).scale);
        }
        assert_eq!(rng.next_range(0, 1000), mirror.next_range(0, 1000));
    }
}
