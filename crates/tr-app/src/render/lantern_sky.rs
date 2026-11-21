//! W4-f40 — the lantern night's background lantern field (`LanternSky :
//! CustomSky`, `GameContent/Skies/LanternSky.cs`). The `SkyManager`'s
//! "Lantern" sky (`ScreenEffectInitializer.cs:74`) activates on the
//! lanterns-up edge (`LanternNight.UpdateTime`, `LanternNight.cs:137-144`)
//! and deactivates on the down-edge; the port drives the same edges off
//! `world_events::update_time_lanterns`' `Option<bool>` return.
//!
//! C# sources read WHOLE (the golden rule):
//! - the `Lantern` struct (`:13-59`): variant/time-until-float/position/
//!   depth/rotation/speed/active + `FloatAdjustedSpeed` (`:53`) and the
//!   3-variant source rect (`:55-58`, `FrameWidth = texture.Width / 3`).
//! - `GenerateLanterns` (`:83-101`): the `maxTilesY / 4` array, the
//!   `screenPosition.Y * 0.7 - screenHeight` band and the
//!   `Next(0, maxTilesX) * 16` / `Next(minValue, num)` position draws.
//! - `ResetLantern` (`:103-111`): the depth ladder `(1 - i/len) * 4.4 +
//!   1.6`, the `-1.5 - 2.5 * NextDouble()` speed, the `Next(3)` variant and
//!   the `(2000 + Next(1200)) * 2` float timer.
//! - `Update` (`:113-152`): the `±0.01` opacity walk toward `LanternsUp`,
//!   the per-lantern sine bob + float + wind drift + rotation, the
//!   `TimeUntilFloat` decay and the `Position.Y < 300` reset/leave.
//! - `Draw` (`:154-240`) + `DrawLantern` (`:242-251`): the depth-band cull,
//!   the `(250, 120, 60, 120)` tint with the depth ladder, the
//!   `(1/Depth, 0.9/Depth) * 1.2` parallax about the view centre, the
//!   4000-px X wrap, the 1000-px cull rect and the 3-copy glow + body.
//!
//! RNG contract: the sky owns a PRIVATE `UnifiedRandom` (`_random`,
//! `LanternSky.cs:71`, `new UnifiedRandom()` = the time-seeded ctor). It is
//! NOT `Main.rand` and draws on no named port stream — the port mirrors it
//! as a dedicated `UnifiedRandom` seeded once (the `Ambience` scheduler's
//! own-stream precedent, but fully private here). The draw ORDER is the
//! C#'s: per lantern X then Y (`GenerateLanterns`), then speed/variant/
//! float-timer (`ResetLantern`); a reset draws speed/variant/timer then the
//! new X (`Update`'s `:141-142`). The seed is fixed (0) for determinism —
//! the C#'s time seed is unobservable in a deterministic port (recorded).
//!
//! Deliberate folds (ANALYSIS "The lantern-night sky (W4-f40)"):
//! - `FocusHelper.PauseSkies` (`:115`) — the port's update rides the same
//!   step-gated ambience phase, so the pause is the phase gate itself.
//! - the `gameMenu` draw-clear (`:156-164`) — the port's sky pass only runs
//!   in-world; the menu never draws it (the front-flow owns the menu).
//! - `SpriteEffects.None` / the centred origin — `push_screen_rot` rotates
//!   about the centre, the C#'s `origin = sourceRect.Size() / 2`.

use glam::Vec2;
use tr_core::UnifiedRandom;
use tr_render::SpriteRenderer;

use crate::assets::Lookups;
use crate::render::bg::push_screen_rot;

/// One background lantern (`LanternSky.Lantern`, `LanternSky.cs:13-59`).
#[derive(Clone, Copy, Debug)]
struct Lantern {
    /// `Variant` — the 3-across source column (`:17`).
    variant: i32,
    /// `TimeUntilFloat` / `TimeUntilFloatMax` (`:19-21`).
    time_until_float: i32,
    time_until_float_max: i32,
    /// `Position` in world pixels (`:25`).
    position: Vec2,
    /// `Depth` — the parallax stratum (`:27`).
    depth: f32,
    /// `Rotation` (`:29`).
    rotation: f32,
    /// `Speed` (`:35`).
    speed: f32,
    /// `Active` (`:37`).
    active: bool,
}

