//! `Terraria.GameContent.Liquid.LiquidRenderer.InternalPrepareDraw` port.
//!
//! Produces the per-tile draw record (visible liquid level, edge walls, frame
//! offset, source rectangle, opacity, surface flag) that `DrawNormalLiquids`
//! consumes. Keeping this out of the renderer mirrors the original split between
//! the draw cache and the sprite batch.

use crate::tile::{LiquidKind, Tile};
use crate::tiledata as td;
use crate::world::World;

/// `LiquidRenderer.CACHE_PADDING`.
pub const CACHE_PADDING: i32 = 2;
/// Extra rows below the requested area so the waterfall fade keeps the same
/// loop bound as the original (`rectangle.Height - 10`).
const BOTTOM_MARGIN: i32 = 10;

const WATERFALL_LENGTH: [i32; 4] = [10, 3, 2, 10];

#[derive(Clone, Copy, Debug)]
pub struct LiquidDraw {
    pub x: i32,
    pub y: i32,
    /// `LiquidDrawCache.SourceRectangle`, already including `FrameOffset`.
    pub src_x: i32,
    pub src_y: i32,
    pub src_w: i32,
    pub src_h: i32,
    /// `LiquidDrawCache.LiquidOffset` in pixels.
    pub offset_x: f32,
    pub offset_y: f32,
    pub is_surface: bool,
    pub opacity: f32,
    /// `VisibleType`: 0 water, 1 lava, 2 honey, 3 shimmer.
    pub liquid_type: u8,
    pub has_wall: bool,
    pub visible: bool,
    /// `LiquidCache.IsSolid` — the wave-mask above-copy's neighbour probe
    /// (`LiquidRenderer.cs:529`).
    pub is_solid: bool,
    /// `LiquidCache.IsHalfBrick` — the same probe.
    pub is_half_brick: bool,
    /// `LiquidCache.HasVisibleLiquid` — the wave-mask fill's own gate
    /// (`:507`); wider than `visible` (`IsVisible`, `:517`) which adds the
    /// half-brick-without-wall refusal the sprite pass reads.
    pub has_visible_liquid: bool,
}

#[derive(Clone, Copy, Default)]
struct Cache {
    liquid_level: f32,
    visible_liquid_level: f32,
    is_half_brick: bool,
    is_solid: bool,
    has_liquid: bool,
    has_visible_liquid: bool,
    has_wall: bool,
    liquid_type: u8,
    visible_type: u8,
    opacity: f32,
    has_left_edge: bool,
    has_right_edge: bool,
    has_top_edge: bool,
    has_bottom_edge: bool,
    left_wall: f32,
    right_wall: f32,
    top_wall: f32,
    bottom_wall: f32,
    visible_left_wall: f32,
    visible_right_wall: f32,
    visible_top_wall: f32,
    visible_bottom_wall: f32,
    frame_x: i32,
    frame_y: i32,
}

#[inline]
fn liquid_type(kind: LiquidKind) -> u8 {
    match kind {
        LiquidKind::Water => 0,
        LiquidKind::Lava => 1,
        LiquidKind::Honey => 2,
        LiquidKind::Shimmer => 3,
    }
}

/// `WorldGen.SolidOrSlopedTile` with its default `includePlatforms = false`.
#[inline]
fn solid_or_sloped(t: &Tile) -> bool {
    t.active()
        && (t.kind as usize) < td::TILE_COUNT
        && td::TILE_SOLID.has(t.kind)
        && !td::TILE_SOLID_TOP.has(t.kind)
}

