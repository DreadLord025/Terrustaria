//! World-tick `Dust.NewDust` sites — the WorldGen editing lane (W4-d54,
//! sites 1-10 of the 40-site census in `docs/research/65-dust-world-sites-
//! claims.md`). The d51/d52 drain convention applies: each body replays its
//! `Dust.NewDust` calls on the stream of the `Main.SwapRandom` pass that
//! executes it in the C# (never a new stream, S1-A-10), then applies the
//! per-site post-arms. One draw = one stream step.
//!
//! Two streams ride these sites:
//! - `Main.rand` — the editing runs from `Player.ItemCheck` /
//!   `TileInteractionsUse` under the `UpdatePlayers` swap
//!   (`Main.cs:17754`); the caller passes that stream in.
//! - `WorldGen.genRand` — the paint post-arm `Next(2)` rolls; the caller
//!   passes the port's runtime `genRand` stand-in (`World::frame_rng`,
//!   `world.rs:53-60`, the `SquareTileFrame` precedent). The worldgen
//!   pipeline streams are untouched — these are runtime editing draws.
//!
//! Batch-1 scope: the ash-tree shake (`TreeGrowFX_AshShake`,
//! `WorldGen.cs:29639-29672`, sites 29649/29657), the golden-door unlock
//! burst (`UnlockDoor`, `:37599-37628`, site 37624), the paint/coating
//! effects (`paintEffect`/`paintCoatEffect`, `:44064-44108`, sites
//! 44074/44097) and the cutter's five-dust bursts (`KillActuator`/
//! `KillWire`×4, `:58022-58229`, sites 58034/58134/58164/58194/58224 —
//! one shared body, the C# repeats it verbatim per wire colour).
//!
//! Live seam: the cutter bursts drain from `world_sim/editing/wiring.rs`.
//! The ash-shake / unlock-door / paint bodies are LATENT — their C#
//! callers (`ShakeTree`/`TreeGrowFXCheck`, `Player.TileInteractionsUse`'s
//! key arms, `paintTile`/`paintWall`) have no port yet; the bodies are
//! ported verbatim ahead of those lanes (the d52 unported-arm precedent).
//!
//! The ash-shake's `Gore.NewGoreDirect` sibling (`:29669`, one per row
//! when `!hitTree`) is the gore lane's (W4-d55/g16) — its `Main.rand`
//! draws are a documented skip here, exactly as the rest of the port
//! treats unported gore (docs/ANALYSIS.md, the d51 convention).

use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World, TILE_PX};

/// `Utils.RandomVector2(random, min, max)` (`Utils.cs:1247-1250`) — two
/// `NextDouble` draws, `(max-min)*NextDouble()+min` per axis.
fn random_vector2(rng: &mut UnifiedRandom, min: f32, max: f32) -> Vec2 {
    let x = (max - min) * rng.next_double() as f32 + min;
    let y = (max - min) * rng.next_double() as f32 + min;
    Vec2::new(x, y)
}

/// `Utils.NextVector2Circular` (`Utils.cs:1266-1269`) — the unit draw
/// (`NextVector2Unit`, one `NextFloat`, `Utils.cs:1261-1264`) THEN a
/// second `NextFloat` radius scale (the npc_ai_empress helper's shape).
fn next_vector2_circular(rng: &mut UnifiedRandom, half_w: f32, half_h: f32) -> Vec2 {
    let angle = std::f32::consts::PI * 2.0 * rng.next_float();
    let unit = Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32);
    let r = rng.next_float();
    Vec2::new(unit.x * half_w * r, unit.y * half_h * r)
}

/// `WorldGen.paintColor(color)` (`WorldGen.cs:43921-44034`) as the full
/// XNA `Color`: the byte table is the shared
/// [`tr_core::tile_light::paint_color_bytes`] (the C# starts from
/// `Color.White` and overwrites RGB per arm — the table's default IS
/// white); the ONLY alpha arm is color 30's `white.A = 150` (`:44029`),
/// every other colour keeps `White`'s 255.
pub(crate) fn paint_color_full(color: u8) -> Color {
    let [r, g, b] = tr_core::tile_light::paint_color_bytes(color);
    Color::new(r, g, b, if color == 30 { 150 } else { 255 })
}

