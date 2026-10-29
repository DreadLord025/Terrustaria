//! `Terraria.WaterfallManager` port: finding, animating and drawing waterfalls.
//!
//! The manager keeps the same 30-frame rescan cadence, the same 8.5 Hz / 20 Hz
//! frame counters and the same per-step geometry walk as the original. The app
//! turns the returned [`WaterfallDraw`] records into sprites; lighting, the
//! shimmer vertex-colour path and dust sparkles are applied there.

use crate::lighting::{LightMap, LightStyle};
use crate::tile::Tile;
use crate::tiledata as td;
use crate::world::World;

/// `WaterfallManager.maxTypes`: `Images/Waterfall_0..27`.
pub const MAX_TYPES: usize = 28;

/// `WaterfallManager.maxWaterfallCountDefault`.
const MAX_WATERFALL_COUNT_DEFAULT: usize = 1000;

/// `Main.gfxQuality`: the original adapts this to the frame rate. A fixed 1.0
/// uses the full waterfall distance (100 tiles) and count (1000), which is what
/// a machine holding 60 FPS converges to.
const GFX_QUALITY: f32 = 1.0;

/// `TileID.Sets.BlocksWaterDrawingBehindSelf`.
fn blocks_water_drawing_behind_self(kind: u16) -> bool {
    matches!(kind, 54 | 328 | 459 | 470 | 541)
}

#[derive(Clone, Copy, Default, Debug)]
struct Waterfall {
    x: i32,
    y: i32,
    kind: i32,
    stop_at_step: i32,
}

/// One textured waterfall quad, in world pixels.
#[derive(Clone, Copy, Debug)]
pub struct WaterfallDraw {
    /// `Images/Waterfall_{texture}` index.
    pub texture: u8,
    /// Top-left position in world pixels.
    pub pos: [f32; 2],
    /// Source rectangle inside the 512x40 / 144x18 sheet.
    pub source: [i32; 4],
    /// `SpriteEffects.FlipHorizontally`.
    pub flip: bool,
    /// Premultiplied-by-opacity tint, matching `Color * alpha`.
    pub color: [f32; 4],
}

/// `WaterfallManager`.
pub struct WaterfallManager {
    waterfalls: Vec<Waterfall>,
    quality_max: usize,
    current_max: usize,
    waterfall_dist: i32,
    find_waterfall_count: i32,
    /// Bumped when a rescan changes the list, for the G6 world-range cache.
    list_revision: u64,

    w_fall_fr_counter: i32,
    regular_frame: i32,
    w_fall_fr_counter2: i32,
    slow_frame: i32,
    rain_frame_counter: i32,
    rain_frame_foreground: i32,
    rain_frame_background: i32,
    lava_rain_frame_counter: i32,
    lava_rain_frame_foreground: i32,
    lava_rain_frame_background: i32,
    snow_frame_counter: i32,
    snow_frame_foreground: i32,

    /// `Main.DiscoR/G/B/Style` for the disco waterfall (tile 160).
    disco_r: i32,
    disco_g: i32,
    disco_b: i32,
    disco_style: i32,
}

impl Default for WaterfallManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WaterfallManager {
    pub fn new() -> Self {
        Self {
            waterfalls: vec![Waterfall::default(); MAX_WATERFALL_COUNT_DEFAULT],
            quality_max: MAX_WATERFALL_COUNT_DEFAULT,
            current_max: 0,
            waterfall_dist: (75.0 * GFX_QUALITY) as i32 + 25,
            // The original increments before testing `< 30`; starting at 29 makes
            // the first update perform the initial scan (the load-time forced scan).
            find_waterfall_count: 29,
            list_revision: 0,
            w_fall_fr_counter: 0,
            regular_frame: 0,
            w_fall_fr_counter2: 0,
            slow_frame: 0,
            rain_frame_counter: 0,
            rain_frame_foreground: 0,
            rain_frame_background: 0,
            lava_rain_frame_counter: 0,
            lava_rain_frame_foreground: 0,
            lava_rain_frame_background: 0,
            snow_frame_counter: 0,
            snow_frame_foreground: 0,
            disco_r: 0,
            disco_g: 0,
            disco_b: 0,
            disco_style: 0,
        }
    }

    pub fn count(&self) -> usize {
        self.current_max
    }

    /// `Main.DiscoR/G/B`: the rainbow cycle, shared with the lighting scanner
    /// (disco torches, gemspark, painted lamps) so both read the same phase.
    pub fn disco_rgb(&self) -> [u8; 3] {
        [
            self.disco_r.clamp(0, 255) as u8,
            self.disco_g.clamp(0, 255) as u8,
            self.disco_b.clamp(0, 255) as u8,
        ]
    }

    /// `WaterfallManager.CheckForWaterfall`.
    pub fn check_for_waterfall(&self, x: i32, y: i32) -> bool {
        self.waterfalls[..self.current_max]
            .iter()
            .any(|w| w.x == x && w.y == y)
    }

    /// Cache key for the animated quads. Zero when no waterfall is visible, so a
    /// quiet world does not invalidate the G6 range on every animation frame.
    pub fn anim_key(&self) -> u64 {
        if self.current_max == 0 {
            return 0;
        }
        (self.regular_frame as u64)
            | (self.slow_frame as u64) << 4
            | (self.rain_frame_foreground as u64) << 8
            | (self.rain_frame_background as u64) << 12
            | (self.lava_rain_frame_foreground as u64) << 16
            | (self.lava_rain_frame_background as u64) << 20
            | (self.snow_frame_foreground as u64) << 24
            | self.list_revision << 32
    }

