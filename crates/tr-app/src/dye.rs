//! The armor-dye registry: the hand port of `DyeInitializer.LoadArmorDyes`
//! (`Initializers/DyeInitializer.cs:44-140`) plus `LoadBasicColorDyes`
//! (`:25-40`). C# assigns every item its dye generically at the end of
//! `SetDefaults` — `this.dye = (byte)GameShaders.Armor.GetShaderIdFromItemId(
//! this.type)` (`Item.cs:48713`) — so the whole item→shader mapping lives in
//! this one binding table, exactly like the vanilla `_shaderLookupDictionary`
//! (`ArmorShaderDataSet.cs:10-16`).
//!
//! The vanilla passes compile from `Main.Assets["PixelShader"]`
//! (`Main.cs:10804`) and the .fx bytecode is **not** part of the decompile —
//! every entry records its C# pass name and the core the port renders it
//! with (`sprite.wgsl`'s dye dispatch). The reconstruction is logged in
//! `docs/ANALYSIS.md`; unimplemented passes render plain until their tasks.

/// The pixel core a binding renders with (`sprite.wgsl`'s dye dispatch).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DyeCore {
    /// `ArmorColored` / `ArmorColoredAndBlack` / `ArmorColoredAndSilverTrim` /
    /// `ArmorBrightnessColored`: the texel's luminance carries the texture
    /// detail, the dye color washes it (the reconstruction shared with the
    /// tile-paint `paint_recolor`, `sprite.wgsl`).
    Colored,
    /// `Armor*Gradient`: vertical lerp uColor → uSecondaryColor over the
    /// sprite, then the luminance wash.
    Gradient,
    /// Bound verbatim (pass name + parameters recorded), rendered plain until
    /// the pass is ported (ANALYSIS "dye pass coverage").
    Unimplemented,
}

/// One `GameShaders.Armor.BindShader` call
/// (`ArmorShaderDataSet.BindShader`, `ArmorShaderDataSet.cs:14-19`).
#[derive(Clone, Copy, Debug)]
pub(crate) struct DyeEntry {
    /// The dye item the shader binds to (`GetShaderIdFromItemId`'s key).
    pub(crate) item: u16,
    /// The `ArmorShaderData` pass name — the C#-side anchor.
    pub(crate) csharp_pass: &'static str,
    /// The 1-based `BindShader` position (`_shaderDataCount` order). Not used
    /// by the renderer — the port's uniform slots are deduped parameters —
    /// kept so the table can be diffed against the C# binding order.
    pub(crate) csharp_id: u8,
    pub(crate) core: DyeCore,
    /// `UseColor(r, g, b)` (linear floats straight from the C# call).
    pub(crate) color: [f32; 3],
    /// `UseSecondaryColor(r, g, b)`.
    pub(crate) secondary: [f32; 3],
    /// `UseSaturation(s)` — recorded for fidelity; the port folds it away on
    /// the CPU because it changes nothing: every binding either uses the 1.0
    /// default or a color that is already fully saturated (pinned by
    /// `tests::vanilla_dye_colors_are_saturated`).
    pub(crate) saturation: f32,
}

// The binding macros mirror the C# calls one-to-one; each expands to the
// matching `push`es in `BindShader` order.

/// `LoadBasicColorDye(baseDyeItem, r, g, b, saturation)`'s four `BindShader`
/// calls (`DyeInitializer.cs:9-23`): base, +12 black, +31 bright
/// (`r*0.5+0.5`), +44 silver.
macro_rules! bind_basic {
    ($v:ident, $base:expr, $id:expr, $r:expr, $g:expr, $b:expr, $sat:expr) => {
        $v.push(DyeEntry {
            item: $base,
            csharp_pass: "ArmorColored",
            csharp_id: $id,
            core: DyeCore::Colored,
            color: [$r, $g, $b],
            secondary: [0.0, 0.0, 0.0],
            saturation: $sat,
        });
        $v.push(DyeEntry {
            item: $base + 12,
            csharp_pass: "ArmorColoredAndBlack",
            csharp_id: $id + 1,
            core: DyeCore::Colored,
            color: [$r, $g, $b],
            secondary: [0.0, 0.0, 0.0],
            saturation: $sat,
        });
        $v.push(DyeEntry {
            item: $base + 31,
            csharp_pass: "ArmorColored",
            csharp_id: $id + 2,
            core: DyeCore::Colored,
            color: [$r * 0.5 + 0.5, $g * 0.5 + 0.5, $b * 0.5 + 0.5],
            secondary: [0.0, 0.0, 0.0],
            saturation: $sat,
        });
        $v.push(DyeEntry {
            item: $base + 44,
            csharp_pass: "ArmorColoredAndSilverTrim",
            csharp_id: $id + 3,
            core: DyeCore::Colored,
            color: [$r, $g, $b],
            secondary: [0.0, 0.0, 0.0],
            saturation: $sat,
        });
    };
}

