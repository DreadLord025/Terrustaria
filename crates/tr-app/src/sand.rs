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
//!   convention — vanilla rolls `Main.rand`, one shared stream);
//! * `Main.gfxQuality` / the `dCount` pool-pressure governor are pinned to
//!   their no-pressure values (1.0 / 0): the emit budget and the off-screen
//!   deaths bound the population here;
//! * `Dust.NewDust`'s overwritten initial velocity rolls are not consumed —
//!   `EmitDust` replaces both components outright;
//! * the sheet cell: `dust.frame` maps type 268 into the shared `Dust` sheet's
//!   second band (`Dust.cs:177-182`: x = 10*268 - 2000, y = 60 + 10*rand(3)),
//!   which is the sheet the atlas already loads.

use glam::Vec2;
use tr_core::scene::SceneMetrics;
use tr_core::tiledata as td;
use tr_core::World;
use tr_render::sprite::SpriteRenderer;

use crate::assets;
use crate::rain::light_at;
use tr_core::lighting::LightMap;

/// `Main.maxScreenW` — the width the spawn density normalises to.
const MAX_SCREEN_W: f32 = 1920.0;

#[derive(Clone, Copy, Debug, PartialEq)]
struct SandP {
    pos: Vec2,
    vel: Vec2,
    scale: f32,
    fade_in: f32,
    /// The sheet row picked at spawn (`10 * Main.rand.Next(3)`).
    frame: u8,
    /// The `EmitDust` weighted colour; the draw's `GetColor` overlay reads it.
    color: [f32; 4],
}

pub struct Sand {
    parts: Vec<SandP>,
    rng: u64,
}

impl Sand {
    pub fn new(seed: u64) -> Self {
        Self {
            parts: Vec::new(),
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

    /// `Main.rand.Next(max)` — `Next(0)` rolls 0 (XNA), so an empty range
    /// passes the caller's gate.
    fn next_int(&mut self, max: u32) -> u32 {
        (self.next_u64() % max.max(1) as u64) as u32
    }

    /// `Sandstorm.EmitDust` (`Sandstorm.cs:140-240`). `severity` is
    /// `Sandstorm.Severity`; `show_visuals` is `ShowSandstormVisuals()`; the
    /// screen inputs are the UI-space size (`Main.screenWidth/Height`) and the
    /// camera centre in world pixels; `metrics` is the stored
    /// `Main.SceneMetrics` (vanilla reads the last scan).
    pub fn emit_dust(
        &mut self,
        metrics: &SceneMetrics,
        severity: f32,
        show_visuals: bool,
        wind_speed_current: f32,
        camera: Vec2,
        screen: Vec2,
        zoom: f32,
        world: &World,
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
        // `:159-164`: the budget — `gfxQuality` pinned 1.0.
        let num7 = 1000.0 * (screen.x / MAX_SCREEN_W);
        let num8 = 20.0 * severity;
        let mut num9 = num7 * (0.5 + 0.5) + num7 * 0.1 - self.parts.len() as f32;
        if num9 <= 0.0 {
            return;
        }
        let num10 = screen.x + 1000.0;
        let num11 = screen.y;
        // `:185-190`: the four sand families' weighted colours (desert /
        // crimson / jungle / snow tile counts; `WeightedRandom` falls through
        // to the first entry on a zero total).
        let weights: [([f32; 4], f32); 4] = [
            (
                [200.0 / 255.0, 160.0 / 255.0, 20.0 / 255.0, 180.0 / 255.0],
                (metrics.tile_counts[53] + metrics.tile_counts[396] + metrics.tile_counts[397])
                    as f32,
            ),
            (
                [103.0 / 255.0, 98.0 / 255.0, 122.0 / 255.0, 180.0 / 255.0],
                (metrics.tile_counts[112] + metrics.tile_counts[400] + metrics.tile_counts[398])
                    as f32,
            ),
            (
                [135.0 / 255.0, 43.0 / 255.0, 34.0 / 255.0, 180.0 / 255.0],
                (metrics.tile_counts[234] + metrics.tile_counts[401] + metrics.tile_counts[399])
                    as f32,
            ),
            (
                [213.0 / 255.0, 196.0 / 255.0, 197.0 / 255.0, 180.0 / 255.0],
                (metrics.tile_counts[116] + metrics.tile_counts[403] + metrics.tile_counts[402])
                    as f32,
            ),
        ];
        let num12 = 0.2 + 0.15 * severity;
        let num13 = 0.5 + 0.2 * severity;
        // `:192`: the cluster shuffle odds.
        let max_value = (1.0 + 9.0 * ((num5 - 0.77) / 0.23000002)) as u32;
        // `Main.screenPosition` — the world px of the screen's top-left.
        let screen_pos = camera - screen * 0.5 / zoom;
        let view = screen / zoom;
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
            let mut pos = screen_pos + pos / zoom;
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
                // `Dust.NewDust`'s box jitter (`Dust.cs:160-162`: the 10x10
                // box, `Next(w-4) + 4`) and its screen reject (`:97-101`; the
                // dCount margin is 0 at the pinned dCount). A rejected slot
                // still burns budget and feeds the shuffle its velocity —
                // `EmitDust` sets the properties on the returned index blindly.
                let bx = pos.x + self.next_int(6) as f32 + 4.0;
                let by = pos.y + self.next_int(6) as f32 + 4.0;
                // `:222-238`: the velocity recipe — y falls, x rides the wind,
                // both scaled by the severity factors. The rolls run even when
                // the slot is rejected.
                let mut vel = Vec2::new(0.0, 0.0);
                vel.y = (2.0 + self.rand01() * 0.2) * 0.35;
                vel.x = num3 * 5.0 + self.rand01() + num3 * num13 * 20.0;
                let fade_in = num13 * 0.2;
                vel *= 1.0 + num12 * 0.5;
                let color = self.pick_weighted(&weights);
                vel *= 1.0 + num12;
                vel *= num5;
                let frame = self.next_int(3) as u8;
                let on_screen = bx + 10.0 >= screen_pos.x
                    && bx <= screen_pos.x + view.x
                    && by + 10.0 >= screen_pos.y
                    && by <= screen_pos.y + view.y;
                if on_screen {
                    self.parts.push(SandP {
                        pos: Vec2::new(bx, by),
                        vel,
                        scale: 0.9,
                        fade_in,
                        frame,
                        color,
                    });
                }
                num9 -= 1.0;
                if num9 <= 0.0 {
                    return;
                }
                // `:245-253`: the shuffle — `RandomVector2(-10, 10)` plus the
                // velocity's mirror; the tile re-read discards.
                if self.next_int(max_value) != 0 {
                    pos += Vec2::new(self.rand01() * 20.0 - 10.0, self.rand01() * 20.0 - 10.0)
                        + vel * -1.1;
                } else {
                    break;
                }
            }
        }
    }