/// `WorldGen.coatingColor(coating)` (`WorldGen.cs:44036-44044`): the
/// two-row switch, default `Color.Transparent` (all-zero).
pub(crate) fn coating_color(coating: u8) -> Color {
    match coating {
        1 => Color::new(235, 170, 255, 255),
        2 => Color::new(180, 245, 255, 255),
        _ => Color::TRANSPARENT,
    }
}

/// The cutter's five type-50 puffs — `WorldGen.KillActuator`
/// (`WorldGen.cs:58032-58035`) and `KillWire`/`KillWire2`/`KillWire3`/
/// `KillWire4` (`:58132-58135`/`:58162-58165`/`:58192-58195`/
/// `:58222-58225`) share the body verbatim: after the item drop,
/// `for k in 0..5 { Dust.NewDust((i*16, j*16), 16, 16, 50) }`. Type 50
/// adds no per-type tweak (`Dust.cs:214-252`), so each spawn is exactly
/// the six default-block draws. The sound (`PlaySound(0)`) precedes the
/// drop in the C#; the port's cutter feedback placement is the existing
/// `wire_placed_feedback` seam (unchanged).
pub(crate) fn cutter_burst(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    tx: i32,
    ty: i32,
) {
    let pos = Vec2::new(tx as f32 * TILE_PX, ty as f32 * TILE_PX);
    for _ in 0..5 {
        dust.new_dust(rng, ctx, world, pos, 16, 16, 50, 0.0, 0.0, 0, Color::default(), 1.0);
    }
}

/// `WorldGen.UnlockDoor`'s dust burst (`WorldGen.cs:37615-37626`): for
/// each of the door's three rows (`k = num..=num+2`, `num` the resolved
/// `frameY == 594` top row), four type-11 puffs at `(i*16, k*16)`, 16×16.
/// The `frameY += 54` write, the `PlaySound(22)` and the achievement
/// notify belong to the unported `UnlockDoor` wrapper (the key-use lane);
/// this body is the dust half, called with the resolved top row. Type 11
/// adds no per-type tweak. LATENT — no live port caller yet.
#[allow(dead_code)]
pub(crate) fn unlock_door_burst(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    i: i32,
    top_y: i32,
) {
    for k in top_y..=top_y + 2 {
        let pos = Vec2::new(i as f32 * TILE_PX, k as f32 * TILE_PX);
        for _ in 0..4 {
            dust.new_dust(rng, ctx, world, pos, 16, 16, 11, 0.0, 0.0, 0, Color::default(), 1.0);
        }
    }
}

/// `WorldGen.paintEffect` (`WorldGen.cs:44064-44085`): ten type-143
/// puffs at `(x*16, y*16)`, 16×16, alpha 50, colour
/// `paintColor(color == 0 ? oldColor : color)`. Per dust, AFTER the
/// spawn: `genRand.Next(2) == 0` → `noGravity` + `scale *= 1.2`, else
/// `scale *= 0.5`. The `Next(2)` rides `genRand` (the caller's
/// `frame_rng` stand-in), the spawn's default block rides `Main.rand`
/// (`UpdatePlayers`). Type 143 adds no per-type tweak. LATENT — the
/// paint-sprayer use path has no port yet.
#[allow(dead_code)]
pub(crate) fn paint_effect(
    rng: &mut UnifiedRandom,
    gen: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    x: i32,
    y: i32,
    color: u8,
    old_color: u8,
) {
    let color2 = if color == 0 { old_color } else { color };
    let new_color = paint_color_full(color2);
    let pos = Vec2::new(x as f32 * TILE_PX, y as f32 * TILE_PX);
    for _ in 0..10 {
        let idx =
            dust.new_dust(rng, ctx, world, pos, 16, 16, 143, 0.0, 0.0, 50, new_color, 1.0);
        let d = dust.slot_mut(idx);
        if gen.next_max(2) == 0 {
            d.no_gravity = true;
            d.scale *= 1.2;
        } else {
            d.scale *= 0.5;
        }
    }
}