    /// `WaterfallManager.UpdateFrame` + `DoUpdate_AnimateDiscoRGB` — both advance
    /// once per 60 Hz tick. `Main.mouseTextColor` (the light flicker) lives in
    /// `crate::lighting`, and `AddLight` for falls is a documented follow-up.
    pub fn update_tick(&mut self) {
        self.w_fall_fr_counter += 1;
        if self.w_fall_fr_counter > 2 {
            self.w_fall_fr_counter = 0;
            self.regular_frame += 1;
            if self.regular_frame > 15 {
                self.regular_frame = 0;
            }
        }

        self.w_fall_fr_counter2 += 1;
        if self.w_fall_fr_counter2 > 6 {
            self.w_fall_fr_counter2 = 0;
            self.slow_frame += 1;
            if self.slow_frame > 15 {
                self.slow_frame = 0;
            }
        }

        self.rain_frame_counter += 1;
        if self.rain_frame_counter > 0 {
            self.rain_frame_foreground += 1;
            if self.rain_frame_foreground > 7 {
                self.rain_frame_foreground -= 8;
            }
            if self.rain_frame_counter > 2 {
                self.rain_frame_counter = 0;
                self.rain_frame_background -= 1;
                if self.rain_frame_background < 0 {
                    self.rain_frame_background = 7;
                }
            }
        }

        self.lava_rain_frame_counter += 1;
        if self.lava_rain_frame_counter == 1 || self.lava_rain_frame_counter == 3 {
            self.lava_rain_frame_foreground += 1;
            if self.lava_rain_frame_foreground > 7 {
                self.lava_rain_frame_foreground -= 8;
            }
        } else if self.lava_rain_frame_counter > 3 {
            self.lava_rain_frame_counter = 0;
            self.lava_rain_frame_background -= 1;
            if self.lava_rain_frame_background < 0 {
                self.lava_rain_frame_background = 7;
            }
        }

        self.snow_frame_counter += 1;
        if self.snow_frame_counter > 3 {
            self.snow_frame_counter = 0;
            self.snow_frame_foreground += 1;
            if self.snow_frame_foreground > 7 {
                self.snow_frame_foreground = 0;
            }
        }

        self.update_disco_rgb();
    }

    fn update_disco_rgb(&mut self) {
        let step = 7;
        if self.disco_style == 0 {
            self.disco_g += step;
            if self.disco_g >= 255 {
                self.disco_g = 255;
                self.disco_style += 1;
            }
        }
        if self.disco_style == 1 {
            self.disco_r -= step;
            if self.disco_r <= 0 {
                self.disco_r = 0;
                self.disco_style += 1;
            }
        }
        if self.disco_style == 2 {
            self.disco_b += step;
            if self.disco_b >= 255 {
                self.disco_b = 255;
                self.disco_style += 1;
            }
        }
        if self.disco_style == 3 {
            self.disco_g -= step;
            if self.disco_g <= 0 {
                self.disco_g = 0;
                self.disco_style += 1;
            }
        }
        if self.disco_style == 4 {
            self.disco_r += step;
            if self.disco_r >= 255 {
                self.disco_r = 255;
                self.disco_style += 1;
            }
        }
        if self.disco_style == 5 {
            self.disco_b -= step;
            if self.disco_b <= 0 {
                self.disco_b = 0;
                self.disco_style = 0;
            }
        }
    }

