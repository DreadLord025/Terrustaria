//! `LiquidRenderer.DrawNormalLiquids`: the prepared-record liquid passes (the
//! opaque background layer and the translucent foreground layer) plus the
//! liquid mask drawn behind blocks standing in water (`DrawTile_LiquidBehindTile`).
use crate::assets;
use crate::render::tiles::{bilerp_corners, solid_full, sub_uv_frac};
use tr_core::lighting::{LightMap, LightStyle};
use tr_core::tiledata as td;
use tr_core::{TILE_PX, World};
use tr_render::SpriteRenderer;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum LiquidPass {
    /// `isBackgroundDraw = true`: opaque, drawn before walls and tiles.
    Background,
    /// `isBackgroundDraw = false`: `DEFAULT_OPACITY`, drawn over entities.
    Foreground,
}

/// `Main.IsLiquidStyleWater` (`Main.cs:57456`): styles 1 (lava), 11 (honey) and
/// 14 (shimmer) are outside the `liquidAlpha` cross-fade ramp.
pub(crate) fn is_liquid_style_water(style: usize) -> bool {
    style != 1 && style != 11 && style != 14
}

/// `Main.DrawWaters`' pass sequence (`Main.cs:57497-57506` plus the `DrawLiquid`
/// tail at `Main.cs:57529-57532`): every non-current water style with a
/// non-zero `liquidAlpha` draws its water cells first (`waterOnly: true`, the
/// background pass at alpha 1), then the current style draws everything, then
/// the shimmer pass runs because `waterOnly` is false.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_waters(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    light: &LightMap,
    style: &LightStyle,
    anim_time: f32,
    draws: &[tr_core::liquid_draw::LiquidDraw],
    pass: LiquidPass,
    water_style: usize,
    liquid_alpha: &[f32; 15],
) -> u32 {
    let mut quads = 0;
    let mut fading = false;
    for j in 0..15 {
        if !is_liquid_style_water(j) || liquid_alpha[j] <= 0.0 || j == water_style {
            continue;
        }
        // `isBackground ? 1f : Main.liquidAlpha[j]` (`Main.cs:57502`).
        let alpha = if pass == LiquidPass::Background {
            1.0
        } else {
            liquid_alpha[j]
        };
        quads += draw_liquids(r, lookups, light, style, anim_time, draws, pass, j, alpha, true);
        fading = true;
    }
    // `this.DrawLiquid(isBackground, Main.waterStyle, flag ? liquidAlpha[style] : 1f)`.
    let alpha = if fading { liquid_alpha[water_style] } else { 1.0 };
    quads += draw_liquids(
        r,
        lookups,
        light,
        style,
        anim_time,
        draws,
        pass,
        water_style,
        alpha,
        false,
    );
    // `if (!waterOnly) { LiquidRenderer.Instance.DrawShimmer(...); }`.
    quads += draw_shimmer(r, lookups, anim_time, draws, pass);
    quads
}

