//! Instanced sprite renderer: one atlas, one pipeline, one draw call per layer.

use std::collections::HashMap;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::atlas::Atlas;
use crate::clip::{ClipStack, Rect as ClipRect};
use crate::font_atlas::FontAtlas;
use crate::gpu::Gpu;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Globals {
    pub screen: [f32; 2],
    pub cam: [f32; 2],
    pub zoom: f32,
    pub time: f32,
    /// Screen-space RGB split for the "Trippy" light mode.
    pub aberration: f32,
    pub _pad: f32,
    pub texel: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct SpriteInstance {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub uv0: [f32; 2],
    pub uv1: [f32; 2],
    pub color: [f32; 4],
    /// Clockwise rotation in radians around the sprite's top-left corner.
    pub rot: f32,
    /// `PaintID` for the fragment-shader recolor (0 = unpainted). Matches
    /// `Main.ConvertPaintIdToTileShaderIndex`'s input ids.
    pub paint: f32,
    /// The armor-dye slot (0 = plain): 1-based index into the dye uniform
    /// table `set_dye_params` uploads — the per-instance channel for the
    /// port's `DrawData.shader` (`PlayerDrawHelper.cs:29-60` unpacks the C#
    /// side per draw entry).
    pub dye: u32,
    /// Which texture the instance samples: 0 = the main RGBA atlas, otherwise
    /// 1-based layer into the BC2 font array (`font_atlas::FontAtlas`) — the
    /// retail `DynamicSpriteFont` glyph pages (`Fonts/*.xnb`).
    pub layer: u32,
    /// The scissor rect in force when the instance was pushed, as
    /// `[x, y, w, h]` in framebuffer pixels ([`crate::clip::NO_CLIP`] when
    /// unclipped). `UIElement.Draw`'s `OverflowHidden` path
    /// (`UIElement.cs:222-237`) is a device scissor in the C#; the port bakes
    /// the rect into the instance and the fragment shader discards outside it.
    pub clip: [f32; 4],
    /// The blend batch this instance rides ([`BlendMode`]): the C# ends and
    /// re-begins `Main.spriteBatch` around the additive / reverse-subtract
    /// batches (`Main.cs:27457-27494`); the port keeps one buffer and splits
    /// each range into contiguous blend runs in `draw_ranges` (S1-G-3).
    pub blend: u32,
}

/// The sprite pipeline's blend batches (S1-G-3, research/41 D-2). The vanilla
/// frame runs at least three `SpriteBatch.Begin` blend configurations; one
/// blend state cannot express them all (`Main.cs:27462-27490`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum BlendMode {
    /// `BlendState.AlphaBlend` (`Main.cs:22193`, `:27494`) — the default
    /// batch; the premultiplied-source convention documented at the pipeline
    /// (`sprite.rs`'s alpha factors, research/41 D-2).
    #[default]
    AlphaBlend = 0,
    /// `BlendState.Additive` (`Main.cs:27490`, the Lightsbane magenta glow):
    /// XNA's stock additive state — `(SourceAlpha, One)` Add on both the
    /// color and the alpha channels.
    Additive = 1,
    /// `Main._multiplyBlendState` (`Main.cs:27462-27470`): the custom
    /// "multiply" batch — `ReverseSubtract` with `ColorSourceBlend =
    /// SourceAlpha`, `ColorDestinationBlend = One`, and the same three
    /// fields on the alpha channel (`dst − src·srcAlpha`).
    Multiply = 2,
}

impl BlendMode {
    /// The fixed-function blend factors, 1:1 with the C# `BlendState` fields
    /// (`Main.cs:27462-27470` for Multiply; XNA's stock `BlendState.Additive`
    /// for Additive). The fragment output is unchanged across the modes —
    /// exactly like the vanilla, where the same SpriteBatch shader feeds all
    /// three `Begin` configurations and only the device blend state differs.
    pub fn blend_state(self) -> wgpu::BlendState {
        match self {
            Self::AlphaBlend => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
            },
            // XNA `BlendState.Additive`: SourceAlpha/One, Add — color and
            // alpha alike (`Main.cs:27490`'s Begin).
            Self::Additive => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
            },
            // Main.cs:27462-27470 — ColorBlendFunction = ReverseSubtract,
            // ColorDestinationBlend = One, ColorSourceBlend = SourceAlpha;
            // AlphaBlendFunction/AlphaDestinationBlend/AlphaSourceBlend the
            // same. ReverseSubtract computes `dst·dstFactor − src·srcFactor`.
            Self::Multiply => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::ReverseSubtract,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::ReverseSubtract,
                },
            },
        }
    }
}

/// One dye uniform-table row: the pass mode and the two colors. Mirrors
/// `sprite.wgsl`'s `DyeParam` (mode 1 = colored wash, 2 = gradient, 0/other =
/// plain — the unused-slot fallback).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct DyeParam {
    pub mode: u32,
    pub _pad: [u32; 3],
    pub color: [f32; 4],
    pub secondary: [f32; 4],
}

/// The dye uniform table's fixed length (`sprite.wgsl` declares the same
/// array bound; the vanilla registry's 36 deduped slots fit with headroom).
pub const DYE_TABLE_LEN: usize = 48;

#[derive(Default, Clone, Copy, Debug)]
pub struct RenderStats {
    pub instances: u32,
    pub draw_calls: u32,
    pub uploaded_bytes: u64,
}

/// The renderer's instance-buffer set — one `pass.draw` per non-empty
/// buffer per `draw_ranges` call (`sprite.rs::draw_ranges`): the background,
/// world, world-fx, map-back, map and UI ranges. The honest draw-call bound
/// the docs claim (`docs/ARCHITECTURE.md`'s renderer rows; S1-G-12 corrected
/// the stale "2 draw calls" — the real structure is up to 6 per frame, 3 + 3
/// under the split post path with 1..4 filter passes on top). S1-G-3 adds
/// blend-run splits inside a range: each contiguous [`BlendMode`] run is its
/// own draw, so a range holding the Lightsbane multiply/additive batches
/// (`Main.cs:27457-27494`) costs up to 3 draws instead of 1 (up to 8 per
/// frame when both batches are live).
pub const PASS_BUFFERS: usize = 6;

