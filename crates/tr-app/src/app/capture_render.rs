//! S1-G-13 consumers — `Main.DrawCapture`'s chunk render
//! (`Main.cs:56673-57081`): the capture camera's [`CaptureRenderer`] GPU
//! half. One 128-tile chunk renders through the render-target family into
//! the 2048² framebuffer (`CaptureCamera._frameBuffer`,
//! `CaptureCamera.cs:97`), then downscales when `UseScaling`
//! (`CaptureCamera.cs:194-204`) and reads the pixels back.
//!
//! The C# saves/restores a dozen `Main.*` statics around the chunk render
//! (`:56675-56700`, `:57060-57080`); the port passes the chunk's camera /
//! liquid-alpha / biome overrides into the passes directly instead of
//! mutating the App, so the live frame state is untouched.
//!
//! Rides `TR_LAYER_TARGETS` like the frame-path consumers; with the gate
//! off the [`crate::app::capture_driver`] keeps its no-op stub (the capture
//! drains its queue without pixels — the S1-E-17 behaviour).
//!
//! Free functions (not methods): the executor borrows the family mutably
//! while the layer callbacks borrow the renderer + world state, so the
//! fields are passed as disjoint arguments.

use glam::Vec2;
use tr_core::lighting::{LightMap, LightParams, LightStyle};
use tr_render::{LayerCamera, RenderTargetFamily, SceneLayer, SpriteRenderer, TargetCompositor};

use crate::capture::{CaptureChunk, CaptureRect, CaptureRenderer, CaptureSettings};
use crate::render::tiles::{draw_tiles, DrawScratch, TileDrawStats, TileFilter};

/// `CaptureCamera.FRAMEBUFFER_PIXEL_SIZE` (`CaptureCamera.cs:36`).
const FB: u32 = 2048;

/// The world-state inputs the capture passes read (the App fields the draw
/// region destructures), borrowed for the chunk render.
pub(crate) struct CaptureWorld<'a> {
    pub world: &'a tr_core::World,
    pub lookups: &'a mut crate::assets::Lookups,
    pub atlas: Option<&'a tr_render::Atlas>,
    pub sky: &'a crate::render::sky::SkyAssets,
    pub light: &'a mut LightMap,
    pub scratch: &'a mut DrawScratch,
    pub params: &'a LightParams,
    pub style: &'a LightStyle,
    pub entity_lights: &'a crate::entity_lights::EntityLights,
    pub anim_time: f32,
    pub waterfall: &'a mut tr_core::waterfall::WaterfallManager,
    pub wind: &'a tr_core::wind::Wind,
    pub grass_wind_counter: f32,
    pub tree_wind_counter: f32,
    pub sunflower_wind_counter: f32,
    pub vine_wind_counter: f32,
    pub wiring: &'a tr_core::wiring::Wiring,
    pub party_is_up: bool,
    pub tile_anim: &'a tr_core::drawdata_anim::TileAnimator,
    pub player_assets: &'a crate::render::entities::PlayerAssets,
    pub npc_assets: &'a crate::render::entities::NpcAssets,
    pub leaf_spawns: &'a mut Vec<crate::render::foliage::LeafSpawn>,
    /// The capture's biome background-index override inputs
    /// (`Main.cs:56675-56680`): the surface chains' texture-id tables read
    /// the world's per-biome styles; the clouds ride the live frame's
    /// sprites (the C# scales their positions for the capture camera,
    /// `:56822-56828` — the port draws them at the chunk camera unscaled,
    /// a documented approximation).
    pub clouds: &'a [crate::render::sky::CloudSprite],
    /// `Main.GameUpdateCount` — the animated background sheets' counter.
    pub counter: u32,
    /// `Main.mouseTextColor` — the Mushroom glow's pulse.
    pub mouse_text_color: u8,
}

