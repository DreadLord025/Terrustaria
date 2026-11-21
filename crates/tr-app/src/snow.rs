//! `Main.snowing()` — the ambient snowfall (`Main.cs:13032-13116`, called
//! from `Main.Update` at `:17238`, right before `Sandstorm.EmitDust` at
//! `:17247`). W4-d130: this replaces the rain rig's biome-gated flake
//! approximation — vanilla snow is NOT a rain mode: it spawns `Dust` type 76
//! into the shared `Main.dust` pool wherever the scene metrics count snow
//! tiles at the surface, storm or no storm.
//!
//! Port notes:
//! * the RNG is this module's own splitmix stream (the per-consumer-stream
//!   convention — vanilla rolls `Main.rand`); the pool's `NewDust` rolls ride
//!   the same stream via `SplitmixRand`, exactly like `sand.rs`;
//! * `Main.Camera.ScaledSize`/`ScaledPosition` are the world-space view
//!   (`screen / zoom`, the view's top-left) — the same fold the scene scan
//!   uses (`scene_scan.rs`'s `visual_area`);
//! * `Main.maxScreenW` is pinned at its Reach value 1920
//!   (`Main.cs:66715`/`:3715` — the port's `MaxWorldViewSize.X` convention,
//!   `rain.rs`'s spawn rate uses the same pin; HiDef's 4096 arm is a
//!   documented drop);
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
use tr_render::sprite::SpriteRenderer;

use crate::assets;
use crate::rain::light_at;
use tr_core::lighting::LightMap;

/// `Main.maxScreenW` — the Reach pin (see the module doc).
const MAX_SCREEN_W: f32 = 1920.0;
/// `SceneMetrics.SnowTileMax` (`SceneMetrics.cs:932`).
const SNOW_TILE_MAX: i32 = 6000;

/// The snowfall's per-call inputs — the `Main` statics `snowing()` reads.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SnowInputs {
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

pub(crate) struct Snow {
    rng: u64,
}