pub struct SpriteRenderer {
    /// One pipeline per [`BlendMode`], indexed by the mode's discriminant
    /// (S1-G-3): alpha-blend (the default), additive, reverse-subtract
    /// "multiply" — the C#'s three `SpriteBatch.Begin` blend batches
    /// (`Main.cs:27457-27494`).
    pub(crate) pipelines: [wgpu::RenderPipeline; 3],
    pub(crate) bgl: wgpu::BindGroupLayout,
    pub(crate) sampler: wgpu::Sampler,
    bind_world: wgpu::BindGroup,
    bind_ui: wgpu::BindGroup,
    /// The BC2 font-array view the retail `Fonts/*.xnb` pages upload into
    /// (`upload_font_atlas`); shared by every sprite bind group.
    pub(crate) font_view: wgpu::TextureView,
    /// Bind group for the world-sized map texture (one texel = one map tile). `None`
    /// until `ensure_map_texture` has run, in which case the `map` range is not drawn.
    bind_map: Option<wgpu::BindGroup>,
    map_tex: Option<wgpu::Texture>,
    map_size: (u32, u32),
    globals_world: wgpu::Buffer,
    globals_ui: wgpu::Buffer,
    /// The dye pass-parameter table (`set_dye_params`); bound in every sprite
    /// bind group, read by `sprite.wgsl`'s dye dispatch.
    pub(crate) dye_table: wgpu::Buffer,
    /// S1-G-13 consumers: an explicit framebuffer size for the world globals.
    /// `None` = the surface size (the frozen default). The layer-target
    /// callbacks render into a padded layer texture whose size differs from
    /// the backbuffer, so they set this for the duration of the layer draw
    /// (`WorldSceneLayerTarget.UpdateContent`'s screen-size override,
    /// `WorldSceneLayerTarget.cs:49`). The composite/capture paths read it.
    pub(crate) screen_override: Option<[f32; 2]>,
    /// Ring of instance buffers so the CPU never rewrites a buffer the GPU may still
    /// be reading (B3). Three buffers stay ahead of `desired_maximum_frame_latency` (≤2).
    instances: Vec<wgpu::Buffer>,
    /// Ring cursor.
    frame_index: usize,
    capacity: u32,
    /// Always-rebuilt background range, drawn before the cached world range.
    pub back: Vec<SpriteInstance>,
    pub world: Vec<SpriteInstance>,
    /// Screen-space world-effect sprites (rain, snow, sand dust, leaf gores)
    /// that the vanilla draws as world content: after the world, **before**
    /// the map batches and the interface, so the fullscreen map and the HUD
    /// cover them (`Main.Draw`'s order — the world draw, `DrawMap`, then
    /// `DrawInterface`). Always rebuilt — the particles change per frame, so
    /// they must not ride the G6-cached world range.
    pub fx: Vec<SpriteInstance>,
    /// Screen-space atlas sprites drawn behind the map (biome `MapBG` backdrop and
    /// the `Images/Map` scroll frame). Always rebuilt; empty until Task 7.
    pub map_back: Vec<SpriteInstance>,
    /// Screen-space quads sampling the map texture, drawn after `map_back`.
    pub map: Vec<SpriteInstance>,
    pub ui: Vec<SpriteInstance>,
    pub sheets: HashMap<String, [f32; 4]>,
    pub atlas_size: (u32, u32),
    pub font_glyph_px: f32,
    /// RGB split applied to the world pass (light mode "Trippy"); 0 disables it.
    pub aberration: f32,
    /// The `GraphicsDevice.ScissorRectangle` stack (`UIElement.cs:222-237`): every
    /// instance pushed while an `OverflowHidden` element is open carries
    /// [`ClipStack::current`] and the fragment shader discards outside it.
    clip: ClipStack,
    font_uv0: [f32; 2],
    font_uv1: [f32; 2],
    white_uv: [f32; 4],
}

const RING: usize = 3;

/// The atlas sampler: XNA `SamplerState.PointClamp` — nearest min/mag/mip,
/// clamp on both axes. The C# world draws run under it: `Camera.Sampler` is
/// `Main.DefaultSamplerState` (`Camera.cs:44`), which is `PointClamp` whenever
/// the frame renders through the capture targets (`Main.cs:2774-2784`,
/// `!drawToScreen`); `LegacyPlayerRenderer.DrawPlayerFull` begins the batch
/// with it (`LegacyPlayerRenderer.cs:264-269`). Nearest filtering is what
/// keeps fractional world positions pixel-crisp: the entity layers truncate
/// their screen positions (`PlayerDrawLayers.cs:1907/1939` `(int)`), and the
/// raw-position draws (NPCs/projectiles, `Main.cs:30136`) still land on whole
/// texels. A LINEAR sampler here would bilinearly blend neighbouring texels
/// at sub-pixel offsets — the smear the user reported (fix/player-motion-blur
/// investigated it; the actual culprit was the invented draw interpolation).
/// Pure fn so the state is test-pinned without a GPU (`tests` below).
pub(crate) fn atlas_sampler_descriptor() -> wgpu::SamplerDescriptor<'static> {
    wgpu::SamplerDescriptor {
        label: Some("atlas-sampler"),
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        ..Default::default()
    }
}

