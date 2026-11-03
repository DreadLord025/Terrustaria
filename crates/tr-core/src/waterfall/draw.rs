//! `WaterfallManager.DrawWaterfall`: the per-waterfall geometry walk that turns
//! each found fall into textured quads. Split out of [`super`] so the manager
//! (finding, frame counters, cadence) stays readable; the walk is a faithful
//! port and keeps the original local numbering (`num11`..`num34`).

use super::{WaterfallDraw, WaterfallManager, GFX_QUALITY};
use crate::lighting::{DynamicLight, LightMap, LightStyle};
use crate::tile::Tile;
use crate::tile_light::FastRandom;
use crate::tiledata as td;
use crate::world::World;
use super::SparkleSpawn;

/// `TileID.Sets.BlocksWaterDrawingBehindSelf`.
fn blocks_water_drawing_behind_self(kind: u16) -> bool {
    matches!(kind, 54 | 328 | 459 | 470 | 541)
}

impl WaterfallManager {
    /// `WaterfallManager.DrawWaterfall`.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_waterfall(
        &mut self,
        draw_style: i32,
        draw_alpha: f32,
        world: &World,
        light: &LightMap,
        style: &LightStyle,
        time: f32,
        mouse_text: u8,
        sparkle_rng: &mut FastRandom,
        view: [i32; 4],
        out: &mut Vec<WaterfallDraw>,
        lights: &mut Vec<DynamicLight>,
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
                let (num23, num24) = rain_frames(self, num12, num13 % 2 == 0);
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
                        // `WaterfallManager.AddLight(num12, num13, num14)` per
                        // fountain cell (`WaterfallManager.cs:464`).
                        lights.push(DynamicLight {
                            x: num13,
                            y: num14,
                            rgb: add_light(num12, mouse_text, self.disco_rgb()),
                        });
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
                // `WaterfallManager.AddLight(num12, num13, num14)` at the top of
                // every step (`WaterfallManager.cs:548`) — before the solid-tile
                // break, so the last cell lights too.
                let light_rgb = add_light(num12, mouse_text, self.disco_rgb());
                if light_rgb != [0.0; 3] {
                    lights.push(DynamicLight {
                        x: num13,
                        y: num14,
                        rgb: light_rgb,
                    });
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

                // `WaterfallManager.TrySparkling(num13, num14, num17, color5)`
                // (`WaterfallManager.cs:679-682`) — the raw `Lighting.GetColor`
                // before `StylizeColor`, 50+ steps into the fall.
                if k > 50 {
                    let raw = light_tint(light, style, num13, num14, time);
                    try_sparkling(
                        sparkle_rng,
                        num13,
                        num14,
                        num17,
                        raw,
                        &mut self.pending_sparkles,
                    );
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
}

/// `WaterfallManager.DrawWaterfall` rain animation frames (`num23` foreground,
/// `num24` background): even columns lead the foreground by 3 frames and the
/// background by 2, both wrapping at the 8-frame rain sheet; snow falls (22)
/// have no background sheet, so the background frame stays 0.
fn rain_frames(manager: &WaterfallManager, kind: i32, even_column: bool) -> (i32, i32) {
    /// `frame + ahead`, wrapping at the 8-frame rain sheet.
    fn lead(frame: i32, ahead: i32) -> i32 {
        let frame = frame + ahead;
        if frame > 7 {
            frame - 8
        } else {
            frame
        }
    }
    match (kind, even_column) {
        (22, true) => (lead(manager.snow_frame_foreground, 3), 0),
        (26, true) => (
            lead(manager.lava_rain_frame_foreground, 3),
            lead(manager.lava_rain_frame_background, 2),
        ),
        (_, true) => (
            lead(manager.rain_frame_foreground, 3),
            lead(manager.rain_frame_background, 2),
        ),
        (22, false) => (manager.snow_frame_foreground, 0),
        (26, false) => (
            manager.lava_rain_frame_foreground,
            manager.lava_rain_frame_background,
        ),
        (_, false) => (manager.rain_frame_foreground, manager.rain_frame_background),
    }
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

/// `WaterfallManager.AddLight` (`WaterfallManager.cs:1067-1116`): the per-cell
/// glow of a waterfall. Only the lava fall, the disco fall, the gemspark falls
/// (15-21), the shimmer fall (25) and the fountain (26) emit light; every other
/// waterfall type returns black.
fn add_light(kind: i32, mouse_text: u8, disco: [u8; 3]) -> [f32; 3] {
    let mtc = mouse_text as f32;
    match kind {
        1 => {
            let n = (0.55 + (270.0 - mtc) / 900.0) * 0.4;
            [n, n * 0.3, n * 0.1]
        }
        26 => {
            let n = (0.55 + (270.0 - mtc) / 900.0) * 0.6;
            [n, n * 0.3, n * 0.1]
        }
        2 => [
            disco[0] as f32 / 255.0 * 0.2,
            disco[1] as f32 / 255.0 * 0.2,
            disco[2] as f32 / 255.0 * 0.2,
        ],
        15 | 17 => [0.0, 0.0, 0.2],
        16 | 18 => [0.0, 0.2, 0.0],
        19 | 21 => [0.2, 0.0, 0.0],
        20 => [0.2, 0.2, 0.2],
        25 => {
            let n = 0.7 + (270.0 - mtc) / 900.0;
            let n2 = 0.7 + (270.0 - mtc) / 125.0;
            [n * 0.6, n2 * 0.25, n * 0.9]
        }
        _ => [0.0; 3],
    }
}

/// `WaterfallManager.TrySparkling` (`WaterfallManager.cs:1046-1065`): where the
/// fall is lit, a 1-in-20000 draw scaled by the brightest channel over 30 spawns
/// a `Dust` 43 sparkle beside the stream. The dust's velocity is zeroed in the
/// original (`Main.dust[num2].velocity *= 0f`), so only the spawn point matters.
fn try_sparkling(
    rng: &mut FastRandom,
    x: i32,
    y: i32,
    direction: i32,
    color: [f32; 4],
    out: &mut Vec<SparkleSpawn>,
) {
    let (r, g, b) = (
        color[0] * 255.0,
        color[1] * 255.0,
        color[2] * 255.0,
    );
    if r > 20.0 || b > 20.0 || g > 20.0 {
        let mut num = r;
        if g > num {
            num = g;
        }
        if b > num {
            num = b;
        }
        if (rng.next_range(0, 20000) as f32) < num / 30.0 {
            out.push(SparkleSpawn {
                x: (x * 16 - direction * 7) as f32,
                y: (y * 16 + 6) as f32,
            });
        }
    }
}

/// `WorldGen.SolidTile`: active, solid, not a solid top, not a half brick or slope.
pub(super) fn solid_tile(t: &Tile) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// `TrySparkling` is the only randomness in the waterfall walk: one
    /// 1-in-20000 roll per lit fall per frame, keyed on the frame-derived
    /// seed (`WaterfallManager.cs:1046-1065`). The grant must be a pure
    /// function of (seed, position, direction, light colour) — two managers
    /// on the same frame must grant the exact same spawn point or none.
    #[test]
    fn try_sparkling_grants_deterministically_for_a_seed() {
        // Find a seed the roll grants. The search itself is deterministic:
        // the same seed always produces the same roll, so this finds the same
        // seed on every machine and every run.
        let seed = (0u64..)
            .map(|s| {
                let mut r = FastRandom::new(s);
                (s, r.next_range(0, 20000))
            })
            .find(|(_, roll)| *roll < 8)
            .expect("the 1-in-20000 roll must grant within the sweep")
            .0;
        let mut first = Vec::new();
        try_sparkling(
            &mut FastRandom::new(seed),
            100,
            40,
            1,
            [1.0, 1.0, 1.0, 1.0],
            &mut first,
        );
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].x, (100 * 16 - 7) as f32);
        assert_eq!(first[0].y, (40 * 16 + 6) as f32);
        // A fresh RNG on the same seed grants the identical spawn.
        let mut second = Vec::new();
        try_sparkling(
            &mut FastRandom::new(seed),
            100,
            40,
            1,
            [1.0, 1.0, 1.0, 1.0],
            &mut second,
        );
        assert_eq!(first, second);
        // A dark fall never rolls (`r/g/b` gate, `WaterfallManager.cs:1052`).
        let mut dark = Vec::new();
        try_sparkling(
            &mut FastRandom::new(seed),
            100,
            40,
            1,
            [0.05, 0.05, 0.05, 1.0],
            &mut dark,
        );
        assert!(dark.is_empty());
    }
}