    /// `WaterfallManager.FindWaterfalls`.
    ///
    /// `view` is the visible tile rectangle `[x0, y0, x1, y1]` (inclusive). The
    /// original works from `Main.screenPosition` / screen size; the caller derives
    /// the same rectangle from the camera.
    pub fn find_waterfalls(&mut self, world: &World, view: [i32; 4], forced: bool) {
        self.find_waterfall_count += 1;
        if self.find_waterfall_count < 30 && !forced {
            return;
        }
        self.find_waterfall_count = 0;

        self.waterfall_dist = (75.0 * GFX_QUALITY) as i32 + 25;
        self.quality_max = (MAX_WATERFALL_COUNT_DEFAULT as f32 * GFX_QUALITY) as usize;
        let mut current = 0usize;
        let mut next: Vec<Waterfall> = Vec::with_capacity(self.quality_max.min(64));

        let mut x0 = view[0] - 1 - self.waterfall_dist;
        let mut x1 = view[2] + 2 + self.waterfall_dist;
        let mut y0 = view[1] - 1 - self.waterfall_dist;
        let mut y1 = view[3] + 2 + 20;
        if x0 <= 0 {
            x0 = 1;
        }
        if x1 >= world.width {
            x1 = world.width - 1;
        }
        if y0 <= 0 {
            y0 = 1;
        }
        if y1 >= world.height {
            y1 = world.height - 1;
        }

        for i in x0..x1 {
            for j in y0..y1 {
                let Some(tile) = world.tile(i, j) else {
                    continue;
                };
                if !tile.active() {
                    continue;
                }
                if tile.half_brick() {
                    let above = world.tile(i, j - 1).copied().unwrap_or_default();
                    if above.liquid < 16 || solid_tile(&above) {
                        let left = world.tile(i - 1, j).copied().unwrap_or_default();
                        let right = world.tile(i + 1, j).copied().unwrap_or_default();
                        if (left.liquid > 160 || right.liquid > 160)
                            && ((left.liquid == 0 && !solid_tile(&left) && left.slope() == 0)
                                || (right.liquid == 0 && !solid_tile(&right) && right.slope() == 0))
                            && current < self.quality_max
                        {
                            let mut kind = 0;
                            if above.liquid_kind_is_lava(&left, &right) {
                                kind = 1;
                            } else if above.liquid_kind_is_honey(&left, &right) {
                                kind = 14;
                            } else if above.liquid_kind_is_shimmer(&left, &right) {
                                kind = 25;
                            }
                            push_waterfall(&mut next, i, j, kind);
                            current += 1;
                        }
                    }
                }
                if tile.kind == 196 {
                    let below = world.tile(i, j + 1).copied().unwrap_or_default();
                    if !solid_tile(&below)
                        && below.liquid == 0
                        && below.slope() == 0
                        && current < self.quality_max
                    {
                        push_waterfall(&mut next, i, j + 1, 11);
                        current += 1;
                    }
                }
                if tile.kind == 460 {
                    let below = world.tile(i, j + 1).copied().unwrap_or_default();
                    if !solid_tile(&below)
                        && below.liquid == 0
                        && below.slope() == 0
                        && current < self.quality_max
                    {
                        push_waterfall(&mut next, i, j + 1, 22);
                        current += 1;
                    }
                }
                if tile.kind == 717 {
                    let below = world.tile(i, j + 1).copied().unwrap_or_default();
                    if !solid_tile(&below)
                        && below.liquid == 0
                        && below.slope() == 0
                        && current < self.quality_max
                    {
                        push_waterfall(&mut next, i, j + 1, 26);
                        current += 1;
                    }
                }
            }
        }

        // Preserve the original scan order and only invalidate the world range
        // when the visible set actually changed.
        let changed = current != self.current_max
            || self.waterfalls[..current]
                .iter()
                .zip(next.iter())
                .any(|(a, b)| a.x != b.x || a.y != b.y || a.kind != b.kind);
        self.waterfalls[..current].copy_from_slice(&next[..current]);
        self.current_max = current;
        if changed {
            self.list_revision = self.list_revision.wrapping_add(1);
        }
    }

