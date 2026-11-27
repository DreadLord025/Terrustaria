//! `Main.snowing()` — the ambient snowfall (`Main.cs:13032-13116`, called
//! from `Main.Update` at `:17238`, right before `Sandstorm.EmitDust` at
//! `:17247`). W4-d130: this replaces the rain rig's biome-gated flake
//! approximation — vanilla snow is NOT a rain mode: it spawns `Dust` type 76
//! into the shared `Main.dust` pool wherever the scene metrics count snow
//! tiles at the surface, storm or no storm.
//!
//! Port notes:
//! * the RNG is the bare-`Main.rand` stand-in, `NamedRngs::global()` (g11
//!   research/83-addendum F1, adjudicated W4-d130): `:17238` sits in
//!   `Main.Update` BEFORE `DoUpdateInWorld` (`:17280` → the `SwapRandom`
//!   wrapper `:17732`) — every swap scope in the game (`:17732-17809`,
//!   `:17887`, `WorldGen.cs:71529`) is nested inside that block, so no swap
//!   is live at the call and `RandomSwap.Dispose` (`Main.cs:116-119`) has
//!   restored the bare instance seeded at `Initialize` (`:6463`). The
//!   earlier private splitmix (the WEATHER-2 per-consumer convention) was
//!   WRONG for this site — snowing's draws must interleave with the global
//!   stream's other pre-swap consumers (the ambience scheduler, the
//!   chest-eat cues). The pool's `NewDust` rolls ride the same
//!   `UnifiedRandom` (`DustRng`);
//! * `Main.Camera.ScaledSize`/`ScaledPosition` are the world-space view
//!   (`screen / zoom`, the view's top-left) — the same fold the scene scan
//!   uses (`scene_scan.rs`'s `visual_area`);
//! * `Main.maxScreenW` is the HiDef 4096 (`Main.cs:3709` — the port runs
//!   HiDef, `app/frame/draw.rs`; g11 `13a1c48e`'s pin, folded). The earlier
//!   Reach-1920 pin (`:66715`/`:3715`) retired with the g11 census —
//!   `sand.rs`'s 1920 emit-budget pin stays its own recorded deviation;
//! * `NPC.IsADeerclopsNearScreen()` (`NPC.cs:44534-44537`) folds to the
//!   active-Deerclops scan (`npcsFoundForCheckActive[668]` — the per-frame
//!   found-active table, `NPC.cs:7229-7239`);
//! * `Main.snowDust` (`Main.cs:621`) is reset per `UpdateDust` head
//!   (`Dust.cs:428`) and incremented per live type-76 slot (`Dust.cs:1747`);
//!   the shared tick's arm maintains [`SnowDust`], the spawn budget reads it
//!   (`Main.cs:13057`);
//! * the flakes' AI is the type-76 arm of `Dust.UpdateDust`
//!   (`Dust.cs:1745-1763`) plus the shared standalone tails (the `!= 304`
//!   velocity drag `:2071-2074`, the rotation `:2079-2082`, the fadeIn ramp
//!   `:2083-2129`); the draw is `Main.DrawDust`'s generic path
//!   (`Main.cs:38968-39018`) with the 8×8 frame at `(4,4)` origin.

use glam::Vec2;
use tr_core::World;
use tr_core::particles::{Color, DUST_SENTINEL, Dust, DustPool, SpawnCtx};
use tr_core::scene::SceneMetrics;
use tr_core::UnifiedRandom;
use tr_render::sprite::SpriteRenderer;

use crate::assets;
use crate::rain::light_at;
use tr_core::lighting::LightMap;

/// `Main.maxScreenW` — the HiDef pin (see the module doc).
const MAX_SCREEN_W: f32 = 4096.0;
/// `SceneMetrics.SnowTileMax` (`SceneMetrics.cs:932`).
const SNOW_TILE_MAX: i32 = 6000;

/// The snowfall's per-call inputs — the `Main` statics `snowing()` reads.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SnowInputs {
    /// `Main.gamePaused` (`Main.cs:13040`) — g11 `13a1c48e`'s env field,
    /// folded: the check runs INSIDE `snowing` (the C# shape), not only in
    /// the caller's cluster gate.
    pub game_paused: bool,
    /// `Main.Camera.ScaledPosition` — the view's world-px top-left.
    pub scaled_position: Vec2,
    /// `Main.Camera.ScaledSize` — the world-space view size.
    pub scaled_size: Vec2,
    /// `Main.cloudAlpha` — the visible intensity (`Rain::visual_alpha`'s
    /// unclamped source; the port passes `Rain.intensity`).
    pub cloud_alpha: f32,
    /// `Main.raining` — the velocity recipe's branch (`Main.cs:13092`).
    pub raining: bool,
    /// `Main.windSpeedCurrent`.
    pub wind_speed_current: f32,
    /// `Main.player[Main.myPlayer].velocity.Y` (`:13067-13070`).
    pub player_vy: f32,
    /// `Main.gfxQuality` (`:13057`).
    pub gfx_quality: f32,
    /// `NPC.IsADeerclopsNearScreen()` (`:13047-13052`, `:13104-13107`).
    pub deerclops_near: bool,
    /// `Main.remixWorld` (`:13034-13037`).
    pub remix_world: bool,
}