/// `WorldGen.paintCoatEffect` (`WorldGen.cs:44087-44108`): ten type-143
/// puffs, colour `coatingColor(paintCoatId)` — EXCEPT coat 0 with a
/// non-empty `oldColors` list, where each iteration FIRST draws
/// `Main.rand.Next(oldColors.Count)` and takes that entry (`:44093-
/// 44096`, the draw precedes the spawn). The post-arm `genRand.Next(2)`
/// is `paintEffect`'s verbatim. LATENT.
#[allow(dead_code)]
pub(crate) fn paint_coat_effect(
    rng: &mut UnifiedRandom,
    gen: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    x: i32,
    y: i32,
    coat_id: u8,
    old_colors: &[Color],
) {
    let color = coating_color(coat_id);
    let pos = Vec2::new(x as f32 * TILE_PX, y as f32 * TILE_PX);
    for _ in 0..10 {
        let mut new_color = color;
        if coat_id == 0 && !old_colors.is_empty() {
            new_color = old_colors[rng.next_max(old_colors.len() as i32) as usize];
        }
        let idx =
            dust.new_dust(rng, ctx, world, pos, 16, 16, 143, 0.0, 0.0, 50, new_color, 1.0);
        let d = dust.slot_mut(idx);
        if gen.next_max(2) == 0 {
            d.no_gravity = true;
            d.scale *= 1.2;
        } else {
            d.scale *= 0.5;
        }
    }
}

