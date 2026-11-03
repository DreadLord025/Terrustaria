// Water coverage mask (M1.4-BG-18 phase 4): every liquid rect the liquid pass
// drew (`LiquidRenderer` records) rasterised as 1 into an R8 target, same
// world -> screen mapping as the sprite pass (`sprite.wgsl`). Vanilla renders
// the liquid layer into `Main.waterTarget` (`Main.cs:56723-56734,57285`) and
// the WaterDistortion pass reads its coverage (`WaterShaderData.Apply`,
// `WaterShaderData.cs:371`); this is the coverage-only equivalent.

struct Globals {
    screen: vec2<f32>,
    cam: vec2<f32>,
    zoom: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
};

@group(0) @binding(0) var<uniform> g: Globals;

struct Instance {
    @location(0) pos: vec2<f32>,
    @location(1) size: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32, inst: Instance) -> @builtin(position) vec4<f32> {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
    );
    let world = inst.pos + corners[vi] * inst.size;
    let screen = (world - g.cam) * g.zoom + g.screen * 0.5;
    return vec4<f32>(
        screen.x / g.screen.x * 2.0 - 1.0,
        1.0 - screen.y / g.screen.y * 2.0,
        0.0,
        1.0,
    );
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