/// Builds the draw records for `[x0..=x1] x [y0..=y1]`.
pub fn build(world: &World, x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<LiquidDraw> {
    let draw_w = (x1 - x0 + 1).max(0);
    let draw_h = (y1 - y0 + 1).max(0);
    if draw_w == 0 || draw_h == 0 {
        return Vec::new();
    }
    let rx = x0 - CACHE_PADDING;
    let ry = y0 - CACHE_PADDING;
    let w = draw_w + CACHE_PADDING * 2;
    let h = draw_h + CACHE_PADDING * 2 + BOTTOM_MARGIN;
    let len = (w * h) as usize;
    let mut cache = vec![Cache::default(); len];
    let hs = h as usize;
    let index = |gx: i32, gy: i32| (gx * h + gy) as usize;

    // Pass 1: per-tile facts.
    for gx in 0..w {
        for gy in 0..h {
            let x = rx + gx;
            let y = ry + gy;
            let t = world.tile(x, y).copied().unwrap_or_default();
            let above_has_liquid = if gy > 0 {
                cache[index(gx, gy - 1)].has_liquid
            } else {
                world.tile(x, y - 1).is_some_and(|a| a.liquid != 0)
            };
            let c = &mut cache[index(gx, gy)];
            c.liquid_level = t.liquid as f32 / 255.0;
            c.is_half_brick = t.half_brick() && above_has_liquid && !td::SET_PLATFORMS.has(t.kind);
            c.is_solid = solid_or_sloped(&t);
            c.has_liquid = t.liquid != 0;
            c.has_wall = t.wall != 0;
            c.liquid_type = if t.liquid != 0 {
                liquid_type(t.liquid_kind())
            } else {
                0
            };
        }
    }

    // Pass 2: visible level (liquid hiding between two liquid cells seeps in).
    for gx in CACHE_PADDING..w - CACHE_PADDING {
        for gy in CACHE_PADDING..h - CACHE_PADDING {
            let i = index(gx, gy);
            let c = cache[i];
            let mut level = 0.0f32;
            if c.is_half_brick && cache[i - 1].has_liquid {
                level = 1.0;
            } else if !c.has_liquid {
                let above = cache[i - 1];
                let below = cache[i + 1];
                let left = cache[i - hs];
                let right = cache[i + hs];
                if above.has_liquid
                    && below.has_liquid
                    && above.liquid_type == below.liquid_type
                    && !above.is_solid
                    && !below.is_solid
                {
                    level = above.liquid_level + below.liquid_level;
                    cache[i].liquid_type = above.liquid_type;
                }
                if left.has_liquid
                    && right.has_liquid
                    && left.liquid_type == right.liquid_type
                    && !left.is_solid
                    && !right.is_solid
                {
                    level = level.max(left.liquid_level + right.liquid_level);
                    cache[i].liquid_type = left.liquid_type;
                }
                level *= 0.5;
            } else {
                level = c.liquid_level;
            }
            cache[i].visible_liquid_level = level;
            cache[i].has_visible_liquid = level != 0.0;
        }
    }

    // Pass 3: waterfall fade (the original's column pass with a 10 row lead).
    for gx in 0..w {
        for gy in 0..h - BOTTOM_MARGIN {
            let i = index(gx, gy);
            let c = cache[i];
            if c.has_visible_liquid && (!c.is_solid || c.is_half_brick) {
                let ty = c.liquid_type as usize;
                cache[i].opacity = 1.0;
                cache[i].visible_type = c.liquid_type;
                let steps = WATERFALL_LENGTH[ty];
                let drop = 1.0 / (steps + 1) as f32;
                let mut fade = 1.0f32;
                for k in 1..=steps {
                    fade -= drop;
                    let j = i + k as usize;
                    if cache[j].is_solid {
                        break;
                    }
                    cache[j].visible_liquid_level = cache[j]
                        .visible_liquid_level
                        .max(c.visible_liquid_level * fade);
                    cache[j].opacity = fade;
                    cache[j].visible_type = c.liquid_type;
                }
            }
            if c.is_solid && !c.is_half_brick {
                cache[i].visible_liquid_level = 1.0;
                cache[i].has_visible_liquid = false;
            } else {
                cache[i].has_visible_liquid = cache[i].visible_liquid_level != 0.0;
            }
        }
    }

    // Pass 4: edges and frame offset.
    for gx in CACHE_PADDING..w - CACHE_PADDING {
        for gy in CACHE_PADDING..h - CACHE_PADDING {
            let i = index(gx, gy);
            if !cache[i].has_visible_liquid {
                cache[i].has_left_edge = false;
                cache[i].has_right_edge = false;
                cache[i].has_top_edge = false;
                cache[i].has_bottom_edge = false;
                continue;
            }
            let above = cache[i - 1];
            let below = cache[i + 1];
            let left = cache[i - hs];
            let right = cache[i + hs];
            let level = cache[i].visible_liquid_level;
            let mut left_wall = 0.0f32;
            let mut right_wall = 1.0f32;
            let mut top_wall = 0.0f32;
            let mut bottom_wall = 1.0f32;
            if !above.has_visible_liquid {
                top_wall += below.visible_liquid_level * (1.0 - level);
            }
            if !below.has_visible_liquid && !below.is_solid && !below.is_half_brick {
                bottom_wall -= above.visible_liquid_level * (1.0 - level);
            }
            if !left.has_visible_liquid && !left.is_solid && !left.is_half_brick {
                left_wall += right.visible_liquid_level * (1.0 - level);
            }
            if !right.has_visible_liquid && !right.is_solid && !right.is_half_brick {
                right_wall -= left.visible_liquid_level * (1.0 - level);
            }
            cache[i].left_wall = left_wall;
            cache[i].right_wall = right_wall;
            cache[i].bottom_wall = bottom_wall;
            cache[i].top_wall = top_wall;
            cache[i].has_top_edge =
                (!above.has_visible_liquid && !above.is_solid) || top_wall != 0.0;
            cache[i].has_bottom_edge =
                (!below.has_visible_liquid && !below.is_solid) || bottom_wall != 1.0;
            cache[i].has_left_edge =
                (!left.has_visible_liquid && !left.is_solid) || left_wall != 0.0;
            cache[i].has_right_edge =
                (!right.has_visible_liquid && !right.is_solid) || right_wall != 1.0;

            let mut fx = 0i32;
            let mut fy = 0i32;
            if !cache[i].has_left_edge {
                if cache[i].has_right_edge {
                    fx += 32;
                } else {
                    fx += 16;
                }
            }
            if cache[i].has_left_edge && cache[i].has_right_edge {
                fx = 16;
                fy += 32;
                if cache[i].has_top_edge {
                    fy = 16;
                }
            } else if !cache[i].has_top_edge {
                if !cache[i].has_left_edge && !cache[i].has_right_edge {
                    fy += 48;
                } else {
                    fy += 16;
                }
            }
            let world_y = ry + gy;
            if fy == 16 && (cache[i].has_left_edge ^ cache[i].has_right_edge) && world_y % 2 == 0 {
                fy += 16;
            }
            cache[i].frame_x = fx;
            cache[i].frame_y = fy;
        }
    }

    // Pass 5: smooth the side walls against stacked neighbours.
    for gx in CACHE_PADDING..w - CACHE_PADDING {
        for gy in CACHE_PADDING..h - CACHE_PADDING {
            let i = index(gx, gy);
            if !cache[i].has_visible_liquid {
                continue;
            }
            let above = cache[i - 1];
            let below = cache[i + 1];
            let left = cache[i - hs];
            let right = cache[i + hs];
            cache[i].visible_left_wall = cache[i].left_wall;
            cache[i].visible_right_wall = cache[i].right_wall;
            cache[i].visible_top_wall = cache[i].top_wall;
            cache[i].visible_bottom_wall = cache[i].bottom_wall;
            if above.has_visible_liquid && below.has_visible_liquid {
                if cache[i].has_left_edge {
                    cache[i].visible_left_wall =
                        (cache[i].left_wall * 2.0 + above.left_wall + below.left_wall) * 0.25;
                }
                if cache[i].has_right_edge {
                    cache[i].visible_right_wall =
                        (cache[i].right_wall * 2.0 + above.right_wall + below.right_wall) * 0.25;
                }
            }
            if left.has_visible_liquid && right.has_visible_liquid {
                if cache[i].has_top_edge {
                    cache[i].visible_top_wall =
                        (cache[i].top_wall * 2.0 + left.top_wall + right.top_wall) * 0.25;
                }
                if cache[i].has_bottom_edge {
                    cache[i].visible_bottom_wall =
                        (cache[i].bottom_wall * 2.0 + left.bottom_wall + right.bottom_wall) * 0.25;
                }
            }
        }
    }

    // Pass 6: surface corner fixes.
    for gx in CACHE_PADDING..w - CACHE_PADDING {
        for gy in CACHE_PADDING..h - CACHE_PADDING {
            let i = index(gx, gy);
            if !cache[i].has_liquid {
                continue;
            }
            let below = cache[i + 1];
            let left = cache[i - hs];
            let right = cache[i + hs];
            if cache[i].has_top_edge
                && !cache[i].has_bottom_edge
                && (cache[i].has_left_edge ^ cache[i].has_right_edge)
            {
                if cache[i].has_right_edge {
                    cache[i].visible_right_wall = below.visible_right_wall;
                    cache[i].visible_top_wall = left.visible_top_wall;
                } else {
                    cache[i].visible_left_wall = below.visible_left_wall;
                    cache[i].visible_top_wall = right.visible_top_wall;
                }
            } else if below.frame_x == 16 && below.frame_y == 32 {
                if cache[i].visible_left_wall > 0.5 {
                    cache[i].visible_left_wall = 0.0;
                    cache[i].frame_x = 0;
                    cache[i].frame_y = 0;
                } else if cache[i].visible_right_wall < 0.5 {
                    cache[i].visible_right_wall = 1.0;
                    cache[i].frame_x = 32;
                    cache[i].frame_y = 0;
                }
            }
        }
    }

    // Pass 7: inner-corner frames.
    for gx in CACHE_PADDING..w - CACHE_PADDING {
        for gy in CACHE_PADDING..h - CACHE_PADDING {
            let i = index(gx, gy);
            if !cache[i].has_liquid
                || cache[i].has_bottom_edge
                || cache[i].has_left_edge
                || cache[i].has_top_edge
                || cache[i].has_right_edge
            {
                continue;
            }
            let above = cache[i - 1];
            let left = cache[i - hs];
            let right = cache[i + hs];
            if left.has_top_edge && above.has_left_edge {
                cache[i].frame_x = 4.max((16.0 - above.visible_left_wall * 16.0) as i32) - 4;
                cache[i].frame_y = 48 + 4.max((16.0 - left.visible_top_wall * 16.0) as i32) - 4;
                cache[i].visible_left_wall = 0.0;
                cache[i].visible_top_wall = 0.0;
                cache[i].visible_right_wall = 1.0;
                cache[i].visible_bottom_wall = 1.0;
            } else if right.has_top_edge && above.has_right_edge {
                cache[i].frame_x = 32 - 16.min((above.visible_right_wall * 16.0) as i32 - 4);
                cache[i].frame_y = 48 + 4.max((16.0 - right.visible_top_wall * 16.0) as i32) - 4;
                cache[i].visible_left_wall = 0.0;
                cache[i].visible_top_wall = 0.0;
                cache[i].visible_right_wall = 1.0;
                cache[i].visible_bottom_wall = 1.0;
            }
        }
    }

    // Final pass: the draw records.
    let mut out = Vec::with_capacity((draw_w * draw_h) as usize);
    for gx in CACHE_PADDING..w - CACHE_PADDING {
        for gy in CACHE_PADDING..h - CACHE_PADDING - BOTTOM_MARGIN {
            let i = index(gx, gy);
            let c = cache[i];
            let left = c.visible_left_wall.min(0.75);
            let right = c.visible_right_wall.max(0.25);
            let top = c.visible_top_wall.min(0.75);
            let mut bottom = c.visible_bottom_wall.max(0.25);
            if c.is_half_brick && c.is_solid && bottom > 0.5 {
                bottom = 0.5;
            }
            let visible = c.has_visible_liquid
                && (c.has_wall || !c.is_half_brick || !c.has_liquid || c.liquid_level >= 1.0);
            let src_x = (16.0 - right * 16.0) as i32 + c.frame_x;
            let src_y = (16.0 - bottom * 16.0) as i32 + c.frame_y;
            let src_w = ((right - left) * 16.0).ceil() as i32;
            let src_h = ((bottom - top) * 16.0).ceil() as i32;
            let world_x = rx + gx;
            let world_y = ry + gy;
            if std::env::var_os("TR_DUMP_LIQUID").is_some()
                && (world_x - 2240).abs() < 6
                && (world_y - 366).abs() < 6
            {
                let above = cache[i - 1];
                let below = cache[i + 1];
                let lft = cache[i - hs];
                let rgt = cache[i + hs];
                log::info!(
                    "LQ {:?} hl={} lvl={:.2} vlvl={:.2} top={:.2} bot={:.2} L={:.2} R={:.2} hT={} hB={} hL={} hR={}",
                    (world_x, world_y),
                    c.has_liquid as i32,
                    c.liquid_level,
                    c.visible_liquid_level,
                    c.top_wall,
                    c.bottom_wall,
                    c.left_wall,
                    c.right_wall,
                    c.has_top_edge as i32,
                    c.has_bottom_edge as i32,
                    c.has_left_edge as i32,
                    c.has_right_edge as i32,
                );
                let _ = (above, below, lft, rgt);
            }
            let is_surface =
                c.frame_x == 16 && c.frame_y == 0 && world_y as f64 > world.world_surface - 40.0;
            out.push(LiquidDraw {
                x: world_x,
                y: world_y,
                src_x,
                src_y,
                src_w,
                src_h,
                offset_x: (left * 16.0).floor(),
                offset_y: (top * 16.0).floor(),
                is_surface,
                opacity: c.opacity,
                liquid_type: c.visible_type,
                has_wall: c.has_wall,
                visible,
                is_solid: c.is_solid,
                is_half_brick: c.is_half_brick,
                has_visible_liquid: c.has_visible_liquid,
            });
        }
    }
    out
}

/// `LiquidRenderer.VISCOSITY_MASK` (`LiquidRenderer.cs:892`): the wave-mask
/// blue channel per visible type (water 0, lava 200, honey 240, shimmer 0,
/// the no-liquid slots 3/4 = 0).
pub const VISCOSITY_MASK: [u8; 5] = [0, 200, 240, 0, 0];

/// `LiquidRenderer.WAVE_MASK_STRENGTH` (`LiquidRenderer.cs:891`): allocated
/// `new byte[5]` and NEVER assigned anywhere in the decompile (full-tree
/// grep) — the R/G/A strength channels are all zero in 1.4.5.8.
pub const WAVE_MASK_STRENGTH: [u8; 5] = [0; 5];

/// One `_waveMask` cell (`InternalPrepareDraw`'s final-pass fill,
/// `LiquidRenderer.cs:524-541`): `R = G = strength >> 1`, `B` = viscosity,
/// `A` = strength. With the all-zero strength table only `B` carries data —
/// exactly what the `WaterShaderData` viscosity filter reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WaveMaskCell {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// The `_waveMask` fill over the prepared records (`LiquidRenderer.cs:499-549`
/// — the draw-cache/wave-mask pass). `draws` is `build()`'s column-major
/// output for the exact area `[x0..=x1] x [y0..=y1]`; the mask is the same
/// shape, indexed `(x - x0) * height + (y - y0)` — the layout
/// `SetWaveMaskData` uploads transposed (`:873`: a `Height x Width` texture).
/// Cells with `has_visible_liquid` take their `VisibleType`'s row; the rest
/// take slot 3 (solid/half-brick) or 4 (open) — both rows are byte-identical
/// (`VISCOSITY_MASK[3] == [4] == 0`, strength all-zero). The above-copy
/// (`:529-531`: a visible cell copies its mask into a non-visible, non-solid,
/// non-half-brick neighbour above) runs in the same column-major order, so
/// it reads the neighbour's already-final cell, exactly like the C# walk.
pub fn wave_mask(draws: &[LiquidDraw], width: usize, height: usize) -> Vec<WaveMaskCell> {
    let mut out = vec![WaveMaskCell::default(); width * height];
    let fill = |c: &mut WaveMaskCell, slot: usize| {
        let b = WAVE_MASK_STRENGTH[slot];
        c.r = b >> 1;
        c.g = b >> 1;
        c.b = VISCOSITY_MASK[slot];
        c.a = b;
    };
    debug_assert_eq!(draws.len(), width * height, "build()'s exact-area output");
    for (i, d) in draws.iter().enumerate() {
        // The fill gate is `HasVisibleLiquid` (`:507`), not `IsVisible`.
        let slot = if d.has_visible_liquid {
            d.liquid_type as usize
        } else if d.is_solid || d.is_half_brick {
            3
        } else {
            4
        };
        fill(&mut out[i], slot.min(4));
        // `num19 != 2` — the copy never reaches above the area's first row
        // (column-major layout: i % height == 0 is the top row).
        if d.has_visible_liquid && i % height != 0 {
            let up = &draws[i - 1];
            if !up.has_visible_liquid && !up.is_solid && !up.is_half_brick {
                out[i - 1] = out[i];
            }
        }
    }
    out
}

/// `LiquidRenderer.HasFullWater` (`LiquidRenderer.cs:799-813`) over the
/// prepared records: a cell covered by a visible NON-surface liquid record
/// is "full water" (the background-coverage probe `Main.DrawBlack`'s
/// `:58008` run-break reads). Out-of-area cells return `true` — the C#'s
/// out-of-cache arm. NOTE: the `Main.DrawBlack` pass itself early-returns
/// under `TileDrawingBase.DrawOwnBlacks` (static ctor `TileDrawingBase.cs:54`
/// sets it true; the only writer is the dev `TimeLogger.ABTestFlag`,
/// `TimeLogger.cs:443-453/:712`) — the port's merged wall+black path in
/// `render/tiles.rs` IS that default path, so this query has no runtime
/// consumer yet; it is ported for the surface's completeness.
pub fn has_full_water(draws: &[LiquidDraw], x: i32, y: i32, x0: i32, y0: i32, x1: i32, y1: i32) -> bool {
    if x < x0 || x > x1 || y < y0 || y > y1 {
        return true;
    }
    match draws.iter().find(|d| d.x == x && d.y == y) {
        Some(d) => d.visible && !d.is_surface,
        None => true,
    }
}

/// `LiquidRenderer.Update` (`LiquidRenderer.cs:831-847`): the wind-driven
/// liquid animation clock. `_frameState += (wind*25 ± 6) * dt` (the +6 arm
/// at `num >= 0`, the -6 arm below), `_waterfallFrameState += 0.5 * dt`;
/// a negative frame state wraps +16, then both wrap %16 and the integer
/// frames are `(int)` truncations. `FocusHelper.PauseLiquidRenderer` (the
/// unfocused-window pause) has no port analogue — the caller gates on its
/// own pause state.
#[derive(Clone, Copy, Debug, Default)]
pub struct LiquidAnimClock {
    pub frame_state: f32,
    pub waterfall_frame_state: f32,
}

impl LiquidAnimClock {
    /// One `Update(gameTime)` step. Returns `(animationFrame,
    /// waterfallAnimationFrame)` — the two `DrawNormalLiquids` source-row
    /// drivers (`:636/:640`).
    pub fn update(&mut self, dt: f32, wind_speed_current: f32) -> (i32, i32) {
        let num = wind_speed_current * 25.0;
        self.frame_state += if num >= 0.0 { num + 6.0 } else { num - 6.0 } * dt;
        self.waterfall_frame_state += 0.5 * dt;
        if self.frame_state < 0.0 {
            self.frame_state += 16.0;
        }
        self.frame_state %= 16.0;
        self.waterfall_frame_state %= 16.0;
        (self.frame_state as i32, self.waterfall_frame_state as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    fn block(x: i32, y: i32) -> World {
        let mut w = World::new("liquid-draw".into(), x + 20, y + 20);
        w.world_surface = 4.0;
        w
    }

    fn set(w: &mut World, x: i32, y: i32, kind: u16, liquid: u8) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            liquid,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
    }

    fn set_water(w: &mut World, x: i32, y: i32) {
        let i = w.index(x, y);
        w.tiles[i].liquid = 255;
    }

    #[test]
    fn open_surface_uses_the_still_frame() {
        let mut w = block(10, 10);
        for x in 0..10 {
            set(&mut w, x, 6, 1, 0);
            for y in 3..6 {
                set_water(&mut w, x, y);
            }
        }
        let out = build(&w, 2, 2, 8, 6);
        let surface: Vec<_> = out.iter().filter(|d| d.y == 3 && d.visible).collect();
        assert!(!surface.is_empty());
        // The open-water middle tiles use the still surface frame ...
        let middle = surface
            .iter()
            .find(|d| d.x == 5)
            .expect("middle surface tile");
        assert!(middle.is_surface, "open surface tile must use frame 1280");
        assert_eq!((middle.src_x, middle.src_y), (16, 0));
        assert_eq!(middle.opacity, 1.0);
        // ... while the shore corner next to the air keeps its edge frame.
        let shore = surface.iter().find(|d| d.x == 2).expect("shore tile");
        assert!(!shore.is_surface);
        assert_eq!(shore.src_x, 0);
    }

    #[test]
    fn interior_water_is_opaque_and_full() {
        let mut w = block(10, 12);
        for x in 0..10 {
            set(&mut w, x, 10, 1, 0);
            for y in 3..10 {
                set_water(&mut w, x, y);
            }
        }
        let out = build(&w, 2, 2, 8, 8);
        let mid = out
            .iter()
            .find(|d| d.x == 5 && d.y == 6 && d.visible)
            .expect("interior water");
        assert!(!mid.is_surface);
        assert_eq!((mid.src_x, mid.src_y), (16, 48));
        assert_eq!((mid.src_w, mid.src_h), (16, 16));
        assert_eq!(mid.opacity, 1.0);
    }

    #[test]
    fn shore_tile_uses_the_left_edge_frame() {
        let mut w = block(10, 10);
        for x in 0..10 {
            set(&mut w, x, 6, 1, 0);
        }
        // Water starts at column 2; columns 0-1 are open air, so column 2 is the shore.
        for x in 2..10 {
            for y in 3..6 {
                set_water(&mut w, x, y);
            }
        }
        let out = build(&w, 0, 2, 8, 5);
        let shore = out
            .iter()
            .find(|d| d.x == 2 && d.y == 4 && d.visible)
            .expect("shore water");
        assert_eq!(shore.src_x, 0, "left edge frame lives in column 0");
        assert!((0.0..16.0).contains(&shore.offset_x));
        assert!(shore.src_w > 0 && shore.src_h > 0);
        assert!(!shore.is_surface);
    }

    /// W4-g47 retire — the wave-mask fill (`LiquidRenderer.cs:524-541`):
    /// only the viscosity blue channel carries data (the strength table is
    /// all-zero, `:891`); lava 200, honey 240, water/shimmer/empty 0; the
    /// above-copy (`:529-531`) paints an open non-visible neighbour.
    /// Written, NOT run (code-only mode).
    #[test]
    fn the_wave_mask_carries_only_the_viscosity_channel() {
        let rec = |x: i32, y: i32, ty: u8, visible: bool| LiquidDraw {
            x,
            y,
            src_x: 0,
            src_y: 0,
            src_w: 16,
            src_h: 16,
            offset_x: 0.0,
            offset_y: 0.0,
            is_surface: false,
            opacity: 1.0,
            liquid_type: ty,
            has_wall: false,
            visible,
            is_solid: false,
            is_half_brick: false,
            has_visible_liquid: visible,
        };
        // Column-major, area (0,0)-(1,3): column 0 rows 0..3, column 1 rows 0..3.
        let mut draws = vec![
            rec(0, 0, 0, false), // open above the lava -> the copy target
            rec(0, 1, 1, true),  // lava
            rec(0, 2, 2, true),  // honey
            rec(0, 3, 3, true),  // shimmer
            rec(1, 0, 0, false), // solid cell -> slot 3, no copy over it
            rec(1, 1, 0, true),  // water below the solid
            rec(1, 2, 0, false),
            rec(1, 3, 0, false),
        ];
        draws[4].is_solid = true;
        let mask = wave_mask(&draws, 2, 4);
        assert_eq!(mask[1].b, 200, "lava viscosity");
        assert_eq!(mask[2].b, 240, "honey viscosity");
        assert_eq!(mask[3].b, 0, "shimmer viscosity");
        assert_eq!(mask[5].b, 0, "water viscosity");
        assert_eq!(mask[0].b, 200, "the open cell above the lava got the copy");
        assert_eq!(mask[4], WaveMaskCell::default(), "the solid above water: no copy");
        for c in &mask {
            assert_eq!((c.r, c.g, c.a), (0, 0, 0), "the strength channels are zero");
        }
    }

    /// W4-g47 retire — `HasFullWater` (`LiquidRenderer.cs:799-813`): a
    /// visible non-surface record answers true, a surface/invisible record
    /// false, out-of-area true (the C#'s out-of-cache arm).
    #[test]
    fn has_full_water_reads_the_visible_non_surface_records() {
        let mut w = block(10, 12);
        for x in 0..10 {
            set(&mut w, x, 10, 1, 0);
            for y in 3..10 {
                set_water(&mut w, x, y);
            }
        }
        let out = build(&w, 2, 2, 8, 8);
        // Interior water (5,6) is visible + non-surface (the interior
        // witness above) -> full water.
        assert!(has_full_water(&out, 5, 6, 2, 2, 8, 8));
        // Out of the draw area -> the C# out-of-cache true.
        assert!(has_full_water(&out, 50, 50, 2, 2, 8, 8));
        // The surface row of the open pool is NOT full water.
        let mut w2 = block(10, 10);
        for x in 0..10 {
            set(&mut w2, x, 6, 1, 0);
            for y in 3..6 {
                set_water(&mut w2, x, y);
            }
        }
        let out2 = build(&w2, 2, 2, 8, 6);
        assert!(
            !has_full_water(&out2, 5, 3, 2, 2, 8, 6),
            "the open surface row answers false"
        );
        // A dry in-area cell has no record entry -> the C# cache default
        // (IsVisible false) answers false; the port's find() misses and
        // returns the out-of-cache true ONLY outside the area — inside,
        // build() emits a record for every cell, so the miss cannot happen.
        let dry = build(&World::new("dry".into(), 30, 30), 2, 2, 8, 8);
        assert!(!has_full_water(&dry, 5, 5, 2, 2, 8, 8), "dry cell: not visible");
    }

    /// W4-g47 retire — the wind-driven animation clock
    /// (`LiquidRenderer.cs:831-847`): calm air advances 6 frames/s, a full
    /// storm 31+6=37 (speed 1.24 -> num 31), a negative wind reverses below
    /// -0.24; the waterfall clock stays wind-independent at 0.5/s; both
    /// wrap at 16 and the negative frame state wraps +16 first.
    #[test]
    fn the_liquid_clock_is_wind_driven_and_wraps_at_sixteen() {
        let mut calm = LiquidAnimClock::default();
        // 1 s of calm air: 6 frames, waterfall 0.
        assert_eq!(calm.update(1.0, 0.0), (6, 0));
        // 2 more seconds: 18 -> %16 = 2; waterfall 1.5 -> 1.
        assert_eq!(calm.update(2.0, 0.0), (2, 1));
        // A full east storm: num = 1.24*25 = 31 -> +37/s.
        let mut storm = LiquidAnimClock::default();
        assert_eq!(storm.update(1.0, 1.24), (37 % 16, 0));
        // A west wind below -0.24 reverses: num = -0.5*25 = -12.5 -> -18.5/s;
        // from 0 the state goes negative and wraps +16: -18.5 + 16 = -2.5,
        // still negative — the C# adds 16 ONCE then %16 keeps the sign
        // (C# % keeps the dividend's sign): -2.5 % 16 = -2.5, (int) = -2.
        let mut west = LiquidAnimClock::default();
        let (f, _) = west.update(1.0, -0.5);
        assert_eq!(f, -2, "the C# single +16 wrap keeps a negative remainder");
        // The waterfall clock ignores wind entirely.
        let mut wf = LiquidAnimClock::default();
        let (_, a) = wf.update(4.0, 1.24);
        let (_, b) = wf.update(4.0, -1.24);
        assert_eq!((a, b), (2, 4), "0.5 frames/s regardless of wind");
    }
}
