//! W4-g14 — the WorldGen.cs `Dust.NewDust*` sites (research/65 census:
//! 29 sites in 12 methods; d54/e4/f6 probed in-flight at base with zero
//! pushed commits, this lane adopts ALL of them, line-ordered).
//!
//! Batch 1 (sites 1-10, C# line = site id):
//! - 29649/29657 — `TreeGrowFX_AshShake` (`WorldGen.cs:29639-29672`).
//! - 37624 — `UnlockDoor`'s 4-per-cell dust loop (`:37599-37628`).
//! - 44074 — `paintEffect` (`:44064-44085`).
//! - 44097 — `paintCoatEffect` (`:44087-44108`).
//! - 58034 — `KillActuator` (`:58022-58039`).
//! - 58134/58164/58194/58224 — `KillWire`/`2`/`3`/`4`
//!   (`:58122-58229`, identical dust bodies).
//!
//! Batch 2 adds site 60287 (`PlaceTile`'s kind-22/140 arm) here; the
//! `KillWall_MakeWallDust` sites 62140-62155 live in
//! `worldgen_dust_wall.rs`.
//!
//! Contract (the S1-E-1 pool, d50's `DustPool`): every draw rides the
//! caller's stream in C# statement order — explicit argument draws
//! FIRST (C# evaluates arguments before the call), then the `NewDust`
//! internals inside `DustPool::new_dust` (`Dust.cs:75-256`), then the
//! post-spawn writes (which hit `Main.dust[6000]` when the pool refuses
//! — the sentinel slot takes the writes exactly like the C#).
//!
//! TWO streams where the C# uses two (research/65's RNG world-pass
//! slots): `rng` is `Main.rand` (the `NewDust` internals + the explicit
//! `Utils` argument draws), `gen` is `WorldGen.genRand` (the paint
//! post-spawn gates). The port's genRand stand-in is the caller's
//! world-pass slot (`world.frame_rng` at runtime, the pass stream during
//! generation — the functions are stream-agnostic, S1-A-10: never a new
//! stream).
//!
//! UNWIRED pure emitters (the d50/e2 precedent): the enclosing methods'
//! tile mutations, sounds and item drops belong to the edit-path lanes
//! (`tr_core::edit` carries `kill_wire`/`kill_actuator` already and cites
//! "its dust are app-side" — these are those dusts). The pool's
//! `SpawnCtx.generating` guard (`Dust.cs:89-92`) means the generation
//! passes themselves emit nothing; these fire from the runtime edit
//! paths at the wiring wave.
//!
//! WRITTEN NOT RUN (code-only mode): the witnesses below are compiled by
//! the merge-verify wave, not executed here.

use crate::particles::{Color, DustPool, SpawnCtx};
use crate::unified_random::UnifiedRandom;
use crate::world::World;
use glam::Vec2;

/// `Utils.RandomVector2(random, min, max)` (`Utils.cs:2431-2434`): X
/// then Y, one `NextDouble` each, the double cast to f32 BEFORE the
/// scale (`(max - min) * (float)random.NextDouble() + min`).
fn random_vector2(rng: &mut UnifiedRandom, min: f32, max: f32) -> Vec2 {
    let x = (max - min) * rng.next_double() as f32 + min;
    let y = (max - min) * rng.next_double() as f32 + min;
    Vec2::new(x, y)
}

/// `Utils.NextVector2Unit()` (`Utils.cs:1258-1264`): one `NextFloat`
/// angle over the full turn, then `(cos, sin)` — the f32 angle promoted
/// to f64 for the trig, cast back (the `npc_ai_empress` port's shape).
fn next_vector2_unit(rng: &mut UnifiedRandom) -> Vec2 {
    let angle = std::f32::consts::TAU * rng.next_float();
    Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32)
}

/// `Utils.NextVector2Circular(r, halfW, halfH)` (`Utils.cs:1266-1269`):
/// the unit draw THEN a second `NextFloat` radius scale, componentwise
/// `unit * (halfW, halfH) * r`.
fn next_vector2_circular(rng: &mut UnifiedRandom, half_w: f32, half_h: f32) -> Vec2 {
    let unit = next_vector2_unit(rng);
    let r = rng.next_float();
    Vec2::new(unit.x * half_w * r, unit.y * half_h * r)
}

