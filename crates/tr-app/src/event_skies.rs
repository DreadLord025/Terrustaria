//! The two world-event skies: `LanternSky` (`GameContent/Skies/LanternSky.cs`)
//! and `PartySky` (`GameContent/Skies/PartySky.cs`) — the lantern-night
//! floating lanterns and the birthday-party balloons. Both are `CustomSky`
//! effects the `SkyManager` activates/deactivates on the schedulers'
//! `UpdateTime` edges (`LanternNight.cs:133-150`, `BirthdayParty.cs:185-197`)
//! and updates `max(1, dayRate)` times per tick (`SkyManager.cs:24-46`).
//!
//! Each owns a `new UnifiedRandom()` (`LanternSky.cs:71`, `PartySky.cs:79`) —
//! the C# parameterless ctor is clock-seeded (`UnifiedRandom.cs:18-21`), so the
//! port seeds it deterministically from the world seed (a recorded deviation:
//! the visual scatter is reproducible, the stream shape is the exact C# one).
//!
//! The draw is a depth-sandwich member (`SkyManager.DrawToDepth`): lanterns
//! span depths 1.6..6.0 (far first), balloons 1.6..3.35. The port draws them
//! in the sky pass's `DrawRemainingDepth` slot (after the surface layers,
//! before the underground backdrop) — a recorded layering deviation (the
//! per-depth boundary interleave with the mountain/tree rows is the ambience
//! pass's own machinery; these two skies draw as one batch at the flush).

use glam::Vec2;
use tr_core::UnifiedRandom;

/// One floating lantern (`LanternSky.Lantern`, `LanternSky.cs:13-59`).
#[derive(Clone, Debug)]
pub(crate) struct Lantern {
    pub active: bool,
    pub position: Vec2,
    pub depth: f32,
    pub rotation: f32,
    pub speed: f32,
    pub variant: u32,
    pub time_until_float: i32,
    pub time_until_float_max: i32,
}

impl Lantern {
    /// `FloatAdjustedSpeed` (`:53`): `Speed * (TimeUntilFloat / TimeUntilFloatMax)`.
    fn float_adjusted_speed(&self) -> f32 {
        self.speed * (self.time_until_float as f32 / self.time_until_float_max as f32)
    }
}

/// One party balloon (`PartySky.Balloon`, `PartySky.cs:11-67`).
#[derive(Clone, Debug)]
pub(crate) struct Balloon {
    pub active: bool,
    pub position: Vec2,
    pub depth: f32,
    pub speed: f32,
    /// The sheet index 0..2 (`Extra_69..71`); 2 is the rare 1/30 variant.
    pub texture: u32,
    pub variant: u32,
    /// `_frameCounter` (the `Frame` setter wraps at 42, `:59`).
    pub frame_counter: i32,
}

impl Balloon {
    /// `GetSourceRectangle` (`:63-66`): column `Variant`, row
    /// `_frameCounter / 14` of the 3x3 grid.
    pub fn frame_cell(&self) -> (u32, u32) {
        (self.variant, (self.frame_counter / 14) as u32)
    }
}

/// The two event skies' live state.
pub(crate) struct EventSkies {
    // --- LanternSky ---
    lantern_active: bool,
    lantern_leaving: bool,
    lantern_opacity: f32,
    lanterns: Vec<Lantern>,
    lanterns_drawing: i32,
    lantern_rng: UnifiedRandom,
    // --- PartySky ---
    party_active: bool,
    party_leaving: bool,
    /// `GameEffect.Opacity` — `SceneState.cs:106` ramps it toward 1 while a
    /// town NPC (or party monolith) is present, else toward 0, 0.01/tick.
    pub party_opacity: f32,
    balloons: Vec<Balloon>,
    balloons_drawing: i32,
    party_rng: UnifiedRandom,
    /// `PartySky.MultipleSkyWorkaroundFix` (`:69`): the dayRate-0 skip latch.
    party_workaround: bool,
}

