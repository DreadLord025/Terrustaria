//! The Slime Rain sky — `SlimeSky : CustomSky`
//! (`GameContent/Skies/SlimeSky.cs`), the falling-slime background the
//! `SkyManager` activates/deactivates on `StartSlimeRain`/`StopSlimeRain`
//! (`Main.cs:64773`/`:64813`, the single-player arms) and updates
//! `max(1, dayRate)` times per tick (`SkyManager.cs:24-46`). W4-f39
//! closeout: the g67 census pinned that the sky overlay never touches
//! `Main.rain` — this module is the event's own visual, disjoint from the
//! rain field (`rain.rs`).
//!
//! The C# owns a `new UnifiedRandom()` (`SlimeSky.cs:69`) — the
//! parameterless ctor is clock-seeded (`UnifiedRandom.cs:18-21`); the port
//! seeds it deterministically from the world seed (the c39 `event_skies`
//! deviation: the visual scatter is reproducible, the stream shape is the
//! exact C# one).
//!
//! Activation edges: the C# calls `SkyManager.Activate("Slime")` inside
//! `StartSlimeRain` and `Deactivate` inside `StopSlimeRain`; the port
//! edge-detects `World.slime_rain` at the update seam instead (the five
//! start/stop call sites stay world-only). Same-tick behaviour: the sky
//! generates on the first update after the flag rises and drains
//! (`_isLeaving`) after it falls. The load path (`WorldFile.cs:767-769`'s
//! `StartSlimeRain(announce: false)` when `slimeRainTime > 0`) is covered
//! by the same edge — the `.trwld` v6 block restores the flag.
//!
//! The draw is a depth-sandwich member (`SkyManager.DrawToDepth`): slimes
//! span depths 1.6..3.35 (+0.5 on the 1/60 rare variant), drawn far-first
//! in index order at the sky pass's per-depth boundaries (the c39
//! `draw_balloons_band` shape, threaded through `draw_sky`/`AmbiencePass`).
//! The C# index walk (`:168-186`) BREAKS on the first slime at/below the
//! band's min — with ascending depths that empties every band whose min
//! is >= 1.6, so the visible slimes draw at the 1.0 front-row close and
//! the flush, and the loop's `j < num2` excludes the LAST (nearest)
//! slime. Both quirks are kept verbatim (the PartySky twin has them too).

use glam::Vec2;
use tr_core::UnifiedRandom;

/// One falling sky slime (`SlimeSky.Slime`, `SlimeSky.cs:11-63`).
#[derive(Clone, Debug)]
pub(crate) struct SkySlime {
    pub active: bool,
    pub position: Vec2,
    pub depth: f32,
    pub speed: f32,
    /// The sheet index 0..3 (`Sky_Slime_1..4`); 3 is the rare 1/60 variant,
    /// 2 the 1/30 one.
    pub texture: u32,
    /// `_frame` — the `Frame` setter wraps at 24 (`:55`, MAX_FRAMES 4 ×
    /// FRAME_RATE 6).
    pub frame: i32,
}

impl SkySlime {
    /// `GetSourceRectangle` (`:59-62`): row `_frame / 6` of the 4-row
    /// sheet, full width.
    pub fn frame_row(&self) -> u32 {
        (self.frame / 6) as u32
    }
}

/// The per-tick inputs the sky reads off `Main` (`screenPosition.Y`,
/// `screenHeight`, `maxTilesX/Y`, `worldSurface`).
#[derive(Clone, Copy)]
pub(crate) struct SlimeSkyCtx {
    pub screen_position_y: f32,
    pub screen_height: f32,
    pub max_tiles_x: i32,
    pub max_tiles_y: i32,
    pub world_surface: f64,
}

/// `SlimeSky`'s live state.
pub(crate) struct SlimeSky {
    /// `_isActive` (`:73`).
    active: bool,
    /// `_isLeaving` (`:75`) — `Deactivate` sets it; the update drains.
    leaving: bool,
    slimes: Vec<SkySlime>,
    /// `_slimesRemaining` (`:71`).
    remaining: i32,
    rng: UnifiedRandom,
    /// The previous tick's `Main.slimeRain` — the activation edge detector
    /// (the port's stand-in for the `SkyManager.Activate/Deactivate` calls
    /// inside `StartSlimeRain`/`StopSlimeRain`).
    was_up: bool,
    /// The 60 Hz accumulator (the `Ambience::update` cadence precedent).
    accumulator: f32,
}