/// Sites 29649/29657 — `WorldGen.TreeGrowFX_AshShake(x, y, height,
/// hitTree)` (`WorldGen.cs:29639-29672`), the ash-tree shake/grow puff
/// column. `vector = (x, y)*16 + (8, 8)`; the loop walks `num` from 0
/// down to `-height + 1`. Per row, in C# order:
///
/// 1. `num >= -height/2 && Main.rand.Next(3) != 0` (the integer
///    `-height/2` truncates toward zero; the gate draw is SHORT-CIRCUITED
///    away below the half-height line) → `hitTree ? 10 : 5` type-31
///    `NewDustDirect`s at `vector + (-16, num*16) + RandomVector2(-30,
///    30)`, box 4×4, speeds (0, -3), alpha 100; each one's velocity is
///    then OVERWRITTEN with `NextVector2Circular(2, 2) + (0, -0.6)`.
/// 2. `(float)num >= (float)(-height)*0.7 && Next(3) != 0` →
///    `hitTree ? 16 : 10` type-6 `NewDustDirect`s, same anchor, speeds
///    (0, -4), alpha 0, scale `1 + 0.7*NextFloat()` (the scale argument
///    draw runs AFTER the position's `RandomVector2` — C# argument
///    order); velocity overwritten with the same circular + (0, -0.6),
///    then `Next(2) == 0` doubles it and sets noGravity, then
///    `velocity += (0, hitTree ? -1 : -2)`.
/// 3. `!hitTree` → the gore sibling (`:29669`): its ARGUMENT draws are
///    consumed here in order — `RandomVector2(-40, 40)` (two
///    `NextDouble`s, the `(0.5, 1)` scale is draw-free) then `Next(3)`
///    for the `61 + …` type; the `NewGoreDirect` body rolls are the
///    W4-g17 gore-world lane's (the e2 argument/body split).
pub fn tree_grow_fx_ash_shake(
    x: i32,
    y: i32,
    height: i32,
    hit_tree: bool,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let vector = Vec2::new(x as f32, y as f32) * 16.0 + Vec2::new(8.0, 8.0);
    let mut num = 0i32;
    while num > -height {
        let anchor = vector + Vec2::new(-16.0, (num * 16) as f32);
        // `:29644-29651` — the type-31 ash flakes.
        if num >= -height / 2 && rng.next_max(3) != 0 {
            let count = if hit_tree { 10 } else { 5 };
            for _ in 0..count {
                let pos = anchor + random_vector2(rng, -30.0, 30.0);
                let idx = dust.new_dust_direct(
                    rng, ctx, world, pos, 4, 4, 31, 0.0, -3.0, 100, Color::default(), 1.0,
                );
                // `:29649` — the velocity OVERWRITE (the spawn's own
                // jitter + (0, -3) is discarded, the draws kept).
                dust.slot_mut(idx).velocity =
                    next_vector2_circular(rng, 2.0, 2.0) + Vec2::new(0.0, -0.6);
            }
        }
        // `:29652-29666` — the type-6 smoke puffs.
        if num as f32 >= (-height) as f32 * 0.7 && rng.next_max(3) != 0 {
            let count = if hit_tree { 16 } else { 10 };
            for _ in 0..count {
                let pos = anchor + random_vector2(rng, -30.0, 30.0);
                let scale = 1.0 + 0.7 * rng.next_float();
                let idx = dust.new_dust_direct(
                    rng, ctx, world, pos, 4, 4, 6, 0.0, -4.0, 0, Color::default(), scale,
                );
                let d = dust.slot_mut(idx);
                d.velocity = next_vector2_circular(rng, 2.0, 2.0) + Vec2::new(0.0, -0.6);
                if rng.next_max(2) == 0 {
                    d.velocity *= 2.0;
                    d.no_gravity = true;
                }
                d.velocity += Vec2::new(0.0, if hit_tree { -1.0 } else { -2.0 });
            }
        }
        // `:29667-29670` — the gore sibling's argument draws (the body
        // is W4-g17's; the position's `(0.5, 1)` multiply is draw-free).
        if !hit_tree {
            let _gore_pos = random_vector2(rng, -40.0, 40.0);
            let _gore_type = 61 + rng.next_max(3);
        }
        num -= 1;
    }
}

