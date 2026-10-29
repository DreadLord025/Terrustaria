//! Tile lighting: a port of `Terraria.Graphics.Light.TileLightScanner` (seed light per
//! tile) and `LightMap` (directional blur that spreads it, attenuated by tile opacity).
//!
//! The original computes lighting for the visible screen area inflated by 28 tiles and
//! blurs it in two passes of four line sweeps. We do the same on demand for the rect the
//! renderer is about to draw, so panning never shows unlit tiles.

use std::sync::OnceLock;

use crate::tile::*;
use crate::tile_light::{self, FastRandom, light_blocked};
use crate::tiledata as td;
use crate::world::World;
use rayon::prelude::*;

/// Extra tiles around the visible rect (`LightingEngine.AREA_PADDING`); the blur pulls
/// light in from outside the screen, so the border must not be visible.
pub const AREA_PADDING: i32 = 28;
/// D3: extra tiles cached around the visible rect. While the camera pans within this
/// margin the previous light map is reused without recomputing, and the visible area
/// always keeps at least `AREA_PADDING` of computed context around it.
pub const CACHE_MARGIN: i32 = 32;
/// Edge rows/columns that are not blurred (`LightingEngine.NON_VISIBLE_PADDING`).
const NON_VISIBLE_PADDING: i32 = 18;

/// Rects with at least this many cells seed and sweep rows on Rayon's persistent
/// global pool; smaller ones stay sequential. Measured on the reference laptop, task
/// dispatch makes the parallel path a net loss below ~100k cells (light 0.20 → 0.55 ms
/// at zoom 1.5), so real zooms (~20k cells max) keep the sequential path and only the
/// zoom-0.25 debug view (~128k cells) goes parallel.
const PAR_LIGHT_MIN_CELLS: usize = 96 * 1024;

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

/// `Lighting.GlobalBrightness` in the original.
pub const GLOBAL_BRIGHTNESS: f32 = 1.2;

/// `Terraria.Graphics.Light.LightMode`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LightMode {
    Color,
    White,
    Retro,
    Trippy,
}

impl LightMode {
    pub const ALL: [LightMode; 4] = [
        LightMode::Color,
        LightMode::White,
        LightMode::Retro,
        LightMode::Trippy,
    ];

    pub fn label(self) -> &'static str {
        match self {
            LightMode::Color => "Color",
            LightMode::White => "White",
            LightMode::Retro => "Retro",
            LightMode::Trippy => "Trippy",
        }
    }

    /// Wraps around; `F2` in the app uses this.
    pub fn next(self) -> Self {
        match self {
            LightMode::Color => LightMode::White,
            LightMode::White => LightMode::Retro,
            LightMode::Retro => LightMode::Trippy,
            LightMode::Trippy => LightMode::Color,
        }
    }
}

/// Rendering behaviour of one light mode. The tile pass reads this description only,
/// so a new mode (Sepia, Monochrome, Aberration...) is a new style plus, when needed,
/// a new shader flag — not a new draw path. Mirrors how `Lighting.Mode` switches the
/// original between its color engine and the legacy renderer.
#[derive(Clone, Copy, Debug)]
pub struct LightStyle {
    pub mode: LightMode,
    /// `Lighting.NotRetro`: solid blocks are drawn as 3x3 light-sliced quads
    /// (`DrawSingleTile_SlicedBlock`), which produces the smooth block edges.
    pub sliced_blocks: bool,
    /// Collapse the light colour to luminance ("White").
    pub grayscale: bool,
    /// Quantise light into this many steps (0/1 = off). Gives "Retro" its banding.
    pub levels: u8,
    /// Position/time-dependent channel mixing ("Trippy").
    pub perturb: f32,
    /// Screen-space RGB split in pixels ("Trippy"); consumed by the shader.
    pub aberration_px: f32,
    /// Applied after the light map, matching `Lighting.GlobalBrightness`.
    pub brightness: f32,
}

impl LightStyle {
    pub fn for_mode(mode: LightMode) -> Self {
        match mode {
            LightMode::Color => Self {
                mode,
                sliced_blocks: true,
                grayscale: false,
                levels: 0,
                perturb: 0.0,
                aberration_px: 0.0,
                brightness: GLOBAL_BRIGHTNESS,
            },
            LightMode::White => Self {
                mode,
                sliced_blocks: true,
                grayscale: true,
                levels: 0,
                perturb: 0.0,
                aberration_px: 0.0,
                brightness: GLOBAL_BRIGHTNESS,
            },
            LightMode::Retro => Self {
                mode,
                sliced_blocks: false,
                grayscale: false,
                levels: 16,
                perturb: 0.0,
                aberration_px: 0.0,
                brightness: GLOBAL_BRIGHTNESS,
            },
            LightMode::Trippy => Self {
                mode,
                sliced_blocks: false,
                grayscale: false,
                levels: 0,
                perturb: 0.65,
                // The shader hook exists (`Globals::aberration`), but applying it to
                // full-screen sky quads would sample past their atlas sheet. Enable
                // it only once instances carry a per-sprite flag.
                aberration_px: 0.0,
                brightness: GLOBAL_BRIGHTNESS,
            },
        }
    }

