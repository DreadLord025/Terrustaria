//! `Sandstorm.EmitDust`: the flying sand particles.
//!
//! The storm's most visible face: `Sandstorm.EmitDust`
//! (`GameContent/Events/Sandstorm.cs:140-240`, called from `Main.cs:17247`)
//! throws budgeted `Dust` type 268 specks across the screen, weighted by the
//! sand-family tile counts, and the per-tick `Dust.UpdateDust` AI
//! (`Dust.cs:1789-1818`) rides them on the wind with a sinusoidal fall.
//!
//! Port notes:
//! * the RNG is this module's own splitmix stream (the port's per-consumer
//!   convention — vanilla rolls `Main.rand`, one shared stream); since S1-E-1
//!   phase 2 the spawn goes through the shared `DustPool` (`App.dust`) and
//!   the pool's own rolls ride this same stream via `SplitmixRand`;
//! * `Main.gfxQuality` is pinned to its no-pressure value (1.0): the emit
//!   budget and the pool's off-screen cull/deaths bound the population;
//! * the budget subtracts the live type-268 slots (`Sandstorm.cs:166`'s
//!   `Dust.SandStormCount` consumer) until E-3's shared head feeds
//!   `pool.sand_storm_count`;
//! * the sheet cell: the pool's per-100 frame shift maps type 268 into the
//!   shared `Dust` sheet's second band (`Dust.cs:177-182`: x = 10*268 - 2000,
//!   y = 60 + 10*rand(3)), which is the sheet the atlas already loads.

use glam::Vec2;
use tr_core::World;
use tr_core::particles::{Color, DUST_SENTINEL, DustPool, SpawnCtx};
use tr_core::scene::SceneMetrics;
use tr_core::tiledata as td;
use tr_render::sprite::SpriteRenderer;

use crate::assets;
use crate::rain::light_at;
use tr_core::lighting::LightMap;

/// `Main.maxScreenW` — the width the spawn density normalises to.
const MAX_SCREEN_W: f32 = 1920.0;

pub struct Sand {
    rng: u64,
}

