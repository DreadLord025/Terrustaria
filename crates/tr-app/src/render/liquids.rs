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
    // `Main.timeForVisualEffects` is a DOUBLE tick counter (`Main.cs:1181`,
    // `++` then wrap at 216000, `Main.cs:17162-17164`). The product runs in
    // f64 so whole ticks stay exact through the f32 narrowing (f32 holds
    // integers below 2^24 losslessly); the old f32 product quantised the
    // phase by up to one ULP at 216000 (W4-g76).
    let tve = ((anim_time as f64 * 60.0) % 216000.0) as f32;
    let mut quads = 0;
    for d in draws {
        if !d.visible || d.liquid_type != 3 || d.src_w <= 0 || d.src_h <= 0 {
            continue;
        }
        // `val = Math.Min(1f, Opacity * (isBackgroundDraw ? 1f : 0.75f))`
        // (`LiquidRenderer.cs:700-702`).
        let val = shimmer_base_val(d.opacity, pass == LiquidPass::Background);
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
        // `SetShimmerVertexColors_Sparkle` (`LiquidRenderer.cs:725-736`)
        // multiplies the glitter colour by the RAW `ptr2->Opacity` (:715),
        // NOT the base quad's clamped foreground `val` — the old port dimmed
        // the sparkle by the extra 0.75 (W4-g76).
        let glitter: [[f32; 3]; 4] = [[d.x, d.y], [d.x + 1, d.y], [d.x, d.y + 1], [d.x + 1, d.y + 1]]
            .map(|[cx, cy]| shimmer_glitter_corner(flag, cx, cy, tve, d.opacity));
        quads += push_split_premultiplied(r, suv, d, &glitter);
    }
    quads
}

