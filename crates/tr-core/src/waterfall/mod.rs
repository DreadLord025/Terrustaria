//! `Terraria.WaterfallManager` port: finding, animating and drawing waterfalls.
//!
//! The manager keeps the same 30-frame rescan cadence, the same 8.5 Hz / 20 Hz
//! frame counters and the same per-step geometry walk as the original. The app
//! turns the returned [`WaterfallDraw`] records into sprites; lighting, the
//! shimmer vertex-colour path and dust sparkles are applied there.
//!
//! The `DrawWaterfall` geometry walk lives in [`draw`].

mod draw;

use self::draw::solid_tile;
use crate::lighting::{DynamicLight, LightMap, LightStyle};
use crate::tile::{LiquidKind, Tile};
use crate::tile_light::FastRandom;
use crate::world::World;

/// `WaterfallManager.maxTypes`: `Images/Waterfall_0..27`.
pub const MAX_TYPES: usize = 28;

/// `WaterfallManager.maxWaterfallCountDefault`.
const MAX_WATERFALL_COUNT_DEFAULT: usize = 1000;

/// `Main.gfxQuality`: the original adapts this to the frame rate. A fixed 1.0
/// uses the full waterfall distance (100 tiles) and count (1000), which is what
/// a machine holding 60 FPS converges to.
const GFX_QUALITY: f32 = 1.0;

#[derive(Clone, Copy, Default, Debug)]
struct Waterfall {
    x: i32,
    y: i32,
    kind: i32,
    stop_at_step: i32,
}

/// One textured waterfall quad, in world pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaterfallDraw {
    /// `Images/Waterfall_{texture}` index.
    pub texture: u8,
    /// Top-left position in world pixels.
    pub pos: [f32; 2],
    /// Source rectangle inside the 512x40 / 144x18 sheet.
    pub source: [i32; 4],
    /// `SpriteEffects.FlipHorizontally`.
    pub flip: bool,
    /// Premultiplied-by-opacity tint, matching `Color * alpha`.
    pub color: [f32; 4],
}

/// A `WaterfallManager.TrySparkling` spawn: the `Dust.NewDust` position in world
/// pixels (`x*16 - direction*7, y*16 + 6`, `WaterfallManager.cs:1061`). The dust
/// has zero velocity and `DustID.Sparkle`'s frame 43.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SparkleSpawn {
    pub x: f32,
    pub y: f32,
}

/// `WaterfallManager`.
pub struct WaterfallManager {
    waterfalls: Vec<Waterfall>,
    quality_max: usize,
    current_max: usize,
    waterfall_dist: i32,
    find_waterfall_count: i32,
    /// Bumped when a rescan changes the list, for the G6 world-range cache.
    list_revision: u64,

    w_fall_fr_counter: i32,
    regular_frame: i32,
    w_fall_fr_counter2: i32,
    slow_frame: i32,
    rain_frame_counter: i32,
    rain_frame_foreground: i32,
    rain_frame_background: i32,
    lava_rain_frame_counter: i32,
    lava_rain_frame_foreground: i32,
    lava_rain_frame_background: i32,
    snow_frame_counter: i32,
    snow_frame_foreground: i32,

    /// `Main.DiscoR/G/B/Style` for the disco waterfall (tile 160).
    disco_r: i32,
    disco_g: i32,
    disco_b: i32,
    disco_style: i32,

    /// `TrySparkling` spawns accumulated by the last `draw` — the caller drains
    /// them into its dust particles after the frame.
    pending_sparkles: Vec<SparkleSpawn>,
    /// The `WaterfallDraw` quads of the last `draw`. The walk runs once per
    /// frame in the fixed pre-draw phase (deterministically, whether or not the
    /// G6 world range rebuilds); the tile pass consumes the buffered quads.
    pending_quads: Vec<WaterfallDraw>,
}

impl Default for WaterfallManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WaterfallManager {
    pub fn new() -> Self {
        Self {
            waterfalls: vec![Waterfall::default(); MAX_WATERFALL_COUNT_DEFAULT],
            quality_max: MAX_WATERFALL_COUNT_DEFAULT,
            current_max: 0,
            waterfall_dist: (75.0 * GFX_QUALITY) as i32 + 25,
            // The original increments before testing `< 30`; starting at 29 makes
            // the first update perform the initial scan (the load-time forced scan).
            find_waterfall_count: 29,
            list_revision: 0,
            w_fall_fr_counter: 0,
            regular_frame: 0,
            w_fall_fr_counter2: 0,
            slow_frame: 0,
            rain_frame_counter: 0,
            rain_frame_foreground: 0,
            rain_frame_background: 0,
            lava_rain_frame_counter: 0,
            lava_rain_frame_foreground: 0,
            lava_rain_frame_background: 0,
            snow_frame_counter: 0,
            snow_frame_foreground: 0,
            disco_r: 0,
            disco_g: 0,
            disco_b: 0,
            disco_style: 0,
            pending_sparkles: Vec::new(),
            pending_quads: Vec::new(),
        }
    }