/// The GPU capture renderer — `Main.DrawCapture` through the family. Owns
/// the 2048² framebuffer + the scaled framebuffer (`CaptureCamera.cs:97`,
/// `:155`); the App's family supplies the seven world layers. The C# swaps
/// `Main.waterTarget` for the camera's own 2048² one (`:56723`/`:57067`);
/// the port renders the capture water into the family's water layer at the
/// chunk camera (the live frame re-renders it next frame — the offscreen
/// check fires), a documented simplification.
pub(crate) struct GpuCaptureRenderer<'a> {
    pub gpu: &'a tr_render::Gpu,
    pub renderer: &'a mut SpriteRenderer,
    pub family: &'a mut RenderTargetFamily,
    pub compositor: &'a mut TargetCompositor,
    pub cw: CaptureWorld<'a>,
    /// The `captureEntities` arms' inputs (`Main.cs:56876-57017`); `None`
    /// when the driver has no live entity state (headless tests).
    pub entities: Option<crate::app::capture_entities::CaptureEntities<'a>>,
    /// `CaptureCamera._frameBuffer` (`:97`).
    pub frame_buffer: Option<(wgpu::Texture, wgpu::TextureView)>,
    /// `CaptureCamera._scaledFrameBuffer` (`:155`), sized per capture.
    pub scaled_buffer: Option<(wgpu::Texture, wgpu::TextureView, u32)>,
}

impl<'a> GpuCaptureRenderer<'a> {
    fn frame_buffer(&mut self) -> wgpu::TextureView {
        let format = self.gpu.config.format;
        let device = &self.gpu.device;
        self.frame_buffer
            .get_or_insert_with(|| {
                let tex = tr_render::offscreen_target_on(device, format, FB, FB, "capture-frame");
                let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
                (tex, view)
            })
            .1
            .clone()
    }

    fn scaled_buffer(&mut self, size: u32) -> wgpu::TextureView {
        let format = self.gpu.config.format;
        let device = &self.gpu.device;
        let need = self.scaled_buffer.as_ref().is_none_or(|(_, _, s)| *s != size);
        if need {
            let tex = tr_render::offscreen_target_on(device, format, size, size, "capture-scaled");
            let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
            self.scaled_buffer = Some((tex, view, size));
        }
        self.scaled_buffer.as_ref().unwrap().1.clone()
    }
}

