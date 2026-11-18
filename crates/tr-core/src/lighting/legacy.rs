//! `Terraria.Graphics.Light.LegacyLighting` (LegacyLighting.cs, 1442 lines) —
//! the pre-1.2 per-tile light engine the White/Retro/Trippy modes select
//! (`Lighting.Mode` setter, `Lighting.cs:29-61`: White → `LegacyEngine.Mode=1`,
//! Retro → 2, Trippy → 3; Color → the new RGB engine, which stays in
//! [`super::LightMap`]).
//!
//! The engine keeps a screen-sized jagged `LightingState[x][y]` array with two
//! value sets: the *presented* `R/G/B` (what `GetColor` serves) and the
//! *working* `R2/G2/B2` the swipe passes propagate. One cycle spans
//! `RenderPhases + 1` frames (`ProcessArea`, `:302-369`):
//!
//! - `renderCount > RenderPhases` → `PreRenderPhase` (`:648-849`): clear the
//!   working set, re-seed it through `TileLightScanner.ExportTo`, apply the
//!   `AddLight` dictionary, reset `renderCount = 0` and run swipe pass 0.
//! - `renderCount == 1` → swipe passes 1 and 2 (`DoColors`, `:997-1015`).
//! - `renderCount == 2` → swipe pass 3, then `CopyFullyProcessedDataOver`
//!   (`:538-621`) publishes `R2 → R` with the camera delta since the pre-render.
//! - `renderCount == 3..=RenderPhases` → `ShiftUnProcessedDataOver`
//!   (`:473-536`) pans the presented values with the camera, and a sky-colour
//!   change re-seeds the surface rows (`UpdateLightToSkyColor`, `:435-471`).
//!
//! The port stores the jagged array as one column-major `Vec` (`x * h + y`,
//! the `_states[x][y]` layout); the horizontal passes read it transposed, the
//! way `_axisFlipStates[y][x]` aliases the same objects (`Rebuild`, `:240-255`).
//!
//! Deviations (recorded in docs/ANALYSIS.md, S1-E-19 wave A):
//! - `FastParallel.For` chunks are ported as ONE sequential chunk per pass.
//!   The C# swipes wrap their whole body in `try/catch` (`:1091/1196/1277/1335`),
//!   so an out-of-bounds inner index aborts the rest of that thread's chunk;
//!   sequential = one chunk, first out-of-bounds aborts the pass. Writes never
//!   cross outer indices, so the chunking cannot change cell values.
//! - `_swipeRandom` / the scanner's `_random` are `CreateWithRandomSeed()` in
//!   vanilla (session-nondeterministic); the port pins deterministic seeds,
//!   the S1-A-6 `LightMap.blur_rng` precedent. Stream *order* is exact:
//!   `NextSeed()` after every swipe pass (`:1082`), `WithModifier(i*1000+j)`
//!   per wet cell (`:1250`), `Next(98,100)` per wet decay draw.
//! - The Wall-of-Flesh `gross` light block (`:678-717`) is not ported: it
//!   needs `Player.gross` / `Main.wofNPCIndex`, neither of which exists in
//!   the port yet. It is gated off in vanilla single-player without those
//!   conditions.
//! - The minimap light export (`TryUpdatingMapWithLight`, `:371-433`, wave B)
//!   is split along the C#'s own seam: the `ProcessArea` gate block
//!   (`:341-358`) runs INLINE in `process_area` (before `PreRenderPhase`
//!   resets the count, so the `renderCount == 3` gate sees the C# value),
//!   writing through [`MapLightExport`] — the `Main.mapTime/mapEnabled`
//!   counters, the `Main.Map` tile array and the `Main.updateMap` rect. The
//!   `Main.mapDelay` countdown (`Main.cs:229`, static ctor `= 2` at
//!   `:66504`, world-load re-arm `WorldGen.cs:6947`) lives on the engine;
//!   the frame driver resets it via [`LegacyLighting::reset_map_delay`] on
//!   world entry. The frame driver attaches the export only while a legacy
//!   mode is active; the modern path keeps `LightingEngine.ExportToMiniMap`
//!   (`WorldMap::update`). The per-frame `Main.mapTime` countdown
//!   (`Main.cs:61818-61821`) stays on the map side (`WorldMap::tick_map_time`);
//!   the export re-arms it (`:373`). The C# `try/catch` around the export
//!   (`:351-357`) becomes an abort-on-out-of-bounds early return — the
//!   re-arm at `:373` happens first either way, tiles written before the
//!   abort persist and `Main.updateMap` stays untouched, exactly like the
//!   C#. The export draws NO random numbers, so the swipe/scanner streams
//!   are untouched (the `map_export_keeps_the_rng_streams` witness).
//! - Mode 0 (`doColors_Mode0_Swipe`, `:1087-1190`) is unreachable through the
//!   mode selector — Color selects the NEW engine (`Lighting.cs:40-44`) — so
//!   only the Mode 1/2/3 swipes are ported; the Mode-0 water/honey decay table
//!   (`:866-930`) is kept for completeness of the White-mode factor block.

use super::{DynamicLight, LightMode, LightParams};
use crate::tile_light::{self, FastRandom};
use crate::tiledata as td;
use crate::world::World;

/// `LegacyLighting.MAX_TEMP_LIGHTS` (`:150`).
const MAX_TEMP_LIGHTS: usize = 2000;

/// One `LightingState` (`:63-89`): presented `r/g/b`, working `r2/g2/b2` and
/// the four medium flags the swipe decay reads.
#[derive(Clone, Copy, Default)]
struct State {
    r: f32,
    g: f32,
    b: f32,
    r2: f32,
    g2: f32,
    b2: f32,
    cracked: bool,
    stop: bool,
    wet: bool,
    honey: bool,
}

/// A world-tile rect as `[left, right, top, bottom]` (`RectArea`, `:14-31`;
/// right/bottom exclusive).
type Rect = [i32; 4];

/// The map-side state `TryUpdatingMapWithLight` (`:371-433`) reads and writes
/// — the `Main.Map` / `Main.mapTime` / `Main.mapEnabled` / `Main.updateMap`
/// seam. The frame driver borrows it from the app's `WorldMap` and attaches
/// it to [`LegacyLighting::process_area`]; `None` keeps the engine's light
/// behaviour identical (the export draws no random numbers).
pub struct MapLightExport<'a> {
    /// `Main.Map`'s tile array (`WorldMap._tiles`), row-major `y * w + x`.
    pub tiles: &'a mut [crate::map::MapTile],
    /// `Main.mapTime` (`Main.cs:803`) — the export countdown; the per-frame
    /// decrement (`Main.cs:61818-61821`) stays on the map side, the export
    /// re-arms it from `map_time_max` (`:373`).
    pub map_time: &'a mut i32,
    /// `Main.mapTimeMax` (`Main.cs:801`) — the gfx-quality refresh budget.
    pub map_time_max: i32,
    /// `Main.mapEnabled` (`Main.cs:813`) — the export gate (`:349`).
    pub map_enabled: bool,
    /// The tiles `Main.Map.UpdateLighting` changed (`WorldMap.cs:49-63`
    /// returns true) — the port's `MapRenderer.QueueChange` analog: the
    /// driver pushes these into the map texture's dirty list.
    pub changed: Vec<(i32, i32)>,
    /// `Main.updateMap` (`:432`) as `[x, y, w, h]` — set on a completed
    /// export; the C# `try/catch` abort (`:351-357`) leaves it untouched.
    pub update_map: Option<Rect>,
}

impl<'a> MapLightExport<'a> {
    pub fn new(
        tiles: &'a mut [crate::map::MapTile],
        map_time: &'a mut i32,
        map_time_max: i32,
        map_enabled: bool,
    ) -> Self {
        Self {
            tiles,
            map_time,
            map_time_max,
            map_enabled,
            changed: Vec::new(),
            update_map: None,
        }
    }
}

/// The per-frame `Main` state the legacy engine reads beyond `LightParams`.
#[derive(Clone, Copy, Debug)]
pub struct LegacyInputs {
    /// `LegacyLighting.RenderPhases` (`:112`, static init 4 at `:1439`); the
    /// gfx-quality pass writes it (`Main.cs:17085-17101`, ported by S1-B-6 as
    /// `GfxQuality::render_phases`).
    pub render_phases: i32,
    /// `Main.waterStyle` — the Mode-0 wet decay table (`:874-929`). White
    /// (Mode 1) ignores it (`:931-937` sets only `wet_r`).
    pub water_style: u8,
    /// `Main.player[Main.myPlayer].nightVision` (`:938/968`).
    pub night_vision: bool,
    /// `Main.player[Main.myPlayer].blind` (`:944/974`).
    pub blind: bool,
    /// `Main.player[Main.myPlayer].blackout` (`:950/980`).
    pub blackout: bool,
    /// `Main.player[Main.myPlayer].headcovered` (`:956/986`).
    pub headcovered: bool,
    /// `Main.gamePaused` — keeps the `AddLight` dictionary across frames
    /// (`:826-829`).
    pub game_paused: bool,
    /// `Main.GameViewMatrix.Translation` (`SpriteViewMatrix.cs:149-153`),
    /// read by the minimap light export (`:375-382`): the zoom frame's
    /// offset, `Round(screen/2 - (screen/2) / Round(zoom/(1/128))*(1/128))`.
    /// `[0, 0]` at zoom 1.
    pub view_translation: [f32; 2],
    /// `Netplay.Connection.StatusMax > 0` (`:341`) — while connecting, the
    /// export countdown is pinned to 1 every shift frame.
    pub net_connecting: bool,
}

impl Default for LegacyInputs {
    fn default() -> Self {
        Self {
            render_phases: 4,
            water_style: 0,
            night_vision: false,
            blind: false,
            blackout: false,
            headcovered: false,
            game_paused: false,
            view_translation: [0.0; 2],
            net_connecting: false,
        }
    }
}

/// The swipe decay factors `DoColors` computes per frame (`:866-994`).
#[derive(Clone, Copy)]
struct Decay {
    neg: f32,
    neg2: f32,
    neg3: f32,
    wet_r: f32,
    wet_g: f32,
    honey_r: f32,
}

/// Which axis a swipe pass walks: vertical reads `_states[x][y]`, horizontal
/// reads the `_axisFlipStates[y][x]` alias (`DoColors`' `JaggedArray` switch,
/// `:1032/1041/1050/1059`).
#[derive(Clone, Copy, PartialEq)]
enum Axis {
    Vertical,
    Horizontal,
}