impl Lantern {
    /// `FloatAdjustedSpeed` (`LanternSky.cs:53`).
    fn float_adjusted_speed(&self) -> f32 {
        self.speed * (self.time_until_float as f32 / self.time_until_float_max as f32)
    }
}

/// `LanternSky : CustomSky` (`LanternSky.cs:11-282`).
pub(crate) struct LanternSky {
    /// `_active` (`:61`) — set true by `Update`, false by the menu clear.
    active: bool,
    /// `_leaving` (`:63`) — the deactivate latch: lanterns that rise past
    /// 300 fade out instead of resetting.
    leaving: bool,
    /// `_opacity` (`:65`) — the 0..1 fade toward `LanternsUp`.
    opacity: f32,
    /// `_lanterns` (`:69`) — the `maxTilesY / 4` array.
    lanterns: Vec<Lantern>,
    /// `_random` (`:71`) — the sky's PRIVATE stream (module docs).
    random: UnifiedRandom,
    /// `_lanternsDrawing` (`:73`) — the live count (the leaving decrement).
    lanterns_drawing: i32,
}

impl Default for LanternSky {
    fn default() -> Self {
        Self::new()
    }
}

impl LanternSky {
    pub(crate) fn new() -> Self {
        Self {
            active: false,
            leaving: false,
            opacity: 0.0,
            lanterns: Vec::new(),
            // The C#'s `new UnifiedRandom()` is time-seeded; the port pins
            // 0 for determinism (the private stream is unobservable — the
            // module docs record it).
            random: UnifiedRandom::new(0),
            lanterns_drawing: 0,
        }
    }

    /// `Activate` (`LanternSky.cs:253-266`): a re-activate while active
    /// fills only the missing lanterns; a fresh activate regenerates all.
    pub(crate) fn activate(&mut self, max_tiles_x: i32, max_tiles_y: i32, screen_y: f32, screen_h: f32, world_surface: f64) {
        if self.active {
            self.leaving = false;
            self.generate_lanterns(true, max_tiles_x, max_tiles_y, screen_y, screen_h, world_surface);
        } else {
            self.generate_lanterns(false, max_tiles_x, max_tiles_y, screen_y, screen_h, world_surface);
            self.active = true;
            self.leaving = false;
        }
    }

    /// `Deactivate` (`LanternSky.cs:268-271`): the leaving latch.
    pub(crate) fn deactivate(&mut self) {
        self.leaving = true;
    }

    /// `Reset` (`LanternSky.cs:278-281`): the world-load clear.
    pub(crate) fn reset(&mut self) {
        self.active = false;
    }

    /// `GenerateLanterns` (`LanternSky.cs:83-101`).
    fn generate_lanterns(&mut self, only_missing: bool, max_tiles_x: i32, max_tiles_y: i32, screen_y: f32, screen_h: f32, world_surface: f64) {
        if !only_missing {
            self.lanterns = vec![
                Lantern {
                    variant: 0,
                    time_until_float: 0,
                    time_until_float_max: 1,
                    position: Vec2::ZERO,
                    depth: 0.0,
                    rotation: 0.0,
                    speed: 0.0,
                    active: false,
                };
                (max_tiles_y / 4).max(0) as usize
            ];
        }
        let len = self.lanterns.len();
        for i in 0..len {
            if !only_missing || !self.lanterns[i].active {
                // `:93-94`: the band's two truncating casts.
                let num = (screen_y as f64 * 0.7 - screen_h as f64) as i32;
                let min_value = (num as f64 - world_surface * 16.0) as i32;
                // `:95`: X then Y, the C#'s draw order.
                let x = self.random.next_range(0, max_tiles_x) * 16;
                let y = self.random.next_range(min_value, num);
                self.lanterns[i].position = Vec2::new(x as f32, y as f32);
                self.reset_lantern(i, len);
                self.lanterns[i].active = true;
            }
        }
        self.lanterns_drawing = len as i32;
    }

    /// `ResetLantern` (`LanternSky.cs:103-111`): the depth ladder + the
    /// speed/variant/float-timer draws (the C#'s order).
    fn reset_lantern(&mut self, i: usize, len: usize) {
        let l = &mut self.lanterns[i];
        l.depth = (1.0 - i as f32 / len as f32) * 4.4 + 1.6;
        l.speed = -1.5 - 2.5 * self.random.next_double() as f32;
        l.variant = self.random.next_max(3);
        l.time_until_float = ((2000 + self.random.next_max(1200)) as f32 * 2.0) as i32;
        l.time_until_float_max = l.time_until_float;
    }