impl SlimeSky {
    /// Seed the private stream from the world seed (the C# parameterless
    /// `UnifiedRandom()` is clock-seeded; the port pins it — the c39
    /// `EventSkies::new` deviation).
    pub(crate) fn new(world_seed: u64) -> Self {
        Self {
            active: false,
            leaving: false,
            slimes: Vec::new(),
            remaining: 0,
            rng: UnifiedRandom::new((world_seed ^ 0x5117_5EED) as i32),
            was_up: false,
            accumulator: 0.0,
        }
    }

    /// `SkyManager.Reset` → `SlimeSky.Reset` (`SlimeSky.cs:241-244`): the
    /// world-load/menu-return clear. `_isActive = false` only; the next
    /// activation regenerates.
    pub(crate) fn reset(&mut self) {
        self.active = false;
        self.leaving = false;
        self.was_up = false;
        self.accumulator = 0.0;
    }

    pub(crate) fn is_active(&self) -> bool {
        self.active
    }

    /// The per-frame entry: the `Main.slimeRain` activation edges, then
    /// the fixed-step 60 Hz update ×`max(1, dayRate)` per step
    /// (`SkyManager.Update`, `SkyManager.cs:24-46`).
    pub(crate) fn update(&mut self, dt: f32, slime_rain: bool, day_rate: i32, ctx: &SlimeSkyCtx) {
        // `StartSlimeRain`'s `Activate("Slime")` (`Main.cs:64773`):
        // `GenerateSlimes` + `_isActive = true` + `_isLeaving = false`
        // (`SlimeSky.cs:229-234`). A re-activate while active regenerates
        // wholesale (unlike LanternSky's fill-missing arm).
        if slime_rain && !self.was_up {
            self.generate(ctx);
            self.active = true;
            self.leaving = false;
        } else if !slime_rain && self.was_up {
            // `StopSlimeRain`'s `Deactivate("Slime")` (`Main.cs:64813`):
            // `_isLeaving = true` (`SlimeSky.cs:236-239`) — the slimes
            // drain past the surface line instead of resetting.
            self.leaving = true;
        }
        self.was_up = slime_rain;
        if !self.active {
            return;
        }
        self.accumulator += dt;
        const STEP: f32 = 1.0 / 60.0;
        while self.accumulator >= STEP {
            self.accumulator -= STEP;
            for _ in 0..day_rate.max(1) {
                self.update_once(ctx);
            }
        }
    }

    /// `GenerateSlimes` (`SlimeSky.cs:87-112`): `maxTilesY / 6` slots
    /// scattered across the world width and the band above the view.
    /// Draw order per slime: `Next(0, maxTilesX)`, `Next(minValue, num)`,
    /// `NextDouble` (speed), `Next(2)` (texture), `Next(60)` (the rare
    /// roll), then the conditional `NextDouble`/`Next(30)` pair.
    fn generate(&mut self, ctx: &SlimeSkyCtx) {
        let count = (ctx.max_tiles_y / 6) as usize;
        self.slimes = vec![
            SkySlime {
                active: false,
                position: Vec2::ZERO,
                depth: 0.0,
                speed: 0.0,
                texture: 0,
                frame: 0,
            };
            count
        ];
        // `num = (int)(screenPosition.Y * 0.7 - screenHeight)` (`:92`),
        // `minValue = (int)(num - worldSurface * 16.0)` (`:93`) — the
        // f64 subtraction truncates AFTER the product.
        let num = (ctx.screen_position_y as f64 * 0.7 - ctx.screen_height as f64) as i32;
        let min_value = (num as f64 - ctx.world_surface * 16.0) as i32;
        let len = self.slimes.len() as f32;
        for i in 0..self.slimes.len() {
            let x = self.rng.next_range(0, ctx.max_tiles_x) * 16;
            let y = self.rng.next_range(min_value, num);
            self.slimes[i].position = Vec2::new(x as f32, y as f32);
            self.slimes[i].speed = 5.0 + 3.0 * self.rng.next_double() as f32;
            self.slimes[i].depth = i as f32 / len * 1.75 + 1.6;
            self.slimes[i].texture = self.rng.next_range(0, 2) as u32;
            if self.rng.next_range(0, 60) == 0 {
                self.slimes[i].texture = 3;
                self.slimes[i].speed = 6.0 + 3.0 * self.rng.next_double() as f32;
                self.slimes[i].depth += 0.5;
            } else if self.rng.next_range(0, 30) == 0 {
                self.slimes[i].texture = 2;
                self.slimes[i].speed = 6.0 + 2.0 * self.rng.next_double() as f32;
            }
            self.slimes[i].active = true;
        }
        self.remaining = self.slimes.len() as i32;
    }