/// `LegacyLighting` — the White/Retro/Trippy engine. Additive to the modern
/// [`super::LightMap`] path: the renderer selects it through the lighting-mode
/// setting and publishes its presented colours into the shared `LightMap`
/// (`present_into`), so every downstream slice/corner consumer keeps working.
/// The modern engine's maths, RNG draws and blur padding are untouched.
pub struct LegacyLighting {
    /// `LegacyLighting.Mode` (`:202`): 1 White, 2 Retro, 3 Trippy.
    mode: u8,
    /// `_rgb` (`:114`, set in `PreRenderPhase` `:662`): Mode 0 or 3.
    rgb: bool,
    /// `Lighting.OffScreenTiles` while the legacy engine is active — 40 for
    /// White, 23 for Retro/Trippy (`ProcessArea`, `:309-318`); the static
    /// initial value is 45 (`Lighting.cs:581`).
    off_screen_tiles: i32,
    /// `_offScreenTiles2` (`:116`): 34 White / 18 Retro-Trippy (`:311/316`).
    off_screen_tiles2: i32,
    /// `_oldSkyColor` / `_skyColor` (`:118-120`).
    old_sky_color: f32,
    sky_color: f32,
    /// `_requestedRect*` (`:122-128`) and `_expandedRect*` (`:154-160`).
    requested: Rect,
    expanded: Rect,
    /// `_states[x][y]` (`:130`) as one column-major buffer, `sw` columns of
    /// `sh` cells; `_axisFlipStates` (`:132`) is the transposed read of it.
    states: Vec<State>,
    sw: i32,
    sh: i32,
    /// `_minBoundArea` / `_requestedArea` / `_expandedArea` /
    /// `_offScreenTiles2ExpandedArea` (`:184-190`), set in `PreRenderPhase`.
    min_bound: Rect,
    requested_area: Rect,
    expanded_area: Rect,
    off2_area: Rect,
    /// `_scrX/_scrY` (`:138-140`): the camera tile at the last pre-render.
    scr_x: i32,
    scr_y: i32,
    /// `_tempLights` (`:152`): the `AddLight` dictionary, in insertion order
    /// (the `PreRenderPhase` foreach at `:789` walks it that way).
    temp_lights: Vec<((i32, i32), [f32; 3])>,
    /// The decay fields (`:162-178`), recomputed by `DoColors`.
    neg: f32,
    neg2: f32,
    neg3: f32,
    wet_r: f32,
    wet_g: f32,
    wet_b: f32,
    honey_r: f32,
    honey_g: f32,
    honey_b: f32,
    /// `_blueWave` / `_blueDir` (`:180-182`): the White-mode water shimmer.
    blue_wave: f32,
    blue_dir: f32,
    /// `Main.renderCount` as the legacy engine drives it; starts at 99
    /// (`Main.cs:66761`) so the first `ProcessArea` pre-renders immediately.
    render_count: i32,
    /// `_lastCameraPosition` (`:196`).
    last_camera: Option<[f32; 2]>,
    /// The camera size of the last `ProcessArea` — `Clear` (`:623-646`) and
    /// the array sizing read `Camera.UnscaledSize`.
    last_size: [f32; 2],
    /// `_swipeRandom` (`:198`, `CreateWithRandomSeed()` at `:1440` — pinned
    /// deterministically here, see the module note).
    swipe_rng: FastRandom,
    /// The legacy engine owns its `TileLightScanner` (`:192`); its `_random`
    /// advances once per `PreRenderPhase` (`Update()`, `:723` →
    /// `TileLightScanner.cs:56-59`), NOT once per frame like the modern path.
    /// Seeded from the frame's `LightParams.rng_seed` on first use.
    scanner_rng: Option<FastRandom>,
    /// `Main.mapDelay` (`Main.cs:229`, static ctor `= 2` at `:66504`,
    /// re-armed on world load `WorldGen.cs:6947`) — the export gate's
    /// countdown (`ProcessArea`, `:345-348`).
    map_delay: i32,
}

impl LegacyLighting {
    pub fn new(mode: LightMode) -> Self {
        Self {
            mode: legacy_mode_byte(mode),
            rgb: true,
            off_screen_tiles: 45,
            off_screen_tiles2: 35,
            old_sky_color: 0.0,
            sky_color: 0.0,
            requested: [0; 4],
            expanded: [0; 4],
            states: Vec::new(),
            sw: 0,
            sh: 0,
            min_bound: [0; 4],
            requested_area: [0; 4],
            expanded_area: [0; 4],
            off2_area: [0; 4],
            scr_x: 0,
            scr_y: 0,
            temp_lights: Vec::new(),
            neg: 0.04,
            neg2: 0.16,
            neg3: 0.08,
            wet_r: 0.16,
            wet_g: 0.16,
            wet_b: 0.16,
            honey_r: 0.16,
            honey_g: 0.16,
            honey_b: 0.16,
            blue_wave: 1.0,
            blue_dir: 1.0,
            render_count: 99,
            last_camera: None,
            last_size: [0.0; 2],
            swipe_rng: FastRandom::new(0),
            scanner_rng: None,
            map_delay: 2,
        }
    }

    /// `Main.mapDelay = 2` on world entry (`WorldGen.cs:6947`, the static
    /// ctor's initial value `Main.cs:66504`): the export gate's countdown
    /// re-arms. The engine owns the counter in the port (the modern map
    /// pass does not read it — see ANALYSIS, S1-E-19 wave B).
    pub fn reset_map_delay(&mut self) {
        self.map_delay = 2;
    }

    /// The `Lighting.Mode` setter's legacy half (`Lighting.cs:45-59`): select
    /// the engine mode and reset `Main.renderCount = 0`. The presented values
    /// are NOT cleared here — vanilla keeps them until the next cycle (the
    /// `NextLightMode` path adds [`Self::clear`]).
    pub fn set_mode(&mut self, mode: LightMode) {
        self.mode = legacy_mode_byte(mode);
        self.render_count = 0;
    }

    pub fn mode(&self) -> u8 {
        self.mode
    }

    /// `ILightingEngine.GetColor` (`LegacyLighting.cs:211-219`): the presented
    /// `R/G/B`, zero outside the expanded rect.
    pub fn get_color(&self, x: i32, y: i32) -> [f32; 3] {
        let [l, r, t, b] = self.expanded;
        if x < l || x >= r || y < t || y >= b {
            return [0.0; 3];
        }
        let (lx, ly) = (x - l, y - t);
        if lx >= self.sw || ly >= self.sh {
            return [0.0; 3];
        }
        let s = self.states[(lx * self.sh + ly) as usize];
        [s.r, s.g, s.b]
    }

    /// The processed area as `(x0, y0, w, h)` — the expanded rect.
    pub fn get_area(&self) -> (i32, i32, i32, i32) {
        let [l, r, t, b] = self.expanded;
        (l, t, r - l, b - t)
    }

    /// `ILightingEngine.Clear` (`:623-646`): drop the camera anchor and zero
    /// the presented colours over the screen-plus-margin region. Called by
    /// `Lighting.NextLightMode` (`Lighting.cs:172`) and the teleport fade
    /// (`Main.cs:62558`).
    pub fn clear(&mut self) {
        self.last_camera = None;
        let nx = (self.last_size[0] as i32) / 16 + self.off_screen_tiles * 2;
        let ny = (self.last_size[1] as i32) / 16 + self.off_screen_tiles * 2;
        for i in 0..nx.min(self.sw) {
            for j in 0..ny.min(self.sh) {
                let s = &mut self.states[(i * self.sh + j) as usize];
                s.r = 0.0;
                s.g = 0.0;
                s.b = 0.0;
            }
        }
    }

    /// `ILightingEngine.AddLight` (`:258-300`): accumulate one dynamic source
    /// into the dictionary. Bounds use the LAST processed requested rect and
    /// the current `OffScreenTiles` (vanilla's `AddLight` calls land between
    /// `ProcessArea` calls); the cap is 2000 entries. Non-RGB modes collapse
    /// the colour to its mean and keep the per-tile maximum of that mean
    /// (`:286-298` — the comparison reads `value.R` only).
    pub fn add_light(&mut self, x: i32, y: i32, color: [f32; 3]) {
        let lx = (x - self.requested[0] + self.off_screen_tiles) as f32;
        let ly = (y - self.requested[2] + self.off_screen_tiles) as f32;
        let max_x = self.last_size[0] / 16.0 + (self.off_screen_tiles * 2) as f32 + 10.0;
        let max_y = self.last_size[1] / 16.0 + (self.off_screen_tiles * 2) as f32 + 10.0;
        if lx < 0.0 || lx >= max_x || ly < 0.0 || ly >= max_y {
            return;
        }
        if self.temp_lights.len() == MAX_TEMP_LIGHTS {
            return;
        }
        let key = (x, y);
        if let Some(entry) = self.temp_lights.iter_mut().find(|(k, _)| *k == key) {
            if self.rgb {
                // `:270-285`: per-channel maximum.
                for c in 0..3 {
                    if entry.1[c] < color[c] {
                        entry.1[c] = color[c];
                    }
                }
            } else {
                // `:286-293`: the mean replaces the whole triplet when brighter.
                let mean = (color[0] + color[1] + color[2]) / 3.0;
                if entry.1[0] < mean {
                    entry.1 = [mean; 3];
                }
            }
        } else {
            let value = if self.rgb {
                color
            } else {
                [(color[0] + color[1] + color[2]) / 3.0; 3]
            };
            self.temp_lights.push((key, value));
        }
    }

    /// `ILightingEngine.ProcessArea` (`:302-369`). `camera_pos` is
    /// `Camera.UnscaledPosition` (the world-pixel top-left, `Camera.cs:8`) and
    /// `camera_size` is `Camera.UnscaledSize` (`Camera.cs:10`, the backbuffer
    /// size). The requested rect is `Main.GetAreaToLight`'s legacy branch
    /// (`Main.cs:63926-63939`). `lights` are the frame's `AddLight` sources;
    /// they are accumulated against the previous rect first, exactly where
    /// vanilla's between-frame `AddLight` calls sit. `map_export` is the
    /// `Main.Map` seam for the `:341-358` gate block (wave B); `None` keeps
    /// the light behaviour identical — the export draws no random numbers.
    pub fn process_area(
        &mut self,
        world: &World,
        camera_pos: [f32; 2],
        camera_size: [f32; 2],
        params: &LightParams,
        inputs: &LegacyInputs,
        lights: &[DynamicLight],
        mut map_export: Option<&mut MapLightExport<'_>>,
    ) {
        // The between-frame `AddLight` accumulation (`Lighting.cs:157-163`).
        for l in lights {
            self.add_light(l.x, l.y, l.rgb);
        }
        self.last_size = camera_size;
        self.ensure_size(camera_size);

        // `:304-308`: the sky colour is the mean of `Main.tileColor`'s BYTES
        // over 255 (`Main.cs:63179-63181` builds the bytes); the port's
        // `params.sky` is the float analogue, quantised back to bytes here.
        self.old_sky_color = self.sky_color;
        let tile_r = sky_byte(params.sky[0]);
        let tile_g = sky_byte(params.sky[1]);
        let tile_b = sky_byte(params.sky[2]);
        self.sky_color = (tile_r + tile_g + tile_b) / 3.0;

        // `:309-318`: the margins per mode family.
        if self.mode < 2 {
            self.off_screen_tiles2 = 34;
            self.off_screen_tiles = 40;
        } else {
            self.off_screen_tiles2 = 18;
            self.off_screen_tiles = 23;
        }

        // `Main.GetAreaToLight` (`Main.cs:63935-63938`), legacy branch.
        let req = [
            (camera_pos[0] / 16.0).floor() as i32 - 1,
            ((camera_pos[0] + camera_size[0]) / 16.0).floor() as i32 + 2,
            (camera_pos[1] / 16.0).floor() as i32 - 1,
            ((camera_pos[1] + camera_size[1]) / 16.0).floor() as i32 + 2,
        ];
        self.requested = req;
        let off = self.off_screen_tiles;
        self.expanded = [req[0] - off, req[1] + off, req[2] - off, req[3] + off];

        // `:327-333`.
        self.render_count += 1;
        let max_x = camera_size[0] as i32 / 16 + off * 2;
        let max_y = camera_size[1] as i32 / 16 + off * 2;
        if self.render_count < 3 {
            self.do_colors(inputs);
        }
        // `:334-363`.
        if self.render_count == 2 {
            self.copy_fully_processed(max_x, max_y, camera_pos);
        } else if let Some(last) = self.last_camera {
            self.shift_unprocessed(max_x, max_y, camera_pos, last);
            // `:341-344`: while connecting, the export countdown is pinned
            // to 1 every shift frame.
            if inputs.net_connecting {
                if let Some(m) = map_export.as_deref_mut() {
                    *m.map_time = 1;
                }
            }
            // `:345-358`: the mapDelay countdown, then the gated export.
            // The C# runs this whether or not the map exists (the counters
            // are `Main` statics); the port only has the seam when the map
            // is live, so `mapDelay` rides the engine either way.
            if self.map_delay > 0 {
                self.map_delay -= 1;
            } else if let Some(m) = map_export {
                if *m.map_time == 0 && m.map_enabled && self.render_count == 3 {
                    // `:351-357`: the try/catch — an abort keeps the re-arm
                    // and the tiles written so far, and skips `updateMap`.
                    self.try_updating_map_with_light(world, camera_pos, camera_size, inputs, m);
                }
            }
            if self.old_sky_color != self.sky_color {
                self.update_light_to_sky_color(world, tile_r, tile_g, tile_b);
            }
        }
        // `:364-367`.
        if self.render_count > inputs.render_phases {
            self.pre_render_phase(world, params, inputs, camera_pos, camera_size);
        }
        self.last_camera = Some(camera_pos);
    }