    pub fn count(&self) -> usize {
        self.current_max
    }

    /// `Main.DiscoR/G/B`: the rainbow cycle, shared with the lighting scanner
    /// (disco torches, gemspark, painted lamps) so both read the same phase.
    pub fn disco_rgb(&self) -> [u8; 3] {
        [
            self.disco_r.clamp(0, 255) as u8,
            self.disco_g.clamp(0, 255) as u8,
            self.disco_b.clamp(0, 255) as u8,
        ]
    }

    /// `WaterfallManager.CheckForWaterfall`.
    pub fn check_for_waterfall(&self, x: i32, y: i32) -> bool {
        self.waterfalls[..self.current_max]
            .iter()
            .any(|w| w.x == x && w.y == y)
    }

    /// Cache key for the animated quads. Zero when no waterfall is visible, so a
    /// quiet world does not invalidate the G6 range on every animation frame.
    pub fn anim_key(&self) -> u64 {
        if self.current_max == 0 {
            return 0;
        }
        (self.regular_frame as u64)
            | (self.slow_frame as u64) << 4
            | (self.rain_frame_foreground as u64) << 8
            | (self.rain_frame_background as u64) << 12
            | (self.lava_rain_frame_foreground as u64) << 16
            | (self.lava_rain_frame_background as u64) << 20
            | (self.snow_frame_foreground as u64) << 24
            | self.list_revision << 32
    }

    /// `WaterfallManager.UpdateFrame` + `DoUpdate_AnimateDiscoRGB` — both advance
    /// once per 60 Hz tick. `Main.mouseTextColor` (the light flicker) lives in
    /// `crate::lighting`, and `AddLight` for falls is a documented follow-up.
    pub fn update_tick(&mut self) {
        self.w_fall_fr_counter += 1;
        if self.w_fall_fr_counter > 2 {
            self.w_fall_fr_counter = 0;
            self.regular_frame += 1;
            if self.regular_frame > 15 {
                self.regular_frame = 0;
            }
        }

        self.w_fall_fr_counter2 += 1;
        if self.w_fall_fr_counter2 > 6 {
            self.w_fall_fr_counter2 = 0;
            self.slow_frame += 1;
            if self.slow_frame > 15 {
                self.slow_frame = 0;
            }
        }

        self.rain_frame_counter += 1;
        if self.rain_frame_counter > 0 {
            self.rain_frame_foreground += 1;
            if self.rain_frame_foreground > 7 {
                self.rain_frame_foreground -= 8;
            }
            if self.rain_frame_counter > 2 {
                self.rain_frame_counter = 0;
                self.rain_frame_background -= 1;
                if self.rain_frame_background < 0 {
                    self.rain_frame_background = 7;
                }
            }
        }

        self.lava_rain_frame_counter += 1;
        if self.lava_rain_frame_counter == 1 || self.lava_rain_frame_counter == 3 {
            self.lava_rain_frame_foreground += 1;
            if self.lava_rain_frame_foreground > 7 {
                self.lava_rain_frame_foreground -= 8;
            }
        } else if self.lava_rain_frame_counter > 3 {
            self.lava_rain_frame_counter = 0;
            self.lava_rain_frame_background -= 1;
            if self.lava_rain_frame_background < 0 {
                self.lava_rain_frame_background = 7;
            }
        }

        self.snow_frame_counter += 1;
        if self.snow_frame_counter > 3 {
            self.snow_frame_counter = 0;
            self.snow_frame_foreground += 1;
            if self.snow_frame_foreground > 7 {
                self.snow_frame_foreground = 0;
            }
        }

        self.update_disco_rgb();
    }

