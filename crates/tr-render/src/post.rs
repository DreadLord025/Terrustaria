//! Offscreen scene target + fullscreen filter pass — the post-process infrastructure
//! for screen filters (M1.4-BG-18 phase 2). Vanilla drives `Filters.Scene["…"]` from
//! `SceneState.ApplyVisuals` (`SceneState.cs:64-140`): named passes of one
//! `ScreenShader.fx` effect, parameterised per `ScreenShaderData`.
//!
//! Plan-contract mapping (plan §Phase 2 "Produces" — equivalent structure, allowed
//! by the plan because the app keeps owning the renderer, the swapchain frame and
//! the present/shot-capture flow):
//! - `resize(size)` → [`PostPipeline::resize`]
//! - `render_scene_to_target(draw: impl FnOnce(&mut SpriteRenderer))` →
//!   [`PostPipeline::scene_view`] + the caller's existing [`SpriteRenderer::render`]
//!   (its `view` parameter at `sprite.rs:630` is the plan's "sprite.rs
//!   (цель-параметр)"; PostPipeline does not own the renderer, so the draw closure
//!   form would add an ownership move without hiding anything)
//! - `apply_filter(params, pass, noise)` → [`PostPipeline::apply_filter`], with
//!   [`FilterImages`] as the extensible resource parameter (phase 4 adds the
//!   `water_mask` slot without touching the signature)
//! - `present()` → stays in the app (`frame.rs` presents the frame it owns; the
//!   TR_SHOT capture sits between filter and present, which DW-2.1 compares)
//!
//! Pixel identity (DW-2.1): the offscreen target uses the surface's sRGB format, so
//! the scene pass sRGB-encodes on store, the filter pass samples the sRGB view
//! (decode) and writes through to the sRGB swapchain (encode). The round trip is
//! byte-exact, and the passthrough (`PassId::PASSTHROUGH`, uniform `pass_index` 0)
//! is the identity function of that sample.

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::gpu::Gpu;

/// Uniform mirror of the `ScreenShaderData.Apply` parameters
/// (`ScreenShaderData.cs:139-187`), plus the WGSL pass selector. Field order is the
/// WGSL layout in `post.wgsl` — do not reorder one without the other.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct FilterParams {
    /// `uColor` (`:143`), vanilla default `Vector3.One` (`:9`).
    pub color: [f32; 4],
    /// `uSecondaryColor` (`:145`), default `One` (`:11`).
    pub secondary_color: [f32; 4],
    /// `uTargetPosition` (`:149`), default `Vector2.One` (`:19`).
    pub target_pos: [f32; 2],
    /// `uImageOffset` (`:150`), default `Zero` (`:25`).
    pub image_offset: [f32; 2],
    /// `_imageScales[0]` (`:183`), default `One` (`:39-44`).
    pub image_scale: [f32; 2],
    /// `uDirection` (`:155`), default `(0, 1)` (`:21`).
    pub direction: [f32; 2],
    /// `uScreenResolution` (`:147`) = `UnscaledScreenSize` (`:96`).
    pub screen_resolution: [f32; 2],
    /// `uScreenPosition` (`:148`) = `UnscaledScreenPosition - offScreenRange` (`:148`).
    pub screen_position: [f32; 2],
    /// `uImageSize[1]` — image dims * scale (`:183`); the noise image is slot 0
    /// of the C# `_uAssetImages` array (textures 1..4 on the device).
    pub image_size: [f32; 2],
    pub _pad0: [f32; 2],
    /// `uIntensity` (`:153`), default 1 (`:17`).
    pub intensity: f32,
    /// `uOpacity` = `CombinedOpacity` (`:144`, `:84`), default 1 (`:13`).
    pub opacity: f32,
    /// `uProgress` (`:154`), default 0 (`:23`).
    pub progress: f32,
    /// `uTime` (`:146`) = `Main.GlobalTimeWrappedHourly`.
    pub time: f32,
    /// WGSL pass selector (`post.wgsl`); vanilla picks the pass by name
    /// (`ScreenShaderData.cs:98-101`).
    pub pass_index: u32,
    /// WGSL uniform layout needs scalar tail padding (an `array<f32,3>` would take
    /// a 16-byte stride in the uniform address space).
    pub _pad1: [u32; 3],
}