    /// `WaterfallManager.Draw`: resets `stopAtStep` and runs the single water-style
    /// pass our worlds use (`DrawWaterfall(0, liquidAlpha[0] = 1)`).
    pub fn draw(
        &mut self,
        world: &World,
        light: &LightMap,
        style: &LightStyle,
        time: f32,
        view: [i32; 4],
        out: &mut Vec<WaterfallDraw>,
    ) {
        for w in &mut self.waterfalls[..self.current_max] {
            w.stop_at_step = self.waterfall_dist;
        }
        self.draw_waterfall(0, 1.0, world, light, style, time, view, out);
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_waterfall(
        &mut self,
        draw_style: i32,
        draw_alpha: f32,
        world: &World,
        light: &LightMap,
        style: &LightStyle,
        time: f32,
        view: [i32; 4],
        out: &mut Vec<WaterfallDraw>,
    ) {
        for index in 0..self.current_max {
            let mut wf = self.waterfalls[index];
            let mut num13 = wf.x;
            let mut num14 = wf.y;
            // `DrawWaterfall` declares these inside the per-waterfall loop.
            let mut num11 = 0;
            let mut num15 = 0;
            let mut num16 = 0;
            let mut num17 = 0;
            let mut num18 = 0;
            let mut num19 = 0;
            let mut num20 = 0;
            let mut color4 = [1.0f32, 1.0, 1.0, 1.0];

            let mut num12 = wf.kind;
            let num21;
            if num12 == 1 || num12 == 14 || num12 == 25 {
                // `Main.drewLava` is always false in our draw order.
                if wf.stop_at_step == 0 {
                    continue;
                }
                num21 = 32 * self.slow_frame;
            } else if num12 == 11 || num12 == 22 || num12 == 26 {
                let mut num22 = self.waterfall_dist / 4;
                if num12 == 22 {
                    num22 = self.waterfall_dist / 2;
                }
                if wf.stop_at_step > num22 {
                    wf.stop_at_step = num22;
                }
                if wf.stop_at_step == 0
                    || (num14 + num22) < view[1]
                    || num13 < view[0] - 20
                    || num13 > view[2] + 20
                {
                    continue;
                }
                let mut num23;
                let mut num24 = 0;
                if num13 % 2 == 0 {
                    match num12 {
                        22 => {
                            num23 = self.snow_frame_foreground + 3;
                            if num23 > 7 {
                                num23 -= 8;
                            }
                        }
                        26 => {
                            num23 = self.lava_rain_frame_foreground + 3;
                            if num23 > 7 {
                                num23 -= 8;
                            }
                            num24 = self.lava_rain_frame_background + 2;
                            if num24 > 7 {
                                num24 -= 8;
                            }
                        }
                        _ => {
                            num23 = self.rain_frame_foreground + 3;
                            if num23 > 7 {
                                num23 -= 8;
                            }
                            num24 = self.rain_frame_background + 2;
                            if num24 > 7 {
                                num24 -= 8;
                            }
                        }
                    }
                } else {
                    match num12 {
                        22 => num23 = self.snow_frame_foreground,
                        26 => {
                            num23 = self.lava_rain_frame_foreground;
                            num24 = self.lava_rain_frame_background;
                        }
                        _ => {
                            num23 = self.rain_frame_foreground;
                            num24 = self.rain_frame_background;
                        }
                    }
                }
                let mut source = [num24 * 18, 0, 16, 16];
                let mut source2 = [num23 * 18, 0, 16, 16];
                let mut position = if num14 % 2 != 0 {
                    [num13 as f32 * 16.0, num14 as f32 * 16.0]
                } else {
                    [num13 as f32 * 16.0 + 1.0, num14 as f32 * 16.0]
                };
                if !in_world(world, num13, num14 - 1) {
                    continue;
                }
                let above = world.tile(num13, num14 - 1).copied().unwrap_or_default();
                if above.active() && above.bottom_slope() {
                    position[1] -= 16.0;
                }
                let mut flag = false;
                for j in 0..num22 {
                    let mut color = light_tint(light, style, num13, num14, time);
                    let mut num25 = 0.6f32;
                    let mut num26 = 0.3f32;
                    if num12 == 26 {
                        color = [1.0, 1.0, 1.0, 127.0 / 255.0];
                        num25 = 0.9;
                        num26 = 0.4;
                    }
                    if j > num22 - 8 {
                        let num27 = (num22 - j) as f32 / 8.0;
                        num25 *= num27;
                        num26 *= num27;
                    }
                    let color2 = scale_color(color, num25);
                    let color3 = scale_color(color, num26);
                    match num12 {
                        22 => push(out, 22, position, source2, color2, false),
                        26 => {
                            push(out, 27, position, source, color3, false);
                            push(out, 26, position, source2, color2, false);
                        }
                        _ => {
                            push(out, 12, position, source, color3, false);
                            push(out, 11, position, source2, color2, false);
                        }
                    }
                    if flag {
                        break;
                    }
                    num14 += 1;
                    if num14 >= world.height {
                        break;
                    }
                    let below = world.tile(num13, num14).copied().unwrap_or_default();
                    if solid_tile_draw(&below) {
                        flag = true;
                    }
                    if below.liquid > 0 {
                        let num28 = (16.0 * (below.liquid as f32 / 255.0)) as i32 & 0xfe;
                        if num28 >= 15 {
                            break;
                        }
                        source2[3] -= num28;
                        source[3] -= num28;
                    }
                    if num14 % 2 == 0 {
                        position[0] += 1.0;
                    } else {
                        position[0] -= 1.0;
                    }
                    position[1] += 16.0;
                }
                self.waterfalls[index].stop_at_step = 0;
                continue;
            } else {
                match num12 {
                    0 => num12 = draw_style,
                    2 => {}
                    _ => {}
                }
                num21 = 32 * self.regular_frame;
            }

            let mut num29 = 0;
            let mut num22 = self.waterfall_dist;
            let mut k = 0;
            while k < num22 {
                if num29 >= 2 {
                    break;
                }
                let tile3 = world.tile(num13, num14).copied().unwrap_or_default();
                if tile3.active()
                    && solid_tile_draw(&tile3)
                    && !td::TILE_SOLID_TOP.has(tile3.kind)
                    && !td::SET_PLATFORMS.has(tile3.kind)
                    && !tile3.half_brick()
                    && tile3.slope() == 0
                {
                    break;
                }
                let tile4 = world.tile(num13 - 1, num14).copied().unwrap_or_default();
                let tile5 = world.tile(num13, num14 + 1).copied().unwrap_or_default();
                let tile6 = world.tile(num13 + 1, num14).copied().unwrap_or_default();
                if solid_tile_draw(&tile5) && !tile3.half_brick() {
                    num11 = 8;
                } else if num16 != 0 {
                    num11 = 0;
                }
                let mut num30 = 0;
                let num31 = num18;
                let mut num32;
                let num33;
                let mut flag2 = false;
                if tile5.top_slope() && !tile3.half_brick() && tile5.kind != 19 {
                    flag2 = true;
                    if tile5.slope() == 1 {
                        num30 = 1;
                        num32 = 1;
                        num17 = 1;
                        num18 = num17;
                    } else {
                        num30 = -1;
                        num32 = -1;
                        num17 = -1;
                        num18 = num17;
                    }
                    num33 = 1;
                } else if (!solid_tile_draw(&tile5) && !tile5.bottom_slope() && !tile3.half_brick())
                    || (!tile5.active() && !tile3.half_brick())
                {
                    num29 = 0;
                    num33 = 1;
                    num32 = 0;
                } else if (solid_tile_draw(&tile4) || tile4.top_slope() || tile4.liquid > 0)
                    && !solid_tile_draw(&tile6)
                    && tile6.liquid == 0
                {
                    if num17 == -1 {
                        num29 += 1;
                    }
                    num32 = 1;
                    num33 = 0;
                    num17 = 1;
                } else if (solid_tile_draw(&tile6) || tile6.top_slope() || tile6.liquid > 0)
                    && !solid_tile_draw(&tile4)
                    && tile4.liquid == 0
                {
                    if num17 == 1 {
                        num29 += 1;
                    }
                    num32 = -1;
                    num33 = 0;
                    num17 = -1;
                } else if ((!solid_tile_draw(&tile6) && !tile3.top_slope()) || tile6.liquid == 0)
                    && !solid_tile_draw(&tile4)
                    && !tile3.top_slope()
                    && tile4.liquid == 0
                {
                    num33 = 0;
                    num32 = num17;
                } else {
                    num29 += 1;
                    num33 = 0;
                    num32 = 0;
                }
                if num29 >= 2 {
                    num17 *= -1;
                    num32 *= -1;
                }

                let mut num34 = -1i32;
                if num12 != 1 && num12 != 14 && num12 != 25 {
                    if tile5.active() {
                        num34 = tile5.kind as i32;
                    }
                    if tile3.active() {
                        num34 = tile3.kind as i32;
                    }
                }
                match num34 {
                    160 => num12 = 2,
                    262..=268 => num12 = 15 + num34 - 262,
                    _ => {}
                }

                let alpha = get_alpha(draw_alpha, num22, num12, num14, k, &tile3, world);
                let color5 = stylize_color(
                    alpha,
                    num12,
                    light_tint(light, style, num13, num14, time),
                    self.disco_r,
                    self.disco_g,
                    self.disco_b,
                );

                let num41 = tile3.liquid as i32 / 16;
                if flag2 && num17 != num31 {
                    let num42 = 2;
                    let src = [num21, 24, 32, 16 - num41 - num42];
                    if num31 == 1 {
                        push(
                            out,
                            num12 as u8,
                            [
                                num13 as f32 * 16.0 - 16.0,
                                num14 as f32 * 16.0 + 16.0 - num42 as f32,
                            ],
                            src,
                            color5,
                            true,
                        );
                    } else {
                        push(
                            out,
                            num12 as u8,
                            [
                                num13 as f32 * 16.0,
                                num14 as f32 * 16.0 + 16.0 - num42 as f32,
                            ],
                            src,
                            color5,
                            false,
                        );
                    }
                }
                if num15 == 0 && num30 != 0 && num16 == 1 && num17 != num18 {
                    num30 = 0;
                    num17 = num18;
                    let color = [1.0, 1.0, 1.0, 1.0];
                    let src = [num21, 24, 32, 16 - num41];
                    push(
                        out,
                        num12 as u8,
                        [num13 as f32 * 16.0 - 16.0, num14 as f32 * 16.0 + 16.0],
                        src,
                        color,
                        true,
                    );
                }
                if num19 != 0 && num32 == 0 && num33 == 1 {
                    if num17 == 1 {
                        let color = if num20 != num12 { color4 } else { color5 };
                        push(
                            out,
                            num12 as u8,
                            [
                                num13 as f32 * 16.0,
                                num14 as f32 * 16.0 + num11 as f32 + 8.0,
                            ],
                            [num21, 0, 16, 16 - num41 - 8],
                            color,
                            true,
                        );
                    } else {
                        push(
                            out,
                            num12 as u8,
                            [
                                num13 as f32 * 16.0,
                                num14 as f32 * 16.0 + num11 as f32 + 8.0,
                            ],
                            [num21, 0, 16, 16 - num41 - 8],
                            color5,
                            false,
                        );
                    }
                }
                if num11 == 8 && num16 == 1 && num19 == 0 {
                    if num18 == -1 {
                        if num20 != num12 {
                            push(
                                out,
                                num20 as u8,
                                [num13 as f32 * 16.0, num14 as f32 * 16.0],
                                [num21, 24, 32, 8],
                                color4,
                                false,
                            );
                        } else {
                            push(
                                out,
                                num12 as u8,
                                [num13 as f32 * 16.0, num14 as f32 * 16.0],
                                [num21, 24, 32, 8],
                                color5,
                                false,
                            );
                        }
                    } else if num20 != num12 {
                        push(
                            out,
                            num20 as u8,
                            [num13 as f32 * 16.0 - 16.0, num14 as f32 * 16.0],
                            [num21, 24, 32, 8],
                            color4,
                            true,
                        );
                    } else {
                        push(
                            out,
                            num12 as u8,
                            [num13 as f32 * 16.0 - 16.0, num14 as f32 * 16.0],
                            [num21, 24, 32, 8],
                            color5,
                            true,
                        );
                    }
                }
                if num30 != 0 && num15 == 0 {
                    if num31 == 1 {
                        if num20 != num12 {
                            push(
                                out,
                                num20 as u8,
                                [num13 as f32 * 16.0 - 16.0, num14 as f32 * 16.0],
                                [num21, 24, 32, 16 - num41],
                                color4,
                                true,
                            );
                        } else {
                            push(
                                out,
                                num12 as u8,
                                [num13 as f32 * 16.0 - 16.0, num14 as f32 * 16.0],
                                [num21, 24, 32, 16 - num41],
                                color5,
                                true,
                            );
                        }
                    } else if num20 != num12 {
                        push(
                            out,
                            num20 as u8,
                            [num13 as f32 * 16.0, num14 as f32 * 16.0],
                            [num21, 24, 32, 16 - num41],
                            color4,
                            false,
                        );
                    } else {
                        push(
                            out,
                            num12 as u8,
                            [num13 as f32 * 16.0, num14 as f32 * 16.0],
                            [num21, 24, 32, 16 - num41],
                            color5,
                            false,
                        );
                    }
                }
                if num33 == 1 && num30 == 0 && num19 == 0 {
                    if num17 == -1 {
                        if num16 == 0 {
                            push(
                                out,
                                num12 as u8,
                                [num13 as f32 * 16.0, num14 as f32 * 16.0 + num11 as f32],
                                [num21, 0, 16, 16 - num41],
                                color5,
                                false,
                            );
                        } else if num20 != num12 {
                            push(
                                out,
                                num20 as u8,
                                [num13 as f32 * 16.0, num14 as f32 * 16.0],
                                [num21, 24, 32, 16 - num41],
                                color4,
                                false,
                            );
                        } else {
                            push(
                                out,
                                num12 as u8,
                                [num13 as f32 * 16.0, num14 as f32 * 16.0],
                                [num21, 24, 32, 16 - num41],
                                color5,
                                false,
                            );
                        }
                    } else if num16 == 0 {
                        push(
                            out,
                            num12 as u8,
                            [num13 as f32 * 16.0, num14 as f32 * 16.0 + num11 as f32],
                            [num21, 0, 16, 16 - num41],
                            color5,
                            true,
                        );
                    } else if num20 != num12 {
                        push(
                            out,
                            num20 as u8,
                            [num13 as f32 * 16.0 - 16.0, num14 as f32 * 16.0],
                            [num21, 24, 32, 16 - num41],
                            color4,
                            true,
                        );
                    } else {
                        push(
                            out,
                            num12 as u8,
                            [num13 as f32 * 16.0 - 16.0, num14 as f32 * 16.0],
                            [num21, 24, 32, 16 - num41],
                            color5,
                            true,
                        );
                    }
                } else {
                    match num32 {
                        1 => {
                            if !(tile3.liquid > 0 && !tile3.half_brick()) {
                                if num30 == 1 {
                                    for m in 0..8 {
                                        let num46 = m * 2;
                                        let num47 = 14 - m * 2;
                                        let mut num48 = num46;
                                        num11 = 8;
                                        if num15 == 0 && m < 2 {
                                            num48 = 4;
                                        }
                                        push(
                                            out,
                                            num12 as u8,
                                            [
                                                num13 as f32 * 16.0 + num46 as f32,
                                                num14 as f32 * 16.0 + num11 as f32 + num48 as f32,
                                            ],
                                            [16 + num21 + num47, 0, 2, 16 - num11],
                                            color5,
                                            true,
                                        );
                                    }
                                } else {
                                    let mut height = 16;
                                    if blocks_water_drawing_behind_self(tile3.kind) {
                                        height = 8;
                                    } else if blocks_water_drawing_behind_self(tile5.kind) {
                                        height = 8;
                                    }
                                    push(
                                        out,
                                        num12 as u8,
                                        [num13 as f32 * 16.0, num14 as f32 * 16.0 + num11 as f32],
                                        [16 + num21, 0, 16, height],
                                        color5,
                                        true,
                                    );
                                }
                            }
                        }
                        -1 => {
                            if !(tile3.liquid > 0 && !tile3.half_brick()) {
                                if num30 == -1 {
                                    for l in 0..8 {
                                        let num43 = l * 2;
                                        let num44 = l * 2;
                                        let mut num45 = 14 - l * 2;
                                        num11 = 8;
                                        if num15 == 0 && l > 5 {
                                            num45 = 4;
                                        }
                                        push(
                                            out,
                                            num12 as u8,
                                            [
                                                num13 as f32 * 16.0 + num43 as f32,
                                                num14 as f32 * 16.0 + num11 as f32 + num45 as f32,
                                            ],
                                            [16 + num21 + num44, 0, 2, 16 - num11],
                                            color5,
                                            true,
                                        );
                                    }
                                } else {
                                    let mut height = 16;
                                    if blocks_water_drawing_behind_self(tile3.kind) {
                                        height = 8;
                                    } else if blocks_water_drawing_behind_self(tile5.kind) {
                                        height = 8;
                                    }
                                    push(
                                        out,
                                        num12 as u8,
                                        [num13 as f32 * 16.0, num14 as f32 * 16.0 + num11 as f32],
                                        [16 + num21, 0, 16, height],
                                        color5,
                                        false,
                                    );
                                }
                            }
                        }
                        _ => {
                            if num33 == 0 {
                                if tile3.liquid <= 0 || tile3.half_brick() {
                                    push(
                                        out,
                                        num12 as u8,
                                        [num13 as f32 * 16.0, num14 as f32 * 16.0 + num11 as f32],
                                        [16 + num21, 0, 16, 16],
                                        color5,
                                        false,
                                    );
                                }
                                k = 1000;
                            }
                        }
                    }
                }
                if tile3.liquid > 0 && !tile3.half_brick() {
                    k = 1000;
                }
                num16 = num33;
                num18 = num17;
                num15 = num32;
                num13 += num32;
                num14 += num33;
                num19 = num30;
                color4 = color5;
                if num20 != num12 {
                    num20 = num12;
                }
                if (tile4.active() && (tile4.kind == 189 || tile4.kind == 196))
                    || (tile6.active() && (tile6.kind == 189 || tile6.kind == 196))
                    || (tile5.active() && (tile5.kind == 189 || tile5.kind == 196))
                {
                    num22 = (40.0 * (world.width as f32 / 4200.0) * GFX_QUALITY) as i32;
                }
                if !in_world(world, num13, num14) {
                    break;
                }
                k += 1;
            }
        }
    }

    /// Test helper: current waterfall list `(x, y, type)`.
    #[cfg(test)]
    fn entries(&self) -> Vec<(i32, i32, i32)> {
        self.waterfalls[..self.current_max]
            .iter()
            .map(|w| (w.x, w.y, w.kind))
            .collect()
    }
}

fn push_waterfall(list: &mut Vec<Waterfall>, x: i32, y: i32, kind: i32) {
    list.push(Waterfall {
        x,
        y,
        kind,
        stop_at_step: 0,
    });
}

fn scale_color(c: [f32; 4], scale: f32) -> [f32; 4] {
    [
        (c[0] * 255.0 * scale) as i32 as f32 / 255.0,
        (c[1] * 255.0 * scale) as i32 as f32 / 255.0,
        (c[2] * 255.0 * scale) as i32 as f32 / 255.0,
        ((c[3] * 255.0 * scale) as i32).clamp(0, 255) as f32 / 255.0,
    ]
}

/// `Lighting.GetColor` through the active light style, as byte-normalised colour.
fn light_tint(light: &LightMap, style: &LightStyle, x: i32, y: i32, time: f32) -> [f32; 4] {
    let c = style.shade(light.get(x, y), x, y, time);
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0) as i32;
    [
        byte(c[0]) as f32 / 255.0,
        byte(c[1]) as f32 / 255.0,
        byte(c[2]) as f32 / 255.0,
        1.0,
    ]
}

