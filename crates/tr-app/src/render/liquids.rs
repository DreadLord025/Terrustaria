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

/// `LiquidRenderer.DrawNormalLiquids`: emit one pass of the prepared liquid records.
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
        // `DEFAULT_OPACITY`: water 0.6, lava/honey 0.95, shimmer 0.75.
        let (tex, default_opacity) = match d.liquid_type {
            1 => (1, 0.95),
            2 => (11, 0.95),
            3 => (14, 0.75),
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
        let alpha = if pass == LiquidPass::Background {
            d.opacity
        } else {
            d.opacity * default_opacity
        }
        .min(1.0);
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
