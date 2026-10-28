//! Instanced sprite renderer: one atlas, one pipeline, one draw call per layer.

use std::collections::HashMap;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::atlas::Atlas;
use crate::gpu::Gpu;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Globals {
    pub screen: [f32; 2],
    pub cam: [f32; 2],
    pub zoom: f32,
    pub time: f32,
    pub _pad: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct SpriteInstance {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub uv0: [f32; 2],
    pub uv1: [f32; 2],
    pub color: [f32; 4],
}

#[derive(Default, Clone, Copy, Debug)]
pub struct RenderStats {
    pub instances: u32,
    pub draw_calls: u32,
    pub uploaded_bytes: u64,
}

pub struct SpriteRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_world: wgpu::BindGroup,
    bind_ui: wgpu::BindGroup,
    globals_world: wgpu::Buffer,
    globals_ui: wgpu::Buffer,
    /// Ring of instance buffers so the CPU never rewrites a buffer the GPU may still
    /// be reading (B3). Three buffers stay ahead of `desired_maximum_frame_latency` (≤2).
    instances: Vec<wgpu::Buffer>,
    /// Ring cursor.
    frame_index: usize,
    capacity: u32,
    pub world: Vec<SpriteInstance>,
    pub ui: Vec<SpriteInstance>,
    pub sheets: HashMap<String, [f32; 4]>,
    pub atlas_size: (u32, u32),
    pub font_glyph_px: f32,
    font_uv0: [f32; 2],
    font_uv1: [f32; 2],
    white_uv: [f32; 4],
}

const RING: usize = 3;

impl SpriteRenderer {
    pub fn new(gpu: &Gpu, atlas: &Atlas, font_glyph_px: u32) -> Self {
        let device = &gpu.device;
        let atlas_tex = device.create_texture_with_data(
            &gpu.queue,
            &wgpu::TextureDescriptor {
                label: Some("atlas"),
                size: wgpu::Extent3d {
                    width: atlas.width,
                    height: atlas.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &atlas.pixels,
        );
        let view = atlas_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("atlas-sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let globals_world = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("globals-world"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let globals_ui = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("globals-ui"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite-bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let mk_bind = |buf: &wgpu::Buffer, label: &str| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buf.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            })
        };
        let bind_world = mk_bind(&globals_world, "bg-world");
        let bind_ui = mk_bind(&globals_ui, "bg-ui");

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sprite"),
            source: wgpu::ShaderSource::Wgsl(include_str!("sprite.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sprite-layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sprite-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<SpriteInstance>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x2,
                        1 => Float32x2,
                        2 => Float32x2,
                        3 => Float32x2,
                        4 => Float32x4,
                    ],
                })],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: gpu.config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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

        let capacity = 262_144u32;
        let make_instances = |cap: u32| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("instances"),
                size: cap as u64 * std::mem::size_of::<SpriteInstance>() as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let instances = (0..RING).map(|_| make_instances(capacity)).collect();

        let mut sheets = HashMap::with_capacity(atlas.sheets.len());
        for (k, v) in &atlas.sheets {
            sheets.insert(
                k.clone(),
                [
                    v.x as f32 / atlas.width as f32,
                    v.y as f32 / atlas.height as f32,
                    (v.x + v.width) as f32 / atlas.width as f32,
                    (v.y + v.height) as f32 / atlas.height as f32,
                ],
            );
        }

        let font = atlas.sheet("__font").expect("font sheet");
        let font_uv0 = [
            font.x as f32 / atlas.width as f32,
            font.y as f32 / atlas.height as f32,
        ];
        let font_uv1 = [
            (font.x + font.width) as f32 / atlas.width as f32,
            (font.y + font.height) as f32 / atlas.height as f32,
        ];
        let white = atlas.sheet("__white").expect("white pixel sheet");
        let white_uv = [
            white.x as f32 / atlas.width as f32,
            white.y as f32 / atlas.height as f32,
            (white.x + white.width) as f32 / atlas.width as f32,
            (white.y + white.height) as f32 / atlas.height as f32,
        ];

        Self {
            pipeline,
            bind_world,
            bind_ui,
            globals_world,
            globals_ui,
            instances,
            frame_index: 0,
            capacity,
            world: Vec::with_capacity(64 * 1024),
            ui: Vec::with_capacity(8 * 1024),
            sheets,
            atlas_size: (atlas.width, atlas.height),
            font_glyph_px: font_glyph_px as f32,
            font_uv0,
            font_uv1,
            white_uv,
        }
    }

    pub fn uv(&self, sheet: &str, sx: f32, sy: f32, sw: f32, sh: f32) -> Option<[f32; 4]> {
        let r = self.sheets.get(sheet)?;
        let aw = self.atlas_size.0 as f32;
        let ah = self.atlas_size.1 as f32;
        let x0 = r[0] * aw;
        let y0 = r[1] * ah;
        Some([
            (x0 + sx) / aw,
            (y0 + sy) / ah,
            (x0 + sx + sw) / aw,
            (y0 + sy + sh) / ah,
        ])
    }

