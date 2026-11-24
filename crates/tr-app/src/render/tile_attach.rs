//! W4-g31 attach phase — the `TR_TILEPASS` SHADOW attach of the tile-pass
//! union (docs/UNION-CONTRACT-tilepass.md §8). The g29 `draw_layer` /
//! `post_draw_tiles` drivers run at the C# `TileDrawing.Draw` /
//! `PostDrawTiles` positions over the SAME camera rect the fused
//! `tiles.rs::draw_tiles` loop draws, with the real sibling binds:
//!
//! * head: g32 `LegacyCaches` per-step clears + g30
//!   `clear_special_block_counts` (the `SpecialBlockCounts` flag arm),
//!   g28 `tr_render::quality_light::quality_light_tiers`, the g32
//!   wind-grid `set_size` (a SHADOW-OWNED grid — the shared
//!   `Wind::wind_grid` is borrowed immutably by the fused pass and its
//!   S1-G-8 update-site sizing stays untouched), the martian `[b,b,b,0]`
//!   quad;
//! * classify: g30 `classify_tile` + `apply_cell_class` into a
//!   shadow-owned `SpecialPoints` registry (the vine crawls are pure
//!   world reads; `LiquidDrops` cells recorded for d60 AND planned in
//!   `drip_plan` row-major — the §8.5 direct-bind call order for g68's
//!   `emit_liquid_drops`, research/136 C5); the 717 glowmask
//!   table-pass colour is PLANNED per drawn cell (`lava_glows`,
//!   research/136 §3 — mask 362, the `Color(v,v,v,v/2)` truncation of
//!   `tile_ctor::lava_light_a`);
//! * post: g29 `post_draw_tiles` → g31 `drain_pass_for_post_step` →
//!   `drain_one_pass` over `&reg.buckets`, ONE shadow-owned
//!   `UnifiedRandom` shared across the ten nature passes (the C# `_rand`
//!   convention) — the stream is seeded from the frame's sparkle seed
//!   and is SEPARATE from the fused pass's `leaf_rng`, so no existing
//!   roll moves.
//!
//! PLAN-ONLY (default OFF, `crate::env::tilepass_shadow`): the shadow
//! pushes NO sprites and mutates NO shared state — the per-cell
//! `DrawSingleTile` body, the dark-cell merge (d56's gate), the cracks
//! (drawn per-frame by `draw_mining_cracks`), the legacy/TE passes (d57)
//! and the ported drain bodies (foliage/vines/pylons — they need the
//! renderer + `SpecialEnv`) are COUNTED, not executed. The pixel-binding
//! merge (the fused loop split into the two `draw_layer` calls with the
//! full `TilePassSeams` impl) is §8's script; this attach proves the
//! orchestration + registry + drain plans run over the live world every
//! rebuild without touching a pixel or a stream.
//!
//! A/B: unset `TR_TILEPASS` = zero union code in the frame path (the
//! gate short-circuits before any allocation). Set = identical pixels
//! (no pushes, separate RNG, G6 key untouched), one `TILEPASS shadow:`
//! log line per world-range rebuild with the per-pass plan counts.
//! The shadow rides the `!reuse_world` arm — a G6 cache hit skips it
//! (use `TR_NOCACHE=1` or a moving camera for a per-frame A/B).
//!
//! Recorded deviations (docs/ANALYSIS.md "The TR_TILEPASS shadow attach"):
//! * `martianLight`/`masterColor` have no port globals — fed 0.0 (the
//!   `resources.rs:499` convention); `devLightTilesCheat` false.
//! * `cache_tree_point` (`:992`) needs the `GetTileDrawData`-corrected
//!   frames the fused loop owns — the shadow's Tree bucket stays empty
//!   (the Trees drain walks 0 points); the pixel merge feeds it.
//! * The black helper stays empty: the dark-cell FEED decision is d56's
//!   `tile_draw_gate` (not on this branch); the fused pass keeps its
//!   greedy merge until the family merge (§2.2: gate = d56, coalescer
//!   = g29).
//! * `allow_emit_effects` fed true (the `FocusHelper` stepper flag is
//!   not threaded into `draw_tiles`); the shadow emits nothing anyway.
//! * The section-framing flags feed false (the framing family's seam).

