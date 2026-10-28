//! Tile lighting: a port of `Terraria.Graphics.Light.TileLightScanner` (seed light per
//! tile) and `LightMap` (directional blur that spreads it, attenuated by tile opacity).
//!
//! The original computes lighting for the visible screen area inflated by 28 tiles and
//! blurs it in two passes of four line sweeps. We do the same on demand for the rect the
//! renderer is about to draw, so panning never shows unlit tiles.

use crate::tile::*;
use crate::tiledata as td;
use crate::world::World;

/// Extra tiles around the visible rect (`LightingEngine.AREA_PADDING`); the blur pulls
/// light in from outside the screen, so the border must not be visible.
pub const AREA_PADDING: i32 = 28;
/// Edge rows/columns that are not blurred (`LightingEngine.NON_VISIBLE_PADDING`).
const NON_VISIBLE_PADDING: i32 = 18;

/// Light carried through a tile; matches `LightMaskMode`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mask {
    Air,
    Solid,
    CrackedBricks,
    Water,
    Honey,
}

impl Mask {
    #[inline]
    fn decay(self) -> [f32; 3] {
        match self {
            Mask::Air => [0.91; 3],
            Mask::Solid => [0.56; 3],
            Mask::CrackedBricks => [0.8; 3],
            // `LightDecayThroughWater` for the default water style.
            Mask::Water => [0.88 * 0.91, 0.96 * 0.91, 1.015 * 0.91],
            Mask::Honey => [0.75 * 0.91, 0.7 * 0.91, 0.6 * 0.91],
        }
    }
}

/// Ambient parameters for one computation.
#[derive(Clone, Copy, Debug)]
pub struct LightParams {
    /// `Main.tileColor`: surface sunlight (derived from the sky colour).
    pub sky: [f32; 3],
    /// `0.55 + sin(t * 2) * 0.08`: the underworld glow.
    pub hell_glow: f32,
    /// Multiplier for emissive tiles (torch flicker).
    pub torch_flicker: f32,
}

impl Default for LightParams {
    fn default() -> Self {
        Self {
            sky: [1.0, 1.0, 1.0],
            hell_glow: 0.55,
            torch_flicker: 1.0,
        }
    }
}

/// A rectangle of per-tile light colours, in world tile coordinates.
pub struct LightMap {
    pub x0: i32,
    pub y0: i32,
    pub w: i32,
    pub h: i32,
    colors: Vec<[f32; 3]>,
    mask: Vec<Mask>,
    /// Tiles that actually received light (debug/statistics).
    pub lit_tiles: u32,
}

impl Default for LightMap {
    fn default() -> Self {
        Self::new()
    }
}

impl LightMap {
    pub fn new() -> Self {
        Self {
            x0: 0,
            y0: 0,
            w: 0,
            h: 0,
            colors: Vec::new(),
            mask: Vec::new(),
            lit_tiles: 0,
        }
    }

    /// Light colour for a tile. Outside the computed rect the world is unlit.
    #[inline]
    pub fn get(&self, x: i32, y: i32) -> [f32; 3] {
        let lx = x - self.x0;
        let ly = y - self.y0;
        if lx < 0 || ly < 0 || lx >= self.w || ly >= self.h {
            return [0.0; 3];
        }
        self.colors[(ly * self.w + lx) as usize]
    }

    /// Light for a point in world pixels, sampling the tile that contains it.
    #[inline]
    pub fn get_px(&self, wx: f32, wy: f32, tile_px: f32) -> [f32; 3] {
        self.get((wx / tile_px).floor() as i32, (wy / tile_px).floor() as i32)
    }

    /// Average brightness across the map (debug overlay).
    pub fn mean_brightness(&self) -> f32 {
        if self.colors.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.colors.iter().map(|c| (c[0] + c[1] + c[2]) / 3.0).sum();
        sum / self.colors.len() as f32
    }

