//! Tile lighting: a port of `Terraria.Graphics.Light.TileLightScanner` (seed light
//! per tile, see [`crate::tile_light`]) and `LightMap` (directional blur that
//! spreads it, attenuated by tile opacity).
//!
//! The original computes lighting for the visible screen area inflated by 28 tiles
//! and blurs it in two passes of four line sweeps. We do the same on demand for the
//! rect the renderer is about to draw, so panning never shows unlit tiles.
//!
//! [`LightEngine`] is the seam the renderer and simulation code talk through,
//! mirroring `Terraria.Graphics.Light.ILightingEngine`; [`LightMap`] is the built-in
//! engine behind it. The mode/style descriptors live in [`params`], the blur sweeps
//! and the tile-medium classification in [`blur`].

mod blur;
mod params;

use std::sync::OnceLock;

use crate::tile::*;
use crate::tile_light::{self, FastRandom};
use crate::world::World;
use rayon::prelude::*;

use blur::{mask_of, surface_cutoff, Mask};
use params::LightKey;
pub use params::{LightMode, LightParams, LightStyle, GLOBAL_BRIGHTNESS};

/// Extra tiles around the visible rect (`LightingEngine.AREA_PADDING`); the blur pulls
/// light in from outside the screen, so the border must not be visible.
pub const AREA_PADDING: i32 = 28;
/// D3: extra tiles cached around the visible rect. While the camera pans within this
/// margin the previous light map is reused without recomputing, and the visible area
/// always keeps at least `AREA_PADDING` of computed context around it.
pub const CACHE_MARGIN: i32 = 32;

/// Rects with at least this many cells seed and sweep rows on Rayon's persistent
/// global pool; smaller ones stay sequential. Measured on the reference laptop, task
/// dispatch makes the parallel path a net loss below ~100k cells (light 0.20 → 0.55 ms
/// at zoom 1.5), so real zooms (~20k cells max) keep the sequential path and only the
/// zoom-0.25 debug view (~128k cells) goes parallel.
const PAR_LIGHT_MIN_CELLS: usize = 96 * 1024;

/// The lighting seam, mirroring `Terraria.Graphics.Light.ILightingEngine`
/// (`ILightingEngine.cs:5-16`): the renderer only ever talks to this surface, so the
/// built-in blur engine can be swapped for another implementation the way the C#
/// constructor accepts any `ILightingEngine`. `Rebuild`/`AddLight`/`Clear` have no
/// counterpart here — the per-frame light list does not exist in this port.
pub trait LightEngine {
    /// The processed area in world tile coordinates as `(x0, y0, w, h)`
    /// (`LightingEngine._activeProcessedArea`); [`LightEngine::get_color`] is zero
    /// outside it.
    fn get_area(&self) -> (i32, i32, i32, i32);

    /// `ILightingEngine.GetColor` (`ILightingEngine.cs:13`) — light at a tile, zero
    /// outside the processed area.
    fn get_color(&self, x: i32, y: i32) -> [f32; 3];

    /// `ILightingEngine.ProcessArea` (`ILightingEngine.cs:11`) — make light
    /// available for the inclusive tile rect, reusing cached results when nothing
    /// that feeds the computation changed.
    fn compute(&mut self, world: &World, x0: i32, y0: i32, x1: i32, y1: i32, params: &LightParams);
}

impl LightEngine for LightMap {
    fn get_area(&self) -> (i32, i32, i32, i32) {
        (self.x0, self.y0, self.w, self.h)
    }

    fn get_color(&self, x: i32, y: i32) -> [f32; 3] {
        self.get(x, y)
    }

