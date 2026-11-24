//! The render-target FAMILY (S1-G-13): `Main.InitTargets`' two families —
//! the seven world-space `WorldSceneLayerTarget` caches (water, backWater,
//! tile, tile2, wall, background, backgroundSwap) and the three full-screen
//! `RenderTarget2D`s (skyTarget, screenTarget, screenTargetSwap) — plus the
//! layer-cache scheduling (`RenderToTargets`), the background double-buffer
//! swap and the skyTarget capture gate.
//!
//! Ports: `Main.cs:58160-58216` (`InitTargets`), `Main.cs:58093-58150`
//! (`ReleaseTargets`), `Main.cs:58152-58158` (`EnsureRenderTargetContent`),
//! `Main.cs:57277-57327` (`RenderToTargets`), `Main.cs:58047-58054`
//! (`ApplyPendingBackgroundTargetSwap`), `Main.cs:62021-62027` (the skyTarget
//! gate), `Main.cs:61751-61766` (the `drawToScreen`/`targetSet` transition),
//! `Graphics/WorldSceneLayerTarget.cs:7-70`.
//!
//! This module is the seam the capture mode (S1-E-17) waits on. The target
//! plumbing + layout math + scheduling are ported and tested here, AND the
//! consumer half now lives here too: [`render_to_targets`] EXECUTES the
//! `RenderToTargets` plan against a live family (driving
//! [`WorldSceneLayerTarget::update_content`] per layer with the App's
//! per-layer render callbacks), [`ScreenTarget::clear_transparent`] is the
//! skyTarget bind's clear (`Main.cs:62025-62026`), and
//! [`layer_composite_position`] is the six composite sites' destination math
//! (`Main.cs:62196`/`:62213`/`:62308`/`:63333`/`:63367`/`:63447`). The App
//! wires the callbacks + the skyTarget gate + `Main.DrawCapture`'s chunk
//! render (S1-G-13 consumers); the default frame path is unchanged — the
//! consumers ride an env gate until the draw-pass-order A/B lands (ANALYSIS
//! "The render-target family (S1-G-13)").

use crate::gpu::offscreen_target_on;

/// `Main.MaxWorldViewSize` (`Main.cs:677-681`, set at `:66714`): 1920×1200.
pub const MAX_WORLD_VIEW_SIZE: (i32, i32) = (1920, 1200);
/// `Main._renderTargetMaxSize` per graphics profile
/// (`SetGraphicsProfileInternal`, `Main.cs:3708-3718`): HiDef 4096, Reach 2048.
pub const RENDER_TARGET_MAX_HIDEF: i32 = 4096;
pub const RENDER_TARGET_MAX_REACH: i32 = 2048;

/// The computed target geometry of one `InitTargets` run: the world-layer
/// size (backbuffer clamped to the view cap, padded by `offScreenRange` on
/// every side) and the full-screen size (the raw backbuffer).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetLayout {
    /// `num`/`num2` after the padding (`Main.cs:58180-58181`) — every
    /// `WorldSceneLayerTarget` is this size.
    pub layer_size: (u32, u32),
    /// `Main.offScreenRange` (`Main.cs:58175-58179`): 192, shrunk so the
    /// padded width fits `_renderTargetMaxSize`.
    pub offscreen_range: i32,
    /// The backbuffer size — skyTarget/screenTarget/screenTargetSwap
    /// (`Main.cs:58194-58196`).
    pub screen_size: (u32, u32),
}

/// `Main.InitTargets`' size math (`Main.cs:58167-58181`), pure. `val` is
/// `MaxWorldViewSize.X` — the C# clamps BOTH axes with the X cap (`:58167-
/// 58174`) — raised to 4096 only for `SupportWideScreen && HiDef` (`:58169-
/// 58172`). `offScreenRange` starts at 192 and shrinks to
/// `(max - num) / 2` (C# int division) when the padded width would exceed
/// `_renderTargetMaxSize`.
pub fn init_target_layout(
    backbuffer: (u32, u32),
    hidef: bool,
    support_wide_screen: bool,
    render_target_max_size: i32,
) -> TargetLayout {
    // Main.cs:58167-58174
    let mut val = MAX_WORLD_VIEW_SIZE.0;
    if support_wide_screen && hidef {
        val = 4096;
    }
    let mut num = (backbuffer.0 as i32).min(val);
    let mut num2 = (backbuffer.1 as i32).min(val);
    // Main.cs:58175-58181
    let mut offscreen_range = 192;
    if num + offscreen_range * 2 > render_target_max_size {
        offscreen_range = (render_target_max_size - num) / 2;
    }
    num += offscreen_range * 2;
    num2 += offscreen_range * 2;
    TargetLayout {
        layer_size: (num.max(1) as u32, num2.max(1) as u32),
        offscreen_range,
        screen_size: (backbuffer.0.max(1), backbuffer.1.max(1)),
    }
}

/// `WorldSceneLayerTarget.IsPartiallyOffscreen`
/// (`WorldSceneLayerTarget.cs:17-34`), pure. A zero position (never
/// rendered) counts as offscreen; otherwise the target's centre must sit
/// within `(targetSize - cameraScaledSize) / 2` of the camera centre on
/// BOTH axes (strict `>` on either axis = partially offscreen).
pub fn is_partially_offscreen(
    position: [f32; 2],
    target_size: [f32; 2],
    camera_center: [f32; 2],
    camera_scaled_size: [f32; 2],
) -> bool {
    if position == [0.0, 0.0] {
        return true;
    }
    let d = [
        position[0] + target_size[0] / 2.0 - camera_center[0],
        position[1] + target_size[1] / 2.0 - camera_center[1],
    ];
    let margin = [
        (target_size[0] - camera_scaled_size[0]) / 2.0,
        (target_size[1] - camera_scaled_size[1]) / 2.0,
    ];
    d[0].abs() > margin[0] || d[1].abs() > margin[1]
}