impl Sand {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: (seed ^ 0x2545_F491_4F6C_DD1D).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1,
        }
    }

    /// The client stream state, for the shared tick's `DustRng` adapter
    /// (S1-E-3 — `Dust.UpdateDust`'s `Main.rand` draws ride it).
    pub(crate) fn rng_mut(&mut self) -> &mut u64 {
        &mut self.rng
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

    /// `Main.rand.Next(max)` — `Next(0)` rolls 0 (XNA), so an empty range
    /// passes the caller's gate.
    fn next_int(&mut self, max: u32) -> u32 {
        (self.next_u64() % max.max(1) as u64) as u32
    }

    /// `Sandstorm.EmitDust` (`Sandstorm.cs:140-240`). `severity` is
    /// `Sandstorm.Severity`; `show_visuals` is `ShowSandstormVisuals()`; `ctx`
    /// carries the `Main` statics the pool's spawn reads (`Main.screenPosition`
    /// world px and the UI `screenWidth/Height` — the C# passes the same
    /// screen rect to `NewDust`'s cull, `Dust.cs:97-101`); `pool` is the App's
    /// `Main.dust`. `metrics` is the stored `Main.SceneMetrics` (vanilla reads
    /// the last scan).
    pub fn emit_dust(
        &mut self,
        metrics: &SceneMetrics,
        severity: f32,
        show_visuals: bool,
        wind_speed_current: f32,
        ctx: &SpawnCtx,
        world: &World,
        pool: &mut DustPool,
    ) {
        if !show_visuals {
            return;
        }
        // `:144-147`: the desert-sand gate.
        let desert_sand = metrics.desert_sand_tile_count;
        if desert_sand < 100 {
            return;
        }
        // `:148-153`: the wind carries the sand; dead calm returns.
        let num = wind_speed_current.signum();
        let num2 = wind_speed_current.abs();
        if num2 < 0.01 {
            return;
        }
        let num3 = num * (0.9 + 0.1 * num2);
        // `:154-158`: the denser the sand, the bigger the clusters.
        let num4 = 2000.0 / desert_sand as f32;
        let num5 = (3.0 / num4).clamp(0.77, 1.0);
        let num6 = num4 as i32;
        // `:159-166`: the budget — `gfxQuality` pinned 1.0, and the C# line
        // subtracts `Dust.SandStormCount` (the live 268 population, which the
        // pool's slots hold until E-3's shared head maintains the static).
        let screen = ctx.screen_size;
        let num7 = 1000.0 * (screen.x / MAX_SCREEN_W);
        let num8 = 20.0 * severity;
        let mut num9 =
            num7 * (0.5 + 0.5) + num7 * 0.1 - crate::minecart::live_type(pool, 268) as f32;
        if num9 <= 0.0 {
            return;
        }
        let num10 = screen.x + 1000.0;
        let num11 = screen.y;
        // `:185-190`: the four sand families' weighted colours (desert /
        // crimson / jungle / snow tile counts; `WeightedRandom` falls through
        // to the first entry on a zero total). XNA byte colours, as spawned.
        let weights: [(Color, f32); 4] = [
            (
                Color::new(200, 160, 20, 180),
                (metrics.tile_counts[53] + metrics.tile_counts[396] + metrics.tile_counts[397])
                    as f32,
            ),
            (
                Color::new(103, 98, 122, 180),
                (metrics.tile_counts[112] + metrics.tile_counts[400] + metrics.tile_counts[398])
                    as f32,
            ),
            (
                Color::new(135, 43, 34, 180),
                (metrics.tile_counts[234] + metrics.tile_counts[401] + metrics.tile_counts[399])
                    as f32,
            ),
            (
                Color::new(213, 196, 197, 180),
                (metrics.tile_counts[116] + metrics.tile_counts[403] + metrics.tile_counts[402])
                    as f32,
            ),
        ];
        let num12 = 0.2 + 0.15 * severity;
        let num13 = 0.5 + 0.2 * severity;
        // `:192`: the cluster shuffle odds.
        let max_value = (1.0 + 9.0 * ((num5 - 0.77) / 0.23000002)) as u32;
        // `Main.screenPosition` — the world px of the screen's top-left; the
        // spawn positions are pre-offset UI px, scaled into the world view.
        let screen_pos = ctx.screen_position;
        let px = tr_core::TILE_PX;

        let mut attempts = 0;
        while (attempts as f32) < num8 {
            attempts += 1;
            // `:196`: the per-attempt gate (`Next(0)` rolls 0 — XNA).
            if self.next_int((num6 / 4).max(0) as u32) != 0 {
                continue;
            }
            let mut pos = Vec2::new(self.rand01() * num10 - 500.0, self.rand01() * -50.0);
            // `:199-205`: the screen-edge cases push spawns behind the
            // windward edge (both rolls always evaluate, C# `&&` order).
            if self.next_int(3) == 0 && num == 1.0 {
                pos.x = self.next_int(500) as f32 - 500.0;
            } else if self.next_int(3) == 0 && num == -1.0 {
                pos.x = self.next_int(500) as f32 + screen.x;
            }
            // `:207-210`: off-screen x spreads the y over the view.
            if pos.x < 0.0 || pos.x > screen.x {
                pos.y += self.rand01() * num11 * 0.9;
            }
            // `:214`: `position += Main.screenPosition` — verbatim: the UI-px
            // spawn offsets meet the world-px view top-left directly (at
            // zoom 1 the two spaces coincide; the stand-in's `/ zoom` scaling
            // retires with the pool migration).
            let mut pos = screen_pos + pos;
            // `:211-215`: the wall gate — no sand behind walls (and none
            // outside the world, where `tile` reads nothing).
            let tx = (pos.x / px).floor() as i32;
            let ty = (pos.y / px).floor() as i32;
            let Some(t) = world.tile(tx, ty) else {
                continue;
            };
            if t.wall != 0 {
                continue;
            }
            // The cluster shuffler (`:216-256`): each iteration spawns one
            // dust, then either stops or nudges the position and repeats.
            loop {
                // `:218`: `Dust.NewDust(position, 10, 10, 268)` — the pool's
                // verbatim spawn; its rolls (and the band rolls) ride the sand
                // stream. A refused slot still burns budget and feeds the
                // shuffle its velocity — the recipe below sets the properties
                // on the returned index blindly (`Main.dust[6000]`,
                // `Dust.cs:79`).
                let idx = pool.new_dust(
                    &mut crate::app::SplitmixRand(&mut self.rng),
                    ctx,
                    world,
                    pos,
                    10,
                    10,
                    268,
                    0.0,
                    0.0,
                    0,
                    Color::default(),
                    1.0,
                );
                let slot = pool.slot_mut(idx);
                // `:222-225`: the y recipe — the rolled scale is consumed
                // (`dust.velocity.Y *= dust.scale`), then ×0.35.
                slot.velocity.y = 2.0 + self.rand01() * 0.2;
                slot.velocity.y *= slot.scale;
                slot.velocity.y *= 0.35;
                // `:226-228`: the wind recipe and the spawn fadeIn (NewDust's
                // is 0, so this is `num13 * 0.2`).
                slot.velocity.x = num3 * 5.0 + self.rand01() + num3 * num13 * 20.0;
                slot.fade_in += num13 * 0.2;
                // `:229-234`: the severity scaling, the weighted colour (the
                // implicit `Get()` rolls once), and `dust.scale = 0.9f`
                // overwriting the pool's rolled scale.
                slot.velocity *= 1.0 + num12 * 0.5;
                slot.color = self.pick_weighted(&weights);
                slot.velocity *= 1.0 + num12;
                slot.velocity *= num5;
                slot.scale = 0.9;
                num9 -= 1.0;
                if num9 <= 0.0 {
                    return;
                }
                // `:245-253`: the shuffle — `RandomVector2(-10, 10)` plus the
                // velocity's mirror; the tile re-read discards.
                if self.next_int(max_value) != 0 {
                    pos += Vec2::new(self.rand01() * 20.0 - 10.0, self.rand01() * 20.0 - 10.0)
                        + slot.velocity * -1.1;
                } else {
                    break;
                }
            }
        }
    }

    /// `WeightedRandom<Color>`'s draw (`WeightedRandom.cs:64-71`): one
    /// `NextDouble` roll over the summed weights, first entry on a zero total.
    fn pick_weighted(&mut self, weights: &[(Color, f32); 4]) -> Color {
        let total: f32 = weights.iter().map(|w| w.1).sum();
        let mut r = self.rand01() * total;
        for w in weights {
            r -= w.1;
            if r < 0.0 {
                return w.0;
            }
        }
        weights[0].0
    }

    /// `Dust.SandStormCount` — the budget subtracts the live population
    /// (`Sandstorm.cs:166`).
    pub fn count(&self, pool: &DustPool) -> usize {
        crate::minecart::live_type(pool, 268)
    }
}