    fn compute(&mut self, world: &World, x0: i32, y0: i32, x1: i32, y1: i32, params: &LightParams) {
        self.compute_into(world, x0, y0, x1, y1, params);
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
    /// D2/D3: key of the last computation and the world rect whose values are valid
    /// (the computed rect shrunk by `AREA_PADDING` on every side).
    cache_key: Option<LightKey>,
    valid_x0: i32,
    valid_y0: i32,
    valid_x1: i32,
    valid_y1: i32,
    /// Incremented whenever the map is recomputed (not on cache hits); the world
    /// instance cache keys on it so reused tile colours are never stale.
    generation: u64,
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
            cache_key: None,
            valid_x0: 1,
            valid_y0: 1,
            valid_x1: 0,
            valid_y1: 0,
            generation: 0,
        }
    }

    pub fn generation(&self) -> u64 {
        self.generation
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

    /// `Lighting.GetColor9Slice`: nine 3x3 light samples averaged with the center
    /// tile. Slice `i` is indexed row-major (top-left first), exactly like the
    /// original's `slices` array, which the draw pass maps onto the 4/8/4 px quad
    /// grid of a solid block.
    pub fn slice9(&self, x: i32, y: i32) -> [[f32; 3]; 9] {
        let own = self.get(x, y);
        let mut out = [[0.0f32; 3]; 9];
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let n = self.get(x + dx, y + dy);
                let idx = ((dy + 1) * 3 + (dx + 1)) as usize;
                out[idx] = [
                    (own[0] + n[0]) * 0.5,
                    (own[1] + n[1]) * 0.5,
                    (own[2] + n[2]) * 0.5,
                ];
            }
        }
        out
    }

    /// `Lighting.GetCornerColors`: each corner averages this tile with its two
    /// orthogonal neighbours and the diagonal between them. Ordered TL, TR, BL, BR.
    /// The original clamps after applying `GlobalBrightness`; the draw pass does
    /// that once it has run the value through the light style.
    pub fn corner4(&self, x: i32, y: i32) -> [[f32; 3]; 4] {
        let c = self.get(x, y);
        let up = self.get(x, y - 1);
        let down = self.get(x, y + 1);
        let left = self.get(x - 1, y);
        let right = self.get(x + 1, y);
        let up_left = self.get(x - 1, y - 1);
        let up_right = self.get(x + 1, y - 1);
        let down_left = self.get(x - 1, y + 1);
        let down_right = self.get(x + 1, y + 1);
        let avg = |a: [f32; 3], b: [f32; 3], d: [f32; 3]| {
            [
                (c[0] + a[0] + b[0] + d[0]) * 0.25,
                (c[1] + a[1] + b[1] + d[1]) * 0.25,
                (c[2] + a[2] + b[2] + d[2]) * 0.25,
            ]
        };
        [
            avg(up, left, up_left),
            avg(up, right, up_right),
            avg(down, left, down_left),
            avg(down, right, down_right),
        ]
    }

    /// `Lighting.GetColor4Slice`: four quadrant samples. Each quadrant takes the
    /// **darker** of its two axis neighbours (`Lighting.cs:530-586`) averaged with
    /// this tile (`(slice + own) * 0.5`), matching the medium-quality block draw.
    pub fn quad4(&self, x: i32, y: i32) -> [[f32; 3]; 4] {
        let own = self.get(x, y);
        let up = self.get(x, y - 1);
        let down = self.get(x, y + 1);
        let left = self.get(x - 1, y);
        let right = self.get(x + 1, y);
        let sum = |c: [f32; 3]| c[0] + c[1] + c[2];
        // `if (up >= left) slice = left else slice = up`: the darker neighbour wins.
        let pick = |a: [f32; 3], b: [f32; 3]| if sum(a) >= sum(b) { b } else { a };
        let avg = |n: [f32; 3]| {
            [
                (own[0] + n[0]) * 0.5,
                (own[1] + n[1]) * 0.5,
                (own[2] + n[2]) * 0.5,
            ]
        };
        [
            avg(pick(up, left)),
            avg(pick(up, right)),
            avg(pick(down, left)),
            avg(pick(down, right)),
        ]
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
    ///
    /// D2: when the rect, ambient parameters and world revision match the previous
    /// call, the storage already holds the correct colours and the compute is a no-op.
    pub fn compute_into(
        &mut self,
        world: &World,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        params: &LightParams,
    ) {
        self.compute_into_impl(world, x0, y0, x1, y1, params, None, &[], 0);
    }

    /// `compute_into` plus `Lighting.AddLight` sources: `lights` are seeded as a
    /// per-cell maximum before the blur, so their glow spreads exactly like a
    /// tile seed. `lights_key` must change whenever the light set changes — it
    /// folds into the D2 cache key. Vanilla applies the waterfall lights one
    /// frame late (the engine keeps the previous frame's `AddLight` list when
    /// `LightTiles` runs), and the caller here does the same.
    pub fn compute_into_with_lights(
        &mut self,
        world: &World,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        params: &LightParams,
        lights: &[DynamicLight],
        lights_key: u64,
    ) {
        self.compute_into_impl(world, x0, y0, x1, y1, params, None, lights, lights_key);
    }

    /// Test hook: forces the sequential or parallel path regardless of rect size.
    #[cfg(test)]
    fn compute_into_with(
        &mut self,
        world: &World,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        params: &LightParams,
        parallel: bool,
    ) {
        self.compute_into_impl(world, x0, y0, x1, y1, params, Some(parallel), &[], 0);
    }

    #[allow(clippy::too_many_arguments)]
    fn compute_into_impl(
        &mut self,
        world: &World,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        params: &LightParams,
        force_parallel: Option<bool>,
        lights: &[DynamicLight],
        lights_key: u64,
    ) {
        let mut key = params.key(world.revision);
        key.lights = lights_key;
        // `TR_NOCACHE=1` forces the full compute every frame so benchmarks can
        // measure the uncached path (D2 is otherwise almost always a hit).
        static NO_CACHE: OnceLock<bool> = OnceLock::new();
        let no_cache = *NO_CACHE.get_or_init(|| std::env::var_os("TR_NOCACHE").is_some());
        // D3: the cached map covers the previous visible rect plus CACHE_MARGIN; a
        // camera pan that stays inside that valid region reuses it untouched.
        if !no_cache
            && self.cache_key == Some(key)
            && x0 >= self.valid_x0
            && y0 >= self.valid_y0
            && x1 <= self.valid_x1
            && y1 <= self.valid_y1
        {
            return;
        }
        self.cache_key = Some(key);
        self.generation = self.generation.wrapping_add(1);

        let rx0 = (x0 - AREA_PADDING - CACHE_MARGIN).max(0);
        let ry0 = (y0 - AREA_PADDING - CACHE_MARGIN).max(0);
        let rx1 = (x1 + AREA_PADDING + CACHE_MARGIN).min(world.width - 1);
        let ry1 = (y1 + AREA_PADDING + CACHE_MARGIN).min(world.height - 1);
        self.valid_x0 = rx0 + AREA_PADDING;
        self.valid_y0 = ry0 + AREA_PADDING;
        self.valid_x1 = (rx1 - AREA_PADDING).max(self.valid_x0 - 1);
        self.valid_y1 = (ry1 - AREA_PADDING).max(self.valid_y0 - 1);
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
        if w <= 0 || h <= 0 {
            return;
        }

        // The surface cutoff of a column is the top of its terrain *or* its topmost
        // liquid. A lakebed is under water, so it must not receive a direct sky
        // seed; light reaches it through the water with the water decay instead.
        let cutoffs: Vec<i32> = (rx0..=rx1).map(|x| surface_cutoff(world, x)).collect();

        // C1/C3: seeding goes to Rayon's persistent global pool once the rect is
        // large enough to amortise task dispatch; see `PAR_LIGHT_MIN_CELLS`.
        let wu = w as usize;
        let width = world.width as usize;
        let parallel = force_parallel.unwrap_or(n >= PAR_LIGHT_MIN_CELLS);
        self.lit_tiles = if parallel {
            self.colors
                .par_chunks_mut(wu)
                .zip(self.mask.par_chunks_mut(wu))
                .enumerate()
                .map(|(ly, (colors_row, mask_row))| {
                    seed_row_range(
                        world,
                        rx0,
                        ry0 + ly as i32,
                        wu,
                        width,
                        params,
                        &cutoffs,
                        colors_row,
                        mask_row,
                    )
                })
                .sum()
        } else {
            self.colors
                .chunks_mut(wu)
                .zip(self.mask.chunks_mut(wu))
                .enumerate()
                .map(|(ly, (colors_row, mask_row))| {
                    seed_row_range(
                        world,
                        rx0,
                        ry0 + ly as i32,
                        wu,
                        width,
                        params,
                        &cutoffs,
                        colors_row,
                        mask_row,
                    )
                })
                .sum()
        };

        // `Lighting.AddLight`: dynamic sources take the maximum against the tile
        // seeds before the blur spreads them (the engine's per-tile light is the
        // maximum over every reference that reaches it).
        for l in lights {
            let (lx, ly) = (l.x - rx0, l.y - ry0);
            if lx < 0 || ly < 0 || lx >= w || ly >= h {
                continue;
            }
            let idx = (ly * w + lx) as usize;
            let c = &mut self.colors[idx];
            c[0] = c[0].max(l.rgb[0]);
            c[1] = c[1].max(l.rgb[1]);
            c[2] = c[2].max(l.rgb[2]);
        }

        blur::blur(self, parallel);
        blur::blur(self, parallel);
    }
}