/// The temporary camera state `WorldSceneLayerTarget.UpdateContent` installs
/// while a layer renders (`WorldSceneLayerTarget.cs:49-52`): the screen size
/// shrinks to the target minus the padding, the screen position snaps to
/// `Utils.Round(center - ScreenSize/2)` (banker's rounding, `Utils.cs:112-
/// 115`) and the zoom resets to 1. The caller's own state is untouched —
/// the C# saves/restores the globals around the render callback (`:45-63`);
/// the port hands the override to the callback instead of mutating globals.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayerCamera {
    pub screen_position: [f32; 2],
    pub screen_size: [f32; 2],
    pub zoom: f32,
}

/// One world-space layer cache (`Graphics/WorldSceneLayerTarget.cs:7-70`):
/// a transparent-cleared render target plus the world position of its top-
/// left corner (`_position`, `:57`).
pub struct WorldSceneLayerTarget {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    position: [f32; 2],
}

impl WorldSceneLayerTarget {
    /// `WorldSceneLayerTarget.cs:38-41`: a `RenderTarget2D` with no mips and
    /// no depth — the port's offscreen format/usage family (`gpu.rs`).
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        label: &str,
    ) -> Self {
        let texture = offscreen_target_on(device, format, width, height, label);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            texture,
            view,
            position: [0.0, 0.0],
        }
    }

    /// `Position` (`WorldSceneLayerTarget.cs:15`).
    pub fn position(&self) -> [f32; 2] {
        self.position
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn size(&self) -> (u32, u32) {
        let s = self.texture.size();
        (s.width, s.height)
    }

    /// `IsPartiallyOffscreen` (`WorldSceneLayerTarget.cs:17-34`) against this
    /// target's size/position. `camera_scaled_size` is `Camera.ScaledSize`
    /// (`Camera.cs:14`: `UnscaledSize - GameViewMatrix.Translation * 2`).
    pub fn is_partially_offscreen(
        &self,
        camera_center: [f32; 2],
        camera_scaled_size: [f32; 2],
    ) -> bool {
        let (w, h) = self.size();
        is_partially_offscreen(
            self.position,
            [w as f32, h as f32],
            camera_center,
            camera_scaled_size,
        )
    }

    /// `UpdateContent` (`WorldSceneLayerTarget.cs:43-64`): compute the layer
    /// camera override, bind + clear the target transparent, record
    /// `_position = screenPosition - (offScreenRange, offScreenRange)` and
    /// run the render callback against the layer view. The C# restores the
    /// previous screen state afterwards (`:60-63`); the port never mutates
    /// the caller's state, so there is nothing to restore.
    pub fn update_content(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera_center: [f32; 2],
        offscreen_range: i32,
        render: impl FnOnce(&wgpu::TextureView, LayerCamera),
    ) {
        // WorldSceneLayerTarget.cs:49-52 — the size override comes FIRST,
        // the position rounds against the overridden size.
        let (w, h) = self.size();
        let osr = offscreen_range as f32;
        let screen_size = [w as f32 - osr * 2.0, h as f32 - osr * 2.0];
        let screen_position = [
            (camera_center[0] - screen_size[0] / 2.0).round_ties_even(),
            (camera_center[1] - screen_size[1] / 2.0).round_ties_even(),
        ];
        let camera = LayerCamera {
            screen_position,
            screen_size,
            zoom: 1.0,
        };
        // WorldSceneLayerTarget.cs:55-56 — SetRenderTarget + Clear(Transparent).
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("layer-clear"),
        });
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("layer-clear-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        queue.submit(Some(encoder.finish()));
        // WorldSceneLayerTarget.cs:57-58
        self.position = [screen_position[0] - osr, screen_position[1] - osr];
        render(&self.view, camera);
    }
}

/// A full-screen `RenderTarget2D` (skyTarget / screenTarget /
/// screenTargetSwap, `Main.cs:58194-58196`): backbuffer-sized, no padding,
/// no position.
pub struct ScreenTarget {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl ScreenTarget {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        label: &str,
    ) -> Self {
        let texture = offscreen_target_on(device, format, width, height, label);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self { texture, view }
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// The skyTarget bind's clear half — `GraphicsDevice.SetRenderTarget(skyTarget)`
    /// + `Clear(Color.Transparent)` (`Main.cs:62025-62026`, and the capture
    /// `BeginDrawCapture` bind, `CaptureCamera.cs:223-224`). wgpu has no global
    /// "current render target": binding is per-pass, so the port's bind is the
    /// caller passing [`view`](Self::view) to its sky pass; this records the
    /// transparent clear the C# does immediately after the bind.
    pub fn clear_transparent(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("sky-target-clear"),
        });
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("sky-target-clear-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        queue.submit(Some(encoder.finish()));
    }
}

/// The seven world-space layers, in `Main.cs:765-777` declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneLayer {
    Water,
    BackWater,
    Tile,
    Tile2,
    Wall,
    Background,
    BackgroundSwap,
}

/// One step of a `RenderToTargets` plan: re-render a layer, or apply the
/// pending background swap (`Main.cs:57313`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerUpdate {
    Render(SceneLayer),
    ApplyBackgroundSwap,
}