use crate::render::special_points::{
    self, SpecialPoints,
};
use crate::render::tile_drains::{
    self, DrainFairyJar, DrainFlame, DrainJarItem, DrainSprite, PlainLight,
    PortedBodyDrain, RisingVineSegment,
};
use crate::render::tile_draw_pass::LegacyCaches;
use crate::render::tile_pass::{
    self, BlackStrip, CrackSite, DrawBlackHelper, DrawLayerInputs, LayerCacheClear,
    LegacyCacheClear, PlacementPreviewGate, PostDrawStep, TilePassSeams,
};
use tr_core::lighting::LightMap;
use tr_core::wind::Wind;
use tr_core::{UnifiedRandom, World};

/// The frame inputs the shadow reads from `draw_tiles`' locals — the
/// camera trio becomes `Main.Camera.ScaledPosition/ScaledSize` (the
/// fused pass's `min`/`max-min`), the counters are the same values the
/// inline grass/vine/sunflower draws consume.
pub(crate) struct ShadowInputs {
    pub camera: glam::Vec2,
    pub zoom: f32,
    pub screen: glam::Vec2,
    /// `Main.gfxQuality`. The tiles.rs site feeds the fused pass's
    /// pinned 0.5 default (its HIGH_LIGHT/MEDIUM_LIGHT convention);
    /// the adaptive `App.gfx_quality.quality` threads in at the pixel
    /// merge (§8 — a `draw_tiles` signature change is contested by d58).
    pub gfx_quality: f32,
    pub grass_counter: f32,
    pub sunflower_counter: f32,
    pub vine_counter: f32,
    pub anim_time: f32,
    /// The frame's sparkle seed — the shadow `_rand` derives from it
    /// (a SEPARATE stream; `leaf_rng` is never passed here).
    pub sparkle_seed: u64,
    pub disco: [u8; 3],
}

/// The shadow's seam impl: real binds for the pure halves, counters for
/// the pixel halves. Holds the registry + caches + drain plans so the
/// witnesses (and the log line) can inspect them after the four calls.
pub(crate) struct ShadowSeams<'a> {
    world: &'a World,
    wind: &'a Wind,
    light: &'a LightMap,
    inputs: &'a ShadowInputs,
    pub reg: SpecialPoints,
    pub caches: LegacyCaches,
    /// The shadow-owned grid (never the shared `Wind::wind_grid`).
    wind_grid: tr_core::wind::WindGrid,
    pub tiers: Option<tr_render::quality_light::QualityLightTiers>,
    pub martian_glow: [u8; 4],
    /// The ONE `_rand` the ten nature passes share (C# `:104`/`:191`).
    rng: UnifiedRandom,
    light_seam: PlainLight,
    ported: CountingPorted,
    pub sprites: Vec<DrainSprite>,
    pub fairy_jars: Vec<DrainFairyJar>,
    pub rising: Vec<RisingVineSegment>,
    pub jar_items: Vec<DrainJarItem>,
    pub flames: Vec<DrainFlame>,
    /// One entry per sequenced C# call the pixel merge must bind.
    pub log: Vec<String>,
    pub strips: Vec<BlackStrip>,
    /// The 717 glowmask table-pass PLAN (research/136 §3, the :1178-1342
    /// pass f15 explicitly scopes OUT): one entry per drawn 717 cell —
    /// `(x, y, color_bytes)` where the colour is the XNA
    /// `Color(v, v, v, v/2)` truncation of `tile_ctor::lava_light_a`.
    /// PLAN-ONLY like every other shadow output (no pushes).
    pub lava_glows: Vec<(i32, i32, [u8; 4])>,
    /// The LiquidDrops-arm plan (research/136 C5 / contract §8.5): one
    /// entry per classified drip cell `(x, y, kind)` in row-major order —
    /// the exact call order the pixel bind feeds g68's
    /// `foliage::emit_liquid_drops(world, y, x, kind, water_style,
    /// zone_corrupt, zone_crimson, zone_hallow, &mut rand, &mut main_rand,
    /// &mut gore)` DIRECTLY per cell (the body is UNWIRED on this branch;
    /// the C# emits inside the Draw switch :688-694 BEFORE the `continue`,
    /// so draining the recorded cells later would re-order the ONE
    /// `_rand` — the plan preserves the row-major gate order).
    pub drip_plan: Vec<(i32, i32, u16)>,
}