/// `LiquidRenderer.DrawNormalLiquids`: emit one pass of the prepared liquid records.
///
/// `water_style`/`global_alpha` are the pass's biome sheet and its
/// `liquidAlpha` entry: type-0 (water) cells take the pass's sheet and alpha,
/// non-water styles are skipped entirely while `water_only` is set (the
/// non-current passes of `Main.cs:57500-57504`), and shimmer cells never draw
/// here — `LiquidRenderer` moves them into its shimmer cache and
/// [`draw_shimmer`] renders them.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_liquids(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    light: &LightMap,
    style: &LightStyle,
    anim_time: f32,
    draws: &[tr_core::liquid_draw::LiquidDraw],
    pass: LiquidPass,
    water_style: usize,
    global_alpha: f32,
    water_only: bool,
) -> u32 {
    // `LiquidRenderer.Update`: body/edge frames advance with the (wind-driven)
    // animation clock, interior frames with the slower waterfall clock.
    let anim_frame = ((anim_time * 8.0) as i32) & 15;
    let waterfall_frame = ((anim_time * 0.5) as i32) & 15;
    let mut quads = 0;
    for d in draws {
        if !d.visible || d.src_w <= 0 || d.src_h <= 0 {
            continue;
        }
        // `if (ptr2->IsVisible && (!waterOnly || ptr2->Type == 0))` — and the
        // type-3 cells live in the shimmer cache (`LiquidRenderer.cs:588-601`).
        if d.liquid_type == 3 || (water_only && d.liquid_type != 0) {
            continue;
        }
        // `DEFAULT_OPACITY`: water 0.6, lava/honey 0.95.
        let (tex, default_opacity) = match d.liquid_type {
            1 => (1, 0.95),
            2 => (11, 0.95),
            _ => (water_style, 0.60),
        };
        let corners = light.corner4(d.x, d.y);
        // `DrawNormalLiquids`: surface tiles snap to the still frame at y=1280,
        // everything else advances inside its 80 px block.
        let src_y = if d.is_surface {
            1280
        } else if d.src_x == 16 {
            d.src_y + waterfall_frame * 80
        } else {
            d.src_y + anim_frame * 80
        };
        let Some(uv) = lookups.liquid_uv(tex, d.src_x, src_y, d.src_w as u32, d.src_h as u32)
        else {
            continue;
        };
        let mut alpha = if pass == LiquidPass::Background {
            d.opacity
        } else {
            d.opacity * default_opacity
        };
        // `case 0: num *= globalAlpha` (`LiquidRenderer.cs:649`) — the biome
        // pass's `liquidAlpha`; lava's `player.lavaOpacity` factor is not
        // modelled (the player has no submersion state in this port).
        if d.liquid_type == 0 {
            alpha *= global_alpha;
        }
        let alpha = alpha.min(1.0);
        // `Lighting.GetCornerColors` interpolates between the four corners on the
        // GPU. The instance format carries one tint, so split the visible rect
        // into a 3x3 grid and evaluate the bilinear corner colour per cell.
        let xs = [0, d.src_w / 3, d.src_w * 2 / 3, d.src_w];
        let ys = [0, d.src_h / 3, d.src_h * 2 / 3, d.src_h];
        for i in 0..3 {
            for j in 0..3 {
                let cw = xs[i + 1] - xs[i];
                let ch = ys[j + 1] - ys[j];
                if cw <= 0 || ch <= 0 {
                    continue;
                }
                let u = (xs[i] + xs[i + 1]) as f32 * 0.5 / d.src_w as f32;
                let v = (ys[j] + ys[j + 1]) as f32 * 0.5 / d.src_h as f32;
                let col = bilerp_corners(corners, u, v);
                let tint = style.shade(col, d.x, d.y, anim_time);
                r.push_sprite(
                    [
                        d.x as f32 * TILE_PX + d.offset_x + xs[i] as f32,
                        d.y as f32 * TILE_PX + d.offset_y + ys[j] as f32,
                    ],
                    [cw as f32, ch as f32],
                    sub_uv_frac(
                        uv,
                        xs[i] as f32,
                        ys[j] as f32,
                        cw as f32,
                        ch as f32,
                        d.src_w as f32,
                        d.src_h as f32,
                    ),
                    [tint[0].min(1.0), tint[1].min(1.0), tint[2].min(1.0), alpha],
                );
                quads += 1;
            }
        }
    }
    quads
}