    /// `SlimeSky.Update` (`SlimeSky.cs:114-160`), one `SkyManager`
    /// iteration. The `FocusHelper.PauseSkies` gate (`:116`) folds into
    /// the port's pause-gated ambience phase (the caller never runs while
    /// paused). Each active slime advances its frame and falls; past the
    /// surface line it resets (draw order: `Next(0, maxTilesX)` for X,
    /// `Next(2)`, `NextDouble`, `Next(60)`, then the conditional pair —
    /// the texture draw precedes the speed draw here, unlike generate)
    /// or deactivates while leaving.
    fn update_once(&mut self, ctx: &SlimeSkyCtx) {
        let len = self.slimes.len() as f32;
        for i in 0..self.slimes.len() {
            if !self.slimes[i].active {
                continue;
            }
            self.slimes[i].frame = (self.slimes[i].frame + 1) % 24;
            self.slimes[i].position.y += self.slimes[i].speed;
            if !((self.slimes[i].position.y as f64) > ctx.world_surface * 16.0) {
                continue;
            }
            if !self.leaving {
                self.slimes[i].depth = i as f32 / len * 1.75 + 1.6;
                let x = self.rng.next_range(0, ctx.max_tiles_x) * 16;
                self.slimes[i].position = Vec2::new(x as f32, -100.0);
                self.slimes[i].texture = self.rng.next_range(0, 2) as u32;
                self.slimes[i].speed = 5.0 + 3.0 * self.rng.next_double() as f32;
                if self.rng.next_range(0, 60) == 0 {
                    self.slimes[i].texture = 3;
                    self.slimes[i].speed = 6.0 + 3.0 * self.rng.next_double() as f32;
                    self.slimes[i].depth += 0.5;
                } else if self.rng.next_range(0, 30) == 0 {
                    self.slimes[i].texture = 2;
                    self.slimes[i].speed = 6.0 + 2.0 * self.rng.next_double() as f32;
                }
            } else {
                self.slimes[i].active = false;
                self.remaining -= 1;
            }
        }
        if self.remaining == 0 {
            self.active = false;
        }
    }

    /// `SlimeSky.Draw`'s index walk (`:168-182`): `num` = the first slime
    /// whose depth is below `max_depth`; the walk BREAKS on the first
    /// depth at/below `min_depth` — with the ascending depth ladder that
    /// is index 0 whenever `min_depth >= 1.6`, leaving `num2 = 0` and the
    /// `j < num2` loop empty (the C# quirk, kept — the visible bands are
    /// the ones below the whole ladder). Otherwise `num2` = the last
    /// index and the loop EXCLUDES it. `None` when no slime is below
    /// `max_depth` (`num == -1`, `:183`).
    fn band_indices(&self, min_depth: f32, max_depth: f32) -> Option<(usize, usize)> {
        let mut num: i32 = -1;
        let mut num2 = 0usize;
        for (i, s) in self.slimes.iter().enumerate() {
            if num == -1 && s.depth < max_depth {
                num = i as i32;
            }
            if s.depth <= min_depth {
                break;
            }
            num2 = i;
        }
        (num != -1).then_some((num as usize, num2))
    }