/// `WaterfallManager.StylizeColor`.
fn stylize_color(
    alpha: f32,
    kind: i32,
    color: [f32; 4],
    disco_r: i32,
    disco_g: i32,
    disco_b: i32,
) -> [f32; 4] {
    let mut num = color[0] * 255.0 * alpha;
    let mut num2 = color[1] * 255.0 * alpha;
    let mut num3 = color[2] * 255.0 * alpha;
    let num4 = color[3] * 255.0 * alpha;
    match kind {
        1 => {
            num = num.max(190.0 * alpha);
            num2 = num2.max(190.0 * alpha);
            num3 = num3.max(190.0 * alpha);
        }
        2 => {
            num = disco_r as f32 * alpha;
            num2 = disco_g as f32 * alpha;
            num3 = disco_b as f32 * alpha;
        }
        15..=21 => {
            num = 255.0 * alpha;
            num2 = 255.0 * alpha;
            num3 = 255.0 * alpha;
        }
        _ => {}
    }
    [
        num.clamp(0.0, 255.0) as i32 as f32 / 255.0,
        num2.clamp(0.0, 255.0) as i32 as f32 / 255.0,
        num3.clamp(0.0, 255.0) as i32 as f32 / 255.0,
        num4.clamp(0.0, 255.0) as i32 as f32 / 255.0,
    ]
}

