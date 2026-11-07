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
@group(0) @binding(3) var<uniform> dyes: array<DyeParam, 48>;
/// The retail font glyph pages (`Fonts/*.xnb`, shipped DXT3 = BC2) packed into
/// 8192x8192 layers; instances pick a layer with `layer` > 0.
@group(0) @binding(4) var fonts: texture_2d_array<f32>;

/// One dye row (`sprite.rs`'s `DyeParam`): the pass mode plus the two colors
/// — the CPU constants `DyeInitializer` bakes at `BindShader` time
/// (`DyeInitializer.cs:9-140`), folded onto the port's deduped slots.
struct DyeParam {
    mode: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
    color: vec4<f32>,
    secondary: vec4<f32>,
}

struct Instance {
    @location(0) pos: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) uv0: vec2<f32>,
    @location(3) uv1: vec2<f32>,
    @location(4) color: vec4<f32>,
    @location(5) rot: f32,
    /// `PaintID` (0 = unpainted); see `paint_recolor`.
    @location(6) paint: f32,
    /// The armor-dye slot (0 = plain): 1-based index into the `dyes` table —
    /// the per-instance channel for `DrawData.shader`
    /// (`PlayerDrawHelper.SetShaderForData`, `PlayerDrawHelper.cs:29-60`).
    @location(7) dye: u32,
    /// Which texture to sample: 0 = the main atlas, otherwise the 1-based
    /// layer into the font texture array (`fonts`).
    @location(8) layer: u32,
};

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    // The paint id is uniform across the quad's corners, so the perspective
    // interpolation returns it exactly.
    @location(2) paint: f32,
    // The dye slot — also corner-uniform; integer varyings need `flat`.
    @location(3) @interpolate(flat) dye: u32,
    // The sprite-local vertical (0 at the top corner, 1 at the bottom), the
    // gradient axis of `Armor*Gradient` (uColor → uSecondaryColor).
    @location(4) grad_v: f32,
    // The texture selector, corner-uniform; integer varyings need `flat`.
    @location(5) @interpolate(flat) layer: u32,
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
    // Optional per-sprite rotation around the top-left corner (zero for everything
    // except rain), exactly like XNA's `SpriteBatch.Draw` with origin `Vector2.Zero`.
    let local = c * inst.size;
    let cs = cos(inst.rot);
    let sn = sin(inst.rot);
    let world = inst.pos + vec2<f32>(local.x * cs - local.y * sn, local.x * sn + local.y * cs);
    let screen = (world - globals.cam) * globals.zoom + globals.screen * 0.5;
    let ndc = vec2<f32>(
        screen.x / globals.screen.x * 2.0 - 1.0,
        1.0 - screen.y / globals.screen.y * 2.0,
    );

    var out: VsOut;
    out.position = vec4<f32>(ndc, 0.0, 1.0);
    // Inset by half a texel: at fractional camera positions a sprite edge can
    // land exactly on a pixel centre, and nearest sampling of uv = 1.0 would
    // pick the transparent padding texel outside the source rect (visible as
    // 1 px holes between tiles). Sampling the texel centres avoids that.
    // Font glyphs bake the inset at build time (their layers carry their own
    // 8192 px texel grid), so only the main-atlas path insets here.
    if (inst.layer == 0u) {
        let half_texel = globals.texel * 0.5;
        out.uv = mix(inst.uv0 + half_texel, inst.uv1 - half_texel, c);
    } else {
        out.uv = mix(inst.uv0, inst.uv1, c);
    }
    out.color = inst.color;
    out.paint = inst.paint;
    out.dye = inst.dye;
    out.layer = inst.layer;
    out.grad_v = c.y;
    return out;
}

fn paint_rgb(pid: u32) -> vec3<f32> {
    var table = array<vec3<f32>, 32>(
        vec3<f32>(1.0),
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(1.0, 0.498, 0.0),
        vec3<f32>(1.0, 1.0, 0.0),
        vec3<f32>(0.498, 1.0, 0.0),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(0.0, 1.0, 0.498),
        vec3<f32>(0.0, 1.0, 1.0),
        vec3<f32>(0.0, 0.498, 1.0),
        vec3<f32>(0.0, 0.0, 1.0),
        vec3<f32>(0.498, 0.0, 1.0),
        vec3<f32>(1.0, 0.0, 1.0),
        vec3<f32>(1.0, 0.0, 0.498),
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(1.0, 0.498, 0.0),
        vec3<f32>(1.0, 1.0, 0.0),
        vec3<f32>(0.498, 1.0, 0.0),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(0.0, 1.0, 0.498),
        vec3<f32>(0.0, 1.0, 1.0),
        vec3<f32>(0.0, 0.498, 1.0),
        vec3<f32>(0.0, 0.0, 1.0),
        vec3<f32>(0.498, 0.0, 1.0),
        vec3<f32>(1.0, 0.0, 1.0),
        vec3<f32>(1.0, 0.0, 0.498),
        vec3<f32>(0.294, 0.294, 0.294),
        vec3<f32>(1.0),
        vec3<f32>(0.686, 0.686, 0.686),
        vec3<f32>(1.0, 0.698, 0.490),
        vec3<f32>(0.098, 0.098, 0.098),
        vec3<f32>(0.784, 0.784, 0.784),
        vec3<f32>(1.0),
    );
    return table[pid];
}