    /// Computes light for the tiles in `rect` (inclusive tile bounds), expanding it by
    /// `AREA_PADDING` so the blurred result has no visible border.
    pub fn compute(
        world: &World,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        params: &LightParams,
    ) -> Self {
        let mut map = Self::new();
        map.compute_into(world, x0, y0, x1, y1, params);
        map
    }

    /// Same as [`LightMap::compute`], but reuses the colour/mask storage (D1). The
    /// per-frame cost drops to zeroing the vectors instead of allocating 1.7 MB.
    pub fn compute_into(
        &mut self,
        world: &World,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        params: &LightParams,
    ) {
        let rx0 = (x0 - AREA_PADDING).max(0);
        let ry0 = (y0 - AREA_PADDING).max(0);
        let rx1 = (x1 + AREA_PADDING).min(world.width - 1);
        let ry1 = (y1 + AREA_PADDING).min(world.height - 1);
        let w = (rx1 - rx0 + 1).max(0);
        let h = (ry1 - ry0 + 1).max(0);
        let n = (w * h).max(0) as usize;

        self.x0 = rx0;
        self.y0 = ry0;
        self.w = w;
        self.h = h;
        self.lit_tiles = 0;
        self.colors.clear();
        self.colors.resize(n, [0.0; 3]);
        self.mask.clear();
        self.mask.resize(n, Mask::Air);

        for ly in 0..h {
            for lx in 0..w {
                let x = rx0 + lx;
                let y = ry0 + ly;
                let i = (ly * w + lx) as usize;
                let Some(tile) = world.tile(x, y) else {
                    continue;
                };
                self.mask[i] = mask_of(tile);
                let c = seed(world, tile, x, y, params);
                if c[0] > 0.0 || c[1] > 0.0 || c[2] > 0.0 {
                    self.lit_tiles += 1;
                }
                self.colors[i] = c;
            }
        }

        self.blur();
        self.blur();
    }

    fn idx(&self, x: i32, y: i32) -> usize {
        (y * self.w + x) as usize
    }

    /// `LightMap.Blur`: two passes over four line sweeps.
    fn blur(&mut self) {
        let (w, h, pad) = (
            self.w,
            self.h,
            NON_VISIBLE_PADDING.min(self.w.min(self.h) / 3),
        );

        for x in 0..w {
            // top -> bottom, then bottom -> top (stopping at the padding).
            self.blur_line(x, 0, 0, 1, h - pad);
            self.blur_line(x, h - 1, 0, -1, h - pad);
        }
        for y in 0..h {
            self.blur_line(0, y, 1, 0, w - pad);
            self.blur_line(w - 1, y, -1, 0, w - pad);
        }
    }

    /// One directional sweep that carries the brightest light seen so far, decaying it
    /// by the tile's opacity (`LightMap.BlurLine`).
    fn blur_line(&mut self, x0: i32, y0: i32, dx: i32, dy: i32, count: i32) {
        let mut carried = [0.0f32; 3];
        let mut done = [false; 3];
        for k in 0..count.max(0) {
            let (x, y) = (x0 + dx * k, y0 + dy * k);
            let i = self.idx(x, y);
            let mask = self.mask[i];
            let decay = mask.decay();
            // The original jitters water decay slightly per cell; a stable hash keeps
            // the frame identical when the camera does not move.
            let jitter = if mask == Mask::Water {
                let j = (i as u32).wrapping_mul(2_654_435_761) % 15;
                0.985 + j as f32 * 0.001
            } else {
                1.0
            };
            let color = &mut self.colors[i];

            for c in 0..3 {
                if carried[c] < color[c] {
                    carried[c] = color[c];
                    done[c] = false;
                } else if !done[c] {
                    if carried[c] < 0.0185 {
                        done[c] = true;
                    } else {
                        color[c] = carried[c];
                    }
                }
            }
            if done[0] && done[1] && done[2] {
                continue;
            }
            for c in 0..3 {
                if !done[c] {
                    carried[c] *= decay[c] * jitter;
                }
            }
        }
    }
}