/// Site 37624 — `WorldGen.UnlockDoor`'s dust loop
/// (`WorldGen.cs:37615-37626`): after the frameY-594 walk resolves the
/// door's top row and the sound plays, each of the three door cells
/// (`top..=top+2`, frameY += 54 first — the edit lane's write) spawns
/// FOUR type-11 dusts at `(i*16, k*16)`, box 16×16, defaults. The walk,
/// the null-tile creation, the frame bump and the achievement notify
/// (`:37627`) are the caller's; this is the dust half only.
pub fn unlock_door_dust(
    i: i32,
    top: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for k in top..=top + 2 {
        for _ in 0..4 {
            dust.new_dust(
                rng,
                ctx,
                world,
                Vec2::new((i * 16) as f32, (k * 16) as f32),
                16,
                16,
                11,
                0.0,
                0.0,
                0,
                Color::default(),
                1.0,
            );
        }
    }
}

/// `WorldGen.paintColor(color)` (`WorldGen.cs:43921-44034`) as a dust
/// `Color`: the byte table is already verbatim in
/// `tile_light::palette::paint_color_bytes` (the map/scanner consumers);
/// the C# starts from `Color.White` (alpha 255) and the ONLY alpha arm
/// is color 30's `white.A = 150` (`:44026-44032`, research/66 §2 —
/// d54's `paint_color_full` had it right; g14's forced 255 fixed).
fn paint_color(color: u8) -> Color {
    let [r, g, b] = crate::tile_light::paint_color_bytes(color);
    Color::new(r, g, b, if color == 30 { 150 } else { 255 })
}


/// Site 44074 — `WorldGen.paintEffect(x, y, color, oldColor)`
/// (`WorldGen.cs:44064-44085`): `color == 0` falls back to `oldColor`,
/// then TEN type-143 dusts at `(x*16, y*16)`, box 16×16, speeds 0,
/// alpha 50, the paint colour; after EACH spawn `genRand.Next(2) == 0`
/// sets noGravity + scale ×1.2, else scale ×0.5. The gate rides `gen`
/// (the C#'s genRand), the spawn internals ride `rng` (Main.rand).
pub fn paint_effect_dust(
    x: i32,
    y: i32,
    color: u8,
    old_color: u8,
    rng: &mut UnifiedRandom,
    gen: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let color2 = if color == 0 { old_color } else { color };
    let new_color = paint_color(color2);
    for _ in 0..10 {
        let idx = dust.new_dust(
            rng,
            ctx,
            world,
            Vec2::new((x * 16) as f32, (y * 16) as f32),
            16,
            16,
            143,
            0.0,
            0.0,
            50,
            new_color,
            1.0,
        );
        let d = dust.slot_mut(idx);
        if gen.next_max(2) == 0 {
            d.no_gravity = true;
            d.scale *= 1.2;
        } else {
            d.scale *= 0.5;
        }
    }
}

/// `WorldGen.coatingColor(coating)` (`WorldGen.cs:44036-44044`): the
/// two coating colours, everything else transparent.
pub fn coating_color(coating: u8) -> Color {
    match coating {
        1 => Color::new(235, 170, 255, 255),
        2 => Color::new(180, 245, 255, 255),
        _ => Color::TRANSPARENT,
    }
}

/// Site 44097 — `WorldGen.paintCoatEffect(x, y, paintCoatId, oldColors)`
/// (`WorldGen.cs:44087-44108`): TEN type-143 dusts, alpha 50; the base
/// colour is `coatingColor(paintCoatId)`, but when `paintCoatId == 0`
/// and the old-coat list is non-empty each dust picks
/// `oldColors[Main.rand.Next(oldColors.Count)]` (the pick draw runs
/// BEFORE the spawn internals, per iteration). The post-spawn
/// `genRand.Next(2)` gate is `paintEffect`'s, verbatim.
pub fn paint_coat_effect_dust(
    x: i32,
    y: i32,
    paint_coat_id: u8,
    old_colors: &[Color],
    rng: &mut UnifiedRandom,
    gen: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    let color = coating_color(paint_coat_id);
    for _ in 0..10 {
        let mut new_color = color;
        if paint_coat_id == 0 && !old_colors.is_empty() {
            new_color = old_colors[rng.next_max(old_colors.len() as i32) as usize];
        }
        let idx = dust.new_dust(
            rng,
            ctx,
            world,
            Vec2::new((x * 16) as f32, (y * 16) as f32),
            16,
            16,
            143,
            0.0,
            0.0,
            50,
            new_color,
            1.0,
        );
        let d = dust.slot_mut(idx);
        if gen.next_max(2) == 0 {
            d.no_gravity = true;
            d.scale *= 1.2;
        } else {
            d.scale *= 0.5;
        }
    }
}

