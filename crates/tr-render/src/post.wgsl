// Fullscreen post-process filter pass (M1.4-BG-18 phase 2).
//
// WGSL port of the vanilla `ScreenShader.fx` pass *semantics* — the HLSL bytecode
// is not translated (plan §Phase 2 OUT); one WGSL function per pass, selected by
// the uniform `pass_index`, mirroring vanilla's one-Effect-many-passes shape
// (`ScreenShaderData.cs:98-101` binds a pass by name out of the same effect).
// Uniform fields follow `ScreenShaderData.Apply` (`ScreenShaderData.cs:139-187`).

struct FilterParams {
    color: vec4<f32>,             // uColor.xyz (`:143`)
    secondary_color: vec4<f32>,   // uSecondaryColor (`:145`)
    target_pos: vec2<f32>,        // uTargetPosition (`:149`)
    image_offset: vec2<f32>,      // uImageOffset (`:150`)
    image_scale: vec2<f32>,       // _imageScales[0] (`:183`)
    direction: vec2<f32>,         // uDirection (`:155`)
    screen_resolution: vec2<f32>, // uScreenResolution (`:147`)
    screen_position: vec2<f32>,   // uScreenPosition (`:148`)
    image_size: vec2<f32>,        // uImageSize[1] — image dims * scale (`:183`)
    _pad0: vec2<f32>,
    intensity: f32,               // uIntensity (`:153`)
    opacity: f32,                 // uOpacity = CombinedOpacity (`:144`)
    progress: f32,                // uProgress (`:154`)
    time: f32,                    // uTime (`:146`, GlobalTimeWrappedHourly)
    pass_index: u32,              // which pass function runs (vanilla: pass name)
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
};

@group(0) @binding(0) var<uniform> params: FilterParams;
@group(0) @binding(1) var scene_tex: texture_2d<f32>;
@group(0) @binding(2) var scene_sampler: sampler;
@group(0) @binding(3) var noise_tex: texture_2d<f32>;
@group(0) @binding(4) var noise_sampler: sampler;

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    // Fullscreen triangle: (-1,-1), (3,-1), (-1,3). The uv maps every output pixel
    // centre to the same texel centre of the scene texture — nearest sampling at
    // texel centres is what makes the passthrough byte-exact (DW-2.1).
    let x = f32(i32(vi & 1u)) * 4.0 - 1.0;
    let y = f32(i32(vi >> 1u)) * 4.0 - 1.0;
    var out: VsOut;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x, y) * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    return out;
}

// Pass 0: pixel-identity passthrough (DW-2.1). Phase 3 adds the ported filter
// passes below it and re-points `post.rs::PASS_TABLE` rows at their indices.
fn pass_passthrough(scene: vec4<f32>) -> vec4<f32> {
    return scene;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let scene = textureSample(scene_tex, scene_sampler, in.uv);
    var result: vec4<f32>;
    switch params.pass_index {
        default: {
            result = pass_passthrough(scene);
        }
    }
    return result;
}