    /// `SlimeSky.Draw` (`SlimeSky.cs:162-227`) over one `DrawDepthRange`
    /// band. `origin` is the unscaled view top-left (`Main.screenPosition`
    /// in the sky pass's space). The index walk (`:168-182`) finds the
    /// first slime below `max_depth` and the last above `min_depth`; the
    /// draw loop is `j < num2` — the LAST in-range slime is excluded (the
    /// C# quirk, kept). The `screenPosition.Y > 10000 || gameMenu` gate
    /// (`:164`) is the caller's (the sky pass only runs in-world; the
    /// depth gate rides here).
    pub(crate) fn draw_band(
        &self,
        r: &mut tr_render::SpriteRenderer,
        sheets: &[Option<crate::render::BgTex>; 4],
        camera: Vec2,
        zoom: f32,
        screen: Vec2,
        origin: Vec2,
        sky_color: [f32; 3],
        min_depth: f32,
        max_depth: f32,
    ) {
        if origin.y > 10000.0 || max_depth <= min_depth {
            return;
        }
        let Some((num, num2)) = self.band_indices(min_depth, max_depth) else {
            return;
        };
        // `vector = screenPosition + (screenWidth >> 1, screenHeight >> 1)`
        // (`:187`) — INTEGER halving.
        let vector = origin
            + Vec2::new(
                ((screen.x as i32) >> 1) as f32,
                ((screen.y as i32) >> 1) as f32,
            );
        for j in num..num2 {
            let s = &self.slimes[j];
            if !s.active {
                continue;
            }
            let Some(tex) = sheets[s.texture as usize] else {
                continue;
            };
            let color = slime_tint(sky_color, s.depth);
            let par = Vec2::new(1.0 / s.depth, 0.9 / s.depth);
            let mut pos = (s.position - vector) * par + vector - origin;
            // The 4000-px X wrap (`:215-220`).
            pos.x = (pos.x + 500.0) % 4000.0;
            if pos.x < 0.0 {
                pos.x += 4000.0;
            }
            pos.x -= 500.0;
            // `rectangle.Contains((int)position.X, (int)position.Y)`
            // (`:188`, `:221`): the (-1000, -1000, w+1000, h+1000) cull.
            let px = pos.x as i32;
            let py = pos.y as i32;
            if px < -1000
                || py < -1000
                || px >= -1000 + (screen.x as i32 + 1000)
                || py >= -1000 + (screen.y as i32 + 1000)
            {
                continue;
            }
            // `GetSourceRectangle`: row `frame / 6` of the 4-row sheet
            // (`FrameHeight = Height / 4`, integer).
            let frame_h = (tex.h / 4.0).floor();
            let dv = tex.uv[3] - tex.uv[1];
            let row = s.frame_row() as f32;
            let uv = [
                tex.uv[0],
                tex.uv[1] + dv * (row * frame_h / tex.h),
                tex.uv[2],
                tex.uv[1] + dv * ((row + 1.0) * frame_h / tex.h),
            ];
            // `scale: vector2.X * 2f` (`:223`), origin Zero (top-left).
            let scale = par.x * 2.0;
            crate::render::bg::push_screen(
                r,
                camera,
                zoom,
                screen,
                [pos.x, pos.y],
                [tex.w * scale, frame_h * scale],
                uv,
                color,
            );
        }
    }
}

/// The slime draw colour (`SlimeSky.cs:193-212`):
/// `Color(ColorOfTheSkies.ToVector4() * 0.9 + Vector4(0.1)) * 0.8`, then
/// the depth tier (1 / 0.9 / 0.8 / 0.7 / 0.6) × 0.8, every step an XNA
/// byte multiply (`(int)(channel * factor)` truncation).
pub(crate) fn slime_tint(sky: [f32; 3], depth: f32) -> [f32; 4] {
    // `Color(Vector4)` clamps each channel and truncates `v * 255`.
    let base = [
        sky[0] * 0.9 + 0.1,
        sky[1] * 0.9 + 0.1,
        sky[2] * 0.9 + 0.1,
        1.0 * 0.9 + 0.1,
    ];
    let b: [u8; 4] = base.map(|v| (v.clamp(0.0, 1.0) * 255.0) as u8);
    // `* 0.8f` — the XNA `Color.Multiply` byte truncation.
    let b: [u8; 4] = b.map(|c| (c as f32 * 0.8) as u8);
    let tier = if depth > 3.0 {
        0.6
    } else if depth > 2.5 {
        0.7
    } else if depth > 2.0 {
        0.8
    } else if depth > 1.5 {
        0.9
    } else {
        1.0
    };
    let t = tier * 0.8;
    b.map(|c| ((c as f32 * t) as u8) as f32 / 255.0)
}

#[cfg(test)]
#[path = "slime_sky_w4_f39.rs"]
mod slime_sky_w4_f39;