impl Default for EventSkies {
    fn default() -> Self {
        Self::new(0)
    }
}

impl EventSkies {
    /// Seed both skies' private streams from the world seed (the C#
    /// parameterless `UnifiedRandom()` is clock-seeded; the port pins it).
    pub(crate) fn new(world_seed: u64) -> Self {
        Self {
            lantern_active: false,
            lantern_leaving: false,
            lantern_opacity: 0.0,
            lanterns: Vec::new(),
            lanterns_drawing: 0,
            lantern_rng: UnifiedRandom::new((world_seed ^ 0x1A2B_3C4D) as i32),
            party_active: false,
            party_leaving: false,
            party_opacity: 0.0,
            balloons: Vec::new(),
            balloons_drawing: 0,
            party_rng: UnifiedRandom::new((world_seed ^ 0x5E6F_7A8B) as i32),
            party_workaround: false,
        }
    }

    /// `LanternSky.Activate` (`:253-266`): a re-activate while active fills
    /// only the missing slots; a fresh one regenerates all.
    pub(crate) fn activate_lanterns(&mut self, ctx: &SkyCtx) {
        if self.lantern_active {
            self.lantern_leaving = false;
            self.generate_lanterns(true, ctx);
        } else {
            self.generate_lanterns(false, ctx);
            self.lantern_active = true;
            self.lantern_leaving = false;
        }
    }

    /// `LanternSky.Deactivate` (`:268-271`).
    pub(crate) fn deactivate_lanterns(&mut self) {
        self.lantern_leaving = true;
    }

    /// `PartySky.Activate` (`:247-260`).
    pub(crate) fn activate_party(&mut self, ctx: &SkyCtx) {
        if self.party_active {
            self.party_leaving = false;
            self.generate_balloons(true, ctx);
        } else {
            self.generate_balloons(false, ctx);
            self.party_active = true;
            self.party_leaving = false;
        }
    }

    /// `PartySky.Deactivate` (`:262-265`).
    pub(crate) fn deactivate_party(&mut self) {
        self.party_leaving = true;
    }

    /// `LanternSky.GenerateLanterns` (`:83-101`): `maxTilesY / 4` slots, each
    /// scattered across the world width and the band above the view.
    fn generate_lanterns(&mut self, only_missing: bool, ctx: &SkyCtx) {
        let count = (ctx.max_tiles_y / 4) as usize;
        if !only_missing {
            self.lanterns = vec![
                Lantern {
                    active: false,
                    position: Vec2::ZERO,
                    depth: 0.0,
                    rotation: 0.0,
                    speed: 0.0,
                    variant: 0,
                    time_until_float: 0,
                    time_until_float_max: 1,
                };
                count
            ];
        }
        let num = (ctx.screen_position_y * 0.7 - ctx.screen_height) as i32;
        let min_value = (num as f64 - ctx.world_surface * 16.0) as i32;
        for i in 0..self.lanterns.len() {
            if !only_missing || !self.lanterns[i].active {
                let x = self.lantern_rng.next_range(0, ctx.max_tiles_x) * 16;
                let y = self.lantern_rng.next_range(min_value, num);
                self.lanterns[i].position = Vec2::new(x as f32, y as f32);
                self.reset_lantern(i);
                self.lanterns[i].active = true;
            }
        }
        self.lanterns_drawing = self.lanterns.len() as i32;
    }

    /// `LanternSky.ResetLantern` (`:103-111`): the depth ladder (far first),
    /// the −1.5..−4 speed, the 3-variant pick and the 4000..6400 float timer.
    fn reset_lantern(&mut self, i: usize) {
        let len = self.lanterns.len() as f32;
        self.lanterns[i].depth = (1.0 - i as f32 / len) * 4.4 + 1.6;
        self.lanterns[i].speed = -1.5 - 2.5 * self.lantern_rng.next_double() as f32;
        self.lanterns[i].variant = self.lantern_rng.next_range(0, 3) as u32;
        let t = ((2000 + self.lantern_rng.next_range(0, 1200)) as f32 * 2.0) as i32;
        self.lanterns[i].time_until_float = t;
        self.lanterns[i].time_until_float_max = t;
    }

