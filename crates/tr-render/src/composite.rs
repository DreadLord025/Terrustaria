//! The layer-target COMPOSITE half (S1-G-13 consumers): drawing a cached
//! render target through the sprite pipeline — the six `Main.DoDraw`
//! composite sites' `spriteBatch.Draw(target.Texture, Position -
//! screenPosition [+ parallax], Color.White)` (`Main.cs:62196`/`:62213`/
//! `:62308`/`:63333`/`:63367`/`:63447`) and `FilterManager.EndCapture`'s
//! skyTarget/screenTarget merge (`FilterManager.cs:148-151`).
//!
//! The C# draws a `RenderTarget2D` like any other texture through the same
//! `SpriteBatch`; the port's sprite pipeline samples ONE atlas, so a target
//! composite rides the map-texture precedent (`ensure_map_texture`): a bind
//! group over the target's view + its own globals buffer (the target size),
//! drawn through the same pipelines with one full-size instance. The bind
//! group is rebuilt per draw — the post pipeline rebuilds per pass for the
//! same reason (`post.rs::apply_filters_on`), and the C# switches the bound
//! texture per draw too.
//!
//! The consumers ride `TR_LAYER_TARGETS` in the App (ANALYSIS "The
//! render-target family (S1-G-13)"); this module is pure mechanism — no
//! frame path changes by its existence.

use crate::gpu::Gpu;
use crate::sprite::{Globals, SpriteInstance, SpriteRenderer};

/// The target-composite state: the one-instance scratch buffer (uv 0..1,
/// white, no rot/paint/dye, atlas layer, no clip, alpha blend baked in; only
/// the destination pos/size move per draw).
pub struct TargetCompositor {
    instance_buf: wgpu::Buffer,
}

impl TargetCompositor {
    pub fn new(device: &wgpu::Device) -> Self {
        use wgpu::util::DeviceExt;
        let instance_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("target-composite-instance"),
            contents: bytemuck::bytes_of(&SpriteInstance {
                pos: [0.0, 0.0],
                size: [0.0, 0.0],
                uv0: [0.0, 0.0],
                uv1: [1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                rot: 0.0,
                paint: 0.0,
                dye: 0,
                layer: 0,
                clip: crate::clip::NO_CLIP,
                blend: 0,
            }),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        Self { instance_buf }
    }
}

impl SpriteRenderer {
    /// `spriteBatch.Draw(target.Texture, position, Color.White)` — one
    /// full-size quad sampling `view`, drawn into `dst` with `LoadOp::Load`
    /// (the composite sites draw OVER the frame, `Main.cs:62196` etc.).
    ///
    /// `position` is the site's destination (the caller computes it with
    /// [`crate::targets::layer_composite_position`]); `camera`/`zoom` are the
    /// frame's world transform (`Main.Transform`), `screen` the backbuffer.
    /// The target's own size drives the quad and the uv half-texel inset.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_target(
        &mut self,
        gpu: &Gpu,
        dst: &wgpu::TextureView,
        view: &wgpu::TextureView,
        target_size: (u32, u32),
        position: [f32; 2],
        camera: [f32; 2],
        zoom: f32,
        screen: [f32; 2],
        time: f32,
    ) {
        let mut compositor = TargetCompositor::new(&gpu.device);
        self.draw_target_with(
            &mut compositor,
            gpu,
            dst,
            view,
            target_size,
            position,
            camera,
            zoom,
            screen,
            time,
            wgpu::LoadOp::Load,
        );
    }

    /// [`Self::draw_target`] with an explicit load op and a caller-owned
    /// compositor (the frame keeps one across the six sites; the capture
    /// downscale clears its target first — `CaptureCamera.cs:196-197`).
    #[allow(clippy::too_many_arguments)]
    pub fn draw_target_with(
        &mut self,
        compositor: &mut TargetCompositor,
        gpu: &Gpu,
        dst: &wgpu::TextureView,
        view: &wgpu::TextureView,
        target_size: (u32, u32),
        position: [f32; 2],
        camera: [f32; 2],
        zoom: f32,
        screen: [f32; 2],
        time: f32,
        load: wgpu::LoadOp<wgpu::Color>,
    ) {
        let device = &gpu.device;
        // The globals buffer carries the TARGET size so the shader's
        // half-texel inset matches the sampled texture, not the atlas.
        let globals = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("target-composite-globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind = Self::mk_bind(
            device,
            &self.bgl,
            &self.sampler,
            &globals,
            view,
            &self.dye_table,
            &self.font_view,
            "bg-target",
        );
        gpu.queue.write_buffer(
            &globals,
            0,
            bytemuck::bytes_of(&Globals {
                screen,
                cam: camera,
                zoom,
                time,
                aberration: 0.0,
                _pad: 0.0,
                texel: [
                    1.0 / target_size.0.max(1) as f32,
                    1.0 / target_size.1.max(1) as f32,
                ],
            }),
        );
        // Only the instance's pos/size head moves; the baked tail (uv,
        // white, layer 0, no clip, alpha blend) stays.
        gpu.queue.write_buffer(
            &compositor.instance_buf,
            0,
            bytemuck::bytes_of(&[
                position[0],
                position[1],
                target_size.0 as f32,
                target_size.1 as f32,
            ]),
        );
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("target-composite"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("target-composite-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: dst,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipelines[0]);
            pass.set_bind_group(0, &bind, &[]);
            pass.set_vertex_buffer(0, compositor.instance_buf.slice(..));
            pass.draw(0..6, 0..1);
        }
        gpu.queue.submit(Some(encoder.finish()));
    }
}