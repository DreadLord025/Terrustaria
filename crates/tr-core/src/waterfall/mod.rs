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

    /// `WaterfallManager.FindWaterfalls`' view-rectangle head
    /// (`WaterfallManager.cs:119-122`): the four `(int)` casts TRUNCATE toward
    /// zero (C# `(int)` on a float, not a floor), and the `-1`/`+2` margins are
    /// applied by [`Self::find_waterfalls`] itself — this returns the raw
    /// `[num+1, num3+1, num2-2, num4-2]` so the existing scan keeps its
    /// verbatim `view[0]-1-dist` / `view[2]+2+dist` / `view[3]+2+20` shape.
    /// `position` is the raw unscaled `Main.screenPosition` (world pixels),
    /// `width`/`height` the raw `Main.screenWidth`/`Height`.
    pub fn scan_view(position: [f32; 2], width: f32, height: f32) -> [i32; 4] {
        [
            (position[0] / 16.0) as i32,
            (position[1] / 16.0) as i32,
            ((position[0] + width) / 16.0) as i32,
            ((position[1] + height) / 16.0) as i32,
        ]
    }

    /// `WaterfallManager.FindWaterfalls`.
    ///
    /// `view` is [`Self::scan_view`]'s truncating tile rectangle
    /// `[x0, y0, x1, y1]`; the `-1`/`+2` margins and the `waterfallDist`/`+20`
    /// pads (`WaterfallManager.cs:123-126`) are applied here verbatim.
    /// `forced` is the call-site flag: `Main.instantBGTransitionCounter == 10`
    /// at the draw-time call (`Main.cs:61841`) or the capture-time `true`
    /// (`:56968`).
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
    ///
    /// `screen` is the C#'s raw `Main.screenPosition` /
    /// `Main.screenWidth`/`Height` in pixels, used by the rain-branch cull
    /// (`:381`) and both ambience scans (`:687-691` / `:710-714`); the caller
    /// derives them from the camera exactly as the original's unscaled screen.
    ///
    /// `show_invisible` is `Main.ShouldShowInvisibleBlocksAndWalls()` sampled
    /// once per `Draw` (`:1120`) and read by `GetAlpha`'s wall gate (`:1037`).
    ///
    /// `drew_lava` is `Main.drewLava` at entry (`Main.cs:1101`): the reset at
    /// the top of this method (`WaterfallManager.cs:1125`) leaves it false
    /// for every pass, so in the port's single-call shape it only gates the
    /// typed/rain/disco skips when a caller passes a stale true — the
    /// `DoDraw_Waterfalls` site (`Main.cs:63281`) runs after `DrawWaters`,
    /// which sets it true (`:57938`), but `Draw()` clears it first.
    ///
    /// Returns the `AddLight` rows plus the `Main.ambientWaterfallX/Y/Strength`
    /// and `Main.ambientLavafallX/Y/Strength` scans of the LAST active pass
    /// (`WaterfallManager.cs:960-965` — each `DrawWaterfall` call overwrites
    /// both triples after resetting its locals at `:328-337`). Honey falls
    /// (kind 14) feed neither scan: the second ambience arm excludes them
    /// verbatim (`:708`). No active pass leaves the `Main.cs:67230-67234`
    /// load reset on both triples.
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
        show_invisible: bool,
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
                    show_invisible,
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
/// original's `tile2.lava() || tile4.lava() || tile3.lava()` order
/// (`WaterfallManager.cs:182/:186/:190`). The C# probes are PURE header-bit
/// tests (`Tile.cs:345-348/:362-365/:379-382`: `(bTileHeader & 0x60) == kind`)
/// — no liquid-amount check — so a stale kind flag on a drained (liquid == 0)
/// tile still classifies the spout; the packed port reads the same flag bits
/// through `liquid_kind()`. A free function replacing the `LiquidKindChecks`
/// port artifact (the original has no such trait).
fn any_liquid_kind(kind: LiquidKind, tiles: &[&Tile; 3]) -> bool {
    tiles.iter().any(|t| t.liquid_kind() == kind)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::{FLAG_ACTIVE, FLAG_HALF_BRICK, FLAG_LIQUID_HONEY, FLAG_LIQUID_LAVA};

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

    /// `WaterfallManager.cs:182` + `Tile.cs:345-348`: the kind probes are pure
    /// header-bit tests — `tile2.lava()` never reads `liquid`. A drained
    /// (liquid == 0) tile above the half brick with a stale lava flag still
    /// classifies the spout as type 1 (the `liquid > 0` guard the first port
    /// pass carried was an invention).
    #[test]
    fn a_stale_liquid_flag_classifies_without_any_liquid_amount() {
        let mut w = world_with_air();
        let mut brick = stone(1);
        brick.flags |= FLAG_HALF_BRICK;
        put(&mut w, 10, 10, brick);
        // Above: liquid 0 (passes the `< 16` head gate) with a lava flag.
        let mut stale = Tile::default();
        stale.flags |= FLAG_LIQUID_LAVA;
        put(&mut w, 10, 9, stale);
        // Right: plain water > 160 supplies the spout condition; left is air.
        put_liquid(&mut w, 11, 10, 200);
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        assert_eq!(mgr.entries(), vec![(10, 10, 1)]);
    }

    /// `WaterfallManager.cs:119-122`: the four `(int)` casts truncate toward
    /// zero — a negative fractional tile coordinate goes to 0, not -1 (a
    /// floor-based derivation would scan one tile wider on the left/top).
    #[test]
    fn the_scan_view_truncates_like_the_csharp_int_casts() {
        // position (-8, -8), screen 640x480: the C# num = (int)(-8/16 - 1) =
        // (int)(-1.5) = -1 (truncation, not floor's -2); num3 likewise. num2
        // pre-margin = (int)(632/16) = 39, num4 = (int)(472/16) = 29. The
        // -1/+2 margins live in find_waterfalls, so scan_view yields the raw
        // truncations: [0, 0, 39, 29].
        assert_eq!(
            WaterfallManager::scan_view([-8.0, -8.0], 640.0, 480.0),
            [0, 0, 39, 29]
        );
        // Exact-grid coordinates are unchanged by the truncation.
        assert_eq!(
            WaterfallManager::scan_view([160.0, 320.0], 640.0, 480.0),
            [10, 20, 50, 50]
        );
    }

    /// `Main.cs:61841`: `FindWaterfalls(instantBGTransitionCounter == 10)` —
    /// the forced flag bypasses the 30-tick gate mid-window, exactly like the
    /// load-time `forced: true` (`WaterfallManager.cs:110`).
    #[test]
    fn the_forced_flag_bypasses_the_thirty_tick_rescan_gate() {
        let mut w = world_with_air();
        put(&mut w, 12, 8, stone(196));
        let mut mgr = WaterfallManager::new();
        // One unforced call right after a scan: the counter is 1, so no scan.
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        let mut w2 = world_with_air();
        put(&mut w2, 13, 8, stone(196));
        mgr.find_waterfalls(&w2, [5, 5, 20, 20], false);
        assert_eq!(mgr.entries(), vec![(12, 9, 11)], "the unforced call skipped");
        // The forced call rescans immediately.
        mgr.find_waterfalls(&w2, [5, 5, 20, 20], true);
        assert_eq!(mgr.entries(), vec![(13, 9, 11)]);
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
            &w, &light, &style, 0.0, &alpha, 190, 0, ScreenInfo { position: [0.0, 0.0], width: 640.0, height: 640.0 },
            false, false,
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
            let (lights_a, wamb_a, lamb_a) = a.draw(
                &wa, &la, &style, frame as f32 / 60.0, &alpha, 190, seed, screen, false, false,
            );
            let (lights_b, wamb_b, lamb_b) = b.draw(
                &wb, &lb, &style, frame as f32 / 60.0, &alpha, 190, seed, screen, false, false,
            );
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
            &w, &light, &style, 0.0, &alpha, 255, 0, ScreenInfo { position: [0.0, 0.0], width: 640.0, height: 640.0 },
            false, false,
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
            &w, &light, &style, 0.0, &alpha2, 255, 0, ScreenInfo { position: [0.0, 0.0], width: 640.0, height: 640.0 },
            false, false,
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

    /// Water-spout fixture: half brick at (10, 10), water on the right — the
    /// walk bends LEFT off the half-brick head (`WaterfallManager.cs:625-634`):
    /// cells (10,10), (9,10), then column 9 down to the world edge (the g75
    /// re-derivation of the g74 fixture: the fixed 51-cell left-bend walk).
    fn water_spout_world() -> World {
        let mut w = world_with_air();
        let mut brick = stone(2);
        brick.flags |= FLAG_HALF_BRICK;
        put(&mut w, 10, 10, brick);
        put_liquid(&mut w, 11, 9, 255);
        put_liquid(&mut w, 11, 10, 255);
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

    /// `WaterfallManager.cs:708-729` + `:960-962`: the water triple is the
    /// summed near-screen strength of the walked water segments and the
    /// nearest segment's cell-centre world pixels — the arm the honey fall
    /// is excluded from. The walk geometry replays the C# direction chain:
    /// the half-brick head cannot fall straight (`:609`'s `!tile3.halfBrick()`
    /// gate), and the liquid-filled right neighbour bends it left (`:625-634`),
    /// so the 51 visited cells are (10,10), (9,10), then (9,11..59) until the
    /// world edge. Also pins the verbatim `:725-728` quirk: the Y sentinel
    /// stores the X distance (`num3 = num38`), which is 0 at the head, so the
    /// stored Y never updates again. (g75's re-derivation of the g74 fixture
    /// walk — same cell set as the original g73 replay.)
    #[test]
    fn waterfall_scan_sums_strength_and_tracks_the_nearest_cell() {
        let w = water_spout_world();
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        mgr.update_tick();
        let light = LightMap::new();
        let style = LightStyle::for_mode(crate::lighting::LightMode::Color);
        // Screen centred on the spout head (168, 168): every walked segment is
        // inside the 2x box, so each contributes `1 - dist/(640*0.75)`.
        let screen = screen_at(168.0 - 320.0, 168.0 - 320.0);
        let (_, scan, lava) = mgr.draw(
            &w, &light, &style, 0.0, &active_alpha(), 190, 0, screen, false, false,
        );
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

    /// A world without falls runs one active pass over an empty list: the
    /// locals reset (`WaterfallManager.cs:328-337`) is stored verbatim
    /// (`:960-965`) — (-1, -1, 0) on BOTH triples, the `Main.cs:67230-67234`
    /// load reset.
    #[test]
    fn an_empty_pass_stores_the_load_reset_triple() {
        let w = world_with_air();
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        let light = LightMap::new();
        let style = LightStyle::for_mode(crate::lighting::LightMode::Color);
        let (_, scan, lava) = mgr.draw(
            &w, &light, &style, 0.0, &active_alpha(), 190, 0, screen_at(0.0, 0.0), false, false,
        );
        assert_eq!(scan, AmbientScan::default());
        assert_eq!(lava, AmbientScan::default());
    }

    /// `WaterfallManager.cs:960-962`: each `DrawWaterfall` pass overwrites the
    /// triple. With two active water styles both passes re-draw the water fall
    /// (only the rain arm zeroes stopAtStep, `:524`), so the last pass's scan
    /// equals the first's segment-for-segment — the store is the last pass's.
    #[test]
    fn the_last_active_pass_overwrites_the_waterfall_triple() {
        let w = water_spout_world();
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        mgr.update_tick();
        let light = LightMap::new();
        let style = LightStyle::for_mode(crate::lighting::LightMode::Color);
        let mut alpha = [0.0f32; 15];
        alpha[0] = 1.0;
        alpha[2] = 1.0;
        let near = screen_at(168.0 - 320.0, 168.0 - 320.0);
        let (_, scan_single, _) = mgr.draw(
            &w, &light, &style, 0.0, &active_alpha(), 190, 0, near, false, false,
        );
        assert!(scan_single.strength > 0.0);
        let (_, scan_both, _) = mgr.draw(
            &w, &light, &style, 0.0, &alpha, 190, 0, near, false, false,
        );
        assert_eq!(scan_both, scan_single);
    }

    /// The scan runs per visited CELL, not per quad: a fall boxed in on both
    /// sides with liquid in the cell takes the `num29++` else (`:640-645`)
    /// and the case-0 arm skips the body quad (`liquid > 0 && !halfBrick`,
    /// `:926-930`) — but the scan block (`:708-730`) already ran for it, so
    /// the cell contributes its full strength 1.0. Also pins the verbatim
    /// `:725-728` quirk: the Y sentinel stores the X distance (`num3 =
    /// num38`), so the stored Y stays the FIRST cell's row (10) even though
    /// the screen centre sits on row 12 — only the X minimum keeps updating.
    #[test]
    fn the_scan_visits_cells_the_quad_list_skips() {
        let mut w = water_spout_world();
        // The spout steps away from its source liquid (`:625-634`), so the
        // fall drops down column 9. Box the last cell in: stone sides at
        // row 12, floor at 13, water pooled in (9, 12).
        put(&mut w, 8, 12, stone(1));
        put(&mut w, 10, 12, stone(1));
        put(&mut w, 9, 13, stone(1));
        put_liquid(&mut w, 9, 12, 200);
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        mgr.update_tick();
        let light = LightMap::new();
        let style = LightStyle::for_mode(crate::lighting::LightMode::Color);
        // Screen centred on the head's column at the boxed-in row (168, 200):
        // the head's X distance is 0, so BOTH sentinels freeze at the head —
        // the verbatim quirk (`num3 = num38`) keeps Y at the head's row.
        let screen = screen_at(168.0 - 320.0, 200.0 - 320.0);
        let (_, scan, _) = mgr.draw(
            &w, &light, &style, 0.0, &active_alpha(), 190, 0, screen, false, false,
        );
        let out = mgr.take_quads();
        // The boxed-in cell emits no row-0 body quad (the row-24 cap of
        // `:777-798` may still draw); every body quad sits above it.
        assert!(out.iter().filter(|d| d.source[1] == 0).all(|d| d.pos[1] < 12.0 * 16.0));
        // Visited cells: the head (10,10), then (9,10), (9,11), (9,12).
        let cells = [(10, 10), (9, 10), (9, 11), (9, 12)];
        let mut expected = 0.0f32;
        for (cx, cy) in cells {
            let dx = (cx as f32 * 16.0 + 8.0 - 168.0).abs();
            let dy = (cy as f32 * 16.0 + 8.0 - 200.0).abs();
            let s = 1.0 - (dx * dx + dy * dy).sqrt() / (640.0 * 0.75);
            if s > 0.0 {
                expected += s;
            }
        }
        assert!((scan.strength - expected).abs() < 1e-3, "{} vs {}", scan.strength, expected);
        assert!(scan.strength > 3.5); // includes the body-quad-less cell's ~0.95
        // Both minima freeze at the head (its X distance is 0).
        assert_eq!(scan.x, 10.0 * 16.0 + 8.0);
        assert_eq!(scan.y, 10.0 * 16.0 + 8.0);
    }

    /// The waterfall-specific slope transitions (`WaterfallManager.cs`):
    /// the spout head steps left onto column 9, falls two cells, then meets
    /// a slope-1 top at (9,12). Hand-walked C# replay of the entry cell
    /// (9,11) — arm 1 (`:590-607`) sets num30/num32/num17 = 1, num31 = -1:
    /// * the flag2 direction-change cap (`:733-743`): 32x14 at row 24,
    ///   `(x*16, y*16+14)`, unflipped (num31 != 1);
    /// * the num30 entry sheet (`:799-819`): 32x16 at row 24, `(x*16,
    ///   y*16)`, unflipped;
    /// * the 8-slice slope fan (`:860-873`): 2x8 strips, source X descending
    ///   30..16, the first two strips pinned to num48 = 4;
    /// then the run over the flat floor at (10,12) (`:886`, flipped, num11
    /// = 8) and the exit back to vertical at (11,12)/(11,13) (`:840` row-0
    /// with the num11 = 8 offset, `:844` row-24 exit sheet at x-16).
    #[test]
    fn slope_entry_and_exit_draw_the_transition_sheets() {
        let mut w = water_spout_world();
        let mut slope = stone(1);
        slope.set_slope(1);
        put(&mut w, 9, 12, slope);
        put(&mut w, 10, 13, stone(1));
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        mgr.update_tick(); // regular_frame stays 0 → num21 = 0
        let light = LightMap::new();
        let style = LightStyle::for_mode(crate::lighting::LightMode::Color);
        mgr.draw(
            &w, &light, &style, 0.0, &active_alpha(), 190, 0, screen_at(0.0, 0.0), false, false,
        );
        let out = mgr.take_quads();
        let has = |pos: [f32; 2], src: [i32; 4], flip: bool| {
            out.iter()
                .any(|d| d.texture == 0 && d.pos == pos && d.source == src && d.flip == flip)
        };
        // The flag2 direction-change cap: 16 - num41 - 2 = 14 tall.
        assert!(has([144.0, 190.0], [0, 24, 32, 14], false));
        // The num30 entry sheet.
        assert!(has([144.0, 176.0], [0, 24, 32, 16], false));
        // The 8-slice fan: y = 176 + 8 + num48, num48 = 4 for m < 2 (and
        // m = 2 coincides), then 2m; source X = 16 + (14 - 2m).
        for m in 0..8i32 {
            let num48 = if m < 2 { 4 } else { m * 2 };
            assert!(
                has(
                    [144.0 + (m * 2) as f32, 184.0 + num48 as f32],
                    [30 - m * 2, 0, 2, 8],
                    true,
                ),
                "slice {m}"
            );
        }
        // The horizontal run over the floor: num11 = 8 lift, flipped.
        assert!(has([160.0, 200.0], [16, 0, 16, 16], true));
        // The exit to vertical keeps the num11 = 8 offset for one cell.
        assert!(has([176.0, 200.0], [0, 0, 16, 16], true));
        // The row-24 exit sheet at x - 16, flipped.
        assert!(has([160.0, 208.0], [0, 24, 32, 16], true));
        // Every quad carries GetAlpha's below-surface 0.6 dim on alpha
        // (world_surface = 10, every visited row >= 10): (int)(255*0.6)=153.
        assert!(out.iter().all(|d| (d.color[3] - 153.0 / 255.0).abs() < 1e-6));
    }

    /// `WaterfallManager.cs:381`: the rain arm culls against the raw
    /// `Main.screenPosition`/`screenWidth` — a fountain (tile 196 → kind 11)
    /// at column 12 draws when the screen covers it and vanishes when the
    /// screen sits 40 tiles below (num14 + num22 = 34 < 40) or 100 tiles
    /// right (12 < 100 - 20).
    #[test]
    fn the_rain_arm_culls_against_the_raw_screen() {
        let mut w = world_with_air();
        put(&mut w, 12, 8, stone(196));
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        mgr.update_tick();
        let light = LightMap::new();
        let style = LightStyle::for_mode(crate::lighting::LightMode::Color);
        let rain_quads = |mgr: &mut WaterfallManager, screen: ScreenInfo| -> usize {
            mgr.draw(&w, &light, &style, 0.0, &active_alpha(), 190, 0, screen, false, false);
            mgr.take_quads()
                .iter()
                .filter(|d| d.texture == 11 || d.texture == 12)
                .count()
        };
        assert!(rain_quads(&mut mgr, screen_at(0.0, 0.0)) > 0);
        assert_eq!(rain_quads(&mut mgr, screen_at(0.0, 640.0)), 0);
        assert_eq!(rain_quads(&mut mgr, screen_at(1600.0, 0.0)), 0);
    }

    /// Lava-spout fixture (the g74 lane, fixed 51-cell left-bend walk): half
    /// brick at (10, 10), lava on the right — the fall bends LEFT off the
    /// head (`WaterfallManager.cs:625-634`): cells (10,10), (9,10), then
    /// column 9 down to the world edge.
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
        let (_, _, scan) =
            mgr.draw(&w, &light, &style, 0.0, &active_alpha(), 190, 0, screen, false, false);
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
        let w = water_spout_world();
        let mut mgr = WaterfallManager::new();
        mgr.find_waterfalls(&w, [5, 5, 20, 20], true);
        mgr.update_tick();
        let light = LightMap::new();
        let style = LightStyle::for_mode(crate::lighting::LightMode::Color);
        let (_, _, scan) = mgr.draw(
            &w, &light, &style, 0.0, &active_alpha(), 190, 0, screen_at(0.0, 0.0), false, false,
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
        let (_, _, scan_single) =
            mgr.draw(&w, &light, &style, 0.0, &active_alpha(), 190, 0, near, false, false);
        assert!(scan_single.strength > 0.0);
        let (_, _, scan_both) =
            mgr.draw(&w, &light, &style, 0.0, &alpha, 190, 0, near, false, false);
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
            mgr.draw(&w, &light, &style, 0.0, &active_alpha(), 190, 0, screen, false, false);
            mgr.take_quads()
                .iter()
                // The head cell bends LEFT off the half-brick spout
                // (`WaterfallManager.cs:625-634`, the lava-filled right
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
            &w, &light, &style, 0.0, &active_alpha(), 190, 0, screen_at(0.0, 0.0), false, false,
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
        let (dim, _, _) =
            mgr.draw(&w, &light, &style, 0.0, &active_alpha(), 190, 0, screen, false, false);
        let (bright, _, _) =
            mgr.draw(&w, &light, &style, 0.0, &active_alpha(), 255, 0, screen, false, false);
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
            mgr.draw(&w, &light, &style, 0.0, &active_alpha(), 190, 0, screen, false, false);
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
        mgr.draw(&w, &light, &style, 0.0, &alpha, 190, 0, screen_at(0.0, 0.0), false, false);
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
        mgr.draw(&w, &lit, &style, 0.0, &alpha, 190, 0, screen_at(0.0, 0.0), false, false);
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
            mgr.draw(&w, &light, &style, 0.0, &active_alpha(), 255, 0, screen, false, false);
        assert!(lights.is_empty());
        assert_eq!(water_scan, AmbientScan::default());
        assert_eq!(lava_scan, AmbientScan::default());
        assert!(mgr.take_quads().iter().any(|d| d.texture == 14));
    }

    /// `WaterfallManager.cs:725-728` verbatim quirk: the Y-distance arm tests
    /// num39 but stores num38 (the X distance) into the num3 sentinel. On a
    /// straight drop num38 is 0 for every row, so the head row already pins
    /// num3 = 0 and no later row can pass the strict `<` — ambientWaterfallY
    /// freezes at the head even when a later row's centre is nearer to the
    /// screen centre (a corrected `num3 = num39` would track it).
    #[test]
    fn waterfall_scan_y_arm_keeps_the_num3_equals_num38_quirk() {
        let w = water_spout_world();
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
            mgr.draw(&w, &light, &style, 0.0, &active_alpha(), 190, 0, screen, false, false);
        assert_eq!(scan.x, 10.0 * 16.0 + 8.0);
        assert_eq!(scan.y, 10.0 * 16.0 + 8.0, "the quirk freezes Y at the head");
    }
}