    /// `PartySky.GenerateBalloons` (`:93-111`).
    fn generate_balloons(&mut self, only_missing: bool, ctx: &SkyCtx) {
        let count = (ctx.max_tiles_y / 4) as usize;
        if !only_missing {
            self.balloons = vec![
                Balloon {
                    active: false,
                    position: Vec2::ZERO,
                    depth: 0.0,
                    speed: 0.0,
                    texture: 0,
                    variant: 0,
                    frame_counter: 0,
                };
                count
            ];
        }
        let num = (ctx.screen_position_y * 0.7 - ctx.screen_height) as i32;
        let min_value = (num as f64 - ctx.world_surface * 16.0) as i32;
        for i in 0..self.balloons.len() {
            if !only_missing || !self.balloons[i].active {
                let x = self.party_rng.next_range(0, ctx.max_tiles_x) * 16;
                let y = self.party_rng.next_range(min_value, num);
                self.balloons[i].position = Vec2::new(x as f32, y as f32);
                self.reset_balloon(i);
                self.balloons[i].active = true;
            }
        }
        self.balloons_drawing = self.balloons.len() as i32;
    }

    /// `PartySky.ResetBalloon` (`:113-123`): the depth ladder (near first),
    /// the speed, the 2-sheet pick + the 1/30 rare third sheet, the 3-variant.
    fn reset_balloon(&mut self, i: usize) {
        let len = self.balloons.len() as f32;
        self.balloons[i].depth = i as f32 / len * 1.75 + 1.6;
        self.balloons[i].speed = -1.5 - 2.5 * self.party_rng.next_double() as f32;
        self.balloons[i].texture = self.party_rng.next_range(0, 2) as u32;
        self.balloons[i].variant = self.party_rng.next_range(0, 3) as u32;
        if self.party_rng.next_range(0, 30) == 0 {
            self.balloons[i].texture = 2;
        }
    }

    /// `LanternSky.Update` (`:113-152`), one `SkyManager` iteration. The
    /// opacity ramps ±0.01 toward `LanternsUp`; each lantern bobs on the
    /// sin of its X, drifts with the wind and the float-adjusted speed, and
    /// resets (or deactivates while leaving) above Y 300.
    pub(crate) fn update_lanterns(&mut self, lanterns_up: bool, wind: f32, ctx: &SkyCtx) {
        let dir = if lanterns_up { 1.0 } else { -1.0 };
        self.lantern_opacity = (self.lantern_opacity + dir * 0.01).clamp(0.0, 1.0);
        let len = self.lanterns.len() as f32;
        for i in 0..self.lanterns.len() {
            if !self.lanterns[i].active {
                continue;
            }
            let mut w = wind;
            if w == 0.0 {
                w = 0.1;
            }
            let num2 = (self.lanterns[i].position.x / 120.0).sin() * 0.5;
            self.lanterns[i].position.y += num2 * 0.5;
            self.lanterns[i].position.y += self.lanterns[i].float_adjusted_speed() * 0.5;
            self.lanterns[i].position.x += (0.1 + w)
                * (3.0 - self.lanterns[i].speed)
                * 0.5
                * (i as f32 / len + 1.5)
                / 2.5;
            self.lanterns[i].rotation = num2 * if w < 0.0 { -1.0 } else { 1.0 } * 0.5;
            self.lanterns[i].time_until_float =
                (self.lanterns[i].time_until_float - 1).max(0);
            if self.lanterns[i].position.y < 300.0 {
                if !self.lantern_leaving {
                    self.reset_lantern(i);
                    let x = self.lantern_rng.next_range(0, ctx.max_tiles_x) * 16;
                    self.lanterns[i].position =
                        Vec2::new(x as f32, ctx.world_surface as f32 * 16.0 + 1600.0);
                } else {
                    self.lanterns[i].active = false;
                    self.lanterns_drawing -= 1;
                }
            }
        }
        self.lantern_active = true;
    }