    fn update_disco_rgb(&mut self) {
        let step = 7;
        if self.disco_style == 0 {
            self.disco_g += step;
            if self.disco_g >= 255 {
                self.disco_g = 255;
                self.disco_style += 1;
            }
        }
        if self.disco_style == 1 {
            self.disco_r -= step;
            if self.disco_r <= 0 {
                self.disco_r = 0;
                self.disco_style += 1;
            }
        }
        if self.disco_style == 2 {
            self.disco_b += step;
            if self.disco_b >= 255 {
                self.disco_b = 255;
                self.disco_style += 1;
            }
        }
        if self.disco_style == 3 {
            self.disco_g -= step;
            if self.disco_g <= 0 {
                self.disco_g = 0;
                self.disco_style += 1;
            }
        }
        if self.disco_style == 4 {
            self.disco_r += step;
            if self.disco_r >= 255 {
                self.disco_r = 255;
                self.disco_style += 1;
            }
        }
        if self.disco_style == 5 {
            self.disco_b -= step;
            if self.disco_b <= 0 {
                self.disco_b = 0;
                self.disco_style = 0;
            }
        }
    }

    /// `WaterfallManager.FindWaterfalls`.
    ///
    /// `view` is the visible tile rectangle `[x0, y0, x1, y1]` (inclusive). The
    /// original works from `Main.screenPosition` / screen size; the caller derives
    /// the same rectangle from the camera.
    pub fn find_waterfalls(&mut self, world: &World, view: [i32; 4], forced: bool) {
        self.find_waterfall_count += 1;
        if self.find_waterfall_count < 30 && !forced {
            return;
        }
        self.find_waterfall_count = 0;

        self.waterfall_dist = (75.0 * GFX_QUALITY) as i32 + 25;
        self.quality_max = (MAX_WATERFALL_COUNT_DEFAULT as f32 * GFX_QUALITY) as usize;
        let mut current = 0usize;
        let mut next: Vec<Waterfall> = Vec::with_capacity(self.quality_max.min(64));

        let mut x0 = view[0] - 1 - self.waterfall_dist;
        let mut x1 = view[2] + 2 + self.waterfall_dist;
        let mut y0 = view[1] - 1 - self.waterfall_dist;
        let mut y1 = view[3] + 2 + 20;
        if x0 <= 0 {
            x0 = 1;
        }
        if x1 >= world.width {
            x1 = world.width - 1;
        }
        if y0 <= 0 {
            y0 = 1;
        }
        if y1 >= world.height {
            y1 = world.height - 1;
        }

        for i in x0..x1 {
            for j in y0..y1 {
                let Some(tile) = world.tile(i, j) else {
                    continue;
                };
                if !tile.active() {
                    continue;
                }
                if tile.half_brick() {
                    let above = world.tile(i, j - 1).copied().unwrap_or_default();
                    if above.liquid < 16 || solid_tile(&above) {
                        let left = world.tile(i - 1, j).copied().unwrap_or_default();
                        let right = world.tile(i + 1, j).copied().unwrap_or_default();
                        if (left.liquid > 160 || right.liquid > 160)
                            && ((left.liquid == 0 && !solid_tile(&left) && left.slope() == 0)
                                || (right.liquid == 0 && !solid_tile(&right) && right.slope() == 0))
                            && current < self.quality_max
                        {
                            let mut kind = 0;
                            if any_liquid_kind(LiquidKind::Lava, &[&above, &left, &right]) {
                                kind = 1;
                            } else if any_liquid_kind(LiquidKind::Honey, &[&above, &left, &right]) {
                                kind = 14;
                            } else if any_liquid_kind(LiquidKind::Shimmer, &[&above, &left, &right])
                            {
                                kind = 25;
                            }
                            push_waterfall(&mut next, i, j, kind);
                            current += 1;
                        }
                    }
                }
                if tile.kind == 196 {
                    let below = world.tile(i, j + 1).copied().unwrap_or_default();
                    if !solid_tile(&below)
                        && below.liquid == 0
                        && below.slope() == 0
                        && current < self.quality_max
                    {
                        push_waterfall(&mut next, i, j + 1, 11);
                        current += 1;
                    }
                }
                if tile.kind == 460 {
                    let below = world.tile(i, j + 1).copied().unwrap_or_default();
                    if !solid_tile(&below)
                        && below.liquid == 0
                        && below.slope() == 0
                        && current < self.quality_max
                    {
                        push_waterfall(&mut next, i, j + 1, 22);
                        current += 1;
                    }
                }
                if tile.kind == 717 {
                    let below = world.tile(i, j + 1).copied().unwrap_or_default();
                    if !solid_tile(&below)
                        && below.liquid == 0
                        && below.slope() == 0
                        && current < self.quality_max
                    {
                        push_waterfall(&mut next, i, j + 1, 26);
                        current += 1;
                    }
                }
            }
        }

        // Preserve the original scan order and only invalidate the world range
        // when the visible set actually changed.
        let changed = current != self.current_max
            || self.waterfalls[..current]
                .iter()
                .zip(next.iter())
                .any(|(a, b)| a.x != b.x || a.y != b.y || a.kind != b.kind);
        self.waterfalls[..current].copy_from_slice(&next[..current]);
        self.current_max = current;
        if changed {
            self.list_revision = self.list_revision.wrapping_add(1);
        }
    }