/// `LiquidRenderer.DrawShimmer` (`LiquidRenderer.cs:675-723`): shimmer liquid
/// does not take the normal path at all — every type-3 cell draws here, twice.
/// The base quad is self-luminous: its corner colours are white × opacity ×
/// `GetShimmerBaseColor` (the golden↔pale wave), *not* the light map — that is
/// why shimmer glows in dark caves. The sparkle overlay quad
/// (`SetShimmerVertexColors_Sparkle`, `LiquidRenderer.cs:725-736`) adds the
/// rainbow `GetShimmerGlitterColor` per corner.
///
/// The original tile batch is premultiplied and the glitter colour carries
/// alpha 0, i.e. the sparkle is an additive term. This renderer blends
/// non-premultiplied, so each quad encodes the premultiplied colour as
/// `rgb/max(r,g,b)` with `alpha = max(r,g,b)` — the same source contribution
/// with a slightly dimmed backdrop under the sparkle (documented deviation).
///
/// `Main.DrawTileInWater` (the tile-518 submerged redraw) is not ported: the
/// generator never places tile 518.
pub(crate) fn draw_shimmer(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    anim_time: f32,
    draws: &[tr_core::liquid_draw::LiquidDraw],
    pass: LiquidPass,
) -> u32 {
    let anim_frame = ((anim_time * 8.0) as i32) & 15;
    // `Main.timeForVisualEffects` (`Main.cs:19751`-shaped wrap at 216000).
    let tve = (anim_time * 60.0) % 216000.0;
    let mut quads = 0;
    for d in draws {
        if !d.visible || d.liquid_type != 3 || d.src_w <= 0 || d.src_h <= 0 {
            continue;
        }
        // `val = Opacity * (isBackgroundDraw ? 1f : 0.75f)`.
        let val = if pass == LiquidPass::Background {
            d.opacity
        } else {
            d.opacity * 0.75
        }
        .min(1.0);
        let src_y = if d.is_surface { 1280 } else { d.src_y + anim_frame * 80 };
        let Some(uv) = lookups.liquid_uv(14, d.src_x, src_y, d.src_w as u32, d.src_h as u32)
        else {
            continue;
        };
        // `SetShimmerVertexColors`: white × opacity, then × `GetShimmerBaseColor`
        // per corner — TL, TR, BL, BR.
        let corners: [[f32; 3]; 4] = [
            [d.x, d.y],
            [d.x + 1, d.y],
            [d.x, d.y + 1],
            [d.x + 1, d.y + 1],
        ]
        .map(|[cx, cy]| {
            let b = tr_core::tile_light::shimmer_base_color(cx, cy, tve);
            [b[0] * val, b[1] * val, b[2] * val]
        });
        quads += push_split(r, uv, d, &corners, val);

        // The sparkle overlay: skipped for surface rows whose base source sits
        // on the `X == 16, Y % 80 == 48` frame unless the cell passes the
        // `(x + y) % 2` coin (`LiquidRenderer.cs:709-717`).
        let flag = d.src_x != 16 || d.src_y % 80 != 48;
        if !flag && (d.x + d.y) % 2 != 0 {
            continue;
        }
        let frame = shimmer_frame(d.x, d.y, flag, tve);
        let Some(suv) =
            lookups.liquid_uv(14, d.src_x + 48, d.src_y + 80 * frame, d.src_w as u32, d.src_h as u32)
        else {
            continue;
        };
        let glitter: [[f32; 3]; 4] = corners
            .iter()
            .zip([[d.x, d.y], [d.x + 1, d.y], [d.x, d.y + 1], [d.x + 1, d.y + 1]])
            .map(|(_, [cx, cy])| {
                let o = glitter_opacity(flag, cx, cy, tve);
                let g = tr_core::tile_light::hsl_to_rgb(
                    (((cx as f32 + cy as f32 / 6.0) + tve / 30.0) / 6.0) % 1.0,
                    1.0,
                    0.5,
                );
                [g[0] * o * val, g[1] * o * val, g[2] * o * val]
            })
            .collect::<Vec<_>>()
            .try_into()
            .expect("four corners");
        quads += push_split_premultiplied(r, suv, d, &glitter);
    }
    quads
}

/// Pushes one record's quad as the 3x3 bilinear split (the shared light-split
/// machinery of the normal pass) with the given per-corner colours.
#[allow(clippy::too_many_arguments)]
fn push_split(
    r: &mut SpriteRenderer,
    uv: [f32; 4],
    d: &tr_core::liquid_draw::LiquidDraw,
    corners: &[[f32; 3]; 4],
    alpha: f32,
) -> u32 {
    let xs = [0, d.src_w / 3, d.src_w * 2 / 3, d.src_w];
    let ys = [0, d.src_h / 3, d.src_h * 2 / 3, d.src_h];
    let mut quads = 0;
    for i in 0..3 {
        for j in 0..3 {
            let cw = xs[i + 1] - xs[i];
            let ch = ys[j + 1] - ys[j];
            if cw <= 0 || ch <= 0 {
                continue;
            }
            let u = (xs[i] + xs[i + 1]) as f32 * 0.5 / d.src_w as f32;
            let v = (ys[j] + ys[j + 1]) as f32 * 0.5 / d.src_h as f32;
            let tint = bilerp_corners(*corners, u, v);
            r.push_sprite(
                [
                    d.x as f32 * TILE_PX + d.offset_x + xs[i] as f32,
                    d.y as f32 * TILE_PX + d.offset_y + ys[j] as f32,
                ],
                [cw as f32, ch as f32],
                sub_uv_frac(
                    uv,
                    xs[i] as f32,
                    ys[j] as f32,
                    cw as f32,
                    ch as f32,
                    d.src_w as f32,
                    d.src_h as f32,
                ),
                [tint[0], tint[1], tint[2], alpha],
            );
            quads += 1;
        }
    }
    quads
}