/// The explicit-item overload `LoadBasicColorDye(baseDyeItem, blackDyeItem,
/// brightDyeItem, silverDyeItem, r, g, b, saturation)` — same four passes,
/// ids given verbatim (the brown family, `DyeInitializer.cs:37`).
macro_rules! bind_basic4 {
    ($v:ident, $base:expr, $black:expr, $bright:expr, $silver:expr, $id:expr, $r:expr, $g:expr, $b:expr, $sat:expr) => {
        $v.push(DyeEntry {
            item: $base,
            csharp_pass: "ArmorColored",
            csharp_id: $id,
            core: DyeCore::Colored,
            color: [$r, $g, $b],
            secondary: [0.0, 0.0, 0.0],
            saturation: $sat,
        });
        $v.push(DyeEntry {
            item: $black,
            csharp_pass: "ArmorColoredAndBlack",
            csharp_id: $id + 1,
            core: DyeCore::Colored,
            color: [$r, $g, $b],
            secondary: [0.0, 0.0, 0.0],
            saturation: $sat,
        });
        $v.push(DyeEntry {
            item: $bright,
            csharp_pass: "ArmorColored",
            csharp_id: $id + 2,
            core: DyeCore::Colored,
            color: [$r * 0.5 + 0.5, $g * 0.5 + 0.5, $b * 0.5 + 0.5],
            secondary: [0.0, 0.0, 0.0],
            saturation: $sat,
        });
        $v.push(DyeEntry {
            item: $silver,
            csharp_pass: "ArmorColoredAndSilverTrim",
            csharp_id: $id + 3,
            core: DyeCore::Colored,
            color: [$r, $g, $b],
            secondary: [0.0, 0.0, 0.0],
            saturation: $sat,
        });
    };
}

macro_rules! bind_colored {
    ($v:ident, $item:expr, $id:expr, $pass:expr, $r:expr, $g:expr, $b:expr, $sat:expr) => {
        $v.push(DyeEntry {
            item: $item,
            csharp_pass: $pass,
            csharp_id: $id,
            core: DyeCore::Colored,
            color: [$r, $g, $b],
            secondary: [0.0, 0.0, 0.0],
            saturation: $sat,
        });
    };
}

macro_rules! bind_grad {
    ($v:ident, $item:expr, $id:expr, $pass:expr, $r:expr, $g:expr, $b:expr, $r2:expr, $g2:expr, $b2:expr, $sat:expr) => {
        $v.push(DyeEntry {
            item: $item,
            csharp_pass: $pass,
            csharp_id: $id,
            core: DyeCore::Gradient,
            color: [$r, $g, $b],
            secondary: [$r2, $g2, $b2],
            saturation: $sat,
        });
    };
}

macro_rules! bind_raw {
    ($v:ident, $item:expr, $id:expr, $pass:expr, $r:expr, $g:expr, $b:expr, $r2:expr, $g2:expr, $b2:expr, $sat:expr) => {
        $v.push(DyeEntry {
            item: $item,
            csharp_pass: $pass,
            csharp_id: $id,
            core: DyeCore::Unimplemented,
            color: [$r, $g, $b],
            secondary: [$r2, $g2, $b2],
            saturation: $sat,
        });
    };
}