    /// `WaterfallManager.Draw` (`WaterfallManager.cs:1118-1174`): resets
    /// `stopAtStep` and runs one `DrawWaterfall(style, liquidAlpha[i])` pass per
    /// water style with a non-zero alpha. The water-style index maps to the
    /// waterfall texture pass (`liquidAlpha[0]→0`, `liquidAlpha[2]→3`,
    /// `liquidAlpha[3]→4`, `liquidAlpha[4]→5`, `liquidAlpha[5..9]→6..10`,
    /// `liquidAlpha[10]→13`, `liquidAlpha[12]→23`, `liquidAlpha[13]→24`); lava,
    /// honey and shimmer falls carry their own type and draw in every active
    /// pass, exactly like the original. The walk also collects `AddLight`
    /// sources and `TrySparkling` spawns for the caller to drain.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        world: &World,
        light: &LightMap,
        style: &LightStyle,
        time: f32,
        liquid_alpha: &[f32; 15],
        mouse_text: u8,
        sparkle_seed: u64,
        view: [i32; 4],
    ) -> Vec<DynamicLight> {
        for w in &mut self.waterfalls[..self.current_max] {
            w.stop_at_step = self.waterfall_dist;
        }
        self.pending_sparkles.clear();
        // Collect into a local first: `draw_waterfall` also mutates the fall
        // list's `stop_at_step` bookkeeping, so it cannot borrow `self` twice.
        let mut quads = Vec::new();
        let mut lights = Vec::new();
        // The sparkle draws use `Main.rand`; the port pins the stream to a
        // frame-derived seed so screenshots stay deterministic.
        let mut sparkle_rng = FastRandom::new(sparkle_seed);
        for (idx, fall_style) in [
            (0usize, 0usize),
            (2, 3),
            (3, 4),
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 8),
            (8, 9),
            (9, 10),
            (10, 13),
            (12, 23),
            (13, 24),
        ] {
            if liquid_alpha[idx] > 0.0 {
                self.draw_waterfall(
                    fall_style as i32,
                    liquid_alpha[idx],
                    world,
                    light,
                    style,
                    time,
                    mouse_text,
                    &mut sparkle_rng,
                    view,
                    &mut quads,
                    &mut lights,
                );
            }
        }
        self.pending_quads = quads;
        lights
    }

    /// The `TrySparkling` spawns accumulated by the last [`Self::draw`].
    pub fn take_sparkles(&mut self) -> Vec<SparkleSpawn> {
        std::mem::take(&mut self.pending_sparkles)
    }

    /// Takes the quads of the last [`Self::draw`], for the tile pass to sprite.
    pub fn take_quads(&mut self) -> Vec<WaterfallDraw> {
        std::mem::take(&mut self.pending_quads)
    }

    /// D2 cache-key contribution of the waterfall `AddLight` list: the lights
    /// change exactly when the fall list does (their flicker rides the light
    /// params' `mouse_text_color`). Zero keeps the cache key untouched, so a
    /// waterfall-free world never invalidates the light map.
    pub fn lights_key(&self) -> u64 {
        if self.current_max == 0 {
            return 0;
        }
        self.list_revision
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ (self.current_max as u64) << 32
    }

    /// Test helper: current waterfall list `(x, y, type)`.
    #[cfg(test)]
    fn entries(&self) -> Vec<(i32, i32, i32)> {
        self.waterfalls[..self.current_max]
            .iter()
            .map(|w| (w.x, w.y, w.kind))
            .collect()
    }
}

fn push_waterfall(list: &mut Vec<Waterfall>, x: i32, y: i32, kind: i32) {
    list.push(Waterfall {
        x,
        y,
        kind,
        stop_at_step: 0,
    });
}