    /// `Rebuild`'s array sizing (`:233-255`): `(screen/16 + 90 + 10)` columns
    /// and rows, grown only (vanilla keeps the larger array on a shrink).
    /// The port folds the dictionary reset (`:225`) into the realloc — the
    /// renderer calls `Rebuild` on a resize, never per frame.
    fn ensure_size(&mut self, camera_size: [f32; 2]) {
        let num = camera_size[0] as i32 / 16 + 90 + 10;
        let num2 = camera_size[1] as i32 / 16 + 90 + 10;
        if self.sw >= num && self.sh >= num2 && !self.states.is_empty() {
            return;
        }
        let (sw, sh) = (self.sw.max(num), self.sh.max(num2));
        self.states = vec![State::default(); (sw * sh) as usize];
        self.sw = sw;
        self.sh = sh;
        self.temp_lights.clear();
    }

    /// `UpdateLightToSkyColor` (`:435-471`): when the sky colour changed, the
    /// surface rows of the expanded rect re-take the sunlight. The C# quirk
    /// at `:457-467` kept verbatim: inside the gate `R` is set
    /// UNCONDITIONALLY while `G`/`B` keep their own `< _skyColor` guards.
    fn update_light_to_sky_color(&mut self, world: &World, tr: f32, tg: f32, tb: f32) {
        let num = self.expanded[0].clamp(0, world.width - 1);
        let num2 = self.expanded[1].clamp(0, world.width - 1);
        let num3 = self.expanded[2].clamp(0, world.height - 1);
        let num4 = self.expanded[3].clamp(0, world.world_surface as i32 - 1);
        if !((num3 as f64) < world.world_surface) {
            return;
        }
        for i in num..num2 {
            for j in num3..num4 {
                let Some(tile) = world.tile(i, j) else {
                    continue;
                };
                let wall_light = (tile.wall as usize) < td::WALL_LIGHT.len()
                    && td::WALL_LIGHT[tile.wall as usize];
                let s = &mut self.states
                    [((i - self.expanded[0]) * self.sh + (j - self.expanded[2])) as usize];
                if (!tile.active() || !td::TILE_NO_SUN_LIGHT.has(tile.kind))
                    && s.r < self.sky_color
                    && tile.liquid < 200
                    && (wall_light || tile.wall == 73)
                {
                    s.r = tr;
                    if s.g < self.sky_color {
                        s.g = tg;
                    }
                    if s.b < self.sky_color {
                        s.b = tb;
                    }
                }
            }
        }
    }

    /// `TryUpdatingMapWithLight` (`:371-433`): the minimap light export. The
    /// presented `R/G/B` collapse to the max channel (×1.5 in White,
    /// `:416-419` — `IsColorOrWhiteMode`, `Mode < 2`, `:204`), quantise to a
    /// byte, take the sky-air 22 and the revealed-floor 22 arms (`:421-428`),
    /// and ride `Main.Map.UpdateLighting` (`WorldMap.cs:49-63`, the shared
    /// [`crate::map::update_map_lighting`] port). The window is the camera's
    /// SCALED view (`:375-382`, `Camera.ScaledPosition`/`ScaledSize`,
    /// `Camera.cs:12-14`) intersected with the requested rect, both clamped
    /// with the C#'s max-first `Utils.Clamp` (`Utils.cs:223-234`). The re-arm
    /// at `:373` happens FIRST — the C# `try/catch` (`:351-357`) aborts the
    /// walk on an out-of-bounds state/tile read but keeps it, the written
    /// tiles and the untouched `Main.updateMap`. No RNG draws here.
    fn try_updating_map_with_light(
        &self,
        world: &World,
        camera_pos: [f32; 2],
        camera_size: [f32; 2],
        inputs: &LegacyInputs,
        m: &mut MapLightExport<'_>,
    ) {
        // `:373`.
        *m.map_time = m.map_time_max;
        // `:374-382`: the scaled view rect. `Camera.ScaledSize = UnscaledSize
        // - Translation * 2` (`Camera.cs:14`), `ScaledPosition =
        // UnscaledPosition + Translation` (`:12`).
        let num = 40;
        let tr = inputs.view_translation;
        let num2 = (camera_size[0] - tr[0] * 2.0) as i32;
        let num3 = (camera_size[1] - tr[1] * 2.0) as i32;
        let vx = camera_pos[0] + tr[0];
        let vy = camera_pos[1] + tr[1];
        let mut value = (vx / 16.0).floor() as i32;
        let mut value2 = ((vx + num2 as f32) / 16.0).floor() as i32 + 1;
        let mut value3 = (vy / 16.0).floor() as i32;
        let mut value4 = ((vy + num3 as f32) / 16.0).floor() as i32 + 1;
        // `:383-394`: `Utils.Clamp` — max first, so a reversed range (a tiny
        // world) folds onto the max instead of panicking.
        let off = self.off_screen_tiles;
        let clamp = crate::wind::cs_clamp;
        value = clamp(value, off, world.width - off);
        value2 = clamp(value2, off, world.width - off);
        value3 = clamp(value3, off, world.height - off);
        value4 = clamp(value4, off, world.height - off);
        let mut value5 = clamp(self.requested[0], num, world.width - num);
        let mut value6 = clamp(self.requested[1], num, world.width - num);
        let mut value7 = clamp(self.requested[2], num, world.height - num);
        let mut value8 = clamp(self.requested[3], num, world.height - num);
        value5 = clamp(value5, value, value2);
        value6 = clamp(value6, value, value2);
        value7 = clamp(value7, value3, value4);
        value8 = clamp(value8, value3, value4);
        // `:396-431`: x outer, y inner — the jagged read is
        // `_states[i - _requestedRectLeft + OffScreenTiles]
        //          [j - _requestedRectTop + OffScreenTiles]`.
        let color_or_white = self.mode < 2;
        for i in value5..value6 {
            let lx = i - self.requested[0] + off;
            // `:398`: the COLUMN is read once, BEFORE the inner loop — an
            // out-of-range column throws (→ the `:355` catch aborts the
            // whole walk) even when the y range is empty.
            if lx < 0 || lx >= self.sw {
                return;
            }
            for j in value7..value8 {
                let ly = j - self.requested[2] + off;
                // The C# `array[...]` / `Main.tile[i, j]` reads throw on an
                // out-of-range index → the `:355` catch aborts the walk.
                if ly < 0 || ly >= self.sh {
                    return;
                }
                let Some(tile) = world.tile(i, j) else {
                    return;
                };
                let s = self.states[(lx * self.sh + ly) as usize];
                // `:403-415`: the max channel of the PRESENTED values.
                let mut num4 = 0.0f32;
                if s.r > num4 {
                    num4 = s.r;
                }
                if s.g > num4 {
                    num4 = s.g;
                }
                if s.b > num4 {
                    num4 = s.b;
                }
                // `:416-419`: White brightens the export 1.5x.
                if color_or_white {
                    num4 *= 1.5;
                }
                // `:420`: `(byte)Math.Min(255f, num4 * 255f)` — the float
                // min then the byte truncation.
                let mut b = (num4 * 255.0).min(255.0) as u8;
                // `:421-424`: sky air keeps the 22 ambient.
                if (j as f64) < world.world_surface
                    && !tile.active()
                    && tile.wall == 0
                    && tile.liquid == 0
                {
                    b = 22;
                }
                // `:425-428`: a revealed tile never drops below 22.
                let old_light = m.tiles[world.index(i, j)].light;
                if (b > 18 || old_light > 0) && b < 22 {
                    b = 22;
                }
                // `:429`: `Main.Map.UpdateLighting(i, j, b)`.
                if crate::map::update_map_lighting(world, m.tiles, i, j, b) {
                    m.changed.push((i, j));
                }
            }
        }
        // `:432`: `Main.updateMap = new Rectangle(value5, value7, w, h)`.
        m.update_map = Some([value5, value7, value6 - value5, value8 - value7]);
    }

    /// `ShiftUnProcessedDataOver` (`:473-536`): pan the PRESENTED values with
    /// the camera (tile deltas, jumps over 5 tiles ignored). Only `R/G/B`
    /// move — the working set and flags stay (`:531-533`).
    fn shift_unprocessed(&mut self, max_x: i32, max_y: i32, camera_pos: [f32; 2], last: [f32; 2]) {
        let mut num = (camera_pos[0] / 16.0).floor() as i32 - (last[0] / 16.0).floor() as i32;
        if num > 5 || num < -5 {
            num = 0;
        }
        let (step_x, start_x, end_x) = if num < 0 {
            num = -num;
            (-1, max_x, num)
        } else {
            (1, 0, max_x - num)
        };
        let mut num5 = (camera_pos[1] / 16.0).floor() as i32 - (last[1] / 16.0).floor() as i32;
        if num5 > 5 || num5 < -5 {
            num5 = 0;
        }
        let (step_y, start_y, end_y) = if num5 < 0 {
            num5 = -num5;
            (-1, max_y, num5)
        } else {
            (1, 0, max_y - num5)
        };
        if num == 0 && num5 == 0 {
            return;
        }
        let sh = self.sh;
        let mut i = start_x;
        while i != end_x {
            let src_i = i + num * step_x;
            let mut j = start_y;
            while j != end_y {
                let src_j = j + num5 * step_y;
                if src_i >= 0 && src_i < self.sw && src_j >= 0 && src_j < sh {
                    let src = self.states[(src_i * sh + src_j) as usize];
                    let dst = &mut self.states[(i * sh + j) as usize];
                    dst.r = src.r;
                    dst.g = src.g;
                    dst.b = src.b;
                }
                j += step_y;
            }
            i += step_x;
        }
    }