/// The full `LoadBasicColorDyes` + `LoadArmorDyes` binding table
/// (`DyeInitializer.cs:25-140`), in `BindShader` order (the C# 1-based
/// shader ids). 13 basic families × 4 + 68 explicit = 120 entries.
fn build_registry() -> Vec<DyeEntry> {
    let mut v = Vec::with_capacity(120);
    bind_basic!(v, 1007, 1, 1.0, 0.0, 0.0, 1.2);
    bind_basic!(v, 1008, 5, 1.0, 0.5, 0.0, 1.2);
    bind_basic!(v, 1009, 9, 1.0, 1.0, 0.0, 1.2);
    bind_basic!(v, 1010, 13, 0.5, 1.0, 0.0, 1.2);
    bind_basic!(v, 1011, 17, 0.0, 1.0, 0.0, 1.2);
    bind_basic!(v, 1012, 21, 0.0, 1.0, 0.5, 1.2);
    bind_basic!(v, 1013, 25, 0.0, 1.0, 1.0, 1.2);
    bind_basic!(v, 1014, 29, 0.2, 0.5, 1.0, 1.2);
    bind_basic!(v, 1015, 33, 0.0, 0.0, 1.0, 1.2);
    bind_basic!(v, 1016, 37, 0.5, 0.0, 1.0, 1.2);
    bind_basic!(v, 1017, 41, 1.0, 0.0, 1.0, 1.2);
    bind_basic!(v, 1018, 45, 1.0, 0.1, 0.5, 1.3);
    // The brown family binds its four ids explicitly (`DyeInitializer.cs:37`,
    // `LoadBasicColorDye(2874, 2875, 2876, 2877, ...)`, default saturation 1).
    bind_basic4!(v, 2874, 2875, 2876, 2877, 49, 0.4, 0.2, 0.0, 1.0);
    // The BrightnessColored entries bind outside the basic families; the port
    // renders them with the same Colored core (`sprite.wgsl` — the per-pass
    // differences live in the unavailable HLSL).
    bind_colored!(v, 1050, 53, "ArmorBrightnessColored", 0.6, 0.6, 0.6, 1.0);
    bind_colored!(v, 1037, 54, "ArmorBrightnessColored", 1.0, 1.0, 1.0, 1.0);
    bind_colored!(v, 3558, 55, "ArmorBrightnessColored", 1.5, 1.5, 1.5, 1.0);
    bind_colored!(v, 2871, 56, "ArmorBrightnessColored", 0.05, 0.05, 0.05, 1.0);
    bind_colored!(v, 3559, 57, "ArmorColoredAndBlack", 1.0, 1.0, 1.0, 1.2);
    bind_grad!(
        v,
        1031,
        58,
        "ArmorColoredGradient",
        1.0,
        0.0,
        0.0,
        1.0,
        1.0,
        0.0,
        1.2
    );
    bind_grad!(
        v,
        1032,
        59,
        "ArmorColoredAndBlackGradient",
        1.0,
        0.0,
        0.0,
        1.0,
        1.0,
        0.0,
        1.5
    );
    bind_grad!(
        v,
        3550,
        60,
        "ArmorColoredAndSilverTrimGradient",
        1.0,
        0.0,
        0.0,
        1.0,
        1.0,
        0.0,
        1.5
    );
    bind_grad!(
        v,
        1063,
        61,
        "ArmorBrightnessGradient",
        1.0,
        0.0,
        0.0,
        1.0,
        1.0,
        0.0,
        1.0
    );
    bind_grad!(
        v,
        1035,
        62,
        "ArmorColoredGradient",
        0.0,
        0.0,
        1.0,
        0.0,
        1.0,
        1.0,
        1.2
    );
    bind_grad!(
        v,
        1036,
        63,
        "ArmorColoredAndBlackGradient",
        0.0,
        0.0,
        1.0,
        0.0,
        1.0,
        1.0,
        1.5
    );
    bind_grad!(
        v,
        3552,
        64,
        "ArmorColoredAndSilverTrimGradient",
        0.0,
        0.0,
        1.0,
        0.0,
        1.0,
        1.0,
        1.5
    );
    bind_grad!(
        v,
        1065,
        65,
        "ArmorBrightnessGradient",
        0.0,
        0.0,
        1.0,
        0.0,
        1.0,
        1.0,
        1.0
    );
    bind_grad!(
        v,
        1033,
        66,
        "ArmorColoredGradient",
        0.0,
        1.0,
        0.0,
        1.0,
        1.0,
        0.0,
        1.2
    );
    bind_grad!(
        v,
        1034,
        67,
        "ArmorColoredAndBlackGradient",
        0.0,
        1.0,
        0.0,
        1.0,
        1.0,
        0.0,
        1.5
    );
    bind_grad!(
        v,
        3551,
        68,
        "ArmorColoredAndSilverTrimGradient",
        0.0,
        1.0,
        0.0,
        1.0,
        1.0,
        0.0,
        1.5
    );
    bind_grad!(
        v,
        1064,
        69,
        "ArmorBrightnessGradient",
        0.0,
        1.0,
        0.0,
        1.0,
        1.0,
        0.0,
        1.0
    );
    bind_grad!(
        v,
        1068,
        70,
        "ArmorColoredGradient",
        0.5,
        1.0,
        0.0,
        1.0,
        0.5,
        0.0,
        1.5
    );
    bind_grad!(
        v,
        1069,
        71,
        "ArmorColoredGradient",
        0.0,
        1.0,
        0.5,
        0.0,
        0.5,
        1.0,
        1.5
    );
    bind_grad!(
        v,
        1070,
        72,
        "ArmorColoredGradient",
        1.0,
        0.0,
        0.5,
        0.5,
        0.0,
        1.0,
        1.5
    );
    bind_raw!(
        v,
        1066,
        73,
        "ArmorColoredRainbow",
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0
    );
    bind_raw!(
        v,
        1067,
        74,
        "ArmorBrightnessRainbow",
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0
    );
    bind_raw!(
        v,
        3556,
        75,
        "ArmorMidnightRainbow",
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0
    );
    bind_raw!(
        v,
        2869,
        76,
        "ArmorLivingFlame",
        1.0,
        0.9,
        0.0,
        1.0,
        0.2,
        0.0,
        1.0
    );
    bind_raw!(
        v,
        2870,
        77,
        "ArmorLivingRainbow",
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0
    );
    bind_raw!(
        v,
        2873,
        78,
        "ArmorLivingOcean",
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0
    );
    bind_raw!(
        v,
        3026,
        79,
        "ArmorReflectiveColor",
        1.0,
        1.0,
        1.0,
        0.0,
        0.0,
        0.0,
        1.0
    );
    bind_raw!(
        v,
        3027,
        80,
        "ArmorReflectiveColor",
        1.5,
        1.2,
        0.5,
        0.0,
        0.0,
        0.0,
        1.0
    );
    bind_raw!(
        v,
        3553,
        81,
        "ArmorReflectiveColor",
        1.35,
        0.7,
        0.4,
        0.0,
        0.0,
        0.0,
        1.0
    );
    bind_raw!(
        v,
        3554,
        82,
        "ArmorReflectiveColor",
        0.25,
        0.0,
        0.7,
        0.0,
        0.0,
        0.0,
        1.0
    );
    bind_raw!(
        v,
        3555,
        83,
        "ArmorReflectiveColor",
        0.4,
        0.4,
        0.4,
        0.0,
        0.0,
        0.0,
        1.0
    );
    bind_raw!(
        v,
        3190,
        84,
        "ArmorReflective",
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0
    );
    // `TeamArmorShaderData` binds pass "ArmorColored" but overrides the color
    // per team at apply time — recorded unimplemented (no teams in the port).
    bind_raw!(
        v,
        1969,
        85,
        "ArmorColored",
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0
    );
    bind_raw!(
        v,
        2864,
        86,
        "ArmorMartian",
        0.0,
        2.0,
        3.0,
        0.0,
        0.0,
        0.0,
        1.0
    );
    bind_raw!(
        v,
        2872,
        87,
        "ArmorInvert",
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0
    );
    bind_raw!(
        v,
        2878,
        88,
        "ArmorWisp",
        0.7,
        1.0,
        0.9,
        0.35,
        0.85,
        0.8,
        1.0
    );
    bind_raw!(v, 2879, 89, "ArmorWisp", 1.0, 1.2, 0.0, 1.0, 0.6, 0.3, 1.0);
    bind_raw!(v, 2885, 90, "ArmorWisp", 1.2, 0.8, 0.0, 0.8, 0.2, 0.0, 1.0);
    bind_raw!(v, 2884, 91, "ArmorWisp", 1.0, 0.0, 1.0, 1.0, 0.3, 0.6, 1.0);
    bind_raw!(
        v,
        2883,
        92,
        "ArmorHighContrastGlow",
        0.0,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0
    );
    bind_raw!(v, 3025, 93, "ArmorFlow", 1.0, 0.5, 1.0, 0.6, 0.1, 1.0, 1.0);
    bind_raw!(
        v,
        3039,
        94,
        "ArmorTwilight",
        0.5,
        0.1,
        1.0,
        0.0,
        0.0,
        0.0,
        1.0
    );
    bind_raw!(v, 3040, 95, "ArmorAcid", 0.5, 1.0, 0.3, 0.0, 0.0, 0.0, 1.0);
    bind_raw!(
        v,
        3041,
        96,
        "ArmorMushroom",
        0.05,
        0.2,
        1.0,
        0.0,
        0.0,
        0.0,
        1.0
    );
    bind_raw!(v, 3042, 97, "ArmorPhase", 0.4, 0.2, 1.5, 0.0, 0.0, 0.0, 1.0);
    bind_raw!(v, 3560, 98, "ArmorAcid", 0.9, 0.2, 0.2, 0.0, 0.0, 0.0, 1.0);
    bind_raw!(v, 3561, 99, "ArmorGel", 0.4, 0.7, 1.4, 0.0, 0.0, 0.1, 1.0);
    bind_raw!(v, 3562, 100, "ArmorGel", 1.4, 0.75, 1.0, 0.45, 0.1, 0.3, 1.0);
    bind_raw!(v, 3024, 101, "ArmorGel", -0.5, -1.0, 0.0, 1.5, 1.0, 2.2, 1.0);
    bind_raw!(v, 4663, 102, "ArmorGel", 2.6, 0.6, 0.6, 0.2, -0.2, -0.2, 1.0);
    bind_raw!(v, 4662, 103, "ArmorFog", 0.95, 0.95, 0.95, 0.3, 0.3, 0.3, 1.0);
    bind_raw!(
        v,
        4778,
        104,
        "ArmorHallowBoss",
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0
    );
    bind_raw!(
        v,
        3534,
        105,
        "ArmorMirage",
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0
    );
    bind_raw!(v, 3028, 106, "ArmorAcid", 0.5, 0.7, 1.5, 0.0, 0.0, 0.0, 1.0);
    bind_raw!(
        v,
        3557,
        107,
        "ArmorPolarized",
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0
    );
    bind_raw!(v, 3978, 108, "ColorOnly", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    bind_raw!(
        v,
        3038,
        109,
        "ArmorHades",
        0.5,
        0.7,
        1.3,
        0.5,
        0.7,
        1.3,
        1.0
    );
    bind_raw!(
        v,
        3600,
        110,
        "ArmorHades",
        0.7,
        0.4,
        1.5,
        0.7,
        0.4,
        1.5,
        1.0
    );
    bind_raw!(
        v,
        3597,
        111,
        "ArmorHades",
        1.5,
        0.6,
        0.4,
        1.5,
        0.6,
        0.4,
        1.0
    );
    bind_raw!(
        v,
        3598,
        112,
        "ArmorHades",
        0.1,
        0.1,
        0.1,
        0.4,
        0.05,
        0.025,
        1.0
    );
    bind_raw!(v, 3599, 113, "ArmorLoki", 0.1, 0.1, 0.1, 0.0, 0.0, 0.0, 1.0);
    bind_raw!(
        v,
        3533,
        114,
        "ArmorShiftingSands",
        1.1,
        1.0,
        0.5,
        0.7,
        0.5,
        0.3,
        1.0
    );
    bind_raw!(
        v,
        3535,
        115,
        "ArmorShiftingPearlsands",
        1.1,
        0.8,
        0.9,
        0.35,
        0.25,
        0.44,
        1.0
    );
    bind_raw!(
        v,
        3526,
        116,
        "ArmorSolar",
        1.0,
        0.0,
        0.0,
        1.0,
        1.0,
        0.0,
        1.0
    );
    bind_raw!(
        v,
        3527,
        117,
        "ArmorNebula",
        1.0,
        0.0,
        1.0,
        1.0,
        1.0,
        1.0,
        1.0
    );
    bind_raw!(
        v,
        3528,
        118,
        "ArmorVortex",
        0.1,
        0.5,
        0.35,
        1.0,
        1.0,
        1.0,
        1.0
    );
    bind_raw!(
        v,
        3529,
        119,
        "ArmorStardust",
        0.4,
        0.6,
        1.0,
        1.0,
        1.0,
        1.0,
        1.0
    );
    bind_raw!(v, 3530, 120, "ArmorVoid", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    v
}

static REGISTRY: std::sync::OnceLock<Vec<DyeEntry>> = std::sync::OnceLock::new();

fn registry() -> &'static [DyeEntry] {
    REGISTRY.get_or_init(build_registry)
}

/// One uniform-table row (`sprite.wgsl`'s `DyeParam`): the core selector plus
/// the two colors, 16-byte aligned for the buffer upload.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DyeParamGpu {
    /// 1 = colored wash, 2 = gradient (the `sprite.wgsl` dispatch).
    pub(crate) mode: u32,
    pub(crate) _pad: [u32; 3],
    /// rgb + unused w.
    pub(crate) color: [f32; 4],
    pub(crate) secondary: [f32; 4],
}