/// `Main.snowing()` (`Main.cs:13032-13116`). `rng` is the bare-`Main.rand`
/// stand-in — the caller passes `app.rngs.global()` (F1: `:17238` precedes
/// every `SwapRandom`, see the module doc). `metrics` is the stored
/// `Main.SceneMetrics` (the last scan), `snow_dust` the live type-76
/// population the previous `UpdateDust` head counted (`Main.snowDust`,
/// read ONCE per call — F4), `ctx` the pool's spawn statics.
/// `netMode != 2` (`:17234`) is the single-player fold.
pub(crate) fn snowing(
    metrics: &SceneMetrics,
    i: &SnowInputs,
    snow_dust: i32,
    rng: &mut UnifiedRandom,
    ctx: &SpawnCtx,
    world: &World,
    pool: &mut DustPool,
) {
    // `:13034-13037`: the remix world has no ambient snowfall.
    if i.remix_world {
        return;
    }
    // `:13040-13043`: paused / no snow tiles / below the surface —
    // nothing. The `gamePaused` disjunct is the C#'s own first term
    // (g11's env carried it; the caller's cluster gate is redundant,
    // not the check's home).
    if i.game_paused || metrics.snow_tile_count <= 0 || !metrics.surface_atmospherics {
        return;
    }
    // `:13044`: the intensity `pow(count/6000, 4 - 3*cloudAlpha)`.
    let mut num = ((metrics.snow_tile_count as f32 / SNOW_TILE_MAX as f32) as f64)
        .powf((4.0 - i.cloud_alpha * 3.0) as f64) as f32;
    // `:13045`: the per-screen budget — the inner `(int)` truncates the
    // 500*view-width factor BEFORE the cloud multiply, then `(int)` again.
    let mut num2 =
        ((500.0 * (i.scaled_size.x / MAX_SCREEN_W)) as i32 as f32 * (1.0 + 2.0 * i.cloud_alpha))
            as i32;
    // `:13046`: the attempt count `1 + 50*cloudAlpha`.
    let num3 = 1.0 + 50.0 * i.cloud_alpha;
    // `:13047-13052`: the Deerclops fades.
    let flag = i.deerclops_near;
    if flag {
        num /= 20.0;
        num2 /= 3;
    }
    // `:13053`: `for (int i = 0; (float)i < num3; i++)` — the int counter
    // shape (g11 F5; the earlier float accumulator was observation-equal).
    let mut attempt = 0;
    while (attempt as f32) < num3 {
        attempt += 1;
        // `:13057-13060`: the budget gate — `snowDust < num2*(q/2+0.5) +
        // num2*0.1` — BREAKS the loop (not a per-attempt skip). The counter
        // is the call's frozen `Main.snowDust` snapshot (F4), constant
        // across the attempts — the C# reads the static once per attempt
        // but `UpdateDust` only refreshes it next tick.
        if !((snow_dust as f32) < num2 as f32 * (i.gfx_quality / 2.0 + 0.5)
            + num2 as f32 * 0.1)
        {
            break;
        }
        // `:13061-13064`: the intensity roll.
        if !(rng.next_float() < num) {
            continue;
        }
        // `:13065-13066`: the spawn window — x across the view +750 each
        // side, y up to 50 px above the view top.
        let mut num4 = rng.next_max(i.scaled_size.x as i32 + 1500) - 750;
        let mut num5 = i.scaled_position.y as i32 - rng.next_max(50);
        // `:13067-13070`: a falling player pulls the window up.
        if i.player_vy > 0.0 {
            num5 -= i.player_vy as i32;
        }
        // `:13071-13078`: the edge pushes — the second `Next(5)` rolls
        // ONLY when the first missed (the else-if).
        if rng.next_max(5) == 0 {
            num4 = rng.next_max(500) - 500;
        } else if rng.next_max(5) == 0 {
            num4 = rng.next_max(500) + i.scaled_size.x as i32;
        }
        // `:13079-13082`: off-screen x spreads y over the view.
        if num4 < 0 || num4 as f32 > i.scaled_size.x {
            num5 += rng.next_max((i.scaled_size.y as f64 * 0.8) as i32)
                + (i.scaled_size.y as f64 * 0.1) as i32;
        }
        // `:13083-13086`: into world space; the open-sky gate —
        // `!tile.nactive() && wall == 0` (`nactive` = active && !actuated,
        // `Tile.cs:240-247`).
        num4 += i.scaled_position.x as i32;
        let num6 = num4 / 16;
        let num7 = num5 / 16;
        let Some(t) = world.tile(num6, num7) else {
            continue;
        };
        if (t.active() && !t.inactive()) || t.wall != 0 {
            continue;
        }
        // `:13088`: `Dust.NewDust(new Vector2(num4, num5), 10, 10, 76)` —
        // the pool's verbatim spawn; its rolls ride the caller's global
        // stream (`DustRng for UnifiedRandom`). A refused slot still takes
        // the recipe writes below (the C# writes `Main.dust[6000]` blindly).
        let idx = pool.new_dust(
            rng,
            ctx,
            world,
            Vec2::new(num4 as f32, num5 as f32),
            10,
            10,
            76,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        );
        // `:13089-13108`: the flake recipe, verbatim order.
        let d = pool.slot_mut(idx);
        d.scale += i.cloud_alpha * 0.2;
        d.velocity.y = 3.0 + rng.next_max(30) as f32 * 0.1;
        d.velocity.y *= d.scale;
        if !i.raining {
            // `:13094-13095`: the dry drift — wind + Next(-10,10)*0.1
            // (exclusive upper: -10..9), then the 15× wind lead. The
            // product is taken in f64 so the positive edge is the exact
            // 0.9 the C# decimal spelling denotes (the f32 product rounds
            // one ULP above it; the witness pins the inclusive bound).
            d.velocity.x = i.wind_speed_current
                + (rng.next_range(-10, 10) as f64 * 0.1) as f32;
            d.velocity.x += i.wind_speed_current * 15.0;
        } else {
            // `:13099-13100`: the storm drift — sqrt(|wind|)*sign(wind)*
            // (cloudAlpha+0.5)*10 + NextFloat()*0.2 - 0.1; the fall halves.
            // `Math.Sign(float)` returns 0 at exactly 0 (Rust's signum
            // returns 1 — the C# shape is mirrored).
            let sign = if i.wind_speed_current > 0.0 {
                1.0
            } else if i.wind_speed_current < 0.0 {
                -1.0
            } else {
                0.0
            };
            d.velocity.x = (i.wind_speed_current.abs() as f64).sqrt() as f32
                * sign
                * (i.cloud_alpha + 0.5)
                * 10.0
                + rng.next_float() * 0.2
                - 0.1;
            d.velocity.y *= 0.5;
        }
        d.velocity.y *= 1.0 + 0.3 * i.cloud_alpha;
        d.scale += i.cloud_alpha * 0.2;
        if flag {
            d.scale -= 0.5;
        }
        d.velocity *= 1.0 + i.cloud_alpha * 0.5;
    }
}