    /// `Update` (`LanternSky.cs:113-152`), one `SkyManager.Update` iteration
    /// (`max(1, dayRate)` per tick — the caller loops). `lanterns_up` is
    /// `LanternNight.LanternsUp`, `wind` is `Main.windSpeedCurrent`.
    pub(crate) fn update(&mut self, lanterns_up: bool, wind: f32, max_tiles_x: i32, world_surface: f64) {
        // `:119`: the opacity walks ±0.01 toward the lanterns-up direction.
        let dir = if lanterns_up { 1.0 } else { -1.0 };
        self.opacity = (self.opacity + dir * 0.01).clamp(0.0, 1.0);
        let len = self.lanterns.len();
        for i in 0..len {
            if !self.lanterns[i].active {
                continue;
            }
            // `:126-130`: the wind stand-in (0 → 0.1).
            let mut num = wind;
            if num == 0.0 {
                num = 0.1;
            }
            // `:131-135`: the sine bob, the float, the wind drift, rotation.
            let num2 = (self.lanterns[i].position.x / 120.0).sin() * 0.5;
            self.lanterns[i].position.y += num2 * 0.5;
            self.lanterns[i].position.y += self.lanterns[i].float_adjusted_speed() * 0.5;
            self.lanterns[i].position.x += (0.1 + num)
                * (3.0 - self.lanterns[i].speed)
                * 0.5
                * (i as f32 / len as f32 + 1.5)
                / 2.5;
            self.lanterns[i].rotation = num2 * if num >= 0.0 { 1.0 } else { -1.0 } * 0.5;
            // `:136`: the float timer decays to 0.
            self.lanterns[i].time_until_float = (self.lanterns[i].time_until_float - 1).max(0);
            // `:137-149`: past the 300-px ceiling, reset (or leave).
            if self.lanterns[i].position.y < 300.0 {
                if !self.leaving {
                    self.reset_lantern(i, len);
                    let x = self.random.next_range(0, max_tiles_x) * 16;
                    self.lanterns[i].position =
                        Vec2::new(x as f32, world_surface as f32 * 16.0 + 1600.0);
                } else {
                    self.lanterns[i].active = false;
                    self.lanterns_drawing -= 1;
                }
            }
        }
        // `:151`: the sky stays active while it updates.
        self.active = true;
    }