/// One `Lighting.AddLight(x, y, r, g, b)` source (`Terraria.Lighting.AddLight`):
/// a world-tile position and the additive colour it contributes. Seeded as a
/// per-cell maximum before the blur, the way the original engine takes the
/// maximum of the seed and every light reference that reaches the tile.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DynamicLight {
    pub x: i32,
    pub y: i32,
    pub rgb: [f32; 3],
}

/// Seeds one row of the light map (`TileLightScanner` per tile). Returns how many
/// tiles received light. Rows never share cells, so this is callable from Rayon tasks.
#[allow(clippy::too_many_arguments)]
fn seed_row_range(
    world: &World,
    rx0: i32,
    y: i32,
    wu: usize,
    width: usize,
    params: &LightParams,
    cutoffs: &[i32],
    colors_row: &mut [[f32; 3]],
    mask_row: &mut [Mask],
) -> u32 {
    let row = y as usize * width;
    let mut lit = 0u32;
    for lx in 0..wu {
        let x = rx0 + lx as i32;
        let tile = &world.tiles[row + x as usize];
        mask_row[lx] = mask_of(tile, x, y, params.draw_invisible_walls, world);
        let c = seed(world, tile, x, y, params, cutoffs[lx]);
        if c[0] > 0.0 || c[1] > 0.0 || c[2] > 0.0 {
            lit += 1;
        }
        colors_row[lx] = c;
    }
    lit
}