// `TilePaintSystemV2`'s baked recolor (`TilePaintSystemV2.cs:43-51`): a
// per-(type, style, paint) recolored copy of the same texture frame. The
// vanilla recolor runs as compiled HLSL passes (the .fx source is not part
// of the decompile); this port shades in the sRGB domain — the paint hue
// washes the texel while its luminance carries the texture detail.
// Negative paint inverts the pixel; illuminant (31) draws unpainted (the
// vanilla pass 0) and glows through the light path.
fn paint_recolor(tex: vec4<f32>, pid: u32) -> vec4<f32> {
    if (pid == 0u) {
        return tex;
    }
    if (pid == 30u) {
        return vec4<f32>(1.0 - tex.rgb, tex.a);
    }
    if (pid == 31u) {
        return tex;
    }
    let lum = dot(tex.rgb, vec3<f32>(0.299, 0.587, 0.114));
    return vec4<f32>(paint_rgb(pid) * lum, tex.a);
}

// The armor-dye passes. The vanilla effect compiles from `Main.Assets[
// "PixelShader"]` and the .fx bytecode is not in the decompile, so the cores
// are the documented reconstruction (docs/ANALYSIS.md "Dye pass coverage"):
// `colored` — the texel's luminance carries the texture detail while the dye
// color washes it (the same shape as the tile-paint `paint_recolor` above);
// `gradient` — a vertical uColor→uSecondaryColor lerp over the sprite before
// the wash. The C# colors are baked constants (`DyeInitializer`); `AndBlack`/
// `AndSilverTrim`/`Brightness` variants ride the colored core with their own
// colors, and unported passes never reach the table (they keep dye 0).
fn dye_recolor(tex: vec4<f32>, dye: u32, grad_v: f32) -> vec4<f32> {
    if (dye == 0u) {
        return tex;
    }
    let p = dyes[min(dye - 1u, 47u)];
    let lum = dot(tex.rgb, vec3<f32>(0.299, 0.587, 0.114));
    if (p.mode == 1u) {
        return vec4<f32>(p.color.rgb * lum, tex.a);
    }
    if (p.mode == 2u) {
        let col = mix(p.color.rgb, p.secondary.rgb, grad_v);
        return vec4<f32>(col * lum, tex.a);
    }
    return tex;
}

/// One sample helper for both textures: the main atlas is D2, the font pages a
/// D2 array. `textureSampleLevel` (explicit LOD 0) is legal in non-uniform
/// control flow, unlike `textureSample`; the textures have no mips, so the
/// result is identical.
fn sample_tex(uv: vec2<f32>, layer: u32) -> vec4<f32> {
    if (layer == 0u) {
        return textureSampleLevel(atlas, atlas_sampler, uv, 0.0);
    }
    return textureSampleLevel(fonts, atlas_sampler, uv, f32(layer - 1u), 0.0);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    var tex = sample_tex(in.uv, in.layer);
    if (tex.a < 0.01) {
        discard;
    }
    // "Trippy" light mode: split the texture channels horizontally, like the
    // original's glitch lighting.
    if (globals.aberration > 0.0) {
        let d = vec2<f32>(globals.aberration * globals.texel.x, 0.0);
        let r = sample_tex(in.uv + d, in.layer).r;
        let b = sample_tex(in.uv - d, in.layer).b;
        tex = vec4<f32>(r, tex.g, b, tex.a);
    }
    // `WorldGen.paintColor` (`WorldGen.cs:43921-44034`): the sRGB bytes of every
    // paint id. 1-12 are the twelve hues, 13-24 the deep variants (the vanilla
    // shader deepens saturation; this port washes both identically — the
    // compiled HLSL is not in the decompile), 25 black, 26 white, 27 gray,
    // 28 brown, 29 shadow, 30 negative, 31 illuminant (drawn unpainted, like
    // the vanilla pass 0; the glow is light-side).
    // Terraria multiplies sprite tints in gamma space: XNA's `Color * Color` is a
    // plain byte multiply on sRGB values. The atlas is sRGB-decoded and the swapchain
    // encodes on write, so pre-linearise the tint to get the same result on screen.
    let tint = pow(max(in.color.rgb, vec3<f32>(0.0)), vec3<f32>(2.2));
    // XNA's `BlendState.AlphaBlend` is premultiplied: the rgb source factor is
    // `One`, so a tint with alpha 0 (the tile flames' `Color(200,200,200,0)`)
    // ADDS the texture rgb to the destination instead of fading out. The atlas
    // textures are premultiplied by the XNA content pipeline (a=0 texels are
    // rgb=0, semi-alpha texels satisfy rgb <= a — verified across the asset
    // set), so the fragment must NOT multiply rgb by the alpha here; the
    // pipeline's (One, OneMinusSrcAlpha) factors finish the equation.
    let alpha = tex.a * in.color.a;
    let dyed = dye_recolor(tex, in.dye, in.grad_v);
    let painted = paint_recolor(dyed, u32(in.paint + 0.5));
    return vec4<f32>(painted.rgb * tint, alpha);
}