impl FilterParams {
    /// The `ScreenShaderData` field defaults (`ScreenShaderData.cs:9-44`): every
    /// setter the driver (phase 3) skips keeps this value. `pass_index` 0 is the
    /// passthrough.
    pub const NEUTRAL: FilterParams = FilterParams {
        color: [1.0, 1.0, 1.0, 0.0],
        secondary_color: [1.0, 1.0, 1.0, 0.0],
        target_pos: [1.0, 1.0],
        image_offset: [0.0, 0.0],
        image_scale: [1.0, 1.0],
        direction: [0.0, 1.0],
        screen_resolution: [0.0, 0.0],
        screen_position: [0.0, 0.0],
        image_size: [0.0, 0.0],
        _pad0: [0.0, 0.0],
        intensity: 1.0,
        opacity: 1.0,
        progress: 0.0,
        time: 0.0,
        pass_index: 0,
        _pad1: [0, 0, 0],
    };
}

/// Selector for the WGSL pass function, sent in the uniform (`pass_index`).
/// Pass 0 is the pixel-identity passthrough; phase 3 wired the tint/noise
/// passes (indices 1-7, `post.wgsl`'s `fs_main` switch) and re-pointed their
/// [`PASS_TABLE`] rows.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PassId(pub u32);

impl PassId {
    pub const PASSTHROUGH: PassId = PassId(0);
    /// `FilterGraveyard` (`post.wgsl`).
    pub const GRAVEYARD: PassId = PassId(1);
    /// `FilterNoir` — the content set lacks `Images/NoirNoise`; the fallback
    /// noise binds.
    pub const NOIR: PassId = PassId(2);
    /// `FilterBloodMoon`.
    pub const BLOOD_MOON: PassId = PassId(3);
    /// `FilterSandstormForeground`.
    pub const SANDSTORM_FOREGROUND: PassId = PassId(4);
    /// `FilterSandstormBackground` — the `Overlays.Scene["Sandstorm"]` twin.
    pub const SANDSTORM_BACKGROUND: PassId = PassId(5);
    /// `FilterBlizzardForeground`.
    pub const BLIZZARD_FOREGROUND: PassId = PassId(6);
    /// `FilterBlizzardBackground` — the `Overlays.Scene["Blizzard"]` twin.
    pub const BLIZZARD_BACKGROUND: PassId = PassId(7);
}

/// Filter name → vanilla pass name → WGSL pass index. Rows are the
/// `Filters.Scene["…"]` registrations of `ScreenEffectInitializer.Load`
/// (`ScreenEffectInitializer.cs:14-48`), pass names verbatim as they appear in
/// `ScreenShader.xnb` (DW-2.4; the XNB also carries `FilterColor`, which no
/// initializer row uses). Indices: 0 = passthrough; phase 3 wired the
/// tint/noise subset (Graveyard/Noir/BloodMoon/Sandstorm/Blizzard × fg/bg) —
/// unported rows stay visually inert.
pub const PASS_TABLE: &[(&str, &str, u32)] = &[
    // (Filters.Scene key, ScreenShader.fx pass name, WGSL pass index)
    ("Nebula", "FilterTower", 0),
    ("Solar", "FilterTower", 0),
    ("Stardust", "FilterTower", 0),
    ("Vortex", "FilterTower", 0),
    ("MonolithNebula", "FilterMiniTower", 0),
    ("MonolithSolar", "FilterMiniTower", 0),
    ("MonolithStardust", "FilterMiniTower", 0),
    ("MonolithVortex", "FilterMiniTower", 0),
    ("MoonLord", "FilterMoonLord", 0),
    ("MoonLordShake", "FilterMoonLordShake", 0),
    ("MonolithMoonLord", "FilterMoonLord", 0),
    ("Graveyard", "FilterGraveyard", 1),
    ("testInvert", "FilterInvert", 0),
    ("BloodMoon", "FilterBloodMoon", 3),
    ("Sepia", "FilterSepia", 0),
    ("Noir", "FilterNoir", 2),
    ("Sandstorm", "FilterSandstormForeground", 4),
    ("SandstormOverlay", "FilterSandstormBackground", 5),
    ("Blizzard", "FilterBlizzardForeground", 6),
    ("BlizzardOverlay", "FilterBlizzardBackground", 7),
    ("HeatDistortion", "FilterHeatDistortion", 0),
    ("WaterDistortion", "FilterWaterDistortion", 0),
    (
        "CrystalDestructionColor",
        "FilterCrystalDestructionColor",
        0,
    ),
    (
        "CrystalDestructionVortex",
        "FilterCrystalDestructionVortex",
        0,
    ),
    ("CrystalWin", "FilterCrystalWin", 0),
    ("Test", "FilterTest", 0),
    ("Test2", "FilterTest2", 0),
    ("CRT", "FilterCRT", 0),
    ("Test3", "FilterTest3", 0),
];