impl Snow {
    pub(crate) fn new(seed: u64) -> Self {
        Self {
            rng: (seed ^ 0x2545_F491_4F6C_DD1D).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1,
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.rng = self.rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn rand01(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u32 << 24) as f32
    }

    /// `Main.rand.Next(max)` — `Next(0)` rolls 0 (XNA), the sand.rs convention.
    fn next_int(&mut self, max: u32) -> u32 {
        (self.next_u64() % max.max(1) as u64) as u32
    }

    /// `Main.snowing()` (`Main.cs:13032-13116`). `metrics` is the stored
    /// `Main.SceneMetrics` (the last scan), `snow_dust` the live type-76
    /// population the previous `UpdateDust` head counted (`Main.snowDust`),
    /// `ctx` the pool's spawn statics. The `gamePaused` early-out
    /// (`:13040`) folds into the caller's pause gate; `netMode != 2`
    /// (`:17234`) is the single-player fold.
    pub(crate) fn snowing(
        &mut self,
        metrics: &SceneMetrics,
        i: &SnowInputs,
        snow_dust: i32,
        ctx: &SpawnCtx,
        world: &World,
        pool: &mut DustPool,
    ) {
        // `:13034-13037`: the remix world has no ambient snowfall.
        if i.remix_world {
            return;
        }
        // `:13040-13043`: no snow tiles / below the surface — nothing.
        if metrics.snow_tile_count <= 0 || !metrics.surface_atmospherics {
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
        let mut attempt = 0.0_f32;
        while attempt < num3 {
            attempt += 1.0;
            // `:13057-13060`: the budget gate — `snowDust < num2*(q/2+0.5) +
            // num2*0.1` — BREAKS the loop (not a per-attempt skip).
            if !((snow_dust as f32) < num2 as f32 * (i.gfx_quality / 2.0 + 0.5)
                + num2 as f32 * 0.1)
            {
                break;
            }
            // `:13061-13064`: the intensity roll.
            if !(self.rand01() < num) {
                continue;
            }
            // `:13065-13066`: the spawn window — x across the view +750 each
            // side, y up to 50 px above the view top.
            let mut num4 = self.next_int(i.scaled_size.x as u32 + 1500) as i32 - 750;
            let mut num5 = i.scaled_position.y as i32 - self.next_int(50) as i32;
            // `:13067-13070`: a falling player pulls the window up.
            if i.player_vy > 0.0 {
                num5 -= i.player_vy as i32;
            }
            // `:13071-13078`: the edge pushes — the second `Next(5)` rolls
            // ONLY when the first missed (the else-if).
            if self.next_int(5) == 0 {
                num4 = self.next_int(500) as i32 - 500;
            } else if self.next_int(5) == 0 {
                num4 = self.next_int(500) as i32 + i.scaled_size.x as i32;
            }
            // `:13079-13082`: off-screen x spreads y over the view.
            if num4 < 0 || num4 as f32 > i.scaled_size.x {
                num5 += self.next_int((i.scaled_size.y as f64 * 0.8) as u32) as i32
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
            // the pool's verbatim spawn; its rolls ride this stream. A
            // refused slot still takes the recipe writes below (the C#
            // writes `Main.dust[6000]` blindly).
            let idx = pool.new_dust(
                &mut crate::app::SplitmixRand(&mut self.rng),
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
            d.velocity.y = 3.0 + self.next_int(30) as f32 * 0.1;
            d.velocity.y *= d.scale;
            if !i.raining {
                // `:13094-13095`: the dry drift — wind + Next(-10,10)*0.1
                // (exclusive upper: -10..9), then the 15× wind lead.
                d.velocity.x = i.wind_speed_current
                    + (self.next_int(20) as i32 - 10) as f32 * 0.1;
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
                    + self.rand01() * 0.2
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

/// `Main.DrawDust`'s generic path (`Main.cs:38968-39018`) over the pool's
/// type-76 slots: the light sampled at `(pos + 4) / 16` (`:38968`), the
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
    for i in 0..DUST_SENTINEL {
        if !pool.slot(i).active || pool.slot(i).dust_type != 76 {
            continue;
        }
        let d = *pool.slot(i);
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
        let mut w = World::new("snow".into(), 200, 120);
        for x in 0..200 {
            for y in 95..120 {
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
        // `Main.cs:13034-13043`: the three early returns.
        let w = snow_world();
        let mut s = Snow::new(7);
        let mut pool = DustPool::new();
        let mut i = inputs();
        s.snowing(&metrics(0, true), &i, 0, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "no snow tiles");
        s.snowing(&metrics(3000, false), &i, 0, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "below the surface");
        i.remix_world = true;
        s.snowing(&metrics(3000, true), &i, 0, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "the remix world");
    }

    #[test]
    fn the_intensity_power_tracks_the_cloud_alpha() {
        // `Main.cs:13044`: `pow(count/6000, 4 - 3*cloudAlpha)` — at count
        // 3000 the dry intensity is 0.5^4 = 0.0625, a full storm 0.5^1 = 0.5.
        // The attempt count (`:13046`) climbs 1 → 51 with the same alpha.
        let w = snow_world();
        let m = metrics(3000, true);
        let mut dry = Snow::new(11);
        let mut dry_pool = DustPool::new();
        let mut i = inputs();
        for _ in 0..600 {
            dry.snowing(&m, &i, 0, &ctx(), &w, &mut dry_pool);
        }
        let mut storm = Snow::new(11);
        let mut storm_pool = DustPool::new();
        i.cloud_alpha = 1.0;
        i.raining = true;
        for _ in 0..600 {
            storm.snowing(&m, &i, 0, &ctx(), &w, &mut storm_pool);
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
        // 1600-px num2 is `(int)(500*1600/1920) = 416` → 457.6. A snowDust
        // already at 458 spawns NOTHING (the break, not a skip).
        let w = snow_world();
        let m = metrics(6000, true); // intensity 1: every attempt passes
        let mut s = Snow::new(13);
        let mut pool = DustPool::new();
        let i = inputs();
        s.snowing(&m, &i, 458, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "the budget break");
        // One under the budget the loop runs (and the pool fills).
        for _ in 0..30 {
            s.snowing(&m, &i, 457, &ctx(), &w, &mut pool);
        }
        assert!(live_type(&pool, 76) > 0);
        // The gfxQuality scale: at 0.5 the budget is 416*0.75 + 41.6 = 353.6.
        let mut q = Snow::new(13);
        let mut q_pool = DustPool::new();
        let mut qi = inputs();
        qi.gfx_quality = 0.5;
        q.snowing(&m, &qi, 354, &ctx(), &w, &mut q_pool);
        assert_eq!(live_type(&q_pool, 76), 0, "the quality-scaled budget");
    }

    #[test]
    fn the_deerclops_fades_the_intensity_and_budget() {
        // `Main.cs:13047-13052`: num /= 20, num2 /= 3 — at count 6000 the
        // dry intensity 1 → 0.05 and the budget 457.6 → 152*1.1 = 167.2.
        // The per-flake scale also drops 0.5 (`:13104-13107`).
        let w = snow_world();
        let m = metrics(6000, true);
        let mut near = Snow::new(17);
        let mut near_pool = DustPool::new();
        let mut i = inputs();
        i.deerclops_near = true;
        for _ in 0..600 {
            near.snowing(&m, &i, 0, &ctx(), &w, &mut near_pool);
        }
        let mut far = Snow::new(17);
        let mut far_pool = DustPool::new();
        i.deerclops_near = false;
        for _ in 0..600 {
            far.snowing(&m, &i, 0, &ctx(), &w, &mut far_pool);
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
        let mut dry = Snow::new(19);
        let mut dry_pool = DustPool::new();
        for _ in 0..30 {
            dry.snowing(&m, &i, 0, &ctx(), &w, &mut dry_pool);
        }
        i.raining = true;
        let mut storm = Snow::new(19);
        let mut storm_pool = DustPool::new();
        for _ in 0..30 {
            storm.snowing(&m, &i, 0, &ctx(), &w, &mut storm_pool);
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
            let mut s = Snow::new(23);
            let mut pool = DustPool::new();
            for _ in 0..30 {
                s.snowing(&m, &i, 0, &ctx(), &w, &mut pool);
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
        let mut s = Snow::new(29);
        let mut pool = DustPool::new();
        for _ in 0..30 {
            s.snowing(&m, &i, 0, &ctx(), &w, &mut pool);
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
}
