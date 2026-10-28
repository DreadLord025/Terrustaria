struct Globals {
    screen: vec2<f32>,
    cam: vec2<f32>,
    zoom: f32,
    time: f32,
    aberration: f32,
    _pad: f32,
    texel: vec2<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var atlas: texture_2d<f32>;
@group(0) @binding(2) var atlas_sampler: sampler;

struct Instance {
    @location(0) pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) uv0: vec2<f32>,
    @location(3) uv1: vec2<f32>,
    @location(4) color: vec4<f32>,
    @location(5) flags: f32,
};

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) world: vec2<f32>,
    @location(3) @interpolate(flat) flags: f32,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32, inst: Instance) -> VsOut {
    // 0---1
    // | \ |
    // 2---3   (two triangles, y grows downwards in world space)
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
    );
    let c = corners[vi];
    let world = inst.pos + c * inst.size;
    let screen = (world - globals.cam) * globals.zoom + globals.screen * 0.5;
    let ndc = vec2<f32>(
        screen.x / globals.screen.x * 2.0 - 1.0,
        1.0 - screen.y / globals.screen.y * 2.0,
    );

    var out: VsOut;
    out.position = vec4<f32>(ndc, 0.0, 1.0);
    out.uv = mix(inst.uv0, inst.uv1, c);
    out.color = inst.color;
    out.world = world;
    out.flags = inst.flags;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    var uv = in.uv;
    // Liquids: distort the sample by a small travelling wave. The source frames
    // sit inside the large `Misc/water_*` strip, so a 1-2 px shift is safe.
    if (in.flags > 0.5) {
        let w = sin(in.world.x * 0.31 + globals.time * 1.9)
            + cos(in.world.y * 0.37 - globals.time * 1.3);
        uv += vec2<f32>(w, w * 0.35) * globals.texel * 1.4;
    }
    var tex = textureSample(atlas, atlas_sampler, uv);
    if (tex.a < 0.01) {
        discard;
    }
    // "Trippy" light mode: split the texture channels horizontally, like the
    // original's glitch lighting.
    if (globals.aberration > 0.0) {
        let d = vec2<f32>(globals.aberration * globals.texel.x, 0.0);
        let r = textureSample(atlas, atlas_sampler, in.uv + d).r;
        let b = textureSample(atlas, atlas_sampler, in.uv - d).b;
        tex = vec4<f32>(r, tex.g, b, tex.a);
    }
    // Terraria multiplies sprite tints in gamma space: XNA's `Color * Color` is a
    // plain byte multiply on sRGB values. The atlas is sRGB-decoded and the swapchain
    // encodes on write, so pre-linearise the tint to get the same result on screen.
    let tint = pow(max(in.color.rgb, vec3<f32>(0.0)), vec3<f32>(2.2));
    return vec4<f32>(tex.rgb * tint, tex.a * in.color.a);
}