/// `Dust.UpdateDust`'s type-268 path over the pool's 268 slots
/// (`Dust.cs:451-454,476,1789-1818`, death `:2342-2346,2393-2394`).
/// `show_visuals` is the same `ShowSandstormVisuals` the spawner read
/// (`Dust.cs:430`); `player_vy` is `Main.player[myPlayer].velocity.Y`.
/// Death deactivates the slot — the pool keeps its 6000 fixed slots.
/// The type-268 chain arms (`Dust.cs:1789-2109`-family) — the shared
/// `UpdateDust` loop's per-type hook (S1-E-3). It returns false: these arms
/// are standalone statements of the C#'s first ladder (they sit between the
/// position move at `:477` and the fresh `if (dust.type >= 130 ...)` chain
/// head at `:2127`), so they do NOT exclude the chain's final
/// `else if (type != 304 && noGravity)` tail (`:2335`) — a no-gravity 268
/// takes both, exactly like the C#.
pub(crate) fn type_268_arm(
    d: &mut tr_core::particles::Dust,
    show_visuals: bool,
    player_vy: f32,
    world: &World,
) -> bool {
    let px = tr_core::TILE_PX;
    // `:1789-1796`: the speed-up and the scale drift; without the visuals
    // the storm's dust shrinks away.
    d.velocity *= 1.0050251;
    d.scale += 0.01;
    if !show_visuals {
        d.scale -= 0.05;
    }
    // `:1800-1809`: the player's fall drags the sand along.
    if player_vy > 0.0 {
        if d.fade_in == 0.0 && d.velocity.y < player_vy {
            d.velocity.y += (player_vy - d.velocity.y) * 0.04;
        }
        d.position.y += player_vy * 0.2;
    }
    // `:1811-1818`: over solid ground the dust dampens; in the open it
    // oscillates on the sine and falls. The spawn fadeIn keeps the first tick
    // damp-free (vanilla reads it before the ramp zeroes).
    if d.fade_in == 0.0 && solid_at(world, d.position, px) {
        d.scale *= 0.9;
        d.velocity *= 0.25;
    } else {
        d.velocity.y = (d.position.x * 0.0043982295).sin() * 2.0 + 3.0;
    }
    // `:2082-2109`: the generic fadeIn ramp — one +0.03 tick while the spawn
    // fadeIn stands, then it zeroes and the generic −0.01 tail takes over.
    if d.fade_in > 0.0 && d.fade_in < 100.0 {
        d.scale += 0.03;
        if d.scale > d.fade_in {
            d.fade_in = 0.0;
        }
    } else {
        d.scale -= 0.01;
    }
    false
}

