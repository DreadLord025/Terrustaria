//! The water coverage mask for the WaterDistortion pass (M1.4-BG-18 phase 4).
//!
//! Vanilla's `WaterShaderData` binds `Main.waterTarget` — the liquid layer drawn
//! into its own render target by `Main.RenderWater` — as the distortion pass's
//! second image (`WaterShaderData.Apply`, `WaterShaderData.cs:371`), and the
//! pass distorts where that target has water. Plan assumption A3 was "the mask
//! is the liquid pass redirected into a target"; the sprite renderer has no
//! alternate-list render and its liquid sprites live in the G6-cached world
//! range with light-tinted alpha, so the fallback applies: a **separate
//! coverage pass over the same liquid records** (`tr-app` collects the rects
//! when the world pass builds them — identical cache invalidation).
//!
//! The target (R8, surface-sized, view-aligned) exists only while the
//! WaterDistortion pass is active and has water in view (plan constraint: "the
//! second target only while WaterDistortion is active") — [`WaterMask::release`]
//! frees it otherwise.

use bytemuck::{Pod, Zeroable};

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

/// One liquid rect in world pixels: `[x, y, w, h]`.
pub type MaskRect = [f32; 4];

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct MaskGlobals {
    screen: [f32; 2],
    cam: [f32; 2],
    zoom: f32,
    _pad: [f32; 3],
}

/// The lazily-allocated mask target + its quad pipeline.
pub struct WaterMask {
    pipeline: wgpu::RenderPipeline,
    bgl: wgpu::BindGroupLayout,
    globals: wgpu::Buffer,
    instances: wgpu::Buffer,
    /// Capacity of `instances` in rects.
    capacity: usize,
    target: Option<(wgpu::Texture, wgpu::TextureView, (u32, u32))>,
}

/// Instance buffer growth: at least 256 rects, power-of-two.
fn grown_capacity(current: usize, needed: usize) -> usize {
    if needed <= current {
        current
    } else {
        needed.next_power_of_two().max(256)
    }
}