    /// Transforms one light sample into the tint written to the instance buffer.
    #[inline]
    pub fn shade(&self, light: [f32; 3], x: i32, y: i32, time: f32) -> [f32; 3] {
        let mut c = [light[0], light[1], light[2]];
        if self.perturb > 0.0 {
            let a =
                ((x as f32 * 0.11 + y as f32 * 0.17 + time * 2.3).sin() * 0.5 + 0.5) * self.perturb;
            let b =
                ((x as f32 * 0.07 - y as f32 * 0.13 + time * 1.1).sin() * 0.5 + 0.5) * self.perturb;
            let (r, g, bl) = (c[0], c[1], c[2]);
            c = [
                r * (1.0 - a) + bl * a,
                g * (1.0 - b) + r * b,
                bl * (1.0 - a) + g * a,
            ];
        }
        for v in c.iter_mut() {
            *v *= self.brightness;
        }
        if self.grayscale {
            let l = c[0] * 0.299 + c[1] * 0.587 + c[2] * 0.114;
            c = [l, l, l];
        }
        if self.levels > 1 {
            let n = (self.levels - 1) as f32;
            for v in c.iter_mut() {
                *v = (*v * n).round() / n;
            }
        }
        [
            c[0].clamp(0.0, 1.0),
            c[1].clamp(0.0, 1.0),
            c[2].clamp(0.0, 1.0),
        ]
    }
}

/// Ambient parameters for one computation. Everything here comes from `Main` and
/// feeds `TileLightScanner`; the per-type light code in `tile_light` reads it.
#[derive(Clone, Copy, Debug)]
pub struct LightParams {
    /// `Main.tileColor`: surface sunlight (derived from the sky colour).
    pub sky: [f32; 3],
    /// `0.55 + sin(t * 2) * 0.08`: the underworld glow.
    pub hell_glow: f32,
    /// `Main.mouseTextColor` (190..=255): drives the torch/lamp flicker terms.
    pub mouse_text_color: u8,
    /// `Main.demonTorch` (0..=1): the demon torch colour ramp.
    pub demon_torch: f32,
    /// `Main.DiscoR/G/B`: the rainbow cycle shared with waterfalls and gemspark.
    pub disco: [u8; 3],
    /// `Main.GlobalTimeWrappedHourly` (wall-clock seconds mod 3600).
    pub global_time: f32,
    /// `Main.timeForVisualEffects` (ticks since start).
    pub time_for_visuals: f32,
    /// `Main.time` (0..=54000 within a day).
    pub time: f32,
    /// `Main.dayTime`.
    pub day_time: bool,
    /// `Main.raining`.
    pub raining: bool,
    /// `Main.shimmerDarken` (0..=1).
    pub shimmer_darken: f32,
    /// `Main.liquidAlpha[13]`: the shimmer liquid's opacity.
    pub shimmer_alpha: f32,
    /// `Main.sundialCooldown`.
    pub sundial_cooldown: i32,
    /// `Main.moondialCooldown`.
    pub moondial_cooldown: i32,
    /// `Main.jellyfishCageMode` — `[type - 316][frame]`.
    pub cage_mode: [[u8; crate::tile_light::CAGE_FRAMES]; 3],
    /// Seed for the per-tile `FastRandom` (`TileLightScanner._random`).
    pub rng_seed: u64,
}

impl Default for LightParams {
    fn default() -> Self {
        Self {
            sky: [1.0, 1.0, 1.0],
            hell_glow: 0.55,
            mouse_text_color: 225,
            demon_torch: 1.0,
            disco: [255, 0, 0],
            global_time: 0.0,
            time_for_visuals: 0.0,
            time: 54000.0,
            day_time: true,
            raining: false,
            shimmer_darken: 0.0,
            shimmer_alpha: 0.0,
            sundial_cooldown: 0,
            moondial_cooldown: 0,
            cage_mode: [[0; crate::tile_light::CAGE_FRAMES]; 3],
            rng_seed: 0,
        }
    }
}