impl CaptureRenderer for GpuCaptureRenderer<'_> {
    /// `CaptureCamera.DrawTick`'s render half (`CaptureCamera.cs:186-209`):
    /// `Main.DrawCapture(chunk.Area, settings, this)` into the framebuffer,
    /// then the `UseScaling` downscale + `GetData` readback. Returns the
    /// chunk's RGBA bytes (scaled size when `use_scaling`, else the raw
    /// 2048² framebuffer the caller crops).
    fn render_chunk(
        &mut self,
        chunk: &CaptureChunk,
        settings: &CaptureSettings,
        scaled_framebuffer_size: i32,
    ) -> Option<Vec<u8>> {
        let area = chunk.area;
        // `Main.screenWidth/Height = area.Width/Height << 4`,
        // `screenPosition = area.TopLeft() * 16` (`Main.cs:56698-56700`).
        let screen = Vec2::new((area.width * 16) as f32, (area.height * 16) as f32);
        let screen_position = Vec2::new((area.x * 16) as f32, (area.y * 16) as f32);
        let camera = screen_position + screen * 0.5;
        // `Main.liquidAlpha[i] = (i == biome.WaterStyle) ? 1 : 0` (`:56701-
        // 56707`, index 1 skipped — the lava slot keeps its value).
        let mut liquid_alpha = [0.0f32; 15];
        let ws = settings.biome.water_style.clamp(0, 14) as usize;
        for (i, a) in liquid_alpha.iter_mut().enumerate() {
            if i != 1 {
                *a = if i == ws { 1.0 } else { 0.0 };
            }
        }
        // `Lighting.Initialize()` + four `LightTiles(area)` (`:56727-56732`):
        // the four calls walk the modern engine's four-state cycle exactly
        // once — Rebuild sets MinimapUpdate(0), then MinimapUpdate (map
        // off → no-op) → ExportMetrics → Scan → Blur+Present
        // (`LightingEngine.cs:76-113,237-244`). The visible light is ONE
        // scan+blur of the area inflated 28 — the port's single
        // `compute_into` (AREA_PADDING = 28) is truly equivalent
        // (ANALYSIS "The four LightTiles calls"). The legacy modes keep
        // the frame's presented light — documented deviation.
        let x0 = area.x.max(0);
        let y0 = area.y.max(0);
        let x1 = (area.x + area.width).min(self.cw.world.width - 1);
        let y1 = (area.y + area.height).min(self.cw.world.height - 1);
        self.cw.light.compute_into(self.cw.world, x0, y0, x1, y1, self.cw.params);

        let fb = self.frame_buffer();
        // `GraphicsDevice.Clear(Transparent)` (`:56756`).
        clear_view(self.gpu, &fb);
        // The chunk renders at zoom 1 into the 2048² framebuffer; the
        // screen-size override makes the sprite globals match it.
        self.renderer.set_screen_override(Some([FB as f32, FB as f32]));
        // `camera.BeginDrawCapture()` (`:56724`): the skyTarget bind + clear
        // (`CaptureCamera.cs:223-224`).
        self.family.sky.clear_transparent(&self.gpu.device, &self.gpu.queue);
        // Drive the seven layers through the executor with renderCount 99
        // (`:56728`) — no phase arm; the fresh/shifted targets are partially
        // offscreen so the fixed offscreen order renders them. Disjoint
        // field borrows: the executor owns the family, the callback owns the
        // renderer + world state.
        let gpu = self.gpu;
        let renderer = &mut *self.renderer;
        let cw = &mut self.cw;
        let capture_background = settings.capture_background;
        let anim_time = cw.anim_time;
        // The biome background-index override (`Main.cs:56675-56680`):
        // fresh zeroed arrays + `UpdateBGVisibility_*(bgIndex, 1f)` —
        // front[bgIndex] = 1, the far layer(s) per the style switch. The
        // live frame's arrays are untouched (the port passes them as
        // arguments — the C# save/restore equivalent).
        let mut bg_front = [0.0f32; 16];
        let mut bg_far = [0.0f32; 16];
        crate::biome::advance_bg_alphas(
            &mut bg_front,
            &mut bg_far,
            settings.biome.background_index,
            cw.world.jungle_bg_style as i32,
            cw.world.hallow_bg_style as i32,
            1.0,
        );
        // GATE RNG fix: one leaf-roll stream per chunk render — the C#
        // `DrawCapture`'s `DrawTiles` calls share the live `TileDrawing._rand`
        // (`TileDrawing.cs:104`); the port pins the chunk's stream to the
        // capture's sparkle seed (0) so the halves consume it in order.
        cw.scratch.leaf_rng_shared = true;
        cw.scratch.leaf_rng =
            Some(tr_core::tile_light::FastRandom::new(0x9E37_79B9_7F4A_7C15));
        tr_render::render_to_targets(
            self.family,
            &gpu.device,
            &gpu.queue,
            true,
            false,
            99,
            [camera.x, camera.y],
            [screen.x, screen.y],
            |layer, view, cam| {
                render_capture_layer(
                    gpu, renderer, cw, layer, view, cam, &liquid_alpha, ws,
                    capture_background, &bg_front, &bg_far,
                );
            },
        );
        // Composite the layers onto the framebuffer in `DrawCapture` order
        // with the `captureEntities` arms interleaved at their C# sites:
        // background (`:56921-56926`) → wall (`:56927`) → the mid entities
        // (`:56928-56953`) → tile2 (`:56957-56963`) → the waterfalls
        // (`:56964-56972`) → tile (`:56980-56988`) → the entity tail
        // (`:56989-57017`) → water (`:57018-57027`).
        let capture_entities = settings.capture_entities;
        composite_capture(gpu, renderer, self.family, self.compositor, camera, screen, &fb, anim_time, SceneLayer::Background);
        composite_capture(gpu, renderer, self.family, self.compositor, camera, screen, &fb, anim_time, SceneLayer::Wall);
        if let Some(ents) = self.entities.as_ref() {
            crate::app::capture_entities::draw_capture_mid_entities(
                gpu, renderer, &self.cw, ents, camera, screen, &fb, capture_entities,
            );
        }
        composite_capture(gpu, renderer, self.family, self.compositor, camera, screen, &fb, anim_time, SceneLayer::Tile2);
        if capture_entities && self.entities.is_some() {
            crate::app::capture_entities::draw_capture_waterfalls(
                gpu, renderer, &mut self.cw, camera, screen, &fb, &liquid_alpha,
            );
        }
        composite_capture(gpu, renderer, self.family, self.compositor, camera, screen, &fb, anim_time, SceneLayer::Tile);
        if capture_entities {
            if let Some(ents) = self.entities.as_mut() {
                crate::app::capture_entities::draw_capture_entity_tail(
                    gpu, renderer, &mut self.cw, ents, camera, screen, &fb,
                );
            }
        }
        composite_capture(gpu, renderer, self.family, self.compositor, camera, screen, &fb, anim_time, SceneLayer::Water);
        // GATE RNG fix: the chunk's stream is consumed; restore the fused
        // per-call re-seed semantics for whatever draws next.
        self.cw.scratch.leaf_rng_shared = false;
        self.cw.scratch.leaf_rng = None;
        self.renderer.set_screen_override(None);
        // `UseScaling`: downscale the framebuffer into the scaled buffer
        // (`CaptureCamera.cs:196-200`) and read it back (`:202`).
        if settings.use_scaling && scaled_framebuffer_size > 0 {
            let size = scaled_framebuffer_size as u32;
            let scaled = self.scaled_buffer(size);
            clear_view(self.gpu, &scaled);
            self.renderer.draw_target_with(
                self.compositor,
                self.gpu,
                &scaled,
                &fb,
                (FB, FB),
                [0.0, 0.0],
                [size as f32 * 0.5, size as f32 * 0.5],
                1.0,
                [size as f32, size as f32],
                self.cw.anim_time,
                wgpu::LoadOp::Load,
            );
            let tex = &self.scaled_buffer.as_ref().unwrap().0;
            Some(tr_render::read_texture_rgba(
                &self.gpu.device,
                &self.gpu.queue,
                tex,
                self.gpu.config.format,
            ))
        } else {
            // The no-scaling arm reads the raw framebuffer; the caller's
            // save path crops the chunk (`CaptureCamera.cs:205-209`).
            let tex = &self.frame_buffer.as_ref().unwrap().0;
            Some(tr_render::read_texture_rgba(
                &self.gpu.device,
                &self.gpu.queue,
                tex,
                self.gpu.config.format,
            ))
        }
    }

    fn save_chunk(&mut self, chunk: &CaptureChunk, settings: &CaptureSettings, rgba: &[u8]) {
        save_capture_chunk_png(rgba, FB as i32, chunk, &settings.output_name);
    }
}