/// Filter name → WGSL pass id. Unknown filter names run the passthrough (a filter
/// key that is not in the table must not panic the frame; phase 3's driver only
/// ever asks for keys it activates). `SandstormOverlay`/`BlizzardOverlay` stand in
/// for the `Overlays.Scene` twins of the same pass family
/// (`ScreenEffectInitializer.cs:30-39`).
pub fn pass_index(filter_name: &str) -> PassId {
    PassId(
        PASS_TABLE
            .iter()
            .find(|(name, _, _)| *name == filter_name)
            .map_or(0, |&(_, _, pass)| pass),
    )
}

/// Auxiliary images for [`PostPipeline::apply_filter`]. Phase 4 extends this struct
/// with the `water_mask` slot additively (plan: "заложить enum-параметр ресурсов" —
/// an extensible resource struct cannot break the phase-4 signature an enum arm
/// would).
#[derive(Clone, Copy, Default)]
pub struct FilterImages<'a> {
    /// Noise image bound to slot 3 (`ScreenShaderData.cs:159-185` binds up to
    /// three; every M1 filter uses only the first).
    pub noise: Option<&'a wgpu::Texture>,
}

/// True when the offscreen target must be (re)created: no target yet, or the
/// surface was resized away from it (DW-2.2 — `WindowEvent::Resized` reconfigures
/// the surface, the next `scene_view` self-heals the target).
fn target_needs_recreate(current: Option<(u32, u32)>, requested: (u32, u32)) -> bool {
    current != Some(requested)
}

/// `ScreenShaderData.cs:175-182`: power-of-two images sample with `LinearWrap`,
/// anything else with the clamping analogue. `Images/Misc/noise` is 256x256 POT.
fn noise_address_mode(width: u32, height: u32) -> wgpu::AddressMode {
    if width.is_power_of_two() && height.is_power_of_two() {
        wgpu::AddressMode::Repeat
    } else {
        wgpu::AddressMode::ClampToEdge
    }
}

/// The 8x8 checkerboard bound when a filter asks for a noise image the content
/// does not ship (plan edge case; e.g. `Images/NoirNoise` is absent from this
/// build). 2x2-cell blocks, mid-grey/green so a misbound noise is visible.
fn fallback_checkerboard() -> Vec<u8> {
    const S: usize = 8;
    let mut px = vec![0u8; S * S * 4];
    for y in 0..S {
        for x in 0..S {
            let on = (x / 2 + y / 2) % 2 == 0;
            let (r, g, b) = if on {
                (0x40, 0x40, 0x40)
            } else {
                (0xC0, 0xC0, 0xC0)
            };
            let i = (y * S + x) * 4;
            px[i..i + 4].copy_from_slice(&[r, g, b, 0xFF]);
        }
    }
    px
}

/// Post-process pipeline: offscreen scene target + fullscreen filter pass.
pub struct PostPipeline {
    /// Offscreen scene color (surface format), recreated on size change.
    target: Option<wgpu::Texture>,
    target_view: Option<wgpu::TextureView>,
    target_size: Option<(u32, u32)>,
    pipeline: wgpu::RenderPipeline,
    params_buf: wgpu::Buffer,
    /// Nearest/clamp — pixel-exact scene read (identity precondition, DW-2.1).
    scene_sampler: wgpu::Sampler,
    /// Vanilla noise sampler rule (`ScreenShaderData.cs:175-182`); the content set
    /// only ships POT noise images (noise.xnb is 256x256), so one wrap sampler
    /// covers them; a future non-POT image needs the `noise_address_mode` clamp arm.
    noise_sampler: wgpu::Sampler,
    /// Bound when a filter requests a noise image the content lacks.
    fallback_noise: (wgpu::Texture, wgpu::TextureView),
}

