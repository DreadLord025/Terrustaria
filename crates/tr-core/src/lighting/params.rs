//! Light modes, styles and the ambient parameter set.
//!
//! `LightMode`/`LightStyle` describe *how* light is rendered (the Rust-side analogue
//! of switching `Lighting.Mode` between the colour engine and the legacy renderer);
//! `LightParams` is the `Main`-state snapshot that `TileLightScanner` consumes.

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
    pub(super) fn key(&self, world_revision: u64) -> LightKey {
        LightKey {
            lights: 0,
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
pub(super) struct LightKey {
    sky: [u32; 3],
    /// The `AddLight` source-set hash (`LightMap::compute_into_with_lights`).
    pub(super) lights: u64,
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

#[cfg(test)]
mod tests {
    use super::*;

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
