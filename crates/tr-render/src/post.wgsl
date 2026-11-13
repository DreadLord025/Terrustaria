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
// Water coverage mask (phase 4): R = 1 where a liquid cell covers the screen
// pixel, same view mapping as the scene (water_mask.wgsl renders it). A 1x1
// zero texture binds when no water is in view. Sampled with `scene_sampler`
// (nearest/clamp).
@group(0) @binding(5) var mask_tex: texture_2d<f32>;

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

// Pass 0: pixel-identity passthrough (DW-2.1). Passes 1-7 are the phase-3
// tint/noise subset (`post.rs::PassId`); unported passes resolve here.
fn pass_passthrough(scene: vec4<f32>) -> vec4<f32> {
    return scene;
}

// The world-pixel coordinate of the fragment: the view uv scaled to the
// unscaled view size plus the view's top-left corner in world pixels
// (`uScreenResolution`/`uScreenPosition`, `ScreenShaderData.cs:147-148`).
// Noise images are sampled at this coordinate divided by `uImageSize` —
// world-anchored, so the pattern sticks while the camera moves — offset by
// the pass's scroll/offset vector.
fn world_px(uv: vec2<f32>) -> vec2<f32> {
    return uv * params.screen_resolution + params.screen_position;
}

// Pass 1: FilterGraveyard — the graveyard mist: a luminance desaturation
// toward uColor weighted by uProgress (= Lerp(0, 0.75, intensity),
// `SceneState.cs:109`), with a distance falloff from uTargetPosition (the
// player center, `:111`) so the tint pools around the player. Approximation
// of the compiled pass (plan A2 — the HLSL bytecode is not translated);
// visual-match port.
fn pass_graveyard(scene: vec4<f32>, uv: vec2<f32>) -> vec4<f32> {
    let radius = max(1.0, 0.5 * params.screen_resolution.y * params.intensity);
    let falloff = 1.0 - min(1.0, distance(world_px(uv), params.target_pos) / radius);
    let amount = params.progress * mix(0.5, 1.0, falloff);
    let lum = dot(scene.rgb, vec3<f32>(0.299, 0.587, 0.114));
    let mist = lum * params.color.rgb;
    return vec4<f32>(mix(scene.rgb, mist, amount), scene.a);
}

// Pass 2: FilterNoir — noise-dithered black-and-white. `Images/NoirNoise`
// (`ScreenEffectInitializer.cs:29`, PointWrap) is absent from the content set,
// so the fallback checkerboard binds. uIntensity carries the depth/tileColor
// formula (`SceneState.cs:117-125`), uImageOffset the per-tenth-second dither
// jump (`:124`). Approximation (plan A2); no M1 activation path (monolith).
fn pass_noir(scene: vec4<f32>, noise: f32) -> vec4<f32> {
    let lum = dot(scene.rgb, vec3<f32>(1.0 / 3.0));
    let dithered = clamp(lum + (noise - 0.5) * 0.2, 0.0, 1.0);
    return vec4<f32>(
        mix(scene.rgb, vec3<f32>(dithered), clamp(params.intensity, 0.0, 1.0)),
        scene.a,
    );
}

