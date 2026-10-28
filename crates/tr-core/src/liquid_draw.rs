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
            c.is_half_brick =
                t.half_brick() && above_has_liquid && !td::SET_PLATFORMS.has(t.kind);
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
                    cache[j].visible_liquid_level =
                        cache[j].visible_liquid_level.max(c.visible_liquid_level * fade);
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
            if fy == 16
                && (cache[i].has_left_edge ^ cache[i].has_right_edge)
                && world_y % 2 == 0
            {
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
                        (cache[i].bottom_wall * 2.0 + left.bottom_wall + right.bottom_wall)
                            * 0.25;
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
                cache[i].frame_y =
                    48 + 4.max((16.0 - left.visible_top_wall * 16.0) as i32) - 4;
                cache[i].visible_left_wall = 0.0;
                cache[i].visible_top_wall = 0.0;
                cache[i].visible_right_wall = 1.0;
                cache[i].visible_bottom_wall = 1.0;
            } else if right.has_top_edge && above.has_right_edge {
                cache[i].frame_x = 32 - 16.min((above.visible_right_wall * 16.0) as i32 - 4);
                cache[i].frame_y =
                    48 + 4.max((16.0 - right.visible_top_wall * 16.0) as i32) - 4;
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
            let is_surface = c.frame_x == 16
                && c.frame_y == 0
                && world_y as f64 > world.world_surface - 40.0;
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
            });
        }
    }
    out
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
}