/// `TileLightScanner.GetTileMask`.
fn mask_of(tile: &Tile) -> Mask {
    if light_blocked(tile) && tile.kind != 131 && tile.slope == 0 {
        if td::SET_CRACKED_BRICKS.has(tile.kind) {
            return Mask::CrackedBricks;
        }
        return Mask::Solid;
    }
    if tile.liquid > 128 && tile.liquid_kind() != LiquidKind::Lava {
        return match tile.liquid_kind() {
            LiquidKind::Honey => Mask::Honey,
            _ => Mask::Water,
        };
    }
    Mask::Air
}

fn light_blocked(tile: &Tile) -> bool {
    tile.active() && td::TILE_BLOCK_LIGHT.has(tile.kind)
}

/// `TileLightScanner.GetTileLight`.
fn seed(world: &World, tile: &Tile, x: i32, y: i32, p: &LightParams) -> [f32; 3] {
    let mut c = [0.0f32; 3];
    if y as f64 <= world.world_surface {
        surface_light(world, tile, x, y, p, &mut c);
    } else if y > world.height - 200 {
        hell_light(world, tile, x, y, p, &mut c);
    }
    wall_light(tile, &mut c);
    if tile.active() {
        tile_light(tile, x, y, p, &mut c);
    }
    liquid_light(tile, &mut c);
    c
}

fn wall_is_light(wall: u16) -> bool {
    wall < td::WALL_COUNT as u16 && td::WALL_LIGHT[wall as usize] || wall == 73 || wall == 227
}

/// `TileLightScanner.ApplySurfaceLight` (without the shimmer/water-style extras).
fn surface_light(world: &World, tile: &Tile, x: i32, y: i32, p: &LightParams, c: &mut [f32; 3]) {
    let avg = (p.sky[0] + p.sky[1] + p.sky[2]) / 3.0;
    let mut light = [0.0f32; 3];

    let open_sides = |dx: i32, dy: i32| world.tile(x + dx, y + dy).map_or(true, |t| t.liquid == 0);
    let sloped = tile.slope != 0 || tile.half_brick();
    let transmits = !tile.active()
        || !td::TILE_NO_SUN_LIGHT.has(tile.kind)
        || (sloped
            && open_sides(-1, 0)
            && open_sides(1, 0)
            && open_sides(0, -1)
            && open_sides(0, 1));

    if c[0] < avg && wall_is_light(tile.wall) {
        if transmits {
            if tile.liquid < 200 {
                if !tile.half_brick() || world.tile(x, y - 1).map_or(true, |t| t.liquid < 200) {
                    light = p.sky;
                }
            }
        }
    }

    if ((!tile.active() || tile.half_brick() || !td::TILE_NO_SUN_LIGHT.has(tile.kind))
        && ((88..=93).contains(&tile.wall) || tile.wall == 241))
        && tile.liquid < 255
    {
        // Gem walls glow in their own colour at the surface.
        let mut gem = [p.sky[0], p.sky[1], p.sky[2]];
        let which = if tile.wall == 241 {
            6
        } else {
            tile.wall as i32 - 88
        };
        let k = match which {
            0 => [0.9, 0.15, 0.9],
            1 => [0.9, 0.9, 0.15],
            2 => [0.15, 0.15, 0.9],
            3 => [0.15, 0.9, 0.15],
            4 => [0.9, 0.15, 0.15],
            5 => [0.82, 0.82, 0.82],
            _ => [0.9, 0.5, 0.0],
        };
        for i in 0..3 {
            gem[i] *= k[i];
        }
        light = gem;
    }

    for i in 0..3 {
        if c[i] < light[i] {
            c[i] = light[i];
        }
    }
}