/// `WaterfallManager.GetAlpha`.
fn get_alpha(
    alpha: f32,
    max_steps: i32,
    kind: i32,
    y: i32,
    s: i32,
    tile: &Tile,
    world: &World,
) -> f32 {
    let mut num = match kind {
        1 => 1.0,
        14 => 0.8,
        25 => 0.75,
        _ => {
            if (tile.wall != 0 && tile.wall != 318) || (y as f64) >= world.world_surface {
                0.6 * alpha
            } else {
                alpha
            }
        }
    };
    if s > max_steps - 10 {
        num *= (max_steps - s) as f32 / 10.0;
    }
    num
}

/// `WorldGen.SolidTile`: active, solid, not a solid top, not a half brick or slope.
fn solid_tile(t: &Tile) -> bool {
    t.active()
        && (t.kind as usize) < td::TILE_COUNT
        && td::TILE_SOLID.has(t.kind)
        && !td::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
}

/// Same as [`solid_tile`], but `WaterfallManager.DrawWaterfall` temporarily clears
/// `Main.tileSolid[546]`, so tile 546 does not stop a fall while drawing.
fn solid_tile_draw(t: &Tile) -> bool {
    if t.kind == 546 {
        return false;
    }
    solid_tile(t)
}

fn in_world(world: &World, x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && x < world.width && y < world.height
}