impl LightParams {
    /// Everything that can change a light sample. The map cache compares this
    /// bit-exactly, so a new field here cannot silently go stale.
    fn key(&self, world_revision: u64) -> LightKey {
        LightKey {
            sky: [
                self.sky[0].to_bits(),
                self.sky[1].to_bits(),
                self.sky[2].to_bits(),
            ],
            hell_glow: self.hell_glow.to_bits(),
            mouse_text_color: self.mouse_text_color,
            demon_torch: self.demon_torch.to_bits(),
            disco: self.disco,
            global_time: self.global_time.to_bits(),
            time_for_visuals: self.time_for_visuals.to_bits(),
            time: self.time.to_bits(),
            day_time: self.day_time,
            raining: self.raining,
            shimmer_darken: self.shimmer_darken.to_bits(),
            shimmer_alpha: self.shimmer_alpha.to_bits(),
            sundial_cooldown: self.sundial_cooldown,
            moondial_cooldown: self.moondial_cooldown,
            cage_mode: self.cage_mode,
            rng_seed: self.rng_seed,
            world_revision,
        }
    }

    /// A cheap identity of every parameter that can change a light sample, for the
    /// app's world-instance cache. The light map keeps its own exact key; this only
    /// has to change whenever the map's contents would.
    pub fn identity(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.key(0).hash(&mut h);
        h.finish()
    }
}

/// D2: identity of one light computation, compared bit-exactly (no hashing). A hit
/// means the map's `colors` already hold the correct result. `world_revision` makes
/// any future tile edit invalidate the cache.
#[derive(Clone, Copy, PartialEq, Hash)]
struct LightKey {
    sky: [u32; 3],
    hell_glow: u32,
    mouse_text_color: u8,
    demon_torch: u32,
    disco: [u8; 3],
    global_time: u32,
    time_for_visuals: u32,
    time: u32,
    day_time: bool,
    raining: bool,
    shimmer_darken: u32,
    shimmer_alpha: u32,
    sundial_cooldown: i32,
    moondial_cooldown: i32,
    cage_mode: [[u8; crate::tile_light::CAGE_FRAMES]; 3],
    rng_seed: u64,
    world_revision: u64,
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
        self.compute_into_impl(world, x0, y0, x1, y1, params, None);
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
        self.compute_into_impl(world, x0, y0, x1, y1, params, Some(parallel));
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
    ) {
        let key = params.key(world.revision);
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

        self.blur(parallel);
        self.blur(parallel);
    }

    /// `LightMap.Blur`: two passes over four line sweeps. Columns are swept first
    /// (sequential: row-major storage makes them strided) and rows afterwards — in
    /// parallel for the large rects that pass `PAR_LIGHT_MIN_CELLS`.
    fn blur(&mut self, parallel: bool) {
        let (w, h, pad) = (
            self.w,
            self.h,
            NON_VISIBLE_PADDING.min(self.w.min(self.h) / 3),
        );
        let wu = w as usize;

        for x in 0..w {
            // top -> bottom, then bottom -> top (stopping at the padding).
            blur_line_v(&mut self.colors, &self.mask, wu, x, 0, 1, h - pad);
            blur_line_v(&mut self.colors, &self.mask, wu, x, h - 1, -1, h - pad);
        }
        let count = w - pad;
        if parallel {
            self.colors
                .par_chunks_mut(wu)
                .zip(self.mask.par_chunks(wu))
                .enumerate()
                .for_each(|(ly, (colors, mask))| {
                    let base = ly * wu;
                    blur_line_h(colors, mask, base, 0, 1, count);
                    blur_line_h(colors, mask, base, w - 1, -1, count);
                });
        } else {
            for (ly, (colors, mask)) in self
                .colors
                .chunks_mut(wu)
                .zip(self.mask.chunks(wu))
                .enumerate()
            {
                let base = ly * wu;
                blur_line_h(colors, mask, base, 0, 1, count);
                blur_line_h(colors, mask, base, w - 1, -1, count);
            }
        }
    }
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
        mask_row[lx] = mask_of(tile);
        let c = seed(world, tile, x, y, params, cutoffs[lx]);
        if c[0] > 0.0 || c[1] > 0.0 || c[2] > 0.0 {
            lit += 1;
        }
        colors_row[lx] = c;
    }
    lit
}