    /// `Draw` (`LanternSky.cs:154-240`) + `DrawLantern` (`:242-251`), one
    /// `DrawToDepth` band. `global_time` is `Main.GlobalTimeWrappedHourly`.
    pub(crate) fn draw(
        &self,
        r: &mut SpriteRenderer,
        lookups: &Lookups,
        camera: Vec2,
        zoom: f32,
        screen: Vec2,
        world_surface: f64,
        global_time: f32,
        min_depth: f32,
        max_depth: f32,
    ) {
        // `:165`: the underground / opacity gate (the menu gate is the
        // phase's own — module docs). `Main.screenPosition.Y` is the
        // unscaled view top-left (`camera.y - screen.y*0.5`).
        if camera.y - screen.y * 0.5 > world_surface * 16.0 || self.opacity <= 0.0 {
            return;
        }
        let Some(sheet) = lookups.lantern_sky else {
            return;
        };
        // `:169-187`: the depth-band index walk. `num` = the first lantern
        // with `depth < maxDepth`; `num2` = the last with `depth > minDepth`.
        let mut num: i32 = -1;
        let mut num2: i32 = 0;
        for (j, l) in self.lanterns.iter().enumerate() {
            if num == -1 && l.depth < max_depth {
                num = j as i32;
            }
            if l.depth <= min_depth {
                break;
            }
            num2 = j as i32;
        }
        if num == -1 {
            return;
        }
        // `:188-189`: the view centre + the 1000-px cull rect. `vector =
        // screenPosition + (screenWidth>>1, screenHeight>>1)` — the integer
        // halves; `screenPosition` is the unscaled view top-left
        // (`camera - screen*0.5`, the bg/ambience convention).
        let screen_position = camera - screen * 0.5;
        let vector = screen_position + Vec2::new((screen.x as i32 >> 1) as f32, (screen.y as i32 >> 1) as f32);
        let frame_w = sheet.width / 3;
        let frame_h = sheet.height;
        // `:244`: the hourly bob phase (the glow's vertical wobble).
        let y = ((global_time % 6.0) / 6.0 * std::f32::consts::TAU).sin();
        for k in num..num2 {
            let l = &self.lanterns[k as usize];
            if !l.active {
                continue;
            }
            // `:194-224`: the (250, 120, 60, 120) tint × the depth ladder.
            let num3 = if l.depth > 5.0 {
                0.3
            } else if l.depth > 4.5 {
                0.4
            } else if l.depth > 4.0 {
                0.5
            } else if l.depth > 3.5 {
                0.6
            } else if l.depth > 3.0 {
                0.7
            } else if l.depth > 2.5 {
                0.8
            } else if l.depth > 2.0 {
                0.9
            } else {
                1.0
            };
            let body = [
                (250.0 * num3) as u8 as f32 / 255.0,
                (120.0 * num3) as u8 as f32 / 255.0,
                (60.0 * num3) as u8 as f32 / 255.0,
                (120.0 * num3) as u8 as f32 / 255.0,
            ];
            // `:225-233`: the parallax about the view centre + the X wrap.
            // `position = (Position - vector) * vector2 + vector -
            // screenPosition` — screen pixels relative to the view top-left.
            let depth_scale = Vec2::new(1.0 / l.depth, 0.9 / l.depth) * 1.2;
            let mut position = (l.position - vector) * depth_scale + vector - screen_position;
            position.x = (position.x + 500.0) % 4000.0;
            if position.x < 0.0 {
                position.x += 4000.0;
            }
            position.x -= 500.0;
            // `:234`: the cull rect (-1000, -1000, screen+1000, screen+1000).
            if position.x < -1000.0
                || position.y < -1000.0
                || position.x >= screen.x + 1000.0
                || position.y >= screen.y + 1000.0
            {
                continue;
            }
            // The source rect: variant column `FrameWidth * Variant`.
            let sx = sheet.x + frame_w * l.variant as u32;
            let uv = [
                sx as f32 / lookups.atlas_size.0 as f32,
                sheet.y as f32 / lookups.atlas_size.1 as f32,
                (sx + frame_w) as f32 / lookups.atlas_size.0 as f32,
                (sheet.y + frame_h) as f32 / lookups.atlas_size.1 as f32,
            ];
            let scale = depth_scale.x * 2.0;
            let size = [frame_w as f32 * scale * zoom, frame_h as f32 * scale * zoom];
            // `DrawLantern` (`:242-251`): the 3 glow copies then the body.
            // The glow is `Color(255,255,255,0) * opacity * num3 * (y*0.2+
            // 0.8) * 0.4` — alpha 0 ADDS the rgb (the premultiplied-source
            // convention, `sprite.wgsl`'s tile-flame note).
            let glow_a = self.opacity * num3 * (y * 0.2 + 0.8) * 0.4;
            let glow = [glow_a, glow_a, glow_a, 0.0];
            for n in 0..3 {
                let frac = n as f32 / 3.0;
                let ang = std::f32::consts::TAU * frac + l.rotation;
                let off = Vec2::new(0.0, 2.0).rotate(Vec2::from_angle(ang)) * y;
                push_screen_rot(
                    r,
                    camera,
                    zoom,
                    screen,
                    [position.x + off.x, position.y + off.y],
                    size,
                    uv,
                    glow,
                    l.rotation,
                );
            }
            // The body: `opacity * this._opacity` (the (250,120,60,120)
            // tint × the depth ladder × the fade).
            let body_tint = [
                body[0] * self.opacity,
                body[1] * self.opacity,
                body[2] * self.opacity,
                body[3] * self.opacity,
            ];
            push_screen_rot(r, camera, zoom, screen, [position.x, position.y], size, uv, body_tint, l.rotation);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// W4-f40 — `GenerateLanterns` (`LanternSky.cs:83-101`): the array is
    /// `maxTilesY / 4`, every lantern activates, and the depth ladder runs
    /// `(1 - i/len) * 4.4 + 1.6` (6.0 at i=0 down toward 1.6). The private
    /// stream is deterministic (the pinned seed).
    #[test]
    fn w4_f40_generate_lanterns_fills_the_depth_ladder() {
        let mut sky = LanternSky::new();
        sky.activate(800, 1200, 5000.0, 720.0, 400.0);
        assert_eq!(sky.lanterns.len(), 300, "maxTilesY / 4 = 1200 / 4");
        assert_eq!(sky.lanterns_drawing, 300);
        assert!(sky.active);
        assert!(sky.lanterns.iter().all(|l| l.active));
        // The depth ladder's endpoints (`:105`).
        assert!((sky.lanterns[0].depth - 6.0).abs() < 1e-4, "i=0 → 6.0");
        let last = sky.lanterns[299].depth;
        assert!((last - ((1.0 - 299.0 / 300.0) * 4.4 + 1.6)).abs() < 1e-4);
        // The float timer is `(2000 + Next(1200)) * 2` ∈ [4000, 6398].
        for l in &sky.lanterns {
            assert!((4000..=6398).contains(&l.time_until_float), "{}", l.time_until_float);
            assert_eq!(l.time_until_float, l.time_until_float_max);
            assert!((0..3).contains(&l.variant));
            // The speed is `-1.5 - 2.5 * NextDouble()` ∈ [-4.0, -1.5).
            assert!(l.speed >= -4.0 && l.speed < -1.5, "{}", l.speed);
        }
        // Determinism: a second sky with the same seed regenerates identically.
        let mut twin = LanternSky::new();
        twin.activate(800, 1200, 5000.0, 720.0, 400.0);
        for (a, b) in sky.lanterns.iter().zip(twin.lanterns.iter()) {
            assert_eq!(a.position, b.position);
            assert_eq!(a.depth, b.depth);
            assert_eq!(a.speed, b.speed);
            assert_eq!(a.variant, b.variant);
        }
    }

    /// W4-f40 — `Update` (`LanternSky.cs:113-152`): the opacity walks +0.01
    /// toward lanterns-up and −0.01 toward down, clamped 0..1; the float
    /// timer decays; a lantern rising past 300 resets (or leaves).
    #[test]
    fn w4_f40_update_walks_the_opacity_and_decays_the_float_timer() {
        let mut sky = LanternSky::new();
        sky.activate(800, 1200, 5000.0, 720.0, 400.0);
        // 100 up-ticks reach full opacity.
        for _ in 0..100 {
            sky.update(true, 0.0, 800, 400.0);
        }
        assert!((sky.opacity - 1.0).abs() < 1e-4, "opacity 1 after 100 up-ticks");
        // The float timers decayed by 100 (clamped at 0).
        assert!(sky.lanterns.iter().all(|l| l.time_until_float <= l.time_until_float_max - 100));
        // 100 down-ticks return to 0.
        for _ in 0..100 {
            sky.update(false, 0.0, 800, 400.0);
        }
        assert!(sky.opacity.abs() < 1e-4, "opacity 0 after 100 down-ticks");
        // The leaving latch: a lantern past the ceiling deactivates.
        sky.deactivate();
        sky.lanterns[0].position.y = 200.0;
        let before = sky.lanterns_drawing;
        sky.update(false, 0.0, 800, 400.0);
        assert!(!sky.lanterns[0].active, "the leaving lantern deactivates");
        assert_eq!(sky.lanterns_drawing, before - 1);
    }

    /// W4-f40 — the reset/leave branch (`LanternSky.cs:137-149`): a lantern
    /// rising past 300 while NOT leaving resets to the surface band
    /// (`worldSurface * 16 + 1600`) with a fresh float timer.
    #[test]
    fn w4_f40_a_rising_lantern_resets_to_the_surface_band() {
        let mut sky = LanternSky::new();
        sky.activate(800, 1200, 5000.0, 720.0, 400.0);
        sky.lanterns[0].position.y = 250.0;
        sky.lanterns[0].time_until_float = 0;
        // Zero the speed so the float term is exactly 0 (the reset Y is then
        // the pure surface band + the sine bob).
        sky.lanterns[0].speed = 0.0;
        sky.update(true, 0.0, 800, 400.0);
        assert!(sky.lanterns[0].active, "the non-leaving lantern stays active");
        // The reset Y is `worldSurface * 16 + 1600` = 400*16+1600 = 8000,
        // then the same tick's bob/float nudges it slightly.
        assert!(sky.lanterns[0].position.y > 7000.0, "reset to the surface band");
        assert!(sky.lanterns[0].time_until_float > 0, "a fresh float timer");
    }
}