/// Any of the three tiles around a half-brick spout holds `kind`, mirroring the
/// original's `tile2.lava() || tile4.lava() || tile3.lava()` order. A free
/// function replacing the `LiquidKindChecks` port artifact (the original has no
/// such trait).
fn any_liquid_kind(kind: LiquidKind, tiles: &[&Tile; 3]) -> bool {
    tiles
        .iter()
        .any(|t| t.liquid > 0 && t.liquid_kind() == kind)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::{FLAG_ACTIVE, FLAG_HALF_BRICK, FLAG_LIQUID_LAVA};

    fn world_with_air() -> World {
        let mut w = World::new("test".into(), 80, 60);
        w.world_surface = 10.0;
        w
    }

    fn stone(kind: u16) -> Tile {
        let mut t = Tile::default();
        t.kind = kind;
        t.flags |= FLAG_ACTIVE;
        t
    }

    fn put(w: &mut World, x: i32, y: i32, t: Tile) {
        let i = w.index(x, y);
        w.tiles[i] = t;
    }

    fn put_liquid(w: &mut World, x: i32, y: i32, amount: u8) {
        let i = w.index(x, y);
        w.tiles[i].liquid = amount;
    }

    #[test]
    fn finds_half_brick_spout_with_liquid_to_the_side() {
        let mut w = world_with_air();
        // Half brick at (10, 10), water running down the right-hand side.
        let mut brick = stone(1);
        brick.flags |= FLAG_HALF_BRICK;
        put(&mut w, 10, 10, brick);
        put_liquid(&mut w, 11, 9, 255);
        put_liquid(&mut w, 11, 10, 200);

        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        assert_eq!(mgr.entries(), vec![(10, 10, 0)]);
    }

    #[test]
    fn fast_liquid_does_not_make_a_waterfall() {
        let mut w = world_with_air();
        let mut brick = stone(1);
        brick.flags |= FLAG_HALF_BRICK;
        put(&mut w, 10, 10, brick);
        put_liquid(&mut w, 10, 9, 10);
        put_liquid(&mut w, 11, 9, 10);
        put_liquid(&mut w, 11, 10, 10);
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        assert!(mgr.entries().is_empty());
    }

    #[test]
    fn lava_spout_uses_type_one() {
        let mut w = world_with_air();
        let mut brick = stone(1);
        brick.flags |= FLAG_HALF_BRICK;
        put(&mut w, 10, 10, brick);
        let mut lava = Tile::default();
        lava.liquid = 255;
        lava.flags |= FLAG_LIQUID_LAVA;
        put(&mut w, 11, 9, lava);
        put(&mut w, 11, 10, lava);
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        assert_eq!(mgr.entries(), vec![(10, 10, 1)]);
    }

    #[test]
    fn waterfall_tile_196_drops_below() {
        let mut w = world_with_air();
        put(&mut w, 12, 8, stone(196));
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        assert_eq!(mgr.entries(), vec![(12, 9, 11)]);
    }

    #[test]
    fn frame_counters_match_the_original_rates() {
        let mut mgr = WaterfallManager::new();
        for _ in 0..3 {
            mgr.update_tick();
        }
        assert_eq!(mgr.regular_frame, 1);
        for _ in 0..21 {
            mgr.update_tick();
        }
        // 24 ticks: regular advances every 3 (8), slow every 7 (3).
        assert_eq!(mgr.regular_frame, 8);
        assert_eq!(mgr.slow_frame, 3);
    }

    #[test]
    fn draw_emits_a_falling_water_quad() {
        let mut w = world_with_air();
        let mut brick = stone(2);
        brick.flags |= FLAG_HALF_BRICK;
        put(&mut w, 10, 10, brick);
        put_liquid(&mut w, 11, 9, 255);
        put_liquid(&mut w, 11, 10, 255);
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        mgr.update_tick();
        let light = LightMap::new();
        let style = LightStyle::for_mode(crate::lighting::LightMode::Color);
        let alpha = {
            let mut a = [0.0f32; 15];
            a[0] = 1.0;
            a
        };
        let lights = mgr.draw(&w, &light, &style, 0.0, &alpha, 190, 0, [0, 0, 40, 40]);
        let out: Vec<WaterfallDraw> = mgr.take_quads();
        assert!(!out.is_empty());
        assert!(out.iter().any(|d| d.texture == 0));
        assert!(out.iter().any(|d| d.pos[0] == 160.0 && d.pos[1] == 160.0));
        assert!(out.iter().all(|d| d.source[2] > 0 && d.source[3] > 0));
        // Type-0 water falls emit no `AddLight` glow.
        assert!(lights.is_empty());
    }

    #[test]
    fn the_draw_walk_is_deterministic_for_the_same_seed_chain() {
        // Two identical managers fed the same per-frame seed chain must stay
        // equal frame by frame: quads, `AddLight` rows and the `TrySparkling`
        // grants. This is the contract the app's `TR_SHOT` harness relies on —
        // the waterfall walk runs every frame, outside the G6 world cache, and
        // its `AddLight` list feeds the light map (frame.rs).
        let build = || {
            let mut w = world_with_air();
            let mut brick = stone(2);
            brick.flags |= FLAG_HALF_BRICK;
            put(&mut w, 10, 10, brick);
            put_liquid(&mut w, 11, 9, 255);
            put_liquid(&mut w, 11, 10, 255);
            w
        };
        let style = LightStyle::for_mode(crate::lighting::LightMode::Color);
        let alpha = {
            let mut a = [0.0f32; 15];
            a[0] = 1.0;
            a
        };
        let (wa, wb) = (build(), build());
        let mut a = WaterfallManager::new();
        let mut b = WaterfallManager::new();
        a.find_waterfalls(&wa, [5, 5, 20, 20], true);
        b.find_waterfalls(&wb, [5, 5, 20, 20], true);
        for frame in 0..600u64 {
            a.update_tick();
            b.update_tick();
            // The rescan gate (`find_waterfall_count < 30`) runs inside.
            a.find_waterfalls(&wa, [5, 5, 20, 20], false);
            b.find_waterfalls(&wb, [5, 5, 20, 20], false);
            let seed = frame
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                ^ 0x0123_4567_89AB_CDEF;
            let la = LightMap::new();
            let lb = LightMap::new();
            let lights_a =
                a.draw(&wa, &la, &style, frame as f32 / 60.0, &alpha, 190, seed, [0, 0, 40, 40]);
            let lights_b =
                b.draw(&wb, &lb, &style, frame as f32 / 60.0, &alpha, 190, seed, [0, 0, 40, 40]);
            assert_eq!(lights_a, lights_b);
            assert_eq!(a.take_quads(), b.take_quads());
            assert_eq!(a.pending_sparkles, b.pending_sparkles);
        }
        assert_eq!(a.pending_sparkles, b.pending_sparkles);
    }

    #[test]
    fn lava_falls_collect_lava_light_and_the_style_passes_dispatch() {
        let mut w = world_with_air();
        let mut brick = stone(2);
        brick.flags |= FLAG_HALF_BRICK;
        put(&mut w, 10, 10, brick);
        // A lava spout: `put_liquid` writes plain water, so set the kind bits.
        let mut lava = Tile::default();
        lava.liquid = 255;
        lava.flags |= FLAG_LIQUID_LAVA;
        put(&mut w, 11, 9, lava);
        put(&mut w, 11, 10, lava);
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        mgr.update_tick();
        let light = LightMap::new();
        let style = LightStyle::for_mode(crate::lighting::LightMode::Color);
        let alpha = {
            let mut a = [0.0f32; 15];
            a[0] = 1.0;
            a
        };
        let lights = mgr.draw(&w, &light, &style, 0.0, &alpha, 255, 0, [0, 0, 40, 40]);
        let out: Vec<WaterfallDraw> = mgr.take_quads();
        assert!(out.iter().any(|d| d.texture == 1));
        // `AddLight(1, ...)`: `(0.55 + (270 - 255) / 900) * 0.4` red-dominant.
        let first = lights.first().expect("lava fall lights its cells");
        assert!((first.rgb[0] - (0.55 + 15.0 / 900.0) * 0.4).abs() < 1e-5);
        assert!(first.rgb[0] > first.rgb[1] && first.rgb[1] > first.rgb[2]);
        // A second active water style runs a second pass (vanilla double-draws
        // the typed falls during a style transition).
        let mut alpha2 = [0.0f32; 15];
        alpha2[0] = 0.5;
        alpha2[2] = 0.5;
        mgr.draw(&w, &light, &style, 0.0, &alpha2, 255, 0, [0, 0, 40, 40]);
        let out2: Vec<WaterfallDraw> = mgr.take_quads();
        let lava_quads = out2.iter().filter(|d| d.texture == 1).count();
        assert!(lava_quads > 0);
    }

    #[test]
    fn anim_key_is_zero_without_waterfalls() {
        let mgr = WaterfallManager::new();
        assert_eq!(mgr.anim_key(), 0);
        assert_eq!(mgr.lights_key(), 0);
    }
}