/// [`push_split`] for the premultiplied glitter colour: encode each corner as
/// `rgb/max(r,g,b)` with `alpha = max(r,g,b)` (see [`draw_shimmer`]).
fn push_split_premultiplied(
    r: &mut SpriteRenderer,
    uv: [f32; 4],
    d: &tr_core::liquid_draw::LiquidDraw,
    corners: &[[f32; 3]; 4],
) -> u32 {
    let max = |c: [f32; 3]| c[0].max(c[1]).max(c[2]);
    let alpha = corners.iter().map(|c| max(*c)).fold(0.0f32, f32::max);
    let encoded: [[f32; 3]; 4] = corners
        .iter()
        .map(|c| {
            let m = max(*c);
            if m <= 0.0 {
                [0.0; 3]
            } else {
                [c[0] / m, c[1] / m, c[2] / m]
            }
        })
        .collect::<Vec<_>>()
        .try_into()
        .expect("four corners");
    push_split(r, uv, d, &encoded, alpha)
}

/// `LiquidRenderer.GetShimmerFrame` (`LiquidRenderer.cs:782-792`): the sparkle
/// strip's frame for a cell, a 16-frame cycle along the same diagonal wave.
fn shimmer_frame(x: i32, y: i32, top: bool, tve: f32) -> i32 {
    let wx = x as f32 + 0.5;
    let wy = y as f32 + 0.5;
    let mut num = (wx + wy / 6.0) / 10.0 - tve / 360.0;
    if !top {
        num += wx + wy;
    }
    ((num as i32) % 16 + 16) % 16
}

/// `LiquidRenderer.GetShimmerGlitterOpacity` (`LiquidRenderer.cs:766-773`).
fn glitter_opacity(top: bool, x: i32, y: i32, tve: f32) -> f32 {
    if top {
        return 0.5;
    }
    let wave = shimmer_wave(x as f32, y as f32, tve);
    let noise = simple_white_noise(x as u32, y as u32) as f32;
    let inner = (noise / 10.0 + tve / 180.0).sin();
    remap(remap(wave, -0.5, 1.0, 0.0, 0.35) * inner, 0.0, 0.5, 0.0, 1.0)
}

/// `LiquidRenderer.GetShimmerWave` (`LiquidRenderer.cs:754-757`).
fn shimmer_wave(x: f32, y: f32, tve: f32) -> f32 {
    (((x + y / 6.0) / 10.0 - tve / 360.0) * std::f32::consts::TAU).sin()
}

/// `LiquidRenderer.SimpleWhiteNoise` (`LiquidRenderer.cs:775-780`).
fn simple_white_noise(x: u32, y: u32) -> u32 {
    let x = 36469u32.wrapping_mul(x & 0xFFFF).wrapping_add(x >> 16);
    let y = 18012u32.wrapping_mul(y & 0xFFFF).wrapping_add(y >> 16);
    x.wrapping_shl(16).wrapping_add(y)
}

/// `Utils.Remap(value, fromMin, fromMax, toMin, toMax)`.
fn remap(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    let t = ((value - from_min) / (from_max - from_min)).clamp(0.0, 1.0);
    to_min + (to_max - to_min) * t
}

/// The WaterDistortion mask input (M1.4-BG-18 phase 4): one world-pixel rect per
/// visible liquid record, exactly the extent [`draw_liquids`]/[`draw_shimmer`]
/// emit (`tile * 16 + offset`, `src_w x src_h`). Vanilla renders the same liquid
/// layer into `Main.waterTarget` (`Main.cs:56723-56734`) and the distortion pass
/// reads its coverage; every liquid type counts (`RenderWater` draws water, lava,
/// honey and shimmer into the one target). `tr-render`'s `WaterMask` rasterises
/// the rects.
pub(crate) fn water_mask_rects(draws: &[tr_core::liquid_draw::LiquidDraw]) -> Vec<[f32; 4]> {
    draws
        .iter()
        .filter(|d| d.visible && d.src_w > 0 && d.src_h > 0)
        .map(|d| {
            [
                d.x as f32 * TILE_PX + d.offset_x,
                d.y as f32 * TILE_PX + d.offset_y,
                d.src_w as f32,
                d.src_h as f32,
            ]
        })
        .collect()
}