/// `Dust.UpdateDust`'s type-76 arm (`Dust.cs:1745-1763`) — the flake AI.
/// Returns false: the arm sits in the FIRST ladder's else-if chain (before
/// the `:2071` drag tail), so the shared tails still run — the caller
/// (`minecart::tick`) applies them after this arm, exactly like the C#.
/// `snow_dust` is the running `Main.snowDust` counter (`:1747`).
pub(crate) fn type_76_arm(
    d: &mut Dust,
    player_vy: f32,
    world: &World,
    snow_dust: &mut i32,
) -> bool {
    // `:1747`: the population count.
    *snow_dust += 1;
    // `:1748`: the slow growth.
    d.scale += 0.009;
    // `:1749-1753`: a falling player drags the flake down (lerp 0.04).
    if player_vy > 0.0 && d.fade_in == 0.0 && d.velocity.y < player_vy {
        d.velocity.y += (player_vy - d.velocity.y) * 0.04;
    }
    // `:1754-1757`: the fall also pushes the position (noLight gates it).
    if !d.no_light && player_vy > 0.0 {
        d.position.y += player_vy * 0.2;
    }
    // `:1758-1762`: over solid ground the flake dampens (the 10×10 box
    // centred on the dust, `pos - one*5`).
    if d.fade_in == 0.0
        && crate::player::Player::solid_collision(
            world,
            d.position.x - 5.0,
            d.position.y - 5.0,
            10.0,
            10.0,
        )
    {
        d.scale *= 0.9;
        d.velocity *= 0.25;
    }
    false
}

/// The shared standalone tails a type-76 slot takes after its arm — the
/// `!= 304` velocity drag (`Dust.cs:2071-2074`, the SECOND chain's final
/// else-if: 76 is not 41/44, so the drag runs), the rotation rebuild
/// (`:2079-2082`, 76 is not 79/268/304) and the fadeIn ramp's else arm
/// (`:2119-2128`, 76 is not 213/260: `scale -= 0.01`). The spawn sets
/// `fadeIn = 0`, so the ramp's first branch never fires for a flake; the
/// generic no-gravity tail (`:2335`) does not either (`noGravity = false`),
/// and the FIRST ladder's gravity else-if (`:1819`) is excluded by the
/// matched 76 arm.
pub(crate) fn type_76_tails(d: &mut Dust) {
    d.velocity.x *= 0.99;
    d.rotation += d.velocity.x * 0.5;
    d.scale -= 0.01;
}