/// `TileLightScanner.GetTileLight` (`TileLightScanner.cs:87`): surface/hell ambient,
/// walls, the self-lit tile types and liquids, each taking the maximum.
fn seed(
    world: &World,
    tile: &Tile,
    x: i32,
    y: i32,
    p: &LightParams,
    local_surface: i32,
) -> [f32; 3] {
    let mut c = [0.0f32; 3];
    // The Echo bits off `World.extras` (vanilla: the `Tile` header flags).
    let inv = tile_light::InvisibleBits {
        block: world.invisible_block(x, y),
        wall: world.invisible_wall(x, y),
    };
    // `TileLightScanner._random.WithModifier(x, y)` (`TileLightScanner.cs:91`): one
    // stream per tile.
    let mut rng = FastRandom::new(p.rng_seed).with_modifier(x, y);
    // The original seeds surface light up to the global `Main.worldSurface`. That
    // leaves hills above the average bright and valleys below it unlit. Seeding up to
    // each column's own surface keeps every slope in sunlight; the light map still
    // stops at solid blocks, so caves stay dark.
    if y <= local_surface {
        tile_light::apply_surface_light(world, tile, x, y, &mut rng, p, inv, &mut c);
    } else if y > world.height - 200 {
        tile_light::apply_hell_light(world, tile, x, y, p, inv, &mut c);
    }
    tile_light::apply_wall_light(tile, world.wall_paint(x, y), inv, &mut rng, p, &mut c);
    if tile.active() {
        tile_light::apply_tile_light(world, tile, x, y, &mut rng, p, &mut c);
    }
    liquid_light(tile, &mut c, p);
    c
}