/// The uniform table's hard cap (`sprite.rs` uploads must fit; 36 deduped
/// vanilla slots leave plenty of headroom).
pub(crate) const MAX_DYE_SLOTS: usize = 48;

static PARAMS: std::sync::OnceLock<Vec<DyeParamGpu>> = std::sync::OnceLock::new();

/// The deduped parameter slots: index `i` is the per-instance dye value
/// `i + 1` (0 stays "no dye"). Two bindings share a slot when their core and
/// both colors agree after the saturation fold.
fn build_params() -> Vec<DyeParamGpu> {
    let mut out: Vec<DyeParamGpu> = Vec::new();
    for e in registry() {
        let (mode, color, secondary) = match e.core {
            DyeCore::Colored => (1u32, dye_color(e), [0.0; 4]),
            DyeCore::Gradient => (2u32, dye_color(e), dye_secondary(e)),
            DyeCore::Unimplemented => continue,
        };
        if out
            .iter()
            .any(|p| p.mode == mode && p.color == color && p.secondary == secondary)
        {
            continue;
        }
        out.push(DyeParamGpu {
            mode,
            _pad: [0; 3],
            color,
            secondary,
        });
    }
    assert!(out.len() <= MAX_DYE_SLOTS, "dye param slots overflow");
    out
}