    /// `WeightedRandom<Color>`'s draw (`WeightedRandom.cs:64-71`): one
    /// `NextDouble` roll over the summed weights, first entry on a zero total.
    fn pick_weighted(&mut self, weights: &[([f32; 4], f32); 4]) -> [f32; 4] {
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

    /// `Dust.UpdateDust`'s type-268 path (`Dust.cs:451-454,476,1789-1818`,
    /// death `:2342-2346,2393-2394`). `show_visuals` is the same
    /// `ShowSandstormVisuals` the spawner read (`Dust.cs:430`); `player_vy`
    /// is `Main.player[myPlayer].velocity.Y`.
    pub fn tick(
        &mut self,
        show_visuals: bool,
        player_vy: f32,
        camera: Vec2,
        screen: Vec2,
        zoom: f32,
        world: &World,
    ) {
        let bottom = camera.y + screen.y * 0.5 / zoom;
        let px = tr_core::TILE_PX;
        let mut i = 0;
        while i < self.parts.len() {
            let mut p = self.parts[i];
            // `Dust.cs:451-454`: runaway scale dies.
            if p.scale > 10.0 {
                self.parts.swap_remove(i);
                continue;
            }
            // `:476`: the move.
            p.pos += p.vel;
            // `:1789-1796`: the speed-up and the scale drift; without the
            // visuals the storm's dust shrinks away.
            p.vel *= 1.0050251;
            p.scale += 0.01;
            if !show_visuals {
                p.scale -= 0.05;
            }
            // `:1800-1809`: the player's fall drags the sand along.
            if player_vy > 0.0 {
                if p.fade_in == 0.0 && p.vel.y < player_vy {
                    p.vel.y += (player_vy - p.vel.y) * 0.04;
                }
                p.pos.y += player_vy * 0.2;
            }
            // `:1811-1818`: over solid ground the dust dampens; in the open it
            // oscillates on the sine and falls. The spawn fadeIn keeps the
            // first tick damp-free (vanilla reads it before the ramp zeroes).
            if p.fade_in == 0.0 && solid_at(world, p.pos, px) {
                p.scale *= 0.9;
                p.vel *= 0.25;
            } else {
                p.vel.y = (p.pos.x * 0.0043982295).sin() * 2.0 + 3.0;
            }
            // `Dust.cs:2082-2109`: the generic fadeIn ramp — one +0.03 tick
            // while the spawn fadeIn stands, then it zeroes and the generic
            // −0.01 tail takes over. With the storm on the two cancel and the
            // scale holds; without it the dust loses 0.05/tick.
            if p.fade_in > 0.0 && p.fade_in < 100.0 {
                p.scale += 0.03;
                if p.scale > p.fade_in {
                    p.fade_in = 0.0;
                }
            } else {
                p.scale -= 0.01;
            }
            // `:2342-2344`: below the view it dies; `:2393-2394`: the scale
            // floor (the dCount nibbles are 0 at the pinned dCount).
            if p.pos.y > bottom || p.scale < 0.1 {
                self.parts.swap_remove(i);
                continue;
            }
            self.parts[i] = p;
            i += 1;
        }
    }

    /// `Main.DrawDust`'s generic path (`Main.cs:39005-39013`): the light-tinted
    /// base pass, then the `GetColor` overlay with the weighted spawn colour
    /// (`Dust.cs:2716-2728`), skipped when the sum stays dark. Full-dark dusts
    /// die at the draw (`Main.cs:39015-39017`).
    pub fn draw(
        &mut self,
        r: &mut SpriteRenderer,
        lookups: &assets::Lookups,
        light: &LightMap,
        camera: Vec2,
        zoom: f32,
        screen: Vec2,
    ) {
        // `Main.cs:39015-39017`: `newColor == Color.Black` deactivates.
        self.parts
            .retain(|p| light_at(light, p.pos).iter().any(|&c| c > 0.0));
        let to_screen = |p: Vec2| {
            [
                (p.x - camera.x) * zoom + screen.x * 0.5,
                (p.y - camera.y) * zoom + screen.y * 0.5,
            ]
        };
        for p in &self.parts {
            let Some(uv) = lookups.dust_uv(680, 60 + p.frame as i32 * 10, 10, 10) else {
                continue;
            };
            let c = light_at(light, p.pos);
            let size = 10.0 * p.scale * zoom;
            // The base pass: `GetAlpha(light)` with `alpha == 0` is the light
            // itself (`Dust.cs:2724-2737`).
            r.push_fx_sprite_rot(to_screen(p.pos), [size, size], uv, [c[0], c[1], c[2], 1.0], 0.0);
            // The overlay pass: `GetColor(light)` = colour + light − 255 per
            // channel, floored at 0; skipped when every channel is 0.
            let oc = [
                channel_overlay(p.color[0], c[0]),
                channel_overlay(p.color[1], c[1]),
                channel_overlay(p.color[2], c[2]),
            ];
            // The alpha: `color.A + light.A - 255`, the light's A is 255.
            let oa = p.color[3];
            if oc.iter().any(|&c| c > 0.0) || oa > 0.0 {
                r.push_fx_sprite_rot(to_screen(p.pos), [size, size], uv, [oc[0], oc[1], oc[2], oa], 0.0);
            }
        }
    }

    /// `Dust.SandStormCount` — the budget subtracts the live population
    /// (`Sandstorm.cs:166`).
    pub fn count(&self) -> usize {
        self.parts.len()
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
    use tr_core::tile::{FLAG_ACTIVE, Tile};

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

    fn emit(sand: &mut Sand, metrics: &SceneMetrics, world: &World, severity: f32) {
        sand.emit_dust(metrics, severity, true, 1.0, CAMERA, SCREEN, 1.0, world);
    }

    #[test]
    fn the_spawn_gate_needs_visuals_sand_and_wind() {
        // `Sandstorm.cs:146-153`: the three early returns.
        let w = sand_world();
        let m = metrics_with(&[], 500);
        let mut s = Sand::new(7);
        s.emit_dust(&m, 1.0, false, 1.0, CAMERA, SCREEN, 1.0, &w);
        assert_eq!(s.count(), 0, "visuals off — no dust");
        s.emit_dust(&m, 1.0, true, 0.005, CAMERA, SCREEN, 1.0, &w);
        assert_eq!(s.count(), 0, "dead calm — no dust");
        let thin = metrics_with(&[], 99);
        s.emit_dust(&thin, 1.0, true, 1.0, CAMERA, SCREEN, 1.0, &w);
        assert_eq!(s.count(), 0, "under 100 sand tiles — no dust");
    }

    #[test]
    fn the_budget_caps_the_population() {
        // `Sandstorm.cs:159-166`: `num9` bounds the population; the same seed
        // never exceeds it.
        let w = sand_world();
        let m = metrics_with(&[(53, 4000)], 5000);
        let mut s = Sand::new(7);
        emit(&mut s, &m, &w, 1.0);
        let budget = 1000.0 * (SCREEN.x / 1920.0) * 1.1;
        assert!(s.count() > 0, "the storm throws sand");
        assert!(
            (s.count() as f32) <= budget,
            "population {} exceeds the budget {}",
            s.count(),
            budget
        );
    }

    #[test]
    fn the_weighted_colour_follows_the_tile_counts() {
        // `Sandstorm.cs:185-190`: with only the desert family counted, every
        // spawn takes its (200,160,20) entry.
        let w = sand_world();
        let m = metrics_with(&[(53, 2000)], 500);
        let mut s = Sand::new(7);
        emit(&mut s, &m, &w, 1.0);
        assert!(s.count() > 0);
        let want = [200.0 / 255.0, 160.0 / 255.0, 20.0 / 255.0, 180.0 / 255.0];
        assert!(s.parts.iter().all(|p| p.color == want), "all desert yellow");
    }

    /// One particle in the open air (well above the surface, off any solid).
    fn air_particle(pos: Vec2) -> SandP {
        SandP {
            pos,
            vel: Vec2::new(5.0, 3.0),
            scale: 0.9,
            fade_in: 0.1,
            frame: 0,
            color: [200.0 / 255.0, 160.0 / 255.0, 20.0 / 255.0, 180.0 / 255.0],
        }
    }

    #[test]
    fn the_ai_holds_scale_with_visuals_and_fades_without() {
        // `Dust.cs:1789-1796` + `:2082-2109`: with the storm on, the +0.01
        // drift, the fadeIn's one-off +0.03 and the generic −0.01 cancel —
        // the scale holds at ~0.94. Without, the net −0.05/tick fades the
        // orphaned dust out inside ~20 ticks.
        let w = sand_world();
        let mut on = Sand::new(7);
        on.parts.push(air_particle(Vec2::new(CAMERA.x, CAMERA.y - 200.0)));
        on.tick(true, 0.0, CAMERA, SCREEN, 1.0, &w);
        assert!((on.parts[0].scale - 0.94).abs() < 1e-3, "tick 1 settles at 0.94");
        on.tick(true, 0.0, CAMERA, SCREEN, 1.0, &w);
        assert!(
            (on.parts[0].scale - 0.94).abs() < 1e-3,
            "the storm's dust holds the stable scale, got {}",
            on.parts[0].scale
        );
        let mut off = Sand::new(7);
        off.parts.push(air_particle(Vec2::new(CAMERA.x, CAMERA.y - 200.0)));
        let mut ticks = 0;
        while off.count() > 0 && ticks < 40 {
            off.tick(false, 0.0, CAMERA, SCREEN, 1.0, &w);
            ticks += 1;
        }
        assert_eq!(off.count(), 0, "the orphaned dust fades out");
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
        emit(&mut s, &m, &w, 1.0);
        s.tick(true, 0.0, CAMERA, SCREEN, 1.0, &w);
        for p in &s.parts {
            let want = (p.pos.x * 0.0043982295).sin() * 2.0 + 3.0;
            assert!(
                (p.vel.y - want).abs() < 1e-4,
                "vel.y {} != sine {}",
                p.vel.y,
                want
            );
        }
    }

    #[test]
    fn the_bottom_death_clears_the_view() {
        // `Dust.cs:2342-2344`: below the screen the dust deactivates.
        let mut s = Sand::new(7);
        s.parts.push(SandP {
            pos: Vec2::new(CAMERA.x, CAMERA.y + SCREEN.y * 0.5 + 100.0),
            vel: Vec2::new(1.0, 1.0),
            scale: 0.9,
            fade_in: 0.0,
            frame: 0,
            color: [1.0, 1.0, 1.0, 1.0],
        });
        s.tick(true, 0.0, CAMERA, SCREEN, 1.0, &World::new("t".into(), 8, 8));
        assert_eq!(s.count(), 0, "below the view it dies");
    }

    #[test]
    fn the_stream_is_seed_deterministic() {
        // The port's per-consumer streams: the same seed and inputs give the
        // same spawn sequence (the screenshot recipe's baseline).
        let w = sand_world();
        let m = metrics_with(&[(53, 2000), (112, 800), (234, 400), (116, 200)], 500);
        let mut a = Sand::new(42);
        let mut b = Sand::new(42);
        for _ in 0..30 {
            emit(&mut a, &m, &w, 0.8);
            emit(&mut b, &m, &w, 0.8);
            a.tick(true, 0.0, CAMERA, SCREEN, 1.0, &w);
            b.tick(true, 0.0, CAMERA, SCREEN, 1.0, &w);
        }
        assert_eq!(a.parts, b.parts, "byte-equal populations");
    }
}