/// One capture layer callback — the chunk-camera variant of the frame
/// path's [`crate::app::layer_targets`] callbacks. The biome's water style
/// overrides the liquid passes (`Main.cs:56899`/`:57026`); the background
/// pass renders the surface chains with the capture-only bg-alpha arrays
/// (the `UpdateBGVisibility_*` override, `:56675-56680`) then the
/// underground backdrop (`DrawBackground`, `:56924`).
#[allow(clippy::too_many_arguments)]
fn render_capture_layer(
    gpu: &tr_render::Gpu,
    renderer: &mut SpriteRenderer,
    cw: &mut CaptureWorld<'_>,
    layer: SceneLayer,
    view: &wgpu::TextureView,
    cam: LayerCamera,
    liquid_alpha: &[f32; 15],
    ws: usize,
    capture_background: bool,
    bg_front: &[f32; 16],
    bg_far: &[f32; 16],
) {
    renderer.set_screen_override(Some(cam.screen_size));
    let layer_cam = Vec2::new(
        cam.screen_position[0] + cam.screen_size[0] * 0.5,
        cam.screen_position[1] + cam.screen_size[1] * 0.5,
    );
    let layer_screen = Vec2::new(cam.screen_size[0], cam.screen_size[1]);
    match layer {
        SceneLayer::Tile | SceneLayer::Tile2 => {
            let filter = if layer == SceneLayer::Tile {
                TileFilter::Solid
            } else {
                TileFilter::NonSolid
            };
            renderer.begin_world_batch();
            let _ = draw_capture_tiles(renderer, cw, filter, layer_cam, layer_screen, liquid_alpha);
            renderer.render_world_only(gpu, view, layer_cam.to_array(), cam.zoom, cw.anim_time, None);
        }
        SceneLayer::Wall => {
            renderer.begin_world_batch();
            let _ = draw_capture_tiles(renderer, cw, TileFilter::Wall, layer_cam, layer_screen, liquid_alpha);
            let (x0, y0, x1, y1) = capture_view_rect(cw.world, layer_cam, layer_screen);
            let mut stats = TileDrawStats::default();
            crate::render::tiles::draw_wall_layer(
                renderer, cw.world, cw.lookups, cw.atlas, cw.light,
                cw.style, cw.anim_time, x0, y0, x1, y1, &mut stats,
            );
            renderer.render_world_only(gpu, view, layer_cam.to_array(), cam.zoom, cw.anim_time, None);
        }
        SceneLayer::Water | SceneLayer::BackWater => {
            renderer.begin_world_batch();
            let pass = if layer == SceneLayer::Water {
                crate::render::liquids::LiquidPass::Foreground
            } else {
                crate::render::liquids::LiquidPass::Background
            };
            let (x0, y0, x1, y1) = capture_view_rect(cw.world, layer_cam, layer_screen);
            let draws = tr_core::liquid_draw::build(cw.world, x0, y0, x1, y1);
            crate::render::liquids::draw_waters(
                renderer, cw.lookups, cw.light, cw.style, cw.anim_time,
                &draws, pass, ws, liquid_alpha,
            );
            renderer.render_world_only(gpu, view, layer_cam.to_array(), cam.zoom, cw.anim_time, None);
        }
        SceneLayer::Background | SceneLayer::BackgroundSwap => {
            if capture_background {
                let parked = renderer.take_back();
                renderer.begin_back_batch();
                // `DrawSimpleSurfaceBackground` + `DrawSurfaceBG`
                // (`Main.cs:56767`, `:56837`): the surface chains at the
                // chunk camera with the capture-only bg-alpha arrays. The
                // sky tint is the frame's `ColorOfTheSkies` (the C#
                // recomputes it from the biome's TileColor via SetBackColor,
                // `:56709-56722` — the port keeps the live sky colour, a
                // documented approximation); no ambience in the capture
                // (the C# draws none — the ambient entities are a DoDraw
                // pass, not a DrawCapture one).
                let tint = crate::render::sky::color_of_the_skies(
                    cw.world.time, cw.world.day_time, cw.world.moon_phase,
                );
                let amb_ctx = crate::ambience::SkyDrawCtx {
                    camera: layer_cam,
                    zoom: 1.0,
                    screen: layer_screen,
                    color_of_the_skies: tint,
                    atmo: 1.0,
                    bg_alpha_front: bg_front,
                    background_enabled: true,
                };
                let mut amb_last = f32::INFINITY;
                crate::render::sky::draw_surface_bg(
                    renderer, layer_cam, 1.0, layer_screen, cw.world, cw.sky,
                    cw.clouds, bg_front, bg_far, tint, 0.0, true, cw.anim_time,
                    cw.counter, cw.mouse_text_color, &[], cw.lookups,
                    &amb_ctx, &mut amb_last,
                );
                // `DrawBackground()` (`Main.cs:56924`): the underground
                // backdrop under the surface chains.
                let mut hell_bottom = 0.0;
                crate::render::bg::draw_background(
                    renderer, layer_cam, 1.0, layer_screen, cw.world,
                    cw.sky, cw.light, 0.0, cw.anim_time, &mut hell_bottom,
                );
                renderer.render_back(gpu, view, layer_cam.to_array(), 1.0, cw.anim_time, None);
                renderer.restore_back(parked);
            }
        }
    }
    renderer.set_screen_override(None);
}