/// `TileLightScanner.ApplyHellLight`.
fn hell_light(world: &World, tile: &Tile, x: i32, y: i32, p: &LightParams, c: &mut [f32; 3]) {
    let glow = p.hell_glow;
    let mut light = [0.0f32; 3];
    let open_sides = |dx: i32, dy: i32| world.tile(x + dx, y + dy).map_or(true, |t| t.liquid == 0);
    let sloped = tile.slope != 0 || tile.half_brick();
    let transmits = !tile.active()
        || !td::TILE_NO_SUN_LIGHT.has(tile.kind)
        || (sloped
            && open_sides(-1, 0)
            && open_sides(1, 0)
            && open_sides(0, -1)
            && open_sides(0, 1));
    if transmits
        && c[0] < glow
        && wall_is_light(tile.wall)
        && tile.liquid < 200
        && (!tile.half_brick() || world.tile(x, y - 1).map_or(true, |t| t.liquid < 200))
    {
        light = [glow, glow * 0.6, glow * 0.2];
    }
    for i in 0..3 {
        if c[i] < light[i] {
            c[i] = light[i];
        }
    }
}

/// `TileLightScanner.ApplyWallLight`: glowing walls (crystal, gem, living fire...).
fn wall_light(tile: &Tile, c: &mut [f32; 3]) {
    let blocked = light_blocked(tile);
    let light: [f32; 3] = match tile.wall {
        357 if !blocked => [0.15, 0.27, 0.3],
        182 if !blocked => [0.24, 0.12, 0.09],
        33 if !blocked => [0.09, 0.0525, 0.24],
        174 if !blocked => [0.2975, 0.0, 0.0],
        175 if !blocked => [0.075, 0.15, 0.4],
        176 if !blocked => [0.1, 0.1, 0.1],
        137 if !blocked => [0.4, 0.2, 0.04],
        44 if !blocked => [0.15, 0.15, 0.15],
        154 => [0.6, 0.0, 0.6],
        166 => [0.6, 0.6, 0.0],
        165 => [0.0, 0.0, 0.6],
        156 => [0.0, 0.6, 0.0],
        164 => [0.6, 0.0, 0.0],
        155 => [0.6, 0.6, 0.6],
        153 => [0.6, 0.3, 0.0],
        341 if !blocked => [0.25, 0.1, 0.0],
        343 if !blocked => [0.0, 0.25, 0.0],
        344 if !blocked => [0.0, 0.16, 0.34],
        342 if !blocked => [0.3, 0.0, 0.17],
        345 if !blocked => [0.3, 0.0, 0.35],
        346 if !blocked => [0.25, 0.25, 0.25],
        _ => [0.0; 3],
    };
    for i in 0..3 {
        if c[i] < light[i] {
            c[i] = light[i];
        }
    }
}

/// `TorchID` base colours (same order as `TorchID.Torch` … `TorchID.Shimmer`).
const TORCH_LIGHT: [[f32; 3]; 24] = [
    [1.0, 0.95, 0.8],
    [0.0, 0.1, 1.3],
    [1.0, 0.1, 0.1],
    [0.0, 1.0, 0.1],
    [0.9, 0.0, 0.9],
    [1.4, 1.4, 1.4],
    [0.9, 0.9, 0.0],
    [0.8, 0.35, 0.35],
    [1.0, 1.6, 0.5],
    [0.75, 0.85, 1.4],
    [1.0, 0.5, 0.0],
    [1.4, 1.4, 0.7],
    [0.75, 1.35, 1.5],
    [0.95, 0.75, 1.3],
    [0.85, 0.85, 0.85],
    [1.0, 0.0, 1.0],
    [1.4, 0.85, 0.55],
    [0.25, 1.3, 0.8],
    [0.95, 0.4, 1.4],
    [1.4, 0.7, 0.5],
    [1.25, 0.6, 1.2],
    [0.75, 1.45, 0.9],
    [0.3, 0.78, 1.2],
    [0.7, 0.55, 0.9],
];