/// `TileLightScanner.ApplyLiquidLight` (`TileLightScanner.cs:108`): lava and shimmer
/// emit light. The original compares every channel against the unscaled base and
/// then stores the scaled value, which is why the green channel can end up brighter
/// than the test.
fn liquid_light(tile: &Tile, c: &mut [f32; 3], p: &LightParams) {
    if tile.liquid == 0 {
        return;
    }
    let delta = (270.0 - p.mouse_text_color as f32) / 900.0;
    match tile.liquid_kind() {
        LiquidKind::Lava => {
            let n = 0.55 + delta;
            if c[0] < n {
                c[0] = n;
            }
            if c[1] < n {
                c[1] = n * 0.6;
            }
            if c[2] < n {
                c[2] = n * 0.2;
            }
        }
        LiquidKind::Shimmer => {
            let n2 = 0.7 + delta;
            let n3 = 0.7 + delta * (900.0 / 125.0);
            if c[0] < n2 {
                c[0] = n2 * 0.6;
            }
            if c[1] < n3 {
                c[1] = n3 * 0.25;
            }
            if c[2] < n2 {
                c[2] = n2 * 0.9;
            }
        }
        _ => {}
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
    fn parallel_light_is_bit_identical_to_sequential() {
        let mut w = World::new("par".into(), 420, 420);
        w.world_surface = 120.0;
        for y in 1..419 {
            for x in 1..419 {
                let i = w.index(x, y);
                let n = (x * 31 + y * 17).rem_euclid(113);
                if n < 70 {
                    w.tiles[i].kind = [0u16, 1, 2, 4, 30, 51][(n % 6) as usize];
                    w.tiles[i].flags |= FLAG_ACTIVE;
                }
                if (x * 7 + y * 11) % 23 == 0 {
                    w.tiles[i].wall = 2;
                }
                if (x * 5 + y * 3) % 211 == 0 {
                    w.tiles[i].liquid = 255;
                    if n % 2 == 0 {
                        w.tiles[i].flags |= FLAG_LIQUID_LAVA;
                    }
                }
            }
        }
        let params = LightParams::default();
        let mut seq = LightMap::new();
        seq.compute_into_with(&w, 20, 20, 400, 400, &params, false);
        let mut par = LightMap::new();
        par.compute_into_with(&w, 20, 20, 400, 400, &params, true);

        assert_eq!((seq.w, seq.h), (par.w, par.h));
        assert_eq!(seq.lit_tiles, par.lit_tiles);
        assert_eq!(seq.colors, par.colors, "parallel light diverged");
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

    /// The M1.5-LIGHT-3 wall: `LightIsBlocked` (`TileLightScanner.cs:151-162`)
    /// treats an `invisibleBlock` tile as blocking only while invisible blocks
    /// are shown; hidden, light passes through — pinned at the mask level, the
    /// exact spot the vanilla branch lives in (`GetTileMask` → `Solid`).
    #[test]
    fn invisible_block_is_transparent_when_hidden() {
        let mut w = World::new("t".into(), 8, 8);
        let i = w.index(4, 4);
        w.tiles[i].kind = 1; // dirt: in TILE_BLOCK_LIGHT
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.set_invisible_block(4, 4, true);
        assert!(
            blur::mask_of(&w.tiles[i], 4, 4, false, &w) != blur::Mask::Solid,
            "hidden invisible block must not form the Solid mask"
        );
        assert_eq!(
            blur::mask_of(&w.tiles[i], 4, 4, true, &w),
            blur::Mask::Solid,
            "shown invisible block keeps blocking"
        );
        // The visible twin keeps blocking regardless of the flag.
        let j = w.index(5, 4);
        w.tiles[j].kind = 1;
        w.tiles[j].flags |= FLAG_ACTIVE;
        assert_eq!(
            blur::mask_of(&w.tiles[j], 5, 4, false, &w),
            blur::Mask::Solid
        );
    }

    /// `TileLightScanner.cs:3149`: a hidden invisible sun-blocking tile still
    /// receives the sun seed when the exemption's four neighbours are dry.
    /// Pinned at the seed level (`apply_surface_light`) — the blur would flood
    /// the probe from its neighbours and hide the gate.
    #[test]
    fn hidden_invisible_block_still_receives_sun() {
        let mut w = World::new("t".into(), 8, 64);
        w.world_surface = 30.0;
        // Kind 11 is in `TILE_NO_SUN_LIGHT`; no wall, dry neighbours.
        let i = w.index(4, 20);
        w.tiles[i].kind = 11;
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.set_invisible_block(4, 20, true);
        let mut c = [0.0f32; 3];
        let mut rng = FastRandom::new(0).with_modifier(4, 20);
        tile_light::apply_surface_light(
            &w,
            &w.tiles[i],
            4,
            20,
            &mut rng,
            &LightParams::default(),
            tile_light::InvisibleBits {
                block: true,
                wall: false,
            },
            &mut c,
        );
        assert!(c[0] > 0.5, "hidden invisible block receives sun: {c:?}");

        // The visible twin gets no direct sun (the type is in the no-sun set).
        let mut c2 = [0.0f32; 3];
        let mut rng2 = FastRandom::new(0).with_modifier(4, 20);
        tile_light::apply_surface_light(
            &w,
            &w.tiles[i],
            4,
            20,
            &mut rng2,
            &LightParams::default(),
            tile_light::InvisibleBits::default(),
            &mut c2,
        );
        assert_eq!(c2, [0.0; 3], "visible no-sun tile gets no seed");
    }

    /// `TileLightScanner.cs:3149`: a hidden invisibleWall qualifies for the sun
    /// seed like a light-passing wall; shown, it behaves like a normal wall.
    #[test]
    fn hidden_invisible_wall_admits_sun() {
        let mut w = World::new("t".into(), 8, 64);
        w.world_surface = 30.0;
        let i = w.index(4, 20);
        w.tiles[i].wall = 1; // dirt wall: not in WALL_LIGHT
        let inv_hidden = tile_light::InvisibleBits {
            block: false,
            wall: true,
        };
        let mut c = [0.0f32; 3];
        let mut rng = FastRandom::new(0).with_modifier(4, 20);
        tile_light::apply_surface_light(
            &w,
            &w.tiles[i],
            4,
            20,
            &mut rng,
            &LightParams::default(),
            inv_hidden,
            &mut c,
        );
        assert!(c[0] > 0.5, "hidden invisible wall admits sun: {c:?}");

        // Shown (or not invisible at all), the non-light wall blocks the seed.
        let mut c2 = [0.0f32; 3];
        let mut rng2 = FastRandom::new(0).with_modifier(4, 20);
        let shown_params = LightParams {
            draw_invisible_walls: true,
            ..LightParams::default()
        };
        tile_light::apply_surface_light(
            &w,
            &w.tiles[i],
            4,
            20,
            &mut rng2,
            &shown_params,
            inv_hidden,
            &mut c2,
        );
        assert_eq!(c2, [0.0; 3], "shown invisible wall blocks the seed");
    }

    #[test]
    fn nine_slice_averages_with_neighbours() {
        let mut w = World::new("t".into(), 32, 32);
        w.world_surface = 0.0;
        for y in 0..32 {
            for x in 0..32 {
                let i = w.index(x, y);
                w.tiles[i].wall = 1; // no light
            }
        }
        let i = w.index(10, 10);
        w.tiles[i].kind = 4; // torch seeds a tile-local light
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.tiles[i].frame_x = 0;
        w.tiles[i].frame_y = 0;
        let lm = LightMap::compute(&w, 4, 4, 26, 26, &LightParams::default());
        let own = lm.get(10, 10);
        let right = lm.get(11, 10);
        let s = lm.slice9(10, 10);
        // Center slice is the tile itself, right edge sits between the two.
        assert!((s[4][0] - own[0]).abs() < 1e-6, "center {:?}", s[4]);
        assert!(s[5][0] < s[4][0] && s[5][0] > right[0], "right edge {s:?}");
    }

    #[test]
    fn panning_within_the_margin_reuses_the_cache() {
        let mut w = World::new("t".into(), 256, 256);
        for x in 0..256 {
            for y in 140..200 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= crate::tile::FLAG_ACTIVE;
            }
        }
        let torch = w.index(120, 138);
        w.tiles[torch].kind = 4;
        w.tiles[torch].flags |= crate::tile::FLAG_ACTIVE;
        let params = LightParams {
            sky: [0.7; 3],
            ..Default::default()
        };
        let mut map = LightMap::new();
        map.compute_into(&w, 80, 110, 160, 170, &params);
        let rect = (map.x0, map.y0, map.w, map.h);
        map.compute_into(&w, 84, 110, 164, 170, &params);
        assert_eq!(
            (map.x0, map.y0, map.w, map.h),
            rect,
            "a small pan must reuse the cached map"
        );
        // The visible area keeps the same values as a fresh computation.
        let fresh = LightMap::compute(&w, 84, 110, 164, 170, &params);
        for y in 110..=170 {
            for x in 84..=164 {
                assert_eq!(map.get(x, y), fresh.get(x, y), "light changed at {x},{y}");
            }
        }
    }

    #[test]
    fn light_engine_seam_matches_the_map() {
        let mut w = World::new("t".into(), 64, 400);
        w.world_surface = 10.0;
        let params = LightParams::default();
        let direct = LightMap::compute(&w, 20, 4, 30, 8, &params);
        let mut engine: Box<dyn LightEngine> = Box::new(LightMap::new());
        engine.compute(&w, 20, 4, 30, 8, &params);
        assert_eq!(
            engine.get_area(),
            (direct.x0, direct.y0, direct.w, direct.h)
        );
        assert!(engine.get_color(25, 6)[0] > 0.95, "seam should serve light");
        assert_eq!(engine.get_color(25, 6), direct.get(25, 6));
        // Outside the processed area the seam reports darkness, like `GetColor`.
        assert_eq!(engine.get_color(25, 300), [0.0; 3]);
    }
}
