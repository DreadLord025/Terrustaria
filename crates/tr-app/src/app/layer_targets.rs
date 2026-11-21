//! S1-G-13 layer-target CONSUMERS — the gated draw path that renders the
//! world through the `RenderTargetFamily` instead of the single G6-cached
//! world range. Rides `TR_LAYER_TARGETS` (default OFF: the fused frame path
//! is frozen until the draw-pass-order A/B lands — ANALYSIS "The
//! render-target family (S1-G-13)").
//!
//! Ports the consumer half of `Main.RenderToTargets` + `Main.DoDraw`'s
//! composite sites:
//! - the per-layer render callbacks (`RenderTiles`/`RenderTiles2`/
//!   `RenderWater`/`RenderBackgroundWater`/`RenderWallsAndBlacks`/
//!   `RenderBackground`, `Main.cs:57329-57386`, `:55183-55218`,
//!   `:58056-58091`) driven through [`tr_render::render_to_targets`];
//! - the skyTarget bind + clear gate (`Main.cs:62021-62027`);
//! - the layer composites (`Main.cs:62196`/`:62213`/`:63333`/`:63367`/
//!   `:63447`; the water composite `:62308` runs after the entity passes).
//!
//! The executor ([`tr_render::render_to_targets`]) owns the layer update
//! ORDER (frozen: the phase arms then tile→tile2→backWater→background→wall→
//! water). This module supplies the callbacks + the composite sequence.
//! Free functions (not `App` methods): the draw region destructures the App
//! fields, so the callbacks receive exactly the borrows they read.

use glam::Vec2;
use tr_core::lighting::{LightMap, LightParams, LightStyle};
use tr_core::{World, TILE_PX};
use tr_render::{LayerCamera, RenderTargetFamily, SceneLayer, SpriteRenderer};

use crate::assets;
use crate::render::entities::{ArmorKinds, NpcAssets, PlayerAssets};
use crate::render::sky::SkyAssets;
use crate::render::tiles::{draw_tiles, DrawScratch, TileDrawStats, TileFilter};

/// The per-frame inputs the layer callbacks + composites read, snapshotted
/// from the draw region before the executor runs.
pub(crate) struct LayerFrame<'a> {
    pub world: &'a World,
    pub lookups: &'a mut assets::Lookups,
    pub atlas: Option<&'a tr_render::Atlas>,
    pub sky: &'a SkyAssets,
    pub light: &'a mut LightMap,
    pub scratch: &'a mut DrawScratch,
    pub params: &'a LightParams,
    pub style: &'a LightStyle,
    pub entity_lights: &'a crate::entity_lights::EntityLights,
    pub anim_time: f32,
    pub waterfall: &'a mut tr_core::waterfall::WaterfallManager,
    pub liquid_alpha: &'a [f32; 15],
    pub sparkle_seed: u64,
    pub wind: &'a tr_core::wind::Wind,
    pub grass_wind_counter: f32,
    pub tree_wind_counter: f32,
    pub sunflower_wind_counter: f32,
    pub vine_wind_counter: f32,
    pub wiring: &'a tr_core::wiring::Wiring,
    pub party_is_up: bool,
    pub show_invisible_blocks: bool,
    pub tile_anim: &'a tr_core::drawdata_anim::TileAnimator,
    /// The frame's world camera (the view centre) + zoom + backbuffer size.
    pub camera: Vec2,
    pub zoom: f32,
    pub screen: Vec2,
    /// `Main.BackgroundViewMatrix`'s zoom (`ForcedMinimumZoom`) — the sky
    /// range renders at this, not the world zoom (`Main.cs:61998`).
    pub sky_zoom: f32,
    /// `Main.caveParallax` — the background composite's X shift.
    pub cave_parallax: f32,
    /// The entity inputs the direct-screen blocks draw (`DoDraw_WallsTilesNPCs`'
    /// entity arms, `Main.cs:63265-63305`); the tile-target callbacks never
    /// draw them (the `TileFilter` halves gate them out).
    pub player: Option<&'a crate::player::Player>,
    pub player_assets: &'a PlayerAssets,
    pub held: Option<(u16, f32, u16)>,
    pub armor: ArmorKinds,
    pub npcs: &'a [crate::npc::Npc],
    pub npc_assets: &'a NpcAssets,
    pub items: &'a [crate::item::ItemDrop],
    pub remote: &'a [crate::net::PlayerNet],
    pub edit_target: Option<(i32, i32)>,
    pub leaf_spawns: &'a mut Vec<crate::render::foliage::LeafSpawn>,
}