/// The `PortedBodyDrain` shadow: the five ported bodies need the
/// renderer + `SpecialEnv` (the `pylons.rs` convention) — count the
/// per-point calls the driver dispatches, draw nothing.
#[derive(Default)]
struct CountingPorted {
    trees: u32,
    vines: u32,
    void_lenses: u32,
    pylons: u32,
    trophies: u32,
}

impl PortedBodyDrain for CountingPorted {
    fn tree_point(&mut self, _world: &World, _x: i32, _y: i32) {
        self.trees += 1;
    }
    fn vine_root(&mut self, _world: &World, _x: i32, _y: i32) {
        self.vines += 1;
    }
    fn void_lens_point(&mut self, _world: &World, _x: i32, _y: i32) {
        self.void_lenses += 1;
    }
    fn pylon_point(&mut self, _world: &World, _x: i32, _y: i32) {
        self.pylons += 1;
    }
    fn trophy_point(&mut self, _world: &World, _x: i32, _y: i32) {
        self.trophies += 1;
    }
}

impl<'a> ShadowSeams<'a> {
    fn new(world: &'a World, wind: &'a Wind, light: &'a LightMap, inputs: &'a ShadowInputs) -> Self {
        Self {
            world,
            wind,
            light,
            inputs,
            reg: SpecialPoints::new(),
            caches: LegacyCaches::default(),
            wind_grid: tr_core::wind::WindGrid::new(),
            tiers: None,
            martian_glow: [0; 4],
            // The C# `_rand` is clock-seeded (`new UnifiedRandom()`);
            // the shadow pins it per frame like the sparkle draws so
            // two runs of the same frame produce identical plans.
            rng: UnifiedRandom::new((inputs.sparkle_seed & 0x7FFF_FFFF) as i32),
            light_seam: PlainLight,
            ported: CountingPorted::default(),
            sprites: Vec::new(),
            fairy_jars: Vec::new(),
            rising: Vec::new(),
            jar_items: Vec::new(),
            flames: Vec::new(),
            log: Vec::new(),
            strips: Vec::new(),
            lava_glows: Vec::new(),
            drip_plan: Vec::new(),
        }
    }

    /// The `DrawLayerInputs` for one layer — the camera trio maps to
    /// the fused pass's own `min`/`max` (world pixels, zoom applied).
    fn layer_inputs(&self, solid_layer: bool) -> DrawLayerInputs {
        let half = self.inputs.screen * 0.5 / self.inputs.zoom;
        let min = self.inputs.camera - half;
        DrawLayerInputs {
            solid_layer,
            into_render_targets: false,
            gfx_quality: self.inputs.gfx_quality,
            dev_light_tiles_cheat: false,
            martian_light: 0.0,
            draw_to_screen: true,
            off_screen_range: 192,
            scaled_position: [min.x, min.y],
            scaled_size: [half.x * 2.0, half.y * 2.0],
            screen_position: [min.x, min.y],
            allow_emit_effects: true,
            sway_enabled: self.wind.sway_enabled,
            any_unfinished_sections: false,
            any_need_refresh: false,
            preview_gate: PlacementPreviewGate::default(),
            preview_is_kite: false,
            preview_hold_style: 0,
            preview_frames_in_movement: 0,
        }
    }
}

