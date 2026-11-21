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
    /// Premultiplied-by-opacity tint, matching `Color * alpha`. The shimmer
    /// fall (type 25) DISCARDS it: `WaterfallManager.DrawWaterfall(int, ...)`
    /// (`WaterfallManager.cs:972-980`) replaces the colour with the
    /// `SetShimmerVertexColors` base/sparkle pair — the consumer rebuilds it
    /// from [`Self::cell`] + [`Self::opacity`].
    pub color: [f32; 4],
    /// The walked cell `(num13, num14)` the `DrawWaterfall` overload receives
    /// (`WaterfallManager.cs:969`): the shimmer vertex colours sample
    /// `GetCornerColors(x, y)` and the per-corner wave functions here.
    pub cell: [i32; 2],
    /// The overload's `opacity` argument — `GetAlpha`'s result, NOT the
    /// stylized tint. `SetShimmerVertexColors` / `_Sparkle` scale the base and
    /// sparkle colours by it (`WaterfallManager.cs:975`/`:978`).
    pub opacity: f32,
}

/// A `WaterfallManager.TrySparkling` spawn: the `Dust.NewDust` position in world
/// pixels (`x*16 - direction*7, y*16 + 6`, `WaterfallManager.cs:1061`). The dust
/// has zero velocity and `DustID.Sparkle`'s frame 43.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SparkleSpawn {
    pub x: f32,
    pub y: f32,
}

/// One `Main.ambient*fallX/Y/Strength` triple `DrawWaterfall` leaves behind —
/// the water-style scan (`WaterfallManager.cs:328-332` init, `:708-729`
/// accumulation, `:960-962` store) or the lavafall scan (`:333-337` init,
/// `:685-707` accumulation, `:963-965` store): the summed near-screen strength
/// of the contributing falls and the nearest segment's cell-centre world pixel
/// (`x*16+8` / `y*16+8`). `x`/`y` stay `-1` when no contributing fall was
/// visited (honey falls contribute to neither scan); `Main.cs:67230-67234`
/// resets both triples on world load.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AmbientScan {
    pub x: f32,
    pub y: f32,
    pub strength: f32,
}

impl Default for AmbientScan {
    fn default() -> Self {
        Self {
            x: -1.0,
            y: -1.0,
            strength: 0.0,
        }
    }
}

/// `Main.screenPosition` / `screenWidth` / `screenHeight` in pixels, the inputs
/// of the two ambience distance scans (`WaterfallManager.cs:687-691` /
/// `:710-714`) and the rain-arm view cull (`:381`).
#[derive(Clone, Copy, Debug)]
pub struct ScreenInfo {
    /// `Main.screenPosition`: the view's top-left corner.
    pub position: [f32; 2],
    pub width: f32,
    pub height: f32,
}

impl ScreenInfo {
    /// `Main.screenPosition + (screenWidth/2, screenHeight/2)` — the scan's
    /// distance origin (`WaterfallManager.cs:687-688`). `Main.screenWidth` is
    /// an `int`, so the halving truncates before the float add.
    pub(super) fn centre(&self) -> [f32; 2] {
        [
            self.position[0] + (self.width as i32 / 2) as f32,
            self.position[1] + (self.height as i32 / 2) as f32,
        ]
    }
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

    /// `WaterfallManager._shouldShowInvisibleBlocksAndWalls` (`:74`): the
    /// static latch `Draw()` refreshes per frame (`:1120`) and `GetAlpha`
    /// reads (`:1037`). False until the app latches it.
    show_invisible_blocks_and_walls: bool,
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
            show_invisible_blocks_and_walls: false,
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