/// `Main.RenderToTargets` (`Main.cs:57277-57327`) EXECUTED with the App's
/// per-layer callbacks, then the `DoDraw` composites through the tile layer
/// with the direct-screen blocks interleaved at their C# sites:
/// backWater (`:62196`) → background (`:62213`) → wall (`:63447`) → tile2
/// (`:63367`) → the extras block (`DoDraw_Waterfalls` `:63306` + the
/// behind-tiles NPCs `:63293` + `PostDrawTiles`' foliage/vines) → tile
/// (`:63333`) → the entity tail (`:63301`, `:62256`, `:62278`). The water
/// composite (`:62308`) runs LATER — [`composite_water`] — after the
/// caller's rain/gore/projectile passes, like the C#.
///
/// `render_count` is `Main.renderCount`'s port (the lighting phase index,
/// `LightingEngine.cs:90-103`); it advances one step per call
/// (`IncrementState`, `:110-113`). Returns the accumulated tile stats.
pub(crate) fn render_layer_targets(
    gpu: &tr_render::Gpu,
    renderer: &mut SpriteRenderer,
    family: &mut RenderTargetFamily,
    frame: &mut LayerFrame<'_>,
    compositor: &mut tr_render::TargetCompositor,
    render_count: &mut i32,
    dst: &wgpu::TextureView,
    now_secs: f32,
) -> TileDrawStats {
    let mut stats = TileDrawStats::default();
    // `Main.Camera.Center` / `Camera.ScaledSize` (`Camera.cs:14,50`): the
    // port's camera is the view centre, the backbuffer the unscaled size
    // (the zoom-1 layer path — the GameViewMatrix translation is 0 at the
    // port's forced-minimum-zoom floor).
    let camera_center = [frame.camera.x, frame.camera.y];
    let camera_scaled_size = [frame.screen.x, frame.screen.y];
    // Park the frame's world range while the layer callbacks render their
    // own batches into the same slot (the C# keeps one spriteBatch per
    // callback, `Main.cs:57374-57384`).
    let parked_world = renderer.take_world();
    let count = *render_count;
    tr_render::render_to_targets(
        family,
        &gpu.device,
        &gpu.queue,
        true,
        false,
        count,
        camera_center,
        camera_scaled_size,
        |layer, view, cam| {
            render_one_layer(gpu, renderer, frame, layer, view, cam, &mut stats);
        },
    );
    // `LightingEngine.IncrementState` (`LightingEngine.cs:110-113`): the
    // phase index cycles Scan(1)→Blur(2)→MinimapUpdate(3)→ExportMetrics(0).
    *render_count = match count {
        1 => 2,
        2 => 3,
        3 => 0,
        _ => 1,
    };
    renderer.restore_world(parked_world);
    // The composites + the interleaved direct-screen blocks.
    let screen_position = [
        frame.camera.x - frame.screen.x * 0.5 / frame.zoom,
        frame.camera.y - frame.screen.y * 0.5 / frame.zoom,
    ];
    // Copy the composite scalars out so the closure does not borrow `frame`
    // (the direct-screen blocks below need `&mut frame`).
    let cave_parallax = frame.cave_parallax;
    let cam_arr = frame.camera.to_array();
    let world_zoom = frame.zoom;
    let sky_zoom = frame.sky_zoom;
    let screen_arr = frame.screen.to_array();
    let mut composite = |layer: SceneLayer| {
        let target = family.layer(layer);
        let position = tr_render::layer_composite_position(
            layer,
            target.position(),
            target.size(),
            screen_position,
            camera_center,
            cave_parallax,
        );
        // The background halves draw under `BackgroundViewMatrix` (zoom =
        // `ForcedMinimumZoom`, `Main.cs:62102`/`:62213`); the world layers
        // draw under `Main.Transform` (the world zoom).
        let zoom = if matches!(layer, SceneLayer::Background | SceneLayer::BackWater) {
            sky_zoom
        } else {
            world_zoom
        };
        renderer.draw_target_with(
            compositor,
            gpu,
            dst,
            target.view(),
            target.size(),
            position,
            cam_arr,
            zoom,
            screen_arr,
            now_secs,
            wgpu::LoadOp::Load,
        );
    };
    // `Main.cs:62196` / `:62213` — the background halves under the world.
    composite(SceneLayer::BackWater);
    composite(SceneLayer::Background);
    // `DoDraw_WallsAndBlacks` (`Main.cs:63447`).
    composite(SceneLayer::Wall);
    // `DoDraw_Tiles_NonSolid` (`Main.cs:63367`).
    composite(SceneLayer::Tile2);
    // The extras block between the tile2 and tile composites:
    // `DoDraw_Waterfalls` (`:63306`) + `DoDraw_DrawNPCsBehindTiles`
    // (`:63293`) + `PostDrawTiles(false)`'s foliage/vines
    // (`TileDrawing.cs:414-432`).
    draw_direct_screen(gpu, renderer, frame, TileFilter::ScreenExtras, dst, now_secs, &mut stats);
    // `DoDraw_Tiles_Solid` (`Main.cs:63333`).
    composite(SceneLayer::Tile);
    // The entity tail: NPCs over tiles (`:63301`), players (`:62256`),
    // items (`:62278`), the edit cursor.
    draw_direct_screen(gpu, renderer, frame, TileFilter::Screen, dst, now_secs, &mut stats);
    stats
}