/// One `DrawTiles(solidLayer, intoRenderTargets: false)` capture pass — the
/// chunk camera, the biome's liquid alpha. The entity args stay empty: the
/// `captureEntities` arms draw through [`crate::app::capture_entities`] at
/// their C# composite sites (`Main.cs:56876-57017`), not through the tile
/// pass (the C# `DrawCapture` calls `DrawTiles` without entities too).
#[allow(clippy::too_many_arguments)]
fn draw_capture_tiles(
    renderer: &mut SpriteRenderer,
    cw: &mut CaptureWorld<'_>,
    filter: TileFilter,
    camera: Vec2,
    screen: Vec2,
    liquid_alpha: &[f32; 15],
) -> TileDrawStats {
    draw_tiles(
        renderer,
        cw.world,
        cw.lookups,
        cw.atlas,
        camera,
        1.0,
        screen,
        cw.params,
        cw.light,
        cw.scratch,
        cw.style,
        cw.entity_lights,
        cw.anim_time,
        cw.waterfall,
        None,
        cw.player_assets,
        None,
        [0, 0, 0],
        &[],
        cw.npc_assets,
        &[],
        &[],
        None,
        liquid_alpha,
        0,
        cw.wind,
        cw.grass_wind_counter,
        cw.tree_wind_counter,
        cw.sunflower_wind_counter,
        cw.leaf_spawns,
        cw.vine_wind_counter,
        cw.wiring,
        cw.party_is_up,
        false,
        cw.tile_anim,
        filter,
    )
}