/// `Main.RenderToTargets` (`Main.cs:57277-57327`), pure scheduling. The
/// `renderCount` phase arms (`LightingEngine.cs:90-103` drives the count:
/// Scan→1, Blur→2, MinimapUpdate→3) refresh one layer group per lighting
/// phase; `renderNow` forces the first arm (the C#'s `else if (renderNow ||
/// ...)` at `:57287`/`:57293` is unreachable when `renderNow` is true — the
/// first arm already ran — ported as the same if/else chain). Afterwards
/// every partially-offscreen layer re-renders in the fixed order tile,
/// tile2, backWater, background (apply the pending swap first, then RE-
/// CHECK, `:57311-57318`), wall, water.
///
/// `offscreen(layer)` answers `IsPartiallyOffscreen` for the CURRENT target
/// (for `Background` the caller must re-answer after a swap — the closure
/// is called again after the plan emits `ApplyBackgroundSwap`; the pure
/// form here models that with `offscreen_after_swap`).
pub fn render_to_targets_plan(
    render: bool,
    render_now: bool,
    render_count: i32,
    mut offscreen: impl FnMut(SceneLayer) -> bool,
    offscreen_after_swap: impl FnOnce() -> bool,
) -> Vec<LayerUpdate> {
    // Main.cs:57279-57282
    if !render {
        return Vec::new();
    }
    let mut plan = Vec::new();
    // Main.cs:57283-57298 — the else-if chain, verbatim.
    if render_now || render_count == 1 {
        plan.push(LayerUpdate::Render(SceneLayer::Water));
    } else if render_now || render_count == 2 {
        plan.push(LayerUpdate::Render(SceneLayer::BackWater));
        plan.push(LayerUpdate::Render(SceneLayer::BackgroundSwap));
        // Main.cs:57291 — backgroundTargetSwapPending = true; the swap is
        // applied by RenderWallsAndBlacks (:58058) or the background
        // offscreen arm below (:57313).
    } else if render_now || render_count == 3 {
        plan.push(LayerUpdate::Render(SceneLayer::Wall));
        plan.push(LayerUpdate::Render(SceneLayer::Tile));
        plan.push(LayerUpdate::Render(SceneLayer::Tile2));
    }
    // Main.cs:57299-57326 — the partial-offscreen re-render order.
    if offscreen(SceneLayer::Tile) {
        plan.push(LayerUpdate::Render(SceneLayer::Tile));
    }
    if offscreen(SceneLayer::Tile2) {
        plan.push(LayerUpdate::Render(SceneLayer::Tile2));
    }
    if offscreen(SceneLayer::BackWater) {
        plan.push(LayerUpdate::Render(SceneLayer::BackWater));
    }
    if offscreen(SceneLayer::Background) {
        plan.push(LayerUpdate::ApplyBackgroundSwap);
        if offscreen_after_swap() {
            plan.push(LayerUpdate::Render(SceneLayer::Background));
        }
    }
    if offscreen(SceneLayer::Wall) {
        plan.push(LayerUpdate::Render(SceneLayer::Wall));
    }
    if offscreen(SceneLayer::Water) {
        plan.push(LayerUpdate::Render(SceneLayer::Water));
    }
    plan
}

/// `Main.RenderToTargets` (`Main.cs:57277-57327`), EXECUTED against a live
/// family — the consumer half of [`render_to_targets_plan`]. The C# drives
/// each arm with `layerTarget.UpdateContent(this.RenderX)`; the port hands
/// the same per-layer callbacks to [`WorldSceneLayerTarget::update_content`]
/// in the same order. `render_layer(layer, view, camera)` is the App's
/// `RenderTiles`/`RenderTiles2`/`RenderWater`/`RenderBackgroundWater`/
/// `RenderWallsAndBlacks`/`RenderBackground` dispatch (`Main.cs:57329-57348`,
/// `:57350-57369`, `:57371-57386`, `:55183-55198`, `:58056-58067`,
/// `:55200-55218`).
///
/// Two C# side effects live inside the callbacks and are reproduced here so
/// the executor stays the single ordering authority:
/// - the phase-2 arm marks the freshly rendered background swap pending
///   AFTER `backgroundTargetSwap.UpdateContent` returns (`Main.cs:57291`);
/// - `RenderWallsAndBlacks` applies the pending background swap FIRST
///   (`Main.cs:58058`) — every wall render (phase arm or offscreen re-render)
///   runs the apply. The C# applies it INSIDE the callback (after the wall
///   target's clear + position record); the port applies it just before
///   `update_content` — provably equivalent: the swap only exchanges the two
///   background targets, which the wall's clear/position/callback never read.
///
/// `camera_center` is `Main.Camera.Center` and `camera_scaled_size` is
/// `Camera.ScaledSize` (`Camera.cs:14`); the offscreen checks re-answer
/// `IsPartiallyOffscreen` against the CURRENT target each time (the
/// background re-check after the swap, `:57314`, included).
pub fn render_to_targets(
    family: &mut RenderTargetFamily,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    render: bool,
    render_now: bool,
    render_count: i32,
    camera_center: [f32; 2],
    camera_scaled_size: [f32; 2],
    mut render_layer: impl FnMut(SceneLayer, &wgpu::TextureView, LayerCamera),
) {
    // Main.cs:57279-57282
    if !render {
        return;
    }
    let osr = family.layout.offscreen_range;
    let offscreen = |f: &RenderTargetFamily, layer: SceneLayer| {
        f.layer(layer)
            .is_partially_offscreen(camera_center, camera_scaled_size)
    };
    // One `layerTarget.UpdateContent(callback)` — clear + position record +
    // the render callback against the layer view.
    let mut update = |f: &mut RenderTargetFamily, layer: SceneLayer| {
        f.layer_mut(layer)
            .update_content(device, queue, camera_center, osr, |view, cam| {
                render_layer(layer, view, cam);
            });
    };
    // Main.cs:57283-57298 — the else-if chain, verbatim (the `renderNow`
    // re-tests are unreachable when the first arm ran; kept like the C#).
    if render_now || render_count == 1 {
        update(family, SceneLayer::Water);
    } else if render_now || render_count == 2 {
        update(family, SceneLayer::BackWater);
        update(family, SceneLayer::BackgroundSwap);
        // Main.cs:57291 — backgroundTargetSwapPending = true.
        family.mark_background_swap_pending();
    } else if render_now || render_count == 3 {
        // Main.cs:58058 — RenderWallsAndBlacks applies the pending swap first.
        family.apply_pending_background_target_swap();
        update(family, SceneLayer::Wall);
        update(family, SceneLayer::Tile);
        update(family, SceneLayer::Tile2);
    }
    // Main.cs:57299-57326 — the partial-offscreen re-render order.
    if offscreen(family, SceneLayer::Tile) {
        update(family, SceneLayer::Tile);
    }
    if offscreen(family, SceneLayer::Tile2) {
        update(family, SceneLayer::Tile2);
    }
    if offscreen(family, SceneLayer::BackWater) {
        update(family, SceneLayer::BackWater);
    }
    if offscreen(family, SceneLayer::Background) {
        // Main.cs:57313 — ApplyPendingBackgroundTargetSwap, then RE-CHECK.
        family.apply_pending_background_target_swap();
        if offscreen(family, SceneLayer::Background) {
            update(family, SceneLayer::Background);
        }
    }
    if offscreen(family, SceneLayer::Wall) {
        // Main.cs:58058 again — the callback's own apply-before-render.
        family.apply_pending_background_target_swap();
        update(family, SceneLayer::Wall);
    }
    if offscreen(family, SceneLayer::Water) {
        update(family, SceneLayer::Water);
    }
}

