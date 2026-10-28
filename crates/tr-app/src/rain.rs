//! Rain: `Rain.MakeRain` / `Rain.Update` / `Main.DrawRain`.
//!
//! Weather follows the original's shape: a target intensity (`Main.maxRaining`) that
//! `cloudAlpha` ramps toward at `0.0005` per 60 Hz tick, a countdown that ends the
//! event, and one drop per `MakeRain` spawn slot per tick with the original fall
//! velocity (`wind * 18, 14` px/tick). Drops are drawn rotated around their top-left
//! origin with the local light colour, and die on solid or wet tiles with a splash.
//! In a snow biome the event renders as snowfall instead: no rain drops, and
//! `Dust` type 76 flakes (vanilla's ambient snow dust) drift down all the time, with
//! the storm intensity increasing the count and the wind.
//!
//! Storms also thicken the cloud deck (`Background_112/113` layered behind the
//! mountains) and, above 0.65 intensity, throw random lightning flashes.
//! `TR_RAIN=<intensity>` pins the weather for screenshots; `TR_NO_WEATHER` disables
//! the scheduler.

use glam::Vec2;
use tr_core::World;
use tr_core::lighting::LightMap;
use tr_core::tiledata as td;
use tr_render::sprite::SpriteRenderer;

use crate::assets;

/// `Main.maxRain`.
const MAX_DROPS: usize = 750;
/// Active snowflakes; vanilla caps the snow dust counter near 500 per screen.
const MAX_SNOW: usize = 600;
const TICKS_PER_SEC: f32 = 60.0;
/// `cloudAlpha` change per tick while raining (vanilla `0.0005 * dayRate`).
const RAMP_PER_TICK: f32 = 0.0005;
const DROP_W: f32 = 2.0;
const DROP_H: f32 = 40.0;
const SPLASH_LIFE: f32 = 0.32;

#[derive(Clone, Copy, Debug)]
struct Drop {
    pos: Vec2,
    vel: Vec2,
    scale: f32,
    rot: f32,
    /// Rain frame type (0..2 for water style 0).
    kind: u8,
}

#[derive(Clone, Copy, Debug)]
struct Splash {
    pos: Vec2,
    vel: Vec2,
    life: f32,
    frame: u8,
}

/// One `DustID.Snowflake` (type 76) particle.
#[derive(Clone, Copy, Debug)]
struct Snowflake {
    pos: Vec2,
    vel: Vec2,
    scale: f32,
    /// Sway phase; each flake drifts horizontally on its own sine.
    phase: f32,
    frame: u8,
}

pub struct Rain {
    /// `Main.cloudAlpha`: current visible intensity, 0..~1.5.
    intensity: f32,
    /// `Main.maxRaining`: intensity the event ramps toward.
    target: f32,
    raining: bool,
    /// Ticks until the current event starts or stops.
    timer: f32,
    /// `Main.windSpeedCurrent`: -1..1.
    wind: f32,
    /// `Main.cloudBGAlpha`: storm cloud-bank opacity, 0..1.
    cloud_bg: f32,
    /// Remaining lightning flash brightness, 0..1.
    flash: f32,
    /// Ticks until the next lightning bolt.
    lightning_timer: f32,
    drops: Vec<Drop>,
    splashes: Vec<Splash>,
    snow: Vec<Snowflake>,
    /// The camera is in a snow biome: rain is replaced by snowfall.
    snow_mode: bool,
    rng: u64,
    spawn_acc: f32,
    forced: Option<f32>,
    accum: f32,
}

impl Rain {
    pub fn new(seed: u64) -> Self {
        let forced = std::env::var("TR_RAIN")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
            .map(|v| v.clamp(0.0, 1.75));
        let start_calm = std::env::var_os("TR_NO_WEATHER").is_some();
        Self::with_forced(seed, forced, start_calm)
    }

    fn with_forced(seed: u64, forced: Option<f32>, start_calm: bool) -> Self {
        let rng = (seed ^ 0x2545_F491_4F6C_DD1D).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        let wind = (((rng >> 40) & 0xFFFF) as f32 / 65535.0 - 0.5) * 0.7;
        let timer = if start_calm {
            f32::INFINITY
        } else {
            1800.0 + (rng % 7200) as f32
        };
        Self {
            intensity: 0.0,
            target: 0.0,
            raining: false,
            timer,
            wind,
            cloud_bg: 0.0,
            flash: 0.0,
            lightning_timer: 0.0,
            drops: Vec::with_capacity(MAX_DROPS),
            splashes: Vec::with_capacity(128),
            snow: Vec::with_capacity(MAX_SNOW),
            snow_mode: false,
            rng,
            spawn_acc: 0.0,
            forced,
            accum: 0.0,
        }
    }