impl SpriteRenderer {
    /// `font` uploads the retail `Fonts/*.xnb` glyph pages as a BC2 texture
    /// array (`font_atlas::FontAtlas`); `None` (or an adapter without the BC
    /// feature, handled by the caller) falls back to the built-in bitmap font.
    pub fn new(gpu: &Gpu, atlas: &Atlas, font_glyph_px: u32, font: Option<&FontAtlas>) -> Self {
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

        // The font texture array: the shipped BC2/DXT3 pages packed per layer,
        // or an empty RGBA stand-in when the retail fonts are unavailable (the
        // layout's texture entry is format-agnostic; instances never address it).
        let font_view = match font {
            Some(font) => Self::upload_font_atlas(gpu, font),
            None => {
                let tex = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("font-atlas-empty"),
                    size: wgpu::Extent3d {
                        width: 4,
                        height: 4,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                });
                tex.create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    ..Default::default()
                })
            }
        };

        let sampler = device.create_sampler(&atlas_sampler_descriptor());

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
        let dye_table = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("dye-table"),
            size: (DYE_TABLE_LEN * std::mem::size_of::<DyeParam>()) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite-bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            DYE_TABLE_LEN as u64 * std::mem::size_of::<DyeParam>() as u64,
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let bind_world = Self::mk_bind(
            device,
            &layout,
            &sampler,
            &globals_world,
            &view,
            &dye_table,
            &font_view,
            "bg-world",
        );
        let bind_ui = Self::mk_bind(
            device,
            &layout,
            &sampler,
            &globals_ui,
            &view,
            &dye_table,
            &font_view,
            "bg-ui",
        );

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sprite"),
            source: wgpu::ShaderSource::Wgsl(include_str!("sprite.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sprite-layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });

        // S1-G-3 (research/41 D-2): one pipeline per blend batch. The C#
        // re-begins `Main.spriteBatch` with a different BlendState around the
        // Lightsbane batches (`Main.cs:27457-27494`); the port keeps one
        // instance buffer and switches pipelines per contiguous blend run.
        // The fragment shader is identical across the three — exactly like
        // the vanilla, where only the device blend state changes.
        let make_pipeline = |blend: BlendMode, label: &str| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
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
                            5 => Float32,
                            6 => Float32,
                            7 => Uint32,
                            8 => Uint32,
                            9 => Float32x4,
                            10 => Uint32,
                        ],
                    })],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: gpu.config.format,
                        // XNA `BlendState.AlphaBlend` (premultiplied): the rgb source
                        // factor is One, so a tint with alpha 0 (the tile flames'
                        // `Color(200,200,200,0)`) is a pure rgb add; the shader
                        // outputs straight-alpha texels without pre-multiplying rgb
                        // by alpha, and these factors finish the equation.
                        blend: Some(blend.blend_state()),
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
            })
        };
        let pipelines = [
            make_pipeline(BlendMode::AlphaBlend, "sprite-pipeline"),
            make_pipeline(BlendMode::Additive, "sprite-pipeline-additive"),
            make_pipeline(BlendMode::Multiply, "sprite-pipeline-multiply"),
        ];

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
            pipelines,
            bgl: layout,
            sampler,
            bind_world,
            bind_ui,
            font_view,
            bind_map: None,
            map_tex: None,
            map_size: (0, 0),
            globals_world,
            globals_ui,
            dye_table,
            screen_override: None,
            instances,
            frame_index: 0,
            capacity,
            back: Vec::with_capacity(1024),
            world: Vec::with_capacity(64 * 1024),
            fx: Vec::with_capacity(2048),
            map_back: Vec::with_capacity(64),
            map: Vec::with_capacity(8),
            ui: Vec::with_capacity(8 * 1024),
            sheets,
            atlas_size: (atlas.width, atlas.height),
            font_glyph_px: font_glyph_px as f32,
            aberration: 0.0,
            clip: ClipStack::new(),
            font_uv0,
            font_uv1,
            white_uv,
        }
    }

    /// Uploads the packed font pages as one BC2 texture array — the vanilla
    /// `SurfaceFormat.Dxt3` path (`Texture2D` uploads the compressed blocks
    /// verbatim, so GPU sampling reproduces the retail pixels).
    fn upload_font_atlas(gpu: &Gpu, font: &FontAtlas) -> wgpu::TextureView {
        let device = &gpu.device;
        let layers = font.layers.len() as u32;
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("font-atlas"),
            size: wgpu::Extent3d {
                width: 8192,
                height: 8192,
                depth_or_array_layers: layers,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bc2RgbaUnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        // One layer per write: BC2 rows are `blocks_per_row * 16` bytes.
        let bytes_per_row = 8192 / 4 * 16;
        for (li, data) in font.layers.iter().enumerate() {
            gpu.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: li as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(8192 / 4),
                },
                wgpu::Extent3d {
                    width: 8192,
                    height: 8192,
                    depth_or_array_layers: 1,
                },
            );
        }
        tex.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        })
    }

    /// Builds one sprite bind group: globals uniform + a texture view + the shared
    /// sampler + the dye table. Used for the atlas view (`bg-world`/`bg-ui`) and
    /// the map view.
    pub(crate) fn mk_bind(
        device: &wgpu::Device,
        bgl: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        buf: &wgpu::Buffer,
        tex: &wgpu::TextureView,
        dye: &wgpu::Buffer,
        font_tex: &wgpu::TextureView,
        label: &str,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout: bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(tex),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: dye.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(font_tex),
                },
            ],
        })
    }

    /// `MapRenderer.CheckMapTargets` (`MapRenderer.cs:357-378`): (re)creates the
    /// world-sized map texture (one texel = one map tile) and its bind group.
    /// Only recreates when width/height differ from the current size.
    pub fn ensure_map_texture(&mut self, gpu: &Gpu, width: u32, height: u32) {
        if self.map_size == (width, height) && self.map_tex.is_some() {
            return;
        }
        let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("map"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // Same sRGB convention as the atlas texture (`sprite.rs:86`): the map
            // colours are authored sRGB bytes and must round-trip, whereas a plain
            // `Rgba8Unorm` view would brighten them.
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        // The map quad is screen-space, so it binds the UI globals (cam = [w/2, h/2],
        // zoom = 1) with the existing nearest sampler.
        self.bind_map = Some(Self::mk_bind(
            &gpu.device,
            &self.bgl,
            &self.sampler,
            &self.globals_ui,
            &view,
            &self.dye_table,
            &self.font_view,
            "bg-map",
        ));
        self.map_tex = Some(tex);
        self.map_size = (width, height);
    }

    /// `MapRenderer.DrawChangesToMap` upload: writes an RGBA8 region (row-major) into
    /// the map texture; `rgba.len() == (w * h) as usize`.
    pub fn write_map_region(
        &mut self,
        gpu: &Gpu,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        rgba: &[[u8; 4]],
    ) {
        gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: self.map_tex.as_ref().expect("map texture"),
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(rgba),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * 4),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
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
        probe(pos, size, uv, color);
        self.world.push(SpriteInstance {
            pos,
            size,
            uv0: [uv[0], uv[1]],
            uv1: [uv[2], uv[3]],
            color,
            rot: 0.0,
            paint: 0.0,
            dye: 0,
            layer: 0,
            clip: self.clip.current(),
            blend: 0,
        });
    }

    /// Background-range sprite (sky, parallax layers, clouds, cave fade).
    pub fn push_back_sprite(
        &mut self,
        pos: [f32; 2],
        size: [f32; 2],
        uv: [f32; 4],
        color: [f32; 4],
    ) {
        probe(pos, size, uv, color);
        self.back.push(SpriteInstance {
            pos,
            size,
            uv0: [uv[0], uv[1]],
            uv1: [uv[2], uv[3]],
            color,
            rot: 0.0,
            paint: 0.0,
            dye: 0,
            layer: 0,
            clip: self.clip.current(),
            blend: 0,
        });
    }

    /// Rotated background-range sprite. Used by the sun and moon, which the
    /// original draws rotated about their centre (`DrawSunAndMoon`).
    pub fn push_back_sprite_rot(
        &mut self,
        pos: [f32; 2],
        size: [f32; 2],
        uv: [f32; 4],
        color: [f32; 4],
        rot: f32,
    ) {
        self.back.push(SpriteInstance {
            pos,
            size,
            uv0: [uv[0], uv[1]],
            uv1: [uv[2], uv[3]],
            color,
            rot,
            paint: 0.0,
            dye: 0,
            layer: 0,
            clip: self.clip.current(),
            blend: 0,
        });
    }

    /// `push_sprite_rot` with a `PaintID`: the fragment shader recolors the
    /// texel through the paint table before the light tint multiplies it.
    pub fn push_sprite_paint(
        &mut self,
        pos: [f32; 2],
        size: [f32; 2],
        uv: [f32; 4],
        color: [f32; 4],
        rot: f32,
        paint: u8,
    ) {
        self.world.push(SpriteInstance {
            pos,
            size,
            uv0: [uv[0], uv[1]],
            uv1: [uv[2], uv[3]],
            color,
            rot,
            paint: paint as f32,
            dye: 0,
            layer: 0,
            clip: self.clip.current(),
            blend: 0,
        });
    }

    /// `push_sprite_rot` with a dye slot: the fragment shader runs the dye
    /// pass over the texel — the per-instance channel for the port's
    /// `DrawData.shader` (`PlayerDrawHelper.SetShaderForData`,
    /// `PlayerDrawHelper.cs:29-60`). `dye` is the 1-based `dye_params()` slot
    /// (0 = plain).
    pub fn push_sprite_dye(
        &mut self,
        pos: [f32; 2],
        size: [f32; 2],
        uv: [f32; 4],
        color: [f32; 4],
        rot: f32,
        dye: u32,
    ) {
        self.world.push(SpriteInstance {
            pos,
            size,
            uv0: [uv[0], uv[1]],
            uv1: [uv[2], uv[3]],
            color,
            rot,
            paint: 0.0,
            dye,
            layer: 0,
            clip: self.clip.current(),
            blend: 0,
        });
    }

    /// Rotated screen-space sprite in an explicit blend batch (S1-G-3): the
    /// C#'s `DrawProj_LightsBane` re-begins the batch with the
    /// reverse-subtract "multiply" state and later `BlendState.Additive`
    /// (`Main.cs:27457-27494`); the port pushes those quads into the same
    /// range the other projectile quads ride (`push_ui_sprite_rot`, screen
    /// pixels) tagged with the mode, and `draw_ranges` splits the range into
    /// contiguous blend runs. Follows [`Self::push_sprite_origin_rot`]'s
    /// convention exactly: `pos` is the UNROTATED top-left — for a C#
    /// `Draw(position, …, origin, scale)` call, `pos = anchor − origin·scale`
    /// (the `foliage.rs::palm_placement` precedent) — and `origin` is the sprite-local pivot
    /// already in scaled screen pixels. The top-left is shifted by
    /// `origin − R(rot)·origin`, so the origin texel lands on `pos + origin`
    /// (the C#'s `position` anchor).
    pub fn push_ui_sprite_blend(
        &mut self,
        pos: [f32; 2],
        size: [f32; 2],
        uv: [f32; 4],
        color: [f32; 4],
        rot: f32,
        origin: [f32; 2],
        blend: BlendMode,
    ) {
        let (s, c) = rot.sin_cos();
        let tl = [
            pos[0] + origin[0] - (origin[0] * c - origin[1] * s),
            pos[1] + origin[1] - (origin[0] * s + origin[1] * c),
        ];
        self.ui.push(SpriteInstance {
            pos: tl,
            size,
            uv0: [uv[0], uv[1]],
            uv1: [uv[2], uv[3]],
            color,
            rot,
            paint: 0.0,
            dye: 0,
            layer: 0,
            clip: self.clip.current(),
            blend: blend as u32,
        });
    }

    /// Uploads the dye uniform table (`dye::dye_params()`'s rows on the
    /// tr-app side); call once after the renderer is created. Slots beyond
    /// `params.len()` stay zeroed (mode 0 = the shader's plain fallback).
    pub fn set_dye_params(&mut self, gpu: &Gpu, params: &[DyeParam]) {
        assert!(params.len() <= DYE_TABLE_LEN, "dye table overflow");
        let mut table = [DyeParam::zeroed(); DYE_TABLE_LEN];
        table[..params.len()].copy_from_slice(params);
        gpu.queue
            .write_buffer(&self.dye_table, 0, bytemuck::cast_slice(&table));
    }

    /// Rotated world sprite. Used by the held item, which the original draws with
    /// `ItemRotation` around the hand.
    pub fn push_sprite_rot(
        &mut self,
        pos: [f32; 2],
        size: [f32; 2],
        uv: [f32; 4],
        color: [f32; 4],
        rot: f32,
    ) {
        self.world.push(SpriteInstance {
            pos,
            size,
            uv0: [uv[0], uv[1]],
            uv1: [uv[2], uv[3]],
            color,
            rot,
            paint: 0.0,
            dye: 0,
            layer: 0,
            clip: self.clip.current(),
            blend: 0,
        });
    }

    /// Rotated screen-space sprite. Used by rain, which the original draws rotated by
    /// `atan2(velocity.x, -velocity.y)` around the top-left origin. Rain lives in the
    /// `SpriteBatch.Draw` with an arbitrary origin: `pos` is the unrotated
    /// top-left and `origin` the sprite-local anchor the rotation keeps in
    /// place. The instance pipeline rotates about the top-left, so the
    /// top-left is shifted by `origin − R(rot)·origin`.
    pub fn push_sprite_origin_rot(
        &mut self,
        pos: [f32; 2],
        size: [f32; 2],
        uv: [f32; 4],
        color: [f32; 4],
        rot: f32,
        origin: [f32; 2],
    ) {
        let (s, c) = rot.sin_cos();
        let tl = [
            pos[0] + origin[0] - (origin[0] * c - origin[1] * s),
            pos[1] + origin[1] - (origin[0] * s + origin[1] * c),
        ];
        self.push_sprite_rot(tl, size, uv, color, rot);
    }

    /// `push_sprite_origin_rot` with a `PaintID` — the vine-strip segments
    /// (`TileDrawing.DrawVineStrip`'s `DrawNature`, `TileDrawing.cs:9124`)
    /// draw through the paint system's recolored texture AND rotate about
    /// the segment origin; the port folds both into one instance.
    #[allow(clippy::too_many_arguments)]
    pub fn push_sprite_origin_rot_paint(
        &mut self,
        pos: [f32; 2],
        size: [f32; 2],
        uv: [f32; 4],
        color: [f32; 4],
        rot: f32,
        origin: [f32; 2],
        paint: u8,
    ) {
        let (s, c) = rot.sin_cos();
        let tl = [
            pos[0] + origin[0] - (origin[0] * c - origin[1] * s),
            pos[1] + origin[1] - (origin[0] * s + origin[1] * c),
        ];
        self.push_sprite_paint(tl, size, uv, color, rot, paint);
    }

    /// always-rebuilt UI range so the cached world range (G6) stays reusable.
    pub fn push_ui_sprite_rot(
        &mut self,
        pos: [f32; 2],
        size: [f32; 2],
        uv: [f32; 4],
        color: [f32; 4],
        rot: f32,
    ) {
        self.ui.push(SpriteInstance {
            pos,
            size,
            uv0: [uv[0], uv[1]],
            uv1: [uv[2], uv[3]],
            color,
            rot,
            paint: 0.0,
            dye: 0,
            layer: 0,
            clip: self.clip.current(),
            blend: 0,
        });
    }

    /// Screen-space world-effect sprite (rain, snow, sand dust, leaf gores):
    /// the UI camera transform, drawn before the map batches and the interface.
    pub fn push_fx_sprite_rot(
        &mut self,
        pos: [f32; 2],
        size: [f32; 2],
        uv: [f32; 4],
        color: [f32; 4],
        rot: f32,
    ) {
        self.fx.push(SpriteInstance {
            pos,
            size,
            uv0: [uv[0], uv[1]],
            uv1: [uv[2], uv[3]],
            color,
            rot,
            paint: 0.0,
            dye: 0,
            layer: 0,
            clip: self.clip.current(),
            blend: 0,
        });
    }

    /// Solid-colour rectangle in the world-effect range (the rain flash).
    pub fn push_fx_rect(&mut self, pos: [f32; 2], size: [f32; 2], color: [f32; 4]) {
        self.fx.push(SpriteInstance {
            pos,
            size,
            uv0: [self.white_uv[0], self.white_uv[1]],
            uv1: [self.white_uv[2], self.white_uv[3]],
            color,
            rot: 0.0,
            paint: 0.0,
            dye: 0,
            layer: 0,
            clip: self.clip.current(),
            blend: 0,
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
            rot: 0.0,
            paint: 0.0,
            dye: 0,
            layer: 0,
            clip: self.clip.current(),
            blend: 0,
        });
    }

    /// Textured sprite in screen space (hotbar icons).
    pub fn push_ui_sprite(&mut self, pos: [f32; 2], size: [f32; 2], uv: [f32; 4], color: [f32; 4]) {
        self.ui.push(SpriteInstance {
            pos,
            size,
            uv0: [uv[0], uv[1]],
            uv1: [uv[2], uv[3]],
            color,
            rot: 0.0,
            paint: 0.0,
            dye: 0,
            layer: 0,
            clip: self.clip.current(),
            blend: 0,
        });
    }

    /// One `DynamicSpriteFont` glyph in screen space: `uv` addresses the packed
    /// page inside the font texture array (`font_atlas::PageRect.layer`),
    /// already inset by the half texel the main-atlas path applies in the
    /// shader. The layer value is the 1-based instance layer (0 = main atlas).
    pub fn push_font_glyph(
        &mut self,
        pos: [f32; 2],
        size: [f32; 2],
        uv: [f32; 4],
        color: [f32; 4],
        layer: u32,
    ) {
        self.ui.push(SpriteInstance {
            pos,
            size,
            uv0: [uv[0], uv[1]],
            uv1: [uv[2], uv[3]],
            color,
            rot: 0.0,
            paint: 0.0,
            dye: 0,
            layer,
            clip: self.clip.current(),
            blend: 0,
        });
    }

    /// Screen-space atlas sprite drawn *behind* the map (biome `MapBG` backdrop —
    /// Task 7 — and the `Images/Map` scroll frame).
    pub fn push_map_back_sprite(
        &mut self,
        pos: [f32; 2],
        size: [f32; 2],
        uv: [f32; 4],
        color: [f32; 4],
    ) {
        self.map_back.push(SpriteInstance {
            pos,
            size,
            uv0: [uv[0], uv[1]],
            uv1: [uv[2], uv[3]],
            color,
            rot: 0.0,
            paint: 0.0,
            dye: 0,
            layer: 0,
            clip: self.clip.current(),
            blend: 0,
        });
    }

    /// Screen-space sprite sampling the map texture (the per-tile map quad).
    pub fn push_map_sprite(
        &mut self,
        pos: [f32; 2],
        size: [f32; 2],
        uv: [f32; 4],
        color: [f32; 4],
    ) {
        self.map.push(SpriteInstance {
            pos,
            size,
            uv0: [uv[0], uv[1]],
            uv1: [uv[2], uv[3]],
            color,
            rot: 0.0,
            paint: 0.0,
            dye: 0,
            layer: 0,
            clip: self.clip.current(),
            blend: 0,
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
                    rot: 0.0,
                    paint: 0.0,
                    dye: 0,
                    layer: 0,
                    clip: self.clip.current(),
                    blend: 0,
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
        self.back.clear();
        self.world.clear();
        self.fx.clear();
        self.map_back.clear();
        self.map.clear();
        self.ui.clear();
        self.clip.clear();
    }

    /// S1-G-13 consumers: clear ONLY the world range — one per-layer render
    /// callback's batch (`Main.spriteBatch.Begin/End` around each
    /// `RenderX`, `Main.cs:57371-57386` etc.). The layer executor renders
    /// the back/fx ranges (if any) around it; the frozen frame path never
    /// calls this.
    pub fn begin_world_batch(&mut self) {
        self.world.clear();
    }

    /// S1-G-13 consumers: clear ONLY the background range — the
    /// `RenderBackground` callback's batch (`Main.cs:55200-55218` pushes
    /// the cave backdrop into the back range via `push_back_sprite`).
    pub fn begin_back_batch(&mut self) {
        self.back.clear();
    }

    /// S1-G-13 consumers: park the frame's world range (the direct-screen
    /// entity passes) while the layer executor renders its own per-layer
    /// batches into the same range slot; [`Self::restore_world`] puts it
    /// back. The C# equivalent is the separate `spriteBatch` per callback
    /// (`Main.cs:57374-57384`) — the port multiplexes one range.
    pub fn take_world(&mut self) -> Vec<SpriteInstance> {
        std::mem::take(&mut self.world)
    }

    /// [`Self::take_world`]'s restore half.
    pub fn restore_world(&mut self, world: Vec<SpriteInstance>) {
        self.world = world;
    }

    /// S1-G-13 consumers: park the frame's background range (the sky quads)
    /// while the `RenderBackground` callback renders the cave backdrop into
    /// the same range slot (`draw_background` pushes via `push_back_sprite`).
    pub fn take_back(&mut self) -> Vec<SpriteInstance> {
        std::mem::take(&mut self.back)
    }

    /// [`Self::take_back`]'s restore half.
    pub fn restore_back(&mut self, back: Vec<SpriteInstance>) {
        self.back = back;
    }

    /// S1-G-13 consumers: set/clear the layer framebuffer size override
    /// (`WorldSceneLayerTarget.UpdateContent`'s `Main.screenWidth/Height`,
    /// `WorldSceneLayerTarget.cs:49-50`). `None` restores the surface size
    /// (the C#'s `:61-62` restore).
    pub fn set_screen_override(&mut self, size: Option<[f32; 2]>) {
        self.screen_override = size;
    }

    /// G6: clear only the UI range and keep the previous frame's world instances. The
    /// caller must have proven that camera, zoom, screen size, sky tint and world
    /// contents are unchanged (see `WorldFrameKey` in the app), otherwise the kept
    /// range is stale.
    pub fn begin_frame_keep_world(&mut self) {
        self.back.clear();
        self.fx.clear();
        self.map_back.clear();
        self.map.clear();
        self.ui.clear();
        self.clip.clear();
    }

    /// `UIElement.Draw`'s `OverflowHidden` enter (`UIElement.cs:222-229`): the
    /// new scissor is `GetClippingRectangle` — `rect` intersected with the
    /// enclosing one (`:265-268`) — and every sprite pushed until the matching
    /// [`pop_clip`](Self::pop_clip) is discarded outside it.
    ///
    /// `rect` is in framebuffer pixels with a top-left origin: the UI camera is
    /// identity (`Globals`'s `cam = screen/2`, `zoom = 1`), so
    /// `GetClippingRectangle`'s `Vector2.Transform(..., Main.UIScaleMatrix)` is
    /// the identity at UI scale 1 (`:256-258`).
    pub fn push_clip(&mut self, rect: ClipRect) {
        self.clip.push(rect);
    }

    /// `UIElement.Draw`'s restore (`:231-237`): back to the enclosing scissor.
    pub fn pop_clip(&mut self) {
        self.clip.pop();
    }

    /// The scissor every sprite pushed now is clipped to
    /// (`GraphicsDevice.ScissorRectangle`).
    pub fn clip_rect(&self) -> ClipRect {
        self.clip.current()
    }

    /// The full composite in the source order: background, world, world
    /// effects, the map batches, the interface.
    pub fn render(
        &mut self,
        gpu: &Gpu,
        view: &wgpu::TextureView,
        camera: [f32; 2],
        zoom: f32,
        time: f32,
        clear: wgpu::Color,
    ) -> RenderStats {
        self.draw_ranges(gpu, view, camera, zoom, time, Some(clear), true, true)
    }

    /// The world half of the split post path (`TR_POST=1`): the background,
    /// world and world-effect ranges, cleared — the target the filter passes
    /// read. The vanilla order puts the filters between the world draw and
    /// `DrawMap`/`DrawInterface`, so the map and the HUD paint over them.
    pub fn render_world_fx(
        &mut self,
        gpu: &Gpu,
        view: &wgpu::TextureView,
        camera: [f32; 2],
        zoom: f32,
        time: f32,
        clear: wgpu::Color,
    ) -> RenderStats {
        self.draw_ranges(gpu, view, camera, zoom, time, Some(clear), true, false)
    }

    /// The interface half of the split post path: the map batches and the UI
    /// range, loaded over the filter output (no clear).
    pub fn render_map_ui(
        &mut self,
        gpu: &Gpu,
        view: &wgpu::TextureView,
        camera: [f32; 2],
        zoom: f32,
        time: f32,
    ) -> RenderStats {
        self.draw_ranges(gpu, view, camera, zoom, time, None, false, true)
    }

    /// S1-G-13 consumers: the world range ALONE — one layer callback's
    /// batch drawn into the layer view (`RenderTiles`/`RenderWater`/etc.
    /// push into the world range, then this renders it; `Main.cs:57329-
    /// 57386`). Same mechanism as [`Self::render_back`]: back/fx are
    /// temporarily emptied so `draw_ranges` neither uploads nor draws them.
    /// `clear: None` keeps the transparent content `UpdateContent` cleared
    /// (`WorldSceneLayerTarget.cs:56`).
    pub fn render_world_only(
        &mut self,
        gpu: &Gpu,
        view: &wgpu::TextureView,
        camera: [f32; 2],
        zoom: f32,
        time: f32,
        clear: Option<wgpu::Color>,
    ) -> RenderStats {
        let back = std::mem::take(&mut self.back);
        let fx = std::mem::take(&mut self.fx);
        let stats = self.draw_ranges(gpu, view, camera, zoom, time, clear, true, false);
        self.back = back;
        self.fx = fx;
        stats
    }

    /// S1-G-13 consumers: the background range alone (the sky half of the
    /// gated layer path — the skyTarget bind renders the sky quads into the
    /// sky target while the world/fx ranges stay parked for the screenTarget
    /// composite). Mechanically `draw_ranges` with the world/fx ranges
    /// temporarily emptied — the upload offsets and draw order are the same
    /// code the frozen path runs; no existing caller changes.
    pub fn render_back(
        &mut self,
        gpu: &Gpu,
        view: &wgpu::TextureView,
        camera: [f32; 2],
        zoom: f32,
        time: f32,
        clear: Option<wgpu::Color>,
    ) -> RenderStats {
        let world = std::mem::take(&mut self.world);
        let fx = std::mem::take(&mut self.fx);
        let stats = self.draw_ranges(gpu, view, camera, zoom, time, clear, true, false);
        self.world = world;
        self.fx = fx;
        stats
    }

    /// S1-G-13 consumers: the world + world-effect ranges WITHOUT the
    /// background range (the screenTarget composite draws the sky/backWater/
    /// background halves first; the direct-screen entity passes follow).
    /// Same mechanism as [`Self::render_back`]: the back range is temporarily
    /// emptied so `draw_ranges` neither uploads nor draws it.
    pub fn render_world_fx_no_back(
        &mut self,
        gpu: &Gpu,
        view: &wgpu::TextureView,
        camera: [f32; 2],
        zoom: f32,
        time: f32,
        clear: Option<wgpu::Color>,
    ) -> RenderStats {
        let back = std::mem::take(&mut self.back);
        let stats = self.draw_ranges(gpu, view, camera, zoom, time, clear, true, false);
        self.back = back;
        stats
    }

    fn draw_ranges(
        &mut self,
        gpu: &Gpu,
        view: &wgpu::TextureView,
        camera: [f32; 2],
        zoom: f32,
        time: f32,
        clear: Option<wgpu::Color>,
        world_half: bool,
        ui_half: bool,
    ) -> RenderStats {
        let mut stats = RenderStats::default();
        // S1-G-13 consumers: the layer callbacks override the framebuffer
        // size (the padded layer texture, `WorldSceneLayerTarget.cs:49`);
        // `None` keeps the surface size — the frozen default.
        let (w, h) = match self.screen_override {
            Some([w, h]) => (w, h),
            None => (gpu.config.width as f32, gpu.config.height as f32),
        };

        gpu.queue.write_buffer(
            &self.globals_world,
            0,
            bytemuck::bytes_of(&Globals {
                screen: [w, h],
                cam: camera,
                zoom,
                time,
                aberration: self.aberration,
                _pad: 0.0,
                texel: [
                    1.0 / self.atlas_size.0 as f32,
                    1.0 / self.atlas_size.1 as f32,
                ],
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
                aberration: 0.0,
                _pad: 0.0,
                texel: [
                    1.0 / self.atlas_size.0 as f32,
                    1.0 / self.atlas_size.1 as f32,
                ],
            }),
        );

        let total = self.back.len()
            + self.world.len()
            + self.fx.len()
            + self.map_back.len()
            + self.map.len()
            + self.ui.len();
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
        // into one buffer at their final offsets (80-byte instances — four
        // pairs of floats, the 16-byte color, three scalars, the `layer` word
        // and the 16-byte `clip` rect — which is a multiple of 4, so the
        // per-range `write_buffer` copies stay valid). The
        // offsets follow the full concatenated layout in both halves of the split
        // post path — a half only uploads (and draws) its own ranges.
        let idx = self.frame_index;
        self.frame_index = (self.frame_index + 1) % RING;
        let buffer = &self.instances[idx];
        let back_bytes = std::mem::size_of_val(self.back.as_slice());
        let world_bytes = std::mem::size_of_val(self.world.as_slice());
        let fx_bytes = std::mem::size_of_val(self.fx.as_slice());
        let map_back_bytes = std::mem::size_of_val(self.map_back.as_slice());
        let map_bytes = std::mem::size_of_val(self.map.as_slice());
        let mut uploaded = 0u64;
        if world_half && !self.back.is_empty() {
            gpu.queue
                .write_buffer(buffer, 0, bytemuck::cast_slice(&self.back));
            uploaded += back_bytes as u64;
        }
        if world_half && !self.world.is_empty() {
            gpu.queue
                .write_buffer(buffer, back_bytes as u64, bytemuck::cast_slice(&self.world));
            uploaded += world_bytes as u64;
        }
        if world_half && !self.fx.is_empty() {
            gpu.queue.write_buffer(
                buffer,
                (back_bytes + world_bytes) as u64,
                bytemuck::cast_slice(&self.fx),
            );
            uploaded += fx_bytes as u64;
        }
        if ui_half && !self.map_back.is_empty() {
            gpu.queue.write_buffer(
                buffer,
                (back_bytes + world_bytes + fx_bytes) as u64,
                bytemuck::cast_slice(&self.map_back),
            );
            uploaded += map_back_bytes as u64;
        }
        if ui_half && !self.map.is_empty() {
            gpu.queue.write_buffer(
                buffer,
                (back_bytes + world_bytes + fx_bytes + map_back_bytes) as u64,
                bytemuck::cast_slice(&self.map),
            );
            uploaded += map_bytes as u64;
        }
        if ui_half && !self.ui.is_empty() {
            gpu.queue.write_buffer(
                buffer,
                (back_bytes + world_bytes + fx_bytes + map_back_bytes + map_bytes) as u64,
                bytemuck::cast_slice(&self.ui),
            );
            uploaded += std::mem::size_of_val(self.ui.as_slice()) as u64;
        }
        stats.uploaded_bytes = uploaded;
        stats.instances = total as u32;

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });

        {
            let load = match clear {
                Some(c) => wgpu::LoadOp::Clear(c),
                None => wgpu::LoadOp::Load,
            };
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
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
            pass.set_vertex_buffer(0, buffer.slice(..));
            if world_half && !self.back.is_empty() {
                pass.set_bind_group(0, &self.bind_world, &[]);
                stats.draw_calls += draw_blend_runs(&mut pass, &self.pipelines, &self.back, 0);
            }
            if world_half && !self.world.is_empty() {
                let start = self.back.len() as u32;
                pass.set_bind_group(0, &self.bind_world, &[]);
                stats.draw_calls += draw_blend_runs(&mut pass, &self.pipelines, &self.world, start);
            }
            // The world-effect range: the UI camera transform, the world draw
            // order — under the map batches and the interface.
            if world_half && !self.fx.is_empty() {
                let start = (self.back.len() + self.world.len()) as u32;
                pass.set_bind_group(0, &self.bind_ui, &[]);
                stats.draw_calls += draw_blend_runs(&mut pass, &self.pipelines, &self.fx, start);
            }
            if ui_half && !self.map_back.is_empty() {
                let start = (self.back.len() + self.world.len() + self.fx.len()) as u32;
                pass.set_bind_group(0, &self.bind_ui, &[]);
                stats.draw_calls +=
                    draw_blend_runs(&mut pass, &self.pipelines, &self.map_back, start);
            }
            if ui_half && !self.map.is_empty() {
                if let Some(bind_map) = &self.bind_map {
                    let start =
                        (self.back.len() + self.world.len() + self.fx.len() + self.map_back.len())
                            as u32;
                    pass.set_bind_group(0, bind_map, &[]);
                    stats.draw_calls +=
                        draw_blend_runs(&mut pass, &self.pipelines, &self.map, start);
                }
            }
            if ui_half && !self.ui.is_empty() {
                let start = (self.back.len()
                    + self.world.len()
                    + self.fx.len()
                    + self.map_back.len()
                    + self.map.len()) as u32;
                pass.set_bind_group(0, &self.bind_ui, &[]);
                stats.draw_calls += draw_blend_runs(&mut pass, &self.pipelines, &self.ui, start);
            }
        }

        gpu.queue.submit(Some(encoder.finish()));
        stats
    }
}