/// The layer composite destination math — the six `Main.DoDraw` sites that
/// draw a cached layer onto the screen: `backWaterTarget` (`Main.cs:62196`),
/// `backgroundTarget` (`:62213-62214`, the cave-parallax X shift), `waterTarget`
/// (`:62308`), `tileTarget` (`:63333`), `tile2Target` (`:63367`) and
/// `wallTarget` (`:63447`). Every site is
/// `spriteBatch.Draw(target.Texture, target.Position - Main.screenPosition
/// [+ parallax], Color.White)`; the background alone adds
/// `(Position + Texture.Size()/2 - Camera.Center) * (caveParallax - 1, 0)`
/// (`:62213`). Returns the world-px top-left the full-size layer texture is
/// drawn at (the caller subtracts nothing — `screen_position` is already
/// `Main.screenPosition`).
pub fn layer_composite_position(
    layer: SceneLayer,
    position: [f32; 2],
    layer_size: (u32, u32),
    screen_position: [f32; 2],
    camera_center: [f32; 2],
    cave_parallax: f32,
) -> [f32; 2] {
    let mut p = [
        position[0] - screen_position[0],
        position[1] - screen_position[1],
    ];
    if layer == SceneLayer::Background {
        // Main.cs:62213 — the X-only parallax shift about the target centre.
        p[0] +=
            (position[0] + layer_size.0 as f32 / 2.0 - camera_center[0]) * (cave_parallax - 1.0);
    }
    p
}

/// The skyTarget capture gate (`Main.cs:62021-62027`): vanilla binds and
/// clears `skyTarget` only when a scene filter can capture (or the
/// foreground sunlight effects need it) AND the frame uses the target
/// family (`!drawToScreen`), is not the fullscreen map / fancy-UI-only, and
/// the lighting mode is not Retro. `flag2`/`flag3` at `:62021-62022`
/// (the C# `&` is non-short-circuiting; same truth value).
pub fn sky_target_active(
    can_capture: bool,
    foreground_sunlight_effects: bool,
    draw_to_screen: bool,
    map_fullscreen: bool,
    only_draw_fancy_ui: bool,
    lighting_not_retro: bool,
) -> bool {
    let flag2 = can_capture || foreground_sunlight_effects;
    (!draw_to_screen && !map_fullscreen && !only_draw_fancy_ui && lighting_not_retro) && flag2
}

/// What `DoDraw` does with the family this frame (`Main.cs:61751-61766`):
/// `Lighting.UpdateEveryFrame` forces `drawToScreen`; leaving the target
/// path releases the family, entering it initialises.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetTransition {
    /// `drawToScreen && targetSet` → `ReleaseTargets` (`Main.cs:61759-61762`).
    Release,
    /// `!drawToScreen && !targetSet` → `InitTargets` (`Main.cs:61763-61766`).
    Init,
    Keep,
}

pub fn target_transition(draw_to_screen: bool, target_set: bool) -> TargetTransition {
    if draw_to_screen && target_set {
        TargetTransition::Release
    } else if !draw_to_screen && !target_set {
        TargetTransition::Init
    } else {
        TargetTransition::Keep
    }
}

/// The whole family: seven world layers + three full-screen targets +
/// the pending background swap flag (`Main.cs:765-785`).
pub struct RenderTargetFamily {
    pub layout: TargetLayout,
    pub water: WorldSceneLayerTarget,
    pub back_water: WorldSceneLayerTarget,
    pub tile: WorldSceneLayerTarget,
    pub tile2: WorldSceneLayerTarget,
    pub wall: WorldSceneLayerTarget,
    pub background: WorldSceneLayerTarget,
    pub background_swap: WorldSceneLayerTarget,
    pub sky: ScreenTarget,
    pub screen: ScreenTarget,
    pub screen_swap: ScreenTarget,
    /// `Main.backgroundTargetSwapPending` (`Main.cs:779`).
    background_swap_pending: bool,
}

impl RenderTargetFamily {
    /// `Main.InitTargets` (`Main.cs:58160-58216`): compute the layout, then
    /// allocate all ten targets in the C#'s creation order (`:58187-58196`).
    /// Deviation (ANALYSIS): the C#'s catch arm falls back to Retro lighting
    /// + map off (`:58203-58215`); wgpu texture creation aborts on failure,
    /// so the fallback is not ported. `ReleaseTargets` (`:58093-58150`) is
    /// Rust `Drop` — dropping the family disposes every target.
    pub fn init(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        backbuffer: (u32, u32),
        hidef: bool,
        support_wide_screen: bool,
        render_target_max_size: i32,
    ) -> Self {
        let layout = init_target_layout(
            backbuffer,
            hidef,
            support_wide_screen,
            render_target_max_size,
        );
        let (lw, lh) = layout.layer_size;
        let (sw, sh) = layout.screen_size;
        let layer = |label: &str| WorldSceneLayerTarget::new(device, format, lw, lh, label);
        Self {
            layout,
            water: layer("waterTarget"),
            back_water: layer("backWaterTarget"),
            tile: layer("tileTarget"),
            tile2: layer("tile2Target"),
            wall: layer("wallTarget"),
            background: layer("backgroundTarget"),
            background_swap: layer("backgroundTargetSwap"),
            sky: ScreenTarget::new(device, format, sw, sh, "skyTarget"),
            screen: ScreenTarget::new(device, format, sw, sh, "screenTarget"),
            screen_swap: ScreenTarget::new(device, format, sw, sh, "screenTargetSwap"),
            background_swap_pending: false,
        }
    }

    /// `Main.EnsureRenderTargetContent` (`Main.cs:58152-58158`) for an
    /// `Option`-held family: re-init when absent or when the backbuffer
    /// size changed. wgpu has no `IsContentLost` (device loss recreates
    /// everything), so the null check + size check is the port's equivalent
    /// (ANALYSIS).
    pub fn ensure(
        slot: &mut Option<Self>,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        backbuffer: (u32, u32),
        hidef: bool,
        support_wide_screen: bool,
        render_target_max_size: i32,
    ) {
        let stale = match slot {
            Some(f) => f.layout.screen_size != (backbuffer.0.max(1), backbuffer.1.max(1)),
            None => true,
        };
        if stale {
            *slot = Some(Self::init(
                device,
                format,
                backbuffer,
                hidef,
                support_wide_screen,
                render_target_max_size,
            ));
        }
    }