    /// `Main.cloudAlpha` clamped to the range the sky tint uses.
    pub fn visual_alpha(&self) -> f32 {
        self.intensity.clamp(0.0, 1.0)
    }

    pub fn drop_count(&self) -> usize {
        self.drops.len()
    }

    pub fn snow_count(&self) -> usize {
        self.snow.len()
    }

    /// Storm cloud-bank opacity (`Main.cloudBGAlpha`).
    pub fn cloud_bg(&self) -> f32 {
        self.cloud_bg
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

    fn range(&mut self, lo: u32, hi: u32) -> u32 {
        lo + (self.next_u64() % (hi.saturating_sub(lo).max(1) as u64)) as u32
    }

    /// Fixed 60 Hz stepping, independent of frame rate.
    pub fn update(
        &mut self,
        world: &World,
        camera: Vec2,
        screen: Vec2,
        zoom: f32,
        dt: f32,
        snow_biome: bool,
    ) {
        self.snow_mode = snow_biome;
        let step = 1.0 / TICKS_PER_SEC;
        let mut acc = self.accum + dt.min(0.25);
        let mut steps = 0;
        while acc >= step && steps < 12 {
            self.tick(world, camera, screen, zoom);
            acc -= step;
            steps += 1;
        }
        self.accum = acc.min(step * 12.0);
    }

    fn tick(&mut self, world: &World, camera: Vec2, screen: Vec2, zoom: f32) {
        if let Some(f) = self.forced {
            self.intensity = f;
            self.target = f;
            self.raining = f > 0.01;
        } else {
            self.timer -= 1.0;
            if self.raining {
                if self.timer <= 0.0 {
                    self.raining = false;
                    self.target = 0.0;
                    self.timer = self.range(3600, 14400) as f32;
                }
            } else if self.timer <= 0.0 {
                self.raining = true;
                self.target = 0.45 + self.rand01() * 0.75;
                self.timer = self.range(3600, 18000) as f32;
                self.wind = (self.rand01() * 2.0 - 1.0) * (0.25 + self.rand01() * 0.35);
            }
            let delta = (self.target - self.intensity).clamp(-RAMP_PER_TICK, RAMP_PER_TICK);
            self.intensity += delta;
        }

        // `cloudBGAlpha` ramps toward the storm strength (vanilla 0.0005/tick; a bit
        // faster here so the cloud deck appears within the ramp, not a minute later).
        let target = if self.intensity > 0.05 {
            self.intensity.min(1.0)
        } else {
            0.0
        };
        let delta = (target - self.cloud_bg).clamp(-0.005, 0.005);
        self.cloud_bg = (self.cloud_bg + delta).clamp(0.0, 1.0);

        // Thunderstorm lightning above 0.65 intensity: a short white flash every
        // 5-15 s. `Main.SpawnStormLightning` plays a bolt and sound; this pass draws
        // the screen flash only.
        if self.intensity > 0.65 {
            self.lightning_timer -= 1.0;
            if self.lightning_timer <= 0.0 {
                self.flash = 1.0;
                self.lightning_timer = self.range(300, 900) as f32;
            }
        } else {
            self.lightning_timer = self.range(120, 360) as f32;
        }
        self.flash = (self.flash - 0.09).max(0.0);

        // Each tick advances the drops at the original 60 Hz velocity.
        self.update_drops(world, camera, screen, zoom);
        self.update_snow(world, camera, screen, zoom);
        if self.snow_mode {
            // Vanilla suppresses rain drops in a snow biome and lets the ambient
            // snow dust carry the weather instead.
            if rain_zone(world, camera) {
                self.spawn_snow(world, camera, screen, zoom);
            }
        } else if self.intensity > 0.01 && rain_zone(world, camera) {
            self.spawn(world, camera, screen, zoom);
        }
        for s in &mut self.splashes {
            s.pos += s.vel;
            s.vel.y += 0.12;
            s.life -= 1.0 / TICKS_PER_SEC;
        }
        self.splashes.retain(|s| s.life > 0.0);
    }

    fn update_drops(&mut self, world: &World, camera: Vec2, screen: Vec2, zoom: f32) {
        let bottom = camera.y + screen.y * 0.5 / zoom + 100.0 / zoom;
        let px = tr_core::TILE_PX;
        let mut i = 0;
        while i < self.drops.len() {
            let mut d = self.drops[i];
            d.pos += d.vel;
            let tx = (d.pos.x / px).floor() as i32;
            let ty = (d.pos.y / px).floor() as i32;
            let hit = world.tile(tx, ty).is_some_and(|t| {
                (t.active() && (t.kind as usize) < td::TILE_COUNT && td::TILE_SOLID.has(t.kind))
                    || t.liquid > 0
            });
            if hit || d.pos.y > bottom {
                if hit {
                    self.spawn_splash(d);
                }
                self.drops.swap_remove(i);
            } else {
                self.drops[i] = d;
                i += 1;
            }
        }
    }

    fn spawn(&mut self, world: &World, camera: Vec2, screen: Vec2, zoom: f32) {
        // `MakeRain`: 25 attempts per 1920 px of screen width, scaled by the
        // intensity, spawn above the top edge of the view (and sideways, with the
        // wind), only into open sky.
        let rate = screen.x / 1920.0 * 25.0 * (0.25 + self.intensity);
        self.spawn_acc += rate;
        let count = self.spawn_acc.floor();
        self.spawn_acc -= count;
        if count <= 0.0 {
            return;
        }
        let px = tr_core::TILE_PX;
        let half = screen * 0.5 / zoom;
        let wind = self.wind + 0.05 * (self.intensity * 9.0).sin();
        for _ in 0..count as usize {
            if self.drops.len() >= MAX_DROPS {
                break;
            }
            let x = camera.x + (self.rand01() * 2.0 - 1.0) * (half.x + 600.0) - wind * 600.0;
            let y = camera.y - half.y - 20.0 - self.rand01() * 80.0;
            let tx = (x / px).floor() as i32;
            let ty = (y / px).floor() as i32;
            let open = world
                .tile(tx, ty)
                .is_some_and(|t| !t.active() && t.wall == 0);
            if !open {
                continue;
            }
            let scale = 1.0 + (self.rand01() * 2.0 - 1.0) * 0.2;
            let vel = Vec2::new(wind * 18.0 * scale, 14.0 * scale);
            let kind = self.range(0, 3) as u8;
            self.drops.push(Drop {
                pos: Vec2::new(x, y),
                vel,
                scale,
                rot: vel.x.atan2(-vel.y),
                kind,
            });
        }
    }

    fn update_snow(&mut self, world: &World, camera: Vec2, screen: Vec2, zoom: f32) {
        let bottom = camera.y + screen.y * 0.5 / zoom + 40.0 / zoom;
        let px = tr_core::TILE_PX;
        let mut i = 0;
        while i < self.snow.len() {
            let mut s = self.snow[i];
            s.phase += 0.06;
            s.pos.x += s.vel.x + s.phase.sin() * 0.45;
            s.pos.y += s.vel.y;
            let tx = (s.pos.x / px).floor() as i32;
            let ty = (s.pos.y / px).floor() as i32;
            let hit = world.tile(tx, ty).is_some_and(|t| {
                (t.active() && (t.kind as usize) < td::TILE_COUNT && td::TILE_SOLID.has(t.kind))
                    || t.liquid > 0
            });
            if hit || s.pos.y > bottom {
                self.snow.swap_remove(i);
            } else {
                self.snow[i] = s;
                i += 1;
            }
        }
    }

    /// Ambient snowfall: vanilla spawns `500 * screenWidth/1920` dust flakes in a snow
    /// biome, multiplied by `1 + 2*cloudAlpha` during a storm.
    fn spawn_snow(&mut self, world: &World, camera: Vec2, screen: Vec2, zoom: f32) {
        let px = tr_core::TILE_PX;
        let half = screen * 0.5 / zoom;
        let rate = screen.x / 1920.0 * 2.4 * (0.6 + 1.8 * self.intensity);
        if self.snow.len() >= MAX_SNOW {
            return;
        }
        let count = rate.max(0.0).floor() as usize;
        for _ in 0..count {
            if self.snow.len() >= MAX_SNOW {
                break;
            }
            let x = camera.x + (self.rand01() * 2.0 - 1.0) * (half.x + 90.0);
            let y = camera.y - half.y - 10.0 - self.rand01() * 60.0;
            let tx = (x / px).floor() as i32;
            let ty = (y / px).floor() as i32;
            if !world
                .tile(tx, ty)
                .is_some_and(|t| !t.active() && t.wall == 0)
            {
                continue;
            }
            let scale = 0.55 + self.rand01() * 0.45;
            let vy = (2.2 + self.rand01() * 2.4) * scale;
            let vx = self.wind * (5.0 + 9.0 * self.intensity) + (self.rand01() * 2.0 - 1.0) * 0.5;
            let frame = self.range(0, 3) as u8;
            let phase = self.rand01() * std::f32::consts::TAU;
            self.snow.push(Snowflake {
                pos: Vec2::new(x, y),
                vel: Vec2::new(vx, vy),
                scale,
                phase,
                frame,
            });
        }
    }

    fn spawn_splash(&mut self, d: Drop) {
        let n = 1 + (self.next_u64() & 1) as usize;
        for _ in 0..n {
            if self.splashes.len() >= 256 {
                break;
            }
            let jitter = Vec2::new((self.rand01() * 2.0 - 1.0) * 1.2, 0.0);
            let frame = self.range(0, 3) as u8;
            self.splashes.push(Splash {
                pos: d.pos + Vec2::new(2.0, 2.0),
                vel: d.vel * 0.0975 + jitter - Vec2::new(0.0, 2.0),
                life: SPLASH_LIFE,
                frame,
            });
        }
    }

    /// Draw in the always-rebuilt UI range (screen space), so the cached world instance
    /// range (G6) is unaffected by the moving drops.
    pub fn draw(
        &self,
        r: &mut SpriteRenderer,
        lookups: &assets::Lookups,
        light: &LightMap,
        camera: Vec2,
        zoom: f32,
        screen: Vec2,
    ) {
        if self.drops.is_empty()
            && self.splashes.is_empty()
            && self.snow.is_empty()
            && self.flash <= 0.01
        {
            return;
        }
        if self.flash > 0.01 {
            r.push_ui_rect(
                [0.0, 0.0],
                [screen.x, screen.y],
                [1.0, 0.98, 0.9, self.flash * 0.38],
            );
        }
        let to_screen = |p: Vec2| {
            [
                (p.x - camera.x) * zoom + screen.x * 0.5,
                (p.y - camera.y) * zoom + screen.y * 0.5,
            ]
        };
        for d in &self.drops {
            let Some(uv) = lookups.rain_uv(d.kind) else {
                continue;
            };
            let c = light_at(light, d.pos);
            r.push_ui_sprite_rot(
                to_screen(d.pos),
                [DROP_W * d.scale * zoom, DROP_H * d.scale * zoom],
                uv,
                [c[0] * 0.85, c[1] * 0.85, c[2] * 0.85, 1.0],
                d.rot,
            );
        }
        for s in &self.snow {
            let Some(uv) = lookups.dust_uv(760, s.frame as i32 * 10, 8, 8) else {
                continue;
            };
            let c = light_at(light, s.pos);
            let shade = [c[0] * 0.92 + 0.12, c[1] * 0.95 + 0.12, c[2] + 0.14];
            let a = (s.scale * 0.75).clamp(0.35, 0.9);
            r.push_ui_sprite_rot(
                to_screen(s.pos),
                [8.0 * s.scale * zoom, 8.0 * s.scale * zoom],
                uv,
                [shade[0], shade[1], shade[2], a],
                0.0,
            );
        }
        for s in &self.splashes {
            let Some(uv) = lookups.dust_uv(330, s.frame as i32 * 10, 8, 8) else {
                continue;
            };
            let c = light_at(light, s.pos);
            let a = (s.life / SPLASH_LIFE).clamp(0.0, 1.0) * 0.8;
            r.push_ui_sprite_rot(
                to_screen(s.pos),
                [8.0 * 0.6 * zoom, 8.0 * 0.6 * zoom],
                uv,
                [c[0], c[1], c[2], a],
                0.0,
            );
        }
    }
}

/// Vanilla only rains while `SceneMetrics.ZoneRain` is set; approximate it by
/// requiring the camera to be near that column's surface.
fn rain_zone(world: &World, camera: Vec2) -> bool {
    let tx = (camera.x / tr_core::TILE_PX) as i32;
    let ty = (camera.y / tr_core::TILE_PX) as i32;
    ty < world.surface_of(tx) + 24
}

/// `Lighting.GetColor` for a world-space point, clamped to the computed light rect.
fn light_at(light: &LightMap, p: Vec2) -> [f32; 3] {
    if light.w <= 0 || light.h <= 0 {
        return [1.0; 3];
    }
    let tx = ((p.x / tr_core::TILE_PX) as i32).clamp(light.x0, light.x0 + light.w - 1);
    let ty = ((p.y / tr_core::TILE_PX) as i32).clamp(light.y0, light.y0 + light.h - 1);
    light.get(tx, ty)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    fn ground() -> World {
        let mut w = World::new("rain".into(), 64, 64);
        for x in 0..64 {
            for y in 30..64 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    fn run(rain: &mut Rain, w: &World, n: usize) {
        let camera = Vec2::new(32.0 * 16.0, 25.0 * 16.0);
        for _ in 0..n {
            rain.update(w, camera, Vec2::new(1600.0, 200.0), 1.0, 1.0 / 60.0, false);
        }
    }

    #[test]
    fn forced_rain_spawns_drops_that_die_on_the_ground() {
        let w = ground();
        let mut rain = Rain::with_forced(7, Some(1.0), false);
        run(&mut rain, &w, 120);
        assert!(rain.drop_count() > 100, "drops {}", rain.drop_count());
        // The ground is below the camera, so impacts leave splashes at a steady state.
        assert!(!rain.splashes.is_empty(), "impacts should leave splashes");
    }

    #[test]
    fn calm_weather_has_no_drops() {
        let w = ground();
        let mut rain = Rain::with_forced(9, None, true);
        run(&mut rain, &w, 240);
        assert_eq!(rain.drop_count(), 0);
        assert_eq!(rain.visual_alpha(), 0.0);
    }

    #[test]
    fn snow_replaces_rain_in_a_snow_biome() {
        let w = ground();
        let mut rain = Rain::with_forced(17, Some(1.0), false);
        let camera = Vec2::new(32.0 * 16.0, 25.0 * 16.0);
        for _ in 0..120 {
            rain.update(&w, camera, Vec2::new(1600.0, 200.0), 1.0, 1.0 / 60.0, true);
        }
        assert_eq!(rain.drop_count(), 0, "no rain drops while snowing");
        assert!(rain.snow_count() > 50, "snow {}", rain.snow_count());
    }

    #[test]
    fn snow_settles_back_into_rain_outside_the_snow_biome() {
        let w = ground();
        let mut rain = Rain::with_forced(19, Some(1.0), false);
        let camera = Vec2::new(32.0 * 16.0, 25.0 * 16.0);
        let screen = Vec2::new(1600.0, 200.0);
        for _ in 0..60 {
            rain.update(&w, camera, screen, 1.0, 1.0 / 60.0, true);
        }
        let snow = rain.snow_count();
        assert!(snow > 0);
        for _ in 0..600 {
            rain.update(&w, camera, screen, 1.0, 1.0 / 60.0, false);
        }
        assert_eq!(rain.snow_count(), 0, "flakes should leave the view");
        assert!(rain.drop_count() > 0);
    }

    #[test]
    fn storms_ramp_the_cloud_deck_and_lightning_decays() {
        let w = ground();
        let mut rain = Rain::with_forced(23, Some(1.0), false);
        let camera = Vec2::new(32.0 * 16.0, 25.0 * 16.0);
        let screen = Vec2::new(1600.0, 200.0);
        for _ in 0..600 {
            rain.update(&w, camera, screen, 1.0, 1.0 / 60.0, false);
        }
        assert!(rain.cloud_bg() > 0.5, "cloud deck {}", rain.cloud_bg());
        rain.flash = 1.0;
        rain.update(&w, camera, screen, 1.0, 1.0 / 60.0, false);
        assert!(rain.flash < 1.0 && rain.flash > 0.7);
        for _ in 0..30 {
            rain.update(&w, camera, screen, 1.0, 1.0 / 60.0, false);
        }
        assert_eq!(rain.flash, 0.0, "flash must decay");
    }

    #[test]
    fn intensity_ramps_toward_the_target() {
        let w = ground();
        let mut rain = Rain::with_forced(11, None, false);
        rain.raining = true;
        rain.target = 1.0;
        run(&mut rain, &w, 600);
        let a = rain.visual_alpha();
        assert!(a > 0.1 && a < 0.5, "ramp {a}");
    }

    #[test]
    fn drops_only_spawn_into_open_sky() {
        let mut w = ground();
        // A wall behind the air blocks rain spawns even with open sky above.
        for x in 0..64 {
            for y in 5..30 {
                let i = w.index(x, y);
                w.tiles[i].wall = 2;
            }
        }
        let mut rain = Rain::with_forced(5, Some(1.0), false);
        run(&mut rain, &w, 120);
        assert_eq!(rain.drop_count(), 0);
    }
}