impl TilePassSeams for ShadowSeams<'_> {
    fn ensure_wind_grid_size(&mut self, width: i32, height: i32) {
        // The g32 `ensure_wind_grid_size` body (`grid.set_size` of the
        // UNPADDED extents) onto the SHADOW grid — the shared grid's
        // S1-G-8 update-site sizing is untouched.
        self.wind_grid.set_size(width, height);
        self.log.push(format!("wind {width}x{height}"));
    }
    fn clear_legacy_cache(&mut self, step: LegacyCacheClear) {
        self.caches.clear_legacy_step(step);
        self.log.push(format!("legacy {step:?}"));
    }
    fn clear_layer_cache(&mut self, step: LayerCacheClear) {
        if self.caches.clear_layer_step(step) {
            // The `SpecialBlockCounts` flag arm chains the g30 clear.
            self.reg.clear_special_block_counts();
        }
        self.log.push(format!("layer {step:?}"));
    }
    fn rebuild_quality_light_tiers(&mut self, gfx_quality: f32, dev_cheat: bool) {
        self.tiers = Some(tr_render::quality_light::quality_light_tiers(gfx_quality, dev_cheat));
        self.log.push(format!("tiers {gfx_quality}"));
    }
    fn reset_critter_cage(&mut self) {
        // No port global (`Main.critterCage` — the special_tiles.rs:375
        // proximity flag is the consumer, unported); count the reset.
        self.log.push("critterCage".into());
    }
    fn set_martian_glow(&mut self, b: u8) {
        self.martian_glow = [b, b, b, 0];
        self.log.push(format!("martian {b}"));
    }
    fn reset_paint_lookup_key(&mut self) {
        self.log.push("paintKey".into());
    }
    fn draw_tile_liquid_behind(&mut self, x: i32, y: i32) {
        // The fused pass draws the real behind-tile liquid per cell
        // (`tiles.rs`'s `draw_tile_liquid_behind` arm); count only.
        self.log.push(format!("liquid {x},{y}"));
    }
    fn classify_and_draw_cell(&mut self, x: i32, y: i32, _black: &mut DrawBlackHelper) {
        // The g30 per-cell half (contract §1.2): classify + registry
        // side effects. `DrawSingleTile` itself is the fused loop's
        // body — the shadow counts the cells that would reach it.
        let Some(tile) = self.world.tile(x, y) else {
            return;
        };
        let class = special_points::classify_tile(
            tile.kind,
            tile.frame_x,
            tile.frame_y,
            self.wind.sway_enabled,
        );
        if special_points::apply_cell_class(self.world, x, y, tile.kind, class, &mut self.reg) {
            self.reg.single_tiles += 1;
            // The 717 glowmask table-pass bind (research/136 §3): the pass
            // runs at CAD step 14 (AFTER DrawBasicTile's flames tail :1723,
            // BEFORE f15's :1343 record bind). PLAN-ONLY: the colour is the
            // exact XNA `Color(v, v, v, v/2)` truncation of `lava_light_a`
            // (mask 362, Main.cs:10056); the pixel push lands with the
            // table-pass lane / family merge. `time_for_visual_effects` =
            // `(anim_time * 60) % 216000` (the frame/draw.rs:374 convention).
            // Streams: NONE (the noise table is the seed-0 OnceLock).
            if tile.kind == 717 {
                let v = crate::render::tile_ctor::lava_light_a(
                    self.world,
                    x,
                    y,
                    (self.inputs.anim_time * 60.0) % 216000.0,
                );
                let b = (v * 255.0) as u8;
                let a = ((v / 2.0) * 255.0) as u8;
                self.lava_glows.push((x, y, [b, b, b, a]));
            }
        }
        // The LiquidDrops arm (research/136 C5, contract §8.5): the C#
        // emits INSIDE the Draw switch (:688-694, kinds 373/374/375/461/
        // 709) BEFORE the `continue`, so the `Next(num*2)` gate draws on
        // the ONE `_rand` in row-major cell order. g68's
        // `foliage::emit_liquid_drops` is UNWIRED on this branch — the
        // shadow plans the direct-bind call order (`drip_plan`); the
        // recorded `liquid_drop_cells` stay the census input. Draining
        // the recorded cells later would re-order the stream (C5).
        if class == special_points::CellClass::LiquidDrops {
            self.drip_plan.push((x, y, tile.kind));
        }
    }
    fn draw_black_strip(&mut self, strip: BlackStrip) {
        self.strips.push(strip);
        self.log.push("blackStrip".into());
    }
    fn restart_layered_batch(&mut self) {
        self.log.push("restartLayered".into());
    }
    fn draw_tile_cracks(&mut self, site: CrackSite) {
        // `draw_mining_cracks` owns the per-frame crack overlay (the
        // S1-G-1 wave C stand-in for this exact site); count only.
        self.log.push(format!("cracks {site:?}"));
    }
    fn restart_sprite_batch(&mut self) {
        self.log.push("restartSprite".into());
    }
    fn draw_special_tiles_legacy(&mut self) {
        // d57's furniture slice + special_tiles.rs bind at the merge.
        self.log.push("legacyDraw".into());
    }
    fn draw_object_preview(&mut self, opacity: f32) {
        self.log.push(format!("preview {opacity}"));
    }
    fn post_draw_pass(&mut self, step: PostDrawStep) {
        self.log.push(format!("post {step:?}"));
        let Some(pass) = tile_drains::drain_pass_for_post_step(step) else {
            // EndBatch / NatureAfterAllObjects / the two TE passes —
            // the batch/nature seam + d57 (the g32 None-arm fold).
            return;
        };
        tile_drains::drain_one_pass(
            pass,
            self.world,
            self.wind,
            self.light,
            self.inputs.grass_counter,
            self.inputs.sunflower_counter,
            self.inputs.vine_counter,
            &mut self.rng,
            &mut self.light_seam,
            &mut self.ported,
            crate::env::show_invisible(),
            self.inputs.disco,
            // `Main.masterColor` — no port global (resources.rs:499).
            0.0,
            self.inputs.sparkle_seed,
            self.inputs.anim_time % 3600.0,
            &self.reg.buckets,
            &mut self.sprites,
            &mut self.fairy_jars,
            &mut self.rising,
            &mut self.jar_items,
            &mut self.flames,
        );
    }
}