    /// `PartySky.Update` (`:125-169`), one iteration. The dayRate-0 skip
    /// (`MultipleSkyWorkaroundFix`), the 42-wrap frame counter, the rise +
    /// wind drift, and the reset/deactivate above Y 300.
    pub(crate) fn update_party(&mut self, day_rate: i32, wind: f32, ctx: &SkyCtx) {
        if !self.party_workaround && day_rate == 0 {
            return;
        }
        self.party_workaround = false;
        for i in 0..self.balloons.len() {
            if !self.balloons[i].active {
                continue;
            }
            self.balloons[i].frame_counter = (self.balloons[i].frame_counter + 1) % 42;
            self.balloons[i].position.y += self.balloons[i].speed;
            self.balloons[i].position.x += wind * (3.0 - self.balloons[i].speed);
            if self.balloons[i].position.y < 300.0 {
                if !self.party_leaving {
                    self.reset_balloon(i);
                    let x = self.party_rng.next_range(0, ctx.max_tiles_x) * 16;
                    self.balloons[i].position =
                        Vec2::new(x as f32, ctx.world_surface as f32 * 16.0 + 1600.0);
                    if self.party_rng.next_range(0, 30) == 0 {
                        self.balloons[i].texture = 2;
                    }
                } else {
                    self.balloons[i].active = false;
                    self.balloons_drawing -= 1;
                }
            }
        }
        if self.balloons_drawing == 0 {
            self.party_active = false;
        }
        self.party_active = true;
    }

    /// `SceneState.cs:106`'s `MoveTowards` on the party sky's `Opacity`:
    /// toward 1 while a town NPC (or party monolith) is present, else 0.
    pub(crate) fn ramp_party_opacity(&mut self, town_present: bool) {
        let target = if town_present { 1.0 } else { 0.0 };
        if self.party_opacity < target {
            self.party_opacity = (self.party_opacity + 0.01).min(target);
        } else if self.party_opacity > target {
            self.party_opacity = (self.party_opacity - 0.01).max(target);
        }
    }

    /// The lantern draw gate (`LanternSky.cs:165`): below the surface line,
    /// not in the menu, opacity > 0.
    pub(crate) fn lantern_draw_visible(&self, screen_y: f32, world_surface_px: f32) -> bool {
        screen_y <= world_surface_px && self.lantern_opacity > 0.0
    }
    /// The balloon draw gate (`PartySky.cs:182`).
    pub(crate) fn party_draw_visible(&self, screen_y: f32, world_surface_px: f32) -> bool {
        screen_y <= world_surface_px && self.party_opacity > 0.0
    }
}

/// The per-tick inputs the skies read off `Main` (`screenPosition`,
/// `screenWidth/Height`, `maxTilesX/Y`, `worldSurface`, `windSpeedCurrent`).
#[derive(Clone, Copy)]
pub(crate) struct SkyCtx {
    pub screen_position_y: f32,
    pub screen_height: f32,
    pub max_tiles_x: i32,
    pub max_tiles_y: i32,
    pub world_surface: f64,
}