/// One `pass.draw` per contiguous same-[`BlendMode`] run of a range (S1-G-3):
/// the C# wraps the additive / reverse-subtract batches in
/// `End()`/`Begin(blendState)` pairs (`Main.cs:27457-27494`), which flush the
/// batch in push order; the port's ranges keep push order too, so a run
/// boundary is exactly the C#'s batch boundary. An all-alpha range (every
/// range but the Lightsbane quads today) stays a single draw — the default
/// path is unchanged. Returns the number of draws issued.
fn draw_blend_runs(
    pass: &mut wgpu::RenderPass<'_>,
    pipelines: &[wgpu::RenderPipeline; 3],
    range: &[SpriteInstance],
    start: u32,
) -> u32 {
    let mut draws = 0;
    for (mode, run_start, run_end) in blend_runs(range) {
        pass.set_pipeline(&pipelines[(mode as usize).min(pipelines.len() - 1)]);
        pass.draw(0..6, (start + run_start)..(start + run_end));
        draws += 1;
    }
    draws
}

/// The contiguous same-blend runs of a range as `(mode, start, end)` index
/// triples — pure so the batch-split contract is unit-testable without a
/// GPU. An all-alpha range yields exactly one run (the pre-S1-G-3 single
/// draw); the Lightsbane sequence alpha…multiply×6, additive×2, alpha…
/// (`Main.cs:27483-27492`) yields one run per C# `Begin` batch.
fn blend_runs(range: &[SpriteInstance]) -> Vec<(u32, u32, u32)> {
    let mut runs = Vec::new();
    let mut run_start = 0usize;
    while run_start < range.len() {
        let mode = range[run_start].blend;
        let mut run_end = run_start + 1;
        while run_end < range.len() && range[run_end].blend == mode {
            run_end += 1;
        }
        runs.push((mode, run_start as u32, run_end as u32));
        run_start = run_end;
    }
    runs
}