/// One vertical sweep through column `x`, carrying the brightest light seen so far
/// and decaying it by the tile's opacity (`LightMap.BlurLine`).
fn blur_line_v(
    colors: &mut [[f32; 3]],
    mask: &[Mask],
    w: usize,
    x: i32,
    y0: i32,
    dy: i32,
    count: i32,
) {
    let mut carried = [0.0f32; 3];
    let mut done = [false; 3];
    for k in 0..count.max(0) {
        let y = y0 + dy * k;
        let i = y as usize * w + x as usize;
        let m = mask[i];
        let decay = m.decay();
        let jitter = if m == Mask::Water {
            let j = (i as u32).wrapping_mul(2_654_435_761) % 15;
            0.985 + j as f32 * 0.001
        } else {
            1.0
        };
        let color = &mut colors[i];
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

/// One horizontal sweep through a single row slice. `base` is the global index of
/// `colors[0]` in the full light map, so the water jitter stays bit-identical to the
/// sequential implementation.
fn blur_line_h(colors: &mut [[f32; 3]], mask: &[Mask], base: usize, x0: i32, dx: i32, count: i32) {
    let mut carried = [0.0f32; 3];
    let mut done = [false; 3];
    for k in 0..count.max(0) {
        let lx = (x0 + dx * k).max(0) as usize;
        let i = base + lx;
        let m = mask[lx];
        let decay = m.decay();
        let jitter = if m == Mask::Water {
            let j = (i as u32).wrapping_mul(2_654_435_761) % 15;
            0.985 + j as f32 * 0.001
        } else {
            1.0
        };
        let color = &mut colors[lx];
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

/// Top of the column's medium: the first solid tile or the first liquid cell,
/// whichever is higher. Seeding stops there, so a lakebed under water is lit by
/// the attenuated light coming through the water instead of a direct sky seed.
fn surface_cutoff(world: &World, x: i32) -> i32 {
    if x < 0 || x >= world.width {
        return 0;
    }
    let first_solid = world
        .column_surface
        .get(x as usize)
        .copied()
        .unwrap_or(world.world_surface as i32);
    for y in 0..first_solid.max(0) {
        if world.tile(x, y).is_some_and(|t| t.liquid > 0) {
            return y;
        }
    }
    first_solid
}

/// `TileLightScanner.GetTileMask`.
fn mask_of(tile: &Tile) -> Mask {
    if light_blocked(tile) && tile.kind != 131 && tile.slope() == 0 {
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

/// `TileLightScanner.GetTileLight`: surface/hell ambient, walls, the self-lit tile
/// types and liquids, each taking the maximum.
fn seed(
    world: &World,
    tile: &Tile,
    x: i32,
    y: i32,
    p: &LightParams,
    local_surface: i32,
) -> [f32; 3] {
    let mut c = [0.0f32; 3];
    // `TileLightScanner._random.WithModifier(x, y)`: one stream per tile.
    let mut rng = FastRandom::new(p.rng_seed).with_modifier(x, y);
    // The original seeds surface light up to the global `Main.worldSurface`. That
    // leaves hills above the average bright and valleys below it unlit. Seeding up to
    // each column's own surface keeps every slope in sunlight; the light map still
    // stops at solid blocks, so caves stay dark.
    if y <= local_surface {
        tile_light::apply_surface_light(world, tile, x, y, &mut rng, p, &mut c);
    } else if y > world.height - 200 {
        tile_light::apply_hell_light(world, tile, x, y, p, &mut c);
    }
    tile_light::apply_wall_light(tile, &mut rng, p, &mut c);
    if tile.active() {
        tile_light::apply_tile_light(world, tile, x, y, &mut rng, p, &mut c);
    }
    liquid_light(tile, &mut c, p);
    c
}

/// `TileLightScanner.ApplyLiquidLight`: lava and shimmer emit light. The original
/// compares every channel against the unscaled base and then stores the scaled
/// value, which is why the green channel can end up brighter than the test.
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
    fn light_styles_transform_samples() {
        let color = LightStyle::for_mode(LightMode::Color);
        let white = LightStyle::for_mode(LightMode::White);
        let retro = LightStyle::for_mode(LightMode::Retro);
        let c = [0.8, 0.4, 0.2];
        let col = color.shade(c, 0, 0, 0.0);
        let wht = white.shade(c, 0, 0, 0.0);
        assert!(wht[0] == wht[1] && wht[1] == wht[2], "white {wht:?}");
        assert!(col[0] > col[1] && col[1] > col[2], "color {col:?}");
        let r = retro.shade([0.13, 0.13, 0.13], 0, 0, 0.0);
        let raw = 0.13 * GLOBAL_BRIGHTNESS;
        let quant = (raw * 15.0).round() / 15.0;
        assert!((r[0] - quant).abs() < 1e-5, "retro {r:?} vs {quant}");
        assert!(
            (r[0] - raw).abs() > 0.01,
            "retro should quantise, got {r:?}"
        );
    }
}