/// The entry's `UseColor` — the saturation folds away (a no-op on every
/// vanilla color; pinned by `tests::vanilla_dye_colors_are_saturated`).
fn dye_color(e: &DyeEntry) -> [f32; 4] {
    let [r, g, b] = e.color;
    [r, g, b, 1.0]
}

fn dye_secondary(e: &DyeEntry) -> [f32; 4] {
    let [r, g, b] = e.secondary;
    [r, g, b, 1.0]
}

/// The deduped uniform table, in first-binding order.
pub(crate) fn dye_params() -> &'static [DyeParamGpu] {
    PARAMS.get_or_init(build_params)
}

/// `GameShaders.Armor.GetShaderIdFromItemId(type)` (`Item.cs:48713`) mapped
/// onto the port's uniform slots: 0 = not a dye (or an unimplemented pass —
/// both render plain), else the 1-based `dye_params()` index. The per-tick
/// `c*` cache and the per-instance render channel both carry this value.
pub(crate) fn shader_id_for(kind: u16) -> u32 {
    let Some(e) = registry().iter().find(|e| e.item == kind) else {
        return 0;
    };
    let (mode, color, secondary) = match e.core {
        DyeCore::Colored => (1u32, dye_color(e), [0.0; 4]),
        DyeCore::Gradient => (2u32, dye_color(e), dye_secondary(e)),
        DyeCore::Unimplemented => return 0,
    };
    dye_params()
        .iter()
        .position(|p| p.mode == mode && p.color == color && p.secondary == secondary)
        .map_or(0, |i| i as u32 + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// DW-1.1: the registry mirrors `LoadBasicColorDyes` + `LoadArmorDyes`
    /// call-for-call — 13×4 basic + 68 explicit bindings, unique items, the
    /// C# ids contiguous from 1.
    #[test]
    fn registry_mirrors_load_armordyes() {
        let reg = registry();
        assert_eq!(reg.len(), 120);
        for (i, e) in reg.iter().enumerate() {
            assert_eq!(e.csharp_id as usize, i + 1, "binding order at {}", e.item);
        }
        let mut items: Vec<u16> = reg.iter().map(|e| e.item).collect();
        items.sort_unstable();
        items.dedup();
        assert_eq!(items.len(), reg.len(), "duplicate item binding");
        // The basic family arithmetic: Red Dye 1007, "and Black" +12, Bright
        // +31, "and Silver" +44 (`LoadBasicColorDye`, `DyeInitializer.cs:9-23`).
        let red = |item: u16| reg.iter().find(|e| e.item == item).unwrap();
        assert_eq!(red(1007).csharp_pass, "ArmorColored");
        assert_eq!(red(1019).csharp_pass, "ArmorColoredAndBlack");
        assert_eq!(red(1038).csharp_pass, "ArmorColored");
        assert_eq!(red(1038).color, [1.0, 0.5, 0.5], "bright = r*0.5+0.5");
        assert_eq!(red(1051).csharp_pass, "ArmorColoredAndSilverTrim");
        // The brown family binds its ids explicitly (`:37`).
        assert_eq!(red(2876).csharp_pass, "ArmorColored");
        assert_eq!(red(2876).color, [0.7, 0.6, 0.5]);
        // Pinned explicit entries (`:48-52`, `:140`).
        assert_eq!(red(2871).csharp_pass, "ArmorBrightnessColored");
        assert_eq!(red(2871).color, [0.05, 0.05, 0.05]);
        assert_eq!(red(3530).csharp_pass, "ArmorVoid");
        assert_eq!(red(3530).core, DyeCore::Unimplemented);
    }

    /// DW-1.2: `shader_id_for` dedupes equal (core, color, secondary)
    /// bindings into one slot and returns 0 for non-dyes.
    #[test]
    fn shader_ids_dedupe_and_default_to_zero() {
        assert_eq!(shader_id_for(89), 0, "Copper Helmet is not a dye");
        assert_eq!(shader_id_for(0), 0);
        // Silver Dye (1037) and the white "and Black" combo (3559) share the
        // white Colored slot.
        assert_ne!(shader_id_for(1037), 0);
        assert_eq!(shader_id_for(1037), shader_id_for(3559));
        // All four red-gradient passes share one slot (`:58-61`).
        let red_grad = shader_id_for(1031);
        assert_ne!(red_grad, 0);
        assert_eq!(red_grad, shader_id_for(1032));
        assert_eq!(red_grad, shader_id_for(3550));
        assert_eq!(red_grad, shader_id_for(1063));
        // Bright Red (1038) differs from Red (1007).
        assert_ne!(shader_id_for(1038), shader_id_for(1007));
        // Unimplemented passes render plain (slot 0)…
        assert_eq!(shader_id_for(1066), 0, "Rainbow is not ported yet");
        // …while every Colored/Gradient binding gets a live slot.
        assert!(registry()
            .iter()
            .filter(|e| !matches!(e.core, DyeCore::Unimplemented))
            .all(|e| shader_id_for(e.item) > 0));
    }

    /// DW-1.3 (dirty): the saturation fold is a no-op on the whole registry —
    /// for every binding either the color is already fully saturated or its
    /// `UseSaturation` is the 1.0 default (the only S<1 colors are the brown
    /// bright variants, bound with the default saturation), so the port can
    /// drop the uniform without a pixel change.
    #[test]
    fn vanilla_dye_colors_are_saturated() {
        let saturated = |c: [f32; 3]| {
            let [r, g, b] = c.map(|v: f32| v.clamp(0.0, 1.0));
            let max = r.max(g).max(b);
            let min = r.min(g).min(b);
            let l = (max + min) / 2.0;
            if l <= 0.0 || l >= 1.0 {
                return true; // black/white cannot saturate further
            }
            let s0 = if l > 0.5 {
                (max - min) / (2.0 - max - min)
            } else {
                (max - min) / (max + min)
            };
            s0 >= 0.999
        };
        for e in registry() {
            if e.saturation == 1.0 {
                continue; // the fold is trivially a no-op here
            }
            assert!(saturated(e.color), "item {} color {:?}", e.item, e.color);
            assert!(
                saturated(e.secondary),
                "item {} secondary {:?}",
                e.item,
                e.secondary
            );
        }
    }

    /// The uniform table stays inside the buffer cap and keeps its contract:
    /// slot 1 exists, modes are 1/2, `shader_id_for` round-trips through it.
    #[test]
    fn param_table_fits_and_round_trips() {
        let params = dye_params();
        assert!(!params.is_empty());
        assert!(params.len() <= MAX_DYE_SLOTS);
        for p in params {
            assert!(matches!(p.mode, 1 | 2));
        }
        for e in registry()
            .iter()
            .filter(|e| !matches!(e.core, DyeCore::Unimplemented))
        {
            let id = shader_id_for(e.item);
            let p = &params[(id - 1) as usize];
            assert_eq!(p.color[3], 1.0);
        }
    }
}