/// The shared five-type-50 burst — `KillActuator` (`:58032-58035`) and
/// `KillWire`/`2`/`3`/`4` (`:58132-58135` and peers) carry the identical
/// loop: FIVE type-50 dusts at `(i*16, j*16)`, box 16×16, defaults, no
/// explicit draws (only the pool internals roll).
fn five_type50(
    i: i32,
    j: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for _ in 0..5 {
        dust.new_dust(
            rng,
            ctx,
            world,
            Vec2::new((i * 16) as f32, (j * 16) as f32),
            16,
            16,
            50,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        );
    }
}

/// Site 58034 — `WorldGen.KillActuator`'s dust (`WorldGen.cs:58032-
/// 58035`): the actuator bit comes off (the edit lane's
/// `edit::kill_actuator`), the sound (0) and the item drop (849) are the
/// caller's, then the five type-50 puffs.
pub fn kill_actuator_dust(
    i: i32,
    j: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    five_type50(i, j, rng, dust, ctx, world);
}

/// Sites 58134/58164/58194/58224 — `WorldGen.KillWire` / `KillWire2` /
/// `KillWire3` / `KillWire4`'s dust (`WorldGen.cs:58132-58135`,
/// `:58162-58165`, `:58192-58195`, `:58222-58225`): the four wire
/// colours' removals carry the SAME five type-50 puffs (the port's
/// `edit::kill_wire` is one colour-parameterised function, so this is
/// one too). Sound (0) + item drop (530) stay the caller's.
pub fn kill_wire_dust(
    i: i32,
    j: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    five_type50(i, j, rng, dust, ctx, world);
}