/// Run the four C# calls in frame order — `Draw(false)` →
/// `PostDrawTiles(false)` → `Draw(true)` → `PostDrawTiles(true)`
/// (`Main.cs:63280`/`:63295` + `:21758-21761`) — over the live world.
/// Returns the seams so the caller can log/inspect the plans.
pub(crate) fn run_shadow<'a>(
    world: &'a World,
    wind: &'a Wind,
    light: &'a LightMap,
    inputs: &'a ShadowInputs,
) -> ShadowSeams<'a> {
    let mut seams = ShadowSeams::new(world, wind, light, inputs);
    let non_solid = seams.layer_inputs(false);
    tile_pass::draw_layer(world, &non_solid, special_points::is_tile_draw_layer_solid, &mut seams);
    tile_pass::post_draw_tiles(false, &mut seams);
    let solid = seams.layer_inputs(true);
    tile_pass::draw_layer(world, &solid, special_points::is_tile_draw_layer_solid, &mut seams);
    tile_pass::post_draw_tiles(true, &mut seams);
    seams
}

impl ShadowSeams<'_> {
    /// The one-line A/B summary (the `TILEPASS shadow:` log payload).
    pub(crate) fn summary(&self) -> String {
        let buckets: usize = self.reg.buckets.iter().map(Vec::len).sum();
        format!(
            "cells={} buckets={} sprites={} jars={} rising={} jar_items={} flames={} \
             ported(t/v/l/p/tr)={}/{}/{}/{}/{} strips={} steps={} legacy={} martian={} lava={} drips={} tiers={:?}",
            self.reg.single_tiles,
            buckets,
            self.sprites.len(),
            self.fairy_jars.len(),
            self.rising.len(),
            self.jar_items.len(),
            self.flames.len(),
            self.ported.trees,
            self.ported.vines,
            self.ported.void_lenses,
            self.ported.pylons,
            self.ported.trophies,
            self.strips.len(),
            self.log.len(),
            self.caches.legacy_points.len(),
            self.martian_glow[0],
            self.lava_glows.len(),
            self.drip_plan.len(),
            self.tiers.map(|t| (t.high, t.medium)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::FLAG_ACTIVE;

    fn world(w: i32, h: i32) -> World {
        let mut world = World::new("t".into(), w, h);
        world.compute_column_surface();
        world.compute_chunks();
        world
    }

    fn put(w: &mut World, x: i32, y: i32, kind: u16, fx: i16, fy: i16) {
        let i = w.index(x, y);
        w.tiles[i].kind = kind;
        w.tiles[i].flags = FLAG_ACTIVE;
        w.tiles[i].frame_x = fx;
        w.tiles[i].frame_y = fy;
    }

    fn inputs() -> ShadowInputs {
        ShadowInputs {
            camera: glam::Vec2::new(480.0, 480.0),
            zoom: 1.0,
            screen: glam::Vec2::new(320.0, 320.0),
            gfx_quality: 0.5,
            grass_counter: 0.0,
            sunflower_counter: 0.0,
            vine_counter: 0.0,
            anim_time: 10.0,
            sparkle_seed: 42,
            disco: [255, 255, 255],
        }
    }

    /// The four-call frame order: the log carries the non-solid head
    /// (critterCage + SpecialBlockCounts), the 12-step non-solid post
    /// list, then the solid head (NO critterCage, the clothing pair)
    /// and the 2-step solid post list — the g29 driver order over the
    /// real binds.
    #[test]
    fn w4_g31_attach_shadow_runs_the_four_call_frame_order() {
        let w = world(128, 128);
        let wind = Wind::new(7);
        let light = LightMap::default();
        let inp = inputs();
        let s = run_shadow(&w, &wind, &light, &inp);
        let log = s.log.join("|");
        // Two heads, two tails, in order.
        let first_critter = log.find("critterCage").expect("non-solid head");
        let solid_clear = log.find("layer DisplayDollTileEntityPositions").expect("solid head");
        assert!(first_critter < solid_clear);
        assert_eq!(log.matches("critterCage").count(), 1, "non-solid only (:472-475)");
        assert_eq!(log.matches("legacy ChestPositions").count(), 2, "every Draw (:477)");
        assert_eq!(log.matches("layer SpecialBlockCounts").count(), 1);
        assert_eq!(log.matches("post DrawMultiTileVines").count(), 1);
        assert_eq!(log.matches("post EndBatch").count(), 1);
        assert_eq!(log.matches("post NatureAfterAllObjects").count(), 1);
        assert_eq!(log.matches("post DrawEntitiesHatRacks").count(), 1);
        assert_eq!(log.matches("post DrawEntitiesDisplayDolls").count(), 1);
        // The hat racks precede the dolls (the solid post order).
        assert!(log.find("post DrawEntitiesHatRacks") < log.find("post DrawEntitiesDisplayDolls"));
        // The real g28 tier bind ran with the fed quality.
        let t = s.tiers.expect("the tier rebuild");
        assert_eq!(t, tr_render::quality_light::quality_light_tiers(0.5, false));
        // martianLight 0 ⇒ b = 100, the [b,b,b,0] quad.
        assert_eq!(s.martian_glow, [100, 100, 100, 0]);
        // The black helper stays empty until d56's gate feeds it.
        assert!(s.strips.is_empty());
    }

    /// The registry + drain binds: a vine cell crawled into bucket 4
    /// dispatches through the Vines post step to the PortedBodyDrain
    /// seam; a kind-3 grass cell (a `SwaysInWindBasic` member, the
    /// default classify arm) lands in the WindyGrass bucket and the
    /// Grass pass walks it.
    #[test]
    fn w4_g31_attach_shadow_classifies_and_drains_over_the_registry() {
        let mut w = world(128, 128);
        // A vine thread (52) hanging from an anchor: the crawl walks
        // down to the root; the root registers in bucket 4 (Vine).
        put(&mut w, 30, 30, 52, 0, 0);
        put(&mut w, 30, 31, 52, 0, 18);
        // A sway-grass cell (kind 3, frame 0 — the DrawGrass family).
        put(&mut w, 40, 40, 3, 0, 0);
        let wind = Wind::new(7);
        let light = LightMap::default();
        let inp = inputs();
        let s = run_shadow(&w, &wind, &light, &inp);
        // The vine root was crawled + registered (the g30 VineCrawl arm).
        assert!(!s.reg.buckets[tile_drains::BUCKET_VINE].is_empty(), "vine root registered");
        // The Vines post step dispatched it to the ported seam.
        assert!(s.ported.vines >= 1, "the Vines drain walked the bucket");
        // The grass cell classified into a drain bucket and the Grass
        // pass ran over it (plan or gated skip — the bucket is fed).
        let grass_family = s.reg.buckets[tile_drains::BUCKET_WINDY_GRASS].len()
            + s.reg.buckets[tile_drains::BUCKET_ANY_DIRECTIONAL_GRASS].len();
        assert!(grass_family >= 1, "the grass cell registered");
        // The summary line renders (the A/B log payload).
        assert!(s.summary().contains("steps="));
    }

    /// Determinism: two runs of the same frame produce identical plans
    /// (the shadow `_rand` is pinned from the sparkle seed, NOT the
    /// clock — and it is a SEPARATE stream from the fused pass's
    /// `leaf_rng`, which is never passed in).
    #[test]
    fn w4_g31_attach_shadow_plans_are_frame_deterministic() {
        let mut w = world(128, 128);
        put(&mut w, 30, 30, 52, 0, 0);
        put(&mut w, 40, 40, 3, 0, 0);
        put(&mut w, 41, 40, 3, 18, 0);
        let wind = Wind::new(7);
        let light = LightMap::default();
        let inp = inputs();
        let a = run_shadow(&w, &wind, &light, &inp);
        let b = run_shadow(&w, &wind, &light, &inp);
        assert_eq!(a.log, b.log);
        assert_eq!(a.sprites.len(), b.sprites.len());
        for (x, y) in a.sprites.iter().zip(&b.sprites) {
            assert_eq!(x.pos, y.pos);
            assert_eq!(x.tint, y.tint);
            assert_eq!(x.rot, y.rot);
        }
        assert_eq!(a.summary(), b.summary());
    }

    /// The 717 glowmask bind (research/136 §3): a drawn 717 cell plans
    /// ONE glow entry whose colour is the `Color(v, v, v, v/2)` XNA
    /// truncation of `tile_ctor::lava_light_a` — `[b,b,b,a]` with
    /// `a == ((v/2)*255) as u8` (NOT `b/2`: the two truncations differ,
    /// e.g. v≈0.999 → b=254, a=127). The clock is
    /// `(anim_time*60) % 216000` (frame/draw.rs:374's convention).
    #[test]
    fn w4_g31_attach_717_glow_plan_is_the_lava_light_truncation() {
        let mut w = world(128, 128);
        // Two lava blocks; a stone above the first (the [0,1] band arm),
        // open air above the second (the [0.8,1] band, tile_ctor.rs:162).
        put(&mut w, 30, 31, 1, 0, 0);
        put(&mut w, 30, 32, 717, 0, 0);
        put(&mut w, 40, 32, 717, 0, 0);
        let wind = Wind::new(7);
        let light = LightMap::default();
        let inp = inputs();
        let s = run_shadow(&w, &wind, &light, &inp);
        assert_eq!(s.lava_glows.len(), 2, "one plan entry per drawn 717 cell");
        assert_eq!(s.lava_glows[0].0, 30);
        assert_eq!(s.lava_glows[1].0, 40);
        for &(x, y, c) in &s.lava_glows {
            let v = crate::render::tile_ctor::lava_light_a(&w, x, y, (inp.anim_time * 60.0) % 216000.0);
            let b = (v * 255.0) as u8;
            assert_eq!(c, [b, b, b, ((v / 2.0) * 255.0) as u8]);
            // The open-air cell rides the [0.8,1] band; the covered cell
            // may sit anywhere in [0,1].
            if x == 40 {
                assert!(v >= 0.8, "the SolidTile-above arm lifts toMin");
            }
        }
        // The A/B summary carries the lava count.
        assert!(s.summary().contains("lava=2"));
    }

    /// The drip seam (contract §8.5 / research/136 C5): every classified
    /// LiquidDrops cell lands in `drip_plan` in ROW-MAJOR order (the C#
    /// emits inside the Draw switch :688-694 BEFORE the `continue` — the
    /// direct-bind order g68's `emit_liquid_drops` must be called in), and
    /// the registry's `liquid_drop_cells` holds the SAME cells (the
    /// census input stays). No RNG moves in the shadow: the plan is the
    /// order pin, the gate rolls bind at the pixel merge.
    #[test]
    fn w4_g31_attach_drip_plan_preserves_the_switch_arm_order() {
        let mut w = world(128, 128);
        // Deliberately out of row-major insertion order; two drip kinds.
        put(&mut w, 40, 30, 374, 0, 0); // lava stalactite
        put(&mut w, 30, 30, 373, 0, 0); // vine tip
        put(&mut w, 35, 31, 709, 0, 0); // water candle
        put(&mut w, 36, 31, 461, 0, 0); // shimmer candle
        let wind = Wind::new(7);
        let light = LightMap::default();
        let inp = inputs();
        let s = run_shadow(&w, &wind, &light, &inp);
        assert_eq!(
            s.drip_plan,
            vec![(30, 30, 373), (40, 30, 374), (35, 31, 709), (36, 31, 461)],
            "row-major (y outer, x inner), the Draw-switch emission order"
        );
        // The registry census holds the same cells (recorded, not drained).
        assert_eq!(s.reg.liquid_drop_cells, s.drip_plan);
        // The drip cells never reach DrawSingleTile (the `continue`):
        // the world has ONLY drip cells active, so zero singles counted.
        assert_eq!(s.reg.single_tiles, 0);
        assert!(s.summary().contains("drips=4"));
    }

    /// The tiles.rs attach site: exactly one gated call, AFTER the
    /// fused walk + the inline foliage/vine passes (the C#
    /// PostDrawTiles position relative to the port's fused pass) and
    /// BEFORE the normal NPC pass; the gate is the env pin and the
    /// shadow never receives `leaf_rng`.
    #[test]
    fn w4_g31_attach_site_is_gated_and_after_the_fused_walk() {
        let src = include_str!("tiles.rs");
        let body = &src[src.find("pub(crate) fn draw_tiles(").unwrap()..];
        let body = &body[..body.find("#[cfg(test)]").unwrap()];
        assert_eq!(body.matches("tile_attach::run_shadow").count(), 1, "one attach site");
        let gate = body.find("crate::env::tilepass_shadow()").expect("the TR_TILEPASS gate");
        let call = body.find("tile_attach::run_shadow").unwrap();
        let vines = body.find("crate::render::vines::draw_vines").expect("the inline vine pass");
        let npcs = body.find("draw_phase(\"npcs\")").expect("the normal NPC pass");
        assert!(gate < call, "the gate guards the call");
        assert!(vines < gate, "the shadow runs after the fused tile walk");
        assert!(call < npcs, "the shadow runs before the NPC pass");
        // The shadow block never touches the fused pass's RNG.
        let block = &body[gate..npcs];
        assert!(!block.contains("leaf_rng"), "the shadow stream is separate");
    }
}