#[allow(clippy::too_many_arguments)]
fn push(
    out: &mut Vec<WaterfallDraw>,
    texture: u8,
    pos: [f32; 2],
    source: [i32; 4],
    color: [f32; 4],
    flip: bool,
) {
    if source[2] <= 0 || source[3] <= 0 {
        return;
    }
    out.push(WaterfallDraw {
        texture,
        pos,
        source,
        flip,
        color,
    });
}

/// Helpers used only inside [`WaterfallManager::find_waterfalls`] to mirror the
/// original's `tile2.lava() || tile4.lava() || tile3.lava()` order.
trait LiquidKindChecks {
    fn liquid_kind_is_lava(&self, a: &Tile, b: &Tile) -> bool;
    fn liquid_kind_is_honey(&self, a: &Tile, b: &Tile) -> bool;
    fn liquid_kind_is_shimmer(&self, a: &Tile, b: &Tile) -> bool;
}

impl LiquidKindChecks for Tile {
    fn liquid_kind_is_lava(&self, a: &Tile, b: &Tile) -> bool {
        (self.liquid > 0 && self.liquid_kind() == crate::tile::LiquidKind::Lava)
            || (a.liquid > 0 && a.liquid_kind() == crate::tile::LiquidKind::Lava)
            || (b.liquid > 0 && b.liquid_kind() == crate::tile::LiquidKind::Lava)
    }