/// `LanternSky.Draw` + `DrawLantern` (`LanternSky.cs:154-251`): the
/// depth-parallax lanterns with the 3-copy additive glow and the body. The
/// port draws them in the sky pass's flush slot (the depth-sandwich interleave
/// is the ambience pass's machinery — a recorded layering deviation). The
/// glow copies ride the additive blend (`push_ui_sprite_blend`'s mode), the
/// body the plain alpha; both in screen space about the view centre.
pub(crate) fn draw_lanterns(
    r: &mut tr_render::SpriteRenderer,
    skies: &EventSkies,
    lookups: &crate::assets::Lookups,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    global_time: f32,
    surface_px: f32,
) {
    let Some(rect) = lookups.extra_134.as_ref() else {
        return;
    };
    if !skies.lantern_draw_visible(camera.y - screen.y * 0.5 / zoom, surface_px) {
        return;
    }
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let frame_w = rect.width / 3;
    let frame_h = rect.height;
    let centre = camera; // the view centre in world px
    let top_left = camera - screen * 0.5 / zoom;
    // `y = (GlobalTimeWrappedHourly % 6 / 6 * 2π).ToRotationVector2().Y`.
    let y = (global_time % 6.0 / 6.0 * std::f32::consts::TAU).sin();
    for l in skies.lanterns.iter() {
        if !l.active {
            continue;
        }
        // The depth colour ladder (`:194-224`): (250,120,60,120) scaled by
        // the depth tier.
        let tier = if l.depth > 5.0 {
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
        let body_a = 120.0 / 255.0 * tier * skies.lantern_opacity;
        let body = [
            250.0 / 255.0 * tier * body_a,
            120.0 / 255.0 * tier * body_a,
            60.0 / 255.0 * tier * body_a,
            body_a,
        ];
        // `vector2 = (1/Depth, 0.9/Depth) * 1.2`.
        let par = Vec2::new(1.0 / l.depth, 0.9 / l.depth) * 1.2;
        let draw_pos = centre + (l.position - centre) * par;
        let mut pos = (draw_pos - top_left) * zoom;
        // The 4000-px X wrap (`:228-233`) — screen space, like the C#.
        pos.x = (pos.x + 500.0) % 4000.0;
        if pos.x < 0.0 {
            pos.x += 4000.0;
        }
        pos.x -= 500.0;
        // The cull rect (`:189`): (-1000, -1000, screen+1000).
        if pos.x < -1000.0
            || pos.x > screen.x + 1000.0
            || pos.y < -1000.0
            || pos.y > screen.y + 1000.0
        {
            continue;
        }
        let uv = [
            (rect.x + l.variant * frame_w) as f32 / aw,
            rect.y as f32 / ah,
            (rect.x + (l.variant + 1) * frame_w) as f32 / aw,
            (rect.y + frame_h) as f32 / ah,
        ];
        // The back range carries the world transform, so the size/origin are
        // in world px (the C# `scale = vector2.X * 2`, no zoom — the batch
        // transform applies it). The screen anchor maps back to world px.
        let scale = par.x * 2.0;
        let size = [frame_w as f32 * scale, frame_h as f32 * scale];
        let origin = [size[0] * 0.5, size[1] * 0.5];
        let anchor_world = top_left + pos / zoom;
        // The glow: 3 additive copies offset by `(0,2).RotatedBy(2π·n + rot)·y`
        // (`:246-249`), colour white·opacity·tier·(y·0.2+0.8)·0.4.
        let glow_a = skies.lantern_opacity * tier * (y * 0.2 + 0.8) * 0.4;
        let glow = [1.0, 1.0, 1.0, glow_a];
        for n in 0..3u32 {
            let ang = std::f32::consts::TAU * (n as f32 / 3.0) + l.rotation;
            let off = crate::bow_launch::rotated_by(Vec2::new(0.0, 2.0), ang as f64) * y;
            let gp = anchor_world + off;
            let tl = [gp[0] - origin[0], gp[1] - origin[1]];
            r.push_back_sprite_origin_rot_blend(
                tl,
                size,
                uv,
                glow,
                l.rotation,
                origin,
                tr_render::BlendMode::Additive,
            );
        }
        // The body (`:250`): plain alpha, `opacity * _opacity`.
        let tl = [anchor_world[0] - origin[0], anchor_world[1] - origin[1]];
        r.push_back_sprite_origin_rot_blend(tl, size, uv, body, l.rotation, origin, tr_render::BlendMode::AlphaBlend);
    }
}

/// `PartySky.Draw` (`PartySky.cs:171-245`): the depth-parallax balloons, the
/// sky-colour tint and the 3x3 animated grid.
pub(crate) fn draw_balloons(
    r: &mut tr_render::SpriteRenderer,
    skies: &EventSkies,
    lookups: &crate::assets::Lookups,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    sky_color: [f32; 3],
    surface_px: f32,
) {
    if !skies.party_draw_visible(camera.y - screen.y * 0.5 / zoom, surface_px) {
        return;
    }
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let centre = camera;
    let top_left = camera - screen * 0.5 / zoom;
    for b in skies.balloons.iter() {
        if !b.active {
            continue;
        }
        let Some(rect) = lookups.extra_balloons[b.texture as usize].as_ref() else {
            continue;
        };
        // The depth colour ladder (`:211-230`): (sky·0.9+0.1)·0.8, then the
        // tier scale ·0.9.
        let tier = if b.depth > 3.0 {
            0.6
        } else if b.depth > 2.5 {
            0.7
        } else if b.depth > 2.0 {
            0.8
        } else if b.depth > 1.5 {
            0.9
        } else {
            1.0
        } * 0.9;
        let base = [
            (sky_color[0] * 0.9 + 0.1) * 0.8,
            (sky_color[1] * 0.9 + 0.1) * 0.8,
            (sky_color[2] * 0.9 + 0.1) * 0.8,
            0.8,
        ];
        let a = base[3] * tier * skies.party_opacity;
        let color = [base[0] * tier * a, base[1] * tier * a, base[2] * tier * a, a];
        let par = Vec2::new(1.0 / b.depth, 0.9 / b.depth);
        let draw_pos = centre + (b.position - centre) * par;
        let mut pos = (draw_pos - top_left) * zoom;
        pos.x = (pos.x + 500.0) % 4000.0;
        if pos.x < 0.0 {
            pos.x += 4000.0;
        }
        pos.x -= 500.0;
        if pos.x < -1000.0
            || pos.x > screen.x + 1000.0
            || pos.y < -1000.0
            || pos.y > screen.y + 1000.0
        {
            continue;
        }
        let (col, row) = b.frame_cell();
        let frame_w = rect.width / 3;
        let frame_h = rect.height / 3;
        let uv = [
            (rect.x + col * frame_w) as f32 / aw,
            (rect.y + row * frame_h) as f32 / ah,
            (rect.x + (col + 1) * frame_w) as f32 / aw,
            (rect.y + (row + 1) * frame_h) as f32 / ah,
        ];
        // `scale: vector2.X * 2`, `origin: Vector2.Zero` (top-left anchor).
        // The back range carries the world transform (no zoom in the size).
        let scale = par.x * 2.0;
        let size = [frame_w as f32 * scale, frame_h as f32 * scale];
        let anchor_world = top_left + pos / zoom;
        r.push_back_sprite_origin_rot_blend(
            anchor_world.to_array(),
            size,
            uv,
            color,
            0.0,
            [0.0, 0.0],
            tr_render::BlendMode::AlphaBlend,
        );
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> SkyCtx {
        SkyCtx {
            screen_position_y: 1000.0,
            screen_height: 720.0,
            max_tiles_x: 4200,
            max_tiles_y: 1200,
            world_surface: 300.0,
        }
    }

    /// W4-c39 — `LanternSky.ResetLantern`'s depth ladder (`:105`): slot 0 is
    /// the farthest (6.0), the last the nearest (1.6); the float timer is
    /// `(2000 + Next(1200)) * 2` (4000..6398).
    #[test]
    fn w4_c39_the_lantern_depth_ladder_runs_far_to_near() {
        let mut skies = EventSkies::new(42);
        skies.activate_lanterns(&ctx());
        let n = skies.lanterns.len();
        assert_eq!(n, 300, "maxTilesY / 4 = 1200 / 4");
        assert!((skies.lanterns[0].depth - 6.0).abs() < 1e-4);
        assert!((skies.lanterns[n - 1].depth - 1.6).abs() < 0.02);
        for l in skies.lanterns.iter() {
            assert!(l.active);
            assert!(l.speed <= -1.5 && l.speed >= -4.0, "speed {}", l.speed);
            assert!(l.variant < 3);
            assert!(
                l.time_until_float >= 4000 && l.time_until_float < 6400,
                "timer {}",
                l.time_until_float
            );
            assert_eq!(l.time_until_float, l.time_until_float_max);
        }
    }

    /// W4-c39 — the lantern opacity ramp (`:119`): +0.01/tick while up,
    /// −0.01 while down, clamped 0..1.
    #[test]
    fn w4_c39_the_lantern_opacity_ramps_with_the_event() {
        let mut skies = EventSkies::new(1);
        skies.activate_lanterns(&ctx());
        for _ in 0..150 {
            skies.update_lanterns(true, 0.0, &ctx());
        }
        assert!((skies.lantern_opacity - 1.0).abs() < 1e-4, "ramps to 1");
        for _ in 0..150 {
            skies.update_lanterns(false, 0.0, &ctx());
        }
        assert!(skies.lantern_opacity <= 0.0 + 1e-4, "ramps back to 0");
    }

    /// W4-c39 — `PartySky.ResetBalloon`'s depth ladder (`:115`): slot 0 the
    /// nearest (1.6), the last the farthest (3.35); the rare 1/30 third sheet.
    #[test]
    fn w4_c39_the_balloon_depth_ladder_runs_near_to_far() {
        let mut skies = EventSkies::new(7);
        skies.activate_party(&ctx());
        let n = skies.balloons.len();
        assert_eq!(n, 300);
        assert!((skies.balloons[0].depth - 1.6).abs() < 1e-4);
        assert!((skies.balloons[n - 1].depth - 3.35).abs() < 0.02);
        let mut rare = 0;
        for b in skies.balloons.iter() {
            assert!(b.active);
            assert!(b.texture < 3);
            assert!(b.variant < 3);
            if b.texture == 2 {
                rare += 1;
            }
        }
        // The 1/30 roll over 300 balloons lands a handful of rare sheets.
        assert!(rare > 0, "the 1/30 rare sheet fires");
    }

    /// W4-c39 — the balloon frame counter wraps at 42 (`:59`), the source row
    /// is `counter / 14` (3 rows).
    #[test]
    fn w4_c39_the_balloon_frame_wraps_at_forty_two() {
        let mut b = Balloon {
            active: true,
            position: Vec2::new(0.0, 1000.0),
            depth: 2.0,
            speed: -2.0,
            texture: 0,
            variant: 1,
            frame_counter: 41,
        };
        b.frame_counter = (b.frame_counter + 1) % 42;
        assert_eq!(b.frame_counter, 0, "42 wraps to 0");
        assert_eq!(b.frame_cell(), (1, 0));
        b.frame_counter = 27;
        assert_eq!(b.frame_cell(), (1, 1), "27 / 14 = row 1");
        b.frame_counter = 41;
        assert_eq!(b.frame_cell(), (1, 2), "41 / 14 = row 2");
    }

    /// W4-c39 — the party opacity ramp (`SceneState.cs:106`): toward 1 with a
    /// town NPC present, toward 0 without, 0.01/tick.
    #[test]
    fn w4_c39_the_party_opacity_ramps_with_the_town() {
        let mut skies = EventSkies::new(3);
        for _ in 0..120 {
            skies.ramp_party_opacity(true);
        }
        assert!((skies.party_opacity - 1.0).abs() < 1e-4);
        for _ in 0..120 {
            skies.ramp_party_opacity(false);
        }
        assert!(skies.party_opacity <= 1e-4);
    }
}