/// `TR_PROBE=x,y`: log every sprite whose rect covers that world point, in draw
/// order, to find which pass paints a stray pixel.
fn probe(pos: [f32; 2], size: [f32; 2], uv: [f32; 4], color: [f32; 4]) {
    static PROBE: std::sync::OnceLock<Option<[f32; 2]>> = std::sync::OnceLock::new();
    let p = PROBE.get_or_init(|| {
        std::env::var("TR_PROBE").ok().and_then(|v| {
            let mut it = v.split(',');
            let x: f32 = it.next()?.parse().ok()?;
            let y: f32 = it.next()?.parse().ok()?;
            Some([x, y])
        })
    });
    if let Some(p) = *p {
        if p[0] >= pos[0] && p[0] < pos[0] + size[0] && p[1] >= pos[1] && p[1] < pos[1] + size[1] {
            eprintln!(
                "PROBE pos {:.1},{:.1} size {:.1}x{:.1} uv {:.4},{:.4}-{:.4},{:.4} color {:.2},{:.2},{:.2},{:.2}",
                pos[0],
                pos[1],
                size[0],
                size[1],
                uv[0],
                uv[1],
                uv[2],
                uv[3],
                color[0],
                color[1],
                color[2],
                color[3]
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// fix/player-motion-blur — the sampler-state pin: the atlas sampler is
    /// XNA `PointClamp` (`Camera.cs:44` → `Main.DefaultSamplerState`,
    /// `Main.cs:2774-2784`; the player batch begins with it,
    /// `LegacyPlayerRenderer.cs:264-269`). Nearest filtering + whole-pixel
    /// screen positions = pixel-perfect entities; a linear min/mag here would
    /// smear every sprite at fractional positions.
    #[test]
    fn atlas_sampler_is_point_clamp() {
        let d = atlas_sampler_descriptor();
        assert_eq!(d.mag_filter, wgpu::FilterMode::Nearest, "PointClamp mag");
        assert_eq!(d.min_filter, wgpu::FilterMode::Nearest, "PointClamp min");
        assert_eq!(
            d.mipmap_filter,
            wgpu::MipmapFilterMode::Nearest,
            "PointClamp mip"
        );
        assert_eq!(d.address_mode_u, wgpu::AddressMode::ClampToEdge);
        assert_eq!(d.address_mode_v, wgpu::AddressMode::ClampToEdge);
    }

    // ---- S1-G-4: the tint's colour-space parity (research/41 D-5) ----

    /// The piecewise sRGB EOTF the GPU applies to an `Rgba8UnormSrgb` view —
    /// the same curve the WGSL helpers implement (`sprite.wgsl`). Kept here so
    /// the parity test and the shader cannot drift apart silently.
    fn srgb_decode(c: f32) -> f32 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    /// The EOTF's exact inverse (`IEC 61966-2-1`).
    fn srgb_encode(c: f32) -> f32 {
        if c <= 0.0031308 {
            c * 12.92
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        }
    }

    /// `encode(decode(b))` returns every stored byte exactly — the contract
    /// the shader's colour algebra relies on (the atlas decodes with this
    /// curve and the swapchain encodes with its inverse).
    #[test]
    fn srgb_eotf_round_trips_every_byte() {
        for b in 0u32..256 {
            let v = b as f32 / 255.0;
            let round = (srgb_encode(srgb_decode(v)) * 255.0).round();
            assert_eq!(round, b as f32, "byte {b} did not round-trip");
        }
    }

    /// The verify criterion: the port's displayed byte must equal the
    /// vanilla's SpriteBatch tint exactly. XNA's formats are plain `Color`
    /// (no sRGB decode), so the vanilla multiplies sRGB bytes in gamma space
    /// and stores the product raw: `display = tex_byte * tint_byte / 255`.
    /// The port's sRGB atlas + sRGB swapchain pair re-derive that product as
    /// `encode(decode(encode(tex) * tint))` — since `encode ∘ decode` is the
    /// identity, the displayed byte lands on the vanilla's without drift.
    /// The pre-S1-G-4 forms (linearising the tint alone, either `pow(2.2)`
    /// or the piecewise decode) solved the wrong equation and drifted.
    #[test]
    fn tint_parity_with_the_csharp_byte_multiply() {
        let mut max_exact = 0u32;
        let mut max_exact_int = 0u32;
        let mut max_pow22 = 0u32;
        let mut max_decode_both = 0u32;
        for tex in 0u32..256 {
            let tex_lin = srgb_decode(tex as f32 / 255.0);
            let tex_srgb = tex as f32 / 255.0;
            for tint in 0u32..256 {
                let tint_srgb = tint as f32 / 255.0;
                // The S1-G-4 fragment: the gamma-space product, linearised once.
                let fragment = srgb_decode(tex_srgb * tint_srgb);
                let exact = (srgb_encode(fragment) * 255.0).round() as i32;
                // The vanilla reference: the SpriteBatch tint reaches the GPU
                // as floats and the non-sRGB backbuffer rounds the store —
                // `round(tex*tint/255)`. The CPU-side `Color * Color`
                // operator instead truncates (`(tex*tint)/255`); the port
                // pins both readings.
                let vanilla_float = (tex_srgb * tint_srgb * 255.0).round() as i32;
                let vanilla_int = (tex * tint) / 255;
                max_exact = max_exact.max((exact - vanilla_float).unsigned_abs());
                max_exact_int = max_exact_int.max((exact - vanilla_int as i32).unsigned_abs());
                // The pre-S1-G-4 forms for contrast.
                let pow22 = (srgb_encode(tex_lin * tint_srgb.powf(2.2)) * 255.0).round() as i32;
                max_pow22 = max_pow22.max((pow22 - vanilla_float).unsigned_abs());
                let both = (srgb_encode(tex_lin * srgb_decode(tint_srgb)) * 255.0).round() as i32;
                max_decode_both = max_decode_both.max((both - vanilla_float).unsigned_abs());
            }
        }
        assert_eq!(max_exact, 0, "the gamma-product form must be exact");
        assert!(
            max_exact_int <= 1,
            "the integer-division reading drifted {max_exact_int} LSB"
        );
        assert!(
            max_pow22 > max_exact,
            "pow(2.2) was no worse than the exact form ({max_pow22})"
        );
        assert!(
            max_decode_both > max_exact,
            "the decode-both form was no worse either ({max_decode_both})"
        );
    }

    /// `sprite.wgsl` stays naga-valid after the S1-G-4 colour-space edits —
    /// `create_shader_module` panics on invalid WGSL, so a broken shader
    /// fails here instead of at the first frame.
    #[test]
    fn sprite_wgsl_compiles_headless() {
        let Some((device, _queue)) = crate::test_gpu::headless() else {
            return;
        };
        let _ = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sprite-wgsl-parity-test"),
            source: wgpu::ShaderSource::Wgsl(include_str!("sprite.wgsl").into()),
        });
    }

    #[test]
    fn the_instance_stride_is_a_multiple_of_four() {
        // `draw_ranges` copies each range straight into the instance ring at
        // `range_index * size_of::<SpriteInstance>()` (no per-range staging
        // buffer), so the stride must be 4-byte aligned for `write_buffer` —
        // wgpu's `COPY_BUFFER_ALIGNMENT`. 84 today: 2+2+2+2 floats (32), color
        // (16), rot/paint (8), dye/layer (8), clip (16), blend (4, S1-G-3).
        let stride = std::mem::size_of::<SpriteInstance>();
        assert_eq!(
            stride % wgpu::COPY_BUFFER_ALIGNMENT as usize,
            0,
            "stride {stride}"
        );
        // And the vertex layout's `array_stride` is that same stride, so the
        // shader's `Instance` struct and the CPU struct cannot drift apart
        // without this failing first.
        assert_eq!(stride, 84, "the fields documented above add up to 84 bytes");
    }

    #[test]
    fn the_shader_locations_cover_every_instance_field() {
        // The vertex attributes declared in `new` (`vertex_attr_array!`) map
        // 1:1 onto `sprite.wgsl`'s `Instance`, and the last one is the scissor
        // rect (`@location(9)`, `clip`). Attribute order encodes the offsets,
        // so the two lists must not be reordered independently.
        let attrs = wgpu::vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
            2 => Float32x2,
            3 => Float32x2,
            4 => Float32x4,
            5 => Float32,
            6 => Float32,
            7 => Uint32,
            8 => Uint32,
            9 => Float32x4,
            10 => Uint32,
        ];
        let total: u64 = attrs.iter().map(|a| a.format.size()).sum();
        assert_eq!(total, std::mem::size_of::<SpriteInstance>() as u64);
        assert_eq!(attrs[9].shader_location, 9);
        assert_eq!(attrs[9].format, wgpu::VertexFormat::Float32x4);
        assert_eq!(attrs[10].shader_location, 10);
        assert_eq!(attrs[10].format, wgpu::VertexFormat::Uint32);
    }

    // ---- S1-G-3: the blend batches (research/41 D-2, Main.cs:27457-27494) ----

    /// The three blend states are 1:1 with the C# `BlendState` fields: the
    /// default alpha batch keeps its premultiplied factors (unchanged from
    /// pre-S1-G-3), Additive is XNA's stock `(SourceAlpha, One)` Add, and
    /// Multiply is `Main._multiplyBlendState`'s ReverseSubtract with
    /// SourceAlpha/One on both channels (`Main.cs:27462-27470`).
    #[test]
    fn blend_states_match_the_csharp_fields() {
        let alpha = BlendMode::AlphaBlend.blend_state();
        for c in [alpha.color, alpha.alpha] {
            assert_eq!(c.src_factor, wgpu::BlendFactor::One);
            assert_eq!(c.dst_factor, wgpu::BlendFactor::OneMinusSrcAlpha);
            assert_eq!(c.operation, wgpu::BlendOperation::Add);
        }
        let add = BlendMode::Additive.blend_state();
        for c in [add.color, add.alpha] {
            assert_eq!(c.src_factor, wgpu::BlendFactor::SrcAlpha);
            assert_eq!(c.dst_factor, wgpu::BlendFactor::One);
            assert_eq!(c.operation, wgpu::BlendOperation::Add);
        }
        let mul = BlendMode::Multiply.blend_state();
        for c in [mul.color, mul.alpha] {
            assert_eq!(c.src_factor, wgpu::BlendFactor::SrcAlpha);
            assert_eq!(c.dst_factor, wgpu::BlendFactor::One);
            assert_eq!(c.operation, wgpu::BlendOperation::ReverseSubtract);
        }
        // The instance tag is the pipeline index — the discriminants must
        // stay 0/1/2 or `draw_blend_runs` picks the wrong pipeline.
        assert_eq!(BlendMode::AlphaBlend as u32, 0);
        assert_eq!(BlendMode::Additive as u32, 1);
        assert_eq!(BlendMode::Multiply as u32, 2);
    }

    /// The batch-split contract: an all-alpha range stays ONE draw (the
    /// pre-S1-G-3 default path is unchanged); the Lightsbane push sequence —
    /// alpha content, six multiply quads, two additive quads, alpha content
    /// (`Main.cs:27483-27494`) — splits into one run per C# `Begin` batch,
    /// in push order.
    #[test]
    fn blend_runs_split_only_at_batch_boundaries() {
        let inst = |blend: u32| SpriteInstance {
            pos: [0.0; 2],
            size: [0.0; 2],
            uv0: [0.0; 2],
            uv1: [0.0; 2],
            color: [1.0; 4],
            rot: 0.0,
            paint: 0.0,
            dye: 0,
            layer: 0,
            clip: crate::clip::NO_CLIP,
            blend,
        };
        let plain: Vec<SpriteInstance> = (0..7).map(|_| inst(0)).collect();
        assert_eq!(blend_runs(&plain), vec![(0, 0, 7)]);
        assert!(blend_runs(&[]).is_empty());
        let mut seq = vec![inst(0), inst(0)];
        seq.extend((0..6).map(|_| inst(BlendMode::Multiply as u32)));
        seq.extend((0..2).map(|_| inst(BlendMode::Additive as u32)));
        seq.push(inst(0));
        assert_eq!(
            blend_runs(&seq),
            vec![(0, 0, 2), (2, 2, 8), (1, 8, 10), (0, 10, 11)]
        );
    }

    /// The GPU blend equations, pinned against the C# on both an sRGB
    /// target (the swapchain convention) and a plain UNORM target (the
    /// vanilla's non-sRGB backbuffer, where the blend maths runs on the
    /// stored bytes directly): a known source over a known destination must
    /// land on `dst + src·srcAlpha` (Additive, `Main.cs:27490`) and
    /// `dst − src·srcAlpha` clamped at 0 (Multiply, `Main.cs:27462-27470`),
    /// while the alpha-blend default keeps its `(One, OneMinusSrcAlpha)`
    /// equation. Tolerance 1 LSB for the sRGB encode/decode round-trip.
    #[test]
    fn blend_equations_land_on_the_csharp_batches() {
        let Some((device, queue)) = crate::test_gpu::headless() else {
            return;
        };
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("blend-test"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
@vertex fn vs(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4<f32> {
    var p = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    return vec4<f32>(p[vi], 0.0, 1.0);
}
@fragment fn fs() -> @location(0) vec4<f32> {
    return vec4<f32>(0.5, 0.25, 0.75, 0.5);
}
"#
                .into(),
            ),
        });
        // src = (0.5, 0.25, 0.75, 0.5), dst = (0.6, 0.6, 0.6, 1.0).
        let src = [0.5f32, 0.25, 0.75, 0.5];
        let dst = [0.6f32, 0.6, 0.6, 1.0];
        for format in [
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Rgba8Unorm,
        ] {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                immediate_size: 0,
            });
            for mode in [
                BlendMode::AlphaBlend,
                BlendMode::Additive,
                BlendMode::Multiply,
            ] {
                let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("blend-test-pipeline"),
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs"),
                        buffers: &[],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format,
                            blend: Some(mode.blend_state()),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview_mask: None,
                    cache: None,
                });
                let tex = crate::gpu::offscreen_target_on(&device, format, 4, 4, "blend-test");
                let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
                // The destination as stored bytes: on the sRGB target the
                // clear value is linear (wgpu encodes it on store); on UNORM
                // it is the raw value, like the vanilla's backbuffer.
                let clear = if format == wgpu::TextureFormat::Rgba8UnormSrgb {
                    wgpu::Color {
                        r: srgb_decode(dst[0]) as f64,
                        g: srgb_decode(dst[1]) as f64,
                        b: srgb_decode(dst[2]) as f64,
                        a: dst[3] as f64,
                    }
                } else {
                    wgpu::Color {
                        r: dst[0] as f64,
                        g: dst[1] as f64,
                        b: dst[2] as f64,
                        a: dst[3] as f64,
                    }
                };
                let mut enc =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
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
                    pass.set_pipeline(&pipeline);
                    pass.draw(0..3, 0..1);
                }
                queue.submit(Some(enc.finish()));
                let px = crate::test_gpu::read_texture(&device, &queue, &tex, (4, 4), 4);
                // The expected bytes: the blend runs in the target's working
                // space — linear on sRGB (the fragment output IS linear, the
                // stored dst decodes, the result re-encodes), raw on UNORM
                // (the vanilla's own backbuffer convention). The source is
                // the shader's output in both cases — never decoded again.
                let s = src;
                let d = if format == wgpu::TextureFormat::Rgba8UnormSrgb {
                    [
                        srgb_decode(dst[0]),
                        srgb_decode(dst[1]),
                        srgb_decode(dst[2]),
                        dst[3],
                    ]
                } else {
                    dst
                };
                let expect: [f32; 4] = match mode {
                    BlendMode::AlphaBlend => [
                        s[0] + d[0] * (1.0 - s[3]),
                        s[1] + d[1] * (1.0 - s[3]),
                        s[2] + d[2] * (1.0 - s[3]),
                        s[3] + d[3] * (1.0 - s[3]),
                    ],
                    BlendMode::Additive => [
                        d[0] + s[0] * s[3],
                        d[1] + s[1] * s[3],
                        d[2] + s[2] * s[3],
                        (d[3] + s[3] * s[3]).min(1.0),
                    ],
                    BlendMode::Multiply => [
                        (d[0] - s[0] * s[3]).max(0.0),
                        (d[1] - s[1] * s[3]).max(0.0),
                        (d[2] - s[2] * s[3]).max(0.0),
                        (d[3] - s[3] * s[3]).max(0.0),
                    ],
                };
                for ch in 0..4 {
                    // The fixed-function blend clamps the result to [0, 1].
                    let e = expect[ch].clamp(0.0, 1.0);
                    let want = if format == wgpu::TextureFormat::Rgba8UnormSrgb && ch < 3 {
                        (srgb_encode(e) * 255.0).round() as i32
                    } else {
                        (e * 255.0).round() as i32
                    };
                    let got = px[ch] as i32;
                    assert!(
                        (got - want).abs() <= 1,
                        "{mode:?} on {format:?} channel {ch}: got {got}, want {want}"
                    );
                }
            }
        }
    }
}