impl WaterMask {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("water-mask"),
            source: wgpu::ShaderSource::Wgsl(include_str!("water_mask.wgsl").into()),
        });
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("water-mask-bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("water-mask-layout"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("water-mask-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<MaskRect>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
                })],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let globals = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("water-mask-globals"),
            size: std::mem::size_of::<MaskGlobals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let capacity = grown_capacity(0, 1);
        Self {
            pipeline,
            bgl,
            globals,
            instances: Self::instance_buffer(device, capacity),
            capacity,
            target: None,
        }
    }

    fn instance_buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("water-mask-instances"),
            size: (capacity * std::mem::size_of::<MaskRect>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    /// The mask texture while one exists (the last [`WaterMask::render`]).
    pub fn texture(&self) -> Option<&wgpu::Texture> {
        self.target.as_ref().map(|(t, _, _)| t)
    }

    /// Frees the target (WaterDistortion inactive / no water in view).
    pub fn release(&mut self) {
        self.target = None;
    }

    /// Rasterises `rects` (world pixels) into the mask as seen from `cam`
    /// (the view centre) at `zoom`, `size` = the surface size in pixels —
    /// the same mapping as `SpriteRenderer::render`, so the mask lines up with
    /// the scene pixel for pixel. Clears the target first (an empty `rects`
    /// yields an all-zero mask). Returns the mask texture.
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: (u32, u32),
        cam: [f32; 2],
        zoom: f32,
        rects: &[MaskRect],
    ) -> &wgpu::Texture {
        let size = (size.0.max(1), size.1.max(1));
        if self.target.as_ref().map(|t| t.2) != Some(size) {
            let tex = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("water-mask"),
                size: wgpu::Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
            self.target = Some((tex, view, size));
        }
        let new_capacity = grown_capacity(self.capacity, rects.len());
        if new_capacity != self.capacity {
            self.instances = Self::instance_buffer(device, new_capacity);
            self.capacity = new_capacity;
        }
        if !rects.is_empty() {
            queue.write_buffer(&self.instances, 0, bytemuck::cast_slice(rects));
        }
        queue.write_buffer(
            &self.globals,
            0,
            bytemuck::bytes_of(&MaskGlobals {
                screen: [size.0 as f32, size.1 as f32],
                cam,
                zoom,
                _pad: [0.0; 3],
            }),
        );
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("water-mask-bind"),
            layout: &self.bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.globals.as_entire_binding(),
            }],
        });
        let (_, view, _) = self.target.as_ref().expect("mask target");
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("water-mask"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("water-mask-pass"),
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
            if !rects.is_empty() {
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &bind, &[]);
                pass.set_vertex_buffer(0, self.instances.slice(..));
                pass.draw(0..6, 0..rects.len() as u32);
            }
        }
        queue.submit(Some(encoder.finish()));
        self.texture().expect("mask target")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_gpu::{headless, read_texture};

    #[test]
    fn instance_capacity_grows_by_powers_of_two() {
        assert_eq!(grown_capacity(256, 10), 256, "no shrink, no regrow");
        assert_eq!(grown_capacity(256, 256), 256);
        assert_eq!(grown_capacity(256, 257), 512);
        assert_eq!(grown_capacity(0, 1), 256, "minimum 256");
        assert_eq!(grown_capacity(256, 5000), 8192);
    }

    /// DW-4.2 (render test): the mask is 255 exactly inside the liquid rects
    /// and 0 elsewhere, through the same world->screen mapping as the sprite
    /// pass (camera centre, zoom).
    #[test]
    fn test_DW_4_2_water_mask_filled_over_liquid_rects() {
        let Some((device, queue)) = headless() else {
            eprintln!("no GPU adapter: skipping");
            return;
        };
        let mut mask = WaterMask::new(&device);
        // 64x48 view at zoom 1 centred on world (100, 100): world x 68..132,
        // y 76..124 maps to screen (0..64, 0..48). Two rects: one fully inside,
        // one hanging off the right edge (clipped, "several water regions").
        let rects: [MaskRect; 2] = [[80.0, 80.0, 16.0, 16.0], [120.0, 100.0, 40.0, 8.0]];
        let tex = mask.render(&device, &queue, (64, 48), [100.0, 100.0], 1.0, &rects);
        let px = read_texture(&device, &queue, tex, (64, 48), 1);
        let at = |x: usize, y: usize| px[y * 64 + x];
        // rect 0 -> screen x 12..28, y 4..20.
        assert_eq!(at(12, 4), 255, "top-left texel of rect 0");
        assert_eq!(at(27, 19), 255, "bottom-right texel of rect 0");
        assert_eq!(at(11, 4), 0, "left of rect 0");
        assert_eq!(at(12, 3), 0, "above rect 0");
        assert_eq!(at(28, 10), 0, "right of rect 0");
        // rect 1 -> screen x 52..92 (clipped at 64), y 24..32.
        assert_eq!(at(52, 24), 255);
        assert_eq!(at(63, 31), 255, "clipped at the view edge");
        assert_eq!(at(51, 24), 0);
        assert_eq!(at(60, 32), 0, "below rect 1");
        // Coverage count = 16*16 + 12*8.
        let filled = px.iter().filter(|&&v| v == 255).count();
        assert_eq!(filled, 16 * 16 + 12 * 8);
    }

    /// Zoom scales the mapping about the view centre (edge case: zoom).
    #[test]
    fn test_water_mask_respects_zoom() {
        let Some((device, queue)) = headless() else {
            return;
        };
        let mut mask = WaterMask::new(&device);
        // Zoom 2: a 8x8 world rect at the camera centre covers 16x16 pixels.
        let tex = mask.render(
            &device,
            &queue,
            (64, 64),
            [50.0, 50.0],
            2.0,
            &[[46.0, 46.0, 8.0, 8.0]],
        );
        let px = read_texture(&device, &queue, tex, (64, 64), 1);
        let filled = px.iter().filter(|&&v| v == 255).count();
        assert_eq!(filled, 16 * 16);
        assert_eq!(px[32 * 64 + 32], 255, "view centre is inside");
        assert_eq!(px[23 * 64 + 32], 0);
    }

    /// Edge cases: water entirely off-screen and an empty rect list both
    /// leave an all-zero mask; a later render clears the previous content.
    #[test]
    fn test_water_mask_offscreen_and_empty_are_all_zero() {
        let Some((device, queue)) = headless() else {
            return;
        };
        let mut mask = WaterMask::new(&device);
        let tex = mask.render(
            &device,
            &queue,
            (32, 32),
            [0.0, 0.0],
            1.0,
            &[[5000.0, 5000.0, 16.0, 16.0]],
        );
        let px = read_texture(&device, &queue, tex, (32, 32), 1);
        assert!(px.iter().all(|&v| v == 0), "off-screen water");
        let tex = mask.render(
            &device,
            &queue,
            (32, 32),
            [16.0, 16.0],
            1.0,
            &[[-4.0, -4.0, 8.0, 8.0]],
        );
        let px = read_texture(&device, &queue, tex, (32, 32), 1);
        assert_eq!(px.iter().filter(|&&v| v == 255).count(), 16, "4x4 corner");
        let tex = mask.render(&device, &queue, (32, 32), [0.0, 0.0], 1.0, &[]);
        let px = read_texture(&device, &queue, tex, (32, 32), 1);
        assert!(px.iter().all(|&v| v == 0), "empty list clears the target");
    }

    /// Release drops the target (the "second target only while active"
    /// constraint); a resize re-creates it at the new size; > initial
    /// capacity rects regrow the instance buffer.
    #[test]
    fn test_water_mask_release_resize_and_regrow() {
        let Some((device, queue)) = headless() else {
            return;
        };
        let mut mask = WaterMask::new(&device);
        assert!(
            mask.texture().is_none(),
            "no target before the first render"
        );
        mask.render(&device, &queue, (16, 16), [8.0, 8.0], 1.0, &[]);
        assert!(mask.texture().is_some());
        mask.release();
        assert!(mask.texture().is_none(), "released");
        let many: Vec<MaskRect> = (0..600)
            .map(|i| [(i % 30) as f32 * 2.0, (i / 30) as f32, 1.0, 1.0])
            .collect();
        let tex = mask.render(&device, &queue, (64, 32), [32.0, 16.0], 1.0, &many);
        assert_eq!((tex.width(), tex.height()), (64, 32));
        let px = read_texture(&device, &queue, tex, (64, 32), 1);
        assert_eq!(
            px.iter().filter(|&&v| v == 255).count(),
            600,
            "all regrown rects drew"
        );
    }
}