/// `Main.DrawDust`'s generic path (`Main.cs:39005-39013`) over the pool's
/// type-268 slots: the light-tinted base pass, then the `GetColor` overlay
/// with the weighted spawn colour (`Dust.cs:2716-2728`), skipped when the sum
/// stays dark. Full-dark dusts die at the draw (`Main.cs:39015-39017`).
pub fn draw(
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
        if !pool.slot(i).active || pool.slot(i).dust_type != 268 {
            continue;
        }
        // `Main.cs:39015-39017`: `newColor == Color.Black` deactivates.
        let pos = pool.slot(i).position;
        let c = light_at(light, pos);
        if !c.iter().any(|&v| v > 0.0) {
            pool.slot_mut(i).active = false;
            continue;
        }
        let d = pool.slot(i);
        // The 268 sheet cell (`Dust.cs:177-182`): the pool's per-100 shift
        // gives x = 680, y = 60 + 10*row; the stand-in's 10 px cell and
        // scale-scaled size hold until the E-3 draw reshape.
        let Some(uv) = lookups.dust_uv(680, d.frame_y, 10, 10) else {
            continue;
        };
        let size = 10.0 * d.scale * zoom;
        // The base pass: `GetAlpha(light)` with `alpha == 0` is the light
        // itself (`Dust.cs:2724-2737`).
        r.push_fx_sprite_rot(
            to_screen(d.position),
            [size, size],
            uv,
            [c[0], c[1], c[2], 1.0],
            0.0,
        );
        // The overlay pass: `GetColor(light)` = colour + light − 255 per
        // channel, floored at 0; skipped when every channel is 0. The XNA
        // bytes normalise to the /255 floats.
        let oc = [
            channel_overlay(d.color.r as f32 / 255.0, c[0]),
            channel_overlay(d.color.g as f32 / 255.0, c[1]),
            channel_overlay(d.color.b as f32 / 255.0, c[2]),
        ];
        // The alpha: `color.A + light.A - 255`, the light's A is 255.
        let oa = d.color.a as f32 / 255.0;
        if oc.iter().any(|&v| v > 0.0) || oa > 0.0 {
            r.push_fx_sprite_rot(
                to_screen(d.position),
                [size, size],
                uv,
                [oc[0], oc[1], oc[2], oa],
                0.0,
            );
        }
    }
}

/// One `GetColor` channel: `colour + light - 255`, floored at 0.
fn channel_overlay(colour: f32, light: f32) -> f32 {
    (colour + light - 1.0).max(0.0)
}