/// `Main.cs:62308` — the waterTarget composite, after the entity passes
/// (the caller runs it once the rain/gore/projectile ranges are drawn).
pub(crate) fn composite_water(
    gpu: &tr_render::Gpu,
    renderer: &mut SpriteRenderer,
    family: &RenderTargetFamily,
    frame: &LayerFrame<'_>,
    compositor: &mut tr_render::TargetCompositor,
    dst: &wgpu::TextureView,
    now_secs: f32,
) {
    let target = &family.water;
    let screen_position = [
        frame.camera.x - frame.screen.x * 0.5 / frame.zoom,
        frame.camera.y - frame.screen.y * 0.5 / frame.zoom,
    ];
    let position = tr_render::layer_composite_position(
        SceneLayer::Water,
        target.position(),
        target.size(),
        screen_position,
        [frame.camera.x, frame.camera.y],
        frame.cave_parallax,
    );
    renderer.draw_target_with(
        compositor,
        gpu,
        dst,
        target.view(),
        target.size(),
        position,
        frame.camera.to_array(),
        frame.zoom,
        frame.screen.to_array(),
        now_secs,
        wgpu::LoadOp::Load,
    );
}

/// The skyTarget bind + clear gate (`Main.cs:62021-62027`) and the sky
/// capture: when [`tr_render::sky_target_active`] holds, the sky range
/// renders into the skyTarget (cleared transparent first, `:62025-62026`)
/// and the target composites onto `dst` at the screen origin
/// (`FilterManager.EndCapture`'s sky draw, `FilterManager.cs:150`);
/// otherwise the sky range renders straight to `dst` (the port's
/// no-filter equivalent of the `drawToScreen` sky path).
pub(crate) fn render_sky_target(
    gpu: &tr_render::Gpu,
    renderer: &mut SpriteRenderer,
    family: &RenderTargetFamily,
    frame: &LayerFrame<'_>,
    compositor: &mut tr_render::TargetCompositor,
    dst: &wgpu::TextureView,
    sky_active: bool,
    now_secs: f32,
) {
    if sky_active {
        // `SetRenderTarget(skyTarget)` + `Clear(Transparent)` (`:62025-62026`).
        family.sky.clear_transparent(&gpu.device, &gpu.queue);
        renderer.render_back(
            gpu,
            family.sky.view(),
            frame.camera.to_array(),
            frame.sky_zoom,
            now_secs,
            None,
        );
        // The sky merge (`FilterManager.cs:150`): a screen-space draw at
        // the origin — the identity camera (cam = screen/2, zoom 1) maps
        // world [0,0] to screen [0,0].
        let size = family.sky.texture().size();
        renderer.draw_target_with(
            compositor,
            gpu,
            dst,
            family.sky.view(),
            (size.width, size.height),
            [0.0, 0.0],
            [frame.screen.x * 0.5, frame.screen.y * 0.5],
            1.0,
            frame.screen.to_array(),
            now_secs,
            wgpu::LoadOp::Load,
        );
    } else {
        renderer.render_back(
            gpu,
            dst,
            frame.camera.to_array(),
            frame.sky_zoom,
            now_secs,
            None,
        );
    }
}