/// One corner of `SetShimmerVertexColors_Sparkle` (`LiquidRenderer.cs:725-736`):
/// `GetShimmerGlitterColor(top, x, y) * opacity` — the byte-quantised rainbow
/// hue (`Main.hslToRgb`, alpha forced to 0 at :762, i.e. the additive term),
/// times the wave/noise opacity, times the RAW record opacity (:731-734).
fn shimmer_glitter_corner(top: bool, x: i32, y: i32, tve: f32, opacity: f32) -> [f32; 3] {
    let o = glitter_opacity(top, x, y, tve);
    let g = tr_core::tile_light::hsl_to_rgb(shimmer_glitter_hue(x as f32, y as f32, tve), 1.0, 0.5);
    [g[0] * o * opacity, g[1] * o * opacity, g[2] * o * opacity]
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
/// The phase `(wx + wy/6)/10` is computed in f32, then `timeForVisualEffects /
/// 360.0` is subtracted in DOUBLE (:786); the `!top` offset adds the f32 sum
/// `(wx + wy)` cast to double (:789). The old all-f32 phase drifted the frame
/// index at large tick counts (W4-g76, mirroring the W4-d66 palette split).
fn shimmer_frame(x: i32, y: i32, top: bool, tve: f32) -> i32 {
    let wx = x as f32 + 0.5;
    let wy = y as f32 + 0.5;
    let mut num = ((wx + wy / 6.0) / 10.0) as f64 - tve as f64 / 360.0;
    if !top {
        num += (wx + wy) as f64;
    }
    ((num as i32) % 16 + 16) % 16
}

/// `LiquidRenderer.GetShimmerGlitterOpacity` (`LiquidRenderer.cs:766-773`).
/// The inner sine is `Math.Sin((double)((float)SimpleWhiteNoise / 10f) +
/// timeForVisualEffects / 180.0)` (:772): the noise is narrowed to f32 and
/// divided by 10 in f32, THEN widened to double for the `+ tve/180` add and the
/// double sin. The old all-f32 add lost the `tve/180` term entirely once the
/// noise/10 magnitude exceeded the f32 ULP (the sparkle froze in time).
fn glitter_opacity(top: bool, x: i32, y: i32, tve: f32) -> f32 {
    if top {
        return 0.5;
    }
    let wave = shimmer_wave(x as f32, y as f32, tve);
    let noise = simple_white_noise(x as u32, y as u32) as f32 / 10.0;
    let inner = (noise as f64 + tve as f64 / 180.0).sin() as f32;
    remap(remap(wave, -0.5, 1.0, 0.0, 0.35) * inner, 0.0, 0.5, 0.0, 1.0)
}

/// `LiquidRenderer.GetShimmerWave` (`LiquidRenderer.cs:754-757`): the position
/// `(x + y/6)/10` is f32, the `timeForVisualEffects / 360.0` subtraction, the
/// `* 6.2831854820251465` and the sin are DOUBLE before the f32 narrowing
/// (W4-g76 — same split W4-d66 landed on the palette-side base colour).
fn shimmer_wave(x: f32, y: f32, tve: f32) -> f32 {
    let pos = (x + y / 6.0) / 10.0;
    ((pos as f64 - tve as f64 / 360.0) * 6.2831854820251465).sin() as f32
}

/// `LiquidRenderer.GetShimmerGlitterColor`'s hue (`LiquidRenderer.cs:761`):
/// `(float)(((double)(x + y/6f) + timeForVisualEffects / 30.0) / 6.0) % 1f` —
/// the `x + y/6` sum is f32, widened to double for `+ tve/30` and `/6`, narrowed
/// back to f32, and only THEN taken `% 1f` (the modulo is f32, not double).
fn shimmer_glitter_hue(x: f32, y: f32, tve: f32) -> f32 {
    let sum = x + y / 6.0;
    (((sum as f64 + tve as f64 / 30.0) / 6.0) as f32) % 1.0
}

/// `DrawShimmer`'s base-quad opacity (`LiquidRenderer.cs:700-702`):
/// `val = Math.Min(1f, Opacity * (isBackgroundDraw ? 1f : 0.75f))`. The sparkle
/// overlay (:715) instead multiplies by the RAW `Opacity`, not this clamped
/// `val` — see [`draw_shimmer`].
fn shimmer_base_val(opacity: f32, background: bool) -> f32 {
    (opacity * if background { 1.0 } else { 0.75 }).min(1.0)
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

/// `TileDrawing.DrawTile_LiquidBehindTile`'s refusal gate
/// (`TileDrawing.cs:3858`): `!active() || inActive() || tileSolidTop[type]`
/// — plus the port's solidity bail (`!tileSolid[type]`) and the
/// `BlocksWaterDrawingBehindSelf && slope() == 0` arm. W4-g80: the
/// `tileCache.inActive()` term was MISSING — an actuated block stands in
/// liquid like air, so the behind-mask never draws for it (the C# refuses
/// before the neighbour-liquid scan). The C#'s half-brick +
/// `waterfallManager.CheckForWaterfall` clause stays a recorded deviation
/// (the port has no waterfall probe here); `DebugOptions.hideWater` rides
/// the caller's `env::no_liqbehind()` gate.
fn liquid_behind_refused(tile: &tr_core::tile::Tile, blocks_behind: bool) -> bool {
    // `TileDrawing.cs:3858` verbatim terms the port carries.
    if !tile.active()
        || tile.inactive()
        || !td::TILE_SOLID.has(tile.kind)
        || td::TILE_SOLID_TOP.has(tile.kind)
    {
        return true;
    }
    blocks_behind && tile.slope() == 0
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
    if crate::env::no_liqbehind() {
        return 0;
    }
    let blocks_behind = BLOCKS_WATER_BEHIND.contains(&tile.kind);
    if liquid_behind_refused(tile, blocks_behind) {
        return 0;
    }
    let slope = tile.slope();
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

    // ---- W4-g76 shimmer-draw witnesses (WRITTEN NOT RUN, code-only) ----
    // Reference values from an f32/f64-split evaluation of the exact C#
    // expressions (LiquidRenderer.cs:754-797): f32 position sums, double
    // time subtraction / multiplication / sin, f32 narrowing where the C#
    // casts. Tolerances 1e-6 discriminate the old all-f32 drift while
    // allowing libm last-ULP differences in the double sin.

    fn close3(a: [f32; 3], b: [f32; 3]) {
        for i in 0..3 {
            assert!((a[i] - b[i]).abs() < 1e-6, "channel {i}: {a:?} != {b:?}");
        }
    }

    /// `GetShimmerWave` (:756): double sin of the f32 position minus
    /// `tve/360.0` in double, times the C# literal 6.2831854820251465.
    /// The old all-f32 wave gave -0.98482203 at (10,20,200000) — 2.6e-6 off.
    #[test]
    fn w4_g76_shimmer_wave_sins_in_double() {
        for (x, y, tve, want) in [
            (10.0, 20.0, 200000.0, -0.9848246574401855f64),
            (3.0, 7.0, 0.0, 0.5),
            (120.0, 340.0, 123456.0, -0.9945167899131775),
            (0.0, 0.0, 215999.0, 0.017347514629364014),
        ] {
            let w = shimmer_wave(x, y, tve);
            assert!((w - want as f32).abs() < 1e-6, "({x},{y},{tve}): {w}");
        }
        // The old all-f32 wrap-edge value (0.0176287) is 2.8e-4 away.
        assert!((shimmer_wave(0.0, 0.0, 215999.0) - 0.0176287).abs() > 1e-4);
    }

    /// `GetShimmerFrame` (:782-792): f32 `(wx + wy/6)/10`, double
    /// `- tve/360`, `!top` adds the f32 `(wx + wy)` cast to double; the
    /// truncation is toward zero and the double modulo re-wraps negatives.
    #[test]
    fn w4_g76_shimmer_frame_double_phase_and_offsets() {
        for (top, x, y, tve, want) in [
            (true, 10, 20, 200000.0, 6),
            (false, 10, 20, 200000.0, 5),
            (true, 120, 340, 123456.0, 11),
            (false, 120, 340, 123456.0, 7),
            (true, 10, 20, 0.0, 1),
            (false, 10, 20, 0.0, 0),
            (true, 0, 0, 215999.0, 9),
            (false, 0, 0, 215999.0, 10),
        ] {
            assert_eq!(shimmer_frame(x, y, top, tve), want, "({x},{y},{top},{tve})");
        }
        // The old all-f32 phase answered 6 here; the double phase gives 7.
        assert_eq!(shimmer_frame(120, 340, true, 113661.0), 7);
    }

    /// `SimpleWhiteNoise` (:775-780): unchecked u32 arithmetic, the two
    /// 16-bit lanes folded with 36469 / 18012 then packed `(x << 16) + y`.
    #[test]
    fn w4_g76_simple_white_noise_pins() {
        assert_eq!(simple_white_noise(10, 20), 2425847600);
        assert_eq!(simple_white_noise(3, 7), 2875255940);
        assert_eq!(simple_white_noise(120, 340), 3342168624);
        assert_eq!(simple_white_noise(0, 0), 0);
        // (float)uint narrows: 2425847600 -> 2425847552 (f32 ULP = 256 here).
        assert_eq!(simple_white_noise(10, 20) as f32, 2425847552.0);
    }

    /// `GetShimmerGlitterOpacity` (:766-773): top is a flat 0.5; otherwise
    /// Remap(Remap(wave,-0.5,1,0,0.35) * sin((double)(noise/10f) + tve/180),
    /// 0,0.5,0,1). The inner sin's time term lives in DOUBLE — the old f32
    /// add dropped it once noise/10 exceeded the f32 ULP (0.145 vs 0.261).
    #[test]
    fn w4_g76_glitter_opacity_double_inner_sin() {
        assert_eq!(glitter_opacity(true, 1, 12, 100.0), 0.5);
        for (x, y, want) in [
            (1, 12, 0.2608494460582733f64),
            (1, 25, 0.6818357706069946),
            (1, 19, 0.49602243304252625),
        ] {
            let o = glitter_opacity(false, x, y, 100.0);
            assert!((o - want as f32).abs() < 1e-6, "({x},{y}): {o}");
        }
        // The old all-f32 answers (0.14531924 / 0.49842459) are far away.
        assert!((glitter_opacity(false, 1, 12, 100.0) - 0.14531924).abs() > 0.1);
        assert!((glitter_opacity(false, 1, 25, 100.0) - 0.49842459).abs() > 0.1);
        // Negative wave remaps to 0 -> the product and the opacity are 0.
        assert_eq!(glitter_opacity(false, 10, 20, 200000.0), 0.0);
    }

    /// The glitter hue (:761): `(float)(((double)(x + y/6f) + tve/30) / 6) % 1f`
    /// — the modulo runs on the f32 narrowing, not in double. At the wrap
    /// edge (0,0,215999) the old all-f32 chain gave 0.99450684 (1.2e-4 off).
    #[test]
    fn w4_g76_glitter_hue_double_add_f32_modulo() {
        for (x, y, tve, want) in [
            (1.0, 12.0, 100.0, 0.05555558204650879f64),
            (10.0, 20.0, 200000.0, 0.3333740234375),
            (0.0, 0.0, 215999.0, 0.994384765625),
        ] {
            let h = shimmer_glitter_hue(x, y, tve);
            assert!((h - want as f32).abs() < 1e-6, "({x},{y},{tve}): {h}");
            assert!((0.0..1.0).contains(&h), "hue stays in [0,1): {h}");
        }
        assert!((shimmer_glitter_hue(0.0, 0.0, 215999.0) - 0.99450684).abs() > 1e-4);
    }

    /// `SetShimmerVertexColors_Sparkle` (:725-736): the byte-quantised
    /// rainbow (hslToRgb, sat 1, lum 0.5) times the glitter opacity times
    /// the RAW record opacity — never the base quad's clamped `val`.
    #[test]
    fn w4_g76_glitter_corner_uses_raw_opacity() {
        // (1,12,100): hue 0.0555556 -> bytes (255,85,0), o = 0.26084945.
        let want = [0.2608494460582733f32, 0.08694981783628464, 0.0];
        close3(shimmer_glitter_corner(false, 1, 12, 100.0, 1.0), want);
        // Raw opacity 0.5 halves the corner — the old port's `val` path
        // would have multiplied by 0.75 * 0.5 = 0.375 instead.
        let half = [0.13042472302913666, 0.04347490891814232, 0.0];
        close3(shimmer_glitter_corner(false, 1, 12, 100.0, 0.5), half);
        // top: flat 0.5 opacity; hue 0.5277778 -> bytes (0,212,255).
        close3(
            shimmer_glitter_corner(true, 5, 5, 100.0, 1.0),
            [0.0, 0.4156862795352936, 0.5],
        );
        // Zero wave opacity -> a black (invisible) additive corner.
        close3(shimmer_glitter_corner(false, 10, 20, 200000.0, 1.0), [0.0; 3]);
    }

    /// `DrawShimmer`'s `val` (:700-702): background keeps the raw opacity
    /// (clamped to 1), foreground takes the 0.75 factor before the clamp.
    #[test]
    fn w4_g76_base_val_foreground_factor_and_clamp() {
        assert_eq!(shimmer_base_val(0.8, true), 0.8);
        assert_eq!(shimmer_base_val(0.8, false), 0.6);
        assert_eq!(shimmer_base_val(1.5, true), 1.0);
        assert_eq!(shimmer_base_val(1.5, false), 1.0);
        assert_eq!(shimmer_base_val(1.0, false), 0.75);
    }

    // ---- W4-g80: the actuated draw lane — the behind-liquid refusal ----

    /// W4-g80 witness — `DrawTile_LiquidBehindTile`'s refusal gate
    /// (`TileDrawing.cs:3858`): the `tileCache.inActive()` term. An actuated
    /// (framed-off) block never draws the behind-mask even when its
    /// neighbours carry liquid — the C# refuses BEFORE the neighbour scan;
    /// the pre-g80 port drew it (the term was MISSING).
    #[test]
    fn w4_g80_an_actuated_block_refuses_the_liquid_behind_draw() {
        use tr_core::tile::{Tile, FLAG_ACTIVE, FLAG_INACTIVE};
        // A sloped stone block: the plain gate passes (active, solid,
        // not solidTop, slope != 0 so the BlocksWater arm is inert).
        let mut t = Tile { kind: 1, flags: FLAG_ACTIVE, ..Default::default() };
        t.set_slope(1);
        assert!(!liquid_behind_refused(&t, false), "the sloped block draws");
        // Actuated: refused, whatever the neighbours hold.
        t.flags |= FLAG_INACTIVE;
        assert!(liquid_behind_refused(&t, false), "TileDrawing.cs:3858 inActive()");
        // The refusal is independent of the actuator bit — `inActive()` is
        // the framed-off flag alone (`Tile.cs:606-608`); a tile can be
        // inactive without an actuator (net/world state).
        assert_eq!(t.flags & tr_core::tile::FLAG_ACTUATOR, 0);
    }

    /// W4-g80 witness — the gate's other `:3858` terms stay verbatim:
    /// `!active()` refuses, `tileSolidTop` refuses, the port's
    /// `!tileSolid` bail refuses, and `BlocksWaterDrawingBehindSelf`
    /// refuses only while unsloped.
    #[test]
    fn w4_g80_the_liquid_behind_gate_keeps_its_other_terms() {
        use tr_core::tile::{Tile, FLAG_ACTIVE};
        // Inactive cell (`!active()`).
        let air = Tile { kind: 1, ..Default::default() };
        assert!(liquid_behind_refused(&air, false));
        // A solidTop kind (19, the platform) refuses.
        let plat = Tile { kind: 19, flags: FLAG_ACTIVE, ..Default::default() };
        assert!(liquid_behind_refused(&plat, false));
        // A non-solid kind (a torch, 4) refuses via the port's solidity bail.
        let torch = Tile { kind: 4, flags: FLAG_ACTIVE, ..Default::default() };
        assert!(liquid_behind_refused(&torch, false));
        // BlocksWaterDrawingBehindSelf (54, the chest family stand-in):
        // unsloped refuses, sloped passes (`:3858`'s `slope() == 0` clause).
        let mut chest = Tile { kind: 54, flags: FLAG_ACTIVE, ..Default::default() };
        assert!(liquid_behind_refused(&chest, true));
        chest.set_slope(2);
        assert!(!liquid_behind_refused(&chest, true));
    }
}