    fn liquid_kind_is_honey(&self, a: &Tile, b: &Tile) -> bool {
        (self.liquid > 0 && self.liquid_kind() == crate::tile::LiquidKind::Honey)
            || (a.liquid > 0 && a.liquid_kind() == crate::tile::LiquidKind::Honey)
            || (b.liquid > 0 && b.liquid_kind() == crate::tile::LiquidKind::Honey)
    }

    fn liquid_kind_is_shimmer(&self, a: &Tile, b: &Tile) -> bool {
        (self.liquid > 0 && self.liquid_kind() == crate::tile::LiquidKind::Shimmer)
            || (a.liquid > 0 && a.liquid_kind() == crate::tile::LiquidKind::Shimmer)
            || (b.liquid > 0 && b.liquid_kind() == crate::tile::LiquidKind::Shimmer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::{FLAG_ACTIVE, FLAG_HALF_BRICK, FLAG_LIQUID_LAVA};

    fn world_with_air() -> World {
        let mut w = World::new("test".into(), 80, 60);
        w.world_surface = 10.0;
        w
    }

    fn stone(kind: u16) -> Tile {
        let mut t = Tile::default();
        t.kind = kind;
        t.flags |= FLAG_ACTIVE;
        t
    }

    fn put(w: &mut World, x: i32, y: i32, t: Tile) {
        let i = w.index(x, y);
        w.tiles[i] = t;
    }

    fn put_liquid(w: &mut World, x: i32, y: i32, amount: u8) {
        let i = w.index(x, y);
        w.tiles[i].liquid = amount;
    }

    #[test]
    fn finds_half_brick_spout_with_liquid_to_the_side() {
        let mut w = world_with_air();
        // Half brick at (10, 10), water running down the right-hand side.
        let mut brick = stone(1);
        brick.flags |= FLAG_HALF_BRICK;
        put(&mut w, 10, 10, brick);
        put_liquid(&mut w, 11, 9, 255);
        put_liquid(&mut w, 11, 10, 200);

        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        assert_eq!(mgr.entries(), vec![(10, 10, 0)]);
    }

    #[test]
    fn fast_liquid_does_not_make_a_waterfall() {
        let mut w = world_with_air();
        let mut brick = stone(1);
        brick.flags |= FLAG_HALF_BRICK;
        put(&mut w, 10, 10, brick);
        put_liquid(&mut w, 10, 9, 10);
        put_liquid(&mut w, 11, 9, 10);
        put_liquid(&mut w, 11, 10, 10);
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        assert!(mgr.entries().is_empty());
    }

    #[test]
    fn lava_spout_uses_type_one() {
        let mut w = world_with_air();
        let mut brick = stone(1);
        brick.flags |= FLAG_HALF_BRICK;
        put(&mut w, 10, 10, brick);
        let mut lava = Tile::default();
        lava.liquid = 255;
        lava.flags |= FLAG_LIQUID_LAVA;
        put(&mut w, 11, 9, lava);
        put(&mut w, 11, 10, lava);
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        assert_eq!(mgr.entries(), vec![(10, 10, 1)]);
    }

    #[test]
    fn waterfall_tile_196_drops_below() {
        let mut w = world_with_air();
        put(&mut w, 12, 8, stone(196));
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        assert_eq!(mgr.entries(), vec![(12, 9, 11)]);
    }

    #[test]
    fn frame_counters_match_the_original_rates() {
        let mut mgr = WaterfallManager::new();
        for _ in 0..3 {
            mgr.update_tick();
        }
        assert_eq!(mgr.regular_frame, 1);
        for _ in 0..21 {
            mgr.update_tick();
        }
        // 24 ticks: regular advances every 3 (8), slow every 7 (3).
        assert_eq!(mgr.regular_frame, 8);
        assert_eq!(mgr.slow_frame, 3);
    }

    #[test]
    fn draw_emits_a_falling_water_quad() {
        let mut w = world_with_air();
        let mut brick = stone(2);
        brick.flags |= FLAG_HALF_BRICK;
        put(&mut w, 10, 10, brick);
        put_liquid(&mut w, 11, 9, 255);
        put_liquid(&mut w, 11, 10, 255);
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        mgr.update_tick();
        let light = LightMap::new();
        let style = LightStyle::for_mode(crate::lighting::LightMode::Color);
        let mut out = Vec::new();
        mgr.draw(&w, &light, &style, 0.0, [0, 0, 40, 40], &mut out);
        assert!(!out.is_empty());
        assert!(out.iter().any(|d| d.texture == 0));
        assert!(out.iter().any(|d| d.pos[0] == 160.0 && d.pos[1] == 160.0));
        assert!(out.iter().all(|d| d.source[2] > 0 && d.source[3] > 0));
    }

    #[test]
    fn anim_key_is_zero_without_waterfalls() {
        let mgr = WaterfallManager::new();
        assert_eq!(mgr.anim_key(), 0);
    }
}