    /// `Main.ApplyPendingBackgroundTargetSwap` (`Main.cs:58047-58054`):
    /// `Utils.Swap(ref backgroundTarget, ref backgroundTargetSwap)` and
    /// clear the flag.
    pub fn apply_pending_background_target_swap(&mut self) {
        if self.background_swap_pending {
            std::mem::swap(&mut self.background, &mut self.background_swap);
            self.background_swap_pending = false;
        }
    }

    /// `Main.cs:57291` — the phase-2 arm marks the freshly rendered
    /// background swap target pending.
    pub fn mark_background_swap_pending(&mut self) {
        self.background_swap_pending = true;
    }

    pub fn background_swap_pending(&self) -> bool {
        self.background_swap_pending
    }

    pub fn layer(&self, layer: SceneLayer) -> &WorldSceneLayerTarget {
        match layer {
            SceneLayer::Water => &self.water,
            SceneLayer::BackWater => &self.back_water,
            SceneLayer::Tile => &self.tile,
            SceneLayer::Tile2 => &self.tile2,
            SceneLayer::Wall => &self.wall,
            SceneLayer::Background => &self.background,
            SceneLayer::BackgroundSwap => &self.background_swap,
        }
    }

    pub fn layer_mut(&mut self, layer: SceneLayer) -> &mut WorldSceneLayerTarget {
        match layer {
            SceneLayer::Water => &mut self.water,
            SceneLayer::BackWater => &mut self.back_water,
            SceneLayer::Tile => &mut self.tile,
            SceneLayer::Tile2 => &mut self.tile2,
            SceneLayer::Wall => &mut self.wall,
            SceneLayer::Background => &mut self.background,
            SceneLayer::BackgroundSwap => &mut self.background_swap,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- InitTargets layout math (Main.cs:58167-58181) ----

    #[test]
    fn layout_1080p_hidef_pads_by_192() {
        // num = min(1920,1920) = 1920; 1920+384 = 2304 <= 4096 → osr 192.
        let l = init_target_layout((1920, 1080), true, false, RENDER_TARGET_MAX_HIDEF);
        assert_eq!(l.layer_size, (2304, 1464));
        assert_eq!(l.offscreen_range, 192);
        assert_eq!(l.screen_size, (1920, 1080));
    }

    #[test]
    fn layout_reach_shrinks_the_offscreen_range() {
        // Main.cs:58176-58179: 1920+384 > 2048 → osr = (2048-1920)/2 = 64.
        let l = init_target_layout((1920, 1080), false, false, RENDER_TARGET_MAX_REACH);
        assert_eq!(l.offscreen_range, 64);
        assert_eq!(l.layer_size, (2048, 1208));
    }

    #[test]
    fn layout_small_window_keeps_192() {
        let l = init_target_layout((1280, 720), true, false, RENDER_TARGET_MAX_HIDEF);
        assert_eq!(l.layer_size, (1664, 1104));
        assert_eq!(l.offscreen_range, 192);
    }

    #[test]
    fn layout_clamps_both_axes_with_the_x_cap() {
        // Main.cs:58167-58174: val = MaxWorldViewSize.X clamps HEIGHT too —
        // a 2160-tall backbuffer clamps to 1920 without wide-screen.
        let l = init_target_layout((3840, 2160), true, false, RENDER_TARGET_MAX_HIDEF);
        assert_eq!(l.layer_size, (1920 + 384, 1920 + 384));
        assert_eq!(l.screen_size, (3840, 2160));
    }

    #[test]
    fn layout_wide_hidef_raises_the_cap_to_4096() {
        // Main.cs:58169-58172: SupportWideScreen && HiDef → val = 4096;
        // 3840+384 > 4096 → osr = (4096-3840)/2 = 128.
        let l = init_target_layout((3840, 2160), true, true, RENDER_TARGET_MAX_HIDEF);
        assert_eq!(l.offscreen_range, 128);
        assert_eq!(l.layer_size, (4096, 2416));
        // Wide-screen without HiDef keeps the 1920 cap (:58169).
        let r = init_target_layout((3840, 2160), false, true, RENDER_TARGET_MAX_REACH);
        assert_eq!(r.layer_size, (2048, 2048));
    }

    // ---- IsPartiallyOffscreen (WorldSceneLayerTarget.cs:17-34) ----

    #[test]
    fn zero_position_is_always_offscreen() {
        assert!(is_partially_offscreen(
            [0.0, 0.0],
            [2304.0, 1464.0],
            [1152.0, 732.0],
            [1920.0, 1080.0]
        ));
    }

    #[test]
    fn centred_target_is_onscreen_and_the_boundary_is_strict() {
        // position = centre - size/2 → d = 0; margin = (2304-1920)/2 = 192.
        let pos = [100.0, 100.0];
        let size = [2304.0, 1464.0];
        let centre = [pos[0] + size[0] / 2.0, pos[1] + size[1] / 2.0];
        let scaled = [1920.0, 1080.0];
        assert!(!is_partially_offscreen(pos, size, centre, scaled));
        // Exactly at the margin: `>` is strict (WorldSceneLayerTarget.cs:28-30).
        assert!(!is_partially_offscreen(
            pos,
            size,
            [centre[0] + 192.0, centre[1]],
            scaled
        ));
        // One texel past it: offscreen.
        assert!(is_partially_offscreen(
            pos,
            size,
            [centre[0] + 193.0, centre[1]],
            scaled
        ));
        // Y axis alone triggers too (:30).
        assert!(is_partially_offscreen(
            pos,
            size,
            [centre[0], centre[1] + 193.0],
            scaled
        ));
    }

    // ---- UpdateContent camera override (WorldSceneLayerTarget.cs:43-64) ----

    #[test]
    fn layer_camera_rounds_bankers_and_resets_zoom() {
        // Utils.Round is Math.Round — banker's rounding (Utils.cs:112-115).
        // screen = 2304x1464, osr 192 → override size 1920x1080;
        // centre (1000.5, 2000.5) - (960, 540) = (40.5, 1460.5)
        // → round-ties-even (40, 1460); position = (40-192, 1460-192).
        let centre = [1000.5, 2000.5];
        let size = [2304.0_f32, 1464.0];
        let osr = 192.0;
        let screen_size = [size[0] - osr * 2.0, size[1] - osr * 2.0];
        let sp = [
            (centre[0] - screen_size[0] / 2.0).round_ties_even(),
            (centre[1] - screen_size[1] / 2.0).round_ties_even(),
        ];
        assert_eq!(sp, [40.0, 1460.0]);
        assert_eq!([sp[0] - osr, sp[1] - osr], [-152.0, 1268.0]);
    }

    // ---- RenderToTargets scheduling (Main.cs:57277-57327) ----

    fn plan(count: i32, now: bool, off: &[SceneLayer]) -> Vec<LayerUpdate> {
        render_to_targets_plan(
            true,
            now,
            count,
            |l| off.contains(&l),
            || off.contains(&SceneLayer::Background),
        )
    }

    #[test]
    fn phase_arms_refresh_one_group_each() {
        use LayerUpdate::Render as R;
        assert_eq!(plan(1, false, &[]), vec![R(SceneLayer::Water)]);
        assert_eq!(
            plan(2, false, &[]),
            vec![R(SceneLayer::BackWater), R(SceneLayer::BackgroundSwap)]
        );
        assert_eq!(
            plan(3, false, &[]),
            vec![
                R(SceneLayer::Wall),
                R(SceneLayer::Tile),
                R(SceneLayer::Tile2)
            ]
        );
        // renderCount 99 (Main.cs:56728/61844): no phase arm fires.
        assert_eq!(plan(99, false, &[]), vec![]);
        // renderNow takes the FIRST arm only — the else-if chain (:57283-57298).
        assert_eq!(plan(2, true, &[]), vec![R(SceneLayer::Water)]);
    }

    #[test]
    fn render_false_skips_everything() {
        // Main.cs:57279-57282.
        assert!(render_to_targets_plan(false, true, 1, |_| true, || true).is_empty());
    }

    #[test]
    fn partial_offscreen_rerenders_in_the_fixed_order() {
        use LayerUpdate::Render as R;
        // All layers offscreen, phase 3: the phase group first, then the
        // offscreen order tile, tile2, backWater, background(swap+recheck),
        // wall, water (Main.cs:57299-57326).
        let all = [
            SceneLayer::Water,
            SceneLayer::BackWater,
            SceneLayer::Tile,
            SceneLayer::Tile2,
            SceneLayer::Wall,
            SceneLayer::Background,
        ];
        assert_eq!(
            plan(3, false, &all),
            vec![
                R(SceneLayer::Wall),
                R(SceneLayer::Tile),
                R(SceneLayer::Tile2),
                R(SceneLayer::Tile),
                R(SceneLayer::Tile2),
                R(SceneLayer::BackWater),
                LayerUpdate::ApplyBackgroundSwap,
                R(SceneLayer::Background),
                R(SceneLayer::Wall),
                R(SceneLayer::Water),
            ]
        );
    }

    #[test]
    fn background_swap_can_cancel_the_rerender() {
        // Main.cs:57311-57318: the swap applies first; if the swapped-in
        // target is NOT offscreen the re-render is skipped.
        let p = render_to_targets_plan(true, false, 99, |l| l == SceneLayer::Background, || false);
        assert_eq!(p, vec![LayerUpdate::ApplyBackgroundSwap]);
    }

    // ---- The skyTarget gate (Main.cs:62021-62027) ----

    #[test]
    fn sky_gate_needs_capture_and_the_target_path() {
        // can_capture, fg_sunlight, drawToScreen, mapFullscreen, fancyUI, notRetro
        assert!(sky_target_active(true, false, false, false, false, true));
        assert!(sky_target_active(false, true, false, false, false, true));
        // No capturable filter and no sunlight effects → no skyTarget.
        assert!(!sky_target_active(false, false, false, false, false, true));
        // Each veto alone kills it (:62022).
        assert!(!sky_target_active(true, false, true, false, false, true));
        assert!(!sky_target_active(true, false, false, true, false, true));
        assert!(!sky_target_active(true, false, false, false, true, true));
        assert!(!sky_target_active(true, false, false, false, false, false));
    }

    // ---- The drawToScreen/targetSet transition (Main.cs:61751-61766) ----

    #[test]
    fn target_transition_matches_the_dodraw_arms() {
        assert_eq!(target_transition(true, true), TargetTransition::Release);
        assert_eq!(target_transition(false, false), TargetTransition::Init);
        assert_eq!(target_transition(true, false), TargetTransition::Keep);
        assert_eq!(target_transition(false, true), TargetTransition::Keep);
    }

    // ---- The layer composite math (Main.cs:62196/62213/62308/63333/63367/63447) ----

    #[test]
    fn composite_position_is_position_minus_screen_position() {
        // Main.cs:62196/62308/63333/63367/63447: every non-background layer
        // draws at `Position - screenPosition`, no parallax term.
        for layer in [
            SceneLayer::Water,
            SceneLayer::BackWater,
            SceneLayer::Tile,
            SceneLayer::Tile2,
            SceneLayer::Wall,
        ] {
            assert_eq!(
                layer_composite_position(
                    layer,
                    [100.0, 200.0],
                    (2304, 1464),
                    [40.0, 60.0],
                    [1000.0, 800.0],
                    0.5,
                ),
                [60.0, 140.0],
                "{layer:?}"
            );
        }
    }

    #[test]
    fn background_composite_adds_the_cave_parallax_x_shift() {
        // Main.cs:62213: vector2 = (Position + Size/2 - Camera.Center) *
        // (caveParallax - 1, 0). Position (100,200), size 2304x1464 → centre
        // (1252,932); camera centre (1000,800) → delta (252,132); parallax
        // 0.5 → shift (252 * -0.5, 0) = (-126, 0). Base (60,140) → (-66,140).
        assert_eq!(
            layer_composite_position(
                SceneLayer::Background,
                [100.0, 200.0],
                (2304, 1464),
                [40.0, 60.0],
                [1000.0, 800.0],
                0.5,
            ),
            [-66.0, 140.0]
        );
        // caveParallax == 1 → the shift vanishes (the surface case).
        assert_eq!(
            layer_composite_position(
                SceneLayer::Background,
                [100.0, 200.0],
                (2304, 1464),
                [40.0, 60.0],
                [1000.0, 800.0],
                1.0,
            ),
            [60.0, 140.0]
        );
    }

    // ---- GPU layout tests (headless; skip without an adapter) ----

    #[test]
    fn gpu_family_sizes_match_the_layout() {
        let Some((device, _queue)) = crate::test_gpu::headless() else {
            return;
        };
        let format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let f = RenderTargetFamily::init(
            &device,
            format,
            (1280, 720),
            true,
            false,
            RENDER_TARGET_MAX_HIDEF,
        );
        // InitTargets: every world layer is the padded size (:58187-58193),
        // every screen target the raw backbuffer (:58194-58196).
        for layer in [
            SceneLayer::Water,
            SceneLayer::BackWater,
            SceneLayer::Tile,
            SceneLayer::Tile2,
            SceneLayer::Wall,
            SceneLayer::Background,
            SceneLayer::BackgroundSwap,
        ] {
            assert_eq!(f.layer(layer).size(), (1664, 1104), "{layer:?}");
            assert_eq!(f.layer(layer).position(), [0.0, 0.0]);
            // A never-rendered layer is partially offscreen (:21-24).
            assert!(f
                .layer(layer)
                .is_partially_offscreen([832.0, 552.0], [1280.0, 720.0]));
        }
        for t in [&f.sky, &f.screen, &f.screen_swap] {
            let s = t.texture().size();
            assert_eq!((s.width, s.height), (1280, 720));
        }
    }

    #[test]
    fn gpu_update_content_positions_and_clears() {
        let Some((device, queue)) = crate::test_gpu::headless() else {
            return;
        };
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let mut layer = WorldSceneLayerTarget::new(&device, format, 1664, 1104, "test-layer");
        let mut seen = None;
        layer.update_content(&device, &queue, [832.5, 552.5], 192, |_view, cam| {
            seen = Some(cam);
        });
        // Override size 1280x720; centre - size/2 = (192.5, 192.5) →
        // banker's round (192, 192); position = (192-192, 192-192) = (0, 0)
        // — the zero-position quirk (IsPartiallyOffscreen :21-24) applies.
        let cam = seen.expect("callback ran");
        assert_eq!(cam.screen_size, [1280.0, 720.0]);
        assert_eq!(cam.zoom, 1.0);
        assert_eq!(cam.screen_position, [192.0, 192.0]);
        assert_eq!(layer.position(), [0.0, 0.0]);
        // The clear ran: read the top-left texel back — transparent.
        let px = crate::test_gpu::read_texture(&device, &queue, layer.texture(), (1, 1), 4);
        assert_eq!(px, vec![0, 0, 0, 0]);
        // A second update at a shifted centre records the new position.
        layer.update_content(&device, &queue, [900.0, 600.0], 192, |_, _| {});
        assert_eq!(layer.position(), [260.0 - 192.0, 240.0 - 192.0]);
    }

    #[test]
    fn gpu_background_swap_exchanges_the_targets() {
        let Some((device, queue)) = crate::test_gpu::headless() else {
            return;
        };
        let format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let mut f = RenderTargetFamily::init(
            &device,
            format,
            (1280, 720),
            true,
            false,
            RENDER_TARGET_MAX_HIDEF,
        );
        // Mark the two targets with distinct positions (the layer's own
        // identity marker — `_position`, WorldSceneLayerTarget.cs:57).
        f.background
            .update_content(&device, &queue, [900.0, 600.0], 192, |_, _| {});
        f.background_swap
            .update_content(&device, &queue, [1000.0, 700.0], 192, |_, _| {});
        let bg = f.background.position();
        let swap = f.background_swap.position();
        assert_ne!(bg, swap);
        // No pending flag → no swap (Main.cs:58049).
        f.apply_pending_background_target_swap();
        assert_eq!(f.background.position(), bg);
        // Phase-2 arm marks it pending (:57291), then the swap exchanges
        // and clears the flag (:58047-58054).
        f.mark_background_swap_pending();
        f.apply_pending_background_target_swap();
        assert_eq!(f.background.position(), swap);
        assert_eq!(f.background_swap.position(), bg);
        assert!(!f.background_swap_pending());
        // Idempotent once cleared.
        f.apply_pending_background_target_swap();
        assert_eq!(f.background.position(), swap);
    }

    #[test]
    fn gpu_ensure_reinits_only_on_size_change() {
        let Some((device, queue)) = crate::test_gpu::headless() else {
            return;
        };
        let format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let mut slot: Option<RenderTargetFamily> = None;
        RenderTargetFamily::ensure(
            &mut slot,
            &device,
            format,
            (1280, 720),
            true,
            false,
            RENDER_TARGET_MAX_HIDEF,
        );
        // Mark the family with a layer position; a kept family preserves it.
        slot.as_mut().unwrap().water.update_content(
            &device,
            &queue,
            [900.0, 600.0],
            192,
            |_, _| {},
        );
        let marker = slot.as_ref().unwrap().water.position();
        // Same size → kept (EnsureRenderTargetContent's null/lost check).
        RenderTargetFamily::ensure(
            &mut slot,
            &device,
            format,
            (1280, 720),
            true,
            false,
            RENDER_TARGET_MAX_HIDEF,
        );
        assert_eq!(slot.as_ref().unwrap().water.position(), marker);
        // Resize → re-init (fresh targets, zero positions).
        RenderTargetFamily::ensure(
            &mut slot,
            &device,
            format,
            (1600, 900),
            true,
            false,
            RENDER_TARGET_MAX_HIDEF,
        );
        let f = slot.as_ref().unwrap();
        assert_eq!(f.water.position(), [0.0, 0.0]);
        assert_eq!(f.layout.screen_size, (1600, 900));
        assert_eq!(f.layout.layer_size, (1984, 1284));
    }

    // ---- The RenderToTargets executor (Main.cs:57277-57327), GPU ----

    /// A family whose every layer sits far from the camera (all partially
    /// offscreen), so the offscreen arms all fire.
    fn offscreen_family(device: &wgpu::Device, queue: &wgpu::Queue) -> RenderTargetFamily {
        let mut f = RenderTargetFamily::init(
            device,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            (1280, 720),
            true,
            false,
            RENDER_TARGET_MAX_HIDEF,
        );
        // Park every layer at a position far from the test camera centre so
        // IsPartiallyOffscreen is true for all seven.
        for layer in [
            SceneLayer::Water,
            SceneLayer::BackWater,
            SceneLayer::Tile,
            SceneLayer::Tile2,
            SceneLayer::Wall,
            SceneLayer::Background,
            SceneLayer::BackgroundSwap,
        ] {
            f.layer_mut(layer)
                .update_content(device, queue, [100000.0, 100000.0], 192, |_, _| {});
        }
        f
    }

    #[test]
    fn gpu_executor_phase_arms_render_the_right_layers() {
        let Some((device, queue)) = crate::test_gpu::headless() else {
            return;
        };
        // Phase 1 → water only; the camera sits ON the parked layers so no
        // offscreen arm fires (centre 100000,100000 matches their position).
        let mut f = offscreen_family(&device, &queue);
        let mut seen = Vec::new();
        render_to_targets(
            &mut f,
            &device,
            &queue,
            true,
            false,
            1,
            [100000.0, 100000.0],
            [1280.0, 720.0],
            |layer, _view, _cam| seen.push(layer),
        );
        assert_eq!(seen, vec![SceneLayer::Water]);
        // Phase 2 → backWater + backgroundSwap, and the pending flag arms.
        let mut f = offscreen_family(&device, &queue);
        let mut seen = Vec::new();
        render_to_targets(
            &mut f,
            &device,
            &queue,
            true,
            false,
            2,
            [100000.0, 100000.0],
            [1280.0, 720.0],
            |layer, _view, _cam| seen.push(layer),
        );
        assert_eq!(
            seen,
            vec![SceneLayer::BackWater, SceneLayer::BackgroundSwap]
        );
        assert!(f.background_swap_pending());
        // Phase 3 → wall + tile + tile2.
        let mut f = offscreen_family(&device, &queue);
        let mut seen = Vec::new();
        render_to_targets(
            &mut f,
            &device,
            &queue,
            true,
            false,
            3,
            [100000.0, 100000.0],
            [1280.0, 720.0],
            |layer, _view, _cam| seen.push(layer),
        );
        assert_eq!(
            seen,
            vec![SceneLayer::Wall, SceneLayer::Tile, SceneLayer::Tile2]
        );
    }

    #[test]
    fn gpu_executor_render_false_is_a_noop() {
        let Some((device, queue)) = crate::test_gpu::headless() else {
            return;
        };
        let mut f = offscreen_family(&device, &queue);
        let before = f.water.position();
        let mut seen = Vec::new();
        render_to_targets(
            &mut f,
            &device,
            &queue,
            false,
            true,
            1,
            [0.0, 0.0],
            [1280.0, 720.0],
            |layer, _view, _cam| seen.push(layer),
        );
        assert!(seen.is_empty());
        assert_eq!(f.water.position(), before);
    }

    #[test]
    fn gpu_executor_offscreen_order_and_background_swap_recheck() {
        let Some((device, queue)) = crate::test_gpu::headless() else {
            return;
        };
        // All layers offscreen, renderCount 99 (no phase arm). The order is
        // tile, tile2, backWater, background(swap+recheck), wall, water.
        let mut f = offscreen_family(&device, &queue);
        // Arm a pending swap so the background arm's apply exchanges targets.
        f.mark_background_swap_pending();
        let bg_before = f.background.position();
        let swap_before = f.background_swap.position();
        let mut seen = Vec::new();
        render_to_targets(
            &mut f,
            &device,
            &queue,
            true,
            false,
            99,
            [0.0, 0.0],
            [1280.0, 720.0],
            |layer, _view, _cam| seen.push(layer),
        );
        assert_eq!(
            seen,
            vec![
                SceneLayer::Tile,
                SceneLayer::Tile2,
                SceneLayer::BackWater,
                SceneLayer::Background,
                SceneLayer::Wall,
                SceneLayer::Water,
            ]
        );
        // The pending swap applied, then Main.cs:57313 re-checked and
        // re-rendered the swapped-in background (it was still offscreen), so
        // its position is the render position - the same one water gets from
        // the identical camera/osr (proving the re-render happened).
        assert_eq!(f.background.position(), f.water.position());
        assert_ne!(f.background.position(), swap_before);
        assert!(!f.background_swap_pending());
        // (bg_before is now in background_swap, which is not re-rendered.)
        assert_eq!(f.background_swap.position(), bg_before);
    }

    #[test]
    fn gpu_executor_wall_render_applies_the_pending_swap() {
        let Some((device, queue)) = crate::test_gpu::headless() else {
            return;
        };
        // RenderWallsAndBlacks applies the pending swap FIRST (Main.cs:58058).
        // Phase 3 with a pending swap: the wall render exchanges the targets.
        let mut f = offscreen_family(&device, &queue);
        f.mark_background_swap_pending();
        let bg_before = f.background.position();
        let swap_before = f.background_swap.position();
        render_to_targets(
            &mut f,
            &device,
            &queue,
            true,
            false,
            3,
            [100000.0, 100000.0],
            [1280.0, 720.0],
            |_layer, _view, _cam| {},
        );
        assert_eq!(f.background.position(), swap_before);
        assert_eq!(f.background_swap.position(), bg_before);
        assert!(!f.background_swap_pending());
    }

    #[test]
    fn gpu_sky_target_clear_reads_back_transparent() {
        let Some((device, queue)) = crate::test_gpu::headless() else {
            return;
        };
        let sky = ScreenTarget::new(
            &device,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            64,
            64,
            "sky-test",
        );
        sky.clear_transparent(&device, &queue);
        let px = crate::test_gpu::read_texture(&device, &queue, sky.texture(), (1, 1), 4);
        assert_eq!(px, vec![0, 0, 0, 0]);
    }
}