    /// `CopyFullyProcessedDataOver` (`:538-621`): publish the working set into
    /// the presented one with the camera delta since the pre-render. Deltas
    /// over 16 tiles collapse to 0 — positive only, the C# never clamps the
    /// negative side (`:543-550`). The inner bounds guard keeps the C# quirk
    /// at `:583/608`: the CONDITION reads the x delta (`num`) while the
    /// SUBTRACTION uses the y delta (`num2`).
    fn copy_fully_processed(&mut self, max_x: i32, max_y: i32, camera_pos: [f32; 2]) {
        let mut num = (camera_pos[0] / 16.0).floor() as i32 - self.scr_x;
        let mut num2 = (camera_pos[1] / 16.0).floor() as i32 - self.scr_y;
        if num > 16 {
            num = 0;
        }
        if num2 > 16 {
            num2 = 0;
        }
        let (mut x0, mut x1) = (0, max_x);
        let (mut y0, mut y1) = (0, max_y);
        if num < 0 {
            x0 -= num;
        } else {
            x1 -= num;
        }
        if num2 < 0 {
            y0 -= num2;
        } else {
            y1 -= num2;
        }
        let (sw, sh) = (self.sw, self.sh);
        let rgb = self.rgb;
        // The outer x bound is ASYMMETRIC (`:573` vs `:598`): the RGB branch
        // starts from the FRESH `maxLightArrayX` (un-subtracted), the non-RGB
        // branch from `num4` (already `-= num` above). Both then clamp by
        // `sw - num - 1`. The inner y guard (`:582-586` / `:607-611`) is the
        // shared quirk: condition on `num`, subtraction on `num2`.
        let x1_rgb = {
            let mut b = max_x;
            if sw <= b + num {
                b = sw - num - 1;
            }
            b
        };
        let x1_nonrgb = {
            let mut b = x1;
            if sw <= b + num {
                b = sw - num - 1;
            }
            b
        };
        let mut y1 = y1;
        if sh <= y1 + num {
            y1 = sh - num2 - 1;
        }
        let x_end = if rgb { x1_rgb } else { x1_nonrgb };
        for i in x0..x_end {
            let src_i = i + num;
            if src_i < 0 || src_i >= sw {
                continue;
            }
            for j in y0..y1 {
                let src_j = j + num2;
                if src_j < 0 || src_j >= sh {
                    continue;
                }
                let src = self.states[(src_i * sh + src_j) as usize];
                let dst = &mut self.states[(i * sh + j) as usize];
                if rgb {
                    dst.r = src.r2;
                    dst.g = src.g2;
                    dst.b = src.b2;
                } else {
                    // `:616-618`: White publishes the R2 luminance to all three.
                    dst.r = src.r2;
                    dst.g = src.r2;
                    dst.b = src.r2;
                }
            }
        }
    }

    /// `PreRenderPhase` (`:648-849`): the once-per-cycle reseed.
    fn pre_render_phase(
        &mut self,
        world: &World,
        params: &LightParams,
        inputs: &LegacyInputs,
        camera_pos: [f32; 2],
        camera_size: [f32; 2],
    ) {
        let off = self.off_screen_tiles;
        let bound_x = camera_size[0] as i32 / 16 + off * 2 + 10;
        let bound_y = camera_size[1] as i32 / 16 + off * 2 + 10;
        // `:662`: `_rgb = Mode == 0 || Mode == 3`.
        self.rgb = self.mode == 0 || self.mode == 3;
        // `:663-677`: clear the working set and the flags (the loop bounds
        // are the camera-derived size; the array is always larger, `:233`).
        for i in 0..bound_x.min(self.sw) {
            for j in 0..bound_y.min(self.sh) {
                let s = &mut self.states[(i * self.sh + j) as usize];
                s.r2 = 0.0;
                s.g2 = 0.0;
                s.b2 = 0.0;
                s.cracked = false;
                s.stop = false;
                s.wet = false;
                s.honey = false;
            }
        }
        // `:678-717`: the Wall-of-Flesh `gross` light block needs
        // `Main.wofNPCIndex` + `Player.gross` — not ported (module note).

        // `:718-727`: the scanner export over the expanded rect clamped to
        // `[5, maxTiles-1]`. `TileLightScanner.Update()` advances the scanner
        // stream once (`TileLightScanner.cs:56-59`).
        let cx0 = self.expanded[0].clamp(5, world.width - 1);
        let cx1 = self.expanded[1].clamp(5, world.width - 1);
        let cy0 = self.expanded[2].clamp(5, world.height - 1);
        let cy1 = self.expanded[3].clamp(5, world.height - 1);
        let mut rng = match self.scanner_rng {
            Some(r) => r,
            None => FastRandom::new(params.rng_seed),
        };
        rng.next_seed_in_place();
        self.scanner_rng = Some(rng);

        // The local bound tracker (`:654-661`): starts inverted.
        let (mut min_x, mut max_x) = (bound_x, 0);
        let (mut min_y, mut max_y) = (bound_y, 0);
        for m in cx0..cx1 {
            for n in cy0..cy1 {
                let lx = m - self.expanded[0];
                let ly = n - self.expanded[2];
                let idx = (lx * self.sh + ly) as usize;
                // `IsTileNullOrTouchingNull` (`TileLightScanner.cs:43-54`):
                // the port's dense tile array never holds null, so the gate
                // reduces to `WorldGen.InWorld(x, y, 1)`.
                let touching_null = m < 1 || m >= world.width - 1 || n < 1 || n >= world.height - 1;
                let (color, mask) = if touching_null {
                    ([0.0; 3], Medium::Air)
                } else {
                    let tile = world.tile(m, n).expect("in-world tile");
                    (
                        legacy_seed(world, tile, m, n, params, &mut rng),
                        legacy_mask(
                            tile,
                            world.invisible_block(m, n),
                            params.draw_invisible_walls,
                        ),
                    )
                };
                let s = &mut self.states[idx];
                if self.rgb {
                    s.r2 = color[0];
                    s.g2 = color[1];
                    s.b2 = color[2];
                } else {
                    // `:748`: White collapses the seed to its mean.
                    let mean = (color[0] + color[1] + color[2]) / 3.0;
                    s.r2 = mean;
                    s.g2 = mean;
                    s.b2 = mean;
                }
                // `:750-765`: the mask flags.
                match mask {
                    Medium::Cracked => s.cracked = true,
                    Medium::Solid => s.stop = true,
                    Medium::Water => s.wet = true,
                    Medium::Honey => {
                        s.wet = true;
                        s.honey = true;
                    }
                    Medium::Air => {}
                }
                // `:766-786`: the lit-cell bound (non-RGB reads R2 only).
                if s.r2 > 0.0 || (self.rgb && (s.g2 > 0.0 || s.b2 > 0.0)) {
                    min_x = min_x.min(lx);
                    max_x = max_x.max(lx + 1);
                    min_y = min_y.min(ly);
                    max_y = max_y.max(ly + 1);
                }
            }
        }
        // `:789-825`: the `AddLight` dictionary, per-cell maximum, expanding
        // the bound.
        let max_lx = camera_size[0] / 16.0 + (off * 2) as f32 + 10.0;
        let max_ly = camera_size[1] / 16.0 + (off * 2) as f32 + 10.0;
        for ((kx, ky), value) in &self.temp_lights {
            let lx = kx - self.requested[0] + off;
            let ly = ky - self.requested[2] + off;
            if lx >= 0
                && (lx as f32) < max_lx
                && ly >= 0
                && (ly as f32) < max_ly
                && lx < self.sw
                && ly < self.sh
            {
                let s = &mut self.states[(lx * self.sh + ly) as usize];
                s.r2 = s.r2.max(value[0]);
                s.g2 = s.g2.max(value[1]);
                s.b2 = s.b2.max(value[2]);
                min_x = min_x.min(lx);
                max_x = max_x.max(lx + 1);
                min_y = min_y.min(ly);
                max_y = max_y.max(ly + 1);
            }
        }
        // `:826-829`: the dictionary survives a paused frame.
        if !inputs.game_paused {
            self.temp_lights.clear();
        }
        // `:836-846`: the areas, the camera anchor, the count reset.
        self.min_bound = [
            min_x + self.expanded[0],
            max_x + self.expanded[0],
            min_y + self.expanded[2],
            max_y + self.expanded[2],
        ];
        self.requested_area = self.requested;
        self.expanded_area = self.expanded;
        let off2 = self.off_screen_tiles2;
        self.off2_area = [
            self.requested[0] - off2,
            self.requested[1] + off2,
            self.requested[2] - off2,
            self.requested[3] + off2,
        ];
        self.scr_x = (camera_pos[0] / 16.0).floor() as i32;
        self.scr_y = (camera_pos[1] / 16.0).floor() as i32;
        self.render_count = 0;
        // `:848`.
        self.do_colors(inputs);
    }