/// `TileDrawing.DrawTile_LiquidBehindTile`: a block that stands in liquid gets the
/// liquid drawn *behind* it, so the transparent parts of edge and slope frames show
/// water instead of the parallax background.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_tile_liquid_behind(
    r: &mut SpriteRenderer,
    world: &World,
    lookups: &assets::Lookups,
    light: &LightMap,
    style: &LightStyle,
    anim_time: f32,
    tx: i32,
    ty: i32,
    tile: &tr_core::tile::Tile,
    water_style: usize,
) -> u32 {
    // `BlocksWaterDrawingBehindSelf`: chests and friends never draw water behind
    // themselves, and while sloped they fall back to the plain liquid frame.
    const BLOCKS_WATER_BEHIND: [u16; 5] = [54, 541, 328, 459, 470];
    if std::env::var_os("TR_NO_LIQBEHIND").is_some() {
        return 0;
    }
    let blocks_behind = BLOCKS_WATER_BEHIND.contains(&tile.kind);
    // `!tileSolid[type] || tileSolidTop[type]` both bail out (line 4064).
    if !tile.active() || !td::TILE_SOLID.has(tile.kind) || td::TILE_SOLID_TOP.has(tile.kind) {
        return 0;
    }
    let slope = tile.slope();
    if blocks_behind && slope == 0 {
        return 0;
    }
    let neighbor = |dx: i32, dy: i32| world.tile(tx + dx, ty + dy).copied().unwrap_or_default();
    let right = neighbor(1, 0);
    let left = neighbor(-1, 0);
    let above = neighbor(0, -1);
    let below = neighbor(0, 1);
    let mut level = 0i32;
    let mut kind = tile.liquid_kind();
    let mut flag_self = false;
    let mut flag_left = false;
    let mut flag_right = false;
    let mut flag_above = false;
    let mut flag_below = false;
    // `flag5`: the tile's own liquid only counts when the block is not full size.
    if tile.liquid > 0 && (slope != 0 || (tile.half_brick() && tile.liquid > 160)) {
        flag_self = true;
        kind = tile.liquid_kind();
        level = tile.liquid as i32;
    }
    if left.liquid > 0 && slope != 1 && slope != 3 {
        flag_left = true;
        kind = left.liquid_kind();
        level = level.max(left.liquid as i32);
    }
    if right.liquid > 0 && slope != 2 && slope != 4 {
        flag_right = true;
        kind = right.liquid_kind();
        level = level.max(right.liquid as i32);
    }
    if above.liquid > 0 && slope != 3 && slope != 4 {
        flag_above = true;
        kind = above.liquid_kind();
    }
    if below.liquid > 0 && slope != 1 && slope != 2 {
        if below.liquid > 240 {
            flag_below = true;
        }
        kind = below.liquid_kind();
    }
    if !(flag_self || flag_left || flag_right || flag_above || flag_below) {
        return 0;
    }
    if flag_below && (flag_left || flag_right) {
        flag_left = true;
        flag_right = true;
    }
    let tex = match kind {
        tr_core::tile::LiquidKind::Lava => 1,
        tr_core::tile::LiquidKind::Honey => 11,
        tr_core::tile::LiquidKind::Shimmer => 14,
        tr_core::tile::LiquidKind::Water => water_style,
    };
    // `(!flag3 || !(flag | flag2)) && !(flag4 & flag3)`: liquid on every side keeps
    // the default full-cell rect at the tile origin instead of a partial strip.
    let partial = (!flag_above || !(flag_left || flag_right)) && !(flag_below && flag_above);
    let (px, py, src_x, src_y, src_w, src_h);
    if !partial {
        px = tx as f32 * TILE_PX;
        py = ty as f32 * TILE_PX;
        src_x = 0;
        // The default rect skips the four surface rows of `Liquid_*`; the slope
        // masks are full-cell triangles, so shifting them would open a seam along
        // the diagonal.
        src_y = if slope != 0 { 0 } else { 4 };
        src_w = 16;
        src_h = 16;
    } else if flag_above {
        px = tx as f32 * TILE_PX;
        py = ty as f32 * TILE_PX;
        src_x = 0;
        src_y = 4;
        src_w = 16;
        src_h = if slope != 0 || tile.half_brick() {
            12
        } else {
            4
        };
    } else if flag_below && !flag_left && !flag_right {
        px = tx as f32 * TILE_PX;
        py = ty as f32 * TILE_PX + 12.0;
        src_x = 0;
        src_y = 4;
        src_w = 16;
        src_h = 4;
    } else {
        // `num14`: how far the liquid surface sits inside the cell (0 at 255).
        let offset = ((256 - level) / 32) * 2;
        let sy = if above.liquid == 0
            && (slope != 0 || tile.half_brick() || !solid_full(world, tx, ty - 1))
        {
            0
        } else {
            4
        };
        let mut x = tx as f32 * TILE_PX;
        py = ty as f32 * TILE_PX + offset as f32;
        src_x = 0;
        src_y = if slope != 0 { offset } else { sy };
        src_h = 16 - offset as u32;
        src_w = if slope != 0 || (flag_left && flag_right) || tile.half_brick() {
            // A slope always draws the whole diagonal mask; clipping it to 4 px
            // leaves the shore step showing the parallax background.
            16
        } else if flag_left {
            4
        } else {
            // Liquid only on the right: a 4 px strip against the right edge.
            x += 12.0;
            4
        };
        px = x;
    }
    // `num15`: 1.0 at/above the surface, otherwise 0.5 (0.6/0.9 in front of a
    // wall), with the half-brick and open-corner exceptions.
    let mut opacity = 0.5f32;
    if ty as f64 <= world.world_surface {
        opacity = 1.0;
        if tile.wall == 21 {
            opacity = 0.9;
        } else if tile.wall > 0 {
            opacity = 0.6;
        }
    }
    if tile.half_brick() && above.liquid > 0 && tile.wall > 0 {
        opacity = 0.0;
    }
    // Fully submerged cells sit under an opaque water column: their open corner is
    // deep water, not a window to the parallax. (The original keys this on the
    // average surface height; per-cell submersion reads the same at the shoreline
    // and avoids semi-transparent holes below a lake.)
    if above.liquid > 0 {
        opacity = 1.0;
    }
    if slope == 4 && left.liquid == 0 && !solid_full(world, tx - 1, ty) {
        opacity = 0.0;
    }
    if slope == 3 && right.liquid == 0 && !solid_full(world, tx + 1, ty) {
        opacity = 0.0;
    }
    if opacity <= 0.0 || src_w == 0 || src_h == 0 {
        return 0;
    }
    let uv = if slope != 0 && !blocks_behind {
        lookups.liquid_slope_uv(tex, slope, src_y, src_h)
    } else {
        // `Liquid_*` is 16 px tall; the original's 4..20 strip relies on texture
        // clamp, which for these uniform body rows is the same as cropping.
        let h = src_h.min(16u32.saturating_sub(src_y as u32)).max(1);
        lookups.liquid_part_uv(tex, src_x, src_y, src_w, h)
    };
    let Some(uv) = uv else {
        return 0;
    };
    let corners = light.corner4(tx, ty);
    let mut quads = 0;
    let xs = [0u32, src_w / 3, src_w * 2 / 3, src_w];
    let ys = [0u32, src_h / 3, src_h * 2 / 3, src_h];
    for i in 0..3 {
        for j in 0..3 {
            let cw = xs[i + 1] - xs[i];
            let ch = ys[j + 1] - ys[j];
            if cw == 0 || ch == 0 {
                continue;
            }
            let u = (xs[i] + xs[i + 1]) as f32 * 0.5 / src_w as f32;
            let v = (ys[j] + ys[j + 1]) as f32 * 0.5 / src_h as f32;
            let tint = style.shade(bilerp_corners(corners, u, v), tx, ty, anim_time);
            r.push_sprite(
                [px + xs[i] as f32, py + ys[j] as f32],
                [cw as f32, ch as f32],
                sub_uv_frac(
                    uv,
                    xs[i] as f32,
                    ys[j] as f32,
                    cw as f32,
                    ch as f32,
                    src_w as f32,
                    src_h as f32,
                ),
                [
                    tint[0].min(1.0),
                    tint[1].min(1.0),
                    tint[2].min(1.0),
                    opacity,
                ],
            );
            quads += 1;
        }
    }
    quads
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::liquid_draw::LiquidDraw;

    fn record(x: i32, y: i32, w: i32, h: i32, visible: bool) -> LiquidDraw {
        LiquidDraw {
            x,
            y,
            src_x: 0,
            src_y: 0,
            src_w: w,
            src_h: h,
            offset_x: 0.0,
            offset_y: 0.0,
            is_surface: false,
            opacity: 1.0,
            liquid_type: 0,
            has_wall: false,
            visible,
        }
    }

    /// DW-4.2: the mask input is the visible liquid extent, in world pixels,
    /// for several separate regions at once; invisible/empty records and
    /// the per-record offset (a partial-height surface cell) are handled.
    #[test]
    fn test_DW_4_2_water_mask_rects_follow_the_liquid_records() {
        let mut surface = record(3, 4, 16, 10, true);
        surface.offset_y = 6.0; // a shallow surface cell sits low in its tile
        let draws = [
            record(1, 2, 16, 16, true),
            record(9, 9, 16, 16, false),  // not visible: no water drawn
            record(5, 5, 0, 16, true),    // zero-width source: nothing drawn
            surface,
            record(40, 41, 12, 16, true), // second region, narrow cell
        ];
        let rects = water_mask_rects(&draws);
        assert_eq!(
            rects,
            vec![
                [16.0, 32.0, 16.0, 16.0],
                [48.0, 70.0, 16.0, 10.0],
                [640.0, 656.0, 12.0, 16.0],
            ]
        );
        assert!(water_mask_rects(&[]).is_empty(), "no water -> no rects");
    }

    /// End to end with the real record builder: a pool of liquid cells yields
    /// rects that stay inside the pool's pixel bounds and cover its area, and
    /// every liquid kind counts (lava, honey, shimmer share `Main.waterTarget`).
    #[test]
    fn test_DW_4_2_water_mask_rects_cover_a_built_pool() {
        let mut world = World::new("mask-pool".into(), 40, 40);
        for (x, y) in [(10, 10), (11, 10), (12, 10), (10, 11), (11, 11), (12, 11)] {
            world.tile_mut(x, y).unwrap().liquid = 255;
        }
        // A stone floor and walls keep the pool from pouring out as a
        // waterfall stream (open air under a liquid cell draws one, which is
        // part of the mask too).
        for (x, y) in [(10, 12), (11, 12), (12, 12), (9, 10), (9, 11), (13, 10), (13, 11)] {
            let t = world.tile_mut(x, y).unwrap();
            t.kind = 1;
            t.flags |= tr_core::tile::FLAG_ACTIVE;
        }
        let draws = tr_core::liquid_draw::build(&world, 0, 0, 39, 39);
        let rects = water_mask_rects(&draws);
        assert!(!rects.is_empty(), "a pool must produce mask rects");
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        let mut area = 0.0;
        for r in &rects {
            x0 = x0.min(r[0]);
            y0 = y0.min(r[1]);
            x1 = x1.max(r[0] + r[2]);
            y1 = y1.max(r[1] + r[3]);
            area += r[2] * r[3];
        }
        // The pool is 3x2 tiles at (10,10): 160..208 x 160..192 px.
        assert!(x0 >= 160.0 - 16.0 && x1 <= 208.0 + 16.0, "x extent {x0}..{x1}");
        assert!(y0 >= 160.0 - 16.0 && y1 <= 192.0 + 16.0, "y extent {y0}..{y1}");
        assert!(area >= 3.0 * 2.0 * 16.0 * 16.0 * 0.5, "covers most of the pool: {area}");
        // No liquid anywhere else -> no rects from an empty world.
        let empty = World::new("mask-dry".into(), 40, 40);
        let dry = tr_core::liquid_draw::build(&empty, 0, 0, 39, 39);
        assert!(water_mask_rects(&dry).is_empty());
    }
}