    /// `WaterfallManager.Draw`'s head latch (`WaterfallManager.cs:1120`):
    /// `_shouldShowInvisibleBlocksAndWalls = Main.ShouldShowInvisibleBlocksAndWalls()`
    /// — read by `GetAlpha`'s default arm (`:1037`) so the Echo coating /
    /// monolith view keeps the wall damp. The app passes its
    /// `LightParams::draw_invisible_walls` (`Main.cs:58023-58030`).
    pub fn set_show_invisible(&mut self, show: bool) {
        self.show_invisible_blocks_and_walls = show;
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
    ///
    /// `drew_lava` is `Main.drewLava` at entry (`Main.cs:1101`): the reset at
    /// the top of this method (`WaterfallManager.cs:1125`) leaves it false
    /// for every pass, so in the port's single-call shape it only gates the
    /// typed/rain/disco skips when a caller passes a stale true — the
    /// `DoDraw_Waterfalls` site (`Main.cs:63281`) runs after `DrawWaters`,
    /// which sets it true (`:57938`), but `Draw()` clears it first.
    ///
    /// Returns the `AddLight` rows plus the `Main.ambientWaterfallX/Y/Strength`
    /// and `Main.ambientLavafallX/Y/Strength` scans of the LAST pass
    /// (`WaterfallManager.cs:960-965` — each `DrawWaterfall` call overwrites
    /// both triples). Honey falls (kind 14) feed neither scan: the second
    /// ambience arm excludes them verbatim (`:708`).
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
        screen: ScreenInfo,
        drew_lava: bool,
    ) -> (Vec<DynamicLight>, AmbientScan, AmbientScan) {
        for w in &mut self.waterfalls[..self.current_max] {
            w.stop_at_step = self.waterfall_dist;
        }
        // `WaterfallManager.Draw` (`WaterfallManager.cs:1125`): the walk
        // clears `Main.drewLava` before the first pass, so every pass of this
        // call sees false regardless of the value at entry (`drew_lava`
        // documents the call-site state — `Main.cs:63281` runs after
        // `DrawWaters` left it true, `:57938`).
        let _entry_drew_lava = drew_lava;
        let drew_lava = false;
        self.pending_sparkles.clear();
        // Collect into a local first: `draw_waterfall` also mutates the fall
        // list's `stop_at_step` bookkeeping, so it cannot borrow `self` twice.
        let mut quads = Vec::new();
        let mut lights = Vec::new();
        // `Main.ambientWaterfall*`/`ambientLavafall*` start at the
        // `Main.cs:67230-67234` load reset; each pass overwrites both triples
        // (`WaterfallManager.cs:960-965`), so the last active pass wins. No
        // active pass leaves the reset values.
        let mut water_ambient = AmbientScan::default();
        let mut lava_ambient = AmbientScan::default();
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
                (water_ambient, lava_ambient) = self.draw_waterfall(
                    fall_style as i32,
                    liquid_alpha[idx],
                    world,
                    light,
                    style,
                    time,
                    mouse_text,
                    &mut sparkle_rng,
                    screen,
                    drew_lava,
                    &mut quads,
                    &mut lights,
                );
            }
        }
        self.pending_quads = quads;
        (lights, water_ambient, lava_ambient)
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
    use crate::tile::{
        FLAG_ACTIVE, FLAG_HALF_BRICK, FLAG_INACTIVE, FLAG_LIQUID_HONEY, FLAG_LIQUID_LAVA,
        FLAG_LIQUID_SHIMMER,
    };

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
        let (lights, _, _) = mgr.draw(
            &w, &light, &style, 0.0, &alpha, 190, 0,
            ScreenInfo { position: [0.0, 0.0], width: 640.0, height: 640.0 },
            false,
        );
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
            let screen = ScreenInfo {
                position: [0.0, 0.0],
                width: 640.0,
                height: 640.0,
            };
            let (lights_a, wamb_a, lamb_a) =
                a.draw(&wa, &la, &style, frame as f32 / 60.0, &alpha, 190, seed, screen, false);
            let (lights_b, wamb_b, lamb_b) =
                b.draw(&wb, &lb, &style, frame as f32 / 60.0, &alpha, 190, seed, screen, false);
            assert_eq!(lights_a, lights_b);
            assert_eq!(wamb_a, wamb_b);
            assert_eq!(lamb_a, lamb_b);
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
        let (lights, _, _) = mgr.draw(
            &w, &light, &style, 0.0, &alpha, 255, 0,
            ScreenInfo { position: [0.0, 0.0], width: 640.0, height: 640.0 },
            false,
        );
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
        mgr.draw(
            &w, &light, &style, 0.0, &alpha2, 255, 0,
            ScreenInfo { position: [0.0, 0.0], width: 640.0, height: 640.0 },
            false,
        );
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

    /// Lava-spout fixture: half brick at (10, 10), lava on the right — the
    /// fall walks straight down column 10 from row 10.
    fn lava_spout_world() -> World {
        let mut w = world_with_air();
        let mut brick = stone(2);
        brick.flags |= FLAG_HALF_BRICK;
        put(&mut w, 10, 10, brick);
        let mut lava = Tile::default();
        lava.liquid = 255;
        lava.flags |= FLAG_LIQUID_LAVA;
        put(&mut w, 11, 9, lava);
        put(&mut w, 11, 10, lava);
        w
    }

    fn active_alpha() -> [f32; 15] {
        let mut a = [0.0f32; 15];
        a[0] = 1.0;
        a
    }

    fn screen_at(x: f32, y: f32) -> ScreenInfo {
        ScreenInfo {
            position: [x, y],
            width: 640.0,
            height: 640.0,
        }
    }

    /// `WaterfallManager.cs:685-707` + `:963-965`: the lavafall triple is the
    /// summed near-screen strength of the kind-1 segments and the nearest
    /// segment's cell-centre world pixels; no lava fall leaves the
    /// `Main.cs:67232-67234` load reset (-1, -1, 0).
    #[test]
    fn lavafall_scan_sums_strength_and_tracks_the_nearest_cell() {
        let w = lava_spout_world();
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        mgr.update_tick();
        let light = LightMap::new();
        let style = LightStyle::for_mode(crate::lighting::LightMode::Color);
        // Screen centred on the spout's first cell (168, 168): every walked
        // segment is inside the 2x box, so each contributes
        // `1 - dist/(640*0.75)`; the nearest cell is the spout head itself.
        let screen = screen_at(168.0 - 320.0, 168.0 - 320.0);
        let (_, _, scan) = mgr.draw(&w, &light, &style, 0.0, &active_alpha(), 190, 0, screen, false);
        assert_eq!(scan.x, 10.0 * 16.0 + 8.0);
        assert_eq!(scan.y, 10.0 * 16.0 + 8.0);
        // The head bends LEFT off the half-brick spout (the lava-filled right
        // neighbour, `WaterfallManager.cs:625-634`), then the drop runs down
        // column 9 until the world edge (y = 59): cells (10,10), (9,10),
        // (9,11..59). Hand-summed C# replay:
        let mut cells = vec![(10, 10), (9, 10)];
        cells.extend((11..60).map(|y| (9, y)));
        let mut expected = 0.0f32;
        for (cx, cy) in cells {
            let dx = (cx as f32 * 16.0 + 8.0 - 168.0).abs();
            let dy = (cy as f32 * 16.0 + 8.0 - 168.0).abs();
            if dx < 640.0 * 2.0 && dy < 640.0 * 2.0 {
                let s = 1.0 - (dx * dx + dy * dy).sqrt() / (640.0 * 0.75);
                if s > 0.0 {
                    expected += s;
                }
            }
        }
        assert!((scan.strength - expected).abs() < 1e-3, "{} vs {}", scan.strength, expected);
        assert!(scan.strength > 0.0);
    }

    /// A water-only world leaves the lavafall triple at the load reset —
    /// the kind-1 arm never runs (`WaterfallManager.cs:685`).
    #[test]
    fn water_falls_leave_the_lavafall_triple_at_the_reset() {
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
        let (_, _, scan) = mgr.draw(
            &w, &light, &style, 0.0, &active_alpha(), 190, 0, screen_at(0.0, 0.0), false,
        );
        assert_eq!(scan, AmbientScan::default());
    }

    /// `WaterfallManager.cs:963-965`: each `DrawWaterfall` pass overwrites the
    /// triple. With two active water styles both passes re-draw the lava fall
    /// (only the rain arm zeroes stopAtStep, `:524`), so the last pass's scan
    /// equals the first's segment-for-segment — the store is the last pass's.
    #[test]
    fn the_last_active_pass_overwrites_the_lavafall_triple() {
        let w = lava_spout_world();
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        mgr.update_tick();
        let light = LightMap::new();
        let style = LightStyle::for_mode(crate::lighting::LightMode::Color);
        let mut alpha = [0.0f32; 15];
        alpha[0] = 1.0;
        alpha[2] = 1.0;
        let near = screen_at(168.0 - 320.0, 168.0 - 320.0);
        let mut single = [0.0f32; 15];
        single[0] = 1.0;
        let (_, _, scan_single) = mgr.draw(&w, &light, &style, 0.0, &single, 190, 0, near, false);
        assert!(scan_single.strength > 0.0);
        let (_, _, scan_both) = mgr.draw(&w, &light, &style, 0.0, &alpha, 190, 0, near, false);
        assert_eq!(scan_both, scan_single);
    }

    /// `WaterfallManager.cs:358`: the lava fall animates on `32 * slowFrame`
    /// (the 7-tick counter), not the 3-tick regular frame the water falls use.
    #[test]
    fn lava_frames_ride_the_slow_counter() {
        let w = lava_spout_world();
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        let light = LightMap::new();
        let style = LightStyle::for_mode(crate::lighting::LightMode::Color);
        let screen = screen_at(0.0, 0.0);
        let lava_x = |mgr: &mut WaterfallManager| -> i32 {
            mgr.draw(&w, &light, &style, 0.0, &active_alpha(), 190, 0, screen, false);
            mgr.take_quads()
                .iter()
                // The head cell bends LEFT off the half-brick spout
                // (`WaterfallManager.cs:625-634`, the honey-filled right
                // neighbour), so the straight drop runs down column 9; its
                // quads take the `[num21, 0, ...]` source (`:851-861`).
                .find(|d| d.texture == 1 && d.pos[0] == 9.0 * 16.0)
                .expect("the lava fall draws")
                .source[0]
        };
        // Tick 0: slowFrame 0 — the sheet's first 32 px column.
        assert_eq!(lava_x(&mut mgr), 0);
        // 7 ticks advance slowFrame to 1 (`UpdateFrame`'s >6 gate).
        for _ in 0..7 {
            mgr.update_tick();
        }
        assert_eq!(lava_x(&mut mgr), 32);
        // 7 more: slowFrame 2. The regular frame has advanced 4 times by now
        // (ticks 3/6/9/12) — a regular-frame fall would sit at 128, not 64.
        for _ in 0..7 {
            mgr.update_tick();
        }
        assert_eq!(lava_x(&mut mgr), 64);
    }

    /// `WaterfallManager.StylizeColor` case 1 (`WaterfallManager.cs:995-1008`):
    /// the lava tint floors every channel at `190 * alpha` — an unlit fall
    /// still glows at 190/255, and `GetAlpha` keeps lava at alpha 1.0 even
    /// below `worldSurface` (`:1034`).
    #[test]
    fn lava_quads_carry_the_190_self_glow_floor() {
        // A tall world so the fall walks the full 100 steps and reaches
        // `GetAlpha`'s last-10-steps fade (k > 90).
        let mut w = World::new("test".into(), 80, 140);
        w.world_surface = 5.0;
        let mut brick = stone(2);
        brick.flags |= FLAG_HALF_BRICK;
        put(&mut w, 10, 10, brick);
        let mut lava = Tile::default();
        lava.liquid = 255;
        lava.flags |= FLAG_LIQUID_LAVA;
        put(&mut w, 11, 9, lava);
        put(&mut w, 11, 10, lava);
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        mgr.update_tick();
        let light = LightMap::new(); // unlit: GetColor returns black
        let style = LightStyle::for_mode(crate::lighting::LightMode::Color);
        mgr.draw(
            &w, &light, &style, 0.0, &active_alpha(), 190, 0, screen_at(0.0, 0.0), false,
        );
        let out = mgr.take_quads();
        let lava: Vec<&WaterfallDraw> = out.iter().filter(|d| d.texture == 1).collect();
        assert!(!lava.is_empty());
        // The head segment: (int)(190 * 1.0) / 255 on every channel, alpha 255.
        assert_eq!(lava[0].color, [190.0 / 255.0, 190.0 / 255.0, 190.0 / 255.0, 1.0]);
        // The fade tail (`GetAlpha`'s last-10-steps ramp, `:1039-1042`) scales
        // the floor with alpha: the final segment (k = 99) fades to 0.1.
        let last = lava.last().unwrap();
        assert!((last.color[0] - 19.0 / 255.0).abs() < 1e-6);
        assert!(last.color[0] < 190.0 / 255.0);
    }

    /// `WaterfallManager.AddLight` case 1 (`WaterfallManager.cs:1071-1075`):
    /// `(0.55 + (270 - mouseTextColor)/900) * 0.4` red-dominant, one row per
    /// walked cell — the flicker rides the shared `mouseTextColor` pulse.
    #[test]
    fn lava_glow_flickers_with_mouse_text_color() {
        let w = lava_spout_world();
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        mgr.update_tick();
        let light = LightMap::new();
        let style = LightStyle::for_mode(crate::lighting::LightMode::Color);
        let screen = screen_at(0.0, 0.0);
        let (dim, _, _) = mgr.draw(&w, &light, &style, 0.0, &active_alpha(), 190, 0, screen, false);
        let (bright, _, _) =
            mgr.draw(&w, &light, &style, 0.0, &active_alpha(), 255, 0, screen, false);
        assert_eq!(dim.len(), bright.len());
        assert!(!dim.is_empty());
        let n_dim = (0.55 + (270.0 - 190.0) / 900.0) * 0.4;
        let n_bright = (0.55 + (270.0 - 255.0) / 900.0) * 0.4;
        assert!((dim[0].rgb[0] - n_dim).abs() < 1e-6);
        assert!((bright[0].rgb[0] - n_bright).abs() < 1e-6);
        assert!(bright[0].rgb[0] > dim[0].rgb[0]);
        for l in dim.iter().chain(bright.iter()) {
            assert!((l.rgb[1] - l.rgb[0] * 0.3).abs() < 1e-6);
            assert!((l.rgb[2] - l.rgb[0] * 0.1).abs() < 1e-6);
        }
        // One light per walked cell, in walk order: the head (10,10), the
        // left bend (9,10) (`WaterfallManager.cs:625-634`), then column 9
        // down to the world edge.
        let mut cells = vec![(10, 10), (9, 10)];
        cells.extend((11..60).map(|y| (9, y)));
        assert_eq!(dim.len(), cells.len());
        for (cell, l) in cells.iter().zip(dim.iter()) {
            assert_eq!((l.x, l.y), *cell);
        }
    }

    /// Honey-spout fixture: half brick at (10, 10), honey on the right — the
    /// `FindWaterfalls` kind probe sets type 14 (`WaterfallManager.cs:186-188`).
    /// The walk bends left off the half-brick head (`:625-634`): cells
    /// (10,10), (9,10), then column 9 down to the world edge.
    fn honey_spout_world() -> World {
        let mut w = world_with_air();
        let mut brick = stone(2);
        brick.flags |= FLAG_HALF_BRICK;
        put(&mut w, 10, 10, brick);
        let mut honey = Tile::default();
        honey.liquid = 255;
        honey.flags |= FLAG_LIQUID_HONEY;
        put(&mut w, 11, 9, honey);
        put(&mut w, 11, 10, honey);
        w
    }

    /// `WaterfallManager.cs:186-188`: the honey spout takes waterfall type 14
    /// (not the lava 1 checked first, not the water 0 default).
    #[test]
    fn honey_spout_uses_type_fourteen() {
        let w = honey_spout_world();
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        assert_eq!(mgr.entries(), vec![(10, 10, 14)]);
    }

    /// `WaterfallManager.cs:352-358`: the honey fall shares the typed-fall arm
    /// with lava/shimmer — `num21 = 32 * slowFrame` (the 7-tick counter), not
    /// the 3-tick regular frame the water falls use.
    #[test]
    fn honey_frames_ride_the_slow_counter() {
        let w = honey_spout_world();
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        let light = LightMap::new();
        let style = LightStyle::for_mode(crate::lighting::LightMode::Color);
        let screen = screen_at(0.0, 0.0);
        let honey_x = |mgr: &mut WaterfallManager| -> i32 {
            mgr.draw(&w, &light, &style, 0.0, &active_alpha(), 190, 0, screen, false);
            mgr.take_quads()
                .iter()
                .find(|d| d.texture == 14)
                .expect("the honey fall draws")
                .source[0]
        };
        // The head cell is the half-brick spout with honey on its right, so
        // the walk bends LEFT (`WaterfallManager.cs:625-634`) and the head
        // quad takes the num32 = -1 arm's `16 + num21` source (`:874-889`).
        // Tick 0: slowFrame 0 — 16 px into the sheet's first 32 px frame.
        assert_eq!(honey_x(&mut mgr), 16);
        // 7 ticks advance slowFrame to 1 (`UpdateFrame`'s >6 gate).
        for _ in 0..7 {
            mgr.update_tick();
        }
        assert_eq!(honey_x(&mut mgr), 16 + 32);
        // 7 more: slowFrame 2. The regular frame has advanced 4 times by now
        // (ticks 3/6/9/12) — a regular-frame fall would sit at 16 + 128.
        for _ in 0..7 {
            mgr.update_tick();
        }
        assert_eq!(honey_x(&mut mgr), 16 + 64);
    }

    /// `WaterfallManager.GetAlpha` (`WaterfallManager.cs:1035`): honey rides a
    /// flat 0.8 — no wall/surface 0.6 damp, no pass-alpha multiply (unlike the
    /// water default arm), and `StylizeColor` has no case 14 (`:993-1025`), so
    /// an unlit fall stays black: no 190 self-glow floor like lava's. The
    /// last-10-steps fade (`:1039-1042`) still scales the 0.8.
    #[test]
    fn honey_alpha_is_flat_point_eight_without_a_glow_floor() {
        // A tall world so the fall walks the full 100 steps (k > 90 fade).
        let mut w = World::new("test".into(), 80, 300);
        w.world_surface = 5.0; // every walked row is below the surface
        let mut brick = stone(2);
        brick.flags |= FLAG_HALF_BRICK;
        put(&mut w, 10, 10, brick);
        let mut honey = Tile::default();
        honey.liquid = 255;
        honey.flags |= FLAG_LIQUID_HONEY;
        put(&mut w, 11, 9, honey);
        put(&mut w, 11, 10, honey);
        // A stone wall behind the fall column: proves GetAlpha's type-14 arm
        // never takes the default arm's wall 0.6 damp (`:1037`).
        for y in 9..115 {
            let i = w.index(9, y);
            w.tiles[i].wall = 1;
            let i = w.index(10, y);
            w.tiles[i].wall = 1;
        }
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        mgr.update_tick();
        let light = LightMap::new(); // unlit: GetColor returns black
        let style = LightStyle::for_mode(crate::lighting::LightMode::Color);
        // Pass alpha 0.5 must NOT scale the typed-fall 0.8 (`:1032-1038`).
        let mut alpha = [0.0f32; 15];
        alpha[0] = 0.5;
        mgr.draw(&w, &light, &style, 0.0, &alpha, 190, 0, screen_at(0.0, 0.0), false);
        let out = mgr.take_quads();
        let honey_quads: Vec<&WaterfallDraw> =
            out.iter().filter(|d| d.texture == 14).collect();
        assert!(!honey_quads.is_empty());
        // Unlit: colour = (int)(0 * 0.8) = 0 on RGB, alpha = (int)(255 * 0.8).
        assert_eq!(honey_quads[0].color, [0.0, 0.0, 0.0, 204.0 / 255.0]);
        // The fade tail: k = 99 fades 0.8 -> 0.08, alpha (int)(255*0.08) = 20.
        let last = honey_quads.last().unwrap();
        assert!((last.color[3] - 20.0 / 255.0).abs() < 1e-6);
        // The lit leg: a torch at (16,10) lights the head cell from 6 tiles
        // away. Sunlight off (sky 0) keeps the head dim — below lava's 190
        // floor — so the tint proves StylizeColor has no case 14 lift. The
        // expected value replays the port's exact chain (shade -> byte/255
        // -> *255*alpha -> (int)).
        let torch = stone(4);
        put(&mut w, 16, 10, torch);
        let params = crate::lighting::LightParams {
            sky: [0.0; 3],
            ..Default::default()
        };
        let lit = crate::lighting::LightMap::compute(&w, 0, 0, 40, 40, &params);
        assert!(lit.get(10, 10)[0] > 0.0, "the torch must light the head");
        mgr.draw(&w, &lit, &style, 0.0, &alpha, 190, 0, screen_at(0.0, 0.0), false);
        let out = mgr.take_quads();
        let head = out
            .iter()
            .find(|d| d.texture == 14 && d.pos == [160.0, 160.0])
            .expect("the honey head segment draws");
        let shaded = style.shade(lit.get(10, 10), 10, 10, 0.0);
        let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0) as i32;
        let tint = byte(shaded[0]) as f32 / 255.0;
        let expected = (tint * 255.0 * 0.8).clamp(0.0, 255.0) as i32 as f32 / 255.0;
        assert!((head.color[0] - expected).abs() < 1e-6);
        // No StylizeColor case 14: the tint is exactly the light times 0.8 —
        // never lifted to lava's 190 floor even where the light is dim.
        assert!(head.color[0] < 190.0 / 255.0, "no lava-style glow floor");
        assert!(head.color[0] > 0.0, "a lit honey fall is visible");
    }

    /// `WaterfallManager.AddLight` (`WaterfallManager.cs:1067-1116`) has no
    /// case 14: the honey fall emits no light. And both ambience scans skip
    /// it — the kind-1 arm (`:685`) and the water arm's verbatim exclusion
    /// (`:708` `num12 != 14`) — so a honey-only world leaves BOTH triples at
    /// the `Main.cs:67230-67234` load reset.
    #[test]
    fn honey_falls_emit_no_light_and_feed_neither_ambience_scan() {
        let w = honey_spout_world();
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        mgr.update_tick();
        let light = LightMap::new();
        let style = LightStyle::for_mode(crate::lighting::LightMode::Color);
        // Screen centred on the spout head: the fall would sum strength if it
        // were not excluded.
        let screen = screen_at(168.0 - 320.0, 168.0 - 320.0);
        let (lights, water_scan, lava_scan) =
            mgr.draw(&w, &light, &style, 0.0, &active_alpha(), 255, 0, screen, false);
        assert!(lights.is_empty());
        assert_eq!(water_scan, AmbientScan::default());
        assert_eq!(lava_scan, AmbientScan::default());
        assert!(mgr.take_quads().iter().any(|d| d.texture == 14));
    }

    /// `WaterfallManager.cs:708-729` + `:960-962`: a water fall sums the
    /// near-screen strength into the water-style triple and tracks the
    /// nearest cell-centre world pixels — the arm the honey fall is excluded
    /// from. The walk geometry replays the C# direction chain: the half-brick
    /// head cannot fall straight (`:609`'s `!tile3.halfBrick()` gate), and the
    /// liquid-filled right neighbour bends it left (`:625-634`), so the cells
    /// are (10,10), (9,10), then (9,11..59) until the world edge.
    #[test]
    fn waterfall_scan_sums_strength_and_tracks_the_nearest_cell() {
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
        let screen = screen_at(168.0 - 320.0, 168.0 - 320.0);
        let (_, scan, lava) =
            mgr.draw(&w, &light, &style, 0.0, &active_alpha(), 190, 0, screen, false);
        // The head is nearest on both axes (dx = dy = 0); the bent column 9
        // never beats it (the Y arm's num3 = num38 quirk pins 0 at the head).
        assert_eq!(scan.x, 10.0 * 16.0 + 8.0);
        assert_eq!(scan.y, 10.0 * 16.0 + 8.0);
        assert_eq!(lava, AmbientScan::default());
        // Hand-summed C# replay over the walked cells:
        let mut cells = vec![(10, 10), (9, 10)];
        cells.extend((11..60).map(|y| (9, y)));
        let mut expected = 0.0f32;
        for (cx, cy) in cells {
            let dx = (cx as f32 * 16.0 + 8.0 - 168.0).abs();
            let dy = (cy as f32 * 16.0 + 8.0 - 168.0).abs();
            if dx < 640.0 * 2.0 && dy < 640.0 * 2.0 {
                let s = 1.0 - (dx * dx + dy * dy).sqrt() / (640.0 * 0.75);
                if s > 0.0 {
                    expected += s;
                }
            }
        }
        assert!((scan.strength - expected).abs() < 1e-3, "{} vs {}", scan.strength, expected);
        assert!(scan.strength > 0.0);
    }

    /// `WaterfallManager.cs:725-728` verbatim quirk: the Y-distance arm tests
    /// num39 but stores num38 (the X distance) into the num3 sentinel. On a
    /// straight drop num38 is 0 for every row, so the head row already pins
    /// num3 = 0 and no later row can pass the strict `<` — ambientWaterfallY
    /// freezes at the head even when a later row's centre is nearer to the
    /// screen centre (a corrected `num3 = num39` would track it).
    #[test]
    fn waterfall_scan_y_arm_keeps_the_num3_equals_num38_quirk() {
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
        // Screen centre y = 184 (row 11's centre): head row 10 has num39 = 16
        // but stores num3 = num38 = 0 (y = 168); row 11 has num39 = 0, which
        // fails the strict `0 < 0`, and every later row has num39 > 0 — the
        // nearest row (11) never wins. The corrected arm would report y = 184.
        let screen = screen_at(168.0 - 320.0, 184.0 - 320.0);
        let (_, scan, _) =
            mgr.draw(&w, &light, &style, 0.0, &active_alpha(), 190, 0, screen, false);
        assert_eq!(scan.x, 10.0 * 16.0 + 8.0);
        assert_eq!(scan.y, 10.0 * 16.0 + 8.0, "the quirk freezes Y at the head");
    }

}