// Pass 3: FilterBloodMoon — the UseColor matrix (2, -0.8, -0.6) applied per
// channel (`ScreenEffectInitializer.cs:27`): red boosted, green/blue crushed,
// mixed toward the result by uOpacity (the depth fade × 0.75 of
// `BloodMoonScreenShaderData.Update` times the fade opacity). Approximation
// (plan A2).
fn pass_blood_moon(scene: vec4<f32>) -> vec4<f32> {
    let tinted = clamp(scene.rgb * params.color.rgb, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(
        mix(scene.rgb, tinted, clamp(params.opacity, 0.0, 1.0)),
        scene.a,
    );
}

// Passes 4/5 (FilterSandstormForeground/Background) and 6/7
// (FilterBlizzardForeground/Background): one storm body. The noise image is
// sampled at the world-pixel coordinate over uImageSize (dims × scale — the
// Blizzard's (3, 0.75) stretches it into streaks), scrolled by uTargetPosition
// (the shader's `_texturePosition`, forced at Apply time by both shader
// classes), lerping uSecondaryColor → uColor by the noise value and mixing
// into the scene at noise-weighted uIntensity × uOpacity. The driver picks
// whichever of the pair carries the larger effective opacity (the foreground
// Filter in the open, the background overlay behind backwalls — the vanilla
// pair cross-fades by exactly those terms). Approximation (plan A2).
fn pass_storm(scene: vec4<f32>, noise: f32) -> vec4<f32> {
    let sand = mix(params.secondary_color.rgb, params.color.rgb, noise);
    let cover = clamp(noise * params.intensity, 0.0, 1.0) * clamp(params.opacity, 0.0, 1.0);
    return vec4<f32>(mix(scene.rgb, sand, cover), scene.a);
}

// Distortion passes (phase 4) displace the scene lookup instead of tinting the
// result: they return the uv the scene is sampled at. Offsets are in world
// pixels (zoom-invariant, like the world-anchored noise) divided by the view
// size. The compiled `FilterHeatDistortion`/`FilterWaterDistortion` HLSL is not
// translated — amplitudes and scroll rates are visual approximations (plan A2).
const HEAT_AMPLITUDE_PX: f32 = 1.5;
const WATER_AMPLITUDE_PX: f32 = 2.0;
// S1-G-6: the Moon Lord ritual shake's displacement scale. The compiled
// `FilterMoonLordShake` HLSL is not translated (the phase-4 rule) — the
// amplitude is the port's, the intensity is the source's.
const MOON_LORD_AMPLITUDE_PX: f32 = 6.0;

// Pass 8: FilterHeatDistortion — the whole view shimmers. Two decorrelated
// reads of the noise image (scrolling upward with uTime) give a 2-D offset
// scaled by uIntensity (`SceneState.cs:145-160`, 0..4) and the fade uOpacity.
fn heat_uv(uv: vec2<f32>) -> vec2<f32> {
    let p = world_px(uv) / params.image_size;
    let scroll = vec2<f32>(0.0, -params.time * 0.15);
    let nx = textureSampleLevel(noise_tex, noise_sampler, p + scroll, 0.0).r;
    let ny = textureSampleLevel(
        noise_tex, noise_sampler, p.yx * 1.37 + vec2<f32>(0.31, 0.0) + scroll, 0.0,
    ).r;
    let off_px = (vec2<f32>(nx, ny) - 0.5) * 2.0 * HEAT_AMPLITUDE_PX
        * params.intensity * clamp(params.opacity, 0.0, 1.0);
    return uv + off_px / params.screen_resolution;
}

// Pass 9: FilterWaterDistortion — only pixels the water mask covers wobble.
// uProgress is the `WaterShaderData._progress` clock (`WaterShaderData.cs:126`)
// that scrolls the noise; uIntensity is the wind/rain/depth value of
// `SceneState.cs:128-132` (0..2.5). Branch-free so the sample stays in uniform
// control flow: the offset is multiplied by the 0/1 mask.
fn water_uv(uv: vec2<f32>) -> vec2<f32> {
    let m = textureSampleLevel(mask_tex, scene_sampler, uv, 0.0).r;
    let p = world_px(uv) / params.image_size * 4.0;
    let t = params.progress * 0.1;
    let nx = textureSampleLevel(noise_tex, noise_sampler, p + vec2<f32>(t, 0.0), 0.0).r;
    let ny = textureSampleLevel(
        noise_tex, noise_sampler, p.yx + vec2<f32>(0.5, t * 0.7), 0.0,
    ).r;
    let off_px = (vec2<f32>(nx, ny) - 0.5) * 2.0 * WATER_AMPLITUDE_PX
        * params.intensity * clamp(params.opacity, 0.0, 1.0);
    return uv + off_px / params.screen_resolution * ceil(m);
}

// Pass 10: FilterMoonLordShake — the Moon Lord ritual's screen shake
// (S1-G-6). The compiled `FilterMoonLordShake` HLSL is not translated (the
// phase-4 rule): the port's approximation is the phase-4 distortion shape —
// two decorrelated noise lookups displace the whole view, scaled by
// `uIntensity` (the `Main.cs:64977-64979` countdown formula the tick driver
// feeds). The vanilla pass also reads `uTargetPosition` (the shake anchor);
// the port passes Zero — the exact `UpdateMoonLordIndex` value while the
// filter is active, because no Moon Lord exists before the countdown lands.
fn moon_lord_uv(uv: vec2<f32>) -> vec2<f32> {
    let p = world_px(uv) / params.image_size * 8.0;
    let t = params.time * 10.0;
    let nx = textureSampleLevel(noise_tex, noise_sampler, p + vec2<f32>(t, 0.0), 0.0).r;
    let ny = textureSampleLevel(
        noise_tex, noise_sampler, p.yx + vec2<f32>(0.5, t * 0.9), 0.0,
    ).r;
    let off_px = (vec2<f32>(nx, ny) - 0.5) * 2.0 * MOON_LORD_AMPLITUDE_PX
        * params.intensity * clamp(params.opacity, 0.0, 1.0);
    return uv + off_px / params.screen_resolution;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    // Passes 0-7 sample the scene at `in.uv` exactly (byte-identical to the
    // phase-2/3 shader); only the distortion passes move the lookup.
    var scene_uv = in.uv;
    switch params.pass_index {
        case 8u: {
            scene_uv = heat_uv(in.uv);
        }
        case 9u: {
            scene_uv = water_uv(in.uv);
        }
        case 10u: {
            scene_uv = moon_lord_uv(in.uv);
        }
        default: {}
    }
    let scene = textureSample(scene_tex, scene_sampler, scene_uv);
    // The storm scroll (uTargetPosition) and the Noir dither (uImageOffset)
    // sample the same noise image at different offsets; both are uniform-flow
    // samples so the pass switch can consume either.
    let storm_noise = textureSample(
        noise_tex,
        noise_sampler,
        world_px(in.uv) / params.image_size + params.target_pos,
    ).r;
    let noir_noise = textureSample(
        noise_tex,
        noise_sampler,
        world_px(in.uv) / params.image_size + params.image_offset,
    ).r;
    var result: vec4<f32>;
    switch params.pass_index {
        case 1u: {
            result = pass_graveyard(scene, in.uv);
        }
        case 2u: {
            result = pass_noir(scene, noir_noise);
        }
        case 3u: {
            result = pass_blood_moon(scene);
        }
        case 4u, 5u, 6u, 7u: {
            result = pass_storm(scene, storm_noise);
        }
        default: {
            result = pass_passthrough(scene);
        }
    }
    return result;
}