/// Composite ONE capture layer onto the framebuffer — the `DrawCapture`
/// composite sites (`Main.cs:56921-57027`): background → wall → tile2 →
/// tile → water, with the `captureEntities` arms interleaved between them
/// (the caller drives the order).
#[allow(clippy::too_many_arguments)]
fn composite_capture(
    gpu: &tr_render::Gpu,
    renderer: &mut SpriteRenderer,
    family: &RenderTargetFamily,
    compositor: &mut TargetCompositor,
    camera: Vec2,
    screen: Vec2,
    fb: &wgpu::TextureView,
    anim_time: f32,
    layer: SceneLayer,
) {
    let screen_position = [camera.x - screen.x * 0.5, camera.y - screen.y * 0.5];
    let target = family.layer(layer);
    let position = tr_render::layer_composite_position(
        layer,
        target.position(),
        target.size(),
        screen_position,
        [camera.x, camera.y],
        0.0,
    );
    renderer.draw_target_with(
        compositor,
        gpu,
        fb,
        target.view(),
        target.size(),
        position,
        camera.to_array(),
        1.0,
        [FB as f32, FB as f32],
        anim_time,
        wgpu::LoadOp::Load,
    );
}

/// Clear a view transparent (`GraphicsDevice.Clear(Color.Transparent)`).
fn clear_view(gpu: &tr_render::Gpu, view: &wgpu::TextureView) {
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("capture-clear"),
        });
    {
        let _pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("capture-clear-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
    gpu.queue.submit(Some(enc.finish()));
}

/// The capture view rect (zoom 1).
fn capture_view_rect(world: &tr_core::World, camera: Vec2, screen: Vec2) -> (i32, i32, i32, i32) {
    let half = screen * 0.5;
    let min = camera - half;
    let max = camera + half;
    let x0 = ((min.x / 16.0).floor() as i32 - 1).clamp(0, world.width - 1);
    let x1 = ((max.x / 16.0).ceil() as i32 + 1).clamp(0, world.width - 1);
    let y0 = ((min.y / 16.0).floor() as i32 - 1).clamp(0, world.height - 1);
    let y1 = ((max.y / 16.0).ceil() as i32 + 1).clamp(0, world.height - 1);
    (x0, y0, x1, y1)
}

/// The capture save path — `CaptureCamera.SaveImage` (`CaptureCamera.cs:284-
/// 312`): `SavePath/Captures/<name>.png`. The port writes beside the world
/// saves (`tr_core::save::default_world_dir`'s parent).
pub(crate) fn save_capture_png(rgba: &[u8], width: i32, height: i32, name: &str) -> bool {
    let Some(dir) = tr_core::save::default_world_dir() else {
        return false;
    };
    let Some(root) = dir.parent() else {
        return false;
    };
    let captures = root.join(crate::capture::CAPTURE_DIRECTORY);
    if std::fs::create_dir_all(&captures).is_err() {
        return false;
    }
    let path = captures.join(format!("{name}.png"));
    image::save_buffer(&path, rgba, width as u32, height as u32, image::ColorType::Rgba8).is_ok()
}

/// The no-scaling chunk save — `Captures/<folder>/<x>-<y>.png`
/// (`CaptureCamera.cs:314-368`): crop the raw framebuffer to the chunk's
/// scaled area and write it.
pub(crate) fn save_capture_chunk_png(
    rgba: &[u8],
    fb_width: i32,
    chunk: &CaptureChunk,
    folder: &str,
) -> bool {
    let area: CaptureRect = chunk.scaled_area;
    let w = area.width.max(0) as usize;
    let h = area.height.max(0) as usize;
    if w == 0 || h == 0 {
        return false;
    }
    let stride = fb_width as usize * 4;
    let mut cropped = Vec::with_capacity(w * h * 4);
    for row in 0..h {
        let src = ((area.y as usize + row) * stride) + (area.x as usize * 4);
        if src + w * 4 <= rgba.len() {
            cropped.extend_from_slice(&rgba[src..src + w * 4]);
        }
    }
    let Some(dir) = tr_core::save::default_world_dir() else {
        return false;
    };
    let Some(root) = dir.parent() else {
        return false;
    };
    let captures = root.join(crate::capture::CAPTURE_DIRECTORY).join(folder);
    if std::fs::create_dir_all(&captures).is_err() {
        return false;
    }
    let path = captures.join(format!("{}-{}.png", chunk.area.x, chunk.area.y));
    image::save_buffer(&path, &cropped, w as u32, h as u32, image::ColorType::Rgba8).is_ok()
}