/// `TileLightScanner.ApplyTileLight`, using the generated constant table plus the
/// frame-dependent torch/campfire cases that the table cannot express.
fn tile_light(tile: &Tile, x: i32, y: i32, p: &LightParams, c: &mut [f32; 3]) {
    let mut light = if (tile.kind as usize) < td::TILE_COUNT {
        td::EMISSIVE[tile.kind as usize]
    } else {
        [0.0; 3]
    };

    match tile.kind {
        // 4: torch. `frameY / 22` selects the torch colour (18 px layout).
        4 if tile.frame_x >= 0 && tile.frame_x < 66 => {
            let id = (tile.frame_y.max(0) as usize) / 22;
            light = TORCH_LIGHT[id.min(TORCH_LIGHT.len() - 1)];
        }
        // 33: candles use the same colour ramp.
        33 if tile.frame_x == 0 => {
            let id = (tile.frame_y.max(0) as usize) / 22;
            light = TORCH_LIGHT[id.min(TORCH_LIGHT.len() - 1)];
        }
        // 215: campfire, lit only in its top frame.
        215 if tile.frame_y < 36 => {
            light = [0.95, 0.65, 0.3];
        }
        _ => {}
    }

    if td::SET_TORCHES.has(tile.kind) && light == [0.0; 3] {
        light = TORCH_LIGHT[0];
    }

    let flicker = if light != [0.0; 3] {
        // `Main.mouseTextColor` pulses the light in the original; approximate it with
        // a slow brightness wobble unless the caller pins it.
        p.torch_flicker
    } else {
        1.0
    };
    let _ = (x, y);
    for i in 0..3 {
        let v = light[i] * flicker;
        if c[i] < v {
            c[i] = v;
        }
    }
}

/// `TileLightScanner.ApplyLiquidLight`: lava and shimmer emit light.
fn liquid_light(tile: &Tile, c: &mut [f32; 3]) {
    if tile.liquid == 0 {
        return;
    }
    let light = match tile.liquid_kind() {
        LiquidKind::Lava => [0.55, 0.55 * 0.6, 0.55 * 0.2],
        LiquidKind::Shimmer => [0.7 * 0.6, 0.7 * 0.25, 0.7 * 0.9],
        _ => return,
    };
    for i in 0..3 {
        if c[i] < light[i] {
            c[i] = light[i];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i].kind = kind;
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    #[test]
    fn open_sky_is_lit() {
        let mut w = World::new("t".into(), 64, 400);
        w.world_surface = 10.0;
        let lm = LightMap::compute(&w, 20, 4, 30, 8, &LightParams::default());
        let c = lm.get(25, 6);
        assert!(c[0] > 0.95, "sky tile should be fully lit, got {c:?}");
    }

    #[test]
    fn solid_ground_shadows_below() {
        let mut w = World::new("t".into(), 64, 400);
        w.world_surface = 10.0;
        for x in 0..64 {
            for y in 10..14 {
                set(&mut w, x, y, 0); // dirt
            }
        }
        let lm = LightMap::compute(&w, 20, 4, 30, 20, &LightParams::default());
        let above = lm.get(25, 9);
        let below = lm.get(25, 16);
        assert!(above[0] > 0.9, "above ground {above:?}");
        assert!(
            below[0] < above[0] * 0.5,
            "below ground {below:?} vs {above:?}"
        );
    }

    #[test]
    fn torch_lights_the_room() {
        let mut w = World::new("t".into(), 64, 400);
        w.world_surface = 0.0; // no sunlight anywhere
        set(&mut w, 32, 32, 4);
        let i = w.index(32, 32);
        w.tiles[i].frame_x = 0;
        w.tiles[i].frame_y = 0;
        let lm = LightMap::compute(&w, 22, 22, 42, 42, &LightParams::default());
        let near = lm.get(32, 33);
        let far = lm.get(22, 42);
        assert!(near[0] > 0.5, "near torch {near:?}");
        assert!(near[0] > far[0], "torch should fade with distance");
    }

    #[test]
    fn lava_glows_and_hell_is_warm() {
        let mut w = World::new("t".into(), 64, 400);
        w.world_surface = 0.0;
        let i = w.index(30, 30);
        w.tiles[i].liquid = 255;
        w.tiles[i].flags |= FLAG_LIQUID_LAVA;
        let lm = LightMap::compute(&w, 24, 24, 38, 38, &LightParams::default());
        let c = lm.get(30, 31);
        assert!(c[0] > 0.3 && c[0] > c[2], "lava light {c:?}");
    }
}