impl PostPipeline {
    pub fn new(gpu: &Gpu) -> Self {
        let device = &gpu.device;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("post"),
            source: wgpu::ShaderSource::Wgsl(include_str!("post.wgsl").into()),
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("post-bgl"),
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
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("post-layout"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("post-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    // Same format as the swapchain: the filter writes sRGB-encoded
                    // values straight through (identity precondition, DW-2.1).
                    format: gpu.config.format,
                    // Opaque replace: the fullscreen triangle writes every pixel.
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

        let params_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("post-params"),
            size: std::mem::size_of::<FilterParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let scene_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("post-scene-sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        let noise_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("post-noise-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            // LinearWrap rule for the POT noise images (`ScreenShaderData.cs:175-177`).
            address_mode_u: noise_address_mode(256, 256),
            address_mode_v: noise_address_mode(256, 256),
            ..Default::default()
        });

        let pattern = fallback_checkerboard();
        let fallback_tex = device.create_texture_with_data(
            &gpu.queue,
            &wgpu::TextureDescriptor {
                label: Some("post-noise-fallback"),
                size: wgpu::Extent3d {
                    width: 8,
                    height: 8,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &pattern,
        );
        let fallback_view = fallback_tex.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            target: None,
            target_view: None,
            target_size: None,
            pipeline,
            params_buf,
            scene_sampler,
            noise_sampler,
            fallback_noise: (fallback_tex, fallback_view),
        }
    }

    /// Recreates the offscreen target for `width`x`height` (DW-2.2; no-op when the
    /// target already matches).
    pub fn resize(&mut self, gpu: &Gpu, width: u32, height: u32) {
        if !target_needs_recreate(self.target_size, (width, height)) {
            return;
        }
        let size = (width.max(1), height.max(1));
        let tex = gpu.offscreen_target(size.0, size.1, "post-scene");
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        self.target = Some(tex);
        self.target_view = Some(view);
        self.target_size = Some(size);
    }

    /// Whether the post pipeline is usable this frame (a target exists). False
    /// before the first `scene_view`, letting the caller fall back to the
    /// direct-to-swapchain path (pixel identity by absence of the pass).
    pub fn active(&self) -> bool {
        self.target_view.is_some()
    }

    /// The view the scene must render into this frame, ensuring the offscreen
    /// target matches the current surface size (the plan's
    /// `render_scene_to_target` step: the caller then runs its normal
    /// `SpriteRenderer::render` with this view).
    pub fn scene_view(&mut self, gpu: &Gpu) -> &wgpu::TextureView {
        let (w, h) = (gpu.config.width, gpu.config.height);
        if target_needs_recreate(self.target_size, (w, h)) {
            self.resize(gpu, w, h);
        }
        self.target_view.as_ref().expect("post target after resize")
    }

    /// Draws the offscreen scene through `pass` into `target` (normally the
    /// swapchain view). The bind group is rebuilt per call — the scene view and
    /// noise view both change across frames, and vanilla rebinds per `Apply`
    /// (`ScreenShaderData.cs:139-187`) as well.
    pub fn apply_filter(
        &mut self,
        gpu: &Gpu,
        target: &wgpu::TextureView,
        params: &FilterParams,
        pass: PassId,
        images: FilterImages<'_>,
    ) {
        let scene_view = self
            .target_view
            .as_ref()
            .expect("post target (render the scene first)");

        let mut frame_params = *params;
        frame_params.pass_index = pass.0;
        gpu.queue
            .write_buffer(&self.params_buf, 0, bytemuck::bytes_of(&frame_params));

        let noise_view = match images.noise {
            Some(tex) => tex.create_view(&wgpu::TextureViewDescriptor::default()),
            None => self.fallback_noise.1.clone(),
        };
        let bind = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("post-bind"),
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.params_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(scene_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.scene_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&noise_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&self.noise_sampler),
                },
            ],
        });

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("post-filter"),
            });
        {
            let mut render = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("post-filter-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
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
            render.set_pipeline(&self.pipeline);
            render.set_bind_group(0, &bind, &[]);
            render.draw(0..3, 0..1);
        }
        gpu.queue.submit(Some(encoder.finish()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// DW-2.4: every `ScreenEffectInitializer.Load` row
    /// (`ScreenEffectInitializer.cs:14-48`) resolves through `pass_index`, and the
    /// pass names in `PASS_TABLE` are exactly the ones the C# rows pass to
    /// `ScreenShaderData`/subclass constructors. Paired with
    /// `tr-content::effect::tests::test_DW_2_4_xnb_pass_names_match_initializer`,
    /// which proves the same names exist in the retail `ScreenShader.xnb`.
    #[test]
    fn test_DW_2_4_pass_table_matches_initializer_rows() {
        // (Filters.Scene key, pass name) — one row per C# registration, in order.
        const INITIALIZER: [(&str, &str); 29] = [
            ("Nebula", "FilterTower"),
            ("Solar", "FilterTower"),
            ("Stardust", "FilterTower"),
            ("Vortex", "FilterTower"),
            ("MonolithNebula", "FilterMiniTower"),
            ("MonolithSolar", "FilterMiniTower"),
            ("MonolithStardust", "FilterMiniTower"),
            ("MonolithVortex", "FilterMiniTower"),
            ("MoonLord", "FilterMoonLord"),
            ("MoonLordShake", "FilterMoonLordShake"),
            ("MonolithMoonLord", "FilterMoonLord"),
            ("Graveyard", "FilterGraveyard"),
            ("testInvert", "FilterInvert"),
            ("BloodMoon", "FilterBloodMoon"),
            ("Sepia", "FilterSepia"),
            ("Noir", "FilterNoir"),
            ("Sandstorm", "FilterSandstormForeground"),
            ("SandstormOverlay", "FilterSandstormBackground"),
            ("Blizzard", "FilterBlizzardForeground"),
            ("BlizzardOverlay", "FilterBlizzardBackground"),
            ("HeatDistortion", "FilterHeatDistortion"),
            ("WaterDistortion", "FilterWaterDistortion"),
            ("CrystalDestructionColor", "FilterCrystalDestructionColor"),
            ("CrystalDestructionVortex", "FilterCrystalDestructionVortex"),
            ("CrystalWin", "FilterCrystalWin"),
            ("Test", "FilterTest"),
            ("Test2", "FilterTest2"),
            ("CRT", "FilterCRT"),
            ("Test3", "FilterTest3"),
        ];
        assert_eq!(PASS_TABLE.len(), INITIALIZER.len());
        for (i, (filter, pass)) in INITIALIZER.iter().enumerate() {
            assert_eq!(
                (PASS_TABLE[i].0, PASS_TABLE[i].1),
                (*filter, *pass),
                "row {i} diverges from ScreenEffectInitializer.cs"
            );
        }
        // The vanilla `Overlays.Scene` keys register the same shader objects under
        // the same names (`ScreenEffectInitializer.cs:32-39`) — our overlay rows are
        // named to stay unambiguous.
        assert_eq!(pass_index("Unknown"), PassId::PASSTHROUGH);
    }

    /// Phase 3 wired the tint/noise subset: the pass-name → index mapping must
    /// agree between `PASS_TABLE` and the WGSL `fs_main` switch (a drift would
    /// run the wrong filter body under a correct name).
    #[test]
    fn test_phase3_pass_indices_match_pass_names() {
        for (filter, pass_name, idx) in PASS_TABLE {
            let expected = match *pass_name {
                "FilterGraveyard" => PassId::GRAVEYARD,
                "FilterNoir" => PassId::NOIR,
                "FilterBloodMoon" => PassId::BLOOD_MOON,
                "FilterSandstormForeground" => PassId::SANDSTORM_FOREGROUND,
                "FilterSandstormBackground" => PassId::SANDSTORM_BACKGROUND,
                "FilterBlizzardForeground" => PassId::BLIZZARD_FOREGROUND,
                "FilterBlizzardBackground" => PassId::BLIZZARD_BACKGROUND,
                // Not ported yet: HeatDistortion/WaterDistortion (phase 4), the
                // towers/monoliths/MoonLord/CRT/test passes (M4/test content).
                _ => PassId::PASSTHROUGH,
            };
            assert_eq!(
                PassId(*idx),
                expected,
                "{filter}/{pass_name} index disagrees with the WGSL switch"
            );
        }
    }

    /// `FilterParams::NEUTRAL` mirrors the `ScreenShaderData` field defaults
    /// (`ScreenShaderData.cs:9-44`) — the driver (phase 3) relies on unset setters
    /// keeping vanilla values.
    #[test]
    fn test_DW_2_1_params_neutral_match_screendata_defaults() {
        let p = FilterParams::NEUTRAL;
        assert_eq!(p.color, [1.0, 1.0, 1.0, 0.0], "uColor = One (`:9`)");
        assert_eq!(
            p.secondary_color,
            [1.0, 1.0, 1.0, 0.0],
            "uSecondaryColor = One (`:11`)"
        );
        assert_eq!(p.opacity, 1.0, "_uOpacity = 1 (`:13`)");
        assert_eq!(p.intensity, 1.0, "_uIntensity = 1 (`:17`)");
        assert_eq!(p.target_pos, [1.0, 1.0], "_uTargetPosition = One (`:19`)");
        assert_eq!(p.direction, [0.0, 1.0], "_uDirection = (0,1) (`:21`)");
        assert_eq!(p.progress, 0.0, "_uProgress = 0 (`:23`)");
        assert_eq!(p.image_offset, [0.0, 0.0], "_uImageOffset = Zero (`:25`)");
        assert_eq!(p.image_scale, [1.0, 1.0], "_imageScales = One (`:39-44`)");
        assert_eq!(p.pass_index, 0, "neutral uniform selects the passthrough");
    }

    /// The uniform must stay 128 bytes / Pod so `bytes_of` matches the WGSL struct
    /// (a layout drift would silently scramble pass parameters).
    #[test]
    fn test_params_layout_is_pod_128_bytes() {
        assert_eq!(std::mem::size_of::<FilterParams>(), 128);
        assert_eq!(std::mem::size_of::<FilterParams>() % 16, 0);
    }

    /// DW-2.2: the target is recreated exactly when it is missing or resized away
    /// from — `WindowEvent::Resized` reconfigures the surface and the next
    /// `scene_view` self-heals (`loop.rs:95-99`, `gpu.rs::resize`).
    #[test]
    fn test_DW_2_2_target_recreated_when_size_changes() {
        assert!(target_needs_recreate(None, (1280, 720)), "no target yet");
        assert!(!target_needs_recreate(Some((1280, 720)), (1280, 720)));
        assert!(
            target_needs_recreate(Some((1280, 720)), (960, 540)),
            "resized"
        );
        assert!(target_needs_recreate(Some((960, 540)), (960, 540)) == false);
    }

    /// `ScreenShaderData.cs:175-182` sampler rule: POT → wrap, otherwise clamp.
    #[test]
    fn test_noise_sampler_address_mode_pot_rule() {
        assert_eq!(noise_address_mode(256, 256), wgpu::AddressMode::Repeat);
        assert_eq!(noise_address_mode(64, 512), wgpu::AddressMode::Repeat);
        assert_eq!(noise_address_mode(100, 256), wgpu::AddressMode::ClampToEdge);
        assert_eq!(noise_address_mode(256, 7), wgpu::AddressMode::ClampToEdge);
    }

    /// The fallback checkerboard is a complete opaque 8x8 RGBA image with 2x2
    /// alternating cells (plan edge case: missing noise images stay bindable).
    #[test]
    fn test_fallback_checkerboard_pattern() {
        let px = fallback_checkerboard();
        assert_eq!(px.len(), 8 * 8 * 4);
        for i in 0..64 {
            assert_eq!(px[i * 4 + 3], 0xFF, "texel {i} opaque");
        }
        let cell = |x: usize, y: usize| px[(y * 8 + x) * 4];
        assert_eq!(cell(0, 0), cell(1, 1), "2x2 blocks");
        assert_ne!(cell(0, 0), cell(2, 0), "blocks alternate");
        assert_ne!(cell(0, 0), cell(0, 2), "blocks alternate vertically");
    }
}