/// One `layerTarget.UpdateContent(this.RenderX)` callback — the App's
/// `RenderTiles`/`RenderTiles2`/`RenderWater`/`RenderBackgroundWater`/
/// `RenderWallsAndBlacks`/`RenderBackground` dispatch. The camera override
/// (`WorldSceneLayerTarget.cs:49-52`) is applied for the duration of the
/// render; the callback's batch renders alone into the layer view (the
/// transparent clear already ran in `update_content`).
fn render_one_layer(
    gpu: &tr_render::Gpu,
    renderer: &mut SpriteRenderer,
    frame: &mut LayerFrame<'_>,
    layer: SceneLayer,
    view: &wgpu::TextureView,
    cam: LayerCamera,
    stats: &mut TileDrawStats,
) {
    // The layer framebuffer size override (`WorldSceneLayerTarget.cs:49-50`).
    renderer.set_screen_override(Some(cam.screen_size));
    // The layer camera: `cam.screen_position` is the world-px top-left, so
    // the view centre is position + size/2; the zoom resets to 1 (`:52`).
    let layer_cam = Vec2::new(
        cam.screen_position[0] + cam.screen_size[0] * 0.5,
        cam.screen_position[1] + cam.screen_size[1] * 0.5,
    );
    let layer_screen = Vec2::new(cam.screen_size[0], cam.screen_size[1]);
    match layer {
        SceneLayer::Tile | SceneLayer::Tile2 => {
            // `RenderTiles`/`RenderTiles2` (`Main.cs:57329-57369`):
            // `DrawTiles(solidLayer, intoRenderTargets: true)`.
            let filter = if layer == SceneLayer::Tile {
                TileFilter::Solid
            } else {
                TileFilter::NonSolid
            };
            renderer.begin_world_batch();
            let s = draw_tile_pass(renderer, frame, filter, layer_cam, cam.zoom, layer_screen);
            accumulate(stats, &s);
            renderer.render_world_only(
                gpu, view, layer_cam.to_array(), cam.zoom, frame.anim_time, None,
            );
        }
        SceneLayer::Wall => {
            // `RenderWallsAndBlacks` (`Main.cs:58056-58067`): `RenderBlacks`
            // (the dark-cell merge) THEN `RenderWalls`, one batch.
            renderer.begin_world_batch();
            let blacks =
                draw_tile_pass(renderer, frame, TileFilter::Wall, layer_cam, cam.zoom, layer_screen);
            accumulate(stats, &blacks);
            let (x0, y0, x1, y1) = view_rect(frame.world, layer_cam, layer_screen);
            crate::render::tiles::draw_wall_layer(
                renderer, frame.world, frame.lookups, frame.atlas, frame.light,
                frame.style, frame.anim_time, x0, y0, x1, y1, stats,
            );
            renderer.render_world_only(
                gpu, view, layer_cam.to_array(), cam.zoom, frame.anim_time, None,
            );
        }
        SceneLayer::Water | SceneLayer::BackWater => {
            // `RenderWater` / `RenderBackgroundWater` (`Main.cs:57371-57386`,
            // `:55183-55198`): `DrawWaters()` / `DrawWaters(isBackground: true)`.
            renderer.begin_world_batch();
            let pass = if layer == SceneLayer::Water {
                crate::render::liquids::LiquidPass::Foreground
            } else {
                crate::render::liquids::LiquidPass::Background
            };
            let (x0, y0, x1, y1) = view_rect(frame.world, layer_cam, layer_screen);
            let draws = tr_core::liquid_draw::build(frame.world, x0, y0, x1, y1);
            let mut water_style = crate::biome::calculate_water_style(frame.world, layer_cam);
            if let Some(n) = crate::env::water_style() {
                water_style = n.min(14);
            }
            stats.liquids += crate::render::liquids::draw_waters(
                renderer, frame.lookups, frame.light, frame.style, frame.anim_time,
                &draws, pass, water_style, frame.liquid_alpha,
            );
            renderer.render_world_only(
                gpu, view, layer_cam.to_array(), cam.zoom, frame.anim_time, None,
            );
        }
        SceneLayer::Background | SceneLayer::BackgroundSwap => {
            // `RenderBackground` (`Main.cs:55200-55218`): `DrawBackground`
            // — the underground backdrop, pushed into the back range. The
            // C# renders it under `BackgroundViewMatrix` (zoom =
            // `ForcedMinimumZoom`); the layer override resets the zoom to 1
            // (`WorldSceneLayerTarget.cs:52`) and `draw_background` lays out
            // in unzoomed screen space, so the backdrop renders at zoom 1.
            let parked_back = renderer.take_back();
            renderer.begin_back_batch();
            // `Main.hellBlackBoxBottom` is carried across frames on the
            // App; the layer callback's own black-box line starts fresh
            // (the box consumes the previous magma pass of THIS target).
            let mut hell_bottom = 0.0;
            crate::render::bg::draw_background(
                renderer, layer_cam, 1.0, layer_screen, frame.world,
                frame.sky, frame.light, frame.cave_parallax, frame.anim_time,
                &mut hell_bottom,
            );
            renderer.render_back(gpu, view, layer_cam.to_array(), 1.0, frame.anim_time, None);
            renderer.restore_back(parked_back);
        }
    }
    renderer.set_screen_override(None);
}