/// Site 60287 — `WorldGen.PlaceTile`'s kind-22/140 dust arm
/// (`WorldGen.cs:60283-60289`): inside the `!mute` block, after the
/// placement sound switch, kinds 22 (Demonite Ore) and 140 (the
/// inactive-stone variant) spawn THREE type-14 dusts at `(i*16, j*16)`,
/// box 16×16, defaults. The caller gates on `!mute && (kind == 22 ||
/// kind == 140)`; this is the dust half only (the sound switch is the
/// audio lane's, `edit::place_tile_sound` already carries the table).
pub fn place_tile_ore_dust(
    i: i32,
    j: i32,
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
) {
    for _ in 0..3 {
        dust.new_dust(
            rng,
            ctx,
            world,
            Vec2::new((i * 16) as f32, (j * 16) as f32),
            16,
            16,
            14,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> World {
        World::new("test".into(), 200, 200)
    }

    fn ctx() -> SpawnCtx {
        SpawnCtx {
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(3200.0, 3200.0),
            max_dust_to_draw: 6000,
            ..SpawnCtx::default()
        }
    }

    fn active(pool: &DustPool) -> Vec<usize> {
        (0..6000).filter(|&i| pool.slot(i).active).collect()
    }

    /// The two streams agree after the call: draw a tail from both and
    /// compare (the census witness shape — every draw accounted for).
    fn streams_agree(a: &mut UnifiedRandom, b: &mut UnifiedRandom) {
        for _ in 0..16 {
            assert_eq!(a.next(), b.next());
        }
    }

    /// W4-g14 — site 37624: twelve type-11 dusts (4 × 3 cells), the
    /// anchors on the 16px grid, no explicit draws — a mirrored stream
    /// through the same pool calls lands identical records.
    #[test]
    fn w4_g14_unlock_door_spawns_twelve_type11() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        let mut mpool = DustPool::new();
        let mut mirror = UnifiedRandom::new(7);
        unlock_door_dust(5, 10, &mut rng, &mut pool, &c, &w);
        for k in 10..=12 {
            for _ in 0..4 {
                mpool.new_dust(
                    &mut mirror, &c, &w, Vec2::new(80.0, (k * 16) as f32),
                    16, 16, 11, 0.0, 0.0, 0, Color::default(), 1.0,
                );
            }
        }
        assert_eq!(active(&pool).len(), 12);
        for i in 0..12 {
            assert_eq!(pool.slot(i).dust_type, 11);
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
        }
        streams_agree(&mut rng, &mut mirror);
    }

    /// W4-g14 — site 44074: ten type-143 dusts, alpha 50, the paint
    /// colour; the genRand gate splits noGravity×1.2 vs ×0.5 — the
    /// mirror replays the SAME two-stream order (internals on rng,
    /// gate on gen) and every record matches.
    #[test]
    fn w4_g14_paint_effect_two_stream_order() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(11);
        let mut gen = UnifiedRandom::new(23);
        let mut mpool = DustPool::new();
        let mut mrng = UnifiedRandom::new(11);
        let mut mgen = UnifiedRandom::new(23);
        paint_effect_dust(3, 4, 0, 9, &mut rng, &mut gen, &mut pool, &c, &w);
        let want_color = paint_color(9); // color==0 falls back to oldColor
        for _ in 0..10 {
            let idx = mpool.new_dust(
                &mut mrng, &c, &w, Vec2::new(48.0, 64.0),
                16, 16, 143, 0.0, 0.0, 50, want_color, 1.0,
            );
            let d = mpool.slot_mut(idx);
            if mgen.next_max(2) == 0 {
                d.no_gravity = true;
                d.scale *= 1.2;
            } else {
                d.scale *= 0.5;
            }
        }
        assert_eq!(active(&pool).len(), 10);
        for i in 0..10 {
            assert_eq!(pool.slot(i).dust_type, 143);
            assert_eq!(pool.slot(i).alpha, 50);
            assert_eq!(pool.slot(i).color, want_color);
            assert_eq!(pool.slot(i).scale, mpool.slot(i).scale);
            assert_eq!(pool.slot(i).no_gravity, mpool.slot(i).no_gravity);
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
        }
        streams_agree(&mut rng, &mut mrng);
        streams_agree(&mut gen, &mut mgen);
    }

    /// W4-g14 — site 44097: the id-0 old-coat pick draws one
    /// `Next(count)` per dust BEFORE the internals; a non-zero id draws
    /// nothing extra and every dust carries `coatingColor(id)`.
    #[test]
    fn w4_g14_paint_coat_pick_order_and_fixed_colour() {
        let w = world();
        let c = ctx();
        let old = [coating_color(1), coating_color(2)];
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(31);
        let mut gen = UnifiedRandom::new(37);
        let mut mpool = DustPool::new();
        let mut mrng = UnifiedRandom::new(31);
        let mut mgen = UnifiedRandom::new(37);
        paint_coat_effect_dust(2, 2, 0, &old, &mut rng, &mut gen, &mut pool, &c, &w);
        for _ in 0..10 {
            let pick = old[mrng.next_max(2) as usize];
            let idx = mpool.new_dust(
                &mut mrng, &c, &w, Vec2::new(32.0, 32.0),
                16, 16, 143, 0.0, 0.0, 50, pick, 1.0,
            );
            let d = mpool.slot_mut(idx);
            if mgen.next_max(2) == 0 {
                d.no_gravity = true;
                d.scale *= 1.2;
            } else {
                d.scale *= 0.5;
            }
        }
        assert_eq!(active(&pool).len(), 10);
        for i in 0..10 {
            assert_eq!(pool.slot(i).color, mpool.slot(i).color);
            assert_eq!(pool.slot(i).scale, mpool.slot(i).scale);
        }
        streams_agree(&mut rng, &mut mrng);
        streams_agree(&mut gen, &mut mgen);
        // The fixed-id path: no pick draws, every dust the id's colour.
        let mut pool2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(31);
        let mut gen2 = UnifiedRandom::new(37);
        paint_coat_effect_dust(2, 2, 1, &old, &mut rng2, &mut gen2, &mut pool2, &c, &w);
        for i in 0..10 {
            assert_eq!(pool2.slot(i).color, coating_color(1));
        }
    }

    /// W4-d54 (research/66 §3 — neither lane had one) — the color-30
    /// alpha-150 paint arm (`WorldGen.cs:44026-44032`): `paintColor(30)`
    /// is (200,200,200,150), every other colour keeps White's alpha
    /// 255; the `color == 0` fold takes `oldColor` first, so
    /// `paintEffect(0, old=30)` spawns ten alpha-150-coloured dusts.
    #[test]
    fn w4_d54_paint_color30_alpha150() {
        assert_eq!(paint_color(30), Color::new(200, 200, 200, 150));
        assert_eq!(paint_color(1), Color::new(255, 0, 0, 255));
        assert_eq!(paint_color(0), Color::new(255, 255, 255, 255));
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(11);
        let mut gen = UnifiedRandom::new(23);
        paint_effect_dust(3, 4, 0, 30, &mut rng, &mut gen, &mut pool, &c, &w);
        assert_eq!(active(&pool).len(), 10);
        for i in 0..10 {
            assert_eq!(pool.slot(i).color, Color::new(200, 200, 200, 150));
        }
    }

    /// W4-g14 — sites 58034/58134/58164/58194/58224: the actuator and
    /// the four wire colours all spawn five type-50 puffs with no
    /// explicit draws; same seed ⇒ identical records across the five
    /// entry points.
    #[test]
    fn w4_g14_hardware_breaks_spawn_five_type50() {
        let w = world();
        let c = ctx();
        let mut base = DustPool::new();
        let mut brng = UnifiedRandom::new(41);
        kill_actuator_dust(6, 7, &mut brng, &mut base, &c, &w);
        assert_eq!(active(&base).len(), 5);
        for i in 0..5 {
            assert_eq!(base.slot(i).dust_type, 50);
            assert!(base.slot(i).position.x >= 96.0 && base.slot(i).position.x < 112.0);
        }
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(41);
        kill_wire_dust(6, 7, &mut rng, &mut pool, &c, &w);
        for i in 0..5 {
            assert_eq!(pool.slot(i).position, base.slot(i).position);
            assert_eq!(pool.slot(i).velocity, base.slot(i).velocity);
        }
        streams_agree(&mut rng, &mut brng);
    }

    /// W4-g14 — site 60287: the PlaceTile 22/140 arm spawns three
    /// type-14 dusts, no explicit draws; a mirrored stream lands the
    /// same records.
    #[test]
    fn w4_g14_place_tile_ore_spawns_three_type14() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(61);
        let mut mpool = DustPool::new();
        let mut m = UnifiedRandom::new(61);
        place_tile_ore_dust(8, 9, &mut rng, &mut pool, &c, &w);
        for _ in 0..3 {
            mpool.new_dust(
                &mut m, &c, &w, Vec2::new(128.0, 144.0), 16, 16, 14,
                0.0, 0.0, 0, Color::default(), 1.0,
            );
        }
        assert_eq!(active(&pool).len(), 3);
        for i in 0..3 {
            assert_eq!(pool.slot(i).dust_type, 14);
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
        }
        streams_agree(&mut rng, &mut m);
    }

    /// W4-g14 — sites 29649/29657: the full ash-shake draw census on a
    /// short column (height 4, hitTree false). Per row the mirror
    /// replays: gate1 (row >= -2: Next(3)), 5 × (RandomVector2 +
    /// internals + Circular), gate2 (row >= -2.8: Next(3)), 10 ×
    /// (RandomVector2 + NextFloat + internals + Circular + Next(2)),
    /// gore args (RandomVector2 + Next(3)). Records and both stream
    /// tails must match.
    #[test]
    fn w4_g14_ash_shake_full_draw_census() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(53);
        let mut mpool = DustPool::new();
        let mut m = UnifiedRandom::new(53);
        tree_grow_fx_ash_shake(10, 12, 4, false, &mut rng, &mut pool, &c, &w);
        let vector = Vec2::new(10.0, 12.0) * 16.0 + Vec2::new(8.0, 8.0);
        let mut num = 0i32;
        let mut spawned = 0usize;
        while num > -4 {
            let anchor = vector + Vec2::new(-16.0, (num * 16) as f32);
            if num >= -2 && m.next_max(3) != 0 {
                for _ in 0..5 {
                    let pos = anchor + random_vector2(&mut m, -30.0, 30.0);
                    let idx = mpool.new_dust_direct(
                        &mut m, &c, &w, pos, 4, 4, 31, 0.0, -3.0, 100,
                        Color::default(), 1.0,
                    );
                    mpool.slot_mut(idx).velocity =
                        next_vector2_circular(&mut m, 2.0, 2.0) + Vec2::new(0.0, -0.6);
                    spawned += 1;
                }
            }
            if num as f32 >= -4.0 * 0.7 && m.next_max(3) != 0 {
                for _ in 0..10 {
                    let pos = anchor + random_vector2(&mut m, -30.0, 30.0);
                    let scale = 1.0 + 0.7 * m.next_float();
                    let idx = mpool.new_dust_direct(
                        &mut m, &c, &w, pos, 4, 4, 6, 0.0, -4.0, 0,
                        Color::default(), scale,
                    );
                    let d = mpool.slot_mut(idx);
                    d.velocity =
                        next_vector2_circular(&mut m, 2.0, 2.0) + Vec2::new(0.0, -0.6);
                    if m.next_max(2) == 0 {
                        d.velocity *= 2.0;
                        d.no_gravity = true;
                    }
                    d.velocity += Vec2::new(0.0, -2.0);
                    spawned += 1;
                }
            }
            let _ = random_vector2(&mut m, -40.0, 40.0);
            let _ = 61 + m.next_max(3);
            num -= 1;
        }
        assert_eq!(active(&pool).len(), spawned);
        for i in 0..spawned {
            assert_eq!(pool.slot(i).dust_type, mpool.slot(i).dust_type);
            assert_eq!(pool.slot(i).position, mpool.slot(i).position);
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
            assert_eq!(pool.slot(i).no_gravity, mpool.slot(i).no_gravity);
            assert_eq!(pool.slot(i).scale, mpool.slot(i).scale);
        }
        streams_agree(&mut rng, &mut m);
    }

    /// W4-g14 — the hitTree variant swaps the counts (10/16), skips the
    /// gore draws entirely and lifts the smoke by -1 instead of -2;
    /// the gate rows still short-circuit identically.
    #[test]
    fn w4_g14_ash_shake_hit_tree_counts_and_no_gore() {
        let w = world();
        let c = ctx();
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(59);
        tree_grow_fx_ash_shake(10, 12, 2, true, &mut rng, &mut pool, &c, &w);
        // Height 2: rows 0, -1. Gate1 rows >= -1: both; gate2 rows >=
        // -1.4: both. Whether the gates pass is seed-fixed; the census
        // mirror pins it.
        let mut m = UnifiedRandom::new(59);
        let mut mpool = DustPool::new();
        let vector = Vec2::new(10.0, 12.0) * 16.0 + Vec2::new(8.0, 8.0);
        let mut num = 0i32;
        let mut spawned = 0usize;
        while num > -2 {
            let anchor = vector + Vec2::new(-16.0, (num * 16) as f32);
            if num >= -1 && m.next_max(3) != 0 {
                for _ in 0..10 {
                    let pos = anchor + random_vector2(&mut m, -30.0, 30.0);
                    let idx = mpool.new_dust_direct(
                        &mut m, &c, &w, pos, 4, 4, 31, 0.0, -3.0, 100,
                        Color::default(), 1.0,
                    );
                    mpool.slot_mut(idx).velocity =
                        next_vector2_circular(&mut m, 2.0, 2.0) + Vec2::new(0.0, -0.6);
                    spawned += 1;
                }
            }
            if num as f32 >= -2.0 * 0.7 && m.next_max(3) != 0 {
                for _ in 0..16 {
                    let pos = anchor + random_vector2(&mut m, -30.0, 30.0);
                    let scale = 1.0 + 0.7 * m.next_float();
                    let idx = mpool.new_dust_direct(
                        &mut m, &c, &w, pos, 4, 4, 6, 0.0, -4.0, 0,
                        Color::default(), scale,
                    );
                    let d = mpool.slot_mut(idx);
                    d.velocity =
                        next_vector2_circular(&mut m, 2.0, 2.0) + Vec2::new(0.0, -0.6);
                    if m.next_max(2) == 0 {
                        d.velocity *= 2.0;
                        d.no_gravity = true;
                    }
                    d.velocity += Vec2::new(0.0, -1.0);
                    spawned += 1;
                }
            }
            num -= 1;
        }
        assert_eq!(active(&pool).len(), spawned);
        for i in 0..spawned {
            assert_eq!(pool.slot(i).velocity, mpool.slot(i).velocity);
        }
        streams_agree(&mut rng, &mut m);
    }
}