    pub fn push_sprite(&mut self, pos: [f32; 2], size: [f32; 2], uv: [f32; 4], color: [f32; 4]) {
        self.world.push(SpriteInstance {
            pos,
            size,
            uv0: [uv[0], uv[1]],
            uv1: [uv[2], uv[3]],
            color,
        });
    }

    /// Solid-colour rectangle in screen space (debug UI).
    pub fn push_ui_rect(&mut self, pos: [f32; 2], size: [f32; 2], color: [f32; 4]) {
        self.ui.push(SpriteInstance {
            pos,
            size,
            uv0: [self.white_uv[0], self.white_uv[1]],
            uv1: [self.white_uv[2], self.white_uv[3]],
            color,
        });
    }

    /// Draws text in screen space (top-left origin, 8x8 glyphs scaled by `scale`).
    pub fn push_text(&mut self, x: f32, y: f32, scale: f32, color: [f32; 4], text: &str) {
        let gw = self.font_glyph_px;
        let du = (self.font_uv1[0] - self.font_uv0[0]) / 96.0;
        let mut cx = x;
        for ch in text.chars() {
            let code = ch as u32;
            if code >= 32 && code < 128 {
                let idx = (code - 32) as f32;
                let u0 = self.font_uv0[0] + du * idx;
                let u1 = u0 + du;
                self.ui.push(SpriteInstance {
                    pos: [cx, y],
                    size: [gw * scale, 8.0 * scale],
                    uv0: [u0, self.font_uv0[1]],
                    uv1: [u1, self.font_uv1[1]],
                    color,
                });
            }
            cx += gw * scale;
        }
    }

    /// UV rect of the 4x4 solid-white pixel block (for untextured quads).
    pub fn white_uv(&self) -> [f32; 4] {
        self.white_uv
    }

    pub fn text_width(&self, text: &str, scale: f32) -> f32 {
        text.chars().count() as f32 * self.font_glyph_px * scale
    }

    pub fn begin_frame(&mut self) {
        self.world.clear();
        self.ui.clear();
    }

    /// G6: clear only the UI range and keep the previous frame's world instances. The
    /// caller must have proven that camera, zoom, screen size, sky tint and world
    /// contents are unchanged (see `WorldFrameKey` in the app), otherwise the kept
    /// range is stale.
    pub fn begin_frame_keep_world(&mut self) {
        self.ui.clear();
    }

    pub fn render(
        &mut self,
        gpu: &Gpu,
        view: &wgpu::TextureView,
        camera: [f32; 2],
        zoom: f32,
        time: f32,
        clear: wgpu::Color,
    ) -> RenderStats {
        let mut stats = RenderStats::default();
        let w = gpu.config.width as f32;
        let h = gpu.config.height as f32;

        gpu.queue.write_buffer(
            &self.globals_world,
            0,
            bytemuck::bytes_of(&Globals {
                screen: [w, h],
                cam: camera,
                zoom,
                time,
                _pad: [0.0; 2],
            }),
        );
        gpu.queue.write_buffer(
            &self.globals_ui,
            0,
            bytemuck::bytes_of(&Globals {
                screen: [w, h],
                cam: [w * 0.5, h * 0.5],
                zoom: 1.0,
                time,
                _pad: [0.0; 2],
            }),
        );

        let total = self.world.len() + self.ui.len();
        if total == 0 {
            return stats;
        }
        if total as u32 > self.capacity {
            let new_cap = (total as u32).next_power_of_two();
            self.instances = (0..RING)
                .map(|_| {
                    gpu.device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("instances"),
                        size: new_cap as u64 * std::mem::size_of::<SpriteInstance>() as u64,
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    })
                })
                .collect();
            self.capacity = new_cap;
            self.frame_index = 0;
        }

        // B1: no per-frame `collect`. The world and UI ranges are written straight
        // into one buffer at their final offsets (48-byte instances keep 4-byte
        // alignment, so a single `write_buffer` copy of each slice is valid).
        let idx = self.frame_index;
        self.frame_index = (self.frame_index + 1) % RING;
        let buffer = &self.instances[idx];
        let world_bytes = std::mem::size_of_val(self.world.as_slice());
        if !self.world.is_empty() {
            gpu.queue
                .write_buffer(buffer, 0, bytemuck::cast_slice(&self.world));
        }
        if !self.ui.is_empty() {
            gpu.queue
                .write_buffer(buffer, world_bytes as u64, bytemuck::cast_slice(&self.ui));
        }
        stats.uploaded_bytes = (world_bytes + std::mem::size_of_val(self.ui.as_slice())) as u64;
        stats.instances = total as u32;

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, buffer.slice(..));
            if !self.world.is_empty() {
                pass.set_bind_group(0, &self.bind_world, &[]);
                pass.draw(0..6, 0..self.world.len() as u32);
                stats.draw_calls += 1;
            }
            if !self.ui.is_empty() {
                let start = self.world.len() as u32;
                pass.set_bind_group(0, &self.bind_ui, &[]);
                pass.draw(0..6, start..(start + self.ui.len() as u32));
                stats.draw_calls += 1;
            }
        }

        gpu.queue.submit(Some(encoder.finish()));
        stats
    }
}