    /// `DoColors` (`:851-1085`): the per-frame decay factors and the swipe
    /// passes the current `renderCount` selects.
    fn do_colors(&mut self, inputs: &LegacyInputs) {
        if self.mode < 2 {
            // `:855-865`: the blue wave ping-pongs 0.97..1.0 by 0.0001.
            self.blue_wave += self.blue_dir * 0.0001;
            if self.blue_wave > 1.0 {
                self.blue_wave = 1.0;
                self.blue_dir = -1.0;
            } else if self.blue_wave < 0.97 {
                self.blue_wave = 0.97;
                self.blue_dir = 1.0;
            }
            if self.rgb {
                // `:866-930` (Mode 0 — unreachable through the mode selector,
                // kept for the table's completeness).
                self.neg = 0.91;
                self.neg2 = 0.56;
                self.neg3 = 0.8;
                self.honey_g = 0.7 * self.neg * self.blue_wave;
                self.honey_r = 0.75 * self.neg * self.blue_wave;
                self.honey_b = 0.6 * self.neg * self.blue_wave;
                let (wr, wg, wb) = water_decay_rgb(inputs.water_style);
                self.wet_r = wr * self.neg * self.blue_wave;
                self.wet_g = wg * self.neg * self.blue_wave;
                self.wet_b = wb * self.neg * self.blue_wave;
            } else {
                // `:931-937`: White. Only `wet_r` is set — the honey factors
                // keep their previous values (the C# quirk at `:174-178`:
                // honey in White mode decays with the stale 0.16).
                self.neg = 0.9;
                self.neg2 = 0.54;
                self.neg3 = 0.8;
                self.wet_r = 0.95 * self.neg * self.blue_wave;
            }
            // `:938-961`: the buff multipliers.
            if inputs.night_vision {
                self.neg *= 1.03;
                self.neg2 *= 1.03;
                self.neg3 *= 1.03;
            }
            if inputs.blind {
                self.neg *= 0.95;
                self.neg2 *= 0.95;
                self.neg3 *= 0.95;
            }
            if inputs.blackout {
                self.neg *= 0.85;
                self.neg2 *= 0.85;
                self.neg3 *= 0.85;
            }
            if inputs.headcovered {
                self.neg *= 0.85;
                self.neg2 *= 0.85;
                self.neg3 *= 0.85;
            }
        } else {
            // `:963-994`: Retro/Trippy — the legacy SUBTRACTIVE falloff.
            self.neg = 0.04;
            self.neg2 = 0.16;
            self.neg3 = 0.08;
            if inputs.night_vision {
                self.neg -= 0.013;
                self.neg2 -= 0.04;
                self.neg3 -= 0.04;
            }
            if inputs.blind {
                self.neg += 0.03;
                self.neg2 += 0.06;
                self.neg3 += 0.06;
            }
            if inputs.blackout {
                self.neg += 0.09;
                self.neg2 += 0.18;
                self.neg3 += 0.18;
            }
            if inputs.headcovered {
                self.neg += 0.09;
                self.neg2 += 0.18;
                self.neg3 += 0.18;
            }
            self.wet_r = self.neg * 1.2;
            self.wet_g = self.neg * 1.1;
        }
        // `:995-1015`: the pass range per renderCount.
        let (p0, p1) = match self.render_count {
            0 => (0, 1),
            1 => (1, 3),
            2 => (3, 4),
            _ => (0, 0),
        };
        let left = self.expanded_area[0];
        let top = self.expanded_area[2];
        let phases = inputs.render_phases;
        let decay = Decay {
            neg: self.neg,
            neg2: self.neg2,
            neg3: self.neg3,
            wet_r: self.wet_r,
            wet_g: self.wet_g,
            honey_r: self.honey_r,
        };
        for pass in p0..p1 {
            // `:1023-1061`: the swipe bounds per pass.
            let (mut in1s, mut in2s, in1e, in2e, mut out0, out1, axis) = match pass {
                0 => (
                    self.min_bound[2] - top,
                    self.min_bound[3] - top,
                    self.requested_area[3] + phases - top,
                    self.requested_area[2] - phases - top,
                    self.min_bound[0] - left,
                    self.min_bound[1] - left,
                    Axis::Vertical,
                ),
                1 => (
                    self.expanded_area[0] - left,
                    self.expanded_area[1] - left,
                    self.requested_area[1] + phases - left,
                    self.requested_area[0] - phases - left,
                    self.expanded_area[2] - top,
                    self.expanded_area[3] - top,
                    Axis::Horizontal,
                ),
                2 => (
                    self.off2_area[2] - top,
                    self.off2_area[3] - top,
                    self.requested_area[3] + phases - top,
                    self.requested_area[2] - phases - top,
                    self.off2_area[0] - left,
                    self.off2_area[1] - left,
                    Axis::Vertical,
                ),
                _ => (
                    self.off2_area[0] - left,
                    self.off2_area[1] - left,
                    self.requested_area[1] + phases - left,
                    self.requested_area[0] - phases - left,
                    self.off2_area[2] - top,
                    self.off2_area[3] - top,
                    Axis::Horizontal,
                ),
            };
            // `:1062-1073`: the bound clamps.
            if in1s > in1e {
                in1s = in1e;
            }
            if in2s < in2e {
                in2s = in2e;
            }
            if out0 > out1 {
                out0 = out1;
            }
            // `:1074-1081`: the mode swipe. Mode 0 is unreachable through the
            // selector (Color uses the new engine) — the C# `_ =>` fallthrough
            // picks Mode 0 (`:1080`); the port keeps the Mode-1 behaviour for
            // the unreachable default arm (documented, no reachable effect).
            swipe_pass(
                &mut self.states,
                self.sw,
                self.sh,
                axis,
                out0,
                out1,
                in1s,
                in1e,
                in2s,
                in2e,
                self.mode,
                &decay,
                &mut self.swipe_rng,
            );
            // `:1082`: one `NextSeed()` per pass, even without wet draws.
            self.swipe_rng.next_seed_in_place();
        }
    }

    /// Publish the presented colours into the shared [`super::LightMap`] so
    /// the renderer's slice/corner consumers read the legacy result. Called
    /// once per frame after `process_area`.
    pub fn present_into(&self, map: &mut super::LightMap) {
        let (x0, y0, w, h) = self.get_area();
        let n = (w.max(0) * h.max(0)) as usize;
        let mut colors = Vec::with_capacity(n);
        let mut lit = 0u32;
        for ly in 0..h.max(0) {
            for lx in 0..w.max(0) {
                let s = self.states[(lx * self.sh + ly) as usize];
                let c = [s.r, s.g, s.b];
                if c[0] > 0.0 || c[1] > 0.0 || c[2] > 0.0 {
                    lit += 1;
                }
                colors.push(c);
            }
        }
        map.present_legacy(x0, y0, w, h, colors, lit);
    }
}

/// `Lighting.Mode` → `LegacyLighting.Mode` (`Lighting.cs:45-56`): White 1,
/// Retro 2, Trippy 3. Color maps to 0 (the legacy engine is idle in that
/// mode; the setter still writes `LegacyEngine.Mode = 0`, `:42`).
fn legacy_mode_byte(mode: LightMode) -> u8 {
    match mode {
        LightMode::Color => 0,
        LightMode::White => 1,
        LightMode::Retro => 2,
        LightMode::Trippy => 3,
    }
}

/// `(float)(int)Main.tileColor.R / 255f` (`:305-307`): the float sky colour
/// quantised to the byte the C# reads.
fn sky_byte(v: f32) -> f32 {
    (v.clamp(0.0, 1.0) * 255.0) as u8 as f32 / 255.0
}

/// The Mode-0 water decay table (`DoColors`, `:874-929`): the `(R, G, B)`
/// multipliers per `Main.waterStyle`, before the `negLight * blueWave` scale.
fn water_decay_rgb(style: u8) -> (f32, f32, f32) {
    match style {
        0 | 1 | 7 | 8 => (0.88, 0.96, 1.015),
        2 => (0.94, 0.85, 1.01),
        3 => (0.84, 0.95, 1.015),
        4 => (0.9, 0.86, 1.01),
        5 => (0.84, 0.99, 1.01),
        6 => (0.83, 0.93, 0.98),
        9 => (1.0, 0.88, 0.84),
        10 => (0.83, 1.0, 1.0),
        12 => (0.95, 0.98, 0.85),
        _ => (0.0, 0.0, 0.0),
    }
}

/// The legacy engine's medium classification — `TileLightScanner.GetTileMask`
/// (`TileLightScanner.cs:66-85`). Unlike the modern blur's `mask_of`, the C#
/// gate includes `!tile.inActive()` (`:68`): an actuated block stops blocking
/// light. Kept separate so the modern path stays byte-identical.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Medium {
    Air,
    Solid,
    Cracked,
    Water,
    Honey,
}

fn legacy_mask(tile: &crate::tile::Tile, invisible_block: bool, draw_invisible: bool) -> Medium {
    if tile_light::light_blocked(tile, invisible_block, draw_invisible)
        && tile.kind != 131
        && !tile.inactive()
        && tile.slope() == 0
    {
        if td::SET_CRACKED_BRICKS.has(tile.kind) {
            return Medium::Cracked;
        }
        return Medium::Solid;
    }
    if tile.liquid > 128 && tile.liquid_kind() != crate::tile::LiquidKind::Lava {
        return match tile.liquid_kind() {
            crate::tile::LiquidKind::Honey => Medium::Honey,
            _ => Medium::Water,
        };
    }
    Medium::Air
}

/// `TileLightScanner.GetTileLight` (`TileLightScanner.cs:87-106`) as the
/// legacy engine consumes it: the surface gate is the GLOBAL
/// `(int)Main.worldSurface` (`:92`), not the modern path's per-column cutoff,
/// and the hell gate is `Main.UnderworldLayer = maxTilesY - 200`
/// (`Main.cs:2790`). The per-type arms are the shared `tile_light` port.
fn legacy_seed(
    world: &World,
    tile: &crate::tile::Tile,
    x: i32,
    y: i32,
    p: &LightParams,
    rng: &mut FastRandom,
) -> [f32; 3] {
    let mut c = [0.0f32; 3];
    let inv = tile_light::InvisibleBits {
        block: world.invisible_block(x, y),
        wall: world.invisible_wall(x, y),
    };
    let mut local = rng.with_modifier(x, y);
    if y <= world.world_surface as i32 {
        tile_light::apply_surface_light(world, tile, x, y, &mut local, p, inv, &mut c);
    } else if y > world.height - 200 {
        tile_light::apply_hell_light(world, tile, x, y, p, inv, &mut c);
    }
    tile_light::apply_wall_light(tile, world.wall_paint(x, y), inv, &mut local, p, &mut c);
    if tile.active() {
        tile_light::apply_tile_light(world, tile, x, y, &mut local, p, &mut c);
    }
    super::liquid_light(tile, &mut c, p);
    c
}