/// `WorldGen.TreeGrowFX_AshShake` (`WorldGen.cs:29639-29672`): the ash
/// tree's shake/grow burst, walked top-down (`num = 0; num > -height;
/// num--`) from `vector = (x,y)*16 + (8,8)`.
///
/// Per row, two gated loops (the `Next(3)` gate draws ONLY when its
/// position half passes — the C# `&&` short-circuit):
/// - `num >= -height/2` (integer division) && `Next(3) != 0`:
///   `hitTree ? 10 : 5` type-31 puffs — `NewDustDirect(vector + (-16,
///   num*16) + RandomVector2(-30,30), 4, 4, 31, 0, -3, 100)`, then
///   `.velocity = NextVector2Circular(2,2) + (0,-0.6)` (`:29649`).
/// - `(float)num >= (float)(-height)*0.7` && `Next(3) != 0`:
///   `hitTree ? 16 : 10` type-6 embers — `NewDustDirect(same position
///   draw, 4, 4, 6, 0, -4, 0, default, 1 + 0.7*NextFloat())` (the scale
///   ARGUMENT draws before the spawn's block), then `.velocity =
///   Circular(2,2) + (0,-0.6)`, the `Next(2) == 0` double-velocity +
///   `noGravity` gate, then `.velocity += (0, hitTree ? -1 : -2)`
///   (`:29657-29664`).
///
/// The `!hitTree` row's `Gore.NewGoreDirect` (`:29669`) is the gore
/// lane's documented skip (module doc). LATENT — `ShakeTree` /
/// `TreeGrowFXCheck` have no port yet.
#[allow(dead_code)]
pub(crate) fn ash_shake(
    rng: &mut UnifiedRandom,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    world: &World,
    x: i32,
    y: i32,
    height: i32,
    hit_tree: bool,
) {
    let vector = Vec2::new(x as f32 * TILE_PX + 8.0, y as f32 * TILE_PX + 8.0);
    let mut num = 0i32;
    while num > -height {
        // `:29644-29651` — the type-31 ash puffs.
        if num >= -height / 2 && rng.next_max(3) != 0 {
            let count = if hit_tree { 10 } else { 5 };
            for _ in 0..count {
                let pos = vector
                    + Vec2::new(-16.0, num as f32 * TILE_PX)
                    + random_vector2(rng, -30.0, 30.0);
                let idx = dust.new_dust_direct(
                    rng, ctx, world, pos, 4, 4, 31, 0.0, -3.0, 100, Color::default(), 1.0,
                );
                dust.slot_mut(idx).velocity =
                    next_vector2_circular(rng, 2.0, 2.0) + Vec2::new(0.0, -0.6);
            }
        }
        // `:29652-29666` — the type-6 embers.
        if num as f32 >= (-height) as f32 * 0.7 && rng.next_max(3) != 0 {
            let count = if hit_tree { 16 } else { 10 };
            for _ in 0..count {
                let pos = vector
                    + Vec2::new(-16.0, num as f32 * TILE_PX)
                    + random_vector2(rng, -30.0, 30.0);
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
        // `:29667-29670` — the gore sibling (W4-d55/g16 skip, module doc).
        num -= 1;
    }
}

#[cfg(test)]
mod tests {
    //! W4-d54 batch-1 dust witnesses — WRITTEN, NOT RUN (code-only mode,
    //! the d52 convention). Each mirror replays the C# draw census on a
    //! probe `UnifiedRandom` and compares `peek()` against the ported
    //! body's stream, then checks the post-spawn field writes.

    use super::*;

    /// A permissive whole-world ctx (the d52 `dust_ctx` house pattern):
    /// the pool never culls, so every spawn lands and the draw census
    /// matches the live empty-pool path.
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
    /// (`Dust.cs:153-193`): the two position `Next(box-4)` rolls (the box
    /// clamps to >= 5), the velocity `Next(-20,21)` pair, the frame
    /// `Next(3)`, the scale `Next(-20,21)`; then the per-type tweak — the
    /// 6/228 family adds one `Next(-10,6)` (`Dust.cs:214-220`); 50/11/
    /// 143/31 add nothing. ChildSafety is disabled (the pool default).
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

    /// `Utils.RandomVector2` — two `NextDouble` draws.
    fn rv2(r: &mut UnifiedRandom) {
        r.next_double();
        r.next_double();
    }

    /// `Utils.NextVector2Circular` — the unit `NextFloat` + the radius
    /// `NextFloat`.
    fn circular(r: &mut UnifiedRandom) {
        r.next_float();
        r.next_float();
    }

    fn drain(dust: &mut DustPool) {
        for s in dust.slots_mut() {
            s.active = false;
        }
    }

    /// `WorldGen.cs:58032-58035` — the cutter burst is exactly five
    /// type-50 default blocks, no post draws.
    #[test]
    fn w4_d54_cutter_burst_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        for i in 0..200 {
            cutter_burst(&mut rng, &mut dust, &ctx, &w, 10, 20);
            for _ in 0..5 {
                block(&mut probe, 50, 16, 16);
            }
            assert_eq!(rng.peek(), probe.peek(), "cutter stream @{i}");
            drain(&mut dust);
        }
        assert_ne!(rng.peek(), UnifiedRandom::new(54).peek(), "the burst drew");
    }

    /// `WorldGen.cs:37615-37626` — the unlock burst is 3 rows × 4
    /// type-11 blocks; the puffs land at the row's tile corner + the
    /// default block's `Next(12)+4` offsets.
    #[test]
    fn w4_d54_unlock_door_burst_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        for i in 0..100 {
            unlock_door_burst(&mut rng, &mut dust, &ctx, &w, 12, 30);
            for _ in 0..12 {
                block(&mut probe, 11, 16, 16);
            }
            assert_eq!(rng.peek(), probe.peek(), "unlock stream @{i}");
            drain(&mut dust);
        }
        // The first puff's position: (12*16 + Next(12)+4, 30*16 + Next(12)+4).
        let mut replay = UnifiedRandom::new(54);
        let px = 192.0 + replay.next_max(12) as f32 + 4.0;
        let py = 480.0 + replay.next_max(12) as f32 + 4.0;
        let mut dust2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(54);
        unlock_door_burst(&mut rng2, &mut dust2, &ctx, &w, 12, 30);
        let d = dust2.slot(0);
        assert!(d.active);
        assert_eq!(d.dust_type, 11);
        assert_eq!((d.position.x, d.position.y), (px, py));
        // Row 2's first puff (index 8) sits two tiles lower: 512 + Next(12)+4.
        let d8 = dust2.slot(8);
        assert!(d8.position.y >= 516.0 && d8.position.y < 528.0);
    }

    /// `WorldGen.cs:44072-44084` — ten type-143 blocks on `Main.rand`,
    /// each followed by ONE `genRand.Next(2)`; the two streams never
    /// interleave. Both post arms fire across the replay.
    #[test]
    fn w4_d54_paint_effect_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let mut rng = UnifiedRandom::new(54);
        let mut probe = UnifiedRandom::new(54);
        let mut gen = UnifiedRandom::new(7);
        let mut gen_probe = UnifiedRandom::new(7);
        let mut dust = DustPool::new();
        let mut saw_big = false;
        let mut saw_small = false;
        for i in 0..100 {
            paint_effect(&mut rng, &mut gen, &mut dust, &ctx, &w, 10, 20, 9, 0);
            for _ in 0..10 {
                block(&mut probe, 143, 16, 16);
                if gen_probe.next_max(2) == 0 {
                    saw_big = true;
                } else {
                    saw_small = true;
                }
            }
            assert_eq!(rng.peek(), probe.peek(), "paint main stream @{i}");
            assert_eq!(gen.peek(), gen_probe.peek(), "paint gen stream @{i}");
            drain(&mut dust);
        }
        assert!(saw_big && saw_small, "both Next(2) arms exercised");
    }

    /// `WorldGen.cs:44066-44071` + `:43921-44034` — the colour fold
    /// (`color == 0` takes `oldColor`) and the alpha-150 color-30 arm;
    /// the post-arm scale writes land on the returned slot.
    #[test]
    fn w4_d54_paint_effect_writes_the_color_and_scale() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let mut rng = UnifiedRandom::new(54);
        let mut gen = UnifiedRandom::new(7);
        let mut dust = DustPool::new();
        paint_effect(&mut rng, &mut gen, &mut dust, &ctx, &w, 10, 20, 0, 30);
        // Replay the first spawn: the block's scale roll is the LAST
        // Next(-20,21); re-derive it.
        let mut replay2 = UnifiedRandom::new(54);
        replay2.next_max(12);
        replay2.next_max(12);
        replay2.next_range(-20, 21);
        replay2.next_range(-20, 21);
        replay2.next_max(3);
        let scale = 1.0 + replay2.next_range(-20, 21) as f32 * 0.01;
        let mut gen_replay = UnifiedRandom::new(7);
        let big = gen_replay.next_max(2) == 0;
        let d = dust.slot(0);
        assert_eq!(d.color, Color::new(200, 200, 200, 150));
        assert_eq!(d.alpha, 50);
        assert_eq!(d.no_gravity, big);
        let want = if big { scale * 1.2 } else { scale * 0.5 };
        assert!((d.scale - want).abs() < 1e-6);
        // The non-zero color keeps its own id (no fold).
        assert_eq!(paint_color_full(1), Color::new(255, 0, 0, 255));
        assert_eq!(paint_color_full(0), Color::new(255, 255, 255, 255));
    }

    /// `WorldGen.cs:44090-44107` — coat 0 with a non-empty old-colour
    /// list draws `Next(count)` BEFORE each spawn; coats 1/2 draw
    /// nothing extra and carry the fixed coating colours.
    #[test]
    fn w4_d54_paint_coat_effect_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let olds = [Color::new(1, 2, 3, 255), Color::new(4, 5, 6, 255)];
        for coat in [0u8, 1, 2, 3] {
            let mut rng = UnifiedRandom::new(54);
            let mut probe = UnifiedRandom::new(54);
            let mut gen = UnifiedRandom::new(7);
            let mut gen_probe = UnifiedRandom::new(7);
            let mut dust = DustPool::new();
            for i in 0..50 {
                paint_coat_effect(&mut rng, &mut gen, &mut dust, &ctx, &w, 10, 20, coat, &olds);
                for _ in 0..10 {
                    if coat == 0 {
                        probe.next_max(2); // Next(oldColors.Count)
                    }
                    block(&mut probe, 143, 16, 16);
                    gen_probe.next_max(2);
                }
                assert_eq!(rng.peek(), probe.peek(), "coat {coat} main @{i}");
                assert_eq!(gen.peek(), gen_probe.peek(), "coat {coat} gen @{i}");
                drain(&mut dust);
            }
        }
        // The fixed coating colours + the transparent default.
        assert_eq!(coating_color(1), Color::new(235, 170, 255, 255));
        assert_eq!(coating_color(2), Color::new(180, 245, 255, 255));
        assert_eq!(coating_color(0), Color::TRANSPARENT);
        assert_eq!(coating_color(9), Color::TRANSPARENT);
    }

    /// `WorldGen.cs:29642-29671` — the ash-shake's full draw census:
    /// per row the two short-circuit `Next(3)` gates, then per puff the
    /// position `RandomVector2` (2), the type-6 scale argument (1, before
    /// the block), the 4×4→5-clamped default block (+ the type-6
    /// `Next(-10,6)` tweak), the velocity `Circular` (2) and the ember's
    /// `Next(2)` gate. The gore sibling draws NOTHING (the documented
    /// skip). Both `hitTree` arms replayed from several seeds.
    #[test]
    fn w4_d54_ash_shake_draws_match_the_oracle() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        for hit_tree in [false, true] {
            for seed in [54, 1, 99] {
                let height = 20;
                let mut rng = UnifiedRandom::new(seed);
                let mut probe = UnifiedRandom::new(seed);
                let mut dust = DustPool::new();
                ash_shake(&mut rng, &mut dust, &ctx, &w, 100, 100, height, hit_tree);
                let mut num = 0i32;
                while num > -height {
                    if num >= -height / 2 && probe.next_max(3) != 0 {
                        for _ in 0..if hit_tree { 10 } else { 5 } {
                            rv2(&mut probe);
                            block(&mut probe, 31, 4, 4);
                            circular(&mut probe);
                        }
                    }
                    if num as f32 >= (-height) as f32 * 0.7 && probe.next_max(3) != 0 {
                        for _ in 0..if hit_tree { 16 } else { 10 } {
                            rv2(&mut probe);
                            probe.next_float(); // the scale argument
                            block(&mut probe, 6, 4, 4);
                            circular(&mut probe);
                            probe.next_max(2);
                        }
                    }
                    num -= 1;
                }
                assert_eq!(
                    rng.peek(),
                    probe.peek(),
                    "ash shake hit={hit_tree} seed={seed}"
                );
                drain(&mut dust);
            }
        }
    }

    /// `WorldGen.cs:29657-29664` — the ember's post writes: velocity is
    /// the Circular + (0,-0.6) overwrite (the spawn's own velocity is
    /// dead), the `Next(2)` pass doubles it and lifts gravity, then the
    /// constant `(0, hitTree ? -1 : -2)` lift always lands.
    #[test]
    fn w4_d54_ash_shake_writes_the_ember_fields() {
        let w = World::new("t".into(), 200, 200);
        let ctx = dust_ctx(&w);
        let mut rng = UnifiedRandom::new(54);
        let mut dust = DustPool::new();
        // height 1: row 0 passes BOTH position gates; find a seed whose
        // first gate passes so the first spawn is a type-31 puff and the
        // ember lane starts at index 5 (hitTree=false).
        ash_shake(&mut rng, &mut dust, &ctx, &w, 100, 100, 1, false);
        let mut probe = UnifiedRandom::new(54);
        let gate31 = probe.next_max(3) != 0;
        let mut idx = 0usize;
        if gate31 {
            for _ in 0..5 {
                rv2(&mut probe);
                block(&mut probe, 31, 4, 4);
                circular(&mut probe);
                idx += 1;
            }
        }
        let gate6 = probe.next_max(3) != 0;
        if gate6 {
            // First ember: replay its draws to recover the velocity.
            rv2(&mut probe);
            probe.next_float(); // scale arg
            block(&mut probe, 6, 4, 4);
            let angle = std::f32::consts::PI * 2.0 * probe.next_float();
            let unit = Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32);
            let r = probe.next_float();
            let mut vel = Vec2::new(unit.x * 2.0 * r, unit.y * 2.0 * r) + Vec2::new(0.0, -0.6);
            let doubled = probe.next_max(2) == 0;
            if doubled {
                vel *= 2.0;
            }
            vel += Vec2::new(0.0, -2.0);
            let d = dust.slot(idx);
            assert_eq!(d.dust_type, 6);
            assert_eq!(d.no_gravity, doubled);
            assert!((d.velocity.x - vel.x).abs() < 1e-5);
            assert!((d.velocity.y - vel.y).abs() < 1e-5);
        }
    }
}