/// `Main.DrawDust`'s generic path (`Main.cs:38836-39023`) over the pool's
/// type-76 slots: the ±1000/±1050 cull rect + else-kill
/// (`:38832`/`:38847`/`:39020-39022` — g67b `2c452eddb`'s fix applies to
/// EVERY DrawDust type, 76 included; the geometry helpers live in
/// `crate::rain`), the light sampled at `(pos + 4) / 16` (`:38968`), the
/// `GetAlpha` default formula (76 has no per-type arm — the light itself at
/// alpha 0), the 8×8 frame at `(4,4)` origin with `GetVisualRotation`/
/// `GetVisualScale` (`:39005`), and the full-dark kill (`:39015-39017`).
/// The `GetColor` overlay never fires (the spawn colour is default-black).
pub(crate) fn draw(
    pool: &mut DustPool,
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    light: &LightMap,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
) {
    let to_screen = |p: Vec2| {
        [
            (p.x - camera.x) * zoom + screen.x * 0.5,
            (p.y - camera.y) * zoom + screen.y * 0.5,
        ]
    };
    // `:38832`: the cull rect off `Main.screenPosition` (the view's top-left).
    let cull = crate::rain::splash_cull_rect(camera - screen * 0.5 / zoom, screen);
    for i in 0..DUST_SENTINEL {
        if !pool.slot(i).active || pool.slot(i).dust_type != 76 {
            continue;
        }
        let d = *pool.slot(i);
        // `:38847` + `:39020-39022`: outside the rect → deactivate, no draw.
        if !crate::rain::dust_in_cull_rect(d.position, cull) {
            pool.slot_mut(i).active = false;
            continue;
        }
        // `:38968`: the light at `(int)(pos + 4.0) / 16` per axis.
        let c = light_at(light, d.position + Vec2::splat(4.0));
        // `:39015-39017`: `newColor == Color.Black` deactivates — the
        // GetAlpha default at alpha 0 is the light itself.
        if !c.iter().any(|&v| v > 0.0) {
            pool.slot_mut(i).active = false;
            continue;
        }
        // The 76 sheet cell (`Dust.cs:174-188`): x = 760, y = 10*row, 8×8.
        let Some(uv) = lookups.dust_uv(d.frame_x, d.frame_y, d.frame_w as u32, d.frame_h as u32)
        else {
            continue;
        };
        let scale = tr_core::dust_table::get_visual_scale(&d);
        let size = 8.0 * scale * zoom;
        // The `(4,4)` origin: the renderer rotates about the top-left, so
        // the draw position shifts by the rotated origin offset.
        let rot = tr_core::dust_table::get_visual_rotation(&d);
        let (sn, cs) = rot.sin_cos();
        let ox = 4.0 * scale * zoom;
        let origin = [ox * cs - ox * sn, ox * sn + ox * cs];
        let p = to_screen(d.position);
        r.push_fx_sprite_rot(
            [p[0] - origin[0], p[1] - origin[1]],
            [size, size],
            uv,
            [c[0], c[1], c[2], 1.0],
            rot,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::minecart::live_type;
    use tr_core::tile::FLAG_ACTIVE;

    const SCREEN: Vec2 = Vec2::new(1600.0, 900.0);
    const CAMERA: Vec2 = Vec2::new(100.0 * 16.0, 95.0 * 16.0);

    /// A flat snow world: rows 95+ snow (147) on dirt, no walls.
    fn snow_world() -> World {
        // 200x200: `solid_collision` clamps its bottom scan to `height - 40`
        // (`Collision.cs:2700-2734`), so the 95-row ground needs a world tall
        // enough for the row to sit inside the scanned band.
        let mut w = World::new("snow".into(), 200, 200);
        for x in 0..200 {
            for y in 95..200 {
                let i = w.index(x, y);
                w.tiles[i].kind = 147;
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    fn metrics(count: i32, surface: bool) -> SceneMetrics {
        let mut m = SceneMetrics::new();
        m.snow_tile_count = count;
        m.surface_atmospherics = surface;
        m
    }

    fn inputs() -> SnowInputs {
        SnowInputs {
            game_paused: false,
            scaled_position: CAMERA - SCREEN * 0.5,
            scaled_size: SCREEN,
            cloud_alpha: 0.0,
            raining: false,
            wind_speed_current: 0.0,
            player_vy: 0.0,
            gfx_quality: 1.0,
            deerclops_near: false,
            remix_world: false,
        }
    }

    fn ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: CAMERA - SCREEN * 0.5,
            screen_size: SCREEN,
            max_dust_to_draw: 6000,
        }
    }

    #[test]
    fn the_gates_need_snow_tiles_at_the_surface_and_no_remix() {
        // `Main.cs:13034-13043`: the four early returns — remix, then the
        // `gamePaused || SnowTileCount <= 0 || !SurfaceAtmospherics` triple
        // (the pause disjunct is g11's env field, folded in-body). F7
        // re-pin (g11 `w4_g11_snowing_gates_before_any_draw`): every gate
        // draws NOTHING on the global stream.
        let w = snow_world();
        let mut rng = UnifiedRandom::new(7);
        let mut pool = DustPool::new();
        let mut i = inputs();
        snowing(&metrics(0, true), &i, 0, &mut rng, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "no snow tiles");
        snowing(&metrics(3000, false), &i, 0, &mut rng, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "below the surface");
        i.remix_world = true;
        snowing(&metrics(3000, true), &i, 0, &mut rng, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "the remix world");
        i.remix_world = false;
        i.game_paused = true;
        snowing(&metrics(3000, true), &i, 0, &mut rng, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "the pause gate");
        assert_eq!(rng, UnifiedRandom::new(7), "the gates precede any draw");
    }

    #[test]
    fn the_intensity_power_tracks_the_cloud_alpha() {
        // `Main.cs:13044`: `pow(count/6000, 4 - 3*cloudAlpha)` — at count
        // 3000 the dry intensity is 0.5^4 = 0.0625, a full storm 0.5^1 = 0.5.
        // The attempt count (`:13046`) climbs 1 → 51 with the same alpha.
        let w = snow_world();
        let m = metrics(3000, true);
        let mut dry = UnifiedRandom::new(11);
        let mut dry_pool = DustPool::new();
        let mut i = inputs();
        for _ in 0..600 {
            snowing(&m, &i, 0, &mut dry, &ctx(), &w, &mut dry_pool);
        }
        let mut storm = UnifiedRandom::new(11);
        let mut storm_pool = DustPool::new();
        i.cloud_alpha = 1.0;
        i.raining = true;
        for _ in 0..600 {
            snowing(&m, &i, 0, &mut storm, &ctx(), &w, &mut storm_pool);
        }
        let dry_n = live_type(&dry_pool, 76);
        let storm_n = live_type(&storm_pool, 76);
        assert!(dry_n > 0, "the dry snowfall still flakes");
        assert!(
            storm_n > dry_n * 4,
            "the storm flakes far denser: dry {dry_n}, storm {storm_n}"
        );
    }

    #[test]
    fn the_budget_gate_breaks_the_loop() {
        // `Main.cs:13057`: at quality 1 the budget is num2 * 1.1; the dry
        // 1600-px num2 at the HiDef maxScreenW 4096 is
        // `(int)(500*1600/4096) = 195` → 214.5. A snowDust already at 215
        // spawns NOTHING (the break, not a skip).
        let w = snow_world();
        let m = metrics(6000, true); // intensity 1: every attempt passes
        let mut s = UnifiedRandom::new(13);
        let mut pool = DustPool::new();
        let i = inputs();
        snowing(&m, &i, 215, &mut s, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "the budget break");
        assert_eq!(s, UnifiedRandom::new(13), "the break precedes any draw");
        // One under the budget the loop runs (and the pool fills).
        for _ in 0..30 {
            snowing(&m, &i, 214, &mut s, &ctx(), &w, &mut pool);
        }
        assert!(live_type(&pool, 76) > 0);
        // The gfxQuality scale: at 0.5 the budget is 195*0.75 + 19.5 = 165.75.
        let mut q = UnifiedRandom::new(13);
        let mut q_pool = DustPool::new();
        let mut qi = inputs();
        qi.gfx_quality = 0.5;
        snowing(&m, &qi, 166, &mut q, &ctx(), &w, &mut q_pool);
        assert_eq!(live_type(&q_pool, 76), 0, "the quality-scaled budget");
        assert_eq!(q, UnifiedRandom::new(13), "the quality break draws nothing");
    }

    #[test]
    fn the_deerclops_fades_the_intensity_and_budget() {
        // `Main.cs:13047-13052`: num /= 20, num2 /= 3 — at count 6000 the
        // dry intensity 1 → 0.05 and the budget 214.5 → 65*1.1 = 71.5.
        // The per-flake scale also drops 0.5 (`:13104-13107`).
        let w = snow_world();
        let m = metrics(6000, true);
        let mut near = UnifiedRandom::new(17);
        let mut near_pool = DustPool::new();
        let mut i = inputs();
        i.deerclops_near = true;
        for _ in 0..600 {
            snowing(&m, &i, 0, &mut near, &ctx(), &w, &mut near_pool);
        }
        let mut far = UnifiedRandom::new(17);
        let mut far_pool = DustPool::new();
        i.deerclops_near = false;
        for _ in 0..600 {
            snowing(&m, &i, 0, &mut far, &ctx(), &w, &mut far_pool);
        }
        let near_n = live_type(&near_pool, 76);
        let far_n = live_type(&far_pool, 76);
        assert!(near_n > 0);
        assert!(
            near_n * 5 < far_n,
            "the /20 intensity thins the snowfall: near {near_n}, far {far_n}"
        );
        // The scale fade: every near flake's spawn scale lost 0.5.
        for d in &near_pool.slots()[..DUST_SENTINEL] {
            if d.active && d.dust_type == 76 {
                assert!(d.scale < 1.0, "the -0.5 fade: {}", d.scale);
            }
        }
    }

    #[test]
    fn the_velocity_recipes_branch_on_raining() {
        // `Main.cs:13092-13101`: dry = wind + Next(-10,10)*0.1 + wind*15;
        // storm = sqrt(|wind|)*sign(wind)*(alpha+0.5)*10 + NextFloat()*0.2
        // - 0.1, and the fall halves. At wind 0.4, alpha 0: dry vx starts
        // 0.4 + jitter + 6.0; storm vx starts 0.632*0.5*10 = 3.162 + jitter.
        let w = snow_world();
        let m = metrics(6000, true);
        let mut i = inputs();
        i.wind_speed_current = 0.4;
        let mut dry = UnifiedRandom::new(19);
        let mut dry_pool = DustPool::new();
        for _ in 0..30 {
            snowing(&m, &i, 0, &mut dry, &ctx(), &w, &mut dry_pool);
        }
        i.raining = true;
        let mut storm = UnifiedRandom::new(19);
        let mut storm_pool = DustPool::new();
        for _ in 0..30 {
            snowing(&m, &i, 0, &mut storm, &ctx(), &w, &mut storm_pool);
        }
        let dry_d = dry_pool
            .slots()
            .iter()
            .find(|d| d.active && d.dust_type == 76)
            .expect("a dry flake");
        let storm_d = storm_pool
            .slots()
            .iter()
            .find(|d| d.active && d.dust_type == 76)
            .expect("a storm flake");
        // The final `*= 1 + alpha*0.5` is 1 at alpha 0 — the recipes hold.
        assert!(
            (dry_d.velocity.x - (0.4 + 6.0)).abs() <= 1.0,
            "dry vx {}",
            dry_d.velocity.x
        );
        let want = (0.4_f64).sqrt() as f32 * 0.5 * 10.0;
        assert!(
            (storm_d.velocity.x - want).abs() <= 0.1,
            "storm vx {} vs {}",
            storm_d.velocity.x,
            want
        );
        // The fall: dry vy = (3 + Next(30)*0.1) * scale; storm halves it.
        assert!(dry_d.velocity.y >= 3.0 * dry_d.scale - 1e-4);
        assert!(storm_d.velocity.y <= (3.0 + 2.9) * storm_d.scale * 0.5 + 1e-4);
    }

    #[test]
    fn the_spawn_is_deterministic_per_seed() {
        let w = snow_world();
        let m = metrics(4000, true);
        let i = inputs();
        let play = || {
            let mut s = UnifiedRandom::new(23);
            let mut pool = DustPool::new();
            for _ in 0..30 {
                snowing(&m, &i, 0, &mut s, &ctx(), &w, &mut pool);
            }
            pool.slots()[..DUST_SENTINEL]
                .iter()
                .filter(|d| d.active && d.dust_type == 76)
                .map(|d| (d.position, d.velocity, d.scale, d.frame_y))
                .collect::<Vec<_>>()
        };
        assert_eq!(play(), play());
    }

    #[test]
    fn the_76_arm_counts_grows_and_dampens() {
        // `Dust.cs:1745-1763`: the counter++, the +0.009 growth, the
        // player-fall lerp/push and the solid-box dampen.
        let w = snow_world();
        let mut d = Dust {
            active: true,
            dust_type: 76,
            scale: 1.0,
            velocity: Vec2::new(1.0, 2.0),
            position: Vec2::new(100.0 * 16.0, 50.0 * 16.0), // open air
            ..Dust::default()
        };
        let mut count = 0;
        type_76_arm(&mut d, 0.0, &w, &mut count);
        assert_eq!(count, 1);
        assert!((d.scale - 1.009).abs() < 1e-6);
        assert_eq!(d.velocity, Vec2::new(1.0, 2.0), "no fall, no dampen");
        // A falling player: vy 2 < 8 lerps up 0.04 and pushes y by 8*0.2.
        let y0 = d.position.y;
        type_76_arm(&mut d, 8.0, &w, &mut count);
        assert_eq!(count, 2);
        assert!((d.velocity.y - (2.0 + 6.0 * 0.04)).abs() < 1e-5);
        assert!((d.position.y - (y0 + 1.6)).abs() < 1e-4);
        // Over solid ground (row 95+): the 10×10 box hits, scale *0.9,
        // velocity *0.25.
        d.position = Vec2::new(100.0 * 16.0, 95.0 * 16.0);
        let s0 = d.scale;
        let v0 = d.velocity;
        type_76_arm(&mut d, 0.0, &w, &mut count);
        assert!((d.scale - (s0 + 0.009) * 0.9).abs() < 1e-5);
        assert_eq!(d.velocity, v0 * 0.25);
        // The tails: the second chain's drag (:2073), the rotation rebuild
        // from the DRAGGED velocity (:2081) and the -0.01 fade (:2127).
        let vx = d.velocity.x * 0.99;
        let r0 = d.rotation;
        let s1 = d.scale;
        type_76_tails(&mut d);
        assert!((d.velocity.x - vx).abs() < 1e-6);
        assert!((d.rotation - (r0 + vx * 0.5)).abs() < 1e-6);
        assert!((d.scale - (s1 - 0.01)).abs() < 1e-6);
    }

    #[test]
    fn the_spawn_recipe_matches_the_csharp_order() {
        // One pinned spawn: intensity 1 (count 6000, alpha 0) — every
        // attempt passes the NextFloat gate, so the first attempt's roll
        // order is x window, y window, the two edge Next(5)s, NewDust's
        // default block, then the recipe's Next(30) fall + Next(-10,10)
        // drift. The recipe invariants hold on every live flake.
        let w = snow_world();
        let m = metrics(6000, true);
        let i = inputs();
        let mut s = UnifiedRandom::new(29);
        let mut pool = DustPool::new();
        for _ in 0..30 {
            snowing(&m, &i, 0, &mut s, &ctx(), &w, &mut pool);
        }
        let n = live_type(&pool, 76);
        assert!(n > 0, "the pinned spawn flakes");
        for d in &pool.slots()[..DUST_SENTINEL] {
            if d.active && d.dust_type == 76 {
                assert_eq!(d.frame_x, 760, "10*76");
                assert!((0..=20).contains(&d.frame_y), "10*Next(3)");
                assert_eq!(d.alpha, 0);
                assert!(!d.no_gravity);
                // The dry recipe at wind 0: vx = Next(-10,10)*0.1 ∈ [-1,0.9].
                assert!(
                    (-1.0..=0.9).contains(&d.velocity.x),
                    "vx {}",
                    d.velocity.x
                );
                // vy = (3 + Next(30)*0.1) * scale ∈ [3,5.9]*scale.
                assert!(d.velocity.y >= 3.0 * d.scale - 1e-4);
                assert!(d.velocity.y <= 5.9 * d.scale + 1e-4);
            }
        }
    }

    // --- F7: the g11 draw-sequence mirrors, re-pinned to the global
    // `UnifiedRandom` stream (research/83-addendum; the `Snow` splitmix
    // retired). The rig mirrors g11's `snow_rig`: a big open world, the view
    // parked far inside it, the stream pinned to the seed.

    /// The g11 snow rig's inputs: view (4096,4096)-(5376,4816) inside a
    /// 512×512 all-air world, full snow count, gfx 0, player falling 5.
    fn g11_env() -> SnowInputs {
        SnowInputs {
            game_paused: false,
            scaled_position: Vec2::new(4096.0, 4096.0),
            scaled_size: Vec2::new(1280.0, 720.0),
            cloud_alpha: 0.0,
            raining: false,
            wind_speed_current: 0.3,
            player_vy: 5.0,
            gfx_quality: 0.0,
            deerclops_near: false,
            remix_world: false,
        }
    }

    fn g11_world() -> World {
        let w = World::new("snow".into(), 512, 512);
        w
    }

    fn g11_ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::new(4096.0, 4096.0),
            screen_size: Vec2::new(1280.0, 720.0),
            max_dust_to_draw: 6000,
        }
    }

    /// The six `NewDust` default-block draws (`Dust.cs:153-193`) for the
    /// 10×10 box — type 76 rolls no per-type tweak draw.
    fn default_block(rng: &mut UnifiedRandom, pos: Vec2) -> (Vec2, Vec2, f32) {
        let px = pos.x + rng.next_max(10 - 4) as f32 + 4.0;
        let py = pos.y + rng.next_max(10 - 4) as f32 + 4.0;
        let vx = rng.next_range(-20, 21) as f32 * 0.1;
        let vy = rng.next_range(-20, 21) as f32 * 0.1;
        let _ = rng.next_max(3); // frame row
        let scale = 1.0 + rng.next_range(-20, 21) as f32 * 0.01;
        (Vec2::new(px, py), Vec2::new(vx, vy), scale)
    }

    /// The `NewDust` off-screen cull (`Dust.cs:97-101`) for the g11 rig:
    /// screen_position (4096,4096), screen 1280×720, dCount 0 → margin 400,
    /// rect (3696,3696,2080,1520); XNA `Intersects` is strict `<`.
    fn snow_cull_passes(x: i32, y: i32) -> bool {
        3696 < x + 10 && x < 3696 + 2080 && 3696 < y + 10 && y < 3696 + 1520
    }

    /// F7 mirror 1 (g11 `w4_g11_snowing_mirrors_the_dry_arm_draw_sequence`):
    /// cloudAlpha 0 → ONE attempt, full snow count → the NextFloat gate
    /// always passes; the mirror walks every draw in C# order on the global
    /// stream and pins position/velocity/scale — including the `wind*15`
    /// dry tail. A culled spawn draws NO internals but the post-writes
    /// still roll onto the sentinel.
    #[test]
    fn w4_g11_snowing_mirrors_the_dry_arm_draw_sequence() {
        let w = g11_world();
        let m = metrics(6000, true);
        let i = g11_env();
        let mut pool = DustPool::new();
        let mut global = UnifiedRandom::new(0x5EED);
        snowing(&m, &i, 0, &mut global, &g11_ctx(), &w, &mut pool);
        let mut rng = UnifiedRandom::new(0x5EED);
        let _ = rng.next_float(); // the spawn gate (`:13061`)
        let mut num4 = rng.next_max(1280 + 1500) - 750;
        let mut num5 = 4096 - rng.next_max(50);
        num5 -= 5; // player_vy > 0 (`:13067-13070`)
        if rng.next_max(5) == 0 {
            num4 = rng.next_max(500) - 500;
        } else if rng.next_max(5) == 0 {
            num4 = rng.next_max(500) + 1280;
        }
        if num4 < 0 || num4 as f32 > 1280.0 {
            num5 += rng.next_max((720.0f64 * 0.8) as i32) + (720.0f64 * 0.1) as i32;
        }
        num4 += 4096;
        let passes = snow_cull_passes(num4, num5);
        let mut v = Vec2::ZERO;
        let mut s = 0.0;
        if passes {
            let (p, vel, scale) = default_block(&mut rng, Vec2::new(num4 as f32, num5 as f32));
            let d = pool.slot(0);
            assert!(d.active);
            assert_eq!(d.dust_type, 76);
            assert_eq!(d.position, p);
            v = vel;
            s = scale;
        } else {
            assert_eq!(live_type(&pool, 76), 0, "the cull refuses the spawn");
        }
        // `:13089-13091`: scale += 0 (cloudAlpha 0), the vy re-roll.
        v.y = 3.0 + rng.next_max(30) as f32 * 0.1;
        v.y *= s;
        // `:13094-13095`: the dry wind arm.
        v.x = 0.3 + rng.next_range(-10, 10) as f32 * 0.1;
        v.x += 0.3 * 15.0;
        // `:13102-13108`: the shared tail at cloudAlpha 0.
        v.y *= 1.0;
        if passes {
            let d = pool.slot(0);
            assert_eq!(d.velocity, v);
            assert_eq!(d.scale, s);
        } else {
            // The sentinel took the writes; its scale is the pool default
            // (0), so only the scale-independent x write is pinned.
            assert_eq!(pool.slot(DUST_SENTINEL).velocity.x, v.x);
        }
        assert_eq!(global, rng, "the whole sequence rides the global stream");
    }

    /// F7 mirror 2 (g11 `…_the_storm_arm_with_deerclops`): the sqrt/sign
    /// wind formula (negative wind → negative carry), the vy ×0.5, the
    /// cloudAlpha double scale bump + the ×(1+0.3a)/×(1+0.5a) tails, and
    /// the Deerclops −0.5 scale cut. cloudAlpha 0.5 → num3 = 26 attempts;
    /// the mirror walks them with the FROZEN counter input (F4 — the
    /// budget reads the call's snapshot, so the mirror's local spawned
    /// count only tracks the pool slots, not the gate). The 0.05 gate is
    /// seed-swept to guarantee at least one spawn.
    #[test]
    fn w4_g11_snowing_mirrors_the_storm_arm_with_deerclops() {
        let mut any_spawn = false;
        for seed in [99i32, 7, 0x5EED, 12345] {
            let w = g11_world();
            let m = metrics(6000, true);
            let mut i = g11_env();
            i.cloud_alpha = 0.5;
            i.raining = true;
            i.wind_speed_current = -0.4;
            i.deerclops_near = true;
            let mut pool = DustPool::new();
            let mut global = UnifiedRandom::new(seed);
            // F4: the counter is frozen per call — the budget never breaks
            // in this rig (cap 104*0.6 = 62.4 > 26 attempts).
            snowing(&m, &i, 0, &mut global, &g11_ctx(), &w, &mut pool);
            let mut rng = UnifiedRandom::new(seed);
            // num = 1/20 = 0.05; num2 = (int)(156*2)/3 = 104.
            let num = 1.0f32 / 20.0;
            let mut spawned = 0usize;
            let mut attempt = 0;
            while (attempt as f32) < 1.0 + 50.0 * 0.5 {
                attempt += 1;
                if !(rng.next_float() < num) {
                    continue;
                }
                let mut num4 = rng.next_max(1280 + 1500) - 750;
                let mut num5 = 4096 - rng.next_max(50) - 5;
                if rng.next_max(5) == 0 {
                    num4 = rng.next_max(500) - 500;
                } else if rng.next_max(5) == 0 {
                    num4 = rng.next_max(500) + 1280;
                }
                if num4 < 0 || num4 as f32 > 1280.0 {
                    num5 += rng.next_max((720.0f64 * 0.8) as i32) + (720.0f64 * 0.1) as i32;
                }
                num4 += 4096;
                let mut v = Vec2::ZERO;
                let mut s = 0.0;
                let in_view = snow_cull_passes(num4, num5);
                if in_view {
                    let (p, vel, scale) =
                        default_block(&mut rng, Vec2::new(num4 as f32, num5 as f32));
                    let d = pool.slot(spawned);
                    assert!(d.active);
                    assert_eq!(d.dust_type, 76);
                    assert_eq!(d.position, p);
                    v = vel;
                    s = scale;
                    spawned += 1;
                }
                s += 0.5 * 0.2;
                v.y = 3.0 + rng.next_max(30) as f32 * 0.1;
                v.y *= s;
                v.x = (0.4f64.sqrt() as f32) * -1.0 * (0.5 + 0.5) * 10.0
                    + rng.next_float() * 0.2
                    - 0.1;
                v.y *= 0.5;
                v.y *= 1.0 + 0.3 * 0.5;
                s += 0.5 * 0.2;
                s -= 0.5; // the Deerclops cut
                v *= 1.0 + 0.5 * 0.5;
                if in_view {
                    let d = pool.slot(spawned - 1);
                    assert_eq!(d.velocity, v);
                    assert_eq!(d.scale, s);
                }
            }
            assert_eq!(live_type(&pool, 76), spawned);
            assert_eq!(global, rng, "the storm sequence rides the global stream");
            any_spawn |= spawned > 0;
        }
        assert!(any_spawn, "the seed sweep must pass the 0.05 gate");
    }

    /// F7 mirror 3 (g11 `…_breaks_on_the_live_76_budget`, re-pinned to the
    /// counter input per F4): with the Deerclops throttle at cloudAlpha 0,
    /// num2 = 156/3 = 52 → cap 52×0.5 + 52×0.1 = 31.2; a `snowDust` counter
    /// of 40 breaks before any draw. The 4096 HiDef pin (F2) is what makes
    /// the 1280-px fixture's num2 156.
    #[test]
    fn w4_g11_snowing_breaks_on_the_counter_budget() {
        let w = g11_world();
        let m = metrics(6000, true);
        let mut i = g11_env();
        i.deerclops_near = true; // num2 = 156/3 = 52 → cap 31.2
        let mut pool = DustPool::new();
        let mut global = UnifiedRandom::new(1);
        snowing(&m, &i, 40, &mut global, &g11_ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0);
        assert_eq!(global, UnifiedRandom::new(1), "the budget break draws nothing");
    }
}