/// One swipe pass — `doColors_Mode1_Swipe` (`:1192-1271`, White),
/// `doColors_Mode2_Swipe` (`:1273-1329`, Retro) and `doColors_Mode3_Swipe`
/// (`:1331-1435`, Trippy). The forward sweep runs `[in1s, in1e)` then the
/// backward sweep `[in2s, in2e)` with step -1, per outer index. The C#
/// `try/catch` abort semantics: an out-of-bounds inner index ends the whole
/// pass (see the module note on the single sequential chunk).
#[allow(clippy::too_many_arguments)]
fn swipe_pass(
    states: &mut [State],
    sw: i32,
    sh: i32,
    axis: Axis,
    out0: i32,
    out1: i32,
    in1s: i32,
    in1e: i32,
    in2s: i32,
    in2e: i32,
    mode: u8,
    decay: &Decay,
    rng: &mut FastRandom,
) {
    let len = if axis == Axis::Vertical { sh } else { sw };
    let outer_len = if axis == Axis::Vertical { sw } else { sh };
    let idx = |outer: i32, inner: i32| -> usize {
        if axis == Axis::Vertical {
            (outer * sh + inner) as usize
        } else {
            (inner * sh + outer) as usize
        }
    };
    for (start, end, step) in [(in1s, in1e, 1i32), (in2s, in2e, -1i32)] {
        for outer in out0..out1 {
            if outer < 0 || outer >= outer_len {
                // The C# jagged read would throw → the catch aborts the pass.
                return;
            }
            let mut carry = [0.0f32; 3];
            let mut j = start;
            while j != end {
                if j < 0 || j >= len {
                    return; // `array[j]` out of bounds → catch.
                }
                let i = idx(outer, j);
                match mode {
                    2 => {
                        // Retro: subtractive falloff, no neighbour gate
                        // (`:1302-1315`).
                        let s = &mut states[i];
                        if s.r2 > carry[0] {
                            carry[0] = s.r2;
                        } else {
                            if carry[0] <= 0.0 {
                                j += step;
                                continue;
                            }
                            s.r2 = carry[0];
                        }
                        carry[0] -= if s.stop {
                            decay.neg2
                        } else if s.cracked {
                            decay.neg3
                        } else if s.wet {
                            decay.wet_r
                        } else {
                            decay.neg
                        };
                    }
                    3 => {
                        // Trippy: per-channel subtractive falloff with the
                        // `<= 0` done gate (`:1362-1421`). The StopLight
                        // subtraction is ADDITIVE to the channel's own decay
                        // (`:1379-1383`), and the B channel has no wet arm
                        // (`:1421`) — both C# quirks kept.
                        let s = &mut states[i];
                        for c in 0..3 {
                            let v = match c {
                                0 => &mut s.r2,
                                1 => &mut s.g2,
                                _ => &mut s.b2,
                            };
                            if *v > carry[c] {
                                carry[c] = *v;
                            } else if carry[c] <= 0.0 {
                                continue;
                            } else {
                                *v = carry[c];
                            }
                            if s.stop {
                                carry[c] -= decay.neg2;
                            }
                            carry[c] -= if s.cracked {
                                decay.neg3
                            } else if s.wet && c == 0 {
                                decay.wet_r
                            } else if s.wet && c == 1 {
                                decay.wet_g
                            } else {
                                decay.neg
                            };
                        }
                    }
                    _ => {
                        // White (Mode 1): the R2 luminance channel only
                        // (`:1222-1257`). Mode 0 falls through here — it is
                        // unreachable through the selector.
                        let s = &mut states[i];
                        if s.r2 > carry[0] {
                            carry[0] = s.r2;
                        } else {
                            if carry[0] <= 0.0185 {
                                j += step;
                                continue;
                            }
                            if s.r2 < carry[0] {
                                s.r2 = carry[0];
                            }
                        }
                        // The neighbour gate reads `array[j+step].R2`
                        // (`:1238`) — out of bounds aborts AFTER the raise.
                        let nj = j + step;
                        if nj < 0 || nj >= len {
                            return;
                        }
                        if !(states[idx(outer, nj)].r2 > carry[0]) {
                            let s = &states[i];
                            carry[0] *= if s.stop {
                                decay.neg2
                            } else if s.cracked {
                                decay.neg3
                            } else if s.wet {
                                // `:1250-1251`: one WithModifier per wet cell,
                                // one Next(98,100) draw per decay. The cast is
                                // the C# `(ulong)(i * 1000 + j)` — wrapping.
                                let mut r = rng.with_modifier_u64(
                                    (outer.wrapping_mul(1000).wrapping_add(j)) as i32 as u64,
                                );
                                let jitter = r.next_range(98, 100) as f32 * 0.01;
                                if s.honey {
                                    decay.honey_r * jitter
                                } else {
                                    decay.wet_r * jitter
                                }
                            } else {
                                decay.neg
                            };
                        }
                    }
                }
                j += step;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lighting::{LightMap, LightParams};
    use crate::tile::{FLAG_ACTIVE, FLAG_LIQUID_LAVA};

    /// A small fixed world: no sunlight, one torch at (48, 48).
    fn torch_world() -> World {
        let mut w = World::new("legacy".into(), 96, 96);
        w.world_surface = 0.0;
        let i = w.index(48, 48);
        w.tiles[i].kind = 4;
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.tiles[i].frame_x = 0;
        w.tiles[i].frame_y = 0;
        w
    }

    fn camera() -> ([f32; 2], [f32; 2]) {
        // The torch centred in a 320x240 backbuffer.
        ([48.0 * 16.0 - 160.0, 48.0 * 16.0 - 120.0], [320.0, 240.0])
    }

    /// Run the engine until a fresh pre-render has been published (count 2
    /// after a count-0 reset). A fresh engine (count 99) publishes on call 3;
    /// after `set_mode` (count 0) the stale working set publishes first, so
    /// the loop must wait for the NEXT pre-render — `phases + 3` calls.
    fn run_cycle(engine: &mut LegacyLighting, w: &World, inputs: &LegacyInputs) {
        let (pos, size) = camera();
        let params = LightParams::default();
        let mut prerendered = false;
        for _ in 0..32 {
            engine.process_area(w, pos, size, &params, inputs, &[], None);
            if engine.render_count == 0 {
                prerendered = true;
            }
            if prerendered && engine.render_count == 2 {
                break;
            }
        }
        assert!(prerendered && engine.render_count == 2, "cycle published");
    }

    #[test]
    fn retro_swipe_falloff_table_is_exact() {
        // doColors_Mode2_Swipe (`:1273-1329`): a 1.0 seed subtracts 0.04 per
        // air cell; the backward sweep raises from the other side. Hand-derived
        // over one column, forward [0,8), backward [11,4).
        let (sw, sh) = (1, 12);
        let mut states = vec![State::default(); (sw * sh) as usize];
        states[2].r2 = 1.0;
        let decay = Decay {
            neg: 0.04,
            neg2: 0.16,
            neg3: 0.08,
            wet_r: 0.048,
            wet_g: 0.044,
            honey_r: 0.16,
        };
        let mut rng = FastRandom::new(0);
        swipe_pass(
            &mut states,
            sw,
            sh,
            Axis::Vertical,
            0,
            1,
            0,
            8,
            11,
            4,
            2,
            &decay,
            &mut rng,
        );
        let mut expect = [0.0f32; 12];
        expect[2] = 1.0;
        let mut c = 1.0f32;
        for k in 3..8 {
            c -= 0.04;
            expect[k] = c;
        }
        for k in 0..12 {
            assert_eq!(states[k].r2, expect[k], "retro cell {k}");
        }
        // The solid gate subtracts 0.16 instead (`:1315`).
        let mut states2 = vec![State::default(); 12];
        states2[2].r2 = 1.0;
        states2[4].stop = true;
        swipe_pass(
            &mut states2,
            1,
            12,
            Axis::Vertical,
            0,
            1,
            0,
            8,
            11,
            4,
            2,
            &decay,
            &mut rng,
        );
        // Forward: cell3 = 0.96, carry 0.92; cell4 raised to 0.92 then the
        // stop gate drops the carry to 0.76; cell5 = 0.76, cell6 = 0.72,
        // cell7 = 0.68. Backward from 0 raises nothing above the forward peaks.
        assert_eq!(states2[3].r2, 1.0 - 0.04);
        assert_eq!(states2[4].r2, 1.0 - 0.04 - 0.04);
        assert_eq!(states2[5].r2, 1.0 - 0.04 - 0.04 - 0.16);
    }

    #[test]
    fn white_swipe_multiplies_and_does_not_bleed_colour() {
        // doColors_Mode1_Swipe (`:1192-1271`): the carry multiplies by 0.9
        // per air cell and ONLY R2 is touched — G2/B2 keep their seed values
        // (the no-colour-bleed property of White mode).
        let mut states = vec![State::default(); 12];
        states[2].r2 = 1.0;
        states[5].g2 = 0.5;
        states[5].b2 = 0.2;
        let decay = Decay {
            neg: 0.9,
            neg2: 0.54,
            neg3: 0.8,
            wet_r: 0.855,
            wet_g: 0.16,
            honey_r: 0.16,
        };
        let mut rng = FastRandom::new(0);
        swipe_pass(
            &mut states,
            1,
            12,
            Axis::Vertical,
            0,
            1,
            0,
            8,
            11,
            4,
            1,
            &decay,
            &mut rng,
        );
        let mut c = 1.0f32;
        assert_eq!(states[2].r2, 1.0);
        for k in 3..8 {
            c *= 0.9;
            assert_eq!(states[k].r2, c, "white cell {k}");
        }
        assert_eq!(states[5].g2, 0.5, "G2 must not bleed");
        assert_eq!(states[5].b2, 0.2, "B2 must not bleed");
        // The 0.0185 cutoff (`:1229`): 0.9^37 ≈ 0.0203 > 0.0185 >= 0.9^38
        // ≈ 0.0183, so cell 37 is the last lit one on a forward-only sweep.
        let mut long = vec![State::default(); 60];
        long[0].r2 = 1.0;
        swipe_pass(
            &mut long,
            1,
            60,
            Axis::Vertical,
            0,
            1,
            0,
            50,
            50,
            50,
            1,
            &decay,
            &mut rng,
        );
        assert!(long[37].r2 > 0.0);
        assert_eq!(long[38].r2, 0.0, "the cutoff must stop the carry");
        assert_eq!(long[45].r2, 0.0);
    }

    #[test]
    fn white_wet_decay_uses_the_swipe_random_stream() {
        // `:1250-1251`: WithModifier((ulong)(i*1000+j)).Next(98,100)*0.01,
        // scaled by wet_r. The draw must match the FastRandom reference.
        let mut states = vec![State::default(); 12];
        states[2].r2 = 1.0;
        states[3].wet = true;
        let wet_r = 0.95f32 * 0.9 * 1.0;
        let decay = Decay {
            neg: 0.9,
            neg2: 0.54,
            neg3: 0.8,
            wet_r,
            wet_g: 0.16,
            honey_r: 0.16,
        };
        let mut rng = FastRandom::new(0);
        swipe_pass(
            &mut states,
            1,
            12,
            Axis::Vertical,
            0,
            1,
            0,
            8,
            11,
            4,
            1,
            &decay,
            &mut rng,
        );
        let mut reference = FastRandom::new(0).with_modifier_u64(3);
        let draw = reference.next_range(98, 100);
        assert!(draw == 98 || draw == 99);
        // The C# scales `(float)Next(98,100) * 0.01f` — not a /100 literal.
        let jitter = draw as f32 * 0.01;
        // Cell3 takes the carry 0.9 (raised), then the wet gate decays the
        // carry with the jitter; cell4 reads that carry.
        let expected = 0.9f32 * (wet_r * jitter);
        assert_eq!(states[4].r2, expected, "wet decay must ride the RNG draw");
    }

    #[test]
    fn retro_engine_golden_on_the_fixed_world() {
        let w = torch_world();
        let mut engine = LegacyLighting::new(LightMode::Retro);
        run_cycle(&mut engine, &w, &LegacyInputs::default());
        let seed = engine.get_color(48, 48);
        assert!(seed[0] > 0.5, "torch seed {seed:?}");
        assert_eq!(seed[0], seed[1], "Retro seeds the luminance mean");
        assert_eq!(seed[1], seed[2]);
        // One air step subtracts exactly 0.04 on each axis; the diagonal
        // takes two steps (vertical pass then horizontal).
        let s = seed[0];
        assert_eq!(engine.get_color(49, 48)[0], s - 0.04);
        assert_eq!(engine.get_color(48, 49)[0], s - 0.04);
        assert_eq!(engine.get_color(49, 49)[0], s - 0.04 - 0.04);
        assert_eq!(engine.get_color(47, 48)[0], s - 0.04);
        // Far darkness: 0.04 steps die after 25 tiles (seed < 1.0).
        assert_eq!(engine.get_color(48, 80)[0], 0.0);
        // Outside the expanded rect GetColor is zero (`:213-216`).
        assert_eq!(engine.get_color(0, 0), [0.0; 3]);
    }

    #[test]
    fn white_engine_golden_on_the_fixed_world() {
        let w = torch_world();
        let mut engine = LegacyLighting::new(LightMode::White);
        run_cycle(&mut engine, &w, &LegacyInputs::default());
        let seed = engine.get_color(48, 48);
        assert!(seed[0] > 0.5, "torch seed {seed:?}");
        let s = seed[0];
        // White multiplies by 0.9 per air cell (`:933`), no colour bleed.
        assert_eq!(engine.get_color(49, 48), [s * 0.9; 3]);
        assert_eq!(engine.get_color(48, 49), [s * 0.9; 3]);
        assert_eq!(engine.get_color(49, 49), [s * 0.9 * 0.9; 3]);
        // Every presented cell is grey.
        let (x0, y0, ww, hh) = engine.get_area();
        for y in y0..y0 + hh {
            for x in x0..x0 + ww {
                let c = engine.get_color(x, y);
                assert_eq!(c[0], c[1], "white must stay grey at {x},{y}");
                assert_eq!(c[1], c[2]);
            }
        }
    }

    #[test]
    fn trippy_engine_keeps_colour_and_subtracts_per_channel() {
        let w = torch_world();
        let mut engine = LegacyLighting::new(LightMode::Trippy);
        run_cycle(&mut engine, &w, &LegacyInputs::default());
        let seed = engine.get_color(48, 48);
        // Trippy is RGB (`:662`): the torch's coloured seed survives.
        assert!(seed[0] > seed[2], "trippy keeps the torch colour {seed:?}");
        let right = engine.get_color(49, 48);
        assert_eq!(right[0], seed[0] - 0.04);
        assert_eq!(right[1], seed[1] - 0.04);
        assert_eq!(right[2], seed[2] - 0.04);
    }

    #[test]
    fn trippy_moving_camera_publishes_the_full_rgb_width() {
        // CopyFullyProcessedDataOver's outer x bound is ASYMMETRIC: the RGB
        // branch (`:573`) starts from the fresh `maxLightArrayX`, the non-RGB
        // branch (`:598`) from `num4` (already `-= num`). A rightward camera
        // move (num in 1..16) must still publish the rightmost columns in
        // Trippy (the only reachable _rgb legacy mode); the flattened bound
        // left them stale. Drive a cycle, shift the camera right 3 tiles,
        // run another cycle, and assert the presented area is fully fresh
        // (no stale zero column at the right edge that GetColor expands into).
        let w = torch_world();
        let mut engine = LegacyLighting::new(LightMode::Trippy);
        run_cycle(&mut engine, &w, &LegacyInputs::default());
        let (pos, size) = camera();
        let shifted = [pos[0] + 3.0 * 16.0, pos[1]];
        let params = LightParams::default();
        let inputs = LegacyInputs::default();
        let mut prerendered = false;
        for _ in 0..32 {
            engine.process_area(&w, shifted, size, &params, &inputs, &[], None);
            if engine.render_count == 0 {
                prerendered = true;
            }
            if prerendered && engine.render_count == 2 {
                break;
            }
        }
        assert!(prerendered && engine.render_count == 2, "shifted cycle published");
        // The torch seed must still be presented after the shift (it sits
        // inside the expanded rect), and the column immediately right of the
        // seed must carry the subtracted fresh value, not a stale zero.
        let seed = engine.get_color(48, 48);
        assert!(seed[0] > seed[2], "trippy keeps colour after shift {seed:?}");
        let right = engine.get_color(49, 48);
        assert_eq!(right[0], seed[0] - 0.04, "right column must be fresh, not stale");
        assert_eq!(right[1], seed[1] - 0.04);
        assert_eq!(right[2], seed[2] - 0.04);
    }

    #[test]
    fn legacy_cycle_is_deterministic() {
        let w = torch_world();
        let mut a = LegacyLighting::new(LightMode::Retro);
        let mut b = LegacyLighting::new(LightMode::Retro);
        run_cycle(&mut a, &w, &LegacyInputs::default());
        run_cycle(&mut b, &w, &LegacyInputs::default());
        let (x0, y0, ww, hh) = a.get_area();
        for y in y0..y0 + hh {
            for x in x0..x0 + ww {
                assert_eq!(a.get_color(x, y), b.get_color(x, y), "at {x},{y}");
            }
        }
    }

    #[test]
    fn mode_switch_does_not_leak_state() {
        let w = torch_world();
        let mut engine = LegacyLighting::new(LightMode::White);
        run_cycle(&mut engine, &w, &LegacyInputs::default());
        let white = engine.get_color(49, 48);
        assert!(white[0] > 0.0);
        // NextLightMode (`Lighting.cs:165-173`): the setter resets the count
        // and Clear zeroes the presented values.
        engine.set_mode(LightMode::Retro);
        engine.clear();
        assert_eq!(engine.get_color(48, 48), [0.0; 3], "clear must zero");
        assert_eq!(engine.get_color(49, 48), [0.0; 3]);
        run_cycle(&mut engine, &w, &LegacyInputs::default());
        let retro = engine.get_color(49, 48);
        let seed = engine.get_color(48, 48);
        assert_eq!(retro[0], seed[0] - 0.04, "retro rules after the switch");
        assert_ne!(retro, white, "no white luminance may survive");
        // And back to White: the multiplicative rule returns.
        engine.set_mode(LightMode::White);
        engine.clear();
        run_cycle(&mut engine, &w, &LegacyInputs::default());
        let s = engine.get_color(48, 48)[0];
        assert_eq!(engine.get_color(49, 48), [s * 0.9; 3]);
    }

    #[test]
    fn presenting_legacy_then_modern_recompute_is_byte_identical() {
        // The modern-path untouched proof: a LightMap that served a legacy
        // present recomputes the EXACT modern result afterwards (the legacy
        // path owns its own RNGs and never touches the modern blur state).
        let w = torch_world();
        let params = LightParams::default();
        let fresh = LightMap::compute(&w, 40, 40, 56, 56, &params);
        let mut shared = LightMap::new();
        let mut engine = LegacyLighting::new(LightMode::Retro);
        run_cycle(&mut engine, &w, &LegacyInputs::default());
        engine.present_into(&mut shared);
        assert!(shared.get(48, 48)[0] > 0.0, "legacy present serves light");
        shared.compute_into(&w, 40, 40, 56, 56, &params);
        let (x0, y0, ww, hh) = (fresh.x0, fresh.y0, fresh.w, fresh.h);
        assert_eq!((shared.x0, shared.y0, shared.w, shared.h), (x0, y0, ww, hh));
        for y in y0..y0 + hh {
            for x in x0..x0 + ww {
                assert_eq!(
                    shared.get(x, y).map(f32::to_bits),
                    fresh.get(x, y).map(f32::to_bits),
                    "modern recompute diverged at {x},{y}"
                );
            }
        }
    }

    #[test]
    fn add_light_caps_and_collapses_for_white() {
        let mut engine = LegacyLighting::new(LightMode::White);
        let (pos, size) = camera();
        let params = LightParams::default();
        engine.process_area(
            &torch_world(),
            pos,
            size,
            &params,
            &LegacyInputs::default(),
            &[],
            None,
        );
        // White collapses to the mean and keeps the maximum (`:286-298`).
        // The expected means are computed with the same f32 expression the
        // engine uses (0.9f32 / 3 is not bit-equal to the 0.3f32 literal).
        engine.add_light(48, 48, [0.9, 0.0, 0.0]);
        engine.add_light(48, 48, [0.0, 0.6, 0.0]);
        assert_eq!(engine.temp_lights.len(), 1);
        let mean = (0.9f32 + 0.0 + 0.0) / 3.0;
        assert_eq!(engine.temp_lights[0].1, [mean; 3]);
        engine.add_light(48, 48, [0.9, 0.9, 0.9]);
        let mean2 = (0.9f32 + 0.9 + 0.9) / 3.0;
        assert_eq!(engine.temp_lights[0].1, [mean2; 3]);
        // The 2000 cap (`:263`).
        for k in 0..2500 {
            engine.add_light(40 + k % 50, 40 + k / 50, [0.1; 3]);
        }
        assert!(engine.temp_lights.len() <= MAX_TEMP_LIGHTS);
    }

    #[test]
    fn actuated_block_does_not_stop_legacy_light() {
        // GetTileMask's `!tile.inActive()` gate (`TileLightScanner.cs:68`) —
        // the legacy mask differs from the modern blur mask here.
        let mut w = torch_world();
        let i = w.index(49, 48);
        w.tiles[i].kind = 1;
        w.tiles[i].flags |= FLAG_ACTIVE | crate::tile::FLAG_INACTIVE;
        let mut engine = LegacyLighting::new(LightMode::Retro);
        run_cycle(&mut engine, &w, &LegacyInputs::default());
        let seed = engine.get_color(48, 48)[0];
        // The actuated dirt is air to the swipe: one 0.04 step, not 0.16.
        assert_eq!(engine.get_color(49, 48)[0], seed - 0.04);
        assert_eq!(
            legacy_mask(&w.tiles[i], false, false),
            Medium::Air,
            "actuated block must not form the Solid mask"
        );
    }

    #[test]
    fn lava_lights_and_water_decays_in_retro() {
        let mut w = torch_world();
        let i = w.index(60, 48);
        w.tiles[i].liquid = 255;
        w.tiles[i].flags |= FLAG_LIQUID_LAVA;
        let mut engine = LegacyLighting::new(LightMode::Retro);
        run_cycle(&mut engine, &w, &LegacyInputs::default());
        let lava = engine.get_color(60, 48);
        assert!(lava[0] > 0.3, "lava seed {lava:?}");
        // Water cells subtract wet_r = neg*1.2 (`:992`).
        let j = w.index(44, 48);
        w.tiles[j].liquid = 255;
        let mut engine2 = LegacyLighting::new(LightMode::Retro);
        run_cycle(&mut engine2, &w, &LegacyInputs::default());
        let seed = engine2.get_color(48, 48)[0];
        let wet = engine2.get_color(44, 48)[0];
        // Three air steps left to the water edge, then the wet gate.
        assert!(wet < seed - 3.0 * 0.04, "water must decay faster: {wet}");
    }

    // ---- S1-E-19 wave B: the minimap light export (tests-as-code, code-only
    // mode — written, not run; see ANALYSIS "The legacy minimap light export").

    /// Drive `frames` ProcessArea calls, attaching a fresh enabled export
    /// seam every frame (the `frame/draw.rs` driver shape). Returns the
    /// per-frame `update_map` results — `Some` exactly on the frames the
    /// `:349` gate fired.
    fn drive_with_export(
        engine: &mut LegacyLighting,
        w: &World,
        inputs: &LegacyInputs,
        tiles: &mut [crate::map::MapTile],
        map_time: &mut i32,
        map_time_max: i32,
        enabled: bool,
        frames: usize,
    ) -> Vec<Option<Rect>> {
        let (pos, size) = camera();
        let params = LightParams::default();
        let mut out = Vec::new();
        for _ in 0..frames {
            let mut e = MapLightExport::new(tiles, map_time, map_time_max, enabled);
            engine.process_area(w, pos, size, &params, inputs, &[], Some(&mut e));
            out.push(e.update_map);
        }
        out
    }

    #[test]
    fn map_export_keeps_the_rng_streams() {
        // The module-note witness: the export draws NO random numbers, so a
        // run with the seam attached every frame must present byte-identical
        // light AND leave both RNG seeds where the seam-less run left them.
        // Wet cells beside the torch force real `WithModifier`/`Next(98,100)`
        // swipe draws (`:1250-1251`), so the witness is not vacuous.
        let mut w = torch_world();
        for x in [46, 47] {
            let i = w.index(x, 48);
            w.tiles[i].liquid = 255; // water (no lava flag)
        }
        let (pos, size) = camera();
        let params = LightParams::default();
        let inputs = LegacyInputs::default();
        let mut a = LegacyLighting::new(LightMode::White);
        let mut b = LegacyLighting::new(LightMode::White);
        let mut tiles = vec![crate::map::MapTile::default(); (w.width * w.height) as usize];
        let mut map_time = 0i32;
        let mut exported = false;
        for _ in 0..20 {
            a.process_area(&w, pos, size, &params, &inputs, &[], None);
            let mut e = MapLightExport::new(&mut tiles, &mut map_time, 0, true);
            b.process_area(&w, pos, size, &params, &inputs, &[], Some(&mut e));
            exported |= e.update_map.is_some();
        }
        assert!(exported, "the seam must actually fire during the run");
        assert!(
            tiles.iter().any(|t| t.light > 0),
            "the export must reveal map tiles"
        );
        assert_eq!(a.swipe_rng.seed, b.swipe_rng.seed, "swipe stream unchanged");
        assert_eq!(
            a.scanner_rng.map(|r| r.seed),
            b.scanner_rng.map(|r| r.seed),
            "scanner stream unchanged"
        );
        let (x0, y0, ww, hh) = a.get_area();
        for y in y0..y0 + hh {
            for x in x0..x0 + ww {
                assert_eq!(
                    a.get_color(x, y).map(f32::to_bits),
                    b.get_color(x, y).map(f32::to_bits),
                    "presented light diverged at {x},{y}"
                );
            }
        }
    }

    #[test]
    fn map_export_gates_on_delay_count_and_render_count() {
        // `ProcessArea` `:334-358`: a fresh engine (count 99) pre-renders on
        // call 1 (count 0); the shift branch (`else if`, any count != 2 with
        // a camera anchor) runs from call 2. `mapDelay` starts 2
        // (`Main.cs:66504`) and counts down on calls 2/4; the export gate
        // additionally needs `renderCount == 3`, so the first export fires
        // on call 9 (the first count-3 frame after the delay expired). With
        // `mapTimeMax = 0` the re-arm keeps the gate open, so call 14 (count
        // 3 again, 5-frame cycle) exports a second time.
        let w = torch_world();
        let mut engine = LegacyLighting::new(LightMode::White);
        let mut tiles = vec![crate::map::MapTile::default(); (w.width * w.height) as usize];
        let mut map_time = 0i32;
        let out = drive_with_export(
            &mut engine,
            &w,
            &LegacyInputs::default(),
            &mut tiles,
            &mut map_time,
            0,
            true,
            14,
        );
        let fired: Vec<usize> = out
            .iter()
            .enumerate()
            .filter(|(_, r)| r.is_some())
            .map(|(i, _)| i)
            .collect();
        assert_eq!(fired, vec![8, 13], "exports only on the count-3 shift frames");
        assert_eq!(map_time, 0, "the re-arm reads mapTimeMax = 0 (`:373`)");
        // The window is the requested rect clamped into the 40 inset
        // (`:383-394`): the 96-wide world folds [37..60] onto [40, 56).
        assert_eq!(out[8], Some([40, 40, 16, 16]));
        assert!(tiles.iter().any(|t| t.light > 0), "the torch reveals");
    }

    #[test]
    fn map_export_disabled_gate_writes_nothing() {
        // `Main.mapEnabled == false` (`:349`): the countdown still runs and
        // the re-arm never happens — no tiles, no `updateMap`.
        let w = torch_world();
        let mut engine = LegacyLighting::new(LightMode::White);
        let mut tiles = vec![crate::map::MapTile::default(); (w.width * w.height) as usize];
        let mut map_time = 0i32;
        let out = drive_with_export(
            &mut engine,
            &w,
            &LegacyInputs::default(),
            &mut tiles,
            &mut map_time,
            3,
            false,
            14,
        );
        assert!(out.iter().all(|r| r.is_none()), "no export while disabled");
        assert_eq!(map_time, 0, "no re-arm while disabled");
        assert!(tiles.iter().all(|t| t.light == 0), "no tile writes");
    }

    #[test]
    fn map_export_net_connecting_pins_map_time_to_one() {
        // `:341-344`: `Netplay.Connection.StatusMax > 0` pokes `mapTime = 1`
        // on EVERY shift frame, before the gate — so the count-3 frame sees
        // 1, not 0, and never exports while connecting.
        let w = torch_world();
        let mut engine = LegacyLighting::new(LightMode::White);
        let mut tiles = vec![crate::map::MapTile::default(); (w.width * w.height) as usize];
        let mut map_time = 0i32;
        let inputs = LegacyInputs {
            net_connecting: true,
            ..Default::default()
        };
        let out = drive_with_export(
            &mut engine, &w, &inputs, &mut tiles, &mut map_time, 0, true, 14,
        );
        assert!(out.iter().all(|r| r.is_none()), "no export while connecting");
        assert_eq!(map_time, 1, "the poke lands on the last shift frame");
    }

    /// A 200x150 air world, surface at y=40, camera centred on tile (100,60)
    /// in a 320x240 backbuffer: the export window folds to [90,111)x[52,68)
    /// (all below the surface, so the sky-air arm never fires).
    fn export_world() -> (World, [f32; 2], [f32; 2]) {
        let mut w = World::new("export".into(), 200, 150);
        w.world_surface = 40.0;
        (w, [1440.0, 840.0], [320.0, 240.0])
    }

    /// One ProcessArea call to set the requested rect / margins / array, then
    /// hand-seed the PRESENTED cells the export reads (`:403-415`).
    fn seed_presented(engine: &mut LegacyLighting, w: &World, pos: [f32; 2], size: [f32; 2]) {
        engine.process_area(
            w,
            pos,
            size,
            &LightParams::default(),
            &LegacyInputs::default(),
            &[],
            None,
        );
        // (100,60): max channel 0.5. (98,60): byte 19 after White's 1.5x.
        // (97,60): byte 18 — the floor's `b > 18` boundary.
        for (x, r) in [(100, 0.5f32), (98, 0.05), (97, 0.048)] {
            let lx = x - engine.requested[0] + engine.off_screen_tiles;
            let ly = 60 - engine.requested[2] + engine.off_screen_tiles;
            let s = &mut engine.states[(lx * engine.sh + ly) as usize];
            s.r = r;
            s.g = r * 0.4;
            s.b = r * 0.2;
        }
    }

    #[test]
    fn map_export_white_brightens_and_floors_at_22() {
        // `TryUpdatingMapWithLight` `:403-431` directly: White's 1.5x
        // (`:416-419`), the byte truncation (`:420`), the revealed-floor 22
        // (`:425-428`), the re-arm FIRST (`:373`) and the `updateMap` rect
        // (`:432`). Below-surface air keeps the exported byte (the map's
        // background arm only forces 255 above the surface).
        let (w, pos, size) = export_world();
        let mut engine = LegacyLighting::new(LightMode::White);
        seed_presented(&mut engine, &w, pos, size);
        let mut tiles = vec![crate::map::MapTile::default(); (w.width * w.height) as usize];
        // (96,60) is revealed at light 10 but dark now → the floor lifts it.
        assert!(crate::map::update_map_lighting(&w, &mut tiles, 96, 60, 10));
        let mut map_time = 0i32;
        let mut e = MapLightExport::new(&mut tiles, &mut map_time, 7, true);
        engine.try_updating_map_with_light(&w, pos, size, &LegacyInputs::default(), &mut e);
        assert_eq!(map_time, 7, "the re-arm happens first (`:373`)");
        assert_eq!(e.update_map, Some([90, 52, 21, 16]), "`:432`");
        // 0.5 * 1.5 * 255 = 191.25 → 191.
        assert_eq!(tiles[w.index(100, 60)].light, 191);
        // 0.05 * 1.5 * 255 = 19.125 → 19 → the `b > 18` floor lifts to 22.
        assert_eq!(tiles[w.index(98, 60)].light, 22);
        // 0.048 * 1.5 * 255 = 18.36 → 18 — NOT > 18, unrevealed → stays 18.
        assert_eq!(tiles[w.index(97, 60)].light, 18);
        // Dark + revealed (old light 10 > 0) → floor 22, max(10, 22) = 22.
        assert_eq!(tiles[w.index(96, 60)].light, 22);
        assert!(e.changed.contains(&(96, 60)), "the floor lift is a change");
        // Dark + unrevealed → `UpdateLighting(0)` refuses (`WorldMap.cs:52-55`).
        assert_eq!(tiles[w.index(95, 60)].light, 0);
        assert!(!e.changed.contains(&(95, 60)));
        // The changed list is x-outer, y-inner (`:396-399`).
        let first = e.changed.iter().position(|&(x, _)| x == 96).unwrap();
        let last = e.changed.iter().rposition(|&(x, _)| x == 96).unwrap();
        assert!(e.changed[first..=last].iter().all(|&(x, _)| x == 96));
    }

    #[test]
    fn map_export_retro_keeps_the_raw_byte() {
        // Retro (`Mode >= 2`) skips the 1.5x brighten (`IsColorOrWhiteMode`,
        // `:204`): 0.5 * 255 = 127.5 → 127, and 0.05 * 255 = 12.75 → 12 stays
        // below the floor's `b > 18` gate.
        let (w, pos, size) = export_world();
        let mut engine = LegacyLighting::new(LightMode::Retro);
        seed_presented(&mut engine, &w, pos, size);
        let mut tiles = vec![crate::map::MapTile::default(); (w.width * w.height) as usize];
        let mut map_time = 0i32;
        let mut e = MapLightExport::new(&mut tiles, &mut map_time, 0, true);
        engine.try_updating_map_with_light(&w, pos, size, &LegacyInputs::default(), &mut e);
        assert_eq!(tiles[w.index(100, 60)].light, 127);
        assert_eq!(tiles[w.index(98, 60)].light, 12);
    }

    #[test]
    fn map_export_sky_air_keeps_the_22_ambient() {
        // `:421-424`: above the surface, inactive + wall 0 + liquid 0 forces
        // b = 22 regardless of the presented light. A wet cell fails the
        // `liquid == 0` conjunct and keeps its own byte — that pair proves
        // the arm, not just the revealed-floor 22. The map's sky-background
        // arm (`MapHelper.GetBackgroundType`) publishes sky cells at 255.
        let (mut w, _, _) = export_world();
        w.world_surface = 100.0; // the whole window is now above the surface
        // (95,60): water — the sky-air arm must NOT fire here.
        let wet = w.index(95, 60);
        w.tiles[wet].liquid = 255;
        let pos = [1440.0, 840.0];
        let size = [320.0, 240.0];
        let mut engine = LegacyLighting::new(LightMode::White);
        seed_presented(&mut engine, &w, pos, size);
        let mut tiles = vec![crate::map::MapTile::default(); (w.width * w.height) as usize];
        let mut map_time = 0i32;
        let mut e = MapLightExport::new(&mut tiles, &mut map_time, 0, true);
        engine.try_updating_map_with_light(&w, pos, size, &LegacyInputs::default(), &mut e);
        // Dark dry sky air reveals (the arm's 22 > 0); the wet dark cell
        // keeps b = 0 and stays unrevealed (`WorldMap.cs:52-55`).
        assert!(tiles[w.index(94, 60)].light > 0, "sky air reveals at 22+");
        assert!(e.changed.contains(&(94, 60)));
        assert_eq!(tiles[wet].light, 0, "the wet cell skips the sky-air arm");
        assert!(!e.changed.contains(&(95, 60)));
        assert_eq!(e.update_map, Some([90, 52, 21, 16]));
    }
}