/// One `draw_tiles` pass under a filter — the tile-target halves
/// (`Solid`/`NonSolid`/`Wall`, into the layer view) or the direct-screen
/// blocks (`ScreenExtras`/`Screen`, into the frame). The entity args ride
/// along; the filter gates which passes actually draw.
fn draw_tile_pass(
    renderer: &mut SpriteRenderer,
    frame: &mut LayerFrame<'_>,
    filter: TileFilter,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
) -> TileDrawStats {
    draw_tiles(
        renderer,
        frame.world,
        frame.lookups,
        frame.atlas,
        camera,
        zoom,
        screen,
        frame.params,
        frame.light,
        frame.scratch,
        frame.style,
        frame.entity_lights,
        frame.anim_time,
        frame.waterfall,
        frame.player,
        frame.player_assets,
        frame.held,
        frame.armor,
        frame.npcs,
        frame.npc_assets,
        frame.items,
        frame.remote,
        frame.edit_target,
        frame.liquid_alpha,
        frame.sparkle_seed,
        frame.wind,
        frame.grass_wind_counter,
        frame.tree_wind_counter,
        frame.sunflower_wind_counter,
        frame.leaf_spawns,
        frame.vine_wind_counter,
        frame.wiring,
        frame.party_is_up,
        frame.show_invisible_blocks,
        frame.tile_anim,
        filter,
    )
}

/// One direct-screen block: clear the world range, run the filtered pass
/// at the FRAME camera/zoom, render it onto `dst` over the composites.
fn draw_direct_screen(
    gpu: &tr_render::Gpu,
    renderer: &mut SpriteRenderer,
    frame: &mut LayerFrame<'_>,
    filter: TileFilter,
    dst: &wgpu::TextureView,
    now_secs: f32,
    stats: &mut TileDrawStats,
) {
    renderer.begin_world_batch();
    let s = draw_tile_pass(renderer, frame, filter, frame.camera, frame.zoom, frame.screen);
    accumulate(stats, &s);
    renderer.render_world_only(
        gpu, dst, frame.camera.to_array(), frame.zoom, now_secs, None,
    );
}

/// The visible tile rect for a layer camera — `draw_tiles`'s own view-rect
/// math, reused for the wall/liquid callbacks that walk the cells directly.
fn view_rect(world: &World, camera: Vec2, screen: Vec2) -> (i32, i32, i32, i32) {
    let half = screen * 0.5;
    let min = camera - half;
    let max = camera + half;
    let x0 = ((min.x / TILE_PX).floor() as i32 - 1).clamp(0, world.width - 1);
    let x1 = ((max.x / TILE_PX).ceil() as i32 + 1).clamp(0, world.width - 1);
    let y0 = ((min.y / TILE_PX).floor() as i32 - 1).clamp(0, world.height - 1);
    let y1 = ((max.y / TILE_PX).ceil() as i32 + 1).clamp(0, world.height - 1);
    (x0, y0, x1, y1)
}

/// Fold one callback's [`TileDrawStats`] into the frame accumulator.
fn accumulate(dst: &mut TileDrawStats, src: &TileDrawStats) {
    dst.visible += src.visible;
    dst.walls += src.walls;
    dst.walls_culled += src.walls_culled;
    dst.dark_cells += src.dark_cells;
    dst.dark_rects += src.dark_rects;
    dst.chunks_skipped += src.chunks_skipped;
    dst.quads += src.quads;
    dst.foliage += src.foliage;
    dst.liquids += src.liquids;
    dst.waterfalls += src.waterfalls;
    dst.items += src.items;
    dst.instances += src.instances;
}