/// `Collision.SolidCollision(pos - one*5, 10, 10)` (`Dust.cs:1811`): any
/// active solid tile in the 10x10 box centred on the dust.
fn solid_at(world: &World, pos: Vec2, px: f32) -> bool {
    let x0 = ((pos.x - 5.0) / px).floor() as i32;
    let x1 = ((pos.x + 4.0) / px).floor() as i32;
    let y0 = ((pos.y - 5.0) / px).floor() as i32;
    let y1 = ((pos.y + 4.0) / px).floor() as i32;
    for tx in x0..=x1 {
        for ty in y0..=y1 {
            if world.tile(tx, ty).is_some_and(|t| {
                t.active() && (t.kind as usize) < td::TILE_COUNT && td::TILE_SOLID.has(t.kind)
            }) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A deterministic stream for the shared tick's ChildSafety arm
    /// (S1-E-3): a fixed seed, no production stream touched.
    fn tr7() -> tr_core::unified_random::UnifiedRandom {
        tr_core::unified_random::UnifiedRandom::new(7)
    }
    use crate::minecart::live_type;
    use tr_core::particles::DustPool;
    use tr_core::tile::FLAG_ACTIVE;

    /// A flat sand world: rows 95-99 sand (53) on dirt, no walls — the emit
    /// gate reads the metrics, not the tiles, so the shapes stay simple.
    fn sand_world() -> World {
        let mut w = World::new("sand".into(), 200, 120);
        for x in 0..200 {
            for y in 95..99 {
                let i = w.index(x, y);
                w.tiles[i].kind = 53;
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
            for y in 99..120 {
                let i = w.index(x, y);
                w.tiles[i].kind = 1;
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    fn metrics_with(counts: &[(u16, u32)], desert: i32) -> SceneMetrics {
        let mut m = SceneMetrics::new();
        for &(id, n) in counts {
            m.tile_counts[id as usize] = n;
        }
        m.desert_sand_tile_count = desert;
        m
    }

    const SCREEN: Vec2 = Vec2::new(1600.0, 900.0);
    const CAMERA: Vec2 = Vec2::new(100.0 * 16.0, 95.0 * 16.0);

    /// The spawn ctx the tests run under: zoom 1, the view's top-left at
    /// `CAMERA − SCREEN/2`, the quality cap wide open.
    fn sand_ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: CAMERA - SCREEN * 0.5,
            screen_size: SCREEN,
            max_dust_to_draw: 6000,
        }
    }

    fn emit(
        sand: &mut Sand,
        pool: &mut DustPool,
        metrics: &SceneMetrics,
        world: &World,
        severity: f32,
    ) {
        sand.emit_dust(metrics, severity, true, 1.0, &sand_ctx(), world, pool);
    }

    fn live_268(pool: &DustPool) -> usize {
        live_type(pool, 268)
    }

    #[test]
    fn the_spawn_gate_needs_visuals_sand_and_wind() {
        // `Sandstorm.cs:146-153`: the three early returns.
        let w = sand_world();
        let m = metrics_with(&[], 500);
        let mut s = Sand::new(7);
        let mut pool = DustPool::new();
        s.emit_dust(&m, 1.0, false, 1.0, &sand_ctx(), &w, &mut pool);
        assert_eq!(live_268(&pool), 0, "visuals off — no dust");
        s.emit_dust(&m, 1.0, true, 0.005, &sand_ctx(), &w, &mut pool);
        assert_eq!(live_268(&pool), 0, "dead calm — no dust");
        let thin = metrics_with(&[], 99);
        s.emit_dust(&thin, 1.0, true, 1.0, &sand_ctx(), &w, &mut pool);
        assert_eq!(live_268(&pool), 0, "under 100 sand tiles — no dust");
    }

    #[test]
    fn the_budget_caps_the_population() {
        // `Sandstorm.cs:159-166`: `num9` bounds the population; the same seed
        // never exceeds it.
        let w = sand_world();
        let m = metrics_with(&[(53, 4000)], 5000);
        let mut s = Sand::new(7);
        let mut pool = DustPool::new();
        emit(&mut s, &mut pool, &m, &w, 1.0);
        let budget = 1000.0 * (SCREEN.x / 1920.0) * 1.1;
        assert!(live_268(&pool) > 0, "the storm throws sand");
        assert!(
            (live_268(&pool) as f32) <= budget,
            "population {} exceeds the budget {}",
            live_268(&pool),
            budget
        );
    }

    /// DW-2.2 — the emit's budget subtracts the live type-268 pool
    /// population (`Sandstorm.cs:166`'s `Dust.SandStormCount` consumer, held
    /// by the pool's slots until E-3's head maintains the static): a pool
    /// already at budget spawns nothing more.
    #[test]
    fn s1_e_1_the_sand_budget_subtracts_the_live_pool_population() {
        let w = sand_world();
        let m = metrics_with(&[(53, 4000)], 5000);
        let mut s = Sand::new(7);
        let mut pool = DustPool::new();
        // Fill every slot the budget could spend: `num9` goes ≤ 0.
        let budget = (1000.0 * (SCREEN.x / 1920.0) * 1.1).ceil() as usize;
        for i in 0..budget {
            let d = pool.slot_mut(i);
            d.active = true;
            d.dust_type = 268;
        }
        emit(&mut s, &mut pool, &m, &w, 1.0);
        assert_eq!(live_268(&pool), budget, "the saturated budget spawns none");
    }

    /// DW-2.2 — the sandstorm spawns through the pool: the emit's dusts are
    /// active type-268 slots carrying the C# EmitDust recipe's final shape
    /// (`Sandstorm.cs:218-234`): the overwritten 0.9 scale, the weighted XNA
    /// byte colour and the per-100 frame band.
    #[test]
    fn s1_e_1_the_sand_emit_spawns_type_268_through_the_pool() {
        let w = sand_world();
        let m = metrics_with(&[(53, 2000)], 500);
        let mut s = Sand::new(7);
        let mut pool = DustPool::new();
        emit(&mut s, &mut pool, &m, &w, 1.0);
        let n = live_268(&pool);
        assert!(n > 0, "the storm threw sand");
        for d in &pool.slots()[..DUST_SENTINEL] {
            if !d.active {
                continue;
            }
            assert_eq!(d.dust_type, 268);
            assert_eq!(d.scale, 0.9, "the recipe pins the scale");
            assert_eq!(d.frame_x, 680, "10*268 − 2×1000");
            assert!((60..=80).contains(&d.frame_y), "60 + 10*row");
            assert_eq!(d.color.a, 180, "the weighted colour's alpha");
            assert!(
                d.fade_in > 0.0,
                "the spawn fadeIn = num13 * 0.2 stands for one tick"
            );
        }
    }

    #[test]
    fn the_weighted_colour_follows_the_tile_counts() {
        // `Sandstorm.cs:185-190`: with only the desert family counted, every
        // spawn takes its (200,160,20) entry.
        let w = sand_world();
        let m = metrics_with(&[(53, 2000)], 500);
        let mut s = Sand::new(7);
        let mut pool = DustPool::new();
        emit(&mut s, &mut pool, &m, &w, 1.0);
        assert!(live_268(&pool) > 0);
        let want = Color::new(200, 160, 20, 180);
        for d in &pool.slots()[..DUST_SENTINEL] {
            if d.active && d.dust_type == 268 {
                assert_eq!(d.color, want, "all desert yellow");
            }
        }
    }

    /// One type-268 slot in the open air (well above the surface, off any
    /// solid) — the AI tests' fixture (the old `SandP` push, now the pool's).
    fn sand_slot(
        pool: &mut DustPool,
        slot: usize,
        pos: Vec2,
        set: impl FnOnce(&mut tr_core::particles::Dust),
    ) {
        let d = pool.slot_mut(slot);
        d.active = true;
        d.dust_type = 268;
        d.position = pos;
        d.velocity = Vec2::new(5.0, 3.0);
        d.scale = 0.9;
        d.fade_in = 0.1;
        d.frame_x = 680;
        d.frame_y = 60;
        d.color = Color::new(200, 160, 20, 180);
        set(d);
    }

    #[test]
    /// DW-2.2/2.3 — the sandstorm tick is the shared `Dust.UpdateDust` loop
    /// with the 268 chain arms as the hook: the shared head moves the slot
    /// (`Dust.cs:477`), the 268 arm runs (the speed-up, the scale drift, the
    /// sine fall), the matched arm excludes the generic gravity tail, and
    /// `dCount` lands at 0.
    #[test]
    fn s1_e_3_the_sand_storm_tick_rides_the_shared_update_dust_loop() {
        let w = sand_world();
        let mut pool = DustPool::new();
        sand_slot(&mut pool, 0, Vec2::new(CAMERA.x, CAMERA.y - 200.0), |d| {
            d.velocity = Vec2::new(2.0, 0.0);
            d.fade_in = 0.0;
            d.scale = 0.0;
        });
        let pos0 = pool.slot(0).position;
        let vel0 = pool.slot(0).velocity;
        crate::minecart::tick(
            &mut pool,
            &mut tr7(),
            6000,
            CAMERA,
            SCREEN,
            1.0,
            &w,
            0.0,
            true,
        );
        let d = pool.slot(0);
        assert_eq!(d.position, pos0 + vel0, "the shared move precedes the arm");
        assert_eq!(d.velocity.x, 2.0 * 1.0050251, "the 268 speed-up, no tail damp");
        // The sine reads the post-move position (the shared head's `:477`
        // precedes the arm).
        assert_eq!(
            d.velocity.y,
            ((pos0.x + vel0.x) * 0.0043982295).sin() * 2.0 + 3.0,
            "the sine fall reads the moved position"
        );
        // fadeIn 0 at the tick head: the drift +0.01 and the ramp's generic
        // −0.01 cancel (the C# note at `Dust.cs:2082-2109`).
        assert!((d.scale - 0.0).abs() < 1e-6, "the storm drift and the fade-done −0.01 cancel");
        assert_eq!(pool.d_count, 0.0, "one active slot, then the dead recompute");
        // The bottom kill rides the shared loop (the shared `else` and tail).
        let mut pool = DustPool::new();
        sand_slot(&mut pool, 0, Vec2::new(CAMERA.x, CAMERA.y + SCREEN.y * 0.5 + 100.0), |d| {
            d.velocity = Vec2::new(1.0, 1.0);
            d.fade_in = 0.0;
        });
        crate::minecart::tick(
            &mut pool,
            &mut tr7(),
            6000,
            CAMERA,
            SCREEN,
            1.0,
            &w,
            0.0,
            true,
        );
        assert_eq!(live_268(&pool), 0, "below the view bottom it dies (shared loop)");
    }

    fn the_ai_holds_scale_with_visuals_and_fades_without() {
        // `Dust.cs:1789-1796` + `:2082-2109`: with the storm on, the +0.01
        // drift, the fadeIn's one-off +0.03 and the generic −0.01 cancel —
        // the scale holds at ~0.94. Without, the net −0.05/tick fades the
        // orphaned dust out inside ~20 ticks.
        let w = sand_world();
        let mut on = DustPool::new();
        sand_slot(&mut on, 0, Vec2::new(CAMERA.x, CAMERA.y - 200.0), |_| {});
        crate::minecart::tick(&mut on, &mut tr7(), 6000, CAMERA, SCREEN, 1.0, &w, 0.0, true);
        assert!(
            (on.slot(0).scale - 0.94).abs() < 1e-3,
            "tick 1 settles at 0.94"
        );
        crate::minecart::tick(&mut on, &mut tr7(), 6000, CAMERA, SCREEN, 1.0, &w, 0.0, true);
        assert!(
            (on.slot(0).scale - 0.94).abs() < 1e-3,
            "the storm's dust holds the stable scale, got {}",
            on.slot(0).scale
        );
        let mut off = DustPool::new();
        sand_slot(&mut off, 0, Vec2::new(CAMERA.x, CAMERA.y - 200.0), |_| {});
        let mut ticks = 0;
        while live_268(&off) > 0 && ticks < 40 {
            crate::minecart::tick(&mut off, &mut tr7(), 6000, CAMERA, SCREEN, 1.0, &w, 0.0, false);
            ticks += 1;
        }
        assert_eq!(live_268(&off), 0, "the orphaned dust fades out");
        assert!(ticks <= 25, "died in {} ticks", ticks);
    }

    #[test]
    fn the_open_sand_rides_the_sine() {
        // `Dust.cs:1816-1817`: the vertical velocity is the sine of the x
        // position once the dust is in the open.
        let mut w = World::new("open".into(), 200, 120);
        w.compute_column_surface();
        let m = metrics_with(&[(53, 2000)], 500);
        let mut s = Sand::new(7);
        let mut pool = DustPool::new();
        emit(&mut s, &mut pool, &m, &w, 1.0);
        crate::minecart::tick(&mut pool, &mut tr7(), 6000, CAMERA, SCREEN, 1.0, &w, 0.0, true);
        for d in &pool.slots()[..DUST_SENTINEL] {
            if !d.active || d.dust_type != 268 {
                continue;
            }
            let want = (d.position.x * 0.0043982295).sin() * 2.0 + 3.0;
            assert!(
                (d.velocity.y - want).abs() < 1e-4,
                "vel.y {} != sine {}",
                d.velocity.y,
                want
            );
        }
    }

    #[test]
    fn the_bottom_death_clears_the_view() {
        // `Dust.cs:2342-2344`: below the screen the dust deactivates.
        let mut pool = DustPool::new();
        sand_slot(
            &mut pool,
            0,
            Vec2::new(CAMERA.x, CAMERA.y + SCREEN.y * 0.5 + 100.0),
            |d| {
                d.velocity = Vec2::new(1.0, 1.0);
                d.fade_in = 0.0;
            },
        );
        crate::minecart::tick(
            &mut pool,
            &mut tr7(),
            6000,
            CAMERA,
            SCREEN,
            1.0,
            &World::new("t".into(), 8, 8),
            0.0,
            true,
        );
        assert_eq!(live_268(&pool), 0, "below the view it dies");
    }

    #[test]
    fn the_stream_is_seed_deterministic() {
        // The port's per-consumer streams: the same seed and inputs give the
        // same spawn sequence (the screenshot recipe's baseline).
        let w = sand_world();
        let m = metrics_with(&[(53, 2000), (112, 800), (234, 400), (116, 200)], 500);
        let mut a = Sand::new(42);
        let mut b = Sand::new(42);
        let mut pa = DustPool::new();
        let mut pb = DustPool::new();
        for _ in 0..30 {
            emit(&mut a, &mut pa, &m, &w, 0.8);
            emit(&mut b, &mut pb, &m, &w, 0.8);
            crate::minecart::tick(&mut pa, &mut tr7(), 6000, CAMERA, SCREEN, 1.0, &w, 0.0, true);
            crate::minecart::tick(&mut pb, &mut tr7(), 6000, CAMERA, SCREEN, 1.0, &w, 0.0, true);
        }
        for i in 0..DUST_SENTINEL {
            let (da, db) = (pa.slot(i), pb.slot(i));
            assert_eq!(da.active, db.active);
            assert_eq!(da.position, db.position);
            assert_eq!(da.velocity, db.velocity);
            assert_eq!(da.scale, db.scale);
            assert_eq!(da.color, db.color);
        }
    }

    /// DW-2.3 (the sand half) — a paused or menu ctx refuses every spawn:
    /// the pool's guard chain (`Dust.cs:77-96`) gates the emit; the recipe
    /// arms still roll onto the refused sentinel (`Main.dust[6000]`), which
    /// never becomes a live slot.
    #[test]
    fn s1_e_1_the_ctx_gates_stop_the_sand_emit() {
        let w = sand_world();
        let m = metrics_with(&[(53, 4000)], 5000);
        for paused in [true, false] {
            let mut s = Sand::new(7);
            let mut pool = DustPool::new();
            let ctx = SpawnCtx {
                game_menu: !paused,
                game_paused: paused,
                ..sand_ctx()
            };
            s.emit_dust(&m, 1.0, true, 1.0, &ctx, &w, &mut pool);
            assert_eq!(live_268(&pool), 0, "the ctx gate refused the emit");
            assert!(!pool.slot(DUST_SENTINEL).active, "the sentinel stays dead");
        }
    }

    /// DW-2.5 (the sand site) — the recorded enumeration, pinned by stream
    /// step count: one minimal emit (one attempt — `20 × 0.05` — one
    /// dust, the shuffle stopping on its first gate roll) consumes exactly 15
    /// steps: attempt gate 1 + position 2 + the edge-case pair 2 + the dust's
    /// 10 (the pool's 6 `NewDust` rolls + the y/x recipes 2 + the weighted
    /// colour 1 + the stopping shuffle gate 1). The pool's 6 replace the
    /// stand-in's 3 (jitter pair + frame) one-for-one; the +3 unconditional
    /// steps are the C# `NewDust` velocity/scale rolls the stand-in skipped,
    /// and the recipe consumes the rolled scale (`Sandstorm.cs:225`). The
    /// severity 0.025 keeps `num8 = 20*severity` strictly under one attempt
    /// (0.05 f32 rounds just past 1.0 and would run a second attempt).
    #[test]
    fn s1_e_1_the_sand_stream_gains_only_the_recorded_pool_rolls() {
        let w = sand_world();
        let m = metrics_with(&[(53, 2000)], 500);
        // `Sand::new`'s scramble makes `seed -> rng` non-obvious; the search
        // runs whole emits until one lands on the minimal 15-step path with
        // a single live dust.
        for seed in 1..200u64 {
            let mut s = Sand::new(seed);
            let start = s.rng;
            let mut pool = DustPool::new();
            s.emit_dust(&m, 0.025, true, 1.0, &sand_ctx(), &w, &mut pool);
            // Count the steps: replay the splitmix from `start` to the
            // advanced state.
            let mut twin = start;
            let mut steps = 0u32;
            while twin != s.rng && steps < 64 {
                // `Sand::next_u64`'s shape: the state advances by the golden
                // constant; the mix is the OUTPUT, not the next state.
                twin = twin.wrapping_add(0x9E37_79B9_7F4A_7C15);
                let mut z = twin;
                z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
                let _output = z ^ (z >> 31);
                steps += 1;
            }
            let n = live_268(&pool);
            if steps == 15 && n == 1 {
                let d = pool
                    .slots()
                    .iter()
                    .find(|d| d.active && d.dust_type == 268)
                    .unwrap();
                assert_eq!(d.scale, 0.9);
                return;
            }
            assert!(n <= 1, "the minimal severity throws at most one cluster");
        }
        panic!("no minimal-path seed found in 1..200");
    }
}
